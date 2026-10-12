//! Session 149 — volet G-P : formes fermées des faits de permissions et
//! rechecks des entrées figées (S149-29/S149-30, oracles G-P, contrats
//! `specs/149-sous-agents-lineage/contracts/permissions.md`).
//!
//! Aucun réseau, aucun modèle, aucun processus fournisseur : tout repose sur
//! des fixtures fichiers sous une racine temporaire privée du test.

use bridget_transport::protocol::{
    claude_launch_policy_options, permission_source_revision, recheck_frozen_inputs,
    recheck_permission_context_sources, ProviderPermissions,
};
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

fn fixture_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bridget-np149-{}-{}-{}",
        label,
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn codex_fact(sandbox: Value, approval: &str) -> ProviderPermissions {
    ProviderPermissions {
        version: 1,
        source: "native_wrapper".into(),
        run_id: "run-149".into(),
        provider_session_id: "sess-149".into(),
        provider_instance_id: "inst-149".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: "/private/tmp".into(),
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

fn claude_fact(mode: &str, callback_kind: &str, tool_approval: &str, t3: bool) -> ProviderPermissions {
    ProviderPermissions {
        version: 1,
        source: if t3 { "provider_turn" } else { "native_wrapper" }.into(),
        run_id: "run-149".into(),
        provider_session_id: "sess-149".into(),
        provider_instance_id: "inst-149".into(),
        revision: 1,
        driver: "claude_stream_json".into(),
        cwd: "/private/tmp".into(),
        runtime_mode: "approval-required".into(),
        interaction_mode: "default".into(),
        provider_policy: json!({
            "kind":"claude",
            "permission_mode":mode,
            "tools":{"type":"preset","preset":"claude_code"},
            "permission_callback":{"kind":callback_kind,"tool_approval":tool_approval,"plan_exit":"deny"},
            "settings_sources":"provider_default",
        }),
    }
}

const SHA: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn valid_codex() -> ProviderPermissions {
    codex_fact(json!({"type":"dangerFullAccess"}), "never")
}

fn valid_claude(t3: bool) -> ProviderPermissions {
    claude_fact("default", if t3 { "t3_runtime" } else { "native_wrapper" }, "prompt", t3)
}

#[test]
fn native149_formes_fermees_codex_accepte_quatre_sandbox_et_refuse_le_reste() {
    assert_eq!(bridget_transport::protocol::validate_permissions(&valid_codex(), false), Ok(()));
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"readOnly","networkAccess":false}), "untrusted"), false),
        Ok(())
    );
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"workspaceWrite"}), "on-request"), false),
        Ok(())
    );
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"externalSandbox","networkAccess":"restricted"}), "never"), false),
        Ok(())
    );
    // Sandbox inconnu : aucune porte de sortie ouverte.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"yoloSandbox"}), "never"), false),
        Err("permission_attestation_unavailable".into())
    );
    // networkAccess non booléen sur readOnly.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"readOnly","networkAccess":"oui"}), "never"), false),
        Err("permission_attestation_unavailable".into())
    );
    // writableRoots avec chemin relatif.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"workspaceWrite","writableRoots":["relatif"]}), "never"), false),
        Err("permission_attestation_unavailable".into())
    );
    // externalSandbox avec valeur réseau hors vocabulaire.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&codex_fact(json!({"type":"externalSandbox","networkAccess":"wild"}), "never"), false),
        Err("permission_attestation_unavailable".into())
    );
    // Clé supplémentaire dans la politique : forme fermée du contenu aussi.
    let mut extra = valid_codex();
    extra.provider_policy["extra"] = json!(1);
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&extra, false),
        Err("permission_attestation_unavailable".into())
    );
    // Approbation granulaire : booléens exigés sur toutes les valeurs.
    let granular = codex_fact(
        json!({"type":"dangerFullAccess"}),
        "granular-placeholder",
    );
    let mut fact = granular.clone();
    fact.provider_policy["approval_policy"] = json!({"granular":{"mcp_elicitations":true,"rules":false,"sandbox_approval":true}});
    assert_eq!(bridget_transport::protocol::validate_permissions(&fact, false), Ok(()));
    fact.provider_policy["approval_policy"]["granular"]["rules"] = json!("oui");
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Relecteur inconnu.
    let mut fact = valid_codex();
    fact.provider_policy["approvals_reviewer"] = json!("robot");
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
}

#[test]
fn native149_formes_fermees_claude_et_callback_correle_au_canal() {
    assert_eq!(bridget_transport::protocol::validate_permissions(&valid_claude(false), false), Ok(()));
    assert_eq!(bridget_transport::protocol::validate_permissions(&valid_claude(true), true), Ok(()));
    // Le callback doit suivre le canal réel : jamais l'inverse.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&valid_claude(false), true),
        Err("permission_attestation_unavailable".into())
    );
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&valid_claude(true), false),
        Err("permission_attestation_unavailable".into())
    );
    // Mode inconnu.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&claude_fact("yolo", "native_wrapper", "prompt", false), false),
        Err("permission_attestation_unavailable".into())
    );
    // Preset autre que claude_code.
    let mut fact = valid_claude(false);
    fact.provider_policy["tools"] = json!({"type":"preset","preset":"autre"});
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // sources de settings autres que provider_default.
    let mut fact = valid_claude(false);
    fact.provider_policy["settings_sources"] = json!(["user"]);
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // allow_dangerously_skip_permissions non booléen.
    let mut fact = valid_claude(false);
    fact.provider_policy["allow_dangerously_skip_permissions"] = json!("oui");
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Sortie de plan autre que deny.
    let mut fact = valid_claude(false);
    fact.provider_policy["permission_callback"]["plan_exit"] = json!("allow");
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // tool_approval hors vocabulaire.
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&claude_fact("default", "native_wrapper", "peut-etre", false), false),
        Err("permission_attestation_unavailable".into())
    );
}

#[test]
fn native149_bornes_identite_revolution_et_taille_du_fait() {
    let mut fact = valid_codex();
    fact.revision = 0;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    let mut fact = valid_codex();
    fact.version = 2;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Le canal pilote la source admise.
    let mut fact = valid_codex();
    fact.source = "provider_turn".into();
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // run_id vide et trop long.
    for run_id in ["", &"a".repeat(257)] {
        let mut fact = valid_codex();
        fact.run_id = run_id.into();
        assert_eq!(
            bridget_transport::protocol::validate_permissions(&fact, false),
            Err("permission_attestation_unavailable".into())
        );
    }
    // cwd relatif.
    let mut fact = valid_codex();
    fact.cwd = "tmp/relatif".into();
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // runtime_mode hors vocabulaire.
    let mut fact = valid_codex();
    fact.runtime_mode = "yolo".into();
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Fait >64KiB : 128 outils de 512 caractères débordent la borne.
    let mut fact = valid_claude(false);
    fact.provider_policy["tools"] = json!((0..128).map(|i| format!("{:0<512}{i}", "o")).collect::<Vec<_>>());
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
}

#[test]
fn native149_contexte_lancement_compose_digests_sources_et_overrides() {
    let root = fixture_root("launch-context");
    let cli = root.join("cli");
    fs::write(&cli, b"binaire-fixture-149").unwrap();
    let cli_revision = permission_source_revision(&cli, 512 * 1024 * 1024).unwrap();
    let context = json!({
        "cli_path":cli,
        "cli_revision":cli_revision,
        "resolved_cli_path":cli,
        "resolved_cli_revision":cli_revision,
        "config_dir":root,
        "settings_sources":"provider_default",
        "permission_sources":[{"kind":"user","path":root.join("settings.json"),"revision":"absent"}],
    });
    let mut fact = valid_claude(false);
    fact.provider_policy["launch_context"] = context.clone();
    assert_eq!(bridget_transport::protocol::validate_permissions(&fact, false), Ok(()));

    // Digest en majuscules refusé.
    let mut cassé = context.clone();
    cassé["cli_revision"] = json!(format!("SHA256:{}", "0".repeat(64)));
    fact.provider_policy["launch_context"] = cassé;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Digest trop court.
    let mut court = context.clone();
    court["resolved_cli_revision"] = json!("sha256:00");
    fact.provider_policy["launch_context"] = court;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Chemins relatifs refusés.
    let mut relatif = context.clone();
    relatif["cli_path"] = json!("relatif/cli");
    fact.provider_policy["launch_context"] = relatif;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Sources dupliquées refusées.
    let mut dupliquées = context.clone();
    dupliquées["settings_sources"] = json!(["user", "user"]);
    fact.provider_policy["launch_context"] = dupliquées;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Source hors vocabulaire.
    let mut git = context.clone();
    git["settings_sources"] = json!(["git"]);
    fact.provider_policy["launch_context"] = git;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Trop de permission_sources : fermé.
    let mut trop = context.clone();
    trop["permission_sources"] = json!((0..33).map(|i| json!({"kind":"cli","path":format!("/private/tmp/{i}"),"revision":SHA})).collect::<Vec<_>>());
    fact.provider_policy["launch_context"] = trop;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );
    // Révision non digest dans une source.
    let mut mauvaise = context.clone();
    mauvaise["permission_sources"][0]["revision"] = json!("sha256:ZZ");
    fact.provider_policy["launch_context"] = mauvaise;
    assert_eq!(
        bridget_transport::protocol::validate_permissions(&fact, false),
        Err("permission_attestation_unavailable".into())
    );

    // Overrides : seules les permissions sont admises, formes fermées.
    // allow/ask/deny portent des règles atomiques (chaînes ≤ 512), jamais des
    // chemins : seul additional_directories exige l'absolu.
    for (overrides, attendu) in [
        (json!({"permissions":{"defaultMode":"plan"}}), Ok(())),
        (json!({"permissions":{"disableBypassPermissionsMode":"disable"}}), Ok(())),
        (json!({"permissions":{"allow":["Bash(ls:*)"]}}), Ok(())),
        (json!({"autre":1}), Err("permission_source_unavailable".into())),
        (json!({"permissions":{"defaultMode":"superMode"}}), Err("permission_source_unavailable".into())),
        (json!({"permissions":{"disableBypassPermissionsMode":"off"}}), Err("permission_source_unavailable".into())),
        (json!({"permissions":{"allow":[42]}}), Err("permission_source_unavailable".into())),
    ] {
        let mut essai = context.clone();
        essai["settings_overrides"] = overrides.clone();
        fact.provider_policy["launch_context"] = essai;
        assert_eq!(
            bridget_transport::protocol::validate_permissions(&fact, false),
            attendu.map_err(|code: &str| code.to_string()),
            "overrides {overrides:?}"
        );
    }
}

#[test]
fn native149_revision_de_source_fichier_dossier_symlink_et_absent() {
    let root = fixture_root("revision");
    let fichier = root.join("fichier.json");
    fs::write(&fichier, b"contenu-a").unwrap();
    let revision = permission_source_revision(&fichier, 16 * 1024 * 1024).unwrap();
    assert!(revision.starts_with("sha256:") && revision.len() == 71, "{revision}");
    assert_eq!(
        revision,
        permission_source_revision(&fichier, 16 * 1024 * 1024).unwrap(),
        "contenu stable, révision stable"
    );
    fs::write(&fichier, b"contenu-b").unwrap();
    assert_ne!(
        revision,
        permission_source_revision(&fichier, 16 * 1024 * 1024).unwrap(),
        "contenu mué, révision muée"
    );
    // Fichier absent : révision "absent", jamais une erreur.
    assert_eq!(
        permission_source_revision(&root.join("absent.json"), 16 * 1024 * 1024).unwrap(),
        "absent"
    );
    // Symlink : source opaque, refus nommé.
    let lien = root.join("lien.json");
    std::os::unix::fs::symlink(&fichier, &lien).unwrap();
    assert_eq!(
        permission_source_revision(&lien, 16 * 1024 * 1024),
        Err("permission_source_unavailable".into())
    );
    // Dossier : révision triée, indépendante de l'ordre de création.
    let (dossier_a, dossier_b) = (root.join("a-dir"), root.join("b-dir"));
    fs::create_dir(&dossier_a).unwrap();
    fs::create_dir(&dossier_b).unwrap();
    fs::write(dossier_a.join("a.txt"), b"x").unwrap();
    fs::write(dossier_a.join("b.txt"), b"y").unwrap();
    fs::write(dossier_b.join("b.txt"), b"y").unwrap();
    fs::write(dossier_b.join("a.txt"), b"x").unwrap();
    assert_eq!(
        permission_source_revision(&dossier_a, 16 * 1024 * 1024).unwrap(),
        permission_source_revision(&dossier_b, 16 * 1024 * 1024).unwrap(),
        "contenu identique, ordre de création différent : même révision"
    );
    fs::write(dossier_a.join("c.txt"), b"z").unwrap();
    assert_ne!(
        permission_source_revision(&dossier_a, 16 * 1024 * 1024).unwrap(),
        permission_source_revision(&dossier_b, 16 * 1024 * 1024).unwrap(),
        "un fichier ajouté change la révision du dossier"
    );
    // Un sous-dossier rend la source inobservable.
    fs::create_dir(dossier_b.join("sous-dossier")).unwrap();
    assert_eq!(
        permission_source_revision(&dossier_b, 16 * 1024 * 1024),
        Err("permission_source_unavailable".into())
    );
}

#[test]
fn native149_options_lancement_cli_resolvent_la_politique_ferme() {
    let mut policy = json!({"kind":"claude","permission_mode":"default"});
    claude_launch_policy_options(
        &[
            "--tools".to_string(),
            "Read,Glob,Grep".to_string(),
            "--allowedTools=Bash,Edit".to_string(),
            "--add-dir".to_string(),
            "/private/tmp".to_string(),
            "--allow-dangerously-skip-permissions".to_string(),
        ],
        &mut policy,
    )
    .unwrap();
    assert_eq!(policy["tools"], json!(["Read", "Glob", "Grep"]));
    assert_eq!(policy["allowed_tools"], json!(["Bash", "Edit"]));
    assert_eq!(policy["additional_directories"], json!(["/private/tmp"]));
    assert_eq!(policy["allow_dangerously_skip_permissions"], json!(true));

    // Le vocabulaire "default" devient le preset fournisseur.
    let mut policy = json!({});
    claude_launch_policy_options(&["--tools".to_string(), "default".to_string()], &mut policy).unwrap();
    assert_eq!(policy["tools"], json!({"type":"preset","preset":"claude_code"}));

    // Borne des 128 valeurs.
    let mut policy = json!({});
    let erreur = claude_launch_policy_options(
        &["--allowedTools".to_string(), (0..129).map(|i| format!("outil{i}")).collect::<Vec<_>>().join(",")],
        &mut policy,
    )
    .unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");

    // Répertoire relatif refusé.
    let mut policy = json!({});
    let erreur = claude_launch_policy_options(&["--add-dir".to_string(), "relatif".to_string()], &mut policy).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");

    // Aucune option : politique inchangée.
    let mut policy = json!({"kind":"claude"});
    claude_launch_policy_options(&["-p".to_string(), "mission".to_string()], &mut policy).unwrap();
    assert_eq!(policy, json!({"kind":"claude"}));
}

#[test]
fn native149_recheck_des_sources_detecte_mutation_ajout_et_suppression() {
    let root = fixture_root("recheck-context");
    let cli = root.join("cli");
    fs::write(&cli, b"cli-149").unwrap();
    let settings = root.join("settings.json");
    fs::write(&settings, br#"{"model":"fixture-149"}"#).unwrap();
    let cli_revision = permission_source_revision(&cli, 512 * 1024 * 1024).unwrap();
    let settings_revision = permission_source_revision(&settings, 16 * 1024 * 1024).unwrap();
    let context = json!({
        "cli_path":cli,
        "cli_revision":cli_revision,
        "resolved_cli_path":cli,
        "resolved_cli_revision":cli_revision,
        "permission_sources":[{"kind":"user","path":settings,"revision":settings_revision}],
    });
    assert_eq!(recheck_permission_context_sources(&context), Ok(()));

    // Mutation : refus nommé.
    fs::write(&settings, br#"{"model":"mute-149"}"#).unwrap();
    assert_eq!(
        recheck_permission_context_sources(&context),
        Err("settings_revision_changed".into())
    );
    // Suppression : même refus nommé.
    fs::remove_file(&settings).unwrap();
    assert_eq!(
        recheck_permission_context_sources(&context),
        Err("settings_revision_changed".into())
    );
    // Ajout là où la révision était "absent" : la source devient observable,
    // le recheck doit refuser.
    let absent = json!({
        "cli_path":cli,
        "cli_revision":cli_revision,
        "resolved_cli_path":cli,
        "resolved_cli_revision":cli_revision,
        "permission_sources":[{"kind":"user","path":settings,"revision":"absent"}],
    });
    assert_eq!(recheck_permission_context_sources(&absent), Ok(()));
    fs::write(&settings, br#"{"model":"ajoute-149"}"#).unwrap();
    assert_eq!(
        recheck_permission_context_sources(&absent),
        Err("settings_revision_changed".into())
    );
    // CLI remplacé : refus nommé.
    let mut cli_mué = context.clone();
    fs::write(&cli, b"cli-remplace-149").unwrap();
    cli_mué["permission_sources"] = json!([]);
    assert_eq!(
        recheck_permission_context_sources(&cli_mué),
        Err("settings_revision_changed".into())
    );
}

#[test]
fn native149_recheck_gele_resout_le_launcher_par_path_et_detecte_le_changement() {
    let root = fixture_root("recheck-frozen");
    // Priorité du PATH passé en entrée : le launcher se résout dans le
    // répertoire de la fixture, jamais ailleurs.
    let bin = root.join("bin");
    fs::create_dir(&bin).unwrap();
    let faux_sh = bin.join("sh");
    fs::write(&faux_sh, b"#!/bin/sh\n").unwrap();
    fs::set_permissions(&faux_sh, fs::Permissions::from_mode(0o755)).unwrap();
    let vrai_sh = PathBuf::from("/bin/sh");
    let revision = permission_source_revision(&vrai_sh, 512 * 1024 * 1024).unwrap();
    let mut context = json!({
        "cli_path":vrai_sh,
        "cli_revision":revision,
        "resolved_cli_path":vrai_sh,
        "resolved_cli_revision":revision,
        "permission_sources":[],
    });
    // Le recheck gelé court-circuite sans policy enfant : on gèle un contexte
    // cohérent avec /bin/sh pour que la résolution par PATH soit réellement
    // exercée. Le guard nettoie l'environnement même si un assert panique.
    struct Garde;
    impl Drop for Garde {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var("BRIDGET_NATIVE_PERMISSION_SOURCES");
                std::env::remove_var("BRIDGET_NATIVE_CHILD_POLICY");
            }
        }
    }
    let _garde = Garde;
    unsafe { std::env::set_var("BRIDGET_NATIVE_CHILD_POLICY", json!({"launch_context": context}).to_string()) };
    assert_eq!(recheck_frozen_inputs("sh", &[("PATH".into(), "/bin".into())]), Ok(()));
    // Le PATH de fixture l'emporte : le launcher résolu n'est plus /bin/sh.
    let path_fixture = [("PATH".into(), format!("{}", bin.display()))];
    assert_eq!(
        recheck_frozen_inputs("sh", &path_fixture),
        Err("settings_revision_changed".into())
    );
    // Commande multi-composants non absolue : refus nommé, jamais résolue.
    assert_eq!(
        recheck_frozen_inputs("./sh", &[]),
        Err("settings_revision_changed".into())
    );
    // Contexte cohérent avec le PATH de fixture : passe.
    let faux_revision = permission_source_revision(&faux_sh, 512 * 1024 * 1024).unwrap();
    context["cli_path"] = json!(faux_sh.canonicalize().unwrap().to_string_lossy().to_string());
    context["cli_revision"] = json!(faux_revision);
    context["resolved_cli_path"] = context["cli_path"].clone();
    context["resolved_cli_revision"] = json!(faux_revision);
    context["permission_sources"] = json!([]);
    unsafe { std::env::set_var("BRIDGET_NATIVE_CHILD_POLICY", json!({"launch_context": context}).to_string()) };
    assert_eq!(recheck_frozen_inputs("sh", &path_fixture), Ok(()));
    unsafe { std::env::remove_var("BRIDGET_NATIVE_CHILD_POLICY") };
    // Sources d'environnement non JSON : refus d'attestation, jamais un succès.
    unsafe { std::env::set_var("BRIDGET_NATIVE_PERMISSION_SOURCES", "pas-du-json") };
    assert_eq!(
        recheck_frozen_inputs("sh", &[]),
        Err("permission_attestation_unavailable".into())
    );
    unsafe { std::env::remove_var("BRIDGET_NATIVE_PERMISSION_SOURCES") };
}
