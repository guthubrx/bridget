use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStopResult, spawn_managed_bootstrap,
};
use bridget_daemon::registry::AgentRegistry;
use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn test_root() -> PathBuf {
    let root = PathBuf::from(format!(
        "/tmp/bg906-{}-{}",
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

fn accept_peer(listener: UnixListener) -> UnixStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return stream;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "aucun Register réel reçu");
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("accept : {error}"),
        }
    }
}

fn run_wrapper(root: &Path, agent_type: &str, agent_id: &str) -> Result<(), String> {
    let registry_path = root.join("state/agents.json");
    let registry = AgentRegistry::from_json(
        &fs::read_to_string(&registry_path).map_err(|error| error.to_string())?,
        &registry_path,
    )
    .map_err(|error| error.to_string())?;
    let definition = registry
        .get(agent_type)
        .map_err(|error| error.to_string())?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args([
            "--",
            definition.command.as_str(),
            "--equipier",
            "--agent-id",
            agent_id,
        ])
        .env_clear()
        .env("HOME", root)
        .env("BRIDGET_HOME", root.join("state"))
        .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
        .env("BRIDGET_CHANNEL", "unix")
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return if status.success() {
                Ok(())
            } else {
                Err(status.to_string())
            };
        }
        if Instant::now() >= deadline {
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
            let stop = Instant::now() + Duration::from_secs(3);
            while child.try_wait().ok().flatten().is_none() && Instant::now() < stop {
                thread::sleep(Duration::from_millis(10));
            }
            return Err("wrapper non arrêté dans le budget".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn wrapper_claude_natif_livre_une_mission_repond_et_tient_un_journal() {
    let root = test_root();
    let socket = root.join("state/bridget.sock");
    let registry_path = root.join("state/agents.json");
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
        let stream = accept_peer(listener);
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
                channel,
                mode,
                location,
                ..
            } => {
                assert_eq!(agent_type, "claude-native");
                assert_eq!(transport.as_deref(), Some("claude_stream_json"));
                assert!(
                    channel
                        .as_deref()
                        .is_some_and(|value| value != "claude_stream_json")
                );
                assert_eq!(mode, Some(PresenceMode::Cli));
                assert_eq!(location, None);
            }
            other => panic!("Register Claude natif attendu, reçu : {other:?}"),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                credential: None,
                agent_id: "89000000-0000-4000-8000-000000000801".to_string(),
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
        let mut mission = BridgetMessage::new(
            "89000000-0000-4000-8000-000000000805",
            "89000000-0000-4000-8000-000000000801",
            "mission native",
        );
        mission.id = "mission-claude-native".to_string();
        mission.reply = true;
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::DeliverExecution {
                message: mission,
                execution_id: "execution-claude-native".to_string(),
                generation: 4,
                revision: 0,
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();

        let mut answered = false;
        let mut started = false;
        let mut completed = false;
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            match decode(line.trim_end()).unwrap() {
                WrapperToDaemon::ExecutionStateChanged { transition } => {
                    assert_eq!(transition.execution_id, "execution-claude-native");
                    assert_eq!(transition.generation, 4);
                    if transition.next_state == "running" {
                        assert_eq!(transition.expected_state, "starting");
                        assert_eq!(transition.expected_revision, 0);
                        started = true;
                    } else {
                        assert_eq!(transition.next_state, "completed");
                        assert_eq!(transition.expected_state, "running");
                        assert_eq!(transition.expected_revision, 1);
                        completed = true;
                    }
                }
                WrapperToDaemon::Send(reply) => {
                    assert_eq!(reply.from, "89000000-0000-4000-8000-000000000801");
                    assert_eq!(reply.to, "89000000-0000-4000-8000-000000000805");
                    assert_eq!(reply.in_reply_to.as_deref(), Some("mission-claude-native"));
                    assert_eq!(reply.body, "claude-opus-5: mission reçue");
                    answered = true;
                    writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
                    writer.flush().unwrap();
                }
                WrapperToDaemon::Unregister => return answered && started && completed,
                _ => {}
            }
        }
    });

    let wrapper_root = root.clone();
    let wrapper = thread::spawn(move || {
        run_wrapper(
            &wrapper_root,
            "claude-native",
            "89000000-0000-4000-8000-000000000801",
        )
    });

    assert_eq!(wrapper.join().unwrap(), Ok(()));
    assert!(daemon.join().unwrap());
    let sessions = root.join("state/sessions/89000000-0000-4000-8000-000000000801");
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
    let socket = root.join("state/bridget.sock");
    let registry = root.join("state/agents.json");
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
        let stream = accept_peer(listener);
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
                credential: None,
                agent_id: "89000000-0000-4000-8000-000000000802".to_string(),
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
            "89000000-0000-4000-8000-000000000802".to_string(),
            serde_json::to_string(&frozen_definition).unwrap(),
        ],
        cwd: root.clone(),
        env: BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            (
                "BRIDGET_HOME".to_string(),
                root.join("state").into_os_string(),
            ),
            (
                "BRIDGET_SOCKET".to_string(),
                root.join("state/bridget.sock").into_os_string(),
            ),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("C")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
            ("BRIDGET_CHANNEL".to_string(), OsString::from("unix")),
        ]),
    };
    let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
    let ready = spawn_managed_bootstrap(&launch)
        .unwrap()
        .wait_ready()
        .unwrap();
    assert_eq!(ready.ready().instance_id, identity.instance_id);
    let mut running = ready
        .persist_marker(&marker_store, "89000000-0000-4000-8000-000000000802")
        .unwrap()
        .release()
        .unwrap();
    match register_rx.recv_timeout(Duration::from_secs(2)).unwrap() {
        WrapperToDaemon::Register {
            agent_type,
            agent_id,
            transport,
            channel,
            mode,
            instance_id,
            domain,
            ..
        } => {
            assert_eq!(agent_type, "claude");
            assert_eq!(agent_id, "89000000-0000-4000-8000-000000000802".to_string());
            assert_eq!(transport.as_deref(), Some("claude_stream_json"));
            assert!(
                channel
                    .as_deref()
                    .is_some_and(|value| value != "claude_stream_json")
            );
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
    let socket = root.join("state/bridget.sock");
    let registry = root.join("state/agents.json");
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
        let stream = accept_peer(listener);
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
                credential: None,
                agent_id: "89000000-0000-4000-8000-000000000803".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Deliver(BridgetMessage::new(
                "humain",
                "89000000-0000-4000-8000-000000000803",
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
            "89000000-0000-4000-8000-000000000803".to_string(),
            serde_json::to_string(&frozen_definition).unwrap(),
        ],
        cwd: root.clone(),
        env: BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            (
                "BRIDGET_HOME".to_string(),
                root.join("state").into_os_string(),
            ),
            (
                "BRIDGET_SOCKET".to_string(),
                root.join("state/bridget.sock").into_os_string(),
            ),
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
        .persist_marker(&marker_store, "89000000-0000-4000-8000-000000000803")
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

#[test]
fn wrapper_borne_le_silence_et_signale_la_saturation_sans_confondre_les_tours() {
    let root = test_root();
    let socket = root.join("state/bridget.sock");
    let registry_path = root.join("state/agents.json");
    let adapter = root.join("claude-silencieux.sh");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    fs::write(
        &adapter,
        r#"#!/bin/sh
while IFS= read -r line; do :; done
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let registry_json = serde_json::json!({
        "agents": {
            "claude-silencieux": {
                "command": adapter,
                "protocol": "claude_stream_json",
                "permissions": "allow",
                "queue_capacity": 1,
                "notify_timeout_secs": 5
            }
        }
    })
    .to_string();
    fs::write(&registry_path, &registry_json).unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let daemon = thread::spawn(move || {
        let stream = accept_peer(listener);
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
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
                credential: None,
                agent_id: "89000000-0000-4000-8000-000000000804".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if matches!(
                decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                WrapperToDaemon::JournalReady
            ) {
                break;
            }
        }

        let id = "silence";
        let mut message =
            BridgetMessage::new("guichet", "89000000-0000-4000-8000-000000000804", id);
        message.id = id.to_string();
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::DeliverExecution {
                message,
                execution_id: format!("execution-{id}"),
                generation: 1,
                revision: 0,
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();

        // Barrière : le premier tour doit être actif avant de remplir la file.
        // Sans elle, le worker peut retirer `silence` entre les trois writes
        // et transformer aléatoirement `saturation` en second tour réel.
        let mut silence_running = false;
        let start_deadline = std::time::Instant::now() + Duration::from_secs(8);
        while std::time::Instant::now() < start_deadline && !silence_running {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if let WrapperToDaemon::ExecutionStateChanged { transition } =
                decode(line.trim_end()).unwrap()
                && transition.execution_id == "execution-silence"
                && transition.next_state == "running"
            {
                assert_eq!(transition.expected_state, "starting");
                assert_eq!(transition.expected_revision, 0);
                silence_running = true;
            }
        }
        assert!(silence_running, "le démarrage du premier tour manque");

        for id in ["en-file", "saturation"] {
            let mut message =
                BridgetMessage::new("guichet", "89000000-0000-4000-8000-000000000804", id);
            message.id = id.to_string();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::DeliverExecution {
                    message,
                    execution_id: format!("execution-{id}"),
                    generation: 1,
                    revision: 0,
                })
                .unwrap()
            )
            .unwrap();
        }
        writer.flush().unwrap();

        let mut silence_failed = false;
        let mut saturation_failed = false;
        let deadline = std::time::Instant::now() + Duration::from_secs(12);
        while std::time::Instant::now() < deadline && !(silence_failed && saturation_failed) {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if let WrapperToDaemon::ExecutionStateChanged { transition } =
                decode(line.trim_end()).unwrap()
            {
                match (
                    transition.execution_id.as_str(),
                    transition.next_state.as_str(),
                ) {
                    ("execution-silence", "running") => {
                        assert_eq!(transition.expected_state, "starting");
                        assert_eq!(transition.expected_revision, 0);
                        silence_running = true;
                    }
                    ("execution-silence", "failed") => {
                        assert_eq!(transition.expected_state, "running");
                        assert_eq!(transition.expected_revision, 1);
                        silence_failed = true;
                    }
                    ("execution-saturation", "failed") => {
                        assert_eq!(transition.expected_state, "starting");
                        assert_eq!(transition.expected_revision, 0);
                        assert_eq!(transition.reason, "provider_queue_full");
                        saturation_failed = true;
                    }
                    _ => {}
                }
            }
        }
        assert!(silence_running, "le démarrage du premier tour manque");
        assert!(
            silence_failed,
            "le fournisseur silencieux doit atteindre une issue bornée"
        );
        assert!(
            saturation_failed,
            "la troisième remise doit être refusée par la file bornée"
        );
        writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
        writer.flush().unwrap();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => return false,
                Ok(_)
                    if matches!(
                        decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                        WrapperToDaemon::Unregister
                    ) =>
                {
                    return true;
                }
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return false;
                }
                Err(error) => panic!("lecture daemon impossible: {error}"),
            }
        }
    });

    let wrapper_root = root.clone();
    let wrapper = thread::spawn(move || {
        run_wrapper(
            &wrapper_root,
            "claude-silencieux",
            "89000000-0000-4000-8000-000000000804",
        )
    });

    assert_eq!(wrapper.join().unwrap(), Ok(()));
    assert!(
        daemon.join().unwrap(),
        "le wrapper ne se ferme pas proprement"
    );
    fs::remove_dir_all(root).unwrap();
}
