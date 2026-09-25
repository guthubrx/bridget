#![cfg(feature = "test-support")]
//! Identité opaque : le nom humain n'est ni l'adresse ni le scope de rejeu.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_daemon::agent_profile::{AgentProfileError, AgentProfileStore, AgentProfileUpdate};
use bridget_transport::protocol::{DisplayNameOutcome, DisplayNameRefusal, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use rusqlite::{Connection, OpenFlags, types::Value as SqlValue};
use serde_json::{Value, json};
use std::io::Write;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn start_mcp(root: &Path, instance: &str) -> McpProcess {
    start_mcp_as(root, ACTOR, instance)
}

fn start_mcp_as(root: &Path, agent: &str, instance: &str) -> McpProcess {
    let mut mcp = McpProcess::start(root, agent, instance);
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
            .is_some_and(|agents| agents.iter().any(|entry| entry["agent_id"] == agent)),
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
    // Préparation du profil de l'émetteur et oracle historique de refus ; le
    // destinataire est renommé plus bas par le vrai binaire CLI, puis repris.
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
fn extension_nom_affiche_v1_a_un_canon_externe_et_ferme() {
    let corpus = include_str!("../../../fixtures/display-name-v1.jsonl");
    for (i, line) in corpus.lines().enumerate() {
        let bytes = if i == 0 {
            encode(&decode::<WrapperToDaemon>(line).unwrap()).unwrap()
        } else {
            encode(&decode::<DaemonToWrapper>(line).unwrap()).unwrap()
        };
        assert_eq!(bytes.as_bytes(), line.as_bytes());
    }
    assert!(decode::<WrapperToDaemon>(r#"{"type":"display_name_set","request":{"version":1,"display_name":"B","future":true}}"#).is_err());
    assert!(
        decode::<DaemonToWrapper>(
            r#"{"type":"display_name_result","outcome":{"status":"rejected","reason":"unknown"}}"#
        )
        .is_err()
    );
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
    let name_file = &identity_files[0].0;
    let instance = name_file
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("instance-")
        .unwrap();
    let rename_cli = |name: &str, caller_instance: &str| {
        let mut command = isolated_command(&root);
        command
            .args(["rename", name])
            .env("BRIDGET_AGENT_ID_FILE", name_file)
            .env("BRIDGET_AGENT_INSTANCE_ID", caller_instance);
        run_command(command)
    };
    let mut profiles = AgentProfileStore::open(&db).unwrap();
    rename_profile(&mut profiles, ACTOR, "Auteur renommé").unwrap();
    // Données historiques non normalisées : renommer ne doit jamais nettoyer
    // ni re-déployer le prompt. Mutation update_profile complet → bytes perdus.
    Connection::open(&db).unwrap().execute("UPDATE agent_profiles SET instructions='  instructions verbatim  ', instructions_revision=7 WHERE agent_id=?1", [ACP_AGENT]).unwrap();
    let renamed = rename_cli("Destinataire renommé", instance);
    assert!(renamed.status.success(), "{}", output_text(&renamed));
    assert_eq!(listed_name(&root, ACP_AGENT), "Destinataire renommé");
    let stable_profile = profiles.profile_detail(ACP_AGENT).unwrap();
    assert_eq!(stable_profile.instructions, "  instructions verbatim  ");
    assert_eq!(stable_profile.summary.instructions_revision, 7);
    let replay = rename_cli("Destinataire renommé", instance);
    assert!(replay.status.success(), "{}", output_text(&replay));
    assert_eq!(
        replay.stdout, renamed.stdout,
        "même nom : pas de nouvelle révision"
    );
    for raw in [
        r#"{"type":"display_name_set","request":{"version":1,"display_name":"B","future":true}}"#,
        r#"{"type":"display_name_set","request":{"version":1,"display_name":"B"},"agent_id":"foreign"}"#,
        r#"{"type":"display_name_set","request":{"version":2,"display_name":"B"}}"#,
        r#"{"type":"display_name_set","request":{"version":1,"display_name":"B\u001b"}}"#,
    ] {
        let mut raw_client = Client::connect(&socket(&root));
        writeln!(raw_client.writer, "{raw}").unwrap();
        raw_client.writer.flush().unwrap();
        assert!(matches!(
            raw_client.receive(),
            DaemonToWrapper::DisplayNameResult {
                outcome: DisplayNameOutcome::Rejected {
                    reason: DisplayNameRefusal::InvalidRequest
                }
            }
        ));
        assert_eq!(profiles.profile_detail(ACP_AGENT).unwrap(), stable_profile);
    }
    let mut anonymous = Client::connect(&socket(&root));
    anonymous.send(WrapperToDaemon::DisplayNameSet {
        request: bridget_transport::protocol::DisplayNameRequest {
            version: 1,
            display_name: "B".into(),
        },
    });
    assert!(matches!(
        anonymous.receive(),
        DaemonToWrapper::DisplayNameResult {
            outcome: DisplayNameOutcome::Rejected {
                reason: DisplayNameRefusal::IdentityUnavailable
            }
        }
    ));
    drop(anonymous);
    for (name, caller_instance) in [
        ("Usurpation", "foreign-instance"),
        ("Auteur renommé", instance),
        ("", instance),
    ] {
        let rejected = rename_cli(name, caller_instance);
        assert!(
            !rejected.status.success(),
            "{name}: {}",
            output_text(&rejected)
        );
        assert_eq!(profiles.profile_detail(ACP_AGENT).unwrap(), stable_profile);
    }
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

#[test]
fn spec094_mcp_modifie_uniquement_sa_propre_presence() {
    let root = test_root("spec094-mcp-self");
    let daemon = spawn_daemon(&root, None);
    let mut mcp = start_mcp(&root, "spec094-instance");
    let mut other = start_mcp_as(&root, ACP_AGENT, "spec094-other-instance");
    let reserved = other.request(json!({"jsonrpc":"2.0","id":8,"method":"tools/call",
        "params":{"name":"bridget_rename","arguments":{"display_name":"Nom réservé 094"}}}));
    assert_eq!(reserved["result"]["structuredContent"]["status"], "applied");
    other.stop();
    fs::write(root.join("state/mcp-agent-name"), ACTOR).unwrap();
    let conflict = mcp.request(json!({"jsonrpc":"2.0","id":9,"method":"tools/call",
        "params":{"name":"bridget_rename","arguments":{"display_name":"Nom réservé 094"}}}));
    assert_eq!(
        conflict["result"]["structuredContent"]["status"],
        "rejected"
    );
    let calls = [
        json!({"jsonrpc":"2.0","id":10,"method":"tools/call","params":{
            "name":"bridget_rename","arguments":{"display_name":"Agent 094"}}}),
        json!({"jsonrpc":"2.0","id":11,"method":"tools/call","params":{
            "name":"bridget_dnd","arguments":{"mode":"on","duration":"1m"}}}),
        json!({"jsonrpc":"2.0","id":12,"method":"tools/call","params":{
            "name":"bridget_domain","arguments":{"domain":"coherence-094"}}}),
        json!({"jsonrpc":"2.0","id":13,"method":"tools/call","params":{
            "name":"bridget_runtime","arguments":{"model":"gpt-5.6-sol","effort":"high"}}}),
    ];
    let mut responses = calls
        .into_iter()
        .map(|request| mcp.request(request))
        .collect::<Vec<_>>();
    for request in [
        json!({"jsonrpc":"2.0","id":20,"method":"tools/call","params":{
            "name":"bridget_dnd","arguments":{"mode":"off"}}}),
        json!({"jsonrpc":"2.0","id":21,"method":"tools/call","params":{
            "name":"bridget_dnd","arguments":{"mode":"on","duration":"1m"}}}),
        json!({"jsonrpc":"2.0","id":22,"method":"tools/call","params":{
            "name":"bridget_domain","arguments":{"reset":true}}}),
        json!({"jsonrpc":"2.0","id":23,"method":"tools/call","params":{
            "name":"bridget_domain","arguments":{"domain":"coherence-094"}}}),
    ] {
        responses.push(mcp.request(request));
    }
    let who = mcp.request(json!({"jsonrpc":"2.0","id":14,"method":"tools/call",
        "params":{"name":"bridget_who","arguments":{}}}));
    let status = mcp.request(json!({"jsonrpc":"2.0","id":15,"method":"tools/call",
        "params":{"name":"bridget_status","arguments":{}}}));
    let control = mcp.request(json!({"jsonrpc":"2.0","id":16,"method":"tools/call",
        "params":{"name":"bridget_control_status","arguments":{"history_limit":2}}}));
    mcp.stop();
    assert_eq!(
        fs::read_to_string(root.join("state/agent-domains").join(ACTOR)).unwrap(),
        "coherence-094"
    );
    let mut resumed = start_mcp(&root, "spec094-instance");
    let after_reconnect = resumed.request(json!({"jsonrpc":"2.0","id":17,"method":"tools/call",
        "params":{"name":"bridget_who","arguments":{}}}));
    resumed.stop();

    for response in responses {
        assert!(
            response.get("error").is_none(),
            "appel 094 refusé : {response}"
        );
        assert_ne!(
            response["result"]["isError"], true,
            "appel 094 en erreur : {response}"
        );
    }
    let actor = who["result"]["structuredContent"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| agent["agent_id"] == ACTOR)
        .expect("appelant MCP encore présent");
    assert_eq!(actor["display_name"], "Agent 094");
    assert_eq!(actor["state"], "dnd");
    assert_eq!(actor["domain"], "coherence-094");
    assert_eq!(actor["model"], "gpt-5.6-sol");
    assert_eq!(actor["effort"], "high");
    let status = &status["result"]["structuredContent"];
    assert_eq!(status["running"], true);
    assert!(status.get("agents_inventory_available").is_some());
    assert!(status.get("agent_count").is_some());
    for secret in ["socket", "socket_path", "db", "db_path", "instance_id"] {
        assert!(
            status.get(secret).is_none(),
            "champ sensible publié : {secret}"
        );
    }
    let control = &control["result"]["structuredContent"];
    assert!(control.get("state").is_some());
    assert!(control["inbox_open_count"].is_u64());
    assert!(control["history"].as_array().unwrap().len() <= 2);
    let reconnected_actor = after_reconnect["result"]["structuredContent"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| agent["agent_id"] == ACTOR)
        .expect("même identité revenue après reconnexion MCP");
    assert_eq!(reconnected_actor["display_name"], "Agent 094");
    assert_eq!(reconnected_actor["state"], "dnd");
    assert_eq!(reconnected_actor["domain"], "coherence-094");
    assert_eq!(reconnected_actor["model"], "gpt-5.6-sol");

    daemon.stop();
    fs::remove_dir_all(&root).unwrap();
}
