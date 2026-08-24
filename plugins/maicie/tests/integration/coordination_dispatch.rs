use maicie::bridget_client::BridgetClientLimits;
use maicie::reconcile::{NotificationReconcileAction, reconcile_notification_startup_with_limits};
use maicie::store::MaicieStore;
use rusqlite::{Connection, params};
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

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

struct Fixture {
    root: PathBuf,
    database_path: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("md-{label}-{}-{sequence}", std::process::id()));
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
