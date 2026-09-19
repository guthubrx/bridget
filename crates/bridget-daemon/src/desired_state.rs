//! État désiré durable des équipiers gérés par le daemon.

use bridget_transport::ResolvedAgentDefinition;
use bridget_transport::fsutil::{AtomicWritePhase, write_private_file_atomic_observed};
use bridget_transport::protocol::ProjectReference;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Schéma écrit par ce binaire. Les schémas 1 à 3 décrivaient uniquement des
/// agents persistants à reprendre. Le schéma 4 devient l'inventaire durable du
/// cycle de vie de tous les agents gérés.
pub const FLEET_SCHEMA_VERSION: u64 = 4;
pub const FLEET_SCHEMA_MIN: u64 = 1;

fn default_persistent() -> bool {
    true
}

/// État désiré du processus, indépendant de sa politique de reprise au boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DesiredLifecycleState {
    #[default]
    Running,
    Stopped,
    Decommissioned,
}

fn schema_lisible(version: u64) -> bool {
    (FLEET_SCHEMA_MIN..=FLEET_SCHEMA_VERSION).contains(&version)
}

/// Références opaques qui relient un équipier persistant à son parent.
/// Les limites restent la politique du lanceur ; elles ne deviennent pas des
/// faits durables dans `fleet.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesiredAgentLink {
    pub link_id: String,
    pub parent_instance_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_execution_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
    pub role: String,
    pub agent_path: String,
}

/// Donnée historique du produit complet, conservée sans moteur Docker.
/// Sa présence interdit la reprise sur l'hôte ; l'effacer rendrait la lecture
/// des anciennes flottes dangereusement permissive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerAgentExecution {
    pub agent_instance_id: String,
    pub generation: u64,
    pub project_id: String,
    pub binding_generation: u64,
    pub environment_epoch: u64,
    pub container_id: String,
    pub exec_id: String,
    pub cwd: PathBuf,
    pub state: ContainerAgentExecutionState,
    pub provider_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerAgentExecutionState {
    Starting,
    Running,
    Terminal,
    Lost,
}

/// Entrée durable d'un équipier dont le daemon possède le cycle de vie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesiredEquipier {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub cwd: PathBuf,
    pub command_id: String,
    pub generation: u64,
    pub created: String,
    /// Reprise automatique d'une entrée `running` au démarrage du daemon.
    /// Les anciens schémas ne contenaient que des agents persistants.
    #[serde(default = "default_persistent")]
    pub persistent: bool,
    /// Une entrée arrêtée reste visible et relançable. Une entrée
    /// décommissionnée est cachée et réserve son nom.
    #[serde(default)]
    pub lifecycle_state: DesiredLifecycleState,
    /// Définition runtime figée lors de la connexion initiale. `None` ne sert
    /// qu'à lire les anciens fichiers : leur reprise est refusée fail-closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_definition: Option<ResolvedAgentDefinition>,
    /// Domaine du protocole, persisté pour recomposer l'équipe après crash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Projet opaque conservé entre reprise et reconstruction de flotte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
    /// Parent, mandat et rôle issus du lien durable Bridget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_link: Option<DesiredAgentLink>,
    /// Exécution Docker corrélée, absente pour les agents host historiques.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_execution: Option<ContainerAgentExecution>,
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

/// Chemin de `fleet.json` dérivé du `db_path` daemon (prod vs tests).
pub fn path_for_daemon_db(db_path: &Path) -> PathBuf {
    let production_home = db_path
        .parent()
        .filter(|directory| directory.file_name().is_some_and(|name| name == "bridget"))
        .and_then(|directory| directory.parent())
        .filter(|directory| directory.file_name().is_some_and(|name| name == ".cache"))
        .and_then(|directory| directory.parent());
    production_home
        .map(|home| home.join(".config/bridget/fleet.json"))
        .unwrap_or_else(|| db_path.with_extension("fleet.json"))
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
                    "schéma fleet.json non supporté dans {}: {version} (attendu: {FLEET_SCHEMA_MIN}..={FLEET_SCHEMA_VERSION})",
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

/// Écrivain unique intra-processus des transitions durables de `fleet.json`.
///
/// T904 construit une seule instance au démarrage du daemon, appelle
/// [`Self::load_at_startup`] avant d'accepter des ordres, puis conserve cet
/// objet comme unique autorité d'écriture. Le verrou sérialise les transitions
/// concurrentes ; aucun autre processus ne doit écrire lorsque le daemon vit.
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

    /// Point d'entrée explicite de T904 pour charger l'état avant la reprise.
    pub fn load_at_startup(&self) -> Result<DesiredFleet, DesiredStateError> {
        self.load()
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

    /// Exerce la même écriture de production avec une observation des deux
    /// frontières du rename. Réservé aux crash-tests déterministes.
    pub fn persist_observed(
        &self,
        fleet: &DesiredFleet,
        observer: impl FnMut(AtomicWritePhase) -> io::Result<()>,
    ) -> Result<(), DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        self.persist_unlocked_observed(fleet, observer)
    }

    /// Réindexe la flotte par agent_id. Une collision prouve que la
    /// migration serait ambiguë et laisse le fichier intact.
    pub fn migrate_agent_ids(
        &self,
        mapping: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let mut migrated = BTreeMap::new();
        for (legacy, entry) in std::mem::take(&mut fleet.equipiers) {
            let agent_id = mapping.get(&legacy).cloned().unwrap_or(legacy);
            if migrated.insert(agent_id.clone(), entry).is_some() {
                return Err(DesiredStateError::InvalidEntry {
                    path: self.path.clone(),
                    name: agent_id,
                    reason: "collision identité de migration",
                });
            }
        }
        fleet.equipiers = migrated;
        self.persist_unlocked(&fleet)
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

    /// Applique une transition explicite de cycle de vie et retourne l'entrée
    /// mise à jour. L'absence n'est jamais transformée en création implicite.
    pub fn set_lifecycle_state(
        &self,
        name: &str,
        lifecycle_state: DesiredLifecycleState,
    ) -> Result<Option<DesiredEquipier>, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let Some(entry) = fleet.equipiers.get_mut(name) else {
            return Ok(None);
        };
        if entry.lifecycle_state != lifecycle_state {
            entry.lifecycle_state = lifecycle_state;
            let updated = entry.clone();
            self.persist_unlocked(&fleet)?;
            return Ok(Some(updated));
        }
        Ok(Some(entry.clone()))
    }

    /// Marque une génération arrêtée uniquement si elle possède encore
    /// l'entrée. Une relance échouée ne peut ainsi effacer l'ancienne
    /// définition `stopped` qu'elle tentait de remplacer.
    pub fn mark_stopped_if_generation(
        &self,
        name: &str,
        command_id: &str,
        generation: u64,
    ) -> Result<bool, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let Some(entry) = fleet.equipiers.get_mut(name) else {
            return Ok(false);
        };
        if entry.command_id != command_id
            || entry.generation != generation
            || entry.lifecycle_state == DesiredLifecycleState::Decommissioned
        {
            return Ok(false);
        }
        if entry.lifecycle_state != DesiredLifecycleState::Stopped {
            entry.lifecycle_state = DesiredLifecycleState::Stopped;
            self.persist_unlocked(&fleet)?;
        }
        Ok(true)
    }

    /// Les agents non persistants ne sont pas repris après un redémarrage,
    /// mais leur identité logique demeure consultable comme arrêtée.
    pub fn stop_non_persistent_running(&self) -> Result<Vec<String>, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let mut changed = Vec::new();
        for (name, entry) in &mut fleet.equipiers {
            if !entry.persistent && entry.lifecycle_state == DesiredLifecycleState::Running {
                entry.lifecycle_state = DesiredLifecycleState::Stopped;
                changed.push(name.clone());
            }
        }
        if !changed.is_empty() {
            self.persist_unlocked(&fleet)?;
        }
        Ok(changed)
    }

    /// Met à jour le domain d'une entrée existante sous le même verrou que
    /// `upsert`/`remove`. Un load+persist disjoint écraserait les insertions
    /// concurrentes.
    pub fn set_domain(&self, name: &str, domain: Option<&str>) -> Result<bool, DesiredStateError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut fleet = self.load_unlocked()?;
        let Some(entry) = fleet.equipiers.get_mut(name) else {
            return Ok(false);
        };
        let next = domain
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        if entry.domain == next {
            return Ok(false);
        }
        entry.domain = next;
        self.persist_unlocked(&fleet)?;
        Ok(true)
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
        if !found_schema.is_some_and(schema_lisible) {
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
        self.persist_unlocked_observed(fleet, |_| Ok(()))
    }

    fn persist_unlocked_observed(
        &self,
        fleet: &DesiredFleet,
        observer: impl FnMut(AtomicWritePhase) -> io::Result<()>,
    ) -> Result<(), DesiredStateError> {
        let mut fleet = fleet.clone();
        fleet.schema = FLEET_SCHEMA_VERSION;
        validate_fleet(&self.path, &fleet)?;
        let mut bytes =
            serde_json::to_vec_pretty(&fleet).map_err(|source| DesiredStateError::InvalidJson {
                path: self.path.clone(),
                source,
            })?;
        bytes.push(b'\n');
        write_private_file_atomic_observed(&self.path, &bytes, observer).map_err(|source| {
            DesiredStateError::Io {
                path: self.path.clone(),
                source,
            }
        })
    }
}

/// Validation O(n) de la frontière fichier, n étant le nombre d'entrées.
fn validate_fleet(path: &Path, fleet: &DesiredFleet) -> Result<(), DesiredStateError> {
    if !schema_lisible(fleet.schema) {
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
        } else if equipier.agent_link.as_ref().is_some_and(|link| {
            link.link_id.trim().is_empty()
                || link.parent_instance_id.trim().is_empty()
                || link.role.trim().is_empty()
                || link.agent_path.trim().is_empty()
        }) {
            Some("lien agent incomplet")
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
    use std::os::fd::RawFd;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    const CRASH_CHILD_ENV: &str = "BRIDGET_T902_CRASH_CHILD";
    const CRASH_PATH_ENV: &str = "BRIDGET_T902_CRASH_PATH";
    const BARRIER_READY_FD: RawFd = 110;
    const BARRIER_RELEASE_FD: RawFd = 111;

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
            persistent: true,
            lifecycle_state: DesiredLifecycleState::Running,
            resolved_definition: None,
            domain: None,
            project: None,
            agent_link: None,
            runtime_execution: None,
        }
    }

    struct BarrierChild {
        child: std::process::Child,
        ready: RawFd,
        release: RawFd,
    }

    impl BarrierChild {
        fn wait_phase(&self, phase: AtomicWritePhase) {
            let mut poll = libc::pollfd {
                fd: self.ready,
                events: libc::POLLIN,
                revents: 0,
            };
            assert_eq!(unsafe { libc::poll(&mut poll, 1, 5_000) }, 1);
            let mut byte = 0_u8;
            assert_eq!(
                unsafe { libc::read(self.ready, (&mut byte as *mut u8).cast(), 1) },
                1
            );
            let expected = match phase {
                AtomicWritePhase::BeforeRename => b'B',
                AtomicWritePhase::AfterRename => b'A',
            };
            assert_eq!(byte, expected);
        }

        fn release_barrier(&self) {
            assert_eq!(
                unsafe { libc::write(self.release, [b'R'].as_ptr().cast(), 1) },
                1
            );
        }
    }

    impl Drop for BarrierChild {
        fn drop(&mut self) {
            unsafe {
                libc::close(self.ready);
                libc::close(self.release);
            }
        }
    }

    fn spawn_crash_writer(path: &Path) -> BarrierChild {
        let current_exe = std::env::current_exe().unwrap();
        assert_ne!(
            current_exe.file_name().and_then(|name| name.to_str()),
            Some("firefox")
        );
        let mut ready_pipe = [-1; 2];
        let mut release_pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(ready_pipe.as_mut_ptr()) }, 0);
        assert_eq!(unsafe { libc::pipe(release_pipe.as_mut_ptr()) }, 0);

        let mut command = Command::new(current_exe);
        command
            .arg("--exact")
            .arg("desired_state::tests::crash_writer_child")
            .arg("--ignored")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CRASH_CHILD_ENV, "1")
            .env(CRASH_PATH_ENV, path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            command.pre_exec(move || {
                if libc::dup2(ready_pipe[1], BARRIER_READY_FD) < 0
                    || libc::dup2(release_pipe[0], BARRIER_RELEASE_FD) < 0
                {
                    return Err(io::Error::last_os_error());
                }
                for fd in ready_pipe.into_iter().chain(release_pipe) {
                    if fd != BARRIER_READY_FD && fd != BARRIER_RELEASE_FD {
                        libc::close(fd);
                    }
                }
                Ok(())
            });
        }
        let child = command.spawn().unwrap();
        unsafe {
            libc::close(ready_pipe[1]);
            libc::close(release_pipe[0]);
        }
        BarrierChild {
            child,
            ready: ready_pipe[0],
            release: release_pipe[1],
        }
    }

    fn terminate_test_child(process: &mut BarrierChild) {
        // Le PID vient du processus de test enfant créé par spawn_crash_writer :
        // ce signal ne peut viser ni Firefox ni un processus étranger.
        assert!(process.child.try_wait().unwrap().is_none());
        let signal_result = unsafe { libc::kill(process.child.id() as libc::pid_t, libc::SIGTERM) };
        assert_eq!(signal_result, 0);
        assert!(!process.child.wait().unwrap().success());
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
    fn domain_est_ecrit_dans_fleet_json_apres_upsert() {
        let root = test_root("domain");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        let mut entry = equipier("command-domain", 1);
        entry.domain = Some("bridget".to_string());
        store.upsert("cursor6".to_string(), entry).unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"domain\": \"bridget\""), "{raw}");
        assert_eq!(
            store.load().unwrap().equipiers["cursor6"].domain.as_deref(),
            Some("bridget")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn schema_ancien_devient_running_persistent_par_defaut() {
        let root = test_root("schema-legacy-lifecycle");
        let path = root.join("fleet.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &path,
            r#"{
  "schema": 3,
  "equipiers": {
    "ancien": {
      "type": "codex",
      "cwd": "/tmp/projet",
      "command_id": "legacy-command",
      "generation": 7,
      "created": "2026-08-22T20:14:00Z"
    }
  }
}"#,
        )
        .unwrap();
        let entry = DesiredStateStore::at_path(&path)
            .load()
            .unwrap()
            .equipiers
            .remove("ancien")
            .unwrap();
        assert!(entry.persistent);
        assert_eq!(entry.lifecycle_state, DesiredLifecycleState::Running);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transitions_de_cycle_de_vie_sont_atomiques_et_bornees() {
        let root = test_root("lifecycle");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        let entry = equipier("command-lifecycle", 8);
        store.upsert("agent".to_string(), entry).unwrap();

        assert!(
            store
                .mark_stopped_if_generation("agent", "command-lifecycle", 8)
                .unwrap()
        );
        assert_eq!(
            store.load().unwrap().equipiers["agent"].lifecycle_state,
            DesiredLifecycleState::Stopped
        );
        assert!(
            !store
                .mark_stopped_if_generation("agent", "autre-command", 8)
                .unwrap()
        );
        let retired = store
            .set_lifecycle_state("agent", DesiredLifecycleState::Decommissioned)
            .unwrap()
            .unwrap();
        assert_eq!(
            retired.lifecycle_state,
            DesiredLifecycleState::Decommissioned
        );
        assert!(
            !store
                .mark_stopped_if_generation("agent", "command-lifecycle", 8)
                .unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn redemarrage_arrete_uniquement_les_non_persistants_running() {
        let root = test_root("non-persistent");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        let mut ephemeral = equipier("ephemeral", 1);
        ephemeral.persistent = false;
        store.upsert("ephemeral".to_string(), ephemeral).unwrap();
        let mut persistent = equipier("persistent", 2);
        persistent.persistent = true;
        store.upsert("persistent".to_string(), persistent).unwrap();

        assert_eq!(
            store.stop_non_persistent_running().unwrap(),
            vec!["ephemeral".to_string()]
        );
        let fleet = store.load().unwrap();
        assert_eq!(
            fleet.equipiers["ephemeral"].lifecycle_state,
            DesiredLifecycleState::Stopped
        );
        assert_eq!(
            fleet.equipiers["persistent"].lifecycle_state,
            DesiredLifecycleState::Running
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lien_agent_optionnel_survit_au_roundtrip_fleet() {
        let root = test_root("agent-link");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        let mut entry = equipier("command-link", 1);
        entry.agent_link = Some(DesiredAgentLink {
            link_id: "link-1".to_string(),
            parent_instance_id: "instance-parent".to_string(),
            parent_execution_id: Some("execution-parent".to_string()),
            objective_id: Some("objective-1".to_string()),
            delegation_id: Some("delegation-1".to_string()),
            project: None,
            role: "verification".to_string(),
            agent_path: "instance-parent/instance-child".to_string(),
        });
        store.upsert("child".to_string(), entry).unwrap();
        let loaded = store.load().unwrap();
        let link = loaded.equipiers["child"].agent_link.as_ref().unwrap();
        assert_eq!(loaded.schema, FLEET_SCHEMA_VERSION);
        assert_eq!(link.parent_instance_id, "instance-parent");
        assert_eq!(link.delegation_id.as_deref(), Some("delegation-1"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fleet_json_schema_1_sans_domain_se_relit() {
        let root = test_root("ancien");
        let path = root.join("fleet.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &path,
            r#"{
  "schema": 1,
  "equipiers": {
    "codex-1": {
      "type": "codex",
      "cwd": "/tmp/projet",
      "command_id": "command-1",
      "generation": 1,
      "created": "2026-08-22T20:14:00Z"
    }
  }
}
"#,
        )
        .unwrap();
        let loaded = DesiredStateStore::at_path(&path).load().unwrap();
        assert_eq!(loaded.schema, 1);
        assert_eq!(loaded.equipiers["codex-1"].domain, None);
        DesiredStateStore::at_path(&path)
            .set_domain("codex-1", Some("bridget"))
            .unwrap();
        let rewritten = fs::read_to_string(&path).unwrap();
        assert!(
            rewritten.contains(&format!("\"schema\": {FLEET_SCHEMA_VERSION}")),
            "{rewritten}"
        );
        assert!(rewritten.contains("\"domain\": \"bridget\""), "{rewritten}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn set_domain_ne_ecrase_pas_un_upsert_concurrent() {
        let root = test_root("course");
        let path = root.join("fleet.json");
        let store = std::sync::Arc::new(DesiredStateStore::at_path(&path));
        store
            .upsert("agent-x".to_string(), equipier("command-x", 1))
            .unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let writer = {
            let store = std::sync::Arc::clone(&store);
            let barrier = std::sync::Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                for index in 0..80_u64 {
                    store
                        .upsert(format!("agent-z{index}"), equipier("command-z", index + 2))
                        .unwrap();
                }
            })
        };
        let updater = {
            let store = std::sync::Arc::clone(&store);
            let barrier = std::sync::Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                for _ in 0..80 {
                    store.set_domain("agent-x", Some("bridget")).unwrap();
                }
            })
        };
        writer.join().unwrap();
        updater.join().unwrap();
        let loaded = store.load().unwrap();
        assert!(loaded.equipiers.contains_key("agent-x"));
        assert_eq!(
            loaded.equipiers["agent-x"].domain.as_deref(),
            Some("bridget")
        );
        for index in 0..80_u64 {
            assert!(
                loaded.equipiers.contains_key(&format!("agent-z{index}")),
                "agent-z{index} perdu par course load+persist"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsupported_or_invalid_schema_is_refused_with_the_path() {
        let root = test_root("version");
        let path = root.join("fleet.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, r#"{"schema":5,"equipiers":{}}"#).unwrap();
        let error = DesiredStateStore::at_path(&path).load().unwrap_err();

        assert!(matches!(
            error,
            DesiredStateError::UnsupportedSchema { found: Some(5), .. }
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

        let mut child = spawn_crash_writer(&path);
        child.wait_phase(AtomicWritePhase::BeforeRename);
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
        let store = DesiredStateStore::at_path(&path);
        store
            .upsert("codex-1".to_string(), equipier("stable-command", 1))
            .unwrap();

        let mut child = spawn_crash_writer(&path);
        child.wait_phase(AtomicWritePhase::BeforeRename);
        child.release_barrier();
        child.wait_phase(AtomicWritePhase::AfterRename);
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
        replacement
            .equipiers
            .insert("codex-1".to_string(), equipier("replacement-command", 2));
        store
            .persist_observed(&replacement, |phase| {
                let signal = match phase {
                    AtomicWritePhase::BeforeRename => b'B',
                    AtomicWritePhase::AfterRename => b'A',
                };
                if unsafe { libc::write(BARRIER_READY_FD, (&signal as *const u8).cast(), 1) } != 1 {
                    return Err(io::Error::last_os_error());
                }
                let mut release = 0_u8;
                if unsafe { libc::read(BARRIER_RELEASE_FD, (&mut release as *mut u8).cast(), 1) }
                    != 1
                {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "barrière fermée",
                    ));
                }
                Ok(())
            })
            .unwrap();
        unreachable!("le parent doit interrompre le processus à une barrière");
    }
}
