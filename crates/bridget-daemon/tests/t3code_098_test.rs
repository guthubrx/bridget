#![cfg(feature = "test-support")]
//! Oracles de la session 098 : vrai daemon, vrai pont `bridget t3 serve`,
//! faux t3code (serveur HTTP local + faux CLI `t3`). Racines privées, aucun
//! compte, aucune écriture hors de la racine du test.
#[path = "support/idempotent.rs"]
pub mod fixture;

use bridget_core::BridgetMessage;
use bridget_transport::protocol::AgentInfo;
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/t3code_server_098.py"
);

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        fixture::cleanup_child(&mut self.0);
    }
}

struct Harness {
    root: PathBuf,
    port: u16,
    _server: ChildGuard,
}

impl Harness {
    fn start(label: &str) -> Self {
        let root = fixture::test_root(label);
        for relative in ["t3home", "bin", "logs"] {
            fixture::private_dir(&root.join(relative)).unwrap();
        }
        // Faux CLI `t3` : un script 0700 qui délègue au fixture Python.
        let t3 = root.join("bin/t3");
        fixture::private_write(
            &t3,
            format!("#!/bin/sh\nexec /usr/bin/python3 {FIXTURE} \"$@\"\n"),
        )
        .unwrap();
        fs::set_permissions(&t3, fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = Command::new("/usr/bin/python3")
            .arg(FIXTURE)
            .arg("serve")
            .arg(root.join("logs"))
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("T3CODE_HOME", root.join("t3home"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(private_log(&root, "t3server.log")))
            .spawn()
            .expect("faux serveur t3code");
        fixture::track(&server);
        let stdout = server.stdout.take().expect("sortie du faux serveur");
        let guard = ChildGuard(server);
        let mut line = String::new();
        BufReader::new(stdout).read_line(&mut line).unwrap();
        let port: u16 = line
            .trim()
            .strip_prefix("READY ")
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(|| panic!("faux serveur non prêt : {line:?}"));
        Self {
            root,
            port,
            _server: guard,
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = fixture::isolated_command(&self.root);
        command
            .args(args)
            .env("T3CODE_HOME", self.root.join("t3home"))
            .env("BRIDGET_T3_BIN", self.root.join("bin/t3"))
            .env("BRIDGET_T3_POLL_MS", "150")
            .env("BRIDGET_T3_TURN_WAIT_SECS", "10");
        command
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        self.command(args).output().unwrap()
    }

    fn cli_log(&self) -> String {
        fs::read_to_string(self.root.join("t3home/fake-t3.log")).unwrap_or_default()
    }

    fn control(&self, path: &str, payload: serde_json::Value) {
        let output = Command::new("/usr/bin/curl")
            .args([
                "-s",
                "-o",
                "/dev/null",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "-d",
                &payload.to_string(),
                &format!("http://127.0.0.1:{}{path}", self.port),
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "contrôle {path}");
    }

    fn dispatches(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.root.join("logs/dispatch.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn serve(&self) -> ChildGuard {
        let child = self
            .command(&["t3", "serve"])
            .env("RUST_LOG", "info")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(private_log(&self.root, "serve.log")))
            .spawn()
            .expect("pont t3code");
        fixture::track(&child);
        ChildGuard(child)
    }

    fn agents(&self) -> Vec<AgentInfo> {
        let mut probe = fixture::Client::connect(&fixture::socket(&self.root));
        probe.send(WrapperToDaemon::ListAgents);
        match probe.receive() {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("liste d'agents : {other:?}"),
        }
    }

    fn wait_t3_agents(&self, expected: usize, timeout: Duration) -> Vec<AgentInfo> {
        let deadline = Instant::now() + timeout;
        loop {
            let agents: Vec<AgentInfo> = self
                .agents()
                .into_iter()
                .filter(|a| a.transport == "t3code" && a.state == "connected")
                .collect();
            if agents.len() == expected {
                return agents;
            }
            assert!(
                Instant::now() < deadline,
                "{expected} agent(s) t3code attendu(s), vus : {agents:?}\n{}",
                fs::read_to_string(self.root.join("serve.log")).unwrap_or_default()
            );
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn private_log(root: &Path, name: &str) -> fs::File {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(name))
        .unwrap()
}

fn start_daemon(root: &Path) -> ChildGuard {
    let child = fixture::isolated_command(root)
        .arg("daemon")
        .env("RUST_LOG", "info")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(private_log(root, "daemon.log")))
        .spawn()
        .expect("daemon réel");
    fixture::track(&child);
    // Le garde possède l'enfant dès maintenant : même une panique d'attente
    // le nettoie, et aucun daemon de test ne survit au banc.
    let daemon = ChildGuard(child);
    let socket = fixture::socket(root);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            return daemon;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("daemon absent : {}", socket.display());
}

struct Sender {
    reader: BufReader<std::os::unix::net::UnixStream>,
    writer: std::io::BufWriter<std::os::unix::net::UnixStream>,
}

impl Sender {
    fn register(socket: &Path) -> Self {
        let stream = std::os::unix::net::UnixStream::connect(socket).expect("connexion émetteur");
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .expect("délai de lecture");
        let mut sender = Self {
            reader: BufReader::new(stream.try_clone().expect("clone")),
            writer: std::io::BufWriter::new(stream),
        };
        sender.send(WrapperToDaemon::Register {
            agent_type: "fixture".to_string(),
            identity_version: 2,
            agent_id: fixture::ACTOR.to_string(),
            host: None,
            transport: Some("unix".to_string()),
            channel: None.into(),
            mode: Some(bridget_transport::protocol::PresenceMode::Acp),
            location: None,
            os: None,
            instance_id: Some("098-sender".to_string()),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        });
        assert!(matches!(
            sender.receive(),
            DaemonToWrapper::Registered { .. }
        ));
        sender
    }

    fn send(&mut self, frame: WrapperToDaemon) {
        use std::io::Write;
        writeln!(
            self.writer,
            "{}",
            bridget_transport::protocol::encode(&frame).expect("encodage")
        )
        .expect("écriture");
        self.writer.flush().expect("flush");
    }

    fn receive(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .expect("le daemon n'a rien envoyé dans le délai");
        assert!(!line.is_empty(), "EOF daemon");
        bridget_transport::protocol::decode(line.trim_end()).expect("trame daemon")
    }
}

fn ask(sender: &mut Sender, to: &str, body: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new(fixture::ACTOR, to, body);
    message.reply = true;
    sender.send(WrapperToDaemon::Send(message.clone()));
    let ack = sender.receive();
    assert!(
        matches!(ack, DaemonToWrapper::Ack { .. }),
        "envoi refusé : {ack:?}"
    );
    message
}

fn wait_reply(sender: &mut Sender, request: &BridgetMessage) -> BridgetMessage {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        assert!(Instant::now() < deadline, "pas de réponse liée");
        if let DaemonToWrapper::Deliver(reply) = sender.receive()
            && reply.in_reply_to.as_deref() == Some(request.id.as_str())
        {
            return reply;
        }
    }
}

#[test]
fn spec098_install_sans_cli_t3_echoue_sans_rien_laisser() {
    let harness = Harness::start("098-sans-t3");
    let output = harness
        .command(&["t3", "install", "--no-service"])
        .env("BRIDGET_T3_BIN", harness.root.join("bin/absent"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("introuvable"),
        "cause nommée attendue : {stderr}"
    );
    assert!(!harness.root.join("state/t3code/token.json").exists());
    assert!(!harness.root.join("state/t3code/manifest.json").exists());
}

#[test]
fn spec098_pont_expose_remet_repond_renouvelle_et_retire() {
    let harness = Harness::start("098-pont");
    let _daemon = start_daemon(&harness.root);

    // US1 : installation sans toucher t3code, jeton privé, idempotente.
    let output = harness.run(&["t3", "install", "--no-service"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let token_path = harness.root.join("state/t3code/token.json");
    assert_eq!(
        fs::metadata(&token_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let token: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&token_path).unwrap()).unwrap();
    let label = token["label"].as_str().unwrap().to_string();
    assert!(label.starts_with("bridget-"), "{label}");
    assert!(
        harness
            .cli_log()
            .contains(&format!("--label {label} --ttl 30d --json"))
    );
    let again = harness.run(&["t3", "install", "--no-service"]);
    assert!(again.status.success());
    assert!(String::from_utf8_lossy(&again.stdout).contains("déjà installé"));
    assert_eq!(harness.cli_log().matches("session issue").count(), 1);

    // US2 : chaque fil vivant est un agent `t3code | cli` du type du fournisseur.
    let _serve = harness.serve();
    let agents = harness.wait_t3_agents(1, Duration::from_secs(10));
    let alpha = &agents[0];
    assert_eq!(alpha.agent_type, "claude");
    assert_eq!(
        alpha.mode,
        Some(bridget_transport::protocol::PresenceMode::Cli)
    );
    let alpha_id = alpha.agent_id.clone();
    let status = harness.run(&["t3", "status"]);
    let status_text = String::from_utf8_lossy(&status.stdout).to_string();
    assert!(status_text.contains(&alpha_id), "{status_text}");

    // US3 : remise dans le fil et réponse liée renvoyée par le pont.
    let mut sender = Sender::register(&fixture::socket(&harness.root));
    let request = ask(&mut sender, &alpha_id, "Question 098 numéro un");
    let reply = wait_reply(&mut sender, &request);
    assert_eq!(reply.from, alpha_id);
    assert!(
        reply.body.contains("echo[bridget]") && reply.body.contains("Question 098 numéro un"),
        "{}",
        reply.body
    );
    let dispatches = harness.dispatches();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0]["commandId"], request.id);
    let injected = dispatches[0]["text"].as_str().unwrap();
    assert!(injected.contains("Message Bridget de") && injected.contains(&request.id));

    // Corrélation par rang : un humain occupe le fil, notre message attend son
    // tour et reçoit SA réponse, pas celle de l'humain.
    harness.control("/__test/slow", serde_json::json!({"seconds": 1.5}));
    harness.control(
        "/__test/human",
        serde_json::json!({"threadId": "thread-alpha", "text": "question humaine"}),
    );
    let second = ask(&mut sender, &alpha_id, "Question 098 numéro deux");
    let reply = wait_reply(&mut sender, &second);
    assert!(
        reply.body.contains("echo[bridget]") && reply.body.contains("numéro deux"),
        "{}",
        reply.body
    );
    assert!(!reply.body.contains("humaine"), "{}", reply.body);

    // Journal : la saisie humaine et la réponse du fil sont projetées.
    let journal_dir = harness.root.join("state/sessions").join(&alpha_id);
    let deadline = Instant::now() + Duration::from_secs(5);
    let journal_text = loop {
        let text = fs::read_dir(&journal_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "jsonl"))
            .filter_map(|e| fs::read_to_string(e.path()).ok())
            .collect::<String>();
        if text.contains("question humaine") && text.contains("numéro deux") {
            break text;
        }
        assert!(Instant::now() < deadline, "journal incomplet : {text}");
        thread::sleep(Duration::from_millis(100));
    };
    assert!(
        !journal_text.contains("historique humain"),
        "l'historique antérieur à l'installation ne doit pas être rejoué"
    );

    // US4 : un 401 isolé déclenche un seul renouvellement, sans perte.
    harness.control("/__test/slow", serde_json::json!({"seconds": 0.2}));
    harness.control("/__test/unauthorized", serde_json::json!({"count": 1}));
    let third = ask(&mut sender, &alpha_id, "Question 098 numéro trois");
    let reply = wait_reply(&mut sender, &third);
    assert!(reply.body.contains("numéro trois"), "{}", reply.body);
    let log = harness.cli_log();
    assert_eq!(log.matches("session issue").count(), 2, "{log}");
    assert!(
        log.contains(&format!(
            "session revoke {}",
            token["session_id"].as_str().unwrap()
        )),
        "{log}"
    );

    // t3code régénère les titres et l'humain les change : le nom suit, sinon
    // l'agent garderait un nom périmé que personne ne reconnaît.
    harness.control(
        "/__test/title",
        serde_json::json!({"threadId": "thread-alpha", "title": "Alpha renommé"}),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let agents = harness.agents();
        if agents
            .iter()
            .any(|a| a.transport == "t3code" && a.display_name == "Alpha renommé")
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "nom non resynchronisé : {:?}",
            agents
                .iter()
                .map(|a| a.display_name.clone())
                .collect::<Vec<_>>()
        );
        thread::sleep(Duration::from_millis(100));
    }

    // La politique du fil peut changer entre deux sondages : le dispatch doit
    // reprendre celle du dernier snapshot, jamais une valeur figée à l'ouverture
    // du lien (t3code refuserait la commande, ou pire on figerait des droits).
    harness.control(
        "/__test/policy",
        serde_json::json!({"threadId": "thread-alpha", "runtimeMode": "auto-accept-edits", "interactionMode": "plan"}),
    );
    let quatre = ask(&mut sender, &alpha_id, "Question 098 numéro quatre");
    let reply = wait_reply(&mut sender, &quatre);
    assert!(reply.body.contains("numéro quatre"), "{}", reply.body);
    let dernier = harness
        .dispatches()
        .pop()
        .expect("au moins un dispatch enregistré");
    assert_eq!(dernier["runtimeMode"], "auto-accept-edits");
    assert_eq!(dernier["interactionMode"], "plan");

    // Nouveau fil jamais démarré (sans session fournisseur, forme réelle) :
    // exposé quand même, typé par son modèle, et joignable pour son premier
    // message — sinon un fil neuf ne pourrait jamais recevoir de mission.
    harness.control(
        "/__test/thread",
        serde_json::json!({"id": "thread-beta", "title": "Beta", "provider": "codex", "withSession": false}),
    );
    let agents = harness.wait_t3_agents(2, Duration::from_secs(10));
    let beta = agents
        .iter()
        .find(|a| a.agent_type == "codex")
        .expect("fil neuf exposé");
    let premier = ask(
        &mut sender,
        &beta.agent_id.clone(),
        "Premier message du fil neuf",
    );
    let reply = wait_reply(&mut sender, &premier);
    assert!(
        reply.body.contains("Premier message du fil neuf"),
        "{}",
        reply.body
    );
    // Un fil rangé quitte l'annuaire comme un fil archivé : l'import
    // d'historique de t3code en produit des centaines d'un coup.
    harness.control(
        "/__test/settle",
        serde_json::json!({"threadId": "thread-beta"}),
    );
    let agents = harness.wait_t3_agents(1, Duration::from_secs(10));
    assert_eq!(agents[0].agent_type, "claude", "seul le fil actif subsiste");

    harness.control(
        "/__test/archive",
        serde_json::json!({"threadId": "thread-alpha"}),
    );
    harness.wait_t3_agents(0, Duration::from_secs(10));

    // Retrait : session révoquée, état effacé, second retrait inoffensif.
    let output = harness.run(&["t3", "uninstall"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!harness.root.join("state/t3code").exists());
    let sessions: Vec<serde_json::Value> = serde_json::from_str(
        &fs::read_to_string(harness.root.join("t3home/fake-sessions.json")).unwrap(),
    )
    .unwrap();
    assert!(sessions.is_empty(), "sessions restantes : {sessions:?}");
    let output = harness.run(&["t3", "uninstall"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("rien à retirer"));
}

#[test]
fn spec098_session_refusee_deux_fois_est_un_echec_explicite() {
    let harness = Harness::start("098-401");
    let _daemon = start_daemon(&harness.root);
    assert!(
        harness
            .run(&["t3", "install", "--no-service"])
            .status
            .success()
    );
    harness.control("/__test/unauthorized", serde_json::json!({"count": 1000}));
    let _serve = harness.serve();
    let status_path = harness.root.join("state/t3code/status.json");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let status: Option<serde_json::Value> = fs::read_to_string(&status_path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        if status.as_ref().is_some_and(|s| {
            s["state"] == "auth_failed" && s["detail"].as_str().unwrap().contains("deux fois")
        }) {
            break;
        }
        assert!(Instant::now() < deadline, "statut : {status:?}");
        thread::sleep(Duration::from_millis(100));
    }
    // Un seul renouvellement tenté, pas une boucle d'émissions : le verrou
    // tient dans la durée, sinon le pont émettrait une session par minute.
    assert_eq!(harness.cli_log().matches("session issue").count(), 2);
    thread::sleep(Duration::from_secs(5));
    assert_eq!(
        harness.cli_log().matches("session issue").count(),
        2,
        "aucune émission supplémentaire après l'échec définitif"
    );
    let status = harness.run(&["t3", "status"]);
    assert!(String::from_utf8_lossy(&status.stdout).contains("auth_failed"));
}
