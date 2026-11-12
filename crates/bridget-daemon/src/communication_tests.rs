use super::*;

#[test]
fn spec138_scope_matrix_and_reason_bounds() {
    let a = CommunicationProject {
        host: "host".into(),
        root: "/a/.git".into(),
    };
    let b = CommunicationProject {
        host: "host".into(),
        root: "/b/.git".into(),
    };
    assert!(
        project_scope(Some(&a), Some(&a), "b", None)
            .unwrap()
            .is_empty()
    );
    assert!(
        project_scope(Some(&a), Some(&b), "b", None)
            .unwrap_err()
            .starts_with("cross_project_reason_required")
    );
    let warnings = project_scope(Some(&a), Some(&b), "b", Some("  shared review  ")).unwrap();
    assert_eq!(warnings[0].code, "cross_project");
    assert_eq!(warnings[0].reason.as_deref(), Some("shared review"));
    assert_eq!(
        project_scope(Some(&a), None, "b", None).unwrap()[0].code,
        "project_unknown"
    );
    for invalid in ["", "  ", "reason\n", "\treason", "rea\0son"] {
        assert!(validate_cross_project_reason(Some(invalid)).is_err());
    }
    assert!(validate_cross_project_reason(Some(&"é".repeat(256))).is_ok());
    assert!(validate_cross_project_reason(Some(&"é".repeat(257))).is_err());
}

#[test]
fn spec138_git_environment_does_not_override_project() {
    const CHILD: &str = "BRIDGET_138_GIT_ENV_CHILD";
    if std::env::var_os(CHILD).is_some() {
        use std::os::unix::fs::DirBuilderExt;
        let root = std::env::temp_dir().join(format!("b138-env-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        assert!(std::process::Command::new("git").args(["init", "--quiet"])
            .arg(&root).env_remove("GIT_DIR").env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR").status().unwrap().success());
        assert!(
            resolve_communication_project(
                root.to_str().unwrap(),
                "test-host",
                CommunicationProjectSource::Git,
                None
            )
            .is_some()
        );
        std::fs::remove_dir_all(&root).unwrap();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "communication::tests::spec138_git_environment_does_not_override_project",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("GIT_DIR", "/nonexistent-bridget-138")
        .env("GIT_WORK_TREE", "/nonexistent-bridget-138-worktree")
        .env("GIT_COMMON_DIR", "/nonexistent-bridget-138-common")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn spec138_real_worktree_and_symlink_share_common_root() {
    use std::os::unix::fs::DirBuilderExt;
    let fixture = std::env::temp_dir().join(format!("b138-common-{}", uuid::Uuid::new_v4().simple()));
    std::fs::DirBuilder::new().mode(0o700).create(&fixture).unwrap();
    let primary = fixture.join("primary");
    let worktree = fixture.join("worktree");
    assert!(std::process::Command::new("git").args(["init", "--quiet"])
        .arg(&primary).env_remove("GIT_DIR").env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR").status().unwrap().success());
    // Git >= 2.44 : un worktree orphelin exerce common-dir sans commit ni identité.
    assert!(std::process::Command::new("git").arg("-C").arg(&primary)
        .args(["worktree", "add", "--quiet", "--orphan"]).arg(&worktree)
        .env_remove("GIT_DIR").env_remove("GIT_WORK_TREE").env_remove("GIT_COMMON_DIR")
        .status().unwrap().success());
    let repository = std::process::Command::new("git")
        .arg("-C")
        .arg(&worktree)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .unwrap();
    let common = std::path::PathBuf::from(String::from_utf8(repository.stdout).unwrap().trim());
    let primary = common.parent().unwrap();
    let a = resolve_communication_project(
        worktree.to_str().unwrap(),
        "test-host",
        CommunicationProjectSource::Git,
        None,
    )
    .unwrap();
    let b = resolve_communication_project(
        primary.to_str().unwrap(),
        "test-host",
        CommunicationProjectSource::Git,
        None,
    )
    .unwrap();
    assert_eq!(a, b);
    let link = std::env::temp_dir().join(format!("b138-link-{}", uuid::Uuid::new_v4().simple()));
    std::os::unix::fs::symlink(worktree, &link).unwrap();
    let result = resolve_communication_project(
        link.to_str().unwrap(),
        "test-host",
        CommunicationProjectSource::Git,
        None,
    );
    std::fs::remove_file(&link).unwrap();
    assert_eq!(result, Some(a));
    assert!(
        resolve_communication_project(
            primary.to_str().unwrap(),
            "inconnu",
            CommunicationProjectSource::Git,
            None
        )
        .is_none()
    );
    std::fs::remove_dir_all(&fixture).unwrap();
}

#[test]
fn spec138_same_basename_repositories_remain_other_projects() {
    use std::os::unix::fs::DirBuilderExt;
    let root =
        std::env::temp_dir().join(format!("b138-homonyms-{}", uuid::Uuid::new_v4().simple()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let mut projects = Vec::new();
    for parent in ["first", "second"] {
        let repository = root.join(parent).join("same-name");
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&repository)
            .unwrap();
        assert!(
            std::process::Command::new("git")
                .args(["init", "--quiet"])
                .arg(&repository)
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .env_remove("GIT_COMMON_DIR")
                .status()
                .unwrap()
                .success()
        );
        projects.push(
            resolve_communication_project(
                repository.to_str().unwrap(),
                "test-host",
                CommunicationProjectSource::Git,
                None,
            )
            .unwrap(),
        );
    }
    assert_ne!(projects[0].root, projects[1].root);
    assert_eq!(
        project_relation(Some(&projects[0]), Some(&projects[1])),
        ProjectRelation::Other
    );
    assert!(
        project_scope(Some(&projects[0]), Some(&projects[1]), "recipient", None)
            .unwrap_err()
            .starts_with("cross_project_reason_required")
    );
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn spec138_reason_is_part_of_canonical_envelope() {
    let base = bridget_core::BridgetMessage::new("a", "b", "exact body");
    let mut wire = serde_json::to_value(&base).unwrap();
    wire["cross_project_reason"] = serde_json::json!("shared review");
    let with_reason: bridget_core::BridgetMessage = serde_json::from_value(wire).unwrap();
    assert_ne!(
        canonical_send("scope", "id", &base, 1),
        canonical_send("scope", "id", &with_reason, 1)
    );
}

#[test]
fn spec138_explicit_null_reason_is_not_legacy_absence() {
    let mut wire =
        serde_json::to_value(bridget_core::BridgetMessage::new("a", "b", "body")).unwrap();
    wire["cross_project_reason"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<bridget_core::BridgetMessage>(wire).is_err());
}

#[test]
fn portee_stable_reste_identique_a_la_reference() {
    assert_eq!(
        issuer_scope("instance-stable-089"),
        "012_scope_37419e46a921ee300d273a45c16ebe1a"
    );
    assert_ne!(
        issuer_scope("instance-stable-089"),
        issuer_scope("instance-other")
    );
}

#[test]
fn canon_historique_garde_ses_octets_et_ses_frontieres() {
    let mut message: bridget_core::BridgetMessage = serde_json::from_str(
            r#"{"id":"message-1","from":"agent-a","to":"agent-b","body":"texte\n  suite","reply":true,"hops":4,"deadline_at":123060,"in_reply_to":"request-0"}"#
        ).unwrap();
    let bytes = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "627269646765742f636c69656e742d73656e642f76310000000000000000163031325f73636f70655f61616161616161616161616100000000000000096d6573736167652d3100000000000000076167656e742d62000000000000000d74657874650a2020737569746501000000000000000400000004010000000000000006313233303630010000000000000009726571756573742d300000000000000008000000000001e078"
    );
    message.from_display_name = Some("Nouveau nom".into());
    assert_eq!(
        bytes,
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000)
    );
    let mut titled = serde_json::to_value(&message).unwrap();
    titled["thread_display_title"] = serde_json::json!("Titre de présentation");
    let titled: bridget_core::BridgetMessage = serde_json::from_value(titled).unwrap();
    assert_eq!(
        bytes,
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &titled, 123000)
    );
    assert_eq!(message.content_key(), titled.content_key());
    message.in_reply_to = Some("request-other".into());
    assert_ne!(
        bytes,
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000)
    );
}

#[test]
fn spec102_v20_canon_sans_notice_inchange_et_chaque_champ_de_notice_compte() {
    let base: bridget_core::BridgetMessage = serde_json::from_str(
            r#"{"id":"message-1","from":"agent-a","to":"agent-b","body":"texte","reply":false,"hops":4}"#,
        )
        .unwrap();
    let reference = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &base, 123000);
    let mut same = base.clone();
    same.thread_notice = None;
    assert_eq!(
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &same, 123000),
        reference
    );
    let notice = bridget_core::ThreadNotice {
        version: 1,
        thread_id: "33333333-3333-4333-8333-333333333333".into(),
        through_seq: 20,
        generation: 2,
    };
    let mut with_notice = base.clone();
    with_notice.thread_notice = Some(notice.clone());
    let canon = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &with_notice, 123000);
    assert_ne!(canon, reference);
    assert!(
        canon.starts_with(&reference),
        "le préfixe historique est conservé"
    );
    for variant in [
        bridget_core::ThreadNotice {
            version: 2,
            ..notice.clone()
        },
        bridget_core::ThreadNotice {
            thread_id: "44444444-4444-4444-8444-444444444444".into(),
            ..notice.clone()
        },
        bridget_core::ThreadNotice {
            through_seq: 21,
            ..notice.clone()
        },
        bridget_core::ThreadNotice {
            generation: 3,
            ..notice.clone()
        },
    ] {
        let mut other = base.clone();
        other.thread_notice = Some(variant);
        assert_ne!(
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &other, 123000),
            canon
        );
    }
}

#[test]
fn spec133_provenance_ne_change_pas_le_rejeu_parental() {
    let mut message = bridget_core::BridgetMessage::new("parent", "cible", "texte");
    message.id = "message-enfant".into();
    let reference = canonical_send("scope-parent", "message-enfant", &message, 123_000);
    message.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
    });
    assert_eq!(
        canonical_send("scope-parent", "message-enfant", &message, 123_000),
        reference,
        "la provenance visible ne change pas le contrat de rejeu du parent"
    );
}

#[test]
fn les_facades_ne_sont_plus_une_dependance_du_noyau() {
    // Oracle d'architecture : remettre un appel via MCP recrée le cycle.
    for source in [
        include_str!("daemon.rs"),
        include_str!("store.rs"),
        include_str!("store/ledger_requests.rs"),
        include_str!("store/service_events.rs"),
        include_str!("store/project_compat.rs"),
        include_str!("idempotency.rs"),
        include_str!("idempotency/send_delivery.rs"),
        include_str!("idempotency/spawn_commands.rs"),
        include_str!("idempotency/agent_links.rs"),
        include_str!("referent_control.rs"),
        include_str!("cli.rs"),
    ] {
        assert!(!source.contains("crate::mcp::issuer_scope"));
    }
}
