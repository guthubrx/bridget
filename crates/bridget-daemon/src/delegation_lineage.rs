//! Projection des faits de la saga dans le même magasin transactionnel.
use super::{DelegationStore, RoundRecord, Task};
use bridget_transport::protocol::{
    HUMAN_LINEAGE_MAX_SEQ, HumanLineageError as E, NativeDelegationRequest,
};
use rusqlite::{OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone)]
pub(crate) struct ProjectionMutation {
    pub generation: String,
    pub seq: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    root: String,
    generation: String,
    seq: u64,
    created_at: i64,
    task_id: String,
}

/// Borne de rétention publiée aux lecteurs T3 (contrats 148 et 149).
pub(crate) const MAX_RETAINED_TASKS: usize = 4096;
/// Âge minimal d'un arbre purgeable hors pression de borne.
const RETENTION_SECS: i64 = 7 * 24 * 3600;
/// Durée de vie d'une pierre tombale de clé de rejeu (comme `purge_expired_sends`).
const TOMBSTONE_SECS: i64 = 30 * 24 * 3600;

/// Arbre soldé : chaque tâche est terminale et nettoyée, et toute remise due
/// (résultat ou avis d'échec) est accusée. L'annulation ne dispense pas des
/// anciennes rondes non accusées ; aucune garde d'inactivité ne doit rester.
fn tree_settled(tree: &[Task]) -> bool {
    tree.iter().all(|t| {
        t.terminal()
            && t.cleanup_done
            && t.idle_deadline_at.is_none()
            && t.unacked_rounds.is_empty()
            && (t.state == "cancelled" || t.delivery_acked)
    })
}

/// Dernière fin de l'arbre ; `updated_at` pour les tâches 148 sans `completed_at`.
fn tree_age(tree: &[Task]) -> i64 {
    tree.iter()
        .map(|t| t.completed_at.unwrap_or(t.updated_at))
        .max()
        .unwrap_or(0)
}

impl Task {
    pub fn root_owner(&self) -> &str {
        if self.root_owner_agent_id.is_empty() {
            &self.owner
        } else {
            &self.root_owner_agent_id
        }
    }
}

pub(crate) fn task_entry(task: &Task, journal_available: bool) -> Result<Value, E> {
    let NativeDelegationRequest::Delegate {
        agent_type,
        model,
        effort,
        posture,
        task: instruction,
        ..
    } = &task.request
    else {
        return Err(E::StoreUnavailable);
    };
    if !matches!(
        task.state.as_str(),
        "queued"
            | "starting"
            | "mission_pending"
            | "working"
            | "waiting_for_children"
            | "cancelling"
            | "result_available"
            | "failed"
            | "cancelled"
    ) {
        return Err(E::StoreUnavailable);
    }
    let title: String = instruction
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .chars()
        .filter(|c| !bridget_core::is_disallowed_control(*c))
        .take(256)
        .collect();
    let error = task.projected_error().map(|error| {
        error
            .chars()
            .filter(|c| !bridget_core::is_disallowed_control(*c))
            .take(1024)
            .collect::<String>()
    });
    Ok(
        json!({"task_id":task.task_id,"parent_task_id":task.parent_task_id,
        "parent_agent_id":task.owner,"child_agent_id":task.child,"child_instance_id":task.child_instance,
        "created_at":task.created_at,"updated_at":task.updated_at.max(task.created_at),
        "started_at":task.started_at,"completed_at":task.completed_at,
        "agent_type":agent_type,"execution_protocol":task.definition.protocol,
        "model":model,"effort":effort,"cwd":task.cwd,"posture":task.effective_posture.or(*posture).unwrap_or(bridget_transport::protocol::SpawnPosture::Discovery),
        "title":title,"status":task.state,"error":error,
        "result_available":task.state=="result_available","journal_available":journal_available}),
    )
}

fn visible_changed(old: &Task, new: &Task) -> bool {
    old.state != new.state
        || old.child_instance != new.child_instance
        || old.projected_error() != new.projected_error()
        || (new.state == "result_available" && old.result != new.result)
}

fn stamp(mut task: Task, old: Option<&Task>) -> Task {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs().min(i64::MAX as u64) as i64);
    task.updated_at = if old.is_none_or(|old| visible_changed(old, &task)) {
        now.max(task.created_at)
    } else {
        old.unwrap().updated_at
    };
    task.started_at = old
        .and_then(|old| old.started_at)
        .or(task.started_at)
        .or_else(|| task.mission_deadline_at.map(|_| now));
    task.completed_at = old
        .and_then(|old| old.completed_at)
        .or(task.completed_at)
        .or_else(|| task.terminal().then_some(now));
    task
}

fn increment(tx: &Transaction<'_>) -> rusqlite::Result<ProjectionMutation> {
    let (mut generation, seq): (String, u64) = tx.query_row(
        "SELECT generation,seq FROM native_delegation_projection_meta WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let seq = if seq >= HUMAN_LINEAGE_MAX_SEQ {
        generation = uuid::Uuid::new_v4().to_string();
        1
    } else {
        seq + 1
    };
    tx.execute(
        "UPDATE native_delegation_projection_meta SET generation=?1,seq=?2 WHERE singleton=1",
        params![generation, seq],
    )?;
    Ok(ProjectionMutation { generation, seq })
}

impl DelegationStore {
    pub(super) fn initialize_lineage(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch("CREATE TABLE IF NOT EXISTS native_delegation_projection_meta(singleton INTEGER PRIMARY KEY CHECK(singleton=1),generation TEXT NOT NULL,seq INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS native_delegation_cancel_receipts(root TEXT NOT NULL,request_id TEXT NOT NULL,task_id TEXT NOT NULL,receipt TEXT NOT NULL,PRIMARY KEY(root,request_id));
            CREATE TABLE IF NOT EXISTS native_delegation_tombstones(owner TEXT NOT NULL,request_id TEXT NOT NULL,purged_at INTEGER NOT NULL,PRIMARY KEY(owner,request_id));")?;
        self.conn.execute("INSERT OR IGNORE INTO native_delegation_projection_meta(singleton,generation,seq) VALUES(1,?1,0)", [uuid::Uuid::new_v4().to_string()])?;
        // Les anciens liens sont dérivés seulement des identités native148 durables.
        let records = self.tasks().map_err(|_| rusqlite::Error::InvalidQuery)?;
        let children: BTreeMap<_, _> = records
            .iter()
            .map(|task| (task.child.as_str(), task))
            .collect();
        let tx = self.conn.unchecked_transaction()?;
        for record in &records {
            if !record.root_owner_agent_id.is_empty() {
                continue;
            }
            let mut task = record.clone();
            let mut current = record;
            // La première mission compte pour un ; sept ancêtres donnent
            // une chaîne de huit missions, huit ancêtres sont refusés.
            let mut visited = HashSet::from([record.task_id.clone()]);
            while let Some(parent) = children.get(current.owner.as_str()).copied() {
                if parent.child_instance.as_deref() != Some(current.origin_owner_instance.as_str())
                {
                    break;
                }
                if !visited.insert(parent.task_id.clone()) || visited.len() > 8 {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                if task.parent_task_id.is_none() {
                    task.parent_task_id = Some(parent.task_id.clone());
                }
                current = parent;
            }
            task.root_owner_agent_id = current.owner.clone();
            task.updated_at = task.created_at;
            let payload =
                serde_json::to_string(&task).map_err(|_| rusqlite::Error::InvalidQuery)?;
            tx.execute(
                "UPDATE native_delegations SET payload=?2 WHERE task_id=?1",
                params![task.task_id, payload],
            )?;
        }
        tx.execute_batch("CREATE UNIQUE INDEX IF NOT EXISTS native_delegations_child ON native_delegations(json_extract(payload,'$.child'));
            CREATE INDEX IF NOT EXISTS native_delegations_root_page ON native_delegations(json_extract(payload,'$.root_owner_agent_id'),json_extract(payload,'$.created_at'),task_id);
            CREATE INDEX IF NOT EXISTS native_delegations_parent ON native_delegations(json_extract(payload,'$.parent_task_id'));")?;
        tx.commit()
    }

    fn committed(&self, root: &str, mutation: ProjectionMutation) {
        self.changes.borrow_mut().insert(root.into(), mutation);
        let _ = self.notify.try_send(());
    }

    pub fn change_receiver(&self) -> Option<std::sync::mpsc::Receiver<()>> {
        self.receiver.borrow_mut().take()
    }
    pub fn take_changes(&self) -> BTreeMap<String, ProjectionMutation> {
        std::mem::take(&mut *self.changes.borrow_mut())
    }

    pub fn projection_meta(&self) -> Result<ProjectionMutation, E> {
        let meta = self
            .conn
            .query_row(
                "SELECT generation,seq FROM native_delegation_projection_meta WHERE singleton=1",
                [],
                |r| {
                    Ok(ProjectionMutation {
                        generation: r.get(0)?,
                        seq: r.get(1)?,
                    })
                },
            )
            .map_err(|_| E::StoreUnavailable)?;
        if meta.seq > HUMAN_LINEAGE_MAX_SEQ
            || crate::threads::canonical_uuid(&meta.generation).as_deref()
                != Some(meta.generation.as_str())
        {
            return Err(E::StoreUnavailable);
        }
        Ok(meta)
    }

    pub fn for_child(&self, child: &str) -> Result<Option<Task>, String> {
        let payload: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM native_delegations WHERE json_extract(payload,'$.child')=?1",
                [child],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        payload
            .map(|p| serde_json::from_str(&p).map_err(|e| e.to_string()))
            .transpose()
    }

    pub fn parent_lineage(&self, parent: &Task) -> Result<(String, Option<String>), String> {
        let mut current = parent.clone();
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(current.task_id.clone()) || visited.len() >= 8 {
                return Err("task_depth_limit".into());
            }
            let Some(id) = &current.parent_task_id else {
                break;
            };
            current = self.get(id)?.ok_or("parent_task_unavailable")?;
            if current.root_owner() != parent.root_owner() {
                return Err("parent_task_unavailable".into());
            }
        }
        Ok((parent.root_owner().into(), Some(parent.task_id.clone())))
    }

    pub fn descendants(&self, task_id: &str) -> Result<Vec<Task>, String> {
        let mut statement=self.conn.prepare("WITH RECURSIVE descendants(id,payload,depth) AS (
            SELECT task_id,payload,1 FROM native_delegations WHERE json_extract(payload,'$.parent_task_id')=?1
            UNION ALL SELECT d.task_id,d.payload,p.depth+1 FROM native_delegations d JOIN descendants p ON json_extract(d.payload,'$.parent_task_id')=p.id WHERE p.depth<8)
            SELECT payload FROM descendants ORDER BY depth, id LIMIT 4097").map_err(|e|e.to_string())?;
        let rows = statement
            .query_map([task_id], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let result: Vec<Task> = rows
            .map(|r| {
                r.map_err(|e| e.to_string())
                    .and_then(|p| serde_json::from_str(&p).map_err(|e| e.to_string()))
            })
            .collect::<Result<_, _>>()?;
        if result.len() > 4096 {
            return Err("descendant_limit".into());
        }
        Ok(result)
    }

    /// Arbres soldés (racine sans parent + descendants), racine en tête.
    /// Complexité : O(r · d) — r racines terminales, d taille d'arbre ; r·d ≤ lignes conservées (≤ 4096).
    fn settled_trees(&self) -> Result<Vec<Vec<Task>>, String> {
        let mut statement=self.conn.prepare("SELECT payload FROM native_delegations WHERE json_extract(payload,'$.parent_task_id') IS NULL AND json_extract(payload,'$.state') IN ('result_available','failed','cancelled')").map_err(|e|e.to_string())?;
        let roots: Vec<Task> = statement
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .map(|r| {
                r.map_err(|e| e.to_string())
                    .and_then(|p| serde_json::from_str(&p).map_err(|e| e.to_string()))
            })
            .collect::<Result<_, _>>()?;
        // Une racine qui n'est le parent de personne est un arbre d'une tâche : pas de requête récursive.
        let parents:HashSet<String>=self.conn.prepare("SELECT DISTINCT json_extract(payload,'$.parent_task_id') FROM native_delegations WHERE json_extract(payload,'$.parent_task_id') IS NOT NULL").map_err(|e|e.to_string())?
            .query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<_,_>>().map_err(|e|e.to_string())?;
        let mut trees = Vec::new();
        for root in roots {
            let mut tree = if parents.contains(&root.task_id) {
                self.descendants(&root.task_id)?
            } else {
                Vec::new()
            };
            tree.insert(0, root);
            if tree_settled(&tree) {
                trees.push(tree)
            }
        }
        Ok(trees)
    }

    /// Purge les arbres soldés du plus ancien au plus récent, `max_trees` au plus par passage
    /// (128 à l'entretien ; sans limite à l'admission, où la borne de rétention suffit).
    /// Hors pression (`bound` absent), seuls les arbres finis depuis plus de 7 jours
    /// partent ; sous pression (`bound` = borne de rétention), l'âge est ignoré et la
    /// purge s'arrête dès que le total repasse sous la borne. `releasable` est le contrôle de vivacité des sous-agents, refait par
    /// l'appelant sous le verrou du daemon : il retire aussi les enfants de la flotte
    /// et rend `false` pour conserver l'arbre. Rend le nombre d'arbres supprimés.
    /// Complexité : O(n log n) — n ≤ 4096 lignes conservées ; une transaction par arbre.
    pub fn purge_trees(
        &self,
        now: i64,
        bound: Option<usize>,
        max_trees: usize,
        mut releasable: impl FnMut(&[Task]) -> bool,
    ) -> Result<usize, String> {
        let mut trees: Vec<(i64, Vec<Task>)> = self
            .settled_trees()?
            .into_iter()
            .map(|t| (tree_age(&t), t))
            .collect();
        trees.sort_by(|a, b| (a.0, &a.1[0].task_id).cmp(&(b.0, &b.1[0].task_id)));
        let mut total: usize = self
            .conn
            .query_row("SELECT count(*) FROM native_delegations", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let mut purged = 0;
        for (age, tree) in trees {
            if purged >= max_trees || bound.is_some_and(|b| total < b) {
                break;
            }
            if bound.is_none() && now.saturating_sub(age) <= RETENTION_SECS {
                break;
            }
            if !releasable(&tree) {
                continue;
            }
            self.delete_tree(&tree, now)?;
            total = total.saturating_sub(tree.len());
            purged += 1;
        }
        Ok(purged)
    }

    /// Libère les clés de rejeu purgées depuis 30 jours ou plus.
    /// Complexité : O(T) — balayage des T pierres (pas d'index sur `purged_at`) ; appelée
    /// une fois par passage d'entretien horaire, T borné par les purges des 30 derniers jours.
    pub fn purge_expired_tombstones(&self, now: i64) -> Result<usize, String> {
        self.conn
            .execute(
                "DELETE FROM native_delegation_tombstones WHERE purged_at<=?1",
                [now.saturating_sub(TOMBSTONE_SECS)],
            )
            .map_err(|e| e.to_string())
    }

    /// Une clé dont la tâche a été purgée se rejoue en refus, jamais en nouvelle tâche.
    pub(super) fn tombstoned(&self, owner: &str, request_id: &str) -> Result<bool, String> {
        self.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_delegation_tombstones WHERE owner=?1 AND request_id=?2)",params![owner,request_id],|r|r.get(0)).map_err(|e|e.to_string())
    }

    /// Suppression atomique d'un arbre : pierres tombales, reçus d'annulation, lignes,
    /// nouvelle génération. Le propriétaire racine est signalé après le commit.
    fn delete_tree(&self, tree: &[Task], now: i64) -> Result<(), String> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        for task in tree {
            tx.execute("INSERT OR REPLACE INTO native_delegation_tombstones(owner,request_id,purged_at) SELECT json_extract(payload,'$.owner'),request_id,?2 FROM native_delegations WHERE task_id=?1",params![task.task_id,now]).map_err(|e|e.to_string())?;
            tx.execute(
                "DELETE FROM native_child_requests WHERE task_id=?1",
                [&task.task_id],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "DELETE FROM native_delegation_cancel_receipts WHERE task_id=?1",
                [&task.task_id],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "DELETE FROM native_delegation_past_missions WHERE task_id=?1",
                [&task.task_id],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "DELETE FROM native_delegations WHERE task_id=?1",
                [&task.task_id],
            )
            .map_err(|e| e.to_string())?;
        }
        let mutation = ProjectionMutation {
            generation: uuid::Uuid::new_v4().to_string(),
            seq: 1,
        };
        tx.execute(
            "UPDATE native_delegation_projection_meta SET generation=?1,seq=?2 WHERE singleton=1",
            params![mutation.generation, mutation.seq],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        self.wait_epoch.bump();
        self.committed(tree[0].root_owner(), mutation);
        Ok(())
    }

    pub(super) fn insert_projected(
        &self,
        task: &Task,
        request_id: &str,
        canonical: &[u8],
    ) -> Result<(), String> {
        let mut task = stamp(task.clone(), None);
        if task.root_owner_agent_id.is_empty() {
            task.root_owner_agent_id = task.owner.clone();
        }
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let payload = serde_json::to_string(&task).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",params![task.task_id,task.owner_instance,request_id,canonical,payload]).map_err(|e|e.to_string())?;
        let mutation = increment(&tx).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        self.wait_epoch.bump();
        self.committed(task.root_owner(), mutation);
        Ok(())
    }

    pub(super) fn save_projected(&self, task: &Task) -> Result<(), String> {
        let old = self.get(&task.task_id)?.ok_or("task_unavailable")?;
        let mut task = stamp(task.clone(), Some(&old));
        // Ces références ne suivent jamais une instance runtime ou un retry.
        task.root_owner_agent_id = old.root_owner_agent_id.clone();
        task.parent_task_id = old.parent_task_id.clone();
        // La réservation est décidée à l'admission, jamais par une sauvegarde stale.
        task.guarded = old.guarded;
        task.guard_released |= old.guard_released;
        if task.guard_released {
            task.idle_deadline_at = None;
        }
        // Seule acknowledge_tasks pose ce fait ; une copie stale ne le remplace pas.
        task.acknowledged_at = old.acknowledged_at;
        let changed = visible_changed(&old, &task)
            || old.started_at != task.started_at
            || old.completed_at != task.completed_at;
        let payload = serde_json::to_string(&task).map_err(|e| e.to_string())?;
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE native_delegations SET payload=?2,owner_instance=?3 WHERE task_id=?1",
            params![task.task_id, payload, task.owner_instance],
        )
        .map_err(|e| e.to_string())?;
        let mutation = if changed {
            Some(increment(&tx).map_err(|e| e.to_string())?)
        } else {
            None
        };
        tx.commit().map_err(|e| e.to_string())?;
        self.wait_epoch.bump();
        if let Some(mutation) = mutation {
            self.committed(task.root_owner(), mutation)
        }
        Ok(())
    }

    /// O(log n + log h + B + r), n tâches, h missions passées, B payload, r <= 7.
    /// La lecture CAS, l'historique et la projection partagent un seul commit.
    pub fn begin_round(
        &self,
        task_id: &str,
        expected_round: u32,
        clock: impl FnOnce() -> i64,
        record: RoundRecord,
        mission: &str,
        round_body: &str,
    ) -> Result<Task, String> {
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let payload: String = tx
            .query_row(
                "SELECT payload FROM native_delegations WHERE task_id=?1",
                [task_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .ok_or("task_unavailable")?;
        let old: Task = serde_json::from_str(&payload).map_err(|error| error.to_string())?;
        if old.round_no() != expected_round {
            return Err("round_in_progress".into());
        }
        if !old.guarded {
            return Err("keep_alive_not_enabled".into());
        }
        if old.guard_released || matches!(old.state.as_str(), "failed" | "cancelled") {
            return Err("keep_alive_ended".into());
        }
        if old.state != "result_available" {
            return Err("round_in_progress".into());
        }
        if !old.result_sent {
            return Err("result_delivery_pending".into());
        }
        let idle_deadline = old.idle_deadline_at.ok_or("keep_alive_ended")?;
        if expected_round >= 8 || old.rounds.len() >= 7 {
            return Err("round_limit".into());
        }
        if record.round != expected_round + 1 {
            return Err("invalid_request".into());
        }
        let mut next = old.clone();
        next.round = expected_round + 1;
        next.mission = mission.into();
        next.round_body = Some(round_body.into());
        next.rounds.push(record);
        next.workspace_report_mission = None;
        if let Some(workspace) = &mut next.workspace {
            workspace.head_commit = None;
            workspace.branch_commit = None;
            workspace.head_on_branch = None;
            workspace.commit_count = None;
            workspace.commits.clear();
            workspace.truncated = false;
            workspace.dirty = None;
        }
        if !old.delivery_acked && !next.unacked_rounds.contains(&expected_round) {
            next.unacked_rounds.push(expected_round);
        }
        next.result = None;
        next.result_sent = false;
        next.delivery_acked = false;
        next.acknowledged_at = None;
        next.failure_sent = false;
        next.error = None;
        next.mission_deadline_at = None;
        next.mission_issued_at = None;
        next.idle_deadline_at = None;
        next.cleanup_done = false;
        next.state = "mission_pending".into();
        next = stamp(next, Some(&old));
        // stamp conserve les fins ordinaires ; une nouvelle ronde en ouvre une autre.
        next.completed_at = None;
        let payload = serde_json::to_string(&next).map_err(|error| error.to_string())?;
        // Dernière lecture d'heure avant les écritures, jamais l'heure d'admission.
        if clock() >= idle_deadline {
            return Err("keep_alive_ended".into());
        }
        tx.execute(
            "INSERT INTO native_delegation_past_missions(mission,task_id) VALUES(?1,?2)",
            params![old.mission, task_id],
        )
        .map_err(|error| error.to_string())?;
        let changed = tx.execute(
            "UPDATE native_delegations SET payload=?2 WHERE task_id=?1 AND max(coalesce(json_extract(payload,'$.round'),0),1)=?3 AND json_extract(payload,'$.state')='result_available'",
            params![task_id, payload, expected_round],
        ).map_err(|error| error.to_string())?;
        if changed != 1 {
            return Err("round_in_progress".into());
        }
        let mutation = increment(&tx).map_err(|error| error.to_string())?;
        tx.commit().map_err(|error| error.to_string())?;
        self.wait_epoch.bump();
        self.committed(next.root_owner(), mutation);
        Ok(next)
    }

    pub fn lineage_page(
        &self,
        root: &str,
        limit: u32,
        cursor: Option<&str>,
        mut journal: impl FnMut(&Task) -> bool,
    ) -> Result<Value, E> {
        if !(1..=100).contains(&limit) {
            return Err(E::InvalidRequest);
        }
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|_| E::StoreUnavailable)?;
        let meta = self.projection_meta()?;
        let after = match cursor {
            Some(c) if c.len() <= 2048 => {
                let c: Cursor = serde_json::from_str(c).map_err(|_| E::InvalidRequest)?;
                if c.root != root {
                    return Err(E::InvalidRequest);
                }
                if c.generation != meta.generation || c.seq != meta.seq {
                    return Err(E::SnapshotChanged);
                }
                if crate::threads::canonical_uuid(&c.task_id).as_deref() != Some(c.task_id.as_str())
                {
                    return Err(E::InvalidRequest);
                }
                Some(c)
            }
            Some(_) => return Err(E::InvalidRequest),
            None => None,
        };
        let mut stmt=tx.prepare("SELECT payload FROM native_delegations WHERE json_extract(payload,'$.root_owner_agent_id')=?1 AND (json_extract(payload,'$.created_at'),task_id)>(?2,?3) ORDER BY json_extract(payload,'$.created_at'),task_id LIMIT ?4").map_err(|_|E::StoreUnavailable)?;
        let rows = stmt
            .query_map(
                params![
                    root,
                    after.as_ref().map_or(i64::MIN, |c| c.created_at),
                    after.as_ref().map_or("", |c| c.task_id.as_str()),
                    limit + 1
                ],
                |r| r.get::<_, String>(0),
            )
            .map_err(|_| E::StoreUnavailable)?;
        let mut entries = Vec::new();
        let mut bytes = 1024;
        let mut last = None;
        let mut more = false;
        for row in rows {
            let task: Task = serde_json::from_str(&row.map_err(|_| E::StoreUnavailable)?)
                .map_err(|_| E::StoreUnavailable)?;
            let entry = task_entry(&task, journal(&task))?;
            let size = serde_json::to_vec(&entry)
                .map_err(|_| E::StoreUnavailable)?
                .len();
            if entries.len() >= limit as usize || bytes + size > 128 * 1024 - 4096 {
                more = true;
                break;
            }
            bytes += size;
            last = Some((task.created_at, task.task_id));
            entries.push(entry);
        }
        let next_cursor = if more {
            let (created_at, task_id) = last.ok_or(E::ResourceLimit)?;
            Some(
                serde_json::to_string(&Cursor {
                    root: root.into(),
                    generation: meta.generation.clone(),
                    seq: meta.seq,
                    created_at,
                    task_id,
                })
                .map_err(|_| E::StoreUnavailable)?,
            )
        } else {
            None
        };
        drop(stmt);
        tx.commit().map_err(|_| E::StoreUnavailable)?;
        Ok(
            json!({"version":1,"status":"ok","generation":meta.generation,"seq":meta.seq,"root_owner_agent_id":root,"tasks":entries,"next_cursor":next_cursor}),
        )
    }

    pub fn lineage_show(
        &self,
        root: &str,
        task_id: &str,
        offset: u32,
        limit: u32,
        journal_available: bool,
    ) -> Result<Value, E> {
        if !(1..=16384).contains(&limit) {
            return Err(E::InvalidRequest);
        }
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|_| E::StoreUnavailable)?;
        let meta = self.projection_meta()?;
        let task = self
            .get(task_id)
            .map_err(|_| E::StoreUnavailable)?
            .filter(|t| t.root_owner() == root)
            .ok_or(E::TaskUnavailable)?;
        let result = if task.state == "result_available" {
            task.result.as_deref()
        } else {
            None
        };
        let total = result.map_or(0, str::len);
        if total > 256 * 1024 {
            return Err(E::StoreUnavailable);
        }
        let start = offset as usize;
        if start > total
            || result.is_some_and(|s| !s.is_char_boundary(start))
            || (result.is_none() && start != 0)
        {
            return Err(E::ResultOffsetInvalid);
        }
        let mut end = (start + limit as usize).min(total);
        if let Some(s) = result {
            while !s.is_char_boundary(end) {
                end -= 1
            }
        }
        if end == start && start < total {
            return Err(E::ResultOffsetInvalid);
        }
        let entry = task_entry(&task, journal_available)?;
        let response = json!({"version":1,"status":"ok","generation":meta.generation,"seq":meta.seq,"root_owner_agent_id":root,"task":entry,"result":result.map(|s|&s[start..end]),"result_offset":offset,"result_next_offset":if end<total {Some(end)}else{None},"result_total_bytes":total});
        tx.commit().map_err(|_| E::StoreUnavailable)?;
        Ok(response)
    }

    pub fn human_cancel(
        &self,
        root: &str,
        task_id: &str,
        request_id: &str,
    ) -> Result<(Value, bool), E> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|_| E::StoreUnavailable)?;
        let saved:Option<(String,String)>=tx.query_row("SELECT task_id,receipt FROM native_delegation_cancel_receipts WHERE root=?1 AND request_id=?2",params![root,request_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|_|E::StoreUnavailable)?;
        if let Some((id, receipt)) = saved {
            if id != task_id {
                return Err(E::EnvelopeMismatch);
            }
            return Ok((
                serde_json::from_str(&receipt).map_err(|_| E::StoreUnavailable)?,
                false,
            ));
        }
        let task = self
            .get(task_id)
            .map_err(|_| E::StoreUnavailable)?
            .filter(|t| t.root_owner() == root)
            .ok_or(E::TaskUnavailable)?;
        let count: u32 = tx
            .query_row(
                "SELECT count(*) FROM native_delegation_cancel_receipts",
                [],
                |r| r.get(0),
            )
            .map_err(|_| E::StoreUnavailable)?;
        if count >= 4096 {
            return Err(E::ResourceLimit);
        }
        let drives = !task.terminal() || (task.guarded && task.state == "result_available");
        let mut changed = task.clone();
        if drives {
            changed.guard_released = true;
            changed.idle_deadline_at = None;
        }
        let state_changed = !task.terminal() && task.state != "cancelling";
        if state_changed {
            changed.state = "cancelling".into();
            changed = stamp(changed, Some(&task));
        }
        if state_changed
            || changed.guard_released != task.guard_released
            || changed.idle_deadline_at != task.idle_deadline_at
        {
            tx.execute(
                "UPDATE native_delegations SET payload=?2 WHERE task_id=?1",
                params![
                    task_id,
                    serde_json::to_string(&changed).map_err(|_| E::StoreUnavailable)?
                ],
            )
            .map_err(|_| E::StoreUnavailable)?;
        }
        let mutation = if state_changed {
            Some(increment(&tx).map_err(|_| E::StoreUnavailable)?)
        } else {
            None
        };
        let receipt = json!({"version":1,"task_id":task_id,"status":changed.state});
        tx.execute("INSERT INTO native_delegation_cancel_receipts(root,request_id,task_id,receipt) VALUES(?1,?2,?3,?4)",params![root,request_id,task_id,receipt.to_string()]).map_err(|_|E::StoreUnavailable)?;
        tx.commit().map_err(|_| E::StoreUnavailable)?;
        self.wait_epoch.bump();
        if let Some(m) = mutation {
            self.committed(root, m)
        }
        Ok((receipt, drives))
    }
}

#[cfg(test)]
#[path = "delegation_lineage_tests.rs"]
mod delegation_lineage_tests;
