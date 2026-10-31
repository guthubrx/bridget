//! Profil administratif inerte. Aucun script du dépôt n'est exécuté.
//! La sélection est propre à la nouvelle ancre Git, pas au dépôt partagé.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
const MANIFEST: &str = "bridget-compact207.json";
const MAX_MANIFEST: u64 = 8 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plan {
    version: u8,
    base: String,
    selected: Vec<String>,
    patterns_sha256: String,
    selected_bytes: u64,
    total_bytes: u64,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn source(path: &str) -> bool {
    matches!(
        Path::new(path).extension().and_then(|e| e.to_str()),
        Some(
            "js" | "mjs"
                | "cjs"
                | "ts"
                | "tsx"
                | "jsx"
                | "py"
                | "rs"
                | "sh"
                | "zsh"
                | "html"
                | "css"
                | "scss"
                | "md"
                | "yml"
                | "yaml"
                | "toml"
                | "lock"
        )
    )
}
// Choix partagé avec la vue exportée ; aucune configuration Git n'est changée.
pub(super) fn selected(path: &str, bytes: u64, symlink: bool) -> bool {
    bytes < 131072 || source(path) || symlink
}
fn patterns(paths: &[String]) -> Result<String, String> {
    let mut text = String::new();
    for path in paths {
        if path.is_empty()
            || path
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
            || path.chars().any(|c| c.is_control())
        {
            return Err(
                "workspace_repo_unsupported: nom Git non représentable dans le profil compact"
                    .into(),
            );
        }
        text.push('/');
        for c in path.chars() {
            if "\\*?[]!# ".contains(c) {
                text.push('\\');
            }
            text.push(c);
        }
        text.push('\n');
    }
    if text.len() as u64 > MAX_MANIFEST {
        return Err("workspace_repo_unsupported: sélection compacte trop grande".into());
    }
    Ok(text)
}
pub(super) fn plan(preflight: &WorkspacePreflight) -> Result<Option<Plan>, String> {
    let config = run_git(
        &preflight.cwd,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get",
            "bridget.worktreeProfile",
        ],
        READ_TIMEOUT,
    )?;
    if config.code == Some(1) && !config.truncated {
        return Ok(None);
    }
    let profile = config.complete()?;
    if profile != "compact-v1" {
        return Err("workspace_repo_unsupported: bridget.worktreeProfile inconnu ; corriger le réglage administratif du dépôt avant une nouvelle admission".into());
    }
    let enabled = run_git(
        &preflight.cwd,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get",
            "extensions.worktreeConfig",
        ],
        READ_TIMEOUT,
    )?
    .complete()?;
    if enabled != "true" {
        return Err("workspace_repo_unsupported: compact-v1 exige extensions.worktreeConfig=true ; faire qualifier ce réglage par le responsable du dépôt".into());
    }
    let listing_output = run_git(
        &preflight.cwd,
        &[
            "ls-tree",
            "--full-tree",
            "-r",
            "-l",
            "-z",
            &preflight.base_commit,
        ],
        READ_TIMEOUT,
    )?;
    if listing_output.truncated {
        return Err("workspace_repo_unsupported: sortie Git tronquée pour compact-v1 (arbre limité à 8 Mio, diagnostic à 1 Mio). Cette préparation est refusée. Faire qualifier la taille du dépôt par son responsable, ou retirer son opt-in bridget.worktreeProfile avant une future admission ; conserver toute tâche déjà admise et sa clé.".into());
    }
    let listing = listing_output.complete()?;
    let mut selected = Vec::new();
    let mut total_bytes = 0_u64;
    let mut selected_bytes = 0_u64;
    for row in listing.split('\0').filter(|row| !row.is_empty()) {
        let (meta, path) = row
            .split_once('\t')
            .ok_or("workspace_repo_unsupported: arbre Git illisible")?;
        let fields: Vec<_> = meta.split_whitespace().collect();
        if fields.len() != 4
            || fields[1] != "blob"
            || !matches!(fields[0], "100644" | "100755" | "120000")
        {
            return Err(
                "workspace_repo_unsupported: compact-v1 exige un arbre de fichiers Git ordinaires"
                    .into(),
            );
        }
        let bytes = fields[3]
            .parse::<u64>()
            .map_err(|_| "workspace_repo_unsupported: taille Git illisible")?;
        total_bytes = total_bytes
            .checked_add(bytes)
            .ok_or("workspace_repo_unsupported: tailles Git excessives")?;
        if self::selected(path, bytes, fields[0] == "120000") {
            selected_bytes = selected_bytes
                .checked_add(bytes)
                .ok_or("workspace_repo_unsupported: tailles Git excessives")?;
            selected.push(path.to_string());
        }
    }
    if selected.is_empty() {
        return Err("workspace_repo_unsupported: profil compact vide".into());
    }
    let text = patterns(&selected)?;
    Ok(Some(Plan {
        version: 1,
        base: preflight.base_commit.clone(),
        selected,
        patterns_sha256: hash(text.as_bytes()),
        selected_bytes,
        total_bytes,
    }))
}
fn read_private(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(io_failure)?;
    let meta = file.metadata().map_err(io_failure)?;
    if !meta.is_file() || meta.len() > MAX_MANIFEST {
        return Err("métadonnées compactes invalides".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_MANIFEST + 1)
        .read_to_end(&mut bytes)
        .map_err(io_failure)?;
    if bytes.len() as u64 > MAX_MANIFEST {
        return Err("métadonnées compactes excessives".into());
    }
    Ok(bytes)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(io_failure)?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(io_failure)
}
pub(super) fn apply(path: &Path, plan: Plan) -> Result<(), String> {
    let dir = PathBuf::from(
        run_git(path, &["rev-parse", "--absolute-git-dir"], READ_TIMEOUT)?.complete()?,
    );
    if dir.canonicalize().ok().as_ref() != Some(&dir) || !dir.is_dir() {
        return Err("ancre compacte non canonique".into());
    }
    // Le fichier précède l'extraction : une coupure garde une ancre reconnaissable.
    let data = serde_json::to_vec(&plan).map_err(|e| e.to_string())?;
    if data.len() as u64 > MAX_MANIFEST {
        return Err("manifeste compact excessif".into());
    }
    write_new(&dir.join(MANIFEST), &data)?;
    fs::create_dir_all(dir.join("info")).map_err(io_failure)?;
    write_new(
        &dir.join("info/sparse-checkout"),
        patterns(&plan.selected)?.as_bytes(),
    )?;
    for (key, value) in [
        ("core.sparseCheckout", "true"),
        ("core.sparseCheckoutCone", "false"),
        ("index.sparse", "false"),
        ("bridget.compactFormat", "207"),
    ] {
        run_git(path, &["config", "--worktree", key, value], READ_TIMEOUT)?.workspace_complete()?;
    }
    run_git(path, &["read-tree", "-mu", "HEAD"], WORKTREE_ADD_TIMEOUT)?.workspace_complete()?;
    audit(&dir, path)
}
pub(super) fn audit(dir: &Path, path: &Path) -> Result<(), String> {
    let read = |args: &[&str]| capture(path, args, READ_TIMEOUT, Some((dir, path)));
    let index_output = read(&["ls-files", "-v", "-z"])?;
    if index_output.truncated {
        return Err("rapport indisponible : sortie Git tronquée (index limité à 8 Mio, diagnostic à 1 Mio). Conserver le dossier ; faire qualifier sa taille par le responsable avant une nouvelle admission. Ce refus concerne ce rapport.".into());
    }
    let flags = index_output.complete()?;
    for entry in flags.split('\0').filter(|e| !e.is_empty()) {
        let (tag, name) = entry.split_once(' ').ok_or("index Git illisible")?;
        if tag.len() != 1
            || tag.as_bytes()[0].is_ascii_lowercase()
            || (tag == "S" && fs::symlink_metadata(path.join(name)).is_ok())
        {
            return Err("rapport indisponible : drapeau Git susceptible de masquer une modification ; vérifier les fichiers et rétablir les drapeaux avant une suite".into());
        }
    }
    let marker = read(&["config", "--no-includes", "--get", "bridget.compactFormat"])?;
    let expected = if marker.code == Some(1) && !marker.truncated {
        false
    } else {
        marker.complete()? == "207"
    };
    match fs::symlink_metadata(dir.join(MANIFEST)) {
        Err(e) if e.kind()==io::ErrorKind::NotFound && !expected => return Ok(()),
        Err(e) if e.kind()==io::ErrorKind::NotFound => return Err("rapport indisponible : manifeste compact absent ; faire vérifier le dossier par le responsable".into()),
        Err(e) => return Err(io_failure(e)),
        Ok(_) => {}
    }
    if !expected {
        return Err("rapport indisponible : marqueur compact altéré".into());
    }
    let plan: Plan = serde_json::from_slice(&read_private(&dir.join(MANIFEST))?)
        .map_err(|_| "manifeste compact invalide")?;
    if plan.version != 1
        || !sha(&plan.base)
        || plan.selected_bytes > plan.total_bytes
        || hash(patterns(&plan.selected)?.as_bytes()) != plan.patterns_sha256
        || hash(&read_private(&dir.join("info/sparse-checkout"))?) != plan.patterns_sha256
    {
        return Err("rapport indisponible : profil compact modifié ; vérifier le dossier et rétablir son profil avant une suite".into());
    }
    let read = |args: &[&str]| {
        capture(path, args, READ_TIMEOUT, Some((dir, path))).and_then(GitOutput::complete)
    };
    if read(&["config", "--worktree", "--get", "core.sparseCheckout"])? != "true"
        || read(&["config", "--worktree", "--get", "core.sparseCheckoutCone"])? != "false"
    {
        return Err("rapport indisponible : configuration compacte modifiée".into());
    }
    let selected: HashSet<_> = plan.selected.iter().map(String::as_str).collect();
    for entry in flags.split('\0').filter(|e| !e.is_empty()) {
        let (tag, name) = entry.split_once(' ').ok_or("index compact illisible")?;
        if tag.len() != 1
            || tag.as_bytes()[0].is_ascii_lowercase()
            || (tag == "S"
                && (selected.contains(name) || fs::symlink_metadata(path.join(name)).is_ok()))
        {
            return Err("rapport indisponible : drapeau Git susceptible de masquer une modification ; vérifier les fichiers et rétablir les drapeaux avant une suite".into());
        }
    }
    Ok(())
}

pub(super) fn is_compact(cwd: &Path) -> bool {
    run_git(cwd, &["rev-parse", "--absolute-git-dir"], READ_TIMEOUT)
        .and_then(GitOutput::complete)
        .is_ok_and(|dir| fs::symlink_metadata(Path::new(&dir).join(MANIFEST)).is_ok())
}

pub(super) fn io_failure(error: io::Error) -> String {
    let code = match error.raw_os_error() {
        Some(libc::ENOSPC | libc::EDQUOT) => "workspace_disk_full",
        Some(libc::EACCES | libc::EPERM | libc::EROFS) => "workspace_access_denied",
        _ => "workspace_create_failed",
    };
    format!("{code}: {error}")
}
pub(super) fn failure_code(error: &str) -> &'static str {
    match error.split(':').next() {
        Some("workspace_disk_full") => "workspace_disk_full",
        Some("workspace_access_denied") => "workspace_access_denied",
        _ => "workspace_create_failed",
    }
}
#[cfg(test)]
#[test]
fn spec207_compact_preserves_io_codes() {
    for (errno, code) in [
        (libc::ENOSPC, "workspace_disk_full"),
        (libc::EDQUOT, "workspace_disk_full"),
        (libc::EACCES, "workspace_access_denied"),
        (libc::EROFS, "workspace_access_denied"),
    ] {
        assert_eq!(
            failure_code(&io_failure(io::Error::from_raw_os_error(errno))),
            code
        );
    }
}
