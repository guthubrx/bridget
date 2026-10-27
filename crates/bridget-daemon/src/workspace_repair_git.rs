//! Réattestation d'un worktree déplacé. Aucun fichier métier n'est déplacé.
use super::*;
use serde::{Deserialize, Serialize};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepairAnchor {
    pub original_path: PathBuf,
    pub path: PathBuf,
    pub git_dir: PathBuf,
    pub path_device: u64,
    pub path_inode: u64,
    pub git_device: u64,
    pub git_inode: u64,
    pub expected_head: String,
}

fn bounded_file(path: &Path) -> Result<String, String> {
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "workspace_repair_anchor_unconfirmed")?;
    if !file
        .metadata()
        .map_err(|_| "workspace_repair_anchor_unconfirmed")?
        .is_file()
    {
        return Err("workspace_repair_anchor_unconfirmed".into());
    }
    let mut text = String::new();
    file.take(8193)
        .read_to_string(&mut text)
        .map_err(|_| "workspace_repair_anchor_unconfirmed")?;
    if text.len() > 8192 {
        return Err("workspace_repair_anchor_unconfirmed".into());
    }
    Ok(text)
}

/// O(A+W+B), A ancêtres, W entrées Git bornées à4096, B rapport borné.
/// Les chemins du candidat ne deviennent jamais une autorité de dépôt.
pub(crate) fn inspect_repair(
    original: &WorkspaceRecord,
    target: &Path,
    expected_head: &str,
) -> Result<(RepairAnchor, WorkspaceRecord), String> {
    let old = Path::new(&original.path);
    let parent = Path::new(&original.toplevel);
    if !sha(expected_head)
        || !old.is_absolute()
        || !parent.is_absolute()
        || old == parent
        || !old.starts_with(parent)
        || !target.is_absolute()
        || target == old
        || target
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        || target.canonicalize().ok().as_deref() != Some(target)
        || parent.canonicalize().ok().as_deref() != Some(parent)
    {
        return Err("workspace_repair_destination_invalid".into());
    }
    let target_meta =
        fs::symlink_metadata(target).map_err(|_| "workspace_repair_destination_invalid")?;
    if !target_meta.is_dir() || target_meta.file_type().is_symlink() {
        return Err("workspace_repair_destination_invalid".into());
    }
    // L'ancienne entrée doit être absente ou être l'alias exact de la cible.
    // Un second dossier réel est un conflit, jamais une preuve de déplacement.
    if let Ok(meta) = fs::symlink_metadata(old) {
        if !meta.file_type().is_symlink() || old.canonicalize().ok().as_deref() != Some(target) {
            return Err("workspace_repair_original_conflict".into());
        }
    }
    let common_text =
        run_git(parent, &["rev-parse", "--git-common-dir"], READ_TIMEOUT)?.complete()?;
    let common_path = PathBuf::from(common_text);
    let common = if common_path.is_absolute() {
        common_path
    } else {
        parent.join(common_path)
    };
    let common = common
        .canonicalize()
        .map_err(|_| "workspace_repair_anchor_unconfirmed")?;
    // Même frontière que report_anchor : métadonnées réelles du parent.
    let common_meta =
        fs::symlink_metadata(&common).map_err(|_| "workspace_repair_anchor_unconfirmed")?;
    if !common_meta.is_dir() || common_meta.file_type().is_symlink() {
        return Err("workspace_repair_anchor_unconfirmed".into());
    }
    let registry = common.join("worktrees");
    let registry_meta =
        fs::symlink_metadata(&registry).map_err(|_| "workspace_repair_anchor_unconfirmed")?;
    if !registry_meta.is_dir()
        || registry_meta.file_type().is_symlink()
        || registry.canonicalize().ok().as_ref() != Some(&registry)
    {
        return Err("workspace_repair_anchor_unconfirmed".into());
    }
    let child_git = bounded_file(&target.join(".git"))?;
    for (index, item) in fs::read_dir(&registry)
        .map_err(|_| "workspace_repair_anchor_unconfirmed")?
        .enumerate()
    {
        if index >= 4096 {
            return Err("workspace_repair_anchor_unconfirmed".into());
        }
        let entry = item
            .map_err(|_| "workspace_repair_anchor_unconfirmed")?
            .path();
        let meta =
            fs::symlink_metadata(&entry).map_err(|_| "workspace_repair_anchor_unconfirmed")?;
        if !meta.is_dir()
            || meta.file_type().is_symlink()
            || entry.canonicalize().ok().as_ref() != Some(&entry)
        {
            continue;
        }
        let backlink = bounded_file(&entry.join("gitdir"))?;
        if Path::new(backlink.trim_end()) != old.join(".git") {
            continue;
        }
        // Le lien aller doit désigner CETTE entrée administrative, sans résolution
        // relative dépendante d'un dossier déplacé et sans métadonnées étrangères.
        if child_git.trim_end() != format!("gitdir: {}", entry.display()) {
            return Err("workspace_repair_anchor_unconfirmed".into());
        }
        let anchor = RepairAnchor {
            original_path: old.to_path_buf(),
            path: target.to_path_buf(),
            git_dir: entry,
            path_device: target_meta.dev(),
            path_inode: target_meta.ino(),
            git_device: meta.dev(),
            git_inode: meta.ino(),
            expected_head: expected_head.to_ascii_lowercase(),
        };
        let report = report_repair(original, &anchor, false)?;
        return Ok((anchor, report));
    }
    Err("workspace_repair_anchor_unconfirmed".into())
}

/// Relecture obligatoire avant chaque effet. Le booléen indique uniquement
/// si Git a déjà confirmé le changement de son lien administratif.
pub(crate) fn report_repair(
    original: &WorkspaceRecord,
    anchor: &RepairAnchor,
    repaired: bool,
) -> Result<WorkspaceRecord, String> {
    for (path, device, inode) in [
        (&anchor.path, anchor.path_device, anchor.path_inode),
        (&anchor.git_dir, anchor.git_device, anchor.git_inode),
    ] {
        let metadata =
            fs::symlink_metadata(path).map_err(|_| "workspace_repair_destination_changed")?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.dev() != device
            || metadata.ino() != inode
            || path.canonicalize().ok().as_ref() != Some(path)
        {
            return Err("workspace_repair_destination_changed".into());
        }
    }
    let expected = if repaired {
        &anchor.path
    } else {
        &anchor.original_path
    };
    if bounded_file(&anchor.git_dir.join("gitdir"))?.trim_end()
        != expected.join(".git").to_string_lossy()
        || bounded_file(&anchor.path.join(".git"))?.trim_end()
            != format!("gitdir: {}", anchor.git_dir.display())
    {
        return Err("workspace_repair_anchor_unconfirmed".into());
    }
    let mut workspace = original.clone();
    workspace.path = anchor.path.to_string_lossy().into_owned();
    let report = report_at(&workspace, &anchor.git_dir, &anchor.path)?;
    if report.head_commit.as_deref() != Some(anchor.expected_head.as_str())
        || report.head_on_branch != Some(true)
        || report.branch_commit != report.head_commit
    {
        return Err("workspace_repair_head_changed".into());
    }
    let ancestry = capture(
        &anchor.path,
        &[
            "merge-base",
            "--is-ancestor",
            &original.base_commit,
            &anchor.expected_head,
        ],
        READ_TIMEOUT,
        Some((&anchor.git_dir, &anchor.path)),
    )?;
    if !ancestry.success() || ancestry.truncated {
        return Err("workspace_repair_history_unconfirmed".into());
    }
    Ok(report)
}

/// Git répare seulement le lien du worktree vérifié ; aucune extraction,
/// aucun reset, aucun nettoyage. Le rejeu accepte le lien déjà réparé.
pub(crate) fn apply_repair_git(
    original: &WorkspaceRecord,
    anchor: &RepairAnchor,
) -> Result<WorkspaceRecord, String> {
    let backlink = bounded_file(&anchor.git_dir.join("gitdir"))?;
    if Path::new(backlink.trim_end()) == anchor.path.join(".git") {
        return report_repair(original, anchor, true);
    }
    report_repair(original, anchor, false)?;
    run_git(
        Path::new(&original.toplevel),
        &[
            "worktree",
            "repair",
            "--",
            anchor
                .path
                .to_str()
                .ok_or("workspace_repair_destination_invalid")?,
        ],
        READ_TIMEOUT,
    )?
    .complete()?;
    report_repair(original, anchor, true)
}
