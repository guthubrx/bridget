//! Persistance SQLite privée de Maicie.
//!
//! La décision locale, la délégation et son outbox sont écrites dans une
//! transaction unique. Aucune méthode de reprise ne reconstruit l'enveloppe :
//! les octets préparés avant I/O sont l'autorité.

use crate::bridget_client::{IdempotencyIssue, SpawnOutcome};
use crate::domain::{
    ActivationOutbox, ApprobationActivation, DecisionCoordination, DomainError,
    EtatActivationOutbox, EtatDecision, EtatDelegation, EtatObjectif, EtatOutboxDelegation,
    ObjectifCoordonne, TypeDecision,
};
use crate::outbox::{
    MAX_MESSAGE_BYTES, OutboxError, PendingDelegationOutbox, PreparedDelegation, RecoverySnapshot,
    StoreCommitPhase,
};
use rusqlite::{
    Connection, ErrorCode, OptionalExtension, Transaction, TransactionBehavior, params,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fmt;
use std::fs::{self, DirBuilder, OpenOptions};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 5;
const DATABASE_MODE: u32 = 0o600;
const DIRECTORY_MODE: u32 = 0o700;
type StoredActivationOutcome = (
    Vec<u8>,
    String,
    i64,
    Option<Vec<u8>>,
    String,
    String,
    String,
);
type StoredActivationApproval = (
    Vec<u8>,
    Vec<u8>,
    String,
    String,
    String,
    String,
    String,
    String,
);

/// Autorité d'écriture unique de l'état Maicie.
pub struct MaicieStore {
    path: PathBuf,
    connection: Connection,
    issuer_scope: String,
}

/// Résultat durable d'une commande `delegate` idempotente. Les identifiants
/// sont conservés séparément de l'outbox afin qu'un rejeu local ne dépende pas
/// de la disponibilité du transport Bridget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDelegateResult {
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    pub message_id: Uuid,
    pub participant: String,
    pub timeout_secs: u64,
}

/// Issue de la réservation atomique d'une commande `delegate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegateReservation {
    Created,
    Replay(StoredDelegateResult),
}

/// Vue corrélée d'un objectif privée de toute interprétation du transport.
/// Les décisions y figurent afin que les commandes explicites restent
/// auditables même lorsqu'elles ne créent aucune nouvelle délégation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveSnapshot {
    pub objective: ObjectifCoordonne,
    pub delegations: Vec<crate::domain::Delegation>,
    pub decisions: Vec<DecisionCoordination>,
}

/// Motif local fermé quand les octets durables ne peuvent jamais produire une
/// remise valide. Il ne crée aucun nouvel état de domaine : l'outbox converge
/// vers `Rejected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalFailureReason {
    FrameTooLarge,
    InvalidEnvelope,
}

impl LocalFailureReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::FrameTooLarge => "frame_too_large",
            Self::InvalidEnvelope => "invalid_envelope",
        }
    }
}

/// Lecture de reprise par entrée : une corruption corrélable ne rend jamais
/// les autres outboxes indisponibles et peut être terminalisée sans rejeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationRecoveryEntry {
    Pending(PendingDelegationOutbox),
    LocalFailure {
        objective_id: Uuid,
        delegation_id: Uuid,
        message_id: Uuid,
        reason: LocalFailureReason,
    },
}

/// Ligne de reprise d'un SpawnOrder. Les octets sont ceux validés lors de
/// l'approbation ; le store ne les désérialise ni ne les reconstruit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingActivationOutbox {
    pub activation: ActivationOutbox,
    pub approval: ApprobationActivation,
}

/// Paramètres revalidés au moment du dispatch. Ils restent séparés de
/// l'approbation persistée : les hashes viennent de l'état courant, tandis que
/// les octets du SpawnOrder deviennent immuables seulement après validation.
pub struct ActivationApprovalRequest<'a> {
    pub now: i64,
    pub profile_hash: &'a [u8],
    pub context_hash: &'a [u8],
    pub spawn_order_bytes: &'a [u8],
    pub retry_until: i64,
    pub dedup_retained_until: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovedSpawnOrder {
    #[serde(rename = "type")]
    kind: String,
    agent_type: String,
    name: Option<String>,
    cwd: String,
    persistent: bool,
    command_id: String,
    issued_at: i64,
    deadline_at: i64,
}

impl MaicieStore {
    /// Ouvre la base privée, applique les migrations idempotentes et charge
    /// l'identité stable utilisée par le contrat client Bridget.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        validate_database_path(path)?;
        prepare_private_database(path)?;

        let mut connection = Connection::open(path).map_err(StoreError::Sql)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(StoreError::Sql)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;\n\
                 PRAGMA synchronous = FULL;",
            )
            .map_err(StoreError::Sql)?;
        migrate(&mut connection)?;
        set_wal_mode(&connection)?;
        let issuer_scope = load_or_create_issuer_scope(&mut connection)?;

        Ok(Self {
            path: path.to_path_buf(),
            connection,
            issuer_scope,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn issuer_scope(&self) -> &str {
        &self.issuer_scope
    }

    pub fn schema_version(&self) -> Result<i64, StoreError> {
        self.connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(StoreError::Sql)
    }

    /// Lit les objectifs Maicie avec leurs délégations et décisions locales.
    /// Cette vue ne joint volontairement aucune donnée Bridget : T018 ajoutera
    /// les sources de transport publiques sans en faire une autorité métier.
    pub fn objective_snapshots(
        &self,
        objective_id: Option<Uuid>,
    ) -> Result<Vec<ObjectiveSnapshot>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, state, payload_json FROM objectives\n\
                 WHERE (?1 IS NULL OR id = ?1) ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.map(|id| id.to_string())], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (id, state, payload) = row.map_err(StoreError::Sql)?;
            let objective: ObjectifCoordonne =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if objective.id.to_string() != id || objective.etat != parse_objective_state(&state)? {
                return Err(StoreError::Corrupt("objectif et index SQLite divergents"));
            }
            let delegations = self.delegations_for(objective.id)?;
            let decisions = self.decisions_for(objective.id)?;
            Ok(ObjectiveSnapshot {
                objective,
                delegations,
                decisions,
            })
        })
        .collect()
    }

    /// Applique une décision locale explicite et son éventuelle mise à jour
    /// d'objectif dans une unique transaction. La décision est l'audit de
    /// l'effet : aucun des deux n'est durable seul.
    pub fn apply_objective_decision(
        &mut self,
        decision: &DecisionCoordination,
        updated_objective: Option<&ObjectifCoordonne>,
    ) -> Result<(), StoreError> {
        decision.verifier().map_err(StoreError::Domain)?;
        if decision.etat != EtatDecision::Appliquee {
            return Err(StoreError::Invalid("décision non appliquée"));
        }
        if updated_objective.is_some_and(|objective| objective.id != decision.objectif_id) {
            return Err(StoreError::Invalid("décision et objectif divergents"));
        }
        let decision_json = serde_json::to_vec(decision).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let exists: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM objectives WHERE id = ?1",
                [decision.objectif_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if exists.is_none() {
            return Err(StoreError::NotFound("objectif absent"));
        }
        if let Some(objective) = updated_objective {
            let payload = serde_json::to_vec(objective).map_err(StoreError::Json)?;
            let changed = tx
                .execute(
                    "UPDATE objectives SET state = ?1, payload_json = ?2 WHERE id = ?3",
                    params![
                        objective_state_name(objective.etat),
                        payload,
                        objective.id.to_string()
                    ],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict("objectif modifié concurremment"));
            }
        }
        let inserted = tx
            .execute(
                "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    decision.id.to_string(),
                    decision.objectif_id.to_string(),
                    decision_state_name(decision.etat),
                    decision_json,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("décision non enregistrée"));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    fn delegations_for(
        &self,
        objective_id: Uuid,
    ) -> Result<Vec<crate::domain::Delegation>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, state, payload_json FROM delegations\n\
                 WHERE objective_id = ?1 ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (id, state, payload) = row.map_err(StoreError::Sql)?;
            let delegation = serde_json::from_slice::<crate::domain::Delegation>(&payload)
                .map_err(StoreError::Json)?;
            if delegation.id.to_string() != id
                || delegation.objectif_id != objective_id
                || delegation.etat != parse_delegation_state(&state)?
            {
                return Err(StoreError::Corrupt("délégation et index SQLite divergents"));
            }
            Ok(delegation)
        })
        .collect()
    }

    fn decisions_for(&self, objective_id: Uuid) -> Result<Vec<DecisionCoordination>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, state, payload_json FROM coordination_decisions\n\
                 WHERE objective_id = ?1 ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (id, state, payload) = row.map_err(StoreError::Sql)?;
            let decision: DecisionCoordination =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if decision.id.to_string() != id
                || decision.objectif_id != objective_id
                || decision.etat != parse_decision_state(&state)?
            {
                return Err(StoreError::Corrupt("décision et index SQLite divergents"));
            }
            Ok(decision)
        })
        .collect()
    }

    /// Écrit l'agrégat complet dans une seule transaction SQLite.
    pub fn create_prepared_delegation(
        &mut self,
        prepared: &PreparedDelegation,
    ) -> Result<(), StoreError> {
        self.create_prepared_delegation_observed(prepared, |_| Ok(()))
    }

    /// Variante de production instrumentée par les crash-tests réels.
    pub fn create_prepared_delegation_observed(
        &mut self,
        prepared: &PreparedDelegation,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        prepared.validate().map_err(StoreError::Outbox)?;
        if prepared.issuer_scope != self.issuer_scope {
            return Err(StoreError::Conflict(
                "issuer_scope différent de l'identité durable du store",
            ));
        }

        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        insert_prepared(&tx, prepared)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(())
    }

    /// Lit un résultat durable déjà réservé pour une clé de commande. Les
    /// octets canoniques font partie de l'identité : une clé réemployée pour
    /// une autre enveloppe est refusée sans écrire quoi que ce soit.
    pub fn lookup_delegate_replay(
        &self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
    ) -> Result<Option<StoredDelegateResult>, StoreError> {
        validate_delegate_idempotency_key(idempotency_key)?;
        let stored = self
            .connection
            .query_row(
                "SELECT canonical_request_bytes, objective_id, delegation_id, message_id, participant, timeout_secs\n\
                 FROM delegate_idempotency WHERE idempotency_key = ?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        stored
            .map(|stored| decode_delegate_result(stored, canonical_request_bytes))
            .transpose()
    }

    /// Réserve dans la même transaction l'agrégat et son résultat de commande.
    /// Le second lookup protège aussi une course entre le lookup optimiste du
    /// cas d'usage et l'insertion effective.
    pub fn lookup_or_reserve_delegate(
        &mut self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
        prepared: &PreparedDelegation,
    ) -> Result<DelegateReservation, StoreError> {
        self.lookup_or_reserve_delegate_observed(
            idempotency_key,
            canonical_request_bytes,
            prepared,
            |_| Ok(()),
        )
    }

    /// Variante instrumentable pour les crash-tests à la frontière de commit.
    pub fn lookup_or_reserve_delegate_observed(
        &mut self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
        prepared: &PreparedDelegation,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<DelegateReservation, StoreError> {
        validate_delegate_idempotency_key(idempotency_key)?;
        prepared.validate().map_err(StoreError::Outbox)?;
        if prepared.issuer_scope != self.issuer_scope {
            return Err(StoreError::Conflict(
                "issuer_scope différent de l'identité durable du store",
            ));
        }

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored = tx
            .query_row(
                "SELECT canonical_request_bytes, objective_id, delegation_id, message_id, participant, timeout_secs\n\
                 FROM delegate_idempotency WHERE idempotency_key = ?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some(stored) = stored {
            return decode_delegate_result(stored, canonical_request_bytes)
                .map(DelegateReservation::Replay);
        }

        insert_prepared(&tx, prepared)?;
        tx.execute(
            "INSERT INTO delegate_idempotency(\n\
                 idempotency_key, canonical_request_bytes, objective_id, delegation_id,\n\
                 message_id, participant, timeout_secs\n\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                idempotency_key,
                canonical_request_bytes,
                prepared.objective.id.to_string(),
                prepared.delegation.id.to_string(),
                prepared.outbox.message_id.to_string(),
                prepared.delegation.participant,
                i64::try_from(prepared.outbox.timeout_secs)
                    .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
            ],
        )
        .map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(DelegateReservation::Created)
    }

    /// Retourne uniquement les outboxes non terminales, avec l'enveloppe
    /// exacte nécessaire au lookup puis au replay de T008.
    pub fn pending_delegation_outboxes(&self) -> Result<Vec<PendingDelegationOutbox>, StoreError> {
        self.raw_pending_delegations()?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    /// Retourne chaque outbox indépendamment. Une enveloppe durable invalide
    /// garde ses identifiants de corrélation et devient un refus local que le
    /// reconciliateur persiste avant de poursuivre les autres entrées.
    pub fn delegation_recovery_entries(&self) -> Result<Vec<DelegationRecoveryEntry>, StoreError> {
        self.raw_pending_delegations()?
            .into_iter()
            .map(|raw| {
                let objective_id = parse_uuid(&raw.objective_id)?;
                let delegation_id = parse_uuid(&raw.delegation_id)?;
                let message_id = parse_uuid(&raw.message_id)?;
                let reason = if raw.body_bytes.len() > MAX_MESSAGE_BYTES
                    || raw.message_bytes.len() > MAX_MESSAGE_BYTES
                {
                    LocalFailureReason::FrameTooLarge
                } else {
                    LocalFailureReason::InvalidEnvelope
                };
                Ok(match raw.try_into() {
                    Ok(pending) => DelegationRecoveryEntry::Pending(pending),
                    Err(_) => DelegationRecoveryEntry::LocalFailure {
                        objective_id,
                        delegation_id,
                        message_id,
                        reason,
                    },
                })
            })
            .collect()
    }

    fn raw_pending_delegations(&self) -> Result<Vec<RawPending>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT objective_id, delegation_id, message_id, issuer_scope, issued_at,\n\
                        target, body_bytes, reply, timeout_secs, deadline_contractuelle,\n\
                        body_hash, message_bytes, state, attempted_at, retry_until,\n\
                        dedup_retained_until\n\
                 FROM delegation_outbox\n\
                 WHERE terminal = 0 AND state IN ('prepared', 'outcome_unknown')\n\
                 ORDER BY issued_at, message_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], raw_pending_from_row)
            .map_err(StoreError::Sql)?;
        rows.map(|row| row.map_err(StoreError::Sql)).collect()
    }

    /// Persiste l'issue observée par lookup. Une issue terminale retire
    /// l'enveloppe de la liste de reprise sans supprimer sa preuve.
    pub fn record_lookup_issue(
        &mut self,
        message_id: Uuid,
        issue: &IdempotencyIssue,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        let next_state = match issue {
            IdempotencyIssue::Accepted { .. } => EtatOutboxDelegation::Accepted,
            IdempotencyIssue::OutcomeUnknown { .. } => EtatOutboxDelegation::OutcomeUnknown,
            IdempotencyIssue::Rejected { .. }
            | IdempotencyIssue::EnvelopeMismatch
            | IdempotencyIssue::IdempotencyExpired
            | IdempotencyIssue::InvalidIssuedAt => EtatOutboxDelegation::Rejected,
        };
        let terminal = matches!(
            next_state,
            EtatOutboxDelegation::Accepted | EtatOutboxDelegation::Rejected
        );
        let issue_json = encode_issue(issue);
        let issue_bytes = serde_json::to_vec(&issue_json).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current: Option<(String, i64, Option<Vec<u8>>)> = tx
            .query_row(
                "SELECT state, terminal, last_issue_json\n\
                 FROM delegation_outbox WHERE message_id = ?1",
                [message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((current_state_name, current_terminal, current_issue)) = current else {
            return Err(StoreError::NotFound("message_id inconnu"));
        };
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict("issue terminale déjà figée"));
        }
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != next_state {
            current_state
                .transition_vers(next_state)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = ?1, terminal = ?2, last_issue_json = ?3,\n\
                     issue_observed_at = ?4, attempted_at = COALESCE(attempted_at, ?4)\n\
                 WHERE message_id = ?5 AND state = ?6 AND terminal = 0",
                params![
                    outbox_state_name(next_state),
                    i64::from(terminal),
                    issue_bytes,
                    observed_at,
                    message_id.to_string(),
                    current_state_name
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant la transition",
            ));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Persiste un refus local déterministe. Le payload fermé permet au
    /// reconciliateur de prouver pourquoi aucun octet n'a été remis à Bridget.
    pub fn record_local_failure(
        &mut self,
        message_id: Uuid,
        reason: LocalFailureReason,
    ) -> Result<(), StoreError> {
        let issue_bytes =
            serde_json::to_vec(&json!({"local": reason.as_str()})).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current: Option<(String, i64, Option<Vec<u8>>)> = tx
            .query_row(
                "SELECT state, terminal, last_issue_json\n\
                 FROM delegation_outbox WHERE message_id = ?1",
                [message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((current_state_name, current_terminal, current_issue)) = current else {
            return Err(StoreError::NotFound("message_id inconnu"));
        };
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict("issue terminale déjà figée"));
        }
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != EtatOutboxDelegation::Rejected {
            current_state
                .transition_vers(EtatOutboxDelegation::Rejected)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = 'rejected', terminal = 1, last_issue_json = ?1\n\
                 WHERE message_id = ?2 AND state = ?3 AND terminal = 0",
                params![issue_bytes, message_id.to_string(), current_state_name],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant le rejet local",
            ));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Marque une transmission sans issue durable. La ligne reste éligible au
    /// lookup de reprise ; aucun nouvel identifiant n'est créé.
    pub fn record_transport_uncertainty(
        &mut self,
        message_id: Uuid,
        attempted_at: i64,
    ) -> Result<(), StoreError> {
        if attempted_at <= 0 {
            return Err(StoreError::Invalid("attempted_at invalide"));
        }
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current_state_name: Option<String> = tx
            .query_row(
                "SELECT state FROM delegation_outbox\n\
                 WHERE message_id = ?1 AND terminal = 0",
                [message_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some(current_state_name) = current_state_name else {
            return Err(StoreError::NotFound("outbox absente ou terminale"));
        };
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != EtatOutboxDelegation::OutcomeUnknown {
            current_state
                .transition_vers(EtatOutboxDelegation::OutcomeUnknown)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = 'outcome_unknown', attempted_at = ?1\n\
                 WHERE message_id = ?2 AND state = ?3 AND terminal = 0",
                params![attempted_at, message_id.to_string(), current_state_name],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant la transition",
            ));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Snapshot corrélé à la délégation, utilisable par T008 sans second
    /// lookup local ni reconstruction de payload.
    pub fn recovery_snapshot(
        &self,
        message_id: Uuid,
    ) -> Result<Option<RecoverySnapshot>, StoreError> {
        let raw = self
            .connection
            .query_row(
                "SELECT o.objective_id, o.delegation_id, o.message_id, o.issuer_scope,\n\
                        o.issued_at, o.target, o.body_bytes, o.reply, o.timeout_secs,\n\
                        o.deadline_contractuelle, o.body_hash, o.message_bytes, o.state,\n\
                        o.attempted_at, o.retry_until, o.dedup_retained_until,\n\
                        obj.state, d.state, o.last_issue_json, o.issue_observed_at\n\
                 FROM delegation_outbox o\n\
                 JOIN objectives obj ON obj.id = o.objective_id\n\
                 JOIN delegations d ON d.id = o.delegation_id\n\
                 WHERE o.message_id = ?1",
                [message_id.to_string()],
                |row| {
                    Ok(RawRecovery {
                        pending: raw_pending_from_row(row)?,
                        objective_state: row.get(16)?,
                        delegation_state: row.get(17)?,
                        last_issue_json: row.get(18)?,
                        issue_observed_at: row.get(19)?,
                    })
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        raw.map(TryInto::try_into).transpose()
    }

    /// Persiste une proposition de réveil et son approbation mono-usage. Le
    /// `command_id` est déjà porté par l'approbation, donc existe avant toute
    /// I/O vers Bridget et reste stable sur chaque replay.
    pub fn create_activation_proposal(
        &mut self,
        decision: &DecisionCoordination,
        approval: &ApprobationActivation,
    ) -> Result<(), StoreError> {
        validate_activation_proposal(decision, approval)?;
        let decision_json = serde_json::to_vec(decision).map_err(StoreError::Json)?;
        let approval_json = serde_json::to_vec(approval).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        tx.execute(
            "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
             VALUES (?1, ?2, ?3, ?4)",
            params![
                decision.id.to_string(),
                decision.objectif_id.to_string(),
                decision_state_name(decision.etat),
                decision_json,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.execute(
            "INSERT INTO activation_approvals(\n\
                 id, command_id, decision_id, objective_id, profile_id, profile_hash,\n\
                 context_hash, state, expires_at, consumed_at, payload_json\n\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'proposed', ?8, NULL, ?9)",
            params![
                approval.id.to_string(),
                approval.command_id.to_string(),
                decision.id.to_string(),
                approval.objective_id.to_string(),
                approval.profile_id,
                approval.profile_hash,
                approval.context_hash,
                approval.expires_at,
                approval_json,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.commit().map_err(StoreError::Sql)
    }

    /// Revalide l'approbation contre les hashes courants et écrit l'outbox
    /// dans la même transaction que l'état approuvé de la décision. Un retry
    /// ne peut jamais remplacer les octets d'un SpawnOrder déjà enregistré.
    pub fn approve_activation(
        &mut self,
        approval_id: Uuid,
        request: &ActivationApprovalRequest<'_>,
    ) -> Result<ActivationOutbox, StoreError> {
        self.approve_activation_observed(approval_id, request, |_| Ok(()))
    }

    /// Variante instrumentée par les crash-tests : la proposition approuvée
    /// et son outbox restent une transaction SQLite indivisible.
    pub fn approve_activation_observed(
        &mut self,
        approval_id: Uuid,
        request: &ActivationApprovalRequest<'_>,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<ActivationOutbox, StoreError> {
        if request.spawn_order_bytes.is_empty()
            || request.retry_until < request.now
            || request.retry_until > request.dedup_retained_until
        {
            return Err(StoreError::Invalid("activation outbox invalide"));
        }
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let row: Option<StoredActivationApproval> = tx
            .query_row(
                "SELECT a.payload_json, d.payload_json, a.id, a.command_id, a.objective_id,\n\
                        a.state, d.objective_id, d.state\n\
                 FROM activation_approvals a\n\
                 JOIN coordination_decisions d ON d.id = a.decision_id\n\
                 WHERE a.id = ?1",
                [approval_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((
            approval_json,
            decision_json,
            stored_approval_id,
            stored_command_id,
            stored_approval_objective_id,
            stored_approval_state,
            stored_decision_objective_id,
            stored_decision_state,
        )) = row
        else {
            return Err(StoreError::NotFound("approbation inconnue"));
        };
        let approval: ApprobationActivation =
            serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
        let mut decision: DecisionCoordination =
            serde_json::from_slice(&decision_json).map_err(StoreError::Json)?;
        if stored_approval_id != approval.id.to_string()
            || stored_command_id != approval.command_id.to_string()
            || stored_approval_objective_id != approval.objective_id.to_string()
            || stored_decision_objective_id != decision.objectif_id.to_string()
            || approval.objective_id != decision.objectif_id
        {
            return Err(StoreError::Corrupt("relations d'approbation divergentes"));
        }
        validate_approved_spawn_order(&approval, request.spawn_order_bytes)?;

        let existing: Option<(Vec<u8>, String, i64, i64)> = tx
            .query_row(
                "SELECT spawn_order_bytes, state, retry_until, dedup_retained_until\n\
                 FROM activation_outbox WHERE approval_id = ?1",
                [approval.id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some((stored_bytes, state, stored_retry_until, stored_retained_until)) = existing {
            if stored_bytes != request.spawn_order_bytes {
                return Err(StoreError::Conflict(
                    "SpawnOrder divergent pour la même approbation",
                ));
            }
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(ActivationOutbox {
                command_id: approval.command_id,
                approval_id: approval.id,
                spawn_order_bytes: stored_bytes,
                etat: parse_activation_state(&state)?,
                retry_until: stored_retry_until,
                dedup_retained_until: stored_retained_until,
            });
        }

        if stored_approval_state != "proposed" || stored_decision_state != "proposed" {
            return Err(StoreError::Corrupt("état d'approbation divergent"));
        }

        approval
            .verifier_pour_dispatch(request.now, request.profile_hash, request.context_hash)
            .map_err(StoreError::Domain)?;
        if decision.etat != EtatDecision::Proposee {
            return Err(StoreError::Conflict("décision déjà traitée"));
        }
        decision.etat = EtatDecision::Approuvee;
        let decision_json = serde_json::to_vec(&decision).map_err(StoreError::Json)?;
        let activation = ActivationOutbox {
            command_id: approval.command_id,
            approval_id: approval.id,
            spawn_order_bytes: request.spawn_order_bytes.to_vec(),
            etat: EtatActivationOutbox::Dispatching,
            retry_until: request.retry_until,
            dedup_retained_until: request.dedup_retained_until,
        };
        activation.verifier().map_err(StoreError::Domain)?;
        let changed = tx
            .execute(
                "UPDATE coordination_decisions SET state = 'approved', payload_json = ?1\n\
             WHERE id = ?2 AND state = 'proposed'",
                params![decision_json, decision.id.to_string()],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("décision modifiée concurremment"));
        }
        let changed = tx
            .execute(
                "UPDATE activation_approvals SET state = 'approved'\n\
             WHERE id = ?1 AND state = 'proposed'",
                [approval.id.to_string()],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("approbation modifiée concurremment"));
        }
        tx.execute(
            "INSERT INTO activation_outbox(\n\
                 command_id, approval_id, spawn_order_bytes, state, retry_until,\n\
                 dedup_retained_until, terminal\n\
             ) VALUES (?1, ?2, ?3, 'dispatching', ?4, ?5, 0)",
            params![
                activation.command_id.to_string(),
                activation.approval_id.to_string(),
                activation.spawn_order_bytes,
                activation.retry_until,
                activation.dedup_retained_until,
            ],
        )
        .map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(activation)
    }

    /// Une issue durable est la seule opération qui consomme l'approbation.
    /// La consommation et l'état terminal de l'outbox partagent la même
    /// transaction afin qu'un crash ne puisse pas laisser un SpawnOrder
    /// appliqué avec une approbation réutilisable.
    pub fn record_activation_outcome(
        &mut self,
        command_id: Uuid,
        outcome: &SpawnOutcome,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        let (next_state, issue_bytes) = activation_issue(command_id, outcome)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let row: Option<StoredActivationOutcome> = tx
            .query_row(
                "SELECT a.payload_json, o.state, o.terminal, o.last_issue_json,\n\
                        o.command_id, o.approval_id, a.command_id\n\
                 FROM activation_outbox o\n\
                 JOIN activation_approvals a ON a.id = o.approval_id\n\
                 WHERE o.command_id = ?1",
                [command_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((
            approval_json,
            state,
            current_terminal,
            current_issue,
            stored_command_id,
            stored_approval_id,
            stored_approval_command_id,
        )) = row
        else {
            return Err(StoreError::NotFound("activation inconnue"));
        };
        let mut approval: ApprobationActivation =
            serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
        if stored_command_id != command_id.to_string()
            || stored_approval_command_id != command_id.to_string()
            || stored_approval_id != approval.id.to_string()
            || approval.command_id != command_id
        {
            return Err(StoreError::Corrupt("relations d'activation divergentes"));
        }
        let current = parse_activation_state(&state)?;
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict(
                "issue d'activation terminale déjà figée",
            ));
        }
        if next_state == EtatActivationOutbox::OutcomeUnknown {
            if !matches!(current, EtatActivationOutbox::Dispatching) {
                return Err(StoreError::Conflict("transition activation interdite"));
            }
            let changed = tx
                .execute(
                    "UPDATE activation_outbox\n\
                 SET state = 'outcome_unknown', last_issue_json = ?1, issue_observed_at = ?2\n\
                 WHERE command_id = ?3 AND state = 'dispatching' AND terminal = 0",
                    params![issue_bytes, observed_at, command_id.to_string()],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict("activation modifiée concurremment"));
            }
            return tx.commit().map_err(StoreError::Sql);
        }
        approval.consumed_at = Some(observed_at);
        let approval_json = serde_json::to_vec(&approval).map_err(StoreError::Json)?;
        let changed = tx.execute(
            "UPDATE activation_approvals SET state = 'consumed', consumed_at = ?1, payload_json = ?2\n\
             WHERE id = ?3 AND consumed_at IS NULL",
            params![observed_at, approval_json, approval.id.to_string()],
        )
        .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("approbation modifiée concurremment"));
        }
        let changed = tx.execute(
            "UPDATE activation_outbox SET state = 'applied', terminal = 1, last_issue_json = ?1, issue_observed_at = ?2\n\
             WHERE command_id = ?3 AND terminal = 0",
            params![issue_bytes, observed_at, command_id.to_string()],
        )
        .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("activation modifiée concurremment"));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    pub fn pending_activation_outboxes(&self) -> Result<Vec<PendingActivationOutbox>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT o.command_id, o.approval_id, o.spawn_order_bytes, o.state,\n\
                        o.retry_until, o.dedup_retained_until, a.payload_json\n\
                 FROM activation_outbox o\n\
                 JOIN activation_approvals a ON a.id = o.approval_id\n\
                 WHERE o.terminal = 0\n\
                 ORDER BY o.command_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (
                command_id,
                approval_id,
                spawn_order_bytes,
                state,
                retry_until,
                retained_until,
                approval_json,
            ) = row.map_err(StoreError::Sql)?;
            let activation = ActivationOutbox {
                command_id: parse_uuid(&command_id)?,
                approval_id: parse_uuid(&approval_id)?,
                spawn_order_bytes,
                etat: parse_activation_state(&state)?,
                retry_until,
                dedup_retained_until: retained_until,
            };
            let approval = serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
            activation.verifier().map_err(StoreError::Domain)?;
            Ok(PendingActivationOutbox {
                activation,
                approval,
            })
        })
        .collect()
    }
}

fn set_wal_mode(connection: &Connection) -> Result<(), StoreError> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match connection.execute_batch("PRAGMA journal_mode = WAL;") {
            Ok(()) => return Ok(()),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == ErrorCode::DatabaseBusy && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(StoreError::Sql(error)),
        }
    }
}

fn validate_database_path(path: &Path) -> Result<(), StoreError> {
    if !path.is_absolute() {
        return Err(StoreError::Invalid("chemin SQLite non absolu"));
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("bridget.db") {
        return Err(StoreError::Invalid(
            "bridget.db appartient exclusivement à Bridget",
        ));
    }
    if path.parent().is_none() {
        return Err(StoreError::Invalid("base SQLite sans répertoire parent"));
    }
    Ok(())
}

fn prepare_private_database(path: &Path) -> Result<(), StoreError> {
    let parent = path.parent().expect("validé");
    if !parent.exists() {
        let mut builder = DirBuilder::new();
        builder.recursive(true).mode(DIRECTORY_MODE);
        match builder.create(parent) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(StoreError::Io(error)),
        }
    }
    let parent_metadata = fs::symlink_metadata(parent).map_err(StoreError::Io)?;
    if !parent_metadata.file_type().is_dir()
        || parent_metadata.permissions().mode() & 0o777 != DIRECTORY_MODE
    {
        return Err(StoreError::Invalid(
            "répertoire SQLite non privé (0700 requis)",
        ));
    }

    if !path.exists() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(DATABASE_MODE)
            .open(path)
        {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(StoreError::Io(error)),
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(StoreError::Io)?;
    if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o777 != DATABASE_MODE {
        return Err(StoreError::Invalid(
            "fichier SQLite non privé (0600 requis)",
        ));
    }
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), StoreError> {
    // L'ouverture est un chemin concurrent normal : plusieurs processus
    // Maicie peuvent démarrer avant qu'un seul ait fini de poser le schéma.
    // Le verrou IMMEDIATE couvre donc la lecture de version et toutes les
    // migrations, pour que le second ouvre ensuite un schéma déjà cohérent.
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StoreError::Sql)?;
    let current_version: i64 = tx
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(StoreError::Sql)?;
    if current_version > SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema {
            found: current_version,
            supported: SCHEMA_VERSION,
        });
    }
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\n\
             version INTEGER PRIMARY KEY,\n\
             applied_at INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS maicie_identity (\n\
             singleton INTEGER PRIMARY KEY CHECK(singleton = 1),\n\
             issuer_scope TEXT NOT NULL UNIQUE\n\
         );\n\
         CREATE TABLE IF NOT EXISTS objectives (\n\
             id TEXT PRIMARY KEY,\n\
             state TEXT NOT NULL,\n\
             payload_json BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS delegations (\n\
             id TEXT PRIMARY KEY,\n\
             objective_id TEXT NOT NULL REFERENCES objectives(id),\n\
             state TEXT NOT NULL,\n\
             payload_json BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS delegation_outbox (\n\
             message_id TEXT PRIMARY KEY,\n\
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),\n\
             objective_id TEXT NOT NULL REFERENCES objectives(id),\n\
             issuer_scope TEXT NOT NULL,\n\
             issued_at INTEGER NOT NULL,\n\
             target TEXT NOT NULL,\n\
             body_bytes BLOB NOT NULL,\n\
             reply INTEGER NOT NULL CHECK(reply IN (0, 1)),\n\
             timeout_secs INTEGER NOT NULL,\n\
             deadline_contractuelle INTEGER NOT NULL,\n\
             body_hash BLOB NOT NULL,\n\
             message_bytes BLOB NOT NULL,\n\
             state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),\n\
             attempted_at INTEGER,\n\
             retry_until INTEGER NOT NULL,\n\
             dedup_retained_until INTEGER NOT NULL,\n\
             last_issue_json BLOB,\n\
             issue_observed_at INTEGER,\n\
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))\n\
         );\n\
         CREATE INDEX IF NOT EXISTS objectives_open_idx\n\
             ON objectives(state) WHERE state != 'clos';\n\
         CREATE INDEX IF NOT EXISTS delegation_outbox_pending_idx\n\
             ON delegation_outbox(terminal, state, retry_until, message_id);
         CREATE TABLE IF NOT EXISTS delegate_idempotency (
             idempotency_key TEXT PRIMARY KEY,
             canonical_request_bytes BLOB NOT NULL,
             objective_id TEXT NOT NULL UNIQUE REFERENCES objectives(id),
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
             message_id TEXT NOT NULL UNIQUE REFERENCES delegation_outbox(message_id),
             participant TEXT NOT NULL,
             timeout_secs INTEGER NOT NULL CHECK(timeout_secs > 0)
         );
         CREATE TABLE IF NOT EXISTS coordination_decisions (
             id TEXT PRIMARY KEY,
             objective_id TEXT NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('proposed','approved','rejected','applied')),
             payload_json BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS activation_approvals (
             id TEXT PRIMARY KEY,
             command_id TEXT NOT NULL UNIQUE,
             decision_id TEXT NOT NULL UNIQUE REFERENCES coordination_decisions(id),
             objective_id TEXT NOT NULL,
             profile_id TEXT NOT NULL,
             profile_hash BLOB NOT NULL,
             context_hash BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('proposed','approved','consumed')),
             expires_at INTEGER NOT NULL,
             consumed_at INTEGER,
             payload_json BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS activation_outbox (
             command_id TEXT PRIMARY KEY REFERENCES activation_approvals(command_id),
             approval_id TEXT NOT NULL UNIQUE REFERENCES activation_approvals(id),
             spawn_order_bytes BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('dispatching','outcome_unknown','applied')),
             retry_until INTEGER NOT NULL,
             dedup_retained_until INTEGER NOT NULL,
             last_issue_json BLOB,
             issue_observed_at INTEGER,
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))
         );
         CREATE INDEX IF NOT EXISTS activation_outbox_pending_idx
             ON activation_outbox(terminal, state, retry_until, command_id);",
    )
    .map_err(StoreError::Sql)?;
    if current_version == 1 {
        migrate_outbox_to_rejected_state(&tx)?;
    }
    if current_version == 3 {
        tx.execute_batch(
            "ALTER TABLE activation_outbox ADD COLUMN last_issue_json BLOB;
             ALTER TABLE activation_outbox ADD COLUMN issue_observed_at INTEGER;",
        )
        .map_err(StoreError::Sql)?;
    }
    for version in (current_version + 1)..=SCHEMA_VERSION {
        tx.execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at)\n\
             VALUES (?1, CAST(strftime('%s','now') AS INTEGER))",
            [version],
        )
        .map_err(StoreError::Sql)?;
    }
    tx.pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(StoreError::Sql)?;
    tx.commit().map_err(StoreError::Sql)
}

fn migrate_outbox_to_rejected_state(tx: &Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(
        "DROP INDEX IF EXISTS delegation_outbox_pending_idx;
         ALTER TABLE delegation_outbox RENAME TO delegation_outbox_v1;
         CREATE TABLE delegation_outbox (
             message_id TEXT PRIMARY KEY,
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
             objective_id TEXT NOT NULL REFERENCES objectives(id),
             issuer_scope TEXT NOT NULL,
             issued_at INTEGER NOT NULL,
             target TEXT NOT NULL,
             body_bytes BLOB NOT NULL,
             reply INTEGER NOT NULL CHECK(reply IN (0, 1)),
             timeout_secs INTEGER NOT NULL,
             deadline_contractuelle INTEGER NOT NULL,
             body_hash BLOB NOT NULL,
             message_bytes BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),
             attempted_at INTEGER,
             retry_until INTEGER NOT NULL,
             dedup_retained_until INTEGER NOT NULL,
             last_issue_json BLOB,
             issue_observed_at INTEGER,
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))
         );
         INSERT INTO delegation_outbox(
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,
             message_bytes, state, attempted_at, retry_until, dedup_retained_until,
             last_issue_json, issue_observed_at, terminal
         )
         SELECT message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
                body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,
                message_bytes,
                CASE
                    WHEN terminal = 1 AND state = 'outcome_unknown' THEN 'rejected'
                    ELSE state
                END,
                attempted_at, retry_until, dedup_retained_until,
                last_issue_json, issue_observed_at, terminal
         FROM delegation_outbox_v1;
         DROP TABLE delegation_outbox_v1;
         CREATE INDEX delegation_outbox_pending_idx
             ON delegation_outbox(terminal, state, retry_until, message_id);",
    )
    .map_err(StoreError::Sql)
}

fn load_or_create_issuer_scope(connection: &mut Connection) -> Result<String, StoreError> {
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StoreError::Sql)?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT issuer_scope FROM maicie_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let candidate = existing.unwrap_or_else(|| format!("maicie-{}", Uuid::new_v4().simple()));
    tx.execute(
        "INSERT OR IGNORE INTO maicie_identity(singleton, issuer_scope) VALUES (1, ?1)",
        [&candidate],
    )
    .map_err(StoreError::Sql)?;
    let scope: String = tx
        .query_row(
            "SELECT issuer_scope FROM maicie_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    tx.commit().map_err(StoreError::Sql)?;
    Ok(scope)
}

fn insert_prepared(tx: &Transaction<'_>, prepared: &PreparedDelegation) -> Result<(), StoreError> {
    upsert_objective(tx, &prepared.objective)?;
    let delegation_json = serde_json::to_vec(&prepared.delegation).map_err(StoreError::Json)?;
    tx.execute(
        "INSERT INTO delegations(id, objective_id, state, payload_json) VALUES (?1, ?2, ?3, ?4)",
        params![
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            delegation_state_name(prepared.delegation.etat),
            delegation_json
        ],
    )
    .map_err(StoreError::Sql)?;
    tx.execute(
        "INSERT INTO delegation_outbox(\n\
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,\n\
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,\n\
             message_bytes, state, attempted_at, retry_until, dedup_retained_until, terminal\n\
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'prepared',NULL,?13,?14,0)",
        params![
            prepared.outbox.message_id.to_string(),
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            prepared.issuer_scope,
            prepared.issued_at,
            prepared.outbox.target,
            prepared.outbox.body_bytes,
            i64::from(prepared.outbox.reply),
            i64::try_from(prepared.outbox.timeout_secs)
                .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
            prepared.outbox.deadline_contractuelle,
            prepared.outbox.body_hash,
            prepared.message_bytes,
            prepared.outbox.retry_until,
            prepared.outbox.dedup_retained_until,
        ],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

type RawDelegateResult = (Vec<u8>, String, String, String, String, i64);

fn decode_delegate_result(
    stored: RawDelegateResult,
    expected_canonical_request_bytes: &[u8],
) -> Result<StoredDelegateResult, StoreError> {
    let (
        canonical_request_bytes,
        objective_id,
        delegation_id,
        message_id,
        participant,
        timeout_secs,
    ) = stored;
    if canonical_request_bytes != expected_canonical_request_bytes {
        return Err(StoreError::EnvelopeMismatch);
    }
    if participant.is_empty() || timeout_secs <= 0 {
        return Err(StoreError::Corrupt(
            "résultat de délégation idempotente invalide",
        ));
    }
    Ok(StoredDelegateResult {
        objective_id: parse_uuid(&objective_id)?,
        delegation_id: parse_uuid(&delegation_id)?,
        message_id: parse_uuid(&message_id)?,
        participant,
        timeout_secs: u64::try_from(timeout_secs)
            .map_err(|_| StoreError::Corrupt("timeout idempotent invalide"))?,
    })
}

fn validate_delegate_idempotency_key(key: &str) -> Result<(), StoreError> {
    if key.is_empty() || key.len() > 128 || key.chars().any(char::is_control) {
        return Err(StoreError::Invalid("clé d'idempotence delegate invalide"));
    }
    Ok(())
}

fn upsert_objective(tx: &Transaction<'_>, objective: &ObjectifCoordonne) -> Result<(), StoreError> {
    let id = objective.id.to_string();
    let incoming_json = serde_json::to_vec(objective).map_err(StoreError::Json)?;
    let current: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM objectives WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;

    let Some((current_state, current_json)) = current else {
        tx.execute(
            "INSERT INTO objectives(id, state, payload_json) VALUES (?1, ?2, ?3)",
            params![id, objective_state_name(objective.etat), incoming_json],
        )
        .map_err(StoreError::Sql)?;
        return Ok(());
    };

    let mut current_objective: ObjectifCoordonne =
        serde_json::from_slice(&current_json).map_err(StoreError::Json)?;
    if parse_objective_state(&current_state)? != current_objective.etat {
        return Err(StoreError::Corrupt(
            "état objectif divergent de son payload",
        ));
    }
    if current_objective == *objective {
        return Ok(());
    }
    current_objective
        .transition(objective.etat, objective.mis_a_jour_at)
        .map_err(|_| StoreError::Conflict("transition objectif interdite"))?;
    if current_objective != *objective {
        return Err(StoreError::Conflict(
            "payload objectif incohérent avec la transition",
        ));
    }
    let changed = tx
        .execute(
            "UPDATE objectives SET state = ?1, payload_json = ?2\n\
             WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
            params![
                objective_state_name(objective.etat),
                incoming_json,
                id,
                current_state,
                current_json
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::Conflict(
            "objectif modifié concurremment pendant la transition",
        ))
    }
}

fn objective_state_name(state: EtatObjectif) -> &'static str {
    match state {
        EtatObjectif::Ouvert => "ouvert",
        EtatObjectif::EnCoordination => "en_coordination",
        EtatObjectif::AEvaluer => "a_evaluer",
        EtatObjectif::Synthetise => "synthetise",
        EtatObjectif::Clos => "clos",
    }
}

fn parse_objective_state(value: &str) -> Result<EtatObjectif, StoreError> {
    match value {
        "ouvert" => Ok(EtatObjectif::Ouvert),
        "en_coordination" => Ok(EtatObjectif::EnCoordination),
        "a_evaluer" => Ok(EtatObjectif::AEvaluer),
        "synthetise" => Ok(EtatObjectif::Synthetise),
        "clos" => Ok(EtatObjectif::Clos),
        _ => Err(StoreError::Corrupt("état objectif inconnu")),
    }
}

fn delegation_state_name(state: EtatDelegation) -> &'static str {
    match state {
        EtatDelegation::Creee => "creee",
        EtatDelegation::AEvaluer => "a_evaluer",
        EtatDelegation::Terminee => "terminee",
        EtatDelegation::Annulee => "annulee",
    }
}

fn parse_delegation_state(value: &str) -> Result<EtatDelegation, StoreError> {
    match value {
        "creee" => Ok(EtatDelegation::Creee),
        "a_evaluer" => Ok(EtatDelegation::AEvaluer),
        "terminee" => Ok(EtatDelegation::Terminee),
        "annulee" => Ok(EtatDelegation::Annulee),
        _ => Err(StoreError::Corrupt("état délégation inconnu")),
    }
}

fn parse_outbox_state(value: &str) -> Result<EtatOutboxDelegation, StoreError> {
    match value {
        "prepared" => Ok(EtatOutboxDelegation::Prepared),
        "outcome_unknown" => Ok(EtatOutboxDelegation::OutcomeUnknown),
        "accepted" => Ok(EtatOutboxDelegation::Accepted),
        "rejected" => Ok(EtatOutboxDelegation::Rejected),
        _ => Err(StoreError::Corrupt("état outbox inconnu")),
    }
}

fn validate_activation_proposal(
    decision: &DecisionCoordination,
    approval: &ApprobationActivation,
) -> Result<(), StoreError> {
    decision.verifier().map_err(StoreError::Domain)?;
    if decision.kind != TypeDecision::ReveillerProfil
        || decision.etat != EtatDecision::Proposee
        || decision.objectif_id != approval.objective_id
        || approval.profile_id.trim().is_empty()
        || approval.context_scope.trim().is_empty()
        || approval.parameters.trim().is_empty()
        || approval.expires_at <= 0
        || approval.consumed_at.is_some()
        || approval.profile_hash.is_empty()
        || approval.context_hash.is_empty()
        || approval.command_id.is_nil()
    {
        return Err(StoreError::Invalid("proposition d'activation invalide"));
    }
    if approval.actor != "local_human" {
        return Err(StoreError::Domain(DomainError::DonneeInvalide(
            "acteur d'approbation invalide",
        )));
    }
    Ok(())
}

fn validate_approved_spawn_order(
    approval: &ApprobationActivation,
    bytes: &[u8],
) -> Result<(), StoreError> {
    let order: ApprovedSpawnOrder = serde_json::from_slice(bytes).map_err(StoreError::Json)?;
    if order.kind != "SpawnOrder"
        || order.command_id != approval.command_id.to_string()
        || order.agent_type != approval.profile_id
        || order.issued_at <= 0
        || order.deadline_at <= order.issued_at
        || order.cwd.is_empty()
        || approval.parameters.as_bytes() != bytes
    {
        return Err(StoreError::Invalid(
            "SpawnOrder non conforme à l'approbation",
        ));
    }
    let _ = (order.name, order.persistent);
    Ok(())
}

fn activation_issue(
    command_id: Uuid,
    outcome: &SpawnOutcome,
) -> Result<(EtatActivationOutbox, Vec<u8>), StoreError> {
    let expected = command_id.to_string();
    let value = match outcome {
        SpawnOutcome::Accepted { command_id, name } => {
            if command_id != &expected {
                return Err(StoreError::Invalid("command_id d'issue divergent"));
            }
            json!({"kind":"accepted","command_id":command_id,"name":name})
        }
        SpawnOutcome::Rejected { command_id, reason } => {
            if command_id != &expected {
                return Err(StoreError::Invalid("command_id d'issue divergent"));
            }
            json!({"kind":"rejected","command_id":command_id,"reason":reason})
        }
        SpawnOutcome::Idempotency(IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id,
        }) => {
            return serde_json::to_vec(&json!({"kind":"outcome_unknown","expires_at":expires_at,"delivery_id":delivery_id}))
                .map(|bytes| (EtatActivationOutbox::OutcomeUnknown, bytes))
                .map_err(StoreError::Json);
        }
        SpawnOutcome::Idempotency(issue) => {
            json!({"kind":"idempotency","issue":encode_issue(issue)})
        }
    };
    serde_json::to_vec(&value)
        .map(|bytes| (EtatActivationOutbox::Applied, bytes))
        .map_err(StoreError::Json)
}

fn decision_state_name(state: EtatDecision) -> &'static str {
    match state {
        EtatDecision::Proposee => "proposed",
        EtatDecision::Approuvee => "approved",
        EtatDecision::Refusee => "rejected",
        EtatDecision::Appliquee => "applied",
    }
}

fn parse_decision_state(value: &str) -> Result<EtatDecision, StoreError> {
    match value {
        "proposed" => Ok(EtatDecision::Proposee),
        "approved" => Ok(EtatDecision::Approuvee),
        "rejected" => Ok(EtatDecision::Refusee),
        "applied" => Ok(EtatDecision::Appliquee),
        _ => Err(StoreError::Corrupt("état décision inconnu")),
    }
}

fn parse_activation_state(value: &str) -> Result<EtatActivationOutbox, StoreError> {
    match value {
        "dispatching" => Ok(EtatActivationOutbox::Dispatching),
        "outcome_unknown" => Ok(EtatActivationOutbox::OutcomeUnknown),
        "applied" => Ok(EtatActivationOutbox::Applied),
        _ => Err(StoreError::Corrupt("état activation inconnu")),
    }
}

fn outbox_state_name(state: EtatOutboxDelegation) -> &'static str {
    match state {
        EtatOutboxDelegation::Prepared => "prepared",
        EtatOutboxDelegation::OutcomeUnknown => "outcome_unknown",
        EtatOutboxDelegation::Accepted => "accepted",
        EtatOutboxDelegation::Rejected => "rejected",
    }
}

fn encode_issue(issue: &IdempotencyIssue) -> Value {
    match issue {
        IdempotencyIssue::Accepted { expires_at } => {
            json!({"kind":"accepted","expires_at":expires_at})
        }
        IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        } => json!({
            "kind":"rejected","category":category,"reason":reason,"expires_at":expires_at
        }),
        IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id,
        } => json!({
            "kind":"outcome_unknown","expires_at":expires_at,"delivery_id":delivery_id
        }),
        IdempotencyIssue::EnvelopeMismatch => json!({"kind":"envelope_mismatch"}),
        IdempotencyIssue::IdempotencyExpired => json!({"kind":"idempotency_expired"}),
        IdempotencyIssue::InvalidIssuedAt => json!({"kind":"invalid_issued_at"}),
    }
}

struct RawPending {
    objective_id: String,
    delegation_id: String,
    message_id: String,
    issuer_scope: String,
    issued_at: i64,
    target: String,
    body_bytes: Vec<u8>,
    reply: i64,
    timeout_secs: i64,
    deadline_contractuelle: i64,
    body_hash: Vec<u8>,
    message_bytes: Vec<u8>,
    state: String,
    attempted_at: Option<i64>,
    retry_until: i64,
    dedup_retained_until: i64,
}

fn raw_pending_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawPending> {
    Ok(RawPending {
        objective_id: row.get(0)?,
        delegation_id: row.get(1)?,
        message_id: row.get(2)?,
        issuer_scope: row.get(3)?,
        issued_at: row.get(4)?,
        target: row.get(5)?,
        body_bytes: row.get(6)?,
        reply: row.get(7)?,
        timeout_secs: row.get(8)?,
        deadline_contractuelle: row.get(9)?,
        body_hash: row.get(10)?,
        message_bytes: row.get(11)?,
        state: row.get(12)?,
        attempted_at: row.get(13)?,
        retry_until: row.get(14)?,
        dedup_retained_until: row.get(15)?,
    })
}

impl TryFrom<RawPending> for PendingDelegationOutbox {
    type Error = StoreError;

    fn try_from(raw: RawPending) -> Result<Self, Self::Error> {
        let pending = Self {
            objective_id: parse_uuid(&raw.objective_id)?,
            delegation_id: parse_uuid(&raw.delegation_id)?,
            message_id: parse_uuid(&raw.message_id)?,
            issuer_scope: raw.issuer_scope,
            issued_at: raw.issued_at,
            target: raw.target,
            body_bytes: raw.body_bytes,
            reply: match raw.reply {
                0 => false,
                1 => true,
                _ => return Err(StoreError::Corrupt("booléen reply invalide")),
            },
            timeout_secs: u64::try_from(raw.timeout_secs)
                .map_err(|_| StoreError::Corrupt("timeout_secs invalide"))?,
            deadline_contractuelle: raw.deadline_contractuelle,
            body_hash: raw.body_hash,
            message_bytes: raw.message_bytes,
            state: parse_outbox_state(&raw.state)?,
            attempted_at: raw.attempted_at,
            retry_until: raw.retry_until,
            dedup_retained_until: raw.dedup_retained_until,
        };
        pending.validate().map_err(StoreError::Outbox)?;
        Ok(pending)
    }
}

struct RawRecovery {
    pending: RawPending,
    objective_state: String,
    delegation_state: String,
    last_issue_json: Option<Vec<u8>>,
    issue_observed_at: Option<i64>,
}

impl TryFrom<RawRecovery> for RecoverySnapshot {
    type Error = StoreError;

    fn try_from(raw: RawRecovery) -> Result<Self, Self::Error> {
        Ok(Self {
            outbox: raw.pending.try_into()?,
            objective_state: parse_objective_state(&raw.objective_state)?,
            delegation_state: parse_delegation_state(&raw.delegation_state)?,
            last_issue: raw
                .last_issue_json
                .map(|bytes| serde_json::from_slice(&bytes).map_err(StoreError::Json))
                .transpose()?,
            issue_observed_at: raw.issue_observed_at,
        })
    }
}

fn parse_uuid(value: &str) -> Result<Uuid, StoreError> {
    Uuid::parse_str(value).map_err(|_| StoreError::Corrupt("UUID stocké invalide"))
}

#[derive(Debug)]
pub enum StoreError {
    Invalid(&'static str),
    Conflict(&'static str),
    EnvelopeMismatch,
    NotFound(&'static str),
    Corrupt(&'static str),
    UnsupportedSchema { found: i64, supported: i64 },
    Io(std::io::Error),
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    Outbox(OutboxError),
    Domain(DomainError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "store invalide : {reason}"),
            Self::Conflict(reason) => write!(formatter, "conflit store : {reason}"),
            Self::EnvelopeMismatch => write!(formatter, "enveloppe idempotente divergente"),
            Self::NotFound(reason) => write!(formatter, "store introuvable : {reason}"),
            Self::Corrupt(reason) => write!(formatter, "store corrompu : {reason}"),
            Self::UnsupportedSchema { found, supported } => write!(
                formatter,
                "schéma SQLite {found} non supporté (maximum {supported})"
            ),
            Self::Io(source) => write!(formatter, "I/O store impossible : {source}"),
            Self::Sql(source) => write!(formatter, "SQLite impossible : {source}"),
            Self::Json(source) => write!(formatter, "JSON store impossible : {source}"),
            Self::Outbox(source) => write!(formatter, "{source}"),
            Self::Domain(source) => write!(formatter, "domaine invalide : {source:?}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Sql(source) => Some(source),
            Self::Json(source) => Some(source),
            Self::Outbox(source) => Some(source),
            Self::Domain(_) => None,
            Self::Invalid(_)
            | Self::Conflict(_)
            | Self::EnvelopeMismatch
            | Self::NotFound(_)
            | Self::Corrupt(_)
            | Self::UnsupportedSchema { .. } => None,
        }
    }
}
