//! Vue humaine froide de la saga native. Aucun moteur de présentation parallèle.
use super::*;
use crate::delegation::ProjectionMutation;
use bridget_transport::protocol::{
    HumanLineageAction as A, HumanLineageError as E, HumanLineageRequest as Request,
    HumanLineageWatchEvent as Event,
};
use serde_json::Value;
use bridget_transport::protocol::{HumanComposeOperation as Op, HumanComposeSendIntent as Intent};

#[derive(Clone)]
pub(super) struct Context {
    request: Request,
    project: Option<bridget_transport::protocol::CommunicationProject>,
    owner: String,
    root: String,
}

/// Preuve privée créée seulement après la garde humaine et la liaison de tâche.
pub(super) struct ComposeAuthority {
    context: Context,
    task: crate::delegation::Task,
}
impl ComposeAuthority {
    pub(super) fn issuer_scope(&self) -> String {
        crate::communication::issuer_scope(&format!("human-compose:{}", self.context.root))
    }
}

fn compose_live_task(
    st: &DaemonState,
    conn: &str,
    context: &Context,
    expected: &crate::delegation::Task,
) -> Result<(), E> {
    guard(st, conn, context)?;
    let task = st
        .delegation_store
        .get(&expected.task_id)
        .map_err(|_| E::StoreUnavailable)?
        .filter(|t| t.root_owner() == context.root)
        .ok_or(E::TaskUnavailable)?;
    if task.round != expected.round
        || task.child != expected.child
        || task.child_instance != expected.child_instance
        || task.remote_placement != expected.remote_placement
        || task.owner != expected.owner
        || task.owner_instance != expected.owner_instance
        || task.origin_owner_instance != expected.origin_owner_instance
    {
        return Err(E::SnapshotChanged);
    }
    if !matches!(task.state.as_str(), "working" | "waiting_for_children") || task.guard_released {
        return Err(E::TaskUnavailable);
    }
    for agent in [&task.owner, &task.child, &context.root] {
        if st
            .delegation_store
            .revoked_agent(agent)
            .map_err(|_| E::StoreUnavailable)?
        {
            return Err(E::BindingUnavailable);
        }
    }
    for instance in [
        Some(task.owner_instance.as_str()),
        Some(task.origin_owner_instance.as_str()),
        task.child_instance.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if st
            .delegation_store
            .revoked(instance)
            .map_err(|_| E::StoreUnavailable)?
        {
            return Err(E::BindingUnavailable);
        }
    }
    let route = st.router.get_agent(&task.child).ok_or(E::TaskUnavailable)?;
    let identity = live_connection_identity(st, &route.connection_id).ok_or(E::TaskUnavailable)?;
    if identity.0 != task.child
        || Some(identity.1.as_str()) != task.child_instance.as_deref()
        || !st.connections.contains_key(&route.connection_id)
    {
        return Err(E::TaskUnavailable);
    }
    if task.remote_placement.is_some() {
        native_delegation::remote_command_identity(st, &task).map_err(|_| E::TaskUnavailable)?;
        let placement =
            native_delegation::remote_lineage_placement(st, &task).ok_or(E::TaskUnavailable)?;
        let worker = st
            .remote_workers
            .get(&placement.machine_id)
            .ok_or(E::TaskUnavailable)?;
        let expected_worker = task.remote_placement.as_ref().ok_or(E::TaskUnavailable)?;
        if placement.contact_state != bridget_transport::protocol::WorkerContactState::Connected
            || worker.identity.worker_id != expected_worker.worker_id
            || worker.identity.provisioning_revision != expected_worker.provisioning_revision
            || worker.last_contact.elapsed()
                >= Duration::from_secs(bridget_transport::protocol::WORKER_STALE_SECS as u64)
            || !st.connections.contains_key(&worker.conn_id)
        {
            return Err(E::TaskUnavailable);
        }
    }
    Ok(())
}

pub(super) fn authorize_compose_send(
    st: &DaemonState,
    conn: &str,
    message: &bridget_core::BridgetMessage,
    proof: &ComposeAuthority,
) -> Result<(), String> {
    compose_live_task(st, conn, &proof.context, &proof.task)
        .map_err(|e| format!("human_compose_{e:?}"))?;
    let A::Compose {
        request_id,
        operation: Op::Send { body, intent },
        ..
    } = &proof.context.request.request
    else {
        return Err("human_compose_operation_mismatch".into());
    };
    let wanted = match intent {
        Intent::Queue => bridget_core::MessageIntent::QueueOnly,
        Intent::Steer => bridget_core::MessageIntent::SteerCurrent,
    };
    if message.id != *request_id
        || message.from != "human"
        || message.to != proof.task.child
        || message.body != *body
        || message.intent != Some(wanted)
        || message.reply
        || message.in_reply_to.is_some()
    {
        return Err("human_compose_envelope_mismatch".into());
    }
    Ok(())
}

pub(super) fn authorize_compose_control(
    st: &DaemonState,
    conn: &str,
    command: &ExecutionControlCommand,
    proof: &ComposeAuthority,
) -> Result<(), String> {
    compose_live_task(st, conn, &proof.context, &proof.task)
        .map_err(|e| format!("human_compose_{e:?}"))?;
    let A::Compose {
        request_id,
        operation,
        ..
    } = &proof.context.request.request
    else {
        return Err("human_compose_operation_mismatch".into());
    };
    let operation_ok = match operation {
        Op::Interrupt {} => {
            command.operation == ExecutionControlOperation::Interrupt && command.message.is_none()
        }
        Op::Send {
            intent: Intent::Steer,
            ..
        } => {
            command.operation == ExecutionControlOperation::SteerCurrent
                && command
                    .message
                    .as_ref()
                    .is_some_and(|message| authorize_compose_send(st, conn, message, proof).is_ok())
        }
        _ => false,
    };
    let snapshot = st
        .execution_store
        .latest_in_chain(&format!("execution-{}", proof.task.mission))
        .map_err(|_| "human_compose_execution_unavailable")?
        .ok_or("human_compose_execution_unavailable")?;
    let target = st
        .execution_store
        .execution_control_target(&snapshot.execution_id)
        .map_err(|_| "human_compose_execution_unavailable")?
        .ok_or("human_compose_execution_unavailable")?;
    if command.command_id != *request_id
        || command.execution_id != snapshot.execution_id
        || command.generation != snapshot.generation
        || command.revision != snapshot.revision
        || !operation_ok
        || target.target_agent != proof.task.child
    {
        return Err("human_compose_execution_mismatch".into());
    }
    Ok(())
}

pub(super) struct Watch {
    context: Context,
    pending: VecDeque<Event>,
    generation: String,
    seq: u64,
    terminal: bool,
    notify: mpsc::SyncSender<()>,
}

pub(super) struct Journal {
    context: Context,
    task: crate::delegation::Task,
    command: Option<bridget_transport::protocol::WorkerCommandIdentity>,
    pub shutdown: Option<UnixStream>,
    notify: mpsc::SyncSender<()>,
}

fn from_guard(e: bridget_transport::protocol::HumanThreadViewError) -> E {
    use bridget_transport::protocol::HumanThreadViewError as H;
    match e {
        H::UnsupportedVersion => E::UnsupportedVersion,
        H::InvalidRequest => E::InvalidRequest,
        H::ProjectMismatch => E::ProjectMismatch,
        H::StorageUnavailable => E::StoreUnavailable,
        _ => E::BindingUnavailable,
    }
}

fn valid_uuid(s: &str) -> bool {
    crate::threads::canonical_uuid(s).as_deref() == Some(s)
}

fn validate(request: &Request) -> Result<(), E> {
    request.validate_response_version()?;
    if request.t3_thread_id.is_empty()
        || request.t3_thread_id.len() > 2048
        || request
            .t3_thread_id
            .chars()
            .any(bridget_core::is_disallowed_control)
        || request.project_root.len() > 4096
        || !PathBuf::from(&request.project_root).is_absolute()
    {
        return Err(E::InvalidRequest);
    }
    match &request.request {
        A::List { limit, cursor }
            if !(1..=100).contains(limit)
                || cursor
                    .as_ref()
                    .is_some_and(|c| c.len() > 2048 || c.chars().any(char::is_control)) =>
        {
            Err(E::InvalidRequest)
        }
        A::Show { task_id, limit, .. } if !valid_uuid(task_id) || !(1..=16384).contains(limit) => {
            Err(E::InvalidRequest)
        }
        A::Journal {
            task_id,
            limit,
            after_seq,
            ..
        } if !valid_uuid(task_id)
            || !(1..=100).contains(limit)
            || *after_seq > bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ =>
        {
            Err(E::InvalidRequest)
        }
        A::Cancel {
            task_id,
            request_id,
        } if !valid_uuid(task_id) || !valid_uuid(request_id) => Err(E::InvalidRequest),
        A::Compose {
            task_id,
            request_id,
            operation,
            expected_round,
        } if !valid_uuid(task_id)
            || !valid_uuid(request_id)
            || !operation.bounded(*expected_round) =>
        {
            Err(E::InvalidRequest)
        }
        _ => Ok(()),
    }
}

fn authorize(state: &Arc<Mutex<DaemonState>>, conn: &str, request: &Request) -> Result<Context, E> {
    validate(request)?;
    {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        human_client_guard(
            &st,
            conn,
            1,
            &request.t3_thread_id,
            request.request.capability(),
        )
        .map_err(from_guard)?;
    }
    let project = human_request_project(state, &request.t3_thread_id, &request.project_root);
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    human_client_guard(
        &st,
        conn,
        1,
        &request.t3_thread_id,
        request.request.capability(),
    )
    .map_err(from_guard)?;
    let (owner, root) =
        human_authority_guard(&st, &request.t3_thread_id, &project).map_err(from_guard)?;
    Ok(Context {
        request: request.clone(),
        project,
        owner,
        root,
    })
}

fn guard(st: &DaemonState, conn: &str, context: &Context) -> Result<(), E> {
    human_client_guard(
        st,
        conn,
        1,
        &context.request.t3_thread_id,
        context.request.request.capability(),
    )
    .map_err(from_guard)?;
    let (owner, root) = human_authority_guard(st, &context.request.t3_thread_id, &context.project)
        .map_err(from_guard)?;
    if owner != context.owner || root != context.root {
        return Err(E::BindingUnavailable);
    }
    if let A::Journal { task_id, .. } = &context.request.request {
        st.delegation_store
            .get(task_id)
            .map_err(|_| E::StoreUnavailable)?
            .filter(|t| t.root_owner() == context.root)
            .ok_or(E::TaskUnavailable)?;
    }
    Ok(())
}

fn guard_journal_snapshot(
    st: &DaemonState,
    conn: &str,
    context: &Context,
    task: &crate::delegation::Task,
    command: Option<&bridget_transport::protocol::WorkerCommandIdentity>,
) -> Result<(), E> {
    guard(st, conn, context)?;
    let current = st
        .delegation_store
        .get(&task.task_id)
        .map_err(|_| E::StoreUnavailable)?
        .ok_or(E::TaskUnavailable)?;
    if current.child != task.child
        || current.child_instance != task.child_instance
        || current.remote_placement != task.remote_placement
    {
        return Err(E::JournalUnavailable);
    }
    if let Some(command) = command
        && &native_delegation::remote_command_identity(st, &current)
            .map_err(|_| E::JournalUnavailable)?
            != command
        {
            return Err(E::JournalUnavailable);
        }
    Ok(())
}

fn journal_directory(st: &DaemonState, child: &str) -> Result<PathBuf, E> {
    if !valid_uuid(child) {
        return Err(E::JournalUnavailable);
    }
    let root = st.db_path.parent().ok_or(E::JournalUnavailable)?;
    Ok(root.join("sessions").join(child))
}

pub(super) fn available(st: &DaemonState, task: &crate::delegation::Task) -> bool {
    if task.remote_placement.is_some() {
        return task.child_instance.is_some()
            && (st.router.get_agent(&task.child).is_some()
                || native_delegation::remote_lineage_placement(st, task).is_some_and(
                    |placement| {
                        placement.contact_state
                            == bridget_transport::protocol::WorkerContactState::Connected
                    },
                ));
    }
    journal_directory(st, &task.child)
        .ok()
        .is_some_and(|directory| directory.is_dir())
}

pub(super) fn handle(
    conn: &str,
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let context = match authorize(state, conn, request) {
        Ok(c) => c,
        Err(e) => return Some(refusal(request, e)),
    };
    if matches!(request.request, A::Watch) {
        return watch(conn, context, state);
    }
    if matches!(request.request, A::Journal { follow: true, .. }) {
        return journal_follow(conn, context, state);
    }
    if matches!(request.request, A::Journal { .. }) {
        return Some(DaemonToWrapper::HumanLineageResult {
            result: journal_page(conn, &context, state).unwrap_or_else(E::result),
        });
    }
    if matches!(request.request,A::Compose{..}) {
        return Some(DaemonToWrapper::HumanLineageResult {
            result:compose(conn,&context,state).unwrap_or_else(E::result),
        });
    }
    let mut drive = false;
    let result = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        guard(&st, conn, &context).and_then(|()| match &request.request {
            A::List { limit, cursor } => st.delegation_store.lineage_page_versioned(
                &context.root,
                *limit,
                cursor.as_deref(),
                request.version,
                |task| {
                    (
                        available(&st, task),
                        native_delegation::remote_lineage_placement(&st, task),
                    )
                },
            ),
            A::Show {
                task_id,
                offset,
                limit,
            } => st.delegation_store.lineage_show_versioned(
                &context.root,
                task_id,
                *offset,
                *limit,
                request.version,
                |task| {
                    (
                        available(&st, task),
                        native_delegation::remote_lineage_placement(&st, task),
                    )
                },
            ),
            A::Cancel {
                task_id,
                request_id,
            } => {
                let (receipt, run) =
                    st.delegation_store
                        .human_cancel(&context.root, task_id, request_id)?;
                drive = run;
                Ok(receipt)
            }
            A::Compose { .. } => unreachable!(),
            A::Journal { .. } => unreachable!(),
            A::Watch => unreachable!(),
        })
    };
    if drive {
        let state = Arc::clone(state);
        thread::spawn(move || native_delegation::tick(&state));
    }
    Some(DaemonToWrapper::HumanLineageResult {
        result: result.unwrap_or_else(E::result),
    })
}

fn compose_catalogue(
    conn: &str,
    context: &Context,
    task: &crate::delegation::Task,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<
    Vec<bridget_transport::provider_catalogue::CatalogueModel>,
    bridget_transport::protocol::RuntimeSelectionRefusal,
> {
    use bridget_transport::protocol::{
        RuntimeCatalogueOutcome, RuntimeSelectionRefusal as Refusal,
    };
    if task.definition.protocol != "codex_app_server" {
        return Err(Refusal::Unsupported);
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    let token = uuid::Uuid::new_v4().to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    let writer = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        if compose_live_task(&st, conn, context, task).is_err() {
            return Err(Refusal::TargetUnavailable);
        }
        if st.pending_runtime_catalogues.len() >= 64 {
            return Err(Refusal::Busy);
        }
        let Some(target) = st
            .router
            .get_agent(&task.child)
            .map(|agent| agent.connection_id.clone())
        else {
            return Err(Refusal::TargetUnavailable);
        };
        if !st
            .runtime_catalogue_connections
            .get(&target)
            .is_some_and(|(agent, instance)| {
                agent == &task.child && Some(instance) == task.child_instance.as_ref()
            })
        {
            return Err(Refusal::Unsupported);
        }
        let Some(writer) = st.connections.get(&target).cloned() else {
            return Err(Refusal::TargetUnavailable);
        };
        st.pending_runtime_catalogues
            .insert(token.clone(), (target, sender));
        writer
    };
    let observed = if push_control_message_until(
        &writer,
        &DaemonToWrapper::ReadRuntimeCatalogue {
            token: token.clone(),
        },
        deadline,
    )
    .is_ok()
    {
        receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .ok()
    } else {
        None
    };
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    st.pending_runtime_catalogues.remove(&token);
    if compose_live_task(&st, conn, context, task).is_err() {
        return Err(Refusal::TargetUnavailable);
    }
    match observed {
        Some(RuntimeCatalogueOutcome::Observed { models })
            if models.len() <= bridget_transport::provider_catalogue::MAX_CATALOGUE_MODELS
                && models
                    .iter()
                    .all(bridget_transport::provider_catalogue::CatalogueModel::valid)
                && models
                    .iter()
                    .map(|m| &m.model)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == models.len()
                && serde_json::to_vec(&models).is_ok_and(|v| {
                    v.len() <= bridget_transport::provider_catalogue::MAX_CATALOGUE_BYTES
                }) =>
        {
            Ok(models)
        }
        Some(RuntimeCatalogueOutcome::Refused { reason }) => Err(reason),
        _ => Err(Refusal::CatalogueUnavailable),
    }
}

fn compose_snapshot(
    st: &DaemonState,
    conn: &str,
    context: &Context,
    task: &crate::delegation::Task,
    request_id: &str,
    models: &[bridget_transport::provider_catalogue::CatalogueModel],
) -> Value {
    let live = compose_live_task(st, conn, context, task).is_ok();
    let active = live
        && st
            .execution_store
            .latest_in_chain(&format!("execution-{}", task.mission))
            .ok()
            .flatten()
            .is_some_and(|s| {
                !matches!(
                    s.state.as_str(),
                    "completed" | "failed" | "interrupted" | "unreachable"
                )
            });
    let supports = |operation| {
        task.definition
            .capabilities
            .observed
            .as_ref()
            .is_some_and(|o| o.supports(operation))
    };
    let interrupt = active && supports(bridget_transport::protocol::ProviderOperation::Interrupt);
    let steer = active && supports(bridget_transport::protocol::ProviderOperation::Steer);
    let select_runtime =
        live && task.definition.protocol == "codex_app_server" && !models.is_empty();
    let selection = task
        .child_instance
        .as_deref()
        .and_then(|id| st.presences.get(id))
        .filter(|p| p.name == task.child)
        .and_then(|p| {
            p.model
                .as_ref()
                .map(|model| serde_json::json!({"model":model,"effort":p.effort}))
        });
    serde_json::json!({"version":1,"status":"ok","request_id":request_id,"task_id":task.task_id,"round":task.round,
        "state":task.state,"outcome":"observed","code":null,
        "message":if live {"Commandes de cet enfant disponibles. Les réglages confirmés concernent les prochains tours."}
            else {"Cet enfant ne peut pas recevoir de commande dans son état actuel. Son journal et le brouillon sont conservés. Les autres tâches suivent leur propre état."},
        "selection":selection,"models":models.iter().map(|m|serde_json::json!({"id":m.model,"name":m.model,"efforts":m.supported_efforts})).collect::<Vec<_>>(),
        "capabilities":{"send":live,"steer":steer,"interrupt":interrupt,"select_runtime":select_runtime,"attachments":false}})
}

fn compose_outcome(value: &mut Value, outcome: &str, code: Option<&str>, message: &str) {
    value["outcome"] = serde_json::json!(outcome);
    value["code"] = serde_json::json!(code);
    value["message"] = serde_json::json!(message);
}

fn compose(conn: &str, context: &Context, state: &Arc<Mutex<DaemonState>>) -> Result<Value, E> {
    let A::Compose {
        task_id,
        request_id,
        expected_round,
        operation,
    } = &context.request.request
    else {
        return Err(E::InvalidRequest);
    };
    let canonical = serde_json::to_string(&context.request).map_err(|_| E::InvalidRequest)?;
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    guard(&st, conn, context)?;
    let task = st
        .delegation_store
        .get(task_id)
        .map_err(|_| E::StoreUnavailable)?
        .filter(|t| t.root_owner() == context.root && t.round > 0)
        .ok_or(E::TaskUnavailable)?;
    // Figer le reçu avant même la lecture fournisseur. Une perte du client
    // pendant cette lecture ne doit pas autoriser un nouvel envoi vers un enfant remplacé.
    let mut initial = compose_snapshot(&st, conn, context, &task, request_id, &[]);
    if !matches!(operation, Op::Read {}) {
        compose_outcome(
            &mut initial,
            "outcome_unknown",
            Some("command_unconfirmed"),
            "La commande est conservée. Relisez sa même clé pour vérifier son issue ; ne créez pas une seconde commande.",
        );
        if let Some(mut saved) = st.delegation_store.human_compose_reserve(
            &context.root,
            task_id,
            request_id,
            &canonical,
            &initial,
        )? {
            // Relire le puits existant peut résoudre une remise tardive, sans nouvel envoi.
            if saved["outcome"] == "outcome_unknown" && matches!(operation, Op::Send { .. }) {
                let scope =
                    crate::communication::issuer_scope(&format!("human-compose:{}", context.root));
                if let Ok(key) = IdempotencyKey::new(scope, OperationKind::Send, request_id.clone())
                {
                    match st.idempotency.lookup(&key, unix_timestamp()) {
                        Ok(LookupResult::Accepted { .. }) => compose_outcome(
                            &mut saved,
                            "accepted",
                            None,
                            "Message accepté par Bridget pour cet enfant.",
                        ),
                        Ok(LookupResult::Rejected {
                            category, reason, ..
                        }) => compose_outcome(&mut saved, "refused", Some(&category), &reason),
                        _ => {}
                    }
                    st.delegation_store.human_compose_complete(
                        &context.root,
                        task_id,
                        request_id,
                        &canonical,
                        &saved,
                    )?;
                }
            }
            if saved["outcome"] == "outcome_unknown"
                && matches!(
                    operation,
                    Op::Interrupt {}
                        | Op::Send {
                            intent: Intent::Steer,
                            ..
                        }
                )
            {
                let scope =
                    crate::communication::issuer_scope(&format!("human-compose:{}", context.root));
                if let Some(record) = st
                    .execution_store
                    .lookup_control_command(&scope, request_id, unix_timestamp())
                    .map_err(|_| E::StoreUnavailable)?
                {
                    match record.status {
                        crate::execution_store::ControlCommandStatus::Accepted => compose_outcome(
                            &mut saved,
                            "accepted",
                            None,
                            "Contrôle accepté par le pilote. La mission n'est pas annulée.",
                        ),
                        crate::execution_store::ControlCommandStatus::Refused { reason } => {
                            compose_outcome(
                                &mut saved,
                                "refused",
                                Some(&reason),
                                "Le pilote a refusé ce contrôle. Le journal et les autres tâches sont conservés.",
                            )
                        }
                        _ => {}
                    }
                    st.delegation_store.human_compose_complete(
                        &context.root,
                        task_id,
                        request_id,
                        &canonical,
                        &saved,
                    )?;
                }
            }
            return Ok(saved);
        }
    }
    drop(st);
    let (models, catalogue_failure) = match compose_catalogue(conn, context, &task, state) {
        Ok(models) => (models, None),
        Err(reason) => (Vec::new(), Some(reason)),
    };
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    guard(&st, conn, context)?;
    let current = st
        .delegation_store
        .get(task_id)
        .map_err(|_| E::StoreUnavailable)?
        .ok_or(E::TaskUnavailable)?;
    if current.child != task.child
        || current.child_instance != task.child_instance
        || current.round != task.round
        || current.remote_placement != task.remote_placement
        || current.owner != task.owner
        || current.owner_instance != task.owner_instance
        || current.origin_owner_instance != task.origin_owner_instance
        || current.root_owner() != context.root
    {
        if current.root_owner() != context.root {
            return Err(E::TaskUnavailable);
        }
        if matches!(operation, Op::Read {}) {
            return Err(E::SnapshotChanged);
        }
        compose_outcome(
            &mut initial,
            "refused",
            Some("snapshot_changed"),
            "La liaison de cet enfant a changé avant la remise. Aucune commande n'a été envoyée. Rechargez ses commandes avant une nouvelle intervention ; le brouillon est conservé.",
        );
        initial["capabilities"] = serde_json::json!({"send":false,"steer":false,"interrupt":false,"select_runtime":false,"attachments":false});
        st.delegation_store.human_compose_complete(
            &context.root,
            task_id,
            request_id,
            &canonical,
            &initial,
        )?;
        return Ok(initial);
    }
    let task = current;
    let mut response = compose_snapshot(&st, conn, context, &task, request_id, &models);
    if response["capabilities"]["send"] == true
        && let Some(reason) = catalogue_failure
    {
        let code = serde_json::to_value(reason).map_err(|_| E::StoreUnavailable)?;
        response["code"] = serde_json::json!(format!(
            "runtime_{}",
            code.as_str().unwrap_or("unavailable")
        ));
        response["message"] = serde_json::json!(format!(
            "Messages disponibles. Le catalogue de cette session n'a pas fourni de choix utilisable ({}). Modèle et effort restent inchangés ; les autres commandes gardent leurs capacités affichées.",
            code.as_str().unwrap_or("unavailable")
        ));
    }
    if matches!(operation, Op::Read {}) {
        return Ok(response);
    }
    compose_outcome(
        &mut response,
        "outcome_unknown",
        Some("command_unconfirmed"),
        "La commande est conservée. Relisez sa même clé pour vérifier son issue ; ne créez pas une seconde commande.",
    );
    if task.round != *expected_round {
        compose_outcome(
            &mut response,
            "refused",
            Some("round_changed"),
            "La ronde de cet enfant a changé. Rechargez ses commandes avant une nouvelle intervention ; le texte est conservé.",
        );
    } else if compose_live_task(&st, conn, context, &task).is_err() {
        compose_outcome(
            &mut response,
            "refused",
            Some("target_unavailable"),
            "Cet enfant n'est pas pilotable avec sa liaison actuelle. Conservez le brouillon et relisez sa tâche ; aucun autre travail n'est arrêté.",
        );
    } else {
        match operation {
            Op::Send {
                body,
                intent: Intent::Queue,
            } => {
                let proof = ComposeAuthority {
                    context: context.clone(),
                    task: task.clone(),
                };
                let mut message = bridget_core::BridgetMessage::new("human", &task.child, body);
                message.id = request_id.clone();
                message.origin = Some(bridget_core::MessageOrigin::Human);
                message.intent = Some(bridget_core::MessageIntent::QueueOnly);
                let mut controls = Vec::new();
                let outcome = handle_idempotent_send_with_result_authority(
                    conn,
                    message,
                    request_id.clone(),
                    unix_timestamp(),
                    IdempotentSendAdmission {
                        project: None,
                        issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                    },
                    &mut st,
                    &mut controls,
                    Some(RetainedSendAuthority::Lineage(&proof)),
                    false,
                );
                st.pending_post_response_controls
                    .entry(conn.into())
                    .or_default()
                    .extend(controls);
                match outcome {
                    DaemonToWrapper::IdempotencyResult {
                        issue: IdempotencyIssue::Accepted { .. },
                        ..
                    } => compose_outcome(
                        &mut response,
                        "accepted",
                        None,
                        "Message accepté par Bridget pour cet enfant.",
                    ),
                    DaemonToWrapper::IdempotencyResult {
                        issue:
                            IdempotencyIssue::Rejected {
                                category, reason, ..
                            },
                        ..
                    } => compose_outcome(&mut response, "refused", Some(&category), &reason),
                    DaemonToWrapper::Nack { reason, .. } => compose_outcome(
                        &mut response,
                        "outcome_unknown",
                        Some("send_unconfirmed"),
                        &format!(
                            "{reason}. Conservez cette même clé pour relire l'issue ; le brouillon reste conservé."
                        ),
                    ),
                    _ => {}
                }
            }
            Op::Interrupt {}
            | Op::Send {
                intent: Intent::Steer,
                ..
            } => {
                compose_execution_control(
                    &mut st,
                    conn,
                    context,
                    &task,
                    request_id,
                    operation,
                    &mut response,
                )?;
            }
            Op::SelectRuntime { model, effort } => {
                drop(st);
                return compose_select_runtime(
                    conn,
                    context,
                    &task,
                    model,
                    effort.as_deref(),
                    &models,
                    &canonical,
                    response,
                    state,
                );
            }
            Op::Read {} => unreachable!(),
        }
    }
    st.delegation_store.human_compose_complete(
        &context.root,
        task_id,
        request_id,
        &canonical,
        &response,
    )?;
    Ok(response)
}

fn compose_execution_control(
    st: &mut DaemonState,
    conn: &str,
    context: &Context,
    task: &crate::delegation::Task,
    request_id: &str,
    operation: &Op,
    response: &mut Value,
) -> Result<(), E> {
    let (kind, message, capability) = match operation {
        Op::Send {
            body,
            intent: Intent::Steer,
        } => {
            let mut message = bridget_core::BridgetMessage::new("human", &task.child, body);
            message.id = request_id.into();
            message.origin = Some(bridget_core::MessageOrigin::Human);
            message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
            (
                ExecutionControlOperation::SteerCurrent,
                Some(message),
                "steer",
            )
        }
        Op::Interrupt {} => (ExecutionControlOperation::Interrupt, None, "interrupt"),
        _ => return Err(E::InvalidRequest),
    };
    if response["capabilities"][capability] != true {
        compose_outcome(
            response,
            "refused",
            Some("capability_unavailable"),
            "Le pilote n'annonce pas ce contrôle sur le tour actuel. Utilisez l'envoi en file s'il est disponible ; le brouillon et les autres tâches sont conservés.",
        );
        return Ok(());
    }
    if let Some(snapshot) = st
        .execution_store
        .latest_in_chain(&format!("execution-{}", task.mission))
        .map_err(|_| E::StoreUnavailable)?
    {
        let proof = ComposeAuthority {
            context: context.clone(),
            task: task.clone(),
        };
        let command = ExecutionControlCommand {
            version: 1,
            command_id: request_id.into(),
            execution_id: snapshot.execution_id,
            generation: snapshot.generation,
            revision: snapshot.revision,
            operation: kind,
            message,
        };
        match handle_execution_control_authorized(
            Some(conn),
            &proof.issuer_scope(),
            command,
            st,
            Some(&proof),
        ) {
            DaemonToWrapper::ControlExecutionResult {
                outcome: ExecutionControlOutcome::Accepted,
                ..
            } => compose_outcome(
                response,
                "accepted",
                None,
                "Contrôle accepté par le pilote sur cet enfant. La mission n'est pas annulée.",
            ),
            DaemonToWrapper::ControlExecutionResult {
                outcome: ExecutionControlOutcome::Refused(reason),
                ..
            } => {
                let code = serde_json::to_value(reason).map_err(|_| E::StoreUnavailable)?;
                compose_outcome(
                    response,
                    "refused",
                    code.as_str(),
                    "Le pilote a refusé ce contrôle. Conservez la tâche et son journal ; les autres travaux suivent leur état.",
                );
            }
            _ => {}
        }
    } else {
        compose_outcome(
            response,
            "refused",
            Some("execution_unavailable"),
            "Aucun tour actif attesté n'est disponible pour cet enfant. Relisez sa tâche ; aucune annulation de mission n'a été demandée.",
        );
    }
    Ok(())
}

fn compose_select_runtime(
    conn: &str,
    context: &Context,
    task: &crate::delegation::Task,
    model: &str,
    effort: Option<&str>,
    models: &[bridget_transport::provider_catalogue::CatalogueModel],
    canonical: &str,
    mut response: Value,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<Value, E> {
    use bridget_transport::protocol::{RuntimeSelection, RuntimeSelectionOutcome as Outcome};
    let A::Compose { request_id, .. } = &context.request.request else {
        return Err(E::InvalidRequest);
    };
    let deadline = Instant::now() + Duration::from_secs(4);
    let token = uuid::Uuid::new_v4().to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    let prepared = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        compose_live_task(&st, conn, context, task)?;
        let choice = models.iter().find(|m| m.model == model);
        if task.definition.protocol != "codex_app_server" || choice.is_none() {
            compose_outcome(
                &mut response,
                "refused",
                Some("model_unavailable"),
                "Ce modèle n'est pas annoncé pour la connexion actuelle de cet enfant. Conservez le réglage courant ; aucun remplacement n'a été effectué.",
            );
            None
        } else {
            let choice = choice.unwrap();
            let effort = effort
                .or(choice.default_effort.as_deref())
                .filter(|e| choice.supported_efforts.iter().any(|known| known == e));
            if let Some(effort) = effort {
                let selection = RuntimeSelection {
                    model: model.into(),
                    effort: effort.into(),
                };
                if !selection.valid() || st.pending_runtime_selections.len() >= 64 {
                    compose_outcome(
                        &mut response,
                        "refused",
                        Some("runtime_busy"),
                        "La commande ne peut pas être admise maintenant. Conservez les réglages actuels ; relisez la même commande avant un nouvel essai.",
                    );
                    None
                } else {
                    let target = st
                        .router
                        .get_agent(&task.child)
                        .ok_or(E::TaskUnavailable)?
                        .connection_id
                        .clone();
                    let writer = st
                        .connections
                        .get(&target)
                        .cloned()
                        .ok_or(E::TaskUnavailable)?;
                    st.pending_runtime_selections
                        .insert(token.clone(), (target, sender));
                    Some((writer, selection))
                }
            } else {
                compose_outcome(
                    &mut response,
                    "refused",
                    Some("effort_unavailable"),
                    "Aucun effort demandé ou défaut natif confirmé n'est disponible pour ce modèle. Choisissez un effort annoncé ; les réglages actuels sont conservés.",
                );
                None
            }
        }
    };
    if let Some((writer, selection)) = prepared {
        let outcome = if push_control_message_until(
            &writer,
            &DaemonToWrapper::SelectRuntime {
                token: token.clone(),
                selection: selection.clone(),
            },
            deadline,
        )
        .is_ok()
        {
            receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or(Outcome::OutcomeUnknown {})
        } else {
            Outcome::OutcomeUnknown {}
        };
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.pending_runtime_selections.remove(&token);
        // Une réponse tardive ou une révocation ne permet pas d'affirmer l'état courant.
        if compose_live_task(&st, conn, context, task).is_ok() {
            match outcome {
                Outcome::Selected {
                    selection: confirmed,
                } if confirmed == selection => {
                    response["selection"] =
                        serde_json::json!({"model":confirmed.model,"effort":confirmed.effort});
                    let _ = handle_runtime_for_instance(
                        &task.child,
                        confirmed.model,
                        Some(confirmed.effort),
                        bridget_transport::protocol::RuntimeSource::CodexAppServer,
                        task.child_instance.as_deref(),
                        &mut st,
                    );
                    compose_outcome(
                        &mut response,
                        "selected",
                        None,
                        "Modèle et effort confirmés par le pilote pour les prochains tours. Le tour courant conserve son exécution.",
                    );
                }
                Outcome::Refused { reason } => {
                    let code = serde_json::to_value(reason).map_err(|_| E::StoreUnavailable)?;
                    compose_outcome(
                        &mut response,
                        "refused",
                        code.as_str(),
                        "Le pilote a refusé ce réglage. Conservez la sélection courante et choisissez parmi les capacités annoncées ; aucun remplacement n'est demandé.",
                    );
                }
                _ => {}
            }
        }
    }
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    guard(&st, conn, context)?;
    st.delegation_store.human_compose_complete(
        &context.root,
        &task.task_id,
        request_id,
        canonical,
        &response,
    )?;
    Ok(response)
}

fn journal_page(
    conn: &str,
    context: &Context,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<Value, E> {
    let A::Journal {
        task_id,
        after_seq,
        limit,
        ..
    } = &context.request.request
    else {
        return Err(E::InvalidRequest);
    };
    let task = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        guard(&st, conn, context)?;
        st.delegation_store
            .get(task_id)
            .map_err(|_| E::StoreUnavailable)?
            .filter(|task| task.root_owner() == context.root)
            .ok_or(E::TaskUnavailable)?
    };
    let (page, command) = read_task_journal(state, &task, *after_seq, *limit)?;
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    guard_journal_snapshot(&st, conn, context, &task, command.as_ref())?;
    Ok(page)
}

/// Lecture native unique ; l'appelant vérifie son autorité avant et après les I/O.
pub(super) fn read_task_journal(
    state: &Arc<Mutex<DaemonState>>,
    task: &crate::delegation::Task,
    after_seq: u64,
    limit: u32,
) -> Result<
    (
        Value,
        Option<bridget_transport::protocol::WorkerCommandIdentity>,
    ),
    E,
> {
    use bridget_transport::protocol::{
        WorkerIdentity, WorkerRequestOperation, WorkerResponseOperation,
    };
    if !(1..=100).contains(&limit) || after_seq > bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ
    {
        return Err(E::InvalidRequest);
    }
    let task_id = &task.task_id;
    let (directory, remote) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(placement) = &task.remote_placement {
            let worker = WorkerIdentity {
                machine_id: placement.machine_id.clone(),
                worker_id: placement.worker_id.clone(),
                provisioning_revision: placement.provisioning_revision,
            };
            let command = native_delegation::remote_command_identity(&st, task)
                .map_err(|_| E::JournalUnavailable)?;
            (None, Some((worker, command)))
        } else {
            (Some(journal_directory(&st, &task.child)?), None)
        }
    };
    let page = if let Some((worker, command)) = &remote {
        let response = remote_worker_roundtrip(
            state,
            worker,
            command,
            WorkerRequestOperation::JournalPage {
                after_seq: after_seq,
                limit: limit,
            },
            Instant::now() + Duration::from_secs(5),
        )
        .map_err(|_| E::JournalUnavailable)?;
        let WorkerResponseOperation::JournalPage {
            fragments,
            through_seq,
            more,
        } = response
        else {
            return Err(E::JournalUnavailable);
        };
        // Le lecteur froid du worker renvoie les entrées complètes, avec les mêmes
        // bornes que le lecteur local. Une lacune reste explicite, jamais du texte.
        let through = through_seq.unwrap_or(after_seq);
        if through < after_seq
            || through > bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ
            || fragments.len() > limit as usize
        {
            return Err(E::JournalUnavailable);
        }
        let mut next = after_seq;
        let mut bytes = 0usize;
        let mut events = Vec::new();
        for fragment in fragments {
            bytes = bytes.saturating_add(fragment.bytes.len());
            if fragment.seq != next.saturating_add(1)
                || fragment.seq > through
                || fragment.offset != 0
                || !fragment.final_fragment
                || bytes > 16 * 1024
            {
                return Err(E::JournalUnavailable);
            }
            let value: Value =
                serde_json::from_slice(&fragment.bytes).map_err(|_| E::JournalUnavailable)?;
            if value["seq"].as_u64() != Some(fragment.seq) {
                return Err(E::JournalUnavailable);
            }
            next = fragment.seq;
            events.push(value);
        }
        if through > next && !more {
            return Err(E::JournalUnavailable);
        }
        let gap=(through>next).then(||serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":through,"reason":"journal_gap"}));
        crate::attach::lineage_journal_result(task_id, events, through, !more, gap)
    } else {
        crate::attach::lineage_journal_page(
            directory.as_deref().ok_or(E::JournalUnavailable)?,
            task_id,
            after_seq,
            limit,
        )?
    };

    Ok((page, remote.map(|(_, command)| command)))
}

fn refusal(request: &Request, e: E) -> DaemonToWrapper {
    if matches!(request.request, A::Watch) {
        DaemonToWrapper::HumanLineageWatchEvent {
            event: Event::Error {
                version: 1,
                code: e,
            },
        }
    } else {
        DaemonToWrapper::HumanLineageResult { result: e.result() }
    }
}

fn terminate(watch: &mut Watch, code: E) {
    if watch.terminal {
        return;
    }
    watch.pending.clear();
    watch.pending.push_back(Event::Error { version: 1, code });
    watch.terminal = true;
    let _ = watch.notify.try_send(());
}

pub(super) fn invalidate(st: &mut DaemonState) {
    for conn in st.lineage_watches.keys().cloned().collect::<Vec<_>>() {
        let result = guard(st, &conn, &st.lineage_watches[&conn].context);
        if let Err(e) = result {
            terminate(st.lineage_watches.get_mut(&conn).unwrap(), e)
        }
    }
    for (conn, journal) in &st.lineage_journals {
        if guard_journal_snapshot(
            st,
            conn,
            &journal.context,
            &journal.task,
            journal.command.as_ref(),
        )
        .is_err()
        {
            let _ = journal.notify.try_send(());
            if let Some(socket) = &journal.shutdown {
                let _ = socket.shutdown(std::net::Shutdown::Both);
            }
        }
    }
}

pub(super) fn invalid_message(st: &mut DaemonState, conn: &str) -> bool {
    if let Some(w) = st.lineage_watches.get_mut(conn) {
        terminate(w, E::InvalidRequest);
        return true;
    }
    if let Some(j) = st.lineage_journals.get(conn) {
        let _ = j.notify.try_send(());
        if let Some(s) = &j.shutdown {
            let _ = s.shutdown(std::net::Shutdown::Both);
        }
        return true;
    }
    false
}

fn publish(st: &mut DaemonState, root: &str, mutation: &ProjectionMutation) {
    if let Some(notifier)=&st.mobile_notifier { notifier.changed(); }
    for conn in st.lineage_watches.keys().cloned().collect::<Vec<_>>() {
        if st.lineage_watches[&conn].context.root != root {
            continue;
        }
        if let Err(e) = guard(st, &conn, &st.lineage_watches[&conn].context) {
            terminate(st.lineage_watches.get_mut(&conn).unwrap(), e);
            continue;
        }
        let watch = st.lineage_watches.get_mut(&conn).unwrap();
        if watch.terminal || (watch.generation == mutation.generation && watch.seq >= mutation.seq)
        {
            continue;
        }
        let resync = watch.generation != mutation.generation
            || watch.pending.len() >= 16
            || watch
                .pending
                .iter()
                .any(|e| matches!(e, Event::Resync { .. }));
        if resync {
            watch.pending.retain(|e| matches!(e, Event::Ready { .. }))
        }
        let event = if resync {
            Event::Resync {
                version: 1,
                generation: mutation.generation.clone(),
                seq: mutation.seq,
            }
        } else {
            Event::Changed {
                version: 1,
                generation: mutation.generation.clone(),
                seq: mutation.seq,
            }
        };
        watch.generation = mutation.generation.clone();
        watch.seq = mutation.seq;
        watch.pending.push_back(event);
        let _ = watch.notify.try_send(());
    }
}

fn watch(conn: &str, context: Context, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = guard(&st, conn, &context) {
        return Some(refusal(&context.request, e));
    }
    if st.lineage_watches.len() + st.lineage_journals.len() >= 128 {
        return Some(refusal(&context.request, E::ResourceLimit));
    }
    let meta = match st.delegation_store.projection_meta() {
        Ok(m) => m,
        Err(e) => return Some(refusal(&context.request, e)),
    };
    let Some(writer) = st.connections.get(conn).cloned() else {
        return Some(refusal(&context.request, E::BindingUnavailable));
    };
    let Some(shutdown) = writer
        .try_lock()
        .ok()
        .and_then(|w| w.get_ref().try_clone().ok())
    else {
        return Some(refusal(&context.request, E::BindingUnavailable));
    };
    let (notify, receiver) = mpsc::sync_channel(1);
    st.lineage_watches.insert(
        conn.into(),
        Watch {
            context,
            pending: VecDeque::from([Event::Ready {
                version: 1,
                generation: meta.generation.clone(),
                seq: 0,
            }]),
            generation: meta.generation,
            seq: meta.seq,
            terminal: false,
            notify,
        },
    );
    if let Some(receiver) = st.delegation_store.change_receiver() {
        let weak = Arc::downgrade(state);
        thread::spawn(move || {
            while receiver.recv().is_ok() {
                let Some(state) = weak.upgrade() else { break };
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                for (root, mutation) in st.delegation_store.take_changes() {
                    publish(&mut st, &root, &mutation)
                }
            }
        });
    }
    let weak = Arc::downgrade(state);
    let conn = conn.to_string();
    thread::spawn(move || {
        loop {
            let Some(state) = weak.upgrade() else { break };
            let event = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let Some(w) = st.lineage_watches.get(&conn) else {
                    break;
                };
                if let Err(e) = guard(&st, &conn, &w.context) {
                    terminate(st.lineage_watches.get_mut(&conn).unwrap(), e)
                }
                st.lineage_watches
                    .get_mut(&conn)
                    .unwrap()
                    .pending
                    .pop_front()
            };
            if let Some(event) = event {
                let mut terminal = matches!(event, Event::Error { .. });
                let sent = push_control_message_until_checked(
                    &writer,
                    &DaemonToWrapper::HumanLineageWatchEvent { event },
                    Instant::now() + CANCEL_NOTIFICATION_BUDGET,
                    || {
                        let st = state
                            .try_lock()
                            .map_err(|_| std::io::Error::from(std::io::ErrorKind::WouldBlock))?;
                        let e = st
                            .lineage_watches
                            .get(&conn)
                            .ok_or(E::BindingUnavailable)
                            .and_then(|w| guard(&st, &conn, &w.context))
                            .err();
                        Ok(e.map(|code| {
                            terminal = true;
                            DaemonToWrapper::HumanLineageWatchEvent {
                                event: Event::Error { version: 1, code },
                            }
                        }))
                    },
                );
                if terminal || sent.is_err() {
                    state
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .lineage_watches
                        .remove(&conn);
                    break;
                }
            } else {
                drop(state);
                if receiver.recv().is_err() {
                    break;
                }
            }
        }
        let _ = shutdown.shutdown(std::net::Shutdown::Both);
    });
    None
}

fn journal_follow(
    conn: &str,
    context: Context,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    let refusal = |e| {
        Some(DaemonToWrapper::HumanLineageResult {
            result: E::result(e),
        })
    };
    if let Err(e) = guard(&st, conn, &context) {
        return refusal(e);
    }
    if st.lineage_watches.len() + st.lineage_journals.len() >= 128 {
        return refusal(E::ResourceLimit);
    }
    let A::Journal {
        task_id,
        after_seq,
        limit,
        ..
    } = &context.request.request
    else {
        unreachable!()
    };
    let task = match st.delegation_store.get(task_id) {
        Ok(Some(t)) if t.root_owner() == context.root => t,
        _ => return refusal(E::TaskUnavailable),
    };
    let command = if task.remote_placement.is_some() {
        match native_delegation::remote_command_identity(&st, &task) {
            Ok(command) => Some(command),
            Err(_) => return refusal(E::JournalUnavailable),
        }
    } else {
        None
    };
    let directory = match journal_directory(&st, &task.child) {
        Ok(d) => d,
        Err(e) => return refusal(e),
    };
    let socket = match crate::environment::Namespace::from_environment() {
        Ok(n) => n.socket,
        Err(_) => return refusal(E::JournalUnavailable),
    };
    let live = st.router.get_agent(&task.child).is_some();
    if task.remote_placement.is_some() && !live {
        return refusal(E::JournalUnavailable);
    }
    let Some(writer) = st.connections.get(conn).cloned() else {
        return refusal(E::BindingUnavailable);
    };
    let shutdown = match writer
        .try_lock()
        .ok()
        .and_then(|w| w.get_ref().try_clone().ok())
    {
        Some(s) => s,
        None => return refusal(E::BindingUnavailable),
    };
    let task_id = task_id.clone();
    let after_seq = *after_seq;
    let limit = *limit;
    let (notify, receiver) = mpsc::sync_channel(1);
    st.lineage_journals.insert(
        conn.into(),
        Journal {
            context: context.clone(),
            task: task.clone(),
            command: command.clone(),
            shutdown: None,
            notify,
        },
    );
    let weak = Arc::downgrade(state);
    let conn = conn.to_string();
    thread::spawn(move || {
        let mut emit = |result: Value| -> Result<(), E> {
            let state = weak.upgrade().ok_or(E::BindingUnavailable)?;
            let mut refused = None;
            let sent = push_control_message_until_checked(
                &writer,
                &DaemonToWrapper::HumanLineageResult { result },
                Instant::now() + CANCEL_NOTIFICATION_BUDGET,
                || {
                    let st = state
                        .try_lock()
                        .map_err(|_| std::io::Error::from(std::io::ErrorKind::WouldBlock))?;
                    Ok(
                        guard_journal_snapshot(&st, &conn, &context, &task, command.as_ref())
                            .err()
                            .map(|e| {
                                refused = Some(e);
                                DaemonToWrapper::HumanLineageResult { result: e.result() }
                            }),
                    )
                },
            );
            sent.map_err(|_| E::BindingUnavailable)?;
            if let Some(e) = refused {
                return Err(e);
            }
            Ok(())
        };
        let result = if live {
            crate::attach::lineage_journal_follow(
                &socket,
                &task.child,
                &task_id,
                after_seq,
                limit,
                |stream| {
                    let state = weak.upgrade().ok_or(E::BindingUnavailable)?;
                    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                    guard_journal_snapshot(&st, &conn, &context, &task, command.as_ref())?;
                    let journal = st
                        .lineage_journals
                        .get_mut(&conn)
                        .ok_or(E::BindingUnavailable)?;
                    journal.shutdown = Some(stream.try_clone().map_err(|_| E::JournalUnavailable)?);
                    Ok(())
                },
                &mut emit,
            )
        } else {
            let mut cursor = after_seq;
            loop {
                let page = match crate::attach::lineage_journal_page(
                    &directory, &task_id, cursor, limit,
                ) {
                    Ok(p) => p,
                    Err(e) => break Err(e),
                };
                let caught_up = page["caught_up"] == true;
                let next = page["next_seq"].as_u64().unwrap_or(cursor);
                if let Err(e) = emit(page) {
                    break Err(e);
                }
                if caught_up {
                    break Ok(());
                }
                if next <= cursor {
                    break Err(E::JournalUnavailable);
                }
                cursor = next;
            }
        };
        let result = result.and_then(|()| {
            // Une frontière caught_up n'est pas une fin d'abonnement. Les seules
            // reprises ici sont les invalidations de garde, sans relire le corps.
            if receiver.recv().is_ok() {
                let state = weak.upgrade().ok_or(E::BindingUnavailable)?;
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                if !st.lineage_journals.contains_key(&conn) {
                    return Ok(());
                }
                guard(&st, &conn, &context)?;
                return Err(E::InvalidRequest);
            }
            Ok(())
        });
        if let Err(error) = result {
            let error = weak
                .upgrade()
                .and_then(|s| {
                    guard(
                        &s.lock().unwrap_or_else(|e| e.into_inner()),
                        &conn,
                        &context,
                    )
                    .err()
                })
                .unwrap_or(error);
            let _ = emit(error.result());
        }
        if let Some(state) = weak.upgrade() {
            state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .lineage_journals
                .remove(&conn);
        }
        let _ = shutdown.shutdown(std::net::Shutdown::Both);
    });
    None
}

#[cfg(test)]
#[path = "native_lineage_tests.rs"]
mod native_lineage_tests;
