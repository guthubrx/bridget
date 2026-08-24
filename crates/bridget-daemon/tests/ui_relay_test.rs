//! Couture du relais UI : HTTP loopback → projections publiques Bridget → Attach.

use bridget_core::BridgetMessage;
use bridget_daemon::ui::{UiRelay, UiRelayConfig};
use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct DaemonProcess(Child);

impl DaemonProcess {
    fn start(home: &Path) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("daemon")
            .env_clear()
            .env("HOME", home)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("daemon réel démarré");
        let daemon = Self(child);
        let socket = home.join(".cache/bridget/bridget.sock");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&socket).is_ok() {
                return daemon;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("daemon non prêt");
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) }, 0);
            let _ = self.0.wait();
        }
    }
}

struct LiveAgent {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
}

impl LiveAgent {
    fn connect(socket: &Path, name: &str) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        let read_stream = stream.try_clone().unwrap();
        let mut agent = Self {
            writer: BufWriter::new(stream),
            reader: BufReader::new(read_stream),
        };
        agent.send(&WrapperToDaemon::Register {
            agent_type: "ui-test".to_string(),
            name: Some(name.to_string()),
            host: Some("test".to_string()),
            transport: Some("unix".to_string()),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: Some("test".to_string()),
            instance_id: Some(format!("ui-test-{name}")),
            domain: Some("test".to_string()),
            turn_in_progress: false,
            journal_available: Some(false),
        });
        assert!(matches!(agent.read(), DaemonToWrapper::Registered { .. }));
        agent.send(&WrapperToDaemon::JournalReady);
        agent
    }

    fn send(&mut self, message: &WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn read(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }
}

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bui-{label}-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    root
}

fn write_maicie_config(root: &Path) -> PathBuf {
    let path = root.join("maicie.json");
    let socket = root.join(".cache/bridget/bridget.sock");
    let database = root.join("maicie.sqlite");
    let json = serde_json::json!({
        "version": 1,
        "bridget_socket": socket,
        "database_path": database,
        "durations": {"short_secs": 1, "normal_secs": 2, "long_secs": 3},
        "profiles": []
    });
    std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    path
}

fn request(address: SocketAddr, path: &str) -> TcpStream {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .unwrap();
    stream
}

fn read_until(stream: &mut TcpStream, expected: &str) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let mut one = [0_u8; 1];
        match stream.read(&mut one) {
            Ok(0) => break,
            Ok(_) => {
                bytes.push(one[0]);
                let text = String::from_utf8_lossy(&bytes);
                if text.contains(expected) {
                    return text.into_owned();
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(error) => panic!("lecture HTTP: {error}"),
        }
    }
    String::from_utf8(bytes).unwrap()
}

#[test]
fn loopback_rend_snapshot_et_relaie_un_fragment_attach_d_un_agent_vivant() {
    let root = root("couteau");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut agent = LiveAgent::connect(&socket, "agent-vivant");
    let mut demandeur = LiveAgent::connect(&socket, "demandeur");
    let mut demande = BridgetMessage::new("demandeur", "agent-vivant", "demande ouverte");
    demande.reply = true;
    let demande_id = demande.id.clone();
    demandeur.send(&WrapperToDaemon::Send(demande));
    assert!(matches!(demandeur.read(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(agent.read(), DaemonToWrapper::Deliver(_)));
    let config = UiRelayConfig {
        daemon_socket: socket,
        maicie_config: write_maicie_config(&root),
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-couture".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let mut snapshot = request(address, "/v1/snapshot?token=jeton-couture");
    let mut snapshot_text = read_until(&mut snapshot, "agent-vivant");
    let mut snapshot_tail = String::new();
    snapshot
        .read_to_string(&mut snapshot_tail)
        .expect("fin de réponse snapshot");
    snapshot_text.push_str(&snapshot_tail);
    assert!(snapshot_text.starts_with("HTTP/1.1 200"), "{snapshot_text}");
    assert!(
        snapshot_text.contains("\"missions\":{\"version\":1"),
        "{snapshot_text}"
    );
    assert!(
        snapshot_text.contains(&demande_id),
        "la projection globale doit inclure la demande suivie réelle; {snapshot_text}"
    );

    let mut events = request(address, "/v1/watch?token=jeton-couture&agent=agent-vivant");
    let subscription_id = match agent.read() {
        DaemonToWrapper::Subscribe {
            subscription_id,
            agent,
            ..
        } => {
            assert_eq!(agent, "agent-vivant");
            subscription_id
        }
        response => panic!("Subscribe wrapper attendu, reçu {response:?}"),
    };
    agent.send(&WrapperToDaemon::Subscribed {
        subscription_id: subscription_id.clone(),
    });
    agent.send(&WrapperToDaemon::JournalFragment {
        subscription_id: subscription_id.clone(),
        seq: 1,
        offset: 0,
        final_fragment: true,
        bytes: b"fragment reel".to_vec(),
    });
    agent.send(&WrapperToDaemon::SnapshotCaughtUp {
        subscription_id,
        through_seq: Some(1),
    });
    let events = read_until(&mut events, "ZnJhZ21lbnQgcmVlbA");
    assert!(events.starts_with("HTTP/1.1 200"), "{events}");
    assert!(events.contains("event: snapshot"), "{events}");
    assert!(events.contains("event: journal"), "{events}");
    assert!(events.contains("JournalFragment"), "{events}");
    assert!(events.contains("ZnJhZ21lbnQgcmVlbA"), "{events}");
    let snapshot_position = events.find("event: snapshot").unwrap();
    let fragment_position = events.find("JournalFragment").unwrap();
    assert!(
        snapshot_position < fragment_position,
        "Mutation : publier l'instantané avant Subscribe ferait perdre le fragment entre les deux; {events}"
    );

    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn requete_loopback_sans_jeton_est_refusee() {
    let config = UiRelayConfig {
        daemon_socket: PathBuf::from("/tmp/ui-inaccessible.sock"),
        maicie_config: PathBuf::from("/tmp/maicie-inaccessible.json"),
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "secret".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());
    let mut response = request(address, "/v1/snapshot");
    let response = read_until(&mut response, "403");
    assert!(response.starts_with("HTTP/1.1 403"), "{response}");
}
