//! Session 149 — volet G-P : projections sanitaires du daemon
//! (`child_policy`, capture/recheck des sources, fait observé, snapshot).
//! S149-29..S149-31, oracles G-P ; mappings complets du contrat permissions.
//! Fixtures fichiers uniquement, aucun modèle, aucun processus fournisseur.

use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "native_permissions155_tests.rs"]
mod native_permissions155_tests;

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

#[test]
fn spec244_parix_source_globale_reste_autorisee_sur_une_autre_machine() {
    let (_root,env,mapping,protected)=remote162_fixture("244-parix");
    let parent=codex_fact("/origin162/opus2d",json!({"type":"dangerFullAccess"}),"never");
    assert!(crate::lifecycle::source_root(&parent,&mapping.source_root).is_ok());
    let (policy,posture)=project_remote_permissions(&parent,&codex_definition(),&mapping,&mapping.remote_root,
        &env,Some(SpawnPosture::Discovery),&protected).unwrap();
    assert_eq!(posture,SpawnPosture::Discovery);assert_eq!(policy["sandbox_policy"]["type"],"readOnly");
    let mut restricted=parent.clone();restricted.provider_policy["sandbox_policy"]=json!({"type":"readOnly"});
    assert!(project_remote_permissions(&restricted,&codex_definition(),&mapping,&mapping.remote_root,
        &env,Some(SpawnPosture::Discovery),&protected).is_err());
}

#[test]
fn spec241_same_host_preserves_attested_extra_directory_without_remote_mapping() {
    let (root,env,mut mapping,protected)=remote162_fixture("241-local-directory");
    mapping.source_root=mapping.remote_root.clone();
    let extra=root.join("attachments");fs::create_dir(&extra).unwrap();
    let parent=codex_fact(mapping.source_root.to_str().unwrap(),json!({
        "type":"workspaceWrite","networkAccess":false,
        "writableRoots":[mapping.source_root,extra],"excludeSlashTmp":true,"excludeTmpdirEnvVar":true
    }),"never");
    let before=serde_json::to_vec(&parent).unwrap();
    let project=|fact:&ProviderPermissions,local| {
        let mut operation=PermissionReadOperation::default();
        project_remote_permissions_in_operation(fact,&codex_definition(),&mapping,
            &mapping.remote_root,&env,Some(SpawnPosture::Development),&protected,&mut operation,false,local)
    };
    assert!(project(&parent,false).is_err(),"une autre machine exige sa correspondance explicite");
    let (policy,posture)=project(&parent,true).expect("le dossier est déjà autorisé et local");
    assert_eq!(posture,SpawnPosture::Development);
    assert_eq!(policy["sandbox_policy"]["writableRoots"],parent.provider_policy["sandbox_policy"]["writableRoots"]);
    assert_eq!(serde_json::to_vec(&parent).unwrap(),before);
    assert!(mapping.additional_root.is_none(),"pas de mutation du registre");
    // Même forme que le parent natif signalé : Claude, périmètre complet,
    // pièces jointes autorisées, puis enfant Codex sur la même machine.
    let mut native=parent.clone();native.driver="claude_stream_json".into();
    native.source="provider_turn".into();
    native.runtime_mode="full-access".into();
    native.provider_policy=json!({"kind":"claude","permission_mode":"bypassPermissions",
        "allow_dangerously_skip_permissions":true,"tools":{"type":"preset","preset":"claude_code"},
        "allowed_tools":["mcp__t3-code__*"],"settings_sources":"provider_default",
        "additional_directories":[mapping.source_root,extra],
        "permission_callback":{"kind":"t3_runtime","tool_approval":"allow","plan_exit":"deny"}});
    let mut target=codex_definition();target.mcp.interactive="codex".into();
    let mut operation=PermissionReadOperation::default();
    let (native_policy,_)=project_remote_permissions_in_operation(&native,&target,&mapping,
        &mapping.remote_root,&env,Some(SpawnPosture::Development),&protected,&mut operation,false,true)
        .expect("les pièces jointes autorisées ne bloquent pas le parent natif");
    assert_eq!(native_policy["sandbox_policy"]["type"],"dangerFullAccess");
    let mut invalid=parent.clone();
    invalid.provider_policy["sandbox_policy"]["writableRoots"]=json!([protected[0]]);
    assert!(project(&invalid,true).is_err());
    invalid.provider_policy["sandbox_policy"]["writableRoots"]=json!([root.join("missing")]);
    assert!(project(&invalid,true).is_err());
    let link=root.join("alias");std::os::unix::fs::symlink(&extra,&link).unwrap();
    invalid.provider_policy["sandbox_policy"]["writableRoots"]=json!([link]);
    assert!(project(&invalid,true).is_err(),"pas de nouvelle autorité via un alias");
}

#[test]
fn spec241_non_git_source_uses_existing_file_authority_without_new_permissions() {
    let (root,env,mut mapping,protected)=remote162_fixture("241-source");
    let project=mapping.remote_root.clone();
    // La fixture de permissions possède un .git sentinelle. La source hors Git
    // est un dossier frère ; sa véritable absence de dépôt doit être vérifiée.
    let external=root.parent().unwrap().join(format!("b241-review-{}",uuid::Uuid::new_v4()));
    fs::create_dir(&external).unwrap();
    let mut parent=codex_fact(project.to_str().unwrap(),json!({"type":"readOnly","networkAccess":false}),"never");
    assert!(crate::lifecycle::source_root(&parent,&external).is_err(),"le libellé full-access ne suffit pas");
    parent.provider_policy["sandbox_policy"]=json!({"type":"workspaceWrite","networkAccess":false,
        "writableRoots":[project,external],"excludeSlashTmp":true,"excludeTmpdirEnvVar":true});
    assert_eq!(crate::lifecycle::source_root(&parent,&external).unwrap(),external);
    let source=bridget_transport::protocol::WorkerSourceBase {cwd:external.to_string_lossy().into_owned(),
        expected_base_commit:None,workspace:Some(bridget_transport::protocol::WorkspaceRequest::Shared{}),
        workload:bridget_transport::protocol::AutoDelegateWorkload::General};
    crate::lifecycle::validate_parent_source_base(&parent,&source).unwrap();
    mapping.source_root=external.clone();mapping.remote_root=external.clone();
    let frozen=serde_json::to_vec(&parent).unwrap();
    let mut operation=PermissionReadOperation::default();
    let (_,posture)=project_remote_permissions_in_operation(&parent,&codex_definition(),&mapping,
        &external,&env,Some(SpawnPosture::Discovery),&protected,&mut operation,false,true).unwrap();
    assert_eq!(posture,SpawnPosture::Discovery);assert_eq!(serde_json::to_vec(&parent).unwrap(),frozen);
    assert!(!external.join(".git").exists());
    parent.provider_policy["sandbox_policy"]=json!({"type":"dangerFullAccess"});
    assert_eq!(crate::lifecycle::source_root(&parent,&external).unwrap(),external,
        "une permission explicite sur tout le système de fichiers couvre ce dossier");
    assert!(crate::lifecycle::source_root(&parent,&external.join("../unrelated")).is_err());
    fs::remove_dir_all(external).unwrap();
}

// 162 : ces fixtures précèdent l'implémentation ; aucune exécution fournisseur.
fn remote162_fixture(
    label: &str,
) -> (
    PathBuf,
    SourceEnvironment,
    crate::registry::ProjectMapping,
    Vec<PathBuf>,
) {
    let (root, env) = fixture_root(label);
    let root = fs::canonicalize(root).unwrap();
    let remote = root.join("project");
    fs::create_dir_all(remote.join("src")).unwrap();
    let private = root.join("worker-private");
    fs::create_dir_all(&private).unwrap();
    let mapping = crate::registry::ProjectMapping {
        additional_root: None,
        source_host: "origin162".into(),
        source_root: PathBuf::from("/origin162/project"),
        remote_root: remote,
        expected_git_identity: "project162".into(),
        mapping_revision: 1,
    };
    (root, env, mapping, vec![private])
}

#[test]
fn spec221_c19_view_projects_linked_parent_without_creating_its_checkout_or_gaining_rights() {
    let (_root,env,mapping,protected)=remote162_fixture("c19-linked-permissions");
    let linked="/origin162/project/.worktrees/source";
    let mut parent=codex_fact(linked,json!({"type":"workspaceWrite","networkAccess":false,
        "writableRoots":[linked,"/origin162/project/src"],
        "excludeSlashTmp":true,"excludeTmpdirEnvVar":true}),"never");
    let before=serde_json::to_vec(&parent).unwrap();
    let project=|parent:&ProviderPermissions,view:bool,posture:SpawnPosture| {
        let mut operation=PermissionReadOperation::default();
        project_remote_permissions_in_operation(parent,&codex_definition(),&mapping,
            &mapping.remote_root,&env,Some(posture),&protected,&mut operation,view,false)
    };
    assert!(project(&parent,false,SpawnPosture::Discovery).is_err(),
        "le mapping historique exige le dossier lié absent");
    let (policy,posture)=project(&parent,true,SpawnPosture::Discovery).unwrap();
    assert_eq!(posture,SpawnPosture::Discovery);
    assert_eq!(policy["sandbox_policy"],json!({"type":"readOnly","networkAccess":false}));
    assert_eq!(policy["approval_policy"],"never");
    assert!(!mapping.remote_root.join(".worktrees/source").exists());
    assert_eq!(serde_json::to_vec(&parent).unwrap(),before);
    assert!(project(&parent,true,SpawnPosture::Development).is_err());
    parent.provider_policy["sandbox_policy"]["writableRoots"]=json!(["/origin162/project-evil"]);
    assert!(project(&parent,true,SpawnPosture::Discovery).is_err());
    parent.provider_policy["sandbox_policy"]["writableRoots"]=json!([format!("{linked}/blocked")]);
    std::os::unix::fs::symlink(&protected[0],mapping.remote_root.join("blocked")).unwrap();
    assert!(project(&parent,true,SpawnPosture::Discovery).is_err());
}

#[test]
fn remote162_codex_roots_projected_by_components_without_origin_io() {
    let (_root, env, mapping, protected) = remote162_fixture("roots");
    let parent = codex_fact(
        "/origin162/project/src",
        json!({"type":"workspaceWrite","networkAccess":false,
        "writableRoots":["/origin162/project/src"],"excludeSlashTmp":true,"excludeTmpdirEnvVar":true}),
        "on-request",
    );
    let (policy, posture) = project_remote_permissions(
        &parent,
        &codex_definition(),
        &mapping,
        &mapping.remote_root.join("src"),
        &env,
        None,
        &protected,
    )
    .unwrap();
    assert_eq!(posture, SpawnPosture::Development);
    assert_eq!(
        policy["sandbox_policy"]["writableRoots"],
        json!([mapping.remote_root.join("src")])
    );
    assert_eq!(policy["sandbox_policy"]["networkAccess"], false);
    assert_eq!(policy["approval_policy"], "on-request");
    assert_eq!(policy["sandbox_policy"]["excludeSlashTmp"], true);
    let subdirectory = mapping.remote_root.join("src/subdirectory");
    std::fs::create_dir(&subdirectory).unwrap();
    let (nested, _) = project_remote_permissions(
        &parent,
        &codex_definition(),
        &mapping,
        &subdirectory,
        &env,
        None,
        &protected,
    )
    .unwrap();
    assert_eq!(
        nested["sandbox_policy"]["writableRoots"],
        policy["sandbox_policy"]["writableRoots"]
    );
}

#[test]
fn remote162_readonly_parent_never_gains_write_or_network() {
    let (_root, env, mapping, protected) = remote162_fixture("readonly");
    let parent = codex_fact(
        "/origin162/project/src",
        json!({"type":"readOnly","networkAccess":false}),
        "never",
    );
    assert!(
        project_remote_permissions(
            &parent,
            &codex_definition(),
            &mapping,
            &mapping.remote_root.join("src"),
            &env,
            Some(SpawnPosture::Development),
            &protected
        )
        .is_err()
    );
    let (policy, posture) = project_remote_permissions(
        &parent,
        &codex_definition(),
        &mapping,
        &mapping.remote_root.join("src"),
        &env,
        None,
        &protected,
    )
    .unwrap();
    assert_eq!(posture, SpawnPosture::Discovery);
    assert_eq!(
        policy["sandbox_policy"],
        parent.provider_policy["sandbox_policy"]
    );
}

#[test]
fn remote162_rejects_unmapped_roots_symlinks_and_worker_overlap() {
    let (_root, env, mapping, protected) = remote162_fixture("protected");
    let mut parent = codex_fact(
        "/origin162/project/src",
        json!({"type":"workspaceWrite","writableRoots":["/origin162/project-evil"]}),
        "never",
    );
    assert!(
        project_remote_permissions(
            &parent,
            &codex_definition(),
            &mapping,
            &mapping.remote_root.join("src"),
            &env,
            None,
            &protected
        )
        .is_err()
    );
    std::os::unix::fs::symlink(&protected[0], mapping.remote_root.join("secret")).unwrap();
    parent.provider_policy["sandbox_policy"]["writableRoots"] =
        json!(["/origin162/project/secret"]);
    assert!(
        project_remote_permissions(
            &parent,
            &codex_definition(),
            &mapping,
            &mapping.remote_root.join("src"),
            &env,
            None,
            &protected
        )
        .is_err()
    );
    assert!(
        project_remote_permissions(
            &parent,
            &codex_definition(),
            &mapping,
            &mapping.remote_root.join("src"),
            &env,
            None,
            &[mapping.remote_root.join("src")]
        )
        .is_err()
    );
}

#[test]
fn remote162_origin_proof_rechecks_revision_absence_and_opaque_rules() {
    let (root, env) = fixture_root("origin-proof");
    let fact = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    let digest = prove_parent_permissions(&fact, fact.revision).unwrap();
    assert_eq!(digest.len(), 64);
    assert!(prove_parent_permissions(&fact, fact.revision + 1).is_err());
    fs::write(
        root.join("settings.json"),
        r#"{"permissions":{"deny":["Read(secret)"]}}"#,
    )
    .unwrap();
    assert!(
        prove_parent_permissions(&fact, fact.revision).is_err(),
        "source absente devenue présente"
    );
    let changed = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    // Lot 171 : la preuve authentifie seulement l'appelant. Les règles ne
    // sont plus transplantées ; le distant applique sa propre configuration.
    assert!(prove_parent_permissions(&changed, changed.revision).is_ok());
}

#[test]
fn remote162_platform_managed_sources_do_not_cross_operating_systems() {
    assert_eq!(
        managed_settings_root("linux").unwrap(),
        PathBuf::from("/etc/claude-code")
    );
    assert_eq!(
        managed_settings_root("macos").unwrap(),
        PathBuf::from("/Library/Application Support/ClaudeCode")
    );
    assert!(managed_settings_root("unknown").is_err());
}

#[test]
fn remote162_claude_destination_uses_only_its_local_sources_and_rechecks() {
    let (root, env, mapping, protected) = remote162_fixture("destination");
    let parent = codex_fact(
        "/origin162/project/src",
        json!({"type":"dangerFullAccess"}),
        "never",
    );
    let definition = claude_definition(&root, &[]);
    let cwd = mapping.remote_root.join("src");
    let (policy, _) =
        project_remote_permissions(&parent, &definition, &mapping, &cwd, &env, None, &protected)
            .unwrap();
    assert_eq!(policy["launch_context"]["config_dir"], json!(root));
    recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected, None, None)
        .unwrap();
    fs::write(root.join("settings.json"), "{}").unwrap();
    assert!(
        recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected, None, None)
            .is_err()
    );
}

#[test]
fn remote162_claude_parent_on_other_host_preserves_explicit_tool_denies() {
    let (root, env, mapping, protected) = remote162_fixture("claude-parent");
    let mut parent = claude_fact(
        &root,
        &mapping.remote_root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    prove_parent_permissions(&parent, parent.revision).unwrap();
    // Le maître conserve les identités attestées ; cette destination ne peut
    // pas relire les fichiers de l'autre machine. Un chemin inexistant le prouve.
    parent.cwd = "/origin162/project/src".into();
    parent.provider_policy["launch_context"]["cli_path"] = json!("/not-mounted-origin/claude");
    parent.provider_policy["disallowed_tools"] = json!(["WebFetch"]);
    let (policy, _) = project_remote_permissions(
        &parent,
        &claude_definition(&root, &[]),
        &mapping,
        &mapping.remote_root.join("src"),
        &env,
        None,
        &protected,
    )
    .unwrap();
    assert_eq!(policy["disallowed_tools"], json!(["WebFetch"]));
    assert_ne!(
        policy["launch_context"]["cli_path"],
        parent.provider_policy["launch_context"]["cli_path"]
    );
    parent.provider_policy["disallowed_tools"] = json!(["Read(/origin162/private/**)"]);
    assert!(
        project_remote_permissions(
            &parent,
            &claude_definition(&root, &[]),
            &mapping,
            &mapping.remote_root.join("src"),
            &env,
            None,
            &protected
        )
        .is_err()
    );
}

#[test]
fn remote162_destination_settings_cannot_add_a_protected_directory() {
    let (root, env, mapping, protected) = remote162_fixture("settings-root");
    fs::write(
        root.join("settings.json"),
        serde_json::to_vec(&json!({
            "permissions":{"additionalDirectories":[protected[0]]}
        }))
        .unwrap(),
    )
    .unwrap();
    let parent = codex_fact(
        "/origin162/project/src",
        json!({"type":"dangerFullAccess"}),
        "never",
    );
    assert!(
        project_remote_permissions(
            &parent,
            &claude_definition(&root, &[]),
            &mapping,
            &mapping.remote_root.join("src"),
            &env,
            None,
            &protected
        )
        .is_err()
    );
}

#[test]
fn remote162_origin_unreadable_source_and_broken_source_refuse() {
    let (root, env) = fixture_root("bad-source");
    let parent = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    fs::write(root.join("settings.json"), "{broken-json").unwrap();
    assert!(prove_parent_permissions(&parent, parent.revision).is_err());
    fs::remove_file(root.join("settings.json")).unwrap();
    std::os::unix::fs::symlink(root.join("missing"), root.join("settings.json")).unwrap();
    assert!(prove_parent_permissions(&parent, parent.revision).is_err());
}

#[test]
fn remote162_origin_fragment_revision_is_rechecked_before_release() {
    let (root, env) = fixture_root("managed-fragment");
    let mut parent = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    // Source administrée en fixture privée, aucune écriture /etc ou /Library.
    let fragments = root.join("managed-settings.d");
    fs::create_dir(&fragments).unwrap();
    fs::write(fragments.join("10-model.json"), r#"{"model":"fixture"}"#).unwrap();
    parent.provider_policy["launch_context"]["permission_sources"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "kind":"managed", "path":fragments, "revision":source_revision(&fragments).unwrap()
        }));
    prove_parent_permissions(&parent, parent.revision).unwrap();
    fs::write(
        fragments.join("20-permissions.json"),
        r#"{"permissions":{"allow":["Bash"]}}"#,
    )
    .unwrap();
    assert!(prove_parent_permissions(&parent, parent.revision).is_err());
}

fn fixture_root(label: &str) -> (PathBuf, SourceEnvironment) {
    let root = std::env::temp_dir().join(format!(
        "bridget-np149d-{}-{}-{}",
        label,
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    // La capture travaille sur des chemins résolus (macOS : /tmp -> /private/tmp).
    let root = root.canonicalize().unwrap();
    let mut env: SourceEnvironment = BTreeMap::new();
    env.insert("HOME".into(), root.as_os_str().to_owned());
    // Fixture unitaire, jamais une preuve modèle : un vrai binaire Mach-O
    // nommé `claude` dans un PATH privé, que le contexte capture comme CLI directe.
    // `.git` borne la remontée des ancêtres : le ~/.claude réel n'est jamais lu.
    fs::create_dir_all(root.join(".git")).unwrap();
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::copy("/bin/echo", bin.join("claude")).unwrap();
    fs::set_permissions(bin.join("claude"), fs::Permissions::from_mode(0o755)).unwrap();
    env.insert(
        "PATH".into(),
        format!("{}:/usr/bin:/bin", bin.display()).into(),
    );
    env.insert("CLAUDE_CONFIG_DIR".into(), root.as_os_str().to_owned());
    (root, env)
}

fn claude_definition(root: &Path, args: &[&str]) -> AgentDefinition {
    AgentDefinition {
        command: "claude".into(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        protocol: "claude_stream_json".into(),
        forbidden_env: vec![],
        pass_env: vec![],
        claude_config_dir: Some(root.to_string_lossy().into_owned()),
        codex_home: None,
        permissions: "allow".into(),
        queue_capacity: 4,
        held_queue: false,
        notify_timeout_secs: 4,
        mcp: Default::default(),
        capabilities: Default::default(),
    }
}

fn codex_definition() -> AgentDefinition {
    AgentDefinition {
        command: "/bin/sh".into(),
        args: vec!["app-server".into()],
        protocol: "codex_app_server".into(),
        forbidden_env: vec![],
        pass_env: vec![],
        claude_config_dir: None,
        codex_home: None,
        permissions: "deny".into(),
        queue_capacity: 4,
        held_queue: false,
        notify_timeout_secs: 4,
        mcp: Default::default(),
        capabilities: Default::default(),
    }
}

fn codex_fact(cwd: &str, sandbox: Value, approval: &str) -> ProviderPermissions {
    ProviderPermissions {
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

fn claude_policy(mode: &str, callback_kind: &str, tool_approval: &str, _t3: bool) -> Value {
    json!({
        "kind":"claude",
        "permission_mode":mode,
        "tools":{"type":"preset","preset":"claude_code"},
        "permission_callback":{"kind":callback_kind,"tool_approval":tool_approval,"plan_exit":"deny"},
        "settings_sources":"provider_default",
    })
}

/// Le contexte est capturé réellement au `cwd` de la demande ; la racine du
/// parent (`cwd_parent`) reste indépendante pour tester les écarts de racine.
fn claude_fact(
    config: &Path,
    cwd: &Path,
    env: &SourceEnvironment,
    mode: &str,
    callback_kind: &str,
    tool_approval: &str,
    t3: bool,
) -> ProviderPermissions {
    let definition = claude_definition(config, &[]);
    let mut policy = claude_policy(mode, callback_kind, tool_approval, t3);
    policy["launch_context"] = capture_context_with_env(&definition, cwd, env).unwrap();
    ProviderPermissions {
        version: 1,
        source: if t3 {
            "provider_turn"
        } else {
            "native_wrapper"
        }
        .into(),
        run_id: "run-149".into(),
        provider_session_id: "sess-149".into(),
        provider_instance_id: "instance-1".into(),
        revision: 1,
        driver: "claude_stream_json".into(),
        cwd: cwd.to_string_lossy().into_owned(),
        runtime_mode: "approval-required".into(),
        interaction_mode: "default".into(),
        provider_policy: policy,
    }
}

#[test]
fn native149_codex_complet_devise_bypass_sans_yolo() {
    let (root, env) = fixture_root("codex-full");
    let fact = codex_fact(
        root.to_str().unwrap(),
        json!({"type":"dangerFullAccess"}),
        "never",
    );
    let definition = claude_definition(&root, &[]);
    let (policy, effective) = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], json!("bypassPermissions"));
    assert_eq!(
        policy["tools"],
        json!({"type":"preset","preset":"claude_code"})
    );
    assert_eq!(
        policy["permission_callback"],
        json!({"kind":"native_wrapper","tool_approval":"allow","plan_exit":"deny"})
    );
    // Le contexte de lancement de l'enfant est capturé réellement.
    assert!(policy["launch_context"]["cli_path"].is_string());
    // Aucun flag global yolo ne survit à la sanitisation des arguments.
    let mut args: Vec<String> = [
        "--dangerously-skip-permissions",
        "--yolo",
        "--permission-mode",
        "plan",
        "-c",
        "approval_policy=never",
        "-p",
        "mission-149",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    apply_child_arguments("claude_stream_json", &policy, &mut args).unwrap();
    assert!(
        !args.iter().any(|arg| matches!(
            arg.as_str(),
            "--yolo" | "--dangerously-skip-permissions" | "--allow-dangerously-skip-permissions"
        )),
        "aucun yolo hérité : {args:?}"
    );
    let position = args
        .iter()
        .position(|arg| arg == "--permission-mode")
        .unwrap();
    assert_eq!(args[position + 1], "bypassPermissions");
    // Lot 171 : Claude 2.1.251 (Cartae) refuse --permission-prompts. Hors
    // contrat de hooks attesté, le mode de droits seul est transmis.
    assert!(!args.iter().any(|arg| arg == "--permission-prompts"), "{args:?}");
    assert!(
        args.contains(&"mission-149".to_string()),
        "les arguments métier restent"
    );
}

#[test]
fn native149_codex_lecteur_refuse_le_developpement_et_reduit_en_decouverte() {
    let (root, env) = fixture_root("codex-readonly");
    let fact = codex_fact(
        root.to_str().unwrap(),
        json!({"type":"readOnly","networkAccess":false}),
        "never",
    );
    let definition = codex_definition();
    // Même famille : demande de développement refusée par nom.
    let erreur = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    // Sans posture : réduction conservatrice en readOnly.
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, None).unwrap();
    assert_eq!(effective, SpawnPosture::Discovery);
    assert_eq!(
        policy["sandbox_policy"],
        json!({"type":"readOnly","networkAccess":false})
    );
}

#[test]
fn native149_decouverte_reduit_et_conserve_les_refus_existant() {
    let (root, env) = fixture_root("discovery");
    let mut policy = claude_policy("default", "native_wrapper", "prompt", false);
    policy["tools"] = json!(["Read", "Glob", "Grep", "Bash", "Edit", "Write"]);
    policy["allowed_tools"] = json!(["Bash"]);
    policy["disallowed_tools"] = json!(["Custom-Tool"]);
    let mut fact = claude_fact(
        &root,
        &root,
        &env,
        "default",
        "native_wrapper",
        "prompt",
        false,
    );
    policy["launch_context"] = fact.provider_policy["launch_context"].clone();
    fact.provider_policy = policy;
    let definition = claude_definition(&root, &[]);
    let (policy, effective) = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Discovery),
    )
    .unwrap();
    assert_eq!(effective, SpawnPosture::Discovery);
    assert_eq!(policy["permission_mode"], json!("plan"));
    assert_eq!(policy["tools"], json!(["Read", "Glob", "Grep"]));
    let denied: Vec<String> = policy["disallowed_tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    for outil in [
        "Bash",
        "Edit",
        "Write",
        "NotebookEdit",
        "WebFetch",
        "WebSearch",
        "Custom-Tool",
    ] {
        assert!(
            denied.contains(&outil.to_string()),
            "{outil} doit rester refusé : {denied:?}"
        );
    }
    // La règle allow existante reste en dessous de l'inventaire sans jamais
    // réintroduire un outil retiré par la réduction.
    assert_eq!(policy["allowed_tools"], json!(["Bash"]));
    // La demande de développement sur un fait default reste admissible : le
    // parent médie déjà par prompts, l'enfant hérite ce régime.
    let (policy, effective) = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], json!("default"));
    assert_eq!(
        policy["permission_callback"]["kind"],
        json!("native_wrapper")
    );
}

#[test]
fn native149_workspace_codex_vers_claude_refuse_le_confinement() {
    let (root, env) = fixture_root("workspace");
    let cwd = root.to_str().unwrap();
    // workspace-write vers Claude : Bridget ne peut pas reproduire le bac à
    // sable du parent → refus nommé, jamais une réduction silencieuse.
    let fact = codex_fact(cwd, json!({"type":"workspaceWrite"}), "never");
    let erreur =
        child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "provider_confinement_unavailable");
    // externalSandbox même famille : même refus.
    let fact = codex_fact(
        cwd,
        json!({"type":"externalSandbox","networkAccess":"enabled"}),
        "never",
    );
    let erreur = child_policy(&fact, &codex_definition(), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "provider_confinement_unavailable");
    // dangerFullAccess mais approbation non "never" : aucun mapping honnête.
    let fact = codex_fact(cwd, json!({"type":"dangerFullAccess"}), "on-request");
    let erreur =
        child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // readOnly avec réseau ouvert vers Claude : mapping indisponible.
    let fact = codex_fact(
        cwd,
        json!({"type":"readOnly","networkAccess":true}),
        "never",
    );
    let erreur =
        child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
}

#[test]
fn native149_claude_meme_famille_exige_contexte_vivant_et_meme_racine() {
    let (root, env) = fixture_root("claude-same");
    let definition = claude_definition(&root, &[]);
    // Sans launch_context : aucune projection possible.
    let sans_context = ProviderPermissions {
        provider_policy: claude_policy("default", "native_wrapper", "prompt", false),
        cwd: root.to_string_lossy().into_owned(),
        ..claude_fact(
            &root,
            &root,
            &env,
            "default",
            "native_wrapper",
            "prompt",
            false,
        )
    };
    let erreur = child_policy(&sans_context, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Contexte vivant (capturé à la racine demandée) mais racine parent
    // différente : mapping refusé.
    let (autre_racine, _) = fixture_root("claude-same-autre");
    let mut fact = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    fact.cwd = autre_racine.to_string_lossy().into_owned();
    let erreur = child_policy(&fact, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Callback T3 avec approbation allow, mode non bypass : un enfant natif ne
    // peut pas s'auto-approuver à la place du parent T3.
    let fact = claude_fact(&root, &root, &env, "default", "t3_runtime", "allow", true);
    let erreur = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Plan hérité ne donne jamais le développement.
    let fact = claude_fact(
        &root,
        &root,
        &env,
        "plan",
        "native_wrapper",
        "prompt",
        false,
    );
    let erreur = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    // Bypass hérité avec le même contexte vivant : development complet.
    let fact = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    let (policy, effective) = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], json!("bypassPermissions"));
    assert_eq!(
        policy["permission_callback"]["kind"],
        json!("native_wrapper")
    );
}

#[test]
fn native149_claude_vers_codex_exige_lire_ou_complet_sans_regles() {
    let (root, env) = fixture_root("claude-codex");
    let definition = codex_definition();
    // Bypass parent → dangerFullAccess codex.
    let fact = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    let (policy, effective) = child_policy(
        &fact,
        &definition,
        &root,
        &env,
        Some(SpawnPosture::Development),
    )
    .unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(
        policy,
        json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}})
    );
    // Plan lecture seule parent → readOnly codex.
    let mut plan = claude_fact(
        &root,
        &root,
        &env,
        "plan",
        "native_wrapper",
        "prompt",
        false,
    );
    plan.provider_policy["tools"] = json!(["Read", "Glob", "Grep"]);
    let (policy, effective) = child_policy(&plan, &definition, &root, &env, None).unwrap();
    assert_eq!(effective, SpawnPosture::Discovery);
    assert_eq!(
        policy["sandbox_policy"],
        json!({"type":"readOnly","networkAccess":false})
    );
    // acceptEdits : ni complet ni lecture seule → mapping refusé.
    let fact = claude_fact(
        &root,
        &root,
        &env,
        "acceptEdits",
        "native_wrapper",
        "allow",
        false,
    );
    let erreur = child_policy(&fact, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Overrides de droits présents : le parent porte des règles → refus.
    let mut overrides = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    overrides.provider_policy["launch_context"]["settings_overrides"] =
        json!({"permissions":{"defaultMode":"default"}});
    let erreur = child_policy(&overrides, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Refus explicites parent : le mapping codex ne les repasse pas.
    let mut deny = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    deny.provider_policy["disallowed_tools"] = json!(["Bash"]);
    let erreur = child_policy(&deny, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
}

/// Régressions revue r1 : F1 (allowlist parent non représentable chez Codex) et
/// F3 (inventaire vide — aucun outil réel ne gagne les outils Codex). Le
/// témoin prouve que le complet valide reste inchangé.
#[test]
fn native149_claude_vers_codex_allowlist_et_inventaire_vide_ne_devienne_jamais_complet() {
    let (root, env) = fixture_root("claude-codex-regles");
    let definition = codex_definition();
    // F1 : bypass+allow avec allowlist restrictive (parent lecteur) — la
    // restriction n'a pas de traduction bac à sable Codex → refus nommé.
    let mut allowlist = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    allowlist.provider_policy["allowed_tools"] = json!(["Read", "Grep"]);
    let erreur = child_policy(&allowlist, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // F3 : inventaire vide en mode plan — jamais un readOnly codex déduit
    // d'un parent sans aucun outil.
    let mut vide = claude_fact(
        &root,
        &root,
        &env,
        "plan",
        "native_wrapper",
        "prompt",
        false,
    );
    vide.provider_policy["tools"] = json!([]);
    let erreur = child_policy(&vide, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Inventaire vide en bypass : ni complet (aucun Bash) ni lecture seule.
    let mut vide_bypass = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    vide_bypass.provider_policy["tools"] = json!([]);
    let erreur = child_policy(&vide_bypass, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Témoin : le complet valide sans règles reste dangerFullAccess.
    let temoin = claude_fact(
        &root,
        &root,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    let (policy, effective) = child_policy(&temoin, &definition, &root, &env, None).unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["sandbox_policy"], json!({"type":"dangerFullAccess"}));
}

#[test]
fn native149_capture_refuse_les_sources_opaques_et_garde_les_inline() {
    let (root, env) = fixture_root("capture");
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("opaque-cli");
    fs::write(&script, b"#!/bin/sh\necho rien\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    // Chaîne shell : pas une capacité de lancement prouvée.
    let mut opaque = env.clone();
    opaque.insert(
        "PATH".into(),
        format!("{}:/usr/bin:/bin", bin.display()).into(),
    );
    let definition = claude_definition(&root, &[]);
    let mut shell = definition.clone();
    shell.command = "opaque-cli".into();
    let erreur = capture_context_with_env(&shell, &root, &opaque).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // F5 : un lanceur binaire (non script) différent du `claude` du PATH ne
    // prouve pas qu'il transmet les drapeaux ; seule l'empreinte épinglée le ferait.
    fs::copy("/bin/echo", bin.join("wrapper-binaire")).unwrap();
    fs::set_permissions(
        bin.join("wrapper-binaire"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let mut binaire = definition.clone();
    binaire.command = "wrapper-binaire".into();
    let erreur = capture_context_with_env(&binaire, &root, &opaque).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Contrôle : la CLI directe `claude` du même PATH reste acceptée.
    assert!(capture_context_with_env(&definition, &root, &opaque).is_ok());
    // BASH_ENV : environnement opaque.
    let mut bash_env = env.clone();
    bash_env.insert("BASH_ENV".into(), "/fixture/injection".into());
    let erreur = capture_context_with_env(&definition, &root, &bash_env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --plugin-dir : refus.
    let erreur = capture_context_with_env(
        &claude_definition(&root, &["--plugin-dir", "/fixture"]),
        &root,
        &env,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --settings inline non JSON : refus.
    let erreur = capture_context_with_env(
        &claude_definition(&root, &["--settings", "{invalide"]),
        &root,
        &env,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --settings inline avec clé hors permissions : refus.
    let erreur = capture_context_with_env(
        &claude_definition(&root, &["--settings", r#"{"model":1}"#]),
        &root,
        &env,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --setting-sources hors vocabulaire : refus.
    let erreur = capture_context_with_env(
        &claude_definition(&root, &["--setting-sources", "git"]),
        &root,
        &env,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // enabledPlugins sans manifeste installé : source opaque.
    fs::write(
        root.join("settings.json"),
        br#"{"enabledPlugins":{"x":true}}"#,
    )
    .unwrap();
    let erreur = capture_context_with_env(&definition, &root, &env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    fs::remove_file(root.join("settings.json")).unwrap();
    // Hooks dans le settings utilisateur : inventaire natif, sans exécution.
    let (autre, _) = fixture_root("capture-hooks");
    let mut positif = env.clone();
    positif.insert("CLAUDE_CONFIG_DIR".into(), autre.as_os_str().to_owned());
    let hook = root.join("hook.sh");
    fs::write(&hook, b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        autre.join("settings.json"),
        r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"${CLAUDE_PROJECT_DIR}/hook.sh"}]}]}}"#,
    )
    .unwrap();
    let hook_context =
        capture_context_with_env(&claude_definition(&autre, &[]), &root, &positif).unwrap();
    assert_eq!(hook_context["hook_execution"], json!("native_same_host"));
    assert!(
        hook_context["permission_sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|source| source["kind"] == "hook" && source["path"] == json!(hook))
    );
    fs::write(
        autre.join("settings.json"),
        br#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"/missing/hook.sh"}]}]}}"#,
    )
    .unwrap();
    assert_eq!(
        capture_context_with_env(&claude_definition(&autre, &[]), &root, &positif),
        Err("permission_source_unavailable".into())
    );
    // Cas positifs : inline permissions conservé, sources sélectionnées.
    fs::remove_file(autre.join("settings.json")).unwrap();
    let context = capture_context_with_env(
        &claude_definition(
            &autre,
            &[
                "--settings",
                r#"{"permissions":{"defaultMode":"plan"}}"#,
                "--setting-sources",
                "user,project",
            ],
        ),
        &root,
        &positif,
    )
    .unwrap();
    assert_eq!(
        context["settings_overrides"],
        json!({"permissions":{"defaultMode":"plan"}})
    );
    assert_eq!(context["settings_sources"], json!(["user", "project"]));
    // Les sources `managed` sont celles de l'hôte (MDM / ClaudeCode système) ; seules celles de la fixture sont absentes.
    assert!(
        context["permission_sources"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|source| source["kind"] != "managed")
            .all(|source| source["revision"] == "absent"),
        "aucune source utilisateur/projet n'existe dans la fixture"
    );
}

#[test]
fn native149_recheck_detecte_la_mutation_du_settings_utilisateur() {
    let (root, env) = fixture_root("recheck");
    fs::write(root.join("settings.json"), br#"{"model":"fixture"}"#).unwrap();
    let definition = claude_definition(&root, &[]);
    let context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert_eq!(
        recheck_context_with_env(&context, &definition, &root, &env),
        Ok(())
    );
    // Mutation : refus nommé.
    fs::write(root.join("settings.json"), br#"{"model":"mute"}"#).unwrap();
    let erreur = recheck_context_with_env(&context, &definition, &root, &env).unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
    // Suppression : même refus.
    fs::remove_file(root.join("settings.json")).unwrap();
    let erreur = recheck_context_with_env(&context, &definition, &root, &env).unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
    // Révision fabriquée dans le contexte : refus.
    let mut faux = capture_context_with_env(&definition, &root, &env).unwrap();
    faux["permission_sources"][0]["revision"] =
        json!("sha256:0000000000000000000000000000000000000000000000000000000000000000");
    let erreur = recheck_context_with_env(&faux, &definition, &root, &env).unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
}

#[test]
fn native149_snapshot_fige_et_deduplique_les_contextes() {
    let (root, env) = fixture_root("snapshot");
    let fact = codex_fact(
        root.to_str().unwrap(),
        json!({"type":"dangerFullAccess"}),
        "never",
    );
    let (policy, _) =
        child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap();
    let fige = snapshot(
        "agent-149",
        "instance-1",
        fact.clone(),
        policy.clone(),
        &root,
    );
    assert_eq!(fige.version, 1);
    assert_eq!(fige.source, "native_wrapper");
    assert_eq!(fige.owner_agent_id, "agent-149");
    assert_eq!(fige.owner_instance_id, "instance-1");
    assert_eq!(fige.parent, fact);
    assert_eq!(fige.child_policy, policy);
    assert_eq!(PathBuf::from(&fige.project_cwd), root);
    // Parent codex sans contexte + enfant claude avec contexte : un seul.
    let contextes = snapshot_contexts(&fige);
    assert_eq!(contextes.len(), 1);
    // Deux contextes distincts (parent Claude, enfant Claude) : conservés tous les deux.
    let contexte_parent = json!({"cli_path":root,"cli_revision":"absent","resolved_cli_path":root,"resolved_cli_revision":"absent","config_dir":root,"settings_sources":"provider_default","permission_sources":[]});
    let mut parent_claude = fact.clone();
    parent_claude.provider_policy = json!({"kind":"claude","launch_context":contexte_parent});
    let deux = snapshot(
        "agent-149",
        "instance-1",
        parent_claude.clone(),
        policy.clone(),
        &root,
    );
    assert_eq!(snapshot_contexts(&deux).len(), 2);
    // Le même contexte parent et enfant : une seule entrée.
    let mut même = policy.clone();
    même["launch_context"] = parent_claude.provider_policy["launch_context"].clone();
    let un = snapshot("a", "i", parent_claude, même, &root);
    assert_eq!(snapshot_contexts(&un).len(), 1);
}

#[test]
fn native149_mode_runtime_suit_la_politique_reelle() {
    assert_eq!(
        runtime_mode(&json!({"permission_mode":"plan"})),
        "approval-required"
    );
    assert_eq!(
        runtime_mode(&json!({"permission_mode":"default"})),
        "approval-required"
    );
    assert_eq!(
        runtime_mode(&json!({"permission_mode":"acceptEdits"})),
        "auto-accept-edits"
    );
    assert_eq!(
        runtime_mode(&json!({"permission_mode":"bypassPermissions"})),
        "full-access"
    );
    assert_eq!(runtime_mode(&json!({"permission_mode":"auto"})), "auto");
    assert_eq!(runtime_mode(&json!({"permission_mode":"dontAsk"})), "auto");
    assert_eq!(
        runtime_mode(
            &json!({"kind":"codex","sandbox_policy":{"type":"dangerFullAccess"},"approval_policy":"never"})
        ),
        "full-access"
    );
    assert_eq!(
        runtime_mode(
            &json!({"kind":"codex","sandbox_policy":{"type":"readOnly"},"approval_policy":"never"})
        ),
        "auto"
    );
    assert_eq!(
        runtime_mode(
            &json!({"kind":"codex","sandbox_policy":{"type":"readOnly"},"approval_policy":"on-request"})
        ),
        "approval-required"
    );
}

struct MockSession {
    perms: Option<(String, Value, u64, Option<String>)>,
}

impl bridget_transport::transport::Transport for MockSession {
    fn deliver(
        &mut self,
        _message: &bridget_core::BridgetMessage,
    ) -> Result<(), bridget_transport::transport::TransportError> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn connection_id(&self) -> &str {
        "mock-149"
    }
}

impl bridget_transport::ManagedSession for MockSession {
    fn provider_permissions(&self) -> Option<(String, Value, u64, Option<String>)> {
        self.perms.clone()
    }
    fn descriptor(&self) -> bridget_transport::managed_session::ManagedSessionDescriptor {
        bridget_transport::managed_session::ManagedSessionDescriptor {
            transport: "mock-149".into(),
            mode: bridget_transport::protocol::PresenceMode::Cli,
            location: None,
        }
    }
    fn process_id(&self) -> u32 {
        std::process::id()
    }
    fn activate_journal(
        &self,
        _root: &Path,
        _agent: &str,
        _live_feed: Option<bridget_transport::journal::JournalLiveFeed>,
    ) -> std::io::Result<()> {
        Ok(())
    }
    fn drain_events(&self) -> Vec<bridget_transport::ManagedEvent> {
        Vec::new()
    }
    fn cancel_delivery(&self, _message_id: &str, _reason: &str) -> bool {
        false
    }
    fn stop(&self) {}
    fn is_busy(&self) -> bool {
        false
    }
}

#[test]
fn native149_fait_observe_atteste_la_session_et_le_canal_natif() {
    let (root, env) = fixture_root("observed");
    let definition = claude_definition(&root, &[]);
    let context = capture_context_with_env(&definition, &root, &env).unwrap();
    // Session Claude : contexte requis, cwd fournisseur réel conservé.
    let session = MockSession {
        perms: Some((
            "sess-149".into(),
            claude_policy("default", "native_wrapper", "prompt", false),
            3,
            Some(root.to_string_lossy().into_owned()),
        )),
    };
    let fact = observed_fact("instance-1", &definition, Some(&context), &session)
        .unwrap()
        .unwrap();
    assert_eq!(fact.provider_session_id, "sess-149");
    assert_eq!(fact.revision, 3);
    assert_eq!(fact.driver, "claude_stream_json");
    assert_eq!(fact.source, "native_wrapper");
    assert_eq!(fact.provider_instance_id, "instance-1");
    assert_eq!(fact.runtime_mode, "approval-required");
    assert_eq!(fact.interaction_mode, "default");
    assert_eq!(
        fact.run_id, "sess-149",
        "sans identité de tour, la session porte le fait"
    );
    assert_eq!(fact.provider_policy["launch_context"], context);
    assert_eq!(PathBuf::from(&fact.cwd), root);
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Ok(())
    );
    // Sans contexte publié : refus nommé.
    let erreur = observed_fact("instance-1", &definition, None, &session).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Codex sans cwd fournisseur : aucun fait inventé.
    let codex = codex_definition();
    let session_sans_cwd = MockSession {
        perms: Some((
            "sess-codex".into(),
            json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
            1,
            None,
        )),
    };
    let erreur = observed_fact("instance-1", &codex, None, &session_sans_cwd).unwrap_err();
    assert_eq!(erreur, "permission_attestation_unavailable");
    // Aucune permission annoncée : aucun fait, jamais un repli.
    let muet = MockSession { perms: None };
    assert_eq!(
        observed_fact("instance-1", &codex, None, &muet).unwrap(),
        None
    );
}

#[test]
fn native149_entrees_claude_sans_regles_sont_classees() {
    let (root, env) = fixture_root("inputs");
    let definition = claude_definition(&root, &[]);
    let mut context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert!(
        claude_inputs_without_rules(&context).unwrap(),
        "sources toutes absentes"
    );
    // Settings sans règles de droits : classé sans règles.
    fs::write(root.join("settings.json"), br#"{"model":"fixture"}"#).unwrap();
    context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert!(claude_inputs_without_rules(&context).unwrap());
    // Settings avec une règle de droits : classé avec règles.
    fs::write(
        root.join("settings.json"),
        br#"{"permissions":{"deny":["Bash"]}}"#,
    )
    .unwrap();
    context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert!(!claude_inputs_without_rules(&context).unwrap());
    // Overrides : avec règles même si les sources sont vides.
    fs::remove_file(root.join("settings.json")).unwrap();
    let mut overrides = capture_context_with_env(&definition, &root, &env).unwrap();
    overrides["settings_overrides"] = json!({"permissions":{"defaultMode":"default"}});
    assert!(!claude_inputs_without_rules(&overrides).unwrap());
    // Révision muée : refus nommé avant tout classement.
    fs::write(root.join("settings.json"), br#"{"model":"mue"}"#).unwrap();
    let erreur = claude_inputs_without_rules(&context).unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
    // Une source sélectionnée comme "cli" est lue comme settings : contenu
    // sans règles de droits, le classement reste positif.
    fs::remove_file(root.join("settings.json")).unwrap();
    fs::write(root.join("cli-settings.json"), br#"{"effortLevel":"high"}"#).unwrap();
    let mut cli_context = capture_context_with_env(&definition, &root, &env).unwrap();
    cli_context["permission_sources"][0]["path"] =
        json!(root.join("cli-settings.json").to_string_lossy().to_string());
    cli_context["permission_sources"][0]["kind"] = json!("cli");
    // Les captures 171 sont au format 2 : révision v2 des sources.
    cli_context["permission_sources"][0]["revision"] = json!(
        bridget_transport::protocol::permission_source_revision_v2(
            &root.join("cli-settings.json"),
            16 * 1024 * 1024
        )
        .unwrap()
    );
    assert!(claude_inputs_without_rules(&cli_context).unwrap());
}

// Un nom de fichier seul ne qualifie pas une dépendance ni son bootstrap.
#[test]
fn native171_session_start_sdk_namesake_does_not_change_permission_refusal() {
    let (root, _) = fixture_root("171-sdk-namesake");
    let script = root.join("ensure_agent_sdk.py");
    fs::write(&script, "raise SystemExit(0)\\n").unwrap();
    let context = json!({
        "attestation_version":2, "hook_execution":"native_same_host",
        "permission_sources":[
            {"kind":"hook","path":script,"revision":"sha256:declared"}
        ]
    });
    assert_eq!(
        super::origin_equivalence(&context).unwrap_err(),
        "permission_hook_equivalence_unavailable"
    );
    assert!(!super::session_start_sdk_bootstrap_declared(&context));
}

#[test]
fn native171_manifestless_plugin_and_hook_revision_recheck() {
    let (root, env) = fixture_root("manifestless171");
    let plugin = root.join("plugins/cache/fixture-market/manifestless/1.0.0");
    let market = root.join("fixture-market");
    fs::create_dir_all(plugin.join("scripts")).unwrap();
    fs::create_dir_all(market.join(".claude-plugin")).unwrap();
    let hook = plugin.join("scripts/gate.sh");
    fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        root.join("settings.json"),
        serde_json::to_vec(&json!({
            "enabledPlugins":{"manifestless@fixture-market":true}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("plugins/installed_plugins.json"),
        serde_json::to_vec(&json!({
            "plugins":{"manifestless@fixture-market":[{"scope":"user","installPath":plugin}]}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("plugins/known_marketplaces.json"),
        serde_json::to_vec(&json!({
            "fixture-market":{"installLocation":market}
        }))
        .unwrap(),
    )
    .unwrap();
    let declaration = market.join(".claude-plugin/marketplace.json");
    fs::write(
        &declaration,
        serde_json::to_vec(&json!({"plugins":[{
            "name":"manifestless","strict":false,"hooks":{"SessionStart":[{"hooks":[{
                "type":"command","command":"\"${CLAUDE_PLUGIN_ROOT}/scripts/gate.sh\""
            }]}]}
        }]}))
        .unwrap(),
    )
    .unwrap();
    let definition = claude_definition(&root, &[]);
    let context = capture_context_with_env(&definition, &root, &env).unwrap();
    let sources = context["permission_sources"].as_array().unwrap();
    assert!(sources.iter().any(
        |s| s["path"] == json!(plugin.join(".claude-plugin/plugin.json"))
            && s["revision"] == "absent"
    ));
    assert!(sources.iter().any(|s| s["path"] == json!(declaration)));
    assert!(
        sources
            .iter()
            .any(|s| s["kind"] == "hook" && s["path"] == json!(hook))
    );
    recheck_context_with_env(&context, &definition, &root, &env).unwrap();
    assert!(!claude_inputs_without_rules(&context).unwrap());
    // Runtime lease markers do not change any selected source file.
    fs::create_dir(plugin.join(".in_use")).unwrap();
    fs::write(plugin.join(".in_use/fixture"), "lease").unwrap();
    recheck_context_with_env(&context, &definition, &root, &env).unwrap();
    fs::write(&hook, "#!/bin/sh\nexit 2\n").unwrap();
    assert_eq!(
        claude_inputs_without_rules(&context).unwrap_err(),
        "settings_revision_changed"
    );
}

/// Racine additionnelle (pièces jointes) sur dossiers synthétiques : traduction
/// vers la cible, refus hors des deux racines, aucune traduction partielle.
#[test]
fn native171_additional_root_mapping_translates_and_refuses_without_private_inputs() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let (root, _) = fixture_root("additional-root171");
    let remote = root.join("project");
    fs::create_dir(&remote).unwrap();
    let extra_remote = root.join("attachments-target");
    fs::create_dir(&extra_remote).unwrap();
    fs::set_permissions(&extra_remote, fs::Permissions::from_mode(0o700)).unwrap();
    let private = root.join("worker-private");
    fs::create_dir(&private).unwrap();
    let protected = vec![private];
    let uid = bridget_transport::fsutil::effective_uid();
    let mapping = crate::registry::ProjectMapping {
        additional_root: Some(crate::registry::AdditionalDirectoryMapping171 {
            source_root: PathBuf::from("/origin171/attachments"),
            remote_root: extra_remote.clone(),
            source_uid: 501,
            source_mode: 0o755,
            remote_uid: uid,
            remote_mode: 0o700,
            access: "inherit".into(),
        }),
        source_host: "origin171".into(),
        source_root: PathBuf::from("/origin171/project"),
        remote_root: remote.clone(),
        expected_git_identity: "project171".into(),
        mapping_revision: 1,
    };
    crate::registry::validate_additional_mapping171(&mapping).unwrap();
    let parent = json!({"permission_mode": "default", "tools": ["Read"],
        "additional_directories": ["/origin171/project", "/origin171/attachments"]});
    let mut projected = parent.clone();
    project_policy_directories171(&mut projected, &mapping, &protected, None, false).unwrap();
    let mut expected = parent.clone();
    expected["additional_directories"] = json!([remote, extra_remote]);
    assert_eq!(projected, expected); // Seuls les dossiers changent.
    assert_eq!(
        mapped_permission_directory171(&mapping, Path::new("/origin171/attachments/sub")).unwrap(),
        extra_remote.join("sub")
    );
    for outside in ["/origin171", "/origin171/attachments-other", "/origin171/attachments/../outside"] {
        assert_eq!(
            mapped_permission_directory171(&mapping, Path::new(outside)),
            Err("permission_mapping_unavailable".into())
        );
    }
    let mut absent = mapping.clone();
    absent.additional_root = None;
    let mut refused = parent.clone();
    assert_eq!(
        project_policy_directories171(&mut refused, &absent, &protected, None, false),
        Err("permission_mapping_unavailable".into())
    );
    assert_eq!(refused, parent); // Aucun parent partiellement traduit.
    let invalid: [fn(&mut crate::registry::AdditionalDirectoryMapping171); 4] = [
        |extra| extra.source_root = PathBuf::from("/origin171/project/sub"),
        |extra| extra.source_root = PathBuf::from("/origin171/../attachments"),
        |extra| extra.access = "all".into(),
        |extra| extra.remote_mode = 0o755,
    ];
    for change in invalid {
        let mut bad = mapping.clone();
        change(bad.additional_root.as_mut().unwrap());
        assert!(crate::registry::validate_additional_mapping171(&bad).is_err());
    }
    let mut overlapping = mapping.clone();
    overlapping.additional_root.as_mut().unwrap().remote_root = remote.clone();
    assert!(crate::registry::validate_additional_mapping171(&overlapping).is_err());
    verify_permission_directory171(&extra_remote, uid, 0o700, &protected).unwrap();
    assert!(verify_permission_directory171(&extra_remote, uid + 1, 0o700, &protected).is_err());
    assert!(verify_permission_directory171(&extra_remote, uid, 0o755, &protected).is_err());
    assert!(verify_permission_directory171(&extra_remote.join("absent171"), uid, 0o700, &protected).is_err());
    let mut blocked = protected.clone();
    blocked.push(extra_remote.clone());
    assert!(verify_permission_directory171(&extra_remote, uid, 0o700, &blocked).is_err());
    let link = root.join("attachments-link");
    symlink(&extra_remote, &link).unwrap();
    assert!(verify_permission_directory171(&link, uid, 0o700, &protected).is_err());
}

#[test]
fn native171_configured_absolute_claude_reduced_path_preserves_identity_and_rechecks() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let (root, _) = fixture_root("absolute-configured-reduced-path171");
    let config = root.join(".claude");
    fs::create_dir(&config).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o700)).unwrap();
    let bin = root.join(".local/bin");
    let versions = root.join(".local/share/claude/versions");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&versions).unwrap();
    let real = versions.join("2.1.251");
    fs::copy("/bin/echo", &real).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o755)).unwrap();
    let configured = bin.join("claude");
    symlink(&real, &configured).unwrap();
    let mut definition = claude_definition(&root, &[]);
    definition.command = configured.to_string_lossy().into_owned();
    definition.claude_config_dir = None; // Same null registry field as the real worker.
    // Exact process PATH read from worker1772632; the operation prepends only
    // its current executable directory. No user-bin or hook PATH is injected.
    let reduced = "/home/moi/.local/lib/bridget-tools/git-2.56.0/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin:/snap/bin";
    let source = SourceEnvironment::from([
        ("HOME".into(), root.as_os_str().to_owned()), ("PATH".into(), reduced.into()),
    ]);
    let operation_environment = build_environment(&definition, &source).unwrap();
    assert_eq!(source.get("PATH").unwrap(), &std::ffi::OsString::from(reduced));
    assert!(!std::env::split_paths(operation_environment.get("PATH").unwrap()).any(|p| p == bin));
    assert_eq!(resolved("claude", &operation_environment), Err("permission_source_unavailable".into()));
    let context = capture_context(&definition, &root, &source).unwrap();
    assert_eq!(context["cli_path"], json!(fs::canonicalize(&real).unwrap()));
    assert_eq!(context["resolved_cli_path"], json!(fs::canonicalize(&real).unwrap()));
    assert_eq!(context["cli_revision"], json!(executable_revision(&real).unwrap()));
    assert_eq!(context["resolved_cli_revision"], context["cli_revision"]);
    recheck_context(&context, &definition, &root, &source).unwrap();
    // No fallback or permission added to a bare or foreign configured command.
    let mut bare = definition.clone();bare.command = "claude".into();
    assert_eq!(capture_context(&bare, &root, &source), Err("permission_source_unavailable".into()));
    let mut wrong = definition.clone();wrong.command = "/bin/echo".into();
    assert_eq!(capture_context(&wrong, &root, &source), Err("permission_source_unavailable".into()));
    let foreign_bin = root.join("foreign/bin");fs::create_dir_all(&foreign_bin).unwrap();
    let foreign = foreign_bin.join("claude");fs::copy(&real, &foreign).unwrap();
    fs::set_permissions(&foreign, fs::Permissions::from_mode(0o755)).unwrap();
    let mut foreign_definition = definition.clone();foreign_definition.command = foreign.to_string_lossy().into_owned();
    assert_eq!(recheck_context(&context, &foreign_definition, &root, &source), Err("settings_revision_changed".into()));
    fs::remove_file(&configured).unwrap();symlink(&foreign, &configured).unwrap();
    assert_eq!(recheck_context(&context, &definition, &root, &source), Err("settings_revision_changed".into()));
    fs::remove_file(&configured).unwrap();symlink(&real, &configured).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(capture_context(&definition, &root, &source), Err("permission_source_unavailable".into()));
    assert_eq!(recheck_context(&context, &definition, &root, &source), Err("settings_revision_changed".into()));
    fs::set_permissions(&real, fs::Permissions::from_mode(0o755)).unwrap();
    let original = fs::read(&real).unwrap();let mut mutated = original.clone();mutated.push(0);fs::write(&real, &mutated).unwrap();
    assert_eq!(recheck_context(&context, &definition, &root, &source), Err("settings_revision_changed".into()));
    fs::write(&real, &original).unwrap();recheck_context(&context, &definition, &root, &source).unwrap();
    let mut injected = operation_environment.clone();injected.insert("BASH_ENV".into(), "/not-authorized171".into());
    assert_eq!(capture_context_with_env(&definition, &root, &injected), Err("permission_source_unavailable".into()));
    // Hook dependencies still need their own closed resolution; this change
    // neither adds a PATH directory nor suppresses a missing native hook tool.
    fs::write(config.join("settings.json"), r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"missing-native-hook171"}]}]}}"#).unwrap();
    assert_eq!(capture_context(&definition, &root, &source), Err("permission_source_unavailable".into()));
    fs::remove_file(config.join("settings.json")).unwrap();
    fs::write(&real, b"#!/bin/sh\nexit 0\n").unwrap();
    assert_eq!(capture_context(&definition, &root, &source), Err("permission_source_unavailable".into()));
    fs::remove_dir_all(root).unwrap();
}


#[test]
fn native171_remote_uses_destination_configuration_without_cloning_origin() {
    let (root, env, mapping, protected) = remote162_fixture("native-host");
    let cwd = mapping.remote_root.join("src");
    let mut parent = claude_fact(&root, &cwd, &env, "bypassPermissions", "native_wrapper", "allow", false);
    parent.cwd = "/origin162/project/src".into();
    parent.provider_policy["launch_context"]["cli_path"] = json!("/another-computer/claude");
    parent.provider_policy["disallowed_tools"] = json!(["WebFetch"]);
    // Real failure shape: native metadata refers to a plugin missing on this
    // computer. Native Claude owns loading/warnings; Bridget does not clone it.
    fs::create_dir_all(root.join("plugins")).unwrap();
    let settings = json!({"enabledPlugins":{"example@market":true},
        "hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"printf NATIVE_HOOK"}]}]}});
    fs::write(root.join("settings.json"), serde_json::to_vec(&settings).unwrap()).unwrap();
    fs::write(root.join("plugins/installed_plugins.json"),
        r#"{"version":2,"plugins":{"example@market":[{"scope":"user","installPath":"/another-computer/plugin"}]}}"#).unwrap();
    let marketplaces = root.join("plugins/known_marketplaces.json");
    let mut catalogue = json!({"market":{"source":{"source":"github","repo":"example/plugins"},
        "installLocation":"/another-computer/market", "lastUpdated":"2026-10-17T04:00:18.085Z"}});
    fs::write(&marketplaces, serde_json::to_vec(&catalogue).unwrap()).unwrap();
    let definition = claude_definition(&root, &["--permission-mode", "dontAsk", "--disallowedTools", "WebSearch"]);
    let (policy, _) = project_remote_permissions(&parent, &definition, &mapping, &cwd, &env,
        Some(SpawnPosture::Development), &protected).unwrap();
    assert_eq!(policy["permission_mode"], "dontAsk");
    assert_eq!(policy["disallowed_tools"], json!(["WebFetch", "WebSearch"]));
    assert_eq!(policy["launch_context"]["config_dir"], json!(root));
    // Exercise the final entry point used by the real wrapper.
    recheck_context(&policy["launch_context"], &definition, &cwd, &env).unwrap();
    assert!(policy["launch_context"].get("source_equivalence").is_none());
    assert!(policy["launch_context"]["permission_sources"].as_array().unwrap().iter()
        .all(|s| s["path"].as_str().unwrap() != "/another-computer/plugin"));
    let mut args = definition.args.clone();
    apply_child_arguments("claude_stream_json", &policy, &mut args).unwrap();
    assert!(!args.iter().any(|a| a == "--permission-prompts"));
    assert!(args.windows(2).any(|a| a == ["--permission-mode", "dontAsk"]));
    assert!(!args.iter().any(|a| a == "--plugin-dir" || a == "--settings" || a == "--setting-sources"));
    recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected, Some(&mapping), Some(&parent)).unwrap();
    catalogue["market"]["lastUpdated"] = json!("2026-10-17T04:05:35.002Z");
    fs::write(&marketplaces, serde_json::to_vec(&catalogue).unwrap()).unwrap();
    recheck_context(&policy["launch_context"], &definition, &cwd, &env).unwrap();
    recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected, Some(&mapping), Some(&parent)).unwrap();
    catalogue["market"]["source"]["repo"] = json!("foreign/plugins");
    fs::write(&marketplaces, serde_json::to_vec(&catalogue).unwrap()).unwrap();
    assert_eq!(recheck_context(&policy["launch_context"], &definition, &cwd, &env).unwrap_err(), "settings_revision_changed");
    catalogue["market"]["source"]["repo"] = json!("example/plugins");
    catalogue["market"]["installLocation"] = json!("/foreign/market");
    fs::write(&marketplaces, serde_json::to_vec(&catalogue).unwrap()).unwrap();
    assert_eq!(recheck_context(&policy["launch_context"], &definition, &cwd, &env).unwrap_err(), "settings_revision_changed");
    catalogue["market"]["installLocation"] = json!("/another-computer/market");
    fs::write(&marketplaces, serde_json::to_vec(&catalogue).unwrap()).unwrap();
    fs::write(root.join("settings.json"), "{}").unwrap();
    assert_eq!(recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected,
        Some(&mapping), Some(&parent)).unwrap_err(), "settings_revision_changed");
    fs::write(root.join("settings.json"), serde_json::to_vec(&settings).unwrap()).unwrap();
    let (readonly, posture) = project_remote_permissions(&parent, &definition, &mapping, &cwd, &env,
        Some(SpawnPosture::Discovery), &protected).unwrap();
    assert_eq!(posture, SpawnPosture::Discovery);
    assert_eq!(readonly["permission_mode"], "plan");
    assert_eq!(readonly["tools"], json!(["Read", "Glob", "Grep"]));
    assert!(readonly["disallowed_tools"].as_array().unwrap().contains(&json!("Bash")));
    fs::write(root.join("settings.json"), serde_json::to_vec(&json!({
        "permissions":{"additionalDirectories":[protected[0]]}})).unwrap()).unwrap();
    assert_eq!(project_remote_permissions(&parent, &definition, &mapping, &cwd, &env,
        None, &protected).unwrap_err(), "machine_path_protected");
    fs::write(root.join("settings.json"), serde_json::to_vec(&settings).unwrap()).unwrap();
    let launcher = root.join("bin/my-provider-launcher");
    fs::write(&launcher, "#!/bin/sh\nexec /bin/echo \"$@\"\n").unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let mut custom = definition.clone(); custom.command = launcher.to_string_lossy().into_owned();
    let (custom_policy, _) = project_remote_permissions(&parent, &custom, &mapping, &cwd, &env,
        Some(SpawnPosture::Development), &protected).unwrap();
    assert_eq!(custom_policy["launch_context"]["cli_path"], json!(launcher));
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(project_remote_permissions(&parent, &custom, &mapping, &cwd, &env,
        Some(SpawnPosture::Development), &protected).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn incident171_native_t3_tools_without_transplant_profile_preserve_rules_and_recheck() {
    let (root, env, mapping, protected) = remote162_fixture("native-t3-tools");
    let mut parent = claude_fact(&root, &mapping.remote_root, &env,
        "bypassPermissions", "t3_runtime", "allow", true);
    parent.cwd = "/origin162/project/src".into();
    parent.provider_policy["allowed_tools"] = json!(["mcp__t3-code__*"]);
    parent.provider_policy["disallowed_tools"] = json!(["WebFetch"]);
    assert!(parent.provider_policy["launch_context"].get("source_equivalence").is_none());
    assert!(parent.provider_policy["launch_context"].get("settings_overrides").is_none());
    let original = parent.clone();
    let mut definition = claude_definition(&root, &[]);
    definition.mcp.interactive = "claude".into();
    let cwd = mapping.remote_root.join("src");
    let project = |p: &ProviderPermissions, d: &AgentDefinition| project_remote_permissions(
        p, d, &mapping, &cwd, &env, Some(SpawnPosture::Development), &protected);
    let (policy, posture) = project(&parent, &definition).unwrap();
    assert_eq!(posture, SpawnPosture::Development);
    // Destination grants are native; the local T3 auto-approval is not transplanted.
    assert!(policy.get("allowed_tools").is_none());
    assert_eq!(policy["disallowed_tools"], parent.provider_policy["disallowed_tools"]);
    assert_eq!(parent, original);
    recheck_remote_child_permissions(&policy, &definition, &cwd, &env,
        &protected, Some(&mapping), Some(&parent)).unwrap();
    let mut bad = parent.clone();
    bad.provider_policy["allowed_tools"] = json!(["mcp__other__*"]);
    assert_eq!(project(&bad, &definition).unwrap_err(), "permission_mapping_unavailable");
    bad = parent.clone(); bad.provider_policy["permission_callback"]["kind"] = json!("native_wrapper");
    assert_eq!(project(&bad, &definition).unwrap_err(), "permission_attestation_unavailable");
    bad = parent.clone(); bad.provider_policy["launch_context"]["settings_overrides"] = json!({"permissions":{"allow":["Bash(*)"]}});
    assert_eq!(project(&bad, &definition).unwrap_err(), "permission_mapping_unavailable");
    let mut unsafe_definition = definition.clone(); unsafe_definition.mcp.interactive = "none".into();
    assert_eq!(project(&parent, &unsafe_definition).unwrap_err(), "permission_mapping_unavailable");
    unsafe_definition = definition.clone(); unsafe_definition.args.push("--mcp-config=foreign.json".into());
    assert_eq!(project(&parent, &unsafe_definition).unwrap_err(), "permission_mapping_unavailable");
    fs::write(root.join("settings.json"), "{}").unwrap();
    assert!(recheck_remote_child_permissions(&policy, &definition, &cwd, &env,
        &protected, Some(&mapping), Some(&parent)).is_err());
}

#[test]
fn incident171_native_sdk_parent_fact_without_launch_context_preserves_auth_and_target_checks() {
    let (root, env, mut mapping, protected) = remote162_fixture("native-parent-fact");
    let cwd = mapping.remote_root.join("src");
    mapping.source_root = mapping.remote_root.clone();
    let mut parent = claude_fact(&root, &cwd, &env, "bypassPermissions", "t3_runtime", "allow", true);
    parent.runtime_mode = "full-access".into();
    parent.provider_policy["allowed_tools"] = json!(["mcp__t3-code__*"]);
    parent.provider_policy["additional_directories"] = json!([mapping.source_root]);
    parent.provider_policy["allow_dangerously_skip_permissions"] = json!(true);
    parent.provider_policy.as_object_mut().unwrap().remove("launch_context");
    let original = parent.clone();
    let proof = prove_parent_permissions(&parent, parent.revision).unwrap();
    assert_eq!(proof, prove_parent_permissions(&parent, parent.revision).unwrap());
    assert_eq!(parent, original);
    assert_eq!(prove_parent_permissions(&parent, parent.revision + 1).unwrap_err(), "permission_revision_changed");
    let mut forged = parent.clone(); forged.provider_policy["permission_callback"]["kind"] = json!("native_wrapper");
    assert_eq!(prove_parent_permissions(&forged, forged.revision).unwrap_err(), "permission_attestation_unavailable");
    forged.source = "native_wrapper".into();
    assert_eq!(prove_parent_permissions(&forged, forged.revision).unwrap_err(), "permission_source_unavailable");
    let mut definition = claude_definition(&root, &[]); definition.mcp.interactive = "claude".into();
    let (policy, _) = project_remote_permissions(&parent, &definition, &mapping, &cwd, &env,
        Some(SpawnPosture::Development), &protected).unwrap();
    recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected,
        Some(&mapping), Some(&parent)).unwrap();
    assert!(policy["launch_context"]["permission_sources"].is_array());
    let observed = claude_fact(&root, &cwd, &env, "bypassPermissions", "t3_runtime", "allow", true);
    prove_parent_permissions(&observed, observed.revision).unwrap();
    fs::write(root.join("settings.json"), "{}").unwrap();
    assert!(prove_parent_permissions(&observed, observed.revision).is_err());
    assert!(recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected,
        Some(&mapping), Some(&parent)).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec214_native_t3_codex_projection_preserves_ceiling_and_refuses_other_rules() {
    let (root, env, mapping, protected) = remote162_fixture("native-t3-codex214");
    let cwd = mapping.remote_root.join("src");
    let mut parent = claude_fact(&root, &cwd, &env, "bypassPermissions", "t3_runtime", "allow", true);
    parent.cwd = "/origin162/project/src".into();
    parent.runtime_mode = "full-access".into();
    parent.provider_policy["tools"] = json!({"type":"preset","preset":"claude_code"});
    parent.provider_policy["allowed_tools"] = json!(["mcp__t3-code__*"]);
    parent.provider_policy["settings_sources"] = json!("provider_default");
    parent.provider_policy.as_object_mut().unwrap().remove("launch_context");
    let original = parent.clone();
    let mut definition = codex_definition();
    definition.mcp.interactive = "codex".into();
    let project = |p: &ProviderPermissions, d: &AgentDefinition, posture|
        project_remote_permissions(p, d, &mapping, &cwd, &env, Some(posture), &protected);
    let (policy, posture) = project(&parent, &definition, SpawnPosture::Development).unwrap();
    assert_eq!(posture, SpawnPosture::Development);
    assert_eq!(policy, json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}));
    assert_eq!(parent, original);
    recheck_remote_child_permissions(&policy, &definition, &cwd, &env, &protected, Some(&mapping), Some(&parent)).unwrap();
    let (policy, posture) = project(&parent, &definition, SpawnPosture::Discovery).unwrap();
    assert_eq!(posture, SpawnPosture::Discovery);
    assert_eq!(policy["sandbox_policy"],json!({"type":"readOnly","networkAccess":false}));
    // Aucune règle autre que l’approbation T3 exacte n’est effacée.
    for value in [json!(["Bash"]),json!(["mcp__other__*"]),json!(["mcp__t3-code__*","Bash"])] {
        let mut bad=parent.clone();bad.provider_policy["allowed_tools"]=value;
        assert!(project(&bad,&definition,SpawnPosture::Development).is_err());
    }
    for (key,value) in [
        ("disallowed_tools",json!(["WebFetch"])),
        ("tools",json!(["Read","Glob","Grep"])),
        ("permission_mode",json!("plan")),
        ("settings_sources",json!("user")),
        ("launch_context",json!({"hooks":{"PreToolUse":[]}})),
        ("permission_callback",json!({"kind":"native_wrapper","tool_approval":"allow","plan_exit":"deny"})),
    ] {
        let mut bad=parent.clone();bad.provider_policy[key]=value;
        assert!(project(&bad,&definition,SpawnPosture::Development).is_err(),"{key}");
    }
    let mut bad=parent.clone();bad.source="native_wrapper".into();
    assert!(project(&bad,&definition,SpawnPosture::Development).is_err());
    bad=parent.clone();bad.runtime_mode="approval-required".into();
    assert!(project(&bad,&definition,SpawnPosture::Development).is_err());
    let mut unmapped=parent.clone();unmapped.cwd="/outside214".into();
    assert!(project(&unmapped,&definition,SpawnPosture::Development).is_err());
    let mut unmounted=definition.clone();unmounted.mcp.interactive="none".into();
    assert!(project(&parent,&unmounted,SpawnPosture::Development).is_err());
    // Sans origine attestée, l’exception ne s’applique jamais.
    assert_eq!(child_policy(&parent,&definition,&cwd,&env,Some(SpawnPosture::Development)).unwrap_err(),"permission_attestation_unavailable");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec241_command_execution_follows_effective_profile_without_widening_rights() {
    let read = json!({"kind":"claude","permission_mode":"plan",
        "tools":["Read","Glob","Grep"],"disallowed_tools":["Bash","Edit","Write"]});
    let before=read.clone();
    assert!(!permits_command_execution(&read,false));
    assert_eq!(read,before);
    let mut shell = read.clone();
    shell["tools"]=json!({"type":"preset","preset":"claude_code"});
    assert!(!permits_command_execution(&shell,false));
    shell["disallowed_tools"]=json!([]);
    assert!(permits_command_execution(&shell,false));
    shell["disallowed_tools"]=json!(["Bash(*)"]);
    assert!(!permits_command_execution(&shell,false));
    assert!(permits_command_execution(&json!({"kind":"codex","sandbox_policy":{"type":"readOnly"}}),false));
    assert!(!permits_command_execution(&json!({"kind":"codex","sandbox_policy":{"type":"readOnly"}}),true));
    assert!(!permits_command_execution(&json!({"kind":"unknown"}),false));
}
