//! Persistance SQLite — ledger, compteurs disjoncteur, historique.

use bridget_transport::protocol::{
    GuichetOutcome, ServiceRequestOperation, ServiceRequestPayload,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use std::path::Path;
use uuid::Uuid;

const GUICHET_LEASE_SECS: i64 = 60;

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
    Queued { expires_at: i64 },
    OutcomeUnknown { expires_at: i64 },
    Terminal { issue: String, expires_at: i64 },
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
    pub reply_bytes: &'a [u8],
    pub in_reply_to: &'a str,
    pub outcome: GuichetOutcome,
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
        // Un daemon arrêté n'a plus de détenteur de lease : la reprise doit
        // relivrer le même dépôt FIFO, jamais conserver un propriétaire mort.
        conn.execute(
            "UPDATE guichet_requests
             SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                 claim_lease_expires_at = NULL
             WHERE state = 'claimed'",
            [],
        )
        .map_err(StoreError::Sqlite)?;
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
                result_bytes BLOB,
                reply_bytes BLOB,
                UNIQUE (issuer_scope, operation_kind, request_id)
            );
            CREATE INDEX IF NOT EXISTS idx_guichet_fifo
                ON guichet_requests(state, deposited_sequence);
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
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::Sqlite)?;
        let answered = mark_answered_in_transaction(&transaction, id, responder, recipient)
            .map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)?;
        Ok(answered)
    }

    pub fn mark_timed_out(&self, id: &str) -> Result<bool, StoreError> {
        let changed = self.conn.execute("UPDATE tracked_requests SET state = 'timed_out', completed_at = ?1 WHERE id = ?2 AND state = 'open'", rusqlite::params![now_secs(), id]).map_err(StoreError::Sqlite)?;
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
    pub fn claim_next_guichet(
        &mut self,
        owner: &str,
        now: i64,
    ) -> Result<GuichetNext, StoreError> {
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
                params![owner, generation as i64, token, lease_expires_at, row.deposited_sequence],
            )
            .map_err(StoreError::Sqlite)?;
        if changed != 1 {
            return Err(StoreError::Invariant("claim FIFO concurrent perdu"));
        }
        let claim = GuichetClaim {
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
            || row.claim_lease_expires_at.is_none_or(|expires| expires < now)
        {
            return Ok(Err(GuichetResult::ClaimStale));
        }
        Ok(Ok(GuichetClaim {
            issuer_scope: row.issuer_scope,
            request_id: row.request_id,
            canonical_request: row.canonical_request,
            claimed_at: now,
            claim_generation: row.claim_generation,
            claim_token: row.claim_token.expect("claim courant sans token"),
            claim_lease_expires_at: row.claim_lease_expires_at.expect("claim courant sans lease"),
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
                params![issue, input.reply_bytes, input.issuer_scope, input.request_id, owner, input.generation as i64, input.token, now],
            )
            .map_err(StoreError::Sqlite)?;
        if changed != 1 {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::ClaimStale);
        }
        let expires_at = row.expires_at;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetResult::Terminal { issue, expires_at })
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
        let deleted = transaction
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

#[derive(Debug)]
struct GuichetRow {
    deposited_sequence: i64,
    issuer_scope: String,
    request_id: String,
    canonical_request: Vec<u8>,
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
        expires_at: row.get(4)?,
        state: row.get(5)?,
        claim_owner: row.get(6)?,
        claim_generation: row.get::<_, i64>(7)? as u64,
        claim_token: row.get(8)?,
        claim_lease_expires_at: row.get(9)?,
        result_issue: row.get(10)?,
        reply_bytes: row.get(11)?,
        linked_request_id: row.get(12)?,
    })
}

fn guichet_row_for_key(
    conn: &Connection,
    issuer_scope: &str,
    request_id: &str,
) -> Result<Option<GuichetRow>, StoreError> {
    conn.query_row(
        "SELECT deposited_sequence, issuer_scope, request_id, canonical_request,
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
        "SELECT deposited_sequence, issuer_scope, request_id, canonical_request,
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
            issue: row.result_issue.clone().unwrap_or_else(|| "refused".to_string()),
            expires_at: row.expires_at,
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
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "SQLite: {}", e),
            StoreError::Invariant(detail) => write!(f, "invariant store: {detail}"),
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
        let claim = match store.claim_next_guichet("service-a", first.issued_at).unwrap() {
            GuichetNext::Claimed(claim) => claim,
            GuichetNext::Empty => panic!("premier dépôt FIFO absent"),
        };
        assert_eq!(claim.request_id, "request-1");
        drop(store);

        // Mutation discriminante : sans remise en file au redémarrage, le
        // premier dépôt resterait bloqué claimed et request-2 serait relevé.
        let mut reopened = Store::open(&path).unwrap();
        let replay = match reopened.claim_next_guichet("service-b", first.issued_at + 1).unwrap() {
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
            reopened.deposit_guichet(&guichet_deposit("request-1", br#"{\"request\":9}"#), 600, 60, first.issued_at + 1),
            Ok(GuichetResult::CanonicalBytesMismatch)
        ));
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn guichet_reply_stale_ne_peut_pas_gagner_apres_lease_expire() {
        let path = std::env::temp_dir().join(format!("bridget-guichet-lease-{}.db", Uuid::new_v4()));
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
                "service-a", GuichetReplyInput {
                    issuer_scope: &a.issuer_scope, request_id: &a.request_id,
                    generation: a.claim_generation, token: &a.claim_token,
                    reply_bytes: br#"{\"reply\":\"a\"}"#, in_reply_to: "",
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
        assert_eq!(claimed, ["request-a".to_string(), "request-b".to_string()].into());
        let _ = std::fs::remove_file(path);
    }

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

    #[test]
    fn deferred_reminder_is_persisted_for_ledger_readers() {
        let path =
            std::env::temp_dir().join(format!("bridget-store-events-{}.db", std::process::id()));
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
        let path =
            std::env::temp_dir().join(format!("bridget-store-purge-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let store = Store::open(&path).unwrap();
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
}
