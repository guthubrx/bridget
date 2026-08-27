use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::config::DurationClasses;
use maicie::domain::{ClasseDuree, DecisionCoordination, EtatDecision, EtatObjectif, TypeDecision};
use maicie::store::{MaicieStore, StoreError};
use serde_json::{Value, json};
use std::cell::Cell;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

#[test]
fn commandes_objectif_rendent_et_persistent_les_decisions_explicites() {
    let fixture = Fixture::new();
    let objective_id = fixture.seed();

    let status = run(&fixture, &["status", &objective_id, "--json"]);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_json: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status_json["kind"], "status");
    assert_eq!(
        status_json["coordination"][0]["objective"]["id"],
        objective_id
    );
    assert_eq!(status_json["transport_snapshot"]["state"], "unknown");
    assert_eq!(status_json["stream_state"], "unavailable");

    let add = run(
        &fixture,
        &[
            "objective",
            &objective_id,
            "add-participant",
            "sentry",
            "--json",
        ],
    );
    assert!(add.status.success());
    let add_json: Value = serde_json::from_slice(&add.stdout).unwrap();
    assert_eq!(add_json["kind"], "decision");
    assert_eq!(add_json["decision"]["kind"], "ajouter_participant");

    let remove = run(
        &fixture,
        &[
            "objective",
            &objective_id,
            "remove-participant",
            "sentry",
            "--reason",
            "périmètre terminé",
            "--json",
        ],
    );
    assert!(remove.status.success());

    let summary = run(
        &fixture,
        &["objective", &objective_id, "summarize", "--json"],
    );
    assert!(summary.status.success());
    let summary_json: Value = serde_json::from_slice(&summary.stdout).unwrap();
    assert_eq!(summary_json["kind"], "summary");
    assert_eq!(
        summary_json["coordination"]["decisions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let close = run(
        &fixture,
        &[
            "objective",
            &objective_id,
            "close",
            "--reason",
            "validation humaine",
            "--json",
        ],
    );
    assert!(close.status.success());
    let close_json: Value = serde_json::from_slice(&close.stdout).unwrap();
    assert_eq!(close_json["decision"]["kind"], "cloturer");

    let final_status = run(&fixture, &["status", &objective_id, "--json"]);
    assert!(final_status.status.success());
    let final_json: Value = serde_json::from_slice(&final_status.stdout).unwrap();
    assert_eq!(final_json["coordination"][0]["objective"]["etat"], "clos");
    assert_eq!(
        final_json["coordination"][0]["decisions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    for command in [
        vec![
            "objective",
            &objective_id,
            "add-participant",
            "sentry",
            "--json",
        ],
        vec![
            "objective",
            &objective_id,
            "remove-participant",
            "sentry",
            "--reason",
            "trop tard",
            "--json",
        ],
        vec![
            "objective",
            &objective_id,
            "close",
            "--reason",
            "doublon",
            "--json",
        ],
    ] {
        let rejected = run(&fixture, &command);
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("objectif déjà clos"));
    }
    // La synthèse est une lecture factuelle : elle reste disponible après la
    // clôture, sans créer de décision supplémentaire.
    assert!(
        run(
            &fixture,
            &["objective", &objective_id, "summarize", "--json"]
        )
        .status
        .success()
    );
}

#[test]
fn decision_obsolete_nefface_pas_l_issue_terminale_reinjectee() {
    let fixture = Fixture::new();
    let objective_id = Uuid::parse_str(&fixture.seed()).unwrap();
    let snapshot = MaicieStore::open(&fixture.database)
        .unwrap()
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .pop()
        .unwrap()
        .objective;

    // Réinjection de la course C3 : une issue terminale est durablement
    // constatée après la projection CLI, juste avant sa décision de clôture.
    // Sans `WHERE id AND state`, le close obsolète écraserait cet état.
    let mut stale_close = snapshot.clone();
    stale_close.clore(200).unwrap();
    let mut terminal_objective = snapshot;
    terminal_objective
        .transition(EtatObjectif::AEvaluer, 150)
        .unwrap();
    let connection = rusqlite::Connection::open(&fixture.database).unwrap();
    connection
        .execute(
            "UPDATE objectives SET state = 'a_evaluer', payload_json = ?1 WHERE id = ?2",
            rusqlite::params![
                serde_json::to_vec(&terminal_objective).unwrap(),
                objective_id.to_string(),
            ],
        )
        .unwrap();
    drop(connection);

    let stale_decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind: TypeDecision::Cloturer,
        proposee_par: "maicie".to_string(),
        etat: EtatDecision::Appliquee,
        motif: "clôture devenue obsolète".to_string(),
    };
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    assert!(matches!(
        store.apply_objective_decision(
            &stale_decision,
            Some(&stale_close),
            EtatObjectif::EnCoordination,
        ),
        Err(StoreError::Conflict("objectif modifié concurremment"))
    ));

    let after = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(after.objective.etat, EtatObjectif::AEvaluer);
    assert!(after.decisions.is_empty());
}

fn run(fixture: &Fixture, tail: &[&str]) -> std::process::Output {
    let server = start_local_daemon(&fixture.socket, fixture.first_run.replace(false));
    let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
    command.args(tail);
    command.args(["--config", fixture.config.to_str().unwrap()]);
    let output = command.output().unwrap();
    server.join().unwrap();
    output
}

fn start_local_daemon(socket: &std::path::Path, reconcile_outbox: bool) -> thread::JoinHandle<()> {
    let socket = socket.to_owned();
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).unwrap();
        ready_tx.send(()).unwrap();
        accept_daemon_identity(&listener);
        if reconcile_outbox {
            accept_reconcile_accepted(&listener);
        }
        accept_empty_guichet(&listener);
        accept_empty_coordination(&listener);
    });
    ready_rx.recv().unwrap();
    server
}

fn accept_reconcile_accepted(listener: &UnixListener) {
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
    let lookup = read_json(&mut reader);
    assert_eq!(lookup["type"], "Lookup");
    assert_eq!(lookup["operation_kind"], "send");
    let message_id = lookup["idempotency_key"].as_str().unwrap();
    write_json(
        &mut writer,
        json!({
            "type":"IdempotencyResult",
            "operation_kind":"send",
            "idempotency_key":message_id,
            "issue":{"kind":"accepted","expires_at":4_102_444_800i64}
        }),
    );
}

fn accept_daemon_identity(listener: &UnixListener) {
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
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"DaemonIdentityRequest"})
    );
    write_json(
        &mut writer,
        json!({
            "type":"DaemonIdentityReport",
            "host":bridget_core::local_host(),
            "db_path":"/var/lib/bridget/bridget.db"
        }),
    );
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
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":3600,
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
    config: PathBuf,
    socket: PathBuf,
    first_run: Cell<bool>,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("maicie-cli-objective-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        let config = root.join("maicie.json");
        let socket = root.join("bridget.sock");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version": 1,
                "bridget_socket": socket,
                "database_path": database,
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": []
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            database,
            config,
            socket,
            first_run: Cell::new(true),
        }
    }

    fn seed(&self) -> String {
        let mut store = MaicieStore::open(&self.database).unwrap();
        let candidates = vec![DelegationCandidate {
            name: "prospective".to_string(),
            tags: vec![],
            available: true,
            dnd: false,
        }];
        let request = DelegateRequest {
            goal: "objectif de test",
            explicit_target: Some("prospective"),
            required_tags: &[],
            duration: ClasseDuree::Normale,
            reply: true,
            constat_id: None,
            review_target: None,
            suite: maicie::domain::SuiteObjective::Aucune,
            depends_on: &[],
            references: &[],
            idempotency_key: "cli-objective-seed",
            now: 100,
            retry_until: 150,
            dedup_retained_until: 200,
            max_frame_bytes: 256 * 1024,
        };
        let result = delegate(
            &mut store,
            DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            "maicie",
            &candidates,
            &request,
        )
        .unwrap();
        let DelegateResult::Created(created) = result else {
            panic!("délégation attendue");
        };
        created.objective_id.to_string()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
