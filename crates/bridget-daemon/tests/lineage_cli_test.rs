//! Bout en bout CLI lineage 149 : daemon réel, wrapper T3 réel, socket Unix
//! réelle, semis SQLite direct. Aucun fournisseur n'est jamais lancé.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;

use bridget_transport::protocol::{
    CommunicationProjectSource, DaemonToWrapper, PresenceMode, WrapperToDaemon,
};
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

const T3: &str = "89000000-0000-4000-8000-000000000149";
const T3_AUTRE: &str = "89000000-0000-4000-8000-000000000148";
const TASK: &str = "14900000-0000-4000-8000-000000000001";
const TASK_TERMINALE: &str = "14900000-0000-4000-8000-000000000002";
const TASK_AUTRE_RACINE: &str = "14900000-0000-4000-8000-0000000000ee";
const REQUEST: &str = "49000000-0000-4000-8000-000000000003";
const REQUEST_AUTRE: &str = "49000000-0000-4000-8000-000000000004";
const HOTE: &str = "idempotency-isolated";

/// Réplique locale de t3code::stable_uuid : v5 du namespace interne présenté
/// comme v4. Toute dérive du namespace casse ces tests visiblement.
fn stable_uuid(name: &str) -> String {
    const NAMESPACE_098: [u8; 16] = [
        0x09, 0x8b, 0x71, 0xd3, 0xc0, 0xde, 0x4a, 0x11, 0x9b, 0x1d, 0x73, 0x63, 0x6f, 0x64, 0x65,
        0x01,
    ];
    let namespace = uuid::Uuid::from_bytes(NAMESPACE_098);
    let bytes = *uuid::Uuid::new_v5(&namespace, name.as_bytes()).as_bytes();
    uuid::Builder::from_random_bytes(bytes)
        .into_uuid()
        .hyphenated()
        .to_string()
}

fn project_root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn run_cli(root: &std::path::Path, args: &[&str]) -> (Option<i32>, Value) {
    let output = support::run_isolated(root, args, false);
    let text = support::output_text(&output);
    let value: Value = serde_json::from_str(text.trim()).unwrap_or(Value::Null);
    (output.status.code(), value)
}

fn definition() -> Value {
    serde_json::json!({
        "command": "npx",
        "args": ["fixture-lineage149"],
        "protocol": "acp",
        "forbidden_env": ["API_KEY"],
        "pass_env": [],
        "claude_config_dir": null,
        "permissions": "allow",
        "queue_capacity": 32,
        "notify_timeout_secs": 600,
        "mcp": {"interactive": "none", "acp_session": false},
        "capabilities": {"execution_paths": ["acp"], "models": {}},
        "digest": "fixture-lineage149"
    })
}

/// Semis SQL direct dans la base du daemon réel : la projection démarre au
/// marqueur seq 0, distinct de toute écriture projetée.
fn seed_task(
    db: &rusqlite::Connection,
    task_id: &str,
    owner: &str,
    state: &str,
    result: Option<&str>,
) {
    let payload = serde_json::json!({
        "task_id": task_id,
        "root_owner_agent_id": owner,
        "owner": owner,
        "owner_instance": format!("inst-{owner}"),
        "origin_owner_instance": format!("inst-{owner}"),
        "request": {
            "operation": "delegate",
            "request_id": format!("req-{task_id}"),
            "agent_type": "fixture",
            "model": "fixture-model",
            "task": format!("Mission CLI {task_id}"),
            "cwd": "/tmp"
        },
        "definition": definition(),
        "cwd": "/tmp",
        "child": stable_uuid(&format!("child:{task_id}")),
        "child_instance": stable_uuid(&format!("child-instance:{task_id}")),
        "mission": format!("mission-{task_id}"),
        "created_at": 1_700_000_000,
        "updated_at": 1_700_000_000,
        "state": state,
        "result": result,
        "error": null,
        "result_sent": false
    });
    db.execute(
        "INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
        rusqlite::params![task_id, format!("inst-{owner}"), format!("req-{task_id}"), b"cli149", payload.to_string()],
    )
    .unwrap();
}

/// Enregistre le wrapper T3, annonce le projet et lie le fil. Le client reste
/// vivant : la liaison disparaît à la déconnexion, jamais avant.
fn register_t3_wrapper(socket: &std::path::Path) -> support::Client {
    let agent = stable_uuid(T3);
    let instance = stable_uuid(&format!("instance:{T3}"));
    let mut t3 = support::Client::connect(socket);
    t3.send(WrapperToDaemon::Register {
        agent_type: "fixture".to_string(),
        identity_version: 2,
        agent_id: agent.clone(),
        host: Some(HOTE.to_string()),
        transport: Some("t3code".to_string()),
        channel: None.into(),
        mode: Some(PresenceMode::Cli),
        location: None,
        os: Some("test".to_string()),
        instance_id: Some(instance),
        domain: None,
        journal_available: Some(false),
        turn_in_progress: false,
    });
    let DaemonToWrapper::Registered { agent_id, .. } = t3.receive() else {
        panic!("wrapper T3 attendu");
    };
    assert_eq!(agent_id, agent);
    let annonce = || WrapperToDaemon::CommunicationProjectFact {
        root: project_root(),
        source: CommunicationProjectSource::T3,
        host: HOTE.to_string(),
        worktree_root: None,
    };
    t3.send(annonce());
    assert!(matches!(t3.receive(), DaemonToWrapper::ProjectContextResult { project: Some(_), .. }));
    t3.send(
        serde_json::from_value(serde_json::json!({
            "type":"T3ThreadBindingFact","version":1,"t3_thread_id":T3
        }))
        .unwrap(),
    );
    // Le fait de liaison n'a pas de réponse : un aller-retour sur la même
    // connexion garantit qu'il est traité avant la première CLI.
    t3.send(annonce());
    assert!(matches!(t3.receive(), DaemonToWrapper::ProjectContextResult { project: Some(_), .. }));
    t3
}

fn journal_ecrire(directory: &std::path::Path, suites: &[(u64, &str)]) {
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
        .unwrap();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(directory.join(format!(
            "{}.jsonl",
            bridget_transport::journal::current_host_date()
        )))
        .unwrap();
    use std::io::Write;
    for (seq, texte) in suites {
        writeln!(
            file,
            "{}",
            serde_json::json!({"v":1,"seq":seq,"event":"note","payload":{"text":texte}})
        )
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// Grammaire fermée sans daemon : chaque forme ouverte sort en exit 2.
// ---------------------------------------------------------------------------

#[test]
fn lineage_cli_149_grammaire_fermee_sans_daemon() {
    let root = support::test_root("l149-cli-grammaire");
    let casses: Vec<Vec<&str>> = vec![
        vec![],
        vec!["inspect"],
        vec!["inspect", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list"],
        vec!["inspect", "--json", "--project-root", "/tmp", "--action", "list"],
        vec!["inspect", "--json", "--t3-thread", T3, "--action", "list"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "relative", "--action", "list"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list", "--inconnu", "x"],
        vec!["inspect", "--json", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list"],
        vec!["inspect", "--json", "--t3-thread", "--project-root", "/tmp", "--action", "list"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "inconnue"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list", "--limit", "0"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list", "--limit", "101"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list", "--cursor", "curseur\ncontrole"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "show", "--task", "pas-un-uuid"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "show", "--task", TASK, "--offset", "262145"],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "journal", "--task", TASK, "--after-seq", "9007199254740992"],
        vec!["cancel", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--task", "pas-un-uuid", "--request-id", REQUEST],
        vec!["inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list", "--task", TASK],
    ];
    for cas in &casses {
        let mut args: Vec<&str> = vec!["lineage"];
        args.extend_from_slice(cas);
        let (code, value) = run_cli(&root, &args);
        assert_eq!(code, Some(2), "{cas:?} → {value}");
        assert_eq!(value["code"], "invalid_request", "{cas:?} → {value}");
        assert_eq!(value["retryable"], false, "{cas:?} → {value}");
    }
    // Socket absente : refus de liaison en exit 3, jamais un silence.
    let (code, value) = run_cli(
        &root,
        &["lineage", "inspect", "--json", "--t3-thread", T3, "--project-root", "/tmp", "--action", "list"],
    );
    assert_eq!(code, Some(3), "{value}");
    assert_eq!(value["code"], "binding_unavailable", "{value}");
    std::fs::remove_dir_all(&root).ok();
}

// ---------------------------------------------------------------------------
// Daemon réel : liste, show, journal, cancel sous garde native complète.
// ---------------------------------------------------------------------------

#[test]
fn lineage_cli_149_bout_en_bout_liste_show_journal_et_cancel() {
    let root = support::test_root("l149-cli-e2e");
    let daemon = support::spawn_daemon(&root, None);
    let _t3 = register_t3_wrapper(&support::socket(&root));

    let db = rusqlite::Connection::open(root.join("state/bridget.db")).unwrap();
    let agent = stable_uuid(T3);
    seed_task(&db, TASK, &agent, "queued", None);
    seed_task(&db, TASK_TERMINALE, &agent, "result_available", Some("Sortie CLI 149 éè"));
    seed_task(&db, TASK_AUTRE_RACINE, "autre-racine-149", "queued", None);
    drop(db);

    let racine = project_root();
    let fil: [&str; 4] = ["--t3-thread", T3, "--project-root", racine.as_str()];

    // List : seq 0 au semis direct, les deux tâches de la racine, jamais celle d'une autre racine.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--action", "list"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["status"], "ok", "{value}");
    assert_eq!(value["seq"], 0, "{value}");
    let tasks = value["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2, "{value}");
    assert!(tasks.iter().all(|task| task["task_id"] != TASK_AUTRE_RACINE), "{value}");
    let entree = tasks.iter().find(|task| task["task_id"] == TASK).expect("TASK listée");
    assert_eq!(entree["journal_available"], false);

    // Show : le résultat textuel n'est servi qu'à la tâche terminale.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--action", "show", "--task", TASK_TERMINALE, "--offset", "0", "--limit", "100"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["result"], "Sortie CLI 149 éè", "{value}");
    assert_eq!(value["result_total_bytes"], "Sortie CLI 149 éè".len(), "{value}");

    // Mauvais fil : liaison indisponible, jamais les données.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&["--t3-thread", T3_AUTRE, "--project-root", racine.as_str()]);
    args.extend_from_slice(&["--action", "show", "--task", TASK_TERMINALE, "--offset", "0", "--limit", "100"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "binding_unavailable", "{value}");

    // Mauvaise racine : mismatch de projet.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&["--t3-thread", T3, "--project-root", "/tmp"]);
    args.extend_from_slice(&["--action", "show", "--task", TASK_TERMINALE, "--offset", "0", "--limit", "100"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "project_mismatch", "{value}");

    // Offset au milieu d'un point de code : refus explicite.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--action", "show", "--task", TASK_TERMINALE, "--offset", "16", "--limit", "100"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "result_offset_invalid", "{value}");

    // Journal d'une tâche inconnue : refus fermé.
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--action", "journal", "--task", "00000000-0000-4000-8000-000000000000", "--after-seq", "0", "--limit", "50"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "task_unavailable", "{value}");

    // Le dossier sessions/<child> privé rend le journal lisible par la CLI.
    let child = stable_uuid(&format!("child:{TASK}"));
    journal_ecrire(
        &root.join("state").join("sessions").join(&child),
        &[(1, "étape un"), (2, "étape deux")],
    );
    let mut args: Vec<&str> = vec!["lineage", "inspect", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--action", "journal", "--task", TASK, "--after-seq", "0", "--limit", "50"]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["status"], "ok", "{value}");
    assert_eq!(value["events"].as_array().unwrap().len(), 2, "{value}");
    assert_eq!(value["caught_up"], true, "{value}");

    // Cancel : reçu moteur, rejeu identique, mismatch d'enveloppe.
    let mut args: Vec<&str> = vec!["lineage", "cancel", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--task", TASK, "--request-id", REQUEST]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value, serde_json::json!({"version":1,"task_id":TASK,"status":"cancelling"}), "{value}");
    let (code, rejoue) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{rejoue}");
    assert_eq!(rejoue, value, "le reçu est rejoué à l'identique");
    let mut args: Vec<&str> = vec!["lineage", "cancel", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--task", TASK_TERMINALE, "--request-id", REQUEST]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(2), "{value}");
    assert_eq!(value["code"], "envelope_mismatch", "{value}");

    // Une seconde demande sur la tâche terminale : reçu sans mutation.
    let mut args: Vec<&str> = vec!["lineage", "cancel", "--json"];
    args.extend_from_slice(&fil);
    args.extend_from_slice(&["--task", TASK_TERMINALE, "--request-id", REQUEST_AUTRE]);
    let (code, value) = run_cli(&root, &args);
    assert_eq!(code, Some(0), "{value}");
    assert_eq!(value["status"], "result_available", "{value}");

    daemon.stop();
}

// ---------------------------------------------------------------------------
// Watch CLI : Ready seq 0 sur la vraie socket, fermeture propre en exit 0.
// ---------------------------------------------------------------------------

#[test]
fn lineage_cli_149_watch_ready_puis_fermeture_propre() {
    let root = support::test_root("l149-cli-watch");
    let daemon = support::spawn_daemon(&root, None);
    let _t3 = register_t3_wrapper(&support::socket(&root));

    let db = rusqlite::Connection::open(root.join("state/bridget.db")).unwrap();
    seed_task(&db, TASK, &stable_uuid(T3), "queued", None);
    drop(db);

    let mut command = support::isolated_command(&root);
    command
        .arg("lineage")
        .arg("watch")
        .arg("--json")
        .arg("--t3-thread")
        .arg(T3)
        .arg("--project-root")
        .arg(project_root());
    command.stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = command.spawn().expect("CLI watch réelle");
    support::track(&child);
    let stdout = child.stdout.take().expect("stdout watch");
    let stdout_fd = stdout.as_raw_fd();
    let mut reader = BufReader::new(stdout);
    let mut poll_fd = libc::pollfd {
        fd: stdout_fd,
        events: libc::POLLIN,
        revents: 0,
    };
    assert!(unsafe { libc::poll(&mut poll_fd, 1, 15_000) } > 0, "ready watch absent");
    let mut ligne = String::new();
    assert!(reader.read_line(&mut ligne).unwrap() > 0, "ligne ready vide");
    let value: Value = serde_json::from_str(ligne.trim()).expect(&ligne);
    assert_eq!(value["status"], "ready", "{ligne}");
    assert_eq!(value["version"], 1, "{ligne}");
    assert_eq!(value["seq"], 0, "le marqueur Ready est distinct de la seq du magasin : {ligne}");

    // Fermeture du stdout : POLLHUP côté CLI → sortie 0, jamais une erreur.
    drop(reader);
    support::wait_child(&mut child, Duration::from_secs(15));
    let status = child.try_wait().unwrap();
    assert_eq!(status.and_then(|s| s.code()), Some(0), "watch CLI non terminé proprement");
    daemon.stop();
}
