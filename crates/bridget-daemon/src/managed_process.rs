//! Bootstrap et marqueurs durables des équipiers supervisés par le daemon.

use bridget_transport::fsutil::write_private_file_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

const RELEASE_FD: RawFd = 100;
const STATUS_FD: RawFd = 101;
const FIRST_AUXILIARY_FD: RawFd = 102;
const RELEASE_BYTE: u8 = b'R';
const ABANDONED_EXIT_CODE: libc::c_int = 125;
pub const MANAGED_STATUS_FD_ENV: &str = "BRIDGET_MANAGED_STATUS_FD";
pub const MANAGED_INSTANCE_ID_ENV: &str = "BRIDGET_MANAGED_INSTANCE_ID";
pub const MANAGED_COMMAND_ID_ENV: &str = "BRIDGET_MANAGED_COMMAND_ID";
pub const MANAGED_GENERATION_ENV: &str = "BRIDGET_MANAGED_GENERATION";

// Le drainage distant ne récolte qu'après destruction des propriétaires Child.
#[cfg(target_os = "linux")]
pub(crate) mod remote_drain {
    use super::*;
    use std::collections::BTreeSet;
    use std::os::fd::{FromRawFd, OwnedFd};
    use std::os::unix::fs::MetadataExt;

    pub const CADENCE: Duration = Duration::from_millis(100);
    const MAX_CHILDREN: usize = 1024;

    pub fn arm() -> io::Result<()> {
        if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // Refuser avant fournisseur les noyaux dépourvus de pidfd.
        let fd = pidfd(std::process::id())?;
        let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
        let result = unsafe {
            libc::waitid(
                libc::P_PIDFD,
                fd.as_raw_fd() as u32,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result != 0 && io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD) {
            return Ok(());
        }
        Err(io::Error::other("remote_drain_pidfd_unavailable"))
    }

    fn pidfd(pid: u32) -> io::Result<OwnedFd> {
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) })
    }

    fn children() -> io::Result<BTreeSet<u32>> {
        let mut children = BTreeSet::new();
        for task in fs::read_dir("/proc/self/task")? {
            let path = task?.path().join("children");
            let text = match fs::read_to_string(path) {
                Ok(text) => text,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            for raw in text.split_whitespace() {
                let pid = raw
                    .parse()
                    .map_err(|_| io::Error::other("invalid_child_pid"))?;
                children.insert(pid);
                if children.len() >= MAX_CHILDREN {
                    return Ok(children);
                }
            }
        }
        Ok(children)
    }

    pub(super) fn checked_pidfd(pid: u32) -> io::Result<OwnedFd> {
        let birth = process_birth(pid)?;
        let fd = pidfd(pid)?;
        let directory = format!("/proc/{pid}");
        let status = fs::read_to_string(format!("{directory}/status"))?;
        let parent = status
            .lines()
            .find_map(|line| line.strip_prefix("PPid:"))
            .and_then(|raw| raw.trim().parse::<u32>().ok());
        // La filiation et le pidfd autorisent la récolte de notre enfant,
        // même s'il a changé d'UID. Le droit de signal reste vérifié séparément.
        if parent != Some(std::process::id()) || process_birth(pid)? != birth {
            return Err(io::Error::other("remote_drain_identity_changed"));
        }
        Ok(fd)
    }

    /// Aucun wait : le transport peut encore être propriétaire de ces Child.
    pub fn signal_pass() -> io::Result<()> {
        let mut failure = None;
        for pid in children()? {
            let fd = match checked_pidfd(pid) {
                Ok(fd) => fd,
                Err(error) if matches!(error.raw_os_error(), Some(libc::ENOENT | libc::ESRCH)) => {
                    continue;
                }
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            };
            // Conserver le plafond historique de signalisation. Un UID
            // différent interdit le signal, pas la récolte après les Child.
            match fs::metadata(format!("/proc/{pid}")) {
                Ok(metadata) if metadata.uid() == unsafe { libc::geteuid() } => {}
                Ok(_) => {
                    failure = Some(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "remote_drain_signal_uid_mismatch",
                    ));
                    continue;
                }
                Err(error) if matches!(error.raw_os_error(), Some(libc::ENOENT | libc::ESRCH)) => continue,
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            }
            let result = unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    fd.as_raw_fd(),
                    libc::SIGTERM,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                )
            };
            if result != 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                failure = Some(io::Error::last_os_error());
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Appeler seulement après la barrière de destruction des propriétaires Child.
    pub fn reap_pass_after_owners() -> io::Result<bool> {
        for pid in children()? {
            let fd = match checked_pidfd(pid) {
                Ok(fd) => fd,
                Err(error) if matches!(error.raw_os_error(), Some(libc::ENOENT | libc::ESRCH)) => {
                    continue;
                }
                Err(error) => return Err(error),
            };
            let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
            let rc = unsafe {
                libc::waitid(
                    libc::P_PIDFD,
                    fd.as_raw_fd() as u32,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG,
                )
            };
            if rc != 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ECHILD) {
                return Err(io::Error::last_os_error());
            }
        }
        let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
        let rc = unsafe {
            libc::waitid(
                libc::P_ALL,
                0,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if rc == 0 {
            return Ok(false);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ECHILD) {
            Ok(true)
        } else {
            Err(error)
        }
    }
}

/// Identité immuable d'une génération supervisée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedIdentity {
    pub instance_id: String,
    pub command_id: String,
    pub generation: u64,
}

/// Preuve que le bootstrap possède son groupe, avant toute exécution du wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapReady {
    pub pid: u32,
    pub pgid: u32,
    pub birth: u64,
    pub instance_id: String,
    pub command_id: String,
    pub generation: u64,
}

/// Événements du canal de statut partagé avec le futur hook wrapper T906.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ManagedStatus {
    BootstrapReady(BootstrapReady),
    StartupFailed {
        kind: String,
        reason: String,
        instance_id: String,
        command_id: String,
        generation: u64,
    },
}

/// Marqueur durable utilisé par la réconciliation après crash du daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedMarker {
    pub pgid: u32,
    pub birth: u64,
    pub instance_id: String,
    pub command_id: String,
    pub generation: u64,
}

impl From<&BootstrapReady> for ManagedMarker {
    fn from(ready: &BootstrapReady) -> Self {
        Self {
            pgid: ready.pgid,
            birth: ready.birth,
            instance_id: ready.instance_id.clone(),
            command_id: ready.command_id.clone(),
            generation: ready.generation,
        }
    }
}

#[derive(Debug)]
pub enum ManagedProcessError {
    Io(io::Error),
    InvalidArgument(String),
    InvalidStatus(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedStopResult {
    Stopped,
    StoppedForced { survivors_killed: usize },
    Timeout,
}

impl fmt::Display for ManagedProcessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => write!(formatter, "erreur processus géré: {source}"),
            Self::InvalidArgument(reason) => write!(formatter, "bootstrap invalide: {reason}"),
            Self::InvalidStatus(reason) => write!(formatter, "statut bootstrap invalide: {reason}"),
        }
    }
}

impl std::error::Error for ManagedProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::InvalidArgument(_) | Self::InvalidStatus(_) => None,
        }
    }
}

impl From<io::Error> for ManagedProcessError {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}

/// Commande complète du bootstrap ; le wrapper n'est exécuté qu'après RELEASE.
#[derive(Debug, Clone)]
pub struct ManagedLaunch {
    pub bootstrap_executable: PathBuf,
    pub identity: ManagedIdentity,
    pub wrapper_executable: PathBuf,
    pub wrapper_args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, OsString>,
}

/// Extrémités détenues par le daemon pendant la phase de bootstrap.
struct ProcessChannels {
    child: Child,
    release: Option<UnixStream>,
    status: Option<BufReader<UnixStream>>,
}

/// Bootstrap lancé mais pas encore reconnu comme prêt.
pub struct ManagedChild {
    inner: ProcessChannels,
}

/// Groupe créé, mais marqueur durable pas encore écrit.
pub struct ReadyManagedChild {
    inner: ProcessChannels,
    ready: BootstrapReady,
}

/// Marqueur durable écrit ; seul cet état autorise l'octet RELEASE.
pub struct MarkedManagedChild {
    inner: ProcessChannels,
    marker: ManagedMarkerHandle,
}

/// Groupe supervisable après RELEASE ou après une tentative de RELEASE incertaine.
pub struct RunningManagedChild {
    inner: ProcessChannels,
    marker: ManagedMarkerHandle,
    // État transitoire du canal, distinct d'un EOF valide après démarrage.
    status_failed: bool,
}

/// macOS rend EINVAL pour SO_RCVTIMEO quand le pair a déjà fermé la socket.
/// La lecture suivante rend alors immédiatement les octets restants puis EOF :
/// ce n'est pas une panne du canal, et le refus doit rester « EOF avant ready ».
fn set_status_timeout(stream: &UnixStream, timeout: Duration) -> io::Result<()> {
    match stream.set_read_timeout(Some(timeout)) {
        Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(libc::EINVAL) => Ok(()),
        result => result,
    }
}

impl ManagedChild {
    pub fn set_ready_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        set_status_timeout(
            self.inner
                .status
                .as_mut()
                .expect("canal de statut présent avant le transfert")
                .get_mut(),
            timeout,
        )
    }

    /// Attend la preuve de création du groupe. Ce statut n'est jamais un succès.
    pub fn wait_ready(mut self) -> Result<ReadyManagedChild, ManagedProcessError> {
        let mut line = String::new();
        let read = self
            .inner
            .status
            .as_mut()
            .expect("canal de statut présent avant le transfert")
            .read_line(&mut line);
        let ready = match read {
            Ok(0) => Err(ManagedProcessError::InvalidStatus(
                "EOF avant BootstrapReady".to_string(),
            )),
            Ok(_) => match serde_json::from_str::<ManagedStatus>(line.trim_end()) {
                Ok(ManagedStatus::BootstrapReady(ready)) => Ok(ready),
                Ok(ManagedStatus::StartupFailed { .. }) => Err(ManagedProcessError::InvalidStatus(
                    "StartupFailed reçu avant BootstrapReady".to_string(),
                )),
                Err(error) => Err(ManagedProcessError::InvalidStatus(error.to_string())),
            },
            Err(error) => Err(error.into()),
        };
        match ready {
            Ok(ready) => Ok(ReadyManagedChild {
                inner: self.inner,
                ready,
            }),
            Err(error) => {
                self.inner.release.take();
                let _ = self.inner.child.wait();
                Err(error)
            }
        }
    }

    /// Variante annulable utilisée par `stop` pendant le bootstrap. Le poll
    /// court borne la prise en compte sans rendre le superviseur bloquant
    /// jusqu'à l'échéance complète du spawn.
    pub fn wait_ready_or_cancel(
        mut self,
        cancelled: impl Fn() -> bool,
        poll_interval: Duration,
    ) -> Result<Option<ReadyManagedChild>, ManagedProcessError> {
        let ready = self.read_ready_or_cancel(cancelled, poll_interval)?;
        Ok(ready.map(|ready| ReadyManagedChild {
            inner: self.inner,
            ready,
        }))
    }

    /// Conserve l'autorité sur le PID en cas d'échec. L'appelant doit attendre
    /// `try_wait` avant de déduire un arrêt ; EOF n'est jamais cette preuve.
    pub(crate) fn wait_ready_or_cancel_retained(
        mut self,
        cancelled: impl Fn() -> bool,
        poll_interval: Duration,
    ) -> Result<Option<ReadyManagedChild>, (ManagedProcessError, Child)> {
        match self.read_ready_or_cancel(cancelled, poll_interval) {
            Ok(ready) => Ok(ready.map(|ready| ReadyManagedChild {
                inner: self.inner,
                ready,
            })),
            Err(error) => {
                self.inner.release.take();
                self.inner.status.take();
                // Le résultat de cet arrêt ne remplace pas la preuve portée
                // par Child : même un timeout conserve le handle supervisable.
                let _ = terminate_bootstrap(&mut self.inner.child, poll_interval);
                Err((error, self.inner.child))
            }
        }
    }

    fn read_ready_or_cancel(
        &mut self,
        cancelled: impl Fn() -> bool,
        poll_interval: Duration,
    ) -> Result<Option<BootstrapReady>, ManagedProcessError> {
        set_status_timeout(
            self.inner
                .status
                .as_mut()
                .expect("canal de statut présent avant le transfert")
                .get_mut(),
            poll_interval,
        )?;
        loop {
            if cancelled() {
                self.inner.release.take();
                self.inner.status.take();
                terminate_bootstrap(&mut self.inner.child, poll_interval)?;
                return Ok(None);
            }
            let mut line = String::new();
            match self
                .inner
                .status
                .as_mut()
                .expect("canal de statut présent avant le transfert")
                .read_line(&mut line)
            {
                Ok(0) => {
                    return Err(ManagedProcessError::InvalidStatus(
                        "EOF avant BootstrapReady".to_string(),
                    ));
                }
                Ok(_) => {
                    let ready = match serde_json::from_str::<ManagedStatus>(line.trim_end()) {
                        Ok(ManagedStatus::BootstrapReady(ready)) => ready,
                        Ok(ManagedStatus::StartupFailed { .. }) => {
                            return Err(ManagedProcessError::InvalidStatus(
                                "StartupFailed reçu avant BootstrapReady".to_string(),
                            ));
                        }
                        Err(error) => {
                            return Err(ManagedProcessError::InvalidStatus(error.to_string()));
                        }
                    };
                    return Ok(Some(ready));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
}

impl ReadyManagedChild {
    /// Le canal est privé, mais sa déclaration ne suffit pas à posséder un PID.
    /// C19 vérifie aussi le Child réel, sa naissance et son groupe avant RELEASE.
    pub(crate) fn belongs_to(&self, expected: &ManagedIdentity) -> bool {
        let pid = self.inner.child.id();
        self.ready.pid == pid && self.ready.pgid == pid && self.ready.birth > 0
            && self.ready.instance_id == expected.instance_id
            && self.ready.command_id == expected.command_id
            && self.ready.generation == expected.generation
            && process_birth(pid).ok() == Some(self.ready.birth)
            && unsafe { libc::getpgid(pid as i32) } == pid as i32
    }
    pub fn ready(&self) -> &BootstrapReady {
        &self.ready
    }

    /// Écrit durablement le marqueur avant de rendre RELEASE accessible.
    pub fn persist_marker(
        self,
        store: &ManagedMarkerStore,
        name: &str,
    ) -> Result<MarkedManagedChild, ManagedProcessError> {
        match self.persist_marker_retained(store, name) {
            Ok(marked) => Ok(marked),
            Err((error, mut child)) => {
                let _ = child.wait();
                Err(error)
            }
        }
    }

    /// Variante worker : l'échec du store conserve le bootstrap abandonné
    /// pour superviser sa fin au lieu de perdre l'autorité sur le PID.
    pub(crate) fn persist_marker_retained(
        mut self,
        store: &ManagedMarkerStore,
        name: &str,
    ) -> Result<MarkedManagedChild, (ManagedProcessError, Child)> {
        match store.persist(name, &self.ready) {
            Ok(marker) => Ok(MarkedManagedChild {
                inner: self.inner,
                marker,
            }),
            Err(error) => {
                self.inner.release.take();
                self.inner.status.take();
                Err((error, self.inner.child))
            }
        }
    }

    /// Crash/abandon avant marqueur : EOF ferme le bootstrap sans exécuter.
    pub fn abandon(mut self) -> Child {
        self.inner.release.take();
        self.inner.status.take();
        self.inner.child
    }
}

impl MarkedManagedChild {
    /// Libère exactement une fois le bootstrap, après la preuve durable portée
    /// par le type. Aucun appelant ne peut obtenir ce type sans écriture.
    pub fn release(self) -> Result<RunningManagedChild, ManagedProcessError> {
        self.release_retained().map_err(|(error, _child)| error)
    }

    /// Une erreur d'écriture peut suivre l'envoi effectif de RELEASE. Conserver
    /// le processus et son marqueur sans assimiler EPIPE à une preuve d'arrêt.
    pub(crate) fn release_retained(
        mut self,
    ) -> Result<RunningManagedChild, (ManagedProcessError, RunningManagedChild)> {
        let result = (|| {
            let mut release = self.inner.release.take().ok_or_else(|| {
                ManagedProcessError::InvalidArgument("RELEASE déjà envoyé".to_string())
            })?;
            release.write_all(&[RELEASE_BYTE])?;
            release.flush()?;
            Ok::<(), ManagedProcessError>(())
        })();
        let mut child = RunningManagedChild {
            inner: self.inner,
            marker: self.marker,
            status_failed: false,
        };
        match result {
            Ok(()) => Ok(child),
            Err(error) => {
                child.fail_status();
                Err((error, child))
            }
        }
    }

    pub fn marker(&self) -> &ManagedMarkerHandle {
        &self.marker
    }

    /// Crash/abandon après marqueur et avant RELEASE : le marqueur persiste.
    pub fn abandon(mut self) -> (Child, ManagedMarkerHandle) {
        self.inner.release.take();
        self.inner.status.take();
        (self.inner.child, self.marker)
    }
}

impl RunningManagedChild {
    pub fn marker(&self) -> &ManagedMarkerHandle {
        &self.marker
    }

    pub fn child_mut(&mut self) -> &mut Child {
        &mut self.inner.child
    }

    pub fn status_reader(&mut self) -> &mut BufReader<UnixStream> {
        self.inner
            .status
            .as_mut()
            .expect("canal de statut déjà fermé")
    }

    pub fn close_status(&mut self) {
        self.inner.status.take();
    }

    pub fn status_is_open(&self) -> bool {
        self.inner.status.is_some()
    }

    /// Le superviseur appelle cette méthode à chaque tick ; `try_wait` utilise
    /// `waitpid(WNOHANG)` sur Unix et ne bloque donc jamais la boucle daemon.
    pub fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.inner.child.try_wait()
    }

    pub fn set_status_nonblocking(&mut self) -> io::Result<()> {
        let result = self
            .inner
            .status
            .as_mut()
            .expect("canal de statut déjà fermé")
            .get_mut()
            .set_nonblocking(true);
        if result.is_err() {
            self.fail_status();
        }
        result
    }

    fn fail_status(&mut self) {
        self.status_failed = true;
        self.inner.status.take();
    }

    /// Lit au plus un statut. Les messages sont des lignes JSON atomiques et
    /// petites ; `WouldBlock` signifie simplement qu'aucun événement n'est
    /// disponible à ce tick.
    pub fn try_status(&mut self) -> Result<Option<ManagedStatus>, ManagedProcessError> {
        if self.status_failed {
            return Err(ManagedProcessError::InvalidStatus(
                "canal de démarrage définitivement refusé".to_string(),
            ));
        }
        let Some(reader) = self.inner.status.as_mut() else {
            return Ok(None);
        };
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                self.inner.status.take();
                Ok(None)
            }
            Ok(_) => serde_json::from_str(line.trim_end())
                .map(Some)
                .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string())),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn remove_marker(&self) -> io::Result<()> {
        match fs::remove_file(self.marker.path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    /// Attend d'abord le chemin coopératif du wrapper, puis termine le groupe
    /// entier au SIGTERM si un descendant survit. Aucun SIGKILL n'est utilisé.
    pub fn stop_group(
        &mut self,
        cooperative_grace: Duration,
        forced_grace: Duration,
        poll_interval: Duration,
    ) -> Result<ManagedStopResult, ManagedProcessError> {
        let pgid = self.marker.marker().pgid;
        if self.wait_group_gone_reaping(pgid, cooperative_grace, poll_interval)? {
            self.remove_marker()?;
            return Ok(ManagedStopResult::Stopped);
        }
        let survivors_killed = group_member_count(pgid)?.max(1);
        signal_group(pgid, libc::SIGTERM)?;
        if self.wait_group_gone_reaping(pgid, forced_grace, poll_interval)? {
            self.remove_marker()?;
            return Ok(ManagedStopResult::StoppedForced { survivors_killed });
        }
        Ok(ManagedStopResult::Timeout)
    }

    /// Demande l'arrêt du groupe sans attendre. Cette phase peut être appelée
    /// pour tous les enfants au début de l'arrêt du daemon afin que les délais
    /// ne s'additionnent pas avec la taille de la flotte.
    pub fn request_group_termination(&mut self) -> Result<(), ManagedProcessError> {
        let pgid = self.marker.marker().pgid;
        let _ = self.inner.child.try_wait()?;
        if !group_exists(pgid)? {
            self.remove_marker()?;
            return Ok(());
        }
        signal_group(pgid, libc::SIGTERM)?;
        Ok(())
    }

    /// Récolte le leader et retire le marqueur seulement une fois le groupe
    /// effectivement absent. Un groupe résiduel reste marquable pour la
    /// réconciliation du prochain démarrage.
    pub fn reap_terminated_group(&mut self) -> Result<bool, ManagedProcessError> {
        let pgid = self.marker.marker().pgid;
        let _ = self.inner.child.try_wait()?;
        if group_exists(pgid)? {
            return Ok(false);
        }
        self.remove_marker()?;
        Ok(true)
    }

    fn wait_group_gone_reaping(
        &mut self,
        pgid: u32,
        timeout: Duration,
        poll_interval: Duration,
    ) -> io::Result<bool> {
        let deadline = Instant::now() + timeout;
        loop {
            // Récolter le leader évite qu'un zombie fasse croire à tort que
            // le groupe est encore vivant. Les descendants restent sondés
            // par kill(-pgid, 0), y compris lorsqu'ils ne sont pas enfants.
            let _ = self.inner.child.try_wait()?;
            if !group_exists(pgid)? {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            std::thread::sleep(poll_interval);
        }
    }
}

fn terminate_bootstrap(child: &mut Child, poll_interval: Duration) -> io::Result<()> {
    if child.try_wait()?.is_some() {
        return Ok(());
    }
    let pid = child.id() as libc::pid_t;
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };
    if result != 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error);
        }
    }
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        std::thread::sleep(poll_interval);
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "bootstrap encore vivant après SIGTERM",
    ))
}

pub fn group_exists(pgid: u32) -> io::Result<bool> {
    if pgid == 0 || pgid > libc::pid_t::MAX as u32 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "pgid invalide"));
    }
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), 0) };
    if result == 0 {
        return Ok(true);
    }
    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error),
    }
}

pub fn signal_group(pgid: u32, signal: libc::c_int) -> io::Result<()> {
    if !group_exists(pgid)? {
        return Ok(());
    }
    let result = unsafe { libc::killpg(pgid as libc::pid_t, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

pub fn wait_group_gone(pgid: u32, timeout: Duration, poll_interval: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        if !group_exists(pgid)? {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(poll_interval);
    }
}

#[cfg(target_os = "macos")]
fn group_member_count(pgid: u32) -> io::Result<usize> {
    const PROC_PGRP_ONLY: u32 = 2;
    let required = unsafe { libc::proc_listpids(PROC_PGRP_ONLY, pgid, std::ptr::null_mut(), 0) };
    if required < 0 {
        return Err(io::Error::last_os_error());
    }
    let capacity = (required as usize)
        .div_ceil(std::mem::size_of::<libc::pid_t>())
        .saturating_add(8);
    let mut pids = vec![0 as libc::pid_t; capacity];
    let written = unsafe {
        libc::proc_listpids(
            PROC_PGRP_ONLY,
            pgid,
            pids.as_mut_ptr().cast(),
            (pids.len() * std::mem::size_of::<libc::pid_t>()) as libc::c_int,
        )
    };
    if written < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(pids
        .into_iter()
        .take(written as usize / std::mem::size_of::<libc::pid_t>())
        .filter(|pid| *pid > 0)
        .count())
}

#[cfg(target_os = "linux")]
fn group_member_count(pgid: u32) -> io::Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir("/proc")? {
        let Ok(entry) = entry else { continue };
        if entry
            .file_name()
            .to_str()
            .is_none_or(|name| !name.bytes().all(|byte| byte.is_ascii_digit()))
        {
            continue;
        }
        let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        let Some((_, tail)) = stat.rsplit_once(')') else {
            continue;
        };
        let member_pgid = tail
            .split_whitespace()
            .nth(2)
            .and_then(|value| value.parse::<u32>().ok());
        if member_pgid == Some(pgid) {
            count += 1;
        }
    }
    Ok(count)
}

/// Lance le sous-mode de production avec deux canaux privés.
pub fn spawn_managed_bootstrap(
    launch: &ManagedLaunch,
) -> Result<ManagedChild, ManagedProcessError> {
    spawn_managed_bootstrap_with_stderr(launch, Stdio::null())
}

/// Variante de production : stderr est ouvert par le daemon avant tout spawn
/// et seulement hérité par le bootstrap puis le wrapper.
pub fn spawn_managed_bootstrap_with_stderr(
    launch: &ManagedLaunch,
    stderr: Stdio,
) -> Result<ManagedChild, ManagedProcessError> {
    validate_identity(&launch.identity)?;
    if launch.wrapper_executable.as_os_str().is_empty() {
        return Err(ManagedProcessError::InvalidArgument(
            "exécutable wrapper vide".to_string(),
        ));
    }
    let mut command = Command::new(&launch.bootstrap_executable);
    command
        .arg("managed-bootstrap")
        .arg("--instance-id")
        .arg(&launch.identity.instance_id)
        .arg("--command-id")
        .arg(&launch.identity.command_id)
        .arg("--generation")
        .arg(launch.identity.generation.to_string())
        .arg("--")
        .arg(&launch.wrapper_executable)
        .args(&launch.wrapper_args);
    command
        .current_dir(&launch.cwd)
        .env_clear()
        .envs(&launch.env);
    spawn_bootstrap_command(command, stderr)
}

/// Point d'entrée du sous-mode caché `managed-bootstrap`.
pub fn run_managed_bootstrap(args: &[String]) -> Result<(), ManagedProcessError> {
    let request = parse_bootstrap_args(args)?;
    run_bootstrap(request)
}

#[derive(Debug, Clone)]
struct BootstrapRequest {
    identity: ManagedIdentity,
    wrapper_executable: PathBuf,
    wrapper_args: Vec<String>,
}

fn parse_bootstrap_args(args: &[String]) -> Result<BootstrapRequest, ManagedProcessError> {
    let separator = args
        .iter()
        .position(|argument| argument == "--")
        .ok_or_else(|| ManagedProcessError::InvalidArgument("séparateur -- absent".to_string()))?;
    let options = &args[..separator];
    let wrapper = &args[separator + 1..];
    if wrapper.is_empty() {
        return Err(ManagedProcessError::InvalidArgument(
            "exécutable wrapper absent".to_string(),
        ));
    }
    let mut instance_id = None;
    let mut command_id = None;
    let mut generation = None;
    let mut index = 0;
    while index < options.len() {
        let value = options.get(index + 1).ok_or_else(|| {
            ManagedProcessError::InvalidArgument(format!("valeur absente après {}", options[index]))
        })?;
        match options[index].as_str() {
            "--instance-id" if instance_id.is_none() => instance_id = Some(value.clone()),
            "--command-id" if command_id.is_none() => command_id = Some(value.clone()),
            "--generation" if generation.is_none() => {
                generation = Some(value.parse::<u64>().map_err(|_| {
                    ManagedProcessError::InvalidArgument("génération invalide".to_string())
                })?)
            }
            unknown => {
                return Err(ManagedProcessError::InvalidArgument(format!(
                    "option inconnue ou dupliquée: {unknown}"
                )));
            }
        }
        index += 2;
    }
    let identity = ManagedIdentity {
        instance_id: instance_id.ok_or_else(|| {
            ManagedProcessError::InvalidArgument("instance_id absent".to_string())
        })?,
        command_id: command_id
            .ok_or_else(|| ManagedProcessError::InvalidArgument("command_id absent".to_string()))?,
        generation: generation.ok_or_else(|| {
            ManagedProcessError::InvalidArgument("génération absente".to_string())
        })?,
    };
    validate_identity(&identity)?;
    Ok(BootstrapRequest {
        identity,
        wrapper_executable: PathBuf::from(&wrapper[0]),
        wrapper_args: wrapper[1..].to_vec(),
    })
}

fn validate_identity(identity: &ManagedIdentity) -> Result<(), ManagedProcessError> {
    for (label, value) in [
        ("instance_id", identity.instance_id.as_str()),
        ("command_id", identity.command_id.as_str()),
    ] {
        if value.is_empty()
            || value.len() > 200
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(ManagedProcessError::InvalidArgument(format!(
                "{label} invalide"
            )));
        }
    }
    if identity.generation == 0 {
        return Err(ManagedProcessError::InvalidArgument(
            "génération nulle".to_string(),
        ));
    }
    Ok(())
}

fn run_bootstrap(request: BootstrapRequest) -> Result<(), ManagedProcessError> {
    set_fd_cloexec(RELEASE_FD, true)?;
    set_fd_cloexec(STATUS_FD, false)?;

    let pgid = unsafe { libc::setsid() };
    if pgid < 0 {
        return Err(io::Error::last_os_error().into());
    }
    let ready = BootstrapReady {
        pid: std::process::id(),
        pgid: pgid as u32,
        birth: process_birth(std::process::id())?,
        instance_id: request.identity.instance_id,
        command_id: request.identity.command_id,
        generation: request.identity.generation,
    };
    write_status_line(&ManagedStatus::BootstrapReady(ready.clone()))?;

    let mut release = [0_u8; 1];
    let read = read_one(RELEASE_FD, &mut release)?;
    if read == 0 {
        unsafe { libc::_exit(ABANDONED_EXIT_CODE) };
    }
    if release[0] != RELEASE_BYTE {
        return Err(ManagedProcessError::InvalidArgument(
            "octet RELEASE invalide".to_string(),
        ));
    }

    let mut wrapper = Command::new(request.wrapper_executable);
    wrapper
        .args(request.wrapper_args)
        .env(MANAGED_STATUS_FD_ENV, STATUS_FD.to_string())
        .env(MANAGED_INSTANCE_ID_ENV, &ready.instance_id)
        .env(MANAGED_COMMAND_ID_ENV, &ready.command_id)
        .env(MANAGED_GENERATION_ENV, ready.generation.to_string());
    let source = wrapper.exec();
    Err(ManagedProcessError::Io(source))
}

/// Identifiant de naissance comparable lors de la réconciliation anti-PID recyclé.
#[cfg(target_os = "macos")]
fn process_snapshot(pid: u32) -> io::Result<(u64, bool)> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let expected = std::mem::size_of::<libc::proc_bsdinfo>();
    let written = unsafe {
        libc::proc_pidinfo(
            pid as libc::c_int,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            expected as libc::c_int,
        )
    };
    if written != expected as libc::c_int {
        return Err(io::Error::last_os_error());
    }
    let info = unsafe { info.assume_init() };
    Ok((info.pbi_start_tvsec.saturating_mul(1_000_000)
        .saturating_add(info.pbi_start_tvusec),
        info.pbi_status != libc::SZOMB && info.pbi_status != libc::SIDL))
}

#[cfg(target_os = "linux")]
fn process_snapshot(pid: u32) -> io::Result<(u64, bool)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let after_name = stat
        .rsplit_once(')')
        .map(|(_, tail)| tail)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "stat de processus invalide"))?;
    // Le suffixe commence au champ 3 ; starttime est le champ 22.
    let state=after_name.split_whitespace().next()
        .ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"état absent"))?;
    let birth=after_name.split_whitespace().nth(19)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "starttime absent"))?
        .parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "starttime invalide"))?;
    Ok((birth,!matches!(state,"Z"|"X"|"x")))
}

pub fn process_birth(pid:u32)->io::Result<u64> {
    process_snapshot(pid).map(|(birth,_)|birth)
}

/// Lecture OS seulement. Pas de signal, de wait ou de changement de statut.
pub fn process_matches_live(pid:u32,birth:u64)->io::Result<bool> {
    match process_snapshot(pid) {
        Ok((actual,live))=>Ok(actual==birth && live),
        Err(error) if error.kind()==io::ErrorKind::NotFound
            || matches!(error.raw_os_error(),Some(libc::ESRCH))=>Ok(false),
        Err(error)=>Err(error),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("managed-bootstrap exige une source de naissance de processus macOS ou Linux");

fn write_status_line(status: &ManagedStatus) -> Result<(), ManagedProcessError> {
    write_status_to_fd(STATUS_FD, status)
}

fn write_status_to_fd(fd: RawFd, status: &ManagedStatus) -> Result<(), ManagedProcessError> {
    let mut bytes = serde_json::to_vec(status)
        .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string()))?;
    bytes.push(b'\n');
    write_all_fd(fd, &bytes)?;
    Ok(())
}

fn read_one(fd: RawFd, buffer: &mut [u8; 1]) -> io::Result<usize> {
    loop {
        let read = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), 1) };
        if read >= 0 {
            return Ok(read as usize);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn write_all_fd(fd: RawFd, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if written > 0 {
            bytes = &bytes[written as usize..];
            continue;
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    Ok(())
}

fn set_fd_cloexec(fd: RawFd, enabled: bool) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    let next = if enabled {
        flags | libc::FD_CLOEXEC
    } else {
        flags & !libc::FD_CLOEXEC
    };
    if unsafe { libc::fcntl(fd, libc::F_SETFD, next) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn spawn_bootstrap_command(
    mut command: Command,
    stderr: Stdio,
) -> Result<ManagedChild, ManagedProcessError> {
    let (daemon_release, bootstrap_release) = UnixStream::pair()?;
    let (daemon_status, bootstrap_status) = UnixStream::pair()?;
    let release_source = bootstrap_release.as_raw_fd();
    let status_source = bootstrap_status.as_raw_fd();

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr);
    unsafe {
        command.pre_exec(move || {
            let release_copy =
                libc::fcntl(release_source, libc::F_DUPFD_CLOEXEC, FIRST_AUXILIARY_FD);
            if release_copy < 0 {
                return Err(io::Error::last_os_error());
            }
            let status_copy = libc::fcntl(status_source, libc::F_DUPFD_CLOEXEC, FIRST_AUXILIARY_FD);
            if status_copy < 0 {
                libc::close(release_copy);
                return Err(io::Error::last_os_error());
            }
            let result = (|| {
                if libc::dup2(release_copy, RELEASE_FD) < 0
                    || libc::dup2(status_copy, STATUS_FD) < 0
                {
                    return Err(io::Error::last_os_error());
                }
                // Les deux cibles traversent l'exec initial vers le bootstrap.
                // Celui-ci remet RELEASE en CLOEXEC avant l'exec du wrapper.
                set_fd_cloexec(RELEASE_FD, false)?;
                set_fd_cloexec(STATUS_FD, false)
            })();
            libc::close(release_copy);
            libc::close(status_copy);
            result
        });
    }
    let child = command.spawn()?;
    drop(bootstrap_release);
    drop(bootstrap_status);
    Ok(ManagedChild {
        inner: ProcessChannels {
            child,
            release: Some(daemon_release),
            status: Some(BufReader::new(daemon_status)),
        },
    })
}

/// Extrémité wrapper du canal `managed-status`. L'identité provient uniquement
/// du bootstrap, jamais des arguments de l'adaptateur ACP.
pub struct ManagedStatusReporter {
    fd: Option<RawFd>,
    identity: ManagedIdentity,
}

impl ManagedStatusReporter {
    pub fn from_environment() -> Result<Option<Self>, ManagedProcessError> {
        let Some(raw_fd) = std::env::var_os(MANAGED_STATUS_FD_ENV) else {
            return Ok(None);
        };
        let fd = raw_fd
            .to_str()
            .and_then(|value| value.parse::<RawFd>().ok())
            .filter(|fd| *fd >= 0)
            .ok_or_else(|| {
                ManagedProcessError::InvalidArgument("FD managed-status invalide".to_string())
            })?;
        if unsafe { libc::fcntl(fd, libc::F_GETFD) } < 0 {
            return Err(io::Error::last_os_error().into());
        }
        // Le bootstrap a volontairement retiré CLOEXEC pour transmettre ce FD
        // au wrapper. Celui-ci le remet immédiatement : l'adaptateur ACP lancé
        // ensuite ne doit pas garder le canal ouvert et retarder indéfiniment
        // le passage BootstrapReady -> Connected côté superviseur.
        set_fd_cloexec(fd, true)?;
        let generation = std::env::var(MANAGED_GENERATION_ENV)
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| {
                ManagedProcessError::InvalidArgument("génération gérée absente".to_string())
            })?;
        let identity = ManagedIdentity {
            instance_id: std::env::var(MANAGED_INSTANCE_ID_ENV).map_err(|_| {
                ManagedProcessError::InvalidArgument("instance gérée absente".to_string())
            })?,
            command_id: std::env::var(MANAGED_COMMAND_ID_ENV).map_err(|_| {
                ManagedProcessError::InvalidArgument("commande gérée absente".to_string())
            })?,
            generation,
        };
        validate_identity(&identity)?;
        Ok(Some(Self {
            fd: Some(fd),
            identity,
        }))
    }

    pub fn instance_id(&self) -> &str {
        &self.identity.instance_id
    }

    pub(crate) fn identity(&self) -> &ManagedIdentity {
        &self.identity
    }

    pub fn startup_failed(
        &mut self,
        kind: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<(), ManagedProcessError> {
        let Some(fd) = self.fd.take() else {
            return Ok(());
        };
        let status = ManagedStatus::StartupFailed {
            kind: kind.into(),
            reason: reason.into(),
            instance_id: self.identity.instance_id.clone(),
            command_id: self.identity.command_id.clone(),
            generation: self.identity.generation,
        };
        let result = write_status_to_fd(fd, &status);
        unsafe {
            libc::close(fd);
        }
        result
    }

    /// Le vrai Register et l'initialisation du transport sont terminés : la
    /// fermeture du FD signifie « aucun StartupFailed », jamais « Connected ».
    pub fn startup_succeeded(&mut self) {
        if let Some(fd) = self.fd.take() {
            unsafe {
                libc::close(fd);
            }
        }
    }
}

impl Drop for ManagedStatusReporter {
    fn drop(&mut self) {
        if self.fd.is_some() {
            let _ = self.startup_failed(
                "negotiation_failed",
                "hook managed-status abandonné avant la fin de l'initialisation",
            );
        }
    }
}

/// Journal stderr privé, distinct pour chaque instance et génération.
pub struct ManagedStderrStore {
    directory: PathBuf,
}

impl ManagedStderrStore {
    pub fn for_home(home: &Path) -> Self {
        Self::at_directory(home.join(".cache/bridget/managed-stderr"))
    }

    pub fn at_directory(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn open(
        &self,
        name: &str,
        identity: &ManagedIdentity,
    ) -> Result<(File, PathBuf), ManagedProcessError> {
        validate_marker_name(name)?;
        validate_identity(identity)?;
        private_directory(&self.directory)?;
        let agent = self.directory.join(name);
        private_directory(&agent)?;
        let generation = agent.join(format!("{}-g{}", identity.instance_id, identity.generation));
        private_directory(&generation)?;
        let path = generation.join("stderr.log");
        if path.exists() {
            let metadata = fs::symlink_metadata(&path)?;
            if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o777 != 0o600 {
                return Err(ManagedProcessError::Io(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("fichier stderr privé non conforme: {}", path.display()),
                )));
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(&path)?;
        Ok((file, path))
    }

    pub fn purge_older_than_days(&self, days: u32) -> Result<usize, ManagedProcessError> {
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(u64::from(days).saturating_mul(86_400)))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mut removed = 0;
        let Ok(agents) = fs::read_dir(&self.directory) else {
            return Ok(0);
        };
        for agent in agents.flatten().filter(|entry| entry.path().is_dir()) {
            let Ok(generations) = fs::read_dir(agent.path()) else {
                continue;
            };
            for generation in generations.flatten().filter(|entry| entry.path().is_dir()) {
                let path = generation.path().join("stderr.log");
                let expired = fs::metadata(&path)
                    .and_then(|metadata| metadata.modified())
                    .is_ok_and(|modified| modified < cutoff);
                if expired {
                    fs::remove_file(&path)?;
                    let _ = fs::remove_dir(generation.path());
                    removed += 1;
                }
            }
            let _ = fs::remove_dir(agent.path());
        }
        Ok(removed)
    }
}

fn private_directory(path: &Path) -> io::Result<()> {
    if path.exists() {
        let metadata = fs::metadata(path)?;
        if !metadata.is_dir() || metadata.permissions().mode() & 0o777 != 0o700 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("répertoire privé non conforme: {}", path.display()),
            ));
        }
        return Ok(());
    }
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder.create(path)
}

fn validate_marker_name(name: &str) -> Result<(), ManagedProcessError> {
    if name.is_empty()
        || name.len() > 100
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ManagedProcessError::InvalidArgument(
            "nom de marqueur invalide".to_string(),
        ));
    }
    Ok(())
}

/// Store minimal des marqueurs `~/.cache/bridget/managed/<nom>.json`.
#[derive(Clone)]
pub struct ManagedMarkerStore {
    directory: PathBuf,
}

/// Preuve typée qu'un marqueur corrélé a été écrit durablement.
pub struct ManagedMarkerHandle {
    path: PathBuf,
    marker: ManagedMarker,
}

impl ManagedMarkerHandle {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn marker(&self) -> &ManagedMarker {
        &self.marker
    }
}

impl ManagedMarkerStore {
    pub fn for_home(home: &Path) -> Self {
        Self::at_directory(home.join(".cache").join("bridget").join("managed"))
    }

    pub fn at_directory(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn persist(
        &self,
        name: &str,
        ready: &BootstrapReady,
    ) -> Result<ManagedMarkerHandle, ManagedProcessError> {
        let path = self.marker_path(name)?;
        let marker = ManagedMarker::from(ready);
        let mut bytes = serde_json::to_vec_pretty(&marker)
            .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string()))?;
        bytes.push(b'\n');
        write_private_file_atomic(&path, &bytes)?;
        Ok(ManagedMarkerHandle { path, marker })
    }

    pub fn load(&self, name: &str) -> Result<ManagedMarker, ManagedProcessError> {
        let path = self.marker_path(name)?;
        let bytes = fs::read(&path)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string()))
    }

    /// Termine, avant toute reprise, les groupes laissés par une génération
    /// précédente du daemon. Ces processus ne sont plus ses enfants : leur
    /// disparition est donc constatée exclusivement par `kill(-pgid, 0)` ; un
    /// `waitpid` produirait `ECHILD` et n'est volontairement jamais utilisé.
    ///
    /// Les marqueurs sont parcourus par nom pour rendre la réconciliation
    /// déterministe. Un marqueur n'est supprimé qu'après disparition confirmée
    /// du groupe, ou lorsque la naissance prouve que le PID a été recyclé.
    pub fn reconcile_stale_groups(
        &self,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Vec<String>, ManagedProcessError> {
        self.reconcile_stale_groups_with_native(
            timeout,
            poll_interval,
            &BTreeMap::new(),
            Duration::ZERO,
        )
    }

    /// Seule la réconciliation du daemon fournit ces identités depuis la saga
    /// native et le lien durable de flotte. Aucun nom ou PID déclaré par RPC.
    pub(crate) fn reconcile_stale_groups_with_native(
        &self,
        timeout: Duration,
        poll_interval: Duration,
        native_identities: &BTreeMap<String, ManagedIdentity>,
        native_grace: Duration,
    ) -> Result<Vec<String>, ManagedProcessError> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| {
                    ManagedProcessError::InvalidStatus(format!(
                        "nom de marqueur non UTF-8: {}",
                        path.display()
                    ))
                })?
                .to_string();
            validate_marker_name(&name)?;
            names.push(name);
        }
        names.sort();

        let mut reconciled = Vec::with_capacity(names.len());
        let mut native_groups = Vec::new();
        let mut ordinary_groups = Vec::new();
        for name in names {
            let marker = self.load(&name)?;
            validate_identity(&ManagedIdentity {
                instance_id: marker.instance_id.clone(),
                command_id: marker.command_id.clone(),
                generation: marker.generation,
            })?;
            if let Some(expected) = native_identities.get(&name)
                && (marker.instance_id != expected.instance_id
                    || marker.command_id != expected.command_id
                    || marker.generation != expected.generation)
            {
                return Err(ManagedProcessError::InvalidStatus(format!(
                    "identité native du marqueur incohérente pour {name}"
                )));
            }
            if marker.pgid == 0 {
                return Err(ManagedProcessError::InvalidStatus(format!(
                    "marqueur géré incomplet pour {name}"
                )));
            }
            let birth = match process_birth(marker.pgid) {
                Ok(birth) => birth,
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH) =>
                {
                    self.remove(&name)?;
                    reconciled.push(name);
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            if birth != marker.birth || !group_exists(marker.pgid)? {
                self.remove(&name)?;
                reconciled.push(name);
                continue;
            }
            if native_identities.contains_key(&name) {
                native_groups.push((name, marker));
            } else {
                ordinary_groups.push((name, marker));
            }
        }
        // Tous les wrappers natifs reçoivent le signal avant l'attente. Une
        // seule échéance couvre le lot ; aucune escalade SIGKILL ne peut tuer
        // leur wrapper avant qu'il ferme SON transport fournisseur.
        let native_deadline = Instant::now() + native_grace;
        for (_, marker) in &native_groups {
            // Recontrôler la naissance juste avant le signal, après lecture
            // des autres marqueurs. Un PID recyclé ne reçoit aucun signal.
            match process_birth(marker.pgid) {
                Ok(birth) if birth == marker.birth => signal_group(marker.pgid, libc::SIGTERM)?,
                Ok(_) => continue,
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH) =>
                {
                    continue;
                }
                Err(error) => return Err(error.into()),
            }
        }
        for (name, marker) in ordinary_groups {
            // La classification en lot ne permet pas de signaler un PID qui
            // a été recyclé pendant le traitement des marqueurs précédents.
            let same_birth = match process_birth(marker.pgid) {
                Ok(birth) => birth == marker.birth,
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH) =>
                {
                    false
                }
                Err(error) => return Err(error.into()),
            };
            if !same_birth || !group_exists(marker.pgid)? {
                self.remove(&name)?;
                reconciled.push(name);
                continue;
            }
            signal_group(marker.pgid, libc::SIGTERM)?;
            let cooperative_grace = timeout / 2;
            let forced_grace = timeout.saturating_sub(cooperative_grace);
            if !wait_group_gone(marker.pgid, cooperative_grace, poll_interval)? {
                match signal_group(marker.pgid, libc::SIGKILL) {
                    Ok(()) => {}
                    Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
                    Err(error) => return Err(error.into()),
                }
            }
            if !wait_group_gone(marker.pgid, forced_grace, poll_interval)? {
                return Err(ManagedProcessError::InvalidStatus(format!(
                    "groupe périmé encore vivant pour {name} (pgid {})",
                    marker.pgid
                )));
            }
            self.remove(&name)?;
            reconciled.push(name);
        }
        for (name, marker) in native_groups {
            let same_birth = match process_birth(marker.pgid) {
                Ok(birth) => birth == marker.birth,
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH) =>
                {
                    false
                }
                Err(error) => return Err(error.into()),
            };
            if same_birth
                && !wait_group_gone(
                    marker.pgid,
                    native_deadline.saturating_duration_since(Instant::now()),
                    poll_interval,
                )?
            {
                return Err(ManagedProcessError::InvalidStatus(format!(
                    "arrêt coopératif natif incomplet pour {name} (pgid {})",
                    marker.pgid
                )));
            }
            self.remove(&name)?;
            reconciled.push(name);
        }
        reconciled.sort();
        Ok(reconciled)
    }

    /// Arrête un groupe retrouvé uniquement par son marqueur après perte de
    /// la table superviseur. Un marqueur dont le leader a disparu ou dont la
    /// naissance ne correspond plus est retiré sans signaler le PID recyclé.
    pub fn stop_current_group(
        &self,
        name: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Option<ManagedStopResult>, ManagedProcessError> {
        let marker = match self.load(name) {
            Ok(marker) => marker,
            Err(ManagedProcessError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let current_birth = match process_birth(marker.pgid) {
            Ok(birth) => birth,
            Err(error)
                if error.kind() == io::ErrorKind::NotFound
                    || error.raw_os_error() == Some(libc::ESRCH) =>
            {
                self.remove(name)?;
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        if current_birth != marker.birth || !group_exists(marker.pgid)? {
            self.remove(name)?;
            return Ok(None);
        }
        let survivors_killed = group_member_count(marker.pgid)?.max(1);
        signal_group(marker.pgid, libc::SIGTERM)?;
        if wait_group_gone(marker.pgid, timeout, poll_interval)? {
            self.remove(name)?;
            return Ok(Some(ManagedStopResult::StoppedForced { survivors_killed }));
        }
        Ok(Some(ManagedStopResult::Timeout))
    }

    pub fn remove(&self, name: &str) -> Result<(), ManagedProcessError> {
        let path = self.marker_path(name)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn marker_path(&self, name: &str) -> Result<PathBuf, ManagedProcessError> {
        validate_marker_name(name)?;
        Ok(self.directory.join(format!("{name}.json")))
    }
}

#[cfg(test)]
#[path = "managed_process_tests.rs"]
pub(crate) mod tests;
