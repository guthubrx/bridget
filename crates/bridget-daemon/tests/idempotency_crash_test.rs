#![cfg(feature = "test-support")]

//! Matrice de crash inter-processus du contrat idempotent.
//!
//! Les deux frontières amont sont exécutées ici dès que le socle et Lookup
//! sont disponibles. Les frontières aval sont ajoutées avec leur relais
//! `DeliverIdempotent` : elles partagent les mêmes jalons et le même harnais.

use bridget_core::BridgetMessage;
use bridget_daemon::test_sync::DIRECTORY_ENV;
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
    let mut recipient = Client::connect(socket);
    recipient.send(WrapperToDaemon::Register {
        agent_type: "fixture".to_string(),
        name: Some("recipient".to_string()),
        host: Some("t1209".to_string()),
        transport: Some("acp".to_string()),
        os: Some("test".to_string()),
        instance_id: Some("t1209-recipient".to_string()),
        domain: None,
        turn_in_progress: false,
    });
    assert!(matches!(
        recipient.receive(),
        DaemonToWrapper::Registered { .. }
    ));
    recipient
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
    let mut client = negotiate_client(socket);
    client.send(idempotent_send(message_id, issued_at));
    match client.receive() {
        DaemonToWrapper::IdempotencyResult { issue, .. } => issue,
        other => panic!("issue idempotente attendue: {other:?}"),
    }
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
