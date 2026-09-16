//! Façade CLI réelle, socket isolée. Pas de daemon ni fournisseur enfant,
//! pas de watchdog SIGKILL ; le client termine naturellement.
use bridget_transport::protocol::{AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::time::Duration;

#[test]
fn spec100_cli_share_and_events_use_registered_identity() {
    use bridget_transport::protocol::{IdentityCredential, ObservationRequest};
    use sha2::{Digest, Sha256};
    const AGENT: &str = "10000000-0000-4000-8000-000000000001";
    const TARGET: &str = "10000000-0000-4000-8000-000000000002";
    let root = std::env::temp_dir().join(format!("bg100-share-{}", uuid::Uuid::new_v4().simple()));
    bridget_transport::fsutil::create_private_dir(&root).unwrap();
    let socket = root.join("s");
    let name = root.join("name");
    bridget_transport::fsutil::write_private_file_atomic(&name, AGENT.as_bytes()).unwrap();
    let proof = root
        .join("agent-names")
        .join(format!("proof-{:x}.json", Sha256::digest(b"instance100")));
    // Preuve fictive uniquement pour un serveur socket simulé, jamais présentée
    // au daemon installé ; l'autorisation réelle est testée dans daemon.rs.
    bridget_transport::fsutil::write_private_file_atomic(&proof,&serde_json::to_vec(&json!({"agent_id":AGENT,"instance_id":"instance100","credential":IdentityCredential::new("fixture-only".into())})).unwrap()).unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    let server = std::thread::spawn(move || {
        for phase in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut read = || {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                decode::<WrapperToDaemon>(line.trim()).unwrap()
            };
            let mut write = |response| {
                writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
            };
            if phase == 0 {
                assert!(matches!(
                    read(),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Attach
                    }
                ));
                write(DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Attach,
                });
                assert!(matches!(read(), WrapperToDaemon::Subscribe { .. }));
                write(DaemonToWrapper::Subscribed {
                    subscription_id: "s".into(),
                });
                write(DaemonToWrapper::JournalFragment {
                    subscription_id: "s".into(),
                    seq: 1,
                    offset: 0,
                    final_fragment: true,
                    bytes: serde_json::to_vec(&json!({"seq":1,"text":"preuve CLI 🦀"})).unwrap(),
                });
                write(DaemonToWrapper::SnapshotCaughtUp {
                    subscription_id: "s".into(),
                    through_seq: Some(1),
                });
            } else {
                assert!(
                    matches!(read(),WrapperToDaemon::RegisterAuxiliary{agent_id,instance_id,..} if agent_id==AGENT && instance_id=="instance100")
                );
                write(DaemonToWrapper::Registered {
                    agent_id: AGENT.into(),
                    credential: None,
                });
                if phase == 1 {
                    let WrapperToDaemon::Send(message) = read() else {
                        panic!("envoi attendu")
                    };
                    assert_eq!(message.to, TARGET);
                    assert_eq!(message.from, AGENT);
                    assert!(message.reply);
                    assert!(message.body.contains("preuve CLI 🦀"));
                    write(DaemonToWrapper::Ack { id: message.id });
                } else {
                    assert!(matches!(
                        read(),
                        WrapperToDaemon::ObservationRequest {
                            request: ObservationRequest::Types {}
                        }
                    ));
                    write(DaemonToWrapper::ObservationResult {
                        result: json!({"events":[],"fixture":true}),
                    });
                }
            }
        }
    });
    for args in [
        vec!["journal", AGENT, "--to", TARGET, "--reply"],
        vec!["events", "types"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args(&args)
            .env_clear()
            .env("HOME", &root)
            .env("BRIDGET_HOME", &root)
            .env("BRIDGET_SOCKET", &socket)
            .env("BRIDGET_AGENT_ID_FILE", &name)
            .env("BRIDGET_AGENT_INSTANCE_ID", "instance100")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    server.join().unwrap();
}

#[test]
fn spec100_cli_journal_real_socket_and_parameter_errors() {
    let root = std::env::temp_dir().join(format!("bg100-cli-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = root.join("s");
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(std::time::Instant::now() < deadline, "client absent");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept: {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        for response in [
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach,
            },
            DaemonToWrapper::Subscribed {
                subscription_id: "s".into(),
            },
        ] {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let command: WrapperToDaemon = decode(line.trim()).unwrap();
            assert!(matches!(
                command,
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach
                } | WrapperToDaemon::Subscribe {
                    window: AttachWindow::Tail(2),
                    ..
                }
            ));
            writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
        }
        for response in [
            DaemonToWrapper::JournalFragment {
                subscription_id: "s".into(),
                seq: 9,
                offset: 0,
                final_fragment: true,
                bytes: serde_json::to_vec(&json!({"seq":9,"text":"relecture é 🦀"})).unwrap(),
            },
            DaemonToWrapper::SnapshotCaughtUp {
                subscription_id: "s".into(),
                through_seq: Some(9),
            },
        ] {
            writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
        }
    });
    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["journal", "source", "--tail", "2"])
        .env("BRIDGET_HOME", &root)
        .env("BRIDGET_SOCKET", &socket)
        .env("HOME", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
    let excerpt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(excerpt["entries"][0]["text"], "relecture é 🦀");
    assert_eq!(excerpt["next_seq"], 10);
    for args in [
        vec!["journal", "source", "--tail", "1", "--from-seq", "1"],
        vec!["journal", "source", "--reply"],
        vec!["journal", "source", "--tail", "201"],
        vec!["events", "list", "--owner", "victim"],
        vec!["events", "sub", "invented"],
        vec!["events", "sub", "turn_ended", "--once", "--once"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args(&args)
            .env("BRIDGET_HOME", &root)
            .env("BRIDGET_SOCKET", root.join("absent"))
            .env("HOME", &root)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "pas de faux reçu pour {args:?}");
    }
}
