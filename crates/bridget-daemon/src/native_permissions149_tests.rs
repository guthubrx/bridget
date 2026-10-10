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

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

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
    env.insert("PATH".into(), format!("{}:/usr/bin:/bin", bin.display()).into());
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
fn claude_fact(config: &Path, cwd: &Path, env: &SourceEnvironment, mode: &str, callback_kind: &str, tool_approval: &str, t3: bool) -> ProviderPermissions {
    let definition = claude_definition(config, &[]);
    let mut policy = claude_policy(mode, callback_kind, tool_approval, t3);
    policy["launch_context"] = capture_context_with_env(&definition, cwd, env).unwrap();
    ProviderPermissions {
        version: 1,
        source: if t3 { "provider_turn" } else { "native_wrapper" }.into(),
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
    let fact = codex_fact(root.to_str().unwrap(), json!({"type":"dangerFullAccess"}), "never");
    let definition = claude_definition(&root, &[]);
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], json!("bypassPermissions"));
    assert_eq!(policy["tools"], json!({"type":"preset","preset":"claude_code"}));
    assert_eq!(policy["permission_callback"], json!({"kind":"native_wrapper","tool_approval":"allow","plan_exit":"deny"}));
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
        !args.iter().any(|arg| matches!(arg.as_str(), "--yolo" | "--dangerously-skip-permissions" | "--allow-dangerously-skip-permissions")),
        "aucun yolo hérité : {args:?}"
    );
    let position = args.iter().position(|arg| arg == "--permission-mode").unwrap();
    assert_eq!(args[position + 1], "bypassPermissions");
    assert!(args.windows(2).any(|pair| pair[0] == "--permission-prompts" && pair[1] == "none"));
    assert!(args.contains(&"mission-149".to_string()), "les arguments métier restent");
}

#[test]
fn native149_codex_lecteur_refuse_le_developpement_et_reduit_en_decouverte() {
    let (root, env) = fixture_root("codex-readonly");
    let fact = codex_fact(root.to_str().unwrap(), json!({"type":"readOnly","networkAccess":false}), "never");
    let definition = codex_definition();
    // Même famille : demande de développement refusée par nom.
    let erreur = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    // Sans posture : réduction conservatrice en readOnly.
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, None).unwrap();
    assert_eq!(effective, SpawnPosture::Discovery);
    assert_eq!(policy["sandbox_policy"], json!({"type":"readOnly","networkAccess":false}));
}

#[test]
fn native149_decouverte_reduit_et_conserve_les_refus_existant() {
    let (root, env) = fixture_root("discovery");
    let mut policy = claude_policy("default", "native_wrapper", "prompt", false);
    policy["tools"] = json!(["Read", "Glob", "Grep", "Bash", "Edit", "Write"]);
    policy["allowed_tools"] = json!(["Bash"]);
    policy["disallowed_tools"] = json!(["Custom-Tool"]);
    let mut fact = claude_fact(&root, &root, &env, "default", "native_wrapper", "prompt", false);
    policy["launch_context"] = fact.provider_policy["launch_context"].clone();
    fact.provider_policy = policy;
    let definition = claude_definition(&root, &[]);
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Discovery)).unwrap();
    assert_eq!(effective, SpawnPosture::Discovery);
    assert_eq!(policy["permission_mode"], json!("plan"));
    assert_eq!(policy["tools"], json!(["Read", "Glob", "Grep"]));
    let denied: Vec<String> = policy["disallowed_tools"].as_array().unwrap().iter().map(|value| value.as_str().unwrap().to_string()).collect();
    for outil in ["Bash", "Edit", "Write", "NotebookEdit", "WebFetch", "WebSearch", "Custom-Tool"] {
        assert!(denied.contains(&outil.to_string()), "{outil} doit rester refusé : {denied:?}");
    }
    // La règle allow existante reste en dessous de l'inventaire sans jamais
    // réintroduire un outil retiré par la réduction.
    assert_eq!(policy["allowed_tools"], json!(["Bash"]));
    // La demande de développement sur un fait default reste admissible : le
    // parent médie déjà par prompts, l'enfant hérite ce régime.
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], json!("default"));
    assert_eq!(policy["permission_callback"]["kind"], json!("native_wrapper"));
}

#[test]
fn native149_workspace_codex_vers_claude_refuse_le_confinement() {
    let (root, env) = fixture_root("workspace");
    let cwd = root.to_str().unwrap();
    // workspace-write vers Claude : Bridget ne peut pas reproduire le bac à
    // sable du parent → refus nommé, jamais une réduction silencieuse.
    let fact = codex_fact(cwd, json!({"type":"workspaceWrite"}), "never");
    let erreur = child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "provider_confinement_unavailable");
    // externalSandbox même famille : même refus.
    let fact = codex_fact(cwd, json!({"type":"externalSandbox","networkAccess":"enabled"}), "never");
    let erreur = child_policy(&fact, &codex_definition(), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "provider_confinement_unavailable");
    // dangerFullAccess mais approbation non "never" : aucun mapping honnête.
    let fact = codex_fact(cwd, json!({"type":"dangerFullAccess"}), "on-request");
    let erreur = child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // readOnly avec réseau ouvert vers Claude : mapping indisponible.
    let fact = codex_fact(cwd, json!({"type":"readOnly","networkAccess":true}), "never");
    let erreur = child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
}

#[test]
fn native149_claude_meme_famille_exige_contexte_vivant_et_meme_racine() {
    let (root, env) = fixture_root("claude-same");
    let definition = claude_definition(&root, &[]);
    // Sans launch_context : aucune projection possible.
    let sans_context = ProviderPermissions { provider_policy: claude_policy("default", "native_wrapper", "prompt", false), cwd: root.to_string_lossy().into_owned(), ..claude_fact(&root, &root, &env, "default", "native_wrapper", "prompt", false) };
    let erreur = child_policy(&sans_context, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Contexte vivant (capturé à la racine demandée) mais racine parent
    // différente : mapping refusé.
    let (autre_racine, _) = fixture_root("claude-same-autre");
    let mut fact = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
    fact.cwd = autre_racine.to_string_lossy().into_owned();
    let erreur = child_policy(&fact, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Callback T3 avec approbation allow, mode non bypass : un enfant natif ne
    // peut pas s'auto-approuver à la place du parent T3.
    let fact = claude_fact(&root, &root, &env, "default", "t3_runtime", "allow", true);
    let erreur = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Plan hérité ne donne jamais le développement.
    let fact = claude_fact(&root, &root, &env, "plan", "native_wrapper", "prompt", false);
    let erreur = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap_err();
    assert_eq!(erreur, "permission_not_inherited");
    // Bypass hérité avec le même contexte vivant : development complet.
    let fact = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy["permission_mode"], json!("bypassPermissions"));
    assert_eq!(policy["permission_callback"]["kind"], json!("native_wrapper"));
}

#[test]
fn native149_claude_vers_codex_exige_lire_ou_complet_sans_regles() {
    let (root, env) = fixture_root("claude-codex");
    let definition = codex_definition();
    // Bypass parent → dangerFullAccess codex.
    let fact = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
    let (policy, effective) = child_policy(&fact, &definition, &root, &env, Some(SpawnPosture::Development)).unwrap();
    assert_eq!(effective, SpawnPosture::Development);
    assert_eq!(policy, json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}));
    // Plan lecture seule parent → readOnly codex.
    let mut plan = claude_fact(&root, &root, &env, "plan", "native_wrapper", "prompt", false);
    plan.provider_policy["tools"] = json!(["Read", "Glob", "Grep"]);
    let (policy, effective) = child_policy(&plan, &definition, &root, &env, None).unwrap();
    assert_eq!(effective, SpawnPosture::Discovery);
    assert_eq!(policy["sandbox_policy"], json!({"type":"readOnly","networkAccess":false}));
    // acceptEdits : ni complet ni lecture seule → mapping refusé.
    let fact = claude_fact(&root, &root, &env, "acceptEdits", "native_wrapper", "allow", false);
    let erreur = child_policy(&fact, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Overrides de droits présents : le parent porte des règles → refus.
    let mut overrides = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
    overrides.provider_policy["launch_context"]["settings_overrides"] = json!({"permissions":{"defaultMode":"default"}});
    let erreur = child_policy(&overrides, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Refus explicites parent : le mapping codex ne les repasse pas.
    let mut deny = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
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
    let mut allowlist = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
    allowlist.provider_policy["allowed_tools"] = json!(["Read", "Grep"]);
    let erreur = child_policy(&allowlist, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // F3 : inventaire vide en mode plan — jamais un readOnly codex déduit
    // d'un parent sans aucun outil.
    let mut vide = claude_fact(&root, &root, &env, "plan", "native_wrapper", "prompt", false);
    vide.provider_policy["tools"] = json!([]);
    let erreur = child_policy(&vide, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Inventaire vide en bypass : ni complet (aucun Bash) ni lecture seule.
    let mut vide_bypass = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
    vide_bypass.provider_policy["tools"] = json!([]);
    let erreur = child_policy(&vide_bypass, &definition, &root, &env, None).unwrap_err();
    assert_eq!(erreur, "permission_mapping_unavailable");
    // Témoin : le complet valide sans règles reste dangerFullAccess.
    let temoin = claude_fact(&root, &root, &env, "bypassPermissions", "native_wrapper", "allow", false);
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
    opaque.insert("PATH".into(), format!("{}:/usr/bin:/bin", bin.display()).into());
    let definition = claude_definition(&root, &[]);
    let mut shell = definition.clone();
    shell.command = "opaque-cli".into();
    let erreur = capture_context_with_env(&shell, &root, &opaque).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // F5 : un lanceur binaire (non script) différent du `claude` du PATH ne
    // prouve pas qu'il transmet les drapeaux ; seule l'empreinte épinglée le ferait.
    fs::copy("/bin/echo", bin.join("wrapper-binaire")).unwrap();
    fs::set_permissions(bin.join("wrapper-binaire"), fs::Permissions::from_mode(0o755)).unwrap();
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
    let erreur = capture_context_with_env(&claude_definition(&root, &["--plugin-dir", "/fixture"]), &root, &env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --settings inline non JSON : refus.
    let erreur = capture_context_with_env(&claude_definition(&root, &["--settings", "{invalide"]), &root, &env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --settings inline avec clé hors permissions : refus.
    let erreur = capture_context_with_env(&claude_definition(&root, &["--settings", r#"{"model":1}"#]), &root, &env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // --setting-sources hors vocabulaire : refus.
    let erreur = capture_context_with_env(&claude_definition(&root, &["--setting-sources", "git"]), &root, &env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // enabledPlugins sans manifeste installé : source opaque.
    fs::write(root.join("settings.json"), br#"{"enabledPlugins":{"x":true}}"#).unwrap();
    let erreur = capture_context_with_env(&definition, &root, &env).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    fs::remove_file(root.join("settings.json")).unwrap();
    // Hooks dans le settings utilisateur : source non observable.
    let (autre, _) = fixture_root("capture-hooks");
    let mut positif = env.clone();
    positif.insert("CLAUDE_CONFIG_DIR".into(), autre.as_os_str().to_owned());
    fs::write(autre.join("settings.json"), br#"{"hooks":{"PreToolUse":[]}}"#).unwrap();
    let erreur = capture_context_with_env(&claude_definition(&autre, &[]), &root, &positif).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Cas positifs : inline permissions conservé, sources sélectionnées.
    fs::remove_file(autre.join("settings.json")).unwrap();
    let context = capture_context_with_env(&claude_definition(&autre, &["--settings", r#"{"permissions":{"defaultMode":"plan"}}"#, "--setting-sources", "user,project"]), &root, &positif).unwrap();
    assert_eq!(context["settings_overrides"], json!({"permissions":{"defaultMode":"plan"}}));
    assert_eq!(context["settings_sources"], json!(["user", "project"]));
    // Les sources `managed` sont celles de l'hôte (MDM / ClaudeCode système) ; seules celles de la fixture sont absentes.
    assert!(context["permission_sources"].as_array().unwrap().iter().filter(|source| source["kind"] != "managed").all(|source| source["revision"] == "absent"), "aucune source utilisateur/projet n'existe dans la fixture");
}

#[test]
fn native149_recheck_detecte_la_mutation_du_settings_utilisateur() {
    let (root, env) = fixture_root("recheck");
    fs::write(root.join("settings.json"), br#"{"model":"fixture"}"#).unwrap();
    let definition = claude_definition(&root, &[]);
    let context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert_eq!(recheck_context_with_env(&context, &definition, &root, &env), Ok(()));
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
    faux["permission_sources"][0]["revision"] = json!("sha256:0000000000000000000000000000000000000000000000000000000000000000");
    let erreur = recheck_context_with_env(&faux, &definition, &root, &env).unwrap_err();
    assert_eq!(erreur, "settings_revision_changed");
}

#[test]
fn native149_snapshot_fige_et_deduplique_les_contextes() {
    let (root, env) = fixture_root("snapshot");
    let fact = codex_fact(root.to_str().unwrap(), json!({"type":"dangerFullAccess"}), "never");
    let (policy, _) = child_policy(&fact, &claude_definition(&root, &[]), &root, &env, None).unwrap();
    let fige = snapshot("agent-149", "instance-1", fact.clone(), policy.clone(), &root);
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
    let deux = snapshot("agent-149", "instance-1", parent_claude.clone(), policy.clone(), &root);
    assert_eq!(snapshot_contexts(&deux).len(), 2);
    // Le même contexte parent et enfant : une seule entrée.
    let mut même = policy.clone();
    même["launch_context"] = parent_claude.provider_policy["launch_context"].clone();
    let un = snapshot("a", "i", parent_claude, même, &root);
    assert_eq!(snapshot_contexts(&un).len(), 1);
}

#[test]
fn native149_mode_runtime_suit_la_politique_reelle() {
    assert_eq!(runtime_mode(&json!({"permission_mode":"plan"})), "approval-required");
    assert_eq!(runtime_mode(&json!({"permission_mode":"default"})), "approval-required");
    assert_eq!(runtime_mode(&json!({"permission_mode":"acceptEdits"})), "auto-accept-edits");
    assert_eq!(runtime_mode(&json!({"permission_mode":"bypassPermissions"})), "full-access");
    assert_eq!(runtime_mode(&json!({"permission_mode":"auto"})), "auto");
    assert_eq!(runtime_mode(&json!({"permission_mode":"dontAsk"})), "auto");
    assert_eq!(runtime_mode(&json!({"kind":"codex","sandbox_policy":{"type":"dangerFullAccess"},"approval_policy":"never"})), "full-access");
    assert_eq!(runtime_mode(&json!({"kind":"codex","sandbox_policy":{"type":"readOnly"},"approval_policy":"never"})), "auto");
    assert_eq!(runtime_mode(&json!({"kind":"codex","sandbox_policy":{"type":"readOnly"},"approval_policy":"on-request"})), "approval-required");
}

struct MockSession {
    perms: Option<(String, Value, u64, Option<String>)>,
}

impl bridget_transport::transport::Transport for MockSession {
    fn deliver(&mut self, _message: &bridget_core::BridgetMessage) -> Result<(), bridget_transport::transport::TransportError> {
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
    let session = MockSession { perms: Some(("sess-149".into(), claude_policy("default", "native_wrapper", "prompt", false), 3, Some(root.to_string_lossy().into_owned()))) };
    let fact = observed_fact("instance-1", &definition, Some(&context), &session).unwrap().unwrap();
    assert_eq!(fact.provider_session_id, "sess-149");
    assert_eq!(fact.revision, 3);
    assert_eq!(fact.driver, "claude_stream_json");
    assert_eq!(fact.source, "native_wrapper");
    assert_eq!(fact.provider_instance_id, "instance-1");
    assert_eq!(fact.runtime_mode, "approval-required");
    assert_eq!(fact.interaction_mode, "default");
    assert_eq!(fact.run_id, "sess-149", "sans identité de tour, la session porte le fait");
    assert_eq!(fact.provider_policy["launch_context"], context);
    assert_eq!(PathBuf::from(&fact.cwd), root);
    assert_eq!(bridget_transport::protocol::validate_permissions(&fact, false), Ok(()));
    // Sans contexte publié : refus nommé.
    let erreur = observed_fact("instance-1", &definition, None, &session).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Codex sans cwd fournisseur : aucun fait inventé.
    let codex = codex_definition();
    let session_sans_cwd = MockSession { perms: Some(("sess-codex".into(), json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}), 1, None)) };
    let erreur = observed_fact("instance-1", &codex, None, &session_sans_cwd).unwrap_err();
    assert_eq!(erreur, "permission_attestation_unavailable");
    // Aucune permission annoncée : aucun fait, jamais un repli.
    let muet = MockSession { perms: None };
    assert_eq!(observed_fact("instance-1", &codex, None, &muet).unwrap(), None);
}

#[test]
fn native149_entrees_claude_sans_regles_sont_classees() {
    let (root, env) = fixture_root("inputs");
    let definition = claude_definition(&root, &[]);
    let mut context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert_eq!(claude_inputs_without_rules(&context).unwrap(), true, "sources toutes absentes");
    // Settings sans règles de droits : classé sans règles.
    fs::write(root.join("settings.json"), br#"{"model":"fixture"}"#).unwrap();
    context = capture_context_with_env(&definition, &root, &env).unwrap();
    assert_eq!(claude_inputs_without_rules(&context).unwrap(), true);
    // Settings avec une règle de droits : classé avec règles.
    fs::write(root.join("settings.json"), br#"{"permissions":{"deny":["Bash"]}}"#).unwrap();
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
    cli_context["permission_sources"][0]["path"] = json!(root.join("cli-settings.json").to_string_lossy().to_string());
    cli_context["permission_sources"][0]["kind"] = json!("cli");
    cli_context["permission_sources"][0]["revision"] = json!(bridget_transport::protocol::permission_source_revision(&root.join("cli-settings.json"), 16 * 1024 * 1024).unwrap());
    assert_eq!(claude_inputs_without_rules(&cli_context).unwrap(), true);
}
