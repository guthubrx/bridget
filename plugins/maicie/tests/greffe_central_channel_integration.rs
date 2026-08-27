use bridget_transport::WrapperToDaemon;
use bridget_transport::greffe_authorization::{
    GREFFE_AUDIT_PATH_ENV, GREFFE_POLICY_PATH_ENV, GreffeAuthorizationGate,
    GreffeDepositAuthorization, GreffeMutationAction,
};
use bridget_transport::protocol::{
    GuichetDelegateMutationStatus, GuichetDurationClass, GuichetRegistreAddStatus,
    GuichetReplyPayload, ServiceRequestOperation, ServiceRequestPayload, ServiceSuiteDeclaration,
    decode, encode,
};
use maicie::app::process_guichet_claim_with_central_service;
use maicie::bridget_client::{BridgetClientLimits, GuichetClaim};
use maicie::config::MaicieConfig;
use maicie::domain::EtatObjectif;
use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CHILD_ENV: &str = "MAICIE_GREFFE_CENTRAL_EFFECT_CHILD";
const ROOT_ENV: &str = "MAICIE_GREFFE_CENTRAL_EFFECT_ROOT";
const PRINCIPAL: &str = "agent-autorise";
const INSTANCE_ID: &str = "instance-autorisee";
const POLICY_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn trois_mutations_federees_autorisees_appliquent_le_greffe_central() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_effect_oracle(Path::new(&std::env::var_os(ROOT_ENV).unwrap()));
        return;
    }

    let root = std::env::temp_dir().join(format!(
        "maicie-greffe-central-{}-{}",
        std::process::id(),
        Uuid::new_v4()
    ));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let policy = root.join("policy.json");
    let audit = root.join("audit.jsonl");
    write_policy(&policy, unix_now() + 600);

    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "trois_mutations_federees_autorisees_appliquent_le_greffe_central",
            "--nocapture",
        ])
        .env(CHILD_ENV, "1")
        .env(ROOT_ENV, &root)
        .env(GREFFE_POLICY_PATH_ENV, &policy)
        .env(GREFFE_AUDIT_PATH_ENV, &audit)
        .env("RUST_TEST_THREADS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "sous-processus rouge\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}

fn run_effect_oracle(root: &Path) {
    let socket = root.join("bridget.sock");
    let database = root.join("maicie.sqlite3");
    let catalogue = root.join("catalogue.jsonl");
    let config_path = root.join("maicie.json");
    fs::write(
        &config_path,
        serde_json::to_vec(&json!({
            "version": 1,
            "bridget_socket": socket,
            "database_path": database,
            "catalogue_path": catalogue,
            "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
            "profiles": [{
                "id": "prospective",
                "display_name": "Prospective",
                "tags": ["review"],
                "personality_ref": "profiles/prospective.md",
                "tools": ["bridget_send"],
                "spawn_order_ref": "agents/prospective"
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let config = MaicieConfig::load(&config_path).unwrap();
    let mut store = MaicieStore::open(&database).unwrap();
    let issuer_scope = store.issuer_scope().to_string();
    let now = unix_now();
    let gate = GreffeAuthorizationGate::from_environment();

    let (ready_tx, ready_rx) = mpsc::channel();
    let server_socket = socket.clone();
    let server = thread::spawn(move || serve_agent_list_once(&server_socket, ready_tx));
    ready_rx.recv().unwrap();
    let delegate = authorized_claim(
        &gate,
        &issuer_scope,
        "request-delegate-central",
        now,
        GreffeMutationAction::Delegate,
        ServiceRequestOperation::Delegate,
        ServiceRequestPayload::Delegate {
            goal: "prouver le chemin fédéré central".to_string(),
            explicit_target: Some("prospective".to_string()),
            required_tags: Vec::new(),
            duration: GuichetDurationClass::Courte,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: Vec::new(),
            references: Vec::new(),
        },
    );
    let delegated = process_guichet_claim_with_central_service(
        &mut store,
        &config,
        BridgetClientLimits::default(),
        &delegate,
        "response-delegate-central",
        now + 1,
    )
    .unwrap();
    server.join().unwrap();
    fs::remove_file(&socket).unwrap();
    assert!(delegated.refusal_reason.is_none());
    let objective_id = delegated.objective_id.expect("objectif durable absent");
    let delegation_id = delegated.delegation_id.expect("délégation durable absente");
    match decode::<WrapperToDaemon>(std::str::from_utf8(&delegated.reply_bytes).unwrap()).unwrap() {
        WrapperToDaemon::GuichetReply {
            payload:
                GuichetReplyPayload::Delegate {
                    status: GuichetDelegateMutationStatus::Created,
                    objective_id: Some(reply_objective),
                    delegation_id: Some(reply_delegation),
                    ..
                },
            ..
        } => {
            assert_eq!(reply_objective, objective_id.to_string());
            assert_eq!(reply_delegation, delegation_id.to_string());
        }
        other => panic!("reçu delegate terminal inattendu: {other:?}"),
    }

    let register_line = r#"{"v":1,"kind":"add","id":"c-canal-central","date":"2026-08-27T12:00:00Z","mission_source":{"kind":"review","id":"r-canal-central"},"severity":"major","text":"mutation fédérée appliquée au journal central"}"#;
    let register = authorized_claim(
        &gate,
        &issuer_scope,
        "request-registre-central",
        now + 2,
        GreffeMutationAction::RegistreAdd,
        ServiceRequestOperation::RegistreAdd,
        ServiceRequestPayload::RegistreAdd {
            line: register_line.to_string(),
        },
    );
    let registered = process_guichet_claim_with_central_service(
        &mut store,
        &config,
        BridgetClientLimits::default(),
        &register,
        "response-registre-central",
        now + 3,
    )
    .unwrap();
    assert!(registered.refusal_reason.is_none());
    match decode::<WrapperToDaemon>(std::str::from_utf8(&registered.reply_bytes).unwrap()).unwrap()
    {
        WrapperToDaemon::GuichetReply {
            payload:
                GuichetReplyPayload::RegistreAdd {
                    status: GuichetRegistreAddStatus::Appended,
                    constat_id,
                },
            ..
        } => assert_eq!(constat_id, "c-canal-central"),
        other => panic!("reçu registre terminal inattendu: {other:?}"),
    }
    let catalogue_text = fs::read_to_string(&catalogue).unwrap();
    let catalogue_lines = catalogue_text.lines().collect::<Vec<_>>();
    assert_eq!(catalogue_lines.len(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(catalogue_lines[0]).unwrap(),
        serde_json::from_str::<Value>(register_line).unwrap()
    );

    let close = authorized_claim(
        &gate,
        &issuer_scope,
        "request-close-central",
        now + 4,
        GreffeMutationAction::ObjectiveClose,
        ServiceRequestOperation::ObjectiveClose,
        ServiceRequestPayload::ObjectiveClose {
            objective_id: objective_id.to_string(),
            reason: "clôture attestée depuis le canal".to_string(),
        },
    );
    let closed = process_guichet_claim_with_central_service(
        &mut store,
        &config,
        BridgetClientLimits::default(),
        &close,
        "response-close-central",
        now + 5,
    )
    .unwrap();
    assert!(closed.refusal_reason.is_none());
    match decode::<WrapperToDaemon>(std::str::from_utf8(&closed.reply_bytes).unwrap()).unwrap() {
        WrapperToDaemon::GuichetReply {
            payload:
                GuichetReplyPayload::ObjectiveClose {
                    objective_id: reply_objective,
                    decision_id,
                    ..
                },
            ..
        } => {
            assert_eq!(reply_objective, objective_id.to_string());
            assert!(!decision_id.is_empty());
        }
        other => panic!("reçu clôture terminal inattendu: {other:?}"),
    }
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snapshot.objective.etat, EtatObjectif::Clos);

    let refused_line = register_line.replace("c-canal-central", "c-sans-attestation");
    let mut refused = authorized_claim(
        &gate,
        &issuer_scope,
        "request-registre-refuse",
        now + 6,
        GreffeMutationAction::RegistreAdd,
        ServiceRequestOperation::RegistreAdd,
        ServiceRequestPayload::RegistreAdd { line: refused_line },
    );
    refused.authorization_attestation = None;
    let refusal = process_guichet_claim_with_central_service(
        &mut store,
        &config,
        BridgetClientLimits::default(),
        &refused,
        "response-registre-refuse",
        now + 7,
    )
    .unwrap();
    assert!(refusal.refusal_reason.is_some());
    assert!(
        !fs::read_to_string(&catalogue)
            .unwrap()
            .contains("c-sans-attestation")
    );

    let audit_lines = fs::read_to_string(root.join("audit.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        audit_lines.len(),
        8,
        "quatre dépôts et quatre contrôles d'effet"
    );
    assert_eq!(
        audit_lines
            .iter()
            .filter(|event| event["allowed"] == json!(true))
            .count(),
        7
    );
}

fn authorized_claim(
    gate: &GreffeAuthorizationGate,
    issuer_scope: &str,
    request_id: &str,
    issued_at: i64,
    action: GreffeMutationAction,
    operation: ServiceRequestOperation,
    payload: ServiceRequestPayload,
) -> GuichetClaim {
    let canonical_request = encode(&WrapperToDaemon::ServiceRequest {
        version: 1,
        issuer_scope: issuer_scope.to_string(),
        request_id: request_id.to_string(),
        issued_at,
        from: PRINCIPAL.to_string(),
        to: "maicie".to_string(),
        operation,
        payload,
    })
    .unwrap()
    .into_bytes();
    let authorization_attestation = gate
        .authorize_deposit(GreffeDepositAuthorization {
            canonical_name: Some(PRINCIPAL),
            canonical_instance_id: Some(INSTANCE_ID),
            declared_from: Some(PRINCIPAL),
            action,
            issuer_scope,
            request_id,
            request_issued_at: issued_at,
            canonical_request: &canonical_request,
            observed_at: issued_at,
        })
        .unwrap();
    GuichetClaim {
        issuer_scope: issuer_scope.to_string(),
        request_id: request_id.to_string(),
        canonical_request,
        authorization_attestation: Some(authorization_attestation),
        claimed_at: issued_at,
        claim_generation: 1,
        claim_token: format!("claim-{request_id}"),
        claim_lease_expires_at: issued_at + 30,
        expires_at: issued_at + 60,
    }
}

fn serve_agent_list_once(socket: &Path, ready: mpsc::Sender<()>) {
    let listener = UnixListener::bind(socket).unwrap();
    ready.send(()).unwrap();
    let (stream, _) = listener.accept().unwrap();
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
    assert_eq!(read_json(&mut reader)["type"], "ClientHello");
    write_json(
        &mut writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":60,
            "issued_at_tolerance_secs":5,
            "capabilities":["send_idempotent","lookup"]
        }),
    );

    let (stream, _) = listener.accept().unwrap();
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    assert_eq!(read_json(&mut reader), json!({"type":"ListAgents"}));
    write_json(
        &mut writer,
        json!({
            "type":"AgentList",
            "agents":[{
                "name":"prospective",
                "agent_type":"codex",
                "connection_id":"fixture-central",
                "host":"fixture",
                "transport":"codex_app_server",
                "state":"connected",
                "last_seen_secs":0,
                "reconnect_count":0,
                "domain":"bridget",
                "model":"test",
                "effort":"low"
            }]
        }),
    );
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn write_json(writer: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *writer, &value).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
}

fn write_policy(path: &Path, expires_at: i64) {
    fs::write(
        path,
        serde_json::to_vec(&json!({
            "version": 1,
            "generation": 7,
            "attestation_key": POLICY_KEY,
            "principals": [{
                "principal": PRINCIPAL,
                "actions": ["delegate", "registre_add", "objective_close"],
                "instances": [{
                    "instance_id": INSTANCE_ID,
                    "expires_at": expires_at,
                    "revoked": false
                }]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

fn unix_now() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap()
}
