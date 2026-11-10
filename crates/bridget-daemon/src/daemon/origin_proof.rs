//! Relais d'origine sur le worker provisionné, corrélé à son époque actuelle.
use super::*;
use std::path::Path;
use bridget_transport::protocol::{
    OriginProofOperation, OriginProofRequest, OriginProofResponse, OriginProofResult,
    WORKER_STALE_SECS, WorkerCapability, WorkerContactState, WorkerIdentity, WorkerRefusal,
};
pub(super) struct Pending {
    pub conn_id: String,
    pub epoch: String,
    worker: WorkerIdentity,
    reply: mpsc::SyncSender<OriginProofResult>,
}
/// L'hôte vient de la présence primaire, jamais du paramètre d'un outil.
pub(super) fn foreign_host(st: &DaemonState, owner: &str) -> Result<Option<String>, String> {
    let Some(route) = st.router.get_agent(owner) else {
        return Ok(None);
    };
    let Some(instance) = st.conn_instances.get(&route.connection_id) else {
        return Err("origin_binding_unavailable".into());
    };
    let presence = st
        .presences
        .get(instance)
        .ok_or("origin_binding_unavailable")?;
    if !bridget_core::host_is_attested(&presence.host) {
        return Err("origin_binding_unavailable".into());
    }
    Ok((presence.host != st.host).then(|| presence.host.clone()))
}
/// Un fils garde sa machine de placement ; un parent natif garde sa présence primaire.
pub(super) fn host_for_owner(st:&DaemonState,owner:&str)->Result<String,String>{
    if let Some(placement)=st.delegation_store.for_child(owner)?.and_then(|task|task.remote_placement){return Ok(placement.machine_id);}
    Ok(foreign_host(st,owner)?.unwrap_or_else(||st.host.clone()))
}

pub(super) fn worker_is_current(st:&DaemonState,host:&str)->bool {
    let Some(provision)=st.registry.machines().get(host) else {return false};
    st.remote_workers.get(host).is_some_and(|worker|provision.enabled
        && provision.worker_id==worker.identity.worker_id
        && provision.provisioning_revision==worker.identity.provisioning_revision
        && worker.contact==WorkerContactState::Connected
        && worker.last_contact.elapsed()<Duration::from_secs(WORKER_STALE_SECS as u64)
        && worker.capabilities.contains(&WorkerCapability::OriginProofV1))
}
fn shape_error(field:&'static str,reason:WorkerRefusal,same_machine:bool)->String {
    let cause=match (field,reason,same_machine) {
        ("target_machine",WorkerRefusal::InvalidEnvelope,true)=>"same_machine_target",
        (_,WorkerRefusal::InvalidEnvelope,_)=>"invalid_envelope",
        (_,WorkerRefusal::PermissionsChanged,_)=>"permissions_changed",
        (_,WorkerRefusal::ResourceLimit,_)=>"resource_limit",
        _=>"unobserved",
    };
    format!("origin_request_invalid:{field}:{cause}")
}
/// Un cwd hors du dossier attesté du parent est un refus de périmètre, pas de forme.
fn cwd_outside_parent(operation:&OriginProofOperation)->bool {
    match operation {
        OriginProofOperation::PrepareProject {parent,source,..}
        | OriginProofOperation::Sources {parent,source:Some(source)} =>
            !bridget_transport::protocol::provider_allows_source(parent,&source.cwd),
        _=>false,
    }
}

fn exchange(
    state: &Arc<Mutex<DaemonState>>,
    host: &str,
    mut operation: OriginProofOperation,
    budget: Duration,
) -> Result<OriginProofResult, String> {
    let deadline = Instant::now() + budget;
    let (tx, rx) = mpsc::sync_channel(1);
    let (writer, request, connection) = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        if !worker_is_current(&st,host) {return Err("origin_worker_unavailable".into());}
        let worker=st.remote_workers.get(host).ok_or("origin_worker_unavailable")?;
        if let OriginProofOperation::PrepareProject {return_context,..}=&mut operation {
            *return_context=worker.capabilities.contains(&WorkerCapability::OriginProjectContextV1);
        }
        let view_source=match &operation {
            OriginProofOperation::Sources {source,..}=>source.as_ref(),
            OriginProofOperation::PrepareProject {source,..}=>Some(source),
            _=>None,
        }.is_some_and(|source|matches!(source.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})));
        // La même capacité couvre la préparation source sans checkout et le créateur.
        // Un ancien lecteur ne reçoit jamais cette demande qu'il pourrait mal préparer.
        if view_source && !crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED {
            return Err("workspace_commit_view_not_enabled".into());
        }
        if view_source && (!worker.capabilities.contains(&WorkerCapability::WorkspaceViewCreateV1)
            || !worker.capabilities.contains(&WorkerCapability::WorkspaceViewV1)) {
            return Err("origin_commit_view_unsupported".into());
        }
        let request = OriginProofRequest {
            version: 1,
            correlation_id: Uuid::new_v4().to_string(),
            connection_epoch: worker.epoch.clone(),
            worker: worker.identity.clone(),
            expires_at_ms: crate::origin_proof::now_ms().saturating_add(budget.as_millis() as i64),
            request: operation,
        };
        let same_machine=matches!(&request.request,OriginProofOperation::PrepareProject {target_machine,..}
            if target_machine==&request.worker.machine_id);
        let outside_parent=cwd_outside_parent(&request.request);
        request.validate_shape_detailed()
            .map_err(|(field,reason)| if field=="cwd" && outside_parent {
                "cwd_outside_parent_project".to_owned()
            } else {shape_error(field,reason,same_machine)})?;
        let connection = worker.conn_id.clone();
        let writer = st
            .connections
            .get(&connection)
            .cloned()
            .ok_or("origin_worker_unavailable")?;
        if st.pending_origin_proofs.len() >= crate::origin_proof::MAX_ORIGIN_PROOFS {
            return Err("origin_proof_busy".into());
        }
        st.pending_origin_proofs.insert(
            request.correlation_id.clone(),
            Pending {
                conn_id: connection.clone(),
                epoch: request.connection_epoch.clone(),
                worker: request.worker.clone(),
                reply: tx,
            },
        );
        (writer, request, connection)
    };
    let preparation=matches!(&request.request,OriginProofOperation::PrepareProject {..});
    let correlation = request.correlation_id.clone();
    let epoch = request.connection_epoch.clone();
    let identity = request.worker.clone();
    let sent = push_control_message_until(
        &writer,
        &DaemonToWrapper::OriginProofRequest { request },
        deadline,
    )
    .is_ok();
    let response = if sent {
        rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .ok()
    } else {
        None
    };
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    st.pending_origin_proofs.remove(&correlation);
    if Instant::now() >= deadline
        || !st.remote_workers.get(host).is_some_and(|w| {
            w.conn_id == connection
                && w.epoch == epoch
                && w.identity == identity
                && w.contact == WorkerContactState::Connected
        })
    {
        return Err("origin_proof_unavailable".into());
    }
    checked_response(response, preparation)
}
fn checked_response(response: Option<OriginProofResult>, preparation: bool) -> Result<OriginProofResult, String> {
    match response {
        Some(OriginProofResult::Rejected { reason: WorkerRefusal::ResourceLimit }) => {
            Err(if preparation { "origin_project_preparation_busy" }
                else { "origin_proof_busy" }.into())
        }
        Some(OriginProofResult::Rejected { reason }) => {
            let reason = serde_json::to_value(reason).map_err(|_| "origin_proof_unavailable".to_owned())?;
            Err(format!("origin_proof_rejected:{}", reason.as_str().ok_or("origin_proof_unavailable")?))
        }
        None => Err("origin_proof_unavailable".into()),
        Some(result) => Ok(result),
    }
}

/// Garder une capacité ou une liaison indisponible distincte d'un refus de preuve.
pub(super) fn attestation_failure(code: String) -> String {
    if rejected_reason(&code).is_some() { return code; }
    match code.as_str() {
        "origin_proof_busy" | "origin_project_preparation_busy" | "origin_proof_unavailable"
        | "origin_worker_unavailable" | "origin_binding_unavailable" => code,
        _ => "permission_attestation_unavailable".into(),
    }
}

pub(super) fn rejected_reason(code: &str) -> Option<WorkerRefusal> {
    serde_json::from_value(serde_json::json!(code.strip_prefix("origin_proof_rejected:")?)).ok()
}

pub(super) fn rejection_message(reason: WorkerRefusal) -> &'static str {
    match reason {
        WorkerRefusal::PermissionsChanged => "Conservez cette clé et la tâche éventuelle, puis vérifiez le tour actif et sa preuve de droits. L’hôte d’origine a refusé cette preuve ; elle peut ne plus appartenir au tour courant. Aucun droit nouveau n’est accordé. Relisez l’état natif avant toute reprise, sans mission de remplacement. Ce refus concerne cette opération ; les autres travaux autorisés suivent leur propre état.",
        WorkerRefusal::Unauthorized => "Conservez cette clé et la tâche éventuelle, puis faites vérifier l’identité et le rattachement sur l’hôte d’origine. Cet hôte a explicitement refusé le contrôle. Aucun changement de droits ni rejeu ne résout ce seul constat. Les autres travaux autorisés suivent leur propre état.",
        WorkerRefusal::PrepareExpired => "Conservez cette clé et la tâche éventuelle, puis faites vérifier l’échéance du contrôle sur l’hôte d’origine. Cet hôte a répondu que le contrôle était expiré. Relisez l’état natif avant toute reprise, sans mission de remplacement. Les autres travaux autorisés suivent leur propre état.",
        _ => "Conservez cette clé et la tâche éventuelle, puis faites vérifier le motif technique renvoyé par l’hôte d’origine. Cet hôte a explicitement refusé le contrôle ; ce code seul ne prouve ni panne globale ni changement de droits. Relisez l’état natif avant toute reprise, sans mission de remplacement. Les autres travaux autorisés suivent leur propre état.",
    }
}
#[cfg(test)]
mod capacity221_tests {
    use super::*;
    #[test]
    fn spec240_attestation_preserves_transient_causes_and_still_refuses_unknown() {
        for code in ["origin_proof_busy", "origin_project_preparation_busy", "origin_proof_unavailable",
            "origin_worker_unavailable", "origin_binding_unavailable"] {
            assert_eq!(attestation_failure(code.into()), code);
        }
        for code in ["permission_attestation_unavailable", "unauthorized", "unknown with secret"] {
            assert_eq!(attestation_failure(code.into()), "permission_attestation_unavailable");
        }
    }
    #[test]
    fn spec243_cwd_hors_du_parent_est_un_refus_de_perimetre() {
        let parent:bridget_transport::protocol::ProviderPermissions=serde_json::from_value(serde_json::json!({
            "version":1,"source":"provider_turn","run_id":"run:243","provider_session_id":"session-243",
            "provider_instance_id":"codex","revision":1,"driver":"codex_app_server","cwd":"/source/project",
            "runtime_mode":"full-access","interaction_mode":"default",
            "provider_policy":{"kind":"codex","approval_policy":"never",
                "approvals_reviewer":"user","sandbox_policy":{"type":"readOnly"}}
        })).unwrap();
        let source=|cwd:&str|bridget_transport::protocol::WorkerSourceBase {cwd:cwd.into(),
            expected_base_commit:None,workspace:None,
            workload:bridget_transport::protocol::AutoDelegateWorkload::General};
        let sources=|cwd:&str|OriginProofOperation::Sources {parent:parent.clone(),source:Some(source(cwd))};
        assert!(cwd_outside_parent(&sources("/source/autre")));
        assert!(cwd_outside_parent(&sources("/source/project-voisin")));
        assert!(!cwd_outside_parent(&sources("/source/project")));
        assert!(!cwd_outside_parent(&sources("/source/project/sous-dossier")));
        assert!(!cwd_outside_parent(&OriginProofOperation::Sources {parent:parent.clone(),source:None}));
    }

    #[test]
    fn spec236_origin_refusal_cause_distinguishes_same_machine_and_invalid_values() {
        assert_eq!(shape_error("target_machine",WorkerRefusal::InvalidEnvelope,true),
            "origin_request_invalid:target_machine:same_machine_target");
        assert_eq!(shape_error("target_machine",WorkerRefusal::InvalidEnvelope,false),
            "origin_request_invalid:target_machine:invalid_envelope");
        assert_eq!(shape_error("parent",WorkerRefusal::PermissionsChanged,true),
            "origin_request_invalid:parent:permissions_changed");
        assert_eq!(shape_error("envelope",WorkerRefusal::ResourceLimit,true),
            "origin_request_invalid:envelope:resource_limit");
    }

    #[test]
    fn spec221_origin_busy_is_capacity_not_a_permission_change() {
        assert_eq!(checked_response(Some(OriginProofResult::Rejected {
            reason: WorkerRefusal::ResourceLimit
        }), false), Err("origin_proof_busy".into()));
        assert_eq!(checked_response(Some(OriginProofResult::Rejected {
            reason: WorkerRefusal::ResourceLimit
        }), true), Err("origin_project_preparation_busy".into()));
        assert_eq!(checked_response(None, false), Err("origin_proof_unavailable".into()));
        for (reason, label) in [(WorkerRefusal::PermissionsChanged,"permissions_changed"),
            (WorkerRefusal::Unauthorized,"unauthorized"),(WorkerRefusal::PrepareExpired,"prepare_expired")] {
            let code=format!("origin_proof_rejected:{label}");
            assert_eq!(checked_response(Some(OriginProofResult::Rejected {reason}),false),Err(code.clone()));
            assert_eq!(attestation_failure(code.clone()),code);
            assert_eq!(rejected_reason(&code),Some(reason));
        }
        assert!(rejected_reason("origin_proof_rejected:unknown secret").is_none());
        let ok=OriginProofResult::CommunicationProject { project: None };
        assert_eq!(checked_response(Some(ok.clone()), false), Ok(ok));
    }
}
pub(super) fn communication_project(state:&Arc<Mutex<DaemonState>>,host:&str,root:&str,
    source:bridget_transport::protocol::CommunicationProjectSource,worktree:Option<&str>)
    ->Result<Option<bridget_transport::protocol::CommunicationProject>,String>{
    let result=exchange(state,host,OriginProofOperation::CommunicationProject {
        root:root.to_owned(),source,worktree:worktree.map(str::to_owned)},Duration::from_secs(10))?;
    match result {OriginProofResult::CommunicationProject {project}=>Ok(project),_=>Err("origin_proof_unavailable".into())}
}
pub(super) fn receive(
    st: &mut DaemonState,
    connection: &str,
    response: OriginProofResponse,
) -> Option<DaemonToWrapper> {
    let invalid = || {
        Some(DaemonToWrapper::WorkerRejected {
            reason: bridget_transport::protocol::WorkerRefusal::StaleConnection,
        })
    };
    let Some(worker) = st.remote_workers.get(&response.worker.machine_id) else {
        return invalid();
    };
    if worker.conn_id != connection
        || worker.epoch != response.connection_epoch
        || worker.identity != response.worker
        || !worker
            .capabilities
            .contains(&WorkerCapability::OriginProofV1)
        || response.validate_shape().is_err()
    {
        return invalid();
    }
    let pending = st.pending_origin_proofs.get(&response.correlation_id)?;
    if pending.conn_id != connection
        || pending.epoch != response.connection_epoch
        || pending.worker != response.worker
    {
        return invalid();
    }
    let pending = st
        .pending_origin_proofs
        .remove(&response.correlation_id)
        .unwrap();
    let _ = pending.reply.try_send(response.response);
    None
}
pub(super) fn reattest(
    connection: &str,
    proof: &bridget_transport::protocol::NativePermissionProof,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<crate::t3code_mcp::SessionProof, String> {
    let binding = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let (owner, instance) = live_connection_identity(&st, connection)
            .ok_or("permission_attestation_unavailable")?;
        let route = st
            .router
            .get_agent(&owner)
            .ok_or("permission_attestation_unavailable")?;
        let primary = route.connection_id.clone();
        let thread = st
            .t3_thread_bindings
            .get(&primary)
            .and_then(|t| t.clone())
            .ok_or("permission_attestation_unavailable")?;
        human_authority_guard(&st, &thread, &agent_communication_project(&st, &owner))
            .map_err(|_| "permission_attestation_unavailable")?;
        (
            owner,
            instance,
            primary,
            thread,
            foreign_host(&st, &live_connection_identity(&st, connection).unwrap().0)?,
        )
    };
    let attested = match &binding.4 {
        None => {
            crate::t3code_mcp::reattest(proof).map_err(|_| "permission_attestation_unavailable")?
        }
        Some(host) => {
            let result = exchange(
                state,
                host,
                OriginProofOperation::Attest {
                    expected_thread: binding.3.clone(),
                    proof: proof.clone(),
                },
                Duration::from_secs(35),
            )?;
            let OriginProofResult::Attested { proof } = result else {
                return Err("permission_attestation_unavailable".into());
            };
            proof
        }
    };
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    if attested.thread_id != binding.3
        || live_connection_identity(&st, connection) != Some((binding.0.clone(), binding.1.clone()))
        || st
            .router
            .get_agent(&binding.0)
            .is_none_or(|r| r.connection_id != binding.2)
        || foreign_host(&st, &binding.0)? != binding.4
        || human_authority_guard(
            &st,
            &binding.3,
            &agent_communication_project(&st, &binding.0),
        )
        .is_err()
    {
        return Err("permission_attestation_unavailable".into());
    }
    Ok(attested)
}
fn source_in_parent211(parent:&bridget_transport::protocol::ProviderPermissions,
    source:Option<&bridget_transport::protocol::WorkerSourceBase>)->Result<(),String>{
    if source.is_some_and(|s|{
        let p=std::path::Path::new(&s.cwd);
        !p.is_absolute() || p.components().any(|c|matches!(c,std::path::Component::ParentDir))
            || crate::lifecycle::source_root(parent,p).is_err()
    }) {return Err("cwd_outside_parent_project".into());}
    Ok(())
}
pub(super) fn sources(
    state:&Arc<Mutex<DaemonState>>,owner:&str,parent:&bridget_transport::protocol::ProviderPermissions,
    source:Option<bridget_transport::protocol::WorkerSourceBase>,
    admitted:Option<&crate::delegation::Task>,
)->Result<Option<(String,Option<String>)>,String>{
    source_in_parent211(parent,source.as_ref())?;
    let operation = OriginProofOperation::Sources {parent:parent.clone(),source};
    let result = if let Some(task) = admitted {
        if task.owner != owner || task.admitted_parent()? != Some(parent) {
            return Err("admission_authority_invalid".into());
        }
        let host = {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            let current = st.delegation_store.get(&task.task_id)?.ok_or("task_unavailable")?;
            super::native_delegation::recheck_remote_dispatch(&st, &current, task)?;
            if current.state != task.state || matches!(current.state.as_str(), "cancelled" | "failed")
                || current.guard_released != task.guard_released {
                return Err("remote_task_changed".into());
            }
            let host = &task.remote_placement.as_ref().ok_or("remote_placement_missing")?.origin_host;
            if *host == st.host { return Ok(None); }
            host.clone()
        };
        // L'hôte vient de la Task attestée. Aucune route de tour, ni credential,
        // n'est nécessaire pour relire les mêmes fichiers sur cet hôte provisionné.
        let response = exchange(state, &host, operation, Duration::from_secs(30))?;
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let current = st.delegation_store.get(&task.task_id)?.ok_or("task_unavailable")?;
        super::native_delegation::recheck_remote_dispatch(&st, &current, task)?;
        if current.state != task.state || current.guard_released != task.guard_released {
            return Err("remote_task_changed".into());
        }
        if let OriginProofResult::Sources {permission_digest,..} = &response {
            use sha2::Digest;
            let expected=format!("{:x}",sha2::Sha256::digest(serde_json::to_vec(parent).map_err(|_|"origin_proof_unavailable")?));
            if *permission_digest != expected { return Err("remote_parent_proof_changed".into()); }
        }
        Some(response)
    } else {
        parent_operation(state,owner,parent,operation,Duration::from_secs(30))?
    };
    match result {
        None=>Ok(None),Some(OriginProofResult::Sources {permission_digest,source_error})=>Ok(Some((permission_digest,source_error))),
        _=>Err("origin_proof_unavailable".into()),
    }
}
pub(super) fn prepare_project(
    state:&Arc<Mutex<DaemonState>>,owner:&str,parent:&bridget_transport::protocol::ProviderPermissions,
    source:bridget_transport::protocol::WorkerSourceBase,target_machine:&str,project_id:Option<&str>,deadline:Instant,
)->Result<(),String>{
    source_in_parent211(parent,Some(&source))?;
    let budget=deadline.saturating_duration_since(Instant::now()).min(Duration::from_secs(595));
    if budget.is_zero() {return Err("project_preparation_timeout".into());}
    let (registry_path,expected,origin_host)={
        let st=state.lock().unwrap_or_else(|e|e.into_inner());
        let registry=AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf())?;
        (registry.source().to_path_buf(),registry.machine(target_machine)?.clone(),host_for_owner(&st,owner)?)
    };
    // exchange choisit return_context sur la capacité négociée de la connexion
    // réellement utilisée, pas sur un numéro de version ni un ancien snapshot.
    let result=parent_operation(state,owner,parent,OriginProofOperation::PrepareProject {
        parent:parent.clone(),source:source.clone(),target_machine:target_machine.to_owned(),project_id:project_id.map(str::to_owned),return_context:false
    },budget)?.ok_or("origin_binding_unavailable")?;
    let OriginProofResult::ProjectPrepared {preparation_error,project,..}=result else {
        return Err("origin_proof_unavailable".into());
    };
    if let Some(error)=preparation_error {return Err(error);}
    let Some(project)=project else {
        // Un ancien worker reste utilisable pour un mapping déjà connu. Son
        // succès sans contexte ne prouve pas le raccordement d'un nouveau projet.
        return AgentRegistry::load_from_path_quiet(registry_path)?
            .select_project_mapping(target_machine,project_id,&origin_host,Path::new(&source.cwd))
            .map(|_|()).map_err(|_|"project_preparation_context_missing".into());
    };
    if project.target.machine_id!=target_machine || project.target.worker_id!=expected.worker_id
        || project.target.provisioning_revision!=expected.provisioning_revision
        || project.credential_fingerprint!=expected.credential_fingerprint {
        return Err("project_preparation_machine_mismatch".into());
    }
    if project_id.is_some_and(|id|id!=project.project_id) {
        return Err("project_mapping_mismatch".into());
    }
    let mapping:crate::registry::ProjectMapping=serde_json::from_value(project.mapping.clone())
        .map_err(|_|"project_mapping_invalid".to_owned())?;
    let mut validated=expected.clone();
    validated.mappings.insert(project.project_id.clone(),mapping);
    crate::registry::validate_machines(&std::collections::BTreeMap::from([(target_machine.to_owned(),validated)]))?;
    crate::project_preparation::prepare_until(&serde_json::json!({
        "operation":"publish_prepared","registry":registry_path,"machine_id":target_machine,
        "authority":{"worker_id":expected.worker_id,"provisioning_revision":expected.provisioning_revision,
            "credential_fingerprint":expected.credential_fingerprint},
        "origin_host":origin_host,"source_root":crate::lifecycle::source_root(parent,Path::new(&source.cwd))?,"cwd":source.cwd,
        "project_id":project.project_id,"mapping":project.mapping
    }),deadline)
}
pub(super) fn catalogue(state:&Arc<Mutex<DaemonState>>,owner:&str,parent:&bridget_transport::protocol::ProviderPermissions,
    blocked:Vec<String>)->Result<Vec<serde_json::Value>,String>{
    let result=parent_operation(state,owner,parent,OriginProofOperation::Catalogue {parent:parent.clone(),blocked},
        Duration::from_secs(30))?.ok_or("origin_binding_unavailable")?;
    match result {OriginProofResult::Catalogue {entries,..}=>Ok(entries),_=>Err("origin_proof_unavailable".into())}
}
fn parent_operation(
    state:&Arc<Mutex<DaemonState>>,owner:&str,parent:&bridget_transport::protocol::ProviderPermissions,
    operation:OriginProofOperation,budget:Duration,
)->Result<Option<OriginProofResult>,String>{
    let binding = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let Some(host) = foreign_host(&st, owner)? else {
            return Ok(None);
        };
        let route = st
            .router
            .get_agent(owner)
            .ok_or("origin_binding_unavailable")?;
        let instance = st
            .conn_instances
            .get(&route.connection_id)
            .cloned()
            .ok_or("origin_binding_unavailable")?;
        (host, route.connection_id.clone(), instance)
    };
    let expected_kind=match &operation {OriginProofOperation::Sources {..}=>0,
        OriginProofOperation::PrepareProject {..}=>1,OriginProofOperation::Catalogue {..}=>2,
        _=>return Err("origin_proof_unavailable".into())};
    let result = exchange(
        state,
        &binding.0,
        operation,
        budget,
    )?;
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    if st
        .router
        .get_agent(owner)
        .is_none_or(|r| r.connection_id != binding.1)
        || st.conn_instances.get(&binding.1) != Some(&binding.2)
        || foreign_host(&st, owner)?.as_deref() != Some(&binding.0)
    {
        return Err("origin_binding_unavailable".into());
    }
    let permission_digest=match (&result,expected_kind) {
        (OriginProofResult::Sources {permission_digest,..},0)
        | (OriginProofResult::ProjectPrepared {permission_digest,..},1)
        | (OriginProofResult::Catalogue {permission_digest,..},2)=>permission_digest,
        _=>return Err("origin_proof_unavailable".into())
    };
    use sha2::Digest;
    let expected=format!("{:x}",sha2::Sha256::digest(serde_json::to_vec(parent).map_err(|_|"origin_proof_unavailable")?));
    if permission_digest!=&expected {return Err("remote_parent_proof_changed".into());}
    Ok(Some(result))
}

#[cfg(test)]
#[path = "origin_proof_tests.rs"]
mod tests;

#[cfg(test)]
mod source211_tests {
    use super::*;
    #[test]
    fn spec211_source_scope_is_refused_before_relay_without_wider_rights(){
        let parent=bridget_transport::protocol::ProviderPermissions {
            version:1,source:"provider_turn".into(),run_id:"run".into(),provider_session_id:"session".into(),
            provider_instance_id:"instance".into(),revision:1,driver:"codex_app_server".into(),
            cwd:"/project/.worktrees/parent".into(),runtime_mode:"full-access".into(),
            interaction_mode:"default".into(),provider_policy:serde_json::json!({"kind":"codex"})
        };
        let mut source=bridget_transport::protocol::WorkerSourceBase {
            cwd:parent.cwd.clone(),expected_base_commit:None,workspace:None,
            workload:bridget_transport::protocol::AutoDelegateWorkload::General,
        };
        assert!(source_in_parent211(&parent,Some(&source)).is_ok());
        source.cwd="/project/.worktrees/parent/sub".into();
        assert!(source_in_parent211(&parent,Some(&source)).is_ok());
        for cwd in ["/project","/project/.worktrees/sibling","/project/.worktrees/parent/../sibling","relative"]{
            source.cwd=cwd.into();
            assert_eq!(source_in_parent211(&parent,Some(&source)),Err("cwd_outside_parent_project".into()));
        }
    }
}
