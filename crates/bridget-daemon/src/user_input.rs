//! Questions humaines181 : domaine natif commun aux clients, sans dépendance T3.
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
const MAX_BYTES: usize = 64 * 1024;
const RETENTION: i64 = 7 * 86400;
fn storage(_: rusqlite::Error) -> String {
    "storage_unavailable".into()
}
fn text(value: &Value, limit: usize, empty: bool) -> bool {
    value.as_str().is_some_and(|s| {
        s.len() <= limit
            && (empty || !s.trim().is_empty())
            && !s.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
    })
}
fn fields(value: &Value, allowed: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|v| v.keys().all(|key| allowed.contains(&key.as_str())))
}
fn uuid(id: &str) -> bool {
    uuid::Uuid::parse_str(id)
        .is_ok_and(|v| matches!(v.get_version_num(), 4 | 5) && v.to_string() == id)
}
fn encoded(value: &Value) -> Result<String, String> {
    let s = serde_json::to_string(value).map_err(|_| "invalid_request")?;
    if s.len() > MAX_BYTES {
        return Err("resource_limit".into());
    }
    Ok(s)
}
pub(crate) fn validate_questions(questions: &Value) -> Result<(), String> {
    encoded(questions)?;
    let q = questions
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 16)
        .ok_or("invalid_questions")?;
    let mut ids = HashSet::new();
    let mut count = q.len();
    for item in q {
        if !fields(
            item,
            &[
                "id",
                "header",
                "question",
                "options",
                "multiSelect",
                "allowCustomAnswer",
                "initialAnswer",
                "required",
            ],
        ) || !text(&item["id"], 4096, false)
            || !text(&item["header"], 1024, false)
            || !text(&item["question"], 16384, false)
            || !ids.insert(item["id"].as_str().ok_or("invalid_questions")?)
            || item.get("multiSelect").is_some_and(|v| !v.is_boolean())
            || item
                .get("allowCustomAnswer")
                .is_some_and(|v| !v.is_boolean())
            || item.get("required").is_some_and(|v| !v.is_boolean())
            || item
                .get("initialAnswer")
                .is_some_and(|v| !text(v, 10000, true))
        {
            return Err("invalid_questions".into());
        }
        let options = item["options"]
            .as_array()
            .filter(|a| a.len() <= 32)
            .ok_or("invalid_questions")?;
        count += options.len();
        if count > 64 {
            return Err("resource_limit".into());
        }
        let mut values = HashSet::new();
        for o in options {
            if !fields(o, &["label", "description", "value"])
                || !text(&o["label"], 4096, false)
                || o.get("description").is_some_and(|v| !text(v, 8192, true))
                || o.get("value").is_some_and(|v| !text(v, 4096, false))
            {
                return Err("invalid_questions".into());
            }
            let key = o
                .get("value")
                .unwrap_or(&o["label"])
                .as_str()
                .ok_or("invalid_questions")?;
            if !values.insert(key) {
                return Err("invalid_questions".into());
            }
        }
        if options.is_empty() && item["allowCustomAnswer"] == false {
            return Err("invalid_questions".into());
        }
    }
    Ok(())
}
pub(crate) fn validate_answers(questions: &Value, answers: &Value) -> Result<(), String> {
    validate_questions(questions)?;
    if encoded(answers)?.len() > 32 * 1024 {
        return Err("resource_limit".into());
    }
    let entries = answers.as_object().ok_or("invalid_answer")?;
    let questions = questions.as_array().ok_or("invalid_questions")?;
    if entries.len() != questions.len() {
        return Err("invalid_answer".into());
    }
    for question in questions {
        let id = question["id"].as_str().ok_or("invalid_questions")?;
        let answer = entries.get(id).ok_or("invalid_answer")?;
        if !fields(answer, &["selected", "text"]) || !text(&answer["text"], 10000, true) {
            return Err("invalid_answer".into());
        }
        let selected = answer["selected"].as_array().ok_or("invalid_answer")?;
        let body = answer["text"].as_str().ok_or("invalid_answer")?;
        if (question["required"] != false && selected.is_empty() && body.trim().is_empty())
            || (question["multiSelect"] != true && selected.len() > 1)
            || (question["allowCustomAnswer"] == false && !body.is_empty())
        {
            return Err("invalid_answer".into());
        }
        let options = question["options"].as_array().ok_or("invalid_questions")?;
        let allowed: HashSet<_> = options
            .iter()
            .filter_map(|v| v.get("value").unwrap_or(&v["label"]).as_str())
            .collect();
        let mut seen = HashSet::new();
        for value in selected {
            let value = value.as_str().ok_or("invalid_answer")?;
            if !allowed.contains(value) || !seen.insert(value) {
                return Err("invalid_answer".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS user_input_requests (
      id TEXT PRIMARY KEY, owner TEXT NOT NULL, instance TEXT NOT NULL, project TEXT,
      origin TEXT NOT NULL, origin_ref TEXT NOT NULL, revision TEXT NOT NULL,
      state TEXT NOT NULL CHECK(state IN ('pending','submitted','applied','resolved_elsewhere','expired','cancelled','unknown','closed')),
      command_id TEXT UNIQUE, actor TEXT, data TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, expires_at INTEGER NOT NULL,
      UNIQUE(owner,instance,origin,origin_ref));
      CREATE INDEX IF NOT EXISTS user_input_requests_owner_state ON user_input_requests(owner,state,created_at,id);
      CREATE INDEX IF NOT EXISTS user_input_requests_state ON user_input_requests(state,created_at,id);
      CREATE INDEX IF NOT EXISTS user_input_requests_expiry ON user_input_requests(expires_at);
      CREATE INDEX IF NOT EXISTS user_input_requests_retention ON user_input_requests(updated_at) WHERE state NOT IN ('pending','submitted','unknown') AND json_type(data,'$.questions') IS NOT NULL;")
}
pub(crate) fn get(conn: &Connection, id: &str) -> Result<Option<Value>, String> {
    let value: Option<String> = conn
        .query_row(
            "SELECT data FROM user_input_requests WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    value
        .map(|s| serde_json::from_str(&s).map_err(|_| "storage_unavailable".into()))
        .transpose()
}
fn expire(conn: &Connection, now: i64) -> Result<(), String> {
    conn.execute("UPDATE user_input_requests SET state='expired',data=json_set(data,'$.state','expired') WHERE expires_at<=?1 AND state='pending'",[now]).map_err(storage)?;
    // Tombstone minimal pour refuser une vieille clé sans garder les textes après sept jours.
    conn.execute("UPDATE user_input_requests SET data=json_remove(data,'$.questions','$.callback','$.answers','$.actor') WHERE updated_at<?1 AND state NOT IN ('pending','submitted','unknown') AND json_type(data,'$.questions') IS NOT NULL",[now-RETENTION]).map_err(storage)?;
    Ok(())
}
pub(crate) fn create(
    conn: &Connection,
    owner: &str,
    instance: &str,
    project: Option<&str>,
    id: &str,
    questions: &Value,
    origin: &str,
    callback: &Value,
    now: i64,
) -> Result<Value, String> {
    validate_questions(questions)?;
    if !uuid(id)
        || owner.is_empty()
        || instance.is_empty()
        || !matches!(origin, "mcp" | "codex" | "claude" | "t3")
        || encoded(callback)?.len() > 16384
    {
        return Err("invalid_request".into());
    }
    let revision = format!("{:x}", Sha256::digest(encoded(questions)?.as_bytes()));
    let origin_ref = if callback.is_null() {
        id.to_owned()
    } else {
        format!("{:x}", Sha256::digest(encoded(callback)?.as_bytes()))
    };
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(storage)?;
    expire(&tx, now)?;
    if let Some(existing) = get(&tx, id)? {
        if existing["owner"] != owner || existing["instance"] != instance {
            return Err("question_unavailable".into());
        }
        if existing["expires_at"].as_i64().is_none_or(|t| t <= now) {
            return Err("idempotency_expired".into());
        }
        if existing["revision"] != revision
            || existing["project"] != json!(project)
            || existing["origin"] != origin
            || existing["callback"] != *callback
        {
            return Err("envelope_mismatch".into());
        }
        tx.commit().map_err(storage)?;
        return Ok(existing);
    }
    let (own,total,retained):(i64,i64,i64)=tx.query_row("SELECT COALESCE(SUM(owner=?1 AND state IN ('pending','submitted','unknown')),0),COALESCE(SUM(state IN ('pending','submitted','unknown')),0),COUNT(*) FROM user_input_requests",[owner],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(storage)?;
    if own >= 32 || total >= 2048 || retained >= 100000 {
        return Err("resource_limit".into());
    }
    let data = json!({"request_id":id,"owner":owner,"instance":instance,"project":project,"origin":origin,"callback":callback,"revision":revision,"questions":questions,"state":"pending","created_at":now,"updated_at":now,"expires_at":now+RETENTION});
    tx.execute("INSERT INTO user_input_requests(id,owner,instance,project,origin,origin_ref,revision,state,data,created_at,updated_at,expires_at) VALUES(?1,?2,?3,?4,?5,?6,?7,'pending',?8,?9,?9,?10)",params![id,owner,instance,project,origin,origin_ref,revision,data.to_string(),now,now+RETENTION]).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(data)
}
pub(crate) fn respond(
    conn: &Connection,
    id: &str,
    revision: &str,
    command: &str,
    actor: &str,
    answers: &Value,
    now: i64,
) -> Result<Value, String> {
    if !uuid(id) || !uuid(command) || actor.is_empty() {
        return Err("invalid_request".into());
    }
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(storage)?;
    let prior: Option<String> = tx
        .query_row(
            "SELECT id FROM user_input_requests WHERE command_id=?1",
            [command],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    if let Some(prior) = prior {
        let row = get(&tx, &prior)?.ok_or("storage_unavailable")?;
        if prior != id
            || row["actor"] != actor
            || row["revision"] != revision
            || row["answers"] != *answers
        {
            return Err("envelope_mismatch".into());
        }
        if row["expires_at"].as_i64().is_none_or(|t| t <= now) {
            return Err("idempotency_expired".into());
        }
        tx.commit().map_err(storage)?;
        return Ok(row);
    }
    let mut row = get(&tx, id)?.ok_or("question_unavailable")?;
    if row["revision"] != revision {
        return Err("question_changed".into());
    }
    if row["expires_at"].as_i64().is_none_or(|t| t <= now) {
        return Err("question_expired".into());
    }
    if row["state"] != "pending" {
        return Err("question_already_resolved".into());
    }
    validate_answers(&row["questions"], answers)?;
    row["state"] = json!("submitted");
    row["command_id"] = json!(command);
    row["actor"] = json!(actor);
    row["answers"] = answers.clone();
    row["updated_at"] = json!(now);
    if tx.execute("UPDATE user_input_requests SET state='submitted',command_id=?1,actor=?2,data=?3,updated_at=?4 WHERE id=?5 AND state='pending'",params![command,actor,row.to_string(),now,id]).map_err(storage)?!=1 {return Err("question_already_resolved".into());}
    tx.commit().map_err(storage)?;
    Ok(row)
}
pub(crate) fn has_active(conn: &Connection, now: i64) -> Result<bool, String> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM user_input_requests WHERE state IN ('submitted','unknown') OR (state='pending' AND expires_at>?1))",[now],|r|r.get(0)).map_err(storage)
}
pub(crate) fn list_page(
    conn: &Connection,
    after: Option<(i64, &str)>,
    now: i64,
    limit: usize,
) -> Result<Vec<Value>, String> {
    if !(1..=100).contains(&limit) {
        return Err("invalid_request".into());
    }
    expire(conn, now)?;
    let mut stmt=conn.prepare("SELECT data FROM user_input_requests WHERE state IN ('pending','submitted','unknown') AND (created_at>?1 OR (created_at=?1 AND id>?2)) ORDER BY created_at,id LIMIT ?3").map_err(storage)?;
    let (t, id) = after.unwrap_or((-1, ""));
    let rows = stmt
        .query_map(params![t, id, limit as i64], |r| r.get::<_, String>(0))
        .map_err(storage)?;
    rows.map(|v| {
        v.map_err(storage)
            .and_then(|s| serde_json::from_str(&s).map_err(|_| "storage_unavailable".into()))
    })
    .collect()
}
pub(crate) fn list(
    conn: &Connection,
    owner: Option<&str>,
    now: i64,
    limit: usize,
) -> Result<Vec<Value>, String> {
    if !(1..=100).contains(&limit) {
        return Err("invalid_request".into());
    }
    expire(conn, now)?;
    let mut stmt=conn.prepare("SELECT data FROM user_input_requests WHERE (?1 IS NULL OR owner=?1) AND state IN ('pending','submitted','unknown') ORDER BY created_at,id LIMIT ?2").map_err(storage)?;
    let rows = stmt
        .query_map(params![owner, limit as i64], |r| r.get::<_, String>(0))
        .map_err(storage)?;
    rows.map(|r| {
        r.map_err(storage)
            .and_then(|s| serde_json::from_str(&s).map_err(|_| "storage_unavailable".into()))
    })
    .collect()
}
pub(crate) fn finish(
    conn: &Connection,
    id: &str,
    owner: &str,
    instance: &str,
    command: Option<&str>,
    state: &str,
    now: i64,
) -> Result<Value, String> {
    if !matches!(
        state,
        "applied" | "resolved_elsewhere" | "expired" | "cancelled" | "unknown" | "closed"
    ) {
        return Err("invalid_request".into());
    }
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate).map_err(storage)?;
    let mut row = get(&tx, id)?.ok_or("question_unavailable")?;
    if row["owner"] != owner || row["instance"] != instance {
        return Err("question_unavailable".into());
    }
    if state == "applied"
        && (command.is_none()
            || row["command_id"].as_str() != command
            || !matches!(row["state"].as_str(), Some("submitted" | "applied")))
    {
        return Err("response_unconfirmed".into());
    }
    if matches!(
        row["state"].as_str(),
        Some("applied" | "resolved_elsewhere" | "expired" | "cancelled" | "closed")
    ) {
        if row["state"] != state {
            return Err("question_already_resolved".into());
        }
        tx.commit().map_err(storage)?;
        return Ok(row);
    }
    row["state"] = json!(state);
    row["updated_at"] = json!(now);
    tx.execute(
        "UPDATE user_input_requests SET state=?1,data=?2,updated_at=?3 WHERE id=?4",
        params![state, row.to_string(), now, id],
    )
    .map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(row)
}
/// Projection publique : ni callback fournisseur, ni instance, ni acteur humain étranger.
pub(crate) fn projection(mut row: Value) -> Value {
    if let Some(v) = row.as_object_mut() {
        for k in ["instance", "callback", "actor"] {
            v.remove(k);
        }
    }
    row
}

#[cfg(test)]
#[path = "user_input_tests.rs"]
mod tests;
