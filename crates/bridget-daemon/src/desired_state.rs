//! État désiré durable des équipiers gérés par le daemon.

use bridget_transport::fsutil::write_private_file_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const FLEET_SCHEMA_VERSION: u64 = 1;

/// Entrée persistante d'un équipier que le daemon doit maintenir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesiredEquipier {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub cwd: PathBuf,
    pub command_id: String,
    pub generation: u64,
    pub created: String,
}

/// Contenu versionné de `fleet.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesiredFleet {
    pub schema: u64,
    pub equipiers: BTreeMap<String, DesiredEquipier>,
}

impl Default for DesiredFleet {
    fn default() -> Self {
        Self {
            schema: FLEET_SCHEMA_VERSION,
            equipiers: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum DesiredStateError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    InvalidJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    UnsupportedSchema {
        path: PathBuf,
        found: Option<u64>,
    },
    InvalidEntry {
        path: PathBuf,
        name: String,
        reason: &'static str,
    },
}

impl fmt::Display for DesiredStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    formatter,
                    "état désiré inaccessible {}: {source}",
                    path.display()
                )
            }
            Self::InvalidJson { path, source } => {
                write!(
                    formatter,
                    "état désiré invalide {}: {source}",
                    path.display()
                )
            }
            Self::UnsupportedSchema { path, found } => match found {
                Some(version) => write!(
                    formatter,
                    "schéma fleet.json non supporté dans {}: {version} (attendu: {FLEET_SCHEMA_VERSION})",
                    path.display()
                ),
                None => write!(
                    formatter,
                    "schéma fleet.json absent ou invalide dans {}",
                    path.display()
                ),
            },
            Self::InvalidEntry { path, name, reason } => write!(
                formatter,
                "équipier persistant invalide '{name}' dans {}: {reason}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DesiredStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidJson { source, .. } => Some(source),
            Self::UnsupportedSchema { .. } | Self::InvalidEntry { .. } => None,
        }
    }
}

/// Propriétaire intra-processus des transitions durables de `fleet.json`.
///
/// Le daemon conserve une seule instance de ce store. Le verrou sérialise les
/// lectures-modifications-écritures concurrentes ; aucun autre processus ne
/// doit écrire le fichier lorsque le daemon fonctionne.
pub struct DesiredStateStore {
    path: PathBuf,
    transition_lock: Mutex<()>,
}

impl DesiredStateStore {
    /// Construit le chemin public `~/.config/bridget/fleet.json`.
    pub fn for_home(home: &Path) -> Self {
        Self::at_path(home.join(".config").join("bridget").join("fleet.json"))
    }

    /// Construit un store à un emplacement explicite, notamment pour les tests.
    pub fn at_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            transition_lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Charge et valide le fichier en O(n), n étant le nombre d'équipiers.
    /// L'absence du fichier au premier démarrage représente une flotte vide.
    pub fn load(&self) -> Result<DesiredFleet, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        self.load_unlocked()
    }

    /// Remplace durablement l'état complet en O(n).
    pub fn persist(&self, fleet: &DesiredFleet) -> Result<(), DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        self.persist_unlocked(fleet)
    }

    /// Insère une génération sous sa clé stable, puis retourne l'ancienne.
    pub fn upsert(
        &self,
        name: String,
        equipier: DesiredEquipier,
    ) -> Result<Option<DesiredEquipier>, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let previous = fleet.equipiers.insert(name, equipier);
        self.persist_unlocked(&fleet)?;
        Ok(previous)
    }

    /// Retire une entrée avant de confirmer l'arrêt à l'appelant.
    pub fn remove(&self, name: &str) -> Result<Option<DesiredEquipier>, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let removed = fleet.equipiers.remove(name);
        if removed.is_some() {
            self.persist_unlocked(&fleet)?;
        }
        Ok(removed)
    }

    fn load_unlocked(&self) -> Result<DesiredFleet, DesiredStateError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(DesiredFleet::default());
            }
            Err(source) => {
                return Err(DesiredStateError::Io {
                    path: self.path.clone(),
                    source,
                });
            }
        };
        let raw: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|source| DesiredStateError::InvalidJson {
                path: self.path.clone(),
                source,
            })?;
        let found_schema = raw.get("schema").and_then(serde_json::Value::as_u64);
        if found_schema != Some(FLEET_SCHEMA_VERSION) {
            return Err(DesiredStateError::UnsupportedSchema {
                path: self.path.clone(),
                found: found_schema,
            });
        }
        let fleet: DesiredFleet =
            serde_json::from_value(raw).map_err(|source| DesiredStateError::InvalidJson {
                path: self.path.clone(),
                source,
            })?;
        validate_fleet(&self.path, &fleet)?;
        Ok(fleet)
    }

    fn persist_unlocked(&self, fleet: &DesiredFleet) -> Result<(), DesiredStateError> {
        validate_fleet(&self.path, fleet)?;
        let mut bytes =
            serde_json::to_vec_pretty(fleet).map_err(|source| DesiredStateError::InvalidJson {
                path: self.path.clone(),
                source,
            })?;
        bytes.push(b'\n');
        write_private_file_atomic(&self.path, &bytes).map_err(|source| DesiredStateError::Io {
            path: self.path.clone(),
            source,
        })
    }
}

/// Validation O(n) de la frontière fichier, n étant le nombre d'entrées.
fn validate_fleet(path: &Path, fleet: &DesiredFleet) -> Result<(), DesiredStateError> {
    if fleet.schema != FLEET_SCHEMA_VERSION {
        return Err(DesiredStateError::UnsupportedSchema {
            path: path.to_path_buf(),
            found: Some(fleet.schema),
        });
    }
    for (name, equipier) in &fleet.equipiers {
        let reason = if name.trim().is_empty() {
            Some("nom vide")
        } else if equipier.agent_type.trim().is_empty() {
            Some("type vide")
        } else if !equipier.cwd.is_absolute() {
            Some("cwd non absolu")
        } else if equipier.command_id.trim().is_empty() {
            Some("command_id vide")
        } else if equipier.generation == 0 {
            Some("génération nulle")
        } else if equipier.created.trim().is_empty() {
            Some("date de création vide")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(DesiredStateError::InvalidEntry {
                path: path.to_path_buf(),
                name: name.clone(),
                reason,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const CRASH_CHILD_ENV: &str = "BRIDGET_T902_CRASH_CHILD";
    const CRASH_MODE_ENV: &str = "BRIDGET_T902_CRASH_MODE";
    const CRASH_PATH_ENV: &str = "BRIDGET_T902_CRASH_PATH";
    const CRASH_READY_ENV: &str = "BRIDGET_T902_CRASH_READY";

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t902-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn equipier(command_id: &str, generation: u64) -> DesiredEquipier {
        DesiredEquipier {
            agent_type: "codex".to_string(),
            cwd: PathBuf::from("/tmp/projet"),
            command_id: command_id.to_string(),
            generation,
            created: "2026-08-22T20:14:00Z".to_string(),
        }
    }

    fn spawn_crash_writer(
        path: &Path,
        mode: &str,
        ready_path: Option<&Path>,
    ) -> std::process::Child {
        let current_exe = std::env::current_exe().unwrap();
        assert_ne!(
            current_exe.file_name().and_then(|name| name.to_str()),
            Some("firefox")
        );
        let mut command = Command::new(current_exe);
        command
            .arg("--exact")
            .arg("desired_state::tests::crash_writer_child")
            .arg("--ignored")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CRASH_CHILD_ENV, "1")
            .env(CRASH_MODE_ENV, mode)
            .env(CRASH_PATH_ENV, path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(ready_path) = ready_path {
            command.env(CRASH_READY_ENV, ready_path);
        }
        command.spawn().unwrap()
    }

    fn terminate_test_child(child: &mut std::process::Child) {
        // Le PID vient du processus de test enfant créé par spawn_crash_writer :
        // ce signal ne peut viser ni Firefox ni un processus étranger.
        let signal_result = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };
        assert_eq!(signal_result, 0);
        assert!(!child.wait().unwrap().success());
    }

    #[test]
    fn fleet_schema_one_roundtrips_with_private_permissions() {
        let root = test_root("schema");
        let path = root.join("config/bridget/fleet.json");
        let store = DesiredStateStore::at_path(&path);
        store
            .upsert("codex-1".to_string(), equipier("command-1", 4))
            .unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.schema, FLEET_SCHEMA_VERSION);
        assert_eq!(loaded.equipiers["codex-1"].generation, 4);
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsupported_or_invalid_schema_is_refused_with_the_path() {
        let root = test_root("version");
        let path = root.join("fleet.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, r#"{"schema":2,"equipiers":{}}"#).unwrap();
        let error = DesiredStateStore::at_path(&path).load().unwrap_err();

        assert!(matches!(
            error,
            DesiredStateError::UnsupportedSchema { found: Some(2), .. }
        ));
        assert!(error.to_string().contains(path.to_str().unwrap()));
        fs::write(&path, r#"{"equipiers":{}}"#).unwrap();
        assert!(matches!(
            DesiredStateStore::at_path(&path).load().unwrap_err(),
            DesiredStateError::UnsupportedSchema { found: None, .. }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn removing_an_entry_is_durable_before_returning() {
        let root = test_root("remove");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        store
            .upsert("codex-1".to_string(), equipier("command-1", 1))
            .unwrap();
        store
            .upsert("claude-1".to_string(), equipier("command-2", 2))
            .unwrap();

        assert_eq!(store.remove("codex-1").unwrap().unwrap().generation, 1);
        drop(store);
        let reopened = DesiredStateStore::at_path(&path).load().unwrap();
        assert!(!reopened.equipiers.contains_key("codex-1"));
        assert_eq!(reopened.equipiers["claude-1"].command_id, "command-2");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn crash_before_rename_keeps_the_previous_fleet() {
        let root = test_root("crash");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        store
            .upsert("codex-1".to_string(), equipier("stable-command", 1))
            .unwrap();

        let mut child = spawn_crash_writer(&path, "before_rename", None);

        let deadline = Instant::now() + Duration::from_secs(10);
        let temporary_seen = loop {
            let seen = fs::read_dir(&root)
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().into_string().ok())
                .any(|name| name.starts_with(".fleet.json.") && name.ends_with(".tmp"));
            if seen {
                break true;
            }
            if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
                break false;
            }
            thread::sleep(Duration::from_millis(1));
        };
        assert!(
            temporary_seen,
            "le processus enfant n'a pas atteint le temporaire"
        );

        terminate_test_child(&mut child);

        let loaded = store.load().unwrap();
        assert_eq!(loaded.equipiers.len(), 1);
        assert_eq!(loaded.equipiers["codex-1"].command_id, "stable-command");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn crash_after_rename_exposes_the_new_fleet_on_reopen() {
        let root = test_root("crash-after");
        let path = root.join("fleet.json");
        let ready_path = root.join("after-rename.ready");
        let store = DesiredStateStore::at_path(&path);
        store
            .upsert("codex-1".to_string(), equipier("stable-command", 1))
            .unwrap();

        let mut child = spawn_crash_writer(&path, "after_rename", Some(&ready_path));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready_path.exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "le writer est sorti avant la barrière post-rename"
            );
            assert!(
                Instant::now() < deadline,
                "la barrière post-rename n'a pas été atteinte"
            );
            thread::sleep(Duration::from_millis(1));
        }
        terminate_test_child(&mut child);

        let loaded = store.load().unwrap();
        assert_eq!(loaded.equipiers.len(), 1);
        assert_eq!(
            loaded.equipiers["codex-1"].command_id,
            "replacement-command"
        );
        assert_eq!(loaded.equipiers["codex-1"].generation, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "sous-processus contrôlé par crash_before_rename_keeps_the_previous_fleet"]
    fn crash_writer_child() {
        if std::env::var(CRASH_CHILD_ENV).as_deref() != Ok("1") {
            return;
        }
        let path = PathBuf::from(std::env::var_os(CRASH_PATH_ENV).unwrap());
        let store = DesiredStateStore::at_path(path);
        let mut replacement = DesiredFleet::default();
        let mut entry = equipier("replacement-command", 2);
        if std::env::var(CRASH_MODE_ENV).as_deref() == Ok("before_rename") {
            // Une charge volumineuse maintient réellement le processus entre
            // création du temporaire et rename, sans hook de production.
            entry.agent_type = "x".repeat(128 * 1024 * 1024);
        }
        replacement.equipiers.insert("codex-1".to_string(), entry);
        store.persist(&replacement).unwrap();
        if std::env::var(CRASH_MODE_ENV).as_deref() == Ok("after_rename") {
            let ready_path = PathBuf::from(std::env::var_os(CRASH_READY_ENV).unwrap());
            fs::write(ready_path, b"renamed-and-synced\n").unwrap();
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        panic!("le parent devait interrompre le processus avant le rename");
    }
}
