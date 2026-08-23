//! Persistance SQLite privée de Maicie.
//!
//! La décision locale, la délégation et son outbox sont écrites dans une
//! transaction unique. Aucune méthode de reprise ne reconstruit l'enveloppe :
//! les octets préparés avant I/O sont l'autorité.

use crate::bridget_client::IdempotencyIssue;
use crate::domain::{EtatDelegation, EtatObjectif, EtatOutboxDelegation, ObjectifCoordonne};
use crate::outbox::{
    OutboxError, PendingDelegationOutbox, PreparedDelegation, RecoverySnapshot, StoreCommitPhase,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Value, json};
use std::fmt;
use std::fs::{self, DirBuilder, OpenOptions};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 2;
const DATABASE_MODE: u32 = 0o600;
const DIRECTORY_MODE: u32 = 0o700;

/// Autorité d'écriture unique de l'état Maicie.
pub struct MaicieStore {
    path: PathBuf,
    connection: Connection,
    issuer_scope: String,
}

impl MaicieStore {
    /// Ouvre la base privée, applique les migrations idempotentes et charge
    /// l'identité stable utilisée par le contrat client Bridget.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        validate_database_path(path)?;
        prepare_private_database(path)?;

        let mut connection = Connection::open(path).map_err(StoreError::Sql)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(StoreError::Sql)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;\n\
                 PRAGMA journal_mode = WAL;\n\
                 PRAGMA synchronous = FULL;",
            )
            .map_err(StoreError::Sql)?;
        migrate(&mut connection)?;
        let issuer_scope = load_or_create_issuer_scope(&mut connection)?;

        Ok(Self {
            path: path.to_path_buf(),
            connection,
            issuer_scope,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn issuer_scope(&self) -> &str {
        &self.issuer_scope
    }

    pub fn schema_version(&self) -> Result<i64, StoreError> {
        self.connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(StoreError::Sql)
    }

    /// Écrit l'agrégat complet dans une seule transaction SQLite.
    pub fn create_prepared_delegation(
        &mut self,
        prepared: &PreparedDelegation,
    ) -> Result<(), StoreError> {
        self.create_prepared_delegation_observed(prepared, |_| Ok(()))
    }

    /// Variante de production instrumentée par les crash-tests réels.
    pub fn create_prepared_delegation_observed(
        &mut self,
        prepared: &PreparedDelegation,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        prepared.validate().map_err(StoreError::Outbox)?;
        if prepared.issuer_scope != self.issuer_scope {
            return Err(StoreError::Conflict(
                "issuer_scope différent de l'identité durable du store",
            ));
        }

        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        insert_prepared(&tx, prepared)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(())
    }

    /// Retourne uniquement les outboxes non terminales, avec l'enveloppe
    /// exacte nécessaire au lookup puis au replay de T008.
    pub fn pending_delegation_outboxes(&self) -> Result<Vec<PendingDelegationOutbox>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT objective_id, delegation_id, message_id, issuer_scope, issued_at,\n\
                        target, body_bytes, reply, timeout_secs, deadline_contractuelle,\n\
                        body_hash, message_bytes, state, attempted_at, retry_until,\n\
                        dedup_retained_until\n\
                 FROM delegation_outbox\n\
                 WHERE terminal = 0 AND state IN ('prepared', 'outcome_unknown')\n\
                 ORDER BY issued_at, message_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], raw_pending_from_row)
            .map_err(StoreError::Sql)?;
        rows.map(|row| row.map_err(StoreError::Sql)?.try_into())
            .collect()
    }

    /// Persiste l'issue observée par lookup. Une issue terminale retire
    /// l'enveloppe de la liste de reprise sans supprimer sa preuve.
    pub fn record_lookup_issue(
        &mut self,
        message_id: Uuid,
        issue: &IdempotencyIssue,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        let next_state = match issue {
            IdempotencyIssue::Accepted { .. } => EtatOutboxDelegation::Accepted,
            IdempotencyIssue::OutcomeUnknown { .. } => EtatOutboxDelegation::OutcomeUnknown,
            IdempotencyIssue::Rejected { .. }
            | IdempotencyIssue::EnvelopeMismatch
            | IdempotencyIssue::IdempotencyExpired
            | IdempotencyIssue::InvalidIssuedAt => EtatOutboxDelegation::Rejected,
        };
        let terminal = matches!(
            next_state,
            EtatOutboxDelegation::Accepted | EtatOutboxDelegation::Rejected
        );
        let issue_json = encode_issue(issue);
        let issue_bytes = serde_json::to_vec(&issue_json).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current: Option<(String, i64, Option<Vec<u8>>)> = tx
            .query_row(
                "SELECT state, terminal, last_issue_json\n\
                 FROM delegation_outbox WHERE message_id = ?1",
                [message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((current_state_name, current_terminal, current_issue)) = current else {
            return Err(StoreError::NotFound("message_id inconnu"));
        };
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict("issue terminale déjà figée"));
        }
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != next_state {
            current_state
                .transition_vers(next_state)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = ?1, terminal = ?2, last_issue_json = ?3,\n\
                     issue_observed_at = ?4, attempted_at = COALESCE(attempted_at, ?4)\n\
                 WHERE message_id = ?5 AND state = ?6 AND terminal = 0",
                params![
                    outbox_state_name(next_state),
                    i64::from(terminal),
                    issue_bytes,
                    observed_at,
                    message_id.to_string(),
                    current_state_name
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant la transition",
            ));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Marque une transmission sans issue durable. La ligne reste éligible au
    /// lookup de reprise ; aucun nouvel identifiant n'est créé.
    pub fn record_transport_uncertainty(
        &mut self,
        message_id: Uuid,
        attempted_at: i64,
    ) -> Result<(), StoreError> {
        if attempted_at <= 0 {
            return Err(StoreError::Invalid("attempted_at invalide"));
        }
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current_state_name: Option<String> = tx
            .query_row(
                "SELECT state FROM delegation_outbox\n\
                 WHERE message_id = ?1 AND terminal = 0",
                [message_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some(current_state_name) = current_state_name else {
            return Err(StoreError::NotFound("outbox absente ou terminale"));
        };
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != EtatOutboxDelegation::OutcomeUnknown {
            current_state
                .transition_vers(EtatOutboxDelegation::OutcomeUnknown)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = 'outcome_unknown', attempted_at = ?1\n\
                 WHERE message_id = ?2 AND state = ?3 AND terminal = 0",
                params![attempted_at, message_id.to_string(), current_state_name],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant la transition",
            ));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Snapshot corrélé à la délégation, utilisable par T008 sans second
    /// lookup local ni reconstruction de payload.
    pub fn recovery_snapshot(
        &self,
        message_id: Uuid,
    ) -> Result<Option<RecoverySnapshot>, StoreError> {
        let raw = self
            .connection
            .query_row(
                "SELECT o.objective_id, o.delegation_id, o.message_id, o.issuer_scope,\n\
                        o.issued_at, o.target, o.body_bytes, o.reply, o.timeout_secs,\n\
                        o.deadline_contractuelle, o.body_hash, o.message_bytes, o.state,\n\
                        o.attempted_at, o.retry_until, o.dedup_retained_until,\n\
                        obj.state, d.state, o.last_issue_json, o.issue_observed_at\n\
                 FROM delegation_outbox o\n\
                 JOIN objectives obj ON obj.id = o.objective_id\n\
                 JOIN delegations d ON d.id = o.delegation_id\n\
                 WHERE o.message_id = ?1",
                [message_id.to_string()],
                |row| {
                    Ok(RawRecovery {
                        pending: raw_pending_from_row(row)?,
                        objective_state: row.get(16)?,
                        delegation_state: row.get(17)?,
                        last_issue_json: row.get(18)?,
                        issue_observed_at: row.get(19)?,
                    })
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        raw.map(TryInto::try_into).transpose()
    }
}

fn validate_database_path(path: &Path) -> Result<(), StoreError> {
    if !path.is_absolute() {
        return Err(StoreError::Invalid("chemin SQLite non absolu"));
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("bridget.db") {
        return Err(StoreError::Invalid(
            "bridget.db appartient exclusivement à Bridget",
        ));
    }
    if path.parent().is_none() {
        return Err(StoreError::Invalid("base SQLite sans répertoire parent"));
    }
    Ok(())
}

fn prepare_private_database(path: &Path) -> Result<(), StoreError> {
    let parent = path.parent().expect("validé");
    if !parent.exists() {
        let mut builder = DirBuilder::new();
        builder.recursive(true).mode(DIRECTORY_MODE);
        builder.create(parent).map_err(StoreError::Io)?;
    }
    let parent_metadata = fs::symlink_metadata(parent).map_err(StoreError::Io)?;
    if !parent_metadata.file_type().is_dir()
        || parent_metadata.permissions().mode() & 0o777 != DIRECTORY_MODE
    {
        return Err(StoreError::Invalid(
            "répertoire SQLite non privé (0700 requis)",
        ));
    }

    if path.exists() {
        let metadata = fs::symlink_metadata(path).map_err(StoreError::Io)?;
        if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o777 != DATABASE_MODE
        {
            return Err(StoreError::Invalid(
                "fichier SQLite non privé (0600 requis)",
            ));
        }
    } else {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(DATABASE_MODE)
            .open(path)
            .map_err(StoreError::Io)?;
    }
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), StoreError> {
    let current_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(StoreError::Sql)?;
    if current_version > SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema {
            found: current_version,
            supported: SCHEMA_VERSION,
        });
    }
    let tx = connection.transaction().map_err(StoreError::Sql)?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\n\
             version INTEGER PRIMARY KEY,\n\
             applied_at INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS maicie_identity (\n\
             singleton INTEGER PRIMARY KEY CHECK(singleton = 1),\n\
             issuer_scope TEXT NOT NULL UNIQUE\n\
         );\n\
         CREATE TABLE IF NOT EXISTS objectives (\n\
             id TEXT PRIMARY KEY,\n\
             state TEXT NOT NULL,\n\
             payload_json BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS delegations (\n\
             id TEXT PRIMARY KEY,\n\
             objective_id TEXT NOT NULL REFERENCES objectives(id),\n\
             state TEXT NOT NULL,\n\
             payload_json BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS delegation_outbox (\n\
             message_id TEXT PRIMARY KEY,\n\
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),\n\
             objective_id TEXT NOT NULL REFERENCES objectives(id),\n\
             issuer_scope TEXT NOT NULL,\n\
             issued_at INTEGER NOT NULL,\n\
             target TEXT NOT NULL,\n\
             body_bytes BLOB NOT NULL,\n\
             reply INTEGER NOT NULL CHECK(reply IN (0, 1)),\n\
             timeout_secs INTEGER NOT NULL,\n\
             deadline_contractuelle INTEGER NOT NULL,\n\
             body_hash BLOB NOT NULL,\n\
             message_bytes BLOB NOT NULL,\n\
             state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),\n\
             attempted_at INTEGER,\n\
             retry_until INTEGER NOT NULL,\n\
             dedup_retained_until INTEGER NOT NULL,\n\
             last_issue_json BLOB,\n\
             issue_observed_at INTEGER,\n\
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))\n\
         );\n\
         CREATE INDEX IF NOT EXISTS objectives_open_idx\n\
             ON objectives(state) WHERE state != 'clos';\n\
         CREATE INDEX IF NOT EXISTS delegation_outbox_pending_idx\n\
             ON delegation_outbox(terminal, state, retry_until, message_id);",
    )
    .map_err(StoreError::Sql)?;
    if current_version == 1 {
        migrate_outbox_to_rejected_state(&tx)?;
    }
    for version in (current_version + 1)..=SCHEMA_VERSION {
        tx.execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at)\n\
             VALUES (?1, CAST(strftime('%s','now') AS INTEGER))",
            [version],
        )
        .map_err(StoreError::Sql)?;
    }
    tx.pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(StoreError::Sql)?;
    tx.commit().map_err(StoreError::Sql)
}

fn migrate_outbox_to_rejected_state(tx: &Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(
        "DROP INDEX IF EXISTS delegation_outbox_pending_idx;
         ALTER TABLE delegation_outbox RENAME TO delegation_outbox_v1;
         CREATE TABLE delegation_outbox (
             message_id TEXT PRIMARY KEY,
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
             objective_id TEXT NOT NULL REFERENCES objectives(id),
             issuer_scope TEXT NOT NULL,
             issued_at INTEGER NOT NULL,
             target TEXT NOT NULL,
             body_bytes BLOB NOT NULL,
             reply INTEGER NOT NULL CHECK(reply IN (0, 1)),
             timeout_secs INTEGER NOT NULL,
             deadline_contractuelle INTEGER NOT NULL,
             body_hash BLOB NOT NULL,
             message_bytes BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),
             attempted_at INTEGER,
             retry_until INTEGER NOT NULL,
             dedup_retained_until INTEGER NOT NULL,
             last_issue_json BLOB,
             issue_observed_at INTEGER,
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))
         );
         INSERT INTO delegation_outbox(
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,
             message_bytes, state, attempted_at, retry_until, dedup_retained_until,
             last_issue_json, issue_observed_at, terminal
         )
         SELECT message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
                body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,
                message_bytes,
                CASE
                    WHEN terminal = 1 AND state = 'outcome_unknown' THEN 'rejected'
                    ELSE state
                END,
                attempted_at, retry_until, dedup_retained_until,
                last_issue_json, issue_observed_at, terminal
         FROM delegation_outbox_v1;
         DROP TABLE delegation_outbox_v1;
         CREATE INDEX delegation_outbox_pending_idx
             ON delegation_outbox(terminal, state, retry_until, message_id);",
    )
    .map_err(StoreError::Sql)
}

fn load_or_create_issuer_scope(connection: &mut Connection) -> Result<String, StoreError> {
    let tx = connection.transaction().map_err(StoreError::Sql)?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT issuer_scope FROM maicie_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let candidate = existing.unwrap_or_else(|| format!("maicie-{}", Uuid::new_v4().simple()));
    tx.execute(
        "INSERT OR IGNORE INTO maicie_identity(singleton, issuer_scope) VALUES (1, ?1)",
        [&candidate],
    )
    .map_err(StoreError::Sql)?;
    let scope: String = tx
        .query_row(
            "SELECT issuer_scope FROM maicie_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    tx.commit().map_err(StoreError::Sql)?;
    Ok(scope)
}

fn insert_prepared(tx: &Transaction<'_>, prepared: &PreparedDelegation) -> Result<(), StoreError> {
    upsert_objective(tx, &prepared.objective)?;
    let delegation_json = serde_json::to_vec(&prepared.delegation).map_err(StoreError::Json)?;
    tx.execute(
        "INSERT INTO delegations(id, objective_id, state, payload_json) VALUES (?1, ?2, ?3, ?4)",
        params![
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            delegation_state_name(prepared.delegation.etat),
            delegation_json
        ],
    )
    .map_err(StoreError::Sql)?;
    tx.execute(
        "INSERT INTO delegation_outbox(\n\
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,\n\
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,\n\
             message_bytes, state, attempted_at, retry_until, dedup_retained_until, terminal\n\
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'prepared',NULL,?13,?14,0)",
        params![
            prepared.outbox.message_id.to_string(),
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            prepared.issuer_scope,
            prepared.issued_at,
            prepared.outbox.target,
            prepared.outbox.body_bytes,
            i64::from(prepared.outbox.reply),
            i64::try_from(prepared.outbox.timeout_secs)
                .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
            prepared.outbox.deadline_contractuelle,
            prepared.outbox.body_hash,
            prepared.message_bytes,
            prepared.outbox.retry_until,
            prepared.outbox.dedup_retained_until,
        ],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

fn upsert_objective(tx: &Transaction<'_>, objective: &ObjectifCoordonne) -> Result<(), StoreError> {
    let id = objective.id.to_string();
    let incoming_json = serde_json::to_vec(objective).map_err(StoreError::Json)?;
    let current: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM objectives WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;

    let Some((current_state, current_json)) = current else {
        tx.execute(
            "INSERT INTO objectives(id, state, payload_json) VALUES (?1, ?2, ?3)",
            params![id, objective_state_name(objective.etat), incoming_json],
        )
        .map_err(StoreError::Sql)?;
        return Ok(());
    };

    let mut current_objective: ObjectifCoordonne =
        serde_json::from_slice(&current_json).map_err(StoreError::Json)?;
    if parse_objective_state(&current_state)? != current_objective.etat {
        return Err(StoreError::Corrupt(
            "état objectif divergent de son payload",
        ));
    }
    if current_objective == *objective {
        return Ok(());
    }
    current_objective
        .transition(objective.etat, objective.mis_a_jour_at)
        .map_err(|_| StoreError::Conflict("transition objectif interdite"))?;
    if current_objective != *objective {
        return Err(StoreError::Conflict(
            "payload objectif incohérent avec la transition",
        ));
    }
    let changed = tx
        .execute(
            "UPDATE objectives SET state = ?1, payload_json = ?2\n\
             WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
            params![
                objective_state_name(objective.etat),
                incoming_json,
                id,
                current_state,
                current_json
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::Conflict(
            "objectif modifié concurremment pendant la transition",
        ))
    }
}

fn objective_state_name(state: EtatObjectif) -> &'static str {
    match state {
        EtatObjectif::Ouvert => "ouvert",
        EtatObjectif::EnCoordination => "en_coordination",
        EtatObjectif::AEvaluer => "a_evaluer",
        EtatObjectif::Synthetise => "synthetise",
        EtatObjectif::Clos => "clos",
    }
}

fn parse_objective_state(value: &str) -> Result<EtatObjectif, StoreError> {
    match value {
        "ouvert" => Ok(EtatObjectif::Ouvert),
        "en_coordination" => Ok(EtatObjectif::EnCoordination),
        "a_evaluer" => Ok(EtatObjectif::AEvaluer),
        "synthetise" => Ok(EtatObjectif::Synthetise),
        "clos" => Ok(EtatObjectif::Clos),
        _ => Err(StoreError::Corrupt("état objectif inconnu")),
    }
}

fn delegation_state_name(state: EtatDelegation) -> &'static str {
    match state {
        EtatDelegation::Creee => "creee",
        EtatDelegation::AEvaluer => "a_evaluer",
        EtatDelegation::Terminee => "terminee",
        EtatDelegation::Annulee => "annulee",
    }
}

fn parse_delegation_state(value: &str) -> Result<EtatDelegation, StoreError> {
    match value {
        "creee" => Ok(EtatDelegation::Creee),
        "a_evaluer" => Ok(EtatDelegation::AEvaluer),
        "terminee" => Ok(EtatDelegation::Terminee),
        "annulee" => Ok(EtatDelegation::Annulee),
        _ => Err(StoreError::Corrupt("état délégation inconnu")),
    }
}

fn parse_outbox_state(value: &str) -> Result<EtatOutboxDelegation, StoreError> {
    match value {
        "prepared" => Ok(EtatOutboxDelegation::Prepared),
        "outcome_unknown" => Ok(EtatOutboxDelegation::OutcomeUnknown),
        "accepted" => Ok(EtatOutboxDelegation::Accepted),
        "rejected" => Ok(EtatOutboxDelegation::Rejected),
        _ => Err(StoreError::Corrupt("état outbox inconnu")),
    }
}

fn outbox_state_name(state: EtatOutboxDelegation) -> &'static str {
    match state {
        EtatOutboxDelegation::Prepared => "prepared",
        EtatOutboxDelegation::OutcomeUnknown => "outcome_unknown",
        EtatOutboxDelegation::Accepted => "accepted",
        EtatOutboxDelegation::Rejected => "rejected",
    }
}

fn encode_issue(issue: &IdempotencyIssue) -> Value {
    match issue {
        IdempotencyIssue::Accepted { expires_at } => {
            json!({"kind":"accepted","expires_at":expires_at})
        }
        IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        } => json!({
            "kind":"rejected","category":category,"reason":reason,"expires_at":expires_at
        }),
        IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id,
        } => json!({
            "kind":"outcome_unknown","expires_at":expires_at,"delivery_id":delivery_id
        }),
        IdempotencyIssue::EnvelopeMismatch => json!({"kind":"envelope_mismatch"}),
        IdempotencyIssue::IdempotencyExpired => json!({"kind":"idempotency_expired"}),
        IdempotencyIssue::InvalidIssuedAt => json!({"kind":"invalid_issued_at"}),
    }
}

struct RawPending {
    objective_id: String,
    delegation_id: String,
    message_id: String,
    issuer_scope: String,
    issued_at: i64,
    target: String,
    body_bytes: Vec<u8>,
    reply: i64,
    timeout_secs: i64,
    deadline_contractuelle: i64,
    body_hash: Vec<u8>,
    message_bytes: Vec<u8>,
    state: String,
    attempted_at: Option<i64>,
    retry_until: i64,
    dedup_retained_until: i64,
}

fn raw_pending_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawPending> {
    Ok(RawPending {
        objective_id: row.get(0)?,
        delegation_id: row.get(1)?,
        message_id: row.get(2)?,
        issuer_scope: row.get(3)?,
        issued_at: row.get(4)?,
        target: row.get(5)?,
        body_bytes: row.get(6)?,
        reply: row.get(7)?,
        timeout_secs: row.get(8)?,
        deadline_contractuelle: row.get(9)?,
        body_hash: row.get(10)?,
        message_bytes: row.get(11)?,
        state: row.get(12)?,
        attempted_at: row.get(13)?,
        retry_until: row.get(14)?,
        dedup_retained_until: row.get(15)?,
    })
}

impl TryFrom<RawPending> for PendingDelegationOutbox {
    type Error = StoreError;

    fn try_from(raw: RawPending) -> Result<Self, Self::Error> {
        let pending = Self {
            objective_id: parse_uuid(&raw.objective_id)?,
            delegation_id: parse_uuid(&raw.delegation_id)?,
            message_id: parse_uuid(&raw.message_id)?,
            issuer_scope: raw.issuer_scope,
            issued_at: raw.issued_at,
            target: raw.target,
            body_bytes: raw.body_bytes,
            reply: match raw.reply {
                0 => false,
                1 => true,
                _ => return Err(StoreError::Corrupt("booléen reply invalide")),
            },
            timeout_secs: u64::try_from(raw.timeout_secs)
                .map_err(|_| StoreError::Corrupt("timeout_secs invalide"))?,
            deadline_contractuelle: raw.deadline_contractuelle,
            body_hash: raw.body_hash,
            message_bytes: raw.message_bytes,
            state: parse_outbox_state(&raw.state)?,
            attempted_at: raw.attempted_at,
            retry_until: raw.retry_until,
            dedup_retained_until: raw.dedup_retained_until,
        };
        pending.validate().map_err(StoreError::Outbox)?;
        Ok(pending)
    }
}

struct RawRecovery {
    pending: RawPending,
    objective_state: String,
    delegation_state: String,
    last_issue_json: Option<Vec<u8>>,
    issue_observed_at: Option<i64>,
}

impl TryFrom<RawRecovery> for RecoverySnapshot {
    type Error = StoreError;

    fn try_from(raw: RawRecovery) -> Result<Self, Self::Error> {
        Ok(Self {
            outbox: raw.pending.try_into()?,
            objective_state: parse_objective_state(&raw.objective_state)?,
            delegation_state: parse_delegation_state(&raw.delegation_state)?,
            last_issue: raw
                .last_issue_json
                .map(|bytes| serde_json::from_slice(&bytes).map_err(StoreError::Json))
                .transpose()?,
            issue_observed_at: raw.issue_observed_at,
        })
    }
}

fn parse_uuid(value: &str) -> Result<Uuid, StoreError> {
    Uuid::parse_str(value).map_err(|_| StoreError::Corrupt("UUID stocké invalide"))
}

#[derive(Debug)]
pub enum StoreError {
    Invalid(&'static str),
    Conflict(&'static str),
    NotFound(&'static str),
    Corrupt(&'static str),
    UnsupportedSchema { found: i64, supported: i64 },
    Io(std::io::Error),
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    Outbox(OutboxError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "store invalide : {reason}"),
            Self::Conflict(reason) => write!(formatter, "conflit store : {reason}"),
            Self::NotFound(reason) => write!(formatter, "store introuvable : {reason}"),
            Self::Corrupt(reason) => write!(formatter, "store corrompu : {reason}"),
            Self::UnsupportedSchema { found, supported } => write!(
                formatter,
                "schéma SQLite {found} non supporté (maximum {supported})"
            ),
            Self::Io(source) => write!(formatter, "I/O store impossible : {source}"),
            Self::Sql(source) => write!(formatter, "SQLite impossible : {source}"),
            Self::Json(source) => write!(formatter, "JSON store impossible : {source}"),
            Self::Outbox(source) => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Sql(source) => Some(source),
            Self::Json(source) => Some(source),
            Self::Outbox(source) => Some(source),
            Self::Invalid(_)
            | Self::Conflict(_)
            | Self::NotFound(_)
            | Self::Corrupt(_)
            | Self::UnsupportedSchema { .. } => None,
        }
    }
}
