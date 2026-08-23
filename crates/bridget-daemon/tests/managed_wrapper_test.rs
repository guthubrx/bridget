use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStatus, ManagedStopResult,
    spawn_managed_bootstrap,
};
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn test_root() -> PathBuf {
    PathBuf::from(format!(
        "/tmp/bg906-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

#[test]
fn wrapper_claude_gere_annonce_un_register_acp_complet() {
    let root = test_root();
    let expected_domain = root
        .file_name()
        .expect("racine de test nommée")
        .to_string_lossy()
        .into_owned();
    let socket = root.join(".cache/bridget/bridget.sock");
    let registry = root.join(".config/bridget/agents.json");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(
        &registry,
        r#"{"agents":{"claude":{"command":"/adaptateur/t906-absent","protocol":"acp","permissions":"allow","queue_capacity":2,"notify_timeout_secs":1}}}"#,
    )
    .unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let (register_tx, register_rx) = mpsc::channel();
    let (accept_tx, accept_rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let register = decode::<WrapperToDaemon>(line.trim_end()).unwrap();
        register_tx.send(register).unwrap();
        accept_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                name: "claude-manage-1".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        line.clear();
        let _ = reader.read_line(&mut line);
    });

    let identity = ManagedIdentity {
        instance_id: "instance-wrapper-reel".to_string(),
        command_id: "command-wrapper-reel".to_string(),
        generation: 3,
    };
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity: identity.clone(),
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".to_string(),
            "claude".to_string(),
            "claude-manage-1".to_string(),
        ],
        cwd: root.clone(),
        env: BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("C")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
        ]),
    };
    let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
    let ready = spawn_managed_bootstrap(&launch)
        .unwrap()
        .wait_ready()
        .unwrap();
    assert_eq!(ready.ready().instance_id, identity.instance_id);
    let mut running = ready
        .persist_marker(&marker_store, "claude-manage-1")
        .unwrap()
        .release()
        .unwrap();
    match register_rx.recv_timeout(Duration::from_secs(2)).unwrap() {
        WrapperToDaemon::Register {
            agent_type,
            name,
            transport,
            instance_id,
            domain,
            ..
        } => {
            assert_eq!(agent_type, "claude");
            assert_eq!(name.as_deref(), Some("claude-manage-1"));
            assert_eq!(transport.as_deref(), Some("acp"));
            assert_eq!(instance_id.as_deref(), Some(identity.instance_id.as_str()));
            assert_eq!(domain.as_deref(), Some(expected_domain.as_str()));
        }
        other => panic!("Register Claude géré attendu, reçu : {other:?}"),
    }

    running
        .status_reader()
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut line = String::new();
    let before_register = running.status_reader().read_line(&mut line).unwrap_err();
    assert!(matches!(
        before_register.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    ));

    accept_tx.send(()).unwrap();
    running
        .status_reader()
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    line.clear();
    running.status_reader().read_line(&mut line).unwrap();
    assert_eq!(
        serde_json::from_str::<ManagedStatus>(line.trim_end()).unwrap(),
        ManagedStatus::StartupFailed {
            kind: "command_missing".to_string(),
            reason: "/adaptateur/t906-absent".to_string(),
            instance_id: identity.instance_id,
            command_id: identity.command_id,
            generation: identity.generation,
        }
    );
    running.close_status();
    assert_eq!(running.child_mut().wait().unwrap().code(), Some(1));
    daemon.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn arret_du_wrapper_reel_termine_l_adaptateur_qui_ignore_l_annulation() {
    let root = test_root();
    let socket = root.join(".cache/bridget/bridget.sock");
    let registry = root.join(".config/bridget/agents.json");
    let adapter = root.join("adaptateur-ignore-cancel.sh");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(
        &adapter,
        r#"#!/bin/sh
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
: > "$HOME/prompt-seen"
while :; do :; done
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        &registry,
        serde_json::to_vec(&serde_json::json!({
            "agents": {
                "fixture-ignore-cancel": {
                    "command": adapter,
                    "protocol": "acp",
                    "permissions": "allow",
                    "queue_capacity": 2,
                    "notify_timeout_secs": 1
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let (registered_tx, registered_rx) = mpsc::channel();
    let (disconnect_tx, disconnect_rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
            WrapperToDaemon::Register { .. }
        ));
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                name: "fixture-ignore-cancel-1".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Deliver(BridgetMessage::new(
                "humain",
                "fixture-ignore-cancel-1",
                "tour bloqué",
            )))
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        registered_tx.send(()).unwrap();
        disconnect_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
        writer.flush().unwrap();

        let mut unregistered = false;
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    unregistered |= matches!(
                        decode::<WrapperToDaemon>(line.trim_end()),
                        Ok(WrapperToDaemon::Unregister)
                    );
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("lecture daemon impossible: {error}"),
            }
        }
        unregistered
    });

    let identity = ManagedIdentity {
        instance_id: "instance-ignore-cancel".to_string(),
        command_id: "command-ignore-cancel".to_string(),
        generation: 4,
    };
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity,
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".to_string(),
            "fixture-ignore-cancel".to_string(),
            "fixture-ignore-cancel-1".to_string(),
        ],
        cwd: root.clone(),
        env: BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("C")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
        ]),
    };
    let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
    let ready = spawn_managed_bootstrap(&launch)
        .unwrap()
        .wait_ready()
        .unwrap();
    let mut running = ready
        .persist_marker(&marker_store, "fixture-ignore-cancel-1")
        .unwrap()
        .release()
        .unwrap();
    let marker_path = running.marker().path().to_path_buf();
    registered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let prompt_seen = root.join("prompt-seen");
    for _ in 0..100 {
        if prompt_seen.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        prompt_seen.exists(),
        "le faux adaptateur n'a pas lu le prompt"
    );

    disconnect_tx.send(()).unwrap();
    assert!(matches!(
        running
            .stop_group(
                Duration::from_secs(3),
                Duration::from_secs(1),
                Duration::from_millis(10),
            )
            .unwrap(),
        ManagedStopResult::Stopped | ManagedStopResult::StoppedForced { .. }
    ));
    assert!(!marker_path.exists());
    assert!(
        daemon.join().unwrap(),
        "le wrapper n'a pas envoyé Unregister"
    );
    fs::remove_dir_all(root).unwrap();
}
