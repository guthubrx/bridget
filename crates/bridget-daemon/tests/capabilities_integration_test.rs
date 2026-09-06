//! Couture de la garde déclarative : aucun processus ni ordre durable ne doit
//! naître lorsqu'une capacité de lancement n'est pas déclarée.

use bridget_core::{BridgetMessage, MessageIntent};
use bridget_transport::protocol::{
    ExecutionProviderContext, ProviderObservation, ProviderOperation, decode, encode,
};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct DaemonProcess(Child);

impl DaemonProcess {
    fn start(home: &Path) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("daemon")
            .env_clear()
            .env("HOME", home)
            .env("BRIDGET_HOME", home.join("state"))
            .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("daemon réel démarré");
        let daemon = Self(child);
        let socket = socket_path(home);
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

struct Peer {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
}

impl Peer {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        Self {
            writer: BufWriter::new(stream),
            reader,
        }
    }

    fn send(&mut self, message: &WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn receive(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }

    fn register(&mut self) {
        self.register_with_name(None);
    }

    fn register_as(&mut self, name: &str) {
        self.register_with_name(Some(name.to_string()));
    }

    fn register_with_name(&mut self, agent_id: Option<String>) {
        self.send(&WrapperToDaemon::Register {
            agent_type: "capability-test".to_string(),
            identity_version: 2,
            agent_id: agent_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            host: None,
            transport: None,
            channel: None.into(),
            mode: None,
            location: None,
            os: None,
            instance_id: None,
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        });
        assert!(matches!(self.receive(), DaemonToWrapper::Registered { .. }));
    }
}

fn root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Le socket daemon ajoute `state/bridget.sock` : un préfixe court
    // évite de transformer cette couture en faux rouge SUN_LEN.
    let root = PathBuf::from(format!("/tmp/bcap-{}-{nonce:x}", std::process::id()));
    bridget_daemon::environment::Namespace::resolve(
        Some(root.join("state")),
        None,
        Some(root.clone()),
    )
    .unwrap()
    .prepare()
    .unwrap();
    // Précondition durable, avant tout daemon : le script de cette couture
    // vérifie le préflight de capacités, pas la posture découverte.
    let database = root.join("state/bridget.db");
    drop(bridget_daemon::store::Store::open(&database).unwrap());
    let connection = rusqlite::Connection::open(&database).unwrap();
    let initial = bridget_daemon::referent_control::read(&connection).unwrap();
    bridget_daemon::referent_control::set(
        &connection,
        bridget_daemon::referent_control::ControlMutation {
            command_id: "capability-fixture-complete",
            expected_generation: initial.generation,
            paused: None,
            auto_objectives_cap: None,
            reason: None,
            actor: "test",
            now: 1,
            agent_posture: Some(bridget_transport::protocol::AgentPosture::Complete),
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    root
}

fn socket_path(root: &Path) -> PathBuf {
    root.join("state/bridget.sock")
}

fn write_registry(root: &Path, marker: &Path, supports_requested_model: bool) {
    let adapter = root.join("adapter-should-not-run.sh");
    std::fs::write(
        &adapter,
        format!("#!/bin/sh\nprintf launched > {}\n", marker.display()),
    )
    .unwrap();
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
    let models = if supports_requested_model {
        serde_json::json!({"gpt-5.6-terra": {"efforts": ["high"]}})
    } else {
        serde_json::json!({"gpt-5.5": {"efforts": ["high"]}})
    };
    let registry = serde_json::json!({
        "execution_projection": {"dual_write": true, "legacy_projection": true},
        "agents": {
            "fixture": {
                "command": adapter,
                "args": ["--model", "gpt-5.6-terra", "--effort", "high"],
                "protocol": "acp",
                "forbidden_env": [],
                "pass_env": [],
                "capabilities": {"execution_paths": ["acp"], "models": models}
            }
        }
    });
    let path = root.join("state/agents.json");
    std::fs::write(&path, serde_json::to_vec(&registry).unwrap()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn write_missing_command_registry(root: &Path) {
    let registry = serde_json::json!({
        "execution_projection": {"dual_write": true, "legacy_projection": true},
        "agents": {
            "fixture": {
                "command": "/definitely/missing/bridget-native-pilot",
                "args": ["--model", "gpt-5.6-terra", "--effort", "high"],
                "protocol": "acp",
                "forbidden_env": [],
                "pass_env": [],
                "capabilities": {
                    "execution_paths": ["acp"],
                    "models": {"gpt-5.6-terra": {"efforts": ["high"]}}
                }
            }
        }
    });
    let path = root.join("state/agents.json");
    std::fs::write(&path, serde_json::to_vec(&registry).unwrap()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn spawn_order(command_id: &str) -> WrapperToDaemon {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "fixture".to_string(),
        project: None,
        agent_id: Some("89000000-0000-4000-8000-000000000201".to_string()),
        cwd: "/tmp".to_string(),
        persistent: false,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 20,
        ownership: None,
    }
}

#[test]
fn modele_non_declare_est_refuse_avant_processus_et_ordre_durable() {
    let root = root();
    let marker = root.join("adapter-ran");
    write_registry(&root, &marker, false);
    let daemon = DaemonProcess::start(&root);
    let command_id = "capability-model-refused";
    let mut peer = Peer::connect(&socket_path(&root));
    peer.register();
    peer.send(&spawn_order(command_id));
    assert!(matches!(
        peer.receive(),
        DaemonToWrapper::SpawnRejected {
            reason: SpawnRefusal::UnsupportedCapability { agent_type, model, capability },
            ..
        } if agent_type == "fixture"
            && model == "gpt-5.6-terra"
            && capability == "modèle pris en charge par l'adaptateur"
    ));
    thread::sleep(Duration::from_millis(150));
    assert!(
        !marker.exists(),
        "le script adaptateur ne doit jamais être exécuté lors du refus"
    );
    drop(peer);
    drop(daemon);

    // Même command_id après un redémarrage et une matrice corrigée : si le
    // refus avait créé un ordre durable, le daemon rejouerait ce refus au lieu
    // de lancer ce script. Cette seconde phase prouve l'absence de résidu.
    write_registry(&root, &marker, true);
    let daemon = DaemonProcess::start(&root);
    let mut peer = Peer::connect(&socket_path(&root));
    peer.register();
    peer.send(&spawn_order(command_id));
    let deadline = Instant::now() + Duration::from_secs(3);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        marker.exists(),
        "la matrice corrigée doit permettre le lancement"
    );
    drop(peer);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn commande_native_absente_est_refusee_sans_residu_puis_reparable_au_meme_id() {
    let root = root();
    let marker = root.join("adapter-ran");
    write_missing_command_registry(&root);
    let daemon = DaemonProcess::start(&root);
    let command_id = "native-command-missing";
    let mut peer = Peer::connect(&socket_path(&root));
    peer.register();
    peer.send(&spawn_order(command_id));
    assert!(matches!(
        peer.receive(),
        DaemonToWrapper::SpawnRejected {
            reason: SpawnRefusal::CommandMissing { command, .. },
            ..
        } if command == "/definitely/missing/bridget-native-pilot"
    ));
    assert!(
        !marker.exists(),
        "CommandMissing doit refuser avant tout processus enfant"
    );
    drop(peer);
    drop(daemon);

    // Mutation discriminante : persister ce refus empêcherait cette seconde
    // phase de lancer le même command_id après correction du registre.
    write_registry(&root, &marker, true);
    let daemon = DaemonProcess::start(&root);
    let mut peer = Peer::connect(&socket_path(&root));
    peer.register();
    peer.send(&spawn_order(command_id));
    let deadline = Instant::now() + Duration::from_secs(3);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        marker.exists(),
        "le même command_id doit rester lançable après correction du registre"
    );
    drop(peer);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn contexte_fournisseur_refuse_le_non_proprietaire_la_generation_et_le_binding_incoherents() {
    let root = root();
    let marker = root.join("adapter-ran");
    write_registry(&root, &marker, true);
    let daemon = DaemonProcess::start(&root);
    let socket = socket_path(&root);
    let mut owner = Peer::connect(&socket);
    owner.register_as("89000000-0000-4000-8000-000000000202");
    let mut intrus = Peer::connect(&socket);
    intrus.register_as("89000000-0000-4000-8000-000000000203");
    let mut source = Peer::connect(&socket);
    source.register_as("89000000-0000-4000-8000-000000000204");

    let mut trigger = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000204",
        "89000000-0000-4000-8000-000000000202",
        "démarrer",
    );
    trigger.id = "provider-context-trigger".to_string();
    trigger.intent = Some(MessageIntent::TriggerTurn);
    source.send(&WrapperToDaemon::Send(trigger));
    let submission_ack = source.receive();
    assert!(
        matches!(submission_ack, DaemonToWrapper::Ack { .. }),
        "accusé de déclenchement inattendu: {submission_ack:?}"
    );
    let execution_id = loop {
        match owner.receive() {
            DaemonToWrapper::Deliver(message) if message.id == "provider-context-trigger" => {}
            DaemonToWrapper::DeliverExecution {
                execution_id,
                generation,
                revision,
                ..
            } => {
                assert_eq!(generation, 1);
                assert_eq!(revision, 0);
                break execution_id;
            }
            other => panic!("livraison d'exécution attendue, reçu {other:?}"),
        }
    };
    let observed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let context = ExecutionProviderContext {
        execution_id: execution_id.clone(),
        generation: 1,
        provider_kind: "cursor".to_string(),
        execution_path: "acp".to_string(),
        observation: ProviderObservation {
            binary_path: "/fixture/cursor-agent".to_string(),
            binary_version: "fixture-1".to_string(),
            binary_digest: "a".repeat(64),
            contract_version: "acp-v1".to_string(),
            operations: vec![ProviderOperation::Interrupt],
        },
        provider_session_id: Some("session-owner".to_string()),
        provider_thread_id: Some("thread-owner".to_string()),
        provider_turn_id: Some("turn-owner".to_string()),
        observed_at,
    };
    owner.send(&WrapperToDaemon::ExecutionProviderObserved {
        context: context.clone(),
    });

    intrus.send(&WrapperToDaemon::ExecutionProviderObserved {
        context: context.clone(),
    });
    assert!(matches!(
        intrus.receive(),
        DaemonToWrapper::Nack { reason, .. }
            if reason == "contexte fournisseur émis par un wrapper non propriétaire"
    ));

    let mut stale = context.clone();
    stale.generation = 2;
    owner.send(&WrapperToDaemon::ExecutionProviderObserved { context: stale });
    assert!(matches!(
        owner.receive(),
        DaemonToWrapper::Nack { reason, .. }
            if reason == "contexte fournisseur de génération périmée"
    ));

    let mut incompatible = context;
    incompatible.provider_thread_id = Some("thread-autre".to_string());
    owner.send(&WrapperToDaemon::ExecutionProviderObserved {
        context: incompatible,
    });
    assert!(matches!(
        owner.receive(),
        DaemonToWrapper::Nack { reason, .. }
            if reason == "contexte fournisseur incompatible"
    ));
    drop(source);

    drop(intrus);
    drop(owner);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}
