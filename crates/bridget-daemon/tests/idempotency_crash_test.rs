#![cfg(feature = "test-support")]

//! Matrice de crash inter-processus du contrat idempotent.
//!
//! Les deux frontières amont sont exécutées ici dès que le socle et Lookup
//! sont disponibles. Les frontières aval sont ajoutées avec leur relais
//! `DeliverIdempotent` : elles partagent les mêmes jalons et le même harnais.

use bridget_core::BridgetMessage;
use bridget_daemon::test_sync::DIRECTORY_ENV;
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::wrapper::launch_acp_with;
use bridget_transport::protocol::{
    decode, encode, ClientCapability, ConnectionRole, IdempotencyIssue, CLIENT_CONTRACT_VERSION,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::ffi::CString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MATRIX_CYCLES: usize = 50;
const GLOBAL_TIMEOUT: Duration = Duration::from_secs(90);
const READY_TIMEOUT: Duration = Duration::from_secs(5);
const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(5);
const SCOPE: &str = "abcdefghijklmnopqrstuv";

struct DaemonProcess {
    child: Child,
    logs: Option<thread::JoinHandle<()>>,
}

impl DaemonProcess {
    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(logs) = self.logs.take() {
            let _ = logs.join();
        }
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

impl Client {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).expect("connexion au daemon");
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
    let mut child = command.spawn().expect("spawn daemon réel");
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
        os: Some("test".to_string()),
        instance_id: Some(instance_id.to_string()),
        domain: None,
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

fn registry_with_counting_acp_agent() -> (AgentRegistry, PathBuf, PathBuf) {
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
read prompt
printf x >> '{}'
echo '{{"jsonrpc":"2.0","id":3,"result":{{"stopReason":"end_turn"}}}}'
sleep 5
"#, counter.display())],
                "protocol": "acp",
                "permissions": "allow",
                "queue_capacity": 2,
                "notify_timeout_secs": 1
            }
        }
    });
    let path = directory.join("agents.json");
    fs::write(&path, serde_json::to_string_pretty(&definition).expect("registre sérialisable"))
        .expect("registre ACP écrit");
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
        assert!(Instant::now() < deadline, "frame session/prompt absente dans la borne");
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
        assert!(Instant::now() < deadline, "wrapper ACP non enregistré dans la borne");
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
fn matrice_crash_amont_rejoue_les_issues_sans_reservation_dupliquee() {
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let result = std::panic::catch_unwind(|| {
            for serial in 0..MATRIX_CYCLES {
                let point = if serial % 2 == 0 {
                    "before_reservation"
                } else {
                    "after_prepared"
                };
                run_amont_cycle(point, serial);
            }
        });
        let _ = done_tx.send(result);
    });
    match done_rx.recv_timeout(GLOBAL_TIMEOUT) {
        Ok(Ok(())) => {}
        Ok(Err(payload)) => std::panic::resume_unwind(payload),
        Err(_) => panic!("matrice T1209 dépassée après {GLOBAL_TIMEOUT:?}"),
    }
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
    let (registry, registry_root, counter) = registry_with_counting_acp_agent();
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
    wait_for_registered_agent(&socket_path, "acp-recipient");
    wait_for_accepted(&socket_path, &command);
    thread::sleep(Duration::from_millis(250));
    assert_eq!(fs::read(&counter).expect("compteur ACP"), b"x");
    restarted.stop();
    assert_eq!(wrapper.join().expect("thread wrapper"), Ok(()));
    fs::remove_dir_all(root).expect("nettoyage daemon ACP");
    fs::remove_dir_all(registry_root).expect("nettoyage registre ACP");
}
