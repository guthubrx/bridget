//! Coordination185 : même tâche et mêmes canons de naissance.
use super::*;
use bridget_transport::protocol::{WorkspaceRepairRequestV1, WorkspaceRepairStatusV1};

pub(crate) struct Pending {
    pub child: String,
    pub instance: String,
    pub task_id: String,
    pub request: WorkspaceRepairRequestV1,
    pub sender: std::sync::mpsc::SyncSender<WorkspaceRepairStatusV1>,
}

/// Réponse corrélée à l'enfant réellement connecté ; jamais une autorité caller.
pub(crate) fn response(
    st: &mut DaemonState,
    conn: &str,
    task_id: &str,
    status: WorkspaceRepairStatusV1,
) {
    let Some((child, instance)) = live_connection_identity(st, conn) else {
        return;
    };
    let key = format!("{task_id}:{}", status.request.request_id);
    let Some(pending) = st.pending_workspace_repairs.get(&key) else {
        return;
    };
    if pending.child != child
        || pending.instance != instance
        || pending.task_id != task_id
        || pending.request != status.request
    {
        return;
    }
    if let Some(pending) = st.pending_workspace_repairs.remove(&key) {
        let _ = pending.sender.send(status);
    }
}

fn save_status(
    st: &mut DaemonState,
    task_id: &str,
    status: WorkspaceRepairStatusV1,
) -> Result<Task, String> {
    let mut task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    let last = task
        .workspace_repairs
        .last_mut()
        .ok_or("workspace_repair_not_prepared")?;
    if last.request != status.request {
        return Err("workspace_repair_envelope_mismatch".into());
    }
    let rank = |phase: &str| match phase {
        "requested" => 0,
        "prepared" => 1,
        "stopped" => 2,
        "resumed" => 3,
        "publishing" => 4,
        "published" => 5,
        _ => 0,
    };
    if last.phase == "published" || rank(&last.phase) > rank(&status.phase) {
        return Ok(task);
    }
    if last.phase == "refused" && status.phase != "refused" {
        return Err("workspace_repair_task_changed".into());
    }
    *last = status;
    if last.phase == "published" {
        let prepared = last
            .prepared
            .as_ref()
            .ok_or("workspace_repair_proof_missing")?;
        let workspace = serde_json::from_value(
            prepared
                .workspace
                .clone()
                .ok_or("workspace_repair_proof_missing")?,
        )
        .map_err(|_| "workspace_repair_proof_invalid")?;
        task.workspace = Some(workspace);
    }
    st.delegation_store.save(&task)?;
    Ok(task)
}

pub(super) fn repair(
    conn: &str,
    task_id: &str,
    request: &WorkspaceRepairRequestV1,
    attested: Option<&bridget_transport::protocol::ProviderPermissions>,
    thread: Option<&str>,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<Value, String> {
    request
        .validate()
        .map_err(|_| "workspace_repair_invalid_request")?;
    let (saved, command, worker, owner, instance) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        recheck_attested_thread(&st, conn, thread)?;
        let (owner, instance) =
            live_connection_identity(&st, conn).ok_or("permission_attestation_unavailable")?;
        let mut task = st
            .delegation_store
            .get(task_id)?
            .ok_or("task_unavailable")?;
        if !can_own_task(&st, &task, &owner, &instance)? {
            return Err("task_unavailable".into());
        }
        // Un reçu terminal est une lecture idempotente, même après expiration
        // de la garde. La propriété vient d'être contrôlée ; aucun effet nouveau.
        if let Some(previous) = task
            .workspace_repairs
            .iter()
            .find(|v| v.request.request_id == request.request_id)
        {
            if previous.request != *request {
                return Err("workspace_repair_envelope_mismatch".into());
            }
            if previous.phase == "published" {
                return current_task_view(&st, &task, true);
            }
            if previous.phase == "refused" {
                return Err(previous
                    .code
                    .clone()
                    .unwrap_or_else(|| "workspace_repair_refused".into()));
            }
        }
        let control = crate::referent_control::read(st.store.connection())
            .map_err(|_| "control_state_unavailable")?;
        let halt = st
            .delegation_store
            .delegation_halt()
            .map_err(|_| "control_state_unavailable")?;
        if let crate::referent_control::Admission::Deferred { motif } =
            crate::referent_control::admit_native_delegation(&control, &halt)
        {
            return Err(if motif == "pause" {
                "referent_paused"
            } else {
                motif
            }
            .into());
        }
        let fact = parent_fact(&st, &owner, &instance, Some(&request.request_id), attested)?;
        if !continuation_parent_matches(
            task.permission_snapshot.as_ref().map(|v| &v.parent),
            fact.as_ref(),
        ) {
            return Err("permission_not_inherited".into());
        }
        if task.state != "result_available"
            || !task.result_sent
            || !task.guarded
            || task.guard_released
            || task.cleanup_done
            || task.idle_deadline_at.is_some_and(|v| unix_timestamp() >= v)
            || !st.delegation_store.open_child_requests(task_id)?.is_empty()
        {
            return Err("workspace_repair_not_idle".into());
        }
        recheck_remote_dispatch(&st, &task, &task)?;
        let placement = task
            .remote_placement
            .as_ref()
            .ok_or("workspace_repair_remote_required")?;
        let command = remote_command_identity(&st, &task)
            .map_err(|_| "workspace_repair_identity_unconfirmed")?;
        let worker = remote_worker_identity(placement);
        if let Some(previous) = task
            .workspace_repairs
            .iter()
            .find(|v| v.request.request_id == request.request_id)
        {
            if previous.request != *request {
                return Err("workspace_repair_envelope_mismatch".into());
            }
            if task
                .workspace_repairs
                .last()
                .is_some_and(|last| last.request.request_id != request.request_id)
            {
                return current_task_view(&st, &task, true);
            }
        }
        if let Some(previous) = task.workspace_repairs.last() {
            if previous.request.request_id == request.request_id {
                if previous.request != *request {
                    return Err("workspace_repair_envelope_mismatch".into());
                }
                if previous.phase == "published" {
                    return current_task_view(&st, &task, true);
                }
                if previous.phase == "refused" {
                    return Err(previous
                        .code
                        .clone()
                        .unwrap_or_else(|| "workspace_repair_refused".into()));
                }
            } else if previous.phase != "refused" || task.workspace_repairs.len() >= 8 {
                return Err("workspace_repair_already_recorded".into());
            } else {
                task.workspace_repairs.push(WorkspaceRepairStatusV1 {
                    request: request.clone(),
                    phase: "requested".into(),
                    prepared: None,
                    proof: None,
                    code: None,
                });
                st.delegation_store.save(&task)?;
            }
        } else {
            task.workspace_repairs.push(WorkspaceRepairStatusV1 {
                request: request.clone(),
                phase: "requested".into(),
                prepared: None,
                proof: None,
                code: None,
            });
            st.delegation_store.save(&task)?;
        }
        (task, command, worker, owner, instance)
    };
    let snapshot = saved
        .permission_snapshot
        .as_ref()
        .ok_or("permission_attestation_unavailable")?;
    remote_parent_proof(state, &saved.owner, &snapshot.parent)?;
    let prepared = remote_worker_roundtrip(
        state,
        &worker,
        &command,
        bridget_transport::protocol::WorkerRequestOperation::RepairWorkspace {
            repair: request.clone(),
            publish: false,
            expires_at: saved
                .idle_deadline_at
                .unwrap_or(unix_timestamp())
                .min(unix_timestamp() + 60),
        },
        Instant::now() + Duration::from_secs(30),
    )
    .map_err(|e| format!("workspace_repair_remote:{e:?}"))?;
    let bridget_transport::protocol::WorkerResponseOperation::WorkspaceRepair { status } = prepared
    else {
        return Err("workspace_repair_proof_missing".into());
    };
    if status.request != *request {
        return Err("workspace_repair_envelope_mismatch".into());
    }
    {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        let current = st
            .delegation_store
            .get(task_id)?
            .ok_or("task_unavailable")?;
        if !can_own_task(&st, &current, &owner, &instance)? || current.state != "result_available" {
            return Err("workspace_repair_task_changed".into());
        }
        recheck_remote_dispatch(&st, &current, &saved)?;
        if current
            .workspace_repairs
            .last()
            .is_some_and(|v| v.phase == "published")
        {
            return current_task_view(&st, &current, true);
        }
        save_status(&mut st, task_id, status.clone())?;
        if status.phase == "refused" {
            return Err(status
                .code
                .unwrap_or_else(|| "workspace_repair_refused".into()));
        }
    }
    if status.phase != "published" {
        let key = format!("{task_id}:{}", request.request_id);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let writer = {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let current = st
                .delegation_store
                .get(task_id)?
                .ok_or("task_unavailable")?;
            if current.state != "result_available"
                || !can_own_task(&st, &current, &owner, &instance)?
            {
                return Err("workspace_repair_task_changed".into());
            }
            if st.pending_workspace_repairs.contains_key(&key) {
                return Err("workspace_repair_pending".into());
            }
            let route = st
                .router
                .get_agent(&saved.child)
                .ok_or("workspace_repair_child_unavailable")?;
            let writer = st
                .connections
                .get(&route.connection_id)
                .cloned()
                .ok_or("workspace_repair_child_unavailable")?;
            st.pending_workspace_repairs.insert(
                key.clone(),
                Pending {
                    child: saved.child.clone(),
                    instance: saved
                        .child_instance
                        .clone()
                        .ok_or("workspace_repair_identity_unconfirmed")?,
                    task_id: task_id.into(),
                    request: request.clone(),
                    sender,
                },
            );
            writer
        };
        // Le résultat est un blob durable du wrapper ; un ACK perdu n'autorise
        // jamais une seconde opération sous une autre clé.
        let control = DaemonToWrapper::WorkspaceRepairControl {
            task_id: task_id.into(),
            status: status.clone(),
        };
        if let Err(error) =
            push_control_message_until(&writer, &control, Instant::now() + Duration::from_secs(5))
        {
            state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .pending_workspace_repairs
                .remove(&key);
            return Err(format!("workspace_repair_control_unconfirmed:{error}"));
        }
        let result = receiver.recv_timeout(Duration::from_secs(30));
        state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pending_workspace_repairs
            .remove(&key);
        let result = result.map_err(|_| "workspace_repair_pending")?;
        if result.request != *request {
            return Err("workspace_repair_envelope_mismatch".into());
        }
        if result.phase != "resumed" {
            let code = result
                .code
                .clone()
                .unwrap_or_else(|| "workspace_repair_resume_unconfirmed".into());
            // Après arrêt, un échec reste fermé et conservé.
            save_status(
                &mut state.lock().unwrap_or_else(|e| e.into_inner()),
                task_id,
                result,
            )?;
            return Err(code);
        }
    }
    remote_parent_proof(state, &saved.owner, &snapshot.parent)?;
    {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let current = st
            .delegation_store
            .get(task_id)?
            .ok_or("task_unavailable")?;
        if current.state != "result_available"
            || current.guard_released
            || current
                .idle_deadline_at
                .is_some_and(|v| unix_timestamp() >= v)
        {
            return Err("workspace_repair_task_changed".into());
        }
        recheck_remote_dispatch(&st, &current, &saved)?;
    }
    let published = remote_worker_roundtrip(
        state,
        &worker,
        &command,
        bridget_transport::protocol::WorkerRequestOperation::RepairWorkspace {
            repair: request.clone(),
            publish: true,
            expires_at: saved
                .idle_deadline_at
                .unwrap_or(unix_timestamp())
                .min(unix_timestamp() + 60),
        },
        Instant::now() + Duration::from_secs(30),
    )
    .map_err(|e| format!("workspace_repair_remote:{e:?}"))?;
    let bridget_transport::protocol::WorkerResponseOperation::WorkspaceRepair { status } =
        published
    else {
        return Err("workspace_repair_resume_unconfirmed".into());
    };
    if status.request != *request || status.phase != "published" {
        return Err("workspace_repair_pending".into());
    }
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    let current = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    if current.state != "result_available" || !can_own_task(&st, &current, &owner, &instance)? {
        return Err("workspace_repair_task_changed".into());
    }
    let task = save_status(&mut st, task_id, status)?;
    current_task_view(&st, &task, true)
}
