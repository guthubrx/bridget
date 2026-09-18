//! Les exemples publiés sont les entrées du vrai MCP, pas leur réécriture locale.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn examples() -> Vec<Value> {
    include_str!("../../../skills/bridget/SKILL.md")
        .split("```json\n")
        .skip(1)
        .map(|block| serde_json::from_str(block.split("```").next().unwrap()).unwrap())
        .collect()
}

fn initialize(mcp: &mut McpProcess) {
    let result = mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert_eq!(result["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
}

fn call(mcp: &mut McpProcess, example: &Value) -> Value {
    let result =
        mcp.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":example}));
    assert_ne!(result["result"]["isError"], true, "{result}");
    let structured = result["result"]["structuredContent"].clone();
    assert_eq!(
        serde_json::from_str::<Value>(result["result"]["content"][0]["text"].as_str().unwrap())
            .unwrap(),
        structured,
        "les deux formes du reçu doivent être identiques"
    );
    structured
}

fn deliver_and_retry(mcp: &mut McpProcess, peer: &mut Client, example: &Value) -> Value {
    let receipt = call(mcp, example);
    assert_eq!(receipt["status"], "in_flight");
    let id = receipt["id"].as_str().expect("id rejouable avant ACK");
    match receive_delivery(peer) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            message,
            ..
        } => {
            assert_eq!(message.id, id);
            assert_eq!(message.body, example["arguments"]["body"]);
            peer.send(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            });
        }
        other => panic!("remise attendue : {other:?}"),
    }
    let mut retry = example.clone();
    retry["arguments"]["id"] = receipt["id"].clone();
    retry["arguments"]["issued_at"] = receipt["issued_at"].clone();
    let deadline = Instant::now() + Duration::from_secs(5);
    let accepted = loop {
        let result = call(mcp, &retry);
        if result["status"] == "accepted" {
            break result;
        }
        assert_eq!(result["status"], "in_flight", "{result}");
        assert!(Instant::now() < deadline, "ACK absent : {result}");
    };
    assert_eq!(call(mcp, &retry), accepted, "rejeu terminal exact");
    accepted
}

#[test]
fn exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon() {
    let root = test_root("089-skill");
    let daemon = spawn_daemon(&root, None);
    let mut actor = register_agent_as(&socket(&root), ACTOR, "089-skill-a");
    let mut recipient = register_recipient_as(&socket(&root), "089-skill-b");
    let mut examples = examples();
    assert_eq!(examples.len(), 4, "chaque exemple doit être exécuté");
    let mut mcp = McpProcess::start(&root, ACTOR, "089-skill-a");
    initialize(&mut mcp);
    let directory = call(&mut mcp, &examples[0]);
    assert!(directory.to_string().contains(RECIPIENT));
    examples[1]["arguments"]["to"] = RECIPIENT.into();
    let question = deliver_and_retry(&mut mcp, &mut recipient, &examples[1]);
    let initial = call(&mut mcp, &examples[3]);
    assert_eq!(initial["requests"][0]["state"], "open");
    mcp.stop();

    // Deuxième session, instance propre : seul l'identifiant reçu fait le lien.
    let mut mcp = McpProcess::start(&root, RECIPIENT, "089-skill-b");
    initialize(&mut mcp);
    examples[2]["arguments"]["to"] = ACTOR.into();
    if examples[2]["arguments"].get("in_reply_to").is_some() {
        examples[2]["arguments"]["in_reply_to"] = question["id"].clone();
    }
    let answer = deliver_and_retry(&mut mcp, &mut actor, &examples[2]);
    let ledger = call(&mut mcp, &examples[3]);
    // Mutant : supprimer in_reply_to DANS LA SKILL garde open, donc casse ici.
    assert_eq!(ledger["requests"][0]["state"], "answered");
    let messages = ledger["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2, "aucune deuxième injection au retry");
    for receipt in [&question, &answer] {
        assert_eq!(
            messages.iter().filter(|m| m["id"] == receipt["id"]).count(),
            1
        );
    }
    assert_no_delivery(&mut actor);
    assert_no_delivery(&mut recipient);
    let cli = run_isolated(&root, &["ledger", "--limit", "20"], false);
    assert!(cli.status.success(), "{}", output_text(&cli));
    let text = String::from_utf8(cli.stdout).unwrap();
    for example in [&examples[1], &examples[2]] {
        assert!(text.contains(example["arguments"]["body"].as_str().unwrap()));
    }
    mcp.stop();
    drop(actor);
    drop(recipient);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
