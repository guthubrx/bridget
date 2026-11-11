use super::*;

#[test]
fn spec241_task_authority_reopens_without_promoting_historical_records() {
    let root = std::env::temp_dir().join(format!("b241-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("s.db");
    let store = DelegationStore::open(&path).unwrap();
    let mut task = legacy_task151();
    assert!(task.admitted_parent().unwrap().is_none());
    assert!(
        serde_json::to_value(&task)
            .unwrap()
            .get("admission_authority_version")
            .is_none()
    );
    let parent = serde_json::from_value(serde_json::json!({
        "version":1,"source":"provider_turn","run_id":"r241","provider_session_id":"s241",
        "provider_instance_id":"i241","revision":1,"driver":"codex_app_server",
        "cwd":task.cwd,"runtime_mode":"full-access","interaction_mode":"default",
        "provider_policy":{"kind":"codex","approval_policy":"never","approvals_reviewer":"user",
            "sandbox_policy":{"type":"dangerFullAccess"}}
    }))
    .unwrap();
    task.permission_snapshot = Some(crate::native_permissions::snapshot(
        &task.owner,
        &task.owner_instance,
        parent,
        serde_json::json!({}),
        Path::new(&task.cwd),
    ));
    task.admission_authority_version = Some(1);
    let canonical = serde_json::to_vec(&task.request).unwrap();
    store.insert(&task, "authority241", &canonical).unwrap();
    drop(store);
    let store = DelegationStore::open(&path).unwrap();
    let read = store.get(&task.task_id).unwrap().unwrap();
    assert_eq!(
        read.admitted_parent().unwrap(),
        task.admitted_parent().unwrap()
    );
    let mut invalid = read.clone();
    invalid.owner = "foreign241".into();
    assert!(invalid.admitted_parent().is_err());
    invalid = read.clone();
    invalid.admission_authority_version = Some(99);
    assert_eq!(
        invalid.admitted_parent().unwrap_err(),
        "admission_authority_unsupported"
    );
    invalid = read.clone();
    invalid.admission_authority_version = None;
    assert!(invalid.admitted_parent().unwrap().is_none());
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec221_c19_reader_reopens_task_and_workspace_without_quarantine() {
    let root = std::env::temp_dir().join(format!("bridget-c19-reader-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("state.db");
    let store = DelegationStore::open(&path).unwrap();
    let mut task = legacy_task151();
    let NativeDelegationRequest::Delegate { workspace, .. } = &mut task.request else {
        panic!("delegate fixture");
    };
    *workspace = Some(bridget_transport::protocol::WorkspaceRequest::CommitView {});
    task.workspace = Some(WorkspaceRecord {
        strategy: WorkspaceStrategy::CommitView,
        commit_view: None,
        state: WorkspaceState::Kept,
        toplevel: "/private/c19-manifests".into(),
        path: "/private/source-views/fixture".into(),
        branch: "c19-manifest-digest".into(),
        base_commit: "a".repeat(40),
        head_commit: None,
        branch_commit: None,
        head_on_branch: None,
        commit_count: None,
        commits: Vec::new(),
        truncated: false,
        dirty: None,
    });
    let canonical = serde_json::to_vec(&task.request).unwrap();
    store.insert(&task, "future-c19", &canonical).unwrap();
    // L'insertion normalise les champs historiques (date/propriétaire).
    // La reprise doit comparer la matière réellement durcie, pas la fixture.
    let expected = serde_json::to_value(store.get(&task.task_id).unwrap().unwrap()).unwrap();
    drop(store);
    let reopened = DelegationStore::open(&path).unwrap();
    let read = reopened.get(&task.task_id).unwrap().unwrap();
    assert!(matches!(
        read.workspace.as_ref().unwrap().strategy,
        WorkspaceStrategy::CommitView
    ));
    assert_eq!(serde_json::to_value(&read).unwrap(), expected);
    assert_eq!(read.request, task.request);
    assert_eq!(
        serde_json::to_value(&read.workspace).unwrap(),
        serde_json::to_value(&task.workspace).unwrap()
    );
    let error = crate::workspace_git::report(read.workspace.as_ref().unwrap()).unwrap_err();
    assert!(error.starts_with("workspace_commit_view_audit_required"));
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}

fn remote162_receipt_fixture() -> (
    DelegationStore,
    Task,
    bridget_transport::protocol::RemoteResultEnvelope,
) {
    use bridget_transport::protocol::*;
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let mut task = legacy_task151();
    let result = RemoteResultEnvelope {
        worker: WorkerIdentity {
            machine_id: "cartae".into(),
            worker_id: "worker".into(),
            provisioning_revision: 1,
        },
        identity: WorkerResultIdentity {
            command: WorkerCommandIdentity {
                command_id: "command".into(),
                task_id: task.task_id.clone(),
                instance_id: "instance".into(),
                generation: 1,
            },
            mission_id: task.mission.clone(),
            round: 1,
            execution_id: "execution".into(),
            execution_generation: 1,
        },
        message_id: "result".into(),
        body: "résultat durable".into(),
        digest: String::new(),
    };
    let mut result = result;
    result.digest = result.canonical_digest().unwrap();
    task.remote_result_slots.push(WorkerResultSlot {
        identity: result.identity.clone(),
        admission_digest: "a".repeat(64),
    });
    store.insert(&task, "request", b"canonical").unwrap();
    let task = store.get(&task.task_id).unwrap().unwrap();
    (store, task, result)
}

#[test]
fn remote162_result_commit_atomique_et_echec_avant_commit() {
    use bridget_transport::protocol::RemoteResultDisposition as D;
    let (store, task, result) = remote162_receipt_fixture();
    let mut next = task.clone();
    next.result = Some(result.body.clone());
    next.state = "waiting_for_children".into();
    store.conn.execute_batch("CREATE TRIGGER refuse_remote_result BEFORE UPDATE ON native_delegations BEGIN SELECT RAISE(ABORT,'fixture rollback'); END;").unwrap();
    assert!(
        store
            .commit_remote_result(&task, next.clone(), &result, D::Accepted {})
            .is_err()
    );
    let untouched = store.get(&task.task_id).unwrap().unwrap();
    assert!(untouched.result.is_none() && untouched.remote_result_receipts.is_empty());
    store
        .conn
        .execute_batch("DROP TRIGGER refuse_remote_result;")
        .unwrap();
    store
        .commit_remote_result(&task, next, &result, D::Accepted {})
        .unwrap();
    let committed = store.get(&task.task_id).unwrap().unwrap();
    assert_eq!(committed.result.as_deref(), Some(result.body.as_str()));
    assert_eq!(
        committed.remote_result_receipts[0].disposition,
        D::Accepted {}
    );
    // Une sauvegarde métier ancienne ne peut retirer la preuve du résultat.
    assert!(store.save(&task).is_err());
    store.save(&committed).unwrap();
    assert_eq!(
        store
            .get(&task.task_id)
            .unwrap()
            .unwrap()
            .remote_result_receipts
            .len(),
        1
    );
}

#[test]
fn remote162_conflits_bornes_et_cas_stale_sans_ecrasement() {
    use bridget_transport::protocol::RemoteResultDisposition as D;
    let (store, task, result) = remote162_receipt_fixture();
    let mut next = task.clone();
    next.result = Some(result.body.clone());
    next.state = "waiting_for_children".into();
    store
        .commit_remote_result(&task, next, &result, D::Accepted {})
        .unwrap();
    assert!(
        store
            .commit_remote_result(&task, task.clone(), &result, D::Accepted {})
            .is_err()
    );
    for index in 0..3 {
        let current = store.get(&task.task_id).unwrap().unwrap();
        let mut conflict = result.clone();
        conflict.body = format!("autre {index}");
        conflict.digest = conflict.canonical_digest().unwrap();
        store
            .commit_remote_result(&current, current.clone(), &conflict, D::RejectedConflict {})
            .unwrap();
    }
    let saved = store.get(&task.task_id).unwrap().unwrap();
    assert_eq!(saved.result, Some(result.body));
    assert_eq!(saved.remote_result_receipts.len(), 1);
    assert!(saved.remote_result_receipts[0].conflict_seen);
    assert_eq!(saved.remote_result_receipts[0].digest, result.digest);
    assert_eq!(saved.remote_result_receipts[0].disposition, D::Accepted {});
}

fn legacy_task151() -> Task {
    let task_id = uuid::Uuid::new_v4().to_string();
    let payload = serde_json::json!({
        "task_id": task_id, "owner": "parent", "owner_instance": "instance-parent",
        "origin_owner_instance": "instance-parent",
        "request": {"operation": "delegate", "request_id": "request-152",
            "agent_type": "fixture", "model": "fixture", "task": "mission", "cwd": "/tmp"},
        "definition": {"command": "fixture", "args": [], "protocol": "acp",
            "forbidden_env": [], "pass_env": [], "permissions": "allow",
            "queue_capacity": 32, "notify_timeout_secs": 600,
            "mcp": {"interactive": "none", "acp_session": false}, "digest": "fixture"},
        "cwd": "/tmp", "child": "child", "child_instance": null, "mission": "mission-152",
        "created_at": 100, "state": "queued", "result": null, "error": null,
        "result_sent": false, "cleanup_done": false, "failure_sent": false,
        "delivery_acked": false, "acknowledged_at": null
    });
    serde_json::from_value(payload).unwrap()
}

fn auto_receipt163(owner: &str, request_id: &str, epoch: i64, at: i64) -> AutoAdmissionReceipt {
    AutoAdmissionReceipt {
        owner: owner.into(),
        request_id: request_id.into(),
        canonical: format!(r#"{{"operation":"auto_delegate","request_id":"{request_id}"}}"#)
            .into_bytes(),
        metadata: br#"{"policy_revision":7,"profiles":["c01"]}"#.to_vec(),
        metadata_version: 1,
        state: AutoAdmissionState::Pending,
        coordinator_epoch: epoch,
        revision: 0,
        policy_revision: 7,
        policy_digest: "a".repeat(64),
        candidate_id: None,
        delegate_canonical: None,
        mode: None,
        reason: None,
        task_id: None,
        refusal_code: None,
        created_at: at,
        updated_at: at,
    }
}

fn auto_refusal163(owner: &str, request_id: &str, epoch: i64, at: i64) -> AutoAdmissionReceipt {
    let mut receipt = auto_receipt163(owner, request_id, epoch, at);
    receipt.state = AutoAdmissionState::Refused;
    receipt.refusal_code = Some("decision_interrupted".into());
    receipt
}

fn child_request155() -> ChildRequest {
    ChildRequest {
        request_id: "request155".into(),
        task_id: "task155".into(),
        child_instance: "child155".into(),
        provider_session: "session155".into(),
        turn_id: "turn155".into(),
        provider_request_id: "42".into(),
        content_hash: "hash155".into(),
        kind: "user_input".into(),
        verdict: "parent".into(),
        state: "open".into(),
        outcome: None,
        summary: "Question".into(),
        facts_json: "{}".into(),
        model_text: "Choisir".into(),
        offered_json: "[\"accept\",\"decline\",\"cancel\"]".into(),
        created_at: 100,
        deadline_at: 200,
        decision: None,
        answer_json: None,
        decided_by: None,
        decided_at: None,
        delivery_state: "none".into(),
        void_reason: None,
        sent_at: None,
        delivered_at: None,
        inbox_item_id: None,
        inbox_settled: false,
    }
}

#[test]
fn native157_stockage_payload150_sans_substitution() {
    // La fixture relit le JSON150 original, sans champ substitution.
    let task = legacy_task151();
    assert!(task.substitution.is_none());
}

#[test]
fn remote162_task_ancienne_locale_et_placement_durable() {
    let mut task = legacy_task151();
    assert!(task.remote_placement.is_none());
    assert!(
        serde_json::to_value(&task)
            .unwrap()
            .get("remote_placement")
            .is_none()
    );
    task.remote_placement = Some(
        serde_json::from_value(serde_json::json!({
            "machine_id":"cartae", "worker_id":"worker", "provisioning_revision":1,
            "project_id":"project", "mapping_revision":1, "origin_host":"mac",
            "origin_cwd":"/Users/moi/project", "source_root_identity":"git:project",
            "relative_cwd":".", "execution_cwd":"/tmp", "base_commit":null,
            "definition_digest":"a".repeat(64), "permission_digest":"b".repeat(64)
        }))
        .unwrap(),
    );
    let root =
        std::env::temp_dir().join(format!("bridget-remote162-task-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("bridget.db");
    let store = DelegationStore::open(&path).unwrap();
    store.insert(&task, "request-152", b"canonical").unwrap();
    drop(store);
    let reopened = DelegationStore::open(&path).unwrap();
    assert_eq!(
        reopened
            .get(&task.task_id)
            .unwrap()
            .unwrap()
            .remote_placement,
        task.remote_placement
    );
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native160_task_payload157_defauts() {
    // Le payload157 ne porte ni attente quota ni décision en attente.
    let mut payload = serde_json::to_value(legacy_task151()).unwrap();
    payload.as_object_mut().unwrap().remove("quota_waits");
    payload.as_object_mut().unwrap().remove("quota_hold");
    let task: Task = serde_json::from_value(payload).unwrap();
    let decoded = serde_json::to_value(task).unwrap();
    assert_eq!(decoded.get("quota_waits"), Some(&serde_json::json!([])));
    assert_eq!(decoded.get("quota_hold"), Some(&serde_json::Value::Null));
}

#[test]
fn native160_task_quota_sans_signal_lineage() {
    // Même sauvegarde/projection réelle que l'oracle152 existant.
    let mut task = legacy_task151();
    task.state = "working".into();
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    store.insert(&task, "request-152", b"canonical").unwrap();
    let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
    store.take_changes();
    let receiver = store.change_receiver().unwrap();
    receiver.try_recv().unwrap();
    let hold = serde_json::json!({
        "since":120, "error":"refus", "execution_id":"execution-mission-152"
    });
    let mut wait = serde_json::json!({
        "since":120, "reset_at":200, "round":1,
        "identity":format!("limit-resume:{}:1:200", task.task_id),
        "execution_id":"execution-mission-152", "phase":"waiting",
        "until":null, "resume_execution_id":null
    });
    for phase in [None, Some("waiting"), Some("relaunching"), Some("done")] {
        let mut updated = serde_json::to_value(store.get(&task.task_id).unwrap().unwrap()).unwrap();
        let (expected_hold, expected_waits) = if let Some(phase) = phase {
            wait["phase"] = serde_json::json!(phase);
            wait["until"] = if phase == "done" {
                serde_json::json!(230)
            } else {
                serde_json::Value::Null
            };
            wait["resume_execution_id"] = if phase == "waiting" {
                serde_json::Value::Null
            } else {
                serde_json::json!("execution-resumed-152")
            };
            (serde_json::Value::Null, serde_json::json!([wait.clone()]))
        } else {
            (hold.clone(), serde_json::json!([]))
        };
        // Le checkpoint durable peut précéder la fermeture SQL de la feuille.
        // Il ne doit pas devenir une erreur publique pendant Relaunching.
        updated["error"] = if phase == Some("relaunching") {
            serde_json::json!("native_relaunch_failed")
        } else {
            serde_json::Value::Null
        };
        updated["quota_hold"] = expected_hold.clone();
        updated["quota_waits"] = expected_waits.clone();
        store
            .save(&serde_json::from_value(updated).unwrap())
            .unwrap();
        let saved = serde_json::to_value(store.get(&task.task_id).unwrap().unwrap()).unwrap();
        assert_eq!(saved.get("quota_hold"), Some(&expected_hold));
        assert_eq!(saved.get("quota_waits"), Some(&expected_waits));
        if phase == Some("relaunching") {
            assert_eq!(
                saved["error"], "native_relaunch_failed",
                "checkpoint interne conservé"
            );
            assert_eq!(
                store.get(&task.task_id).unwrap().unwrap().projected_error(),
                None
            );
        }
        assert_eq!(
            store.lineage_page("parent", 100, None, |_| false).unwrap(),
            before
        );
        assert!(store.take_changes().is_empty());
        assert!(receiver.try_recv().is_err());
    }
    // Après la clôture confirmée, failed et son erreur deviennent visibles.
    let mut failed = store.get(&task.task_id).unwrap().unwrap();
    failed.state = "failed".into();
    failed.error = Some("native_relaunch_failed".into());
    store.save(&failed).unwrap();
    let after = store.lineage_page("parent", 100, None, |_| false).unwrap();
    assert_ne!(after, before);
    assert_eq!(after["tasks"][0]["status"], "failed");
    assert_eq!(after["tasks"][0]["error"], "native_relaunch_failed");
    assert!(!store.take_changes().is_empty());
    assert!(receiver.try_recv().is_ok());
}

#[test]
fn native155_store_payload150_sans_mode() {
    let task = legacy_task151();
    assert_eq!(task.child_requests, None);
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    store.insert(&task, "request-152", b"canonical").unwrap();
    assert_eq!(
        store.get(&task.task_id).unwrap().unwrap().child_requests,
        None
    );
}

#[test]
fn native155_store_doublon_identique_meme_ligne() {
    use bridget_transport::protocol::ChildProviderRequestId;
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    for (index, provider_id) in [
        ChildProviderRequestId::Number(i64::MIN),
        ChildProviderRequestId::String(i64::MIN.to_string()),
    ]
    .into_iter()
    .enumerate()
    {
        let mut request = child_request155();
        request.request_id = format!("request155-{index}");
        request.provider_request_id = serde_json::to_string(&provider_id).unwrap();
        assert_eq!(store.insert_child_request(&request).unwrap(), request);
        assert!(
            store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    Some("{}"),
                    "parent",
                    "answered_by_parent",
                    110
                )
                .unwrap()
        );
        let decided = store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<ChildProviderRequestId>(&decided.provider_request_id).unwrap(),
            provider_id
        );
        assert_eq!(store.insert_child_request(&request).unwrap(), decided);
    }
    assert_eq!(store.child_request_counts("task155").unwrap(), (0, 2));
}

#[test]
fn native155_store_doublon_different_request_conflict() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let request = child_request155();
    store.insert_child_request(&request).unwrap();
    let mut divergent = request.clone();
    divergent.content_hash = "other".into();
    assert_eq!(
        store.insert_child_request(&divergent).unwrap_err(),
        "request_conflict"
    );
    assert_eq!(
        store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap(),
        request
    );
}

#[test]
fn native155_store_terminal_immuable() {
    for state in ["expired", "cancelled", "refused", "answered"] {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let request = child_request155();
        store.insert_child_request(&request).unwrap();
        if state == "answered" {
            assert!(
                store
                    .decide_child_request(
                        &request.request_id,
                        "accept",
                        Some("{}"),
                        "parent",
                        "answered_by_parent",
                        110
                    )
                    .unwrap()
            );
        } else {
            assert!(
                store
                    .close_child_request(&request.request_id, state, state, 110)
                    .unwrap()
            );
        }
        let closed = store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap();
        assert!(
            !store
                .close_child_request(&request.request_id, "cancelled", "cancelled", 120)
                .unwrap()
        );
        assert!(
            !store
                .decide_child_request(
                    &request.request_id,
                    "decline",
                    None,
                    "human",
                    "answered_by_human",
                    120
                )
                .unwrap()
        );
        assert_eq!(
            store
                .get_child_request(&request.request_id)
                .unwrap()
                .unwrap(),
            closed
        );
    }
}

#[test]
fn native155_store_void_ne_redevient_pas_pending() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let request = child_request155();
    store.insert_child_request(&request).unwrap();
    assert!(
        store
            .decide_child_request(
                &request.request_id,
                "accept",
                Some("{}"),
                "parent",
                "answered_by_parent",
                110
            )
            .unwrap()
    );
    assert!(
        store
            .void_pending_child_request(&request.request_id, "paused")
            .unwrap()
    );
    assert!(
        !store
            .decide_child_request(
                &request.request_id,
                "accept",
                Some("{}"),
                "parent",
                "answered_by_parent",
                120
            )
            .unwrap()
    );
    assert!(!store.claim_child_request(&request.request_id, 120).unwrap());
    let void = store
        .get_child_request(&request.request_id)
        .unwrap()
        .unwrap();
    assert_eq!(void.delivery_state, "void");
    assert_eq!(void.void_reason.as_deref(), Some("paused"));
    assert_eq!(void.decision.as_deref(), Some("accept"));
    assert_eq!(store.insert_child_request(&request).unwrap(), void);
}

#[test]
fn native155_store_sent_ne_redevient_pas_pending() {
    for delivery in ["delivered", "void", "unknown"] {
        let store = DelegationStore::open(Path::new(":memory:")).unwrap();
        let request = child_request155();
        store.insert_child_request(&request).unwrap();
        assert!(
            store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    Some("{}"),
                    "parent",
                    "answered_by_parent",
                    110
                )
                .unwrap()
        );
        assert!(store.claim_child_request(&request.request_id, 120).unwrap());
        assert!(!store.claim_child_request(&request.request_id, 121).unwrap());
        assert!(
            !store
                .void_pending_child_request(&request.request_id, "cancelled")
                .unwrap()
        );
        assert!(
            !store
                .decide_child_request(
                    &request.request_id,
                    "accept",
                    Some("{}"),
                    "parent",
                    "answered_by_parent",
                    121
                )
                .unwrap()
        );
        let sent = store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap();
        assert_eq!(sent.delivery_state, "sent");
        assert_eq!(sent.sent_at, Some(120));
        assert!(
            store
                .settle_sent_child_request(
                    &request.request_id,
                    delivery,
                    Some("closed_before_apply"),
                    130
                )
                .unwrap()
        );
        assert!(
            !store
                .settle_sent_child_request(&request.request_id, "void", None, 131)
                .unwrap()
        );
        let settled = store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap();
        assert_eq!(settled.state, sent.state);
        assert_eq!(settled.decision, sent.decision);
        assert_eq!(settled.answer_json, sent.answer_json);
        assert_eq!(settled.decided_at, sent.decided_at);
        assert_eq!(settled.delivery_state, delivery);
        assert_eq!(
            settled.outcome.as_deref(),
            Some(if delivery == "unknown" {
                "cancelled_outcome_unknown"
            } else {
                "answered_by_parent"
            })
        );
        assert_eq!(
            settled.delivered_at,
            if delivery == "delivered" {
                Some(130)
            } else {
                None
            }
        );
        assert_eq!(store.insert_child_request(&request).unwrap(), settled);
    }
}

#[test]
fn native155_store_cas_echeance_et_claim_distinct() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let request = child_request155();
    store.insert_child_request(&request).unwrap();
    assert_eq!(
        store.child_request_counts(&request.task_id).unwrap(),
        (1, 1)
    );
    assert_eq!(
        store.open_child_requests(&request.task_id).unwrap(),
        vec![request.clone()]
    );
    assert!(store.due_child_requests(199).unwrap().is_empty());
    assert_eq!(
        store.due_child_requests(200).unwrap(),
        vec![request.clone()]
    );
    assert!(
        !store
            .decide_child_request(
                &request.request_id,
                "accept",
                None,
                "parent",
                "answered_by_parent",
                200
            )
            .unwrap()
    );
    assert!(!store.claim_child_request(&request.request_id, 199).unwrap());
    assert_eq!(
        store
            .get_child_request(&request.request_id)
            .unwrap()
            .unwrap(),
        request
    );
    assert!(
        store
            .decide_child_request(
                &request.request_id,
                "accept",
                None,
                "parent",
                "answered_by_parent",
                199
            )
            .unwrap()
    );
    assert!(!store.claim_child_request(&request.request_id, 200).unwrap());
    assert!(store.claim_child_request(&request.request_id, 199).unwrap());
    assert!(!store.claim_child_request(&request.request_id, 199).unwrap());
}

#[test]
fn native154_stockage_payloads150_152_153_sans_workspace() {
    let base = serde_json::to_value(legacy_task151()).unwrap();
    for version in [150, 152, 153] {
        let mut payload = base.clone();
        payload.as_object_mut().unwrap().remove("workspace");
        payload
            .as_object_mut()
            .unwrap()
            .remove("workspace_report_mission");
        if version >= 152 {
            payload["launch_issued_at"] = serde_json::json!(110);
            payload["restart_resumes"] = serde_json::json!(1);
        }
        if version >= 153 {
            payload["round"] = serde_json::json!(2);
            payload["guarded"] = serde_json::json!(true);
            payload["request"]["keep_alive_secs"] = serde_json::json!(600);
            payload["request"]["brief"] =
                serde_json::json!({"objective":"objectif","summary":"résultat"});
        }
        let task: Task = serde_json::from_value(payload).unwrap();
        assert!(task.workspace.is_none(), "payload {version}");
        assert!(task.workspace_report_mission.is_none(), "payload {version}");
    }
}

#[test]
fn native154_stockage_workspace_sans_signal_lineage() {
    let mut task = legacy_task151();
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    store.insert(&task, "request-152", b"canonical").unwrap();
    let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
    store.take_changes();
    let receiver = store.change_receiver().unwrap();
    receiver.try_recv().unwrap();
    task.workspace = Some(WorkspaceRecord {
        strategy: WorkspaceStrategy::Worktree,
        commit_view: None,
        state: WorkspaceState::Active,
        toplevel: "/tmp/repository".into(),
        path: "/tmp/repository/.worktrees/task".into(),
        branch: "bridget/task".into(),
        base_commit: "a".repeat(40),
        head_commit: None,
        branch_commit: None,
        head_on_branch: None,
        commit_count: None,
        commits: Vec::new(),
        truncated: false,
        dirty: None,
    });
    for state in [WorkspaceState::Active, WorkspaceState::Kept] {
        task.workspace.as_mut().unwrap().state = state;
        if task.workspace.as_ref().unwrap().state == WorkspaceState::Kept {
            task.workspace_report_mission = Some(task.mission.clone());
        }
        store.save(&task).unwrap();
        let saved = store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(saved.workspace).unwrap(),
            serde_json::to_value(&task.workspace).unwrap()
        );
        assert_eq!(
            saved.workspace_report_mission,
            task.workspace_report_mission
        );
        assert_eq!(
            store.lineage_page("parent", 100, None, |_| false).unwrap(),
            before
        );
        assert!(store.take_changes().is_empty());
        assert!(receiver.try_recv().is_err());
    }
    let mut record = serde_json::to_value(task.workspace.unwrap()).unwrap();
    record["state"] = serde_json::json!("removed");
    assert!(serde_json::from_value::<WorkspaceRecord>(record).is_err());
}

#[test]
fn native152_task_payload151_defauts_et_champs_invisibles_lineage() {
    let task = legacy_task151();
    let task_id = task.task_id.clone();
    let decoded = serde_json::to_value(&task).unwrap();
    for (field, default) in [
        ("launch_issued_at", serde_json::json!(null)),
        ("mission_issued_at", serde_json::json!(null)),
        ("restart_resumes", serde_json::json!(0)),
        ("cancel_stopped", serde_json::json!([])),
    ] {
        assert_eq!(decoded.get(field), Some(&default));
    }
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    store.insert(&task, "request-152", b"canonical").unwrap();
    let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
    store.take_changes();
    let receiver = store.change_receiver().unwrap();
    receiver.try_recv().unwrap();
    let mut updated = serde_json::to_value(store.get(&task_id).unwrap().unwrap()).unwrap();
    let new_values = [
        ("launch_issued_at", serde_json::json!(110)),
        ("mission_issued_at", serde_json::json!(120)),
        ("restart_resumes", serde_json::json!(2)),
        ("cancel_stopped", serde_json::json!(["child"])),
    ];
    for (field, value) in &new_values {
        updated[*field] = value.clone();
    }
    store
        .save(&serde_json::from_value(updated).unwrap())
        .unwrap();
    let saved = serde_json::to_value(store.get(&task_id).unwrap().unwrap()).unwrap();
    for (field, value) in new_values {
        assert_eq!(saved.get(field), Some(&value));
    }
    assert_eq!(
        store.lineage_page("parent", 100, None, |_| false).unwrap(),
        before
    );
    assert!(store.take_changes().is_empty());
    assert!(receiver.try_recv().is_err());
}

#[test]
fn native152_refus_rejeu_identique() {
    let dir = std::env::temp_dir().join(format!("native152-refus-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("native.sqlite");
    let store = DelegationStore::open(&path).unwrap();
    let epoch = store.wait_epoch().get();
    let before = store.lineage_page("parent", 100, None, |_| false).unwrap();
    let receiver = store.change_receiver().unwrap();
    let codes = [
        "cwd_outside_parent_project",
        "cwd_unavailable",
        "model_unavailable",
        "effort_unavailable",
        "agent_type_unknown",
    ];
    for (index, code) in codes.into_iter().enumerate() {
        let request = format!("request-{index}");
        assert!(
            store
                .record_refusal("parent", &request, b"canonical", code, 100)
                .unwrap()
        );
        assert!(
            !store
                .record_refusal("parent", &request, b"different", "cwd_unavailable", 200)
                .unwrap()
        );
        assert_eq!(
            store
                .by_agent_request("parent", &request, b"canonical")
                .unwrap_err(),
            code
        );
        let saved: (Vec<u8>, String, i64) = store.conn.query_row(
                "SELECT canonical,code,refused_at FROM native_delegation_refusals WHERE owner=?1 AND request_id=?2",
                params!["parent", request], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).unwrap();
        assert_eq!(saved, (b"canonical".to_vec(), code.to_string(), 100));
    }
    assert_eq!(store.wait_epoch().get(), epoch);
    assert!(store.take_changes().is_empty());
    assert!(receiver.try_recv().is_err());
    assert_eq!(
        store.lineage_page("parent", 100, None, |_| false).unwrap(),
        before
    );
    drop(store);
    let store = DelegationStore::open(&path).unwrap();
    assert_eq!(
        store
            .by_agent_request("parent", "request-0", b"canonical")
            .unwrap_err(),
        codes[0]
    );
    assert!(
        store
            .by_agent_request("other", "request-0", b"canonical")
            .unwrap()
            .is_none()
    );
    let mut task = legacy_task151();
    if let NativeDelegationRequest::Delegate { request_id, .. } = &mut task.request {
        *request_id = "request-0".into();
    }
    store.insert(&task, "request-0", b"canonical").unwrap();
    assert_eq!(
        store
            .by_agent_request("parent", "request-0", b"canonical")
            .unwrap()
            .unwrap()
            .task_id,
        task.task_id
    );
    store
        .conn
        .execute(
            "DELETE FROM native_delegation_refusals WHERE request_id='request-0'",
            [],
        )
        .unwrap();
    assert!(
        !store
            .record_refusal("parent", "request-0", b"canonical", codes[0], 300)
            .unwrap()
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM native_delegation_refusals WHERE request_id='request-0'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn native152_refus_autre_enveloppe_mismatch() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    store
        .record_refusal("parent", "request", b"canonical", "model_unavailable", 100)
        .unwrap();
    assert_eq!(
        store
            .by_agent_request("parent", "request", b"other")
            .unwrap_err(),
        "envelope_mismatch"
    );
}

fn assert_refusal_not_retained(code: &str) {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    assert!(
        !store
            .record_refusal("parent", "request", b"canonical", code, 100)
            .unwrap()
    );
    assert!(
        store
            .by_agent_request("parent", "request", b"other")
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM native_delegation_refusals",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn native152_refus_transitoire_jamais_garde_referent_paused() {
    assert_refusal_not_retained("referent_paused");
}
#[test]
fn native152_refus_transitoire_jamais_garde_task_limit() {
    assert_refusal_not_retained("task_limit");
}
#[test]
fn native152_refus_transitoire_jamais_garde_retention_saturated() {
    assert_refusal_not_retained("retention_saturated");
}
#[test]
fn native152_refus_transitoire_jamais_garde_identity_unavailable() {
    assert_refusal_not_retained("identity_unavailable");
}
#[test]
fn native152_refus_transitoire_jamais_garde_reply_timeout_exceeds_parent() {
    assert_refusal_not_retained("reply_timeout_exceeds_parent");
}
#[test]
fn native152_refus_transitoire_jamais_garde_parent_task_unavailable() {
    assert_refusal_not_retained("parent_task_unavailable");
}
#[test]
fn native152_refus_transitoire_jamais_garde_parent_execution_ambiguous() {
    assert_refusal_not_retained("parent_execution_ambiguous");
}
#[test]
fn native152_refus_transitoire_jamais_garde_delegation_grant_required() {
    assert_refusal_not_retained("delegation_grant_required");
}
#[test]
fn native152_refus_transitoire_jamais_garde_permission_not_inherited() {
    assert_refusal_not_retained("permission_not_inherited");
}
#[test]
fn native152_refus_transitoire_jamais_garde_permission_attestation_unavailable() {
    assert_refusal_not_retained("permission_attestation_unavailable");
}

#[test]
fn native151_attente_compteur_reveil_et_lecture_non_bloquante() {
    let epoch = WaitEpoch::default();
    let sleeper = epoch.clone();
    let seen = epoch.get();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let join = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        sleeper.wait_changed(seen, Duration::from_secs(60));
    });
    started_rx.recv().unwrap();
    let start = std::time::Instant::now();
    assert_eq!(epoch.get(), 0);
    epoch.bump();
    join.join().unwrap();
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(epoch.get(), 1);
}

#[test]
fn native151_attente_compteur_tranche_et_evenement_deja_passe() {
    let epoch = WaitEpoch::default();
    let start = std::time::Instant::now();
    epoch.wait_changed(0, Duration::from_millis(40));
    assert!(start.elapsed() >= Duration::from_millis(35));
    epoch.bump();
    let start = std::time::Instant::now();
    epoch.wait_changed(0, Duration::from_secs(60));
    assert!(start.elapsed() < Duration::from_millis(100));
}
#[test]
fn grant_survives_restart_and_revoke() {
    let dir = std::env::temp_dir().join(format!("bridget-native-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("native.sqlite");
    let store = DelegationStore::open(&path).unwrap();
    store
        .grant("owner", &dir, SpawnPosture::Development, false)
        .unwrap();
    drop(store);
    let store = DelegationStore::open(&path).unwrap();
    assert_eq!(
        store.permission("owner").unwrap().unwrap().1,
        SpawnPosture::Development
    );
    assert!(store.permission("other").unwrap().is_none());
    store
        .grant("owner", &dir, SpawnPosture::Discovery, true)
        .unwrap();
    assert!(store.permission("owner").unwrap().is_none());
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn control158_halt_store_pose_levee() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    assert_eq!(
        store.delegation_halt().unwrap(),
        HaltState {
            halted: false,
            since: None,
            by: None,
            reason: None,
            generation: 0,
        }
    );
    let pose = store
        .set_delegation_halt("A", true, "cli", Some("essai"), || Ok(0), 100)
        .unwrap();
    assert_eq!(
        pose,
        serde_json::json!({"type":"delegation_halt_state","halted":true,
            "since":100,"by":"cli","reason":"essai","generation":1,"cancelled":0,"replayed":false})
    );
    let levee = store
        .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
        .unwrap();
    assert_eq!(levee["halted"], false);
    assert_eq!(levee["generation"], 2);
    assert!(levee["since"].is_null() && levee["by"].is_null() && levee["reason"].is_null());
    assert!(!store.delegation_halt().unwrap().halted);
    assert_eq!(
        store
            .conn
            .query_row("SELECT count(*) FROM native_delegation_halt", [], |row| row
                .get::<_, u32>(0))
            .unwrap(),
        1
    );
}

#[test]
fn control158_halt_store_a_b_rejeu_a() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let first = store
        .set_delegation_halt("A", true, "cli", Some("premier"), || Ok(0), 100)
        .unwrap();
    store
        .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
        .unwrap();
    let before = store.delegation_halt().unwrap();
    let original_receipt: String = store
        .conn
        .query_row(
            "SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let replay = store
        .set_delegation_halt(
            "A",
            true,
            "humain",
            Some("autre"),
            || panic!("aucun comptage au rejeu"),
            999,
        )
        .unwrap();
    let mut expected = first;
    expected["replayed"] = serde_json::json!(true);
    assert_eq!(replay, expected);
    assert_eq!(store.delegation_halt().unwrap(), before);
    assert!(!before.halted);
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        original_receipt
    );
}

#[test]
fn control158_halt_store_a_b_rejeu_a_apres_redemarrage() {
    // Reprise du montage SQLite disque déjà utilisé par les tests du magasin.
    let dir = std::env::temp_dir().join(format!("native158-halt-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("native.sqlite");
    let store = DelegationStore::open(&path).unwrap();
    let first = store
        .set_delegation_halt("A", true, "cli", Some("premier"), || Ok(0), 100)
        .unwrap();
    store
        .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
        .unwrap();
    drop(store);
    let store = DelegationStore::open(&path).unwrap();
    let before = store.delegation_halt().unwrap();
    let mut expected = first;
    expected["replayed"] = serde_json::json!(true);
    assert_eq!(
        store
            .set_delegation_halt(
                "A",
                true,
                "cli",
                Some("premier"),
                || panic!("aucun comptage après reprise"),
                300
            )
            .unwrap(),
        expected
    );
    assert_eq!(store.delegation_halt().unwrap(), before);
    assert!(!before.halted);
    assert_eq!(before.generation, 2);
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn control158_halt_store_recu_porte_la_reponse() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    // Trois arbres ciblés : valeur fournie par l'appelant, pas une terminaison simulée.
    let first = store
        .set_delegation_halt("A", true, "cli", Some("compte fourni"), || Ok(3), 100)
        .unwrap();
    let saved: String = store
        .conn
        .query_row(
            "SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&saved).unwrap(),
        first
    );
    assert_eq!(first["cancelled"], 3);
    store
        .set_delegation_halt("B", false, "cli", None, || Ok(0), 200)
        .unwrap();
    let replay = store
        .set_delegation_halt("A", true, "cli", None, || Ok(0), 300)
        .unwrap();
    assert_eq!(replay["cancelled"], 3);
    assert_eq!(replay["generation"], first["generation"]);
    assert_eq!(replay["replayed"], true);
}

#[test]
fn control158_halt_store_recus_bornes() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    for index in 0..257 {
        store
            .set_delegation_halt(
                &format!("command-{index}"),
                true,
                "cli",
                None,
                || Ok(0),
                index,
            )
            .unwrap();
    }
    let count: u32 = store
        .conn
        .query_row(
            "SELECT count(*) FROM native_delegation_halt_commands",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 256);
    assert!(!store.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_delegation_halt_commands WHERE command_id='command-0')",
            [], |row| row.get::<_,bool>(0)).unwrap());
    assert!(store.conn.query_row("SELECT EXISTS(SELECT 1 FROM native_delegation_halt_commands WHERE command_id='command-256')",
            [], |row| row.get::<_,bool>(0)).unwrap());
    assert_eq!(
        store
            .set_delegation_halt("command-256", true, "cli", None, || Ok(0), 999)
            .unwrap()["replayed"],
        true
    );
}

#[test]
fn control158_halt_store_motif_borne() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let text = "é".repeat(crate::referent_control::MAX_REASON_CHARS + 1);
    let response = store
        .set_delegation_halt("A", true, "cli", Some(&text), || Ok(0), 100)
        .unwrap();
    let expected = "é".repeat(crate::referent_control::MAX_REASON_CHARS);
    assert_eq!(response["reason"], expected);
    assert_eq!(
        store.delegation_halt().unwrap().reason.as_deref(),
        Some(expected.as_str())
    );
}

// Quatre consommateurs réels ; même legacy_task151 et vraie insertion du magasin.
// Aucun faux verdict, processus, harnais ou hook d'horloge.
fn overview158_insert(store: &DelegationStore, index: u32, mut task: Task) -> Task {
    task.owner = format!("parent-{}", index % 3);
    task.owner_instance = format!("instance-parent-{}", index % 3);
    task.child = format!("child-{index}");
    task.mission = format!("mission-{index}");
    task.mission_deadline_at = Some(task.created_at + 1000);
    let key = format!("overview-{index}");
    store.insert(&task, &key, key.as_bytes()).unwrap();
    task
}

#[test]
fn control158_overview_trois_proprietaires() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut tasks: Vec<Task> = Vec::new();
    for index in 0..10 {
        let mut task = legacy_task151();
        task.state = "working".into();
        task.created_at = now - 30 + index as i64;
        // La lignée est fixée avant insertion, comme dans le magasin livré.
        if index == 3 {
            task.parent_task_id = Some(tasks[0].task_id.clone());
        }
        tasks.push(overview158_insert(&store, index, task));
    }
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let response = store.overview_page("all", 100).unwrap();
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 10);
    assert_eq!(response["total_active"], 10);
    let owners: std::collections::BTreeSet<&str> = items
        .iter()
        .map(|item| item["owner"].as_str().unwrap())
        .collect();
    assert_eq!(owners.len(), 3);
    for (item, task) in items.iter().zip(tasks.iter().rev()) {
        assert_eq!(item["task_id"], task.task_id);
        assert_eq!(item["owner"], task.owner);
        assert_eq!(item["child"], task.child);
        assert_eq!(item["state"], "working");
        assert_eq!(
            item["parent_task_id"],
            serde_json::to_value(&task.parent_task_id).unwrap()
        );
        let age = item["age_secs"].as_i64().unwrap();
        let remaining = item["deadline_in_secs"].as_i64().unwrap();
        assert!(
            (before.min(after) - task.created_at..=before.max(after) - task.created_at)
                .contains(&age)
        );
        assert!(
            (task.mission_deadline_at.unwrap() - before.max(after)
                ..=task.mission_deadline_at.unwrap() - before.min(after))
                .contains(&remaining)
        );
    }
    assert_eq!(response["halted"], false);
    assert!(
        response["since"].is_null() && response["by"].is_null() && response["reason"].is_null()
    );
}

#[test]
fn control158_overview_filtre_actif() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let mut live = Vec::new();
    for (index, state) in [
        "working",
        "cancelling",
        "result_available",
        "failed",
        "cancelled",
    ]
    .into_iter()
    .enumerate()
    {
        let mut task = legacy_task151();
        task.state = state.into();
        task.created_at = 100 + index as i64;
        if index == 2 {
            task.guarded = true;
        }
        // Garde immuable posée avant admission.
        else if index > 2 {
            task.cleanup_done = true;
        }
        let task = overview158_insert(&store, index as u32, task);
        if index <= 2 {
            live.push(task.task_id);
        }
    }
    let active = store.overview_page("active", 100).unwrap();
    let ids: Vec<&str> = active["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["task_id"].as_str().unwrap())
        .collect();
    let expected: Vec<&str> = live.iter().rev().map(String::as_str).collect();
    assert_eq!(ids, expected);
    assert_eq!(active["total_active"], 3);
    assert_eq!(
        store.overview_page("all", 100).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn control158_overview_limite_100() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    for index in 0..101 {
        let mut task = legacy_task151();
        task.state = "working".into();
        task.created_at = index as i64;
        overview158_insert(&store, index, task);
    }
    let page = store.overview_page("all", 100).unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 100);
    assert_eq!(page["total_active"], 101, "le total précède la pagination");
    assert_eq!(
        store.overview_page("active", 1).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for invalid in [0, 101] {
        assert_eq!(
            store.overview_page("all", invalid).unwrap_err(),
            bridget_transport::protocol::DelegationControlRefusal::InvalidRequest
        );
    }
}

#[test]
fn control158_overview_sans_corps_ni_resultat() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let mut seed = legacy_task151();
    seed.state = "result_available".into();
    seed.created_at = 100;
    let mut task = overview158_insert(&store, 0, seed);
    if let NativeDelegationRequest::Delegate { task: body, .. } = &mut task.request {
        *body = "MISSION_PRIVEE_158".into();
    }
    task.result = Some("RESULTAT_PRIVE_158".into());
    task.cwd = "/chemin-prive-158".into();
    task.mission_deadline_at = None;
    store.save(&task).unwrap();
    let response = store.overview_page("all", 1).unwrap();
    let item = &response["items"][0];
    let keys: std::collections::BTreeSet<&str> = item
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        std::collections::BTreeSet::from([
            "task_id",
            "owner",
            "child",
            "state",
            "age_secs",
            "deadline_in_secs",
            "parent_task_id"
        ])
    );
    assert!(item["deadline_in_secs"].is_null());
    let wire = response.to_string();
    for private in [
        "MISSION_PRIVEE_158",
        "RESULTAT_PRIVE_158",
        "/chemin-prive-158",
    ] {
        assert!(
            !wire.contains(private),
            "la projection ne contient aucune donnée privée"
        );
    }
}

#[test]
fn spec163_recu_auto_additif_borne_et_idempotent() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let receipt = AutoAdmissionReceipt {
        owner: "owner-163".into(),
        request_id: "request-163".into(),
        canonical: br#"{"operation":"auto_delegate"}"#.to_vec(),
        metadata: br#"{"policy_revision":7,"profiles":["c01"]}"#.to_vec(),
        metadata_version: 1,
        state: AutoAdmissionState::Pending,
        coordinator_epoch: 1,
        revision: 0,
        policy_revision: 7,
        policy_digest: "a".repeat(64),
        candidate_id: None,
        delegate_canonical: None,
        mode: None,
        reason: None,
        task_id: None,
        refusal_code: None,
        created_at: 100,
        updated_at: 100,
    };
    assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
    assert!(!store.insert_auto_admission_receipt(&receipt).unwrap());
    assert_eq!(
        store
            .auto_admission_receipt("owner-163", "request-163")
            .unwrap(),
        Some(receipt.clone())
    );
    let mut collision = receipt.clone();
    collision.canonical = b"different".to_vec();
    assert_eq!(
        store.insert_auto_admission_receipt(&collision).unwrap_err(),
        "envelope_mismatch"
    );
    let mut oversized = receipt;
    oversized.request_id = "too-large-163".into();
    oversized.metadata = vec![b'x'; MAX_AUTO_ADMISSION_METADATA_BYTES + 1];
    assert_eq!(
        store.insert_auto_admission_receipt(&oversized).unwrap_err(),
        "auto_admission_invalid"
    );
}

const LEGACY_AUTO_RECEIPT_CREATE_FOR_168: &str = "CREATE TABLE auto_admission_receipts(
          owner TEXT NOT NULL, request_id TEXT NOT NULL,
          canonical BLOB NOT NULL CHECK (length(canonical) BETWEEN 1 AND 524288),
          metadata BLOB NOT NULL CHECK (length(metadata) BETWEEN 1 AND 16384),
          state TEXT NOT NULL CHECK (state IN ('pending','decision_started','chosen','admitted','refused')),
          coordinator_epoch INTEGER NOT NULL CHECK (coordinator_epoch >= 0),
          revision INTEGER NOT NULL CHECK (revision >= 0), policy_revision INTEGER NOT NULL CHECK (policy_revision > 0),
          policy_digest TEXT NOT NULL, candidate_id TEXT, delegate_canonical BLOB,
          mode TEXT CHECK (mode IS NULL OR mode IN ('explicit','automatic','fallback')),
          reason TEXT, task_id TEXT, refusal_code TEXT, created_at INTEGER NOT NULL,
          updated_at INTEGER NOT NULL, PRIMARY KEY(owner, request_id));";

#[test]
fn spec168_migre_163_sans_alterer_les_blobs_et_borne_v2() {
    let dir = std::env::temp_dir().join(format!("bridget-auto168-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("admission.sqlite");
    let legacy_canonical = b"legacy-canonical".to_vec();
    let legacy_metadata = br#"{"legacy":true}"#.to_vec();
    let legacy_delegate = b"legacy-delegate".to_vec();
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(LEGACY_AUTO_RECEIPT_CREATE_FOR_168)
        .unwrap();
    conn.execute("INSERT INTO auto_admission_receipts VALUES(?1,?2,?3,?4,'chosen',1,2,7,?5,'c01',?6,'automatic',NULL,NULL,NULL,10,11)",
            params!["owner", "legacy", legacy_canonical.clone(), legacy_metadata.clone(), "a".repeat(64), legacy_delegate.clone()]).unwrap();
    let mut legacy_rows = vec![(
        "legacy".to_string(),
        "chosen",
        legacy_canonical,
        legacy_metadata,
        legacy_delegate,
    )];
    for state in ["pending", "decision_started", "admitted", "refused"] {
        let mut canonical = format!("canonical-{state}").into_bytes();
        canonical.extend_from_slice(&[0, 255]);
        let metadata = serde_json::to_vec(&serde_json::json!({"state":state})).unwrap();
        let mut delegate = format!("delegate-{state}").into_bytes();
        delegate.extend_from_slice(&[0, 128, 255]);
        conn.execute(
                "INSERT INTO auto_admission_receipts VALUES(?1,?2,?3,?4,?5,1,0,7,?6,NULL,?7,NULL,NULL,NULL,?8,10,11)",
                params!["owner", format!("legacy-{state}"), canonical, metadata, state,
                    "a".repeat(64), delegate,
                    if state == "refused" { Some("decision_interrupted") } else { None }],
            ).unwrap();
        legacy_rows.push((
            format!("legacy-{state}"),
            state,
            canonical,
            metadata,
            delegate,
        ));
    }
    drop(conn);

    let store = DelegationStore::open(&path).unwrap();
    let renamed_sql: String = store
        .test_connection()
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='auto_admission_receipts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        renamed_sql.contains("\"auto_admission_receipts\""),
        "le schéma renommé par SQLite conserve bien le nom cité"
    );
    for (request, state, canonical, metadata, delegate) in legacy_rows {
        let migrated = store
            .auto_admission_receipt("owner", &request)
            .unwrap()
            .unwrap();
        assert_eq!(migrated.state.as_sql(), state);
        assert_eq!(migrated.metadata_version, 1);
        assert_eq!(migrated.canonical, canonical, "canon opaque de {state}");
        assert_eq!(migrated.metadata, metadata, "snapshot opaque de {state}");
        assert_eq!(
            migrated.delegate_canonical.as_deref(),
            Some(delegate.as_slice()),
            "canon de délégation opaque de {state}"
        );
    }
    let mut v2 = auto_receipt163("owner", "v2", 2, 12);
    v2.metadata_version = 2;
    v2.metadata = serde_json::to_vec(&serde_json::json!({
        "decision_level": "decision_service",
        "snapshot": "x".repeat(MAX_AUTO_ADMISSION_METADATA_V1_BYTES),
    }))
    .unwrap();
    assert!(v2.metadata.len() > MAX_AUTO_ADMISSION_METADATA_V1_BYTES);
    assert!(store.insert_auto_admission_receipt(&v2).unwrap());
    drop(store);
    let store = DelegationStore::open(&path).unwrap();
    assert_eq!(
        store.auto_admission_receipt("owner", "v2").unwrap(),
        Some(v2.clone()),
        "réouverture du snapshot v2 au-delà de 16 Kio, octet pour octet"
    );
    let mut v1_over_limit = auto_receipt163("owner", "too-large-v1", 2, 12);
    v1_over_limit.metadata = serde_json::to_vec(&serde_json::json!({
        "snapshot": "x".repeat(MAX_AUTO_ADMISSION_METADATA_V1_BYTES),
    }))
    .unwrap();
    assert_eq!(
        store
            .insert_auto_admission_receipt(&v1_over_limit)
            .unwrap_err(),
        "auto_admission_invalid"
    );
    v2.request_id = "too-large-v2".into();
    v2.metadata = serde_json::to_vec(&serde_json::json!({
        "decision_level": "decision_service",
        "snapshot": "x".repeat(MAX_AUTO_ADMISSION_METADATA_V2_BYTES),
    }))
    .unwrap();
    assert_eq!(
        store.insert_auto_admission_receipt(&v2).unwrap_err(),
        "auto_admission_invalid"
    );
    let mut oversized_canonical = auto_receipt163("owner", "too-large-canonical", 2, 12);
    oversized_canonical.canonical = vec![b'x'; MAX_AUTO_ADMISSION_CANONICAL_BYTES + 1];
    assert_eq!(
        store
            .insert_auto_admission_receipt(&oversized_canonical)
            .unwrap_err(),
        "auto_admission_invalid"
    );
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn spec168_migration_faute_copie_rollback_conserve_legacy() {
    let dir = std::env::temp_dir().join(format!("bridget-auto168-copy-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("admission.sqlite");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(LEGACY_AUTO_RECEIPT_CREATE_FOR_168)
        .unwrap();
    let canonical = b"copy-fault-canonical".to_vec();
    let metadata = vec![b'x'; MAX_AUTO_ADMISSION_METADATA_V1_BYTES + 1];
    let delegate = vec![0_u8, 128, 255];
    // Seulement pour construire la fixture invalide : la copie réelle
    // réapplique ensuite les contraintes du DDL168, sans aucun failpoint.
    conn.execute_batch("PRAGMA ignore_check_constraints=ON;")
        .unwrap();
    conn.execute(
            "INSERT INTO auto_admission_receipts VALUES('owner','copy-fault',?1,?2,'chosen',1,0,7,?3,'c01',?4,'automatic',NULL,NULL,NULL,10,11)",
            params![canonical, metadata, "a".repeat(64), delegate],
        ).unwrap();
    conn.execute_batch("PRAGMA ignore_check_constraints=OFF;")
        .unwrap();
    let legacy_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='auto_admission_receipts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    drop(conn);
    let error = match DelegationStore::open(&path) {
        Ok(_) => panic!("la copie doit refuser le BLOB legacy hors borne"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("CASE metadata_version"),
        "la faute doit provenir du CHECK de copie168 : {error}"
    );
    let conn = Connection::open(&path).unwrap();
    let after_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='auto_admission_receipts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(after_sql, legacy_sql, "rollback du DDL complet");
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM pragma_table_info('auto_admission_receipts')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        17
    );
    assert_eq!(conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='auto_admission_receipts_168_new'
             OR name IN ('auto_admission_receipts_maintenance','auto_admission_receipts_task','auto_admission_receipts_refused_purge')",
            [], |row| row.get::<_, i64>(0),
        ).unwrap(), 0, "ni table de migration ni index créés avant la copie ne subsistent");
    let blobs: (Vec<u8>, Vec<u8>, Vec<u8>) = conn.query_row(
            "SELECT canonical,metadata,delegate_canonical FROM auto_admission_receipts WHERE owner='owner' AND request_id='copy-fault'",
            [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
    assert_eq!(blobs, (canonical, metadata, delegate));
    assert_eq!(
        conn.query_row("SELECT count(*) FROM auto_admission_receipts", [], |row| {
            row.get::<_, i64>(0)
        },)
            .unwrap(),
        1
    );
    drop(conn);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn spec168_cas_niveau_epoch_revision_et_choix_sont_fermes() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let mut receipt = auto_receipt163("owner", "levels", 9, 10);
    receipt.metadata_version = 2;
    receipt.metadata = br#"{"decision_level":"decision_service","fallback":"c002"}"#.to_vec();
    assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
    let started = store
        .start_auto_decision_level(
            &receipt,
            9,
            0,
            AutoAdmissionDecisionLevel::DecisionService,
            11,
        )
        .unwrap();
    let lightweight = store
        .advance_auto_admission_decision_level(
            &started,
            9,
            1,
            AutoAdmissionDecisionLevel::DecisionService,
            br#"{"decision_level":"lightweight_model","fallback":"c002"}"#.to_vec(),
            12,
        )
        .unwrap();
    assert_eq!(lightweight.revision, 2);
    assert_eq!(
        store
            .choose_auto_admission_at(
                &started,
                9,
                1,
                AutoAdmissionDecisionLevel::DecisionService,
                "c001".into(),
                b"late-service".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                br#"{"decision_level":"decision_service","decision":{"source":"service"}}"#
                    .to_vec(),
                13,
            )
            .unwrap_err(),
        "admission_pending"
    );
    let chosen = store
            .choose_auto_admission_at(
                &lightweight,
                9,
                2,
                AutoAdmissionDecisionLevel::LightweightModel,
                "c001".into(),
                b"lightweight".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                br#"{"decision_level":"lightweight_model","decision":{"source":"lightweight","effort":"low","combination":"fallback"}}"#.to_vec(),
                14,
            )
            .unwrap();
    assert_eq!(chosen.state, AutoAdmissionState::Chosen);
    assert_eq!(
            chosen.metadata,
            br#"{"decision_level":"lightweight_model","decision":{"source":"lightweight","effort":"low","combination":"fallback"}}"#.to_vec()
        );
    assert_eq!(
        store
            .advance_auto_admission_decision_level(
                &lightweight,
                9,
                2,
                AutoAdmissionDecisionLevel::LightweightModel,
                br#"{"decision_level":"decision_service"}"#.to_vec(),
                15,
            )
            .unwrap_err(),
        "admission_pending"
    );
    assert_eq!(
        store.start_auto_decision(&receipt, 16).unwrap_err(),
        "auto_admission_level_unsupported"
    );
    assert_eq!(
        store
            .choose_auto_admission(
                &receipt.owner,
                &receipt.request_id,
                &receipt.canonical,
                "c01".into(),
                b"legacy".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                16,
            )
            .unwrap_err(),
        "auto_admission_level_unsupported"
    );
}

#[test]
fn spec168_reouverture_refuse_trigger_et_fk_entrante() {
    for (suffix, dependency) in [
        (
            "trigger",
            "CREATE TRIGGER receipt_guard BEFORE INSERT ON auto_admission_receipts BEGIN SELECT 1; END;",
        ),
        (
            "incoming",
            "CREATE TABLE receipt_child(receipt_owner TEXT, receipt_request TEXT, FOREIGN KEY(receipt_owner,receipt_request) REFERENCES auto_admission_receipts(owner,request_id));",
        ),
        (
            "incoming-uppercase-cascade",
            "CREATE TABLE receipt_child(receipt_owner TEXT, receipt_request TEXT, FOREIGN KEY(receipt_owner,receipt_request) REFERENCES AUTO_ADMISSION_RECEIPTS(owner,request_id) ON DELETE CASCADE);",
        ),
    ] {
        let dir =
            std::env::temp_dir().join(format!("bridget-auto168-{suffix}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("admission.sqlite");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(LEGACY_AUTO_RECEIPT_CREATE_FOR_168)
            .unwrap();
        let sentinel = vec![0_u8, 128, 255];
        conn.execute(
                "INSERT INTO auto_admission_receipts VALUES('owner','sentinel',?1,?2,'pending',1,0,7,?3,NULL,?1,NULL,NULL,NULL,NULL,10,11)",
                params![sentinel, b"{}".as_slice(), "a".repeat(64)],
            ).unwrap();
        conn.execute_batch(dependency).unwrap();
        if suffix != "trigger" {
            conn.execute("INSERT INTO receipt_child VALUES('owner','sentinel')", [])
                .unwrap();
        }
        let legacy_sql: String = conn.query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='auto_admission_receipts'",
                [], |row| row.get(0),
            ).unwrap();
        drop(conn);
        assert!(DelegationStore::open(&path).is_err(), "{suffix}");
        let conn = Connection::open(&path).unwrap();
        let preserved: Vec<u8> = conn.query_row(
                "SELECT canonical FROM auto_admission_receipts WHERE owner='owner' AND request_id='sentinel'",
                [], |row| row.get(0),
            ).unwrap();
        assert_eq!(preserved, sentinel, "parent legacy conservé : {suffix}");
        let after_sql: String = conn.query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='auto_admission_receipts'",
                [], |row| row.get(0),
            ).unwrap();
        assert_eq!(after_sql, legacy_sql, "refus avant remplacement du schéma");
        if suffix != "trigger" {
            assert_eq!(conn.query_row(
                    "SELECT count(*) FROM receipt_child WHERE receipt_owner='owner' AND receipt_request='sentinel'",
                    [], |row| row.get::<_, i64>(0),
                ).unwrap(), 1, "la sentinelle enfant ne doit pas être supprimée : {suffix}");
        }
        drop(conn);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn spec168_refuse_schema_index_fk_sortante_et_preserve_la_base() {
    let dir = std::env::temp_dir().join(format!("bridget-auto168-schema-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();

    let index_path = dir.join("index.sqlite");
    let store = DelegationStore::open(&index_path).unwrap();
    store
        .test_connection()
        .execute_batch(
            "DROP INDEX auto_admission_receipts_task;
                 CREATE INDEX auto_admission_receipts_task ON auto_admission_receipts(state);",
        )
        .unwrap();
    drop(store);
    assert!(DelegationStore::open(&index_path).is_err());
    let conn = Connection::open(&index_path).unwrap();
    let index_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='index' AND name='auto_admission_receipts_task'",
                [],
                |row| row.get(0),
            )
            .unwrap();
    assert!(
        index_sql.contains("(state)"),
        "échec sans réparation implicite"
    );
    drop(conn);

    let unknown_path = dir.join("unknown.sqlite");
    let store = DelegationStore::open(&unknown_path).unwrap();
    store
        .test_connection()
        .execute_batch("ALTER TABLE auto_admission_receipts ADD COLUMN unknown_shape TEXT")
        .unwrap();
    drop(store);
    assert!(DelegationStore::open(&unknown_path).is_err());
    let conn = Connection::open(&unknown_path).unwrap();
    let columns: i64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('auto_admission_receipts')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(columns, 19, "aucun effacement d'une forme inconnue");
    drop(conn);

    let foreign_key_path = dir.join("outgoing.sqlite");
    let conn = Connection::open(&foreign_key_path).unwrap();
    conn.execute_batch("CREATE TABLE receipt_parent(owner TEXT PRIMARY KEY);
CREATE TABLE auto_admission_receipts(
  owner TEXT NOT NULL, request_id TEXT NOT NULL,
  canonical BLOB NOT NULL CHECK (length(canonical) BETWEEN 1 AND 524288),
  metadata BLOB NOT NULL CHECK (length(metadata) BETWEEN 1 AND 16384),
  state TEXT NOT NULL CHECK (state IN ('pending','decision_started','chosen','admitted','refused')),
  coordinator_epoch INTEGER NOT NULL CHECK (coordinator_epoch >= 0),
  revision INTEGER NOT NULL CHECK (revision >= 0), policy_revision INTEGER NOT NULL CHECK (policy_revision > 0),
  policy_digest TEXT NOT NULL, candidate_id TEXT, delegate_canonical BLOB,
  mode TEXT CHECK (mode IS NULL OR mode IN ('explicit','automatic','fallback')),
  reason TEXT, task_id TEXT, refusal_code TEXT, created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL, PRIMARY KEY(owner, request_id),
  FOREIGN KEY(owner) REFERENCES receipt_parent(owner));
CREATE INDEX auto_admission_receipts_maintenance ON auto_admission_receipts(coordinator_epoch,state,updated_at);
CREATE INDEX auto_admission_receipts_task ON auto_admission_receipts(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX auto_admission_receipts_refused_purge ON auto_admission_receipts(state,task_id,updated_at,owner,request_id) WHERE state='refused' AND task_id IS NULL;").unwrap();
    drop(conn);
    assert!(DelegationStore::open(&foreign_key_path).is_err());
    let conn = Connection::open(&foreign_key_path).unwrap();
    let columns: i64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('auto_admission_receipts')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(columns, 17, "le rollback ne remplace pas une forme refusée");
    drop(conn);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn spec168_v2_persiste_decision_fermee_et_admission_atomique() {
    let dir = std::env::temp_dir().join(format!("bridget-auto168-v2-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("admission.sqlite");
    let store = DelegationStore::open(&path).unwrap();
    let mut receipt = auto_receipt163("owner-v2", "atomic-v2", 7, 10);
    receipt.metadata_version = 2;
    receipt.metadata = br#"{"decision_level":"decision_service","decision":{"source":"service","effort":"medium"}}"#.to_vec();
    assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
    assert_eq!(
        store
            .start_auto_decision_level(
                &receipt,
                8,
                0,
                AutoAdmissionDecisionLevel::DecisionService,
                11,
            )
            .unwrap_err(),
        "admission_pending"
    );
    assert_eq!(
        store
            .start_auto_decision_level(
                &receipt,
                7,
                1,
                AutoAdmissionDecisionLevel::DecisionService,
                11,
            )
            .unwrap_err(),
        "admission_pending"
    );
    assert_eq!(
        store
            .start_auto_decision_level(
                &receipt,
                7,
                0,
                AutoAdmissionDecisionLevel::LightweightModel,
                11,
            )
            .unwrap_err(),
        "admission_pending"
    );
    let started = store
        .start_auto_decision_level(
            &receipt,
            7,
            0,
            AutoAdmissionDecisionLevel::DecisionService,
            11,
        )
        .unwrap();
    let final_metadata = br#"{"decision_level":"decision_service","decision":{"source":"service","effort":"high","score":91,"combination":"single"}}"#.to_vec();
    assert_eq!(
        store
            .choose_auto_admission_at(
                &started,
                7,
                1,
                AutoAdmissionDecisionLevel::LightweightModel,
                "c001".into(),
                b"delegate".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                final_metadata.clone(),
                12,
            )
            .unwrap_err(),
        "admission_pending"
    );
    assert_eq!(
        store
            .choose_auto_admission_at(
                &started,
                8,
                1,
                AutoAdmissionDecisionLevel::DecisionService,
                "c001".into(),
                b"delegate".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                final_metadata.clone(),
                12,
            )
            .unwrap_err(),
        "admission_pending"
    );
    assert_eq!(
        store
            .choose_auto_admission_at(
                &started,
                7,
                1,
                AutoAdmissionDecisionLevel::DecisionService,
                "c001".into(),
                b"delegate".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                br#"{"decision_level":"lightweight_model","decision":{"source":"wrong-level"}}"#
                    .to_vec(),
                12,
            )
            .unwrap_err(),
        "auto_admission_invalid"
    );
    let chosen = store
        .choose_auto_admission_at(
            &started,
            7,
            1,
            AutoAdmissionDecisionLevel::DecisionService,
            "c001".into(),
            b"delegate".to_vec(),
            AutoAdmissionMode::Automatic,
            None,
            final_metadata.clone(),
            12,
        )
        .unwrap();
    let mut wrong_owner = legacy_task151();
    wrong_owner.owner = "other-owner".into();
    assert_eq!(
        store
            .admit_auto_task(&chosen, &wrong_owner, &receipt.request_id, b"delegate", 13)
            .unwrap_err(),
        "native_owner_identity_mismatch"
    );
    assert_eq!(
        store
            .auto_admission_receipt(&receipt.owner, &receipt.request_id)
            .unwrap()
            .unwrap()
            .state,
        AutoAdmissionState::Chosen
    );
    let mut task = legacy_task151();
    task.owner = receipt.owner.clone();
    task.root_owner_agent_id = receipt.owner.clone();
    task.task_id = "16800000-0000-4000-8000-000000000001".into();
    task.child = "16800000-0000-4000-8000-000000000002".into();
    task.mission = "16800000-0000-4000-8000-000000000003".into();
    store.take_changes();
    store
        .test_connection()
        .execute_batch(
            "CREATE TRIGGER reject_v2_after_task BEFORE UPDATE OF state ON auto_admission_receipts
             WHEN NEW.state='admitted' AND NEW.request_id='atomic-v2'
             BEGIN
               SELECT CASE
                 WHEN EXISTS(SELECT 1 FROM native_delegations WHERE task_id=NEW.task_id)
                 THEN RAISE(ABORT,'fixture v2 rollback after Task')
                 ELSE RAISE(ABORT,'fixture v2 Task absent')
               END;
             END;",
        )
        .unwrap();
    let error = store
        .admit_auto_task(&chosen, &task, &receipt.request_id, b"delegate", 13)
        .unwrap_err();
    assert!(
        error.contains("fixture v2 rollback after Task"),
        "la faute doit survenir après l'insertion réelle de Task : {error}"
    );
    assert_eq!(
        store
            .auto_admission_receipt(&receipt.owner, &receipt.request_id)
            .unwrap(),
        Some(chosen.clone()),
        "rollback du choix et de toutes ses métadonnées v2"
    );
    assert!(
        store.get(&task.task_id).unwrap().is_none(),
        "aucune Task après rollback"
    );
    assert!(
        store.take_changes().is_empty(),
        "aucune projection avant commit"
    );
    store
        .test_connection()
        .execute_batch("DROP TRIGGER reject_v2_after_task")
        .unwrap();
    drop(store);
    let store = DelegationStore::open(&path).unwrap();
    assert_eq!(
        store
            .auto_admission_receipt(&receipt.owner, &receipt.request_id)
            .unwrap(),
        Some(chosen.clone()),
        "choix v2 intact après réouverture sans la faute"
    );
    assert!(store.get(&task.task_id).unwrap().is_none());
    store
        .admit_auto_task(&chosen, &task, &receipt.request_id, b"delegate", 14)
        .unwrap();
    drop(store);
    let reopened = DelegationStore::open(&path).unwrap();
    let admitted = reopened
        .auto_admission_receipt(&receipt.owner, &receipt.request_id)
        .unwrap()
        .unwrap();
    assert_eq!(admitted.state, AutoAdmissionState::Admitted);
    assert_eq!(admitted.metadata, final_metadata);
    assert!(reopened.get(&task.task_id).unwrap().is_some());
    assert_eq!(
        reopened
            .admit_auto_task(&chosen, &task, &receipt.request_id, b"delegate", 15)
            .unwrap_err(),
        "admission_pending"
    );
    drop(reopened);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn spec163_choix_et_task_sont_durables_et_atomiques() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let receipt = AutoAdmissionReceipt {
        owner: "owner-163".into(),
        request_id: "request-atomic-163".into(),
        canonical: br#"{"operation":"auto_delegate","request_id":"request-atomic-163"}"#.to_vec(),
        metadata: br#"{"policy_revision":7,"candidates":["c01"]}"#.to_vec(),
        metadata_version: 1,
        state: AutoAdmissionState::Pending,
        coordinator_epoch: 1,
        revision: 0,
        policy_revision: 7,
        policy_digest: "a".repeat(64),
        candidate_id: None,
        delegate_canonical: None,
        mode: None,
        reason: None,
        task_id: None,
        refusal_code: None,
        created_at: 100,
        updated_at: 100,
    };
    assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
    let chosen = store
        .choose_auto_admission(
            &receipt.owner,
            &receipt.request_id,
            &receipt.canonical,
            "c01".into(),
            b"delegate-163".to_vec(),
            AutoAdmissionMode::Automatic,
            Some("profil autorisé".into()),
            101,
        )
        .unwrap();
    assert_eq!(chosen.state, AutoAdmissionState::Chosen);
    assert_eq!(
        store
            .choose_auto_admission(
                &receipt.owner,
                &receipt.request_id,
                &receipt.canonical,
                "c01".into(),
                b"delegate-163".to_vec(),
                AutoAdmissionMode::Automatic,
                None,
                102,
            )
            .unwrap_err(),
        "auto_admission_already_resolved"
    );

    let mut task = legacy_task151();
    task.owner = receipt.owner.clone();
    task.root_owner_agent_id = receipt.owner.clone();
    task.task_id = "16300000-0000-4000-8000-000000000001".into();
    task.child = "16300000-0000-4000-8000-000000000002".into();
    task.mission = "16300000-0000-4000-8000-000000000003".into();
    store
        .admit_auto_task(&chosen, &task, &receipt.request_id, b"delegate-163", 102)
        .unwrap();
    let admitted = store
        .auto_admission_receipt(&receipt.owner, &receipt.request_id)
        .unwrap()
        .unwrap();
    assert_eq!(admitted.state, AutoAdmissionState::Admitted);
    assert_eq!(admitted.task_id.as_deref(), Some(task.task_id.as_str()));
    assert!(store.get(&task.task_id).unwrap().is_some());
    assert_eq!(
        store
            .refuse_auto_admission(&chosen, "remote_prepare_invalid", 103)
            .unwrap_err(),
        "admission_pending",
        "le CAS stale ne ferme pas une Task déjà admise"
    );
    assert_eq!(
        store
            .auto_admission_receipt(&receipt.owner, &receipt.request_id)
            .unwrap()
            .unwrap()
            .state,
        AutoAdmissionState::Admitted
    );
}

#[test]
fn spec163_t026_compteurs_union_places_et_collision_native() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let owner = "owner-t026";
    let pending = auto_receipt163(owner, "auto-pending", 1, 10);
    let refused = auto_refusal163(owner, "auto-refused", 1, 11);
    let linked = auto_receipt163(owner, "auto-linked", 1, 12);
    assert!(store.insert_auto_admission_receipt(&pending).unwrap());
    assert!(store.insert_auto_admission_receipt(&refused).unwrap());
    assert!(store.insert_auto_admission_receipt(&linked).unwrap());
    assert!(
        !store
            .record_refusal(owner, &pending.request_id, b"native", "cwd_unavailable", 12)
            .unwrap(),
        "un refus natif ne double jamais une clé auto"
    );

    let chosen = store
        .choose_auto_admission(
            &linked.owner,
            &linked.request_id,
            &linked.canonical,
            "c01".into(),
            b"delegate-linked-t026".to_vec(),
            AutoAdmissionMode::Automatic,
            None,
            13,
        )
        .unwrap();
    let mut task = legacy_task151();
    task.owner = owner.into();
    task.root_owner_agent_id = owner.into();
    task.owner_instance = "instance-t026".into();
    task.mission = "mission-t026-linked".into();
    store
        .admit_auto_task(
            &chosen,
            &task,
            &linked.request_id,
            b"delegate-linked-t026",
            14,
        )
        .unwrap();

    assert!(
        store
            .record_refusal(owner, "native-refusal", b"native", "cwd_unavailable", 15)
            .unwrap()
    );
    let native_collision = auto_receipt163(owner, "native-refusal", 1, 16);
    assert_eq!(
        store
            .insert_auto_admission_receipt(&native_collision)
            .unwrap_err(),
        "auto_admission_reserved"
    );
    assert_eq!(
        store.admission_counts("instance-t026").unwrap(),
        (2, 1, 3),
        "refus natif historique hors compteur et reçu/Task lié compté une fois"
    );
    assert_eq!(
        store.admission_counts_for_owner(owner).unwrap(),
        (2, 2, 3),
        "les places auto pending et Task liée utilisent l'owner durable"
    );
}

#[test]
fn spec163_t026_purge_refus_auto_tombstone_et_retention_trente_jours() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let owner = "owner-purge-t026";
    let old = auto_refusal163(owner, "old", 1, 10);
    let recent = auto_refusal163(owner, "recent", 1, 20);
    let pending = auto_receipt163(owner, "pending", 1, 30);
    for receipt in [&old, &recent, &pending] {
        assert!(store.insert_auto_admission_receipt(receipt).unwrap());
    }
    assert_eq!(store.purge_auto_refusals_under_pressure(100, 3).unwrap(), 1);
    assert_eq!(
        store.auto_admission_receipt(owner, "old").unwrap_err(),
        "request_expired"
    );
    assert_eq!(
        store
            .by_agent_request(owner, "old", &old.canonical)
            .unwrap_err(),
        "request_expired"
    );
    assert_eq!(
        store.insert_auto_admission_receipt(&old).unwrap_err(),
        "request_expired"
    );
    assert_eq!(
        store.auto_admission_receipt(owner, "recent").unwrap(),
        Some(recent)
    );
    assert_eq!(
        store.auto_admission_receipt(owner, "pending").unwrap(),
        Some(pending)
    );

    let aged = auto_refusal163(owner, "aged", 1, 0);
    assert!(store.insert_auto_admission_receipt(&aged).unwrap());
    let thirty_days = 30 * 24 * 3600;
    assert_eq!(
        store
            .purge_expired_auto_admission_refusals(thirty_days, 128)
            .unwrap(),
        0,
        "le refus est conservé pendant les trente jours complets"
    );
    assert_eq!(
        store.auto_admission_receipt(owner, "aged").unwrap(),
        Some(aged.clone())
    );
    assert_eq!(
        store
            .purge_expired_auto_admission_refusals(thirty_days + 1, 128)
            .unwrap(),
        1
    );
    assert_eq!(
        store.auto_admission_receipt(owner, "aged").unwrap_err(),
        "request_expired"
    );
}

#[test]
fn spec163_t026_ne_purge_ni_vivants_ni_etat_non_committe() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let owner = "owner-vivants-t026";
    let pending = auto_receipt163(owner, "pending", 1, 30);
    let started = auto_receipt163(owner, "started", 1, 20);
    let chosen = auto_receipt163(owner, "chosen", 1, 10);
    let refused = auto_refusal163(owner, "refused", 1, 1);
    for receipt in [&pending, &started, &chosen, &refused] {
        assert!(store.insert_auto_admission_receipt(receipt).unwrap());
    }
    store.start_auto_decision(&started, 21).unwrap();
    store
        .choose_auto_admission(
            &chosen.owner,
            &chosen.request_id,
            &chosen.canonical,
            "c01".into(),
            b"delegate-vivant-t026".to_vec(),
            AutoAdmissionMode::Automatic,
            None,
            11,
        )
        .unwrap();
    assert_eq!(store.purge_auto_refusals_under_pressure(100, 4).unwrap(), 1);
    for request_id in ["pending", "started", "chosen"] {
        assert!(
            store
                .auto_admission_receipt(owner, request_id)
                .unwrap()
                .is_some()
        );
    }

    let rollback = auto_refusal163(owner, "rollback", 1, 0);
    assert!(store.insert_auto_admission_receipt(&rollback).unwrap());
    store
        .test_connection()
        .execute_batch(
            "CREATE TRIGGER reject_auto_purge BEFORE DELETE ON auto_admission_receipts
                 WHEN OLD.request_id='rollback' BEGIN SELECT RAISE(ABORT,'fixture rollback'); END;",
        )
        .unwrap();
    assert!(store.purge_auto_refusals_under_pressure(101, 4).is_err());
    assert_eq!(
        store.auto_admission_receipt(owner, "rollback").unwrap(),
        Some(rollback)
    );
    let tombstones: usize = store
            .test_connection()
            .query_row(
                "SELECT count(*) FROM native_delegation_tombstones WHERE owner=?1 AND request_id='rollback'",
                [owner],
                |row| row.get(0),
            )
            .unwrap();
    assert_eq!(tombstones, 0, "rollback SQL : ni purge ni fausse clôture");
}

#[test]
fn spec163_t048_reservation_precede_les_plafonds_et_rejoue_sans_double_compte() {
    let global = DelegationStore::open(Path::new(":memory:")).unwrap();
    let replay = auto_receipt163("owner-replay-t048", "replay", 1, 10);
    assert!(global.reserve_auto_admission_receipt(&replay, 10).unwrap());
    for index in 0..(MAX_ACTIVE_ADMISSIONS - 1) {
        let receipt = auto_receipt163(
            &format!("owner-global-{index:03}"),
            &format!("global-{index:03}"),
            1,
            11,
        );
        assert!(global.reserve_auto_admission_receipt(&receipt, 11).unwrap());
    }
    assert_eq!(
        global
            .admission_counts_for_owner("owner-replay-t048")
            .unwrap(),
        (MAX_ACTIVE_ADMISSIONS, 1, MAX_ACTIVE_ADMISSIONS),
    );
    assert!(
        !global.reserve_auto_admission_receipt(&replay, 12).unwrap(),
        "un rejeu exact est lu avant la garde globale pleine"
    );
    assert_eq!(
        global
            .reserve_auto_admission_receipt(
                &auto_receipt163("owner-overflow", "overflow", 1, 12),
                12,
            )
            .unwrap_err(),
        "task_limit"
    );

    let owner = DelegationStore::open(Path::new(":memory:")).unwrap();
    for index in 0..MAX_ACTIVE_ADMISSIONS_PER_OWNER {
        let receipt = auto_receipt163("owner-t048", &format!("owner-{index:02}"), 1, 20);
        assert!(owner.reserve_auto_admission_receipt(&receipt, 20).unwrap());
    }
    assert_eq!(
        owner
            .reserve_auto_admission_receipt(
                &auto_receipt163("owner-t048", "owner-overflow", 1, 21),
                21,
            )
            .unwrap_err(),
        "task_limit"
    );

    let retained = DelegationStore::open(Path::new(":memory:")).unwrap();
    for index in 0..MAX_RETAINED_TASKS {
        let receipt = auto_refusal163(
            &format!("owner-retained-{index:04}"),
            &format!("retained-{index:04}"),
            1,
            index as i64,
        );
        assert!(retained.insert_auto_admission_receipt(&receipt).unwrap());
    }
    let oldest = auto_refusal163("owner-retained-0000", "retained-0000", 1, 0);
    let fresh = auto_receipt163("owner-fresh-t048", "fresh", 1, 5_000);
    assert!(
        retained
            .reserve_auto_admission_receipt(&fresh, 5_000)
            .unwrap()
    );
    assert_eq!(
        retained
            .auto_admission_receipt(&oldest.owner, &oldest.request_id)
            .unwrap_err(),
        "request_expired",
        "la purge sous pression écrit le tombstone avant la nouvelle réservation"
    );
    assert_eq!(
        retained
            .admission_counts_for_owner("owner-fresh-t048")
            .unwrap()
            .2,
        MAX_RETAINED_TASKS
    );

    let rollback = DelegationStore::open(Path::new(":memory:")).unwrap();
    rollback
            .test_connection()
            .execute_batch(
                "CREATE TRIGGER reject_auto_reservation BEFORE INSERT ON auto_admission_receipts
                 WHEN NEW.request_id='rollback' BEGIN SELECT RAISE(ABORT,'fixture reservation rollback'); END;",
            )
            .unwrap();
    let receipt = auto_receipt163("owner-rollback-t048", "rollback", 1, 1);
    assert!(
        rollback
            .reserve_auto_admission_receipt(&receipt, 1)
            .is_err()
    );
    assert_eq!(
        rollback
            .auto_admission_receipt(&receipt.owner, &receipt.request_id)
            .unwrap(),
        None
    );
    assert_eq!(
        rollback.admission_counts_for_owner(&receipt.owner).unwrap(),
        (0, 0, 0),
        "erreur SQL : aucune clé ni compteur fantôme"
    );
}

#[test]
fn spec163_t027_lot_epoch_borne_reutilise_les_cas_existants() {
    let empty = DelegationStore::open(Path::new(":memory:")).unwrap();
    assert_eq!(empty.next_auto_coordinator_epoch().unwrap(), 1);

    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let owner = "owner-epoch-t027";
    let started = auto_receipt163(owner, "started", 1, 10);
    let pending = auto_receipt163(owner, "pending", 1, 30);
    let chosen = auto_receipt163(owner, "chosen", 2, 20);
    let current = auto_receipt163(owner, "current", 3, 1);
    let terminal = auto_refusal163(owner, "terminal", 1, 0);
    for receipt in [&started, &pending, &chosen, &current, &terminal] {
        assert!(store.insert_auto_admission_receipt(receipt).unwrap());
    }
    store.start_auto_decision(&started, 11).unwrap();
    store
        .choose_auto_admission(
            &chosen.owner,
            &chosen.request_id,
            &chosen.canonical,
            "c01".into(),
            b"delegate-epoch-t027".to_vec(),
            AutoAdmissionMode::Fallback,
            None,
            21,
        )
        .unwrap();
    let batch = store.auto_admission_receipts_before_epoch(3, 128).unwrap();
    assert_eq!(
        batch
            .iter()
            .map(|receipt| receipt.request_id.as_str())
            .collect::<Vec<_>>(),
        vec!["started", "pending", "chosen"]
    );
    assert_eq!(
        store
            .auto_admission_receipts_before_epoch(3, 2)
            .unwrap()
            .len(),
        2
    );
    let coordinator_epoch = store.next_auto_coordinator_epoch().unwrap();
    assert_eq!(
        coordinator_epoch, 4,
        "strictement après les reçus existants"
    );
    for request_id in ["coordinator-a", "coordinator-b"] {
        assert!(
            store
                .insert_auto_admission_receipt(&auto_receipt163(
                    owner,
                    request_id,
                    coordinator_epoch,
                    40,
                ))
                .unwrap()
        );
    }
    assert_eq!(
        store.next_auto_coordinator_epoch().unwrap(),
        5,
        "l'époque portée par tous les nouveaux reçus avance le prochain démarrage"
    );
    assert_eq!(
        store
            .auto_admission_receipts_before_epoch(5, 128)
            .unwrap()
            .iter()
            .map(|receipt| receipt.request_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "started",
            "pending",
            "chosen",
            "current",
            "coordinator-a",
            "coordinator-b"
        ]
    );
    assert_eq!(
        store
            .auto_admission_receipts_before_epoch(3, 0)
            .unwrap_err(),
        "auto_admission_invalid"
    );
}

#[test]
fn spec163_t027_reouverture_disque_conserve_recu_choix_et_lien_task() {
    let dir = std::env::temp_dir().join(format!("bridget-auto163-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("admission.sqlite");
    let owner = "owner-reopen-t027";
    let receipt = auto_receipt163(owner, "reopen", 1, 10);

    let store = DelegationStore::open(&path).unwrap();
    assert!(store.insert_auto_admission_receipt(&receipt).unwrap());
    drop(store);

    let store = DelegationStore::open(&path).unwrap();
    let pending = store
        .auto_admission_receipt(owner, "reopen")
        .unwrap()
        .unwrap();
    assert_eq!(pending.state, AutoAdmissionState::Pending);
    let _started = store.start_auto_decision(&pending, 11).unwrap();
    drop(store);

    let store = DelegationStore::open(&path).unwrap();
    let started = store
        .auto_admission_receipt(owner, "reopen")
        .unwrap()
        .unwrap();
    assert_eq!(started.state, AutoAdmissionState::DecisionStarted);
    let chosen = store
        .choose_auto_admission(
            owner,
            "reopen",
            &started.canonical,
            "c01".into(),
            b"delegate-reopen-t027".to_vec(),
            AutoAdmissionMode::Automatic,
            None,
            12,
        )
        .unwrap();
    drop(store);

    let store = DelegationStore::open(&path).unwrap();
    let chosen_after_reopen = store
        .auto_admission_receipt(owner, "reopen")
        .unwrap()
        .unwrap();
    assert_eq!(chosen_after_reopen, chosen);
    let mut task = legacy_task151();
    task.owner = owner.into();
    task.root_owner_agent_id = owner.into();
    task.task_id = "16300000-0000-4000-8000-000000000027".into();
    task.child = "16300000-0000-4000-8000-000000000028".into();
    task.mission = "16300000-0000-4000-8000-000000000029".into();
    store
        .test_connection()
        .execute_batch(
            "CREATE TRIGGER reject_auto_task BEFORE INSERT ON native_delegations
                 WHEN NEW.task_id='16300000-0000-4000-8000-000000000027'
                 BEGIN SELECT RAISE(ABORT,'fixture admission rollback'); END;",
        )
        .unwrap();
    assert!(
        store
            .admit_auto_task(
                &chosen_after_reopen,
                &task,
                "reopen",
                b"delegate-reopen-t027",
                13,
            )
            .is_err()
    );
    assert_eq!(
        store
            .auto_admission_receipt(owner, "reopen")
            .unwrap()
            .unwrap(),
        chosen_after_reopen,
        "le rollback disque conserve le reçu chosen"
    );
    assert!(store.get(&task.task_id).unwrap().is_none());
    assert!(
        store.take_changes().is_empty(),
        "aucune projection n'est publiée avant le commit commun"
    );
    store
        .test_connection()
        .execute_batch("DROP TRIGGER reject_auto_task;")
        .unwrap();
    store
        .admit_auto_task(
            &chosen_after_reopen,
            &task,
            "reopen",
            b"delegate-reopen-t027",
            13,
        )
        .unwrap();
    drop(store);

    let store = DelegationStore::open(&path).unwrap();
    let admitted = store
        .auto_admission_receipt(owner, "reopen")
        .unwrap()
        .unwrap();
    assert_eq!(admitted.state, AutoAdmissionState::Admitted);
    assert_eq!(admitted.task_id.as_deref(), Some(task.task_id.as_str()));
    assert_eq!(admitted.canonical, receipt.canonical);
    assert_eq!(admitted.policy_digest, receipt.policy_digest);
    assert_eq!(
        store
            .by_agent_request(owner, "reopen", b"delegate-reopen-t027")
            .unwrap()
            .unwrap()
            .task_id,
        task.task_id
    );
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn spec163_projection_ne_publie_qu_apres_commit_externe() {
    let store = DelegationStore::open(Path::new(":memory:")).unwrap();
    let task = legacy_task151();
    let tx =
        rusqlite::Transaction::new_unchecked(&store.conn, rusqlite::TransactionBehavior::Immediate)
            .unwrap();
    let (stamped, _mutation) = store
        .insert_projected_in(&tx, &task, "request-163", b"canonical-163")
        .unwrap();
    drop(tx);
    assert!(store.get(&stamped.task_id).unwrap().is_none());
    assert!(store.take_changes().is_empty());

    let tx =
        rusqlite::Transaction::new_unchecked(&store.conn, rusqlite::TransactionBehavior::Immediate)
            .unwrap();
    let (stamped, mutation) = store
        .insert_projected_in(&tx, &task, "request-163", b"canonical-163")
        .unwrap();
    tx.commit().unwrap();
    assert!(store.take_changes().is_empty());
    store.publish_projection(stamped.root_owner(), mutation);
    assert!(!store.take_changes().is_empty());
}
#[test]
fn spec234_pending_keeps_cancelled_unpublished_view_until_native_conservation() {
    let root = std::env::temp_dir().join(format!("bridget234-pending-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let store = DelegationStore::open(&root.join("bridget.db")).unwrap();
    let mut task = legacy_task151();
    task.state = "cancelled".into();
    task.workspace = None;
    task.cleanup_done = false;
    task.commit_view_prepare=Some(serde_json::from_value(serde_json::json!({
            "worker":{"machine_id":"ksstore","worker_id":"worker-ksstore","provisioning_revision":1},
            "command":{"task_id":task.task_id,"command_id":task.task_id,"instance_id":"","generation":0},
            "prepare_digest":"a".repeat(64)})).unwrap());
    store.insert(&task, "request234", b"canonical234").unwrap();
    assert_eq!(
        store.pending().unwrap().len(),
        1,
        "copie non publiée : reprendre la conservation native après retour du worker"
    );
    task.cleanup_done = true;
    store.save(&task).unwrap();
    assert!(store.pending().unwrap().is_empty());
    task.cleanup_done = false;
    task.commit_view_prepare = None;
    store.save(&task).unwrap();
    assert!(
        store.pending().unwrap().is_empty(),
        "annulation ordinaire sans espace : comportement conservé"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
