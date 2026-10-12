//! SPEC154 : oracles métier sur le daemon réel, Git de fixture uniquement.
use super::*;
use super::native_delegation_permissions149_tests::{
    claude_fact, codex_fact, delegate_request, fixture_with_reader, insert_fact,
};
use std::fs;
use std::process::Command;
use std::os::unix::fs::{symlink, PermissionsExt};

struct Fixture154 {
    state: Arc<Mutex<DaemonState>>,
    root: PathBuf,
    _config: DaemonConfig,
    _commands: Receiver<ManagedSupervisorCommand>,
    _reader: BufReader<UnixStream>,
}

fn git154(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C").arg(cwd).args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output().unwrap();
    assert!(output.status.success(), "Git fixture {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim_end().to_owned()
}

fn fixture154(label: &str) -> Fixture154 {
    let (mut st, config, commands, reader) = fixture_with_reader(label);
    let root = st.fixture_root.as_ref().unwrap().0.join("project-149");
    git154(&root, &["init", "--initial-branch=main"]);
    fs::write(root.join(".gitignore"), ".worktrees/\n").unwrap();
    fs::write(root.join("a.rs"), "base\n").unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/a.rs"), "sous-dossier\n").unwrap();
    git154(&root, &["add", "."]);
    git154(&root, &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-m", "Base fixture"]);
    insert_fact(&mut st, codex_fact(root.to_str().unwrap(), json!({"type":"dangerFullAccess"}), "never"), None);
    Fixture154 { state: Arc::new(Mutex::new(st)), root, _config:config, _commands:commands, _reader:reader }
}

fn request154(root: &Path, key: &str, workspace: Option<Value>) -> NativeDelegationRequest {
    let mut value = serde_json::to_value(delegate_request("native-test", key, root.to_str().unwrap(), Some(SpawnPosture::Development))).unwrap();
    if let Some(workspace) = workspace { value["workspace"] = workspace; }
    serde_json::from_value(value).unwrap()
}

fn call154(fixture: &Fixture154, request: NativeDelegationRequest) -> Value {
    call_state154(&fixture.state, "conn-1", request)
}

fn call_state154(state: &Arc<Mutex<DaemonState>>, conn: &str, request: NativeDelegationRequest) -> Value {
    match handle_with_fact(conn, request, None, None, state, None) {
        DaemonToWrapper::NativeDelegationResult { result } => result,
        other => panic!("réponse native attendue: {other:?}"),
    }
}

fn task154(fixture: &Fixture154, receipt: &Value) -> Task {
    let st = fixture.state.lock().unwrap();
    st.delegation_store.get(receipt["task_id"].as_str().unwrap()).unwrap().unwrap()
}

// Les tests de rapport/conservation sèment un vrai worktree indépendamment
// de l'admission. Leur oracle porte ainsi sur le rapport, pas sur la création.
fn seed_workspace154(fixture: &Fixture154) -> Task {
    let receipt = call154(fixture, request154(&fixture.root, &format!("seed-{}", Uuid::new_v4()), None));
    let mut task = task154(fixture, &receipt);
    let short = &task.task_id[..8];
    let branch = format!("bridget/{short}");
    let path = fixture.root.join(".worktrees").join(short);
    let base = git154(&fixture.root, &["rev-parse", "HEAD"]);
    git154(&fixture.root, &["worktree", "add", "-b", &branch, path.to_str().unwrap(), &base]);
    let mut value = serde_json::to_value(&task).unwrap();
    value["workspace"] = json!({"strategy":"worktree", "state":"active", "toplevel":fixture.root,
        "path":path, "branch":branch, "base_commit":base, "head_commit":null, "branch_commit":null,
        "head_on_branch":null, "commit_count":null, "commits":[], "truncated":false, "dirty":null});
    task = serde_json::from_value(value).unwrap();
    task.cwd = path.to_string_lossy().into_owned();
    fixture.state.lock().unwrap().delegation_store.save(&task).unwrap();
    task
}

#[test]
fn native154_creation_refus_et_rejeu_enveloppe() {
    let fixture = fixture154("refus");
    let request = request154(&fixture.root, "same-key", Some(json!({"strategy":"shared"})));
    let first = call154(&fixture, request.clone());
    assert_eq!(call154(&fixture, request)["task_id"], first["task_id"], "le rejeu rend la même tâche");
    let mismatch = call154(&fixture, request154(&fixture.root, "same-key", Some(json!({"strategy":"worktree"}))));
    assert_eq!(mismatch["code"], "envelope_mismatch");
    let denied = call154(&fixture, request154(&fixture.root, "bad-branch", Some(json!({"strategy":"worktree", "branch":"bad..branch"}))));
    assert_eq!(denied["code"], "workspace_branch_invalid", "un refus précède toute création");
    assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 1);
    assert_eq!(git154(&fixture.root, &["branch", "--list", "bridget/*"]), "");
    assert_eq!(fixture.state.lock().unwrap().delegation_store.admission_counts("instance-1").unwrap().2, 1);
}

#[test]
fn native154_isolation_creation_cwd_et_parent_intact() {
    let fixture = fixture154("isole");
    let before = (git154(&fixture.root, &["rev-parse", "HEAD"]), git154(&fixture.root, &["ls-files", "--stage"]), git154(&fixture.root, &["status", "--porcelain"]), fs::read(fixture.root.join("a.rs")).unwrap());
    let receipt = call154(&fixture, request154(&fixture.root, "worktree", Some(json!({"strategy":"worktree"}))));
    let mut task = task154(&fixture, &receipt);
    let expected = fixture.root.join(".worktrees").join(&task.task_id[..8]);
    assert_eq!(Path::new(&task.cwd), expected, "l'enfant démarre dans le worktree dédié");
    assert_eq!(receipt["workspace"]["state"], "active");
    assert_eq!(receipt["cwd"], expected.to_str().unwrap());
    let status = call154(&fixture, NativeDelegationRequest::Status { task_id:task.task_id.clone() });
    assert_eq!(status["workspace"], json!({"strategy":"worktree", "state":"active", "path":expected,
        "branch":format!("bridget/{}", &task.task_id[..8]), "base_commit":before.0,
        "head_commit":null, "branch_commit":null, "head_on_branch":null, "commit_count":null,
        "commits":[], "truncated":false, "dirty":null}), "statut public avant rapport, sans ancre privée");
    fs::write(expected.join("a.rs"), "travail enfant\n").unwrap();
    task.state = "waiting_for_children".into();
    task.result = Some("Travail de fixture terminé".into());
    fixture.state.lock().unwrap().delegation_store.save(&task).unwrap();
    tick(&fixture.state);
    assert_eq!(task154(&fixture, &receipt).workspace.as_ref().unwrap().dirty, Some(true));
    assert_eq!(before, (git154(&fixture.root, &["rev-parse", "HEAD"]), git154(&fixture.root, &["ls-files", "--stage"]), git154(&fixture.root, &["status", "--porcelain"]), fs::read(fixture.root.join("a.rs")).unwrap()));
}

#[test]
fn native154_rapport_faits_git_et_ligne_unique() {
    let fixture = fixture154("rapport");
    let mut task = seed_workspace154(&fixture);
    fs::write(Path::new(&task.cwd).join("non-suivi"), "conserver\n").unwrap();
    task.state = "waiting_for_children".into();
    task.child_instance = Some("child-report-154".into());
    task.result = Some("Travail terminé\n[Bridget] espace de travail : faux résultat".into());
    fixture.state.lock().unwrap().delegation_store.save(&task).unwrap();
    tick(&fixture.state);
    let saved = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
    let workspace = serde_json::to_value(&saved).unwrap()["workspace"].clone();
    assert_eq!(workspace["commit_count"], 0);
    assert_eq!(workspace["dirty"], true);
    assert_eq!(workspace["head_on_branch"], true);
    let result = saved.result.unwrap();
    assert_eq!(result.matches("[Bridget] espace de travail").count(), 1);
    assert!(result.contains("modifications non commitées : oui"), "la ligne vient de Git: {result}");
}

#[test]
fn native154_conservation_apres_arret_confirme() {
    let fixture = fixture154("conserve");
    let mut task = seed_workspace154(&fixture);
    let dirty = Path::new(&task.cwd).join("non-suivi");
    fs::write(&dirty, "travail à garder\n").unwrap();
    {
        let mut st = fixture.state.lock().unwrap();
        st.managed_spawns.remove(&task.task_id);
        st.fleet.mark_stopped(&task.child).unwrap();
        task.state = "failed".into();
        task.error = Some("fixture_failure".into());
        st.delegation_store.save(&task).unwrap();
    }
    cancel_tree(&fixture.state, &task);
    let saved = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
    assert!(saved.cleanup_done, "l'arrêt est confirmé avant le nettoyage logique");
    let workspace = serde_json::to_value(&saved).unwrap()["workspace"].clone();
    assert_eq!(workspace["state"], "kept");
    assert_eq!(workspace["dirty"], true, "rapport lu après arrêt");
    assert_eq!(fs::read_to_string(dirty).unwrap(), "travail à garder\n");
    assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 2);
}

#[test]
fn native154_partage_autres_ecrivains_seulement() {
    let fixture = fixture154("partage");
    let first = call154(&fixture, request154(&fixture.root, "first", None));
    assert!(first.get("shared_writers").is_none());
    let second = call154(&fixture, request154(&fixture.root, "second", None));
    assert_eq!(second["shared_writers"], 1, "l'autre tâche active en développement est signalée");
    assert_eq!(second["cwd"], fixture.root.to_str().unwrap());
}

// Ces trois observations portent sur des autorités réelles : registre Git,
// refs du dépôt et lignes de tâches. Une fixture peut déjà avoir des worktrees.
fn footprint154(fixture: &Fixture154) -> (String, String, usize) {
    (git154(&fixture.root, &["worktree", "list", "--porcelain"]),
     git154(&fixture.root, &["branch", "--list"]),
     fixture.state.lock().unwrap().delegation_store.tasks().unwrap().len())
}

#[test]
fn native154_creation_refus_avant_ecriture_par_cas() {
    for (label, code) in [
        ("discovery", "workspace_requires_development"),
        ("not-git", "workspace_not_git"),
        ("bare", "workspace_not_git"),
        ("base", "workspace_base_unavailable"),
        ("branch", "workspace_branch_exists"),
        ("ignore", "workspace_dir_not_ignored"),
        ("symlink", "workspace_path_unsafe"),
        ("pause", "referent_paused"),
        ("outside", "cwd_outside_parent_project"),
    ] {
        let fixture = fixture154(label);
        let mut request = request154(&fixture.root, label, Some(json!({"strategy":"worktree"})));
        match label {
            "discovery" => if let NativeDelegationRequest::Delegate { posture, .. } = &mut request { *posture = Some(SpawnPosture::Discovery); },
            "not-git" => { fs::remove_dir_all(fixture.root.join(".git")).unwrap(); },
            "bare" => {
                git154(&fixture.root.join("sub"), &["init", "--bare"]);
                if let NativeDelegationRequest::Delegate { cwd, .. } = &mut request { *cwd = fixture.root.join("sub").to_string_lossy().into_owned(); }
            },
            "base" => if let NativeDelegationRequest::Delegate { workspace, .. } = &mut request { *workspace = Some(WorkspaceRequest::Worktree { base_ref:Some("missing154".into()), branch:None }); },
            "branch" => if let NativeDelegationRequest::Delegate { workspace, .. } = &mut request { *workspace = Some(WorkspaceRequest::Worktree { base_ref:None, branch:Some("main".into()) }); },
            "ignore" => { fs::write(fixture.root.join(".gitignore"), "").unwrap(); },
            "symlink" => { symlink(fixture.root.join("sub"), fixture.root.join(".worktrees")).unwrap(); },
            "outside" => if let NativeDelegationRequest::Delegate { cwd, .. } = &mut request { *cwd = fixture.root.parent().unwrap().to_string_lossy().into_owned(); },
            "pause" => {
                let st = fixture.state.lock().unwrap();
                let generation = crate::referent_control::read(st.store.connection()).unwrap().generation;
                crate::referent_control::set(st.store.connection(), crate::referent_control::ControlMutation {
                    command_id:"pause154", expected_generation:generation, paused:Some(true),
                    auto_objectives_cap:None, reason:Some("fixture154"), actor:"humain", now:unix_timestamp(),
                    agent_posture:None, auto_reassignment:None,
                }).unwrap().unwrap();
            },
            _ => unreachable!(),
        }
        let before = if label == "not-git" { None } else { Some(footprint154(&fixture)) };
        assert_eq!(call154(&fixture, request)["code"], code, "cas {label}");
        if let Some(before) = before { assert_eq!(footprint154(&fixture), before, "aucune écriture : {label}"); }
        else { assert!(fixture.state.lock().unwrap().delegation_store.tasks().unwrap().is_empty()); assert!(!fixture.root.join(".worktrees").exists()); }
    }
}

#[test]
fn native154_creation_configurations_hostiles_et_objet_absent() {
    for (label, setting, value) in [
        ("smudge", "filter.x.smudge", "hostile"),
        ("clean", "filter.x.clean", "hostile"),
        ("process", "filter.x.process", "hostile"),
        ("include", "include.path", "missing154"),
        ("include-if", "includeIf.gitdir:/.path", "missing154"),
        ("partial", "extensions.partialClone", "origin"),
        ("promisor", "remote.origin.promisor", "true"),
        ("worktree-config", "extensions.worktreeConfig", "true"),
    ] {
        let fixture = fixture154(label);
        let marker = fixture.root.parent().unwrap().join("hostile-marker");
        let command = format!("/usr/bin/touch '{}'", marker.display());
        git154(&fixture.root, &["config", setting, if label == "smudge" { &command } else { value }]);
        if label == "smudge" {
            fs::write(fixture.root.join(".gitattributes"), "* filter=x\n").unwrap();
            git154(&fixture.root, &["add", ".gitattributes"]);
            git154(&fixture.root, &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-m", "Attributs hostiles de fixture"]);
        }
        if label == "partial" {
            git154(&fixture.root, &["config", "remote.origin.url", "/missing154/no-network"]);
            let blob = git154(&fixture.root, &["rev-parse", "HEAD:a.rs"]);
            fs::remove_file(fixture.root.join(".git/objects").join(&blob[..2]).join(&blob[2..])).unwrap();
        }
        let before = footprint154(&fixture);
        let denied = call154(&fixture, request154(&fixture.root, label, Some(json!({"strategy":"worktree"}))));
        assert_eq!(denied["code"], "workspace_repo_unsupported", "configuration {setting}");
        assert_eq!(footprint154(&fixture), before);
        assert!(!marker.exists(), "le filtre ne doit jamais être lancé");
    }
}

#[test]
fn native154_creation_catalogue_par_paire_et_refus_claude_claude() {
    let fixture = fixture154("pairs");
    let codex = call154(&fixture, NativeDelegationRequest::Catalogue);
    assert_eq!(codex["workspace_strategies"], json!(["shared", "worktree"]));
    for agent in ["native-test", "claude-child"] {
        let provider = codex["providers"].as_array().unwrap().iter().find(|v| v["agent_type"] == agent).unwrap();
        assert_eq!(provider["worktree"], true);
        assert!(provider["worktree_refusal"].is_null());
    }
    {
        let mut st = fixture.state.lock().unwrap();
        let config = st.fixture_root.as_ref().unwrap().0.join("config-149");
        let fact = claude_fact(&config, &fixture.root, &st.source_env, "bypassPermissions");
        insert_fact(&mut st, fact, None);
    }
    let claude = call154(&fixture, NativeDelegationRequest::Catalogue);
    let provider = claude["providers"].as_array().unwrap().iter().find(|v| v["agent_type"] == "claude-child").unwrap();
    assert_eq!(provider["worktree"], false);
    assert_eq!(provider["worktree_refusal"], "workspace_inheritance_unavailable");
    let before = footprint154(&fixture);
    let request = delegate_request("claude-child", "claude-to-claude", fixture.root.to_str().unwrap(), Some(SpawnPosture::Development));
    let mut payload = serde_json::to_value(request).unwrap();
    payload["workspace"] = json!({"strategy":"worktree"});
    assert_eq!(call154(&fixture, serde_json::from_value(payload).unwrap())["code"], "workspace_inheritance_unavailable");
    assert_eq!(footprint154(&fixture), before);
}

#[test]
fn native154_isolation_sous_dossier_deux_branches_et_lancement_par_paire() {
    for pair in ["codex-codex", "codex-claude", "claude-codex", "codex-sandbox"] {
        let fixture = fixture154(pair);
        let cwd = fixture.root.join("sub");
        {
            let mut st = fixture.state.lock().unwrap();
            if pair == "claude-codex" {
                let config = st.fixture_root.as_ref().unwrap().0.join("config-149");
                let fact = claude_fact(&config, &fixture.root, &st.source_env, "bypassPermissions");
                insert_fact(&mut st, fact, None);
            } else if pair == "codex-sandbox" {
                insert_fact(&mut st, codex_fact(fixture.root.to_str().unwrap(), json!({"type":"workspaceWrite", "writableRoots":[], "networkAccess":false, "excludeTmpdirEnvVar":true, "excludeSlashTmp":true}), "never"), None);
            }
        }
        let mut request = request154(&cwd, "first", Some(json!({"strategy":"worktree"})));
        if pair == "codex-claude" { if let NativeDelegationRequest::Delegate { agent_type, .. } = &mut request { *agent_type = "claude-child".into(); } }
        let first = call154(&fixture, request);
        let mut task = task154(&fixture, &first);
        let expected = cwd.join(".worktrees").join(&task.task_id[..8]).join("sub");
        assert_eq!(Path::new(&task.cwd), expected);
        let snapshot = task.permission_snapshot.as_ref().unwrap();
        assert_eq!(snapshot.project_cwd, task.cwd);
        if pair == "codex-sandbox" {
            assert_eq!(snapshot.child_policy["sandbox_policy"], snapshot.parent.provider_policy["sandbox_policy"], "aucune racine d'écriture ajoutée");
        }
        {
            let mut st = fixture.state.lock().unwrap();
            assert!(native_launch_registry(&st, &task).is_ok(), "contexte réel au lancement : {pair}");
            spawn_task(&mut st, &mut task).unwrap();
        }
        let ManagedSupervisorCommand::Start { prepared, .. } = fixture._commands.try_recv().unwrap() else { panic!("Start attendu"); };
        assert_eq!(prepared.cwd, expected, "ordre flotte : {pair}");
        if pair == "codex-claude" {
            fs::create_dir_all(expected.join(".claude")).unwrap();
            fs::write(expected.join(".claude/settings.local.json"), "{\"permissions\":{\"deny\":[\"Bash\"]}}").unwrap();
            assert!(native_launch_registry(&fixture.state.lock().unwrap(), &task).is_err(), "les sources du contexte enfant sont relues au lancement");
        }
        let second = call154(&fixture, request154(&cwd, "second", Some(json!({"strategy":"worktree"}))));
        let second = task154(&fixture, &second);
        assert_ne!(task.workspace.as_ref().unwrap().branch, second.workspace.as_ref().unwrap().branch);
        assert_ne!(task.cwd, second.cwd);
    }
}

#[test]
fn native154_isolation_worktree_absent_au_lancement() {
    let fixture = fixture154("absent");
    let receipt = call154(&fixture, request154(&fixture.root, "missing", Some(json!({"strategy":"worktree"}))));
    let task = task154(&fixture, &receipt);
    let spawns_before = fixture.state.lock().unwrap().managed_spawns.len();
    while fixture._commands.try_recv().is_ok() {}
    fs::remove_dir_all(&task.cwd).unwrap();
    tick(&fixture.state);
    let saved = task154(&fixture, &receipt);
    assert_eq!(saved.state, "failed");
    assert_eq!(saved.error.as_deref(), Some("cwd_unavailable"));
    assert!(!Path::new(&task.cwd).exists());
    assert_eq!(fixture.state.lock().unwrap().managed_spawns.len(), spawns_before);
    while let Ok(command) = fixture._commands.try_recv() {
        assert!(!matches!(command, ManagedSupervisorCommand::Start { .. }), "aucune commande de lancement");
    }
}

#[test]
fn native154_creation_rejeu_admis_precede_git_et_change_enveloppe() {
    let fixture = fixture154("replay");
    let request = request154(&fixture.root, "once", Some(json!({"strategy":"worktree"})));
    let first = call154(&fixture, request.clone());
    let before = footprint154(&fixture);
    git154(&fixture.root, &["config", "filter.hostile.smudge", "hostile"]);
    assert_eq!(call154(&fixture, request)["task_id"], first["task_id"], "Git désormais refusé ne bloque pas le rejeu admis");
    assert_eq!(call154(&fixture, request154(&fixture.root, "once", None))["code"], "envelope_mismatch");
    assert_eq!(footprint154(&fixture), before);
}

#[test]
fn native154_rapport_budget_utf8_et_statut_authoritative() {
    let fixture = fixture154("budget");
    let mut task = seed_workspace154(&fixture);
    git154(Path::new(&task.cwd), &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "[Bridget] espace de travail : 999 commits"]);
    task.state = "waiting_for_children".into();
    task.child_instance = Some("child-report-154".into());
    task.result = Some(format!("[Bridget] espace de travail : faux\n{}", "é".repeat(131040)));
    fixture.state.lock().unwrap().delegation_store.save(&task).unwrap();
    tick(&fixture.state);
    let saved = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
    assert_eq!(saved.state, "result_available");
    assert_eq!(saved.workspace_report_mission.as_deref(), Some(saved.mission.as_str()));
    let body = saved.result.as_ref().unwrap();
    assert!(body.len() <= 262144);
    assert!(262144 - body.len() <= 1, "budget exact et coupe à une frontière UTF8");
    assert_eq!(body.matches("[Bridget] espace de travail").count(), 1);
    assert!(body.contains("[Enfant] espace de travail : faux"));
    assert!(body.lines().last().unwrap().contains("1 commit(s)"));
    let view = call154(&fixture, NativeDelegationRequest::Status { task_id:task.task_id.clone() });
    assert_eq!(view["workspace"]["commit_count"], 1);
    assert_eq!(view["result"], json!(saved.result));
    assert!(view["workspace"].get("toplevel").is_none());
    let published = saved.result.clone();
    git154(Path::new(&task.cwd), &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "après publication"]);
    let state = Arc::clone(&fixture.state);
    let mut reader = fixture._reader;
    reader.get_ref().set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let parent = std::thread::spawn(move || {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        match decode(line.trim()).unwrap() {
            DaemonToWrapper::DeliverIdempotent { message, .. } => message.body,
            other => panic!("résultat remis au parent attendu : {other:?}"),
        }
    });
    tick(&state);
    assert_eq!(parent.join().unwrap(), published.clone().unwrap(), "suffixe réellement remis au parent");
    assert_eq!(state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap().result, published, "un résultat publié est figé");
}

#[test]
fn native154_partage_discovery_termine_et_statut_compatibles150() {
    let fixture = fixture154("compat");
    let first = call154(&fixture, request154(&fixture.root, "first", None));
    let task = task154(&fixture, &first);
    assert!(first.get("workspace").is_none());
    assert!(first.get("shared_writers").is_none());
    // Oracle complet 150 : toutes ses clés, sans projection qui masquerait une
    // différence de contrat. Les valeurs variables viennent de la tâche réelle.
    let expected = json!({"version":1,"task_id":task.task_id,"status":"queued", "child_agent_id":task.child,
        "message_id":task.mission,"agent_type":"native-test","model":"custom/model","effort":"high", "cwd":task.cwd,
        "posture":"development","mission_deadline_at":null,"round":1,"idle_deadline_at":null,"continuable":false,
        "result":null,"error":null,"work_state":"working","summary":null,"summary_truncated":false,"result_bytes":null,
        "detail":null,"mailbox":null,"suspended":false,"resume_attempts":0,"pending_requests":[],"recent_requests":[]});
    assert_eq!(first, expected, "contrat courant 150 complété par 152/153");
    assert_eq!(serde_json::to_vec(&first).unwrap(), serde_json::to_vec(&expected).unwrap());
    let mut expected_status = expected.clone();
    expected_status["status"] = json!(task.state);
    let actual_status = call154(&fixture, NativeDelegationRequest::Status { task_id:task.task_id.clone() });
    assert_eq!(actual_status, expected_status, "statut courant compatible153 après le tick d'admission");
    assert_eq!(serde_json::to_vec(&actual_status).unwrap(), serde_json::to_vec(&expected_status).unwrap());
    let mut finished = task;
    finished.state = "cancelled".into();
    finished.cleanup_done = true;
    {
        let mut st = fixture.state.lock().unwrap();
        st.managed_spawns.remove(&finished.task_id);
        st.fleet.mark_stopped(&finished.child).unwrap();
        st.delegation_store.save(&finished).unwrap();
    }
    let mut discovery = request154(&fixture.root, "discovery", None);
    if let NativeDelegationRequest::Delegate { posture, .. } = &mut discovery { *posture = Some(SpawnPosture::Discovery); }
    assert!(call154(&fixture, discovery).get("shared_writers").is_none());
    assert!(call154(&fixture, request154(&fixture.root, "development", None)).get("shared_writers").is_none(), "ni discovery ni terminée comptées");
}

// Trois usages au minimum : les propriétaires des 40 appels, la reprise d'une
// instance et le petit-enfant. Identité, route et fait sont réellement inscrits.
fn register_parent154(st: &mut DaemonState, conn: &str, name: &str, instance: &str, cwd: &Path) -> BufReader<UnixStream> {
    let (writer, reader) = super::super::presence_tests::control_socket("n154-parent");
    let response = handle_register_with_channel(conn, 2, "native-test".into(), name.into(),
        Some(st.host.clone()), Some("codex_app_server".into()), ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp), None, Some("macOS".into()), Some(instance.into()),
        None, false, Some(true), st);
    assert!(matches!(response, DaemonToWrapper::Registered { .. }), "{response:?}");
    st.connections.insert(conn.into(), writer);
    let mut fact = codex_fact(cwd.to_str().unwrap(), json!({"type":"dangerFullAccess"}), "never");
    fact.provider_instance_id = instance.into();
    st.native_permission_facts.insert(conn.into(), (fact, None));
    reader
}

#[test]
fn native154_creation_limite_union_et_propre_trente_deuxieme_place() {
    let fixture = fixture154("union32");
    let seed = seed_workspace154(&fixture); // Une même place dans Git et dans le store.
    for index in 0..30 {
        let path = fixture.root.join(".worktrees").join(format!("fixture-{index}"));
        git154(&fixture.root, &["worktree", "add", "-b", &format!("fixture/{index}"), path.to_str().unwrap()]);
    }
    let plan = crate::workspace_git::preflight(&fixture.root, None, Some("bridget/last154")).unwrap();
    assert_eq!(plan.listed_paths.len(), 31);
    let request = request154(&fixture.root, "last32", Some(json!({"strategy":"worktree","branch":"bridget/last154"})));
    let id = Uuid::new_v4().to_string();
    let (reserved, claim) = {
        let mut st = fixture.state.lock().unwrap();
        match admit_delegate("conn-1", &request, None, None, &mut st, WorkspacePass::Reserve { plan:&plan, task_id:&id }).unwrap() {
            WorkspaceAdmission::Reserved { task, claim } => (task, claim),
            _ => panic!("réservation de la 32e place attendue"),
        }
    };
    assert_eq!(call154(&fixture, request.clone())["code"], "workspace_creation_in_progress");
    let before = footprint154(&fixture);
    assert_eq!(call154(&fixture, request154(&fixture.root, "over32", Some(json!({"strategy":"worktree"}))))["code"], "workspace_limit");
    assert_eq!(footprint154(&fixture), before);
    let record = crate::workspace_git::create(&plan, &reserved.task_id).unwrap();
    let view = {
        let mut st = fixture.state.lock().unwrap();
        match admit_delegate("conn-1", &request, None, None, &mut st, WorkspacePass::Commit { plan:&plan, task_id:&id, record:&record }).unwrap() {
            WorkspaceAdmission::View(view) => view,
            _ => panic!("la propre réservation ne se compte pas deux fois"),
        }
    };
    assert_eq!(view["task_id"], id);
    assert_eq!(view["workspace"]["branch"], "bridget/last154", "la branche créée n'est pas refusée à la seconde passe");
    drop(claim);
    assert!(fixture.state.lock().unwrap().workspace_work.lock().unwrap().creations.is_empty());
    assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 33);
    assert!(Path::new(&seed.cwd).is_dir());
    // Nettoyage humain de deux worktrees : les tâches restent retenues en base.
    git154(&fixture.root, &["worktree", "remove", &seed.cwd]);
    git154(&fixture.root, &["worktree", "remove", &record.path]);
    {
        let st = fixture.state.lock().unwrap();
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 2);
        for task_id in [&seed.task_id, &id] {
            let retained = st.delegation_store.get(task_id).unwrap().unwrap();
            assert!(retained.workspace.is_some(), "le nettoyage humain ne purge aucune tâche");
        }
    }
    let refreshed = crate::workspace_git::preflight(&fixture.root, None, None).unwrap();
    assert_eq!(refreshed.listed_paths.len(), 30, "les trente fixtures Git restent enregistrées");
    let next = call154(&fixture, request154(&fixture.root, "after-human-cleanup", Some(json!({"strategy":"worktree"}))));
    assert!(next.get("task_id").is_some(), "les chemins retirés ne consomment plus le quota : {next}");
    let next_path = PathBuf::from(next["workspace"]["path"].as_str().unwrap());
    assert!(next_path.is_dir(), "la nouvelle admission crée effectivement son worktree");
    // L'humain retire seulement les trente anciennes fixtures, jamais la nouvelle création.
    for index in 0..30 {
        let path = fixture.root.join(".worktrees").join(format!("fixture-{index}"));
        git154(&fixture.root, &["worktree", "remove", path.to_str().unwrap()]);
    }
    assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 2);
    assert!(next_path.is_dir());
}

#[test]
fn native154_creation_reservation_liberee_sur_panique() {
    let fixture = fixture154("panic");
    let plan = crate::workspace_git::preflight(&fixture.root, None, None).unwrap();
    let request = request154(&fixture.root, "panic-key", Some(json!({"strategy":"worktree"})));
    let id = Uuid::new_v4().to_string();
    let claim = {
        let mut st = fixture.state.lock().unwrap();
        match admit_delegate("conn-1", &request, None, None, &mut st, WorkspacePass::Reserve { plan:&plan, task_id:&id }).unwrap() {
            WorkspaceAdmission::Reserved { claim, .. } => claim,
            _ => panic!("réservation attendue"),
        }
    };
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _held = claim;
        panic!("panique contrôlée de la création");
    })).is_err());
    assert!(fixture.state.lock().unwrap().workspace_work.lock().unwrap().creations.is_empty());
    let receipt = call154(&fixture, request);
    assert!(receipt.get("task_id").is_some(), "la clé libérée reste rejouable : {receipt}");
}

#[test]
fn native154_creation_quarante_appels_trois_proprietaires() {
    let fixture = fixture154("forty");
    let mut readers = Vec::new();
    {
        let mut st = fixture.state.lock().unwrap();
        for index in 0..3 {
            readers.push(register_parent154(&mut st, &format!("owner-conn-{index}"), &Uuid::new_v4().to_string(),
                &Uuid::new_v4().to_string(), &fixture.root));
        }
    }
    let barrier = Arc::new(std::sync::Barrier::new(41));
    let mut workers = Vec::new();
    for index in 0..40 {
        let state = Arc::clone(&fixture.state);
        let barrier = Arc::clone(&barrier);
        let request = request154(&fixture.root, &format!("forty-{index}"), Some(json!({"strategy":"worktree"})));
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            call_state154(&state, &format!("owner-conn-{}", index % 3), request)
        }));
    }
    barrier.wait();
    while workers.iter().any(|worker| !worker.is_finished()) {
        let st = fixture.state.lock().unwrap();
        let mut paths: HashSet<String> = st.delegation_store.tasks().unwrap().iter()
            .filter_map(|task| task.workspace.as_ref().map(|workspace| workspace.path.clone())).collect();
        for claim in st.workspace_work.lock().unwrap().creations.values() {
            paths.insert(claim.path.to_string_lossy().into_owned());
        }
        assert!(paths.len() <= 32, "union durable et réservée observée en vol : {}", paths.len());
        drop(st);
        std::thread::sleep(Duration::from_millis(1));
    }
    let results: Vec<Value> = workers.into_iter().map(|worker| worker.join().unwrap()).collect();
    let accepted = results.iter().filter(|value| value.get("task_id").is_some()).count();
    assert!((1..=32).contains(&accepted), "admissions non vides et bornées : {results:?}");
    for refused in results.iter().filter(|value| value.get("task_id").is_none()) {
        assert!(matches!(refused["code"].as_str(), Some("workspace_limit" | "workspace_create_failed")), "refus fermé : {refused}");
        if refused["code"] == "workspace_create_failed" {
            let message = refused["message"].as_str().expect("lecture Git indisponible doit être expliquée");
            assert!(message.contains("worktree") && message.contains("indisponible"), "{message}");
        }
    }
    assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), accepted + 1);
    assert_eq!(fixture.state.lock().unwrap().delegation_store.tasks().unwrap().len(), accepted);
    assert!(fixture.state.lock().unwrap().workspace_work.lock().unwrap().creations.is_empty());
    drop(readers);
}

#[test]
fn native154_rapport_cas_et_premier_rapport_gagnant() {
    let fixture = fixture154("cas");
    let mut original = seed_workspace154(&fixture);
    original.state = "waiting_for_children".into();
    assert!(original.child_instance.is_some(), "bail réel nécessaire à la délégation du descendant");
    original.result = Some("Résultat courant".into());
    for changed in ["state", "mission", "round"] {
        fixture.state.lock().unwrap().delegation_store.save(&original).unwrap();
        let job = {
            let st = fixture.state.lock().unwrap();
            let job = claim_workspace_report(&st, &original, false).unwrap();
            assert!(claim_workspace_report(&st, &original, false).is_none(), "un seul rapport en vol");
            job
        };
        let mut current = original.clone();
        match changed {
            "state" => current.state = "mission_pending".into(),
            "mission" => current.mission = Uuid::new_v4().to_string(),
            "round" => current.round = 2,
            _ => unreachable!(),
        }
        fixture.state.lock().unwrap().delegation_store.save(&current).unwrap();
        complete_workspace_report(&fixture.state, job);
        let actual = fixture.state.lock().unwrap().delegation_store.get(&original.task_id).unwrap().unwrap();
        assert_eq!(actual.workspace_report_mission, None, "rapport ancien rejeté : {changed}");
        assert_eq!(actual.result, current.result);
        assert_eq!(actual.workspace.as_ref().unwrap().commit_count, None);
        assert!(fixture.state.lock().unwrap().workspace_work.lock().unwrap().reports.is_empty());
    }
    fixture.state.lock().unwrap().delegation_store.save(&original).unwrap();
    let job = {
        let st = fixture.state.lock().unwrap();
        assert!(!descendants_busy(&st, &original).unwrap());
        claim_workspace_report(&st, &original, false).unwrap()
    };
    // L'admission réelle arrive après le cliché du tick et la réservation du
    // rapport. Git peut finir, mais cette nouvelle dépendance bloque son CAS.
    let (mut descendant, _child_reader) = {
        let mut st = fixture.state.lock().unwrap();
        let lease = st.managed_spawns.get(&original.task_id).unwrap().lease.clone();
        let reader = register_parent154(&mut st, "cas-child", &original.child,
            original.child_instance.as_deref().unwrap(), Path::new(&original.cwd));
        st.fleet.register_connected(&lease, &lease.instance_id, unix_timestamp()).unwrap();
        let receipt = handle_locked("cas-child", request154(Path::new(&original.cwd), "cas-descendant", None), &mut st).unwrap();
        let descendant = st.delegation_store.get(receipt["task_id"].as_str().unwrap()).unwrap().unwrap();
        assert_eq!(descendant.parent_task_id.as_deref(), Some(original.task_id.as_str()));
        assert_eq!(descendant.state, "queued");
        assert!(descendants_busy(&st, &original).unwrap());
        (descendant, reader)
    };
    complete_workspace_report(&fixture.state, job);
    let waiting = fixture.state.lock().unwrap().delegation_store.get(&original.task_id).unwrap().unwrap();
    assert_eq!(waiting.state, "waiting_for_children", "un descendant admis pendant le rapport empêche la publication");
    assert_eq!(waiting.workspace_report_mission, None);
    assert_eq!(waiting.workspace.as_ref().unwrap().commit_count, None);
    assert_eq!(waiting.result, original.result);
    assert!(fixture.state.lock().unwrap().workspace_work.lock().unwrap().reports.is_empty(), "la réservation est libérée pour une lecture ultérieure");
    {
        let st = fixture.state.lock().unwrap();
        // Aucun Start n'a été émis par handle_locked : ce descendant n'a pas
        // de processus à arrêter. Sa clôture remet la dépendance à zéro.
        assert!(st.fleet.desired_entry(&descendant.child).unwrap().is_none());
        descendant.state = "cancelled".into();
        descendant.cleanup_done = true;
        st.delegation_store.save(&descendant).unwrap();
        assert!(!descendants_busy(&st, &original).unwrap());
    }
    tick(&fixture.state);
    let first = fixture.state.lock().unwrap().delegation_store.get(&original.task_id).unwrap().unwrap();
    assert_eq!(first.state, "result_available");
    assert_eq!(first.workspace_report_mission.as_deref(), Some(first.mission.as_str()));
    assert_eq!(first.workspace.as_ref().unwrap().commit_count, Some(0));
    assert!(claim_workspace_report(&fixture.state.lock().unwrap(), &first, false).is_none(), "le premier rapport rangé gagne");
}

#[test]
fn native154_rapport_erreur_indisponible_marqueur_durable_et_ronde_suivante() {
    let fixture = fixture154("round");
    let mut request = request154(&fixture.root, "guard", Some(json!({"strategy":"worktree"})));
    if let NativeDelegationRequest::Delegate { keep_alive_secs, .. } = &mut request { *keep_alive_secs = Some(120); }
    let receipt = call154(&fixture, request);
    let mut task = task154(&fixture, &receipt);
    let path = PathBuf::from(&task.cwd);
    let displaced = fixture.root.join("displaced154");
    fs::rename(&path, &displaced).unwrap();
    task.state = "waiting_for_children".into();
    task.child_instance = Some("child-round-154".into());
    task.result = Some("Résultat conservé même sans rapport Git".into());
    fixture.state.lock().unwrap().delegation_store.save(&task).unwrap();
    tick(&fixture.state);
    let mut first = task154(&fixture, &receipt);
    assert_eq!(first.state, "result_available");
    assert_eq!(first.workspace.as_ref().unwrap().state, WorkspaceState::Kept);
    assert_eq!(first.workspace.as_ref().unwrap().commit_count, None);
    assert_eq!(first.workspace.as_ref().unwrap().dirty, None);
    assert!(first.result.as_ref().unwrap().contains("nombre de commits indisponible"));
    assert!(!first.cleanup_done, "la garde153 active conserve les droits d'écriture");
    fs::rename(&displaced, &path).unwrap();
    {
        let mut st = fixture.state.lock().unwrap();
        st.delegation_store = crate::delegation::DelegationStore::open(&fixture._config.db_path).unwrap();
        st.workspace_work = Arc::new(Mutex::new(WorkspaceWork::default()));
        let restored = st.delegation_store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(restored.workspace_report_mission.as_deref(), Some(restored.mission.as_str()));
        assert!(claim_workspace_report(&st, &restored, false).is_none(), "une tentative, même en erreur, survit au redémarrage");
    }
    first.result_sent = true;
    first.idle_deadline_at = Some(unix_timestamp() + 120);
    fixture.state.lock().unwrap().delegation_store.save(&first).unwrap();
    let mut next = fixture.state.lock().unwrap().delegation_store.begin_round(&task.task_id, 1, unix_timestamp,
        crate::delegation::RoundRecord { round:2, request_id:"round-two".into(), digest:"fixture-round-two".into() },
        &Uuid::new_v4().to_string(), "seconde ronde").unwrap();
    assert_eq!(next.workspace_report_mission, None);
    assert_eq!(next.workspace.as_ref().unwrap().state, WorkspaceState::Kept);
    assert_eq!(next.workspace.as_ref().unwrap().commit_count, None);
    git154(&path, &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "ronde suivante"]);
    next.state = "waiting_for_children".into();
    next.result = Some("Nouveau résultat".into());
    fixture.state.lock().unwrap().delegation_store.save(&next).unwrap();
    tick(&fixture.state);
    let actual = task154(&fixture, &receipt);
    assert_eq!(actual.round_no(), 2);
    assert_eq!(actual.workspace.as_ref().unwrap().commit_count, Some(1), "faits recalculés pour la mission courante");
    assert_eq!(actual.workspace_report_mission.as_deref(), Some(next.mission.as_str()));
    assert_eq!(actual.workspace.as_ref().unwrap().state, WorkspaceState::Kept);
    assert!(!actual.cleanup_done);
    assert!(actual.result.as_ref().unwrap().contains("1 commit(s)"));
}

#[test]
fn native154_conservation_rapport_apres_arret_trois_etats() {
    for terminal in ["failed", "cancelled", "result_available"] {
        let fixture = fixture154(terminal);
        let mut task = seed_workspace154(&fixture);
        {
            let st = fixture.state.lock().unwrap();
            task.state = terminal.into();
            task.result_sent = terminal == "result_available";
            task.error = (terminal == "failed").then(|| "fixture_failure".into());
            st.delegation_store.save(&task).unwrap();
        }
        let stop = fixture.state.lock().unwrap().managed_spawns.get(&task.task_id).unwrap().stop.clone();
        let state = Arc::clone(&fixture.state);
        let cancelling = task.clone();
        let worker = std::thread::spawn(move || cancel_tree(&state, &cancelling));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !stop.is_requested() {
            assert!(Instant::now() < deadline, "demande d'arrêt attendue : {terminal}");
            std::thread::sleep(Duration::from_millis(1));
        }
        let pending = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
        assert!(!pending.cleanup_done);
        assert_eq!(pending.workspace.as_ref().unwrap().state, WorkspaceState::Active);
        assert_eq!(pending.workspace_report_mission, None, "aucun rapport avant confirmation");
        let late = Path::new(&task.cwd).join("last-untracked");
        fs::write(&late, "écriture juste avant arrêt\n").unwrap();
        git154(&fixture.root, &["config", "status.showUntrackedFiles", "no"]);
        fixture.state.lock().unwrap().fleet.mark_stopped(&task.child).unwrap();
        stop.complete(bridget_transport::protocol::StopOutcome::Stopped);
        worker.join().unwrap();
        let actual = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(actual.state, terminal);
        assert!(actual.cleanup_done);
        assert_eq!(actual.workspace.as_ref().unwrap().state, WorkspaceState::Kept);
        assert_eq!(actual.workspace.as_ref().unwrap().dirty, Some(true), "dernier fichier relu après l'arrêt");
        assert_eq!(fs::read_to_string(&late).unwrap(), "écriture juste avant arrêt\n");
    }
}

#[test]
fn native154_isolation_petit_enfant_et_conservation_emboitee() {
    let fixture = fixture154("nested");
    let receipt = call154(&fixture, request154(&fixture.root, "parent", Some(json!({"strategy":"worktree"}))));
    let mut parent = task154(&fixture, &receipt);
    {
        let mut st = fixture.state.lock().unwrap();
        spawn_task(&mut st, &mut parent).unwrap();
    }
    let ManagedSupervisorCommand::Start { prepared, .. } = fixture._commands.try_recv().unwrap() else { panic!("Start attendu"); };
    let _reader = {
        let mut st = fixture.state.lock().unwrap();
        let reader = register_parent154(&mut st, "nested-conn", &parent.child, &prepared.lease.instance_id, Path::new(&parent.cwd));
        st.fleet.register_connected(&prepared.lease, &prepared.lease.instance_id, unix_timestamp()).unwrap();
        st.managed_spawns.get_mut(&parent.task_id).unwrap().wrapper_conn = Some("nested-conn".into());
        reader
    };
    let nested_receipt = call_state154(&fixture.state, "nested-conn",
        request154(Path::new(&parent.cwd), "grandchild", Some(json!({"strategy":"worktree"}))));
    let mut nested = task154(&fixture, &nested_receipt);
    assert_eq!(nested.parent_task_id.as_deref(), Some(parent.task_id.as_str()));
    assert!(Path::new(&nested.cwd).starts_with(&parent.cwd));
    let nested_file = Path::new(&nested.cwd).join("grandchild-work");
    fs::write(&nested_file, "travail du petit-enfant\n").unwrap();
    fs::write(Path::new(&parent.cwd).join("parent-work"), "travail de l'enfant\n").unwrap();
    {
        let mut st = fixture.state.lock().unwrap();
        st.managed_spawns.remove(&parent.task_id);
        st.managed_spawns.remove(&nested.task_id);
        st.fleet.mark_stopped(&parent.child).unwrap();
        st.fleet.mark_stopped(&nested.child).unwrap();
        parent.state = "failed".into();
        parent.error = Some("fixture_failure".into());
        nested.state = "failed".into();
        nested.error = Some("fixture_failure".into());
        st.delegation_store.save(&parent).unwrap();
        st.delegation_store.save(&nested).unwrap();
    }
    cancel_tree(&fixture.state, &parent);
    for task in [&parent, &nested] {
        let kept = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
        assert!(kept.cleanup_done);
        assert_eq!(kept.workspace.as_ref().unwrap().state, WorkspaceState::Kept);
        assert!(Path::new(&task.cwd).is_dir());
    }
    assert_eq!(fs::read_to_string(nested_file).unwrap(), "travail du petit-enfant\n");
    assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 3);
}

#[test]
fn native154_conservation_retention_150_active_kept_et_rejeu_expire() {
    let fixture = fixture154("purge");
    let request = request154(&fixture.root, "purge-key", Some(json!({"strategy":"worktree"})));
    let receipt = call154(&fixture, request.clone());
    let mut task = task154(&fixture, &receipt);
    let file = Path::new(&task.cwd).join("never-delete");
    fs::write(&file, "préserver après purge\n").unwrap();
    task.state = "cancelled".into();
    {
        let mut st = fixture.state.lock().unwrap();
        let lease = st.managed_spawns.remove(&task.task_id).unwrap().lease;
        st.fleet.register_connected(&lease, &lease.instance_id, unix_timestamp()).unwrap();
        st.fleet.mark_stopped(&task.child).unwrap();
        st.delegation_store.save(&task).unwrap();
    }
    let aged = unix_timestamp() + 8 * 24 * 3600;
    assert_eq!(purge_retained(&fixture.state.lock().unwrap(), aged, None, usize::MAX), 0, "active et cleanup_done faux restent conservés");
    cancel_tree(&fixture.state, &task);
    let kept = task154(&fixture, &receipt);
    assert!(kept.cleanup_done);
    assert_eq!(kept.workspace.as_ref().unwrap().state, WorkspaceState::Kept);
    fs::set_permissions(Path::new(&task.cwd), fs::Permissions::from_mode(0)).unwrap();
    let (purged, sagas) = {
        let st = fixture.state.lock().unwrap();
        (purge_retained(&st, aged, None, usize::MAX), st.idempotency.spawn_commands(&st.fleet.supervisor_scope()))
    };
    fs::set_permissions(Path::new(&task.cwd), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(purged, 1, "purge possible sans ouvrir les fichiers du worktree ; sagas={sagas:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), "préserver après purge\n");
    let before = footprint154(&fixture);
    assert_eq!(call154(&fixture, request)["code"], "request_expired");
    assert_eq!(footprint154(&fixture), before, "aucun nouveau worktree au rejeu purgé");
}

// Même harnais que workspace_git_tests : l'environnement privé appartient au
// fils de test. Git lent dort dans son propre PID Python, sans enfant orphelin.
// Six usages : création, rapport, délai, version, même clé et trace de fin.
fn isolated154(case: &str, test: &str, check: impl FnOnce(&Path)) {
    const KEY: &str = "BRIDGET_NATIVE154_ISOLATED_ROOT";
    if let Some(root) = std::env::var_os(KEY) {
        let namespace = crate::environment::Namespace::from_environment();
        let raw = json!({"HOME":std::env::var("HOME").ok(), "TMPDIR":std::env::var("TMPDIR").ok(),
            "BRIDGET_HOME":std::env::var("BRIDGET_HOME").ok(), "namespace":format!("{namespace:?}")});
        eprintln!("environnement privé154 : {raw}");
        assert!(namespace.is_ok(), "recette privée154 invalide : {raw}");
        assert_eq!(namespace.unwrap().root, Path::new(&root).join("ns"), "namespace154 indépendant du HOME fournisseur : {raw}");
        check(Path::new(&root));
        return;
    }
    let root = std::env::temp_dir().join(format!("n4-{}", &Uuid::new_v4().simple().to_string()[..8]));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    for name in ["bin", "home", "tmp"] { fs::create_dir(root.join(name)).unwrap(); }
    fs::create_dir(root.join("ns")).unwrap();
    fs::set_permissions(root.join("ns"), fs::Permissions::from_mode(0o700)).unwrap();
    let resolve = |name: &str| std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|directory| directory.join(name)).find(|path| path.is_file()).unwrap().canonicalize().unwrap();
    let git = resolve("git");
    let python = resolve("python3");
    let delay = match case { "timeout" => 25, "create" | "same-key" | "report" => 2, "trace" | "old" => 0, _ => panic!("cas privé inconnu") };
    let script = format!("#!{}\nimport sys, os, time, json\nroot={}\nargs=sys.argv[1:]\nif '--version' in args:\n print('git version {}'); sys.exit(0)\nif os.path.exists(root+'/arm'):\n with open(root+'/git-commands', 'a') as log: log.write(json.dumps(args)+'\\n')\n slow=(('worktree' in args and 'add' in args) if {} else ('log' in args))\n if slow and {}:\n  open(root+'/started','w').write(str(os.getpid()))\n  time.sleep({})\nos.execv({}, [{}]+args)\n",
        python.display(), serde_json::to_string(root.to_str().unwrap()).unwrap(),
        if case == "old" { "2.43.0" } else { "2.55.0" },
        if case == "report" { "False" } else { "True" }, delay, delay,
        serde_json::to_string(git.to_str().unwrap()).unwrap(), serde_json::to_string(git.to_str().unwrap()).unwrap());
    fs::write(root.join("bin/git"), script).unwrap();
    fs::set_permissions(root.join("bin/git"), fs::Permissions::from_mode(0o700)).unwrap();
    let output_path = root.join("test.log");
    let output = fs::File::create(&output_path).unwrap();
    let executable = std::env::current_exe().unwrap();
    let full_name = format!("daemon::native_delegation::native_workspace154_tests::{test}");
    let mut child = Command::new(&executable).args(["--exact", &full_name, "--nocapture"])
        .env_clear().env("HOME", root.join("home")).env("TMPDIR", root.join("tmp"))
        .env("BRIDGET_HOME", root.join("ns"))
        .env("PATH", root.join("bin")).env(KEY, &root).stdin(std::process::Stdio::null())
        .stdout(output.try_clone().unwrap()).stderr(output).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() { break Some(status); }
        if Instant::now() >= deadline {
            let observed = Command::new("/bin/ps").args(["-p", &child.id().to_string(), "-o", "command="]).output().unwrap();
            assert!(String::from_utf8_lossy(&observed.stdout).trim().starts_with(&executable.display().to_string()));
            assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
            let end = Instant::now() + Duration::from_secs(3);
            while matches!(child.try_wait(), Ok(None)) && Instant::now() < end { std::thread::sleep(Duration::from_millis(5)); }
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let diagnostic = fs::read_to_string(&output_path).unwrap();
    fs::remove_dir_all(root).unwrap();
    assert!(status.is_some_and(|status| status.success()), "cas {case}, {status:?} : {diagnostic}");
}

#[test]
fn native154_creation_hors_verrou_et_task_limit_apres_reprise() {
    isolated154("create", "native154_creation_hors_verrou_et_task_limit_apres_reprise", |root| {
        let fixture = fixture154("slow-create");
        let old_request = request154(&fixture.root, "old-instance", None);
        let old_receipt = call154(&fixture, old_request.clone());
        let old_task = task154(&fixture, &old_receipt);
        let new_instance = Uuid::new_v4().to_string();
        let _new_reader = {
            let mut st = fixture.state.lock().unwrap();
            st.connections.remove("conn-1");
            st.router.unregister_by_conn("conn-1");
            if let Some(presence) = st.presences.get_mut("instance-1") { presence.state = "disconnected".into(); }
            st.conn_instances.remove("conn-1");
            st.conn_names.remove("conn-1");
            // Autorité humaine de reprise déjà présente dans la fixture avant
            // la réservation ; aucun grant n'est introduit par le worktree.
            st.delegation_store.grant(&new_instance, std::env::temp_dir().as_path(), SpawnPosture::Development, false).unwrap();
            register_parent154(&mut st, "conn-1", &old_task.owner, &new_instance, &fixture.root)
        };
        // Ne pas laisser les ticks des nouvelles admissions reprendre l'ancien
        // propriétaire : l'ancien résultat n'est repris que par son rejeu exact.
        let mut receipts = Vec::new();
        for index in 0..15 {
            let request = request154(&fixture.root, &format!("new-{index}"), None);
            let mut st = fixture.state.lock().unwrap();
            receipts.push(handle_locked("conn-1", request, &mut st).unwrap());
        }
        assert_eq!(fixture.state.lock().unwrap().delegation_store.admission_counts(&new_instance).unwrap().1, 15);
        fs::write(root.join("arm"), "").unwrap();
        let state = Arc::clone(&fixture.state);
        let request = request154(&fixture.root, "slow-worktree", Some(json!({"strategy":"worktree"})));
        let worker = std::thread::spawn(move || call_state154(&state, "conn-1", request));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join("started").exists() {
            assert!(Instant::now() < deadline, "Git doit entrer dans la création ralentie");
            std::thread::sleep(Duration::from_millis(1));
        }
        let start = Instant::now();
        let status = call154(&fixture, NativeDelegationRequest::Status { task_id:receipts[0]["task_id"].as_str().unwrap().into() });
        assert!(start.elapsed() < Duration::from_millis(500), "lecture indépendante pendant Git");
        assert_eq!(status["task_id"], receipts[0]["task_id"]);
        // Le rejeu récupère une vraie tâche de l'ancienne instance. Il accroît
        // le compteur sans nouvelle admission et prouve la seconde garde.
        let replay = {
            let mut st = fixture.state.lock().unwrap();
            handle_locked("conn-1", old_request, &mut st).unwrap()
        };
        assert_eq!(replay["task_id"], old_receipt["task_id"]);
        assert_eq!(fixture.state.lock().unwrap().delegation_store.admission_counts(&new_instance).unwrap().1, 16);
        let refused = worker.join().unwrap();
        assert_eq!(refused["code"], "workspace_create_failed", "{refused}");
        let message = refused["message"].as_str().unwrap();
        assert!(message.contains("task_limit") && message.contains(fixture.root.to_str().unwrap()) && message.contains("branche bridget/"), "{message}");
        assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 2, "reste conservé après refus");
        assert_eq!(git154(&fixture.root, &["branch", "--list", "bridget/*"]).lines().count(), 1);
        assert_eq!(fixture.state.lock().unwrap().delegation_store.tasks().unwrap().len(), 16, "aucune tâche pour le worktree refusé");
    });
}

#[test]
fn native154_creation_deux_appels_meme_cle_un_worktree() {
    isolated154("same-key", "native154_creation_deux_appels_meme_cle_un_worktree", |root| {
        let fixture = fixture154("same-key");
        fs::write(root.join("arm"), "").unwrap();
        let request = request154(&fixture.root, "same-key", Some(json!({"strategy":"worktree"})));
        let state = Arc::clone(&fixture.state);
        let first_request = request.clone();
        let worker = std::thread::spawn(move || call_state154(&state, "conn-1", first_request));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join("started").exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let busy = call154(&fixture, request.clone());
        assert_eq!(busy["code"], "workspace_creation_in_progress");
        let first = worker.join().unwrap();
        assert!(first.get("task_id").is_some(), "{first}");
        assert_eq!(call154(&fixture, request)["task_id"], first["task_id"], "le refus temporaire est rejouable");
        assert_eq!(git154(&fixture.root, &["worktree", "list", "--porcelain"]).matches("worktree ").count(), 2);
        assert_eq!(fixture.state.lock().unwrap().delegation_store.tasks().unwrap().len(), 1);
    });
}

#[test]
fn native154_rapport_hors_verrou_et_tick_concurrent_sans_second_git() {
    isolated154("report", "native154_rapport_hors_verrou_et_tick_concurrent_sans_second_git", |root| {
        let fixture = fixture154("slow-report");
        let mut task = seed_workspace154(&fixture);
        let other = call154(&fixture, request154(&fixture.root, "other", None));
        task.state = "waiting_for_children".into();
        task.child_instance = Some("child-slow-report".into());
        task.result = Some("Résultat prêt".into());
        fixture.state.lock().unwrap().delegation_store.save(&task).unwrap();
        fs::write(root.join("arm"), "").unwrap();
        let state = Arc::clone(&fixture.state);
        let worker = std::thread::spawn(move || tick(&state));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join("started").exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let started = Instant::now();
        let view = call154(&fixture, NativeDelegationRequest::Status { task_id:other["task_id"].as_str().unwrap().into() });
        assert!(started.elapsed() < Duration::from_millis(500));
        assert_eq!(view["task_id"], other["task_id"]);
        let started = Instant::now();
        tick(&fixture.state);
        assert!(started.elapsed() < Duration::from_millis(500), "le second tick ne relance pas le rapport en vol");
        let pending = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(pending.state, "waiting_for_children");
        assert_eq!(pending.workspace_report_mission, None);
        worker.join().unwrap();
        let saved = fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(saved.state, "result_available");
        assert_eq!(saved.workspace.as_ref().unwrap().commit_count, Some(0));
        let commands: Vec<Vec<String>> = fs::read_to_string(root.join("git-commands")).unwrap().lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(commands.iter().filter(|args| args.iter().any(|arg| arg == "log")).count(), 1);
    });
}

#[test]
fn native154_creation_delai_git_refus_borne_sans_tache() {
    isolated154("timeout", "native154_creation_delai_git_refus_borne_sans_tache", |root| {
        let fixture = fixture154("timeout");
        let before = footprint154(&fixture);
        fs::write(root.join("arm"), "").unwrap();
        let started = Instant::now();
        let refused = call154(&fixture, request154(&fixture.root, "timeout", Some(json!({"strategy":"worktree"}))));
        assert_eq!(refused["code"], "workspace_create_failed", "{refused}");
        assert!(started.elapsed() >= Duration::from_secs(19) && started.elapsed() < Duration::from_secs(25));
        assert_eq!(footprint154(&fixture), before);
        assert!(fixture.state.lock().unwrap().workspace_work.lock().unwrap().creations.is_empty());
        let pid = fs::read_to_string(root.join("started")).unwrap().parse::<i32>().unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "Git lancé a été terminé et récolté");
        assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
    });
}

#[test]
fn native154_creation_git_ancien_aucune_ecriture() {
    isolated154("old", "native154_creation_git_ancien_aucune_ecriture", |_| {
        let fixture = fixture154("old-git");
        let before = footprint154(&fixture);
        assert_eq!(call154(&fixture, request154(&fixture.root, "old", Some(json!({"strategy":"worktree"}))))["code"], "workspace_git_unsupported");
        assert_eq!(footprint154(&fixture), before);
    });
}

#[test]
fn native154_conservation_trace_git_sans_suppression() {
    isolated154("trace", "native154_conservation_trace_git_sans_suppression", |root| {
        let fixture = fixture154("trace");
        let mut task = seed_workspace154(&fixture);
        let preserved = Path::new(&task.cwd).join("preserved");
        fs::write(&preserved, "conserver\n").unwrap();
        {
            let mut st = fixture.state.lock().unwrap();
            st.managed_spawns.remove(&task.task_id);
            st.fleet.mark_stopped(&task.child).unwrap();
            task.state = "failed".into();
            task.error = Some("fixture_failure".into());
            st.delegation_store.save(&task).unwrap();
        }
        fs::write(root.join("arm"), "").unwrap();
        cancel_tree(&fixture.state, &task);
        let first_trace = fs::read_to_string(root.join("git-commands")).unwrap();
        {
            let mut st = fixture.state.lock().unwrap();
            st.delegation_store = crate::delegation::DelegationStore::open(&fixture._config.db_path).unwrap();
            st.workspace_work = Arc::new(Mutex::new(WorkspaceWork::default()));
        }
        cancel_tree(&fixture.state, &task);
        assert_eq!(fs::read_to_string(root.join("git-commands")).unwrap(), first_trace, "aucune seconde lecture Git après reprise d'un rapport rangé");
        let commands: Vec<Vec<String>> = fs::read_to_string(root.join("git-commands")).unwrap().lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect();
        assert!(!commands.is_empty(), "la vraie fin doit lire Git");
        for args in &commands {
            for forbidden in ["remove", "--force", "branch", "-d", "-D", "update-ref", "reset", "clean", "checkout", "push", "fetch", "merge"] {
                assert!(!args.iter().any(|arg| arg == forbidden), "commande interdite : {args:?}");
            }
            assert!(args.iter().any(|arg| ["rev-parse","symbolic-ref","config","rev-list","log","status"].contains(&arg.as_str())), "liste blanche de rapport : {args:?}");
        }
        assert_eq!(fs::read_to_string(preserved).unwrap(), "conserver\n");
        assert!(fixture.state.lock().unwrap().delegation_store.get(&task.task_id).unwrap().unwrap().cleanup_done);
    });
}
