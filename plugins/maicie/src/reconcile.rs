//! Reprise déterministe des délégations durablement préparées.
//!
//! Ce module ne réinterprète jamais une délégation et ne reconstruit aucun
//! payload. Il consulte d'abord Bridget dans la portée durable de l'outbox,
//! puis ne rejoue que l'enveloppe filaire strictement identique enregistrée
//! avant la première I/O.

use crate::bridget_client::{BridgetClient, BridgetClientError, IdempotencyIssue};
use crate::outbox::{OutboxError, PendingDelegationOutbox};
use crate::store::{MaicieStore, StoreError};
use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Frontières observables des crash-tests de reprise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcilePhase {
    /// L'outbox est durable mais aucune I/O Bridget n'a commencé.
    BeforeSocket,
    /// Bridget a renvoyé une issue à l'envoi, sans commit local correspondant.
    AfterIssueBeforeStoreCommit,
}

/// Conséquence factuelle d'une ligne d'outbox pendant une passe de reprise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileAction {
    /// Une issue terminale existait déjà chez Bridget ; aucun replay n'a eu lieu.
    IssueTerminale {
        objective_id: Uuid,
        message_id: Uuid,
        issue: IdempotencyIssue,
    },
    /// Bridget connaît encore une remise en cours ; Maicie attend sans réémettre.
    IssueEnCours {
        objective_id: Uuid,
        message_id: Uuid,
        issue: IdempotencyIssue,
    },
    /// Aucune issue n'existait dans l'horizon valide : l'enveloppe exacte a été rejouée.
    Rejouee {
        objective_id: Uuid,
        message_id: Uuid,
        issue: IdempotencyIssue,
    },
    /// La frontière réseau est ambiguë ; la ligne reste éligible au prochain lookup.
    TransportIncertain {
        objective_id: Uuid,
        message_id: Uuid,
    },
}

/// Résultat d'une passe explicite de démarrage. Maicie n'exécute aucun timer.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    pub actions: Vec<ReconcileAction>,
}

/// Erreur non ambiguë : elle ne doit pas être transformée en nouvel envoi.
#[derive(Debug)]
pub enum ReconcileError {
    Store(StoreError),
    Outbox(OutboxError),
    Client(BridgetClientError),
    InvalidSnapshot(&'static str),
    Clock,
}

impl fmt::Display for ReconcileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "store Maicie impossible : {error}"),
            Self::Outbox(error) => write!(formatter, "outbox Maicie invalide : {error}"),
            Self::Client(error) => write!(formatter, "contrat Bridget invalide : {error}"),
            Self::InvalidSnapshot(reason) => {
                write!(formatter, "snapshot de reprise invalide : {reason}")
            }
            Self::Clock => formatter.write_str("horloge système antérieure à l'époque Unix"),
        }
    }
}

impl std::error::Error for ReconcileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Outbox(error) => Some(error),
            Self::Client(error) => Some(error),
            Self::InvalidSnapshot(_) | Self::Clock => None,
        }
    }
}

impl From<StoreError> for ReconcileError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<OutboxError> for ReconcileError {
    fn from(error: OutboxError) -> Self {
        Self::Outbox(error)
    }
}

/// Reprend les outboxes non terminales lors d'un démarrage explicite.
pub fn reconcile_startup(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
) -> Result<ReconcileReport, ReconcileError> {
    let observed_at = unix_now()?;
    reconcile_startup_at(store, bridget_socket, observed_at)
}

/// Variante déterministe pour les tests et les appels qui possèdent déjà une
/// horloge. Aucun délai local n'est programmé par ce module.
pub fn reconcile_startup_at(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
) -> Result<ReconcileReport, ReconcileError> {
    reconcile_startup_at_observed(store, bridget_socket, observed_at, |_| Ok(()))
}

/// Même reprise avec des jalons qui ne servent qu'aux crash-tests réels.
pub fn reconcile_startup_at_observed(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    mut observer: impl FnMut(ReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<ReconcileReport, ReconcileError> {
    if observed_at <= 0 {
        return Err(ReconcileError::InvalidSnapshot("observed_at invalide"));
    }
    let socket = bridget_socket.as_ref();
    let mut report = ReconcileReport::default();
    for outbox in store.pending_delegation_outboxes()? {
        observer(ReconcilePhase::BeforeSocket)?;
        let action = reconcile_one(store, socket, &outbox, observed_at, &mut observer)?;
        report.actions.push(action);
    }
    Ok(report)
}

fn reconcile_one(
    store: &mut MaicieStore,
    socket: &Path,
    outbox: &PendingDelegationOutbox,
    observed_at: i64,
    observer: &mut impl FnMut(ReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<ReconcileAction, ReconcileError> {
    outbox.validate()?;
    if outbox.issuer_scope != store.issuer_scope() {
        return Err(ReconcileError::InvalidSnapshot(
            "issuer_scope de l'outbox différent du store",
        ));
    }
    let message_id = outbox.message_id.to_string();
    let mut client = match BridgetClient::connect(socket, &outbox.issuer_scope) {
        Ok(client) => client,
        Err(error) => return uncertain_or_error(store, outbox, observed_at, error),
    };

    match client.lookup(&message_id) {
        Ok(IdempotencyIssue::IdempotencyExpired)
            if can_replay_absent(outbox, &client, observed_at) =>
        {
            replay_exact(store, outbox, observed_at, observer, &mut client)
        }
        Ok(issue) => record_known_issue(store, outbox, issue, observed_at),
        Err(error) => uncertain_or_error(store, outbox, observed_at, error),
    }
}

fn replay_exact(
    store: &mut MaicieStore,
    outbox: &PendingDelegationOutbox,
    observed_at: i64,
    observer: &mut impl FnMut(ReconcilePhase) -> Result<(), ReconcileError>,
    client: &mut BridgetClient,
) -> Result<ReconcileAction, ReconcileError> {
    match client.replay_idempotent_bytes(
        &outbox.message_bytes,
        &outbox.message_id.to_string(),
        outbox.issued_at,
    ) {
        Ok(issue) => {
            observer(ReconcilePhase::AfterIssueBeforeStoreCommit)?;
            store.record_lookup_issue(outbox.message_id, &issue, observed_at)?;
            Ok(ReconcileAction::Rejouee {
                objective_id: outbox.objective_id,
                message_id: outbox.message_id,
                issue,
            })
        }
        Err(error) => uncertain_or_error(store, outbox, observed_at, error),
    }
}

fn record_known_issue(
    store: &mut MaicieStore,
    outbox: &PendingDelegationOutbox,
    issue: IdempotencyIssue,
    observed_at: i64,
) -> Result<ReconcileAction, ReconcileError> {
    store.record_lookup_issue(outbox.message_id, &issue, observed_at)?;
    let action = match issue {
        IdempotencyIssue::OutcomeUnknown { .. } => ReconcileAction::IssueEnCours {
            objective_id: outbox.objective_id,
            message_id: outbox.message_id,
            issue,
        },
        issue => ReconcileAction::IssueTerminale {
            objective_id: outbox.objective_id,
            message_id: outbox.message_id,
            issue,
        },
    };
    Ok(action)
}

fn uncertain_or_error(
    store: &mut MaicieStore,
    outbox: &PendingDelegationOutbox,
    observed_at: i64,
    error: BridgetClientError,
) -> Result<ReconcileAction, ReconcileError> {
    if !transport_is_ambiguous(&error) {
        return Err(ReconcileError::Client(error));
    }
    store.record_transport_uncertainty(outbox.message_id, observed_at)?;
    Ok(ReconcileAction::TransportIncertain {
        objective_id: outbox.objective_id,
        message_id: outbox.message_id,
    })
}

fn can_replay_absent(
    outbox: &PendingDelegationOutbox,
    client: &BridgetClient,
    observed_at: i64,
) -> bool {
    let horizon_ends = outbox
        .issued_at
        .checked_add(client.negotiated().horizon_secs)
        .unwrap_or(i64::MIN);
    observed_at < outbox.retry_until && observed_at < horizon_ends
}

fn transport_is_ambiguous(error: &BridgetClientError) -> bool {
    matches!(
        error,
        BridgetClientError::Connect { .. }
            | BridgetClientError::Read(_)
            | BridgetClientError::Write(_)
            | BridgetClientError::Decode { .. }
            | BridgetClientError::Closed
            | BridgetClientError::Protocol(_)
            | BridgetClientError::Timeout { .. }
            | BridgetClientError::FrameTooLarge { .. }
    )
}

fn unix_now() -> Result<i64, ReconcileError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ReconcileError::Clock)
        .and_then(|duration| i64::try_from(duration.as_secs()).map_err(|_| ReconcileError::Clock))
}
