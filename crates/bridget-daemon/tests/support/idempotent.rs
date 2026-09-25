//! Harnais commun 012/089 : enfants réels isolés, watchdog, sockets et MCP.
//! Déplacé sans modifier les barrières ni les oracles de la matrice historique.

use bridget_core::BridgetMessage;
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::store::Store;
#[cfg(feature = "test-support")]
use bridget_daemon::test_sync::DIRECTORY_ENV;
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    IdentityCredential, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, Once, OnceLock, Weak, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const MATRIX_CYCLES: usize = 50;
// Le wrapper ACP reconnecte volontairement avec un premier délai d'une seconde :
// cinquante redémarrages réels restent donc bornés, sans rendre le banc fragile.
pub const GLOBAL_TIMEOUT: Duration = Duration::from_secs(360);
pub const READY_TIMEOUT: Duration = Duration::from_secs(5);
pub const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(5);
/// Session 123 : une réponse juste peut dépasser 3 s sur une machine à charge
/// 60 ; l'échec faux brouillait chaque recette. Un vrai blocage échoue encore,
/// et GLOBAL_TIMEOUT borne l'ensemble.
pub const CLIENT_READ_TIMEOUT: Duration = Duration::from_secs(30);
pub const SCOPE: &str = "abcdefghijklmnopqrstuv";
pub const RECIPIENT: &str = "96389249-07a4-4e29-83f0-9c46bd775021";
pub const ACTOR: &str = "da78fd70-41e8-424c-a88d-e29e2c5babcd";
pub const MATRIX_AGENT: &str = "c8bf5ed7-7ba4-4193-afc0-ea5c404906c2";
pub const ACP_AGENT: &str = "5da585af-5bc7-4808-985f-c73670633990";

static CHILDREN: OnceLock<Mutex<Vec<i32>>> = OnceLock::new();
static WATCHDOG: Once = Once::new();
/// Session 116 : racines de fixture créées par ce binaire de test. Elles
/// n'étaient jamais supprimées : une recette complète en abandonnait une
/// cinquantaine dans /tmp, plus d'un gigaoctet, jusqu'à remplir le disque.
static ROOTS: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();
static ROOTS_CLEANUP: Once = Once::new();

fn created_roots() -> &'static Mutex<Vec<PathBuf>> {
    ROOTS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Appelée par `exit` à la fin du binaire de test, quand tous ses tests ont
/// rendu leurs processus. Les tests qui effacent déjà leur racine sont sans
/// effet ici. Un fichier encore tenu par un enfant survivant peut rester.
extern "C" fn remove_created_roots() {
    let roots = std::mem::take(
        &mut *created_roots()
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
    );
    for root in roots {
        let _ = fs::remove_dir_all(root);
    }
}
pub fn children() -> &'static Mutex<Vec<i32>> {
    CHILDREN.get_or_init(|| Mutex::new(Vec::new()))
}
pub fn track(child: &Child) {
    children().lock().unwrap().push(child.id() as i32);
}
pub fn untrack(child: &Child) {
    children()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .retain(|pid| *pid != child.id() as i32);
}
pub fn is_owned_running_process(pid: i32) -> bool {
    let Ok(observed) = Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
        .output()
    else {
        return false;
    };
    let command = String::from_utf8_lossy(&observed.stdout);
    // Un PID conservé par erreur ne suffit jamais : le processus doit encore
    // être notre enfant direct, avec notre exécutable et son groupe dédié.
    command
        .split_whitespace()
        .next()
        .and_then(|p| p.parse::<u32>().ok())
        == Some(std::process::id())
        && (command.contains(env!("CARGO_BIN_EXE_bridget"))
            || std::env::current_exe().ok().is_some_and(|path| {
                command.contains(path.to_string_lossy().as_ref())
                    && (command.contains("--exact fixture::performance_daemon_worker")
                        || command.contains("--exact sc005_worker"))
            }))
        && !command.to_ascii_lowercase().contains("firefox")
        && unsafe { libc::getpgid(pid) } == pid
}
pub fn cleanup_child(child: &mut Child) {
    if matches!(child.try_wait(), Ok(Some(_))) {
        untrack(child);
        return;
    }
    let pid = child.id() as i32;
    if is_owned_running_process(pid) {
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if matches!(child.try_wait(), Ok(Some(_))) {
            untrack(child);
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
    // Drop ne panique jamais, même pendant l'échec d'un oracle. Le watchdog
    // conserve ce seul enfant identifié si son état n'a pas pu être récolté.
    eprintln!("nettoyage enfant {pid} non attesté dans la borne");
}
pub fn signal_test_group(child: &mut Child, signal: i32) {
    if child.try_wait().expect("état enfant").is_some() {
        untrack(child);
        return;
    }
    let pid = child.id() as i32;
    assert!(children().lock().unwrap().contains(&pid), "PID non possédé");
    let owned = is_owned_running_process(pid);
    if child
        .try_wait()
        .expect("état après identification")
        .is_some()
    {
        untrack(child);
        return;
    }
    assert!(
        owned,
        "PID {pid} non identifié comme enfant Bridget du test"
    );
    let rc = unsafe { libc::kill(-pid, signal) };
    assert!(rc == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH));
}
pub fn wait_child(child: &mut Child, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while child.try_wait().expect("état enfant").is_none() {
        assert!(
            Instant::now() < deadline,
            "enfant ne termine pas dans {timeout:?}"
        );
        thread::sleep(Duration::from_millis(5)); // watchdog, pas une barrière métier
    }
    untrack(child);
}
pub fn private_dir(path: &Path) -> std::io::Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
}
pub fn private_write(path: &Path, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes.as_ref())
}
pub fn fixture_store(database: &Path) -> Result<Store, bridget_daemon::store::StoreError> {
    private_dir(database.parent().unwrap()).unwrap();
    if !database.exists() {
        private_write(database, []).unwrap();
    }
    Store::open(database)
}
pub fn isolated_command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    command
        .env_clear()
        .env("HOME", root.join("provider"))
        .env("BRIDGET_HOME", root.join("state"))
        .env("BRIDGET_SOCKET", socket(root))
        .env("TMPDIR", root.join("tmp"))
        .env("XDG_CACHE_HOME", root.join("provider/.cache"))
        .env("XDG_CONFIG_HOME", root.join("provider/.config"))
        .env("XDG_DATA_HOME", root.join("provider/.local/share"))
        .env("XDG_STATE_HOME", root.join("provider/.local/state"))
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOSTNAME", "idempotency-isolated")
        .stdin(Stdio::null());
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command
}
pub fn run_isolated(root: &Path, args: &[&str], linked: bool) -> std::process::Output {
    let mut command = isolated_command(root);
    command.args(args);
    if linked {
        let instance = private_fixture_instance(&socket(root), ACTOR)
            .unwrap_or_else(|| "shared-cli-mcp-instance".into());
        command
            .env("BRIDGET_AGENT_ID", ACTOR)
            .env("BRIDGET_AGENT_INSTANCE_ID", instance);
    }
    run_command(command)
}

pub fn run_command(mut command: Command) -> std::process::Output {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = WrapperProcess(command.spawn().expect("CLI réelle"));
    track(&child.0);
    let mut stdout = child.0.stdout.take().unwrap();
    let mut stderr = child.0.stderr.take().unwrap();
    let stdout = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let stderr = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).unwrap();
        bytes
    });
    wait_child(&mut child.0, Duration::from_secs(15));
    std::process::Output {
        status: child.0.wait().unwrap(),
        stdout: stdout.join().unwrap(),
        stderr: stderr.join().unwrap(),
    }
}

pub struct DaemonProcess {
    pub child: Child,
    pub logs: Option<thread::JoinHandle<()>>,
}

/// Possède le daemon de la matrice : une panique de timeout ne laisse jamais
/// l'enfant actif après la fin du banc.
pub struct MatrixDaemonGuard(pub Option<DaemonProcess>);

impl MatrixDaemonGuard {
    pub fn start(root: &Path, sync: &Path) -> Self {
        Self(Some(spawn_daemon(root, Some(sync))))
    }

    pub fn restart(&mut self, root: &Path) {
        self.stop();
        self.0 = Some(spawn_daemon(root, None));
    }

    pub fn restart_with_sync(&mut self, root: &Path, sync: &Path) {
        // Un SIGTERM envoie Disconnect aux wrappers externes et termine leur
        // session : ce réarmement doit aussi être un crash, pas un arrêt poli
        // dont la notification dépendrait d'un sommeil de 20 ms du harnais.
        self.crash();
        self.0 = Some(spawn_daemon(root, Some(sync)));
    }

    pub fn stop(&mut self) {
        if let Some(daemon) = self.0.take() {
            daemon.stop();
        }
    }

    pub fn crash(&mut self) {
        if let Some(daemon) = self.0.take() {
            daemon.crash();
        }
    }
}

impl Drop for MatrixDaemonGuard {
    fn drop(&mut self) {
        // Le Drop non paniquant de DaemonProcess récolte l'enfant même si un
        // oracle échoue pendant un jalon bloqué.
        drop(self.0.take());
    }
}

impl DaemonProcess {
    pub fn stop(mut self) {
        signal_test_group(&mut self.child, libc::SIGTERM);
        let deadline = Instant::now() + Duration::from_secs(3);
        while self.child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        if self.child.try_wait().unwrap().is_none() {
            signal_test_group(&mut self.child, libc::SIGKILL);
        }
        wait_child(&mut self.child, Duration::from_secs(3));
        if let Some(logs) = self.logs.take() {
            let _ = logs.join();
        }
    }

    pub fn crash(mut self) {
        signal_test_group(&mut self.child, libc::SIGKILL);
        wait_child(&mut self.child, Duration::from_secs(3));
        if let Some(logs) = self.logs.take() {
            let _ = logs.join();
        }
    }
}
impl Drop for DaemonProcess {
    fn drop(&mut self) {
        cleanup_child(&mut self.child);
    }
}

pub struct WrapperProcess(pub Child);
impl WrapperProcess {
    pub fn start(root: &Path, registry: &AgentRegistry, agent_id: &str) -> Self {
        // Même wrapper réel que launch_acp_with, désormais dans un processus
        // env_clear : aucun HOME/namespace partagé avec le moteur de tests.
        private_write(
            &root.join("state/agents.json"),
            fs::read(registry.source()).unwrap(),
        )
        .unwrap();
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("wrapper.log"))
            .expect("journal privé du wrapper");
        let child = isolated_command(root)
            .args(["--", "/bin/sh", "--equipier", "--agent-id", agent_id])
            .env("RUST_LOG", "debug")
            .stdout(Stdio::null())
            .stderr(Stdio::from(log))
            .spawn()
            .expect("wrapper réel");
        track(&child);
        Self(child)
    }
    pub fn join(mut self) -> Result<(), String> {
        wait_child(&mut self.0, Duration::from_secs(10));
        let status = self.0.wait().unwrap();
        if status.success() {
            Ok(())
        } else {
            Err(format!("wrapper: {status}"))
        }
    }
}
impl Drop for WrapperProcess {
    fn drop(&mut self) {
        cleanup_child(&mut self.0);
    }
}

pub struct Client {
    pub reader: BufReader<UnixStream>,
    pub writer: BufWriter<UnixStream>,
    /// Les clients concurrents partagent le vrai owner ; le dernier client
    /// libère sa connexion. Aucun owner artificiel n'est créé dans le daemon.
    pub owner: Option<Arc<Client>>,
}

pub struct McpProcess {
    pub child: Child,
    pub input: Option<BufWriter<std::process::ChildStdin>>,
    pub output: BufReader<std::process::ChildStdout>,
}

impl McpProcess {
    pub fn start(root: &Path, name: &str, instance_id: &str) -> Self {
        let name_file = root.join("state/mcp-agent-name");
        private_write(&name_file, name).expect("nom MCP écrit");
        let mut child = isolated_command(root)
            .arg("mcp")
            .env("BRIDGET_AGENT_ID_FILE", &name_file)
            .env("BRIDGET_AGENT_INSTANCE_ID", instance_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("serveur MCP réel démarré");
        track(&child);
        Self {
            input: Some(BufWriter::new(child.stdin.take().expect("stdin MCP"))),
            output: BufReader::new(child.stdout.take().expect("stdout MCP")),
            child,
        }
    }

    pub fn request(&mut self, request: serde_json::Value) -> serde_json::Value {
        writeln!(
            self.input.as_mut().expect("stdin MCP ouvert"),
            "{}",
            serde_json::to_string(&request).expect("requête MCP sérialisable")
        )
        .expect("requête MCP écrite");
        self.input
            .as_mut()
            .unwrap()
            .flush()
            .expect("requête MCP vidée");
        let mut ready = libc::pollfd {
            fd: self.output.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut ready, 1, 15_000) } > 0,
            "réponse MCP absente avant la borne"
        );
        let mut line = String::new();
        self.output
            .read_line(&mut line)
            .expect("réponse MCP lisible");
        serde_json::from_str(&line).expect("réponse MCP JSON")
    }

    pub fn notify(&mut self, notification: serde_json::Value) {
        writeln!(
            self.input.as_mut().expect("stdin MCP ouvert"),
            "{}",
            serde_json::to_string(&notification).expect("notification MCP sérialisable")
        )
        .expect("notification MCP écrite");
        self.input
            .as_mut()
            .unwrap()
            .flush()
            .expect("notification MCP vidée");
    }

    pub fn stop(mut self) {
        drop(self.input.take());
        wait_child(&mut self.child, Duration::from_secs(5));
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        cleanup_child(&mut self.child);
    }
}

impl Client {
    pub fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).expect("connexion au daemon");
        stream
            .set_read_timeout(Some(CLIENT_READ_TIMEOUT))
            .expect("borne lecture client");
        let reader = BufReader::new(stream.try_clone().expect("clone lecture"));
        Self {
            reader,
            writer: BufWriter::new(stream),
            owner: None,
        }
    }

    pub fn send(&mut self, message: WrapperToDaemon) {
        writeln!(
            self.writer,
            "{}",
            encode(&message).expect("message encodable")
        )
        .expect("écriture client");
        self.writer.flush().expect("flush client");
    }

    pub fn receive(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).expect("réponse daemon");
        decode(line.trim_end()).expect("réponse daemon décodable")
    }
}

pub fn test_root(_label: &str) -> PathBuf {
    WATCHDOG.call_once(|| {
        thread::spawn(|| {
            thread::sleep(GLOBAL_TIMEOUT);
            let pids = children()
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone();
            for pid in pids {
                if is_owned_running_process(pid) {
                    unsafe {
                        libc::kill(-pid, libc::SIGKILL);
                    }
                }
            }
            eprintln!("watchdog global idempotency_crash_test: {GLOBAL_TIMEOUT:?}");
            std::process::exit(124);
        });
    });
    let root = PathBuf::from("/tmp").join(format!(
        "bid-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    ));
    for relative in ["", "provider", "state", "tmp"] {
        private_dir(&root.join(relative)).unwrap();
    }
    ROOTS_CLEANUP.call_once(|| unsafe {
        libc::atexit(remove_created_roots);
    });
    created_roots()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .push(root.clone());
    root
}

pub fn socket(root: &Path) -> PathBuf {
    root.join("state/bridget.sock")
}

pub fn make_fifo(path: &Path) {
    let path = CString::new(path.as_os_str().as_bytes()).expect("chemin FIFO sans NUL");
    let result = unsafe { libc::mkfifo(path.as_ptr(), 0o600) };
    assert!(
        result == 0,
        "création FIFO: {}",
        std::io::Error::last_os_error()
    );
}

pub fn checkpoint_root(root: &Path, point: &str) -> (PathBuf, PathBuf) {
    let sync = root.join("sync");
    private_dir(&sync).expect("répertoire de synchronisation");
    make_fifo(&sync.join(format!("{point}.fifo")));
    let marker = sync.join(format!("{point}.ready"));
    (sync, marker)
}

pub fn arm_checkpoint(sync: &Path, points: &[&str], point: &str) {
    for candidate in points {
        let _ = fs::remove_file(sync.join(format!("{candidate}.fifo")));
        let _ = fs::remove_file(sync.join(format!("{candidate}.ready")));
    }
    make_fifo(&sync.join(format!("{point}.fifo")));
}

pub fn spawn_daemon(root: &Path, sync: Option<&Path>) -> DaemonProcess {
    let mut command = isolated_command(root);
    command
        .arg("daemon")
        .env("RUST_LOG", "info")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(feature = "test-support")]
    {
        if let Some(sync) = sync {
            command.env(DIRECTORY_ENV, sync);
        } else {
            command.env_remove(DIRECTORY_ENV);
        }
    }
    // Les tests sans barrière fonctionnent aussi avec le build normal.
    // Ne jamais prétendre armer une barrière absente du binaire testé.
    #[cfg(not(feature = "test-support"))]
    assert!(
        sync.is_none(),
        "une barrière nécessite la feature test-support"
    );
    spawn_daemon_command(command)
}

/// Même boucle daemon, en enfant isolé, avec le seul disjoncteur de charge
/// explicitement dimensionné. Aucun réglage de production ni contournement
/// du résultat d'un Send : la protection normale reste testée ailleurs.
pub fn spawn_performance_daemon(root: &Path) -> DaemonProcess {
    let isolated = isolated_command(root);
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .env_clear()
        .envs(isolated.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))));
    command
        .args([
            "--exact",
            "fixture::performance_daemon_worker",
            "--ignored",
            "--nocapture",
        ])
        .env("BRIDGET_PERFORMANCE_DAEMON", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    spawn_daemon_command(command)
}

#[test]
#[ignore = "sous-processus privé des bancs, jamais un daemon de production"]
fn performance_daemon_worker() {
    assert_eq!(
        std::env::var("BRIDGET_PERFORMANCE_DAEMON").as_deref(),
        Ok("1")
    );
    bridget_daemon::environment::initialize_process().unwrap();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    let config = bridget_daemon::daemon::DaemonConfig {
        circuit_breaker_limit: 10_000,
        ..Default::default()
    };
    bridget_daemon::daemon::run(config).unwrap();
}

fn spawn_daemon_command(mut command: Command) -> DaemonProcess {
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn().expect("spawn daemon réel");
    track(&child);
    let stderr = child.stderr.take().expect("stderr daemon");
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let logs = thread::spawn(move || {
        let mut captured = Vec::new();
        let mut ready_tx = Some(ready_tx);
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            captured.push(line.clone());
            if !line.contains("daemon écoute") {
                continue;
            }
            if let Some(ready_tx) = ready_tx.take() {
                let _ = ready_tx.send(Ok(()));
            }
        }
        if let Some(ready_tx) = ready_tx {
            let _ = ready_tx.send(Err(captured.join("\n")));
        }
    });
    let process = DaemonProcess {
        child,
        logs: Some(logs),
    };
    match ready_rx.recv_timeout(READY_TIMEOUT) {
        Ok(Ok(())) => {}
        Ok(Err(logs)) => panic!("daemon arrêté avant disponibilité: {logs}"),
        Err(_) => panic!("daemon non prêt dans la borne"),
    }
    process
}

/// Attend l'apparition d'un jalon fichier via kqueue (Darwin/BSD).
/// Sous Linux le corps kqueue n'est pas compilé — voir le stub ci-dessous et les
/// `#[cfg_attr(target_os = "linux", ignore = …)]` des tests qui l'appellent.
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub fn watch_marker(directory: &Path, marker: &Path) {
    let directory = CString::new(directory.as_os_str().as_bytes()).expect("répertoire sans NUL");
    let directory_fd = unsafe { libc::open(directory.as_ptr(), libc::O_RDONLY) };
    assert!(
        directory_fd >= 0,
        "ouverture jalon: {}",
        std::io::Error::last_os_error()
    );
    let queue = unsafe { libc::kqueue() };
    assert!(queue >= 0, "kqueue: {}", std::io::Error::last_os_error());
    let change = libc::kevent {
        ident: directory_fd as libc::uintptr_t,
        filter: libc::EVFILT_VNODE,
        flags: libc::EV_ADD | libc::EV_ENABLE | libc::EV_CLEAR,
        fflags: libc::NOTE_WRITE,
        data: 0,
        udata: std::ptr::null_mut(),
    };
    let registered =
        unsafe { libc::kevent(queue, &change, 1, std::ptr::null_mut(), 0, std::ptr::null()) };
    assert_eq!(registered, 0, "inscription kqueue");

    let deadline = Instant::now() + CHECKPOINT_TIMEOUT;
    while !marker.exists() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .expect("jalon absent dans la borne");
        let timeout = libc::timespec {
            tv_sec: remaining.as_secs() as libc::time_t,
            tv_nsec: remaining.subsec_nanos() as libc::c_long,
        };
        let mut event: libc::kevent = unsafe { std::mem::zeroed() };
        let observed = unsafe { libc::kevent(queue, std::ptr::null(), 0, &mut event, 1, &timeout) };
        assert!(observed > 0, "jalon absent dans la borne");
    }
    unsafe {
        libc::close(queue);
        libc::close(directory_fd);
    }
}

/// Stub de liaison hors Darwin/BSD : le fichier doit compiler avec
/// `test-support`, mais aucun test kqueue-dépendant ne doit s'exécuter.
/// Ce n'est PAS un équivalent inotify — hors périmètre SpecKit-030.
///
/// Si cette panique se déclenche, un `#[cfg_attr(target_os = "linux", ignore)]`
/// a été retiré ou un nouvel appel a été ajouté sans la garde — ce n'est
/// pas un flocon du banc, c'est une garde absente.
#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
#[allow(dead_code)]
pub fn watch_marker(_directory: &Path, _marker: &Path) {
    panic!(
        "watch_marker: ce test exige kqueue (Darwin/BSD) et doit être \
         #[ignore] sous Linux via cfg_attr(target_os = \"linux\", ignore = \
         \"exige kqueue…\"). Stub de liaison uniquement — pas un observateur \
         de jalon. Voir la garde kqueue / les cfg_attr ignore dans {file} \
         (stub déclenché à la ligne {line}). Si tu lis ceci, un ignore a été \
         retiré ou un appel a été ajouté sans la garde.",
        file = file!(),
        line = line!()
    );
}

pub fn register_recipient(socket: &Path) -> Client {
    register_recipient_as(socket, "poste-essai-recipient")
}

pub fn register_recipient_as(socket: &Path, instance_id: &str) -> Client {
    register_agent_as(socket, RECIPIENT, instance_id)
}

pub fn register_agent_as(socket: &Path, agent_id: &str, instance_id: &str) -> Client {
    let mut recipient = Client::connect(socket);
    recipient.send(WrapperToDaemon::Register {
        agent_type: "fixture".to_string(),
        identity_version: 2,
        agent_id: agent_id.to_string(),
        host: Some("poste-essai".to_string()),
        transport: Some("acp".to_string()),
        channel: None.into(),
        mode: Some(bridget_transport::protocol::PresenceMode::Acp),
        location: None,
        os: Some("test".to_string()),
        instance_id: Some(instance_id.to_string()),
        domain: None,
        journal_available: None,
        turn_in_progress: false,
    });
    let DaemonToWrapper::Registered {
        agent_id: registered,
        credential: Some(credential),
    } = recipient.receive()
    else {
        panic!("inscription propriétaire avec preuve attendue");
    };
    assert_eq!(registered, agent_id);
    save_fixture_credential(socket, agent_id, instance_id, credential);
    recipient
}

/// Reprend le format privé du wrapper, sans utiliser un raccourci
/// d'autorisation : credential vient obligatoirement du vrai Registered.
pub fn save_fixture_credential(
    socket: &Path,
    agent_id: &str,
    instance_id: &str,
    credential: IdentityCredential,
) {
    let path = socket.parent().unwrap().join("agent-names").join(format!(
        "proof-{:x}.json",
        Sha256::digest(instance_id.as_bytes())
    ));
    bridget_transport::fsutil::write_private_file_atomic(
        &path,
        &serde_json::to_vec(&serde_json::json!({
            "agent_id": agent_id, "instance_id": instance_id, "credential": credential,
        }))
        .unwrap(),
    )
    .unwrap();
}

pub fn attest_agent(socket: &Path, client: &mut Client, agent_id: &str, instance_id: &str) {
    let path = socket.parent().unwrap().join("agent-names").join(format!(
        "proof-{:x}.json",
        Sha256::digest(instance_id.as_bytes())
    ));
    let proof: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("preuve privée émise par le owner")).unwrap();
    assert_eq!(proof["agent_id"], agent_id);
    assert_eq!(proof["instance_id"], instance_id);
    let credential: IdentityCredential =
        serde_json::from_value(proof["credential"].clone()).unwrap();
    client.send(WrapperToDaemon::RegisterAuxiliary {
        agent_id: agent_id.into(),
        instance_id: instance_id.into(),
        credential,
    });
    assert!(
        matches!(client.receive(), DaemonToWrapper::Registered { agent_id: accepted, .. } if accepted == agent_id)
    );
}

/// Retrouve l'incarnation effectivement inscrite par ce scénario dans son
/// répertoire privé ; le fichier le plus récent remplace un ancien owner.
fn private_fixture_instance(socket: &Path, agent_id: &str) -> Option<String> {
    fs::read_dir(socket.parent()?.join("agent-names"))
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let value: serde_json::Value =
                serde_json::from_slice(&fs::read(entry.path()).ok()?).ok()?;
            if value["agent_id"] != agent_id {
                return None;
            }
            Some((
                entry.metadata().ok()?.modified().ok()?,
                value["instance_id"].as_str()?.to_string(),
            ))
        })
        .max_by_key(|entry| entry.0)
        .map(|entry| entry.1)
}

pub fn receive_delivery(recipient: &mut Client) -> DaemonToWrapper {
    match recipient.receive() {
        delivery @ DaemonToWrapper::DeliverIdempotent { .. } => delivery,
        other => panic!("remise idempotente attendue: {other:?}"),
    }
}

pub fn assert_no_delivery(recipient: &mut Client) {
    recipient
        .reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("borne de lecture");
    let mut line = String::new();
    let result = recipient.reader.read_line(&mut line);
    assert!(
        matches!(
            result,
            Err(ref error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                )
        ),
        "une instance remplacée ne doit jamais recevoir une remise figée: {result:?}"
    );
}

pub fn negotiate_client(socket: &Path) -> Client {
    // Sérialiser seulement l'établissement du owner afin que les huit clients
    // du test de concurrence ne remplacent pas mutuellement son incarnation.
    static OWNERS: OnceLock<Mutex<BTreeMap<PathBuf, Weak<Client>>>> = OnceLock::new();
    let mut owners = OWNERS
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .unwrap();
    let mut probe = Client::connect(socket);
    probe.send(WrapperToDaemon::ListAgents);
    let DaemonToWrapper::AgentList { agents } = probe.receive() else {
        panic!("annuaire attendu");
    };
    let actor_online = agents.iter().any(|agent| {
        agent.agent_id == ACTOR && matches!(agent.state.as_str(), "connected" | "busy")
    });
    drop(probe);
    let owner = if actor_online {
        owners.get(socket).and_then(Weak::upgrade)
    } else {
        let owner = Arc::new(register_agent_as(socket, ACTOR, "shared-cli-mcp-instance"));
        owners.insert(socket.to_path_buf(), Arc::downgrade(&owner));
        Some(owner)
    };
    // Un owner déclaré par le scénario peut employer une autre incarnation.
    // L'annuaire public ne publie pas la preuve : on lit le fichier privé
    // effectivement enregistré par la fixture, jamais une preuve inventée.
    let instance_id =
        private_fixture_instance(socket, ACTOR).expect("preuve privée du owner ACTOR");
    let mut client = Client::connect(socket);
    client.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    client.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: SCOPE.to_string(),
        capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    attest_agent(socket, &mut client, ACTOR, &instance_id);
    client.owner = owner;
    client
}

pub fn idempotent_send(message_id: String, issued_at: i64) -> WrapperToDaemon {
    let mut message = BridgetMessage::new(ACTOR, RECIPIENT, "crash matrix");
    message.id = message_id.clone();
    WrapperToDaemon::SendIdempotent {
        message,
        message_id,
        issued_at,
    }
}

pub fn retry_issue(socket: &Path, message_id: String, issued_at: i64) -> IdempotencyIssue {
    retry_command_issue(socket, idempotent_send(message_id, issued_at))
}

pub fn retry_command_issue(socket: &Path, command: WrapperToDaemon) -> IdempotencyIssue {
    let mut client = negotiate_client(socket);
    client.send(command);
    match client.receive() {
        DaemonToWrapper::IdempotencyResult { issue, .. } => issue,
        other => panic!("issue idempotente attendue: {other:?}"),
    }
}

pub fn registry_with_counting_acp_agent(prompt_limit: usize) -> (AgentRegistry, PathBuf, PathBuf) {
    let directory = test_root("acp-registry");
    private_dir(&directory).expect("répertoire du registre ACP");
    let counter = directory.join("session-prompt-count");
    let definition = serde_json::json!({
        "agents": {
            "fixture-acp": {
                "command": "/bin/sh",
                "args": ["-c", format!(r#"
read initialize
echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":1}}}}'
read new_session
echo '{{"jsonrpc":"2.0","id":2,"result":{{"sessionId":"fixture-acp"}}}}'
count=0
while IFS= read -r prompt; do
  printf x >> '{}'
  request_id=$(printf '%s' "$prompt" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  echo "{{\"jsonrpc\":\"2.0\",\"id\":${{request_id}},\"result\":{{\"stopReason\":\"end_turn\"}}}}"
  count=$((count + 1))
  [ "$count" -ge {} ] && break
done
"#, counter.display(), prompt_limit)],
                "protocol": "acp",
                "permissions": "allow",
                "queue_capacity": 2,
                // Ce banc mesure la reprise et l'unicité, pas l'expiration
                // d'une mission. Le délai de reconnexion est déjà de 1 s ;
                // un TTL de 1 s, tronqué en secondes, peut donc expirer avant
                // le prompt. Les watchdogs de jalon/prompt restent à 5 s.
                "notify_timeout_secs": 30
            }
        }
    });
    let path = directory.join("agents.json");
    private_write(
        &path,
        serde_json::to_string_pretty(&definition).expect("registre sérialisable"),
    )
    .expect("registre ACP écrit");
    let registry = AgentRegistry::from_json(
        &fs::read_to_string(&path).expect("registre ACP lisible"),
        &path,
    )
    .expect("registre ACP valide");
    (registry, directory, counter)
}

pub fn wait_for_counter(counter: &Path, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read(counter).map_or(0, |bytes| bytes.len()) < expected {
        assert!(
            Instant::now() < deadline,
            "frame session/prompt absente dans la borne"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn wait_for_registered_agent(socket: &Path, name: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut probe = Client::connect(socket);
        probe.send(WrapperToDaemon::ListAgents);
        if matches!(
            probe.receive(),
            DaemonToWrapper::AgentList { agents } if agents.iter().any(|agent| agent.agent_id == name)
        ) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "wrapper ACP non enregistré dans la borne"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn wait_for_accepted(socket: &Path, command: &WrapperToDaemon) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let issue = retry_command_issue(socket, command.clone());
        if matches!(issue, IdempotencyIssue::Accepted { .. }) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("Accepted absent dans la borne, dernière issue: {issue:?}");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn issued_at() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64
}

pub fn run_linked_cli(root: &Path, args: &[&str]) -> std::process::Output {
    run_isolated(root, args, true)
}

pub fn output_text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn output_field(output: &std::process::Output, field: &str) -> String {
    let prefix = format!("{field}=");
    output_text(output)
        .split_whitespace()
        .find_map(|token| token.strip_prefix(&prefix))
        .map(|value| value.trim_end_matches(':').to_string())
        .unwrap_or_else(|| panic!("champ {field} absent de la sortie: {}", output_text(output)))
}

pub fn mcp_send_call(
    request_id: i64,
    message_id: &str,
    sent_at: i64,
    body: &str,
) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": "bridget_send",
            "arguments": {
                "to": RECIPIENT,
                "body": body,
                "in_reply_to": "request-open",
                "id": message_id,
                "issued_at": sent_at
            }
        }
    })
}
