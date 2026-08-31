//! Prévisualisation bornée des dossiers projet pour l'interface locale.
//!
//! Ce module ne crée, ne déplace ni ne supprime jamais de dossier. Il accepte
//! uniquement des chemins validés par ProjectRootPolicy et exécute, pour le
//! diagnostic Git, une commande fixe sans shell et sans argument contrôlé hors
//! du chemin déjà validé.

use crate::project_policy::ProjectRootPolicy;
use bridget_transport::protocol::ProjectRegistryRefusal;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectFolderMode {
    Create,
    Import,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitDiagnostic {
    Absent,
    Clean,
    Modified,
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPreview {
    pub mode: ProjectFolderMode,
    pub canonical_path: PathBuf,
    pub display_name: String,
    pub git: GitDiagnostic,
    pub git_initialization_proposed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinatorConfiguration {
    pub launcher_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub effort: String,
    pub permission_profile_ref: String,
    pub resolved_definition_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryWindow {
    seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectWorkspaceError {
    InvalidFolderName,
    FolderAlreadyExists,
    FolderMissing,
    Io(String),
    Refusal(ProjectRegistryRefusal),
    InvalidDiscoveryDuration,
}

impl std::fmt::Display for ProjectWorkspaceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFolderName => formatter.write_str("nom de dossier projet invalide"),
            Self::FolderAlreadyExists => formatter.write_str("le dossier projet existe déjà"),
            Self::FolderMissing => formatter.write_str("le dossier projet est introuvable"),
            Self::Io(message) => write!(formatter, "diagnostic de dossier impossible: {message}"),
            Self::Refusal(refusal) => {
                write!(formatter, "dossier refusé par la politique: {refusal:?}")
            }
            Self::InvalidDiscoveryDuration => {
                formatter.write_str("durée de découverte invalide ou supérieure à deux heures")
            }
        }
    }
}

impl std::error::Error for ProjectWorkspaceError {}

impl From<ProjectRegistryRefusal> for ProjectWorkspaceError {
    fn from(value: ProjectRegistryRefusal) -> Self {
        Self::Refusal(value)
    }
}

impl ProjectPreview {
    /// Prévisualise une création sans faire d'I/O mutante.
    pub fn create(
        policy: &ProjectRootPolicy,
        requested_parent: &Path,
        folder_name: &str,
    ) -> Result<Self, ProjectWorkspaceError> {
        validate_folder_name(folder_name)?;
        let parent = policy.validate_requested_root(requested_parent)?;
        let candidate = parent.join(folder_name);
        if candidate.exists() {
            return Err(ProjectWorkspaceError::FolderAlreadyExists);
        }
        Ok(Self {
            mode: ProjectFolderMode::Create,
            canonical_path: candidate,
            display_name: folder_name.to_string(),
            git: GitDiagnostic::Absent,
            git_initialization_proposed: true,
        })
    }

    /// Prévisualise un import. Le diagnostic ne lit aucun contenu de fichier.
    pub fn import(
        policy: &ProjectRootPolicy,
        requested_root: &Path,
    ) -> Result<Self, ProjectWorkspaceError> {
        let canonical_path = policy.validate_requested_root(requested_root)?;
        let display_name = canonical_path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or(ProjectWorkspaceError::InvalidFolderName)?
            .to_string();
        let git = inspect_git(&canonical_path)?;
        Ok(Self {
            mode: ProjectFolderMode::Import,
            canonical_path,
            display_name,
            git_initialization_proposed: git == GitDiagnostic::Absent,
            git,
        })
    }
}

impl CoordinatorConfiguration {
    pub fn is_compatible_with(&self, resolved_definition_digest: &str) -> bool {
        !self.launcher_id.is_empty()
            && !self.provider_id.is_empty()
            && !self.model_id.is_empty()
            && !self.effort.is_empty()
            && !self.permission_profile_ref.is_empty()
            && self.resolved_definition_digest == resolved_definition_digest
    }
}

impl DiscoveryWindow {
    pub const DEFAULT_SECONDS: u32 = 10 * 60;
    pub const MAX_SECONDS: u32 = 2 * 60 * 60;

    pub fn new(seconds: u32) -> Result<Self, ProjectWorkspaceError> {
        if seconds == 0 || seconds > Self::MAX_SECONDS {
            return Err(ProjectWorkspaceError::InvalidDiscoveryDuration);
        }
        Ok(Self { seconds })
    }

    pub fn seconds(self) -> u32 {
        self.seconds
    }
}

fn validate_folder_name(name: &str) -> Result<(), ProjectWorkspaceError> {
    let path = Path::new(name);
    if name.is_empty()
        || name == "."
        || name == ".."
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(ProjectWorkspaceError::InvalidFolderName);
    }
    Ok(())
}

fn inspect_git(path: &Path) -> Result<GitDiagnostic, ProjectWorkspaceError> {
    let result = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|error| ProjectWorkspaceError::Io(error.to_string()))?;
    if !result.status.success() {
        return Ok(GitDiagnostic::Absent);
    }

    let worktree = path.join(".git").is_file();
    let status = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .output()
        .map_err(|error| ProjectWorkspaceError::Io(error.to_string()))?;
    if !status.status.success() {
        return Err(ProjectWorkspaceError::Io(
            "git status a refusé le dossier validé".to_string(),
        ));
    }
    if worktree {
        return Ok(GitDiagnostic::Worktree);
    }
    if status.stdout.is_empty() {
        Ok(GitDiagnostic::Clean)
    } else {
        Ok(GitDiagnostic::Modified)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CoordinatorConfiguration, DiscoveryWindow, GitDiagnostic, ProjectFolderMode,
        ProjectPreview, ProjectWorkspaceError,
    };
    use crate::project_policy::ProjectRootPolicy;
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        policy_path: PathBuf,
        projects: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "bridget-spec-076-workspace-{}-{nonce}",
                std::process::id()
            ));
            let projects = root.join("projects");
            fs::create_dir_all(&projects).unwrap();
            let policy_path = root.join("policy.json");
            fs::write(
                &policy_path,
                serde_json::to_vec(&json!({
                    "contract_version": 1,
                    "allowed_project_roots": [projects],
                    "policy_generation": 1,
                }))
                .unwrap(),
            )
            .unwrap();
            fs::set_permissions(&policy_path, fs::Permissions::from_mode(0o600)).unwrap();
            Self {
                root,
                policy_path,
                projects,
            }
        }

        fn policy(&self) -> ProjectRootPolicy {
            ProjectRootPolicy::load(&self.policy_path).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn spec_076_previsualisation_create_ne_cree_ni_n_ecrase() {
        let fixture = Fixture::new();
        let policy = fixture.policy();

        let preview = ProjectPreview::create(&policy, &fixture.projects, "nouveau").unwrap();

        assert_eq!(preview.mode, ProjectFolderMode::Create);
        assert_eq!(preview.display_name, "nouveau");
        assert!(!preview.canonical_path.exists());
        assert!(preview.git_initialization_proposed);
        assert_eq!(
            ProjectPreview::create(&policy, &fixture.projects, "../sortie"),
            Err(ProjectWorkspaceError::InvalidFolderName)
        );

        fs::create_dir_all(fixture.projects.join("nouveau")).unwrap();
        assert_eq!(
            ProjectPreview::create(&policy, &fixture.projects, "nouveau"),
            Err(ProjectWorkspaceError::FolderAlreadyExists)
        );
    }

    #[test]
    fn spec_076_import_non_git_ne_modifie_pas_le_dossier() {
        let fixture = Fixture::new();
        let policy = fixture.policy();
        let imported = fixture.projects.join("ancien");
        fs::create_dir_all(&imported).unwrap();
        fs::write(imported.join("preexistant.txt"), "conserver").unwrap();

        let preview = ProjectPreview::import(&policy, &imported).unwrap();

        assert_eq!(preview.mode, ProjectFolderMode::Import);
        assert_eq!(preview.git, GitDiagnostic::Absent);
        assert!(preview.git_initialization_proposed);
        assert_eq!(
            fs::read_to_string(imported.join("preexistant.txt")).unwrap(),
            "conserver"
        );
        assert!(!imported.join(".git").exists());
    }

    #[test]
    fn spec_076_duree_decouverte_est_bornee() {
        assert_eq!(
            DiscoveryWindow::new(DiscoveryWindow::DEFAULT_SECONDS)
                .unwrap()
                .seconds(),
            600
        );
        assert_eq!(DiscoveryWindow::new(120 * 60).unwrap().seconds(), 7200);
        assert_eq!(
            DiscoveryWindow::new(0),
            Err(ProjectWorkspaceError::InvalidDiscoveryDuration)
        );
        assert_eq!(
            DiscoveryWindow::new(120 * 60 + 1),
            Err(ProjectWorkspaceError::InvalidDiscoveryDuration)
        );
    }

    #[test]
    fn spec_076_configuration_coordinateur_exige_le_digest_atteste() {
        let configuration = CoordinatorConfiguration {
            launcher_id: "codex".to_string(),
            provider_id: "openai".to_string(),
            model_id: "gpt-5.6".to_string(),
            effort: "medium".to_string(),
            permission_profile_ref: "lecture-seule".to_string(),
            resolved_definition_digest: "sha256:atteste".to_string(),
        };
        assert!(configuration.is_compatible_with("sha256:atteste"));
        assert!(!configuration.is_compatible_with("sha256:divergent"));
    }
}
