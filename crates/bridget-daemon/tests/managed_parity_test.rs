use bridget_core::BridgetMessage;
use bridget_transport::protocol::{AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MATRIX_VERSION: &str = "fr-008-v1";
const MATRIX_RUNS_PER_MODE: usize = 3;
const MATRIX_EXPECTED_TURNS: usize = 4;
const MATRIX_TIMEOUT: Duration = Duration::from_secs(10);
const FROZEN_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";

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
            time.sleep(0.15)
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
        while !socket.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            socket.exists(),
            "le daemon de parité n'a pas ouvert sa socket"
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

fn wait_agent(control: &mut Peer, name: &str) -> (String, String) {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        control.send(&WrapperToDaemon::ListAgents);
        if let DaemonToWrapper::AgentList { agents } = control.recv()
            && let Some(agent) = agents.into_iter().find(|agent| agent.name == name)
        {
            return (agent.transport, agent.state);
        }
        assert!(
            Instant::now() < deadline,
            "l'équipier {name} ne s'est pas enregistré"
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

fn run_corpus(socket: &Path, agent: &str, run: usize) -> ModeObservables {
    let mut peer = Peer::register(socket, &format!("parity-sender-{run}"));
    let (transport, state) = wait_agent(&mut peer, agent);
    assert_eq!(transport, "acp");
    assert_eq!(state, "connected");

    let first = send_tracked(&mut peer, agent, "TRACKED");
    let mut replies = receive_replies(&mut peer, &[first]);
    let exact = "l'apostrophe d'usage, \"guillemets\", $VAR, `backticks`,\net ce saut de ligne.";
    let second = send_tracked(&mut peer, agent, exact);
    replies.extend(receive_replies(&mut peer, &[second]));

    let slow = send_tracked(&mut peer, agent, "QUEUE-SLOW");
    let next = send_tracked(&mut peer, agent, "QUEUE-NEXT");
    replies.extend(receive_replies(&mut peer, &[slow, next]));
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
    });
    let request_states: Vec<String> = match peer.recv() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), MATRIX_EXPECTED_TURNS);
            requests.into_iter().map(|request| request.state).collect()
        }
        other => panic!("liste de demandes inattendue: {other:?}"),
    };
    assert!(request_states.iter().all(|state| state == "answered"));

    ModeObservables {
        replies,
        request_states,
        agent_transport: transport,
        agent_state: state,
        journal: collect_journal(socket, agent),
    }
}

fn stop_managed(control: &mut Peer, name: &str, run: usize) {
    control.send(&WrapperToDaemon::StopOrder {
        name: name.to_string(),
        command_id: format!("stop-parity-{run}"),
    });
    assert!(matches!(
        control.recv(),
        DaemonToWrapper::StopResult {
            outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
            ..
        }
    ));
}

#[test]
fn matrice_fr008_compare_le_meme_corpus_et_les_frames_attach() {
    assert_eq!(MATRIX_VERSION, "fr-008-v1");
    assert_eq!(MATRIX_RUNS_PER_MODE, 3);
    for run in 0..MATRIX_RUNS_PER_MODE {
        let root = test_root(&format!("matrix-{run}"));
        let adapter = write_fixture(&root);
        let daemon = DaemonProcess::start(&root, false, false);

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
        let terminal_observables = run_corpus(&daemon.socket, &terminal_name, run * 2);
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
        let managed_observables = run_corpus(&daemon.socket, &managed_name, run * 2 + 1);

        assert_eq!(
            terminal_observables, managed_observables,
            "{MATRIX_VERSION}: divergence au run {run}"
        );
        stop_managed(&mut control, &managed_name, run);
        daemon.stop();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn matrice_fr008_compare_la_garde_de_facturation() {
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
