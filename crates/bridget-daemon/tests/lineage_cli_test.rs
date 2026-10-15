//! Bout en bout CLI lineage 149 : daemon réel, wrapper T3 réel, socket Unix
//! réelle, semis SQLite direct. Aucun fournisseur n'est jamais lancé.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;

use bridget_transport::protocol::{
    CommunicationProjectSource, DaemonToWrapper, PresenceMode, WrapperToDaemon,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

const T3: &str = "89000000-0000-4000-8000-000000000149";
const T3_AUTRE: &str = "89000000-0000-4000-8000-000000000148";
const TASK: &str = "14900000-0000-4000-8000-000000000001";
const TASK_TERMINALE: &str = "14900000-0000-4000-8000-000000000002";
const TASK_AUTRE_RACINE: &str = "14900000-0000-4000-8000-0000000000ee";
const REQUEST: &str = "49000000-0000-4000-8000-000000000003";
const REQUEST_AUTRE: &str = "49000000-0000-4000-8000-000000000004";
const HOTE: &str = "idempotency-isolated";

/// Réplique locale de t3code::stable_uuid : v5 du namespace interne présenté
/// comme v4. Toute dérive du namespace casse ces tests visiblement.
fn stable_uuid(name: &str) -> String {
    const NAMESPACE_098: [u8; 16] = [
        0x09, 0x8b, 0x71, 0xd3, 0xc0, 0xde, 0x4a, 0x11, 0x9b, 0x1d, 0x73, 0x63, 0x6f, 0x64, 0x65,
        0x01,
    ];
    let namespace = uuid::Uuid::from_bytes(NAMESPACE_098);
    let bytes = *uuid::Uuid::new_v5(&namespace, name.as_bytes()).as_bytes();
    uuid::Builder::from_random_bytes(bytes)
        .into_uuid()
        .hyphenated()
        .to_string()
}

fn project_root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn run_cli(root: &std::path::Path, args: &[&str]) -> (Option<i32>, Value) {
    let output = support::run_isolated(root, args, false);
    let text = support::output_text(&output);
    let value: Value = serde_json::from_str(text.trim()).unwrap_or(Value::Null);
    (output.status.code(), value)
}

/// Même échange JSONL borné que spec145_human_cli::negotiated_cli, avec le
/// support isolé existant. Ce faux maître ne projette pas de données métier.
fn negotiated_lineage162(action: &str, reply: Value) -> (Option<i32>, Value) {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    let root = support::test_root("l162-cli-negociation");
    let socket = support::socket(&root);
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server_listener = listener.try_clone().unwrap();
    let server = std::thread::spawn(move || {
        let mut fd = libc::pollfd {
            fd: server_listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut fd, 1, 6_000) } > 0,
            "connexion CLI lineage absente"
        );
        let (stream, _) = server_listener.accept().unwrap();
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut writer = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut seen = Vec::new();
        for response in [
            serde_json::json!({"type":"RoleAccepted","role":"client"}),
            serde_json::json!({
                "type":"ClientWelcome","version":1,"build_id":"fixture",
                "horizon_secs":60,"issued_at_tolerance_secs":5,
                "capabilities":["human_lineage_view_v1"]
            }),
            serde_json::json!({"type":"HumanLineageResult","result":reply}),
        ] {
            let mut line = String::new();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            seen.push(serde_json::from_str::<Value>(&line).unwrap());
            writeln!(writer, "{response}").unwrap();
            writer.flush().unwrap();
        }
        let mut extra = String::new();
        assert_eq!(
            reader.read_line(&mut extra).unwrap(),
            0,
            "aucun repli sur la connexion existante : {extra}"
        );
        seen
    });
    let mut args = vec![
        "lineage",
        "inspect",
        "--json",
        "--t3-thread",
        T3,
        "--project-root",
        "/fixture",
        "--action",
        action,
        "--response-version",
        "2",
    ];
    if action == "show" {
        args.extend_from_slice(&["--task", TASK]);
    }
    let output = support::run_isolated(&root, &args, false);
    let seen = server.join().unwrap();
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "aucune seconde connexion ou tentative de repli CLI"
    );
    assert_eq!(seen.len(), 3);
    assert_eq!(
        seen[0],
        serde_json::json!({"type":"RoleHandshake","role":"client"})
    );
    assert_eq!(seen[1]["type"], "ClientHello");
    assert_eq!(seen[1]["contract_version"], 1);
    assert_eq!(
        seen[1]["capabilities"],
        serde_json::json!(["human_lineage_view_v1"])
    );
    assert_eq!(seen[2]["type"], "HumanLineage");
    assert_eq!(seen[2]["request"]["version"], 2);
    assert_eq!(seen[2]["request"]["t3_thread_id"], T3);
    assert_eq!(seen[2]["request"]["project_root"], "/fixture");
    assert_eq!(seen[2]["request"]["request"]["action"], action);
    if action == "show" {
        assert_eq!(seen[2]["request"]["request"]["task_id"], TASK);
    }
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "sortie CLI JSON invalide : {error}; {}",
            support::output_text(&output)
        )
    });
    drop(listener);
    std::fs::remove_dir_all(&root).unwrap();
    (output.status.code(), value)
}

/// La réponse explicite du maître est relayée telle quelle par la CLI. Ce
/// maître contrôlé exerce le client réel, pas le parseur interne seul.
fn negotiated_lineage163(
    action: &str,
    replies: Vec<Value>,
    response_version: &str,
) -> (Option<i32>, Value, Vec<Vec<Value>>) {
    let root = support::test_root("l163-cli-negociation");
    let socket = support::socket(&root);
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server_listener = listener.try_clone().unwrap();
    let server = std::thread::spawn(move || {
        replies
            .into_iter()
            .map(|reply| {
                let mut fd = libc::pollfd {
                    fd: server_listener.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                assert!(
                    unsafe { libc::poll(&mut fd, 1, 6_000) } > 0,
                    "connexion CLI Lineage v3 absente"
                );
                let (stream, _) = server_listener.accept().unwrap();
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut writer = stream.try_clone().unwrap();
                let mut reader = BufReader::new(stream);
                let mut seen = Vec::new();
                for response in [
                    serde_json::json!({"type":"RoleAccepted","role":"client"}),
                    serde_json::json!({
                        "type":"ClientWelcome","version":1,"build_id":"fixture",
                        "horizon_secs":60,"issued_at_tolerance_secs":5,
                        "capabilities":["human_lineage_view_v1"]
                    }),
                    serde_json::json!({"type":"HumanLineageResult","result":reply}),
                ] {
                    let mut line = String::new();
                    assert!(reader.read_line(&mut line).unwrap() > 0);
                    seen.push(serde_json::from_str::<Value>(&line).unwrap());
                    writeln!(writer, "{response}").unwrap();
                    writer.flush().unwrap();
                }
                seen
            })
            .collect::<Vec<_>>()
    });
    let mut args = vec![
        "lineage",
        "inspect",
        "--json",
        "--t3-thread",
        T3,
        "--project-root",
        "/fixture",
        "--action",
        action,
        "--response-version",
        response_version,
    ];
    if action == "show" {
        args.extend_from_slice(&["--task", TASK]);
    }
    let output = support::run_isolated(&root, &args, false);
    let seen = server.join().unwrap();
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "aucun repli supplémentaire ne doit être tenté"
    );
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "sortie CLI JSON invalide : {error}; {}",
            support::output_text(&output)
        )
    });
    drop(listener);
    std::fs::remove_dir_all(&root).unwrap();
    (output.status.code(), value, seen)
}

/// Prépare l'identité privée déjà exigée par la CLI auxiliaire. Le serveur
/// contrôlé vérifie ensuite que la commande réemploie bien le handler MCP.
fn auto_cli_identity(root: &std::path::Path) -> (String, String) {
    let agent_id = support::ACTOR.to_string();
    let instance_id = "auto-cli-163".to_string();
    let state = root.join("state");
    let name = state.join("auto-cli-name");
    support::private_write(&name, &agent_id).unwrap();
    let proofs = state.join("agent-names");
    support::private_dir(&proofs).unwrap();
    let proof = proofs.join(format!(
        "proof-{:x}.json",
        Sha256::digest(instance_id.as_bytes())
    ));
    support::private_write(
        &proof,
        serde_json::to_vec(&serde_json::json!({
            "agent_id": agent_id,
            "instance_id": instance_id,
            "credential": "credential-auto-cli-163"
        }))
        .unwrap(),
    )
    .unwrap();
    (name.to_string_lossy().into_owned(), instance_id)
}

/// Même harnais socket que le MCP : l'oracle ne laisse aucun maître ancien
/// recevoir une NativeDelegation auto, et inspecte le wire du maître capable.
fn negotiated_auto_delegate163(features: &[&str]) -> (std::process::Output, Vec<Value>) {
    let root = support::test_root("auto-cli-163");
    let socket = support::socket(&root);
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    let advertised: Vec<String> = features.iter().map(|value| (*value).to_owned()).collect();
    let supports_auto = features.contains(&"auto_delegate_v2");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut seen = Vec::new();
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        seen.push(serde_json::from_str::<Value>(&line).unwrap());
        writeln!(
            stream,
            "{}",
            serde_json::json!({"type":"Registered","agent_id":support::ACTOR,"credential":null})
        )
        .unwrap();
        line.clear();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        seen.push(serde_json::from_str::<Value>(&line).unwrap());
        writeln!(
            stream,
            "{}",
            serde_json::json!({
                "type":"DaemonIdentityReport","host":"fixture","db_path":"fixture",
                "instance_id":"fixture","features":advertised
            })
        )
        .unwrap();
        if supports_auto {
            line.clear();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            seen.push(serde_json::from_str::<Value>(&line).unwrap());
            writeln!(
                stream,
                "{}",
                serde_json::json!({
                    "type":"NativeDelegationResult",
                    "result":{"status":"admitted","task_id":"task-auto-cli-163"}
                })
            )
            .unwrap();
        } else {
            line.clear();
            assert_eq!(
                reader.read_line(&mut line).unwrap(),
                0,
                "un maître sans feature ne reçoit aucune admission auto"
            );
        }
        seen
    });
    let (name, instance_id) = auto_cli_identity(&root);
    let mut command = support::isolated_command(&root);
    command
        .env("BRIDGET_AGENT_ID_FILE", name)
        .env("BRIDGET_AGENT_INSTANCE_ID", instance_id)
        .args([
            "auto-delegate",
            "--request-id",
            "auto-cli-163",
            "--task",
            "mission CLI auto",
            "--cwd",
            "/tmp",
            "--urgent",
            "true",
            "--deliverable",
            "preuve",
        ]);
    let output = support::run_command(command);
    let seen = server.join().unwrap();
    std::fs::remove_dir_all(&root).unwrap();
    (output, seen)
}

#[test]
fn native162_lineage_cli_accepts_requested_v2_success() {
    for action in ["list", "show"] {
        let reply = serde_json::json!({"version":2,"status":"ok","seq":0});
        let (code, value) = negotiated_lineage162(action, reply.clone());
        assert_eq!(code, Some(0), "{action}: {value}");
        assert_eq!(value, reply, "{action}");
    }
}

#[test]
fn native162_lineage_cli_preserves_v1_error_for_v2_request() {
    use bridget_transport::protocol::HumanLineageError;
    for action in ["list", "show"] {
        for error in [
            HumanLineageError::UnsupportedVersion,
            HumanLineageError::InvalidRequest,
        ] {
            let reply = error.result();
            let (code, value) = negotiated_lineage162(action, reply.clone());
            assert_eq!(code, Some(2), "{action}: {value}");
            assert_eq!(value, reply, "{action}");
        }
    }
}

#[test]
fn native162_lineage_cli_rejects_v1_success_for_v2_request() {
    for action in ["list", "show"] {
        let reply = serde_json::json!({"version":1,"status":"ok","seq":0});
        let (code, value) = negotiated_lineage162(action, reply);
        assert_eq!(code, Some(3), "{action}: {value}");
        assert_eq!(
            value,
            bridget_transport::protocol::HumanLineageError::BindingUnavailable.result(),
            "{action}"
        );
    }
}

#[test]
fn spec163_lineage_cli_version_explicite_est_stricte_et_v3_garde_placement() {
    for action in ["list", "show"] {
        let mut direct_v3 = serde_json::json!({
            "version":3,"status":"ok","seq":0,
            "placement":{"machine_id":"cartae","display_name":null,"worker_id":"worker","last_contact_at":null,"contact_state":"unknown"},
            "decision":null
        });
        if action == "list" {
            let task = serde_json::json!({"task_id":TASK,"placement":direct_v3["placement"].clone(),"decision":null});
            direct_v3.as_object_mut().unwrap().remove("placement");
            direct_v3.as_object_mut().unwrap().remove("decision");
            direct_v3["tasks"] = serde_json::json!([task]);
        } else {
            let task = serde_json::json!({"task_id":TASK,"placement":direct_v3["placement"].clone(),"decision":null});
            direct_v3.as_object_mut().unwrap().remove("placement");
            direct_v3.as_object_mut().unwrap().remove("decision");
            direct_v3["task"] = task;
        }
        let (code, value, seen) = negotiated_lineage163(action, vec![direct_v3.clone()], "3");
        assert_eq!(code, Some(0), "{action}: {value}");
        assert_eq!(value, direct_v3, "{action}");
        assert_eq!(seen.len(), 1, "{action}");
        assert_eq!(seen[0][2]["request"]["version"], 3, "{action}");

        let refusal = bridget_transport::protocol::HumanLineageError::UnsupportedVersion.result();
        let (code, value, seen) = negotiated_lineage163(action, vec![refusal.clone()], "3");
        assert_eq!(code, Some(2), "{action}: {value}");
        assert_eq!(value, refusal, "le refus v3 reste brut : {action}");
        assert_eq!(seen.len(), 1, "aucun downgrade CLI : {action}");
        assert_eq!(seen[0][2]["request"]["version"], 3, "{action}");

        let direct_v2 = serde_json::json!({"version":2,"status":"ok","seq":0});
        let (code, value) = negotiated_lineage162(action, direct_v2.clone());
        assert_eq!(code, Some(0), "v2 explicite permis : {action}: {value}");
        assert_eq!(value, direct_v2, "v2 explicite permis : {action}");
    }
}

#[test]
fn spec168_lineage_cli_v4_conserve_version_et_refus_sans_downgrade() {
    for action in ["list", "show"] {
        let mut success = serde_json::json!({"version":4,"status":"ok","seq":0});
        let decision = serde_json::json!({
            "candidate_id":"c012", "connection_id":"codex-work", "machine_id":"cartae",
            "engine":"codex", "model":"gpt-6.1-sol", "effort":"high",
            "source":"deterministic",
            "facts":{"scored_at":1000,
                "resources":{"observed_at":null,"age_secs":null,"fresh":false,
                    "cpu_percent":null,"available_memory_percent":null,
                    "normalized_load_percent":null,"compilation_slots":null},
                "compilation_capacity":null,"continuity":"unknown",
                "quota":{"observed_at":null,"age_secs":null,"used_percent":null,"resets_at":null},
                "score_terms":{"cpu":0,"memory":0,"load":0,"continuity":0,"compilation":0,
                    "quota_remaining":0,"quota_reset":0,"saturation_penalty":0,"total":0}}
        });
        if action == "list" {
            success["tasks"] = serde_json::json!([{"task_id":TASK,"decision":decision}]);
        } else {
            success["task"] = serde_json::json!({"task_id":TASK,"decision":decision});
        }
        let (code, value, seen) = negotiated_lineage163(action, vec![success.clone()], "4");
        assert_eq!(code, Some(0), "{action}: {value}");
        assert_eq!(
            value, success,
            "faits figés et inconnus transmis sans reconstruction"
        );
        let decision = if action == "list" {
            &value["tasks"][0]["decision"]
        } else {
            &value["task"]["decision"]
        };
        assert!(decision.get("reason").is_none());
        assert!(decision["facts"]["resources"]["cpu_percent"].is_null());
        assert_eq!(decision["facts"]["scored_at"], 1000);
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0][2]["request"]["version"], 4);

        let refusal = bridget_transport::protocol::HumanLineageError::UnsupportedVersion.result();
        let (code, value, seen) = negotiated_lineage163(action, vec![refusal.clone()], "4");
        assert_eq!(code, Some(2), "{action}: {value}");
        assert_eq!(value, refusal);
        assert_eq!(seen.len(), 1, "aucun downgrade v4 : {action}");
        assert_eq!(seen[0][2]["request"]["version"], 4);
    }
}

#[test]
fn spec163_lineage_cli_v3_invalid_request_ne_declenche_pas_de_repli() {
    let refusal = bridget_transport::protocol::HumanLineageError::InvalidRequest.result();
    let (code, value, seen) = negotiated_lineage163("show", vec![refusal.clone()], "3");
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value, refusal);
    assert_eq!(seen.len(), 1, "un refus v3 invalide est terminal");
    assert_eq!(seen[0][2]["request"]["version"], 3);
}

#[test]
fn spec163_auto_delegate_cli_negocie_le_feature_et_ne_masque_pas_les_erreurs() {
    let (output, seen) = negotiated_auto_delegate163(&["auto_delegate_v2"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        support::output_text(&output)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        serde_json::json!({"status":"admitted","task_id":"task-auto-cli-163"})
    );
    assert_eq!(seen.len(), 3);
    assert_eq!(seen[0]["type"], "RegisterAuxiliary");
    assert_eq!(seen[1]["type"], "DaemonIdentityRequest");
    assert_eq!(seen[2]["type"], "NativeDelegation");
    assert_eq!(seen[2]["request"]["operation"], "auto_delegate_v2");
    assert_eq!(seen[2]["request"]["request_id"], "auto-cli-163");
    assert_eq!(seen[2]["request"]["task"], "mission CLI auto");
    assert_eq!(seen[2]["request"]["urgent"], true);
    assert!(seen[2]["request"].get("fallback").is_none());

    let (output, seen) = negotiated_auto_delegate163(&[]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        support::output_text(&output)
    );
    assert!(output.stdout.is_empty(), "aucun succès synthétique");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("auto_delegate_v2_unsupported"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(seen.len(), 2, "aucune NativeDelegation sans feature");
}

fn definition() -> Value {
    serde_json::json!({
        "command": "npx",
        "args": ["fixture-lineage149"],
        "protocol": "acp",
        "forbidden_env": ["API_KEY"],
        "pass_env": [],
        "claude_config_dir": null,
        "permissions": "allow",
        "queue_capacity": 32,
        "notify_timeout_secs": 600,
        "mcp": {"interactive": "none", "acp_session": false},
        "capabilities": {"execution_paths": ["acp"], "models": {}},
        "digest": "fixture-lineage149"
    })
}

/// Semis SQL direct dans la base du daemon réel : la projection démarre au
/// marqueur seq 0, distinct de toute écriture projetée.
fn seed_task(
    db: &rusqlite::Connection,
    task_id: &str,
    owner: &str,
    state: &str,
    result: Option<&str>,
) {
    let payload = serde_json::json!({
        "task_id": task_id,
        "root_owner_agent_id": owner,
        "owner": owner,
        "owner_instance": format!("inst-{owner}"),
        "origin_owner_instance": format!("inst-{owner}"),
        "request": {
            "operation": "delegate",
            "request_id": format!("req-{task_id}"),
            "agent_type": "fixture",
            "model": "fixture-model",
            "task": format!("Mission CLI {task_id}"),
            "cwd": "/tmp"
        },
        "definition": definition(),
        "cwd": "/tmp",
        "child": stable_uuid(&format!("child:{task_id}")),
        "child_instance": stable_uuid(&format!("child-instance:{task_id}")),
        "mission": format!("mission-{task_id}"),
        "created_at": 1_700_000_000,
        "updated_at": 1_700_000_000,
        "state": state,
        "result": result,
        "error": null,
        "result_sent": false
    });
    db.execute(
        "INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
        rusqlite::params![task_id, format!("inst-{owner}"), format!("req-{task_id}"), b"cli149", payload.to_string()],
    )
    .unwrap();
}

/// Enregistre le wrapper T3, annonce le projet et lie le fil. Le client reste
/// vivant : la liaison disparaît à la déconnexion, jamais avant.
fn register_t3_wrapper(socket: &std::path::Path) -> support::Client {
    let agent = stable_uuid(T3);
    let instance = stable_uuid(&format!("instance:{T3}"));
    let mut t3 = support::Client::connect(socket);
    t3.send(WrapperToDaemon::Register {
        agent_type: "fixture".to_string(),
        identity_version: 2,
        agent_id: agent.clone(),
        host: Some(HOTE.to_string()),
        transport: Some("t3code".to_string()),
        channel: None.into(),
        mode: Some(PresenceMode::Cli),
        location: None,
        os: Some("test".to_string()),
        instance_id: Some(instance),
        domain: None,
        journal_available: Some(false),
        turn_in_progress: false,
        turn_attest: false,
        queue_fallback: false,
    });
    let DaemonToWrapper::Registered { agent_id, .. } = t3.receive() else {
        panic!("wrapper T3 attendu");
    };
    assert_eq!(agent_id, agent);
    let annonce = || WrapperToDaemon::CommunicationProjectFact {
        root: project_root(),
        source: CommunicationProjectSource::T3,
        host: HOTE.to_string(),
        worktree_root: None,
    };
    t3.send(annonce());
    assert!(matches!(
        t3.receive(),
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
    t3.send(
        serde_json::from_value(serde_json::json!({
            "type":"T3ThreadBindingFact","version":1,"t3_thread_id":T3
        }))
        .unwrap(),
    );
    // Le fait de liaison n'a pas de réponse : un aller-retour sur la même
    // connexion garantit qu'il est traité avant la première CLI.
    t3.send(annonce());
    assert!(matches!(
        t3.receive(),
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
    t3
}

fn journal_ecrire(directory: &std::path::Path, suites: &[(u64, &str)]) {
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
        .unwrap();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(directory.join(format!(
            "{}.jsonl",
            bridget_transport::journal::current_host_date()
        )))
        .unwrap();
    use std::io::Write;
    for (seq, texte) in suites {
        writeln!(
            file,
            "{}",
            serde_json::json!({"v":1,"seq":seq,"event":"note","payload":{"text":texte}})
        )
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// Grammaire fermée sans daemon : chaque forme ouverte sort en exit 2.
// ---------------------------------------------------------------------------

#[test]
fn lineage_cli_149_grammaire_fermee_sans_daemon() {
    let root = support::test_root("l149-cli-grammaire");
    let casses: Vec<Vec<&str>> = vec![
        vec![],
        vec!["inspect"],
        vec![
            "inspect",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
        ],
        vec![
            "inspect",
            "--json",
            "--project-root",
            "/tmp",
            "--action",
            "list",
        ],
        vec!["inspect", "--json", "--t3-thread", T3, "--action", "list"],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "relative",
            "--action",
            "list",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
            "--inconnu",
            "x",
        ],
        vec![
            "inspect",
            "--json",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            "--project-root",
            "/tmp",
            "--action",
            "list",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "inconnue",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
            "--limit",
            "0",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
            "--limit",
            "101",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
            "--cursor",
            "curseur\ncontrole",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "show",
            "--task",
            "pas-un-uuid",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "show",
            "--task",
            TASK,
            "--offset",
            "262145",
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "journal",
            "--task",
            TASK,
            "--after-seq",
            "9007199254740992",
        ],
        vec![
            "cancel",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--task",
            "pas-un-uuid",
            "--request-id",
            REQUEST,
        ],
        vec![
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
            "--task",
            TASK,
        ],
    ];
    for cas in &casses {
        let mut args: Vec<&str> = vec!["lineage"];
        args.extend_from_slice(cas);
        let (code, value) = run_cli(&root, &args);
        assert_eq!(code, Some(2), "{cas:?} → {value}");
        assert_eq!(value["code"], "invalid_request", "{cas:?} → {value}");
        assert_eq!(value["retryable"], false, "{cas:?} → {value}");
    }
    // Socket absente : refus de liaison en exit 3, jamais un silence.
    let (code, value) = run_cli(
        &root,
        &[
            "lineage",
            "inspect",
            "--json",
            "--t3-thread",
            T3,
            "--project-root",
            "/tmp",
            "--action",
            "list",
        ],
    );
    assert_eq!(code, Some(3), "{value}");
    assert_eq!(value["code"], "binding_unavailable", "{value}");
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------------
// Daemon réel : liste, show, journal, cancel sous garde native complète.
// ---------------------------------------------------------------------------

#[test]
fn lineage_cli_149_bout_en_bout_liste_show_journal_et_cancel() {
    let root = support::test_root("l149-cli-e2e");
    let daemon = support::spawn_daemon(&root, None);
    let _t3 = register_t3_wrapper(&support::socket(&root));

    let db = rusqlite::Connection::open(root.join("state/bridget.db")).unwrap();
    let agent = stable_uuid(T3);
    seed_task(&db, TASK, &agent, "queued", None);
    seed_task(
        &db,
        TASK_TERMINALE,
        &agent,
        "result_available",
        Some("Sortie CLI 149 éè"),
    );
    seed_task(&db, TASK_AUTRE_RACINE, "autre-racine-149", "queued", None);
    drop(db);

    let racine = project_root();
    let fil: [&str; 4] = ["--t3-thread", T3, "--project-root", racine.as_str()];

    // List : seq 0 au semis direct, les deux tâches de la racine, jamais celle d'une autre racine.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--action", "list"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["status"], "ok", "{value}");
    assert_eq!(value["seq"], 0, "{value}");
    let tasks = value["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2, "{value}");
    assert!(
        tasks
            .iter()
            .all(|task| task["task_id"] != TASK_AUTRE_RACINE),
        "{value}"
    );
    let entree = tasks
        .iter()
        .find(|task| task["task_id"] == TASK)
        .expect("TASK listée");
    assert_eq!(entree["journal_available"], false);

    // Show : le résultat textuel n'est servi qu'à la tâche terminale.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&[
        "--action",
        "show",
        "--task",
        TASK_TERMINALE,
        "--offset",
        "0",
        "--limit",
        "100",
    ]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["result"], "Sortie CLI 149 éè", "{value}");
    assert_eq!(
        value["result_total_bytes"],
        "Sortie CLI 149 éè".len(),
        "{value}"
    );

    // Mauvais fil : liaison indisponible, jamais les données.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&["--t3-thread", T3_AUTRE, "--project-root", racine.as_str()]);
    args.extend_from_slice(&[
        "--action",
        "show",
        "--task",
        TASK_TERMINALE,
        "--offset",
        "0",
        "--limit",
        "100",
    ]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "binding_unavailable", "{value}");

    // Mauvaise racine : mismatch de projet.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&["--t3-thread", T3, "--project-root", "/tmp"]);
    args.extend_from_slice(&[
        "--action",
        "show",
        "--task",
        TASK_TERMINALE,
        "--offset",
        "0",
        "--limit",
        "100",
    ]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "project_mismatch", "{value}");

    // Offset au milieu d'un point de code : refus explicite.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&[
        "--action",
        "show",
        "--task",
        TASK_TERMINALE,
        "--offset",
        "16",
        "--limit",
        "100",
    ]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "result_offset_invalid", "{value}");

    // Journal d'une tâche inconnue : refus fermé.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&[
        "--action",
        "journal",
        "--task",
        "00000000-0000-4000-8000-000000000000",
        "--after-seq",
        "0",
        "--limit",
        "50",
    ]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "task_unavailable", "{value}");

    // Le dossier sessions/<child> privé rend le journal lisible par la CLI.
    let child = stable_uuid(&format!("child:{TASK}"));
    journal_ecrire(
        &root.join("state").join("sessions").join(&child),
        &[(1, "étape un"), (2, "étape deux")],
    );
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&[
        "--action",
        "journal",
        "--task",
        TASK,
        "--after-seq",
        "0",
        "--limit",
        "50",
    ]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["status"], "ok", "{value}");
    assert_eq!(value["events"].as_array().unwrap().len(), 2, "{value}");
    assert_eq!(value["caught_up"], true, "{value}");

    // Cancel : reçu moteur, rejeu identique, mismatch d'enveloppe.
    let mut args: Vec<&str> = vec!["lineage", "cancel", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--task", TASK, "--request-id", REQUEST]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(
        value,
        serde_json::json!({"version":1,"task_id":TASK,"status":"cancelling"}),
        "{value}"
    );
    let (code, rejoue) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{rejoue}");
    assert_eq!(rejoue, value, "le reçu est rejoué à l'identique");
    let mut args: Vec<&str> = vec!["lineage", "cancel", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--task", TASK_TERMINALE, "--request-id", REQUEST]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "envelope_mismatch", "{value}");

    // Une seconde demande sur la tâche terminale : reçu sans mutation.
    let mut args: Vec<&str> = vec!["lineage", "cancel", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--task", TASK_TERMINALE, "--request-id", REQUEST_AUTRE]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["status"], "result_available", "{value}");

    daemon.stop();
}

// ---------------------------------------------------------------------------
// Watch CLI : Ready seq 0 sur la vraie socket, fermeture propre en exit 0.
// ---------------------------------------------------------------------------

#[test]
fn lineage_cli_149_watch_ready_puis_fermeture_propre() {
    let root = support::test_root("l149-cli-watch");
    let daemon = support::spawn_daemon(&root, None);
    let _t3 = register_t3_wrapper(&support::socket(&root));

    let db = rusqlite::Connection::open(root.join("state/bridget.db")).unwrap();
    seed_task(&db, TASK, &stable_uuid(T3), "queued", None);
    drop(db);

    let mut command = support::isolated_command(&root);
    command
        .arg("lineage")
        .arg("watch")
        .arg("--json")
        .arg("--t3-thread")
        .arg(T3)
        .arg("--project-root")
        .arg(project_root());
    command.stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = command.spawn().expect("CLI watch réelle");
    support::track(&child);
    let stdout = child.stdout.take().expect("stdout watch");
    let stdout_fd = stdout.as_raw_fd();
    let mut reader = BufReader::new(stdout);
    let mut poll_fd = libc::pollfd {
        fd: stdout_fd,
        events: libc::POLLIN,
        revents: 0,
    };
    assert!(
        unsafe { libc::poll(&mut poll_fd, 1, 15_000) } > 0,
        "ready watch absent"
    );
    let mut ligne = String::new();
    assert!(
        reader.read_line(&mut ligne).unwrap() > 0,
        "ligne ready vide"
    );
    let value: Value = serde_json::from_str(ligne.trim()).expect(&ligne);
    assert_eq!(value["status"], "ready", "{ligne}");
    assert_eq!(value["version"], 1, "{ligne}");
    assert_eq!(
        value["seq"], 0,
        "le marqueur Ready est distinct de la seq du magasin : {ligne}"
    );

    // Fermeture du stdout : POLLHUP côté CLI → sortie 0, jamais une erreur.
    drop(reader);
    support::wait_child(&mut child, Duration::from_secs(15));
    let status = child.try_wait().unwrap();
    assert_eq!(
        status.and_then(|s| s.code()),
        Some(0),
        "watch CLI non terminé proprement"
    );
    daemon.stop();
}
