//! Saga native durable. Les identités et effets appartiennent au daemon.
use bridget_transport::protocol::{NativeDelegationRequest, ResolvedAgentDefinition, SpawnPosture};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

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
    pub child_requests: Option<String>,
    #[serde(default)]
    pub permission_snapshot: Option<bridget_transport::protocol::NativePermissionSnapshot>,
    #[serde(default)]
    pub effective_posture: Option<SpawnPosture>,
    pub definition: ResolvedAgentDefinition,
    pub cwd: String,
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

    #[cfg(test)]
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
        let tx = rusqlite::Transaction::new_unchecked(
            &conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
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
CREATE INDEX IF NOT EXISTS native_child_requests_inbox ON native_child_requests(inbox_settled, state);")?;
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
        self.conn
            .execute(
                "INSERT OR IGNORE INTO native_delegation_refusals(owner,request_id,canonical,code,refused_at)
                 SELECT ?1,?2,?3,?4,?5 WHERE NOT EXISTS (
                     SELECT 1 FROM native_delegations WHERE json_extract(payload,'$.owner')=?1 AND request_id=?2
                 )",
                params![owner, request_id, canonical, code, now],
            )
            .map(|changed| changed != 0)
            .map_err(|error| error.to_string())
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

    /// Comptages d'admission : (tâches actives, actives de l'instance, tâches conservées).
    /// Complexité : O(log n + a) pour les places (a ≤ 128, index d'état étendu),
    /// O(n) pour le total (balayage de l'index, n ≤ 4096) ; aucun payload n'est lu.
    pub fn admission_counts(&self, instance: &str) -> Result<(usize, usize, usize), String> {
        const HOLDS_SLOT: &str = "json_extract(payload,'$.state') IN ('queued','starting','mission_pending','working','waiting_for_children','cancelling') OR (json_extract(payload,'$.state') IN ('result_available','failed','cancelled') AND coalesce(json_extract(payload,'$.guarded'),0)=1 AND coalesce(json_extract(payload,'$.cleanup_done'),0)=0)";
        let count = |sql: String, args: &[&dyn rusqlite::ToSql]| -> Result<usize, String> {
            self.conn.query_row(&sql, args, |row| row.get(0)).map_err(|error| error.to_string())
        };
        Ok((
            // Évite le parcours des terminaux par l'index owner_instance seul.
            count(format!("SELECT count(*) FROM native_delegations INDEXED BY native_delegations_state WHERE ({HOLDS_SLOT})"), &[])?,
            count(format!("SELECT count(*) FROM native_delegations INDEXED BY native_delegations_state WHERE owner_instance=?1 AND ({HOLDS_SLOT})"), &[&instance])?,
            count("SELECT count(*) FROM native_delegations".into(), &[])?,
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
        payload.map(|payload| serde_json::from_str(&payload).map_err(|error| error.to_string())).transpose()
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
            payload.as_object_mut().unwrap().remove("workspace_report_mission");
            if version >= 152 {
                payload["launch_issued_at"] = serde_json::json!(110);
                payload["restart_resumes"] = serde_json::json!(1);
            }
            if version >= 153 {
                payload["round"] = serde_json::json!(2);
                payload["guarded"] = serde_json::json!(true);
                payload["request"]["keep_alive_secs"] = serde_json::json!(600);
                payload["request"]["brief"] = serde_json::json!({"objective":"objectif","summary":"résultat"});
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
            assert_eq!(serde_json::to_value(saved.workspace).unwrap(), serde_json::to_value(&task.workspace).unwrap());
            assert_eq!(saved.workspace_report_mission, task.workspace_report_mission);
            assert_eq!(store.lineage_page("parent", 100, None, |_| false).unwrap(), before);
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
}
