use super::*;
fn c19_master_file_fixture() -> (
    DaemonState,
    DaemonConfig,
    Task,
    bridget_transport::protocol::RemoteResultEnvelope,
) {
    use crate::delegation::{
        CommitViewRecord, ViewBinding, WorkspaceRecord, WorkspaceState, WorkspaceStrategy,
    };
    let (state, config, mut task, result) = super::tests::remote162_capture_fixture();
    let mut prepare = result.identity.command.clone();
    prepare.instance_id.clear();
    prepare.generation = 0;
    let binding = ViewBinding {
        worker: result.worker.clone(),
        command: prepare,
        prepare_digest: "a".repeat(64),
    };
    task.commit_view_prepare = Some(binding.clone());
    task.workspace = Some(WorkspaceRecord {
        strategy: WorkspaceStrategy::CommitView,
        commit_view: Some(CommitViewRecord {
            proof_root: "/private/proofs".into(),
            intent_digest: "d".repeat(64),
            binding,
        }),
        state: WorkspaceState::Kept,
        toplevel: "/private/source".into(),
        path: "/private/view".into(),
        branch: String::new(),
        base_commit: "a".repeat(40),
        head_commit: None,
        branch_commit: None,
        head_on_branch: None,
        commit_count: None,
        commits: vec![],
        truncated: false,
        dirty: Some(false),
    });
    state.delegation_store.save(&task).unwrap();
    (state, config, task, result)
}

#[test]
fn spec221_c19_file_context_binds_parent_child_round_revocation_and_pause_without_mutation() {
    let (mut state, config, task, result) = c19_master_file_fixture();
    let before =
        serde_json::to_vec(&state.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap();
    assert_eq!(
        commit_file_context(&state, "conn-1", Some(&task.task_id), None)
            .unwrap()
            .1,
        result.identity
    );
    assert!(matches!(
        commit_file_context(&state, "conn-1", None, None),
        Err("commit_file_child_unlinked")
    ));
    // L'enfant courant lit sa propre vue sans droit de créer d'autres enfants.
    let instance = task.child_instance.clone().unwrap();
    assert!(matches!(
        handle_register_with_channel(
            "file-child",
            2,
            "native-test".into(),
            task.child.clone(),
            Some("cartae".into()),
            Some("codex_app_server".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Cli),
            None,
            Some("linux".into()),
            Some(instance.clone()),
            None,
            false,
            Some(true),
            &mut state,
            false
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let (socket, peer) = UnixStream::pair().unwrap();
    state.connections.insert(
        "file-child".into(),
        Arc::new(Mutex::new(BufWriter::new(socket))),
    );
    state
        .conn_instances
        .insert("file-child".into(), instance.clone());
    assert_eq!(
        commit_file_context(&state, "file-child", None, None)
            .unwrap()
            .1,
        result.identity
    );
    assert!(matches!(
        commit_file_context(&state, "file-child", Some(&task.task_id), None),
        Err("task_unavailable")
    ));
    for case in 0..9 {
        let (case_state, case_config, mut wrong, _) = c19_master_file_fixture();
        match case {
            0 => wrong.round += 1,
            1 => wrong.child_instance = Some("other-instance".into()),
            2 => wrong.remote_result_slots[0].identity.execution_generation += 1,
            3 => wrong
                .remote_result_slots
                .push(wrong.remote_result_slots[0].clone()),
            4 => wrong.cleanup_done = true,
            5 => wrong.guard_released = true,
            6 => wrong.state = "result_available".into(),
            7 => wrong.mission_deadline_at = Some(unix_timestamp() - 1),
            _ => {
                wrong.workspace.as_mut().unwrap().strategy =
                    crate::delegation::WorkspaceStrategy::Worktree
            }
        }
        match case_state.delegation_store.save(&wrong) {
            Ok(()) => assert!(
                commit_file_context(&case_state, "conn-1", Some(&wrong.task_id), None).is_err(),
                "case {case}"
            ),
            Err(code) => {
                assert!(
                    matches!(case, 2 | 3),
                    "reçu refusé avant la lecture : case {case}, {code}"
                );
                assert_eq!(
                    code,
                    if case == 2 {
                        "remote_result_slots_stale"
                    } else {
                        "remote_result_slot_invalid"
                    }
                );
            }
        }
        drop(case_state);
        super::tests::cleanup(&case_config);
    }
    let control = crate::referent_control::read(state.store.connection()).unwrap();
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "file-test-pause",
            expected_generation: control.generation,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: Some("test"),
            actor: "test",
            now: unix_timestamp(),
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    assert!(matches!(
        commit_file_context(&state, "conn-1", Some(&task.task_id), None),
        Err("commit_file_task_paused")
    ));
    let control = crate::referent_control::read(state.store.connection()).unwrap();
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "file-test-resume",
            expected_generation: control.generation,
            paused: Some(false),
            auto_objectives_cap: None,
            reason: Some("test"),
            actor: "test",
            now: unix_timestamp(),
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    state
        .delegation_store
        .grant(
            &instance,
            Path::new("/private/view"),
            SpawnPosture::Discovery,
            true,
        )
        .unwrap();
    assert!(matches!(
        commit_file_context(&state, "conn-1", Some(&task.task_id), None),
        Err("commit_file_revoked")
    ));
    assert_eq!(
        serde_json::to_vec(&state.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap(),
        before
    );
    assert!(state.pending_worker_responses.is_empty());
    drop(peer);
    drop(state);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_file_roundtrip_gates_old_peer_and_rechecks_task_and_execution_after_response() {
    use bridget_transport::protocol::*;
    use std::io::{BufRead, Read};
    let (mut state, config, task, result) = c19_master_file_fixture();
    let raw = json!({"agents":{"native-test":{"command":"/bin/sh","args":["app-server"],
            "protocol":"codex_app_server","forbidden_env":[]}},
            "machines":{"cartae":{"enabled":true,"worker_id":result.worker.worker_id,
                "provisioning_revision":result.worker.provisioning_revision,"credential_fingerprint":"a".repeat(64),
                "allowed_agent_types":["native-test"],"mappings":{}}}});
    state.registry =
        AgentRegistry::from_json(&raw.to_string(), state.registry.source().to_path_buf()).unwrap();
    let (socket, mut peer) = UnixStream::pair().unwrap();
    peer.set_nonblocking(true).unwrap();
    state.connections.insert(
        "file-worker".into(),
        Arc::new(Mutex::new(BufWriter::new(socket))),
    );
    state.remote_workers.insert(
        "cartae".into(),
        RemoteWorkerConnection {
            identity: result.worker.clone(),
            conn_id: "file-worker".into(),
            epoch: "file-epoch".into(),
            capabilities: vec![],
            catalogues: HashMap::new(),
            catalogue_refresh_id: "file-epoch".into(),
            catalogue_refresh_at: None,
            contact: WorkerContactState::Connected,
            last_contact: Instant::now(),
            last_contact_at: unix_timestamp(),
        },
    );
    let before =
        serde_json::to_vec(&state.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap();
    let shared = Arc::new(Mutex::new(state));
    let refused = read_commit_file("conn-1", Some(&task.task_id), "large.bin", None, &shared);
    assert_eq!(refused["detail"]["worker_reason"], "capability_required");
    let mut byte = [0u8; 1];
    assert_eq!(
        peer.read(&mut byte).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert_eq!(
        serde_json::to_vec(
            &shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        before
    );
    shared
        .lock()
        .unwrap()
        .remote_workers
        .get_mut("cartae")
        .unwrap()
        .capabilities
        .push(WorkerCapability::WorkspaceViewV1);
    peer.set_nonblocking(false).unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    let incoming = Arc::clone(&shared);
    let execution = result.identity.clone();
    let task_id = task.task_id.clone();
    let responder = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(peer);
        for case in 0..4 {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = serde_json::from_str(&line).unwrap()
            else {
                panic!("demande worker attendue");
            };
            assert_eq!(
                request.request,
                WorkerRequestOperation::ReadCommitViewFile {
                    execution: execution.clone(),
                    path: "large.bin".into()
                }
            );
            let mut returned = execution.clone();
            if case == 2 {
                returned.execution_id = "foreign-file-execution".into();
            }
            if case == 3 {
                let st = incoming.lock().unwrap();
                let mut ended = st.delegation_store.get(&task_id).unwrap().unwrap();
                ended.state = "failed".into();
                st.delegation_store.save(&ended).unwrap();
            }
            let response = WorkerResponse {
                version: request.version,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                command: request.command,
                response: WorkerResponseOperation::CommitViewFile {
                    execution: returned,
                    path: "large.bin".into(),
                    available: case != 1,
                    reason: (case == 1).then_some(WorkerRefusal::ResourceLimit),
                    failure: (case == 1).then_some(CommitViewFailure::OperationCapacity),
                },
            };
            response.validate().unwrap();
            assert!(
                super::super::handle_worker_message(
                    &incoming,
                    "file-worker",
                    WrapperToDaemon::WorkerResponse { response }
                )
                .is_none()
            );
        }
    });
    let available = read_commit_file("conn-1", Some(&task.task_id), "large.bin", None, &shared);
    assert_eq!(available["status"], "available");
    assert_eq!(available["file"], "/private/view/large.bin");
    assert_eq!(available["base_commit"], "a".repeat(40));
    assert_eq!(available["machine_id"], "cartae");
    assert!(!available.to_string().contains("/private/proofs"));
    assert_eq!(
        serde_json::to_vec(
            &shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        before
    );
    let occupied = read_commit_file("conn-1", Some(&task.task_id), "large.bin", None, &shared);
    assert_eq!(occupied["detail"]["cause_kind"], "commit_view");
    assert!(
        occupied["message"]
            .as_str()
            .unwrap()
            .contains("demander de nouveau le même fichier")
    );
    assert!(!occupied["message"].as_str().unwrap().contains("  "));
    let foreign = read_commit_file("conn-1", Some(&task.task_id), "large.bin", None, &shared);
    assert_eq!(foreign["code"], "commit_file_response_unavailable");
    let ended = read_commit_file("conn-1", Some(&task.task_id), "large.bin", None, &shared);
    assert_eq!(ended["code"], "commit_file_task_not_active");
    responder.join().unwrap();
    assert!(shared.lock().unwrap().pending_worker_responses.is_empty());
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_file_worker_refusals_keep_observed_code_and_specific_next_action() {
    use bridget_transport::protocol::WorkerRefusal::*;
    let (state, config, task, _) = c19_master_file_fixture();
    for (reason, fragment) in [
        (CapabilityRequired, "WorkspaceViewV1"),
        (WorkerUnavailable, "lien natif"),
        (StaleConnection, "lien natif"),
        (Unauthorized, "autorisation native"),
        (PlacementMismatch, "placement"),
        (CommandMismatch, "génération"),
        (ResourceLimit, "borne"),
        (StoreUnavailable, "store"),
    ] {
        let refused = commit_file_worker_refusal(reason, &task);
        assert_eq!(refused["detail"]["cause"], json!(reason));
        assert_eq!(refused["detail"]["worker_reason"], json!(reason));
        assert_eq!(refused["detail"]["cause_kind"], "worker");
        assert_eq!(refused["detail"]["machine_id"], "cartae");
        let message = refused["message"].as_str().unwrap();
        assert!(message.contains(fragment) && message.contains("cette lecture sur cette machine"));
        assert!(message.contains("Les autres travaux autorisés suivent leur propre état"));
        assert!(
            refused["detail"]["human_action"]
                .as_str()
                .unwrap()
                .contains("utilisateur")
        );
        assert!(!refused.to_string().contains("/private/proofs"));
    }
    drop(state);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_preparation_capacity_never_recommends_replaying_a_file_read() {
    use bridget_transport::protocol::{CommitViewFailure, WorkerRefusal};
    let (state, config, mut task, _) = c19_master_file_fixture();
    task.error = Some(format!(
        "remote_start_failed:{}",
        workspace_preparation::view_error_code(
            WorkerRefusal::ResourceLimit,
            CommitViewFailure::OperationCapacity
        )
    ));
    let mut view = json!({});
    workspace_preparation::project(&task, &mut view);
    let message = view["error_message"].as_str().unwrap();
    assert!(!message.contains("fichier"));
    assert!(message.contains("relire l'état de la même tâche"));
    assert!(
        view["error_recovery"]
            .as_str()
            .unwrap()
            .contains("ne relance pas la préparation")
    );
    drop(state);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_view_retention_distinguishes_file_report_from_git_without_mutation() {
    use crate::delegation::{
        CommitViewRecord, ViewBinding, WorkspaceRecord, WorkspaceState, WorkspaceStrategy,
    };
    let (state, config, mut task, _) = super::tests::remote162_capture_fixture();
    task.state = "failed".into();
    task.error = None;
    task.cleanup_done = true;
    task.workspace_report_mission = Some(task.mission.clone());
    let reference = CommitViewRecord {
        proof_root: "/private/c19-proof".into(),
        intent_digest: "d".repeat(64),
        binding: ViewBinding {
            worker: bridget_transport::protocol::WorkerIdentity {
                machine_id: "fixture".into(),
                worker_id: "worker".into(),
                provisioning_revision: 1,
            },
            command: bridget_transport::protocol::WorkerCommandIdentity {
                task_id: task.task_id.clone(),
                command_id: "prepare".into(),
                instance_id: String::new(),
                generation: 0,
            },
            prepare_digest: "a".repeat(64),
        },
    };
    task.workspace = Some(WorkspaceRecord {
        strategy: WorkspaceStrategy::CommitView,
        commit_view: Some(reference),
        state: WorkspaceState::Kept,
        toplevel: "/private/source".into(),
        path: "/private/view".into(),
        branch: String::new(),
        base_commit: "a".repeat(40),
        head_commit: None,
        branch_commit: None,
        head_on_branch: None,
        commit_count: None,
        commits: vec![],
        truncated: false,
        dirty: Some(false),
    });
    for (dirty, expected) in [
        (Some(false), "conforme"),
        (Some(true), "divergence constatée"),
        (None, "indisponible"),
    ] {
        let mut result = task.clone();
        result.workspace.as_mut().unwrap().dirty = dirty;
        result.result = Some("Résultat conservé".into());
        append_workspace_result(&mut result);
        let body = result.result.as_ref().unwrap();
        assert!(body.contains(&format!("vue de lecture du commit {}", "a".repeat(40))));
        assert!(body.contains(&format!("dernier contrôle des fichiers : {expected}")));
        assert!(body.contains("dossier conservé : /private/view"));
        assert!(body.contains("Les autres travaux autorisés suivent leur propre état"));
        for invented in [
            "branche ",
            "commit(s)",
            "worktree conservé",
            "tête sur la branche",
        ] {
            assert!(!body.contains(invented));
        }
        let mut contradictory = result.clone();
        contradictory.workspace.as_mut().unwrap().head_on_branch = Some(true);
        contradictory.result = Some("Résultat".into());
        append_workspace_result(&mut contradictory);
        assert!(
            contradictory
                .result
                .unwrap()
                .contains("dernier contrôle des fichiers : indisponible")
        );
    }
    let before = serde_json::to_value(&task).unwrap();
    let view = task_view(&task, None, true);
    assert_eq!(view["workspace"]["retention"]["code"], "review_required");
    assert!(
        view["workspace"]["retention"]["recovery"]
            .as_str()
            .unwrap()
            .contains("retrait natif")
    );
    assert_eq!(serde_json::to_value(&task).unwrap(), before);
    task.workspace.as_mut().unwrap().dirty = Some(true);
    let view = task_view(&task, None, true);
    assert_eq!(view["workspace"]["retention"]["code"], "changes_present");
    assert!(
        view["workspace"]["retention"]["recovery"]
            .as_str()
            .unwrap()
            .contains("ne régénérez pas")
    );
    task.workspace.as_mut().unwrap().dirty = Some(false);
    task.workspace.as_mut().unwrap().head_on_branch = Some(true);
    assert_eq!(
        task_view(&task, None, true)["workspace"]["retention"]["code"],
        "report_unavailable"
    );
    task.workspace.as_mut().unwrap().head_on_branch = None;
    let known_reference = task.workspace.as_ref().unwrap().commit_view.clone();
    task.workspace.as_mut().unwrap().commit_view = None;
    assert_eq!(
        task_view(&task, None, true)["workspace"]["retention"]["code"],
        "report_unavailable"
    );
    task.cleanup_done = false;
    assert_eq!(
        task_view(&task, None, true)["workspace"]["retention"]["code"],
        "stop_unconfirmed"
    );
    task.workspace.as_mut().unwrap().commit_view = known_reference;
    task.guarded = true;
    task.guard_released = false;
    assert_eq!(
        task_view(&task, None, true)["workspace"]["retention"]["code"],
        "stop_unconfirmed"
    );
    task.guard_released = true;
    let pending = task_view(&task, None, true);
    assert_eq!(
        pending["workspace"]["retention"]["code"],
        "view_conservation_unconfirmed"
    );
    assert!(
        pending["workspace"]["retention"]["recovery"]
            .as_str()
            .unwrap()
            .contains("cause précise")
    );
    assert!(
        pending["workspace"]["retention"]["recovery"]
            .as_str()
            .unwrap()
            .contains("Les autres travaux")
    );
    task.state = "result_available".into();
    task.result_sent = false;
    task.idle_deadline_at = None;
    assert_eq!(
        task_view(&task, None, true)["workspace"]["retention"]["code"],
        "result_delivery_pending"
    );

    drop(state);
    super::tests::cleanup(&config);
}

#[test]
fn spec235_late_conservation_refusal_preserves_channel_and_task_guards() {
    use bridget_transport::protocol::*;
    let (mut state, config, mut task, _) = super::tests::remote162_capture_fixture();
    let worker = remote_worker_identity(task.remote_placement.as_ref().unwrap());
    let command = WorkerCommandIdentity {
        command_id: task.task_id.clone(),
        task_id: task.task_id.clone(),
        instance_id: String::new(),
        generation: 0,
    };
    task.state = "cancelled".into();
    task.guarded = false;
    task.cleanup_done = false;
    task.workspace = None;
    task.commit_view_prepare = Some(crate::delegation::ViewBinding {
        worker: worker.clone(),
        command: command.clone(),
        prepare_digest: "a".repeat(64),
    });
    state.delegation_store.save(&task).unwrap();
    state.remote_workers.insert(
        "cartae".into(),
        RemoteWorkerConnection {
            identity: worker.clone(),
            conn_id: "late235".into(),
            epoch: "epoch235".into(),
            capabilities: vec![WorkerCapability::WorkspaceViewV1],
            catalogues: HashMap::new(),
            catalogue_refresh_id: "epoch235".into(),
            catalogue_refresh_at: None,
            contact: WorkerContactState::Connected,
            last_contact: Instant::now(),
            last_contact_at: unix_timestamp(),
        },
    );
    let shared = Arc::new(Mutex::new(state));
    let response = WorkerResponse {
        version: REMOTE_WORKER_VERSION,
        correlation_id: "expired235".into(),
        connection_epoch: "epoch235".into(),
        worker: worker.clone(),
        command: command.clone(),
        response: WorkerResponseOperation::Rejected {
            reason: WorkerRefusal::UnknownExecution,
        },
    };
    let before = serde_json::to_vec(
        &shared
            .lock()
            .unwrap()
            .delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let deliver = |response: WorkerResponse| {
        super::super::handle_worker_message(
            &shared,
            "late235",
            WrapperToDaemon::WorkerResponse { response },
        )
    };
    assert!(
        deliver(response.clone()).is_none(),
        "un refus tardif connu ne ferme pas le canal"
    );
    assert!(
        serde_json::to_vec(
            &shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap()
            == before
    );
    for case in 0..4 {
        let mut foreign = response.clone();
        match case {
            0 => foreign.command.command_id = "foreign235".into(),
            1 => foreign.command.task_id = "foreign235".into(),
            2 => foreign.connection_epoch = "old235".into(),
            _ => foreign.worker.worker_id = "foreign235".into(),
        }
        assert!(
            matches!(
                deliver(foreign),
                Some(DaemonToWrapper::WorkerRejected { .. })
            ),
            "case{case}"
        );
    }
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    shared.lock().unwrap().pending_worker_responses.insert(
        response.correlation_id.clone(),
        super::super::PendingWorkerResponse {
            conn_id: "late235".into(),
            epoch: "epoch235".into(),
            worker,
            command,
            reply: tx,
        },
    );
    assert!(deliver(response.clone()).is_none());
    assert_eq!(
        rx.try_recv().unwrap(),
        response.response,
        "une attente encore présente reçoit le refus"
    );
    task.state = "working".into();
    shared.lock().unwrap().delegation_store.save(&task).unwrap();
    assert!(matches!(
        deliver(response.clone()),
        Some(DaemonToWrapper::WorkerRejected {
            reason: WorkerRefusal::CommandMismatch
        })
    ));
    // La réservation est immuable après admission : vérifier la garde
    // sans demander à save() de modifier ce fait protégé de la fixture.
    task.state = "cancelled".into();
    task.guarded = true;
    task.guard_released = false;
    assert!(!super::super::commit_view_conservation_matches(
        &task,
        &response.worker,
        &response.command,
        &task.commit_view_binding().unwrap().prepare_digest
    ));
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_master_conservation_requires_closed_noncontinuable_task_and_exact_reference() {
    use crate::delegation::{
        CommitViewRecord, ViewBinding, WorkspaceRecord, WorkspaceState, WorkspaceStrategy,
    };
    use std::io::Read;
    let (mut state, config, mut task, _) = super::tests::remote162_capture_fixture();
    let worker = remote_worker_identity(task.remote_placement.as_ref().unwrap());
    let command = bridget_transport::protocol::WorkerCommandIdentity {
        task_id: task.task_id.clone(),
        command_id: "original-view-preparation".into(),
        instance_id: String::new(),
        generation: 0,
    };
    let digest = "a".repeat(64);
    task.state = "result_available".into();
    task.cleanup_done = true;
    task.result_sent = false;
    task.idle_deadline_at = None;
    task.workspace_report_mission = Some(task.mission.clone());
    task.guarded = false;
    task.guard_released = false;
    task.workspace = Some(WorkspaceRecord {
        strategy: WorkspaceStrategy::CommitView,
        state: WorkspaceState::Kept,
        commit_view: Some(CommitViewRecord {
            proof_root: "/private/proof".into(),
            intent_digest: "b".repeat(64),
            binding: ViewBinding {
                worker: worker.clone(),
                command: command.clone(),
                prepare_digest: digest.clone(),
            },
        }),
        toplevel: "/private/source".into(),
        path: "/private/view".into(),
        branch: String::new(),
        base_commit: "c".repeat(40),
        head_commit: None,
        branch_commit: None,
        head_on_branch: None,
        commit_count: None,
        commits: vec![],
        truncated: false,
        dirty: None,
    });
    let check = |task: &Task| {
        super::super::commit_view_conservation_matches(task, &worker, &command, &digest)
    };
    assert!(check(&task));
    let before = serde_json::to_value(&task).unwrap();
    for case in 0..8 {
        let mut wrong = task.clone();
        match case {
            0 => wrong.workspace_report_mission = None,
            1 => {
                wrong.result_sent = true;
                wrong.idle_deadline_at = Some(unix_timestamp() + 60);
            }
            2 => wrong.state = "working".into(),
            3 => {
                wrong.guarded = true;
                wrong.guard_released = false;
            }
            4 => wrong.workspace.as_mut().unwrap().strategy = WorkspaceStrategy::Worktree,
            5 => wrong.workspace.as_mut().unwrap().commit_view = None,
            6 => {
                wrong
                    .workspace
                    .as_mut()
                    .unwrap()
                    .commit_view
                    .as_mut()
                    .unwrap()
                    .binding
                    .prepare_digest = "f".repeat(64)
            }
            _ => wrong.task_id = "other-task".into(),
        }
        assert!(!check(&wrong), "case{case}");
    }
    let mut other_worker = worker.clone();
    other_worker.worker_id = "other-worker".into();
    assert!(!super::super::commit_view_conservation_matches(
        &task,
        &other_worker,
        &command,
        &digest
    ));
    let mut other_command = command.clone();
    other_command.command_id = "other-preparation".into();
    assert!(!super::super::commit_view_conservation_matches(
        &task,
        &worker,
        &other_command,
        &digest
    ));
    assert_eq!(serde_json::to_value(&task).unwrap(), before);
    // Ancien pair : refus avant écriture de la moindre trame. La machine
    // est désactivée, sans réactivation : seule la conservation d'une Task
    // connue serait autorisée si la capacité était présente.
    let path = state.registry.source().to_path_buf();
    // La fixture initiale charge son registre en mémoire, sans fichier.
    let raw = json!({"agents":{"native-test":{"command":"/bin/sh","args":["app-server"],
            "protocol":"codex_app_server","forbidden_env":[]}},
            "machines":{"cartae":{"enabled":false,"worker_id":worker.worker_id,
                "provisioning_revision":worker.provisioning_revision,"credential_fingerprint":"a".repeat(64),
                "allowed_agent_types":["native-test"],"mappings":{}}}});

    state.registry = AgentRegistry::from_json(&raw.to_string(), &path).unwrap();
    task.cleanup_done = false;
    assert!(check(&task), "la clôture de la vue précède cleanup_done");
    state.delegation_store.save(&task).unwrap();
    let (socket, mut peer) = UnixStream::pair().unwrap();
    peer.set_nonblocking(true).unwrap();
    state.connections.insert(
        "view-worker".into(),
        Arc::new(Mutex::new(BufWriter::new(socket))),
    );
    state.remote_workers.insert(
        "cartae".into(),
        RemoteWorkerConnection {
            identity: worker.clone(),
            conn_id: "view-worker".into(),
            epoch: "view-epoch".into(),
            capabilities: vec![],
            catalogues: HashMap::new(),
            catalogue_refresh_id: "view-epoch".into(),
            catalogue_refresh_at: None,
            contact: bridget_transport::protocol::WorkerContactState::Connected,
            last_contact: Instant::now(),
            last_contact_at: unix_timestamp(),
        },
    );
    let shared = Arc::new(Mutex::new(state));
    let operation = bridget_transport::protocol::WorkerRequestOperation::ConserveCommitView {
        prepare_digest: digest.clone(),
    };
    assert_eq!(
        remote_worker_roundtrip(
            &shared,
            &worker,
            &command,
            operation.clone(),
            Instant::now() + Duration::from_secs(1)
        ),
        Err(bridget_transport::protocol::WorkerRefusal::CapabilityRequired)
    );
    let mut byte = [0u8; 1];
    assert_eq!(
        peer.read(&mut byte).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(!complete_commit_view_conservation(&shared, &task));
    assert!(
        !shared
            .lock()
            .unwrap()
            .delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap()
            .cleanup_done
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .workspace_work
            .lock()
            .unwrap()
            .reports
            .is_empty()
    );
    assert_eq!(
        peer.read(&mut byte).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );

    {
        let mut st = shared.lock().unwrap();
        st.remote_workers
            .get_mut("cartae")
            .unwrap()
            .capabilities
            .push(bridget_transport::protocol::WorkerCapability::WorkspaceViewV1);
        let mut active = task.clone();
        active.guarded = true;
        active.result_sent = true;
        active.idle_deadline_at = Some(unix_timestamp() + 60);
        assert!(active.continuation_guard_open());
        st.delegation_store.save(&active).unwrap();
    }
    assert_eq!(
        remote_worker_roundtrip(
            &shared,
            &worker,
            &command,
            operation,
            Instant::now() + Duration::from_secs(1)
        ),
        Err(bridget_transport::protocol::WorkerRefusal::Unauthorized)
    );
    assert_eq!(
        peer.read(&mut byte).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(shared.lock().unwrap().pending_worker_responses.is_empty());
    shared.lock().unwrap().delegation_store.save(&task).unwrap();
    peer.set_nonblocking(false).unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let incoming = Arc::clone(&shared);
    let original_digest = digest.clone();
    let task_id = task.task_id.clone();
    let responder = std::thread::spawn(move || {
        use std::io::BufRead;
        let mut line = String::new();
        std::io::BufReader::new(peer).read_line(&mut line).unwrap();
        let DaemonToWrapper::WorkerRequest { request } = serde_json::from_str(&line).unwrap()
        else {
            panic!("demande worker attendue");
        };
        assert_eq!(
            request.request,
            bridget_transport::protocol::WorkerRequestOperation::ConserveCommitView {
                prepare_digest: original_digest.clone()
            }
        );
        assert!(
            !incoming
                .lock()
                .unwrap()
                .delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .cleanup_done,
            "ne jamais publier cleanup_done avant l'ACK"
        );
        let response = bridget_transport::protocol::WorkerResponse {
            version: request.version,
            correlation_id: request.correlation_id,
            connection_epoch: request.connection_epoch,
            worker: request.worker,
            command: request.command,
            response: bridget_transport::protocol::WorkerResponseOperation::CommitViewConserved {
                prepare_digest: original_digest,
            },
        };
        assert!(
            super::super::handle_worker_message(
                &incoming,
                "view-worker",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none()
        );
        response
    });
    assert!(complete_commit_view_conservation(&shared, &task));
    let reply = responder.join().unwrap();
    let final_task = shared
        .lock()
        .unwrap()
        .delegation_store
        .get(&task.task_id)
        .unwrap()
        .unwrap();
    assert!(final_task.cleanup_done);
    assert_eq!(
        final_task.workspace.as_ref().unwrap().state,
        WorkspaceState::Kept
    );
    let final_bytes = serde_json::to_vec(&final_task).unwrap();
    // Réponse tardive exacte : absorbée sans effet ni faux nouvel arrêt.
    assert!(
        super::super::handle_worker_message(
            &shared,
            "view-worker",
            WrapperToDaemon::WorkerResponse {
                response: reply.clone()
            }
        )
        .is_none()
    );
    let mut stale = reply.clone();
    stale.connection_epoch = "old-epoch".into();
    assert!(matches!(
        super::super::handle_worker_message(
            &shared,
            "view-worker",
            WrapperToDaemon::WorkerResponse { response: stale }
        ),
        Some(DaemonToWrapper::WorkerRejected {
            reason: bridget_transport::protocol::WorkerRefusal::StaleConnection
        })
    ));
    let mut wrong = reply.clone();
    wrong.command.command_id = "unknown-preparation".into();
    assert!(matches!(
        super::super::handle_worker_message(
            &shared,
            "view-worker",
            WrapperToDaemon::WorkerResponse { response: wrong }
        ),
        Some(DaemonToWrapper::WorkerRejected {
            reason: bridget_transport::protocol::WorkerRefusal::CommandMismatch
        })
    ));
    assert_eq!(
        serde_json::to_vec(
            &shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        final_bytes
    );
    assert!(shared.lock().unwrap().pending_worker_responses.is_empty());
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_preparation_failure_names_cause_scope_owner_action_and_preserves_task() {
    use bridget_transport::protocol::{CommitViewFailure as V, WorkerRefusal};
    let (state, config, mut task, _) = super::tests::remote162_capture_fixture();
    task.state = "failed".into();
    task.workspace = None;
    task.cleanup_done = false;
    for cause in [
        V::ObjectUnavailable,
        V::StopUnconfirmed,
        V::MetadataCapacity,
        V::StorageSpaceUnknown,
        V::Unknown,
    ] {
        task.error = Some(format!(
            "remote_start_failed:{}",
            workspace_preparation::view_error_code(WorkerRefusal::WorkspacePrepareFailed, cause)
        ));
        let before = serde_json::to_value(&task).unwrap();
        let view = task_view(&task, None, true);
        assert_eq!(
            view["detail"]["commit_view_failure"]["cause"],
            serde_json::to_value(cause).unwrap()
        );
        assert!(
            view["error_message"]
                .as_str()
                .unwrap()
                .contains("préparation de cette tâche")
        );
        assert!(
            view["error_message"]
                .as_str()
                .unwrap()
                .contains("autres travaux")
        );
        assert!(
            view["error_recovery"]
                .as_str()
                .unwrap()
                .contains("clôture native")
        );
        assert!(
            view["error_human_action"]
                .as_str()
                .unwrap()
                .contains("utilisateur")
        );
        if cause == V::MetadataCapacity {
            let message = view["error_message"].as_str().unwrap();
            assert!(
                message.contains("réservations de résultats") && message.contains("mission active")
            );
            assert!(message.contains("Laisser ces travaux finir"));
        }
        assert_eq!(serde_json::to_value(&task).unwrap(), before);
    }
    drop(state);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_unpublished_preparation_has_exact_owner_binding_and_no_invented_report() {
    use crate::delegation::ViewBinding;
    let (state, config, mut task, _) = super::tests::remote162_capture_fixture();
    let worker = remote_worker_identity(task.remote_placement.as_ref().unwrap());
    let command = bridget_transport::protocol::WorkerCommandIdentity {
        command_id: task.task_id.clone(),
        task_id: task.task_id.clone(),
        instance_id: String::new(),
        generation: 0,
    };
    let binding = ViewBinding {
        worker: worker.clone(),
        command: command.clone(),
        prepare_digest: "a".repeat(64),
    };
    task.workspace = None;
    task.workspace_report_mission = None;
    task.commit_view_prepare = Some(binding.clone());
    task.state = "failed".into();
    task.guarded = false;
    task.guard_released = false;
    task.cleanup_done = false;
    task.result_sent = false;
    task.idle_deadline_at = None;
    assert!(super::super::commit_view_conservation_matches(
        &task,
        &worker,
        &command,
        &binding.prepare_digest
    ));
    let before = serde_json::to_value(&task).unwrap();
    let full = task_view(&task, None, true);
    assert_eq!(
        full["detail"]["commit_view_preparation"]["publication"],
        "not_attested"
    );
    assert_eq!(
        full["detail"]["commit_view_preparation"]["conservation"],
        "unconfirmed"
    );
    assert!(
        full["detail"]["commit_view_preparation"]["next_action"]
            .as_str()
            .unwrap()
            .contains("ne pas créer")
    );
    assert!(task_view(&task, None, false)["detail"]["commit_view_preparation"].is_null());
    assert_eq!(serde_json::to_value(&task).unwrap(), before);
    let mut wrong = task.clone();
    wrong.commit_view_prepare.as_mut().unwrap().prepare_digest = "f".repeat(64);
    assert!(!super::super::commit_view_conservation_matches(
        &wrong,
        &worker,
        &command,
        &binding.prepare_digest
    ));
    wrong = task.clone();
    wrong.state = "working".into();
    assert!(!super::super::commit_view_conservation_matches(
        &wrong,
        &worker,
        &command,
        &binding.prepare_digest
    ));
    task.cleanup_done = true;
    assert_eq!(
        task_view(&task, None, true)["detail"]["commit_view_preparation"]["conservation"],
        "confirmed"
    );
    drop(state);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c19_direct_delegate_requires_auto_sha_before_task_or_source_effects() {
    let (state, _config, _commands, _reader) = super::tests::fixture("c19");
    let state = Arc::new(Mutex::new(state));
    let request: NativeDelegationRequest = serde_json::from_value(json!({
        "operation":"delegate","request_id":"c19-not-enabled","agent_type":"native-test",
        "model":"fixture","task":"Read only","cwd":"/unavailable-source",
        "posture":"discovery","workspace":{"strategy":"commit_view"}
    }))
    .unwrap();
    let before =
        serde_json::to_vec(&state.lock().unwrap().delegation_store.tasks().unwrap()).unwrap();
    let result = handle("conn-1", request, &state);
    let DaemonToWrapper::NativeDelegationResult { result } = result else {
        panic!("native result");
    };
    assert_eq!(result["code"], "expected_base_commit_required");
    assert!(
        result["message"]
            .as_str()
            .unwrap()
            .contains("autres travaux")
    );
    assert_eq!(result["detail"]["task_id"], Value::Null);
    assert_eq!(
        serde_json::to_vec(&state.lock().unwrap().delegation_store.tasks().unwrap()).unwrap(),
        before
    );
}
#[test]
fn spec221_sender_internal_diagnostic_keeps_refusal_and_identity_guards() {
    let (mut st, config, task, _) = super::tests::remote162_capture_fixture();
    let before = serde_json::to_value(&task).unwrap();
    let parent_conn = st
        .router
        .get_agent(&task.owner)
        .unwrap()
        .connection_id
        .clone();
    for (mode, expected) in [
        (0, "native_sender_instance_changed"),
        (1, "native_sender_not_live"),
        (2, "native_sender_primary_absent"),
    ] {
        if mode == 0 {
            st.conn_instances
                .insert(parent_conn.clone(), "other-instance".into());
        } else if mode == 1 {
            st.conn_instances
                .insert(parent_conn.clone(), task.owner_instance.clone());
            st.connections.remove(&parent_conn);
        } else {
            st.router.unregister_by_conn(&parent_conn);
        }
        let mut message =
            bridget_core::BridgetMessage::new(&task.owner, &task.child, "synthetic private brief");
        message.id = format!("sender221-{mode}");
        let id = message.id.clone();
        let mut controls = Vec::new();
        let result = send_as(
            &mut st,
            &task.owner,
            &task.owner_instance,
            &task.task_id,
            message,
            unix_timestamp(),
            &mut controls,
        );
        assert!(
            matches!(result,DaemonToWrapper::Nack {ref reason,..} if reason==expected),
            "{result:?}"
        );
        assert!(controls.is_empty());
        assert!(st.store.get_request(&id).unwrap().is_none());
        assert!(
            !st.conn_names
                .contains_key(&format!("native-delegation:{id}"))
        );
        assert_eq!(
            serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap(),
            before
        );
    }
    super::tests::cleanup(&config);
}
#[test]
fn spec221_sender_and_prepare_guidance_preserves_unknown_cause_and_task_bytes() {
    let (st, config, mut task, _) = super::tests::remote162_capture_fixture();
    task.state = "failed".into();
    for (error, field, cause) in [
        (
            "identité expéditeur non attestée : rattachement auxiliaire requis",
            "sender_attestation",
            "unobserved",
        ),
        (
            "native_sender_instance_changed",
            "sender_attestation",
            "parent_instance_changed",
        ),
        (
            "native_sender_not_live",
            "sender_attestation",
            "live_identity_unconfirmed",
        ),
        (
            "native_sender_primary_absent",
            "sender_attestation",
            "primary_absent",
        ),
        (
            "remote_start_failed:remote_workspace_prepare:ResourceLimit",
            "preparation_capacity",
            "unobserved",
        ),
    ] {
        task.error = Some(error.into());
        let before = serde_json::to_value(&task).unwrap();
        let view = task_view(&task, None, true);
        assert_eq!(view["error"], error);
        assert_eq!(view["detail"][field]["cause"], cause);
        if field == "sender_attestation" && cause == "unobserved" {
            assert!(
                view["error_message"]
                    .as_str()
                    .unwrap()
                    .contains("cause précise reste inconnue")
            );
            assert!(
                !view["error_message"]
                    .as_str()
                    .unwrap()
                    .contains("identité actuelle")
            );
        }
        assert!(
            view["error_message"]
                .as_str()
                .unwrap()
                .contains("autres travaux")
        );
        assert!(view["error_recovery"].as_str().unwrap().contains("tâche"));
        assert!(
            view["error_human_action"]
                .as_str()
                .unwrap()
                .contains("responsable Bridget")
        );
        assert_eq!(serde_json::to_value(&task).unwrap(), before);
        assert!(!view.to_string().contains("synthetic private brief"));
    }
    drop(st);
    super::tests::cleanup(&config);
}
