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
use std::path::{Path, PathBuf};

pub const CONTROL_SETTINGS_VERSION: u8 = 1;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_policy::ProjectLocationKind;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn catalogue_est_ferme_et_seule_la_politique_projet_devient_modifiable() {
        let descriptors = server_setting_descriptors(true);
        assert_eq!(descriptors.len(), 4);
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.key == "project_roots.allowed_roots"
                && descriptor.access == SettingAccess::Writable
        }));
        assert!(
            descriptors
                .iter()
                .filter(|descriptor| descriptor.access == SettingAccess::Writable)
                .count()
                == 1
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
}
