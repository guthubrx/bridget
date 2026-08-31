//! Migration explicite des identités historiques vers des UUID opaques.
//!
//! Aucun démarrage du daemon ne modifie les données. La migration est lancée
//! volontairement par l'opérateur et laisse des sauvegardes privées ainsi
//! qu'un journal durable, afin de pouvoir reprendre après un arrêt entre deux
//! ressources.

use crate::agent_profile::AgentProfileStore;
use crate::desired_state::{DesiredStateStore, path_for_daemon_db};
use crate::store::Store;
use bridget_core::router::validate_agent_id;
use bridget_transport::fsutil::write_private_file_atomic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const RESERVED_PRINCIPALS: &[&str] = &["human", "humain", "maicie"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityMigrationPaths {
    pub bridget_db: PathBuf,
    pub fleet_path: PathBuf,
    pub maicie_config: Option<PathBuf>,
}

impl IdentityMigrationPaths {
    pub fn for_bridget_db(bridget_db: PathBuf, maicie_config: Option<PathBuf>) -> Self {
        let fleet_path = path_for_daemon_db(&bridget_db);
        Self {
            bridget_db,
            fleet_path,
            maicie_config,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityMigrationPlan {
    pub paths: IdentityMigrationPaths,
    pub mapping: BTreeMap<String, String>,
    pub requires_retarget: BTreeSet<String>,
    pub maicie_db: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityMigrationOutcome {
    pub migrated_agents: usize,
    pub backup_paths: Vec<PathBuf>,
    pub journal_path: PathBuf,
}

#[derive(Debug)]
pub enum IdentityMigrationError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Profile(String),
    BridgetStore(String),
    Fleet(String),
    MaicieConfig(String),
    MaicieStore(String),
    Invalid(&'static str),
}

impl fmt::Display for IdentityMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {}", path.display(), source),
            Self::Json { path, source } => write!(formatter, "{}: {}", path.display(), source),
            Self::Profile(reason) => write!(formatter, "profils Bridget: {reason}"),
            Self::BridgetStore(reason) => write!(formatter, "registre Bridget: {reason}"),
            Self::Fleet(reason) => write!(formatter, "flotte Bridget: {reason}"),
            Self::MaicieConfig(reason) => write!(formatter, "configuration Maicie: {reason}"),
            Self::MaicieStore(reason) => write!(formatter, "registre Maicie: {reason}"),
            Self::Invalid(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for IdentityMigrationError {}

#[derive(Debug, Serialize, Deserialize)]
struct MigrationJournal {
    version: u8,
    state: String,
    mapping: BTreeMap<String, String>,
    backups: Vec<PathBuf>,
    completed: Vec<String>,
    failure: Option<String>,
}

pub fn plan(
    paths: IdentityMigrationPaths,
) -> Result<IdentityMigrationPlan, IdentityMigrationError> {
    ensure_existing_file(&paths.bridget_db)?;
    if paths.fleet_path.exists() {
        ensure_existing_file(&paths.fleet_path)?;
    }

    let mut historical_references = BTreeSet::new();
    if paths.fleet_path.exists() {
        let fleet = DesiredStateStore::at_path(&paths.fleet_path)
            .load()
            .map_err(|error| IdentityMigrationError::Fleet(error.to_string()))?;
        historical_references.extend(fleet.equipiers.into_keys());
    }

    // Une référence déclarée par la configuration Maicie correspond encore à
    // un agent connu, même si cet agent n'est pas dans la flotte courante.
    // Seules les références uniquement historisées dans SQLite doivent être
    // mises en attente de retarget.
    let mut known_references = historical_references.clone();
    let (maicie_db, maicie_references) = match paths.maicie_config.as_ref() {
        Some(path) => {
            ensure_existing_file(path)?;
            let value = read_json(path)?;
            let references = config_agent_references(&value);
            known_references.extend(references.iter().cloned());
            let db = value
                .get("database_path")
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .ok_or(IdentityMigrationError::Invalid(
                    "database_path Maicie absent de la configuration",
                ))?;
            if !db.is_absolute() {
                return Err(IdentityMigrationError::Invalid(
                    "database_path Maicie doit être absolu",
                ));
            }
            ensure_existing_file(&db)?;
            let persisted = maicie::store::MaicieStore::open(&db)
                .map_err(|error| IdentityMigrationError::MaicieStore(error.to_string()))?
                .agent_references_for_identity_migration()
                .map_err(|error| IdentityMigrationError::MaicieStore(error.to_string()))?;
            (Some(db), references.into_iter().chain(persisted).collect())
        }
        None => (None, BTreeSet::new()),
    };
    historical_references.extend(maicie_references.clone());

    let mut profiles = AgentProfileStore::open(&paths.bridget_db)
        .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;
    profiles
        .import_historical_routing_names()
        .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;
    let opaque_ids = historical_references
        .iter()
        .filter(|reference| validate_agent_id(reference).is_ok())
        .cloned()
        .collect::<Vec<_>>();
    profiles
        .ensure_agent_ids(opaque_ids)
        .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;
    let legacy = historical_references
        .iter()
        .filter(|reference| {
            !is_reserved_principal(reference) && validate_agent_id(reference).is_err()
        })
        .cloned()
        .collect::<Vec<_>>();
    profiles
        .ensure_routing_names(legacy)
        .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;

    let mut mapping = profiles
        .legacy_routing_map()
        .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;
    mapping.retain(|legacy, _| !is_reserved_principal(legacy));
    let requires_retarget = maicie_references
        .iter()
        .filter(|reference| !known_references.contains(*reference))
        .filter_map(|reference| mapping.get(reference).cloned())
        .collect();

    Ok(IdentityMigrationPlan {
        paths,
        mapping,
        requires_retarget,
        maicie_db,
    })
}

pub fn apply(
    plan: &IdentityMigrationPlan,
) -> Result<IdentityMigrationOutcome, IdentityMigrationError> {
    let mut backup_sources = vec![plan.paths.bridget_db.clone()];
    if plan.paths.fleet_path.exists() {
        backup_sources.push(plan.paths.fleet_path.clone());
    }
    if let Some(config) = &plan.paths.maicie_config {
        backup_sources.push(config.clone());
    }
    if let Some(database) = &plan.maicie_db {
        backup_sources.push(database.clone());
    }

    let migration_id = uuid::Uuid::new_v4();
    let journal_path = plan
        .paths
        .bridget_db
        .parent()
        .ok_or(IdentityMigrationError::Invalid("base Bridget sans parent"))?
        .join(format!("identity-migration-{migration_id}.json"));
    let backups = backup_sources
        .iter()
        .map(|path| backup_file(path.as_path(), &migration_id.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let mut journal = MigrationJournal {
        version: 1,
        state: "running".to_string(),
        mapping: plan.mapping.clone(),
        backups: backups.clone(),
        completed: Vec::new(),
        failure: None,
    };
    write_journal(&journal_path, &journal)?;

    let result = (|| {
        let mut store = Store::open(&plan.paths.bridget_db)
            .map_err(|error| IdentityMigrationError::BridgetStore(error.to_string()))?;
        store
            .migrate_agent_references(&plan.mapping)
            .map_err(|error| IdentityMigrationError::BridgetStore(error.to_string()))?;
        journal.completed.push("bridget_ledger".to_string());
        write_journal(&journal_path, &journal)?;

        if plan.paths.fleet_path.exists() {
            DesiredStateStore::at_path(&plan.paths.fleet_path)
                .migrate_agent_ids(&plan.mapping)
                .map_err(|error| IdentityMigrationError::Fleet(error.to_string()))?;
            journal.completed.push("fleet".to_string());
            write_journal(&journal_path, &journal)?;
        }

        if let (Some(config_path), Some(database_path)) =
            (plan.paths.maicie_config.as_ref(), plan.maicie_db.as_ref())
        {
            let value = read_json(config_path)?;
            let migrated = migrate_config_value(value, &plan.mapping)?;
            let typed: maicie::config::MaicieConfig = serde_json::from_value(migrated.clone())
                .map_err(|source| IdentityMigrationError::Json {
                    path: config_path.clone(),
                    source,
                })?;
            typed
                .validate()
                .map_err(|error| IdentityMigrationError::MaicieConfig(error.to_string()))?;

            let mut maicie = maicie::store::MaicieStore::open(database_path)
                .map_err(|error| IdentityMigrationError::MaicieStore(error.to_string()))?;
            maicie
                .migrate_agent_participants(&plan.mapping)
                .map_err(|error| IdentityMigrationError::MaicieStore(error.to_string()))?;
            maicie
                .mark_agents_requires_retarget(&plan.requires_retarget)
                .map_err(|error| IdentityMigrationError::MaicieStore(error.to_string()))?;
            journal.completed.push("maicie_store".to_string());
            write_journal(&journal_path, &journal)?;

            let bytes = serde_json::to_vec_pretty(&migrated).map_err(|source| {
                IdentityMigrationError::Json {
                    path: config_path.clone(),
                    source,
                }
            })?;
            write_private_file_atomic(config_path, &bytes).map_err(|source| {
                IdentityMigrationError::Io {
                    path: config_path.clone(),
                    source,
                }
            })?;
            journal.completed.push("maicie_config".to_string());
            write_journal(&journal_path, &journal)?;
        }

        let mut profiles = AgentProfileStore::open(&plan.paths.bridget_db)
            .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;
        profiles
            .retire_legacy_routing_aliases()
            .map_err(|error| IdentityMigrationError::Profile(error.to_string()))?;
        journal.completed.push("legacy_aliases_removed".to_string());
        Ok::<(), IdentityMigrationError>(())
    })();

    match result {
        Ok(()) => {
            journal.state = "completed".to_string();
            write_journal(&journal_path, &journal)?;
            Ok(IdentityMigrationOutcome {
                migrated_agents: plan.mapping.len(),
                backup_paths: backups,
                journal_path,
            })
        }
        Err(error) => {
            journal.state = "failed".to_string();
            journal.failure = Some(error.to_string());
            let _ = write_journal(&journal_path, &journal);
            Err(error)
        }
    }
}

fn is_reserved_principal(value: &str) -> bool {
    RESERVED_PRINCIPALS.contains(&value)
}

fn ensure_existing_file(path: &Path) -> Result<(), IdentityMigrationError> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(IdentityMigrationError::Invalid(
            "un fichier régulier est requis",
        )),
        Err(source) => Err(IdentityMigrationError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn read_json(path: &Path) -> Result<Value, IdentityMigrationError> {
    let bytes = fs::read(path).map_err(|source| IdentityMigrationError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| IdentityMigrationError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn config_agent_references(value: &Value) -> BTreeSet<String> {
    let mut references = BTreeSet::new();
    collect_config_references(value, &mut references);
    references
}

fn collect_config_references(value: &Value, references: &mut BTreeSet<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_config_references(value, references);
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                if matches!(
                    key.as_str(),
                    "agent_name" | "agent_id" | "participant_id" | "referent_id"
                ) && let Some(value) = value.as_str()
                {
                    references.insert(value.to_string());
                }
                collect_config_references(value, references);
            }
        }
        _ => {}
    }
}

fn migrate_config_value(
    mut value: Value,
    mapping: &BTreeMap<String, String>,
) -> Result<Value, IdentityMigrationError> {
    migrate_value(&mut value, mapping);
    Ok(value)
}

fn migrate_value(value: &mut Value, mapping: &BTreeMap<String, String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                migrate_value(value, mapping);
            }
        }
        Value::Object(object) => {
            if let Some(agent_name) = object.remove("agent_name") {
                if let Some(legacy) = agent_name.as_str() {
                    let agent_id = mapping
                        .get(legacy)
                        .cloned()
                        .unwrap_or_else(|| legacy.to_string());
                    object.insert("agent_id".to_string(), Value::String(agent_id));
                }
            }
            for (key, value) in object.iter_mut() {
                if matches!(key.as_str(), "agent_id" | "participant_id" | "referent_id")
                    && let Some(legacy) = value.as_str()
                    && let Some(agent_id) = mapping.get(legacy)
                {
                    *value = Value::String(agent_id.clone());
                }
                migrate_value(value, mapping);
            }
        }
        _ => {}
    }
}

fn backup_file(path: &Path, migration_id: &str) -> Result<PathBuf, IdentityMigrationError> {
    let backup = path.with_extension(format!(
        "{}.identity-migration-{migration_id}.bak",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("data")
    ));
    fs::copy(path, &backup).map_err(|source| IdentityMigrationError::Io {
        path: backup.clone(),
        source,
    })?;
    let permissions = fs::metadata(path)
        .map_err(|source| IdentityMigrationError::Io {
            path: path.to_path_buf(),
            source,
        })?
        .permissions();
    fs::set_permissions(&backup, permissions).map_err(|source| IdentityMigrationError::Io {
        path: backup.clone(),
        source,
    })?;
    Ok(backup)
}

fn write_journal(path: &Path, journal: &MigrationJournal) -> Result<(), IdentityMigrationError> {
    let bytes =
        serde_json::to_vec_pretty(journal).map_err(|source| IdentityMigrationError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    write_private_file_atomic(path, &bytes).map_err(|source| IdentityMigrationError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::{IdentityMigrationPaths, apply, plan};
    use crate::store::Store;
    use std::fs;

    #[test]
    fn conversion_conserve_les_messages_et_supprime_les_alias() {
        let root =
            std::env::temp_dir().join(format!("bridget-identity-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let db = root.join("bridget.db");
        let fleet = root.join("fleet.json");
        drop(Store::open(&db).unwrap());
        let connection = rusqlite::Connection::open(&db).unwrap();
        connection
            .execute(
                "INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    "legacy-message",
                    1_i64,
                    "legacy-agent",
                    "human",
                    "bonjour",
                    "legacy-agent:human",
                ],
            )
            .unwrap();

        let paths = IdentityMigrationPaths {
            bridget_db: db.clone(),
            fleet_path: fleet,
            maicie_config: None,
        };
        let migration = plan(paths).unwrap();
        assert_eq!(migration.mapping.len(), 1);
        apply(&migration).unwrap();

        let store = Store::open(&db).unwrap();
        let messages = store.participant_messages("human", 10).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].sender,
            *migration.mapping.get("legacy-agent").unwrap()
        );
        let profiles = crate::agent_profile::AgentProfileStore::open(&db).unwrap();
        assert!(profiles.legacy_routing_map().unwrap().is_empty());
        fs::remove_dir_all(root).unwrap();
    }
}
