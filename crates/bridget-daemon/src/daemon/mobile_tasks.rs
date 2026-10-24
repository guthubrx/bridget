//! Projections et annulation humaines des tâches ; le magasin natif reste l'autorité.
use super::*;
use crate::mobile::store::{Device, Operation, Store};
use bridget_transport::protocol::{
    HumanLineageError as E, MobileOperation, MobileOperationRequest,
};
use serde_json::{Value, json};
use std::path::Path;

fn failure(code: &str) -> Value {
    json!({"status":"refused","code":code})
}
fn uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|v| v.get_version_num() == 4 && v.to_string() == id)
}
fn cancel_key(device: &str, id: &str) -> String {
    format!("mobile177:cancel:{device}:{id}")
}
fn native_error(error: E) -> Value {
    let code = match error {
        E::SnapshotChanged => "resync_required",
        E::TaskUnavailable => "not_found",
        E::EnvelopeMismatch => "envelope_mismatch",
        E::StoreUnavailable => "storage_unavailable",
        E::ResourceLimit => "resource_limit",
        _ => "invalid_request",
    };
    failure(code)
}
fn decorate(entry: &mut Value, st: &DaemonState, device: &Device) {
    if !entry["error"].is_null() {
        entry["error"] = json!("Le moteur signale un blocage ; consultez le journal de la tâche.");
    }
    let parent = entry["parent_agent_id"].as_str().unwrap_or("");
    let instruction = st
        .router
        .get_agent(parent)
        .and_then(|route| live_connection_identity(st, &route.connection_id))
        .and_then(|(_, instance)| st.presences.get(&instance))
        .is_some_and(|presence| {
            device.scope.permits(
                Operation::Message,
                &presence.host,
                agent_communication_project(st, parent).as_ref(),
            )
        })
        && device.scope.operations.contains(&Operation::TaskContinue);
    entry["actions"] = json!({"cancel_native":device.scope.operations.contains(&Operation::TaskCancel),
        "request_pause":instruction,"request_continue":instruction,"direct_continue":false,"direct_pause":false});
    entry["action_semantics"] =
        json!("Une instruction au parent est une demande ; seul l’état natif confirme son effet.");
}

pub(super) fn handle(
    request: &MobileOperationRequest,
    root: &Path,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let mut drive = false;
    let result = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let device = match Store::open(&root.join("mobile-devices.db")).and_then(|s| {
            s.authenticate(request.credential.expose_for_attestation(), unix_now_secs())
        }) {
            Ok(d) => d,
            Err(e) => {
                return DaemonToWrapper::MobileOperationResult {
                    result: failure(e.code()),
                };
            }
        };
        let op = match request.operation {
            MobileOperation::TaskCancel { .. } | MobileOperation::TaskCancelReceipt { .. } => {
                Operation::TaskCancel
            }
            _ => Operation::TaskRead,
        };
        if !device.scope.permits_federation(op) {
            return DaemonToWrapper::MobileOperationResult {
                result: failure("forbidden_scope"),
            };
        }
        match &request.operation {
            MobileOperation::TaskList {
                agent_id,
                after,
                limit,
            } => {
                if !uuid(agent_id)
                    || !(1..=50).contains(limit)
                    || after.as_ref().is_some_and(|s| s.len() > 2048)
                {
                    failure("invalid_request")
                } else {
                    let owner = match st.delegation_store.for_child(agent_id) {
                        Ok(Some(task)) => task.root_owner().to_owned(),
                        Ok(None) => agent_id.clone(),
                        Err(_) => {
                            return DaemonToWrapper::MobileOperationResult {
                                result: failure("storage_unavailable"),
                            };
                        }
                    };
                    match st.delegation_store.lineage_page_versioned(
                        &owner,
                        *limit,
                        after.as_deref(),
                        4,
                        |t| {
                            (
                                native_lineage::available(&st, t),
                                native_delegation::remote_lineage_placement(&st, t),
                            )
                        },
                    ) {
                        Ok(mut page) => {
                            if let Some(entries) =
                                page.get_mut("tasks").and_then(Value::as_array_mut)
                            {
                                for entry in entries {
                                    decorate(entry, &st, &device);
                                }
                            }
                            page
                        }
                        Err(e) => native_error(e),
                    }
                }
            }
            MobileOperation::TaskShow { task_id, offset } => {
                if !uuid(task_id) {
                    failure("invalid_request")
                } else {
                    match st.delegation_store.get(task_id) {
                        Ok(Some(task)) => match st.delegation_store.lineage_show_versioned(
                            task.root_owner(),
                            task_id,
                            *offset,
                            16384,
                            4,
                            |t| {
                                (
                                    native_lineage::available(&st, t),
                                    native_delegation::remote_lineage_placement(&st, t),
                                )
                            },
                        ) {
                            Ok(mut page) => {
                                decorate(&mut page["task"], &st, &device);
                                page
                            }
                            Err(e) => native_error(e),
                        },
                        Ok(None) => failure("not_found"),
                        Err(_) => failure("storage_unavailable"),
                    }
                }
            }
            MobileOperation::TaskCancel {
                device_id,
                command_id,
                task_id,
            }
            | MobileOperation::TaskCancelReceipt {
                device_id,
                command_id,
                task_id,
            } => {
                if device_id != &device.device_id {
                    failure("command_origin_changed")
                } else if !uuid(command_id) || !uuid(task_id) {
                    failure("invalid_request")
                } else {
                    let key = cancel_key(&device.device_id, command_id);
                    match st.delegation_store.get(task_id) {
                        Ok(Some(task)) => {
                            let prior = st.delegation_store.human_cancel_scoped_receipt(
                                task.root_owner(),
                                task_id,
                                &key,
                            );
                            let outcome = match prior {
                                Ok(Some(receipt)) => Ok(receipt),
                                Ok(None)
                                    if matches!(
                                        request.operation,
                                        MobileOperation::TaskCancelReceipt { .. }
                                    ) =>
                                {
                                    Err(E::TaskUnavailable)
                                }
                                Ok(None) => st
                                    .delegation_store
                                    .human_cancel(task.root_owner(), task_id, &key)
                                    .map(|(receipt, run)| {
                                        drive = run;
                                        receipt
                                    }),
                                Err(e) => Err(e),
                            };
                            match outcome {
                                Ok(receipt) => {
                                    json!({"status":"ok","operation_kind":"task_cancel","command_id":command_id,
                                    "issue":{"kind":"accepted"},"receipt":receipt,
                                    "message":"Demande enregistrée ; relisez la tâche pour confirmer son arrêt."})
                                }
                                Err(e) => native_error(e),
                            }
                        }
                        Ok(None) => failure("not_found"),
                        Err(_) => failure("storage_unavailable"),
                    }
                }
            }
            _ => failure("invalid_request"),
        }
    };
    if drive {
        let state = state.clone();
        thread::spawn(move || native_delegation::tick(&state));
    }
    // Une révocation pendant les lectures n'expose aucun contenu nouveau.
    if let Err(e) = Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        return DaemonToWrapper::MobileOperationResult {
            result: failure(e.code()),
        };
    }
    DaemonToWrapper::MobileOperationResult { result }
}
