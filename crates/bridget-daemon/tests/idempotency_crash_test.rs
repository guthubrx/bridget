#![cfg(feature = "test-support")]

//! Matrice de crash inter-processus du contrat idempotent.
//!
//! Les deux frontières amont sont exécutées ici dès que le socle et Lookup
//! sont disponibles. Les frontières aval sont ajoutées avec leur relais
//! `DeliverIdempotent` : elles partagent les mêmes jalons et le même harnais.

use bridget_core::BridgetMessage;
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::store::Store;
use bridget_daemon::test_sync::DIRECTORY_ENV;
use bridget_daemon::wrapper::launch_acp_with;
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::ffi::CString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MATRIX_CYCLES: usize = 50;
// Le wrapper ACP reconnecte volontairement avec un premier délai d'une seconde :
// cinquante redémarrages réels restent donc bornés, sans rendre le banc fragile.
const GLOBAL_TIMEOUT: Duration = Duration::from_secs(360);
const READY_TIMEOUT: Duration = Duration::from_secs(5);
const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(5);
const SCOPE: &str = "abcdefghijklmnopqrstuv";

struct DaemonProcess {
    child: Child,
    process_group_id: i32,
    logs: Option<thread::JoinHandle<()>>,
}

/// Possède le daemon de la matrice : une panique de timeout ne laisse jamais
/// l'enfant actif après la fin du banc.
struct MatrixDaemonGuard(Option<DaemonProcess>);

impl MatrixDaemonGuard {
    fn start(root: &Path, sync: &Path) -> Self {
        Self(Some(spawn_daemon(root, Some(sync))))
    }

    fn restart(&mut self, root: &Path) {
        self.stop();
        self.0 = Some(spawn_daemon(root, None));
    }

    fn restart_with_sync(&mut self, root: &Path, sync: &Path) {
        self.stop();
        self.0 = Some(spawn_daemon(root, Some(sync)));
    }

    fn stop(&mut self) {
        if let Some(daemon) = self.0.take() {
            daemon.stop();
        }
    }

    fn crash(&mut self) {
        if let Some(daemon) = self.0.take() {
            daemon.crash();
        }
    }
}

impl Drop for MatrixDaemonGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

impl DaemonProcess {
    fn stop(mut self) {
        unsafe {
            libc::kill(-self.process_group_id, libc::SIGTERM);
        }
        thread::sleep(Duration::from_millis(20));
        let _ = unsafe { libc::kill(-self.process_group_id, libc::SIGKILL) };
        let _ = self.child.wait();
        if let Some(logs) = self.logs.take() {
            let _ = logs.join();
        }
    }

    fn crash(mut self) {
        unsafe {
            libc::kill(-self.process_group_id, libc::SIGKILL);
        }
        let _ = self.child.wait();
        if let Some(logs) = self.logs.take() {
            let _ = logs.join();
        }
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        unsafe {
            libc::kill(-self.process_group_id, libc::SIGKILL);
        }
        let _ = self.child.wait();
    }
}

struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

struct McpProcess {
    child: Child,
    input: BufWriter<std::process::ChildStdin>,
    output: BufReader<std::process::ChildStdout>,
}

impl McpProcess {
    fn start(root: &Path, name: &str, instance_id: &str) -> Self {
        let name_file = root.join("mcp-agent-name");
        fs::write(&name_file, name).expect("nom MCP écrit");
        let mut child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("mcp")
            .env("HOME", root)
            .env("BRIDGET_AGENT_NAME_FILE", &name_file)
            .env("BRIDGET_AGENT_INSTANCE_ID", instance_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("serveur MCP réel démarré");
        Self {
            input: BufWriter::new(child.stdin.take().expect("stdin MCP")),
            output: BufReader::new(child.stdout.take().expect("stdout MCP")),
            child,
        }
    }

    fn request(&mut self, request: serde_json::Value) -> serde_json::Value {
        writeln!(
            self.input,
            "{}",
            serde_json::to_string(&request).expect("requête MCP sérialisable")
        )
        .expect("requête MCP écrite");
        self.input.flush().expect("requête MCP vidée");
        let mut line = String::new();
        self.output
            .read_line(&mut line)
            .expect("réponse MCP lisible");
        serde_json::from_str(&line).expect("réponse MCP JSON")
    }

    fn notify(&mut self, notification: serde_json::Value) {
        writeln!(
            self.input,
            "{}",
            serde_json::to_string(&notification).expect("notification MCP sérialisable")
        )
        .expect("notification MCP écrite");
        self.input.flush().expect("notification MCP vidée");
    }

    fn stop(self) {
        let Self {
            mut child,
            input,
            output,
        } = self;
        drop(input);
        drop(output);
        let _ = child.wait();
    }
}

impl Client {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).expect("connexion au daemon");
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .expect("borne lecture client");
        let reader = BufReader::new(stream.try_clone().expect("clone lecture"));
        Self {
            reader,
            writer: BufWriter::new(stream),
        }
    }

    fn send(&mut self, message: WrapperToDaemon) {
        writeln!(
            self.writer,
            "{}",
            encode(&message).expect("message encodable")
        )
        .expect("écriture client");
        self.writer.flush().expect("flush client");
    }

    fn receive(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).expect("réponse daemon");
        decode(line.trim_end()).expect("réponse daemon décodable")
    }
}

fn test_root(label: &str) -> PathBuf {
    PathBuf::from("/tmp").join(format!(
        "bridget-t1209-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge")
            .as_nanos()
    ))
}

fn socket(root: &Path) -> PathBuf {
    root.join(".cache/bridget/bridget.sock")
}

fn make_fifo(path: &Path) {
    let path = CString::new(path.as_os_str().as_bytes()).expect("chemin FIFO sans NUL");
    let result = unsafe { libc::mkfifo(path.as_ptr(), 0o600) };
    assert!(
        result == 0,
        "création FIFO: {}",
        std::io::Error::last_os_error()
    );
}

fn checkpoint_root(root: &Path, point: &str) -> (PathBuf, PathBuf) {
    let sync = root.join("sync");
    fs::create_dir_all(&sync).expect("répertoire de synchronisation");
    make_fifo(&sync.join(format!("{point}.fifo")));
    let marker = sync.join(format!("{point}.ready"));
    (sync, marker)
}

fn arm_checkpoint(sync: &Path, points: &[&str], point: &str) {
    for candidate in points {
        let _ = fs::remove_file(sync.join(format!("{candidate}.fifo")));
        let _ = fs::remove_file(sync.join(format!("{candidate}.ready")));
    }
    make_fifo(&sync.join(format!("{point}.fifo")));
}

fn spawn_daemon(root: &Path, sync: Option<&Path>) -> DaemonProcess {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    command
        .arg("daemon")
        .env("HOME", root)
        .env("RUST_LOG", "info")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(sync) = sync {
        command.env(DIRECTORY_ENV, sync);
    } else {
        command.env_remove(DIRECTORY_ENV);
    }
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn().expect("spawn daemon réel");
    let process_group_id = child.id() as i32;
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
    match ready_rx.recv_timeout(READY_TIMEOUT) {
        Ok(Ok(())) => {}
        Ok(Err(logs)) => panic!("daemon arrêté avant disponibilité: {logs}"),
        Err(_) => panic!("daemon non prêt dans la borne"),
    }
    DaemonProcess {
        child,
        process_group_id,
        logs: Some(logs),
    }
}

fn watch_marker(directory: &Path, marker: &Path) {
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

#[allow(dead_code)]
fn register_recipient(socket: &Path) -> Client {
    register_recipient_as(socket, "t1209-recipient")
}

fn register_recipient_as(socket: &Path, instance_id: &str) -> Client {
    let mut recipient = Client::connect(socket);
    recipient.send(WrapperToDaemon::Register {
        agent_type: "fixture".to_string(),
        name: Some("recipient".to_string()),
        host: Some("t1209".to_string()),
        transport: Some("acp".to_string()),
        channel: None,
        mode: Some(bridget_transport::protocol::PresenceMode::Acp),
        location: None,
        os: Some("test".to_string()),
        instance_id: Some(instance_id.to_string()),
        domain: None,
        journal_available: None,
        turn_in_progress: false,
    });
    assert!(matches!(
        recipient.receive(),
        DaemonToWrapper::Registered { .. }
    ));
    recipient
}

fn receive_delivery(recipient: &mut Client) -> DaemonToWrapper {
    match recipient.receive() {
        delivery @ DaemonToWrapper::DeliverIdempotent { .. } => delivery,
        other => panic!("remise idempotente attendue: {other:?}"),
    }
}

fn assert_no_delivery(recipient: &mut Client) {
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

fn negotiate_client(socket: &Path) -> Client {
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
    client
}

fn idempotent_send(message_id: String, issued_at: i64) -> WrapperToDaemon {
    let mut message = BridgetMessage::new("human", "recipient", "crash matrix");
    message.id = message_id.clone();
    WrapperToDaemon::SendIdempotent {
        message,
        message_id,
        issued_at,
    }
}

fn retry_issue(socket: &Path, message_id: String, issued_at: i64) -> IdempotencyIssue {
    retry_command_issue(socket, idempotent_send(message_id, issued_at))
}

fn retry_command_issue(socket: &Path, command: WrapperToDaemon) -> IdempotencyIssue {
    let mut client = negotiate_client(socket);
    client.send(command);
    match client.receive() {
        DaemonToWrapper::IdempotencyResult { issue, .. } => issue,
        other => panic!("issue idempotente attendue: {other:?}"),
    }
}

fn registry_with_counting_acp_agent(prompt_limit: usize) -> (AgentRegistry, PathBuf, PathBuf) {
    let directory = test_root("acp-registry");
    fs::create_dir_all(&directory).expect("répertoire du registre ACP");
    let counter = directory.join("session-prompt-count");
    let definition = serde_json::json!({
        "agents": {
            "fixture-acp": {
                "command": "sh",
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
                "notify_timeout_secs": 1
            }
        }
    });
    let path = directory.join("agents.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&definition).expect("registre sérialisable"),
    )
    .expect("registre ACP écrit");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
        .expect("permissions privées du registre ACP");
    let registry = AgentRegistry::from_json(
        &fs::read_to_string(&path).expect("registre ACP lisible"),
        &path,
    )
    .expect("registre ACP valide");
    (registry, directory, counter)
}

fn wait_for_counter(counter: &Path, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read(counter).map_or(0, |bytes| bytes.len()) < expected {
        assert!(
            Instant::now() < deadline,
            "frame session/prompt absente dans la borne"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_registered_agent(socket: &Path, name: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut probe = Client::connect(socket);
        probe.send(WrapperToDaemon::ListAgents);
        if matches!(
            probe.receive(),
            DaemonToWrapper::AgentList { agents } if agents.iter().any(|agent| agent.name == name)
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

fn wait_for_accepted(socket: &Path, command: &WrapperToDaemon) {
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

fn issued_at() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64
}

fn run_linked_cli(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(args)
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin")
        .env("BRIDGET_AGENT_NAME", "worker")
        .env("BRIDGET_AGENT_INSTANCE_ID", "shared-cli-mcp-instance")
        .output()
        .expect("binaire Bridget exécuté")
}

fn output_text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn output_field(output: &std::process::Output, field: &str) -> String {
    let prefix = format!("{field}=");
    output_text(output)
        .split_whitespace()
        .find_map(|token| token.strip_prefix(&prefix))
        .map(|value| value.trim_end_matches(':').to_string())
        .unwrap_or_else(|| panic!("champ {field} absent de la sortie: {}", output_text(output)))
}

fn mcp_send_call(request_id: i64, message_id: &str, sent_at: i64, body: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": "bridget_send",
            "arguments": {
                "to": "recipient",
                "body": body,
                "in_reply_to": "request-open",
                "id": message_id,
                "issued_at": sent_at
            }
        }
    })
}

#[allow(dead_code)]
fn run_amont_cycle(point: &str, serial: usize) {
    let root = test_root(point);
    let sync = root.join("sync");
    fs::create_dir_all(&sync).expect("répertoire de synchronisation");
    make_fifo(&sync.join(format!("{point}.fifo")));
    let marker = sync.join(format!("{point}.ready"));
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let _recipient = register_recipient(&socket_path);
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let message_id = format!("t1209-{point}-{serial}");
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    watch_marker(&sync, &marker);
    daemon.stop();
    drop(client);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let _recipient = register_recipient(&socket_path);
    let first = retry_issue(&socket_path, message_id.clone(), issued_at);
    let second = retry_issue(&socket_path, message_id, issued_at);
    assert_eq!(first, second, "une issue rejouée doit être stable");
    assert!(matches!(first, IdempotencyIssue::OutcomeUnknown { .. }));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage cycle");
}

#[test]
#[ignore = "banc de gate SC-001 : cargo test --features test-support --test idempotency_crash_test -- --ignored --test-threads=1"]
fn matrice_crash_sc001_redelivre_cinquante_prompts_uniques() {
    let root = test_root("sc001-matrix");
    let sync = root.join("sync");
    fs::create_dir_all(&sync).expect("répertoire de synchronisation");
    let points = [
        "before_reservation",
        "after_prepared",
        "after_delivery_before_issue",
        "after_issue_before_client_ack",
    ];
    arm_checkpoint(&sync, &points, points[0]);

    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(MATRIX_CYCLES);
    let wrapper_home = registry_root.join("wrapper-home");
    fs::create_dir_all(&wrapper_home).expect("home wrapper ACP");
    let wrapper_registry = registry.clone();
    let wrapper_socket = socket_path.clone();
    let wrapper = thread::spawn(move || {
        launch_acp_with(
            "fixture-acp",
            &[],
            Some("acp-matrix"),
            &wrapper_registry,
            &wrapper_socket,
            &wrapper_home,
        )
        .map_err(|error| error.to_string())
    });
    wait_for_registered_agent(&socket_path, "acp-matrix");
    let deadline = Instant::now() + GLOBAL_TIMEOUT;

    for serial in 0..MATRIX_CYCLES {
        assert!(
            Instant::now() < deadline,
            "matrice T1209 dépassée après {GLOBAL_TIMEOUT:?}"
        );
        let point = points[serial % points.len()];
        let marker = sync.join(format!("{point}.ready"));
        let _ = fs::remove_file(&marker);
        let issued_at = issued_at();
        let message_id = format!("sc001-{serial}");
        let mut message = BridgetMessage::new("human", "acp-matrix", "matrice SC-001");
        message.id = message_id.clone();
        let command = WrapperToDaemon::SendIdempotent {
            message,
            message_id,
            issued_at,
        };
        let mut client = negotiate_client(&socket_path);
        client.send(command.clone());
        watch_marker(&sync, &marker);
        daemon.crash();

        daemon.restart(&root);
        wait_for_registered_agent(&socket_path, "acp-matrix");
        let first_replay = retry_command_issue(&socket_path, command.clone());
        let second_replay = retry_command_issue(&socket_path, command.clone());
        assert_eq!(first_replay, second_replay, "le rejeu en vol est stable");
        assert!(matches!(
            first_replay,
            IdempotencyIssue::OutcomeUnknown { .. }
        ));
        wait_for_counter(&counter, serial + 1);
        wait_for_accepted(&socket_path, &command);
        let first_terminal = retry_command_issue(&socket_path, command.clone());
        let second_terminal = retry_command_issue(&socket_path, command.clone());
        assert_eq!(
            first_terminal, second_terminal,
            "le rejeu terminal est stable"
        );
        assert!(matches!(first_terminal, IdempotencyIssue::Accepted { .. }));
        if serial + 1 < MATRIX_CYCLES {
            arm_checkpoint(&sync, &points, points[(serial + 1) % points.len()]);
            daemon.restart_with_sync(&root, &sync);
            wait_for_registered_agent(&socket_path, "acp-matrix");
        }
    }

    assert_eq!(
        fs::read(&counter).expect("compteur ACP"),
        vec![b'x'; MATRIX_CYCLES],
        "chaque crash remet exactement un prompt, sans doublon"
    );
    daemon.stop();
    assert_eq!(wrapper.join().expect("thread wrapper"), Ok(()));
    fs::remove_dir_all(root).expect("nettoyage matrice");
    fs::remove_dir_all(registry_root).expect("nettoyage registre ACP");
}

#[test]
fn reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance() {
    let root = test_root("redelivery");
    let daemon = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-stable");
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let message_id = "t1205bis-redelivery".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    let first = receive_delivery(&mut recipient);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-stable");
    let replayed = receive_delivery(&mut recipient);
    assert_eq!(
        encode(&first).expect("première remise encodable"),
        encode(&replayed).expect("remise reprise encodable"),
        "la reprise doit relire et rejouer les bytes persistés"
    );
    let (delivery_id, delivery_generation) = match replayed {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    let terminal = retry_issue(&socket_path, message_id, issued_at);
    assert!(matches!(terminal, IdempotencyIssue::Accepted { .. }));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage reprise");
}

#[test]
fn destination_remplacee_reste_indeterminee_sans_reroutage() {
    let root = test_root("destination-remplacee");
    let daemon = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-original");
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let message_id = "t1205bis-destination-remplacee".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliveryIndeterminate {
        delivery_id,
        delivery_generation,
    });
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut replacement = register_recipient_as(&socket_path, "recipient-replacement");
    assert_no_delivery(&mut replacement);
    let issue = retry_issue(&socket_path, message_id, issued_at);
    assert!(matches!(issue, IdempotencyIssue::OutcomeUnknown { .. }));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage destination remplacée");
}

#[test]
fn recovery_terminal_acked_rejoue_accepted_apres_crash_daemon() {
    let root = test_root("recovery-terminal");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_acked");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-terminal-instance");
    let issued_at = issued_at();
    let message_id = "recovery-terminal".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    let _ = client.receive();
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    watch_marker(&sync, &marker);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-terminal-instance");
    assert_no_delivery(&mut recipient);
    wait_for_accepted(&socket_path, &idempotent_send(message_id, issued_at));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage terminal");
}

#[test]
fn recovery_ack_d_une_reponse_liee_cloture_la_demande_atomiquement() {
    let root = test_root("linked-reply-ack");
    let database = root.join(".cache/bridget/bridget.db");
    fs::create_dir_all(database.parent().expect("parent base")).expect("répertoire base");
    Store::open(&database)
        .expect("store initial")
        .create_request("request-open", "recipient", "human", 60)
        .expect("demande suivie initiale");

    let (sync, marker) = checkpoint_root(&root, "after_delivery_acked");
    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "linked-reply-instance");
    let issued_at = issued_at();
    let mut command = idempotent_send("linked-reply-ack".to_string(), issued_at);
    let WrapperToDaemon::SendIdempotent { message, .. } = &mut command else {
        unreachable!("commande idempotente attendue");
    };
    message.in_reply_to = Some("request-open".to_string());
    let mut client = negotiate_client(&socket_path);
    client.send(command);
    let _ = client.receive();
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);
    drop(recipient);

    let mut restarted = MatrixDaemonGuard::start(&root, &sync);
    let request = Store::open(&database)
        .expect("store après redémarrage")
        .get_request("request-open")
        .expect("demande lisible")
        .expect("demande présente");
    assert_eq!(request.state, "answered");
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage réponse liée");
}

#[test]
fn outil_mcp_rejette_la_reponse_liee_divergente_sans_muter_les_demandes() {
    let root = test_root("mcp-linked-mismatch");
    let database = root.join(".cache/bridget/bridget.db");
    fs::create_dir_all(database.parent().expect("parent base")).expect("répertoire base");
    let store = Store::open(&database).expect("store initial");
    store
        .create_request("request-a", "recipient", "human", 60)
        .expect("demande A initiale");
    store
        .create_request("request-b", "recipient", "human", 60)
        .expect("demande B initiale");

    let sync = root.join("sync");
    fs::create_dir_all(&sync).expect("synchronisation vide");
    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "mcp-linked-recipient");
    let mut mcp = McpProcess::start(&root, "human", "mcp-linked-instance");
    let initialize = mcp.request(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
    }));
    assert_eq!(initialize["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(serde_json::json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    }));

    let issued_at = issued_at();
    let call = |id, in_reply_to| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": "bridget_send",
                "arguments": {
                    "to": "recipient",
                    "body": "réponse MCP liée",
                    "in_reply_to": in_reply_to,
                    "id": "mcp-linked-retry",
                    "issued_at": issued_at
                }
            }
        })
    };
    // Le premier envoi est un DÉPÔT RÉUSSI, et ce banc le prouve bout en bout :
    // le destinataire reçoit sa remise dès la ligne suivante. L'annoncer
    // `outcome_unknown` gravait le défaut dans le contrat — c'est ce statut,
    // lu comme une perte, qui a fait diagnostiquer un canal cassé à un
    // relecteur et graver un constat bloquant faux au registre.
    let first = mcp.request(call(2, "request-a"));
    assert_eq!(first["result"]["structuredContent"]["status"], "in_flight");
    assert!(
        first["result"]["structuredContent"]["delivery_id"].is_string(),
        "un dépôt attesté doit publier la preuve qui le distingue d'un sort inconnu"
    );
    // Oracle (ii) : visible au ledger AVANT DeliverAcked, pendant dispatching.
    let store_before_ack = Store::open(&database).expect("store avant ack");
    let before_ack = store_before_ack
        .recent_messages(20)
        .expect("ledger avant ack");
    let en_vol = before_ack
        .iter()
        .find(|entry| entry.id == "mcp-linked-retry")
        .expect("message visible avant ack");
    assert_eq!(
        en_vol.delivery_phase.as_deref(),
        Some("dispatching"),
        "phase en vol avant DeliverAcked"
    );
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });

    let accepted = mcp.request(call(3, "request-a"));
    assert_eq!(
        accepted["result"]["structuredContent"]["status"],
        "accepted"
    );
    let tool_ledger = mcp.request(serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "bridget_ledger",
            "arguments": { "view": "messages", "limit": 20 }
        }
    }));
    let tool_messages = tool_ledger["result"]["structuredContent"]["messages"]
        .as_array()
        .expect("projection MCP messages");
    let tool_entry = tool_messages
        .iter()
        .find(|entry| entry["id"] == "mcp-linked-retry")
        .expect("message au ledger MCP après ack");
    assert_eq!(
        tool_entry["delivery_status"], "recu",
        "après DeliverAcked le DTO doit exposer reçu, pas en_vol"
    );
    let cli_ledger = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("ledger")
        .env("HOME", &root)
        .output()
        .expect("ledger CLI exécuté");
    assert!(cli_ledger.status.success(), "ledger CLI: {cli_ledger:?}");
    let cli_output = String::from_utf8(cli_ledger.stdout).expect("ledger CLI UTF-8");
    assert!(
        cli_output.contains("réponse MCP liée"),
        "le renderer CLI doit exposer l'envoi MCP livré: {cli_output}"
    );
    assert!(
        cli_output.contains("[reçu]"),
        "le renderer CLI doit marquer le message accusé: {cli_output}"
    );
    assert_eq!(
        store
            .recent_messages(20)
            .expect("ledger persistant lisible")
            .iter()
            .filter(|entry| entry.id == "mcp-linked-retry")
            .count(),
        1,
        "le ledger persistant ne contient qu'une remise émise (visibilité ≠ accusé)"
    );
    let before_a = store
        .get_request("request-a")
        .expect("demande A lisible")
        .expect("demande A présente");
    let before_b = store
        .get_request("request-b")
        .expect("demande B lisible")
        .expect("demande B présente");
    assert_eq!(before_a.state, "answered");
    assert_eq!(before_b.state, "open");

    let mismatch = mcp.request(call(5, "request-b"));
    assert_eq!(
        mismatch["result"]["structuredContent"]["status"],
        "envelope_mismatch"
    );
    assert_eq!(
        store
            .get_request("request-a")
            .expect("demande A finale lisible")
            .expect("demande A finale présente"),
        before_a
    );
    assert_eq!(
        store
            .get_request("request-b")
            .expect("demande B finale lisible")
            .expect("demande B finale présente"),
        before_b
    );
    mcp.stop();
    drop(recipient);
    daemon.stop();
    fs::remove_dir_all(root).expect("nettoyage MCP réponse liée");
}

#[test]
fn binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee() {
    let root = test_root("cli-mcp-linked-parity");
    let database = root.join(".cache/bridget/bridget.db");
    fs::create_dir_all(database.parent().expect("parent base")).expect("répertoire base");
    let store = Store::open(&database).expect("store initial");
    store
        .create_request("request-open", "recipient", "worker", 60)
        .expect("demande suivie initiale");

    let sync = root.join("sync");
    fs::create_dir_all(&sync).expect("synchronisation vide");
    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-parity-instance");

    let first = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            "recipient",
            "--in-reply-to",
            "request-open",
            "réponse liée paritaire",
        ],
    );
    // Le premier envoi est nominalement « sort inconnu » : le daemon répond
    // avant l'accusé du destinataire. Il a pourtant PRIS la remise, donc le
    // dépôt a réussi et le code de sortie doit le dire. Ce banc attestait
    // l'inverse — c'était le défaut, gravé en contrat.
    assert!(
        first.status.success(),
        "une remise en vol est un dépôt réussi: {}",
        output_text(&first)
    );
    assert!(output_text(&first).contains("en vol"));
    assert!(
        !output_text(&first).contains("perdu"),
        "rien n'est perdu tant que la remise est en vol: {}",
        output_text(&first)
    );
    let message_id = output_field(&first, "id");
    let sent_at = output_field(&first, "issued_at")
        .parse::<i64>()
        .expect("issued_at CLI entier");
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            delivery_generation,
            ..
        } => {
            assert_eq!(message.id, message_id);
            assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
            (delivery_id, delivery_generation)
        }
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });

    let retry = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            "recipient",
            "--in-reply-to",
            "request-open",
            "--id",
            &message_id,
            "--issued-at",
            &sent_at.to_string(),
            "réponse liée paritaire",
        ],
    );
    assert!(retry.status.success(), "retry CLI: {}", output_text(&retry));
    assert!(output_text(&retry).contains("accepted"));
    assert_no_delivery(&mut recipient);

    let mut mcp = McpProcess::start(&root, "worker", "shared-cli-mcp-instance");
    let initialized = mcp.request(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
    }));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(serde_json::json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    }));
    let mcp_retry = mcp.request(mcp_send_call(
        2,
        &message_id,
        sent_at,
        "réponse liée paritaire",
    ));
    assert_eq!(
        mcp_retry["result"]["structuredContent"]["status"],
        "accepted"
    );
    assert_no_delivery(&mut recipient);

    let duplicate_id = "cli-linked-new-id";
    let duplicate_at = issued_at();
    let duplicate = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            "recipient",
            "--in-reply-to",
            "request-open",
            "--id",
            duplicate_id,
            "--issued-at",
            &duplicate_at.to_string(),
            "réponse liée paritaire",
        ],
    );
    assert!(!duplicate.status.success());
    assert!(
        output_text(&duplicate).contains("REJET: duplicate"),
        "projection CLI du doublon: {}",
        output_text(&duplicate)
    );
    assert_no_delivery(&mut recipient);

    recipient.send(WrapperToDaemon::Availability {
        agent: "recipient".to_string(),
        until_secs: Some((issued_at() + 60) as u64),
    });
    assert!(matches!(recipient.receive(), DaemonToWrapper::Ack { .. }));
    let closed_at = issued_at();
    let closed = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            "recipient",
            "--in-reply-to",
            "request-open",
            "--id",
            "cli-terminal-request",
            "--issued-at",
            &closed_at.to_string(),
            "message ordinaire après clôture",
        ],
    );
    assert!(!closed.status.success());
    assert!(
        output_text(&closed).contains("REJET: dnd"),
        "D-208 doit conserver DND côté CLI: {}",
        output_text(&closed)
    );
    let mcp_closed = mcp.request(mcp_send_call(
        3,
        "mcp-terminal-request",
        issued_at(),
        "autre message ordinaire après clôture",
    ));
    assert_eq!(mcp_closed["result"]["structuredContent"]["status"], "dnd");
    assert_no_delivery(&mut recipient);
    assert_eq!(
        store
            .get_request("request-open")
            .expect("demande lisible")
            .expect("demande présente")
            .state,
        "answered"
    );

    mcp.stop();
    drop(recipient);
    daemon.stop();
    fs::remove_dir_all(root).expect("nettoyage parité CLI MCP");
}

#[test]
fn recovery_acked_wrapper_finalise_accepted_apres_crash_daemon() {
    let root = test_root("recovery-acked-wrapper");
    let daemon = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-acked-wrapper-instance");
    let issued_at = issued_at();
    let message_id = "recovery-acked-wrapper".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    let _ = client.receive();
    let first = receive_delivery(&mut recipient);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-acked-wrapper-instance");
    let redelivery = receive_delivery(&mut recipient);
    assert_eq!(
        encode(&first).expect("première remise encodable"),
        encode(&redelivery).expect("remise reprise encodable")
    );
    let (delivery_id, delivery_generation) = match redelivery {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::Accepted { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage acked wrapper");
}

#[test]
fn recovery_seen_indeterminate_maintient_outcome_unknown_apres_crash_daemon() {
    let root = test_root("recovery-indeterminate");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_indeterminate");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-indeterminate-instance");
    let issued_at = issued_at();
    let message_id = "recovery-indeterminate".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    let _ = client.receive();
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliveryIndeterminate {
        delivery_id,
        delivery_generation,
    });
    watch_marker(&sync, &marker);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-indeterminate-instance");
    assert_no_delivery(&mut recipient);
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::OutcomeUnknown { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage indéterminé");
}

#[test]
fn recovery_absent_redelivre_sans_doublon_apres_crash_daemon() {
    let root = test_root("recovery-absent");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_before_issue");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let recipient = register_recipient_as(&socket_path, "recovery-absent-instance");
    let issued_at = issued_at();
    let message_id = "recovery-absent".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    watch_marker(&sync, &marker);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-absent-instance");
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::Accepted { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage absent");
}

#[test]
fn recovery_prepared_reprend_le_dispatch_apres_crash_daemon() {
    let root = test_root("recovery-prepared");
    let (sync, marker) = checkpoint_root(&root, "after_prepared");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let recipient = register_recipient_as(&socket_path, "recovery-prepared-instance");
    let issued_at = issued_at();
    let message_id = "recovery-prepared".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    watch_marker(&sync, &marker);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-prepared-instance");
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::Accepted { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage prepared");
}

#[test]
fn recovery_terminal_acked_vrai_wrapper_rejoue_sans_second_prompt() {
    let root = test_root("acp-daemon-restart");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_acked");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(1);
    let wrapper_home = registry_root.join("wrapper-home");
    fs::create_dir_all(&wrapper_home).expect("home wrapper ACP");
    let wrapper_registry = registry.clone();
    let wrapper_socket = socket_path.clone();
    let wrapper = thread::spawn(move || {
        launch_acp_with(
            "fixture-acp",
            &[],
            Some("acp-recipient"),
            &wrapper_registry,
            &wrapper_socket,
            &wrapper_home,
        )
        .map_err(|error| error.to_string())
    });
    wait_for_registered_agent(&socket_path, "acp-recipient");
    let issued_at = issued_at();
    let message_id = "vrai-wrapper-acp".to_string();
    let mut client = negotiate_client(&socket_path);
    let mut message = BridgetMessage::new("human", "acp-recipient", "frame réelle");
    message.id = message_id.clone();
    let command = WrapperToDaemon::SendIdempotent {
        message,
        message_id: message_id.clone(),
        issued_at,
    };
    client.send(command.clone());
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    wait_for_counter(&counter, 1);
    // L'accusé provient du wrapper réel. Le jalon est atteint après son
    // commit durable, mais avant que le daemon puisse poursuivre son cycle.
    watch_marker(&sync, &marker);

    daemon.stop();
    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    wait_for_accepted(&socket_path, &command);
    thread::sleep(Duration::from_millis(250));
    assert_eq!(fs::read(&counter).expect("compteur ACP"), b"x");
    restarted.stop();
    assert_eq!(wrapper.join().expect("thread wrapper"), Ok(()));
    fs::remove_dir_all(root).expect("nettoyage daemon ACP");
    fs::remove_dir_all(registry_root).expect("nettoyage registre ACP");
}
