//! Socle transactionnel d'idempotence du daemon.
//!
//! Ce module ne connaît ni le routage ni la remise au wrapper. Il est le seul
//! propriétaire de la clé, des octets canoniques, de l'échéance et du résultat
//! public d'une opération idempotente.

use bridget_transport::ResolvedAgentDefinition;
use bridget_transport::protocol::{DelegatedRuntimeEventKind, ProjectReference};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
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
    Accepted {
        expires_at: i64,
    },
    Rejected {
        category: String,
        reason: String,
    },
    /// Destinataire purgé : terminale, distincte d'un rejet métier.
    Orphaned {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupResult {
    Accepted {
        expires_at: i64,
    },
    Rejected {
        category: String,
        reason: String,
        expires_at: i64,
    },
    OutcomeUnknown {
        expires_at: i64,
    },
    Orphaned {
        expires_at: i64,
        reason: String,
    },
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

/// Lien causal ajouté au-dessus d'une remise idempotente inchangée.
///
/// Les trois identifiants restent émis par Bridget. Aucun identifiant fournisseur
/// ne peut se substituer à ce rattachement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryExecutionLink {
    pub delivery_id: String,
    pub submission_id: String,
    pub execution_id: String,
}

/// Projection de stockage : la colonne `message_bytes` reste nullable pour
/// les lignes héritées de v1, classées `indeterminate` par la migration v2.
struct StoredSendDelivery {
    delivery_id: String,
    recipient_instance_id: String,
    delivery_generation: u64,
    expires_at: i64,
    phase: String,
    message_bytes: Option<Vec<u8>>,
}

impl StoredSendDelivery {
    fn into_delivery(self) -> Option<SendDelivery> {
        self.message_bytes.map(|message_bytes| SendDelivery {
            delivery_id: self.delivery_id,
            recipient_instance_id: self.recipient_instance_id,
            delivery_generation: self.delivery_generation,
            expires_at: self.expires_at,
            message_bytes,
        })
    }
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

/// État métier d'un ordre de spawn consommateur du socle idempotent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnCommandState {
    Requested,
    Reserved,
    Starting,
    Connected,
    Failed,
    Cancelled,
}

impl SpawnCommandState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Reserved => "reserved",
            Self::Starting => "starting",
            Self::Connected => "connected",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn from_str(value: &str) -> Result<Self, IdempotencyError> {
        match value {
            "requested" => Ok(Self::Requested),
            "reserved" => Ok(Self::Reserved),
            "starting" => Ok(Self::Starting),
            "connected" => Ok(Self::Connected),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(IdempotencyError::CorruptRecord(
                "état de commande spawn inconnu",
            )),
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Connected | Self::Failed | Self::Cancelled)
    }
}

/// Issue métier durable d'un ordre de spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnCommandIssue {
    Connected {
        name: String,
        generation: u64,
        instance_id: String,
        definition: Option<Box<ResolvedAgentDefinition>>,
    },
    Failed {
        category: String,
        reason: String,
    },
    Cancelled {
        reason: String,
    },
}

/// Projection durable de la saga `spawn_commands`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnCommand {
    pub command_id: String,
    pub name: String,
    pub generation: u64,
    pub persistent: bool,
    pub state: SpawnCommandState,
    pub instance_id: Option<String>,
    pub deadline_at: i64,
    pub expires_at: i64,
    pub issue: Option<SpawnCommandIssue>,
    pub resolved_definition: Option<ResolvedAgentDefinition>,
}

/// Résultat atomique de la réservation socle + saga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnReservation {
    Requested(SpawnCommand),
    InFlight(SpawnCommand),
    Replayed(SpawnCommandIssue),
    EnvelopeMismatch,
    IdempotencyExpired,
}

/// Cycle de vie durable de la propriété parent-enfant, distinct de la
/// connexion du processus et de l'issue de sa saga de lancement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentLinkState {
    Reserved,
    Open,
    Transferred,
    Closed,
    Orphaned,
}

impl AgentLinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reserved => "reserved",
            Self::Open => "open",
            Self::Transferred => "transferred",
            Self::Closed => "closed",
            Self::Orphaned => "orphaned",
        }
    }

    fn from_str(value: &str) -> Result<Self, IdempotencyError> {
        match value {
            "reserved" => Ok(Self::Reserved),
            "open" => Ok(Self::Open),
            "transferred" => Ok(Self::Transferred),
            "closed" => Ok(Self::Closed),
            "orphaned" => Ok(Self::Orphaned),
            _ => Err(IdempotencyError::CorruptRecord(
                "état de lien agent inconnu",
            )),
        }
    }

    pub fn owns_child(self) -> bool {
        matches!(self, Self::Reserved | Self::Open | Self::Transferred)
    }
}

/// Lien durable de propriété d'un enfant. Les références de mandat restent
/// opaques : Bridget les conserve mais ne décide jamais de leur cycle métier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLinkRecord {
    pub link_id: String,
    pub parent_instance_id: String,
    pub child_instance_id: String,
    pub parent_execution_id: Option<String>,
    pub objective_id: Option<String>,
    pub delegation_id: Option<String>,
    pub project: Option<ProjectReference>,
    pub role: String,
    pub agent_path: String,
    pub state: AgentLinkState,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    pub revision: u64,
}

/// Fait durable et cursé de changement de propriété. Il est distinct de la
/// présence d'un processus afin de pouvoir réveiller un parent après reprise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLinkEvent {
    pub cursor: u64,
    pub event_id: String,
    pub link_id: String,
    pub parent_instance_id: String,
    pub child_instance_id: String,
    pub state: AgentLinkState,
    pub observed_at: i64,
    pub project: Option<ProjectReference>,
}

/// Entrée redacted d'un fait runtime observé chez un enfant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegatedRuntimeEventInput {
    pub child_instance_id: String,
    pub child_execution_id: String,
    pub kind: DelegatedRuntimeEventKind,
    pub code: String,
    pub reference: String,
    pub observed_at: i64,
}

/// Fait runtime délégué durable. Le parent provient exclusivement du lien.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegatedRuntimeEventRecord {
    pub cursor: u64,
    pub event_id: String,
    pub link_id: String,
    pub parent_instance_id: String,
    pub child_instance_id: String,
    pub child_execution_id: String,
    pub kind: DelegatedRuntimeEventKind,
    pub code: String,
    pub reference: String,
    pub observed_at: i64,
    pub acknowledged_at: Option<i64>,
    pub project: Option<ProjectReference>,
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
    InvalidSpawnCommand,
    InvalidDelegatedRuntimeEvent,
    InvalidAgentLink,
    AgentLinkOwned,
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
            Self::InvalidSpawnCommand => write!(formatter, "commande spawn invalide"),
            Self::InvalidAgentLink => write!(formatter, "lien agent invalide"),
            Self::InvalidDelegatedRuntimeEvent => {
                write!(formatter, "fait runtime délégué invalide")
            }
            Self::AgentLinkOwned => write!(formatter, "enfant déjà possédé par un lien ouvert"),
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

/// Remise devenue orpheline après purge de la présence destinataire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanedDeliveryNotice {
    pub delivery_id: String,
    pub message_id: String,
    pub sender: String,
    pub target: String,
    pub reason: String,
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

    /// Retourne l'identité durable du superviseur 009, créée une seule fois
    /// avant sa première réservation. Le préfixe réserve cet espace aux
    /// opérations internes et le distingue des portées déclarées par les
    /// clients publics.
    pub fn supervisor_scope(&mut self) -> Result<String, IdempotencyError> {
        let candidate = format!("supervisor_{}", uuid::Uuid::new_v4().simple());
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO supervisor_identity (singleton, issuer_scope)
             VALUES (1, ?1) ON CONFLICT(singleton) DO NOTHING",
            params![candidate],
        )?;
        let scope = tx.query_row(
            "SELECT issuer_scope FROM supervisor_identity WHERE singleton = 1",
            [],
            |row| row.get::<_, String>(0),
        )?;
        validate_issuer_scope(&scope)?;
        tx.commit()?;
        Ok(scope)
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
                phase TEXT NOT NULL CHECK (phase IN ('dispatching', 'acked', 'indeterminate', 'orphaned')),
                expires_at INTEGER NOT NULL,
                message_bytes BLOB,
                FOREIGN KEY (issuer_scope, operation_kind, idempotency_key)
                    REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                    ON DELETE CASCADE
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_send_deliveries_operation
                ON send_deliveries(issuer_scope, operation_kind, idempotency_key);
            -- Lookup ledger : jointure sur (operation_kind, idempotency_key)
            -- sans issuer_scope - n'emprunte PAS l'UNIQUE ci-dessus (préfixe
            -- issuer_scope). Index dédié pour SEARCH, pas SCAN.
            CREATE INDEX IF NOT EXISTS idx_send_deliveries_kind_key
                ON send_deliveries(operation_kind, idempotency_key);
            CREATE TABLE IF NOT EXISTS supervisor_identity (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                issuer_scope TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS spawn_commands (
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL CHECK (operation_kind = 'spawn'),
                command_id TEXT NOT NULL,
                name TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation > 0),
                persistent INTEGER NOT NULL CHECK (persistent IN (0, 1)),
                state TEXT NOT NULL CHECK (state IN (
                    'requested', 'reserved', 'starting',
                    'connected', 'failed', 'cancelled'
                )),
                instance_id TEXT,
                deadline_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                issue_kind TEXT CHECK (issue_kind IN ('connected', 'failed', 'cancelled')),
                issue_category TEXT,
                issue_reason TEXT,
                resolved_definition_json TEXT,
                PRIMARY KEY (issuer_scope, operation_kind, command_id),
                FOREIGN KEY (issuer_scope, operation_kind, command_id)
                    REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                    ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_spawn_commands_state
                ON spawn_commands(state);
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
            );
            -- Visibilité d'émission (fait distinct de l'accusé). Créée ici pour
            -- que le socle idempotent puisse graver le ledger sans dépendre du
            -- Store applicatif dans les tests unitaires isolés.
            CREATE TABLE IF NOT EXISTS ledger (
                id TEXT NOT NULL,
                ts INTEGER NOT NULL,
                sender TEXT NOT NULL,
                target TEXT NOT NULL,
                body TEXT NOT NULL,
                conversation_key TEXT NOT NULL,
                PRIMARY KEY (id, target)
            );
            CREATE INDEX IF NOT EXISTS idx_ledger_ts ON ledger(ts);
            CREATE INDEX IF NOT EXISTS idx_ledger_conv ON ledger(conversation_key, ts);",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agent_links (
                link_id TEXT PRIMARY KEY,
                parent_instance_id TEXT NOT NULL,
                child_instance_id TEXT NOT NULL UNIQUE,
                parent_execution_id TEXT,
                objective_id TEXT,
                delegation_id TEXT,
                role TEXT NOT NULL,
                agent_path TEXT NOT NULL,
                state TEXT NOT NULL CHECK (state IN ('reserved', 'open', 'transferred', 'closed', 'orphaned')),
                created_at INTEGER NOT NULL,
                closed_at INTEGER,
                revision INTEGER NOT NULL CHECK (revision >= 0)
            );
            CREATE INDEX IF NOT EXISTS idx_agent_links_parent_state
                ON agent_links(parent_instance_id, state);
            CREATE INDEX IF NOT EXISTS idx_agent_links_child_state
                ON agent_links(child_instance_id, state);
            CREATE TABLE IF NOT EXISTS agent_link_events (
                cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                link_id TEXT NOT NULL,
                parent_instance_id TEXT NOT NULL,
                child_instance_id TEXT NOT NULL,
                state TEXT NOT NULL CHECK (state IN ('reserved', 'open', 'transferred', 'closed', 'orphaned')),
                observed_at INTEGER NOT NULL,
                FOREIGN KEY (link_id) REFERENCES agent_links(link_id)
            );
            CREATE INDEX IF NOT EXISTS idx_agent_link_events_parent_cursor
                ON agent_link_events(parent_instance_id, cursor);
            CREATE TABLE IF NOT EXISTS delegated_runtime_events (
                cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                link_id TEXT NOT NULL,
                parent_instance_id TEXT NOT NULL,
                child_instance_id TEXT NOT NULL,
                child_execution_id TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('warning', 'failed')),
                code TEXT NOT NULL,
                reference TEXT NOT NULL,
                observed_at INTEGER NOT NULL,
                acknowledged_at INTEGER,
                FOREIGN KEY (link_id) REFERENCES agent_links(link_id)
            );
            CREATE INDEX IF NOT EXISTS idx_delegated_runtime_events_parent_pending_cursor
                ON delegated_runtime_events(parent_instance_id, acknowledged_at, cursor);",
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
                tx.execute(
                    "ALTER TABLE send_deliveries ADD COLUMN message_bytes BLOB",
                    [],
                )?;
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
        let definition_migration_applied = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 3)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !definition_migration_applied {
            let has_definition = tx
                .prepare("PRAGMA table_info(spawn_commands)")?
                .query_map([], |row| row.get::<_, String>(1))?
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "resolved_definition_json");
            if !has_definition {
                tx.execute(
                    "ALTER TABLE spawn_commands ADD COLUMN resolved_definition_json TEXT",
                    [],
                )?;
            }
            tx.execute(
                "INSERT INTO idempotency_schema_migrations(version) VALUES (3)",
                [],
            )?;
        }
        // v4 : phase `orphaned` — destinataire purgé, sort CONNU (≠ indeterminate,
        // ≠ outcome_unknown). SQLite ne sait pas élargir un CHECK : reconstruction.
        //
        // `foreign_keys=ON` à l'ouverture : l'INSERT SELECT revalide chaque
        // enfant. Une base touchée hors daemon (`.backup`, SQL CLI — FK OFF
        // par défaut) peut contenir des remises sans parent ; les laisser
        // ferait échouer la migration au démarrage et bloquerait la flotte.
        //
        // Décision (écrite — charge 4) :
        // - SUPPRIMER plutôt que conserver : sans parent `idempotency_records`,
        //   la remise n'est plus adressable (lookup, rejeu, signal émetteur).
        //   Ce n'est pas un orphelin métier (`orphaned`) : c'est un fantôme de
        //   schéma que le daemon ne peut pas créer (FK ON à l'ouverture) ; seule
        //   une intervention externe peut l'introduire. Origine inconnue ⇒
        //   données hors contrat, pas un sort à préserver.
        // - MAINTENANT (à la migration) plutôt qu'à l'usage : un seul fantôme
        //   ferait échouer l'INSERT SELECT et bloquerait open() / la flotte.
        // - Ne PAS désactiver `foreign_keys` : on reste sous la règle du daemon.
        // - Ne PAS écarter en silence : `warn!` avec compte + delivery_id —
        //   même règle que ce lot : un sort connu vaut mieux qu'un sort muet.
        //
        // Nature (mesurée, greffe) : PRÉCAUTION, pas incident évité de justesse.
        // Copie `.backup` de `~/.cache/bridget/bridget.db` : 2477 send_deliveries,
        // `PRAGMA foreign_key_check` → zéro ligne. Mécanisme prouvé ailleurs
        // (contrôle négatif : orphelin injecté → Err FK sans ce geste ; Ok avec).
        // Occurrence nulle aujourd'hui ≠ mécanisme inutile — les deux coexistent.
        let orphan_migration_applied = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 4)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !orphan_migration_applied {
            let schema_orphans: Vec<String> = {
                let mut stmt = tx.prepare(
                    "SELECT delivery_id FROM send_deliveries
                     WHERE NOT EXISTS (
                         SELECT 1 FROM idempotency_records r
                         WHERE r.issuer_scope = send_deliveries.issuer_scope
                           AND r.operation_kind = send_deliveries.operation_kind
                           AND r.idempotency_key = send_deliveries.idempotency_key
                     )",
                )?;
                stmt.query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<_>, _>>()?
            };
            if !schema_orphans.is_empty() {
                log::warn!(
                    "migration v4: écarté {} remise(s) send_deliveries sans parent \
                     idempotency_records (origine hors daemon — CLI/backup, FK OFF ; \
                     non rejouables ni signalables). Suppression plutôt que \
                     conservation ou échec FK : démarrer en disant ce qui est perdu \
                     plutôt que bloquer la flotte. delivery_id={:?}",
                    schema_orphans.len(),
                    schema_orphans
                );
                tx.execute(
                    "DELETE FROM send_deliveries
                     WHERE NOT EXISTS (
                         SELECT 1 FROM idempotency_records r
                         WHERE r.issuer_scope = send_deliveries.issuer_scope
                           AND r.operation_kind = send_deliveries.operation_kind
                           AND r.idempotency_key = send_deliveries.idempotency_key
                     )",
                    [],
                )?;
            }
            tx.execute_batch(
                "CREATE TABLE send_deliveries_v4 (
                    delivery_id TEXT PRIMARY KEY,
                    issuer_scope TEXT NOT NULL,
                    operation_kind TEXT NOT NULL CHECK (operation_kind = 'send'),
                    idempotency_key TEXT NOT NULL,
                    recipient_instance_id TEXT NOT NULL,
                    delivery_generation INTEGER NOT NULL CHECK (delivery_generation > 0),
                    phase TEXT NOT NULL CHECK (phase IN ('dispatching', 'acked', 'indeterminate', 'orphaned')),
                    expires_at INTEGER NOT NULL,
                    message_bytes BLOB,
                    FOREIGN KEY (issuer_scope, operation_kind, idempotency_key)
                        REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                        ON DELETE CASCADE
                );
                INSERT INTO send_deliveries_v4 (
                    delivery_id, issuer_scope, operation_kind, idempotency_key,
                    recipient_instance_id, delivery_generation, phase, expires_at, message_bytes
                ) SELECT
                    delivery_id, issuer_scope, operation_kind, idempotency_key,
                    recipient_instance_id, delivery_generation, phase, expires_at, message_bytes
                FROM send_deliveries;
                DROP TABLE send_deliveries;
                ALTER TABLE send_deliveries_v4 RENAME TO send_deliveries;
                CREATE UNIQUE INDEX IF NOT EXISTS idx_send_deliveries_operation
                    ON send_deliveries(issuer_scope, operation_kind, idempotency_key);
                CREATE INDEX IF NOT EXISTS idx_send_deliveries_kind_key
                    ON send_deliveries(operation_kind, idempotency_key);
                INSERT INTO idempotency_schema_migrations(version) VALUES (4);",
            )?;
        }
        let project_reference_migration_applied = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 6)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !project_reference_migration_applied {
            for statement in [
                "ALTER TABLE agent_links ADD COLUMN project_id TEXT",
                "ALTER TABLE agent_links ADD COLUMN binding_generation INTEGER",
                "ALTER TABLE agent_link_events ADD COLUMN project_id TEXT",
                "ALTER TABLE agent_link_events ADD COLUMN binding_generation INTEGER",
                "ALTER TABLE delegated_runtime_events ADD COLUMN project_id TEXT",
                "ALTER TABLE delegated_runtime_events ADD COLUMN binding_generation INTEGER",
            ] {
                tx.execute(statement, [])?;
            }
            tx.execute(
                "INSERT INTO idempotency_schema_migrations(version) VALUES (6)",
                [],
            )?;
        }
        // Une seule copie du DDL : bases neuves, montée v4, et têtes antérieures
        // déjà en v4 sans la table. CREATE IF NOT EXISTS couvre les trois ;
        // le dupliquer ailleurs (CHECK / schéma) rendait la migration invisible
        // aux témoins — même motif que la charge 1 sur send_deliveries.
        //
        // `notified_at` : émission socket Ok, pas réception wrapper (limite
        // déclarée — voir flush_pending_orphan_emitter_notices).
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS orphan_emitter_notices (
                delivery_id TEXT PRIMARY KEY,
                sender TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                notified_at INTEGER
            );",
        )?;
        tx.execute("CREATE TABLE IF NOT EXISTS send_delivery_execution_links (delivery_id TEXT PRIMARY KEY, submission_id TEXT NOT NULL, execution_id TEXT NOT NULL, FOREIGN KEY (delivery_id) REFERENCES send_deliveries(delivery_id) ON DELETE CASCADE)", [])?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_send_delivery_execution_links_submission ON send_delivery_execution_links(submission_id, execution_id)", [])?;
        tx.execute(
            "INSERT OR IGNORE INTO idempotency_schema_migrations(version) VALUES (5)",
            [],
        )?;
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

    /// Réserve atomiquement une clé `spawn` et sa ligne de saga. Cette API est
    /// l'unique point de décision du consommateur 009 : le code `fleet` ne
    /// refait ni lookup, ni comparaison canonique, ni logique de rejeu.
    #[allow(clippy::too_many_arguments)]
    pub fn reserve_spawn(
        &mut self,
        key: &IdempotencyKey,
        canonical_bytes: &[u8],
        issued_at: i64,
        horizon_secs: i64,
        now: i64,
        issued_at_tolerance_secs: i64,
        name: &str,
        generation: u64,
        persistent: bool,
        deadline_at: i64,
        instance_id: &str,
    ) -> Result<SpawnReservation, IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn
            || name.is_empty()
            || generation == 0
            || deadline_at <= issued_at
            || instance_id.is_empty()
        {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        validate_canonical_bytes(canonical_bytes)?;
        if issued_at > now.saturating_add(issued_at_tolerance_secs.max(0)) {
            return Err(IdempotencyError::InvalidIssuedAt);
        }

        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = load_record_from(&tx, key)?;
        if let Some(record) = existing {
            let result = if record.expires_at <= now {
                SpawnReservation::IdempotencyExpired
            } else if record.canonical_bytes != canonical_bytes {
                SpawnReservation::EnvelopeMismatch
            } else {
                let command = load_spawn_command_from(&tx, key)?
                    .ok_or(IdempotencyError::CorruptRecord("saga spawn absente"))?;
                if record.state == RecordState::Terminal {
                    let issue = command
                        .issue
                        .clone()
                        .ok_or(IdempotencyError::CorruptRecord(
                            "issue spawn terminale absente",
                        ))?;
                    record.validate_spawn_issue(&issue)?;
                    SpawnReservation::Replayed(issue)
                } else if command.state.is_terminal() {
                    return Err(IdempotencyError::CorruptRecord(
                        "saga terminale avec socle non terminal",
                    ));
                } else {
                    SpawnReservation::InFlight(command)
                }
            };
            tx.commit()?;
            return Ok(result);
        }

        if horizon_secs <= 0 {
            return Err(IdempotencyError::InvalidHorizon);
        }
        let expires_at = issued_at
            .checked_add(horizon_secs)
            .ok_or(IdempotencyError::InvalidHorizon)?;
        if expires_at <= now {
            tx.commit()?;
            return Ok(SpawnReservation::IdempotencyExpired);
        }

        tx.execute(
            "INSERT INTO idempotency_records (
                issuer_scope, operation_kind, idempotency_key, canonical_bytes,
                state, issued_at, expires_at
             ) VALUES (?1, 'spawn', ?2, ?3, 'prepared', ?4, ?5)",
            params![
                key.issuer_scope,
                key.idempotency_key,
                canonical_bytes,
                issued_at,
                expires_at,
            ],
        )?;
        tx.execute(
            "INSERT INTO spawn_commands (
                issuer_scope, operation_kind, command_id, name, generation,
                persistent, state, instance_id, deadline_at, expires_at
             ) VALUES (?1, 'spawn', ?2, ?3, ?4, ?5, 'requested', ?6, ?7, ?8)",
            params![
                key.issuer_scope,
                key.idempotency_key,
                name,
                generation,
                persistent,
                instance_id,
                deadline_at,
                expires_at,
            ],
        )?;
        tx.commit()?;
        Ok(SpawnReservation::Requested(SpawnCommand {
            command_id: key.idempotency_key.clone(),
            name: name.to_string(),
            generation,
            persistent,
            state: SpawnCommandState::Requested,
            instance_id: Some(instance_id.to_string()),
            deadline_at,
            expires_at,
            issue: None,
            resolved_definition: None,
        }))
    }

    /// Avance la saga en vol. Le passage `Requested→Reserved` rend le socle
    /// `Dispatching` dans la même transaction que la réservation métier.
    pub fn advance_spawn(
        &mut self,
        key: &IdempotencyKey,
        generation: u64,
        from: SpawnCommandState,
        to: SpawnCommandState,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn
            || !matches!(
                (from, to),
                (SpawnCommandState::Requested, SpawnCommandState::Reserved)
                    | (SpawnCommandState::Reserved, SpawnCommandState::Starting)
            )
        {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if from == SpawnCommandState::Requested {
            let core = tx.execute(
                "UPDATE idempotency_records SET state = 'dispatching'
                 WHERE issuer_scope = ?1 AND operation_kind = 'spawn'
                   AND idempotency_key = ?2 AND state = 'prepared'",
                params![key.issuer_scope, key.idempotency_key],
            )?;
            if core != 1 {
                return Err(IdempotencyError::DispatchUnavailable);
            }
        }
        let saga = tx.execute(
            "UPDATE spawn_commands SET state = ?1
             WHERE issuer_scope = ?2 AND operation_kind = 'spawn'
               AND command_id = ?3 AND generation = ?4 AND state = ?5",
            params![
                to.as_str(),
                key.issuer_scope,
                key.idempotency_key,
                generation,
                from.as_str(),
            ],
        )?;
        if saga != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    pub fn advance_spawn_with_definition(
        &mut self,
        key: &IdempotencyKey,
        generation: u64,
        from: SpawnCommandState,
        to: SpawnCommandState,
        definition: &ResolvedAgentDefinition,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn
            || (from, to) != (SpawnCommandState::Reserved, SpawnCommandState::Starting)
        {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let definition_json =
            serde_json::to_string(definition).map_err(|_| IdempotencyError::InvalidSpawnCommand)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let updated = tx.execute(
            "UPDATE spawn_commands
             SET state = 'starting', resolved_definition_json = ?1
             WHERE issuer_scope = ?2 AND operation_kind = 'spawn'
               AND command_id = ?3 AND generation = ?4 AND state = 'reserved'",
            params![
                definition_json,
                key.issuer_scope,
                key.idempotency_key,
                generation
            ],
        )?;
        if updated != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    /// Finalise ensemble la saga et le résultat public du socle.
    pub fn finish_spawn(
        &mut self,
        key: &IdempotencyKey,
        generation: u64,
        issue: &SpawnCommandIssue,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn || generation == 0 {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let (state, issue_kind, instance_id, category, reason, public_kind) = match issue {
            SpawnCommandIssue::Connected {
                name: _,
                generation: issue_generation,
                instance_id,
                definition: _,
            } if *issue_generation == generation && !instance_id.is_empty() => (
                SpawnCommandState::Connected,
                "connected",
                Some(instance_id.as_str()),
                None,
                None,
                "accepted",
            ),
            SpawnCommandIssue::Failed { category, reason }
                if !category.is_empty() && !reason.is_empty() =>
            {
                (
                    SpawnCommandState::Failed,
                    "failed",
                    None,
                    Some(category.as_str()),
                    Some(reason.as_str()),
                    "rejected",
                )
            }
            SpawnCommandIssue::Cancelled { reason } if !reason.is_empty() => (
                SpawnCommandState::Cancelled,
                "cancelled",
                None,
                Some("cancelled"),
                Some(reason.as_str()),
                "rejected",
            ),
            _ => return Err(IdempotencyError::InvalidSpawnCommand),
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let saga = tx.execute(
            "UPDATE spawn_commands
             SET state = ?1, instance_id = COALESCE(?2, instance_id), issue_kind = ?3,
                 issue_category = ?4, issue_reason = ?5
             WHERE issuer_scope = ?6 AND operation_kind = 'spawn'
               AND command_id = ?7 AND generation = ?8
               AND state IN ('requested', 'reserved', 'starting')
               AND (?3 != 'connected' OR instance_id = ?2)",
            params![
                state.as_str(),
                instance_id,
                issue_kind,
                category,
                reason,
                key.issuer_scope,
                key.idempotency_key,
                generation,
            ],
        )?;
        if saga != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let core = tx.execute(
            "UPDATE idempotency_records
             SET state = 'terminal', public_result_kind = ?1,
                 public_result_category = ?2, public_result_reason = ?3
             WHERE issuer_scope = ?4 AND operation_kind = 'spawn'
               AND idempotency_key = ?5 AND state IN ('prepared', 'dispatching')",
            params![
                public_kind,
                category,
                reason,
                key.issuer_scope,
                key.idempotency_key,
            ],
        )?;
        if core != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    /// Charge les sagas d'un superviseur pour la réconciliation au démarrage.
    pub fn spawn_commands(
        &self,
        issuer_scope: &str,
    ) -> Result<Vec<SpawnCommand>, IdempotencyError> {
        validate_issuer_scope(issuer_scope)?;
        let mut statement = self.conn.prepare(
            "SELECT command_id, name, generation, persistent, state,
                    instance_id, deadline_at, expires_at,
                    issue_kind, issue_category, issue_reason, resolved_definition_json
             FROM spawn_commands WHERE issuer_scope = ?1 AND operation_kind = 'spawn'
             ORDER BY generation, command_id",
        )?;
        let rows = statement.query_map(params![issuer_scope], spawn_command_from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Crée le lien de propriété avant que la génération ne puisse démarrer.
    /// Le même enfant ne peut pas recevoir un second propriétaire ouvert.
    pub fn create_agent_link(&mut self, link: &AgentLinkRecord) -> Result<(), IdempotencyError> {
        if link.link_id.trim().is_empty()
            || link.parent_instance_id.trim().is_empty()
            || link.child_instance_id.trim().is_empty()
            || link.role.trim().is_empty()
            || link.agent_path.trim().is_empty()
            || link.created_at <= 0
            || link.revision != 0
            || link.state != AgentLinkState::Reserved
            || link.closed_at.is_some()
        {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let inserted = tx.execute(
            "INSERT INTO agent_links (
                link_id, parent_instance_id, child_instance_id, parent_execution_id,
                objective_id, delegation_id, role, agent_path, state, created_at,
                closed_at, revision, project_id, binding_generation
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'reserved', ?9, NULL, 0, ?10, ?11)
             ON CONFLICT(child_instance_id) DO NOTHING",
            params![
                link.link_id,
                link.parent_instance_id,
                link.child_instance_id,
                link.parent_execution_id,
                link.objective_id,
                link.delegation_id,
                link.role,
                link.agent_path,
                link.created_at,
                link.project.as_ref().map(|project| &project.project_id),
                link.project
                    .as_ref()
                    .map(|project| i64::try_from(project.binding_generation))
                    .transpose()
                    .map_err(|_| IdempotencyError::InvalidAgentLink)?,
            ],
        )?;
        if inserted == 0 {
            let existing = tx
                .query_row(
                    "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                            objective_id, delegation_id, role, agent_path, state, created_at,
                            closed_at, revision, project_id, binding_generation
                     FROM agent_links WHERE child_instance_id = ?1",
                    [&link.child_instance_id],
                    agent_link_from_row,
                )
                .optional()?;
            tx.commit()?;
            return match existing {
                Some(existing) if existing == *link => Ok(()),
                Some(_) => Err(IdempotencyError::AgentLinkOwned),
                None => Err(IdempotencyError::CorruptRecord(
                    "lien agent absent après conflit",
                )),
            };
        }
        record_agent_link_event(&tx, link, link.created_at)?;
        tx.commit()?;
        Ok(())
    }

    pub fn agent_link_for_child(
        &self,
        child_instance_id: &str,
    ) -> Result<Option<AgentLinkRecord>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE child_instance_id = ?1",
                [child_instance_id],
                agent_link_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn agent_link_by_id(
        &self,
        link_id: &str,
    ) -> Result<Option<AgentLinkRecord>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE link_id = ?1",
                [link_id],
                agent_link_from_row,
            )
            .optional()
            .map_err(Into::into)
    }
    /// Enfants qui ont encore un propriétaire actif. Les liens fermés ou
    /// orphelins restent consultables par enfant mais ne consomment plus quota.
    pub fn open_agent_links_for_parent(
        &self,
        parent_instance_id: &str,
    ) -> Result<Vec<AgentLinkRecord>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                    objective_id, delegation_id, role, agent_path, state, created_at,
                    closed_at, revision, project_id, binding_generation
             FROM agent_links
             WHERE parent_instance_id = ?1
               AND state IN ('reserved', 'open', 'transferred')
             ORDER BY created_at, link_id",
        )?;
        statement
            .query_map([parent_instance_id], agent_link_from_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Compte les descendants encore rattachés à un propriétaire. Le décompte
    /// direct exploite l'index parent-état ; le total est réservé à la
    /// projection UI et suit le chemin stable, sans modifier les liens.
    pub fn open_agent_link_descendant_counts(
        &self,
        instance_id: &str,
    ) -> Result<(u64, u64), IdempotencyError> {
        let direct: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agent_links
             WHERE parent_instance_id = ?1
               AND state IN ('reserved', 'open', 'transferred')",
            [instance_id],
            |row| row.get(0),
        )?;
        let path = self
            .agent_link_for_child(instance_id)?
            .map(|link| link.agent_path)
            .unwrap_or_else(|| instance_id.to_string());
        let escaped = path
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let descendants: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agent_links
             WHERE agent_path LIKE ?1 ESCAPE '\\'
               AND state IN ('reserved', 'open', 'transferred')",
            [format!("{escaped}/%")],
            |row| row.get(0),
        )?;
        Ok((
            u64::try_from(direct).unwrap_or(0),
            u64::try_from(descendants).unwrap_or(0),
        ))
    }
    /// Relit les changements durables après un curseur pour reprendre une
    /// attente interrompue sans redemander l'état courant à un fournisseur.
    pub fn agent_link_events_after(
        &self,
        parent_instance_id: &str,
        after_cursor: Option<u64>,
    ) -> Result<Vec<AgentLinkEvent>, IdempotencyError> {
        let after_cursor = i64::try_from(after_cursor.unwrap_or_default()).unwrap_or(i64::MAX);
        let mut statement = self.conn.prepare(
            "SELECT cursor, event_id, link_id, parent_instance_id, child_instance_id, state, observed_at,
                    project_id, binding_generation
             FROM agent_link_events
             WHERE parent_instance_id = ?1 AND cursor > ?2
             ORDER BY cursor
             LIMIT 128",
        )?;
        statement
            .query_map(
                params![parent_instance_id, after_cursor],
                agent_link_event_from_row,
            )?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
    /// Persiste un fait runtime corrélé à un enfant déjà lié. L'identifiant est
    /// déterministe sur les seuls champs redacted afin que le rejeu ne duplique
    /// jamais le même diagnostic ou le même terminal.
    pub fn record_delegated_runtime_event(
        &mut self,
        input: DelegatedRuntimeEventInput,
    ) -> Result<DelegatedRuntimeEventRecord, IdempotencyError> {
        validate_delegated_runtime_event_input(&input)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let link = tx
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE child_instance_id = ?1",
                [&input.child_instance_id],
                agent_link_from_row,
            )
            .optional()?
            .ok_or(IdempotencyError::InvalidDelegatedRuntimeEvent)?;
        let event_id = delegated_runtime_event_id(&link, &input);
        tx.execute(
            "INSERT INTO delegated_runtime_events (
                event_id, link_id, parent_instance_id, child_instance_id,
                child_execution_id, kind, code, reference, observed_at, acknowledged_at,
                project_id, binding_generation
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, ?11)
             ON CONFLICT(event_id) DO NOTHING",
            params![
                &event_id,
                &link.link_id,
                &link.parent_instance_id,
                &link.child_instance_id,
                &input.child_execution_id,
                input.kind.as_str(),
                &input.code,
                &input.reference,
                input.observed_at,
                link.project.as_ref().map(|project| &project.project_id),
                link.project
                    .as_ref()
                    .map(|project| i64::try_from(project.binding_generation))
                    .transpose()
                    .map_err(|_| IdempotencyError::InvalidDelegatedRuntimeEvent)?,
            ],
        )?;
        let event = tx.query_row(
            "SELECT cursor, event_id, link_id, parent_instance_id, child_instance_id,
                    child_execution_id, kind, code, reference, observed_at, acknowledged_at,
                    project_id, binding_generation
             FROM delegated_runtime_events WHERE event_id = ?1",
            [&event_id],
            delegated_runtime_event_from_row,
        )?;
        tx.commit()?;
        Ok(event)
    }

    /// Relit dans l'ordre les faits qui n'ont pas encore franchi une frontière
    /// observable du transport du parent.
    pub fn delegated_runtime_events_for_parent(
        &self,
        parent_instance_id: &str,
    ) -> Result<Vec<DelegatedRuntimeEventRecord>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT cursor, event_id, link_id, parent_instance_id, child_instance_id,
                    child_execution_id, kind, code, reference, observed_at, acknowledged_at,
                    project_id, binding_generation
             FROM delegated_runtime_events
             WHERE parent_instance_id = ?1 AND acknowledged_at IS NULL
             ORDER BY cursor
             LIMIT 128",
        )?;
        statement
            .query_map([parent_instance_id], delegated_runtime_event_from_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Accuse un fait seulement si le wrapper qui parle est son parent attesté.
    /// Un accusé répétitif du même parent est idempotent; toute autre identité
    /// est un refus sans écriture.
    pub fn acknowledge_delegated_runtime_event(
        &mut self,
        event_id: &str,
        parent_instance_id: &str,
    ) -> Result<bool, IdempotencyError> {
        if event_id.trim().is_empty() || parent_instance_id.trim().is_empty() {
            return Err(IdempotencyError::InvalidDelegatedRuntimeEvent);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (recorded_parent, acknowledged_at): (String, Option<i64>) = tx
            .query_row(
                "SELECT parent_instance_id, acknowledged_at
                 FROM delegated_runtime_events WHERE event_id = ?1",
                [event_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(IdempotencyError::InvalidDelegatedRuntimeEvent)?;
        if recorded_parent != parent_instance_id {
            return Err(IdempotencyError::InvalidDelegatedRuntimeEvent);
        }
        if acknowledged_at.is_some() {
            tx.commit()?;
            return Ok(false);
        }
        let acknowledged_at = unix_now_secs();
        let changed = tx.execute(
            "UPDATE delegated_runtime_events
             SET acknowledged_at = ?1
             WHERE event_id = ?2 AND parent_instance_id = ?3 AND acknowledged_at IS NULL",
            params![acknowledged_at, event_id, parent_instance_id],
        )?;
        if changed != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(true)
    }

    pub fn transition_agent_link(
        &mut self,
        link_id: &str,
        allowed: &[AgentLinkState],
        next: AgentLinkState,
        changed_at: i64,
    ) -> Result<AgentLinkRecord, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE link_id = ?1",
                [link_id],
                agent_link_from_row,
            )
            .optional()?
            .ok_or(IdempotencyError::CorruptRecord("lien agent absent"))?;
        if current.state == next {
            tx.commit()?;
            return Ok(current);
        }
        if !allowed.contains(&current.state) {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let changed_at = changed_at.max(current.created_at);
        let closed_at = if next == AgentLinkState::Closed {
            Some(changed_at)
        } else {
            None
        };
        let changed = tx.execute(
            "UPDATE agent_links
             SET state = ?1, closed_at = ?2, revision = revision + 1
             WHERE link_id = ?3 AND revision = ?4",
            params![next.as_str(), closed_at, link_id, current.revision],
        )?;
        if changed != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let updated = tx.query_row(
            "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                    objective_id, delegation_id, role, agent_path, state, created_at,
                    closed_at, revision, project_id, binding_generation
             FROM agent_links WHERE link_id = ?1",
            [link_id],
            agent_link_from_row,
        )?;
        record_agent_link_event(&tx, &updated, changed_at)?;
        tx.commit()?;
        Ok(updated)
    }

    pub fn orphan_agent_links_for_parent(
        &mut self,
        parent_instance_id: &str,
        observed_at: i64,
    ) -> Result<usize, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let active = tx
            .prepare(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links
                 WHERE parent_instance_id = ?1
                   AND state IN ('reserved', 'open', 'transferred')",
            )?
            .query_map([parent_instance_id], agent_link_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        let changed = tx.execute(
            "UPDATE agent_links
             SET state = 'orphaned', revision = revision + 1
             WHERE parent_instance_id = ?1
               AND state IN ('reserved', 'open', 'transferred')",
            [parent_instance_id],
        )?;
        for link in active {
            let updated = AgentLinkRecord {
                state: AgentLinkState::Orphaned,
                revision: link.revision + 1,
                closed_at: None,
                ..link
            };
            record_agent_link_event(&tx, &updated, observed_at)?;
        }
        tx.commit()?;
        Ok(changed)
    }

    pub fn transfer_agent_link(
        &mut self,
        link_id: &str,
        parent_instance_id: &str,
        agent_path: &str,
        observed_at: i64,
    ) -> Result<AgentLinkRecord, IdempotencyError> {
        if parent_instance_id.trim().is_empty() || agent_path.trim().is_empty() {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE link_id = ?1",
                [link_id],
                agent_link_from_row,
            )
            .optional()?
            .ok_or(IdempotencyError::CorruptRecord("lien agent absent"))?;
        if !matches!(
            current.state,
            AgentLinkState::Reserved
                | AgentLinkState::Open
                | AgentLinkState::Transferred
                | AgentLinkState::Orphaned
        ) {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let changed = tx.execute(
            "UPDATE agent_links
             SET parent_instance_id = ?1, agent_path = ?2, state = 'transferred',
                 closed_at = NULL, revision = revision + 1
             WHERE link_id = ?3 AND revision = ?4",
            params![parent_instance_id, agent_path, link_id, current.revision],
        )?;
        if changed != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let updated = tx.query_row(
            "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                    objective_id, delegation_id, role, agent_path, state, created_at,
                    closed_at, revision, project_id, binding_generation
             FROM agent_links WHERE link_id = ?1",
            [link_id],
            agent_link_from_row,
        )?;
        record_agent_link_event(&tx, &updated, observed_at)?;
        tx.commit()?;
        Ok(updated)
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

    /// Fige la remise d'un envoi. Le changement d'état du socle, l'entrée
    /// `send_deliveries` et la **visibilité ledger** (fait d'émission) partagent
    /// une transaction SQLite. L'accusé (`acked`) reste un fait distinct :
    /// `outcome_unknown` reste nominal tant que `DeliverAcked` n'est pas reçu.
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
        // Jamais de .ok() silencieux : une remise sans enveloppe lisible
        // recréerait la classe « invisible au ledger » que la gravure à
        // begin_send_delivery existe pour supprimer.
        let emitted_message =
            serde_json::from_slice::<bridget_core::BridgetMessage>(&delivery.message_bytes)
                .map_err(|_| IdempotencyError::CorruptRecord("enveloppe de remise illisible"))?;
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
        let conversation_key = format!("{}|{}", emitted_message.from, emitted_message.to);
        crate::store::record_message_in_transaction(&tx, &emitted_message, &conversation_key)?;
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

    /// Attache une remise déjà gravée à son contexte causal Bridget.
    /// La première écriture est définitive : un rejeu ne peut pas rerouter la remise.
    pub fn link_send_delivery(&self, link: &DeliveryExecutionLink) -> Result<(), IdempotencyError> {
        if link.delivery_id.is_empty()
            || link.submission_id.is_empty()
            || link.execution_id.is_empty()
        {
            return Err(IdempotencyError::InvalidDelivery);
        }

        self.conn.execute(
            "INSERT INTO send_delivery_execution_links (delivery_id, submission_id, execution_id) VALUES (?1, ?2, ?3) ON CONFLICT(delivery_id) DO NOTHING",
            params![link.delivery_id, link.submission_id, link.execution_id],
        )?;

        match self.delivery_execution_link(&link.delivery_id)? {
            Some(existing) if existing == *link => Ok(()),
            Some(_) => Err(IdempotencyError::InvalidDelivery),
            None => Err(IdempotencyError::CorruptRecord(
                "lien causal de remise absent",
            )),
        }
    }

    /// Lit le rattachement causal sans modifier la remise ni son état de rejeu.
    pub fn delivery_execution_link(
        &self,
        delivery_id: &str,
    ) -> Result<Option<DeliveryExecutionLink>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id, submission_id, execution_id FROM send_delivery_execution_links WHERE delivery_id = ?1",
                [delivery_id],
                |row| {
                    Ok(DeliveryExecutionLink {
                        delivery_id: row.get(0)?,
                        submission_id: row.get(1)?,
                        execution_id: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Réassigne les remises encore en vol d'une instance morte vers l'instance
    /// vivante du même équipier. Ne touche ni `acked` ni `indeterminate`.
    pub fn reassign_dispatching_deliveries(
        &mut self,
        from_instance_id: &str,
        to_instance_id: &str,
        now: i64,
    ) -> Result<usize, IdempotencyError> {
        if from_instance_id.is_empty()
            || to_instance_id.is_empty()
            || from_instance_id == to_instance_id
        {
            return Ok(0);
        }
        let updated = self.conn.execute(
            "UPDATE send_deliveries
             SET recipient_instance_id = ?1
             WHERE recipient_instance_id = ?2
               AND phase = 'dispatching'
               AND expires_at > ?3",
            params![to_instance_id, from_instance_id, now],
        )?;
        Ok(updated)
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

    /// Remise ENCORE EN VOL pour cette clé, et elle seule.
    ///
    /// Le filtre de phase n'est pas cosmétique : une remise passée en
    /// `indeterminate` — par échec de reprise, par `DeliveryIndeterminate`, ou
    /// par la migration v2 qui écarte les enveloppes absentes — est un état
    /// ABSORBANT. Plus rien ne l'accusera jamais, et elle occupe pourtant sa
    /// clé jusqu'à l'expiration de l'horizon (7 jours). Sans ce filtre, elle
    /// remontait comme une remise en vol : l'appelant lisait « le dépôt a
    /// réussi, rejouez pour lire le sort », rc=0, sur un message qui ne
    /// partirait plus jamais. Le mensonge optimiste est pire que celui qu'on
    /// corrige — d'où `dispatching` seul.
    ///
    /// Complexité : O(log n) via l'index unique de la clé idempotente.
    /// `orphaned` est aussi absorbant : le destinataire a été purgé, le sort
    /// est connu. Ne pas le confondre avec une remise en vol.
    pub fn send_delivery(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        let Some(delivery) = self.stored_send_delivery(key)? else {
            return Ok(None);
        };
        if delivery.phase != "dispatching" {
            return Ok(None);
        }
        Ok(delivery.into_delivery())
    }

    fn stored_send_delivery(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<StoredSendDelivery>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at,
                        phase, message_bytes
                 FROM send_deliveries
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2",
                params![key.issuer_scope, key.idempotency_key],
                |row| {
                    Ok(StoredSendDelivery {
                        delivery_id: row.get(0)?,
                        recipient_instance_id: row.get(1)?,
                        delivery_generation: row.get(2)?,
                        expires_at: row.get(3)?,
                        phase: row.get(4)?,
                        message_bytes: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Mutant livré : retire uniquement la garde de phase, sans modifier la
    /// projection nullable. Hors chemin de production par construction.
    #[cfg(test)]
    fn send_delivery_mutant_sans_filtre_de_phase(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        Ok(self
            .stored_send_delivery(key)?
            .and_then(StoredSendDelivery::into_delivery))
    }

    /// Identifiant de remise orpheline (phase `orphaned`), pour le rejeu honnête.
    pub fn orphaned_delivery_id(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<String>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id FROM send_deliveries
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2
                   AND phase = 'orphaned'",
                params![key.issuer_scope, key.idempotency_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Reprise bornée : seules les remises encore en cours pour l'instance
    /// exacte sont relivrées. Une remise Acked ou Indeterminate ne l'est pas.
    /// Toute ligne `dispatching` sans enveloppe est classée `indeterminate`
    /// dans la même transaction avant que les autres remises soient rendues.
    ///
    /// Complexité : O(N), N étant le nombre total de remises faute d'index sur
    /// l'instance ; le reclassement est groupé pour éviter N écritures.
    pub fn dispatching_deliveries_for_instance(
        &mut self,
        recipient_instance_id: &str,
        now: i64,
    ) -> Result<Vec<SendDelivery>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let reclassified = tx.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate'
             WHERE recipient_instance_id = ?1 AND phase = 'dispatching'
               AND message_bytes IS NULL",
            params![recipient_instance_id],
        )?;
        let stored = {
            let mut statement = tx.prepare(
                "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at, message_bytes
                 FROM send_deliveries
                 WHERE recipient_instance_id = ?1 AND phase = 'dispatching' AND expires_at > ?2
                 ORDER BY delivery_id",
            )?;
            statement
                .query_map(params![recipient_instance_id, now], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, u64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Option<Vec<u8>>>(4)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?
        };
        let deliveries = stored
            .into_iter()
            .map(
                |(
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message_bytes,
                )| {
                    let message_bytes = message_bytes.ok_or(IdempotencyError::CorruptRecord(
                        "remise dispatching sans enveloppe après reclassement",
                    ))?;
                    Ok(SendDelivery {
                        delivery_id,
                        recipient_instance_id,
                        delivery_generation,
                        expires_at,
                        message_bytes,
                    })
                },
            )
            .collect::<Result<Vec<_>, IdempotencyError>>()?;
        tx.commit()?;
        if reclassified > 0 {
            log::warn!(
                "{reclassified} remise(s) de {recipient_instance_id} classée(s) indeterminate: enveloppe locale absente"
            );
        }
        Ok(deliveries)
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

    /// Après purge d'une présence : les remises encore `dispatching` pour cette
    /// instance deviennent `orphaned` (sort CONNU) et le socle passe en
    /// terminal `orphaned`. Distinct de `indeterminate` (quarantaine d'injection)
    /// et de `outcome_unknown` (sort réellement inconnu).
    pub fn orphan_dispatching_for_instance(
        &mut self,
        recipient_instance_id: &str,
        reason: &str,
    ) -> Result<Vec<OrphanedDeliveryNotice>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut statement = tx.prepare(
            "SELECT delivery_id, issuer_scope, idempotency_key, message_bytes
             FROM send_deliveries
             WHERE recipient_instance_id = ?1 AND phase = 'dispatching'",
        )?;
        let rows: Vec<(String, String, String, Vec<u8>)> = statement
            .query_map(params![recipient_instance_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?.unwrap_or_default(),
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);

        let mut notices = Vec::with_capacity(rows.len());
        for (delivery_id, issuer_scope, idempotency_key, message_bytes) in rows {
            let message =
                serde_json::from_slice::<bridget_core::BridgetMessage>(&message_bytes).ok();
            let (message_id, sender, target) = match &message {
                Some(msg) => (msg.id.clone(), msg.from.clone(), msg.to.clone()),
                None => (
                    delivery_id.clone(),
                    "inconnu".to_string(),
                    "inconnu".to_string(),
                ),
            };
            let delivery = tx.execute(
                "UPDATE send_deliveries SET phase = 'orphaned'
                 WHERE delivery_id = ?1 AND phase = 'dispatching'",
                params![delivery_id],
            )?;
            let record = tx.execute(
                "UPDATE idempotency_records
                 SET state = 'terminal', public_result_kind = 'orphaned',
                     public_result_category = NULL, public_result_reason = ?1
                 WHERE issuer_scope = ?2 AND operation_kind = 'send'
                   AND idempotency_key = ?3 AND state = 'dispatching'",
                params![reason, issuer_scope, idempotency_key],
            )?;
            if delivery != 1 || record != 1 {
                return Err(IdempotencyError::DispatchUnavailable);
            }
            // Conduite (pas seulement le constat) : `orphaned` est absorbant —
            // le réflexe REJEU_A_L_IDENTIQUE enseigné pour in_flight/outcome_unknown
            // ferait tourner l'émetteur en rond. Aligné sur mcp::CONDUITE_ORPHELIN.
            let body = format!(
                "ORPHELIN: le message {} destiné à {} n'a pas été livré — présence purgée (delivery {}). {}\n\
                 Le rejeu à l'identique ne sert à rien — cette clé est close. \
                 Change de destinataire, ou attends son retour avec une clé neuve.",
                message_id, target, delivery_id, reason
            );
            // Même transaction : le signal survit à un crash avant le Deliver.
            tx.execute(
                "INSERT INTO orphan_emitter_notices (delivery_id, sender, body, created_at, notified_at)
                 VALUES (?1, ?2, ?3, strftime('%s','now'), NULL)
                 ON CONFLICT(delivery_id) DO NOTHING",
                params![delivery_id, sender, body],
            )?;
            notices.push(OrphanedDeliveryNotice {
                delivery_id,
                message_id,
                sender,
                target,
                reason: reason.to_string(),
            });
        }
        tx.commit()?;
        Ok(notices)
    }

    /// Notices émetteur encore non poussées (crash entre orphan et Deliver).
    pub fn pending_orphan_emitter_notices(
        &self,
    ) -> Result<Vec<(String, String, String)>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT delivery_id, sender, body FROM orphan_emitter_notices
             WHERE notified_at IS NULL ORDER BY created_at, delivery_id",
        )?;
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Marque la notice comme émise sur la socket (`writeln!`+`flush` Ok).
    /// Limite : ce n'est pas une attestation de réception par le wrapper.
    pub fn mark_orphan_emitter_notified(
        &self,
        delivery_id: &str,
        now: i64,
    ) -> Result<(), IdempotencyError> {
        self.conn.execute(
            "UPDATE orphan_emitter_notices SET notified_at = ?1
             WHERE delivery_id = ?2 AND notified_at IS NULL",
            params![now, delivery_id],
        )?;
        Ok(())
    }

    /// Accusé aval : la remise et le résultat public deviennent terminaux dans
    /// une même transaction, après validation de l'instance et génération.
    /// Le ledger d'émission a déjà été gravé à `begin_send_delivery` — ici on
    /// ne fait que soldater l'accusé et le cycle de réponse, sans réécrire le
    /// fait d'émission (visibilité ≠ accusé).
    pub fn acknowledge_send_delivery(
        &mut self,
        delivery_id: &str,
        recipient_instance_id: &str,
        delivery_generation: u64,
    ) -> Result<Option<String>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let row = tx.query_row(
            "SELECT issuer_scope, idempotency_key, recipient_instance_id, delivery_generation, phase, message_bytes
             FROM send_deliveries WHERE delivery_id = ?1",
            params![delivery_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<Vec<u8>>>(5)?,
            )),
        ).optional()?.ok_or(IdempotencyError::InvalidDelivery)?;
        if row.2 != recipient_instance_id || row.3 != delivery_generation {
            return Err(IdempotencyError::InvalidDelivery);
        }
        if row.4 == "acked" {
            tx.commit()?;
            return Ok(None);
        }
        if row.4 != "dispatching" {
            return Err(IdempotencyError::InvalidDelivery);
        }
        let envelope_missing = row.5.is_none();
        let message = row
            .5
            .as_deref()
            .and_then(|bytes| serde_json::from_slice::<bridget_core::BridgetMessage>(bytes).ok());
        let delivery = tx.execute("UPDATE send_deliveries SET phase = 'acked' WHERE delivery_id = ?1 AND phase = 'dispatching'", params![delivery_id])?;
        let record = tx.execute(
            "UPDATE idempotency_records SET state = 'terminal', public_result_kind = 'accepted', public_result_category = NULL, public_result_reason = NULL
             WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2 AND state = 'dispatching'",
            params![row.0, row.1],
        )?;
        if delivery != 1 || record != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let answered_request = if let Some(message) = message {
            if let Some(request_id) = message.in_reply_to.as_deref() {
                let changed = crate::store::mark_answered_in_transaction(
                    &tx,
                    request_id,
                    &message.from,
                    &message.to,
                )?;
                changed.then(|| request_id.to_string())
            } else {
                None
            }
        } else {
            None
        };
        tx.commit()?;
        if envelope_missing {
            log::warn!(
                "accusé idempotent {delivery_id} enregistré sans enveloppe locale: corrélation in_reply_to impossible"
            );
        }
        Ok(answered_request)
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
            PublicResult::Orphaned { reason } => ("orphaned", None, Some(reason.as_str())),
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
        load_record_from(&self.conn, key)
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

fn load_record_from(
    conn: &Connection,
    key: &IdempotencyKey,
) -> Result<Option<Record>, IdempotencyError> {
    conn.query_row(
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

fn load_spawn_command_from(
    conn: &Connection,
    key: &IdempotencyKey,
) -> Result<Option<SpawnCommand>, IdempotencyError> {
    conn.query_row(
        "SELECT command_id, name, generation, persistent, state,
                instance_id, deadline_at, expires_at,
                issue_kind, issue_category, issue_reason, resolved_definition_json
         FROM spawn_commands
         WHERE issuer_scope = ?1 AND operation_kind = 'spawn' AND command_id = ?2",
        params![key.issuer_scope, key.idempotency_key],
        spawn_command_from_row,
    )
    .optional()
    .map_err(Into::into)
}

fn agent_link_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentLinkRecord> {
    Ok(AgentLinkRecord {
        link_id: row.get(0)?,
        parent_instance_id: row.get(1)?,
        child_instance_id: row.get(2)?,
        parent_execution_id: row.get(3)?,
        objective_id: row.get(4)?,
        delegation_id: row.get(5)?,
        role: row.get(6)?,
        agent_path: row.get(7)?,
        state: AgentLinkState::from_str(&row.get::<_, String>(8)?).map_err(to_sql_error)?,
        created_at: row.get(9)?,
        closed_at: row.get(10)?,
        revision: row.get(11)?,
        project: project_reference_from_parts(row.get(12)?, row.get(13)?)?,
    })
}

fn agent_link_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentLinkEvent> {
    Ok(AgentLinkEvent {
        cursor: row.get(0)?,
        event_id: row.get(1)?,
        link_id: row.get(2)?,
        parent_instance_id: row.get(3)?,
        child_instance_id: row.get(4)?,
        state: AgentLinkState::from_str(&row.get::<_, String>(5)?).map_err(to_sql_error)?,
        observed_at: row.get(6)?,
        project: project_reference_from_parts(row.get(7)?, row.get(8)?)?,
    })
}

fn record_agent_link_event(
    tx: &rusqlite::Transaction<'_>,
    link: &AgentLinkRecord,
    observed_at: i64,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO agent_link_events (
            event_id, link_id, parent_instance_id, child_instance_id, state, observed_at,
            project_id, binding_generation
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            format!("{}:{}", link.link_id, link.revision),
            link.link_id,
            link.parent_instance_id,
            link.child_instance_id,
            link.state.as_str(),
            observed_at.max(link.created_at),
            link.project.as_ref().map(|project| &project.project_id),
            link.project
                .as_ref()
                .map(|project| i64::try_from(project.binding_generation))
                .transpose()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(7, i64::MAX))?,
        ],
    )?;
    Ok(())
}

fn delegated_runtime_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<DelegatedRuntimeEventRecord> {
    let kind = row.get::<_, String>(6)?;
    Ok(DelegatedRuntimeEventRecord {
        cursor: row.get(0)?,
        event_id: row.get(1)?,
        link_id: row.get(2)?,
        parent_instance_id: row.get(3)?,
        child_instance_id: row.get(4)?,
        child_execution_id: row.get(5)?,
        kind: DelegatedRuntimeEventKind::from_str(&kind)
            .ok_or_else(|| to_sql_error(IdempotencyError::InvalidDelegatedRuntimeEvent))?,
        code: row.get(7)?,
        reference: row.get(8)?,
        observed_at: row.get(9)?,
        acknowledged_at: row.get(10)?,
        project: project_reference_from_parts(row.get(11)?, row.get(12)?)?,
    })
}

fn project_reference_from_parts(
    project_id: Option<String>,
    binding_generation: Option<i64>,
) -> rusqlite::Result<Option<ProjectReference>> {
    match (project_id, binding_generation) {
        (None, None) => Ok(None),
        (Some(project_id), Some(binding_generation))
            if !project_id.trim().is_empty() && binding_generation > 0 =>
        {
            Ok(Some(ProjectReference {
                project_id,
                binding_generation: u64::try_from(binding_generation)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, binding_generation))?,
            }))
        }
        _ => Err(to_sql_error(IdempotencyError::InvalidAgentLink)),
    }
}

fn validate_delegated_runtime_event_input(
    input: &DelegatedRuntimeEventInput,
) -> Result<(), IdempotencyError> {
    let token = |value: &str, limit: usize| {
        !value.is_empty()
            && value.len() <= limit
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    };
    let reference = input.reference.strip_prefix("sha256:").unwrap_or_default();
    if !token(&input.child_instance_id, 256)
        || !token(&input.child_execution_id, 256)
        || !token(&input.code, 128)
        || reference.len() != 64
        || !reference.bytes().all(|byte| byte.is_ascii_hexdigit())
        || input.observed_at <= 0
    {
        return Err(IdempotencyError::InvalidDelegatedRuntimeEvent);
    }
    Ok(())
}

fn delegated_runtime_event_id(
    link: &AgentLinkRecord,
    input: &DelegatedRuntimeEventInput,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"bridget/delegated-runtime-event/v1\0");
    let kind = input.kind.as_str();
    let parts: [&[u8]; 5] = [
        link.link_id.as_bytes(),
        input.child_execution_id.as_bytes(),
        kind.as_bytes(),
        input.code.as_bytes(),
        input.reference.as_bytes(),
    ];
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    format!("delegated-runtime:{:x}", digest.finalize())
}

fn unix_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

fn spawn_command_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SpawnCommand> {
    let command_id = row.get(0)?;
    let name: String = row.get(1)?;
    let generation: u64 = row.get(2)?;
    let persistent: bool = row.get(3)?;
    let state = SpawnCommandState::from_str(&row.get::<_, String>(4)?).map_err(to_sql_error)?;
    let instance_id: Option<String> = row.get(5)?;
    let deadline_at = row.get(6)?;
    let expires_at = row.get(7)?;
    let issue_kind: Option<String> = row.get(8)?;
    let issue_category: Option<String> = row.get(9)?;
    let issue_reason: Option<String> = row.get(10)?;
    let resolved_definition = row
        .get::<_, Option<String>>(11)?
        .map(|json| {
            serde_json::from_str::<ResolvedAgentDefinition>(&json).map_err(|_| {
                to_sql_error(IdempotencyError::CorruptRecord(
                    "définition résolue du spawn invalide",
                ))
            })
        })
        .transpose()?;
    let issue = match issue_kind.as_deref() {
        None if !state.is_terminal() => None,
        Some("connected") if state == SpawnCommandState::Connected => {
            Some(SpawnCommandIssue::Connected {
                name: name.clone(),
                generation,
                instance_id: instance_id.clone().ok_or_else(|| {
                    to_sql_error(IdempotencyError::CorruptRecord(
                        "instance spawn connectée absente",
                    ))
                })?,
                definition: resolved_definition.clone().map(Box::new),
            })
        }
        Some("failed") if state == SpawnCommandState::Failed => Some(SpawnCommandIssue::Failed {
            category: issue_category.ok_or_else(|| {
                to_sql_error(IdempotencyError::CorruptRecord("catégorie spawn absente"))
            })?,
            reason: issue_reason.ok_or_else(|| {
                to_sql_error(IdempotencyError::CorruptRecord("motif spawn absent"))
            })?,
        }),
        Some("cancelled") if state == SpawnCommandState::Cancelled => {
            Some(SpawnCommandIssue::Cancelled {
                reason: issue_reason.ok_or_else(|| {
                    to_sql_error(IdempotencyError::CorruptRecord(
                        "motif annulation spawn absent",
                    ))
                })?,
            })
        }
        _ => {
            return Err(to_sql_error(IdempotencyError::CorruptRecord(
                "issue spawn incohérente",
            )));
        }
    };
    Ok(SpawnCommand {
        command_id,
        name,
        generation,
        persistent,
        state,
        instance_id,
        deadline_at,
        expires_at,
        issue,
        resolved_definition,
    })
}

impl Record {
    fn validate_spawn_issue(&self, issue: &SpawnCommandIssue) -> Result<(), IdempotencyError> {
        let coherent = match issue {
            SpawnCommandIssue::Connected { .. } => {
                self.public_result_kind.as_deref() == Some("accepted")
                    && self.public_result_category.is_none()
                    && self.public_result_reason.is_none()
            }
            SpawnCommandIssue::Failed { category, reason } => {
                self.public_result_kind.as_deref() == Some("rejected")
                    && self.public_result_category.as_deref() == Some(category)
                    && self.public_result_reason.as_deref() == Some(reason)
            }
            SpawnCommandIssue::Cancelled { reason } => {
                self.public_result_kind.as_deref() == Some("rejected")
                    && self.public_result_category.as_deref() == Some("cancelled")
                    && self.public_result_reason.as_deref() == Some(reason)
            }
        };
        coherent
            .then_some(())
            .ok_or(IdempotencyError::CorruptRecord(
                "issue spawn incohérente avec le socle",
            ))
    }

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
            Some("orphaned") => Ok(LookupResult::Orphaned {
                expires_at: self.expires_at,
                reason: self
                    .public_result_reason
                    .clone()
                    .unwrap_or_else(|| "destinataire purgé — remise orpheline".to_string()),
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
    use std::sync::{Arc, Barrier, Mutex, Once};
    use std::thread;

    const NOW: i64 = 1_000_000;
    const HORIZON: i64 = 3600;

    /// Un seul logger global (contrainte `log`) ; les tests qui lisent la trace
    /// prennent `TEST_LOG_LOCK` pour ne pas se marcher dessus.
    static TEST_LOG_LOCK: Mutex<()> = Mutex::new(());
    static TEST_WARN_LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

    struct TempDbGuard(std::path::PathBuf);

    impl TempDbGuard {
        fn new(name_prefix: &str) -> Self {
            Self(std::env::temp_dir().join(format!("{name_prefix}-{}.db", uuid::Uuid::new_v4())))
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempDbGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn init_test_warn_logger() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            struct CapturingLogger;
            impl log::Log for CapturingLogger {
                fn enabled(&self, metadata: &log::Metadata) -> bool {
                    metadata.level() <= log::Level::Warn
                }
                fn log(&self, record: &log::Record) {
                    if record.level() <= log::Level::Warn {
                        TEST_WARN_LOGS
                            .lock()
                            .expect("TEST_WARN_LOGS")
                            .push(record.args().to_string());
                    }
                }
                fn flush(&self) {}
            }
            static LOGGER: CapturingLogger = CapturingLogger;
            log::set_logger(&LOGGER).expect("logger de test installable");
            log::set_max_level(log::LevelFilter::Warn);
        });
    }

    fn commencer_capture_trace_sans_enveloppe() {
        init_test_warn_logger();
        let _guard = TEST_LOG_LOCK.lock().expect("TEST_LOG_LOCK");
        TEST_WARN_LOGS.lock().expect("TEST_WARN_LOGS").clear();
    }

    fn traces_sans_enveloppe() -> Vec<String> {
        TEST_WARN_LOGS
            .lock()
            .expect("TEST_WARN_LOGS")
            .iter()
            .filter(|line| line.contains("corrélation in_reply_to impossible"))
            .cloned()
            .collect()
    }

    fn key() -> IdempotencyKey {
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message-1").unwrap()
    }

    fn spawn_key() -> IdempotencyKey {
        IdempotencyKey::new(
            "supervisor_009_aaaaaaaaaaaa",
            OperationKind::Spawn,
            "command-1",
        )
        .unwrap()
    }

    fn reserve(store: &IdempotencyStore, bytes: &[u8]) -> Reservation {
        store.reserve(&key(), bytes, NOW, HORIZON, NOW, 30).unwrap()
    }

    #[test]
    fn migration_v3_ajoute_la_definition_resolue_aux_sagas_existantes() {
        let path = std::env::temp_dir().join(format!(
            "bridget-idempotency-definition-migration-{}.db",
            uuid::Uuid::new_v4()
        ));
        {
            let legacy = Connection::open(&path).unwrap();
            legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );",
                )
                .unwrap();
        }
        let store = IdempotencyStore::open(&path).unwrap();
        let columns = store
            .conn
            .prepare("PRAGMA table_info(spawn_commands)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            columns
                .iter()
                .any(|column| column == "resolved_definition_json")
        );
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    /// ORACLE — migration v4 : une base pré-orphaned DOIT élargir le CHECK.
    /// Meurt si l'on retire le bloc v4 alors que le CREATE fresh porte déjà
    /// `orphaned` (les oracles de phase restent verts, la montée reste muette).
    ///
    /// Nature de la charge : trois migrations sur quatre avaient déjà leur
    /// témoin dans ce fichier — la v4 était la seule orpheline. Ce n'est pas
    /// une pratique à instaurer, c'est une pratique à ne pas rompre. Montage :
    /// DDL **v3** à la main (CHECK à trois phases), pas une base neuve.
    #[test]
    fn migration_v4_elargit_le_check_pour_accepter_orphaned() {
        let path = std::env::temp_dir().join(format!(
            "bridget-idempotency-v4-orphan-{}.db",
            uuid::Uuid::new_v4()
        ));
        {
            let legacy = Connection::open(&path).unwrap();
            legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                     CREATE TABLE idempotency_records (
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
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        resolved_definition_json TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );
                     INSERT INTO idempotency_records VALUES (
                        '012_scope_aaaaaaaaaaaa', 'send', 'legacy-key', X'00',
                        'dispatching', NULL, NULL, NULL, 1000000, 1003600
                     );
                     INSERT INTO send_deliveries VALUES (
                        'delivery-legacy', '012_scope_aaaaaaaaaaaa', 'send', 'legacy-key',
                        'instance-1', 1, 'dispatching', 1003600, X'7b7d'
                     );",
                )
                .unwrap();
            // `X'7b7d'` (= `{}`) suffit ici : cet oracle ne teste que le CHECK
            // via UPDATE brut. Un chemin réel (`orphan_dispatching_for_instance`)
            // exige un vrai `BridgetMessage` sérialisé — voir
            // `chemin_reel_sur_base_migree_depuis_v3`.
            // Sur le CHECK pré-v4, orphaned est refusé.
            let refused = legacy.execute(
                "UPDATE send_deliveries SET phase = 'orphaned' WHERE delivery_id = 'delivery-legacy'",
                [],
            );
            assert!(
                refused.is_err(),
                "précondition : le CHECK pré-v4 doit refuser orphaned"
            );
        }

        let store = IdempotencyStore::open(&path).unwrap();
        let has_v4: bool = store
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 4)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_v4, "la migration v4 doit être consignées");
        store
            .conn
            .execute(
                "UPDATE send_deliveries SET phase = 'orphaned' WHERE delivery_id = 'delivery-legacy'",
                [],
            )
            .expect("après v4, orphaned doit passer le CHECK");
        let phase: String = store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(phase, "orphaned");
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    /// ORACLE — chemin de production sur base réellement migrée v3→v4.
    /// Complète `migration_v4_elargit…` (CHECK par UPDATE brut) : ici
    /// `orphan_dispatching_for_instance` désérialise `message_bytes`, écrit
    /// `orphan_emitter_notices`, et le lookup rend `Orphaned`.
    #[test]
    fn chemin_reel_sur_base_migree_depuis_v3() {
        let path =
            std::env::temp_dir().join(format!("bridget-chemin-reel-{}.db", uuid::Uuid::new_v4()));
        let message = bridget_core::BridgetMessage::new("emetteur-x", "cible-y", "mandat perdu");
        let message_id = message.id.clone();
        let bytes = serde_json::to_vec(&message).unwrap();
        let key = IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "cle-reelle")
            .unwrap();

        {
            let legacy = Connection::open(&path).unwrap();
            legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                     CREATE TABLE idempotency_records (
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
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        resolved_definition_json TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );
                     INSERT INTO idempotency_records VALUES (
                        '012_scope_aaaaaaaaaaaa', 'send', 'cle-reelle', X'00',
                        'dispatching', NULL, NULL, NULL, 1000000, 1003600
                     );",
                )
                .unwrap();
            legacy
                .execute(
                    "INSERT INTO send_deliveries VALUES (
                        'delivery-reel', '012_scope_aaaaaaaaaaaa', 'send', 'cle-reelle',
                        'instance-1', 1, 'dispatching', 1003600, ?1)",
                    params![bytes],
                )
                .unwrap();
        }

        let mut store = IdempotencyStore::open(&path).expect("migration v4");
        let notices = store
            .orphan_dispatching_for_instance("instance-1", "destinataire purge")
            .expect("chemin réel sur base migrée");

        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].delivery_id, "delivery-reel");
        assert_eq!(notices[0].message_id, message_id);
        assert_eq!(notices[0].sender, "emetteur-x");
        assert_eq!(notices[0].target, "cible-y");

        let phase: String = store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-reel'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(phase, "orphaned");

        let notices_en_base: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM orphan_emitter_notices", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(notices_en_base, 1);

        assert_eq!(
            store.lookup(&key, 1_000_000).unwrap(),
            LookupResult::Orphaned {
                expires_at: 1_003_600,
                reason: "destinataire purge".to_string(),
            }
        );

        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    /// ORACLE — migration v4 face à des enfants sans parent (FK OFF hors daemon).
    /// Meurt si l'INSERT SELECT échoue au démarrage : flotte bloquée.
    /// Meurt aussi si l'écart est silencieux (mutant : DELETE sans `warn!`).
    /// Nature : prouve le *mécanisme* (précaution). Occurrence nulle mesurée
    /// sur la copie prod du jour — ne pas lire cet oracle comme un incident.
    #[test]
    fn migration_v4_nettoie_les_enfants_sans_parent_sans_bloquer_le_daemon() {
        init_test_warn_logger();
        let _log_guard = TEST_LOG_LOCK.lock().expect("TEST_LOG_LOCK");
        TEST_WARN_LOGS.lock().expect("TEST_WARN_LOGS").clear();

        let db = TempDbGuard::new("bridget-idempotency-v4-fk-orphan");
        {
            let legacy = Connection::open(db.path()).unwrap();
            // Comme le CLI SQLite : FK OFF par défaut → orphelin de schéma possible.
            legacy.pragma_update(None, "foreign_keys", false).unwrap();
            legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                     CREATE TABLE idempotency_records (
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
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        resolved_definition_json TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );
                     -- Parent valide + enfant valide.
                     INSERT INTO idempotency_records VALUES (
                        '012_scope_aaaaaaaaaaaa', 'send', 'kept-key', X'00',
                        'dispatching', NULL, NULL, NULL, 1000000, 1003600
                     );
                     INSERT INTO send_deliveries VALUES (
                        'delivery-kept', '012_scope_aaaaaaaaaaaa', 'send', 'kept-key',
                        'instance-1', 1, 'dispatching', 1003600, X'7b7d'
                     );
                     -- Enfant SANS parent (impossible sous le daemon, possible hors daemon).
                     INSERT INTO send_deliveries VALUES (
                        'delivery-sans-parent', '012_scope_bbbbbbbbbbbb', 'send', 'ghost-key',
                        'instance-ghost', 1, 'dispatching', 1003600, X'7b7d'
                     );",
                )
                .unwrap();
        }

        // Ne doit PAS paniquer / échouer : c'est le démarrage du daemon.
        let store = IdempotencyStore::open(db.path())
            .expect("v4 ne doit pas bloquer le démarrage sur un enfant sans parent");
        let has_v4: bool = store
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 4)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_v4);
        let kept: usize = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id = 'delivery-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let ghost: usize = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id = 'delivery-sans-parent'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(kept, 1, "l'enfant valide survit");
        assert_eq!(ghost, 0, "l'enfant sans parent est nettoyé, pas bloquant");

        let logs = TEST_WARN_LOGS.lock().expect("TEST_WARN_LOGS").clone();
        let dit = logs.iter().any(|line| {
            line.contains("migration v4")
                && line.contains("écarté")
                && line.contains("delivery-sans-parent")
        });
        assert!(
            dit,
            "l'écart doit être DIT (warn! compte + delivery_id) ; logs={logs:?}"
        );

        drop(store);
    }

    #[test]
    fn creation_spawn_et_saga_sont_atomiques_sous_la_meme_fk() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = spawn_key();
        assert!(
            store
                .reserve_spawn(
                    &key,
                    b"canon-spawn",
                    NOW,
                    HORIZON,
                    NOW,
                    30,
                    "codex-1",
                    u64::MAX,
                    true,
                    NOW + 60,
                    "instance-1",
                )
                .is_err()
        );
        assert_eq!(store.record_count().unwrap(), 0);
        assert!(matches!(
            store
                .reserve_spawn(
                    &key,
                    b"canon-spawn",
                    NOW,
                    HORIZON,
                    NOW,
                    30,
                    "codex-1",
                    1,
                    true,
                    NOW + 60,
                    "instance-1",
                )
                .unwrap(),
            SpawnReservation::Requested(_)
        ));
        assert_eq!(store.spawn_commands(&key.issuer_scope).unwrap().len(), 1);
    }

    #[test]
    fn echec_de_finalisation_socle_annule_la_finalisation_saga() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = spawn_key();
        store
            .reserve_spawn(
                &key,
                b"canon-spawn",
                NOW,
                HORIZON,
                NOW,
                30,
                "codex-1",
                1,
                true,
                NOW + 60,
                "instance-1",
            )
            .unwrap();
        store
            .advance_spawn(
                &key,
                1,
                SpawnCommandState::Requested,
                SpawnCommandState::Reserved,
            )
            .unwrap();
        store
            .conn
            .execute(
                "UPDATE idempotency_records SET state = 'terminal',
                    public_result_kind = 'accepted'
                 WHERE issuer_scope = ?1 AND operation_kind = 'spawn'
                   AND idempotency_key = ?2",
                params![key.issuer_scope, key.idempotency_key],
            )
            .unwrap();
        assert!(
            store
                .finish_spawn(
                    &key,
                    1,
                    &SpawnCommandIssue::Failed {
                        category: "startup_failed".to_string(),
                        reason: "fixture".to_string(),
                    },
                )
                .is_err()
        );
        assert_eq!(
            store.spawn_commands(&key.issuer_scope).unwrap()[0].state,
            SpawnCommandState::Reserved
        );
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
            message_bytes: sample_message_bytes("message-1", "peer-1"),
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
    fn lien_causal_de_livraison_preserve_le_rejeu_exact() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-causale".to_string(),
            recipient_instance_id: "instance-1".to_string(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: sample_message_bytes("message-1", "peer-1"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        let expected = DeliveryExecutionLink {
            delivery_id: delivery.delivery_id.clone(),
            submission_id: "submission-1".to_string(),
            execution_id: "execution-1".to_string(),
        };
        store.link_send_delivery(&expected).unwrap();
        assert_eq!(
            store
                .delivery_execution_link(&delivery.delivery_id)
                .unwrap(),
            Some(expected)
        );
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            })
        );
        assert_eq!(store.send_delivery(&key).unwrap(), Some(delivery));
        assert!(matches!(
            store.link_send_delivery(&DeliveryExecutionLink {
                delivery_id: "delivery-causale".to_string(),
                submission_id: "submission-2".to_string(),
                execution_id: "execution-1".to_string(),
            }),
            Err(IdempotencyError::InvalidDelivery)
        ));
    }

    #[test]
    fn retry_en_vol_rejoue_unknown_puis_accepted_apres_accuse() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-retry".to_string(),
            recipient_instance_id: "instance-1".to_string(),
            delivery_generation: 9,
            expires_at: NOW + HORIZON,
            message_bytes: sample_message_bytes("message-1", "peer-1"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            })
        );
        store
            .acknowledge_send_delivery("delivery-retry", "instance-1", 9)
            .unwrap();
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::Accepted {
                expires_at: NOW + HORIZON
            })
        );
    }

    fn sample_message_bytes(id: &str, to: &str) -> Vec<u8> {
        let mut message = bridget_core::BridgetMessage::new("sender-peer", to, "corps collège");
        message.id = id.to_string();
        serde_json::to_vec(&message).expect("message sérialisable")
    }

    fn ledger_rows_for(store: &IdempotencyStore, id: &str) -> usize {
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM ledger WHERE id = ?1",
                params![id],
                |row| row.get::<_, usize>(0),
            )
            .unwrap()
    }

    fn delivery_phase(store: &IdempotencyStore, delivery_id: &str) -> String {
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = ?1",
                params![delivery_id],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn delivery_row_count(store: &IdempotencyStore, delivery_id: &str) -> usize {
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id = ?1",
                params![delivery_id],
                |row| row.get::<_, usize>(0),
            )
            .unwrap()
    }

    /// Oracle : enveloppe illisible → erreur explicite, aucune remise créée.
    /// Meurt si un `.ok()` silencieux réapparaît et laisse passer une remise
    /// `dispatching` sans ligne au ledger.
    #[test]
    fn begin_send_refuse_enveloppe_illisible_sans_creer_remise() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let err = store
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-corrupt".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: b"pas-du-json-message".to_vec(),
                },
            )
            .expect_err("enveloppe illisible doit remonter");
        assert!(
            matches!(err, IdempotencyError::CorruptRecord(_)),
            "attendu CorruptRecord, obtenu {err:?}"
        );
        assert_eq!(delivery_row_count(&store, "delivery-corrupt"), 0);
        assert_eq!(ledger_rows_for(&store, "message-1"), 0);
        assert_eq!(store.send_delivery(&key).unwrap(), None);
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT state FROM idempotency_records WHERE idempotency_key = 'message-1'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "prepared",
            "parse avant transaction : pas de demi-écriture dispatching"
        );
    }

    /// Oracle (ii) : un message vers un destinataire encore non accusé est
    /// VISIBLE au ledger ; la phase reste `dispatching` (accusé distinct).
    #[test]
    fn ledger_visible_pendant_dispatching_avant_accuse() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-visible".to_string(),
            recipient_instance_id: "instance-busy".to_string(),
            delivery_generation: 3,
            expires_at: NOW + HORIZON,
            message_bytes: sample_message_bytes("msg-visible-busy", "peer-busy"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        assert_eq!(delivery_phase(&store, "delivery-visible"), "dispatching");
        assert_eq!(
            ledger_rows_for(&store, "msg-visible-busy"),
            1,
            "le fait d'émission doit être au ledger avant DeliverAcked"
        );
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            }),
            "outcome_unknown reste nominal tant que l'accusé manque"
        );
        store
            .acknowledge_send_delivery("delivery-visible", "instance-busy", 3)
            .unwrap();
        assert_eq!(delivery_phase(&store, "delivery-visible"), "acked");
        assert_eq!(
            ledger_rows_for(&store, "msg-visible-busy"),
            1,
            "l'accusé ne doit pas dupliquer la ligne d'émission"
        );
        assert_eq!(
            reserve(&store, b"canon"),
            Reservation::Replayed(LookupResult::Accepted {
                expires_at: NOW + HORIZON
            })
        );
    }

    /// Oracle (iii) : les remises d'une instance morte sont reprises sur la
    /// nouvelle instance (réassignation), sans toucher aux phases terminales.
    #[test]
    fn remises_dispatching_migrees_au_changement_d_instance() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-orphan".to_string(),
            recipient_instance_id: "instance-morte".to_string(),
            delivery_generation: 7,
            expires_at: NOW + HORIZON,
            message_bytes: sample_message_bytes("msg-orphan", "peer-respawn"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        assert_eq!(
            store
                .reassign_dispatching_deliveries("instance-morte", "instance-vivante", NOW)
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .dispatching_deliveries_for_instance("instance-morte", NOW)
                .unwrap()
                .len(),
            0
        );
        let revived = store
            .dispatching_deliveries_for_instance("instance-vivante", NOW)
            .unwrap();
        assert_eq!(revived.len(), 1);
        assert_eq!(revived[0].delivery_id, "delivery-orphan");
        assert_eq!(revived[0].recipient_instance_id, "instance-vivante");
        // Ack sur la nouvelle instance doit aboutir.
        store
            .acknowledge_send_delivery("delivery-orphan", "instance-vivante", 7)
            .unwrap();
        assert_eq!(delivery_phase(&store, "delivery-orphan"), "acked");
        assert_eq!(
            store
                .reassign_dispatching_deliveries("instance-vivante", "autre", NOW)
                .unwrap(),
            0,
            "une remise déjà acked ne migre pas"
        );
    }

    /// ORACLE — purge de présence ⇒ phase `orphaned`, lookup ≠ outcome_unknown.
    #[test]
    fn purge_orpheline_les_remises_dispatching_sans_outcome_unknown() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-purge".to_string(),
            recipient_instance_id: "instance-purgée".to_string(),
            delivery_generation: 2,
            expires_at: NOW + HORIZON,
            message_bytes: sample_message_bytes("msg-purge", "relec-zombie"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        let notices = store
            .orphan_dispatching_for_instance(
                "instance-purgée",
                "destinataire purgé — présence absente ; remise orpheline",
            )
            .unwrap();
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].delivery_id, "delivery-purge");
        assert_eq!(notices[0].target, "relec-zombie");
        assert_eq!(delivery_phase(&store, "delivery-purge"), "orphaned");
        assert!(
            store
                .dispatching_deliveries_for_instance("instance-purgée", NOW)
                .unwrap()
                .is_empty(),
            "plus aucune remise en vol après orphelinage"
        );
        assert_eq!(
            store.lookup(&key, NOW).unwrap(),
            LookupResult::Orphaned {
                expires_at: NOW + HORIZON,
                reason: "destinataire purgé — présence absente ; remise orpheline".to_string(),
            },
            "le rejeu doit dire orphelin, jamais outcome_unknown"
        );
        assert_eq!(
            store.orphaned_delivery_id(&key).unwrap().as_deref(),
            Some("delivery-purge")
        );
        assert!(
            store
                .orphan_dispatching_for_instance("instance-purgée", "rejeu")
                .unwrap()
                .is_empty(),
            "orphelinage absorbant"
        );
    }

    #[test]
    fn purge_expired_removes_its_delivery_through_the_foreign_key() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        store
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-expired".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 2,
                    expires_at: NOW + HORIZON,
                    message_bytes: sample_message_bytes("message-1", "peer-1"),
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
        let second =
            IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message-2")
                .unwrap();
        assert!(matches!(
            reserve(&store, b"first"),
            Reservation::Prepared { .. }
        ));
        store
            .begin_send_delivery(
                &first,
                &SendDelivery {
                    delivery_id: "same-delivery".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 3,
                    expires_at: NOW + HORIZON,
                    message_bytes: sample_message_bytes("message-1", "peer-1"),
                },
            )
            .unwrap();
        assert!(matches!(
            store
                .reserve(&second, b"second", NOW, HORIZON, NOW, 30)
                .unwrap(),
            Reservation::Prepared { .. }
        ));
        assert!(
            store
                .begin_send_delivery(
                    &second,
                    &SendDelivery {
                        delivery_id: "same-delivery".to_string(),
                        recipient_instance_id: "instance-1".to_string(),
                        delivery_generation: 4,
                        expires_at: NOW + HORIZON,
                        message_bytes: sample_message_bytes("message-2", "peer-1"),
                    },
                )
                .is_err()
        );
        assert_eq!(
            store.lookup(&second, NOW).unwrap(),
            LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            }
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
            message_bytes: sample_message_bytes("message-1", "agent-2"),
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
            assert!(matches!(
                reserve(&store, b"canon"),
                Reservation::Prepared { .. }
            ));
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
            assert!(
                store
                    .begin_send_delivery_with_reply(&key, &delivery, &reply)
                    .is_err()
            );
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
        assert_eq!(
            reopened.prepared_expiry(&key, NOW).unwrap(),
            Some(NOW + HORIZON)
        );
        reopened
            .conn
            .execute(
                "DELETE FROM tracked_requests WHERE id = 'request-reply'",
                [],
            )
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
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        store
            .reject_prepared(&key, "routing", "cible absente")
            .unwrap();
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

        let mut store = IdempotencyStore::open(&path).unwrap();
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

    /// ORACLE C1 — chemin RUNTIME, enveloppe PRÉSENTE.
    ///
    /// C'est le cas de l'unique occurrence réelle relevée en production : une
    /// remise que le daemon n'a pas pu redéployer (échec de reprise) ou que le
    /// wrapper a déclarée indéterminée passe en quarantaine. Son enveloppe est
    /// intacte, sa ligne est là — et c'est précisément ce qui la rendait
    /// crédible : `send_delivery` la remontait, l'appelant lisait « en vol,
    /// rejouez », rc=0, pendant les 7 jours de l'horizon. Or la quarantaine est
    /// un état ABSORBANT : plus aucune transition n'en sort, ce message ne sera
    /// jamais accusé.
    ///
    /// Cet oracle tue le mutant exact qui retire la garde de phase : avec une
    /// enveloppe présente, l'écart porte sur le sens métier et non sur le
    /// décodage SQLite.
    #[test]
    fn une_remise_en_quarantaine_ne_remonte_plus_comme_une_remise_en_vol() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let key = key();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        let delivery = SendDelivery {
            delivery_id: "delivery-quarantaine".to_string(),
            recipient_instance_id: "instance-1".to_string(),
            delivery_generation: 4,
            expires_at: NOW + HORIZON,
            message_bytes: sample_message_bytes("message-1", "peer-1"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
        // Avant la quarantaine, la remise est bien en vol : sans ce constat,
        // l'oracle passerait aussi pour une clé qui n'a jamais rien déposé.
        assert_eq!(store.send_delivery(&key).unwrap(), Some(delivery.clone()));

        store
            .mark_delivery_indeterminate("delivery-quarantaine", "instance-1", 4)
            .unwrap();

        assert_eq!(
            store
                .send_delivery_mutant_sans_filtre_de_phase(&key)
                .unwrap(),
            Some(delivery),
            "contrôle positif du mutant : sans garde de phase, une enveloppe \
             présente ferait passer la quarantaine pour une remise en vol"
        );
        assert_eq!(
            store.send_delivery(&key).unwrap(),
            None,
            "une remise en quarantaine doit cesser d'attester un dépôt : \
             sans cela, l'appelant lit « en vol, rejouez » sur un message \
             que plus rien n'accusera jamais"
        );
        // L'enveloppe reste en base — la ligne n'est pas supprimée, elle est
        // seulement écartée de ce que « remise en vol » désigne.
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-quarantaine'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "indeterminate"
        );
    }

    /// ORACLE C1 — chemin MIGRATION, enveloppe ABSENTE.
    ///
    /// La seconde porte vers la quarantaine, et la plus large : la migration v2
    /// écarte d'un coup toutes les remises v1 dépourvues d'enveloppe. Une
    /// montée de version pouvait donc fabriquer en masse des remises
    /// définitivement inaccusables qui continuaient à s'annoncer comme des
    /// dépôts réussis.
    ///
    /// Cet oracle garde la projection nullable : même sous le mutant exact qui
    /// retire la garde de phase, l'absence d'enveloppe reste non relivrable et
    /// ne fuit jamais en `InvalidColumnType`. L'oracle runtime voisin, dont
    /// l'enveloppe est présente, est celui qui tue ce mutant sur le sens métier.
    #[test]
    fn une_remise_mise_en_quarantaine_par_la_migration_n_atteste_plus_un_depot() {
        let path = std::env::temp_dir().join(format!(
            "bridget-idempotency-quarantaine-migration-{}-{}.db",
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
        conn.execute(
            "INSERT INTO idempotency_records VALUES (?1, 'send', 'message-1', X'00', 'dispatching', NULL, NULL, NULL, ?2, ?3)",
            params!["012_scope_aaaaaaaaaaaa", NOW, NOW + HORIZON],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO send_deliveries VALUES ('delivery-sans-enveloppe', ?1, 'send', 'message-1', 'instance-1', 1, 'dispatching', ?2, NULL)",
            params!["012_scope_aaaaaaaaaaaa", NOW + HORIZON],
        )
        .unwrap();
        drop(conn);

        // L'ouverture applique la migration, qui met la remise en quarantaine.
        let store = IdempotencyStore::open(&path).unwrap();
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-sans-enveloppe'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "indeterminate",
            "prémisse de l'oracle : la migration doit bien avoir mis en quarantaine"
        );

        assert_eq!(
            store
                .send_delivery_mutant_sans_filtre_de_phase(&key())
                .unwrap(),
            None,
            "le mutant de phase ne doit pas réintroduire l'échec de décodage : \
             sans enveloppe, la remise reste non relivrable"
        );
        assert_eq!(
            store.send_delivery(&key()).unwrap(),
            None,
            "une remise mise en quarantaine par la migration ne doit pas \
             attester un dépôt : une montée de version en fabriquerait en masse"
        );
        std::fs::remove_file(path).unwrap();
    }

    // ========== ORACLES JURY (relecteur fable2 du lot parent) ==========
    // PROPRIÉTÉ SOUS TEST — celle que le lot déclare tenir :
    // « toute lecture de send_deliveries est totale sur le schéma hérité ;
    //   si message_bytes vaut NULL, aucun Sqlite(InvalidColumnType) ne fuit. »
    // Le lot parent la tient sur send_delivery. Ces deux oracles couvrent les
    // DEUX AUTRES chemins de lecture de message_bytes restés ouverts. Base
    // héritée où la quarantaine ne rejoue pas (v2 déjà marquée).

    fn base_heritee_dispatching_sans_enveloppe() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "bridget-jury-lecture-totale-{}-{}.db",
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
            );
            CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
            INSERT INTO idempotency_schema_migrations(version) VALUES (2);
            INSERT INTO idempotency_schema_migrations(version) VALUES (3);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO idempotency_records VALUES (?1, 'send', 'message-1', X'00', 'dispatching', NULL, NULL, NULL, ?2, ?3)",
            params!["012_scope_aaaaaaaaaaaa", NOW, NOW + HORIZON],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO send_deliveries VALUES ('delivery-encore-en-vol', ?1, 'send', 'message-1', 'instance-1', 1, 'dispatching', ?2, NULL)",
            params!["012_scope_aaaaaaaaaaaa", NOW + HORIZON],
        )
        .unwrap();
        drop(conn);
        path
    }

    fn ajouter_remise_valide_heritee(path: &Path, key: &str, delivery_id: &str, generation: u64) {
        let conn = Connection::open(path).unwrap();
        conn.execute(
            "INSERT INTO idempotency_records VALUES (?1, 'send', ?2, X'00', 'dispatching', NULL, NULL, NULL, ?3, ?4)",
            params!["012_scope_aaaaaaaaaaaa", key, NOW, NOW + HORIZON],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO send_deliveries VALUES (?1, ?2, 'send', ?3, 'instance-1', ?4, 'dispatching', ?5, ?6)",
            params![
                delivery_id,
                "012_scope_aaaaaaaaaaaa",
                key,
                generation,
                NOW + HORIZON,
                sample_message_bytes(key, "instance-1"),
            ],
        )
        .unwrap();
    }

    /// Chemin de REPRISE (daemon.rs:2293). `collect::<Result<Vec<_>>>` : une
    /// seule ligne sans enveloppe fait échouer la reprise ENTIÈRE de
    /// l'instance, y compris ses remises légitimes.
    #[test]
    fn jury_lecture_totale_chemin_reprise() {
        let path = base_heritee_dispatching_sans_enveloppe();
        let mut store = IdempotencyStore::open(&path).unwrap();
        let lu = store.dispatching_deliveries_for_instance("instance-1", NOW);
        let _ = std::fs::remove_file(&path);
        assert!(
            lu.is_ok(),
            "lecture non totale sur le chemin de reprise : {:?}",
            lu.err()
        );
    }

    /// Chemin d'ACCUSÉ. La lecture stricte de message_bytes précède même la
    /// garde de phase : aucune phase ne protège de la fuite.
    #[test]
    fn jury_lecture_totale_chemin_accuse() {
        let path = base_heritee_dispatching_sans_enveloppe();
        let mut store = IdempotencyStore::open(&path).unwrap();
        let lu = store.acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1);
        let _ = std::fs::remove_file(&path);
        assert!(
            !matches!(
                lu,
                Err(IdempotencyError::Sqlite(
                    rusqlite::Error::InvalidColumnType(..)
                ))
            ),
            "lecture non totale sur le chemin d'accusé : {:?}",
            lu.err()
        );
    }

    #[test]
    fn reprise_reclasse_sans_enveloppe_en_indeterminate() {
        let path = base_heritee_dispatching_sans_enveloppe();
        let mut store = IdempotencyStore::open(&path).unwrap();

        let deliveries = store
            .dispatching_deliveries_for_instance("instance-1", NOW)
            .expect("la reprise doit lire totalement le schéma hérité");

        assert_eq!(deliveries, Vec::<SendDelivery>::new());
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-encore-en-vol'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "indeterminate",
            "écarter une enveloppe absente sans reclasser laisserait un état absorbant silencieux"
        );
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn reprise_isole_deux_remises_valides_d_une_sans_enveloppe() {
        let path = base_heritee_dispatching_sans_enveloppe();
        ajouter_remise_valide_heritee(&path, "message-valid-a", "delivery-valid-a", 2);
        ajouter_remise_valide_heritee(&path, "message-valid-b", "delivery-valid-b", 3);
        let mut store = IdempotencyStore::open(&path).unwrap();

        let ids = store
            .dispatching_deliveries_for_instance("instance-1", NOW)
            .expect("une remise illisible ne doit pas faire échouer toute l'instance")
            .into_iter()
            .map(|delivery| delivery.delivery_id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "delivery-valid-a".to_string(),
                "delivery-valid-b".to_string(),
            ]
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-encore-en-vol'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "indeterminate",
            "rendre les valides ne suffit pas si la remise écartée reste dispatching"
        );
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn accuse_sans_enveloppe_respecte_les_trois_verdicts_metier() {
        commencer_capture_trace_sans_enveloppe();

        let path_dispatching = base_heritee_dispatching_sans_enveloppe();
        let mut store = IdempotencyStore::open(&path_dispatching).unwrap();
        let acknowledgement =
            store.acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1);
        let durable_state = store
            .conn
            .query_row(
                "SELECT d.phase, r.state, r.public_result_kind
                 FROM send_deliveries d
                 JOIN idempotency_records r
                   ON r.issuer_scope = d.issuer_scope
                  AND r.operation_kind = d.operation_kind
                  AND r.idempotency_key = d.idempotency_key
                 WHERE d.delivery_id = 'delivery-encore-en-vol'",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .unwrap();
        let traces = traces_sans_enveloppe();
        assert!(
            matches!(acknowledgement, Ok(None))
                && durable_state
                    == (
                        "acked".to_string(),
                        "terminal".to_string(),
                        Some("accepted".to_string()),
                    )
                && traces.iter().any(|trace| {
                    trace.contains("delivery-encore-en-vol")
                        && trace.contains("corrélation in_reply_to impossible")
                }),
            "l'accusé et sa trace doivent survivre à l'enveloppe locale absente: résultat={acknowledgement:?}, état={durable_state:?}, traces={traces:?}"
        );
        drop(store);
        std::fs::remove_file(path_dispatching).unwrap();

        let path_acked = base_heritee_dispatching_sans_enveloppe();
        let conn = Connection::open(&path_acked).unwrap();
        conn.execute(
            "UPDATE send_deliveries SET phase = 'acked' WHERE delivery_id = 'delivery-encore-en-vol'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE idempotency_records SET state = 'terminal', public_result_kind = 'accepted' WHERE idempotency_key = 'message-1'",
            [],
        )
        .unwrap();
        drop(conn);
        let mut store = IdempotencyStore::open(&path_acked).unwrap();
        assert_eq!(
            store
                .acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1)
                .expect("un accusé déjà enregistré reste idempotent"),
            None
        );
        drop(store);
        std::fs::remove_file(path_acked).unwrap();

        let path_indeterminate = base_heritee_dispatching_sans_enveloppe();
        let conn = Connection::open(&path_indeterminate).unwrap();
        conn.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate' WHERE delivery_id = 'delivery-encore-en-vol'",
            [],
        )
        .unwrap();
        drop(conn);
        let mut store = IdempotencyStore::open(&path_indeterminate).unwrap();
        assert!(matches!(
            store.acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1),
            Err(IdempotencyError::InvalidDelivery)
        ));
        assert_eq!(
            delivery_phase(&store, "delivery-encore-en-vol"),
            "indeterminate"
        );
        drop(store);
        std::fs::remove_file(path_indeterminate).unwrap();
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
    fn migration_v6_ajoute_les_references_projet_apres_une_base_deja_en_v5() {
        let path = std::env::temp_dir().join(format!(
            "bridget-idempotency-v6-{}.db",
            uuid::Uuid::new_v4()
        ));
        let legacy = Connection::open(&path).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (4);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (5);
                 CREATE TABLE agent_links (
                    link_id TEXT PRIMARY KEY,
                    parent_instance_id TEXT NOT NULL,
                    child_instance_id TEXT NOT NULL UNIQUE,
                    parent_execution_id TEXT,
                    objective_id TEXT,
                    delegation_id TEXT,
                    role TEXT NOT NULL,
                    agent_path TEXT NOT NULL,
                    state TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    closed_at INTEGER,
                    revision INTEGER NOT NULL
                 );
                 CREATE TABLE agent_link_events (
                    cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_id TEXT NOT NULL UNIQUE,
                    link_id TEXT NOT NULL,
                    parent_instance_id TEXT NOT NULL,
                    child_instance_id TEXT NOT NULL,
                    state TEXT NOT NULL,
                    observed_at INTEGER NOT NULL
                 );
                 CREATE TABLE delegated_runtime_events (
                    cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_id TEXT NOT NULL UNIQUE,
                    link_id TEXT NOT NULL,
                    parent_instance_id TEXT NOT NULL,
                    child_instance_id TEXT NOT NULL,
                    child_execution_id TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    code TEXT NOT NULL,
                    reference TEXT NOT NULL,
                    observed_at INTEGER NOT NULL,
                    acknowledged_at INTEGER
                 );",
            )
            .unwrap();
        drop(legacy);

        let store = IdempotencyStore::open(&path).unwrap();
        for table in [
            "agent_links",
            "agent_link_events",
            "delegated_runtime_events",
        ] {
            let has_project_id: bool = store
                .conn
                .query_row(
                    &format!(
                        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('{table}') WHERE name = 'project_id')"
                    ),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(has_project_id, "{table} migre project_id");
        }
        let has_v6: bool = store
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 6)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_v6);
        drop(store);
        std::fs::remove_file(path).unwrap();
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
    #[test]
    fn spec_068_faits_runtime_delegues_restent_ordonnes_et_accuses() {
        let mut store = IdempotencyStore::open_in_memory().unwrap();
        let project = ProjectReference {
            project_id: "project-068".to_string(),
            binding_generation: 4,
        };
        store
            .create_agent_link(&AgentLinkRecord {
                link_id: "link-068".to_string(),
                parent_instance_id: "instance-parent".to_string(),
                child_instance_id: "instance-child".to_string(),
                parent_execution_id: Some("execution-parent".to_string()),
                objective_id: Some("objective-068".to_string()),
                delegation_id: Some("delegation-068".to_string()),
                project: Some(project.clone()),
                role: "worker".to_string(),
                agent_path: "parent/child".to_string(),
                state: AgentLinkState::Reserved,
                created_at: NOW,
                closed_at: None,
                revision: 0,
            })
            .unwrap();

        assert_eq!(
            store
                .agent_link_events_after("instance-parent", None)
                .unwrap()
                .as_slice(),
            &[AgentLinkEvent {
                cursor: 1,
                event_id: "link-068:0".to_string(),
                link_id: "link-068".to_string(),
                parent_instance_id: "instance-parent".to_string(),
                child_instance_id: "instance-child".to_string(),
                state: AgentLinkState::Reserved,
                observed_at: NOW,
                project: Some(project.clone()),
            }]
        );

        let warning_reference = format!("sha256:{}", "a".repeat(64));
        let failed_reference = format!("sha256:{}", "b".repeat(64));

        let warning = store
            .record_delegated_runtime_event(DelegatedRuntimeEventInput {
                child_instance_id: "instance-child".to_string(),
                child_execution_id: "execution-child".to_string(),
                kind: DelegatedRuntimeEventKind::Warning,
                code: "unsupported_provider_request".to_string(),
                reference: warning_reference.clone(),
                observed_at: NOW + 1,
            })
            .unwrap();
        let failed = store
            .record_delegated_runtime_event(DelegatedRuntimeEventInput {
                child_instance_id: "instance-child".to_string(),
                child_execution_id: "execution-child".to_string(),
                kind: DelegatedRuntimeEventKind::Failed,
                code: "provider_failed".to_string(),
                reference: failed_reference.clone(),
                observed_at: NOW + 2,
            })
            .unwrap();
        assert_eq!(warning.project, Some(project.clone()));
        assert_eq!(failed.project, Some(project.clone()));
        assert_eq!(
            store
                .record_delegated_runtime_event(DelegatedRuntimeEventInput {
                    child_instance_id: "instance-child".to_string(),
                    child_execution_id: "execution-child".to_string(),
                    kind: DelegatedRuntimeEventKind::Warning,
                    code: "unsupported_provider_request".to_string(),
                    reference: warning_reference,
                    observed_at: NOW + 99,
                })
                .unwrap()
                .event_id,
            warning.event_id
        );
        let pending = store
            .delegated_runtime_events_for_parent("instance-parent")
            .unwrap();
        assert!(
            pending
                .iter()
                .all(|event| event.project == Some(project.clone()))
        );
        assert_eq!(
            pending
                .iter()
                .map(|event| event.event_id.as_str())
                .collect::<Vec<_>>(),
            vec![warning.event_id.as_str(), failed.event_id.as_str()]
        );
        assert!(matches!(
            store.acknowledge_delegated_runtime_event(&warning.event_id, "other-parent"),
            Err(IdempotencyError::InvalidDelegatedRuntimeEvent)
        ));
        assert!(
            store
                .acknowledge_delegated_runtime_event(&warning.event_id, "instance-parent")
                .unwrap()
        );
        assert!(
            !store
                .acknowledge_delegated_runtime_event(&warning.event_id, "instance-parent")
                .unwrap()
        );
        assert_eq!(
            store
                .delegated_runtime_events_for_parent("instance-parent")
                .unwrap()
                .iter()
                .map(|event| event.event_id.as_str())
                .collect::<Vec<_>>(),
            vec![failed.event_id.as_str()]
        );
    }
}
