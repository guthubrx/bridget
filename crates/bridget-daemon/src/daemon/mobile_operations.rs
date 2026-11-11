//! Dernier contrôle de portée sur l'identité courante, avant les effets natifs.
use bridget_transport::poison::RecoverPoison;
use super::*;
use crate::mobile::store::{Operation, Store};
use bridget_transport::protocol::{MobileOperation, MobileOperationRequest};
use std::path::Path;

fn refusal(code: &str) -> DaemonToWrapper {
    DaemonToWrapper::MobileOperationResult {
        result: serde_json::json!({"status":"refused","code":code}),
    }
}
fn valid_command_id(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|u| u.get_version_num() == 4 && u.to_string() == id)
}
fn public_receipt(response: DaemonToWrapper, expected: &str) -> DaemonToWrapper {
    let DaemonToWrapper::IdempotencyResult {
        operation_kind,
        idempotency_key,
        mut issue,
        ..
    } = response
    else {
        return refusal("outcome_unknown");
    };
    if operation_kind != "send" || idempotency_key != expected {
        return refusal("outcome_unknown");
    }
    if let IdempotencyIssue::Rejected {
        category, reason, ..
    } = &mut issue
    {
        if category.len() > 64
            || !category
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
        {
            *category = "native_refused".into();
        }
        *reason = "Commande refusée par les règles natives ; conservez son reçu.".into();
    }
    if let IdempotencyIssue::Orphaned { reason, .. } = &mut issue {
        *reason =
            "Le destinataire de la remise n'est plus disponible ; ne recréez pas la commande."
                .into();
    }
    DaemonToWrapper::MobileOperationResult {
        result: serde_json::json!({
            "status":"ok","command_id":idempotency_key,"operation_kind":"send","issue":issue
        }),
    }
}

/// Le journal historique n'atteste pas encore une partition par projet/machine.
fn agent_journal(
    request: &MobileOperationRequest,
    device: &crate::mobile::store::Device,
    root: &Path,
    agent_id: &str,
    after_seq: u64,
    limit: u32,
    tail: bool,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    if Uuid::parse_str(agent_id).is_err()
        || !(1..=50).contains(&limit)
        || tail && after_seq != 0
        || after_seq > bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ
    {
        return refusal("invalid_request");
    }
    if !device.scope.operations.contains(&Operation::Read) {
        return refusal("forbidden_scope");
    }
    if !device.scope.all_projects || !device.scope.all_machines {
        return refusal("history_scope_unavailable");
    }
    let (task, identity, foreign, socket) = {
        let st = state.lock().recover_poison();
        let task = match st.delegation_store.for_child(agent_id) {
            Ok(task) => task,
            Err(_) => return refusal("storage_unavailable"),
        };
        let identity = st
            .router
            .get_agent(agent_id)
            .and_then(|route| live_connection_identity(&st, &route.connection_id));
        if task.is_none() && identity.is_none() {
            return refusal("not_found");
        }
        let foreign=identity.as_ref().and_then(|(_,instance)|st.presences.get(instance))
            .is_some_and(|presence|presence.host!=st.host);
        (task, identity, foreign, st.socket_path.clone())
    };
    let after_seq = if tail {
        match crate::attach::journal_tail_start(&socket,agent_id,limit) {
            Ok(start)=>start, Err(_)=>return refusal("journal_unavailable")
        }
    } else { after_seq };
    let page = if let Some(task) = &task {
        match native_lineage::read_task_journal(state, task, after_seq, limit) {
            Ok((page, _)) => page,
            Err(_) => return refusal("journal_unavailable"),
        }
    } else if foreign {
        match crate::attach::lineage_journal_remote_page(&socket,agent_id,after_seq,limit) {
            Ok(page)=>page,Err(_)=>return refusal("journal_unavailable")
        }
    } else {
        match crate::attach::lineage_journal_page(
            &root.join("sessions").join(agent_id),
            agent_id,
            after_seq,
            limit,
        ) {
            Ok(page) => page,
            Err(_) => return refusal("journal_unavailable"),
        }
    };
    {
        let st = state.lock().recover_poison();
        if let Some(snapshot) = &task {
            let current = match st.delegation_store.get(&snapshot.task_id) {
                Ok(Some(task)) => task,
                _ => return refusal("journal_unavailable"),
            };
            if current.child != snapshot.child
                || current.child_instance != snapshot.child_instance
                || current.remote_placement != snapshot.remote_placement
            {
                return refusal("journal_unavailable");
            }
        } else if identity
            != st
                .router
                .get_agent(agent_id)
                .and_then(|route| live_connection_identity(&st, &route.connection_id))
        {
            return refusal("journal_unavailable");
        }
    }
    // Révocation/expiration relues après le disque ou le worker distant.
    if let Err(error) = Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        return refusal(error.code());
    }
    let agent_state={let st=state.lock().recover_poison();
        st.router.get_agent(agent_id).and_then(|route|st.conn_instances.get(&route.connection_id))
            .and_then(|instance|st.presences.get(instance)).map(|p|p.state.clone()).unwrap_or_else(||"unknown".into())};
    DaemonToWrapper::MobileOperationResult {
        result: serde_json::json!({
            "status":"ok","agent_id":agent_id,"agent_state":agent_state,"items":page.get("events"),
            "next_seq":page.get("next_seq"),"caught_up":page.get("caught_up"),"gap":page.get("gap"),
            "older_before_seq":if after_seq>0 {Some(after_seq+1)} else {None}
        }),
    }
}

pub(super) fn handle(
    conn_id: &str,
    request: MobileOperationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let required_capabilities: &[ClientCapability] =
        if matches!(&request.operation, MobileOperation::Specialists { .. }) {
            &[ClientCapability::MobileOperationsV1, ClientCapability::SpecialistsV1]
        } else {
            &[ClientCapability::MobileOperationsV1]
        };
    let root = {
        let st = state.lock().recover_poison();
        if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Client)
            || st.auxiliary_connections.contains(conn_id)
            || st.conn_names.contains_key(conn_id)
            || st.peer_uids.get(conn_id).copied() != Some(bridget_transport::fsutil::effective_uid())
            || !st.client_negotiations.get(conn_id).is_some_and(|n| {
                n.version == CLIENT_CONTRACT_VERSION
                    && n.capabilities.as_slice() == required_capabilities
            })
        {
            return refusal("authorization_required");
        }
        let Some(root) = st.db_path.parent() else {
            return refusal("storage_unavailable");
        };
        root.to_path_buf()
    };
    // Pas d'I/O mobile sous le verrou métier ; aucune opération n'est acceptée sur le seul label ClientHello.
    let device = match Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        Ok(device) => device,
        Err(error) => return refusal(error.code()),
    };
    if let MobileOperation::Specialists {request:command}=&request.operation {
        return super::specialists222::handle_mobile(conn_id,command,&device,state);
    }
    if let MobileOperation::AgentTracking {request:command}=&request.operation {
        return super::agent_tracking230::mobile(command,&device,state);
    }
    if matches!(
        request.operation,
        MobileOperation::UpdatesStatus
            | MobileOperation::UpdatesCheck
            | MobileOperation::UpdatesInstall { .. }
            | MobileOperation::UpdatesConfigure { .. }
    ) {
        let granted = match crate::updates::device_granted(&root, &device.device_id) {
            Ok(granted) => granted,
            Err(code) => return refusal(code),
        };
        if !device.scope.all_machines || !device.scope.all_projects || !granted {
            return refusal("forbidden_scope");
        }
        let q = match request.operation {
            MobileOperation::UpdatesCheck => bridget_transport::protocol::UpdateRequest::Check,
            MobileOperation::UpdatesInstall { version } => {
                bridget_transport::protocol::UpdateRequest::Install { version }
            }
            MobileOperation::UpdatesConfigure { automatic } => {
                bridget_transport::protocol::UpdateRequest::Configure { automatic }
            }
            _ => bridget_transport::protocol::UpdateRequest::Status,
        };
        return match crate::updates::policy(&root, &q) {
            Ok(result) => DaemonToWrapper::MobileOperationResult { result: super::mobile_admin::project_update_status(result, state) },
            Err(code) => refusal(code),
        };
    }
    if matches!(
        request.operation,
        MobileOperation::Questions { .. }
            | MobileOperation::UserInputs { .. }
            | MobileOperation::UserInputShow { .. }
            | MobileOperation::UserInputRespond { .. }
            | MobileOperation::UserInputReceipt { .. }
            | MobileOperation::QuestionShow { .. }
            | MobileOperation::QuestionRespond { .. }
            | MobileOperation::QuestionReceipt { .. }
    ) {
        return super::mobile_questions::handle(&request, &root, state);
    }
    if matches!(
        request.operation,
        MobileOperation::TaskList { .. }
            | MobileOperation::TaskShow { .. }
            | MobileOperation::TaskCancel { .. }
            | MobileOperation::TaskCancelReceipt { .. }
    ) {
        return super::mobile_tasks::handle(&request, &root, state);
    }
    if matches!(
        request.operation,
        MobileOperation::ArtifactUpload { .. }
            | MobileOperation::ArtifactRead { .. }
            | MobileOperation::ArtifactList { .. }
            | MobileOperation::ArtifactLookup { .. }
    ) {
        return super::mobile_artifacts::handle(&request, &device, &root, state);
    }
    if let MobileOperation::AgentJournal {
        agent_id,
        after_seq,
        limit,
        tail,
    } = &request.operation
    {
        return agent_journal(
            &request, &device, &root, agent_id, *after_seq, *limit, *tail, state,
        );
    }
    let mut st = state.lock().recover_poison();
    if st.conn_names.contains_key(conn_id)
        || st.connection_roles.get(conn_id) != Some(&ConnectionRole::Client)
    {
        return refusal("authorization_required");
    }
    let Some(negotiated) = st.client_negotiations.get_mut(conn_id) else {
        return refusal("authorization_required");
    };
    negotiated.issuer_scope = format!("mobile177_{}", device.device_id.replace('-', ""));
    if matches!(request.operation, MobileOperation::MessageQueue { .. } | MobileOperation::MessageQueueChange { .. }) {
        return super::mobile_queue::handle(conn_id, &request.operation, &device, &mut st);
    }
    match request.operation {
        MobileOperation::Specialists { .. }
        | MobileOperation::AgentTracking { .. }
        | MobileOperation::MessageQueue { .. }
        | MobileOperation::MessageQueueChange { .. }
        | MobileOperation::UpdatesStatus
        | MobileOperation::UpdatesCheck
        | MobileOperation::UpdatesInstall { .. }
        | MobileOperation::UpdatesConfigure { .. }
        | MobileOperation::Questions { .. }
        | MobileOperation::UserInputs { .. }
            | MobileOperation::UserInputShow { .. }
            | MobileOperation::UserInputRespond { .. }
            | MobileOperation::UserInputReceipt { .. }
            | MobileOperation::QuestionShow { .. }
        | MobileOperation::QuestionRespond { .. }
        | MobileOperation::QuestionReceipt { .. }
        | MobileOperation::TaskList { .. }
        | MobileOperation::TaskShow { .. }
        | MobileOperation::TaskCancel { .. }
        | MobileOperation::TaskCancelReceipt { .. }
        | MobileOperation::AgentJournal { .. }
        | MobileOperation::ArtifactUpload { .. }
        | MobileOperation::ArtifactRead { .. }
        | MobileOperation::ArtifactList { .. }
        | MobileOperation::ArtifactLookup { .. } => refusal("invalid_request"),
        MobileOperation::Federation { after, limit } => {
            if !(1..=50).contains(&limit) || after.as_ref().is_some_and(|s| s.len() > 256) {
                return refusal("invalid_request");
            }
            if !device.scope.operations.contains(&Operation::Read) {
                return refusal("forbidden_scope");
            }
            let mut names = std::collections::BTreeSet::new();
            names.insert(st.host.clone());
            names.extend(st.registry.machines().keys().cloned());
            names.extend(st.remote_workers.keys().cloned());
            names.extend(st.presences.values().map(|p| p.host.clone()));
            let selected = names
                .into_iter()
                .filter(|name| device.scope.all_machines || device.scope.machines.contains(name))
                .filter(|name| after.as_ref().is_none_or(|cursor| name > cursor))
                .take(usize::from(limit) + 1)
                .collect::<Vec<_>>();
            let more = selected.len() > usize::from(limit);
            let next = more.then(|| selected[usize::from(limit) - 1].clone());
            let items=selected.into_iter().take(usize::from(limit)).map(|machine| {
                let (availability,age)=if machine==st.host {("connected",Some(0))}
                    else if let Some(worker)=st.remote_workers.get(&machine) {
                        let current=worker.contact==bridget_transport::protocol::WorkerContactState::Connected
                            && worker.last_contact.elapsed()<Duration::from_secs(bridget_transport::protocol::WORKER_STALE_SECS as u64);
                        (if current {"connected"}else{"stale"},Some(worker.last_contact.elapsed().as_secs()))
                    } else if st.presences.values().any(|p|p.host==machine
                        && matches!(p.state.as_str(),"connected"|"busy") && presence_within_retention(p)) {
                        ("observed_agent",None)
                    } else {("unknown",None)};
                serde_json::json!({"machine":machine,"availability":availability,"observed_age_secs":age})
            }).collect::<Vec<_>>();
            DaemonToWrapper::MobileOperationResult {
                result: serde_json::json!({
                    "status":"ok","items":items,"next_cursor":next,"observed_at":unix_now_secs()
                }),
            }
        }
        MobileOperation::Lookup { command_id } => {
            if !valid_command_id(&command_id) {
                return refusal("invalid_request");
            }
            if !device.scope.operations.contains(&Operation::Message) {
                return refusal("forbidden_scope");
            }
            public_receipt(
                handle_idempotency_lookup(conn_id, "send".into(), command_id.clone(), &st),
                &command_id,
            )
        }
        MobileOperation::Message {
            device_id,
            command_id,
            issued_at,
            agent_id,
            body,
            guide_now,
        } => {
            if device_id != device.device_id {
                return refusal("command_origin_changed");
            }
            if !valid_command_id(&command_id)
                || Uuid::parse_str(&agent_id).is_err()
                || body.trim().is_empty()
                || body.len() > 10_000
                || body.contains('\0')
            {
                return refusal("invalid_request");
            }
            let Some((target, instance)) = st
                .router
                .get_agent(&agent_id)
                .and_then(|route| live_connection_identity(&st, &route.connection_id))
            else {
                return refusal("not_found");
            };
            if target != agent_id {
                return refusal("not_found");
            }
            let Some(presence) = st.presences.get(&instance) else {
                return refusal("not_found");
            };
            let project = agent_communication_project(&st, &agent_id);
            if !device
                .scope
                .permits(Operation::Message, &presence.host, project.as_ref())
            {
                return refusal("not_found");
            }
            let mut message = bridget_core::BridgetMessage::new("human", &agent_id, body);
            message.id = command_id.clone();
            if guide_now {
                message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
                message.references.push("steer-or-queue".into());
            }
            // Pas de reply réservé à une identité d'agent ; les résultats se lisent dans le journal.
            let mut controls = Vec::new();
            let response = handle_idempotent_send_with_result_authority(
                conn_id,
                message,
                command_id.clone(),
                issued_at,
                IdempotentSendAdmission {
                    project: None,
                    issued_at_tolerance_secs: 300,
                },
                &mut st,
                &mut controls,
                None,
                true,
            );
            st.pending_post_response_controls
                .entry(conn_id.into())
                .or_default()
                .extend(controls);
            public_receipt(response, &command_id)
        }
    }
}

#[cfg(test)]
#[path = "mobile_operations_tests.rs"]
mod tests;
