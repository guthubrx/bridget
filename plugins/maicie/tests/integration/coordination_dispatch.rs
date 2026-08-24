use maicie::bridget_client::BridgetClientLimits;
use maicie::reconcile::{
    NotificationReconcileAction, NotificationReconcilePhase,
    reconcile_notification_startup_observed_with_limits,
    reconcile_notification_startup_with_limits,
};
use maicie::store::MaicieStore;
use rusqlite::{Connection, params};
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);
const CHILD_MODE: &str = "MAICIE_T1608_CRASH_MODE";
const CHILD_DATABASE: &str = "MAICIE_T1608_CRASH_DATABASE";
const CHILD_SOCKET: &str = "MAICIE_T1608_CRASH_SOCKET";
const CHILD_BARRIER: &str = "MAICIE_T1608_CRASH_BARRIER";

#[test]
fn notification_rejoue_les_octets_durables_et_ne_les_emet_qu_une_fois() {
    let fixture = Fixture::new("exact-bytes");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);

    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion notification attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);

        let mut actual = Vec::new();
        reader
            .read_until(b'\n', &mut actual)
            .expect("rejeu notification lu");
        assert_eq!(
            actual, expected,
            "le dispatcher doit transmettre les bytes persistés, sans re-sérialisation"
        );
        write_json(
            &mut writer,
            json!({
                "type":"IdempotencyResult",
                "operation_kind":"send",
                "idempotency_key":message_id.to_string(),
                "issue":{"kind":"accepted","expires_at":1_787_600_000_i64}
            }),
        );
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(300)),
    )
    .expect("reprise notification");
    assert!(matches!(
        report.actions.as_slice(),
        [NotificationReconcileAction::Issue { message_id: actual, .. }] if *actual == message_id
    ));
    server.join().expect("serveur notification termine");
    assert!(
        store
            .pending_notification_outboxes()
            .expect("pending relues")
            .is_empty(),
        "Accepted consomme durablement la boîte"
    );

    // Mutation discriminante : réémettre une notification déjà accepted
    // ouvrirait une seconde connexion. Le listener fermé rendrait ce second
    // passage visible comme une erreur au lieu d'un faux vert silencieux.
    let second = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(80)),
    )
    .expect("aucune boîte terminale ne doit être réémise");
    assert!(second.actions.is_empty());
}

#[test]
fn notification_partage_une_echeance_absolue_entre_handshake_et_rejeu() {
    let fixture = Fixture::new("absolute-deadline");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);

    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion notification attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"RoleHandshake","role":"client"})
        );
        thread::sleep(Duration::from_millis(170));
        write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
        let _hello = read_json(&mut reader);
        thread::sleep(Duration::from_millis(170));
        // Une échéance absolue correcte a déjà expiré. Un client mutant qui
        // renouvelle l'échéance par phase recevrait ce welcome et enverrait
        // alors SendIdempotent : l'assertion ci-dessous le rend rouge.
        let _ = write_json_if_open(
            &mut writer,
            json!({
                "type":"ClientWelcome",
                "version":1,
                "horizon_secs":3600,
                "issued_at_tolerance_secs":30,
                "capabilities":["send_idempotent","lookup"]
            }),
        );
        reader
            .get_mut()
            .set_nonblocking(true)
            .expect("lecture serveur non bloquante");
        let until = Instant::now() + Duration::from_millis(250);
        loop {
            let mut unexpected = String::new();
            match reader.read_line(&mut unexpected) {
                Ok(0) if Instant::now() >= until => break,
                Ok(0) => thread::sleep(Duration::from_millis(5)),
                Err(error) if error.kind() == ErrorKind::WouldBlock && Instant::now() < until => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Ok(_) => panic!(
                    "un délai réinitialisé par phase aurait envoyé une trame après le budget : {unexpected}"
                ),
                Err(error) => panic!("lecture serveur inattendue : {error}"),
            }
        }
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(300)),
    )
    .expect("budget global respecté");
    assert!(matches!(
        report.actions.as_slice(),
        [NotificationReconcileAction::TransportIndisponible { message_id: actual, .. }]
            if *actual == message_id
    ));
    server.join().expect("serveur budget termine");
    assert_eq!(
        store
            .pending_notification_outboxes()
            .expect("pending relues")
            .len(),
        1,
        "une échéance dépassée ne consomme jamais la notification"
    );
}

#[test]
fn crash_reel_apres_write_ou_ack_rejoue_la_meme_notification_sans_doublon_local() {
    crash_avant_socket_apres_commit_laisse_la_boite_durable();
    crash_apres_write_avant_ack_rejoue_exactement();
    crash_apres_ack_avant_consommation_rejoue_exactement();
}

#[test]
fn notification_crash_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let database = PathBuf::from(std::env::var_os(CHILD_DATABASE).expect("base enfant"));
    let socket = PathBuf::from(std::env::var_os(CHILD_SOCKET).expect("socket enfant"));
    let barrier = PathBuf::from(std::env::var_os(CHILD_BARRIER).expect("barrière enfant"));
    let phase = match mode.as_str() {
        "before_socket" => NotificationReconcilePhase::BeforeSocket,
        "after_write" => NotificationReconcilePhase::AfterWriteBeforeAck,
        "after_ack" => NotificationReconcilePhase::AfterIssueBeforeStoreCommit,
        other => panic!("mode enfant inconnu : {other}"),
    };
    let mut store = MaicieStore::open(database).expect("store enfant");
    reconcile_notification_startup_observed_with_limits(
        &mut store,
        socket,
        limits(Duration::from_secs(2)),
        |observed| {
            if observed == phase {
                block_at_barrier(&barrier, &mode);
            }
            Ok(())
        },
    )
    .expect("reprise enfant");
}

fn crash_avant_socket_apres_commit_laisse_la_boite_durable() {
    let fixture = Fixture::new("crash-before-socket");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);
    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);

    // La boîte est déjà commitée (frontière T1607), mais aucun listener
    // n'existe. Le jalon prouve qu'un kill avant socket ne peut produire I/O.
    let (mut child, mut barrier, barrier_path) = spawn_crash_child("before_socket", &fixture);
    wait_barrier(&mut barrier, "before_socket");
    child.kill().expect("kill enfant réel");
    child.wait().expect("wait enfant réel");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    let store = MaicieStore::open(&fixture.database_path).expect("store après crash avant socket");
    assert_eq!(
        store
            .pending_notification_outboxes()
            .expect("outbox durable")
            .len(),
        1,
        "le kill avant I/O ne consomme pas une boîte déjà commitée"
    );
    drop(store);

    replay_after_crash(&fixture, message_id, &expected);
}

fn crash_apres_write_avant_ack_rejoue_exactement() {
    let fixture = Fixture::new("crash-after-write");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);
    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);

    let listener = fixture.bind();
    let first_expected = expected.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion enfant attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);
        let mut actual = Vec::new();
        reader
            .read_until(b'\n', &mut actual)
            .expect("write enfant lu");
        assert_eq!(
            actual, first_expected,
            "le write avant crash utilise les bytes durables"
        );
        let mut eof = [0_u8; 1];
        assert_eq!(
            reader.get_mut().read(&mut eof).expect("EOF enfant"),
            0,
            "le serveur garde la socket ouverte : seul le kill enfant coupe avant ACK"
        );
        let _ = writer.flush();
    });
    let (mut child, mut barrier, barrier_path) = spawn_crash_child("after_write", &fixture);
    wait_barrier(&mut barrier, "after_write");
    child.kill().expect("kill enfant réel");
    child.wait().expect("wait enfant réel");
    server.join().expect("serveur crash write");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    fs::remove_file(fixture.socket_path()).expect("socket crash supprimée");

    replay_after_crash(&fixture, message_id, &expected);
}

fn crash_apres_ack_avant_consommation_rejoue_exactement() {
    let fixture = Fixture::new("crash-after-ack");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);
    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);

    let listener = fixture.bind();
    let first_expected = expected.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion enfant attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);
        let mut actual = Vec::new();
        reader
            .read_until(b'\n', &mut actual)
            .expect("send enfant lu");
        assert_eq!(actual, first_expected, "l'ACK porte sur la trame exacte");
        write_accepted(&mut writer, message_id);
    });
    let (mut child, mut barrier, barrier_path) = spawn_crash_child("after_ack", &fixture);
    wait_barrier(&mut barrier, "after_ack");
    child.kill().expect("kill enfant réel");
    child.wait().expect("wait enfant réel");
    server.join().expect("serveur crash ACK");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    fs::remove_file(fixture.socket_path()).expect("socket crash supprimée");

    replay_after_crash(&fixture, message_id, &expected);
}

fn replay_after_crash(fixture: &Fixture, message_id: Uuid, expected: &[u8]) {
    let listener = fixture.bind();
    let expected = expected.to_vec();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("reprise attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);
        let mut actual = Vec::new();
        reader.read_until(b'\n', &mut actual).expect("rejeu lu");
        assert_eq!(
            actual, expected,
            "un crash ne change ni clé ni bytes de notification"
        );
        write_accepted(&mut writer, message_id);
    });
    let mut store = MaicieStore::open(&fixture.database_path).expect("store repris");
    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("reprise après crash");
    assert!(matches!(
        report.actions.as_slice(),
        [NotificationReconcileAction::Issue { message_id: actual, .. }] if *actual == message_id
    ));
    assert!(
        store
            .pending_notification_outboxes()
            .expect("outbox relue")
            .is_empty(),
        "un seul Accepted durable consomme la notification après le crash"
    );
    server.join().expect("serveur reprise termine");
}

const ISSUED_AT: i64 = 1_787_500_000;

fn limits(timeout: Duration) -> BridgetClientLimits {
    BridgetClientLimits {
        connect_timeout: timeout,
        io_timeout: timeout,
        max_frame_bytes: 64 * 1024,
    }
}

fn message_bytes(message_id: Uuid) -> Vec<u8> {
    let mut bytes = b"{\"id\":\"".to_vec();
    bytes.extend_from_slice(message_id.to_string().as_bytes());
    bytes.extend_from_slice(
        r#"","from":"maicie","to":"alice","body":"clôture attestée","reply":false,"hops":4,"extension_future":{"opaque":true}}"#
            .as_bytes(),
    );
    bytes
}

fn replay_frame(message_bytes: &[u8], message_id: Uuid, issued_at: i64) -> Vec<u8> {
    let mut frame = br#"{"type":"SendIdempotent","message":"#.to_vec();
    frame.extend_from_slice(message_bytes);
    frame.extend_from_slice(br#","message_id":"#);
    frame.extend_from_slice(
        serde_json::to_string(&message_id.to_string())
            .unwrap()
            .as_bytes(),
    );
    frame.extend_from_slice(br#","issued_at":"#);
    frame.extend_from_slice(issued_at.to_string().as_bytes());
    frame.extend_from_slice(b"}\n");
    frame
}

fn insert_notification(database_path: &PathBuf, message_id: Uuid, message_bytes: &[u8]) {
    let _store = MaicieStore::open(database_path).expect("schéma Maicie créé");
    let connection = Connection::open(database_path).expect("base fixture ouverte");
    // Le harnais isole le dispatcher du producteur T1607 : il ne fabrique pas
    // de clôture de coordination, mais insère une boîte déjà durable. La FK
    // vers l'objectif est donc explicitement hors de l'oracle C.
    connection
        .execute_batch("PRAGMA foreign_keys = OFF;")
        .expect("FK fixture désactivées");
    connection
        .execute(
            "INSERT INTO notification_outbox(
                 message_id, idempotency_key, issued_at, objective_id, delegation_id,
                 generation, event_id, policy_version, recipient, message_bytes,
                 state, last_issue_json, terminal
             ) VALUES(?1, ?2, ?3, ?4, NULL, NULL, ?5, 1, 'alice', ?6, 'prepared', NULL, 0)",
            params![
                message_id.to_string(),
                format!("notification:{message_id}"),
                ISSUED_AT,
                Uuid::new_v4().to_string(),
                format!("event:{message_id}"),
                message_bytes,
            ],
        )
        .expect("notification durable insérée");
}

fn complete_client_handshake(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
) {
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
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().expect("clone socket")),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("ligne client");
    serde_json::from_str(&line).expect("JSON client")
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).expect("écriture serveur");
    writer.flush().expect("flush serveur");
}

fn write_json_if_open(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) -> bool {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).is_ok() && writer.flush().is_ok()
}

fn write_accepted(writer: &mut BufWriter<UnixStream>, message_id: Uuid) {
    write_json(
        writer,
        json!({
            "type":"IdempotencyResult",
            "operation_kind":"send",
            "idempotency_key":message_id.to_string(),
            "issue":{"kind":"accepted","expires_at":1_787_600_000_i64}
        }),
    );
}

fn spawn_crash_child(mode: &str, fixture: &Fixture) -> (Child, UnixStream, PathBuf) {
    let barrier_path = fixture.root.join(format!("{mode}.barrier.sock"));
    let listener = UnixListener::bind(&barrier_path).expect("barrière parent");
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        sender
            .send(listener.accept().map(|pair| pair.0))
            .expect("barrière connectée");
    });
    let mut child = Command::new(std::env::current_exe().expect("binaire test"))
        .arg("--exact")
        .arg("coordination_dispatch::notification_crash_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_DATABASE, &fixture.database_path)
        .env(CHILD_SOCKET, &fixture.socket)
        .env(CHILD_BARRIER, &barrier_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("enfant crash lancé");
    let barrier = match receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(stream)) => stream,
        other => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("barrière enfant absente : {other:?}");
        }
    };
    (child, barrier, barrier_path)
}

fn wait_barrier(stream: &mut UnixStream, phase: &str) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout barrière");
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .expect("ligne barrière");
    assert_eq!(line.trim(), format!("BARRIER:{phase}"));
}

fn block_at_barrier(path: &PathBuf, phase: &str) {
    let mut stream = UnixStream::connect(path).expect("barrière enfant jointe");
    writeln!(stream, "BARRIER:{phase}").expect("jalon enfant écrit");
    stream.flush().expect("jalon enfant flush");
    let mut release = [0_u8; 1];
    let _ = stream.read(&mut release);
}

struct Fixture {
    root: PathBuf,
    database_path: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let _ = label;
        let root = std::env::temp_dir().join(format!("md-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&root).expect("répertoire fixture");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("permissions fixture");
        Self {
            database_path: root.join("maicie.sqlite"),
            socket: root.join("bridget.sock"),
            root,
        }
    }

    fn bind(&self) -> UnixListener {
        UnixListener::bind(&self.socket).expect("socket fixture")
    }

    fn socket_path(&self) -> &PathBuf {
        &self.socket
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
