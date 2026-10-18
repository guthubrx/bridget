//! Session 149 — volet G-P : admission délégataire avec fait de permissions
//! (S149-29..S149-32, oracles G-P-01/G-P-02/G-P-07). Harnais hérité de la
//! session 148 : état réel, registre fixture, aucune mission fourisseur.
//! `handle_locked` est utilisé volontairement : aucun tick, donc aucun
//! lancement caché — chaque `spawn_task` reste explicite dans le test.

#[path = "native_child_requests155_tests.rs"]
mod native_child_requests155_tests;

use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn fixture(
    label: &str,
) -> (
    DaemonState,
    DaemonConfig,
    Receiver<ManagedSupervisorCommand>,
) {
    let (st, config, rx, _reader) = fixture_with_reader(label);
    (st, config, rx)
}

/// Même harnais, mais le lecteur de la connexion de contrôle reste ouvert :
/// un `tick` qui écrit au parent n'échoue pas sur une socket fermée.
pub(in crate::daemon) fn fixture_with_reader(
    label: &str,
) -> (
    DaemonState,
    DaemonConfig,
    Receiver<ManagedSupervisorCommand>,
    BufReader<UnixStream>,
) {
    let label = format!(
        "n149-{}-{}",
        &label[..label.len().min(3)],
        &Uuid::new_v4().to_string()[..8]
    );
    let (mut st, config) = super::super::presence_tests::state_with_registered_agent(&label);
    let (writer, reader) = super::super::presence_tests::control_socket(&label);
    st.connections.insert("conn-1".into(), writer);
    let fleet_root = st.fixture_root.as_ref().unwrap().0.clone();
    std::fs::set_permissions(&fleet_root, std::fs::Permissions::from_mode(0o700)).unwrap();
    st.fleet = Arc::new(
        FleetSupervisor::open(
            &config.db_path,
            DesiredStateStore::at_path(fleet_root.join("fleet.json")),
            FleetConfig::from_env(),
        )
        .unwrap(),
    );
    let (tx, rx) = mpsc::channel();
    st.managed_tx = tx;
    // Racine projet partagée par les faits et les demandes ; profil Claude
    // privé dans la fixture, jamais celui du user.
    let project = fleet_root.join("project-149");
    fs::create_dir_all(&project).unwrap();
    let config_dir = fleet_root.join("config-149");
    fs::create_dir_all(&config_dir).unwrap();
    fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700)).unwrap();
    let config_dir_str = config_dir.to_string_lossy().into_owned();
    let mut env: crate::lifecycle::SourceEnvironment = Default::default();
    env.insert("HOME".into(), fleet_root.as_os_str().to_owned());
    // Fixture unitaire, jamais une preuve modèle : un vrai binaire Mach-O
    // nommé `claude` dans un PATH privé, que le contexte capture comme CLI directe.
    // `.git` borne la remontée des ancêtres : le ~/.claude réel n'est jamais lu.
    fs::create_dir_all(fleet_root.join(".git")).unwrap();
    let bin = fleet_root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::copy("/bin/echo", bin.join("claude")).unwrap();
    fs::set_permissions(bin.join("claude"), fs::Permissions::from_mode(0o755)).unwrap();
    env.insert(
        "PATH".into(),
        format!("{}:/usr/bin:/bin", bin.display()).into(),
    );
    env.insert(
        "CLAUDE_CONFIG_DIR".into(),
        config_dir.as_os_str().to_owned(),
    );
    st.source_env = env;
    st.registry = AgentRegistry::from_json(
        &format!(
            r#"{{"agents":{{
                "native-test":{{"command":"/bin/sh","args":["app-server"],"protocol":"codex_app_server","forbidden_env":[],"capabilities":{{"execution_paths":["codex_app_server"],"models":{{"custom/model":{{"efforts":["high"]}}}}}}}},
                "claude-child":{{"command":"claude","args":["-c","import time;time.sleep(120)"],"protocol":"claude_stream_json","forbidden_env":[],"pass_env":[],"claude_config_dir":"{config_dir_str}","capabilities":{{"execution_paths":["claude_stream_json"],"models":{{"custom/model":{{"efforts":["high"]}}}}}}}}
            }}}}"#
        ),
        Path::new("/tmp/native149-registry.json"),
    )
    .unwrap();
    st.conn_hosts.insert("conn-1".into(), st.host.clone());
    st.presences.get_mut("instance-1").unwrap().host = st.host.clone();
    st.delegation_store
        .grant(
            "instance-1",
            std::env::temp_dir().as_path(),
            SpawnPosture::Development,
            false,
        )
        .unwrap();
    (st, config, rx, reader)
}

pub(super) fn codex_fact(
    cwd: &str,
    sandbox: Value,
    approval: &str,
) -> bridget_transport::protocol::ProviderPermissions {
    bridget_transport::protocol::ProviderPermissions {
        version: 1,
        source: "native_wrapper".into(),
        run_id: "run-149".into(),
        provider_session_id: "sess-149".into(),
        provider_instance_id: "instance-1".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: cwd.into(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: json!({
            "kind":"codex",
            "approval_policy":approval,
            "approvals_reviewer":"user",
            "sandbox_policy":sandbox,
        }),
    }
}

/// Définition identique à celle du registre du harnais : lancement python
/// réel, profil Claude dans la fixture, aucun argument de droits.
fn claude_definition(config_dir: &Path) -> crate::registry::AgentDefinition {
    crate::registry::AgentDefinition {
        command: "claude".into(),
        args: vec!["-c".into(), "import time;time.sleep(120)".into()],
        protocol: "claude_stream_json".into(),
        forbidden_env: vec![],
        pass_env: vec![],
        claude_config_dir: Some(config_dir.to_string_lossy().into_owned()),
        permissions: "deny".into(),
        queue_capacity: 4,
        held_queue: false,
        notify_timeout_secs: 4,
        mcp: Default::default(),
        capabilities: Default::default(),
    }
}

pub(super) fn claude_fact(
    config_dir: &Path,
    cwd: &Path,
    env: &SourceEnvironment,
    mode: &str,
) -> bridget_transport::protocol::ProviderPermissions {
    let definition = claude_definition(config_dir);
    let mut policy = json!({
        "kind":"claude",
        "permission_mode":mode,
        "tools":{"type":"preset","preset":"claude_code"},
        "permission_callback":{"kind":"native_wrapper","tool_approval":if mode=="bypassPermissions"{"allow"}else{"prompt"},"plan_exit":"deny"},
        "settings_sources":"provider_default",
    });
    policy["launch_context"] =
        crate::native_permissions::capture_context_with_env(&definition, cwd, env).unwrap();
    bridget_transport::protocol::ProviderPermissions {
        version: 1,
        source: "native_wrapper".into(),
        run_id: "run-149".into(),
        provider_session_id: "sess-149".into(),
        provider_instance_id: "instance-1".into(),
        revision: 1,
        driver: "claude_stream_json".into(),
        cwd: cwd.to_string_lossy().into_owned(),
        runtime_mode: if mode == "bypassPermissions" {
            "full-access"
        } else {
            "approval-required"
        }
        .into(),
        interaction_mode: "default".into(),
        provider_policy: policy,
    }
}

/// `AgentRegistry::from_json` fusionne le registre hôte (claude, codex, cursor…
/// avec leurs binaires et arguments réels) : seules les entrées de la fixture
/// sont des oracles hermétiques.
fn fixture_entries(catalogue: &Value) -> Vec<&Value> {
    let entries: Vec<&Value> = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            matches!(
                entry["agent_type"].as_str(),
                Some("native-test" | "claude-child")
            )
        })
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "les deux entrées de la fixture sont présentes"
    );
    entries
}

pub(super) fn delegate_request(
    agent_type: &str,
    request_id: &str,
    cwd: &str,
    posture: Option<SpawnPosture>,
) -> NativeDelegationRequest {
    NativeDelegationRequest::Delegate {
        machine_id: None,
        project_id: None,
        resume_on_limit: None,
        fallback: Vec::new(),
        request_id: request_id.into(),
        agent_type: agent_type.into(),
        model: "custom/model".into(),
        effort: Some("high".into()),
        task: "Mission fixture 149".into(),
        cwd: cwd.into(),
        posture,
        reply_timeout: None,
        keep_alive_secs: None,
        brief: None,
        child_requests: None,
        workspace: None,
    }
}

pub(super) fn insert_fact(
    st: &mut DaemonState,
    fact: bridget_transport::protocol::ProviderPermissions,
    scope: Option<&str>,
) {
    st.native_permission_facts
        .insert("conn-1".into(), (fact, scope.map(str::to_owned)));
}

#[test]
fn native155_admission_forward_codex() {
    let (mut st, config, rx, _reader) = fixture_with_reader("fwd155");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let mut request = delegate_request(
        "native-test",
        "fwd155",
        cwd.to_str().unwrap(),
        Some(SpawnPosture::Development),
    );
    if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
        *child_requests = Some("forward".into());
    }
    let receipt = handle_locked("conn-1", request, &mut st).unwrap();
    let mut task = st
        .delegation_store
        .get(receipt["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(task.child_requests.as_deref(), Some("forward"));
    spawn_task(&mut st, &mut task).unwrap();
    let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
        panic!("lancement attendu")
    };
    assert_eq!(
        prepared
            .env
            .get(crate::wrapper::NATIVE_CHILD_REQUESTS_ENV)
            .and_then(|value| value.to_str()),
        Some("forward")
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native155_admission_forward_non_supporte() {
    let (mut st, config, rx, _reader) = fixture_with_reader("non155");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let mut request = delegate_request(
        "claude-child",
        "non155",
        cwd.to_str().unwrap(),
        Some(SpawnPosture::Development),
    );
    if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
        *child_requests = Some("forward".into());
    }
    assert_eq!(
        handle_locked("conn-1", request, &mut st).unwrap_err(),
        "child_requests_unsupported"
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(rx.try_recv().is_err());
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native155_admission_auto_review_refuse() {
    for case in [
        "auto_review",
        "reviewer_absent",
        "policy_incomplete",
        "parent_absent",
    ] {
        let (mut st, config, rx, _reader) = fixture_with_reader("auto155");
        let cwd = st
            .fixture_root
            .as_ref()
            .unwrap()
            .0
            .join("project-149")
            .canonicalize()
            .unwrap();
        let mut fact = codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        );
        match case {
            "auto_review" => fact.provider_policy["approvals_reviewer"] = json!("auto_review"),
            "reviewer_absent" => {
                fact.provider_policy
                    .as_object_mut()
                    .unwrap()
                    .remove("approvals_reviewer");
            }
            "policy_incomplete" => {
                fact.provider_policy
                    .as_object_mut()
                    .unwrap()
                    .remove("approval_policy");
            }
            _ => {}
        }
        if case != "parent_absent" {
            insert_fact(&mut st, fact, None);
        }
        let mut request = delegate_request(
            "native-test",
            "auto155",
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Development),
        );
        if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
            *child_requests = Some("forward".into());
        }
        assert_eq!(
            handle_locked("conn-1", request, &mut st).unwrap_err(),
            "child_requests_reviewer_unsupported",
            "{case}"
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        assert!(rx.try_recv().is_err());
        let _ = fs::remove_file(&config.db_path);
    }
}

#[test]
fn native155_admission_guardian_subagent_refuse() {
    let (mut st, config, rx, _reader) = fixture_with_reader("guard155");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    let mut fact = codex_fact(
        cwd.to_str().unwrap(),
        json!({"type":"dangerFullAccess"}),
        "never",
    );
    fact.provider_policy["approvals_reviewer"] = json!("guardian_subagent");
    insert_fact(&mut st, fact, None);
    let mut request = delegate_request(
        "native-test",
        "guard155",
        cwd.to_str().unwrap(),
        Some(SpawnPosture::Development),
    );
    if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
        *child_requests = Some("forward".into());
    }
    assert_eq!(
        handle_locked("conn-1", request, &mut st).unwrap_err(),
        "child_requests_reviewer_unsupported"
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(rx.try_recv().is_err());
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native155_admission_octets_canoniques_inchanges() {
    let request = delegate_request("native-test", "canonical155", "/tmp", None);
    let canonical = serde_json::to_vec(&request).unwrap();
    assert_eq!(canonical.as_slice(), br#"{"operation":"delegate","request_id":"canonical155","agent_type":"native-test","model":"custom/model","effort":"high","task":"Mission fixture 149","cwd":"/tmp","posture":null}"#);
    let mut explicit = request;
    if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut explicit {
        *child_requests = Some("deny".into());
    }
    let explicit = serde_json::to_vec(&explicit).unwrap();
    assert_ne!(canonical, explicit);
    assert_eq!(
        serde_json::from_slice::<Value>(&explicit).unwrap()["child_requests"],
        "deny"
    );
}

#[test]
fn native155_admission_rejeu_autre_valeur_envelope_mismatch() {
    let (mut st, config, rx, _reader) = fixture_with_reader("replay155");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let mut request = delegate_request(
        "native-test",
        "replay155",
        cwd.to_str().unwrap(),
        Some(SpawnPosture::Development),
    );
    let receipt = handle_locked("conn-1", request.clone(), &mut st).unwrap();
    let id = receipt["task_id"].as_str().unwrap();
    assert_eq!(
        handle_locked("conn-1", request.clone(), &mut st).unwrap()["task_id"],
        id
    );
    for mode in ["deny", "forward"] {
        if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
            *child_requests = Some(mode.into());
        }
        assert_eq!(
            handle_locked("conn-1", request.clone(), &mut st).unwrap_err(),
            "envelope_mismatch"
        );
    }
    let task = st.delegation_store.get(id).unwrap().unwrap();
    assert_eq!(task.child_requests, None);
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
    assert!(rx.try_recv().is_err());
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native155_admission_politique_enfant_inchangee() {
    let (mut st, config, rx, _reader) = fixture_with_reader("policy155");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    let fact = codex_fact(
        cwd.to_str().unwrap(),
        json!({"type":"workspaceWrite","networkAccess":false,"writableRoots":[cwd.to_str().unwrap()]}),
        "never",
    );
    insert_fact(&mut st, fact.clone(), None);
    let mut policies = Vec::new();
    for (key, mode) in [
        ("policy155-deny", None),
        ("policy155-forward", Some("forward")),
    ] {
        let mut request = delegate_request(
            "native-test",
            key,
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Development),
        );
        if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
            *child_requests = mode.map(str::to_owned);
        }
        let receipt = handle_locked("conn-1", request, &mut st).unwrap();
        let task = st
            .delegation_store
            .get(receipt["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap();
        policies.push(task.permission_snapshot.unwrap().child_policy);
    }
    assert_eq!(policies[0], policies[1]);
    assert_eq!(policies[1], fact.provider_policy);
    assert_eq!(policies[1]["approval_policy"], "never");
    assert_eq!(policies[1]["approvals_reviewer"], "user");
    assert!(rx.try_recv().is_err());
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native155_denyenv_absent_et_heritage_forward_retire() {
    for mode in [None, Some("deny")] {
        let (mut st, config, rx, _reader) = fixture_with_reader("deny155");
        let cwd = st
            .fixture_root
            .as_ref()
            .unwrap()
            .0
            .join("project-149")
            .canonicalize()
            .unwrap();
        let fact = codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        );
        insert_fact(&mut st, fact.clone(), None);
        let mut request = delegate_request(
            "native-test",
            "deny155",
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Development),
        );
        if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
            *child_requests = mode.map(str::to_owned);
        }
        let receipt = handle_locked("conn-1", request, &mut st).unwrap();
        let mut task = st
            .delegation_store
            .get(receipt["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap();
        spawn_task(&mut st, &mut task).unwrap();
        let ManagedSupervisorCommand::Start { mut prepared, .. } = rx.try_recv().unwrap() else {
            panic!("lancement attendu")
        };
        assert!(
            !prepared
                .env
                .contains_key(crate::wrapper::NATIVE_CHILD_REQUESTS_ENV)
        );
        assert_eq!(
            serde_json::from_str::<Value>(
                prepared.env[crate::native_permissions::CHILD_POLICY_ENV]
                    .to_str()
                    .unwrap()
            )
            .unwrap(),
            fact.provider_policy
        );
        prepared.env.insert(
            crate::wrapper::NATIVE_CHILD_REQUESTS_ENV.into(),
            "forward".into(),
        );
        native_child_env(&task, &mut prepared).unwrap();
        assert!(
            !prepared
                .env
                .contains_key(crate::wrapper::NATIVE_CHILD_REQUESTS_ENV)
        );
        assert_eq!(
            task.permission_snapshot.unwrap().child_policy,
            fact.provider_policy
        );
        let _ = fs::remove_file(&config.db_path);
    }
}

/// T001/T012 — même catalogue privé avant et après la session 157.
/// Cinq définitions Claude et leurs cinq variantes de découverte font dix entrées.
/// Aucun tick ni lancement : la mesure porte sur le gestionnaire du catalogue.
#[test]
#[ignore = "banc T001/T012 : exécution explicite dans un namespace privé"]
fn native157_catalogue_latence() {
    let (mut st, config, rx, _reader) = fixture_with_reader("lat157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    let template = st.registry.get("claude-child").unwrap();
    let definition = json!({
        "command": template.command,
        "args": template.args,
        "protocol": template.protocol,
        "forbidden_env": template.forbidden_env,
        "pass_env": template.pass_env,
        "claude_config_dir": template.claude_config_dir,
        "permissions": template.permissions,
        "capabilities": {
            "execution_paths": ["claude_stream_json"],
            "models": {"custom/model": {"efforts": ["high"]}}
        }
    });
    // from_json ajoute quatre défauts et les variantes project-discovery.
    // Cinq sources privées donnent exactement dix entrées, sans CLI réelle.
    let sources = vec!["codex", "claude", "cursor", "gemini", "native157-01"];
    let mut names: Vec<String> = sources.iter().map(|name| (*name).to_owned()).collect();
    names.extend(
        sources
            .iter()
            .map(|name| format!("project-discovery-{name}")),
    );
    names.sort_unstable();
    let agents: serde_json::Map<String, Value> = sources
        .iter()
        .map(|name| ((*name).to_owned(), definition.clone()))
        .collect();
    let registry = serde_json::to_string(&json!({"agents": agents})).unwrap();
    let registry_path = root.join("catalogue157.json");
    bridget_transport::fsutil::write_private_file_atomic(&registry_path, registry.as_bytes())
        .unwrap();
    st.registry = AgentRegistry::from_json(&registry, &registry_path).unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type": "dangerFullAccess"}),
            "never",
        ),
        None,
    );

    let mut elapsed_ns = Vec::with_capacity(50);
    for _ in 0..50 {
        let start = std::time::Instant::now();
        let catalogue =
            handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
        let elapsed: u64 = start.elapsed().as_nanos().try_into().unwrap();
        elapsed_ns.push(elapsed);
        // Les validations sont hors de la fenêtre chronométrée.
        let providers = catalogue["providers"].as_array().unwrap();
        let mut actual: Vec<&str> = providers
            .iter()
            .map(|provider| provider["agent_type"].as_str().unwrap())
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, names);
        for provider in providers {
            assert_eq!(provider["protocol"], "claude_stream_json");
        }
    }
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    let mut sorted = elapsed_ns.clone();
    sorted.sort_unstable();
    // Médiane des deux valeurs centrales ; p95 au rang supérieur ceil(0,95 × 50).
    let median_ns = (sorted[24] as f64 + sorted[25] as f64) / 2.0;
    let p95_ns = sorted[47];
    println!(
        "native157_catalogue_latence {}",
        json!({
            "providers": 10,
            "samples": elapsed_ns.len(),
            "median_ns": median_ns,
            "p95_ns": p95_ns,
            "elapsed_ns": elapsed_ns,
            "measurement": "handle_locked_catalogue",
            "warmup_calls": 0,
            "fixture": "native149_codex_full_never_5_claude_plus_5_discovery",
            "p95_method": "nearest_rank_48_of_50"
        })
    );
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain() {
    let (mut st, config, _rx) = fixture("catalogue-full");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    assert_eq!(catalogue["version"], 1);
    assert_eq!(catalogue["cwd_scope"], "project");
    assert_eq!(catalogue["cwd_root"], racine.to_str().unwrap());
    assert_eq!(catalogue["max_children"], 16);
    assert_eq!(catalogue["max_depth"], 8);
    // Aucune invitation à un grant humain : la forme de la réponse est fermée.
    let mut clés: Vec<&str> = catalogue
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    clés.sort();
    assert_eq!(
        clés,
        [
            "brief_max_bytes",
            "checks",
            "child_request_human_window_secs",
            "child_request_parent_window_secs",
            "child_requests_default",
            "cwd_root",
            "cwd_scope",
            "delegations_halted",
            "features",
            "keep_alive_max_secs",
            "keep_alive_min_secs",
            "max_children",
            "max_depth",
            "max_rounds",
            "mission_reply_limit_secs",
            "mission_reply_max_secs",
            "mission_reply_min_secs",
            "providers",
            "queue",
            "verdict_context",
            "version",
            "wait_timeout_default_secs",
            "wait_timeout_max_secs",
            "workspace_strategies"
        ]
    );
    assert_eq!(catalogue["mission_reply_limit_secs"], 3600);
    assert_eq!(catalogue["mission_reply_min_secs"], 60);
    assert_eq!(catalogue["mission_reply_max_secs"], 86_400);
    assert_eq!(catalogue["brief_max_bytes"], 16_384);
    assert_eq!(catalogue["keep_alive_min_secs"], 60);
    assert_eq!(catalogue["keep_alive_max_secs"], 7200);
    assert_eq!(catalogue["max_rounds"], 8);
    assert_eq!(catalogue["child_requests_default"], "deny");
    assert_eq!(catalogue["child_request_parent_window_secs"], 600);
    assert_eq!(catalogue["child_request_human_window_secs"], 3600);
    for entry in fixture_entries(&catalogue) {
        assert_eq!(
            entry["child_requests_supported"],
            entry["protocol"] == "codex_app_server"
        );
        assert_eq!(entry["inherit"], true, "{}", entry["agent_type"]);
        assert_eq!(entry["development"], true, "{}", entry["agent_type"]);
        assert!(entry["inherit_refusal"].is_null());
        assert!(entry["development_refusal"].is_null());
    }
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_catalogue_parent_lecteur_et_confinement_refuses_par_nom() {
    let (mut st, config, _rx) = fixture("catalogue-lecteur");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"readOnly","networkAccess":false}),
            "never",
        ),
        None,
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    for entry in fixture_entries(&catalogue) {
        assert_eq!(entry["development"], false, "{}", entry["agent_type"]);
        assert_eq!(
            entry["development_refusal"], "permission_not_inherited",
            "{}",
            entry["agent_type"]
        );
        assert_eq!(
            entry["inherit"], true,
            "la découverte reste possible : {}",
            entry["agent_type"]
        );
    }
    let _ = fs::remove_file(&config.db_path);

    let (mut st, config, _rx) = fixture("catalogue-workspace");
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"workspaceWrite"}),
            "never",
        ),
        None,
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    for entry in fixture_entries(&catalogue) {
        if entry["agent_type"] == "claude-child" {
            assert_eq!(entry["inherit_refusal"], "provider_confinement_unavailable");
            assert_eq!(entry["inherit"], false);
        } else {
            assert_eq!(entry["inherit"], true, "{}", entry["agent_type"]);
        }
    }
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_developpement_d_un_parent_lecteur_refuse_sans_tache_ni_invitation() {
    let (mut st, config, rx) = fixture("lecteur-dev");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"readOnly","networkAccess":false}),
            "never",
        ),
        None,
    );
    let erreur = handle_locked(
        "conn-1",
        delegate_request(
            "native-test",
            "req-149",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    // Le refus publié est le JSON exact : aucun champ de grant humain.
    assert_eq!(
        failure("permission_not_inherited"),
        json!({"status":"refused","code":"permission_not_inherited"})
    );
    assert!(
        st.delegation_store.tasks().unwrap().is_empty(),
        "aucune tâche insérée"
    );
    assert!(rx.try_recv().is_err(), "aucun lancement");
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_posture_omise_resout_le_developpement_effectif_du_fait() {
    let (mut st, config, rx) = fixture("posture-omise");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let vue = handle_locked(
        "conn-1",
        delegate_request("native-test", "req-149", racine.to_str().unwrap(), None),
        &mut st,
    )
    .unwrap();
    assert_eq!(vue["status"], "queued");
    assert_eq!(
        vue["posture"], "development",
        "la posture effective vient du fait, jamais d'un défaut silencieux"
    );
    assert!(
        rx.try_recv().is_err(),
        "handle_locked ne lance pas : spawn explicite seulement"
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_sans_fait_le_v1_est_refuse_et_les_lectures_survivent_au_tombstone() {
    let (mut st, config, rx) = fixture("v1-noinherit");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    // (b) jamais de fait : développement refusé, posture omise refusée aussi.
    for posture in [Some(SpawnPosture::Development), None] {
        let erreur = handle_locked(
            "conn-1",
            delegate_request(
                "native-test",
                "req-sans-fait",
                racine.to_str().unwrap(),
                posture,
            ),
            &mut st,
        )
        .unwrap_err();
        assert_eq!(erreur, "permission_attestation_unavailable");
        assert!(rx.try_recv().is_err());
    }
    // (a) le fait a existé puis a cessé (tombstone) : jamais de retombée v1.
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let vue = handle_locked(
        "conn-1",
        delegate_request(
            "native-test",
            "req-149",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap();
    let task_id = vue["task_id"].as_str().unwrap().to_string();
    st.native_permission_facts.remove("conn-1");
    let erreur = handle_locked(
        "conn-1",
        delegate_request(
            "native-test",
            "req-apres-tombstone",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_attestation_unavailable");
    assert!(rx.try_recv().is_err(), "aucun lancement sans preuve 149");
    // (c) identité et lectures : status et cancel continuent.
    let status = handle_locked(
        "conn-1",
        NativeDelegationRequest::Status {
            task_id: task_id.clone(),
        },
        &mut st,
    )
    .unwrap();
    assert_eq!(status["status"], "queued");
    let cancel = handle_locked(
        "conn-1",
        NativeDelegationRequest::Cancel { task_id },
        &mut st,
    )
    .unwrap();
    assert_eq!(cancel["status"], "cancelling");
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_un_fait_d_autre_demande_ne_donne_aucun_droit() {
    let (mut st, config, rx) = fixture("scope");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            racine.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        Some("autre-req"),
    );
    let erreur = handle_locked(
        "conn-1",
        delegate_request(
            "native-test",
            "req-149",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_attestation_unavailable");
    assert!(rx.try_recv().is_err());
    // Le fait borné à la demande exacte ouvre, le fait global aussi.
    for (scope, request_id) in [(Some("req-149"), "req-149"), (None, "req-global")] {
        insert_fact(
            &mut st,
            codex_fact(
                racine.to_str().unwrap(),
                json!({"type":"dangerFullAccess"}),
                "never",
            ),
            scope,
        );
        let vue = handle_locked(
            "conn-1",
            delegate_request(
                "native-test",
                request_id,
                racine.to_str().unwrap(),
                Some(SpawnPosture::Development),
            ),
            &mut st,
        )
        .unwrap();
        assert_eq!(vue["status"], "queued");
    }
    let _ = fs::remove_file(&config.db_path);
}

fn tache_enregistree(
    definition: bridget_transport::protocol::ResolvedAgentDefinition,
) -> crate::delegation::Task {
    crate::delegation::Task {
        remote_placement: None,
        remote_prepared: false,
        remote_result_slots: Vec::new(),
        remote_result_receipts: Vec::new(),
        substitution: None,
        task_id: Uuid::new_v4().to_string(),
        root_owner_agent_id: "racine-149".into(),
        parent_task_id: None,
        updated_at: unix_timestamp(),
        started_at: None,
        completed_at: None,
        owner: "enfant-149".into(),
        owner_instance: "instance-enfant-149".into(),
        origin_owner_instance: "instance-origin-149".into(),
        parent_execution_id: None,
        request: delegate_request(
            "native-test",
            "req-enfant",
            "/private/tmp",
            Some(SpawnPosture::Development),
        ),
        permission_snapshot: None,
        effective_posture: Some(SpawnPosture::Development),
        definition,
        cwd: "/private/tmp".into(),
        child: Uuid::new_v4().to_string(),
        child_instance: None,
        mission: Uuid::new_v4().to_string(),
        mission_deadline_at: None,
        created_at: unix_timestamp(),
        state: "queued".into(),
        result: None,
        error: None,
        result_sent: false,
        cleanup_done: false,
        failure_sent: false,
        reply_timeout: None,
        delivery_acked: false,
        acknowledged_at: None,
        round: 0,
        idle_deadline_at: None,
        round_body: None,
        rounds: Vec::new(),
        guarded: false,
        guard_released: false,
        unacked_rounds: Vec::new(),
        quota_waits: Vec::new(),
        quota_hold: None,
        launch_issued_at: None,
        mission_issued_at: None,
        restart_resumes: 0,
        cancel_stopped: Vec::new(),
        child_requests: None,
        workspace: None,
        workspace_report_mission: None,
    }
}

#[test]
fn native149_revocation_de_la_racine_coupe_enfant_et_petit_enfant() {
    let (mut st, config, _rx) = fixture("revocation");
    let (_, definition) = st
        .registry
        .resolved_definitions()
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    // Mission parente : `enfant-149` en est l'enfant, racine `racine-149`.
    let mut parent = tache_enregistree(definition.clone());
    parent.owner = "racine-149".into();
    parent.owner_instance = "instance-1".into();
    parent.child = "enfant-149".into();
    let parent_canonical = serde_json::to_vec(&parent.request).unwrap();
    st.delegation_store
        .insert(&parent, "req-parent", &parent_canonical)
        .unwrap();
    // Mission du petit-enfant : `enfant-149` la délègue, même racine.
    let mut task = tache_enregistree(definition);
    task.parent_task_id = Some(parent.task_id.clone());
    task.cwd = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    if let NativeDelegationRequest::Delegate { cwd, .. } = &mut task.request {
        *cwd = task.cwd.clone();
    }
    let canonical = serde_json::to_vec(&task.request).unwrap();
    st.delegation_store
        .insert(&task, "req-enfant", &canonical)
        .unwrap();
    // La révocation racine ferme d'abord le droit stable (table des révocations).
    st.delegation_store
        .grant_agent(
            "racine-149",
            "instance-1",
            std::env::temp_dir().as_path(),
            SpawnPosture::Discovery,
            true,
        )
        .unwrap();
    // parent_fact du descendant : coupé avant tout contrôle d'infrastructure.
    let erreur = parent_fact(
        &st,
        "enfant-149",
        "instance-enfant-149",
        Some("req-x"),
        None,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    // spawn_task du même descendant : coupé avant tout effet, sans lancement.
    let erreur = spawn_task(&mut st, &mut task).unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_spawn_refuse_les_entrees_gelees_changees_sans_lancement() {
    let (mut st, config, rx) = fixture("gelees");
    let racine = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    let settings = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("config-149")
        .join("settings.json");
    fs::write(&settings, br#"{"model":"fixture-149"}"#).unwrap();
    // La chaîne claude→claude seule porte un launch_context revérifiable.
    let config_dir = st.fixture_root.as_ref().unwrap().0.join("config-149");
    let fait = claude_fact(&config_dir, &racine, &st.source_env, "bypassPermissions");
    insert_fact(&mut st, fait, None);
    // Tâche A : admission puis lancement sur des entrées gelées intactes.
    let vue = handle_locked(
        "conn-1",
        delegate_request(
            "claude-child",
            "req-a",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap();
    let mut task = st
        .delegation_store
        .get(vue["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert!(
        task.permission_snapshot.is_some(),
        "le snapshot est figé à l'admission"
    );
    spawn_task(&mut st, &mut task).unwrap();
    assert!(matches!(
        rx.try_recv().unwrap(),
        ManagedSupervisorCommand::Start { .. }
    ));
    // Tâche B : admise intacte, source mutée avant le lancement : le recheck
    // du spawn refuse par son nom, le nombre de lancements reste à un.
    let vue = handle_locked(
        "conn-1",
        delegate_request(
            "claude-child",
            "req-b",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap();
    let mut task = st
        .delegation_store
        .get(vue["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    fs::write(&settings, br#"{"model":"mute-149"}"#).unwrap();
    let erreur = spawn_task(&mut st, &mut task).unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
    assert!(rx.try_recv().is_err(), "launch count inchangé");
    // Tâche C : la mutation précède l'admission : le recheck d'admission
    // refuse déjà, aucune tâche n'est créée.
    let erreur = handle_locked(
        "conn-1",
        delegate_request(
            "claude-child",
            "req-c",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
    assert!(rx.try_recv().is_err(), "launch count inchangé");
    // Tâche D : source supprimée depuis le fait : même refus nommé.
    fs::remove_file(&settings).unwrap();
    let erreur = handle_locked(
        "conn-1",
        delegate_request(
            "claude-child",
            "req-d",
            racine.to_str().unwrap(),
            Some(SpawnPosture::Development),
        ),
        &mut st,
    )
    .unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
    assert!(rx.try_recv().is_err(), "launch count inchangé");
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native153_ronde_politique_complete_matrice() {
    for (parent, target) in [
        ("codex", "native-test"),
        ("codex", "claude-child"),
        ("claude", "claude-child"),
        ("native", "native-test"),
    ] {
        for posture in [
            None,
            Some(SpawnPosture::Discovery),
            Some(SpawnPosture::Development),
        ] {
            let (mut st, config, rx, _parent_reader) = fixture_with_reader("round-policy");
            let root = st.fixture_root.as_ref().unwrap().0.clone();
            let cwd = root.join("project-149").canonicalize().unwrap();
            let fact = if parent == "claude" {
                claude_fact(
                    &root.join("config-149"),
                    &cwd,
                    &st.source_env,
                    "bypassPermissions",
                )
            } else {
                codex_fact(
                    cwd.to_str().unwrap(),
                    json!({"type":"dangerFullAccess"}),
                    "never",
                )
            };
            insert_fact(&mut st, fact, None);
            let mut readers = Vec::new();
            let mut task = None;
            let mut requester = "conn-1".to_owned();
            for stage in 0..if parent == "native" { 2 } else { 1 } {
                let mut request = delegate_request(
                    target,
                    &format!("policy-{stage}"),
                    cwd.to_str().unwrap(),
                    posture,
                );
                if let NativeDelegationRequest::Delegate {
                    keep_alive_secs, ..
                } = &mut request
                {
                    *keep_alive_secs = Some(120);
                }
                let view = handle_locked(&requester, request, &mut st).unwrap();
                let mut admitted = st
                    .delegation_store
                    .get(view["task_id"].as_str().unwrap())
                    .unwrap()
                    .unwrap();
                spawn_task(&mut st, &mut admitted).unwrap();
                let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap()
                else {
                    panic!("lancement de la fixture attendu")
                };
                let connection = format!("policy-child-{stage}");
                let (writer, reader) = super::super::presence_tests::control_socket("n153-policy");
                let host = st.host.clone();
                assert!(matches!(
                    handle_register_with_channel(
                        &connection,
                        2,
                        prepared.agent_type.clone(),
                        prepared.lease.name.clone(),
                        Some(host),
                        Some(prepared.resolved_definition.protocol.clone()),
                        ChannelReport::Known("unix".into()),
                        Some(
                            if prepared.resolved_definition.protocol == "codex_app_server" {
                                PresenceMode::Acp
                            } else {
                                PresenceMode::Cli
                            }
                        ),
                        None,
                        Some("macOS".into()),
                        Some(prepared.lease.instance_id.clone()),
                        None,
                        false,
                        Some(true),
                        &mut st,
                        false
                    ),
                    DaemonToWrapper::Registered { .. }
                ));
                st.connections.insert(connection.clone(), writer);
                st.fleet
                    .register_connected(
                        &prepared.lease,
                        &prepared.lease.instance_id,
                        unix_timestamp(),
                    )
                    .unwrap();
                st.managed_spawns
                    .get_mut(&admitted.task_id)
                    .unwrap()
                    .wrapper_conn = Some(connection.clone());
                admitted.state = "working".into();
                st.store
                    .create_request(&admitted.mission, &admitted.owner, &admitted.child, 3600)
                    .unwrap();
                st.delegation_store.save(&admitted).unwrap();
                readers.push(reader);
                task = Some(admitted);
                if parent == "native" && stage == 0 {
                    requester = connection;
                }
            }
            let mut task = task.unwrap();
            assert!(task.permission_snapshot.is_some());
            let policy = task
                .permission_snapshot
                .as_ref()
                .unwrap()
                .child_policy
                .clone();
            let definition = task.definition.clone();
            task.state = "result_available".into();
            task.result = Some("résultat de la politique héritée".into());
            task.result_sent = true;
            task.idle_deadline_at = Some(unix_timestamp() + 120);
            st.delegation_store.save(&task).unwrap();
            let view = handle_locked(
                &requester,
                NativeDelegationRequest::Continue {
                    task_id: task.task_id.clone(),
                    request_id: "policy-round-2".into(),
                    task: "relecture".into(),
                    brief: None,
                },
                &mut st,
            )
            .unwrap();
            assert_eq!(
                view["status"], "mission_pending",
                "parent={parent}, cible={target}, posture={posture:?}"
            );
            let saved = st.delegation_store.get(&task.task_id).unwrap().unwrap();
            assert_eq!(
                saved.permission_snapshot.as_ref().unwrap().child_policy,
                policy,
                "toute la politique, aucun champ retiré"
            );
            assert_eq!(saved.definition, definition);
            let NativeDelegationRequest::Delegate {
                posture: original, ..
            } = saved.request
            else {
                unreachable!()
            };
            assert_eq!(original, posture, "None reste la demande d'origine");
            assert_eq!(saved.child_instance, task.child_instance);
            assert!(rx.try_recv().is_err());
            let _ = fs::remove_file(&config.db_path);
        }
    }
}

#[test]
fn native153_ronde_sources_changees_et_attestation_courante() {
    for (case, expected) in [
        ("source", "settings_revision_changed"),
        ("reduite", "permission_not_inherited"),
        ("absente", "permission_attestation_unavailable"),
    ] {
        let (mut st, config, rx, _parent_reader) = fixture_with_reader("round-recheck");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").canonicalize().unwrap();
        let settings = root.join("config-149/settings.json");
        fs::write(&settings, br#"{"model":"fixture-153"}"#).unwrap();
        let target = if case == "source" {
            "claude-child"
        } else {
            "native-test"
        };
        let fact = if case == "source" {
            claude_fact(
                &root.join("config-149"),
                &cwd,
                &st.source_env,
                "bypassPermissions",
            )
        } else {
            codex_fact(
                cwd.to_str().unwrap(),
                json!({"type":"dangerFullAccess"}),
                "never",
            )
        };
        insert_fact(&mut st, fact, None);
        let mut request = delegate_request(target, "recheck-first", cwd.to_str().unwrap(), None);
        if let NativeDelegationRequest::Delegate {
            keep_alive_secs, ..
        } = &mut request
        {
            *keep_alive_secs = Some(120);
        }
        let view = handle_locked("conn-1", request, &mut st).unwrap();
        let mut task = st
            .delegation_store
            .get(view["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap();
        spawn_task(&mut st, &mut task).unwrap();
        let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
            panic!("lancement géré attendu")
        };
        let (writer, _reader) = super::super::presence_tests::control_socket("n153-recheck");
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "recheck-child",
                2,
                prepared.agent_type.clone(),
                prepared.lease.name.clone(),
                Some(host),
                Some(prepared.resolved_definition.protocol.clone()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Cli),
                None,
                Some("macOS".into()),
                Some(prepared.lease.instance_id.clone()),
                None,
                false,
                Some(true),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
        st.connections.insert("recheck-child".into(), writer);
        st.fleet
            .register_connected(
                &prepared.lease,
                &prepared.lease.instance_id,
                unix_timestamp(),
            )
            .unwrap();
        st.managed_spawns
            .get_mut(&task.task_id)
            .unwrap()
            .wrapper_conn = Some("recheck-child".into());
        task.state = "result_available".into();
        task.result = Some("résultat acquis avant recontrôle".into());
        task.result_sent = true;
        task.idle_deadline_at = Some(unix_timestamp() + 120);
        st.delegation_store.save(&task).unwrap();
        if case == "source" {
            fs::write(&settings, br#"{"model":"mutated-153"}"#).unwrap();
        }
        if case == "reduite" {
            insert_fact(
                &mut st,
                codex_fact(
                    cwd.to_str().unwrap(),
                    json!({"type":"readOnly","networkAccess":false}),
                    "never",
                ),
                None,
            );
        }
        if case == "absente" {
            st.native_permission_facts.clear();
        }
        let before =
            serde_json::to_vec(&st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap();
        let error = handle_locked(
            "conn-1",
            NativeDelegationRequest::Continue {
                task_id: task.task_id.clone(),
                request_id: "recheck-round-2".into(),
                task: "suite".into(),
                brief: None,
            },
            &mut st,
        )
        .unwrap_err();
        assert_eq!(error, expected, "cas={case}");
        assert_eq!(
            serde_json::to_vec(&st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap(),
            before
        );
        assert!(rx.try_recv().is_err());
        let _ = fs::remove_file(&config.db_path);
    }
}

// ── Reprise après redémarrage du daemon (recette R9.2) ──────────────────────
//
// Une mission native engagée ne change jamais de fournisseur. Les tests
// parcourent les vrais chemins de production : `reserve_managed_recoveries`
// (démarrage), `schedule_idempotent_delivery_recovery` et
// `schedule_execution_recovery` (reconnexion), puis `tick`.

const AGENT_149: &str = "89000000-0000-4000-8000-000000000102";

fn temp_canonique() -> std::path::PathBuf {
    std::env::temp_dir().canonicalize().unwrap()
}

/// Amène une mission native au stade voulu par le chemin de production :
/// admission, `spawn_task` (bail de flotte réel), puis connexion du fils.
pub(in crate::daemon) fn mission_native(
    st: &mut DaemonState,
    rx: &Receiver<ManagedSupervisorCommand>,
    request_id: &str,
    stade: &str,
) -> crate::delegation::Task {
    let cwd = temp_canonique();
    let vue = handle_locked(
        "conn-1",
        delegate_request(
            "native-test",
            request_id,
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Discovery),
        ),
        st,
    )
    .unwrap();
    let mut task = st
        .delegation_store
        .get(vue["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(task.state, "queued");
    if stade == "queued" {
        return task;
    }
    spawn_task(st, &mut task).unwrap();
    assert!(matches!(
        rx.try_recv().unwrap(),
        ManagedSupervisorCommand::Start { .. }
    ));
    if stade == "starting" {
        return task;
    }
    let lease = st.managed_spawns[&task.task_id].lease.clone();
    // `spawn_task` a déjà passé le bail en `Starting` (submit_spawn).
    st.fleet
        .register_connected(&lease, &lease.instance_id, unix_timestamp() + 1)
        .unwrap();
    task.state = stade.into();
    st.delegation_store.save(&task).unwrap();
    task
}

/// Agent persistant ordinaire (hors saga native), connecté dans la flotte.
fn equipier_ordinaire(st: &DaemonState) -> String {
    let nom = Uuid::new_v4().to_string();
    let now = unix_timestamp();
    let ordre = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "native-test".into(),
        project: None,
        requested_name: Some(nom.clone()),
        cwd: temp_canonique(),
        persistent: true,
        command_id: format!("ordinaire-{nom}"),
        issued_at: now,
        deadline_at: now + 60,
        ownership: None,
    };
    let lease = match st.fleet.request_spawn(&ordre, now).unwrap() {
        crate::fleet::SpawnSubmission::Start(lease) => lease,
        autre => panic!("bail persistant attendu: {autre:?}"),
    };
    let definition = st.registry.resolved_definition("native-test").unwrap();
    st.fleet.mark_starting(&lease, now, &definition).unwrap();
    st.fleet
        .register_connected(&lease, &lease.instance_id, now + 1)
        .unwrap();
    nom
}

/// Nouveau processus : le superviseur relit la base et `fleet.json`, et les
/// tables en mémoire du daemon précédent n'existent plus.
fn redemarrer_flotte(st: &mut DaemonState, config: &DaemonConfig) {
    let racine = st.fixture_root.as_ref().unwrap().0.clone();
    st.fleet = Arc::new(
        FleetSupervisor::open(
            &config.db_path,
            DesiredStateStore::at_path(racine.join("fleet.json")),
            FleetConfig::from_env(),
        )
        .unwrap(),
    );
    st.managed_spawns.clear();
    st.managed_by_instance.clear();
    st.recovery_commands.clear();
    st.recovering = false;
}

fn cycle_de_vie(
    st: &DaemonState,
    nom: &str,
) -> Option<crate::desired_state::DesiredLifecycleState> {
    st.fleet
        .desired_entry(nom)
        .unwrap()
        .map(|entree| entree.lifecycle_state)
}

#[test]
fn native149_redemarrage_echoue_les_missions_engagees_sans_relancer_ni_toucher_aux_durables() {
    use crate::desired_state::DesiredLifecycleState::{Running, Stopped};
    let (mut st, config, rx, _reader) = fixture_with_reader("redemarrage");
    let starting = mission_native(&mut st, &rx, "r-starting", "starting");
    let pending = mission_native(&mut st, &rx, "r-pending", "mission_pending");
    let working = mission_native(&mut st, &rx, "r-working", "working");
    let queued = mission_native(&mut st, &rx, "r-queued", "queued");
    let mut resultat = mission_native(&mut st, &rx, "r-result", "working");
    resultat.state = "result_available".into();
    resultat.result = Some("résultat durable 149".into());
    st.delegation_store.save(&resultat).unwrap();
    let mut attente = mission_native(&mut st, &rx, "r-wait", "working");
    attente.state = "waiting_for_children".into();
    st.delegation_store.save(&attente).unwrap();
    let ordinaire = equipier_ordinaire(&st);
    // Préconditions : les fils connectés sont bien des agents persistants actifs.
    for tache in [&pending, &working, &resultat, &attente] {
        assert_eq!(
            cycle_de_vie(&st, &tache.child),
            Some(Running),
            "{} connecté avant coupure",
            tache.state
        );
    }
    assert_eq!(cycle_de_vie(&st, &ordinaire), Some(Running));

    redemarrer_flotte(&mut st, &config);
    let reprises = super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();

    // FR013/014 : les deux identités natives attestées rejoignent l'ordinaire.
    let mut noms: Vec<&str> = reprises
        .iter()
        .map(|reprise| reprise.0.lease.name.as_str())
        .collect();
    let mut attendus = vec![
        ordinaire.as_str(),
        pending.child.as_str(),
        working.child.as_str(),
    ];
    noms.sort_unstable();
    attendus.sort_unstable();
    assert_eq!(noms, attendus);
    assert_eq!(st.recovery_commands.len(), 3);
    assert_eq!(st.managed_spawns.len(), 3);
    assert!(
        st.managed_spawns
            .values()
            .all(|record| attendus.contains(&record.lease.name.as_str()))
    );
    assert!(rx.try_recv().is_err(), "aucun Start pendant la réservation");
    assert_eq!(
        cycle_de_vie(&st, &ordinaire),
        Some(Running),
        "l'ordinaire reste actif"
    );

    // FR029 : ce Starting n'a aucun checkpoint durable d'identité de flotte.
    assert!(st.fleet.desired_entry(&starting.child).unwrap().is_none());
    let relue = st.delegation_store.get(&starting.task_id).unwrap().unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("failed", Some("native_restart_identity_mismatch"))
    );
    assert_eq!(relue.restart_resumes, 0);
    // FR013/015 : garder mission, état et instance jusqu'à la reconnexion.
    for tache in [&pending, &working] {
        let relue = st.delegation_store.get(&tache.task_id).unwrap().unwrap();
        assert_eq!(relue.state, tache.state);
        assert_eq!(relue.error, None);
        assert_eq!(relue.restart_resumes, 1);
        assert_eq!(relue.mission, tache.mission);
        assert_eq!(relue.child_instance, tache.child_instance);
    }
    // Non engagée, résultat et attente des descendants : inchangés, durables.
    let relue = st.delegation_store.get(&queued.task_id).unwrap().unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("queued", None)
    );
    let relue = st.delegation_store.get(&resultat.task_id).unwrap().unwrap();
    assert_eq!(relue.state, "result_available");
    assert_eq!(relue.result.as_deref(), Some("résultat durable 149"));
    assert_eq!(relue.error, None);
    let relue = st.delegation_store.get(&attente.task_id).unwrap().unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("waiting_for_children", None)
    );
    for tache in [&pending, &working] {
        assert_eq!(
            cycle_de_vie(&st, &tache.child),
            Some(Running),
            "{} reprenable",
            tache.state
        );
    }
    // FR017 : aucun nouveau fournisseur pour le résultat ou l'attente durables.
    for tache in [&resultat, &attente] {
        assert_eq!(
            cycle_de_vie(&st, &tache.child),
            Some(Stopped),
            "{} arrêté",
            tache.state
        );
    }
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

/// Dans le même processus, les baux actifs sont des candidats de reprise
/// (`recovery_candidates`). FR029 refuse celui sans entrée de flotte attestée.
#[test]
fn native149_bail_actif_d_un_enfant_natif_n_est_pas_un_candidat_de_reprise() {
    let (mut st, config, rx, _reader) = fixture_with_reader("bail-actif");
    let natif = mission_native(&mut st, &rx, "b-starting", "starting");
    assert!(st.fleet.desired_entry(&natif.child).unwrap().is_none());
    let ordinaire = equipier_ordinaire(&st);
    let candidats: Vec<String> = st
        .fleet
        .recovery_candidates()
        .iter()
        .map(|c| c.lease.name.clone())
        .collect();
    assert!(
        candidats.contains(&natif.child),
        "précondition : le bail natif est candidat ({candidats:?})"
    );

    let reprises = super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();

    let noms: Vec<&str> = reprises
        .iter()
        .map(|reprise| reprise.0.lease.name.as_str())
        .collect();
    assert_eq!(
        noms,
        vec![ordinaire.as_str()],
        "contrôle positif : l'ordinaire est repris, jamais le natif"
    );
    let relue = st.delegation_store.get(&natif.task_id).unwrap().unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("failed", Some("native_restart_identity_mismatch"))
    );
    assert_eq!(relue.restart_resumes, 0);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_apres_redemarrage_le_tick_admet_le_queue_et_ne_relance_aucune_mission_echouee() {
    let (mut st, config, rx, _reader) = fixture_with_reader("tick");
    let engagees = [
        mission_native(&mut st, &rx, "t-starting", "starting"),
        mission_native(&mut st, &rx, "t-pending", "mission_pending"),
        mission_native(&mut st, &rx, "t-working", "working"),
    ];
    // Fixture Working : sa mission a déjà une admission suivie durable.
    st.store
        .create_request(
            &engagees[2].mission,
            &engagees[2].owner,
            &engagees[2].child,
            u64::try_from(
                engagees[2]
                    .reply_timeout
                    .unwrap_or(MISSION_REPLY_LIMIT_SECS),
            )
            .unwrap(),
        )
        .unwrap();
    let queued = mission_native(&mut st, &rx, "t-queued", "queued");
    redemarrer_flotte(&mut st, &config);
    let reprises = super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();
    // FR013/014 : reprendre seulement les deux enfants à identité attestée.
    let mut noms: Vec<&str> = reprises
        .iter()
        .map(|reprise| reprise.0.lease.name.as_str())
        .collect();
    let mut attendus = vec![engagees[1].child.as_str(), engagees[2].child.as_str()];
    noms.sort_unstable();
    attendus.sort_unstable();
    assert_eq!(noms, attendus);
    assert!(st.recovering);
    assert_eq!(
        st.delegation_store
            .get(&queued.task_id)
            .unwrap()
            .unwrap()
            .state,
        "queued"
    );
    assert!(rx.try_recv().is_err(), "aucun Start pendant la réservation");
    // Fin des reprises préparées : utiliser les mêmes baux et le garde existant.
    let mut lecteurs_repris = Vec::new();
    for reprise in &reprises {
        let lease = &reprise.0.lease;
        let conn = format!("reprise-{}", lease.instance_id);
        let (writer, reader) = super::super::presence_tests::control_socket("r149repr");
        assert!(matches!(
            handle_register_with_channel(
                &conn,
                2,
                "native-test".into(),
                lease.name.clone(),
                Some(st.host.clone()),
                Some("codex_app_server".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".into()),
                Some(lease.instance_id.clone()),
                None,
                false,
                Some(true),
                &mut st,
                false,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        st.connections.insert(conn.clone(), writer);
        st.fleet
            .register_connected(lease, &lease.instance_id, unix_timestamp())
            .unwrap();
        st.managed_spawns
            .get_mut(&lease.command_id)
            .unwrap()
            .wrapper_conn = Some(conn);
        super::super::finish_recovery_command(&mut st, &lease.command_id);
        lecteurs_repris.push((lease.name.clone(), reader));
    }
    assert!(
        !st.recovering,
        "l'admission reprend après les deux connexions"
    );

    let shared = Arc::new(Mutex::new(st));
    tick(&shared);
    let st = shared.lock().unwrap();
    let mut lancements = Vec::new();
    while let Ok(commande) = rx.try_recv() {
        if let ManagedSupervisorCommand::Start { prepared, .. } = commande {
            lancements.push(prepared.lease.command_id.clone());
        }
    }
    assert_eq!(
        lancements,
        vec![queued.task_id.clone()],
        "seul le queue non engagé est admis"
    );
    let relue = st.delegation_store.get(&queued.task_id).unwrap().unwrap();
    assert_eq!(relue.state, "starting");
    let relue = st
        .delegation_store
        .get(&engagees[0].task_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("failed", Some("native_restart_identity_mismatch"))
    );
    assert_eq!(relue.restart_resumes, 0);
    assert_eq!(relue.child_instance, engagees[0].child_instance);
    for tache in &engagees[1..] {
        let relue = st.delegation_store.get(&tache.task_id).unwrap().unwrap();
        assert_eq!(
            (relue.state.as_str(), relue.error.as_deref()),
            ("working", None)
        );
        assert_eq!(relue.restart_resumes, 1);
        assert_eq!(relue.mission, tache.mission);
        let reprise = reprises
            .iter()
            .find(|reprise| reprise.0.lease.name == tache.child)
            .unwrap();
        assert_eq!(
            relue.child_instance.as_deref(),
            Some(reprise.0.lease.instance_id.as_str()),
            "le checkpoint suit l'instance reprise et attestée"
        );
    }
    // FR014/015/027 : la mission en attente est remise au bon wrapper, une fois.
    for (enfant, reader) in &mut lecteurs_repris {
        let trames = trames_recues(reader);
        let remises: Vec<_> = trames
            .iter()
            .filter_map(|trame| match trame {
                DaemonToWrapper::DeliverIdempotent { message, .. } => Some(message),
                _ => None,
            })
            .collect();
        if enfant.as_str() == engagees[1].child.as_str() {
            assert_eq!(
                remises.len(),
                1,
                "une remise de la mission en attente : {trames:?}"
            );
            let NativeDelegationRequest::Delegate { task: body, .. } = &engagees[1].request else {
                panic!("demande de délégation attendue")
            };
            assert_eq!(remises[0].id, engagees[1].mission);
            assert_eq!(remises[0].body, *body);
            assert!(remises[0].reply);
        } else {
            assert_eq!(enfant.as_str(), engagees[2].child.as_str());
            assert!(
                remises.is_empty(),
                "pas de remise brute de la mission déjà working : {trames:?}"
            );
        }
        eprintln!("T032149 réel : enfant={enfant}, remises={}", remises.len());
    }
    drop(st);
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

fn livraison_en_cours(
    st: &mut DaemonState,
    message: &bridget_core::BridgetMessage,
    octets: Vec<u8>,
    generation: u64,
) -> String {
    use super::super::{
        CLIENT_IDEMPOTENCY_HORIZON_SECS, CLIENT_ISSUED_AT_TOLERANCE_SECS, IdempotencyKey,
        OperationKind, SendDelivery,
    };
    let cle =
        IdempotencyKey::new("012_scope_bbbbbbbbbbbb", OperationKind::Send, &message.id).unwrap();
    let now = unix_now_secs();
    st.idempotency
        .reserve(
            &cle,
            message.id.as_bytes(),
            now,
            CLIENT_IDEMPOTENCY_HORIZON_SECS,
            now,
            CLIENT_ISSUED_AT_TOLERANCE_SECS,
        )
        .unwrap();
    let id = format!("delivery-{}", message.id);
    st.idempotency
        .begin_send_delivery(
            &cle,
            &SendDelivery {
                delivery_id: id.clone(),
                recipient_instance_id: "instance-1".into(),
                delivery_generation: generation,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: octets,
            },
        )
        .unwrap();
    id
}

#[test]
fn native149_reconnexion_ne_rejoue_pas_la_remise_incertaine_d_une_mission_native() {
    let (mut st, config, _rx, _reader) = fixture_with_reader("reconnexion");
    let definition = st.registry.resolved_definition("native-test").unwrap();
    let mut tache = tache_enregistree(definition);
    tache.child = AGENT_149.into();
    tache.state = "working".into();
    let canonique = serde_json::to_vec(&tache.request).unwrap();
    st.delegation_store
        .insert(&tache, "req-reco", &canonique)
        .unwrap();

    let mut mission = bridget_core::BridgetMessage::new("racine-149", AGENT_149, "mission native");
    mission.id = tache.mission.clone();
    let mut ordinaire = bridget_core::BridgetMessage::new("peer-a", AGENT_149, "message ordinaire");
    ordinaire.id = "msg-ordinaire-149".into();
    let id_mission =
        livraison_en_cours(&mut st, &mission, serde_json::to_vec(&mission).unwrap(), 11);
    let id_ordinaire = livraison_en_cours(
        &mut st,
        &ordinaire,
        serde_json::to_vec(&ordinaire).unwrap(),
        12,
    );
    let avant = st
        .idempotency
        .dispatching_deliveries_for_instance("instance-1", unix_now_secs())
        .unwrap();
    assert_eq!(
        avant.len(),
        2,
        "les deux remises sont en cours avant la reconnexion"
    );

    super::super::schedule_idempotent_delivery_recovery(&mut st, "conn-1", "instance-1");

    // Contrôle positif : l'ordinaire est rejoué à l'identique, une seule fois.
    let controles = st
        .pending_post_response_controls
        .get("conn-1")
        .expect("remise ordinaire différée");
    assert_eq!(controles.len(), 1);
    match &controles[0].message {
        DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            ..
        } => {
            assert_eq!(message.id, ordinaire.id);
            assert_eq!(delivery_id, &id_ordinaire);
        }
        autre => panic!("remise idempotente attendue: {autre:?}"),
    }
    // La mission native passe indéterminée, sans rejeu.
    let apres = st
        .idempotency
        .dispatching_deliveries_for_instance("instance-1", unix_now_secs())
        .unwrap();
    let restantes: Vec<&str> = apres
        .iter()
        .map(|livraison| livraison.delivery_id.as_str())
        .collect();
    assert_eq!(restantes, vec![id_ordinaire.as_str()]);
    assert!(!restantes.contains(&id_mission.as_str()));
    assert!(
        st.idempotency
            .mark_delivery_indeterminate(&id_mission, "instance-1", 11)
            .is_err(),
        "la remise native n'est plus 'dispatching' : déjà indéterminée"
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_reprise_d_execution_ignore_la_mission_native_et_garde_l_ordinaire() {
    // Contrôle positif : sans saga native, la reprise ordinaire crée sa continuation.
    let (mut ordinaire, config_ordinaire, _rx, _reader) = fixture_with_reader("exec-ordinaire");
    let mut message =
        bridget_core::BridgetMessage::new("humain", AGENT_149, "reprends exactement ceci");
    message.id = "message-ordinaire-149".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    ordinaire
        .execution_store
        .admit_starting_message(&message, "execution-ordinaire-149", unix_now_secs())
        .unwrap();
    super::super::schedule_execution_recovery(
        &mut ordinaire,
        "conn-1",
        "instance-1",
        AGENT_149,
        false,
    );
    let controles = ordinaire
        .pending_post_response_controls
        .get("conn-1")
        .expect("continuation ordinaire");
    assert_eq!(controles.len(), 1);
    assert!(
        matches!(&controles[0].message, DaemonToWrapper::DeliverIdempotent { message: livre, .. } if livre.id == message.id)
    );
    let _ = fs::remove_file(&config_ordinaire.db_path);

    // Mission native : même agent, même cible, mais l'exécution est celle de la saga.
    let (mut natif, config_natif, _rx, _reader) = fixture_with_reader("exec-natif");
    let definition = natif.registry.resolved_definition("native-test").unwrap();
    let mut tache = tache_enregistree(definition);
    tache.child = AGENT_149.into();
    tache.state = "working".into();
    let canonique = serde_json::to_vec(&tache.request).unwrap();
    natif
        .delegation_store
        .insert(&tache, "req-exec", &canonique)
        .unwrap();
    let mut mission = bridget_core::BridgetMessage::new("racine-149", AGENT_149, "mission native");
    mission.id = tache.mission.clone();
    mission.origin = Some(bridget_core::MessageOrigin::Human);
    mission.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    let execution = format!("execution-{}", tache.mission);
    natif
        .execution_store
        .admit_starting_message(&mission, &execution, unix_now_secs())
        .unwrap();
    super::super::schedule_execution_recovery(&mut natif, "conn-1", "instance-1", AGENT_149, false);
    assert!(
        natif
            .pending_post_response_controls
            .get("conn-1")
            .is_none_or(|controles| controles.is_empty()),
        "aucune remise générique de la mission native"
    );
    let instantane = natif
        .execution_store
        .execution_snapshot(&execution)
        .unwrap()
        .unwrap();
    assert_ne!(
        instantane.state, "unreachable",
        "l'exécution native n'est pas marquée reprise"
    );
    assert_eq!(
        natif
            .execution_store
            .recoverable_execution_ids_for_agent(AGENT_149)
            .unwrap(),
        vec![execution],
        "aucune continuation ordinaire créée"
    );
    let _ = fs::remove_file(&config_natif.db_path);
}

// ---------------------------------------------------------------------------
// F2 (r2 réseau) : descendant perdu au redémarrage et résultat racine retenu.
// Les tests passent par les mêmes appels que la production : redémarrage de
// flotte, `reserve_managed_recoveries` (donc `prepare_restart`), puis `tick`.
// ---------------------------------------------------------------------------

use std::io::BufRead as _;

/// Racine native qui a capturé son vrai résultat et attend ses descendants.
fn racine_en_attente(
    st: &mut DaemonState,
    rx: &Receiver<ManagedSupervisorCommand>,
    id: &str,
) -> crate::delegation::Task {
    let mut racine = mission_native(st, rx, id, "working");
    racine.state = "waiting_for_children".into();
    racine.result = Some(format!("résultat réel capturé pour {id}"));
    st.delegation_store.save(&racine).unwrap();
    racine
}

/// Fait avancer une exécution admise (`starting`) jusqu'à `etat`, par le CAS
/// exact du store, comme le ferait le runtime avant la coupure.
fn avancer_execution(st: &DaemonState, execution_id: &str, etat: &str) {
    let chemin: &[&str] = match etat {
        "starting" => &[],
        "running" => &["running"],
        "completed" => &["running", "completed"],
        "interrupted" => &["running", "interrupted"],
        "failed" => &["failed"],
        "unreachable" => &["unreachable"],
        autre => panic!("état d'exécution de fixture inconnu: {autre}"),
    };
    for suivant in chemin {
        let courant = st
            .execution_store
            .execution_snapshot(execution_id)
            .unwrap()
            .unwrap();
        match st
            .execution_store
            .transition_if_current(
                execution_id,
                &courant.state,
                courant.revision,
                courant.generation,
                suivant,
                "fixture149",
                unix_timestamp(),
            )
            .unwrap()
        {
            ConditionalTransition::Applied(_) => {}
            autre => panic!("transition de fixture refusée: {autre:?}"),
        }
    }
}

/// Lien de flotte parent -> enfant, tel que le pose `spawn_task`, enfant connecté.
fn lier_descendant(
    st: &DaemonState,
    parent_instance: &str,
    enfant: &str,
    delegation: &str,
) -> crate::fleet::SpawnLease {
    let definition = st.registry.resolved_definition("native-test").unwrap();
    let now = unix_timestamp();
    let ordre = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "native-test".into(),
        project: None,
        requested_name: Some(enfant.into()),
        cwd: temp_canonique(),
        persistent: true,
        command_id: delegation.into(),
        issued_at: now,
        deadline_at: now + 60,
        ownership: Some(SpawnOwnership {
            parent_instance_id: parent_instance.into(),
            parent_execution_id: None,
            objective_id: None,
            delegation_id: Some(delegation.into()),
            project: None,
            role: "native_delegate".into(),
            max_children: Some(16),
            max_depth: Some(8),
        }),
    };
    let lease = match st.fleet.request_spawn(&ordre, now).unwrap() {
        crate::fleet::SpawnSubmission::Start(lease) => lease,
        autre => panic!("bail descendant attendu: {autre:?}"),
    };
    st.fleet.mark_starting(&lease, now, &definition).unwrap();
    st.fleet
        .register_connected(&lease, &lease.instance_id, now + 1)
        .unwrap();
    lease
}

/// Descendant natif de `racine` tel que l'ancien code l'a laissé après un
/// redémarrage : tâche `failed`/`unreachable`, lien de flotte posé, mais
/// exécution restée dans `etat_execution`. `cible` force le destinataire de
/// l'exécution quand le test veut une incohérence.
fn descendant_perdu(
    st: &mut DaemonState,
    racine: &crate::delegation::Task,
    id: &str,
    etat_execution: &str,
    cible: Option<&str>,
) -> (crate::delegation::Task, String) {
    let definition = st.registry.resolved_definition("native-test").unwrap();
    let parent_instance = racine.child_instance.clone().expect("racine connectée");
    let mut tache = tache_enregistree(definition.clone());
    tache.owner = racine.child.clone();
    tache.owner_instance = parent_instance.clone();
    tache.origin_owner_instance = parent_instance.clone();
    tache.root_owner_agent_id = racine.root_owner_agent_id.clone();
    tache.parent_task_id = Some(racine.task_id.clone());
    tache.cwd = temp_canonique().to_string_lossy().into_owned();
    tache.request = delegate_request("native-test", id, &tache.cwd, Some(SpawnPosture::Discovery));
    tache.state = "failed".into();
    tache.error = Some("unreachable".into());
    let now = unix_timestamp();
    let lease = lier_descendant(st, &parent_instance, &tache.child, &tache.task_id);
    tache.child_instance = Some(lease.instance_id.clone());
    let canonique = serde_json::to_vec(&tache.request).unwrap();
    st.delegation_store
        .insert(&tache, &format!("req-{id}"), &canonique)
        .unwrap();
    st.delegation_store.save(&tache).unwrap();

    let mut mission = bridget_core::BridgetMessage::new(
        &tache.owner,
        cible.unwrap_or(&tache.child),
        "mission descendante",
    );
    mission.id = tache.mission.clone();
    mission.origin = Some(bridget_core::MessageOrigin::Agent);
    mission.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    let execution_id = format!("execution-{}", tache.mission);
    assert!(
        st.execution_store
            .admit_starting_message(&mission, &execution_id, now)
            .unwrap()
    );
    avancer_execution(st, &execution_id, etat_execution);
    (tache, execution_id)
}

fn etat_execution(st: &DaemonState, execution_id: &str) -> (String, u64, u64) {
    let s = st
        .execution_store
        .execution_snapshot(execution_id)
        .unwrap()
        .unwrap();
    (s.state, s.revision, s.generation)
}

/// Trames de contrôle reçues par le propriétaire jusqu'à une courte accalmie.
fn trames_recues(reader: &mut BufReader<UnixStream>) -> Vec<DaemonToWrapper> {
    reader
        .get_ref()
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    let mut trames = Vec::new();
    loop {
        let mut ligne = String::new();
        match reader.read_line(&mut ligne) {
            Ok(0) => break,
            Ok(_) => trames.push(bridget_transport::protocol::decode(ligne.trim()).unwrap()),
            Err(erreur)
                if matches!(
                    erreur.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(erreur) => panic!("lecture du contrôle: {erreur}"),
        }
    }
    trames
}

fn remises_du_resultat(
    trames: &[DaemonToWrapper],
    racine: &crate::delegation::Task,
) -> Vec<(String, String)> {
    let id = format!("native-result-{}", racine.task_id);
    trames
        .iter()
        .filter_map(|trame| match trame {
            DaemonToWrapper::DeliverIdempotent {
                message,
                delivery_id,
                ..
            } if message.id == id => Some((delivery_id.clone(), message.body.clone())),
            _ => None,
        })
        .collect()
}

fn commandes_start(rx: &Receiver<ManagedSupervisorCommand>) -> Vec<String> {
    let mut lancements = Vec::new();
    while let Ok(commande) = rx.try_recv() {
        if let ManagedSupervisorCommand::Start { prepared, .. } = commande {
            lancements.push(prepared.lease.command_id.clone());
        }
    }
    lancements
}

#[test]
fn native149_f2_descendant_perdu_en_running_est_ferme_puis_la_racine_livre_son_vrai_resultat_une_fois()
 {
    let (mut st, config, rx, mut reader) = fixture_with_reader("f2-reprise");
    let racine = racine_en_attente(&mut st, &rx, "f2-racine");
    let non_engagee = mission_native(&mut st, &rx, "f2-queue", "queued");
    let (descendant, execution) = descendant_perdu(&mut st, &racine, "f2-desc", "running", None);
    // Précondition = F2 : tel que l'ancien code l'a laissé, le descendant
    // fermé mais son exécution `running` bloque la racine (`descendants_busy`).
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "précondition F2 : racine bloquée"
    );
    let (etat_avant, revision_avant, generation_avant) = etat_execution(&st, &execution);
    assert_eq!(etat_avant, "running");

    redemarrer_flotte(&mut st, &config);
    let reprises = super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();
    assert!(reprises.is_empty(), "aucun agent ordinaire à reprendre");

    // Réparation : exécution fermée par le CAS exact, tâche descendante intacte.
    let (etat, revision, generation) = etat_execution(&st, &execution);
    assert_eq!(etat, "unreachable");
    assert_eq!(
        revision,
        revision_avant + 1,
        "une seule fermeture, par le CAS"
    );
    assert_eq!(generation, generation_avant);
    let relue = st
        .delegation_store
        .get(&descendant.task_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("failed", Some("unreachable"))
    );
    assert!(
        !descendants_busy(&st, &racine).unwrap(),
        "la racine n'est plus bloquée"
    );
    // La mission engagée suivante reste une reprise interdite : aucune exécution
    // active n'est plus listée pour le descendant.
    assert!(
        st.execution_store
            .recoverable_execution_ids_for_agent(&descendant.child)
            .unwrap()
            .is_empty()
    );

    let shared = Arc::new(Mutex::new(st));
    tick(&shared);
    let st = shared.lock().unwrap();
    let relue = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
    assert_eq!(
        relue.state, "result_available",
        "la racine publie son résultat capturé"
    );
    assert!(
        relue.result_sent,
        "livré malgré l'enfant racine sans route primaire"
    );
    assert_eq!(relue.result, racine.result);
    assert_eq!(
        relue.child_instance, racine.child_instance,
        "aucune nouvelle instance"
    );
    let trames = trames_recues(&mut reader);
    let remises = remises_du_resultat(&trames, &racine);
    assert_eq!(remises.len(), 1, "une seule remise du résultat: {trames:?}");
    assert_eq!(
        Some(remises[0].1.as_str()),
        racine.result.as_deref(),
        "vrai résultat, pas un message de panne"
    );
    // F1 : aucun lancement hors la tâche non engagée, aucune relance du descendant.
    let starts = commandes_start(&rx);
    assert_eq!(
        starts,
        vec![non_engagee.task_id.clone()],
        "seule la tâche non engagée est admise"
    );
    let relue = st
        .delegation_store
        .get(&descendant.task_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("failed", Some("unreachable"))
    );
    assert_eq!(
        relue.child_instance, descendant.child_instance,
        "pas de nouveau fournisseur pour le descendant"
    );
    // Un tick de plus ne remet rien.
    drop(st);
    tick(&shared);
    let st = shared.lock().unwrap();
    assert!(
        remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty(),
        "aucune seconde remise"
    );
    assert!(commandes_start(&rx).is_empty());
    drop(st);
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f2_execution_terminale_n_est_jamais_rouverte_au_redemarrage() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f2-terminal");
    let racine = racine_en_attente(&mut st, &rx, "f2-term-racine");
    let (_, completee) = descendant_perdu(&mut st, &racine, "f2-term-completed", "completed", None);
    let (_, echouee) = descendant_perdu(&mut st, &racine, "f2-term-failed", "failed", None);
    let (_, injoignable) =
        descendant_perdu(&mut st, &racine, "f2-term-unreach", "unreachable", None);
    // Contrôle positif dans le même état : un `running` voisin, lui, est fermé.
    let (_, active) = descendant_perdu(&mut st, &racine, "f2-term-running", "running", None);
    let avant: Vec<_> = [&completee, &echouee, &injoignable]
        .iter()
        .map(|e| etat_execution(&st, e))
        .collect();

    prepare_restart(&mut st).unwrap();

    let apres: Vec<_> = [&completee, &echouee, &injoignable]
        .iter()
        .map(|e| etat_execution(&st, e))
        .collect();
    assert_eq!(
        avant, apres,
        "état, révision et génération terminaux inchangés"
    );
    assert_eq!(apres[0].0, "completed");
    assert_eq!(
        etat_execution(&st, &active).0,
        "unreachable",
        "contrôle positif : l'exécution active est fermée"
    );
    // Idempotent : un second redémarrage ne réécrit rien.
    let apres_active = etat_execution(&st, &active);
    prepare_restart(&mut st).unwrap();
    assert_eq!(etat_execution(&st, &active), apres_active);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f2_execution_d_un_autre_agent_est_refusee_sans_etre_modifiee() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f2-agent");
    let racine = racine_en_attente(&mut st, &rx, "f2-agent-racine");
    let (_, execution) = descendant_perdu(
        &mut st,
        &racine,
        "f2-agent-desc",
        "running",
        Some("agent-etranger-149"),
    );
    let avant = etat_execution(&st, &execution);

    let erreur = prepare_restart(&mut st).unwrap_err();

    assert_eq!(erreur, "native_execution_mismatch");
    assert_eq!(
        etat_execution(&st, &execution),
        avant,
        "l'exécution d'un autre agent n'est pas fermée"
    );
    let _ = fs::remove_file(&config.db_path);
}

/// Le CAS n'accepte que l'instantané exact : une révision ancienne est refusée
/// et laisse l'exécution intacte (le chemin `native_execution_changed` de
/// `prepare_restart` repose sur ce contrat ; il n'est pas atteignable sans
/// écrivain concurrent).
#[test]
fn native149_f2_cas_d_execution_refuse_un_instantane_perime() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f2-cas");
    let racine = racine_en_attente(&mut st, &rx, "f2-cas-racine");
    let (_, execution) = descendant_perdu(&mut st, &racine, "f2-cas-desc", "running", None);
    let instantane = st
        .execution_store
        .execution_snapshot(&execution)
        .unwrap()
        .unwrap();

    for (etat, revision, generation) in [
        ("starting", instantane.revision, instantane.generation),
        ("running", instantane.revision - 1, instantane.generation),
        ("running", instantane.revision, instantane.generation + 1),
    ] {
        let issue = st
            .execution_store
            .transition_if_current(
                &execution,
                etat,
                revision,
                generation,
                "unreachable",
                "test",
                unix_timestamp(),
            )
            .unwrap();
        assert!(
            matches!(issue, ConditionalTransition::Rejected(_)),
            "instantané périmé refusé: {issue:?}"
        );
        assert_eq!(etat_execution(&st, &execution).0, "running");
    }
    let appliquee = st
        .execution_store
        .transition_if_current(
            &execution,
            "running",
            instantane.revision,
            instantane.generation,
            "unreachable",
            "test",
            unix_timestamp(),
        )
        .unwrap();
    assert!(
        matches!(appliquee, ConditionalTransition::Applied(_)),
        "contrôle positif"
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f2_proprietaire_hors_ligne_garde_le_resultat_en_attente_puis_livre_une_seule_fois() {
    let (mut st, config, rx, mut reader) = fixture_with_reader("f2-hors-ligne");
    let racine = racine_en_attente(&mut st, &rx, "f2-hl-racine");
    let (_, execution) = descendant_perdu(&mut st, &racine, "f2-hl-desc", "running", None);
    redemarrer_flotte(&mut st, &config);
    super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();
    assert_eq!(etat_execution(&st, &execution).0, "unreachable");
    // Le propriétaire est hors ligne : même instance, même connexion, route retirée.
    let inscrit = st
        .router
        .unregister_by_conn("conn-1")
        .expect("propriétaire inscrit");
    let shared = Arc::new(Mutex::new(st));

    tick(&shared);
    tick(&shared);
    {
        let st = shared.lock().unwrap();
        let relue = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
        assert_eq!(
            relue.state, "result_available",
            "l'attente des descendants est levée sans propriétaire"
        );
        assert!(!relue.result_sent, "aucun faux result_sent");
        assert_eq!(relue.error.as_deref(), Some("native_result_owner_offline"));
        assert_eq!(
            relue.result, racine.result,
            "le vrai résultat reste durable"
        );
    }
    assert!(
        remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty(),
        "rien livré hors ligne"
    );

    // Reconnexion du même propriétaire : une seule remise.
    shared
        .lock()
        .unwrap()
        .router
        .register(
            &inscrit.agent_id,
            &inscrit.agent_type,
            &inscrit.connection_id,
        )
        .unwrap();
    tick(&shared);
    let remises = remises_du_resultat(&trames_recues(&mut reader), &racine);
    assert_eq!(remises.len(), 1, "une remise à la reconnexion");
    let relue = shared
        .lock()
        .unwrap()
        .delegation_store
        .get(&racine.task_id)
        .unwrap()
        .unwrap();
    assert!(relue.result_sent);
    assert_eq!(relue.error, None);
    tick(&shared);
    assert!(
        remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty(),
        "pas de seconde remise"
    );

    // ACK/replay durable : si l'écriture de `result_sent` est perdue, le rejeu
    // retrouve la clé d'idempotence et ne produit aucune nouvelle remise.
    {
        let st = shared.lock().unwrap();
        let mut perdue = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
        perdue.result_sent = false;
        st.delegation_store.save(&perdue).unwrap();
    }
    tick(&shared);
    assert!(
        remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty(),
        "le rejeu ne redélivre pas"
    );
    let relue = shared
        .lock()
        .unwrap()
        .delegation_store
        .get(&racine.task_id)
        .unwrap()
        .unwrap();
    assert!(relue.result_sent, "le rejeu retrouve la remise acceptée");
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

// ---------------------------------------------------------------------------
// F2 : la capacité de relayer le résultat retenu n'est jamais une donnée du
// message. Elle naît dans la saga, et chaque champ du résultat est recoupé.
// ---------------------------------------------------------------------------

/// Tâche racine prête à livrer : résultat durable, enfant vivant sous son instance.
fn racine_resultat_disponible(
    st: &mut DaemonState,
    rx: &Receiver<ManagedSupervisorCommand>,
    id: &str,
) -> crate::delegation::Task {
    let mut racine = mission_native(st, rx, id, "working");
    racine.state = "result_available".into();
    racine.result = Some(format!("résultat réel capturé pour {id}"));
    st.delegation_store.save(&racine).unwrap();
    racine
}

fn message_resultat(task: &crate::delegation::Task) -> bridget_core::BridgetMessage {
    let mut message = bridget_core::BridgetMessage::new(
        &task.child,
        &task.owner,
        task.result.as_deref().unwrap(),
    );
    message.id = format!("native-result-{}", task.task_id);
    message.in_reply_to = Some(task.mission.clone());
    message
}

/// Même identité interne que `send_as_with_result_authority`, posée à la main
/// pour interroger `authorize_retained_result` seul.
fn poser_connexion_saga(
    st: &mut DaemonState,
    task: &crate::delegation::Task,
    message: &bridget_core::BridgetMessage,
) -> String {
    let conn = format!("native-delegation:{}", message.id);
    st.conn_names.insert(conn.clone(), task.child.clone());
    st.conn_instances
        .insert(conn.clone(), task.child_instance.clone().unwrap());
    st.auxiliary_connections.insert(conn.clone());
    st.client_negotiations.insert(
        conn.clone(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope(&task.task_id),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    );
    conn
}

fn envoyer_avec_autorite(
    st: &mut DaemonState,
    task: &crate::delegation::Task,
    message: bridget_core::BridgetMessage,
) -> DaemonToWrapper {
    let autorite = RetainedResultAuthority {
        task_id: task.task_id.clone(),
    };
    let mut controles = Vec::new();
    send_as_with_result_authority(
        st,
        &task.child,
        task.child_instance.as_deref().unwrap(),
        &task.task_id,
        message,
        task.created_at,
        &mut controles,
        Some(&autorite),
    )
}

fn refus(reponse: &DaemonToWrapper) -> Option<&str> {
    match reponse {
        DaemonToWrapper::Nack { reason, .. } => Some(reason.as_str()),
        _ => None,
    }
}

#[test]
fn native149_f2_autorite_retenue_recoupe_chaque_champ_du_resultat() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f2-autorite");
    let task = racine_resultat_disponible(&mut st, &rx, "f2-aut-racine");
    let message = message_resultat(&task);
    let conn = poser_connexion_saga(&mut st, &task, &message);
    let autorite = RetainedResultAuthority {
        task_id: task.task_id.clone(),
    };
    let verifier = |st: &DaemonState, conn: &str, message: &bridget_core::BridgetMessage| {
        authorize_retained_result(st, &autorite, conn, message)
    };
    assert_eq!(
        verifier(&st, &conn, &message),
        Ok(()),
        "contrôle positif : le résultat exact passe"
    );

    // Champs du message : tout écart refuse, avant la moindre réservation.
    let mut variantes: Vec<(&str, bridget_core::BridgetMessage)> = Vec::new();
    for (nom, modifier) in [
        (
            "corps",
            Box::new(|m: &mut bridget_core::BridgetMessage| m.body.push('x'))
                as Box<dyn Fn(&mut bridget_core::BridgetMessage)>,
        ),
        ("corps vide", Box::new(|m| m.body = "  ".into())),
        ("expéditeur", Box::new(|m| m.from = "autre-agent".into())),
        ("destinataire", Box::new(|m| m.to = "autre-agent".into())),
        (
            "identifiant",
            Box::new(|m| m.id = "native-result-autre".into()),
        ),
        (
            "corrélation",
            Box::new(|m| m.in_reply_to = Some("autre-mission".into())),
        ),
        ("corrélation absente", Box::new(|m| m.in_reply_to = None)),
        ("réponse attendue", Box::new(|m| m.reply = true)),
        (
            "intention",
            Box::new(|m| m.intent = Some(bridget_core::MessageIntent::TriggerTurn)),
        ),
    ] {
        let mut variante = message.clone();
        modifier(&mut variante);
        variantes.push((nom, variante));
    }
    for (nom, variante) in &variantes {
        assert_eq!(
            verifier(&st, &conn, variante),
            Err("native_result_invalid".into()),
            "champ changé : {nom}"
        );
    }

    // Connexion : nom, instance, portée, statut auxiliaire et route primaire.
    assert_eq!(
        verifier(&st, "native-delegation:autre", &message),
        Err("native_result_invalid".into()),
        "connexion étrangère"
    );
    let nom = st
        .conn_names
        .insert(conn.clone(), "autre-agent".into())
        .unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_invalid".into()),
        "nom de connexion"
    );
    st.conn_names.insert(conn.clone(), nom);
    let instance = st
        .conn_instances
        .insert(conn.clone(), "instance-etrangere".into())
        .unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_invalid".into()),
        "instance de connexion"
    );
    st.conn_instances.insert(conn.clone(), instance);
    let negociation = st.client_negotiations.get(&conn).cloned().unwrap();
    st.client_negotiations.get_mut(&conn).unwrap().issuer_scope =
        crate::communication::issuer_scope("autre-tache");
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_invalid".into()),
        "portée d'émetteur"
    );
    st.client_negotiations.insert(conn.clone(), negociation);
    st.auxiliary_connections.remove(&conn);
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_invalid".into()),
        "connexion non auxiliaire"
    );
    st.auxiliary_connections.insert(conn.clone());
    assert_eq!(
        verifier(&st, &conn, &message),
        Ok(()),
        "état restauré : le témoin repasse"
    );

    // État durable : seule la tâche `result_available` au même résultat est admise.
    let mut modifiee = task.clone();
    modifiee.state = "waiting_for_children".into();
    st.delegation_store.save(&modifiee).unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_invalid".into()),
        "tâche non publiée"
    );
    modifiee.state = "result_available".into();
    modifiee.result = Some("autre résultat".into());
    st.delegation_store.save(&modifiee).unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_invalid".into()),
        "résultat durable différent"
    );
    st.delegation_store.save(&task).unwrap();
    assert_eq!(verifier(&st, &conn, &message), Ok(()));

    // Propriété : le propriétaire vivant doit être celui de la tâche.
    let mut autre_instance = task.clone();
    autre_instance.owner_instance = "instance-autre-149".into();
    st.delegation_store.save(&autre_instance).unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_owner_unavailable".into()),
        "autre instance propriétaire"
    );
    st.delegation_store.save(&task).unwrap();
    let inscrit = st.router.unregister_by_conn("conn-1").unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_owner_offline".into()),
        "propriétaire hors ligne"
    );
    st.router
        .register(
            &inscrit.agent_id,
            &inscrit.agent_type,
            &inscrit.connection_id,
        )
        .unwrap();
    assert_eq!(verifier(&st, &conn, &message), Ok(()));

    // Aucune remise d'un message refusé ne réserve la clé : le bon message passe ensuite.
    let mut corps_change = message.clone();
    corps_change.body.push_str(" falsifié");
    assert_eq!(
        refus(&envoyer_avec_autorite(&mut st, &task, corps_change)),
        Some("native_result_invalid")
    );
    let mut destinataire_change = message.clone();
    destinataire_change.to = "autre-agent".into();
    assert_eq!(
        refus(&envoyer_avec_autorite(&mut st, &task, destinataire_change)),
        Some("native_result_invalid")
    );
    let remise = envoyer_avec_autorite(&mut st, &task, message.clone());
    assert!(
        matches!(
            remise,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. }
                    | IdempotencyIssue::OutcomeUnknown {
                        delivery_id: Some(_),
                        ..
                    },
                ..
            }
        ),
        "refus sans réservation : pas d'EnvelopeMismatch ensuite: {remise:?}"
    );

    // Révocation : propriétaire révoqué coupe la livraison, même rejouée.
    // `send_as_with_result_authority` retire sa connexion interne : on la repose.
    let conn = poser_connexion_saga(&mut st, &task, &message);
    st.delegation_store
        .grant_agent(
            &task.owner,
            "instance-1",
            std::env::temp_dir().as_path(),
            SpawnPosture::Discovery,
            true,
        )
        .unwrap();
    assert_eq!(
        verifier(&st, &conn, &message),
        Err("native_result_authority_revoked".into()),
        "propriétaire révoqué"
    );
    let rejeu = envoyer_avec_autorite(&mut st, &task, message);
    assert_eq!(
        refus(&rejeu),
        Some("native_result_authority_revoked"),
        "même un rejeu vérifie la révocation d'abord"
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f2_message_externe_au_meme_format_ne_recoit_aucune_capacite_privee() {
    let (mut st, config, rx, mut reader) = fixture_with_reader("f2-externe");
    let task = racine_resultat_disponible(&mut st, &rx, "f2-ext-racine");
    let message = message_resultat(&task);
    let enfant_sans_route = st.router.get_agent(&task.child).is_none();
    assert!(
        enfant_sans_route,
        "précondition F2 : l'enfant n'a pas de route primaire"
    );

    // 1. Défaut inchangé : sans capacité, la saga elle-même est refusée par le garde expéditeur.
    let mut controles = Vec::new();
    let defaut = send_as(
        &mut st,
        &task.child,
        task.child_instance.as_deref().unwrap(),
        &task.task_id,
        message.clone(),
        task.created_at,
        &mut controles,
    );
    assert!(
        refus(&defaut).is_some_and(|raison| raison.contains("non attestée")),
        "garde par défaut inchangé: {defaut:?}"
    );

    // 2. Un agent attesté (le propriétaire) ne peut pas emprunter l'identité de l'enfant.
    let mut controles = Vec::new();
    let usurpation = super::super::handle_idempotent_send(
        "conn-1",
        message.clone(),
        message.id.clone(),
        unix_timestamp(),
        super::super::IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: 60,
        },
        &mut st,
        &mut controles,
    );
    assert!(
        refus(&usurpation).is_some_and(|raison| raison.contains("non attestée")),
        "usurpation refusée: {usurpation:?}"
    );
    assert!(controles.is_empty(), "aucune remise différée");

    // 3. La capture de réponse ne s'ouvre pas non plus à ce même format.
    assert_eq!(capture_reply(&mut st, "conn-1", &message), Ok(false));
    let relue = st.delegation_store.get(&task.task_id).unwrap().unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.result_sent),
        ("result_available", false)
    );
    assert!(
        remises_du_resultat(&trames_recues(&mut reader), &task).is_empty(),
        "rien livré à l'owner"
    );

    // 4. Rien n'a été réservé : la vraie capacité de la saga passe ensuite, une fois.
    let remise = envoyer_avec_autorite(&mut st, &task, message);
    assert!(
        matches!(
            remise,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. }
                    | IdempotencyIssue::OutcomeUnknown {
                        delivery_id: Some(_),
                        ..
                    },
                ..
            }
        ),
        "la saga livre après les refus externes: {remise:?}"
    );
    let _ = fs::remove_file(&config.db_path);
}

/// Chaîne complète sans écrire l'état à la main : l'enfant racine répond sur sa
/// vraie connexion (capture), son descendant est perdu au redémarrage, puis le
/// résultat capturé est livré une fois, enfant racine mort.
#[test]
fn native149_f2_reponse_reelle_capturee_survit_au_redemarrage_et_au_descendant_perdu() {
    let (mut st, config, rx, mut reader) = fixture_with_reader("f2-capture");
    let racine = mission_native(&mut st, &rx, "f2-cap-racine", "working");
    let (_, execution) = descendant_perdu(&mut st, &racine, "f2-cap-desc", "running", None);
    let type_agent = st
        .router
        .get_agent(&racine.owner)
        .unwrap()
        .agent_type
        .clone();
    let instance_enfant = racine.child_instance.clone().unwrap();
    st.router
        .register(&racine.child, &type_agent, "conn-enfant")
        .unwrap();
    st.conn_names
        .insert("conn-enfant".into(), racine.child.clone());
    st.conn_instances
        .insert("conn-enfant".into(), instance_enfant);
    let mut reponse = bridget_core::BridgetMessage::new(
        &racine.child,
        &racine.owner,
        "RÉPONSE-RACINE-149 capturée",
    );
    reponse.id = "reponse-racine-149".into();
    reponse.in_reply_to = Some(racine.mission.clone());
    assert_eq!(
        capture_reply(&mut st, "conn-enfant", &reponse),
        Ok(true),
        "capture par la connexion attestée de l'enfant"
    );
    let capturee = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
    assert_eq!(capturee.state, "waiting_for_children");
    assert_eq!(
        capturee.result.as_deref(),
        Some("RÉPONSE-RACINE-149 capturée")
    );
    // L'enfant racine meurt avec le daemon : plus de route ni de connexion.
    st.router.unregister_by_conn("conn-enfant").unwrap();
    st.conn_names.remove("conn-enfant");
    st.conn_instances.remove("conn-enfant");

    redemarrer_flotte(&mut st, &config);
    super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();
    assert_eq!(etat_execution(&st, &execution).0, "unreachable");

    let shared = Arc::new(Mutex::new(st));
    tick(&shared);
    let relue = shared
        .lock()
        .unwrap()
        .delegation_store
        .get(&racine.task_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.result_sent),
        ("result_available", true)
    );
    let remises = remises_du_resultat(&trames_recues(&mut reader), &racine);
    assert_eq!(remises.len(), 1);
    assert_eq!(
        remises[0].1, "RÉPONSE-RACINE-149 capturée",
        "le vrai résultat de la racine, enfant mort"
    );
    tick(&shared);
    assert!(remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty());
    assert!(
        commandes_start(&rx).is_empty(),
        "aucune relance de fournisseur pendant toute la chaîne"
    );
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f2_mission_engagee_est_fermee_avec_son_execution_et_l_ordinaire_reste_intact() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f2-engagee");
    let racine = racine_en_attente(&mut st, &rx, "f2-eng-racine");
    // Nouveau chemin (sans ancien état) : tâche encore `working`, exécution `running`.
    let (mut engagee, execution) =
        descendant_perdu(&mut st, &racine, "f2-eng-desc", "running", None);
    engagee.state = "working".into();
    engagee.error = None;
    st.delegation_store.save(&engagee).unwrap();
    // Témoin ordinaire : un agent persistant hors saga, avec sa propre exécution `running`.
    let ordinaire = equipier_ordinaire(&st);
    let mut travail =
        bridget_core::BridgetMessage::new("humain", &ordinaire, "travail ordinaire 149");
    travail.id = "travail-ordinaire-149".into();
    travail.origin = Some(bridget_core::MessageOrigin::Human);
    travail.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    let execution_ordinaire = "execution-ordinaire-f2-149";
    assert!(
        st.execution_store
            .admit_starting_message(&travail, execution_ordinaire, unix_timestamp())
            .unwrap()
    );
    avancer_execution(&st, execution_ordinaire, "running");
    let ordinaire_avant = etat_execution(&st, execution_ordinaire);
    let native_avant = etat_execution(&st, &execution);

    prepare_restart(&mut st).unwrap();

    // FR013/015/027 : une mission attestée reprend sans changer son exécution.
    let relue = st.delegation_store.get(&engagee.task_id).unwrap().unwrap();
    assert_eq!(
        (relue.state.as_str(), relue.error.as_deref()),
        ("working", None)
    );
    assert_eq!(relue.restart_resumes, 1);
    assert_eq!(relue.child_instance, engagee.child_instance);
    assert_eq!(relue.mission, engagee.mission);
    assert_eq!(
        etat_execution(&st, &execution),
        native_avant,
        "état, révision et génération de la mission reprenable sont conservés"
    );
    assert_eq!(
        etat_execution(&st, execution_ordinaire),
        ordinaire_avant,
        "l'exécution ordinaire n'est pas touchée"
    );
    assert_eq!(
        st.execution_store
            .recoverable_execution_ids_for_agent(&ordinaire)
            .unwrap(),
        vec![execution_ordinaire.to_string()],
        "l'ordinaire reste récupérable par sa reprise normale"
    );
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "la racine attend le descendant natif encore actif"
    );
    // Contrôle négatif : un descendant lié qui n'est pas une mission native perdue
    // (exécution ordinaire `running`) bloque toujours la racine après redémarrage.
    let parent_instance = racine.child_instance.clone().unwrap();
    let lie = Uuid::new_v4().to_string();
    lier_descendant(
        &st,
        &parent_instance,
        &lie,
        &format!("lien-ordinaire-{lie}"),
    );
    let mut vivant = bridget_core::BridgetMessage::new(&racine.child, &lie, "travail vivant 149");
    vivant.id = "travail-vivant-149".into();
    vivant.origin = Some(bridget_core::MessageOrigin::Agent);
    vivant.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    assert!(
        st.execution_store
            .admit_starting_message(&vivant, "execution-vivant-149", unix_timestamp())
            .unwrap()
    );
    avancer_execution(&st, "execution-vivant-149", "running");
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "témoin : un descendant réellement actif bloque"
    );
    prepare_restart(&mut st).unwrap();
    assert_eq!(
        etat_execution(&st, "execution-vivant-149").0,
        "running",
        "hors saga native : jamais fermé"
    );
    assert!(descendants_busy(&st, &racine).unwrap());
    let _ = fs::remove_file(&config.db_path);
}

/// Une ancienne interruption de contrôle garde une ligne de reprise : après la
/// clôture exacte de la mission native perdue, elle ne bloque plus la racine.
/// Témoin : la même ligne sur une mission encore active la bloque toujours.
#[test]
fn native149_f2_ligne_de_reprise_d_une_mission_native_perdue_ne_bloque_pas_la_racine() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f2-pause");
    let racine = racine_en_attente(&mut st, &rx, "f2-pause-racine");
    let (perdue, execution) =
        descendant_perdu(&mut st, &racine, "f2-pause-desc", "interrupted", None);
    st.execution_store
        .record_pause_interruption(&execution, &perdue.child, 1, unix_timestamp())
        .unwrap();
    assert_eq!(
        st.execution_store
            .recoverable_execution_ids_for_agent(&perdue.child)
            .unwrap(),
        vec![execution.clone()],
        "précondition : la ligne de reprise liste encore l'exécution"
    );

    assert!(
        !descendants_busy(&st, &racine).unwrap(),
        "mission perdue fermée : racine non bloquée"
    );

    // Témoin : même ligne, mais la mission n'est pas la mission native perdue.
    let mut encore_active = st.delegation_store.get(&perdue.task_id).unwrap().unwrap();
    encore_active.state = "working".into();
    encore_active.error = None;
    st.delegation_store.save(&encore_active).unwrap();
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "témoin : mission encore active, racine bloquée"
    );
    let _ = fs::remove_file(&config.db_path);
}
// ---------------------------------------------------------------------------
// Ronde r9 : E2 (enfant en file dont le propriétaire natif est perdu), F3
// (annulation et exécution restée active) et O4 (arrêt coopératif des
// wrappers natifs à la réconciliation du démarrage).
//
// Les tests d'arrêt lancent de VRAIS groupes de processus (`/bin/sh` en groupe
// propre, piège SIGTERM) et passent par `reconcile_stale_groups_with_native`,
// l'appel exact de `run()`. Chaque groupe est arrêté au plus par SIGTERM,
// individuellement, après contrôle de sa naissance.
// ---------------------------------------------------------------------------

/// Enfant en file d'un propriétaire natif déjà lancé : aucune instance, aucun
/// lien de flotte, aucune exécution. Seule la filiation (`parent_task_id`)
/// le rattache à la racine.
fn descendant_en_file(
    st: &mut DaemonState,
    racine: &crate::delegation::Task,
    id: &str,
) -> crate::delegation::Task {
    descendant_en_file_modifie(st, racine, id, |_| {})
}

/// `parent_task_id` et `root_owner_agent_id` sont figés par le store à
/// l'insertion (`save` les conserve) : une filiation contradictoire doit donc
/// être fabriquée AVANT `insert`, d'où cette mutation en amont.
fn descendant_en_file_modifie(
    st: &mut DaemonState,
    racine: &crate::delegation::Task,
    id: &str,
    mutation: impl FnOnce(&mut crate::delegation::Task),
) -> crate::delegation::Task {
    let definition = st.registry.resolved_definition("native-test").unwrap();
    let parent_instance = racine
        .child_instance
        .clone()
        .unwrap_or_else(|| "instance-absente".into());
    let mut tache = tache_enregistree(definition);
    tache.owner = racine.child.clone();
    tache.owner_instance = parent_instance.clone();
    tache.origin_owner_instance = parent_instance;
    tache.root_owner_agent_id = racine.root_owner_agent_id.clone();
    tache.parent_task_id = Some(racine.task_id.clone());
    tache.cwd = temp_canonique().to_string_lossy().into_owned();
    tache.request = delegate_request("native-test", id, &tache.cwd, Some(SpawnPosture::Discovery));
    mutation(&mut tache);
    let canonique = serde_json::to_vec(&tache.request).unwrap();
    st.delegation_store
        .insert(&tache, &format!("req-{id}"), &canonique)
        .unwrap();
    tache
}

fn etat_tache(st: &DaemonState, tache: &crate::delegation::Task) -> (String, Option<String>) {
    let relue = st.delegation_store.get(&tache.task_id).unwrap().unwrap();
    (relue.state, relue.error)
}

#[test]
fn native149_e2_enfant_en_file_au_proprietaire_perdu_echoue_et_la_racine_livre_son_vrai_resultat_une_fois()
 {
    let (mut st, config, rx, mut reader) = fixture_with_reader("e2-file");
    let racine = racine_en_attente(&mut st, &rx, "e2-racine");
    let imbrique = descendant_en_file(&mut st, &racine, "e2-imbrique");
    let externe = mission_native(&mut st, &rx, "e2-externe", "queued");
    assert_eq!(etat_tache(&st, &imbrique), ("queued".into(), None));
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "précondition E2 : l'enfant en file bloque la racine sans fin"
    );

    redemarrer_flotte(&mut st, &config);
    let reprises = super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();
    assert!(reprises.is_empty());

    assert_eq!(
        etat_tache(&st, &imbrique),
        ("failed".into(), Some("unreachable".into())),
        "son propriétaire natif est exclu de toute reprise"
    );
    assert_eq!(
        etat_tache(&st, &externe),
        ("queued".into(), None),
        "un enfant en file externe (sans filiation) reste normal"
    );
    assert!(
        !descendants_busy(&st, &racine).unwrap(),
        "la racine n'attend plus"
    );

    let shared = Arc::new(Mutex::new(st));
    tick(&shared);
    {
        let st = shared.lock().unwrap();
        let relue = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
        assert_eq!(relue.state, "result_available");
        assert!(relue.result_sent);
        assert_eq!(relue.result, racine.result, "le vrai résultat capturé");
        assert_eq!(relue.child_instance, racine.child_instance);
        let imbrique_relu = st.delegation_store.get(&imbrique.task_id).unwrap().unwrap();
        assert_eq!(
            imbrique_relu.child_instance, None,
            "aucun fournisseur lancé"
        );
    }
    let remises = remises_du_resultat(&trames_recues(&mut reader), &racine);
    assert_eq!(remises.len(), 1, "une seule remise du résultat");
    assert_eq!(Some(remises[0].1.as_str()), racine.result.as_deref());
    assert_eq!(
        commandes_start(&rx),
        vec![externe.task_id.clone()],
        "seul l'enfant en file externe est admis, jamais l'enfant imbriqué"
    );
    tick(&shared);
    assert!(remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty());
    assert!(commandes_start(&rx).is_empty());
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_e2_proprietaire_hors_ligne_garde_le_resultat_puis_livre_une_fois_a_la_reconnexion() {
    let (mut st, config, rx, mut reader) = fixture_with_reader("e2-hl");
    let racine = racine_en_attente(&mut st, &rx, "e2hl-racine");
    let imbrique = descendant_en_file(&mut st, &racine, "e2hl-imbrique");
    redemarrer_flotte(&mut st, &config);
    super::super::reserve_managed_recoveries(&mut st, unix_timestamp()).unwrap();
    assert_eq!(
        etat_tache(&st, &imbrique),
        ("failed".into(), Some("unreachable".into()))
    );
    let inscrit = st
        .router
        .unregister_by_conn("conn-1")
        .expect("propriétaire inscrit");
    let shared = Arc::new(Mutex::new(st));

    tick(&shared);
    {
        let st = shared.lock().unwrap();
        let relue = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
        assert_eq!(relue.state, "result_available");
        assert!(!relue.result_sent, "aucun faux result_sent hors ligne");
        assert_eq!(relue.error.as_deref(), Some("native_result_owner_offline"));
        assert_eq!(relue.result, racine.result);
    }
    assert!(remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty());

    shared
        .lock()
        .unwrap()
        .router
        .register(
            &inscrit.agent_id,
            &inscrit.agent_type,
            &inscrit.connection_id,
        )
        .unwrap();
    tick(&shared);
    let remises = remises_du_resultat(&trames_recues(&mut reader), &racine);
    assert_eq!(remises.len(), 1, "une remise unique à la reconnexion");
    assert_eq!(Some(remises[0].1.as_str()), racine.result.as_deref());
    tick(&shared);
    assert!(remises_du_resultat(&trames_recues(&mut reader), &racine).is_empty());
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_e2_filiation_contradictoire_ou_incomplete_est_refusee_sans_fermer_l_enfant() {
    type Variante = (
        &'static str,
        fn(&mut DaemonState, &crate::delegation::Task) -> crate::delegation::Task,
        &'static str,
    );
    let mismatch = "native_owner_identity_mismatch";
    let variantes: Vec<Variante> = vec![
        (
            "propriétaire d'un autre nom",
            |st, racine| {
                descendant_en_file_modifie(st, racine, "e2r-nom", |t| {
                    t.owner = "agent-etranger-149".into()
                })
            },
            mismatch,
        ),
        (
            "instance propriétaire différente",
            |st, racine| {
                descendant_en_file_modifie(st, racine, "e2r-inst", |t| {
                    t.owner_instance = "instance-etrangere".into()
                })
            },
            mismatch,
        ),
        (
            "instance d'origine différente",
            |st, racine| {
                descendant_en_file_modifie(st, racine, "e2r-orig", |t| {
                    t.origin_owner_instance = "instance-etrangere".into()
                })
            },
            mismatch,
        ),
        (
            "racine différente",
            |st, racine| {
                descendant_en_file_modifie(st, racine, "e2r-racine", |t| {
                    t.root_owner_agent_id = "autre-racine-149".into()
                })
            },
            mismatch,
        ),
        (
            "tâche parente inconnue",
            |st, racine| {
                descendant_en_file_modifie(st, racine, "e2r-inconnue", |t| {
                    t.parent_task_id = Some(Uuid::new_v4().to_string())
                })
            },
            "parent_task_unavailable",
        ),
        (
            "parent jamais lancé (aucune instance à exclure)",
            |st, racine| {
                let parent = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
                let mut jamais = parent.clone();
                jamais.task_id = Uuid::new_v4().to_string();
                jamais.mission = Uuid::new_v4().to_string();
                jamais.child = Uuid::new_v4().to_string();
                jamais.child_instance = None;
                jamais.state = "queued".into();
                jamais.result = None;
                let canonique = serde_json::to_vec(&jamais.request).unwrap();
                st.delegation_store
                    .insert(&jamais, "req-e2r-jamais", &canonique)
                    .unwrap();
                descendant_en_file_modifie(st, &jamais, "e2r-jamais", |_| {})
            },
            mismatch,
        ),
        (
            "instance du parent liée à une autre délégation",
            |st, racine| {
                // Même instance que la racine, mais la tâche parente est une
                // autre : le lien de flotte désigne `racine`, pas elle.
                let parent = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
                let mut autre = parent.clone();
                autre.task_id = Uuid::new_v4().to_string();
                autre.mission = Uuid::new_v4().to_string();
                autre.child = Uuid::new_v4().to_string();
                autre.result = None;
                autre.state = "working".into();
                let canonique = serde_json::to_vec(&autre.request).unwrap();
                st.delegation_store
                    .insert(&autre, "req-e2r-autre", &canonique)
                    .unwrap();
                descendant_en_file_modifie(st, &autre, "e2r-autre", |_| {})
            },
            mismatch,
        ),
        (
            "instance du parent sans lien de flotte",
            |st, racine| {
                let mut fantome = st.delegation_store.get(&racine.task_id).unwrap().unwrap();
                fantome.child_instance = Some("instance-fantome".into());
                st.delegation_store.save(&fantome).unwrap();
                descendant_en_file_modifie(st, &fantome, "e2r-fantome", |_| {})
            },
            mismatch,
        ),
    ];
    for (nom, fabriquer, attendu) in variantes {
        let (mut st, config, rx, _reader) = fixture_with_reader("e2-refus");
        let racine = racine_en_attente(&mut st, &rx, "e2r-racine-base");
        let contradictoire = fabriquer(&mut st, &racine);
        assert_eq!(etat_tache(&st, &contradictoire), ("queued".into(), None));

        let erreur = prepare_restart(&mut st).expect_err(nom);

        assert_eq!(erreur, attendu, "{nom}");
        assert_eq!(
            etat_tache(&st, &contradictoire),
            ("queued".into(), None),
            "{nom} : l'enfant n'est pas fermé"
        );
        let _ = fs::remove_file(&config.db_path);
    }
}

/// Descendant au statut `etat` (annulation) avec son exécution dans `exec`.
fn descendant_annule(
    st: &mut DaemonState,
    racine: &crate::delegation::Task,
    id: &str,
    etat: &str,
    exec: &str,
) -> (crate::delegation::Task, String) {
    let (mut tache, execution) = descendant_perdu(st, racine, id, exec, None);
    tache.state = etat.into();
    tache.error = None;
    st.delegation_store.save(&tache).unwrap();
    (tache, execution)
}

#[test]
fn native149_f3_annulation_et_execution_active_sont_fermees_une_fois_au_redemarrage() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f3-ferme");
    let racine = racine_en_attente(&mut st, &rx, "f3-racine");
    let mut cas = Vec::new();
    for etat in ["cancelled", "cancelling"] {
        for exec in ["starting", "running"] {
            let id = format!("f3-{etat}-{exec}");
            let (tache, execution) = descendant_annule(&mut st, &racine, &id, etat, exec);
            let avant = etat_execution(&st, &execution);
            assert_eq!(avant.0, exec, "précondition F3");
            cas.push((etat, exec, tache, execution, avant));
        }
    }

    prepare_restart(&mut st).unwrap();

    for (etat, exec, tache, execution, avant) in &cas {
        let (apres, revision, generation) = etat_execution(&st, execution);
        assert_eq!(apres, "unreachable", "{etat}/{exec} fermée");
        assert_eq!(avant.1 + 1, revision, "{etat}/{exec} : un seul CAS");
        assert_eq!(avant.2, generation);
        assert_eq!(
            etat_tache(&st, tache),
            (etat.to_string(), None),
            "la tâche garde son état d'annulation"
        );
    }
    // Idempotent : un second redémarrage n'écrit plus rien.
    let figees: Vec<_> = cas.iter().map(|c| etat_execution(&st, &c.3)).collect();
    prepare_restart(&mut st).unwrap();
    let relues: Vec<_> = cas.iter().map(|c| etat_execution(&st, &c.3)).collect();
    assert_eq!(figees, relues);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f3_execution_terminale_d_une_tache_annulee_n_est_jamais_rouverte() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f3-terminal");
    let racine = racine_en_attente(&mut st, &rx, "f3t-racine");
    let (_, completee) =
        descendant_annule(&mut st, &racine, "f3t-completed", "cancelled", "completed");
    let (_, echouee) = descendant_annule(&mut st, &racine, "f3t-failed", "cancelling", "failed");
    let (_, active) = descendant_annule(&mut st, &racine, "f3t-active", "cancelled", "running");
    let avant = [
        etat_execution(&st, &completee),
        etat_execution(&st, &echouee),
    ];

    prepare_restart(&mut st).unwrap();

    assert_eq!(
        [
            etat_execution(&st, &completee),
            etat_execution(&st, &echouee)
        ],
        avant,
        "état, révision et génération terminaux inchangés"
    );
    assert_eq!(avant[0].0, "completed");
    assert_eq!(
        etat_execution(&st, &active).0,
        "unreachable",
        "témoin : l'exécution active du même lot est fermée"
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f3_execution_d_un_autre_agent_refuse_la_fermeture_d_une_tache_annulee() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f3-agent");
    let racine = racine_en_attente(&mut st, &rx, "f3a-racine");
    let (mut tache, execution) = descendant_perdu(
        &mut st,
        &racine,
        "f3a-desc",
        "running",
        Some("agent-etranger-149"),
    );
    tache.state = "cancelled".into();
    tache.error = None;
    st.delegation_store.save(&tache).unwrap();
    let avant = etat_execution(&st, &execution);

    assert_eq!(
        prepare_restart(&mut st).unwrap_err(),
        "native_execution_mismatch"
    );
    assert_eq!(
        etat_execution(&st, &execution),
        avant,
        "aucune écriture à l'aveugle"
    );
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f3_cancel_tree_ferme_l_execution_exacte_avant_de_publier_cancelled() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f3-cancel");
    let racine = racine_en_attente(&mut st, &rx, "f3c-racine");
    let (ok, exec_ok) = descendant_annule(&mut st, &racine, "f3c-ok", "cancelling", "starting");
    let (termine, exec_terminee) =
        descendant_annule(&mut st, &racine, "f3c-term", "cancelling", "completed");
    let (etranger, exec_etrangere) = {
        let (mut t, e) = descendant_perdu(
            &mut st,
            &racine,
            "f3c-etr",
            "running",
            Some("agent-etranger-149"),
        );
        t.state = "cancelling".into();
        t.error = None;
        st.delegation_store.save(&t).unwrap();
        (t, e)
    };
    let terminee_avant = etat_execution(&st, &exec_terminee);
    let etrangere_avant = etat_execution(&st, &exec_etrangere);
    let (_, rev_ok, gen_ok) = etat_execution(&st, &exec_ok);
    let shared = Arc::new(Mutex::new(st));

    // `cancel_tree` : arrêts confirmés (flotte sans marqueur), puis clôture CAS.
    cancel_tree(&shared, &ok);
    cancel_tree(&shared, &termine);
    cancel_tree(&shared, &etranger);

    let st = shared.lock().unwrap();
    let relue = st.delegation_store.get(&ok.task_id).unwrap().unwrap();
    assert_eq!(relue.state, "cancelled");
    assert!(relue.cleanup_done);
    assert_eq!(
        etat_execution(&st, &exec_ok),
        ("unreachable".into(), rev_ok + 1, gen_ok)
    );

    let relue = st.delegation_store.get(&termine.task_id).unwrap().unwrap();
    assert_eq!(
        relue.state, "cancelled",
        "terminale : l'annulation se publie"
    );
    assert_eq!(
        etat_execution(&st, &exec_terminee),
        terminee_avant,
        "une exécution completed n'est jamais rouverte"
    );

    let relue = st.delegation_store.get(&etranger.task_id).unwrap().unwrap();
    assert_eq!(
        relue.state, "cancelling",
        "clôture refusée : rien n'est publié"
    );
    assert!(!relue.cleanup_done);
    assert_eq!(etat_execution(&st, &exec_etrangere), etrangere_avant);
    drop(st);
    drop(shared);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_f3_une_tache_annulee_a_execution_interrompue_ne_bloque_pas_la_racine() {
    let (mut st, config, rx, _reader) = fixture_with_reader("f3-filtre");
    let racine = racine_en_attente(&mut st, &rx, "f3f-racine");
    // Ancienne ligne de reprise laissée par une interruption de contrôle.
    let (annulee, execution) =
        descendant_annule(&mut st, &racine, "f3f-annulee", "cancelled", "interrupted");
    st.execution_store
        .record_pause_interruption(&execution, &annulee.child, 1, unix_timestamp())
        .unwrap();
    assert_eq!(
        st.execution_store
            .recoverable_execution_ids_for_agent(&annulee.child)
            .unwrap(),
        vec![execution.clone()],
        "précondition : la ligne de reprise liste encore l'exécution"
    );
    assert!(
        !descendants_busy(&st, &racine).unwrap(),
        "annulée + interrupted : plus bloquante"
    );
    // Témoins : la même exécution sur une tâche d'un autre état, ou une autre
    // instance, ou une exécution encore active, bloque toujours.
    let mut etat_autre = annulee.clone();
    etat_autre.state = "failed".into();
    etat_autre.error = Some("autre-cause".into());
    st.delegation_store.save(&etat_autre).unwrap();
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "failed hors unreachable : bloquante"
    );
    let mut instance_autre = annulee.clone();
    instance_autre.child_instance = Some("instance-etrangere".into());
    st.delegation_store.save(&instance_autre).unwrap();
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "autre instance : bloquante"
    );
    st.delegation_store.save(&annulee).unwrap();
    assert!(!descendants_busy(&st, &racine).unwrap());
    let (_, active) = descendant_annule(&mut st, &racine, "f3f-active", "cancelled", "running");
    assert!(
        descendants_busy(&st, &racine).unwrap(),
        "annulée mais exécution active : bloquante tant qu'elle n'est pas fermée"
    );
    prepare_restart(&mut st).unwrap();
    assert_eq!(etat_execution(&st, &active).0, "unreachable");
    assert!(
        !descendants_busy(&st, &racine).unwrap(),
        "fermée par le redémarrage"
    );
    let _ = fs::remove_file(&config.db_path);
}

// ── O4 : groupes de processus réels ─────────────────────────────────────────

use crate::managed_process::{ManagedIdentity, ManagedMarker, ManagedMarkerStore};
use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
use std::time::Instant as Horloge;

const PRET: &str = r#": > "$D/ready"; while :; do sleep 0.05; done"#;

/// Quitte par un code 0 coopératif juste après SIGTERM.
fn script_rapide() -> String {
    format!(r#"trap 'echo x > "$D/term"; exit 0' TERM; {PRET}"#)
}

/// Termine après un temps de fermeture : un SIGKILL se voit à son signal.
fn script_lent() -> String {
    format!(r#"trap 'echo x > "$D/term"; sleep 0.4; exit 0' TERM; {PRET}"#)
}

/// Ignore SIGTERM tant que `release` n'existe pas.
fn script_tetu() -> String {
    format!(r#"trap 'echo x > "$D/term"; [ -e "$D/release" ] && exit 0' TERM; {PRET}"#)
}

/// Ne sort qu'après avoir vu le SIGTERM des TROIS groupes : prouve un envoi
/// groupé avant toute attente.
fn script_barriere() -> String {
    format!(
        r#"trap 'echo x > "$D/term"; : > "$B/$ME"; n=0; while [ $n -lt 500 ] && {{ [ ! -e "$B/a" ] || [ ! -e "$B/b" ] || [ ! -e "$B/c" ]; }}; do sleep 0.02; n=$((n+1)); done; [ -e "$B/a" ] && [ -e "$B/b" ] && [ -e "$B/c" ] && exit 0; exit 7' TERM; {PRET}"#
    )
}

fn racine_privee(label: &str) -> std::path::PathBuf {
    let chemin = std::env::temp_dir().join(format!(
        "o4-{label}-{}",
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    fs::create_dir_all(&chemin).unwrap();
    fs::set_permissions(&chemin, fs::Permissions::from_mode(0o700)).unwrap();
    chemin
}

fn attendre_condition(quoi: &str, mut condition: impl FnMut() -> bool) {
    let fin = Horloge::now() + Duration::from_secs(15);
    while !condition() {
        assert!(Horloge::now() < fin, "délai dépassé : {quoi}");
        thread::sleep(Duration::from_millis(10));
    }
}

/// Groupe de processus réel, leader = `/bin/sh`. Un fil attend sa fin (il
/// supprime le zombie : sans cela `kill(-pgid, 0)` réussirait indéfiniment).
struct GroupeReel {
    nom: String,
    pgid: u32,
    naissance: u64,
    dossier: std::path::PathBuf,
    sortie: Arc<Mutex<Option<std::process::ExitStatus>>>,
}

impl GroupeReel {
    fn lancer(racine: &Path, nom: &str, script: &str, extra: &[(&str, &str)]) -> Self {
        let dossier = racine.join(format!("g-{nom}"));
        fs::create_dir_all(&dossier).unwrap();
        let mut commande = std::process::Command::new("/bin/sh");
        commande
            .arg("-c")
            .arg(script)
            .env("D", &dossier)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0);
        for (clef, valeur) in extra {
            commande.env(clef, valeur);
        }
        let mut enfant = commande.spawn().unwrap();
        let pgid = enfant.id();
        let sortie = Arc::new(Mutex::new(None));
        let copie = sortie.clone();
        thread::spawn(move || {
            let statut = enfant.wait().unwrap();
            *copie.lock().unwrap() = Some(statut);
        });
        let groupe = Self {
            nom: nom.into(),
            pgid,
            naissance: crate::managed_process::process_birth(pgid).unwrap(),
            dossier,
            sortie,
        };
        let pret = groupe.dossier.join("ready");
        attendre_condition("script prêt (piège installé)", || pret.exists());
        groupe
    }

    fn a_recu_term(&self) -> bool {
        self.dossier.join("term").exists()
    }

    fn vivant(&self) -> bool {
        self.sortie.lock().unwrap().is_none()
    }

    fn statut(&self) -> std::process::ExitStatus {
        attendre_condition("sortie du groupe", || !self.vivant());
        self.sortie.lock().unwrap().unwrap()
    }

    fn marqueur(&self, instance: &str, commande: &str, generation: u64) -> ManagedMarker {
        ManagedMarker {
            pgid: self.pgid,
            birth: self.naissance,
            instance_id: instance.into(),
            command_id: commande.into(),
            generation,
        }
    }

    fn identite(
        &self,
        instance: &str,
        commande: &str,
        generation: u64,
    ) -> (String, ManagedIdentity) {
        (
            self.nom.clone(),
            ManagedIdentity {
                instance_id: instance.into(),
                command_id: commande.into(),
                generation,
            },
        )
    }
}

impl Drop for GroupeReel {
    /// Nettoyage individuel et coopératif : SIGTERM seul, jamais SIGKILL, et
    /// seulement si la naissance prouve que le PID est encore le nôtre.
    fn drop(&mut self) {
        if self.vivant()
            && crate::managed_process::process_birth(self.pgid).is_ok_and(|b| b == self.naissance)
        {
            let _ = fs::write(self.dossier.join("release"), b"x");
            unsafe { libc::killpg(self.pgid as libc::pid_t, libc::SIGTERM) };
            let fin = Horloge::now() + Duration::from_secs(10);
            while self.vivant() && Horloge::now() < fin {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn poser_marqueur(racine: &Path, nom: &str, marqueur: &ManagedMarker) -> ManagedMarkerStore {
    let dossier = racine.join("managed");
    fs::create_dir_all(&dossier).unwrap();
    fs::write(
        dossier.join(format!("{nom}.json")),
        serde_json::to_vec(marqueur).unwrap(),
    )
    .unwrap();
    ManagedMarkerStore::at_directory(dossier)
}

fn marqueur_present(racine: &Path, nom: &str) -> bool {
    racine.join("managed").join(format!("{nom}.json")).exists()
}

const COURT: Duration = Duration::from_millis(100);
const SONDE: Duration = Duration::from_millis(10);

#[test]
fn native149_o4_wrapper_natif_recoit_sigterm_et_sort_de_lui_meme_sans_sigkill() {
    let racine = racine_privee("coop");
    let groupe = GroupeReel::lancer(&racine, "natif-coop", &script_lent(), &[]);
    let magasin = poser_marqueur(
        &racine,
        &groupe.nom,
        &groupe.marqueur("inst-o4", "cmd-o4", 3),
    );
    let identites = std::collections::BTreeMap::from([groupe.identite("inst-o4", "cmd-o4", 3)]);

    // Le délai ordinaire (100 ms, SIGKILL à 50 ms) tuerait ce groupe lent.
    let fait = magasin
        .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_secs(10))
        .unwrap();

    assert_eq!(fait, vec![groupe.nom.clone()]);
    assert!(groupe.a_recu_term());
    let statut = groupe.statut();
    assert_eq!(
        statut.code(),
        Some(0),
        "sortie coopérative, pas un signal: {statut:?}"
    );
    assert_eq!(statut.signal(), None);
    assert!(!marqueur_present(&racine, &groupe.nom));
    let _ = fs::remove_dir_all(&racine);
}

#[test]
fn native149_o4_groupe_ordinaire_garde_sigterm_puis_sigkill_apres_sa_moitie_de_delai() {
    let racine = racine_privee("ordinaire");
    let rapide = GroupeReel::lancer(&racine, "ordi-rapide", &script_rapide(), &[]);
    let tetu = GroupeReel::lancer(&racine, "ordi-tetu", &script_tetu(), &[]);
    let magasin = poser_marqueur(&racine, &rapide.nom, &rapide.marqueur("i1", "c1", 1));
    poser_marqueur(&racine, &tetu.nom, &tetu.marqueur("i2", "c2", 1));

    // Aucune identité native : chemin historique 500 ms + 500 ms.
    let fait = magasin
        .reconcile_stale_groups_with_native(
            Duration::from_secs(1),
            SONDE,
            &std::collections::BTreeMap::new(),
            Duration::ZERO,
        )
        .unwrap();

    assert_eq!(fait.len(), 2);
    assert_eq!(rapide.statut().code(), Some(0), "coopératif accepté");
    assert!(tetu.a_recu_term(), "SIGTERM d'abord");
    assert_eq!(
        tetu.statut().signal(),
        Some(libc::SIGKILL),
        "le groupe têtu est escaladé en SIGKILL : comportement ordinaire inchangé"
    );
    assert!(!marqueur_present(&racine, &rapide.nom));
    assert!(!marqueur_present(&racine, &tetu.nom));
    let _ = fs::remove_dir_all(&racine);
}

#[test]
fn native149_o4_aucune_escalade_native_marqueur_conserve_et_wrapper_vivant_apres_la_grace() {
    let racine = racine_privee("refus");
    let groupe = GroupeReel::lancer(&racine, "natif-tetu", &script_tetu(), &[]);
    let magasin = poser_marqueur(&racine, &groupe.nom, &groupe.marqueur("inst", "cmd", 1));
    let identites = std::collections::BTreeMap::from([groupe.identite("inst", "cmd", 1)]);

    let debut = Horloge::now();
    let erreur = magasin
        .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_millis(600))
        .unwrap_err();

    assert!(
        erreur
            .to_string()
            .contains("arrêt coopératif natif incomplet"),
        "{erreur}"
    );
    assert!(
        debut.elapsed() >= Duration::from_millis(550),
        "la grâce est attendue en entier"
    );
    assert!(groupe.a_recu_term());
    // Un SIGKILL envoyé juste avant le retour serait visible après un court délai.
    thread::sleep(Duration::from_millis(500));
    assert!(
        groupe.vivant(),
        "aucun SIGKILL natif : le wrapper vit encore"
    );
    assert!(marqueur_present(&racine, &groupe.nom), "marqueur conservé");
    // Le démarrage refusé laisse le groupe intact ; le test le libère par SIGTERM.
    drop(groupe);
    let _ = fs::remove_dir_all(&racine);
}

#[test]
fn native149_o4_la_grace_est_commune_tous_les_wrappers_natifs_recoivent_sigterm_avant_l_attente() {
    let racine = racine_privee("commune");
    let barriere = racine.join("barriere");
    fs::create_dir_all(&barriere).unwrap();
    let b = barriere.to_string_lossy().into_owned();
    let groupes: Vec<GroupeReel> = ["a", "b", "c"]
        .iter()
        .map(|me| {
            GroupeReel::lancer(
                &racine,
                &format!("natif-{me}"),
                &script_barriere(),
                &[("B", &b), ("ME", me)],
            )
        })
        .collect();
    let mut magasin = None;
    let mut identites = std::collections::BTreeMap::new();
    for groupe in &groupes {
        magasin = Some(poser_marqueur(
            &racine,
            &groupe.nom,
            &groupe.marqueur("inst", &groupe.nom, 1),
        ));
        identites.extend([groupe.identite("inst", &groupe.nom, 1)]);
    }

    // Un envoi séquentiel (signal, attente, signal suivant) bloquerait le
    // premier groupe jusqu'à l'échéance : la barrière ne tomberait jamais.
    let fait = magasin
        .unwrap()
        .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_secs(10))
        .unwrap();

    assert_eq!(fait, vec!["natif-a", "natif-b", "natif-c"]);
    for groupe in &groupes {
        assert_eq!(
            groupe.statut().code(),
            Some(0),
            "{} : sortie coopérative",
            groupe.nom
        );
        assert!(!marqueur_present(&racine, &groupe.nom));
    }
    let _ = fs::remove_dir_all(&racine);
}

#[test]
fn native149_o4_lot_mixte_garde_la_grace_native_et_l_escalade_ordinaire() {
    let racine = racine_privee("mixte");
    let natif = GroupeReel::lancer(&racine, "natif-mixte", &script_lent(), &[]);
    let ordinaire = GroupeReel::lancer(&racine, "ordi-mixte", &script_tetu(), &[]);
    let magasin = poser_marqueur(&racine, &natif.nom, &natif.marqueur("inst", "cmd", 2));
    poser_marqueur(
        &racine,
        &ordinaire.nom,
        &ordinaire.marqueur("autre", "autre", 1),
    );
    let identites = std::collections::BTreeMap::from([natif.identite("inst", "cmd", 2)]);

    let fait = magasin
        .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_secs(10))
        .unwrap();

    assert_eq!(fait.len(), 2);
    assert_eq!(natif.statut().code(), Some(0), "natif : coopératif");
    assert_eq!(
        ordinaire.statut().signal(),
        Some(libc::SIGKILL),
        "ordinaire : escaladé"
    );
    let _ = fs::remove_dir_all(&racine);
}

#[test]
fn native149_o4_naissance_differente_ou_groupe_disparu_ne_recoit_aucun_signal() {
    let racine = racine_privee("naissance");
    let natif = GroupeReel::lancer(&racine, "natif-recycle", &script_rapide(), &[]);
    let ordinaire = GroupeReel::lancer(&racine, "ordi-recycle", &script_rapide(), &[]);
    let mut m_natif = natif.marqueur("inst", "cmd", 1);
    m_natif.birth += 1;
    let mut m_ordi = ordinaire.marqueur("i", "c", 1);
    m_ordi.birth += 1;
    let magasin = poser_marqueur(&racine, &natif.nom, &m_natif);
    poser_marqueur(&racine, &ordinaire.nom, &m_ordi);
    // Groupe déjà disparu : son marqueur est seulement retiré.
    let mort = GroupeReel::lancer(&racine, "natif-mort", &format!(r#"{PRET}"#), &[]);
    let m_mort = mort.marqueur("inst", "mort", 1);
    unsafe { libc::killpg(mort.pgid as libc::pid_t, libc::SIGTERM) };
    mort.statut();
    poser_marqueur(&racine, &mort.nom, &m_mort);
    let identites = std::collections::BTreeMap::from([
        natif.identite("inst", "cmd", 1),
        mort.identite("inst", "mort", 1),
    ]);

    let fait = magasin
        .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_secs(5))
        .unwrap();

    assert_eq!(fait.len(), 3);
    for groupe in [&natif, &ordinaire] {
        assert!(
            !groupe.a_recu_term(),
            "{} : PID recyclé, jamais de signal",
            groupe.nom
        );
        assert!(groupe.vivant());
        assert!(
            !marqueur_present(&racine, &groupe.nom),
            "marqueur obsolète retiré"
        );
    }
    let _ = fs::remove_dir_all(&racine);
}

#[test]
fn native149_o4_identite_native_incoherente_refuse_avant_tout_signal_meme_pour_un_autre_groupe() {
    for (instance, commande, generation) in [
        ("autre-instance", "cmd", 1u64),
        ("inst", "autre-commande", 1),
        ("inst", "cmd", 2),
    ] {
        let racine = racine_privee("identite");
        // `a-sain` précède `z-faux` dans l'ordre lexical : il ne doit pas être
        // signalé avant que le lot entier soit classé.
        let sain = GroupeReel::lancer(&racine, "a-sain", &script_rapide(), &[]);
        let faux = GroupeReel::lancer(&racine, "z-faux", &script_rapide(), &[]);
        let magasin = poser_marqueur(&racine, &sain.nom, &sain.marqueur("inst-s", "cmd-s", 1));
        poser_marqueur(&racine, &faux.nom, &faux.marqueur("inst", "cmd", 1));
        let identites = std::collections::BTreeMap::from([
            sain.identite("inst-s", "cmd-s", 1),
            faux.identite(instance, commande, generation),
        ]);

        let erreur = magasin
            .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_secs(5))
            .unwrap_err();

        assert!(
            erreur
                .to_string()
                .contains("identité native du marqueur incohérente"),
            "{instance}/{commande}/{generation} : {erreur}"
        );
        for groupe in [&sain, &faux] {
            assert!(
                !groupe.a_recu_term(),
                "{} : aucun signal avant refus",
                groupe.nom
            );
            assert!(groupe.vivant());
            assert!(
                marqueur_present(&racine, &groupe.nom),
                "marqueurs conservés"
            );
        }
        drop((sain, faux));
        let _ = fs::remove_dir_all(&racine);
    }
}

#[test]
fn native149_o4_les_identites_de_demarrage_viennent_de_la_saga_et_pilotent_l_arret_cooperatif() {
    let (mut st, config, rx, _reader) = fixture_with_reader("o4-glue");
    let natif = mission_native(&mut st, &rx, "o4-glue", "working");
    let _en_file = mission_native(&mut st, &rx, "o4-glue-file", "queued");
    let racine = racine_privee("glue");
    let groupe = GroupeReel::lancer(&racine, &natif.child, &script_lent(), &[]);
    let entree = st
        .fleet
        .desired_fleet()
        .unwrap()
        .equipiers
        .get(&natif.child)
        .cloned()
        .unwrap();
    let instance = natif.child_instance.clone().unwrap();
    let magasin = poser_marqueur(
        &racine,
        &natif.child,
        &groupe.marqueur(&instance, &natif.task_id, entree.generation),
    );
    st.marker_store = magasin.clone();

    let identites = restart_identities(&st).unwrap();

    assert_eq!(
        identites.len(),
        1,
        "seule la tâche avec marqueur est une autorité"
    );
    let identite = &identites[&natif.child];
    assert_eq!(identite.instance_id, instance);
    assert_eq!(identite.command_id, natif.task_id);
    assert_eq!(identite.generation, entree.generation);
    let fait = magasin
        .reconcile_stale_groups_with_native(COURT, SONDE, &identites, Duration::from_secs(10))
        .unwrap();
    assert_eq!(fait, vec![natif.child.clone()]);
    assert_eq!(groupe.statut().code(), Some(0), "arrêt coopératif natif");
    let _ = fs::remove_dir_all(&racine);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native149_o4_restart_identities_refuse_toute_incoherence_saga_flotte_marqueur() {
    let (mut st, config, rx, _reader) = fixture_with_reader("o4-ident");
    let natif = mission_native(&mut st, &rx, "o4-ident", "working");
    let racine = racine_privee("ident");
    let magasin = poser_marqueur(
        &racine,
        &natif.child,
        &ManagedMarker {
            pgid: 1,
            birth: 1,
            instance_id: natif.child_instance.clone().unwrap(),
            command_id: natif.task_id.clone(),
            generation: 1,
        },
    );
    st.marker_store = magasin;
    let sain = st.delegation_store.get(&natif.task_id).unwrap().unwrap();
    assert_eq!(restart_identities(&st).unwrap().len(), 1, "témoin positif");

    let cas: Vec<(&str, Box<dyn Fn(&mut crate::delegation::Task)>)> = vec![
        (
            "répertoire de travail",
            Box::new(|t| t.cwd = "/private/tmp/ailleurs".into()),
        ),
        (
            "définition figée",
            Box::new(|t| t.definition.args.push("--autre".into())),
        ),
        (
            "type d'agent",
            Box::new(|t| {
                if let NativeDelegationRequest::Delegate { agent_type, .. } = &mut t.request {
                    *agent_type = "autre-type".into();
                }
            }),
        ),
        (
            "instance enfant absente",
            Box::new(|t| t.child_instance = None),
        ),
        (
            "instance enfant inconnue de la flotte",
            Box::new(|t| t.child_instance = Some("instance-inconnue".into())),
        ),
        (
            "parent du lien",
            Box::new(|t| {
                t.owner_instance = "instance-etrangere".into();
                t.origin_owner_instance = "instance-etrangere".into();
            }),
        ),
    ];
    for (nom, mutation) in cas {
        let mut modifiee = sain.clone();
        mutation(&mut modifiee);
        st.delegation_store.save(&modifiee).unwrap();
        // FR029 : l'échec d'identité reste local, sans bloquer le daemon.
        let identites = restart_identities(&st).unwrap();
        assert!(
            identites.is_empty(),
            "{nom} : aucune identité étrangère annoncée"
        );
        let relue = st.delegation_store.get(&natif.task_id).unwrap().unwrap();
        assert_eq!(
            (relue.state.as_str(), relue.error.as_deref()),
            ("failed", Some("native_restart_identity_mismatch")),
            "{nom}"
        );
        assert_eq!(relue.child_instance, modifiee.child_instance, "{nom}");
        assert_eq!(relue.restart_resumes, sain.restart_resumes, "{nom}");
        st.delegation_store.save(&sain).unwrap();
    }
    assert_eq!(
        restart_identities(&st).unwrap().len(),
        1,
        "retour au témoin"
    );
    let _ = fs::remove_dir_all(&racine);
    let _ = fs::remove_file(&config.db_path);
}

/// T007 : quatre alias hermétiques. Réemploi de la fixture 149 dans six tests ;
/// aucune politique ni résultat d'admission n'est calculé par ce helper.
fn native157_registre_coherence(st: &mut DaemonState) {
    let claude = st.registry.get("claude-child").unwrap();
    let claude = json!({
        "command": claude.command, "args": claude.args,
        "protocol": "claude_stream_json", "forbidden_env": claude.forbidden_env,
        "pass_env": claude.pass_env, "claude_config_dir": claude.claude_config_dir,
        "permissions": claude.permissions,
        "capabilities": {"execution_paths": ["claude_stream_json"],
            "models": {"custom/model": {"efforts": ["high"]}}}
    });
    let codex = st.registry.get("native-test").unwrap();
    let codex = json!({
        "command": codex.command, "args": codex.args,
        "protocol": "codex_app_server", "forbidden_env": codex.forbidden_env,
        "pass_env": codex.pass_env, "permissions": codex.permissions,
        "capabilities": {"execution_paths": ["codex_app_server"],
            "models": {"custom/model": {"efforts": ["high"]}}}
    });
    let registry = serde_json::to_string(&json!({"agents": {
        "claude": claude.clone(), "glm": claude,
        "codex": codex.clone(), "codex-pro": codex
    }}))
    .unwrap();
    let path = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("coherence157.json");
    bridget_transport::fsutil::write_private_file_atomic(&path, registry.as_bytes()).unwrap();
    st.registry = AgentRegistry::from_json(&registry, path).unwrap();
}

#[test]
fn native157_coherence_t3_claude_sans_launch_context_vers_codex() {
    let (mut st, config, rx, _reader) = fixture_with_reader("c157sans");
    native157_registre_coherence(&mut st);
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    let mut fact = claude_fact(&root.join("config-149"), &cwd, &st.source_env, "plan");
    fact.source = "provider_turn".into();
    fact.provider_policy["permission_callback"]["kind"] = json!("t3_runtime");
    fact.provider_policy
        .as_object_mut()
        .unwrap()
        .remove("launch_context");
    bridget_transport::protocol::validate_permissions(&fact, true).unwrap();
    insert_fact(&mut st, fact, None);
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "codex-pro")
        .unwrap();
    let admission = handle_locked(
        "conn-1",
        delegate_request(
            "codex-pro",
            "sans157",
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Discovery),
        ),
        &mut st,
    );
    assert_eq!(admission.unwrap_err(), "permission_source_unavailable");
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(
        entry["discovery"], false,
        "catalogue incohérent avec le refus réel : {entry}"
    );
    assert_eq!(entry["can_run"], false);
    assert!(
        entry["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|constraint| constraint["code"] == "permission_source_unavailable")
    );
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_coherence_launch_context_present_source_illisible() {
    let (mut st, config, rx, _reader) = fixture_with_reader("c157source");
    native157_registre_coherence(&mut st);
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    let mut fact = claude_fact(&root.join("config-149"), &cwd, &st.source_env, "plan");
    let settings = root.join("config-149/settings.json");
    // Source stable, révision exacte, JSON illisible : pas une mutation après le fait.
    bridget_transport::fsutil::write_private_file_atomic(&settings, b"{json-invalide157").unwrap();
    let revision = crate::native_permissions::source_revision(&settings).unwrap();
    let source = fact.provider_policy["launch_context"]["permission_sources"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|source| source["path"].as_str() == settings.to_str())
        .unwrap();
    source["revision"] = json!(revision.clone());
    bridget_transport::protocol::validate_permissions(&fact, false).unwrap();
    assert!(fact.provider_policy.get("launch_context").is_some());
    insert_fact(&mut st, fact, None);
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "codex-pro")
        .unwrap();
    let admission = handle_locked(
        "conn-1",
        delegate_request(
            "codex-pro",
            "source157",
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Discovery),
        ),
        &mut st,
    );
    assert_eq!(admission.unwrap_err(), "permission_source_unavailable");
    assert_eq!(
        crate::native_permissions::source_revision(&settings).unwrap(),
        revision
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(
        entry["discovery"], false,
        "catalogue incohérent avec la source réelle illisible : {entry}"
    );
    assert_eq!(entry["can_run"], false);
    assert!(
        entry["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|constraint| constraint["code"] == "permission_source_unavailable")
    );
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_coherence_matrice() {
    let mut contradictions = Vec::new();
    let mut contexts = Vec::new();
    let mut cells = 0;
    for parent in [
        "claude-sans",
        "claude-contexte",
        "claude-source-illisible",
        "codex",
        "aucun-fait",
    ] {
        for target in ["claude", "codex", "glm", "codex-pro"] {
            for (key, posture) in [
                ("inherit", None),
                ("discovery", Some(SpawnPosture::Discovery)),
                ("development", Some(SpawnPosture::Development)),
            ] {
                // État indépendant : aucune admission précédente ne consomme un slot.
                let (mut st, config, rx, _reader) = fixture_with_reader("c157mat");
                native157_registre_coherence(&mut st);
                let root = st.fixture_root.as_ref().unwrap().0.clone();
                let cwd = root.join("project-149").canonicalize().unwrap();
                st.delegation_store
                    .grant("instance-1", &cwd, SpawnPosture::Development, false)
                    .unwrap();
                if parent == "codex" {
                    insert_fact(
                        &mut st,
                        codex_fact(
                            cwd.to_str().unwrap(),
                            json!({"type":"dangerFullAccess"}),
                            "never",
                        ),
                        None,
                    );
                } else if parent != "aucun-fait" {
                    let mut fact =
                        claude_fact(&root.join("config-149"), &cwd, &st.source_env, "plan");
                    if parent == "claude-sans" {
                        fact.provider_policy
                            .as_object_mut()
                            .unwrap()
                            .remove("launch_context");
                    } else if parent == "claude-source-illisible" {
                        let settings = root.join("config-149/settings.json");
                        bridget_transport::fsutil::write_private_file_atomic(
                            &settings,
                            b"{json-invalide157",
                        )
                        .unwrap();
                        let revision =
                            crate::native_permissions::source_revision(&settings).unwrap();
                        let source = fact.provider_policy["launch_context"]["permission_sources"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|source| source["path"].as_str() == settings.to_str())
                            .unwrap();
                        source["revision"] = json!(revision);
                    }
                    bridget_transport::protocol::validate_permissions(&fact, false).unwrap();
                    insert_fact(&mut st, fact, None);
                }
                let catalogue =
                    handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
                let entry = catalogue["providers"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|entry| entry["agent_type"] == target)
                    .unwrap();
                let request_id = format!("mat157-{parent}-{target}-{key}");
                let result = handle_locked(
                    "conn-1",
                    delegate_request(target, &request_id, cwd.to_str().unwrap(), posture),
                    &mut st,
                );
                let allowed = entry[key].as_bool().unwrap();
                if allowed != result.is_ok() {
                    contradictions.push(format!(
                        "{parent}/{target}/{key}: catalogue={allowed}, admission={result:?}"
                    ));
                }
                contexts.push((
                    catalogue["verdict_context"].clone(),
                    json!({"cwd":cwd,"fact_scope":"session"}),
                ));
                assert_eq!(
                    st.delegation_store.tasks().unwrap().len(),
                    usize::from(result.is_ok())
                );
                assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
                cells += 1;
                drop(st);
                let _ = fs::remove_file(&config.db_path);
            }
        }
    }
    assert_eq!(cells, 60);
    assert!(contradictions.is_empty(), "{}", contradictions.join("\n"));
    for (actual, expected) in contexts {
        assert_eq!(actual, expected, "contexte de verdict explicite");
    }
}

#[test]
fn native157_coherence_hors_contexte_fait_limite_a_une_demande() {
    let (mut st, config, rx, _reader) = fixture_with_reader("c157scope");
    native157_registre_coherence(&mut st);
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        Some("scope157"),
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "codex-pro")
        .unwrap();
    assert_eq!(
        entry["inherit"], false,
        "le fait limité à une demande ne définit pas le catalogue"
    );
    let result = handle_locked(
        "conn-1",
        delegate_request("codex-pro", "scope157", cwd.to_str().unwrap(), None),
        &mut st,
    )
    .unwrap();
    let task = st
        .delegation_store
        .get(result["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert!(
        task.permission_snapshot.is_some(),
        "le fait est utilisé pour sa seule demande"
    );
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(catalogue["verdict_context"]["fact_scope"], "session");
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_coherence_hors_contexte_sous_repertoire() {
    let (mut st, config, rx, _reader) = fixture_with_reader("c157cwd");
    native157_registre_coherence(&mut st);
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    let child_cwd = cwd.join("sous-repertoire157");
    fs::create_dir(&child_cwd).unwrap();
    let fact = claude_fact(&root.join("config-149"), &cwd, &st.source_env, "plan");
    insert_fact(&mut st, fact, None);
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "claude")
        .unwrap();
    assert_eq!(entry["inherit"], true);
    let admission = handle_locked(
        "conn-1",
        delegate_request("claude", "cwd157", child_cwd.to_str().unwrap(), None),
        &mut st,
    );
    // Recapture des permission_sources du sous-répertoire avant la garde cwd.
    assert_eq!(admission.unwrap_err(), "settings_revision_changed");
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(
        catalogue["verdict_context"],
        json!({"cwd":cwd,"fact_scope":"session"})
    );
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_coherence_hors_contexte_binaire_retire_entre_deux_appels() {
    let (mut st, config, rx, _reader) = fixture_with_reader("c157bin");
    native157_registre_coherence(&mut st);
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "claude")
        .unwrap();
    assert_eq!(entry["inherit"], true);
    fs::remove_file(root.join("bin/claude")).unwrap();
    let admission = handle_locked(
        "conn-1",
        delegate_request("claude", "bin157", cwd.to_str().unwrap(), None),
        &mut st,
    );
    assert_eq!(admission.unwrap_err(), "command_missing");
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(
        entry["can_run"], true,
        "catalogue établi avant le retrait du binaire"
    );
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_preflight_admission_binaire_absent_sans_tache() {
    let (mut st, config, rx, _reader) = fixture_with_reader("p157admit");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    st.source_env
        .insert("PATH".into(), root.join("bin").into_os_string());
    fs::remove_file(root.join("bin/claude")).unwrap();
    let request = delegate_request("claude-child", "missing157", cwd.to_str().unwrap(), None);
    assert_eq!(
        handle_locked("conn-1", request.clone(), &mut st).unwrap_err(),
        "command_missing"
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    // Le refus ne réserve pas la clé : le même appel est admis après réparation humaine.
    fs::copy("/bin/echo", root.join("bin/claude")).unwrap();
    fs::set_permissions(root.join("bin/claude"), fs::Permissions::from_mode(0o755)).unwrap();
    let accepted = handle_locked("conn-1", request, &mut st).unwrap();
    assert_eq!(accepted["status"], "queued");
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_preflight_catalogue_binaire_absent() {
    let (mut st, config, rx, _reader) = fixture_with_reader("p157cat");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").canonicalize().unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    st.source_env
        .insert("PATH".into(), root.join("bin").into_os_string());
    fs::remove_file(root.join("bin/claude")).unwrap();
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "claude-child")
        .unwrap();
    assert_eq!(entry["can_run"], false);
    assert!(
        entry["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "command_missing")
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_quota_catalogue_epuise() {
    let (mut st, config, rx, _reader) = fixture_with_reader("q157cat");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let now = unix_timestamp();
    let resets_at = now + 3600;
    st.provider_limits.record(
        "native-test",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rate_limit_reached".into(),
            resets_at: Some(resets_at),
            used_percent: Some(100),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        now,
    );
    assert!(
        st.provider_limits
            .quota("native-test", now)
            .unwrap()
            .exhausted
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "native-test")
        .unwrap();
    assert_eq!(entry["quota"]["exhausted"], true);
    assert_eq!(entry["quota"]["resets_at"], resets_at);
    assert_eq!(entry["can_run"], false);
    assert!(
        entry["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|constraint| constraint["code"] == "provider_quota_exhausted")
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_quota_admission_refuse_avec_resets_at() {
    let (mut st, config, rx, _reader) = fixture_with_reader("q157adm");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let now = unix_timestamp();
    let resets_at = now + 3600;
    st.provider_limits.record(
        "native-test",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rate_limit_reached".into(),
            resets_at: Some(resets_at),
            used_percent: Some(100),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        now,
    );
    let request = delegate_request(
        "native-test",
        "quota157-cle-libre",
        cwd.to_str().unwrap(),
        None,
    );
    let code = handle_locked("conn-1", request.clone(), &mut st)
        .expect_err("quota épuisé : aucun tick ni lancement au RED");
    // Le même rendu natif que le wrapper public, sans son tick en cas d'admission.
    let result = failure(&code);
    assert_eq!(result["status"], "refused");
    assert_eq!(result["code"], "provider_quota_exhausted");
    assert_eq!(result["detail"], json!({"resets_at":resets_at}));
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    let refusals: i64 = st
        .store
        .connection()
        .query_row(
            "SELECT count(*) FROM native_delegation_refusals WHERE request_id=?1",
            ["quota157-cle-libre"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(refusals, 0, "le quota ne consomme pas la clé idempotente");
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    st.provider_limits.record(
        "native-test",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "available".into(),
            resets_at: Some(resets_at),
            used_percent: Some(0),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        now,
    );
    let accepted = handle_locked("conn-1", request, &mut st).unwrap();
    assert!(accepted["task_id"].as_str().is_some());
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_quota_perime_ignore() {
    let (mut st, config, rx, _reader) = fixture_with_reader("q157old");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    let now = unix_timestamp();
    // Le fait était futur à l'écriture, puis son retour est passé à la lecture.
    st.provider_limits.record(
        "native-test",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rate_limit_reached".into(),
            resets_at: Some(now - 1),
            used_percent: Some(100),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        now - 100,
    );
    assert!(st.provider_limits.quota("native-test", now).is_none());
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "native-test")
        .unwrap();
    let accepted = handle_locked(
        "conn-1",
        delegate_request(
            "native-test",
            "quota157-perime",
            cwd.to_str().unwrap(),
            None,
        ),
        &mut st,
    )
    .unwrap();
    assert!(accepted["task_id"].as_str().is_some());
    assert!(
        entry.as_object().unwrap().contains_key("quota"),
        "quota périmé publié comme null explicite"
    );
    assert_eq!(entry["quota"], Value::Null);
    assert_eq!(entry["can_run"], true);
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_quota_inconnu_null() {
    let (mut st, config, rx, _reader) = fixture_with_reader("q157null");
    let cwd = st
        .fixture_root
        .as_ref()
        .unwrap()
        .0
        .join("project-149")
        .canonicalize()
        .unwrap();
    insert_fact(
        &mut st,
        codex_fact(
            cwd.to_str().unwrap(),
            json!({"type":"dangerFullAccess"}),
            "never",
        ),
        None,
    );
    assert!(
        st.provider_limits
            .quota("native-test", unix_timestamp())
            .is_none()
    );
    let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
    let entry = catalogue["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["agent_type"] == "native-test")
        .unwrap();
    assert!(
        entry.as_object().unwrap().contains_key("quota"),
        "absence de fait représentée par quota:null"
    );
    assert_eq!(entry["quota"], Value::Null);
    assert_eq!(entry["can_run"], true);
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_quota_survit_fin_presence() {
    let (mut st, config, rx, _reader) = fixture_with_reader("q157fin");
    st.presences.get_mut("instance-1").unwrap().agent_type = "native-test".into();
    let now = unix_timestamp();
    let ack = super::super::handle_rate_limit(
        "89000000-0000-4000-8000-000000000102",
        "five_hour".into(),
        "rate_limit_reached".into(),
        Some(now + 3600),
        Some(100),
        None,
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        &mut st,
    );
    assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
    let before = serde_json::to_value(
        st.provider_limits
            .quota("native-test", now)
            .expect("le vrai handler enregistre la limite"),
    )
    .unwrap();
    st.router.unregister_by_conn("conn-1");
    assert!(st.presences.remove("instance-1").is_some());
    assert!(st.presences.is_empty());
    let after =
        serde_json::to_value(st.provider_limits.quota("native-test", now).unwrap()).unwrap();
    assert_eq!(after, before, "mémoire quota indépendante de la présence");
    assert_eq!(after["exhausted"], true);
    assert_eq!(after["resets_at"], now + 3600);
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(st);
    let _ = fs::remove_file(&config.db_path);
}

#[test]
fn native157_quota_nom_enregistre_par_les_trois_sources() {
    use bridget_transport::protocol::RateLimitSource;
    for (source, agent_type, status) in [
        (
            RateLimitSource::ClaudeStreamJson,
            "claude-child",
            "rejected",
        ),
        (
            RateLimitSource::CodexAppServer,
            "native-test",
            "rate_limit_reached",
        ),
        (RateLimitSource::ClaudeStatusLine, "claude-child", "unknown"),
    ] {
        let (mut st, config, rx, _reader) = fixture_with_reader("q157sources");
        let presence = st.presences.get_mut("instance-1").unwrap();
        presence.agent_type = agent_type.into();
        presence.name = "affichage-quota157".into();
        let now = unix_timestamp();
        let ack = super::super::handle_rate_limit(
            "89000000-0000-4000-8000-000000000102",
            "five_hour".into(),
            status.into(),
            Some(now + 3600),
            Some(100),
            None,
            source,
            &mut st,
        );
        assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
        let quota = st
            .provider_limits
            .quota(agent_type, unix_timestamp())
            .expect("nom du registre enregistré par le vrai handler");
        assert!(quota.exhausted, "{source}");
        assert_eq!(quota.resets_at, Some(now + 3600));
        assert_eq!(quota.windows.len(), 1);
        assert!(quota.observed_at >= now);
        assert!(quota.observed_at <= unix_timestamp());
        assert!(
            st.provider_limits
                .quota("affichage-quota157", now)
                .is_none()
        );
        assert!(
            st.provider_limits
                .quota("89000000-0000-4000-8000-000000000102", now)
                .is_none()
        );
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        drop(st);
        let _ = fs::remove_file(&config.db_path);
    }
}

// Session157 T017/T019/T021 : admission réelle, fixture149, aucun tick/spawn.
// Ces trois raccords de fixture ont chacun plusieurs consommateurs ci-dessous.
fn fallback157_definition(command: &str, protocol: &str) -> Value {
    json!({"command":command,"args":[],"protocol":protocol,"forbidden_env":[],
        "capabilities":{"execution_paths":[protocol],"models":{"custom/model":{"efforts":["high"]}}}})
}

fn fallback157_registry(st: &mut DaemonState, target: Value, first: Value) {
    // Le préflight157 exige un profil privé ; ne modifier que cette fixture.
    fs::set_permissions(
        st.fixture_root.as_ref().unwrap().0.join("config-149"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    st.registry = AgentRegistry::from_json(
        &json!({"agents":{"fallback157-target":target,"fallback157-first":first,
            "fallback157-second":fallback157_definition("/bin/sh", "codex_app_server")}})
        .to_string(),
        Path::new("/tmp/fallback157-registry.json"),
    )
    .unwrap();
}

fn fallback157_request(cwd: &str, key: &str) -> NativeDelegationRequest {
    let mut request = delegate_request("fallback157-target", key, cwd, None);
    if let NativeDelegationRequest::Delegate { fallback, .. } = &mut request {
        *fallback = vec!["fallback157-first", "fallback157-second"]
            .into_iter()
            .map(|name| bridget_transport::protocol::FallbackTarget {
                agent_type: name.into(),
                model: "custom/model".into(),
                effort: Some("high".into()),
            })
            .collect();
    }
    request
}

#[test]
fn native157_fallback_premier_puis_second_candidat_valide() {
    for first_missing in [false, true] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("ord157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        insert_fact(
            &mut st,
            codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
            None,
        );
        let mut target = fallback157_definition(
            root.join("absent-target").to_str().unwrap(),
            "codex_app_server",
        );
        target["capabilities"]["models"]["requested/model"] = json!({"efforts":["high"]});
        fallback157_registry(
            &mut st,
            target,
            fallback157_definition(
                if first_missing {
                    "/missing/fallback157-first"
                } else {
                    "/bin/sh"
                },
                "codex_app_server",
            ),
        );
        let mut request = fallback157_request(&cwd, "ordre157");
        if let NativeDelegationRequest::Delegate {
            model, fallback, ..
        } = &mut request
        {
            *model = "requested/model".into();
            for candidate in fallback {
                candidate.effort = None;
            }
        }
        let response = handle_locked("conn-1", request.clone(), &mut st).unwrap();
        let selected = if first_missing {
            "fallback157-second"
        } else {
            "fallback157-first"
        };
        assert_eq!(response["agent_type"], selected);
        assert_eq!(response["model"], "custom/model");
        assert!(response["effort"].is_null());
        assert_eq!(
            response["substitution"]["requested"]["model"],
            "requested/model"
        );
        assert_eq!(response["substitution"]["requested"]["effort"], "high");
        assert_eq!(
            response["substitution"]["selected"]["model"],
            "custom/model"
        );
        assert!(response["substitution"]["selected"]["effort"].is_null());
        assert_eq!(
            response["substitution"]["requested"]["agent_type"],
            "fallback157-target"
        );
        assert_eq!(response["substitution"]["selected"]["agent_type"], selected);
        assert_eq!(response["substitution"]["reason"], "command_missing");
        if first_missing {
            assert!(
                response["substitution"]["tried"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|attempt| attempt["agent_type"] == "fallback157-first"
                        && attempt["code"] == "command_missing")
            );
        }
        let saved = st
            .delegation_store
            .get(response["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(saved.substitution.as_ref().unwrap()).unwrap(),
            response["substitution"]
        );
        assert_eq!(
            st.delegation_store
                .by_agent_request(
                    &saved.owner,
                    "ordre157",
                    &serde_json::to_vec(&request).unwrap()
                )
                .unwrap()
                .unwrap()
                .task_id,
            saved.task_id,
            "canonical conserve la requête originale et sa liste"
        );
        if let NativeDelegationRequest::Delegate {
            agent_type,
            fallback,
            ..
        } = &saved.request
        {
            assert_eq!(agent_type, selected);
            assert!(
                fallback.is_empty(),
                "la requête effective ne porte plus la liste originale"
            );
        } else {
            panic!("requête de mission attendue");
        }
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
    }
    // Même cas de stabilité T017/T018 : aucune API/hook de test ajouté.
    // Les vraies passes154 sont séparées uniquement pour invalider leur état.
    for invalidated_pass in ["reserve", "commit"] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("wt157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let project = root.join("project-149");
        fs::write(project.join(".gitignore"), b".worktrees/\n").unwrap();
        fs::write(project.join("mission.txt"), b"fixture157\n").unwrap();
        for args in [
            vec!["init", "-b", "main"],
            vec!["add", "--", ".gitignore", "mission.txt"],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-m",
                "fixture157",
            ],
        ] {
            let output = std::process::Command::new("git")
                .current_dir(&project)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let cwd = project.to_string_lossy().into_owned();
        insert_fact(
            &mut st,
            codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
            None,
        );
        fallback157_registry(
            &mut st,
            fallback157_definition("/missing/fallback157-target", "codex_app_server"),
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        let mut request = fallback157_request(&cwd, "worktree-stable157");
        if let NativeDelegationRequest::Delegate {
            posture, workspace, ..
        } = &mut request
        {
            *posture = Some(SpawnPosture::Development);
            *workspace = Some(WorkspaceRequest::Worktree {
                base_ref: None,
                branch: None,
            });
        }
        let (probed_cwd, selected, raw, substitution) = match admit_delegate(
            "conn-1",
            &request,
            None,
            None,
            &mut st,
            WorkspacePass::Probe,
            None,
        )
        .unwrap()
        {
            WorkspaceAdmission::Probe(cwd, selected, raw, substitution) => {
                (cwd, selected, raw, substitution)
            }
            _ => panic!("sonde worktree attendue"),
        };
        if let NativeDelegationRequest::Delegate {
            agent_type,
            fallback,
            ..
        } = &selected
        {
            assert_eq!(agent_type, "fallback157-first");
            assert!(fallback.is_empty());
        } else {
            panic!("requête effective attendue");
        }
        let plan = crate::workspace_git::preflight(&probed_cwd, None, None).unwrap();
        let id = Uuid::new_v4().to_string();
        if invalidated_pass == "reserve" {
            st.provider_limits.record(
                "fallback157-first",
                bridget_transport::protocol::RateLimitFact {
                    credits: None,
                    window: "primary/300m".into(),
                    status: "rejected".into(),
                    resets_at: Some(unix_timestamp() + 3600),
                    used_percent: Some(100),
                },
                bridget_transport::protocol::RateLimitSource::CodexAppServer,
                unix_timestamp(),
            );
            let code = match admit_delegate(
                "conn-1",
                &request,
                None,
                None,
                &mut st,
                WorkspacePass::Reserve {
                    plan: &plan,
                    task_id: &id,
                    selection: (&selected, &raw, substitution.as_ref()),
                },
                None,
            ) {
                Err(code) => code,
                Ok(_) => panic!("quota atteint après Probe : refus, jamais le second candidat"),
            };
            assert_eq!(failure(&code)["code"], "provider_quota_exhausted");
        } else {
            let (reserved, claim) = match admit_delegate(
                "conn-1",
                &request,
                None,
                None,
                &mut st,
                WorkspacePass::Reserve {
                    plan: &plan,
                    task_id: &id,
                    selection: (&selected, &raw, substitution.as_ref()),
                },
                None,
            )
            .unwrap()
            {
                WorkspaceAdmission::Reserved { task, claim } => (task, claim),
                _ => panic!("réservation worktree attendue"),
            };
            let record = crate::workspace_git::create(&plan, &reserved.task_id).unwrap();
            st.provider_limits.record(
                "fallback157-first",
                bridget_transport::protocol::RateLimitFact {
                    credits: None,
                    window: "primary/300m".into(),
                    status: "rejected".into(),
                    resets_at: Some(unix_timestamp() + 3600),
                    used_percent: Some(100),
                },
                bridget_transport::protocol::RateLimitSource::CodexAppServer,
                unix_timestamp(),
            );
            let code = match admit_delegate(
                "conn-1",
                &request,
                None,
                None,
                &mut st,
                WorkspacePass::Commit {
                    plan: &plan,
                    task_id: &id,
                    record: &record,
                    selection: (&selected, &raw, substitution.as_ref()),
                },
                None,
            ) {
                Err(code) => code,
                Ok(_) => panic!("quota atteint après Reserve : refus, jamais le second candidat"),
            };
            assert_eq!(failure(&code)["code"], "provider_quota_exhausted");
            assert!(
                Path::new(&record.path).is_dir(),
                "le vrai dossier enfant154 a été utilisé"
            );
            drop(claim);
        }
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        assert!(st.workspace_work.lock().unwrap().creations.is_empty());
    }
}

#[test]
fn native157_fallback_motifs_remplacables_fr013() {
    for reason in [
        "command_missing",
        "billing_guard",
        "env_unfit",
        "unsupported_capability",
        "negotiation_failed",
        "permission_source_unavailable",
        "permission_mapping_unavailable",
        "provider_confinement_unavailable",
        "provider_quota_exhausted",
    ] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("mot157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        let mut target = fallback157_definition("/bin/sh", "codex_app_server");
        let mut parent = codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never");
        match reason {
            "command_missing" => target["command"] = json!(root.join("absent-target")),
            "billing_guard" => {
                target["forbidden_env"] = json!(["FALLBACK157_BILLING"]);
                st.source_env
                    .insert("FALLBACK157_BILLING".into(), "fixture-only".into());
            }
            "unsupported_capability" => target["capabilities"]["execution_paths"] = json!(["acp"]),
            "negotiation_failed" => target = fallback157_definition("/bin/sh", "tmux"),
            "env_unfit"
            | "permission_source_unavailable"
            | "permission_mapping_unavailable"
            | "provider_confinement_unavailable" => {
                target = fallback157_definition("claude", "claude_stream_json");
                let profile = root.join("config-149");
                target["claude_config_dir"] = json!(if reason == "env_unfit" {
                    root.join("absent-profile")
                } else {
                    profile.clone()
                });
                if reason == "permission_source_unavailable" {
                    fs::write(profile.join("settings.json"), b"{").unwrap();
                }
                if reason == "permission_mapping_unavailable" {
                    parent.provider_policy["approval_policy"] = json!("on-request");
                }
                if reason == "provider_confinement_unavailable" {
                    parent.provider_policy["sandbox_policy"] = json!({"type":"workspaceWrite",
                        "writableRoots":[cwd],"networkAccess":false,
                        "excludeTmpdirEnvVar":false,"excludeSlashTmp":false});
                }
            }
            "provider_quota_exhausted" => st.provider_limits.record(
                "fallback157-target",
                bridget_transport::protocol::RateLimitFact {
                    credits: None,
                    window: "primary/300m".into(),
                    status: "rejected".into(),
                    resets_at: Some(unix_timestamp() + 3600),
                    used_percent: Some(100),
                },
                bridget_transport::protocol::RateLimitSource::CodexAppServer,
                unix_timestamp(),
            ),
            _ => unreachable!(),
        }
        insert_fact(&mut st, parent, None);
        fallback157_registry(
            &mut st,
            target,
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        let response =
            handle_locked("conn-1", fallback157_request(&cwd, "motif157"), &mut st).unwrap();
        assert_eq!(
            response["agent_type"], "fallback157-first",
            "{reason}: {response}"
        );
        assert_eq!(response["substitution"]["reason"], reason);
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
    }
}

#[test]
fn native157_fallback_modele_effort_type_avant_indisponibilite() {
    for (scenario, expected) in [
        ("model_binary", "model_unavailable"),
        ("model_policy", "model_unavailable"),
        ("effort_quota", "effort_unavailable"),
        ("unknown_type", "agent_type_unknown"),
    ] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("val157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        let mut parent = codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never");
        let mut target = fallback157_definition("/missing/fallback157-target", "codex_app_server");
        if scenario == "model_policy" {
            target = fallback157_definition("claude", "claude_stream_json");
            target["claude_config_dir"] = json!(root.join("config-149"));
            parent.provider_policy["approval_policy"] = json!("on-request");
        }
        insert_fact(&mut st, parent, None);
        fallback157_registry(
            &mut st,
            target,
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        let mut request = fallback157_request(&cwd, "validation157");
        if let NativeDelegationRequest::Delegate {
            agent_type,
            model,
            effort,
            ..
        } = &mut request
        {
            match scenario {
                "model_binary" | "model_policy" => *model = "absent/model".into(),
                "effort_quota" => {
                    *effort = Some("absent-effort".into());
                }
                "unknown_type" => *agent_type = "absent-fallback157-type".into(),
                _ => unreachable!(),
            }
        }
        if scenario == "effort_quota" {
            st.provider_limits.record(
                "fallback157-target",
                bridget_transport::protocol::RateLimitFact {
                    credits: None,
                    window: "primary/300m".into(),
                    status: "rejected".into(),
                    resets_at: Some(unix_timestamp() + 3600),
                    used_percent: Some(100),
                },
                bridget_transport::protocol::RateLimitSource::CodexAppServer,
                unix_timestamp(),
            );
        }
        let response =
            handle_locked("conn-1", request, &mut st).unwrap_or_else(|code| failure(&code));
        assert_eq!(response["code"], expected, "{scenario}: {response}");
        assert!(response.get("substitution").is_none());
        assert!(st.delegation_store.tasks().unwrap().is_empty());
    }
}

#[test]
fn native157_fallback_ordre_premier_refus_contractuel() {
    for expected in [
        "identity_unavailable",
        "invalid_request",
        "reply_timeout_out_of_range",
        "delegation_grant_required",
    ] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("err157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        insert_fact(
            &mut st,
            codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
            None,
        );
        fallback157_registry(
            &mut st,
            fallback157_definition("/missing/fallback157-target", "codex_app_server"),
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        st.delegation_store
            .grant(
                "instance-1",
                Path::new(&cwd),
                SpawnPosture::Development,
                true,
            )
            .unwrap();
        let mut request = fallback157_request(&cwd, "precedence157");
        if let NativeDelegationRequest::Delegate {
            model,
            fallback,
            reply_timeout,
            ..
        } = &mut request
        {
            *model = "absent/model".into();
            if expected != "delegation_grant_required" {
                *reply_timeout = Some(0);
            }
            if matches!(expected, "identity_unavailable" | "invalid_request") {
                fallback[1] = fallback[0].clone();
            }
        }
        if expected == "identity_unavailable" {
            st.conn_instances.remove("conn-1");
        }
        let response =
            handle_locked("conn-1", request, &mut st).unwrap_or_else(|code| failure(&code));
        assert_eq!(
            response["code"], expected,
            "le premier refus du contrat gagne : {response}"
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
    }
}

#[test]
fn native157_fallback_droits_parent_jamais_remplaces() {
    for expected in [
        "permission_not_inherited",
        "delegation_grant_required",
        "permission_attestation_unavailable",
        "child_requests_reviewer_unsupported",
        "workspace_requires_development",
        "workspace_inheritance_unavailable",
        "child_requests_unsupported",
    ] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("par157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        let mut parent = codex_fact(
            &cwd,
            json!({"type":"readOnly","networkAccess":false}),
            "never",
        );
        let mut target = fallback157_definition("/bin/sh", "codex_app_server");
        let mut request = fallback157_request(&cwd, "parent157");
        if expected == "permission_not_inherited" {
            if let NativeDelegationRequest::Delegate { posture, .. } = &mut request {
                *posture = Some(SpawnPosture::Development);
            }
        } else if expected == "delegation_grant_required" {
            st.delegation_store
                .grant(
                    "instance-1",
                    Path::new(&cwd),
                    SpawnPosture::Development,
                    true,
                )
                .unwrap();
        } else if expected == "permission_attestation_unavailable" {
            // Aucun fait attesté : un candidat de repli ne fabrique pas la preuve du parent.
        } else if expected == "child_requests_reviewer_unsupported" {
            parent.provider_policy["approvals_reviewer"] = json!("guardian_subagent");
            if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
                *child_requests = Some("forward".into());
            }
        } else if expected == "workspace_requires_development" {
            if let NativeDelegationRequest::Delegate {
                posture, workspace, ..
            } = &mut request
            {
                *posture = Some(SpawnPosture::Discovery);
                *workspace = Some(WorkspaceRequest::Worktree {
                    base_ref: None,
                    branch: None,
                });
            }
        } else {
            let profile = root.join("config-149");
            fs::set_permissions(&profile, fs::Permissions::from_mode(0o700)).unwrap();
            target = fallback157_definition("claude", "claude_stream_json");
            target["claude_config_dir"] = json!(profile);
            if expected == "workspace_inheritance_unavailable" {
                parent = claude_fact(
                    &profile,
                    Path::new(&cwd),
                    &st.source_env,
                    "bypassPermissions",
                );
                if let NativeDelegationRequest::Delegate {
                    posture, workspace, ..
                } = &mut request
                {
                    *posture = Some(SpawnPosture::Development);
                    *workspace = Some(WorkspaceRequest::Worktree {
                        base_ref: None,
                        branch: None,
                    });
                }
            } else {
                parent = codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never");
                if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
                    *child_requests = Some("forward".into());
                }
            }
        }
        if expected != "permission_attestation_unavailable" {
            insert_fact(&mut st, parent, None);
        }
        fallback157_registry(
            &mut st,
            target,
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        let response =
            handle_locked("conn-1", request, &mut st).unwrap_or_else(|code| failure(&code));
        assert_eq!(response["code"], expected, "{response}");
        assert!(st.delegation_store.tasks().unwrap().is_empty());
    }
}

#[test]
fn native157_fallback_cible_valide_ignore_liste() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("ok157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
        None,
    );
    fallback157_registry(
        &mut st,
        fallback157_definition("/bin/sh", "codex_app_server"),
        fallback157_definition("/missing/fallback157-first", "codex_app_server"),
    );
    let response = handle_locked("conn-1", fallback157_request(&cwd, "valid157"), &mut st).unwrap();
    assert_eq!(response["agent_type"], "fallback157-target");
    assert!(response["substitution"].is_null());
    let task = st
        .delegation_store
        .get(response["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert!(task.substitution.is_none());
}

#[test]
fn native157_fallback_sans_liste_codes_historiques() {
    for expected in [
        "command_missing",
        "permission_not_inherited",
        "model_unavailable",
    ] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("old157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        insert_fact(
            &mut st,
            codex_fact(
                &cwd,
                json!({"type":"readOnly","networkAccess":false}),
                "never",
            ),
            None,
        );
        fallback157_registry(
            &mut st,
            fallback157_definition(
                if expected == "command_missing" {
                    "/missing/fallback157-target"
                } else {
                    "/bin/sh"
                },
                "codex_app_server",
            ),
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        let mut request = delegate_request("fallback157-target", "sans157", &cwd, None);
        if let NativeDelegationRequest::Delegate { posture, model, .. } = &mut request {
            if expected == "permission_not_inherited" {
                *posture = Some(SpawnPosture::Development);
            }
            if expected == "model_unavailable" {
                *model = "absent/model".into();
            }
        }
        let response =
            handle_locked("conn-1", request, &mut st).unwrap_or_else(|code| failure(&code));
        assert_eq!(response["code"], expected);
        assert!(st.delegation_store.tasks().unwrap().is_empty());
    }
}

#[test]
fn native157_fallback_validation_liste_fermee() {
    for malformed in ["duplicate", "target", "four", "empty"] {
        let (mut st, _config, _rx, _reader) = fixture_with_reader("lst157");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let cwd = root.join("project-149").to_string_lossy().into_owned();
        insert_fact(
            &mut st,
            codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
            None,
        );
        fallback157_registry(
            &mut st,
            fallback157_definition("/bin/sh", "codex_app_server"),
            fallback157_definition("/bin/sh", "codex_app_server"),
        );
        let mut request = fallback157_request(&cwd, "invalid157");
        if let NativeDelegationRequest::Delegate { fallback, .. } = &mut request {
            match malformed {
                "duplicate" => fallback[1] = fallback[0].clone(),
                "target" => fallback[0].agent_type = "fallback157-target".into(),
                "four" => {
                    fallback.push(bridget_transport::protocol::FallbackTarget {
                        agent_type: "third157".into(),
                        model: "custom/model".into(),
                        effort: Some("high".into()),
                    });
                    fallback.push(bridget_transport::protocol::FallbackTarget {
                        agent_type: "fourth157".into(),
                        model: "custom/model".into(),
                        effort: Some("high".into()),
                    });
                }
                "empty" => fallback[0].model.clear(),
                _ => unreachable!(),
            }
        }
        let response =
            handle_locked("conn-1", request, &mut st).unwrap_or_else(|code| failure(&code));
        assert_eq!(
            response["code"], "invalid_request",
            "{malformed}: {response}"
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
    }
}

#[test]
fn native157_fallback_aucune_elevation_des_droits() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("cap157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(
            &cwd,
            json!({"type":"readOnly","networkAccess":false}),
            "never",
        ),
        None,
    );
    let mut first = fallback157_definition("claude", "claude_stream_json");
    first["claude_config_dir"] = json!(root.join("config-149"));
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        first,
    );
    let response =
        handle_locked("conn-1", fallback157_request(&cwd, "rights157"), &mut st).unwrap();
    assert_eq!(response["agent_type"], "fallback157-first");
    let task = st
        .delegation_store
        .get(response["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    let snapshot = task.permission_snapshot.unwrap();
    assert_eq!(snapshot.child_policy["permission_mode"], "plan");
    assert_eq!(
        snapshot.child_policy["tools"],
        json!(["Read", "Glob", "Grep"])
    );
    for forbidden in [
        "Bash",
        "Edit",
        "Write",
        "NotebookEdit",
        "WebFetch",
        "WebSearch",
    ] {
        assert!(
            snapshot.child_policy["disallowed_tools"]
                .as_array()
                .unwrap()
                .contains(&json!(forbidden))
        );
    }
}

#[test]
fn native157_fallback_epuise_liste_les_essais() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("all157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
        None,
    );
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        fallback157_definition("/missing/fallback157-first", "codex_app_server"),
    );
    st.provider_limits.record(
        "fallback157-second",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "primary/300m".into(),
            status: "rejected".into(),
            resets_at: Some(unix_timestamp() + 3600),
            used_percent: Some(100),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        unix_timestamp(),
    );
    let response = handle_locked("conn-1", fallback157_request(&cwd, "exhausted157"), &mut st)
        .unwrap_or_else(|code| failure(&code));
    assert_eq!(response["code"], "fallback_exhausted");
    let attempts = response["detail"]["tried"].as_array().unwrap();
    // Tous les candidats explicitement listés apparaissent dans leur ordre.
    let candidates: Vec<&Value> = attempts
        .iter()
        .filter(|item| item["agent_type"] != "fallback157-target")
        .collect();
    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates[0],
        &json!({"agent_type":"fallback157-first","model":"custom/model","code":"command_missing"})
    );
    assert_eq!(
        candidates[1],
        &json!({"agent_type":"fallback157-second","model":"custom/model","code":"provider_quota_exhausted"})
    );
    assert!(st.delegation_store.tasks().unwrap().is_empty());
}

#[test]
fn native157_fallback_epuise_sans_tache_cle_libre() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("key157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
        None,
    );
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        fallback157_definition("/missing/fallback157-first", "codex_app_server"),
    );
    st.provider_limits.record(
        "fallback157-second",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "primary/300m".into(),
            status: "rejected".into(),
            resets_at: Some(unix_timestamp() + 3600),
            used_percent: Some(100),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        unix_timestamp(),
    );
    let request = fallback157_request(&cwd, "free-key157");
    let refused =
        handle_locked("conn-1", request.clone(), &mut st).unwrap_or_else(|code| failure(&code));
    assert_eq!(refused["code"], "fallback_exhausted");
    assert!(st.delegation_store.tasks().unwrap().is_empty());
    // La même clé et la même enveloppe deviennent admissibles après réparation du binaire.
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        fallback157_definition("/bin/sh", "codex_app_server"),
    );
    let accepted = handle_locked("conn-1", request, &mut st).unwrap();
    assert_eq!(accepted["agent_type"], "fallback157-first");
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
}

#[test]
fn native157_fallback_rejeu_meme_substitution() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("rep157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
        None,
    );
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        fallback157_definition("/bin/sh", "codex_app_server"),
    );
    let request = fallback157_request(&cwd, "stable-key157");
    let first = handle_locked("conn-1", request.clone(), &mut st).unwrap();
    let id = first["task_id"].as_str().unwrap().to_owned();
    st.provider_limits.record(
        "fallback157-first",
        bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "primary/300m".into(),
            status: "rejected".into(),
            resets_at: Some(unix_timestamp() + 3600),
            used_percent: Some(100),
        },
        bridget_transport::protocol::RateLimitSource::CodexAppServer,
        unix_timestamp(),
    );
    fallback157_registry(
        &mut st,
        fallback157_definition("/bin/sh", "codex_app_server"),
        fallback157_definition("/missing/fallback157-first", "codex_app_server"),
    );
    let replay = handle_locked("conn-1", request, &mut st).unwrap();
    assert_eq!(replay["task_id"], id);
    assert_eq!(replay["agent_type"], first["agent_type"]);
    assert_eq!(replay["substitution"], first["substitution"]);
    let status = handle_locked(
        "conn-1",
        NativeDelegationRequest::Status {
            task_id: id.clone(),
        },
        &mut st,
    )
    .unwrap();
    assert_eq!(status["substitution"], first["substitution"]);
    let stored = st.delegation_store.get(&id).unwrap().unwrap();
    assert_eq!(
        serde_json::to_value(stored.substitution.as_ref().unwrap()).unwrap(),
        first["substitution"]
    );
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
}

#[test]
fn native157_fallback_rejeu_autre_liste_envelope_mismatch() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("env157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
        None,
    );
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        fallback157_definition("/bin/sh", "codex_app_server"),
    );
    let request = fallback157_request(&cwd, "envelope157");
    let accepted = handle_locked("conn-1", request.clone(), &mut st).unwrap();
    let mut changed = request;
    if let NativeDelegationRequest::Delegate { fallback, .. } = &mut changed {
        fallback.reverse();
    }
    let refused = handle_locked("conn-1", changed, &mut st).unwrap_or_else(|code| failure(&code));
    assert_eq!(refused["code"], "envelope_mismatch");
    let stored = st
        .delegation_store
        .get(accepted["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(stored.substitution.as_ref().unwrap()).unwrap(),
        accepted["substitution"]
    );
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
}

#[test]
fn native157_fallback_projection_lineage_cible_effective() {
    let (mut st, _config, _rx, _reader) = fixture_with_reader("lin157");
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    let cwd = root.join("project-149").to_string_lossy().into_owned();
    insert_fact(
        &mut st,
        codex_fact(&cwd, json!({"type":"dangerFullAccess"}), "never"),
        None,
    );
    let mut first = fallback157_definition("/bin/sh", "codex_app_server");
    first["capabilities"]["models"] = json!({"selected/model":{"efforts":["high"]}});
    fallback157_registry(
        &mut st,
        fallback157_definition("/missing/fallback157-target", "codex_app_server"),
        first,
    );
    let mut request = fallback157_request(&cwd, "lineage157");
    if let NativeDelegationRequest::Delegate { fallback, .. } = &mut request {
        fallback[0].model = "selected/model".into();
    }
    let accepted = handle_locked("conn-1", request, &mut st).unwrap();
    assert_eq!(
        accepted["substitution"]["requested"]["model"],
        "custom/model"
    );
    let task = st
        .delegation_store
        .get(accepted["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    let page = st
        .delegation_store
        .lineage_page(task.root_owner(), 16, None, |_| false)
        .unwrap();
    assert_eq!(page["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(page["tasks"][0]["task_id"], task.task_id);
    assert_eq!(page["tasks"][0]["agent_type"], "fallback157-first");
    assert_eq!(page["tasks"][0]["model"], "selected/model");
    assert_eq!(page["tasks"][0]["effort"], "high");
}
