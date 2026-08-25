//! Couture réelle wrapper/daemon/attach du pilote Codex natif.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn root() -> PathBuf {
    // `sun_path` est limité : le HOME isolé doit lui-même être court.
    let root = PathBuf::from("/tmp").join(format!(
        "bcn-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge")
            .as_nanos()
    ));
    fs::create_dir_all(root.join(".config/bridget")).expect("configuration temporaire");
    root
}

fn write_frame(writer: &mut BufWriter<UnixStream>, frame: &WrapperToDaemon) {
    writeln!(writer, "{}", encode(frame).expect("encodage")).expect("écriture");
    writer.flush().expect("flush");
}

fn read_frame(reader: &mut BufReader<UnixStream>) -> DaemonToWrapper {
    let mut line = String::new();
    reader.read_line(&mut line).expect("lecture daemon");
    assert!(!line.is_empty(), "EOF daemon inattendu");
    decode(line.trim_end()).expect("trame daemon")
}

fn start_daemon(root: &Path) -> ChildGuard {
    let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("daemon")
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("daemon réel");
    let daemon = ChildGuard(child);
    let socket = root.join(".cache/bridget/bridget.sock");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if UnixStream::connect(&socket).is_ok() {
            return daemon;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("daemon absent: {}", socket.display());
}

fn start_native_wrapper(root: &Path, extra_environment: &[(String, String)]) -> ChildGuard {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    let child = command
        .args(["codex", "--equipier", "--name", "codex-native"])
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin")
        .envs(extra_environment.iter().cloned())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("wrapper Codex natif");
    ChildGuard(child)
}

fn write_native_registry(root: &Path, script: &str) {
    let registry = serde_json::json!({
        "agents": {
            "codex": {
                "command": "sh",
                "args": ["-c", script],
                "protocol": "codex_app_server",
                "permissions": "allow",
                "queue_capacity": 4,
                "notify_timeout_secs": 3,
                "forbidden_env": [],
                "pass_env": []
            }
        }
    });
    let registry_path = root.join(".config/bridget/agents.json");
    fs::write(
        &registry_path,
        serde_json::to_vec(&registry).expect("registre JSON"),
    )
    .expect("registre privé");
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600))
        .expect("permissions registre");
}

fn sender(socket: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let stream = UnixStream::connect(socket).expect("connexion expéditeur");
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .expect("timeout expéditeur");
    let mut reader = BufReader::new(stream.try_clone().expect("clone lecteur"));
    let mut writer = BufWriter::new(stream);
    write_frame(
        &mut writer,
        &WrapperToDaemon::Register {
            agent_type: "cli".to_string(),
            name: Some("sender-native".to_string()),
            host: None,
            transport: Some("unix".to_string()),
            channel: None,
            mode: Some(bridget_transport::protocol::PresenceMode::Cli),
            location: None,
            os: None,
            instance_id: None,
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        },
    );
    assert!(matches!(
        read_frame(&mut reader),
        DaemonToWrapper::Registered { .. }
    ));
    (reader, writer)
}

fn attach(socket: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let stream = UnixStream::connect(socket).expect("connexion attach");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("timeout attach");
    let mut reader = BufReader::new(stream.try_clone().expect("clone attach"));
    let mut writer = BufWriter::new(stream);
    write_frame(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
    );
    assert!(matches!(
        read_frame(&mut reader),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        }
    ));
    (reader, writer)
}

#[test]
fn wrapper_codex_natif_repond_et_reste_attachable() {
    let root = root();
    let script = r#"started=0; while IFS= read -r line; do
        case "$line" in
            *'"method":"initialize"'*) printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"macos"}}' ;;
            *'"method":"initialized"'*) started=1 ;;
            *'"method":"thread/start"'*) printf '%s\n' '{"id":2,"result":{"thread":{"id":"thread-native"},"model":"gpt-5.6-terra","reasoningEffort":"high"}}' ;;
            *'"method":"account/rateLimits/read"'*) printf '%s\n' '{"id":3,"result":{"rateLimits":{"primary":{"usedPercent":42,"windowDurationMins":300,"resetsAt":1787572200},"rateLimitReachedType":null}}}' ;;
            *'"method":"turn/start"'*)
                [ "$started" = 1 ] || exit 72
                printf '%s\n' '{"id":4,"result":{"turn":{"id":"turn-native"}}}'
                printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"item","delta":"réponse gpt-5.6-terra"}}'
                printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}' ;;
        esac
    done"#;
    write_native_registry(&root, script);

    let daemon = start_daemon(&root);
    let wrapper = start_native_wrapper(&root, &[]);
    let socket = root.join(".cache/bridget/bridget.sock");
    let (mut sender_reader, mut sender_writer) = sender(&socket);
    let (mut attach_reader, mut attach_writer) = attach(&socket);
    let deadline = Instant::now() + Duration::from_secs(5);
    let subscription_id = loop {
        write_frame(
            &mut attach_writer,
            &WrapperToDaemon::Subscribe {
                agent: "codex-native".to_string(),
                window: bridget_transport::AttachWindow::Seq(0),
            },
        );
        match read_frame(&mut attach_reader) {
            DaemonToWrapper::Subscribed { subscription_id } => break subscription_id,
            DaemonToWrapper::AttachRejected { .. } if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(20));
            }
            other => panic!("attach natif refusé: {other:?}"),
        }
    };
    write_frame(&mut sender_writer, &WrapperToDaemon::ListAgents);
    let agents = match read_frame(&mut sender_reader) {
        DaemonToWrapper::AgentList { agents } => agents,
        other => panic!("annuaire Codex natif inattendu: {other:?}"),
    };
    let codex = agents
        .iter()
        .find(|agent| agent.name == "codex-native")
        .expect("agent Codex natif absent de l'annuaire");
    assert_eq!(codex.model.as_deref(), Some("gpt-5.6-terra"));
    assert_eq!(codex.effort.as_deref(), Some("high"));
    assert!(matches!(
        codex.rate_limits.as_slice(),
        [limit]
            if limit.window == "primary/300m"
                && limit.status == "available"
                && limit.resets_at == Some(1_787_572_200)
                && limit.used_percent == Some(42)
    ));
    let who = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("who")
        .env_clear()
        .env("HOME", &root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("exécution who réelle");
    assert!(
        who.status.success(),
        "who réel échoue: {}",
        String::from_utf8_lossy(&who.stderr)
    );
    let who = String::from_utf8(who.stdout).expect("who UTF-8");
    assert!(who.contains("EFFORT") && who.contains("LIMITE"));
    assert!(who.contains("high"));
    assert!(who.contains("5h 42% rst "), "format compact LIMITE: {who}");
    let mut request = BridgetMessage::new("sender-native", "codex-native", "mission réelle");
    request.reply = true;
    write_frame(&mut sender_writer, &WrapperToDaemon::Send(request.clone()));
    assert!(matches!(
        read_frame(&mut sender_reader),
        DaemonToWrapper::Ack { .. }
    ));
    let reply = loop {
        match read_frame(&mut sender_reader) {
            DaemonToWrapper::Deliver(message) => break message,
            DaemonToWrapper::Ack { .. } => {}
            other => panic!("retour natif inattendu: {other:?}"),
        }
    };
    assert_eq!(
        reply.in_reply_to.as_deref(),
        Some(request.id.as_str()),
        "réponse Codex réelle non corrélée: {reply:?}"
    );
    assert_eq!(reply.body, "réponse gpt-5.6-terra");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut saw_journal = false;
    while Instant::now() < deadline {
        match read_frame(&mut attach_reader) {
            DaemonToWrapper::JournalFragment {
                subscription_id: received,
                ..
            } if received == subscription_id => {
                saw_journal = true;
                break;
            }
            DaemonToWrapper::SnapshotCaughtUp { .. } => {}
            other => panic!("flux attach natif inattendu: {other:?}"),
        }
    }
    assert!(saw_journal, "attach ne reçoit aucun journal natif");
    drop(wrapper);
    drop(daemon);
    fs::remove_dir_all(root).expect("nettoyage HOME isolé");
}

#[test]
fn wrapper_codex_sans_signal_laisse_effort_et_limite_inconnus() {
    let root = root();
    let script = r#"while IFS= read -r line; do
        case "$line" in
            *'"method":"initialize"'*) printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"macos"}}' ;;
            *'"method":"thread/start"'*) printf '%s\n' '{"id":2,"result":{"thread":{"id":"thread-native"}}}' ;;
            *'"method":"account/rateLimits/read"'*) printf '%s\n' '{"id":3,"result":{}}' ;;
        esac
    done"#;
    write_native_registry(&root, script);

    let daemon = start_daemon(&root);
    let wrapper = start_native_wrapper(&root, &[]);
    let socket = root.join(".cache/bridget/bridget.sock");
    let (mut sender_reader, mut sender_writer) = sender(&socket);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        write_frame(&mut sender_writer, &WrapperToDaemon::ListAgents);
        match read_frame(&mut sender_reader) {
            DaemonToWrapper::AgentList { agents }
                if agents.iter().any(|agent| agent.name == "codex-native") =>
            {
                let codex = agents
                    .iter()
                    .find(|agent| agent.name == "codex-native")
                    .expect("agent Codex natif absent");
                assert!(codex.effort.is_none(), "effort inventé: {codex:?}");
                assert!(codex.rate_limits.is_empty(), "limite inventée: {codex:?}");
                break;
            }
            DaemonToWrapper::AgentList { .. } if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(20));
            }
            other => panic!("annuaire sans signal inattendu: {other:?}"),
        }
    }
    drop(wrapper);
    drop(daemon);
    fs::remove_dir_all(root).expect("nettoyage HOME isolé");
}

/// Gate hors CI : le binaire Codex local et le modèle demandé sont l'oracle,
/// pas le faux serveur JSONL de la couture déterministe ci-dessus.
#[test]
#[ignore = "requiert un compte Codex local et BRIDGET_CODEX_NATIVE_GATE=1"]
fn gate_reel_codex_app_server_gpt_5_6_terra_et_attach() {
    assert_eq!(
        std::env::var("BRIDGET_CODEX_NATIVE_GATE").as_deref(),
        Ok("1"),
        "la gate réelle exige BRIDGET_CODEX_NATIVE_GATE=1"
    );
    let root = root();
    let codex = std::env::var("BRIDGET_CODEX_APP_SERVER_BIN")
        .unwrap_or_else(|_| "/opt/homebrew/bin/codex".to_string());
    assert!(Path::new(&codex).is_file(), "binaire Codex absent: {codex}");
    let codex_home = std::env::var("CODEX_HOME").unwrap_or_else(|_| "/Users/moi/.codex".into());
    let registry = serde_json::json!({
        "agents": {
            "codex": {
                "command": codex,
                "args": ["-c", "model=\"gpt-5.6-terra\"", "app-server"],
                "protocol": "codex_app_server",
                "permissions": "allow",
                "queue_capacity": 4,
                "notify_timeout_secs": 30,
                "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
                "pass_env": ["CODEX_HOME"]
            }
        }
    });
    let registry_path = root.join(".config/bridget/agents.json");
    fs::write(
        &registry_path,
        serde_json::to_vec(&registry).expect("registre JSON"),
    )
    .expect("registre privé");
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600))
        .expect("permissions registre");

    let daemon = start_daemon(&root);
    let wrapper = start_native_wrapper(&root, &[("CODEX_HOME".to_string(), codex_home)]);
    let socket = root.join(".cache/bridget/bridget.sock");
    let (mut sender_reader, mut sender_writer) = sender(&socket);
    let (mut attach_reader, mut attach_writer) = attach(&socket);
    let deadline = Instant::now() + Duration::from_secs(30);
    let subscription_id = loop {
        write_frame(
            &mut attach_writer,
            &WrapperToDaemon::Subscribe {
                agent: "codex-native".to_string(),
                window: bridget_transport::AttachWindow::Seq(0),
            },
        );
        match read_frame(&mut attach_reader) {
            DaemonToWrapper::Subscribed { subscription_id } => break subscription_id,
            DaemonToWrapper::AttachRejected { .. } if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(50));
            }
            other => panic!("attach Codex réel refusé: {other:?}"),
        }
    };
    write_frame(&mut sender_writer, &WrapperToDaemon::ListAgents);
    let agents = match read_frame(&mut sender_reader) {
        DaemonToWrapper::AgentList { agents } => agents,
        other => panic!("annuaire Codex réel inattendu: {other:?}"),
    };
    let codex = agents
        .iter()
        .find(|agent| agent.name == "codex-native")
        .expect("agent Codex réel absent de l'annuaire");
    assert_eq!(codex.model.as_deref(), Some("gpt-5.6-terra"));
    assert!(
        codex.effort.is_some(),
        "effort Codex non attesté: {codex:?}"
    );
    assert!(
        !codex.rate_limits.is_empty(),
        "limite Codex non attestée: {codex:?}"
    );
    let who = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("who")
        .env_clear()
        .env("HOME", &root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("exécution who Codex réelle");
    assert!(
        who.status.success(),
        "who Codex réel échoue: {}",
        String::from_utf8_lossy(&who.stderr)
    );
    let who = String::from_utf8(who.stdout).expect("who Codex réel UTF-8");
    assert!(who.contains(codex.effort.as_deref().expect("effort attesté")));
    // who affiche l'abréviation (5h/7d…), pas le nom brut primary/…
    assert!(
        who.contains("rst ") || who.contains('%'),
        "LIMITE compacte absente de who={who} faits={:?}",
        codex.rate_limits
    );
    let mut request = BridgetMessage::new(
        "sender-native",
        "codex-native",
        "Réponds avec une phrase courte confirmant la réception de cette mission Bridget.",
    );
    request.reply = true;
    write_frame(&mut sender_writer, &WrapperToDaemon::Send(request.clone()));
    assert!(matches!(
        read_frame(&mut sender_reader),
        DaemonToWrapper::Ack { .. }
    ));
    let reply = loop {
        match read_frame(&mut sender_reader) {
            DaemonToWrapper::Deliver(message) => break message,
            DaemonToWrapper::Ack { .. } => {}
            other => panic!("retour Codex réel inattendu: {other:?}"),
        }
    };
    assert_eq!(
        reply.in_reply_to.as_deref(),
        Some(request.id.as_str()),
        "réponse Codex réelle non corrélée: {reply:?}"
    );
    assert!(!reply.body.trim().is_empty(), "réponse réelle Codex vide");

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut saw_journal = false;
    while Instant::now() < deadline {
        match read_frame(&mut attach_reader) {
            DaemonToWrapper::JournalFragment {
                subscription_id: received,
                ..
            } if received == subscription_id => {
                saw_journal = true;
                break;
            }
            DaemonToWrapper::SnapshotCaughtUp { .. } => {}
            other => panic!("flux attach Codex réel inattendu: {other:?}"),
        }
    }
    assert!(saw_journal, "attach ne reçoit aucun journal Codex réel");
    drop(wrapper);
    drop(daemon);
    fs::remove_dir_all(root).expect("nettoyage HOME isolé");
}
