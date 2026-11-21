//! État désiré durable des équipiers gérés par le daemon.

use bridget_transport::ResolvedAgentDefinition;
use bridget_transport::fsutil::{AtomicWritePhase, write_private_file_atomic_observed};
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::ProjectReference;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Schéma écrit par ce binaire. Les schémas 1 à 3 décrivaient uniquement des
/// agents persistants à reprendre. Le schéma 4 devient l'inventaire durable du
/// cycle de vie de tous les agents gérés. Le schéma 5 conserve le placement
/// distant ; aucun ancien binaire ne doit reprendre ce store comme local.
pub const FLEET_SCHEMA_VERSION: u64 = 5;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    dev: u64,
    ino: u64,
    len: u64,
    mtime_sec: i64,
    mtime_nsec: i64,
    ctime_sec: i64,
    ctime_nsec: i64,
}

#[derive(Default)]
struct DesiredStateCache {
    stamp: Option<FileStamp>,
    fleet: Option<Arc<DesiredFleet>>,
}

#[derive(Deserialize)]
struct FleetSchemaHeader {
    #[serde(default)]
    schema: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct WireDesiredFleet<'a> {
    schema: u64,
    equipiers: &'a BTreeMap<String, DesiredEquipier>,
}

fn file_stamp(path: &Path) -> Result<Option<FileStamp>, DesiredStateError> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(DesiredStateError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    #[cfg(unix)]
    let stamp = FileStamp {
        dev: metadata.dev(),
        ino: metadata.ino(),
        len: metadata.size(),
        mtime_sec: metadata.mtime(),
        mtime_nsec: metadata.mtime_nsec(),
        ctime_sec: metadata.ctime(),
        ctime_nsec: metadata.ctime_nsec(),
    };
    #[cfg(not(unix))]
    let stamp = {
        let modified = metadata
            .modified()
            .ok()
            .and_then(|ts| ts.duration_since(std::time::UNIX_EPOCH).ok())
            .unwrap_or_default();
        FileStamp {
            dev: 0,
            ino: 0,
            len: metadata.len(),
            mtime_sec: modified.as_secs() as i64,
            mtime_nsec: modified.subsec_nanos() as i64,
            ctime_sec: 0,
            ctime_nsec: 0,
        }
    };
    Ok(Some(stamp))
}

/// Écrivain unique intra-processus des transitions durables de `fleet.json`.
///
/// T904 construit une seule instance au démarrage du daemon, appelle
/// [`Self::load_at_startup`] avant d'accepter des ordres, puis conserve cet
/// objet comme unique autorité d'écriture. Le verrou sérialise les transitions
/// concurrentes ; aucun autre processus ne doit écrire lorsque le daemon vit.
/// Un cache mémoire validé par empreinte fichier évite de relire et analyser
/// l'intégralité de `fleet.json` à chaque requête de lecture.
pub struct DesiredStateStore {
    path: PathBuf,
    state: Mutex<DesiredStateCache>,
    /// Nombre d'écritures réussies de `fleet.json`, pour mesurer le coût des lots en test.
    #[cfg(test)]
    writes: std::sync::atomic::AtomicUsize,
    /// Nombre de lectures disque de `fleet.json`, pour vérifier le cache en test.
    #[cfg(test)]
    reads: std::sync::atomic::AtomicUsize,
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
            state: Mutex::new(DesiredStateCache::default()),
            #[cfg(test)]
            writes: std::sync::atomic::AtomicUsize::new(0),
            #[cfg(test)]
            reads: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn write_count(&self) -> usize {
        self.writes.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn read_count(&self) -> usize {
        self.reads.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Point d'entrée explicite de T904 pour charger l'état avant la reprise.
    pub fn load_at_startup(&self) -> Result<DesiredFleet, DesiredStateError> {
        self.load()
    }

    /// Retourne un instantané partagé de la flotte, servi depuis le cache mémoire
    /// lorsque l'empreinte fichier est inchangée. Complexité : O(1) sur hit, O(n) sur miss.
    pub fn load_shared(&self) -> Result<Arc<DesiredFleet>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        self.load_cached(&mut cache)
    }

    /// Charge et valide le fichier en O(n), n étant le nombre d'équipiers.
    /// L'absence du fichier au premier démarrage représente une flotte vide.
    pub fn load(&self) -> Result<DesiredFleet, DesiredStateError> {
        Ok((*self.load_shared()?).clone())
    }

    /// Retourne une seule entrée par nom sans cloner toute la flotte.
    pub fn get(&self, name: &str) -> Result<Option<DesiredEquipier>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let fleet = self.load_cached(&mut cache)?;
        Ok(fleet.equipiers.get(name).cloned())
    }

    /// Vérifie la présence d'un nom sans cloner la flotte.
    pub fn contains_key(&self, name: &str) -> Result<bool, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let fleet = self.load_cached(&mut cache)?;
        Ok(fleet.equipiers.contains_key(name))
    }

    /// Retourne le domaine d'un équipier sans cloner la flotte.
    pub fn domain(&self, name: &str) -> Result<Option<String>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let fleet = self.load_cached(&mut cache)?;
        Ok(fleet
            .equipiers
            .get(name)
            .and_then(|entry| entry.domain.clone()))
    }

    /// Remplace durablement l'état complet en O(n).
    pub fn persist(&self, fleet: &DesiredFleet) -> Result<(), DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        cache.stamp = None;
        cache.fleet = None;
        self.persist_unlocked(fleet)
    }

    /// Exerce la même écriture de production avec une observation des deux
    /// frontières du rename. Réservé aux crash-tests déterministes.
    pub fn persist_observed(
        &self,
        fleet: &DesiredFleet,
        observer: impl FnMut(AtomicWritePhase) -> io::Result<()>,
    ) -> Result<(), DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        cache.stamp = None;
        cache.fleet = None;
        self.persist_unlocked_observed(fleet, observer)
    }

    /// Réindexe la flotte par agent_id. Une collision prouve que la
    /// migration serait ambiguë et laisse le fichier intact.
    pub fn migrate_agent_ids(
        &self,
        mapping: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let mut fleet = self.take_owned_fleet(&mut cache)?;
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
        fleet.schema = FLEET_SCHEMA_VERSION;
        self.persist_unlocked(&fleet)?;
        self.store_cache(&mut cache, fleet);
        Ok(())
    }

    /// Insère une génération sous sa clé stable, puis retourne l'ancienne.
    pub fn upsert(
        &self,
        name: String,
        equipier: DesiredEquipier,
    ) -> Result<Option<DesiredEquipier>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        let previous = fleet.equipiers.insert(name, equipier);
        fleet.schema = FLEET_SCHEMA_VERSION;
        self.persist_unlocked(&fleet)?;
        self.store_cache(&mut cache, fleet);
        Ok(previous)
    }

    /// Retire une entrée avant de confirmer l'arrêt à l'appelant.
    pub fn remove(&self, name: &str) -> Result<Option<DesiredEquipier>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        if !self.load_cached(&mut cache)?.equipiers.contains_key(name) {
            return Ok(None);
        }
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        let removed = fleet.equipiers.remove(name);
        if removed.is_some() {
            fleet.schema = FLEET_SCHEMA_VERSION;
            self.persist_unlocked(&fleet)?;
            self.store_cache(&mut cache, fleet);
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
        let mut cache = self.state.lock().recover_poison();
        {
            let current = self.load_cached(&mut cache)?;
            let Some(entry) = current.equipiers.get(name) else {
                return Ok(None);
            };
            if entry.lifecycle_state == lifecycle_state {
                return Ok(Some(entry.clone()));
            }
        }
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        let Some(entry) = fleet.equipiers.get_mut(name) else {
            return Ok(None);
        };
        entry.lifecycle_state = lifecycle_state;
        let updated = entry.clone();
        fleet.schema = FLEET_SCHEMA_VERSION;
        self.persist_unlocked(&fleet)?;
        self.store_cache(&mut cache, fleet);
        Ok(Some(updated))
    }

    /// Applique un état de cycle de vie à plusieurs entrées en une lecture et au plus
    /// une écriture. Les noms absents sont ignorés. Rend les noms des entrées présentes.
    /// Complexité : O(F + d) — F entrées de flotte, d noms demandés.
    pub fn set_lifecycle_states(
        &self,
        names: &[&str],
        lifecycle_state: DesiredLifecycleState,
    ) -> Result<Vec<String>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let mut present = Vec::new();
        let mut changed = false;
        {
            let current = self.load_cached(&mut cache)?;
            for name in names {
                if let Some(entry) = current.equipiers.get(*name) {
                    changed |= entry.lifecycle_state != lifecycle_state;
                    present.push((*name).to_string());
                }
            }
        }
        if !changed {
            return Ok(present);
        }
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        for name in &present {
            if let Some(entry) = fleet.equipiers.get_mut(name) {
                entry.lifecycle_state = lifecycle_state;
            }
        }
        fleet.schema = FLEET_SCHEMA_VERSION;
        self.persist_unlocked(&fleet)?;
        self.store_cache(&mut cache, fleet);
        Ok(present)
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
        let mut cache = self.state.lock().recover_poison();
        {
            let current = self.load_cached(&mut cache)?;
            let Some(entry) = current.equipiers.get(name) else {
                return Ok(false);
            };
            if entry.command_id != command_id
                || entry.generation != generation
                || entry.lifecycle_state == DesiredLifecycleState::Decommissioned
            {
                return Ok(false);
            }
            if entry.lifecycle_state == DesiredLifecycleState::Stopped {
                return Ok(true);
            }
        }
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        if let Some(entry) = fleet.equipiers.get_mut(name) {
            entry.lifecycle_state = DesiredLifecycleState::Stopped;
            fleet.schema = FLEET_SCHEMA_VERSION;
            self.persist_unlocked(&fleet)?;
            self.store_cache(&mut cache, fleet);
        }
        Ok(true)
    }

    /// Les agents non persistants ne sont pas repris après un redémarrage,
    /// mais leur identité logique demeure consultable comme arrêtée.
    pub fn stop_non_persistent_running(&self) -> Result<Vec<String>, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let changed: Vec<String> = {
            let current = self.load_cached(&mut cache)?;
            current
                .equipiers
                .iter()
                .filter(|(_, entry)| {
                    !entry.persistent
                        && entry.remote_placement.is_none()
                        && entry.lifecycle_state == DesiredLifecycleState::Running
                })
                .map(|(name, _)| name.clone())
                .collect()
        };
        if changed.is_empty() {
            return Ok(changed);
        }
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        for name in &changed {
            if let Some(entry) = fleet.equipiers.get_mut(name) {
                entry.lifecycle_state = DesiredLifecycleState::Stopped;
            }
        }
        fleet.schema = FLEET_SCHEMA_VERSION;
        self.persist_unlocked(&fleet)?;
        self.store_cache(&mut cache, fleet);
        Ok(changed)
    }

    /// Met à jour le domain d'une entrée existante sous le même verrou que
    /// `upsert`/`remove`. Un load+persist disjoint écraserait les insertions
    /// concurrentes.
    pub fn set_domain(&self, name: &str, domain: Option<&str>) -> Result<bool, DesiredStateError> {
        let mut cache = self.state.lock().recover_poison();
        let next = domain
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        {
            let current = self.load_cached(&mut cache)?;
            let Some(entry) = current.equipiers.get(name) else {
                return Ok(false);
            };
            if entry.domain == next {
                return Ok(false);
            }
        }
        let mut fleet = self.take_owned_fleet(&mut cache)?;
        if let Some(entry) = fleet.equipiers.get_mut(name) {
            entry.domain = next;
            fleet.schema = FLEET_SCHEMA_VERSION;
            self.persist_unlocked(&fleet)?;
            self.store_cache(&mut cache, fleet);
        }
        Ok(true)
    }

    fn load_cached(
        &self,
        cache: &mut DesiredStateCache,
    ) -> Result<Arc<DesiredFleet>, DesiredStateError> {
        let stamp = file_stamp(&self.path)?;
        let Some(stamp) = stamp else {
            cache.stamp = None;
            cache.fleet = None;
            return Ok(Arc::new(DesiredFleet::default()));
        };
        if cache.stamp == Some(stamp)
            && let Some(fleet) = &cache.fleet
        {
            return Ok(Arc::clone(fleet));
        }
        let loaded = Arc::new(self.load_from_disk_unlocked()?);
        cache.stamp = Some(stamp);
        cache.fleet = Some(Arc::clone(&loaded));
        Ok(loaded)
    }

    fn take_owned_fleet(
        &self,
        cache: &mut DesiredStateCache,
    ) -> Result<DesiredFleet, DesiredStateError> {
        let shared = self.load_cached(cache)?;
        cache.stamp = None;
        cache.fleet = None;
        Ok(Arc::unwrap_or_clone(shared))
    }

    fn store_cache(&self, cache: &mut DesiredStateCache, fleet: DesiredFleet) {
        cache.stamp = file_stamp(&self.path).ok().flatten();
        cache.fleet = Some(Arc::new(fleet));
    }

    fn load_from_disk_unlocked(&self) -> Result<DesiredFleet, DesiredStateError> {
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
        #[cfg(test)]
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Sonder d'abord le seul champ `schema` sans construire un arbre DOM complet
        // (`serde_json::Value`) pour les milliers d'entrées de `equipiers`.
        let header: FleetSchemaHeader =
            serde_json::from_slice(&bytes).map_err(|source| DesiredStateError::InvalidJson {
                path: self.path.clone(),
                source,
            })?;
        let found_schema = header.schema.as_ref().and_then(serde_json::Value::as_u64);
        if !found_schema.is_some_and(schema_lisible) {
            return Err(DesiredStateError::UnsupportedSchema {
                path: self.path.clone(),
                found: found_schema,
            });
        }
        let fleet: DesiredFleet =
            serde_json::from_slice(&bytes).map_err(|source| DesiredStateError::InvalidJson {
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
        validate_fleet_entries(&self.path, FLEET_SCHEMA_VERSION, &fleet.equipiers)?;
        let wire = WireDesiredFleet {
            schema: FLEET_SCHEMA_VERSION,
            equipiers: &fleet.equipiers,
        };
        let mut bytes =
            serde_json::to_vec_pretty(&wire).map_err(|source| DesiredStateError::InvalidJson {
                path: self.path.clone(),
                source,
            })?;
        bytes.push(b'\n');
        write_private_file_atomic_observed(&self.path, &bytes, observer).map_err(|source| {
            DesiredStateError::Io {
                path: self.path.clone(),
                source,
            }
        })?;
        #[cfg(test)]
        self.writes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

/// Validation O(n) de la frontière fichier, n étant le nombre d'entrées.
fn validate_fleet(path: &Path, fleet: &DesiredFleet) -> Result<(), DesiredStateError> {
    validate_fleet_entries(path, fleet.schema, &fleet.equipiers)
}

fn validate_fleet_entries(
    path: &Path,
    schema: u64,
    equipiers: &BTreeMap<String, DesiredEquipier>,
) -> Result<(), DesiredStateError> {
    if !schema_lisible(schema) {
        return Err(DesiredStateError::UnsupportedSchema {
            path: path.to_path_buf(),
            found: Some(schema),
        });
    }
    for (name, equipier) in equipiers {
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
        } else if equipier.remote_placement.as_ref().is_some_and(|placement| {
            schema < 5
                || placement.validate().is_err()
                || equipier.cwd != Path::new(&placement.execution_cwd)
                || equipier.runtime_execution.is_some()
        }) {
            Some("placement distant invalide")
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
            remote_placement: None,
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
            // SAFETY: `poll` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            assert_eq!(unsafe { libc::poll(&mut poll, 1, 5_000) }, 1);
            let mut byte = 0_u8;
            assert_eq!(
                // SAFETY: `byte` est un octet local valide ; la longueur passée à read vaut 1.
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
                // SAFETY: le descripteur est valide et tous les pointeurs de tampon/structure pointent vers des variables locales valides avec leur taille exacte.
                unsafe { libc::write(self.release, [b'R'].as_ptr().cast(), 1) },
                1
            );
        }
    }

    impl Drop for BarrierChild {
        fn drop(&mut self) {
            // SAFETY: close ne déréférence aucun pointeur ; ses arguments sont des entiers.
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
        // SAFETY: les pointeurs passés à pipe désignent des variables locales valides pendant l'appel.
        assert_eq!(unsafe { libc::pipe(ready_pipe.as_mut_ptr()) }, 0);
        // SAFETY: les pointeurs passés à pipe désignent des variables locales valides pendant l'appel.
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
        // SAFETY: la closure `pre_exec` n'exécute que des appels POSIX async-signal-safe entre `fork` et `exec`.
        unsafe {
            command.pre_exec(move || {
                // Copier d'abord au-dessus des numéros cibles : une extrémité
                // déjà numérotée 110 ou 111 serait sinon écrasée par le premier
                // dup2 (vu sous charge : EPIPE à la libération).
                let ready = libc::fcntl(ready_pipe[1], libc::F_DUPFD, BARRIER_RELEASE_FD + 1);
                let release = libc::fcntl(release_pipe[0], libc::F_DUPFD, BARRIER_RELEASE_FD + 1);
                if ready < 0
                    || release < 0
                    || libc::dup2(ready, BARRIER_READY_FD) < 0
                    || libc::dup2(release, BARRIER_RELEASE_FD) < 0
                {
                    return Err(io::Error::last_os_error());
                }
                for fd in ready_pipe
                    .into_iter()
                    .chain(release_pipe)
                    .chain([ready, release])
                {
                    if fd != BARRIER_READY_FD && fd != BARRIER_RELEASE_FD {
                        libc::close(fd);
                    }
                }
                Ok(())
            });
        }
        let child = command.spawn().unwrap();
        // SAFETY: close ne déréférence aucun pointeur ; ses arguments sont des entiers.
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
        // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
        let signal_result = unsafe { libc::kill(process.child.id() as libc::pid_t, libc::SIGTERM) };
        assert_eq!(signal_result, 0);
        assert!(!process.child.wait().unwrap().success());
    }

    #[test]
    fn remote162_fleet_migre_local_refuse_downgrade_et_preserve_distant_inconnu() {
        let root = test_root("remote162-schema");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        let mut entry = equipier("command162", 1);
        let legacy = serde_json::json!({"schema":4,"equipiers":{"agent":entry}});
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(
            store.load().unwrap().equipiers["agent"]
                .remote_placement
                .is_none()
        );
        entry.remote_placement = Some(
            serde_json::from_value(serde_json::json!({
                "machine_id":"cartae", "worker_id":"worker", "provisioning_revision":1,
                "project_id":"project", "mapping_revision":1, "origin_host":"mac",
                "origin_cwd":"/Users/moi/project", "source_root_identity":"git:project",
                "relative_cwd":".", "execution_cwd":"/tmp/projet", "base_commit":null,
                "definition_digest":"a".repeat(64), "permission_digest":"b".repeat(64)
            }))
            .unwrap(),
        );
        entry.persistent = false;
        store.upsert("agent".into(), entry.clone()).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.schema, 5);
        assert_eq!(
            loaded.equipiers["agent"].remote_placement,
            entry.remote_placement
        );
        assert!(store.stop_non_persistent_running().unwrap().is_empty());
        assert_eq!(
            store.load().unwrap().equipiers["agent"].lifecycle_state,
            DesiredLifecycleState::Running
        );
        // Un JSON v4 portant du distant n'est jamais interprété comme local.
        let mut downgraded = serde_json::to_value(loaded).unwrap();
        downgraded["schema"] = serde_json::json!(4);
        fs::write(&path, serde_json::to_vec(&downgraded).unwrap()).unwrap();
        assert!(matches!(
            store.load(),
            Err(DesiredStateError::InvalidEntry { .. })
        ));
        fs::remove_dir_all(root).unwrap();
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
    fn cache_memoire_evite_les_relectures_et_invalide_sur_modification_externe() {
        let root = test_root("cache-hit");
        let path = root.join("fleet.json");
        let store = DesiredStateStore::at_path(&path);
        let mut entry = equipier("command-cache", 1);
        entry.domain = Some("parix".to_string());
        store.upsert("agent-cache".to_string(), entry).unwrap();
        assert_eq!(store.read_count(), 0);
        assert_eq!(store.write_count(), 1);

        let first = store.load_shared().unwrap();
        let second = store.load_shared().unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert!(store.contains_key("agent-cache").unwrap());
        assert_eq!(
            store.domain("agent-cache").unwrap().as_deref(),
            Some("parix")
        );
        assert_eq!(
            store.get("agent-cache").unwrap().unwrap().command_id,
            "command-cache"
        );
        // Aucun accès disque supplémentaire après l'upsert qui a rempli le cache.
        assert_eq!(store.read_count(), 0);

        // Mutation sans changement : ni lecture ni écriture disque.
        assert!(!store.set_domain("agent-cache", Some("parix")).unwrap());
        assert_eq!(store.write_count(), 1);
        assert_eq!(store.read_count(), 0);

        // Un second store frais charge le fichier une seule fois pour plusieurs lectures.
        let reader = DesiredStateStore::at_path(&path);
        let a = reader.load_shared().unwrap();
        let b = reader.load_shared().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(reader.get("agent-cache").unwrap().unwrap().generation, 1);
        assert_eq!(reader.read_count(), 1);

        // Une écriture externe atomique change l'inode : le cache s'invalide et relit.
        let replacement = DesiredFleet::default();
        store.persist(&replacement).unwrap();
        let reloaded = reader.load_shared().unwrap();
        assert!(reloaded.equipiers.is_empty());
        assert_eq!(reader.read_count(), 2);

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
        fs::write(&path, r#"{"schema":6,"equipiers":{}}"#).unwrap();
        let error = DesiredStateStore::at_path(&path).load().unwrap_err();

        assert!(matches!(
            error,
            DesiredStateError::UnsupportedSchema { found: Some(6), .. }
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

    /// Collision forcée : le tube de libération reçoit les numéros 110/111,
    /// cibles mêmes du passage à l'enfant (cas vu sous charge, EPIPE).
    #[test]
    fn spec243_barriere_resiste_aux_numeros_cibles() {
        let mut fillers = Vec::new();
        loop {
            // SAFETY: les descripteurs de répertoire sont ouverts et valides et les chaînes `CString` terminées par nul restent vivantes pendant l'appel.
            let fd = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
            assert!(fd >= 0);
            if fd >= 108 {
                // SAFETY: close ne déréférence aucun pointeur ; ses arguments sont des entiers.
                unsafe { libc::close(fd) };
                break;
            }
            fillers.push(fd);
        }
        crash_after_rename_exposes_the_new_fleet_on_reopen();
        for fd in fillers {
            // SAFETY: close ne déréférence aucun pointeur ; ses arguments sont des entiers.
            unsafe { libc::close(fd) };
        }
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
                // SAFETY: le descripteur est valide et tous les pointeurs de tampon/structure pointent vers des variables locales valides avec leur taille exacte.
                if unsafe { libc::write(BARRIER_READY_FD, (&signal as *const u8).cast(), 1) } != 1 {
                    return Err(io::Error::last_os_error());
                }
                let mut release = 0_u8;
                // SAFETY: `release` est un octet local valide ; la longueur passée à read vaut 1.
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
