#![allow(
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_if,
    unused_mut,
    non_snake_case
)]
use bridget_core::BridgetMessage;
// 090 : le sous-harnais Codex/faux-tmux (prompt et reprise injectés dans argv)
// est remplacé par codex_interactive_090_test : vraie TUI, même fil et MCP.
// Ce fichier conserve le corpus transport géré/terminal ACP et ses 4 messages.
// La clôture MCP/canon/identité reste gardée automatiquement par core_089_*.
use bridget_daemon::managed_process::{ManagedMarkerStore, group_exists};
use bridget_transport::protocol::{AgentInfo, AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MATRIX_VERSION: &str = "fr-008-v1";
const MATRIX_RUNS_PER_MODE: usize = 3;
const MATRIX_EXPECTED_TURNS: usize = 4;
/// turn_start + update(text) + reasoning(terminal) + turn_end — L4 C3.
const MATRIX_EVENTS_PER_TURN: usize = 4;
const MATRIX_TIMEOUT: Duration = Duration::from_secs(10);
// Seul QUEUE-SLOW doit franchir T/3 et prouver la relance différée. Sous
// contention, appliquer le même délai court aux tours nominaux injectait un
// rappel légitime dans le compteur du faux adaptateur et créait un 5e tour.
const MATRIX_FAST_REPLY_TIMEOUT_SECS: u64 = 30;
const MATRIX_SLOW_REPLY_TIMEOUT_SECS: u64 = 5;
const FROZEN_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";
static MANAGED_BENCH_LOCK: Mutex<()> = Mutex::new(());

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Les scénarios gardent des libellés lisibles, mais le protocole ne transporte
/// plus ces libellés comme identités de routage.
fn agent_id_for(label: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    let value = hasher.finish();
    format!(
        "{:08x}-0000-4000-8000-{:012x}",
        value as u32,
        value & 0x0000_0fff_ffff_ffff
    )
}

fn test_root(label: &str) -> PathBuf {
    let root = PathBuf::from(format!(
        "/tmp/bg909-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    bridget_daemon::environment::Namespace::resolve(
        Some(root.join("state")),
        None,
        Some(root.clone()),
    )
    .unwrap()
    .prepare()
    .unwrap();
    root
}

fn write_fixture(root: &Path) -> PathBuf {
    let adapter = root.join("parity-acp.py");
    fs::create_dir_all(root.join("state")).unwrap();
    fs::write(
        &adapter,
        r#"#!/usr/bin/python3
import json
import os
import sys
import time

turn = 0
for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"protocolVersion":1}}), flush=True)
    elif method == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"sessionId":"parity-session"}}), flush=True)
    elif method == "session/prompt":
        prompt = request["params"]["prompt"][0]["text"]
        if "Carte de reprise Bridget (faits durables, aucune mémoire reconstruite)." in prompt:
            print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"stopReason":"end_turn"}}), flush=True)
            continue
        turn += 1
        if "QUEUE-SLOW" in prompt:
            time.sleep(2.2)
        update = {
            "jsonrpc":"2.0",
            "method":"session/update",
            "params":{
                "sessionId":"parity-session",
                "update":{
                    "sessionUpdate":"agent_message_chunk",
                    "content":{"type":"text","text":f"fixture-response-{turn}"}
                }
            }
        }
        print(json.dumps(update), flush=True)
        print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"stopReason":"end_turn"}}), flush=True)
        if os.environ.get("PARITY_SINGLE_TURN") == "1":
            break
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let registry = serde_json::json!({
        "agents": {
            "parity": {
                "command": adapter,
                "protocol": "acp",
                "permissions": "allow",
                "queue_capacity": 8,
                "notify_timeout_secs": 6,
                "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
                "pass_env": ["PARITY_SINGLE_TURN"]
            }
        }
    });
    fs::write(
        root.join("state/agents.json"),
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
    fs::set_permissions(
        root.join("state/agents.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    // Précondition privée du scénario : ce fournisseur synthétique n'offre
    // pas de mode découverte. La garde humaine reste testée par le gate 089.
    let database = root.join("state/bridget.db");
    drop(bridget_daemon::store::Store::open(&database).unwrap());
    let connection = rusqlite::Connection::open(&database).unwrap();
    let initial = bridget_daemon::referent_control::read(&connection).unwrap();
    bridget_daemon::referent_control::set(
        &connection,
        bridget_daemon::referent_control::ControlMutation {
            command_id: "parity-fixture-complete",
            expected_generation: initial.generation,
            paused: None,
            auto_objectives_cap: None,
            reason: None,
            actor: "test",
            now: unix_now(),
            agent_posture: Some(bridget_transport::protocol::AgentPosture::Complete),
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    adapter
}

/// Un échec d'oracle ne doit pas laisser le wrapper de test en reconnexion.
struct WrapperChild(Option<Child>, Option<PathBuf>);

impl std::ops::Deref for WrapperChild {
    type Target = Child;
    fn deref(&self) -> &Child {
        self.0.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for WrapperChild {
    fn deref_mut(&mut self) -> &mut Child {
        self.0.as_mut().unwrap()
    }
}

impl Drop for WrapperChild {
    fn drop(&mut self) {
        if let Some(release) = &self.1 {
            let _ = fs::write(release, b"release");
        }
        stop_daemon_child_best_effort(self.0.as_mut());
    }
}

fn write_cached_npx_fixture(root: &Path) {
    let adapter = write_fixture(root);
    let package = root.join("node_modules/parity-acp");
    let binaries = root.join("node_modules/.bin");
    fs::create_dir_all(&package).unwrap();
    fs::create_dir_all(&binaries).unwrap();
    let package_adapter = package.join("index.py");
    fs::copy(adapter, &package_adapter).unwrap();
    fs::set_permissions(&package_adapter, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        package.join("package.json"),
        br#"{"name":"parity-acp","version":"1.0.0","bin":{"parity-acp":"index.py"}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink("../parity-acp/index.py", binaries.join("parity-acp")).unwrap();
    let registry_path = root.join("state/agents.json");
    let mut registry: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
    registry["agents"]["parity"]["command"] = serde_json::json!("npx");
    registry["agents"]["parity"]["args"] =
        serde_json::json!(["--offline", "--no-install", "parity-acp"]);
    fs::write(
        &registry_path,
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();
}

/// Possède le daemon de parité. La garde existe avant le spawn : un échec
/// d'amorçage ne laisse jamais l'enfant sous PID 1. `Drop` ne panique jamais.
struct DaemonProcess {
    child: Option<Child>,
    socket: PathBuf,
}

impl DaemonProcess {
    fn start(root: &Path, billing_key: bool, single_turn: bool) -> Self {
        // Garde créée avant le spawn : le chemin d'erreur précoce (assert prêt,
        // panique injectée) libère encore le processus via Drop.
        let mut process = Self {
            child: None,
            socket: root.join("state/bridget.sock"),
        };
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", root)
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if billing_key {
            command.env("OPENAI_API_KEY", "forbidden-test-key");
        }
        if single_turn {
            command.env("PARITY_SINGLE_TURN", "1");
        }
        process.child = Some(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut accepting = false;
        while Instant::now() < deadline {
            if UnixStream::connect(&process.socket).is_ok() {
                accepting = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            accepting,
            "le daemon de parité n'accepte pas encore les connexions"
        );
        process
    }

    fn stop(mut self) {
        let stopped = stop_daemon_child_best_effort(self.child.as_mut());
        if stopped {
            let _ = self.child.take();
            return;
        }
        // L'enfant reste dans Option pour que Drop retente sans paniquer pendant
        // le dépliage ; l'assertion explicite reste sur le chemin stop() heureux.
        panic!("le daemon de parité n'a pas terminé après SIGTERM");
    }

    fn kill(mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = unsafe { libc::kill(child.id() as i32, libc::SIGKILL) };
            let _ = child.wait();
        }
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        let _ = stop_daemon_child_best_effort(self.child.as_mut());
    }
}

/// Arrêt best-effort : jamais de panique (y compris pendant un dépliage).
fn stop_daemon_child_best_effort(child: Option<&mut Child>) -> bool {
    let Some(child) = child else {
        return true;
    };
    match child.try_wait() {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => {}
    }
    let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => break,
        }
    }
    let _ = child.kill();
    child.wait().is_ok()
}

/// Compte les daemons dont le HOME est exactement celui de CE test.
/// Un compteur borné au PID du harnais croise les voisins parallèles.
fn daemon_count_for_home(home: &Path) -> usize {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()
        .expect("ps pour l'oracle de non-fuite");
    assert!(output.status.success(), "ps indisponible pour l'oracle");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (pid, command) = line.split_once(char::is_whitespace)?;
            let command = command.trim_start();
            let mut parts = command.split_whitespace();
            let binary = parts.next()?;
            let argv1 = parts.next()?;
            if !binary.contains("bridget") || argv1 != "daemon" || parts.next().is_some() {
                return None;
            }
            Some(pid.trim())
        })
        .filter(|pid| daemon_home_is(pid, home))
        .count()
}

fn daemon_home_is(pid: &str, home: &Path) -> bool {
    let output = Command::new("lsof").args(["-p", pid, "-Fn"]).output();
    let Ok(output) = output else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix('n'))
        .any(|path| path_is_under_home(path, home))
}

fn path_is_under_home(path: &str, home: &Path) -> bool {
    let home = home.to_string_lossy();
    path == home.as_ref() || path.starts_with(&format!("{home}/"))
}

fn assert_daemon_count_for_home(home: &Path, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if daemon_count_for_home(home) == expected {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        daemon_count_for_home(home),
        expected,
        "daemon orphelin pour le HOME du test: {}",
        home.display()
    );
}

struct CutProxy {
    socket: PathBuf,
    latest_wrapper: Arc<Mutex<Option<(UnixStream, UnixStream)>>>,
    wrapper_connections: Arc<AtomicUsize>,
    stopping: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CutProxy {
    fn start(socket: PathBuf, target: PathBuf) -> Self {
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let latest_wrapper = Arc::new(Mutex::new(None));
        let wrapper_connections = Arc::new(AtomicUsize::new(0));
        let stopping = Arc::new(AtomicBool::new(false));
        let observed_wrapper = Arc::clone(&latest_wrapper);
        let observed_connections = Arc::clone(&wrapper_connections);
        let observed_stop = Arc::clone(&stopping);
        let handle = thread::spawn(move || {
            while !observed_stop.load(Ordering::SeqCst) {
                let client = match listener.accept() {
                    Ok((client, _)) => client,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("accept du proxy impossible: {error}"),
                };
                client.set_nonblocking(false).unwrap();
                let target_stream = connect_target_within_bound(&target);
                target_stream.set_nonblocking(false).unwrap();
                let current_wrapper = Arc::clone(&observed_wrapper);
                let connection_count = Arc::clone(&observed_connections);
                thread::spawn(move || {
                    let mut client_reader = BufReader::new(client.try_clone().unwrap());
                    let mut target_writer = target_stream.try_clone().unwrap();
                    let mut first_line = String::new();
                    if client_reader.read_line(&mut first_line).unwrap() == 0 {
                        return;
                    }
                    target_writer.write_all(first_line.as_bytes()).unwrap();
                    target_writer.flush().unwrap();
                    let is_wrapper = matches!(
                        decode::<WrapperToDaemon>(first_line.trim_end()),
                        Ok(WrapperToDaemon::Register { ref agent_type, .. })
                            if agent_type == "parity" || agent_type == "codex"
                    );
                    if is_wrapper {
                        *current_wrapper
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner()) = Some((
                            client.try_clone().unwrap(),
                            target_stream.try_clone().unwrap(),
                        ));
                        connection_count.fetch_add(1, Ordering::SeqCst);
                    }
                    let mut target_reader = target_stream.try_clone().unwrap();
                    let mut client_writer = client.try_clone().unwrap();
                    let reverse = thread::spawn(move || {
                        let _ = std::io::copy(&mut target_reader, &mut client_writer);
                        let _ = client_writer.shutdown(std::net::Shutdown::Both);
                    });
                    let _ = std::io::copy(&mut client_reader, &mut target_writer);
                    let _ = target_writer.shutdown(std::net::Shutdown::Both);
                    let _ = reverse.join();
                });
            }
        });
        Self {
            socket,
            latest_wrapper,
            wrapper_connections,
            stopping,
            handle: Some(handle),
        }
    }

    fn wrapper_connection_count(&self) -> usize {
        self.wrapper_connections.load(Ordering::SeqCst)
    }

    fn cut_wrapper_and_wait_for_reconnect(&self) {
        let before = self.wrapper_connection_count();
        let sockets = self
            .latest_wrapper
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
            .expect("connexion wrapper à couper");
        shutdown_for_cut(&sockets.0);
        shutdown_for_cut(&sockets.1);
        let deadline = Instant::now() + MATRIX_TIMEOUT;
        while self.wrapper_connection_count() == before && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            self.wrapper_connection_count(),
            before + 1,
            "le wrapper ne s'est pas reconnecté après la coupure réelle"
        );
    }

    fn stop(mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        let _ = fs::remove_file(&self.socket);
    }
}

/// La moitié du proxy peut déjà avoir atteint EOF quand l'autre la coupe.
/// `NotConnected` confirme alors la même coupure et ne rend pas la matrice
/// moins stricte : la reconnexion suivante reste attendue et contrôlée.
fn shutdown_for_cut(stream: &UnixStream) {
    match stream.shutdown(std::net::Shutdown::Both) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotConnected => {}
        Err(error) => panic!("coupure du proxy impossible: {error}"),
    }
}

/// Le fichier socket du daemon existe avant que son écoute soit atomiquement
/// disponible. Le proxy de test attend donc cette disponibilité bornée au lieu
/// de transformer ce démarrage concurrent en `ConnectionRefused` aléatoire.
fn connect_target_within_bound(target: &Path) -> UnixStream {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        match UnixStream::connect(target) {
            Ok(stream) => return stream,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
                ) && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("connexion du proxy au daemon impossible: {error}"),
        }
    }
}

fn start_daemon_behind_proxy(root: &Path) -> (DaemonProcess, CutProxy) {
    let daemon = DaemonProcess::start(root, false, false);
    let target = root.join("state/backend.sock");
    let database = root.join("state/bridget.db");
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while !daemon_ledger_is_ready(&database) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        daemon_ledger_is_ready(&database),
        "le daemon n'a pas achevé l'initialisation de son ledger"
    );
    // Déplacer uniquement l'entrée de socket du listener prêt. Le namespace
    // et sa base ne changent jamais ; aucun symlink ni HOME global. Les vrais
    // wrappers héritent du socket public, désormais servi par le proxy.
    fs::rename(&daemon.socket, &target).unwrap();
    let proxy = CutProxy::start(daemon.socket.clone(), target);
    (daemon, proxy)
}

/// Le fichier SQLite existe dès `Connection::open`, avant les DDL du store.
/// La bascule du proxy attend donc l'observable utile pour le premier Send.
fn daemon_ledger_is_ready(database: &Path) -> bool {
    let flags =
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let Ok(connection) = rusqlite::Connection::open_with_flags(database, flags) else {
        return false;
    };
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'ledger'",
            [],
            |_| Ok(()),
        )
        .is_ok()
}

struct Peer {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
    name: String,
}

impl Peer {
    fn register(socket: &Path, name: &str) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream.set_read_timeout(Some(MATRIX_TIMEOUT)).unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream);
        let mut peer = Self {
            reader,
            writer,
            name: name.to_string(),
        };
        peer.send(&WrapperToDaemon::Register {
            identity_version: 2,
            agent_type: "parity-client".to_string(),
            agent_id: agent_id_for(name),
            host: Some("fixture-host".to_string()),
            transport: Some("unix".to_string()),
            channel: None.into(),
            mode: Some(bridget_transport::protocol::PresenceMode::Acp),
            location: None,
            os: Some("fixture-os".to_string()),
            instance_id: Some(format!("instance-{name}")),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        });
        match peer.recv() {
            DaemonToWrapper::Registered {
                agent_id: assigned, ..
            } => peer.name = assigned,
            other => panic!("enregistrement inattendu: {other:?}"),
        }
        peer
    }

    fn attach(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream.set_read_timeout(Some(MATRIX_TIMEOUT)).unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream);
        let mut peer = Self {
            reader,
            writer,
            name: "attach".to_string(),
        };
        peer.send(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        });
        assert!(matches!(
            peer.recv(),
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            }
        ));
        peer
    }

    fn send(&mut self, message: &WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn recv(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        assert!(!line.is_empty(), "EOF inattendu depuis le daemon");
        decode(line.trim_end()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModeObservables {
    replies: Vec<String>,
    request_states: Vec<String>,
    agent_transport: String,
    agent_state: String,
    journal: Vec<String>,
}

fn wait_agent(control: &mut Peer, name: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        control.send(&WrapperToDaemon::ListAgents);
        if let DaemonToWrapper::AgentList { agents } = control.recv()
            && let Some(agent) = agents
                .into_iter()
                .find(|agent| agent.agent_id == agent_id_for(name))
        {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "l'équipier {name} ne s'est pas enregistré"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_busy_reconnected(control: &mut Peer, name: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        let agent = wait_agent(control, name);
        if agent.state == "busy" && agent.reconnect_count >= 1 {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "{name} n'a pas conservé busy après sa reconnexion"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_agent_state(control: &mut Peer, name: &str, expected: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        let agent = wait_agent(control, name);
        if agent.state == expected {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "{name} n'a pas atteint l'état {expected}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn send_tracked(peer: &mut Peer, to: &str, body: &str) -> String {
    send_tracked_with_timeout(peer, to, body, 5)
}

fn send_tracked_with_timeout(peer: &mut Peer, to: &str, body: &str, timeout: u64) -> String {
    let mut message = BridgetMessage::new(&peer.name, agent_id_for(to), body);
    message.reply = true;
    message.reply_timeout = Some(timeout);
    let id = message.id.clone();
    peer.send(&WrapperToDaemon::Send(message));
    match peer.recv() {
        DaemonToWrapper::Ack { id: acked } => assert_eq!(acked, id),
        other => panic!("accusé inattendu: {other:?}"),
    }
    id
}

fn receive_replies(peer: &mut Peer, expected_ids: &[String]) -> Vec<String> {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    let mut replies = Vec::new();
    while replies.len() < expected_ids.len() {
        assert!(Instant::now() < deadline, "réponses ACP incomplètes");
        match peer.recv() {
            DaemonToWrapper::Deliver(message) => {
                let in_reply_to = message.in_reply_to.as_deref().unwrap_or_default();
                if in_reply_to.is_empty() {
                    assert_eq!(
                        message.from, "bridget",
                        "message non corrélé inattendu pendant l'attente d'une réponse: {message:?}"
                    );
                    assert!(
                        message
                            .body
                            .starts_with("Échec de livraison de la demande #"),
                        "notification Bridget non reconnue pendant l'attente d'une réponse: {message:?}"
                    );
                } else {
                    assert_eq!(
                        in_reply_to,
                        expected_ids[replies.len()],
                        "réponse corrélée à une autre demande: {message:?}"
                    );
                    replies.push(message.body);
                }
            }
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                message,
                ..
            } => {
                let in_reply_to = message.in_reply_to.as_deref().unwrap_or_default();
                assert_eq!(
                    in_reply_to,
                    expected_ids[replies.len()],
                    "réponse idempotente sans corrélation exploitable: {message:?}"
                );
                replies.push(message.body);
                peer.send(&WrapperToDaemon::DeliverAcked {
                    delivery_id,
                    delivery_generation,
                });
            }
            _ => {}
        }
    }
    replies
}

fn normalized_entry(bytes: &[u8]) -> String {
    let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let event = value["event"].as_str().unwrap();
    let payload = &value["payload"];
    match event {
        "turn_start" => format!(
            "turn_start|reply={}|body={}",
            payload["reply"].as_bool().unwrap(),
            payload["body"].as_str().unwrap()
        ),
        "update" => format!("update|{}", payload["text"].as_str().unwrap_or_default()),
        "reasoning" => format!(
            "reasoning|available={}",
            payload["available"].as_bool().unwrap_or(false)
        ),
        "turn_end" => format!(
            "turn_end|{}|routed={}",
            payload["stop_reason"].as_str().unwrap_or_default(),
            payload.get("routed_to").is_some()
        ),
        other => format!("{other}|{payload}"),
    }
}

fn collect_journal(socket: &Path, agent: &str) -> Vec<String> {
    let mut attach = Peer::attach(socket);
    attach.send(&WrapperToDaemon::Subscribe {
        agent: agent_id_for(agent),
        window: AttachWindow::Seq(0),
    });
    let mut fragments: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut complete = BTreeMap::new();
    let mut caught_up = false;
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while (!caught_up || complete.len() < MATRIX_EXPECTED_TURNS * MATRIX_EVENTS_PER_TURN)
        && Instant::now() < deadline
    {
        match attach.recv() {
            DaemonToWrapper::Subscribed { .. } => {}
            DaemonToWrapper::JournalFragment {
                seq,
                offset,
                final_fragment,
                bytes,
                ..
            } => {
                let current = fragments.entry(seq).or_default();
                assert_eq!(
                    offset as usize,
                    current.len(),
                    "offset non contigu pour seq {seq}"
                );
                current.extend(bytes);
                if final_fragment {
                    complete.insert(seq, fragments.remove(&seq).unwrap());
                }
            }
            DaemonToWrapper::SnapshotCaughtUp { .. } => caught_up = true,
            DaemonToWrapper::Gap {
                from_seq, to_seq, ..
            } => {
                panic!("Gap inattendu dans le corpus de parité: {from_seq}..={to_seq}")
            }
            other => panic!("frame attach inattendue: {other:?}"),
        }
    }
    assert!(caught_up, "SnapshotCaughtUp absent");
    let bootstrap_message_ids = complete
        .values()
        .filter_map(|bytes| {
            let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
            (value["event"] == "turn_start" && value["payload"]["from"] == "bridget-reprise")
                .then(|| value["message_id"].as_str().map(str::to_string))
                .flatten()
        })
        .collect::<BTreeSet<_>>();
    let business: Vec<Vec<u8>> = complete
        .into_values()
        .filter(|bytes| {
            let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            !value["message_id"]
                .as_str()
                .is_some_and(|message_id| bootstrap_message_ids.contains(message_id))
        })
        .collect();
    if business.len() != MATRIX_EXPECTED_TURNS * MATRIX_EVENTS_PER_TURN {
        // Diagnostic : en cas d'écart, inventorier les tours métier reçus pour
        // situer un tour manquant ou rejoué (sous charge, un tour en trop a été vu).
        for bytes in &business {
            let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            eprintln!(
                "DIAG event={} message_id={} from={} body={}",
                value["event"],
                value["message_id"],
                value["payload"]["from"],
                value["payload"]["body"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(30)
                    .collect::<String>()
            );
        }
    }
    assert_eq!(
        business.len(),
        MATRIX_EXPECTED_TURNS * MATRIX_EVENTS_PER_TURN,
        "la fixture doit produire {MATRIX_EVENTS_PER_TURN} événements par tour métier (start+text+reasoning+end), hors carte de reprise"
    );
    // Preuve d'unicité : exactement un event=reasoning par message_id, available=false
    // (fixture parity sans thought_chunk — cas Gemini).
    let mut reasoning_per_message = BTreeMap::<String, usize>::new();
    for bytes in &business {
        let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        if value["event"] == "reasoning" {
            let message_id = value["message_id"]
                .as_str()
                .expect("reasoning sans message_id")
                .to_string();
            *reasoning_per_message.entry(message_id).or_default() += 1;
            assert_eq!(
                value["payload"],
                serde_json::json!({ "available": false }),
                "reasoning terminal sans pensée = available:false, sans summary/raw"
            );
        }
    }
    assert_eq!(
        reasoning_per_message.len(),
        MATRIX_EXPECTED_TURNS,
        "un message_id métier sans reasoning terminal: {reasoning_per_message:?}"
    );
    assert!(
        reasoning_per_message.values().all(|count| *count == 1),
        "doublon event=reasoning sur un tour: {reasoning_per_message:?}"
    );
    business
        .iter()
        .map(|bytes| normalized_entry(bytes))
        .collect()
}

fn run_corpus(socket: &Path, agent: &str, run: usize, proxy: &CutProxy) -> ModeObservables {
    let mut peer = Peer::register(socket, &format!("parity-sender-{run}"));
    // Quickstart 007 §1 : l'équipier est visible en ACP, prêt à recevoir.
    // Attendre connected : la carte de reprise peut encore tourner (busy) —
    // L4 ajoute un event=reasoning qui allonge cette fenêtre.
    let initial_agent = wait_agent_state(&mut peer, agent, "connected");
    assert_eq!(initial_agent.transport, "acp");
    assert_eq!(initial_agent.state, "connected");

    // Quickstart 007 §2 : une demande suivie reçoit sa réponse et se clôt.
    let first =
        send_tracked_with_timeout(&mut peer, agent, "TRACKED", MATRIX_FAST_REPLY_TIMEOUT_SECS);
    let mut replies = receive_replies(&mut peer, &[first]);
    // Quickstart 007 §3 : le corps riche traverse le transport octet pour octet.
    let exact = "l'apostrophe d'usage, \"guillemets\", $VAR, `backticks`,\net ce saut de ligne.";
    let second = send_tracked_with_timeout(&mut peer, agent, exact, MATRIX_FAST_REPLY_TIMEOUT_SECS);
    replies.extend(receive_replies(&mut peer, &[second]));

    // Quickstart 007 §4 : FIFO pendant un tour, relance différée et
    // reconnexion conservant l'état busy.
    let slow = send_tracked_with_timeout(
        &mut peer,
        agent,
        "QUEUE-SLOW",
        MATRIX_SLOW_REPLY_TIMEOUT_SECS,
    );
    let next = send_tracked_with_timeout(
        &mut peer,
        agent,
        "QUEUE-NEXT",
        MATRIX_FAST_REPLY_TIMEOUT_SECS,
    );
    // La remise est asynchrone par destinataire (spec 099) : l'état busy
    // s'observe en attendant, pas en lisant l'annuaire juste après l'envoi.
    let busy = wait_agent_state(&mut peer, agent, "busy");
    assert_eq!(busy.state, "busy", "le tour lent doit être observable");
    proxy.cut_wrapper_and_wait_for_reconnect();
    let reconnected = wait_busy_reconnected(&mut peer, agent);
    assert_eq!(reconnected.transport, "acp");
    replies.extend(receive_replies(&mut peer, &[slow.clone(), next]));
    let final_agent = wait_agent_state(&mut peer, agent, "connected");
    assert_eq!(
        replies,
        vec![
            "fixture-response-1",
            "fixture-response-2",
            "fixture-response-3",
            "fixture-response-4"
        ]
    );

    peer.send(&WrapperToDaemon::ListRequests {
        sender: peer.name.clone(),
        limit: 200,
    });
    let request_states: Vec<String> = match peer.recv() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), MATRIX_EXPECTED_TURNS);
            let slow_request = requests
                .iter()
                .find(|request| request.id == slow)
                .expect("demande lente absente du ledger");
            assert!(
                slow_request.deferred_reminder_level.is_some(),
                "la relance différée doit être consignée pendant le tour busy"
            );
            requests.into_iter().map(|request| request.state).collect()
        }
        other => panic!("liste de demandes inattendue: {other:?}"),
    };
    assert!(request_states.iter().all(|state| state == "answered"));

    let journal = collect_journal(socket, agent);
    let turn_starts = journal
        .iter()
        .filter(|entry| entry.starts_with("turn_start|"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        turn_starts,
        vec![
            "turn_start|reply=true|body=TRACKED".to_string(),
            format!("turn_start|reply=true|body={exact}"),
            "turn_start|reply=true|body=QUEUE-SLOW".to_string(),
            "turn_start|reply=true|body=QUEUE-NEXT".to_string(),
        ],
        "l'oracle de corps doit détecter une perte commune aux deux modes"
    );

    ModeObservables {
        replies,
        request_states,
        agent_transport: final_agent.transport,
        agent_state: final_agent.state,
        journal,
    }
}

fn stop_managed(control: &mut Peer, name: &str, run: usize) {
    control.send(&WrapperToDaemon::StopOrder {
        agent_id: agent_id_for(name),
        command_id: format!("stop-parity-{run}"),
    });
    let response = control.recv();
    assert!(
        matches!(
            response,
            DaemonToWrapper::StopResult {
                outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
                ..
            }
        ),
        "arrêt géré inattendu pour {name}: {response:?}"
    );
}

fn spawn_managed(control: &mut Peer, root: &Path, name: &str, command_id: &str, persistent: bool) {
    let now = unix_now();
    control.send(&WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "parity".to_string(),
        agent_id: Some(agent_id_for(name)),
        cwd: root.to_string_lossy().into_owned(),
        persistent,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 10,
        project: None,
        ownership: None,
    });
    assert!(matches!(
        control.recv(),
        DaemonToWrapper::SpawnAccepted { agent_id: accepted, .. }
            if accepted == agent_id_for(name)
    ));
}

fn wait_named_agents(socket: &Path, expected: &[String], stopped: &[String]) {
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut control = Peer::register(socket, "persistence-observer");
    loop {
        control.send(&WrapperToDaemon::ListAgents);
        let agents = match control.recv() {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("annuaire de persistance inattendu: {other:?}"),
        };
        let ready = expected.iter().all(|name| {
            agents
                .iter()
                .any(|agent| agent.agent_id == agent_id_for(name) && agent.state == "connected")
        });
        let stopped_visible = stopped.iter().all(|name| {
            agents
                .iter()
                .any(|agent| agent.agent_id == agent_id_for(name) && agent.state == "stopped")
        });
        if ready && stopped_visible {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "annuaire persistant incomplet: attendu={expected:?}, arrêtés={stopped:?}, reçu={agents:?}"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn process_group_members(pgid: u32) -> Vec<(u32, String)> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,pgid=,command="])
        .output()
        .expect("instantané ps");
    assert!(output.status.success(), "ps doit réussir");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse::<u32>().ok()?;
            let group = fields.next()?.parse::<u32>().ok()?;
            let command = fields.collect::<Vec<_>>().join(" ");
            (group == pgid).then_some((pid, command))
        })
        .collect()
}

fn assert_npx_descendant(pgid: u32) -> Vec<(u32, String)> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let members = process_group_members(pgid);
        let has_npx = members.iter().any(|(_, command)| {
            command.contains("npx")
                || command == "npm"
                || command.contains("npm exec")
                || command.contains("npm-cli.js exec")
        });
        if members.len() >= 2 && has_npx {
            return members;
        }
        assert!(
            Instant::now() < deadline,
            "le groupe {pgid} doit contenir le wrapper et son descendant npx: {members:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_groups_gone(pgids: &[u32]) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let survivors = pgids
            .iter()
            .copied()
            .filter(|pgid| group_exists(*pgid).unwrap_or(false))
            .collect::<Vec<_>>();
        if survivors.is_empty() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "groupes encore vivants après arrêt: {survivors:?}"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn marker_pgids(root: &Path, names: &[String]) -> Vec<u32> {
    let store = ManagedMarkerStore::at_directory(root.join("state/managed"));
    names
        .iter()
        .map(|name| store.load(&agent_id_for(name)).unwrap().pgid)
        .collect()
}

#[test]
fn matrice_fr008_compare_le_meme_corpus_et_les_frames_attach() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(MATRIX_VERSION, "fr-008-v1");
    assert_eq!(MATRIX_RUNS_PER_MODE, 3);

    for run in 0..MATRIX_RUNS_PER_MODE {
        let root = test_root(&format!("matrix-{run}"));
        let adapter = write_fixture(&root);
        let (daemon, proxy) = start_daemon_behind_proxy(&root);

        let terminal_name = format!("parity-terminal-{run}");
        let mut terminal = WrapperChild(
            Some(
                Command::new(env!("CARGO_BIN_EXE_bridget"))
                    .args([
                        "--",
                        adapter.to_str().unwrap(),
                        "--equipier",
                        "--agent-id",
                        &agent_id_for(&terminal_name),
                    ])
                    .env_clear()
                    .env("HOME", &root)
                    .env("BRIDGET_HOME", root.join("state"))
                    .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
                    .env("PATH", FROZEN_PATH)
                    .env("USER", "parity-test")
                    .env("LANG", "C")
                    .env("TMPDIR", "/tmp")
                    .env("BRIDGET_CHANNEL", "unix")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            ),
            None,
        );
        let terminal_observables = run_corpus(&daemon.socket, &terminal_name, run * 2, &proxy);
        unsafe {
            libc::kill(terminal.id() as i32, libc::SIGTERM);
        }
        let _ = terminal.wait();

        let managed_name = format!("parity-managed-{run}");
        let mut control = Peer::register(&daemon.socket, &format!("spawn-client-{run}"));
        let now = unix_now();
        control.send(&WrapperToDaemon::SpawnOrder {
            posture: None,
            agent_type: "parity".to_string(),
            agent_id: Some(agent_id_for(&managed_name)),
            cwd: root.to_string_lossy().into_owned(),
            persistent: false,
            command_id: format!("spawn-parity-{run}"),
            issued_at: now,
            deadline_at: now + 8,
            project: None,
            ownership: None,
        });
        assert!(matches!(
            control.recv(),
            DaemonToWrapper::SpawnAccepted { ref agent_id, .. }
                if agent_id == &agent_id_for(&managed_name)
        ));
        let managed_observables = run_corpus(&daemon.socket, &managed_name, run * 2 + 1, &proxy);

        assert_eq!(
            terminal_observables, managed_observables,
            "{MATRIX_VERSION}: divergence au run {run}"
        );
        stop_managed(&mut control, &managed_name, run);
        daemon.stop();
        proxy.stop();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn matrice_fr008_compare_la_garde_de_facturation() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("billing");
    let adapter = write_fixture(&root);
    let daemon = DaemonProcess::start(&root, true, false);

    let terminal = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args([
            "--",
            adapter.to_str().unwrap(),
            "--equipier",
            "--agent-id",
            &agent_id_for("billing-terminal"),
        ])
        .env_clear()
        .env("HOME", &root)
        .env("BRIDGET_HOME", root.join("state"))
        .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
        .env("PATH", FROZEN_PATH)
        .env("USER", "parity-test")
        .env("LANG", "C")
        .env("TMPDIR", "/tmp")
        .env("BRIDGET_CHANNEL", "unix")
        .env("OPENAI_API_KEY", "forbidden-test-key")
        .output()
        .unwrap();
    assert!(!terminal.status.success());
    assert!(
        String::from_utf8_lossy(&terminal.stderr).contains("OPENAI_API_KEY"),
        "{}",
        String::from_utf8_lossy(&terminal.stderr)
    );

    let mut control = Peer::register(&daemon.socket, "billing-client");
    let now = unix_now();
    control.send(&WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "parity".to_string(),
        agent_id: Some(agent_id_for("billing-managed")),
        cwd: root.to_string_lossy().into_owned(),
        persistent: false,
        command_id: "spawn-billing".to_string(),
        issued_at: now,
        deadline_at: now + 5,
        project: None,
        ownership: None,
    });
    assert!(matches!(
        control.recv(),
        DaemonToWrapper::SpawnRejected {
            reason: SpawnRefusal::BillingGuard { ref variable },
            ..
        } if variable == "OPENAI_API_KEY"
    ));

    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sc001_vingt_spawns_survivent_a_la_fermeture_du_client_et_repondent() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    const SPAWNS: usize = 20;
    const SPAWN_P95_LIMIT: Duration = Duration::from_secs(10);
    const GLOBAL_BENCH_TIMEOUT: Duration = Duration::from_secs(120);

    let root = test_root("sc001");
    write_cached_npx_fixture(&root);
    let daemon = DaemonProcess::start(&root, false, true);
    let deadline = Instant::now() + GLOBAL_BENCH_TIMEOUT;
    let mut sender = Peer::register(&daemon.socket, "sc001-sender");
    let mut spawn_latencies = Vec::with_capacity(SPAWNS);

    for index in 0..SPAWNS {
        assert!(Instant::now() < deadline, "timeout global SC-001");
        let name = format!("sc001-agent-{index}");
        let command_id = format!("sc001-spawn-{index}");
        let now = unix_now();
        let started = Instant::now();
        let mut ordering_terminal =
            Peer::register(&daemon.socket, &format!("sc001-orderer-{index}"));
        ordering_terminal.send(&WrapperToDaemon::SpawnOrder {
            posture: None,
            agent_type: "parity".to_string(),
            agent_id: Some(agent_id_for(&name)),
            cwd: root.to_string_lossy().into_owned(),
            persistent: false,
            command_id,
            issued_at: now,
            deadline_at: now + 10,
            project: None,
            ownership: None,
        });
        match ordering_terminal.recv() {
            DaemonToWrapper::SpawnAccepted {
                agent_id: accepted, ..
            } => {
                assert_eq!(accepted, agent_id_for(&name))
            }
            other => panic!("spawn SC-001 {index} inattendu: {other:?}"),
        }
        spawn_latencies.push(started.elapsed());

        // Ce drop ferme réellement la socket du terminal donneur d'ordre.
        // L'échange suivant ne possède donc plus aucun lien avec ce client.
        drop(ordering_terminal);

        let request = send_tracked(&mut sender, &name, &format!("SC001-{index}"));
        assert_eq!(
            receive_replies(&mut sender, &[request]),
            vec!["fixture-response-1"]
        );
    }

    sender.send(&WrapperToDaemon::ListRequests {
        sender: sender.name.clone(),
        limit: 200,
    });
    match sender.recv() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), SPAWNS);
            assert!(requests.iter().all(|request| request.state == "answered"));
        }
        other => panic!("liste finale SC-001 inattendue: {other:?}"),
    }

    spawn_latencies.sort_unstable();
    let p95_index = (SPAWNS * 95).div_ceil(100).saturating_sub(1);
    let p95 = spawn_latencies[p95_index];
    let max = *spawn_latencies.last().unwrap();
    eprintln!(
        "SC-001 009: {SPAWNS}/{SPAWNS} spawns et échanges, p95={p95:?}, max={max:?}, total={:?}",
        GLOBAL_BENCH_TIMEOUT - deadline.saturating_duration_since(Instant::now())
    );
    assert!(p95 < SPAWN_P95_LIMIT, "p95 spawn SC-001={p95:?}");
    assert!(Instant::now() < deadline, "timeout global SC-001");

    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sc005_sc006_persistance_arrets_cooperatifs_et_reconciliation_sigkill() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    const CYCLES: usize = 3;
    const FLEET_SIZE: usize = 3;
    const GLOBAL_TIMEOUT: Duration = Duration::from_secs(120);

    let root = test_root("persistence");
    write_cached_npx_fixture(&root);
    let persistent = (0..FLEET_SIZE)
        .map(|index| format!("persistent-{index}"))
        .collect::<Vec<_>>();
    let ephemeral = (0..FLEET_SIZE)
        .map(|index| format!("ephemeral-{index}"))
        .collect::<Vec<_>>();
    let deadline = Instant::now() + GLOBAL_TIMEOUT;

    let mut daemon = DaemonProcess::start(&root, false, false);
    let mut control = Peer::register(&daemon.socket, "persistence-orderer");
    for (index, name) in persistent.iter().enumerate() {
        spawn_managed(
            &mut control,
            &root,
            name,
            &format!("persistent-spawn-{index}"),
            true,
        );
    }
    for (index, name) in ephemeral.iter().enumerate() {
        spawn_managed(
            &mut control,
            &root,
            name,
            &format!("ephemeral-spawn-{index}"),
            false,
        );
    }
    wait_named_agents(&daemon.socket, &persistent, &[]);
    wait_named_agents(&daemon.socket, &ephemeral, &[]);

    let mut cooperative_pgids = Vec::new();
    for cycle in 0..CYCLES {
        assert!(Instant::now() < deadline, "timeout global SC-005/SC-006");
        let active_names = if cycle == 0 {
            persistent
                .iter()
                .chain(ephemeral.iter())
                .cloned()
                .collect::<Vec<_>>()
        } else {
            persistent.clone()
        };
        let pgids = marker_pgids(&root, &active_names);
        for pgid in &pgids {
            let _ = assert_npx_descendant(*pgid);
        }
        cooperative_pgids.push(pgids.clone());
        daemon.stop();
        wait_groups_gone(&pgids);

        daemon = DaemonProcess::start(&root, false, false);
        wait_named_agents(&daemon.socket, &persistent, &ephemeral);
    }

    let pre_crash_pgids = marker_pgids(&root, &persistent);
    daemon.kill();
    assert!(
        pre_crash_pgids
            .iter()
            .any(|pgid| group_exists(*pgid).unwrap_or(false)),
        "SIGKILL doit laisser au moins un groupe à réconcilier"
    );
    daemon = DaemonProcess::start(&root, false, false);
    wait_groups_gone(&pre_crash_pgids);
    wait_named_agents(&daemon.socket, &persistent, &ephemeral);
    let post_crash_pgids = marker_pgids(&root, &persistent);
    assert!(
        post_crash_pgids
            .iter()
            .all(|pgid| !pre_crash_pgids.contains(pgid)),
        "la reprise doit créer une génération distincte"
    );

    let mut control = Peer::register(&daemon.socket, "persistence-stopper");
    for (index, name) in persistent.iter().enumerate() {
        stop_managed(&mut control, name, 10_000 + index);
    }
    daemon.stop();
    daemon = DaemonProcess::start(&root, false, false);
    wait_named_agents(&daemon.socket, &[], &persistent);

    eprintln!(
        "SC-005/SC-006 009: cycles={CYCLES}, persistants={persistent:?}, éphémères={ephemeral:?}, pgid_coopératifs={cooperative_pgids:?}, pgid_avant_sigkill={pre_crash_pgids:?}, pgid_après_reprise={post_crash_pgids:?}"
    );
    assert!(Instant::now() < deadline, "timeout global SC-005/SC-006");
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

/// La branche forcée traverse la même garde que le happy-path. Elle verrouille
/// la fuite qui laissait des `bridget daemon` sous PID 1 après panique.
/// Compteur borné au HOME de CE test (pas au PID du harnais). Le verrou
/// sérialise seulement la contention CPU/IO avec les autres bancs lourds.
#[test]
fn daemon_process_nettoie_apres_une_panique_injectee() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root_slot = Mutex::new(None::<PathBuf>);
    let mid = AtomicUsize::new(usize::MAX);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let root = test_root("drop-oracle");
        fs::create_dir_all(&root).unwrap();
        *root_slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(root.clone());
        // Compteur borné à CE HOME : un voisin parallèle ne peut pas le fausser.
        assert_eq!(
            daemon_count_for_home(&root),
            0,
            "le HOME du test doit être vide avant le spawn"
        );
        let _daemon = DaemonProcess::start(&root, false, false);
        // Premier temps : le spawn doit apparaître au compteur (sinon l'égalité
        // avant/après ne prouverait rien).
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen = daemon_count_for_home(&root);
        while seen != 1 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
            seen = daemon_count_for_home(&root);
        }
        mid.store(seen, Ordering::SeqCst);
        panic!("échec injecté après le spawn : la garde doit nettoyer");
    }));
    assert!(
        failed.is_err(),
        "la branche d'échec doit réellement paniquer"
    );
    assert_eq!(
        mid.load(Ordering::SeqCst),
        1,
        "premier temps : le daemon spawné doit être compté"
    );
    let root = root_slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
        .expect("HOME du test créé avant la panique");
    // Second temps : après Drop (dépliage), plus aucun daemon sur CE HOME.
    assert_daemon_count_for_home(&root, 0);
    let _ = fs::remove_dir_all(root);
}

const ABANDON_CAUSE_HARDCODED: &str =
    "abandon relance fournisseur après 2 tentatives (processus fournisseur terminé)";

fn provider_pids_in_group(pgid: u32) -> Vec<u32> {
    process_group_members(pgid)
        .into_iter()
        .filter_map(|(pid, command)| {
            let is_wrapper = command.contains("managed-wrapper");
            let is_provider = command.contains("parity-acp")
                || command.contains("python")
                || command.contains("npx")
                || command.contains("npm");
            if !is_wrapper && is_provider {
                Some(pid)
            } else {
                None
            }
        })
        .collect()
}

fn wrapper_pid_in_group(pgid: u32) -> Option<u32> {
    process_group_members(pgid)
        .into_iter()
        .find_map(|(pid, command)| command.contains("managed-wrapper").then_some(pid))
}

fn kill_provider_sigterm(pgid: u32) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let providers = provider_pids_in_group(pgid);
        if let Some(pid) = providers.into_iter().next() {
            assert_eq!(
                unsafe { libc::kill(pid as i32, libc::SIGTERM) },
                0,
                "SIGTERM fournisseur {pid}"
            );
            return pid;
        }
        assert!(
            Instant::now() < deadline,
            "aucun PID fournisseur dans le groupe {pgid}: {:?}",
            process_group_members(pgid)
        );
        thread::sleep(Duration::from_millis(25));
    }
}

/// Oracle : MEURT si un agent persistant joignable avant le kill ne revient pas
/// sans redémarrage du daemon. Prouve d'abord la joignabilité (sinon projection vide).
#[test]
fn TEMOIN_persistant_tue_redevient_joignable_sans_redemarrer_le_daemon() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("persist-relaunch");
    write_fixture(&root);
    // Borne courte pour le test, backoff minimal.
    let mut daemon = {
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut process = DaemonProcess {
            child: None,
            socket: root.join("state/bridget.sock"),
        };
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", &root)
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp/bt")
            .env("BRIDGET_PROVIDER_RELAUNCH_MAX", "5")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        process.child = Some(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&process.socket).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            UnixStream::connect(&process.socket).is_ok(),
            "daemon relance non prêt"
        );
        process
    };

    let name = "essai-persist-relaunch".to_string();
    let mut control = Peer::register(&daemon.socket, "persist-orderer");
    spawn_managed(&mut control, &root, &name, "persist-relaunch-1", true);
    wait_named_agents(&daemon.socket, &[name.clone()], &[]);

    // Preuve de joignabilité AVANT le kill — sinon l'oracle passe sur une projection vide.
    let mut sender = Peer::register(&daemon.socket, "persist-sender");
    let before = send_tracked(&mut sender, &name, "AVANT-KILL");
    assert_eq!(
        receive_replies(&mut sender, &[before]),
        vec!["fixture-response-1"],
        "joignable avant kill"
    );

    let pgids = marker_pgids(&root, &[name.clone()]);
    assert_eq!(pgids.len(), 1);
    let wrapper_before = wrapper_pid_in_group(pgids[0]).expect("wrapper avant kill");
    let killed = kill_provider_sigterm(pgids[0]);
    assert_ne!(
        killed, wrapper_before,
        "on tue le fournisseur, pas le wrapper"
    );

    // Le wrapper doit survivre (propriété nommée par le service compagnon).
    let survive_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < survive_deadline {
        if unsafe { libc::kill(wrapper_before as i32, 0) } != 0 {
            panic!("wrapper {wrapper_before} mort avec le fournisseur {killed}");
        }
        thread::sleep(Duration::from_millis(50));
    }

    wait_named_agents(&daemon.socket, &[name.clone()], &[]);
    let after = send_tracked(&mut sender, &name, "APRES-KILL");
    assert_eq!(
        receive_replies(&mut sender, &[after]),
        vec!["fixture-response-1"],
        "joignable après kill sans redémarrage daemon"
    );
    assert_eq!(
        unsafe { libc::kill(wrapper_before as i32, 0) },
        0,
        "même wrapper encore vivant après reprise"
    );

    stop_managed(&mut control, &name, 1);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

/// Oracle : MEURT si l'abandon après N tentatives n'est pas nommé en dur.
#[test]
fn TEMOIN_abandon_apres_N_tentatives_est_nomme() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("persist-abandon");
    let adapter = write_fixture(&root);
    let mut daemon = {
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut process = DaemonProcess {
            child: None,
            socket: root.join("state/bridget.sock"),
        };
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", &root)
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp/bt")
            .env("BRIDGET_PROVIDER_RELAUNCH_MAX", "2")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        process.child = Some(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&process.socket).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        process
    };

    let name = "essai-persist-abandon".to_string();
    let mut control = Peer::register(&daemon.socket, "abandon-orderer");
    spawn_managed(&mut control, &root, &name, "persist-abandon-1", true);
    wait_named_agents(&daemon.socket, &[name.clone()], &[]);

    let mut sender = Peer::register(&daemon.socket, "abandon-sender");
    let before = send_tracked(&mut sender, &name, "AVANT-ABANDON");
    assert_eq!(
        receive_replies(&mut sender, &[before]),
        vec!["fixture-response-1"]
    );

    let pgids = marker_pgids(&root, &[name.clone()]);
    // Remplacer le fournisseur par un binaire qui refuse de démarrer.
    fs::write(&adapter, "#!/bin/sh\necho refuse-auth >&2\nexit 1\n").unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let _ = kill_provider_sigterm(pgids[0]);

    // Le daemon conserve l'entrée en state=stopped après Unregister — ne pas
    // exiger la disparition du roster. La propriété mesurée : plus joignable.
    let gone_deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let mut observer = Peer::register(&daemon.socket, "abandon-observer");
        observer.send(&WrapperToDaemon::ListAgents);
        let agents = match observer.recv() {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("liste inattendue: {other:?}"),
        };
        let still_connected = agents
            .iter()
            .any(|agent| agent.agent_id == agent_id_for(&name) && agent.state == "connected");
        if !still_connected {
            break;
        }
        assert!(
            Instant::now() < gone_deadline,
            "agent encore connected après abandon attendu: {agents:?}"
        );
        thread::sleep(Duration::from_millis(100));
    }

    // Cause nommée EN DUR dans stderr géré.
    let stderr_root = root.join("state/managed-stderr");
    let mut found = false;
    let scan_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < scan_deadline && !found {
        if stderr_root.exists() {
            for entry in walkdir_files(&stderr_root) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(ABANDON_CAUSE_HARDCODED) {
                        found = true;
                        break;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        found,
        "cause d'abandon absente des stderr gérés; attendu en dur: {ABANDON_CAUSE_HARDCODED}"
    );

    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

fn walkdir_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}
