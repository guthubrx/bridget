//! Politique hôte fermée des racines pouvant être liées à un projet.

use bridget_transport::protocol::ProjectRegistryRefusal;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::ffi::{CStr, OsStr};
use std::fs::OpenOptions;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const PROJECT_ROOT_POLICY_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectRootPolicyDocument {
    contract_version: u16,
    allowed_project_roots: Vec<PathBuf>,
}

/// Racines canoniques explicitement autorisées pour de nouvelles liaisons.
///
/// Une instance est chargée une fois au démarrage du daemon. Elle ne porte
/// jamais de valeur implicite: l'absence de document ferme les mutations du
/// registre sans affecter les lancements historiques.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRootPolicy {
    allowed_roots: Vec<PathBuf>,
}

impl ProjectRootPolicy {
    pub fn load(source: &Path) -> Result<Self, ProjectRegistryRefusal> {
        Self::load_for_owner(source, unsafe { libc::geteuid() }, daemon_home().as_deref())
    }

    fn load_for_owner(
        source: &Path,
        expected_owner: u32,
        daemon_home: Option<&Path>,
    ) -> Result<Self, ProjectRegistryRefusal> {
        if !source.is_absolute() {
            return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
        }

        let link_metadata = std::fs::symlink_metadata(source).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                ProjectRegistryRefusal::ProjectRootPolicyUnavailable
            } else {
                ProjectRegistryRefusal::ProjectRootPolicyInvalid
            }
        })?;
        if link_metadata.file_type().is_symlink() {
            return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
        }

        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(source)
            .map_err(|_| ProjectRegistryRefusal::ProjectRootPolicyInvalid)?;
        let metadata = file
            .metadata()
            .map_err(|_| ProjectRegistryRefusal::ProjectRootPolicyInvalid)?;
        if !metadata.is_file() {
            return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
        }
        if metadata.uid() != expected_owner || metadata.permissions().mode() & 0o022 != 0 {
            return Err(ProjectRegistryRefusal::ProjectRootPolicyPermissionsInvalid);
        }

        let mut content = String::new();
        std::io::Read::read_to_string(&mut file, &mut content)
            .map_err(|_| ProjectRegistryRefusal::ProjectRootPolicyInvalid)?;
        let document: ProjectRootPolicyDocument = serde_json::from_str(&content)
            .map_err(|_| ProjectRegistryRefusal::ProjectRootPolicyInvalid)?;
        if document.contract_version != PROJECT_ROOT_POLICY_CONTRACT_VERSION
            || document.allowed_project_roots.is_empty()
        {
            return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
        }

        let mut seen = BTreeSet::new();
        let mut allowed_roots = Vec::with_capacity(document.allowed_project_roots.len());
        for root in document.allowed_project_roots {
            let canonical_root = canonical_directory(&root)?;
            if is_broad_root(&canonical_root, daemon_home)
                || canonical_root
                    .metadata()
                    .map_err(|_| ProjectRegistryRefusal::ProjectRootPolicyInvalid)?
                    .uid()
                    != expected_owner
                || !seen.insert(canonical_root.clone())
            {
                return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
            }
            allowed_roots.push(canonical_root);
        }

        Ok(Self { allowed_roots })
    }

    /// Canonicalise une racine candidate et vérifie qu'elle reste sous une
    /// frontière explicitement autorisée.
    pub fn validate_requested_root(
        &self,
        requested_root: &Path,
    ) -> Result<PathBuf, ProjectRegistryRefusal> {
        if !requested_root.is_absolute() {
            return Err(ProjectRegistryRefusal::InvalidAbsoluteRoot);
        }
        let canonical_root = std::fs::canonicalize(requested_root).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                ProjectRegistryRefusal::RootMissing
            } else {
                ProjectRegistryRefusal::InvalidAbsoluteRoot
            }
        })?;
        if !canonical_root.is_dir() {
            return Err(ProjectRegistryRefusal::RootNotDirectory);
        }
        if is_broad_root(&canonical_root, daemon_home().as_deref()) {
            return Err(ProjectRegistryRefusal::RootTooBroad);
        }
        if !self
            .allowed_roots
            .iter()
            .any(|allowed_root| canonical_root.starts_with(allowed_root))
        {
            return Err(ProjectRegistryRefusal::RootOutsideAllowedPrefixes);
        }
        Ok(canonical_root)
    }
}

fn canonical_directory(root: &Path) -> Result<PathBuf, ProjectRegistryRefusal> {
    if !root.is_absolute() {
        return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
    }
    let canonical_root = std::fs::canonicalize(root)
        .map_err(|_| ProjectRegistryRefusal::ProjectRootPolicyInvalid)?;
    if !canonical_root.is_dir() {
        return Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid);
    }
    Ok(canonical_root)
}

fn is_broad_root(root: &Path, daemon_home: Option<&Path>) -> bool {
    root == Path::new("/")
        || root == Path::new("/home")
        || root == Path::new("/Users")
        || daemon_home.is_some_and(|home| root == home)
}

fn daemon_home() -> Option<PathBuf> {
    let passwd = unsafe { libc::getpwuid(libc::geteuid()) };
    if passwd.is_null() || unsafe { (*passwd).pw_dir }.is_null() {
        return None;
    }
    let directory = unsafe { CStr::from_ptr((*passwd).pw_dir) };
    let path = PathBuf::from(OsStr::from_bytes(directory.to_bytes()));
    std::fs::canonicalize(path).ok()
}

#[cfg(test)]
mod tests {
    use super::ProjectRootPolicy;
    use bridget_transport::protocol::ProjectRegistryRefusal;
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        policy: PathBuf,
        allowed: PathBuf,
        outside: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "bridget-spec-065-project-policy-{}-{nonce}",
                std::process::id()
            ));
            let allowed = root.join("projects");
            let outside = root.join("outside");
            fs::create_dir_all(allowed.join("worktree")).unwrap();
            fs::create_dir_all(&outside).unwrap();
            Self {
                policy: root.join("project-root-policy.json"),
                root,
                allowed,
                outside,
            }
        }

        fn write_policy(&self, value: serde_json::Value) {
            fs::write(&self.policy, serde_json::to_vec(&value).unwrap()).unwrap();
            fs::set_permissions(&self.policy, fs::Permissions::from_mode(0o600)).unwrap();
        }

        fn valid_document(&self) -> serde_json::Value {
            json!({
                "contract_version": 1,
                "allowed_project_roots": [self.allowed],
            })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn spec_065_politique_absente_ferme_les_mutations() {
        let fixture = Fixture::new();

        assert_eq!(
            ProjectRootPolicy::load(&fixture.policy),
            Err(ProjectRegistryRefusal::ProjectRootPolicyUnavailable)
        );
    }

    #[test]
    fn spec_065_politique_refuse_schema_vide_inconnu_ou_frontiere_large() {
        let fixture = Fixture::new();
        let daemon_home = std::env::var_os("HOME").map(PathBuf::from);
        let mut invalid_documents = vec![
            json!({"contract_version": 1, "allowed_project_roots": []}),
            json!({"contract_version": 2, "allowed_project_roots": [fixture.allowed]}),
            json!({
                "contract_version": 1,
                "allowed_project_roots": [fixture.allowed],
                "unknown": true,
            }),
            json!({"contract_version": 1, "allowed_project_roots": ["/"]}),
            json!({"contract_version": 1, "allowed_project_roots": ["/home"]}),
        ];
        if let Some(home) = daemon_home {
            invalid_documents.push(json!({
                "contract_version": 1,
                "allowed_project_roots": [home],
            }));
        }

        for document in invalid_documents {
            fixture.write_policy(document);
            assert_eq!(
                ProjectRootPolicy::load(&fixture.policy),
                Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid)
            );
        }
    }

    #[test]
    fn spec_065_politique_refuse_proprietaire_ou_mode_non_prive() {
        let fixture = Fixture::new();
        fixture.write_policy(fixture.valid_document());

        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o620)).unwrap();
        assert_eq!(
            ProjectRootPolicy::load(&fixture.policy),
            Err(ProjectRegistryRefusal::ProjectRootPolicyPermissionsInvalid)
        );

        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
        let unexpected_owner = unsafe { libc::geteuid() }.saturating_add(1);
        assert_eq!(
            ProjectRootPolicy::load_for_owner(&fixture.policy, unexpected_owner, None),
            Err(ProjectRegistryRefusal::ProjectRootPolicyPermissionsInvalid)
        );
    }

    #[test]
    fn spec_065_politique_canonicalise_et_contient_les_racines_candidates() {
        let fixture = Fixture::new();
        fixture.write_policy(fixture.valid_document());
        let policy = ProjectRootPolicy::load(&fixture.policy).unwrap();
        let expected = fixture.allowed.canonicalize().unwrap();
        let candidate = fixture.allowed.join("worktree/..");

        assert_eq!(
            policy.validate_requested_root(&candidate).unwrap(),
            expected
        );
        assert_eq!(
            policy.validate_requested_root(&fixture.outside),
            Err(ProjectRegistryRefusal::RootOutsideAllowedPrefixes)
        );
        assert_eq!(
            policy.validate_requested_root(Path::new("/")),
            Err(ProjectRegistryRefusal::RootTooBroad)
        );
    }

    #[test]
    fn spec_065_politique_refuse_les_alias_de_politique_et_canonicalise_les_symlinks() {
        let fixture = Fixture::new();
        fixture.write_policy(fixture.valid_document());
        let policy_alias = fixture.root.join("policy-alias.json");
        symlink(&fixture.policy, &policy_alias).unwrap();
        assert_eq!(
            ProjectRootPolicy::load(&policy_alias),
            Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid)
        );

        let alias = fixture.root.join("allowed-alias");
        symlink(&fixture.allowed, &alias).unwrap();
        fixture.write_policy(json!({
            "contract_version": 1,
            "allowed_project_roots": [fixture.allowed, alias],
        }));
        assert_eq!(
            ProjectRootPolicy::load(&fixture.policy),
            Err(ProjectRegistryRefusal::ProjectRootPolicyInvalid)
        );

        fixture.write_policy(fixture.valid_document());
        let policy = ProjectRootPolicy::load(&fixture.policy).unwrap();
        let candidate_alias = fixture.root.join("candidate-alias");
        symlink(fixture.allowed.join("worktree"), &candidate_alias).unwrap();
        assert_eq!(
            policy.validate_requested_root(&candidate_alias).unwrap(),
            fixture.allowed.join("worktree").canonicalize().unwrap()
        );

        let outside_alias = fixture.root.join("outside-alias");
        symlink(&fixture.outside, &outside_alias).unwrap();
        assert_eq!(
            policy.validate_requested_root(&outside_alias),
            Err(ProjectRegistryRefusal::RootOutsideAllowedPrefixes)
        );
    }
}
