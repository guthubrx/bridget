//! Session 102 — SQL des fils inter-agents, sur la connexion `Store` existante.
//!
//! Ce sous-module classe le SQL des six tables de fils ; il ne décide ni des
//! identités ni des codes d'erreur publics (voir `crate::threads`). Chaque
//! opération d'écriture est une transaction `IMMEDIATE` unique : entrée,
//! opération de rejeu et intentions de sollicitation sont commises ensemble
//! ou pas du tout. Aucune E/S fournisseur ni appel réseau ici.
//!
//! Complexités : lecture O(log E + P) par l'index `(thread_id, seq)`,
//! publication O(log E + M) avec M ≤ 16 membres, liste O(log T + L) par
//! l'index `(agent_id, thread_id)`. Aucune requête ne parcourt les corps.

use super::{Store, StoreError};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};

/// Version propre aux fils, distincte du schéma idempotence (`store_schema`).
pub(crate) const THREAD_SCHEMA_VERSION: i64 = 1;

/// Bornes vérifiées dans la transaction ; les valeurs V1 sont fixées par le
/// module métier, jamais par une option de configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ThreadLimits {
    pub max_threads: u64,
    pub max_open_per_creator: u64,
    pub max_entries_per_thread: u64,
    pub max_body_bytes_per_thread: u64,
    pub max_body_bytes_total: u64,
}

/// Cibles effectives demandées par un dépôt, déjà dédupliquées et sans
/// l'auteur ; `All` est résolu dans la transaction sur les membres réels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NotifySpec {
    None,
    Targets(Vec<String>),
    All,
}

impl NotifySpec {
    fn mode(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Targets(_) => "targets",
            Self::All => "all",
        }
    }
}

/// Refus décidés par les données ; le module métier les projette en codes
/// publics sans révéler d'information à un non-membre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ThreadRefusal {
    EnvelopeMismatch,
    ThreadUnavailable,
    ThreadClosed,
    CreatorRequired,
    NotAMember(Vec<String>),
    InvalidReplyReference,
    CapacityExceeded(&'static str),
    ReceiptInvalid,
    ReceiptObsolete,
    CursorConflict,
    RangeUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ThreadTxOutcome {
    /// Résultat frais, déjà persisté pour create/post/close.
    Done(Value),
    /// Rejeu exact d'une opération réussie : résultat sauvegardé.
    Replayed(Value),
    Refused(ThreadRefusal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreadRow {
    pub thread_id: String,
    pub creator_id: String,
    pub title: String,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    pub last_seq: u64,
    pub body_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MemberRow {
    pub agent_id: String,
    pub acked_seq: u64,
    pub last_ack_receipt: Option<String>,
    pub last_ack_through_seq: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WakeRow {
    pub thread_id: String,
    pub agent_id: String,
    pub pending_seq: u64,
    pub generation: u64,
    pub active_through_seq: Option<u64>,
    pub delivery_id: Option<String>,
    pub delivery_key: Option<String>,
    pub recipient_instance_id: Option<String>,
    pub delivery_generation: Option<u64>,
    pub issued_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub state: String,
    pub reason: Option<String>,
    pub dispatched_seq: u64,
    pub last_attempt_at: Option<i64>,
    pub last_uncertain_generation: Option<u64>,
    pub last_uncertain_seq: Option<u64>,
}

impl WakeRow {
    /// Projection publique d'une ligne de sollicitation propre à l'appelant.
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "state": self.state,
            "reason": self.reason,
            "pending_seq": self.pending_seq,
            "generation": self.generation,
            "active_through_seq": self.active_through_seq,
            "dispatched_seq": self.dispatched_seq,
            "last_uncertain_generation": self.last_uncertain_generation,
            "last_uncertain_seq": self.last_uncertain_seq,
        })
    }

    pub(crate) fn none() -> Value {
        WakeRow {
            state: "none".into(),
            ..WakeRow::default()
        }
        .to_json()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreadView {
    pub thread: ThreadRow,
    pub members: Vec<String>,
    pub own: MemberRow,
    pub own_wake: Option<WakeRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreadSummary {
    pub thread_id: String,
    pub title: String,
    pub creator_id: String,
    pub closed: bool,
    pub last_seq: u64,
}

/// Page relue depuis les entrées immuables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Page {
    pub entries: Vec<Value>,
    pub base_seq: u64,
    pub through_seq: u64,
    pub snapshot_seq: u64,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReadOutcome {
    Page {
        page: Page,
        receipt: Option<String>,
        expires_at: Option<i64>,
        requested_limit: u32,
        replayed: bool,
    },
    Refused(ThreadRefusal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AckOutcome {
    Acknowledged {
        acked_seq: u64,
        wake: Option<WakeRow>,
    },
    AlreadyAcknowledged {
        acked_seq: u64,
        wake: Option<WakeRow>,
    },
    Refused(ThreadRefusal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryOutcome {
    Page(Page),
    Refused(ThreadRefusal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateThread<'a> {
    pub actor: &'a str,
    pub operation_id: &'a str,
    pub canonical_hash: &'a str,
    pub thread_id: &'a str,
    pub title: &'a str,
    /// Triés, dédupliqués, créateur inclus.
    pub members: &'a [String],
    pub now: i64,
    pub limits: &'a ThreadLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PostEntry<'a> {
    pub actor: &'a str,
    pub operation_id: &'a str,
    pub canonical_hash: &'a str,
    pub thread_id: &'a str,
    pub message_id: &'a str,
    pub body: &'a str,
    pub notify: &'a NotifySpec,
    pub notices: &'a [&'a str],
    pub reply_to_seq: Option<u64>,
    pub ack_receipt: Option<&'a str>,
    pub now: i64,
    pub limits: &'a ThreadLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CloseThread<'a> {
    pub actor: &'a str,
    pub operation_id: &'a str,
    pub canonical_hash: &'a str,
    pub thread_id: &'a str,
    pub now: i64,
}

/// Lecture bornée des nouveautés propres au membre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadRequest<'a> {
    pub actor: &'a str,
    pub thread_id: &'a str,
    pub limit: u32,
    pub byte_budget: usize,
    pub receipt_id: &'a str,
    pub receipt_ttl_secs: i64,
    pub now: i64,
}

/// Sollicitation prête à partir, sélectionnée par le daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WakeCandidate {
    pub thread_id: String,
    pub agent_id: String,
    pub pending_seq: u64,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WakeReservation<'a> {
    pub thread_id: &'a str,
    pub agent_id: &'a str,
    /// Génération attendue avant réservation (contrôle de course).
    pub expected_generation: u64,
    pub through_seq: u64,
    pub delivery_id: &'a str,
    pub delivery_key: &'a str,
    pub recipient_instance_id: &'a str,
    pub delivery_generation: u64,
    pub issued_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WakeOutcome {
    Dispatched,
    Refused(String),
    OutcomeUnknown,
}

pub(crate) fn ensure_schema(conn: &Connection) -> Result<(), StoreError> {
    // Hors transaction : la pragma est ignorée dans une transaction ouverte.
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(StoreError::Sqlite)?;
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)
        .map_err(StoreError::Sqlite)?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS thread_schema_migrations (
             version INTEGER PRIMARY KEY,
             applied_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS discussion_threads (
             thread_id TEXT PRIMARY KEY,
             creator_id TEXT NOT NULL,
             title TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             closed_at INTEGER,
             last_seq INTEGER NOT NULL DEFAULT 0 CHECK (last_seq >= 0),
             body_bytes INTEGER NOT NULL DEFAULT 0 CHECK (body_bytes >= 0),
             schema_version INTEGER NOT NULL DEFAULT 1
         );
         CREATE INDEX IF NOT EXISTS idx_discussion_threads_creator_open
             ON discussion_threads(creator_id) WHERE closed_at IS NULL;
         CREATE TABLE IF NOT EXISTS discussion_members (
             thread_id TEXT NOT NULL REFERENCES discussion_threads(thread_id),
             agent_id TEXT NOT NULL,
             acked_seq INTEGER NOT NULL DEFAULT 0 CHECK (acked_seq >= 0),
             last_ack_receipt TEXT,
             last_ack_through_seq INTEGER,
             PRIMARY KEY (thread_id, agent_id)
         );
         CREATE INDEX IF NOT EXISTS idx_discussion_members_agent
             ON discussion_members(agent_id, thread_id);
         CREATE TABLE IF NOT EXISTS discussion_entries (
             thread_id TEXT NOT NULL,
             seq INTEGER NOT NULL CHECK (seq > 0),
             message_id TEXT NOT NULL UNIQUE,
             author_id TEXT NOT NULL,
             body TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             notify_json TEXT NOT NULL,
             reply_to_seq INTEGER,
             PRIMARY KEY (thread_id, seq),
             FOREIGN KEY (thread_id, author_id)
                 REFERENCES discussion_members(thread_id, agent_id)
         );
         CREATE TABLE IF NOT EXISTS thread_operations (
             actor_id TEXT NOT NULL,
             operation_id TEXT NOT NULL,
             action TEXT NOT NULL CHECK (action IN ('create', 'post', 'close')),
             thread_id TEXT NOT NULL,
             canonical_hash TEXT NOT NULL,
             result_json TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             PRIMARY KEY (actor_id, operation_id)
         );
         CREATE INDEX IF NOT EXISTS idx_thread_operations_thread
             ON thread_operations(thread_id);
         CREATE TABLE IF NOT EXISTS thread_reads (
             thread_id TEXT NOT NULL,
             agent_id TEXT NOT NULL,
             receipt_id TEXT NOT NULL UNIQUE,
             base_seq INTEGER NOT NULL CHECK (base_seq >= 0),
             through_seq INTEGER NOT NULL CHECK (through_seq > base_seq),
             snapshot_seq INTEGER NOT NULL CHECK (snapshot_seq >= through_seq),
             expires_at INTEGER NOT NULL,
             requested_limit INTEGER NOT NULL,
             PRIMARY KEY (thread_id, agent_id),
             FOREIGN KEY (thread_id, agent_id)
                 REFERENCES discussion_members(thread_id, agent_id)
         );
         CREATE TABLE IF NOT EXISTS thread_wakes (
             thread_id TEXT NOT NULL,
             agent_id TEXT NOT NULL,
             pending_seq INTEGER NOT NULL DEFAULT 0,
             generation INTEGER NOT NULL DEFAULT 0,
             active_through_seq INTEGER,
             delivery_id TEXT,
             delivery_key TEXT,
             recipient_instance_id TEXT,
             delivery_generation INTEGER,
             issued_at INTEGER,
             expires_at INTEGER,
             state TEXT NOT NULL CHECK (state IN (
                 'none', 'pending', 'in_flight', 'dispatched', 'refused',
                 'outcome_unknown', 'satisfied_by_read', 'cancelled'
             )),
             reason TEXT,
             dispatched_seq INTEGER NOT NULL DEFAULT 0,
             last_attempt_at INTEGER,
             last_uncertain_generation INTEGER,
             last_uncertain_seq INTEGER,
             PRIMARY KEY (thread_id, agent_id),
             FOREIGN KEY (thread_id, agent_id)
                 REFERENCES discussion_members(thread_id, agent_id)
         );
         CREATE UNIQUE INDEX IF NOT EXISTS idx_thread_wakes_delivery
             ON thread_wakes(delivery_id) WHERE delivery_id IS NOT NULL;
         CREATE INDEX IF NOT EXISTS idx_thread_wakes_state
             ON thread_wakes(state, last_attempt_at);",
    )
    .map_err(StoreError::Sqlite)?;
    tx.execute(
        "INSERT OR IGNORE INTO thread_schema_migrations (version, applied_at)
         VALUES (?1, strftime('%s','now'))",
        params![THREAD_SCHEMA_VERSION],
    )
    .map_err(StoreError::Sqlite)?;
    tx.commit().map_err(StoreError::Sqlite)?;
    Ok(())
}

/// Préflight lecture seule : tables et index attendus présents.
pub(crate) fn schema_ready(conn: &Connection) -> Result<bool, StoreError> {
    let expected = [
        ("table", "discussion_threads"),
        ("table", "discussion_members"),
        ("table", "discussion_entries"),
        ("table", "thread_operations"),
        ("table", "thread_reads"),
        ("table", "thread_wakes"),
        ("index", "idx_discussion_members_agent"),
        ("index", "idx_thread_wakes_delivery"),
    ];
    for (kind, name) in expected {
        let present: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2)",
                params![kind, name],
                |row| row.get(0),
            )
            .map_err(StoreError::Sqlite)?;
        if !present {
            return Ok(false);
        }
    }
    Ok(true)
}

fn sql(error: rusqlite::Error) -> StoreError {
    StoreError::Sqlite(error)
}

fn u64_of(value: i64) -> u64 {
    value.max(0) as u64
}

fn opt_u64(value: Option<i64>) -> Option<u64> {
    value.map(u64_of)
}

fn load_thread(tx: &Transaction<'_>, thread_id: &str) -> Result<Option<ThreadRow>, StoreError> {
    tx.query_row(
        "SELECT thread_id, creator_id, title, created_at, closed_at, last_seq, body_bytes
         FROM discussion_threads WHERE thread_id = ?1",
        [thread_id],
        |row| {
            Ok(ThreadRow {
                thread_id: row.get(0)?,
                creator_id: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                closed_at: row.get(4)?,
                last_seq: u64_of(row.get(5)?),
                body_bytes: u64_of(row.get(6)?),
            })
        },
    )
    .optional()
    .map_err(sql)
}

fn load_member(
    tx: &Transaction<'_>,
    thread_id: &str,
    agent_id: &str,
) -> Result<Option<MemberRow>, StoreError> {
    tx.query_row(
        "SELECT agent_id, acked_seq, last_ack_receipt, last_ack_through_seq
         FROM discussion_members WHERE thread_id = ?1 AND agent_id = ?2",
        params![thread_id, agent_id],
        |row| {
            Ok(MemberRow {
                agent_id: row.get(0)?,
                acked_seq: u64_of(row.get(1)?),
                last_ack_receipt: row.get(2)?,
                last_ack_through_seq: opt_u64(row.get(3)?),
            })
        },
    )
    .optional()
    .map_err(sql)
}

/// Fil et appartenance de l'acteur en une seule décision : un non-membre et
/// un fil inexistant produisent le même refus, sans information.
pub(crate) fn load_thread_for_member(
    tx: &Transaction<'_>,
    thread_id: &str,
    actor: &str,
) -> Result<Result<(ThreadRow, MemberRow), ThreadRefusal>, StoreError> {
    let Some(thread) = load_thread(tx, thread_id)? else {
        return Ok(Err(ThreadRefusal::ThreadUnavailable));
    };
    let Some(member) = load_member(tx, thread_id, actor)? else {
        return Ok(Err(ThreadRefusal::ThreadUnavailable));
    };
    Ok(Ok((thread, member)))
}

fn load_members(tx: &Transaction<'_>, thread_id: &str) -> Result<Vec<String>, StoreError> {
    let mut statement = tx
        .prepare("SELECT agent_id FROM discussion_members WHERE thread_id = ?1 ORDER BY agent_id")
        .map_err(sql)?;
    let rows = statement
        .query_map([thread_id], |row| row.get::<_, String>(0))
        .map_err(sql)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql)
}

fn wake_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WakeRow> {
    Ok(WakeRow {
        thread_id: row.get(0)?,
        agent_id: row.get(1)?,
        pending_seq: u64_of(row.get(2)?),
        generation: u64_of(row.get(3)?),
        active_through_seq: opt_u64(row.get(4)?),
        delivery_id: row.get(5)?,
        delivery_key: row.get(6)?,
        recipient_instance_id: row.get(7)?,
        delivery_generation: opt_u64(row.get(8)?),
        issued_at: row.get(9)?,
        expires_at: row.get(10)?,
        state: row.get(11)?,
        reason: row.get(12)?,
        dispatched_seq: u64_of(row.get(13)?),
        last_attempt_at: row.get(14)?,
        last_uncertain_generation: opt_u64(row.get(15)?),
        last_uncertain_seq: opt_u64(row.get(16)?),
    })
}

const WAKE_COLUMNS: &str = "thread_id, agent_id, pending_seq, generation, active_through_seq, \
     delivery_id, delivery_key, recipient_instance_id, delivery_generation, issued_at, \
     expires_at, state, reason, dispatched_seq, last_attempt_at, last_uncertain_generation, \
     last_uncertain_seq";

fn load_wake(
    conn: &Connection,
    thread_id: &str,
    agent_id: &str,
) -> Result<Option<WakeRow>, StoreError> {
    conn.query_row(
        &format!("SELECT {WAKE_COLUMNS} FROM thread_wakes WHERE thread_id = ?1 AND agent_id = ?2"),
        params![thread_id, agent_id],
        wake_from_row,
    )
    .optional()
    .map_err(sql)
}

/// Rejeu d'une opération : `Ok(None)` clé neuve, `Ok(Some(Ok(json)))` rejeu
/// exact, `Ok(Some(Err(())))` même clé pour une autre enveloppe.
fn lookup_operation(
    tx: &Transaction<'_>,
    actor: &str,
    operation_id: &str,
    canonical_hash: &str,
) -> Result<Option<Result<Value, ()>>, StoreError> {
    let found: Option<(String, String)> = tx
        .query_row(
            "SELECT canonical_hash, result_json FROM thread_operations
             WHERE actor_id = ?1 AND operation_id = ?2",
            params![actor, operation_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(sql)?;
    Ok(found.map(|(hash, result)| {
        if hash == canonical_hash {
            Ok(serde_json::from_str(&result).unwrap_or(Value::Null))
        } else {
            Err(())
        }
    }))
}

struct OperationRecord<'a> {
    actor: &'a str,
    operation_id: &'a str,
    action: &'static str,
    thread_id: &'a str,
    canonical_hash: &'a str,
    now: i64,
}

fn record_operation(
    tx: &Transaction<'_>,
    record: &OperationRecord<'_>,
    result: &Value,
) -> Result<(), StoreError> {
    tx.execute(
        "INSERT INTO thread_operations
             (actor_id, operation_id, action, thread_id, canonical_hash, result_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            record.actor,
            record.operation_id,
            record.action,
            record.thread_id,
            record.canonical_hash,
            result.to_string(),
            record.now
        ],
    )
    .map_err(sql)?;
    Ok(())
}

fn entry_json(
    seq: u64,
    message_id: &str,
    author_id: &str,
    created_at: i64,
    body: &str,
    notify_json: &str,
    reply_to_seq: Option<u64>,
) -> Value {
    let notify: Value =
        serde_json::from_str(notify_json).unwrap_or_else(|_| json!({"mode":"none","targets":[]}));
    json!({
        "seq": seq,
        "message_id": message_id,
        "author_id": author_id,
        "created_at": created_at,
        "body": body,
        "notify": notify,
        "reply_to_seq": reply_to_seq,
    })
}

/// Lit une plage croissante, bornée en nombre et en octets sérialisés ; ne
/// coupe jamais une entrée. La première entrée est toujours transmise : le
/// dépôt garantit qu'elle tient dans le budget (`entry_too_large` sinon).
fn read_range(
    tx: &Transaction<'_>,
    thread_id: &str,
    from_seq: u64,
    to_seq: u64,
    limit: u32,
    byte_budget: usize,
) -> Result<Vec<Value>, StoreError> {
    if from_seq > to_seq || limit == 0 {
        return Ok(Vec::new());
    }
    let mut statement = tx
        .prepare(
            "SELECT seq, message_id, author_id, created_at, body, notify_json, reply_to_seq
             FROM discussion_entries
             WHERE thread_id = ?1 AND seq >= ?2 AND seq <= ?3
             ORDER BY seq ASC LIMIT ?4",
        )
        .map_err(sql)?;
    let rows = statement
        .query_map(
            params![thread_id, from_seq as i64, to_seq as i64, i64::from(limit)],
            |row| {
                Ok(entry_json(
                    u64_of(row.get(0)?),
                    &row.get::<_, String>(1)?,
                    &row.get::<_, String>(2)?,
                    row.get(3)?,
                    &row.get::<_, String>(4)?,
                    &row.get::<_, String>(5)?,
                    opt_u64(row.get(6)?),
                ))
            },
        )
        .map_err(sql)?;
    let mut entries = Vec::new();
    let mut used = 0usize;
    for entry in rows {
        let entry = entry.map_err(sql)?;
        let size = entry.to_string().len() + 1;
        if !entries.is_empty() && used + size > byte_budget {
            break;
        }
        used += size;
        entries.push(entry);
    }
    Ok(entries)
}

fn last_seq_of(entries: &[Value]) -> Option<u64> {
    entries.last().and_then(|entry| entry["seq"].as_u64())
}

/// Résultat d'une confirmation appliquée dans une transaction déjà ouverte.
enum AckApplied {
    Acknowledged(u64),
    Already(u64),
    Refused(ThreadRefusal),
}

fn apply_ack(
    tx: &Transaction<'_>,
    thread_id: &str,
    member: &MemberRow,
    receipt: &str,
) -> Result<AckApplied, StoreError> {
    if member.last_ack_receipt.as_deref() == Some(receipt) {
        return Ok(AckApplied::Already(member.acked_seq));
    }
    let active: Option<(String, i64, i64)> = tx
        .query_row(
            "SELECT receipt_id, base_seq, through_seq FROM thread_reads
             WHERE thread_id = ?1 AND agent_id = ?2",
            params![thread_id, member.agent_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(sql)?;
    let Some((receipt_id, base_seq, through_seq)) = active else {
        return Ok(AckApplied::Refused(ThreadRefusal::ReceiptInvalid));
    };
    if receipt_id != receipt {
        // Un reçu remplacé par une lecture plus récente n'a plus d'effet ;
        // un reçu inconnu de ce membre non plus. Le premier cas est le seul
        // que la ligne courante permette d'attester comme « obsolète ».
        return Ok(AckApplied::Refused(ThreadRefusal::ReceiptObsolete));
    }
    if u64_of(base_seq) != member.acked_seq {
        return Ok(AckApplied::Refused(ThreadRefusal::CursorConflict));
    }
    let through_seq = u64_of(through_seq);
    tx.execute(
        "UPDATE discussion_members
         SET acked_seq = ?1, last_ack_receipt = ?2, last_ack_through_seq = ?1
         WHERE thread_id = ?3 AND agent_id = ?4",
        params![through_seq as i64, receipt, thread_id, member.agent_id],
    )
    .map_err(sql)?;
    tx.execute(
        "DELETE FROM thread_reads WHERE thread_id = ?1 AND agent_id = ?2",
        params![thread_id, member.agent_id],
    )
    .map_err(sql)?;
    // Une sollicitation couverte par la lecture est satisfaite ; une remise
    // déjà figée au-delà, ou une mention supérieure, restent intactes.
    tx.execute(
        "UPDATE thread_wakes
         SET state = 'satisfied_by_read', reason = NULL
         WHERE thread_id = ?1 AND agent_id = ?2
           AND pending_seq <= ?3
           AND active_through_seq IS NULL
           AND state IN ('pending', 'refused', 'outcome_unknown', 'dispatched')",
        params![thread_id, member.agent_id, through_seq as i64],
    )
    .map_err(sql)?;
    Ok(AckApplied::Acknowledged(through_seq))
}

impl Store {
    pub(crate) fn thread_create(
        &self,
        request: CreateThread<'_>,
    ) -> Result<ThreadTxOutcome, StoreError> {
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        match lookup_operation(
            &tx,
            request.actor,
            request.operation_id,
            request.canonical_hash,
        )? {
            Some(Ok(result)) => return Ok(ThreadTxOutcome::Replayed(result)),
            Some(Err(())) => return Ok(ThreadTxOutcome::Refused(ThreadRefusal::EnvelopeMismatch)),
            None => {}
        }
        let total: i64 = tx
            .query_row("SELECT COUNT(*) FROM discussion_threads", [], |row| {
                row.get(0)
            })
            .map_err(sql)?;
        if u64_of(total) >= request.limits.max_threads {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded(
                "threads",
            )));
        }
        let open_by_creator: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM discussion_threads WHERE creator_id = ?1 AND closed_at IS NULL",
                [request.actor],
                |row| row.get(0),
            )
            .map_err(sql)?;
        if u64_of(open_by_creator) >= request.limits.max_open_per_creator {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded(
                "open_threads_per_creator",
            )));
        }
        tx.execute(
            "INSERT INTO discussion_threads (thread_id, creator_id, title, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![request.thread_id, request.actor, request.title, request.now],
        )
        .map_err(sql)?;
        for member in request.members {
            tx.execute(
                "INSERT INTO discussion_members (thread_id, agent_id) VALUES (?1, ?2)",
                params![request.thread_id, member],
            )
            .map_err(sql)?;
        }
        let result = json!({
            "status": "created",
            "thread_id": request.thread_id,
            "title": request.title,
            "creator_id": request.actor,
            "members": request.members,
            "state": "open",
            "created_at": request.now,
        });
        record_operation(
            &tx,
            &OperationRecord {
                actor: request.actor,
                operation_id: request.operation_id,
                action: "create",
                thread_id: request.thread_id,
                canonical_hash: request.canonical_hash,
                now: request.now,
            },
            &result,
        )?;
        tx.commit().map_err(sql)?;
        Ok(ThreadTxOutcome::Done(result))
    }

    /// Dépôt : entrée, opération et intentions dans une transaction. Un
    /// refus ne laisse aucune mutation, ACK joint compris.
    pub(crate) fn thread_post(
        &self,
        request: PostEntry<'_>,
    ) -> Result<ThreadTxOutcome, StoreError> {
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        match lookup_operation(
            &tx,
            request.actor,
            request.operation_id,
            request.canonical_hash,
        )? {
            Some(Ok(result)) => return Ok(ThreadTxOutcome::Replayed(result)),
            Some(Err(())) => return Ok(ThreadTxOutcome::Refused(ThreadRefusal::EnvelopeMismatch)),
            None => {}
        }
        let (thread, member) = match load_thread_for_member(&tx, request.thread_id, request.actor)?
        {
            Ok(found) => found,
            Err(refusal) => return Ok(ThreadTxOutcome::Refused(refusal)),
        };
        if thread.closed_at.is_some() {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::ThreadClosed));
        }
        if let Some(reply_to) = request.reply_to_seq
            && (reply_to == 0 || reply_to > thread.last_seq)
        {
            return Ok(ThreadTxOutcome::Refused(
                ThreadRefusal::InvalidReplyReference,
            ));
        }
        let members = load_members(&tx, request.thread_id)?;
        let targets: Vec<String> = match request.notify {
            NotifySpec::None => Vec::new(),
            NotifySpec::All => members
                .iter()
                .filter(|member| member.as_str() != request.actor)
                .cloned()
                .collect(),
            NotifySpec::Targets(wanted) => {
                let unknown: Vec<String> = wanted
                    .iter()
                    .filter(|target| !members.contains(target))
                    .cloned()
                    .collect();
                if !unknown.is_empty() {
                    return Ok(ThreadTxOutcome::Refused(ThreadRefusal::NotAMember(unknown)));
                }
                let mut targets: Vec<String> = wanted
                    .iter()
                    .filter(|target| target.as_str() != request.actor)
                    .cloned()
                    .collect();
                targets.sort();
                targets.dedup();
                targets
            }
        };
        if let Some(receipt) = request.ack_receipt {
            match apply_ack(&tx, request.thread_id, &member, receipt)? {
                AckApplied::Acknowledged(_) | AckApplied::Already(_) => {}
                AckApplied::Refused(refusal) => return Ok(ThreadTxOutcome::Refused(refusal)),
            }
        }
        let body_len = request.body.len() as u64;
        if thread.last_seq + 1 > request.limits.max_entries_per_thread {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded(
                "entries_per_thread",
            )));
        }
        if thread.body_bytes + body_len > request.limits.max_body_bytes_per_thread {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded(
                "body_bytes_per_thread",
            )));
        }
        // Somme sur au plus `max_threads` lignes de compteurs, jamais sur les corps.
        let total_bytes: i64 = tx
            .query_row(
                "SELECT COALESCE(SUM(body_bytes), 0) FROM discussion_threads",
                [],
                |row| row.get(0),
            )
            .map_err(sql)?;
        if u64_of(total_bytes) + body_len > request.limits.max_body_bytes_total {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded(
                "body_bytes_total",
            )));
        }
        let seq = thread.last_seq + 1;
        let notify_json = json!({"mode": request.notify.mode(), "targets": targets}).to_string();
        tx.execute(
            "INSERT INTO discussion_entries
                 (thread_id, seq, message_id, author_id, body, created_at, notify_json, reply_to_seq)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                request.thread_id,
                seq as i64,
                request.message_id,
                request.actor,
                request.body,
                request.now,
                notify_json,
                request.reply_to_seq.map(|value| value as i64),
            ],
        )
        .map_err(sql)?;
        tx.execute(
            "UPDATE discussion_threads SET last_seq = ?1, body_bytes = body_bytes + ?2
             WHERE thread_id = ?3",
            params![seq as i64, body_len as i64, request.thread_id],
        )
        .map_err(sql)?;
        let mut wakes = Vec::with_capacity(targets.len());
        for target in &targets {
            tx.execute(
                "INSERT INTO thread_wakes (thread_id, agent_id, pending_seq, state)
                 VALUES (?1, ?2, ?3, 'pending')
                 ON CONFLICT(thread_id, agent_id) DO UPDATE SET
                     pending_seq = MAX(pending_seq, excluded.pending_seq),
                     state = CASE
                         WHEN active_through_seq IS NOT NULL THEN state
                         ELSE 'pending'
                     END,
                     reason = CASE
                         WHEN active_through_seq IS NOT NULL THEN reason
                         ELSE NULL
                     END",
                params![request.thread_id, target, seq as i64],
            )
            .map_err(sql)?;
            // La ligne vient d'être insérée ou mise à jour : son absence serait
            // une incohérence de la base, jamais un état « vide » à publier.
            let wake = load_wake(&tx, request.thread_id, target)?.ok_or(StoreError::Invariant(
                "sollicitation absente après écriture",
            ))?;
            wakes.push(json!({
                "agent_id": target,
                "state": wake.state,
                "reason": wake.reason,
            }));
        }
        let result = json!({
            "status": "posted",
            "thread_id": request.thread_id,
            "message_id": request.message_id,
            "seq": seq,
            "targets": targets,
            "notices": request.notices,
            "wakes": wakes,
        });
        record_operation(
            &tx,
            &OperationRecord {
                actor: request.actor,
                operation_id: request.operation_id,
                action: "post",
                thread_id: request.thread_id,
                canonical_hash: request.canonical_hash,
                now: request.now,
            },
            &result,
        )?;
        tx.commit().map_err(sql)?;
        Ok(ThreadTxOutcome::Done(result))
    }

    pub(crate) fn thread_close(
        &self,
        request: CloseThread<'_>,
    ) -> Result<ThreadTxOutcome, StoreError> {
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        match lookup_operation(
            &tx,
            request.actor,
            request.operation_id,
            request.canonical_hash,
        )? {
            Some(Ok(result)) => return Ok(ThreadTxOutcome::Replayed(result)),
            Some(Err(())) => return Ok(ThreadTxOutcome::Refused(ThreadRefusal::EnvelopeMismatch)),
            None => {}
        }
        let (thread, _) = match load_thread_for_member(&tx, request.thread_id, request.actor)? {
            Ok(found) => found,
            Err(refusal) => return Ok(ThreadTxOutcome::Refused(refusal)),
        };
        if thread.creator_id != request.actor {
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::CreatorRequired));
        }
        if thread.closed_at.is_some() {
            // Nouvelle clé sur un fil déjà clos : aucune ligne d'opération.
            return Ok(ThreadTxOutcome::Refused(ThreadRefusal::ThreadClosed));
        }
        tx.execute(
            "UPDATE discussion_threads SET closed_at = ?1 WHERE thread_id = ?2",
            params![request.now, request.thread_id],
        )
        .map_err(sql)?;
        // Les intentions pas encore parties sont annulées ; un envoi en vol
        // reste tracé jusqu'à son issue.
        tx.execute(
            "UPDATE thread_wakes SET state = 'cancelled', reason = NULL
             WHERE thread_id = ?1 AND active_through_seq IS NULL
               AND state IN ('pending', 'refused')",
            [request.thread_id],
        )
        .map_err(sql)?;
        let result = json!({
            "status": "closed",
            "thread_id": request.thread_id,
            "closed_at": request.now,
        });
        record_operation(
            &tx,
            &OperationRecord {
                actor: request.actor,
                operation_id: request.operation_id,
                action: "close",
                thread_id: request.thread_id,
                canonical_hash: request.canonical_hash,
                now: request.now,
            },
            &result,
        )?;
        tx.commit().map_err(sql)?;
        Ok(ThreadTxOutcome::Done(result))
    }

    /// Fils dont l'appelant est membre, ordre technique UUID, `after` exclusif.
    pub(crate) fn thread_list(
        &self,
        actor: &str,
        limit: u32,
        after: Option<&str>,
    ) -> Result<(Vec<ThreadSummary>, Option<String>), StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.thread_id, t.title, t.creator_id, t.closed_at IS NOT NULL, t.last_seq
                 FROM discussion_members m
                 JOIN discussion_threads t ON t.thread_id = m.thread_id
                 WHERE m.agent_id = ?1 AND (?2 IS NULL OR m.thread_id > ?2)
                 ORDER BY m.thread_id ASC LIMIT ?3",
            )
            .map_err(sql)?;
        let rows = statement
            .query_map(params![actor, after, i64::from(limit) + 1], |row| {
                Ok(ThreadSummary {
                    thread_id: row.get(0)?,
                    title: row.get(1)?,
                    creator_id: row.get(2)?,
                    closed: row.get(3)?,
                    last_seq: u64_of(row.get(4)?),
                })
            })
            .map_err(sql)?;
        let mut threads = rows.collect::<Result<Vec<_>, _>>().map_err(sql)?;
        let next_after = if threads.len() > limit as usize {
            threads.truncate(limit as usize);
            threads.last().map(|thread| thread.thread_id.clone())
        } else {
            None
        };
        Ok((threads, next_after))
    }

    pub(crate) fn thread_show(
        &self,
        actor: &str,
        thread_id: &str,
    ) -> Result<Result<ThreadView, ThreadRefusal>, StoreError> {
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        let (thread, own) = match load_thread_for_member(&tx, thread_id, actor)? {
            Ok(found) => found,
            Err(refusal) => return Ok(Err(refusal)),
        };
        let members = load_members(&tx, thread_id)?;
        let own_wake = load_wake(&tx, thread_id, actor)?;
        Ok(Ok(ThreadView {
            thread,
            members,
            own,
            own_wake,
        }))
    }

    /// Lecture bornée des nouveautés propres au membre, avec reçu.
    pub(crate) fn thread_read(&self, request: &ReadRequest<'_>) -> Result<ReadOutcome, StoreError> {
        let ReadRequest {
            actor,
            thread_id,
            limit,
            byte_budget,
            receipt_id,
            receipt_ttl_secs,
            now,
        } = *request;
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        let (thread, member) = match load_thread_for_member(&tx, thread_id, actor)? {
            Ok(found) => found,
            Err(refusal) => return Ok(ReadOutcome::Refused(refusal)),
        };
        let active: Option<(String, i64, i64, i64, i64, i64)> = tx
            .query_row(
                "SELECT receipt_id, base_seq, through_seq, snapshot_seq, expires_at, requested_limit
                 FROM thread_reads WHERE thread_id = ?1 AND agent_id = ?2",
                params![thread_id, actor],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .optional()
            .map_err(sql)?;
        if let Some((receipt, base_seq, through_seq, snapshot_seq, expires_at, requested_limit)) =
            active
            && expires_at > now
        {
            // Reçu actif : exactement la même plage, sans nouveau reçu.
            let entries = read_range(
                &tx,
                thread_id,
                u64_of(base_seq) + 1,
                u64_of(through_seq),
                u32::MAX,
                usize::MAX,
            )?;
            return Ok(ReadOutcome::Page {
                page: Page {
                    entries,
                    base_seq: u64_of(base_seq),
                    through_seq: u64_of(through_seq),
                    snapshot_seq: u64_of(snapshot_seq),
                    has_more: through_seq < snapshot_seq,
                },
                receipt: Some(receipt),
                expires_at: Some(expires_at),
                requested_limit: requested_limit.clamp(1, i64::from(u32::MAX)) as u32,
                replayed: true,
            });
        }
        let snapshot_seq = thread.last_seq;
        let base_seq = member.acked_seq;
        let entries = read_range(
            &tx,
            thread_id,
            base_seq + 1,
            snapshot_seq,
            limit,
            byte_budget,
        )?;
        let Some(through_seq) = last_seq_of(&entries) else {
            return Ok(ReadOutcome::Page {
                page: Page {
                    entries,
                    base_seq,
                    through_seq: base_seq,
                    snapshot_seq,
                    has_more: false,
                },
                receipt: None,
                expires_at: None,
                requested_limit: limit,
                replayed: false,
            });
        };
        let expires_at = now.saturating_add(receipt_ttl_secs);
        tx.execute(
            "INSERT OR REPLACE INTO thread_reads
                 (thread_id, agent_id, receipt_id, base_seq, through_seq, snapshot_seq, expires_at, requested_limit)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                thread_id,
                actor,
                receipt_id,
                base_seq as i64,
                through_seq as i64,
                snapshot_seq as i64,
                expires_at,
                i64::from(limit)
            ],
        )
        .map_err(sql)?;
        tx.commit().map_err(sql)?;
        Ok(ReadOutcome::Page {
            page: Page {
                entries,
                base_seq,
                through_seq,
                snapshot_seq,
                has_more: through_seq < snapshot_seq,
            },
            receipt: Some(receipt_id.to_string()),
            expires_at: Some(expires_at),
            requested_limit: limit,
            replayed: false,
        })
    }

    pub(crate) fn thread_ack(
        &self,
        actor: &str,
        thread_id: &str,
        receipt: &str,
    ) -> Result<AckOutcome, StoreError> {
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        let (_, member) = match load_thread_for_member(&tx, thread_id, actor)? {
            Ok(found) => found,
            Err(refusal) => return Ok(AckOutcome::Refused(refusal)),
        };
        let applied = apply_ack(&tx, thread_id, &member, receipt)?;
        let outcome = match applied {
            AckApplied::Acknowledged(acked_seq) => {
                let wake = load_wake(&tx, thread_id, actor)?;
                tx.commit().map_err(sql)?;
                AckOutcome::Acknowledged { acked_seq, wake }
            }
            AckApplied::Already(acked_seq) => {
                let wake = load_wake(&tx, thread_id, actor)?;
                AckOutcome::AlreadyAcknowledged { acked_seq, wake }
            }
            AckApplied::Refused(refusal) => AckOutcome::Refused(refusal),
        };
        Ok(outcome)
    }

    /// Relecture explicite d'une plage, sans reçu ni déplacement du curseur.
    pub(crate) fn thread_history(
        &self,
        actor: &str,
        thread_id: &str,
        from_seq: u64,
        to_seq: Option<u64>,
        limit: u32,
        byte_budget: usize,
    ) -> Result<HistoryOutcome, StoreError> {
        let tx = self.conn.unchecked_transaction().map_err(sql)?;
        let (thread, _) = match load_thread_for_member(&tx, thread_id, actor)? {
            Ok(found) => found,
            Err(refusal) => return Ok(HistoryOutcome::Refused(refusal)),
        };
        let snapshot_seq = match to_seq {
            Some(to_seq) if to_seq > thread.last_seq => {
                return Ok(HistoryOutcome::Refused(ThreadRefusal::RangeUnavailable));
            }
            Some(to_seq) => to_seq,
            None => thread.last_seq,
        };
        let entries = read_range(&tx, thread_id, from_seq, snapshot_seq, limit, byte_budget)?;
        let through_seq = last_seq_of(&entries).unwrap_or(from_seq.saturating_sub(1));
        let has_more = !entries.is_empty() && through_seq < snapshot_seq;
        Ok(HistoryOutcome::Page(Page {
            entries,
            base_seq: from_seq.saturating_sub(1),
            through_seq,
            snapshot_seq,
            has_more,
        }))
    }

    pub(crate) fn thread_wake_by_delivery(
        &self,
        delivery_id: &str,
    ) -> Result<Option<WakeRow>, StoreError> {
        self.conn
            .query_row(
                &format!("SELECT {WAKE_COLUMNS} FROM thread_wakes WHERE delivery_id = ?1"),
                [delivery_id],
                wake_from_row,
            )
            .optional()
            .map_err(sql)
    }

    /// Sollicitations à faire partir : mention non couverte par une lecture
    /// confirmée ni par une alerte déjà injectée, fil ouvert, aucune remise
    /// active non échue. Après une issue inconnue échue, seule une mention
    /// strictement supérieure à la borne figée est candidate. Rotation par
    /// dernier essai pour qu'un absent ne bloque pas les autres.
    pub(crate) fn thread_wake_candidates(
        &self,
        now: i64,
        batch: u32,
    ) -> Result<Vec<WakeCandidate>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT w.thread_id, w.agent_id, w.pending_seq, w.generation
                 FROM thread_wakes w
                 JOIN discussion_threads t ON t.thread_id = w.thread_id
                 JOIN discussion_members m ON m.thread_id = w.thread_id AND m.agent_id = w.agent_id
                 WHERE t.closed_at IS NULL
                   AND w.state IN ('pending', 'refused', 'outcome_unknown', 'dispatched')
                   AND w.pending_seq > MAX(m.acked_seq, w.dispatched_seq)
                   AND (
                        w.active_through_seq IS NULL
                        OR (w.state = 'outcome_unknown'
                            AND w.expires_at IS NOT NULL AND w.expires_at <= ?1
                            AND w.pending_seq > w.active_through_seq)
                   )
                 ORDER BY w.last_attempt_at IS NOT NULL, w.last_attempt_at ASC
                 LIMIT ?2",
            )
            .map_err(sql)?;
        let rows = statement
            .query_map(params![now, i64::from(batch)], |row| {
                Ok(WakeCandidate {
                    thread_id: row.get(0)?,
                    agent_id: row.get(1)?,
                    pending_seq: u64_of(row.get(2)?),
                    generation: u64_of(row.get(3)?),
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }

    /// Fige une génération de remise avant toute projection vers le transport.
    /// Renvoie `false` si la ligne a changé entre la sélection et la réservation.
    pub(crate) fn thread_wake_reserve(
        &self,
        reservation: WakeReservation<'_>,
        now: i64,
    ) -> Result<bool, StoreError> {
        let changed = self
            .conn
            .execute(
                "UPDATE thread_wakes SET
                     last_uncertain_generation = CASE
                         WHEN state = 'outcome_unknown' THEN generation
                         ELSE last_uncertain_generation END,
                     last_uncertain_seq = CASE
                         WHEN state = 'outcome_unknown' THEN active_through_seq
                         ELSE last_uncertain_seq END,
                     generation = generation + 1,
                     active_through_seq = ?3,
                     delivery_id = ?4,
                     delivery_key = ?5,
                     recipient_instance_id = ?6,
                     delivery_generation = ?7,
                     issued_at = ?8,
                     expires_at = ?9,
                     state = 'in_flight',
                     reason = NULL,
                     last_attempt_at = ?10
                 WHERE thread_id = ?1 AND agent_id = ?2 AND generation = ?11
                   AND (active_through_seq IS NULL
                        OR (state = 'outcome_unknown' AND expires_at <= ?10))",
                params![
                    reservation.thread_id,
                    reservation.agent_id,
                    reservation.through_seq as i64,
                    reservation.delivery_id,
                    reservation.delivery_key,
                    reservation.recipient_instance_id,
                    reservation.delivery_generation as i64,
                    reservation.issued_at,
                    reservation.expires_at,
                    now,
                    reservation.expected_generation as i64,
                ],
            )
            .map_err(sql)?;
        Ok(changed == 1)
    }

    /// Motif d'attente sans lancement : absent, DND, capacité, occupé, débit.
    pub(crate) fn thread_wake_defer(
        &self,
        thread_id: &str,
        agent_id: &str,
        reason: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE thread_wakes SET state = 'pending', reason = ?3, last_attempt_at = ?4
                 WHERE thread_id = ?1 AND agent_id = ?2 AND active_through_seq IS NULL",
                params![thread_id, agent_id, reason, now],
            )
            .map_err(sql)?;
        Ok(())
    }

    /// Issue d'une génération figée ; une génération étrangère est ignorée.
    pub(crate) fn thread_wake_settle(
        &self,
        thread_id: &str,
        agent_id: &str,
        generation: u64,
        outcome: WakeOutcome,
    ) -> Result<bool, StoreError> {
        let changed = match outcome {
            WakeOutcome::Dispatched => self.conn.execute(
                "UPDATE thread_wakes SET
                     dispatched_seq = MAX(dispatched_seq, COALESCE(active_through_seq, 0)),
                     state = 'dispatched', reason = NULL,
                     active_through_seq = NULL, delivery_key = NULL,
                     recipient_instance_id = NULL, delivery_generation = NULL,
                     issued_at = NULL, expires_at = NULL
                 WHERE thread_id = ?1 AND agent_id = ?2 AND generation = ?3
                   AND state IN ('in_flight', 'outcome_unknown')",
                params![thread_id, agent_id, generation as i64],
            ),
            WakeOutcome::Refused(reason) => self.conn.execute(
                "UPDATE thread_wakes SET
                     state = 'refused', reason = ?4,
                     active_through_seq = NULL, delivery_id = NULL, delivery_key = NULL,
                     recipient_instance_id = NULL, delivery_generation = NULL,
                     issued_at = NULL, expires_at = NULL
                 WHERE thread_id = ?1 AND agent_id = ?2 AND generation = ?3
                   AND state = 'in_flight'",
                params![thread_id, agent_id, generation as i64, reason],
            ),
            WakeOutcome::OutcomeUnknown => self.conn.execute(
                "UPDATE thread_wakes SET state = 'outcome_unknown'
                 WHERE thread_id = ?1 AND agent_id = ?2 AND generation = ?3
                   AND state = 'in_flight'",
                params![thread_id, agent_id, generation as i64],
            ),
        }
        .map_err(sql)?;
        Ok(changed == 1)
    }

    /// Une remise figée sans issue après son échéance d'injection devient une
    /// issue inconnue : elle n'est jamais rejouée sous la même borne.
    pub(crate) fn thread_wakes_expire(&self, now: i64) -> Result<usize, StoreError> {
        self.conn
            .execute(
                "UPDATE thread_wakes SET state = 'outcome_unknown'
                 WHERE state = 'in_flight' AND expires_at IS NOT NULL AND expires_at <= ?1",
                [now],
            )
            .map_err(sql)
    }

    /// Remises figées dont l'issue n'a pas été constatée (reprise au démarrage).
    pub(crate) fn thread_wakes_in_flight(&self) -> Result<Vec<WakeRow>, StoreError> {
        let mut statement = self
            .conn
            .prepare(&format!(
                "SELECT {WAKE_COLUMNS} FROM thread_wakes WHERE state = 'in_flight'"
            ))
            .map_err(sql)?;
        let rows = statement.query_map([], wake_from_row).map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
}

#[cfg(test)]
mod spec102_quota_tests {
    use super::*;
    use crate::store::Store;

    const A: &str = "10200000-0000-4000-8000-00000000000a";
    const B: &str = "10200000-0000-4000-8000-00000000000b";

    fn store() -> (Store, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("spec102-quota-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).unwrap();
        (Store::open(&root.join("db.sqlite")).unwrap(), root)
    }

    fn create(store: &Store, limits: &ThreadLimits, thread_id: &str) {
        let members = vec![A.to_string(), B.to_string()];
        let outcome = store
            .thread_create(CreateThread {
                actor: A,
                operation_id: &uuid::Uuid::new_v4().hyphenated().to_string(),
                canonical_hash: "h",
                thread_id,
                title: "Quota",
                members: &members,
                now: 1,
                limits,
            })
            .unwrap();
        assert!(matches!(outcome, ThreadTxOutcome::Done(_)), "{outcome:?}");
    }

    fn post(store: &Store, limits: &ThreadLimits, thread_id: &str, body: &str) -> ThreadTxOutcome {
        store
            .thread_post(PostEntry {
                actor: A,
                operation_id: &uuid::Uuid::new_v4().hyphenated().to_string(),
                canonical_hash: &format!("h-{}", uuid::Uuid::new_v4().simple()),
                thread_id,
                message_id: &uuid::Uuid::new_v4().hyphenated().to_string(),
                body,
                notify: &NotifySpec::None,
                notices: &[],
                reply_to_seq: None,
                ack_receipt: None,
                now: 2,
                limits,
            })
            .unwrap()
    }

    fn entries(store: &Store, thread_id: &str) -> u64 {
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM discussion_entries WHERE thread_id = ?1",
                [thread_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap() as u64
    }

    #[test]
    fn spec102_v31_plafonds_d_entrees_et_d_octets_n_puis_n_plus_1() {
        let (store, root) = store();
        let limits = ThreadLimits {
            max_threads: 256,
            max_open_per_creator: 32,
            max_entries_per_thread: 3,
            max_body_bytes_per_thread: 10,
            max_body_bytes_total: 15,
        };
        let t1 = "11111111-1111-4111-8111-111111111111";
        let t2 = "22222222-2222-4222-8222-222222222222";
        create(&store, &limits, t1);
        create(&store, &limits, t2);
        // Octets par fil : 4 + 4 acceptés (8), le troisième dépôt de 4 octets dépasse 10.
        assert!(matches!(
            post(&store, &limits, t1, "abcd"),
            ThreadTxOutcome::Done(_)
        ));
        assert!(matches!(
            post(&store, &limits, t1, "efgh"),
            ThreadTxOutcome::Done(_)
        ));
        assert_eq!(
            post(&store, &limits, t1, "ijkl"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("body_bytes_per_thread"))
        );
        assert!(
            matches!(post(&store, &limits, t1, "mn"), ThreadTxOutcome::Done(_)),
            "10 octets exactement acceptés"
        );
        assert_eq!(entries(&store, t1), 3);
        // Entrées par fil : la quatrième est refusée même pour un octet.
        assert_eq!(
            post(&store, &limits, t1, "o"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("entries_per_thread"))
        );
        assert_eq!(entries(&store, t1), 3, "aucune mutation au refus");
        // Total tous fils : 10 déjà consommés, 5 restent ; 6 octets refusés, 5 acceptés.
        assert_eq!(
            post(&store, &limits, t2, "pqrstu"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("body_bytes_total"))
        );
        assert!(matches!(
            post(&store, &limits, t2, "vwxyz"),
            ThreadTxOutcome::Done(_)
        ));
        assert_eq!(
            post(&store, &limits, t2, "!"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("body_bytes_total"))
        );
        let total: i64 = store
            .conn
            .query_row(
                "SELECT SUM(body_bytes) FROM discussion_threads",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total, 15, "compteurs cohérents avec les entrées");
        let _ = std::fs::remove_dir_all(root);
    }
}
