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
            ON native_delegation_past_missions(task_id);")?;
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
