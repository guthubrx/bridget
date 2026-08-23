use maicie::bridget_client::{BridgetClientLimits, PublicMessage};
use maicie::domain::{
    ClasseDuree, Delegation, EtatDelegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif,
    ObjectifCoordonne, OutboxDelegation,
};
use maicie::outbox::{stable_body_hash, PreparedDelegation};
use maicie::reconcile::{
    reconcile_startup_at, reconcile_startup_at_observed, reconcile_startup_at_with_limits,
    ReconcileAction,
};
use maicie::store::{LocalFailureReason, MaicieStore};
use serde_json::{json, Value};
use std::fs::{self, DirBuilder};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

const OBJECTIVE_ID: &str = "41000000-0000-4000-8000-000000000001";
const DELEGATION_ID: &str = "42000000-0000-4000-8000-000000000002";
const MESSAGE_ID: &str = "43000000-0000-4000-8000-000000000003";
const ISSUED_AT: i64 = 1_000;
const CHILD_MODE: &str = "MAICIE_T008_CHILD_MODE";
const CHILD_DATABASE: &str = "MAICIE_T008_CHILD_DATABASE";
const CHILD_SOCKET: &str = "MAICIE_T008_CHILD_SOCKET";
const CHILD_BARRIER: &str = "MAICIE_T008_CHILD_BARRIER";

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

#[test]
fn prepared_dans_l_horizon_est_rejouee_octet_pour_octet_apres_lookup() {
    let root = unique_root("replay");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("bridget.sock");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    let expected_message = prepared.message_bytes.clone();
    store.create_prepared_delegation(&prepared).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(&mut writer, json!({"kind":"idempotency_expired"}));

        let raw_send = read_line(&mut reader);
        let request: Value = serde_json::from_str(&raw_send).unwrap();
        assert_eq!(request["type"], "SendIdempotent");
        let expected_fragment = format!(
            "\"message\":{}",
            std::str::from_utf8(&expected_message).unwrap()
        );
        assert!(
            raw_send.contains(&expected_fragment),
            "les bytes de message doivent être injectés sans reconstruction"
        );
        write_issue(
            &mut writer,
            json!({"kind":"accepted","expires_at":1_100_i64}),
        );
    });

    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::Rejouee {
            objective_id,
            message_id,
            ..
        }] if *objective_id == uuid(OBJECTIVE_ID) && *message_id == uuid(MESSAGE_ID)
    ));
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    assert_eq!(
        store
            .recovery_snapshot(uuid(MESSAGE_ID))
            .unwrap()
            .unwrap()
            .last_issue
            .unwrap()["kind"],
        "accepted"
    );
    server.join().unwrap();
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn prepared_hors_horizon_devient_rejected_sans_rejeu() {
    let root = unique_root("expired");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("bridget.sock");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(&mut writer, json!({"kind":"idempotency_expired"}));
        let mut unexpected = String::new();
        assert_eq!(reader.read_line(&mut unexpected).unwrap(), 0);
    });

    let report = reconcile_startup_at(&mut store, &socket, 5_000).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::IssueTerminale {
            objective_id,
            message_id,
            issue: maicie::bridget_client::IdempotencyIssue::IdempotencyExpired,
        }] if *objective_id == uuid(OBJECTIVE_ID) && *message_id == uuid(MESSAGE_ID)
    ));
    let snapshot = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
    assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Rejected);
    assert_eq!(snapshot.last_issue.unwrap()["kind"], "idempotency_expired");
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    drop(store);
    server.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issue_terminale_connue_est_persistee_sans_second_envoi() {
    let root = unique_root("known-terminal");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("bridget.sock");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(
            &mut writer,
            json!({
                "kind":"rejected",
                "category":"policy",
                "reason":"refus explicite",
                "expires_at":1_100_i64
            }),
        );
        let mut unexpected = String::new();
        assert_eq!(reader.read_line(&mut unexpected).unwrap(), 0);
    });

    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::IssueTerminale {
            objective_id,
            message_id,
            issue: maicie::bridget_client::IdempotencyIssue::Rejected { .. },
        }] if *objective_id == uuid(OBJECTIVE_ID) && *message_id == uuid(MESSAGE_ID)
    ));
    assert_eq!(
        store
            .recovery_snapshot(uuid(MESSAGE_ID))
            .unwrap()
            .unwrap()
            .outbox
            .state,
        EtatOutboxDelegation::Rejected
    );
    drop(store);
    server.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_utilise_la_meme_borne_runtime_que_la_preparation() {
    let root = unique_root("runtime-limit");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("bridget.sock");
    let mut store = MaicieStore::open(&database).unwrap();
    let default_limits = BridgetClientLimits::default();
    let high_limits = BridgetClientLimits {
        max_frame_bytes: default_limits.max_frame_bytes + 128,
        ..default_limits
    };
    let empty_message = PublicMessage {
        id: uuid(MESSAGE_ID).to_string(),
        from: maicie::MAICIE_IDENTITY.to_string(),
        to: "prospective".to_string(),
        body: String::new(),
        reply: true,
        hops: 4,
        reply_timeout: Some(60),
        deadline_at: Some(1_060),
        in_reply_to: None,
    };
    let replay_overhead = serde_json::to_vec(&json!({
        "type":"SendIdempotent",
        "message":empty_message,
        "message_id":MESSAGE_ID,
        "issued_at":ISSUED_AT,
    }))
    .unwrap()
    .len();
    let body = "x".repeat(default_limits.max_frame_bytes - replay_overhead);
    let prepared = fixture_with_body(store.issuer_scope(), body, high_limits.max_frame_bytes);
    let expected_message = prepared.message_bytes.clone();
    store.create_prepared_delegation(&prepared).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || serve_lookup_then_replay(listener, &expected_message));
    let report = reconcile_startup_at_with_limits(&mut store, &socket, 1_010, high_limits).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::Rejouee { .. }]
    ));
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    server.join().unwrap();
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn trame_trop_grande_devient_rejet_local_durable_sans_rejeu() {
    let root = unique_root("local-frame-too-large");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("bridget.sock");
    let mut store = MaicieStore::open(&database).unwrap();
    let default_limits = BridgetClientLimits::default();
    let high_limits = BridgetClientLimits {
        max_frame_bytes: default_limits.max_frame_bytes + 128,
        ..default_limits
    };
    let empty_message = PublicMessage {
        id: uuid(MESSAGE_ID).to_string(),
        from: maicie::MAICIE_IDENTITY.to_string(),
        to: "prospective".to_string(),
        body: String::new(),
        reply: true,
        hops: 4,
        reply_timeout: Some(60),
        deadline_at: Some(1_060),
        in_reply_to: None,
    };
    let replay_overhead = serde_json::to_vec(&json!({
        "type":"SendIdempotent",
        "message":empty_message,
        "message_id":MESSAGE_ID,
        "issued_at":ISSUED_AT,
    }))
    .unwrap()
    .len();
    let prepared = fixture_with_body(
        store.issuer_scope(),
        "x".repeat(default_limits.max_frame_bytes - replay_overhead),
        high_limits.max_frame_bytes,
    );
    store.create_prepared_delegation(&prepared).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(&mut writer, json!({"kind":"idempotency_expired"}));
        let mut unexpected = String::new();
        assert_eq!(reader.read_line(&mut unexpected).unwrap(), 0);
    });

    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::RejetLocal {
            objective_id,
            message_id,
            reason: LocalFailureReason::FrameTooLarge,
        }] if *objective_id == uuid(OBJECTIVE_ID) && *message_id == uuid(MESSAGE_ID)
    ));
    let snapshot = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
    assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Rejected);
    assert_eq!(
        snapshot.last_issue.unwrap(),
        json!({"local":"frame_too_large"})
    );
    server.join().unwrap();

    assert!(reconcile_startup_at(&mut store, &socket, 1_011)
        .unwrap()
        .actions
        .is_empty());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn enveloppe_corrompue_devient_rejet_local_sans_ouvrir_de_socket() {
    let root = unique_root("local-invalid-envelope");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("absent.sock");
    {
        let mut store = MaicieStore::open(&database).unwrap();
        let prepared = fixture(store.issuer_scope());
        store.create_prepared_delegation(&prepared).unwrap();
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE delegation_outbox SET body_bytes = ?1 WHERE message_id = ?2",
            rusqlite::params![b"corrompu".as_slice(), MESSAGE_ID],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::RejetLocal {
            objective_id,
            message_id,
            reason: LocalFailureReason::InvalidEnvelope,
        }] if *objective_id == uuid(OBJECTIVE_ID) && *message_id == uuid(MESSAGE_ID)
    ));
    assert!(reconcile_startup_at(&mut store, &socket, 1_011)
        .unwrap()
        .actions
        .is_empty());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn connexion_indisponible_conserve_l_outbox_prepared() {
    let root = unique_root("unavailable");
    let database = root.join("maicie.sqlite3");
    let socket = root.join("absent.sock");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::TransportIndisponible { .. }]
    ));
    assert_eq!(
        store
            .recovery_snapshot(uuid(MESSAGE_ID))
            .unwrap()
            .unwrap()
            .outbox
            .state,
        EtatOutboxDelegation::Prepared
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn crash_reel_aux_frontieres_de_reprise_ne_cree_ni_double_envoi_ni_perte() {
    crash_avant_socket_rejoue_une_seule_fois();
    crash_apres_ecriture_avant_ack_reste_en_cours_sans_rejeu();
    crash_apres_ack_avant_commit_rejoue_l_issue_terminale_sans_envoi();
}

#[test]
fn crash_recovery_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let database = PathBuf::from(std::env::var_os(CHILD_DATABASE).unwrap());
    let socket = PathBuf::from(std::env::var_os(CHILD_SOCKET).unwrap());
    let barrier = PathBuf::from(std::env::var_os(CHILD_BARRIER).unwrap());
    let mut store = MaicieStore::open(database).unwrap();
    reconcile_startup_at_observed(&mut store, socket, 1_010, |phase| {
        let expected = match mode.as_str() {
            "before_socket" => maicie::reconcile::ReconcilePhase::BeforeSocket,
            "after_ack" => maicie::reconcile::ReconcilePhase::AfterIssueBeforeStoreCommit,
            other => panic!("mode enfant inconnu: {other}"),
        };
        if phase == expected {
            block_at_barrier(&barrier, &mode);
        }
        Ok(())
    })
    .unwrap();
}

fn crash_avant_socket_rejoue_une_seule_fois() {
    let root = unique_root("crash-before-socket");
    let database = root.join("maicie.sqlite3");
    {
        let mut store = MaicieStore::open(&database).unwrap();
        let prepared = fixture(store.issuer_scope());
        store.create_prepared_delegation(&prepared).unwrap();
    }
    let socket = root.join("bridget.sock");
    let (mut child, mut barrier, barrier_path) =
        spawn_crash_child("before_socket", &database, &socket, &root);
    wait_barrier(&mut barrier, "before_socket");
    child.kill().unwrap();
    child.wait().unwrap();
    fs::remove_file(barrier_path).unwrap();

    let mut store = MaicieStore::open(&database).unwrap();
    let expected = store
        .pending_delegation_outboxes()
        .unwrap()
        .remove(0)
        .message_bytes;
    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || serve_lookup_then_replay(listener, &expected));
    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::Rejouee { .. }]
    ));
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    server.join().unwrap();
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

fn crash_apres_ecriture_avant_ack_reste_en_cours_sans_rejeu() {
    let root = unique_root("crash-after-write");
    let database = root.join("maicie.sqlite3");
    {
        let mut store = MaicieStore::open(&database).unwrap();
        let prepared = fixture(store.issuer_scope());
        store.create_prepared_delegation(&prepared).unwrap();
    }
    let socket = root.join("bridget.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let (sent, observed_send) = mpsc::sync_channel(1);
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(&mut writer, json!({"kind":"idempotency_expired"}));
        let raw_send = read_line(&mut reader);
        assert!(raw_send.contains("\"type\":\"SendIdempotent\""));
        sent.send(()).unwrap();
        let mut eof = [0_u8; 1];
        assert_eq!(
            reader.read(&mut eof).unwrap(),
            0,
            "le serveur garde la socket ouverte jusqu'au crash enfant"
        );
    });
    let mut child = spawn_crash_child_without_barrier("after_ack", &database, &socket, &root);
    observed_send.recv_timeout(Duration::from_secs(5)).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    server.join().unwrap();
    fs::remove_file(&socket).unwrap();

    let mut store = MaicieStore::open(&database).unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(
            &mut writer,
            json!({
                "kind":"outcome_unknown",
                "expires_at":1_100_i64,
                "delivery_id":"delivery-en-cours"
            }),
        );
        let mut unexpected = String::new();
        assert_eq!(reader.read_line(&mut unexpected).unwrap(), 0);
    });
    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::IssueEnCours { .. }]
    ));
    assert_eq!(
        store
            .recovery_snapshot(uuid(MESSAGE_ID))
            .unwrap()
            .unwrap()
            .outbox
            .state,
        EtatOutboxDelegation::OutcomeUnknown
    );
    server.join().unwrap();
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

fn crash_apres_ack_avant_commit_rejoue_l_issue_terminale_sans_envoi() {
    let root = unique_root("crash-after-ack");
    let database = root.join("maicie.sqlite3");
    {
        let mut store = MaicieStore::open(&database).unwrap();
        let prepared = fixture(store.issuer_scope());
        store.create_prepared_delegation(&prepared).unwrap();
    }
    let socket = root.join("bridget.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || serve_lookup_then_replay(listener, b""));
    let (mut child, mut barrier, barrier_path) =
        spawn_crash_child("after_ack", &database, &socket, &root);
    wait_barrier(&mut barrier, "after_ack");
    child.kill().unwrap();
    child.wait().unwrap();
    server.join().unwrap();
    fs::remove_file(&socket).unwrap();
    fs::remove_file(barrier_path).unwrap();

    let mut store = MaicieStore::open(&database).unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        negotiate_client(&mut reader, &mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        write_issue(
            &mut writer,
            json!({"kind":"accepted","expires_at":1_100_i64}),
        );
        let mut unexpected = String::new();
        assert_eq!(reader.read_line(&mut unexpected).unwrap(), 0);
    });
    let report = reconcile_startup_at(&mut store, &socket, 1_010).unwrap();
    assert!(matches!(
        &report.actions[..],
        [ReconcileAction::IssueTerminale { .. }]
    ));
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    server.join().unwrap();
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

fn serve_lookup_then_replay(listener: UnixListener, expected_message: &[u8]) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    negotiate_client(&mut reader, &mut writer);
    assert_eq!(read_json(&mut reader)["type"], "Lookup");
    write_issue(&mut writer, json!({"kind":"idempotency_expired"}));
    let raw_send = read_line(&mut reader);
    if !expected_message.is_empty() {
        let expected_fragment = format!(
            "\"message\":{}",
            std::str::from_utf8(expected_message).unwrap()
        );
        assert!(raw_send.contains(&expected_fragment));
    }
    write_issue(
        &mut writer,
        json!({"kind":"accepted","expires_at":1_100_i64}),
    );
}

fn spawn_crash_child(
    mode: &str,
    database: &Path,
    socket: &Path,
    root: &Path,
) -> (Child, UnixStream, PathBuf) {
    let barrier_path = root.join(format!("{mode}.barrier.sock"));
    let listener = UnixListener::bind(&barrier_path).unwrap();
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        sender.send(listener.accept().map(|pair| pair.0)).unwrap();
    });
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("ack_lost_recovery::crash_recovery_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_DATABASE, database)
        .env(CHILD_SOCKET, socket)
        .env(CHILD_BARRIER, &barrier_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let barrier = match receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(stream)) => stream,
        other => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("connexion à la barrière impossible : {other:?}");
        }
    };
    (child, barrier, barrier_path)
}

fn spawn_crash_child_without_barrier(
    mode: &str,
    database: &Path,
    socket: &Path,
    root: &Path,
) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("ack_lost_recovery::crash_recovery_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_DATABASE, database)
        .env(CHILD_SOCKET, socket)
        .env(CHILD_BARRIER, root.join("unused.barrier.sock"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
}

fn wait_barrier(stream: &mut UnixStream, expected: &str) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).unwrap();
    assert_eq!(line.trim(), format!("BARRIER:{expected}"));
}

fn block_at_barrier(path: &Path, name: &str) {
    let mut socket = UnixStream::connect(path).unwrap();
    socket
        .write_all(format!("BARRIER:{name}\n").as_bytes())
        .unwrap();
    socket.flush().unwrap();
    let mut release = [0_u8; 1];
    socket.read_exact(&mut release).unwrap();
}

fn negotiate_client(reader: &mut BufReader<UnixStream>, writer: &mut BufWriter<UnixStream>) {
    assert_eq!(
        read_json(reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(writer, json!({"type":"RoleAccepted","role":"client"}));
    let hello = read_json(reader);
    assert_eq!(hello["type"], "ClientHello");
    write_json(
        writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3_600_i64,
            "issued_at_tolerance_secs":30_i64,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
}

fn write_issue(writer: &mut BufWriter<UnixStream>, issue: Value) {
    write_json(
        writer,
        json!({
            "type":"IdempotencyResult",
            "operation_kind":"send",
            "idempotency_key":MESSAGE_ID,
            "issue":issue
        }),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    (reader, BufWriter::new(stream))
}

fn read_line(reader: &mut BufReader<UnixStream>) -> String {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    line
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    serde_json::from_str(&read_line(reader)).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).unwrap();
    writer.flush().unwrap();
}

fn fixture(issuer_scope: &str) -> PreparedDelegation {
    fixture_with_body(
        issuer_scope,
        "Inspecte les invariants".to_string(),
        BridgetClientLimits::default().max_frame_bytes,
    )
}

fn fixture_with_body(
    issuer_scope: &str,
    body: String,
    max_frame_bytes: usize,
) -> PreparedDelegation {
    let objective = ObjectifCoordonne {
        id: uuid(OBJECTIVE_ID),
        but: "Récupérer une délégation".to_string(),
        mode: ModeObjectif::Delegue,
        etat: EtatObjectif::EnCoordination,
        cree_at: ISSUED_AT,
        mis_a_jour_at: ISSUED_AT,
        synthese: None,
        decision_en_attente_id: None,
    };
    let delegation = Delegation {
        id: uuid(DELEGATION_ID),
        objectif_id: objective.id,
        participant: "prospective".to_string(),
        instruction: body.clone(),
        duree: ClasseDuree::Normale,
        etat: EtatDelegation::Creee,
        raison: "reprise contrôlée".to_string(),
    };
    let body = body.into_bytes();
    let outbox = OutboxDelegation {
        message_id: uuid(MESSAGE_ID),
        delegation_id: delegation.id,
        target: delegation.participant.clone(),
        body_bytes: body.clone(),
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_060,
        body_hash: stable_body_hash(&body),
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_050,
        dedup_retained_until: 1_100,
    };
    PreparedDelegation::new(
        objective,
        delegation,
        outbox,
        issuer_scope,
        ISSUED_AT,
        max_frame_bytes,
    )
    .unwrap()
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}

fn unique_root(label: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(format!(
        "/tmp/maicie-t008-{}-{sequence}-{label}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder.create(&root).unwrap();
    root
}
