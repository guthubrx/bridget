#![cfg(feature = "test-support")]
//! CLI et MCP binaires, client public indépendant, un seul daemon réel.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::Path;

// Valeur figée du corpus 089 pour instance-stable-089, pas calculée par le
// helper de production que ce test veut protéger.
const CONTRACT_SCOPE: &str = "012_scope_37419e46a921ee300d273a45c16ebe1a";
const INSTANCE: &str = "instance-stable-089";
const ID: &str = "core-089-canonical";
const LINK: &str = "core-089-linked-request";
const BODY: &str = "  réponse UTF-8 : été 🐙\n{\"texte\":\"espaces  conservés\"}\nfin  ";

fn reference_client(root: &Path) -> Client {
    let owner = std::sync::Arc::new(register_agent_as(&socket(root), ACTOR, INSTANCE));
    let mut client = Client::connect(&socket(root));
    client.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    client.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: CONTRACT_SCOPE.into(),
        capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    attest_agent(&socket(root), &mut client, ACTOR, INSTANCE);
    client.owner = Some(owner);
    client
}

fn issue(client: &mut Client, message: &BridgetMessage, timestamp: i64) -> IdempotencyIssue {
    client.send(WrapperToDaemon::SendIdempotent {
        message: message.clone(),
        message_id: ID.into(),
        issued_at: timestamp,
    });
    match client.receive() {
        DaemonToWrapper::IdempotencyResult { issue, .. } => issue,
        other => panic!("issue attendue, reçu {other:?}"),
    }
}

fn snapshot(database: &Path) -> Vec<Vec<Vec<SqlValue>>> {
    let conn = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    // Tous les champs, pas seulement le statut habituellement regardé.
    [
        "idempotency_records",
        "send_deliveries",
        "tracked_requests",
        "ledger",
    ]
    .map(|table| {
        let mut stmt = conn
            .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
            .unwrap();
        let columns = stmt.column_count();
        stmt.query_map([], |row| (0..columns).map(|i| row.get(i)).collect())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    })
    .to_vec()
}

fn canonical_bytes(database: &Path) -> Vec<u8> {
    Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap()
        .query_row("SELECT canonical_bytes FROM idempotency_records WHERE issuer_scope=?1 AND idempotency_key=?2", [CONTRACT_SCOPE, ID], |row| row.get(0)).unwrap()
}

fn tool_call(rpc_id: u64, timestamp: i64, arguments: Value) -> Value {
    let mut base = json!({"to": RECIPIENT, "body": BODY, "in_reply_to": LINK, "id": ID, "issued_at": timestamp});
    for (key, value) in arguments.as_object().unwrap() {
        base[key] = value.clone();
    }
    json!({"jsonrpc":"2.0", "id":rpc_id, "method":"tools/call", "params":{"name":"bridget_send", "arguments":base}})
}

fn assert_tool_status(response: &Value, status: &str) {
    assert!(response.get("error").is_none(), "{response}");
    let result = &response["result"];
    assert_eq!(result["structuredContent"]["status"], status, "{response}");
    let text: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        text, result["structuredContent"],
        "même résultat sur les deux projections MCP"
    );
}

#[test]
fn client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation() {
    let root = test_root("core-contract");
    let db = root.join("state/bridget.db");
    let store = fixture_store(&db).unwrap();
    store.create_request(LINK, RECIPIENT, ACTOR, 120).unwrap();
    drop(store);
    let daemon = spawn_daemon(&root, None);
    let mut recipient = register_recipient_as(&socket(&root), "core-recipient");
    let timestamp = issued_at();
    let mut message = BridgetMessage::new(ACTOR, RECIPIENT, BODY);
    message.id = ID.into();
    message.in_reply_to = Some(LINK.into());
    let mut reference = reference_client(&root);
    assert!(matches!(
        issue(&mut reference, &message, timestamp),
        IdempotencyIssue::OutcomeUnknown {
            delivery_id: Some(_),
            ..
        }
    ));
    match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            message: delivered,
            delivery_id,
            delivery_generation,
            ..
        } => {
            assert_eq!(delivered.body.as_bytes(), BODY.as_bytes());
            assert_eq!(delivered.in_reply_to.as_deref(), Some(LINK));
            recipient.send(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            });
        }
        other => panic!("remise attendue : {other:?}"),
    }
    // Barrière de protocole sur LA connexion qui a accusé, pas une attente
    // calée sur la vitesse supposée de SQLite.
    recipient.send(WrapperToDaemon::ListAgents);
    assert!(matches!(
        recipient.receive(),
        DaemonToWrapper::AgentList { .. }
    ));
    let expected = snapshot(&db);
    let canon = canonical_bytes(&db);
    assert!(canon.starts_with(b"bridget/client-send/v1\0"));
    assert!(matches!(
        issue(&mut reference, &message, timestamp),
        IdempotencyIssue::Accepted { .. }
    ));
    let time = timestamp.to_string();
    let cli = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            RECIPIENT,
            "--in-reply-to",
            LINK,
            "--id",
            ID,
            "--issued-at",
            &time,
            "--issuer-scope",
            CONTRACT_SCOPE,
            "--",
            BODY,
        ],
    );
    assert!(cli.status.success(), "{}", output_text(&cli));
    assert!(output_text(&cli).contains("accepted"));
    assert_eq!(canonical_bytes(&db), canon);
    assert_eq!(snapshot(&db), expected);

    let mut mcp = McpProcess::start(&root, ACTOR, INSTANCE);
    let initialized =
        mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    assert_tool_status(&mcp.request(tool_call(2, timestamp, json!({}))), "accepted");
    assert_eq!(canonical_bytes(&db), canon);
    assert_eq!(snapshot(&db), expected);
    assert_no_delivery(&mut recipient);

    // Chaque mutant traverse la vraie admission : l'issue terminale ne doit
    // PAS court-circuiter la comparaison du canon, ni altérer le record.
    for (label, changed) in [
        (
            "body",
            BridgetMessage {
                body: "divergent".into(),
                ..message.clone()
            },
        ),
        (
            "target",
            BridgetMessage {
                to: MATRIX_AGENT.into(),
                ..message.clone()
            },
        ),
        (
            "reply",
            BridgetMessage {
                reply: true,
                ..message.clone()
            },
        ),
        (
            "deadline",
            BridgetMessage {
                deadline_at: Some(timestamp as u64 + 60),
                ..message.clone()
            },
        ),
        (
            "in_reply_to",
            BridgetMessage {
                in_reply_to: Some("another-request".into()),
                ..message.clone()
            },
        ),
    ] {
        assert!(
            matches!(
                issue(&mut reference, &changed, timestamp),
                IdempotencyIssue::EnvelopeMismatch
            ),
            "{label}"
        );
        assert_eq!(snapshot(&db), expected, "mutation durable après {label}");
        assert_eq!(canonical_bytes(&db), canon, "canon réécrit après {label}");
    }
    for (index, changed) in [
        json!({"body":"divergent"}),
        json!({"to":MATRIX_AGENT}),
        json!({"reply":true}),
        json!({"in_reply_to":"another-request"}),
    ]
    .into_iter()
    .enumerate()
    {
        assert_tool_status(
            &mcp.request(tool_call(index as u64 + 3, timestamp, changed)),
            "envelope_mismatch",
        );
        assert_eq!(snapshot(&db), expected);
    }
    for (target, body, link, reply) in [
        (RECIPIENT, "divergent", LINK, false),
        (MATRIX_AGENT, BODY, LINK, false),
        (RECIPIENT, BODY, "another-request", false),
        (RECIPIENT, BODY, LINK, true),
    ] {
        let mut args = vec![
            "send",
            "--to",
            target,
            "--in-reply-to",
            link,
            "--id",
            ID,
            "--issued-at",
            &time,
            "--issuer-scope",
            CONTRACT_SCOPE,
        ];
        if reply {
            args.push("--reply");
        }
        args.extend(["--", body]);
        let output = run_linked_cli(&root, &args);
        assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
        assert!(
            output_text(&output).contains("envelope_mismatch"),
            "{}",
            output_text(&output)
        );
        assert_eq!(snapshot(&db), expected);
    }
    assert!(matches!(
        issue(&mut reference, &message, timestamp + 1),
        IdempotencyIssue::EnvelopeMismatch
    ));
    assert_tool_status(
        &mcp.request(tool_call(99, timestamp + 1, json!({}))),
        "envelope_mismatch",
    );
    assert_eq!(snapshot(&db), expected);
    assert_no_delivery(&mut recipient);
    mcp.stop();
    drop(reference);
    drop(recipient);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn six_formes_partielles_cli_refusees_avant_la_socket() {
    let root = test_root("core-partial-options");
    let listener = UnixListener::bind(socket(&root)).unwrap();
    // La racine est déjà 0700 ; la sentinelle doit satisfaire le préflight
    // du namespace pour que l'oracle atteigne vraiment les options partielles.
    fs::set_permissions(socket(&root), fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let timestamp = issued_at().to_string();
    let options = [
        ["--id", ID],
        ["--issued-at", timestamp.as_str()],
        ["--issuer-scope", CONTRACT_SCOPE],
    ];
    for mask in 1_u8..7 {
        let mut args = vec!["send", "--to", RECIPIENT];
        for (index, pair) in options.iter().enumerate() {
            if mask & (1 << index) != 0 {
                args.extend(pair);
            }
        }
        args.extend(["--", BODY]);
        let result = run_linked_cli(&root, &args);
        assert_eq!(
            result.status.code(),
            Some(2),
            "mask={mask}: {}",
            output_text(&result)
        );
        assert!(
            output_text(&result).contains("obligatoires ensemble"),
            "mask={mask}: {}",
            output_text(&result)
        );
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
        assert!(!root.join("state/bridget.db").exists());
    }
    drop(listener);
    fs::remove_dir_all(root).unwrap();
}

fn register_identity(
    socket: &Path,
    client: &mut Client,
    id: &str,
    kind: &str,
    instance: Option<&str>,
) {
    client.send(WrapperToDaemon::Register {
        agent_type: kind.into(),
        identity_version: 2,
        agent_id: id.into(),
        host: Some("core-contract-test".into()),
        transport: Some("unix".into()),
        channel: None.into(),
        mode: None,
        location: None,
        os: None,
        instance_id: instance.map(str::to_string),
        domain: None,
        turn_in_progress: false,
        journal_available: None,
    });
    let DaemonToWrapper::Registered { credential, .. } = client.receive() else {
        panic!("inscription attendue");
    };
    if let Some(instance) = instance {
        save_fixture_credential(
            socket,
            id,
            instance,
            credential.expect("preuve du propriétaire"),
        );
    }
}

#[test]
fn cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere() {
    let root = test_root("core-sender");
    let daemon = spawn_daemon(&root, None);
    let mut actor = Client::connect(&socket(&root));
    register_identity(
        &socket(&root),
        &mut actor,
        ACTOR,
        "codex",
        Some("actual-actor"),
    );
    let mut recipient = register_recipient_as(&socket(&root), "actual-recipient");
    let output = run_linked_cli(&root, &["send", "--to", RECIPIENT, "message hérité"]);
    assert!(output.status.success(), "{}", output_text(&output));
    match recipient.receive() {
        DaemonToWrapper::Deliver(message) => assert_eq!(message.from, ACTOR),
        other => panic!("remise historique attendue : {other:?}"),
    }
    for claimed in [ACTOR, MATRIX_AGENT] {
        let refused = run_isolated(
            &root,
            &[
                "send",
                "--from",
                claimed,
                "--to",
                RECIPIENT,
                "tentative explicite",
            ],
            false,
        );
        assert_eq!(refused.status.code(), Some(1), "{}", output_text(&refused));
        assert!(
            output_text(&refused).contains("identité expéditeur non attestée"),
            "{}",
            output_text(&refused)
        );
    }
    // Le CLI temporaire est lui-même adressable pendant son appel, mais ne
    // peut promettre une réponse future. Mutant : ancien test de préfixe UUID
    // toujours faux => Ack, nouvelle demande suivie et seconde livraison.
    let mut temporary = Client::connect(&socket(&root));
    register_identity(&socket(&root), &mut temporary, MATRIX_AGENT, "cli", None);
    let mut message = BridgetMessage::new(MATRIX_AGENT, RECIPIENT, "réponse impossible");
    message.reply = true;
    let message_id = message.id.clone();
    temporary.send(WrapperToDaemon::Send(message));
    assert!(matches!(temporary.receive(), DaemonToWrapper::Nack { id, .. } if id == message_id));
    let conn = Connection::open_with_flags(
        root.join("state/bridget.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let requests: i64 = conn
        .query_row("SELECT COUNT(*) FROM tracked_requests", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(requests, 0);
    assert_no_delivery(&mut recipient);
    drop((actor, temporary, recipient, conn));
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}
