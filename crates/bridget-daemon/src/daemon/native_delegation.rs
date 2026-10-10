//! Effets de la saga native, sous les mêmes gardes que la flotte et les remises.
use super::*;
use crate::delegation::Task;
use bridget_transport::protocol::{NativeDelegationRequest, SpawnOwnership, SpawnPosture};
use serde_json::{Value, json};
const MISSION_REPLY_LIMIT_SECS: i64 = 3600;

enum ParentPermission {
    Root(PathBuf, SpawnPosture),
    SameProject(bridget_transport::protocol::CommunicationProject),
}
impl ParentPermission {
    fn maximum(&self) -> SpawnPosture {
        match self {
            Self::Root(_, posture) => *posture,
            Self::SameProject(_) => SpawnPosture::Discovery,
        }
    }
    fn root(&self) -> Option<&std::path::Path> {
        match self {
            Self::Root(path, _) => Some(path.as_path()),
            Self::SameProject(_) => None,
        }
    }
    fn scope(&self) -> &'static str {
        match self {
            Self::Root(_, _) => "root",
            Self::SameProject(_) => "same_project",
        }
    }
}

fn failure(code: &str) -> Value {
    json!({"status":"refused","code":code})
}

fn task_view(task: &Task) -> Value {
    let (agent_type, model, effort, cwd, posture) = match &task.request {
        NativeDelegationRequest::Delegate {
            agent_type,
            model,
            effort,
            cwd,
            posture,
            ..
        } => (agent_type, model, effort, cwd, posture),
        _ => unreachable!("seules les missions sont persistées"),
    };
    let _ = cwd;
    json!({"version":1,"task_id":task.task_id,"status":task.state,
        "child_agent_id":task.child,"message_id":task.mission,
        "agent_type":agent_type,"model":model,"effort":effort,"cwd":task.cwd,"posture":posture,
        "mission_deadline_at":task.mission_deadline_at,
        "result":if task.state=="result_available" {task.result.as_deref()} else {None},"error":task.error})
}

fn root_permission(
    st: &DaemonState,
    name: &str,
    instance: &str,
) -> Result<Option<ParentPermission>, String> {
    if st.delegation_store.revoked_agent(name)? || st.delegation_store.revoked(instance)? {
        return Ok(None);
    }
    if let Some((root, posture)) = st.delegation_store.permission(instance)? {
        return Ok(Some(ParentPermission::Root(PathBuf::from(root), posture)));
    }
    if let Some(entry) = st
        .fleet
        .desired_entry(name)
        .map_err(|error| error.to_string())?
        && st.managed_by_instance.get(instance) == Some(&entry.command_id)
    {
        let Some(definition) = entry.resolved_definition.as_ref() else {
            return Ok(None);
        };
        // Héritage de la seule posture explicitement construite par Bridget.
        let effective = |key: &str| {
            definition
                .args
                .iter()
                .filter_map(|arg| arg.strip_prefix(key))
                .next_back()
        };
        let development = definition.protocol == "codex_app_server"
            && definition.permissions == "deny"
            && effective("sandbox_mode=") == Some("\"workspace-write\"")
            && effective("sandbox_workspace_write.network_access=") == Some("false")
            && effective("sandbox_workspace_write.writable_roots=") == Some("[]")
            && effective("sandbox_workspace_write.exclude_tmpdir_env_var=") == Some("true")
            && effective("sandbox_workspace_write.exclude_slash_tmp=") == Some("true");
        return Ok(Some(ParentPermission::Root(
            entry.cwd,
            if development {
                SpawnPosture::Development
            } else {
                SpawnPosture::Discovery
            },
        )));
    }
    // Une attestation de projet autorise seulement la découverte du même arbre.
    Ok(agent_communication_project(st, name).map(ParentPermission::SameProject))
}

fn can_own_task(
    st: &DaemonState,
    task: &Task,
    owner: &str,
    instance: &str,
) -> Result<bool, String> {
    if task.owner != owner {
        return Ok(false);
    }
    if task.owner_instance == instance {
        return Ok(true);
    }
    // Un grant ne permet jamais de reprendre une conversation encore vivante.
    if st
        .presences
        .get(&task.owner_instance)
        .is_some_and(|presence| matches!(presence.state.as_str(), "connected" | "busy"))
        || st.conn_instances.iter().any(|(conn, existing)| {
            existing == &task.owner_instance
                && !st.auxiliary_connections.contains(conn)
                && st.connections.contains_key(conn)
        })
    {
        return Ok(false);
    }
    if st.delegation_store.revoked_agent(owner)? {
        return Ok(false);
    }
    let managed = st
        .fleet
        .desired_entry(owner)
        .map_err(|error| error.to_string())?
        .is_some_and(|entry| st.managed_by_instance.get(instance) == Some(&entry.command_id));
    Ok(managed || st.delegation_store.permission(instance)?.is_some())
}

fn recover_owner(
    st: &DaemonState,
    task: &mut Task,
    owner: &str,
    instance: &str,
) -> Result<(), String> {
    if !can_own_task(st, task, owner, instance)? {
        return Err("task_unavailable".into());
    }
    if task.owner != owner {
        return Err("task_unavailable".into());
    }
    if task.owner_instance == instance {
        return Ok(());
    }
    task.owner_instance = instance.into();
    st.delegation_store.save(task)
}

fn recover_child(st: &DaemonState, task: &mut Task, instance: &str) -> Result<bool, String> {
    if task.child_instance.as_deref() == Some(instance) {
        return Ok(true);
    }
    let Some(command) = st.managed_by_instance.get(instance) else {
        return Ok(false);
    };
    let Some(definition) = st.fleet.resolved_definition_for_command(command) else {
        return Ok(false);
    };
    if definition.digest != task.definition.digest {
        return Ok(false);
    }
    let Some(link) = st
        .fleet
        .agent_link_for_child(instance)
        .map_err(|error| error.to_string())?
    else {
        return Ok(false);
    };
    if link.delegation_id.as_deref() != Some(task.task_id.as_str())
        || (link.parent_instance_id != task.owner_instance
            && link.parent_instance_id != task.origin_owner_instance)
    {
        return Ok(false);
    }
    if link.parent_instance_id != task.owner_instance {
        st.fleet
            .transfer_agent_link(&link.link_id, &task.owner_instance, unix_timestamp())
            .map_err(|error| error.to_string())?;
    }
    task.child_instance = Some(instance.into());
    st.delegation_store.save(task)?;
    Ok(true)
}

fn authorize_cwd(
    st: &DaemonState,
    name: &str,
    instance: &str,
    cwd: &str,
    posture: SpawnPosture,
    candidate_project: Option<&bridget_transport::protocol::CommunicationProject>,
) -> Result<PathBuf, String> {
    let path = std::fs::canonicalize(cwd).map_err(|_| "cwd_unavailable")?;
    if !path.is_dir() {
        return Err("cwd_unavailable".into());
    }
    let permission = root_permission(st, name, instance)?.ok_or("delegation_grant_required")?;
    if posture == SpawnPosture::Development && permission.maximum() != SpawnPosture::Development {
        return Err("development_grant_required".into());
    }
    match permission {
        ParentPermission::Root(root, _) => {
            let root = std::fs::canonicalize(root).map_err(|_| "grant_root_unavailable")?;
            if !path.starts_with(&root) {
                return Err("cwd_outside_parent_grant".into());
            }
        }
        ParentPermission::SameProject(project) => {
            if candidate_project != Some(&project) {
                return Err("cwd_outside_parent_project".into());
            }
        }
    }
    Ok(path)
}

pub(super) fn handle(
    conn: &str,
    request: NativeDelegationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    // Résoudre Git hors du verrou, puis comparer avec le fait vivant au puits.
    let candidate_project = if let NativeDelegationRequest::Delegate { cwd, .. } = &request {
        let host = state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .host
            .clone();
        crate::communication::resolve_communication_project(
            cwd,
            &host,
            bridget_transport::protocol::CommunicationProjectSource::Git,
            None,
        )
    } else {
        None
    };
    let drives = matches!(
        &request,
        NativeDelegationRequest::Delegate { .. } | NativeDelegationRequest::Cancel { .. }
    );
    let result = {
        let mut st = state.lock().unwrap_or_else(|error| error.into_inner());
        match handle_locked_with_project(conn, request, candidate_project.as_ref(), &mut st) {
            Ok(result) => result,
            Err(code) => failure(&code),
        }
    };
    if drives {
        tick(state);
    }
    DaemonToWrapper::NativeDelegationResult { result }
}

#[cfg(test)]
fn handle_locked(
    conn: &str,
    request: NativeDelegationRequest,
    st: &mut DaemonState,
) -> Result<Value, String> {
    handle_locked_with_project(conn, request, None, st)
}

fn handle_locked_with_project(
    conn: &str,
    request: NativeDelegationRequest,
    candidate_project: Option<&bridget_transport::protocol::CommunicationProject>,
    st: &mut DaemonState,
) -> Result<Value, String> {
    if let NativeDelegationRequest::Grant {
        agent_id,
        cwd,
        posture,
        revoke,
    } = request
    {
        if st.connection_roles.get(conn) != Some(&ConnectionRole::Client)
            || control_client_actor(st, conn).is_none()
            || st.conn_names.contains_key(conn)
            || !st.client_negotiations.get(conn).is_some_and(|client| {
                client
                    .capabilities
                    .contains(&ClientCapability::ControlStateV1)
            })
        {
            return Err("human_principal_required".into());
        }
        let route = st.router.get_agent(&agent_id).ok_or("agent_unavailable")?;
        let (_, instance) =
            live_connection_identity(st, &route.connection_id).ok_or("agent_unavailable")?;
        let path = if revoke {
            PathBuf::from(cwd)
        } else {
            std::fs::canonicalize(cwd).map_err(|_| "cwd_unavailable")?
        };
        if !revoke && !path.is_dir() {
            return Err("cwd_unavailable".into());
        }
        st.delegation_store
            .grant_agent(&agent_id, &instance, &path, posture, revoke)?;
        return Ok(
            json!({"version":1,"status":if revoke {"revoked"} else {"granted"},"agent_id":agent_id,"cwd":path,"posture":posture}),
        );
    }
    let (owner, instance) = live_connection_identity(st, conn).ok_or("identity_unavailable")?;
    match &request {
        NativeDelegationRequest::Catalogue => {
            let permission = root_permission(st, &owner, &instance)?;
            let entries: Vec<Value> = st.registry.resolved_definitions()?.into_iter()
                .filter(|(name,_)|!name.starts_with("__"))
                .map(|(name,definition)| {
                    let discovery = st.registry.for_spawn_posture(&name,SpawnPosture::Discovery).is_ok();
                    let development = permission.as_ref().is_some_and(|permission|permission.maximum()==SpawnPosture::Development)
                        && st.registry.for_spawn_posture(&name,SpawnPosture::Development).is_ok();
                    json!({"agent_type":name,"protocol":definition.protocol,"models":definition.capabilities.models,
                        "discovery":discovery && permission.is_some(),"development":development,
                        "development_refusal":if development {None} else if definition.protocol!="codex_app_server" {Some("development_protocol_unavailable")} else {Some("development_grant_required")}})
                }).collect();
            Ok(
                json!({"version":1,"providers":entries,"cwd_root":permission.as_ref().and_then(ParentPermission::root),"cwd_scope":permission.as_ref().map(ParentPermission::scope),"max_children":16,"max_depth":8,"mission_reply_limit_secs":MISSION_REPLY_LIMIT_SECS}),
            )
        }
        NativeDelegationRequest::Delegate {
            request_id,
            agent_type,
            model,
            effort,
            task,
            cwd,
            posture,
        } => {
            if request_id.is_empty()
                || request_id.len() > 128
                || request_id.chars().any(bridget_core::is_disallowed_control)
                || task.trim().is_empty()
                || task.len() > 65536
            {
                return Err("invalid_request".into());
            }
            let canonical = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
            if let Some(mut saved) = st
                .delegation_store
                .by_agent_request(&owner, request_id, &canonical)?
            {
                recover_owner(st, &mut saved, &owner, &instance)?;
                return Ok(task_view(&saved));
            }
            let frozen_cwd =
                authorize_cwd(st, &owner, &instance, cwd, *posture, candidate_project)?;
            crate::communication::project_scope(
                agent_communication_project(st, &owner).as_ref(),
                candidate_project,
                "native-child",
                None,
            )?;
            let scoped =
                st.registry
                    .for_delegation(agent_type, model, effort.as_deref(), *posture)?;
            let tasks = st.delegation_store.tasks()?;
            if tasks.len() >= 4096
                || tasks.iter().filter(|task| !task.terminal()).count() >= 128
                || tasks
                    .iter()
                    .filter(|task| task.owner_instance == instance && !task.terminal())
                    .count()
                    >= 16
            {
                return Err("task_limit".into());
            }
            let task_id = Uuid::new_v4().to_string();
            let executions = st
                .execution_store
                .recoverable_execution_ids_for_agent(&owner)
                .map_err(|error| error.to_string())?;
            if executions.len() > 1 {
                return Err("parent_execution_ambiguous".into());
            }
            let record = Task {
                task_id: task_id.clone(),
                owner,
                owner_instance: instance.clone(),
                origin_owner_instance: instance,
                parent_execution_id: executions.into_iter().next(),
                request: request.clone(),
                definition: scoped.resolved_definition(agent_type)?,
                cwd: frozen_cwd.to_string_lossy().into_owned(),
                child: Uuid::new_v4().to_string(),
                child_instance: None,
                mission: Uuid::new_v4().to_string(),
                mission_deadline_at: None,
                created_at: unix_timestamp(),
                state: "queued".into(),
                result: None,
                error: None,
                result_sent: false,
                cleanup_done: false,
                failure_sent: false,
            };
            st.delegation_store
                .insert(&record, request_id, &canonical)?;
            Ok(task_view(&record))
        }
        NativeDelegationRequest::Status { task_id } => {
            let task = st
                .delegation_store
                .get(task_id)?
                .ok_or("task_unavailable")?;
            // Une lecture reste pure. La reprise d'instance est effectuée par
            // le tick ou un rejeu de la mutation initiale, jamais par status.
            if !can_own_task(st, &task, &owner, &instance)? {
                return Err("task_unavailable".into());
            }
            Ok(task_view(&task))
        }
        NativeDelegationRequest::Cancel { task_id } => {
            let mut task = st
                .delegation_store
                .get(task_id)?
                .ok_or("task_unavailable")?;
            recover_owner(st, &mut task, &owner, &instance)?;
            if !matches!(task.state.as_str(), "cancelled" | "result_available") {
                task.state = "cancelling".into();
                st.delegation_store.save(&task)?;
            }
            Ok(task_view(&task))
        }
        NativeDelegationRequest::Grant { .. } => unreachable!(),
    }
}

fn spawn_task(st: &mut DaemonState, task: &mut Task) -> Result<(), String> {
    let NativeDelegationRequest::Delegate {
        agent_type,
        posture,
        ..
    } = &task.request
    else {
        return Err("invalid_record".into());
    };
    if std::fs::canonicalize(&task.cwd).map_err(|_| "cwd_unavailable")?
        != std::path::Path::new(&task.cwd)
    {
        return Err("cwd_changed".into());
    }
    let registry = AgentRegistry::from_resolved(agent_type, &task.definition)?;
    let order = FleetSpawnOrder {
        posture: Some(*posture),
        agent_type: agent_type.clone(),
        project: None,
        requested_name: Some(task.child.clone()),
        cwd: PathBuf::from(&task.cwd),
        persistent: true,
        command_id: task.task_id.clone(),
        issued_at: task.created_at,
        deadline_at: task.created_at.saturating_add(300),
        ownership: Some(SpawnOwnership {
            parent_instance_id: task.origin_owner_instance.clone(),
            parent_execution_id: task.parent_execution_id.clone(),
            objective_id: None,
            delegation_id: Some(task.task_id.clone()),
            project: None,
            role: "native_delegate".into(),
            max_children: Some(16),
            max_depth: Some(8),
        }),
    };
    let hosts = crate::lifecycle::SpawnHosts {
        searched_on: st.host.clone(),
        requested_from: st.host.clone(),
    };
    match crate::lifecycle::submit_spawn(
        &st.fleet,
        &registry,
        &st.source_env,
        &order,
        unix_timestamp(),
        st.recovering,
        &hosts,
    )
    .map_err(|error| error.to_string())?
    {
        SpawnDecision::Ready(mut prepared) => {
            prepared.env.insert(
                crate::wrapper::NATIVE_MISSION_BOOTSTRAP_ENV.into(),
                serde_json::to_string(&crate::wrapper::NativeMissionBootstrap {
                    instance_id: prepared.lease.instance_id.clone(),
                    mission_id: task.mission.clone(),
                })
                .map_err(|error| error.to_string())?
                .into(),
            );
            task.child_instance = Some(prepared.lease.instance_id.clone());
            task.state = "starting".into();
            st.delegation_store.save(task)?;
            let stop = Arc::new(ManagedStopControl::new());
            st.managed_by_instance.insert(
                prepared.lease.instance_id.clone(),
                prepared.lease.command_id.clone(),
            );
            st.managed_spawns.insert(
                prepared.lease.command_id.clone(),
                ManagedSpawnRecord {
                    lease: prepared.lease.clone(),
                    agent_type: prepared.agent_type.clone(),
                    requester_conns: Vec::new(),
                    wrapper_conn: None,
                    stop: Arc::clone(&stop),
                },
            );
            if st
                .managed_tx
                .send(ManagedSupervisorCommand::Start {
                    prepared: prepared.clone(),
                    stop,
                })
                .is_err()
            {
                let _ = st.fleet.fail(
                    &prepared.lease,
                    "negotiation_failed",
                    "superviseur de processus indisponible",
                );
                st.managed_spawns.remove(&task.task_id);
                st.managed_by_instance.remove(&prepared.lease.instance_id);
                st.managed_terminal_instances
                    .insert(prepared.lease.instance_id);
                return Err("managed_supervisor_unavailable".into());
            }
        }
        SpawnDecision::Accepted { .. } => {
            task.state = "mission_pending".into();
            if let Some(route) = st.router.get_agent(&task.child) {
                task.child_instance = st.conn_instances.get(&route.connection_id).cloned();
            }
            st.delegation_store.save(task)?;
        }
        SpawnDecision::Await(_) => {
            task.state = "starting".into();
            st.delegation_store.save(task)?;
        }
        SpawnDecision::Rejected(reason) => return Err(format!("spawn_refused:{reason:?}")),
        SpawnDecision::EnvelopeMismatch => return Err("spawn_envelope_mismatch".into()),
    }
    Ok(())
}

fn send_as(
    st: &mut DaemonState,
    owner: &str,
    instance: &str,
    task_scope: &str,
    message: bridget_core::BridgetMessage,
    issued_at: i64,
    controls: &mut Vec<DeferredControl>,
) -> DaemonToWrapper {
    // Identité interne dérivée de la saga admise. Jamais créée depuis un payload externe.
    let conn = format!("native-delegation:{}", message.id);
    st.conn_names.insert(conn.clone(), owner.into());
    st.conn_instances.insert(conn.clone(), instance.into());
    st.auxiliary_connections.insert(conn.clone());
    st.client_negotiations.insert(
        conn.clone(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope(task_scope),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    );
    let response = handle_idempotent_send(
        &conn,
        message.clone(),
        message.id.clone(),
        issued_at,
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: i64::MAX,
        },
        st,
        controls,
    );
    st.conn_names.remove(&conn);
    st.conn_instances.remove(&conn);
    st.auxiliary_connections.remove(&conn);
    st.client_negotiations.remove(&conn);
    response
}

pub(super) fn capture_reply(
    st: &mut DaemonState,
    conn: &str,
    message: &bridget_core::BridgetMessage,
) -> Result<bool, String> {
    if conn.starts_with("native-delegation:") {
        return Ok(false);
    }
    let Some(correlation) = message.in_reply_to.as_deref() else {
        return Ok(false);
    };
    let Some(mut task) = st.delegation_store.for_mission(correlation)? else {
        return Ok(false);
    };
    if task.child != message.from
        || task.owner != message.to
        || !sender_is_authorized(st, conn, &message.from)
    {
        return Ok(false);
    }
    let Some(instance) = st.conn_instances.get(conn) else {
        return Ok(false);
    };
    if !recover_child(st, &mut task, instance)? {
        return Ok(false);
    }
    if message.body.len() > 262144 || message.body.trim().is_empty() {
        return Err("native_result_invalid".into());
    }
    if !task.terminal() && task.state != "cancelling" && task.result.is_none() {
        task.result = Some(message.body.clone());
        task.state = "waiting_for_children".into();
    }
    // Un résultat est durable avant l'ACK au wrapper. Le parent sera réveillé
    // seulement quand tous les mandats descendants ont fini.
    st.delegation_store.save(&task)?;
    Ok(true)
}

pub(super) fn rejected(st: &DaemonState, conn: &str, mission: &str, reason: &str) {
    let Ok(Some(mut task)) = st.delegation_store.for_mission(mission) else {
        return;
    };
    if task.terminal()
        || task.state == "cancelling"
        || task.child_instance.as_deref() != st.conn_instances.get(conn).map(String::as_str)
    {
        return;
    }
    task.state = "failed".into();
    task.error = Some(reason.chars().take(1024).collect());
    if let Err(error) = st.delegation_store.save(&task) {
        error!("délégation native refus non persisté: {error}");
    }
}

pub(super) fn tick(state: &Arc<Mutex<DaemonState>>) {
    let mut controls = Vec::new();
    let mut cancellations = Vec::new();
    let mut cleanups = Vec::new();
    {
        let mut st = state.lock().unwrap_or_else(|error| error.into_inner());
        let Ok(mut tasks) = st.delegation_store.pending() else {
            return;
        };
        let owners: HashSet<String> = st
            .presences
            .iter()
            .filter(|(_, presence)| presence.state == "connected")
            .map(|(_, presence)| presence.name.clone())
            .collect();
        for owner in owners {
            if let Ok(notices) = st.delegation_store.failed_for_owner(&owner) {
                tasks.extend(notices);
            }
        }
        for mut task in tasks.iter().cloned() {
            let owner = task.owner.clone();
            let owner_proved = if let Some(instance) = st
                .router
                .get_agent(&task.owner)
                .and_then(|route| st.conn_instances.get(&route.connection_id))
                .cloned()
            {
                recover_owner(&st, &mut task, &owner, &instance).is_ok()
            } else {
                false
            };
            if let Some(instance) = st
                .router
                .get_agent(&task.child)
                .and_then(|route| st.conn_instances.get(&route.connection_id))
                .cloned()
            {
                let _ = recover_child(&st, &mut task, &instance);
            }
            if task.state == "cancelling" {
                cancellations.push(task);
                continue;
            }
            if task.state == "working" && task.mission_deadline_at.is_none() {
                let admitted_at = st
                    .execution_store
                    .execution_snapshot(&format!("execution-{}", task.mission))
                    .ok()
                    .flatten()
                    .map(|snapshot| snapshot.created_at)
                    .unwrap_or(task.created_at);
                task.mission_deadline_at =
                    Some(admitted_at.saturating_add(MISSION_REPLY_LIMIT_SECS));
                if let Err(error) = st.delegation_store.save(&task) {
                    error!("échéance native non persistée: {error}");
                    continue;
                }
            }
            if task.state == "working"
                && task
                    .mission_deadline_at
                    .is_some_and(|deadline| unix_timestamp() >= deadline)
            {
                task.state = "failed".into();
                task.error = Some("mission_reply_timeout".into());
                if let Err(error) = st.delegation_store.save(&task) {
                    error!("échéance native non persistée: {error}");
                    continue;
                }
            }
            if task.state == "failed" {
                let parent_live = owner_proved
                    && st
                        .router
                        .get_agent(&task.owner)
                        .and_then(|route| st.conn_instances.get(&route.connection_id))
                        == Some(&task.owner_instance);
                if !task.failure_sent
                    && parent_live
                    && st
                        .presences
                        .get(&task.owner_instance)
                        .is_some_and(|presence| presence.state == "connected")
                {
                    let mut notice = bridget_core::BridgetMessage::new(
                        &task.owner,
                        &task.owner,
                        format!(
                            "Délégation native {} échouée : {}",
                            task.task_id,
                            task.error.as_deref().unwrap_or("failure")
                        ),
                    );
                    notice.id = format!("native-failure-{}", task.task_id);
                    notice.origin = Some(bridget_core::MessageOrigin::System);
                    let outcome = send_as(
                        &mut st,
                        &task.owner,
                        &task.owner_instance,
                        &task.task_id,
                        notice,
                        task.created_at,
                        &mut controls,
                    );
                    if matches!(
                        outcome,
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::Accepted { .. }
                                | IdempotencyIssue::OutcomeUnknown { .. },
                            ..
                        }
                    ) {
                        task.failure_sent = true;
                        let _ = st.delegation_store.save(&task);
                    }
                }
                if !task.cleanup_done {
                    cleanups.push(task);
                }
                continue;
            }
            if task.state == "cancelled" {
                continue;
            }
            if task.state == "result_available" && task.result_sent && !task.cleanup_done {
                cleanups.push(task);
                continue;
            }
            let parent_live = owner_proved
                && st
                    .router
                    .get_agent(&task.owner)
                    .and_then(|route| st.conn_instances.get(&route.connection_id))
                    == Some(&task.owner_instance);
            if !parent_live {
                continue;
            }
            if matches!(task.state.as_str(), "queued" | "starting")
                && let Err(error) = spawn_task(&mut st, &mut task)
            {
                task.state = "failed".into();
                task.error = Some(error);
                let _ = st.delegation_store.save(&task);
            }
            if task.state == "mission_pending" {
                let NativeDelegationRequest::Delegate { task: body, .. } = &task.request else {
                    continue;
                };
                if task.child_instance.is_none() {
                    continue;
                }
                let mut message = bridget_core::BridgetMessage::new(&task.owner, &task.child, body);
                message.id = task.mission.clone();
                message.reply = true;
                message.reply_timeout = Some(3600);
                message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
                let response = send_as(
                    &mut st,
                    &task.owner,
                    &task.owner_instance,
                    &task.task_id,
                    message,
                    task.created_at,
                    &mut controls,
                );
                match response {
                    DaemonToWrapper::IdempotencyResult {
                        issue:
                            IdempotencyIssue::Accepted { .. } | IdempotencyIssue::OutcomeUnknown { .. },
                        ..
                    } => {
                        task.state = "working".into();
                        // Figer après admission de la remise, sans renouveler au rejeu.
                        let admitted_at = st
                            .execution_store
                            .execution_snapshot(&format!("execution-{}", task.mission))
                            .ok()
                            .flatten()
                            .map(|snapshot| snapshot.created_at)
                            .unwrap_or(task.created_at);
                        task.mission_deadline_at
                            .get_or_insert(admitted_at.saturating_add(MISSION_REPLY_LIMIT_SECS));
                    }
                    DaemonToWrapper::IdempotencyResult {
                        issue:
                            IdempotencyIssue::Rejected {
                                category, reason, ..
                            },
                        ..
                    } => {
                        task.state = "failed".into();
                        task.error = Some(format!("{category}:{reason}"));
                    }
                    DaemonToWrapper::Nack { reason, .. } => {
                        task.state = "failed".into();
                        task.error = Some(reason);
                    }
                    _ => {}
                }
                let _ = st.delegation_store.save(&task);
            }
            let execution_id = format!("execution-{}", task.mission);
            // completed seul ne prouve pas la réponse corrélée. On attend
            // la capture, sans inventer un succès vide.
            if task.state == "working"
                && let Ok(Some(snapshot)) = st.execution_store.execution_snapshot(&execution_id)
                && matches!(
                    snapshot.state.as_str(),
                    "failed" | "unreachable" | "interrupted"
                )
            {
                task.state = "failed".into();
                task.error = Some(snapshot.state);
                let _ = st.delegation_store.save(&task);
            }
            if task.state == "waiting_for_children" {
                let active = tasks.iter().any(|child| {
                    Some(&child.owner_instance) == task.child_instance.as_ref() && !child.terminal()
                }) || descendants_busy(&st, &task).unwrap_or(true);
                if !active {
                    task.state = "result_available".into();
                    if let Err(error) = st.delegation_store.save(&task) {
                        error!("résultat natif non publié: {error}");
                        continue;
                    }
                }
            }
            if task.state == "result_available" && !task.result_sent {
                let Some(child_instance) = task.child_instance.as_deref() else {
                    continue;
                };
                let mut response = bridget_core::BridgetMessage::new(
                    &task.child,
                    &task.owner,
                    task.result.as_deref().unwrap_or(""),
                );
                response.id = format!("native-result-{}", task.task_id);
                response.in_reply_to = Some(task.mission.clone());
                let key = crate::idempotency::IdempotencyKey::new(
                    crate::communication::issuer_scope(&task.task_id),
                    crate::idempotency::OperationKind::Send,
                    response.id.clone(),
                );
                let replayed = key.as_ref().ok().is_some_and(|key| {
                    matches!(
                        st.idempotency.lookup(key, unix_timestamp()),
                        Ok(crate::idempotency::LookupResult::Accepted { .. })
                    ) || (matches!(
                        st.idempotency.lookup(key, unix_timestamp()),
                        Ok(crate::idempotency::LookupResult::OutcomeUnknown { .. })
                    ) && st.idempotency.send_delivery(key).ok().flatten().is_some())
                });
                let result = if replayed {
                    None
                } else {
                    Some(send_as(
                        &mut st,
                        &task.child,
                        child_instance,
                        &task.task_id,
                        response,
                        task.created_at,
                        &mut controls,
                    ))
                };
                if replayed
                    || matches!(
                        result,
                        Some(DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::Accepted { .. }
                                | IdempotencyIssue::OutcomeUnknown { .. },
                            ..
                        })
                    )
                {
                    task.result_sent = true;
                    if st.delegation_store.save(&task).is_err() {
                        task.result_sent = false;
                    }
                }
            }
            if task.state == "result_available" && task.result_sent && !task.cleanup_done {
                cleanups.push(task);
            }
        }
    }
    let _ = execute_controls(controls);
    for task in cancellations {
        cancel_tree(state, &task);
    }
    for task in cleanups {
        cancel_tree(state, &task);
    }
}

fn descendants_busy(st: &DaemonState, task: &Task) -> Result<bool, String> {
    let Some(instance) = task.child_instance.clone() else {
        return Ok(false);
    };
    let mut queue = vec![instance];
    let mut visited = HashSet::new();
    let fleet = st
        .fleet
        .desired_fleet()
        .map_err(|error| error.to_string())?;
    while let Some(parent) = queue.pop() {
        if !visited.insert(parent.clone()) {
            continue;
        }
        if visited.len() > 4096 {
            return Err("descendant_limit".into());
        }
        for link in st
            .fleet
            .agent_links_for_parent(&parent)
            .map_err(|error| error.to_string())?
        {
            queue.push(link.child_instance_id.clone());
            let name = st
                .presences
                .get(&link.child_instance_id)
                .map(|presence| presence.name.clone())
                .or_else(|| {
                    fleet
                        .equipiers
                        .iter()
                        .find(|(_, entry)| {
                            entry
                                .agent_link
                                .as_ref()
                                .is_some_and(|fact| fact.link_id == link.link_id)
                        })
                        .map(|(name, _)| name.clone())
                });
            if let Some(name) = name {
                if !st
                    .execution_store
                    .recoverable_execution_ids_for_agent(&name)
                    .map_err(|error| error.to_string())?
                    .is_empty()
                    || st
                        .presences
                        .get(&link.child_instance_id)
                        .is_some_and(|presence| presence.state == "busy")
                {
                    return Ok(true);
                }
            } else if matches!(link.state, crate::idempotency::AgentLinkState::Reserved) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn cancel_tree(state: &Arc<Mutex<DaemonState>>, task: &Task) {
    let mut names = vec![task.child.clone()];
    {
        let st = state.lock().unwrap_or_else(|error| error.into_inner());
        let Ok(tasks) = st.delegation_store.tasks() else {
            return;
        };
        let mut instances = vec![task.child_instance.clone()];
        for _ in 0..8 {
            let children: Vec<Task> = tasks
                .iter()
                .filter(|candidate| {
                    instances.iter().any(|instance| {
                        instance.as_deref() == Some(candidate.owner_instance.as_str())
                    })
                })
                .cloned()
                .collect();
            let mut added = false;
            for mut child in children {
                if !names.contains(&child.child) {
                    names.push(child.child.clone());
                    instances.push(child.child_instance.clone());
                    added = true;
                }
                if !child.terminal() {
                    child.state = "cancelling".into();
                    let _ = st.delegation_store.save(&child);
                }
            }
            if !added {
                break;
            }
        }
        // Les liens flotte couvrent aussi les descendants créés par la CLI
        // propriétaire avant l'arrivée de la façade à un appel.
        let fleet = match st.fleet.desired_fleet() {
            Ok(fleet) => fleet,
            Err(_) => return,
        };
        let mut queue: Vec<String> = instances.iter().flatten().cloned().collect();
        let mut visited = HashSet::new();
        while let Some(parent) = queue.pop() {
            if !visited.insert(parent.clone()) {
                continue;
            }
            if visited.len() > 4096 {
                return;
            }
            let links = match st.fleet.agent_links_for_parent(&parent) {
                Ok(links) => links,
                Err(_) => return,
            };
            for link in links {
                queue.push(link.child_instance_id.clone());
                let name = st
                    .presences
                    .get(&link.child_instance_id)
                    .map(|presence| presence.name.clone())
                    .or_else(|| {
                        fleet
                            .equipiers
                            .iter()
                            .find(|(_, entry)| {
                                entry
                                    .agent_link
                                    .as_ref()
                                    .is_some_and(|fact| fact.link_id == link.link_id)
                            })
                            .map(|(name, _)| name.clone())
                    });
                if let Some(name) = name
                    && !names.contains(&name)
                {
                    names.push(name);
                }
            }
        }
    }
    let mut success = true;
    for name in names.iter().rev() {
        let outcome = await_managed_stop(prepare_managed_stop(state, name), name);
        if !matches!(
            outcome,
            bridget_transport::protocol::StopOutcome::Stopped
                | bridget_transport::protocol::StopOutcome::StoppedForced { .. }
                | bridget_transport::protocol::StopOutcome::NotFound
        ) {
            success = false;
        }
    }
    if success {
        let st = state.lock().unwrap_or_else(|error| error.into_inner());
        if let Ok(tasks) = st.delegation_store.tasks() {
            for mut entry in tasks {
                if names.contains(&entry.child) && entry.state == "cancelling" {
                    entry.state = "cancelled".into();
                    let _ = st.delegation_store.save(&entry);
                }
                if names.contains(&entry.child) && entry.terminal() {
                    entry.cleanup_done = true;
                    let _ = st.delegation_store.save(&entry);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    const OWNER: &str = "89000000-0000-4000-8000-000000000102";

    fn fixture(
        label: &str,
    ) -> (
        DaemonState,
        DaemonConfig,
        Receiver<ManagedSupervisorCommand>,
        BufReader<UnixStream>,
    ) {
        let label = format!(
            "n148-{}-{}",
            &label[..label.len().min(3)],
            &Uuid::new_v4().to_string()[..8]
        );
        let (mut st, config) = super::super::presence_tests::state_with_registered_agent(&label);
        let (writer, reader) = super::super::presence_tests::control_socket(&label);
        st.connections.insert("conn-1".into(), writer);
        let fleet_root = st.fixture_root.as_ref().unwrap().0.clone();
        std::fs::set_permissions(&fleet_root, std::fs::Permissions::from_mode(0o700)).unwrap();
        st.fleet = Arc::new(
            FleetSupervisor::open(
                &config.db_path,
                DesiredStateStore::at_path(fleet_root.join("fleet.json")),
                FleetConfig::from_env(),
            )
            .unwrap(),
        );
        let (tx, rx) = mpsc::channel();
        st.managed_tx = tx;
        st.registry=AgentRegistry::from_json(r#"{"agents":{"native-test":{"command":"/bin/sh","args":["app-server"],"protocol":"codex_app_server","forbidden_env":[],"capabilities":{"execution_paths":["codex_app_server"],"models":{"custom/model":{"efforts":["high"]}}}}}}"#,Path::new("/tmp/native148-registry.json")).unwrap();
        st.delegation_store
            .grant(
                "instance-1",
                std::env::temp_dir().as_path(),
                SpawnPosture::Development,
                false,
            )
            .unwrap();
        (st, config, rx, reader)
    }
    fn request(key: &str) -> NativeDelegationRequest {
        NativeDelegationRequest::Delegate {
            request_id: key.into(),
            agent_type: "native-test".into(),
            model: "custom/model".into(),
            effort: Some("high".into()),
            task: "Inspecte les faits locaux".into(),
            cwd: std::env::temp_dir().to_string_lossy().into_owned(),
            posture: SpawnPosture::Development,
        }
    }
    fn cleanup(config: &DaemonConfig) {
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn native148_attested_git_discovery_accepts_repo_and_external_worktree_only() {
        use bridget_transport::protocol::CommunicationProjectSource;
        let (mut st, config, rx, _reader) = fixture("git-scope");
        let root = st.fixture_root.as_ref().unwrap().0.clone();
        let repo = root.join("repo");
        let worktree = root.join("outside-worktree");
        let foreign = root.join("foreign");
        for directory in [&repo, &foreign] {
            std::fs::create_dir(directory).unwrap();
            assert!(
                std::process::Command::new("git")
                    .args(["init", "--quiet"])
                    .arg(directory)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args([
                    "worktree",
                    "add",
                    "--quiet",
                    "--orphan",
                    "-b",
                    "native148-fixture"
                ])
                .arg(&worktree)
                .status()
                .unwrap()
                .success()
        );
        let canonical_repo = repo.canonicalize().unwrap();
        let canonical_worktree = worktree.canonicalize().unwrap();
        assert!(
            !canonical_worktree.starts_with(&canonical_repo),
            "worktree réellement externe à la racine du dépôt"
        );
        let project = crate::communication::resolve_communication_project(
            repo.to_str().unwrap(),
            &st.host,
            CommunicationProjectSource::Git,
            None,
        )
        .unwrap();
        let worktree_project = crate::communication::resolve_communication_project(
            worktree.to_str().unwrap(),
            &st.host,
            CommunicationProjectSource::Git,
            None,
        )
        .unwrap();
        assert_eq!(project, worktree_project);
        assert_eq!(
            PathBuf::from(&project.root),
            repo.join(".git").canonicalize().unwrap(),
            "donnée Git réelle : le projet est common-dir, pas cwd"
        );
        rusqlite::Connection::open(&config.db_path)
            .unwrap()
            .execute("DELETE FROM native_delegation_grants", [])
            .unwrap();
        st.conn_hosts.insert("conn-1".into(), st.host.clone());
        st.presences.get_mut("instance-1").unwrap().host = st.host.clone();
        let shared = Arc::new(Mutex::new(st));
        let announced = announce_communication_project(
            &shared,
            "conn-1",
            repo.to_str().unwrap(),
            CommunicationProjectSource::Git,
            &project.host,
            None,
        );
        assert!(matches!(
            announced,
            DaemonToWrapper::ProjectContextResult {
                project: Some(_),
                ..
            }
        ));
        let DaemonToWrapper::NativeDelegationResult { result: catalogue } =
            handle("conn-1", NativeDelegationRequest::Catalogue, &shared)
        else {
            panic!("catalogue attendu")
        };
        assert_eq!(catalogue["cwd_scope"], "same_project");
        assert!(catalogue["cwd_root"].is_null());
        assert!(
            catalogue["providers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["agent_type"] == "native-test"
                    && entry["discovery"] == true
                    && entry["development"] == false)
        );
        for (key, cwd) in [("repo", &repo), ("worktree", &worktree)] {
            let mut request = request(key);
            let NativeDelegationRequest::Delegate {
                cwd: path, posture, ..
            } = &mut request
            else {
                unreachable!()
            };
            *path = cwd.to_string_lossy().into_owned();
            *posture = SpawnPosture::Discovery;
            let DaemonToWrapper::NativeDelegationResult { result } =
                handle("conn-1", request, &shared)
            else {
                panic!("reçu attendu")
            };
            assert_eq!(result["status"], "queued");
            assert!(matches!(
                rx.try_recv().unwrap(),
                ManagedSupervisorCommand::Start { .. }
            ));
        }
        let mut outsider = request("foreign");
        let NativeDelegationRequest::Delegate { cwd, posture, .. } = &mut outsider else {
            unreachable!()
        };
        *cwd = foreign.to_string_lossy().into_owned();
        *posture = SpawnPosture::Discovery;
        let DaemonToWrapper::NativeDelegationResult { result } =
            handle("conn-1", outsider, &shared)
        else {
            panic!("refus attendu")
        };
        assert_eq!(result["code"], "cwd_outside_parent_project");
        assert!(rx.try_recv().is_err());
        {
            let st = shared.lock().unwrap();
            assert_eq!(st.delegation_store.tasks().unwrap().len(), 2);
            st.delegation_store
                .grant_agent(OWNER, "instance-1", &repo, SpawnPosture::Discovery, true)
                .unwrap();
        }
        let mut revoked = request("revoked-repo");
        let NativeDelegationRequest::Delegate { cwd, posture, .. } = &mut revoked else {
            unreachable!()
        };
        *cwd = repo.to_string_lossy().into_owned();
        *posture = SpawnPosture::Discovery;
        let DaemonToWrapper::NativeDelegationResult { result } = handle("conn-1", revoked, &shared)
        else {
            panic!("refus attendu")
        };
        assert_eq!(result["code"], "delegation_grant_required");
        assert!(rx.try_recv().is_err());
        {
            let st = shared.lock().unwrap();
            st.delegation_store
                .grant_agent(
                    OWNER,
                    "instance-1",
                    &canonical_repo,
                    SpawnPosture::Discovery,
                    false,
                )
                .unwrap();
        }
        let mut outside_root = request("granted-root");
        let NativeDelegationRequest::Delegate { cwd, posture, .. } = &mut outside_root else {
            unreachable!()
        };
        *cwd = worktree.to_string_lossy().into_owned();
        *posture = SpawnPosture::Discovery;
        let DaemonToWrapper::NativeDelegationResult { result } =
            handle("conn-1", outside_root, &shared)
        else {
            panic!("refus attendu")
        };
        assert_eq!(result["code"], "cwd_outside_parent_grant");
        assert!(rx.try_recv().is_err());
        let DaemonToWrapper::NativeDelegationResult { result: catalogue } =
            handle("conn-1", NativeDelegationRequest::Catalogue, &shared)
        else {
            panic!("catalogue attendu")
        };
        assert_eq!(catalogue["cwd_scope"], "root");
        assert_eq!(
            catalogue["cwd_root"],
            canonical_repo.to_string_lossy().as_ref()
        );
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .tasks()
                .unwrap()
                .len(),
            2
        );
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_live_original_owner_blocks_replay_status_and_cancel() {
        let (mut st, config, _rx, _reader) = fixture("live-original");
        let first = handle_locked("conn-1", request("live-original"), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_owned();
        let presence = st.presences.get("instance-1").unwrap().clone();
        st.presences.insert("instance-2".into(), presence);
        st.connections
            .insert("old-primary".into(), st.connections["conn-1"].clone());
        st.conn_instances
            .insert("old-primary".into(), "instance-1".into());
        st.conn_instances
            .insert("conn-1".into(), "instance-2".into());
        st.delegation_store
            .grant_agent(
                OWNER,
                "instance-2",
                Path::new("/tmp"),
                SpawnPosture::Development,
                false,
            )
            .unwrap();
        for request in [
            request("live-original"),
            NativeDelegationRequest::Status {
                task_id: task_id.clone(),
            },
            NativeDelegationRequest::Cancel {
                task_id: task_id.clone(),
            },
        ] {
            assert_eq!(
                handle_locked("conn-1", request, &mut st).unwrap_err(),
                "task_unavailable"
            );
        }
        st.presences.remove("instance-1");
        assert_eq!(
            handle_locked("conn-1", request("live-original"), &mut st).unwrap_err(),
            "task_unavailable",
            "la connexion primaire suffit aussi à fermer la reprise"
        );
        assert_eq!(
            st.delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .owner_instance,
            "instance-1"
        );
        assert_eq!(
            st.delegation_store.get(&task_id).unwrap().unwrap().state,
            "queued"
        );
        st.connections.remove("old-primary");
        st.conn_instances.remove("old-primary");
        assert_eq!(
            handle_locked("conn-1", request("live-original"), &mut st).unwrap()["task_id"],
            first["task_id"]
        );
        cleanup(&config);
    }

    #[test]
    fn native148_agent_revocation_survives_a_new_instance_until_explicit_grant() {
        use bridget_transport::protocol::CommunicationProject;
        let (mut st, config, _rx, _reader) = fixture("stable-revoke");
        st.delegation_store
            .grant_agent(
                OWNER,
                "instance-1",
                Path::new("/tmp"),
                SpawnPosture::Development,
                true,
            )
            .unwrap();
        assert!(
            crate::delegation::DelegationStore::open(&config.db_path)
                .unwrap()
                .revoked_agent(OWNER)
                .unwrap()
        );
        let presence = st.presences.remove("instance-1").unwrap();
        st.presences.insert("instance-2".into(), presence);
        st.conn_instances
            .insert("conn-1".into(), "instance-2".into());
        let project = CommunicationProject {
            host: st.host.clone(),
            root: std::fs::canonicalize("/tmp")
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        };
        st.communication_projects.insert(
            "conn-1".into(),
            (Some(project.clone()), Some(project), false),
        );
        assert!(root_permission(&st, OWNER, "instance-2").unwrap().is_none());
        // Un droit d'instance ne supprime pas la décision humaine stable.
        st.delegation_store
            .grant(
                "instance-2",
                Path::new("/tmp"),
                SpawnPosture::Development,
                false,
            )
            .unwrap();
        assert!(root_permission(&st, OWNER, "instance-2").unwrap().is_none());
        assert_eq!(
            handle_locked("conn-1", request("revoked-new"), &mut st).unwrap_err(),
            "delegation_grant_required"
        );
        st.delegation_store
            .grant_agent(
                OWNER,
                "instance-2",
                Path::new("/tmp"),
                SpawnPosture::Development,
                false,
            )
            .unwrap();
        assert!(root_permission(&st, OWNER, "instance-2").unwrap().is_some());
        cleanup(&config);
    }

    #[test]
    fn native148_capture_database_failure_never_acks_or_delivers_directly() {
        use std::io::BufRead;
        let (mut st, config, _rx, mut reader) = fixture("capture-fault");
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        handle_locked("conn-1", request("capture-fault"), &mut st).unwrap();
        let mut task = st.delegation_store.tasks().unwrap().pop().unwrap();
        task.child = OWNER.into();
        task.child_instance = Some("instance-1".into());
        task.state = "working".into();
        st.delegation_store.save(&task).unwrap();
        let fault = rusqlite::Connection::open(&config.db_path).unwrap();
        fault.execute_batch("CREATE TRIGGER native148_capture_fail BEFORE UPDATE ON native_delegations BEGIN SELECT RAISE(FAIL, 'fixture write failure'); END;").unwrap();
        let mut response = bridget_core::BridgetMessage::new(OWNER, OWNER, "résultat à retenir");
        response.in_reply_to = Some(task.mission.clone());
        let shared = Arc::new(Mutex::new(st));
        let legacy =
            handle_wrapper_message("conn-1", WrapperToDaemon::Send(response.clone()), &shared)
                .unwrap();
        assert!(
            matches!(legacy, DaemonToWrapper::Nack { reason, .. } if reason=="native_result_not_persisted")
        );
        {
            let mut st = shared.lock().unwrap();
            st.client_negotiations.insert(
                "conn-1".into(),
                NegotiatedClient {
                    version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: crate::communication::issuer_scope("fault-test"),
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            let result = handle_idempotent_send(
                "conn-1",
                response.clone(),
                response.id.clone(),
                unix_timestamp(),
                IdempotentSendAdmission {
                    project: None,
                    issued_at_tolerance_secs: 300,
                },
                &mut st,
                &mut Vec::new(),
            );
            assert!(
                matches!(result, DaemonToWrapper::Nack { reason, .. } if reason=="native_result_not_persisted")
            );
            let saved = st.delegation_store.get(&task.task_id).unwrap().unwrap();
            assert_eq!(saved.state, "working");
            assert!(saved.result.is_none());
        }
        let mut line = String::new();
        assert!(
            reader.read_line(&mut line).is_err(),
            "aucune réponse ne sort avant sa persistance"
        );
        fault
            .execute_batch("DROP TRIGGER native148_capture_fail; DROP TABLE native_delegations;")
            .unwrap();
        let reading =
            handle_wrapper_message("conn-1", WrapperToDaemon::Send(response), &shared).unwrap();
        assert!(
            matches!(reading, DaemonToWrapper::Nack { reason, .. } if reason=="native_result_not_persisted")
        );
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_completed_without_reply_expires_at_its_frozen_mission_deadline() {
        let (mut st, config, _rx, _reader) = fixture("deadline");
        handle_locked("conn-1", request("deadline"), &mut st).unwrap();
        let mut task = st.delegation_store.tasks().unwrap().pop().unwrap();
        task.state = "working".into();
        task.mission_deadline_at = Some(unix_timestamp() - 1);
        st.delegation_store.save(&task).unwrap();
        let mut message = bridget_core::BridgetMessage::new(OWNER, &task.child, "mission");
        message.id = task.mission.clone();
        let execution = format!("execution-{}", task.mission);
        st.execution_store
            .admit_starting_message_for_project(&message, &execution, None, unix_timestamp() - 3601)
            .unwrap();
        st.execution_store
            .transition_if_current(
                &execution,
                "starting",
                0,
                1,
                "completed",
                "fixture",
                unix_timestamp(),
            )
            .unwrap();
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        let st = shared.lock().unwrap();
        let saved = st.delegation_store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(saved.state, "failed");
        assert_eq!(saved.error.as_deref(), Some("mission_reply_timeout"));
        assert_eq!(saved.mission_deadline_at, task.mission_deadline_at);
        assert!(saved.cleanup_done);
        drop(st);
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_other_project_is_refused_before_admission() {
        use bridget_transport::protocol::CommunicationProject;
        let (mut st, config, rx, _reader) = fixture("project");
        let parent = CommunicationProject {
            host: st.host.clone(),
            root: "/project-one/.git".into(),
        };
        let other = CommunicationProject {
            host: st.host.clone(),
            root: "/project-two/.git".into(),
        };
        st.communication_projects
            .insert("conn-1".into(), (Some(parent.clone()), Some(parent), false));
        let error =
            handle_locked_with_project("conn-1", request("other-project"), Some(&other), &mut st)
                .unwrap_err();
        assert!(error.starts_with("cross_project_reason_required"));
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        assert!(rx.try_recv().is_err());
        cleanup(&config);
    }

    #[test]
    fn native148_cancel_stops_native_descendants_before_parent() {
        let (mut st, config, rx, _reader) = fixture("cancel-tree");
        handle_locked("conn-1", request("cancel-tree"), &mut st).unwrap();
        let mut root = st.delegation_store.tasks().unwrap().pop().unwrap();
        spawn_task(&mut st, &mut root).unwrap();
        let ManagedSupervisorCommand::Start {
            stop: root_stop, ..
        } = rx.try_recv().unwrap()
        else {
            panic!("Start attendu")
        };
        let mut descendant = root.clone();
        descendant.task_id = Uuid::new_v4().to_string();
        descendant.owner = root.child.clone();
        descendant.owner_instance = root.child_instance.clone().unwrap();
        descendant.origin_owner_instance = descendant.owner_instance.clone();
        descendant.child = Uuid::new_v4().to_string();
        descendant.child_instance = None;
        descendant.mission = Uuid::new_v4().to_string();
        descendant.state = "queued".into();
        descendant.request = request("cancel-descendant");
        st.delegation_store
            .insert(
                &descendant,
                "cancel-descendant",
                &serde_json::to_vec(&descendant.request).unwrap(),
            )
            .unwrap();
        spawn_task(&mut st, &mut descendant).unwrap();
        let ManagedSupervisorCommand::Start {
            stop: descendant_stop,
            ..
        } = rx.try_recv().unwrap()
        else {
            panic!("Start descendant attendu")
        };
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Cancel {
                task_id: root.task_id.clone(),
            },
            &mut st,
        )
        .unwrap();
        let completion = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !descendant_stop.is_requested() {
                assert!(
                    !root_stop.is_requested(),
                    "le descendant doit être arrêté avant son parent"
                );
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
            descendant_stop.complete(StopOutcome::Stopped);
            while !root_stop.is_requested() {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
            root_stop.complete(StopOutcome::Stopped);
        });
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        completion.join().unwrap();
        let st = shared.lock().unwrap();
        for id in [&root.task_id, &descendant.task_id] {
            let task = st.delegation_store.get(id).unwrap().unwrap();
            assert_eq!(task.state, "cancelled");
            assert!(task.cleanup_done);
        }
        assert!(st.delegation_store.pending().unwrap().is_empty());
        drop(st);
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_failed_cleanup_does_not_require_a_live_proved_parent() {
        let (mut st, config, rx, _reader) = fixture("failed-cleanup");
        handle_locked("conn-1", request("failed-cleanup"), &mut st).unwrap();
        let mut task = st.delegation_store.tasks().unwrap().pop().unwrap();
        spawn_task(&mut st, &mut task).unwrap();
        let ManagedSupervisorCommand::Start { stop, .. } = rx.try_recv().unwrap() else {
            panic!("Start attendu")
        };
        task.state = "failed".into();
        task.error = Some("delivery_failed".into());
        st.delegation_store.save(&task).unwrap();
        let presence = st.presences.remove("instance-1").unwrap();
        st.presences.insert("unproved-instance".into(), presence);
        st.conn_instances
            .insert("conn-1".into(), "unproved-instance".into());
        let completion = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !stop.is_requested() {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
            stop.complete(StopOutcome::Stopped);
        });
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        completion.join().unwrap();
        let st = shared.lock().unwrap();
        let saved = st.delegation_store.get(&task.task_id).unwrap().unwrap();
        assert!(saved.cleanup_done);
        assert!(!saved.failure_sent);
        assert_eq!(saved.owner_instance, "instance-1");
        assert!(st.delegation_store.pending().unwrap().is_empty());
        assert_eq!(
            st.delegation_store.failed_for_owner(OWNER).unwrap().len(),
            1
        );
        drop(st);
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_retry_ten_times_one_task_and_one_spawn() {
        let (mut st, config, rx, _reader) = fixture("replay");
        let first = handle_locked("conn-1", request("stable"), &mut st).unwrap();
        for _ in 0..10 {
            assert_eq!(
                handle_locked("conn-1", request("stable"), &mut st).unwrap(),
                first
            );
        }
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
            panic!("spawn attendu")
        };
        assert_eq!(
            crate::registry::runtime_model_and_effort(&prepared.args),
            Some(("custom/model".into(), Some("high".into())))
        );
        assert_eq!(prepared.resolved_definition.permissions, "deny");
        for _ in 0..10 {
            tick(&shared);
        }
        assert!(rx.try_recv().is_err());
        assert_eq!(
            shared.lock().unwrap().delegation_store.tasks().unwrap()[0].state,
            "starting"
        );
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_replay_precedes_grant_revocation_and_definition_change() {
        let (mut st, config, _rx, _reader) = fixture("replay-grant");
        let first = handle_locked("conn-1", request("stable"), &mut st).unwrap();
        st.delegation_store
            .grant(
                "instance-1",
                Path::new("/tmp"),
                SpawnPosture::Development,
                true,
            )
            .unwrap();
        st.registry = AgentRegistry::from_json("{}", "/tmp/empty148.json").unwrap();
        assert_eq!(
            handle_locked("conn-1", request("stable"), &mut st).unwrap(),
            first
        );
        assert_eq!(
            handle_locked("conn-1", request("new"), &mut st).unwrap_err(),
            "delegation_grant_required"
        );
        let mut divergent = request("stable");
        if let NativeDelegationRequest::Delegate { task, .. } = &mut divergent {
            *task = "autre mission".into();
        }
        assert_eq!(
            handle_locked("conn-1", divergent, &mut st).unwrap_err(),
            "envelope_mismatch"
        );
        cleanup(&config);
    }

    #[test]
    fn native148_parent_and_model_refusals_have_no_effect() {
        let (mut st, config, rx, _reader) = fixture("refusals");
        assert_eq!(
            handle_locked("unknown", request("missing"), &mut st).unwrap_err(),
            "identity_unavailable"
        );
        let mut wrong_model = request("wrong-model");
        if let NativeDelegationRequest::Delegate { model, .. } = &mut wrong_model {
            *model = "custom/model-alias".into();
        }
        assert_eq!(
            handle_locked("conn-1", wrong_model, &mut st).unwrap_err(),
            "model_unavailable"
        );
        let mut wrong_effort = request("wrong-effort");
        if let NativeDelegationRequest::Delegate { effort, .. } = &mut wrong_effort {
            *effort = Some("xhigh".into());
        }
        assert_eq!(
            handle_locked("conn-1", wrong_effort, &mut st).unwrap_err(),
            "effort_unavailable"
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        assert!(rx.try_recv().is_err());
        cleanup(&config);
    }

    #[test]
    fn native148_cwd_and_development_grants_are_closed() {
        let (mut st, config, _rx, _reader) = fixture("permissions");
        st.delegation_store
            .grant(
                "instance-1",
                std::env::temp_dir().as_path(),
                SpawnPosture::Discovery,
                false,
            )
            .unwrap();
        assert_eq!(
            handle_locked("conn-1", request("dev"), &mut st).unwrap_err(),
            "development_grant_required"
        );
        let mut outside = request("outside");
        if let NativeDelegationRequest::Delegate { cwd, posture, .. } = &mut outside {
            *cwd = "/".into();
            *posture = SpawnPosture::Discovery;
        }
        assert_eq!(
            handle_locked("conn-1", outside, &mut st).unwrap_err(),
            "cwd_outside_parent_grant"
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        cleanup(&config);
    }

    #[test]
    fn native148_only_human_can_grant() {
        let (mut st, config, _rx, _reader) = fixture("grant");
        let grant = NativeDelegationRequest::Grant {
            agent_id: OWNER.into(),
            cwd: std::env::temp_dir().to_string_lossy().into_owned(),
            posture: SpawnPosture::Development,
            revoke: false,
        };
        assert_eq!(
            handle_locked("conn-1", grant.clone(), &mut st).unwrap_err(),
            "human_principal_required"
        );
        st.connection_roles
            .insert("human".into(), ConnectionRole::Client);
        st.client_negotiations.insert(
            "human".into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("bridget-control-cli"),
                capabilities: vec![ClientCapability::ControlStateV1],
            },
        );
        assert_eq!(
            handle_locked("human", grant, &mut st).unwrap()["status"],
            "granted"
        );
        cleanup(&config);
    }

    #[test]
    fn native148_status_cancel_are_owner_scoped_and_repeatable() {
        let (mut st, config, _rx, _reader) = fixture("owner");
        let first = handle_locked("conn-1", request("owner"), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_owned();
        assert_eq!(
            handle_locked(
                "unknown",
                NativeDelegationRequest::Status {
                    task_id: task_id.clone()
                },
                &mut st
            )
            .unwrap_err(),
            "identity_unavailable"
        );
        let cancelled = handle_locked(
            "conn-1",
            NativeDelegationRequest::Cancel {
                task_id: task_id.clone(),
            },
            &mut st,
        )
        .unwrap();
        assert_eq!(cancelled["status"], "cancelling");
        assert_eq!(
            handle_locked(
                "conn-1",
                NativeDelegationRequest::Cancel {
                    task_id: task_id.clone()
                },
                &mut st
            )
            .unwrap(),
            cancelled
        );
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .state,
            "cancelled"
        );
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_cut_after_admission_retains_exact_ids_and_frozen_model() {
        let (mut st, config, _rx, _reader) = fixture("restart");
        let first = handle_locked("conn-1", request("restart"), &mut st).unwrap();
        let store = crate::delegation::DelegationStore::open(&config.db_path).unwrap();
        let restored = store
            .by_request(
                "instance-1",
                "restart",
                &serde_json::to_vec(&request("restart")).unwrap(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(task_view(&restored), first);
        assert_eq!(
            crate::registry::runtime_model_and_effort(&restored.definition.args),
            Some(("custom/model".into(), Some("high".into())))
        );
        cleanup(&config);
    }

    #[test]
    fn native148_reincarnated_parent_requires_proof_and_reuses_the_same_task() {
        let (mut st, config, _rx, _reader) = fixture("new-instance");
        let first = handle_locked("conn-1", request("stable-new-instance"), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_owned();
        let presence = st.presences.remove("instance-1").unwrap();
        st.presences.insert("instance-2".into(), presence);
        st.conn_instances
            .insert("conn-1".into(), "instance-2".into());
        assert_eq!(
            handle_locked("conn-1", request("stable-new-instance"), &mut st).unwrap_err(),
            "task_unavailable"
        );
        st.delegation_store
            .grant(
                "instance-2",
                Path::new("/tmp"),
                SpawnPosture::Development,
                false,
            )
            .unwrap();
        assert_eq!(
            handle_locked(
                "conn-1",
                NativeDelegationRequest::Status {
                    task_id: task_id.clone()
                },
                &mut st
            )
            .unwrap(),
            first
        );
        assert_eq!(
            st.delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .owner_instance,
            "instance-1",
            "status ne modifie pas la saga"
        );
        assert_eq!(
            handle_locked("conn-1", request("stable-new-instance"), &mut st).unwrap(),
            first
        );
        assert_eq!(
            st.delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .owner_instance,
            "instance-2"
        );
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
        cleanup(&config);
    }

    #[test]
    fn native148_glm_catalogue_uses_native_registry_and_discovery_permissions() {
        let (mut st, config, rx, _reader) = fixture("glm");
        st.registry=AgentRegistry::from_json(r#"{"agents":{"glm-fake":{"command":"/bin/sh","args":["--model","glm.old","--dangerously-skip-permissions"],"protocol":"claude_stream_json","capabilities":{"execution_paths":["claude_stream_json"],"models":{"glm/custom-exact":{"efforts":[]}}}}}}"#,"/tmp/native-glm148.json").unwrap();
        let catalogue =
            handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
        let glm = catalogue["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["agent_type"] == "glm-fake")
            .unwrap();
        assert!(glm["models"]["glm/custom-exact"].is_object());
        assert_eq!(glm["discovery"], true);
        assert_eq!(glm["development"], false);
        assert_eq!(
            glm["development_refusal"],
            "development_protocol_unavailable"
        );
        let job = NativeDelegationRequest::Delegate {
            request_id: "glm".into(),
            agent_type: "glm-fake".into(),
            model: "glm/custom-exact".into(),
            effort: None,
            task: "Analyse les faits".into(),
            cwd: std::env::temp_dir().to_string_lossy().into_owned(),
            posture: SpawnPosture::Discovery,
        };
        handle_locked("conn-1", job, &mut st).unwrap();
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
            panic!("lancement natif attendu")
        };
        assert_eq!(prepared.resolved_definition.protocol, "claude_stream_json");
        assert_eq!(prepared.resolved_definition.permissions, "deny");
        assert_eq!(
            crate::registry::runtime_model_and_effort(&prepared.args),
            Some(("glm/custom-exact".into(), None))
        );
        assert!(
            prepared
                .args
                .windows(2)
                .any(|args| args == ["--permission-mode", "plan"])
        );
        assert!(
            !prepared
                .args
                .iter()
                .any(|arg| arg == "--dangerously-skip-permissions")
        );
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_turn_completed_alone_never_makes_result_available() {
        let (mut st, config, _rx, _reader) = fixture("completion");
        handle_locked("conn-1", request("completion"), &mut st).unwrap();
        let mut task = st.delegation_store.tasks().unwrap().pop().unwrap();
        task.state = "working".into();
        st.delegation_store.save(&task).unwrap();
        let message = bridget_core::BridgetMessage::new(OWNER, &task.child, "mission");
        let mut message = message;
        message.id = task.mission.clone();
        let id = format!("execution-{}", task.mission);
        st.execution_store
            .admit_starting_message_for_project(&message, &id, None, unix_timestamp())
            .unwrap();
        st.execution_store
            .transition_if_current(&id, "starting", 0, 1, "completed", "test", unix_timestamp())
            .unwrap();
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .state,
            "working"
        );
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native148_mission_replay_and_result_wait_for_children_without_t3() {
        use std::io::BufRead;
        let (mut st, config, rx, mut parent_reader) = fixture("mission");
        let first = handle_locked("conn-1", request("mission"), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_owned();
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        let ManagedSupervisorCommand::Start { prepared, stop } = rx.try_recv().unwrap() else {
            panic!("Start attendu")
        };
        let (child_writer, mut child_reader) =
            super::super::presence_tests::control_socket("n148child");
        child_reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        parent_reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        {
            let mut st = shared.lock().unwrap();
            assert!(matches!(
                handle_register_with_channel(
                    "child-conn",
                    2,
                    "native-test".into(),
                    prepared.lease.name.clone(),
                    Some(st.host.clone()),
                    Some("codex_app_server".into()),
                    ChannelReport::Known("unix".into()),
                    Some(PresenceMode::Acp),
                    None,
                    Some("macOS".into()),
                    Some(prepared.lease.instance_id.clone()),
                    None,
                    false,
                    Some(true),
                    &mut st
                ),
                DaemonToWrapper::Registered { .. }
            ));
            st.connections.insert("child-conn".into(), child_writer);
            st.conn_instances
                .insert("child-conn".into(), prepared.lease.instance_id.clone());
            st.fleet
                .register_connected(
                    &prepared.lease,
                    &prepared.lease.instance_id,
                    unix_timestamp(),
                )
                .unwrap();
            st.managed_spawns.get_mut(&task_id).unwrap().wrapper_conn = Some("child-conn".into());
            assert_eq!(
                root_permission(&st, &prepared.lease.name, &prepared.lease.instance_id)
                    .unwrap()
                    .unwrap()
                    .maximum(),
                SpawnPosture::Development
            );
            st.delegation_store
                .grant(
                    &prepared.lease.instance_id,
                    Path::new("/tmp"),
                    SpawnPosture::Development,
                    true,
                )
                .unwrap();
            assert!(
                root_permission(&st, &prepared.lease.name, &prepared.lease.instance_id)
                    .unwrap()
                    .is_none(),
                "une révocation explicite ferme aussi l'héritage managed"
            );
            st.delegation_store
                .grant(
                    &prepared.lease.instance_id,
                    Path::new("/tmp"),
                    SpawnPosture::Development,
                    false,
                )
                .unwrap();
        }
        tick(&shared);
        let mut line = String::new();
        child_reader.read_line(&mut line).unwrap();
        let delivered: DaemonToWrapper = decode(line.trim()).unwrap();
        let DaemonToWrapper::DeliverIdempotent { message, .. } = delivered else {
            panic!("mission idempotente attendue : {delivered:?}")
        };
        assert_eq!(message.id, first["message_id"]);
        assert!(message.reply);
        assert_eq!(
            message.intent,
            Some(bridget_core::MessageIntent::TriggerTurn)
        );
        {
            let st = shared.lock().unwrap();
            let mut task = st.delegation_store.get(&task_id).unwrap().unwrap();
            assert_eq!(task.state, "working");
            // Coupure entre la remise et le checkpoint : seul son sort est relu.
            task.state = "mission_pending".into();
            st.delegation_store.save(&task).unwrap();
        }
        tick(&shared);
        line.clear();
        assert!(child_reader.read_line(&mut line).is_err());
        let mut response = bridget_core::BridgetMessage::new(
            &prepared.lease.name,
            OWNER,
            "résultat initial stable",
        );
        response.in_reply_to = Some(message.id.clone());
        let descendant_id;
        {
            let mut st = shared.lock().unwrap();
            assert!(
                !capture_reply(&mut st, "conn-1", &response).unwrap(),
                "le parent ne peut pas forger la réponse enfant"
            );
            assert!(capture_reply(&mut st, "child-conn", &response).unwrap());
            let mut descendant = st.delegation_store.get(&task_id).unwrap().unwrap();
            descendant.task_id = Uuid::new_v4().to_string();
            descendant_id = descendant.task_id.clone();
            descendant.owner = prepared.lease.name.clone();
            descendant.owner_instance = prepared.lease.instance_id.clone();
            descendant.child = Uuid::new_v4().to_string();
            descendant.child_instance = Some("grandchild-instance".into());
            descendant.mission = Uuid::new_v4().to_string();
            descendant.state = "working".into();
            descendant.result = None;
            descendant.request = request("grandchild");
            st.delegation_store
                .insert(
                    &descendant,
                    "grandchild",
                    &serde_json::to_vec(&descendant.request).unwrap(),
                )
                .unwrap();
        }
        tick(&shared);
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .state,
            "waiting_for_children"
        );
        line.clear();
        assert!(
            parent_reader.read_line(&mut line).is_err(),
            "aucun réveil du parent pendant le travail descendant"
        );
        {
            let st = shared.lock().unwrap();
            let mut child = st.delegation_store.get(&descendant_id).unwrap().unwrap();
            child.state = "result_available".into();
            child.result_sent = true;
            st.delegation_store.save(&child).unwrap();
        }
        let completion = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !stop.is_requested() {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
            stop.complete(StopOutcome::Stopped);
        });
        tick(&shared);
        completion.join().unwrap();
        line.clear();
        parent_reader.read_line(&mut line).unwrap();
        let result: DaemonToWrapper = decode(line.trim()).unwrap();
        let DaemonToWrapper::DeliverIdempotent {
            message: result, ..
        } = result
        else {
            panic!("résultat corrélé attendu")
        };
        assert_eq!(result.body, "résultat initial stable");
        assert_eq!(result.in_reply_to.as_deref(), Some(message.id.as_str()));
        {
            let mut st = shared.lock().unwrap();
            response.body = "réponse tardive divergente".into();
            assert!(capture_reply(&mut st, "child-conn", &response).unwrap());
            let mut task = st.delegation_store.get(&task_id).unwrap().unwrap();
            assert_eq!(task.result.as_deref(), Some("résultat initial stable"));
            assert_eq!(task.state, "result_available");
            // Coupure après remise du résultat mais avant son checkpoint.
            task.result_sent = false;
            st.delegation_store.save(&task).unwrap();
        }
        tick(&shared);
        line.clear();
        assert!(parent_reader.read_line(&mut line).is_err());
        drop(shared);
        cleanup(&config);
    }
}
