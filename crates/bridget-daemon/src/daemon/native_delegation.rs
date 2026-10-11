//! Effets de la saga native, sous les mêmes gardes que la flotte et les remises.
use super::*;
use crate::delegation::Task;
use bridget_transport::protocol::{NativeDelegationRequest, SpawnOwnership, SpawnPosture};
use serde_json::{Value, json};
use std::path::Path;
const MISSION_REPLY_LIMIT_SECS: i64 = 3600;
const WAIT_DEFAULT_SECS: u64 = 600;
const WAIT_MAX_SECS: u64 = 3600;
/// Bornes du délai de réponse choisi par le parent (session 150).
const MISSION_REPLY_MIN_SECS: u64 = 60;
const MISSION_REPLY_MAX_SECS: u64 = 86_400;

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

fn parent_fact(st:&DaemonState,owner:&str,instance:&str,request_id:Option<&str>,attested:Option<&bridget_transport::protocol::ProviderPermissions>)->Result<Option<bridget_transport::protocol::ProviderPermissions>,String>{
    // Une révocation directe conserve le refus148, avant toute attestation.
    if st.delegation_store.revoked_agent(owner)?||st.delegation_store.revoked(instance)?{return Err("delegation_grant_required".into())}
    if let Some(parent)=st.delegation_store.for_child(owner)? {
        if st.delegation_store.revoked_agent(&parent.root_owner_agent_id)?{return Err("permission_not_inherited".into())}
        if st.managed_by_instance.get(instance)!=Some(&parent.task_id)
            || st.fleet.resolved_definition_for_command(&parent.task_id).is_none_or(|d|d.digest!=parent.definition.digest)
            || st.fleet.agent_link_for_child(instance).map_err(|e|e.to_string())?.is_none_or(|link|link.delegation_id.as_deref()!=Some(parent.task_id.as_str())) {return Err("parent_task_unavailable".into())}
        if let Some(snapshot)=parent.permission_snapshot{
            return Ok(Some(bridget_transport::protocol::ProviderPermissions{
                version:1,source:"native_wrapper".into(),run_id:parent.mission,provider_session_id:parent.child.clone(),provider_instance_id:instance.into(),revision:1,
                driver:parent.definition.protocol,cwd:parent.cwd,runtime_mode:crate::native_permissions::runtime_mode(&snapshot.child_policy).into(),
                interaction_mode:if snapshot.child_policy["permission_mode"]=="plan"||snapshot.child_policy["sandbox_policy"]["type"]=="readOnly"{"plan"}else{"default"}.into(),provider_policy:snapshot.child_policy,
            }))
        }
    }
    if let Some(fact)=attested{return Ok(Some(fact.clone()))}
    let mut facts=st.native_permission_facts.iter().filter(|(conn,(fact,scope))|
        (scope.is_none()||scope.as_deref()==request_id)&&fact.provider_instance_id==instance&&!st.auxiliary_connections.contains(*conn)
        &&live_connection_identity(st,conn)==Some((owner.into(),instance.into())));
    let first=facts.next().map(|(_,f)|f.0.clone());
    if facts.next().is_some(){return Err("permission_attestation_unavailable".into())}
    if first.is_none(){
        for (connection,code) in &st.native_permission_refusals{
            if !st.auxiliary_connections.contains(connection)&&live_connection_identity(st,connection)==Some((owner.into(),instance.into())){return Err(code.clone())}
        }
    }
    Ok(first)
}

fn task_view(task: &Task, phase: Option<&str>, full: bool) -> Value {
    let (agent_type, model, effort, posture) = match &task.request {
        NativeDelegationRequest::Delegate {
            agent_type,
            model,
            effort,
            posture,
            ..
        } => (agent_type, model, effort, posture),
        _ => unreachable!("seules les missions sont persistées"),
    };
    let body = (task.state == "result_available")
        .then_some(task.result.as_deref())
        .flatten();
    let summary = body.map(|body| {
        &body[..crate::store::ledger_search::floor_char_boundary(body, body.len().min(2048))]
    });
    let mailbox = matches!(task.state.as_str(), "result_available" | "failed").then(|| {
        let state = if task.acknowledged_at.is_some() {
            "acknowledged"
        } else if task.delivery_acked || phase == Some("acked") {
            "delivered"
        } else if phase.is_some() || task.result_sent || task.failure_sent {
            "claimed"
        } else {
            "pending"
        };
        json!({"state":state})
    });
    let work_state = match task.state.as_str() {
        "waiting_for_children" => "waiting_for_children",
        "result_available" => "result_available",
        "failed" => "failed",
        "cancelled" => "cancelled",
        _ => "working",
    };
    let mut view = json!({"version":1,"task_id":task.task_id,"status":task.state,
        "child_agent_id":task.child,"message_id":task.mission,
        "agent_type":agent_type,"model":model,"effort":effort,"cwd":task.cwd,"posture":task.effective_posture.or(*posture),
        "mission_deadline_at":task.mission_deadline_at,
        "result":if full {body} else {None},"error":task.error,
        "work_state":work_state,"summary":summary,
        "summary_truncated":body.zip(summary).is_some_and(|(body, summary)| body.len() > summary.len()),
        "result_bytes":body.map(str::len),
        "detail":matches!(task.state.as_str(),"result_available"|"failed").then(|| json!({"tool":"bridget_task_status","task_id":task.task_id})),
        "mailbox":mailbox});
    if !full {
        view.as_object_mut().unwrap().remove("result");
    }
    view
}

/// O(log n) : lecture par la clé indexée, aucune mutation de remise.
fn current_task_view(st: &DaemonState, task: &Task, full: bool) -> Result<Value, String> {
    let phase =
        if matches!(task.state.as_str(), "result_available" | "failed") && !task.delivery_acked {
            let prefix = if task.state == "failed" {
                "native-failure-"
            } else {
                "native-result-"
            };
            let key = IdempotencyKey::new(
                &crate::communication::issuer_scope(&task.task_id),
                OperationKind::Send,
                &format!("{prefix}{}", task.task_id),
            )
            .map_err(|e| e.to_string())?;
            st.idempotency
                .send_delivery_phase(&key)
                .map_err(|e| e.to_string())?
        } else {
            None
        };
    Ok(task_view(task, phase.as_deref(), full))
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

#[cfg(test)]
pub(super) fn handle(
    conn: &str,
    request: NativeDelegationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    handle_with_peer(conn, request, state, None)
}

pub(super) fn handle_with_peer(
    conn: &str,
    request: NativeDelegationRequest,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
) -> DaemonToWrapper {
    handle_with_fact(conn, request, None, None, state, peer)
}

pub(super) fn handle_attested(
    conn: &str,
    request: NativeDelegationRequest,
    proof: &bridget_transport::protocol::NativePermissionProof,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
) -> DaemonToWrapper {
    // HTTP hors verrou. La connexion, le fil et le projet sont revérifiés
    // après la réponse authentifiée, au même puits que l'admission durable.
    let attested = crate::t3code_mcp::reattest(proof);
    let proof = match attested {
        Ok(p) => p,
        Err(_) => {
            return DaemonToWrapper::NativeDelegationResult {
                result: failure("permission_attestation_unavailable"),
            };
        }
    };
    let valid = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let expected = (
            crate::t3code::stable_uuid(&proof.thread_id),
            crate::t3code::stable_uuid(&format!("instance:{}", proof.thread_id)),
        );
        live_connection_identity(&st, conn) == Some(expected.clone())
            && human_authority_guard(
                &st,
                &proof.thread_id,
                &agent_communication_project(&st, &expected.0),
            )
            .is_ok()
    };
    if !valid {
        return DaemonToWrapper::NativeDelegationResult {
            result: failure("permission_attestation_unavailable"),
        };
    }
    handle_with_fact(
        conn,
        request,
        proof.permissions,
        Some(&proof.thread_id),
        state,
        peer,
    )
}

fn handle_with_fact(
    conn: &str,
    request: NativeDelegationRequest,
    fact: Option<bridget_transport::protocol::ProviderPermissions>,
    thread: Option<&str>,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
) -> DaemonToWrapper {
    if let NativeDelegationRequest::Wait {
        task_ids,
        wait_timeout,
    } = &request
    {
        return DaemonToWrapper::NativeDelegationResult {
            result: match wait_tasks(
                conn,
                task_ids,
                *wait_timeout,
                thread,
                state,
                peer,
                Duration::from_secs(5),
            ) {
                Ok(result) => result,
                Err(code) => failure(&code),
            },
        };
    }
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
        if let Some(thread) = thread {
            let owner = crate::t3code::stable_uuid(thread);
            let expected = (
                owner.clone(),
                crate::t3code::stable_uuid(&format!("instance:{thread}")),
            );
            if live_connection_identity(&st, conn) != Some(expected)
                || human_authority_guard(&st, thread, &agent_communication_project(&st, &owner))
                    .is_err()
            {
                return DaemonToWrapper::NativeDelegationResult {
                    result: failure("permission_attestation_unavailable"),
                };
            }
        }
        match handle_locked_with_project(
            conn,
            request,
            candidate_project.as_ref(),
            fact.as_ref(),
            &mut st,
        ) {
            Ok(result) => result,
            Err(code) => failure(&code),
        }
    };
    // Un refus ne crée ni ne modifie aucune tâche active : il ne doit pas faire
    // avancer les autres tâches (une purge de borne éventuelle ne retire que des arbres soldés).
    if drives && result["status"] != "refused" {
        tick(state);
    }
    DaemonToWrapper::NativeDelegationResult { result }
}

fn validate_task_ids(ids: &[String]) -> Result<(), String> {
    if ids.is_empty()
        || ids.len() > 16
        || ids.iter().any(|id| {
            id.is_empty() || id.len() > 128 || id.chars().any(bridget_core::is_disallowed_control)
        })
        || ids.iter().collect::<HashSet<_>>().len() != ids.len()
    {
        return Err("invalid_request".into());
    }
    Ok(())
}

/// O(k) lectures indexées par réveil, k <= 16 ; aucun verrou d'état pendant le sommeil.
fn wait_tasks(
    conn: &str,
    task_ids: &[String],
    timeout: Option<u64>,
    thread: Option<&str>,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
    slice: Duration,
) -> Result<Value, String> {
    validate_task_ids(task_ids)?;
    let timeout = timeout.unwrap_or(WAIT_DEFAULT_SECS);
    if !(1..=WAIT_MAX_SECS).contains(&timeout) {
        return Err("wait_timeout_out_of_range".into());
    }
    let start = Instant::now();
    let deadline = start + Duration::from_secs(timeout);
    let (identity, epoch, active) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        (
            live_connection_identity(&st, conn).ok_or("identity_unavailable")?,
            st.delegation_store.wait_epoch(),
            st.active_waits.clone(),
        )
    };
    let mut permit = None;
    loop {
        // Observer le compteur AVANT l'état évite de perdre un commit entre lecture et sommeil.
        let seen = epoch.get();
        if wait_client_gone(peer) || SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
            return Err("daemon_unreachable".into());
        }
        let (ready, pending) = {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            if live_connection_identity(&st, conn).as_ref() != Some(&identity) {
                return Err("task_unavailable".into());
            }
            if let Some(thread) = thread {
                human_authority_guard(&st, thread, &agent_communication_project(&st, &identity.0))
                    .map_err(|_| "task_unavailable")?;
            }
            // Valider tout le lot avant de construire une réponse, sans révéler un lot partiel.
            let mut tasks = Vec::with_capacity(task_ids.len());
            for id in task_ids {
                let task = st.delegation_store.get(id)?.ok_or("task_unavailable")?;
                if !can_own_task(&st, &task, &identity.0, &identity.1)? {
                    return Err("task_unavailable".into());
                }
                tasks.push(task);
            }
            if permit.is_none() {
                permit = Some(WaitPermit::acquire(&active)?);
            }
            let mut ready = Vec::new();
            let mut pending = Vec::new();
            for task in tasks {
                if task.terminal() {
                    ready.push(current_task_view(&st, &task, false)?);
                } else {
                    pending.push(task.task_id);
                }
            }
            (ready, pending)
        };
        if !ready.is_empty() || Instant::now() >= deadline {
            return Ok(
                json!({"version":1,"timed_out":ready.is_empty(),"waited_secs":start.elapsed().as_secs(),"ready":ready,"pending":pending}),
            );
        }
        epoch.wait_changed(
            seen,
            slice.min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

#[cfg(test)]
fn handle_locked(
    conn: &str,
    request: NativeDelegationRequest,
    st: &mut DaemonState,
) -> Result<Value, String> {
    handle_locked_with_project(conn, request, None, None, st)
}

fn handle_locked_with_project(
    conn: &str,
    request: NativeDelegationRequest,
    candidate_project: Option<&bridget_transport::protocol::CommunicationProject>,
    attested: Option<&bridget_transport::protocol::ProviderPermissions>,
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
            let fact=parent_fact(st,&owner,&instance,None,attested)?;
            let entries: Vec<Value> = st.registry.resolved_definitions()?.into_iter()
                .filter(|(name,_)|!name.starts_with("__"))
                .map(|(name,definition)| {
                    let discovery = st.registry.for_spawn_posture(&name,SpawnPosture::Discovery).is_ok();
                    let inherited=fact.as_ref().ok_or_else(||"permission_attestation_unavailable".to_string()).and_then(|fact|
                        crate::native_permissions::child_policy(fact,st.registry.get(&name)?,Path::new(&fact.cwd),&st.source_env,None));
                    let development=fact.as_ref().ok_or_else(||"permission_attestation_unavailable".to_string()).and_then(|fact|
                        crate::native_permissions::child_policy(fact,st.registry.get(&name)?,Path::new(&fact.cwd),&st.source_env,Some(SpawnPosture::Development)));
                    json!({"agent_type":name,"protocol":definition.protocol,"models":definition.capabilities.models,
                        "inherit":inherited.is_ok(),"inherit_refusal":inherited.err(),
                        "discovery":discovery && (permission.is_some()||fact.is_some()),"development":development.as_ref().is_ok_and(|(_,p)|*p==SpawnPosture::Development),
                        "development_refusal":development.err()})
                }).collect();
            Ok(
                json!({"version":1,"providers":entries,"cwd_root":fact.as_ref().map(|f|Path::new(&f.cwd)).or_else(||permission.as_ref().and_then(ParentPermission::root)),"cwd_scope":if fact.is_some(){Some("project")}else{permission.as_ref().map(ParentPermission::scope)},"max_children":16,"max_depth":8,"mission_reply_limit_secs":MISSION_REPLY_LIMIT_SECS,"mission_reply_min_secs":MISSION_REPLY_MIN_SECS,"mission_reply_max_secs":MISSION_REPLY_MAX_SECS,"wait_timeout_default_secs":WAIT_DEFAULT_SECS,"wait_timeout_max_secs":WAIT_MAX_SECS}),
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
            reply_timeout,
        } => {
            if request_id.is_empty()
                || request_id.len() > 128
                || request_id.chars().any(bridget_core::is_disallowed_control)
                || task.trim().is_empty()
                || task.len() > 65536
            {
                return Err("invalid_request".into());
            }
            if reply_timeout.is_some_and(|secs| !(MISSION_REPLY_MIN_SECS..=MISSION_REPLY_MAX_SECS).contains(&secs)) {
                return Err("reply_timeout_out_of_range".into());
            }
            let canonical = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
            if let Some(mut saved) = st
                .delegation_store
                .by_agent_request(&owner, request_id, &canonical)?
            {
                recover_owner(st, &mut saved, &owner, &instance)?;
                return current_task_view(st, &saved, true);
            }
            // ADR 027 : garde unique des puits autonomes. Après le rejeu (une clé admise ou
            // purgée garde sa réponse), avant toute lecture de droits ; aucune écriture.
            let control = crate::referent_control::read(st.store.connection())
                .map_err(|_| "control_state_unavailable")?;
            if matches!(
                crate::referent_control::admit_autonomous_effect(crate::referent_control::AutonomousEffect::NativeDelegation, &control),
                crate::referent_control::Admission::Deferred { .. }
            ) {
                return Err("referent_paused".into());
            }
            let fact=parent_fact(st,&owner,&instance,Some(request_id),attested)?;
            let frozen_cwd=if let Some(fact)=&fact{
                let path=std::fs::canonicalize(cwd).map_err(|_|"cwd_unavailable")?;
                let root=std::fs::canonicalize(&fact.cwd).map_err(|_|"cwd_unavailable")?;
                if !path.is_dir()||!path.starts_with(&root){return Err("cwd_outside_parent_project".into())}
                path
            }else if *posture==Some(SpawnPosture::Discovery){
                authorize_cwd(st,&owner,&instance,cwd,SpawnPosture::Discovery,candidate_project)?
            }else{return Err("permission_attestation_unavailable".into())};
            crate::communication::project_scope(
                agent_communication_project(st, &owner).as_ref(),
                candidate_project,
                "native-child",
                None,
            )?;
            let (scoped,permission_snapshot,effective_posture)=if let Some(fact)=fact{
                let target=st.registry.get(agent_type)?;
                let (policy,effective)=crate::native_permissions::child_policy(&fact,target,&frozen_cwd,&st.source_env,*posture)?;
                let scoped=st.registry.for_inherited_delegation(agent_type,model,effort.as_deref(),&policy)?;
                (scoped,Some(crate::native_permissions::snapshot(&owner,&instance,fact,policy,&frozen_cwd)),Some(effective))
            }else{(st.registry.for_delegation(agent_type,model,effort.as_deref(),SpawnPosture::Discovery)?,None,Some(SpawnPosture::Discovery))};
            // Complexité : O(a + n) — comptages SQL, aucune tâche chargée (n ≤ 4096).
            let (active, instance_active, mut retained) = st.delegation_store.admission_counts(&instance)?;
            if active >= 128 || instance_active >= 16 {
                return Err("task_limit".into());
            }
            if retained >= crate::delegation::MAX_RETAINED_TASKS {
                // Borne de rétention atteinte : purge de borne (accusés recopiés d'abord,
                // arbres soldés du plus ancien au plus récent), puis seulement refus.
                if let Err(error) = copy_delivery_acks(st) {
                    warn!("rétention native: accusés non recopiés avant la purge de borne : {error}");
                }
                purge_retained(st, unix_timestamp(), Some(crate::delegation::MAX_RETAINED_TASKS), usize::MAX);
                retained = st.delegation_store.admission_counts(&instance)?.2;
                if retained >= crate::delegation::MAX_RETAINED_TASKS {
                    return Err("retention_saturated".into());
                }
            }
            let task_id = Uuid::new_v4().to_string();
            let executions = st
                .execution_store
                .recoverable_execution_ids_for_agent(&owner)
                .map_err(|error| error.to_string())?;
            if executions.len() > 1 {
                return Err("parent_execution_ambiguous".into());
            }
            let (root_owner_agent_id,parent_task_id)=match st.delegation_store.for_child(&owner)? {
                Some(mut parent)=>{
                    if st.managed_by_instance.get(&instance)!=Some(&parent.task_id)
                        || st.fleet.resolved_definition_for_command(&parent.task_id)
                            .is_none_or(|d|d.digest!=parent.definition.digest)
                        || !recover_child(st,&mut parent,&instance)? {
                        return Err("parent_task_unavailable".into());
                    }
                    let link=st.fleet.agent_link_for_child(&instance).map_err(|e|e.to_string())?.ok_or("parent_task_unavailable")?;
                    if link.delegation_id.as_deref()!=Some(parent.task_id.as_str()) {return Err("parent_task_unavailable".into())}
                    // Un délai explicite d'enfant ne dépasse pas le temps restant du parent.
                    // Sans délai explicite, comportement 149 : aucun contrôle.
                    if let (Some(secs), Some(deadline)) = (reply_timeout, parent.mission_deadline_at)
                        && i64::try_from(*secs).map_or(true, |secs| secs > deadline.saturating_sub(unix_timestamp()))
                    {
                        return Err("reply_timeout_exceeds_parent".into());
                    }
                    st.delegation_store.parent_lineage(&parent)?
                },
                None=>(owner.clone(),None),
            };
            let record = Task {
                task_id: task_id.clone(),
                root_owner_agent_id,
                parent_task_id,
                updated_at: unix_timestamp(),
                started_at: None,
                completed_at: None,
                owner,
                owner_instance: instance.clone(),
                origin_owner_instance: instance,
                parent_execution_id: executions.into_iter().next(),
                request: request.clone(),
                permission_snapshot,
                effective_posture,
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
                reply_timeout: reply_timeout.and_then(|secs| i64::try_from(secs).ok()),
                delivery_acked: false,
                acknowledged_at: None,
            };
            st.delegation_store
                .insert(&record, request_id, &canonical)?;
            current_task_view(st, &record, true)
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
            current_task_view(st, &task, true)
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
            current_task_view(st, &task, true)
        }
        NativeDelegationRequest::Ack { task_ids } => {
            validate_task_ids(task_ids)?;
            let mut tasks = Vec::with_capacity(task_ids.len());
            for id in task_ids {
                let task = st.delegation_store.get(id)?.ok_or("task_unavailable")?;
                if !can_own_task(st, &task, &owner, &instance)? {
                    return Err("task_unavailable".into());
                }
                tasks.push(task);
            }
            // Toute la propriété est établie avant de révéler un état de préparation.
            if tasks
                .iter()
                .any(|task| !matches!(task.state.as_str(), "result_available" | "failed"))
            {
                return Err("task_not_ready".into());
            }
            let (acknowledged, already) = st
                .delegation_store
                .acknowledge_tasks(task_ids, unix_timestamp())?;
            Ok(json!({"version":1,"acknowledged":acknowledged,"already":already}))
        }
        NativeDelegationRequest::Wait { .. } => Err("wait_unsupported".into()),
        NativeDelegationRequest::Grant { .. } => unreachable!(),
    }
}

fn spawn_task(st: &mut DaemonState, task: &mut Task) -> Result<(), String> {
    if st.delegation_store.revoked_agent(&task.owner)?||st.delegation_store.revoked_agent(&task.root_owner_agent_id)?||st.delegation_store.revoked(&task.owner_instance)?{return Err("permission_not_inherited".into())}
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
    if let Some(snapshot)=&task.permission_snapshot {
        for context in crate::native_permissions::snapshot_contexts(snapshot){bridget_transport::protocol::recheck_permission_context_sources(&context)?;}
        if let Some(context)=snapshot.child_policy.get("launch_context"){
            crate::native_permissions::recheck_context(context,registry.get(agent_type)?,Path::new(&task.cwd),&st.source_env)?;
        }
    }
    let order = FleetSpawnOrder {
        // La définition héritée est déjà figée. Une nouvelle conversion de
        // posture réduirait silencieusement sa politique fournisseur.
        posture: if task.permission_snapshot.is_some(){None}else{*posture},
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
            if let Some(snapshot)=&task.permission_snapshot {
                prepared.env.insert(crate::native_permissions::CHILD_POLICY_ENV.into(),serde_json::to_string(&snapshot.child_policy).map_err(|e|e.to_string())?.into());
                prepared.env.insert(crate::native_permissions::CHILD_SOURCES_ENV.into(),serde_json::to_string(&crate::native_permissions::snapshot_contexts(snapshot)).map_err(|e|e.to_string())?.into());
            }
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

// Cette capacité ne provient d'aucune trame. Seule la saga la construit pour
// relayer un résultat qu'elle a déjà capturé sous l'identité vivante de l'enfant.
pub(super) struct RetainedResultAuthority {
    task_id: String,
}

pub(super) fn authorize_retained_result(
    st: &DaemonState,
    authority: &RetainedResultAuthority,
    conn: &str,
    message: &bridget_core::BridgetMessage,
) -> Result<(), String> {
    let invalid = || "native_result_invalid".to_string();
    let task = st.delegation_store.get(&authority.task_id)?.ok_or_else(invalid)?;
    let child_instance = task.child_instance.as_ref().ok_or_else(invalid)?;
    if task.state != "result_available" || task.result.as_deref() != Some(message.body.as_str())
        || message.body.trim().is_empty() || message.body.len() > 262144
        || message.id != format!("native-result-{}", task.task_id)
        || message.from != task.child || message.to != task.owner
        || message.in_reply_to.as_deref() != Some(task.mission.as_str())
        || message.reply || message.intent.is_some()
        || conn != format!("native-delegation:{}", message.id)
        || st.connections.contains_key(conn) || !st.auxiliary_connections.contains(conn)
        || st.conn_names.get(conn) != Some(&task.child)
        || st.conn_instances.get(conn) != Some(child_instance)
        || !st.client_negotiations.get(conn).is_some_and(|client|
            client.issuer_scope == crate::communication::issuer_scope(&task.task_id))
    {
        return Err(invalid());
    }
    if st.delegation_store.revoked_agent(&task.owner)?
        || st.delegation_store.revoked_agent(&task.root_owner_agent_id)?
        || st.delegation_store.revoked(&task.owner_instance)? {
        return Err("native_result_authority_revoked".into());
    }
    let owner_route = st.router.get_agent(&task.owner).ok_or("native_result_owner_offline")?;
    if st.communication_projects.get(&owner_route.connection_id).is_some_and(|fact| fact.2) {
        return Err("native_result_authority_revoked".into());
    }
    if live_connection_identity(st, &owner_route.connection_id)
        != Some((task.owner.clone(), task.owner_instance.clone()))
        || !can_own_task(st, &task, &task.owner, &task.owner_instance)? {
        return Err("native_result_owner_unavailable".into());
    }
    if st.presences.get(&task.owner_instance).is_some_and(|p| p.transport == "t3code") {
        let thread = st.t3_thread_bindings.get(&owner_route.connection_id)
            .and_then(Option::as_deref).ok_or("native_result_owner_unavailable")?;
        human_authority_guard(st, thread, &agent_communication_project(st, &task.owner))
            .map_err(|_| "native_result_owner_unavailable".to_string())?;
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
    send_as_with_result_authority(st, owner, instance, task_scope, message, issued_at, controls, None)
}

fn send_as_with_result_authority(
    st: &mut DaemonState,
    owner: &str,
    instance: &str,
    task_scope: &str,
    message: bridget_core::BridgetMessage,
    issued_at: i64,
    controls: &mut Vec<DeferredControl>,
    authority: Option<&RetainedResultAuthority>,
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
    let response = handle_idempotent_send_with_result_authority(
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
        authority,
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

/// Autorités de bootstrap natif issues de l'admission propriétaire. Le
/// marqueur fournit la naissance PID ; la saga et les deux liens de flotte
/// prouvent qu'il s'agit du wrapper natif de CETTE mission, pas d'un nom RPC.
pub(super) fn restart_identities(
    st: &DaemonState,
) -> Result<std::collections::BTreeMap<String, ManagedIdentity>, String> {
    let desired = st.fleet.desired_fleet().map_err(|error| error.to_string())?;
    let mut identities = std::collections::BTreeMap::new();
    for task in st.delegation_store.tasks()? {
        match st.marker_store.load(&task.child) {
            Ok(_) => {}
            Err(crate::managed_process::ManagedProcessError::Io(error))
                if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
        }
        let NativeDelegationRequest::Delegate { agent_type, .. } = &task.request else {
            return Err("native_restart_identity_mismatch".into());
        };
        let instance = task.child_instance.as_deref().ok_or("native_restart_identity_mismatch")?;
        let entry = desired.equipiers.get(&task.child).ok_or("native_restart_identity_mismatch")?;
        let persisted_link = entry.agent_link.as_ref().ok_or("native_restart_identity_mismatch")?;
        let link = st.fleet.agent_link_for_child(instance).map_err(|error| error.to_string())?
            .ok_or("native_restart_identity_mismatch")?;
        if entry.command_id != task.task_id
            || &entry.agent_type != agent_type
            || entry.cwd != Path::new(&task.cwd)
            || entry.resolved_definition.as_ref() != Some(&task.definition)
            || link.child_instance_id != instance
            || (link.parent_instance_id != task.origin_owner_instance
                && link.parent_instance_id != task.owner_instance)
            || link.delegation_id.as_deref() != Some(task.task_id.as_str())
            || link.role != "native_delegate"
            || persisted_link.link_id != link.link_id
            || (persisted_link.parent_instance_id != task.origin_owner_instance
                && persisted_link.parent_instance_id != task.owner_instance)
            || persisted_link.delegation_id != link.delegation_id
            || persisted_link.role != link.role
        {
            return Err("native_restart_identity_mismatch".into());
        }
        identities.insert(task.child, ManagedIdentity {
            instance_id: instance.into(), command_id: task.task_id, generation: entry.generation,
        });
    }
    Ok(identities)
}

/// Une tâche imbriquée queued n'a pas encore de fournisseur. Son propriétaire
/// est pourtant perdu définitivement si la filiation désigne une instance
/// native déjà lancée : cette instance est exclue de toute reprise.
fn queued_owner_lost(st: &DaemonState, task: &Task) -> Result<bool, String> {
    let Some(parent_id) = task.parent_task_id.as_deref() else { return Ok(false) };
    let parent = st.delegation_store.get(parent_id)?.ok_or("parent_task_unavailable")?;
    let instance = parent.child_instance.as_deref().ok_or("native_owner_identity_mismatch")?;
    let link = st.fleet.agent_link_for_child(instance).map_err(|error| error.to_string())?
        .ok_or("native_owner_identity_mismatch")?;
    if parent.child != task.owner
        || instance != task.owner_instance
        || instance != task.origin_owner_instance
        || parent.root_owner() != task.root_owner()
        || link.child_instance_id != instance
        || (link.parent_instance_id != parent.origin_owner_instance
            && link.parent_instance_id != parent.owner_instance)
        || link.delegation_id.as_deref() != Some(parent.task_id.as_str())
        || link.role != "native_delegate"
    {
        return Err("native_owner_identity_mismatch".into());
    }
    Ok(true)
}

/// Fermer seulement la ligne de mission exacte, par comparaison atomique.
/// Une exécution déjà terminale (notamment completed) n'est jamais rouverte.
fn close_lost_execution(st: &DaemonState, task: &Task, reason: &str) -> Result<(), String> {
    let execution_id = format!("execution-{}", task.mission);
    let Some(target) = st.execution_store.execution_control_target(&execution_id)
        .map_err(|error| error.to_string())? else { return Ok(()) };
    if target.target_agent != task.child {
        return Err("native_execution_mismatch".into());
    }
    let snapshot = target.snapshot;
    if matches!(snapshot.state.as_str(), "queued" | "starting" | "running"
        | "waiting_approval" | "waiting_user_input" | "interrupting") {
        match st.execution_store.transition_if_current(&execution_id, &snapshot.state,
            snapshot.revision, snapshot.generation, "unreachable", reason, unix_timestamp())
            .map_err(|error| error.to_string())? {
            ConditionalTransition::Applied(_) => {}
            _ => return Err("native_execution_changed".into()),
        }
    }
    Ok(())
}

/// La flotte ordinaire peut reprendre ses agents persistants. Un enfant natif
/// appartient à sa saga : rejouer un tour engagé après une coupure est interdit.
pub(super) fn prepare_restart(st: &DaemonState) -> Result<HashSet<String>, String> {
    let mut children = HashSet::new();
    let running: HashSet<String> = st.fleet.desired_fleet()
        .map_err(|error| error.to_string())?.equipiers.into_iter()
        .filter(|(_, entry)| entry.lifecycle_state == DesiredLifecycleState::Running)
        .map(|(name, _)| name).collect();
    for mut task in st.delegation_store.tasks()? {
        children.insert(task.child.clone());
        if matches!(task.state.as_str(), "starting" | "mission_pending" | "working")
            || (task.state == "queued" && queued_owner_lost(st, &task)?)
        {
            task.state = "failed".into();
            task.error = Some("unreachable".into());
        }
        if task.state == "failed" && task.error.as_deref() == Some("unreachable") {
            close_lost_execution(st, &task, "daemon_restart")?;
            st.delegation_store.save(&task)?;
        }
        if matches!(task.state.as_str(), "cancelled" | "cancelling") {
            // La réconciliation des groupes est achevée avant cette fonction.
            // Cette instance native ne peut donc plus reprendre sa mission.
            close_lost_execution(st, &task, "native_cancelled_restart")?;
        }
        // Les résultats déjà capturés et l'attente des descendants restent
        // durables. Aucun nouveau fournisseur ne doit porter la même mission.
        if running.contains(&task.child) {
            st.fleet.mark_stopped(&task.child).map_err(|error| error.to_string())?;
        }
    }
    Ok(children)
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
                    Some(admitted_at.saturating_add(task.reply_timeout.unwrap_or(MISSION_REPLY_LIMIT_SECS)));
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
            // L'absence temporaire du destinataire ne maintient pas une
            // attente de descendants déjà terminés. Le résultat reste durable.
            if task.state == "waiting_for_children" {
                let busy = match descendants_busy(&st, &task) {
                    Ok(busy) => busy,
                    Err(error) => {
                        task.state = "failed".into();
                        task.error = Some(format!("native_descendants_unavailable:{error}")
                            .chars().take(1024).collect());
                        let _ = st.delegation_store.save(&task);
                        continue;
                    }
                };
                let active = tasks.iter().any(|child| {
                    Some(&child.owner_instance) == task.child_instance.as_ref() && !child.terminal()
                }) || busy;
                if !active {
                    if task.result.as_ref().is_none_or(|body| body.trim().is_empty() || body.len() > 262144)
                        || task.child_instance.is_none() {
                        task.state = "failed".into();
                        task.error = Some("native_result_invalid".into());
                    } else {
                        task.state = "result_available".into();
                    }
                    if let Err(error) = st.delegation_store.save(&task) {
                        error!("résultat natif non publié: {error}");
                        continue;
                    }
                }
            }
            if !parent_live {
                if task.state == "result_available" && !task.result_sent {
                    task.error = Some("native_result_owner_offline".into());
                    let _ = st.delegation_store.save(&task);
                }
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
                message.reply_timeout = u64::try_from(task.reply_timeout.unwrap_or(MISSION_REPLY_LIMIT_SECS)).ok();
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
                            .get_or_insert(admitted_at.saturating_add(task.reply_timeout.unwrap_or(MISSION_REPLY_LIMIT_SECS)));
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
            if task.state == "working" {
                align_request_deadline(&st, &task);
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
            if task.state == "result_available" && !task.result_sent {
                let Some(child_instance) = task.child_instance.as_deref() else {
                    task.state = "failed".into();
                    task.error = Some("native_result_invalid".into());
                    let _ = st.delegation_store.save(&task);
                    continue;
                };
                let mut response = bridget_core::BridgetMessage::new(
                    &task.child,
                    &task.owner,
                    task.result.as_deref().unwrap_or(""),
                );
                response.id = format!("native-result-{}", task.task_id);
                response.in_reply_to = Some(task.mission.clone());
                // Même un replay vérifie la tâche durable et son destinataire
                // actuel avant de consulter la clé idempotente existante.
                let authority = RetainedResultAuthority { task_id: task.task_id.clone() };
                let result = send_as_with_result_authority(
                    &mut st,
                    &task.child,
                    child_instance,
                    &task.task_id,
                    response,
                    task.created_at,
                    &mut controls,
                    Some(&authority),
                );
                if matches!(
                        &result,
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::Accepted { .. }
                                | IdempotencyIssue::OutcomeUnknown { delivery_id: Some(_), .. },
                            ..
                        }
                    )
                {
                    task.result_sent = true;
                    task.error = None;
                    if st.delegation_store.save(&task).is_err() {
                        task.result_sent = false;
                    }
                } else {
                    match result {
                        DaemonToWrapper::Nack { reason, .. } => {
                            if matches!(reason.as_str(), "native_result_invalid" | "native_result_authority_revoked") {
                                task.state = "failed".into();
                            }
                            task.error = Some(reason.chars().take(1024).collect());
                        }
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::Rejected { category, reason, .. }, ..
                        } => {
                            task.state = "failed".into();
                            task.error = Some(format!("{category}:{reason}").chars().take(1024).collect());
                        }
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::EnvelopeMismatch, ..
                        } => {
                            task.state = "failed".into();
                            task.error = Some("native_result_invalid".into());
                        }
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::Orphaned { reason, .. }, ..
                        } => {
                            task.state = "failed".into();
                            task.error = Some(reason.chars().take(1024).collect());
                        }
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::IdempotencyExpired
                                | IdempotencyIssue::InvalidIssuedAt, ..
                        } => {
                            task.state = "failed".into();
                            task.error = Some("native_result_delivery_expired".into());
                        }
                        _ => { task.error = Some("native_result_delivery_pending".into()); }
                    }
                    let _ = st.delegation_store.save(&task);
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
    if st.delegation_store.descendants(&task.task_id)?.iter().any(|child|!child.terminal()) {
        return Ok(true);
    }
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
                let executions = st.execution_store
                    .recoverable_execution_ids_for_agent(&name)
                    .map_err(|error| error.to_string())?;
                let mut active_execution = false;
                for execution_id in executions {
                    // Une ancienne interruption de contrôle peut garder une
                    // ligne de reprise après la clôture exacte au redémarrage.
                    // Elle ne ressuscite pas une mission native perdue.
                    let closed_native = if let Some(mission) = execution_id.strip_prefix("execution-") {
                        st.delegation_store.for_mission(mission)?.is_some_and(|child|
                            child.child == name
                                && child.child_instance.as_deref() == Some(link.child_instance_id.as_str())
                                && (child.state == "cancelled"
                                    || (child.state == "failed"
                                        && child.error.as_deref() == Some("unreachable"))))
                            && st.execution_store.execution_snapshot(&execution_id)
                                .map_err(|error| error.to_string())?
                                .is_some_and(|snapshot| matches!(snapshot.state.as_str(),
                                    "failed" | "unreachable" | "interrupted"))
                    } else { false };
                    if !closed_native {
                        active_execution = true;
                        break;
                    }
                }
                if active_execution || st
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
        let Ok(tasks) = st.delegation_store.descendants(&task.task_id) else {
            return;
        };
        let mut instances = vec![task.child_instance.clone()];
        for mut child in tasks {
            names.push(child.child.clone());
            instances.push(child.child_instance.clone());
            if !child.terminal() {
                child.state="cancelling".into();
                let _=st.delegation_store.save(&child);
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
                if names.contains(&entry.child)
                    && matches!(entry.state.as_str(), "cancelling" | "cancelled")
                {
                    // Tous les arrêts ci-dessus sont confirmés. Ne publier la
                    // fin de l'annulation qu'après la clôture CAS de sa mission.
                    if let Err(error) = close_lost_execution(&st, &entry, "native_cancelled") {
                        error!("annulation native: clôture d'exécution refusée: {error}");
                        continue;
                    }
                    if entry.state == "cancelling" {
                        entry.state = "cancelled".into();
                        if let Err(error) = st.delegation_store.save(&entry) {
                            error!("annulation native non persistée: {error}");
                            continue;
                        }
                    }
                }
                if names.contains(&entry.child) && entry.terminal() {
                    entry.cleanup_done = true;
                    let _ = st.delegation_store.save(&entry);
                }
            }
        }
    }
}

/// Délai explicite seulement : le suivi durable de la demande suit l'échéance de la
/// mission (`mission_deadline_at`). L'enveloppe de l'envoi (octets) ne change pas.
/// Appelée à chaque tick d'une mission `working` : un échec d'écriture est donc réessayé
/// au tick suivant, tant que l'échéance du suivi diffère. Une demande close n'est pas rouverte.
/// Complexité : O(1) — une lecture indexée par mission, une écriture seulement si l'échéance diffère.
fn align_request_deadline(st: &DaemonState, task: &Task) {
    let (Some(_), Some(deadline)) = (task.reply_timeout, task.mission_deadline_at) else {
        return;
    };
    match st.store.get_request(&task.mission) {
        Ok(Some(tracked)) if tracked.state == "open" && tracked.deadline_at != deadline => {
            if let Err(error) = st.store.set_request_deadline(&task.mission, deadline) {
                warn!("suivi de mission {} non aligné sur son échéance, nouvel essai au prochain tick : {error}", task.mission);
            }
        }
        Ok(_) => {}
        Err(error) => warn!("suivi de mission {} illisible, nouvel essai au prochain tick : {error}", task.mission),
    }
}

/// Purge les arbres de tâches soldés, sous le verrou du daemon (R2, R6, R10).
/// Le contrôle de vivacité est refait ici, au moment de la suppression : un
/// sous-agent connecté, en cours de lancement ou désiré `Running` conserve son
/// arbre. Avant la suppression, chaque enfant est décommissionné pour qu'une
/// entrée `Stopped` ne puisse plus être relancée. Un échec de décommission
/// conserve l'arbre. `bound` : voir `DelegationStore::purge_trees`.
/// `max_trees` : plafond d'arbres supprimés par appel. L'entretien le fixe à 128 par
/// passage (âge et borne cumulés) ; l'admission passe `usize::MAX`, sans limite.
/// Complexité : O(S + F + P + M + n log n) — S sagas, F entrées de flotte, P présences,
/// M lancements gérés, n ≤ 4096 tâches.
pub(super) fn purge_retained(st: &DaemonState, now: i64, bound: Option<usize>, max_trees: usize) -> usize {
    let (launching, desired) = match (
        st.idempotency.spawn_commands(&st.fleet.supervisor_scope()),
        st.fleet.desired_fleet(),
    ) {
        (Ok(sagas), Ok(desired)) => (sagas, desired),
        (Err(error), _) => {
            warn!("rétention native: sagas de lancement illisibles, aucune purge : {error}");
            return 0;
        }
        (_, Err(error)) => {
            warn!("rétention native: flotte illisible, aucune purge : {error}");
            return 0;
        }
    };
    // Complexité : O(1) par enfant grâce à ces ensembles construits une fois.
    let present: HashSet<&str> = st
        .presences
        .values()
        .filter(|presence| matches!(presence.state.as_str(), "connected" | "busy"))
        .map(|presence| presence.name.as_str())
        .collect();
    let spawning: HashSet<&str> = st.managed_spawns.values().map(|spawn| spawn.lease.name.as_str()).collect();
    let launching: HashSet<&str> = launching
        .iter()
        .filter(|saga| {
            matches!(
                saga.state,
                crate::idempotency::SpawnCommandState::Requested
                    | crate::idempotency::SpawnCommandState::Reserved
                    | crate::idempotency::SpawnCommandState::Starting
            )
        })
        .map(|saga| saga.name.as_str())
        .collect();
    let may_run = |child: &str| {
        st.router.get_agent(child).is_some()
            || present.contains(child)
            || st.decommissioning_names.contains(child)
            || spawning.contains(child)
            || launching.contains(child)
            || desired
                .equipiers
                .get(child)
                .is_some_and(|entry| entry.lifecycle_state == DesiredLifecycleState::Running)
    };
    let purged = st.delegation_store.purge_trees(now, bound, max_trees, |tree| {
        if tree.iter().any(|task| may_run(&task.child)) {
            return false;
        }
        let children: Vec<&str> = tree.iter().map(|task| task.child.as_str()).collect();
        match st.fleet.decommission_all(&children) {
            Ok(_) => true,
            Err(error) => {
                warn!("rétention native: décommission de l'arbre {} refusée, arbre conservé : {error}", tree[0].task_id);
                false
            }
        }
    });
    match purged {
        Ok(count) => count,
        Err(error) => {
            warn!("rétention native: purge interrompue : {error}");
            0
        }
    }
}

/// Recopie l'accusé des remises natives dans la tâche (`delivery_acked`), preuve
/// durable que `purge_expired_sends` ne peut plus effacer (R2.2). Lit la phase de
/// `native-result-<task>` ou `native-failure-<task>` sous `issuer_scope(task_id)`.
/// Sans lot : le total de tâches est borné à 4096. Une remise indéterminée ou
/// effacée ne produit aucune preuve : l'arbre reste conservé. Rend le nombre de
/// preuves recopiées, ou une erreur dès qu'une lecture ou une écriture échoue :
/// l'appelant ne doit alors pas effacer les remises.
/// Complexité : O(n log n) — n ≤ 4096 tâches, une lecture indexée par tâche en attente.
pub(super) fn copy_delivery_acks(st: &DaemonState) -> Result<usize, String> {
    let mut copied = 0;
    for mut task in st.delegation_store.tasks()?.into_iter().filter(|task| !task.delivery_acked) {
        let prefix = match task.state.as_str() {
            "result_available" if task.result_sent => "native-result-",
            "failed" if task.failure_sent => "native-failure-",
            _ => continue,
        };
        let key = IdempotencyKey::new(
            &crate::communication::issuer_scope(&task.task_id),
            OperationKind::Send,
            &format!("{prefix}{}", task.task_id),
        )
        .map_err(|error| error.to_string())?;
        let phase = st.idempotency.send_delivery_phase(&key).map_err(|error| error.to_string())?;
        if phase.as_deref() != Some("acked") {
            continue;
        }
        task.delivery_acked = true;
        st.delegation_store.save(&task).map_err(|error| format!("accusé de {} non persisté : {error}", task.task_id))?;
        copied += 1;
    }
    Ok(copied)
}

/// Passage d'entretien de la rétention : arbres soldés de plus de 7 jours, puis
/// purge de borne, puis pierres tombales de plus de 30 jours. Au plus 128 arbres
/// supprimés par passage, âge et borne cumulés.
pub(super) fn maintain_retention(st: &DaemonState, now: i64) {
    let max_trees = crate::disk_hygiene::MAX_ENTRIES_PER_PASS;
    let mut trees = purge_retained(st, now, None, max_trees);
    // Purge de borne : une base au-dessus de 4096 tâches (écriture externe, ancien
    // volume) redescend par lots, sans attendre une admission. Le quota de 128 arbres
    // est partagé avec la purge d'âge : le second appel ne reçoit que le reste.
    let over = st.delegation_store.admission_counts("").map(|counts| counts.2 >= crate::delegation::MAX_RETAINED_TASKS);
    let reste = max_trees.saturating_sub(trees);
    if over.unwrap_or(false) && reste > 0 {
        trees += purge_retained(st, now, Some(crate::delegation::MAX_RETAINED_TASKS), reste);
    }
    if trees > 0 {
        info!("entretien: {trees} arbre(s) de délégation native purgé(s)");
    }
    if let Err(error) = st.delegation_store.purge_expired_tombstones(now) {
        warn!("entretien: pierres tombales de délégation : {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    const OWNER: &str = "89000000-0000-4000-8000-000000000102";

    fn tache_151(st: &mut DaemonState, key: &str) -> Task {
        let created = handle_locked("conn-1", request(key), st).unwrap();
        st.delegation_store
            .get(created["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap()
    }

    fn attendre_151(shared: &Arc<Mutex<DaemonState>>, ids: Vec<String>, timeout: u64) -> Value {
        let DaemonToWrapper::NativeDelegationResult { result } = handle(
            "conn-1",
            NativeDelegationRequest::Wait {
                task_ids: ids,
                wait_timeout: Some(timeout),
            },
            shared,
        ) else {
            panic!("réponse native")
        };
        result
    }

    #[test]
    fn native151_t027_accuse_reel_reveille_request_wait_deux_scopes() {
        let (mut st, config, _rx, _reader) = fixture("request-ack-scopes");
        let cases = [
            (
                "012_scope_aaaaaaaaaaaa",
                "demande-a",
                "enfant-a",
                "première réponse €",
                "remise-a",
            ),
            (
                "012_scope_bbbbbbbbbbbb",
                "demande-b",
                "enfant-b",
                "seconde réponse différente",
                "remise-b",
            ),
        ];
        let now = unix_timestamp();
        for &(scope, request, responder, body, delivery) in &cases {
            st.store
                .create_request(request, OWNER, responder, 60)
                .unwrap();
            let key = IdempotencyKey::new(scope, OperationKind::Send, "reponse-partagee").unwrap();
            st.idempotency
                .reserve(
                    &key,
                    body.as_bytes(),
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let mut message = bridget_core::BridgetMessage::new(responder, OWNER, body);
            message.id = "reponse-partagee".into();
            message.in_reply_to = Some(request.into());
            st.idempotency
                .begin_send_delivery(
                    &key,
                    &SendDelivery {
                        delivery_id: delivery.into(),
                        recipient_instance_id: "instance-1".into(),
                        delivery_generation: 1,
                        expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                        message_bytes: serde_json::to_vec(&message).unwrap(),
                    },
                )
                .unwrap();
        }
        let active = st.active_waits.clone();
        let epoch = st.delegation_store.wait_epoch();
        let shared = Arc::new(Mutex::new(st));
        for &(scope, request, responder, body, delivery) in &cases {
            let expected = json!({"id":"reponse-partagee","from":responder,"summary":body,"summary_truncated":false});
            std::thread::scope(|threads| {
                let worker = shared.clone();
                let (ready, result) = mpsc::sync_channel(1);
                threads.spawn(move || {
                    ready
                        .send(handle_wrapper_message(
                            "conn-1",
                            WrapperToDaemon::RequestWait {
                                request_id: request.into(),
                                wait_timeout: 10,
                            },
                            &worker,
                        ))
                        .unwrap();
                });
                // Le compteur est acquis après le premier snapshot : l'attente a lu open.
                let opened = Instant::now();
                while active.load(Ordering::Acquire) != 1
                    && opened.elapsed() < Duration::from_secs(2)
                {
                    std::thread::sleep(Duration::from_millis(1));
                }
                assert_eq!(
                    active.load(Ordering::Acquire),
                    1,
                    "RequestWait non ouvert : {scope}"
                );
                assert!(
                    matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)),
                    "réponse avant ACK : {scope}"
                );
                assert_eq!(
                    shared
                        .lock()
                        .unwrap()
                        .store
                        .request_wait_snapshot(request, OWNER)
                        .unwrap()
                        .unwrap()
                        .0,
                    "open"
                );
                let before = epoch.get();
                // Ancrage conservateur : avant même le traitement de l'accusé normal.
                let ack_started = Instant::now();
                assert!(
                    handle_wrapper_message(
                        "conn-1",
                        WrapperToDaemon::DeliverAcked {
                            delivery_id: delivery.into(),
                            delivery_generation: 1,
                        },
                        &shared
                    )
                    .is_none()
                );
                let answer = result
                    .recv_timeout(Duration::from_secs(2))
                    .expect("RequestWait non réveillé après ACK réel")
                    .expect("réponse RequestWait");
                let elapsed = ack_started.elapsed();
                assert!(elapsed <= Duration::from_secs(2), "{scope} : {elapsed:?}");
                assert!(
                    matches!(&answer, DaemonToWrapper::RequestWaitResult {
                        state, timed_out_wait:false, reply:Some(reply), reply_reason:None, ..
                    } if state == "answered" && reply == &expected),
                    "{scope} : {answer:?}"
                );
                assert_eq!(
                    epoch.get(),
                    before + 1,
                    "seul le vrai handler émet le réveil"
                );
                assert_eq!(active.load(Ordering::Acquire), 0, "capacité non libérée");
                eprintln!(
                    "T027 SC009 : scope={scope}, request={request}, ACK réel -> answered en {} µs, résumé exact={expected}",
                    elapsed.as_micros()
                );
            });
        }
        let state = shared.lock().unwrap();
        let ledger_body: String = state
            .store
            .connection()
            .query_row(
                "SELECT body FROM ledger WHERE id='reponse-partagee' AND target=?1",
                [OWNER],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            ledger_body, cases[1].3,
            "le ledger partage la même clé de réponse"
        );
        for &(_, request, responder, body, _) in &cases {
            let (state_name, _, summary) = state
                .store
                .request_wait_snapshot(request, OWNER)
                .unwrap()
                .unwrap();
            assert_eq!(state_name, "answered");
            assert_eq!(
                serde_json::from_str::<Value>(&summary.unwrap()).unwrap(),
                json!({"id":"reponse-partagee","from":responder,"summary":body,"summary_truncated":false}),
                "le résumé reste propre à la demande après le second scope"
            );
        }
        drop(state);
        cleanup(&config);
    }

    #[test]
    fn native151_envoi_etats_resume_et_lecture_pure() {
        let (mut st, config, _rx, _reader) = fixture("request-states");
        for id in ["answered", "guichet", "expired", "open", "cancelled"] {
            st.store.create_request(id, OWNER, "target", 60).unwrap();
        }
        let reply = bridget_core::BridgetMessage::new("target", OWNER, "€".repeat(500));
        let frozen = crate::store::request_reply_summary(&reply);
        st.store
            .mark_answered("answered", "target", OWNER, Some(&frozen))
            .unwrap();
        st.store
            .mark_answered("guichet", "target", OWNER, None)
            .unwrap();
        st.store
            .set_request_deadline("expired", unix_timestamp() - 1)
            .unwrap();
        st.store.cancel_request("cancelled", OWNER, None).unwrap();
        let shared = Arc::new(Mutex::new(st));
        for (id, expected) in [
            ("answered", "answered"),
            ("guichet", "answered"),
            ("expired", "timed_out"),
            ("cancelled", "cancelled"),
            ("open", "open"),
        ] {
            let before = shared
                .lock()
                .unwrap()
                .store
                .request_wait_snapshot(id, OWNER)
                .unwrap();
            let answer = wait_request("conn-1", id, 1, &shared, None, Duration::from_secs(60));
            let DaemonToWrapper::RequestWaitResult {
                state,
                timed_out_wait,
                reply,
                reply_reason,
                ..
            } = answer
            else {
                panic!("réponse attendue : {answer:?}")
            };
            assert_eq!(state, expected);
            assert_eq!(timed_out_wait, id == "open");
            if id == "answered" {
                assert_eq!(reply, Some(serde_json::from_str(&frozen).unwrap()));
            } else {
                assert_eq!(reply, None);
            }
            assert_eq!(
                reply_reason.as_deref(),
                (id == "guichet").then_some("reply_unavailable")
            );
            assert_eq!(
                before,
                shared
                    .lock()
                    .unwrap()
                    .store
                    .request_wait_snapshot(id, OWNER)
                    .unwrap()
            );
        }
        cleanup(&config);
    }

    #[test]
    fn native151_envoi_refus_et_rappel_apres_coupure() {
        let (st, config, _rx, _reader) = fixture("request-refuse");
        st.store
            .create_request("foreign", "other", "target", 60)
            .unwrap();
        st.store.create_request("own", OWNER, "target", 60).unwrap();
        let shared = Arc::new(Mutex::new(st));
        for id in ["foreign", "missing"] {
            let DaemonToWrapper::NativeDelegationResult { result } =
                wait_request("conn-1", id, 1, &shared, None, Duration::from_secs(5))
            else {
                panic!("refus")
            };
            assert_eq!(result["code"], "request_unavailable");
        }
        for timeout in [0, 3601] {
            let DaemonToWrapper::NativeDelegationResult { result } = wait_request(
                "conn-1",
                "own",
                timeout,
                &shared,
                None,
                Duration::from_secs(5),
            ) else {
                panic!("refus")
            };
            assert_eq!(result["code"], "wait_timeout_out_of_range");
        }
        let (server, client) = UnixStream::pair().unwrap();
        drop(client);
        let _ = wait_request(
            "conn-1",
            "own",
            2,
            &shared,
            Some(&server),
            Duration::from_secs(5),
        );
        shared
            .lock()
            .unwrap()
            .store
            .mark_answered("own", "target", OWNER, None)
            .unwrap();
        assert!(
            matches!(wait_request("conn-1","own",1,&shared,None,Duration::from_secs(5)),DaemonToWrapper::RequestWaitResult{state,..} if state=="answered")
        );
        cleanup(&config);
    }

    #[test]
    fn native151_envoi_annulation_reveil_sans_filet() {
        let (st, config, _rx, _reader) = fixture("request-cancel");
        st.store.create_request("own", OWNER, "target", 60).unwrap();
        let shared = Arc::new(Mutex::new(st));
        let worker = shared.clone();
        let start = Instant::now();
        let join = std::thread::spawn(move || {
            wait_request("conn-1", "own", 3, &worker, None, Duration::from_secs(60))
        });
        let active = shared.lock().unwrap().active_waits.clone();
        let admission = Instant::now();
        while active.load(Ordering::Acquire) != 1 && admission.elapsed() < Duration::from_secs(2) {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(active.load(Ordering::Acquire), 1);
        assert!(!join.is_finished());
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .store
                .request_wait_snapshot("own", OWNER)
                .unwrap()
                .unwrap()
                .0,
            "open"
        );
        let cancelled = handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::CancelRequest {
                id: "own".into(),
                sender: OWNER.into(),
                reason: None,
            },
            &shared,
        );
        assert!(
            matches!(cancelled,Some(DaemonToWrapper::RequestCancelled{state,..}) if state=="cancelled")
        );
        assert!(
            matches!(join.join().unwrap(),DaemonToWrapper::RequestWaitResult{state,..} if state=="cancelled")
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        cleanup(&config);
    }

    #[test]
    fn native151_envoi_guichet_reveil_sans_filet() {
        use bridget_transport::protocol::{
            GuichetOutcome, GuichetReplyPayload, ServiceRequestOperation, ServiceRequestPayload,
        };
        let (mut st, config, _rx, _reader) = fixture("request-guichet");
        let now = unix_timestamp();
        st.store.create_request("own", OWNER, "target", 60).unwrap();
        let scope = "015_scope_0123456789abcdef0123456789abcdef";
        st.store
            .deposit_guichet(
                &GuichetDeposit {
                    issuer_scope: scope.into(),
                    request_id: "report".into(),
                    issued_at: now,
                    from: "target".into(),
                    operation: ServiceRequestOperation::DeliveryReport,
                    payload: ServiceRequestPayload::DeliveryReport {
                        objective_id: "objective".into(),
                        delegation_id: "delegation".into(),
                        delivery_hash: "0".repeat(64),
                        in_reply_to: "own".into(),
                        review_verdict: None,
                    },
                    canonical_bytes: b"report".to_vec(),
                    authorization_attestation: None,
                },
                600,
                60,
                now,
            )
            .unwrap();
        let GuichetNext::Claimed(claim) = st.store.claim_next_guichet("service", now).unwrap()
        else {
            panic!("claim")
        };
        let shared = Arc::new(Mutex::new(st));
        handle_wrapper_message(
            "service",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service,
            },
            &shared,
        );
        handle_wrapper_message(
            "service",
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                service: "guichet".into(),
                issuer_scope: scope.into(),
                capabilities: vec![ServiceCapability::GuichetV1],
            },
            &shared,
        );
        let worker = shared.clone();
        let start = Instant::now();
        let join = std::thread::spawn(move || {
            wait_request("conn-1", "own", 3, &worker, None, Duration::from_secs(60))
        });
        let active = shared.lock().unwrap().active_waits.clone();
        let admission = Instant::now();
        while active.load(Ordering::Acquire) != 1 && admission.elapsed() < Duration::from_secs(2) {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(active.load(Ordering::Acquire), 1);
        assert!(!join.is_finished());
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .store
                .request_wait_snapshot("own", OWNER)
                .unwrap()
                .unwrap()
                .0,
            "open"
        );
        let response = handle_wrapper_message(
            "service",
            WrapperToDaemon::GuichetReply {
                version: SERVICE_CONTRACT_VERSION,
                issuer_scope: claim.issuer_scope,
                request_id: claim.request_id,
                claim_generation: claim.claim_generation,
                claim_token: claim.claim_token,
                response_message_id: "reply-report".into(),
                in_reply_to: "own".into(),
                outcome: GuichetOutcome::Accepted,
                payload: GuichetReplyPayload::DeliveryReport {
                    objective_id: "objective".into(),
                    delegation_id: "delegation".into(),
                    delivery_hash: "0".repeat(64),
                    review_verdict: None,
                },
            },
            &shared,
        );
        assert!(
            matches!(response, Some(DaemonToWrapper::GuichetResult { .. })),
            "{response:?}"
        );
        assert!(
            matches!(join.join().unwrap(),DaemonToWrapper::RequestWaitResult{state,reply:None,reply_reason:Some(reason),..} if state=="answered" && reason=="reply_unavailable")
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        cleanup(&config);
    }

    #[test]
    fn native151_attente_immediate_et_lecture_pure() {
        let (mut st, config, _rx, _reader) = fixture("wait-ready");
        let mut task = tache_151(&mut st, "wait");
        task.state = "result_available".into();
        task.result = Some("€".repeat(1000));
        st.delegation_store.save(&task).unwrap();
        let shared = Arc::new(Mutex::new(st));
        for phase in 0..3 {
            if phase == 1 {
                let mut st = shared.lock().unwrap();
                remise_native_150(&mut st, &task, "native-result-", false);
                task.result_sent = true;
                st.delegation_store.save(&task).unwrap();
            } else if phase == 2 {
                let mut st = shared.lock().unwrap();
                st.idempotency
                    .mark_delivery_indeterminate(
                        &format!("delivery-native-result-{}", task.task_id),
                        "instance-1",
                        1,
                    )
                    .unwrap();
            }
            let before = serde_json::to_value(
                shared
                    .lock()
                    .unwrap()
                    .delegation_store
                    .get(&task.task_id)
                    .unwrap(),
            )
            .unwrap();
            let start = Instant::now();
            let answer = attendre_151(&shared, vec![task.task_id.clone()], 1);
            assert_eq!(answer["timed_out"], false, "{answer}");
            assert_eq!(answer["ready"][0]["task_id"], task.task_id);
            assert!(answer["ready"][0].get("result").is_none());
            assert_eq!(answer["pending"], json!([]));
            assert!(start.elapsed() < Duration::from_millis(500));
            assert_eq!(
                before,
                serde_json::to_value(
                    shared
                        .lock()
                        .unwrap()
                        .delegation_store
                        .get(&task.task_id)
                        .unwrap()
                )
                .unwrap()
            );
        }
        cleanup(&config);
    }

    #[test]
    fn native151_attente_expiration_une_seconde_et_pause() {
        let (mut st, config, _rx, _reader) = fixture("wait-timeout");
        let task = tache_151(&mut st, "wait");
        pauser_150(&st, true);
        let before = serde_json::to_value(&task).unwrap();
        let shared = Arc::new(Mutex::new(st));
        let start = Instant::now();
        let answer = attendre_151(&shared, vec![task.task_id.clone()], 1);
        assert_eq!(answer["timed_out"], true, "{answer}");
        assert!(start.elapsed() >= Duration::from_millis(950));
        assert!(start.elapsed() < Duration::from_millis(1500));
        assert_eq!(answer["pending"], json!([task.task_id]));
        assert_eq!(
            before,
            serde_json::to_value(
                shared
                    .lock()
                    .unwrap()
                    .delegation_store
                    .get(&task.task_id)
                    .unwrap()
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            shared.lock().unwrap().active_waits.load(Ordering::Acquire),
            0
        );
        cleanup(&config);
    }

    #[test]
    fn native151_sc008_task_wait_select_calme_cinq_secondes() {
        let (mut st, config, _rx, _reader) = fixture("wait-sql-calm");
        let mut task = tache_151(&mut st, "calm");
        task.state = "working".into();
        st.delegation_store.save(&task).unwrap();
        let before =
            serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap();
        let epoch = st.delegation_store.wait_epoch();
        let seen = epoch.get();
        let active = st.active_waits.clone();
        let shared = Arc::new(Mutex::new(st));
        let trace = native151_sql_trace_tests::SelectTrace::new(
            shared.lock().unwrap().delegation_store.test_connection(),
            b"SELECT payload FROM native_delegations WHERE task_id=?1",
        );
        let result = attendre_151(&shared, vec![task.task_id.clone()], 10);
        assert_eq!(result["timed_out"], true, "{result}");
        assert_eq!(result["waited_secs"], 10, "{result}");
        assert_eq!(result["pending"], json!([task.task_id]), "{result}");
        trace.assert_calm("TaskWait");
        drop(trace);
        assert_eq!(epoch.get(), seen);
        assert_eq!(active.load(Ordering::Acquire), 0);
        assert_eq!(
            serde_json::to_value(
                shared
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
        eprintln!(
            "SC008 TaskWait : epoch inchangé={seen}, capacité libérée, tâche working conservée"
        );
        cleanup(&config);
    }

    #[test]
    fn native151_attente_evenement_sans_filet_et_compteur_ecritures() {
        let (mut st, config, _rx, _reader) = fixture("wait-event");
        let mut task = tache_151(&mut st, "wait");
        let epoch = st.delegation_store.wait_epoch();
        let before = epoch.get();
        let projection = st.delegation_store.projection_meta().unwrap();
        task.result_sent = true;
        st.delegation_store.save(&task).unwrap();
        assert_eq!(epoch.get(), before + 1);
        assert_eq!(
            st.delegation_store.projection_meta().unwrap().seq,
            projection.seq
        );
        let shared = Arc::new(Mutex::new(st));
        let worker = shared.clone();
        let id = task.task_id.clone();
        let start = Instant::now();
        let join = std::thread::spawn(move || {
            wait_tasks(
                "conn-1",
                &[id],
                Some(3),
                None,
                &worker,
                None,
                Duration::from_secs(60),
            )
        });
        let active = shared.lock().unwrap().active_waits.clone();
        let admission = Instant::now();
        while active.load(Ordering::Acquire) != 1 && admission.elapsed() < Duration::from_secs(2) {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(active.load(Ordering::Acquire), 1);
        assert!(!join.is_finished());
        assert!(
            !shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .terminal()
        );
        task.state = "result_available".into();
        task.result = Some("terminé".into());
        shared.lock().unwrap().delegation_store.save(&task).unwrap();
        let answer = join.join().unwrap().unwrap();
        assert_eq!(answer["ready"][0]["task_id"], task.task_id);
        assert!(start.elapsed() < Duration::from_secs(2));
        cleanup(&config);
    }

    #[test]
    fn native151_attente_refus_atomiques_et_capacite() {
        let (mut st, config, _rx, _reader) = fixture("wait-refus");
        let task = tache_151(&mut st, "wait");
        let id = task.task_id.clone();
        let shared = Arc::new(Mutex::new(st));
        for ids in [vec![], vec![id.clone(); 17], vec![id.clone(), id.clone()]] {
            assert_eq!(attendre_151(&shared, ids, 1)["code"], "invalid_request");
        }
        for timeout in [0, 3601] {
            assert_eq!(
                attendre_151(&shared, vec![id.clone()], timeout)["code"],
                "wait_timeout_out_of_range"
            );
        }
        assert_eq!(
            attendre_151(&shared, vec![id.clone(), "unknown".into()], 1)["code"],
            "task_unavailable"
        );
        let active = shared.lock().unwrap().active_waits.clone();
        let permits: Vec<_> = (0..64)
            .map(|_| WaitPermit::acquire(&active).unwrap())
            .collect();
        assert_eq!(
            attendre_151(&shared, vec![id.clone()], 1)["code"],
            "wait_capacity"
        );
        assert_eq!(active.load(Ordering::Acquire), 64);
        drop(permits);
        assert_eq!(active.load(Ordering::Acquire), 0);
        let mut foreign = task;
        foreign.owner = "foreign".into();
        shared
            .lock()
            .unwrap()
            .delegation_store
            .save(&foreign)
            .unwrap();
        assert_eq!(
            attendre_151(&shared, vec![id], 1)["code"],
            "task_unavailable"
        );
        cleanup(&config);
    }

    #[test]
    fn native151_attente_client_ferme_libere_la_reservation() {
        let (mut st, config, _rx, _reader) = fixture("wait-close");
        let task = tache_151(&mut st, "wait");
        let shared = Arc::new(Mutex::new(st));
        let worker = shared.clone();
        let id = task.task_id.clone();
        let (server, client) = UnixStream::pair().unwrap();
        let start = Instant::now();
        let join = std::thread::spawn(move || {
            wait_tasks(
                "conn-1",
                &[id],
                Some(30),
                None,
                &worker,
                Some(&server),
                Duration::from_secs(5),
            )
        });
        std::thread::sleep(Duration::from_millis(80));
        drop(client);
        assert_eq!(join.join().unwrap().unwrap_err(), "daemon_unreachable");
        assert!(start.elapsed() < Duration::from_secs(6));
        assert_eq!(
            shared.lock().unwrap().active_waits.load(Ordering::Acquire),
            0
        );
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .state,
            "queued"
        );
        cleanup(&config);
    }

    #[test]
    fn native151_groupe_trois_resultats_une_reponse() {
        let (mut st, config, _rx, _reader) = fixture("wait-group");
        let mut ids = Vec::new();
        for key in ["one", "two", "three"] {
            let mut task = tache_151(&mut st, key);
            task.state = "result_available".into();
            task.result = Some(key.into());
            st.delegation_store.save(&task).unwrap();
            ids.push(task.task_id);
        }
        ids.reverse();
        let shared = Arc::new(Mutex::new(st));
        let answer = attendre_151(&shared, ids.clone(), 1);
        assert_eq!(answer["pending"], json!([]));
        let got: Vec<_> = answer["ready"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["task_id"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(got, ids);
        cleanup(&config);
    }

    #[test]
    fn native151_attente_course_publication_cent_repetitions() {
        let (mut st, config, _rx, _reader) = fixture("wait-races");
        let mut task = tache_151(&mut st, "wait");
        let shared = Arc::new(Mutex::new(st));
        let start = Instant::now();
        for _ in 0..100 {
            task.state = "working".into();
            task.result = None;
            shared.lock().unwrap().delegation_store.save(&task).unwrap();
            let worker = shared.clone();
            let id = task.task_id.clone();
            let join = std::thread::spawn(move || {
                wait_tasks(
                    "conn-1",
                    &[id],
                    Some(2),
                    None,
                    &worker,
                    None,
                    Duration::from_secs(60),
                )
            });
            task.state = "result_available".into();
            task.result = Some("publié".into());
            shared.lock().unwrap().delegation_store.save(&task).unwrap();
            assert_eq!(join.join().unwrap().unwrap()["timed_out"], false);
        }
        assert!(start.elapsed() < Duration::from_secs(10));
        cleanup(&config);
    }

    #[test]
    fn native151_facade_progression_64_attentes() {
        let (mut st, config, _rx, _reader) = fixture("wait-load");
        let task = tache_151(&mut st, "wait");
        let mut foreign = tache_151(&mut st, "foreign");
        let active = st.active_waits.clone();
        let shared = Arc::new(Mutex::new(st));
        let joins: Vec<_> = (0..64)
            .map(|_| {
                let worker = shared.clone();
                let id = task.task_id.clone();
                std::thread::spawn(move || attendre_151(&worker, vec![id], 10))
            })
            .collect();
        let admission = Instant::now();
        while active.load(Ordering::Acquire) < 64 && admission.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(active.load(Ordering::Acquire), 64);
        assert_eq!(
            attendre_151(&shared, vec![task.task_id.clone()], 1)["code"],
            "wait_capacity"
        );
        let writer = shared.clone();
        let writes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let progress = writes.clone();
        let (stop, stopped) = mpsc::channel();
        let (first_save, started) = mpsc::sync_channel(0);
        let noise = std::thread::spawn(move || {
            loop {
                match stopped.try_recv() {
                    Ok(()) | Err(mpsc::TryRecvError::Disconnected) => break,
                    Err(mpsc::TryRecvError::Empty) => {}
                }
                foreign.result_sent = !foreign.result_sent;
                writer
                    .lock()
                    .unwrap()
                    .delegation_store
                    .save(&foreign)
                    .unwrap();
                if progress.fetch_add(1, Ordering::AcqRel) == 0 && first_save.send(()).is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        });
        started.recv_timeout(Duration::from_secs(3)).unwrap();
        let await_progress = |before| {
            let deadline = Instant::now() + Duration::from_secs(2);
            while writes.load(Ordering::Acquire) <= before {
                assert!(
                    Instant::now() < deadline,
                    "écrivain étranger sans progression"
                );
                std::thread::sleep(Duration::from_millis(2));
            }
            writes.load(Ordering::Acquire)
        };
        let status_before = writes.load(Ordering::Acquire);
        assert!(!noise.is_finished());
        let start = Instant::now();
        let DaemonToWrapper::NativeDelegationResult { result } = handle(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: task.task_id.clone(),
            },
            &shared,
        ) else {
            panic!("status")
        };
        let status_elapsed = start.elapsed();
        assert_eq!(result["task_id"], task.task_id);
        assert!(status_elapsed < Duration::from_millis(500));
        assert!(!noise.is_finished());
        let status_after = await_progress(status_before);
        assert_eq!(active.load(Ordering::Acquire), 64);
        let cancel_before = writes.load(Ordering::Acquire);
        let start = Instant::now();
        let DaemonToWrapper::NativeDelegationResult { result } = handle(
            "conn-1",
            NativeDelegationRequest::Cancel {
                task_id: task.task_id.clone(),
            },
            &shared,
        ) else {
            panic!("cancel")
        };
        let cancel_elapsed = start.elapsed();
        assert_ne!(result["status"], "refused");
        assert!(cancel_elapsed < Duration::from_millis(500));
        assert!(!noise.is_finished());
        let cancel_after = await_progress(cancel_before);
        stop.send(()).unwrap();
        noise.join().unwrap();
        eprintln!(
            "SC008 charge : 64 attentes, Status={} µs, Cancel={} µs, saves Status {}->{}, Cancel {}->{}, total={}, stop/join après contrôles",
            status_elapsed.as_micros(),
            cancel_elapsed.as_micros(),
            status_before,
            status_after,
            cancel_before,
            cancel_after,
            writes.load(Ordering::Acquire)
        );
        // Finalisation synthétique : aucun modèle ni vrai enfant n'est lancé par la fixture.
        {
            let st = shared.lock().unwrap();
            let mut ended = st.delegation_store.get(&task.task_id).unwrap().unwrap();
            ended.state = "cancelled".into();
            st.delegation_store.save(&ended).unwrap();
        }
        for join in joins {
            assert_eq!(join.join().unwrap()["timed_out"], false);
        }
        assert_eq!(active.load(Ordering::Acquire), 0);
        cleanup(&config);
    }

    #[test]
    fn native151_boite_phases_absentes_orphelines_et_accusees() {
        let (mut st, config, _rx, _reader) = fixture("mail-phases");
        let mut task = tache_151(&mut st, "box");
        task.state = "failed".into();
        for (phase, sent, acked, expected) in [
            (None, false, false, "pending"),
            (None, true, false, "claimed"),
            (Some("dispatching"), false, false, "claimed"),
            (Some("indeterminate"), true, false, "claimed"),
            (Some("orphaned"), true, false, "claimed"),
            (Some("acked"), false, false, "delivered"),
            (None, true, true, "delivered"),
        ] {
            task.failure_sent = sent;
            task.delivery_acked = acked;
            assert_eq!(
                task_view(&task, phase, true)["mailbox"],
                json!({"state":expected})
            );
        }
        cleanup(&config);
    }

    #[test]
    fn native151_vue_table_et_detail() {
        let (mut st, config, _rx, _reader) = fixture("vue-table");
        let mut task = tache_151(&mut st, "vue");
        for (state, work) in [
            ("queued", "working"),
            ("starting", "working"),
            ("mission_pending", "working"),
            ("working", "working"),
            ("waiting_for_children", "waiting_for_children"),
            ("cancelling", "working"),
            ("result_available", "result_available"),
            ("failed", "failed"),
            ("cancelled", "cancelled"),
        ] {
            task.state = state.into();
            task.result = Some("résultat privé".into());
            let view = task_view(&task, None, true);
            assert_eq!(view["work_state"], work, "{state}");
            assert_eq!(
                view["result"].as_str(),
                (state == "result_available").then_some("résultat privé"),
                "contrat historique 150 : {state}"
            );
            assert_eq!(
                view["summary"].is_null(),
                state != "result_available",
                "{state}"
            );
            assert_eq!(
                view["result_bytes"].is_null(),
                state != "result_available",
                "{state}"
            );
            if matches!(state, "result_available" | "failed") {
                assert_eq!(
                    view["detail"],
                    json!({"tool":"bridget_task_status","task_id":task.task_id})
                );
            } else {
                assert!(view["detail"].is_null(), "{state}");
            }
        }
        cleanup(&config);
    }

    #[test]
    fn native151_vue_resume_frontieres_et_stabilite() {
        let (mut st, config, _rx, _reader) = fixture("vue-utf8");
        let mut task = tache_151(&mut st, "vue");
        task.state = "result_available".into();
        for body in [
            String::new(),
            "x".repeat(2047),
            "x".repeat(2048),
            "x".repeat(2049),
            format!("{}€z", "x".repeat(2047)),
        ] {
            task.result = Some(body.clone());
            let view = task_view(&task, None, true);
            let summary = view["summary"].as_str().expect("résumé présent");
            assert!(summary.len() <= 2048);
            assert!(body.starts_with(summary));
            assert_eq!(view["result_bytes"], body.len());
            assert_eq!(view["summary_truncated"], summary.len() < body.len());
            assert_eq!(view, task_view(&task, None, true));
            assert_eq!(view["result"], body);
        }
        cleanup(&config);
    }

    #[test]
    fn native151_vue_cles_historiques_conservees() {
        let (mut st, config, _rx, _reader) = fixture("vue-legacy");
        let task = tache_151(&mut st, "vue");
        let view = task_view(&task, None, true);
        let historical = [
            "version",
            "task_id",
            "status",
            "child_agent_id",
            "message_id",
            "agent_type",
            "model",
            "effort",
            "cwd",
            "posture",
            "mission_deadline_at",
            "result",
            "error",
        ];
        assert!(historical.into_iter().all(|key| view.get(key).is_some()));
        assert_eq!(view["status"], "queued");
        assert_eq!(view["agent_type"], "native-test");
        assert_eq!(view["model"], "custom/model");
        assert_eq!(view["effort"], "high");
        assert_eq!(view["work_state"], "working");
        cleanup(&config);
    }

    #[test]
    fn native151_boite_table_et_status_n_ecrit_rien() {
        let (mut st, config, _rx, _reader) = fixture("boite-table");
        let mut task = tache_151(&mut st, "boite");
        let id = task.task_id.clone();
        for state in ["working", "cancelled"] {
            task.state = state.into();
            st.delegation_store.save(&task).unwrap();
            let view = handle_locked(
                "conn-1",
                NativeDelegationRequest::Status {
                    task_id: id.clone(),
                },
                &mut st,
            )
            .unwrap();
            assert!(view.get("mailbox").is_some(), "clé présente");
            assert!(view["mailbox"].is_null());
        }
        task.state = "result_available".into();
        task.result = Some("terminé".into());
        st.delegation_store.save(&task).unwrap();
        let read = |st: &mut DaemonState| {
            handle_locked(
                "conn-1",
                NativeDelegationRequest::Status {
                    task_id: id.clone(),
                },
                st,
            )
            .unwrap()
        };
        assert_eq!(read(&mut st)["mailbox"], json!({"state":"pending"}));
        let delivery = remise_native_150(&mut st, &task, "native-result-", false);
        task.result_sent = true;
        st.delegation_store.save(&task).unwrap();
        assert_eq!(read(&mut st)["mailbox"], json!({"state":"claimed"}));
        st.idempotency
            .mark_delivery_indeterminate(&delivery, "instance-1", 1)
            .unwrap();
        assert_eq!(read(&mut st)["mailbox"], json!({"state":"claimed"}));
        let before = serde_json::to_value(st.delegation_store.get(&id).unwrap().unwrap()).unwrap();
        read(&mut st);
        assert_eq!(
            before,
            serde_json::to_value(st.delegation_store.get(&id).unwrap().unwrap()).unwrap()
        );
        // Une preuve durable d'ACK prévaut même si la ligne de remise n'existe plus.
        task.delivery_acked = true;
        st.delegation_store.save(&task).unwrap();
        assert_eq!(read(&mut st)["mailbox"], json!({"state":"delivered"}));
        cleanup(&config);
    }

    #[test]
    fn native151_ack_lot_premier_timestamp_repeat_et_reouverture() {
        let (mut st, config, _rx, _reader) = fixture("ack-first");
        let model = tache_151(&mut st, "model");
        let result = semer_terminale_150(&st, &model, 15101, "result_available");
        let failed = semer_terminale_150(&st, &model, 15102, "failed");
        let ids = json!([failed.task_id, result.task_id]);
        let epoch = st.delegation_store.wait_epoch();
        let seen = epoch.get();
        let meta = st.delegation_store.projection_meta().unwrap();
        st.delegation_store.take_changes();
        let before = unix_timestamp();
        let ack = |st: &mut DaemonState| {
            handle_locked(
                "conn-1",
                serde_json::from_value(json!({"operation":"ack","task_ids":ids}))
                    .expect("opération ACK décodable"),
                st,
            )
            .unwrap()
        };
        assert_eq!(
            ack(&mut st),
            json!({"version":1,"acknowledged":ids,"already":[]})
        );
        let saved =
            |id: &str| serde_json::to_value(st.delegation_store.get(id).unwrap().unwrap()).unwrap();
        let first = saved(&result.task_id)["acknowledged_at"]
            .as_i64()
            .expect("premier timestamp durable");
        assert!((before..=unix_timestamp()).contains(&first));
        assert_eq!(saved(&failed.task_id)["acknowledged_at"], first);
        assert_eq!(epoch.get(), seen + 1, "un seul réveil pour le lot committé");
        for id in [&result.task_id, &failed.task_id] {
            let task = st.delegation_store.get(id).unwrap().unwrap();
            assert_eq!(
                task_view(&task, None, true)["mailbox"],
                json!({"state":"acknowledged"})
            );
        }
        assert_eq!(
            ack(&mut st),
            json!({"version":1,"acknowledged":[],"already":ids})
        );
        assert_eq!(epoch.get(), seen + 1, "le rejeu ne réveille personne");
        let after_meta = st.delegation_store.projection_meta().unwrap();
        assert_eq!(
            (after_meta.generation, after_meta.seq),
            (meta.generation, meta.seq)
        );
        assert!(st.delegation_store.take_changes().is_empty());
        let reopened = crate::delegation::DelegationStore::open(&config.db_path).unwrap();
        for id in [&result.task_id, &failed.task_id] {
            assert_eq!(
                serde_json::to_value(reopened.get(id).unwrap().unwrap()).unwrap()["acknowledged_at"],
                first
            );
        }
        drop(reopened);
        cleanup(&config);
    }

    #[test]
    fn native151_ack_bornes_et_lot_de_seize() {
        let (mut st, config, _rx, _reader) = fixture("ack-bounds");
        let model = tache_151(&mut st, "model");
        let tasks: Vec<_> = (0..17)
            .map(|i| semer_terminale_150(&st, &model, 15200 + i, "result_available"))
            .collect();
        let ids: Vec<_> = tasks.iter().map(|task| task.task_id.clone()).collect();
        let accuser = |st: &mut DaemonState, ids: Value| {
            handle_locked(
                "conn-1",
                serde_json::from_value(json!({"operation":"ack","task_ids":ids}))
                    .expect("opération ACK décodable"),
                st,
            )
        };
        let before = serde_json::to_value(st.delegation_store.tasks().unwrap()).unwrap();
        let epoch = st.delegation_store.wait_epoch();
        let seen = epoch.get();
        for invalid in [json!([]), json!(ids), json!([ids[0], ids[0]]), json!([""])] {
            assert_eq!(accuser(&mut st, invalid).unwrap_err(), "invalid_request");
            assert_eq!(
                serde_json::to_value(st.delegation_store.tasks().unwrap()).unwrap(),
                before
            );
            assert_eq!(epoch.get(), seen);
        }
        assert_eq!(
            accuser(&mut st, json!([ids[0]])).unwrap(),
            json!({"version":1,"acknowledged":[ids[0]],"already":[]})
        );
        assert_eq!(
            accuser(&mut st, json!(ids[1..])).unwrap(),
            json!({"version":1,"acknowledged":ids[1..],"already":[]})
        );
        cleanup(&config);
    }

    #[test]
    fn native151_ack_propriete_tout_lot_avant_preparation() {
        let (mut st, config, _rx, _reader) = fixture("ack-owner");
        let active = tache_151(&mut st, "active");
        let mut foreign = semer_terminale_150(&st, &active, 15301, "failed");
        foreign.owner = "89000000-0000-4000-8000-000000999999".into();
        foreign.owner_instance = "foreign-instance".into();
        st.delegation_store.save(&foreign).unwrap();
        let before = serde_json::to_value(st.delegation_store.tasks().unwrap()).unwrap();
        let epoch = st.delegation_store.wait_epoch();
        let seen = epoch.get();
        let meta = st.delegation_store.projection_meta().unwrap();
        st.delegation_store.take_changes();
        for other in [foreign.task_id.as_str(), "unknown-task"] {
            for ids in [
                json!([active.task_id, other]),
                json!([other, active.task_id]),
            ] {
                let request = serde_json::from_value(json!({"operation":"ack","task_ids":ids}))
                    .expect("opération ACK décodable");
                assert_eq!(
                    handle_locked("conn-1", request, &mut st).unwrap_err(),
                    "task_unavailable",
                    "propriété complète avant task_not_ready"
                );
                assert_eq!(
                    serde_json::to_value(st.delegation_store.tasks().unwrap()).unwrap(),
                    before
                );
                assert_eq!(epoch.get(), seen);
                let after = st.delegation_store.projection_meta().unwrap();
                assert_eq!((&after.generation, after.seq), (&meta.generation, meta.seq));
                assert!(st.delegation_store.take_changes().is_empty());
            }
        }
        cleanup(&config);
    }

    #[test]
    fn native151_ack_sept_etats_non_prets_sans_mutation() {
        let (mut st, config, _rx, _reader) = fixture("ack-states");
        let mut task = tache_151(&mut st, "task");
        for state in [
            "queued",
            "starting",
            "mission_pending",
            "working",
            "waiting_for_children",
            "cancelling",
            "cancelled",
        ] {
            task.state = state.into();
            st.delegation_store.save(&task).unwrap();
            let before =
                serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap())
                    .unwrap();
            let epoch = st.delegation_store.wait_epoch();
            let seen = epoch.get();
            let meta = st.delegation_store.projection_meta().unwrap();
            st.delegation_store.take_changes();
            let request =
                serde_json::from_value(json!({"operation":"ack","task_ids":[task.task_id]}))
                    .expect("opération ACK décodable");
            assert_eq!(
                handle_locked("conn-1", request, &mut st).unwrap_err(),
                "task_not_ready",
                "{state}"
            );
            assert_eq!(
                serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap())
                    .unwrap(),
                before
            );
            assert_eq!(epoch.get(), seen);
            let after = st.delegation_store.projection_meta().unwrap();
            assert_eq!((after.generation, after.seq), (meta.generation, meta.seq));
            assert!(st.delegation_store.take_changes().is_empty());
        }
        cleanup(&config);
    }

    #[test]
    fn native151_ack_rollback_deuxieme_update_et_commit_sans_reveil() {
        for commit_fault in [false, true] {
            let (mut st, config, _rx, _reader) = fixture("ack-rollback");
            let model = tache_151(&mut st, "model");
            let first = semer_terminale_150(&st, &model, 15401, "result_available");
            let second = semer_terminale_150(&st, &model, 15402, "failed");
            let conn = st.delegation_store.test_connection();
            if commit_fault {
                conn.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE ack151_parent(id TEXT PRIMARY KEY); CREATE TABLE ack151_violation(id TEXT REFERENCES ack151_parent(id) DEFERRABLE INITIALLY DEFERRED);").unwrap();
                conn.execute_batch(&format!("CREATE TRIGGER ack151_commit AFTER UPDATE OF payload ON native_delegations WHEN NEW.task_id='{}' AND json_extract(NEW.payload,'$.acknowledged_at') IS NOT NULL BEGIN INSERT INTO ack151_violation(id) VALUES('absent'); END;", second.task_id)).unwrap();
            } else {
                conn.execute_batch(&format!("CREATE TRIGGER ack151_second BEFORE UPDATE OF payload ON native_delegations WHEN OLD.task_id='{}' AND json_extract(NEW.payload,'$.acknowledged_at') IS NOT NULL BEGIN SELECT RAISE(ABORT,'second update denied'); END;", second.task_id)).unwrap();
            }
            let before: Vec<(String, String)> = st
                .delegation_store
                .test_connection()
                .prepare("SELECT task_id,payload FROM native_delegations ORDER BY task_id")
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            let epoch = st.delegation_store.wait_epoch();
            let seen = epoch.get();
            let meta = st.delegation_store.projection_meta().unwrap();
            st.delegation_store.take_changes();
            let request = serde_json::from_value(
                json!({"operation":"ack","task_ids":[first.task_id, second.task_id]}),
            )
            .expect("opération ACK décodable");
            assert!(
                handle_locked("conn-1", request, &mut st).is_err(),
                "échec SQL attendu, commit={commit_fault}"
            );
            let after: Vec<(String, String)> = st
                .delegation_store
                .test_connection()
                .prepare("SELECT task_id,payload FROM native_delegations ORDER BY task_id")
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(
                after, before,
                "rollback de tout le lot, commit={commit_fault}"
            );
            assert_eq!(epoch.get(), seen, "aucun signal avant un commit réussi");
            let after_meta = st.delegation_store.projection_meta().unwrap();
            assert_eq!(
                (after_meta.generation, after_meta.seq),
                (meta.generation, meta.seq)
            );
            assert!(st.delegation_store.take_changes().is_empty());
            if commit_fault {
                assert_eq!(
                    st.delegation_store
                        .test_connection()
                        .query_row("SELECT count(*) FROM ack151_violation", [], |row| row
                            .get::<_, i64>(0))
                        .unwrap(),
                    0
                );
            }
            cleanup(&config);
        }
    }

    #[test]
    fn native151_ack_sauvegarde_stale_et_forgee_conserve_premier() {
        let (mut st, config, _rx, _reader) = fixture("ack-stale");
        let model = tache_151(&mut st, "model");
        let task = semer_terminale_150(&st, &model, 15501, "result_available");
        let stale = st.delegation_store.get(&task.task_id).unwrap().unwrap();
        let mut forged = serde_json::to_value(&stale).unwrap();
        forged["acknowledged_at"] = json!(12345);
        st.delegation_store
            .save(&serde_json::from_value(forged).unwrap())
            .unwrap();
        assert_eq!(
            serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap())
                .unwrap()
                .get("acknowledged_at"),
            Some(&Value::Null),
            "une sauvegarde ne peut créer l'accusé"
        );
        let request = serde_json::from_value(json!({"operation":"ack","task_ids":[task.task_id]}))
            .expect("opération ACK décodable");
        handle_locked("conn-1", request, &mut st).unwrap();
        let first = serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap())
            .unwrap()["acknowledged_at"]
            .as_i64()
            .unwrap();
        st.delegation_store.save(&stale).unwrap();
        for value in [Value::Null, json!(first + 777)] {
            let mut forged = serde_json::to_value(&stale).unwrap();
            forged["acknowledged_at"] = value;
            st.delegation_store
                .save(&serde_json::from_value(forged).unwrap())
                .unwrap();
            assert_eq!(
                serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap())
                    .unwrap()["acknowledged_at"],
                first
            );
        }
        cleanup(&config);
    }

    #[test]
    fn native151_ack_purete_payload_remise_message_lineage_et_purge() {
        let (mut st, config, _rx, mut reader) = fixture("ack-purity");
        let model = tache_151(&mut st, "model");
        let mut task = semer_terminale_150(&st, &model, 15601, "result_available");
        task.result_sent = true;
        task.cleanup_done = true;
        task.delivery_acked = false;
        st.delegation_store.save(&task).unwrap();
        let delivery = remise_native_150(&mut st, &task, "native-result-", false);
        st.idempotency
            .mark_delivery_indeterminate(&delivery, "instance-1", 1)
            .unwrap();
        // Une ancienne tâche sans dates ne doit pas être estampillée par l'ACK.
        st.delegation_store.test_connection().execute("UPDATE native_delegations SET payload=json_set(payload,'$.updated_at',7,'$.started_at',NULL,'$.completed_at',NULL) WHERE task_id=?1", [&task.task_id]).unwrap();
        let mut before =
            serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap();
        let key = IdempotencyKey::new(
            &crate::communication::issuer_scope(&task.task_id),
            OperationKind::Send,
            &format!("native-result-{}", task.task_id),
        )
        .unwrap();
        let phase = st.idempotency.send_delivery_phase(&key).unwrap();
        let ledger = format!("{:?}", st.store.recent_messages(200).unwrap());
        let meta = st.delegation_store.projection_meta().unwrap();
        st.delegation_store.take_changes();
        let request = serde_json::from_value(json!({"operation":"ack","task_ids":[task.task_id]}))
            .expect("opération ACK décodable");
        handle_locked("conn-1", request, &mut st).unwrap();
        let mut after =
            serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap();
        assert!(after["acknowledged_at"].as_i64().is_some());
        before.as_object_mut().unwrap().remove("acknowledged_at");
        after.as_object_mut().unwrap().remove("acknowledged_at");
        assert_eq!(
            after, before,
            "aucun stamp, date, livraison ou nettoyage modifié"
        );
        assert_eq!(st.idempotency.send_delivery_phase(&key).unwrap(), phase);
        assert_eq!(
            format!("{:?}", st.store.recent_messages(200).unwrap()),
            ledger
        );
        let after_meta = st.delegation_store.projection_meta().unwrap();
        assert_eq!(
            (after_meta.generation, after_meta.seq),
            (meta.generation, meta.seq)
        );
        assert!(st.delegation_store.take_changes().is_empty());
        assert_eq!(
            st.delegation_store
                .purge_trees(unix_timestamp() + 90 * JOUR_150, None, usize::MAX, |_| true)
                .unwrap(),
            0,
            "ACK informatif sans preuve de livraison150"
        );
        assert!(st.delegation_store.get(&task.task_id).unwrap().is_some());
        reader.get_ref().set_nonblocking(true).unwrap();
        let mut wire = String::new();
        assert!(
            matches!(reader.read_line(&mut wire), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
        assert!(wire.is_empty(), "aucun message nouveau");
        cleanup(&config);
    }

    #[test]
    fn native151_ack_ancien_payload_absent_null_sans_dates_inventees() {
        let (mut st, config, _rx, _reader) = fixture("ack-legacy");
        let task = tache_151(&mut st, "legacy");
        let mut old = serde_json::to_value(&task).unwrap();
        for key in [
            "acknowledged_at",
            "updated_at",
            "started_at",
            "completed_at",
        ] {
            old.as_object_mut().unwrap().remove(key);
        }
        let decoded: Task = serde_json::from_value(old).unwrap();
        assert_eq!(decoded.updated_at, 0);
        assert_eq!(decoded.started_at, None);
        assert_eq!(decoded.completed_at, None);
        let encoded = serde_json::to_value(&decoded).unwrap();
        assert_eq!(
            encoded.get("acknowledged_at"),
            Some(&Value::Null),
            "ancien payload décodable et champ NULL sérialisé"
        );
        assert_eq!(encoded["updated_at"], 0);
        assert_eq!(encoded["started_at"], Value::Null);
        assert_eq!(encoded["completed_at"], Value::Null);
        cleanup(&config);
    }

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
            posture: Some(SpawnPosture::Discovery),
            reply_timeout: None,
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
            *posture = Some(SpawnPosture::Discovery);
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
        *posture = Some(SpawnPosture::Discovery);
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
        *posture = Some(SpawnPosture::Discovery);
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
        *posture = Some(SpawnPosture::Discovery);
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
            handle_locked_with_project("conn-1", request("other-project"), Some(&other), None, &mut st)
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
        descendant.parent_task_id = Some(root.task_id.clone());
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
        // Contrat 149 : development sans fait attesté n'ouvre jamais de grant
        // humain, il rend permission_attestation_unavailable (ancien oracle
        // 148 : development_grant_required).
        let mut dev = request("dev");
        if let NativeDelegationRequest::Delegate { posture, .. } = &mut dev {
            *posture = Some(SpawnPosture::Development);
        }
        assert_eq!(
            handle_locked("conn-1", dev, &mut st).unwrap_err(),
            "permission_attestation_unavailable"
        );
        let mut outside = request("outside");
        if let NativeDelegationRequest::Delegate { cwd, posture, .. } = &mut outside {
            *cwd = "/".into();
            *posture = Some(SpawnPosture::Discovery);
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
        assert_eq!(task_view(&restored, None, true), first);
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
        // Contrat 149 : sans fait attesté, le refus development du catalogue est
        // permission_attestation_unavailable (ancien oracle 148 :
        // development_protocol_unavailable).
        assert_eq!(
            glm["development_refusal"],
            "permission_attestation_unavailable"
        );
        let job = NativeDelegationRequest::Delegate {
            request_id: "glm".into(),
            agent_type: "glm-fake".into(),
            model: "glm/custom-exact".into(),
            effort: None,
            task: "Analyse les faits".into(),
            cwd: std::env::temp_dir().to_string_lossy().into_owned(),
            posture: Some(SpawnPosture::Discovery),
            reply_timeout: None,
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
                // Sans fait attesté, la mission 149 est admise en discovery
                // (ancien oracle 148 : development hérité).
                SpawnPosture::Discovery
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

    const JOUR_150: i64 = 24 * 3600;

    /// Termine une tâche et la rend purgeable (annulée, nettoyée).
    fn solder_150(st: &DaemonState, task_id: &str) {
        let mut task = st.delegation_store.get(task_id).unwrap().unwrap();
        task.state = "cancelled".into();
        task.cleanup_done = true;
        st.delegation_store.save(&task).unwrap();
    }

    #[test]
    fn native150_purge_rejeu_cle_purgee() {
        let (mut st, config, _rx, _reader) = fixture("n150-rejeu-purge");
        let first = handle_locked("conn-1", request("cle"), &mut st).unwrap();
        solder_150(&st, first["task_id"].as_str().unwrap());
        let now = unix_timestamp() + 8 * JOUR_150;
        assert_eq!(st.delegation_store.purge_trees(now, None, usize::MAX, |_| true).unwrap(), 1);
        // La clé purgée est refusée, quelle que soit l'enveloppe, sans tâche créée.
        assert_eq!(
            handle_locked("conn-1", request("cle"), &mut st).unwrap_err(),
            "request_expired"
        );
        let mut autre = request("cle");
        if let NativeDelegationRequest::Delegate { task, .. } = &mut autre {
            *task = "autre mission".into();
        }
        assert_eq!(
            handle_locked("conn-1", autre, &mut st).unwrap_err(),
            "request_expired"
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        // Une clé jamais vue reste admise.
        assert!(handle_locked("conn-1", request("neuve"), &mut st).is_ok());
        cleanup(&config);
    }

    #[test]
    fn native150_purge_pierre_expiree_libere_la_cle() {
        let (mut st, config, _rx, _reader) = fixture("n150-pierre");
        let first = handle_locked("conn-1", request("cle"), &mut st).unwrap();
        solder_150(&st, first["task_id"].as_str().unwrap());
        let purged_at = unix_timestamp() + 8 * JOUR_150;
        assert_eq!(st.delegation_store.purge_trees(purged_at, None, usize::MAX, |_| true).unwrap(), 1);
        // Moins de 30 jours après la purge : la pierre reste, la clé reste refusée.
        assert_eq!(st.delegation_store.purge_expired_tombstones(purged_at + 29 * JOUR_150).unwrap(), 0);
        assert_eq!(
            handle_locked("conn-1", request("cle"), &mut st).unwrap_err(),
            "request_expired"
        );
        // Au-delà : pierre purgée, clé libre, nouvelle tâche.
        assert_eq!(st.delegation_store.purge_expired_tombstones(purged_at + 30 * JOUR_150).unwrap(), 1);
        let again = handle_locked("conn-1", request("cle"), &mut st).unwrap();
        assert_ne!(again["task_id"], first["task_id"]);
        cleanup(&config);
    }

    /// Crée une tâche native puis la solde : elle ne dépend plus que du contrôle de vivacité.
    fn tache_soldee_150(st: &mut DaemonState, key: &str) -> Task {
        let first = handle_locked("conn-1", request(key), st).unwrap();
        let id = first["task_id"].as_str().unwrap().to_string();
        solder_150(st, &id);
        st.delegation_store.get(&id).unwrap().unwrap()
    }

    fn ordre_lancement_150(child: &str, command_id: &str) -> FleetSpawnOrder {
        let now = unix_timestamp();
        FleetSpawnOrder {
            posture: None,
            agent_type: "native-test".into(),
            requested_name: Some(child.into()),
            cwd: std::env::temp_dir(),
            persistent: false,
            command_id: command_id.into(),
            issued_at: now,
            deadline_at: now + 60,
            project: None,
            ownership: None,
        }
    }

    #[test]
    fn native150_purge_enfant_present_conserve() {
        let (mut st, config, _rx, _reader) = fixture("n150-present");
        let task = tache_soldee_150(&mut st, "cle");
        let aged = unix_timestamp() + 8 * JOUR_150;
        let mut presence = st.presences.get("instance-1").unwrap().clone();
        presence.name = task.child.clone();
        presence.state = "connected".into();
        st.presences.insert("instance-enfant".into(), presence);
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "enfant connecté");
        st.presences.get_mut("instance-enfant").unwrap().state = "busy".into();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "enfant occupé");
        st.presences.get_mut("instance-enfant").unwrap().state = "stopped".into();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 1, "enfant arrêté : arbre purgé");
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        cleanup(&config);
    }

    #[test]
    fn native150_purge_lancement_en_cours_conserve() {
        let (mut st, config, _rx, _reader) = fixture("n150-lancement");
        let task = tache_soldee_150(&mut st, "cle");
        let aged = unix_timestamp() + 8 * JOUR_150;
        // Saga de lancement sous le superviseur : réservée, pas encore connectée.
        let lease = match st.fleet.request_spawn(&ordre_lancement_150(&task.child, "saga-150"), unix_timestamp()).unwrap() {
            crate::fleet::SpawnSubmission::Start(lease) => lease,
            other => panic!("réservation inattendue: {other:?}"),
        };
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "saga requested/reserved");
        let definition = st.registry.resolved_definition("native-test").unwrap();
        st.fleet.mark_starting(&lease, unix_timestamp(), &definition).unwrap();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "saga starting");
        // Lancement suivi par le daemon (managed_spawns), indépendamment de la saga.
        st.fleet.register_connected(&lease, &lease.instance_id, unix_timestamp()).unwrap();
        st.fleet.mark_stopped(&task.child).unwrap();
        st.managed_spawns.insert(
            lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: lease.clone(),
                agent_type: "native-test".into(),
                requester_conns: Vec::new(),
                wrapper_conn: None,
                stop: Arc::new(ManagedStopControl::new()),
            },
        );
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "managed_spawns");
        st.managed_spawns.clear();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 1);
        cleanup(&config);
    }

    #[test]
    fn native150_purge_agent_route_conserve() {
        let (mut st, config, _rx, _reader) = fixture("n150-route");
        let task = tache_soldee_150(&mut st, "cle");
        let aged = unix_timestamp() + 8 * JOUR_150;
        // Route de routeur vivante, sans présence : l'enfant peut encore recevoir un message.
        st.router.register(&task.child, &bridget_core::AgentType::Codex, "conn-enfant").unwrap();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "agent routé");
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
        st.router.unregister_by_conn("conn-enfant").unwrap();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 1, "route retirée : arbre purgé");
        cleanup(&config);
    }

    #[test]
    fn native150_purge_decommission_en_cours_conserve() {
        let (mut st, config, _rx, _reader) = fixture("n150-decommission");
        let task = tache_soldee_150(&mut st, "cle");
        let aged = unix_timestamp() + 8 * JOUR_150;
        // Une décommission déjà en cours (autre chemin) garde l'arbre jusqu'à sa fin.
        st.decommissioning_names.insert(task.child.clone());
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0, "décommission en cours");
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
        st.decommissioning_names.remove(&task.child);
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 1, "décommission terminée : arbre purgé");
        cleanup(&config);
    }

    #[test]
    fn native150_purge_entree_stopped_decommissionnee() {
        let (mut st, config, _rx, _reader) = fixture("n150-stopped");
        let task = tache_soldee_150(&mut st, "cle");
        let aged = unix_timestamp() + 8 * JOUR_150;
        let lease = match st.fleet.request_spawn(&ordre_lancement_150(&task.child, "saga-stop"), unix_timestamp()).unwrap() {
            crate::fleet::SpawnSubmission::Start(lease) => lease,
            other => panic!("réservation inattendue: {other:?}"),
        };
        let definition = st.registry.resolved_definition("native-test").unwrap();
        st.fleet.mark_starting(&lease, unix_timestamp(), &definition).unwrap();
        st.fleet.register_connected(&lease, &lease.instance_id, unix_timestamp()).unwrap();
        // Entrée désirée Running : le sous-agent peut tourner, l'arbre reste.
        assert_eq!(
            st.fleet.desired_entry(&task.child).unwrap().unwrap().lifecycle_state,
            DesiredLifecycleState::Running
        );
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 0);
        // Entrée Stopped : relançable ; la purge la décommissionne avant de supprimer.
        st.fleet.mark_stopped(&task.child).unwrap();
        assert_eq!(purge_retained(&st, aged, None, usize::MAX), 1);
        assert_eq!(
            st.fleet.desired_entry(&task.child).unwrap().unwrap().lifecycle_state,
            DesiredLifecycleState::Decommissioned
        );
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        cleanup(&config);
    }

    /// Dépose une remise native sous la clé et le périmètre de la tâche, puis l'accuse ou non.
    fn remise_native_150(st: &mut DaemonState, task: &Task, prefix: &str, accuser: bool) -> String {
        let id = format!("{prefix}{}", task.task_id);
        let key = IdempotencyKey::new(&crate::communication::issuer_scope(&task.task_id), OperationKind::Send, &id).unwrap();
        let now = unix_timestamp();
        st.idempotency
            .reserve(&key, id.as_bytes(), now, CLIENT_IDEMPOTENCY_HORIZON_SECS, now, CLIENT_ISSUED_AT_TOLERANCE_SECS)
            .unwrap();
        let mut message = bridget_core::BridgetMessage::new(&task.child, &task.owner, "sortie");
        message.id = id.clone();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{id}"),
            recipient_instance_id: "instance-1".into(),
            delivery_generation: 1,
            expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
            message_bytes: serde_json::to_vec(&message).unwrap(),
        };
        st.idempotency.begin_send_delivery(&key, &delivery).unwrap();
        if accuser {
            st.idempotency.acknowledge_send_delivery(&delivery.delivery_id, "instance-1", 1).unwrap();
        }
        delivery.delivery_id
    }

    /// Copie le modèle en tâche terminale `etat`, remise envoyée, nettoyage fait.
    fn semer_terminale_150(st: &DaemonState, modele: &Task, i: usize, etat: &str) -> Task {
        let mut task = modele.clone();
        task.task_id = format!("t150-{i:04}");
        task.child = format!("child-150-{i:04}");
        task.mission = format!("mission-150-{i:04}");
        task.state = etat.into();
        task.cleanup_done = true;
        task.result = Some("sortie".into());
        task.result_sent = etat == "result_available";
        task.failure_sent = etat == "failed";
        task.completed_at = Some(unix_timestamp() + 120 * JOUR_150);
        st.delegation_store.insert(&task, &format!("req-150-{i:04}"), b"canon").unwrap();
        task
    }

    #[test]
    fn native150_purge_accuse_recopie() {
        let (mut st, config, _rx, _reader) = fixture("n150-accuse");
        let modele = tache_soldee_150(&mut st, "modele");
        let resultat = semer_terminale_150(&st, &modele, 1, "result_available");
        let echec = semer_terminale_150(&st, &modele, 2, "failed");
        let non_accusee = semer_terminale_150(&st, &modele, 3, "result_available");
        remise_native_150(&mut st, &resultat, "native-result-", true);
        remise_native_150(&mut st, &echec, "native-failure-", true);
        remise_native_150(&mut st, &non_accusee, "native-result-", false);
        assert_eq!(copy_delivery_acks(&st).unwrap(), 2);
        let acked = |id: &str| st.delegation_store.get(id).unwrap().unwrap().delivery_acked;
        assert!(acked(&resultat.task_id) && acked(&echec.task_id));
        assert!(!acked(&non_accusee.task_id), "remise en vol : aucune preuve");
        assert_eq!(copy_delivery_acks(&st).unwrap(), 0, "idempotent");
        cleanup(&config);
    }

    #[test]
    fn native150_purge_outcome_unknown_puis_remise_effacee_conserve() {
        let (mut st, config, _rx, _reader) = fixture("n150-inconnu");
        let modele = tache_soldee_150(&mut st, "modele");
        let task = semer_terminale_150(&st, &modele, 1, "result_available");
        let livraison = remise_native_150(&mut st, &task, "native-result-", false);
        st.idempotency.mark_delivery_indeterminate(&livraison, "instance-1", 1).unwrap();
        assert_eq!(copy_delivery_acks(&st).unwrap(), 0, "remise indéterminée : pas d'accusé");
        // L'entretien efface la remise (ligne absente) : toujours aucune preuve.
        let loin = unix_timestamp() + 2 * CLIENT_IDEMPOTENCY_HORIZON_SECS;
        assert_eq!(st.idempotency.purge_expired_sends(loin, 0).unwrap(), 1);
        assert_eq!(copy_delivery_acks(&st).unwrap(), 0);
        // L'arbre ne se purge pas, même très ancien et sous pression de borne :
        // seul l'arbre modèle (annulé, sans remise due) part.
        assert_eq!(purge_retained(&st, unix_timestamp() + 90 * JOUR_150, Some(1), usize::MAX), 1);
        assert!(st.delegation_store.get(&task.task_id).unwrap().is_some());
        cleanup(&config);
    }

    #[test]
    fn native150_purge_129_accuses_expires_tous_recopies() {
        let (mut st, config, _rx, _reader) = fixture("n150-129");
        let modele = tache_soldee_150(&mut st, "modele");
        let total = crate::disk_hygiene::MAX_ENTRIES_PER_PASS + 1;
        let ids: Vec<String> = (1..=total)
            .map(|i| {
                let task = semer_terminale_150(&st, &modele, i, "result_available");
                remise_native_150(&mut st, &task, "native-result-", true);
                task.task_id
            })
            .collect();
        // Toutes les remises ont expiré : l'entretien recopie AVANT de les effacer.
        // Les tâches finissent dans le futur : l'âge les protège de la purge d'arbre.
        let loin = unix_timestamp() + 90 * JOUR_150;
        super::super::maintain_stores(&st, loin);
        let cle = IdempotencyKey::new(&crate::communication::issuer_scope(&ids[0]), OperationKind::Send, &format!("native-result-{}", ids[0])).unwrap();
        assert_eq!(st.idempotency.send_delivery_phase(&cle).unwrap(), None, "remise effacée");
        for id in &ids {
            assert!(st.delegation_store.get(id).unwrap().unwrap().delivery_acked, "{id}");
        }
        cleanup(&config);
    }

    /// Sème des tâches terminales ou actives par SQL direct (une transaction), sans
    /// passer par l'admission. `completed_at` croît avec `i` : plus petit = plus ancien.
    #[allow(clippy::too_many_arguments)]
    fn semer_brut_150(
        config: &DaemonConfig,
        modele: &Task,
        debut: usize,
        nombre: usize,
        etat: &str,
        acquittee: bool,
        instance: &str,
        fin_base: i64,
    ) {
        let mut conn = rusqlite::Connection::open(&config.db_path).unwrap();
        conn.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
        let tx = conn.transaction().unwrap();
        for i in debut..debut + nombre {
            let mut task = modele.clone();
            task.task_id = format!("b150-{i:05}");
            task.child = format!("child-b150-{i:05}");
            task.mission = format!("mission-b150-{i:05}");
            task.owner_instance = instance.into();
            task.root_owner_agent_id = task.owner.clone();
            task.state = etat.into();
            task.cleanup_done = matches!(etat, "result_available" | "failed" | "cancelled");
            task.delivery_acked = acquittee;
            task.result_sent = etat == "result_available";
            task.result = (etat == "result_available").then(|| "sortie".to_string());
            task.completed_at = task.cleanup_done.then_some(fin_base + i as i64);
            tx.execute(
                "INSERT INTO native_delegations(task_id,owner_instance,request_id,canonical,payload) VALUES(?1,?2,?3,?4,?5)",
                rusqlite::params![task.task_id, task.owner_instance, format!("req-b150-{i:05}"), b"canon".to_vec(), serde_json::to_string(&task).unwrap()],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    fn modele_150(st: &mut DaemonState) -> Task {
        let mut modele = tache_soldee_150(st, "modele-admission");
        // Le modèle est purgé du magasin : seules ses données servent de gabarit.
        st.delegation_store.purge_trees(unix_timestamp() + 90 * JOUR_150, None, usize::MAX, |_| true).unwrap();
        modele.parent_task_id = None;
        modele
    }

    #[test]
    fn native150_admission_apres_5000_terminees() {
        let (mut st, config, _rx, _reader) = fixture("n150-5000");
        let modele = modele_150(&mut st);
        semer_brut_150(&config, &modele, 0, 5000, "result_available", true, "instance-autre", 1_000);
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 5000);
        let admise = handle_locked("conn-1", request("apres-5000"), &mut st).unwrap();
        assert_eq!(admise["status"], "queued");
        let restantes = st.delegation_store.tasks().unwrap().len();
        assert!(restantes <= 4096, "la borne de rétention tient : {restantes}");
        cleanup(&config);
    }

    #[test]
    fn native150_admission_saturation_sans_purgeable() {
        let (mut st, config, _rx, _reader) = fixture("n150-saturation");
        let modele = modele_150(&mut st);
        // 4096 résultats jamais accusés : rien n'est purgeable, aucune perte permise.
        semer_brut_150(&config, &modele, 0, 4096, "result_available", false, "instance-autre", 1_000);
        assert_eq!(
            handle_locked("conn-1", request("sature"), &mut st).unwrap_err(),
            "retention_saturated"
        );
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 4096, "aucune écriture ni purge");
        cleanup(&config);
    }

    #[test]
    fn native150_admission_limites_actives_inchangees() {
        let (mut st, config, _rx, _reader) = fixture("n150-actives");
        let modele = modele_150(&mut st);
        // 16 actives pour l'instance appelante : la 17e est refusée.
        semer_brut_150(&config, &modele, 0, 16, "working", false, "instance-1", 0);
        assert_eq!(handle_locked("conn-1", request("17e"), &mut st).unwrap_err(), "task_limit");
        // 128 actives au total (autres instances) : refus aussi pour une instance libre.
        sqlite_supprimer_instance_150(&config, "instance-1");
        semer_brut_150(&config, &modele, 16, 128, "working", false, "instance-autre", 0);
        assert_eq!(handle_locked("conn-1", request("129e"), &mut st).unwrap_err(), "task_limit");
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 128);
        cleanup(&config);
    }

    fn sqlite_supprimer_instance_150(config: &DaemonConfig, instance: &str) {
        rusqlite::Connection::open(&config.db_path)
            .unwrap()
            .execute("DELETE FROM native_delegations WHERE owner_instance=?1", [instance])
            .unwrap();
    }

    #[test]
    fn native150_admission_purge_de_borne_plus_ancien_d_abord() {
        let (mut st, config, _rx, _reader) = fixture("n150-borne");
        let modele = modele_150(&mut st);
        // 4096 arbres soldés de moins de 7 jours : l'âge ne protège plus sous pression.
        let recent = unix_timestamp() - 3600;
        semer_brut_150(&config, &modele, 0, 4096, "result_available", true, "instance-autre", recent - 5000);
        let admise = handle_locked("conn-1", request("borne"), &mut st).unwrap();
        assert_eq!(admise["status"], "queued");
        let ids: HashSet<String> = st.delegation_store.tasks().unwrap().into_iter().map(|t| t.task_id).collect();
        assert_eq!(ids.len(), 4096, "le total ne dépasse jamais 4096");
        assert!(!ids.contains("b150-00000"), "le plus ancien est purgé en premier");
        assert!(ids.contains("b150-00001"), "un seul arbre est libéré");
        assert!(ids.contains(admise["task_id"].as_str().unwrap()));
        cleanup(&config);
    }

    /// Pause ou levée du référent, comme le principal humain le ferait.
    fn pauser_150(st: &DaemonState, pause: bool) {
        let conn = st.store.connection();
        let generation = crate::referent_control::read(conn).unwrap().generation;
        crate::referent_control::set(
            conn,
            crate::referent_control::ControlMutation {
                command_id: &format!("pause-150-{generation}"),
                expected_generation: generation,
                paused: Some(pause),
                auto_objectives_cap: None,
                reason: pause.then_some("test 150"),
                actor: "humain",
                now: unix_timestamp(),
                agent_posture: None,
                auto_reassignment: None,
            },
        )
        .unwrap()
        .unwrap();
    }

    #[test]
    fn native150_pause_refuse_nouvelle_cle() {
        let (mut st, config, rx, _reader) = fixture("n150-pause");
        pauser_150(&st, true);
        assert_eq!(handle_locked("conn-1", request("neuve"), &mut st).unwrap_err(), "referent_paused");
        assert!(st.delegation_store.tasks().unwrap().is_empty(), "aucune tâche créée");
        assert!(rx.try_recv().is_err());
        cleanup(&config);
    }

    #[test]
    fn native150_pause_etat_illisible_refus_ferme() {
        let (mut st, config, _rx, _reader) = fixture("n150-pause-illisible");
        st.store.connection().execute("DELETE FROM control_state WHERE id = 1", []).unwrap();
        assert_eq!(handle_locked("conn-1", request("neuve"), &mut st).unwrap_err(), "control_state_unavailable");
        assert!(st.delegation_store.tasks().unwrap().is_empty(), "aucune tâche créée");
        cleanup(&config);
    }

    #[test]
    fn native150_pause_rejeu_cle_admise_rend_la_tache() {
        let (mut st, config, _rx, _reader) = fixture("n150-pause-rejeu");
        let first = handle_locked("conn-1", request("admise"), &mut st).unwrap();
        pauser_150(&st, true);
        assert_eq!(handle_locked("conn-1", request("admise"), &mut st).unwrap(), first);
        // Un rejeu d'une clé purgée garde son refus propre, avant la pause.
        solder_150(&st, first["task_id"].as_str().unwrap());
        st.delegation_store.purge_trees(unix_timestamp() + 8 * JOUR_150, None, usize::MAX, |_| true).unwrap();
        assert_eq!(handle_locked("conn-1", request("admise"), &mut st).unwrap_err(), "request_expired");
        cleanup(&config);
    }

    #[test]
    fn native150_pause_cle_libre_apres_levee() {
        let (mut st, config, _rx, _reader) = fixture("n150-pause-levee");
        pauser_150(&st, true);
        assert_eq!(handle_locked("conn-1", request("k"), &mut st).unwrap_err(), "referent_paused");
        pauser_150(&st, false);
        let admise = handle_locked("conn-1", request("k"), &mut st).unwrap();
        assert_eq!(admise["status"], "queued");
        cleanup(&config);
    }

    #[test]
    fn native150_pause_statut_et_annulation_disponibles() {
        let (mut st, config, _rx, _reader) = fixture("n150-pause-lecture");
        let first = handle_locked("conn-1", request("k"), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_string();
        pauser_150(&st, true);
        let catalogue = handle_locked("conn-1", NativeDelegationRequest::Catalogue, &mut st).unwrap();
        assert_eq!(catalogue["version"], 1);
        let statut = handle_locked("conn-1", NativeDelegationRequest::Status { task_id: task_id.clone() }, &mut st).unwrap();
        assert_eq!(statut["task_id"], first["task_id"]);
        let annulee = handle_locked("conn-1", NativeDelegationRequest::Cancel { task_id }, &mut st).unwrap();
        assert_eq!(annulee["status"], "cancelling");
        cleanup(&config);
    }

    fn request_delai_150(key: &str, reply_timeout: Option<u64>) -> NativeDelegationRequest {
        let mut req = request(key);
        if let NativeDelegationRequest::Delegate { reply_timeout: slot, .. } = &mut req {
            *slot = reply_timeout;
        }
        req
    }

    #[test]
    fn native150_delai_bornes() {
        let (mut st, config, _rx, _reader) = fixture("n150-delai-bornes");
        for hors_bornes in [0, 59, 86_401, u64::MAX] {
            assert_eq!(
                handle_locked("conn-1", request_delai_150("hors", Some(hors_bornes)), &mut st).unwrap_err(),
                "reply_timeout_out_of_range",
                "{hors_bornes}"
            );
        }
        assert!(st.delegation_store.tasks().unwrap().is_empty(), "aucune tâche, aucune valeur corrigée");
        for (cle, secs) in [("min", 60), ("max", 86_400), ("milieu", 1_800)] {
            let admise = handle_locked("conn-1", request_delai_150(cle, Some(secs)), &mut st).unwrap();
            let task = st.delegation_store.get(admise["task_id"].as_str().unwrap()).unwrap().unwrap();
            assert_eq!(task.reply_timeout, Some(secs as i64));
        }
        // Sans valeur : rien n'est stocké, le défaut 3600 s s'applique plus tard.
        let defaut = handle_locked("conn-1", request_delai_150("defaut", None), &mut st).unwrap();
        assert_eq!(st.delegation_store.get(defaut["task_id"].as_str().unwrap()).unwrap().unwrap().reply_timeout, None);
        cleanup(&config);
    }

    #[test]
    fn native150_delai_rejeu_autre_valeur_envelope_mismatch() {
        let (mut st, config, _rx, _reader) = fixture("n150-delai-rejeu");
        let first = handle_locked("conn-1", request_delai_150("k", Some(120)), &mut st).unwrap();
        assert_eq!(handle_locked("conn-1", request_delai_150("k", Some(120)), &mut st).unwrap(), first);
        for autre in [Some(121), None, Some(3600)] {
            assert_eq!(
                handle_locked("conn-1", request_delai_150("k", autre), &mut st).unwrap_err(),
                "envelope_mismatch",
                "{autre:?}"
            );
        }
        // « omis » et « 3600 explicite » sont deux enveloppes.
        handle_locked("conn-1", request_delai_150("omis", None), &mut st).unwrap();
        assert_eq!(
            handle_locked("conn-1", request_delai_150("omis", Some(3600)), &mut st).unwrap_err(),
            "envelope_mismatch"
        );
        cleanup(&config);
    }

    /// Une mission parente en cours (`working`, échéance posée) dont l'enfant délègue à son tour.
    fn parent_en_cours_150(delai: Option<u64>, retard: i64) -> (Arc<Mutex<DaemonState>>, DaemonConfig, String, i64, bridget_core::BridgetMessage) {
        parent_en_cours_bloque_150(delai, retard, false)
    }

    /// `bloquer_suivi` : toute mise à jour d'échéance du suivi échoue pendant l'envoi de la mission.
    fn parent_en_cours_bloque_150(delai: Option<u64>, retard: i64, bloquer_suivi: bool) -> (Arc<Mutex<DaemonState>>, DaemonConfig, String, i64, bridget_core::BridgetMessage) {
        use std::io::BufRead;
        let (mut st, config, rx, _parent_reader) = fixture("n150-delai-parent");
        let first = handle_locked("conn-1", request_delai_150("parent", delai), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_owned();
        if retard > 0 {
            // Lancement retardé : la tâche a été admise `retard` secondes avant d'être lancée.
            let mut task = st.delegation_store.get(&task_id).unwrap().unwrap();
            task.created_at -= retard;
            st.delegation_store.save(&task).unwrap();
        }
        let shared = Arc::new(Mutex::new(st));
        tick(&shared);
        let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
            panic!("Start attendu")
        };
        let (child_writer, mut child_reader) = super::super::presence_tests::control_socket("n150child");
        child_reader.get_ref().set_read_timeout(Some(Duration::from_millis(50))).unwrap();
        {
            let mut st = shared.lock().unwrap();
            assert!(matches!(
                handle_register_with_channel(
                    "child-conn", 2, "native-test".into(), prepared.lease.name.clone(),
                    Some(st.host.clone()), Some("codex_app_server".into()),
                    ChannelReport::Known("unix".into()), Some(PresenceMode::Acp), None,
                    Some("macOS".into()), Some(prepared.lease.instance_id.clone()), None,
                    false, Some(true), &mut st
                ),
                DaemonToWrapper::Registered { .. }
            ));
            st.connections.insert("child-conn".into(), child_writer);
            st.conn_instances.insert("child-conn".into(), prepared.lease.instance_id.clone());
            st.fleet.register_connected(&prepared.lease, &prepared.lease.instance_id, unix_timestamp()).unwrap();
            st.managed_spawns.get_mut(&task_id).unwrap().wrapper_conn = Some("child-conn".into());
        }
        if bloquer_suivi {
            rusqlite::Connection::open(&config.db_path)
                .unwrap()
                .execute_batch("CREATE TRIGGER s150_suivi_bloque BEFORE UPDATE OF deadline_at ON tracked_requests BEGIN SELECT RAISE(ABORT,'bloque'); END;")
                .unwrap();
        }
        tick(&shared);
        let mut line = String::new();
        child_reader.read_line(&mut line).unwrap();
        let DaemonToWrapper::DeliverIdempotent { message, .. } = decode(line.trim()).unwrap() else {
            panic!("mission idempotente attendue")
        };
        let deadline = {
            let st = shared.lock().unwrap();
            let parent = st.delegation_store.get(&task_id).unwrap().unwrap();
            assert_eq!(parent.state, "working");
            parent.mission_deadline_at.expect("échéance du parent posée")
        };
        (shared, config, task_id, deadline, message)
    }

    #[test]
    fn native150_delai_enfant_explicite_trop_long() {
        let (shared, config, parent_id, deadline, _) = parent_en_cours_150(None, 0);
        let mut st = shared.lock().unwrap();
        // Il reste au plus 3600 s au parent : 3601 s explicites le dépassent.
        assert!(deadline - unix_timestamp() <= 3600);
        assert_eq!(
            handle_locked("child-conn", request_delai_150("sous-long", Some(3601)), &mut st).unwrap_err(),
            "reply_timeout_exceeds_parent"
        );
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1, "aucune tâche créée");
        let court = handle_locked("child-conn", request_delai_150("sous-court", Some(120)), &mut st).unwrap();
        let sous = st.delegation_store.get(court["task_id"].as_str().unwrap()).unwrap().unwrap();
        assert_eq!(sous.parent_task_id.as_deref(), Some(parent_id.as_str()));
        assert_eq!(sous.reply_timeout, Some(120));
        drop(st);
        cleanup(&config);
    }

    #[test]
    fn native150_delai_enfant_implicite_inchange() {
        let (shared, config, _parent_id, _deadline, _) = parent_en_cours_150(None, 0);
        let mut st = shared.lock().unwrap();
        // Sans délai explicite : aucun contrôle, comme en 149.
        let implicite = handle_locked("child-conn", request_delai_150("sous-implicite", None), &mut st).unwrap();
        let sous = st.delegation_store.get(implicite["task_id"].as_str().unwrap()).unwrap().unwrap();
        assert_eq!(sous.reply_timeout, None);
        drop(st);
        cleanup(&config);
    }

    #[test]
    fn native150_delai_applique_echeance_et_suivi() {
        let (shared, config, task_id, deadline, message) = parent_en_cours_150(Some(120), 0);
        assert_eq!(message.reply_timeout, Some(120), "suivi de la demande de mission");
        let created_at = shared.lock().unwrap().delegation_store.get(&task_id).unwrap().unwrap().created_at;
        assert!((120..=125).contains(&(deadline - created_at)), "échéance = admission + 120 s : {}", deadline - created_at);
        drop(shared);
        cleanup(&config);
        // Sans délai explicite : 3600 s, comme en 149.
        let (shared, config, _task_id, _deadline, message) = parent_en_cours_150(None, 0);
        assert_eq!(message.reply_timeout, Some(3600));
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native150_pause_precede_refus_de_droits() {
        let (mut st, config, _rx, _reader) = fixture("n150-pause-droits");
        st.delegation_store
            .grant("instance-1", Path::new("/tmp"), SpawnPosture::Development, true)
            .unwrap();
        // Contrôle positif : sans pause, la révocation donne le refus de droits.
        assert_eq!(handle_locked("conn-1", request("k1"), &mut st).unwrap_err(), "delegation_grant_required");
        pauser_150(&st, true);
        assert_eq!(handle_locked("conn-1", request("k2"), &mut st).unwrap_err(), "referent_paused");
        assert!(st.delegation_store.tasks().unwrap().is_empty(), "aucune écriture");
        cleanup(&config);
    }

    #[test]
    fn native150_statut_tache_purgee_rend_task_unavailable() {
        let (mut st, config, _rx, _reader) = fixture("n150-statut-purge");
        let first = handle_locked("conn-1", request("k"), &mut st).unwrap();
        let task_id = first["task_id"].as_str().unwrap().to_string();
        solder_150(&st, &task_id);
        assert_eq!(st.delegation_store.purge_trees(unix_timestamp() + 8 * JOUR_150, None, usize::MAX, |_| true).unwrap(), 1);
        assert_eq!(
            handle_locked("conn-1", NativeDelegationRequest::Status { task_id }, &mut st).unwrap_err(),
            "task_unavailable"
        );
        cleanup(&config);
    }

    #[test]
    fn native150_delai_enfant_explicite_parent_sans_echeance() {
        let (shared, config, parent_id, _deadline, _) = parent_en_cours_150(None, 0);
        let mut st = shared.lock().unwrap();
        // Parent sans échéance durable : aucun contrôle (R8), même pour le maximum.
        let mut parent = st.delegation_store.get(&parent_id).unwrap().unwrap();
        parent.mission_deadline_at = None;
        st.delegation_store.save(&parent).unwrap();
        let admise = handle_locked("child-conn", request_delai_150("sous-sans-echeance", Some(86_400)), &mut st).unwrap();
        let sous = st.delegation_store.get(admise["task_id"].as_str().unwrap()).unwrap().unwrap();
        assert_eq!(sous.reply_timeout, Some(86_400));
        drop(st);
        cleanup(&config);
    }

    #[test]
    fn native150_purge_echec_recopie_suspend_purge_des_envois() {
        let (mut st, config, _rx, _reader) = fixture("n150-echec-recopie");
        let modele = tache_soldee_150(&mut st, "modele");
        let task = semer_terminale_150(&st, &modele, 1, "result_available");
        remise_native_150(&mut st, &task, "native-result-", true);
        let cle = IdempotencyKey::new(&crate::communication::issuer_scope(&task.task_id), OperationKind::Send, &format!("native-result-{}", task.task_id)).unwrap();
        let loin = unix_timestamp() + 90 * JOUR_150;
        // Écriture impossible : la preuve d'accusé ne peut pas être enregistrée.
        let externe = rusqlite::Connection::open(&config.db_path).unwrap();
        externe.execute_batch("CREATE TRIGGER s150_bloque BEFORE UPDATE ON native_delegations BEGIN SELECT RAISE(ABORT,'bloque'); END;").unwrap();
        assert!(copy_delivery_acks(&st).is_err());
        super::super::maintain_stores(&st, loin);
        assert_eq!(st.idempotency.send_delivery_phase(&cle).unwrap().as_deref(), Some("acked"), "la remise survit au passage");
        assert!(!st.delegation_store.get(&task.task_id).unwrap().unwrap().delivery_acked);
        // L'écriture revient : la preuve est recopiée, puis seulement la remise est purgée.
        externe.execute_batch("DROP TRIGGER s150_bloque;").unwrap();
        super::super::maintain_stores(&st, loin);
        assert!(st.delegation_store.get(&task.task_id).unwrap().unwrap().delivery_acked);
        assert_eq!(st.idempotency.send_delivery_phase(&cle).unwrap(), None);
        cleanup(&config);
    }

    #[test]
    fn native150_purge_arbre_volumineux_une_ecriture_de_flotte() {
        let (mut st, config, _rx, _reader) = fixture("n150-lot-flotte");
        let modele = tache_soldee_150(&mut st, "modele");
        let definition = st.registry.resolved_definition("native-test").unwrap();
        // Arbre : une racine et 4 enfants, chacun avec une entrée de flotte réellement présente.
        let mut ids = Vec::new();
        for i in 0..5 {
            let mut task = modele.clone();
            task.task_id = format!("lot150-{i}");
            task.child = Uuid::new_v4().to_string();
            task.mission = Uuid::new_v4().to_string();
            task.parent_task_id = (i > 0).then(|| "lot150-0".to_string());
            task.state = "cancelled".into();
            task.cleanup_done = true;
            task.completed_at = Some(unix_timestamp() + 120 * JOUR_150);
            st.delegation_store.insert(&task, &format!("req-lot150-{i}"), b"canon").unwrap();
            let lease = match st.fleet.request_spawn(&ordre_lancement_150(&task.child, &format!("saga-lot-{i}")), unix_timestamp()).unwrap() {
                crate::fleet::SpawnSubmission::Start(lease) => lease,
                other => panic!("réservation inattendue: {other:?}"),
            };
            st.fleet.mark_starting(&lease, unix_timestamp(), &definition).unwrap();
            st.fleet.register_connected(&lease, &lease.instance_id, unix_timestamp()).unwrap();
            st.fleet.mark_stopped(&task.child).unwrap();
            ids.push(task.child);
        }
        let avant = st.fleet.desired_write_count();
        assert_eq!(purge_retained(&st, unix_timestamp() + 200 * JOUR_150, None, usize::MAX), 2, "modèle + arbre de test");
        assert_eq!(st.fleet.desired_write_count() - avant, 1, "une seule écriture de flotte pour tout l'arbre");
        for child in &ids {
            assert_eq!(
                st.fleet.desired_entry(child).unwrap().unwrap().lifecycle_state,
                DesiredLifecycleState::Decommissioned
            );
        }
        cleanup(&config);
    }

    #[test]
    fn native150_pause_refus_par_le_gestionnaire_complet_ne_fait_rien_avancer() {
        let (mut st, config, rx, _reader) = fixture("n150-gestionnaire");
        // Tâche préexistante en attente : seul un tick la ferait avancer.
        let attente = handle_locked("conn-1", request("attente"), &mut st).unwrap();
        pauser_150(&st, true);
        let shared = Arc::new(Mutex::new(st));
        let DaemonToWrapper::NativeDelegationResult { result } = handle("conn-1", request("refusee"), &shared) else {
            panic!("résultat attendu")
        };
        assert_eq!(result["code"], "referent_paused", "{result}");
        assert!(rx.try_recv().is_err(), "un refus ne lance aucun tick");
        let task = shared.lock().unwrap().delegation_store.get(attente["task_id"].as_str().unwrap()).unwrap().unwrap();
        assert_eq!(task.state, "queued");
        // Contrôle positif : une délégation admise (clé déjà admise) fait avancer le tick.
        let DaemonToWrapper::NativeDelegationResult { result } = handle("conn-1", request("attente"), &shared) else {
            panic!("résultat attendu")
        };
        assert_eq!(result["status"], "queued");
        assert!(matches!(rx.try_recv(), Ok(ManagedSupervisorCommand::Start { .. })));
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native150_entretien_purge_de_borne_sans_admission() {
        let (mut st, config, _rx, _reader) = fixture("n150-entretien-borne");
        let modele = modele_150(&mut st);
        // 5000 arbres soldés et récents : seule la pression de borne peut les purger.
        let recent = unix_timestamp() - 3600;
        semer_brut_150(&config, &modele, 0, 5000, "result_available", true, "instance-autre", recent - 6000);
        let mut precedent = 5000;
        let mut passes = 0;
        loop {
            super::super::maintain_stores(&st, unix_timestamp());
            let restantes = st.delegation_store.tasks().unwrap().len();
            assert!(precedent - restantes <= crate::disk_hygiene::MAX_ENTRIES_PER_PASS, "au plus 128 arbres par passage");
            if restantes == precedent {
                break;
            }
            precedent = restantes;
            passes += 1;
            assert!(passes <= 10, "la purge doit converger");
        }
        assert!(precedent < 4096, "total sous la borne : {precedent}");
        assert!(precedent >= 4096 - crate::disk_hygiene::MAX_ENTRIES_PER_PASS, "pas de purge au-delà du nécessaire : {precedent}");
        assert!(passes >= 8, "plusieurs passages : {passes}");
        cleanup(&config);
    }

    #[test]
    fn native150_entretien_borne_et_age_cumulent_au_plus_128() {
        let (mut st, config, _rx, _reader) = fixture("n150-entretien-cumul");
        let modele = modele_150(&mut st);
        // 200 arbres soldés de plus de 7 jours (purge d'âge) et 4100 récents (pression de borne).
        let ancien = unix_timestamp() - 30 * JOUR_150;
        let recent = unix_timestamp() - 3600;
        semer_brut_150(&config, &modele, 0, 200, "result_available", true, "instance-autre", ancien);
        semer_brut_150(&config, &modele, 200, 4100, "result_available", true, "instance-autre", recent - 6000);
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 4300);
        super::super::maintain_stores(&st, unix_timestamp());
        let restantes = st.delegation_store.tasks().unwrap().len();
        assert!(
            4300 - restantes <= crate::disk_hygiene::MAX_ENTRIES_PER_PASS,
            "au plus 128 arbres par passage, âge et borne cumulés : {restantes}"
        );
        assert_eq!(restantes, 4300 - crate::disk_hygiene::MAX_ENTRIES_PER_PASS, "le quota du passage est consommé");
        cleanup(&config);
    }

    #[test]
    fn native150_delai_suivi_realigne_apres_echec_d_ecriture() {
        // Écriture du suivi impossible au moment de l'envoi : il garde l'échéance datée de la création.
        let (shared, config, task_id, deadline, message) = parent_en_cours_bloque_150(Some(120), 30, true);
        let suivi_deadline = |shared: &Arc<Mutex<DaemonState>>| {
            shared.lock().unwrap().store.get_request(&message.id).unwrap().expect("demande suivie").deadline_at
        };
        assert_ne!(suivi_deadline(&shared), deadline, "premier alignement échoué");
        // Tant que l'écriture échoue, le tick réessaie sans casser la tâche.
        tick(&shared);
        assert_ne!(suivi_deadline(&shared), deadline);
        assert_eq!(shared.lock().unwrap().delegation_store.get(&task_id).unwrap().unwrap().state, "working");
        // L'écriture revient : le tick suivant réaligne le suivi sur mission_deadline_at.
        rusqlite::Connection::open(&config.db_path).unwrap().execute_batch("DROP TRIGGER s150_suivi_bloque;").unwrap();
        tick(&shared);
        assert_eq!(suivi_deadline(&shared), deadline, "suivi réaligné");
        drop(shared);
        cleanup(&config);
    }

    #[test]
    fn native150_delai_suivi_aligne_sur_admission_apres_lancement_retarde() {
        // Délai explicite, lancement retardé de 30 s : le suivi durable suit l'admission de la mission.
        let (shared, config, task_id, deadline, message) = parent_en_cours_150(Some(120), 30);
        {
            let st = shared.lock().unwrap();
            let task = st.delegation_store.get(&task_id).unwrap().unwrap();
            let suivi = st.store.get_request(&message.id).unwrap().expect("demande suivie");
            assert_eq!(suivi.deadline_at, deadline, "suivi aligné sur mission_deadline_at");
            assert!(deadline - task.created_at >= 150, "l'échéance compte depuis l'admission, pas la création");
        }
        // L'enveloppe de l'envoi de mission n'a pas changé : le délai reste celui du parent.
        assert_eq!(message.reply_timeout, Some(120));
        drop(shared);
        cleanup(&config);
        // Sans délai explicite : comportement 149 inchangé (suivi daté de la création).
        let (shared, config, task_id, deadline, message) = parent_en_cours_150(None, 30);
        {
            let st = shared.lock().unwrap();
            let task = st.delegation_store.get(&task_id).unwrap().unwrap();
            let suivi = st.store.get_request(&message.id).unwrap().expect("demande suivie");
            assert_eq!(suivi.deadline_at, task.created_at + 3600);
            assert!(deadline > suivi.deadline_at);
        }
        drop(shared);
        cleanup(&config);
    }
}

#[cfg(test)]
#[path = "native_delegation_permissions149_tests.rs"]
mod native_delegation_permissions149_tests;
