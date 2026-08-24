//! Persistance SQLite — ledger, compteurs disjoncteur, historique.

use bridget_transport::protocol::{
    CoordinationEventKind, GuichetLifecycleState, GuichetOutcome, ServiceRequestOperation,
    ServiceRequestPayload,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use std::path::Path;
use uuid::Uuid;

const GUICHET_LEASE_SECS: i64 = 60;
pub(crate) const MAX_GUICHET_FRAME_BYTES: usize = 64 * 1024;

/// Requête de guichet validée par le daemon avant toute persistance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetDeposit {
    pub issuer_scope: String,
    pub request_id: String,
    pub issued_at: i64,
    pub from: String,
    pub operation: ServiceRequestOperation,
    pub payload: ServiceRequestPayload,
    pub canonical_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetClaim {
    pub deposited_sequence: i64,
    pub issuer_scope: String,
    pub request_id: String,
    pub canonical_request: Vec<u8>,
    pub claimed_at: i64,
    pub claim_generation: u64,
    pub claim_token: String,
    pub claim_lease_expires_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuichetResult {
    Queued {
        expires_at: i64,
    },
    OutcomeUnknown {
        expires_at: i64,
    },
    Terminal {
        issue: String,
        expires_at: i64,
        newly_finalized: bool,
    },
    CanonicalBytesMismatch,
    IdempotencyExpired,
    InvalidIssuedAt,
    ClaimStale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuichetNext {
    Claimed(GuichetClaim),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetReplyInput<'a> {
    pub issuer_scope: &'a str,
    pub request_id: &'a str,
    pub generation: u64,
    pub token: &'a str,
    pub response_message_id: &'a str,
    pub reply_bytes: &'a [u8],
    pub in_reply_to: &'a str,
    pub outcome: GuichetOutcome,
}

/// Événement de cycle durable, relivable par une connexion de service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetLifecycleEvent {
    pub issuer_scope: String,
    pub event_id: String,
    pub request_id: String,
    pub state: GuichetLifecycleState,
    pub observed_at: i64,
    pub in_reply_to: Option<String>,
    pub response_message_id: Option<String>,
}

/// Fait de coordination v1 durable, produit par le transport après une
/// relance réellement écrite. Il reste séparé des terminaux 015 : plusieurs
/// relances peuvent appartenir à une même demande suivie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetCoordinationEvent {
    pub event_id: String,
    pub request_id: String,
    pub kind: CoordinationEventKind,
    pub reminder_message_id: String,
    pub recipient: String,
    pub generation: u64,
    pub observed_at: i64,
}

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
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(StoreError::Sqlite)?;
        Self::init_schema(&conn)?;
        Ok(Store { conn })
    }

    fn init_schema(conn: &Connection) -> Result<(), StoreError> {
        // `result_bytes` a été retirée du schéma canonique : `reply_bytes`
        // porte déjà les octets terminaux rejouables. Les bases antérieures
        // peuvent conserver cette colonne nullable ignorée ; reconstruire la
        // table pour la supprimer n'apporterait aucun invariant supplémentaire.
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
            CREATE TABLE IF NOT EXISTS request_events (
                request_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                level INTEGER NOT NULL,
                ts INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_request_events_request ON request_events(request_id, ts);
            CREATE TABLE IF NOT EXISTS guichet_requests (
                deposited_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                request_id TEXT NOT NULL,
                canonical_request BLOB NOT NULL,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                sender TEXT NOT NULL,
                linked_request_id TEXT,
                state TEXT NOT NULL CHECK (state IN ('queued', 'claimed', 'replied', 'rejected')),
                claim_owner TEXT,
                claim_generation INTEGER NOT NULL DEFAULT 0,
                claim_token TEXT,
                claim_lease_expires_at INTEGER,
                result_issue TEXT,
                reply_bytes BLOB,
                UNIQUE (issuer_scope, operation_kind, request_id)
            );
            CREATE INDEX IF NOT EXISTS idx_guichet_fifo
                ON guichet_requests(state, deposited_sequence);
            CREATE TABLE IF NOT EXISTS guichet_lifecycle_events (
                issuer_scope TEXT NOT NULL,
                request_id TEXT NOT NULL,
                event_id TEXT NOT NULL UNIQUE,
                state TEXT NOT NULL CHECK (state IN ('answered', 'cancelled', 'timed_out')),
                observed_at INTEGER NOT NULL,
                in_reply_to TEXT,
                response_message_id TEXT,
                PRIMARY KEY (issuer_scope, request_id)
            );
            CREATE TABLE IF NOT EXISTS guichet_coordination_events (
                event_id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('reminder_sent')),
                reminder_message_id TEXT NOT NULL,
                recipient TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation > 0),
                observed_at INTEGER NOT NULL,
                UNIQUE (request_id, generation)
            );
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

    pub fn requests_for_participant(
        &self,
        participant: &str,
        limit: usize,
    ) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests(
            "SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE sender = ?1 OR target = ?1 ORDER BY created_at DESC LIMIT ?2",
            rusqlite::params![participant, limit.clamp(1, 200)],
        )
    }

    /// Projection bornée des demandes, destinée aux lecteurs neutres du
    /// daemon (CLI fédérée aujourd'hui, vue ledger demain).
    pub fn recent_requests(&self, limit: usize) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests(
            "SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests ORDER BY created_at DESC LIMIT ?1",
            rusqlite::params![limit as i64],
        )
    }

    pub fn open_requests(&self) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE state = 'open' ORDER BY deadline_at", [])
    }

    pub fn cancel_request(
        &mut self,
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
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(StoreError::Sqlite)?;
            let changed = tx
                .execute(
                    "UPDATE tracked_requests
                     SET state = 'cancelled', cancel_reason = ?1, completed_at = ?2
                     WHERE id = ?3 AND state = 'open'",
                    rusqlite::params![reason, completed_at, id],
                )
                .map_err(StoreError::Sqlite)?;
            if changed == 1 {
                record_lifecycle_for_linked_request_in_transaction(
                    &tx,
                    id,
                    GuichetLifecycleState::Cancelled,
                    completed_at,
                )?;
            }
            tx.commit().map_err(StoreError::Sqlite)?;
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
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::Sqlite)?;
        let answered = mark_answered_in_transaction(&transaction, id, responder, recipient)
            .map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)?;
        Ok(answered)
    }

    pub fn mark_timed_out(&mut self, id: &str) -> Result<bool, StoreError> {
        let completed_at = now_secs();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let changed = tx
            .execute(
                "UPDATE tracked_requests SET state = 'timed_out', completed_at = ?1
                 WHERE id = ?2 AND state = 'open'",
                rusqlite::params![completed_at, id],
            )
            .map_err(StoreError::Sqlite)?;
        if changed == 1 {
            record_lifecycle_for_linked_request_in_transaction(
                &tx,
                id,
                GuichetLifecycleState::TimedOut,
                completed_at,
            )?;
        }
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }

    pub fn set_escalation_level(&self, id: &str, level: u8) -> Result<(), StoreError> {
        self.conn.execute("UPDATE tracked_requests SET escalation_level = ?1 WHERE id = ?2 AND state = 'open'", rusqlite::params![level, id]).map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Événement de cycle de vie consultable par la vue ledger : une relance a
    /// été retenue parce que l'équipier avait un tour ACP en cours.
    pub fn record_deferred_reminder(&self, id: &str, level: u8) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO request_events (request_id, event_type, level, ts) VALUES (?1, 'reminder_deferred', ?2, ?3)",
            rusqlite::params![id, level, now_secs()],
        ).map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Dernier report différé, exposé par la vue publique des demandes.
    pub fn latest_deferred_reminder(&self, id: &str) -> Result<Option<(u8, i64)>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT level, ts FROM request_events WHERE request_id = ?1 AND event_type = 'reminder_deferred' ORDER BY ts DESC, rowid DESC LIMIT 1",
        ).map_err(StoreError::Sqlite)?;
        let mut rows = statement
            .query(rusqlite::params![id])
            .map_err(StoreError::Sqlite)?;
        rows.next()
            .map_err(StoreError::Sqlite)?
            .map(|row| Ok((row.get(0)?, row.get(1)?)))
            .transpose()
            .map_err(StoreError::Sqlite)
    }

    /// Dépose ou rejoue une demande de guichet. La clé et les octets restent
    /// dans l'unique transaction IMMEDIATE : deux producteurs concurrents ne
    /// peuvent ni créer deux lignes, ni remplacer le canon du premier.
    pub fn deposit_guichet(
        &mut self,
        deposit: &GuichetDeposit,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        now: i64,
    ) -> Result<GuichetResult, StoreError> {
        if deposit.canonical_bytes.len().saturating_add(1) > MAX_GUICHET_FRAME_BYTES {
            return Err(StoreError::FrameTooLarge {
                max_frame_bytes: MAX_GUICHET_FRAME_BYTES,
            });
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let existing = guichet_row_for_key(&tx, &deposit.issuer_scope, &deposit.request_id)?;
        if let Some(row) = existing {
            let result = if row.canonical_request != deposit.canonical_bytes {
                GuichetResult::CanonicalBytesMismatch
            } else {
                guichet_existing_result(&row, now)
            };
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(result);
        }
        if deposit.issued_at > now.saturating_add(issued_at_tolerance_secs) {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::InvalidIssuedAt);
        }
        let expires_at = deposit.issued_at.saturating_add(horizon_secs);
        if expires_at < now {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::IdempotencyExpired);
        }
        tx.execute(
            "INSERT INTO guichet_requests
                (issuer_scope, operation_kind, request_id, canonical_request,
                 issued_at, expires_at, sender, linked_request_id, state)
             VALUES (?1, 'service_request', ?2, ?3, ?4, ?5, ?6, ?7, 'queued')",
            params![
                deposit.issuer_scope,
                deposit.request_id,
                deposit.canonical_bytes,
                deposit.issued_at,
                expires_at,
                deposit.from,
                linked_request_id(&deposit.payload),
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetResult::Queued { expires_at })
    }

    /// Relève atomiquement le prochain dépôt FIFO. Une tête abandonnée est
    /// remise en file avant la sélection ; en v1, des crashes répétés sur la
    /// même tête peuvent donc la faire réapparaître jusqu'à son refus métier.
    pub fn claim_next_guichet(&mut self, owner: &str, now: i64) -> Result<GuichetNext, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        tx.execute(
            "UPDATE guichet_requests
             SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                 claim_lease_expires_at = NULL
             WHERE state = 'claimed' AND claim_lease_expires_at < ?1",
            params![now],
        )
        .map_err(StoreError::Sqlite)?;
        let Some(row) = guichet_next_queued(&tx, now)? else {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetNext::Empty);
        };
        let generation = row.claim_generation.saturating_add(1);
        let token = claim_token();
        let lease_expires_at = now.saturating_add(GUICHET_LEASE_SECS);
        let changed = tx
            .execute(
                "UPDATE guichet_requests
                 SET state = 'claimed', claim_owner = ?1, claim_generation = ?2,
                     claim_token = ?3, claim_lease_expires_at = ?4
                 WHERE deposited_sequence = ?5 AND state = 'queued'",
                params![
                    owner,
                    generation as i64,
                    token,
                    lease_expires_at,
                    row.deposited_sequence
                ],
            )
            .map_err(StoreError::Sqlite)?;
        if changed != 1 {
            return Err(StoreError::Invariant("claim FIFO concurrent perdu"));
        }
        let claim = GuichetClaim {
            deposited_sequence: row.deposited_sequence,
            issuer_scope: row.issuer_scope,
            request_id: row.request_id,
            canonical_request: row.canonical_request,
            claimed_at: now,
            claim_generation: generation,
            claim_token: token,
            claim_lease_expires_at: lease_expires_at,
            expires_at: row.expires_at,
        };
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetNext::Claimed(claim))
    }

    /// Rejoue strictement le claim courant sans prolonger sa lease.
    pub fn claim_guichet(
        &self,
        owner: &str,
        issuer_scope: &str,
        request_id: &str,
        token: &str,
        now: i64,
    ) -> Result<Result<GuichetClaim, GuichetResult>, StoreError> {
        let Some(row) = guichet_row_for_key(&self.conn, issuer_scope, request_id)? else {
            return Ok(Err(GuichetResult::IdempotencyExpired));
        };
        if row.state != "claimed"
            || row.claim_owner.as_deref() != Some(owner)
            || row.claim_token.as_deref() != Some(token)
            || row
                .claim_lease_expires_at
                .is_none_or(|expires| expires < now)
        {
            return Ok(Err(GuichetResult::ClaimStale));
        }
        Ok(Ok(GuichetClaim {
            deposited_sequence: row.deposited_sequence,
            issuer_scope: row.issuer_scope,
            request_id: row.request_id,
            canonical_request: row.canonical_request,
            claimed_at: now,
            claim_generation: row.claim_generation,
            claim_token: row.claim_token.expect("claim courant sans token"),
            claim_lease_expires_at: row
                .claim_lease_expires_at
                .expect("claim courant sans lease"),
            expires_at: row.expires_at,
        }))
    }

    pub fn lookup_guichet(
        &self,
        issuer_scope: &str,
        request_id: &str,
        now: i64,
    ) -> Result<GuichetResult, StoreError> {
        Ok(guichet_row_for_key(&self.conn, issuer_scope, request_id)?
            .map(|row| guichet_existing_result(&row, now))
            .unwrap_or(GuichetResult::IdempotencyExpired))
    }

    /// Consomme un claim seulement si le détenteur, sa génération, son token
    /// et sa lease sont encore courants. Le résultat entier est durable avant
    /// toute réponse socket, ce qui rend le retry reconstructible après crash.
    pub fn reply_guichet(
        &mut self,
        owner: &str,
        input: GuichetReplyInput<'_>,
        now: i64,
    ) -> Result<GuichetResult, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let Some(row) = guichet_row_for_key(&tx, input.issuer_scope, input.request_id)? else {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::IdempotencyExpired);
        };
        if matches!(row.state.as_str(), "replied" | "rejected") {
            let result = if row.reply_bytes.as_deref() == Some(input.reply_bytes) {
                guichet_existing_result(&row, now)
            } else {
                GuichetResult::CanonicalBytesMismatch
            };
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(result);
        }
        if row
            .linked_request_id
            .as_deref()
            .is_some_and(|linked| linked != input.in_reply_to)
        {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::CanonicalBytesMismatch);
        }
        let issue = guichet_outcome_name(input.outcome).to_string();
        let changed = tx
            .execute(
                "UPDATE guichet_requests
                 SET state = CASE WHEN ?1 = 'refused' THEN 'rejected' ELSE 'replied' END,
                     result_issue = ?1, reply_bytes = ?2
                 WHERE issuer_scope = ?3 AND operation_kind = 'service_request'
                   AND request_id = ?4 AND state = 'claimed'
                   AND claim_owner = ?5 AND claim_generation = ?6
                   AND claim_token = ?7 AND claim_lease_expires_at >= ?8",
                params![
                    issue,
                    input.reply_bytes,
                    input.issuer_scope,
                    input.request_id,
                    owner,
                    input.generation as i64,
                    input.token,
                    now
                ],
            )
            .map_err(StoreError::Sqlite)?;
        if changed != 1 {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::ClaimStale);
        }
        if input.outcome == GuichetOutcome::Accepted
            && let Some(linked_request_id) = row.linked_request_id.as_deref()
        {
            // La clôture, lorsqu'elle est encore ouverte, est indissociable
            // du résultat guichet durable. Une demande déjà terminale relève
            // de D-208 : le rapport reste traçable sans la rouvrir.
            let answered =
                mark_answered_in_transaction(&tx, linked_request_id, &row.sender, "maicie")
                    .map_err(StoreError::Sqlite)?;
            if answered {
                record_lifecycle_event_in_transaction(
                    &tx,
                    &row.issuer_scope,
                    &row.request_id,
                    GuichetLifecycleState::Answered,
                    now,
                    Some(linked_request_id),
                    Some(input.response_message_id),
                )?;
            }
        }
        let expires_at = row.expires_at;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetResult::Terminal {
            issue,
            expires_at,
            newly_finalized: true,
        })
    }

    /// Lit les faits terminaux retenus par Bridget dans l'ordre d'observation.
    /// La réception est idempotente côté Maicie par `event_id`; une nouvelle
    /// connexion peut donc relever sans réinventer une transition.
    pub fn guichet_lifecycle_events(&self) -> Result<Vec<GuichetLifecycleEvent>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT issuer_scope, event_id, request_id, state, observed_at,
                        in_reply_to, response_message_id
                 FROM guichet_lifecycle_events
                 ORDER BY observed_at ASC, event_id ASC",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map([], lifecycle_event_from_row)
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)
    }

    /// Atteste une relance après son écriture transport. La clé
    /// `(request_id, generation)` empêche qu'un même palier soit exposé deux
    /// fois après un rejeu local ; les clients relisent ensuite les mêmes
    /// bytes via l'`event_id` durable.
    pub fn record_reminder_sent(
        &mut self,
        request_id: &str,
        reminder_message_id: &str,
        recipient: &str,
        generation: u64,
        observed_at: i64,
    ) -> Result<GuichetCoordinationEvent, StoreError> {
        if request_id.is_empty()
            || reminder_message_id.is_empty()
            || recipient.is_empty()
            || generation == 0
        {
            return Err(StoreError::Invariant("rappel de coordination incomplet"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let existing = tx
            .query_row(
                "SELECT event_id, request_id, kind, reminder_message_id, recipient,
                        generation, observed_at
                 FROM guichet_coordination_events
                 WHERE request_id = ?1 AND generation = ?2",
                params![request_id, generation as i64],
                coordination_event_from_row,
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        if let Some(event) = existing {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(event);
        }
        let event = GuichetCoordinationEvent {
            event_id: format!("evt-{}", Uuid::new_v4()),
            request_id: request_id.to_string(),
            kind: CoordinationEventKind::ReminderSent,
            reminder_message_id: reminder_message_id.to_string(),
            recipient: recipient.to_string(),
            generation,
            observed_at,
        };
        tx.execute(
            "INSERT INTO guichet_coordination_events
                 (event_id, request_id, kind, reminder_message_id, recipient,
                  generation, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.event_id,
                event.request_id,
                coordination_event_kind_name(event.kind),
                event.reminder_message_id,
                event.recipient,
                event.generation as i64,
                event.observed_at,
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(event)
    }

    /// Relève les faits 016 dans l'ordre attesté. T1604 ajoutera le curseur
    /// et les états de fraîcheur sans demander à Maicie une lecture SQLite.
    pub fn guichet_coordination_events(
        &self,
    ) -> Result<Vec<GuichetCoordinationEvent>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT event_id, request_id, kind, reminder_message_id, recipient,
                        generation, observed_at
                 FROM guichet_coordination_events
                 ORDER BY observed_at ASC, event_id ASC",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map([], coordination_event_from_row)
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)
    }

    /// La disparition d'une connexion ne laisse jamais son droit de claim
    /// actif : la prochaine relève récupère le même dépôt et une génération neuve.
    pub fn release_guichet_claims(&mut self, owner: &str) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE guichet_requests
                 SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                     claim_lease_expires_at = NULL
                 WHERE state = 'claimed' AND claim_owner = ?1",
                params![owner],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Uniquement au bootstrap du daemon : toute lease d'un processus arrêté
    /// redevient FIFO. Ouvrir un second handle SQLite ne doit jamais voler le
    /// claim vivant du premier.
    pub fn recover_guichet_claims_after_restart(&mut self) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE guichet_requests
                 SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                     claim_lease_expires_at = NULL
                 WHERE state = 'claimed'",
                [],
            )
            .map_err(StoreError::Sqlite)?;
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
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::Sqlite)?;
        record_message_in_transaction(&transaction, msg, conversation_key)
            .map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)?;
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

        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::Sqlite)?;
        let mut deleted = transaction
            .execute(
                "DELETE FROM ledger WHERE ts < ?1",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        transaction
            .execute(
                "DELETE FROM request_events
                 WHERE request_id IN (
                    SELECT id FROM tracked_requests
                    WHERE state != 'open' AND completed_at < ?1
                 )",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        transaction
            .execute(
                "DELETE FROM tracked_requests WHERE state != 'open' AND completed_at < ?1",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        deleted += transaction
            .execute(
                "DELETE FROM guichet_lifecycle_events
                 WHERE observed_at < ?1
                   AND EXISTS (
                     SELECT 1 FROM guichet_requests
                     WHERE guichet_requests.issuer_scope = guichet_lifecycle_events.issuer_scope
                       AND guichet_requests.request_id = guichet_lifecycle_events.request_id
                       AND guichet_requests.state IN ('replied', 'rejected')
                       AND guichet_requests.expires_at < ?1
                   )",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        deleted += transaction
            .execute(
                "DELETE FROM guichet_requests
                 WHERE state IN ('replied', 'rejected') AND expires_at < ?1",
                rusqlite::params![cutoff],
            )
            .map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)?;
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

/// Transition commune de résolution d'une demande, réutilisable lorsqu'une
/// opération adjacente doit être rendue atomique avec cette clôture.
pub(crate) fn mark_answered_in_transaction(
    transaction: &Transaction<'_>,
    id: &str,
    responder: &str,
    recipient: &str,
) -> Result<bool, rusqlite::Error> {
    let changed = transaction.execute(
        "UPDATE tracked_requests
         SET state = 'answered', completed_at = ?1
         WHERE id = ?2 AND sender = ?3 AND target = ?4 AND state = 'open'",
        rusqlite::params![now_secs(), id, recipient, responder],
    )?;
    Ok(changed == 1)
}

/// Insère le message livré dans le ledger sans quitter la transaction en
/// cours. La clé `(id, target)` rend une reprise du même message idempotente.
pub(crate) fn record_message_in_transaction(
    transaction: &Transaction<'_>,
    msg: &bridget_core::BridgetMessage,
    conversation_key: &str,
) -> Result<(), rusqlite::Error> {
    transaction.execute(
        "INSERT OR REPLACE INTO ledger (id, ts, sender, target, body, conversation_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            msg.id,
            now_secs(),
            msg.from,
            msg.to,
            msg.body,
            conversation_key,
        ],
    )?;
    Ok(())
}

fn record_lifecycle_event_in_transaction(
    transaction: &Transaction<'_>,
    issuer_scope: &str,
    request_id: &str,
    state: GuichetLifecycleState,
    observed_at: i64,
    in_reply_to: Option<&str>,
    response_message_id: Option<&str>,
) -> Result<GuichetLifecycleEvent, StoreError> {
    let existing = transaction
        .query_row(
            "SELECT issuer_scope, event_id, request_id, state, observed_at,
                    in_reply_to, response_message_id
             FROM guichet_lifecycle_events
             WHERE issuer_scope = ?1 AND request_id = ?2",
            params![issuer_scope, request_id],
            lifecycle_event_from_row,
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let event = GuichetLifecycleEvent {
        issuer_scope: issuer_scope.to_string(),
        event_id: format!("evt-{}", Uuid::new_v4()),
        request_id: request_id.to_string(),
        state,
        observed_at,
        in_reply_to: in_reply_to.map(str::to_string),
        response_message_id: response_message_id.map(str::to_string),
    };
    transaction
        .execute(
            "INSERT INTO guichet_lifecycle_events
                 (issuer_scope, request_id, event_id, state, observed_at,
                  in_reply_to, response_message_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.issuer_scope,
                event.request_id,
                event.event_id,
                lifecycle_state_name(event.state),
                event.observed_at,
                event.in_reply_to,
                event.response_message_id,
            ],
        )
        .map_err(StoreError::Sqlite)?;
    Ok(event)
}

fn record_lifecycle_for_linked_request_in_transaction(
    transaction: &Transaction<'_>,
    linked_request_id: &str,
    state: GuichetLifecycleState,
    observed_at: i64,
) -> Result<(), StoreError> {
    let deposits = {
        let mut statement = transaction
            .prepare(
                "SELECT issuer_scope, request_id
                 FROM guichet_requests
                 WHERE linked_request_id = ?1
                 ORDER BY deposited_sequence ASC",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map(params![linked_request_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)?
    };
    for (issuer_scope, request_id) in deposits {
        let _ = record_lifecycle_event_in_transaction(
            transaction,
            &issuer_scope,
            &request_id,
            state,
            observed_at,
            Some(linked_request_id),
            None,
        )?;
    }
    Ok(())
}

fn lifecycle_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GuichetLifecycleEvent> {
    let state = match row.get::<_, String>(3)?.as_str() {
        "answered" => GuichetLifecycleState::Answered,
        "cancelled" => GuichetLifecycleState::Cancelled,
        "timed_out" => GuichetLifecycleState::TimedOut,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(GuichetLifecycleEvent {
        issuer_scope: row.get(0)?,
        event_id: row.get(1)?,
        request_id: row.get(2)?,
        state,
        observed_at: row.get(4)?,
        in_reply_to: row.get(5)?,
        response_message_id: row.get(6)?,
    })
}

fn lifecycle_state_name(state: GuichetLifecycleState) -> &'static str {
    match state {
        GuichetLifecycleState::Answered => "answered",
        GuichetLifecycleState::Cancelled => "cancelled",
        GuichetLifecycleState::TimedOut => "timed_out",
    }
}

fn coordination_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<GuichetCoordinationEvent> {
    let kind = match row.get::<_, String>(2)?.as_str() {
        "reminder_sent" => CoordinationEventKind::ReminderSent,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(GuichetCoordinationEvent {
        event_id: row.get(0)?,
        request_id: row.get(1)?,
        kind,
        reminder_message_id: row.get(3)?,
        recipient: row.get(4)?,
        generation: row.get::<_, i64>(5)? as u64,
        observed_at: row.get(6)?,
    })
}

fn coordination_event_kind_name(kind: CoordinationEventKind) -> &'static str {
    match kind {
        CoordinationEventKind::ReminderSent => "reminder_sent",
    }
}

#[derive(Debug)]
struct GuichetRow {
    deposited_sequence: i64,
    issuer_scope: String,
    request_id: String,
    canonical_request: Vec<u8>,
    sender: String,
    expires_at: i64,
    state: String,
    claim_owner: Option<String>,
    claim_generation: u64,
    claim_token: Option<String>,
    claim_lease_expires_at: Option<i64>,
    result_issue: Option<String>,
    reply_bytes: Option<Vec<u8>>,
    linked_request_id: Option<String>,
}

fn guichet_row_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GuichetRow> {
    Ok(GuichetRow {
        deposited_sequence: row.get(0)?,
        issuer_scope: row.get(1)?,
        request_id: row.get(2)?,
        canonical_request: row.get(3)?,
        sender: row.get(4)?,
        expires_at: row.get(5)?,
        state: row.get(6)?,
        claim_owner: row.get(7)?,
        claim_generation: row.get::<_, i64>(8)? as u64,
        claim_token: row.get(9)?,
        claim_lease_expires_at: row.get(10)?,
        result_issue: row.get(11)?,
        reply_bytes: row.get(12)?,
        linked_request_id: row.get(13)?,
    })
}

fn guichet_row_for_key(
    conn: &Connection,
    issuer_scope: &str,
    request_id: &str,
) -> Result<Option<GuichetRow>, StoreError> {
    conn.query_row(
        "SELECT deposited_sequence, issuer_scope, request_id, canonical_request, sender,
                expires_at, state, claim_owner, claim_generation, claim_token,
                claim_lease_expires_at, result_issue, reply_bytes, linked_request_id
         FROM guichet_requests
         WHERE issuer_scope = ?1 AND operation_kind = 'service_request' AND request_id = ?2",
        params![issuer_scope, request_id],
        guichet_row_from_row,
    )
    .optional()
    .map_err(StoreError::Sqlite)
}

fn guichet_next_queued(conn: &Connection, now: i64) -> Result<Option<GuichetRow>, StoreError> {
    conn.query_row(
        "SELECT deposited_sequence, issuer_scope, request_id, canonical_request, sender,
                expires_at, state, claim_owner, claim_generation, claim_token,
                claim_lease_expires_at, result_issue, reply_bytes, linked_request_id
         FROM guichet_requests
         WHERE state = 'queued' AND expires_at >= ?1
         ORDER BY deposited_sequence ASC LIMIT 1",
        params![now],
        guichet_row_from_row,
    )
    .optional()
    .map_err(StoreError::Sqlite)
}

fn guichet_existing_result(row: &GuichetRow, now: i64) -> GuichetResult {
    if row.expires_at < now {
        return GuichetResult::IdempotencyExpired;
    }
    match row.state.as_str() {
        "queued" | "claimed" => GuichetResult::OutcomeUnknown {
            expires_at: row.expires_at,
        },
        "replied" | "rejected" => GuichetResult::Terminal {
            issue: row
                .result_issue
                .clone()
                .unwrap_or_else(|| "refused".to_string()),
            expires_at: row.expires_at,
            newly_finalized: false,
        },
        _ => GuichetResult::IdempotencyExpired,
    }
}

fn guichet_outcome_name(outcome: GuichetOutcome) -> &'static str {
    match outcome {
        GuichetOutcome::Accepted => "accepted",
        GuichetOutcome::RequestAlreadyTerminal => "request_already_terminal",
        GuichetOutcome::RecipientUnavailable => "recipient_unavailable",
        GuichetOutcome::Refused => "refused",
    }
}

fn linked_request_id(payload: &ServiceRequestPayload) -> Option<&str> {
    match payload {
        ServiceRequestPayload::DeliveryReport { in_reply_to, .. } => Some(in_reply_to),
        ServiceRequestPayload::Delegation { .. } => None,
    }
}

fn claim_token() -> String {
    // UUID v4 fournit 128 bits tirés par l'OS. Le contrat fixe une base64url
    // sans padding : 16 octets deviennent exactement 22 caractères opaques.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let bytes = Uuid::new_v4().into_bytes();
    let mut encoded = String::with_capacity(22);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(ALPHABET[(a >> 2) as usize] as char);
        encoded.push(ALPHABET[((a & 0x03) << 4 | b >> 4) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(ALPHABET[((b & 0x0f) << 2 | c >> 6) as usize] as char);
        }
        if chunk.len() > 2 {
            encoded.push(ALPHABET[(c & 0x3f) as usize] as char);
        }
    }
    encoded
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
    Invariant(&'static str),
    FrameTooLarge { max_frame_bytes: usize },
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "SQLite: {}", e),
            StoreError::Invariant(detail) => write!(f, "invariant store: {detail}"),
            StoreError::FrameTooLarge { max_frame_bytes } => {
                write!(f, "trame guichet supérieure à {max_frame_bytes} octets")
            }
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn guichet_deposit(request_id: &str, bytes: &[u8]) -> GuichetDeposit {
        GuichetDeposit {
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: request_id.to_string(),
            issued_at: 1_787_500_000,
            from: "codex-1".to_string(),
            operation: ServiceRequestOperation::MissionStatus,
            payload: ServiceRequestPayload::Delegation {
                delegation_id: "delegation-1".to_string(),
            },
            canonical_bytes: bytes.to_vec(),
        }
    }

    #[test]
    fn guichet_rejoue_les_octets_et_releve_fifo_apres_reouverture() {
        let path = std::env::temp_dir().join(format!("bridget-guichet-{}.db", Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let first = guichet_deposit("request-1", br#"{\"request\":1}"#);
        let second = guichet_deposit("request-2", br#"{\"request\":2}"#);
        assert!(matches!(
            store.deposit_guichet(&first, 600, 60, first.issued_at),
            Ok(GuichetResult::Queued { .. })
        ));
        assert!(matches!(
            store.deposit_guichet(&second, 600, 60, second.issued_at),
            Ok(GuichetResult::Queued { .. })
        ));
        let claim = match store
            .claim_next_guichet("service-a", first.issued_at)
            .unwrap()
        {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("premier dépôt FIFO absent"),
        };
        assert_eq!(claim.request_id, "request-1");
        drop(store);

        // Mutation discriminante : sans remise en file au redémarrage, le
        // premier dépôt resterait bloqué claimed et request-2 serait relevé.
        let mut reopened = Store::open(&path).unwrap();
        reopened.recover_guichet_claims_after_restart().unwrap();
        let replay = match reopened
            .claim_next_guichet("service-b", first.issued_at + 1)
            .unwrap()
        {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("dépôt persistant absent"),
        };
        assert_eq!(replay.request_id, "request-1");
        assert_ne!(replay.claim_token, claim.claim_token);
        assert_eq!(replay.claim_generation, claim.claim_generation + 1);
        assert!(matches!(
            reopened.deposit_guichet(&first, 600, 60, first.issued_at + 1),
            Ok(GuichetResult::OutcomeUnknown { .. })
        ));
        assert!(matches!(
            reopened.deposit_guichet(
                &guichet_deposit("request-1", br#"{\"request\":9}"#),
                600,
                60,
                first.issued_at + 1
            ),
            Ok(GuichetResult::CanonicalBytesMismatch)
        ));
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn depot_direct_refuse_une_trame_guichet_superieure_a_64_kio() {
        let path =
            std::env::temp_dir().join(format!("bridget-guichet-frame-limit-{}.db", Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let now = 1_787_500_000;
        let exact = guichet_deposit("request-exact", &vec![b'x'; MAX_GUICHET_FRAME_BYTES - 1]);
        let oversized = guichet_deposit("request-oversized", &vec![b'x'; MAX_GUICHET_FRAME_BYTES]);

        assert!(matches!(
            store.deposit_guichet(&exact, 600, 60, now),
            Ok(GuichetResult::Queued { .. })
        ));
        assert!(matches!(
            store.deposit_guichet(&oversized, 600, 60, now),
            Err(StoreError::FrameTooLarge {
                max_frame_bytes: MAX_GUICHET_FRAME_BYTES
            })
        ));
        let oversized_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM guichet_requests WHERE request_id = ?1",
                [&oversized.request_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(oversized_count, 0, "le refus précède toute persistance");
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn schema_guichet_ne_cree_plus_la_colonne_result_bytes() {
        let path =
            std::env::temp_dir().join(format!("bridget-guichet-schema-{}.db", Uuid::new_v4()));
        let store = Store::open(&path).unwrap();
        let mut statement = store
            .conn
            .prepare("PRAGMA table_info(guichet_requests)")
            .unwrap();
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!columns.iter().any(|column| column == "result_bytes"));
        assert!(columns.iter().any(|column| column == "reply_bytes"));
        drop(statement);
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn second_handle_ne_revoque_un_claim_qu_au_bootstrap_explicite() {
        let path =
            std::env::temp_dir().join(format!("bridget-guichet-handles-{}.db", Uuid::new_v4()));
        let mut first = Store::open(&path).unwrap();
        let now = 1_787_500_000;
        let finalized = guichet_deposit("request-finalized", br#"{\"request\":1}"#);
        let recovered = guichet_deposit("request-recovered", br#"{\"request\":2}"#);
        first.deposit_guichet(&finalized, 600, 60, now).unwrap();
        first.deposit_guichet(&recovered, 600, 60, now).unwrap();
        let claim_finalized = match first.claim_next_guichet("service-a", now).unwrap() {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("claim A finalisable absent"),
        };

        // Ouvrir SQLite une seconde fois n'est pas un redémarrage : la lease
        // d'A reste active et A peut encore finaliser son premier dépôt.
        let mut second = Store::open(&path).unwrap();
        assert!(matches!(
            first.reply_guichet(
                "service-a",
                GuichetReplyInput {
                    issuer_scope: &claim_finalized.issuer_scope,
                    request_id: &claim_finalized.request_id,
                    generation: claim_finalized.claim_generation,
                    token: &claim_finalized.claim_token,
                    response_message_id: "reply-finalized",
                    reply_bytes: br#"{\"reply\":\"a\"}"#,
                    in_reply_to: "",
                    outcome: GuichetOutcome::Accepted,
                },
                now,
            ),
            Ok(GuichetResult::Terminal { ref issue, .. }) if issue == "accepted"
        ));

        let claim_recovered = match first.claim_next_guichet("service-a", now).unwrap() {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("claim A à reprendre absent"),
        };
        assert_eq!(claim_recovered.request_id, "request-recovered");

        // Mutation discriminante : si Store::open libérait encore les leases,
        // le claim suivant d'A ou la relève de B ne prouverait plus que seul le
        // bootstrap remet les claims vivants en FIFO.
        second.recover_guichet_claims_after_restart().unwrap();
        let claimed_by_b = match second.claim_next_guichet("service-b", now + 1).unwrap() {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("claim B repris absent"),
        };
        assert_eq!(
            claimed_by_b.deposited_sequence,
            claim_recovered.deposited_sequence
        );
        assert_eq!(claimed_by_b.request_id, claim_recovered.request_id);
        assert_eq!(
            claimed_by_b.claim_generation,
            claim_recovered.claim_generation + 1
        );
        assert_ne!(claimed_by_b.claim_token, claim_recovered.claim_token);
        drop(first);
        drop(second);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn guichet_reply_stale_ne_peut_pas_gagner_apres_lease_expire() {
        let path =
            std::env::temp_dir().join(format!("bridget-guichet-lease-{}.db", Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let deposit = guichet_deposit("request-lease", br#"{\"request\":1}"#);
        let now = deposit.issued_at;
        store.deposit_guichet(&deposit, 600, 60, now).unwrap();
        let a = match store.claim_next_guichet("service-a", now).unwrap() {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("claim A absent"),
        };
        let b = match store
            .claim_next_guichet("service-b", a.claim_lease_expires_at + 1)
            .unwrap()
        {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("claim B absent"),
        };
        assert!(matches!(
            store.reply_guichet(
                "service-a",
                GuichetReplyInput {
                    issuer_scope: &a.issuer_scope,
                    request_id: &a.request_id,
                    generation: a.claim_generation,
                    token: &a.claim_token,
                    response_message_id: "reply-stale",
                    reply_bytes: br#"{\"reply\":\"a\"}"#,
                    in_reply_to: "",
                    outcome: GuichetOutcome::Accepted,
                },
                b.claim_lease_expires_at - 1,
            ),
            Ok(GuichetResult::ClaimStale)
        ));
        assert!(matches!(
            store.reply_guichet(
                "service-b", GuichetReplyInput {
                    issuer_scope: &b.issuer_scope, request_id: &b.request_id,
                    generation: b.claim_generation, token: &b.claim_token,
                    response_message_id: "reply-current",
                    reply_bytes: br#"{\"reply\":\"b\"}"#, in_reply_to: "",
                    outcome: GuichetOutcome::Accepted,
                },
                b.claim_lease_expires_at - 1,
            ),
            Ok(GuichetResult::Terminal { ref issue, .. }) if issue == "accepted"
        ));
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn reponse_guichet_accepted_clot_atomiquement_la_demande_liee() {
        let path =
            std::env::temp_dir().join(format!("bridget-guichet-answer-{}.db", Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let now = 1_787_500_000;
        store
            .create_request("message-lie", "maicie", "codex-1", 60)
            .unwrap();
        let deposit = GuichetDeposit {
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "request-answer".to_string(),
            issued_at: now,
            from: "codex-1".to_string(),
            operation: ServiceRequestOperation::DeliveryReport,
            payload: ServiceRequestPayload::DeliveryReport {
                objective_id: "objective-1".to_string(),
                delegation_id: "delegation-1".to_string(),
                delivery_hash: "0".repeat(64),
                in_reply_to: "message-lie".to_string(),
            },
            canonical_bytes: br#"{"type":"service_request"}"#.to_vec(),
        };
        store.deposit_guichet(&deposit, 600, 60, now).unwrap();
        let claim = match store.claim_next_guichet("maicie-connection", now).unwrap() {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("dépôt lié absent"),
        };
        assert!(matches!(
            store.reply_guichet(
                "maicie-connection",
                GuichetReplyInput {
                    issuer_scope: &claim.issuer_scope,
                    request_id: &claim.request_id,
                    generation: claim.claim_generation,
                    token: &claim.claim_token,
                    response_message_id: "reply-linked",
                    reply_bytes: br#"{"type":"guichet_reply"}"#,
                    in_reply_to: "message-lie",
                    outcome: GuichetOutcome::Accepted,
                },
                now,
            ),
            Ok(GuichetResult::Terminal { ref issue, .. }) if issue == "accepted"
        ));
        assert_eq!(
            store.get_request("message-lie").unwrap().unwrap().state,
            "answered",
            "mutation discriminante : sans mark_answered_in_transaction dans la transaction du reply, la demande resterait open"
        );
        let events = store.guichet_lifecycle_events().unwrap();
        assert!(matches!(
            events.as_slice(),
            [GuichetLifecycleEvent {
                request_id,
                state: GuichetLifecycleState::Answered,
                in_reply_to: Some(in_reply_to),
                response_message_id: Some(response_message_id),
                ..
            }] if request_id == "request-answer"
                && in_reply_to == "message-lie"
                && response_message_id == "reply-linked"
        ));

        // Mutation discriminante : si l'événement était écrit hors de la
        // transaction de clôture, un rejeu pourrait créer une seconde preuve
        // ou laisser une demande answered sans fait durable correspondant.
        assert!(matches!(
            store.reply_guichet(
                "maicie-connection",
                GuichetReplyInput {
                    issuer_scope: &claim.issuer_scope,
                    request_id: &claim.request_id,
                    generation: claim.claim_generation,
                    token: &claim.claim_token,
                    response_message_id: "reply-linked",
                    reply_bytes: br#"{"type":"guichet_reply"}"#,
                    in_reply_to: "message-lie",
                    outcome: GuichetOutcome::Accepted,
                },
                now + 1,
            ),
            Ok(GuichetResult::Terminal {
                newly_finalized: false,
                ..
            })
        ));
        assert_eq!(store.guichet_lifecycle_events().unwrap().len(), 1);
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn guichet_claim_next_est_atomique_entre_deux_stores() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let path = std::env::temp_dir().join(format!("bridget-guichet-race-{}.db", Uuid::new_v4()));
        let mut seed = Store::open(&path).unwrap();
        let now = 1_787_500_000;
        for request_id in ["request-a", "request-b"] {
            let deposit = guichet_deposit(request_id, request_id.as_bytes());
            seed.deposit_guichet(&deposit, 600, 60, now).unwrap();
        }
        drop(seed);
        let mut first = Store::open(&path).unwrap();
        let mut second = Store::open(&path).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let left = Arc::clone(&barrier);
        let one = thread::spawn(move || {
            left.wait();
            first.claim_next_guichet("service-a", now).unwrap()
        });
        let right = Arc::clone(&barrier);
        let two = thread::spawn(move || {
            right.wait();
            second.claim_next_guichet("service-b", now).unwrap()
        });
        let claimed = [one.join().unwrap(), two.join().unwrap()]
            .into_iter()
            .map(|next| match next {
                GuichetNext::Claimed(claim) => claim.request_id,
                GuichetNext::Empty => panic!("deux dépôts FIFO devaient être relevés"),
            })
            .collect::<std::collections::BTreeSet<_>>();
        // Mutation discriminante : un SELECT puis UPDATE non IMMEDIATE peut
        // donner request-a deux fois ou échouer au lieu de distribuer A puis B.
        assert_eq!(
            claimed,
            ["request-a".to_string(), "request-b".to_string()].into()
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cancellation_is_idempotent_and_terminal() {
        let path = std::env::temp_dir().join(format!("bridget-store-{}.db", Uuid::new_v4()));
        let _ = std::fs::remove_file(&path);
        let mut store = Store::open(&path).unwrap();
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

    #[test]
    fn transitions_cancelled_et_timed_out_deposent_un_fait_guichet_unique() {
        let path =
            std::env::temp_dir().join(format!("bridget-guichet-terminal-{}.db", Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let now = 1_787_500_000;
        for (request_id, state) in [
            ("message-cancelled", GuichetLifecycleState::Cancelled),
            ("message-timed-out", GuichetLifecycleState::TimedOut),
        ] {
            store
                .create_request(request_id, "alice", "bob", 60)
                .unwrap();
            let deposit = GuichetDeposit {
                issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
                request_id: format!("request-{request_id}"),
                issued_at: now,
                from: "alice".to_string(),
                operation: ServiceRequestOperation::DeliveryReport,
                payload: ServiceRequestPayload::DeliveryReport {
                    objective_id: "objective-1".to_string(),
                    delegation_id: "delegation-1".to_string(),
                    delivery_hash: "0".repeat(64),
                    in_reply_to: request_id.to_string(),
                },
                canonical_bytes: format!("{{\"request\":\"{request_id}\"}}").into_bytes(),
            };
            store.deposit_guichet(&deposit, 600, 60, now).unwrap();
            match state {
                GuichetLifecycleState::Cancelled => {
                    store
                        .cancel_request(request_id, "alice", Some("annulé"))
                        .unwrap();
                    store
                        .cancel_request(request_id, "alice", Some("rejeu"))
                        .unwrap();
                }
                GuichetLifecycleState::TimedOut => {
                    assert!(store.mark_timed_out(request_id).unwrap());
                    assert!(!store.mark_timed_out(request_id).unwrap());
                }
                GuichetLifecycleState::Answered => unreachable!(),
            }
        }
        let events = store.guichet_lifecycle_events().unwrap();
        assert!(events.iter().any(|event| {
            event.request_id == "request-message-cancelled"
                && event.state == GuichetLifecycleState::Cancelled
                && event.in_reply_to.as_deref() == Some("message-cancelled")
        }));
        assert!(events.iter().any(|event| {
            event.request_id == "request-message-timed-out"
                && event.state == GuichetLifecycleState::TimedOut
                && event.in_reply_to.as_deref() == Some("message-timed-out")
        }));
        assert_eq!(
            events.len(),
            2,
            "chaque transition terminale ne dépose qu'un seul fait"
        );
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn deferred_reminder_is_persisted_for_ledger_readers() {
        let path = std::env::temp_dir().join(format!("bridget-store-events-{}.db", Uuid::new_v4()));
        let _ = std::fs::remove_file(&path);
        let store = Store::open(&path).unwrap();
        store
            .create_request("request-1", "alice", "bob", 60)
            .unwrap();
        store.record_deferred_reminder("request-1", 2).unwrap();
        assert_eq!(
            store
                .latest_deferred_reminder("request-1")
                .unwrap()
                .map(|event| event.0),
            Some(2)
        );
        drop(store);
        let reopened = Store::open(&path).unwrap();
        assert_eq!(
            reopened
                .latest_deferred_reminder("request-1")
                .unwrap()
                .map(|event| event.0),
            Some(2)
        );
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn purge_supprime_avec_la_demande_les_evenements_associes() {
        let path = std::env::temp_dir().join(format!("bridget-store-purge-{}.db", Uuid::new_v4()));
        let _ = std::fs::remove_file(&path);
        let mut store = Store::open(&path).unwrap();
        store
            .create_request("request-1", "alice", "bob", 60)
            .unwrap();
        assert!(store.mark_timed_out("request-1").unwrap());
        store.record_deferred_reminder("request-1", 2).unwrap();
        store
            .conn
            .execute(
                "UPDATE tracked_requests SET completed_at = ?1 WHERE id = ?2",
                rusqlite::params![now_secs() - 86_401, "request-1"],
            )
            .unwrap();

        store.purge_older_than_days(1).unwrap();

        assert!(store.get_request("request-1").unwrap().is_none());
        assert!(
            store
                .latest_deferred_reminder("request-1")
                .unwrap()
                .is_none()
        );
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn purge_guichet_conserve_les_demandes_encore_vivantes() {
        let path =
            std::env::temp_dir().join(format!("bridget-store-purge-guichet-{}.db", Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let now = now_secs();
        let cutoff = now - 86_400;
        let mut terminal = guichet_deposit("request-terminal", br#"{\"request\":1}"#);
        terminal.issued_at = now;
        let mut live = guichet_deposit("request-live", br#"{\"request\":2}"#);
        live.issued_at = now;
        store.deposit_guichet(&terminal, 600, 60, now).unwrap();
        store.deposit_guichet(&live, 600, 60, now).unwrap();
        store
            .conn
            .execute(
                "UPDATE guichet_requests
                 SET state = 'replied', result_issue = 'accepted', expires_at = ?1
                 WHERE request_id = ?2",
                params![cutoff - 1, terminal.request_id],
            )
            .unwrap();
        store
            .conn
            .execute(
                "UPDATE guichet_requests SET expires_at = ?1 WHERE request_id = ?2",
                params![cutoff - 1, live.request_id],
            )
            .unwrap();
        for deposit in [&terminal, &live] {
            store
                .conn
                .execute(
                    "INSERT INTO guichet_lifecycle_events
                     (issuer_scope, request_id, event_id, state, observed_at)
                     VALUES (?1, ?2, ?3, 'answered', ?4)",
                    params![
                        deposit.issuer_scope,
                        deposit.request_id,
                        format!("event-{}", deposit.request_id),
                        cutoff - 1
                    ],
                )
                .unwrap();
        }

        store.purge_older_than_days(1).unwrap();

        let terminal_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM guichet_requests WHERE request_id = ?1",
                [&terminal.request_id],
                |row| row.get(0),
            )
            .unwrap();
        let live_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM guichet_requests WHERE request_id = ?1",
                [&live.request_id],
                |row| row.get(0),
            )
            .unwrap();
        let terminal_event_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM guichet_lifecycle_events WHERE request_id = ?1",
                [&terminal.request_id],
                |row| row.get(0),
            )
            .unwrap();
        let live_event_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM guichet_lifecycle_events WHERE request_id = ?1",
                [&live.request_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(terminal_count, 0);
        assert_eq!(terminal_event_count, 0);
        assert_eq!(live_count, 1, "une demande non terminale reste vivante");
        assert_eq!(
            live_event_count, 1,
            "l'événement d'une demande vivante ne doit pas être purgé"
        );
        drop(store);
        let _ = std::fs::remove_file(path);
    }
}
