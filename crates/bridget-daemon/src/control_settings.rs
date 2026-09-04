//! Catalogue fermé des réglages serveur exposables au relais UI.
//!
//! Ce module est volontairement étroit : il ne connaît aucune commande shell,
//! aucun chemin de configuration libre et aucun secret. La première clé
//! modifiable est la liste de racines de projets, déjà protégée par
//! `ProjectRootPolicy`.

use crate::project_policy::{
    ProjectLocation, ProjectLocationCatalogReceipt, ProjectLocationCatalogReplacementPreview,
    ProjectRootPolicy, ProjectRootPolicyReceipt, ProjectRootPolicyReplacementPreview,
};
use bridget_transport::protocol::{ProjectBackend, ProjectRegistryRefusal};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub const CONTROL_SETTINGS_VERSION: u8 = 1;
pub const SERVER_EXECUTION_SETTINGS_VERSION: u8 = 1;

/// Valeur par défaut du backend pour les prochains projets seulement.
/// Les liaisons déjà publiées ne sont jamais réécrites par ce réglage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionDefaultBackend {
    Host,
    Docker,
}

/// Raisons bornées d'une capacité Docker incomplète. Aucun détail du daemon,
/// de son socket ou de la politique ne sort dans cette projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCapabilityReason {
    DockerUnavailable,
    PolicyUnavailable,
    ResourceCatalogUnavailable,
    ImageUnattested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapability {
    pub docker_available: bool,
    pub policy_available: bool,
    pub resource_catalog_available: bool,
    pub image_attested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<RuntimeCapabilityReason>,
}

impl RuntimeCapability {
    pub fn host_only() -> Self {
        Self {
            docker_available: false,
            policy_available: false,
            resource_catalog_available: false,
            image_attested: false,
            reason: Some(RuntimeCapabilityReason::DockerUnavailable),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.docker_available
            && self.policy_available
            && self.resource_catalog_available
            && self.image_attested
            && self.reason.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerExecutionSettings {
    pub version: u8,
    pub generation: u64,
    pub default_backend: ExecutionDefaultBackend,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_receipt: Option<ExecutionSettingsReceipt>,
}

impl ServerExecutionSettings {
    pub fn host_default() -> Self {
        Self {
            version: SERVER_EXECUTION_SETTINGS_VERSION,
            generation: 1,
            default_backend: ExecutionDefaultBackend::Host,
            default_policy_id: None,
            default_policy_version: None,
            last_receipt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSettingsReceipt {
    pub command_id: String,
    pub expected_generation: u64,
    pub resulting_generation: u64,
    pub default_backend: ExecutionDefaultBackend,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_policy_version: Option<u64>,
    pub observed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionSettingsChange {
    pub command_id: String,
    pub expected_generation: u64,
    pub default_backend: ExecutionDefaultBackend,
    pub default_policy_id: Option<String>,
    pub default_policy_version: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingAccess {
    Writable,
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingDescriptor {
    pub key: &'static str,
    pub scope: &'static str,
    pub access: SettingAccess,
    pub summary: &'static str,
}

pub fn server_setting_descriptors(policy_available: bool) -> Vec<SettingDescriptor> {
    vec![
        SettingDescriptor {
            key: "project_roots.allowed_roots",
            scope: "server",
            access: if policy_available {
                SettingAccess::Writable
            } else {
                SettingAccess::ReadOnly
            },
            summary: "Catalogue versionné des emplacements de projets autorisés.",
        },
        SettingDescriptor {
            key: "providers.observation",
            scope: "server",
            access: SettingAccess::ReadOnly,
            summary: "Capacités fournisseur attestées, sans secret.",
        },
        SettingDescriptor {
            key: "execution.default_backend",
            scope: "server",
            access: if policy_available {
                SettingAccess::Writable
            } else {
                SettingAccess::ReadOnly
            },
            summary: "Backend par défaut des prochains projets, Host ou Docker attesté.",
        },
        SettingDescriptor {
            key: "execution.policy",
            scope: "project",
            access: SettingAccess::ReadOnly,
            summary: "Politique contrôlée par le projet concerné.",
        },
        SettingDescriptor {
            key: "maintenance.status",
            scope: "server",
            access: SettingAccess::ReadOnly,
            summary: "Information de version et de diagnostic uniquement.",
        },
    ]
}

/// Descripteur isolé de la surface experte. Il ne se mélange pas aux réglages
/// ordinaires et reste lecture seule tant qu'aucun projet système attesté n'est
/// disponible sur ce serveur.
pub fn dogfooding_bridget_descriptor(system_available: bool) -> SettingDescriptor {
    SettingDescriptor {
        key: "dogfooding.bridget",
        scope: "server",
        access: if system_available {
            SettingAccess::Writable
        } else {
            SettingAccess::ReadOnly
        },
        summary: "Mode expert Bridget : édition et commits dans un worktree attribué; jamais merge, push, installation, redémarrage ou déploiement automatiques.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRootsChange {
    pub command_id: String,
    pub expected_generation: u64,
    pub allowed_project_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRootsPreview {
    pub current_generation: u64,
    pub current_roots: Vec<PathBuf>,
    pub requested_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRootsApplied {
    pub receipt: ProjectRootPolicyReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLocationCatalogChange {
    pub command_id: String,
    pub expected_generation: u64,
    pub locations: Vec<ProjectLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLocationCatalogPreview {
    pub current_generation: u64,
    pub current_locations: Vec<ProjectLocation>,
    pub requested_locations: Vec<ProjectLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLocationCatalogApplied {
    pub receipt: ProjectLocationCatalogReceipt,
}

pub fn preview_project_location_catalog(
    policy_path: Option<&Path>,
    change: &ProjectLocationCatalogChange,
) -> Result<ProjectLocationCatalogPreview, ControlSettingsRefusal> {
    validate_catalog_change(change)?;
    let source = policy_path.ok_or(ControlSettingsRefusal::Unavailable)?;
    let ProjectLocationCatalogReplacementPreview {
        current_generation,
        current_locations,
        requested_locations,
    } = ProjectRootPolicy::preview_catalog_replacement(
        source,
        change.expected_generation,
        change.locations.clone(),
    )
    .map_err(map_policy_error)?;
    Ok(ProjectLocationCatalogPreview {
        current_generation,
        current_locations,
        requested_locations,
    })
}

pub fn apply_project_location_catalog(
    policy_path: Option<&Path>,
    change: ProjectLocationCatalogChange,
    observed_at: i64,
) -> Result<ProjectLocationCatalogApplied, ControlSettingsRefusal> {
    validate_catalog_change(&change)?;
    let source = policy_path.ok_or(ControlSettingsRefusal::Unavailable)?;
    let policy = ProjectRootPolicy::replace_catalog_atomically_with_receipt(
        source,
        change.command_id,
        change.expected_generation,
        change.locations,
        observed_at,
    )
    .map_err(map_policy_error)?;
    Ok(ProjectLocationCatalogApplied {
        receipt: policy
            .last_catalog_receipt()
            .cloned()
            .ok_or(ControlSettingsRefusal::ConflictOrRefusal)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSettingsRefusal {
    Unavailable,
    InvalidRequest,
    ConflictOrRefusal,
    RuntimeCapabilityIncomplete,
}

/// Valeur fermée du réglage expert. Le défaut explicite évite qu'une migration
/// ou une absence de ligne transforme un ancien serveur en serveur inscriptible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DogfoodingBridgetMode {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DogfoodingBridgetState {
    pub project_id: String,
    pub binding_generation: u64,
    pub setting_generation: u64,
    pub mode: DogfoodingBridgetMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DogfoodingBridgetChange {
    pub command_id: String,
    pub expected_setting_generation: u64,
    pub expected_binding_generation: u64,
    pub requested_mode: DogfoodingBridgetMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DogfoodingBridgetPreview {
    pub current: DogfoodingBridgetState,
    pub next: DogfoodingBridgetState,
    pub requires_recreate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DogfoodingBridgetRefusal {
    InvalidRequest,
    GenerationMismatch,
    BindingGenerationMismatch,
    DockerRequired,
    ActiveSystemAgent,
}

/// Prépare un basculement sans muter le réglage. Le daemon doit effectuer le
/// stop/recreate attesté avant de persister `next` : l'échec conserve donc
/// mécaniquement la valeur confirmée `current`.
pub fn preview_dogfooding_bridget(
    current: &DogfoodingBridgetState,
    change: &DogfoodingBridgetChange,
    backend: ProjectBackend,
    active_system_agent: bool,
) -> Result<DogfoodingBridgetPreview, DogfoodingBridgetRefusal> {
    if !valid_command_id(&change.command_id)
        || current.project_id.trim().is_empty()
        || current.binding_generation == 0
        || current.setting_generation == 0
    {
        return Err(DogfoodingBridgetRefusal::InvalidRequest);
    }
    if change.expected_setting_generation != current.setting_generation {
        return Err(DogfoodingBridgetRefusal::GenerationMismatch);
    }
    if change.expected_binding_generation != current.binding_generation {
        return Err(DogfoodingBridgetRefusal::BindingGenerationMismatch);
    }
    if backend != ProjectBackend::Docker {
        return Err(DogfoodingBridgetRefusal::DockerRequired);
    }
    if active_system_agent && change.requested_mode != current.mode {
        return Err(DogfoodingBridgetRefusal::ActiveSystemAgent);
    }
    let mut next = current.clone();
    if next.mode != change.requested_mode {
        next.mode = change.requested_mode;
        next.setting_generation = next.setting_generation.saturating_add(1);
    }
    Ok(DogfoodingBridgetPreview {
        requires_recreate: next.mode != current.mode,
        current: current.clone(),
        next,
    })
}

pub fn preview_project_roots(
    policy_path: Option<&Path>,
    change: &ProjectRootsChange,
) -> Result<ProjectRootsPreview, ControlSettingsRefusal> {
    validate_change(change)?;
    let source = policy_path.ok_or(ControlSettingsRefusal::Unavailable)?;
    let ProjectRootPolicyReplacementPreview {
        current_generation,
        current_roots,
        requested_roots,
    } = ProjectRootPolicy::preview_replacement(
        source,
        change.expected_generation,
        change.allowed_project_roots.clone(),
    )
    .map_err(map_policy_error)?;
    Ok(ProjectRootsPreview {
        current_generation,
        current_roots,
        requested_roots,
    })
}

pub fn apply_project_roots(
    policy_path: Option<&Path>,
    change: ProjectRootsChange,
    observed_at: i64,
) -> Result<ProjectRootsApplied, ControlSettingsRefusal> {
    validate_change(&change)?;
    let source = policy_path.ok_or(ControlSettingsRefusal::Unavailable)?;
    let policy = ProjectRootPolicy::replace_atomically_with_receipt(
        source,
        change.command_id,
        change.expected_generation,
        change.allowed_project_roots,
        observed_at,
    )
    .map_err(map_policy_error)?;
    let receipt = policy
        .last_control_receipt()
        .cloned()
        .ok_or(ControlSettingsRefusal::ConflictOrRefusal)?;
    Ok(ProjectRootsApplied { receipt })
}

fn validate_change(change: &ProjectRootsChange) -> Result<(), ControlSettingsRefusal> {
    if valid_command_id(&change.command_id)
        && change.expected_generation > 0
        && !change.allowed_project_roots.is_empty()
    {
        Ok(())
    } else {
        Err(ControlSettingsRefusal::InvalidRequest)
    }
}

fn validate_catalog_change(
    change: &ProjectLocationCatalogChange,
) -> Result<(), ControlSettingsRefusal> {
    let id = change.command_id.as_bytes();
    let valid_id = !id.is_empty()
        && id.len() <= 160
        && id
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_'));
    if valid_id && change.expected_generation > 0 && !change.locations.is_empty() {
        Ok(())
    } else {
        Err(ControlSettingsRefusal::InvalidRequest)
    }
}

fn valid_command_id(value: &str) -> bool {
    let id = value.as_bytes();
    !id.is_empty()
        && id.len() <= 160
        && id
            .iter()
            .all(|byte| (*byte).is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_'))
}

fn map_policy_error(error: ProjectRegistryRefusal) -> ControlSettingsRefusal {
    match error {
        ProjectRegistryRefusal::ProjectRootPolicyUnavailable => ControlSettingsRefusal::Unavailable,
        _ => ControlSettingsRefusal::ConflictOrRefusal,
    }
}

/// Lit le réglage d'exécution, ou installe la migration Host minimale lorsque
/// l'installation n'avait encore aucun document. La migration ne consulte ni
/// le registre ni les bindings existants et ne peut donc pas les modifier.
pub fn load_or_initialize_execution_settings(
    source: &Path,
) -> Result<ServerExecutionSettings, ControlSettingsRefusal> {
    if !source.exists() {
        write_execution_settings_atomically(source, &ServerExecutionSettings::host_default())?;
    }
    load_execution_settings(source)
}

pub fn load_execution_settings(
    source: &Path,
) -> Result<ServerExecutionSettings, ControlSettingsRefusal> {
    let metadata = fs::symlink_metadata(source).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(ControlSettingsRefusal::Unavailable);
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(ControlSettingsRefusal::Unavailable);
    }
    let raw = fs::read(source).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    let settings: ServerExecutionSettings =
        serde_json::from_slice(&raw).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    validate_execution_settings(&settings, None)?;
    Ok(settings)
}

pub fn preview_execution_settings(
    source: &Path,
    change: &ExecutionSettingsChange,
    capability: &RuntimeCapability,
) -> Result<ServerExecutionSettings, ControlSettingsRefusal> {
    validate_execution_change(change, capability)?;
    let current = load_or_initialize_execution_settings(source)?;
    if current.generation != change.expected_generation {
        return Err(ControlSettingsRefusal::ConflictOrRefusal);
    }
    Ok(ServerExecutionSettings {
        version: SERVER_EXECUTION_SETTINGS_VERSION,
        generation: current.generation.saturating_add(1),
        default_backend: change.default_backend,
        default_policy_id: normalized_policy_id(change.default_policy_id.as_deref()),
        default_policy_version: change.default_policy_version,
        last_receipt: None,
    })
}

/// Applique un défaut pour les futures créations. Un rejeu strictement
/// identique retrouve son reçu sans incrémenter la génération.
pub fn apply_execution_settings(
    source: &Path,
    change: ExecutionSettingsChange,
    capability: &RuntimeCapability,
    observed_at: i64,
) -> Result<ServerExecutionSettings, ControlSettingsRefusal> {
    validate_execution_change(&change, capability)?;
    let current = load_or_initialize_execution_settings(source)?;
    let policy_id = normalized_policy_id(change.default_policy_id.as_deref());
    if let Some(receipt) = current
        .last_receipt
        .as_ref()
        .filter(|receipt| receipt.command_id == change.command_id)
    {
        if receipt.expected_generation == change.expected_generation
            && receipt.default_backend == change.default_backend
            && receipt.default_policy_id == policy_id
            && receipt.default_policy_version == change.default_policy_version
        {
            return Ok(current);
        }
        return Err(ControlSettingsRefusal::ConflictOrRefusal);
    }
    if current.generation != change.expected_generation {
        return Err(ControlSettingsRefusal::ConflictOrRefusal);
    }
    let resulting_generation = current.generation.saturating_add(1);
    let settings = ServerExecutionSettings {
        version: SERVER_EXECUTION_SETTINGS_VERSION,
        generation: resulting_generation,
        default_backend: change.default_backend,
        default_policy_id: policy_id.clone(),
        default_policy_version: change.default_policy_version,
        last_receipt: Some(ExecutionSettingsReceipt {
            command_id: change.command_id,
            expected_generation: change.expected_generation,
            resulting_generation,
            default_backend: change.default_backend,
            default_policy_id: policy_id,
            default_policy_version: change.default_policy_version,
            observed_at,
        }),
    };
    write_execution_settings_atomically(source, &settings)?;
    load_execution_settings(source)
}

fn validate_execution_change(
    change: &ExecutionSettingsChange,
    capability: &RuntimeCapability,
) -> Result<(), ControlSettingsRefusal> {
    if !valid_control_command_id(&change.command_id) || change.expected_generation == 0 {
        return Err(ControlSettingsRefusal::InvalidRequest);
    }
    let settings = ServerExecutionSettings {
        version: SERVER_EXECUTION_SETTINGS_VERSION,
        generation: change.expected_generation.saturating_add(1),
        default_backend: change.default_backend,
        default_policy_id: normalized_policy_id(change.default_policy_id.as_deref()),
        default_policy_version: change.default_policy_version,
        last_receipt: None,
    };
    validate_execution_settings(&settings, Some(capability))
}

fn validate_execution_settings(
    settings: &ServerExecutionSettings,
    capability: Option<&RuntimeCapability>,
) -> Result<(), ControlSettingsRefusal> {
    if settings.version != SERVER_EXECUTION_SETTINGS_VERSION || settings.generation == 0 {
        return Err(ControlSettingsRefusal::InvalidRequest);
    }
    match settings.default_backend {
        ExecutionDefaultBackend::Host => {
            if settings.default_policy_id.is_some() || settings.default_policy_version.is_some() {
                return Err(ControlSettingsRefusal::InvalidRequest);
            }
        }
        ExecutionDefaultBackend::Docker => {
            if normalized_policy_id(settings.default_policy_id.as_deref()).is_none()
                || settings.default_policy_version.unwrap_or(0) == 0
            {
                return Err(ControlSettingsRefusal::InvalidRequest);
            }
            if capability.is_some_and(|capability| !capability.is_complete()) {
                return Err(ControlSettingsRefusal::RuntimeCapabilityIncomplete);
            }
        }
    }
    Ok(())
}

fn normalized_policy_id(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .map(str::to_string)
}

fn valid_control_command_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn write_execution_settings_atomically(
    source: &Path,
    settings: &ServerExecutionSettings,
) -> Result<(), ControlSettingsRefusal> {
    write_settings_document_atomically(source, settings)
}

/// Écriture atomique 0600 partagée par les documents de réglages du relais.
fn write_settings_document_atomically<T: Serialize>(
    source: &Path,
    settings: &T,
) -> Result<(), ControlSettingsRefusal> {
    let parent = source.parent().ok_or(ControlSettingsRefusal::Unavailable)?;
    fs::create_dir_all(parent).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    let payload =
        serde_json::to_vec_pretty(settings).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    let temporary = source.with_extension(format!("{}.tmp", uuid::Uuid::new_v4().simple()));
    #[cfg(unix)]
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&temporary)
        .map_err(|_| ControlSettingsRefusal::Unavailable)?;
    #[cfg(not(unix))]
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| ControlSettingsRefusal::Unavailable)?;
    file.write_all(&payload)
        .and_then(|_| file.sync_all())
        .map_err(|_| ControlSettingsRefusal::Unavailable)?;
    fs::rename(&temporary, source).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    #[cfg(unix)]
    fs::set_permissions(source, fs::Permissions::from_mode(0o600))
        .map_err(|_| ControlSettingsRefusal::Unavailable)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// SPEC-088 : profils de droits. Les valeurs serveur vivent dans `control_state`
// (daemon, ADR-027) ; ici seulement la matrice fermée et sa dérivation.
// ---------------------------------------------------------------------------

pub use bridget_transport::protocol::AgentPosture;

pub const RIGHTS_TEST_STALE_AFTER_SECS: i64 = 24 * 3600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsProfile {
    Prudent,
    Balanced,
    Confident,
    Custom,
}

impl RightsProfile {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "prudent" => Some(Self::Prudent),
            "balanced" => Some(Self::Balanced),
            "confident" => Some(Self::Confident),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Valeurs d'un profil pour les lignes serveur ET les lignes locales : le
/// relais n'écrit que les premières ; la vue applique les secondes dans le
/// navigateur qui a choisi le profil (FR-013).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileValues {
    pub agent_posture: AgentPosture,
    pub auto_reassignment: bool,
    pub auto_objectives_cap: u32,
    pub external_links: bool,
    pub remote_images: bool,
    pub file_references: bool,
}

/// Matrice fermée (research R6). `Custom` n'a pas de valeurs : c'est l'état
/// calculé quand une ligne diverge.
pub const PROFILE_MATRIX: &[(RightsProfile, ProfileValues)] = &[
    (
        RightsProfile::Prudent,
        ProfileValues {
            agent_posture: AgentPosture::Discovery,
            auto_reassignment: false,
            auto_objectives_cap: 5,
            external_links: false,
            remote_images: false,
            file_references: false,
        },
    ),
    (
        RightsProfile::Balanced,
        ProfileValues {
            agent_posture: AgentPosture::Complete,
            auto_reassignment: true,
            auto_objectives_cap: 5,
            external_links: true,
            remote_images: false,
            file_references: true,
        },
    ),
    (
        RightsProfile::Confident,
        ProfileValues {
            agent_posture: AgentPosture::Complete,
            auto_reassignment: true,
            auto_objectives_cap: 20,
            external_links: true,
            remote_images: true,
            file_references: true,
        },
    ),
];

pub fn profile_values(profile: RightsProfile) -> Option<ProfileValues> {
    PROFILE_MATRIX
        .iter()
        .find(|(candidate, _)| *candidate == profile)
        .map(|(_, values)| *values)
}

/// Profil réellement en vigueur sur les lignes serveur : le premier profil
/// dont les trois valeurs serveur correspondent, sinon `Custom`. `None` sur
/// une valeur (daemon antérieur) ⇒ `Custom`. O(3).
pub fn profile_for(
    agent_posture: Option<AgentPosture>,
    auto_reassignment: Option<bool>,
    auto_objectives_cap: u32,
) -> RightsProfile {
    let (Some(agent_posture), Some(auto_reassignment)) = (agent_posture, auto_reassignment) else {
        return RightsProfile::Custom;
    };
    PROFILE_MATRIX
        .iter()
        .find(|(_, values)| {
            values.agent_posture == agent_posture
                && values.auto_reassignment == auto_reassignment
                && values.auto_objectives_cap == auto_objectives_cap
        })
        .map(|(profile, _)| *profile)
        .unwrap_or(RightsProfile::Custom)
}

#[cfg(test)]
mod rights_tests {
    use super::*;

    #[test]
    fn spec_088_profile_for_rend_custom_quand_une_ligne_diverge_ou_manque() {
        assert_eq!(
            profile_for(Some(AgentPosture::Discovery), Some(false), 5),
            RightsProfile::Prudent
        );
        assert_eq!(
            profile_for(Some(AgentPosture::Complete), Some(true), 5),
            RightsProfile::Balanced
        );
        assert_eq!(
            profile_for(Some(AgentPosture::Complete), Some(true), 20),
            RightsProfile::Confident
        );
        assert_eq!(
            profile_for(Some(AgentPosture::Complete), Some(false), 5),
            RightsProfile::Custom
        );
        assert_eq!(
            profile_for(Some(AgentPosture::Discovery), Some(false), 7),
            RightsProfile::Custom
        );
        // Daemon antérieur : valeur inconnue ⇒ jamais un profil nommé.
        assert_eq!(profile_for(None, Some(false), 5), RightsProfile::Custom);
        assert_eq!(
            profile_for(Some(AgentPosture::Discovery), None, 5),
            RightsProfile::Custom
        );
    }

    #[test]
    fn spec_088_chaque_profil_nomme_a_ses_valeurs_et_custom_aucune() {
        for profile in [
            RightsProfile::Prudent,
            RightsProfile::Balanced,
            RightsProfile::Confident,
        ] {
            let values = profile_values(profile).expect("valeurs du profil");
            assert_eq!(
                profile_for(
                    Some(values.agent_posture),
                    Some(values.auto_reassignment),
                    values.auto_objectives_cap
                ),
                profile
            );
            assert_eq!(
                RightsProfile::parse(serde_json::to_string(&profile).unwrap().trim_matches('"')),
                Some(profile)
            );
        }
        assert_eq!(profile_values(RightsProfile::Custom), None);
        assert_eq!(RightsProfile::parse("libre"), None);
    }
}

// ---------------------------------------------------------------------------
// SPEC-088 : tentatives de test d'un droit (relais). Une tentative est
// corrélée par le `message_id` rendu à l'envoi et par la commande canonique
// exacte ; elle ne se résout que sur une fin de commande, sinon elle expire.
// ---------------------------------------------------------------------------

pub const RIGHTS_TEST_EXPIRY_SECS: i64 = 120;
pub const RIGHTS_TEST_TOKEN_PREFIX: &str = "BRIDGET-TEST-";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsTestLine {
    Shell,
    Files,
    Internet,
    Bridget,
}

impl RightsTestLine {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "shell" => Some(Self::Shell),
            "files" => Some(Self::Files),
            "internet" => Some(Self::Internet),
            "bridget" => Some(Self::Bridget),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsTestOutcome {
    Pending,
    Passed,
    RefusedProviderSandbox,
    RefusedServerRuntime,
    RefusedBridget,
    /// Ligne « Modifier Bridget » : le réglage est activé mais aucun geste
    /// non mutateur ne mesure cette capacité ; on ne dit pas « réussi ».
    EnabledNotMeasured,
    Unreachable,
    NoAgent,
    UnknownExpired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightsTestAttempt {
    pub test_id: String,
    pub line: RightsTestLine,
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    pub expected_command: String,
    pub expected_sha256: String,
    pub token: String,
    pub started_at: i64,
    pub outcome: RightsTestOutcome,
    #[serde(default)]
    pub raw: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightsTestsDocument {
    #[serde(default)]
    pub attempts: Vec<RightsTestAttempt>,
}

/// Commande fermée d'un geste de test (FR-021). Aucune n'écrit, n'installe,
/// ni ne dépense ; `files` et `bridget` lisent zéro octet d'un fichier connu.
pub fn test_gesture(line: RightsTestLine, token: &str, known_file: &str) -> String {
    match line {
        RightsTestLine::Shell => format!("printf '{token}\\n'"),
        RightsTestLine::Files | RightsTestLine::Bridget => {
            format!("head -c 0 '{known_file}' && printf '{token}\\n'")
        }
        RightsTestLine::Internet => {
            format!("curl -sS -o /dev/null -w '%{{http_code}} {token}\\n' https://example.com/")
        }
    }
}

pub fn sha256_hex(text: &str) -> String {
    bridget_transport::protocol::sha256_hex(text.as_bytes())
}

pub fn new_rights_test_attempt(
    line: RightsTestLine,
    agent_id: &str,
    known_file: &str,
    now: i64,
) -> RightsTestAttempt {
    let short = uuid::Uuid::new_v4().simple().to_string();
    let token = format!("{RIGHTS_TEST_TOKEN_PREFIX}{}", &short[..12]);
    let expected_command = test_gesture(line, &token, known_file);
    RightsTestAttempt {
        test_id: uuid::Uuid::new_v4().to_string(),
        line,
        agent_id: agent_id.to_string(),
        message_id: None,
        expected_sha256: sha256_hex(&expected_command),
        expected_command,
        token,
        started_at: now,
        outcome: RightsTestOutcome::Pending,
        raw: String::new(),
        finished_at: None,
    }
}

/// Lecture partagée : fichier régulier, non symlink, jamais inscriptible par
/// le groupe ou les autres (même garde que les réglages d'exécution).
fn read_settings_document(source: &Path) -> Result<Vec<u8>, ControlSettingsRefusal> {
    let metadata = fs::symlink_metadata(source).map_err(|_| ControlSettingsRefusal::Unavailable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(ControlSettingsRefusal::Unavailable);
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(ControlSettingsRefusal::Unavailable);
    }
    fs::read(source).map_err(|_| ControlSettingsRefusal::Unavailable)
}

pub fn load_rights_tests(source: &Path) -> RightsTestsDocument {
    if !source.exists() {
        return RightsTestsDocument::default();
    }
    read_settings_document(source)
        .ok()
        .and_then(|raw| serde_json::from_slice(&raw).ok())
        .unwrap_or_default()
}

pub fn save_rights_tests(
    source: &Path,
    document: &RightsTestsDocument,
) -> Result<(), ControlSettingsRefusal> {
    write_settings_document_atomically(source, document)
}

/// Remplace la tentative de la même ligne ; une tentative par ligne.
pub fn upsert_rights_test(document: &mut RightsTestsDocument, attempt: RightsTestAttempt) {
    document
        .attempts
        .retain(|existing| existing.line != attempt.line);
    document.attempts.push(attempt);
}

fn bounded(text: &str) -> String {
    text.chars()
        .take(bridget_transport::refusals::MAX_RAW_CHARS)
        .collect()
}

/// Résolution pure d'une tentative à partir des événements du journal de
/// l'agent (objets `{ message_id, event, payload }`). Contrat rights-v1 :
/// seule une fin de commande au texte exactement attendu conclut ; un acte
/// `refusal` du même message et du même item conclut en refus ; sinon la
/// tentative reste `pending` puis expire. Complexité : O(n) sur les
/// événements fournis, n borné par la fenêtre lue.
pub fn resolve_rights_test(
    attempt: &RightsTestAttempt,
    events: &[serde_json::Value],
    now: i64,
) -> RightsTestAttempt {
    if attempt.outcome != RightsTestOutcome::Pending {
        return attempt.clone();
    }
    let mut resolved = attempt.clone();
    let Some(message_id) = attempt.message_id.as_deref() else {
        return expire_if_due(resolved, now);
    };
    let mine = events.iter().filter(|event| {
        event.get("message_id").and_then(serde_json::Value::as_str) == Some(message_id)
            && event.get("event").and_then(serde_json::Value::as_str) == Some("update")
    });
    // Parcours complet : le producteur écrit la fin de commande AVANT l'acte
    // refusal du même item ; conclure au premier événement masquerait le refus.
    let mut refusals_by_item: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut completion: Option<(bool, i64, String, String)> = None;
    for event in mine {
        let payload = event.get("payload").cloned().unwrap_or_default();
        let kind = payload
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let item_id = payload
            .get("item_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        if kind == "refusal" {
            let raw = payload
                .get("raw")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            refusals_by_item.entry(item_id).or_insert(raw);
            continue;
        }
        if completion.is_some()
            || kind != "command"
            || payload.get("detail").and_then(serde_json::Value::as_str) != Some("item/completed")
        {
            continue;
        }
        let text = payload
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if normalize_command_text(text) != normalize_command_text(&attempt.expected_command) {
            continue;
        }
        let failed = payload.get("state").and_then(serde_json::Value::as_str) == Some("failed");
        let exit_code = payload
            .get("exit_code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-1);
        let tail = payload
            .get("output_tail")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        completion = Some((failed, exit_code, tail, item_id));
    }
    let Some((failed, exit_code, tail, item_id)) = completion else {
        return expire_if_due(resolved, now);
    };
    resolved.finished_at = Some(now);
    resolved.raw = bounded(&tail);
    if !failed && exit_code == 0 && tail.contains(&attempt.token) {
        resolved.outcome = RightsTestOutcome::Passed;
        return resolved;
    }
    if failed && let Some(raw) = refusals_by_item.get(&item_id) {
        resolved.outcome = RightsTestOutcome::RefusedProviderSandbox;
        resolved.raw = bounded(raw);
        return resolved;
    }
    if attempt.line == RightsTestLine::Internet
        && failed
        && (tail.contains("curl: (6)") || tail.contains("curl: (7)") || tail.contains("curl: (28)"))
    {
        resolved.outcome = RightsTestOutcome::Unreachable;
        return resolved;
    }
    // Terminaison sans jeton ni signal reconnu : on ne classe pas.
    resolved.outcome = RightsTestOutcome::UnknownExpired;
    resolved
}

/// Codex exécute la commande demandée dans une enveloppe `/bin/bash -lc "…"`
/// et échappe guillemets et antislashs (mesuré le 2026-09-04 sur Jim). La
/// comparaison exacte porte sur la commande DÉNUDÉE de cette enveloppe ; une
/// commande voisine qui imprime le jeton reste différente.
pub fn normalize_command_text(text: &str) -> String {
    let mut inner = text.trim();
    for prefix in [
        "/bin/bash -lc ",
        "bash -lc ",
        "/bin/sh -lc ",
        "sh -lc ",
        "/bin/bash -c ",
        "bash -c ",
        "/bin/sh -c ",
        "sh -c ",
    ] {
        if let Some(rest) = inner.strip_prefix(prefix) {
            inner = rest.trim();
            break;
        }
    }
    let unquoted = if inner.len() >= 2
        && ((inner.starts_with('"') && inner.ends_with('"'))
            || (inner.starts_with('\'') && inner.ends_with('\'')))
    {
        &inner[1..inner.len() - 1]
    } else {
        inner
    };
    unquoted
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .trim()
        .to_string()
}

fn expire_if_due(mut attempt: RightsTestAttempt, now: i64) -> RightsTestAttempt {
    if now.saturating_sub(attempt.started_at) > RIGHTS_TEST_EXPIRY_SECS {
        attempt.outcome = RightsTestOutcome::UnknownExpired;
        attempt.finished_at = Some(now);
    }
    attempt
}

#[cfg(test)]
mod rights_test_tests {
    use super::*;
    use serde_json::json;

    fn attempt() -> RightsTestAttempt {
        let mut attempt = new_rights_test_attempt(RightsTestLine::Shell, "agent-1", "/x", 1_000);
        attempt.message_id = Some("m-1".to_string());
        attempt
    }

    fn command_completed(
        message_id: &str,
        text: &str,
        state: &str,
        exit_code: i64,
        tail: &str,
    ) -> serde_json::Value {
        json!({ "message_id": message_id, "event": "update", "payload": {
            "kind": "command", "detail": "item/completed", "text": text, "state": state,
            "exit_code": exit_code, "output_tail": tail, "item_id": "item-1" } })
    }

    #[test]
    fn spec_088_gestes_fermes_portent_le_jeton_et_n_ecrivent_jamais() {
        for line in [
            RightsTestLine::Shell,
            RightsTestLine::Files,
            RightsTestLine::Internet,
            RightsTestLine::Bridget,
        ] {
            let command = test_gesture(line, "BRIDGET-TEST-abc", "/srv/p/README.md");
            assert!(command.contains("BRIDGET-TEST-abc"), "{command}");
            for forbidden in [
                " > ",
                " >> ",
                "tee ",
                "rm ",
                "mv ",
                "chmod ",
                "curl -o /tmp",
            ] {
                assert!(
                    !command.contains(forbidden),
                    "{command} contient {forbidden:?}"
                );
            }
        }
        let a = new_rights_test_attempt(RightsTestLine::Shell, "agent-1", "/x", 1);
        assert_eq!(a.expected_sha256, sha256_hex(&a.expected_command));
        assert_eq!(a.outcome, RightsTestOutcome::Pending);
    }

    #[test]
    fn spec_088_l_enveloppe_bash_de_codex_est_reconnue_mais_pas_une_commande_voisine() {
        // Échantillon réel (journal de Jim, 2026-09-04 04:14 UTC).
        let expected = "printf 'BRIDGET-TEST-726c7780ac6a\\n'";
        let real = "/bin/bash -lc \"printf 'BRIDGET-TEST-726c7780ac6a\\\\n'\"";
        assert_eq!(
            normalize_command_text(real),
            normalize_command_text(expected)
        );
        assert_eq!(normalize_command_text(expected), expected);
        assert_ne!(
            normalize_command_text("/bin/bash -lc \"echo BRIDGET-TEST-726c7780ac6a\""),
            normalize_command_text(expected)
        );
        let mut a = new_rights_test_attempt(RightsTestLine::Shell, "agent-1", "/x", 1_000);
        a.message_id = Some("m-1".to_string());
        let wrapped = format!(
            "/bin/bash -lc \"{}\"",
            a.expected_command.replace('\\', "\\\\")
        );
        let done = command_completed("m-1", &wrapped, "completed", 0, &format!("{}\n", a.token));
        assert_eq!(
            resolve_rights_test(&a, &[done], 1_010).outcome,
            RightsTestOutcome::Passed
        );
    }

    #[test]
    fn spec_088_commande_demarree_sans_fin_reste_pending_puis_expire() {
        let a = attempt();
        let started = json!({ "message_id": "m-1", "event": "update", "payload": {
            "kind": "command", "detail": "item/started", "text": a.expected_command } });
        assert_eq!(
            resolve_rights_test(&a, std::slice::from_ref(&started), 1_010).outcome,
            RightsTestOutcome::Pending
        );
        assert_eq!(
            resolve_rights_test(&a, &[started], 1_121).outcome,
            RightsTestOutcome::UnknownExpired
        );
    }

    #[test]
    fn spec_088_fin_reussie_avec_jeton_passe_et_autre_message_ne_compte_pas() {
        let a = attempt();
        let other = command_completed("m-2", &a.expected_command, "completed", 0, &a.token);
        assert_eq!(
            resolve_rights_test(&a, &[other], 1_010).outcome,
            RightsTestOutcome::Pending
        );
        let mine = command_completed(
            "m-1",
            &a.expected_command,
            "completed",
            0,
            &format!("{}\n", a.token),
        );
        let resolved = resolve_rights_test(&a, &[mine], 1_010);
        assert_eq!(resolved.outcome, RightsTestOutcome::Passed);
        assert_eq!(resolved.finished_at, Some(1_010));
        // Une autre commande qui imprime le jeton n'est pas la commande attendue.
        let forged = command_completed("m-1", "echo BRIDGET", "completed", 0, &a.token);
        assert_eq!(
            resolve_rights_test(&a, &[forged], 1_010).outcome,
            RightsTestOutcome::Pending
        );
    }

    #[test]
    fn spec_088_fin_en_echec_apres_ligne_reconnue_est_un_refus_de_sandbox() {
        let a = attempt();
        let refusal = json!({ "message_id": "m-1", "event": "update", "payload": {
            "kind": "refusal", "layer": "provider_sandbox", "item_id": "item-1",
            "raw": "bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted" } });
        let failed = command_completed("m-1", &a.expected_command, "failed", 1, "");
        // Ordre RÉEL du producteur : la fin de commande, PUIS l'acte refusal.
        let resolved = resolve_rights_test(&a, &[failed.clone(), refusal.clone()], 1_010);
        assert_eq!(resolved.outcome, RightsTestOutcome::RefusedProviderSandbox);
        assert!(resolved.raw.starts_with("bwrap:"));
        // Ordre inverse : même verdict.
        assert_eq!(
            resolve_rights_test(&a, &[refusal.clone(), failed.clone()], 1_010).outcome,
            RightsTestOutcome::RefusedProviderSandbox
        );
        // Deux items : le refus de l'autre item ne contamine pas la commande attendue.
        let other_item_failed = json!({ "message_id": "m-1", "event": "update", "payload": {
            "kind": "command", "detail": "item/completed", "text": "ls", "state": "failed",
            "exit_code": 1, "output_tail": "", "item_id": "item-2" } });
        let other_item_refusal = json!({ "message_id": "m-1", "event": "update", "payload": {
            "kind": "refusal", "layer": "provider_sandbox", "item_id": "item-2", "raw": "bwrap: y" } });
        let ok = command_completed("m-1", &a.expected_command, "completed", 0, &a.token);
        assert_eq!(
            resolve_rights_test(&a, &[other_item_failed, other_item_refusal, ok], 1_010).outcome,
            RightsTestOutcome::Passed
        );
        // Refus d'un autre item : non classé.
        let other_refusal = json!({ "message_id": "m-1", "event": "update", "payload": {
            "kind": "refusal", "layer": "provider_sandbox", "item_id": "item-9", "raw": "bwrap: x" } });
        assert_eq!(
            resolve_rights_test(&a, &[other_refusal, failed], 1_010).outcome,
            RightsTestOutcome::UnknownExpired
        );
    }

    #[test]
    fn spec_088_internet_injoignable_reconnu_par_curl() {
        let mut a = new_rights_test_attempt(RightsTestLine::Internet, "agent-1", "/x", 1_000);
        a.message_id = Some("m-1".to_string());
        let failed = command_completed(
            "m-1",
            &a.expected_command,
            "failed",
            6,
            "curl: (6) Could not resolve host",
        );
        assert_eq!(
            resolve_rights_test(&a, &[failed], 1_010).outcome,
            RightsTestOutcome::Unreachable
        );
    }

    #[test]
    fn spec_088_une_tentative_par_ligne_et_document_0600() {
        let root = std::env::temp_dir().join(format!(
            "bridget-rights-tests-{}",
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("server-rights-tests.json");
        let mut document = load_rights_tests(&path);
        upsert_rights_test(
            &mut document,
            new_rights_test_attempt(RightsTestLine::Shell, "a", "/x", 1),
        );
        upsert_rights_test(
            &mut document,
            new_rights_test_attempt(RightsTestLine::Shell, "b", "/x", 2),
        );
        upsert_rights_test(
            &mut document,
            new_rights_test_attempt(RightsTestLine::Files, "a", "/x", 3),
        );
        assert_eq!(document.attempts.len(), 2);
        assert_eq!(document.attempts[0].agent_id, "b");
        save_rights_tests(&path, &document).unwrap();
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(load_rights_tests(&path), document);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_policy::ProjectLocationKind;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn execution_fixture_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-execution-settings-{}-{}.json",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn complete_runtime_capability() -> RuntimeCapability {
        RuntimeCapability {
            docker_available: true,
            policy_available: true,
            resource_catalog_available: true,
            image_attested: true,
            reason: None,
        }
    }

    #[test]
    fn catalogue_est_ferme_et_seule_la_politique_projet_devient_modifiable() {
        let descriptors = server_setting_descriptors(true);
        assert_eq!(descriptors.len(), 5);
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.key == "project_roots.allowed_roots"
                && descriptor.access == SettingAccess::Writable
        }));
        assert!(
            descriptors
                .iter()
                .filter(|descriptor| descriptor.access == SettingAccess::Writable)
                .count()
                == 2
        );
        assert!(
            validate_change(&ProjectRootsChange {
                command_id: "wrong/id".to_string(),
                expected_generation: 1,
                allowed_project_roots: vec![PathBuf::from("/tmp")],
            })
            .is_err()
        );
    }
    #[test]
    fn spec_084_catalogue_v2_previsualise_applique_rejoue_et_refuse_generation_obsolete() {
        let root = std::env::temp_dir().join(format!(
            "bridget-spec-084-control-settings-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let projects = root.join("projects");
        fs::create_dir_all(&projects).unwrap();
        let policy = root.join("project-root-policy.json");
        fs::write(
            &policy,
            serde_json::to_vec(&serde_json::json!({
                "contract_version": 1,
                "policy_generation": 1,
                "allowed_project_roots": [projects],
            }))
            .unwrap(),
        )
        .unwrap();
        fs::set_permissions(&policy, fs::Permissions::from_mode(0o600)).unwrap();
        let change = ProjectLocationCatalogChange {
            command_id: "catalog-084".to_string(),
            expected_generation: 1,
            locations: vec![ProjectLocation {
                location_id: "workspace-main".to_string(),
                label: "Projets".to_string(),
                canonical_path: projects,
                kind: ProjectLocationKind::Workspace,
                system_only: false,
                default_creation: true,
            }],
        };
        let invalid = ProjectLocationCatalogChange {
            command_id: "catalog-invalid".to_string(),
            expected_generation: 1,
            locations: vec![],
        };
        assert_eq!(
            apply_project_location_catalog(Some(&policy), invalid, 99),
            Err(ControlSettingsRefusal::InvalidRequest)
        );
        assert_eq!(ProjectRootPolicy::load(&policy).unwrap().generation(), 1);
        let preview = preview_project_location_catalog(Some(&policy), &change).unwrap();
        assert_eq!(preview.current_generation, 1);
        assert_eq!(
            preview.current_locations[0].kind,
            ProjectLocationKind::ExactProject
        );
        assert_eq!(
            preview.requested_locations[0].kind,
            ProjectLocationKind::Workspace
        );
        let applied = apply_project_location_catalog(Some(&policy), change.clone(), 100).unwrap();
        assert_eq!(applied.receipt.resulting_generation, 2);
        let replay = apply_project_location_catalog(Some(&policy), change, 101).unwrap();
        assert_eq!(replay.receipt.observed_at, 100);
        let stale = ProjectLocationCatalogChange {
            command_id: "catalog-stale".to_string(),
            expected_generation: 1,
            locations: vec![ProjectLocation {
                location_id: "workspace-main".to_string(),
                label: "Projets".to_string(),
                canonical_path: ProjectRootPolicy::load(&policy).unwrap().locations()[0]
                    .canonical_path
                    .clone(),
                kind: ProjectLocationKind::Workspace,
                system_only: false,
                default_creation: true,
            }],
        };
        assert_eq!(
            preview_project_location_catalog(Some(&policy), &stale),
            Err(ControlSettingsRefusal::ConflictOrRefusal)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn spec_085_migration_du_reglage_execution_conserve_host_et_ne_touche_aucune_liaison() {
        let path = execution_fixture_path();
        let migrated = load_or_initialize_execution_settings(&path).unwrap();
        assert_eq!(migrated.generation, 1);
        assert_eq!(migrated.default_backend, ExecutionDefaultBackend::Host);
        assert_eq!(migrated.default_policy_id, None);
        assert_eq!(migrated.default_policy_version, None);
        assert_eq!(
            load_execution_settings(&path).unwrap(),
            migrated,
            "la migration n'écrit qu'un réglage Host déterministe"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn spec_085_execution_default_refuse_docker_tant_que_la_capacite_est_incomplete() {
        let path = execution_fixture_path();
        let host = load_or_initialize_execution_settings(&path).unwrap();
        let change = ExecutionSettingsChange {
            command_id: "execution-docker-incomplete".to_string(),
            expected_generation: host.generation,
            default_backend: ExecutionDefaultBackend::Docker,
            default_policy_id: Some("production".to_string()),
            default_policy_version: Some(1),
        };
        assert_eq!(
            preview_execution_settings(&path, &change, &RuntimeCapability::host_only()),
            Err(ControlSettingsRefusal::RuntimeCapabilityIncomplete)
        );
        assert_eq!(load_execution_settings(&path).unwrap(), host);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn spec_085_execution_default_docker_est_versionne_et_idempotent() {
        let path = execution_fixture_path();
        let host = load_or_initialize_execution_settings(&path).unwrap();
        let change = ExecutionSettingsChange {
            command_id: "execution-docker-v1".to_string(),
            expected_generation: host.generation,
            default_backend: ExecutionDefaultBackend::Docker,
            default_policy_id: Some("production".to_string()),
            default_policy_version: Some(7),
        };
        let preview =
            preview_execution_settings(&path, &change, &complete_runtime_capability()).unwrap();
        assert_eq!(preview.generation, 2);
        let applied =
            apply_execution_settings(&path, change.clone(), &complete_runtime_capability(), 123)
                .unwrap();
        assert_eq!(
            applied,
            preview
                .clone()
                .with_receipt_for_test("execution-docker-v1", 1, 123)
        );
        assert_eq!(
            apply_execution_settings(&path, change, &complete_runtime_capability(), 999).unwrap(),
            applied,
            "un rejeu ne doit ni réécrire ni augmenter la génération"
        );
        let _ = fs::remove_file(path);
    }

    trait ExecutionSettingsTestReceipt {
        fn with_receipt_for_test(
            self,
            command_id: &str,
            expected_generation: u64,
            observed_at: i64,
        ) -> Self;
    }

    impl ExecutionSettingsTestReceipt for ServerExecutionSettings {
        fn with_receipt_for_test(
            mut self,
            command_id: &str,
            expected_generation: u64,
            observed_at: i64,
        ) -> Self {
            self.last_receipt = Some(ExecutionSettingsReceipt {
                command_id: command_id.to_string(),
                expected_generation,
                resulting_generation: self.generation,
                default_backend: self.default_backend,
                default_policy_id: self.default_policy_id.clone(),
                default_policy_version: self.default_policy_version,
                observed_at,
            });
            self
        }
    }

    #[test]
    fn spec_086_dogfooding_est_desactive_par_defaut_et_refuse_les_transitions_non_admissibles() {
        let current = DogfoodingBridgetState {
            project_id: "bridget-system".to_string(),
            binding_generation: 4,
            setting_generation: 1,
            mode: DogfoodingBridgetMode::Disabled,
        };
        let change = DogfoodingBridgetChange {
            command_id: "dogfooding-086-enable".to_string(),
            expected_setting_generation: 1,
            expected_binding_generation: 4,
            requested_mode: DogfoodingBridgetMode::Enabled,
        };
        let preview =
            preview_dogfooding_bridget(&current, &change, ProjectBackend::Docker, false).unwrap();
        assert!(preview.requires_recreate);
        assert_eq!(preview.next.setting_generation, 2);
        assert_eq!(preview.next.mode, DogfoodingBridgetMode::Enabled);
        assert_eq!(current.mode, DogfoodingBridgetMode::Disabled);
        assert_eq!(
            dogfooding_bridget_descriptor(false).access,
            SettingAccess::ReadOnly
        );
        assert!(
            dogfooding_bridget_descriptor(true)
                .summary
                .contains("jamais merge")
        );
        assert_eq!(
            preview_dogfooding_bridget(&current, &change, ProjectBackend::Host, false),
            Err(DogfoodingBridgetRefusal::DockerRequired)
        );
        assert_eq!(
            preview_dogfooding_bridget(&current, &change, ProjectBackend::Docker, true),
            Err(DogfoodingBridgetRefusal::ActiveSystemAgent)
        );
        let mut stale = change.clone();
        stale.expected_setting_generation = 2;
        assert_eq!(
            preview_dogfooding_bridget(&current, &stale, ProjectBackend::Docker, false),
            Err(DogfoodingBridgetRefusal::GenerationMismatch)
        );
        let mut stale_binding = change;
        stale_binding.expected_binding_generation = 5;
        assert_eq!(
            preview_dogfooding_bridget(&current, &stale_binding, ProjectBackend::Docker, false),
            Err(DogfoodingBridgetRefusal::BindingGenerationMismatch)
        );
    }
}
