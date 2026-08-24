//! Reprise déterministe des délégations durablement préparées.
//!
//! Ce module ne réinterprète jamais une délégation et ne reconstruit aucun
//! payload. Il consulte d'abord Bridget dans la portée durable de l'outbox,
//! puis ne rejoue que l'enveloppe filaire strictement identique enregistrée
//! avant la première I/O.

use crate::app::{
    GuichetError, apply_attested_coordination_event, process_guichet_claim,
    record_guichet_lifecycle_event,
};
use crate::bridget_client::{
    BridgetClient, BridgetClientError, BridgetClientLimits, CoordinationClient,
    CoordinationStreamItem, GuichetClient, IdempotencyIssue, SpawnOutcome,
};
use crate::domain::{MotifRefusGreffe, NotificationOutbox};
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

/// Conséquence factuelle d'une notification de clôture pendant une passe
/// bornée. Les octets et l'identité viennent exclusivement de l'outbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationReconcileAction {
    Issue {
        objective_id: Uuid,
        message_id: Uuid,
        issue: IdempotencyIssue,
    },
    TransportIndisponible {
        objective_id: Uuid,
        message_id: Uuid,
    },
    TransportIncertain {
        objective_id: Uuid,
        message_id: Uuid,
    },
    BudgetEpuise,
}

/// Résultat d'une relève de notifications. Une commande ne conserve jamais
/// ce rapport : il décrit seulement les faits de sa passe pull-only.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NotificationReconcileReport {
    pub actions: Vec<NotificationReconcileAction>,
}

/// Frontières de crash du dispatch F27. Elles restent absentes du chemin CLI
/// normal et servent seulement à tuer un processus de test à un état précis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationReconcilePhase {
    BeforeSocket,
    AfterWriteBeforeAck,
    AfterIssueBeforeStoreCommit,
}

/// Fait constaté pendant une relève pull-only du guichet. Cette projection ne
/// devient jamais un runtime : chaque commande lui fournit une échéance unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuichetReconcileAction {
    ReponseAttestee { request_id: String, issue: String },
    RejetAtteste { request_id: String, reason: MotifRefusGreffe },
    EvenementAtteste { request_id: String, state: String },
    ClaimPerime { request_id: String },
    Vide,
    BudgetEpuise,
    TransportIndisponible,
    TransportIncertain,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GuichetReconcileReport {
    pub actions: Vec<GuichetReconcileAction>,
}

const MAX_COORDINATION_ITEMS_PER_PASS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinationReconcileAction {
    EvenementApplique {
        cursor: u64,
    },
    TerminalApplique {
        request_id: String,
        state: String,
    },
    SnapshotAtteint {
        through_cursor: Option<u64>,
    },
    Gap {
        from_cursor: u64,
        to_cursor: u64,
        reason: String,
    },
    Unavailable {
        reason: String,
    },
    TransportIndisponible,
    BudgetEpuise,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CoordinationReconcileReport {
    pub actions: Vec<CoordinationReconcileAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationReconcilePhase {
    BeforeApply,
    BeforeStoreCommit,
    AfterStoreCommit,
}

/// Jalons réservés aux crash-tests de la relève guichet. Ils encadrent la
/// frontière entre le claim Bridget, la greffe SQLite Maicie et la réponse
/// liée ; aucune commande de production ne les observe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuichetReconcilePhase {
    BeforeClaim,
    AfterClaimBeforeStoreCommit,
    AfterStoreCommitBeforeReply,
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
    Guichet(GuichetError),
    InvalidSnapshot(&'static str),
    Clock,
}

impl fmt::Display for ReconcileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "store Maicie impossible : {error}"),
            Self::Outbox(error) => write!(formatter, "outbox Maicie invalide : {error}"),
            Self::Client(error) => write!(formatter, "contrat Bridget invalide : {error}"),
            Self::Guichet(error) => write!(formatter, "traitement guichet impossible : {error}"),
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
            // `GuichetError` reste un contrat applicatif de lot B et
            // n'expose volontairement pas de chaîne d'erreurs technique.
            Self::Guichet(_) => None,
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

/// Rejoue les notifications de coordination sans jamais reconstruire leur
/// enveloppe. La requête envoyée est la projection filaire des seuls octets
/// durables de l'outbox ; un `OutcomeUnknown` demeure donc pending et sera
/// rejoué identiquement lors d'une passe ultérieure.
///
/// Une unique échéance couvre toute la relève. Après une indisponibilité ou
/// une frontière ambiguë, les lignes suivantes viseraient le même socket :
/// les retenter ne produirait aucune information supplémentaire et ferait
/// croître une commande CLI avec le nombre d'outboxes.
pub fn reconcile_notification_startup_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    limits: BridgetClientLimits,
) -> Result<NotificationReconcileReport, ReconcileError> {
    reconcile_notification_startup_observed_with_limits(store, bridget_socket, limits, |_| Ok(()))
}

/// Variante réservée aux crash-tests : les jalons entourent la seule
/// frontière I/O et le commit local de consommation, sans modifier les
/// octets envoyés ni la machine d'état de production.
#[doc(hidden)]
pub fn reconcile_notification_startup_observed_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    limits: BridgetClientLimits,
    mut observer: impl FnMut(NotificationReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<NotificationReconcileReport, ReconcileError> {
    let socket = bridget_socket.as_ref();
    let deadline = Instant::now() + reconciliation_budget(limits);
    let mut report = NotificationReconcileReport::default();

    for outbox in store.pending_notification_outboxes()? {
        if Instant::now() >= deadline {
            report
                .actions
                .push(NotificationReconcileAction::BudgetEpuise);
            break;
        }

        observer(NotificationReconcilePhase::BeforeSocket)?;
        let action =
            reconcile_notification_one(store, socket, &outbox, limits, deadline, &mut observer)?;
        let stop = matches!(
            action,
            NotificationReconcileAction::TransportIndisponible { .. }
                | NotificationReconcileAction::TransportIncertain { .. }
        );
        report.actions.push(action);
        if stop {
            break;
        }
    }

    Ok(report)
}

fn reconcile_notification_one(
    store: &mut MaicieStore,
    socket: &Path,
    outbox: &NotificationOutbox,
    limits: BridgetClientLimits,
    deadline: Instant,
    observer: &mut impl FnMut(NotificationReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<NotificationReconcileAction, ReconcileError> {
    outbox.verifier().map_err(|_| {
        ReconcileError::InvalidSnapshot("notification durable invalide avant la reprise")
    })?;

    let mut client = match BridgetClient::connect_with_limits_until(
        socket,
        store.issuer_scope(),
        limits,
        deadline,
    ) {
        Ok(client) => client,
        Err(BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. }) => {
            return Ok(NotificationReconcileAction::TransportIndisponible {
                objective_id: outbox.objectif_id,
                message_id: outbox.message_id,
            });
        }
        Err(error) if transport_is_ambiguous(&error) => {
            return Ok(NotificationReconcileAction::TransportIncertain {
                objective_id: outbox.objectif_id,
                message_id: outbox.message_id,
            });
        }
        Err(error) => return Err(ReconcileError::Client(error)),
    };

    let mut observer_error = None;
    let replay = client.replay_idempotent_bytes_observed(
        &outbox.message_bytes,
        &outbox.message_id.to_string(),
        outbox.issued_at,
        |_| {
            if let Err(error) = observer(NotificationReconcilePhase::AfterWriteBeforeAck) {
                observer_error = Some(error);
            }
        },
    );
    if let Some(error) = observer_error {
        return Err(error);
    }

    match replay {
        Ok(issue) => {
            observer(NotificationReconcilePhase::AfterIssueBeforeStoreCommit)?;
            store.record_notification_issue(outbox.message_id, &issue)?;
            Ok(NotificationReconcileAction::Issue {
                objective_id: outbox.objectif_id,
                message_id: outbox.message_id,
                issue,
            })
        }
        Err(BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. }) => {
            Ok(NotificationReconcileAction::TransportIndisponible {
                objective_id: outbox.objectif_id,
                message_id: outbox.message_id,
            })
        }
        Err(error) if transport_is_ambiguous(&error) => {
            Ok(NotificationReconcileAction::TransportIncertain {
                objective_id: outbox.objectif_id,
                message_id: outbox.message_id,
            })
        }
        Err(error) => Err(ReconcileError::Client(error)),
    }
}

/// Relève le guichet au début d'une commande sans conserver de connexion ni
/// de curseur après son retour. Tous les échanges consomment la même échéance
/// absolue : une boîte aux lettres indisponible ne peut donc pas transformer
/// une commande Maicie en boucle de polling ou en attente non bornée.
pub fn reconcile_guichet_startup_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    limits: BridgetClientLimits,
) -> Result<GuichetReconcileReport, ReconcileError> {
    reconcile_guichet_startup_observed_with_limits(
        store,
        bridget_socket,
        observed_at,
        limits,
        |_| Ok(()),
    )
}

/// Variante des crash-tests : les jalons ne modifient aucun état et servent
/// uniquement à interrompre un vrai processus aux trois frontières durables.
pub fn reconcile_guichet_startup_observed_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    observed_at: i64,
    limits: BridgetClientLimits,
    mut observer: impl FnMut(GuichetReconcilePhase) -> Result<(), ReconcileError>,
) -> Result<GuichetReconcileReport, ReconcileError> {
    if observed_at <= 0 {
        return Err(ReconcileError::InvalidSnapshot("observed_at invalide"));
    }
    let deadline = Instant::now() + reconciliation_budget(limits);
    let mut report = GuichetReconcileReport::default();
    let mut client = match GuichetClient::connect_with_limits_until(
        bridget_socket,
        store.issuer_scope(),
        limits,
        deadline,
    ) {
        Ok(client) => client,
        Err(BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. }) => {
            report
                .actions
                .push(GuichetReconcileAction::TransportIndisponible);
            return Ok(report);
        }
        Err(error) => return Err(ReconcileError::Client(error)),
    };

    loop {
        if Instant::now() >= deadline {
            report.actions.push(GuichetReconcileAction::BudgetEpuise);
            return Ok(report);
        }
        observer(GuichetReconcilePhase::BeforeClaim)?;
        let Some(claim) = (match client.claim_next() {
            Ok(claim) => claim,
            Err(error) => {
                record_guichet_transport_error(&mut report, error)?;
                return Ok(report);
            }
        }) else {
            report.actions.push(GuichetReconcileAction::Vide);
            return Ok(report);
        };

        observer(GuichetReconcilePhase::AfterClaimBeforeStoreCommit)?;

        // Le résultat applicatif est greffé avant toute réponse socket. Au
        // rejeu, l'app réutilise ou régénère le seul `reply_bytes` durable
        // pour le claim courant : ce réconciliateur ne reconstruit jamais de
        // réponse ni de token par lui-même.
        let response_message_id = Uuid::new_v4().to_string();
        let processed = process_guichet_claim(store, &claim, &response_message_id, observed_at)
            .map_err(ReconcileError::Guichet)?;
        observer(GuichetReconcilePhase::AfterStoreCommitBeforeReply)?;
        let response = match client.reply_exact_bytes(&processed.reply_bytes) {
            Ok(response) => response,
            Err(error) => {
                record_guichet_transport_error(&mut report, error)?;
                return Ok(report);
            }
        };
        if response.issue == "claim_stale" {
            report.actions.push(GuichetReconcileAction::ClaimPerime {
                request_id: claim.request_id,
            });
            continue;
        }
        if let Some(reason) = processed.refusal_reason {
            report.actions.push(GuichetReconcileAction::RejetAtteste {
                request_id: processed.request_id,
                reason,
            });
        } else {
            report
                .actions
                .push(GuichetReconcileAction::ReponseAttestee {
                    request_id: processed.request_id,
                    issue: response.issue,
                });
        }

        // Bridget pousse l'événement seulement après avoir rendu l'issue de
        // réponse durable. Son absence à l'échéance reste un fait transport :
        // la greffe locale déjà durable n'est ni annulée ni réinterprétée.
        match client.next_lifecycle_event() {
            Ok(event) => {
                let request_id = event.request_id.clone();
                let state = event.state.clone();
                record_guichet_lifecycle_event(store, &event).map_err(ReconcileError::Guichet)?;
                report
                    .actions
                    .push(GuichetReconcileAction::EvenementAtteste { request_id, state });
            }
            Err(BridgetClientError::Timeout { .. }) => {}
            Err(error) => {
                record_guichet_transport_error(&mut report, error)?;
                return Ok(report);
            }
        }
    }
}

/// Relève un snapshot cursé au début d'une commande, puis applique au plus une
/// fenêtre bornée de faits attestés. Les événements ne sont jamais frais avant
/// `SnapshotCaughtUp` : la passe les conserve donc en mémoire sans mutation,
/// puis les commit un par un dans l'ordre du curseur.
pub fn reconcile_coordination_startup_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    limits: BridgetClientLimits,
) -> Result<CoordinationReconcileReport, ReconcileError> {
    reconcile_coordination_startup_observed_with_limits(store, bridget_socket, limits, |_| Ok(()))
}

#[doc(hidden)]
pub fn reconcile_coordination_startup_observed_with_limits(
    store: &mut MaicieStore,
    bridget_socket: impl AsRef<Path>,
    limits: BridgetClientLimits,
    mut observer: impl FnMut(CoordinationReconcilePhase) -> Result<(), StoreError>,
) -> Result<CoordinationReconcileReport, ReconcileError> {
    let deadline = Instant::now() + reconciliation_budget(limits);
    let after_cursor = store.coordination_cursor()?;
    let mut report = CoordinationReconcileReport::default();
    let mut client = match CoordinationClient::connect_with_limits_until(
        bridget_socket,
        store.issuer_scope(),
        limits,
        deadline,
    ) {
        Ok(client) => client,
        Err(BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. }) => {
            report
                .actions
                .push(CoordinationReconcileAction::TransportIndisponible);
            return Ok(report);
        }
        Err(error) => return Err(ReconcileError::Client(error)),
    };
    let items = match client.snapshot_after(after_cursor, MAX_COORDINATION_ITEMS_PER_PASS) {
        Ok(items) => items,
        Err(BridgetClientError::Timeout { .. } | BridgetClientError::ItemLimitExceeded { .. }) => {
            report
                .actions
                .push(CoordinationReconcileAction::BudgetEpuise);
            return Ok(report);
        }
        Err(error) => return Err(ReconcileError::Client(error)),
    };
    let boundary = items.last().ok_or(ReconcileError::InvalidSnapshot(
        "relève coordination sans frontière terminale",
    ))?;
    match boundary {
        CoordinationStreamItem::Gap {
            from_cursor,
            to_cursor,
            reason,
        } => {
            report.actions.push(CoordinationReconcileAction::Gap {
                from_cursor: *from_cursor,
                to_cursor: *to_cursor,
                reason: reason.clone(),
            });
            return Ok(report);
        }
        CoordinationStreamItem::Unavailable { reason } => {
            report
                .actions
                .push(CoordinationReconcileAction::Unavailable {
                    reason: reason.clone(),
                });
            return Ok(report);
        }
        CoordinationStreamItem::SnapshotCaughtUp { through_cursor } => {
            report
                .actions
                .push(CoordinationReconcileAction::SnapshotAtteint {
                    through_cursor: *through_cursor,
                });
        }
        _ => {
            return Err(ReconcileError::InvalidSnapshot(
                "relève coordination incomplète",
            ));
        }
    }

    for item in items
        .into_iter()
        .take_while(|item| !matches!(item, CoordinationStreamItem::SnapshotCaughtUp { .. }))
    {
        observer(CoordinationReconcilePhase::BeforeApply)?;
        match item {
            CoordinationStreamItem::Event { canonical_bytes } => {
                apply_attested_coordination_event(store, &canonical_bytes, |phase| {
                    observer(match phase {
                        crate::store::CoordinationCommitPhase::BeforeCommit => {
                            CoordinationReconcilePhase::BeforeStoreCommit
                        }
                        crate::store::CoordinationCommitPhase::AfterCommit => {
                            CoordinationReconcilePhase::AfterStoreCommit
                        }
                        _ => return Ok(()),
                    })
                })
                .map_err(ReconcileError::Guichet)?;
                let cursor =
                    store
                        .coordination_cursor()?
                        .ok_or(ReconcileError::InvalidSnapshot(
                            "événement appliqué sans curseur local",
                        ))?;
                report
                    .actions
                    .push(CoordinationReconcileAction::EvenementApplique { cursor });
            }
            CoordinationStreamItem::Lifecycle(event) => {
                let request_id = event.request_id.clone();
                let state = event.state.clone();
                record_guichet_lifecycle_event(store, &event).map_err(ReconcileError::Guichet)?;
                observer(CoordinationReconcilePhase::AfterStoreCommit)?;
                report
                    .actions
                    .push(CoordinationReconcileAction::TerminalApplique { request_id, state });
            }
            CoordinationStreamItem::Gap { .. }
            | CoordinationStreamItem::Unavailable { .. }
            | CoordinationStreamItem::SnapshotCaughtUp { .. } => {
                return Err(ReconcileError::InvalidSnapshot(
                    "frontière coordination avant la fin du lot",
                ));
            }
        }
    }
    Ok(report)
}

fn record_guichet_transport_error(
    report: &mut GuichetReconcileReport,
    error: BridgetClientError,
) -> Result<(), ReconcileError> {
    match error {
        BridgetClientError::Connect { .. } | BridgetClientError::Timeout { .. } => {
            report
                .actions
                .push(GuichetReconcileAction::TransportIndisponible);
            Ok(())
        }
        error if transport_is_ambiguous(&error) => {
            report
                .actions
                .push(GuichetReconcileAction::TransportIncertain);
            Ok(())
        }
        error => Err(ReconcileError::Client(error)),
    }
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
        // `retry_until` borne la tentative initiale, mais la tombstone 009
        // reste encore consultable jusqu'à `dedup_retained_until`. Dans cette
        // fenêtre, le replay exact est le lookup sûr : il peut rendre une
        // issue durable sans créer de second SpawnOrder.
        let action = if observed_at >= activation.dedup_retained_until {
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
                        // Le spawn a réellement eu lieu : le persister comme
                        // refus mentirait. L'issue terminale conserve donc
                        // `accepted` et journalise explicitement l'agent
                        // lancé sans suivi Maicie implicite.
                        store.record_activation_definition_divergence(
                            command_id,
                            &replay.outcome,
                            &pending.approval.context_hash,
                            replay.definition_digest.as_deref().unwrap_or_default(),
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
