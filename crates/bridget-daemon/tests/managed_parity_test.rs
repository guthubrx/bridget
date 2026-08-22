use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{ManagedMarkerStore, group_exists};
use bridget_transport::protocol::{AgentInfo, AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MATRIX_VERSION: &str = "fr-008-v1";
const REDUCED_PROMPT: &str = include_str!("fixtures/prompts/v1-after.txt");
const MATRIX_RUNS_PER_MODE: usize = 3;
const MATRIX_EXPECTED_TURNS: usize = 4;
const MATRIX_TIMEOUT: Duration = Duration::from_secs(10);
const FROZEN_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";
static MANAGED_BENCH_LOCK: Mutex<()> = Mutex::new(());

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn test_root(label: &str) -> PathBuf {
    PathBuf::from(format!(
        "/tmp/bg909-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn write_fixture(root: &Path) -> PathBuf {
    let adapter = root.join("parity-acp.py");
    fs::create_dir_all(root.join(".config/bridget")).unwrap();
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
        turn += 1
        prompt = request["params"]["prompt"][0]["text"]
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
                "notify_timeout_secs": 2,
                "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
                "pass_env": ["PARITY_SINGLE_TURN"]
            }
        }
    });
    fs::write(
        root.join(".config/bridget/agents.json"),
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
    adapter
}

fn capture_interactive_mcp_prompt(root: &Path, run: usize) {
    let bin = root.join("prompt-bin");
    let capture = root.join("captured-prompt.json");
    let release = root.join("release-prompt-cli");
    let codex = bin.join("codex");
    fs::create_dir_all(&bin).unwrap();
    fs::write(
        &codex,
        r#"#!/usr/bin/python3
import json
import os
import sys
import time

capture = os.environ["BRIDGET_PROMPT_CAPTURE"]
temporary = capture + ".tmp"
with open(temporary, "w", encoding="utf-8") as output:
    json.dump(sys.argv[1:], output, ensure_ascii=False)
os.replace(temporary, capture)
while not os.path.exists(os.environ["BRIDGET_PROMPT_RELEASE"]):
    time.sleep(0.01)
"#,
    )
    .unwrap();
    fs::set_permissions(&codex, fs::Permissions::from_mode(0o700)).unwrap();

    let name = format!("prompt-mcp-{run}");
    let mut child = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["codex", "--name", &name])
        .env_clear()
        .env("HOME", root)
        .env("PATH", format!("{}:{FROZEN_PATH}", bin.display()))
        .env("USER", "parity-test")
        .env("LANG", "C")
        .env("TMPDIR", "/tmp")
        .env("BRIDGET_PROMPT_CAPTURE", &capture)
        .env("BRIDGET_PROMPT_RELEASE", &release)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while !capture.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if !capture.exists() {
        let _ = child.kill();
        let output = child.wait_with_output().unwrap();
        panic!(
            "le vrai wrapper MCP n'a pas transmis son prompt : {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let arguments: Vec<String> = serde_json::from_slice(&fs::read(&capture).unwrap()).unwrap();
    let actual = arguments
        .iter()
        .find(|argument| argument.starts_with("Tu es l'agent"))
        .expect("prompt Bridget absent des arguments du CLI MCP");
    let expected = REDUCED_PROMPT
        .trim_end_matches('\n')
        .replace("agent-fixture", &name);
    assert_eq!(
        actual, &expected,
        "le lancement MCP n'utilise pas la fixture réduite"
    );

    fs::write(&release, b"release").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "le wrapper de capture a échoué : {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
    let registry_path = root.join(".config/bridget/agents.json");
    let mut registry: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
    registry["agents"]["parity"]["command"] = serde_json::json!("npx");
    registry["agents"]["parity"]["args"] =
        serde_json::json!(["--offline", "--no-install", "parity-acp"]);
    fs::write(registry_path, serde_json::to_vec_pretty(&registry).unwrap()).unwrap();
}

struct DaemonProcess {
    child: Child,
    socket: PathBuf,
}

impl DaemonProcess {
    fn start(root: &Path, billing_key: bool, single_turn: bool) -> Self {
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", root)
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
        let child = command.spawn().unwrap();
        let socket = root.join(".cache/bridget/bridget.sock");
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut accepting = false;
        while Instant::now() < deadline {
            if UnixStream::connect(&socket).is_ok() {
                accepting = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            accepting,
            "le daemon de parité n'accepte pas encore les connexions"
        );
        Self { child, socket }
    }

    fn stop(mut self) {
        unsafe {
            libc::kill(self.child.id() as i32, libc::SIGTERM);
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.child.try_wait().unwrap().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        panic!("le daemon de parité n'a pas terminé après SIGTERM");
    }

    fn kill(mut self) {
        unsafe {
            libc::kill(self.child.id() as i32, libc::SIGKILL);
        }
        let _ = self.child.wait();
    }
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
                let target_stream = UnixStream::connect(&target).unwrap();
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
                        Ok(WrapperToDaemon::Register { ref agent_type, .. }) if agent_type == "parity"
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
        sockets.0.shutdown(std::net::Shutdown::Both).unwrap();
        sockets.1.shutdown(std::net::Shutdown::Both).unwrap();
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

fn start_daemon_behind_proxy(root: &Path) -> (DaemonProcess, CutProxy) {
    let cache_parent = root.join(".cache");
    let daemon_cache = root.join("daemon-cache");
    let proxy_cache = root.join("proxy-cache");
    fs::create_dir_all(&cache_parent).unwrap();
    fs::create_dir_all(&daemon_cache).unwrap();
    fs::create_dir_all(&proxy_cache).unwrap();
    let cache_link = cache_parent.join("bridget");
    std::os::unix::fs::symlink(&daemon_cache, &cache_link).unwrap();
    let mut daemon = DaemonProcess::start(root, false, false);
    let target = daemon_cache.join("bridget.sock");
    let database = daemon_cache.join("bridget.db");
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while !database.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(target.exists());
    assert!(
        database.exists(),
        "le daemon n'a pas achevé son initialisation"
    );
    fs::remove_file(&cache_link).unwrap();
    std::os::unix::fs::symlink(&proxy_cache, &cache_link).unwrap();
    let proxy_socket = proxy_cache.join("bridget.sock");
    let proxy = CutProxy::start(proxy_socket, target);
    daemon.socket = cache_link.join("bridget.sock");
    (daemon, proxy)
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
            agent_type: "parity-client".to_string(),
            name: Some(name.to_string()),
            host: Some("fixture-host".to_string()),
            transport: Some("unix".to_string()),
            os: Some("fixture-os".to_string()),
            instance_id: Some(format!("instance-{name}")),
            domain: None,
            turn_in_progress: false,
        });
        match peer.recv() {
            DaemonToWrapper::Registered { name: assigned } => peer.name = assigned,
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
            && let Some(agent) = agents.into_iter().find(|agent| agent.name == name)
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
    let mut message = BridgetMessage::new(&peer.name, to, body);
    message.reply = true;
    message.reply_timeout = Some(5);
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
        if let DaemonToWrapper::Deliver(message) = peer.recv() {
            let in_reply_to = message.in_reply_to.as_deref().unwrap_or_default();
            assert_eq!(in_reply_to, expected_ids[replies.len()]);
            replies.push(message.body);
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
        agent: agent.to_string(),
        window: AttachWindow::Seq(0),
    });
    let mut fragments: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut complete = BTreeMap::new();
    let mut caught_up = false;
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while (!caught_up || complete.len() < MATRIX_EXPECTED_TURNS * 3) && Instant::now() < deadline {
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
    assert_eq!(
        complete.len(),
        MATRIX_EXPECTED_TURNS * 3,
        "la fixture doit produire trois événements par tour"
    );
    complete
        .into_values()
        .map(|bytes| normalized_entry(&bytes))
        .collect()
}

fn run_corpus(socket: &Path, agent: &str, run: usize, proxy: &CutProxy) -> ModeObservables {
    let mut peer = Peer::register(socket, &format!("parity-sender-{run}"));
    // Quickstart 007 §1 : l'équipier est visible en ACP, prêt à recevoir.
    let initial_agent = wait_agent(&mut peer, agent);
    assert_eq!(initial_agent.transport, "acp");
    assert_eq!(initial_agent.state, "connected");

    // Quickstart 007 §2 : une demande suivie reçoit sa réponse et se clôt.
    let first = send_tracked(&mut peer, agent, "TRACKED");
    let mut replies = receive_replies(&mut peer, &[first]);
    // Quickstart 007 §3 : le corps riche traverse le transport octet pour octet.
    let exact = "l'apostrophe d'usage, \"guillemets\", $VAR, `backticks`,\net ce saut de ligne.";
    let second = send_tracked(&mut peer, agent, exact);
    replies.extend(receive_replies(&mut peer, &[second]));

    // Quickstart 007 §4 : FIFO pendant un tour, relance différée et
    // reconnexion conservant l'état busy.
    let slow = send_tracked(&mut peer, agent, "QUEUE-SLOW");
    let next = send_tracked(&mut peer, agent, "QUEUE-NEXT");
    let busy = wait_agent(&mut peer, agent);
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
        name: name.to_string(),
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
        agent_type: "parity".to_string(),
        name: Some(name.to_string()),
        cwd: root.to_string_lossy().into_owned(),
        persistent,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 10,
    });
    assert!(matches!(
        control.recv(),
        DaemonToWrapper::SpawnAccepted { name: accepted, .. } if accepted == name
    ));
}

fn wait_named_agents(socket: &Path, expected: &[String], absent: &[String]) {
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
                .any(|agent| agent.name == *name && agent.state == "connected")
        });
        let excluded = absent
            .iter()
            .all(|name| agents.iter().all(|agent| agent.name != *name));
        if ready && excluded {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "annuaire persistant incomplet: attendu={expected:?}, absent={absent:?}, reçu={agents:?}"
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
    let store = ManagedMarkerStore::for_home(root);
    names
        .iter()
        .map(|name| store.load(name).unwrap().pgid)
        .collect()
}

#[test]
fn matrice_fr008_compare_le_meme_corpus_et_les_frames_attach() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(MATRIX_VERSION, "fr-008-v1");
    assert_eq!(MATRIX_RUNS_PER_MODE, 3);
    assert!(REDUCED_PROMPT.contains("reply=yes"));
    assert!(REDUCED_PROMPT.contains("reply=no"));
    assert!(!REDUCED_PROMPT.contains("bridget send"));

    let prompt_root = PathBuf::from(format!(
        "/tmp/bg10p-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    write_fixture(&prompt_root);
    let prompt_daemon = DaemonProcess::start(&prompt_root, false, false);
    capture_interactive_mcp_prompt(&prompt_root, 0);
    prompt_daemon.stop();
    fs::remove_dir_all(prompt_root).unwrap();

    for run in 0..MATRIX_RUNS_PER_MODE {
        let root = test_root(&format!("matrix-{run}"));
        let adapter = write_fixture(&root);
        let (daemon, proxy) = start_daemon_behind_proxy(&root);

        let terminal_name = format!("parity-terminal-{run}");
        let mut terminal = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args([
                "--",
                adapter.to_str().unwrap(),
                "--equipier",
                "--name",
                &terminal_name,
            ])
            .env_clear()
            .env("HOME", &root)
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let terminal_observables = run_corpus(&daemon.socket, &terminal_name, run * 2, &proxy);
        unsafe {
            libc::kill(terminal.id() as i32, libc::SIGTERM);
        }
        let _ = terminal.wait();

        let managed_name = format!("parity-managed-{run}");
        let mut control = Peer::register(&daemon.socket, &format!("spawn-client-{run}"));
        let now = unix_now();
        control.send(&WrapperToDaemon::SpawnOrder {
            agent_type: "parity".to_string(),
            name: Some(managed_name.clone()),
            cwd: root.to_string_lossy().into_owned(),
            persistent: false,
            command_id: format!("spawn-parity-{run}"),
            issued_at: now,
            deadline_at: now + 8,
        });
        assert!(matches!(
            control.recv(),
            DaemonToWrapper::SpawnAccepted { ref name, .. } if name == &managed_name
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
            "--name",
            "billing-terminal",
        ])
        .env_clear()
        .env("HOME", &root)
        .env("PATH", FROZEN_PATH)
        .env("USER", "parity-test")
        .env("LANG", "C")
        .env("TMPDIR", "/tmp")
        .env("OPENAI_API_KEY", "forbidden-test-key")
        .output()
        .unwrap();
    assert!(!terminal.status.success());
    assert!(String::from_utf8_lossy(&terminal.stderr).contains("OPENAI_API_KEY"));

    let mut control = Peer::register(&daemon.socket, "billing-client");
    let now = unix_now();
    control.send(&WrapperToDaemon::SpawnOrder {
        agent_type: "parity".to_string(),
        name: Some("billing-managed".to_string()),
        cwd: root.to_string_lossy().into_owned(),
        persistent: false,
        command_id: "spawn-billing".to_string(),
        issued_at: now,
        deadline_at: now + 5,
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
            agent_type: "parity".to_string(),
            name: Some(name.clone()),
            cwd: root.to_string_lossy().into_owned(),
            persistent: false,
            command_id,
            issued_at: now,
            deadline_at: now + 10,
        });
        match ordering_terminal.recv() {
            DaemonToWrapper::SpawnAccepted { name: accepted, .. } => {
                assert_eq!(accepted, name)
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
