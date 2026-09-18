//! Consommateur externe par le protocole public, sans dépendance à le service compagnon.
//! Les six scénarios 015 conservés, replay brut et droits par connexion.
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    ConnectionRole, GuichetDurationClass, GuichetOutcome, GuichetReplyPayload,
    SERVICE_CONTRACT_VERSION, ServiceCapability, ServiceRequestOperation, ServiceRequestPayload,
    ServiceSuiteDeclaration, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SCOPE: &str = "015_scope_0123456789abcdef0123456789abcdef";
const SERVICE_SCOPE: &str = "015_service_abcdef0123456789abcdef0123456789";

const PRODUCER: &str = "89000000-0000-4000-8000-000000000001";
const COORDINATOR: &str = "89000000-0000-4000-8000-000000000002";
const TARGET: &str = "89000000-0000-4000-8000-000000000003";

fn isolated_bridget_command(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", home)
        .env("TMPDIR", home.join("tmp"))
        .env("BRIDGET_HOME", home.join("state"))
        .env("BRIDGET_SOCKET", socket(home))
        .env("HOSTNAME", "guichet-fixture")
        .env(
            "BRIDGET_GREFFE_POLICY_PATH",
            home.join("greffe-authorization.json"),
        )
        .env("BRIDGET_GREFFE_AUDIT_PATH", home.join("greffe-audit.jsonl"));
    command
}

fn unique_home() -> PathBuf {
    // Racine courte, canonique, privée et sans collision entre tests parallèles.
    let home = std::fs::canonicalize("/tmp")
        .unwrap()
        .join(format!("b89g-{}", uuid::Uuid::new_v4().simple()));
    for path in [&home, &home.join("tmp"), &home.join("state")] {
        std::fs::DirBuilder::new().mode(0o700).create(path).unwrap();
    }
    home
}

fn socket(home: &Path) -> PathBuf {
    home.join("state/bridget.sock")
}

fn write_greffe_policy(home: &Path, principal: &str, instance_id: &str, expires_at: i64) {
    let path = home.join("greffe-authorization.json");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(
        &serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "generation": 1,
            "attestation_key": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "principals": [{
                "principal": principal,
                "actions": ["delegate"],
                "instances": [{
                    "instance_id": instance_id,
                    "expires_at": expires_at,
                    "revoked": false
                }]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
}

fn git(repo: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
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
            isolated_bridget_command(home)
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
            assert!(is_owned_daemon(&child), "PID extérieur au harnais");
            child.kill().expect("SIGKILL du seul daemon enfant du test");
            child.wait().expect("récolte du daemon tué");
        }
    }
}

fn is_owned_daemon(child: &Child) -> bool {
    let output = Command::new("/bin/ps")
        .args(["-p", &child.id().to_string(), "-o", "command="])
        .output();
    output.is_ok_and(|output| {
        output.status.success()
            && String::from_utf8_lossy(&output.stdout).trim()
                == format!("{} daemon", env!("CARGO_BIN_EXE_bridget"))
    })
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
    if !is_owned_daemon(child) {
        return false;
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
    if !is_owned_daemon(child) {
        return false;
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
    service_with_capabilities(home, issuer_scope, vec![ServiceCapability::GuichetV1])
}

fn service_with_capabilities(
    home: &Path,
    issuer_scope: &str,
    capabilities: Vec<ServiceCapability>,
) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
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
                service: "guichet".to_string(),
                issuer_scope: issuer_scope.to_string(),
                capabilities,
            },
        ),
        DaemonToWrapper::ServiceWelcome { .. }
    ));
    (reader, writer)
}

#[test]
fn une_connexion_suivante_ne_recoit_ni_claim_ni_lookup_ni_reply_sans_capacite() {
    let home = unique_home();
    let daemon = DaemonGuard::start(&home);
    let (mut capable_reader, mut capable_writer) = service(&home, SERVICE_SCOPE);
    assert!(matches!(
        request(
            &mut capable_reader,
            &mut capable_writer,
            WrapperToDaemon::GuichetClaimNext {
                version: SERVICE_CONTRACT_VERSION
            }
        ),
        DaemonToWrapper::GuichetEmpty { .. }
    ));
    drop(capable_reader);
    drop(capable_writer);
    let (mut reader, mut writer) = service_with_capabilities(&home, SERVICE_SCOPE, vec![]);
    for frame in [
        WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION,
        },
        WrapperToDaemon::GuichetLookup {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: SCOPE.into(),
            request_id: "gate-request-1".into(),
        },
        WrapperToDaemon::GuichetClaim {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: SCOPE.into(),
            request_id: "gate-request-1".into(),
            claim_token: "a".repeat(22),
        },
        reply(1, "a".repeat(22), "forged"),
    ] {
        // Mutation : une capacité globale/reprise de la précédente connexion
        // rendrait une issue métier (Empty/Expired), au lieu du refus en amont.
        assert!(matches!(
            request(&mut reader, &mut writer, frame),
            DaemonToWrapper::ServiceRejected {
                reason: bridget_transport::protocol::ServiceRefusal::CapabilityRequired
            }
        ));
    }
    let db = rusqlite::Connection::open(home.join("state/bridget.db")).unwrap();
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM guichet_requests", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
    drop(reader);
    drop(writer);
    daemon.kill();
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn delegate_est_admis_comme_depot_sans_etre_confondu_avec_un_succes_metier() {
    let home = unique_home();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    write_greffe_policy(&home, PRODUCER, "delegate-admission-instance", now + 300);
    let daemon = DaemonGuard::start(&home);
    let (mut wrapper_reader, mut wrapper_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut wrapper_reader,
            &mut wrapper_writer,
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                identity_version: 2,
                agent_id: PRODUCER.to_string(),
                host: None,
                transport: Some("unix".to_string()),
                channel: Some("unix".to_string()).into(),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("delegate-admission-instance".to_string()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let prerequisite = "51000000-0000-4000-8000-000000000001".to_string();
    let deposit = WrapperToDaemon::ServiceRequest {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: SCOPE.to_string(),
        request_id: "delegate-admission-request".to_string(),
        issued_at: now,
        from: PRODUCER.to_string(),
        to: "guichet".to_string(),
        operation: ServiceRequestOperation::Delegate,
        payload: ServiceRequestPayload::Delegate {
            goal: format!("lot dépendant de {prerequisite}"),
            review_target: None,
            explicit_target: Some(TARGET.to_string()),
            required_tags: Vec::new(),
            duration: GuichetDurationClass::Normale,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: vec![prerequisite],
            references: Vec::new(),
            origin: None,
            focus: None,
        },
    };
    assert!(matches!(
        request(&mut wrapper_reader, &mut wrapper_writer, deposit),
        DaemonToWrapper::GuichetResult { ref issue, .. } if issue == "queued"
    ));

    let (mut service_reader, mut service_writer) = service(&home, SERVICE_SCOPE);
    let claimed = request(
        &mut service_reader,
        &mut service_writer,
        WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION,
        },
    );
    let canonical_request = match claimed {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            canonical_request,
            ..
        } => {
            assert_eq!(request_id, "delegate-admission-request");
            canonical_request
        }
        other => panic!("claim delegate attendu, reçu {other:?}"),
    };
    let canonical_request = String::from_utf8(canonical_request).unwrap();
    assert!(matches!(
        decode::<WrapperToDaemon>(&canonical_request).unwrap(),
        WrapperToDaemon::ServiceRequest {
            operation: ServiceRequestOperation::Delegate,
            payload: ServiceRequestPayload::Delegate {
                suite: ServiceSuiteDeclaration::Aucune,
                ref depends_on,
                ..
            },
            ..
        } if depends_on == &["51000000-0000-4000-8000-000000000001"]
    ));

    drop(wrapper_reader);
    drop(wrapper_writer);
    drop(service_reader);
    drop(service_writer);
    drop(daemon);
    assert_daemon_count_for_home(&home, 0);
    let _ = std::fs::remove_dir_all(home);
}

#[test]
fn crash_reel_claim_rejoue_fifo_et_refuse_le_detenteur_perime() {
    let home = unique_home();
    let daemon = DaemonGuard::start(&home);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Un vrai producteur protocolaire dépose pendant l'absence du service compagnon.
    let (mut wrapper_reader, mut wrapper_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut wrapper_reader,
            &mut wrapper_writer,
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                identity_version: 2,
                agent_id: PRODUCER.to_string(),
                host: None,
                transport: Some("unix".to_string()),
                channel: None.into(),
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
        from: PRODUCER.to_string(),
        to: "guichet".to_string(),
        operation: ServiceRequestOperation::DeliveryReport,
        payload: ServiceRequestPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "0".repeat(64),
            in_reply_to: "message-1".to_string(),
            review_verdict: None,
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
    let (generation_a, token_a, bytes_a) = match claim_a {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            claim_generation,
            claim_token,
            canonical_request,
            ..
        } => {
            assert_eq!(request_id, "gate-request-1");
            (claim_generation, claim_token, canonical_request)
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
    let (generation_b, token_b, bytes_b) = match claim_b {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            claim_generation,
            claim_token,
            canonical_request,
            ..
        } => {
            assert_eq!(request_id, "gate-request-1");
            (claim_generation, claim_token, canonical_request)
        }
        other => panic!("claim B attendu après crash, reçu {other:?}"),
    };
    assert!(generation_b > generation_a);
    assert_ne!(token_b, token_a);
    // Un rejeu ne reconstruit pas une requête seulement équivalente : même
    // dépôt durable, mêmes octets malgré le remplacement de génération.
    assert_eq!(bytes_b, bytes_a);

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
            review_verdict: None,
        },
    }
}

/// Oracle binaire FR-2104 : le vrai `bridget guichet deposer` s'exécute dans
/// un dépôt dont HEAD possède un commit local non poussé. Le mandat et le
/// remote restent sur A ; la trame doit donc porter HEAD=B et remote=A. Copier
/// `expected_head` dans l'un des champs d'observation fait rougir ce test.
///
/// Ce témoin atteste seulement l'identité Git observée par la CLI. Il ne lit ni
/// propreté du worktree, ni target Cargo, ni dépendances, ni environnement de
/// test, et ne gèle pas l'URL associée au nom du remote local.
#[test]
fn depot_cli_reel_mesure_head_et_remote_au_lieu_de_copier_le_mandat() {
    let home = unique_home();
    std::fs::create_dir_all(&home).unwrap();
    let remote = home.join("remote.git");
    let repository = home.join("repo");
    git(&home, &["init", "--bare", remote.to_str().unwrap()]);
    git(&home, &["init", repository.to_str().unwrap()]);
    git(
        &repository,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    std::fs::write(repository.join("preuve.txt"), "A\n").unwrap();
    git(&repository, &["add", "preuve.txt"]);
    git(
        &repository,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-m",
            "fixture A",
        ],
    );
    let expected_head = git(&repository, &["rev-parse", "HEAD"]);
    git(
        &repository,
        &["push", "origin", "HEAD:refs/heads/fix/review"],
    );
    std::fs::write(repository.join("preuve.txt"), "B\n").unwrap();
    git(&repository, &["add", "preuve.txt"]);
    git(
        &repository,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-m",
            "fixture B locale",
        ],
    );
    let measured_head = git(&repository, &["rev-parse", "HEAD"]);
    assert_ne!(expected_head, measured_head, "le témoin exige deux têtes");

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
                identity_version: 2,
                agent_id: PRODUCER.to_string(),
                host: None,
                transport: Some("unix".to_string()),
                channel: Some("unix".to_string()).into(),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("codex-review-instance".to_string()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let (mut guichet_reader, mut guichet_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut guichet_reader,
            &mut guichet_writer,
            WrapperToDaemon::Register {
                agent_type: "guichet".to_string(),
                identity_version: 2,
                agent_id: COORDINATOR.to_string(),
                host: None,
                transport: Some("unix".to_string()),
                channel: Some("unix".to_string()).into(),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("guichet-review-instance".to_string()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let mut tracked = BridgetMessage::new(COORDINATOR, PRODUCER, "revue attendue");
    tracked.reply = true;
    tracked.reply_timeout = Some(60);
    assert!(matches!(
        request(
            &mut guichet_reader,
            &mut guichet_writer,
            WrapperToDaemon::Send(tracked.clone())
        ),
        DaemonToWrapper::Ack { .. }
    ));

    let output = isolated_bridget_command(&home)
        .args([
            "guichet",
            "deposer",
            "delivery-report",
            "--objective",
            "objective-review",
            "--delegation",
            "delegation-review",
            "--hash",
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            "--in-reply-to",
            &tracked.id,
            "--verdict",
            "approve",
            "--review-ref",
            "origin/fix/review",
            "--expected-head",
            &expected_head,
            "--id",
            "gate-cli-review",
            "--issued-at",
            &now.to_string(),
            "--issuer-scope",
            SCOPE,
        ])
        .current_dir(&repository)
        .env("HOME", &home)
        .env("BRIDGET_AGENT_ID", PRODUCER)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dépôt de revue CLI: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (mut service_reader, mut service_writer) = service(&home, SERVICE_SCOPE);
    let canonical_request = match request(
        &mut service_reader,
        &mut service_writer,
        WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION,
        },
    ) {
        DaemonToWrapper::GuichetClaimed {
            request_id,
            canonical_request,
            ..
        } if request_id == "gate-cli-review" => canonical_request,
        other => panic!("claim du verdict CLI attendu, reçu {other:?}"),
    };
    let canonical: serde_json::Value = serde_json::from_slice(&canonical_request).unwrap();
    let evidence = &canonical["payload"]["review_verdict"];
    assert_eq!(evidence["expected_head"], expected_head);
    assert_eq!(evidence["measured_head"], measured_head);
    assert_eq!(evidence["observed_target_head"], expected_head);
    assert_eq!(evidence["target_ref"], "origin/fix/review");

    daemon.kill();
    let _ = std::fs::remove_dir_all(home);
}

/// Les attendus restent littéraux : le banc ne dérive jamais le message du
/// prédicat qu'il exerce. Chaque cas lance le vrai binaire et doit mourir sur
/// les assertions de code retour et de première ligne stderr.
///
/// Le dixième message nommé, « aucun SHA », n'est volontairement pas simulé :
/// `git_stdout` applique `trim()` avant le parsing de `ls-remote`, donc une
/// ligne vide ou composée d'espaces prend toujours le chemin « aucune tête ».
/// C'est une branche de production inatteignable, pas un cas de test manquant.
#[test]
fn depot_cli_refuse_exactement_les_neuf_erreurs_git_atteignables() {
    struct FailureCase {
        name: &'static str,
        git_case: &'static str,
        review_args: &'static [&'static str],
        expected_error: &'static str,
    }

    const HEAD: &str = "1111111111111111111111111111111111111111";
    const UPPER_HEAD: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let cases = [
        FailureCase {
            name: "attestation_incomplete",
            git_case: "must_not_run",
            review_args: &["--verdict", "approve"],
            expected_error: "erreur: --verdict, --review-ref et --expected-head doivent être fournis ensemble",
        },
        FailureCase {
            name: "verdict_inconnu",
            git_case: "must_not_run",
            review_args: &[
                "--verdict",
                "inconnu",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: --verdict attend approve|approve_with_changes|amender|stop",
        },
        FailureCase {
            name: "reference_invalide",
            git_case: "must_not_run",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "./main",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: --review-ref attend <remote>/<branche> valide",
        },
        FailureCase {
            name: "sha_attendu_non_canonique",
            git_case: "must_not_run",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                UPPER_HEAD,
            ],
            expected_error: "erreur: --expected-head attend exactement 40 hexadécimaux minuscules",
        },
        FailureCase {
            name: "head_mesure_non_canonique",
            git_case: "head_noncanonical",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: git rev-parse n'a pas rendu un SHA-1 canonique",
        },
        FailureCase {
            name: "aucune_tete_distante",
            git_case: "remote_empty",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: git ls-remote n'a rendu aucune tête",
        },
        FailureCase {
            name: "tete_distante_ambigue",
            git_case: "remote_ambiguous",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: git ls-remote a rendu une tête ambiguë",
        },
        FailureCase {
            name: "reference_distante_absente",
            git_case: "remote_without_ref",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: git ls-remote n'a rendu aucune référence",
        },
        FailureCase {
            name: "observation_distante_non_canonique",
            git_case: "remote_noncanonical",
            review_args: &[
                "--verdict",
                "approve",
                "--review-ref",
                "origin/fix/review",
                "--expected-head",
                HEAD,
            ],
            expected_error: "erreur: git ls-remote a rendu une observation non canonique",
        },
    ];

    let root = unique_home();
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let fake_git = bin.join("git");
    std::fs::write(
        &fake_git,
        r#"#!/bin/sh
if [ "$BRIDGET_TEST_GIT_CASE" = "must_not_run" ]; then
    exit 99
fi
sha="1111111111111111111111111111111111111111"
case "$1" in
    remote)
        printf '%s\n' '/tmp/fake-review-remote'
        ;;
    rev-parse)
        if [ "$BRIDGET_TEST_GIT_CASE" = "head_noncanonical" ]; then
            printf '%s\n' 'pas-un-sha'
        else
            printf '%s\n' "$sha"
        fi
        ;;
    ls-remote)
        case "$BRIDGET_TEST_GIT_CASE" in
            remote_empty)
                ;;
            remote_ambiguous)
                printf '%s\t%s\n%s\t%s\n' "$sha" 'refs/heads/fix/review' "$sha" 'refs/heads/fix/review'
                ;;
            remote_without_ref)
                printf '%s\n' "$sha"
                ;;
            remote_noncanonical)
                printf '%s\t%s\n' "$sha" 'refs/heads/fix/autre'
                ;;
            *)
                printf '%s\t%s\n' "$sha" 'refs/heads/fix/review'
                ;;
        esac
        ;;
    *)
        exit 98
        ;;
esac
"#,
    )
    .unwrap();
    std::fs::set_permissions(&fake_git, std::fs::Permissions::from_mode(0o700)).unwrap();

    for case in cases {
        let mut args = vec![
            "guichet",
            "deposer",
            "delivery-report",
            "--from",
            PRODUCER,
            "--objective",
            "objective-review",
            "--delegation",
            "delegation-review",
            "--hash",
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            "--in-reply-to",
            "message-review",
        ];
        args.extend_from_slice(case.review_args);
        let output = isolated_bridget_command(&root)
            .args(args)
            .current_dir(&root)
            .env("HOME", &root)
            .env("PATH", &bin)
            .env("BRIDGET_AGENT_ID", PRODUCER)
            .env("BRIDGET_TEST_GIT_CASE", case.git_case)
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(2),
            "{} doit être un refus de parsing, stderr={stderr}",
            case.name
        );
        assert_eq!(
            stderr.lines().next(),
            Some(case.expected_error),
            "{} doit mourir dans l'assertion du message exact",
            case.name
        );
        assert_eq!(
            stderr.lines().nth(1),
            Some(
                "usage: bridget guichet deposer <delivery-report|mission-status|deadline-question|delegate|registre-add|objective-close> [options]"
            ),
            "{} doit atteindre le refus CLI attendu",
            case.name
        );
    }

    let _ = std::fs::remove_dir_all(root);
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
                identity_version: 2,
                agent_id: PRODUCER.to_string(),
                host: None,
                transport: Some("unix".to_string()),
                channel: None.into(),
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
    let (mut guichet_reader, mut guichet_writer) = connect(&home);
    assert!(matches!(
        request(
            &mut guichet_reader,
            &mut guichet_writer,
            WrapperToDaemon::Register {
                agent_type: "guichet".to_string(),
                identity_version: 2,
                agent_id: COORDINATOR.to_string(),
                host: None,
                transport: Some("unix".to_string()),
                channel: None.into(),
                mode: None,
                location: None,
                os: None,
                instance_id: Some("guichet-instance".to_string()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
            },
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let mut tracked = BridgetMessage::new(COORDINATOR, PRODUCER, "rapport attendu");
    tracked.reply = true;
    tracked.reply_timeout = Some(60);
    assert!(matches!(
        request(
            &mut guichet_reader,
            &mut guichet_writer,
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
    let output = isolated_bridget_command(&home)
        .args(&cli_args)
        .env("HOME", &home)
        .env("BRIDGET_AGENT_ID", PRODUCER)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dépôt CLI: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("DÉPÔT: queued"));
    let retry = isolated_bridget_command(&home)
        .args(&cli_args)
        .env("HOME", &home)
        .env("BRIDGET_AGENT_ID", PRODUCER)
        .output()
        .unwrap();
    assert!(retry.status.success());
    assert!(String::from_utf8_lossy(&retry.stdout).contains("DÉPÔT: outcome_unknown"));

    let (mut service_reader, mut service_writer) = service(&home, SERVICE_SCOPE);
    assert_ne!(
        SCOPE, SERVICE_SCOPE,
        "le scope de dépôt n'est pas la session service compagnon"
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
            review_verdict: None,
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
            &mut guichet_reader,
            &mut guichet_writer,
            WrapperToDaemon::ListRequests { sender: COORDINATOR.to_string(), limit: 10 },
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
    let database = rusqlite::Connection::open(home.join("state/bridget.db")).unwrap();
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
    drop(service_reader);
    drop(service_writer);
    drop(guichet_reader);
    drop(guichet_writer);
    drop(recipient_reader);
    drop(recipient_writer);
    let restarted = DaemonGuard::start(&home);
    let (mut replay_reader, replay_writer) = service(&home, SERVICE_SCOPE);
    let mut replay_line = String::new();
    replay_reader.read_line(&mut replay_line).unwrap();
    assert_eq!(
        replay_line.as_bytes(),
        lifecycle_line.as_bytes(),
        "mêmes octets et event_id après SIGKILL, pas une reconstruction équivalente"
    );
    drop(replay_reader);
    drop(replay_writer);
    restarted.kill();
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
