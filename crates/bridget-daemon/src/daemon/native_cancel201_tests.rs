//! Annulation avant reçu : aucune création de fournisseur.
use super::*;
use super::admission_tests::{fixture_guarded, saved};
use super::super::tests::cleanup;
use bridget_transport::protocol::{REMOTE_WORKER_VERSION, WorkerResponse};
#[test]
fn spec201_unknown_execution_fenced_then_cancelled_without_spawn() {
    let (state, config, task, _, _) = fixture_guarded(true);
    assert_eq!(saved(&state, &task.task_id)["guarded"], true);
    let before = saved(&state, &task.task_id);
    let (socket, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    state.lock().unwrap().connections.insert(
        "worker198".into(),
        Arc::new(Mutex::new(std::io::BufWriter::new(socket))),
    );
    let responder = state.clone();
    let listener = thread::spawn(move || {
        let mut reader = std::io::BufReader::new(peer);
        for fence in [false, true] {
            let mut line = String::new();
            std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = serde_json::from_str(&line).unwrap() else { panic!("worker request") };
            let response = if fence {
                assert!(matches!(request.request, WorkerRequestOperation::FenceUnstarted { .. }));
                WorkerResponseOperation::StartFenced {}
            } else {
                assert!(matches!(request.request, WorkerRequestOperation::Cancel { .. }));
                WorkerResponseOperation::Rejected { reason: WorkerRefusal::UnknownExecution }
            };
            let response = WorkerResponse { version: REMOTE_WORKER_VERSION,
                correlation_id: request.correlation_id, connection_epoch: request.connection_epoch,
                worker: request.worker, command: request.command, response };
            assert!(super::super::super::handle_worker_message(&responder, "worker198",
                WrapperToDaemon::WorkerResponse { response }).is_none());
        }

    });
    let view = handle(
        "conn-1",
        NativeDelegationRequest::Cancel {
            task_id: task.task_id.clone(),
        },
        &state,
    );
    listener.join().unwrap();
    let DaemonToWrapper::NativeDelegationResult { result } = view else {
        panic!("annulationnative198")
    };
    assert!(
        matches!(result["status"].as_str(), Some("cancelling" | "cancelled")),
        "{result}"
    );
    let waited = handle(
        "conn-1",
        NativeDelegationRequest::Wait {
            task_ids: vec![task.task_id.clone()],
            wait_timeout: Some(2),
        },
        &state,
    );
    let epoch = state.lock().unwrap().delegation_store.wait_epoch();
    let deadline = Instant::now() + Duration::from_secs(2);
    let after = loop {
        let seen = epoch.get();
        let current = saved(&state, &task.task_id);
        if current["cleanup_done"] == true || Instant::now() >= deadline {
            break current;
        }
        epoch.wait_changed(seen, deadline.saturating_duration_since(Instant::now()));
    };
    assert_eq!(
        after["state"], "cancelled",
        "état={after}, attente={waited:?}"
    );
    assert_eq!(after["cleanup_done"], true);
    assert_eq!(after["guard_released"], true);
    assert_eq!(
        after["error"], before["error"],
        "le refus d’origine reste conservé"
    );
    assert!(
        after["cancel_stopped"]
            .as_array()
            .unwrap()
            .contains(&before["child"])
    );
    assert_eq!(after["task_id"], before["task_id"]);
    assert_eq!(after["mission"], before["mission"]);
    drop(state);
    cleanup(&config);
}

#[test]
fn spec201_fence_requires_cancelling_unissued_exact_task_and_capability() {
    let (state, config, mut task, _, _) = fixture_guarded(true);
    let command = remote_command_identity(&state.lock().unwrap(), &task).unwrap();
    assert_eq!(fence_cancel_unstarted(&state, &task.task_id, &command), Err("not_unstarted"));
    task.state = "cancelling".into();
    task.mission_issued_at = Some(unix_timestamp());
    state.lock().unwrap().delegation_store.save(&task).unwrap();
    assert_eq!(fence_cancel_unstarted(&state, &task.task_id, &command), Err("not_unstarted"));
    task.mission_issued_at = None;
    state.lock().unwrap().delegation_store.save(&task).unwrap();
    let mut wrong = command.clone(); wrong.generation += 1;
    assert_eq!(fence_cancel_unstarted(&state, &task.task_id, &wrong), Err("identity_changed"));
    state.lock().unwrap().remote_workers.get_mut("cartae").unwrap().capabilities.clear();
    assert_eq!(fence_cancel_unstarted(&state, &task.task_id, &command), Err("capability_required"));
    assert_eq!(saved(&state, &task.task_id)["cleanup_done"], false);
    drop(state); cleanup(&config);
}
#[test]
fn spec201_cancel_guidance_is_closed_and_keeps_unknown_unknown() {
    let (state, config, mut task, _, _) = fixture_guarded(true);
    task.state = "cancelling".into();
    for cause in ["capability_required", "resource_limit_unobserved", "reply_unavailable", "identity_changed", "not_unstarted"] {
        task.error = Some(format!("native_cancel_unverifiable:{}:{cause}", task.child));
        let view = task_view(&task, None, true);
        assert_eq!(view["error_cause"], cause);
        assert!(view["error_recovery"].as_str().unwrap().len() > 30);
    }
    task.error = Some(format!("native_cancel_unverifiable:{}:private_value", task.child));
    assert_eq!(task_view(&task, None, true)["error_cause"], "unobserved");
    assert!(cancel_fence_cause("cancel_fence:private_value").is_none());
    drop(state); cleanup(&config);
}

fn fence_refusal_case(response: WorkerResponseOperation, change_task: bool, expected: &str) {
    let (state, config, mut task, _, _) = fixture_guarded(true);
    task.state = "cancelling".into();
    state.lock().unwrap().delegation_store.save(&task).unwrap();
    let command = remote_command_identity(&state.lock().unwrap(), &task).unwrap();
    let before = saved(&state, &task.task_id);
    let (socket, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    state.lock().unwrap().connections.insert("worker198".into(), Arc::new(Mutex::new(std::io::BufWriter::new(socket))));
    let responder = state.clone();
    let listener = thread::spawn(move || {
        let mut reader = std::io::BufReader::new(peer);
        let mut line = String::new();
        std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
        let DaemonToWrapper::WorkerRequest { request } = serde_json::from_str(&line).unwrap() else { panic!("worker") };
        assert!(matches!(request.request, WorkerRequestOperation::FenceUnstarted { .. }));
        if change_task {
            let st = responder.lock().unwrap();
            let mut current = st.delegation_store.get(&request.command.task_id).unwrap().unwrap();
            current.mission_issued_at = Some(unix_timestamp());
            st.delegation_store.save(&current).unwrap();
        }
        let response = WorkerResponse { version: REMOTE_WORKER_VERSION,
            correlation_id: request.correlation_id, connection_epoch: request.connection_epoch,
            worker: request.worker, command: request.command, response };
        assert!(super::super::super::handle_worker_message(&responder, "worker198",
            WrapperToDaemon::WorkerResponse { response }).is_none());
    });
    assert_eq!(fence_cancel_unstarted(&state, &task.task_id, &command), Err(expected));
    listener.join().unwrap();
    let after = saved(&state, &task.task_id);
    for field in ["task_id", "child", "child_instance", "mission", "round", "request", "remote_placement", "cleanup_done", "guard_released"] {
        assert_eq!(after[field], before[field], "{field}");
    }
    drop(state); cleanup(&config);
}
#[test]
fn spec201_capacity_or_missing_ack_never_closes_guard() {
    fence_refusal_case(WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResourceLimit }, false, "resource_limit_unobserved");
    fence_refusal_case(WorkerResponseOperation::Rejected { reason: WorkerRefusal::WorkerUnavailable }, false, "reply_unavailable");
}
#[test]
fn spec201_state_changed_during_fence_does_not_close_guard() {
    fence_refusal_case(WorkerResponseOperation::StartFenced {}, true, "identity_changed");
}

#[test]
fn spec201_local_pending_capacity_is_known_without_worker_send() {
    use bridget_transport::protocol::MAX_WORKER_STORE_ENTRIES;
    let (state, config, mut task, _, _) = fixture_guarded(true);
    task.state = "cancelling".into();
    state.lock().unwrap().delegation_store.save(&task).unwrap();
    let command = remote_command_identity(&state.lock().unwrap(), &task).unwrap();
    let worker = remote_worker_identity(task.remote_placement.as_ref().unwrap());
    let (socket, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    {
        let mut st = state.lock().unwrap();
        st.connections.insert("worker198".into(), Arc::new(Mutex::new(std::io::BufWriter::new(socket))));
        for n in 0..MAX_WORKER_STORE_ENTRIES {
            let (tx, _rx) = mpsc::sync_channel(1);
            st.pending_worker_responses.insert(format!("pending201-{n}"),
                super::super::super::PendingWorkerResponse { conn_id: "worker198".into(),
                    epoch: "epoch198".into(), worker: worker.clone(), command: command.clone(), reply: tx });
        }
    }
    assert_eq!(fence_cancel_unstarted(&state, &task.task_id, &command), Err("master_pending"));
    let mut reader = std::io::BufReader::new(peer); let mut line = String::new();
    assert!(std::io::BufRead::read_line(&mut reader, &mut line).is_err());
    assert!(line.is_empty());
    task.error = Some(format!("native_cancel_unverifiable:{}:master_pending", task.child));
    let view = task_view(&task, None, true);
    assert_eq!(view["error_cause"], "master_pending");
    assert!(view["error_recovery"].as_str().unwrap().contains("aucune requête"));
    drop(state); cleanup(&config);
}
