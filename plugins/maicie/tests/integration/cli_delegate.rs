use maicie::domain::EtatOutboxDelegation;
use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

#[test]
fn deux_delegations_cli_avec_la_meme_cle_rejouent_les_memes_ids_sans_seconde_outbox() {
    let fixture = Fixture::new();
    let (ready_tx, ready_rx) = mpsc::channel();
    let socket = fixture.socket.clone();
    let server = thread::spawn(move || serve_delegate_fixture(&socket, ready_tx));
    ready_rx.recv().unwrap();

    let first = run_delegate(&fixture, "conversation/delegate-1");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_json: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(first_json["kind"], "created");
    assert_eq!(first_json["replayed"], false);
    let first_objective = first_json["objective_id"].clone();
    let first_delegation = first_json["delegations"][0]["id"].clone();
    let first_message = first_json["delegations"][0]["message_id"].clone();
    assert!(first_message.as_str().is_some_and(|id| !id.is_empty()));
    assert_eq!(first_json["delegations"][0]["duration"], "courte");
    assert_eq!(first_json["delegations"][0]["timeout_secs"], 30);
    assert!(
        first_json["delegations"][0]["deadline_contractuelle"]
            .as_i64()
            .is_some_and(|deadline| deadline > 0)
    );

    let second = run_delegate(&fixture, "conversation/delegate-1");
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_json: Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(second_json["kind"], "created");
    assert_eq!(second_json["replayed"], true);
    assert_eq!(second_json["objective_id"], first_objective);
    assert_eq!(second_json["delegations"][0]["id"], first_delegation);
    assert_eq!(second_json["delegations"][0]["message_id"], first_message);
    assert_eq!(
        second_json["delegations"][0]["deadline_contractuelle"],
        first_json["delegations"][0]["deadline_contractuelle"]
    );

    let store = MaicieStore::open(&fixture.database).unwrap();
    let pending = store.pending_delegation_outboxes().unwrap();
    assert!(pending.is_empty());
    let snapshot = store
        .recovery_snapshot(Uuid::parse_str(first_message.as_str().unwrap()).unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Accepted);
    drop(store);
    server.join().unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args([
            "status",
            "--config",
            fixture.config.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(status.status.success());
    let status_json: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status_json["transport_snapshot"]["state"], "unknown");
    assert_eq!(
        status_json["coordination"][0]["remises_locales"][0]["state"],
        "accepted"
    );
    assert_eq!(
        status_json["coordination"][0]["remises_locales"][0]["issue"]["kind"],
        "accepted"
    );
}

#[test]
fn erreur_d_usage_est_json_et_sort_avec_le_code_contractuel() {
    let output = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args(["delegate", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "usage");
}

#[test]
fn echec_d_expedition_post_commit_conserve_une_unique_outbox_prepared() {
    let fixture = Fixture::new();
    let (ready_tx, ready_rx) = mpsc::channel();
    let socket = fixture.socket.clone();
    let server = thread::spawn(move || serve_list_only_fixture(&socket, ready_tx));
    ready_rx.recv().unwrap();

    let output = run_delegate(&fixture, "conversation/post-commit-unavailable");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();

    let store = MaicieStore::open(&fixture.database).unwrap();
    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].state, EtatOutboxDelegation::Prepared);
}

#[test]
fn cible_connectee_sans_profil_explique_l_inscription_maicie_manquante() {
    let fixture = Fixture::new();
    let (ready_tx, ready_rx) = mpsc::channel();
    let socket = fixture.socket.clone();
    let server = thread::spawn(move || {
        serve_list_only_fixture_with_agent(&socket, ready_tx, "cursorbridget")
    });
    ready_rx.recv().unwrap();

    let output = run_delegate_to(&fixture, "cursorbridget", "registration/missing-profile");

    assert_eq!(output.status.code(), Some(5));
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "target_missing_maicie_profile");
    assert_eq!(
        error["error"]["message"],
        format!(
        "agent Bridget sans profil Maicie : cursorbridget; ajoutez un profil dans {} avec \"agent_name\": \"cursorbridget\"",
            fixture.config.display()
        )
    );
    server.join().unwrap();
}

fn run_delegate(fixture: &Fixture, idempotency_key: &str) -> std::process::Output {
    run_delegate_to(fixture, "prospective", idempotency_key)
}

fn run_delegate_to(fixture: &Fixture, target: &str, idempotency_key: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args([
            "delegate",
            "--config",
            fixture.config.to_str().unwrap(),
            "--goal",
            "vérifier le contrat CLI",
            "--suite",
            "aucune",
            "--to",
            target,
            "--duration",
            "courte",
            "--idempotency-key",
            idempotency_key,
            "--json",
        ])
        .output()
        .unwrap()
}

fn serve_delegate_fixture(socket: &Path, ready: mpsc::Sender<()>) {
    let listener = UnixListener::bind(socket).unwrap();
    ready.send(()).unwrap();
    let (stream, _) = listener.accept().unwrap();
    serve_guichet_empty(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_coordination_empty(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_client_handshake(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_agent_list(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_reconcile_send(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_guichet_empty(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_coordination_empty(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_client_handshake(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_agent_list(stream);
}

fn serve_list_only_fixture(socket: &Path, ready: mpsc::Sender<()>) {
    serve_list_only_fixture_with_agent(socket, ready, "prospective");
}

fn serve_list_only_fixture_with_agent(socket: &Path, ready: mpsc::Sender<()>, agent: &str) {
    let listener = UnixListener::bind(socket).unwrap();
    ready.send(()).unwrap();
    let (stream, _) = listener.accept().unwrap();
    serve_guichet_empty(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_coordination_empty(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_client_handshake(stream);
    let (stream, _) = listener.accept().unwrap();
    serve_agent_list_named(stream, agent);
}

/// Toute commande Maicie relève d'abord le guichet en rôle `service`. La
/// fixture répond explicitement vide : ce test CLI ne doit pas confondre
/// l'absence de demande guichet avec une ancienne poignée `client`.
fn serve_guichet_empty(stream: UnixStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type": "RoleHandshake", "role": "service"})
    );
    write_json(
        &mut writer,
        json!({"type": "RoleAccepted", "role": "service"}),
    );
    let hello = read_json(&mut reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(hello["service"], "maicie");
    assert_eq!(hello["capabilities"], json!(["maicie_guichet"]));
    write_json(
        &mut writer,
        json!({
            "type": "ServiceWelcome",
            "version": 1,
            "horizon_secs": 60,
            "issued_at_tolerance_secs": 5,
            "capabilities": ["maicie_guichet"]
        }),
    );
    assert_eq!(
        read_json(&mut reader),
        json!({"type": "guichet_claim_next", "v": 1})
    );
    write_json(&mut writer, json!({"type": "guichet_empty", "v": 1}));
}

fn serve_coordination_empty(stream: UnixStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type": "RoleHandshake", "role": "service"})
    );
    write_json(
        &mut writer,
        json!({"type": "RoleAccepted", "role": "service"}),
    );
    let hello = read_json(&mut reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(
        hello["capabilities"],
        json!(["maicie_guichet", "coordination_events_v2"])
    );
    write_json(
        &mut writer,
        json!({
            "type": "ServiceWelcome",
            "version": 1,
            "horizon_secs": 60,
            "issued_at_tolerance_secs": 5,
            "capabilities": ["maicie_guichet", "coordination_events_v2"]
        }),
    );
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"coordination_subscribe","v":2})
    );
    write_json(
        &mut writer,
        json!({"type":"coordination_snapshot_caught_up","v":2}),
    );
}

fn serve_client_handshake(stream: UnixStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    serve_client_handshake_io(&mut reader, &mut writer);
}

fn serve_client_handshake_io(reader: &mut BufReader<UnixStream>, writer: &mut UnixStream) {
    assert_eq!(
        read_json(reader),
        json!({"type": "RoleHandshake", "role": "client"})
    );
    write_json(writer, json!({"type": "RoleAccepted", "role": "client"}));
    let hello = read_json(reader);
    assert_eq!(hello["type"], "ClientHello");
    assert!(
        hello["issuer_scope"]
            .as_str()
            .is_some_and(|scope| !scope.is_empty())
    );
    write_json(
        writer,
        json!({
            "type": "ClientWelcome",
            "version": 1,
            "horizon_secs": 60,
            "issued_at_tolerance_secs": 5,
            "capabilities": ["send_idempotent", "lookup"]
        }),
    );
}

fn serve_agent_list(stream: UnixStream) {
    serve_agent_list_named(stream, "prospective");
}

fn serve_agent_list_named(stream: UnixStream, name: &str) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    assert_eq!(read_json(&mut reader), json!({"type": "ListAgents"}));
    write_json(
        &mut writer,
        json!({
            "type": "AgentList",
            "agents": [{
                "name": name,
                "agent_type": "codex",
                "connection_id": "fixture-1",
                "host": "fixture",
                "transport": "acp",
                "os": "macOS",
                "state": "connected",
                "last_seen_secs": 0,
                "reconnect_count": 0,
                "domain": "bridget",
                "model": "test",
                "effort": "low"
            }]
        }),
    );
}

fn serve_reconcile_send(stream: UnixStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    serve_client_handshake_io(&mut reader, &mut writer);

    let lookup = read_json(&mut reader);
    assert_eq!(lookup["type"], "Lookup");
    assert_eq!(lookup["operation_kind"], "send");
    let message_id = lookup["idempotency_key"].as_str().unwrap().to_string();
    write_json(
        &mut writer,
        json!({
            "type": "IdempotencyResult",
            "operation_kind": "send",
            "idempotency_key": message_id,
            "issue": {"kind": "idempotency_expired"}
        }),
    );

    let send = read_json(&mut reader);
    assert_eq!(send["type"], "SendIdempotent");
    assert_eq!(send["message_id"], message_id);
    write_json(
        &mut writer,
        json!({
            "type": "IdempotencyResult",
            "operation_kind": "send",
            "idempotency_key": message_id,
            "issue": {"kind": "accepted", "expires_at": 4_102_444_800i64}
        }),
    );
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn write_json(writer: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *writer, &value).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
}

struct Fixture {
    root: PathBuf,
    socket: PathBuf,
    database: PathBuf,
    config: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("maicie-cli-delegate-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let socket = std::env::temp_dir().join(format!("mc-{}.sock", Uuid::new_v4()));
        let database = root.join("maicie.sqlite3");
        let config = root.join("maicie.json");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version": 1,
                "bridget_socket": socket,
                "database_path": database,
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": [{
                    "id": "prospective",
                    "display_name": "Prospective",
                    "tags": ["review"],
                    "personality_ref": "profiles/prospective.md",
                    "tools": ["bridget_send"],
                    "spawn_order_ref": "agents/prospective"
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            socket,
            database,
            config,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket);
        let _ = fs::remove_dir_all(&self.root);
    }
}
