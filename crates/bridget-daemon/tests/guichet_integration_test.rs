use bridget_transport::protocol::{
    ConnectionRole, GuichetOutcome, GuichetReplyPayload, ServiceCapability,
    ServiceRequestOperation, ServiceRequestPayload, SERVICE_CONTRACT_VERSION, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SCOPE: &str = "015_scope_0123456789abcdef0123456789abcdef";

fn unique_home() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Le socket Unix du daemon est sous HOME/.cache/bridget. Garder HOME
    // volontairement court rend ce test portable vis-à-vis de SUN_LEN.
    std::env::temp_dir().join(format!("bg-{}-{:x}", std::process::id(), nonce & 0xffff))
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
        assert!(Instant::now() < deadline, "daemon guichet non démarré");
        std::thread::sleep(Duration::from_millis(10));
    }
    child
}

fn connect(home: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let stream = UnixStream::connect(socket(home)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    (BufReader::new(stream.try_clone().unwrap()), BufWriter::new(stream))
}

fn request(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
    message: WrapperToDaemon,
) -> DaemonToWrapper {
    writeln!(writer, "{}", encode(&message).unwrap()).unwrap();
    writer.flush().unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    decode(line.trim_end()).unwrap()
}

fn service(
    home: &Path,
) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let (mut reader, mut writer) = connect(home);
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::RoleHandshake { role: ConnectionRole::Service },
        ),
        DaemonToWrapper::RoleAccepted { role: ConnectionRole::Service }
    ));
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                service: "maicie".to_string(),
                issuer_scope: SCOPE.to_string(),
                capabilities: vec![ServiceCapability::MaicieGuichet],
            },
        ),
        DaemonToWrapper::ServiceWelcome { .. }
    ));
    (reader, writer)
}

#[test]
fn crash_reel_claim_rejoue_fifo_et_refuse_le_detenteur_perime() {
    let home = unique_home();
    let mut daemon = start_daemon(&home);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // Un vrai producteur protocolaire dépose pendant l'absence de Maicie.
    let (mut wrapper_reader, mut wrapper_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut wrapper_reader,
            &mut wrapper_writer,
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                name: Some("codex-1".to_string()),
                host: None,
                transport: Some("unix".to_string()),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("gate-wrapper-instance".to_string()),
                domain: None,
                turn_in_progress: false,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let deposit = WrapperToDaemon::ServiceRequest {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: SCOPE.to_string(),
        request_id: "gate-request-1".to_string(),
        issued_at: now,
        from: "codex-1".to_string(),
        to: "maicie".to_string(),
        operation: ServiceRequestOperation::DeliveryReport,
        payload: ServiceRequestPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "0".repeat(64),
            in_reply_to: "message-1".to_string(),
        },
    };
    assert!(matches!(
        request(&mut wrapper_reader, &mut wrapper_writer, deposit),
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "queued"
    ));
    drop(wrapper_reader);
    drop(wrapper_writer);

    let (mut reader_a, mut writer_a) = service(&home);
    let claim_a = request(
        &mut reader_a,
        &mut writer_a,
        WrapperToDaemon::GuichetClaimNext { version: SERVICE_CONTRACT_VERSION },
    );
    let (generation_a, token_a) = match claim_a {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            claim_generation,
            claim_token,
            ..
        } => {
            assert_eq!(request_id, "gate-request-1");
            (claim_generation, claim_token)
        }
        other => panic!("claim A attendu, reçu {other:?}"),
    };

    // Crash réel : ce n'est pas le chemin SIGTERM coopératif du daemon.
    daemon.kill().unwrap();
    daemon.wait().unwrap();
    drop(reader_a);
    drop(writer_a);

    let mut restarted = start_daemon(&home);
    // A survit côté client au crash du daemon, puis se reconnecte : son ancien
    // token doit rester sans droit quand B obtient une génération neuve.
    let (mut reader_a_after_crash, mut writer_a_after_crash) = service(&home);
    let (mut reader_b, mut writer_b) = service(&home);
    let claim_b = request(
        &mut reader_b,
        &mut writer_b,
        WrapperToDaemon::GuichetClaimNext { version: SERVICE_CONTRACT_VERSION },
    );
    let (generation_b, token_b) = match claim_b {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            claim_generation,
            claim_token,
            ..
        } => {
            assert_eq!(request_id, "gate-request-1");
            (claim_generation, claim_token)
        }
        other => panic!("claim B attendu après crash, reçu {other:?}"),
    };
    assert!(generation_b > generation_a);
    assert_ne!(token_b, token_a);

    // Mutation discriminante : retirer l'un des prédicats lease/génération/
    // token/propriétaire ferait accepter cette réponse périmée au lieu de
    // laisser le détenteur B être le seul à finaliser le dépôt.
    let stale = request(
        &mut reader_a_after_crash,
        &mut writer_a_after_crash,
        reply(generation_a, token_a, "response-a"),
    );
    assert!(matches!(
        stale,
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "claim_stale"
    ));
    let accepted = request(
        &mut reader_b,
        &mut writer_b,
        reply(generation_b, token_b, "response-b"),
    );
    assert!(matches!(
        accepted,
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "accepted"
    ));

    restarted.kill().unwrap();
    restarted.wait().unwrap();
    let _ = std::fs::remove_dir_all(home);
}

fn reply(generation: u64, token: String, response_message_id: &str) -> WrapperToDaemon {
    WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: SCOPE.to_string(),
        request_id: "gate-request-1".to_string(),
        claim_generation: generation,
        claim_token: token,
        response_message_id: response_message_id.to_string(),
        in_reply_to: "message-1".to_string(),
        outcome: GuichetOutcome::Accepted,
        payload: GuichetReplyPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "0".repeat(64),
        },
    }
}
