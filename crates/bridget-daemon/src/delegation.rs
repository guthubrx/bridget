//! Saga native durable. Les identités et effets appartiennent au daemon.
use bridget_transport::protocol::{NativeDelegationRequest, ResolvedAgentDefinition, SpawnPosture};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) struct DelegationStore {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Task {
    pub task_id: String,
    pub owner: String,
    pub owner_instance: String,
    pub origin_owner_instance: String,
    #[serde(default)]
    pub parent_execution_id: Option<String>,
    pub request: NativeDelegationRequest,
    pub definition: ResolvedAgentDefinition,
    pub cwd: String,
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
}

impl Task {
    pub fn terminal(&self) -> bool {
        matches!(
            self.state.as_str(),
            "result_available" | "failed" | "cancelled"
        )
    }
}

impl DelegationStore {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("CREATE TABLE IF NOT EXISTS native_delegations (
            task_id TEXT PRIMARY KEY, owner_instance TEXT NOT NULL,
            request_id TEXT NOT NULL, canonical BLOB NOT NULL, payload TEXT NOT NULL,
            UNIQUE(owner_instance, request_id));
            CREATE INDEX IF NOT EXISTS native_delegations_state ON native_delegations(json_extract(payload,'$.state'));
            CREATE UNIQUE INDEX IF NOT EXISTS native_delegations_mission ON native_delegations(json_extract(payload,'$.mission'));
            CREATE UNIQUE INDEX IF NOT EXISTS native_delegations_owner_request ON native_delegations(json_extract(payload,'$.owner'),request_id);
            CREATE INDEX IF NOT EXISTS native_delegations_failed_agent ON native_delegations(json_extract(payload,'$.owner'),json_extract(payload,'$.state'));
            CREATE TABLE IF NOT EXISTS native_delegation_grants (
            owner_instance TEXT PRIMARY KEY, cwd TEXT NOT NULL, posture TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS native_delegation_revocations (agent_id TEXT PRIMARY KEY);")?;
        Ok(Self { conn })
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
            None => Ok(None),
        }
    }

    pub fn insert(&self, task: &Task, request_id: &str, canonical: &[u8]) -> Result<(), String> {
        let payload = serde_json::to_string(task).map_err(|error| error.to_string())?;
        self.conn.execute("INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
            params![task.task_id, task.owner_instance, request_id, canonical, payload]).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn save(&self, task: &Task) -> Result<(), String> {
        let payload = serde_json::to_string(task).map_err(|error| error.to_string())?;
        self.conn
            .execute(
                "UPDATE native_delegations SET payload=?2,owner_instance=?3 WHERE task_id=?1",
                params![task.task_id, payload, task.owner_instance],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
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

    pub fn pending(&self) -> Result<Vec<Task>, String> {
        let mut statement=self.conn.prepare("SELECT payload FROM native_delegations WHERE json_extract(payload,'$.state') NOT IN ('failed','cancelled','result_available') OR (json_extract(payload,'$.state')='result_available' AND (json_extract(payload,'$.result_sent')=0 OR coalesce(json_extract(payload,'$.cleanup_done'),0)=0)) OR (json_extract(payload,'$.state')='failed' AND coalesce(json_extract(payload,'$.cleanup_done'),0)=0) ORDER BY task_id").map_err(|error|error.to_string())?;
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
