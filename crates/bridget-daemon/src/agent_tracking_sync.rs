//! File de classement : une issue inconnue précède tout choix ultérieur du même agent.
use super::*;
use bridget_transport::agent_tracking::AgentTrackingSyncOutcome;

pub(super) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS agent_tracking_outbox (
        operation_id TEXT PRIMARY KEY,agent_id TEXT NOT NULL,revision INTEGER NOT NULL,
        thread_id TEXT NOT NULL,source_host TEXT NOT NULL,dormant INTEGER NOT NULL,
        phase TEXT NOT NULL CHECK(phase IN ('queued','inflight','waiting_source','applied','superseded','refused')),
        attempt INTEGER NOT NULL DEFAULT 0 CHECK(attempt BETWEEN 0 AND 2),source_revision TEXT,last_attempt_at INTEGER,
        applied_sequence INTEGER,
        FOREIGN KEY(agent_id) REFERENCES agent_tracking(agent_id));
        CREATE INDEX IF NOT EXISTS agent_tracking_outbox_agent ON agent_tracking_outbox(agent_id,phase,revision);
        CREATE INDEX IF NOT EXISTS agent_tracking_outbox_host ON agent_tracking_outbox(source_host,phase,last_attempt_at);
        CREATE TABLE IF NOT EXISTS agent_tracking_attempt_receipts(
        operation_id TEXT NOT NULL,attempt INTEGER NOT NULL,receipt TEXT NOT NULL,
        recorded_at INTEGER NOT NULL,PRIMARY KEY(operation_id,attempt),
        FOREIGN KEY(operation_id) REFERENCES agent_tracking_outbox(operation_id));")
}

pub(super) fn enqueue(conn: &Connection, agent: &str, operation: &str) -> rusqlite::Result<()> {
    // Aucun ordre non remis n'a d'effet à conserver. Une remise incertaine, elle,
    // reste gravée et devra être rapprochée avant l'application du choix suivant.
    conn.execute("UPDATE agent_tracking_outbox SET phase='superseded' WHERE agent_id=?1 AND phase IN ('queued','waiting_source','refused')",[agent])?;
    conn.execute("INSERT INTO agent_tracking_outbox(operation_id,agent_id,revision,thread_id,source_host,dormant,phase,source_revision)
        SELECT ?2,agent_id,revision,thread_id,json_extract(snapshot,'$.host'),dormant,'queued',source_revision
        FROM agent_tracking WHERE agent_id=?1 AND synchronize=1 AND thread_id IS NOT NULL",params![agent,operation])?;
    conn.execute("UPDATE agent_tracking SET pending_id=CASE WHEN synchronize=1 AND thread_id IS NOT NULL THEN ?2 ELSE NULL END WHERE agent_id=?1",params![agent,operation])?;
    Ok(())
}

fn pending(conn: &Connection, agent: &str) -> rusqlite::Result<bool> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM agent_tracking_outbox WHERE agent_id=?1 AND phase IN ('queued','inflight','waiting_source')) OR EXISTS(SELECT 1 FROM agent_tracking WHERE agent_id=?1 AND pending_id IS NOT NULL)",[agent],|r|r.get(0))
}

pub(crate) fn sync(
    conn: &Connection,
    host: &str,
    observations: &[AgentTrackingObservation],
    confirmations: &[AgentTrackingConfirmation],
    now: i64,
) -> Result<Value, &'static str> {
    if host.is_empty()
        || host.len() > 256
        || observations.len() > 10000
        || confirmations.len() > 128
        || observations.iter().any(|o| {
            o.thread_id.is_empty()
                || o.thread_id.len() > 2048
                || o.source_revision.is_empty()
                || o.source_revision.len() > 128
                || !matches!(
                    o.explicit_state.as_deref(),
                    None | Some("settled" | "active")
                )
        })
    {
        return Err("invalid_request");
    }
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| "storage_unavailable")?;
    let fresh: BTreeMap<_, _> = observations
        .iter()
        .map(|o| (o.thread_id.as_str(), o))
        .collect();
    if fresh.len() != observations.len() {
        return Err("invalid_request");
    }
    let mut confirmed = BTreeMap::new();
    for ack in confirmations {
        type Pending = (String, bool, u32, Option<String>);
        let existing:Option<Pending>=tx.query_row("SELECT agent_id,dormant,attempt,source_revision FROM agent_tracking_outbox WHERE operation_id=?1 AND thread_id=?2 AND source_host=?3 AND phase='inflight'",params![ack.operation_id,ack.thread_id,host],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(|_|"storage_unavailable")?;
        let Some((agent, dormant, attempt, source_revision)) = existing else {
            continue;
        };
        if attempt != ack.attempt {
            continue;
        }
        let receipt = serde_json::to_string(ack).map_err(|_| "invalid_request")?;
        tx.execute("INSERT OR IGNORE INTO agent_tracking_attempt_receipts(operation_id,attempt,receipt,recorded_at) VALUES(?1,?2,?3,?4)",params![ack.operation_id,ack.attempt,receipt,now]).map_err(|_|"storage_unavailable")?;
        match ack.outcome {
            AgentTrackingSyncOutcome::Applied { sequence } => {
                if sequence > i64::MAX as u64 {
                    return Err("invalid_request");
                }
                tx.execute("UPDATE agent_tracking_outbox SET phase='applied',applied_sequence=?2 WHERE operation_id=?1",params![ack.operation_id,sequence]).map_err(|_|"storage_unavailable")?;
                tx.execute(
                    "UPDATE agent_tracking SET minimum_source_sequence=?2,source_state=CASE WHEN source_sequence<=?2 THEN ?3 ELSE source_state END,source_revision=CASE WHEN source_sequence<=?2 THEN NULL ELSE source_revision END,source_sequence=MAX(source_sequence,?2) WHERE agent_id=?1",
                    params![agent, sequence,if dormant {"settled"}else{"active"}],
                )
                .map_err(|_| "storage_unavailable")?;
                tx.execute(
                    "UPDATE agent_tracking SET pending_id=NULL WHERE agent_id=?1 AND pending_id=?2",
                    params![agent, ack.operation_id],
                )
                .map_err(|_| "storage_unavailable")?;
                confirmed.insert(ack.thread_id.as_str(), dormant);
            }
            AgentTrackingSyncOutcome::Rejected {} => {
                let revision = fresh
                    .get(ack.thread_id.as_str())
                    .map(|o| o.source_revision.clone())
                    .or(source_revision);
                // Trois départs au plus ; seuls des refus certains autorisent
                // une nouvelle commande de classement, après observation nouvelle.
                let superseded:bool=tx.query_row("SELECT a.revision>o.revision FROM agent_tracking a JOIN agent_tracking_outbox o ON o.agent_id=a.agent_id WHERE o.operation_id=?1",[&ack.operation_id],|r|r.get(0)).map_err(|_|"storage_unavailable")?;
                let phase = if superseded {
                    "superseded"
                } else if attempt < 2 {
                    "waiting_source"
                } else {
                    "refused"
                };
                tx.execute("UPDATE agent_tracking_outbox SET phase=?2,source_revision=?3 WHERE operation_id=?1",params![ack.operation_id,phase,revision]).map_err(|_|"storage_unavailable")?;
            }
        }
    }
    for o in observations {
        type Prior = (String, bool, bool, Option<String>, Option<String>, u64);
        if o.source_sequence > i64::MAX as u64 {
            return Err("invalid_request");
        }
        let prior:Option<Prior>=tx.query_row("SELECT agent_id,dormant,synchronize,source_revision,source_state,source_sequence FROM agent_tracking WHERE thread_id=?1 AND json_extract(snapshot,'$.host')=?2 AND minimum_source_sequence<=?3",params![o.thread_id,host,o.source_sequence],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(|_|"storage_unavailable")?;
        let Some((agent, dormant, synchronize, last_revision, last_state, last_sequence)) = prior
        else {
            continue;
        };
        if o.source_sequence < last_sequence
            || o.source_sequence == last_sequence
                && (last_revision
                    .as_ref()
                    .is_some_and(|r| r != &o.source_revision)
                    || (last_revision.is_some() || last_state.is_some())
                        && last_state != o.explicit_state)
        {
            continue;
        }
        let newer_choice = confirmed.get(o.thread_id.as_str()).is_some_and(|d| {
            o.explicit_state.as_deref() == Some(if *d { "active" } else { "settled" })
        });
        if synchronize
            && !pending(&tx, &agent).map_err(|_| "storage_unavailable")?
            && ((o.explicit_state != last_state
                && last_revision.as_ref() != Some(&o.source_revision))
                || newer_choice)
            && o.explicit_state.is_some()
        {
            let sleep = o.explicit_state.as_deref() == Some("settled");
            if sleep != dormant {
                record_change(&tx, &agent, sleep, true, "linked_client", now)
                    .map_err(|_| "storage_unavailable")?;
            }
        }
        tx.execute(
            "UPDATE agent_tracking SET source_revision=?2,source_state=?3,source_sequence=?4 WHERE agent_id=?1",
            params![agent, o.source_revision, o.explicit_state,o.source_sequence],
        )
        .map_err(|_| "storage_unavailable")?;
    }
    // Une seule remise par échange, équitable entre agents. Une opération
    // incertaine bloque son agent, jamais la file entière de l'hôte.
    let mut query=tx.prepare("SELECT o.operation_id,o.agent_id,o.thread_id,o.dormant,o.phase,o.attempt,o.source_revision,a.minimum_source_sequence,a.source_revision,a.source_sequence FROM agent_tracking_outbox o JOIN agent_tracking a ON a.agent_id=o.agent_id
        WHERE o.source_host=?1 AND o.phase IN ('queued','inflight','waiting_source')
        AND NOT EXISTS(SELECT 1 FROM agent_tracking_outbox older WHERE older.agent_id=o.agent_id AND older.revision<o.revision AND older.phase='inflight')
        ORDER BY o.last_attempt_at IS NOT NULL,o.last_attempt_at,o.revision LIMIT 10000").map_err(|_|"storage_unavailable")?;
    type Candidate = (
        String,
        String,
        String,
        bool,
        String,
        u32,
        Option<String>,
        u64,
        Option<String>,
        u64,
    );
    let candidates = query
        .query_map([host], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
            ))
        })
        .map_err(|_| "storage_unavailable")?
        .collect::<rusqlite::Result<Vec<Candidate>>>()
        .map_err(|_| "storage_unavailable")?;
    drop(query);
    let mut selected = None;
    for (
        id,
        _agent,
        thread,
        dormant,
        phase,
        mut attempt,
        source_revision,
        floor,
        current_revision,
        last_sequence,
    ) in candidates
    {
        let observation = fresh.get(thread.as_str()).filter(|o| {
            o.source_sequence >= floor
                && o.source_sequence >= last_sequence
                && current_revision
                    .as_ref()
                    .is_none_or(|r| &o.source_revision == r)
        });
        if phase != "inflight" && dormant && !observation.is_some_and(|o| o.can_settle) {
            continue;
        }
        if phase == "waiting_source" {
            if !observation.is_some_and(|o| {
                source_revision
                    .as_deref()
                    .is_none_or(|r| o.source_revision.as_str() != r)
            }) {
                continue;
            }
            attempt += 1;
        }
        tx.execute("UPDATE agent_tracking_outbox SET phase='inflight',attempt=?2,last_attempt_at=?3 WHERE operation_id=?1",params![id,attempt,now]).map_err(|_|"storage_unavailable")?;
        let command = crate::t3code::stable_uuid(&format!("agent-tracking:{id}:{attempt}"));
        selected = Some(
            json!({"thread_id":thread,"operation_id":id,"dormant":dormant,"attempt":attempt,"client_command_id":command}),
        );
        break;
    }
    tx.commit().map_err(|_| "storage_unavailable")?;
    Ok(json!({"status":"ok","pending":selected.into_iter().collect::<Vec<_>>()}))
}
