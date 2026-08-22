//! Bootstrap et marqueurs durables des équipiers supervisés par le daemon.

use bridget_transport::fsutil::write_private_file_atomic;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const RELEASE_FD: RawFd = 100;
const STATUS_FD: RawFd = 101;
const FIRST_AUXILIARY_FD: RawFd = 102;
const RELEASE_BYTE: u8 = b'R';
const ABANDONED_EXIT_CODE: libc::c_int = 125;
pub const MANAGED_STATUS_FD_ENV: &str = "BRIDGET_MANAGED_STATUS_FD";

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
}

/// Extrémités détenues par le daemon pendant la phase de bootstrap.
pub struct ManagedChild {
    pub child: Child,
    release: Option<UnixStream>,
    status: BufReader<UnixStream>,
}

impl ManagedChild {
    /// Attend la preuve de création du groupe. Ce statut n'est jamais un succès.
    pub fn wait_ready(&mut self) -> Result<BootstrapReady, ManagedProcessError> {
        let mut line = String::new();
        if self.status.read_line(&mut line)? == 0 {
            return Err(ManagedProcessError::InvalidStatus(
                "EOF avant BootstrapReady".to_string(),
            ));
        }
        match serde_json::from_str::<ManagedStatus>(line.trim_end()) {
            Ok(ManagedStatus::BootstrapReady(ready)) => Ok(ready),
            Ok(ManagedStatus::StartupFailed { .. }) => Err(ManagedProcessError::InvalidStatus(
                "StartupFailed reçu avant BootstrapReady".to_string(),
            )),
            Err(error) => Err(ManagedProcessError::InvalidStatus(error.to_string())),
        }
    }

    /// Libère exactement une fois le bootstrap après durabilité du marqueur.
    pub fn release(&mut self) -> Result<(), ManagedProcessError> {
        let mut release = self.release.take().ok_or_else(|| {
            ManagedProcessError::InvalidArgument("RELEASE déjà envoyé".to_string())
        })?;
        release.write_all(&[RELEASE_BYTE])?;
        release.flush()?;
        drop(release);
        Ok(())
    }

    /// Fermer sans octet est l'abandon explicite ; le bootstrap doit `_exit`.
    pub fn abandon(&mut self) {
        self.release.take();
    }

    pub fn status_reader(&mut self) -> &mut BufReader<UnixStream> {
        &mut self.status
    }
}

/// Lance le sous-mode de production avec deux canaux privés.
pub fn spawn_managed_bootstrap(
    launch: &ManagedLaunch,
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
    spawn_bootstrap_command(command)
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
    write_status_line(&ManagedStatus::BootstrapReady(ready))?;

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
        .env(MANAGED_STATUS_FD_ENV, STATUS_FD.to_string());
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
    let mut bytes = serde_json::to_vec(status)
        .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string()))?;
    bytes.push(b'\n');
    write_all_fd(STATUS_FD, &bytes)?;
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

fn spawn_bootstrap_command(mut command: Command) -> Result<ManagedChild, ManagedProcessError> {
    let (daemon_release, bootstrap_release) = UnixStream::pair()?;
    let (daemon_status, bootstrap_status) = UnixStream::pair()?;
    let release_source = bootstrap_release.as_raw_fd();
    let status_source = bootstrap_status.as_raw_fd();

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
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
        child,
        release: Some(daemon_release),
        status: BufReader::new(daemon_status),
    })
}

/// Store minimal des marqueurs `~/.cache/bridget/managed/<nom>.json`.
pub struct ManagedMarkerStore {
    directory: PathBuf,
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
    ) -> Result<PathBuf, ManagedProcessError> {
        let path = self.marker_path(name)?;
        let mut bytes = serde_json::to_vec_pretty(&ManagedMarker::from(ready))
            .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string()))?;
        bytes.push(b'\n');
        write_private_file_atomic(&path, &bytes)?;
        Ok(path)
    }

    pub fn load(&self, name: &str) -> Result<ManagedMarker, ManagedProcessError> {
        let path = self.marker_path(name)?;
        let bytes = fs::read(&path)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| ManagedProcessError::InvalidStatus(error.to_string()))
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
        Ok(self.directory.join(format!("{name}.json")))
    }
}

#[cfg(test)]
mod tests {
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

    fn spawn_test_bootstrap() -> ManagedChild {
        let request = BootstrapRequest {
            identity: identity(),
            wrapper_executable: current_test_executable(),
            wrapper_args: vec![
                "--exact".to_string(),
                "managed_process::tests::fd_probe_child".to_string(),
                "--ignored".to_string(),
                "--nocapture".to_string(),
                "--test-threads=1".to_string(),
            ],
        };
        let mut command = helper_command("managed_process::tests::bootstrap_child");
        command
            .env(BOOTSTRAP_CHILD_ENV, "1")
            .env(
                BOOTSTRAP_REQUEST_ENV,
                serde_json::to_string(&json!({
                    "instance_id": request.identity.instance_id,
                    "command_id": request.identity.command_id,
                    "generation": request.identity.generation,
                    "wrapper_executable": request.wrapper_executable,
                    "wrapper_args": request.wrapper_args,
                }))
                .unwrap(),
            )
            .env(FD_PROBE_ENV, "1");
        spawn_bootstrap_command(command).unwrap()
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

    fn read_probe(child: &mut ManagedChild) -> serde_json::Value {
        let mut line = String::new();
        child.status_reader().read_line(&mut line).unwrap();
        serde_json::from_str(line.trim_end()).unwrap()
    }

    #[test]
    fn eof_abandonne_tandis_que_release_exec_la_sonde() {
        let mut abandoned = spawn_test_bootstrap();
        let ready = abandoned.wait_ready().unwrap();
        assert_eq!(ready.pid, ready.pgid);
        assert_eq!(ready.birth, process_birth(ready.pid).unwrap());
        abandoned.abandon();
        let status = abandoned.child.wait().unwrap();
        assert_eq!(status.code(), Some(ABANDONED_EXIT_CODE));

        let mut released = spawn_test_bootstrap();
        released.wait_ready().unwrap();
        released.release().unwrap();
        let probe = read_probe(&mut released);
        assert_eq!(probe["event"], "fd_probe");
        assert_eq!(probe["status_open"], true);
        assert_eq!(probe["release_open"], false);
        let status_stream = released.status.into_inner();
        drop(status_stream);
        assert!(released.child.wait().unwrap().success());
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
        let path = store.persist("codex-1", &ready).unwrap();
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
        let mut managed = spawn_test_bootstrap();
        let ready = managed.wait_ready().unwrap();
        let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
        if stage != "before_marker" {
            marker_store.persist("codex-1", &ready).unwrap();
        }
        if stage == "after_release" {
            managed.release().unwrap();
            let probe = read_probe(&mut managed);
            assert_eq!(probe["status_open"], true);
            assert_eq!(probe["release_open"], false);
        }
        write_private_file_atomic(
            &root.join("boundary.json"),
            serde_json::to_string(&ready).unwrap().as_bytes(),
        )
        .unwrap();
        loop {
            thread::park();
        }
    }

    fn wait_for_path(path: &Path) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !path.exists() {
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
            wait_for_path(&boundary);
            let ready: BootstrapReady =
                serde_json::from_slice(&fs::read(&boundary).unwrap()).unwrap();
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
