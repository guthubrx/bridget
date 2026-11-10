//! Administration mobile locale : rôle, capacité et UID avant tout accès privé.
use super::*;
use bridget_transport::protocol::{MobileAdministrationRequest, MobileAdministrationResult};

pub(super) fn project_update_status(value: serde_json::Value, state: &Arc<Mutex<DaemonState>>) -> serde_json::Value {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let mut machines = std::collections::BTreeSet::new();
    machines.insert(st.host.clone());
    machines.extend(st.registry.machines().keys().cloned());
    machines.extend(st.remote_workers.keys().cloned());
    crate::updates::project_federation(value, &machines, unix_now_secs())
}

pub(super) fn handle(
    conn_id: &str,
    request: MobileAdministrationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let root = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let valid = st.connection_roles.get(conn_id) == Some(&ConnectionRole::Client)
            && !st.auxiliary_connections.contains(conn_id)
            && !st.conn_names.contains_key(conn_id)
            && st.peer_uids.get(conn_id).copied() == Some(unsafe { libc::geteuid() })
            && st.client_negotiations.get(conn_id).is_some_and(|n| {
                n.version == CLIENT_CONTRACT_VERSION
                    && n.capabilities.as_slice() == [ClientCapability::MobileAdministrationV1]
            });
        if !valid {
            return DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Error {
                    code: "human_principal_required".into(),
                },
            };
        }
        let Some(root) = st.db_path.parent().map(PathBuf::from) else {
            return DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Error {
                    code: "storage_unavailable".into(),
                },
            };
        };
        root
    };
    if let MobileAdministrationRequest::Updates { request } = request {
        use bridget_transport::protocol::UpdateRequest as Q;
        let result = (|| -> Result<serde_json::Value, &'static str> {
            match &request {
                Q::Prepare { token } => {
                    // L'état et le compteur sont observés sous le même verrou métier.
                    let st = state.lock().unwrap_or_else(|e| e.into_inner());
                    if !st
                        .delegation_store
                        .pending()
                        .map_err(|_| "update_activity_unknown")?
                        .is_empty()
                        || st
                            .delegation_store
                            .update_has_pending_admissions()
                            .map_err(|_| "update_activity_unknown")?
                        || st.fleet.active_count() != 0
                        || st.recovering
                        || !st.managed_spawns.is_empty()
                        || st.presences.values().any(|p| p.state == "busy")
                        || st.remote_result_reservation.is_some()
                        || !st.pending_replies.is_empty()
                        || !st.pending_post_response_controls.is_empty()
                        || !st.pending_worker_responses.is_empty()
                        || !st.pending_origin_proofs.is_empty()
                        || !st.pending_candidate_probes_v2.is_empty()
                        || !st.pending_attach_sends.is_empty()
                        || st.active_waits.load(Ordering::Acquire)!=0
                        || crate::user_input::has_active(st.store.connection(),unix_now_secs()).map_err(|_|"update_activity_unknown")?
                    {
                        return Err("update_missions_active_or_unconfirmed");
                    }
                    st.update_gate.prepare(token)
                }
                Q::Finish { token } => state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .update_gate
                    .finish(token),
                _ => crate::updates::policy(&root, &request).map(|v| project_update_status(v, state)),
            }
        })();
        return DaemonToWrapper::MobileAdministrationResult {
            result: match result {
                Ok(value) => MobileAdministrationResult::Updates { value },
                Err(code) => MobileAdministrationResult::Error { code: code.into() },
            },
        };
    }
    if let MobileAdministrationRequest::GrantUpdates { device_id } = request {
        let result = crate::updates::grant_device(&root, &device_id);
        return DaemonToWrapper::MobileAdministrationResult {
            result: match result {
                Ok(found) => MobileAdministrationResult::Updates {
                    value: serde_json::json!({"status":"ok","granted":found,"device_id":device_id}),
                },
                Err(e) => MobileAdministrationResult::Error {
                    code: e.into(),
                },
            },
        };
    }
    // Aucun appel réseau ni écriture du magasin mobile sous le mutex métier.
    let reload = matches!(
        &request,
        MobileAdministrationRequest::ConfigureNotifications { .. }
    );
    let result = crate::mobile::administer(&root, request, unix_now_secs());
    if reload
        && matches!(
            &result,
            MobileAdministrationResult::NotificationsConfigured { configured: true }
        )
    {
        let notifier = crate::mobile::notifications::Notifier::start(&root);
        state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .mobile_notifier = notifier;
    }
    DaemonToWrapper::MobileAdministrationResult { result }
}

#[cfg(test)]
#[path = "mobile_admin_tests.rs"]
mod tests;
