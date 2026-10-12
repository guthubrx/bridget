//! T006/T008 : droits figés149 et chemins uniquement cités, jamais autorisants.
use super::*;
use bridget_transport::protocol::{
    ChildProviderRequest, ChildProviderRequestId, ChildRequestDecision, ChildRequestKind,
    ChildRequestVerdict,
};

#[test]
fn native155_rights_table_verdicts_et_limites_absolues() {
    let parent = codex_fact(
        "/bridget-fixture155",
        json!({"type":"dangerFullAccess"}),
        "on-request",
    );
    let frozen = snapshot(
        "parent155",
        "instance-1",
        parent.clone(),
        parent.provider_policy.clone(),
        Path::new("/bridget-fixture155"),
    );
    let base = ChildProviderRequest {
        version: 1,
        provider_session: "child-session155".into(),
        turn_id: "child-turn155".into(),
        provider_request_id: ChildProviderRequestId::Number(155),
        kind: ChildRequestKind::ApprovalCommand,
        details: json!({
            "method":"item/commandExecution/requestApproval", "kind":"command", "duration":"once",
            "effects":{"command":true,"network":false,"file_write":false},
            "complete":true, "secret":false, "command":"printf fixture", "paths_cited":[]
        }),
        offered: vec![
            ChildRequestDecision::Accept,
            ChildRequestDecision::Decline,
            ChildRequestDecision::Cancel,
        ],
        question_ids: vec![],
        truncated: false,
    };
    // Attentes indépendantes du code futur. La table couvre aussi les gardes
    // T002 ; ApprovalFile ici est une preuve de politique synthétique complète,
    // pas une affirmation de support du requestApproval fileChange réel.
    let cases = json!([
        {"label":"question valide","kind":"user_input","expected":"parent"},
        {"label":"parent absent ou révoqué","kind":"user_input","parent_available":false,"expected":"forbidden","code":"parent_unavailable"},
        {"label":"question à secret","kind":"user_input","details":{"secret":true,"questions":[{"id":"q1","header":"Secret","question":"Valeur privée","isSecret":true}]},"expected":"forbidden","code":"secret_not_forwardable"},
        {"label":"formulaire tiers typé non vide","kind":"elicitation_form","details":{"mode":"form","requestedSchema":{"type":"object","properties":{"q":{"type":"string"}}}},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"formulaire tiers vide","kind":"elicitation_form","details":{"mode":"form","requestedSchema":{"type":"object","properties":{}}},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"mode enrichi openai/form","kind":"elicitation_form","details":{"mode":"openai/form"},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"mode enrichi openaiForm","kind":"elicitation_form","details":{"mode":"openaiForm"},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"meta approval non vide","kind":"elicitation_form","details":{"mode":"form","_meta":{"approval":true}},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"meta effet non établi","kind":"elicitation_form","details":{"mode":"form","_meta":{"futureEffect":true}},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"mode URL","kind":"elicitation_form","details":{"mode":"url"},"expected":"forbidden","code":"url_mode_unsupported"},
        {"label":"méthode permissions","details":{"method":"item/permissions/requestApproval"},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"méthode inconnue","details":{"method":"item/future/requestApproval"},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"writeStdin hors périmètre","details":{"kind":"writeStdin"},"expected":"forbidden","code":"approval_method_unsupported"},
        {"label":"détails tronqués","truncated":true,"expected":"forbidden","code":"details_incomplete"},
        {"label":"détails incomplets","details":{"complete":false},"expected":"forbidden","code":"details_incomplete"},
        {"label":"champ effet inconnu","details":{"effects":{"command":true,"network":false,"file_write":false,"futureGrant":true}},"expected":"forbidden","code":"details_incomplete"},
        {"label":"enfant readOnly","child_sandbox":{"type":"readOnly","networkAccess":false},"expected":"forbidden","code":"child_restricted"},
        {"label":"parent readOnly","parent_sandbox":{"type":"readOnly","networkAccess":false},"expected":"forbidden","code":"parent_limit_unknown"},
        {"label":"parent externalSandbox","parent_sandbox":{"type":"externalSandbox","networkAccess":"enabled"},"expected":"forbidden","code":"parent_limit_unknown"},
        {"label":"politique parent incomplète","remove_parent_field":"approval_policy","expected":"forbidden","code":"parent_limit_unknown"},
        {"label":"reviewer auto_review","parent_reviewer":"auto_review","expected":"forbidden","code":"parent_limit_unknown"},
        {"label":"reviewer guardian_subagent","parent_reviewer":"guardian_subagent","expected":"forbidden","code":"parent_limit_unknown"},
        {"label":"écriture workspaceWrite","kind":"approval_file","parent_sandbox":{"type":"workspaceWrite","networkAccess":true},"expected":"forbidden","code":"file_confinement_unproven"},
        {"label":"écriture synthétique plein accès","kind":"approval_file","expected":"human"},
        {"label":"réseau workspaceWrite accordé","parent_sandbox":{"type":"workspaceWrite","networkAccess":true},"details":{"effects":{"command":false,"network":true,"file_write":false}},"expected":"human"},
        {"label":"réseau workspaceWrite interdit malgré humain","parent_sandbox":{"type":"workspaceWrite","networkAccess":false},"details":{"effects":{"command":false,"network":true,"file_write":false}},"expected":"forbidden","code":"network_not_granted"},
        {"label":"réseau absent donc non accordé","parent_sandbox":{"type":"workspaceWrite"},"details":{"effects":{"command":false,"network":true,"file_write":false}},"expected":"forbidden","code":"network_not_granted"},
        {"label":"commande plein accès","expected":"human"},
        {"label":"commande workspaceWrite malgré humain","parent_sandbox":{"type":"workspaceWrite","networkAccess":true},"expected":"forbidden","code":"command_unbounded"},
        {"label":"effets mixtes écriture commande réseau","parent_sandbox":{"type":"workspaceWrite","networkAccess":true},"details":{"effects":{"command":true,"network":true,"file_write":true}},"expected":"forbidden","code":"file_confinement_unproven"},
        {"label":"durée session","details":{"duration":"session"},"expected":"forbidden","code":"duration_unsupported"}
    ]);
    assert!(cases.as_array().unwrap().len() >= 24);
    for case in cases.as_array().unwrap() {
        let mut policy = frozen.clone();
        if let Some(sandbox) = case.get("parent_sandbox") {
            policy.parent.provider_policy["sandbox_policy"] = sandbox.clone();
        }
        if let Some(sandbox) = case.get("child_sandbox") {
            policy.child_policy["sandbox_policy"] = sandbox.clone();
        }
        if let Some(field) = case.get("remove_parent_field").and_then(Value::as_str) {
            policy
                .parent
                .provider_policy
                .as_object_mut()
                .unwrap()
                .remove(field);
        }
        if let Some(reviewer) = case.get("parent_reviewer") {
            policy.parent.provider_policy["approvals_reviewer"] = reviewer.clone();
        }
        let mut request = base.clone();
        if let Some(kind) = case.get("kind") {
            request.kind = serde_json::from_value(kind.clone()).unwrap();
            match request.kind {
                ChildRequestKind::UserInput => {
                    request.details["method"] = json!("item/tool/requestUserInput");
                    request.details["effects"] =
                        json!({"command":false,"network":false,"file_write":false});
                    request.details["questions"] = json!([{"id":"q1","header":"Choix","question":"Quelle couleur ?","isSecret":false}]);
                    request.question_ids = vec!["q1".into()];
                }
                ChildRequestKind::ElicitationForm => {
                    request.details["method"] = json!("mcpServer/elicitation/request")
                }
                ChildRequestKind::ApprovalFile => {
                    request.details["method"] = json!("item/fileChange/requestApproval");
                    request.details["effects"] =
                        json!({"command":false,"network":false,"file_write":true});
                }
                ChildRequestKind::ApprovalCommand => {}
            }
        }
        if let Some(details) = case.get("details").and_then(Value::as_object) {
            for (key, value) in details {
                request.details[key] = value.clone();
            }
        }
        request.truncated = case
            .get("truncated")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let expected: ChildRequestVerdict =
            serde_json::from_value(case["expected"].clone()).unwrap();
        let result = super::super::classify_request(
            &policy,
            &request,
            case.get("parent_available")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        );
        assert_eq!(
            result,
            (expected, case.get("code").and_then(Value::as_str)),
            "cas {}",
            case["label"]
        );
    }
}

#[test]
fn native155_files_chemins_sans_valeur_d_autorisation() {
    let (root, _) = fixture_root("files155");
    fs::create_dir(root.join("zone")).unwrap();
    fs::create_dir(root.join("outside")).unwrap();
    fs::create_dir(root.join("changing")).unwrap();
    std::os::unix::fs::symlink(root.join("outside"), root.join("alias")).unwrap();
    let parent = codex_fact(
        root.to_str().unwrap(),
        json!({"type":"workspaceWrite","networkAccess":true,"writableRoots":[root]}),
        "on-request",
    );
    let frozen = snapshot(
        "parent155",
        "instance-1",
        parent.clone(),
        json!({"kind":"codex","approval_policy":"on-request","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
        &root,
    );
    let mut request = ChildProviderRequest {
        version: 1,
        provider_session: "child-session155".into(),
        turn_id: "child-turn155".into(),
        provider_request_id: ChildProviderRequestId::String("files155".into()),
        kind: ChildRequestKind::ApprovalFile,
        details: json!({"method":"item/fileChange/requestApproval","duration":"once","complete":true,"secret":false,"effects":{"command":false,"network":false,"file_write":true},"paths_cited":[]}),
        offered: vec![ChildRequestDecision::Accept, ChildRequestDecision::Decline],
        question_ids: vec![],
        truncated: false,
    };
    let paths = [
        ("dans la zone", root.join("zone/tracked")),
        ("avec dotdot", root.join("zone/../outside/tracked")),
        ("lien symbolique", root.join("alias/tracked")),
        ("fichier neuf", root.join("zone/new-file")),
        ("dossier remplacé", root.join("changing/new-file")),
        ("metadata git", root.join(".git/config")),
        (
            "tmp extérieur",
            PathBuf::from("/tmp/bridget-native155-cited"),
        ),
    ];
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for (label, path) in paths {
            request.details["paths_cited"] = json!([path]);
            assert_eq!(
                super::super::classify_request(&frozen, &request, true),
                (
                    ChildRequestVerdict::Forbidden,
                    Some("file_confinement_unproven")
                ),
                "{label}"
            );
            if label == "dossier remplacé" {
                fs::remove_dir(root.join("changing")).unwrap();
                std::os::unix::fs::symlink(root.join("outside"), root.join("changing")).unwrap();
                assert_eq!(
                    super::super::classify_request(&frozen, &request, true),
                    (
                        ChildRequestVerdict::Forbidden,
                        Some("file_confinement_unproven")
                    ),
                    "après mutation réelle en lien"
                );
            }
        }
        let mut full = frozen.clone();
        full.parent.provider_policy["sandbox_policy"] = json!({"type":"dangerFullAccess"});
        assert_eq!(
            super::super::classify_request(&full, &request, true),
            (ChildRequestVerdict::Human, None),
            "écriture synthétique établie avec plein accès"
        );
    }));
    fs::remove_dir_all(root).unwrap();
    if let Err(error) = outcome {
        std::panic::resume_unwind(error);
    }
}

#[test]
fn native155_files_affichage_borne_nettoye_non_verifie_sans_effet_sur_verdict() {
    let paths = (0..65)
        .map(|index| format!("/cited/path{index:03}"))
        .collect::<Vec<_>>();
    let shown = super::super::format_cited_paths(&paths);
    assert!(shown.contains("non vérifi"));
    assert!(shown.contains("path000") && shown.contains("path063"));
    assert!(!shown.contains("path064"), "64 chemins au plus");
    assert!(shown.len() <= 4096);
    let hostile = vec![format!("/cited/{}\u{202e}\n\u{001b}", "é".repeat(3000))];
    let cleaned = super::super::format_cited_paths(&hostile);
    assert!(cleaned.len() <= 4096, "borne en octets UTF-8");
    assert!(cleaned.contains("non vérifi"));
    assert!(!cleaned.chars().any(bridget_core::is_disallowed_control));
    for sandbox in [
        json!({"type":"workspaceWrite","networkAccess":true}),
        json!({"type":"dangerFullAccess"}),
    ] {
        let parent = codex_fact("/bridget-fixture155", sandbox, "on-request");
        let frozen = snapshot(
            "parent155",
            "instance-1",
            parent.clone(),
            json!({"kind":"codex","approval_policy":"on-request","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
            Path::new("/bridget-fixture155"),
        );
        let mut request = ChildProviderRequest {
            version: 1,
            provider_session: "child-session155".into(),
            turn_id: "child-turn155".into(),
            provider_request_id: ChildProviderRequestId::Number(155),
            kind: ChildRequestKind::ApprovalFile,
            details: json!({"method":"item/fileChange/requestApproval","duration":"once","complete":true,"secret":false,"effects":{"command":false,"network":false,"file_write":true},"paths_cited":hostile}),
            offered: vec![ChildRequestDecision::Accept, ChildRequestDecision::Decline],
            question_ids: vec![],
            truncated: false,
        };
        let before = super::super::classify_request(&frozen, &request, true);
        request.details["paths_cited"] = json!([cleaned]);
        assert_eq!(
            super::super::classify_request(&frozen, &request, true),
            before,
            "afficher/nettoyer des chemins ne donne aucun droit"
        );
    }
}
