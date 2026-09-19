//! Projections réelles CLI/MCP/daemon ; données historiques figées pour le rendu.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::protocol::LedgerScope;
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::json;

#[test]
fn daemon_absent_ne_devient_pas_un_ledger_local_vide() {
    let root = test_root("089-ledger-offline");
    let result = run_isolated(&root, &["ledger"], false);
    assert!(
        !result.status.success(),
        "absence daemon déguisée en réussite : {}",
        output_text(&result)
    );
    assert!(
        result.stdout.is_empty(),
        "aucune donnée inventée sans source"
    );
    assert!(
        !root.join("state/bridget.db").exists(),
        "la lecture distante n'initialise pas une base locale"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cli_golden_et_mcp_lisent_la_meme_projection_bornee_du_maitre() {
    let root = test_root("089-ledger");
    let database = root.join("state/bridget.db");
    let store = fixture_store(&database).unwrap();
    for (id, from, to, body) in [
        ("ancien", "alice", "bob", "premier"),
        ("recent", "bob", "alice", "corps riche $VAR\nintact"),
    ] {
        let mut message = BridgetMessage::new(from, to, body);
        message.id = id.into();
        store.record_message(&message, id).unwrap();
    }
    for (id, from, to) in [
        ("sortante", ACTOR, RECIPIENT),
        ("entrante", RECIPIENT, ACTOR),
        ("etrangere", RECIPIENT, ACP_AGENT),
    ] {
        store.create_request(id, from, to, 600).unwrap();
    }
    // Horloge de fixture fixée à 2100 : le bootstrap de rétention ne doit pas
    // effacer les deux lignes avant que leur rendu indépendant soit comparé.
    let sql = rusqlite::Connection::open(&database).unwrap();
    sql.execute(
        "UPDATE ledger SET ts=CASE id WHEN 'ancien' THEN 4102444801 ELSE 4102444802 END",
        [],
    )
    .unwrap();
    drop(sql);
    drop(store);
    let daemon = spawn_daemon(&root, None);
    let actor = register_agent_as(&socket(&root), ACTOR, "089-ledger-instance");
    let mut client = Client::connect(&socket(&root));
    client.send(WrapperToDaemon::LedgerProjection {
        scope: LedgerScope::Both,
        limit: 100,
    });
    let (messages, requests) = match client.receive() {
        DaemonToWrapper::LedgerProjection { messages, requests } => (messages, requests),
        other => panic!("projection daemon : {other:?}"),
    };
    assert_eq!(messages.len(), 2);
    assert_eq!(requests.len(), 3);
    let cli = run_isolated(&root, &["ledger"], false);
    assert!(cli.status.success(), "{}", output_text(&cli));
    assert_eq!(
        cli.stdout,
        include_bytes!("../../../fixtures/ledger-cli-golden-v1.txt")
    );
    let limited = run_isolated(&root, &["ledger", "--limit", "1"], false);
    assert!(limited.status.success());
    assert_eq!(
        String::from_utf8(limited.stdout).unwrap(),
        "Derniers 1 messages :\n  [4102444802] bob → alice: corps riche $VAR\nintact\n… vue bornée à 1 : des messages plus anciens existent et ne sont pas montrés (élargir avec --limit N, maximum 200).\n"
    );
    let mut mcp = McpProcess::start(&root, ACTOR, "089-ledger-instance");
    mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}}));
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    let result = mcp.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_ledger","arguments":{"view":"both","limit":100,"requests_scope":"all"}}}));
    let payload = &result["result"]["structuredContent"];
    assert_eq!(
        payload["messages"],
        json!([
            {"id":"recent","from":"bob","to":"alice","body":"corps riche $VAR\nintact","ts":4102444802_i64},
            {"id":"ancien","from":"alice","to":"bob","body":"premier","ts":4102444801_i64}
        ])
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            result["result"]["content"][0]["text"].as_str().unwrap()
        )
        .unwrap(),
        *payload
    );
    assert_eq!(payload["requests"].as_array().unwrap().len(), 3);
    for request in requests {
        let dto = payload["requests"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == request.id)
            .unwrap();
        assert_eq!(
            dto,
            &json!({"id":request.id,"from":request.sender,"to":request.target,"created":request.created_at,"deadline":request.deadline_at,"state":request.state})
        );
    }
    let own = mcp.request(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bridget_ledger","arguments":{"view":"requests","limit":100}}}));
    let owned = own["result"]["structuredContent"]["requests"]
        .as_array()
        .unwrap();
    assert_eq!(
        owned.len(),
        2,
        "entrantes ET sortantes, pas les demandes étrangères"
    );
    for id in ["entrante", "sortante"] {
        assert!(owned.iter().any(|r| r["id"] == id));
    }
    let bounded = mcp.request(json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"bridget_ledger","arguments":{"view":"both","limit":1}}}));
    assert_eq!(
        bounded["result"]["structuredContent"]["messages"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        bounded["result"]["structuredContent"]["requests"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    mcp.stop();
    drop(actor);
    drop(client);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
