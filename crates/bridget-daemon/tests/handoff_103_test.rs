//! Session 103 — dossier de passation : contrats MCP/CLI, transport 099 réel,
//! conservation et parité. Home, socket, base et journaux temporaires ; aucun
//! fournisseur réel. Le daemon de test est arrêté par SIGTERM et sa sortie est
//! attendue ; ces tests n'émettent jamais de SIGKILL.
#![allow(dead_code)]

#[path = "support/idempotent.rs"]
mod support;

use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};
use support::*;

const PRODUCTION_SOCKET: &str = "/Users/user/.cache/bridget-core/bridget.sock";
const AGENT_A: &str = "10300000-0000-4000-8000-00000000000a";
const AGENT_B: &str = "10300000-0000-4000-8000-00000000000b";
const MARKER: &str = "[Bridget handoff v1]\n";

fn spec103_root(label: &str) -> std::path::PathBuf {
    let root = test_root(label);
    assert!(root.starts_with("/tmp"));
    assert_ne!(socket(&root), Path::new(PRODUCTION_SOCKET));
    root
}

/// Arrêt coopératif : SIGTERM puis attente réelle ; échec sans escalade.
fn stop_cooperatively(mut daemon: DaemonProcess) {
    signal_test_group(&mut daemon.child, libc::SIGTERM);
    let deadline = Instant::now() + Duration::from_secs(10);
    while daemon.child.try_wait().expect("état du daemon").is_none() {
        assert!(Instant::now() < deadline, "daemon non terminé sur SIGTERM");
        std::thread::sleep(Duration::from_millis(10));
    }
    wait_child(&mut daemon.child, Duration::from_secs(1));
    if let Some(logs) = daemon.logs.take() {
        let _ = logs.join();
    }
}

fn minimal_draft() -> Value {
    json!({"objective":"Corriger la pagination","summary":"Défaut reproduit sur la deuxième page ; cause encore incertaine."})
}

fn mcp_as(root: &Path, agent: &str, instance: &str) -> McpProcess {
    let mut mcp = McpProcess::start(root, agent, instance);
    let init = mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert!(init["result"]["protocolVersion"].is_string(), "{init}");
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    mcp
}

fn call(mcp: &mut McpProcess, id: u64, arguments: Value) -> Value {
    mcp.request(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"bridget_handoff","arguments":arguments}}))
}

/// CLI réelle avec identité attestée (preuve privée déjà émise par le owner)
/// et objet JSON sur stdin.
fn cli_handoff(
    root: &Path,
    agent: &str,
    instance: &str,
    args: &[&str],
    stdin: &[u8],
) -> std::process::Output {
    let name_file = root.join(format!("state/name-{instance}"));
    private_write(&name_file, agent).unwrap();
    let mut command = isolated_command(root);
    command
        .env("BRIDGET_AGENT_ID_FILE", &name_file)
        .env("BRIDGET_AGENT_INSTANCE_ID", instance)
        .arg("handoff")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("CLI réelle");
    track(&child);
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().expect("sortie CLI")
}

fn json_out(output: &std::process::Output) -> Value {
    serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .unwrap_or_else(|error| panic!("sortie non JSON ({error}) : {}", output_text(output)))
}

fn ledger_bodies(db: &Path, sender: &str) -> Vec<String> {
    let conn = rusqlite::Connection::open(db).unwrap();
    let mut statement = conn
        .prepare("SELECT body FROM ledger WHERE sender = ?1 ORDER BY ts, id")
        .unwrap();
    let rows = statement
        .query_map([sender], |row| row.get::<_, String>(0))
        .unwrap();
    rows.map(Result::unwrap).collect()
}

#[test]
fn spec103_s02_apercu_sans_daemon_ni_socket_ni_identifiant() {
    let root = spec103_root("spec103-s02");
    let mut mcp = mcp_as(&root, AGENT_A, "spec103-a");
    let response = call(
        &mut mcp,
        2,
        json!({"action":"preview","draft":minimal_draft()}),
    );
    let result = &response["result"]["structuredContent"];
    assert_eq!(result["status"], "preview_valid", "{response}");
    assert!(result["body"].as_str().unwrap().starts_with(MARKER));
    assert_eq!(result["bytes"], result["body"].as_str().unwrap().len());
    assert_eq!(
        result["warnings"],
        json!([
            "sources_not_verified",
            "retention_follows_ledger",
            "ledger_visibility_not_recipient_private"
        ])
    );
    assert!(result.get("id").is_none() && result.get("issued_at").is_none());
    assert!(
        !socket(&root).exists(),
        "aucune socket : l'aperçu ne se connecte pas"
    );
    let refused = call(
        &mut mcp,
        3,
        json!({"action":"preview","to":AGENT_B,"draft":minimal_draft()}),
    );
    assert_eq!(refused["error"]["code"], -32602, "{refused}");
    assert!(refused["error"]["message"].as_str().unwrap().contains("to"));
    let unknown = call(
        &mut mcp,
        4,
        json!({"action":"preview","draft":{"objective":"o","summary":"s","state":"done"}}),
    );
    assert_eq!(unknown["error"]["code"], -32602);
    mcp.stop();
}

#[test]
fn spec103_s06_s12_s13_s17_envoi_reel_rejeu_et_mise_a_jour() {
    let root = spec103_root("spec103-s06");
    let socket_path = socket(&root);
    let db = root.join("state/bridget.db");
    let daemon = spawn_daemon(&root, None);
    let _owner = register_agent_as(&socket_path, AGENT_A, "spec103-a");
    let mut recipient = register_agent_as(&socket_path, AGENT_B, "spec103-b");
    let mut mcp = mcp_as(&root, AGENT_A, "spec103-a");
    let preview = call(
        &mut mcp,
        2,
        json!({"action":"preview","draft":minimal_draft()}),
    );
    let expected_body = preview["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap()
        .to_string();
    let issued_at = issued_at();
    let send = json!({"action":"send","to":AGENT_B,"id":"handoff-pagination-01","issued_at":issued_at,"draft":minimal_draft()});
    let first = call(&mut mcp, 3, send.clone());
    let receipt = &first["result"]["structuredContent"];
    assert_eq!(receipt["status"], "in_flight", "{first}");
    assert_eq!(receipt["id"], "handoff-pagination-01");
    assert_eq!(receipt["issued_at"], issued_at);
    assert_eq!(receipt["handoff_version"], 1);
    assert_eq!(receipt["bytes"], expected_body.len());
    assert!(
        receipt.get("body").is_none(),
        "send ne répète pas le dossier"
    );
    let delivered = receive_delivery(&mut recipient);
    let DaemonToWrapper::DeliverIdempotent {
        message,
        delivery_id,
        delivery_generation,
        ..
    } = delivered
    else {
        unreachable!()
    };
    assert_eq!(
        message.body, expected_body,
        "le destinataire reçoit le corps exact"
    );
    assert_eq!(message.from, AGENT_A, "auteur réel de l'envoi");
    assert!(!message.reply, "aucune réponse attendue par défaut");
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    // Dix rejeux identiques : un seul message dans le ledger, aucune remise nouvelle.
    for index in 0..10 {
        let again = call(&mut mcp, 10 + index, send.clone());
        let status = again["result"]["structuredContent"]["status"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(
            matches!(status.as_str(), "accepted" | "in_flight"),
            "rejeu {index} : {again}"
        );
        assert_eq!(
            again["result"]["structuredContent"]["id"],
            "handoff-pagination-01"
        );
    }
    assert_no_delivery(&mut recipient);
    assert_eq!(
        ledger_bodies(&db, AGENT_A).len(),
        1,
        "une seule opération après dix rejeux"
    );
    // Même clé, dossier différent : refus, première opération intacte.
    let mut divergent = send.clone();
    divergent["draft"]["summary"] = json!("Résumé modifié sous la même clé.");
    let mismatch = call(&mut mcp, 30, divergent);
    let status = mismatch["result"]["structuredContent"]["status"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(status.contains("mismatch"), "{mismatch}");
    assert_eq!(ledger_bodies(&db, AGENT_A), vec![expected_body.clone()]);
    // Mise à jour : nouvelle clé, nouveau message ; l'ancien n'est pas muté.
    let update = json!({"action":"send","to":AGENT_B,"id":"handoff-pagination-02","issued_at":issued_at,"draft":{"objective":"Corriger la pagination","summary":"Cause trouvée : purge concurrente.","references":[{"kind":"message","label":"dossier précédent","id":"handoff-pagination-01","target":AGENT_B,"source_label":"Bridget de test"}]}});
    let second = call(&mut mcp, 31, update);
    assert_eq!(
        second["result"]["structuredContent"]["status"], "in_flight",
        "{second}"
    );
    let bodies = ledger_bodies(&db, AGENT_A);
    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[0], expected_body);
    assert!(bodies[1].contains("handoff-pagination-01"));
    mcp.stop();
    drop(recipient);
    stop_cooperatively(daemon);
}

#[test]
fn spec103_s09_conservation_apres_reouverture() {
    let root = spec103_root("spec103-s09");
    let socket_path = socket(&root);
    let db = root.join("state/bridget.db");
    let daemon = spawn_daemon(&root, None);
    let _owner = register_agent_as(&socket_path, AGENT_A, "spec103-a");
    let _recipient = register_agent_as(&socket_path, AGENT_B, "spec103-b");
    let mut mcp = mcp_as(&root, AGENT_A, "spec103-a");
    let draft = json!({"objective":"Conserver","summary":"Corps à retrouver après redémarrage : é / \\ \t 😀","limitations":["Test non exécuté sur la branche cible."]});
    let body =
        call(&mut mcp, 2, json!({"action":"preview","draft":draft}))["result"]["structuredContent"]
            ["body"]
            .as_str()
            .unwrap()
            .to_string();
    let sent = call(
        &mut mcp,
        3,
        json!({"action":"send","to":AGENT_B,"draft":draft}),
    );
    assert_eq!(
        sent["result"]["structuredContent"]["status"], "in_flight",
        "{sent}"
    );
    mcp.stop();
    stop_cooperatively(daemon);
    let daemon = spawn_daemon(&root, None);
    stop_cooperatively(daemon);
    assert_eq!(ledger_bodies(&db, AGENT_A), vec![body]);
}

#[test]
fn spec103_s15_s16_reponse_facultative_et_dnd_sans_contournement() {
    let root = spec103_root("spec103-s15");
    let socket_path = socket(&root);
    let daemon = spawn_daemon(&root, None);
    let _owner = register_agent_as(&socket_path, AGENT_A, "spec103-a");
    let mut recipient = register_agent_as(&socket_path, AGENT_B, "spec103-b");
    let mut mcp = mcp_as(&root, AGENT_A, "spec103-a");
    let asked = call(
        &mut mcp,
        2,
        json!({"action":"send","to":AGENT_B,"reply":true,"reply_timeout":120,"draft":minimal_draft()}),
    );
    assert_eq!(
        asked["result"]["structuredContent"]["status"], "in_flight",
        "{asked}"
    );
    let DaemonToWrapper::DeliverIdempotent { message, .. } = receive_delivery(&mut recipient)
    else {
        unreachable!()
    };
    assert!(message.reply, "réponse demandée explicitement");
    assert_eq!(message.reply_timeout, Some(120));
    let bad = call(
        &mut mcp,
        3,
        json!({"action":"send","to":AGENT_B,"reply_timeout":5,"draft":minimal_draft()}),
    );
    assert_eq!(bad["error"]["code"], -32602, "{bad}");
    // DND : refus 099 nommé, aucune relance ni nouvelle clé.
    let until = issued_at() as u64 + 600;
    recipient.send(WrapperToDaemon::Availability {
        agent: AGENT_B.into(),
        until_secs: Some(until),
    });
    assert!(matches!(recipient.receive(), DaemonToWrapper::Ack { .. }));
    let dnd = call(
        &mut mcp,
        4,
        json!({"action":"send","to":AGENT_B,"id":"handoff-dnd","issued_at":issued_at(),"draft":{"objective":"Pendant le DND","summary":"Un autre dossier, corps distinct du précédent."}}),
    );
    assert_eq!(dnd["result"]["structuredContent"]["status"], "dnd", "{dnd}");
    assert_eq!(dnd["result"]["structuredContent"]["id"], "handoff-dnd");
    assert_no_delivery(&mut recipient);
    mcp.stop();
    drop(recipient);
    stop_cooperatively(daemon);
}

#[test]
fn spec103_s18_s19_s20_parite_cli_mcp_stdin_borne_et_catalogue() {
    let root = spec103_root("spec103-s18");
    let socket_path = socket(&root);
    let daemon = spawn_daemon(&root, None);
    let _a = register_agent_as(&socket_path, AGENT_A, "spec103-a");
    let mut b = register_agent_as(&socket_path, AGENT_B, "spec103-b");
    let mut mcp = mcp_as(&root, AGENT_A, "spec103-a");
    let tools = mcp.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let tool = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "bridget_handoff")
        .expect("outil au catalogue");
    assert_eq!(
        tool["inputSchema"]["properties"]["action"]["enum"],
        json!(["preview", "send"])
    );
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);

    let draft = json!({"objective":"Parité","summary":"Même objet, mêmes octets.","results":[{"text":"Un résultat déclaré."}],"next_step":"Relancer le test ciblé."});
    let preview_mcp =
        call(&mut mcp, 3, json!({"action":"preview","draft":draft}))["result"]["structuredContent"]
            .clone();
    let stdin = serde_json::to_vec(&json!({"action":"preview","draft":draft})).unwrap();
    let preview_cli = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["preview", "--json-stdin", "--json"],
        &stdin,
    );
    assert!(
        preview_cli.status.success(),
        "{}",
        output_text(&preview_cli)
    );
    assert_eq!(
        json_out(&preview_cli),
        preview_mcp,
        "aperçu identique par CLI et MCP"
    );
    let human = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["preview", "--json-stdin"],
        &stdin,
    );
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).starts_with(MARKER));

    // Envoi CLI : même reçu que MCP, exit 1 tant que la remise n'est qu'en vol.
    let issued = issued_at();
    let send = json!({"action":"send","to":AGENT_B,"id":"handoff-cli-01","issued_at":issued,"draft":draft});
    let send_cli = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["send", "--json-stdin", "--json"],
        &serde_json::to_vec(&send).unwrap(),
    );
    let receipt_cli = json_out(&send_cli);
    assert_eq!(
        receipt_cli["status"],
        "in_flight",
        "{}",
        output_text(&send_cli)
    );
    assert_eq!(
        send_cli.status.code(),
        Some(1),
        "issue non confirmée → code 1, reçu imprimé"
    );
    let DaemonToWrapper::DeliverIdempotent { message, .. } = receive_delivery(&mut b) else {
        unreachable!()
    };
    assert_eq!(message.body, preview_mcp["body"]);
    assert_eq!(message.from, AGENT_A);
    let replay_mcp = call(&mut mcp, 4, send)["result"]["structuredContent"].clone();
    assert_eq!(replay_mcp["id"], receipt_cli["id"]);
    assert_eq!(replay_mcp["issued_at"], receipt_cli["issued_at"]);
    assert_eq!(
        replay_mcp["handoff_version"],
        receipt_cli["handoff_version"]
    );
    assert_eq!(replay_mcp["bytes"], receipt_cli["bytes"]);
    assert_eq!(replay_mcp["warnings"], receipt_cli["warnings"]);
    assert_no_delivery(&mut b);

    // Erreurs : action incohérente, JSON invalide, stdin trop long, sans --json-stdin.
    let mismatch = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["send", "--json-stdin"],
        &stdin,
    );
    assert_eq!(mismatch.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("sous-commande"));
    let invalid = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["preview", "--json-stdin"],
        b"{",
    );
    assert_eq!(invalid.status.code(), Some(2));
    let mut long = br#"{"action":"preview","draft":{"objective":"o","summary":""#.to_vec();
    long.extend(std::iter::repeat_n(b's', 64 * 1024));
    long.extend_from_slice(b"\"}}");
    let too_long = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["preview", "--json-stdin"],
        &long,
    );
    assert_eq!(
        too_long.status.code(),
        Some(2),
        "{}",
        output_text(&too_long)
    );
    assert!(String::from_utf8_lossy(&too_long.stderr).contains("65536"));
    let no_stdin = cli_handoff(&root, AGENT_A, "spec103-a", &["preview"], &stdin);
    assert_eq!(no_stdin.status.code(), Some(2));
    let bad_draft = cli_handoff(
        &root,
        AGENT_A,
        "spec103-a",
        &["preview", "--json-stdin", "--json"],
        br#"{"action":"preview","draft":{"objective":"o"}}"#,
    );
    assert_eq!(bad_draft.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad_draft.stderr).contains("summary"));
    mcp.stop();
    drop(b);
    stop_cooperatively(daemon);
}
