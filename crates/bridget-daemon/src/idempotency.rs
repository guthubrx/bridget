//! Socle transactionnel d'idempotence du daemon.
//!
//! Ce module ne connaît ni le routage ni la remise au wrapper. Il est le seul
//! propriétaire de la clé, des octets canoniques, de l'échéance et du résultat
//! public d'une opération idempotente.

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const MIN_ISSUER_SCOPE_LEN: usize = 22;
const MAX_ISSUER_SCOPE_LEN: usize = 128;
const MAX_IDEMPOTENCY_KEY_LEN: usize = 256;
const MAX_CANONICAL_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    Send,
    Spawn,
}

impl OperationKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Send => "send",
            Self::Spawn => "spawn",
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyKey {
    pub issuer_scope: String,
    pub operation_kind: OperationKind,
    pub idempotency_key: String,
}

impl IdempotencyKey {
    pub fn new(
        issuer_scope: impl Into<String>,
        operation_kind: OperationKind,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, IdempotencyError> {
        let issuer_scope = issuer_scope.into();
        validate_issuer_scope(&issuer_scope)?;
        let idempotency_key = idempotency_key.into();
        validate_idempotency_key(&idempotency_key)?;
        Ok(Self {
            issuer_scope,
            operation_kind,
            idempotency_key,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordState {
    Prepared,
    Dispatching,
    Terminal,
}

impl RecordState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Dispatching => "dispatching",
            Self::Terminal => "terminal",
        }
    }

    fn from_str(value: &str) -> Result<Self, IdempotencyError> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "dispatching" => Ok(Self::Dispatching),
            "terminal" => Ok(Self::Terminal),
            _ => Err(IdempotencyError::CorruptRecord("état inconnu")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicResult {
    Accepted { expires_at: i64 },
    Rejected { category: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupResult {
    Accepted { expires_at: i64 },
    Rejected { category: String, reason: String },
    OutcomeUnknown { expires_at: i64 },
    IdempotencyExpired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reservation {
    Prepared { expires_at: i64 },
    Replayed(LookupResult),
    EnvelopeMismatch,
    IdempotencyExpired,
}

#[derive(Debug)]
pub enum IdempotencyError {
    InvalidIssuerScope,
    InvalidIdempotencyKey,
    CanonicalTooLarge,
    InvalidIssuedAt,
    InvalidHorizon,
    InvalidTransition { from: RecordState, to: RecordState },
    MissingRecord,
    CorruptRecord(&'static str),
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for IdempotencyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIssuerScope => write!(formatter, "issuer_scope invalide"),
            Self::InvalidIdempotencyKey => write!(formatter, "clé d'idempotence invalide"),
            Self::CanonicalTooLarge => write!(formatter, "enveloppe canonique trop grande"),
            Self::InvalidIssuedAt => write!(formatter, "issued_at invalide"),
            Self::InvalidHorizon => write!(formatter, "horizon d'idempotence invalide"),
            Self::InvalidTransition { from, to } => {
                write!(formatter, "transition interdite: {from:?} vers {to:?}")
            }
            Self::MissingRecord => write!(formatter, "enregistrement d'idempotence absent"),
            Self::CorruptRecord(detail) => write!(formatter, "enregistrement corrompu: {detail}"),
            Self::Sqlite(error) => write!(formatter, "SQLite: {error}"),
        }
    }
}

impl std::error::Error for IdempotencyError {}

impl From<rusqlite::Error> for IdempotencyError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

pub struct IdempotencyStore {
    conn: Connection,
}

impl IdempotencyStore {
    pub fn open(path: &Path) -> Result<Self, IdempotencyError> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, IdempotencyError> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    fn init_schema(conn: &Connection) -> Result<(), IdempotencyError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS idempotency_records (
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL CHECK (operation_kind IN ('send', 'spawn')),
                idempotency_key TEXT NOT NULL,
                canonical_bytes BLOB NOT NULL,
                state TEXT NOT NULL CHECK (state IN ('prepared', 'dispatching', 'terminal')),
                public_result_kind TEXT,
                public_result_category TEXT,
                public_result_reason TEXT,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
            );
            CREATE INDEX IF NOT EXISTS idx_idempotency_records_expires_at
                ON idempotency_records(expires_at);",
        )?;
        Ok(())
    }

    pub fn reserve(
        &self,
        key: &IdempotencyKey,
        canonical_bytes: &[u8],
        issued_at: i64,
        horizon_secs: i64,
        now: i64,
        issued_at_tolerance_secs: i64,
    ) -> Result<Reservation, IdempotencyError> {
        validate_canonical_bytes(canonical_bytes)?;
        if issued_at > now.saturating_add(issued_at_tolerance_secs.max(0)) {
            return Err(IdempotencyError::InvalidIssuedAt);
        }
        if let Some(replay) = self.replay_existing(key, canonical_bytes, now)? {
            return Ok(replay);
        }
        if horizon_secs <= 0 {
            return Err(IdempotencyError::InvalidHorizon);
        }
        let expires_at = issued_at
            .checked_add(horizon_secs)
            .ok_or(IdempotencyError::InvalidHorizon)?;
        if expires_at <= now {
            return Ok(Reservation::IdempotencyExpired);
        }

        let inserted = self.conn.execute(
            "INSERT INTO idempotency_records (
                issuer_scope, operation_kind, idempotency_key, canonical_bytes,
                state, issued_at, expires_at
            ) VALUES (?1, ?2, ?3, ?4, 'prepared', ?5, ?6)
            ON CONFLICT(issuer_scope, operation_kind, idempotency_key) DO NOTHING",
            params![
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
                canonical_bytes,
                issued_at,
                expires_at,
            ],
        )?;
        if inserted == 1 {
            return Ok(Reservation::Prepared { expires_at });
        }

        self.replay_existing(key, canonical_bytes, now)?
            .ok_or(IdempotencyError::MissingRecord)
    }

    pub fn lookup(&self, key: &IdempotencyKey, now: i64) -> Result<LookupResult, IdempotencyError> {
        match self.load_record(key)? {
            Some(record) if record.expires_at <= now => Ok(LookupResult::IdempotencyExpired),
            Some(record) => record.lookup_result(),
            None => Ok(LookupResult::IdempotencyExpired),
        }
    }

    pub fn transition(
        &self,
        key: &IdempotencyKey,
        from: RecordState,
        to: RecordState,
    ) -> Result<(), IdempotencyError> {
        if !matches!((from, to), (RecordState::Prepared, RecordState::Dispatching)) {
            return Err(IdempotencyError::InvalidTransition { from, to });
        }
        let updated = self.conn.execute(
            "UPDATE idempotency_records SET state = ?1
             WHERE issuer_scope = ?2 AND operation_kind = ?3 AND idempotency_key = ?4
               AND state = ?5",
            params![
                to.as_str(),
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
                from.as_str(),
            ],
        )?;
        if updated == 1 {
            Ok(())
        } else if self.load_record(key)?.is_some() {
            Err(IdempotencyError::InvalidTransition { from, to })
        } else {
            Err(IdempotencyError::MissingRecord)
        }
    }

    pub fn finalize(
        &self,
        key: &IdempotencyKey,
        result: PublicResult,
    ) -> Result<(), IdempotencyError> {
        let (kind, category, reason) = match &result {
            PublicResult::Accepted { .. } => ("accepted", None, None),
            PublicResult::Rejected { category, reason } => {
                ("rejected", Some(category.as_str()), Some(reason.as_str()))
            }
        };
        let updated = self.conn.execute(
            "UPDATE idempotency_records
             SET state = 'terminal', public_result_kind = ?1,
                 public_result_category = ?2, public_result_reason = ?3
             WHERE issuer_scope = ?4 AND operation_kind = ?5 AND idempotency_key = ?6
               AND state = 'dispatching'",
            params![
                kind,
                category,
                reason,
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
            ],
        )?;
        if updated == 1 {
            Ok(())
        } else if let Some(record) = self.load_record(key)? {
            Err(IdempotencyError::InvalidTransition {
                from: record.state,
                to: RecordState::Terminal,
            })
        } else {
            Err(IdempotencyError::MissingRecord)
        }
    }

    pub fn purge_expired(&self, now: i64) -> Result<usize, IdempotencyError> {
        self.conn
            .execute("DELETE FROM idempotency_records WHERE expires_at <= ?1", params![now])
            .map_err(Into::into)
    }

    fn load_record(&self, key: &IdempotencyKey) -> Result<Option<Record>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT canonical_bytes, state, public_result_kind,
                        public_result_category, public_result_reason, expires_at
                 FROM idempotency_records
                 WHERE issuer_scope = ?1 AND operation_kind = ?2 AND idempotency_key = ?3",
                params![
                    key.issuer_scope,
                    key.operation_kind.as_str(),
                    key.idempotency_key,
                ],
                |row| {
                    Ok(Record {
                        canonical_bytes: row.get(0)?,
                        state: RecordState::from_str(&row.get::<_, String>(1)?).map_err(to_sql_error)?,
                        public_result_kind: row.get(2)?,
                        public_result_category: row.get(3)?,
                        public_result_reason: row.get(4)?,
                        expires_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn replay_existing(
        &self,
        key: &IdempotencyKey,
        canonical_bytes: &[u8],
        now: i64,
    ) -> Result<Option<Reservation>, IdempotencyError> {
        let Some(record) = self.load_record(key)? else {
            return Ok(None);
        };
        if record.expires_at <= now {
            return Ok(Some(Reservation::IdempotencyExpired));
        }
        if record.canonical_bytes != canonical_bytes {
            return Ok(Some(Reservation::EnvelopeMismatch));
        }
        Ok(Some(Reservation::Replayed(record.lookup_result()?)))
    }
}

struct Record {
    canonical_bytes: Vec<u8>,
    state: RecordState,
    public_result_kind: Option<String>,
    public_result_category: Option<String>,
    public_result_reason: Option<String>,
    expires_at: i64,
}

impl Record {
    fn lookup_result(&self) -> Result<LookupResult, IdempotencyError> {
        if self.state != RecordState::Terminal {
            return Ok(LookupResult::OutcomeUnknown {
                expires_at: self.expires_at,
            });
        }
        match self.public_result_kind.as_deref() {
            Some("accepted") => Ok(LookupResult::Accepted {
                expires_at: self.expires_at,
            }),
            Some("rejected") => Ok(LookupResult::Rejected {
                category: self
                    .public_result_category
                    .clone()
                    .ok_or(IdempotencyError::CorruptRecord("catégorie absente"))?,
                reason: self
                    .public_result_reason
                    .clone()
                    .ok_or(IdempotencyError::CorruptRecord("motif absent"))?,
            }),
            _ => Err(IdempotencyError::CorruptRecord("issue terminale absente")),
        }
    }
}

pub(crate) fn validate_issuer_scope(value: &str) -> Result<(), IdempotencyError> {
    if !(MIN_ISSUER_SCOPE_LEN..=MAX_ISSUER_SCOPE_LEN).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(IdempotencyError::InvalidIssuerScope);
    }
    Ok(())
}

fn validate_idempotency_key(value: &str) -> Result<(), IdempotencyError> {
    if value.is_empty()
        || value.len() > MAX_IDEMPOTENCY_KEY_LEN
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(IdempotencyError::InvalidIdempotencyKey);
    }
    Ok(())
}

fn validate_canonical_bytes(value: &[u8]) -> Result<(), IdempotencyError> {
    if value.len() > MAX_CANONICAL_BYTES {
        return Err(IdempotencyError::CanonicalTooLarge);
    }
    Ok(())
}

fn to_sql_error(error: IdempotencyError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        1,
        rusqlite::types::Type::Text,
        Box::new(error),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    const NOW: i64 = 1_000_000;
    const HORIZON: i64 = 3600;

    fn key() -> IdempotencyKey {
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message-1").unwrap()
    }

    fn reserve(store: &IdempotencyStore, bytes: &[u8]) -> Reservation {
        store
            .reserve(&key(), bytes, NOW, HORIZON, NOW, 30)
            .unwrap()
    }

    #[test]
    fn reserve_then_replay_uses_the_same_record() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert_eq!(reserve(&store, b"canon"), Reservation::Prepared { expires_at: NOW + HORIZON });
        assert_eq!(reserve(&store, b"canon"), Reservation::Replayed(LookupResult::OutcomeUnknown { expires_at: NOW + HORIZON }));
    }

    #[test]
    fn a_single_byte_difference_is_an_envelope_mismatch() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        assert_eq!(reserve(&store, b"canOn"), Reservation::EnvelopeMismatch);
    }

    #[test]
    fn terminal_result_is_replayed_without_mutation() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        store.transition(&key, RecordState::Prepared, RecordState::Dispatching).unwrap();
        store.finalize(&key, PublicResult::Rejected {
            category: "dnd".to_string(),
            reason: "occupé".to_string(),
        }).unwrap();
        assert_eq!(reserve(&store, b"canon"), Reservation::Replayed(LookupResult::Rejected {
            category: "dnd".to_string(),
            reason: "occupé".to_string(),
        }));
    }

    #[test]
    fn first_send_outside_its_horizon_is_expired() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert_eq!(
            store.reserve(&key(), b"canon", NOW - HORIZON - 1, HORIZON, NOW, 30).unwrap(),
            Reservation::IdempotencyExpired
        );
        assert_eq!(store.lookup(&key(), NOW).unwrap(), LookupResult::IdempotencyExpired);
    }

    #[test]
    fn transitions_are_monotone() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        assert!(matches!(
            store.transition(&key, RecordState::Prepared, RecordState::Terminal),
            Err(IdempotencyError::InvalidTransition { .. })
        ));
        store.transition(&key, RecordState::Prepared, RecordState::Dispatching).unwrap();
        store.finalize(&key, PublicResult::Accepted { expires_at: NOW + HORIZON }).unwrap();
        assert!(matches!(
            store.transition(&key, RecordState::Terminal, RecordState::Dispatching),
            Err(IdempotencyError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn purge_uses_each_record_expiry_not_a_new_configuration() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        assert_eq!(store.purge_expired(NOW + HORIZON - 1).unwrap(), 0);
        assert!(matches!(store.lookup(&key(), NOW + HORIZON - 1).unwrap(), LookupResult::OutcomeUnknown { .. }));
        assert_eq!(store.purge_expired(NOW + HORIZON).unwrap(), 1);
        assert_eq!(store.lookup(&key(), NOW + HORIZON).unwrap(), LookupResult::IdempotencyExpired);
    }

    #[test]
    fn retry_keeps_the_original_horizon_after_a_configuration_drop() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        assert_eq!(
            store.reserve(&key(), b"canon", NOW, 10, NOW + 11, 30).unwrap(),
            Reservation::Replayed(LookupResult::OutcomeUnknown { expires_at: NOW + HORIZON })
        );
    }

    #[test]
    fn invalid_horizons_are_refused_for_a_first_reservation() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(
            store.reserve(&key(), b"zero", NOW, 0, NOW, 30),
            Err(IdempotencyError::InvalidHorizon)
        ));
        assert!(matches!(
            store.reserve(&key(), b"negative", NOW, -1, NOW, 30),
            Err(IdempotencyError::InvalidHorizon)
        ));
    }

    #[test]
    fn issuer_scope_requires_a_base64url_sized_opaque_value() {
        assert!(IdempotencyKey::new("scope-too-short", OperationKind::Send, "message").is_err());
        assert!(IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message").is_ok());
        assert!(IdempotencyKey::new("012_scope_aaaaaaaaaaaa!", OperationKind::Send, "message").is_err());
    }

    #[test]
    fn two_concurrent_reservations_have_one_winner() {
        let db_path = std::env::temp_dir().join(format!("bridget-idempotency-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&db_path);
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let path = db_path.clone();
            let barrier = barrier.clone();
            handles.push(thread::spawn(move || {
                let store = IdempotencyStore::open(&path).unwrap();
                barrier.wait();
                reserve(&store, b"canon")
            }));
        }
        let results: Vec<_> = handles.into_iter().map(|handle| handle.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|result| matches!(result, Reservation::Prepared { .. })).count(), 1);
        assert_eq!(results.iter().filter(|result| matches!(result, Reservation::Replayed(_))).count(), 1);
        let _ = std::fs::remove_file(db_path);
    }
}
