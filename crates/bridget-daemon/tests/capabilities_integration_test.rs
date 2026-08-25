//! Couture de la garde déclarative : aucun processus ni ordre durable ne doit
//! naître lorsqu'une capacité de lancement n'est pas déclarée.

use bridget_transport::protocol::{decode, encode};
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
        self.send(&WrapperToDaemon::Register {
            agent_type: "capability-test".to_string(),
            name: None,
            host: None,
            transport: None,
            channel: None,
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
    // Le socket daemon ajoute `.cache/bridget/bridget.sock` : un préfixe court
    // évite de transformer cette couture en faux rouge SUN_LEN.
    let root = PathBuf::from(format!("/tmp/bcap-{}-{nonce:x}", std::process::id()));
    std::fs::create_dir_all(root.join(".config/bridget")).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn socket_path(root: &Path) -> PathBuf {
    root.join(".cache/bridget/bridget.sock")
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
    let path = root.join(".config/bridget/agents.json");
    std::fs::write(&path, serde_json::to_vec(&registry).unwrap()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn write_missing_command_registry(root: &Path) {
    let registry = serde_json::json!({
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
    let path = root.join(".config/bridget/agents.json");
    std::fs::write(&path, serde_json::to_vec(&registry).unwrap()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn spawn_order(command_id: &str) -> WrapperToDaemon {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    WrapperToDaemon::SpawnOrder {
        agent_type: "fixture".to_string(),
        name: Some("capability-fixture".to_string()),
        cwd: "/tmp".to_string(),
        persistent: false,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 20,
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
