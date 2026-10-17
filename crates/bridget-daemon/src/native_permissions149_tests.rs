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
    assert!(
        prove_parent_permissions(&changed, changed.revision).is_err(),
        "règle opaque non exportable"
    );
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
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--permission-prompts" && pair[1] == "none")
    );
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
        format!(
            r#"{{"hooks":{{"PreToolUse":[{{"hooks":[{{"type":"command","command":"${{CLAUDE_PROJECT_DIR}}/hook.sh"}}]}}]}}}}"#
        ),
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
    assert_eq!(
        claude_inputs_without_rules(&context).unwrap(),
        true,
        "sources toutes absentes"
    );
    // Settings sans règles de droits : classé sans règles.
    fs::write(root.join("settings.json"), br#"{"model":"fixture"}"#).unwrap();
    context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert_eq!(claude_inputs_without_rules(&context).unwrap(), true);
    // Settings avec une règle de droits : classé avec règles.
    fs::write(
        root.join("settings.json"),
        br#"{"permissions":{"deny":["Bash"]}}"#,
    )
    .unwrap();
    context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert_eq!(claude_inputs_without_rules(&context).unwrap(), false);
    // Overrides : avec règles même si les sources sont vides.
    fs::remove_file(root.join("settings.json")).unwrap();
    let mut overrides = capture_context_with_env(&definition, &root, &env).unwrap();
    overrides["settings_overrides"] = json!({"permissions":{"defaultMode":"default"}});
    assert_eq!(claude_inputs_without_rules(&overrides).unwrap(), false);
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
    cli_context["permission_sources"][0]["revision"] = json!(
        bridget_transport::protocol::permission_source_revision(
            &root.join("cli-settings.json"),
            16 * 1024 * 1024
        )
        .unwrap()
    );
    assert_eq!(claude_inputs_without_rules(&cli_context).unwrap(), true);
}

/// 171 structural inheritance fixture: real files and a fake CLI binary only.
/// No Claude process, model, Git mutation or proof of native hook application.
fn native171_remote_fixture(
    label: &str,
) -> (
    PathBuf,
    ProviderPermissions,
    AgentDefinition,
    SourceEnvironment,
    crate::registry::ProjectMapping,
    Vec<PathBuf>,
) {
    let (root, mut env) = fixture_root(label);
    let origin = root.join("origin");
    let target = root.join("target");
    let a = root.join("profile-a");
    let b = root.join("profile-b");
    for path in [&origin, &target, &a, &b] {
        fs::create_dir_all(path).unwrap();
    }
    for profile in [&a, &b] {
        fs::set_permissions(profile, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let project_hooks = json!({"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{
        "type":"command","command":"\"$CLAUDE_PROJECT_DIR\"/guard.sh avant","timeout":10}]}]}});
    let user_hooks = json!({"hooks":{"Stop":[{"hooks":[{
        "type":"command","command":"\"$CLAUDE_PROJECT_DIR\"/guard.sh","timeout":10}]}]}});
    for path in [&origin, &target] {
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::create_dir_all(path.join(".claude")).unwrap();
        fs::write(
            path.join(".claude/settings.json"),
            serde_json::to_vec(&project_hooks).unwrap(),
        )
        .unwrap();
        fs::write(path.join("guard.sh"), b"#!/bin/sh\nexit 2\n").unwrap();
        fs::set_permissions(path.join("guard.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    for path in [&a, &b] {
        fs::write(
            path.join("settings.json"),
            serde_json::to_vec(&user_hooks).unwrap(),
        )
        .unwrap();
    }
    env.insert("CLAUDE_CONFIG_DIR".into(), a.as_os_str().to_owned());
    let mut parent = claude_fact(
        &a,
        &origin,
        &env,
        "bypassPermissions",
        "native_wrapper",
        "allow",
        false,
    );
    parent.provider_policy["disallowed_tools"] = json!(["Write"]);
    assert!(
        parent.provider_policy["launch_context"]
            .get("source_equivalence")
            .is_some()
    );
    env.insert("CLAUDE_CONFIG_DIR".into(), b.as_os_str().to_owned());
    let mapping = crate::registry::ProjectMapping {
        additional_root: None,
        source_host: "origin171".into(),
        source_root: origin,
        remote_root: target,
        expected_git_identity: "fixture171-git-identity".into(),
        mapping_revision: 7,
    };
    let private = root.join("worker-private");
    fs::create_dir(&private).unwrap();
    (
        root,
        parent,
        claude_definition(&b, &[]),
        env,
        mapping,
        vec![private],
    )
}

#[test]
fn native171_remote_sources_project_and_user_bound_before_and_after_recapture() {
    let (_root, parent, mut definition, env, mapping, protected) =
        native171_remote_fixture("remote171-positive");
    assert_eq!(
        prove_parent_permissions(&parent, parent.revision)
            .unwrap()
            .len(),
        64
    );
    let (mut policy, posture) = project_remote_permissions(
        &parent,
        &definition,
        &mapping,
        &mapping.remote_root,
        &env,
        None,
        &protected,
    )
    .unwrap();
    assert_eq!(posture, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], "bypassPermissions");
    assert_eq!(policy["disallowed_tools"], json!(["Write"]));
    assert_eq!(
        policy["launch_context"]["hook_execution"],
        "native_remote_equivalent"
    );
    let witness = &policy["launch_context"]["source_equivalence"];
    assert_eq!(witness["binding"]["parent_revision"], parent.revision);
    assert_eq!(witness["binding"]["mapping_revision"], 7);
    let origin_rows = witness["origin"]["rows"].as_array().unwrap();
    assert!(
        origin_rows
            .iter()
            .any(|r| r["kind"] == "user" && r["revision"] != "absent")
    );
    assert!(
        origin_rows
            .iter()
            .any(|r| r["kind"] == "project" && r["revision"] != "absent")
    );
    let pairs = witness["pairs"].as_array().unwrap();
    for i in 0..origin_rows.len() {
        assert_eq!(
            pairs
                .iter()
                .filter(|p| p["origin"].as_u64() == Some(i as u64))
                .count(),
            1
        );
    }
    apply_child_arguments("claude_stream_json", &policy, &mut definition.args).unwrap();
    refresh_remote_permissions_context(
        &mut policy,
        &definition,
        &mapping.remote_root,
        &env,
        &mapping,
    )
    .unwrap();
    assert_eq!(
        policy["launch_context"]["hook_execution"],
        "native_remote_equivalent"
    );
    recheck_remote_child_permissions(
        &policy,
        &definition,
        &mapping.remote_root,
        &env,
        &protected,
        Some(&mapping),
        Some(&parent),
    )
    .unwrap();
}

#[test]
fn native171_remote_witness_omission_partial_source_and_local_marker_are_refused() {
    let (_root, mut parent, definition, env, mapping, protected) =
        native171_remote_fixture("remote171-coverage");
    let good = parent.clone();
    parent.provider_policy["launch_context"]
        .as_object_mut()
        .unwrap()
        .remove("source_equivalence");
    assert!(
        project_remote_permissions(
            &parent,
            &definition,
            &mapping,
            &mapping.remote_root,
            &env,
            None,
            &protected
        )
        .is_err()
    );
    parent = good.clone();
    parent.provider_policy["launch_context"]["source_equivalence"]["manifest"]["rows"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(
        project_remote_permissions(
            &parent,
            &definition,
            &mapping,
            &mapping.remote_root,
            &env,
            None,
            &protected
        )
        .is_err()
    );
    let (mut policy, _) = project_remote_permissions(
        &good,
        &definition,
        &mapping,
        &mapping.remote_root,
        &env,
        None,
        &protected,
    )
    .unwrap();
    policy["launch_context"]["hook_execution"] = json!("native_same_host");
    assert!(
        recheck_remote_child_permissions(
            &policy,
            &definition,
            &mapping.remote_root,
            &env,
            &protected,
            Some(&mapping),
            Some(&good)
        )
        .is_err()
    );
}

#[test]
fn native171_remote_recheck_rejects_mapping_parent_revision_and_mode_or_deny_change() {
    let (_root, parent, definition, env, mapping, protected) =
        native171_remote_fixture("remote171-binding");
    let (policy, _) = project_remote_permissions(
        &parent,
        &definition,
        &mapping,
        &mapping.remote_root,
        &env,
        None,
        &protected,
    )
    .unwrap();
    let mut other_mapping = mapping.clone();
    other_mapping.mapping_revision += 1;
    assert!(
        recheck_remote_child_permissions(
            &policy,
            &definition,
            &mapping.remote_root,
            &env,
            &protected,
            Some(&other_mapping),
            Some(&parent)
        )
        .is_err()
    );
    let mut other_parent = parent.clone();
    other_parent.revision += 1;
    assert!(
        recheck_remote_child_permissions(
            &policy,
            &definition,
            &mapping.remote_root,
            &env,
            &protected,
            Some(&mapping),
            Some(&other_parent)
        )
        .is_err()
    );
    for field in ["permission_mode", "disallowed_tools"] {
        let mut other = policy.clone();
        other[field] = if field == "permission_mode" {
            json!("auto")
        } else {
            json!([])
        };
        assert!(
            recheck_remote_child_permissions(
                &other,
                &definition,
                &mapping.remote_root,
                &env,
                &protected,
                Some(&mapping),
                Some(&parent)
            )
            .is_err()
        );
    }
}

#[test]
fn native171_remote_script_dependency_and_profile_mutations_are_not_ignored() {
    let (_root, parent, definition, env, mapping, protected) =
        native171_remote_fixture("remote171-mutation");
    let (policy, _) = project_remote_permissions(
        &parent,
        &definition,
        &mapping,
        &mapping.remote_root,
        &env,
        None,
        &protected,
    )
    .unwrap();
    fs::set_permissions(
        mapping.remote_root.join("guard.sh"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    assert!(
        recheck_remote_child_permissions(
            &policy,
            &definition,
            &mapping.remote_root,
            &env,
            &protected,
            Some(&mapping),
            Some(&parent)
        )
        .is_err()
    );
    fs::set_permissions(
        mapping.remote_root.join("guard.sh"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    fs::write(
        mapping.remote_root.join("guard.sh"),
        b"#!/bin/sh\nuname -s\nexit 2\n",
    )
    .unwrap();
    let local = capture_context(&definition, &mapping.remote_root, &env).unwrap();
    assert_eq!(local["hook_execution"], "native_same_host");
    assert!(
        local.get("source_equivalence").is_none(),
        "unknown dependencies do not erase local hooks"
    );
    assert!(
        project_remote_permissions(
            &parent,
            &definition,
            &mapping,
            &mapping.remote_root,
            &env,
            None,
            &protected
        )
        .is_err()
    );
}

#[test]
fn native171_remote_user_hook_not_ported_and_script_bytes_changed_are_refused() {
    let (root, parent, definition, env, mapping, protected) =
        native171_remote_fixture("remote171-user-and-bytes");
    let (valid, _) = project_remote_permissions(
        &parent,
        &definition,
        &mapping,
        &mapping.remote_root,
        &env,
        None,
        &protected,
    )
    .expect("projection valide avant retrait du hook utilisateur");
    assert_eq!(
        valid["launch_context"]["hook_execution"],
        "native_remote_equivalent"
    );
    fs::write(root.join("profile-b/settings.json"), b"{}").unwrap();
    assert_eq!(
        project_remote_permissions(
            &parent,
            &definition,
            &mapping,
            &mapping.remote_root,
            &env,
            None,
            &protected
        )
        .unwrap_err(),
        "permission_mapping_unavailable"
    );
    fs::copy(
        root.join("profile-a/settings.json"),
        root.join("profile-b/settings.json"),
    )
    .unwrap();
    let (restored, _) = project_remote_permissions(
        &parent,
        &definition,
        &mapping,
        &mapping.remote_root,
        &env,
        None,
        &protected,
    )
    .expect("projection valide avant changement des octets du script");
    assert_eq!(
        restored["launch_context"]["hook_execution"],
        "native_remote_equivalent"
    );
    fs::write(mapping.remote_root.join("guard.sh"), b"#!/bin/sh\nexit 0\n").unwrap();
    assert_eq!(
        project_remote_permissions(
            &parent,
            &definition,
            &mapping,
            &mapping.remote_root,
            &env,
            None,
            &protected
        )
        .unwrap_err(),
        "permission_mapping_unavailable"
    );
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

/// One affected case over the complete actual captured profile. No Claude,
/// model, business command, SDK import or admission is started by this case.
#[cfg(target_os = "linux")]
#[test]
fn native171_real_captured_profile_closed_origin_target() {
    use bridget_transport::protocol::ProviderPermissions;
    use sha2::{Digest, Sha256};
    let root = PathBuf::from(std::env::var("BRIDGET_REAL_HOOK_PROFILE171").expect("private profile root required"));
    let descriptor: Value = serde_json::from_slice(&fs::read(root.join("descriptor171.json")).unwrap()).unwrap();
    let origin: Value = serde_json::from_slice(&fs::read(root.join("origin-context171.json")).unwrap()).unwrap();
    let manifest: Value = serde_json::from_slice(&fs::read(descriptor["target_path"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(manifest["origin_sources"].as_array().unwrap().len(), 74);
    assert_eq!(manifest["entries"].as_array().unwrap().len(), 17);
    assert_eq!(manifest["entries"].as_array().unwrap().iter().filter(|e| e["origin"]["hook"]["type"] == "prompt").count(), 2);
    assert_eq!(manifest["hosts"]["target"]["versions"]["sdk"], "0.1.68");
    assert_eq!(manifest["target_controls"]["plugins"].as_array().unwrap().len(), 4);
    assert_eq!(manifest["target_entries"].as_array().unwrap().len(), 3);
    assert!(manifest["target_controls"]["plugins"].as_array().unwrap().iter().all(|p| !p["source_files"].as_array().unwrap().is_empty()));
    let cwd = PathBuf::from(manifest["mapping"]["remote_root"].as_str().unwrap());
    let mapping = crate::registry::ProjectMapping {
        additional_root: None,
        source_host: manifest["mapping"]["source_host"].as_str().unwrap().into(),
        source_root: PathBuf::from(manifest["mapping"]["source_root"].as_str().unwrap()),
        remote_root: cwd.clone(),
        expected_git_identity: manifest["mapping"]["source_identity"].as_str().unwrap().into(),
        mapping_revision: 1,
    };
    let mut provider_policy = claude_policy("bypassPermissions", "t3_runtime", "allow", true);
    provider_policy["launch_context"] = origin;
    let parent = ProviderPermissions {
        version:1, source:"provider_turn".into(), run_id:"run-real-profile171-fixture".into(),
        provider_session_id:"session-real-profile171-fixture".into(), provider_instance_id:"instance-real-profile171-fixture".into(),
        revision:1, driver:"claude_stream_json".into(), cwd:mapping.source_root.to_string_lossy().into_owned(),
        runtime_mode:"full-access".into(), interaction_mode:"default".into(), provider_policy,
    };
    let mut definition = claude_definition(Path::new("/home/moi/.claude"), &[]);
    definition.claude_config_dir = None;
    let source = SourceEnvironment::from([
        ("HOME".into(), "/home/moi".into()),
        ("PATH".into(), "/home/moi/.local/bin:/home/moi/.local/lib/bridget-tools/git-2.56.0/bin:/usr/bin:/bin".into()),
    ]);
    let protected = vec![
        PathBuf::from("/home/moi/.config/bridget/worker-cartae.credential"),
        PathBuf::from("/home/moi/.config/bridget"),
        PathBuf::from("/home/moi/.local/share/bridget-worker/cartae"),
    ];
    let nominal_started = std::time::Instant::now();
    let mut operation = PermissionReadOperation::default();
    let (mut policy, posture) = project_remote_permissions_in_operation(&parent, &definition, &mapping, &cwd, &source,
        Some(SpawnPosture::Development), &protected, &mut operation).expect("complete real profile projection");
    assert_eq!(posture, SpawnPosture::Development);
    assert_eq!(policy["launch_context"]["hook_execution"], "native_remote_guarded");
    assert_eq!(policy["launch_context"]["source_equivalence"]["profile"], descriptor);
    assert_eq!(policy["permission_mode"], parent.provider_policy["permission_mode"]);
    assert_eq!(policy["tools"], parent.provider_policy["tools"]);
    let mut scoped = definition.clone();
    apply_child_arguments_in_operation(&scoped.protocol, &policy, &mut scoped.args, &mut operation).unwrap();
    assert_eq!(scoped.args.iter().filter(|a| a.as_str() == "--settings").count(), 1);
    assert_eq!(scoped.args.iter().filter(|a| a.as_str() == "--plugin-dir").count(), 4);
    refresh_remote_permissions_context_in_operation(&mut policy, &scoped, &cwd, &source, &mapping, &mut operation).unwrap();
    recheck_remote_child_permissions_in_operation(&policy, &scoped, &cwd, &source, &protected, Some(&mapping), Some(&parent), &mut operation).unwrap();
    operation.finish().unwrap();
    let nominal_ms = nominal_started.elapsed().as_millis();
    fs::write(root.join("nominal-cost171.json"), serde_json::to_vec(&json!({
        "nominal_projection_arguments_refresh_recheck_ms": nominal_ms,
        "includes_fresh_operation_finish": true,
        "negative_scenarios_included": false
    })).unwrap()).unwrap();
    println!("native171_nominal_projection_arguments_refresh_recheck_ms={nominal_ms}");
    // Binding substitution cannot acquire another parent/mapping identity.
    let mut other_mapping = mapping.clone(); other_mapping.mapping_revision += 1;
    assert_eq!(recheck_remote_child_permissions(&policy, &scoped, &cwd, &source, &protected, Some(&other_mapping), Some(&parent)),
               Err("permission_hook_equivalence_unavailable".into()));
    // All negative mutations are private copies, never a source/target hook.
    let negative_dir = root.join("negative-profile-case"); fs::create_dir(&negative_dir).unwrap();
    let mut missing = manifest.clone(); missing["entries"].as_array_mut().unwrap().pop();
    let bytes = serde_json::to_vec(&missing).unwrap();
    let negative = negative_dir.join("missing-hook.json"); fs::write(&negative, &bytes).unwrap();
    fs::set_permissions(&negative, fs::Permissions::from_mode(0o600)).unwrap();
    let mut context = policy["launch_context"].clone();
    context["source_equivalence"]["profile"]["target_path"] = json!(negative);
    context["source_equivalence"]["profile"]["revision"] = json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
    assert_eq!(bridget_transport::protocol::recheck_guarded_hook_profile(&context),
               Err("permission_hook_profile_incomplete".into()));
    // No provider process or deployment call occurs in this test.
}

/// One new affected origin recheck case over the actual 74-source profile.
/// No provider, model, admission or runtime profile mutation is performed.
#[cfg(target_os = "macos")]
#[test]
fn native171_origin_v2_recheck_reconstructs_verified_witness_exact_sources() {
    use sha2::{Digest, Sha256};
    let root = PathBuf::from(std::env::var("BRIDGET_REAL_ORIGIN_RECHECK171").expect("private inputs required"));
    let context: Value = serde_json::from_slice(&fs::read(root.join("origin-context171.json")).unwrap()).unwrap();
    let descriptor: Value = serde_json::from_slice(&fs::read(root.join("descriptor171.json")).unwrap()).unwrap();
    let manifest: Value = serde_json::from_slice(&fs::read(descriptor["origin_path"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(descriptor["revision"], "sha256:526d56069ccfa07286a69469676bd7d43c2d9bb7f2aa2bb4946696780cd90e50");
    assert_eq!(context["permission_sources"], manifest["origin_sources"]);
    assert_eq!(context["permission_sources"].as_array().unwrap().len(), 74);
    let cwd = PathBuf::from(manifest["mapping"]["source_root"].as_str().unwrap());
    let mut definition = claude_definition(Path::new(context["config_dir"].as_str().unwrap()), &[]);
    definition.claude_config_dir = None;
    definition.command = context["cli_path"].as_str().unwrap().into();
    let env = SourceEnvironment::from([
        ("HOME".into(), manifest["hosts"]["origin"]["home"].as_str().unwrap().into()),
        ("PATH".into(), "/Users/moi/.local/bin:/opt/homebrew/bin:/usr/bin:/bin".into()),
        ("CLAUDE_CONFIG_DIR".into(), context["config_dir"].as_str().unwrap().into()),
        ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".into(), "1".into()),
    ]);
    // The unchanged native capture is legacy: this is the mechanical mismatch
    // previously compared directly with the valid origin-v2 witness.
    let legacy = capture_context_with_env(&definition, &cwd, &env).unwrap();
    assert_ne!(legacy["source_equivalence"], context["source_equivalence"]);
    assert_eq!(legacy["permission_sources"], context["permission_sources"]);
    let mut operation = PermissionReadOperation::default();
    recheck_context_with_env_in_operation(&context, &definition, &cwd, &env, &mut operation).unwrap();
    let verified = operation.profile(&context).unwrap();
    let environment = json!({"TMUX":null,"TMUX_PANE":null,"CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC":"1"});
    let mut rebuilt = legacy.clone();
    rebuilt["source_equivalence"] = verified.origin_witness_for(&rebuilt, &cwd, &environment).unwrap();
    assert_eq!(rebuilt, context);
    assert_eq!(rebuilt["source_equivalence"]["version"], 2);
    // Same token, same operation: negatives cannot acquire different sources,
    // environment, mapping or witness authority.
    let mut bad_source = context.clone();
    bad_source["permission_sources"][0]["revision"] = json!(format!("sha256:{}", "0".repeat(64)));
    assert_eq!(recheck_context_with_env_in_operation(&bad_source, &definition, &cwd, &env, &mut operation),
        Err("permission_hook_profile_unavailable".into()));
    let mut bad_env = env.clone(); bad_env.insert("TMUX".into(), "unattested171".into());
    assert_eq!(recheck_context_with_env_in_operation(&context, &definition, &cwd, &bad_env, &mut operation),
        Err("permission_hook_environment_unavailable".into()));
    assert_eq!(verified.origin_witness_for(&legacy, &cwd.join("unmapped171"), &environment),
        Err("permission_hook_equivalence_unavailable".into()));
    let mut bad_witness = context.clone(); bad_witness["source_equivalence"]["version"] = json!(999);
    assert_eq!(recheck_context_with_env_in_operation(&bad_witness, &definition, &cwd, &env, &mut operation),
        Err("settings_revision_changed".into()));
    operation.finish().unwrap(); // Fresh complete recheck at the operation boundary.
    // A dependency mutation is only in a private manifest copy; no real file is changed.
    let negatives = root.join("negative-origin-case"); fs::create_dir(&negatives).unwrap();
    let mut bad_profile = manifest.clone();
    let row = bad_profile["files"].as_array_mut().unwrap().iter_mut()
        .find(|f| f["variants"].get("origin").is_some()).unwrap();
    row["variants"]["origin"]["revision"] = json!(format!("sha256:{}", "0".repeat(64)));
    let bytes = serde_json::to_vec(&bad_profile).unwrap();
    let path = negatives.join("dependency-changed.json"); fs::write(&path, &bytes).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let mut dependency = context.clone();
    dependency["source_equivalence"]["profile"]["origin_path"] = json!(path);
    dependency["source_equivalence"]["profile"]["revision"] = json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
    assert_eq!(recheck_context_with_env(&dependency, &definition, &cwd, &env),
        Err("permission_hook_dependency_changed".into()));
    recheck_context_with_env(&legacy, &definition, &cwd, &env).unwrap(); // Legacy behavior unchanged.
}


/// The real T3 run492 has SDK settings, a scoped T3 MCP rule and TWO roots.
/// The SDK port is checked here; the unmapped attachments root stays refused.
#[cfg(target_os = "macos")]
#[test]
fn native171_authenticated_sdk_rules_recheck_preserves_owner_and_unmapped_root() {
    let root = PathBuf::from(std::env::var("BRIDGET_SDK_PERMISSIONS171_INPUTS").expect("private inputs required"));
    let parent: bridget_transport::protocol::ProviderPermissions = serde_json::from_slice(
        &fs::read(root.join("parent-permissions492.json")).unwrap()).unwrap();
    let definition: AgentDefinition = serde_json::from_slice(
        &fs::read(root.join("definition-origin492.json")).unwrap()).unwrap();
    let context = &parent.provider_policy["launch_context"];
    assert_eq!(parent.revision, 6);
    assert_eq!(parent.provider_policy["allowed_tools"], json!(["mcp__t3-code__*"]));
    assert_eq!(context["settings_overrides"], sdk_settings171());
    assert_eq!(context["permission_sources"].as_array().unwrap().len(), 74);
    assert_eq!(parent.provider_policy["additional_directories"], json!([
        "/Users/moi/Nextcloud/10.Scripts/69.opus2D", "/Users/moi/.t3/userdata/attachments"]));
    assert!(!definition.args.iter().any(|arg| arg.starts_with("--settings")));
    let cwd = Path::new(&parent.cwd);
    let source = SourceEnvironment::from([
        ("HOME".into(), "/Users/moi".into()),
        ("PATH".into(), "/Users/moi/.local/bin:/opt/homebrew/bin:/usr/bin:/bin".into()),
    ]);
    let environment = build_environment(&definition, &source).unwrap();
    let recaptured = capture_context_with_env(&definition, cwd, &environment).unwrap();
    assert!(recaptured.get("settings_overrides").is_none());
    assert_eq!(recaptured["permission_sources"], context["permission_sources"]);
    for key in ["cli_path","cli_revision","resolved_cli_path","resolved_cli_revision","config_dir","settings_sources"] {
        assert_eq!(recaptured.get(key), context.get(key), "native field {key}");
    }
    // The actual catalogue port reads the authenticated SDK fact separately
    // from registry args. It does not add the fact to a registry or global file.
    recheck_parent_context171(&parent, &definition, cwd, &source).unwrap();
    let (policy, posture) = child_policy_reader(&parent, &definition, cwd, &source)(Some(SpawnPosture::Development)).unwrap();
    assert_eq!(posture, SpawnPosture::Development);
    assert_eq!(policy["launch_context"], *context);
    assert_eq!(policy["allowed_tools"], parent.provider_policy["allowed_tools"]);
    assert_eq!(policy["additional_directories"], parent.provider_policy["additional_directories"]);
    assert!(portable_claude_rules(&parent));
    let mut local_args = definition.args.clone();
    apply_child_arguments("claude_stream_json", &policy, &mut local_args).unwrap();
    let sdk_arg = local_args.windows(2).find(|p| p[0] == "--settings").unwrap();
    assert_eq!(serde_json::from_str::<Value>(&sdk_arg[1]).unwrap(), sdk_settings171());
    let allowed = local_args.windows(2).find(|p| p[0] == "--allowedTools").unwrap();
    assert_eq!(allowed[1], "mcp__t3-code__*");

    // Every SDK form is closed; no third rule, alternate server, mode, missing
    // owner fact or unauthenticated source can acquire this reconciliation.
    for bad_value in [json!({"permissions":{"allow":["mcp__bridget__*"]}}),
                      json!({"permissions":{"allow":["mcp__bridget__bridget_capabilities","mcp__bridget__bridget_task_status","Bash"]}}),
                      json!({"permissions":{"allow":["mcp__bridget__bridget_capabilities","mcp__bridget__bridget_task_status"],"defaultMode":"bypassPermissions"}})] {
        let mut altered = parent.clone(); altered.provider_policy["launch_context"]["settings_overrides"] = bad_value;
        assert!(AuthenticatedSdkInputs171::from_parent(&altered).is_err());
        assert!(!portable_claude_rules(&altered));
    }
    let mut foreign = parent.clone(); foreign.source = "human_grant".into();
    assert!(AuthenticatedSdkInputs171::from_parent(&foreign).is_err());
    let mut no_owner = parent.clone(); no_owner.provider_session_id.clear();
    assert!(AuthenticatedSdkInputs171::from_parent(&no_owner).is_err());
    let mut prefix = parent.clone(); prefix.provider_policy["allowed_tools"] = json!(["mcp__*"]);
    assert!(AuthenticatedSdkInputs171::from_parent(&prefix).is_err());
    let mut cli_override = definition.clone(); cli_override.args.extend(["--settings".into(), sdk_settings171().to_string()]);
    assert_eq!(recheck_parent_context171(&parent, &cli_override, cwd, &source), Err("settings_revision_changed".into()));
    let mut changed = parent.clone(); changed.provider_policy["launch_context"]["permission_sources"][0]["revision"] = json!(format!("sha256:{}","0".repeat(64)));
    assert!(recheck_parent_context171(&changed, &definition, cwd, &source).is_err());
    let mut wrong_env = parent.clone(); wrong_env.provider_policy["launch_context"]["source_equivalence"]["profile"]["origin_environment"]["TMUX"] = json!("other-owner");
    assert!(recheck_parent_context171(&wrong_env, &definition, cwd, &source).is_err());

    // Exact native target settings copy: keep all hooks, asks, denies and mode.
    let base: Value = serde_json::from_slice(&fs::read(root.join("target-settings171.json")).unwrap()).unwrap();
    let merged = compose_sdk_settings171(&base, &context["settings_overrides"]).unwrap();
    let mut expected = base.clone();
    for rule in sdk_settings171()["permissions"]["allow"].as_array().unwrap() {
        expected["permissions"]["allow"].as_array_mut().unwrap().push(rule.clone());
    }
    assert_eq!(merged, expected);
    assert_eq!(merged["hooks"], base["hooks"]);
    for key in ["ask","deny","defaultMode","disableBypassPermissionsMode","additionalDirectories"] {
        assert_eq!(merged["permissions"].get(key), base["permissions"].get(key));
    }
    let mut target = definition.clone(); target.mcp.interactive = "claude".into();
    require_sdk_native_mount171(context, &target).unwrap();
    let mut uncontrolled = target.clone(); uncontrolled.mcp.interactive = "none".into();
    assert_eq!(require_sdk_native_mount171(context, &uncontrolled), Err("permission_mapping_unavailable".into()));
    let mut server_override = target.clone(); server_override.args.extend(["--mcp-config".into(), "other-server.json".into()]);
    assert_eq!(require_sdk_native_mount171(context, &server_override), Err("permission_mapping_unavailable".into()));
    let mut selected_profile = json!({"settings":{"target":{"path":root.join("target-settings171.json")}}});
    let mut selected_context = context.clone(); selected_context["hook_execution"] = json!("native_remote_guarded");
    let mut selected = target.clone(); selected.args.extend(["--settings".into(), merged.to_string()]);
    validate_guarded_settings_args171(&selected, &selected_profile, &selected_context).unwrap();
    let mut altered = merged.clone(); altered["permissions"]["allow"].as_array_mut().unwrap().push(json!("Bash"));
    *selected.args.last_mut().unwrap() = altered.to_string();
    assert_eq!(validate_guarded_settings_args171(&selected, &selected_profile, &selected_context), Err("settings_revision_changed".into()));
    // Mapping is still exact and single-root. The real second root is retained,
    // and is explicitly refused rather than omitted to make the test nominal.
    let project = Path::new("/Users/moi/Nextcloud/10.Scripts/69.opus2D");
    assert_eq!(mapped_permission_suffix171(project, project).unwrap(), Path::new(""));
    assert_eq!(mapped_permission_suffix171(Path::new("/Users/moi/.t3/userdata/attachments"), project), Err("permission_mapping_unavailable".into()));
    assert_eq!(mapped_permission_suffix171(&project.join("../outside"), project), Err("permission_mapping_unavailable".into()));
    selected_profile["settings"]["target"]["path"] = json!("/unavailable-sdk-settings171");
    assert!(guarded_sdk_settings171(&selected_profile, &selected_context).is_err());
}


/// Mapping-only case. Uses the complete real run492 parent, without replaying
/// its already-passed SDK recheck. Real target directory must be provisioned
/// separately before this test; it never creates the provisioned directory.
#[cfg(target_os = "linux")]
#[test]
fn native171_additional_root_real_parent_closed_mapping_preserves_permissions() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let input = PathBuf::from(std::env::var("BRIDGET_ADDITIONAL_ROOT171_INPUTS").expect("private inputs required"));
    let parent: bridget_transport::protocol::ProviderPermissions = serde_json::from_slice(
        &fs::read(input.join("parent-permissions492.json")).unwrap()).unwrap();
    let encoded: Value = serde_json::from_slice(&fs::read(input.join("mapping171.json")).unwrap()).unwrap();
    let mapping: crate::registry::ProjectMapping = serde_json::from_value(encoded.clone()).unwrap();
    let profile: Value = serde_json::from_slice(&fs::read(input.join("complete-profile171.json")).unwrap()).unwrap();
    let extra = mapping.additional_root.as_ref().unwrap();
    assert_eq!(extra.source_root, Path::new("/Users/moi/.t3/userdata/attachments"));
    assert_eq!(extra.remote_root, Path::new("/home/moi/.local/share/bridget-attachments/opus2d"));
    assert_eq!((extra.source_uid,extra.source_mode,extra.remote_uid,extra.remote_mode),(501,0o755,1002,0o700));
    assert_eq!(extra.access, "inherit");
    assert_eq!(parent.provider_policy["additional_directories"],json!([
        "/Users/moi/Nextcloud/10.Scripts/69.opus2D","/Users/moi/.t3/userdata/attachments"]));
    assert_eq!(parent.provider_policy["launch_context"]["permission_sources"].as_array().unwrap().len(),74);
    let protected = vec![PathBuf::from("/home/moi/.config/bridget/worker-cartae.credential"),
        PathBuf::from("/home/moi/.config/bridget"),PathBuf::from("/home/moi/.local/share/bridget-worker/cartae")];
    crate::registry::validate_additional_mapping171(&mapping).unwrap();
    check_profile_additional_root171(&profile,&mapping).unwrap();
    check_parent_additional_root171(&parent,&profile).unwrap();
    let registry = crate::registry::AgentRegistry::from_json(
        &fs::read_to_string(input.join("registry-target171.json")).unwrap(), input.join("registry-target171.json")).unwrap();
    check_additional_registry_authority171(&profile,&registry).unwrap();
    let mut wrong_authority: Value = serde_json::from_slice(&fs::read(input.join("registry-target171.json")).unwrap()).unwrap();
    wrong_authority["machines"]["cartae"]["mappings"]["opus2d"]["additional_root"]["remote_root"] = json!("/home/moi/unknown-root171");
    let wrong_authority = crate::registry::AgentRegistry::from_json(&wrong_authority.to_string(), input.join("registry-target171.json")).unwrap();
    assert_eq!(check_additional_registry_authority171(&profile,&wrong_authority),Err("permission_hook_equivalence_unavailable".into()));
    verify_permission_directory171(&extra.remote_root,1002,0o700,&protected).unwrap();
    // One nominal pass through the real target projection/recheck boundary.
    // This is the new mapping case, not the old SDK test being replayed.
    let definition: AgentDefinition = serde_json::from_slice(&fs::read(input.join("definition-target171.json")).unwrap()).unwrap();
    let source = SourceEnvironment::from([
        ("HOME".into(), "/home/moi".into()),
        ("PATH".into(), "/home/moi/.local/bin:/usr/local/bin:/usr/bin:/bin".into()),
    ]);
    let mut operation = PermissionReadOperation::default();
    let (actual_policy, actual_posture) = project_remote_permissions_in_operation(&parent,&definition,&mapping,
        &mapping.remote_root,&source,Some(SpawnPosture::Development),&protected,&mut operation).unwrap();
    assert_eq!(actual_posture,SpawnPosture::Development);
    assert_eq!(actual_policy["additional_directories"],json!([
        "/home/moi/10.Scripts/69.opus2D","/home/moi/.local/share/bridget-attachments/opus2d"]));
    for field in ["permission_mode","tools","allowed_tools","disallowed_tools"] {
        assert_eq!(actual_policy.get(field),parent.provider_policy.get(field));
    }
    assert_eq!(actual_policy["launch_context"]["settings_overrides"],parent.provider_policy["launch_context"]["settings_overrides"]);
    let mut scoped = definition.clone();
    apply_child_arguments_in_operation(&scoped.protocol,&actual_policy,&mut scoped.args,&mut operation).unwrap();
    recheck_remote_child_permissions_in_operation(&actual_policy,&scoped,&mapping.remote_root,&source,&protected,
        Some(&mapping),Some(&parent),&mut operation).unwrap();
    operation.finish().unwrap();
    let mut projected = parent.provider_policy.clone();
    project_policy_directories171(&mut projected,&mapping,&protected).unwrap();
    assert_eq!(projected["additional_directories"],json!([
        "/home/moi/10.Scripts/69.opus2D","/home/moi/.local/share/bridget-attachments/opus2d"]));
    let mut expected = parent.provider_policy.clone();
    expected["additional_directories"] = projected["additional_directories"].clone();
    assert_eq!(projected,expected); // No other rule, tool, mode, source or SDK field changed.
    assert_eq!(mapped_permission_directory171(&mapping,Path::new("/Users/moi/.t3/userdata/attachments")).unwrap(),extra.remote_root);
    for outside in ["/Users/moi", "/Users/moi/.t3/userdata", "/Users/moi/.t3/userdata/attachments-other", "/Users/moi/.t3/userdata/attachments/../outside"] {
        assert_eq!(mapped_permission_directory171(&mapping,Path::new(outside)),Err("permission_mapping_unavailable".into()));
    }
    let mut absent = mapping.clone(); absent.additional_root=None;
    let mut refused=parent.provider_policy.clone();
    assert_eq!(project_policy_directories171(&mut refused,&absent,&protected),Err("permission_mapping_unavailable".into()));
    assert_eq!(refused,parent.provider_policy); // No partially translated parent on error.
    let mut overlapping=mapping.clone();overlapping.additional_root.as_mut().unwrap().source_root=mapping.source_root.join("sub");
    assert!(crate::registry::validate_additional_mapping171(&overlapping).is_err());
    let mut overlapping=mapping.clone();overlapping.additional_root.as_mut().unwrap().remote_root=mapping.remote_root.clone();
    assert!(crate::registry::validate_additional_mapping171(&overlapping).is_err());
    let mut traversal=mapping.clone();traversal.additional_root.as_mut().unwrap().source_root=PathBuf::from("/Users/moi/.t3/../attachments");
    assert!(crate::registry::validate_additional_mapping171(&traversal).is_err());
    let mut unknown=encoded.clone();unknown["additional_root"]["extra_permission"]=json!(true);
    assert!(serde_json::from_value::<crate::registry::ProjectMapping>(unknown).is_err());
    let mut unknown=mapping.clone();unknown.additional_root.as_mut().unwrap().access="all".into();
    assert!(crate::registry::validate_additional_mapping171(&unknown).is_err());
    let mut revision=mapping.clone();revision.mapping_revision+=1;
    assert_eq!(check_profile_additional_root171(&profile,&revision),Err("permission_hook_equivalence_unavailable".into()));
    let mut wrong_pair=profile.clone();wrong_pair["mapping"]["additional_root"]["remote_root"]=json!("/home/moi/other");
    assert_eq!(check_profile_additional_root171(&wrong_pair,&mapping),Err("permission_hook_equivalence_unavailable".into()));
    assert_eq!(verify_permission_directory171(&extra.remote_root,1003,0o700,&protected),Err("permission_mapping_unavailable".into()));
    assert_eq!(verify_permission_directory171(&extra.remote_root,1002,0o755,&protected),Err("permission_mapping_unavailable".into()));
    assert_eq!(verify_permission_directory171(&extra.remote_root.join("absent-root171"),1002,0o700,&protected),Err("machine_path_unverifiable".into()));
    let mut blocked=protected.clone();blocked.push(extra.remote_root.clone());
    assert_eq!(verify_permission_directory171(&extra.remote_root,1002,0o700,&blocked),Err("machine_path_protected".into()));
    let link=input.join("negative-root-link");symlink(&extra.remote_root,&link).unwrap();
    assert_eq!(verify_permission_directory171(&link,1002,0o700,&protected),Err("permission_mapping_unavailable".into()));
    fs::remove_file(link).unwrap();
    // No attachment contents are read, listed, copied or synchronized here.
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
