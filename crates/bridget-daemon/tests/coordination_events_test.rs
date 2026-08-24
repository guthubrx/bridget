use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    ConnectionRole, CoordinationEventKind, SERVICE_CONTRACT_VERSION, ServiceCapability, decode,
    encode,
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

fn start_daemon(home: &Path) -> Child {
    std::fs::create_dir_all(home).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("daemon")
        .env("HOME", home)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while UnixStream::connect(socket(home)).is_err() {
        assert!(Instant::now() < deadline, "daemon de coordination non démarré");
        std::thread::sleep(Duration::from_millis(10));
    }
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
                turn_in_progress: false,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
}

fn coordination_service(
    home: &Path,
) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
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
    assert!(matches!(next(&mut recipient_reader), DaemonToWrapper::Deliver(_)));

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
    assert!(matches!(next(&mut recipient_reader), DaemonToWrapper::Deliver(_)));
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
