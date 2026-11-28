//! Façades d'organisation ; mêmes principaux natifs que les autres contrôles humains.
use super::*;
use bridget_transport::agent_tracking::AgentTrackingRequest;
use bridget_transport::poison::RecoverPoison;
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

pub(super) fn bind_live(
    st: &mut DaemonState,
    agent_id: &str,
    thread: &str,
) -> Result<(), &'static str> {
    let agent = st
        .agent_infos()
        .into_iter()
        .find(|a| a.agent_id == agent_id)
        .ok_or("live_metadata_unavailable")?;
    let communication_project = agent_communication_project(st, agent_id);
    let snapshot = ScopedAgentInfo {
        agent,
        communication_project,
        project_relation: ProjectRelation::Unknown,
        tracking: None,
    };
    crate::agent_tracking::bind(st.store.connection(), &snapshot, thread)
        .map_err(|_| "tracking_storage_unavailable")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CaptureOutcome {
    NotCaptured,
    Captured,
    RetainedWithoutWakeup {
        category: &'static str,
        reason: String,
    },
}

fn retained_classic_reply_notice(
    correlation: &str,
    recipient: &str,
    request_state: &str,
    dormant: bool,
) -> (&'static str, String) {
    match (dormant, request_state) {
        (true, "open") => (
            "recipient_dormant",
            format!(
                "Réponse conservée dans le ledger sans réveil : le destinataire ({recipient}) est dormant (tracking.dormant=true), aucun nouveau tour n'a été lancé chez lui. Son utilisateur doit réveiller sa fiche depuis son interface ou avec bridget agents wake {recipient} pour reprendre les échanges en direct ; la réponse reste consultable dans le ledger. Cet envoi est clos ; les autres travaux autorisés peuvent continuer."
            ),
        ),
        (true, closed_state) => (
            "recipient_dormant_and_request_closed",
            format!(
                "Réponse conservée dans le ledger sans réveil : la demande {correlation} est déjà close ({closed_state}) et le destinataire ({recipient}) est dormant (tracking.dormant=true). Aucun nouveau tour n'a été lancé chez lui. Son utilisateur doit réveiller sa fiche depuis son interface ou avec bridget agents wake {recipient}, puis un nouveau message sans in_reply_to pourra le solliciter en direct. Cet envoi est clos ; les autres travaux autorisés peuvent continuer."
            ),
        ),
        (false, closed_state) => (
            "request_closed",
            format!(
                "Réponse conservée dans le ledger sans réveil : la demande {correlation} est déjà close ({closed_state}), aucun nouveau tour n'a été lancé chez {recipient}. Pour solliciter cet agent en direct, envoyez un nouveau message distinct sans in_reply_to. Cet envoi est clos ; les autres travaux autorisés peuvent continuer."
            ),
        ),
    }
}

/// Une réponse à une demande close ou à un destinataire dormant reste
/// consultable sans réveil. Une clôture acquise ne change jamais d’état.
pub(super) fn retain_classic_reply(
    st: &mut DaemonState,
    conn: &str,
    message: &bridget_core::BridgetMessage,
) -> Result<CaptureOutcome, String> {
    use rusqlite::{OptionalExtension, params};
    if !sender_is_authorized(st, conn, &message.from) {
        return Ok(CaptureOutcome::NotCaptured);
    }
    let Some(correlation) = message.in_reply_to.as_deref() else {
        return Ok(CaptureOutcome::NotCaptured);
    };
    let dormant = crate::agent_tracking::dormant(st.store.connection(), &message.to)
        .map_err(|e| e.to_string())?;
    let tx = st
        .store
        .connection()
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    let request: Option<(String, Option<String>)> = tx.query_row(
        "SELECT state,reply_summary FROM tracked_requests WHERE id=?1 AND sender=?2 AND target=?3",
        params![correlation,message.to,message.from], |r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    let Some((state, _summary)) = request else {
        return Ok(CaptureOutcome::NotCaptured);
    };
    if state == "open" && !dormant {
        return Ok(CaptureOutcome::NotCaptured);
    }
    if state != "open"
        && crate::communication::project_scope_typed(
            connection_communication_project(st, conn).as_ref(),
            agent_communication_project(st, &message.to).as_ref(),
            &message.to,
            message.cross_project_reason.as_deref(),
        )
        .is_err()
    {
        return Ok(CaptureOutcome::NotCaptured);
    }
    let existing: Option<(String, String)> = tx
        .query_row(
            "SELECT sender,body FROM ledger WHERE id=?1 AND target=?2",
            params![message.id, message.to],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if existing
        .as_ref()
        .is_some_and(|(sender, body)| sender != &message.from || body != &message.body)
    {
        return Err("retained_reply_conflict: Identifiant déjà utilisé par une autre réponse. Conservez les deux enveloppes et relisez le journal ; aucune nouvelle sollicitation n'a été remise.".into());
    }
    if existing.is_none() {
        crate::store::record_message_in_transaction(
            &tx,
            message,
            &format!("{}|{}", message.from, message.to),
        )
        .map_err(|e| e.to_string())?;
    }
    let (category, reason) =
        retained_classic_reply_notice(correlation, &message.to, &state, dormant);
    if state != "open" {
        tx.commit().map_err(|e| e.to_string())?;
        return Ok(CaptureOutcome::RetainedWithoutWakeup { category, reason });
    }
    if !crate::store::mark_answered_in_transaction(
        &tx,
        correlation,
        &message.from,
        &message.to,
        Some(&crate::store::request_reply_summary(message)),
    )
    .map_err(|e| e.to_string())?
    {
        return Ok(CaptureOutcome::NotCaptured);
    }
    tx.commit().map_err(|e| e.to_string())?;
    st.pending_replies
        .retain(|pending| pending.msg_id != correlation);
    st.delegation_store.wait_epoch().bump();
    Ok(CaptureOutcome::RetainedWithoutWakeup { category, reason })
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
            if agent.agent.transport == "t3code" {
                let thread = st
                    .router
                    .get_agent(agent_id)
                    .and_then(|r| st.t3_thread_bindings.get(&r.connection_id))
                    .and_then(Clone::clone);
                if let Some(thread) = thread {
                    crate::agent_tracking::bind(st.store.connection(), &agent, &thread)
                        .map_err(|_| "storage_unavailable")?;
                }
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
    let mut st = state.lock().recover_poison();
    let allowed = st.connection_roles.get(conn) == Some(&ConnectionRole::Client)
        && !st.auxiliary_connections.contains(conn)
        && !st.conn_names.contains_key(conn)
        && st.peer_uids.get(conn).copied() == Some(bridget_transport::fsutil::effective_uid())
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
    let mut st = state.lock().recover_poison();
    let result = apply(
        &mut st,
        &format!("mobile:{}", device.device_id),
        request,
        Some(device),
    )
    .unwrap_or_else(failure);
    DaemonToWrapper::MobileOperationResult { result }
}
