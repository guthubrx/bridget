//! Vue exportée C19 : fichiers du commit exact, aucun dépôt ni index enfant.
//! Ce plan de données ne donne aucun droit ; admission et preuve privée restent
//! à la charge du parcours natif. Aucun appelant de production avant son gate.
use super::*;
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::process::{ChildStderr, ChildStdin, ChildStdout};

const NOTICE: &str = ".bridget-commit-view.json";
const MAX_ENTRIES: usize = 65_536;
const MAX_BYTES: u64 = 512 * 1024 * 1024;
const MAX_LINK: u64 = 4096;
const DEADLINE: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Entry {
    path: String,
    mode: String,
    oid: String,
    bytes: u64,
    selected: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    version: u8,
    base: String,
    cwd: String,
    entries: Vec<Entry>,
}
fn refusal(cause: &str) -> String {
    format!("workspace_commit_view_unavailable:cause={cause}")
}
fn path_valid(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && !path.chars().any(char::is_control)
        && path.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.eq_ignore_ascii_case(".git")
                && !p.eq_ignore_ascii_case(NOTICE)
        })
        && !path.starts_with('/')
}
fn blob_hash(size: u64) -> Context {
    let mut hash = Context::new(&SHA1_FOR_LEGACY_USE_ONLY);
    hash.update(format!("blob {size}\0").as_bytes());
    hash
}
fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Un processus, requêtes fermées de moins de 64 octets. Chaque attente d'E/S
/// draine stderr ; ni blob complet en mémoire ni thread lecteur bloquant.
struct Objects {
    child: Child,
    input: ChildStdin,
    output: ChildStdout,
    diagnostic: ChildStderr,
    deadline: Instant,
    diagnostic_bytes: usize,
}
impl Objects {
    fn open(source: &Path) -> Result<Self, String> {
        run_git(source, &["--version"], READ_TIMEOUT)?.complete()?;
        let mut child = spawn_git(
            source,
            &["cat-file", "--batch-command"],
            None,
            Stdio::piped(),
            true,
        )?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| refusal("git_pipe_unavailable"))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| refusal("git_pipe_unavailable"))?;
        let diagnostic = child
            .stderr
            .take()
            .ok_or_else(|| refusal("git_pipe_unavailable"))?;
        for fd in [
            input.as_raw_fd(),
            output.as_raw_fd(),
            diagnostic.as_raw_fd(),
        ] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
            {
                let stopped = stop_git(&mut child);
                return Err(format!(
                    "{}; arrêt {stopped:?}",
                    refusal("git_pipe_unavailable")
                ));
            }
        }
        Ok(Self {
            child,
            input,
            output,
            diagnostic,
            deadline: Instant::now() + DEADLINE,
            diagnostic_bytes: 0,
        })
    }
    fn ready(&mut self, fd: i32, events: i16) -> Result<(), String> {
        if Instant::now() >= self.deadline {
            return Err(refusal("git_timeout"));
        }
        let mut buffer = [0; 8192];
        loop {
            match self.diagnostic.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    self.diagnostic_bytes = self.diagnostic_bytes.saturating_add(n);
                    if self.diagnostic_bytes > MAX_OUTPUT {
                        return Err(refusal("git_diagnostic_limit"));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(refusal("git_read_failed")),
            }
            if Instant::now() >= self.deadline {
                return Err(refusal("git_timeout"));
            }
        }
        let mut fds = [
            libc::pollfd {
                fd,
                events,
                revents: 0,
            },
            libc::pollfd {
                fd: self.diagnostic.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        // Courte attente commune, toujours sous la même échéance absolue.
        if unsafe { libc::poll(fds.as_mut_ptr(), 2, 10) } < 0
            && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted
        {
            return Err(refusal("git_poll_failed"));
        }
        Ok(())
    }
    fn send(&mut self, command: &[u8]) -> Result<(), String> {
        let mut remaining = command;
        while !remaining.is_empty() {
            self.ready(self.input.as_raw_fd(), libc::POLLOUT)?;
            match self.input.write(remaining) {
                Ok(0) => return Err(refusal("git_write_failed")),
                Ok(n) => remaining = &remaining[n..],
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(refusal("git_write_failed")),
            }
        }
        Ok(())
    }
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), String> {
        let mut position = 0;
        while position < bytes.len() {
            self.ready(self.output.as_raw_fd(), libc::POLLIN)?;
            match self.output.read(&mut bytes[position..]) {
                Ok(0) => return Err(refusal("git_object_unavailable")),
                Ok(n) => position += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(refusal("git_read_failed")),
            }
        }
        Ok(())
    }
    fn line(&mut self) -> Result<String, String> {
        let mut line = Vec::new();
        while line.len() < 128 {
            let mut byte = [0];
            self.read(&mut byte)?;
            if byte[0] == b'\n' {
                return String::from_utf8(line).map_err(|_| refusal("git_object_header_invalid"));
            }
            line.push(byte[0]);
        }
        Err(refusal("git_object_header_invalid"))
    }
    fn object(&mut self, entry: &Entry, sink: &mut dyn Write) -> Result<(), String> {
        if !sha(&entry.oid) {
            return Err(refusal("git_object_invalid"));
        }
        self.send(format!("contents {}\n", entry.oid).as_bytes())?;
        let expected = format!("{} blob {}", entry.oid, entry.bytes);
        if self.line()? != expected {
            return Err(refusal("git_object_unavailable"));
        }
        let mut hash = blob_hash(entry.bytes);
        let mut remaining = entry.bytes;
        let mut buffer = [0; 65536];
        while remaining > 0 {
            let count = remaining.min(buffer.len() as u64) as usize;
            self.read(&mut buffer[..count])?;
            hash.update(&buffer[..count]);
            sink.write_all(&buffer[..count])
                .map_err(compact::io_failure)?;
            remaining -= count as u64;
        }
        let mut separator = [0];
        self.read(&mut separator)?;
        if separator != [b'\n'] || hexadecimal(hash.finish().as_ref()) != entry.oid {
            return Err(refusal("git_object_integrity_failed"));
        }
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        stop_git(&mut self.child).map_err(|_| refusal("git_stop_unconfirmed"))
    }
}
impl Drop for Objects {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub(super) fn plan(source: &Path, base: &str) -> Result<Manifest, String> {
    if !sha(base) {
        return Err(refusal("commit_required"));
    }
    let output = capture_with_objects(
        source,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            "--end-of-options",
            &format!("{base}^{{commit}}"),
        ],
        READ_TIMEOUT,
        None,
        true,
    )?;
    if !output.success() || output.truncated || output.stdout.trim() != base {
        return Err(refusal("commit_unavailable"));
    }
    let output = capture_with_objects(
        source,
        &["ls-tree", "--full-tree", "-r", "-l", "-z", base],
        READ_TIMEOUT,
        None,
        true,
    )?;
    if output.truncated {
        return Err(refusal("tree_limit"));
    }
    if !output.success() {
        return Err(refusal("tree_unavailable"));
    }
    let mut entries = Vec::new();
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for row in output.stdout.split('\0').filter(|r| !r.is_empty()) {
        if entries.len() >= MAX_ENTRIES {
            return Err(refusal("entry_limit"));
        }
        let (meta, path) = row
            .split_once('\t')
            .ok_or_else(|| refusal("tree_invalid"))?;
        if !path_valid(path) || path.split('/').count() > 64 || !paths.insert(path.to_owned()) {
            return Err(refusal("path_unsupported"));
        }
        let fields = meta.split_ascii_whitespace().collect::<Vec<_>>();
        if fields.len() != 4
            || fields[1] != "blob"
            || !matches!(fields[0], "100644" | "100755" | "120000")
            || !sha(fields[2])
        {
            return Err(refusal("object_type_unsupported"));
        }
        let bytes = fields[3]
            .parse::<u64>()
            .map_err(|_| refusal("tree_invalid"))?;
        let selected = compact::selected(path, bytes, fields[0] == "120000");
        if selected {
            total = total
                .checked_add(bytes)
                .ok_or_else(|| refusal("byte_limit"))?;
            if total > MAX_BYTES {
                return Err(refusal("byte_limit"));
            }
        }
        if fields[0] == "120000" && bytes > MAX_LINK {
            return Err(refusal("link_unsupported"));
        }
        entries.push(Entry {
            path: path.into(),
            mode: fields[0].into(),
            oid: fields[2].into(),
            bytes,
            selected,
        });
    }
    if entries.is_empty() {
        return Err(refusal("empty_tree"));
    }
    let canonical_source = source
        .canonicalize()
        .map_err(|_| refusal("source_unavailable"))?;
    let top = run_git(source, &["rev-parse", "--show-toplevel"], READ_TIMEOUT)?
        .complete()
        .map_err(|_| refusal("source_unavailable"))?;
    let top = PathBuf::from(top)
        .canonicalize()
        .map_err(|_| refusal("source_unavailable"))?;
    let offset = canonical_source
        .strip_prefix(top)
        .map_err(|_| refusal("source_unavailable"))?;
    let cwd = if offset.as_os_str().is_empty() {
        ".".into()
    } else {
        offset
            .to_str()
            .ok_or_else(|| refusal("path_unsupported"))?
            .into()
    };
    let manifest = Manifest {
        version: 1,
        base: base.into(),
        cwd,
        entries,
    };
    validate_manifest(&manifest)?;
    Ok(manifest)
}

// Normalisation lexicale, puis expansion de chaque lien intermédiaire. Pas de
// canonicalize : la cible interne peut être absente ou volontairement omise.
fn resolve_link(
    path: &str,
    target: &str,
    links: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut components = path.split('/').map(str::to_owned).collect::<Vec<_>>();
    components.pop();
    if target.is_empty() || target.starts_with('/') || target.chars().any(char::is_control) {
        return Err(refusal("link_outside_view"));
    }
    let mut pending =
        std::collections::VecDeque::from(target.split('/').map(str::to_string).collect::<Vec<_>>());
    let mut expanded = 0usize;
    while let Some(part) = pending.pop_front() {
        match part.as_str() {
            "" | "." => continue,
            ".." => {
                if components.pop().is_none() {
                    return Err(refusal("link_outside_view"));
                }
            }
            _ => {
                components.push(part.clone());
                let next = components.join("/");
                if let Some(value) = links.get(&next) {
                    expanded += 1;
                    if expanded > 64 {
                        return Err(refusal("link_cycle"));
                    }
                    components.pop();
                    if value.starts_with('/') {
                        return Err(refusal("link_outside_view"));
                    }
                    for p in value.split('/').rev() {
                        pending.push_front(p.to_owned());
                    }
                }
            }
        }
    }
    Ok(components.join("/"))
}

// Compter aussi les répertoires implicites. Un listing de feuilles compact
// peut sinon multiplier par64 le nombre de chemins alloués en mémoire.
fn manifest_paths(manifest: &Manifest) -> Result<(BTreeSet<String>, BTreeSet<String>), String> {
    let files = manifest
        .entries
        .iter()
        .map(|e| e.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut all = BTreeSet::from([String::new()]);
    let mut initial = all.clone();
    for entry in &manifest.entries {
        all.insert(entry.path.clone());
        if entry.selected {
            initial.insert(entry.path.clone());
        }
        let mut parent = Path::new(&entry.path).parent();
        while let Some(path) = parent.filter(|p| !p.as_os_str().is_empty()) {
            let text = path.to_str().ok_or_else(|| refusal("path_unsupported"))?;
            if files.contains(text) {
                return Err(refusal("manifest_invalid"));
            }
            all.insert(text.into());
            if entry.selected
                || manifest.cwd == text
                || manifest.cwd.starts_with(&format!("{text}/"))
            {
                initial.insert(text.into());
            }
            if all.len() > MAX_ENTRIES + 1 {
                return Err(refusal("entry_limit"));
            }
            parent = path.parent();
        }
        if all.len() > MAX_ENTRIES + 1 {
            return Err(refusal("entry_limit"));
        }
    }
    Ok((all, initial))
}
fn observed_links(
    root: &Directory,
    manifest: &Manifest,
    deadline: Instant,
) -> Result<BTreeMap<String, String>, String> {
    let mut links = BTreeMap::new();
    for entry in manifest.entries.iter().filter(|e| e.mode == "120000") {
        if Instant::now() >= deadline {
            return Err(refusal("view_link_timeout"));
        }
        let (parent, name) = root.parent(&entry.path, false, &mut BTreeSet::new())?;
        let bytes = parent.read_link(&name)?;
        let mut hash = blob_hash(entry.bytes);
        hash.update(&bytes);
        if bytes.len() as u64 != entry.bytes || hexadecimal(hash.finish().as_ref()) != entry.oid {
            return Err(refusal("view_integrity_failed"));
        }
        let target = String::from_utf8(bytes).map_err(|_| refusal("view_integrity_failed"))?;
        root.link_is_present(&entry.path, &target, deadline)?;
        links.insert(entry.path.clone(), target);
    }
    Ok(links)
}

fn notice(manifest: &Manifest, links: &BTreeMap<String, String>) -> Result<Vec<u8>, String> {
    let omitted = manifest
        .entries
        .iter()
        .filter(|e| !e.selected)
        .map(|e| serde_json::json!({"path":e.path,"bytes":e.bytes,"reason":"compact_selection"}))
        .collect::<Vec<_>>();
    let (all, initial) = manifest_paths(manifest)?;
    let mut link_notes = Vec::new();
    for (path, target) in links {
        let resolved = resolve_link(path, target, links)?;
        link_notes.push(serde_json::json!({"path":path,"target":target,
            "exact_case_target_in_commit":all.contains(&resolved),
            "exact_case_target_selected_at_export":initial.contains(&resolved)}));
    }
    let bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "version":1,"commit":manifest.base,"cwd":manifest.cwd,"partial_at_export":!omitted.is_empty(),
        "git_repository":false,"manifest":manifest,"omitted_at_export":omitted,"links":link_notes,
        "link_scope":"Les cibles sont décrites selon les chemins exacts du commit. Une cible non déclarée peut être absente ; la sécurité des alias du système de fichiers est contrôlée séparément.",
        "next_action":"Cette carte décrit la sélection initiale ; un fichier omis peut avoir été matérialisé depuis. Lire les fichiers présents. Pour un chemin omis, appeler bridget_read_commit_file avec path égal au chemin exact de cette carte, puis lire le chemin renvoyé. L'enfant utilise sa connexion courante sans task_id ; le parent indique sa propre task_id. Si l'outil manque, faire charger le client compatible ; ne pas remplacer la mission ou sa base. Aucun git show/log dans cette vue ; aucun fichier absent n'est une preuve d'absence du commit."
    })).map_err(|_|refusal("manifest_invalid"))?;
    if bytes.len() > 8 * MAX_OUTPUT {
        return Err(refusal("notice_limit"));
    }
    Ok(bytes)
}

// Ancrage sur des descripteurs ouverts : une substitution de dossier par un
// lien pendant l'export ne peut rediriger une écriture hors de la vue.
struct Directory(fs::File);
impl Directory {
    fn open(path: &Path) -> Result<Self, String> {
        fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .map(Self)
            .map_err(compact::io_failure)
    }
    fn name(name: &str) -> Result<std::ffi::CString, String> {
        if name.is_empty() || name.contains('/') || matches!(name, "." | "..") {
            return Err(refusal("path_unsupported"));
        }
        std::ffi::CString::new(name).map_err(|_| refusal("path_unsupported"))
    }

    fn names(&self) -> Result<Vec<String>, String> {
        // dup partage l'offset : chaque Directory n'est énuméré qu'une fois.
        let fd = unsafe { libc::dup(self.0.as_raw_fd()) };
        if fd < 0 {
            return Err(compact::io_failure(io::Error::last_os_error()));
        }
        let stream = unsafe { libc::fdopendir(fd) };
        if stream.is_null() {
            unsafe { libc::close(fd) };
            return Err(refusal("view_read_failed"));
        }
        struct Stream(*mut libc::DIR);
        impl Drop for Stream {
            fn drop(&mut self) {
                unsafe { libc::closedir(self.0) };
            }
        }
        let stream = Stream(stream);
        let mut names = Vec::new();
        loop {
            #[cfg(target_os = "linux")]
            let errno = unsafe { libc::__errno_location() };
            #[cfg(not(target_os = "linux"))]
            let errno = unsafe { libc::__error() };
            unsafe { *errno = 0 };
            let entry = unsafe { libc::readdir(stream.0) };
            if entry.is_null() {
                if unsafe { *errno } != 0 {
                    return Err(refusal("view_read_failed"));
                }
                break;
            }
            let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }
                .to_str()
                .map_err(|_| refusal("path_unsupported"))?;
            if matches!(name, "." | "..") {
                continue;
            }
            Self::name(name)?;
            if names.len() >= MAX_ENTRIES + 1 {
                return Err(refusal("view_entry_limit"));
            }
            names.push(name.to_owned());
        }
        Ok(names)
    }
    fn optional_kind(&self, name: &str) -> Result<Option<libc::mode_t>, String> {
        let name = Self::name(name)?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe {
            libc::fstatat(
                self.0.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(compact::io_failure(error));
        }
        Ok(Some(unsafe { stat.assume_init() }.st_mode & libc::S_IFMT))
    }
    fn kind(&self, name: &str) -> Result<libc::mode_t, String> {
        self.optional_kind(name)?
            .ok_or_else(|| refusal("view_integrity_failed"))
    }
    fn read_link(&self, name: &str) -> Result<Vec<u8>, String> {
        let name = Self::name(name)?;
        let mut bytes = vec![0; MAX_LINK as usize + 1];
        let n = unsafe {
            libc::readlinkat(
                self.0.as_raw_fd(),
                name.as_ptr(),
                bytes.as_mut_ptr().cast(),
                bytes.len(),
            )
        };
        if n < 0 {
            return Err(compact::io_failure(io::Error::last_os_error()));
        }
        if n as u64 > MAX_LINK {
            return Err(refusal("link_unsupported"));
        }
        bytes.truncate(n as usize);
        Ok(bytes)
    }

    /// Résolution contrôlée par descripteurs. Le FS peut reconnaître un alias
    /// de casse/Unicode absent de la carte Git : lire chaque lien réel sans
    /// laisser le noyau le suivre, et ne jamais remonter au-dessus de la vue.
    fn link_is_present(&self, path: &str, target: &str, deadline: Instant) -> Result<bool, String> {
        if target.is_empty() || target.starts_with('/') || target.chars().any(char::is_control) {
            return Err(refusal("link_outside_view"));
        }
        let mut stack = vec![Self(self.0.try_clone().map_err(compact::io_failure)?)];
        let mut prefix = path.split('/').collect::<Vec<_>>();
        prefix.pop();
        for part in prefix {
            if Instant::now() >= deadline {
                return Err(refusal("view_link_timeout"));
            }
            let next = stack.last().unwrap().child(part)?;
            stack.push(next);
        }
        let mut pending = target
            .split('/')
            .map(str::to_owned)
            .collect::<std::collections::VecDeque<_>>();
        let mut expanded = 0usize;
        while let Some(part) = pending.pop_front() {
            if Instant::now() >= deadline {
                return Err(refusal("view_link_timeout"));
            }
            if part.is_empty() || part == "." {
                continue;
            }
            if part == ".." {
                if stack.len() == 1 {
                    return Err(refusal("link_outside_view"));
                }
                stack.pop();
                continue;
            }
            let current = stack.last().unwrap();
            match current.optional_kind(&part)? {
                None => return Ok(false),
                Some(libc::S_IFLNK) => {
                    expanded += 1;
                    if expanded > 64 {
                        return Err(refusal("link_cycle"));
                    }
                    let bytes = current.read_link(&part)?;
                    let value =
                        std::str::from_utf8(&bytes).map_err(|_| refusal("link_unsupported"))?;
                    if value.is_empty()
                        || value.starts_with('/')
                        || value.chars().any(char::is_control)
                    {
                        return Err(refusal("link_outside_view"));
                    }
                    for component in value.split('/').rev() {
                        pending.push_front(component.into());
                    }
                }
                Some(libc::S_IFDIR) => {
                    if stack.len() >= 65 {
                        return Err(refusal("link_depth_limit"));
                    }
                    stack.push(current.child(&part)?);
                }
                Some(libc::S_IFREG) => return Ok(pending.is_empty()),
                Some(_) => return Err(refusal("view_integrity_failed")),
            }
        }
        Ok(true)
    }
    fn ensure_parent(&self, path: &str) -> Result<(Self, String), String> {
        let mut current = Self(self.0.try_clone().map_err(compact::io_failure)?);
        let mut parts = path.split('/').peekable();
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                return Ok((current, part.to_owned()));
            }
            current = match current.optional_kind(part)? {
                None => {
                    let next = current.create(part)?;
                    current.0.sync_all().map_err(compact::io_failure)?;
                    next
                }
                Some(libc::S_IFDIR) => current.child(part)?,
                Some(_) => return Err(refusal("view_integrity_failed")),
            };
        }
        Err(refusal("path_unsupported"))
    }
    fn child(&self, name: &str) -> Result<Self, String> {
        use std::os::fd::FromRawFd;
        let name = Self::name(name)?;
        let fd = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(compact::io_failure(io::Error::last_os_error()));
        }
        Ok(Self(unsafe { fs::File::from_raw_fd(fd) }))
    }
    fn create(&self, name: &str) -> Result<Self, String> {
        let encoded = Self::name(name)?;
        if unsafe { libc::mkdirat(self.0.as_raw_fd(), encoded.as_ptr(), 0o700) } != 0 {
            return Err(compact::io_failure(io::Error::last_os_error()));
        }
        self.child(name)
    }
    fn parent(
        &self,
        path: &str,
        create: bool,
        known: &mut BTreeSet<String>,
    ) -> Result<(Self, String), String> {
        let mut current = Self(self.0.try_clone().map_err(compact::io_failure)?);
        let mut parts = path.split('/').peekable();
        let mut prefix = String::new();
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                return Ok((current, part.to_owned()));
            }
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            current = if create && known.insert(prefix.clone()) {
                current.create(part)?
            } else {
                current.child(part)?
            };
        }
        Err(refusal("path_unsupported"))
    }
    fn file(&self, name: &str, create: bool) -> Result<fs::File, String> {
        use std::os::fd::FromRawFd;
        let name = Self::name(name)?;
        let flags = if create {
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL
        } else {
            libc::O_RDONLY
        };
        let fd = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                name.as_ptr(),
                flags | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(compact::io_failure(io::Error::last_os_error()));
        }
        Ok(unsafe { fs::File::from_raw_fd(fd) })
    }
    fn link(&self, name: &str, target: &str) -> Result<(), String> {
        let name = Self::name(name)?;
        let target = std::ffi::CString::new(target).map_err(|_| refusal("link_unsupported"))?;
        if unsafe { libc::symlinkat(target.as_ptr(), self.0.as_raw_fd(), name.as_ptr()) } != 0 {
            return Err(compact::io_failure(io::Error::last_os_error()));
        }
        Ok(())
    }
}

// Chaque fichier et chaque dossier est d'abord poussé vers le périphérique.
// Une seule barrière forte finale sur la notice, dans le même volume, remplace
// les milliers de F_FULLFSYNC de File::sync_all sur macOS. Aucun sync global.
fn sync_to_device(file: &fs::File, deadline: Instant) -> Result<(), String> {
    loop {
        if Instant::now() >= deadline {
            return Err(refusal("view_sync_timeout"));
        }
        if unsafe { libc::fsync(file.as_raw_fd()) } == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(compact::io_failure(error));
        }
    }
}

/// destination exclusivement neuve. Un échec conserve sa matière partielle pour
/// le reçu natif ; il ne la publie pas et ne supprime rien pour tenter à nouveau.
pub(super) fn export(source: &Path, manifest: &Manifest, destination: &Path) -> Result<(), String> {
    validate_manifest(manifest)?;
    let source = source
        .canonicalize()
        .map_err(|_| refusal("source_unavailable"))?;
    let parent = destination
        .parent()
        .ok_or_else(|| refusal("destination_invalid"))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| refusal("destination_invalid"))?;
    if parent != canonical_parent || destination.file_name().is_none() {
        return Err(refusal("destination_invalid"));
    }
    // Un cwd peut être un sous-dossier ou un linked checkout. Protéger la
    // racine entière ET ses métadonnées externes, sans second canonicalize.
    let mut forbidden = Vec::new();
    for argument in ["--show-toplevel", "--absolute-git-dir", "--git-common-dir"] {
        let value = run_git(&source, &["rev-parse", argument], READ_TIMEOUT)
            .and_then(GitOutput::complete)
            .map_err(|_| refusal("source_unavailable"))?;
        let path = PathBuf::from(value);
        let path = if path.is_absolute() {
            path
        } else {
            source.join(path)
        };
        forbidden.push(
            path.canonicalize()
                .map_err(|_| refusal("source_unavailable"))?,
        );
    }
    // Le cwd peut être un checkout lié. Protéger aussi le principal et les
    // autres checkouts inscrits ; common-dir ne désigne que leurs métadonnées.
    let listing = run_git(
        &source,
        &["worktree", "list", "--porcelain", "-z"],
        READ_TIMEOUT,
    )?
    .complete()
    .map_err(|_| refusal("source_inventory_unavailable"))?;
    for row in listing.split('\0') {
        if let Some(path) = row.strip_prefix("worktree ") {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(refusal("source_inventory_unavailable"));
            }
            forbidden.push(path.canonicalize().unwrap_or(path));
        }
    }
    if forbidden.iter().any(|path| destination.starts_with(path)) {
        return Err(refusal("destination_inside_source"));
    }
    let mut objects = Objects::open(&source)?;
    // Valider TOUS les liens avant le premier effet disque.
    let mut links = BTreeMap::new();
    for entry in manifest.entries.iter().filter(|e| e.mode == "120000") {
        let mut bytes = Vec::new();
        objects.object(entry, &mut bytes)?;
        let target = String::from_utf8(bytes).map_err(|_| refusal("link_unsupported"))?;
        if target.is_empty() || target.chars().any(char::is_control) {
            return Err(refusal("link_unsupported"));
        }
        links.insert(entry.path.clone(), target);
    }
    for (path, target) in &links {
        resolve_link(path, target, &links)?;
    }
    let public_notice = notice(manifest, &links)?;
    let parent_anchor = Directory::open(&canonical_parent)?;
    let root = parent_anchor.create(
        destination
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| refusal("destination_invalid"))?,
    )?;
    let mut created = BTreeSet::new();
    if manifest.cwd != "." {
        // Matérialiser le dossier de travail même si ses fichiers sont tous omis.
        root.parent(
            &format!("{}/__cwd_only__", manifest.cwd),
            true,
            &mut created,
        )?;
    }
    for entry in manifest
        .entries
        .iter()
        .filter(|e| e.selected && e.mode != "120000")
    {
        let (parent, name) = root.parent(&entry.path, true, &mut created)?;
        let mut file = parent.file(&name, true)?;
        objects.object(entry, &mut file)?;
        file.set_permissions(fs::Permissions::from_mode(if entry.mode == "100755" {
            0o500
        } else {
            0o400
        }))
        .map_err(compact::io_failure)?;
        if file.metadata().map_err(compact::io_failure)?.dev()
            != root.0.metadata().map_err(compact::io_failure)?.dev()
        {
            return Err(refusal("view_device_changed"));
        }
        sync_to_device(&file, objects.deadline)?;
    }
    // Aucun lien ne participe à la création d'un fichier ou dossier.
    for (path, target) in &links {
        let (parent, name) = root.parent(&path, true, &mut created)?;
        parent.link(&name, target)?;
    }
    observed_links(&root, manifest, objects.deadline)?;
    let mut file = root.file(NOTICE, true)?;
    file.write_all(&public_notice)
        .map_err(compact::io_failure)?;
    objects.stop()?;
    // Ordre descendants -> parents ; aucun dossier n'est déduit durable depuis
    // la seule synchronisation de son parent.
    for directory in created.iter().rev() {
        let (directory, _) = root.parent(
            &format!("{directory}/__sync_only__"),
            false,
            &mut BTreeSet::new(),
        )?;
        sync_to_device(&directory.0, objects.deadline)?;
    }
    sync_to_device(&root.0, objects.deadline)?;
    sync_to_device(&parent_anchor.0, objects.deadline)?;
    if file.metadata().map_err(compact::io_failure)?.dev()
        != root.0.metadata().map_err(compact::io_failure)?.dev()
    {
        return Err(refusal("view_device_changed"));
    }
    file.sync_all().map_err(compact::io_failure)?;
    audit(manifest, destination)
}
fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.version != 1
        || !sha(&manifest.base)
        || manifest.entries.is_empty()
        || manifest.entries.len() > MAX_ENTRIES
    {
        return Err(refusal("manifest_invalid"));
    }
    if manifest.cwd != "."
        && (!path_valid(&manifest.cwd)
            || manifest.cwd.split('/').count() > 64
            || !manifest
                .entries
                .iter()
                .any(|e| e.path.starts_with(&format!("{}/", manifest.cwd))))
    {
        return Err(refusal("cwd_not_in_commit"));
    }
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    let mut link_bytes = 0u64;
    for entry in &manifest.entries {
        if !path_valid(&entry.path)
            || entry.path.split('/').count() > 64
            || !sha(&entry.oid)
            || !paths.insert(&entry.path)
            || !matches!(entry.mode.as_str(), "100644" | "100755" | "120000")
            || entry.selected != compact::selected(&entry.path, entry.bytes, entry.mode == "120000")
            || (entry.mode == "120000" && entry.bytes > MAX_LINK)
        {
            return Err(refusal("manifest_invalid"));
        }
        if entry.mode == "120000" {
            link_bytes = link_bytes
                .checked_add(entry.bytes)
                .ok_or_else(|| refusal("link_metadata_limit"))?;
            if link_bytes > 8 * MAX_OUTPUT as u64 {
                return Err(refusal("link_metadata_limit"));
            }
        }
        if entry.selected {
            total = total
                .checked_add(entry.bytes)
                .ok_or_else(|| refusal("byte_limit"))?;
            if total > MAX_BYTES {
                return Err(refusal("byte_limit"));
            }
        }
    }
    manifest_paths(manifest)?;
    Ok(())
}

fn check_file(
    entry: &Entry,
    parent: &Directory,
    name: &str,
    deadline: Instant,
) -> Result<(), String> {
    let mut hash = blob_hash(entry.bytes);
    if entry.mode == "120000" {
        let bytes = parent.read_link(name)?;
        if bytes.len() as u64 != entry.bytes {
            return Err(refusal("view_integrity_failed"));
        }
        hash.update(&bytes);
    } else {
        let mut file = parent.file(name, false)?;
        let metadata = file.metadata().map_err(compact::io_failure)?;
        if !metadata.is_file()
            || metadata.len() != entry.bytes
            || (metadata.permissions().mode() & 0o111 != 0) != (entry.mode == "100755")
        {
            return Err(refusal("view_integrity_failed"));
        }
        let mut buffer = [0; 65536];
        let mut remaining = entry.bytes;
        while remaining > 0 {
            if Instant::now() >= deadline {
                return Err(refusal("view_audit_timeout"));
            }
            let limit = remaining.min(buffer.len() as u64) as usize;
            let n = file
                .read(&mut buffer[..limit])
                .map_err(compact::io_failure)?;
            if n == 0 {
                return Err(refusal("view_integrity_failed"));
            }
            hash.update(&buffer[..n]);
            remaining -= n as u64;
        }
        if file.read(&mut buffer[..1]).map_err(compact::io_failure)? != 0 {
            return Err(refusal("view_integrity_failed"));
        }
    }
    if hexadecimal(hash.finish().as_ref()) != entry.oid {
        return Err(refusal("view_integrity_failed"));
    }
    Ok(())
}
struct Audit {
    bytes: u64,
    found: BTreeSet<String>,
}
pub(super) fn audit(manifest: &Manifest, root: &Path) -> Result<(), String> {
    audit_inventory(manifest, root).map(|_| ())
}
fn audit_inventory(manifest: &Manifest, root: &Path) -> Result<Audit, String> {
    validate_manifest(manifest)?;
    if root.canonicalize().ok().as_deref() != Some(root) {
        return Err(refusal("view_integrity_failed"));
    }
    let deadline = Instant::now() + DEADLINE;
    let anchor = Directory::open(root)?;
    let links = observed_links(&anchor, manifest, deadline)?;
    let expected_notice = notice(manifest, &links)?;
    let entries = manifest
        .entries
        .iter()
        .map(|e| (e.path.as_str(), e))
        .collect::<BTreeMap<_, _>>();
    let (known_paths, _) = manifest_paths(manifest)?;
    let mut found = BTreeSet::new();
    let mut stack = vec![(anchor, String::new())];
    let mut count = 0usize;
    let mut audited_bytes = 0u64;
    while let Some((directory, prefix)) = stack.pop() {
        if Instant::now() >= deadline {
            return Err(refusal("view_audit_timeout"));
        }
        for name in directory.names()? {
            count += 1;
            if count > MAX_ENTRIES + 1 {
                return Err(refusal("view_entry_limit"));
            }
            let relative = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let kind = directory.kind(&name)?;
            if relative == NOTICE {
                if kind != libc::S_IFREG {
                    return Err(refusal("view_integrity_failed"));
                }
                let expected = &expected_notice;
                let mut file = directory.file(&name, false)?;
                let meta = file.metadata().map_err(compact::io_failure)?;
                if !meta.is_file() || meta.len() != expected.len() as u64 {
                    return Err(refusal("view_integrity_failed"));
                }
                let mut actual = Vec::new();
                Read::by_ref(&mut file)
                    .take(expected.len() as u64 + 1)
                    .read_to_end(&mut actual)
                    .map_err(compact::io_failure)?;
                if actual != *expected {
                    return Err(refusal("view_integrity_failed"));
                }
            } else if kind == libc::S_IFDIR
                && known_paths.contains(&relative)
                && !entries.contains_key(relative.as_str())
            {
                stack.push((directory.child(&name)?, relative.clone()));
            } else if let Some(entry) = entries.get(relative.as_str()) {
                if kind
                    != if entry.mode == "120000" {
                        libc::S_IFLNK
                    } else {
                        libc::S_IFREG
                    }
                {
                    return Err(refusal("view_integrity_failed"));
                }
                audited_bytes = audited_bytes
                    .checked_add(entry.bytes)
                    .ok_or_else(|| refusal("byte_limit"))?;
                if audited_bytes > MAX_BYTES {
                    return Err(refusal("byte_limit"));
                }
                check_file(entry, &directory, &name, deadline)?;
            } else {
                return Err(refusal("view_integrity_failed"));
            }
            found.insert(relative);
        }
    }
    if !found.contains(NOTICE)
        || manifest
            .entries
            .iter()
            .any(|e| e.selected && !found.contains(&e.path))
    {
        return Err(refusal("view_integrity_failed"));
    }
    Ok(Audit {
        bytes: audited_bytes,
        found,
    })
}

/// Matérialisation interne d'un seul chemin lié au manifeste privé. Ce helper
/// ne remplace pas les gardes de Task/instance du port natif qui l'appellera.
/// true = fichier créé ; false = fichier déjà constaté exact, jamais écrasé.
pub(super) fn materialize(
    source: &Path,
    manifest: &Manifest,
    root: &Path,
    path: &str,
) -> Result<bool, String> {
    let inventory = audit_inventory(manifest, root)?;
    let entry = manifest
        .entries
        .iter()
        .find(|e| e.path == path)
        .ok_or_else(|| refusal("path_not_in_commit"))?;
    if inventory.found.contains(path) {
        return Ok(false);
    }
    if entry.selected || entry.mode == "120000" {
        return Err(refusal("view_integrity_failed"));
    }
    if inventory
        .bytes
        .checked_add(entry.bytes)
        .is_none_or(|n| n > MAX_BYTES)
    {
        return Err(refusal("byte_limit"));
    }
    let root = Directory::open(root)?;
    let mut objects = Objects::open(source)?;
    // Objet absent : aucun dossier ou fichier créé. Sa disparition ultérieure
    // reste un échec conservé ; elle ne permet aucune substitution de base.
    objects.send(format!("info {}\n", entry.oid).as_bytes())?;
    if objects.line()? != format!("{} blob {}", entry.oid, entry.bytes) {
        return Err(refusal("git_object_unavailable"));
    }
    let (parent, name) = root.ensure_parent(path)?;
    let mut file = parent.file(&name, true)?;
    objects.object(entry, &mut file)?;
    file.set_permissions(fs::Permissions::from_mode(if entry.mode == "100755" {
        0o500
    } else {
        0o400
    }))
    .map_err(compact::io_failure)?;
    file.sync_all().map_err(compact::io_failure)?;
    parent.0.sync_all().map_err(compact::io_failure)?;
    objects.stop()?;
    check_file(entry, &parent, &name, Instant::now() + DEADLINE)?;
    Ok(true)
}

pub(crate) use crate::delegation::ViewBinding;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ViewIntent {
    version: u8,
    pub(crate) binding: ViewBinding,
    pub(crate) source: PathBuf,
    pub(crate) data_root: PathBuf,
    manifest: Manifest,
}
/// Intention légère durable, réservée avant toute commande Git.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ViewPlan {
    version:u8,
    pub(crate) binding:ViewBinding,
    pub(crate) source:PathBuf,
    pub(crate) data_root:PathBuf,
    pub(crate) base:String,
    pub(crate) source_root:PathBuf,
    pub(crate) git_identity:String,
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub(crate) native_placement_v1:bool,
    created_at:u64,
    expires_at:u64,
}
const MAX_PLAN:usize=16*1024;
impl ViewPlan {
    pub(crate) fn new(binding:ViewBinding,source:&Path,data_root:&Path,base:&str,source_root:&Path,git_identity:&str)->Result<Self,String> {
        let now=Self::now()?;
        let seed=Self {version:1,binding,source:source.into(),data_root:data_root.into(),
            base:base.into(),source_root:source_root.into(),git_identity:git_identity.into(),native_placement_v1:false,created_at:now,expires_at:now.checked_add(120).ok_or_else(||refusal("proof_invalid"))?};
        seed.encoded()?;Ok(seed)
    }
    pub(crate) fn new_native(binding:ViewBinding,source:&Path,base:&str,source_root:&Path,git_identity:&str)
        ->Result<Self,String> {
        let mut seed=Self::new(binding,source,&Self::native_data_root(source_root)?,base,source_root,git_identity)?;
        seed.native_placement_v1=true;seed.encoded()?;Ok(seed)
    }
    fn now()->Result<u64,String> {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|duration|duration.as_secs()).map_err(|_|refusal("preparation_expired"))
    }
    fn encoded(&self)->Result<Vec<u8>,String> {
        ViewIntent::validate_roots(&self.binding,&self.source,&self.data_root)?;
        ViewIntent::validate_roots(&self.binding,&self.source_root,&self.data_root)?;
        if !self.source.starts_with(&self.source_root) || self.git_identity.is_empty() || self.git_identity.len()>4096
            || self.git_identity.chars().any(|c|c.is_control()) {
            return Err(refusal("proof_binding_invalid"));
        }
        if self.version!=1 || self.base.len()!=40 || !self.base.bytes().all(|b|b.is_ascii_hexdigit())
            || self.created_at==0 || self.expires_at.checked_sub(self.created_at)!=Some(120) {
            return Err(refusal("proof_binding_invalid"));
        }
        let bytes=serde_json::to_vec(self).map_err(|_|refusal("proof_invalid"))?;
        if bytes.len()>MAX_PLAN {return Err(refusal("proof_byte_limit"));}
        Ok(bytes)
    }
    pub(crate) fn live(&self)->Result<(),String> {
        self.encoded()?;
        let now=Self::now()?;
        if now<self.created_at || now>=self.expires_at {return Err(refusal("preparation_expired"));}
        Ok(())
    }
    pub(crate) fn native_data_root(source_root:&Path)->Result<PathBuf,String> {
        let parent=source_root.parent().ok_or_else(||refusal("destination_invalid"))?;
        if parent==source_root {return Err(refusal("destination_invalid"));}
        Ok(parent.join(".bridget-commit-views"))
    }
    fn planned_data_root(&self,top:&Path)->Result<PathBuf,String> {
        if !self.native_placement_v1 {return Ok(self.data_root.clone());}
        let device=Directory::open(&self.source_root)?.0.metadata().map_err(compact::io_failure)?.dev();
        // Politique engagée dans la graine : premier ancêtre privé sur ce
        // volume et hors Git. Pas un repli de mapping ni une racine libre.
        for parent in self.source_root.ancestors().skip(1).take(256) {
            if parent.canonicalize().map_err(compact::io_failure)?!=parent {return Err(refusal("destination_invalid"));}
            let metadata=Directory::open(parent)?.0.metadata().map_err(compact::io_failure)?;
            if metadata.dev()!=device {return Err(refusal("destination_volume_mismatch"));}
            if parent.starts_with(top) {continue;}
            if metadata.uid()==unsafe{libc::geteuid()} && metadata.mode()&0o022==0 {
                return Ok(parent.join(".bridget-commit-views"));
            }
        }
        Err(refusal("destination_unavailable"))
    }
    /// Exécuté seulement dans le helper après RELEASE : aucun Git ni mkdir
    /// depuis la boucle de commande. La sélection A précède le premier effet.
    fn planned_intent(&self)->Result<ViewIntent,String> {
        self.check_source()?;
        let source=self.source.canonicalize().map_err(|_|refusal("source_unavailable"))?;
        if source!=self.source {return Err(refusal("source_identity_changed"));}
        let manifest=plan(&source,&self.base)?;
        self.live()?;
        let mut top=source.clone();
        if manifest.cwd!="." {
            for part in Path::new(&manifest.cwd).components().rev() {
                if top.file_name()!=Some(part.as_os_str()) || !top.pop() {return Err(refusal("source_unavailable"));}
            }
        }
        let data_root=self.planned_data_root(&top)?;
        let parent_path=data_root.parent().ok_or_else(||refusal("destination_invalid"))?;
        if parent_path.canonicalize().map_err(|_|refusal("destination_invalid"))?!=parent_path
            || data_root.starts_with(&top) || (!self.native_placement_v1
                && parent_path!=self.source_root.parent().ok_or_else(||refusal("destination_invalid"))?) {
            return Err(refusal("destination_inside_source"));
        }
        let parent=Directory::open(parent_path)?;
        let parent_meta=parent.0.metadata().map_err(compact::io_failure)?;
        let source_meta=Directory::open(&self.source_root)?.0.metadata().map_err(compact::io_failure)?;
        if parent_meta.uid()!=unsafe{libc::geteuid()} || parent_meta.mode()&0o022!=0
            || parent_meta.dev()!=source_meta.dev() {return Err(refusal("destination_invalid"));}
        let name=data_root.file_name().and_then(|name|name.to_str()).ok_or_else(||refusal("destination_invalid"))?;
        let data=match parent.optional_kind(name)? {
            Some(libc::S_IFDIR)=>parent.child(name)?,
            None if self.native_placement_v1 || data_root==Self::native_data_root(&self.source_root)?=>{
                let child=parent.create(name)?;parent.0.sync_all().map_err(compact::io_failure)?;child
            }
            _=>return Err(refusal("destination_invalid")),
        };
        let metadata=data.0.metadata().map_err(compact::io_failure)?;
        if metadata.uid()!=unsafe{libc::geteuid()} || metadata.mode()&0o077!=0
            || metadata.dev()!=source_meta.dev() {return Err(refusal("destination_invalid"));}
        let mut stat=std::mem::MaybeUninit::<libc::statvfs>::uninit();
        if unsafe{libc::fstatvfs(data.0.as_raw_fd(),stat.as_mut_ptr())}!=0 {
            return Err(refusal("storage_space_unavailable"));
        }
        let stat=unsafe{stat.assume_init()};
        let block=if stat.f_frsize>0 {stat.f_frsize as u128} else {stat.f_bsize as u128};
        if block==0 {return Err(refusal("storage_space_unavailable"));}
        let needed=manifest.entries.iter().filter(|entry|entry.selected).map(|entry|entry.bytes as u128).sum::<u128>()
            +manifest.entries.len() as u128*block+MAX_INTENT as u128+MAX_PLAN as u128;
        if (stat.f_bavail as u128).saturating_mul(block)<needed {
            return Err("workspace_disk_full: espace insuffisant pour cette vue".into());
        }
        self.live()?;
        let intent=ViewIntent{version:1,binding:self.binding.clone(),source,data_root,manifest};
        intent.encoded()?;Ok(intent)
    }
    pub(crate) fn remaining(&self)->Result<Duration,String> {
        self.live()?;
        let remaining=self.expires_at.checked_sub(Self::now()?).filter(|seconds|*seconds>0)
            .ok_or_else(||refusal("preparation_expired"))?;
        Ok(Duration::from_secs(remaining))
    }
    fn check_source(&self)->Result<(),String> {
        self.live()?;
        let origin=run_git(&self.source_root,
            &["config","--local","--no-includes","--get","remote.origin.url"],READ_TIMEOUT)?;
        if !origin.success() || origin.truncated
            || origin.stdout.trim_end_matches(['\r','\n'])!=self.git_identity {
            return Err(refusal("source_identity_changed"));
        }
        self.live()
    }
    pub(crate) fn proof_digest(&self)->Result<String,String>{Ok(view_digest(&self.encoded()?))}
    pub(crate) fn reserved_bytes(&self)->u64 {(MAX_INTENT+MAX_PLAN+MAX_PUBLICATION+2*2048) as u64}
    pub(crate) fn adopted_reserved_bytes(&self,intent:&ViewIntent)->Result<u64,String> {
        self.matches(intent)?;
        intent.reserved_bytes()?.checked_add(self.encoded()?.len() as u64)
            .ok_or_else(||refusal("proof_byte_limit"))
    }

    pub(crate) fn matches(&self,intent:&ViewIntent)->Result<(),String> {
        self.encoded()?;
        intent.validate()?;
        let placement_matches=if self.native_placement_v1 {
            let mut top=intent.source.clone();
            for part in Path::new(&intent.manifest.cwd).components().rev() {
                if matches!(part,std::path::Component::CurDir){continue;}
                if !matches!(part,std::path::Component::Normal(_))
                    || top.file_name()!=Some(part.as_os_str()) || !top.pop() {
                    return Err(refusal("proof_binding_mismatch"));
                }
            }
            // Une graine de sous-mapping n'accorde jamais un ancêtre dans Git.
            // Réutiliser la même politique que le helper, pas une seconde liste.
            !intent.data_root.starts_with(&top) && self.planned_data_root(&top)?==intent.data_root
        } else {intent.data_root==self.data_root};
        if intent.binding!=self.binding || intent.source!=self.source || !placement_matches
            || intent.base_commit()!=self.base {return Err(refusal("proof_binding_mismatch"));}
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewPublished {
    version: u8,
    intent_digest: String,
    device: u64,
    inode: u64,
}

/// Ancre privée d'une vue close. Le manifeste public n'a aucune autorité
/// avant reconstruction et comparaison à cette empreinte complète.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosedViewIntent {
    format: String,
    binding: ViewBinding,
    source: PathBuf,
    data_root: PathBuf,
    intent_digest: String,
}
impl ClosedViewIntent {
    fn from_intent(intent: &ViewIntent) -> Result<Self, String> {
        Ok(Self {
            format: "closed_ref_v1".into(),
            binding: intent.binding.clone(),
            source: intent.source.clone(),
            data_root: intent.data_root.clone(),
            intent_digest: intent.proof_digest()?,
        })
    }
    fn encoded(&self) -> Result<Vec<u8>, String> {
        let bytes = serde_json::to_vec(self).map_err(|_| refusal("proof_invalid"))?;
        if bytes.len() > 16 * 1024 {
            return Err(refusal("proof_byte_limit"));
        }
        Ok(bytes)
    }
}

const MAX_INTENT: usize = 8 * MAX_OUTPUT;
const MAX_PUBLICATION: usize = 1024;
fn view_digest(bytes: &[u8]) -> String {
    hexadecimal(ring::digest::digest(&ring::digest::SHA256, bytes).as_ref())
}
fn digest_valid(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
}
impl ViewBinding {
    pub(crate) fn key(&self) -> Result<String, String> {
        self.worker
            .validate()
            .map_err(|_| refusal("proof_binding_invalid"))?;
        self.command
            .validate_preparation()
            .map_err(|_| refusal("proof_binding_invalid"))?;
        if !digest_valid(&self.prepare_digest) {
            return Err(refusal("proof_binding_invalid"));
        }
        Ok(view_digest(
            &serde_json::to_vec(self).map_err(|_| refusal("proof_binding_invalid"))?,
        ))
    }
}
impl ViewIntent {
    pub(crate) fn proof_digest(&self) -> Result<String, String> {
        Ok(view_digest(&self.encoded()?))
    }
    pub(crate) fn base_commit(&self) -> &str {
        &self.manifest.base
    }
    pub(crate) fn plan(
        binding: ViewBinding,
        source: &Path,
        data_root: &Path,
        base: &str,
    ) -> Result<Self, String> {
        binding.key()?;
        let source = source
            .canonicalize()
            .map_err(|_| refusal("source_unavailable"))?;
        let canonical_data = data_root
            .canonicalize()
            .map_err(|_| refusal("destination_invalid"))?;
        if data_root != canonical_data {
            return Err(refusal("destination_invalid"));
        }
        let intent = Self {
            version: 1,
            binding,
            manifest: plan(&source, base)?,
            source,
            data_root: canonical_data,
        };
        intent.encoded()?;
        Ok(intent)
    }
    fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(refusal("proof_binding_invalid"));
        }
        Self::validate_roots(&self.binding, &self.source, &self.data_root)?;
        validate_manifest(&self.manifest)
    }
    fn validate_roots(
        binding: &ViewBinding,
        source: &Path,
        data_root: &Path,
    ) -> Result<(), String> {
        binding.key()?;
        if !source.is_absolute()
            || !data_root.is_absolute()
            || source
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
            || data_root
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
        {
            return Err(refusal("proof_binding_invalid"));
        }
        Ok(())
    }
    fn encoded(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| refusal("proof_invalid"))?;
        if bytes.len() > MAX_INTENT {
            return Err(refusal("proof_byte_limit"));
        }
        Ok(bytes)
    }
    pub(crate) fn closed_reserved_bytes(&self) -> Result<u64, String> {
        Ok(
            (ClosedViewIntent::from_intent(self)?.encoded()?.len() + MAX_PUBLICATION + 2 * 2048)
                as u64,
        )
    }
    /// O(n*d*log(n*d)), validation du manifeste comprise ; n <=65536 et d <=64.
    /// Remplacer la racine en conservant le décalage du cwd, sans HEAD ni notice.
    pub(crate) fn execution_paths(&self, source_root: &Path) -> Result<(PathBuf, PathBuf), String> {
        self.validate()?;
        if !source_root.is_absolute()
            || source_root
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(refusal("proof_binding_mismatch"));
        }
        let relative = self
            .source
            .strip_prefix(source_root)
            .map_err(|_| refusal("proof_binding_mismatch"))?;
        let mut offset = PathBuf::from(&self.manifest.cwd);
        for part in relative.components().rev() {
            if matches!(part, std::path::Component::CurDir) {
                continue;
            }
            if !matches!(part, std::path::Component::Normal(_))
                || offset.file_name() != Some(part.as_os_str())
                || !offset.pop()
            {
                return Err(refusal("proof_binding_mismatch"));
            }
        }
        let root = self.destination()?;
        Ok((root.join(offset), root.join(&self.manifest.cwd)))
    }

    pub(crate) fn destination(&self) -> Result<PathBuf, String> {
        Ok(self.data_root.join(self.binding.key()?))
    }
    /// Budget des preuves privées : intention, publication et deux marqueurs.
    /// Le magasin compte aussi leurs orphelins. Les fichiers exportés vivent
    /// sur leur volume ; ce budget de métadonnées n'est pas un quota disque.
    pub(crate) fn reserved_bytes(&self) -> Result<u64, String> {
        // Même un manifeste vide réserve sa future référence dès le début.
        let bytes = self
            .encoded()?
            .len()
            .max(ClosedViewIntent::from_intent(self)?.encoded()?.len());
        Ok((bytes + MAX_PUBLICATION + 2 * 2048) as u64)
    }
}

/// Répertoire privé déjà préparé par l'autorité du store. Aucun mkdir caché,
/// aucune recherche de preuve depuis la notice modifiable de la vue.
pub(crate) struct ViewProofs {
    directory: Directory,
    path: PathBuf,
}
/// Le créneau appartient au contrôleur, pas aux descriptions de fichier
/// éventuellement héritées par un autre fork avant son exec. CLOEXEC empêche
/// la fuite après exec ; LOCK_UN termine explicitement notre possession.
/// Le marqueur durable garde séparément la preuve d'un processus non arrêté.
struct ViewLease(fs::File);
impl Drop for ViewLease {
    fn drop(&mut self) {
        unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) };
    }
}
impl ViewProofs {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        if path
            .canonicalize()
            .map_err(|_| refusal("proof_unavailable"))?
            != path
        {
            return Err(refusal("proof_path_unsafe"));
        }
        let directory = Directory::open(path)?;
        let metadata = directory.0.metadata().map_err(compact::io_failure)?;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(refusal("proof_path_unsafe"));
        }
        Ok(Self {
            directory,
            path: path.to_owned(),
        })
    }
    fn read(&self, name: &str, maximum: usize) -> Result<Option<Vec<u8>>, String> {
        if self.directory.optional_kind(name)?.is_none() {
            return Ok(None);
        }
        let file = self.directory.file(name, false)?;
        let metadata = file.metadata().map_err(compact::io_failure)?;
        if !metadata.is_file()
            || metadata.nlink() != 1
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o077 != 0
            || metadata.len() > maximum as u64
        {
            return Err(refusal("proof_invalid"));
        }
        let mut bytes = Vec::new();
        file.take(maximum as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(compact::io_failure)?;
        if bytes.len() > maximum {
            return Err(refusal("proof_byte_limit"));
        }
        Ok(Some(bytes))
    }
    /// Création exclusive : un fichier partiel après coupure reste un refus,
    /// jamais une permission de le remplacer ou de recréer la vue.
    fn write_new(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        let mut file = self.directory.file(name, true)?;
        file.write_all(bytes).map_err(compact::io_failure)?;
        file.sync_all().map_err(compact::io_failure)?;
        self.directory.0.sync_all().map_err(compact::io_failure)
    }
    pub(crate) fn write_plan(&self,seed:&ViewPlan)->Result<(),String> {
        let bytes=seed.encoded()?;let key=seed.binding.key()?;
        if self.path.starts_with(&seed.source) || self.path.starts_with(&seed.data_root) {
            return Err(refusal("proof_path_unsafe"));
        }
        if let Some(existing)=self.read(&format!("{key}.plan.json"),MAX_PLAN)? {
            return if existing==bytes {Ok(())} else {Err(refusal("proof_binding_mismatch"))};
        }
        seed.live()?;
        if self.read(&format!("{key}.intent.json"),MAX_INTENT)?.is_some()
            || fs::symlink_metadata(seed.data_root.join(&key)).is_ok() {
            return Err(refusal("preparation_incomplete"));
        }
        self.write_new(&format!("{key}.plan.json"),&bytes)
    }
    pub(crate) fn load_plan(&self,key:&str)->Result<ViewPlan,String> {
        if !digest_valid(key) {return Err(refusal("proof_binding_invalid"));}
        let bytes=self.read(&format!("{key}.plan.json"),MAX_PLAN)?
            .ok_or_else(||refusal("proof_unavailable"))?;
        let seed:ViewPlan=serde_json::from_slice(&bytes).map_err(|_|refusal("proof_invalid"))?;
        if seed.encoded()?!=bytes || seed.binding.key()?!=key
            || self.path.starts_with(&seed.source) || self.path.starts_with(&seed.data_root) {
            return Err(refusal("proof_binding_mismatch"));
        }
        Ok(seed)
    }
    pub(crate) fn write_intent(&self, intent: &ViewIntent) -> Result<(), String> {
        let bytes = intent.encoded()?;
        let key = intent.binding.key()?;
        let destination = intent.destination()?;
        if self.path.starts_with(&intent.data_root) || self.path.starts_with(&intent.source) {
            return Err(refusal("proof_path_unsafe"));
        }
        // Une preuve ne peut être créée dans les données qu'elle atteste.
        // dev/inode désignent le répertoire ouvert, même en cas de substitution.
        let data = Directory::open(&intent.data_root)?;
        if self
            .directory
            .0
            .metadata()
            .map_err(compact::io_failure)?
            .dev()
            == data.0.metadata().map_err(compact::io_failure)?.dev()
            && self
                .directory
                .0
                .metadata()
                .map_err(compact::io_failure)?
                .ino()
                == data.0.metadata().map_err(compact::io_failure)?.ino()
        {
            return Err(refusal("proof_path_unsafe"));
        }
        if let Some(existing) = self.read(&format!("{key}.intent.json"), MAX_INTENT)? {
            return if existing == bytes {
                Ok(())
            } else {
                Err(refusal("proof_binding_mismatch"))
            };
        }
        if fs::symlink_metadata(&destination).is_ok()
            || self
                .read(&format!("{key}.published.json"), MAX_PUBLICATION)?
                .is_some()
        {
            return Err(refusal("preparation_incomplete"));
        }
        self.write_new(&format!("{key}.intent.json"), &bytes)
    }
    pub(crate) fn load(&self, key: &str) -> Result<ViewIntent, String> {
        if !digest_valid(key) {
            return Err(refusal("proof_binding_invalid"));
        }
        let bytes = self
            .read(&format!("{key}.intent.json"), MAX_INTENT)?
            .ok_or_else(|| refusal("proof_unavailable"))?;
        let intent = match serde_json::from_slice::<ViewIntent>(&bytes) {
            Ok(intent) if intent.encoded()? == bytes => intent,
            Ok(_) => return Err(refusal("proof_binding_mismatch")),
            Err(_) => {
                let closed: ClosedViewIntent =
                    serde_json::from_slice(&bytes).map_err(|_| refusal("proof_invalid"))?;
                if closed.format != "closed_ref_v1"
                    || !digest_valid(&closed.intent_digest)
                    || closed.encoded()? != bytes
                    || closed.binding.key()? != key
                    || self.path.starts_with(&closed.data_root)
                    || self.path.starts_with(&closed.source)
                {
                    return Err(refusal("proof_binding_mismatch"));
                }
                // Les racines ne viennent que de la petite preuve privée, jamais
                // des champs de la carte. Vérifier avant toute lecture publique.
                ViewIntent::validate_roots(&closed.binding, &closed.source, &closed.data_root)?;
                let publication = self.read_publication(key, &closed.intent_digest)?;
                let destination = closed.data_root.join(key);
                let root = Directory::open(&destination)?;
                let metadata = root.0.metadata().map_err(compact::io_failure)?;
                if metadata.dev() != publication.device || metadata.ino() != publication.inode {
                    return Err(refusal("view_identity_changed"));
                }
                let mut file = root.file(NOTICE, false)?;
                let meta = file.metadata().map_err(compact::io_failure)?;
                if !meta.is_file() || meta.len() > MAX_INTENT as u64 {
                    return Err(refusal("proof_invalid"));
                }
                let mut card = Vec::new();
                std::io::Read::by_ref(&mut file)
                    .take(MAX_INTENT as u64 + 1)
                    .read_to_end(&mut card)
                    .map_err(compact::io_failure)?;
                if card.len() > MAX_INTENT {
                    return Err(refusal("proof_byte_limit"));
                }
                let mut value: serde_json::Value =
                    serde_json::from_slice(&card).map_err(|_| refusal("proof_invalid"))?;
                let manifest = serde_json::from_value(
                    value
                        .get_mut("manifest")
                        .ok_or_else(|| refusal("proof_invalid"))?
                        .take(),
                )
                .map_err(|_| refusal("proof_invalid"))?;
                let intent = ViewIntent {
                    version: 1,
                    binding: closed.binding,
                    source: closed.source,
                    data_root: closed.data_root,
                    manifest,
                };
                if intent.proof_digest()? != closed.intent_digest {
                    return Err(refusal("proof_binding_mismatch"));
                }
                let after = Directory::open(&destination)?
                    .0
                    .metadata()
                    .map_err(compact::io_failure)?;
                if after.dev() != metadata.dev() || after.ino() != metadata.ino() {
                    return Err(refusal("view_identity_changed"));
                }
                intent
            }
        };
        intent.validate()?;
        if intent.binding.key()? != key {
            return Err(refusal("proof_binding_mismatch"));
        }
        if self.path.starts_with(&intent.data_root) || self.path.starts_with(&intent.source) {
            return Err(refusal("proof_path_unsafe"));
        }
        Ok(intent)
    }
    /// L'appelant natif a déjà figé le reçu sous son verrou. Le créneau de
    /// processus et l'audit empêchent la réduction d'une exportation en cours.
    /// O(n*d*log(n*d)) comme l'audit ; aucune écriture dans la vue.
    pub(crate) fn compact_closed(&self, intent: &ViewIntent) -> Result<u64, String> {
        let key = intent.binding.key()?;
        let _lease = self.lease(&key)?;
        let expected = ClosedViewIntent::from_intent(intent)?.encoded()?;
        self.recover(&intent.binding, &intent.source, &intent.data_root)?;
        let name = format!("{key}.intent.json");
        let actual = self
            .read(&name, MAX_INTENT)?
            .ok_or_else(|| refusal("proof_unavailable"))?;
        if actual != expected {
            if actual != intent.encoded()? {
                return Err(refusal("proof_binding_mismatch"));
            }
            // Petit temporaire dans la réserve de transition du store. En cas de
            // coupure il est compté ; une version partielle reste conservée/refusée.
            let temporary = format!("{key}.closed.tmp");
            match self.read(&temporary, 16 * 1024)? {
                Some(bytes) if bytes == expected => {}
                Some(_) => return Err(refusal("preparation_incomplete")),
                None => self.write_new(&temporary, &expected)?,
            }
            let from = Directory::name(&temporary)?;
            let to = Directory::name(&name)?;
            if unsafe {
                libc::renameat(
                    self.directory.0.as_raw_fd(),
                    from.as_ptr(),
                    self.directory.0.as_raw_fd(),
                    to.as_ptr(),
                )
            } != 0
            {
                return Err(compact::io_failure(io::Error::last_os_error()));
            }
            self.directory.0.sync_all().map_err(compact::io_failure)?;
        }
        intent.closed_reserved_bytes()
    }
    /// Aucun helper d'écriture ni rejeu d'export sur une référence close.
    fn require_live_intent(&self, intent: &ViewIntent) -> Result<(), String> {
        if self
            .read(
                &format!("{}.intent.json", intent.binding.key()?),
                MAX_INTENT,
            )?
            .as_deref()
            != Some(intent.encoded()?.as_slice())
        {
            return Err(refusal("view_closed"));
        }
        Ok(())
    }
    pub(crate) fn read_intent(
        &self,
        binding: &ViewBinding,
        source: &Path,
        data_root: &Path,
    ) -> Result<ViewIntent, String> {
        let intent = self.load(&binding.key()?)?;
        if intent.binding != *binding || intent.source != source || intent.data_root != data_root {
            return Err(refusal("proof_binding_mismatch"));
        }
        Ok(intent)
    }
    pub(crate) fn source_marker(
        &self,
        binding: &ViewBinding,
    ) -> Result<Option<crate::managed_process::ManagedMarker>, String> {
        self.operation_marker(
            binding,
            &crate::managed_process::ManagedIdentity {
                instance_id: binding.command.task_id.clone(),
                command_id: binding.command.command_id.clone(),
                generation: 1,
            },
        )
    }
    pub(crate) fn operation_marker(
        &self,
        binding: &ViewBinding,
        expected: &crate::managed_process::ManagedIdentity,
    ) -> Result<Option<crate::managed_process::ManagedMarker>, String> {
        let Some(bytes) = self.read(&format!("view-{}.json", binding.key()?), 2048)? else {
            return Ok(None);
        };
        let marker: crate::managed_process::ManagedMarker =
            serde_json::from_slice(&bytes).map_err(|_| refusal("proof_invalid"))?;
        if marker.instance_id != expected.instance_id
            || marker.command_id != expected.command_id
            || marker.generation != expected.generation
            || marker.pgid == 0
            || marker.birth == 0
        {
            return Err(refusal("proof_binding_mismatch"));
        }
        Ok(Some(marker))
    }
    /// L'appelant détient le reçu d'arrêt durable. Ne retirer que le marqueur
    /// privé attendu ; aucun signal, aucun parcours ou retrait de données.
    pub(crate) fn remove_stopped_marker(
        &self,
        binding: &ViewBinding,
        expected: &crate::managed_process::ManagedMarker,
    ) -> Result<(), String> {
        let identity = crate::managed_process::ManagedIdentity {
            instance_id: expected.instance_id.clone(),
            command_id: expected.command_id.clone(),
            generation: expected.generation,
        };
        let Some(actual) = self.operation_marker(binding, &identity)? else {
            return Ok(());
        };
        if &actual != expected {
            return Err(refusal("proof_binding_mismatch"));
        }
        if crate::managed_process::group_exists(expected.pgid).map_err(compact::io_failure)?
            && crate::managed_process::process_birth(expected.pgid)
                .ok()
                .is_none_or(|birth| birth == expected.birth)
        {
            return Err(refusal("git_stop_unconfirmed"));
        }
        let name = Directory::name(&format!("view-{}.json", binding.key()?))?;
        if unsafe { libc::unlinkat(self.directory.0.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::NotFound {
                return Err(compact::io_failure(error));
            }
        }
        self.directory.0.sync_all().map_err(compact::io_failure)
    }

    fn lease(&self, key: &str) -> Result<ViewLease, String> {
        if !digest_valid(key) {
            return Err(refusal("proof_binding_invalid"));
        }
        let name = format!("{key}.lock");
        let file = self
            .directory
            .file(&name, self.directory.optional_kind(&name)?.is_none())?;
        let metadata = file.metadata().map_err(compact::io_failure)?;
        if !metadata.is_file()
            || metadata.nlink() != 1
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o077 != 0
        {
            return Err(refusal("proof_path_unsafe"));
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(refusal("export_busy"));
        }
        Ok(ViewLease(file))
    }
    fn matches_intent(&self, intent: &ViewIntent) -> Result<Vec<u8>, String> {
        let bytes = intent.encoded()?;
        if self.load(&intent.binding.key()?)?.encoded()? != bytes {
            return Err(refusal("proof_binding_mismatch"));
        }
        Ok(bytes)
    }
    pub(crate) fn publish(&self, intent: &ViewIntent) -> Result<(), String> {
        self.require_live_intent(intent)?;
        let bytes = self.matches_intent(intent)?;
        let destination = intent.destination()?;
        let root = Directory::open(&destination)?;
        let metadata = root.0.metadata().map_err(compact::io_failure)?;
        audit(&intent.manifest, &destination)?;
        let observed = Directory::open(&destination)?
            .0
            .metadata()
            .map_err(compact::io_failure)?;
        if metadata.dev() != observed.dev() || metadata.ino() != observed.ino() {
            return Err(refusal("view_identity_changed"));
        }
        let publication = ViewPublished {
            version: 1,
            intent_digest: view_digest(&bytes),
            device: metadata.dev(),
            inode: metadata.ino(),
        };
        let name = format!("{}.published.json", intent.binding.key()?);
        let encoded = serde_json::to_vec(&publication).map_err(|_| refusal("proof_invalid"))?;
        if encoded.len() > MAX_PUBLICATION {
            return Err(refusal("proof_byte_limit"));
        }
        if let Some(previous) = self.read(&name, MAX_PUBLICATION)? {
            return if previous == encoded {
                Ok(())
            } else {
                Err(refusal("proof_binding_mismatch"))
            };
        }
        self.write_new(&name, &encoded)
    }
    /// Métadonnées privées seulement. L'audit de données est effectué par le
    /// helper possédé après RELEASE, pas avant l'autorisation du contrôleur.
    fn publication(&self, intent: &ViewIntent) -> Result<ViewPublished, String> {
        self.matches_intent(intent)?;
        self.read_publication(&intent.binding.key()?, &intent.proof_digest()?)
    }
    fn read_publication(&self, key: &str, digest: &str) -> Result<ViewPublished, String> {
        let publication: ViewPublished = serde_json::from_slice(
            &self
                .read(&format!("{key}.published.json"), MAX_PUBLICATION)?
                .ok_or_else(|| refusal("preparation_incomplete"))?,
        )
        .map_err(|_| refusal("proof_invalid"))?;
        if publication.version != 1 || publication.intent_digest != digest {
            return Err(refusal("proof_binding_mismatch"));
        }
        Ok(publication)
    }

    /// Une publication et un audit exact sont tous deux nécessaires. La source
    /// peut être indisponible ; elle n'est jamais utilisée pour décider l'intégrité.
    pub(crate) fn recover(
        &self,
        binding: &ViewBinding,
        source: &Path,
        data_root: &Path,
    ) -> Result<ViewIntent, String> {
        let intent = self.read_intent(binding, source, data_root)?;
        let publication = self.publication(&intent)?;
        let destination = intent.destination()?;
        let metadata = Directory::open(&destination)?
            .0
            .metadata()
            .map_err(compact::io_failure)?;
        if publication.device != metadata.dev() || publication.inode != metadata.ino() {
            return Err(refusal("proof_binding_mismatch"));
        }
        audit(&intent.manifest, &destination)?;
        let after = Directory::open(&destination)?
            .0
            .metadata()
            .map_err(compact::io_failure)?;
        if after.dev() != metadata.dev() || after.ino() != metadata.ino() {
            return Err(refusal("view_identity_changed"));
        }
        Ok(intent)
    }
}

impl ViewProofs {
    pub(crate) fn file_available(&self, intent: &ViewIntent, path: &str) -> Result<bool, String> {
        if !intent
            .manifest
            .entries
            .iter()
            .any(|entry| entry.path == path && entry.mode != "120000")
        {
            return Err(refusal("proof_binding_invalid"));
        }
        self.recover(&intent.binding, &intent.source, &intent.data_root)?;
        match fs::symlink_metadata(intent.destination()?.join(path)) {
            Ok(metadata) => Ok(metadata.is_file() && !metadata.file_type().is_symlink()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(compact::io_failure(error)),
        }
    }
    pub(crate) fn workspace(&self, intent: &ViewIntent) -> Result<WorkspaceRecord, String> {
        self.require_live_intent(intent)?;
        self.audited_workspace(intent, WorkspaceState::Active)
    }
    fn audited_workspace(
        &self,
        intent: &ViewIntent,
        state: WorkspaceState,
    ) -> Result<WorkspaceRecord, String> {
        self.recover(&intent.binding, &intent.source, &intent.data_root)?;
        self.workspace_metadata(intent,state)
    }
    /// Métadonnées d'un instantané audité par le lecteur possédé. Ne remplace
    /// jamais recover() pour un rapport utilisateur ou une relecture Release.
    pub(crate) fn workspace_metadata(&self,intent:&ViewIntent,state:WorkspaceState)
        ->Result<WorkspaceRecord,String> {
        if matches!(&state,WorkspaceState::Active) {self.require_live_intent(intent)?;}
        let publication=self.publication(intent)?;
        let metadata=Directory::open(&intent.destination()?)?.0.metadata().map_err(compact::io_failure)?;
        if publication.device!=metadata.dev() || publication.inode!=metadata.ino() {
            return Err(refusal("view_identity_changed"));
        }
        if !job::source_group_stopped(self, intent)? {
            return Err(refusal("git_stop_unconfirmed"));
        }
        Ok(WorkspaceRecord {
            strategy: WorkspaceStrategy::CommitView,
            state,
            toplevel: intent.source.to_string_lossy().into(),
            path: intent.destination()?.to_string_lossy().into(),
            branch: String::new(),
            base_commit: intent.manifest.base.clone(),
            head_commit: None,
            branch_commit: None,
            head_on_branch: None,
            commit_count: None,
            commits: Vec::new(),
            truncated: false,
            dirty: Some(false),
            commit_view: Some(crate::delegation::CommitViewRecord {
                proof_root: self.path.to_string_lossy().into(),
                binding: intent.binding.clone(),
                intent_digest: intent.proof_digest()?,
            }),
        })
    }
}
pub(super) fn report_view(workspace: &WorkspaceRecord) -> Result<WorkspaceRecord, String> {
    let reference = workspace.commit_view.as_ref()
        .ok_or_else(|| "workspace_commit_view_audit_required: conserver cette vue ; le rapport exige son manifeste privé".to_owned())?;
    let proofs = ViewProofs::open(Path::new(&reference.proof_root))?;
    let intent = proofs.load(&reference.binding.key()?)?;
    if intent.binding != reference.binding
        || intent.proof_digest()? != reference.intent_digest
        || intent.source != Path::new(&workspace.toplevel)
        || intent.destination()? != Path::new(&workspace.path)
        || intent.manifest.base != workspace.base_commit
        || !workspace.branch.is_empty()
    {
        return Err(refusal("proof_binding_mismatch"));
    }
    proofs.audited_workspace(&intent, WorkspaceState::Kept)
}

#[path = "workspace_commit_view_job.rs"]
mod job;
pub(crate) use job::{
    ExportFailure, JobPoll, ViewJob, ViewJobRequest, run_commit_view_plan, run_commit_view_export,
    run_commit_view_materialize,
};

#[cfg(test)]
#[path = "workspace_commit_view_tests.rs"]
mod tests;

pub(crate) use job::failure as view_export_failure;
