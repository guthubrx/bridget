#![cfg(feature = "test-support")]
//! Identité opaque : le nom humain n'est ni l'adresse ni le scope de rejeu.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_daemon::agent_profile::{AgentProfileError, AgentProfileStore, AgentProfileUpdate};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn start_mcp(root: &Path, instance: &str) -> McpProcess {
    let mut mcp = McpProcess::start(root, ACTOR, instance);
    assert!(
        mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}))
            .get("error")
            .is_none()
    );
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    let who = mcp.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"bridget_who","arguments":{}}}));
    assert!(
        who["result"]["structuredContent"]["agents"]
            .as_array()
            .is_some_and(|agents| agents.iter().any(|agent| agent["agent_id"] == ACTOR)),
        "{who}"
    );
    mcp
}

fn call(rpc_id: u64, timestamp: i64) -> Value {
    json!({"jsonrpc":"2.0","id":rpc_id,"method":"tools/call","params":{
        "name":"bridget_send","arguments":{"to":ACP_AGENT,"body":"identité stable 🐙",
        "id":"same-key-two-instances","issued_at":timestamp}}})
}

fn wait_accepted(mcp: &mut McpProcess, timestamp: i64) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let response = mcp.request(call(10, timestamp));
        let result = &response["result"]["structuredContent"];
        match result["status"].as_str() {
            Some("accepted") => return result.clone(),
            Some("outcome_unknown" | "in_flight") => {}
            _ => panic!("issue inattendue : {response}"),
        }
        assert!(Instant::now() < deadline, "ACK absent : {response}");
        std::thread::yield_now();
    }
}

fn records(db: &Path) -> Vec<Vec<SqlValue>> {
    let conn = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM idempotency_records ORDER BY issuer_scope, operation_kind, idempotency_key").unwrap();
    let count = stmt.column_count();
    stmt.query_map([], |row| (0..count).map(|i| row.get(i)).collect())
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn rename_profile(
    profiles: &mut AgentProfileStore,
    agent: &str,
    name: &str,
) -> Result<(), AgentProfileError> {
    // Vraie primitive de profil conservée, aucune écriture SQL de simulation.
    // Cette preuve ne prétend PAS tester une commande rename CLI (T015 reste
    // ouverte tant que la surface sans GUI n'est pas raccordée).
    let current = profiles.profile_detail(agent)?;
    profiles
        .update_profile(
            agent,
            AgentProfileUpdate {
                expected_revision: current.summary.revision,
                display_name: name.into(),
                labels: current.summary.labels,
                avatar_shape: current.summary.avatar_shape,
                avatar_color: current.summary.avatar_color,
                instructions: current.instructions,
            },
        )
        .map(|_| ())
}

fn listed_name(root: &Path, agent: &str) -> String {
    let mut client = Client::connect(&socket(root));
    client.send(WrapperToDaemon::ListAgents);
    match client.receive() {
        DaemonToWrapper::AgentList { agents } => {
            agents
                .into_iter()
                .find(|entry| entry.agent_id == agent)
                .expect("agent vivant dans l'annuaire")
                .display_name
        }
        other => panic!("annuaire attendu : {other:?}"),
    }
}

#[test]
fn noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper() {
    let root = test_root("identity-restart");
    let db = root.join("state/bridget.db");
    let daemon = spawn_daemon(&root, None);
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(10);
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    let timestamp = issued_at();
    let mut mcp = start_mcp(&root, "identity-instance-a");
    let accepted = wait_accepted(&mut mcp, timestamp);
    wait_for_counter(&counter, 1);
    let before = records(&db);
    assert_eq!(before.len(), 1);
    let identity_files = fs::read_dir(root.join("state/agent-names"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (path.clone(), fs::read(path).unwrap())
        })
        .collect::<Vec<_>>();
    assert!(!identity_files.is_empty(), "fichiers du vrai wrapper");
    let mut profiles = AgentProfileStore::open(&db).unwrap();
    rename_profile(&mut profiles, ACTOR, "Auteur renommé").unwrap();
    rename_profile(&mut profiles, ACP_AGENT, "Destinataire renommé").unwrap();
    assert_eq!(listed_name(&root, ACP_AGENT), "Destinataire renommé");
    let stable_profile = profiles.profile_detail(ACP_AGENT).unwrap();
    for (name, collision) in [("Auteur renommé", true), ("", false)] {
        let error = rename_profile(&mut profiles, ACP_AGENT, name).unwrap_err();
        assert!(if collision {
            matches!(error, AgentProfileError::DisplayNameConflict)
        } else {
            matches!(error, AgentProfileError::Invalid(_))
        });
        assert_eq!(profiles.profile_detail(ACP_AGENT).unwrap(), stable_profile);
    }
    drop(profiles);
    assert_eq!(wait_accepted(&mut mcp, timestamp), accepted);
    assert_eq!(records(&db), before);
    mcp.stop();

    // Barrière = issue Accepted observée après l'ACK du vrai wrapper, puis
    // modification de profil confirmée par l'annuaire. Pas de sommeil métier.
    daemon.crash();
    let restarted = spawn_daemon(&root, None);
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    assert_eq!(listed_name(&root, ACP_AGENT), "Destinataire renommé");
    for (path, bytes) in identity_files {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    let mut resumed_mcp = start_mcp(&root, "identity-instance-a");
    assert_eq!(wait_accepted(&mut resumed_mcp, timestamp), accepted);
    assert_eq!(records(&db), before, "canon et terminal durables inchangés");
    assert_eq!(
        fs::read(&counter).unwrap(),
        b"x",
        "pas de second prompt après reprise"
    );
    resumed_mcp.stop();
    restarted.stop();
    assert_eq!(wrapper.join(), Ok(()));
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(registry_root).unwrap();
}

#[test]
fn deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance() {
    let root = test_root("two-instances");
    let db = root.join("state/bridget.db");
    let daemon = spawn_daemon(&root, None);
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(10);
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    let timestamp = issued_at();
    let mut first = start_mcp(&root, "identity-instance-a");
    let mut second = start_mcp(&root, "identity-instance-b");
    assert_ne!(
        first.child.id(),
        second.child.id(),
        "même binaire, processus distincts"
    );
    let a = wait_accepted(&mut first, timestamp);
    let b = wait_accepted(&mut second, timestamp);
    wait_for_counter(&counter, 2);
    let before = records(&db);
    // Mutation discriminante : dériver issuer_scope de l'agent_id au lieu de
    // l'instance fusionnerait ces deux clés et n'injecterait qu'un prompt.
    assert_eq!(before.len(), 2);
    let deliveries: i64 = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .unwrap()
        .query_row(
            "SELECT COUNT(DISTINCT delivery_id) FROM send_deliveries",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        deliveries, 2,
        "deux scopes : deux remises durablement distinctes"
    );
    assert_eq!(wait_accepted(&mut first, timestamp), a);
    assert_eq!(wait_accepted(&mut second, timestamp), b);
    assert_eq!(records(&db), before);
    assert_eq!(fs::read(&counter).unwrap(), b"xx");
    first.stop();
    second.stop();
    daemon.stop();
    assert_eq!(wrapper.join(), Ok(()));
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(registry_root).unwrap();
}
