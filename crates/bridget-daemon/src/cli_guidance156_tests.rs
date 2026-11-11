use super::*;

#[test]
fn remote162_worker_cli_options_explicites_sans_namespace() {
    let args: Vec<String> = [
        "--machine-id",
        "cartae",
        "--worker-id",
        "worker-cartae",
        "--provisioning-revision",
        "1",
        "--credential-file",
        "/private/credential",
        "--state-root",
        "/private/state",
        "--registry-file",
        "/private/registry.json",
        "--master-socket",
        "/private/master.sock",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let config = parse_worker_args(&args).unwrap();
    assert_eq!(config.machine_id, "cartae");
    assert_eq!(config.provisioning_revision, 1);
    assert!(parse_worker_args(&[]).is_err());
    for (index, value) in [
        (0, "--unknown"),
        (2, "--machine-id"),
        (5, "0"),
        (7, "relative"),
        (9, "/private/../other"),
    ] {
        let mut invalid = args.clone();
        invalid[index] = value.into();
        assert!(parse_worker_args(&invalid).is_err(), "{index}: {value}");
    }
}

use std::os::unix::net::UnixListener;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Instant;

#[test]
fn wake158b_cli_g7_formes_fermees_et_identifiant_retry() {
    use bridget_transport::protocol::WakeAction;
    if let Ok(args) = std::env::var("BRIDGET_GUIDANCE156_CLI_ARGS") {
        let mut args = serde_json::from_str::<Vec<String>>(&args).unwrap();
        if let Ok(receipt) = std::env::var("BRIDGET_GUIDANCE156_CLI_REAL_RECEIPT") {
            // Daemon privé existant dans l'enfant uniquement : exit ferme ses threads.
            let (socket, db) = super::idempotency_projection_tests::start_real_daemon();
            let identity = super::idempotency_projection_tests::caller_identity();
            let mut owner = super::idempotency_projection_tests::owner_connection(&socket);
            // Réemploi cli_register : deuxième membre102 réellement inscrit, sans fournisseur.
            let peer_id = "15800000-0000-4000-8000-000000000017";
            let mut peer = DaemonConnection::connect(&socket).unwrap();
            let mut registration = cli_register("fixture");
            if let WrapperToDaemon::Register {
                agent_id,
                agent_type,
                instance_id,
                ..
            } = &mut registration
            {
                *agent_id = peer_id.into();
                *agent_type = "fixture".into();
                *instance_id = Some("test-cli-peer158b".into());
            }
            assert!(matches!(
                peer.exchange(&registration).unwrap(),
                DaemonToWrapper::Registered {
                    credential: Some(_),
                    ..
                }
            ));
            let request = serde_json::from_value(serde_json::json!({"version":1,"request":{
                    "action":"create","title":"CLI fil headless158B","members":[peer_id],"operation_id":uuid::Uuid::new_v4().to_string()
                }})).unwrap();
            let DaemonToWrapper::ThreadResult { result } = owner
                .exchange(&WrapperToDaemon::ThreadRequest { request })
                .unwrap()
            else {
                panic!("création102 réelle attendue");
            };
            assert_eq!(result.result["status"], "created", "{}", result.result);
            let thread_id = result.result["thread_id"].as_str().unwrap();
            let daemon_root = socket.parent().unwrap();
            let name_file = daemon_root.join("agent-name158b");
            bridget_transport::fsutil::write_private_file_atomic(
                &name_file,
                identity.name.as_bytes(),
            )
            .unwrap();
            let id = args.windows(2).find(|w| w[0] == "--id").unwrap()[1].clone();
            args[0] = thread_id.to_owned();
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                std::env::set_var("BRIDGET_HOME", daemon_root);
                std::env::set_var("BRIDGET_SOCKET", &socket);
                if id != "missing-context158b" {
                    std::env::set_var("BRIDGET_AGENT_ID_FILE", &name_file);
                    std::env::set_var(
                        "BRIDGET_AGENT_INSTANCE_ID",
                        if id == "invalid-context158b" {
                            "autre-instance158b"
                        } else {
                            &identity.instance_id
                        },
                    );
                }
            }
            bridget_transport::fsutil::write_private_file_atomic(Path::new(&receipt), serde_json::to_vec(&serde_json::json!({
                    "db":db,"thread_id":thread_id,"owner":identity.name,"instance":identity.instance_id,"id":id
                })).unwrap().as_slice()).unwrap();
            cmd_wake(&args);
            // Maintenir la primaire et sa credential réelles jusqu'au retour CLI.
            drop(peer);
            drop(owner);
        } else {
            cmd_wake(&args);
        }
        return;
    }
    let explicit = ["agent-connu", "--label", "libellé", "--id", "retry158b"].map(str::to_string);
    assert!(
        matches!(parse_wake_args(&explicit).unwrap(), WakeAction::CliOnce {
            target, label:Some(label), id,
        } if target == "agent-connu" && label == "libellé" && id == "retry158b")
    );
    let fresh = parse_wake_args(&["agent-connu".into()]).unwrap();
    let WakeAction::CliOnce { target, label, id } = fresh else {
        panic!("déclenchement CLI unitaire attendu");
    };
    assert_eq!(target, "agent-connu");
    assert!(label.is_none());
    assert!(
        uuid::Uuid::parse_str(&id).is_ok(),
        "--id omis crée une clé unique réelle"
    );
    assert!(
        matches!(parse_wake_args(&["agent-connu".into(), "--label".into(), "".into()]).unwrap(),
            WakeAction::CliOnce { label:Some(label), .. } if label.is_empty())
    );
    for (flag, too_large) in [("--label", "é".repeat(1025)), ("--id", "x".repeat(257))] {
        assert!(parse_wake_args(&["agent-connu".into(), flag.into(), too_large]).is_err());
    }
    assert!(matches!(
        parse_wake_args(&["list".into()]).unwrap(),
        WakeAction::List {}
    ));
    assert!(matches!(
        parse_wake_args(&["list".into(), "--all".into()]).unwrap(),
        WakeAction::AdminList {}
    ));
    let schedule_id = "15800000-0000-4000-8000-000000000007";
    assert!(
        matches!(parse_wake_args(&["cancel".into(), schedule_id.into()]).unwrap(),
            WakeAction::AdminCancel { schedule_id:id } if id == schedule_id)
    );
    for invalid in [
        vec![],
        vec!["--label", "sans-cible"],
        vec!["agent-connu", "--label"],
        vec!["agent-connu", "--id"],
        vec!["agent-connu", "--from", "humain"],
        vec!["agent-connu", "--owner", "autre"],
        vec!["agent-connu", "--issued-at", "0"],
        vec!["list", "--inconnu"],
        vec!["cancel"],
        vec!["cancel", "pas-un-uuid"],
        vec!["agent-connu", "--id", "a", "--id", "b"],
    ] {
        let args = invalid.into_iter().map(str::to_string).collect::<Vec<_>>();
        assert!(
            parse_wake_args(&args).is_err(),
            "forme CLI refusée avant connexion : {args:?}"
        );
    }
    // Même commande sans TTY : le socket observe la vraie négociation et
    // l'objet fermé. Le faux serveur ne prouve ni quotas ni écritures SQL.
    for (id, exit_code) in [
        ("retry158b", 0),
        ("refused158b", 2),
        ("error158b", 1),
        ("transport158b", 1),
    ] {
        let (output, wire) = guidance156_cli_run(
            "wake158b_cli_g7_formes_fermees_et_identifiant_retry",
            &["agent-connu", "--label", "libellé", "--id", id],
        );
        assert_eq!(output.status.code(), Some(exit_code), "{id}: {output:?}");
        let wire: serde_json::Value =
            serde_json::from_str(&wire.expect("WakeRequest réellement reçu")).unwrap();
        assert_eq!(
            wire,
            serde_json::json!({"type":"WakeRequest","request":{"version":1,"request":{
                "action":"cli_once","target":"agent-connu","label":"libellé","id":id
            }}}),
            "aucune autorité client ajoutée"
        );
    }
    for (id, exit_code) in [
        ("attested-thread158b", 0),
        ("missing-context158b", 2),
        ("invalid-context158b", 1),
    ] {
        let (output, proof) = guidance156_cli_run(
            "wake158b_cli_g7_formes_fermees_et_identifiant_retry",
            &["fil-résolu-dans-enfant", "--label", "headless", "--id", id],
        );
        assert_eq!(output.status.code(), Some(exit_code), "{id}: {output:?}");
        let proof: serde_json::Value =
            serde_json::from_str(&proof.expect("preuve SQL privée de la CLI réelle")).unwrap();
        assert_eq!(proof["id"], id);
        assert_eq!(
            proof["reservations"],
            if id == "attested-thread158b" { 1 } else { 0 }
        );
        if id == "attested-thread158b" {
            assert_eq!(proof["canonical_owner"], proof["owner"]);
            assert_eq!(
                proof["canonical_target"],
                serde_json::json!({"kind":"thread","id":proof["thread_id"]})
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("prepared"));
        } else {
            assert_eq!(
                proof["posts"], 0,
                "contexte absent/invalide : aucun post102"
            );
        }
    }
}

// Recette cli_arguments_integration_test::run_cli_as/capture_one_message,
// adaptée au testlib par current_exe (core_089_isolation_test).
// Trois usages : cmd_send sort du processus, aucun parseur factice requis.
fn guidance156_cli_run(test: &str, extra: &[&str]) -> (Output, Option<String>) {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!(
        "g156-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    for path in [
        &root,
        &root.join("ns"),
        &root.join("home"),
        &root.join("tmp"),
    ] {
        crate::environment::ensure_private_directory(path).unwrap();
    }
    let socket = root.join("ns/bridget.sock");
    let queue_parse_only = matches!(
        test,
        "guidance156_cli_queue_parse" | "control158_delegations_cli_parse"
    );
    let information = extra
        .windows(2)
        .any(|pair| pair == ["--intent", "information"]);
    let wake_cli = test == "wake158b_cli_g7_formes_fermees_et_identifiant_retry";
    let wake_id = extra
        .windows(2)
        .find(|pair| pair[0] == "--id")
        .map(|pair| pair[1].to_string())
        .unwrap_or_default();
    let real_wake_cli = wake_cli
        && matches!(
            wake_id.as_str(),
            "attested-thread158b" | "missing-context158b" | "invalid-context158b"
        );
    let queue_parse_only = queue_parse_only || real_wake_cli;
    let receipt = root.join("real-cli-receipt.json");
    let listener = UnixListener::bind(&socket).unwrap();
    // Le harness ne passe pas par initialize_process : socket privée sur Linux aussi.
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    if queue_parse_only {
        std::fs::remove_file(&socket).unwrap();
    }
    let (stop, stopped) = mpsc::channel();
    let server_wake_id = wake_id.clone();
    let server = std::thread::spawn(move || {
        if queue_parse_only {
            return None;
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break Some(stream),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if stopped.try_recv().is_ok() || Instant::now() >= deadline {
                        break None;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("socket privée : {error}"),
            }
        };
        let stream = stream?;
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        assert!(matches!(
            read_control_message_for_guidance156(&mut reader),
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            }
        ));
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        let hello = read_control_message_for_guidance156(&mut reader);
        if wake_cli {
            assert!(
                matches!(hello, WrapperToDaemon::ClientHello { contract_version, issuer_scope, capabilities }
                    if contract_version == CLIENT_CONTRACT_VERSION && !issuer_scope.is_empty() && capabilities.is_empty())
            );
        } else {
            assert!(
                matches!(hello, WrapperToDaemon::ClientHello { capabilities, .. }
                    if capabilities == if information {vec![ClientCapability::SendIdempotent,ClientCapability::SendInformationV1]} else {vec![ClientCapability::SendIdempotent]})
            );
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                build_id: "test-build".into(),
                horizon_secs: 60,
                issued_at_tolerance_secs: 5,
                capabilities: if wake_cli {
                    Vec::new()
                } else if information {
                    vec![
                        ClientCapability::SendIdempotent,
                        ClientCapability::SendInformationV1,
                    ]
                } else {
                    vec![ClientCapability::SendIdempotent]
                },
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        let sent = read_control_message_for_guidance156(&mut reader);
        if wake_cli {
            assert!(
                matches!(&sent, WrapperToDaemon::WakeRequest { request } if request.version == 1)
            );
            let wire = encode(&sent).unwrap();
            // EOF après réception est une erreur technique, jamais un succès.
            if server_wake_id != "transport158b" {
                let result = match server_wake_id.as_str() {
                    "refused158b" => {
                        serde_json::json!({"status":"refused","code":"rate_limited"})
                    }
                    "error158b" => {
                        serde_json::json!({"status":"error","code":"storage_unavailable"})
                    }
                    _ => serde_json::json!({"status":"accepted","id":server_wake_id}),
                };
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::WakeResult {
                        result: bridget_transport::protocol::WakeResult { version: 1, result },
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
            }
            return Some(wire);
        }
        assert!(matches!(&sent, WrapperToDaemon::SendIdempotent { .. }));
        let wire = encode(&sent).unwrap();
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::IdempotencyResult {
                operation_kind: "send".into(),
                idempotency_key: "guidance156-cli".into(),
                issue: IdempotencyIssue::Accepted { expires_at: 60 },
                project_warnings: Vec::new(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        Some(wire)
    });
    let mut args: Vec<String> = if queue_parse_only || wake_cli {
        Vec::new()
    } else {
        [
            "--to",
            "89000000-0000-4000-8000-000000000157",
            "--from",
            "human",
            "--id",
            "guidance156-cli",
            "--issued-at",
            "1700000000",
            "--issuer-scope",
            "guidance156-cli",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    };
    args.extend(extra.iter().map(|argument| argument.to_string()));
    if !queue_parse_only && !wake_cli {
        args.extend(["--".into(), "octets $VAR intacts".into()]);
    }
    let mut child = Command::new(std::env::current_exe().unwrap());
    child.env_clear();
    if real_wake_cli {
        child.env("BRIDGET_GUIDANCE156_CLI_REAL_RECEIPT", &receipt);
    }
    let output = child
        .args([
            "--exact",
            &format!("cli::guidance156_tests::{test}"),
            "--nocapture",
        ])
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env("BRIDGET_HOME", root.join("ns"))
        .env("BRIDGET_SOCKET", &socket)
        .env("PATH", "/usr/bin:/bin")
        .env(
            "BRIDGET_GUIDANCE156_CLI_ARGS",
            serde_json::to_string(&args).unwrap(),
        )
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let _ = stop.send(());
    let mut wire = server.join().unwrap();
    if real_wake_cli {
        let mut proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&receipt).unwrap()).unwrap();
        let db = PathBuf::from(proof["db"].as_str().unwrap());
        let connection =
            rusqlite::Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        let reservations:i64 = connection.query_row(
                "SELECT count(*) FROM idempotency_records WHERE issuer_scope=?1 AND operation_kind='send' AND idempotency_key=?2",
                rusqlite::params![crate::wake_schedules::CLI_WAKE_SCOPE, wake_id], |r|r.get(0)).unwrap();
        let posts: i64 = connection
            .query_row(
                "SELECT count(*) FROM discussion_entries WHERE thread_id=?1",
                [proof["thread_id"].as_str().unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        proof["reservations"] = reservations.into();
        proof["posts"] = posts.into();
        if reservations == 1 {
            let bytes:Vec<u8> = connection.query_row("SELECT canonical_bytes FROM idempotency_records WHERE issuer_scope=?1 AND idempotency_key=?2",
                    rusqlite::params![crate::wake_schedules::CLI_WAKE_SCOPE, wake_id], |r|r.get(0)).unwrap();
            let canon: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            proof["canonical_owner"] = canon["owner"].clone();
            proof["canonical_target"] = canon["target"].clone();
            assert!(
                canon["thread_operation_id"]
                    .as_str()
                    .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
            );
        }
        drop(connection);
        let daemon_root = db.parent().unwrap();
        assert!(
            daemon_root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("bgcanon-")
        );
        std::fs::remove_dir_all(daemon_root).unwrap();
        wire = Some(proof.to_string());
    }
    std::fs::remove_dir_all(root).unwrap();
    (output, wire)
}

fn read_control_message_for_guidance156(reader: &mut BufReader<UnixStream>) -> WrapperToDaemon {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    decode(line.trim_end()).unwrap()
}

#[test]
fn guidance156_cli_intent_valide() {
    if let Ok(args) = std::env::var("BRIDGET_GUIDANCE156_CLI_ARGS") {
        cmd_send(&serde_json::from_str::<Vec<String>>(&args).unwrap());
        return;
    }
    for (intent, expected) in [
        ("queue", None),
        ("information", Some(bridget_core::MessageIntent::StoreOnly)),
        ("steer", Some(bridget_core::MessageIntent::SteerCurrent)),
        (
            "interrupt",
            Some(bridget_core::MessageIntent::InterruptAndStart),
        ),
    ] {
        let (output, wire) =
            guidance156_cli_run("guidance156_cli_intent_valide", &["--intent", intent]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let WrapperToDaemon::SendIdempotent { message, .. } = decode(&wire.unwrap()).unwrap()
        else {
            panic!("SendIdempotent")
        };
        assert_eq!(message.intent, expected);
        if intent == "information" {
            assert!(String::from_utf8_lossy(&output.stdout).contains("STOCKÉ SANS RÉVEIL"));
        }
    }
}

#[test]
fn guidance156_cli_intent_inconnue() {
    if let Ok(args) = std::env::var("BRIDGET_GUIDANCE156_CLI_ARGS") {
        cmd_send(&serde_json::from_str::<Vec<String>>(&args).unwrap());
        return;
    }
    let (output, wire) =
        guidance156_cli_run("guidance156_cli_intent_inconnue", &["--intent", "auto"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid_intent"));
    assert!(wire.is_none(), "valeur inconnue refusée avant connexion");
}

#[test]
fn guidance156_cli_sans_intent_inchange() {
    if let Ok(args) = std::env::var("BRIDGET_GUIDANCE156_CLI_ARGS") {
        cmd_send(&serde_json::from_str::<Vec<String>>(&args).unwrap());
        return;
    }
    let (output, wire) = guidance156_cli_run("guidance156_cli_sans_intent_inchange", &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut message = BridgetMessage::new(
        "human",
        "89000000-0000-4000-8000-000000000157",
        "octets $VAR intacts",
    );
    message.id = "guidance156-cli".into();
    message.from_declared = true;
    let expected = encode(&WrapperToDaemon::SendIdempotent {
        message,
        message_id: "guidance156-cli".into(),
        issued_at: 1_700_000_000,
    })
    .unwrap();
    assert_eq!(
        wire.unwrap(),
        expected,
        "trame historique complète inchangée"
    );
}

#[test]
fn edit159_cli_parse() {
    let list = ["--target", "agent"].map(str::to_owned).to_vec();
    let parsed = parse_queue_args(&list).unwrap();
    assert_eq!(
        serde_json::to_value(parsed).unwrap(),
        serde_json::json!({"type":"QueueList","target":"agent"})
    );
    for action in ["cancel", "reorder", "promote"] {
        let mut args = [
            action,
            "--target",
            "agent",
            "--message",
            "m159",
            "--request",
            "r159",
        ]
        .map(str::to_owned)
        .to_vec();
        if action == "reorder" {
            args.extend(["--before".into(), "avant159".into()]);
        }
        let parsed = parse_queue_args(&args).unwrap();
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            serde_json::json!({
                "type":"QueueEditRequest","request_id":"r159","target":"agent","action":action,
                "message_id":"m159","before_message_id":if action=="reorder" {Some("avant159")} else {None}
            })
        );
        let missing_id = [action, "--target", "agent", "--message", "m159"]
            .map(str::to_owned)
            .to_vec();
        assert!(parse_queue_args(&missing_id).is_err());
    }
    let end = [
        "reorder",
        "--target",
        "agent",
        "--message",
        "m159",
        "--request",
        "r159",
    ]
    .map(str::to_owned)
    .to_vec();
    assert_eq!(
        serde_json::to_value(parse_queue_args(&end).unwrap()).unwrap()["before_message_id"],
        serde_json::Value::Null
    );
    for args in [
        vec![
            "cancel",
            "--target",
            "agent",
            "--message",
            "m159",
            "--request",
            "",
        ],
        vec![
            "cancel",
            "--target",
            "agent",
            "--message",
            "m159",
            "--request",
            "r159",
            "--before",
            "avant159",
        ],
        vec![
            "promote",
            "--target",
            "agent",
            "--message",
            "m159",
            "--request",
            "r159",
            "--unknown",
            "x",
        ],
    ] {
        assert!(
            parse_queue_args(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err()
        );
    }
}

#[test]
fn guidance156_cli_queue_parse() {
    if let Ok(args) = std::env::var("BRIDGET_GUIDANCE156_CLI_ARGS") {
        let mut command = vec!["bridget".to_string()];
        command.extend(serde_json::from_str::<Vec<String>>(&args).unwrap());
        run_with_args(command);
        return;
    }
    let (accepted, wire) = guidance156_cli_run(
        "guidance156_cli_queue_parse",
        &["queue", "--target", "89000000-0000-4000-8000-000000000157"],
    );
    assert_eq!(
        accepted.status.code(),
        Some(1),
        "arguments acceptés puis socket privée absente : {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert!(String::from_utf8_lossy(&accepted.stderr).contains("daemon_unreachable"));
    assert!(
        wire.is_none(),
        "aucun daemon n'est lancé pour ce test de parse"
    );
    for args in [
        &["queue"][..],
        &["queue", "--target", ""][..],
        &["queue", "--target", "cible", "--intent", "queue"][..],
    ] {
        let (rejected, wire) = guidance156_cli_run("guidance156_cli_queue_parse", args);
        assert_eq!(rejected.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("queue : attendu --target"));
        assert!(wire.is_none());
    }
}

#[test]
fn control158_delegations_cli_parse() {
    if let Ok(args) = std::env::var("BRIDGET_GUIDANCE156_CLI_ARGS") {
        let mut command = vec!["bridget".to_string()];
        command.extend(serde_json::from_str::<Vec<String>>(&args).unwrap());
        run_with_args(command);
        return;
    }
    let defaults = vec!["delegations".to_string()];
    assert!(
        matches!(parse_delegation_control(&defaults), Ok((WrapperToDaemon::DelegationOverviewRead { version:1, ref state, limit:50 }, false)) if state == "active")
    );
    let list: Vec<String> = [
        "delegations",
        "list",
        "--state",
        "all",
        "--limit",
        "100",
        "--json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert!(
        matches!(parse_delegation_control(&list), Ok((WrapperToDaemon::DelegationOverviewRead { version:1, ref state, limit:100 }, true)) if state == "all")
    );
    for args in [
        &["control", "delegations", "--limit", "0"][..],
        &["control", "delegations", "--limit", "101"][..],
        &["control", "delegations", "--inconnu"][..],
        &["control", "delegations", "cancel", "pas-un-uuid"][..],
    ] {
        let (output, wire) = guidance156_cli_run("control158_delegations_cli_parse", args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?} : {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(wire.is_none(), "usage refusé avant ouverture de socket");
    }
}

#[test]
fn control158_halt_aucun_outil_mcp() {
    let tools = crate::mcp::tools();
    let controls: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .filter(|name| {
            name.starts_with("bridget_control")
                || name.contains("halt_delegations")
                || name.contains("resume_delegations")
        })
        .collect();
    assert_eq!(
        controls,
        ["bridget_control_status"],
        "seule la lecture du contrôle humain est exposée par MCP"
    );
}
