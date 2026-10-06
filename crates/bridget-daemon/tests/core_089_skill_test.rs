//! Les exemples publiés sont les entrées du vrai MCP, pas leur réécriture locale.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_transport::protocol::{CommunicationProjectSource, PresenceMode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn examples() -> Vec<Value> {
    include_str!("../../../skills/bridget/SKILL.md")
        .split("```json\n")
        .skip(1)
        .flat_map(|block| {
            serde_json::Deserializer::from_str(block.split("```").next().unwrap())
                .into_iter::<Value>()
        })
        .map(|example| example.expect("chaque objet JSON publié doit être valide"))
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
            assert_eq!(
                message.cross_project_reason.as_deref(),
                example["arguments"]["cross_project_reason"].as_str()
            );
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
    let local_project = root.join("local-project");
    let other_project = root.join("other-project");
    for project in [&local_project, &other_project] {
        std::fs::create_dir(project).unwrap();
        assert!(
            std::process::Command::new("/usr/bin/git")
                .args(["init", "--quiet"])
                .arg(project)
                .status()
                .unwrap()
                .success()
        );
    }
    let outside = "89000000-0000-4000-8000-000000000138";
    let mut actor = Client::connect(&socket(&root));
    let mut recipient = Client::connect(&socket(&root));
    let mut external = Client::connect(&socket(&root));
    // Le helper de registration historique fixe un autre hostname. Ici la
    // preuve Git doit matcher l'hôte réel du daemon isolé. Les connexions et
    // la sauvegarde du credential restent les mécanismes existants.
    for (peer, id, instance, project) in [
        (&mut actor, ACTOR, "089-skill-a", &local_project),
        (&mut recipient, RECIPIENT, "089-skill-b", &local_project),
        (&mut external, outside, "089-skill-outside", &other_project),
    ] {
        peer.send(WrapperToDaemon::Register {
            identity_version: 2,
            agent_type: "fixture".into(),
            agent_id: id.into(),
            host: Some("idempotency-isolated".into()),
            transport: Some("acp".into()),
            channel: None.into(),
            mode: Some(PresenceMode::Acp),
            location: None,
            os: Some("test".into()),
            instance_id: Some(instance.into()),
            domain: None,
            journal_available: None,
            turn_in_progress: false,
        });
        let DaemonToWrapper::Registered {
            agent_id,
            credential: Some(credential),
        } = peer.receive()
        else {
            panic!("propriétaire réel attendu")
        };
        assert_eq!(agent_id, id);
        save_fixture_credential(&socket(&root), id, instance, credential);
        peer.send(WrapperToDaemon::CommunicationProjectFact {
            root: project.to_string_lossy().into_owned(),
            source: CommunicationProjectSource::Git,
            host: "idempotency-isolated".into(),
            worktree_root: None,
        });
        assert!(matches!(
            peer.receive(),
            DaemonToWrapper::ProjectContextResult {
                project: Some(_),
                ..
            }
        ));
    }
    let mut examples = examples();
    assert_eq!(
        examples.len(),
        6,
        "chaque exemple doit être exécuté, sans filtre"
    );
    let mut mcp = McpProcess::start(&root, ACTOR, "089-skill-a");
    initialize(&mut mcp);
    let directory = call(&mut mcp, &examples[0]);
    assert!(directory.to_string().contains(RECIPIENT));
    assert!(directory.to_string().contains(outside));
    examples[1]["arguments"]["to"] = outside.into();
    let cross_project = deliver_and_retry(&mut mcp, &mut external, &examples[1]);
    assert_eq!(
        cross_project["project_warnings"][0]["code"],
        "cross_project"
    );
    let local = call(&mut mcp, &examples[2]);
    assert!(local.to_string().contains(RECIPIENT));
    assert!(!local.to_string().contains(outside));
    examples[3]["arguments"]["to"] = RECIPIENT.into();
    let question = deliver_and_retry(&mut mcp, &mut recipient, &examples[3]);
    let initial = call(&mut mcp, &examples[5]);
    assert_eq!(initial["requests"][0]["state"], "open");
    mcp.stop();

    // Deuxième session, instance propre : seul l'identifiant reçu fait le lien.
    let mut mcp = McpProcess::start(&root, RECIPIENT, "089-skill-b");
    initialize(&mut mcp);
    examples[4]["arguments"]["to"] = ACTOR.into();
    if examples[4]["arguments"].get("in_reply_to").is_some() {
        examples[4]["arguments"]["in_reply_to"] = question["id"].clone();
    }
    let answer = deliver_and_retry(&mut mcp, &mut actor, &examples[4]);
    let ledger = call(&mut mcp, &examples[5]);
    // Mutant : supprimer in_reply_to DANS LA SKILL garde open, donc casse ici.
    assert_eq!(ledger["requests"][0]["state"], "answered");
    let messages = ledger["messages"].as_array().unwrap();
    assert_eq!(
        messages.len(),
        3,
        "trois exemples d'envoi, aucune deuxième injection au retry"
    );
    for receipt in [&cross_project, &question, &answer] {
        assert_eq!(
            messages.iter().filter(|m| m["id"] == receipt["id"]).count(),
            1
        );
    }
    assert_no_delivery(&mut actor);
    assert_no_delivery(&mut recipient);
    assert_no_delivery(&mut external);
    let cli = run_isolated(&root, &["ledger", "--limit", "20"], false);
    assert!(cli.status.success(), "{}", output_text(&cli));
    let text = String::from_utf8(cli.stdout).unwrap();
    for example in [&examples[1], &examples[3], &examples[4]] {
        assert!(text.contains(example["arguments"]["body"].as_str().unwrap()));
    }
    mcp.stop();
    drop(actor);
    drop(recipient);
    drop(external);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
