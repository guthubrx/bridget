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

/// Wrapper exécuté et supervisable par T904/T906.
pub struct RunningManagedChild {
    inner: ProcessChannels,
    marker: ManagedMarkerHandle,
}

impl ManagedChild {
    pub fn set_ready_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        self.inner
            .status
            .as_mut()
            .expect("canal de statut présent avant le transfert")
            .get_mut()
            .set_read_timeout(Some(timeout))
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
        self.inner
            .status
            .as_mut()
            .expect("canal de statut présent avant le transfert")
            .get_mut()
            .set_read_timeout(Some(poll_interval))?;
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
                    return Ok(Some(ReadyManagedChild {
                        inner: self.inner,
                        ready,
                    }));
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
    pub fn ready(&self) -> &BootstrapReady {
        &self.ready
    }

    /// Écrit durablement le marqueur avant de rendre RELEASE accessible.
    pub fn persist_marker(
        mut self,
        store: &ManagedMarkerStore,
        name: &str,
    ) -> Result<MarkedManagedChild, ManagedProcessError> {
        match store.persist(name, &self.ready) {
            Ok(marker) => Ok(MarkedManagedChild {
                inner: self.inner,
                marker,
            }),
            Err(error) => {
                self.inner.release.take();
                let _ = self.inner.child.wait();
                Err(error)
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
    pub fn release(mut self) -> Result<RunningManagedChild, ManagedProcessError> {
        let mut release = self.inner.release.take().ok_or_else(|| {
            ManagedProcessError::InvalidArgument("RELEASE déjà envoyé".to_string())
        })?;
        release.write_all(&[RELEASE_BYTE])?;
        release.flush()?;
        drop(release);
        Ok(RunningManagedChild {
            inner: self.inner,
            marker: self.marker,
        })
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
        self.inner
            .status
            .as_mut()
            .expect("canal de statut déjà fermé")
            .get_mut()
            .set_nonblocking(true)
    }

    /// Lit au plus un statut. Les messages sont des lignes JSON atomiques et
    /// petites ; `WouldBlock` signifie simplement qu'aucun événement n'est
    /// disponible à ce tick.
    pub fn try_status(&mut self) -> Result<Option<ManagedStatus>, ManagedProcessError> {
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
pub fn process_birth(pid: u32) -> io::Result<u64> {
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
    Ok(info
        .pbi_start_tvsec
        .saturating_mul(1_000_000)
        .saturating_add(info.pbi_start_tvusec))
}

#[cfg(target_os = "linux")]
pub fn process_birth(pid: u32) -> io::Result<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let after_name = stat
        .rsplit_once(')')
        .map(|(_, tail)| tail)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "stat de processus invalide"))?;
    // Le suffixe commence au champ 3 ; starttime est le champ 22.
    after_name
        .split_whitespace()
        .nth(19)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "starttime absent"))?
        .parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "starttime invalide"))
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
        for name in names {
            let marker = self.load(&name)?;
            validate_identity(&ManagedIdentity {
                instance_id: marker.instance_id.clone(),
                command_id: marker.command_id.clone(),
                generation: marker.generation,
            })?;
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
pub(crate) mod tests {
    use super::*;
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;
    use std::process::ExitStatus;
    use std::thread;
    use std::time::{Duration, Instant};

    const BOOTSTRAP_CHILD_ENV: &str = "BRIDGET_T903_BOOTSTRAP_CHILD";
    const BOOTSTRAP_REQUEST_ENV: &str = "BRIDGET_T903_BOOTSTRAP_REQUEST";
    const FD_PROBE_ENV: &str = "BRIDGET_T903_FD_PROBE";
    const CONTROLLER_ENV: &str = "BRIDGET_T903_CRASH_CONTROLLER";
    const CONTROLLER_STAGE_ENV: &str = "BRIDGET_T903_CRASH_STAGE";
    const CONTROLLER_ROOT_ENV: &str = "BRIDGET_T903_CRASH_ROOT";

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t903-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn identity() -> ManagedIdentity {
        ManagedIdentity {
            instance_id: "instance-test".to_string(),
            command_id: "command-test".to_string(),
            generation: 7,
        }
    }

    fn current_test_executable() -> PathBuf {
        let executable = std::env::current_exe().unwrap();
        assert_ne!(
            executable.file_name().and_then(|name| name.to_str()),
            Some("firefox")
        );
        executable
    }

    fn helper_command(test_name: &str) -> Command {
        let mut command = Command::new(current_test_executable());
        command
            .arg("--exact")
            .arg(test_name)
            .arg("--ignored")
            .arg("--nocapture")
            .arg("--test-threads=1");
        command
    }

    pub(crate) fn spawn_test_bootstrap() -> ManagedChild {
        spawn_test_bootstrap_with_wrapper(
            current_test_executable(),
            vec![
                "--exact".to_string(),
                "managed_process::tests::fd_probe_child".to_string(),
                "--ignored".to_string(),
                "--nocapture".to_string(),
                "--test-threads=1".to_string(),
            ],
        )
    }

    fn spawn_test_bootstrap_with_wrapper(
        wrapper_executable: PathBuf,
        wrapper_args: Vec<String>,
    ) -> ManagedChild {
        let request = BootstrapRequest {
            identity: identity(),
            wrapper_executable,
            wrapper_args,
        };
        let needs_fd_probe = request
            .wrapper_args
            .iter()
            .any(|argument| argument == "managed_process::tests::fd_probe_child");
        let mut command = helper_command("managed_process::tests::bootstrap_child");
        command.env(BOOTSTRAP_CHILD_ENV, "1").env(
            BOOTSTRAP_REQUEST_ENV,
            serde_json::to_string(&json!({
                "instance_id": request.identity.instance_id,
                "command_id": request.identity.command_id,
                "generation": request.identity.generation,
                "wrapper_executable": request.wrapper_executable,
                "wrapper_args": request.wrapper_args,
            }))
            .unwrap(),
        );
        if needs_fd_probe {
            command.env(FD_PROBE_ENV, "1");
        }
        spawn_bootstrap_command(command, Stdio::null()).unwrap()
    }

    #[test]
    #[ignore]
    fn bootstrap_child() {
        if std::env::var(BOOTSTRAP_CHILD_ENV).ok().as_deref() != Some("1") {
            return;
        }
        let value: serde_json::Value =
            serde_json::from_str(&std::env::var(BOOTSTRAP_REQUEST_ENV).unwrap()).unwrap();
        let request = BootstrapRequest {
            identity: ManagedIdentity {
                instance_id: value["instance_id"].as_str().unwrap().to_string(),
                command_id: value["command_id"].as_str().unwrap().to_string(),
                generation: value["generation"].as_u64().unwrap(),
            },
            wrapper_executable: PathBuf::from(value["wrapper_executable"].as_str().unwrap()),
            wrapper_args: value["wrapper_args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|argument| argument.as_str().unwrap().to_string())
                .collect(),
        };
        run_bootstrap(request).unwrap();
    }

    #[test]
    #[ignore]
    fn fd_probe_child() {
        if std::env::var(FD_PROBE_ENV).ok().as_deref() != Some("1") {
            return;
        }
        let status_open = unsafe { libc::fcntl(STATUS_FD, libc::F_GETFD) } >= 0;
        let release_open = unsafe { libc::fcntl(RELEASE_FD, libc::F_GETFD) } >= 0;
        write_all_fd(
            STATUS_FD,
            format!(
                "{}\n",
                json!({
                    "event": "fd_probe",
                    "status_open": status_open,
                    "release_open": release_open,
                })
            )
            .as_bytes(),
        )
        .unwrap();
        let mut byte = [0_u8; 1];
        while read_one(STATUS_FD, &mut byte).unwrap_or(0) != 0 {}
    }

    fn read_probe(child: &mut RunningManagedChild) -> serde_json::Value {
        let mut line = String::new();
        child.status_reader().read_line(&mut line).unwrap();
        serde_json::from_str(line.trim_end()).unwrap()
    }

    #[test]
    fn eof_abandonne_tandis_que_release_exec_la_sonde() {
        let abandoned = spawn_test_bootstrap().wait_ready().unwrap();
        assert_eq!(abandoned.ready().pid, abandoned.ready().pgid);
        assert_eq!(
            abandoned.ready().birth,
            process_birth(abandoned.ready().pid).unwrap()
        );
        let mut child = abandoned.abandon();
        let status = child.wait().unwrap();
        assert_eq!(status.code(), Some(ABANDONED_EXIT_CODE));

        let root = test_root("release");
        let store = ManagedMarkerStore::at_directory(root.join("managed"));
        let mut released = spawn_test_bootstrap()
            .wait_ready()
            .unwrap()
            .persist_marker(&store, "codex-1")
            .unwrap()
            .release()
            .unwrap();
        assert!(released.try_wait().unwrap().is_none());
        let probe = read_probe(&mut released);
        assert_eq!(probe["event"], "fd_probe");
        assert_eq!(probe["status_open"], true);
        assert_eq!(probe["release_open"], false);
        released.close_status();
        assert!(released.child_mut().wait().unwrap().success());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn annulation_avant_marqueur_termine_le_bootstrap_sans_exec() {
        let child = spawn_test_bootstrap();
        let pid = child.inner.child.id();
        let outcome = child
            .wait_ready_or_cancel(|| true, Duration::from_millis(10))
            .unwrap();

        assert!(outcome.is_none());
        wait_pid_gone(pid);
    }

    #[test]
    fn stop_force_termine_l_intermediaire_npx_qui_ignore_l_annulation_et_son_descendant() {
        let root = test_root("stop-group");
        fs::create_dir_all(&root).unwrap();
        let descendant_path = root.join("descendant.pid");
        let fake_npx = root.join("npx");
        fs::write(&fake_npx, "#!/bin/sh\nsleep 30 &\necho $! > \"$1\"\nwait\n").unwrap();
        fs::set_permissions(&fake_npx, fs::Permissions::from_mode(0o700)).unwrap();
        let store = ManagedMarkerStore::at_directory(root.join("managed"));
        let mut child = spawn_test_bootstrap_with_wrapper(
            fake_npx,
            vec![descendant_path.to_string_lossy().into_owned()],
        )
        .wait_ready()
        .unwrap()
        .persist_marker(&store, "codex-1")
        .unwrap()
        .release()
        .unwrap();
        let descendant = wait_for_path(&descendant_path, |content| {
            std::str::from_utf8(content)
                .ok()?
                .trim()
                .parse::<u32>()
                .ok()
        });

        assert!(matches!(
            child
                .stop_group(
                    Duration::ZERO,
                    Duration::from_secs(2),
                    Duration::from_millis(10),
                )
                .unwrap(),
            ManagedStopResult::StoppedForced { survivors_killed }
                if survivors_killed >= 2
        ));
        assert!(matches!(
            store.load("codex-1"),
            Err(ManagedProcessError::Io(ref error))
                if error.kind() == io::ErrorKind::NotFound
        ));
        wait_pid_gone(descendant);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marqueur_perime_est_retire_sans_signaler_le_pid_recycle() {
        let root = test_root("stale-marker");
        let store = ManagedMarkerStore::at_directory(root.join("managed"));
        // Même avec le mutant qui ignore la naissance, le seul groupe
        // signalable est cet enfant possédé, JAMAIS le groupe du runner.
        let child = spawn_test_bootstrap().wait_ready().unwrap();
        let mut ready = child.ready().clone();
        assert_eq!(ready.pid, ready.pgid);
        assert_ne!(ready.pgid, unsafe { libc::getpgrp() } as u32);
        ready.birth = ready.birth.saturating_add(1);
        store.persist("codex-1", &ready).unwrap();
        let result = store.stop_current_group(
            "codex-1",
            Duration::from_millis(50),
            Duration::from_millis(5),
        );
        let status = child.abandon().wait().unwrap();
        assert_eq!(result.unwrap(), None);
        assert_eq!(
            status.code(),
            Some(ABANDONED_EXIT_CODE),
            "aucun signal au PID périmé"
        );
        assert!(matches!(
            store.load("codex-1"),
            Err(ManagedProcessError::Io(ref error))
                if error.kind() == io::ErrorKind::NotFound
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reconciliation_termine_un_groupe_reel_avant_de_supprimer_son_marqueur() {
        let root = test_root("reconcile-real-group");
        fs::create_dir_all(&root).unwrap();
        let store = ManagedMarkerStore::at_directory(root.join("managed"));
        let mut running = spawn_test_bootstrap_with_wrapper(
            PathBuf::from("/bin/sh"),
            vec![
                "-c".to_string(),
                "trap '' TERM; while :; do sleep 1; done".to_string(),
            ],
        )
        .wait_ready()
        .unwrap()
        .persist_marker(&store, "codex-reconcile")
        .unwrap()
        .release()
        .unwrap();
        let pgid = running.marker().marker().pgid;
        let reaper = thread::spawn(move || running.child_mut().wait().unwrap());

        assert_eq!(
            store
                .reconcile_stale_groups(Duration::from_secs(2), Duration::from_millis(10))
                .unwrap(),
            ["codex-reconcile"]
        );
        assert!(!group_exists(pgid).unwrap());
        let _ = reaper.join().unwrap();
        assert!(matches!(
            store.load("codex-reconcile"),
            Err(ManagedProcessError::Io(ref error))
                if error.kind() == io::ErrorKind::NotFound
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stderr_est_prive_separe_par_generation_et_purge_par_age() {
        let root = test_root("stderr");
        let store = ManagedStderrStore::at_directory(root.join("stderr"));
        let (mut file, first) = store.open("codex-1", &identity()).unwrap();
        writeln!(file, "diagnostic privé").unwrap();
        file.flush().unwrap();
        let second_identity = ManagedIdentity {
            generation: identity().generation + 1,
            ..identity()
        };
        let (_second, second) = store.open("codex-1", &second_identity).unwrap();
        assert_ne!(first.parent(), second.parent());
        for directory in [
            first.parent().unwrap(),
            first.parent().unwrap().parent().unwrap(),
            first.parent().unwrap().parent().unwrap().parent().unwrap(),
        ] {
            assert_eq!(
                fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(
            fs::metadata(&first).unwrap().permissions().mode() & 0o777,
            0o600
        );
        drop(file);
        assert_eq!(store.purge_older_than_days(0).unwrap(), 2);
        assert!(!first.exists());
        assert!(!second.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stderr_refuse_un_repertoire_existant_aux_permissions_trop_larges() {
        let root = test_root("stderr-permissions");
        let directory = root.join("stderr");
        fs::create_dir_all(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).unwrap();
        let store = ManagedStderrStore::at_directory(&directory);

        let error = store.open("codex-1", &identity()).unwrap_err();
        assert!(matches!(
            error,
            ManagedProcessError::Io(ref source)
                if source.kind() == io::ErrorKind::PermissionDenied
        ));
        assert_eq!(
            fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
            0o755
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stderr_refuse_un_fichier_existant_non_regulier_ou_trop_permissif() {
        let root = test_root("stderr-file-permissions");
        let store = ManagedStderrStore::at_directory(root.join("stderr"));
        let (file, path) = store.open("codex-1", &identity()).unwrap();
        drop(file);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        let error = store.open("codex-1", &identity()).unwrap_err();
        assert!(matches!(
            error,
            ManagedProcessError::Io(ref source)
                if source.kind() == io::ErrorKind::PermissionDenied
        ));
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o644
        );
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(matches!(
            store.open("codex-1", &identity()),
            Err(ManagedProcessError::Io(ref source))
                if source.kind() == io::ErrorKind::PermissionDenied
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sous_mode_parse_une_commande_wrapper_sans_confondre_ses_arguments() {
        let request = parse_bootstrap_args(&[
            "--instance-id".to_string(),
            "instance-9".to_string(),
            "--command-id".to_string(),
            "command-9".to_string(),
            "--generation".to_string(),
            "9".to_string(),
            "--".to_string(),
            "/usr/bin/agent".to_string(),
            "--name".to_string(),
            "codex-9".to_string(),
        ])
        .unwrap();
        assert_eq!(request.identity.generation, 9);
        assert_eq!(request.wrapper_executable, Path::new("/usr/bin/agent"));
        assert_eq!(request.wrapper_args, ["--name", "codex-9"]);
    }

    #[test]
    fn marqueur_prive_roundtrip_et_refuse_un_nom_traversant() {
        let root = test_root("marker");
        let store = ManagedMarkerStore::at_directory(root.join("managed"));
        let ready = BootstrapReady {
            pid: 42,
            pgid: 42,
            birth: 123,
            instance_id: identity().instance_id,
            command_id: identity().command_id,
            generation: identity().generation,
        };
        let marker = store.persist("codex-1", &ready).unwrap();
        let path = marker.path().to_path_buf();
        assert_eq!(store.load("codex-1").unwrap(), ManagedMarker::from(&ready));
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(store.persist("../escape", &ready).is_err());
        store.remove("codex-1").unwrap();
        assert!(!path.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    #[ignore]
    fn crash_controller_child() {
        if std::env::var(CONTROLLER_ENV).ok().as_deref() != Some("1") {
            return;
        }
        let stage = std::env::var(CONTROLLER_STAGE_ENV).unwrap();
        let root = PathBuf::from(std::env::var_os(CONTROLLER_ROOT_ENV).unwrap());
        let ready = spawn_test_bootstrap().wait_ready().unwrap();
        let ready_info = ready.ready().clone();
        let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
        match stage.as_str() {
            "before_marker" => {
                write_crash_boundary(&root, &ready_info);
                let _held_ready = ready;
                loop {
                    thread::park();
                }
            }
            "after_marker" => {
                let marked = ready.persist_marker(&marker_store, "codex-1").unwrap();
                write_crash_boundary(&root, &ready_info);
                let _held_marked = marked;
                loop {
                    thread::park();
                }
            }
            "after_release" => {
                let mut running = ready
                    .persist_marker(&marker_store, "codex-1")
                    .unwrap()
                    .release()
                    .unwrap();
                let probe = read_probe(&mut running);
                assert_eq!(probe["status_open"], true);
                assert_eq!(probe["release_open"], false);
                write_crash_boundary(&root, &ready_info);
                let _held_running = running;
                loop {
                    thread::park();
                }
            }
            unknown => panic!("frontière inconnue: {unknown}"),
        }
    }

    fn write_crash_boundary(root: &Path, ready: &BootstrapReady) {
        write_private_file_atomic(
            &root.join("boundary.json"),
            serde_json::to_string(ready).unwrap().as_bytes(),
        )
        .unwrap();
    }

    fn wait_for_path<T>(path: &Path, parse: impl Fn(&[u8]) -> Option<T>) -> T {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(content) = fs::read(path)
                && !content.is_empty()
                && let Some(value) = parse(&content)
            {
                return value;
            }
            assert!(
                Instant::now() < deadline,
                "barrière absente: {}",
                path.display()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_pid_gone(pid: u32) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
            if result < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "processus descendant {pid} encore vivant"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn terminate_controller(child: &mut Child) -> ExitStatus {
        // Ce PID est celui du processus de test créé juste au-dessus ; son
        // exécutable a été vérifié différent de Firefox avant le spawn.
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) },
            0
        );
        child.wait().unwrap()
    }

    #[test]
    fn crash_reel_aux_trois_frontieres_preserve_la_preuve_attendue() {
        for stage in ["before_marker", "after_marker", "after_release"] {
            let root = test_root(stage);
            let boundary = root.join("boundary.json");
            let mut controller = helper_command("managed_process::tests::crash_controller_child");
            controller
                .env(CONTROLLER_ENV, "1")
                .env(CONTROLLER_STAGE_ENV, stage)
                .env(CONTROLLER_ROOT_ENV, &root)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let mut controller = controller.spawn().unwrap();
            let ready: BootstrapReady =
                wait_for_path(&boundary, |content| serde_json::from_slice(content).ok());
            let status = terminate_controller(&mut controller);
            assert!(!status.success());
            wait_pid_gone(ready.pid);

            let marker = root.join("managed/codex-1.json");
            assert_eq!(
                marker.exists(),
                stage != "before_marker",
                "frontière {stage}"
            );
            let _ = fs::remove_dir_all(root);
        }
    }
}
