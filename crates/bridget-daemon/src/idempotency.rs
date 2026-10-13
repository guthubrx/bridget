//! Socle transactionnel d'idempotence du daemon.
//!
//! Ce module ne connaît ni le routage ni la remise au wrapper. Il est le seul
//! propriétaire de la clé, des octets canoniques, de l'échéance et du résultat
//! public d'une opération idempotente.

mod agent_links;
mod send_delivery;
mod spawn_commands;

use bridget_transport::ResolvedAgentDefinition;
use bridget_transport::protocol::{DelegatedRuntimeEventKind, ProjectReference};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::Deserialize;
use std::path::{Path, PathBuf};

const MIN_ISSUER_SCOPE_LEN: usize = 22;
const MAX_ISSUER_SCOPE_LEN: usize = 128;
const MAX_IDEMPOTENCY_KEY_LEN: usize = 256;
const MAX_CANONICAL_BYTES: usize = 1024 * 1024;
const QUEUE_EDIT_RETENTION_SECS: i64 = 7 * 86_400;

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
pub(crate) struct StoredSendDelivery {
    pub(crate) delivery_id: String,
    pub(crate) recipient_instance_id: String,
    pub(crate) delivery_generation: u64,
    expires_at: i64,
    pub(crate) phase: String,
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
    pub remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
}

/// Preuve durable minimale permettant d'adopter explicitement un agent
/// historique arrêté. Elle ne peut provenir que d'une saga de spawn ayant
/// atteint l'état `connected` dans ce store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalManagedSpawn {
    pub agent_type: String,
    pub cwd: PathBuf,
    pub command_id: String,
    pub generation: u64,
    pub persistent: bool,
    pub project: Option<ProjectReference>,
    pub resolved_definition: ResolvedAgentDefinition,
    pub remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
}

#[derive(Debug, Deserialize)]
struct HistoricalCanonicalSpawnOrder {
    command_id: String,
    agent_type: String,
    cwd: String,
    persistent: bool,
    #[serde(default)]
    project: Option<ProjectReference>,
    #[serde(default)]
    remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
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
    Schema(crate::store_schema::SchemaError),
    InvalidIssuerScope,
    InvalidIdempotencyKey,
    CanonicalTooLarge,
    CliWakeRateLimited { retry_after_secs: u64 },
    InvalidIssuedAt,
    InvalidHorizon,
    InvalidTransition { from: RecordState, to: RecordState },
    InvalidDelivery,
    HeldQueueFull,
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
            Self::Schema(error) => write!(formatter, "{error}"),
            Self::InvalidIssuerScope => write!(formatter, "issuer_scope invalide"),
            Self::InvalidIdempotencyKey => write!(formatter, "clé d'idempotence invalide"),
            Self::CliWakeRateLimited { retry_after_secs } => write!(
                formatter,
                "rate_limited: retry_after_secs={retry_after_secs}"
            ),
            Self::CanonicalTooLarge => write!(formatter, "enveloppe canonique trop grande"),
            Self::InvalidIssuedAt => write!(formatter, "issued_at invalide"),
            Self::InvalidHorizon => write!(formatter, "horizon d'idempotence invalide"),
            Self::InvalidTransition { from, to } => {
                write!(formatter, "transition interdite: {from:?} vers {to:?}")
            }
            Self::InvalidDelivery => write!(formatter, "remise idempotente invalide"),
            Self::HeldQueueFull => write!(formatter, "file retenue pleine"),
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

/// Issue159B finale ; un rejeu n'emporte jamais de second effet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueueEditReceipt {
    pub state: bridget_transport::protocol::QueueEditState,
    pub refusal: Option<String>,
    pub effect: Option<bridget_transport::protocol::QueueEditEffect>,
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
        crate::store_schema::validate(conn).map_err(IdempotencyError::Schema)?;
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
                completed_at INTEGER,
                reply_summary TEXT
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
        // 151 : même colonne que Store ; l'ouvreur ne devine aucun ancien résumé.
        let has_reply_summary = tx
            .prepare("PRAGMA table_info(tracked_requests)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .any(|column| column == "reply_summary");
        if !has_reply_summary {
            tx.execute(
                "ALTER TABLE tracked_requests ADD COLUMN reply_summary TEXT",
                [],
            )?;
        }
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
                    rowid, delivery_id, issuer_scope, operation_kind, idempotency_key,
                    recipient_instance_id, delivery_generation, phase, expires_at, message_bytes
                ) SELECT
                    rowid, delivery_id, issuer_scope, operation_kind, idempotency_key,
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

        //159B : objet additif sur toute base v7, sans version8 ni clé étrangère.
        // delivery_id reste NULL lorsque le message public ne résout aucune remise.
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS queue_edits (
                issuer_scope TEXT NOT NULL,
                request_id TEXT NOT NULL,
                canonical_bytes BLOB NOT NULL,
                action TEXT NOT NULL CHECK (action IN ('cancel','reorder','promote')),
                actor TEXT NOT NULL,
                delivery_id TEXT,
                before_delivery_id TEXT,
                state TEXT NOT NULL CHECK (state IN ('applied','refused')),
                refusal TEXT,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                PRIMARY KEY (issuer_scope,request_id)
            );
            CREATE INDEX IF NOT EXISTS idx_queue_edits_expires ON queue_edits(expires_at);
            CREATE INDEX IF NOT EXISTS idx_queue_edits_delivery ON queue_edits(delivery_id);",
        )?;

        // Session156 : objets additifs APRÈS la reconstructionv4 éventuelle.
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS guidance_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                at INTEGER NOT NULL,
                actor TEXT NOT NULL,
                target TEXT NOT NULL,
                action TEXT NOT NULL CHECK (action IN ('steer','interrupt')),
                path TEXT NOT NULL CHECK (path IN ('send_idempotent','send','control','cancel')),
                message_id TEXT,
                outcome TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_guidance_events_target
                ON guidance_events(target, action, at);
            CREATE INDEX IF NOT EXISTS idx_guidance_events_pair
                ON guidance_events(actor, target, action, at);
            CREATE INDEX IF NOT EXISTS idx_send_deliveries_recipient_phase
                ON send_deliveries(recipient_instance_id, phase);",
        )?;
        // v7, dans l’Immediate existante : même saga, même FK, aucun compteur métier.
        // Coût froid O(d log r + (d+l) log(d+l) + B), d remises, l liens, r reçus,
        // B octets JSON. Copie SQLite temporaire O(l), détruite avant commit/rollback :
        // INSERT SELECT évite une requête d’écriture par lien et aucun registre ne demeure.
        let held_migration_applied = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 7)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !held_migration_applied {
            // Comme v4 : une modification externe FK OFF peut avoir créé un fantôme.
            // Sa suppression cascade aussi ses liens ; aucun enfant valide n’est retiré.
            let schema_orphans: Vec<String> = {
                let mut stmt = tx.prepare(
                    "SELECT d.delivery_id FROM send_deliveries d
                 WHERE NOT EXISTS (
                     SELECT 1 FROM idempotency_records r
                     WHERE r.issuer_scope = d.issuer_scope
                       AND r.operation_kind = d.operation_kind
                       AND r.idempotency_key = d.idempotency_key
                 )",
                )?;
                let rows = stmt
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            };
            if !schema_orphans.is_empty() {
                log::warn!(
                    "migration v7: {} remise(s) sans parent écartées dans la transaction ; \
                 origine externe, FK OFF ; delivery_id={:?}",
                    schema_orphans.len(),
                    schema_orphans,
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
            // CASE protège json_type/json_extract des BLOB NULL/JSON invalides.
            // Sans destinataire textuel, garder bytes et phase ; pas d’identité inventée.
            let unreadable: Vec<String> = {
                let mut stmt = tx.prepare(
                    "SELECT delivery_id FROM send_deliveries
                 WHERE CASE WHEN json_valid(message_bytes)
                            THEN COALESCE(json_type(message_bytes,'$.to'),'') != 'text'
                                 OR json_extract(message_bytes,'$.to') = ''
                            ELSE 1 END",
                )?;
                let rows = stmt
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            };
            if !unreadable.is_empty() {
                log::warn!(
                    "migration v7: destinataire illisible ; enveloppes et phases conservées, \
                 recipient_agent vide, aucun reroutage ; delivery_id={:?}",
                    unreadable,
                );
            }
            // DROP du parent déclenche ON DELETE CASCADE même avec une TX ouverte.
            // Copier les liens avant DROP, puis les restaurer sous FK ON ; aucun PRAGMA OFF.
            tx.execute_batch(
            "CREATE TEMP TABLE send_delivery_execution_links_v7_copy AS
                 SELECT delivery_id,submission_id,execution_id FROM send_delivery_execution_links;
             CREATE TABLE send_deliveries_v7 (
                 queue_rank INTEGER PRIMARY KEY AUTOINCREMENT,
                 delivery_id TEXT NOT NULL UNIQUE,
                 issuer_scope TEXT NOT NULL,
                 operation_kind TEXT NOT NULL CHECK (operation_kind = 'send'),
                 idempotency_key TEXT NOT NULL,
                 recipient_instance_id TEXT NOT NULL,
                 delivery_generation INTEGER NOT NULL CHECK (delivery_generation > 0),
                 phase TEXT NOT NULL CHECK (phase IN (
                     'dispatching','acked','indeterminate','orphaned','held','cancelled'
                 )),
                 expires_at INTEGER NOT NULL,
                 message_bytes BLOB,
                 recipient_agent TEXT NOT NULL,
                 turn_gate INTEGER NOT NULL DEFAULT 0 CHECK (turn_gate IN (0,1)),
                 acked_at INTEGER,
                 FOREIGN KEY (issuer_scope,operation_kind,idempotency_key)
                     REFERENCES idempotency_records(issuer_scope,operation_kind,idempotency_key)
                     ON DELETE CASCADE
             );
             INSERT INTO send_deliveries_v7 (
                 queue_rank,delivery_id,issuer_scope,operation_kind,idempotency_key,
                 recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,
                 recipient_agent,turn_gate
             ) SELECT rowid,delivery_id,issuer_scope,operation_kind,idempotency_key,
                 recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,
                 CASE WHEN json_valid(message_bytes)
                      THEN CASE WHEN json_type(message_bytes,'$.to') = 'text'
                                THEN json_extract(message_bytes,'$.to') ELSE '' END
                      ELSE '' END,
                 0
             FROM send_deliveries;
             DROP TABLE send_deliveries;
             ALTER TABLE send_deliveries_v7 RENAME TO send_deliveries;
             CREATE UNIQUE INDEX idx_send_deliveries_operation
                 ON send_deliveries(issuer_scope,operation_kind,idempotency_key);
             CREATE INDEX idx_send_deliveries_kind_key
                 ON send_deliveries(operation_kind,idempotency_key);
             CREATE INDEX idx_send_deliveries_recipient_phase
                 ON send_deliveries(recipient_instance_id,phase,queue_rank);
             CREATE INDEX idx_send_deliveries_agent_phase
                 ON send_deliveries(recipient_agent,phase,queue_rank);
             INSERT INTO send_delivery_execution_links (delivery_id,submission_id,execution_id)
                 SELECT delivery_id,submission_id,execution_id FROM temp.send_delivery_execution_links_v7_copy;
             DROP TABLE temp.send_delivery_execution_links_v7_copy;",
        )?;
            // Toute violation restante refuse l’ouverture ; le DROP et les copies seront annulés.
            let foreign_key_violation: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
                [],
                |row| row.get(0),
            )?;
            if foreign_key_violation {
                return Err(IdempotencyError::CorruptRecord(
                    "migration v7: violation de clé étrangère",
                ));
            }
            tx.execute(
                "INSERT INTO idempotency_schema_migrations(version) VALUES (7)",
                [],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// O(log r+c+B), c lignes CLI retenues. Rejeu/collision avant quota, COUNT et INSERT même TX.
    pub(crate) fn reserve_cli_wake(
        &mut self,
        id: &str,
        owner: &str,
        target: &bridget_transport::protocol::WakeTarget,
        label: &str,
        owner_project: Option<&bridget_transport::protocol::CommunicationProject>,
        now: i64,
    ) -> Result<(Reservation, crate::wake_schedules::FrozenWake), IdempotencyError> {
        use crate::wake_schedules::{CLI_WAKE_SCOPE, FrozenWake, WakePhase, WakeSource};
        let key = IdempotencyKey::new(CLI_WAKE_SCOPE, OperationKind::Send, id)?;
        if owner.is_empty() || label.len() > 2048 {
            return Err(IdempotencyError::InvalidDelivery);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(record) = load_record_from(&tx, &key)? {
            let mut frozen: FrozenWake = serde_json::from_slice(&record.canonical_bytes)
                .map_err(|_| IdempotencyError::CorruptRecord("canon wake CLI"))?;
            let record_issued_at:i64=tx.query_row("SELECT issued_at FROM idempotency_records WHERE issuer_scope=?1 AND operation_kind='send' AND idempotency_key=?2",params![CLI_WAKE_SCOPE,id],|r|r.get(0))?;
            if frozen.version != 1
                || frozen.source != WakeSource::CliOnce
                || frozen.id != id
                || frozen.issued_at != record_issued_at
            {
                return Err(IdempotencyError::CorruptRecord("domaine wake CLI"));
            }
            frozen.canonical_bytes = record.canonical_bytes.clone();
            let result = if record.expires_at <= now {
                Reservation::IdempotencyExpired
            } else if frozen.owner != owner || &frozen.target != target || frozen.label != label {
                Reservation::EnvelopeMismatch
            } else {
                Reservation::Replayed(record.lookup_result()?)
            };
            return Ok((result, frozen));
        }
        let count:i64=tx.query_row("SELECT count(*) FROM idempotency_records WHERE issuer_scope=?1 AND operation_kind='send' AND issued_at>?2",params![CLI_WAKE_SCOPE,now.saturating_sub(60)],|r|r.get(0))?;
        if count >= 60 {
            let oldest:i64=tx.query_row("SELECT min(issued_at) FROM idempotency_records WHERE issuer_scope=?1 AND operation_kind='send' AND issued_at>?2",params![CLI_WAKE_SCOPE,now.saturating_sub(60)],|r|r.get(0))?;
            return Err(IdempotencyError::CliWakeRateLimited {
                retry_after_secs: oldest.saturating_add(60).saturating_sub(now).max(1) as u64,
            });
        }
        let expires_at = now
            .checked_add(7 * 86400)
            .ok_or(IdempotencyError::InvalidHorizon)?;
        let mut frozen = FrozenWake {
            version: 1,
            source: WakeSource::CliOnce,
            target: target.clone(),
            owner: owner.into(),
            owner_project: owner_project.cloned(),
            owner_native: None,
            due_at: now,
            issued_at: now,
            grace_until: now
                .checked_add(600)
                .ok_or(IdempotencyError::InvalidIssuedAt)?,
            id: id.into(),
            label: label.into(),
            body: format!("Réveil déclenché par {owner}\n{label}"),
            canonical_bytes: Vec::new(),
            thread_operation_id: matches!(
                target,
                bridget_transport::protocol::WakeTarget::Thread { .. }
            )
            .then(|| uuid::Uuid::new_v4().to_string()),
            delivery_id: None,
            phase: WakePhase::Prepared,
        };
        frozen
            .freeze_canonical()
            .map_err(|_| IdempotencyError::InvalidDelivery)?;
        validate_canonical_bytes(&frozen.canonical_bytes)?;
        tx.execute("INSERT INTO idempotency_records(issuer_scope,operation_kind,idempotency_key,canonical_bytes,state,issued_at,expires_at) VALUES(?1,'send',?2,?3,'prepared',?4,?5)",params![CLI_WAKE_SCOPE,id,frozen.canonical_bytes,now,expires_at])?;
        tx.commit()?;
        Ok((Reservation::Prepared { expires_at }, frozen))
    }
    /// O(log r+B). Lecture fermée du seul domaine CLI, aucune nouvelle identité.
    pub(crate) fn cli_wake(
        &self,
        id: &str,
    ) -> Result<Option<(crate::wake_schedules::FrozenWake, RecordState, i64)>, IdempotencyError>
    {
        let key = IdempotencyKey::new(
            crate::wake_schedules::CLI_WAKE_SCOPE,
            OperationKind::Send,
            id,
        )?;
        let Some(record) = self.load_record(&key)? else {
            return Ok(None);
        };
        let (issued_at,): (i64,)=self.conn.query_row("SELECT issued_at FROM idempotency_records WHERE issuer_scope=?1 AND operation_kind='send' AND idempotency_key=?2",params![key.issuer_scope,id],|r|Ok((r.get(0)?,)))?;
        let mut frozen: crate::wake_schedules::FrozenWake =
            serde_json::from_slice(&record.canonical_bytes)
                .map_err(|_| IdempotencyError::CorruptRecord("canon wake CLI"))?;
        if frozen.version != 1
            || frozen.source != crate::wake_schedules::WakeSource::CliOnce
            || frozen.id != id
            || frozen.issued_at != issued_at
        {
            return Err(IdempotencyError::CorruptRecord("domaine wake CLI"));
        }
        frozen.canonical_bytes = record.canonical_bytes;
        Ok(Some((frozen, record.state, record.expires_at)))
    }
    /// O(c+128B), c lignes CLI retenues ; les lignes soldées sortent du scan ; aucun parsing de body.
    pub(crate) fn pending_cli_wakes(
        &self,
        now: i64,
        after: Option<(i64, String)>,
    ) -> Result<Vec<(crate::wake_schedules::FrozenWake, RecordState, i64)>, IdempotencyError> {
        let mut statement=self.conn.prepare("SELECT r.idempotency_key FROM idempotency_records r WHERE r.issuer_scope=?1 AND r.operation_kind='send' AND r.state IN ('prepared','dispatching') AND r.expires_at>?2 AND (?3 IS NULL OR r.issued_at>?3 OR (r.issued_at=?3 AND r.idempotency_key>?4)) ORDER BY r.issued_at,r.idempotency_key LIMIT 128")?;
        let (after_at, after_id) = after.map_or((None, None), |(at, id)| (Some(at), Some(id)));
        let ids = statement
            .query_map(
                params![
                    crate::wake_schedules::CLI_WAKE_SCOPE,
                    now,
                    after_at,
                    after_id
                ],
                |r| r.get::<_, String>(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        ids.into_iter()
            .map(|id| self.cli_wake(&id)?.ok_or(IdempotencyError::MissingRecord))
            .collect()
    }
    /// O(log d), preuve par scope interne de la remise, pas par texte cité.
    pub(crate) fn is_wake_delivery(&self, delivery_id: &str) -> Result<bool, IdempotencyError> {
        self.conn.query_row("SELECT EXISTS(SELECT 1 FROM send_deliveries WHERE delivery_id=?1 AND operation_kind='send' AND issuer_scope IN (?2,?3,?4))",params![delivery_id,crate::wake_schedules::CLI_WAKE_SCOPE,crate::wake_schedules::SCHEDULED_WAKE_SCOPE,crate::wake_schedules::MISSED_SUMMARY_SCOPE],|r|r.get(0)).map_err(Into::into)
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

    /// O(log n), journal de guidage seulement ; jamais un reçu de rejeu.
    pub(crate) fn record_guidance_event(
        &self,
        at: i64,
        actor: &str,
        target: &str,
        action: &str,
        path: &str,
        message_id: Option<&str>,
        outcome: &str,
    ) -> Result<(), IdempotencyError> {
        self.conn.execute(
            "INSERT INTO guidance_events(at,actor,target,action,path,message_id,outcome)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![at, actor, target, action, path, message_id, outcome],
        )?;
        Ok(())
    }

    /// O(log n+k), k événements de la fenêtre ; compteur et LAST admis du couple.
    /// La borne since est inclusive. La garde pure admet exactement last+30<=now.
    pub(crate) fn count_interrupts_pair(
        &self,
        actor: &str,
        target: &str,
        since: i64,
    ) -> Result<(u32, Option<i64>), IdempotencyError> {
        self.conn
            .query_row(
                "SELECT COUNT(*), MAX(at) FROM guidance_events
             WHERE actor=?1 AND actor!='human' AND target=?2 AND action='interrupt'
               AND outcome='admitted' AND at>=?3",
                params![actor, target, since],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(Into::into)
    }

    /// O(log n+k), compteur et THIRD_LATEST admis cible pour retry_after exact.
    /// Même si k>3, ce fait désigne le3e plus récent, pas MIN(toute la fenêtre).
    /// Aucun timestamp si moins de3 ; since inclusif, expiry third+600<=now.
    pub(crate) fn count_interrupts_target(
        &self,
        target: &str,
        since: i64,
    ) -> Result<(u32, Option<i64>), IdempotencyError> {
        self.conn
            .query_row(
                "SELECT COUNT(*),
                (SELECT at FROM guidance_events
                 WHERE target=?1 AND action='interrupt' AND actor!='human'
                   AND outcome='admitted' AND at>=?2
                 ORDER BY at DESC,id DESC LIMIT 1 OFFSET 2)
             FROM guidance_events
             WHERE target=?1 AND action='interrupt' AND actor!='human'
               AND outcome='admitted' AND at>=?2",
                params![target, since],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(Into::into)
    }

    /// O(n), purge24h du journal seul ; reçus idempotents et remises conservés.
    /// Le caller run_maintenance fournit son horloge commune.
    pub(crate) fn purge_guidance_events(&self, now: i64) -> Result<usize, IdempotencyError> {
        self.conn
            .execute(
                "DELETE FROM guidance_events WHERE at<?1",
                params![now.saturating_sub(86_400)],
            )
            .map_err(Into::into)
    }

    /// O(log E + K log E), E reçus, K suppressions indexées ; mémoire O(1).
    /// L'horizon exact reste rejouable ; maintenance horaire après sept jours.
    pub(crate) fn purge_queue_edits(&self, now: i64) -> Result<usize, IdempotencyError> {
        self.conn
            .execute("DELETE FROM queue_edits WHERE expires_at<?1", params![now])
            .map_err(Into::into)
    }

    pub fn purge_expired(&self, now: i64) -> Result<usize, IdempotencyError> {
        let records = self.conn.execute(
            "DELETE FROM idempotency_records WHERE expires_at <= ?1",
            params![now],
        )?;
        // Même relève horaire existante ; deux purges, sans atomicité métier promise.
        Ok(records + self.purge_queue_edits(now)?)
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
mod tests;

#[cfg(test)]
mod wake158b_tests {
    use super::*;
    use bridget_transport::protocol::WakeTarget;
    #[test]
    fn wake158B_groupe7_cli_quota_rejeu_collision_et_reprise() {
        let dir =
            std::env::temp_dir().join(format!("bridget-wake-cli158B-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(
            &dir,
            <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700),
        )
        .unwrap();
        let path = dir.join("cli.sqlite");
        let mut store = IdempotencyStore::open(&path).unwrap();
        let target = WakeTarget::Thread {
            id: uuid::Uuid::new_v4().to_string(),
        };
        let (result, first) = store
            .reserve_cli_wake("cli0", "owner", &target, "entier", None, 1000)
            .unwrap();
        assert!(matches!(result, Reservation::Prepared { .. }));
        assert!(first.thread_operation_id.is_some());
        assert_eq!(first.issued_at, 1000);
        for i in 1..60 {
            assert!(matches!(
                store
                    .reserve_cli_wake(&format!("cli{i}"), "owner", &target, "entier", None, 1000)
                    .unwrap()
                    .0,
                Reservation::Prepared { .. }
            ));
        }
        assert!(matches!(
            store.reserve_cli_wake("cli60", "owner", &target, "entier", None, 1000),
            Err(IdempotencyError::CliWakeRateLimited {
                retry_after_secs: 60
            })
        ));
        let (replay, original) = store
            .reserve_cli_wake("cli0", "owner", &target, "entier", None, 1001)
            .unwrap();
        assert!(matches!(replay, Reservation::Replayed(_)));
        assert_eq!(original, first);
        assert!(matches!(
            store
                .reserve_cli_wake("cli0", "owner", &target, "différent", None, 1001)
                .unwrap()
                .0,
            Reservation::EnvelopeMismatch
        ));
        assert_eq!(store.record_count().unwrap(), 60);
        drop(store);
        let mut store = IdempotencyStore::open(&path).unwrap();
        assert_eq!(store.cli_wake("cli0").unwrap().unwrap().0, first);
        assert_eq!(store.pending_cli_wakes(1001, None).unwrap().len(), 60);
        let key = IdempotencyKey::new(
            crate::wake_schedules::CLI_WAKE_SCOPE,
            OperationKind::Send,
            "cli0",
        )
        .unwrap();
        store
            .transition(&key, RecordState::Prepared, RecordState::Dispatching)
            .unwrap();
        store
            .finalize(
                &key,
                PublicResult::Accepted {
                    expires_at: 1000 + 7 * 86400,
                },
            )
            .unwrap();
        assert!(matches!(
            store
                .reserve_cli_wake("cli0", "owner", &target, "entier", None, 1002)
                .unwrap()
                .0,
            Reservation::Replayed(LookupResult::Accepted { .. })
        ));
        assert_eq!(store.pending_cli_wakes(1002, None).unwrap().len(), 59);
        assert!(matches!(
            store
                .reserve_cli_wake("cli60", "owner", &target, "entier", None, 1060)
                .unwrap()
                .0,
            Reservation::Prepared { .. }
        ));
        for i in 61..130 {
            store
                .reserve_cli_wake(
                    &format!("cli{i}"),
                    "owner",
                    &target,
                    "entier",
                    None,
                    1060 + ((i - 60) / 60) * 60,
                )
                .unwrap();
        }
        let page = store.pending_cli_wakes(1120, None).unwrap();
        assert_eq!(page.len(), 128);
        let last = &page.last().unwrap().0;
        let second = store
            .pending_cli_wakes(1120, Some((last.issued_at, last.id.clone())))
            .unwrap();
        assert_eq!(second.len(), 1);
        assert_ne!(page[0].0.id, second[0].0.id);
        assert!(matches!(
            store
                .reserve_cli_wake("cli0", "owner", &target, "entier", None, 1000 + 7 * 86400)
                .unwrap()
                .0,
            Reservation::IdempotencyExpired
        ));
        drop(store);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
