//! Git natif154 : un exécuteur borné, aucune suppression ni commande réseau.
use crate::delegation::{WorkspaceCommit, WorkspaceRecord, WorkspaceState, WorkspaceStrategy};
use bridget_core::is_disallowed_control;
use std::fs;
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "workspace_git_tests.rs"]
mod tests;

pub(crate) const READ_TIMEOUT: Duration = Duration::from_secs(10);
const CREATE_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_OUTPUT: usize = 1024 * 1024;
const HOSTILE_CONFIG: &str = "^(filter\\..*\\.(smudge|clean|process)|include\\..*|includeif\\..*|extensions\\.(partialclone|worktreeconfig)|remote\\..*\\.promisor)$";

#[derive(Debug)]
pub(crate) struct GitOutput {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
}

impl GitOutput {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }

    fn complete(self) -> Result<String, String> {
        if self.truncated {
            return Err("sortie Git tronquée à 1 Mio".into());
        }
        if !self.success() {
            return Err(format!("Git {:?} : {}", self.code, self.stderr.trim()));
        }
        Ok(self.stdout.trim_end().to_string())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WorkspacePreflight {
    pub toplevel: PathBuf,
    pub cwd: PathBuf,
    pub offset: PathBuf,
    pub worktree_dir: PathBuf,
    pub branch: Option<String>,
    pub base_commit: String,
    pub listed_paths: Vec<PathBuf>,
}

/// O(L), L longueur du nom ; contrôle syntaxique sans accès disque.
pub(crate) fn validate_branch(branch: &str) -> Result<(), String> {
    if branch.is_empty()
        || branch.len() > 100
        || !branch.as_bytes()[0].is_ascii_alphanumeric()
        || !branch
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._/-".contains(&c))
        || branch.contains("..")
        || branch.contains("//")
        || branch.contains("@{")
        || branch.ends_with('/')
        || branch.ends_with('.')
        || branch
            .split('/')
            .any(|component| component.starts_with('.') || component.ends_with(".lock"))
    {
        return Err("workspace_branch_invalid: nom de branche invalide".into());
    }
    Ok(())
}

/// O(L), L longueur de la référence ; aucune option ne traverse cette garde.
pub(crate) fn validate_base_ref(base_ref: &str) -> Result<(), String> {
    if base_ref.is_empty()
        || base_ref.len() > 200
        || base_ref.starts_with('-')
        || base_ref
            .chars()
            .any(|c| c.is_whitespace() || is_disallowed_control(c))
    {
        return Err("workspace_base_unavailable: référence de base invalide".into());
    }
    Ok(())
}

fn sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|c| c.is_ascii_hexdigit())
}

fn revision(value: &str) -> bool {
    validate_base_ref(value.strip_suffix("^{commit}").unwrap_or(value)).is_ok()
}

fn range(value: &str) -> bool {
    value
        .split_once("..")
        .is_some_and(|(base, head)| sha(base) && sha(head))
}

// Formes fermées, pas seulement noms de commandes : ni --output ni --config,
// ni -C/-c/--git-dir fourni par l'appelant ne peuvent atteindre Git.
fn allowed(args: &[&str]) -> bool {
    match args {
        ["--version"] => true,
        ["rev-parse", "--show-toplevel" | "--git-common-dir"] => true,
        ["rev-parse", "--short", "HEAD"] => true,
        ["rev-parse", "--verify", "--end-of-options", value] => revision(value),
        ["symbolic-ref", "-q" | "--quiet" | "--short", "HEAD"] => true,
        ["check-ref-format", "--branch", branch] => validate_branch(branch).is_ok(),
        ["check-ignore", "-q", "--", path] => !path.starts_with('-') && !path.contains('\0'),
        ["show-ref", "--verify", "--quiet", branch] => branch
            .strip_prefix("refs/heads/")
            .is_some_and(|branch| validate_branch(branch).is_ok()),
        [
            "config",
            "--local",
            "--no-includes",
            "--get-regexp",
            pattern,
        ] => *pattern == HOSTILE_CONFIG,
        [
            "config",
            "--local",
            "--no-includes",
            "--get",
            "remote.origin.url",
        ] => true,
        ["worktree", "list", "--porcelain"] | ["worktree", "list", "--porcelain", "-z"] => true,
        ["worktree", "add", "-b", branch, "--", path, base] => {
            validate_branch(branch).is_ok() && Path::new(path).is_absolute() && sha(base)
        }
        ["rev-list", "--count", "--end-of-options", commits] => range(commits),
        [
            "log",
            "--max-count=50",
            "--format=%H%x1f%s",
            "--end-of-options",
            commits,
        ] => range(commits),
        ["status", "--porcelain=v1", "--untracked-files=normal"] => true,
        _ => false,
    }
}

/// O(B), B sortie drainée ; mémoire retenue <= 1 Mio, attente bornée.
pub(crate) fn run_git(cwd: &Path, args: &[&str], timeout: Duration) -> Result<GitOutput, String> {
    capture(cwd, args, timeout, None)
}

// Trois sorties d'erreur partagent cette récolte bornée. SIGTERM concerne
// uniquement le Child attesté par spawn ; aucun groupe, aucun SIGKILL.
fn stop_git(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Ok(());
    }
    if unsafe { libc::kill(child.id() as i32, libc::SIGTERM) } != 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error.to_string());
        }
    }
    let deadline = Instant::now() + Duration::from_millis(250);
    while Instant::now() < deadline {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("enfant Git non arrêté après SIGTERM".into())
}

// UNIQUE point de lancement. L'ancre privée sert au rapport : Git ignore alors
// le fichier .git modifiable de l'enfant et utilise les métadonnées enregistrées.
fn capture(
    cwd: &Path,
    args: &[&str],
    timeout: Duration,
    anchor: Option<(&Path, &Path)>,
) -> Result<GitOutput, String> {
    let timeout = timeout.min(if args.get(1) == Some(&"add") {
        CREATE_TIMEOUT
    } else {
        READ_TIMEOUT
    });
    let deadline = Instant::now() + timeout;
    if !allowed(args) {
        return Err("commande Git hors liste blanche".into());
    }
    // Chaque entrée réelle recontrôle le Git qui sera lancé. Pas de cache
    // global : le PATH d'un autre appel peut choisir un autre exécutable.
    if args != ["--version"] {
        capture(
            cwd,
            &["--version"],
            deadline.saturating_duration_since(Instant::now()),
            anchor,
        )?
        .complete()?;
    }
    if Instant::now() >= deadline {
        return Err("délai Git dépassé avant lancement".into());
    }
    let mut command = Command::new("git");
    for (name, _) in
        std::env::vars_os().filter(|(name, _)| name.to_string_lossy().starts_with("GIT_"))
    {
        command.env_remove(name);
    }
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_ALLOW_PROTOCOL", "")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args([
            "--no-pager",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "protocol.allow=never",
            "-c",
            "log.showSignature=false",
            "-c",
            "diff.external=",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
        ])
        .arg("-C")
        .arg(cwd);
    if let Some((git_dir, work_tree)) = anchor {
        command.arg(format!("--git-dir={}", git_dir.display()));
        command.arg(format!("--work-tree={}", work_tree.display()));
    }
    if args.first() == Some(&"log") {
        command
            .arg("log")
            .args(["--no-show-signature", "--no-textconv", "--no-ext-diff"])
            .args(&args[1..]);
    } else {
        command.args(args);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("workspace_git_unsupported: Git indisponible : {error}"))?;
    let mut stdout = child.stdout.take().ok_or("stdout Git indisponible")?;
    let mut stderr = child.stderr.take().ok_or("stderr Git indisponible")?;
    for fd in [stdout.as_raw_fd(), stderr.as_raw_fd()] {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            let cause = io::Error::last_os_error();
            let stopped = stop_git(&mut child);
            return Err(format!(
                "canal Git non bloquant indisponible : {cause}; arrêt {stopped:?}"
            ));
        }
    }
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut truncated = false;
    let mut retained = 0_usize;
    let mut eof = [false; 2];
    let mut status = None;
    loop {
        for (index, reader, bytes) in [
            (0, &mut stdout as &mut dyn Read, &mut out),
            (1, &mut stderr as &mut dyn Read, &mut err),
        ] {
            if eof[index] {
                continue;
            }
            let mut buffer = [0_u8; 8192];
            match reader.read(&mut buffer) {
                Ok(0) => eof[index] = true,
                Ok(count) => {
                    // La somme est recalculée avant chaque lecture, avec un seul
                    // budget pour stdout ET stderr. Le surplus est drainé.
                    let remaining = MAX_OUTPUT.saturating_sub(retained);
                    bytes.extend_from_slice(&buffer[..count.min(remaining)]);
                    retained += count.min(remaining);
                    truncated |= count > remaining;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    let stopped = stop_git(&mut child);
                    return Err(format!(
                        "lecture Git impossible : {error}; arrêt {stopped:?}"
                    ));
                }
            }
        }
        if status.is_none() {
            status = child.try_wait().map_err(|error| error.to_string())?;
        }
        if status.is_some() && eof.iter().all(|eof| *eof) {
            break;
        }
        if Instant::now() >= deadline {
            let stopped = stop_git(&mut child);
            return Err(format!("délai Git dépassé; arrêt {stopped:?}"));
        }
        // Pas de thread lecteur à rejoindre : un descendant gardant un pipe
        // ouvert ne peut prolonger le délai ni épuiser la mémoire.
        std::thread::sleep(Duration::from_millis(1));
    }
    let output = GitOutput {
        code: status.and_then(|status| status.code()),
        stdout: String::from_utf8(out).map_err(|_| "sortie Git non UTF-8")?,
        stderr: String::from_utf8(err).map_err(|_| "erreur Git non UTF-8")?,
        truncated,
    };
    if args == ["--version"] {
        let version = output
            .stdout
            .strip_prefix("git version ")
            .and_then(|version| version.split_whitespace().next())
            .and_then(|version| {
                let mut parts = version.split('.');
                Some((
                    parts.next()?.parse::<u32>().ok()?,
                    parts.next()?.parse::<u32>().ok()?,
                ))
            });
        if !output.success()
            || output.truncated
            || !version.is_some_and(|version| version >= (2, 44))
        {
            return Err("workspace_git_unsupported: Git 2.44 ou plus est requis".into());
        }
    }
    Ok(output)
}

/// O(B), B configuration locale bornée ; aucune inclusion n'est développée.
pub(crate) fn executable_configuration(cwd: &Path) -> Result<bool, String> {
    let output = run_git(
        cwd,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get-regexp",
            HOSTILE_CONFIG,
        ],
        READ_TIMEOUT,
    )?;
    if output.truncated {
        return Err("configuration Git tronquée".into());
    }
    match output.code {
        Some(0) => Ok(!output.stdout.is_empty()),
        Some(1) => Ok(false),
        _ => Err(format!(
            "configuration Git indisponible : {}",
            output.stderr.trim()
        )),
    }
}

/// O(B+w), B configuration/sorties bornées, w worktrees listés. Aucune écriture.
pub(crate) fn preflight(
    cwd: &Path,
    base_ref: Option<&str>,
    branch: Option<&str>,
) -> Result<WorkspacePreflight, String> {
    run_git(cwd, &["--version"], READ_TIMEOUT)?.complete()?;
    let cwd = cwd
        .canonicalize()
        .map_err(|error| format!("workspace_not_git: {error}"))?;
    let top = run_git(&cwd, &["rev-parse", "--show-toplevel"], READ_TIMEOUT)
        .and_then(GitOutput::complete)
        .map_err(|error| format!("workspace_not_git: {error}"))?;
    let toplevel = PathBuf::from(top)
        .canonicalize()
        .map_err(|error| format!("workspace_not_git: {error}"))?;
    let offset = cwd
        .strip_prefix(&toplevel)
        .map_err(|_| "workspace_path_unsafe: dossier hors dépôt")?
        .to_path_buf();
    if executable_configuration(&cwd)
        .map_err(|error| format!("workspace_repo_unsupported: {error}"))?
    {
        return Err(
            "workspace_repo_unsupported: filtre, inclusion, clone partiel ou worktreeConfig".into(),
        );
    }
    let base = base_ref.unwrap_or("HEAD");
    validate_base_ref(base)?;
    let base = format!("{base}^{{commit}}");
    let base_commit = run_git(
        &cwd,
        &["rev-parse", "--verify", "--end-of-options", &base],
        READ_TIMEOUT,
    )
    .and_then(GitOutput::complete)
    .map_err(|error| format!("workspace_base_unavailable: {error}"))?;
    if !sha(&base_commit) {
        return Err("workspace_base_unavailable: commit de base non SHA-1".into());
    }
    if let Some(branch) = branch {
        validate_branch(branch)?;
        run_git(
            &cwd,
            &["check-ref-format", "--branch", branch],
            READ_TIMEOUT,
        )
        .and_then(GitOutput::complete)
        .map_err(|error| format!("workspace_branch_invalid: {error}"))?;
        let reference = format!("refs/heads/{branch}");
        let exists = run_git(
            &cwd,
            &["show-ref", "--verify", "--quiet", &reference],
            READ_TIMEOUT,
        )
        .map_err(|error| {
            format!("workspace_branch_invalid: lecture de branche indisponible : {error}")
        })?;
        if exists.success() {
            return Err("workspace_branch_exists: la branche existe déjà".into());
        }
        if exists.code != Some(1) || exists.truncated {
            return Err("workspace_branch_invalid: lecture de branche impossible".into());
        }
    }
    let worktree_dir = cwd.join(".worktrees");
    match fs::symlink_metadata(&worktree_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err("workspace_path_unsafe: .worktrees doit être un vrai dossier".into());
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("workspace_path_unsafe: {error}")),
    }
    let ignored = run_git(
        &cwd,
        &["check-ignore", "-q", "--", ".worktrees/x"],
        READ_TIMEOUT,
    )
    .map_err(|error| {
        format!("workspace_dir_not_ignored: lecture de l'exclusion indisponible : {error}")
    })?;
    if !ignored.success() || ignored.truncated {
        return Err("workspace_dir_not_ignored: .worktrees/ doit être ignoré".into());
    }
    let list = run_git(
        &cwd,
        &["worktree", "list", "--porcelain", "-z"],
        READ_TIMEOUT,
    )
    .and_then(GitOutput::complete)
    .map_err(|error| {
        format!(
            "workspace_create_failed: liste des worktrees indisponible avant création : {error}"
        )
    })?;
    let listed_paths = list
        .split('\0')
        .filter_map(|field| field.strip_prefix("worktree "))
        .map(PathBuf::from)
        .filter(|path| path.starts_with(&worktree_dir))
        .collect::<Vec<_>>();
    if listed_paths.len() >= 32 {
        return Err("workspace_limit: 32 worktrees déjà enregistrés".into());
    }
    Ok(WorkspacePreflight {
        toplevel,
        cwd,
        offset,
        worktree_dir,
        branch: branch.map(str::to_string),
        base_commit,
        listed_paths,
    })
}

/// O(f), f fichiers extraits par Git. Aucun retrait, y compris sur échec tardif.
pub(crate) fn create(
    preflight: &WorkspacePreflight,
    task_id: &str,
) -> Result<WorkspaceRecord, String> {
    let task =
        uuid::Uuid::parse_str(task_id).map_err(|_| "workspace_path_unsafe: task_id invalide")?;
    let task8 = &task.simple().to_string()[..8];
    let path = preflight.worktree_dir.join(task8);
    let branch = preflight
        .branch
        .clone()
        .unwrap_or_else(|| format!("bridget/{task8}"));
    let residue = |code: &str, error: String| {
        format!(
            "{code}: {error}; chemin {}; branche {branch}",
            path.display()
        )
    };
    if preflight.cwd.canonicalize().ok().as_ref() != Some(&preflight.cwd)
        || preflight.toplevel.canonicalize().ok().as_ref() != Some(&preflight.toplevel)
        || preflight.worktree_dir != preflight.cwd.join(".worktrees")
    {
        return Err(residue(
            "workspace_path_unsafe",
            "racine canonique modifiée".into(),
        ));
    }
    match fs::symlink_metadata(&preflight.worktree_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(residue(
                "workspace_path_unsafe",
                ".worktrees remplacé".into(),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(residue("workspace_path_unsafe", error.to_string())),
    }
    let path_text = path
        .to_str()
        .ok_or_else(|| residue("workspace_path_unsafe", "chemin non UTF-8".into()))?;
    run_git(
        &preflight.cwd,
        &[
            "worktree",
            "add",
            "-b",
            &branch,
            "--",
            path_text,
            &preflight.base_commit,
        ],
        CREATE_TIMEOUT,
    )
    .and_then(GitOutput::complete)
    .map_err(|error| residue("workspace_create_failed", error))?;
    let canonical = path
        .canonicalize()
        .map_err(|error| residue("workspace_path_unsafe", error.to_string()))?;
    let child_path = path.join(&preflight.offset);
    if canonical != path
        || !canonical.starts_with(&preflight.cwd)
        || child_path.canonicalize().ok().as_ref() != Some(&child_path)
    {
        return Err(residue(
            "workspace_path_unsafe",
            "worktree ou décalage non canonique".into(),
        ));
    }
    Ok(WorkspaceRecord {
        strategy: WorkspaceStrategy::Worktree,
        state: WorkspaceState::Active,
        toplevel: preflight.toplevel.to_string_lossy().into_owned(),
        path: path_text.into(),
        branch,
        base_commit: preflight.base_commit.clone(),
        head_commit: None,
        branch_commit: None,
        head_on_branch: None,
        commit_count: None,
        commits: Vec::new(),
        truncated: false,
        dirty: None,
    })
}

// O(a+w), a ancêtres du parent, w entrées du registre (sortie/fichiers bornés).
// Le parent est la source de confiance. On ne lit JAMAIS path/.git enfant.
fn report_anchor(workspace: &WorkspaceRecord) -> Result<(PathBuf, PathBuf), String> {
    use std::os::unix::fs::OpenOptionsExt;
    let path = PathBuf::from(&workspace.path);
    if path.canonicalize().ok().as_ref() != Some(&path) {
        return Err("worktree absent ou chemin remplacé".into());
    }
    let parent = PathBuf::from(&workspace.toplevel);
    if path == parent || !path.starts_with(&parent) {
        return Err("ancre du parent hors de la structure attendue".into());
    }
    let mut candidates = parent
        .ancestors()
        .filter_map(|ancestor| {
            let candidate = ancestor.join(".git");
            fs::symlink_metadata(&candidate)
                .ok()
                .filter(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
                .and_then(|_| candidate.canonicalize().ok())
        })
        .collect::<Vec<_>>();
    let parent_common = (|| {
        // Un principal déjà worktree lié peut avoir sa metadata hors de tous
        // ses ancêtres. Le parent reste externe au worktree que l'on rapporte.
        let text = run_git(&parent, &["rev-parse", "--git-common-dir"], READ_TIMEOUT)
            .and_then(GitOutput::complete)
            .ok()?;
        let candidate = PathBuf::from(text);
        let candidate = if candidate.is_absolute() {
            candidate
        } else {
            parent.join(candidate)
        };
        let metadata = fs::symlink_metadata(&candidate).ok()?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return None;
        }
        candidate.canonicalize().ok()
    })();
    if let Some(common) = parent_common {
        if !candidates.contains(&common) {
            candidates.push(common);
        }
    }
    let expected_gitfile = path.join(".git");
    for common in candidates {
        let registry = common.join("worktrees");
        let Ok(registry_metadata) = fs::symlink_metadata(&registry) else {
            continue;
        };
        if !registry_metadata.is_dir() || registry_metadata.file_type().is_symlink() {
            continue;
        }
        for (index, entry) in fs::read_dir(&registry)
            .map_err(|error| error.to_string())?
            .enumerate()
        {
            if index >= 4096 {
                return Err("registre Git trop grand".into());
            }
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(metadata) = entry.file_type() else {
                continue;
            };
            if !metadata.is_dir() || metadata.is_symlink() {
                continue;
            }
            let directory = entry.path();
            if directory.canonicalize().ok().as_ref() != Some(&directory) {
                continue;
            }
            let gitdir = directory.join("gitdir");
            // Un voisin peut être encore en création ou illisible. Il ne prouve
            // rien sur la cible ; celle-ci doit toujours avoir sa propre ancre.
            let Ok(file) = fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(&gitdir)
            else {
                continue;
            };
            let Ok(metadata) = file.metadata() else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            let mut bytes = Vec::new();
            if file.take(8193).read_to_end(&mut bytes).is_err() {
                continue;
            }
            if bytes.len() > 8192 {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            if Path::new(text.trim_end()) == expected_gitfile {
                return Ok((directory, path));
            }
        }
    }
    Err("worktree absent du registre Git du parent".into())
}

/// O(B+50), sorties bornées et 50 commits ; faits inconnus renvoyés en erreur.
pub(crate) fn report(workspace: &WorkspaceRecord) -> Result<WorkspaceRecord, String> {
    let (git_dir, path) = report_anchor(workspace)?;
    let read = |args: &[&str]| capture(&path, args, READ_TIMEOUT, Some((&git_dir, &path)));
    let configuration = read(&[
        "config",
        "--local",
        "--no-includes",
        "--get-regexp",
        HOSTILE_CONFIG,
    ])?;
    if configuration.truncated || configuration.code != Some(1) {
        return Err("rapport indisponible (configuration exécutable ou illisible)".into());
    }
    let head = read(&["rev-parse", "--verify", "--end-of-options", "HEAD"])?.complete()?;
    if !sha(&head) {
        return Err("tête Git non SHA-1".into());
    }
    let symbolic = read(&["symbolic-ref", "-q", "HEAD"])?;
    if symbolic.truncated || !matches!(symbolic.code, Some(0 | 1)) {
        return Err("branche courante illisible".into());
    }
    let reference = format!("refs/heads/{}", workspace.branch);
    let branch = read(&["rev-parse", "--verify", "--end-of-options", &reference])?.complete()?;
    if !sha(&branch) || !sha(&workspace.base_commit) {
        return Err("branche ou base Git non SHA-1".into());
    }
    let range = format!("{}..{head}", workspace.base_commit);
    let count = read(&["rev-list", "--count", "--end-of-options", &range])?
        .complete()?
        .parse::<u32>()
        .map_err(|_| "nombre de commits illisible")?;
    let log = read(&[
        "log",
        "--max-count=50",
        "--format=%H%x1f%s",
        "--end-of-options",
        &range,
    ])?
    .complete()?;
    let mut commits = Vec::new();
    for line in log.lines().take(50) {
        let (commit, subject) = line.split_once('\u{1f}').ok_or("commit illisible")?;
        if !sha(commit) {
            return Err("commit Git non SHA-1".into());
        }
        commits.push(WorkspaceCommit {
            sha: commit.into(),
            subject: subject
                .chars()
                .filter(|c| !is_disallowed_control(*c))
                .take(200)
                .collect(),
        });
    }
    let status = read(&["status", "--porcelain=v1", "--untracked-files=normal"])?;
    if !status.success() {
        return Err(format!("état Git illisible : {}", status.stderr));
    }
    if status.truncated && status.stdout.is_empty() {
        return Err("état Git tronqué sans fait utilisable".into());
    }
    let mut result = workspace.clone();
    result.state = WorkspaceState::Kept;
    result.head_commit = Some(head);
    result.branch_commit = Some(branch);
    result.head_on_branch = Some(symbolic.success() && symbolic.stdout.trim_end() == reference);
    result.commit_count = Some(count);
    result.commits = commits;
    result.truncated = count > 50;
    result.dirty = Some(!status.stdout.is_empty());
    Ok(result)
}
