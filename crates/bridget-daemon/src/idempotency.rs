//! Socle transactionnel d'idempotence du daemon.
//!
//! Ce module ne connaît ni le routage ni la remise au wrapper. Il est le seul
//! propriétaire de la clé, des octets canoniques, de l'échéance et du résultat
//! public d'une opération idempotente.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
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
    pub fn as_str(self) -> &'static str {
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
    Rejected {
        category: String,
        reason: String,
        expires_at: i64,
    },
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

/// Identité durable de la remise aval, créée atomiquement au passage à
/// `Dispatching` afin d'interdire tout reroutage lors d'un rejeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendDelivery {
    pub delivery_id: String,
    pub recipient_instance_id: String,
    pub delivery_generation: u64,
    pub expires_at: i64,
    /// Enveloppe de remise exacte, possédée par la saga `send_deliveries`.
    /// Elle permet la reprise sans relire les structures éphémères du daemon
    /// ni résoudre à nouveau le nom du destinataire.
    pub message_bytes: Vec<u8>,
}

/// Demande suivie créée avec la remise d'un `reply=yes` dans l'unique
/// transaction SQLite : aucune remise idempotente ne peut survivre sans son
/// cycle de réponse durable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyTracking {
    pub request_id: String,
    pub sender: String,
    pub target: String,
    pub created_at: i64,
    pub deadline_at: i64,
}

#[derive(Debug)]
pub enum IdempotencyError {
    InvalidIssuerScope,
    InvalidIdempotencyKey,
    CanonicalTooLarge,
    InvalidIssuedAt,
    InvalidHorizon,
    InvalidTransition { from: RecordState, to: RecordState },
    InvalidDelivery,
    DispatchUnavailable,
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
            Self::InvalidDelivery => write!(formatter, "remise idempotente invalide"),
            Self::DispatchUnavailable => write!(formatter, "remise déjà traitée ou indisponible"),
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
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))?;
        Self::init_schema(&mut conn)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, IdempotencyError> {
        let mut conn = Connection::open_in_memory()?;
        Self::init_schema(&mut conn)?;
        Ok(Self { conn })
    }

    fn init_schema(conn: &mut Connection) -> Result<(), IdempotencyError> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
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
                ON idempotency_records(expires_at);
            CREATE TABLE IF NOT EXISTS send_deliveries (
                delivery_id TEXT PRIMARY KEY,
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL CHECK (operation_kind = 'send'),
                idempotency_key TEXT NOT NULL,
                recipient_instance_id TEXT NOT NULL,
                delivery_generation INTEGER NOT NULL CHECK (delivery_generation > 0),
                phase TEXT NOT NULL CHECK (phase IN ('dispatching', 'acked', 'indeterminate')),
                expires_at INTEGER NOT NULL,
                message_bytes BLOB,
                FOREIGN KEY (issuer_scope, operation_kind, idempotency_key)
                    REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                    ON DELETE CASCADE
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_send_deliveries_operation
                ON send_deliveries(issuer_scope, operation_kind, idempotency_key);
            CREATE TABLE IF NOT EXISTS idempotency_schema_migrations (
                version INTEGER PRIMARY KEY
            );
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
            );",
        )?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let migration_applied = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 2)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !migration_applied {
            let has_message_bytes = tx
                .prepare("PRAGMA table_info(send_deliveries)")?
                .query_map([], |row| row.get::<_, String>(1))?
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "message_bytes");
            if !has_message_bytes {
                tx.execute("ALTER TABLE send_deliveries ADD COLUMN message_bytes BLOB", [])?;
            }
            // Une remise v1 sans enveloppe est irréparable sans reroutage :
            // elle devient indéterminée dans la même migration avant v2.
            tx.execute(
                "UPDATE send_deliveries
                 SET phase = 'indeterminate'
                 WHERE phase = 'dispatching' AND message_bytes IS NULL",
                [],
            )?;
            tx.execute(
                "INSERT INTO idempotency_schema_migrations(version) VALUES (2)",
                [],
            )?;
        }
        tx.commit()?;
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
            #[cfg(feature = "test-support")]
            crate::test_sync::checkpoint("after_prepared");
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

    /// Signale qu'une réservation a survécu sans remise : le daemon peut
    /// reprendre ce seul état de récupération, jamais rerouter une remise
    /// déjà créée.
    pub fn prepared_expiry(
        &self,
        key: &IdempotencyKey,
        now: i64,
    ) -> Result<Option<i64>, IdempotencyError> {
        Ok(match self.load_record(key)? {
            Some(record) if record.state == RecordState::Prepared && record.expires_at > now => {
                Some(record.expires_at)
            }
            _ => None,
        })
    }

    pub fn transition(
        &self,
        key: &IdempotencyKey,
        from: RecordState,
        to: RecordState,
    ) -> Result<(), IdempotencyError> {
        if !matches!(
            (from, to),
            (RecordState::Prepared, RecordState::Dispatching)
        ) {
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

    /// Fige la remise d'un envoi. Le changement d'état du socle et l'entrée
    /// `send_deliveries` partagent une transaction SQLite et une même clé FK.
    pub fn begin_send_delivery(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_inner(key, delivery, None)
    }

    /// Prépare atomiquement la remise et le suivi d'une réponse attendue.
    pub fn begin_send_delivery_with_reply(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: &ReplyTracking,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_inner(key, delivery, Some(reply))
    }

    fn begin_send_delivery_inner(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: Option<&ReplyTracking>,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Send || delivery.delivery_id.is_empty() {
            return Err(IdempotencyError::InvalidDelivery);
        }
        if delivery.message_bytes.is_empty() || delivery.message_bytes.len() > MAX_CANONICAL_BYTES {
            return Err(IdempotencyError::InvalidDelivery);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let transitioned = tx.execute(
            "UPDATE idempotency_records SET state = 'dispatching'
             WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2
               AND state = 'prepared'",
            params![key.issuer_scope, key.idempotency_key],
        )?;
        if transitioned != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.execute(
            "INSERT INTO send_deliveries (
                delivery_id, issuer_scope, operation_kind, idempotency_key,
                recipient_instance_id, delivery_generation, phase, expires_at, message_bytes
             ) VALUES (?1, ?2, 'send', ?3, ?4, ?5, 'dispatching', ?6, ?7)",
            params![
                delivery.delivery_id,
                key.issuer_scope,
                key.idempotency_key,
                delivery.recipient_instance_id,
                delivery.delivery_generation,
                delivery.expires_at,
                delivery.message_bytes,
            ],
        )?;
        if let Some(reply) = reply {
            tx.execute(
                "INSERT INTO tracked_requests (
                    id, sender, target, state, created_at, deadline_at, escalation_level
                 ) VALUES (?1, ?2, ?3, 'open', ?4, ?5, 0)",
                params![
                    reply.request_id,
                    reply.sender,
                    reply.target,
                    reply.created_at,
                    reply.deadline_at,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Finalise un refus de clé neuve sans rendre l'état intermédiaire
    /// `Dispatching` observable à travers un crash ou une autre connexion.
    pub fn reject_prepared(
        &mut self,
        key: &IdempotencyKey,
        category: &str,
        reason: &str,
    ) -> Result<(), IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let dispatched = tx.execute(
            "UPDATE idempotency_records SET state = 'dispatching'
             WHERE issuer_scope = ?1 AND operation_kind = ?2 AND idempotency_key = ?3
               AND state = 'prepared'",
            params![
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
            ],
        )?;
        if dispatched != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let terminal = tx.execute(
            "UPDATE idempotency_records
             SET state = 'terminal', public_result_kind = 'rejected',
                 public_result_category = ?1, public_result_reason = ?2
             WHERE issuer_scope = ?3 AND operation_kind = ?4 AND idempotency_key = ?5
               AND state = 'dispatching'",
            params![
                category,
                reason,
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
            ],
        )?;
        if terminal != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    pub fn send_delivery(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at, message_bytes
                 FROM send_deliveries
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2",
                params![key.issuer_scope, key.idempotency_key],
                |row| {
                    Ok(SendDelivery {
                        delivery_id: row.get(0)?,
                        recipient_instance_id: row.get(1)?,
                        delivery_generation: row.get(2)?,
                        expires_at: row.get(3)?,
                        message_bytes: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Reprise bornée : seules les remises encore en cours pour l'instance
    /// exacte sont relivrées. Une remise Acked ou Indeterminate ne l'est pas.
    pub fn dispatching_deliveries_for_instance(
        &self,
        recipient_instance_id: &str,
        now: i64,
    ) -> Result<Vec<SendDelivery>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at, message_bytes
             FROM send_deliveries
             WHERE recipient_instance_id = ?1 AND phase = 'dispatching' AND expires_at > ?2
             ORDER BY delivery_id",
        )?;
        statement
            .query_map(params![recipient_instance_id, now], |row| {
                Ok(SendDelivery {
                    delivery_id: row.get(0)?,
                    recipient_instance_id: row.get(1)?,
                    delivery_generation: row.get(2)?,
                    expires_at: row.get(3)?,
                    message_bytes: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Une marque `Seen` sans observable d'injection reste indéterminée : le
    /// daemon conserve OutcomeUnknown jusqu'à l'expiration et ne réinjecte pas.
    pub fn mark_delivery_indeterminate(
        &mut self,
        delivery_id: &str,
        recipient_instance_id: &str,
        delivery_generation: u64,
    ) -> Result<(), IdempotencyError> {
        let updated = self.conn.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate'
             WHERE delivery_id = ?1 AND recipient_instance_id = ?2
               AND delivery_generation = ?3 AND phase = 'dispatching'",
            params![delivery_id, recipient_instance_id, delivery_generation],
        )?;
        if updated == 1 {
            Ok(())
        } else {
            Err(IdempotencyError::InvalidDelivery)
        }
    }

    /// Accusé aval : la remise et le résultat public deviennent terminaux dans
    /// une même transaction, après validation de l'instance et génération.
    pub fn acknowledge_send_delivery(
        &mut self,
        delivery_id: &str,
        recipient_instance_id: &str,
        delivery_generation: u64,
    ) -> Result<(), IdempotencyError> {
        let tx = self.conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let row = tx.query_row(
            "SELECT issuer_scope, idempotency_key, recipient_instance_id, delivery_generation, phase
             FROM send_deliveries WHERE delivery_id = ?1",
            params![delivery_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, u64>(3)?, row.get::<_, String>(4)?)),
        ).optional()?.ok_or(IdempotencyError::InvalidDelivery)?;
        if row.2 != recipient_instance_id || row.3 != delivery_generation { return Err(IdempotencyError::InvalidDelivery); }
        if row.4 == "acked" { tx.commit()?; return Ok(()); }
        if row.4 != "dispatching" { return Err(IdempotencyError::InvalidDelivery); }
        let delivery = tx.execute("UPDATE send_deliveries SET phase = 'acked' WHERE delivery_id = ?1 AND phase = 'dispatching'", params![delivery_id])?;
        let record = tx.execute(
            "UPDATE idempotency_records SET state = 'terminal', public_result_kind = 'accepted', public_result_category = NULL, public_result_reason = NULL
             WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2 AND state = 'dispatching'",
            params![row.0, row.1],
        )?;
        if delivery != 1 || record != 1 { return Err(IdempotencyError::DispatchUnavailable); }
        tx.commit()?;
        Ok(())
    }

    #[cfg(test)]
    pub fn record_count(&self) -> Result<usize, IdempotencyError> {
        self.conn
            .query_row("SELECT COUNT(*) FROM idempotency_records", [], |row| {
                row.get(0)
            })
            .map_err(Into::into)
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
            .execute(
                "DELETE FROM idempotency_records WHERE expires_at <= ?1",
                params![now],
            )
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
                        state: RecordState::from_str(&row.get::<_, String>(1)?)
                            .map_err(to_sql_error)?,
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
                expires_at: self.expires_at,
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
    rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error))
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
        store.reserve(&key(), bytes, NOW, HORIZON, NOW, 30).unwrap()
    }

    #[test]
    fn reserve_then_replay_uses_the_same_record() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Prepared {
                expires_at: NOW + HORIZON
            }
        );
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            })
        );
    }

    #[test]
    fn a_single_byte_difference_is_an_envelope_mismatch() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        assert_eq!(reserve(&store, b"canOn"), Reservation::EnvelopeMismatch);
    }

    #[test]
    fn terminal_result_is_replayed_without_mutation() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        store
            .transition(&key, RecordState::Prepared, RecordState::Dispatching)
            .unwrap();
        store
            .finalize(
                &key,
                PublicResult::Rejected {
            category: "dnd".to_string(),
            reason: "occupé".to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::Rejected {
            category: "dnd".to_string(),
            reason: "occupé".to_string(),
            expires_at: NOW + HORIZON,
            })
        );
    }

    #[test]
    fn first_send_outside_its_horizon_is_expired() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert_eq!(
            store
                .reserve(&key(), b"canon", NOW - HORIZON - 1, HORIZON, NOW, 30)
                .unwrap(),
            Reservation::IdempotencyExpired
        );
        assert_eq!(
            store.lookup(&key(), NOW).unwrap(),
            LookupResult::IdempotencyExpired
        );
    }

    #[test]
    fn transitions_are_monotone() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        assert!(matches!(
            store.transition(&key, RecordState::Prepared, RecordState::Terminal),
            Err(IdempotencyError::InvalidTransition { .. })
        ));
        store
            .transition(&key, RecordState::Prepared, RecordState::Dispatching)
            .unwrap();
        store
            .finalize(
                &key,
                PublicResult::Accepted {
                    expires_at: NOW + HORIZON,
                },
            )
            .unwrap();
        assert!(matches!(
            store.transition(&key, RecordState::Terminal, RecordState::Dispatching),
            Err(IdempotencyError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn purge_uses_each_record_expiry_not_a_new_configuration() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        assert_eq!(store.purge_expired(NOW + HORIZON - 1).unwrap(), 0);
        assert!(matches!(
            store.lookup(&key(), NOW + HORIZON - 1).unwrap(),
            LookupResult::OutcomeUnknown { .. }
        ));
        assert_eq!(store.purge_expired(NOW + HORIZON).unwrap(), 1);
        assert_eq!(
            store.lookup(&key(), NOW + HORIZON).unwrap(),
            LookupResult::IdempotencyExpired
        );
    }

    #[test]
    fn retry_keeps_the_original_horizon_after_a_configuration_drop() {
        let store = IdempotencyStore::open_in_memory().unwrap();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        assert_eq!(
            store
                .reserve(&key(), b"canon", NOW, 10, NOW + 11, 30)
                .unwrap(),
            Reservation::Replayed(LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            })
        );
    }

    #[test]
    fn dispatch_and_delivery_are_persisted_in_one_transaction() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-1".to_string(),
            recipient_instance_id: "instance-1".to_string(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: b"message-1".to_vec(),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        assert_eq!(store.send_delivery(&key).unwrap(), Some(delivery));
        assert_eq!(
            store.lookup(&key, NOW).unwrap(),
            LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            }
        );
    }

    #[test]
    fn retry_en_vol_rejoue_unknown_puis_accepted_apres_accuse() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        let delivery = SendDelivery {
            delivery_id: "delivery-retry".to_string(),
            recipient_instance_id: "instance-1".to_string(),
            delivery_generation: 9,
            expires_at: NOW + HORIZON,
            message_bytes: b"message-retry".to_vec(),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::OutcomeUnknown { expires_at: NOW + HORIZON })
        );
        store.acknowledge_send_delivery("delivery-retry", "instance-1", 9).unwrap();
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::Accepted { expires_at: NOW + HORIZON })
        );
    }

    #[test]
    fn purge_expired_removes_its_delivery_through_the_foreign_key() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        store
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-expired".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 2,
                    expires_at: NOW + HORIZON,
                    message_bytes: b"message-expired".to_vec(),
                },
            )
            .unwrap();
        assert_eq!(store.purge_expired(NOW + HORIZON).unwrap(), 1);
        assert_eq!(store.send_delivery(&key).unwrap(), None);
    }

    #[test]
    fn failed_delivery_insert_rolls_back_the_dispatch_transition() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let first = key();
        let second = IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message-2").unwrap();
        assert!(matches!(reserve(&store, b"first"), Reservation::Prepared { .. }));
        store
            .begin_send_delivery(
                &first,
                &SendDelivery {
                    delivery_id: "same-delivery".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 3,
                    expires_at: NOW + HORIZON,
                    message_bytes: b"message-first".to_vec(),
                },
            )
            .unwrap();
        assert!(matches!(
            store.reserve(&second, b"second", NOW, HORIZON, NOW, 30).unwrap(),
            Reservation::Prepared { .. }
        ));
        assert!(store
            .begin_send_delivery(
                &second,
                &SendDelivery {
                    delivery_id: "same-delivery".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 4,
                    expires_at: NOW + HORIZON,
                    message_bytes: b"message-second".to_vec(),
                },
            )
            .is_err());
        assert_eq!(
            store.lookup(&second, NOW).unwrap(),
            LookupResult::OutcomeUnknown { expires_at: NOW + HORIZON }
        );
        assert_eq!(store.send_delivery(&second).unwrap(), None);
    }

    #[test]
    fn failed_reply_tracking_rolls_back_then_a_retry_after_restart_prepares_both() {
        let path = std::env::temp_dir().join(format!(
            "bridget-idempotency-reply-fault-{}-{}.db",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let key = key();
        let delivery = SendDelivery {
            delivery_id: "delivery-reply".to_string(),
            recipient_instance_id: "instance-1".to_string(),
            delivery_generation: 5,
            expires_at: NOW + HORIZON,
            message_bytes: b"message-reply".to_vec(),
        };
        let reply = ReplyTracking {
            request_id: "request-reply".to_string(),
            sender: "maicie".to_string(),
            target: "agent-2".to_string(),
            created_at: NOW,
            deadline_at: NOW + 60,
        };
        {
            let mut store = IdempotencyStore::open(&path).unwrap();
            assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
            // Injection de faute : la clé primaire déjà présente force l'INSERT
            // de suivi à échouer après l'INSERT de remise, dans la transaction.
            store
                .conn
                .execute(
                    "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at, escalation_level)
                     VALUES ('request-reply', 'old', 'target', 'open', 1, 2, 0)",
                    [],
                )
                .unwrap();
            assert!(store
                .begin_send_delivery_with_reply(&key, &delivery, &reply)
                .is_err());
            assert_eq!(store.send_delivery(&key).unwrap(), None);
            assert_eq!(
                store.lookup(&key, NOW).unwrap(),
                LookupResult::OutcomeUnknown {
                    expires_at: NOW + HORIZON
                }
            );
        }
        let mut reopened = IdempotencyStore::open(&path).unwrap();
        assert!(matches!(
            reserve(&reopened, b"canon"),
            Reservation::Replayed(LookupResult::OutcomeUnknown { .. })
        ));
        assert_eq!(reopened.prepared_expiry(&key, NOW).unwrap(), Some(NOW + HORIZON));
        reopened
            .conn
            .execute("DELETE FROM tracked_requests WHERE id = 'request-reply'", [])
            .unwrap();
        reopened
            .begin_send_delivery_with_reply(&key, &delivery, &reply)
            .unwrap();
        assert_eq!(reopened.send_delivery(&key).unwrap(), Some(delivery));
        assert_eq!(
            reopened
                .conn
                .query_row(
                    "SELECT sender || ':' || target FROM tracked_requests WHERE id = 'request-reply'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "maicie:agent-2"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn reject_prepared_publishes_only_the_terminal_refusal() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(reserve(&store, b"canon"), Reservation::Prepared { .. }));
        store.reject_prepared(&key, "routing", "cible absente").unwrap();
        assert_eq!(
            store.lookup(&key, NOW).unwrap(),
            LookupResult::Rejected {
                category: "routing".to_string(),
                reason: "cible absente".to_string(),
                expires_at: NOW + HORIZON,
            }
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
    fn migration_v1_classe_les_remises_sans_payload_sans_bloquer_les_valides() {
        let path = std::env::temp_dir().join(format!(
            "bridget-idempotency-v1-{}-{}.db",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE idempotency_records (
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                canonical_bytes BLOB NOT NULL,
                state TEXT NOT NULL,
                public_result_kind TEXT,
                public_result_category TEXT,
                public_result_reason TEXT,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
            );
            CREATE TABLE send_deliveries (
                delivery_id TEXT PRIMARY KEY,
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                recipient_instance_id TEXT NOT NULL,
                delivery_generation INTEGER NOT NULL,
                phase TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                message_bytes BLOB
            );",
        )
        .unwrap();
        for (key, delivery) in [("legacy", "delivery-legacy"), ("valid", "delivery-valid")] {
            conn.execute(
                "INSERT INTO idempotency_records VALUES (?1, 'send', ?2, X'00', 'dispatching', NULL, NULL, NULL, ?3, ?4)",
                params!["012_scope_aaaaaaaaaaaa", key, NOW, NOW + HORIZON],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO send_deliveries VALUES (?1, ?2, 'send', ?3, 'instance-1', 1, 'dispatching', ?4, ?5)",
                params![
                    delivery,
                    "012_scope_aaaaaaaaaaaa",
                    key,
                    NOW + HORIZON,
                    (key == "valid").then(|| b"payload".to_vec()),
                ],
            )
            .unwrap();
        }
        drop(conn);

        let store = IdempotencyStore::open(&path).unwrap();
        let deliveries = store
            .dispatching_deliveries_for_instance("instance-1", NOW)
            .unwrap();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].delivery_id, "delivery-valid");
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-legacy'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "indeterminate"
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM idempotency_schema_migrations WHERE version = 2",
                    [],
                    |row| row.get::<_, u32>(0),
                )
                .unwrap(),
            1
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn issuer_scope_requires_a_base64url_sized_opaque_value() {
        assert!(IdempotencyKey::new("scope-too-short", OperationKind::Send, "message").is_err());
        assert!(
            IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message").is_ok()
        );
        assert!(
            IdempotencyKey::new("012_scope_aaaaaaaaaaaa!", OperationKind::Send, "message").is_err()
        );
    }

    #[test]
    fn two_concurrent_reservations_have_one_winner() {
        let db_path =
            std::env::temp_dir().join(format!("bridget-idempotency-{}.db", std::process::id()));
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
        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Reservation::Prepared { .. }))
                .count(),
            1
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Reservation::Replayed(_)))
                .count(),
            1
        );
        let _ = std::fs::remove_file(db_path);
    }
}
