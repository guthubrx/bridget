//! Façades d'organisation ; mêmes principaux natifs que les autres contrôles humains.
use super::*;
use bridget_transport::agent_tracking::AgentTrackingRequest;
use bridget_transport::protocol::{ProjectRelation, ScopedAgentInfo};
use serde_json::{Value, json};

pub(super) fn failure(code: &str) -> Value {
    json!({"status":"refused","code":code,"message":match code {
        "agent_not_found"=>"La fiche n'est plus connue. Actualisez l'annuaire et choisissez une fiche existante.",
        "revision_conflict"=>"La fiche a changé sur un autre appareil. Actualisez-la avant de confirmer votre choix.",
        "envelope_mismatch"=>"Cet identifiant appartient à une autre action. Conservez son reçu.",
        "human_principal_required"=>"Modifiez ce choix depuis votre terminal humain ou un appareil appairé.",
        "forbidden_scope"=>"Cet appareil n'a pas accès à l'organisation de cette fiche dans sa portée actuelle.",
        _=>"L'action sur cette fiche n'est pas confirmée. Conservez son identifiant et relisez l'état avant toute nouvelle action. Les autres travaux autorisés peuvent continuer."
    }})
}

pub(super) fn bind_live(st:&mut DaemonState,agent_id:&str,thread:&str)->Result<(),&'static str> {
    let agent=st.agent_infos().into_iter().find(|a|a.agent_id==agent_id).ok_or("live_metadata_unavailable")?;
    let communication_project=agent_communication_project(st,agent_id);
    let snapshot=ScopedAgentInfo{agent,communication_project,project_relation:ProjectRelation::Unknown,tracking:None};
    crate::agent_tracking::bind(st.store.connection(),&snapshot,thread).map_err(|_|"tracking_storage_unavailable")
}

pub(super) fn directory(st: &mut DaemonState) -> Result<Vec<ScopedAgentInfo>, &'static str> {
    let mut agents = crate::agent_tracking::retained(st.store.connection())
        .map_err(|_| "storage_unavailable")?;
    for agent in st.agent_infos() {
        let id = agent.agent_id.clone();
        let communication_project = agent_communication_project(st, &id);
        let tracking = crate::agent_tracking::view(st.store.connection(), &id)
            .map_err(|_| "storage_unavailable")?;
        agents.insert(
            id,
            ScopedAgentInfo {
                agent,
                communication_project,
                project_relation: ProjectRelation::Unknown,
                tracking,
            },
        );
    }
    Ok(agents.into_values().collect())
}

fn apply(
    st: &mut DaemonState,
    actor: &str,
    request: &AgentTrackingRequest,
    device: Option<&crate::mobile::store::Device>,
) -> Result<Value, &'static str> {
    match request {
        AgentTrackingRequest::Set {
            request_id,
            agent_id,
            dormant,
            synchronize,
            expected_revision,
        } => {
            let agent = directory(st)?
                .into_iter()
                .find(|a| &a.agent.agent_id == agent_id)
                .ok_or("agent_not_found")?;
            if let Some(device) = device {
                use crate::mobile::store::Operation;
                if !device.scope.permits(
                    Operation::Message,
                    &agent.agent.host,
                    agent.communication_project.as_ref(),
                ) || !device.scope.permits(
                    Operation::Read,
                    &agent.agent.host,
                    agent.communication_project.as_ref(),
                ) {
                    return Err("forbidden_scope");
                }
            }
            if agent.agent.transport=="t3code" {
                let thread=st.router.get_agent(agent_id).and_then(|r|st.t3_thread_bindings.get(&r.connection_id)).and_then(Clone::clone);
                if let Some(thread)=thread {crate::agent_tracking::bind(st.store.connection(),&agent,&thread).map_err(|_|"storage_unavailable")?;}
            }
            let canonical = serde_json::to_vec(request).map_err(|_| "invalid_request")?;
            crate::agent_tracking::set(
                st.store.connection(),
                actor,
                request_id,
                &canonical,
                &agent,
                *dormant,
                *synchronize,
                *expected_revision,
                unix_now_secs(),
            )
        }
        AgentTrackingRequest::Sync {
            source_host,
            observations,
            confirmations,
        } => {
            if device.is_some() {
                return Err("forbidden_scope");
            }
            crate::agent_tracking::sync(
                st.store.connection(),
                source_host,
                observations,
                confirmations,
                unix_now_secs(),
            )
        }
    }
}

pub(super) fn human(
    conn: &str,
    request: &AgentTrackingRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    let allowed = st.connection_roles.get(conn) == Some(&ConnectionRole::Client)
        && !st.auxiliary_connections.contains(conn)
        && !st.conn_names.contains_key(conn)
        && st.peer_uids.get(conn).copied() == Some(unsafe { libc::geteuid() })
        && control_client_actor(&st, conn).is_some()
        && st.client_negotiations.get(conn).is_some_and(|n| {
            n.version == CLIENT_CONTRACT_VERSION
                && n.capabilities.contains(&ClientCapability::AgentTrackingV1)
        });
    let result = if allowed {
        let actor = st.client_negotiations[conn].issuer_scope.clone();
        apply(&mut st, &actor, request, None).unwrap_or_else(failure)
    } else {
        failure("human_principal_required")
    };
    DaemonToWrapper::AgentTrackingResult { result }
}

pub(super) fn mobile(
    request: &AgentTrackingRequest,
    device: &crate::mobile::store::Device,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    let result = apply(
        &mut st,
        &format!("mobile:{}", device.device_id),
        request,
        Some(device),
    )
    .unwrap_or_else(failure);
    DaemonToWrapper::MobileOperationResult { result }
}
