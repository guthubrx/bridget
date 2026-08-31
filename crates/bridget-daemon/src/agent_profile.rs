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
    pub profile_ref: String,
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
    pub profile_ref: String,
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
    pub profile_ref: String,
    pub display_name: String,
    pub event_type: AttentionEventType,
    pub created_at: i64,
    pub seen: bool,
    pub native_notified: bool,
    pub attention_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientNotificationPreference {
    pub profile_ref: String,
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
                 current_routing_name TEXT NOT NULL UNIQUE,
                 created_at INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS agent_routing_aliases (
                 routing_name TEXT PRIMARY KEY,
                 agent_id TEXT NOT NULL REFERENCES agent_identities(agent_id),
                 is_current INTEGER NOT NULL CHECK (is_current IN (0, 1)),
                 observed_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_agent_routing_aliases_agent
                 ON agent_routing_aliases(agent_id, is_current DESC);
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
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        for routing_name in names {
            let existing = transaction
                .query_row(
                    "SELECT agent_id FROM agent_routing_aliases WHERE routing_name = ?1",
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
            transaction
                .execute(
                    "INSERT INTO agent_identities(agent_id, current_routing_name, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?3)",
                    params![agent_id, routing_name, now],
                )
                .map_err(AgentProfileError::Sqlite)?;
            transaction
                .execute(
                    "INSERT INTO agent_routing_aliases(routing_name, agent_id, is_current, observed_at)
                     VALUES (?1, ?2, 1, ?3)",
                    params![routing_name, agent_id, now],
                )
                .map_err(AgentProfileError::Sqlite)?;
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
        let row = self
            .conn
            .query_row(
                "SELECT identity.agent_id, profile.display_name, profile.avatar_shape, profile.avatar_color,
                        profile.revision, profile.instructions_revision, application.status, profile.updated_at
                 FROM agent_routing_aliases alias
                 JOIN agent_identities identity ON identity.agent_id = alias.agent_id
                 JOIN agent_profiles profile ON profile.agent_id = identity.agent_id
                 LEFT JOIN agent_profile_applications application ON application.agent_id = identity.agent_id
                 WHERE alias.routing_name = ?1",
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

    pub fn profile_detail(
        &self,
        profile_ref: &str,
    ) -> Result<AgentProfileDetail, AgentProfileError> {
        let row = self
            .conn
            .query_row(
                "SELECT profile.agent_id, profile.display_name, profile.avatar_shape, profile.avatar_color,
                        profile.revision, profile.instructions_revision, application.status, profile.updated_at,
                        profile.instructions
                 FROM agent_profiles profile
                 LEFT JOIN agent_profile_applications application ON application.agent_id = profile.agent_id
                 WHERE profile.agent_id = ?1",
                [profile_ref],
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

    pub fn update_profile(
        &mut self,
        profile_ref: &str,
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
                [profile_ref],
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
        let conflict = transaction
            .query_row(
                "SELECT agent_id FROM agent_profiles WHERE display_name_normalized = ?1 AND agent_id != ?2",
                params![normalized_display_name, profile_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(AgentProfileError::Sqlite)?;
        if conflict.is_some() {
            return Err(AgentProfileError::DisplayNameConflict);
        }
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
                    profile_ref,
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
                [profile_ref],
            )
            .map_err(AgentProfileError::Sqlite)?;
        for (position, (label, normalized_label)) in labels.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO agent_profile_labels(agent_id, position, label, normalized_label)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![profile_ref, position as i64, label, normalized_label],
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
                    params![profile_ref, instructions_revision, status, now],
                )
                .map_err(AgentProfileError::Sqlite)?;
        }
        transaction.commit().map_err(AgentProfileError::Sqlite)?;
        self.profile_detail(profile_ref)
    }

    pub fn pending_instructions_for_routing_name(
        &self,
        routing_name: &str,
    ) -> Result<Option<PendingProfileInstructions>, AgentProfileError> {
        let row = self
            .conn
            .query_row(
                "SELECT profile.agent_id, profile.instructions, profile.instructions_revision
                 FROM agent_routing_aliases alias
                 JOIN agent_profiles profile ON profile.agent_id = alias.agent_id
                 JOIN agent_profile_applications application ON application.agent_id = profile.agent_id
                 WHERE alias.routing_name = ?1
                   AND application.status = 'pending_restart'",
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
        row.map(|(profile_ref, instructions, revision)| {
            Ok(PendingProfileInstructions {
                profile_ref,
                revision: u64::try_from(revision)
                    .map_err(|_| AgentProfileError::Invalid("révision instructions invalide"))?,
                instructions,
            })
        })
        .transpose()
    }

    pub fn mark_instruction_application(
        &mut self,
        profile_ref: &str,
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
                    profile_ref,
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
        let now = now_secs();
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AgentProfileError::Sqlite)?;
        let agent_id = transaction
            .query_row(
                "SELECT agent_id FROM agent_routing_aliases WHERE routing_name = ?1",
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
                        profile_ref: row.get(1)?,
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
                    profile_ref: row.get(0)?,
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
        let mut profile_refs = HashSet::new();
        if preferences
            .iter()
            .any(|preference| !profile_refs.insert(preference.profile_ref.as_str()))
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
                    [&preference.profile_ref],
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
                        preference.profile_ref,
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
            profile_ref: row.0,
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

    fn labels_for_profile(&self, profile_ref: &str) -> Result<Vec<String>, AgentProfileError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT label FROM agent_profile_labels WHERE agent_id = ?1 ORDER BY position ASC",
            )
            .map_err(AgentProfileError::Sqlite)?;
        statement
            .query_map([profile_ref], |row| row.get::<_, String>(0))
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
                &profile.profile_ref,
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
                &pending.profile_ref,
                pending.revision,
                "spawn-1",
                InstructionStatus::Applied,
                None,
            )
            .unwrap();
        assert_eq!(
            store
                .profile_detail(&pending.profile_ref)
                .unwrap()
                .summary
                .instruction_status,
            InstructionStatus::Applied,
        );
        assert!(matches!(
            store.update_profile(
                &profile.profile_ref,
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
                    profile_ref: profile.profile_ref.clone(),
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
