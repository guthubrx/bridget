//! Saga native durable. Les identités et effets appartiennent au daemon.
use bridget_transport::protocol::{
    FallbackTarget, NativeDelegationRequest, ResolvedAgentDefinition, SpawnPosture,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub(crate) const MAX_AUTO_ADMISSION_CANONICAL_BYTES: usize = 512 * 1024;
pub(crate) const MAX_AUTO_ADMISSION_METADATA_BYTES: usize = 16 * 1024;
pub(crate) const MAX_ACTIVE_ADMISSIONS: usize = 128;
pub(crate) const MAX_ACTIVE_ADMISSIONS_PER_OWNER: usize = 16;

const NATIVE_HOLDS_SLOT_SQL: &str = "json_extract(payload,'$.state') IN ('queued','starting','mission_pending','working','waiting_for_children','cancelling') OR (json_extract(payload,'$.state') IN ('result_available','failed','cancelled') AND coalesce(json_extract(payload,'$.guarded'),0)=1 AND coalesce(json_extract(payload,'$.cleanup_done'),0)=0)";
const AUTO_HOLDS_SLOT_SQL: &str = "state IN ('pending','decision_started','chosen')";

/// Compteur éphémère commun aux lectures en attente. Chaque opération est O(1).
/// Ordre des verrous : état puis compteur. Ne jamais garder le compteur en prenant l'état.
#[derive(Clone, Default)]
pub(crate) struct WaitEpoch(Arc<(Mutex<u64>, Condvar)>);

impl WaitEpoch {
    pub fn get(&self) -> u64 {
        *self.0.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn bump(&self) {
        let mut epoch = self.0.0.lock().unwrap_or_else(|e| e.into_inner());
        *epoch = epoch.wrapping_add(1);
        self.0.1.notify_all();
    }

    pub fn wait_changed(&self, seen: u64, slice: Duration) {
        let epoch = self.0.0.lock().unwrap_or_else(|e| e.into_inner());
        drop(
            self.0
                .1
                .wait_timeout_while(epoch, slice, |epoch| *epoch == seen)
                .unwrap_or_else(|e| e.into_inner()),
        );
    }
}

#[path = "delegation_lineage.rs"]
mod lineage;
pub(crate) use lineage::{MAX_RETAINED_TASKS, ProjectionMutation};

pub(crate) struct DelegationStore {
    conn: Connection,
    changes: std::cell::RefCell<std::collections::BTreeMap<String, ProjectionMutation>>,
    notify: std::sync::mpsc::SyncSender<()>,
    receiver: std::cell::RefCell<Option<std::sync::mpsc::Receiver<()>>>,
    wait_epoch: WaitEpoch,
}

/// Ligne durable155. L'identifiant fournisseur contient son JSON canonique,
/// y compris les guillemets d'une chaîne ; un entier reste distinct d'une chaîne.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ChildRequest {
    pub request_id: String,
    pub task_id: String,
    pub child_instance: String,
    pub provider_session: String,
    pub turn_id: String,
    pub provider_request_id: String,
    pub content_hash: String,
    pub kind: String,
    pub verdict: String,
    pub state: String,
    pub outcome: Option<String>,
    pub summary: String,
    pub facts_json: String,
    pub model_text: String,
    pub offered_json: String,
    pub created_at: i64,
    pub deadline_at: i64,
    pub decision: Option<String>,
    pub answer_json: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<i64>,
    pub delivery_state: String,
    pub void_reason: Option<String>,
    pub sent_at: Option<i64>,
    pub delivered_at: Option<i64>,
    pub inbox_item_id: Option<String>,
    pub inbox_settled: bool,
}

impl ChildRequest {
    /// O(B), B taille bornée de la ligne, sans interprétation des faits stockés.
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            request_id: row.get("request_id")?,
            task_id: row.get("task_id")?,
            child_instance: row.get("child_instance")?,
            provider_session: row.get("provider_session")?,
            turn_id: row.get("turn_id")?,
            provider_request_id: row.get("provider_request_id")?,
            content_hash: row.get("content_hash")?,
            kind: row.get("kind")?,
            verdict: row.get("verdict")?,
            state: row.get("state")?,
            outcome: row.get("outcome")?,
            summary: row.get("summary")?,
            facts_json: row.get("facts_json")?,
            model_text: row.get("model_text")?,
            offered_json: row.get("offered_json")?,
            created_at: row.get("created_at")?,
            deadline_at: row.get("deadline_at")?,
            decision: row.get("decision")?,
            answer_json: row.get("answer_json")?,
            decided_by: row.get("decided_by")?,
            decided_at: row.get("decided_at")?,
            delivery_state: row.get("delivery_state")?,
            void_reason: row.get("void_reason")?,
            sent_at: row.get("sent_at")?,
            delivered_at: row.get("delivered_at")?,
            inbox_item_id: row.get("inbox_item_id")?,
            inbox_settled: row.get("inbox_settled")?,
        })
    }
}

/// Ligne unique d'arrêt158A ; distincte de la pause et des tâches natives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HaltState {
    pub halted: bool,
    pub since: Option<i64>,
    pub by: Option<String>,
    pub reason: Option<String>,
    pub generation: u64,
}

/// État durable d'une admission automatique, distinct de toute Task native.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AutoAdmissionState {
    Pending,
    DecisionStarted,
    Chosen,
    Admitted,
    Refused,
}

impl AutoAdmissionState {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::DecisionStarted => "decision_started",
            Self::Chosen => "chosen",
            Self::Admitted => "admitted",
            Self::Refused => "refused",
        }
    }

    fn from_sql(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "decision_started" => Some(Self::DecisionStarted),
            "chosen" => Some(Self::Chosen),
            "admitted" => Some(Self::Admitted),
            "refused" => Some(Self::Refused),
            _ => None,
        }
    }
}

/// Information projetable, mais jamais une autorisation de lancement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AutoAdmissionMode {
    Explicit,
    Automatic,
    Fallback,
}

impl AutoAdmissionMode {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Automatic => "automatic",
            Self::Fallback => "fallback",
        }
    }

    fn from_sql(value: &str) -> Option<Self> {
        match value {
            "explicit" => Some(Self::Explicit),
            "automatic" => Some(Self::Automatic),
            "fallback" => Some(Self::Fallback),
            _ => None,
        }
    }
}

/// Reçu 163. Le canon auto et celui du Delegate résolu ne sont pas
/// interchangeables ; le premier seul porte l'idempotence de l'appelant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AutoAdmissionReceipt {
    pub owner: String,
    pub request_id: String,
    pub canonical: Vec<u8>,
    pub metadata: Vec<u8>,
    pub state: AutoAdmissionState,
    pub coordinator_epoch: i64,
    pub revision: i64,
    pub policy_revision: i64,
    pub policy_digest: String,
    pub candidate_id: Option<String>,
    pub delegate_canonical: Option<Vec<u8>>,
    pub mode: Option<AutoAdmissionMode>,
    pub reason: Option<String>,
    pub task_id: Option<String>,
    pub refusal_code: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AutoAdmissionReceipt {
    fn valid_atom(value: &str, max: usize) -> bool {
        !value.is_empty()
            && value.len() <= max
            && !value.chars().any(bridget_core::is_disallowed_control)
    }

    fn valid_digest(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    fn validate(&self) -> Result<(), String> {
        if !Self::valid_atom(&self.owner, 256)
            || !Self::valid_atom(&self.request_id, 256)
            || self.canonical.is_empty()
            || self.canonical.len() > MAX_AUTO_ADMISSION_CANONICAL_BYTES
            || self.metadata.is_empty()
            || self.metadata.len() > MAX_AUTO_ADMISSION_METADATA_BYTES
            || !serde_json::from_slice::<serde_json::Value>(&self.metadata)
                .ok()
                .is_some_and(|value| value.is_object())
            || self.coordinator_epoch < 0
            || self.revision < 0
            || self.policy_revision <= 0
            || !Self::valid_digest(&self.policy_digest)
            || self.created_at < 0
            || self.updated_at < self.created_at
            || self
                .candidate_id
                .as_deref()
                .is_some_and(|value| !bridget_transport::protocol::is_auto_candidate_id(value))
            || self.delegate_canonical.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > MAX_AUTO_ADMISSION_CANONICAL_BYTES
            })
            || self
                .reason
                .as_deref()
                .is_some_and(|value| !bridget_transport::protocol::is_auto_admission_reason(value))
            || self
                .task_id
                .as_deref()
                .is_some_and(|value| !Self::valid_atom(value, 256))
            || self
                .refusal_code
                .as_deref()
                .is_some_and(|value| !Self::valid_atom(value, 128))
        {
            return Err("auto_admission_invalid".into());
        }
        let chosen =
            self.candidate_id.is_some() && self.delegate_canonical.is_some() && self.mode.is_some();
        match self.state {
            AutoAdmissionState::Pending | AutoAdmissionState::DecisionStarted
                if chosen || self.task_id.is_some() || self.refusal_code.is_some() =>
            {
                Err("auto_admission_invalid".into())
            }
            AutoAdmissionState::Chosen
                if !chosen || self.task_id.is_some() || self.refusal_code.is_some() =>
            {
                Err("auto_admission_invalid".into())
            }
            AutoAdmissionState::Admitted
                if !chosen || self.task_id.is_none() || self.refusal_code.is_some() =>
            {
                Err("auto_admission_invalid".into())
            }
            AutoAdmissionState::Refused
                if self.refusal_code.is_none() || self.task_id.is_some() =>
            {
                Err("auto_admission_invalid".into())
            }
            _ => Ok(()),
        }
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let state: String = row.get("state")?;
        let mode: Option<String> = row.get("mode")?;
        let state = AutoAdmissionState::from_sql(&state).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(4, "state".into(), rusqlite::types::Type::Text)
        })?;
        let mode = match mode.as_deref() {
            None => None,
            Some(value) => Some(AutoAdmissionMode::from_sql(value).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(11, "mode".into(), rusqlite::types::Type::Text)
            })?),
        };
        Ok(Self {
            owner: row.get("owner")?,
            request_id: row.get("request_id")?,
            canonical: row.get("canonical")?,
            metadata: row.get("metadata")?,
            state,
            coordinator_epoch: row.get("coordinator_epoch")?,
            revision: row.get("revision")?,
            policy_revision: row.get("policy_revision")?,
            policy_digest: row.get("policy_digest")?,
            candidate_id: row.get("candidate_id")?,
            delegate_canonical: row.get("delegate_canonical")?,
            mode,
            reason: row.get("reason")?,
            task_id: row.get("task_id")?,
            refusal_code: row.get("refusal_code")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

fn auto_admission_receipt_in(
    tx: &Transaction<'_>,
    owner: &str,
    request_id: &str,
) -> Result<Option<AutoAdmissionReceipt>, String> {
    tx.query_row(
        "SELECT owner,request_id,canonical,metadata,state,coordinator_epoch,revision,policy_revision,policy_digest,candidate_id,delegate_canonical,mode,reason,task_id,refusal_code,created_at,updated_at FROM auto_admission_receipts WHERE owner=?1 AND request_id=?2",
        params![owner, request_id],
        AutoAdmissionReceipt::from_row,
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn tombstoned_in(tx: &Transaction<'_>, owner: &str, request_id: &str) -> Result<bool, String> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM native_delegation_tombstones WHERE owner=?1 AND request_id=?2)",
        params![owner, request_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn native_admission_key_reserved_in(
    tx: &Transaction<'_>,
    owner: &str,
    request_id: &str,
) -> Result<bool, String> {
    tx.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM native_delegations
            WHERE json_extract(payload,'$.owner')=?1 AND request_id=?2
            UNION ALL
            SELECT 1 FROM native_delegation_refusals
            WHERE owner=?1 AND request_id=?2
        )",
        params![owner, request_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn retained_admission_keys_in(tx: &Transaction<'_>) -> Result<usize, String> {
    tx.query_row(
        "SELECT count(*) FROM (
            SELECT json_extract(payload,'$.owner') AS owner,request_id FROM native_delegations
            UNION
            SELECT owner,request_id FROM auto_admission_receipts
        )",
        [],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn admission_counts_for_owner_in(
    tx: &Transaction<'_>,
    owner: &str,
) -> Result<(usize, usize, usize), String> {
    let active_keys = active_admission_keys_sql();
    let active = tx
        .query_row(
            &format!("SELECT count(*) FROM ({active_keys})"),
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let owner_active = tx
        .query_row(
            &format!("SELECT count(*) FROM ({active_keys}) WHERE owner=?1"),
            [owner],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    Ok((active, owner_active, retained_admission_keys_in(tx)?))
}

fn active_admission_keys_sql() -> String {
    format!(
        "SELECT json_extract(payload,'$.owner') AS owner,request_id
         FROM native_delegations INDEXED BY native_delegations_state
         WHERE ({NATIVE_HOLDS_SLOT_SQL})
         UNION
         SELECT owner,request_id FROM auto_admission_receipts
         WHERE {AUTO_HOLDS_SLOT_SQL}"
    )
}

fn purge_auto_refusals_in(
    tx: &Transaction<'_>,
    now: i64,
    limit: usize,
    older_than: Option<i64>,
) -> Result<usize, String> {
    if limit == 0 {
        return Ok(0);
    }
    let limit = i64::try_from(limit).map_err(|_| "auto_admission_invalid")?;
    let keys = match older_than {
        Some(older_than) => {
            let mut statement = tx
                .prepare(
                    "SELECT receipt.owner,receipt.request_id
                     FROM auto_admission_receipts AS receipt
                     WHERE receipt.state='refused' AND receipt.task_id IS NULL
                       AND receipt.updated_at<?1
                       AND NOT EXISTS (
                           SELECT 1 FROM native_delegations AS task
                           WHERE json_extract(task.payload,'$.owner')=receipt.owner
                             AND task.request_id=receipt.request_id
                       )
                     ORDER BY receipt.updated_at,receipt.owner,receipt.request_id
                     LIMIT ?2",
                )
                .map_err(|error| error.to_string())?;
            statement
                .query_map(params![older_than, limit], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| error.to_string())?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|error| error.to_string())?
        }
        None => {
            let mut statement = tx
                .prepare(
                    "SELECT receipt.owner,receipt.request_id
                     FROM auto_admission_receipts AS receipt
                     WHERE receipt.state='refused' AND receipt.task_id IS NULL
                       AND NOT EXISTS (
                           SELECT 1 FROM native_delegations AS task
                           WHERE json_extract(task.payload,'$.owner')=receipt.owner
                             AND task.request_id=receipt.request_id
                       )
                     ORDER BY receipt.updated_at,receipt.owner,receipt.request_id
                     LIMIT ?1",
                )
                .map_err(|error| error.to_string())?;
            statement
                .query_map([limit], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| error.to_string())?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|error| error.to_string())?
        }
    };
    for (owner, request_id) in &keys {
        // La pierre est inscrite avant la suppression, dans la même transaction :
        // une erreur de suppression annule aussi ce refus request_expired.
        tx.execute(
            "INSERT INTO native_delegation_tombstones(owner,request_id,purged_at) VALUES(?1,?2,?3)
             ON CONFLICT(owner,request_id) DO UPDATE SET purged_at=excluded.purged_at",
            params![owner, request_id, now],
        )
        .map_err(|error| error.to_string())?;
        let deleted = tx
            .execute(
                "DELETE FROM auto_admission_receipts
                 WHERE owner=?1 AND request_id=?2 AND state='refused' AND task_id IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM native_delegations AS task
                       WHERE json_extract(task.payload,'$.owner')=?1 AND task.request_id=?2
                   )",
                params![owner, request_id],
            )
            .map_err(|error| error.to_string())?;
        if deleted != 1 {
            return Err("auto_admission_purge_conflict".into());
        }
    }
    Ok(keys.len())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RoundRecord {
    pub round: u32,
    pub request_id: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspaceStrategy {
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspaceState {
    Active,
    Kept,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkspaceCommit {
    pub sha: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkspaceRecord {
    pub strategy: WorkspaceStrategy,
    pub state: WorkspaceState,
    pub toplevel: String,
    pub path: String,
    pub branch: String,
    pub base_commit: String,
    pub head_commit: Option<String>,
    pub branch_commit: Option<String>,
    pub head_on_branch: Option<bool>,
    pub commit_count: Option<u32>,
    pub commits: Vec<WorkspaceCommit>,
    pub truncated: bool,
    pub dirty: Option<bool>,
}

/// Trace durable de la sélection, distincte de l'enveloppe canonique originale.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Substitution {
    pub requested: FallbackTarget,
    pub selected: FallbackTarget,
    pub reason: String,
    /// Objets {agent_type, model, code} produits par l'admission, sans DTO neuf.
    pub tried: Vec<serde_json::Value>,
}

/// Phase durable de la relance160 ; seule la tâche porte le checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum QuotaWaitPhase {
    Waiting,
    Relaunching,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct QuotaWait {
    pub since: i64,
    pub reset_at: i64,
    pub round: u8,
    pub identity: String,
    pub execution_id: String,
    pub phase: QuotaWaitPhase,
    pub until: Option<i64>,
    pub resume_execution_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct QuotaHold {
    pub since: i64,
    pub error: String,
    pub execution_id: String,
}

/// Huit rondes × trois exécutions quota + deux reprises processus sur toute
/// la tâche = 26. Les autres reprises consomment aussi la borne64 ; saturation
/// refuse avant dispatch, sans supprimer une réservation ou preuve active.
pub(crate) const MAX_REMOTE_RESULT_SLOTS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RemoteResultReceipt {
    pub worker: bridget_transport::protocol::WorkerIdentity,
    pub identity: bridget_transport::protocol::WorkerResultIdentity,
    pub message_id: String,
    pub digest: String,
    pub disposition: bridget_transport::protocol::RemoteResultDisposition,
    #[serde(default)]
    pub conflict_seen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Task {
    pub task_id: String,
    #[serde(default)]
    pub root_owner_agent_id: String,
    #[serde(default)]
    pub parent_task_id: Option<String>,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    pub owner: String,
    pub owner_instance: String,
    pub origin_owner_instance: String,
    #[serde(default)]
    pub parent_execution_id: Option<String>,
    pub request: NativeDelegationRequest,
    #[serde(default)]
    pub substitution: Option<Substitution>,
    #[serde(default)]
    pub child_requests: Option<String>,
    #[serde(default)]
    pub permission_snapshot: Option<bridget_transport::protocol::NativePermissionSnapshot>,
    #[serde(default)]
    pub effective_posture: Option<SpawnPosture>,
    pub definition: ResolvedAgentDefinition,
    pub cwd: String,
    /// Placement figé162 ; l'absence est locale pour ce lecteur seulement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
    #[serde(default)]
    pub remote_prepared: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_result_slots: Vec<bridget_transport::protocol::WorkerResultSlot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_result_receipts: Vec<RemoteResultReceipt>,
    #[serde(default)]
    pub workspace: Option<WorkspaceRecord>,
    /// Mission dont le rapport est rangé, y compris si Git était indisponible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_report_mission: Option<String>,
    pub child: String,
    pub child_instance: Option<String>,
    pub mission: String,
    #[serde(default)]
    pub mission_deadline_at: Option<i64>,
    pub created_at: i64,
    pub state: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub result_sent: bool,
    #[serde(default)]
    pub cleanup_done: bool,
    #[serde(default)]
    pub failure_sent: bool,
    /// Délai de réponse choisi à la délégation (secondes) ; absent = 3600.
    #[serde(default)]
    pub reply_timeout: Option<i64>,
    /// Preuve durable que la remise du résultat ou de l'avis d'échec a été accusée.
    #[serde(default)]
    pub delivery_acked: bool,
    /// Lecture explicite du parent ; distincte de la preuve de livraison.
    #[serde(default)]
    pub acknowledged_at: Option<i64>,
    /// Enveloppe du premier lancement ; absente dans les payloads antérieurs.
    #[serde(default)]
    pub launch_issued_at: Option<i64>,
    /// Enveloppe du premier envoi de mission, conservée pour ses rejeux.
    #[serde(default)]
    pub mission_issued_at: Option<i64>,
    /// Reprises après redémarrage, distinctes des reprises de pause.
    #[serde(default)]
    pub restart_resumes: u8,
    /// Historique append-only160 ; la longueur est le compteur de rondes.
    #[serde(default)]
    pub quota_waits: Vec<QuotaWait>,
    /// Décision d'échec bornée160 ; l'opt-in reste dans request.
    #[serde(default)]
    pub quota_hold: Option<QuotaHold>,
    /// Arrêts déjà confirmés pour l'annulation de cet arbre.
    #[serde(default)]
    pub cancel_stopped: Vec<String>,
    #[serde(default)]
    pub round: u32,
    #[serde(default)]
    pub idle_deadline_at: Option<i64>,
    #[serde(default)]
    pub round_body: Option<String>,
    #[serde(default)]
    pub rounds: Vec<RoundRecord>,
    #[serde(default)]
    pub guarded: bool,
    #[serde(default)]
    pub guard_released: bool,
    #[serde(default)]
    pub unacked_rounds: Vec<u32>,
}

impl Task {
    /// O(W+B), W attentes quota (au plus2), B octets du motif ; mémoire O(B).
    /// Le checkpoint d'échec reste interne tant que la clôture de relance est ouverte.
    pub(crate) fn projected_error(&self) -> Option<&str> {
        if self.state == "working"
            && self
                .quota_waits
                .iter()
                .any(|wait| wait.phase != QuotaWaitPhase::Done)
        {
            return None;
        }
        let error = self.error.as_deref()?;
        if let Ok(serde_json::Value::Object(object)) =
            serde_json::from_str::<serde_json::Value>(error)
            && object.len() == 2
            && object.get("code").and_then(serde_json::Value::as_str) == Some("usage_limit")
            && object
                .get("limit_reset_at")
                .and_then(serde_json::Value::as_i64)
                .is_some_and(|at| at >= 0)
        {
            return Some("usage_limit");
        }
        Some(error)
    }

    pub fn terminal(&self) -> bool {
        matches!(
            self.state.as_str(),
            "result_available" | "failed" | "cancelled"
        )
    }

    pub fn round_no(&self) -> u32 {
        self.round.max(1)
    }

    pub fn result_message_id(&self) -> String {
        self.result_message_id_for_round(self.round_no())
    }

    /// O(L), L longueur de l'identifiant ; même règle pour les ACK passés.
    pub fn result_message_id_for_round(&self, round: u32) -> String {
        if round <= 1 {
            format!("native-result-{}", self.task_id)
        } else {
            format!("native-result-{}-r{round}", self.task_id)
        }
    }

    /// O(1+B), B octets des seuls labels/identifiants ; projection froide158A.
    /// Partagé par la vue globale et la ligne rendue après l'annulation humaine.
    pub(crate) fn overview_row(&self, now: i64) -> serde_json::Value {
        serde_json::json!({
            "task_id":self.task_id,"owner":self.owner,"child":self.child,"state":self.state,
            "age_secs":now.saturating_sub(self.created_at),
            "deadline_in_secs":self.mission_deadline_at.map(|deadline| deadline.saturating_sub(now)),
            "parent_task_id":self.parent_task_id,
        })
    }

    pub fn holds_slot(&self) -> bool {
        !self.terminal() || (self.guarded && !self.cleanup_done)
    }

    pub fn continuable(&self) -> bool {
        self.state == "result_available"
            && self.result_sent
            && self.idle_deadline_at.is_some()
            && !self.guard_released
    }
}

impl DelegationStore {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let tx =
            rusqlite::Transaction::new_unchecked(&conn, rusqlite::TransactionBehavior::Immediate)?;
        tx.execute_batch("CREATE TABLE IF NOT EXISTS native_delegations (
            task_id TEXT PRIMARY KEY, owner_instance TEXT NOT NULL,
            request_id TEXT NOT NULL, canonical BLOB NOT NULL, payload TEXT NOT NULL,
            UNIQUE(owner_instance, request_id));
            CREATE UNIQUE INDEX IF NOT EXISTS native_delegations_mission ON native_delegations(json_extract(payload,'$.mission'));
            CREATE UNIQUE INDEX IF NOT EXISTS native_delegations_owner_request ON native_delegations(json_extract(payload,'$.owner'),request_id);
            CREATE INDEX IF NOT EXISTS native_delegations_failed_agent ON native_delegations(json_extract(payload,'$.owner'),json_extract(payload,'$.state'));
            CREATE TABLE IF NOT EXISTS native_delegation_grants (
            owner_instance TEXT PRIMARY KEY, cwd TEXT NOT NULL, posture TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS native_delegation_revocations (agent_id TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS native_delegation_refusals (
            owner TEXT NOT NULL, request_id TEXT NOT NULL, canonical BLOB NOT NULL,
            code TEXT NOT NULL, refused_at INTEGER NOT NULL, PRIMARY KEY(owner, request_id));
            CREATE TABLE IF NOT EXISTS native_delegation_past_missions (
            mission TEXT PRIMARY KEY, task_id TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS native_delegation_past_missions_task
            ON native_delegation_past_missions(task_id);
CREATE TABLE IF NOT EXISTS native_child_requests(
  request_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  child_instance TEXT NOT NULL,
  provider_session TEXT NOT NULL,
  turn_id TEXT NOT NULL,
  provider_request_id TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('approval_command','approval_file','elicitation_form','user_input')),
  verdict TEXT NOT NULL CHECK (verdict IN ('parent','human','forbidden')),
  state TEXT NOT NULL CHECK (state IN ('open','answered','expired','cancelled','refused')),
  outcome TEXT,
  summary TEXT NOT NULL,
  facts_json TEXT NOT NULL,
  model_text TEXT NOT NULL,
  offered_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  deadline_at INTEGER NOT NULL,
  decision TEXT,
  answer_json TEXT,
  decided_by TEXT,
  decided_at INTEGER,
  delivery_state TEXT NOT NULL DEFAULT 'none' CHECK (delivery_state IN ('none','pending','sent','delivered','void','unknown')),
  void_reason TEXT,
  sent_at INTEGER,
  delivered_at INTEGER,
  inbox_item_id TEXT,
  inbox_settled INTEGER NOT NULL DEFAULT 0,
  UNIQUE(task_id, child_instance, provider_session, turn_id, provider_request_id));
CREATE INDEX IF NOT EXISTS native_child_requests_open ON native_child_requests(state, deadline_at);
CREATE INDEX IF NOT EXISTS native_child_requests_task ON native_child_requests(task_id, state);
CREATE INDEX IF NOT EXISTS native_child_requests_inbox ON native_child_requests(inbox_settled, state);
CREATE TABLE IF NOT EXISTS native_delegation_halt(
  id INTEGER PRIMARY KEY CHECK (id = 1),
  halted INTEGER NOT NULL CHECK (halted IN (0, 1)),
  since INTEGER,
  by_actor TEXT,
  reason TEXT,
  generation INTEGER NOT NULL
);
INSERT OR IGNORE INTO native_delegation_halt(id, halted, generation) VALUES (1, 0, 0);
CREATE TABLE IF NOT EXISTS native_delegation_halt_commands(
  command_id TEXT PRIMARY KEY,
  applied_at INTEGER NOT NULL,
  response_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS auto_admission_receipts(
  owner TEXT NOT NULL,
  request_id TEXT NOT NULL,
  canonical BLOB NOT NULL CHECK (length(canonical) BETWEEN 1 AND 524288),
  metadata BLOB NOT NULL CHECK (length(metadata) BETWEEN 1 AND 16384),
  state TEXT NOT NULL CHECK (state IN ('pending','decision_started','chosen','admitted','refused')),
  coordinator_epoch INTEGER NOT NULL CHECK (coordinator_epoch >= 0),
  revision INTEGER NOT NULL CHECK (revision >= 0),
  policy_revision INTEGER NOT NULL CHECK (policy_revision > 0),
  policy_digest TEXT NOT NULL,
  candidate_id TEXT,
  delegate_canonical BLOB,
  mode TEXT CHECK (mode IS NULL OR mode IN ('explicit','automatic','fallback')),
  reason TEXT,
  task_id TEXT,
  refusal_code TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY(owner, request_id)
);
CREATE INDEX IF NOT EXISTS auto_admission_receipts_maintenance
ON auto_admission_receipts(coordinator_epoch, state, updated_at);
CREATE INDEX IF NOT EXISTS auto_admission_receipts_task
ON auto_admission_receipts(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS auto_admission_receipts_refused_purge
ON auto_admission_receipts(state, task_id, updated_at, owner, request_id)
WHERE state='refused' AND task_id IS NULL;
")?;
        const LEGACY_INDEX: &str = "CREATE INDEX native_delegations_state ON native_delegations(json_extract(payload,'$.state'))";
        const CURRENT_INDEX: &str = "CREATE INDEX native_delegations_state ON native_delegations(json_extract(payload,'$.state'),coalesce(json_extract(payload,'$.guarded'),0),coalesce(json_extract(payload,'$.cleanup_done'),0),owner_instance)";
        let existing: Option<String> = tx.query_row(
            "SELECT sql FROM sqlite_master WHERE type='index' AND name='native_delegations_state'",
            [],
            |row| row.get(0),
        ).optional()?;
        // Migration froide O(n log n) : un seul index existant est reconstruit.
        match existing {
            None => tx.execute_batch(CURRENT_INDEX)?,
            Some(sql) if sql.trim() == CURRENT_INDEX => {}
            Some(sql) if sql.trim() == LEGACY_INDEX => {
                tx.execute_batch("DROP INDEX native_delegations_state")?;
                tx.execute_batch(CURRENT_INDEX)?;
            }
            Some(_) => return Err(rusqlite::Error::InvalidQuery),
        }
        tx.commit()?;
        let (notify, receiver) = std::sync::mpsc::sync_channel(1);
        let store = Self {
            conn,
            changes: std::cell::RefCell::new(std::collections::BTreeMap::new()),
            notify,
            receiver: std::cell::RefCell::new(Some(receiver)),
            wait_epoch: WaitEpoch::default(),
        };
        store.initialize_lineage()?;
        Ok(store)
    }

    /// O(1), lecture de l'unique ligne ; aucun effet sur pause, tâches ou réveils.
    pub fn delegation_halt(
        &self,
    ) -> Result<HaltState, bridget_transport::protocol::DelegationControlRefusal> {
        use bridget_transport::protocol::DelegationControlRefusal as E;
        self.conn.query_row(
            "SELECT halted,since,by_actor,reason,generation FROM native_delegation_halt WHERE id=1",
            [], |row| Ok(HaltState { halted:row.get(0)?, since:row.get(1)?, by:row.get(2)?,
                reason:row.get(3)?, generation:row.get(4)? }),
        ).map_err(|_| E::StoreUnavailable)
    }

    /// O(r log r+B), r<=257 reçus, B<=512 caractères ; transaction immédiate.
    /// L'appelant est déjà autorisé. `cancelled` est le nombre d'arbres ciblés
    /// pour annulation à cet instant ; le callback ne s'évalue qu'après absence
    /// du reçu, sous cette transaction et le verrou d'état tenu par l'appelant.
    /// Aucun parcours d'annulation ici. Un rejeu ne modifie jamais le reçu ni l'état.
    pub fn set_delegation_halt(
        &self,
        command_id: &str,
        halted: bool,
        actor: &str,
        reason: Option<&str>,
        cancelled: impl FnOnce() -> Result<u32, bridget_transport::protocol::DelegationControlRefusal>,
        now: i64,
    ) -> Result<serde_json::Value, bridget_transport::protocol::DelegationControlRefusal> {
        use bridget_transport::protocol::DelegationControlRefusal as E;
        let tx = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )
        .map_err(|_| E::StoreUnavailable)?;
        let saved: Option<String> = tx
            .query_row(
                "SELECT response_json FROM native_delegation_halt_commands WHERE command_id=?1",
                [command_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| E::StoreUnavailable)?;
        if let Some(saved) = saved {
            // Décodage typé : un reçu malformé ne devient jamais une mutation neuve.
            let mut response: bridget_transport::protocol::DaemonToWrapper =
                serde_json::from_str(&saved).map_err(|_| E::StoreUnavailable)?;
            let bridget_transport::protocol::DaemonToWrapper::DelegationHaltState {
                replayed, ..
            } = &mut response
            else {
                return Err(E::StoreUnavailable);
            };
            *replayed = true;
            let response = serde_json::to_value(response).map_err(|_| E::StoreUnavailable)?;
            tx.commit().map_err(|_| E::StoreUnavailable)?;
            return Ok(response);
        }
        // Même convention que la pause087 : un booléen inchangé conserve auteur/date/motif.
        let mut state = self.delegation_halt()?;
        let cancelled = cancelled()?;
        if state.halted != halted {
            state.halted = halted;
            state.generation = state.generation.checked_add(1).ok_or(E::StoreUnavailable)?;
            state.since = halted.then_some(now);
            state.by = halted.then(|| actor.to_owned());
            state.reason = if halted {
                reason.map(|text| {
                    text.chars()
                        .take(crate::referent_control::MAX_REASON_CHARS)
                        .collect()
                })
            } else {
                None
            };
            tx.execute("UPDATE native_delegation_halt SET halted=?1,since=?2,by_actor=?3,reason=?4,generation=?5 WHERE id=1",
                params![state.halted,state.since,state.by,state.reason,state.generation])
                .map_err(|_| E::StoreUnavailable)?;
        }
        let response = bridget_transport::protocol::DaemonToWrapper::DelegationHaltState {
            halted: state.halted,
            since: state.since,
            by: state.by,
            reason: state.reason,
            generation: state.generation,
            cancelled,
            replayed: false,
        };
        let response = serde_json::to_value(response).map_err(|_| E::StoreUnavailable)?;
        tx.execute("INSERT INTO native_delegation_halt_commands(command_id,applied_at,response_json) VALUES(?1,?2,?3)",
            params![command_id,now,response.to_string()]).map_err(|_| E::StoreUnavailable)?;
        // Plus anciens d'abord ; rowid départage les commandes d'une même seconde.
        tx.execute("DELETE FROM native_delegation_halt_commands WHERE command_id IN (
            SELECT command_id FROM native_delegation_halt_commands ORDER BY applied_at DESC,rowid DESC LIMIT -1 OFFSET 256)",
            []).map_err(|_| E::StoreUnavailable)?;
        tx.commit().map_err(|_| E::StoreUnavailable)?;
        Ok(response)
    }

    /// Complexité : O(n log n) - n = tâches conservées, borné à4096.
    /// Le décodage déjà fourni par tasks() ajoute O(B), B octets des payloads.
    /// Projection globale en lecture seule ; aucun corps de mission/résultat/cwd.
    pub fn overview_page(
        &self,
        state_filter: &str,
        limit: u32,
    ) -> Result<serde_json::Value, bridget_transport::protocol::DelegationControlRefusal> {
        use bridget_transport::protocol::DelegationControlRefusal as E;
        if !matches!(state_filter, "active" | "all") || !(1..=100).contains(&limit) {
            return Err(E::InvalidRequest);
        }
        let mut tasks = self.tasks().map_err(|_| E::StoreUnavailable)?;
        let total_active = tasks.iter().filter(|task| task.holds_slot()).count() as u32;
        if state_filter == "active" {
            tasks.retain(Task::holds_slot);
        }
        tasks.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.task_id.cmp(&a.task_id))
        });
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| E::StoreUnavailable)?
            .as_secs()
            .min(i64::MAX as u64) as i64;
        let items = tasks
            .into_iter()
            .take(limit as usize)
            .map(|task| task.overview_row(now))
            .collect();
        // Tous les champs d'arrêt viennent de cette seule lecture, jamais de fragments distincts.
        let halt = self.delegation_halt()?;
        serde_json::to_value(
            bridget_transport::protocol::DaemonToWrapper::DelegationOverview {
                halted: halt.halted,
                since: halt.since,
                by: halt.by,
                reason: halt.reason,
                items,
                total_active,
            },
        )
        .map_err(|_| E::StoreUnavailable)
    }

    /// O(log n+B). Les deux clés uniques et l'empreinte sont contrôlées dans
    /// une transaction immédiate ; un conflit ne modifie jamais la ligne.
    pub fn insert_child_request(&self, request: &ChildRequest) -> Result<ChildRequest, String> {
        let provider_id: bridget_transport::protocol::ChildProviderRequestId =
            serde_json::from_str(&request.provider_request_id).map_err(|e| e.to_string())?;
        let provider_id = serde_json::to_string(&provider_id).map_err(|e| e.to_string())?;
        let tx = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO native_child_requests (request_id, task_id, child_instance, provider_session, turn_id, provider_request_id, content_hash, kind, verdict, state, outcome, summary, facts_json, model_text, offered_json, created_at, deadline_at, decision, answer_json, decided_by, decided_at, delivery_state, void_reason, sent_at, delivered_at, inbox_item_id, inbox_settled) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27) ON CONFLICT DO NOTHING",
            params![request.request_id, request.task_id, request.child_instance, request.provider_session, request.turn_id, provider_id, request.content_hash, request.kind, request.verdict, request.state, request.outcome, request.summary, request.facts_json, request.model_text, request.offered_json, request.created_at, request.deadline_at, request.decision, request.answer_json, request.decided_by, request.decided_at, request.delivery_state, request.void_reason, request.sent_at, request.delivered_at, request.inbox_item_id, request.inbox_settled],
        ).map_err(|e| e.to_string())?;
        let existing = tx.query_row(
            "SELECT * FROM native_child_requests WHERE task_id=?1 AND child_instance=?2 AND provider_session=?3 AND turn_id=?4 AND provider_request_id=?5",
            params![request.task_id, request.child_instance, request.provider_session, request.turn_id, provider_id],
            ChildRequest::from_row,
        ).optional().map_err(|e| e.to_string())?.ok_or("request_conflict")?;
        if existing.content_hash != request.content_hash {
            return Err("request_conflict".into());
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(existing)
    }

    /// O(log n+B), lecture par clé primaire.
    pub fn get_child_request(&self, request_id: &str) -> Result<Option<ChildRequest>, String> {
        self.conn
            .query_row(
                "SELECT * FROM native_child_requests WHERE request_id=?1",
                [request_id],
                ChildRequest::from_row,
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    /// O(log n+k+B), index task/state ; k<=32 selon l'admission155.
    pub(crate) fn child_requests_for_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<ChildRequest>, String> {
        let mut statement=self.conn.prepare("SELECT * FROM native_child_requests WHERE task_id=?1 ORDER BY created_at,request_id LIMIT 32").map_err(|e|e.to_string())?;
        statement
            .query_map([task_id], ChildRequest::from_row)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// O(n+B), parcours de l'index state/deadline existant, au plus512 lignes.
    /// Une remise void ne prouve pas Closed ; les clôtures confirmées sont exclues.
    pub(crate) fn child_request_followups(&self) -> Result<Vec<ChildRequest>, String> {
        let mut statement=self.conn.prepare("SELECT * FROM native_child_requests INDEXED BY native_child_requests_open WHERE state='open' OR delivery_state IN ('pending','sent') OR (delivery_state IN ('none','void') AND coalesce(void_reason,'')!='closed_before_apply') ORDER BY deadline_at,request_id LIMIT 512").map_err(|e|e.to_string())?;
        statement
            .query_map([], ChildRequest::from_row)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// O(u+B), u lignes non soldées de l'index existant, sortie<=512 lignes.
    /// Les décisions humaines sont lues par leur item exact, sans relever la boîte.
    pub(crate) fn child_inbox_followups(&self) -> Result<Vec<ChildRequest>, String> {
        let mut statement=self.conn.prepare("SELECT * FROM native_child_requests INDEXED BY native_child_requests_inbox WHERE inbox_settled=0 AND inbox_item_id IS NOT NULL ORDER BY state LIMIT 512").map_err(|e|e.to_string())?;
        statement
            .query_map([], ChildRequest::from_row)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// O(log n), CAS : un lien déjà publié ne peut pas être remplacé.
    pub(crate) fn link_child_inbox(&self, request_id: &str, item_id: &str) -> Result<bool, String> {
        self.conn.execute("UPDATE native_child_requests SET inbox_item_id=?2 WHERE request_id=?1 AND inbox_item_id IS NULL",params![request_id,item_id])
            .map(|count|count==1).map_err(|e|e.to_string())
    }

    /// O(log n), après règlement exact de l'item ; ne touche aucune décision.
    pub(crate) fn mark_child_inbox_settled(&self, request_id: &str) -> Result<bool, String> {
        self.conn.execute("UPDATE native_child_requests SET inbox_settled=1 WHERE request_id=?1 AND inbox_item_id IS NOT NULL AND inbox_settled=0",[request_id]).map(|count|count==1).map_err(|e|e.to_string())
    }

    /// O(log n), trace de fermeture demandée ; aucune décision historique changée.
    pub(crate) fn mark_child_closing(&self, request_id: &str, reason: &str) -> Result<(), String> {
        self.conn.execute("UPDATE native_child_requests SET void_reason=coalesce(void_reason,?2) WHERE request_id=?1 AND delivery_state NOT IN ('delivered','unknown') AND coalesce(void_reason,'')!='closed_before_apply'",params![request_id,reason])
            .map(|_|()).map_err(|e|e.to_string())
    }

    /// O(log n+B). Preuve : Closed reçu, OU arrêt OS149/152 attesté AVANT
    /// premier envoi (sent_at=None), via settle_after_stop ; aucune trame inventée.
    /// La décision et l'issue terminales sont conservées.
    pub(crate) fn confirm_child_closed(
        &self,
        request_id: &str,
        applied: bool,
        now: i64,
    ) -> Result<bool, String> {
        self.conn.execute("UPDATE native_child_requests SET delivery_state=CASE WHEN ?2 THEN 'delivered' ELSE 'void' END,void_reason=CASE WHEN ?2 THEN void_reason ELSE 'closed_before_apply' END,delivered_at=CASE WHEN ?2 THEN ?3 ELSE delivered_at END,state=CASE WHEN state='open' THEN 'cancelled' ELSE state END,outcome=CASE WHEN state='open' THEN 'cancelled' ELSE outcome END,decided_by=CASE WHEN state='open' THEN 'system' ELSE decided_by END,decided_at=CASE WHEN state='open' THEN ?3 ELSE decided_at END WHERE request_id=?1 AND delivery_state NOT IN ('delivered','unknown') AND coalesce(void_reason,'')!='closed_before_apply'",params![request_id,applied,now])
            .map(|count|count==1).map_err(|e|e.to_string())
    }

    /// O(log n+k), k<=32 ; provider ET item humain soldés avant cleanup_done.
    pub(crate) fn child_closure_pending(&self, task_id: &str) -> Result<bool, String> {
        self.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_child_requests WHERE task_id=?1 AND ((delivery_state NOT IN ('delivered','unknown') AND coalesce(void_reason,'')!='closed_before_apply') OR (inbox_item_id IS NOT NULL AND inbox_settled=0)))",[task_id],|row|row.get(0)).map_err(|e|e.to_string())
    }

    /// O(log n+k log k+B), k lignes ouvertes de la tâche (borne métier : 4).
    pub fn open_child_requests(&self, task_id: &str) -> Result<Vec<ChildRequest>, String> {
        let mut stmt = self.conn.prepare("SELECT * FROM native_child_requests WHERE task_id=?1 AND state='open' ORDER BY created_at,request_id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([task_id], ChildRequest::from_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// O(log n+k+B), index state/deadline ; au plus 512 lignes par lecture.
    pub fn due_child_requests(&self, now: i64) -> Result<Vec<ChildRequest>, String> {
        let mut stmt = self.conn.prepare("SELECT * FROM native_child_requests WHERE state='open' AND deadline_at<=?1 ORDER BY deadline_at LIMIT 512")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([now], ChildRequest::from_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// O(log n+k), index task/state. Retour : (ouvertes, total).
    pub fn child_request_counts(&self, task_id: &str) -> Result<(usize, usize), String> {
        self.conn.query_row("SELECT coalesce(sum(state='open'),0),count(*) FROM native_child_requests WHERE task_id=?1", [task_id],
            |row| Ok((row.get(0)?, row.get(1)?))).map_err(|e| e.to_string())
    }

    /// O(log n+B). CAS de décision distinct du claim de remise ; false ne
    /// donne aucun droit d'envoi. Les validations métier appartiennent à l'appelant.
    pub fn decide_child_request(
        &self,
        request_id: &str,
        decision: &str,
        answer_json: Option<&str>,
        decided_by: &str,
        outcome: &str,
        now: i64,
    ) -> Result<bool, String> {
        if !matches!(decision, "accept" | "decline" | "cancel") {
            return Err("decision_not_offered".into());
        }
        self.conn.execute("UPDATE native_child_requests SET state='answered',outcome=?2,decision=?3,answer_json=?4,decided_by=?5,decided_at=?6,delivery_state='pending' WHERE request_id=?1 AND state='open' AND deadline_at>?6 AND delivery_state='none'",
            params![request_id,outcome,decision,answer_json,decided_by,now]).map(|count| count == 1).map_err(|e| e.to_string())
    }

    /// O(log n+B), clôture unique : aucun état terminal ne peut être réécrit.
    pub fn close_child_request(
        &self,
        request_id: &str,
        state: &str,
        outcome: &str,
        now: i64,
    ) -> Result<bool, String> {
        if !matches!(state, "expired" | "cancelled" | "refused") {
            return Err("request_closed".into());
        }
        self.conn.execute("UPDATE native_child_requests SET state=?2,outcome=?3,decided_by='system',decided_at=?4 WHERE request_id=?1 AND state='open'",
            params![request_id,state,outcome,now]).map(|count| count == 1).map_err(|e| e.to_string())
    }

    /// O(log n). À appeler APRÈS revalidation FR027 sous le verrou métier.
    /// sent_at est committé avant toute trame ; false interdit l'envoi.
    pub fn claim_child_request(&self, request_id: &str, now: i64) -> Result<bool, String> {
        self.conn.execute("UPDATE native_child_requests SET delivery_state='sent',sent_at=?2 WHERE request_id=?1 AND state='answered' AND delivery_state='pending' AND deadline_at>?2",
            params![request_id,now]).map(|count| count == 1).map_err(|e| e.to_string())
    }

    /// O(log n+B). Une invalidation ne réactive jamais un accord déjà envoyé.
    pub fn void_pending_child_request(
        &self,
        request_id: &str,
        reason: &str,
    ) -> Result<bool, String> {
        self.conn.execute("UPDATE native_child_requests SET delivery_state='void',void_reason=?2 WHERE request_id=?1 AND delivery_state='pending'",
            params![request_id,reason]).map(|count| count == 1).map_err(|e| e.to_string())
    }

    /// O(log n+B). Seule une remise sent peut être réglée ; les décisions
    /// historiques restent intactes, y compris si leur application est inconnue.
    pub fn settle_sent_child_request(
        &self,
        request_id: &str,
        delivery: &str,
        reason: Option<&str>,
        now: i64,
    ) -> Result<bool, String> {
        if !matches!(delivery, "delivered" | "void" | "unknown") {
            return Err("request_closed".into());
        }
        self.conn.execute("UPDATE native_child_requests SET delivery_state=?2,void_reason=?3,delivered_at=CASE WHEN ?2='delivered' THEN ?4 ELSE delivered_at END,outcome=CASE WHEN ?2='unknown' THEN 'cancelled_outcome_unknown' ELSE outcome END WHERE request_id=?1 AND delivery_state='sent'",
            params![request_id,delivery,reason,now]).map(|count| count == 1).map_err(|e| e.to_string())
    }

    pub fn wait_epoch(&self) -> WaitEpoch {
        self.wait_epoch.clone()
    }

    #[cfg(test)]
    pub fn test_connection(&self) -> &Connection {
        &self.conn
    }

    #[cfg(test)]
    pub fn by_request(
        &self,
        owner: &str,
        request_id: &str,
        canonical: &[u8],
    ) -> Result<Option<Task>, String> {
        let row: Option<(Vec<u8>, String)> = self.conn.query_row(
            "SELECT canonical,payload FROM native_delegations WHERE owner_instance=?1 AND request_id=?2",
            params![owner, request_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional().map_err(|error| error.to_string())?;
        match row {
            Some((saved, payload)) if saved == canonical => serde_json::from_str(&payload)
                .map(Some)
                .map_err(|error| error.to_string()),
            Some(_) => Err("envelope_mismatch".into()),
            None => Ok(None),
        }
    }

    pub fn by_agent_request(
        &self,
        owner: &str,
        request_id: &str,
        canonical: &[u8],
    ) -> Result<Option<Task>, String> {
        let row: Option<(Vec<u8>,String)>=self.conn.query_row("SELECT canonical,payload FROM native_delegations WHERE json_extract(payload,'$.owner')=?1 AND request_id=?2",params![owner,request_id],|row|Ok((row.get(0)?,row.get(1)?))).optional().map_err(|error|error.to_string())?;
        match row {
            Some((saved, payload)) if saved == canonical => serde_json::from_str(&payload)
                .map(Some)
                .map_err(|error| error.to_string()),
            Some(_) => Err("envelope_mismatch".into()),
            None if self.tombstoned(owner, request_id)? => Err("request_expired".into()),
            None => {
                let receipt: Option<(Vec<u8>, String)> = self
                    .conn
                    .query_row(
                        "SELECT canonical,code FROM native_delegation_refusals WHERE owner=?1 AND request_id=?2",
                        params![owner, request_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                    .map_err(|error| error.to_string())?;
                match receipt {
                    Some((saved, code)) if saved == canonical => Err(code),
                    Some(_) => Err("envelope_mismatch".into()),
                    None => Ok(None),
                }
            }
        }
    }

    pub(crate) fn auto_admission_receipt(
        &self,
        owner: &str,
        request_id: &str,
    ) -> Result<Option<AutoAdmissionReceipt>, String> {
        let receipt = self
            .conn
            .query_row(
                "SELECT owner,request_id,canonical,metadata,state,coordinator_epoch,revision,policy_revision,policy_digest,candidate_id,delegate_canonical,mode,reason,task_id,refusal_code,created_at,updated_at FROM auto_admission_receipts WHERE owner=?1 AND request_id=?2",
                params![owner, request_id],
                AutoAdmissionReceipt::from_row,
            )
            .optional()
            .map_err(|error| error.to_string())?;
        if receipt.is_none() && self.tombstoned(owner, request_id)? {
            return Err("request_expired".into());
        }
        Ok(receipt)
    }

    /// Lot borné pour l'entretien post-réconciliation. Les transitions CAS
    /// existantes restent les seules autorisées à modifier chaque reçu.
    pub(crate) fn auto_admission_receipts_before_epoch(
        &self,
        current_epoch: i64,
        limit: usize,
    ) -> Result<Vec<AutoAdmissionReceipt>, String> {
        if current_epoch < 0 || !(1..=128).contains(&limit) {
            return Err("auto_admission_invalid".into());
        }
        self.conn
            .prepare(
                "SELECT owner,request_id,canonical,metadata,state,coordinator_epoch,revision,policy_revision,policy_digest,candidate_id,delegate_canonical,mode,reason,task_id,refusal_code,created_at,updated_at
                 FROM auto_admission_receipts
                 WHERE coordinator_epoch<?1
                   AND state IN ('pending','decision_started','chosen')
                 ORDER BY coordinator_epoch,updated_at,owner,request_id
                 LIMIT ?2",
            )
            .map_err(|error| error.to_string())?
            .query_map(params![current_epoch, limit as i64], AutoAdmissionReceipt::from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())
    }

    /// Alloue l'époque du coordinateur au démarrage. La lecture prend le
    /// verrou d'écriture SQLite pour ne jamais régresser devant un reçu auto
    /// concurrent ; l'époque zéro reste réservée aux données antérieures.
    pub(crate) fn next_auto_coordinator_epoch(&self) -> Result<i64, String> {
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let latest: Option<i64> = tx
            .query_row(
                "SELECT max(coordinator_epoch) FROM auto_admission_receipts",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        let epoch = latest
            .unwrap_or(0)
            .checked_add(1)
            .filter(|epoch| *epoch > 0)
            .ok_or("auto_admission_invalid")?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(epoch)
    }

    /// Sous pression, libère juste les clés sûres nécessaires à repasser sous
    /// `bound`. Les refus vivent trente jours hors pression ; aucun autre état
    /// auto n'est une cible de collecte.
    pub(crate) fn purge_auto_refusals_under_pressure(
        &self,
        now: i64,
        bound: usize,
    ) -> Result<usize, String> {
        if bound == 0 {
            return Err("auto_admission_invalid".into());
        }
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let retained = retained_admission_keys_in(&tx)?;
        let purged = if retained < bound {
            0
        } else {
            purge_auto_refusals_in(&tx, now, retained - bound + 1, None)?
        };
        tx.commit().map_err(|error| error.to_string())?;
        Ok(purged)
    }

    /// Entretien borné hors pression : le refus exact reste lisible trente
    /// jours, puis la même transition atomique le remplace par un tombstone.
    pub(crate) fn purge_expired_auto_admission_refusals(
        &self,
        now: i64,
        limit: usize,
    ) -> Result<usize, String> {
        if !(1..=128).contains(&limit) {
            return Err("auto_admission_invalid".into());
        }
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let purged =
            purge_auto_refusals_in(&tx, now, limit, Some(now.saturating_sub(30 * 24 * 3600)))?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(purged)
    }

    /// Persiste l'unique choix autorisé avant le puits native. Le CAS par
    /// révision interdit qu'une réponse tardive remplace un état repris.
    pub(crate) fn choose_auto_admission(
        &self,
        owner: &str,
        request_id: &str,
        canonical: &[u8],
        candidate_id: String,
        delegate_canonical: Vec<u8>,
        mode: AutoAdmissionMode,
        reason: Option<String>,
        now: i64,
    ) -> Result<AutoAdmissionReceipt, String> {
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let mut receipt =
            auto_admission_receipt_in(&tx, owner, request_id)?.ok_or("auto_admission_missing")?;
        if receipt.canonical != canonical {
            return Err("envelope_mismatch".into());
        }
        if !matches!(
            receipt.state,
            AutoAdmissionState::Pending | AutoAdmissionState::DecisionStarted
        ) {
            return Err("auto_admission_already_resolved".into());
        }
        receipt.state = AutoAdmissionState::Chosen;
        receipt.candidate_id = Some(candidate_id);
        receipt.delegate_canonical = Some(delegate_canonical);
        receipt.mode = Some(mode);
        receipt.reason = reason;
        receipt.refusal_code = None;
        receipt.revision = receipt
            .revision
            .checked_add(1)
            .ok_or("auto_admission_invalid")?;
        receipt.updated_at = now.max(receipt.created_at);
        receipt.validate()?;
        let changed = tx
            .execute(
                "UPDATE auto_admission_receipts SET state=?3,revision=?4,candidate_id=?5,delegate_canonical=?6,mode=?7,reason=?8,refusal_code=NULL,updated_at=?9 WHERE owner=?1 AND request_id=?2 AND revision=?10",
                params![
                    owner,
                    request_id,
                    receipt.state.as_sql(),
                    receipt.revision,
                    receipt.candidate_id,
                    receipt.delegate_canonical,
                    receipt.mode.map(AutoAdmissionMode::as_sql),
                    receipt.reason,
                    receipt.updated_at,
                    receipt.revision - 1,
                ],
            )
            .map_err(|error| error.to_string())?;
        if changed != 1 {
            return Err("admission_pending".into());
        }
        tx.commit().map_err(|error| error.to_string())?;
        Ok(receipt)
    }

    /// Marque la frontière juste avant l'appel HTTP. Une reprise ne doit pas
    /// réouvrir ce reçu ni réémettre une décision après un crash demandeur.
    pub(crate) fn start_auto_decision(
        &self,
        receipt: &AutoAdmissionReceipt,
        now: i64,
    ) -> Result<AutoAdmissionReceipt, String> {
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let mut current = auto_admission_receipt_in(&tx, &receipt.owner, &receipt.request_id)?
            .ok_or("auto_admission_missing")?;
        if current.canonical != receipt.canonical {
            return Err("envelope_mismatch".into());
        }
        if current.state != AutoAdmissionState::Pending || current.revision != receipt.revision {
            return Err(if matches!(
                current.state,
                AutoAdmissionState::Pending | AutoAdmissionState::DecisionStarted
            ) {
                "admission_pending"
            } else {
                "auto_admission_already_resolved"
            }
            .into());
        }
        current.state = AutoAdmissionState::DecisionStarted;
        current.revision = current
            .revision
            .checked_add(1)
            .ok_or("auto_admission_invalid")?;
        current.updated_at = now.max(current.created_at);
        current.validate()?;
        let changed = tx
            .execute(
                "UPDATE auto_admission_receipts SET state='decision_started',revision=?3,updated_at=?4 WHERE owner=?1 AND request_id=?2 AND revision=?5",
                params![
                    current.owner,
                    current.request_id,
                    current.revision,
                    current.updated_at,
                    current.revision - 1,
                ],
            )
            .map_err(|error| error.to_string())?;
        if changed != 1 {
            return Err("admission_pending".into());
        }
        tx.commit().map_err(|error| error.to_string())?;
        Ok(current)
    }

    /// Le refus de puits après `chosen` reste terminal dans le reçu auto ; il
    /// n'écrit jamais dans le catalogue fermé des refus natifs.
    pub(crate) fn refuse_auto_admission(
        &self,
        receipt: &AutoAdmissionReceipt,
        code: &str,
        now: i64,
    ) -> Result<AutoAdmissionReceipt, String> {
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let mut current = auto_admission_receipt_in(&tx, &receipt.owner, &receipt.request_id)?
            .ok_or("auto_admission_missing")?;
        if current.canonical != receipt.canonical {
            return Err("envelope_mismatch".into());
        }
        if current.state == AutoAdmissionState::Refused {
            return Ok(current);
        }
        if !matches!(
            current.state,
            AutoAdmissionState::Pending
                | AutoAdmissionState::DecisionStarted
                | AutoAdmissionState::Chosen
        ) || current.revision != receipt.revision
        {
            return Err("admission_pending".into());
        }
        current.state = AutoAdmissionState::Refused;
        current.refusal_code = Some(code.into());
        current.revision = current
            .revision
            .checked_add(1)
            .ok_or("auto_admission_invalid")?;
        current.updated_at = now.max(current.created_at);
        current.validate()?;
        let changed = tx
            .execute(
                "UPDATE auto_admission_receipts SET state='refused',revision=?3,refusal_code=?4,updated_at=?5 WHERE owner=?1 AND request_id=?2 AND revision=?6",
                params![
                    current.owner,
                    current.request_id,
                    current.revision,
                    current.refusal_code,
                    current.updated_at,
                    current.revision - 1,
                ],
            )
            .map_err(|error| error.to_string())?;
        if changed != 1 {
            return Err("admission_pending".into());
        }
        tx.commit().map_err(|error| error.to_string())?;
        Ok(current)
    }

    /// La Task et son lien au reçu passent dans une seule transaction Immediate.
    /// La projection ne devient visible qu'après le commit commun.
    pub(crate) fn admit_auto_task(
        &self,
        receipt: &AutoAdmissionReceipt,
        task: &Task,
        request_id: &str,
        delegate_canonical: &[u8],
        now: i64,
    ) -> Result<(), String> {
        if task.owner != receipt.owner {
            return Err("native_owner_identity_mismatch".into());
        }
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let mut current = auto_admission_receipt_in(&tx, &receipt.owner, &receipt.request_id)?
            .ok_or("auto_admission_missing")?;
        if current.canonical != receipt.canonical {
            return Err("envelope_mismatch".into());
        }
        if current.state != AutoAdmissionState::Chosen
            || current.revision != receipt.revision
            || current.delegate_canonical.as_deref() != Some(delegate_canonical)
        {
            return Err("admission_pending".into());
        }
        let (stored, mutation) =
            self.insert_projected_in(&tx, task, request_id, delegate_canonical)?;
        current.state = AutoAdmissionState::Admitted;
        current.task_id = Some(stored.task_id.clone());
        current.revision = current
            .revision
            .checked_add(1)
            .ok_or("auto_admission_invalid")?;
        current.updated_at = now.max(current.created_at);
        current.validate()?;
        let changed = tx
            .execute(
                "UPDATE auto_admission_receipts SET state='admitted',revision=?3,task_id=?4,updated_at=?5 WHERE owner=?1 AND request_id=?2 AND revision=?6",
                params![
                    current.owner,
                    current.request_id,
                    current.revision,
                    current.task_id,
                    current.updated_at,
                    current.revision - 1,
                ],
            )
            .map_err(|error| error.to_string())?;
        if changed != 1 {
            return Err("admission_pending".into());
        }
        tx.commit().map_err(|error| error.to_string())?;
        self.publish_projection(stored.root_owner(), mutation);
        Ok(())
    }

    /// Écriture initiale additive, avant toute décision ou création de Task.
    /// La transaction immédiate rend le canon immuable pour les rejouements.
    pub(crate) fn insert_auto_admission_receipt(
        &self,
        receipt: &AutoAdmissionReceipt,
    ) -> Result<bool, String> {
        receipt.validate()?;
        let tx = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )
        .map_err(|error| error.to_string())?;
        let inserted = Self::insert_auto_admission_receipt_in(&tx, receipt)?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(inserted)
    }

    /// Réserve une clé auto nouvelle sous le même verrou SQLite que ses trois
    /// plafonds. Un rejeu identique est relu avant toute garde : il ne peut ni
    /// déclencher une purge ni se voir refuser par une limite devenue pleine.
    pub(crate) fn reserve_auto_admission_receipt(
        &self,
        receipt: &AutoAdmissionReceipt,
        now: i64,
    ) -> Result<bool, String> {
        receipt.validate()?;
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        if let Some(existing) = auto_admission_receipt_in(&tx, &receipt.owner, &receipt.request_id)?
        {
            if existing.canonical == receipt.canonical {
                tx.commit().map_err(|error| error.to_string())?;
                return Ok(false);
            }
            return Err("envelope_mismatch".into());
        }
        if tombstoned_in(&tx, &receipt.owner, &receipt.request_id)? {
            return Err("request_expired".into());
        }
        if native_admission_key_reserved_in(&tx, &receipt.owner, &receipt.request_id)? {
            return Err("auto_admission_reserved".into());
        }
        let (active, owner_active, mut retained) =
            admission_counts_for_owner_in(&tx, &receipt.owner)?;
        if active >= MAX_ACTIVE_ADMISSIONS || owner_active >= MAX_ACTIVE_ADMISSIONS_PER_OWNER {
            return Err("task_limit".into());
        }
        if retained >= MAX_RETAINED_TASKS {
            purge_auto_refusals_in(&tx, now, retained - MAX_RETAINED_TASKS + 1, None)?;
            retained = retained_admission_keys_in(&tx)?;
            if retained >= MAX_RETAINED_TASKS {
                return Err("retention_saturated".into());
            }
        }
        let inserted = Self::insert_auto_admission_receipt_in(&tx, receipt)?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(inserted)
    }

    pub(super) fn insert_auto_admission_receipt_in(
        tx: &rusqlite::Transaction<'_>,
        receipt: &AutoAdmissionReceipt,
    ) -> Result<bool, String> {
        receipt.validate()?;
        if tombstoned_in(tx, &receipt.owner, &receipt.request_id)? {
            return Err("request_expired".into());
        }
        if native_admission_key_reserved_in(tx, &receipt.owner, &receipt.request_id)? {
            return Err("auto_admission_reserved".into());
        }
        let changed = tx
            .execute(
                "INSERT OR IGNORE INTO auto_admission_receipts(owner,request_id,canonical,metadata,state,coordinator_epoch,revision,policy_revision,policy_digest,candidate_id,delegate_canonical,mode,reason,task_id,refusal_code,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
                params![
                    receipt.owner,
                    receipt.request_id,
                    receipt.canonical,
                    receipt.metadata,
                    receipt.state.as_sql(),
                    receipt.coordinator_epoch,
                    receipt.revision,
                    receipt.policy_revision,
                    receipt.policy_digest,
                    receipt.candidate_id,
                    receipt.delegate_canonical,
                    receipt.mode.map(AutoAdmissionMode::as_sql),
                    receipt.reason,
                    receipt.task_id,
                    receipt.refusal_code,
                    receipt.created_at,
                    receipt.updated_at,
                ],
            )
            .map_err(|error| error.to_string())?;
        if changed == 1 {
            return Ok(true);
        }
        let existing: Vec<u8> = tx
            .query_row(
                "SELECT canonical FROM auto_admission_receipts WHERE owner=?1 AND request_id=?2",
                params![receipt.owner, receipt.request_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if existing == receipt.canonical {
            Ok(false)
        } else {
            Err("envelope_mismatch".into())
        }
    }

    /// O(log T + log R), T tâches et R refus : deux clés indexées.
    /// Le premier reçu est immuable ; une tâche admise prime sur le refus.
    /// Aucun texte dynamique, refus transitoire ou signal de projection.
    pub fn record_refusal(
        &self,
        owner: &str,
        request_id: &str,
        canonical: &[u8],
        code: &str,
        now: i64,
    ) -> Result<bool, String> {
        if !matches!(
            code,
            "cwd_outside_parent_project"
                | "cwd_unavailable"
                | "model_unavailable"
                | "effort_unavailable"
                | "agent_type_unknown"
        ) {
            return Ok(false);
        }
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let inserted = tx
            .execute(
                "INSERT OR IGNORE INTO native_delegation_refusals(owner,request_id,canonical,code,refused_at)
                 SELECT ?1,?2,?3,?4,?5 WHERE NOT EXISTS (
                     SELECT 1 FROM native_delegations WHERE json_extract(payload,'$.owner')=?1 AND request_id=?2
                 ) AND NOT EXISTS (
                     SELECT 1 FROM auto_admission_receipts WHERE owner=?1 AND request_id=?2
                 )",
                params![owner, request_id, canonical, code, now],
            )
            .map_err(|error| error.to_string())?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(inserted != 0)
    }

    /// O(R) balayage des R reçus ; appelé par l'entretien déjà existant.
    /// Supprime seulement les refus âgés de plus de 30 jours, sans signal.
    pub fn purge_expired_refusals(&self, now: i64) -> Result<usize, String> {
        self.conn
            .execute(
                "DELETE FROM native_delegation_refusals WHERE refused_at < ?1",
                [now.saturating_sub(30 * 24 * 3600)],
            )
            .map_err(|error| error.to_string())
    }

    pub fn insert(&self, task: &Task, request_id: &str, canonical: &[u8]) -> Result<(), String> {
        self.insert_projected(task, request_id, canonical)
    }

    pub fn save(&self, task: &Task) -> Result<(), String> {
        self.save_projected(task)
    }

    /// O(k) lectures et écritures indexées, k <= 16. Le daemon a validé le lot.
    /// Un seul commit ; aucun stamp, projection ou fait de livraison modifié.
    pub fn acknowledge_tasks(
        &self,
        task_ids: &[String],
        now: i64,
    ) -> Result<(Vec<String>, Vec<String>), String> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        let mut acknowledged = Vec::with_capacity(task_ids.len());
        let mut already = Vec::with_capacity(task_ids.len());
        for id in task_ids {
            let saved: Option<Option<i64>> = tx
                .query_row(
                    "SELECT json_extract(payload,'$.acknowledged_at') FROM native_delegations WHERE task_id=?1",
                    [id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| error.to_string())?;
            let saved = saved.ok_or("task_unavailable")?;
            if saved.is_some() {
                already.push(id.clone());
            } else {
                let changed = tx
                    .execute(
                        "UPDATE native_delegations SET payload=json_set(payload,'$.acknowledged_at',?2) WHERE task_id=?1 AND json_extract(payload,'$.acknowledged_at') IS NULL",
                        params![id, now],
                    )
                    .map_err(|error| error.to_string())?;
                if changed != 1 {
                    return Err("task_unavailable".into());
                }
                acknowledged.push(id.clone());
            }
        }
        tx.commit().map_err(|error| error.to_string())?;
        if !acknowledged.is_empty() {
            self.wait_epoch.bump();
        }
        Ok((acknowledged, already))
    }

    pub fn get(&self, task_id: &str) -> Result<Option<Task>, String> {
        let payload: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM native_delegations WHERE task_id=?1",
                [task_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        payload
            .map(|payload| serde_json::from_str(&payload).map_err(|error| error.to_string()))
            .transpose()
    }

    pub fn tasks(&self) -> Result<Vec<Task>, String> {
        let mut statement = self
            .conn
            .prepare("SELECT payload FROM native_delegations ORDER BY task_id")
            .map_err(|error| error.to_string())?;
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?
            .map(|row| {
                row.map_err(|error| error.to_string()).and_then(|payload| {
                    serde_json::from_str(&payload).map_err(|error| error.to_string())
                })
            })
            .collect()
    }

    /// Comptages compatibles : (actives UNION, actives de l'instance native,
    /// clés retenues UNION). Le second champ ne peut pas compter une réservation
    /// auto par propriétaire durable ; les nouveaux puits utilisent la méthode
    /// dédiée ci-dessous.
    pub fn admission_counts(&self, instance: &str) -> Result<(usize, usize, usize), String> {
        let count = |sql: String, args: &[&dyn rusqlite::ToSql]| -> Result<usize, String> {
            self.conn
                .query_row(&sql, args, |row| row.get(0))
                .map_err(|error| error.to_string())
        };
        let active_keys = active_admission_keys_sql();
        Ok((
            count(format!("SELECT count(*) FROM ({active_keys})"), &[])?,
            count(
                format!(
                    "SELECT count(*) FROM native_delegations INDEXED BY native_delegations_state WHERE owner_instance=?1 AND ({NATIVE_HOLDS_SLOT_SQL})"
                ),
                &[&instance],
            )?,
            count(
                "SELECT count(*) FROM (
                    SELECT json_extract(payload,'$.owner') AS owner,request_id FROM native_delegations
                    UNION
                    SELECT owner,request_id FROM auto_admission_receipts
                )"
                .into(),
                &[],
            )?,
        ))
    }

    /// Comptages du puits 163 : le plafond de 16 porte sur l'owner durable
    /// attesté, donc une réservation auto et sa Task liée ne consomment qu'une
    /// place commune. Complexité O(n), n borné par la rétention UNION.
    pub(crate) fn admission_counts_for_owner(
        &self,
        owner: &str,
    ) -> Result<(usize, usize, usize), String> {
        let count = |sql: String, args: &[&dyn rusqlite::ToSql]| -> Result<usize, String> {
            self.conn
                .query_row(&sql, args, |row| row.get(0))
                .map_err(|error| error.to_string())
        };
        let active_keys = active_admission_keys_sql();
        Ok((
            count(format!("SELECT count(*) FROM ({active_keys})"), &[])?,
            count(
                format!("SELECT count(*) FROM ({active_keys}) WHERE owner=?1"),
                &[&owner],
            )?,
            count(
                "SELECT count(*) FROM (
                    SELECT json_extract(payload,'$.owner') AS owner,request_id FROM native_delegations
                    UNION
                    SELECT owner,request_id FROM auto_admission_receipts
                )"
                .into(),
                &[],
            )?,
        ))
    }

    /// O(log n + a), a ≤ 128 tâches actives ; seul le compteur quitte SQLite.
    /// Appelé avant l'insertion de la candidate, donc compte les autres tâches.
    pub fn shared_writers(&self, cwd: &str) -> Result<usize, String> {
        self.conn.query_row(
            "SELECT count(*) FROM native_delegations INDEXED BY native_delegations_state WHERE json_extract(payload,'$.state') IN ('queued','starting','mission_pending','working','waiting_for_children','cancelling') AND json_extract(payload,'$.effective_posture')='development' AND json_extract(payload,'$.cwd')=?1",
            [cwd],
            |row| row.get(0),
        ).map_err(|error| error.to_string())
    }

    pub fn pending(&self) -> Result<Vec<Task>, String> {
        let mut statement=self.conn.prepare("SELECT payload FROM native_delegations WHERE json_extract(payload,'$.state') NOT IN ('failed','cancelled','result_available') OR (json_extract(payload,'$.state')='result_available' AND (json_extract(payload,'$.result_sent')=0 OR coalesce(json_extract(payload,'$.cleanup_done'),0)=0)) OR (json_extract(payload,'$.state')='failed' AND coalesce(json_extract(payload,'$.cleanup_done'),0)=0) OR (json_extract(payload,'$.state')='cancelled' AND json_extract(payload,'$.workspace') IS NOT NULL AND coalesce(json_extract(payload,'$.cleanup_done'),0)=0) ORDER BY task_id").map_err(|error|error.to_string())?;
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?
            .map(|row| {
                row.map_err(|error| error.to_string()).and_then(|payload| {
                    serde_json::from_str(&payload).map_err(|error| error.to_string())
                })
            })
            .collect()
    }

    pub fn failed_for_owner(&self, owner: &str) -> Result<Vec<Task>, String> {
        let mut statement=self.conn.prepare("SELECT payload FROM native_delegations WHERE json_extract(payload,'$.owner')=?1 AND json_extract(payload,'$.state')='failed' AND coalesce(json_extract(payload,'$.failure_sent'),0)=0 AND coalesce(json_extract(payload,'$.cleanup_done'),0)=1 ORDER BY task_id LIMIT 16").map_err(|error|error.to_string())?;
        statement
            .query_map([owner], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?
            .map(|row| {
                row.map_err(|error| error.to_string()).and_then(|payload| {
                    serde_json::from_str(&payload).map_err(|error| error.to_string())
                })
            })
            .collect()
    }

    pub fn for_mission(&self, mission: &str) -> Result<Option<Task>, String> {
        let payload: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM native_delegations WHERE json_extract(payload,'$.mission')=?1",
                [mission],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        payload
            .map(|payload| serde_json::from_str(&payload).map_err(|error| error.to_string()))
            .transpose()
    }

    /// O(log h + log n + B), h missions passées, n tâches et B octets du payload.
    pub fn past_mission_task(&self, mission: &str) -> Result<Option<Task>, String> {
        let payload: Option<String> = self.conn.query_row(
            "SELECT task.payload FROM native_delegation_past_missions past JOIN native_delegations task ON task.task_id=past.task_id WHERE past.mission=?1",
            [mission],
            |row| row.get(0),
        ).optional().map_err(|error| error.to_string())?;
        payload
            .map(|payload| serde_json::from_str(&payload).map_err(|error| error.to_string()))
            .transpose()
    }

    pub fn grant(
        &self,
        owner: &str,
        cwd: &Path,
        posture: SpawnPosture,
        revoke: bool,
    ) -> Result<(), String> {
        if revoke {
            self.conn.execute("INSERT INTO native_delegation_grants(owner_instance,cwd,posture) VALUES(?1,'','revoked') ON CONFLICT(owner_instance) DO UPDATE SET cwd='',posture='revoked'",[owner]).map_err(|error| error.to_string())?;
        } else {
            self.conn.execute("INSERT INTO native_delegation_grants(owner_instance,cwd,posture) VALUES(?1,?2,?3) ON CONFLICT(owner_instance) DO UPDATE SET cwd=excluded.cwd,posture=excluded.posture",
                params![owner,cwd.to_string_lossy(),match posture { SpawnPosture::Discovery => "discovery", SpawnPosture::Development => "development" }]).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn permission(&self, owner: &str) -> Result<Option<(String, SpawnPosture)>, String> {
        self.conn.query_row("SELECT cwd,posture FROM native_delegation_grants WHERE owner_instance=?1 AND posture!='revoked'",[owner],|row| {
            let posture: String = row.get(1)?;
            Ok((row.get(0)?,if posture == "development" {SpawnPosture::Development} else {SpawnPosture::Discovery}))
        }).optional().map_err(|error| error.to_string())
    }

    pub fn grant_agent(
        &self,
        agent: &str,
        instance: &str,
        cwd: &Path,
        posture: SpawnPosture,
        revoke: bool,
    ) -> Result<(), String> {
        if revoke {
            // Fermer d'abord le droit stable. Une panne ultérieure reste fermée.
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO native_delegation_revocations(agent_id) VALUES(?1)",
                    [agent],
                )
                .map_err(|error| error.to_string())?;
            self.grant(instance, cwd, posture, true)
        } else {
            self.grant(instance, cwd, posture, false)?;
            self.conn
                .execute(
                    "DELETE FROM native_delegation_revocations WHERE agent_id=?1",
                    [agent],
                )
                .map_err(|error| error.to_string())?;
            Ok(())
        }
    }

    pub fn revoked_agent(&self, agent: &str) -> Result<bool, String> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM native_delegation_revocations WHERE agent_id=?1)",
                [agent],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())
    }

    pub fn revoked(&self, owner: &str) -> Result<bool, String> {
        self.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_delegation_grants WHERE owner_instance=?1 AND posture='revoked')",[owner],|row|row.get(0)).map_err(|error|error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote162_receipt_fixture() -> (
        DelegationStore,
        Task,
        bridget_transport::protocol::RemoteResultEnvelope,
    ) {
        use bridget_transport::protocol::*;
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let mut task = legacy_task151();
        let result = RemoteResultEnvelope {
            worker: WorkerIdentity {
                machine_id: "cartae".into(),
                worker_id: "worker".into(),
                provisioning_revision: 1,
            },
            identity: WorkerResultIdentity {
                command: WorkerCommandIdentity {
                    command_id: "command".into(),
                    task_id: task.task_id.clone(),
                    instance_id: "instance".into(),
                    generation: 1,
                },
                mission_id: task.mission.clone(),
                round: 1,
                execution_id: "execution".into(),
                execution_generation: 1,
            },
            message_id: "result".into(),
            body: "résultat durable".into(),
            digest: String::new(),
        };
        let mut result = result;
        result.digest = result.canonical_digest().unwrap();
        task.remote_result_slots.push(WorkerResultSlot {
            identity: result.identity.clone(),
            admission_digest: "a".repeat(64),
        });
        store.insert(&task, "request", b"canonical").unwrap();
        let task = store.get(&task.task_id).unwrap().unwrap();
        (store, task, result)
    }

    #[test]
    fn remote162_result_commit_atomique_et_echec_avant_commit() {
        use bridget_transport::protocol::RemoteResultDisposition as D;
        let (store, task, result) = remote162_receipt_fixture();
        let mut next = task.clone();
        next.result = Some(result.body.clone());
        next.state = "waiting_for_children".into();
        store.conn.execute_batch("CREATE TRIGGER refuse_remote_result BEFORE UPDATE ON native_delegations BEGIN SELECT RAISE(ABORT,'fixture rollback'); END;").unwrap();
        assert!(
            store
                .commit_remote_result(&task, next.clone(), &result, D::Accepted {})
                .is_err()
        );
        let untouched = store.get(&task.task_id).unwrap().unwrap();
        assert!(untouched.result.is_none() && untouched.remote_result_receipts.is_empty());
        store
            .conn
            .execute_batch("DROP TRIGGER refuse_remote_result;")
            .unwrap();
        store
            .commit_remote_result(&task, next, &result, D::Accepted {})
            .unwrap();
        let committed = store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(committed.result.as_deref(), Some(result.body.as_str()));
        assert_eq!(
            committed.remote_result_receipts[0].disposition,
            D::Accepted {}
        );
        // Une sauvegarde métier ancienne ne peut retirer la preuve du résultat.
        assert!(store.save(&task).is_err());
        store.save(&committed).unwrap();
        assert_eq!(
            store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .remote_result_receipts
                .len(),
            1
        );
    }

    #[test]
    fn remote162_conflits_bornes_et_cas_stale_sans_ecrasement() {
        use bridget_transport::protocol::RemoteResultDisposition as D;
        let (store, task, result) = remote162_receipt_fixture();
        let mut next = task.clone();
        next.result = Some(result.body.clone());
        next.state = "waiting_for_children".into();
        store
            .commit_remote_result(&task, next, &result, D::Accepted {})
            .unwrap();
        assert!(
            store
                .commit_remote_result(&task, task.clone(), &result, D::Accepted {})
                .is_err()
        );
        for index in 0..3 {
            let current = store.get(&task.task_id).unwrap().unwrap();
            let mut conflict = result.clone();
            conflict.body = format!("autre {index}");
            conflict.digest = conflict.canonical_digest().unwrap();
            store
                .commit_remote_result(&current, current.clone(), &conflict, D::RejectedConflict {})
                .unwrap();
        }
        let saved = store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(saved.result, Some(result.body));
        assert_eq!(saved.remote_result_receipts.len(), 1);
        assert!(saved.remote_result_receipts[0].conflict_seen);
        assert_eq!(saved.remote_result_receipts[0].digest, result.digest);
        assert_eq!(saved.remote_result_receipts[0].disposition, D::Accepted {});
    }

    fn legacy_task151() -> Task {
        let task_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::json!({
            "task_id": task_id, "owner": "parent", "owner_instance": "instance-parent",
            "origin_owner_instance": "instance-parent",
            "request": {"operation": "delegate", "request_id": "request-152",
                "agent_type": "fixture", "model": "fixture", "task": "mission", "cwd": "/tmp"},
            "definition": {"command": "fixture", "args": [], "protocol": "acp",
                "forbidden_env": [], "pass_env": [], "permissions": "allow",
                "queue_capacity": 32, "notify_timeout_secs": 600,
                "mcp": {"interactive": "none", "acp_session": false}, "digest": "fixture"},
            "cwd": "/tmp", "child": "child", "child_instance": null, "mission": "mission-152",
            "created_at": 100, "state": "queued", "result": null, "error": null,
            "result_sent": false, "cleanup_done": false, "failure_sent": false,
            "delivery_acked": false, "acknowledged_at": null
        });
        serde_json::from_value(payload).unwrap()
    }

    fn auto_receipt163(owner: &str, request_id: &str, epoch: i64, at: i64) -> AutoAdmissionReceipt {
        AutoAdmissionReceipt {
            owner: owner.into(),
            request_id: request_id.into(),
            canonical: format!(r#"{{"operation":"auto_delegate","request_id":"{request_id}"}}"#)
                .into_bytes(),
            metadata: br#"{"policy_revision":7,"profiles":["c01"]}"#.to_vec(),
            state: AutoAdmissionState::Pending,
            coordinator_epoch: epoch,
            revision: 0,
            policy_revision: 7,
            policy_digest: "a".repeat(64),
            candidate_id: None,
            delegate_canonical: None,
            mode: None,
            reason: None,
            task_id: None,
            refusal_code: None,
            created_at: at,
            updated_at: at,
        }
    }

    fn auto_refusal163(owner: &str, request_id: &str, epoch: i64, at: i64) -> AutoAdmissionReceipt {
        let mut receipt = auto_receipt163(owner, request_id, epoch, at);
        receipt.state = AutoAdmissionState::Refused;
        receipt.refusal_code = Some("decision_interrupted".into());
        receipt
    }

    fn child_request155() -> ChildRequest {
        ChildRequest {
            request_id: "request155".into(),
            task_id: "task155".into(),
            child_instance: "child155".into(),
            provider_session: "session155".into(),
            turn_id: "turn155".into(),
            provider_request_id: "42".into(),
            content_hash: "hash155".into(),
            kind: "user_input".into(),
            verdict: "parent".into(),
            state: "open".into(),
            outcome: None,
            summary: "Question".into(),
            facts_json: "{}".into(),
            model_text: "Choisir".into(),
            offered_json: "[\"accept\",\"decline\",\"cancel\"]".into(),
            created_at: 100,
            deadline_at: 200,
            decision: None,
            answer_json: None,
            decided_by: None,
            decided_at: None,
            delivery_state: "none".into(),
            void_reason: None,
            sent_at: None,
            delivered_at: None,
            inbox_item_id: None,
            inbox_settled: false,
        }
    }

    #[test]
    fn native157_stockage_payload150_sans_substitution() {
        // La fixture relit le JSON150 original, sans champ substitution.
        let task = legacy_task151();
        assert!(task.substitution.is_none());
    }

    #[test]
    fn remote162_task_ancienne_locale_et_placement_durable() {
        let mut task = legacy_task151();
        assert!(task.remote_placement.is_none());
        assert!(
            serde_json::to_value(&task)
                .unwrap()
                .get("remote_placement")
                .is_none()
        );
        task.remote_placement = Some(
            serde_json::from_value(serde_json::json!({
                "machine_id":"cartae", "worker_id":"worker", "provisioning_revision":1,
                "project_id":"project", "mapping_revision":1, "origin_host":"mac",
                "origin_cwd":"/Users/moi/project", "source_root_identity":"git:project",
                "relative_cwd":".", "execution_cwd":"/tmp", "base_commit":null,
                "definition_digest":"a".repeat(64), "permission_digest":"b".repeat(64)
            }))
            .unwrap(),
        );
        let root =
            std::env::temp_dir().join(format!("bridget-remote162-task-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("bridget.db");
        let store = DelegationStore::open(&path).unwrap();
        store.insert(&task, "request-152", b"canonical").unwrap();
        drop(store);
        let reopened = DelegationStore::open(&path).unwrap();
        assert_eq!(
            reopened
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .remote_placement,
            task.remote_placement
        );
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native160_task_payload157_defauts() {
        // Le payload157 ne porte ni attente quota ni décision en attente.
        let mut payload = serde_json::to_value(legacy_task151()).unwrap();
        payload.as_object_mut().unwrap().remove("quota_waits");
        payload.as_object_mut().unwrap().remove("quota_hold");
        let task: Task = serde_json::from_value(payload).unwrap();
        let decoded = serde_json::to_value(task).unwrap();
        assert_eq!(decoded.get("quota_waits"), Some(&serde_json::json!([])));
        assert_eq!(decoded.get("quota_hold"), Some(&serde_json::Value::Null));
    }

    #[test]
    fn native160_task_quota_sans_signal_lineage() {
        // Même sauvegarde/projection réelle que l'oracle152 existant.
        let mut task = legacy_task151();
        task.state = "working".into();
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        store.insert(&task, "request-152", b"canonical").unwrap();
        let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
        store.take_changes();
        let receiver = store.change_receiver().unwrap();
        receiver.try_recv().unwrap();
        let hold = serde_json::json!({
            "since":120, "error":"refus", "execution_id":"execution-mission-152"
        });
        let mut wait = serde_json::json!({
            "since":120, "reset_at":200, "round":1,
            "identity":format!("limit-resume:{}:1:200", task.task_id),
            "execution_id":"execution-mission-152", "phase":"waiting",
            "until":null, "resume_execution_id":null
        });
        for phase in [None, Some("waiting"), Some("relaunching"), Some("done")] {
            let mut updated =
                serde_json::to_value(store.get(&task.task_id).unwrap().unwrap()).unwrap();
            let (expected_hold, expected_waits) = if let Some(phase) = phase {
                wait["phase"] = serde_json::json!(phase);
                wait["until"] = if phase == "done" {
                    serde_json::json!(230)
                } else {
                    serde_json::Value::Null
                };
                wait["resume_execution_id"] = if phase == "waiting" {
                    serde_json::Value::Null
                } else {
                    serde_json::json!("execution-resumed-152")
                };
                (serde_json::Value::Null, serde_json::json!([wait.clone()]))
            } else {
                (hold.clone(), serde_json::json!([]))
            };
            // Le checkpoint durable peut précéder la fermeture SQL de la feuille.
            // Il ne doit pas devenir une erreur publique pendant Relaunching.
            updated["error"] = if phase == Some("relaunching") {
                serde_json::json!("native_relaunch_failed")
            } else {
                serde_json::Value::Null
            };
            updated["quota_hold"] = expected_hold.clone();
            updated["quota_waits"] = expected_waits.clone();
            store
                .save(&serde_json::from_value(updated).unwrap())
                .unwrap();
            let saved = serde_json::to_value(store.get(&task.task_id).unwrap().unwrap()).unwrap();
            assert_eq!(saved.get("quota_hold"), Some(&expected_hold));
            assert_eq!(saved.get("quota_waits"), Some(&expected_waits));
            if phase == Some("relaunching") {
                assert_eq!(
                    saved["error"], "native_relaunch_failed",
                    "checkpoint interne conservé"
                );
                assert_eq!(
                    store.get(&task.task_id).unwrap().unwrap().projected_error(),
                    None
                );
            }
            assert_eq!(
                store.lineage_page("parent", 100, None, |_| false).unwrap(),
                before
            );
            assert!(store.take_changes().is_empty());
            assert!(receiver.try_recv().is_err());
        }
        // Après la clôture confirmée, failed et son erreur deviennent visibles.
        let mut failed = store.get(&task.task_id).unwrap().unwrap();
        failed.state = "failed".into();
        failed.error = Some("native_relaunch_failed".into());
        store.save(&failed).unwrap();
        let after = store.lineage_page("parent", 100, None, |_| false).unwrap();
        assert_ne!(after, before);
        assert_eq!(after["tasks"][0]["status"], "failed");
        assert_eq!(after["tasks"][0]["error"], "native_relaunch_failed");
        assert!(!store.take_changes().is_empty());
        assert!(receiver.try_recv().is_ok());
    }

    #[test]
    fn native155_store_payload150_sans_mode() {
        let task = legacy_task151();
        assert_eq!(task.child_requests, None);
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        store.insert(&task, "request-152", b"canonical").unwrap();
        assert_eq!(
            store.get(&task.task_id).unwrap().unwrap().child_requests,
            None
        );
    }

    #[test]
    fn native155_store_doublon_identique_meme_ligne() {
        use bridget_transport::protocol::ChildProviderRequestId;
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        for (index, provider_id) in [
            ChildProviderRequestId::Number(i64::MIN),
            ChildProviderRequestId::String(i64::MIN.to_string()),
        ]
        .into_iter()
        .enumerate()
        {
            let mut request = child_request155();
            request.request_id = format!("request155-{index}");
            request.provider_request_id = serde_json::to_string(&provider_id).unwrap();
            assert_eq!(store.insert_child_request(&request).unwrap(), request);
            assert!(
                store
                    .decide_child_request(
                        &request.request_id,
                        "accept",
                        Some("{}"),
                        "parent",
                        "answered_by_parent",
                        110
                    )
                    .unwrap()
            );
            let decided = store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap();
            assert_eq!(
                serde_json::from_str::<ChildProviderRequestId>(&decided.provider_request_id)
                    .unwrap(),
                provider_id
            );
            assert_eq!(store.insert_child_request(&request).unwrap(), decided);
        }
        assert_eq!(store.child_request_counts("task155").unwrap(), (0, 2));
    }

    #[test]
    fn native155_store_doublon_different_request_conflict() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let request = child_request155();
        store.insert_child_request(&request).unwrap();
        let mut divergent = request.clone();
        divergent.content_hash = "other".into();
        assert_eq!(
            store.insert_child_request(&divergent).unwrap_err(),
            "request_conflict"
        );
        assert_eq!(
            store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap(),
            request
        );
    }

    #[test]
    fn native155_store_terminal_immuable() {
        for state in ["expired", "cancelled", "refused", "answered"] {
            let store = DelegationStore::open(Path::new(":memory:")).unwrap();
            let request = child_request155();
            store.insert_child_request(&request).unwrap();
            if state == "answered" {
                assert!(
                    store
                        .decide_child_request(
                            &request.request_id,
                            "accept",
                            Some("{}"),
                            "parent",
                            "answered_by_parent",
                            110
                        )
                        .unwrap()
                );
            } else {
                assert!(
                    store
                        .close_child_request(&request.request_id, state, state, 110)
                        .unwrap()
                );
            }
            let closed = store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap();
            assert!(
                !store
                    .close_child_request(&request.request_id, "cancelled", "cancelled", 120)
                    .unwrap()
            );
            assert!(
                !store
                    .decide_child_request(
                        &request.request_id,
                        "decline",
                        None,
                        "human",
                        "answered_by_human",
                        120
                    )
                    .unwrap()
            );
            assert_eq!(
                store
                    .get_child_request(&request.request_id)
                    .unwrap()
                    .unwrap(),
                closed
            );
        }
    }

    #[test]
    fn native155_store_void_ne_redevient_pas_pending() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let request = child_request155();
        store.insert_child_request(&request).unwrap();
        assert!(
            store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    Some("{}"),
                    "parent",
                    "answered_by_parent",
                    110
                )
                .unwrap()
        );
        assert!(
            store
                .void_pending_child_request(&request.request_id, "paused")
                .unwrap()
        );
        assert!(
            !store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    Some("{}"),
                    "parent",
                    "answered_by_parent",
                    120
                )
                .unwrap()
        );
        assert!(!store.claim_child_request(&request.request_id, 120).unwrap());
        let void = store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap();
        assert_eq!(void.delivery_state, "void");
        assert_eq!(void.void_reason.as_deref(), Some("paused"));
        assert_eq!(void.decision.as_deref(), Some("accept"));
        assert_eq!(store.insert_child_request(&request).unwrap(), void);
    }

    #[test]
    fn native155_store_sent_ne_redevient_pas_pending() {
        for delivery in ["delivered", "void", "unknown"] {
            let store = DelegationStore::open(Path::new(":memory:")).unwrap();
            let request = child_request155();
            store.insert_child_request(&request).unwrap();
            assert!(
                store
                    .decide_child_request(
                        &request.request_id,
                        "accept",
                        Some("{}"),
                        "parent",
                        "answered_by_parent",
                        110
                    )
                    .unwrap()
            );
            assert!(store.claim_child_request(&request.request_id, 120).unwrap());
            assert!(!store.claim_child_request(&request.request_id, 121).unwrap());
            assert!(
                !store
                    .void_pending_child_request(&request.request_id, "cancelled")
                    .unwrap()
            );
            assert!(
                !store
                    .decide_child_request(
                        &request.request_id,
                        "accept",
                        Some("{}"),
                        "parent",
                        "answered_by_parent",
                        121
                    )
                    .unwrap()
            );
            let sent = store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap();
            assert_eq!(sent.delivery_state, "sent");
            assert_eq!(sent.sent_at, Some(120));
            assert!(
                store
                    .settle_sent_child_request(
                        &request.request_id,
                        delivery,
                        Some("closed_before_apply"),
                        130
                    )
                    .unwrap()
            );
            assert!(
                !store
                    .settle_sent_child_request(&request.request_id, "void", None, 131)
                    .unwrap()
            );
            let settled = store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap();
            assert_eq!(settled.state, sent.state);
            assert_eq!(settled.decision, sent.decision);
            assert_eq!(settled.answer_json, sent.answer_json);
            assert_eq!(settled.decided_at, sent.decided_at);
            assert_eq!(settled.delivery_state, delivery);
            assert_eq!(
                settled.outcome.as_deref(),
                Some(if delivery == "unknown" {
                    "cancelled_outcome_unknown"
                } else {
                    "answered_by_parent"
                })
            );
            assert_eq!(
                settled.delivered_at,
                if delivery == "delivered" {
                    Some(130)
                } else {
                    None
                }
            );
            assert_eq!(store.insert_child_request(&request).unwrap(), settled);
        }
    }

    #[test]
    fn native155_store_cas_echeance_et_claim_distinct() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let request = child_request155();
        store.insert_child_request(&request).unwrap();
        assert_eq!(
            store.child_request_counts(&request.task_id).unwrap(),
            (1, 1)
        );
        assert_eq!(
            store.open_child_requests(&request.task_id).unwrap(),
            vec![request.clone()]
        );
        assert!(store.due_child_requests(199).unwrap().is_empty());
        assert_eq!(
            store.due_child_requests(200).unwrap(),
            vec![request.clone()]
        );
        assert!(
            !store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    None,
                    "parent",
                    "answered_by_parent",
                    200
                )
                .unwrap()
        );
        assert!(!store.claim_child_request(&request.request_id, 199).unwrap());
        assert_eq!(
            store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap(),
            request
        );
        assert!(
            store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    None,
                    "parent",
                    "answered_by_parent",
                    199
                )
                .unwrap()
        );
        assert!(!store.claim_child_request(&request.request_id, 200).unwrap());
        assert!(store.claim_child_request(&request.request_id, 199).unwrap());
        assert!(!store.claim_child_request(&request.request_id, 199).unwrap());
    }

    #[test]
    fn native154_stockage_payloads150_152_153_sans_workspace() {
        let base = serde_json::to_value(legacy_task151()).unwrap();
        for version in [150, 152, 153] {
            let mut payload = base.clone();
            payload.as_object_mut().unwrap().remove("workspace");
            payload
                .as_object_mut()
                .unwrap()
                .remove("workspace_report_mission");
            if version >= 152 {
                payload["launch_issued_at"] = serde_json::json!(110);
                payload["restart_resumes"] = serde_json::json!(1);
            }
            if version >= 153 {
                payload["round"] = serde_json::json!(2);
                payload["guarded"] = serde_json::json!(true);
                payload["request"]["keep_alive_secs"] = serde_json::json!(600);
                payload["request"]["brief"] =
                    serde_json::json!({"objective":"objectif","summary":"résultat"});
            }
            let task: Task = serde_json::from_value(payload).unwrap();
            assert!(task.workspace.is_none(), "payload {version}");
            assert!(task.workspace_report_mission.is_none(), "payload {version}");
        }
    }

    #[test]
    fn native154_stockage_workspace_sans_signal_lineage() {
        let mut task = legacy_task151();
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        store.insert(&task, "request-152", b"canonical").unwrap();
        let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
        store.take_changes();
        let receiver = store.change_receiver().unwrap();
        receiver.try_recv().unwrap();
        task.workspace = Some(WorkspaceRecord {
            strategy: WorkspaceStrategy::Worktree,
            state: WorkspaceState::Active,
            toplevel: "/tmp/repository".into(),
            path: "/tmp/repository/.worktrees/task".into(),
            branch: "bridget/task".into(),
            base_commit: "a".repeat(40),
            head_commit: None,
            branch_commit: None,
            head_on_branch: None,
            commit_count: None,
            commits: Vec::new(),
            truncated: false,
            dirty: None,
        });
        for state in [WorkspaceState::Active, WorkspaceState::Kept] {
            task.workspace.as_mut().unwrap().state = state;
            if task.workspace.as_ref().unwrap().state == WorkspaceState::Kept {
                task.workspace_report_mission = Some(task.mission.clone());
            }
            store.save(&task).unwrap();
            let saved = store.get(&task.task_id).unwrap().unwrap();
            assert_eq!(
                serde_json::to_value(saved.workspace).unwrap(),
                serde_json::to_value(&task.workspace).unwrap()
            );
            assert_eq!(
                saved.workspace_report_mission,
                task.workspace_report_mission
            );
            assert_eq!(
                store.lineage_page("parent", 100, None, |_| false).unwrap(),
                before
            );
            assert!(store.take_changes().is_empty());
            assert!(receiver.try_recv().is_err());
        }
        let mut record = serde_json::to_value(task.workspace.unwrap()).unwrap();
        record["state"] = serde_json::json!("removed");
        assert!(serde_json::from_value::<WorkspaceRecord>(record).is_err());
    }

    #[test]
    fn native152_task_payload151_defauts_et_champs_invisibles_lineage() {
        let task = legacy_task151();
        let task_id = task.task_id.clone();
        let decoded = serde_json::to_value(&task).unwrap();
        for (field, default) in [
            ("launch_issued_at", serde_json::json!(null)),
            ("mission_issued_at", serde_json::json!(null)),
            ("restart_resumes", serde_json::json!(0)),
            ("cancel_stopped", serde_json::json!([])),
        ] {
            assert_eq!(decoded.get(field), Some(&default));
        }
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        store.insert(&task, "request-152", b"canonical").unwrap();
        let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
        store.take_changes();
        let receiver = store.change_receiver().unwrap();
        receiver.try_recv().unwrap();
        let mut updated = serde_json::to_value(store.get(&task_id).unwrap().unwrap()).unwrap();
        let new_values = [
            ("launch_issued_at", serde_json::json!(110)),
            ("mission_issued_at", serde_json::json!(120)),
            ("restart_resumes", serde_json::json!(2)),
            ("cancel_stopped", serde_json::json!(["child"])),
        ];
        for (field, value) in &new_values {
            updated[*field] = value.clone();
        }
        store
            .save(&serde_json::from_value(updated).unwrap())
            .unwrap();
        let saved = serde_json::to_value(store.get(&task_id).unwrap().unwrap()).unwrap();
        for (field, value) in new_values {
            assert_eq!(saved.get(field), Some(&value));
        }
        assert_eq!(
            store.lineage_page("parent", 100, None, |_| false).unwrap(),
            before
        );
        assert!(store.take_changes().is_empty());
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn native152_refus_rejeu_identique() {
        let dir = std::env::temp_dir().join(format!("native152-refus-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("native.sqlite");
        let store = DelegationStore::open(&path).unwrap();
        let epoch = store.wait_epoch().get();
        let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
        let receiver = store.change_receiver().unwrap();
        let codes = [
            "cwd_outside_parent_project",
            "cwd_unavailable",
            "model_unavailable",
            "effort_unavailable",
            "agent_type_unknown",
        ];
        for (index, code) in codes.into_iter().enumerate() {
            let request = format!("request-{index}");
            assert!(
                store
                    .record_refusal("parent", &request, b"canonical", code, 100)
                    .unwrap()
            );
            assert!(
                !store
                    .record_refusal("parent", &request, b"different", "cwd_unavailable", 200)
                    .unwrap()
            );
            assert_eq!(
                store
                    .by_agent_request("parent", &request, b"canonical")
                    .unwrap_err(),
                code
            );
            let saved: (Vec<u8>, String, i64) = store.conn.query_row(
                "SELECT canonical,code,refused_at FROM native_delegation_refusals WHERE owner=?1 AND request_id=?2",
                params!["parent", request], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).unwrap();
            assert_eq!(saved, (b"canonical".to_vec(), code.to_string(), 100));
        }
        assert_eq!(store.wait_epoch().get(), epoch);
        assert!(store.take_changes().is_empty());
        assert!(receiver.try_recv().is_err());
        assert_eq!(
            store.lineage_page("parent", 100, None, |_| false).unwrap(),
            before
        );
        drop(store);
        let store = DelegationStore::open(&path).unwrap();
        assert_eq!(
            store
                .by_agent_request("parent", "request-0", b"canonical")
                .unwrap_err(),
            codes[0]
        );
        assert!(
            store
                .by_agent_request("other", "request-0", b"canonical")
                .unwrap()
                .is_none()
        );
        let mut task = legacy_task151();
        if let NativeDelegationRequest::Delegate { request_id, .. } = &mut task.request {
            *request_id = "request-0".into();
        }
        store.insert(&task, "request-0", b"canonical").unwrap();
        assert_eq!(
            store
                .by_agent_request("parent", "request-0", b"canonical")
                .unwrap()
                .unwrap()
                .task_id,
            task.task_id
        );
        store
            .conn
            .execute(
                "DELETE FROM native_delegation_refusals WHERE request_id='request-0'",
                [],
            )
            .unwrap();
        assert!(
            !store
                .record_refusal("parent", "request-0", b"canonical", codes[0], 300)
                .unwrap()
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM native_delegation_refusals WHERE request_id='request-0'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        drop(store);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn native152_refus_autre_enveloppe_mismatch() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        store
            .record_refusal("parent", "request", b"canonical", "model_unavailable", 100)
            .unwrap();
        assert_eq!(
            store
                .by_agent_request("parent", "request", b"other")
                .unwrap_err(),
            "envelope_mismatch"
        );
    }

    fn assert_refusal_not_retained(code: &str) {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        assert!(
            !store
                .record_refusal("parent", "request", b"canonical", code, 100)
                .unwrap()
        );
        assert!(
            store
                .by_agent_request("parent", "request", b"other")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM native_delegation_refusals",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn native152_refus_transitoire_jamais_garde_referent_paused() {
        assert_refusal_not_retained("referent_paused");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_task_limit() {
        assert_refusal_not_retained("task_limit");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_retention_saturated() {
        assert_refusal_not_retained("retention_saturated");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_identity_unavailable() {
        assert_refusal_not_retained("identity_unavailable");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_reply_timeout_exceeds_parent() {
        assert_refusal_not_retained("reply_timeout_exceeds_parent");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_parent_task_unavailable() {
        assert_refusal_not_retained("parent_task_unavailable");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_parent_execution_ambiguous() {
        assert_refusal_not_retained("parent_execution_ambiguous");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_delegation_grant_required() {
        assert_refusal_not_retained("delegation_grant_required");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_permission_not_inherited() {
        assert_refusal_not_retained("permission_not_inherited");
    }
    #[test]
    fn native152_refus_transitoire_jamais_garde_permission_attestation_unavailable() {
        assert_refusal_not_retained("permission_attestation_unavailable");
    }

    #[test]
    fn native151_attente_compteur_reveil_et_lecture_non_bloquante() {
        let epoch = WaitEpoch::default();
        let sleeper = epoch.clone();
        let seen = epoch.get();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let join = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            sleeper.wait_changed(seen, Duration::from_secs(60));
        });
        started_rx.recv().unwrap();
        let start = std::time::Instant::now();
        assert_eq!(epoch.get(), 0);
        epoch.bump();
        join.join().unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(epoch.get(), 1);
    }

    #[test]
    fn native151_attente_compteur_tranche_et_evenement_deja_passe() {
        let epoch = WaitEpoch::default();
        let start = std::time::Instant::now();
        epoch.wait_changed(0, Duration::from_millis(40));
        assert!(start.elapsed() >= Duration::from_millis(35));
        epoch.bump();
        let start = std::time::Instant::now();
        epoch.wait_changed(0, Duration::from_secs(60));
        assert!(start.elapsed() < Duration::from_millis(100));
    }
    #[test]
    fn grant_survives_restart_and_revoke() {
        let dir = std::env::temp_dir().join(format!("bridget-native-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("native.sqlite");
        let store = DelegationStore::open(&path).unwrap();
        store
            .grant("owner", &dir, SpawnPosture::Development, false)
            .unwrap();
        drop(store);
        let store = DelegationStore::open(&path).unwrap();
        assert_eq!(
            store.permission("owner").unwrap().unwrap().1,
            SpawnPosture::Development
        );
        assert!(store.permission("other").unwrap().is_none());
        store
            .grant("owner", &dir, SpawnPosture::Discovery, true)
            .unwrap();
        assert!(store.permission("owner").unwrap().is_none());
        drop(store);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn control158_halt_store_pose_levee() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        assert_eq!(
            store.delegation_halt().unwrap(),
            HaltState {
                halted: false,
                since: None,
                by: None,
                reason: None,
                generation: 0,
            }
        );
        let pose = store
            .set_delegation_halt("A", true, "cli", Some("essai"), || Ok(0), 100)
            .unwrap();
        assert_eq!(
            pose,
            serde_json::json!({"type":"delegation_halt_state","halted":true,
            "since":100,"by":"cli","reason":"essai","generation":1,"cancelled":0,"replayed":false})
        );
        let levee = store
            .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
            .unwrap();
        assert_eq!(levee["halted"], false);
        assert_eq!(levee["generation"], 2);
        assert!(levee["since"].is_null() && levee["by"].is_null() && levee["reason"].is_null());
        assert!(!store.delegation_halt().unwrap().halted);
        assert_eq!(
            store
                .conn
                .query_row("SELECT count(*) FROM native_delegation_halt", [], |row| row
                    .get::<_, u32>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn control158_halt_store_a_b_rejeu_a() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let first = store
            .set_delegation_halt("A", true, "cli", Some("premier"), || Ok(0), 100)
            .unwrap();
        store
            .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
            .unwrap();
        let before = store.delegation_halt().unwrap();
        let original_receipt: String = store
            .conn
            .query_row(
                "SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let replay = store
            .set_delegation_halt(
                "A",
                true,
                "humain",
                Some("autre"),
                || panic!("aucun comptage au rejeu"),
                999,
            )
            .unwrap();
        let mut expected = first;
        expected["replayed"] = serde_json::json!(true);
        assert_eq!(replay, expected);
        assert_eq!(store.delegation_halt().unwrap(), before);
        assert!(!before.halted);
        assert_eq!(store.conn.query_row("SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A'",
            [], |row| row.get::<_,String>(0)).unwrap(), original_receipt);
    }

    #[test]
    fn control158_halt_store_a_b_rejeu_a_apres_redemarrage() {
        // Reprise du montage SQLite disque déjà utilisé par les tests du magasin.
        let dir = std::env::temp_dir().join(format!("native158-halt-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("native.sqlite");
        let store = DelegationStore::open(&path).unwrap();
        let first = store
            .set_delegation_halt("A", true, "cli", Some("premier"), || Ok(0), 100)
            .unwrap();
        store
            .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
            .unwrap();
        drop(store);
        let store = DelegationStore::open(&path).unwrap();
        let before = store.delegation_halt().unwrap();
        let mut expected = first;
        expected["replayed"] = serde_json::json!(true);
        assert_eq!(
            store
                .set_delegation_halt(
                    "A",
                    true,
                    "cli",
                    Some("premier"),
                    || panic!("aucun comptage après reprise"),
                    300
                )
                .unwrap(),
            expected
        );
        assert_eq!(store.delegation_halt().unwrap(), before);
        assert!(!before.halted);
        assert_eq!(before.generation, 2);
        drop(store);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn control158_halt_store_recu_porte_la_reponse() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        // Trois arbres ciblés : valeur fournie par l'appelant, pas une terminaison simulée.
        let first = store
            .set_delegation_halt("A", true, "cli", Some("compte fourni"), || Ok(3), 100)
            .unwrap();
        let saved: String = store
            .conn
            .query_row(
                "SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&saved).unwrap(),
            first
        );
        assert_eq!(first["cancelled"], 3);
        store
            .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
            .unwrap();
        let replay = store
            .set_delegation_halt("A", true, "cli", None, || Ok(0), 300)
            .unwrap();
        assert_eq!(replay["cancelled"], 3);
        assert_eq!(replay["generation"], first["generation"]);
        assert_eq!(replay["replayed"], true);
    }

    #[test]
    fn control158_halt_store_recus_bornes() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        for index in 0..257 {
            store
                .set_delegation_halt(
                    &format!("command-{index}"),
                    true,
                    "cli",
                    None,
                    || Ok(0),
                    index,
                )
                .unwrap();
        }
        let count: u32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM native_delegation_halt_commands",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 256);
        assert!(!store.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_delegation_halt_commands WHERE command_id='command-0')",
            [], |row| row.get::<_,bool>(0)).unwrap());
        assert!(store.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_delegation_halt_commands WHERE command_id='command-256')",
            [], |row| row.get::<_,bool>(0)).unwrap());
        assert_eq!(
            store
                .set_delegation_halt("command-256", true, "cli", None, || Ok(0), 999)
                .unwrap()["replayed"],
            true
        );
    }

    #[test]
    fn control158_halt_store_motif_borne() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let text = "é".repeat(crate::referent_control::MAX_REASON_CHARS + 1);
        let response = store
            .set_delegation_halt("A", true, "cli", Some(&text), || Ok(0), 100)
            .unwrap();
        let expected = "é".repeat(crate::referent_control::MAX_REASON_CHARS);
        assert_eq!(response["reason"], expected);
        assert_eq!(
            store.delegation_halt().unwrap().reason.as_deref(),
            Some(expected.as_str())
        );
    }

    // Quatre consommateurs réels ; même legacy_task151 et vraie insertion du magasin.
    // Aucun faux verdict, processus, harnais ou hook d'horloge.
    fn overview158_insert(store: &DelegationStore, index: u32, mut task: Task) -> Task {
        task.owner = format!("parent-{}", index % 3);
        task.owner_instance = format!("instance-parent-{}", index % 3);
        task.child = format!("child-{index}");
        task.mission = format!("mission-{index}");
        task.mission_deadline_at = Some(task.created_at + 1000);
        let key = format!("overview-{index}");
        store.insert(&task, &key, key.as_bytes()).unwrap();
        task
    }

    #[test]
    fn control158_overview_trois_proprietaires() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut tasks: Vec<Task> = Vec::new();
        for index in 0..10 {
            let mut task = legacy_task151();
            task.state = "working".into();
            task.created_at = now - 30 + index as i64;
            // La lignée est fixée avant insertion, comme dans le magasin livré.
            if index == 3 {
                task.parent_task_id = Some(tasks[0].task_id.clone());
            }
            tasks.push(overview158_insert(&store, index, task));
        }
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let response = store.overview_page("all", 100).unwrap();
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let items = response["items"].as_array().unwrap();
        assert_eq!(items.len(), 10);
        assert_eq!(response["total_active"], 10);
        let owners: std::collections::BTreeSet<&str> = items
            .iter()
            .map(|item| item["owner"].as_str().unwrap())
            .collect();
        assert_eq!(owners.len(), 3);
        for (item, task) in items.iter().zip(tasks.iter().rev()) {
            assert_eq!(item["task_id"], task.task_id);
            assert_eq!(item["owner"], task.owner);
            assert_eq!(item["child"], task.child);
            assert_eq!(item["state"], "working");
            assert_eq!(
                item["parent_task_id"],
                serde_json::to_value(&task.parent_task_id).unwrap()
            );
            let age = item["age_secs"].as_i64().unwrap();
            let remaining = item["deadline_in_secs"].as_i64().unwrap();
            assert!(
                (before.min(after) - task.created_at..=before.max(after) - task.created_at)
                    .contains(&age)
            );
            assert!(
                (task.mission_deadline_at.unwrap() - before.max(after)
                    ..=task.mission_deadline_at.unwrap() - before.min(after))
                    .contains(&remaining)
            );
        }
        assert_eq!(response["halted"], false);
        assert!(
            response["since"].is_null() && response["by"].is_null() && response["reason"].is_null()
        );
    }

    #[test]
    fn control158_overview_filtre_actif() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let mut live = Vec::new();
        for (index, state) in [
            "working",
            "cancelling",
            "result_available",
            "failed",
            "cancelled",
        ]
        .into_iter()
        .enumerate()
        {
            let mut task = legacy_task151();
            task.state = state.into();
            task.created_at = 100 + index as i64;
            if index == 2 {
                task.guarded = true;
            }
            // Garde immuable posée avant admission.
            else if index > 2 {
                task.cleanup_done = true;
            }
            let task = overview158_insert(&store, index as u32, task);
            if index <= 2 {
                live.push(task.task_id);
            }
        }
        let active = store.overview_page("active", 100).unwrap();
        let ids: Vec<&str> = active["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["task_id"].as_str().unwrap())
            .collect();
        let expected: Vec<&str> = live.iter().rev().map(String::as_str).collect();
        assert_eq!(ids, expected);
        assert_eq!(active["total_active"], 3);
        assert_eq!(
            store.overview_page("all", 100).unwrap()["items"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
    }

    #[test]
    fn control158_overview_limite_100() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        for index in 0..101 {
            let mut task = legacy_task151();
            task.state = "working".into();
            task.created_at = index as i64;
            overview158_insert(&store, index, task);
        }
        let page = store.overview_page("all", 100).unwrap();
        assert_eq!(page["items"].as_array().unwrap().len(), 100);
        assert_eq!(page["total_active"], 101, "le total précède la pagination");
        assert_eq!(
            store.overview_page("active", 1).unwrap()["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        for invalid in [0, 101] {
            assert_eq!(
                store.overview_page("all", invalid).unwrap_err(),
                bridget_transport::protocol::DelegationControlRefusal::InvalidRequest
            );
        }
    }

    #[test]
    fn control158_overview_sans_corps_ni_resultat() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let mut seed = legacy_task151();
        seed.state = "result_available".into();
        seed.created_at = 100;
        let mut task = overview158_insert(&store, 0, seed);
        if let NativeDelegationRequest::Delegate { task: body, .. } = &mut task.request {
            *body = "MISSION_PRIVEE_158".into();
        }
        task.result = Some("RESULTAT_PRIVE_158".into());
        task.cwd = "/chemin-prive-158".into();
        task.mission_deadline_at = None;
        store.save(&task).unwrap();
        let response = store.overview_page("all", 1).unwrap();
        let item = &response["items"][0];
        let keys: std::collections::BTreeSet<&str> = item
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            std::collections::BTreeSet::from([
                "task_id",
                "owner",
                "child",
                "state",
                "age_secs",
                "deadline_in_secs",
                "parent_task_id"
            ])
        );
        assert!(item["deadline_in_secs"].is_null());
        let wire = response.to_string();
        for private in [
            "MISSION_PRIVEE_158",
            "RESULTAT_PRIVE_158",
            "/chemin-prive-158",
        ] {
            assert!(
                !wire.contains(private),
                "la projection ne contient aucune donnée privée"
            );
        }
    }

    #[test]
    fn spec163_recu_auto_additif_borne_et_idempotent() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let receipt = AutoAdmissionReceipt {
            owner: "owner-163".into(),
            request_id: "request-163".into(),
            canonical: br#"{"operation":"auto_delegate"}"#.to_vec(),
            metadata: br#"{"policy_revision":7,"profiles":["c01"]}"#.to_vec(),
            state: AutoAdmissionState::Pending,
            coordinator_epoch: 1,
            revision: 0,
            policy_revision: 7,
            policy_digest: "a".repeat(64),
            candidate_id: None,
            delegate_canonical: None,
            mode: None,
            reason: None,
            task_id: None,
            refusal_code: None,
            created_at: 100,
            updated_at: 100,
        };
        assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
        assert!(!store.insert_auto_admission_receipt(&receipt).unwrap());
        assert_eq!(
            store
                .auto_admission_receipt("owner-163", "request-163")
                .unwrap(),
            Some(receipt.clone())
        );
        let mut collision = receipt.clone();
        collision.canonical = b"different".to_vec();
        assert_eq!(
            store.insert_auto_admission_receipt(&collision).unwrap_err(),
            "envelope_mismatch"
        );
        let mut oversized = receipt;
        oversized.request_id = "too-large-163".into();
        oversized.metadata = vec![b'x'; MAX_AUTO_ADMISSION_METADATA_BYTES + 1];
        assert_eq!(
            store.insert_auto_admission_receipt(&oversized).unwrap_err(),
            "auto_admission_invalid"
        );
    }

    #[test]
    fn spec163_choix_et_task_sont_durables_et_atomiques() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let receipt = AutoAdmissionReceipt {
            owner: "owner-163".into(),
            request_id: "request-atomic-163".into(),
            canonical: br#"{"operation":"auto_delegate","request_id":"request-atomic-163"}"#
                .to_vec(),
            metadata: br#"{"policy_revision":7,"candidates":["c01"]}"#.to_vec(),
            state: AutoAdmissionState::Pending,
            coordinator_epoch: 1,
            revision: 0,
            policy_revision: 7,
            policy_digest: "a".repeat(64),
            candidate_id: None,
            delegate_canonical: None,
            mode: None,
            reason: None,
            task_id: None,
            refusal_code: None,
            created_at: 100,
            updated_at: 100,
        };
        assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
        let chosen = store
            .choose_auto_admission(
                &receipt.owner,
                &receipt.request_id,
                &receipt.canonical,
                "c01".into(),
                b"delegate-163".to_vec(),
                AutoAdmissionMode::Automatic,
                Some("profil autorisé".into()),
                101,
            )
            .unwrap();
        assert_eq!(chosen.state, AutoAdmissionState::Chosen);
        assert_eq!(
            store
                .choose_auto_admission(
                    &receipt.owner,
                    &receipt.request_id,
                    &receipt.canonical,
                    "c01".into(),
                    b"delegate-163".to_vec(),
                    AutoAdmissionMode::Automatic,
                    None,
                    102,
                )
                .unwrap_err(),
            "auto_admission_already_resolved"
        );

        let mut task = legacy_task151();
        task.owner = receipt.owner.clone();
        task.root_owner_agent_id = receipt.owner.clone();
        task.task_id = "16300000-0000-4000-8000-000000000001".into();
        task.child = "16300000-0000-4000-8000-000000000002".into();
        task.mission = "16300000-0000-4000-8000-000000000003".into();
        store
            .admit_auto_task(&chosen, &task, &receipt.request_id, b"delegate-163", 102)
            .unwrap();
        let admitted = store
            .auto_admission_receipt(&receipt.owner, &receipt.request_id)
            .unwrap()
            .unwrap();
        assert_eq!(admitted.state, AutoAdmissionState::Admitted);
        assert_eq!(admitted.task_id.as_deref(), Some(task.task_id.as_str()));
        assert!(store.get(&task.task_id).unwrap().is_some());
        assert_eq!(
            store
                .refuse_auto_admission(&chosen, "remote_prepare_invalid", 103)
                .unwrap_err(),
            "admission_pending",
            "le CAS stale ne ferme pas une Task déjà admise"
        );
        assert_eq!(
            store
                .auto_admission_receipt(&receipt.owner, &receipt.request_id)
                .unwrap()
                .unwrap()
                .state,
            AutoAdmissionState::Admitted
        );
    }

    #[test]
    fn spec163_t026_compteurs_union_places_et_collision_native() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let owner = "owner-t026";
        let pending = auto_receipt163(owner, "auto-pending", 1, 10);
        let refused = auto_refusal163(owner, "auto-refused", 1, 11);
        let linked = auto_receipt163(owner, "auto-linked", 1, 12);
        assert!(store.insert_auto_admission_receipt(&pending).unwrap());
        assert!(store.insert_auto_admission_receipt(&refused).unwrap());
        assert!(store.insert_auto_admission_receipt(&linked).unwrap());
        assert!(
            !store
                .record_refusal(owner, &pending.request_id, b"native", "cwd_unavailable", 12)
                .unwrap(),
            "un refus natif ne double jamais une clé auto"
        );

        let chosen = store
            .choose_auto_admission(
                &linked.owner,
                &linked.request_id,
                &linked.canonical,
                "c01".into(),
                b"delegate-linked-t026".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                13,
            )
            .unwrap();
        let mut task = legacy_task151();
        task.owner = owner.into();
        task.root_owner_agent_id = owner.into();
        task.owner_instance = "instance-t026".into();
        task.mission = "mission-t026-linked".into();
        store
            .admit_auto_task(
                &chosen,
                &task,
                &linked.request_id,
                b"delegate-linked-t026",
                14,
            )
            .unwrap();

        assert!(
            store
                .record_refusal(owner, "native-refusal", b"native", "cwd_unavailable", 15)
                .unwrap()
        );
        let native_collision = auto_receipt163(owner, "native-refusal", 1, 16);
        assert_eq!(
            store
                .insert_auto_admission_receipt(&native_collision)
                .unwrap_err(),
            "auto_admission_reserved"
        );
        assert_eq!(
            store.admission_counts("instance-t026").unwrap(),
            (2, 1, 3),
            "refus natif historique hors compteur et reçu/Task lié compté une fois"
        );
        assert_eq!(
            store.admission_counts_for_owner(owner).unwrap(),
            (2, 2, 3),
            "les places auto pending et Task liée utilisent l'owner durable"
        );
    }

    #[test]
    fn spec163_t026_purge_refus_auto_tombstone_et_retention_trente_jours() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let owner = "owner-purge-t026";
        let old = auto_refusal163(owner, "old", 1, 10);
        let recent = auto_refusal163(owner, "recent", 1, 20);
        let pending = auto_receipt163(owner, "pending", 1, 30);
        for receipt in [&old, &recent, &pending] {
            assert!(store.insert_auto_admission_receipt(receipt).unwrap());
        }
        assert_eq!(store.purge_auto_refusals_under_pressure(100, 3).unwrap(), 1);
        assert_eq!(
            store.auto_admission_receipt(owner, "old").unwrap_err(),
            "request_expired"
        );
        assert_eq!(
            store
                .by_agent_request(owner, "old", &old.canonical)
                .unwrap_err(),
            "request_expired"
        );
        assert_eq!(
            store.insert_auto_admission_receipt(&old).unwrap_err(),
            "request_expired"
        );
        assert_eq!(
            store.auto_admission_receipt(owner, "recent").unwrap(),
            Some(recent)
        );
        assert_eq!(
            store.auto_admission_receipt(owner, "pending").unwrap(),
            Some(pending)
        );

        let aged = auto_refusal163(owner, "aged", 1, 0);
        assert!(store.insert_auto_admission_receipt(&aged).unwrap());
        let thirty_days = 30 * 24 * 3600;
        assert_eq!(
            store
                .purge_expired_auto_admission_refusals(thirty_days, 128)
                .unwrap(),
            0,
            "le refus est conservé pendant les trente jours complets"
        );
        assert_eq!(
            store.auto_admission_receipt(owner, "aged").unwrap(),
            Some(aged.clone())
        );
        assert_eq!(
            store
                .purge_expired_auto_admission_refusals(thirty_days + 1, 128)
                .unwrap(),
            1
        );
        assert_eq!(
            store.auto_admission_receipt(owner, "aged").unwrap_err(),
            "request_expired"
        );
    }

    #[test]
    fn spec163_t026_ne_purge_ni_vivants_ni_etat_non_committe() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let owner = "owner-vivants-t026";
        let pending = auto_receipt163(owner, "pending", 1, 30);
        let started = auto_receipt163(owner, "started", 1, 20);
        let chosen = auto_receipt163(owner, "chosen", 1, 10);
        let refused = auto_refusal163(owner, "refused", 1, 1);
        for receipt in [&pending, &started, &chosen, &refused] {
            assert!(store.insert_auto_admission_receipt(receipt).unwrap());
        }
        store.start_auto_decision(&started, 21).unwrap();
        store
            .choose_auto_admission(
                &chosen.owner,
                &chosen.request_id,
                &chosen.canonical,
                "c01".into(),
                b"delegate-vivant-t026".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                11,
            )
            .unwrap();
        assert_eq!(store.purge_auto_refusals_under_pressure(100, 4).unwrap(), 1);
        for request_id in ["pending", "started", "chosen"] {
            assert!(
                store
                    .auto_admission_receipt(owner, request_id)
                    .unwrap()
                    .is_some()
            );
        }

        let rollback = auto_refusal163(owner, "rollback", 1, 0);
        assert!(store.insert_auto_admission_receipt(&rollback).unwrap());
        store
            .test_connection()
            .execute_batch(
                "CREATE TRIGGER reject_auto_purge BEFORE DELETE ON auto_admission_receipts
                 WHEN OLD.request_id='rollback' BEGIN SELECT RAISE(ABORT,'fixture rollback'); END;",
            )
            .unwrap();
        assert!(store.purge_auto_refusals_under_pressure(101, 4).is_err());
        assert_eq!(
            store.auto_admission_receipt(owner, "rollback").unwrap(),
            Some(rollback)
        );
        let tombstones: usize = store
            .test_connection()
            .query_row(
                "SELECT count(*) FROM native_delegation_tombstones WHERE owner=?1 AND request_id='rollback'",
                [owner],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tombstones, 0, "rollback SQL : ni purge ni fausse clôture");
    }

    #[test]
    fn spec163_t048_reservation_precede_les_plafonds_et_rejoue_sans_double_compte() {
        let global = DelegationStore::open(Path::new(":memory:")).unwrap();
        let replay = auto_receipt163("owner-replay-t048", "replay", 1, 10);
        assert!(global.reserve_auto_admission_receipt(&replay, 10).unwrap());
        for index in 0..(MAX_ACTIVE_ADMISSIONS - 1) {
            let receipt = auto_receipt163(
                &format!("owner-global-{index:03}"),
                &format!("global-{index:03}"),
                1,
                11,
            );
            assert!(global.reserve_auto_admission_receipt(&receipt, 11).unwrap());
        }
        assert_eq!(
            global
                .admission_counts_for_owner("owner-replay-t048")
                .unwrap(),
            (MAX_ACTIVE_ADMISSIONS, 1, MAX_ACTIVE_ADMISSIONS),
        );
        assert!(
            !global.reserve_auto_admission_receipt(&replay, 12).unwrap(),
            "un rejeu exact est lu avant la garde globale pleine"
        );
        assert_eq!(
            global
                .reserve_auto_admission_receipt(
                    &auto_receipt163("owner-overflow", "overflow", 1, 12),
                    12,
                )
                .unwrap_err(),
            "task_limit"
        );

        let owner = DelegationStore::open(Path::new(":memory:")).unwrap();
        for index in 0..MAX_ACTIVE_ADMISSIONS_PER_OWNER {
            let receipt = auto_receipt163("owner-t048", &format!("owner-{index:02}"), 1, 20);
            assert!(owner.reserve_auto_admission_receipt(&receipt, 20).unwrap());
        }
        assert_eq!(
            owner
                .reserve_auto_admission_receipt(
                    &auto_receipt163("owner-t048", "owner-overflow", 1, 21),
                    21,
                )
                .unwrap_err(),
            "task_limit"
        );

        let retained = DelegationStore::open(Path::new(":memory:")).unwrap();
        for index in 0..MAX_RETAINED_TASKS {
            let receipt = auto_refusal163(
                &format!("owner-retained-{index:04}"),
                &format!("retained-{index:04}"),
                1,
                index as i64,
            );
            assert!(retained.insert_auto_admission_receipt(&receipt).unwrap());
        }
        let oldest = auto_refusal163("owner-retained-0000", "retained-0000", 1, 0);
        let fresh = auto_receipt163("owner-fresh-t048", "fresh", 1, 5_000);
        assert!(
            retained
                .reserve_auto_admission_receipt(&fresh, 5_000)
                .unwrap()
        );
        assert_eq!(
            retained
                .auto_admission_receipt(&oldest.owner, &oldest.request_id)
                .unwrap_err(),
            "request_expired",
            "la purge sous pression écrit le tombstone avant la nouvelle réservation"
        );
        assert_eq!(
            retained
                .admission_counts_for_owner("owner-fresh-t048")
                .unwrap()
                .2,
            MAX_RETAINED_TASKS
        );

        let rollback = DelegationStore::open(Path::new(":memory:")).unwrap();
        rollback
            .test_connection()
            .execute_batch(
                "CREATE TRIGGER reject_auto_reservation BEFORE INSERT ON auto_admission_receipts
                 WHEN NEW.request_id='rollback' BEGIN SELECT RAISE(ABORT,'fixture reservation rollback'); END;",
            )
            .unwrap();
        let receipt = auto_receipt163("owner-rollback-t048", "rollback", 1, 1);
        assert!(
            rollback
                .reserve_auto_admission_receipt(&receipt, 1)
                .is_err()
        );
        assert_eq!(
            rollback
                .auto_admission_receipt(&receipt.owner, &receipt.request_id)
                .unwrap(),
            None
        );
        assert_eq!(
            rollback.admission_counts_for_owner(&receipt.owner).unwrap(),
            (0, 0, 0),
            "erreur SQL : aucune clé ni compteur fantôme"
        );
    }

    #[test]
    fn spec163_t027_lot_epoch_borne_reutilise_les_cas_existants() {
        let empty = DelegationStore::open(Path::new(":memory:")).unwrap();
        assert_eq!(empty.next_auto_coordinator_epoch().unwrap(), 1);

        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let owner = "owner-epoch-t027";
        let started = auto_receipt163(owner, "started", 1, 10);
        let pending = auto_receipt163(owner, "pending", 1, 30);
        let chosen = auto_receipt163(owner, "chosen", 2, 20);
        let current = auto_receipt163(owner, "current", 3, 1);
        let terminal = auto_refusal163(owner, "terminal", 1, 0);
        for receipt in [&started, &pending, &chosen, &current, &terminal] {
            assert!(store.insert_auto_admission_receipt(receipt).unwrap());
        }
        store.start_auto_decision(&started, 11).unwrap();
        store
            .choose_auto_admission(
                &chosen.owner,
                &chosen.request_id,
                &chosen.canonical,
                "c01".into(),
                b"delegate-epoch-t027".to_vec(),
                AutoAdmissionMode::Fallback,
                None,
                21,
            )
            .unwrap();
        let batch = store.auto_admission_receipts_before_epoch(3, 128).unwrap();
        assert_eq!(
            batch
                .iter()
                .map(|receipt| receipt.request_id.as_str())
                .collect::<Vec<_>>(),
            vec!["started", "pending", "chosen"]
        );
        assert_eq!(
            store
                .auto_admission_receipts_before_epoch(3, 2)
                .unwrap()
                .len(),
            2
        );
        let coordinator_epoch = store.next_auto_coordinator_epoch().unwrap();
        assert_eq!(
            coordinator_epoch, 4,
            "strictement après les reçus existants"
        );
        for request_id in ["coordinator-a", "coordinator-b"] {
            assert!(
                store
                    .insert_auto_admission_receipt(&auto_receipt163(
                        owner,
                        request_id,
                        coordinator_epoch,
                        40,
                    ))
                    .unwrap()
            );
        }
        assert_eq!(
            store.next_auto_coordinator_epoch().unwrap(),
            5,
            "l'époque portée par tous les nouveaux reçus avance le prochain démarrage"
        );
        assert_eq!(
            store
                .auto_admission_receipts_before_epoch(5, 128)
                .unwrap()
                .iter()
                .map(|receipt| receipt.request_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "started",
                "pending",
                "chosen",
                "current",
                "coordinator-a",
                "coordinator-b"
            ]
        );
        assert_eq!(
            store
                .auto_admission_receipts_before_epoch(3, 0)
                .unwrap_err(),
            "auto_admission_invalid"
        );
    }

    #[test]
    fn spec163_t027_reouverture_disque_conserve_recu_choix_et_lien_task() {
        let dir = std::env::temp_dir().join(format!("bridget-auto163-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("admission.sqlite");
        let owner = "owner-reopen-t027";
        let receipt = auto_receipt163(owner, "reopen", 1, 10);

        let store = DelegationStore::open(&path).unwrap();
        assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
        drop(store);

        let store = DelegationStore::open(&path).unwrap();
        let pending = store
            .auto_admission_receipt(owner, "reopen")
            .unwrap()
            .unwrap();
        assert_eq!(pending.state, AutoAdmissionState::Pending);
        let _started = store.start_auto_decision(&pending, 11).unwrap();
        drop(store);

        let store = DelegationStore::open(&path).unwrap();
        let started = store
            .auto_admission_receipt(owner, "reopen")
            .unwrap()
            .unwrap();
        assert_eq!(started.state, AutoAdmissionState::DecisionStarted);
        let chosen = store
            .choose_auto_admission(
                owner,
                "reopen",
                &started.canonical,
                "c01".into(),
                b"delegate-reopen-t027".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                12,
            )
            .unwrap();
        drop(store);

        let store = DelegationStore::open(&path).unwrap();
        let chosen_after_reopen = store
            .auto_admission_receipt(owner, "reopen")
            .unwrap()
            .unwrap();
        assert_eq!(chosen_after_reopen, chosen);
        let mut task = legacy_task151();
        task.owner = owner.into();
        task.root_owner_agent_id = owner.into();
        task.task_id = "16300000-0000-4000-8000-000000000027".into();
        task.child = "16300000-0000-4000-8000-000000000028".into();
        task.mission = "16300000-0000-4000-8000-000000000029".into();
        store
            .test_connection()
            .execute_batch(
                "CREATE TRIGGER reject_auto_task BEFORE INSERT ON native_delegations
                 WHEN NEW.task_id='16300000-0000-4000-8000-000000000027'
                 BEGIN SELECT RAISE(ABORT,'fixture admission rollback'); END;",
            )
            .unwrap();
        assert!(
            store
                .admit_auto_task(
                    &chosen_after_reopen,
                    &task,
                    "reopen",
                    b"delegate-reopen-t027",
                    13,
                )
                .is_err()
        );
        assert_eq!(
            store
                .auto_admission_receipt(owner, "reopen")
                .unwrap()
                .unwrap(),
            chosen_after_reopen,
            "le rollback disque conserve le reçu chosen"
        );
        assert!(store.get(&task.task_id).unwrap().is_none());
        assert!(
            store.take_changes().is_empty(),
            "aucune projection n'est publiée avant le commit commun"
        );
        store
            .test_connection()
            .execute_batch("DROP TRIGGER reject_auto_task;")
            .unwrap();
        store
            .admit_auto_task(
                &chosen_after_reopen,
                &task,
                "reopen",
                b"delegate-reopen-t027",
                13,
            )
            .unwrap();
        drop(store);

        let store = DelegationStore::open(&path).unwrap();
        let admitted = store
            .auto_admission_receipt(owner, "reopen")
            .unwrap()
            .unwrap();
        assert_eq!(admitted.state, AutoAdmissionState::Admitted);
        assert_eq!(admitted.task_id.as_deref(), Some(task.task_id.as_str()));
        assert_eq!(admitted.canonical, receipt.canonical);
        assert_eq!(admitted.policy_digest, receipt.policy_digest);
        assert_eq!(
            store
                .by_agent_request(owner, "reopen", b"delegate-reopen-t027")
                .unwrap()
                .unwrap()
                .task_id,
            task.task_id
        );
        drop(store);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn spec163_projection_ne_publie_qu_apres_commit_externe() {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let task = legacy_task151();
        let tx = rusqlite::Transaction::new_unchecked(
            &store.conn,
            rusqlite::TransactionBehavior::Immediate,
        )
        .unwrap();
        let (stamped, _mutation) = store
            .insert_projected_in(&tx, &task, "request-163", b"canonical-163")
            .unwrap();
        drop(tx);
        assert!(store.get(&stamped.task_id).unwrap().is_none());
        assert!(store.take_changes().is_empty());

        let tx = rusqlite::Transaction::new_unchecked(
            &store.conn,
            rusqlite::TransactionBehavior::Immediate,
        )
        .unwrap();
        let (stamped, mutation) = store
            .insert_projected_in(&tx, &task, "request-163", b"canonical-163")
            .unwrap();
        tx.commit().unwrap();
        assert!(store.take_changes().is_empty());
        store.publish_projection(stamped.root_owner(), mutation);
        assert!(!store.take_changes().is_empty());
    }
}
