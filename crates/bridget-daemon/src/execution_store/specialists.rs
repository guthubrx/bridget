//! Attribution222 : métadonnées et soumission native réservées dans la même transaction.
use super::*;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct SpecialistSubmission {
    pub issuer_scope: String,
    pub request_id: String,
    pub selected_agent_id: String,
    pub profile_revision: u64,
    pub grant_id: Option<String>,
    pub grant_revision: u64,
    pub native_submission_id: String,
    pub created_at: i64,
    #[serde(skip_serializing)]
    pub message: bridget_core::BridgetMessage,
    pub profile_snapshot: serde_json::Value,
    pub submission_state: String,
    pub execution_state: Option<String>,
}
pub(crate) fn specialist_submission_id(issuer: &str, request: &str, canonical: &[u8]) -> String {
    let digest = format!("{:x}", Sha256::digest(canonical));
    let tuple = serde_json::to_vec(&(issuer, request, digest)).expect("string tuple");
    let name = format!("bridget:specialist:v1:{:x}", Sha256::digest(tuple));
    let hash = Sha256::digest(name.as_bytes());
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}
impl ExecutionStore {
    pub(crate) fn init_specialist_requests(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch("CREATE TABLE IF NOT EXISTS specialist_requests (
            issuer_scope TEXT NOT NULL, request_id TEXT NOT NULL, canonical_bytes BLOB NOT NULL,
            selected_agent_id TEXT NOT NULL, profile_revision INTEGER NOT NULL,
            grant_id TEXT, grant_revision INTEGER NOT NULL,
            native_submission_id TEXT NOT NULL UNIQUE REFERENCES work_submissions(submission_id),
            created_at INTEGER NOT NULL,profile_snapshot TEXT NOT NULL DEFAULT '{}', PRIMARY KEY(issuer_scope,request_id));
            CREATE TABLE IF NOT EXISTS specialist_compositions (
                issuer_scope TEXT NOT NULL,request_id TEXT NOT NULL,canonical_bytes BLOB NOT NULL,
                plan_json TEXT NOT NULL,created_at INTEGER NOT NULL,PRIMARY KEY(issuer_scope,request_id));
            CREATE INDEX IF NOT EXISTS specialist_requests_target ON specialist_requests(selected_agent_id,created_at);")
    }
    pub(crate) fn specialist_request(
        &self,
        issuer: &str,
        request_id: &str,
        canonical: Option<&[u8]>,
    ) -> rusqlite::Result<Option<SpecialistSubmission>> {
        let row = self.conn.query_row(
            "SELECT r.canonical_bytes,r.selected_agent_id,r.profile_revision,r.grant_id,r.grant_revision,
             r.native_submission_id,r.created_at,s.message_json,s.state,
             (SELECT e.state FROM executions e WHERE e.submission_id=r.native_submission_id ORDER BY e.created_at DESC LIMIT 1),r.profile_snapshot
             FROM specialist_requests r JOIN work_submissions s ON s.submission_id=r.native_submission_id
             WHERE r.issuer_scope=?1 AND r.request_id=?2", params![issuer,request_id],
            |r| Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,String>(1)?,r.get::<_,u64>(2)?,
                r.get::<_,Option<String>>(3)?,r.get::<_,u64>(4)?,r.get::<_,String>(5)?,
                r.get::<_,i64>(6)?,r.get::<_,String>(7)?,r.get::<_,String>(8)?,
                r.get::<_,Option<String>>(9)?,r.get::<_,String>(10)?))
        ).optional()?;
        row.map(|r| {
            if canonical.is_some_and(|c| c != r.0.as_slice()) {
                return Err(rusqlite::Error::InvalidParameterName(
                    "envelope_mismatch".into(),
                ));
            }
            let message = serde_json::from_str(&r.7).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(7, Type::Text, Box::new(e))
            })?;
            Ok(SpecialistSubmission {
                issuer_scope: issuer.into(),
                request_id: request_id.into(),
                selected_agent_id: r.1,
                profile_revision: r.2,
                grant_id: r.3,
                grant_revision: r.4,
                native_submission_id: r.5,
                created_at: r.6,
                message,
                submission_state: r.8,
                execution_state: r.9,
                profile_snapshot: serde_json::from_str(&r.10).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(10, Type::Text, Box::new(e))
                })?,
            })
        })
        .transpose()
    }
    /// Plan immuable seulement : états, résultats et dépendances réalisées se lisent
    /// dans les soumissions et les réponses natives, jamais dans cette table.
    pub(crate) fn specialist_composition(
        &self,
        issuer: &str,
        key: &str,
        canonical: Option<&[u8]>,
    ) -> rusqlite::Result<Option<serde_json::Value>> {
        let row:Option<(Vec<u8>,String)>=self.conn.query_row(
            "SELECT canonical_bytes,plan_json FROM specialist_compositions WHERE issuer_scope=?1 AND request_id=?2",
            params![issuer,key],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        row.map(|(stored, body)| {
            if canonical.is_some_and(|v| v != stored) {
                return Err(rusqlite::Error::InvalidParameterName(
                    "envelope_mismatch".into(),
                ));
            }
            serde_json::from_str(&body)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, Type::Text, Box::new(e)))
        })
        .transpose()
    }
    pub(crate) fn reserve_specialist_composition(
        &self,
        issuer: &str,
        key: &str,
        canonical: &[u8],
        plan: &serde_json::Value,
        now: i64,
    ) -> rusqlite::Result<serde_json::Value> {
        if let Some(known) = self.specialist_composition(issuer, key, Some(canonical))? {
            return Ok(known);
        }
        self.conn.execute("INSERT INTO specialist_compositions(issuer_scope,request_id,canonical_bytes,plan_json,created_at) VALUES(?1,?2,?3,?4,?5)",
            params![issuer,key,canonical,plan.to_string(),now])?;
        self.specialist_composition(issuer, key, Some(canonical))?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }
    /// Appel sous la garde du daemon, après choix/autorité. Aucun départ ni file ici.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_specialist_request(
        &self,
        issuer: &str,
        request_id: &str,
        canonical: &[u8],
        selected_agent: &str,
        profile_revision: u64,
        grant_id: Option<&str>,
        grant_revision: u64,
        message: &bridget_core::BridgetMessage,
        profile_snapshot: &serde_json::Value,
        now: i64,
    ) -> rusqlite::Result<SpecialistSubmission> {
        if let Some(existing) = self.specialist_request(issuer, request_id, Some(canonical))? {
            return Ok(existing);
        }
        let id = specialist_submission_id(issuer, request_id, canonical);
        if message.id != id || message.to != selected_agent {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let body = serde_json::to_string(message)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let refs = serde_json::to_string(&message.references)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("INSERT INTO work_submissions (submission_id,message_id,target_agent,origin,intent,references_json,message_json,state,accepted_at)
            VALUES (?1,?1,?2,'agent','trigger_turn',?3,?4,'prepared',?5)",params![id,selected_agent,refs,body,now])?;
        tx.execute("INSERT INTO specialist_requests (issuer_scope,request_id,canonical_bytes,selected_agent_id,profile_revision,grant_id,grant_revision,native_submission_id,created_at,profile_snapshot)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![issuer,request_id,canonical,selected_agent,profile_revision,grant_id,grant_revision,id,now,serde_json::to_string(profile_snapshot).map_err(|e|rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?])?;
        tx.commit()?;
        self.specialist_request(issuer, request_id, Some(canonical))?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }
    /// Le seul raccord222 qui promeut prepared. Le caller recontrôle le grant sous
    /// DaemonState avant cet appel et avant remise ; une lecture n'appelle jamais ce port.
    pub(crate) fn accept_specialist_request(
        &self,
        issuer: &str,
        request_id: &str,
        canonical: &[u8],
        grant_revision: u64,
        project: Option<&ProjectReference>,
        now: i64,
    ) -> rusqlite::Result<bool> {
        let tx = self.conn.unchecked_transaction()?;
        let (id,stored,revision):(String,Vec<u8>,u64)=tx.query_row(
            "SELECT native_submission_id,canonical_bytes,grant_revision FROM specialist_requests WHERE issuer_scope=?1 AND request_id=?2",
            params![issuer,request_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        if stored != canonical || revision != grant_revision {
            return Err(rusqlite::Error::InvalidParameterName(
                "mandate_changed".into(),
            ));
        }
        let changed=tx.execute("UPDATE work_submissions SET state='accepted' WHERE submission_id=?1 AND state='prepared'",[&id])?;
        if changed == 1 {
            tx.execute("INSERT INTO executions (execution_id,submission_id,agent_instance_id,project_id,binding_generation,generation,state,revision,reason,created_at,updated_at)
                VALUES (?1,?2,NULL,?3,?4,1,'starting',0,NULL,?5,?5)",
                params![format!("execution-{id}"),id,project.map(|p|&p.project_id),project.map(|p|p.binding_generation),now])?;
        }
        tx.commit()?;
        Ok(changed == 1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec222_prepared_reservation_is_atomic_inert_and_idempotent() {
        let root =
            std::env::temp_dir().join(format!("bridget222-reserve-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let store = ExecutionStore::open(&root.join("state.db")).unwrap();
        store.init_specialist_requests().unwrap();
        let canon = b"canonical";
        let mut m = bridget_core::BridgetMessage::new("owner", "target", "mission");
        m.id = specialist_submission_id("owner", "request", canon);
        m.intent = Some(bridget_core::MessageIntent::TriggerTurn);
        let first = store
            .reserve_specialist_request(
                "owner",
                "request",
                canon,
                "target",
                1,
                Some("grant"),
                2,
                &m,
                &serde_json::json!({"budget":{}}),
                10,
            )
            .unwrap();
        assert_eq!(first.submission_state, "prepared");
        assert!(store.take_next_submission("target").unwrap().is_none());
        assert!(
            store
                .record_starting(&m.id, "forbidden", "target", 11)
                .is_err()
        );
        assert!(store.execution_snapshot("forbidden").unwrap().is_none());
        assert_eq!(
            store
                .reserve_specialist_request(
                    "owner",
                    "request",
                    canon,
                    "target",
                    1,
                    Some("grant"),
                    2,
                    &m,
                    &serde_json::json!({"budget":{}}),
                    20
                )
                .unwrap()
                .created_at,
            10
        );
        assert!(
            store
                .specialist_request("owner", "request", Some(b"changed"))
                .is_err()
        );
        assert!(
            store
                .accept_specialist_request("owner", "request", canon, 3, None, 21)
                .is_err()
        );
        assert!(
            store
                .accept_specialist_request("owner", "request", canon, 2, None, 22)
                .unwrap()
        );
        assert!(
            !store
                .accept_specialist_request("owner", "request", canon, 2, None, 23)
                .unwrap()
        );
        assert_eq!(
            store
                .specialist_request("owner", "request", None)
                .unwrap()
                .unwrap()
                .execution_state
                .as_deref(),
            Some("starting")
        );
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
}
