use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    ConnectionRole, GuichetOutcome, GuichetReplyPayload, SERVICE_CONTRACT_VERSION,
    ServiceCapability, ServiceRequestOperation, ServiceRequestPayload, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SCOPE: &str = "015_scope_0123456789abcdef0123456789abcdef";
const SERVICE_SCOPE: &str = "015_service_abcdef0123456789abcdef0123456789";

// Le daemon et ses wrappers propagent ces variables à leurs descendants. Un
// harnais qui remplace seulement HOME hériterait sinon l'identité de l'agent
// qui lance cargo, notamment son fichier de nom absolu.
const INHERITED_BRIDGET_ENV: &[&str] = &[
    "BRIDGET_AGENT_NAME",
    "BRIDGET_AGENT_NAME_FILE",
    "BRIDGET_AGENT_INSTANCE_ID",
    "BRIDGET_MANAGED_STATUS_FD",
    "BRIDGET_MANAGED_INSTANCE_ID",
    "BRIDGET_MANAGED_COMMAND_ID",
    "BRIDGET_MANAGED_GENERATION",
    "BRIDGET_TRANSPORT",
];

fn isolated_bridget_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    for variable in INHERITED_BRIDGET_ENV {
        command.env_remove(variable);
    }
    command
}

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

/// Possède le daemon guichet. Créée avant le spawn : panique d'amorçage ou
/// injectée ne laisse pas l'enfant sous PID 1. `Drop` ne panique jamais.
struct DaemonGuard {
    child: Option<Child>,
}

impl DaemonGuard {
    fn start(home: &Path) -> Self {
        let mut guard = Self { child: None };
        std::fs::create_dir_all(home).unwrap();
        guard.child = Some(
            isolated_bridget_command()
                .arg("daemon")
                .env("HOME", home)
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(3);
        while UnixStream::connect(socket(home)).is_err() {
            assert!(Instant::now() < deadline, "daemon guichet non démarré");
            std::thread::sleep(Duration::from_millis(10));
        }
        guard
    }

    /// Crash réel : ce n'est pas le chemin SIGTERM coopératif du daemon.
    fn kill(mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = stop_daemon_child_best_effort(self.child.as_mut());
    }
}

fn stop_daemon_child_best_effort(child: Option<&mut Child>) -> bool {
    let Some(child) = child else {
        return true;
    };
    match child.try_wait() {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => {}
    }
    let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(_) => break,
        }
    }
    let _ = child.kill();
    child.wait().is_ok()
}

/// Compte les daemons dont le HOME est exactement celui de CE test.
/// Un compteur borné au PID du harnais croise les voisins parallèles.
fn daemon_count_for_home(home: &Path) -> usize {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()
        .expect("ps pour l'oracle de non-fuite");
    assert!(output.status.success(), "ps indisponible pour l'oracle");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (pid, command) = line.split_once(char::is_whitespace)?;
            let command = command.trim_start();
            let mut parts = command.split_whitespace();
            let binary = parts.next()?;
            let argv1 = parts.next()?;
            if !binary.contains("bridget") || argv1 != "daemon" || parts.next().is_some() {
                return None;
            }
            Some(pid.trim())
        })
        .filter(|pid| daemon_home_is(pid, home))
        .count()
}

fn daemon_home_is(pid: &str, home: &Path) -> bool {
    let output = Command::new("lsof").args(["-p", pid, "-Fn"]).output();
    let Ok(output) = output else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix('n'))
        .any(|path| path_is_under_home(path, home))
}

fn path_is_under_home(path: &str, home: &Path) -> bool {
    let home = home.to_string_lossy();
    path == home.as_ref() || path.starts_with(&format!("{home}/"))
}

fn assert_daemon_count_for_home(home: &Path, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if daemon_count_for_home(home) == expected {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        daemon_count_for_home(home),
        expected,
        "daemon orphelin pour le HOME du test: {}",
        home.display()
    );
}

fn connect(home: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let stream = UnixStream::connect(socket(home)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
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
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    decode(line.trim_end()).unwrap()
}

fn service(home: &Path, issuer_scope: &str) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let (mut reader, mut writer) = connect(home);
    assert!(matches!(
        request(
            &mut reader,
            &mut writer,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service
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
                issuer_scope: issuer_scope.to_string(),
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
    let daemon = DaemonGuard::start(&home);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

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
                journal_available: None,
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

    let (mut reader_a, mut writer_a) = service(&home, SERVICE_SCOPE);
    let claim_a = request(
        &mut reader_a,
        &mut writer_a,
        WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION,
        },
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
    daemon.kill();
    drop(reader_a);
    drop(writer_a);

    let restarted = DaemonGuard::start(&home);
    // A survit côté client au crash du daemon, puis se reconnecte : son ancien
    // token doit rester sans droit quand B obtient une génération neuve.
    let (mut reader_a_after_crash, mut writer_a_after_crash) = service(&home, SERVICE_SCOPE);
    let (mut reader_b, mut writer_b) = service(&home, SERVICE_SCOPE);
    let claim_b = request(
        &mut reader_b,
        &mut writer_b,
        WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION,
        },
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

    restarted.kill();
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

#[test]
fn depot_cli_reel_et_reponse_guichet_cloturent_une_demande_liee_une_seule_fois() {
    let home = unique_home();
    let daemon = DaemonGuard::start(&home);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let (mut recipient_reader, mut recipient_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut recipient_reader,
            &mut recipient_writer,
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                name: Some("codex-1".to_string()),
                host: None,
                transport: Some("unix".to_string()),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("codex-instance".to_string()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let (mut maicie_reader, mut maicie_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut maicie_reader,
            &mut maicie_writer,
            WrapperToDaemon::Register {
                agent_type: "maicie".to_string(),
                name: Some("maicie".to_string()),
                host: None,
                transport: Some("unix".to_string()),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("maicie-instance".to_string()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let mut tracked = BridgetMessage::new("maicie", "codex-1", "rapport attendu");
    tracked.reply = true;
    tracked.reply_timeout = Some(60);
    assert!(matches!(
        request(
            &mut maicie_reader,
            &mut maicie_writer,
            WrapperToDaemon::Send(tracked.clone())
        ),
        DaemonToWrapper::Ack { .. }
    ));

    // Le vrai binaire utilise son inscription CLI temporaire, mais le daemon
    // vérifie encore que le nom déclaré désigne le wrapper producteur actif.
    let issued_at = now.to_string();
    let cli_args = vec![
        "guichet",
        "deposer",
        "delivery-report",
        "--objective",
        "objective-1",
        "--delegation",
        "delegation-1",
        "--hash",
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        "--in-reply-to",
        &tracked.id,
        "--id",
        "gate-cli-deposit",
        "--issued-at",
        &issued_at,
        "--issuer-scope",
        SCOPE,
    ];
    let output = isolated_bridget_command()
        .args(&cli_args)
        .env("HOME", &home)
        .env("BRIDGET_AGENT_NAME", "codex-1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dépôt CLI: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("DÉPÔT: queued"));
    let retry = isolated_bridget_command()
        .args(&cli_args)
        .env("HOME", &home)
        .env("BRIDGET_AGENT_NAME", "codex-1")
        .output()
        .unwrap();
    assert!(retry.status.success());
    assert!(String::from_utf8_lossy(&retry.stdout).contains("DÉPÔT: outcome_unknown"));

    let (mut service_reader, mut service_writer) = service(&home, SERVICE_SCOPE);
    assert_ne!(
        SCOPE, SERVICE_SCOPE,
        "le scope de dépôt n'est pas la session Maicie"
    );
    // Mutation discriminante : rétablir la comparaison avec le scope négocié
    // du service refuse ce lookup, puis le claim et la réponse du dépôt tiers.
    assert!(matches!(
        request(
            &mut service_reader,
            &mut service_writer,
            WrapperToDaemon::GuichetLookup {
                version: SERVICE_CONTRACT_VERSION,
                issuer_scope: SCOPE.to_string(),
                request_id: "gate-cli-deposit".to_string(),
            },
        ),
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "outcome_unknown"
    ));
    let (generation, token) = match request(
        &mut service_reader,
        &mut service_writer,
        WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION,
        },
    ) {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            claim_generation,
            claim_token,
            ..
        } => {
            assert_eq!(request_id, "gate-cli-deposit");
            (claim_generation, claim_token)
        }
        other => panic!("claim du dépôt CLI attendu, reçu {other:?}"),
    };
    assert!(matches!(
        request(
            &mut service_reader,
            &mut service_writer,
            WrapperToDaemon::GuichetClaim {
                version: SERVICE_CONTRACT_VERSION,
                issuer_scope: SCOPE.to_string(),
                request_id: "gate-cli-deposit".to_string(),
                claim_token: token.clone(),
            },
        ),
        DaemonToWrapper::GuichetClaimed { request_id, .. } if request_id == "gate-cli-deposit"
    ));
    let accepted = WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: SCOPE.to_string(),
        request_id: "gate-cli-deposit".to_string(),
        claim_generation: generation,
        claim_token: token,
        response_message_id: "guichet-response-1".to_string(),
        in_reply_to: tracked.id.clone(),
        outcome: GuichetOutcome::Accepted,
        payload: GuichetReplyPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
                .to_string(),
        },
    };
    assert!(matches!(
        request(&mut service_reader, &mut service_writer, accepted.clone()),
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "accepted"
    ));
    let mut lifecycle_line = String::new();
    service_reader.read_line(&mut lifecycle_line).unwrap();
    assert!(matches!(
        decode(lifecycle_line.trim_end()).unwrap(),
        DaemonToWrapper::RequestLifecycleEvent {
            request_id,
            state: bridget_transport::protocol::GuichetLifecycleState::Answered,
            in_reply_to: Some(in_reply_to),
            response_message_id: Some(response_message_id),
            ..
        } if request_id == "gate-cli-deposit"
            && in_reply_to == tracked.id
            && response_message_id == "guichet-response-1"
    ));
    assert!(matches!(
        request(
            &mut maicie_reader,
            &mut maicie_writer,
            WrapperToDaemon::ListRequests { sender: "maicie".to_string(), limit: 10 },
        ),
        DaemonToWrapper::RequestList { requests }
            if requests.iter().any(|request| request.id == tracked.id && request.state == "answered")
    ));

    // Un retry de la transition durable renvoie la même issue sans créer un
    // second fait. La ligne est relevable à une reconnexion du vrai service,
    // ce qui est le contrat de consommation de GuichetClient.
    assert!(matches!(
        request(&mut service_reader, &mut service_writer, accepted),
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "accepted"
    ));
    let database = rusqlite::Connection::open(home.join(".cache/bridget/bridget.db")).unwrap();
    let event_count: i64 = database
        .query_row(
            "SELECT COUNT(*) FROM guichet_lifecycle_events
             WHERE issuer_scope = ?1 AND request_id = ?2",
            [SCOPE, "gate-cli-deposit"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        event_count, 1,
        "un retry ne duplique jamais l'événement durable"
    );

    // Mutation discriminante : retirer mark_answered_in_transaction du reply
    // laisse la demande ouverte malgré GuichetResult accepted.
    daemon.kill();
    let _ = std::fs::remove_dir_all(home);
}

/// La garde doit être active avant le spawn : une panique juste après le
/// démarrage ne laisse aucun daemon sous le HOME de CE test.
#[test]
fn daemon_guard_nettoie_apres_une_panique_injectee() {
    let home_slot = Mutex::new(None::<PathBuf>);
    let mid = std::sync::atomic::AtomicUsize::new(usize::MAX);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let home = unique_home();
        *home_slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(home.clone());
        // Compteur borné à CE HOME : un voisin parallèle ne peut pas le fausser.
        assert_eq!(
            daemon_count_for_home(&home),
            0,
            "le HOME du test doit être vide avant le spawn"
        );
        let _daemon = DaemonGuard::start(&home);
        // Premier temps : le spawn doit apparaître au compteur (sinon l'égalité
        // avant/après ne prouverait rien).
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen = daemon_count_for_home(&home);
        while seen != 1 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
            seen = daemon_count_for_home(&home);
        }
        mid.store(seen, std::sync::atomic::Ordering::SeqCst);
        panic!("échec injecté après le spawn : la garde doit nettoyer");
    }));
    assert!(
        failed.is_err(),
        "la branche d'échec doit réellement paniquer"
    );
    assert_eq!(
        mid.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "premier temps : le daemon spawné doit être compté"
    );
    let home = home_slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
        .expect("HOME du test créé avant la panique");
    // Second temps : après Drop (dépliage), plus aucun daemon sur CE HOME.
    assert_daemon_count_for_home(&home, 0);
    let _ = std::fs::remove_dir_all(home);
}
