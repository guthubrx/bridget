use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::bridget_client::BridgetClientLimits;
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
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

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
    let mut store = MaicieStore::open(database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
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
            explicit_target: Some("prospective"),
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
    .unwrap()
    else {
        panic!("délégation attendue")
    };
    created
}

#[test]
fn releve_pull_only_greffe_repond_et_enregistre_l_evenement() {
    let root = root("delivery");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let issuer_scope = MaicieStore::open(&database).unwrap().issuer_scope().to_string();
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
        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
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
        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
        write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
    });

    let mut store = MaicieStore::open(&database).unwrap();
    let report = reconcile_guichet_startup_with_limits(
        &mut store,
        fixture.path(),
        1_010,
        limits(),
    )
    .unwrap();
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
    assert!(matches!(report.actions.last(), Some(GuichetReconcileAction::Vide)));
    assert_eq!(store.objective_snapshots(Some(created.objective_id)).unwrap()[0].decisions.len(), 1);
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
    let issuer_scope = MaicieStore::open(&database).unwrap().issuer_scope().to_string();
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
        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
        write_json(
            &mut writer,
            claimed(&issuer_scope, "request-reply-lost", &canonical_request, 1),
        );
        let first_reply = read_json(&mut reader);
        let response_message_id = first_reply["response_message_id"].as_str().unwrap().to_string();
        let first_payload = first_reply["payload"].clone();
        drop(writer);
        drop(reader);

        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
        write_json(&mut writer, welcome());
        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
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
        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
        write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
    });

    let mut store = MaicieStore::open(&database).unwrap();
    let first = reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_010, limits())
        .unwrap();
    assert!(matches!(
        first.actions.last(),
        Some(GuichetReconcileAction::TransportIncertain)
    ));
    assert_eq!(store.objective_snapshots(Some(created.objective_id)).unwrap()[0].decisions.len(), 1);

    let second = reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_011, limits())
        .unwrap();
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
    assert_eq!(store.objective_snapshots(Some(created.objective_id)).unwrap()[0].decisions.len(), 1);
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
    let issuer_scope = MaicieStore::open(&database).unwrap().issuer_scope().to_string();
    let fixture = SocketFixture::new("budget");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer, &issuer_scope);
        write_json(&mut writer, welcome());
        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
        thread::sleep(Duration::from_millis(120));
    });

    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_millis(30),
        io_timeout: Duration::from_millis(30),
        max_frame_bytes: 64 * 1024,
    };
    let started = Instant::now();
    let mut store = MaicieStore::open(&database).unwrap();
    let report = reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_010, limits)
        .unwrap();
    assert!(started.elapsed() < Duration::from_millis(100));
    assert!(matches!(
        report.actions.last(),
        Some(GuichetReconcileAction::TransportIndisponible)
    ));
    assert!(store.objective_snapshots(Some(created.objective_id)).unwrap()[0]
        .decisions
        .is_empty());
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
        let issuer_scope = MaicieStore::open(&database).unwrap().issuer_scope().to_string();
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
        let report = reconcile_guichet_startup_with_limits(&mut store, fixture.path(), 1_011, limits())
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
        assert_eq!(store.objective_snapshots(Some(created.objective_id)).unwrap()[0].decisions.len(), 1);
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
    assert_eq!(read_json(reader), json!({"type":"RoleHandshake","role":"service"}));
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
    (BufReader::new(stream.try_clone().unwrap()), BufWriter::new(stream))
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
        encoded.push(if chunk.len() > 1 { TABLE[((value >> 6) & 0x3f) as usize] as char } else { '=' });
        encoded.push(if chunk.len() > 2 { TABLE[(value & 0x3f) as usize] as char } else { '=' });
    }
    encoded
}

struct SocketFixture {
    path: PathBuf,
}

impl SocketFixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        Self { path: std::env::temp_dir().join(format!("mg-{label}-{}-{sequence}.sock", std::process::id())) }
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
