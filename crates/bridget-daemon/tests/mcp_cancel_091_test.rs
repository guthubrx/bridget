#![cfg(feature = "test-support")]
//! Annulation MCP réelle : autorité du daemon, identité et reçu durable.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::{Value, json};
use std::path::Path;

fn mcp(root: &Path, agent: &str, instance: &str) -> McpProcess {
    let mut process = McpProcess::start(root, agent, instance);
    let init = process.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert!(init.get("result").is_some());
    process.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    process
}

fn cancel(process: &mut McpProcess, arguments: Value) -> Value {
    process.request(
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
            "name":"bridget_cancel", "arguments": arguments
        }}),
    )
}

fn state(root: &Path) -> String {
    let mut observer = Client::connect(&socket(root));
    observer.send(WrapperToDaemon::ListRequests {
        sender: ACTOR.into(),
        limit: 10,
    });
    match observer.receive() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), 1);
            requests[0].state.clone()
        }
        other => panic!("demande attendue : {other:?}"),
    }
}

#[test]
fn annulation_mcp_verifie_identite_refuse_usurpation_et_persiste_le_recu() {
    let root = test_root("091-mcp-cancel");
    let daemon = spawn_daemon(&root, None);
    let mut sender = register_agent_as(&socket(&root), ACTOR, "091-owner");
    let mut recipient = register_recipient(&socket(&root));
    let _intruder = register_agent_as(&socket(&root), MATRIX_AGENT, "091-intruder");
    let mut request = BridgetMessage::new(ACTOR, RECIPIENT, "mission annulable");
    request.reply = true;
    sender.send(WrapperToDaemon::Send(request.clone()));
    assert!(matches!(sender.receive(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(recipient.receive(), DaemonToWrapper::Deliver(_)));
    assert_eq!(state(&root), "open");

    let mut intruder = mcp(&root, MATRIX_AGENT, "091-intruder");
    let refused = cancel(&mut intruder, json!({"id":request.id}));
    assert_eq!(refused["result"]["structuredContent"]["status"], "rejected");
    assert!(
        refused["result"]["structuredContent"]["reason"]
            .as_str()
            .unwrap()
            .contains("seul l'émetteur")
    );
    let spoof = cancel(&mut intruder, json!({"id":request.id, "sender":ACTOR}));
    assert_eq!(spoof["error"]["code"], -32602);
    assert_eq!(state(&root), "open");
    intruder.stop();

    let mut wrong_instance = mcp(&root, ACTOR, "091-foreign-instance");
    let refused = cancel(&mut wrong_instance, json!({"id":request.id}));
    assert_eq!(refused["result"]["isError"], true);
    assert_eq!(state(&root), "open");
    wrong_instance.stop();

    let mut owner = mcp(&root, ACTOR, "091-owner");
    let catalogue = owner.request(json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}));
    assert!(
        catalogue["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "bridget_cancel")
    );
    for _ in 0..2 {
        let receipt = cancel(
            &mut owner,
            json!({"id":request.id, "reason":"priorité changée"}),
        );
        assert_eq!(
            receipt["result"]["structuredContent"],
            json!({"id":request.id,"status":"cancelled"})
        );
        assert_eq!(state(&root), "cancelled");
    }
    owner.stop();
    drop((sender, recipient, _intruder));
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
