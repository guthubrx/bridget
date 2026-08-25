use base64::Engine;
use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::bridget_client::IdempotencyIssue;
use maicie::config::DurationClasses;
use maicie::domain::ClasseDuree;
use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[test]
fn status_capture_des_faits_acp_ephemeres_sans_etat_metier_invente() {
    let fixture = Fixture::new(Some(400));
    fixture.seed_delegation_terminale();
    let (ready_tx, ready_rx) = mpsc::channel();
    let socket = fixture.socket.clone();
    let server = thread::spawn(move || serve_status(&socket, ready_tx));
    ready_rx.recv().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args([
            "status",
            "--config",
            fixture.config.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["transport_snapshot"]["state"], "unknown");
    assert_eq!(
        value["transport_snapshot"]["reason"],
        "request_status_public_unavailable"
    );
    assert_eq!(
        value["availability"][0]["agent"], "prospective",
        "{}",
        value
    );
    assert_eq!(value["availability"][0]["source"], "bridget");
    assert_eq!(value["runtime"][0]["stream_state"], "fresh");
    assert_eq!(value["runtime"][0]["source"], "acp_subscription");
    assert_eq!(
        value["runtime"][0]["observations"][0]["nature"],
        "permission_auto_decidee"
    );
    assert_eq!(
        value["runtime"][0]["observations"][0]["details"]["outcome"],
        "selected"
    );
    assert_eq!(value["freshness"]["state"], "fresh");
    assert_eq!(value["coordination_freshness"]["state"], "fresh");
    // Les états JSON ci-dessus sont l'oracle. Un grep anti-libellé (« bloqu »,
    // « en_attente ») casserait à la première reformulation sans changer le
    // comportement — retiré volontairement.
    server.join().unwrap();
}

#[test]
fn status_sans_budget_ne_fabrique_ni_fraicheur_ni_capture() {
    let fixture = Fixture::new(None);
    fixture.seed_delegation_terminale();

    let output = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args([
            "status",
            "--config",
            fixture.config.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["stream_state"], "unavailable");
    assert_eq!(value["availability_state"], "unavailable");
    assert_eq!(value["availability_reason"], "budget_capture_non_configure");
    assert_eq!(value["freshness"]["reason"], "budget_capture_non_configure");
    assert_eq!(
        value["coordination_freshness"]["state"], "unavailable",
        "la relève Bridget indisponible reste une observation distincte des faits locaux"
    );
    assert!(value["runtime"].as_array().unwrap().is_empty());

    let plain = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args(["status", "--config", fixture.config.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(plain.status.success());
    let rendered = String::from_utf8(plain.stdout).unwrap();
    assert!(rendered.contains("snapshot_transport=unknown"));
    assert!(rendered.contains("fraîcheur=unavailable"));
}

#[test]
fn status_epuise_le_budget_global_sans_conserver_une_fausse_observation() {
    let fixture = Fixture::new(Some(120));
    fixture.seed_delegation_terminale();
    let (ready_tx, ready_rx) = mpsc::channel();
    let socket = fixture.socket.clone();
    let server = thread::spawn(move || serve_status_until_timeout(&socket, ready_tx));
    ready_rx.recv().unwrap();

    let output = run_status_with_watchdog(&fixture);
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["runtime"][0]["stream_state"], "unavailable");
    assert_eq!(value["runtime"][0]["reason"], "budget_capture_epuise");
    assert_eq!(value["freshness"]["state"], "unavailable");
    assert!(
        server.join().unwrap(),
        "le client doit fermer l'abonnement bloqué après l'épuisement du budget"
    );
}

/// Le watchdog est un disjoncteur de harnais, non une mesure de performance.
/// Mutation discriminante : si une phase réinitialise l'échéance au lieu de
/// propager le budget global, le serveur garde la socket ouverte et ce garde
/// expire au lieu de recevoir l'EOF attendu.
fn run_status_with_watchdog(fixture: &Fixture) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args([
            "status",
            "--config",
            fixture.config.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("watchdog : status n'a pas fermé l'abonnement après son budget global");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn serve_status(socket: &std::path::Path, ready: mpsc::Sender<()>) {
    let _ = fs::remove_file(socket);
    let listener = UnixListener::bind(socket).unwrap();
    ready.send(()).unwrap();
    accept_empty_guichet(&listener);
    accept_empty_coordination(&listener);
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
    assert_eq!(read_json(&mut reader)["type"], "ClientHello");
    write_json(
        &mut writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
    let (stream, _) = listener.accept().unwrap();
    let (mut directory_reader, mut directory_writer) = split(stream);
    assert_eq!(
        read_json(&mut directory_reader),
        json!({"type":"ListAgents"})
    );
    write_json(
        &mut directory_writer,
        json!({"type":"AgentList","agents":[{
            "name":"prospective","agent_type":"codex","connection_id":"conn-1",
            "host":"local","transport":"acp","state":"connected",
            "last_seen_secs":0,"reconnect_count":0
        }]}),
    );

    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"attach"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
    assert_eq!(read_json(&mut reader)["type"], "Subscribe");
    write_json(
        &mut writer,
        json!({"type":"Subscribed","subscription_id":"sub-status"}),
    );
    let entry = serde_json::to_vec(&json!({
        "v":1,"seq":7,"ts":"2026-08-23T12:00:00Z","session_id":"session-status",
        "event":"permission","payload":{"decision":{"outcome":"selected","option_id":"allow"}}
    }))
    .unwrap();
    write_json(
        &mut writer,
        json!({
            "type":"JournalFragment","subscription_id":"sub-status","seq":7,
            "offset":0,"final":true,
            "bytes":base64::engine::general_purpose::STANDARD.encode(entry)
        }),
    );
    write_json(
        &mut writer,
        json!({"type":"SnapshotCaughtUp","subscription_id":"sub-status","through_seq":7}),
    );
}

fn serve_status_until_timeout(socket: &std::path::Path, ready: mpsc::Sender<()>) -> bool {
    let _ = fs::remove_file(socket);
    let listener = UnixListener::bind(socket).unwrap();
    ready.send(()).unwrap();
    accept_empty_guichet(&listener);
    accept_empty_coordination(&listener);
    accept_status_client(&listener);
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(read_json(&mut reader), json!({"type":"ListAgents"}));
    write_json(
        &mut writer,
        json!({"type":"AgentList","agents":[{
            "name":"prospective","agent_type":"codex","connection_id":"conn-1",
            "host":"local","transport":"acp","state":"connected",
            "last_seen_secs":0,"reconnect_count":0
        }]}),
    );
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"attach"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
    assert_eq!(read_json(&mut reader)["type"], "Subscribe");
    write_json(
        &mut writer,
        json!({"type":"Subscribed","subscription_id":"sub-timeout"}),
    );
    reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut unexpected = String::new();
    matches!(reader.read_line(&mut unexpected), Ok(0))
}

fn accept_empty_guichet(listener: &UnixListener) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(&mut reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(hello["service"], "maicie");
    assert_eq!(hello["capabilities"], json!(["maicie_guichet"]));
    write_json(
        &mut writer,
        json!({
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet"]
        }),
    );
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"guichet_claim_next","v":1})
    );
    write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
}

fn accept_empty_coordination(listener: &UnixListener) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(&mut reader);
    assert_eq!(
        hello["capabilities"],
        json!(["maicie_guichet", "coordination_events_v2"])
    );
    write_json(
        &mut writer,
        json!({
            "type":"ServiceWelcome","version":1,"horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet","coordination_events_v2"]
        }),
    );
    assert_eq!(read_json(&mut reader)["type"], "coordination_subscribe");
    write_json(
        &mut writer,
        json!({"type":"coordination_snapshot_caught_up","v":2}),
    );
}

fn accept_status_client(listener: &UnixListener) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
    assert_eq!(read_json(&mut reader)["type"], "ClientHello");
    write_json(
        &mut writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().unwrap()),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    serde_json::to_writer(&mut *writer, &value).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    socket: PathBuf,
    config: PathBuf,
}

impl Fixture {
    fn new(status_capture_budget_ms: Option<u64>) -> Self {
        let root = std::env::temp_dir().join(format!("maicie-status-sources-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        // La borne Unix/macOS est plus courte que certains répertoires tmp :
        // le socket doit rester indépendant du répertoire de fixture long.
        let socket = std::env::temp_dir().join(format!("mc-status-{}.sock", Uuid::new_v4()));
        let config = root.join("maicie.json");
        let mut config_value = json!({
            "version":1,
            "bridget_socket":socket,
            "database_path":database,
            "durations":{"short_secs":30,"normal_secs":60,"long_secs":90},
            "profiles":[]
        });
        if let Some(budget_ms) = status_capture_budget_ms {
            config_value["status_capture_budget_ms"] = json!(budget_ms);
        }
        fs::write(&config, serde_json::to_vec(&config_value).unwrap()).unwrap();
        Self {
            root,
            database,
            socket,
            config,
        }
    }

    fn seed_delegation_terminale(&self) {
        let mut store = MaicieStore::open(&self.database).unwrap();
        let candidates = vec![DelegationCandidate {
            name: "prospective".to_string(),
            tags: Vec::new(),
            available: true,
            dnd: false,
        }];
        let result = delegate(
            &mut store,
            DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            "maicie",
            &candidates,
            &DelegateRequest {
                goal: "observer une permission",
                explicit_target: Some("prospective"),
                required_tags: &[],
                duration: ClasseDuree::Normale,
                reply: false,
                constat_id: None,
                review_target: None,
                suite: maicie::domain::SuiteObjective::Aucune,
                depends_on: &[],
                references: &[],
                idempotency_key: "status-sources",
                now: 100,
                retry_until: 150,
                dedup_retained_until: 200,
                max_frame_bytes: 256 * 1024,
            },
        )
        .unwrap();
        let DelegateResult::Created(created) = result else {
            panic!("délégation attendue")
        };
        store
            .record_lookup_issue(
                created.message_id.unwrap(),
                &IdempotencyIssue::Accepted { expires_at: 150 },
                110,
            )
            .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket);
        let _ = fs::remove_dir_all(&self.root);
    }
}
