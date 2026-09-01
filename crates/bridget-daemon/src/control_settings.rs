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
use bridget_transport::protocol::ProjectRegistryRefusal;
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
    let id = change.command_id.as_bytes();
    let valid_id = !id.is_empty()
        && id.len() <= 160
        && id
            .iter()
            .all(|byte| (*byte).is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_'));
    if valid_id && change.expected_generation > 0 && !change.allowed_project_roots.is_empty() {
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
}
