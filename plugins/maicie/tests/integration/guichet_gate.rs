use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::bridget_client::{BridgetClient, BridgetClientLimits};
use maicie::config::DurationClasses;
use maicie::domain::ClasseDuree;
use maicie::reconcile::{
    GuichetReconcileAction, GuichetReconcilePhase, reconcile_guichet_startup_observed_with_limits,
    reconcile_guichet_startup_with_limits,
};
use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);
static G1504_PROCESS_GATE: OnceLock<Mutex<()>> = OnceLock::new();

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-guichet-gate-{label}-{}", Uuid::new_v4()))
}

fn durations() -> DurationClasses {
    DurationClasses {
        short_secs: 30,
        normal_secs: 60,
        long_secs: 90,
    }
}

fn seed(database: &std::path::Path) -> maicie::app::DelegationCreated {
    seed_for(database, "prospective")
}

fn seed_for(database: &std::path::Path, participant: &str) -> maicie::app::DelegationCreated {
    let mut store = MaicieStore::open(database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: participant.to_string(),
        tags: vec!["rust".to_string()],
        available: true,
        dnd: false,
    }];
    let DelegateResult::Created(created) = delegate(
        &mut store,
        durations(),
        "maicie",
        &candidates,
        &DelegateRequest {
            goal: "produire le rapport",
            explicit_target: Some(participant),
            required_tags: &[],
            duration: ClasseDuree::Normale,
            reply: true,
            idempotency_key: "guichet-gate-seed",
            now: 900,
            retry_until: 1_100,
            dedup_retained_until: 1_200,
            max_frame_bytes: 256 * 1024,
        },
    )
    .unwrap() else {
        panic!("délégation attendue")
    };
    created
}

#[test]
fn releve_pull_only_greffe_repond_et_enregistre_l_evenement() {
    let root = root("delivery");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let issuer_scope = MaicieStore::open(&database)
        .unwrap()
        .issuer_scope()
        .to_string();
    let fixture = SocketFixture::new("delivery");
    let listener = fixture.bind();
    let canonical_request = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"{issuer_scope}\",\"request_id\":\"request-gate-01\",\"issued_at\":1000,\"from\":\"prospective\",\"to\":\"maicie\",\"operation\":\"delivery_report\",\"payload\":{{\"objective_id\":\"{}\",\"delegation_id\":\"{}\",\"delivery_hash\":\"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",\"in_reply_to\":\"{}\"}}}}",
        created.objective_id, created.delegation_id, created.message_id
    )
    .into_bytes();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
        write_json(&mut writer, welcome());
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
        write_json(
            &mut writer,
            json!({
                "type":"guichet_claimed",
                "v":1,
                "issuer_scope":issuer_scope,
                "request_id":"request-gate-01",
                "canonical_request":base64(&canonical_request),
                "claimed_at":1000,
                "claim_generation":1,
                "claim_token":"Q2xhaW0tdG9rZW4tMTI4LWJpdHM",
                "claim_lease_expires_at":1100,
                "expires_at":1200
            }),
        );
        let reply = read_json(&mut reader);
        assert_eq!(reply["type"], "guichet_reply");
        assert_eq!(reply["request_id"], "request-gate-01");
        assert_eq!(reply["payload"]["kind"], "delivery_report");
        write_json(
            &mut writer,
            json!({
                "type":"guichet_result",
                "v":1,
                "issuer_scope":issuer_scope,
                "request_id":"request-gate-01",
                "issue":"accepted",
                "expires_at":1200
            }),
        );
        write_json(
            &mut writer,
            json!({
                "type":"request_lifecycle_event",
                "v":1,
                "issuer_scope":issuer_scope,
                "event_id":"event-gate-01",
                "request_id":"request-gate-01",
                "state":"answered",
                "observed_at":1010,
                "in_reply_to":created.message_id.to_string(),
                "response_message_id":reply["response_message_id"].as_str().unwrap()
            }),
        );
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
        write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
    });

    let mut store = MaicieStore::open(&database).unwrap();
    let report =
        reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_010, limits()).unwrap();
    assert!(report.actions.iter().any(|action| matches!(
        action,
        GuichetReconcileAction::ReponseAttestee { request_id, issue }
            if request_id == "request-gate-01" && issue == "accepted"
    )));
    assert!(report.actions.iter().any(|action| matches!(
        action,
        GuichetReconcileAction::EvenementAtteste { request_id, state }
            if request_id == "request-gate-01" && state == "answered"
    )));
    assert!(matches!(
        report.actions.last(),
        Some(GuichetReconcileAction::Vide)
    ));
    assert_eq!(
        store
            .objective_snapshots(Some(created.objective_id))
            .unwrap()[0]
            .decisions
            .len(),
        1
    );
    drop(store);
    server.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

/// Mutation discriminante : si `process_guichet_claim` recréait une décision
/// après la coupure, le second passage produirait deux décisions. Si le
/// réconciliateur reconstruisait lui-même la réponse, son `response_message_id`
/// ou son payload divergerait de ceux durablement reçus lors du premier claim.
#[test]
fn issue_perdue_puis_releve_regeneree_ne_double_ni_decision_ni_reponse() {
    let root = root("reply-lost");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let issuer_scope = MaicieStore::open(&database)
        .unwrap()
        .issuer_scope()
        .to_string();
    let fixture = SocketFixture::new("reply-lost");
    let listener = fixture.bind();
    let canonical_request = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"{issuer_scope}\",\"request_id\":\"request-reply-lost\",\"issued_at\":1000,\"from\":\"prospective\",\"to\":\"maicie\",\"operation\":\"delivery_report\",\"payload\":{{\"objective_id\":\"{}\",\"delegation_id\":\"{}\",\"delivery_hash\":\"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",\"in_reply_to\":\"{}\"}}}}",
        created.objective_id, created.delegation_id, created.message_id
    )
    .into_bytes();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
        write_json(&mut writer, welcome());
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
        write_json(
            &mut writer,
            claimed(&issuer_scope, "request-reply-lost", &canonical_request, 1),
        );
        let first_reply = read_json(&mut reader);
        let response_message_id = first_reply["response_message_id"]
            .as_str()
            .unwrap()
            .to_string();
        let first_payload = first_reply["payload"].clone();
        drop(writer);
        drop(reader);

        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
        write_json(&mut writer, welcome());
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
        write_json(
            &mut writer,
            claimed(&issuer_scope, "request-reply-lost", &canonical_request, 2),
        );
        let replay = read_json(&mut reader);
        assert_eq!(replay["claim_generation"], 2);
        assert_eq!(replay["response_message_id"], response_message_id);
        assert_eq!(replay["payload"], first_payload);
        write_json(
            &mut writer,
            json!({
                "type":"guichet_result",
                "v":1,
                "issuer_scope":issuer_scope,
                "request_id":"request-reply-lost",
                "issue":"accepted",
                "expires_at":1200
            }),
        );
        write_json(
            &mut writer,
            json!({
                "type":"request_lifecycle_event",
                "v":1,
                "issuer_scope":issuer_scope,
                "event_id":"event-reply-lost",
                "request_id":"request-reply-lost",
                "state":"answered",
                "observed_at":1011,
                "in_reply_to":created.message_id.to_string(),
                "response_message_id":response_message_id
            }),
        );
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
        write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
    });

    let mut store = MaicieStore::open(&database).unwrap();
    let first =
        reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_010, limits()).unwrap();
    assert!(matches!(
        first.actions.last(),
        Some(GuichetReconcileAction::TransportIncertain)
    ));
    assert_eq!(
        store
            .objective_snapshots(Some(created.objective_id))
            .unwrap()[0]
            .decisions
            .len(),
        1
    );

    let second =
        reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_011, limits()).unwrap();
    assert!(second.actions.iter().any(|action| matches!(
        action,
        GuichetReconcileAction::ReponseAttestee { request_id, issue }
            if request_id == "request-reply-lost" && issue == "accepted"
    )));
    assert!(second.actions.iter().any(|action| matches!(
        action,
        GuichetReconcileAction::EvenementAtteste { request_id, state }
            if request_id == "request-reply-lost" && state == "answered"
    )));
    assert_eq!(
        store
            .objective_snapshots(Some(created.objective_id))
            .unwrap()[0]
            .decisions
            .len(),
        1
    );
    drop(store);
    server.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

/// Mutation discriminante : si la lecture de `ClaimNext` cessait de recevoir
/// l'échéance absolue, le sommeil du serveur dépasserait la borne de cette
/// commande. Si elle greffait avant d'obtenir une issue, une décision
/// apparaîtrait malgré l'absence totale de réponse Bridget.
#[test]
fn budget_epuise_conserve_la_demande_sans_decision_locale() {
    let root = root("budget");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let issuer_scope = MaicieStore::open(&database)
        .unwrap()
        .issuer_scope()
        .to_string();
    let fixture = SocketFixture::new("budget");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
        write_json(&mut writer, welcome());
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
        thread::sleep(Duration::from_millis(120));
    });

    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_millis(30),
        io_timeout: Duration::from_millis(30),
        max_frame_bytes: 64 * 1024,
    };
    let started = Instant::now();
    let mut store = MaicieStore::open(&database).unwrap();
    let report =
        reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_010, limits).unwrap();
    assert!(started.elapsed() < Duration::from_millis(100));
    assert!(matches!(
        report.actions.last(),
        Some(GuichetReconcileAction::TransportIndisponible)
    ));
    assert!(
        store
            .objective_snapshots(Some(created.objective_id))
            .unwrap()[0]
            .decisions
            .is_empty()
    );
    drop(store);
    server.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

/// SC-1502 : les trois coupures frappent un vrai sous-processus Maicie. La
/// mutation qui déplacerait un jalon avant/après sa frontière ferait échouer
/// soit la barrière, soit l'assertion de décision/réponse unique au redémarrage.
#[test]
fn crash_reel_aux_trois_frontieres_releve_une_unique_decision() {
    for phase in [
        "before_claim",
        "after_claim_before_store_commit",
        "after_store_commit_before_reply",
    ] {
        let root = root(phase);
        let database = root.join("maicie.sqlite3");
        let marker = root.join("crash-barrier");
        let created = seed(&database);
        let issuer_scope = MaicieStore::open(&database)
            .unwrap()
            .issuer_scope()
            .to_string();
        let fixture = SocketFixture::new(phase);
        let listener = fixture.bind();
        let request_id = format!("request-crash-{phase}");
        let canonical_request = format!(
            "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"{issuer_scope}\",\"request_id\":\"{request_id}\",\"issued_at\":1000,\"from\":\"prospective\",\"to\":\"maicie\",\"operation\":\"delivery_report\",\"payload\":{{\"objective_id\":\"{}\",\"delegation_id\":\"{}\",\"delivery_hash\":\"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",\"in_reply_to\":\"{}\"}}}}",
            created.objective_id, created.delegation_id, created.message_id
        )
        .into_bytes();
        let server = thread::spawn({
            let phase = phase.to_string();
            let server_request_id = request_id.clone();
            move || {
                let (stream, _) = listener.accept().unwrap();
                let (mut reader, mut writer) = split(stream);
                assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
                write_json(&mut writer, welcome());
                if phase != "before_claim" {
                    assert_eq!(
                        read_json(&mut reader),
                        json!({"type":"guichet_claim_next","v":1})
                    );
                    write_json(
                        &mut writer,
                        claimed(&issuer_scope, &server_request_id, &canonical_request, 1),
                    );
                }
                drop(writer);
                drop(reader);

                let (stream, _) = listener.accept().unwrap();
                let (mut reader, mut writer) = split(stream);
                assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
                write_json(&mut writer, welcome());
                assert_eq!(
                    read_json(&mut reader),
                    json!({"type":"guichet_claim_next","v":1})
                );
                let generation = u64::from(phase != "before_claim") + 1;
                write_json(
                    &mut writer,
                    claimed(
                        &issuer_scope,
                        &server_request_id,
                        &canonical_request,
                        generation,
                    ),
                );
                let reply = read_json(&mut reader);
                assert_eq!(reply["type"], "guichet_reply");
                assert_eq!(reply["request_id"], server_request_id);
                assert_eq!(reply["claim_generation"], generation);
                let response_message_id = reply["response_message_id"].as_str().unwrap();
                write_json(
                    &mut writer,
                    json!({
                        "type":"guichet_result",
                        "v":1,
                        "issuer_scope":issuer_scope,
                        "request_id":server_request_id,
                        "issue":"accepted",
                        "expires_at":1200
                    }),
                );
                write_json(
                    &mut writer,
                    json!({
                        "type":"request_lifecycle_event",
                        "v":1,
                        "issuer_scope":issuer_scope,
                        "event_id":format!("event-{phase}"),
                        "request_id":server_request_id,
                        "state":"answered",
                        "observed_at":1010,
                        "in_reply_to":created.message_id.to_string(),
                        "response_message_id":response_message_id
                    }),
                );
                assert_eq!(
                    read_json(&mut reader),
                    json!({"type":"guichet_claim_next","v":1})
                );
                write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
            }
        });

        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("guichet_gate::crash_worker")
            .arg("--nocapture")
            .env("MAICIE_GUICHET_CRASH_DB", &database)
            .env("MAICIE_GUICHET_CRASH_SOCKET", fixture.path())
            .env("MAICIE_GUICHET_CRASH_PHASE", phase)
            .env("MAICIE_GUICHET_CRASH_MARKER", &marker)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !marker.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(marker.exists(), "barrière {phase} non atteinte");
        let process = Command::new("ps")
            .args(["-p", &child.id().to_string(), "-o", "command="])
            .output()
            .unwrap();
        let command = String::from_utf8_lossy(&process.stdout);
        assert!(command.contains("guichet_gate_integration"));
        assert!(!command.contains("Firefox"));
        unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
        child.wait().unwrap();

        let mut store = MaicieStore::open(&database).unwrap();
        let report =
            reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_011, limits())
                .unwrap();
        assert!(report.actions.iter().any(|action| matches!(
            action,
            GuichetReconcileAction::ReponseAttestee { request_id: actual, issue }
                if actual == &request_id && issue == "accepted"
        )));
        assert!(report.actions.iter().any(|action| matches!(
            action,
            GuichetReconcileAction::EvenementAtteste { request_id: actual, state }
                if actual == &request_id && state == "answered"
        )));
        assert_eq!(
            store
                .objective_snapshots(Some(created.objective_id))
                .unwrap()[0]
                .decisions
                .len(),
            1
        );
        drop(store);
        server.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn crash_worker() {
    let Some(database) = std::env::var_os("MAICIE_GUICHET_CRASH_DB") else {
        return;
    };
    let socket = std::env::var_os("MAICIE_GUICHET_CRASH_SOCKET").unwrap();
    let requested = std::env::var("MAICIE_GUICHET_CRASH_PHASE").unwrap();
    let marker = PathBuf::from(std::env::var_os("MAICIE_GUICHET_CRASH_MARKER").unwrap());
    let mut store = MaicieStore::open(database).unwrap();
    let _ = reconcile_guichet_startup_observed_with_limits(
        &mut store,
        PathBuf::from(socket),
        1_010,
        limits(),
        |observed| {
            let selected = matches!(
                (requested.as_str(), observed),
                ("before_claim", GuichetReconcilePhase::BeforeClaim)
                    | (
                        "after_claim_before_store_commit",
                        GuichetReconcilePhase::AfterClaimBeforeStoreCommit
                    )
                    | (
                        "after_store_commit_before_reply",
                        GuichetReconcilePhase::AfterStoreCommitBeforeReply
                    )
            );
            if selected {
                fs::write(&marker, b"ready").unwrap();
                let mut byte = [0_u8; 1];
                std::io::stdin().read_exact(&mut byte).unwrap();
            }
            Ok(())
        },
    );
}

fn limits() -> BridgetClientLimits {
    BridgetClientLimits {
        connect_timeout: Duration::from_millis(200),
        io_timeout: Duration::from_millis(200),
        max_frame_bytes: 64 * 1024,
    }
}

fn welcome() -> Value {
    json!({
        "type":"ServiceWelcome",
        "version":1,
        "horizon_secs":60,
        "issued_at_tolerance_secs":5,
        "capabilities":["maicie_guichet"]
    })
}

fn claimed(
    issuer_scope: &str,
    request_id: &str,
    canonical_request: &[u8],
    generation: u64,
) -> Value {
    json!({
        "type":"guichet_claimed",
        "v":1,
        "issuer_scope":issuer_scope,
        "request_id":request_id,
        "canonical_request":base64(canonical_request),
        "claimed_at":1000,
        "claim_generation":generation,
        "claim_token":format!("Q2xhaW0tdG9rZW4tMTI4LWJpdHM{generation}"),
        "claim_lease_expires_at":1100,
        "expires_at":1200
    })
}

fn assert_service_handshake(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
    issuer_scope: &str,
) {
    assert_eq!(
        read_json(reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(writer, json!({"type":"RoleAccepted","role":"service"}));
    assert_eq!(
        read_json(reader),
        json!({
            "type":"ServiceHello",
            "version":1,
            "service":"maicie",
            "issuer_scope":issuer_scope,
            "capabilities":["maicie_guichet"]
        })
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
    serde_json::from_str(line.trim_end()).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).unwrap();
    writer.flush().unwrap();
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            TABLE[((value >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            TABLE[(value & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    encoded
}

struct SocketFixture {
    path: PathBuf,
}

impl SocketFixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        Self {
            path: std::env::temp_dir()
                .join(format!("mg-{label}-{}-{sequence}.sock", std::process::id())),
        }
    }

    fn bind(&self) -> UnixListener {
        let _ = fs::remove_file(&self.path);
        UnixListener::bind(&self.path).unwrap()
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for SocketFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// G1504 : le dépôt provient du vrai binaire Bridget au nom d'un wrapper ACP
/// réellement lancé. Maicie n'existe entre le dépôt et la commande `status`
/// que comme SQLite privée : aucune boucle résidente ne peut absorber la lettre.
///
/// Exécution explicite :
/// `BRIDGET_MVP_GATE_BIN=/chemin/absolu/bridget cargo test -p maicie --test guichet_gate_integration -- --ignored --nocapture`.
#[test]
#[ignore = "gate G1504 réel : requiert BRIDGET_MVP_GATE_BIN vers le binaire Bridget du worktree"]
fn parcours_reel_g1504_releve_une_lettre_et_ne_la_duplique_pas() {
    let _serial = G1504_PROCESS_GATE
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    run_g1504(false);
}

/// La branche forcée traverse la même garde que le gate heureux. Elle verrouille
/// la fuite qui a auparavant laissé des managed-wrapper orphelins sous PID 1.
#[test]
#[ignore = "preuve G1504 réelle : requiert BRIDGET_MVP_GATE_BIN vers le binaire Bridget du worktree"]
fn g1504_nettoie_le_groupe_apres_un_echec_injecte() {
    let _serial = G1504_PROCESS_GATE
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    let before = managed_g1504_process_count();
    let failed = catch_unwind(AssertUnwindSafe(|| run_g1504(true)));
    assert!(
        failed.is_err(),
        "la branche d'échec doit réellement paniquer"
    );
    assert_managed_g1504_process_count(before);
}

fn run_g1504(force_failure_after_spawn: bool) {
    let before = managed_g1504_process_count();
    let bridget = PathBuf::from(
        std::env::var_os("BRIDGET_MVP_GATE_BIN")
            .expect("BRIDGET_MVP_GATE_BIN doit désigner le binaire Bridget réel"),
    );
    {
        let fixture = RealGateFixture::new(&bridget);
        let created = seed_for(&fixture.database, "g1504-agent");
        fixture.configure_agent(&created);
        let started = Instant::now();
        let mut run = RealGateRun::new(&fixture);
        run.start_daemon();
        fixture.wait_for_agent();
        if force_failure_after_spawn {
            panic!("échec G1504 injecté après le spawn : la garde doit nettoyer");
        }
        fixture.start_ephemeral_maicie();
        fixture.wait_for_deposit();
        let tracked_id = created.message_id.to_string();

        let deposit = fixture.deposit_args(&tracked_id);

        // La commande Maicie est le premier consommateur de la boîte aux lettres.
        // Son ouverture relève, greffe et répond exclusivement depuis les bytes
        // persistés, puis relève l'événement terminal Bridget sur la même session.
        let status = fixture.maicie(&[
            "status".to_string(),
            created.objective_id.to_string(),
            "--config".to_string(),
            fixture.config.display().to_string(),
            "--json".to_string(),
        ]);
        assert!(status.status.success(), "status G1504: {:?}", status.stderr);
        let status_json: Value = serde_json::from_slice(&status.stdout).unwrap();
        assert_eq!(status_json["coordination"].as_array().unwrap().len(), 1);

        fixture.assert_request_answered(&tracked_id);
        let connection = rusqlite::Connection::open(&fixture.database).unwrap();
        let decisions: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM coordination_decisions
             WHERE id IN (SELECT decision_id FROM guichet_receptions)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let lifecycle: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM guichet_lifecycle_events\n             WHERE issuer_scope = ?1 AND request_id = ?2 AND state = 'answered'",
                [
                    "015_scope_0123456789abcdef0123456789abcdef",
                    "g1504-depot-01",
                ],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(decisions, 1, "la greffe réelle ne crée qu'une décision");
        assert_eq!(
            lifecycle, 1,
            "l'événement answered est relevé une seule fois"
        );

        // Mutation discriminante : sans clé tripartite durable ou sans rejet du
        // rejeu terminal, le second dépôt recréerait un claim, une décision ou un
        // événement. Le daemon rejoue l'issue terminale `accepted` (la CLI la
        // présente comme issue non-queued et sort donc volontairement en erreur),
        // puis les compteurs durables prouvent l'absence de second effet.
        let replay = fixture.bridget(&deposit);
        assert!(
            !replay.status.success(),
            "un dépôt terminal ne doit pas redevenir queued: {:?}",
            replay.stdout
        );
        assert!(
            String::from_utf8_lossy(&replay.stdout).contains("DÉPÔT: accepted"),
            "rejeu dépôt: {:?}",
            replay.stdout
        );
        let repeated_status = fixture.maicie(&[
            "status".to_string(),
            created.objective_id.to_string(),
            "--config".to_string(),
            fixture.config.display().to_string(),
            "--json".to_string(),
        ]);
        assert!(
            repeated_status.status.success(),
            "status rejeu: {:?}",
            repeated_status.stderr
        );
        let repeated_decisions: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM coordination_decisions
             WHERE id IN (SELECT decision_id FROM guichet_receptions)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let repeated_lifecycle: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM guichet_lifecycle_events\n             WHERE issuer_scope = ?1 AND request_id = ?2",
                [
                    "015_scope_0123456789abcdef0123456789abcdef",
                    "g1504-depot-01",
                ],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(repeated_decisions, 1, "rejeu sans seconde décision");
        assert_eq!(repeated_lifecycle, 1, "rejeu sans second événement");
        fixture.assert_request_answered(&tracked_id);

        eprintln!(
            "G1504: dépôt absent→relève→greffe→answered={} ms",
            started.elapsed().as_millis()
        );
        run.finish();
    }
    assert_managed_g1504_process_count(before);
}

struct RealGateFixture {
    root: PathBuf,
    bridget: PathBuf,
    socket: PathBuf,
    config: PathBuf,
    database: PathBuf,
    release_deposit: PathBuf,
    deposit_sentinel: PathBuf,
    deposit_output: PathBuf,
    adapter_error: PathBuf,
    adapter_pgid: PathBuf,
}

/// Possède les processus créés par un run G1504. Il doit être construit avant
/// tout point qui peut paniquer : supprimer le répertoire temporaire ne suffit
/// pas à arrêter un wrapper déjà adopté par le daemon.
struct RealGateRun<'a> {
    fixture: &'a RealGateFixture,
    daemon: Option<Child>,
    finished: bool,
}

impl<'a> RealGateRun<'a> {
    fn new(fixture: &'a RealGateFixture) -> Self {
        Self {
            fixture,
            daemon: None,
            finished: false,
        }
    }

    fn start_daemon(&mut self) {
        self.daemon = Some(self.fixture.start_daemon());
    }

    fn cleanup(&mut self) -> bool {
        let agents_stopped = self.fixture.stop_agents_best_effort();
        let daemon_stopped = match self.daemon.as_mut() {
            Some(daemon) => stop_real_daemon_best_effort(daemon),
            None => true,
        };
        agents_stopped && daemon_stopped
    }

    fn finish(&mut self) {
        // Si cette assertion échoue, Drop refait une tentative avant de laisser
        // le test remonter son échec : aucun chemin de sortie ne saute le stop.
        assert!(self.cleanup(), "nettoyage G1504 incomplet");
        self.finished = true;
    }
}

impl Drop for RealGateRun<'_> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.cleanup();
        }
    }
}

impl RealGateFixture {
    fn new(bridget: &Path) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from("/tmp").join(format!("mg1504-{}-{sequence}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".config/bridget")).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let release_deposit = root.join("release-deposit");
        let deposit_sentinel = root.join("guichet-deposited");
        let deposit_output = root.join("guichet-deposit.out");
        let adapter_error = root.join("g1504-adapter.err");
        let adapter_pgid = root.join("adapter-pgid");
        let socket = root.join(".cache/bridget/bridget.sock");
        assert!(socket.as_os_str().len() < 104, "socket G1504 trop longue");
        let database = root.join("maicie.sqlite3");
        let config = root.join("maicie.json");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version":1,
                "bridget_socket":socket,
                "database_path":database,
                "durations":{"short_secs":30,"normal_secs":60,"long_secs":90},
                "profiles":[]
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            bridget: bridget.to_path_buf(),
            socket,
            config,
            database,
            release_deposit,
            deposit_sentinel,
            deposit_output,
            adapter_error,
            adapter_pgid,
        }
    }

    fn configure_agent(&self, created: &maicie::app::DelegationCreated) {
        let adapter = self.root.join("g1504-acp.sh");
        let emitter = self.root.join("maicie-emitter-acp.sh");
        let issued_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let scope = "015_scope_0123456789abcdef0123456789abcdef";
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        fs::write(
            self.root.join("objective-id"),
            created.objective_id.to_string(),
        )
        .unwrap();
        fs::write(
            self.root.join("delegation-id"),
            created.delegation_id.to_string(),
        )
        .unwrap();
        fs::write(self.root.join("deposit-issued-at"), issued_at.to_string()).unwrap();
        fs::write(
            &adapter,
            format!(
                "#!/bin/sh\nps -o pgid= -p $$ | tr -d ' ' > '{pgid}'\nread initialize\necho '{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":1}}}}'\nread session\necho '{{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{{\"sessionId\":\"g1504\"}}}}'\n(\n  while [ ! -e '{release_deposit}' ]; do sleep 0.01; done\n  '{bridget}' guichet deposer delivery-report --from g1504-agent --objective '{objective}' --delegation '{delegation}' --hash '{hash}' --in-reply-to '{tracked_id}' --id g1504-depot-01 --issued-at {issued_at} --issuer-scope '{scope}' > '{deposit}' 2>> '{errors}' || exit 33\n  grep -q 'DÉPÔT: queued' '{deposit}' || exit 34\n  : > '{deposited}'\n) &\nread prompt\nprintf '%s' \"$prompt\" | grep -q '\"method\":\"session/prompt\"' || exit 23\necho '{{\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{{\"stopReason\":\"end_turn\"}}}}'\nwhile read ignored; do :; done\n",
                pgid = self.adapter_pgid.display(),
                bridget = self.bridget.display(),
                errors = self.adapter_error.display(),
                objective = created.objective_id,
                delegation = created.delegation_id,
                tracked_id = created.message_id,
                hash = hash,
                issued_at = issued_at,
                scope = scope,
                deposit = self.deposit_output.display(),
                deposited = self.deposit_sentinel.display(),
                release_deposit = self.release_deposit.display(),
            ),
        )
        .unwrap();
        fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            &emitter,
            format!(
                "#!/bin/sh\nread initialize\necho '{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":1}}}}'\nread session\necho '{{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{{\"sessionId\":\"g1504-maicie-emitter\"}}}}'\n'{bridget}' send --from maicie --to g1504-agent --reply --timeout 60 --id '{tracked_id}' --issued-at {issued_at} --issuer-scope '{scope}' 'attestation de livraison attendue' > '{errors}.emitter' 2>&1\nsend_status=$?\n[ $send_status -eq 0 ] || grep -q 'ISSUE INCONNUE' '{errors}.emitter' || exit 41\n: > '{release_deposit}'\n",
                bridget = self.bridget.display(),
                tracked_id = created.message_id,
                issued_at = issued_at,
                scope = scope,
                errors = self.adapter_error.display(),
                release_deposit = self.release_deposit.display(),
            ),
        )
        .unwrap();
        fs::set_permissions(&emitter, fs::Permissions::from_mode(0o700)).unwrap();
        let registry = self.root.join(".config/bridget/agents.json");
        fs::write(
            &registry,
            serde_json::to_vec(&json!({"agents":{"g1504_fixture":{
                "command":adapter,
                "protocol":"acp",
                "permissions":"allow",
                "queue_capacity":1,
                "notify_timeout_secs":5,
                "mcp":{"interactive":"none","acp_session":false}
            },"maicie_emitter":{
                "command":emitter,
                "protocol":"acp",
                "permissions":"allow",
                "queue_capacity":1,
                "notify_timeout_secs":5,
                "mcp":{"interactive":"none","acp_session":false}
            }}}))
            .unwrap(),
        )
        .unwrap();
        fs::set_permissions(&registry, fs::Permissions::from_mode(0o600)).unwrap();
    }

    fn start_daemon(&self) -> Child {
        let mut child = Command::new(&self.bridget)
            .arg("daemon")
            .env("HOME", &self.root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("daemon G1504 arrêté pendant l'amorçage: {status}");
            }
            let probe = self.bridget(&["requests".to_string(), "--json".to_string()]);
            let ready = self.socket.exists() && probe.status.success();
            if ready {
                let spawn = self.bridget(&[
                    "spawn".to_string(),
                    "g1504_fixture".to_string(),
                    "--name".to_string(),
                    "g1504-agent".to_string(),
                    "--cwd".to_string(),
                    self.root.display().to_string(),
                ]);
                assert!(spawn.status.success(), "spawn G1504: {:?}", spawn.stderr);
                return child;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            stop_real_daemon_best_effort(&mut child),
            "daemon G1504 ne s'arrête pas après un amorçage incomplet"
        );
        panic!("daemon G1504 non prêt");
    }

    fn bridget(&self, args: &[String]) -> std::process::Output {
        Command::new(&self.bridget)
            .args(args)
            .env("HOME", &self.root)
            .output()
            .unwrap()
    }

    fn maicie(&self, args: &[String]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_maicie"))
            .args(args)
            .output()
            .unwrap()
    }

    fn start_ephemeral_maicie(&self) {
        let spawn = self.bridget(&[
            "spawn".to_string(),
            "maicie_emitter".to_string(),
            "--name".to_string(),
            "maicie".to_string(),
            "--cwd".to_string(),
            self.root.display().to_string(),
        ]);
        assert!(
            spawn.status.success(),
            "spawn émetteur Maicie éphémère: {:?}",
            spawn.stderr
        );
    }

    fn wait_for_agent(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if BridgetClient::list_agents_at(&self.socket)
                .unwrap_or_default()
                .iter()
                .any(|agent| agent.name == "g1504-agent" && agent.state == "connected")
            {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "wrapper G1504 non connecté: {}",
            fs::read_to_string(&self.adapter_error).unwrap_or_default(),
        );
    }

    fn wait_for_deposit(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.deposit_sentinel.exists() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "le wrapper réel n'a pas déposé sa lettre: dépôt={} émetteur={}",
            fs::read_to_string(&self.adapter_error).unwrap_or_default(),
            fs::read_to_string(format!("{}.emitter", self.adapter_error.display()))
                .unwrap_or_default(),
        );
    }

    fn deposit_args(&self, tracked_id: &str) -> Vec<String> {
        vec![
            "guichet".to_string(),
            "deposer".to_string(),
            "delivery-report".to_string(),
            "--from".to_string(),
            "g1504-agent".to_string(),
            "--objective".to_string(),
            self.created_objective_id(),
            "--delegation".to_string(),
            self.created_delegation_id(),
            "--hash".to_string(),
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "--in-reply-to".to_string(),
            tracked_id.to_string(),
            "--id".to_string(),
            "g1504-depot-01".to_string(),
            "--issued-at".to_string(),
            self.deposit_issued_at(),
            "--issuer-scope".to_string(),
            "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        ]
    }

    fn created_objective_id(&self) -> String {
        fs::read_to_string(self.root.join("objective-id"))
            .unwrap()
            .trim()
            .to_string()
    }

    fn created_delegation_id(&self) -> String {
        fs::read_to_string(self.root.join("delegation-id"))
            .unwrap()
            .trim()
            .to_string()
    }

    fn deposit_issued_at(&self) -> String {
        fs::read_to_string(self.root.join("deposit-issued-at"))
            .unwrap()
            .trim()
            .to_string()
    }

    fn assert_request_answered(&self, request_id: &str) {
        let requests = Command::new(&self.bridget)
            .args(["requests", "--json"])
            .env("HOME", &self.root)
            .env("BRIDGET_AGENT_NAME", "maicie")
            .output()
            .unwrap();
        assert!(requests.status.success(), "requests: {:?}", requests.stderr);
        let requests: Value = serde_json::from_slice(&requests.stdout).unwrap();
        assert!(
            requests
                .as_array()
                .unwrap()
                .iter()
                .any(|request| { request["id"] == request_id && request["state"] == "answered" })
        );
    }

    fn stop_agents_best_effort(&self) -> bool {
        for name in ["g1504-agent", "maicie"] {
            let _ = Command::new(&self.bridget)
                .args(["stop", name])
                .env("HOME", &self.root)
                .output();
        }
        self.stop_adapter_group_best_effort()
    }

    fn stop_adapter_group_best_effort(&self) -> bool {
        let Some(pgid) = fs::read_to_string(&self.adapter_pgid)
            .ok()
            .and_then(|value| value.trim().parse::<i32>().ok())
            .filter(|pgid| *pgid > 1)
        else {
            return true;
        };

        let exists = (unsafe { libc::kill(-pgid, 0) }) == 0;
        if exists && (unsafe { libc::kill(-pgid, libc::SIGTERM) }) != 0 {
            return false;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if (unsafe { libc::kill(-pgid, 0) }) != 0 {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        (unsafe { libc::kill(-pgid, 0) }) != 0
    }
}

impl Drop for RealGateFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stop_real_daemon_best_effort(daemon: &mut Child) -> bool {
    match daemon.try_wait() {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => return false,
    }
    if unsafe { libc::kill(daemon.id() as i32, libc::SIGTERM) } != 0 {
        return false;
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match daemon.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => {}
            Err(_) => return false,
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

fn managed_g1504_process_count() -> usize {
    let output = Command::new("/bin/ps")
        .args(["-axo", "command="])
        .output()
        .expect("ps doit être disponible pour le gate G1504");
    assert!(output.status.success(), "ps G1504 indisponible");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.contains("managed-wrapper g1504_fixture g1504-agent"))
        .count()
}

fn assert_managed_g1504_process_count(expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if managed_g1504_process_count() == expected {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        managed_g1504_process_count(),
        expected,
        "le gate G1504 a laissé un managed-wrapper orphelin"
    );
}
