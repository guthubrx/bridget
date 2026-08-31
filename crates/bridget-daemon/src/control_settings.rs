//! Catalogue fermé des réglages serveur exposables au relais UI.
//!
//! Ce module est volontairement étroit : il ne connaît aucune commande shell,
//! aucun chemin de configuration libre et aucun secret. La première clé
//! modifiable est la liste de racines de projets, déjà protégée par
//! `ProjectRootPolicy`.

use crate::project_policy::{
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
            summary: "Racines de projets autorisées par le serveur.",
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

fn map_policy_error(error: ProjectRegistryRefusal) -> ControlSettingsRefusal {
    match error {
        ProjectRegistryRefusal::ProjectRootPolicyUnavailable => ControlSettingsRefusal::Unavailable,
        _ => ControlSettingsRefusal::ConflictOrRefusal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
