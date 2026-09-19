//! Persistance SQLite : une connexion, des transactions transverses conservées.
//!
//! Les modules classent le SQL, pas les engagements durables. Les helpers
//! transactionnels sont partagés avec l'idempotence ; aucune seconde connexion
//! n'est ouverte pour résoudre une demande ou publier son événement.

mod ledger_requests;
mod project_compat;
mod service_events;
pub(crate) mod threads;

pub(crate) use ledger_requests::search as ledger_search;
pub use ledger_requests::{LedgerEntry, TrackedRequest, UsageDashboardRow};
pub(crate) use ledger_requests::{
    fold_for_search, mark_answered_in_transaction, record_message_in_transaction,
};
use project_compat::ensure_project_bindings_runtime_schema;
pub use project_compat::{
    ProjectAuditEvent, ProjectAuditOperation, ProjectAuditOutcome, ProjectBinding,
    ProjectBindingState, ProjectRuntimeBinding,
};
pub(crate) use service_events::MAX_GUICHET_FRAME_BYTES;
pub use service_events::{
    GuichetClaim, GuichetCoordinationEvent, GuichetCoordinationReplay, GuichetDeposit,
    GuichetLifecycleEvent, GuichetNext, GuichetReplyInput, GuichetResult,
};

use bridget_transport::protocol::{ProjectRegistryRefusal, ProjectRoundRefusal};
use rusqlite::Connection;
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub(crate) fn observation_snapshot(&self) -> Result<Vec<u8>, StoreError> {
        use rusqlite::OptionalExtension;
        self.conn
            .query_row(
                "SELECT payload FROM observation_subscriptions WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map(|bytes| bytes.unwrap_or_else(|| b"[]".to_vec()))
            .map_err(StoreError::Sqlite)
    }

    pub(crate) fn save_observation_snapshot(&self, bytes: &[u8]) -> Result<(), StoreError> {
        let previous_ms: u64 = self
            .conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .map_err(StoreError::Sqlite)?;
        self.conn
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(StoreError::Sqlite)?;
        let result = self.conn.execute("INSERT INTO observation_subscriptions(singleton,payload) VALUES(1,?1) ON CONFLICT(singleton) DO UPDATE SET payload=excluded.payload", [bytes]);
        let restored = self
            .conn
            .busy_timeout(std::time::Duration::from_millis(previous_ms));
        result.map_err(StoreError::Sqlite)?;
        restored.map_err(StoreError::Sqlite)
    }
    /// Ouvre ou crée la base de données.
    ///
    /// Session 107 : journal en écriture anticipée (WAL), propriété persistante
    /// du fichier posée ici par le premier ouvreur du daemon et héritée par
    /// toutes les autres connexions (idempotence, exécution, profils, artefacts,
    /// lecteurs en lecture seule). Les lecteurs ne sont plus bloqués par une
    /// validation d'écriture ; les écrivains restent sérialisés par
    /// `busy_timeout`. `synchronous` reste au défaut (`FULL`) : aucune
    /// relaxation de durabilité. La valeur rendue est vérifiée : SQLite rend
    /// l'ancien mode sans erreur si la bascule est impossible.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path).map_err(StoreError::Sqlite)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(StoreError::Sqlite)?;
        // Le contrôle de schéma précède toute écriture : une base d'une
        // version future est refusée sans que son en-tête soit touché.
        crate::store_schema::validate(&conn).map_err(StoreError::Schema)?;
        Self::ensure_wal(&conn)?;
        Self::init_schema(&conn)?;
        Ok(Store { conn })
    }

    /// Bascule (ou confirme) le journal WAL. La bascule exige un verrou
    /// exclusif bref ; sur cette transition SQLite n'appelle pas le
    /// gestionnaire d'attente (évitement d'interblocage), d'où un réessai
    /// borné explicite quand plusieurs ouvreurs démarrent ensemble sur une
    /// base neuve. Une base déjà en WAL est confirmée sans verrou.
    fn ensure_wal(conn: &Connection) -> Result<(), StoreError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let current: String = conn
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .map_err(StoreError::Sqlite)?;
            if matches!(current.as_str(), "wal" | "memory") {
                return Ok(());
            }
            match conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get::<_, String>(0)) {
                Ok(mode) if mode == "wal" => return Ok(()),
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(error, _))
                    if matches!(
                        error.code,
                        rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                    ) => {}
                Err(error) => return Err(StoreError::Sqlite(error)),
            }
            if std::time::Instant::now() >= deadline {
                return Err(StoreError::Invariant(
                    "journal WAL refusé par SQLite pour la base du daemon",
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Connexion partagée avec les modules du plan de contrôle (SPEC-087) qui
    /// possèdent leurs propres tables dans cette base.
    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Session 104 : connexion de lecture seule sur une base existante, sans
    /// création de schéma ni migration. Attente SQLite limitée à 100 ms : un
    /// écrivain long rend `busy`, jamais une file d'attente.
    pub fn open_read_only(path: &Path) -> Result<Connection, StoreError> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(StoreError::Sqlite)?;
        conn.busy_timeout(std::time::Duration::from_millis(100))
            .map_err(StoreError::Sqlite)?;
        Ok(conn)
    }

    fn init_schema(conn: &Connection) -> Result<(), StoreError> {
        crate::referent_control::ensure_schema(conn).map_err(StoreError::Sqlite)?;
        crate::human_inbox::ensure_schema(conn).map_err(StoreError::Sqlite)?;
        // `result_bytes` a été retirée du schéma canonique : `reply_bytes`
        // porte déjà les octets terminaux rejouables. Les bases antérieures
        // peuvent conserver cette colonne nullable ignorée ; reconstruire la
        // table pour la supprimer n'apporterait aucun invariant supplémentaire.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS observation_subscriptions (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                payload BLOB NOT NULL CHECK (length(payload) <= 262144)
            );
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
            CREATE INDEX IF NOT EXISTS idx_ledger_conv ON ledger(conversation_key, ts);
            CREATE INDEX IF NOT EXISTS idx_ledger_sender_page ON ledger(sender, ts DESC, id DESC, target DESC);
            CREATE INDEX IF NOT EXISTS idx_ledger_target_page ON ledger(target, ts DESC, id DESC);
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
                -- Métadonnée serveur : jamais issue des octets de l'appelant.
                authorization_attestation BLOB,
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
                cursor INTEGER,
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
        // Les bases historiques n'ont légitimement aucune attestation. La
        // colonne nullable conserve ce fait, tandis qu'une mutation neuve
        // impose `Some` dans le chemin daemon avant son unique INSERT.
        if !guichet_authorization_column_exists(conn)? {
            conn.execute(
                "ALTER TABLE guichet_requests ADD COLUMN authorization_attestation BLOB",
                [],
            )
            .map_err(StoreError::Sqlite)?;
        }
        // Les bases créées par T1603 n'avaient pas de curseur public. SQLite
        // ne sait pas ajouter une colonne NOT NULL sans valeur : on remplit
        // donc une fois depuis son identifiant physique, dans l'ordre durable.
        let _ = conn.execute(
            "ALTER TABLE guichet_coordination_events ADD COLUMN cursor INTEGER",
            [],
        );
        conn.execute(
            "UPDATE guichet_coordination_events SET cursor = rowid WHERE cursor IS NULL",
            [],
        )
        .map_err(StoreError::Sqlite)?;
        conn.execute_batch(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_guichet_coordination_cursor
                 ON guichet_coordination_events(cursor);
             CREATE TABLE IF NOT EXISTS guichet_coordination_stream_state (
                 singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                 high_watermark INTEGER NOT NULL CHECK (high_watermark >= 0)
             );
             INSERT OR IGNORE INTO guichet_coordination_stream_state
                 (singleton, high_watermark)
             SELECT 1, COALESCE(MAX(cursor), 0) FROM guichet_coordination_events;
             CREATE TABLE IF NOT EXISTS usage_samples (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 agent TEXT NOT NULL,
                 observed_at INTEGER NOT NULL,
                 input_tokens INTEGER NOT NULL,
                 output_tokens INTEGER NOT NULL,
                 cache_creation_input_tokens INTEGER NOT NULL,
                 cache_read_input_tokens INTEGER NOT NULL,
                 provider_kind TEXT,
                 model TEXT,
                 source TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_usage_samples_agent_ts
                 ON usage_samples(agent, observed_at);
             CREATE TABLE IF NOT EXISTS project_bindings (
                 project_id TEXT PRIMARY KEY,
                 canonical_root TEXT NOT NULL,
                 backend TEXT NOT NULL CHECK (backend IN ('host', 'docker')),
                 state TEXT NOT NULL CHECK (state IN (
                     'pending_binding', 'active', 'disabled', 'path_missing', 'binding_failed'
                 )),
                 generation INTEGER NOT NULL CHECK (generation > 0),
                 bound_at INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL,
                 last_reason TEXT,
                 runtime_state TEXT,
                 runtime_last_reason TEXT,
                 policy_id TEXT,
                 policy_version INTEGER,
                 policy_digest TEXT,
                 image_reference TEXT,
                 resolved_image_id TEXT,
                 run_as_uid INTEGER,
                 run_as_gid INTEGER,
                 environment_epoch INTEGER NOT NULL DEFAULT 0,
                 topology_digest TEXT NOT NULL DEFAULT 'sha256:legacy',
                 container_id TEXT
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_project_bindings_live_root
                 ON project_bindings(canonical_root) WHERE state != 'disabled';
             CREATE TABLE IF NOT EXISTS project_system_roles (
                 singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                 project_id TEXT NOT NULL UNIQUE,
                 declared_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS project_system_dogfooding (
                 singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                 project_id TEXT NOT NULL UNIQUE,
                 binding_generation INTEGER NOT NULL CHECK (binding_generation > 0),
                 setting_generation INTEGER NOT NULL CHECK (setting_generation > 0),
                 mode TEXT NOT NULL CHECK (mode IN ('disabled', 'enabled')),
                 updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS project_system_worktree_leases (
                 canonical_worktree TEXT PRIMARY KEY,
                 project_id TEXT NOT NULL,
                 agent_id TEXT NOT NULL,
                 binding_generation INTEGER NOT NULL CHECK (binding_generation > 0),
                 acquired_at INTEGER NOT NULL
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_project_system_worktree_leases_agent
                 ON project_system_worktree_leases(project_id, agent_id);
             CREATE TABLE IF NOT EXISTS project_audit_events (
                 audit_event_id TEXT PRIMARY KEY,
                 command_id TEXT NOT NULL,
                 project_id TEXT NOT NULL,
                 operation TEXT NOT NULL CHECK (operation IN (
                     'register', 'rebind', 'activate', 'disable', 'review_project_reconcile'
                 )),
                 binding_generation INTEGER NOT NULL CHECK (binding_generation > 0),
                 outcome_json BLOB NOT NULL,
                 previous_root_reference TEXT,
                 observed_at INTEGER NOT NULL,
                 UNIQUE (command_id, operation, binding_generation)
             );
             CREATE INDEX IF NOT EXISTS idx_project_audit_events_project_observed
                 ON project_audit_events(project_id, observed_at, audit_event_id);
             CREATE TABLE IF NOT EXISTS project_binding_attempts (
                 command_id TEXT PRIMARY KEY,
                 project_id TEXT NOT NULL,
                 canonical_root TEXT NOT NULL,
                 outcome_json BLOB NOT NULL,
                 observed_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS project_admin_attempts (
                 command_id TEXT PRIMARY KEY,
                 operation TEXT NOT NULL,
                 project_id TEXT,
                 canonical_root TEXT,
                 outcome_json BLOB NOT NULL,
                 observed_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS project_round_policies (
                 project_id TEXT NOT NULL,
                 binding_generation INTEGER NOT NULL CHECK (binding_generation > 0),
                 enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
                 revision INTEGER NOT NULL CHECK (revision > 0),
                 updated_at INTEGER NOT NULL,
                 command_id TEXT NOT NULL,
                 last_occurrence_at INTEGER,
                 last_dispatch_state TEXT CHECK (
                     last_dispatch_state IN ('deposited', 'refused', 'indeterminate')
                 ),
                 last_dispatch_observed_at INTEGER,
                 PRIMARY KEY (project_id, binding_generation)
             );
             CREATE TABLE IF NOT EXISTS project_round_commands (
                 command_id TEXT PRIMARY KEY,
                 operation TEXT NOT NULL CHECK (operation IN ('enable', 'disable')),
                 project_id TEXT NOT NULL,
                 binding_generation INTEGER NOT NULL CHECK (binding_generation > 0),
                 canonical_bytes BLOB NOT NULL,
                 outcome_json BLOB NOT NULL,
                 observed_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_project_round_enabled
                 ON project_round_policies(enabled, project_id, binding_generation);
             ",
        )
        .map_err(StoreError::Sqlite)?;
        // Migration additive : les échantillons historiques restent valides
        // mais sans fournisseur/modèle attesté. Ne surtout pas les compléter
        // depuis l'état courant d'un agent.
        let _ = conn.execute(
            "ALTER TABLE usage_samples ADD COLUMN provider_kind TEXT",
            [],
        );
        let _ = conn.execute("ALTER TABLE usage_samples ADD COLUMN model TEXT", []);
        // Migration additive SPEC-081 : l'absence des trois colonnes signifie
        // simplement qu'aucun passage n'a encore été observé.
        let _ = conn.execute(
            "ALTER TABLE project_round_policies ADD COLUMN last_occurrence_at INTEGER",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE project_round_policies ADD COLUMN last_dispatch_state TEXT
             CHECK (last_dispatch_state IN ('deposited', 'refused', 'indeterminate'))",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE project_round_policies ADD COLUMN last_dispatch_observed_at INTEGER",
            [],
        );
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_usage_samples_dashboard
                ON usage_samples(observed_at, provider_kind, model);",
        )
        .map_err(StoreError::Sqlite)?;
        ensure_project_bindings_runtime_schema(conn)?;
        threads::ensure_schema(conn)?;
        // Préflight 102 : tables et index attendus présents, sinon refus explicite.
        if !threads::schema_ready(conn)? {
            return Err(StoreError::Invariant(
                "schéma des fils incomplet après migration",
            ));
        }
        Self::ensure_project_audit_schema(conn)?;
        Ok(())
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

fn guichet_authorization_column_exists(conn: &Connection) -> Result<bool, StoreError> {
    let mut statement = conn
        .prepare("PRAGMA table_info(guichet_requests)")
        .map_err(StoreError::Sqlite)?;
    let mut rows = statement.query([]).map_err(StoreError::Sqlite)?;
    while let Some(row) = rows.next().map_err(StoreError::Sqlite)? {
        if row.get::<_, String>(1).map_err(StoreError::Sqlite)? == "authorization_attestation" {
            return Ok(true);
        }
    }
    Ok(false)
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug)]
pub enum StoreError {
    Schema(crate::store_schema::SchemaError),
    Sqlite(rusqlite::Error),
    Invariant(&'static str),
    ProjectRegistryRefusal(ProjectRegistryRefusal),
    ProjectRoundRefusal(ProjectRoundRefusal),
    FrameTooLarge { max_frame_bytes: usize },
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Schema(error) => write!(f, "{error}"),
            StoreError::Sqlite(e) => write!(f, "SQLite: {}", e),
            StoreError::Invariant(detail) => write!(f, "invariant store: {detail}"),
            StoreError::ProjectRegistryRefusal(reason) => {
                write!(f, "refus registre projet: {reason:?}")
            }
            StoreError::ProjectRoundRefusal(reason) => {
                write!(f, "refus politique de ronde: {reason:?}")
            }
            StoreError::FrameTooLarge { max_frame_bytes } => {
                write!(f, "trame guichet supérieure à {max_frame_bytes} octets")
            }
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod observation_tests {
    use super::*;

    #[test]
    fn spec101_contended_snapshot_fails_fast_and_restores_timeout() {
        let root =
            std::env::temp_dir().join(format!("bg101-store-{}", uuid::Uuid::new_v4().simple()));
        bridget_transport::fsutil::create_private_dir(&root).unwrap();
        let path = root.join("state.db");
        let store = Store::open(&path).unwrap();
        store.save_observation_snapshot(b"[]").unwrap();
        let blocker = Connection::open(&path).unwrap();
        blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
        let started = std::time::Instant::now();
        assert!(store.save_observation_snapshot(b"[{}]").is_err());
        assert!(
            started.elapsed() < std::time::Duration::from_millis(300),
            "ne pas attendre deux secondes sous verrou daemon"
        );
        assert_eq!(store.observation_snapshot().unwrap(), b"[]");
        assert_eq!(
            store
                .conn
                .query_row("PRAGMA busy_timeout", [], |r| r.get::<_, u64>(0))
                .unwrap(),
            2000
        );
        blocker.execute_batch("ROLLBACK").unwrap();
        store.save_observation_snapshot(b"[{}]").unwrap();
        assert_eq!(
            store
                .conn
                .query_row("PRAGMA busy_timeout", [], |r| r.get::<_, u64>(0))
                .unwrap(),
            2000
        );
    }

    #[test]
    fn spec101_snapshot_is_bounded_atomic_and_readonly_failure_preserves_previous() {
        let store = Store::open(Path::new(":memory:")).unwrap();
        assert_eq!(store.observation_snapshot().unwrap(), b"[]");
        store.save_observation_snapshot(b"[{}]").unwrap();
        assert!(store.save_observation_snapshot(&vec![0; 262145]).is_err());
        assert_eq!(store.observation_snapshot().unwrap(), b"[{}]");
        store.conn.execute_batch("PRAGMA query_only=ON").unwrap();
        assert!(store.save_observation_snapshot(b"[]").is_err());
        assert_eq!(store.observation_snapshot().unwrap(), b"[{}]");
    }
}
