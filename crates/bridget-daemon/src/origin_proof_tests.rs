use super::*;
fn request() -> OriginProofRequest {
    OriginProofRequest {
        version: 1,
        correlation_id: uuid::Uuid::new_v4().to_string(),
        connection_epoch: "epoch177".into(),
        worker: WorkerIdentity {
            machine_id: "origin177".into(),
            worker_id: uuid::Uuid::new_v4().to_string(),
            provisioning_revision: 1,
        },
        expires_at_ms: 20_000,
        request: OriginProofOperation::Attest {
            expected_thread: "thread177".into(),
            proof: NativePermissionProof {
                endpoint: "http://127.0.0.1:1/mcp".into(),
                authorization: IdentityCredential::new("Bearer private-177".into()),
            },
        },
    }
}
fn proof(thread: &str) -> OriginSessionProof {
    OriginSessionProof {
        version: 1,
        environment_id: "environment177".into(),
        thread_id: thread.into(),
        provider_session_id: "session177".into(),
        provider_instance_id: "instance177".into(),
        permissions: None,
    }
}
#[test]
fn mobile177_origin_attestation_requires_expected_thread_and_live_deadline() {
    let r = request();
    assert!(matches!(
        execute_with(&r, || 10_000, |_| Ok(proof("thread177"))),
        OriginProofResult::Attested { .. }
    ));
    assert!(matches!(
        execute_with(&r, || 10_000, |_| Ok(proof("other"))),
        OriginProofResult::Rejected {
            reason: WorkerRefusal::Unauthorized
        }
    ));
    assert!(matches!(
        execute_with(
            &r,
            || 20_000,
            |_| panic!("expired request must not call provider")
        ),
        OriginProofResult::Rejected {
            reason: WorkerRefusal::PrepareExpired
        }
    ));
    let clock = std::cell::Cell::new(10_000);
    assert!(matches!(
        execute_with(
            &r,
            || clock.get(),
            |_| {
                clock.set(20_001);
                Ok(proof("thread177"))
            }
        ),
        OriginProofResult::Rejected {
            reason: WorkerRefusal::PrepareExpired
        }
    ));
    assert!(!format!("{:?}", r).contains("private-177"));
}
#[test]
fn spec254_local_directory_prepare_reaches_real_helper_without_git_or_ssh() {
    use std::os::unix::fs::DirBuilderExt;
    use std::path::PathBuf;
    // Processus de test isolé : le helper ne doit jamais découvrir le
    // registre réel du compte, ni changer HOME dans le processus des tests.
    if std::env::var_os("BRIDGET_TEST_LOCAL254").is_none() {
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("b254-{}", uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let home = root.join("home");
        std::fs::create_dir(&home).unwrap();
        let status=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","origin_proof::tests::spec254_local_directory_prepare_reaches_real_helper_without_git_or_ssh","--nocapture"])
                .env("BRIDGET_TEST_LOCAL254",&root).env("HOME",&home).output().unwrap();
        assert!(
            status.status.success(),
            "{}",
            String::from_utf8_lossy(&status.stdout)
        );
        std::fs::remove_dir_all(root).unwrap();
        return;
    }
    let root = PathBuf::from(std::env::var_os("BRIDGET_TEST_LOCAL254").unwrap());
    let source = root.join("documents");
    std::fs::create_dir(&source).unwrap();
    let namespace = root.join("registry");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&namespace)
        .unwrap();
    let path = namespace.join("agents.json");
    let mut envelope = request();
    let config = serde_json::json!({"machines":{envelope.worker.machine_id.clone():{
            "enabled":true,"worker_id":envelope.worker.worker_id,"provisioning_revision":1,
            "credential_fingerprint":"a".repeat(64),"mappings":{}}}});
    bridget_transport::fsutil::write_private_file_atomic(
        &path,
        &serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    // Le Mac réel possède aussi un registre de profil, distinct du worker.
    // Il ne doit être ni choisi ni modifié par ce parcours attesté local.
    let generic = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".config/bridget/agents.json");
    std::fs::create_dir_all(generic.parent().unwrap()).unwrap();
    let mut profile = config.clone();
    profile["machines"][&envelope.worker.machine_id]["worker_id"] =
        serde_json::json!("other-profile254");
    let generic_before = serde_json::to_vec(&profile).unwrap();
    bridget_transport::fsutil::write_private_file_atomic(&generic, &generic_before).unwrap();
    let parent = ProviderPermissions {
        version: 1,
        source: "provider_turn".into(),
        run_id: "run254".into(),
        provider_session_id: "session254".into(),
        provider_instance_id: "instance254".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: source.to_string_lossy().into_owned(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never",
                "approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
    };
    envelope.request = OriginProofOperation::PrepareProject {
        parent: parent.clone(),
        source: WorkerSourceBase {
            cwd: parent.cwd.clone(),
            expected_base_commit: None,
            workspace: Some(WorkspaceRequest::Shared {}),
            workload: AutoDelegateWorkload::General,
        },
        target_machine: envelope.worker.machine_id.clone(),
        project_id: None,
        return_context: true,
    };
    assert!(envelope.validate_shape().is_ok());
    let before = std::fs::read(&path).unwrap();
    for case in 0..4 {
        let mut wrong = envelope.clone();
        if let OriginProofOperation::PrepareProject { source, .. } = &mut wrong.request {
            match case {
                0 => source.expected_base_commit = Some("a".repeat(40)),
                1 => {
                    source.workspace = Some(WorkspaceRequest::Worktree {
                        base_ref: None,
                        branch: None,
                    })
                }
                2 => source.workload = AutoDelegateWorkload::Compilation,
                _ => source.cwd = "relative".into(),
            }
        }
        assert!(wrong.validate_shape().is_err());
    }
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let result = execute_with_context(&envelope, || 10_000, |_| panic!("no provider"), Some(&path));
    assert!(
        matches!(
            result,
            OriginProofResult::ProjectPrepared {
                preparation_error: None,
                project: Some(_),
                ..
            }
        ),
        "{result:?}"
    );
    assert!(!source.join(".git").exists());
    assert_eq!(
        std::fs::read_dir(&source).unwrap().count(),
        0,
        "aucun contenu copié ou créé"
    );
    assert_eq!(
        std::fs::read(&generic).unwrap(),
        generic_before,
        "registre de profil inchangé"
    );
}

#[test]
fn mobile177_origin_sources_read_real_git_and_reject_escape_without_launching() {
    use std::os::unix::fs::{DirBuilderExt, symlink};
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("bridget177-origin-git-{}", uuid::Uuid::new_v4()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let source = root.join("source");
    std::fs::create_dir(&source).unwrap();
    let git = |args: &[&str]| {
        let result = std::process::Command::new("git")
            .arg("-C")
            .arg(&source)
            .args(args)
            .output()
            .unwrap();
        assert!(result.status.success(), "fixture Git command failed");
        String::from_utf8(result.stdout).unwrap().trim().to_owned()
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::write(source.join("file"), b"fixture177").unwrap();
    git(&["add", "file"]);
    git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.com",
        "commit",
        "-qm",
        "Fixture177",
    ]);
    let head = git(&["rev-parse", "HEAD"]);
    let parent = ProviderPermissions {
        version: 1,
        source: "provider_turn".into(),
        run_id: "run177".into(),
        provider_session_id: "session177".into(),
        provider_instance_id: "instance177".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: source.to_string_lossy().into_owned(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
    };
    let mut envelope = request();
    envelope.request = OriginProofOperation::Sources {
        parent: parent.clone(),
        source: Some(WorkerSourceBase {
            cwd: parent.cwd.clone(),
            expected_base_commit: Some(head.clone()),
            workspace: Some(WorkspaceRequest::Shared {}),
            workload: AutoDelegateWorkload::General,
        }),
    };
    let prove = |request: &OriginProofRequest| {
        execute_with(
            request,
            || 10_000,
            |_| panic!("source check does not create a provider session"),
        )
    };
    assert!(matches!(
        prove(&envelope),
        OriginProofResult::Sources {
            source_error: None,
            ..
        }
    ));
    if let OriginProofOperation::Sources {
        source: Some(source),
        ..
    } = &mut envelope.request
    {
        source.expected_base_commit = Some("b".repeat(40));
    }
    assert!(
        matches!(prove(&envelope),OriginProofResult::Sources {source_error:Some(code),..} if code=="expected_base_commit_mismatch")
    );
    let outside = root.join("outside");
    std::fs::create_dir(&outside).unwrap();
    symlink(&outside, source.join("escape")).unwrap();
    if let OriginProofOperation::Sources {
        source: Some(input),
        ..
    } = &mut envelope.request
    {
        input.cwd = source.join("escape").to_string_lossy().into_owned();
    }
    assert!(
        matches!(prove(&envelope),OriginProofResult::Sources {source_error:Some(code),..} if code=="cwd_outside_parent_project")
    );
    envelope.request = OriginProofOperation::PrepareProject {
        parent: parent.clone(),
        source: WorkerSourceBase {
            cwd: parent.cwd.clone(),
            expected_base_commit: Some(head.clone()),
            workspace: Some(WorkspaceRequest::Shared {}),
            workload: AutoDelegateWorkload::General,
        },
        target_machine: "target177".into(),
        project_id: None,
        return_context: false,
    };
    let legacy = serde_json::to_value(&envelope.request).unwrap();
    assert!(
        legacy.get("return_context").is_none(),
        "un ancien pair garde sa trame exacte"
    );
    let decoded: OriginProofOperation = serde_json::from_value(legacy).unwrap();
    assert!(matches!(
        decoded,
        OriginProofOperation::PrepareProject {
            return_context: false,
            ..
        }
    ));
    assert!(
        matches!(execute_with_context(&envelope,||10_000,|_|panic!("no provider"),
            Some(&root.join("agents.json"))),OriginProofResult::ProjectPrepared {preparation_error:Some(code),..}
            if code=="project_preparation_link_missing")
    );
    assert!(matches!(
        prove(&envelope),
        OriginProofResult::Rejected {
            reason: WorkerRefusal::Unauthorized
        }
    ));
    if let OriginProofOperation::PrepareProject { source, .. } = &mut envelope.request {
        source.expected_base_commit = Some("b".repeat(40));
    }
    assert!(
        matches!(execute_with_context(&envelope,||10_000,|_|panic!("no provider"),
            Some(&root.join("agents.json"))),OriginProofResult::ProjectPrepared {preparation_error:Some(code),..}
            if code=="expected_base_commit_mismatch")
    );
    if let OriginProofOperation::PrepareProject { target_machine, .. } = &mut envelope.request {
        *target_machine = "origin177".into();
    }
    assert_eq!(
        envelope.validate_shape_detailed().unwrap_err().0,
        "target_machine"
    );
    //236 : le helper prend déjà en charge objects_only sur la machine source.
    if let OriginProofOperation::PrepareProject { source, .. } = &mut envelope.request {
        source.workspace = Some(WorkspaceRequest::CommitView {});
        source.expected_base_commit = Some(head.clone());
    }
    assert!(envelope.validate_shape().is_ok());
    assert!(
        matches!(execute_with_context(&envelope,||10_000,|_|panic!("no provider"),
            Some(&root.join("agents.json"))),OriginProofResult::ProjectPrepared {preparation_error:Some(code),..}
            if code=="project_preparation_link_missing")
    );
    let valid = envelope.clone();
    for (field, case) in [
        ("cwd", 0),
        ("expected_base_commit", 1),
        ("target_machine", 2),
        ("parent", 3),
        ("worker", 4),
    ] {
        let mut wrong = valid.clone();
        if let OriginProofOperation::PrepareProject {
            parent,
            source,
            target_machine,
            ..
        } = &mut wrong.request
        {
            match case {
                0 => source.cwd = "relative236".into(),
                1 => source.expected_base_commit = Some("invalid236".into()),
                2 => *target_machine = String::new(),
                3 => parent.revision = 0,
                _ => wrong.worker.provisioning_revision = 0,
            }
        }
        let (found, reason) = wrong.validate_shape_detailed().unwrap_err();
        assert_eq!(found, field);
        assert_eq!(wrong.validate_shape(), Err(reason));
    }
    for workspace in [
        WorkspaceRequest::Shared {},
        WorkspaceRequest::Worktree {
            branch: None,
            base_ref: None,
        },
    ] {
        let mut wrong = valid.clone();
        if let OriginProofOperation::PrepareProject { source, .. } = &mut wrong.request {
            source.workspace = Some(workspace);
        }
        assert_eq!(
            wrong.validate_shape_detailed().unwrap_err().0,
            "target_machine"
        );
    }
    envelope.request = OriginProofOperation::Catalogue {
        parent: parent.clone(),
        blocked: vec![],
    };
    assert!(matches!(
        prove(&envelope),
        OriginProofResult::Rejected {
            reason: WorkerRefusal::Unauthorized
        }
    ));
    let catalogue = execute_with_context(
        &envelope,
        || 10_000,
        |_| panic!("catalogue does not launch a provider"),
        Some(&root.join("agents.json")),
    );
    let OriginProofResult::Catalogue {
        permission_digest,
        entries,
    } = catalogue
    else {
        panic!("origin catalogue refused");
    };
    assert_eq!(
        permission_digest,
        crate::native_permissions::prove_parent_permissions(&parent, parent.revision).unwrap()
    );
    assert!(!entries.is_empty());
    assert!(entries.len() <= 128);
    assert!(entries.iter().all(|entry| entry["agent_type"].is_string()));
    assert!(!root.join("agents.json").exists());
    assert!(!root.join("bridget.db").exists());
    assert_eq!(git(&["rev-parse", "HEAD"]), head);
    assert!(!source.join(".worktrees").exists());
    std::fs::remove_dir_all(root).unwrap();
}
