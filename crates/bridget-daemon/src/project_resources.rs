use super::RuntimeIssue;
use bridget_transport::protocol::{
    ProjectResourceKind, ProjectResourceRef, ResolvedProjectResource,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectResourceCatalog {
    pub contract_version: u16,
    pub extension_roots: Vec<PathBuf>,
    pub secret_roots: Vec<PathBuf>,
    pub sources: Vec<ProjectResourceSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectResourceSource {
    pub source_ref: String,
    pub kind: ProjectResourceKind,
    pub expected_uid: u32,
    pub expected_gid: u32,
    pub canonical_path: PathBuf,
    pub source_revision: u64,
    pub allowed_project_ids: Vec<String>,
}

impl ProjectResourceCatalog {
    pub fn from_json(value: &str) -> Result<Self, RuntimeIssue> {
        let catalog: Self = serde_json::from_str(value)
            .map_err(|_| RuntimeIssue::PolicyInvalid("catalogue invalide".to_string()))?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn load(path: &Path) -> Result<Self, RuntimeIssue> {
        let metadata = fs::symlink_metadata(path).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.mode() & 0o022 != 0
        {
            return Err(RuntimeIssue::PolicyInvalid(
                "catalogue non prive".to_string(),
            ));
        }
        let bytes = fs::read(path).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
        if bytes.len() > 1024 * 1024 {
            return Err(RuntimeIssue::PolicyInvalid(
                "catalogue trop volumineux".to_string(),
            ));
        }
        let value = std::str::from_utf8(&bytes)
            .map_err(|_| RuntimeIssue::PolicyInvalid("catalogue invalide".to_string()))?;
        Self::from_json(value)
    }

    pub fn source_for(
        &self,
        project_id: &str,
        source_ref: &str,
    ) -> Result<&ProjectResourceSource, RuntimeIssue> {
        let source = self
            .sources
            .iter()
            .find(|source| source.source_ref == source_ref)
            .ok_or_else(|| RuntimeIssue::PolicyInvalid("source catalogue absente".to_string()))?;
        if !source
            .allowed_project_ids
            .iter()
            .any(|allowed| allowed == project_id)
        {
            return Err(RuntimeIssue::PolicyInvalid(
                "projet non autorise".to_string(),
            ));
        }
        Ok(source)
    }

    fn validate(&self) -> Result<(), RuntimeIssue> {
        if self.contract_version != 1
            || self
                .extension_roots
                .iter()
                .chain(&self.secret_roots)
                .any(|root| !root.is_absolute())
        {
            return Err(RuntimeIssue::PolicyInvalid(
                "catalogue invalide".to_string(),
            ));
        }
        let mut source_refs = HashSet::new();
        for source in &self.sources {
            let allowed_roots = match source.kind {
                ProjectResourceKind::Extension => &self.extension_roots,
                ProjectResourceKind::SecretFile
                | ProjectResourceKind::SecretDirectory
                | ProjectResourceKind::SecretProcessEnv => &self.secret_roots,
            };
            if source.source_ref.trim().is_empty()
                || !source.canonical_path.is_absolute()
                || source
                    .canonical_path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
                || source.source_revision == 0
                || source.expected_uid == 0
                || source.expected_gid == 0
                || source.allowed_project_ids.is_empty()
                || !source_refs.insert(source.source_ref.as_str())
                || !allowed_roots
                    .iter()
                    .any(|root| source.canonical_path.starts_with(root))
                || source
                    .allowed_project_ids
                    .iter()
                    .any(|id| id.trim().is_empty() || id == "*")
            {
                return Err(RuntimeIssue::PolicyInvalid(
                    "source catalogue invalide".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretSourceStamp {
    pub source_ref: String,
    pub kind: ProjectResourceKind,
    pub device: u64,
    pub inode: u64,
    pub size: u64,
    pub mtime_nsec: i64,
    pub ctime_nsec: i64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProjectResourceSource {
    pub source: ProjectResourceSource,
    pub attestation_digest: String,
    pub secret_stamp: Option<SecretSourceStamp>,
}

impl ProjectResourceCatalog {
    pub fn attest(
        &self,
        project_id: &str,
        source_ref: &str,
    ) -> Result<ResolvedProjectResourceSource, RuntimeIssue> {
        let source = self.source_for(project_id, source_ref)?;
        let entries = self.validate_source(source)?;
        let root = entries
            .first()
            .ok_or_else(|| RuntimeIssue::PolicyInvalid("source catalogue vide".to_string()))?;
        let manifest_digest = digest_manifest(&entries);
        let attestation_digest = format!(
            "sha256:{:x}",
            Sha256::digest(format!("{}:{}", source.source_revision, manifest_digest))
        );
        let secret_stamp = match source.kind {
            ProjectResourceKind::Extension => None,
            ProjectResourceKind::SecretFile
            | ProjectResourceKind::SecretDirectory
            | ProjectResourceKind::SecretProcessEnv => Some(SecretSourceStamp {
                source_ref: source.source_ref.clone(),
                kind: source.kind,
                device: root.device,
                inode: root.inode,
                size: root.size,
                mtime_nsec: root.mtime_nsec,
                ctime_nsec: root.ctime_nsec,
                mode: root.mode,
                uid: root.uid,
                gid: root.gid,
                manifest_digest,
            }),
        };
        Ok(ResolvedProjectResourceSource {
            source: source.clone(),
            attestation_digest,
            secret_stamp,
        })
    }
}

impl ProjectResourceCatalog {
    pub fn resolve_refs(
        &self,
        project_id: &str,
        references: &[ProjectResourceRef],
    ) -> Result<Vec<ResolvedProjectResource>, RuntimeIssue> {
        let mut destinations = HashSet::new();
        let mut resources = Vec::with_capacity(references.len());
        for reference in references {
            if reference.resource_id.trim().is_empty() || reference.generation == 0 {
                return Err(RuntimeIssue::PolicyInvalid(
                    "reference ressource invalide".to_string(),
                ));
            }
            let resolved = self.attest(project_id, &reference.source_ref)?;

            if reference.kind != resolved.source.kind {
                return Err(RuntimeIssue::PolicyInvalid(
                    "type ressource divergent".to_string(),
                ));
            }
            validate_destination(reference)?;
            if !destinations.insert(reference.destination.as_str()) {
                return Err(RuntimeIssue::PolicyInvalid(
                    "destination ressource dupliquee".to_string(),
                ));
            }
            if reference.kind == ProjectResourceKind::Extension
                && reference.content_digest.as_deref() != Some(resolved.attestation_digest.as_str())
            {
                return Err(RuntimeIssue::PolicyInvalid(
                    "digest extension divergent".to_string(),
                ));
            }
            resources.push(ResolvedProjectResource {
                reference: reference.clone(),
                source_revision: resolved.source.source_revision,
                attestation_digest: resolved.attestation_digest,
            });
        }
        Ok(resources)
    }
}

#[derive(Debug, Clone)]
struct SourceMetadata {
    relative_path: PathBuf,
    device: u64,
    inode: u64,
    size: u64,
    mtime_nsec: i64,
    ctime_nsec: i64,
    mode: u32,
    uid: u32,
    gid: u32,
    is_file: bool,
    is_directory: bool,
}
impl ProjectResourceCatalog {
    fn validate_source(
        &self,
        source: &ProjectResourceSource,
    ) -> Result<Vec<SourceMetadata>, RuntimeIssue> {
        let allowed_roots = match source.kind {
            ProjectResourceKind::Extension => &self.extension_roots,
            ProjectResourceKind::SecretFile
            | ProjectResourceKind::SecretDirectory
            | ProjectResourceKind::SecretProcessEnv => &self.secret_roots,
        };
        let canonical_source = fs::canonicalize(&source.canonical_path)
            .map_err(|_| RuntimeIssue::PolicyInvalid("source catalogue absente".to_string()))?;
        let in_declared_root = allowed_roots
            .iter()
            .filter_map(|root| fs::canonicalize(root).ok())
            .any(|root| canonical_source.starts_with(root));
        if canonical_source != source.canonical_path || !in_declared_root {
            return Err(RuntimeIssue::PolicyInvalid(
                "source catalogue non canonique".to_string(),
            ));
        }
        let mut entries = Vec::new();
        collect_source_metadata(&source.canonical_path, PathBuf::new(), &mut entries)?;
        if entries.is_empty() {
            return Err(RuntimeIssue::PolicyInvalid(
                "source catalogue vide".to_string(),
            ));
        }
        let root = &entries[0];
        if root.uid != source.expected_uid || root.gid != source.expected_gid {
            return Err(RuntimeIssue::PolicyInvalid(
                "proprietaire source invalide".to_string(),
            ));
        }
        let private = matches!(
            source.kind,
            ProjectResourceKind::SecretFile
                | ProjectResourceKind::SecretDirectory
                | ProjectResourceKind::SecretProcessEnv
        );
        let valid_kind = match source.kind {
            ProjectResourceKind::Extension => root.is_file || root.is_directory,
            ProjectResourceKind::SecretFile | ProjectResourceKind::SecretProcessEnv => root.is_file,
            ProjectResourceKind::SecretDirectory => root.is_directory,
        };
        if !valid_kind {
            return Err(RuntimeIssue::PolicyInvalid(
                "type source secret invalide".to_string(),
            ));
        }
        for entry in &entries {
            if entry.uid != source.expected_uid || entry.gid != source.expected_gid {
                return Err(RuntimeIssue::PolicyInvalid(
                    "proprietaire source invalide".to_string(),
                ));
            }
            if private && (entry.is_file || entry.is_directory) && entry.mode & 0o077 != 0 {
                return Err(RuntimeIssue::PolicyInvalid(
                    "permission secret invalide".to_string(),
                ));
            }
            if !private && entry.mode & 0o022 != 0 {
                return Err(RuntimeIssue::PolicyInvalid(
                    "extension inscriptible refusee".to_string(),
                ));
            }
        }
        Ok(entries)
    }
}
fn collect_source_metadata(
    path: &Path,
    relative_path: PathBuf,
    entries: &mut Vec<SourceMetadata>,
) -> Result<(), RuntimeIssue> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| RuntimeIssue::PolicyInvalid("source catalogue absente".to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(RuntimeIssue::PolicyInvalid(
            "lien symbolique refuse".to_string(),
        ));
    }
    let file_type = metadata.file_type();
    if !file_type.is_file() && !file_type.is_dir() {
        return Err(RuntimeIssue::PolicyInvalid(
            "type source invalide".to_string(),
        ));
    }
    entries.push(SourceMetadata {
        relative_path: relative_path.clone(),
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.size(),
        mtime_nsec: metadata.mtime_nsec(),
        ctime_nsec: metadata.ctime_nsec(),
        mode: metadata.mode() & 0o7777,
        uid: metadata.uid(),
        gid: metadata.gid(),
        is_file: file_type.is_file(),
        is_directory: file_type.is_dir(),
    });
    if file_type.is_dir() {
        let mut children = fs::read_dir(path)
            .map_err(|_| RuntimeIssue::PolicyInvalid("source catalogue absente".to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| RuntimeIssue::PolicyInvalid("source catalogue absente".to_string()))?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            collect_source_metadata(
                &child.path(),
                relative_path.join(child.file_name()),
                entries,
            )?;
        }
    }
    Ok(())
}
fn digest_manifest(entries: &[SourceMetadata]) -> String {
    let mut material = String::new();
    for entry in entries {
        material.push_str(&format!(
            "{}\\0{}\\0{}\\0{}\\0{}\\0{}\\0{}\\0{}\\0{}\\0{}\\n",
            entry.relative_path.display(),
            entry.device,
            entry.inode,
            entry.size,
            entry.mtime_nsec,
            entry.ctime_nsec,
            entry.mode,
            entry.uid,
            entry.gid,
            entry.is_directory,
        ));
    }
    format!("sha256:{:x}", Sha256::digest(material))
}

fn validate_destination(reference: &ProjectResourceRef) -> Result<(), RuntimeIssue> {
    let destination = reference.destination.as_str();
    let valid = match reference.kind {
        ProjectResourceKind::Extension => {
            destination.starts_with("/opt/bridget/extensions/") && !destination.contains("..")
        }
        ProjectResourceKind::SecretFile | ProjectResourceKind::SecretDirectory => {
            destination.starts_with("/run/bridget/secrets/") && !destination.contains("..")
        }
        ProjectResourceKind::SecretProcessEnv => {
            !destination.is_empty()
                && destination
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == 95_u8)
        }
    };
    if !valid {
        return Err(RuntimeIssue::PolicyInvalid(
            "destination ressource invalide".to_string(),
        ));
    }
    Ok(())
}
