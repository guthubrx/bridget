//! Rechargement à chaud sûr de la façade `bridget mcp` (Spec 315).
//!
//! Sur Unix, surveille le lien symbolique actif (typiquement `$HOME/.local/bin/bridget`)
//! pendant les périodes calmes de `stdin` via `poll(2)`, sans aucun tampon d'avance
//! sur `stdin`. Dès qu'une nouvelle image valide est détectée et qu'aucun appel d'outil
//! n'est en vol, le processus se remplace en place via `execve` en conservant PID,
//! filiation d'identité, descripteurs stdio et état de session, puis émet
//! `notifications/tools/list_changed`.

use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs;
use std::io::{self, BufRead, Read};
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) const HOT_RELOAD_ENV: &str = "BRIDGET_MCP_HOT_RELOAD_V1";
const HOT_RELOAD_TOGGLE_ENV: &str = "BRIDGET_MCP_HOT_RELOAD";
const HOT_RELOAD_WATCH_LINK_ENV: &str = "BRIDGET_MCP_WATCH_LINK";
const HOT_RELOAD_POLL_MS_ENV: &str = "BRIDGET_MCP_HOT_RELOAD_POLL_MS";
const HOT_RELOAD_MIN_INTERVAL_MS_ENV: &str = "BRIDGET_MCP_HOT_RELOAD_MIN_INTERVAL_MS";
const T3_EXECUTABLE_ENV: &str = "T3CODE_BRIDGET_EXECUTABLE";

const HOT_RELOAD_STATE_VERSION: u8 = 1;
const DEFAULT_POLL_MS: u64 = 2_000;
const MIN_POLL_MS: u64 = 50;
const MAX_POLL_MS: u64 = 60_000;
const DEFAULT_MIN_INTERVAL_MS: u64 = 2_000;
const MAX_MIN_INTERVAL_MS: u64 = 60_000;
const MAX_REEXEC_PER_WINDOW: u32 = 3;
const REEXEC_WINDOW_SECS: i64 = 60;
const MAX_MCP_LINE_BYTES: usize = 16 * 1024 * 1024;

const INSTALL_ROOT_REL: &str = ".local/lib/bridget-communication";
const ACTIVE_LINK_REL: &str = ".local/bin/bridget";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileFingerprint {
    pub(crate) dev: u64,
    pub(crate) ino: u64,
    pub(crate) size: u64,
    pub(crate) mtime_sec: i64,
    pub(crate) mtime_nsec: i64,
}

impl FileFingerprint {
    fn from_metadata(meta: &fs::Metadata) -> Self {
        Self {
            dev: meta.dev(),
            ino: meta.ino(),
            size: meta.size(),
            mtime_sec: meta.mtime(),
            mtime_nsec: meta.mtime_nsec(),
        }
    }
}

/// État interne transmis entre deux images via `BRIDGET_MCP_HOT_RELOAD_V1`.
///
/// Volontairement sans `deny_unknown_fields` afin qu'un rollback vers une image
/// antérieure ignore proprement d'éventuels champs ajoutés ultérieurement sans
/// perdre l'état `initialized` de la session MCP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HotReloadState {
    pub(crate) version: u8,
    pub(crate) initialize_seen: bool,
    pub(crate) initialized: bool,
    #[serde(default)]
    pub(crate) watch_link: Option<PathBuf>,
    #[serde(default)]
    pub(crate) reexec_window_start: i64,
    #[serde(default)]
    pub(crate) reexec_count: u32,
    #[serde(default)]
    pub(crate) last_reexec_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HotReloadPlan {
    pub(crate) target_binary: PathBuf,
    pub(crate) target_fingerprint: FileFingerprint,
    pub(crate) state: HotReloadState,
}

pub(crate) enum ReadLineEvent {
    Line(String),
    IdleTimeout,
    Eof,
}

pub(crate) trait LineSource {
    fn read_event(&mut self, timeout_ms: i32) -> io::Result<ReadLineEvent>;
    fn stdin_has_hangup_only(&self) -> bool {
        false
    }
}

pub(crate) struct BufReadSource<R: BufRead> {
    reader: R,
    buf: String,
}

impl<R: BufRead> BufReadSource<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
        }
    }
}

impl<R: BufRead> LineSource for BufReadSource<R> {
    fn read_event(&mut self, _timeout_ms: i32) -> io::Result<ReadLineEvent> {
        self.buf.clear();
        if self.reader.read_line(&mut self.buf)? == 0 {
            return Ok(ReadLineEvent::Eof);
        }
        Ok(ReadLineEvent::Line(std::mem::take(&mut self.buf)))
    }
}

/// Lecteur de lignes sans tampon d'avance sur descripteur Unix.
///
/// Compromis assumé (article XVIII) : un appel système `read(2)` par octet sur la
/// ligne courante (`O(n)` octets par message JSON-RPC, typiquement quelques Ko,
/// borné à 16 Mio). Ce coût linéaire reste négligeable devant la latence réseau/LLM
/// et garantit qu'aucun octet du message suivant ne reste piégé en mémoire utilisateur
/// au moment d'un `execve` entre deux requêtes.
pub(crate) struct PollLineReader {
    fd: RawFd,
}

impl PollLineReader {
    pub(crate) fn stdin() -> Self {
        Self {
            fd: libc::STDIN_FILENO,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }

    fn poll_fd(&self, timeout_ms: i32) -> io::Result<(libc::c_int, i16)> {
        loop {
            let mut pfd = libc::pollfd {
                fd: self.fd,
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: `pfd` est une structure `libc::pollfd` valide sur la pile et nfds vaut 1.
            let rc = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
            if rc < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }
            return Ok((rc, pfd.revents));
        }
    }

    fn read_one_byte(&self) -> io::Result<Option<u8>> {
        let mut byte = 0u8;
        loop {
            // SAFETY: `byte` est un tampon valide d'un octet et `self.fd` est le descripteur
            // d'entrée possédé/emprunté pour toute la durée de lecture.
            let n = unsafe { libc::read(self.fd, (&mut byte as *mut u8).cast(), 1) };
            if n < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }
            if n == 0 {
                return Ok(None);
            }
            return Ok(Some(byte));
        }
    }
}

impl LineSource for PollLineReader {
    fn read_event(&mut self, timeout_ms: i32) -> io::Result<ReadLineEvent> {
        let (rc, revents) = self.poll_fd(timeout_ms)?;
        if rc == 0 {
            return Ok(ReadLineEvent::IdleTimeout);
        }
        if (revents & libc::POLLNVAL) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "descripteur stdin invalide",
            ));
        }
        if (revents & libc::POLLIN) == 0 && (revents & (libc::POLLHUP | libc::POLLERR)) != 0 {
            return Ok(ReadLineEvent::Eof);
        }

        let mut bytes = Vec::with_capacity(256);
        loop {
            match self.read_one_byte()? {
                None => {
                    if bytes.is_empty() {
                        return Ok(ReadLineEvent::Eof);
                    }
                    break;
                }
                Some(b) => {
                    bytes.push(b);
                    if b == b'\n' {
                        break;
                    }
                    if bytes.len() > MAX_MCP_LINE_BYTES {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "ligne MCP dépassant la taille maximale autorisée",
                        ));
                    }
                }
            }
        }
        let line = String::from_utf8(bytes)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(ReadLineEvent::Line(line))
    }

    fn stdin_has_hangup_only(&self) -> bool {
        match self.poll_fd(0) {
            Ok((rc, revents)) if rc > 0 => {
                if (revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL)) == 0 {
                    return false;
                }
                if (revents & libc::POLLIN) == 0 {
                    return true;
                }
                // Sur macOS/BSD, la fermeture d'un tube vide lève `POLLIN | POLLHUP`.
                // `FIONREAD == 0` confirme qu'aucun octet lisible ne subsiste avant EOF.
                let mut available: libc::c_int = 0;
                // SAFETY: `FIONREAD` écrit un `c_int` dans `available`, pointeur valide sur la pile.
                let ioctl_rc = unsafe { libc::ioctl(self.fd, libc::FIONREAD, &mut available) };
                ioctl_rc == 0 && available == 0
            }
            _ => false,
        }
    }
}

/// Lit puis supprime immédiatement l'état de passation interne `BRIDGET_MCP_HOT_RELOAD_V1`.
///
/// Doit être appelé au tout début de `run_stdio()`, avant toute création de thread.
/// Si la variable est présente mais illisible ou incompatible, renvoie une erreur
/// explicite afin que le client MCP redémarre proprement la session au lieu de
/// rester bloqué sur un état dégradé (`-32002`).
pub(crate) fn take_handoff_state() -> io::Result<Option<HotReloadState>> {
    let Ok(raw) = std::env::var(HOT_RELOAD_ENV) else {
        return Ok(None);
    };
    // SAFETY: appelé en mono-thread au tout début de `run_stdio()`, après
    // `cli::run()` (qui ne lance aucun thread avant `cmd_mcp()`) et avant tout
    // `spawn`, pour empêcher la fuite de cette variable interne vers des sous-processus.
    unsafe { std::env::remove_var(HOT_RELOAD_ENV) };
    parse_handoff_state(&raw).map(Some)
}

fn parse_handoff_state(raw: &str) -> io::Result<HotReloadState> {
    let state: HotReloadState = serde_json::from_str(raw).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("état hot-reload MCP illisible ({err})"),
        )
    })?;
    if state.version != HOT_RELOAD_STATE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "version d'état hot-reload MCP non prise en charge ({})",
                state.version
            ),
        ));
    }
    Ok(state)
}

fn current_euid() -> u32 {
    // SAFETY: `libc::geteuid` ne prend aucun pointeur et ne peut échouer.
    unsafe { libc::geteuid() }
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn hot_reload_disabled_value(raw: &str) -> bool {
    let trimmed = raw.trim();
    trimmed == "0" || trimmed.eq_ignore_ascii_case("false") || trimmed.eq_ignore_ascii_case("off")
}

fn parse_bounded_ms(raw: Option<&str>, default_ms: u64, min_ms: u64, max_ms: u64) -> u64 {
    raw.and_then(|value| value.trim().parse::<u64>().ok())
        .map(|value| value.clamp(min_ms, max_ms))
        .unwrap_or(default_ms)
}

#[derive(Debug, Default)]
struct WatchEnvInputs {
    disabled: bool,
    explicit_watch_link: Option<PathBuf>,
    t3_executable: Option<PathBuf>,
    home: Option<PathBuf>,
    argv0: Option<PathBuf>,
    current_exe: Option<PathBuf>,
    handoff_watch_link: Option<PathBuf>,
    poll_ms: u64,
    min_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedWatch {
    watch_link: PathBuf,
    managed_root: Option<PathBuf>,
}

fn resolve_absolute_no_follow(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else if path.components().count() > 1 {
        std::env::current_dir().ok().map(|cwd| cwd.join(path))
    } else {
        None
    }
}

fn is_symlink_path(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

fn resolve_watch_target(inputs: &WatchEnvInputs) -> Option<ResolvedWatch> {
    if inputs.disabled {
        return None;
    }
    if let Some(link) = inputs
        .explicit_watch_link
        .as_ref()
        .filter(|p| !p.as_os_str().is_empty())
    {
        let resolved = resolve_absolute_no_follow(link)?;
        return Some(ResolvedWatch {
            watch_link: resolved,
            managed_root: None,
        });
    }

    let managed_root = inputs
        .home
        .as_ref()
        .map(|home| home.join(INSTALL_ROOT_REL))
        .and_then(|root| fs::canonicalize(root).ok());
    let default_active_link = inputs.home.as_ref().map(|home| home.join(ACTIVE_LINK_REL));

    // Si une passation précédente connaissait déjà le lien surveillé, on le conserve
    // en gardant le verrou `managed_root` s'il s'agit du lien installé canonique.
    if let Some(link) = inputs
        .handoff_watch_link
        .as_ref()
        .filter(|p| !p.as_os_str().is_empty())
    {
        let keep_managed_root = default_active_link
            .as_ref()
            .is_some_and(|active| active == link);
        return Some(ResolvedWatch {
            watch_link: link.clone(),
            managed_root: if keep_managed_root {
                managed_root
            } else {
                None
            },
        });
    }

    // Si T3 pointe explicitement vers un exécutable hors arborescence gérée qui n'est
    // pas un lien symbolique (ex. binaire de test dédié), on ne bascule pas vers le lien global.
    if let Some(t3_exec) = inputs
        .t3_executable
        .as_ref()
        .filter(|p| !p.as_os_str().is_empty())
        && let Some(abs_t3) = resolve_absolute_no_follow(t3_exec)
    {
        if is_symlink_path(&abs_t3) {
            let keep_managed_root = default_active_link
                .as_ref()
                .is_some_and(|active| active == &abs_t3);
            return Some(ResolvedWatch {
                watch_link: abs_t3,
                managed_root: if keep_managed_root {
                    managed_root
                } else {
                    None
                },
            });
        }
        let in_managed = managed_root.as_ref().is_some_and(|root| {
            fs::canonicalize(&abs_t3)
                .ok()
                .is_some_and(|canon| canon.starts_with(root))
        });
        if !in_managed {
            return None;
        }
    }

    // Cas A : invoqué directement via un lien symbolique (ex. ~/.local/bin/bridget).
    if let Some(argv0) = inputs.argv0.as_ref()
        && let Some(abs_argv0) = resolve_absolute_no_follow(argv0)
        && is_symlink_path(&abs_argv0)
    {
        let keep_managed_root = default_active_link
            .as_ref()
            .is_some_and(|active| active == &abs_argv0);
        return Some(ResolvedWatch {
            watch_link: abs_argv0,
            managed_root: if keep_managed_root {
                managed_root
            } else {
                None
            },
        });
    }

    // Cas B : le lanceur (ex. T3 Code) a résolu le lien avant `exec` ; `current_exe`
    // réside dans `~/.local/lib/bridget-communication/<build>/bin/bridget`.
    if let (Some(root), Some(active_link), Some(exe)) = (
        managed_root,
        default_active_link,
        inputs
            .current_exe
            .as_ref()
            .and_then(|p| fs::canonicalize(p).ok()),
    ) && exe.starts_with(&root)
    {
        return Some(ResolvedWatch {
            watch_link: active_link,
            managed_root: Some(root),
        });
    }

    None
}

fn validate_owner_and_mode_dir(dir: &Path, euid: u32) -> Result<(), &'static str> {
    let meta = fs::metadata(dir).map_err(|_| "parent_inaccessible")?;
    if !meta.is_dir() {
        return Err("parent_not_dir");
    }
    if meta.uid() != euid {
        return Err("parent_wrong_owner");
    }
    if (meta.mode() & 0o022) != 0 {
        return Err("parent_writable_by_group_or_other");
    }
    Ok(())
}

fn has_native_executable_magic(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_err() {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        matches!(
            magic,
            [0xFE, 0xED, 0xFA, 0xCE]
                | [0xFE, 0xED, 0xFA, 0xCF]
                | [0xCE, 0xFA, 0xED, 0xFE]
                | [0xCF, 0xFA, 0xED, 0xFE]
                | [0xCA, 0xFE, 0xBA, 0xBE]
                | [0xBE, 0xBA, 0xFE, 0xCA]
        )
    }
    #[cfg(target_os = "linux")]
    {
        magic == [0x7F, b'E', b'L', b'F']
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        !magic.iter().all(|&b| b == 0)
    }
}

fn validate_target_binary(
    watch_link: &Path,
    managed_root: Option<&Path>,
    euid: u32,
) -> Result<(PathBuf, FileFingerprint), &'static str> {
    let link_parent = watch_link.parent().ok_or("watch_link_no_parent")?;
    validate_owner_and_mode_dir(link_parent, euid)?;

    let link_meta = fs::symlink_metadata(watch_link).map_err(|_| "watch_link_missing")?;
    if link_meta.uid() != euid {
        return Err("watch_link_wrong_owner");
    }

    let canonical = fs::canonicalize(watch_link).map_err(|_| "target_unresolvable")?;
    if let Some(root) = managed_root
        && !canonical.starts_with(root)
    {
        return Err("target_outside_managed_root");
    }

    let target_parent = canonical.parent().ok_or("target_no_parent")?;
    validate_owner_and_mode_dir(target_parent, euid)?;

    let target_meta = fs::symlink_metadata(&canonical).map_err(|_| "target_stat_failed")?;
    if !target_meta.is_file() {
        return Err("target_not_regular_file");
    }
    if target_meta.uid() != euid {
        return Err("target_wrong_owner");
    }
    if (target_meta.mode() & 0o022) != 0 {
        return Err("target_writable_by_group_or_other");
    }
    if (target_meta.mode() & 0o100) == 0 {
        return Err("target_not_executable");
    }
    if !has_native_executable_magic(&canonical) {
        return Err("target_not_native_executable");
    }

    Ok((canonical, FileFingerprint::from_metadata(&target_meta)))
}

pub(crate) struct HotReloadController {
    enabled: bool,
    watch_link: Option<PathBuf>,
    managed_root: Option<PathBuf>,
    current_fingerprint: Option<FileFingerprint>,
    poll_interval: Duration,
    min_interval_ms: i64,
    reexec_window_start: i64,
    reexec_count: u32,
    last_reexec_at_ms: i64,
    last_check_at: Option<Instant>,
    failed_fingerprint: Option<FileFingerprint>,
    last_rejected_reason: Option<&'static str>,
    loop_guard_warned: bool,
}

impl HotReloadController {
    pub(crate) fn from_process_env(handoff: Option<&HotReloadState>) -> Self {
        let disabled = std::env::var(HOT_RELOAD_TOGGLE_ENV)
            .ok()
            .is_some_and(|v| hot_reload_disabled_value(&v));
        let explicit_watch_link = std::env::var_os(HOT_RELOAD_WATCH_LINK_ENV).map(PathBuf::from);
        let t3_executable = std::env::var_os(T3_EXECUTABLE_ENV).map(PathBuf::from);
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let argv0 = std::env::args_os().next().map(PathBuf::from);
        let current_exe = std::env::current_exe().ok();
        let poll_ms = parse_bounded_ms(
            std::env::var(HOT_RELOAD_POLL_MS_ENV).ok().as_deref(),
            DEFAULT_POLL_MS,
            MIN_POLL_MS,
            MAX_POLL_MS,
        );
        let min_interval_ms = parse_bounded_ms(
            std::env::var(HOT_RELOAD_MIN_INTERVAL_MS_ENV)
                .ok()
                .as_deref(),
            DEFAULT_MIN_INTERVAL_MS,
            0,
            MAX_MIN_INTERVAL_MS,
        );

        let inputs = WatchEnvInputs {
            disabled,
            explicit_watch_link,
            t3_executable,
            home,
            argv0,
            current_exe: current_exe.clone(),
            handoff_watch_link: handoff.and_then(|s| s.watch_link.clone()),
            poll_ms,
            min_interval_ms,
        };
        let current_fingerprint = current_exe
            .as_deref()
            .and_then(|exe| fs::metadata(exe).ok())
            .map(|meta| FileFingerprint::from_metadata(&meta));

        Self::from_inputs(inputs, current_fingerprint, handoff)
    }

    fn from_inputs(
        inputs: WatchEnvInputs,
        current_fingerprint: Option<FileFingerprint>,
        handoff: Option<&HotReloadState>,
    ) -> Self {
        let resolved = resolve_watch_target(&inputs);
        let enabled = resolved.is_some() && current_fingerprint.is_some();
        Self {
            enabled,
            watch_link: resolved.as_ref().map(|r| r.watch_link.clone()),
            managed_root: resolved.and_then(|r| r.managed_root),
            current_fingerprint,
            poll_interval: Duration::from_millis(inputs.poll_ms),
            min_interval_ms: inputs.min_interval_ms as i64,
            reexec_window_start: handoff.map_or(0, |s| s.reexec_window_start),
            reexec_count: handoff.map_or(0, |s| s.reexec_count),
            last_reexec_at_ms: handoff.map_or(0, |s| s.last_reexec_at_ms),
            last_check_at: None,
            failed_fingerprint: None,
            last_rejected_reason: None,
            loop_guard_warned: false,
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn poll_timeout_ms(&self) -> i32 {
        if !self.enabled {
            return -1;
        }
        self.poll_interval.as_millis().min(i32::MAX as u128) as i32
    }

    pub(crate) fn check_due(&self) -> bool {
        self.enabled
            && self
                .last_check_at
                .is_none_or(|last| last.elapsed() >= self.poll_interval)
    }

    pub(crate) fn mark_failed(&mut self, fingerprint: FileFingerprint) {
        self.failed_fingerprint = Some(fingerprint);
    }

    pub(crate) fn check_reload(
        &mut self,
        session_initialize_seen: bool,
        session_initialized: bool,
    ) -> Option<HotReloadPlan> {
        self.check_reload_at(session_initialize_seen, session_initialized, now_epoch_ms())
    }

    fn check_reload_at(
        &mut self,
        session_initialize_seen: bool,
        session_initialized: bool,
        now_ms: i64,
    ) -> Option<HotReloadPlan> {
        if !self.enabled {
            return None;
        }
        // Ne jamais remplacer le processus au milieu du handshake initialize -> initialized.
        if session_initialize_seen && !session_initialized {
            return None;
        }
        self.last_check_at = Some(Instant::now());

        let watch_link = self.watch_link.as_ref()?;
        let current_fp = self.current_fingerprint?;
        let euid = current_euid();

        let (target_binary, target_fp) =
            match validate_target_binary(watch_link, self.managed_root.as_deref(), euid) {
                Ok(validated) => {
                    self.last_rejected_reason = None;
                    validated
                }
                Err(reason) => {
                    if self.last_rejected_reason != Some(reason) {
                        eprintln!(
                            "bridget mcp: hot-reload différé, cible {} refusée ({reason})",
                            watch_link.display()
                        );
                        self.last_rejected_reason = Some(reason);
                    }
                    return None;
                }
            };

        if target_fp == current_fp {
            self.failed_fingerprint = None;
            return None;
        }
        if self.failed_fingerprint == Some(target_fp) {
            return None;
        }

        if self.last_reexec_at_ms > 0
            && now_ms.saturating_sub(self.last_reexec_at_ms) < self.min_interval_ms
        {
            return None;
        }

        let now_secs = now_ms / 1_000;
        let (window_start, count) = if self.reexec_window_start == 0
            || now_secs.saturating_sub(self.reexec_window_start) >= REEXEC_WINDOW_SECS
        {
            (now_secs, 0)
        } else {
            (self.reexec_window_start, self.reexec_count)
        };

        if count >= MAX_REEXEC_PER_WINDOW {
            if !self.loop_guard_warned {
                eprintln!(
                    "bridget mcp: hot-reload temporairement suspendu (plafond de {MAX_REEXEC_PER_WINDOW} bascules par {REEXEC_WINDOW_SECS}s atteint)"
                );
                self.loop_guard_warned = true;
            }
            return None;
        }
        self.loop_guard_warned = false;

        let state = HotReloadState {
            version: HOT_RELOAD_STATE_VERSION,
            initialize_seen: session_initialize_seen,
            initialized: session_initialized,
            watch_link: Some(watch_link.clone()),
            reexec_window_start: window_start,
            reexec_count: count + 1,
            last_reexec_at_ms: now_ms,
        };

        Some(HotReloadPlan {
            target_binary,
            target_fingerprint: target_fp,
            state,
        })
    }

    /// Remplace le processus courant par la nouvelle image cible via `execve(2)` direct
    /// (jamais `execvp`, qui peut retomber sur `/bin/sh` sur `ENOEXEC`), sans muter
    /// l'environnement courant si `execve` échoue.
    pub(crate) fn exec_replace(plan: &HotReloadPlan) -> io::Error {
        let encoded = match serde_json::to_string(&plan.state) {
            Ok(encoded) => encoded,
            Err(err) => return io::Error::other(err),
        };
        let Ok(program) = CString::new(plan.target_binary.as_os_str().as_bytes()) else {
            return io::Error::new(io::ErrorKind::InvalidInput, "chemin cible avec octet nul");
        };
        let Ok(subcmd) = CString::new("mcp") else {
            return io::Error::new(io::ErrorKind::InvalidInput, "sous-commande invalide");
        };
        let argv = [program.as_ptr(), subcmd.as_ptr(), std::ptr::null()];

        let mut env_cstrings: Vec<CString> = Vec::new();
        for (key, value) in std::env::vars_os() {
            if key == HOT_RELOAD_ENV {
                continue;
            }
            let mut entry = Vec::with_capacity(key.len() + 1 + value.len());
            entry.extend_from_slice(key.as_bytes());
            entry.push(b'=');
            entry.extend_from_slice(value.as_bytes());
            if let Ok(cstr) = CString::new(entry) {
                env_cstrings.push(cstr);
            }
        }
        let Ok(handoff_entry) = CString::new(format!("{HOT_RELOAD_ENV}={encoded}")) else {
            return io::Error::new(
                io::ErrorKind::InvalidInput,
                "état hot-reload avec octet nul",
            );
        };
        env_cstrings.push(handoff_entry);

        let mut envp: Vec<*const libc::c_char> = env_cstrings.iter().map(|s| s.as_ptr()).collect();
        envp.push(std::ptr::null());

        // SAFETY: `program`, `argv` et `envp` sont terminés par `NULL` et adossés à des
        // `CString` vivants pour toute la durée de l'appel `execve(2)`. L'appelant garantit
        // que le processus est mono-thread (après jonction du `std::thread::scope`).
        unsafe {
            libc::execve(program.as_ptr(), argv.as_ptr(), envp.as_ptr());
        }
        io::Error::last_os_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn private_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bridget-hr-{label}-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..10]
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        dir
    }

    fn write_exec_file(path: &Path, content: &[u8], mode: u32) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).unwrap();
        }
        fs::write(path, content).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    #[cfg(target_os = "macos")]
    const TEST_EXEC_BYTES: &[u8] = &[0xCF, 0xFA, 0xED, 0xFE, 0, 0, 0, 0];
    #[cfg(not(target_os = "macos"))]
    const TEST_EXEC_BYTES: &[u8] = &[0x7F, b'E', b'L', b'F', 0, 0, 0, 0];

    #[test]
    fn spec315_validation_refuse_permissions_non_sures_et_hors_racine() {
        let root = private_temp_dir("sec");
        let managed = root.join("managed");
        let bin_dir = managed.join("v2/bin");
        let target = bin_dir.join("bridget");
        write_exec_file(&target, TEST_EXEC_BYTES, 0o755);

        let link_dir = root.join("bin");
        fs::create_dir_all(&link_dir).unwrap();
        fs::set_permissions(&link_dir, fs::Permissions::from_mode(0o700)).unwrap();
        let link = link_dir.join("bridget");
        symlink(&target, &link).unwrap();

        let euid = current_euid();
        let canonical_managed = fs::canonicalize(&managed).unwrap();
        assert!(validate_target_binary(&link, Some(&canonical_managed), euid).is_ok());

        // Binaire sans en-tête exécutable natif refusé.
        fs::write(&target, b"#!/bin/sh\n").unwrap();
        assert_eq!(
            validate_target_binary(&link, Some(&canonical_managed), euid).unwrap_err(),
            "target_not_native_executable"
        );
        fs::write(&target, TEST_EXEC_BYTES).unwrap();

        // Binaire world-writable refusé.
        fs::set_permissions(&target, fs::Permissions::from_mode(0o777)).unwrap();
        assert_eq!(
            validate_target_binary(&link, Some(&canonical_managed), euid).unwrap_err(),
            "target_writable_by_group_or_other"
        );

        // Binaire non exécutable refusé.
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            validate_target_binary(&link, Some(&canonical_managed), euid).unwrap_err(),
            "target_not_executable"
        );
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();

        // Répertoire parent inscriptible par le groupe refusé.
        fs::set_permissions(&bin_dir, fs::Permissions::from_mode(0o775)).unwrap();
        assert_eq!(
            validate_target_binary(&link, Some(&canonical_managed), euid).unwrap_err(),
            "parent_writable_by_group_or_other"
        );
        fs::set_permissions(&bin_dir, fs::Permissions::from_mode(0o755)).unwrap();

        // Cible hors racine gérée refusée.
        let outside = root.join("outside/bridget");
        write_exec_file(&outside, TEST_EXEC_BYTES, 0o755);
        let _ = fs::remove_file(&link);
        symlink(&outside, &link).unwrap();
        assert_eq!(
            validate_target_binary(&link, Some(&canonical_managed), euid).unwrap_err(),
            "target_outside_managed_root"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn spec315_resolution_lien_actif_couvre_symlink_managed_et_opt_out() {
        let home = private_temp_dir("home");
        let managed_bin = home.join(INSTALL_ROOT_REL).join("368-test/bin/bridget");
        write_exec_file(&managed_bin, b"#!/bin/sh\n", 0o755);
        let active_link = home.join(ACTIVE_LINK_REL);
        fs::create_dir_all(active_link.parent().unwrap()).unwrap();
        fs::set_permissions(
            active_link.parent().unwrap(),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        symlink(&managed_bin, &active_link).unwrap();

        // 1. Invoqué par chemin réel sous bridget-communication (cas T3 Code).
        let resolved = resolve_watch_target(&WatchEnvInputs {
            home: Some(home.clone()),
            argv0: Some(managed_bin.clone()),
            current_exe: Some(managed_bin.clone()),
            poll_ms: 2000,
            min_interval_ms: 2000,
            ..Default::default()
        })
        .expect("lien actif résolu depuis managed_root");
        assert_eq!(resolved.watch_link, active_link);
        assert!(resolved.managed_root.is_some());

        // 2. Opt-out explicite désactive la surveillance.
        assert!(
            resolve_watch_target(&WatchEnvInputs {
                disabled: true,
                home: Some(home.clone()),
                argv0: Some(active_link.clone()),
                current_exe: Some(managed_bin.clone()),
                ..Default::default()
            })
            .is_none()
        );

        // 3. Binaire dev hors bridget-communication et sans symlink : désactivé.
        let dev_bin = home.join("repo/target/debug/bridget");
        write_exec_file(&dev_bin, b"#!/bin/sh\n", 0o755);
        assert!(
            resolve_watch_target(&WatchEnvInputs {
                home: Some(home.clone()),
                argv0: Some(dev_bin.clone()),
                current_exe: Some(dev_bin),
                ..Default::default()
            })
            .is_none()
        );

        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn spec315_anti_boucle_et_handshake_proteges() {
        let root = private_temp_dir("loop");
        let v1 = root.join("v1/bridget");
        let v2 = root.join("v2/bridget");
        write_exec_file(&v1, TEST_EXEC_BYTES, 0o755);
        write_exec_file(&v2, TEST_EXEC_BYTES, 0o755);
        let link = root.join("active/bridget");
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        fs::set_permissions(link.parent().unwrap(), fs::Permissions::from_mode(0o700)).unwrap();
        symlink(&v2, &link).unwrap();

        let v1_fp = FileFingerprint::from_metadata(&fs::metadata(&v1).unwrap());
        let mut controller = HotReloadController::from_inputs(
            WatchEnvInputs {
                explicit_watch_link: Some(link.clone()),
                poll_ms: 100,
                min_interval_ms: 2_000,
                ..Default::default()
            },
            Some(v1_fp),
            Some(&HotReloadState {
                version: 1,
                initialize_seen: true,
                initialized: true,
                watch_link: Some(link.clone()),
                reexec_window_start: 1_000,
                reexec_count: 2,
                last_reexec_at_ms: 1_010_000,
            }),
        );

        // Bloqué pendant le handshake intermédiaire.
        assert!(controller.check_reload_at(true, false, 1_020_000).is_none());
        // Bloqué sous min_interval (1_011_000 - 1_010_000 < 2_000).
        assert!(controller.check_reload_at(true, true, 1_011_000).is_none());
        // Autorisé une 3e fois dans la fenêtre.
        let plan = controller
            .check_reload_at(true, true, 1_015_000)
            .expect("3e bascule autorisée");
        assert_eq!(plan.state.reexec_count, 3);

        // Au 4e essai dans la même fenêtre de 60s : suspendu.
        let mut throttled = HotReloadController::from_inputs(
            WatchEnvInputs {
                explicit_watch_link: Some(link.clone()),
                poll_ms: 100,
                min_interval_ms: 2_000,
                ..Default::default()
            },
            Some(v1_fp),
            Some(&plan.state),
        );
        assert!(throttled.check_reload_at(true, true, 1_020_000).is_none());
        // Après expiration de la fenêtre de 60s : autorisé avec compteur réinitialisé à 1.
        let reset = throttled
            .check_reload_at(true, true, 1_065_000)
            .expect("fenêtre réinitialisée");
        assert_eq!(reset.state.reexec_count, 1);

        // Cible marquée en échec ignorée tant que son empreinte ne change pas.
        controller.mark_failed(plan.target_fingerprint);
        assert!(controller.check_reload_at(true, true, 1_025_000).is_none());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn spec315_poll_line_reader_ne_prelit_aucun_octet_de_la_ligne_suivante() {
        let (mut reader_end, mut writer_end) = os_pipe_pair();
        writer_end.write_all(b"{\"id\":1}\n{\"id\":2}\n").unwrap();
        writer_end.flush().unwrap();

        let mut poll_reader = PollLineReader::from_raw_fd(reader_end.as_raw_fd());
        match poll_reader.read_event(100).unwrap() {
            ReadLineEvent::Line(line) => assert_eq!(line, "{\"id\":1}\n"),
            _ => panic!("ligne 1 attendue"),
        }

        // Vérifier que la ligne 2 est restée intégralement dans le tube noyau
        // en la lisant directement depuis le descripteur brut (simulant le processus après execv).
        let mut remaining = [0u8; 9];
        std::io::Read::read_exact(&mut reader_end, &mut remaining).unwrap();
        assert_eq!(&remaining, b"{\"id\":2}\n");

        // Tube vide ouvert -> IdleTimeout.
        assert!(matches!(
            poll_reader.read_event(60).unwrap(),
            ReadLineEvent::IdleTimeout
        ));

        // Fermeture côté écriture -> hangup puis Eof.
        drop(writer_end);
        assert!(poll_reader.stdin_has_hangup_only());
        assert!(matches!(
            poll_reader.read_event(60).unwrap(),
            ReadLineEvent::Eof
        ));
    }

    #[test]
    fn spec315_take_handoff_state_lit_et_supprime_la_variable() {
        let _guard = ENV_LOCK.lock().unwrap();
        let state = HotReloadState {
            version: 1,
            initialize_seen: true,
            initialized: true,
            watch_link: Some(PathBuf::from("/tmp/example-link")),
            reexec_window_start: 100,
            reexec_count: 1,
            last_reexec_at_ms: 100_000,
        };
        let encoded = serde_json::to_string(&state).unwrap();
        // SAFETY: test sérialisé par `ENV_LOCK`.
        unsafe { std::env::set_var(HOT_RELOAD_ENV, encoded) };
        let loaded = take_handoff_state().unwrap().expect("état lu");
        assert_eq!(loaded, state);
        assert!(
            std::env::var_os(HOT_RELOAD_ENV).is_none(),
            "la variable interne doit être retirée immédiatement"
        );
    }

    #[test]
    fn spec315_parse_handoff_tolere_champs_futurs_et_refuse_version_inconnue() {
        let with_future_field =
            r#"{"version":1,"initialize_seen":true,"initialized":true,"future_flag":42}"#;
        let parsed = parse_handoff_state(with_future_field).expect("champs futurs tolérés");
        assert!(parsed.initialize_seen);
        assert!(parsed.initialized);
        assert_eq!(parsed.reexec_count, 0);

        let wrong_version = r#"{"version":2,"initialize_seen":true,"initialized":true}"#;
        assert!(
            parse_handoff_state(wrong_version).is_err(),
            "version inconnue doit échouer explicitement"
        );

        assert!(
            parse_handoff_state("not-json").is_err(),
            "JSON corrompu doit échouer explicitement"
        );
    }

    fn os_pipe_pair() -> (fs::File, fs::File) {
        use std::os::fd::FromRawFd;
        let mut fds = [0 as libc::c_int; 2];
        // SAFETY: `fds` est un tableau valide de 2 entiers.
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        // SAFETY: les deux descripteurs viennent d'être créés par `pipe(2)`.
        unsafe { (fs::File::from_raw_fd(fds[0]), fs::File::from_raw_fd(fds[1])) }
    }
}
