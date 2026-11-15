//! Administration mobile locale : rôle, capacité et UID avant tout accès privé.
use super::*;
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::{MobileAdministrationRequest, MobileAdministrationResult};

pub(super) fn project_update_status(
    value: serde_json::Value,
    state: &Arc<Mutex<DaemonState>>,
) -> serde_json::Value {
    let st = state.lock().recover_poison();
    let mut machines = std::collections::BTreeSet::new();
    machines.insert(st.host.clone());
    machines.extend(st.registry.machines().keys().cloned());
    machines.extend(st.remote_workers.keys().cloned());
    crate::updates::project_federation(value, &machines, unix_now_secs())
}

/// Même observation pour le refus de préparation et son diagnostic local.
/// Lecture des tâches déjà requise par le garde. Admissions : O(a log a) pour
/// le tri du lot. Seize détails par catégorie ; la réponse complète est mesurée.
fn activation_blockers(st: &DaemonState) -> Result<serde_json::Value, &'static str> {
    use serde_json::json;
    let tasks = st
        .delegation_store
        .pending()
        .map_err(|_| "update_activity_unknown")?;
    let mut blockers = Vec::new();
    if !tasks.is_empty() {
        blockers.push(json!({
            "code":"native_tasks_pending", "count":tasks.len(),
            "tasks":tasks.iter().take(16).map(|task| json!({
                "task_id":task.task_id, "owner":task.owner, "state":task.state,
                "cleanup_done":task.cleanup_done,
                "guard_open":task.continuation_guard_open(),
                "next_action":if task.terminal() {
                    "Vérifier l'arrêt physique et la conservation sur le worker de cette tâche. Une annulation logique seule ne clôt pas ces preuves."
                } else {
                    "Lire la tâche avec son propriétaire. Préserver une mission active ; demander une annulation native seulement si son arrêt est autorisé."
                }
            })).collect::<Vec<_>>(),
            "truncated":tasks.len()>16
        }));
    }
    // Aucun canon, message, chemin ni métadonnée de permission n'est projeté.
    let admissions = (|| -> rusqlite::Result<Vec<serde_json::Value>> {
        let mut query = st.store.connection().prepare(
            "SELECT owner,request_id,state,count(*) OVER () FROM auto_admission_receipts WHERE state IN ('pending','decision_started','chosen') ORDER BY updated_at,owner,request_id LIMIT 16"
        )?;
        query.query_map([], |row| Ok(json!({
            "owner":row.get::<_,String>(0)?, "request_id":row.get::<_,String>(1)?,
            "state":row.get::<_,String>(2)?, "total":row.get::<_,u64>(3)?
        })))?.collect()
    })().map_err(|_| "update_activity_unknown")?;
    if let Some(first) = admissions.first() {
        blockers.push(json!({
            "code":"admissions_pending", "count":first["total"],
            "truncated":first["total"].as_u64().unwrap_or(0)>16,
            "admissions":admissions,
            "next_action":"Le propriétaire relit la même clé. Si l'abandon est autorisé, utiliser bridget_cancel avec kind=admission et id=request_id. Si une tâche existe, suivre son task_id ; ne pas créer une nouvelle clé."
        }));
    }
    for (code, count) in [
        ("fleet_active", st.fleet.active_count()),
        ("coordinator_recovering", usize::from(st.recovering)),
        ("managed_spawns", st.managed_spawns.len()),
        (
            "busy_presences",
            st.presences.values().filter(|p| p.state == "busy").count(),
        ),
        (
            "remote_result_reservation",
            usize::from(st.remote_result_reservation.is_some()),
        ),
        ("pending_replies", st.pending_replies.len()),
        (
            "pending_post_response_controls",
            st.pending_post_response_controls.len(),
        ),
        (
            "pending_worker_responses",
            st.pending_worker_responses.len(),
        ),
        ("pending_origin_proofs", st.pending_origin_proofs.len()),
        (
            "pending_candidate_probes",
            st.pending_candidate_probes_v2.len(),
        ),
        ("pending_attach_sends", st.pending_attach_sends.len()),
        ("active_waits", st.active_waits.load(Ordering::Acquire)),
        (
            "active_user_input",
            usize::from(
                crate::user_input::has_active(st.store.connection(), unix_now_secs())
                    .map_err(|_| "update_activity_unknown")?,
            ),
        ),
    ] {
        if count != 0 {
            blockers.push(json!({"code":code,"count":count}));
        }
    }
    Ok(json!({
        "status":if blockers.is_empty() {"idle_observed"} else {"blocked"},
        "scope":"coordinator", "observed_at":unix_now_secs(), "blockers":blockers,
        "next_action":"Traiter les éléments indiqués par les parcours natifs, puis refaire le contrôle. Cette lecture ne réserve pas l'activation. Ne pas effacer les preuves ni forcer la base. Les autres travaux suivent leur propre état."
    }))
}

pub(super) fn handle(
    conn_id: &str,
    request: MobileAdministrationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let root = {
        let st = state.lock().recover_poison();
        let valid = st.connection_roles.get(conn_id) == Some(&ConnectionRole::Client)
            && !st.auxiliary_connections.contains(conn_id)
            && !st.conn_names.contains_key(conn_id)
            && st.peer_uids.get(conn_id).copied()
                == Some(bridget_transport::fsutil::effective_uid())
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
                    let st = state.lock().recover_poison();
                    if activation_blockers(&st)?["status"] != "idle_observed" {
                        return Err("update_missions_active_or_unconfirmed");
                    }
                    st.update_gate.prepare(token)
                }
                Q::Status => {
                    let mut value =
                        project_update_status(crate::updates::policy(&root, &request)?, state);
                    // Détails réservés au port d'administration locale authentifié.
                    // La projection mobile fédérée ne reçoit pas ces identifiants.
                    value["activation"] = match activation_blockers(&state.lock().recover_poison())
                    {
                        Ok(activity) => activity,
                        Err(code) => serde_json::json!({"status":"unknown", "code":code,
                            "next_action":"Faire vérifier la lecture des états natifs. Aucune activation ni clôture n'est attestée."}),
                    };
                    // Le client update accepte au plus 64 Kio, saut de ligne compris.
                    if serde_json::to_vec(&value)
                        .map_err(|_| "update_state_unavailable")?
                        .len()
                        >= 64 * 1024
                    {
                        return Err("update_status_too_large");
                    }
                    Ok(value)
                }
                Q::Finish { token } => state.lock().recover_poison().update_gate.finish(token),
                _ => {
                    crate::updates::policy(&root, &request).map(|v| project_update_status(v, state))
                }
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
                Err(e) => MobileAdministrationResult::Error { code: e.into() },
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
        state.lock().recover_poison().mobile_notifier = notifier;
    }
    DaemonToWrapper::MobileAdministrationResult { result }
}

#[cfg(test)]
#[path = "mobile_admin_tests.rs"]
mod tests;
