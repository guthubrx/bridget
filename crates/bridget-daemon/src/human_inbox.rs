//! Boîte de réception humaine (SPEC-087).
//!
//! Table transport, au même titre que le guichet : le daemon et le service compagnon y
//! déposent, seul le référent tranche, le producteur relève les décisions
//! sans rien marquer et n'acquitte qu'après avoir appliqué durablement. Un
//! plantage entre relève et application relit la même décision.

use bridget_transport::protocol::{
    HumanInboxDecision, HumanInboxItemFrame, HumanInboxKind, HumanInboxListFilter,
    HumanInboxPendingDecision, HumanInboxProducer, HumanInboxRefusal, HumanInboxState,
    HumanInboxSubject,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

/// Un contexte est un résumé et quelques faits, pas un journal.
pub const MAX_CONTEXT_BYTES: usize = 16 * 1024;
/// Résumé poussé vers le canal externe, borné pour ne pas fuir un corps entier.
pub const MAX_SUMMARY_CHARS: usize = 400;
/// Historique des tentatives de notification conservé par item.
pub const MAX_NOTIFY_ATTEMPTS: usize = 20;
pub const DEFAULT_REMINDER_AFTER_SECS: i64 = 3600;

fn sql_literal<T: Serialize>(value: T) -> String {
    let wire = serde_json::to_string(&value).unwrap_or_default();
    wire.trim_matches('"').to_string()
}

fn state_in_clause() -> String {
    let quoted: Vec<String> = [
        HumanInboxState::Open,
        HumanInboxState::Resolved,
        HumanInboxState::ClosedSelf,
    ]
    .into_iter()
    .map(|state| format!("'{}'", sql_literal(state)))
    .collect();
    format!("IN ({})", quoted.join(", "))
}

fn producer_in_clause() -> String {
    let quoted: Vec<String> = [HumanInboxProducer::Daemon, HumanInboxProducer::Guichet]
        .into_iter()
        .map(|producer| format!("'{}'", sql_literal(producer)))
        .collect();
    format!("IN ({})", quoted.join(", "))
}

pub fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(&format!(
        "CREATE TABLE IF NOT EXISTS human_inbox (
            id TEXT PRIMARY KEY,
            dedup_key TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind {kinds}),
            subject_json TEXT NOT NULL,
            context_json TEXT NOT NULL,
            options_json TEXT NOT NULL,
            state TEXT NOT NULL CHECK (state {states}),
            producer TEXT NOT NULL CHECK (producer {producers}),
            created_at INTEGER NOT NULL,
            resolved_at INTEGER,
            occurrences INTEGER NOT NULL DEFAULT 1,
            decision_json TEXT,
            acked_at INTEGER,
            notify_attempts_json TEXT NOT NULL DEFAULT '[]',
            last_reminded_at INTEGER
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_human_inbox_open_dedup
            ON human_inbox(dedup_key) WHERE state = 'open';",
        kinds = HumanInboxKind::sql_in_clause(),
        states = state_in_clause(),
        producers = producer_in_clause(),
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepositRequest<'a> {
    pub dedup_key: &'a str,
    pub kind: HumanInboxKind,
    pub subject: &'a HumanInboxSubject,
    pub context: &'a str,
    pub options: &'a [String],
    pub producer: HumanInboxProducer,
    pub now: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deposited {
    pub item_id: String,
    pub created: bool,
    pub occurrences: u32,
}

/// Dépose un item, ou rattache une occurrence à l'item ouvert de même clé.
pub fn deposit(
    conn: &Connection,
    request: DepositRequest<'_>,
) -> rusqlite::Result<Result<Deposited, HumanInboxRefusal>> {
    if request.dedup_key.trim().is_empty()
        || request.context.len() > MAX_CONTEXT_BYTES
        || request.options.is_empty()
        || request
            .options
            .iter()
            .any(|option| option.trim().is_empty())
    {
        return Ok(Err(HumanInboxRefusal::InvalidRequest));
    }
    let tx = conn.unchecked_transaction()?;
    let existing: Option<(String, i64)> = tx
        .query_row(
            "SELECT id, occurrences FROM human_inbox WHERE dedup_key = ?1 AND state = 'open'",
            params![request.dedup_key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let deposited = if let Some((item_id, occurrences)) = existing {
        tx.execute(
            "UPDATE human_inbox SET occurrences = occurrences + 1 WHERE id = ?1",
            params![item_id],
        )?;
        Deposited {
            item_id,
            created: false,
            occurrences: (occurrences + 1).max(1) as u32,
        }
    } else {
        let item_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO human_inbox (
                id, dedup_key, kind, subject_json, context_json, options_json, state,
                producer, created_at, occurrences
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', ?7, ?8, 1)",
            params![
                item_id,
                request.dedup_key,
                request.kind.as_sql(),
                serde_json::to_string(request.subject).unwrap_or_else(|_| "{}".to_string()),
                request.context,
                serde_json::to_string(request.options).unwrap_or_else(|_| "[]".to_string()),
                sql_literal(request.producer),
                request.now,
            ],
        )?;
        Deposited {
            item_id,
            created: true,
            occurrences: 1,
        }
    };
    tx.commit()?;
    Ok(Ok(deposited))
}

fn row_to_frame(row: &rusqlite::Row<'_>) -> rusqlite::Result<HumanInboxItemFrame> {
    let kind: String = row.get(2)?;
    let subject_json: String = row.get(3)?;
    let options_json: String = row.get(5)?;
    let state: String = row.get(6)?;
    let producer: String = row.get(7)?;
    let decision_json: Option<String> = row.get(11)?;
    Ok(HumanInboxItemFrame {
        id: row.get(0)?,
        dedup_key: row.get(1)?,
        kind: HumanInboxKind::from_sql(&kind).unwrap_or(HumanInboxKind::InterventionRequired),
        subject: serde_json::from_str(&subject_json).unwrap_or_default(),
        context: row.get(4)?,
        options: serde_json::from_str(&options_json).unwrap_or_default(),
        state: serde_json::from_value(serde_json::Value::String(state))
            .unwrap_or(HumanInboxState::Open),
        producer: serde_json::from_value(serde_json::Value::String(producer))
            .unwrap_or(HumanInboxProducer::Daemon),
        created_at: row.get(8)?,
        resolved_at: row.get(9)?,
        occurrences: row.get::<_, i64>(10)?.max(0) as u32,
        decision: decision_json.and_then(|json| serde_json::from_str(&json).ok()),
        acked_at: row.get(12)?,
    })
}

const SELECT_ITEM: &str = "SELECT id, dedup_key, kind, subject_json, context_json, options_json,
        state, producer, created_at, resolved_at, occurrences, decision_json, acked_at
 FROM human_inbox";

pub fn get(conn: &Connection, item_id: &str) -> rusqlite::Result<Option<HumanInboxItemFrame>> {
    conn.query_row(
        &format!("{SELECT_ITEM} WHERE id = ?1"),
        params![item_id],
        row_to_frame,
    )
    .optional()
}

pub fn list(
    conn: &Connection,
    filter: HumanInboxListFilter,
    limit: u32,
) -> rusqlite::Result<(Vec<HumanInboxItemFrame>, u32)> {
    let limit = i64::from(limit.clamp(1, 500));
    let items = match filter {
        HumanInboxListFilter::Open => {
            let mut statement = conn.prepare(&format!(
                "{SELECT_ITEM} WHERE state = 'open' ORDER BY created_at DESC LIMIT ?1"
            ))?;
            let rows = statement.query_map(params![limit], row_to_frame)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
        HumanInboxListFilter::All => {
            let mut statement =
                conn.prepare(&format!("{SELECT_ITEM} ORDER BY created_at DESC LIMIT ?1"))?;
            let rows = statement.query_map(params![limit], row_to_frame)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
    };
    let open_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM human_inbox WHERE state = 'open'",
        [],
        |row| row.get(0),
    )?;
    Ok((items, open_count.max(0) as u32))
}

pub fn open_count(conn: &Connection) -> rusqlite::Result<u32> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM human_inbox WHERE state = 'open'",
        [],
        |row| row.get(0),
    )?;
    Ok(count.max(0) as u32)
}

/// Le référent tranche. Le choix doit appartenir aux options offertes.
pub fn resolve(
    conn: &Connection,
    item_id: &str,
    choice: &str,
    actor: &str,
    now: i64,
) -> rusqlite::Result<Result<HumanInboxItemFrame, HumanInboxRefusal>> {
    let tx = conn.unchecked_transaction()?;
    let Some(item) = get(&tx, item_id)? else {
        return Ok(Err(HumanInboxRefusal::UnknownItem));
    };
    if item.state != HumanInboxState::Open {
        return Ok(Err(HumanInboxRefusal::AlreadyResolved));
    }
    if !item.options.iter().any(|option| option == choice) {
        return Ok(Err(HumanInboxRefusal::ChoiceNotOffered));
    }
    let decision = HumanInboxDecision {
        decision_id: Uuid::new_v4().to_string(),
        choice: choice.to_string(),
        actor: actor.to_string(),
        at: now,
    };
    tx.execute(
        "UPDATE human_inbox SET state = 'resolved', resolved_at = ?1, decision_json = ?2
         WHERE id = ?3",
        params![
            now,
            serde_json::to_string(&decision).unwrap_or_default(),
            item_id
        ],
    )?;
    let updated = get(&tx, item_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    tx.commit()?;
    Ok(Ok(updated))
}

/// Décisions résolues et non acquittées d'un producteur. Ne modifie rien.
pub fn pending_decisions(
    conn: &Connection,
    producer: HumanInboxProducer,
    limit: u32,
) -> rusqlite::Result<Vec<HumanInboxPendingDecision>> {
    let mut statement = conn.prepare(&format!(
        "{SELECT_ITEM} WHERE state = 'resolved' AND acked_at IS NULL AND producer = ?1
         ORDER BY resolved_at ASC LIMIT ?2"
    ))?;
    let rows = statement.query_map(
        params![sql_literal(producer), i64::from(limit.clamp(1, 200))],
        row_to_frame,
    )?;
    let mut decisions = Vec::new();
    for item in rows {
        let item = item?;
        if let Some(decision) = item.decision {
            decisions.push(HumanInboxPendingDecision {
                decision_id: decision.decision_id,
                item_id: item.id,
                dedup_key: item.dedup_key,
                kind: item.kind,
                subject: item.subject,
                choice: decision.choice,
                at: decision.at,
            });
        }
    }
    Ok(decisions)
}

/// Acquitte après application durable. Idempotent : rend l'horodatage déjà
/// posé le cas échéant, `None` si la décision est inconnue.
pub fn ack(conn: &Connection, decision_id: &str, now: i64) -> rusqlite::Result<Option<i64>> {
    let tx = conn.unchecked_transaction()?;
    let found: Option<(String, Option<i64>)> = tx
        .query_row(
            "SELECT id, acked_at FROM human_inbox
             WHERE state = 'resolved' AND json_extract(decision_json, '$.decision_id') = ?1",
            params![decision_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((item_id, acked_at)) = found else {
        return Ok(None);
    };
    if let Some(acked_at) = acked_at {
        return Ok(Some(acked_at));
    }
    tx.execute(
        "UPDATE human_inbox SET acked_at = ?1 WHERE id = ?2",
        params![now, item_id],
    )?;
    tx.commit()?;
    Ok(Some(now))
}

/// Le producteur ferme un item dont l'objet a disparu.
pub fn close_self(
    conn: &Connection,
    item_id: &str,
    reason: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE human_inbox SET state = 'closed_self', resolved_at = ?1,
                context_json = json_set(context_json, '$.closed_reason', ?2)
         WHERE id = ?3 AND state = 'open'",
        params![now, reason, item_id],
    )?;
    Ok(changed == 1)
}

/// Items ouverts depuis plus de `after_secs` et jamais rappelés dans cette
/// fenêtre.
pub fn due_for_reminder(
    conn: &Connection,
    now: i64,
    after_secs: i64,
) -> rusqlite::Result<Vec<HumanInboxItemFrame>> {
    let threshold = now.saturating_sub(after_secs);
    let mut statement = conn.prepare(&format!(
        "{SELECT_ITEM} WHERE state = 'open' AND created_at <= ?1
         AND (last_reminded_at IS NULL OR last_reminded_at <= ?1)
         ORDER BY created_at ASC LIMIT 50"
    ))?;
    let rows = statement.query_map(params![threshold], row_to_frame)?;
    rows.collect()
}

pub fn mark_reminded(conn: &Connection, item_id: &str, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE human_inbox SET last_reminded_at = ?1 WHERE id = ?2",
        params![now, item_id],
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotifyAttempt {
    pub at: i64,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn record_notify_attempt(
    conn: &Connection,
    item_id: &str,
    attempt: &NotifyAttempt,
) -> rusqlite::Result<()> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT notify_attempts_json FROM human_inbox WHERE id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .optional()?;
    let mut attempts: Vec<NotifyAttempt> = existing
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();
    attempts.push(attempt.clone());
    if attempts.len() > MAX_NOTIFY_ATTEMPTS {
        let excess = attempts.len() - MAX_NOTIFY_ATTEMPTS;
        attempts.drain(..excess);
    }
    conn.execute(
        "UPDATE human_inbox SET notify_attempts_json = ?1 WHERE id = ?2",
        params![
            serde_json::to_string(&attempts).unwrap_or_else(|_| "[]".to_string()),
            item_id
        ],
    )?;
    Ok(())
}

pub fn notify_attempts(conn: &Connection, item_id: &str) -> rusqlite::Result<Vec<NotifyAttempt>> {
    let json: Option<String> = conn
        .query_row(
            "SELECT notify_attempts_json FROM human_inbox WHERE id = ?1",
            params![item_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(json
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

/// Canal externe personnel : une commande configurée, jamais un client réseau
/// dans le daemon. Fichier 0600, chemin absolu obligatoire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanChannelConfig {
    pub command: Vec<String>,
    #[serde(default = "default_reminder_after_secs")]
    pub reminder_after_secs: i64,
}

fn default_reminder_after_secs() -> i64 {
    DEFAULT_REMINDER_AFTER_SECS
}

pub fn default_channel_config_path() -> Option<PathBuf> {
    crate::environment::Namespace::from_environment()
        .ok()
        .map(|namespace| namespace.root.join("human-channel.json"))
}

/// Charge la configuration si elle existe et est valide ; `None` sinon, avec
/// la raison dans le journal. L'absence de fichier n'est pas une erreur.
pub fn load_channel_config(path: &Path) -> Result<Option<HumanChannelConfig>, String> {
    crate::environment::validate_state_file(path, false)?;
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .permissions()
            .mode()
            & 0o777;
        if mode & 0o077 != 0 {
            return Err(format!(
                "{}: permissions {mode:o} trop larges, 0600 attendu",
                path.display()
            ));
        }
    }
    let config: HumanChannelConfig =
        serde_json::from_str(&raw).map_err(|error| format!("{}: {error}", path.display()))?;
    let Some(program) = config.command.first() else {
        return Err(format!("{}: commande vide", path.display()));
    };
    if !Path::new(program).is_absolute() {
        return Err(format!(
            "{}: la commande doit être un chemin absolu",
            path.display()
        ));
    }
    if config.reminder_after_secs < 60 {
        return Err(format!(
            "{}: reminder_after_secs doit valoir au moins 60",
            path.display()
        ));
    }
    Ok(Some(config))
}

/// Résumé borné poussé sur le canal externe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelSummary {
    pub item_id: String,
    pub kind: HumanInboxKind,
    pub summary: String,
    pub options: Vec<String>,
    pub open_count: u32,
    pub reminder: bool,
}

pub fn channel_summary(
    item: &HumanInboxItemFrame,
    open_count: u32,
    reminder: bool,
) -> ChannelSummary {
    let summary_source = serde_json::from_str::<serde_json::Value>(&item.context)
        .ok()
        .and_then(|value| {
            value
                .get("summary")
                .and_then(|summary| summary.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| item.kind.as_sql().replace('_', " "));
    ChannelSummary {
        item_id: item.id.clone(),
        kind: item.kind,
        summary: summary_source.chars().take(MAX_SUMMARY_CHARS).collect(),
        options: item.options.clone(),
        open_count,
        reminder,
    }
}

/// Lance la commande du canal avec le résumé JSON sur stdin. Ne bloque jamais
/// le dépôt : tout échec devient une tentative consignée.
pub fn push_to_channel(
    config: &HumanChannelConfig,
    summary: &ChannelSummary,
) -> Result<(), String> {
    let payload = serde_json::to_vec(summary).map_err(|error| error.to_string())?;
    let mut child = Command::new(&config.command[0])
        .args(&config.command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("lancement: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&payload)
            .map_err(|error| format!("stdin: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("attente: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "sortie {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.chars().take(200).collect::<String>()
        ))
    }
}

/// Dépose puis pousse vers le canal si un item a été créé ; l'échec de la
/// poussée est consigné et jamais propagé.
pub fn deposit_and_notify(
    conn: &Connection,
    request: DepositRequest<'_>,
    channel: Option<&HumanChannelConfig>,
) -> rusqlite::Result<Result<Deposited, HumanInboxRefusal>> {
    let outcome = deposit(conn, request.clone())?;
    if let Ok(deposited) = &outcome
        && deposited.created
        && let Some(config) = channel
        && let Some(item) = get(conn, &deposited.item_id)?
    {
        let count = open_count(conn)?;
        let result = push_to_channel(config, &channel_summary(&item, count, false));
        record_notify_attempt(
            conn,
            &deposited.item_id,
            &NotifyAttempt {
                at: request.now,
                ok: result.is_ok(),
                error: result.err(),
            },
        )?;
    }
    Ok(outcome)
}

/// Rappel des items ouverts depuis trop longtemps. Rend le nombre d'items
/// rappelés ; sans canal configuré, ne fait rien.
pub fn remind_overdue(
    conn: &Connection,
    channel: Option<&HumanChannelConfig>,
    now: i64,
) -> rusqlite::Result<usize> {
    let Some(config) = channel else {
        return Ok(0);
    };
    let due = due_for_reminder(conn, now, config.reminder_after_secs)?;
    let count = open_count(conn)?;
    let mut reminded = 0;
    for item in due {
        let result = push_to_channel(config, &channel_summary(&item, count, true));
        record_notify_attempt(
            conn,
            &item.id,
            &NotifyAttempt {
                at: now,
                ok: result.is_ok(),
                error: result.err(),
            },
        )?;
        mark_reminded(conn, &item.id, now)?;
        reminded += 1;
    }
    Ok(reminded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        conn
    }

    fn subject() -> HumanInboxSubject {
        HumanInboxSubject {
            objective_id: Some("o1".to_string()),
            delegation_id: Some("d1".to_string()),
            ..HumanInboxSubject::default()
        }
    }

    fn request<'a>(subject: &'a HumanInboxSubject, options: &'a [String]) -> DepositRequest<'a> {
        DepositRequest {
            dedup_key: "chain-exhausted:d1",
            kind: HumanInboxKind::ChainExhausted,
            subject,
            context: r#"{"summary":"chaîne épuisée","attempts":2}"#,
            options,
            producer: HumanInboxProducer::Guichet,
            now: 100,
        }
    }

    #[test]
    fn schema_check_derive_des_enums() {
        let conn = conn();
        let ddl: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = 'human_inbox'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        for kind in HumanInboxKind::ALL {
            assert!(ddl.contains(&format!("'{}'", kind.as_sql())), "{ddl}");
        }
        assert!(
            ddl.contains("'closed_self'") && ddl.contains("'guichet'"),
            "{ddl}"
        );
    }

    #[test]
    fn depot_dedoublonne_parmi_les_items_ouverts() {
        let conn = conn();
        let subject = subject();
        let options = vec!["reassign:a1".to_string(), "cancel".to_string()];
        let first = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(first.created);
        let second = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(!second.created);
        assert_eq!(second.item_id, first.item_id);
        assert_eq!(second.occurrences, 2);
        let (items, open) = list(&conn, HumanInboxListFilter::Open, 10).unwrap();
        assert_eq!(open, 1);
        assert_eq!(items[0].occurrences, 2);
        assert_eq!(items[0].kind, HumanInboxKind::ChainExhausted);
        assert_eq!(items[0].subject, subject);
    }

    #[test]
    fn depot_invalide_refuse() {
        let conn = conn();
        let subject = subject();
        let empty: Vec<String> = Vec::new();
        assert_eq!(
            deposit(&conn, request(&subject, &empty))
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::InvalidRequest
        );
        let options = vec!["ack".to_string()];
        let big = "x".repeat(MAX_CONTEXT_BYTES + 1);
        let refused = deposit(
            &conn,
            DepositRequest {
                context: &big,
                ..request(&subject, &options)
            },
        )
        .unwrap()
        .unwrap_err();
        assert_eq!(refused, HumanInboxRefusal::InvalidRequest);
    }

    #[test]
    fn resolution_verifie_options_et_etat() {
        let conn = conn();
        let subject = subject();
        let options = vec!["reassign:a1".to_string(), "cancel".to_string()];
        let deposited = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert_eq!(
            resolve(&conn, "absent", "cancel", "humain", 200)
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::UnknownItem
        );
        assert_eq!(
            resolve(&conn, &deposited.item_id, "raise_budget", "humain", 200)
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::ChoiceNotOffered
        );
        let resolved = resolve(&conn, &deposited.item_id, "cancel", "humain", 200)
            .unwrap()
            .unwrap();
        assert_eq!(resolved.state, HumanInboxState::Resolved);
        assert_eq!(resolved.decision.as_ref().unwrap().choice, "cancel");
        assert_eq!(
            resolve(&conn, &deposited.item_id, "cancel", "humain", 201)
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::AlreadyResolved
        );
        // Une nouvelle occurrence après résolution crée un item neuf.
        let again = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(again.created);
        assert_ne!(again.item_id, deposited.item_id);
    }

    #[test]
    fn decision_relue_jusqu_a_l_acquittement() {
        let conn = conn();
        let subject = subject();
        let options = vec!["ack".to_string()];
        let deposited = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(
            pending_decisions(&conn, HumanInboxProducer::Guichet, 10)
                .unwrap()
                .is_empty()
        );
        let resolved = resolve(&conn, &deposited.item_id, "ack", "humain", 200)
            .unwrap()
            .unwrap();
        let decision_id = resolved.decision.unwrap().decision_id;
        let first = pending_decisions(&conn, HumanInboxProducer::Guichet, 10).unwrap();
        let second = pending_decisions(&conn, HumanInboxProducer::Guichet, 10).unwrap();
        assert_eq!(first.len(), 1, "la relève ne marque rien");
        assert_eq!(first, second);
        assert_eq!(first[0].decision_id, decision_id);
        assert!(
            pending_decisions(&conn, HumanInboxProducer::Daemon, 10)
                .unwrap()
                .is_empty()
        );
        assert_eq!(ack(&conn, &decision_id, 300).unwrap(), Some(300));
        assert_eq!(
            ack(&conn, &decision_id, 400).unwrap(),
            Some(300),
            "idempotent"
        );
        assert_eq!(ack(&conn, "inconnue", 400).unwrap(), None);
        assert!(
            pending_decisions(&conn, HumanInboxProducer::Guichet, 10)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            get(&conn, &deposited.item_id).unwrap().unwrap().acked_at,
            Some(300)
        );
    }

    #[test]
    fn fermeture_par_le_producteur_et_rappel() {
        let conn = conn();
        let subject = subject();
        let options = vec!["ack".to_string()];
        let deposited = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(
            due_for_reminder(&conn, 100 + 3599, 3600)
                .unwrap()
                .is_empty()
        );
        assert_eq!(due_for_reminder(&conn, 100 + 3600, 3600).unwrap().len(), 1);
        mark_reminded(&conn, &deposited.item_id, 100 + 3600).unwrap();
        assert!(
            due_for_reminder(&conn, 100 + 3601, 3600)
                .unwrap()
                .is_empty()
        );
        assert!(close_self(&conn, &deposited.item_id, "objet disparu", 500).unwrap());
        assert!(!close_self(&conn, &deposited.item_id, "objet disparu", 501).unwrap());
        let item = get(&conn, &deposited.item_id).unwrap().unwrap();
        assert_eq!(item.state, HumanInboxState::ClosedSelf);
        assert!(item.context.contains("closed_reason"));
        assert_eq!(open_count(&conn).unwrap(), 0);
    }

    #[test]
    fn canal_externe_commande_factice_et_echec_consignes() {
        let conn = conn();
        let subject = subject();
        let options = vec!["ack".to_string()];
        let dir = std::env::temp_dir().join(format!("hi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sink = dir.join("recu.json");
        let ok_config = HumanChannelConfig {
            command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                format!("cat > {}", sink.display()),
            ],
            reminder_after_secs: 3600,
        };
        let deposited = deposit_and_notify(&conn, request(&subject, &options), Some(&ok_config))
            .unwrap()
            .unwrap();
        let received: ChannelSummary =
            serde_json::from_str(&std::fs::read_to_string(&sink).unwrap()).unwrap();
        assert_eq!(received.item_id, deposited.item_id);
        assert_eq!(received.summary, "chaîne épuisée");
        assert!(!received.reminder);
        let attempts = notify_attempts(&conn, &deposited.item_id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert!(attempts[0].ok);

        let failing = HumanChannelConfig {
            command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "exit 3".to_string(),
            ],
            reminder_after_secs: 3600,
        };
        let reminded = remind_overdue(&conn, Some(&failing), 100 + 7200).unwrap();
        assert_eq!(reminded, 1);
        let attempts = notify_attempts(&conn, &deposited.item_id).unwrap();
        assert_eq!(attempts.len(), 2);
        assert!(!attempts[1].ok);
        assert!(
            attempts[1]
                .error
                .as_deref()
                .unwrap_or("")
                .contains("sortie 3")
        );
        assert_eq!(
            open_count(&conn).unwrap(),
            1,
            "l'échec du canal ne perd rien"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn configuration_du_canal_exige_0600_et_chemin_absolu() {
        let dir = std::env::temp_dir().join(format!("hc-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("human-channel.json");
        assert_eq!(
            load_channel_config(&path).unwrap(),
            None,
            "absent = aucun canal"
        );
        std::fs::write(&path, r#"{"command":["/usr/bin/true"]}"#).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
            // Seules les permissions changent : un refus ne doit pas dépendre
            // du libellé humain ni être causé par une commande déjà invalide.
            assert!(load_channel_config(&path).is_err());
            assert_eq!(
                std::fs::read(&path).unwrap(),
                br#"{"command":["/usr/bin/true"]}"#
            );
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert!(load_channel_config(&path).unwrap().is_some());
        }
        std::fs::write(&path, r#"{"command":["relatif/notify"]}"#).unwrap();
        assert!(load_channel_config(&path).unwrap_err().contains("absolu"));
        std::fs::write(
            &path,
            r#"{"command":["/usr/bin/true"],"reminder_after_secs":30}"#,
        )
        .unwrap();
        assert!(load_channel_config(&path).unwrap_err().contains("60"));
        std::fs::write(&path, r#"{"command":["/usr/bin/true"]}"#).unwrap();
        let config = load_channel_config(&path).unwrap().unwrap();
        assert_eq!(config.reminder_after_secs, DEFAULT_REMINDER_AFTER_SECS);
        let _ = std::fs::remove_dir_all(dir);
    }
}
