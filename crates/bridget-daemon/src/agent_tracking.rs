//! Sommeil durable : aucune présence, mission ou définition n'est réécrite.
use bridget_transport::{
    agent_tracking::{AgentTrackingConfirmation, AgentTrackingObservation, AgentTrackingView},
    protocol::ScopedAgentInfo,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "agent_tracking230_tests.rs"]
mod tests;

pub(crate) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS agent_tracking (
        agent_id TEXT PRIMARY KEY, dormant INTEGER NOT NULL DEFAULT 0 CHECK(dormant IN (0,1)),
        synchronize INTEGER NOT NULL DEFAULT 1 CHECK(synchronize IN (0,1)),
        revision INTEGER NOT NULL DEFAULT 0, changed_at INTEGER NOT NULL DEFAULT 0,
        changed_by TEXT NOT NULL DEFAULT '', snapshot TEXT NOT NULL,
        thread_id TEXT UNIQUE, source_revision TEXT, source_state TEXT, pending_id TEXT,
        minimum_source_sequence INTEGER NOT NULL DEFAULT 0,source_sequence INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS agent_tracking_receipts (
        actor TEXT NOT NULL, request_id TEXT NOT NULL, canonical BLOB NOT NULL,
        result TEXT NOT NULL, PRIMARY KEY(actor,request_id));
        CREATE TABLE IF NOT EXISTS agent_tracking_events (
        seq INTEGER PRIMARY KEY, agent_id TEXT NOT NULL, dormant INTEGER NOT NULL,
        synchronize INTEGER NOT NULL, actor TEXT NOT NULL, changed_at INTEGER NOT NULL);",
    )?;
    sync_queue::ensure_schema(conn)
}

fn row_view(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentTrackingView> {
    Ok(AgentTrackingView {
        dormant: row.get(0)?,
        synchronize: row.get(1)?,
        revision: row.get(2)?,
        changed_at: row.get(3)?,
        changed_by: row.get(4)?,
        synchronization_pending: row.get(5)?,
        synchronization_status: row.get(6)?,
    })
}

pub(crate) fn view(conn: &Connection, agent: &str) -> rusqlite::Result<Option<AgentTrackingView>> {
    conn.query_row("SELECT dormant,synchronize,revision,changed_at,changed_by,
        pending_id IS NOT NULL OR EXISTS(SELECT 1 FROM agent_tracking_outbox o WHERE o.agent_id=agent_tracking.agent_id AND phase IN ('queued','inflight','waiting_source'))
        ,CASE
        WHEN EXISTS(SELECT 1 FROM agent_tracking_outbox o WHERE o.agent_id=agent_tracking.agent_id AND phase='inflight') THEN 'awaiting_confirmation'
        WHEN EXISTS(SELECT 1 FROM agent_tracking_outbox o WHERE o.operation_id=agent_tracking.pending_id AND phase='refused') THEN 'refused'
        WHEN EXISTS(SELECT 1 FROM agent_tracking_outbox o WHERE o.agent_id=agent_tracking.agent_id AND phase='waiting_source') THEN 'waiting_for_change'
        WHEN pending_id IS NOT NULL THEN 'pending'
        WHEN synchronize=0 THEN 'local_only'
        WHEN thread_id IS NULL THEN 'not_linked'
        ELSE 'last_action_applied' END
        FROM agent_tracking WHERE agent_id=?1", [agent], row_view).optional()
}

pub(crate) fn dormant(conn: &Connection, agent: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT dormant FROM agent_tracking WHERE agent_id=?1",
        [agent],
        |r| r.get(0),
    )
    .optional()
    .map(|v| v.unwrap_or(false))
}

pub(crate) fn dormant_ids(
    conn: &Connection,
) -> rusqlite::Result<std::collections::HashSet<String>> {
    conn.prepare("SELECT agent_id FROM agent_tracking WHERE dormant=1")?
        .query_map([], |r| r.get(0))?
        .collect()
}

/// La fiche de secours ne constitue jamais une observation d'un processus.
pub(crate) fn retained(conn: &Connection) -> rusqlite::Result<BTreeMap<String, ScopedAgentInfo>> {
    let mut query = conn.prepare("SELECT agent_id,snapshot FROM agent_tracking")?;
    query
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .map(|r| {
            let (id, snapshot) = r?;
            let mut agent: ScopedAgentInfo = serde_json::from_str(&snapshot)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            agent.agent.state = "unobserved".into();
            agent.agent.connection_id.clear();
            agent.agent.execution = None;
            agent.agent.rate_limits.clear();
            agent.agent.disk_space = None;
            agent.agent.model_mismatch = None;
            agent.tracking = view(conn, &id)?;
            Ok((id, agent))
        })
        .collect()
}

pub(crate) fn bind(
    conn: &Connection,
    agent: &ScopedAgentInfo,
    thread: &str,
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    let snapshot = serde_json::to_string(agent)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    tx.execute(
        "INSERT INTO agent_tracking(agent_id,snapshot,thread_id) VALUES(?1,?2,?3)
        ON CONFLICT(agent_id) DO UPDATE SET snapshot=excluded.snapshot,thread_id=excluded.thread_id
        WHERE agent_tracking.thread_id IS NULL OR agent_tracking.thread_id=excluded.thread_id",
        params![agent.agent.agent_id, snapshot, thread],
    )?;
    let needs_sync:bool=tx.query_row("SELECT revision>0 AND synchronize=1 AND source_revision IS NULL AND NOT EXISTS(SELECT 1 FROM agent_tracking_outbox o WHERE o.agent_id=agent_tracking.agent_id) FROM agent_tracking WHERE agent_id=?1",[&agent.agent.agent_id],|r|r.get(0))?;
    if needs_sync {
        sync_queue::enqueue(
            &tx,
            &agent.agent.agent_id,
            &uuid::Uuid::new_v4().to_string(),
        )?;
    }
    tx.commit()
}

pub(crate) const DORMANT_MESSAGE: &str = "Agent dormant : cette nouvelle sollicitation est refusée. Son utilisateur doit demander son réveil, depuis sa fiche ou avec bridget agents wake <UUID>. Après réveil confirmé, une nouvelle sollicitation sera possible ; conservez ce reçu refusé. Les missions déjà engagées et leurs résultats sont conservés ; les autres travaux autorisés peuvent continuer.";

fn record_change(
    conn: &Connection,
    agent: &str,
    dormant: bool,
    synchronize: bool,
    actor: &str,
    now: i64,
) -> rusqlite::Result<u64> {
    conn.execute("INSERT INTO agent_tracking_events(agent_id,dormant,synchronize,actor,changed_at) VALUES(?1,?2,?3,?4,?5)",params![agent,dormant,synchronize,actor,now])?;
    let revision = conn.last_insert_rowid() as u64;
    conn.execute("UPDATE agent_tracking SET dormant=?2,synchronize=?3,revision=?4,changed_at=?5,changed_by=?6 WHERE agent_id=?1",params![agent,dormant,synchronize,revision,now,actor])?;
    if dormant {
        conn.execute("UPDATE thread_wakes SET state='cancelled',reason='agent_dormant' WHERE agent_id=?1 AND active_through_seq IS NULL AND state IN ('pending','refused')",[agent])?;
    }
    // L'ancien binaire doit refuser d'ouvrir un état dont il ignore le sommeil.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS idempotency_schema_migrations(version INTEGER PRIMARY KEY);
        INSERT OR IGNORE INTO idempotency_schema_migrations(version) VALUES(8);",
    )?;
    Ok(revision)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn set(
    conn: &Connection,
    actor: &str,
    request: &str,
    canonical: &[u8],
    agent: &ScopedAgentInfo,
    dormant: bool,
    synchronize: Option<bool>,
    expected: Option<u64>,
    now: i64,
) -> Result<Value, &'static str> {
    if uuid::Uuid::parse_str(request).is_err() {
        return Err("invalid_request");
    }
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| "storage_unavailable")?;
    if let Some((old, result)) = tx
        .query_row(
            "SELECT canonical,result FROM agent_tracking_receipts WHERE actor=?1 AND request_id=?2",
            params![actor, request],
            |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|_| "storage_unavailable")?
    {
        return if old == canonical {
            serde_json::from_str(&result).map_err(|_| "storage_unavailable")
        } else {
            Err("envelope_mismatch")
        };
    }
    let previous = view(&tx, &agent.agent.agent_id).map_err(|_| "storage_unavailable")?;
    if expected.is_some_and(|e| e != previous.as_ref().map_or(0, |v| v.revision)) {
        return Err("revision_conflict");
    }
    let synchronize =
        synchronize.unwrap_or_else(|| previous.as_ref().is_none_or(|v| v.synchronize));
    let snapshot = serde_json::to_string(agent).map_err(|_| "invalid_request")?;
    tx.execute(
        "INSERT OR IGNORE INTO agent_tracking(agent_id,snapshot) VALUES(?1,?2)",
        params![agent.agent.agent_id, snapshot],
    )
    .map_err(|_| "storage_unavailable")?;
    let retry_refused:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM agent_tracking_outbox o JOIN agent_tracking a ON a.pending_id=o.operation_id WHERE a.agent_id=?1 AND o.phase='refused')",[&agent.agent.agent_id],|r|r.get(0)).map_err(|_|"storage_unavailable")?;
    if retry_refused
        || previous
            .as_ref()
            .is_none_or(|v| v.dormant != dormant || v.synchronize != synchronize)
    {
        record_change(&tx, &agent.agent.agent_id, dormant, synchronize, actor, now)
            .map_err(|_| "storage_unavailable")?;
        sync_queue::enqueue(&tx, &agent.agent.agent_id, request)
            .map_err(|_| "storage_unavailable")?;
    }
    let result = json!({"status":"ok","agent_id":agent.agent.agent_id,"request_id":request,"tracking":view(&tx,&agent.agent.agent_id).map_err(|_|"storage_unavailable")?});
    tx.execute("INSERT INTO agent_tracking_receipts(actor,request_id,canonical,result) VALUES(?1,?2,?3,?4)",params![actor,request,canonical,result.to_string()]).map_err(|_|"storage_unavailable")?;
    tx.commit().map_err(|_| "storage_unavailable")?;
    Ok(result)
}

#[path = "agent_tracking_sync230.rs"]
mod sync_queue;
pub(crate) use sync_queue::sync;
