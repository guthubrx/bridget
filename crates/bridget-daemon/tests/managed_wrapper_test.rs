use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStopResult, spawn_managed_bootstrap,
};
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::wrapper::launch_acp_with;
use bridget_transport::protocol::{PresenceMode, decode, encode};
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
fn wrapper_claude_natif_livre_une_mission_repond_et_tient_un_journal() {
    let root = test_root();
    let socket = root.join(".cache/bridget/bridget.sock");
    let registry_path = root.join(".config/bridget/agents.json");
    let adapter = root.join("claude-stream-json-fixture.sh");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    fs::write(
        &adapter,
        r#"#!/bin/sh
while IFS= read -r line; do
  printf '%s\n' '{"type":"system","subtype":"init","model":"claude-opus-5"}'
  printf '%s\n' '{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"claude-opus-5: mission reçue"}}}'
  printf '%s\n' '{"type":"result","is_error":false,"terminal_reason":"completed","result":"claude-opus-5: mission reçue"}'
done
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let registry_json = serde_json::json!({
        "agents": {
            "claude-native": {
                "command": adapter,
                "args": ["--model", "claude-opus-5"],
                "protocol": "claude_stream_json",
                "permissions": "allow",
                "queue_capacity": 2,
                "notify_timeout_secs": 2
            }
        }
    })
    .to_string();
    fs::write(&registry_path, &registry_json).unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let daemon = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        match decode(line.trim_end()).unwrap() {
            WrapperToDaemon::Register {
                agent_type,
                transport,
                mode,
                location,
                ..
            } => {
                assert_eq!(agent_type, "claude-native");
                assert_eq!(transport.as_deref(), Some("stdio"));
                assert_eq!(mode, Some(PresenceMode::Cli));
                assert_eq!(location, None);
            }
            other => panic!("Register Claude natif attendu, reçu : {other:?}"),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                name: "claude-native-1".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();

        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if matches!(
                decode(line.trim_end()).unwrap(),
                WrapperToDaemon::JournalReady
            ) {
                break;
            }
        }
        let mut mission = BridgetMessage::new("demandeur", "claude-native-1", "mission native");
        mission.id = "mission-claude-native".to_string();
        mission.reply = true;
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Deliver(mission)).unwrap()
        )
        .unwrap();
        writer.flush().unwrap();

        let mut answered = false;
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            match decode(line.trim_end()).unwrap() {
                WrapperToDaemon::Send(reply) => {
                    assert_eq!(reply.from, "claude-native-1");
                    assert_eq!(reply.to, "demandeur");
                    assert_eq!(reply.in_reply_to.as_deref(), Some("mission-claude-native"));
                    assert_eq!(reply.body, "claude-opus-5: mission reçue");
                    answered = true;
                    writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
                    writer.flush().unwrap();
                }
                WrapperToDaemon::Unregister => return answered,
                _ => {}
            }
        }
    });

    let registry = AgentRegistry::from_json(&registry_json, &registry_path).unwrap();
    let wrapper_root = root.clone();
    let wrapper_socket = socket.clone();
    let wrapper = thread::spawn(move || {
        launch_acp_with(
            "claude-native",
            &[],
            Some("claude-native-1"),
            &registry,
            &wrapper_socket,
            &wrapper_root,
        )
        .map_err(|error| error.to_string())
    });

    assert_eq!(wrapper.join().unwrap(), Ok(()));
    assert!(daemon.join().unwrap());
    let sessions = root.join(".cache/bridget/sessions/claude-native-1");
    let entries = fs::read_dir(&sessions).unwrap().count();
    assert!(
        entries > 0,
        "journal Claude natif absent: {}",
        sessions.display()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn wrapper_claude_gere_annonce_un_register_natif_complet() {
    let root = test_root();
    let expected_domain = root
        .file_name()
        .expect("racine de test nommée")
        .to_string_lossy()
        .into_owned();
    let socket = root.join(".cache/bridget/bridget.sock");
    let registry = root.join(".config/bridget/agents.json");
    let adapter = root.join("claude-stream-json-managed.sh");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(
        &adapter,
        r#"#!/bin/sh
printf '%s' "$HOME" > "$HOME/claude-home-seen"
while IFS= read -r line; do :; done
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let registry_json = serde_json::json!({
        "agents": {
            "claude": {
                "command": adapter,
                "args": ["--model", "claude-opus-5"],
                "protocol": "claude_stream_json",
                "permissions": "allow",
                "queue_capacity": 2,
                "notify_timeout_secs": 1
            }
        }
    })
    .to_string();
    fs::write(&registry, &registry_json).unwrap();
    fs::set_permissions(&registry, fs::Permissions::from_mode(0o600)).unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let (register_tx, register_rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let register = decode::<WrapperToDaemon>(line.trim_end()).unwrap();
        register_tx.send(register).unwrap();
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
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if matches!(
                decode(line.trim_end()).unwrap(),
                WrapperToDaemon::JournalReady
            ) {
                writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
                writer.flush().unwrap();
                break;
            }
        }
    });

    let identity = ManagedIdentity {
        instance_id: "instance-wrapper-reel".to_string(),
        command_id: "command-wrapper-reel".to_string(),
        generation: 3,
    };
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let frozen_definition = AgentRegistry::from_json(&registry_json, &registry)
        .unwrap()
        .resolved_definition("claude")
        .unwrap();
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity: identity.clone(),
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".to_string(),
            "claude".to_string(),
            "claude-manage-1".to_string(),
            serde_json::to_string(&frozen_definition).unwrap(),
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
            mode,
            instance_id,
            domain,
            ..
        } => {
            assert_eq!(agent_type, "claude");
            assert_eq!(name.as_deref(), Some("claude-manage-1"));
            assert_eq!(transport.as_deref(), Some("stdio"));
            assert_eq!(mode, Some(PresenceMode::Cli));
            assert_eq!(instance_id.as_deref(), Some(identity.instance_id.as_str()));
            assert_eq!(domain.as_deref(), Some(expected_domain.as_str()));
        }
        other => panic!("Register Claude géré attendu, reçu : {other:?}"),
    }
    let mut line = String::new();
    running
        .status_reader()
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    assert_eq!(running.status_reader().read_line(&mut line).unwrap(), 0);
    running.close_status();
    assert_eq!(running.child_mut().wait().unwrap().code(), Some(0));
    daemon.join().unwrap();
    assert_eq!(
        fs::read_to_string(root.join("claude-home-seen")).unwrap(),
        root.display().to_string(),
        "le CLI Claude géré doit recevoir le HOME transmis par le daemon"
    );
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
    let registry_json = serde_json::json!({
        "agents": {
            "fixture-ignore-cancel": {
                "command": adapter,
                "protocol": "acp",
                "permissions": "allow",
                "queue_capacity": 2,
                "notify_timeout_secs": 1
            }
        }
    })
    .to_string();
    fs::write(&registry, &registry_json).unwrap();
    fs::set_permissions(&registry, fs::Permissions::from_mode(0o600)).unwrap();
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
    let frozen_definition = AgentRegistry::from_json(&registry_json, &registry)
        .unwrap()
        .resolved_definition("fixture-ignore-cancel")
        .unwrap();
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity,
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".to_string(),
            "fixture-ignore-cancel".to_string(),
            "fixture-ignore-cancel-1".to_string(),
            serde_json::to_string(&frozen_definition).unwrap(),
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
