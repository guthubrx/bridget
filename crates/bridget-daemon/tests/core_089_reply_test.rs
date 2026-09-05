#![cfg(feature = "test-support")]
//! Cycle suivi par les vrais CLI/MCP/daemon ; pairs publics sans fournisseur.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::json;
use std::path::Path;
use std::time::{Duration, Instant};

fn ack_delivery(peer: &mut Client, expected_id: &str) {
    match receive_delivery(peer) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            message,
            ..
        } => {
            assert_eq!(message.id, expected_id);
            peer.send(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            });
        }
        _ => unreachable!(),
    }
}

fn request_cli(root: &Path, id: &str, timestamp: &str) {
    let output = run_linked_cli(
        root,
        &[
            "send",
            "--to",
            RECIPIENT,
            "--reply",
            "--timeout",
            "6",
            "--id",
            id,
            "--issued-at",
            timestamp,
            "--issuer-scope",
            SCOPE,
            &format!("question {id} UTF-8 : été 🐙"),
        ],
    );
    assert!(output.status.success(), "{}", output_text(&output));
}

fn requests(client: &mut Client) -> Vec<bridget_transport::protocol::RequestInfo> {
    client.send(WrapperToDaemon::ListRequests {
        sender: ACTOR.into(),
        limit: 100,
    });
    match client.receive() {
        DaemonToWrapper::RequestList { requests } => requests,
        other => panic!("projection de demandes attendue : {other:?}"),
    }
}

#[test]
fn cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent() {
    let root = test_root("089-reply");
    let daemon = spawn_daemon(&root, None);
    let mut actor = register_agent_as(&socket(&root), ACTOR, "shared-cli-mcp-instance");
    let mut recipient = register_recipient_as(&socket(&root), "089-reply-recipient");
    let mut observer = Client::connect(&socket(&root));
    let timestamp = issued_at();
    let date = timestamp.to_string();
    request_cli(&root, "089-answer-me", &date);
    ack_delivery(&mut recipient, "089-answer-me");
    let initial = requests(&mut observer);
    let request = initial.iter().find(|r| r.id == "089-answer-me").unwrap();
    assert_eq!(request.state, "open");
    assert_eq!(request.sender, ACTOR);
    assert_eq!(request.target, RECIPIENT);
    // L'échéance est celle du canon immuable issued_at, pas l'heure de lecture.
    assert_eq!(request.deadline_at, timestamp + 6);

    let mut mcp = McpProcess::start(&root, RECIPIENT, "089-reply-recipient");
    let init = mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    let reply = json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
        "name":"bridget_send","arguments":{
            "to":ACTOR,"body":"réponse liée — fin", "in_reply_to":"089-answer-me",
            "id":"089-answer","issued_at":timestamp
        }
    }});
    let first = mcp.request(reply.clone());
    assert_eq!(first["result"]["structuredContent"]["status"], "in_flight");
    // Sans ACK, pas de réponse attestée : supprimer ce garde rend l'oracle rouge.
    assert_eq!(
        requests(&mut observer)
            .iter()
            .find(|r| r.id == "089-answer-me")
            .unwrap()
            .state,
        "open"
    );
    ack_delivery(&mut actor, "089-answer");
    let deadline = Instant::now() + Duration::from_secs(5);
    let accepted = loop {
        let result = mcp.request(reply.clone());
        if result["result"]["structuredContent"]["status"] == "accepted" {
            break result;
        }
        assert!(Instant::now() < deadline, "ACK MCP non convergé : {result}");
    };
    assert_eq!(
        mcp.request(reply.clone()),
        accepted,
        "retry : même issue exacte"
    );
    assert_eq!(
        requests(&mut observer)
            .iter()
            .find(|r| r.id == "089-answer-me")
            .unwrap()
            .state,
        "answered"
    );

    request_cli(&root, "089-cancel-me", &date);
    ack_delivery(&mut recipient, "089-cancel-me");
    actor.send(WrapperToDaemon::CancelRequest {
        id: "089-cancel-me".into(),
        sender: ACTOR.into(),
        reason: Some("annulation explicite".into()),
    });
    assert!(
        matches!(actor.receive(), DaemonToWrapper::RequestCancelled { id, state } if id == "089-cancel-me" && state == "cancelled")
    );
    assert!(
        matches!(recipient.receive(), DaemonToWrapper::CancelDelivery { id, .. } if id == "089-cancel-me")
    );

    // La sentinelle traverse les VRAIS paliers et expire après les deux demandes
    // précédentes. C'est une barrière d'état, pas « dormir six secondes ».
    request_cli(&root, "089-timeout-sentinel", &date);
    ack_delivery(&mut recipient, "089-timeout-sentinel");
    actor
        .reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_secs(12)))
        .unwrap();
    assert!(matches!(actor.receive(), DaemonToWrapper::Deliver(message) if message.to == ACTOR));
    let final_requests = requests(&mut observer);
    for (id, state) in [
        ("089-answer-me", "answered"),
        ("089-cancel-me", "cancelled"),
        ("089-timeout-sentinel", "timed_out"),
    ] {
        assert_eq!(
            final_requests.iter().find(|r| r.id == id).unwrap().state,
            state
        );
    }
    let store = fixture_store(&root.join("state/bridget.db")).unwrap();
    let reminders = store.guichet_coordination_events().unwrap();
    assert!(
        !reminders.is_empty(),
        "la sentinelle doit observer les vrais rappels"
    );
    // Mutant : conserver le pending répondu/annulé => rappel supplémentaire
    // durable après la clôture, alors que la sentinelle continue de progresser.
    assert!(
        reminders
            .iter()
            .all(|event| event.request_id == "089-timeout-sentinel"),
        "rappel après clôture : {reminders:?}"
    );
    let ledger = store.recent_messages(100).unwrap();
    for id in [
        "089-answer-me",
        "089-answer",
        "089-cancel-me",
        "089-timeout-sentinel",
    ] {
        assert_eq!(
            ledger.iter().filter(|m| m.id == id).count(),
            1,
            "ledger unique {id}"
        );
    }
    let tool = mcp.request(json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"bridget_ledger","arguments":{"view":"messages","limit":100}}}));
    assert_eq!(
        tool["result"]["structuredContent"]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["id"] == "089-answer")
            .count(),
        1
    );
    let binary = run_isolated(&root, &["ledger"], false);
    assert!(binary.status.success(), "{}", output_text(&binary));
    assert!(
        String::from_utf8(binary.stdout)
            .unwrap()
            .contains("réponse liée — fin")
    );
    mcp.stop();
    drop(actor);
    drop(recipient);
    drop(observer);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
