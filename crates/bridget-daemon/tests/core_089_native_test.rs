//! Couture réelle wrapper/daemon/attach des pilotes natifs.
//! Les gates fournisseur sont explicites ; les deux fixtures ne les remplacent pas.

#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        fixture::cleanup_child(&mut self.0);
    }
}

fn stop_session(mut daemon: ChildGuard, mut wrapper: ChildGuard) {
    // Observer les enfants DIRECTS de notre wrapper vivant, pas une recherche
    // par nom de fournisseur qui pourrait toucher les sessions de l'utilisateur.
    let observed = std::process::Command::new("/bin/ps")
        .args(["-axo", "pid=,ppid="])
        .output()
        .unwrap();
    let children: Vec<i32> = String::from_utf8(observed.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let pair: Vec<_> = line.split_whitespace().collect();
            (pair.len() == 2 && pair[1].parse::<u32>().ok() == Some(wrapper.0.id()))
                .then(|| pair[0].parse().unwrap())
        })
        .collect();
    assert!(
        !children.is_empty(),
        "aucun processus fournisseur réel observé"
    );
    fixture::signal_test_group(&mut daemon.0, libc::SIGTERM);
    fixture::wait_child(&mut daemon.0, Duration::from_secs(10));
    fixture::wait_child(&mut wrapper.0, Duration::from_secs(10));
    let deadline = Instant::now() + Duration::from_secs(3);
    for child in children {
        while unsafe { libc::kill(child, 0) } == 0 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5)); // récolte, pas synchronisation métier
        }
        assert_eq!(
            unsafe { libc::kill(child, 0) },
            -1,
            "fournisseur orphelin après arrêt du wrapper"
        );
        assert_eq!(
            unsafe { libc::kill(-child, 0) },
            -1,
            "groupe fournisseur orphelin"
        );
    }
}

fn root() -> PathBuf {
    fixture::test_root("089-codex")
}

fn private_log(root: &Path, name: &str) -> fs::File {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(name))
        .unwrap()
}

struct PrivateCredentials(PathBuf);
impl Drop for PrivateCredentials {
    fn drop(&mut self) {
        // Copie créée par CE test seulement, même après panique. Jamais la source.
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requiert l'autorisation d'utiliser le compte Claude local : BRIDGET_CLAUDE_NATIVE_GATE=1"]
fn gate_reel_claude_stream_json_reponse_liee_et_attach() {
    assert_eq!(
        std::env::var("BRIDGET_CLAUDE_NATIVE_GATE").as_deref(),
        Ok("1"),
        "gate réel Claude : opt-in explicite requis"
    );
    let executable = std::env::var("BRIDGET_TEST_CLAUDE_BIN").expect("CLI Claude local requis");
    // Session 097 : le CLI officiel retrouve sa session d'abonnement seulement
    // avec le HOME réel ET `USER` (lecture du trousseau par compte). Aucune
    // clé d'API, aucune lecture du trousseau par le test, aucun jeton dans
    // l'environnement : l'état Bridget reste privé, l'authentification est
    // celle de l'humain. La configuration utilisateur est neutralisée par les
    // arguments (`--setting-sources ""`, `--strict-mcp-config`, `--tools ""`).
    let provider_home =
        std::env::var("BRIDGET_TEST_CLAUDE_HOME").expect("HOME réel explicite du compte Claude");
    let provider_user =
        std::env::var("BRIDGET_TEST_CLAUDE_USER").expect("USER explicite du compte Claude");
    let model = std::env::var("BRIDGET_TEST_CLAUDE_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());
    let root = root();
    fixture::private_write(&root.join("state/agents.json"), serde_json::to_vec(&serde_json::json!({
        "agents": {"claude": {
            "command": executable,
            "args": ["--model", &model, "--tools", "", "--strict-mcp-config", "--setting-sources", ""],
            "protocol": "claude_stream_json", "permissions": "deny", "notify_timeout_secs": 60,
            "forbidden_env": ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "CLAUDE_CODE_OAUTH_TOKEN"],
            "pass_env": [],
            "mcp": {"interactive":"none", "acp_session":false},
            "capabilities": {"execution_paths":["claude_stream_json"], "models":{&model:{"efforts":[]}}}
        }}
    })).unwrap()).unwrap();
    let daemon = start_daemon(&root);
    let wrapper = start_wrapper(
        &root,
        "claude",
        &[
            ("HOME".into(), provider_home),
            ("USER".into(), provider_user),
        ],
    );
    fixture::wait_for_registered_agent(&fixture::socket(&root), fixture::ACP_AGENT);
    let (mut reader, mut writer) = sender(&fixture::socket(&root));
    let (mut journal, mut journal_writer) = attach(&fixture::socket(&root));
    let deadline = Instant::now() + Duration::from_secs(15);
    let subscription = loop {
        write_frame(
            &mut journal_writer,
            &WrapperToDaemon::Subscribe {
                agent: fixture::ACP_AGENT.into(),
                window: bridget_transport::AttachWindow::Seq(0),
            },
        );
        match read_frame(&mut journal) {
            DaemonToWrapper::Subscribed { subscription_id } => break subscription_id,
            DaemonToWrapper::AttachRejected {
                reason: bridget_transport::protocol::AttachRefusal::JournalUnavailable,
                ..
            } => {
                assert!(Instant::now() < deadline, "journal Claude non attesté");
            }
            other => panic!("attache Claude refusée : {other:?}"),
        }
    };
    let mut message = BridgetMessage::new(
        fixture::ACTOR,
        fixture::ACP_AGENT,
        "Test de communication isolé. Réponds seulement : BRIDGET-089-CLAUDE-OK. Aucun outil, aucune autre action.",
    );
    message.reply = true;
    write_frame(&mut writer, &WrapperToDaemon::Send(message.clone()));
    let ack = read_frame(&mut reader);
    assert!(
        matches!(ack, DaemonToWrapper::Ack { .. }),
        "envoi refusé : {ack:?}"
    );
    let answer = read_frame(&mut reader);
    assert!(
        matches!(answer, DaemonToWrapper::Deliver(ref reply)
        if reply.in_reply_to.as_deref() == Some(message.id.as_str()) && reply.body.contains("BRIDGET-089-CLAUDE-OK")),
        "pas de réponse fournisseur corrélée : {answer:?}"
    );
    loop {
        match read_frame(&mut journal) {
            DaemonToWrapper::JournalFragment {
                subscription_id,
                bytes,
                ..
            } => {
                assert_eq!(subscription_id, subscription);
                assert!(!bytes.is_empty());
                break;
            }
            DaemonToWrapper::SnapshotCaughtUp { .. } => {}
            other => panic!("journal Claude : {other:?}"),
        }
    }
    // Politique 091 : le modèle déclaré vient d'un ordre de lancement ; un
    // équipier lancé à la main n'en a pas et la colonne reste « — ». Le pilote
    // compare le modèle servi (`system/init`) au modèle épinglé par `--model`
    // et publie l'écart éventuel : son absence est le verdict observable.
    write_frame(&mut writer, &WrapperToDaemon::ListAgents);
    let info = read_frame(&mut reader);
    assert!(
        matches!(info, DaemonToWrapper::AgentList { ref agents }
        if agents.iter().any(|agent| agent.agent_id == fixture::ACP_AGENT
            && agent.transport == "claude_stream_json"
            && agent.model_mismatch.is_none())),
        "présence Claude gérée incohérente : {info:?}"
    );
    let who = fixture::run_isolated(&root, &["who"], false);
    assert!(who.status.success());
    let who_text = String::from_utf8_lossy(&who.stdout);
    assert!(
        who_text.contains("claude_stream_json") && !who_text.contains('≠'),
        "{who_text}"
    );
    stop_session(daemon, wrapper);
    fs::remove_dir_all(root).unwrap();
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
    let child = fixture::isolated_command(root)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(private_log(root, "daemon.log")))
        .spawn()
        .expect("daemon réel");
    fixture::track(&child);
    let daemon = ChildGuard(child);
    let socket = fixture::socket(root);
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
    start_wrapper(root, "codex", extra_environment)
}

fn start_wrapper(root: &Path, kind: &str, extra_environment: &[(String, String)]) -> ChildGuard {
    let mut command = fixture::isolated_command(root);
    let child = command
        .args([
            kind,
            "--equipier",
            "--agent-id",
            "5da585af-5bc7-4808-985f-c73670633990",
        ])
        .current_dir(root.join("provider"))
        .envs(extra_environment.iter().cloned())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(private_log(root, "wrapper.log")))
        .spawn()
        .expect("wrapper Codex natif");
    fixture::track(&child);
    ChildGuard(child)
}

fn write_native_registry(root: &Path, script: &str) {
    let registry = serde_json::json!({
        "agents": {
            "codex": {
                "command": "/bin/sh",
                "args": ["-c", script],
                "protocol": "codex_app_server",
                "permissions": "deny",
                "queue_capacity": 4,
                "notify_timeout_secs": 3,
                "forbidden_env": [],
                "pass_env": []
            }
        }
    });
    let registry_path = root.join("state/agents.json");
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
            agent_type: "fixture".to_string(),
            identity_version: 2,
            agent_id: "da78fd70-41e8-424c-a88d-e29e2c5babcd".to_string(),
            host: None,
            transport: Some("unix".to_string()),
            channel: None.into(),
            mode: Some(bridget_transport::protocol::PresenceMode::Acp),
            location: None,
            os: None,
            instance_id: Some("089-native-sender".to_string()),
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
    let socket = fixture::socket(&root);
    let (mut sender_reader, mut sender_writer) = sender(&socket);
    let (mut attach_reader, mut attach_writer) = attach(&socket);
    let deadline = Instant::now() + Duration::from_secs(5);
    let subscription_id = loop {
        write_frame(
            &mut attach_writer,
            &WrapperToDaemon::Subscribe {
                agent: "5da585af-5bc7-4808-985f-c73670633990".to_string(),
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
        .find(|agent| agent.agent_id == "5da585af-5bc7-4808-985f-c73670633990")
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
    let who = fixture::isolated_command(&root)
        .arg("who")
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
    let mut request = BridgetMessage::new(
        "da78fd70-41e8-424c-a88d-e29e2c5babcd",
        "5da585af-5bc7-4808-985f-c73670633990",
        "mission réelle",
    );
    request.reply = true;
    write_frame(&mut sender_writer, &WrapperToDaemon::Send(request.clone()));
    let ack = read_frame(&mut sender_reader);
    assert!(
        matches!(ack, DaemonToWrapper::Ack { .. }),
        "accusé du tour : {ack:?}"
    );
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
    stop_session(daemon, wrapper);
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
    let socket = fixture::socket(&root);
    let (mut sender_reader, mut sender_writer) = sender(&socket);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        write_frame(&mut sender_writer, &WrapperToDaemon::ListAgents);
        match read_frame(&mut sender_reader) {
            DaemonToWrapper::AgentList { agents }
                if agents
                    .iter()
                    .any(|agent| agent.agent_id == "5da585af-5bc7-4808-985f-c73670633990") =>
            {
                let codex = agents
                    .iter()
                    .find(|agent| agent.agent_id == "5da585af-5bc7-4808-985f-c73670633990")
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
    stop_session(daemon, wrapper);
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
    let auth_source = std::env::var("BRIDGET_TEST_CODEX_AUTH_FILE")
        .expect("chemin explicite d'authentification ChatGPT requis");
    let credentials = fs::read(auth_source).expect("authentification locale lisible");
    let auth: serde_json::Value = serde_json::from_slice(&credentials).unwrap();
    assert!(
        auth["OPENAI_API_KEY"].is_null(),
        "une clé API ne valide pas l'abonnement"
    );
    assert!(
        auth["tokens"]["access_token"].is_string(),
        "session ChatGPT absente"
    );
    let codex_home = root.join("provider/codex-private");
    fixture::private_dir(&codex_home).unwrap();
    let _credentials_guard = PrivateCredentials(codex_home.clone());
    fixture::private_write(&codex_home.join("auth.json"), credentials).unwrap();
    let registry = serde_json::json!({
        "agents": {
            "codex": {
                "command": codex,
                "args": ["-c", "model=\"gpt-5.6-terra\"", "-c", "model_reasoning_effort=\"low\"", "app-server"],
                "protocol": "codex_app_server",
                "permissions": "deny",
                "queue_capacity": 4,
                "notify_timeout_secs": 30,
                "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
                "pass_env": ["CODEX_HOME"],
                "mcp": {"interactive":"none","acp_session":false},
                "capabilities": {"execution_paths":["codex_app_server"],"models":{"gpt-5.6-terra":{"efforts":["low"]}}}
            }
        }
    });
    let registry_path = root.join("state/agents.json");
    fs::write(
        &registry_path,
        serde_json::to_vec(&registry).expect("registre JSON"),
    )
    .expect("registre privé");
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600))
        .expect("permissions registre");

    let daemon = start_daemon(&root);
    let wrapper = start_native_wrapper(
        &root,
        &[(
            "CODEX_HOME".to_string(),
            codex_home.to_string_lossy().into_owned(),
        )],
    );
    let socket = fixture::socket(&root);
    let (mut sender_reader, mut sender_writer) = sender(&socket);
    let (mut attach_reader, mut attach_writer) = attach(&socket);
    let deadline = Instant::now() + Duration::from_secs(30);
    let subscription_id = loop {
        write_frame(
            &mut attach_writer,
            &WrapperToDaemon::Subscribe {
                agent: "5da585af-5bc7-4808-985f-c73670633990".to_string(),
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
        .find(|agent| agent.agent_id == "5da585af-5bc7-4808-985f-c73670633990")
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
    let who = fixture::isolated_command(&root)
        .arg("who")
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
        "da78fd70-41e8-424c-a88d-e29e2c5babcd",
        "5da585af-5bc7-4808-985f-c73670633990",
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
    stop_session(daemon, wrapper);
    fs::remove_dir_all(root).expect("nettoyage HOME isolé");
}
