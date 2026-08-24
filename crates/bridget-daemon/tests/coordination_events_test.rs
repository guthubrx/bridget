use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    decode, encode, ConnectionRole, CoordinationEventKind, ServiceCapability,
    COORDINATION_STREAM_VERSION, SERVICE_CONTRACT_VERSION,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SERVICE_SCOPE: &str = "016_service_abcdef0123456789abcdef0123456789";

fn unique_home() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("bc-{}-{:x}", std::process::id(), nonce & 0xffff))
}

fn socket(home: &Path) -> PathBuf {
    home.join(".cache/bridget/bridget.sock")
}

fn wait_for_coordination_schema(home: &Path) {
    let database = home.join(".cache/bridget/bridget.db");
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let schema_ready = rusqlite::Connection::open(&database)
            .and_then(|connection| {
                connection.query_row(
                    "SELECT EXISTS(
                         SELECT 1 FROM sqlite_master
                         WHERE type = 'table'
                           AND name = 'guichet_coordination_stream_state'
                     )",
                    [],
                    |row| row.get::<_, bool>(0),
                )
            })
            .unwrap_or(false);
        if schema_ready {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "schéma de coordination du daemon non initialisé"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn start_daemon(home: &Path) -> Child {
    std::fs::create_dir_all(home).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("daemon")
        .env("HOME", home)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while UnixStream::connect(socket(home)).is_err() {
        assert!(
            Instant::now() < deadline,
            "daemon de coordination non démarré"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // `listen(2)` précède l'ouverture du Store dans le daemon : une connexion
    // réussie ne prouve donc pas encore que les migrations sont terminées.
    wait_for_coordination_schema(home);
    child
}

fn start_daemon_with_sync(home: &Path, sync: &Path) -> Child {
    std::fs::create_dir_all(home).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("daemon")
        .env("HOME", home)
        .env("BRIDGET_TEST_SYNC_DIR", sync)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while UnixStream::connect(socket(home)).is_err() {
        assert!(
            Instant::now() < deadline,
            "daemon de coordination non démarré"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    wait_for_coordination_schema(home);
    child
}

fn connect(home: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let stream = UnixStream::connect(socket(home)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    (
        BufReader::new(stream.try_clone().unwrap()),
        BufWriter::new(stream),
    )
}

fn request(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
    message: WrapperToDaemon,
) -> DaemonToWrapper {
    writeln!(writer, "{}", encode(&message).unwrap()).unwrap();
    writer.flush().unwrap();
    next(reader)
}

fn next(reader: &mut BufReader<UnixStream>) -> DaemonToWrapper {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(!line.is_empty(), "réponse daemon attendue");
    decode(line.trim_end()).unwrap()
}

fn next_raw(reader: &mut BufReader<UnixStream>) -> String {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(!line.is_empty(), "réponse daemon attendue");
    line
}

fn register(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
    name: &str,
    instance_id: &str,
) {
    assert!(matches!(
        request(
            reader,
            writer,
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                name: Some(name.to_string()),
                host: None,
                transport: Some("unix".to_string()),
                mode: None,
                location: None,
                os: None,
                instance_id: Some(instance_id.to_string()),
                domain: None,
                journal_available: None,
                turn_in_progress: false,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
}

fn coordination_service(home: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let (mut reader, mut writer) = connect(home);
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service,
            },
        ),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Service
        }
    ));
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                service: "maicie".to_string(),
                issuer_scope: SERVICE_SCOPE.to_string(),
                capabilities: vec![
                    ServiceCapability::MaicieGuichet,
                    ServiceCapability::CoordinationEventsV1,
                ],
            },
        ),
        DaemonToWrapper::ServiceWelcome { capabilities, .. }
            if capabilities == vec![
                ServiceCapability::MaicieGuichet,
                ServiceCapability::CoordinationEventsV1,
            ]
    ));
    (reader, writer)
}

fn coordination_stream_service(home: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let (mut reader, mut writer) = connect(home);
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service,
            },
        ),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Service
        }
    ));
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                service: "maicie".to_string(),
                issuer_scope: SERVICE_SCOPE.to_string(),
                capabilities: vec![
                    ServiceCapability::MaicieGuichet,
                    ServiceCapability::CoordinationEventsV2,
                ],
            },
        ),
        DaemonToWrapper::ServiceWelcome { capabilities, .. }
            if capabilities == vec![
                ServiceCapability::MaicieGuichet,
                ServiceCapability::CoordinationEventsV2,
            ]
    ));
    (reader, writer)
}

fn subscribe_coordination(writer: &mut BufWriter<UnixStream>, after_cursor: Option<u64>) {
    writeln!(
        writer,
        "{}",
        encode(&WrapperToDaemon::CoordinationSubscribe {
            version: COORDINATION_STREAM_VERSION,
            after_cursor,
        })
        .unwrap()
    )
    .unwrap();
    writer.flush().unwrap();
}

fn make_fifo(path: &Path) {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
}

fn wait_marker(marker: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !marker.exists() {
        assert!(
            Instant::now() < deadline,
            "jalon de crash absent: {marker:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn kill_sigkill(child: &mut Child) {
    assert_eq!(
        unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGKILL) },
        0,
        "SIGKILL réel du daemon"
    );
    child.wait().unwrap();
}

#[test]
fn reminder_sent_est_atteste_apres_ecriture_et_releve_au_meme_event_id() {
    let home = unique_home();
    let mut daemon = start_daemon(&home);
    let (mut sender_reader, mut sender_writer) = connect(&home);
    let (mut recipient_reader, mut recipient_writer) = connect(&home);
    register(
        &mut sender_reader,
        &mut sender_writer,
        "maicie",
        "coordination-sender",
    );
    register(
        &mut recipient_reader,
        &mut recipient_writer,
        "codex-1",
        "coordination-recipient",
    );
    let (mut service_reader, service_writer) = coordination_service(&home);

    let mut tracked = BridgetMessage::new("maicie", "codex-1", "rapport attendu");
    tracked.reply = true;
    tracked.reply_timeout = Some(3);
    let request_id = tracked.id.clone();
    assert!(matches!(
        request(
            &mut sender_reader,
            &mut sender_writer,
            WrapperToDaemon::Send(tracked),
        ),
        DaemonToWrapper::Ack { .. }
    ));
    assert!(matches!(
        next(&mut recipient_reader),
        DaemonToWrapper::Deliver(_)
    ));

    // Mutation discriminante : si le fait était créé avant l'écriture/flush du
    // rappel, cette corrélation pourrait être observée sans `Deliver` réel.
    let first = next(&mut service_reader);
    let (event_id, reminder_message_id, observed_at) = match first {
        DaemonToWrapper::CoordinationEvent {
            version,
            event_id,
            request_id: event_request_id,
            kind: CoordinationEventKind::ReminderSent,
            reminder_message_id,
            recipient,
            generation,
            observed_at,
            cursor: _,
        } => {
            assert_eq!(version, 1);
            assert_eq!(event_request_id, request_id);
            assert_eq!(recipient, "codex-1");
            assert_eq!(generation, 1);
            assert!(observed_at > 0, "l'instant est attesté par Bridget");
            (event_id, reminder_message_id, observed_at)
        }
        other => panic!("coordination_event attendu, reçu {other:?}"),
    };
    let reminder = next(&mut recipient_reader);
    assert!(matches!(
        reminder,
        DaemonToWrapper::Deliver(message) if message.id == reminder_message_id
    ));

    drop(service_reader);
    drop(service_writer);
    let (mut replay_reader, _replay_writer) = coordination_service(&home);
    assert!(matches!(
        next(&mut replay_reader),
        DaemonToWrapper::CoordinationEvent {
            event_id: replay_event_id,
            request_id: replay_request_id,
            reminder_message_id: replay_message_id,
            observed_at: replay_observed_at,
            ..
        } if replay_event_id == event_id
            && replay_request_id == request_id
            && replay_message_id == reminder_message_id
            && replay_observed_at == observed_at
    ));

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn fait_de_coordination_inconnu_est_refuse_avant_toute_negociation() {
    let home = unique_home();
    let mut daemon = start_daemon(&home);
    let (mut reader, mut writer) = connect(&home);
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service,
            },
        ),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Service
        }
    ));
    writeln!(
        writer,
        "{{\"type\":\"coordination_event\",\"v\":1,\"event_id\":\"evt-1\",\"request_id\":\"req-1\",\"kind\":\"future_fact\",\"reminder_message_id\":\"msg-1\",\"recipient\":\"codex-1\",\"generation\":1,\"observed_at\":1}}"
    )
    .unwrap();
    writer.flush().unwrap();
    // Mutation discriminante : retirer `coordination_event` de la garde JSON
    // rendrait ce producteur inconnu silencieux au lieu du refus fermé.
    assert!(matches!(
        next(&mut reader),
        DaemonToWrapper::ServiceRejected {
            reason: bridget_transport::protocol::ServiceRefusal::InvalidEnvelope
        }
    ));
    writeln!(
        writer,
        "{{\"type\":\"coordination_event\",\"v\":1,\"event_id\":\"evt-2\",\"request_id\":\"req-1\",\"kind\":\"reminder_sent\",\"reminder_message_id\":\"msg-1\",\"generation\":1,\"observed_at\":1}}"
    )
    .unwrap();
    writer.flush().unwrap();
    assert!(matches!(
        next(&mut reader),
        DaemonToWrapper::ServiceRejected {
            reason: bridget_transport::protocol::ServiceRefusal::InvalidEnvelope
        }
    ));
    writeln!(
        writer,
        "{{\"type\":\"ServiceHello\",\"version\":1,\"service\":\"maicie\",\"issuer_scope\":\"016_service_abcdef0123456789abcdef0123456789\",\"capabilities\":[\"maicie_guichet\",\"coordination_events_v1\"],\"future\":true}}"
    )
    .unwrap();
    writer.flush().unwrap();
    assert!(matches!(
        next(&mut reader),
        DaemonToWrapper::ServiceRejected {
            reason: bridget_transport::protocol::ServiceRefusal::CanonicalBytesMismatch
        }
    ));
    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn texte_relance_ne_fabrique_jamais_un_fait_de_coordination() {
    let home = unique_home();
    let mut daemon = start_daemon(&home);
    let (mut sender_reader, mut sender_writer) = connect(&home);
    let (mut recipient_reader, mut recipient_writer) = connect(&home);
    register(
        &mut sender_reader,
        &mut sender_writer,
        "maicie",
        "coordination-text-sender",
    );
    register(
        &mut recipient_reader,
        &mut recipient_writer,
        "codex-1",
        "coordination-text-recipient",
    );
    let (mut service_reader, _service_writer) = coordination_service(&home);

    assert!(matches!(
        request(
            &mut sender_reader,
            &mut sender_writer,
            WrapperToDaemon::Send(BridgetMessage::new(
                "maicie",
                "codex-1",
                "relance : ce texte ordinaire ne constitue pas un fait",
            )),
        ),
        DaemonToWrapper::Ack { .. }
    ));
    assert!(matches!(
        next(&mut recipient_reader),
        DaemonToWrapper::Deliver(_)
    ));
    service_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let mut line = String::new();
    assert!(
        service_reader.read_line(&mut line).is_err(),
        "un texte ne doit pas créer de coordination_event"
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}

// Les jalons `before/after_coordination_persist` ne vivent que derrière
// `feature = "test-support"`. Sans elle le binaire est muet et le banc
// attendait un `.ready` qui n'arrive jamais (« jalon de crash absent ») —
// rouge permanent sur `cargo test --workspace` nu. Skip honnête nommé ;
// la couverture crash reste exercée avec `--features test-support`
// (specs/012, gates crash, `cargo test -p bridget-daemon --features test-support`).
#[test]
#[cfg_attr(
    not(feature = "test-support"),
    ignore = "exige --features test-support"
)]
fn reprise_cursee_survit_aux_crashs_reels_et_conserve_les_octets() {
    let home = unique_home();
    let sync = home.join("sync");
    std::fs::create_dir_all(&sync).unwrap();

    // Frontière avant persistance : le SIGKILL au jalon prouve qu'aucun fait
    // n'est inventé après redémarrage. Ce n'est pas un arrêt coopératif.
    let before_fifo = sync.join("before_coordination_persist.fifo");
    make_fifo(&before_fifo);
    let before_marker = sync.join("before_coordination_persist.ready");
    let mut daemon = start_daemon_with_sync(&home, &sync);
    let (mut sender_reader, mut sender_writer) = connect(&home);
    let (mut recipient_reader, mut recipient_writer) = connect(&home);
    register(
        &mut sender_reader,
        &mut sender_writer,
        "maicie",
        "cursor-sender",
    );
    register(
        &mut recipient_reader,
        &mut recipient_writer,
        "codex-1",
        "cursor-recipient",
    );
    let mut tracked = BridgetMessage::new("maicie", "codex-1", "rappel cursé");
    tracked.reply = true;
    tracked.reply_timeout = Some(3);
    assert!(matches!(
        request(
            &mut sender_reader,
            &mut sender_writer,
            WrapperToDaemon::Send(tracked),
        ),
        DaemonToWrapper::Ack { .. }
    ));
    assert!(matches!(
        next(&mut recipient_reader),
        DaemonToWrapper::Deliver(_)
    ));
    wait_marker(&before_marker);
    kill_sigkill(&mut daemon);
    let database = home.join(".cache/bridget/bridget.db");
    let connection = rusqlite::Connection::open(&database).unwrap();
    let before_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_coordination_events",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        before_count, 0,
        "SIGKILL avant persist ne laisse aucun fait"
    );
    drop(connection);

    // Nouvelle demande et crash après la transaction : son événement doit être
    // relu identiquement, y compris son curseur, après deux redémarrages.
    std::fs::remove_file(&before_fifo).unwrap();
    let after_fifo = sync.join("after_coordination_persist.fifo");
    make_fifo(&after_fifo);
    let after_marker = sync.join("after_coordination_persist.ready");
    let mut daemon = start_daemon_with_sync(&home, &sync);
    let (mut sender_reader, mut sender_writer) = connect(&home);
    let (mut recipient_reader, mut recipient_writer) = connect(&home);
    register(
        &mut sender_reader,
        &mut sender_writer,
        "maicie",
        "cursor-sender-restarted",
    );
    register(
        &mut recipient_reader,
        &mut recipient_writer,
        "codex-1",
        "cursor-recipient-restarted",
    );
    let mut tracked = BridgetMessage::new("maicie", "codex-1", "rappel durable cursé");
    tracked.reply = true;
    tracked.reply_timeout = Some(3);
    assert!(matches!(
        request(
            &mut sender_reader,
            &mut sender_writer,
            WrapperToDaemon::Send(tracked),
        ),
        DaemonToWrapper::Ack { .. }
    ));
    assert!(matches!(
        next(&mut recipient_reader),
        DaemonToWrapper::Deliver(_)
    ));
    wait_marker(&after_marker);
    kill_sigkill(&mut daemon);

    let mut daemon = start_daemon(&home);
    let (mut first_reader, mut first_writer) = coordination_stream_service(&home);
    subscribe_coordination(&mut first_writer, None);
    let first_bytes = next_raw(&mut first_reader);
    let first = decode::<DaemonToWrapper>(first_bytes.trim_end()).unwrap();
    let (event_id, cursor) = match first {
        DaemonToWrapper::CoordinationEvent {
            version,
            event_id,
            cursor: Some(cursor),
            ..
        } if version == COORDINATION_STREAM_VERSION => (event_id, cursor),
        other => panic!("coordination_event v2 attendu, reçu {other:?}"),
    };
    assert!(matches!(
        next(&mut first_reader),
        DaemonToWrapper::CoordinationSnapshotCaughtUp {
            version: COORDINATION_STREAM_VERSION,
            through_cursor: Some(through),
        } if through == cursor
    ));
    kill_sigkill(&mut daemon);

    let mut daemon = start_daemon(&home);
    let (mut replay_reader, mut replay_writer) = coordination_stream_service(&home);
    subscribe_coordination(&mut replay_writer, None);
    let replay_bytes = next_raw(&mut replay_reader);
    let replay = decode::<DaemonToWrapper>(replay_bytes.trim_end()).unwrap();
    assert!(matches!(
        replay,
        DaemonToWrapper::CoordinationEvent {
            event_id: replay_id,
            cursor: Some(replay_cursor),
            ..
        } if replay_id == event_id && replay_cursor == cursor
    ));
    assert_eq!(
        first_bytes, replay_bytes,
        "le redémarrage rejoue les mêmes octets, pas un équivalent re-sérialisé"
    );
    assert!(matches!(
        next(&mut replay_reader),
        DaemonToWrapper::CoordinationSnapshotCaughtUp {
            through_cursor: Some(through),
            ..
        } if through == cursor
    ));
    let (mut caught_reader, mut caught_writer) = coordination_stream_service(&home);
    subscribe_coordination(&mut caught_writer, Some(cursor));
    assert!(matches!(
        next(&mut caught_reader),
        DaemonToWrapper::CoordinationSnapshotCaughtUp {
            through_cursor: Some(through),
            ..
        } if through == cursor
    ));
    caught_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(150)))
        .unwrap();
    let mut extra = String::new();
    assert!(
        caught_reader.read_line(&mut extra).is_err(),
        "aucun doublon après curseur"
    );
    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn gap_et_unavailable_restant_des_observations_distinctes() {
    let home = unique_home();
    let mut daemon = start_daemon(&home);
    let database = home.join(".cache/bridget/bridget.db");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "INSERT INTO guichet_coordination_stream_state (singleton, high_watermark)
             VALUES (1, 1)
             ON CONFLICT(singleton) DO UPDATE SET high_watermark = 1",
            [],
        )
        .unwrap();
    drop(connection);

    let (mut gap_reader, mut gap_writer) = coordination_stream_service(&home);
    subscribe_coordination(&mut gap_writer, None);
    assert!(matches!(
        next(&mut gap_reader),
        DaemonToWrapper::CoordinationGap {
            version: COORDINATION_STREAM_VERSION,
            from_cursor: 1,
            to_cursor: 1,
            ..
        }
    ));

    // Mutation discriminante : remplacer `CoordinationGap` par Unavailable
    // ferait échouer l'assertion précédente. Les deux causes ne se confondent
    // donc pas sous le même état non frais.
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute("DROP TABLE guichet_coordination_events", [])
        .unwrap();
    drop(connection);
    let (mut unavailable_reader, mut unavailable_writer) = coordination_stream_service(&home);
    subscribe_coordination(&mut unavailable_writer, None);
    assert!(matches!(
        next(&mut unavailable_reader),
        DaemonToWrapper::CoordinationUnavailable {
            version: COORDINATION_STREAM_VERSION,
            reason,
        } if reason == "source_unavailable"
    ));
    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn releve_cursee_refuse_un_champ_futur_avant_toute_lecture() {
    let home = unique_home();
    let mut daemon = start_daemon(&home);
    let (mut reader, mut writer) = coordination_stream_service(&home);
    writeln!(
        writer,
        r#"{{"type":"coordination_subscribe","v":2,"after_cursor":0,"future":true}}"#
    )
    .unwrap();
    writer.flush().unwrap();
    assert!(matches!(
        next(&mut reader),
        DaemonToWrapper::ServiceRejected {
            reason: bridget_transport::protocol::ServiceRefusal::CanonicalBytesMismatch
        }
    ));
    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}
