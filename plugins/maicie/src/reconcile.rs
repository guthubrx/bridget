//! Reprise déterministe des délégations durablement préparées.
//!
//! Ce module ne réinterprète jamais une délégation et ne reconstruit aucun
//! payload. Il consulte d'abord Bridget dans la portée durable de l'outbox,
//! puis ne rejoue que l'enveloppe filaire strictement identique enregistrée
//! avant la première I/O.

use crate::bridget_client::{
    BridgetClient, BridgetClientError, BridgetClientLimits, IdempotencyIssue, SpawnOutcome,
};
use crate::outbox::{OutboxError, PendingDelegationOutbox};
use crate::profiles::definition_digest_matches;
use crate::store::{DelegationRecoveryEntry, LocalFailureReason, MaicieStore, StoreError};
use serde_json::json;
use std::fmt;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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
    /// Bridget était indisponible avant toute écriture ; l'outbox reste prepared.
    TransportIndisponible {
        objective_id: Uuid,
        message_id: Uuid,
    },
    /// Les octets durables sont invalides localement : le store les a figés
    /// comme rejetés, sans les transmettre ni les assimiler à un refus Bridget.
    RejetLocal {
        objective_id: Uuid,
        message_id: Uuid,
        reason: LocalFailureReason,
    },
}

/// Résultat d'une passe explicite de démarrage. Maicie n'exécute aucun timer.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    pub actions: Vec<ReconcileAction>,
}

/// Conséquence d'une passe de reprise des activations approuvées. Le rejet de
/// digest est terminal et visible : Maicie ne refait jamais approuver ni ne
/// génère un nouveau command_id de sa propre initiative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationReconcileAction {
    Rejouee { command_id: Uuid },
    IssueEnCours { command_id: Uuid },
    IssueTerminale { command_id: Uuid },
    TransportIncertain { command_id: Uuid },
    TransportIndisponible { command_id: Uuid },
    DefinitionDivergente { command_id: Uuid },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActivationReconcileReport {
    pub actions: Vec<ActivationReconcileAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationReconcilePhase {
    BeforeSocket,
    AfterIssueBeforeStoreCommit,
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
    reconcile_startup_with_limits(store, bridget_socket, BridgetClientLimits::default())
}

/// Variante runtime : la même borne de trame sert à préparer puis à rejouer
/// l'enveloppe, afin qu'une reprise ne devienne jamais plus restrictive.
pub fn reconcile_startup_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    limits: BridgetClientLimits,
) -> Result<ReconcileReport, ReconcileError> {
    let observed_at = unix_now()?;
    reconcile_startup_at_with_limits(store, bridget_socket, observed_at, limits)
}

/// Variante déterministe pour les tests et les appels qui possèdent déjà une
/// horloge. Aucun délai local n'est programmé par ce module.
pub fn reconcile_startup_at(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
) -> Result<ReconcileReport, ReconcileError> {
    reconcile_startup_at_with_limits(
        store,
        bridget_socket,
        observed_at,
        BridgetClientLimits::default(),
    )
}

pub fn reconcile_startup_at_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    limits: BridgetClientLimits,
) -> Result<ReconcileReport, ReconcileError> {
    reconcile_startup_at_observed_with_limits(
        store,
        bridget_socket,
        observed_at,
        limits,
        |_| Ok(()),
    )
}

/// Même reprise avec des jalons qui ne servent qu'aux crash-tests réels.
pub fn reconcile_startup_at_observed(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    observer: impl FnMut(ReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<ReconcileReport, ReconcileError> {
    reconcile_startup_at_observed_with_limits(
        store,
        bridget_socket,
        observed_at,
        BridgetClientLimits::default(),
        observer,
    )
}

pub fn reconcile_startup_at_observed_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    limits: BridgetClientLimits,
    mut observer: impl FnMut(ReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<ReconcileReport, ReconcileError> {
    if observed_at <= 0 {
        return Err(ReconcileError::InvalidSnapshot("observed_at invalide"));
    }
    let socket = bridget_socket.as_ref();
    let mut report = ReconcileReport::default();
    let deadline = Instant::now() + reconciliation_budget(limits);
    for entry in store.delegation_recovery_entries()? {
        let Some(_) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        let action = match entry {
            DelegationRecoveryEntry::Pending(outbox) => {
                observer(ReconcilePhase::BeforeSocket)?;
                reconcile_one(
                    store,
                    socket,
                    &outbox,
                    observed_at,
                    limits,
                    deadline,
                    &mut observer,
                )?
            }
            DelegationRecoveryEntry::LocalFailure {
                objective_id,
                message_id,
                reason,
                ..
            } => {
                store.record_local_failure_at(message_id, reason, observed_at)?;
                ReconcileAction::RejetLocal {
                    objective_id,
                    message_id,
                    reason,
                }
            }
        };
        let socket_unavailable = matches!(
            action,
            ReconcileAction::TransportIndisponible { .. }
                | ReconcileAction::TransportIncertain { .. }
        );
        report.actions.push(action);
        // Toutes les lignes visent le même socket. Après une indisponibilité,
        // retenter chaque outbox ne produit aucune information supplémentaire
        // et transformerait une commande CLI en boucle O(N × délai).
        if socket_unavailable {
            break;
        }
    }
    Ok(report)
}

/// Reprend les SpawnOrder non terminaux. Pour cette frontière, le replay des
/// octets approuvés est volontairement le lookup : 009 ne publie aucun lookup
/// séparé dans la portée interne du superviseur. Une issue durable est alors
/// rejouée par Bridget sans nouveau lancement.
pub fn reconcile_activation_startup_at(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
) -> Result<ActivationReconcileReport, ReconcileError> {
    reconcile_activation_startup_at_observed(store, bridget_socket, observed_at, |_| Ok(()))
}

/// Variante réservée aux crash-tests : les jalons encadrent les bytes
/// durablement préparés et l'issue Bridget avant sa transaction locale.
pub fn reconcile_activation_startup_at_observed(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    mut observer: impl FnMut(ActivationReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<ActivationReconcileReport, ReconcileError> {
    if observed_at <= 0 {
        return Err(ReconcileError::InvalidSnapshot("observed_at invalide"));
    }
    let socket = bridget_socket.as_ref();
    let mut report = ActivationReconcileReport::default();
    for pending in store.pending_activation_outboxes()? {
        let activation = &pending.activation;
        let command_id = activation.command_id;
        let action = if observed_at >= activation.retry_until {
            record_activation_terminal(
                store,
                command_id,
                SpawnOutcome::Rejected {
                    command_id: command_id.to_string(),
                    reason: json!({"kind":"idempotency_expired"}),
                },
                observed_at,
            )?;
            ActivationReconcileAction::IssueTerminale { command_id }
        } else {
            observer(ActivationReconcilePhase::BeforeSocket)?;
            match BridgetClient::replay_spawn_order_bytes_at(
                socket,
                BridgetClientLimits::default(),
                &activation.spawn_order_bytes,
            ) {
                Ok(replay) => match replay.outcome {
                    SpawnOutcome::Accepted { .. }
                        if definition_digest_matches(
                            &pending.approval.context_hash,
                            replay.definition_digest.as_deref().unwrap_or_default(),
                        ) =>
                    {
                        observer(ActivationReconcilePhase::AfterIssueBeforeStoreCommit)?;
                        record_activation_terminal(store, command_id, replay.outcome, observed_at)?;
                        ActivationReconcileAction::IssueTerminale { command_id }
                    }
                    SpawnOutcome::Accepted { .. } => {
                        observer(ActivationReconcilePhase::AfterIssueBeforeStoreCommit)?;
                        record_activation_terminal(
                            store,
                            command_id,
                            SpawnOutcome::Rejected {
                                command_id: command_id.to_string(),
                                reason: json!({"kind":"definition_digest_mismatch"}),
                            },
                            observed_at,
                        )?;
                        ActivationReconcileAction::DefinitionDivergente { command_id }
                    }
                    SpawnOutcome::Idempotency(IdempotencyIssue::OutcomeUnknown { .. }) => {
                        observer(ActivationReconcilePhase::AfterIssueBeforeStoreCommit)?;
                        store.record_activation_outcome(
                            command_id,
                            &replay.outcome,
                            observed_at,
                        )?;
                        ActivationReconcileAction::IssueEnCours { command_id }
                    }
                    outcome => {
                        observer(ActivationReconcilePhase::AfterIssueBeforeStoreCommit)?;
                        record_activation_terminal(store, command_id, outcome, observed_at)?;
                        ActivationReconcileAction::IssueTerminale { command_id }
                    }
                },
                Err(BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. }) => {
                    ActivationReconcileAction::TransportIndisponible { command_id }
                }
                Err(error) if transport_is_ambiguous(&error) => {
                    let unknown = SpawnOutcome::Idempotency(IdempotencyIssue::OutcomeUnknown {
                        expires_at: activation.dedup_retained_until,
                        delivery_id: None,
                    });
                    store.record_activation_outcome(command_id, &unknown, observed_at)?;
                    ActivationReconcileAction::TransportIncertain { command_id }
                }
                Err(error) => return Err(ReconcileError::Client(error)),
            }
        };
        let unavailable = matches!(
            action,
            ActivationReconcileAction::TransportIndisponible { .. }
                | ActivationReconcileAction::TransportIncertain { .. }
        );
        report.actions.push(action);
        if unavailable {
            break;
        }
    }
    Ok(report)
}

fn record_activation_terminal(
    store: &mut MaicieStore,
    command_id: Uuid,
    outcome: SpawnOutcome,
    observed_at: i64,
) -> Result<(), ReconcileError> {
    store.record_activation_outcome(command_id, &outcome, observed_at)?;
    Ok(())
}

/// Borne une passe d'ouverture entière, pas chaque outbox séparément.
/// Les échanges locaux sains restent quasi immédiats ; un daemon absent ne
/// consomme jamais davantage qu'un délai client.
fn reconciliation_budget(limits: BridgetClientLimits) -> Duration {
    limits.connect_timeout.max(limits.io_timeout)
}

fn reconcile_one(
    store: &mut MaicieStore,
    socket: &Path,
    outbox: &PendingDelegationOutbox,
    observed_at: i64,
    limits: BridgetClientLimits,
    deadline: Instant,
    observer: &mut impl FnMut(ReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<ReconcileAction, ReconcileError> {
    outbox.validate()?;
    if outbox.issuer_scope != store.issuer_scope() {
        return Err(ReconcileError::InvalidSnapshot(
            "issuer_scope de l'outbox différent du store",
        ));
    }
    let message_id = outbox.message_id.to_string();
    let mut client = match BridgetClient::connect_with_limits_until(
        socket,
        &outbox.issuer_scope,
        limits,
        deadline,
    ) {
        Ok(client) => client,
        Err(error) => return unavailable_or_error(outbox, error),
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
    if let Some(reason) = local_failure_reason(&error) {
        store.record_local_failure_at(outbox.message_id, reason, observed_at)?;
        return Ok(ReconcileAction::RejetLocal {
            objective_id: outbox.objective_id,
            message_id: outbox.message_id,
            reason,
        });
    }
    if !transport_is_ambiguous(&error) {
        return Err(ReconcileError::Client(error));
    }
    store.record_transport_uncertainty(outbox.message_id, observed_at)?;
    Ok(ReconcileAction::TransportIncertain {
        objective_id: outbox.objective_id,
        message_id: outbox.message_id,
    })
}

fn local_failure_reason(error: &BridgetClientError) -> Option<LocalFailureReason> {
    match error {
        BridgetClientError::FrameTooLarge { .. } => Some(LocalFailureReason::FrameTooLarge),
        BridgetClientError::InvalidEnvelope(_) => Some(LocalFailureReason::InvalidEnvelope),
        _ => None,
    }
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
        BridgetClientError::Read(_)
            | BridgetClientError::Write(_)
            | BridgetClientError::Decode { .. }
            | BridgetClientError::Closed
            | BridgetClientError::Protocol(_)
            | BridgetClientError::Timeout { .. }
    )
}

fn unavailable_or_error(
    outbox: &PendingDelegationOutbox,
    error: BridgetClientError,
) -> Result<ReconcileAction, ReconcileError> {
    match error {
        BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. } => {
            Ok(ReconcileAction::TransportIndisponible {
                objective_id: outbox.objective_id,
                message_id: outbox.message_id,
            })
        }
        error => Err(ReconcileError::Client(error)),
    }
}

fn unix_now() -> Result<i64, ReconcileError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ReconcileError::Clock)
        .and_then(|duration| i64::try_from(duration.as_secs()).map_err(|_| ReconcileError::Clock))
}
