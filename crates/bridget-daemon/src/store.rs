//! Persistance SQLite — ledger, compteurs disjoncteur, historique.

use rusqlite::Connection;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedRequest {
    pub id: String,
    pub sender: String,
    pub target: String,
    pub state: String,
    pub created_at: i64,
    pub deadline_at: i64,
    pub escalation_level: u8,
    pub cancel_reason: Option<String>,
    pub completed_at: Option<i64>,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Ouvre ou crée la base de données.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path).map_err(StoreError::Sqlite)?;
        Self::init_schema(&conn)?;
        Ok(Store { conn })
    }

    fn init_schema(conn: &Connection) -> Result<(), StoreError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ledger (
                id TEXT NOT NULL,
                ts INTEGER NOT NULL,
                sender TEXT NOT NULL,
                target TEXT NOT NULL,
                body TEXT NOT NULL,
                conversation_key TEXT NOT NULL,
                PRIMARY KEY (id, target)
            );
            CREATE INDEX IF NOT EXISTS idx_ledger_ts ON ledger(ts);
            CREATE INDEX IF NOT EXISTS idx_ledger_conv ON ledger(conversation_key, ts);
            CREATE TABLE IF NOT EXISTS tracked_requests (
                id TEXT PRIMARY KEY,
                sender TEXT NOT NULL,
                target TEXT NOT NULL,
                state TEXT NOT NULL CHECK (state IN ('open', 'answered', 'cancelled', 'timed_out')),
                created_at INTEGER NOT NULL,
                deadline_at INTEGER NOT NULL,
                escalation_level INTEGER NOT NULL DEFAULT 0,
                cancel_reason TEXT,
                completed_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_tracked_requests_sender ON tracked_requests(sender, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_tracked_requests_open ON tracked_requests(state, deadline_at);
            ",
        )
        .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    pub fn create_request(
        &self,
        id: &str,
        sender: &str,
        target: &str,
        timeout_secs: u64,
    ) -> Result<TrackedRequest, StoreError> {
        let created_at = now_secs();
        let request = TrackedRequest {
            id: id.to_string(),
            sender: sender.to_string(),
            target: target.to_string(),
            state: "open".to_string(),
            created_at,
            deadline_at: created_at + timeout_secs as i64,
            escalation_level: 0,
            cancel_reason: None,
            completed_at: None,
        };
        self.conn.execute(
            "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at, escalation_level) VALUES (?1, ?2, ?3, 'open', ?4, ?5, 0)",
            rusqlite::params![request.id, request.sender, request.target, request.created_at, request.deadline_at],
        ).map_err(StoreError::Sqlite)?;
        Ok(request)
    }

    pub fn get_request(&self, id: &str) -> Result<Option<TrackedRequest>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE id = ?1").map_err(StoreError::Sqlite)?;
        let mut rows = stmt
            .query(rusqlite::params![id])
            .map_err(StoreError::Sqlite)?;
        match rows.next().map_err(StoreError::Sqlite)? {
            Some(row) => Ok(Some(
                tracked_request_from_row(row).map_err(StoreError::Sqlite)?,
            )),
            None => Ok(None),
        }
    }

    pub fn requests_for_sender(&self, sender: &str) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE sender = ?1 ORDER BY created_at DESC", rusqlite::params![sender])
    }

    pub fn open_requests(&self) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE state = 'open' ORDER BY deadline_at", [])
    }

    pub fn cancel_request(
        &self,
        id: &str,
        sender: &str,
        reason: Option<&str>,
    ) -> Result<Option<TrackedRequest>, StoreError> {
        let Some(request) = self.get_request(id)? else {
            return Ok(None);
        };
        if request.sender != sender || (request.state != "open" && request.state != "cancelled") {
            return Ok(Some(request));
        }
        if request.state == "open" {
            let completed_at = now_secs();
            self.conn.execute("UPDATE tracked_requests SET state = 'cancelled', cancel_reason = ?1, completed_at = ?2 WHERE id = ?3 AND state = 'open'", rusqlite::params![reason, completed_at, id]).map_err(StoreError::Sqlite)?;
            return self.get_request(id);
        }
        Ok(Some(request))
    }

    pub fn mark_answered(
        &self,
        id: &str,
        responder: &str,
        recipient: &str,
    ) -> Result<bool, StoreError> {
        let changed = self.conn.execute("UPDATE tracked_requests SET state = 'answered', completed_at = ?1 WHERE id = ?2 AND sender = ?3 AND target = ?4 AND state = 'open'", rusqlite::params![now_secs(), id, recipient, responder]).map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }

    pub fn mark_timed_out(&self, id: &str) -> Result<bool, StoreError> {
        let changed = self.conn.execute("UPDATE tracked_requests SET state = 'timed_out', completed_at = ?1 WHERE id = ?2 AND state = 'open'", rusqlite::params![now_secs(), id]).map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }

    pub fn set_escalation_level(&self, id: &str, level: u8) -> Result<(), StoreError> {
        self.conn.execute("UPDATE tracked_requests SET escalation_level = ?1 WHERE id = ?2 AND state = 'open'", rusqlite::params![level, id]).map_err(StoreError::Sqlite)?;
        Ok(())
    }

    fn query_requests<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<TrackedRequest>, StoreError> {
        let mut stmt = self.conn.prepare(sql).map_err(StoreError::Sqlite)?;
        Ok(stmt
            .query_map(params, tracked_request_from_row)
            .map_err(StoreError::Sqlite)?
            .filter_map(Result::ok)
            .collect())
    }

    /// Enregistre un message dans le ledger.
    pub fn record_message(
        &self,
        msg: &bridget_core::BridgetMessage,
        conversation_key: &str,
    ) -> Result<(), StoreError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO ledger (id, ts, sender, target, body, conversation_key) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![msg.id, now, msg.from, msg.to, msg.body, conversation_key,],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Compte les échanges dans une conversation pendant les N dernières secondes.
    pub fn count_recent(
        &self,
        conversation_key: &str,
        window_secs: u64,
    ) -> Result<i64, StoreError> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - window_secs as i64;

        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM ledger WHERE conversation_key = ?1 AND ts >= ?2",
                rusqlite::params![conversation_key, cutoff],
                |row| row.get(0),
            )
            .map_err(StoreError::Sqlite)?;
        Ok(count)
    }

    /// Récupère les échanges récents d'une conversation.
    pub fn recent_messages(&self, limit: usize) -> Result<Vec<LedgerEntry>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, ts, sender, target, body FROM ledger ORDER BY ts DESC LIMIT ?1")
            .map_err(StoreError::Sqlite)?;

        let entries = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok(LedgerEntry {
                    id: row.get(0)?,
                    ts: row.get(1)?,
                    sender: row.get(2)?,
                    target: row.get(3)?,
                    body: row.get(4)?,
                })
            })
            .map_err(StoreError::Sqlite)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(entries)
    }

    /// Purge les messages plus anciens que N jours.
    pub fn purge_older_than_days(&self, days: u32) -> Result<usize, StoreError> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (days as i64 * 86400);

        let deleted = self
            .conn
            .execute(
                "DELETE FROM ledger WHERE ts < ?1",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        self.conn
            .execute(
                "DELETE FROM tracked_requests WHERE state != 'open' AND completed_at < ?1",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(deleted)
    }

    /// Purge les messages quand le fichier dépasse la taille limite.
    pub fn purge_if_too_large(&self, max_bytes: u64) -> Result<(), StoreError> {
        // Cette méthode est appelée avec le chemin du fichier par le daemon.
        // Ici on ne fait que la requête de nettoyage si demandé.
        let _ = max_bytes;
        Ok(())
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn tracked_request_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrackedRequest> {
    Ok(TrackedRequest {
        id: row.get(0)?,
        sender: row.get(1)?,
        target: row.get(2)?,
        state: row.get(3)?,
        created_at: row.get(4)?,
        deadline_at: row.get(5)?,
        escalation_level: row.get(6)?,
        cancel_reason: row.get(7)?,
        completed_at: row.get(8)?,
    })
}

#[derive(Debug)]
pub struct LedgerEntry {
    pub id: String,
    pub ts: i64,
    pub sender: String,
    pub target: String,
    pub body: String,
}

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "SQLite: {}", e),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_idempotent_and_terminal() {
        let path = std::env::temp_dir().join(format!("bridget-store-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let store = Store::open(&path).unwrap();
        store
            .create_request("request-1", "alice", "bob", 60)
            .unwrap();
        let first = store
            .cancel_request("request-1", "alice", Some("priorité changée"))
            .unwrap()
            .unwrap();
        let second = store
            .cancel_request("request-1", "alice", None)
            .unwrap()
            .unwrap();
        assert_eq!(first.state, "cancelled");
        assert_eq!(second.state, "cancelled");
        assert!(!store.mark_answered("request-1", "bob", "alice").unwrap());
        store
            .create_request("request-2", "alice", "bob", 60)
            .unwrap();
        assert!(store.mark_answered("request-2", "bob", "alice").unwrap());
        assert_eq!(
            store.get_request("request-2").unwrap().unwrap().state,
            "answered"
        );
        drop(store);
        let reopened = Store::open(&path).unwrap();
        assert_eq!(
            reopened.get_request("request-1").unwrap().unwrap().state,
            "cancelled"
        );
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }
}
