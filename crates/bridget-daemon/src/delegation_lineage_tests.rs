//! Tests du magasin de projection lineage 149. Ce fichier est un module de
//! test autonome : le principal le câble en fin de `delegation_lineage.rs`
//! avec `#[cfg(test)] #[path = "delegation_lineage_tests.rs"]
//! mod delegation_lineage_tests;`. Aucun cfg(test) n'est ajouté à la
//! production par cette ronde.
use super::*;
use rusqlite::Connection;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Fixtures locales — aucun fournisseur, aucun socket, SQLite privé temporaire.
// ---------------------------------------------------------------------------

fn definition() -> bridget_transport::protocol::ResolvedAgentDefinition {
    bridget_transport::protocol::ResolvedAgentDefinition {
        command: "npx".into(),
        args: vec!["fixture-lineage149".into()],
        protocol: "acp".into(),
        forbidden_env: vec!["API_KEY".into()],
        pass_env: Vec::new(),
        claude_config_dir: None,
        permissions: "allow".into(),
        queue_capacity: 32,
        notify_timeout_secs: 600,
        mcp: bridget_transport::ResolvedMcpDefinition {
            interactive: "none".into(),
            acp_session: false,
        },
        capabilities: bridget_transport::AdapterCapabilities::default(),
        digest: "fixture-lineage149".into(),
    }
}

fn delegate_request(request_id: &str, instruction: &str) -> NativeDelegationRequest {
    NativeDelegationRequest::Delegate {
        request_id: request_id.into(),
        agent_type: "fixture".into(),
        model: "fixture-model".into(),
        effort: None,
        task: instruction.into(),
        cwd: "/tmp".into(),
        posture: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn task(
    task_id: &str,
    owner: &str,
    owner_instance: &str,
    origin_owner_instance: &str,
    child: &str,
    child_instance: Option<&str>,
    state: &str,
    created_at: i64,
) -> Task {
    Task {
        task_id: task_id.into(),
        root_owner_agent_id: String::new(),
        parent_task_id: None,
        updated_at: 0,
        started_at: None,
        completed_at: None,
        owner: owner.into(),
        owner_instance: owner_instance.into(),
        origin_owner_instance: origin_owner_instance.into(),
        parent_execution_id: None,
        request: delegate_request(&format!("req-{task_id}"), "Mission 149\nligne secondaire"),
        permission_snapshot: None,
        effective_posture: None,
        definition: definition(),
        cwd: "/tmp".into(),
        child: child.into(),
        child_instance: child_instance.map(Into::into),
        mission: format!("mission-{task_id}"),
        mission_deadline_at: None,
        created_at,
        state: state.into(),
        result: None,
        error: None,
        result_sent: false,
        cleanup_done: false,
        failure_sent: false,
    }
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("bridget-l149-{label}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn open_store(path: &std::path::Path) -> DelegationStore {
    DelegationStore::open(path).unwrap()
}

/// Schéma 148 minimal : la table native_delegations seule, sans table de
/// projection ni index lineage. Les payloads omettent les champs 149 ; les
/// défauts serde doivent les combler à la migration.
const V148_DDL: &str = "CREATE TABLE native_delegations (
    task_id TEXT PRIMARY KEY, owner_instance TEXT NOT NULL,
    request_id TEXT NOT NULL, canonical BLOB NOT NULL, payload TEXT NOT NULL,
    UNIQUE(owner_instance, request_id));";

#[allow(clippy::too_many_arguments)]
fn v148_payload(
    task_id: &str,
    owner: &str,
    owner_instance: &str,
    origin_owner_instance: &str,
    child: &str,
    child_instance: Option<&str>,
) -> Value {
    json!({
        "task_id": task_id,
        "owner": owner,
        "owner_instance": owner_instance,
        "origin_owner_instance": origin_owner_instance,
        "request": {
            "operation": "delegate",
            "request_id": format!("req-{task_id}"),
            "agent_type": "fixture",
            "model": "fixture-model",
            "task": "Mission 148 héritée",
            "cwd": "/tmp"
        },
        "definition": {
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
        },
        "cwd": "/tmp",
        "child": child,
        "child_instance": child_instance,
        "mission": format!("mission-{task_id}"),
        "created_at": 1_000,
        "state": "queued",
        "result": null,
        "error": null,
        "result_sent": false
    })
}

fn seed_v148(path: &std::path::Path, rows: &[(&str, &str, &str, &str, &str, Option<&str>)]) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(V148_DDL).unwrap();
    for (task_id, owner, owner_instance, origin, child, child_instance) in rows {
        conn.execute(
            "INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
            params![task_id, owner_instance, format!("req-{task_id}"), b"v148", v148_payload(task_id, owner, owner_instance, origin, child, *child_instance).to_string()],
        ).unwrap();
    }
}

fn chain_payload(path: &std::path::Path, depth: usize) {
    // Chaîne fermée : t_i+1 est possédé par l'enfant de t_i et porte
    // origin_owner_instance = child_instance de t_i.
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(V148_DDL).unwrap();
    for i in 0..depth {
        let task_id = format!("t-chain-{i}");
        let owner = if i == 0 { "racine-149".to_string() } else { format!("agent-chain-{}", i - 1) };
        let child = format!("agent-chain-{i}");
        let origin = if i == 0 { "inst-origin-0".to_string() } else { format!("inst-child-{}", i - 1) };
        let payload = v148_payload(&task_id, &owner, &format!("inst-{i}"), &origin, &child, Some(&format!("inst-child-{i}")));
        conn.execute(
            "INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
            params![task_id, format!("inst-{i}"), format!("req-{task_id}"), b"v148", payload.to_string()],
        ).unwrap();
    }
}

fn meta(store: &DelegationStore) -> ProjectionMutation {
    store.projection_meta().unwrap()
}

fn entry_keys_sorted(entry: &Value) -> Vec<String> {
    let mut keys: Vec<String> = entry.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

const ENTRY_KEYS: &[&str] = &[
    "agent_type",
    "child_agent_id",
    "child_instance_id",
    "completed_at",
    "created_at",
    "cwd",
    "effort",
    "error",
    "execution_protocol",
    "journal_available",
    "model",
    "parent_agent_id",
    "parent_task_id",
    "posture",
    "result_available",
    "started_at",
    "status",
    "task_id",
    "title",
    "updated_at",
];

// ---------------------------------------------------------------------------
// Migration 148 → 149.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_migration_148_derive_racine_parent_et_horodatages() {
    let dir = temp_dir("migration");
    let path = dir.join("store.sqlite");
    seed_v148(
        &path,
        &[
            // (task_id, owner, owner_instance, origin_owner_instance, child, child_instance)
            ("t-racine", "humain", "inst-h", "inst-h", "agent-racine", Some("inst-racine")),
            ("t-enfant", "agent-racine", "inst-racine", "inst-racine", "agent-enfant", Some("inst-enfant")),
            ("t-isole", "humain-2", "inst-h2", "inst-h2", "agent-isole", Some("inst-iso")),
            // Chaîne rompue : l'instance d'origine ne correspond pas.
            ("t-cassee", "agent-racine", "inst-racine", "autre-instance", "agent-casse", Some("inst-casse")),
        ],
    );
    let store = open_store(&path);

    let racine = store.get("t-racine").unwrap().unwrap();
    assert_eq!(racine.root_owner_agent_id, "humain");
    assert_eq!(racine.parent_task_id, None);
    // Les défauts 149 : updated = created, aucun horodatage de vie.
    assert_eq!(racine.updated_at, 1_000);
    assert_eq!(racine.started_at, None);
    assert_eq!(racine.completed_at, None);

    let enfant = store.get("t-enfant").unwrap().unwrap();
    assert_eq!(enfant.parent_task_id.as_deref(), Some("t-racine"));
    assert_eq!(enfant.root_owner_agent_id, "humain");
    assert_eq!(enfant.updated_at, 1_000);

    let isole = store.get("t-isole").unwrap().unwrap();
    assert_eq!(isole.root_owner_agent_id, "humain-2");
    assert_eq!(isole.parent_task_id, None);

    let cassee = store.get("t-cassee").unwrap().unwrap();
    assert_eq!(cassee.root_owner_agent_id, "agent-racine");
    assert_eq!(cassee.parent_task_id, None);

    // La projection démarre au marqueur seq 0, distinct de tout signal.
    assert_eq!(meta(&store).seq, 0);

    // La liste racine rend les quatre tâches avec leur racine dérivée.
    let page = store.lineage_page("humain", 100, None, |_| false).unwrap();
    let ids: Vec<String> = page["tasks"].as_array().unwrap().iter().map(|t| t["task_id"].as_str().unwrap().to_string()).collect();
    assert_eq!(ids, vec!["t-enfant".to_string(), "t-racine".to_string()]);

    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn delegation149_migration_fermee_cycle_profondeur_et_volume() {
    // Cycle : chaque tâche prétend descendre de l'autre.
    let dir = temp_dir("cycle");
    let path = dir.join("store.sqlite");
    seed_v148(
        &path,
        &[
            ("t-cycle-a", "c2", "inst-a", "ib2", "c1", Some("ia1")),
            ("t-cycle-b", "c1", "inst-b", "ia1", "c2", Some("ib2")),
        ],
    );
    assert!(DelegationStore::open(&path).is_err());
    drop(std::fs::remove_dir_all(dir));

    // Chaîne plus profonde que la borne de migration (10 maillons).
    let dir = temp_dir("profondeur");
    let path = dir.join("store.sqlite");
    chain_payload(&path, 10);
    assert!(DelegationStore::open(&path).is_err());
    drop(std::fs::remove_dir_all(dir));

    // Volume : au-delà de 4096 records, l'ouverture refuse.
    let dir = temp_dir("volume");
    let path = dir.join("store.sqlite");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(V148_DDL).unwrap();
        for i in 0..4097 {
            let task_id = format!("t-vol-{i}");
            conn.execute(
                "INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
                params![task_id, format!("inst-{i}"), format!("req-{task_id}"), b"v148", v148_payload(&task_id, "humain", "inst", "inst", &format!("agent-{i}"), Some("inst-c")).to_string()],
            ).unwrap();
        }
    }
    assert!(DelegationStore::open(&path).is_err());
    drop(std::fs::remove_dir_all(dir));
}

// ---------------------------------------------------------------------------
// Transaction seq : signal durable, lecture silencieuse, rollback.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_insert_seq_signal_durable_et_horodatages() {
    let dir = temp_dir("seq");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);

    let initial = meta(&store);
    assert_eq!(initial.seq, 0);
    let rx = store.change_receiver().unwrap();

    let mut premier = task("t-1", "humain", "inst", "inst", "agent-1", Some("inst-c"), "queued", 1_700_000_000);
    premier.mission_deadline_at = Some(1_700_000_060);
    store.insert(&premier, "req-t-1", b"canon").unwrap();

    let apres = meta(&store);
    assert_eq!(apres.seq, 1);
    assert_eq!(apres.generation, initial.generation);
    // Le signal n'est émis qu'après le commit.
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let changes = store.take_changes();
    let mutation = changes.get("humain").expect("mutation racine");
    assert_eq!(mutation.seq, 1);
    assert_eq!(mutation.generation, apres.generation);
    assert!(store.take_changes().is_empty());

    let saved = store.get("t-1").unwrap().unwrap();
    assert_eq!(saved.root_owner_agent_id, "humain");
    assert!(saved.updated_at >= saved.created_at);
    // Une échéance de mission horodate le démarrage, pas la fin.
    assert!(saved.started_at.is_some());
    assert_eq!(saved.completed_at, None);

    let mut terminal = task("t-2", "humain", "inst", "inst", "agent-2", Some("inst-c"), "result_available", 1_700_000_000);
    terminal.result = Some("sortie".into());
    store.insert(&terminal, "req-t-2", b"canon").unwrap();
    assert_eq!(meta(&store).seq, 2);
    assert!(store.get("t-2").unwrap().unwrap().completed_at.is_some());

    // Durabilité : la réouverture retrouve la même génération et la même seq.
    drop(store);
    let store = open_store(&path);
    let rouvert = meta(&store);
    assert_eq!(rouvert.seq, 2);
    assert_eq!(rouvert.generation, apres.generation);

    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn delegation149_lectures_silencieuses_et_save_inchange() {
    let dir = temp_dir("silence");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);
    for i in 1..=3 {
        let t = task(&format!("t-{i}"), "humain", "inst", "inst", &format!("agent-{i}"), Some("inst-c"), "queued", 100 * i as i64);
        store.insert(&t, &format!("req-t-{i}"), b"canon").unwrap();
    }
    assert_eq!(meta(&store).seq, 3);
    // Les insertions ont déjà publié leurs changements : on les consomme.
    let _ = store.take_changes();

    // Aucune lecture n'incrémente la projection ni ne signale.
    for _ in 0..100 {
        store.lineage_page("humain", 100, None, |_| false).unwrap();
        store.lineage_show("humain", "t-1", 0, 16_384, false).unwrap();
        store.get("t-1").unwrap();
        store.tasks().unwrap();
        store.pending().unwrap();
        store.for_mission("mission-t-1").unwrap();
    }
    assert_eq!(meta(&store).seq, 3);
    assert!(store.take_changes().is_empty());

    // Un save sans changement visible reste muet.
    let identique = store.get("t-1").unwrap().unwrap();
    store.save(&identique).unwrap();
    assert_eq!(meta(&store).seq, 3);

    // Un résultat qui change hors result_available n'est pas visible.
    let mut premature = store.get("t-1").unwrap().unwrap();
    premature.result = Some("sortie prématurée".into());
    store.save(&premature).unwrap();
    assert_eq!(meta(&store).seq, 3);

    // Un changement d'état visible incrémente et signale.
    let mut actif = store.get("t-1").unwrap().unwrap();
    actif.state = "working".into();
    store.save(&actif).unwrap();
    assert_eq!(meta(&store).seq, 4);
    let rx = store.change_receiver().unwrap();
    rx.recv_timeout(Duration::from_secs(2)).unwrap();

    // La racine et le parent ne suivent jamais un save.
    let mut reecrit = store.get("t-2").unwrap().unwrap();
    reecrit.root_owner_agent_id = String::new();
    reecrit.parent_task_id = Some("t-9".into());
    reecrit.state = "failed".into();
    reecrit.error = Some("échec".into());
    store.save(&reecrit).unwrap();
    let stocke = store.get("t-2").unwrap().unwrap();
    assert_eq!(stocke.root_owner_agent_id, "humain");
    assert_eq!(stocke.parent_task_id, None);
    assert_eq!(stocke.state, "failed");

    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn delegation149_conflits_insertion_rollback() {
    let dir = temp_dir("rollback");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);
    let base = task("t-1", "humain", "inst", "inst", "agent-1", Some("inst-c"), "queued", 100);
    store.insert(&base, "req-1", b"canon").unwrap();
    assert_eq!(meta(&store).seq, 1);
    // L'insertion initiale a publié son changement : on le consomme.
    let _ = store.take_changes();

    // task_id déjà pris.
    let dup_id = task("t-1", "humain", "inst", "inst", "agent-9", Some("inst-c"), "queued", 200);
    assert!(store.insert(&dup_id, "req-dup", b"canon").is_err());
    // child déjà pris.
    let dup_child = task("t-9", "humain", "inst", "inst", "agent-1", Some("inst-c"), "queued", 200);
    assert!(store.insert(&dup_child, "req-9", b"canon").is_err());
    // (owner_instance, request_id) déjà pris.
    let dup_request = task("t-8", "humain", "inst", "inst", "agent-8", Some("inst-c"), "queued", 200);
    assert!(store.insert(&dup_request, "req-1", b"canon").is_err());
    // mission déjà prise.
    let mut dup_mission = task("t-7", "humain", "inst", "inst", "agent-7", Some("inst-c"), "queued", 200);
    dup_mission.mission = "mission-t-1".into();
    assert!(store.insert(&dup_mission, "req-7", b"canon").is_err());

    // Aucun conflit n'incrémente ni ne signale ; la ligne d'origine reste.
    assert_eq!(meta(&store).seq, 1);
    assert!(store.take_changes().is_empty());
    let original = store.get("t-1").unwrap().unwrap();
    assert_eq!(original.child, "agent-1");
    assert_eq!(original.created_at, 100);
    assert!(store.get("t-9").unwrap().is_none());
    assert!(store.get("t-8").unwrap().is_none());
    assert!(store.get("t-7").unwrap().is_none());

    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

// ---------------------------------------------------------------------------
// Pages : bornes, ordre, curseur, budget d'octets.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_page_bornes_ordre_pagination_budget_et_curseur() {
    let dir = temp_dir("page");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);
    for limite in [0u32, 101] {
        assert!(matches!(store.lineage_page("humain", limite, None, |_| false), Err(E::InvalidRequest)));
    }

    let mut t1 = task("14900000-0000-4000-8000-0000000000a1", "humain", "inst", "inst", "agent-a1", Some("inst-c"), "queued", 100);
    t1.child_instance = None;
    let t2 = task("14900000-0000-4000-8000-0000000000a2", "humain", "inst", "inst", "agent-a2", Some("inst-c"), "queued", 100);
    let t3 = task("14900000-0000-4000-8000-0000000000b1", "humain", "inst", "inst", "agent-b1", Some("inst-c"), "result_available", 200);
    store.insert(&t1, "req-a1", b"c").unwrap();
    store.insert(&t2, "req-a2", b"c").unwrap();
    store.insert(&t3, "req-b1", b"c").unwrap();
    // Une autre racine, jamais visible depuis « humain ».
    let mut autre = task("14900000-0000-4000-8000-0000000000c1", "autre", "inst-x", "inst-x", "agent-x1", Some("inst-c"), "queued", 50);
    autre.root_owner_agent_id = "autre".into();
    store.insert(&autre, "req-x1", b"c").unwrap();

    // Ordre (created_at, task_id) et clés exactes de l'entrée.
    let page = store.lineage_page("humain", 100, None, |t| t.child == "agent-a1").unwrap();
    assert_eq!(page["version"], 1);
    assert_eq!(page["status"], "ok");
    assert_eq!(page["root_owner_agent_id"], "humain");
    let tasks = page["tasks"].as_array().unwrap();
    let ids: Vec<&str> = tasks.iter().map(|t| t["task_id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec!["14900000-0000-4000-8000-0000000000a1", "14900000-0000-4000-8000-0000000000a2", "14900000-0000-4000-8000-0000000000b1"]);
    assert_eq!(entry_keys_sorted(&tasks[0]), ENTRY_KEYS.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert_eq!(tasks[0]["journal_available"], true);
    assert_eq!(tasks[1]["journal_available"], false);
    assert_eq!(tasks[2]["result_available"], true);
    assert!(page["next_cursor"].is_null());

    // Pagination déterministe et curseur rejouable.
    let page1 = store.lineage_page("humain", 2, None, |_| false).unwrap();
    assert_eq!(page1["tasks"].as_array().unwrap().len(), 2);
    let curseur = page1["next_cursor"].as_str().unwrap().to_string();
    let rejoue = store.lineage_page("humain", 2, Some(&curseur), |_| false).unwrap();
    let page2 = store.lineage_page("humain", 2, Some(&curseur), |_| false).unwrap();
    assert_eq!(rejoue, page2);
    let ids2: Vec<&str> = page2["tasks"].as_array().unwrap().iter().map(|t| t["task_id"].as_str().unwrap()).collect();
    assert_eq!(ids2, vec!["14900000-0000-4000-8000-0000000000b1"]);
    assert!(page2["next_cursor"].is_null());

    // Un curseur est lié à sa racine.
    assert!(matches!(store.lineage_page("autre", 2, Some(&curseur), |_| false), Err(E::InvalidRequest)));
    // Un curseur refuse les formes ouvertes.
    let generation = meta(&store).generation;
    let corrompus = [
        format!("{{\"root\":\"autre\",\"generation\":\"{generation}\",\"seq\":4,\"created_at\":100,\"task_id\":\"14900000-0000-4000-8000-0000000000a2\"}}"),
        "[1,2]".to_string(),
        format!("{{\"root\":\"humain\",\"generation\":\"{generation}\",\"seq\":4,\"created_at\":100,\"task_id\":\"14900000-0000-4000-8000-0000000000a2\",\"extra\":1}}"),
        format!("{{\"root\":\"humain\",\"generation\":\"{generation}\",\"seq\":4,\"created_at\":100,\"task_id\":\"pas-un-uuid\"}}"),
        "x".repeat(2049),
    ];
    for curseur in &corrompus {
        assert!(
            matches!(store.lineage_page("humain", 2, Some(curseur), |_| false), Err(E::InvalidRequest)),
            "curseur accepté à tort: {curseur}"
        );
    }

    // Une mutation entre deux pages exige un resync, jamais une suite muette.
    let retard = store.lineage_page("humain", 2, None, |_| false).unwrap();
    let vieux_curseur = retard["next_cursor"].as_str().unwrap().to_string();
    let nouveau = task("14900000-0000-4000-8000-0000000000e9", "humain", "inst", "inst", "agent-z9", Some("inst-c"), "queued", 300);
    store.insert(&nouveau, "req-z9", b"c").unwrap();
    assert!(matches!(store.lineage_page("humain", 2, Some(&vieux_curseur), |_| false), Err(E::SnapshotChanged)));

    // Budget d'octets : deux entrées lourdes ne tiennent pas sur une page.
    let dir_lourd = temp_dir("budget");
    let path_lourd = dir_lourd.join("store.sqlite");
    let store_lourd = open_store(&path_lourd);
    for i in 1..=2 {
        let mut lourd = task(&format!("14900000-0000-4000-8000-00000000d00{i}"), "humain", "inst", "inst", &format!("agent-lourd-{i}"), Some("inst-c"), "queued", 100 * i as i64);
        lourd.cwd = "z".repeat(80_000);
        store_lourd.insert(&lourd, &format!("req-lourd-{i}"), b"c").unwrap();
    }
    let page1 = store_lourd.lineage_page("humain", 100, None, |_| false).unwrap();
    assert_eq!(page1["tasks"].as_array().unwrap().len(), 1);
    assert!(page1["next_cursor"].as_str().is_some());
    let curseur = page1["next_cursor"].as_str().unwrap().to_string();
    let page2 = store_lourd.lineage_page("humain", 100, Some(&curseur), |_| false).unwrap();
    assert_eq!(page2["tasks"].as_array().unwrap().len(), 1);
    assert!(page2["next_cursor"].is_null());

    // Une entrée seule plus lourde que le budget refuse (ResourceLimit).
    let dir_geant = temp_dir("geant");
    let path_geant = dir_geant.join("store.sqlite");
    let store_geant = open_store(&path_geant);
    let mut geant = task("t-geant", "humain", "inst", "inst", "agent-geant", Some("inst-c"), "queued", 100);
    geant.cwd = "z".repeat(150_000);
    store_geant.insert(&geant, "req-geant", b"c").unwrap();
    assert!(matches!(store_geant.lineage_page("humain", 100, None, |_| false), Err(E::ResourceLimit)));

    for dossier in [dir, dir_lourd, dir_geant] {
        drop(std::fs::remove_dir_all(dossier));
    }
}

// ---------------------------------------------------------------------------
// Entrée de liste : titre filtré, erreur bornée, états fermés.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_entree_titre_erreur_etats_fermes() {
    let mut tache = task("t-e1", "humain", "inst", "inst", "agent-e1", Some("inst-c"), "queued", 100);
    tache.request = delegate_request("req-e1", "\n \nPremière\u{7} ligne visible\nseconde");
    let entree = task_entry(&tache, true).unwrap();
    assert_eq!(entree["title"], "Première ligne visible");
    assert_eq!(entree["status"], "queued");
    assert_eq!(entree["journal_available"], true);
    assert_eq!(entree["result_available"], false);
    assert!(entree["error"].is_null());

    // Le titre est borné à 256 caractères filtrés.
    let mut long = task("t-e2", "humain", "inst", "inst", "agent-e2", Some("inst-c"), "working", 100);
    long.request = delegate_request("req-e2", &"é".repeat(400));
    let entree = task_entry(&long, false).unwrap();
    assert_eq!(entree["title"].as_str().unwrap().chars().count(), 256);

    // L'erreur est bornée à 1024 caractères filtrés.
    let mut fautive = task("t-e3", "humain", "inst", "inst", "agent-e3", Some("inst-c"), "failed", 100);
    fautive.error = Some(format!("x\u{7}{}", "y".repeat(2000)));
    let entree = task_entry(&fautive, false).unwrap();
    assert_eq!(entree["error"].as_str().unwrap().chars().count(), 1024);
    assert!(!entree["error"].as_str().unwrap().contains('\u{7}'));

    // Requête non Delegate et état inconnu : fermeture en StoreUnavailable.
    let mut catalogue = task("t-e4", "humain", "inst", "inst", "agent-e4", Some("inst-c"), "queued", 100);
    catalogue.request = NativeDelegationRequest::Catalogue;
    assert!(matches!(task_entry(&catalogue, false), Err(E::StoreUnavailable)));
    let mut etat_inconnu = task("t-e5", "humain", "inst", "inst", "agent-e5", Some("inst-c"), "archive-mystere", 100);
    etat_inconnu.state = "archive-mystere".into();
    assert!(matches!(task_entry(&etat_inconnu, false), Err(E::StoreUnavailable)));
}

// ---------------------------------------------------------------------------
// Show : bornes, offsets UTF-8, fenêtres, lecture sans effet de bord.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_show_bornes_offset_utf8_et_fenetres() {
    let dir = temp_dir("show");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);
    for limite in [0u32, 16_385] {
        assert!(matches!(store.lineage_show("humain", "t-s1", 0, limite, false), Err(E::InvalidRequest)));
    }

    let texte = "héllo wörld"; // 13 octets, 11 caractères ; é occupe les octets 1..3, ö les octets 8..10
    let mut tache = task("t-s1", "humain", "inst", "inst", "agent-s1", Some("inst-c"), "result_available", 100);
    tache.result = Some(texte.into());
    store.insert(&tache, "req-s1", b"c").unwrap();
    let total = texte.len() as u32;

    let plein = store.lineage_show("humain", "t-s1", 0, 16_384, true).unwrap();
    assert_eq!(plein["result"], texte);
    assert_eq!(plein["result_offset"], 0);
    assert_eq!(plein["result_total_bytes"], total);
    assert!(plein["result_next_offset"].is_null());

    // Fenêtre à une frontière valide puis fenêtre finale.
    let fenetre = store.lineage_show("humain", "t-s1", 3, 4, false).unwrap();
    assert_eq!(fenetre["result"], "llo ");
    assert_eq!(fenetre["result_next_offset"], 7);
    let fin = store.lineage_show("humain", "t-s1", total, 4, false).unwrap();
    assert_eq!(fin["result"], "");
    assert!(fin["result_next_offset"].is_null());

    // Offset au milieu d'un point de code : refus explicite.
    assert!(matches!(store.lineage_show("humain", "t-s1", 2, 4, false), Err(E::ResultOffsetInvalid)));
    assert!(matches!(store.lineage_show("humain", "t-s1", total + 5, 4, false), Err(E::ResultOffsetInvalid)));

    // Tâche non result_available : jamais de résultat exposé.
    let mut attente = task("t-s2", "humain", "inst", "inst", "agent-s2", Some("inst-c"), "working", 100);
    attente.result = Some("sortie cachée".into());
    store.insert(&attente, "req-s2", b"c").unwrap();
    let cache = store.lineage_show("humain", "t-s2", 0, 16_384, false).unwrap();
    assert!(cache["result"].is_null());
    assert_eq!(cache["result_total_bytes"], 0);
    assert_eq!(cache["task"]["result_available"], false);
    assert!(matches!(store.lineage_show("humain", "t-s2", 1, 16_384, false), Err(E::ResultOffsetInvalid)));

    // Racines étrangères et tâches absentes : même refus fermé.
    assert!(matches!(store.lineage_show("autre", "t-s1", 0, 16_384, false), Err(E::TaskUnavailable)));
    assert!(matches!(store.lineage_show("humain", "t-absent", 0, 16_384, false), Err(E::TaskUnavailable)));

    // Résultat au-delà de 256 Kio : StoreUnavailable.
    let mut geant = task("t-s3", "humain", "inst", "inst", "agent-s3", Some("inst-c"), "result_available", 100);
    geant.result = Some("z".repeat(256 * 1024 + 1));
    store.insert(&geant, "req-s3", b"c").unwrap();
    assert!(matches!(store.lineage_show("humain", "t-s3", 0, 16_384, false), Err(E::StoreUnavailable)));

    // Lecture sans effet de bord : la ligne SQL reste octet pour octet.
    let avant = {
        let brute = Connection::open(&path).unwrap();
        brute.query_row("SELECT payload FROM native_delegations WHERE task_id='t-s1'", [], |r| r.get::<_, String>(0)).unwrap()
    };
    for _ in 0..10 {
        store.lineage_show("humain", "t-s1", 0, 16_384, false).unwrap();
    }
    let apres = {
        let brute = Connection::open(&path).unwrap();
        brute.query_row("SELECT payload FROM native_delegations WHERE task_id='t-s1'", [], |r| r.get::<_, String>(0)).unwrap()
    };
    assert_eq!(avant, apres);
    let relu = store.get("t-s1").unwrap().unwrap();
    assert!(!relu.result_sent);
    assert_eq!(meta(&store).seq, 3);

    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

// ---------------------------------------------------------------------------
// Cancel : rejeu, mismatch, états, plafond de reçus.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_cancel_rejeu_mismatch_etats_et_plafond() {
    let dir = temp_dir("cancel");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);
    let t1 = task("t-c1", "humain", "inst", "inst", "agent-c1", Some("inst-c"), "queued", 100);
    let t2 = task("t-c2", "humain", "inst", "inst", "agent-c2", Some("inst-c"), "queued", 100);
    let mut t3 = task("t-c3", "humain", "inst", "inst", "agent-c3", Some("inst-c"), "result_available", 100);
    t3.result = Some("faite".into());
    store.insert(&t1, "req-c1", b"c").unwrap();
    store.insert(&t2, "req-c2", b"c").unwrap();
    store.insert(&t3, "req-c3", b"c").unwrap();

    // Annulation motrice : cancelling + signal séquentiel.
    let (recu, moteur) = store.human_cancel("humain", "t-c1", "u-1").unwrap();
    assert!(moteur);
    assert_eq!(recu, json!({"version":1,"task_id":"t-c1","status":"cancelling"}));
    assert_eq!(meta(&store).seq, 4);
    let rx = store.change_receiver().unwrap();
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(store.get("t-c1").unwrap().unwrap().state, "cancelling");
    // Les mutations précédentes ont publié leurs changements : on les consomme.
    let _ = store.take_changes();

    // Rejeu identique : même reçu, aucun signal, aucune mutation.
    let (recu2, moteur2) = store.human_cancel("humain", "t-c1", "u-1").unwrap();
    assert!(!moteur2);
    assert_eq!(recu2, recu);
    assert_eq!(meta(&store).seq, 4);
    assert!(store.take_changes().is_empty());

    // Même request_id sur une autre tâche : mismatch d'enveloppe.
    assert!(matches!(store.human_cancel("humain", "t-c2", "u-1"), Err(E::EnvelopeMismatch)));

    // Tâche déjà terminale : reçu sans mutation.
    let (recu3, moteur3) = store.human_cancel("humain", "t-c3", "u-3").unwrap();
    assert!(!moteur3);
    assert_eq!(recu3, json!({"version":1,"task_id":"t-c3","status":"result_available"}));
    assert_eq!(meta(&store).seq, 4);

    // Une seconde demande sur cancelling : reçu courant, sans incrément.
    let (recu4, moteur4) = store.human_cancel("humain", "t-c1", "u-4").unwrap();
    assert!(moteur4);
    assert_eq!(recu4["status"], "cancelling");
    assert_eq!(meta(&store).seq, 4);

    // Racines étrangères et tâches absentes : même refus.
    assert!(matches!(store.human_cancel("autre", "t-c1", "u-5"), Err(E::TaskUnavailable)));
    assert!(matches!(store.human_cancel("humain", "t-absent", "u-5"), Err(E::TaskUnavailable)));

    // Plafond : 4096 reçus font fermer toute nouvelle demande.
    {
        let brute = Connection::open(&path).unwrap();
        brute.execute_batch("BEGIN").unwrap();
        for i in 0..4096 {
            brute.execute(
                "INSERT INTO native_delegation_cancel_receipts(root,request_id,task_id,receipt) VALUES('humain',?1,'t-c2','{}')",
                [format!("rempli-{i}")],
            ).unwrap();
        }
        brute.execute_batch("COMMIT").unwrap();
    }
    assert!(matches!(store.human_cancel("humain", "t-c2", "u-final"), Err(E::ResourceLimit)));
    // Le rejeu d'un reçu existant reste servi au-delà du plafond.
    let (recu5, moteur5) = store.human_cancel("humain", "t-c1", "u-1").unwrap();
    assert!(!moteur5);
    assert_eq!(recu5, recu);

    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

// ---------------------------------------------------------------------------
// parent_lineage et descendants : profondeur 8, racines croisées, plafond.
// ---------------------------------------------------------------------------

#[test]
fn delegation149_parent_lineage_profondeur_et_descendants() {
    let dir = temp_dir("profondeur-parent");
    let path = dir.join("store.sqlite");
    let store = open_store(&path);

    let racine = task("t-p1", "humain", "inst", "inst", "agent-p1", Some("inst-c"), "queued", 100);
    store.insert(&racine, "req-p1", b"c").unwrap();

    // Une chaîne de 7 nœuds existants admet un descendant de profondeur 8.
    for i in 2..=7 {
        let parent = store.get(&format!("t-p{}", i - 1)).unwrap().unwrap();
        let (racine, parent_id) = store.parent_lineage(&parent).unwrap();
        let mut enfant = task(&format!("t-p{i}"), "humain", "inst", "inst", &format!("agent-p{i}"), Some("inst-c"), "queued", 100 + i as i64);
        enfant.root_owner_agent_id = racine;
        enfant.parent_task_id = parent_id;
        store.insert(&enfant, &format!("req-p{i}"), b"c").unwrap();
    }
    let septieme = store.get("t-p7").unwrap().unwrap();
    let (racine, parent_id) = store.parent_lineage(&septieme).unwrap();
    assert_eq!(racine, "humain");
    assert_eq!(parent_id.as_deref(), Some("t-p7"));
    let mut huitieme = task("t-p8", "humain", "inst", "inst", "agent-p8", Some("inst-c"), "queued", 108);
    huitieme.root_owner_agent_id = racine;
    huitieme.parent_task_id = parent_id;
    store.insert(&huitieme, "req-p8", b"c").unwrap();

    // Une chaîne existante de 8 nœuds refuse un 9e maillon.
    let profond = store.get("t-p8").unwrap().unwrap();
    assert_eq!(store.parent_lineage(&profond).unwrap_err(), "task_depth_limit");

    // Racine croisée et parent absent : refus de filiation.
    let mut etranger = task("t-p9", "humain", "inst", "inst", "agent-p9", Some("inst-c"), "queued", 109);
    etranger.root_owner_agent_id = "autre".into();
    etranger.parent_task_id = Some("t-p8".into());
    assert_eq!(store.parent_lineage(&etranger).unwrap_err(), "parent_task_unavailable");
    let mut orphelin = task("t-p9", "humain", "inst", "inst", "agent-p9", Some("inst-c"), "queued", 109);
    orphelin.parent_task_id = Some("t-absent".into());
    assert_eq!(store.parent_lineage(&orphelin).unwrap_err(), "parent_task_unavailable");

    // Descendants : ordre par profondeur, racines séparées.
    let descendants = store.descendants("t-p1").unwrap();
    let ids: Vec<String> = descendants.iter().map(|t| t.task_id.clone()).collect();
    assert_eq!(ids, vec!["t-p2", "t-p3", "t-p4", "t-p5", "t-p6", "t-p7", "t-p8"]);
    assert!(store.descendants("t-absent").unwrap().is_empty());

    // Plafond descendant : 4097 fils directs ferment la lecture.
    let dir_volume = temp_dir("descendants-volume");
    let path_volume = dir_volume.join("store.sqlite");
    let store_volume = open_store(&path_volume);
    let socle = task("t-v0", "humain", "inst", "inst", "agent-v0", Some("inst-c"), "queued", 1);
    store_volume.insert(&socle, "req-v0", b"c").unwrap();
    for i in 1..=4097 {
        let mut fils = task(&format!("t-v{i}"), "humain", "inst", "inst", &format!("agent-v{i}"), Some("inst-c"), "queued", 1 + i as i64);
        fils.root_owner_agent_id = "humain".into();
        fils.parent_task_id = Some("t-v0".into());
        store_volume.insert(&fils, &format!("req-v{i}"), b"c").unwrap();
    }
    assert_eq!(store_volume.descendants("t-v0").unwrap_err(), "descendant_limit");

    drop(store);
    drop(store_volume);
    std::fs::remove_dir_all(dir).unwrap();
    std::fs::remove_dir_all(dir_volume).unwrap();
}
