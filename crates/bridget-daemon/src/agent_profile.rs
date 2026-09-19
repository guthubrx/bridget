//! Profils d'agents partagés par le relais, séparés du routage technique.

use crate::store::fold_for_search;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

pub const MAX_DISPLAY_NAME_CHARS: usize = 80;
pub const MAX_LABEL_CHARS: usize = 32;
pub const MAX_LABELS: usize = 12;
pub const MAX_INSTRUCTIONS_CHARS: usize = 8_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDisplayName {
    pub agent_id: String,
    pub display_name: String,
    pub revision: u64,
}

const AVATAR_SHAPES: &[&str] = &[
    "round",
    "soft-square",
    "pill",
    "triangle",
    "hexagon",
    "cloud",
    "drop",
    "pebble",
];
const AVATAR_COLORS: &[&str] = &[
    "white", "brown", "red", "orange", "amber", "green", "teal", "blue", "purple", "pink", "gray",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProfileSummary {
    pub agent_id: String,
    pub display_name: String,
    pub labels: Vec<String>,
    pub avatar_shape: String,
    pub avatar_color: String,
    pub revision: u64,
    pub instructions_revision: u64,
    pub instruction_status: InstructionStatus,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProfileDetail {
    pub summary: AgentProfileSummary,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProfileUpdate {
    pub expected_revision: u64,
    pub display_name: String,
    pub labels: Vec<String>,
    pub avatar_shape: String,
    pub avatar_color: String,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingProfileInstructions {
    pub agent_id: String,
    pub revision: u64,
    pub instructions: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionEventType {
    HumanInputNeeded,
    TaskCompleted,
    TerminalFailure,
}

impl AttentionEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HumanInputNeeded => "human_input_needed",
            Self::TaskCompleted => "task_completed",
            Self::TerminalFailure => "terminal_failure",
        }
    }

    fn from_db(value: &str) -> Result<Self, AgentProfileError> {
        match value {
            "human_input_needed" => Ok(Self::HumanInputNeeded),
            "task_completed" => Ok(Self::TaskCompleted),
            "terminal_failure" => Ok(Self::TerminalFailure),
            _ => Err(AgentProfileError::Invalid("type d'attention invalide")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionEvent {
    pub event_id: String,
    pub agent_id: String,
    pub display_name: String,
    pub event_type: AttentionEventType,
    pub created_at: i64,
    pub seen: bool,
    pub native_notified: bool,
    pub attention_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientNotificationPreference {
    pub agent_id: String,
    pub human_input_needed: bool,
    pub task_completed: bool,
    pub terminal_failure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionStatus {
    PendingRestart,
    Applied,
    Unsupported,
    Failed,
}

impl InstructionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PendingRestart => "pending_restart",
            Self::Applied => "applied",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
        }
    }

    fn from_db(value: &str) -> Self {
        match value {
            "applied" => Self::Applied,
            "unsupported" => Self::Unsupported,
            "failed" => Self::Failed,
            _ => Self::PendingRestart,
        }
    }
}

#[derive(Debug)]
pub enum AgentProfileError {
    Sqlite(rusqlite::Error),
    Invalid(&'static str),
    NotFound,
    RevisionConflict,
    DisplayNameConflict,
}

impl std::fmt::Display for AgentProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(error) => write!(formatter, "SQLite profil: {error}"),
            Self::Invalid(reason) => write!(formatter, "profil invalide: {reason}"),
            Self::NotFound => write!(formatter, "profil introuvable"),
            Self::RevisionConflict => write!(formatter, "profil modifié entre-temps"),
            Self::DisplayNameConflict => write!(formatter, "nom affiché déjà utilisé"),
        }
    }
}

impl std::error::Error for AgentProfileError {}

#[derive(Debug)]
pub struct AgentProfileStore {
    conn: Connection,
}

impl AgentProfileStore {
    pub fn open(path: &Path) -> Result<Self, AgentProfileError> {
        let conn = Connection::open(path).map_err(AgentProfileError::Sqlite)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(AgentProfileError::Sqlite)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS agent_identities (
                 agent_id TEXT PRIMARY KEY,
                 created_at INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS agent_profiles (
                 agent_id TEXT PRIMARY KEY REFERENCES agent_identities(agent_id),
                 display_name TEXT NOT NULL,
                 display_name_normalized TEXT NOT NULL UNIQUE,
                 avatar_shape TEXT NOT NULL,
                 avatar_color TEXT NOT NULL,
                 instructions TEXT NOT NULL DEFAULT '',
                 instructions_revision INTEGER NOT NULL DEFAULT 1 CHECK (instructions_revision > 0),
                 revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
                 updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS agent_profile_labels (
                 agent_id TEXT NOT NULL REFERENCES agent_profiles(agent_id),
                 position INTEGER NOT NULL CHECK (position >= 0),
                 label TEXT NOT NULL,
                 normalized_label TEXT NOT NULL,
                 PRIMARY KEY(agent_id, normalized_label),
                 UNIQUE(agent_id, position)
             );
             CREATE TABLE IF NOT EXISTS agent_profile_applications (
                 agent_id TEXT PRIMARY KEY REFERENCES agent_profiles(agent_id),
                 provider_spawn_id TEXT,
                 instructions_revision INTEGER NOT NULL CHECK (instructions_revision > 0),
                 status TEXT NOT NULL CHECK (status IN ('pending_restart', 'applied', 'unsupported', 'failed')),
                 observed_at INTEGER NOT NULL,
                 diagnostic_code TEXT
             );
             CREATE TABLE IF NOT EXISTS attention_events (
                 event_id TEXT PRIMARY KEY,
                 occurrence_key TEXT NOT NULL UNIQUE,
                 agent_id TEXT NOT NULL REFERENCES agent_identities(agent_id),
                 event_type TEXT NOT NULL CHECK (event_type IN ('human_input_needed', 'task_completed', 'terminal_failure')),
                 source_message_id TEXT,
                 summary TEXT NOT NULL,
                 created_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_attention_events_created
                 ON attention_events(created_at, event_id);
             CREATE TABLE IF NOT EXISTS client_notification_preferences (
                 client_id TEXT NOT NULL,
                 agent_id TEXT NOT NULL REFERENCES agent_identities(agent_id),
                 human_input_needed INTEGER NOT NULL DEFAULT 0 CHECK (human_input_needed IN (0, 1)),
                 task_completed INTEGER NOT NULL DEFAULT 0 CHECK (task_completed IN (0, 1)),
                 terminal_failure INTEGER NOT NULL DEFAULT 0 CHECK (terminal_failure IN (0, 1)),
                 updated_at INTEGER NOT NULL,
                 PRIMARY KEY(client_id, agent_id)
             );
             CREATE TABLE IF NOT EXISTS client_attention_state (
                 client_id TEXT NOT NULL,
                 event_id TEXT NOT NULL REFERENCES attention_events(event_id),
                 seen_at INTEGER,
                 native_notified_at INTEGER,
                 PRIMARY KEY(client_id, event_id)
             );",
        )
        .map_err(AgentProfileError::Sqlite)?;
        Ok(Self { conn })
    }

    fn table_exists(&self, table: &str) -> Result<bool, AgentProfileError> {
        self.conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |_| Ok(()),
            )
            .optional()
            .map(|row| row.is_some())
            .map_err(AgentProfileError::Sqlite)
    }

    fn has_legacy_identity_schema(&self) -> Result<bool, AgentProfileError> {
        let mut statement = self
            .conn
            .prepare("PRAGMA table_info(agent_identities)")
            .map_err(AgentProfileError::Sqlite)?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(AgentProfileError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AgentProfileError::Sqlite)?;
        Ok(columns
            .iter()
            .any(|column| column == "current_routing_name"))
    }

    fn legacy_alias_table(&self, create: bool) -> Result<Option<&'static str>, AgentProfileError> {
        if self.table_exists("agent_routing_aliases")? {
            return Ok(Some("agent_routing_aliases"));
        }
        if create {
            self.conn
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS identity_migration_aliases (
                     routing_name TEXT PRIMARY KEY,
                     agent_id TEXT NOT NULL REFERENCES agent_identities(agent_id),
                     observed_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_identity_migration_aliases_agent
                     ON identity_migration_aliases(agent_id);",
                )
                .map_err(AgentProfileError::Sqlite)?;
            return Ok(Some("identity_migration_aliases"));
        }
        if self.table_exists("identity_migration_aliases")? {
            Ok(Some("identity_migration_aliases"))
        } else {
            Ok(None)
        }
    }

    pub fn ensure_routing_names<I>(&mut self, names: I) -> Result<(), AgentProfileError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let now = now_secs();
        let mut seen = HashSet::new();
        let names = names
            .into_iter()
            .map(|name| name.as_ref().trim().to_string())
            .filter(|name| !name.is_empty() && seen.insert(name.clone()))
            .collect::<Vec<_>>();
        if names.is_empty() {
            return Ok(());
        }
        let alias_table = self
            .legacy_alias_table(true)?
            .ok_or(AgentProfileError::Invalid("table de migration absente"))?;
        let legacy_identity_schema = self.has_legacy_identity_schema()?;
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        for routing_name in names {
            let existing = transaction
                .query_row(
                    &format!("SELECT agent_id FROM {alias_table} WHERE routing_name = ?1"),
                    [&routing_name],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(AgentProfileError::Sqlite)?;
            if existing.is_some() {
                continue;
            }
            let agent_id = Uuid::new_v4().to_string();
            let display_name = available_display_name(&transaction, "Agent")?;
            let normalized = normalize_display_name(&display_name)?;
            if legacy_identity_schema {
                transaction
                    .execute(
                        "INSERT INTO agent_identities(agent_id, current_routing_name, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?3)",
                        params![agent_id, routing_name, now],
                    )
                    .map_err(AgentProfileError::Sqlite)?;
            } else {
                transaction
                    .execute(
                        "INSERT INTO agent_identities(agent_id, created_at, updated_at) VALUES (?1, ?2, ?2)",
                        params![agent_id, now],
                    )
                    .map_err(AgentProfileError::Sqlite)?;
            }
            if alias_table == "agent_routing_aliases" {
                transaction
                    .execute(
                        "INSERT INTO agent_routing_aliases(routing_name, agent_id, is_current, observed_at)
                         VALUES (?1, ?2, 1, ?3)",
                        params![routing_name, agent_id, now],
                    )
                    .map_err(AgentProfileError::Sqlite)?;
            } else {
                transaction
                    .execute(
                        "INSERT INTO identity_migration_aliases(routing_name, agent_id, observed_at)
                         VALUES (?1, ?2, ?3)",
                        params![routing_name, agent_id, now],
                    )
                    .map_err(AgentProfileError::Sqlite)?;
            }
            transaction
                .execute(
                    "INSERT INTO agent_profiles(
                         agent_id, display_name, display_name_normalized, avatar_shape,
                         avatar_color, instructions, instructions_revision, revision, updated_at
                     ) VALUES (?1, ?2, ?3, 'round', 'blue', '', 1, 1, ?4)",
                    params![agent_id, display_name, normalized, now],
                )
                .map_err(AgentProfileError::Sqlite)?;
            transaction
                .execute(
                    "INSERT INTO agent_profile_applications(
                         agent_id, provider_spawn_id, instructions_revision, status, observed_at, diagnostic_code
                     ) VALUES (?1, NULL, 1, 'applied', ?2, NULL)",
                    params![agent_id, now],
                )
                .map_err(AgentProfileError::Sqlite)?;
        }
        transaction.commit().map_err(AgentProfileError::Sqlite)
    }

    /// Crée les profils des identifiants opaques déjà établis. Cette voie ne
    /// crée aucun alias ni nom de routage lisible.
    pub fn ensure_agent_ids<I>(&mut self, agent_ids: I) -> Result<(), AgentProfileError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let now = now_secs();
        let legacy_identity_schema = self.has_legacy_identity_schema()?;
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        for agent_id in agent_ids {
            let agent_id = agent_id.as_ref().trim();
            if agent_id.is_empty() {
                continue;
            }
            let exists = transaction
                .query_row(
                    "SELECT 1 FROM agent_identities WHERE agent_id = ?1",
                    [agent_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(AgentProfileError::Sqlite)?
                .is_some();
            if exists {
                continue;
            }
            let display_name = available_display_name(&transaction, "Agent")?;
            let normalized = normalize_display_name(&display_name)?;
            if legacy_identity_schema {
                transaction.execute(
                    "INSERT INTO agent_identities(agent_id, current_routing_name, created_at, updated_at) VALUES (?1, ?1, ?2, ?2)",
                    params![agent_id, now],
                ).map_err(AgentProfileError::Sqlite)?;
            } else {
                transaction.execute(
                    "INSERT INTO agent_identities(agent_id, created_at, updated_at) VALUES (?1, ?2, ?2)",
                    params![agent_id, now],
                ).map_err(AgentProfileError::Sqlite)?;
            }
            transaction.execute(
                "INSERT INTO agent_profiles(agent_id, display_name, display_name_normalized, avatar_shape, avatar_color, instructions, instructions_revision, revision, updated_at) VALUES (?1, ?2, ?3, 'round', 'blue', '', 1, 1, ?4)",
                params![agent_id, display_name, normalized, now],
            ).map_err(AgentProfileError::Sqlite)?;
            transaction.execute(
                "INSERT INTO agent_profile_applications(agent_id, provider_spawn_id, instructions_revision, status, observed_at, diagnostic_code) VALUES (?1, NULL, 1, 'applied', ?2, NULL)",
                params![agent_id, now],
            ).map_err(AgentProfileError::Sqlite)?;
        }
        transaction.commit().map_err(AgentProfileError::Sqlite)
    }

    /// Lit la correspondance de migration, sans rien modifier. Les clés sont des
    /// routes historiques et les valeurs les UUID déjà attribués aux profils.
    pub fn legacy_routing_map(
        &self,
    ) -> Result<std::collections::BTreeMap<String, String>, AgentProfileError> {
        let Some(alias_table) = self.legacy_alias_table(false)? else {
            return Ok(std::collections::BTreeMap::new());
        };
        let mut statement = self
            .conn
            .prepare(&format!(
                "SELECT routing_name, agent_id FROM {alias_table} ORDER BY routing_name"
            ))
            .map_err(AgentProfileError::Sqlite)?;
        statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(AgentProfileError::Sqlite)?
            .collect::<Result<std::collections::BTreeMap<_, _>, _>>()
            .map_err(AgentProfileError::Sqlite)
    }

    /// Supprime réellement le schéma transitoire de routage après que tous les
    /// stores ont été réécrits. Les identités restantes ne contiennent plus
    /// aucune colonne ni table permettant de retrouver un ancien nom.
    pub fn retire_legacy_routing_aliases(&mut self) -> Result<(), AgentProfileError> {
        let has_legacy_identities = self.has_legacy_identity_schema()?;
        let has_historical_aliases = self.table_exists("agent_routing_aliases")?;
        let has_temporary_aliases = self.table_exists("identity_migration_aliases")?;
        if !has_legacy_identities && !has_historical_aliases && !has_temporary_aliases {
            return Ok(());
        }

        // SQLite ne sait pas retirer une colonne couverte par une contrainte
        // UNIQUE. On reconstruit donc exclusivement la table d'identités,
        // sans jamais renommer l'ancienne table : les clés étrangères des
        // tables de profils restent ainsi reliées au même nom final.
        self.conn
            .execute_batch(
                "PRAGMA foreign_keys = OFF;
             PRAGMA legacy_alter_table = ON;
             BEGIN IMMEDIATE;
             CREATE TABLE IF NOT EXISTS agent_identities_v2 (
                 agent_id TEXT PRIMARY KEY,
                 created_at INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL
             );
             INSERT OR IGNORE INTO agent_identities_v2(agent_id, created_at, updated_at)
                 SELECT agent_id, created_at, updated_at FROM agent_identities;",
            )
            .map_err(AgentProfileError::Sqlite)?;
        if has_historical_aliases {
            self.conn
                .execute("DROP TABLE agent_routing_aliases", [])
                .map_err(AgentProfileError::Sqlite)?;
        }
        if has_temporary_aliases {
            self.conn
                .execute("DROP TABLE identity_migration_aliases", [])
                .map_err(AgentProfileError::Sqlite)?;
        }
        if has_legacy_identities {
            self.conn
                .execute("DROP TABLE agent_identities", [])
                .map_err(AgentProfileError::Sqlite)?;
            self.conn
                .execute(
                    "ALTER TABLE agent_identities_v2 RENAME TO agent_identities",
                    [],
                )
                .map_err(AgentProfileError::Sqlite)?;
        } else {
            self.conn
                .execute("DROP TABLE agent_identities_v2", [])
                .map_err(AgentProfileError::Sqlite)?;
        }
        self.conn
            .execute_batch("COMMIT; PRAGMA foreign_keys = ON; PRAGMA legacy_alter_table = OFF;")
            .map_err(AgentProfileError::Sqlite)?;
        let foreign_key_issue = self
            .conn
            .query_row("PRAGMA foreign_key_check", [], |_| Ok(()))
            .optional()
            .map_err(AgentProfileError::Sqlite)?;
        if foreign_key_issue.is_some() {
            return Err(AgentProfileError::Invalid(
                "intégrité des profils après migration",
            ));
        }
        Ok(())
    }

    pub fn profile_for_agent_id(
        &self,
        agent_id: &str,
    ) -> Result<Option<AgentProfileSummary>, AgentProfileError> {
        let row = self.conn.query_row(
            "SELECT profile.agent_id, profile.display_name, profile.avatar_shape, profile.avatar_color, profile.revision, profile.instructions_revision, application.status, profile.updated_at FROM agent_profiles profile LEFT JOIN agent_profile_applications application ON application.agent_id = profile.agent_id WHERE profile.agent_id = ?1",
            [agent_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?, row.get::<_, i64>(5)?, row.get::<_, Option<String>>(6)?, row.get::<_, i64>(7)?)),
        ).optional().map_err(AgentProfileError::Sqlite)?;
        row.map(|row| self.summary_from_row(row)).transpose()
    }

    pub fn import_historical_routing_names(&mut self) -> Result<(), AgentProfileError> {
        let names = {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT sender AS routing_name FROM ledger
                     UNION
                     SELECT target AS routing_name FROM ledger",
                )
                .map_err(AgentProfileError::Sqlite)?;
            statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(AgentProfileError::Sqlite)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(AgentProfileError::Sqlite)?
        };
        self.ensure_routing_names(names)
    }

    pub fn profile_for_routing_name(
        &self,
        routing_name: &str,
    ) -> Result<Option<AgentProfileSummary>, AgentProfileError> {
        let Some(alias_table) = self.legacy_alias_table(false)? else {
            return Ok(None);
        };
        let row = self
            .conn
            .query_row(
                &format!("SELECT identity.agent_id, profile.display_name, profile.avatar_shape, profile.avatar_color,
                        profile.revision, profile.instructions_revision, application.status, profile.updated_at
                 FROM {alias_table} alias
                 JOIN agent_identities identity ON identity.agent_id = alias.agent_id
                 JOIN agent_profiles profile ON profile.agent_id = identity.agent_id
                 LEFT JOIN agent_profile_applications application ON application.agent_id = identity.agent_id
                 WHERE alias.routing_name = ?1"),
                [routing_name],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?;
        row.map(|row| self.summary_from_row(row)).transpose()
    }

    pub fn profile_detail(&self, agent_id: &str) -> Result<AgentProfileDetail, AgentProfileError> {
        let row = self
            .conn
            .query_row(
                "SELECT profile.agent_id, profile.display_name, profile.avatar_shape, profile.avatar_color,
                        profile.revision, profile.instructions_revision, application.status, profile.updated_at,
                        profile.instructions
                 FROM agent_profiles profile
                 LEFT JOIN agent_profile_applications application ON application.agent_id = profile.agent_id
                 WHERE profile.agent_id = ?1",
                [agent_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?, row.get::<_, i64>(4)?, row.get::<_, i64>(5)?,
                        row.get::<_, Option<String>>(6)?, row.get::<_, i64>(7)?, row.get::<_, String>(8)?,
                    ))
                },
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?
            .ok_or(AgentProfileError::NotFound)?;
        let summary =
            self.summary_from_row((row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7))?;
        Ok(AgentProfileDetail {
            summary,
            instructions: row.8,
        })
    }

    /// Mise à jour de présentation sous verrou d'écriture, jamais une réécriture
    /// du profil complet : instructions et étiquettes ne sont même pas relues.
    /// Un retry du même nom est sans écriture.
    /// Même normalisation et même index UNIQUE que le renommage. Lecture seule,
    /// y compris pour un profil sans présence après redémarrage du daemon.
    pub fn agent_id_for_display_name(
        &self,
        name: &str,
    ) -> Result<Option<String>, AgentProfileError> {
        let normalized = normalize_display_name(name)?;
        self.conn
            .query_row(
                "SELECT agent_id FROM agent_profiles WHERE display_name_normalized=?1",
                [normalized],
                |row| row.get(0),
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)
    }

    /// Session 110 : `holder_is_live` répond « ce détenteur est-il présent
    /// dans l'annuaire vivant ? ». Un nom détenu par une identité éteinte est
    /// transféré au demandeur, et l'ancien détenteur reçoit un nom dérivé ;
    /// un nom détenu par une identité vivante reste refusé. Le magasin ne
    /// consulte pas l'annuaire lui-même : c'est un état mémoire du daemon.
    pub fn rename_display_name(
        &mut self,
        agent_id: &str,
        name: &str,
        holder_is_live: impl Fn(&str) -> bool,
    ) -> Result<AgentDisplayName, AgentProfileError> {
        if name.chars().any(char::is_control) {
            return Err(AgentProfileError::Invalid(
                "caractère de contrôle dans le nom",
            ));
        }
        let display_name = clean_display_name(name)?;
        let normalized = normalize_display_name(&display_name)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        let (current_name, revision) = tx
            .query_row(
                "SELECT display_name, revision FROM agent_profiles WHERE agent_id=?1",
                [agent_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?
            .ok_or(AgentProfileError::NotFound)?;
        if current_name != display_name {
            // Le détenteur éteint cède son nom dans la MÊME transaction : soit
            // les deux profils changent, soit aucun.
            if let Some(holder) = display_name_holder(&tx, agent_id, &normalized)? {
                if holder_is_live(&holder) {
                    return Err(AgentProfileError::DisplayNameConflict);
                }
                release_display_name(&tx, &holder)?;
            }
            let changed = tx.execute("UPDATE agent_profiles SET display_name=?2, display_name_normalized=?3, revision=revision+1, updated_at=?4 WHERE agent_id=?1 AND revision=?5",
                params![agent_id, display_name, normalized, now_secs(), revision]).map_err(AgentProfileError::Sqlite)?;
            if changed != 1 {
                return Err(AgentProfileError::RevisionConflict);
            }
        }
        let revision = tx
            .query_row(
                "SELECT revision FROM agent_profiles WHERE agent_id=?1",
                [agent_id],
                |row| row.get::<_, u64>(0),
            )
            .map_err(AgentProfileError::Sqlite)?;
        tx.commit().map_err(AgentProfileError::Sqlite)?;
        Ok(AgentDisplayName {
            agent_id: agent_id.into(),
            display_name,
            revision,
        })
    }

    pub fn update_profile(
        &mut self,
        agent_id: &str,
        update: AgentProfileUpdate,
    ) -> Result<AgentProfileDetail, AgentProfileError> {
        let display_name = clean_display_name(&update.display_name)?;
        let normalized_display_name = normalize_display_name(&display_name)?;
        let labels = clean_labels(&update.labels)?;
        validate_avatar(&update.avatar_shape, &update.avatar_color)?;
        let instructions = update.instructions.trim().to_string();
        if instructions.chars().count() > MAX_INSTRUCTIONS_CHARS {
            return Err(AgentProfileError::Invalid("instructions trop longues"));
        }
        let now = now_secs();
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        let current = transaction
            .query_row(
                "SELECT display_name_normalized, revision, instructions_revision, instructions
                 FROM agent_profiles WHERE agent_id = ?1",
                [agent_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?
            .ok_or(AgentProfileError::NotFound)?;
        if u64::try_from(current.1).ok() != Some(update.expected_revision) {
            return Err(AgentProfileError::RevisionConflict);
        }
        ensure_display_name_available(&transaction, agent_id, &normalized_display_name)?;
        let instructions_changed = current.3 != instructions;
        let instructions_revision = if instructions_changed {
            current.2 + 1
        } else {
            current.2
        };
        transaction
            .execute(
                "UPDATE agent_profiles
                 SET display_name = ?2, display_name_normalized = ?3, avatar_shape = ?4,
                     avatar_color = ?5, instructions = ?6, instructions_revision = ?7,
                     revision = revision + 1, updated_at = ?8
                 WHERE agent_id = ?1",
                params![
                    agent_id,
                    display_name,
                    normalized_display_name,
                    update.avatar_shape,
                    update.avatar_color,
                    instructions,
                    instructions_revision,
                    now
                ],
            )
            .map_err(AgentProfileError::Sqlite)?;
        transaction
            .execute(
                "DELETE FROM agent_profile_labels WHERE agent_id = ?1",
                [agent_id],
            )
            .map_err(AgentProfileError::Sqlite)?;
        for (position, (label, normalized_label)) in labels.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO agent_profile_labels(agent_id, position, label, normalized_label)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![agent_id, position as i64, label, normalized_label],
                )
                .map_err(AgentProfileError::Sqlite)?;
        }
        if instructions_changed {
            let status = if instructions.is_empty() {
                "applied"
            } else {
                "pending_restart"
            };
            transaction
                .execute(
                    "UPDATE agent_profile_applications
                     SET provider_spawn_id = NULL, instructions_revision = ?2,
                         status = ?3, observed_at = ?4, diagnostic_code = NULL
                     WHERE agent_id = ?1",
                    params![agent_id, instructions_revision, status, now],
                )
                .map_err(AgentProfileError::Sqlite)?;
        }
        transaction.commit().map_err(AgentProfileError::Sqlite)?;
        self.profile_detail(agent_id)
    }

    pub fn pending_instructions_for_agent_id(
        &self,
        agent_id: &str,
    ) -> Result<Option<PendingProfileInstructions>, AgentProfileError> {
        let row = self
            .conn
            .query_row(
                "SELECT profile.agent_id, profile.instructions, profile.instructions_revision
                 FROM agent_profiles profile
                 JOIN agent_profile_applications application ON application.agent_id = profile.agent_id
                 WHERE profile.agent_id = ?1
                   AND application.status = 'pending_restart'",
                [agent_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?;
        row.map(|(agent_id, instructions, revision)| {
            Ok(PendingProfileInstructions {
                agent_id,
                revision: u64::try_from(revision)
                    .map_err(|_| AgentProfileError::Invalid("révision instructions invalide"))?,
                instructions,
            })
        })
        .transpose()
    }

    pub fn pending_instructions_for_routing_name(
        &self,
        routing_name: &str,
    ) -> Result<Option<PendingProfileInstructions>, AgentProfileError> {
        let Some(alias_table) = self.legacy_alias_table(false)? else {
            return Ok(None);
        };
        let row = self
            .conn
            .query_row(
                &format!(
                    "SELECT profile.agent_id, profile.instructions, profile.instructions_revision
                 FROM {alias_table} alias
                 JOIN agent_profiles profile ON profile.agent_id = alias.agent_id
                 JOIN agent_profile_applications application ON application.agent_id = profile.agent_id
                 WHERE alias.routing_name = ?1
                   AND application.status = 'pending_restart'"
                ),
                [routing_name],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?;
        row.map(|(agent_id, instructions, revision)| {
            Ok(PendingProfileInstructions {
                agent_id,
                revision: u64::try_from(revision)
                    .map_err(|_| AgentProfileError::Invalid("révision instructions invalide"))?,
                instructions,
            })
        })
        .transpose()
    }

    pub fn mark_instruction_application(
        &mut self,
        agent_id: &str,
        revision: u64,
        provider_spawn_id: &str,
        status: InstructionStatus,
        diagnostic_code: Option<&str>,
    ) -> Result<(), AgentProfileError> {
        let now = now_secs();
        let changed = self
            .conn
            .execute(
                "UPDATE agent_profile_applications
                 SET provider_spawn_id = ?3, status = ?4, observed_at = ?5, diagnostic_code = ?6
                 WHERE agent_id = ?1 AND instructions_revision = ?2",
                params![
                    agent_id,
                    i64::try_from(revision).map_err(|_| AgentProfileError::Invalid(
                        "révision instructions invalide"
                    ))?,
                    provider_spawn_id,
                    status.as_str(),
                    now,
                    diagnostic_code,
                ],
            )
            .map_err(AgentProfileError::Sqlite)?;
        if changed == 0 {
            return Err(AgentProfileError::RevisionConflict);
        }
        Ok(())
    }

    pub fn record_attention_for_agent_id(
        &mut self,
        agent_id: &str,
        event_type: AttentionEventType,
        occurrence_key: &str,
    ) -> Result<bool, AgentProfileError> {
        let agent_id = agent_id.trim();
        if Uuid::parse_str(agent_id).is_err()
            || occurrence_key.is_empty()
            || occurrence_key.len() > 240
            || !occurrence_key.is_ascii()
        {
            return Err(AgentProfileError::Invalid("événement d'attention invalide"));
        }
        self.ensure_agent_ids([agent_id])?;
        let now = now_secs();
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        let inserted = transaction
            .execute(
                "INSERT INTO attention_events(
                     event_id, occurrence_key, agent_id, event_type, source_message_id, summary, created_at
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)
                 ON CONFLICT(occurrence_key) DO NOTHING",
                params![
                    Uuid::new_v4().to_string(),
                    occurrence_key,
                    agent_id,
                    event_type.as_str(),
                    event_type.as_str(),
                    now,
                ],
            )
            .map_err(AgentProfileError::Sqlite)?
            == 1;
        transaction.commit().map_err(AgentProfileError::Sqlite)?;
        Ok(inserted)
    }

    pub fn record_attention_for_routing_name(
        &mut self,
        routing_name: &str,
        event_type: AttentionEventType,
        occurrence_key: &str,
    ) -> Result<bool, AgentProfileError> {
        let routing_name = routing_name.trim();
        if routing_name.is_empty()
            || occurrence_key.is_empty()
            || occurrence_key.len() > 240
            || !occurrence_key.is_ascii()
        {
            return Err(AgentProfileError::Invalid("événement d'attention invalide"));
        }
        self.ensure_routing_names([routing_name])?;
        let alias_table = self
            .legacy_alias_table(false)?
            .ok_or(AgentProfileError::Invalid("table de migration absente"))?;
        let now = now_secs();
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        let agent_id = transaction
            .query_row(
                &format!("SELECT agent_id FROM {alias_table} WHERE routing_name = ?1"),
                [routing_name],
                |row| row.get::<_, String>(0),
            )
            .map_err(AgentProfileError::Sqlite)?;
        let inserted = transaction
            .execute(
                "INSERT INTO attention_events(
                     event_id, occurrence_key, agent_id, event_type, source_message_id, summary, created_at
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)
                 ON CONFLICT(occurrence_key) DO NOTHING",
                params![
                    Uuid::new_v4().to_string(),
                    occurrence_key,
                    agent_id,
                    event_type.as_str(),
                    event_type.as_str(),
                    now,
                ],
            )
            .map_err(AgentProfileError::Sqlite)?;
        transaction.commit().map_err(AgentProfileError::Sqlite)?;
        Ok(inserted == 1)
    }

    pub fn attention_for_client(
        &self,
        client_id: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<(Vec<AttentionEvent>, Option<String>), AgentProfileError> {
        validate_client_id(client_id)?;
        let limit = limit.clamp(1, 100);
        let cursor = match after {
            Some(event_id) => Some(
                self.conn
                    .query_row(
                        "SELECT created_at FROM attention_events WHERE event_id = ?1",
                        [event_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(AgentProfileError::Sqlite)?
                    .ok_or(AgentProfileError::NotFound)?,
            ),
            None => None,
        };
        let query =
            "SELECT event.event_id, profile.agent_id, profile.display_name, event.event_type,
                            event.created_at, state.seen_at, state.native_notified_at,
                            preference.human_input_needed, preference.task_completed,
                            preference.terminal_failure
                     FROM attention_events event
                     JOIN agent_profiles profile ON profile.agent_id = event.agent_id
                     LEFT JOIN client_attention_state state
                         ON state.event_id = event.event_id AND state.client_id = ?1
                     LEFT JOIN client_notification_preferences preference
                         ON preference.agent_id = event.agent_id AND preference.client_id = ?1
                     WHERE (?2 IS NULL OR event.created_at < ?2
                            OR (event.created_at = ?2 AND event.event_id < ?3))
                     ORDER BY event.created_at DESC, event.event_id DESC
                     LIMIT ?4";
        let mut statement = self
            .conn
            .prepare(query)
            .map_err(AgentProfileError::Sqlite)?;
        let rows = statement
            .query_map(
                params![
                    client_id,
                    cursor,
                    after,
                    i64::try_from(limit).unwrap_or(100)
                ],
                |row| {
                    let event_type = AttentionEventType::from_db(&row.get::<_, String>(3)?)
                        .map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?;
                    let human_input_needed = row.get::<_, Option<i64>>(7)?.unwrap_or(0) != 0;
                    let task_completed = row.get::<_, Option<i64>>(8)?.unwrap_or(0) != 0;
                    let terminal_failure = row.get::<_, Option<i64>>(9)?.unwrap_or(0) != 0;
                    let attention_enabled = match event_type {
                        AttentionEventType::HumanInputNeeded => human_input_needed,
                        AttentionEventType::TaskCompleted => task_completed,
                        AttentionEventType::TerminalFailure => terminal_failure,
                    };
                    Ok(AttentionEvent {
                        event_id: row.get(0)?,
                        agent_id: row.get(1)?,
                        display_name: row.get(2)?,
                        event_type,
                        created_at: row.get(4)?,
                        seen: row.get::<_, Option<i64>>(5)?.is_some(),
                        native_notified: row.get::<_, Option<i64>>(6)?.is_some(),
                        attention_enabled,
                    })
                },
            )
            .map_err(AgentProfileError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AgentProfileError::Sqlite)?;
        let next_after = rows.last().map(|event| event.event_id.clone());
        Ok((rows, next_after))
    }

    pub fn preferences_for_client(
        &self,
        client_id: &str,
    ) -> Result<Vec<ClientNotificationPreference>, AgentProfileError> {
        validate_client_id(client_id)?;
        let mut statement = self
            .conn
            .prepare(
                "SELECT profile.agent_id, preference.human_input_needed, preference.task_completed,
                        preference.terminal_failure
                 FROM agent_profiles profile
                 LEFT JOIN client_notification_preferences preference
                     ON preference.agent_id = profile.agent_id AND preference.client_id = ?1
                 ORDER BY profile.display_name_normalized ASC",
            )
            .map_err(AgentProfileError::Sqlite)?;
        statement
            .query_map([client_id], |row| {
                Ok(ClientNotificationPreference {
                    agent_id: row.get(0)?,
                    human_input_needed: row.get::<_, Option<i64>>(1)?.unwrap_or(0) != 0,
                    task_completed: row.get::<_, Option<i64>>(2)?.unwrap_or(0) != 0,
                    terminal_failure: row.get::<_, Option<i64>>(3)?.unwrap_or(0) != 0,
                })
            })
            .map_err(AgentProfileError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AgentProfileError::Sqlite)
    }

    pub fn replace_preferences_for_client(
        &mut self,
        client_id: &str,
        preferences: &[ClientNotificationPreference],
    ) -> Result<Vec<ClientNotificationPreference>, AgentProfileError> {
        validate_client_id(client_id)?;
        let mut agent_ids = HashSet::new();
        if preferences
            .iter()
            .any(|preference| !agent_ids.insert(preference.agent_id.as_str()))
        {
            return Err(AgentProfileError::Invalid("préférences dupliquées"));
        }
        let now = now_secs();
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        for preference in preferences {
            let exists = transaction
                .query_row(
                    "SELECT 1 FROM agent_profiles WHERE agent_id = ?1",
                    [&preference.agent_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(AgentProfileError::Sqlite)?
                .is_some();
            if !exists {
                return Err(AgentProfileError::NotFound);
            }
        }
        transaction
            .execute(
                "DELETE FROM client_notification_preferences WHERE client_id = ?1",
                [client_id],
            )
            .map_err(AgentProfileError::Sqlite)?;
        for preference in preferences {
            transaction
                .execute(
                    "INSERT INTO client_notification_preferences(
                         client_id, agent_id, human_input_needed, task_completed, terminal_failure, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        client_id,
                        preference.agent_id,
                        preference.human_input_needed as i64,
                        preference.task_completed as i64,
                        preference.terminal_failure as i64,
                        now,
                    ],
                )
                .map_err(AgentProfileError::Sqlite)?;
        }
        transaction.commit().map_err(AgentProfileError::Sqlite)?;
        self.preferences_for_client(client_id)
    }

    pub fn mark_attention_state(
        &mut self,
        client_id: &str,
        event_ids: &[String],
        action: &str,
    ) -> Result<(), AgentProfileError> {
        validate_client_id(client_id)?;
        if event_ids.is_empty() || event_ids.len() > 100 {
            return Err(AgentProfileError::Invalid(
                "événements d'attention invalides",
            ));
        }
        let now = now_secs();
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        let field = match action {
            "mark_seen" => "seen_at",
            "mark_native_notified" => "native_notified_at",
            _ => return Err(AgentProfileError::Invalid("action d'attention invalide")),
        };
        for event_id in event_ids {
            if Uuid::parse_str(event_id).is_err() {
                return Err(AgentProfileError::Invalid("événement d'attention invalide"));
            }
            let exists = transaction
                .query_row(
                    "SELECT 1 FROM attention_events WHERE event_id = ?1",
                    [event_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(AgentProfileError::Sqlite)?
                .is_some();
            if !exists {
                return Err(AgentProfileError::NotFound);
            }
            let query = format!(
                "INSERT INTO client_attention_state(client_id, event_id, {field}) VALUES (?1, ?2, ?3)
                 ON CONFLICT(client_id, event_id) DO UPDATE SET {field} = COALESCE({field}, excluded.{field})"
            );
            transaction
                .execute(&query, params![client_id, event_id, now])
                .map_err(AgentProfileError::Sqlite)?;
        }
        transaction.commit().map_err(AgentProfileError::Sqlite)
    }

    fn summary_from_row(
        &self,
        row: (
            String,
            String,
            String,
            String,
            i64,
            i64,
            Option<String>,
            i64,
        ),
    ) -> Result<AgentProfileSummary, AgentProfileError> {
        let labels = self.labels_for_profile(&row.0)?;
        Ok(AgentProfileSummary {
            agent_id: row.0,
            display_name: row.1,
            avatar_shape: row.2,
            avatar_color: row.3,
            revision: u64::try_from(row.4)
                .map_err(|_| AgentProfileError::Invalid("révision invalide"))?,
            instructions_revision: u64::try_from(row.5)
                .map_err(|_| AgentProfileError::Invalid("révision instructions invalide"))?,
            instruction_status: InstructionStatus::from_db(
                row.6.as_deref().unwrap_or("pending_restart"),
            ),
            updated_at: row.7,
            labels,
        })
    }

    fn labels_for_profile(&self, agent_id: &str) -> Result<Vec<String>, AgentProfileError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT label FROM agent_profile_labels WHERE agent_id = ?1 ORDER BY position ASC",
            )
            .map_err(AgentProfileError::Sqlite)?;
        statement
            .query_map([agent_id], |row| row.get::<_, String>(0))
            .map_err(AgentProfileError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AgentProfileError::Sqlite)
    }
}

fn available_display_name(
    conn: &rusqlite::Transaction<'_>,
    candidate: &str,
) -> Result<String, AgentProfileError> {
    let base = clean_display_name(candidate)?;
    for suffix in 1..=1_000 {
        let display_name = if suffix == 1 {
            base.clone()
        } else {
            format!("{base} ({suffix})")
        };
        let normalized = normalize_display_name(&display_name)?;
        let exists = conn
            .query_row(
                "SELECT 1 FROM agent_profiles WHERE display_name_normalized = ?1",
                [normalized],
                |_| Ok(()),
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?
            .is_some();
        if !exists {
            return Ok(display_name);
        }
    }
    Err(AgentProfileError::Invalid("noms affichés épuisés"))
}

fn ensure_display_name_available(
    tx: &rusqlite::Transaction<'_>,
    agent_id: &str,
    normalized: &str,
) -> Result<(), AgentProfileError> {
    if display_name_holder(tx, agent_id, normalized)?.is_some() {
        return Err(AgentProfileError::DisplayNameConflict);
    }
    Ok(())
}

/// Détenteur actuel d'un nom normalisé, hors le demandeur lui-même.
fn display_name_holder(
    tx: &rusqlite::Transaction<'_>,
    agent_id: &str,
    normalized: &str,
) -> Result<Option<String>, AgentProfileError> {
    tx.query_row(
        "SELECT agent_id FROM agent_profiles WHERE display_name_normalized=?1 AND agent_id!=?2",
        params![normalized, agent_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(AgentProfileError::Sqlite)
}

/// Session 110 : dépossède un détenteur éteint en lui attribuant un nom dérivé
/// disponible. L'identité, l'avatar et les instructions ne bougent pas ; seule
/// la révision avance, pour que les lecteurs voient le changement.
fn release_display_name(
    tx: &rusqlite::Transaction<'_>,
    holder: &str,
) -> Result<(), AgentProfileError> {
    let current: String = tx
        .query_row(
            "SELECT display_name FROM agent_profiles WHERE agent_id=?1",
            [holder],
            |row| row.get(0),
        )
        .map_err(AgentProfileError::Sqlite)?;
    let fallback = available_display_name(tx, &current)?;
    let normalized = normalize_display_name(&fallback)?;
    let changed = tx
        .execute(
            "UPDATE agent_profiles SET display_name=?2, display_name_normalized=?3, revision=revision+1, updated_at=?4 WHERE agent_id=?1",
            params![holder, fallback, normalized, now_secs()],
        )
        .map_err(AgentProfileError::Sqlite)?;
    if changed != 1 {
        return Err(AgentProfileError::NotFound);
    }
    Ok(())
}

fn clean_display_name(raw: &str) -> Result<String, AgentProfileError> {
    let value = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty() || value.chars().count() > MAX_DISPLAY_NAME_CHARS {
        return Err(AgentProfileError::Invalid("nom affiché invalide"));
    }
    Ok(value)
}

fn normalize_display_name(raw: &str) -> Result<String, AgentProfileError> {
    let value = clean_display_name(raw)?;
    Ok(fold_for_search(&value))
}

fn clean_labels(raw_labels: &[String]) -> Result<Vec<(String, String)>, AgentProfileError> {
    let mut labels = Vec::new();
    let mut seen = HashSet::new();
    for raw in raw_labels {
        for part in raw.split(',') {
            let label = part.split_whitespace().collect::<Vec<_>>().join(" ");
            if label.is_empty() {
                continue;
            }
            if label.chars().count() > MAX_LABEL_CHARS {
                return Err(AgentProfileError::Invalid("label trop long"));
            }
            let normalized = fold_for_search(&label);
            if seen.insert(normalized.clone()) {
                labels.push((label, normalized));
            }
        }
    }
    if labels.len() > MAX_LABELS {
        return Err(AgentProfileError::Invalid("trop de labels"));
    }
    Ok(labels)
}

fn validate_avatar(shape: &str, color: &str) -> Result<(), AgentProfileError> {
    if !AVATAR_SHAPES.contains(&shape) || !AVATAR_COLORS.contains(&color) {
        return Err(AgentProfileError::Invalid("apparence invalide"));
    }
    Ok(())
}

fn validate_client_id(client_id: &str) -> Result<(), AgentProfileError> {
    if Uuid::parse_str(client_id).is_err() {
        return Err(AgentProfileError::Invalid("client invalide"));
    }
    Ok(())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (AgentProfileStore, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("bridget-profile-{}.db", Uuid::new_v4()));
        (AgentProfileStore::open(&path).unwrap(), path)
    }

    /// Session 110 : un fil T3 réimporté reprend le nom que son ancienne
    /// incarnation, éteinte, détenait encore.
    #[test]
    fn spec110_nom_detenu_par_une_identite_eteinte_est_transfere() {
        let (mut store, path) = store();
        let ancien = Uuid::new_v4().to_string();
        let nouveau = Uuid::new_v4().to_string();
        store
            .ensure_agent_ids([ancien.as_str(), nouveau.as_str()])
            .unwrap();
        store
            .rename_display_name(&ancien, "claude-horizon", |_| false)
            .unwrap();
        let revision_avant: u64 = store
            .conn
            .query_row(
                "SELECT revision FROM agent_profiles WHERE agent_id=?1",
                [ancien.as_str()],
                |row| row.get(0),
            )
            .unwrap();

        // L'ancien détenteur est absent de l'annuaire : le nom est cédé.
        let applied = store
            .rename_display_name(&nouveau, "claude-horizon", |holder| {
                assert_eq!(holder, ancien, "le détenteur consulté est bien l'ancien");
                false
            })
            .unwrap();
        assert_eq!(applied.display_name, "claude-horizon");
        assert_eq!(
            store.agent_id_for_display_name("claude-horizon").unwrap(),
            Some(nouveau.clone()),
            "la résolution rend le détenteur courant"
        );

        // L'ancien garde son identité, reçoit un nom dérivé distinct et voit sa
        // révision avancer.
        let (nom_ancien, revision_apres): (String, u64) = store
            .conn
            .query_row(
                "SELECT display_name, revision FROM agent_profiles WHERE agent_id=?1",
                [ancien.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_ne!(nom_ancien, "claude-horizon");
        assert!(nom_ancien.starts_with("claude-horizon"), "{nom_ancien}");
        assert!(revision_apres > revision_avant, "révision non avancée");
        let profils: i64 = store
            .conn
            .query_row("SELECT count(*) FROM agent_profiles", [], |row| row.get(0))
            .unwrap();
        assert_eq!(profils, 2, "aucune identité créée ni supprimée");
        let _ = std::fs::remove_file(&path);
    }

    /// Le nom d'un agent vivant n'est jamais volé : le refus reste le même.
    #[test]
    fn spec110_nom_detenu_par_une_identite_vivante_reste_refuse() {
        let (mut store, path) = store();
        let vivant = Uuid::new_v4().to_string();
        let demandeur = Uuid::new_v4().to_string();
        store
            .ensure_agent_ids([vivant.as_str(), demandeur.as_str()])
            .unwrap();
        store
            .rename_display_name(&vivant, "cursor-listen", |_| false)
            .unwrap();

        let refus =
            store.rename_display_name(&demandeur, "cursor-listen", |holder| holder == vivant);
        assert!(
            matches!(refus, Err(AgentProfileError::DisplayNameConflict)),
            "{refus:?}"
        );
        assert_eq!(
            store.agent_id_for_display_name("cursor-listen").unwrap(),
            Some(vivant.clone()),
            "le vivant garde son nom"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// Reprendre son propre nom ne déclenche aucun transfert.
    #[test]
    fn spec110_reprendre_son_propre_nom_ne_transfere_rien() {
        let (mut store, path) = store();
        let id = Uuid::new_v4().to_string();
        store.ensure_agent_ids([id.as_str()]).unwrap();
        store.rename_display_name(&id, "wiki", |_| false).unwrap();
        let applied = store
            .rename_display_name(&id, "wiki", |_| panic!("aucun détenteur à consulter"))
            .unwrap();
        assert_eq!(applied.display_name, "wiki");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn resolution_du_nom_survit_a_la_reouverture_sans_creer_de_profil() {
        let (mut store, path) = store();
        let id = Uuid::new_v4().to_string();
        store.ensure_agent_ids([id.as_str()]).unwrap();
        store
            .rename_display_name(&id, "Gui-Codér", |_| true)
            .unwrap();
        drop(store);
        let store = AgentProfileStore::open(&path).unwrap();
        assert_eq!(
            store.agent_id_for_display_name("gui-coder").unwrap(),
            Some(id)
        );
        let before: i64 = store
            .conn
            .query_row("SELECT count(*) FROM agent_profiles", [], |row| row.get(0))
            .unwrap();
        assert_eq!(store.agent_id_for_display_name("absent").unwrap(), None);
        let after: i64 = store
            .conn
            .query_row("SELECT count(*) FROM agent_profiles", [], |row| row.get(0))
            .unwrap();
        assert_eq!(before, after);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn migration_expose_la_table_puis_purge_les_alias_legacy() {
        let (mut store, path) = store();
        store.ensure_routing_names(["ancien-agent"]).unwrap();
        let mapping = store.legacy_routing_map().unwrap();
        let agent_id = mapping.get("ancien-agent").unwrap().clone();
        assert!(Uuid::parse_str(&agent_id).is_ok());
        store.retire_legacy_routing_aliases().unwrap();
        assert!(store.legacy_routing_map().unwrap().is_empty());
        assert!(store.profile_for_agent_id(&agent_id).unwrap().is_some());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn import_is_idempotent_and_keeps_routing_out_of_profile() {
        let (mut store, path) = store();
        store
            .ensure_routing_names(["agent-technique", "agent-technique"])
            .unwrap();
        store.ensure_routing_names(["agent-technique"]).unwrap();
        let profile = store
            .profile_for_routing_name("agent-technique")
            .unwrap()
            .unwrap();
        assert_eq!(profile.display_name, "Agent");
        assert_eq!(profile.labels, Vec::<String>::new());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn update_splits_labels_and_rejects_stale_revision() {
        let (mut store, path) = store();
        store.ensure_routing_names(["alpha"]).unwrap();
        let profile = store.profile_for_routing_name("alpha").unwrap().unwrap();
        let saved = store
            .update_profile(
                &profile.agent_id,
                AgentProfileUpdate {
                    expected_revision: profile.revision,
                    display_name: "Alpha".to_string(),
                    labels: vec![
                        "coordination, recherche".to_string(),
                        "recherche".to_string(),
                    ],
                    avatar_shape: "round".to_string(),
                    avatar_color: "blue".to_string(),
                    instructions: "Réponds en français.".to_string(),
                },
            )
            .unwrap();
        assert_eq!(saved.summary.labels, vec!["coordination", "recherche"]);
        assert_eq!(
            saved.summary.instruction_status,
            InstructionStatus::PendingRestart
        );
        let pending = store
            .pending_instructions_for_routing_name("alpha")
            .unwrap()
            .expect("consigne à appliquer");
        assert_eq!(pending.instructions, "Réponds en français.");
        store
            .mark_instruction_application(
                &pending.agent_id,
                pending.revision,
                "spawn-1",
                InstructionStatus::Applied,
                None,
            )
            .unwrap();
        assert_eq!(
            store
                .profile_detail(&pending.agent_id)
                .unwrap()
                .summary
                .instruction_status,
            InstructionStatus::Applied,
        );
        assert!(matches!(
            store.update_profile(
                &profile.agent_id,
                AgentProfileUpdate {
                    expected_revision: profile.revision,
                    display_name: "Alpha".to_string(),
                    labels: vec![],
                    avatar_shape: "round".to_string(),
                    avatar_color: "blue".to_string(),
                    instructions: String::new(),
                }
            ),
            Err(AgentProfileError::RevisionConflict)
        ));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn attention_est_idempotente_et_isolee_par_client() {
        let (mut store, path) = store();
        store.ensure_routing_names(["alpha"]).unwrap();
        let profile = store.profile_for_routing_name("alpha").unwrap().unwrap();
        let client_a = Uuid::new_v4().to_string();
        let client_b = Uuid::new_v4().to_string();
        assert!(
            store
                .record_attention_for_routing_name(
                    "alpha",
                    AttentionEventType::HumanInputNeeded,
                    "wait:alpha:execution-1:request-1",
                )
                .unwrap()
        );
        assert!(
            !store
                .record_attention_for_routing_name(
                    "alpha",
                    AttentionEventType::HumanInputNeeded,
                    "wait:alpha:execution-1:request-1",
                )
                .unwrap()
        );
        let preferences = store
            .replace_preferences_for_client(
                &client_a,
                &[ClientNotificationPreference {
                    agent_id: profile.agent_id.clone(),
                    human_input_needed: true,
                    task_completed: false,
                    terminal_failure: true,
                }],
            )
            .unwrap();
        assert_eq!(preferences.len(), 1);
        let (for_a, _) = store.attention_for_client(&client_a, None, 100).unwrap();
        assert_eq!(for_a.len(), 1);
        assert!(for_a[0].attention_enabled);
        assert!(!for_a[0].seen);
        let (for_b, _) = store.attention_for_client(&client_b, None, 100).unwrap();
        assert_eq!(for_b.len(), 1);
        assert!(!for_b[0].attention_enabled);
        store
            .mark_attention_state(&client_a, &[for_a[0].event_id.clone()], "mark_seen")
            .unwrap();
        let (for_a_after, _) = store.attention_for_client(&client_a, None, 100).unwrap();
        let (for_b_after, _) = store.attention_for_client(&client_b, None, 100).unwrap();
        assert!(for_a_after[0].seen);
        assert!(!for_b_after[0].seen);
        std::fs::remove_file(path).unwrap();
    }
}
