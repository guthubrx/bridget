//! Relais d'origine sur le worker provisionné, corrélé à son époque actuelle.
use super::*;
use bridget_transport::protocol::{
    OriginProofOperation, OriginProofRequest, OriginProofResponse, OriginProofResult,
    WORKER_STALE_SECS, WorkerCapability, WorkerContactState, WorkerIdentity,
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
fn exchange(
    state: &Arc<Mutex<DaemonState>>,
    host: &str,
    operation: OriginProofOperation,
    budget: Duration,
) -> Result<OriginProofResult, String> {
    let deadline = Instant::now() + budget;
    let (tx, rx) = mpsc::sync_channel(1);
    let (writer, request, connection) = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        if !worker_is_current(&st,host) {return Err("origin_worker_unavailable".into());}
        let worker=st.remote_workers.get(host).ok_or("origin_worker_unavailable")?;
        let request = OriginProofRequest {
            version: 1,
            correlation_id: Uuid::new_v4().to_string(),
            connection_epoch: worker.epoch.clone(),
            worker: worker.identity.clone(),
            expires_at_ms: crate::origin_proof::now_ms().saturating_add(budget.as_millis() as i64),
            request: operation,
        };
        request
            .validate_shape()
            .map_err(|_| "origin_request_invalid")?;
        let connection = worker.conn_id.clone();
        let writer = st
            .connections
            .get(&connection)
            .cloned()
            .ok_or("origin_worker_unavailable")?;
        if st.pending_origin_proofs.len() >= 32 {
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
    match response {
        Some(OriginProofResult::Rejected { .. }) | None => Err("origin_proof_unavailable".into()),
        Some(result) => Ok(result),
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
    let Some(pending) = st.pending_origin_proofs.get(&response.correlation_id) else {
        return None;
    };
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
pub(super) fn sources(
    state:&Arc<Mutex<DaemonState>>,owner:&str,parent:&bridget_transport::protocol::ProviderPermissions,
    source:Option<bridget_transport::protocol::WorkerSourceBase>,
)->Result<Option<(String,Option<String>)>,String>{
    match parent_operation(state,owner,parent,OriginProofOperation::Sources {parent:parent.clone(),source},Duration::from_secs(30))? {
        None=>Ok(None),Some(OriginProofResult::Sources {permission_digest,source_error})=>Ok(Some((permission_digest,source_error))),
        _=>Err("origin_proof_unavailable".into()),
    }
}
pub(super) fn prepare_project(
    state:&Arc<Mutex<DaemonState>>,owner:&str,parent:&bridget_transport::protocol::ProviderPermissions,
    source:bridget_transport::protocol::WorkerSourceBase,target_machine:&str,project_id:Option<&str>,deadline:Instant,
)->Result<(),String>{
    let budget=deadline.saturating_duration_since(Instant::now()).min(Duration::from_secs(595));
    if budget.is_zero() {return Err("project_preparation_timeout".into());}
    let result=parent_operation(state,owner,parent,OriginProofOperation::PrepareProject {
        parent:parent.clone(),source,target_machine:target_machine.to_owned(),project_id:project_id.map(str::to_owned)
    },budget)?.ok_or("origin_binding_unavailable")?;
    match result {OriginProofResult::ProjectPrepared {preparation_error,..}=>preparation_error.map_or(Ok(()),Err),
        _=>Err("origin_proof_unavailable".into())}
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
mod tests {
    use super::*;
    use bridget_transport::protocol::*;
    fn fixture() -> (Arc<Mutex<DaemonState>>, PathBuf, WorkerIdentity) {
        use sha2::Digest;
        use std::os::unix::fs::DirBuilderExt;
        let root = std::env::temp_dir().join(format!("bridget177-origin-{}", Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let config = DaemonConfig {
            socket_path: root.join("s"),
            db_path: root.join("b.db"),
            log_path: root.join("log"),
            ..DaemonConfig::default()
        };
        let (tx, _) = mpsc::channel();
        let mut st = DaemonState::new(&config, tx).unwrap();
        st.registry = AgentRegistry::from_json(
            &serde_json::json!({"machines":{"origin177":{
            "enabled":true,"worker_id":"worker-origin177","provisioning_revision":1,
            "credential_fingerprint":format!("{:x}",sha2::Sha256::digest(b"fixture177-worker-key")),
            "allowed_agent_types":[],"mappings":{}}}})
            .to_string(),
            "/tmp/origin177-fixture-registry.json",
        )
        .unwrap();
        st.connection_roles
            .insert("worker177".into(), ConnectionRole::Worker);
        st.peer_uids
            .insert("worker177".into(), unsafe { libc::geteuid() });
        let shared = Arc::new(Mutex::new(st));
        let identity = WorkerIdentity {
            machine_id: "origin177".into(),
            worker_id: "worker-origin177".into(),
            provisioning_revision: 1,
        };
        let reply = handle_worker_message(
            &shared,
            "worker177",
            WrapperToDaemon::WorkerHelloWithCapabilities {
                hello: WorkerHelloWithCapabilities {
                    hello: WorkerHello {
                        version: 1,
                        identity: identity.clone(),
                        credential: IdentityCredential::new("fixture177-worker-key".into()),
                        build: "fixture177".into(),
                        capability: ClientCapability::RemoteWorkerV1,
                    },
                    capabilities: vec![WorkerCapability::OriginProofV1],
                },
            },
        );
        assert!(matches!(
            reply,
            Some(DaemonToWrapper::WorkerAcceptedWithCapabilities { .. })
        ));
        (shared, root, identity)
    }
    #[test]
    fn mobile177_foreign_project_requires_worker_proof_and_matching_primary_host() {
        use std::io::BufRead;
        let (shared,root,_identity)=fixture();
        let thread="foreign-project177";
        let owner=crate::t3code::stable_uuid(thread);
        let instance=crate::t3code::stable_uuid(&format!("instance:{thread}"));
        let (primary,_primary_peer)=UnixStream::pair().unwrap();
        {let mut st=shared.lock().unwrap();
            st.connections.insert("primary177".into(),Arc::new(Mutex::new(BufWriter::new(primary))));
            assert!(matches!(handle_register_with_channel("primary177",2,"fixture".into(),owner.clone(),
                Some("origin177".into()),Some("t3code".into()),ChannelReport::Known("ssh-unix".into()),
                Some(PresenceMode::Cli),None,Some("fixture".into()),Some(instance.clone()),None,false,Some(false),&mut st,false),
                DaemonToWrapper::Registered {..}));
        }
        let (writer,peer)=UnixStream::pair().unwrap();
        shared.lock().unwrap().connections.insert("worker177".into(),Arc::new(Mutex::new(BufWriter::new(writer))));
        let for_worker=shared.clone();
        let fake=std::thread::spawn(move||{
            peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut reader=std::io::BufReader::new(peer);
            for _ in 0..2 {
            let mut line=String::new();reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::OriginProofRequest {request}=serde_json::from_str(&line).unwrap() else {panic!("origin request")};
            assert!(matches!(&request.request,OriginProofOperation::CommunicationProject {source:CommunicationProjectSource::T3,..}));
            let response=crate::origin_proof::execute(&request,std::path::Path::new("/unused"));
            assert!(matches!(&response,OriginProofResult::CommunicationProject {project:Some(_)}));
            receive(&mut for_worker.lock().unwrap(),"worker177",OriginProofResponse {version:1,
                correlation_id:request.correlation_id,connection_epoch:request.connection_epoch,worker:request.worker,response});
            }
        });
        assert!(matches!(announce_communication_project(&shared,"primary177",root.to_str().unwrap(),
            CommunicationProjectSource::Git,"origin177",None),DaemonToWrapper::ProjectContextResult {project:None,..}));
        let result=announce_communication_project(&shared,"primary177",root.to_str().unwrap(),
            CommunicationProjectSource::T3,"origin177",None);
        let human_project=human_request_project(&shared,thread,root.to_str().unwrap());
        fake.join().unwrap();
        assert!(human_project.is_some());
        assert!(matches!(result,DaemonToWrapper::ProjectContextResult {project:Some(_),..}));
        assert!(handle_wrapper_message("primary177",WrapperToDaemon::T3ThreadBindingFact {
            version:HUMAN_THREAD_VIEW_VERSION,t3_thread_id:thread.into()},&shared).is_none());
        {let mut st=shared.lock().unwrap();let project=agent_communication_project(&st,&owner);
            assert_eq!(human_authority_guard(&st,thread,&project),Ok(("primary177".into(),owner.clone())));
            let mut forged=project.clone().unwrap();forged.host="other177".into();
            assert!(human_authority_guard(&st,thread,&Some(forged)).is_err());
            let local=st.host.clone();
            st.presences.get_mut(&instance).unwrap().host=local.clone();
            assert!(human_authority_guard(&st,thread,&project).is_err());
            st.presences.get_mut(&instance).unwrap().host="origin177".into();
            st.conn_hosts.insert("primary177".into(),local);
            assert!(human_authority_guard(&st,thread,&project).is_err());
            st.conn_hosts.insert("primary177".into(),"origin177".into());
            remote_worker_disconnected(&mut st,"worker177");
            assert!(human_authority_guard(&st,thread,&project).is_err());
            assert_eq!(agent_communication_project(&st,&owner),project,"known scope must not become unknown on worker loss");
            st.remote_workers.clear();
        }
        let retained=announce_communication_project(&shared,"primary177",root.to_str().unwrap(),
            CommunicationProjectSource::T3,"origin177",None);
        assert!(matches!(retained,DaemonToWrapper::ProjectContextResult {project:Some(_),..}));
        {let st=shared.lock().unwrap();let known=agent_communication_project(&st,&owner).unwrap();
            let other=CommunicationProject {host:known.host.clone(),root:"/other-project177".into()};
            assert!(crate::communication::project_scope(Some(&known),Some(&other),"recipient177",None).is_err());
            assert!(st.t3_thread_bindings.get("primary177").is_none());
            assert!(human_authority_guard(&st,thread,&Some(known)).is_err());
        }
        drop(shared);std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn mobile177_origin_response_rejects_other_worker_epoch_and_consumed_nonce() {
        let (shared, root, identity) = fixture();
        let (tx, rx) = mpsc::sync_channel(1);
        let epoch = shared.lock().unwrap().remote_workers["origin177"]
            .epoch
            .clone();
        let response = OriginProofResponse {
            version: 1,
            correlation_id: Uuid::new_v4().to_string(),
            connection_epoch: epoch.clone(),
            worker: identity.clone(),
            response: OriginProofResult::Sources {
                permission_digest: "a".repeat(64),
                source_error: None,
            },
        };
        {
            let mut st = shared.lock().unwrap();
            st.pending_origin_proofs.insert(
                response.correlation_id.clone(),
                Pending {
                    conn_id: "worker177".into(),
                    epoch: epoch.clone(),
                    worker: identity.clone(),
                    reply: tx,
                },
            );
            let mut old = response.clone();
            old.connection_epoch = "previous".into();
            assert!(receive(&mut st, "worker177", old).is_some());
            assert!(receive(&mut st, "other", response.clone()).is_some());
            let mut other = response.clone();
            other.worker.machine_id = "other177".into();
            assert!(receive(&mut st, "worker177", other).is_some());
            assert!(rx.try_recv().is_err());
            assert!(receive(&mut st, "worker177", response.clone()).is_none());
            assert!(matches!(
                rx.try_recv(),
                Ok(OriginProofResult::Sources { .. })
            ));
            assert!(receive(&mut st, "worker177", response).is_none());
            assert!(rx.try_recv().is_err());
        }
        drop(shared);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn mobile177_origin_real_wire_correlates_once_and_timeout_drops_pending() {
        let (shared, root, identity) = fixture();
        let (writer, peer) = UnixStream::pair().unwrap();
        shared.lock().unwrap().connections.insert(
            "worker177".into(),
            Arc::new(Mutex::new(BufWriter::new(writer))),
        );
        let for_worker = shared.clone();
        let fake = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(peer);
            use std::io::BufRead;
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::OriginProofRequest { request } =
                serde_json::from_str(&line).unwrap()
            else {
                panic!("origin frame");
            };
            assert_eq!(request.worker, identity);
            request.validate_shape().unwrap();
            let response = OriginProofResponse {
                version: 1,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                response: OriginProofResult::Attested {
                    proof: OriginSessionProof {
                        version: 1,
                        environment_id: "environment177".into(),
                        thread_id: "thread177".into(),
                        provider_session_id: "session177".into(),
                        provider_instance_id: "instance177".into(),
                        permissions: None,
                    },
                },
            };
            assert!(
                handle_worker_message(
                    &for_worker,
                    "worker177",
                    WrapperToDaemon::OriginProofResponse { response }
                )
                .is_none()
            );
        });
        let operation = OriginProofOperation::Attest {
            expected_thread: "thread177".into(),
            proof: NativePermissionProof {
                endpoint: "http://127.0.0.1:1/mcp".into(),
                authorization: IdentityCredential::new("Bearer fixture177".into()),
            },
        };
        assert!(matches!(
            exchange(
                &shared,
                "origin177",
                operation.clone(),
                Duration::from_secs(2)
            ),
            Ok(OriginProofResult::Attested { .. })
        ));
        fake.join().unwrap();
        // Transport indisponible : aucune attente durable et aucune remise métier.
        assert!(exchange(&shared, "origin177", operation, Duration::from_millis(20)).is_err());
        assert!(shared.lock().unwrap().pending_origin_proofs.is_empty());
        assert!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .pending()
                .unwrap()
                .is_empty()
        );
        drop(shared);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires the documented private two-host origin fixture"]
    fn mobile177_origin_live_two_hosts_no_task_or_provider_launch(){
        use std::io::BufRead;
        let file=std::env::var_os("BRIDGET177_ORIGIN_LIVE").expect("explicit private fixture file required");
        let file=PathBuf::from(file);
        crate::environment::validate_state_file(&file,false).unwrap();
        let config:serde_json::Value=serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        let (shared,root,_identity)=fixture();
        // The live worker must authenticate itself on this real Unix/SSH connection.
        {let mut st=shared.lock().unwrap();st.remote_workers.clear();st.connection_roles.clear();st.peer_uids.clear();}
        let socket=file.parent().unwrap().join("s");
        let listener=std::os::unix::net::UnixListener::bind(&socket).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(file.parent().unwrap().join("ready"),"listening").unwrap();
        let state=shared.clone();
        let witness=file.parent().unwrap().join("wire-events.jsonl");
        let (accepted,ready)=mpsc::sync_channel(1);
        let service=std::thread::spawn(move||{
            let (stream,_)=listener.accept().unwrap();
            let uid=unix_peer_uid(&stream).unwrap();
            let writer=Arc::new(Mutex::new(BufWriter::new(stream.try_clone().unwrap())));
            {
                let mut st=state.lock().unwrap();
                st.connections.insert("live177".into(),writer.clone());
                st.peer_uids.insert("live177".into(),uid);
            }
            let mut reader=std::io::BufReader::new(stream);
            loop{
                let mut line=String::new();
                if reader.read_line(&mut line).unwrap_or(0)==0{break;}
                let frame:WrapperToDaemon=serde_json::from_str(&line).unwrap();
                let kind=serde_json::to_value(&frame).unwrap()["type"].clone();
                let response=handle_wrapper_message("live177",frame,&state);
                {use std::io::Write;let out=std::fs::OpenOptions::new().create(true).append(true).open(&witness).unwrap();
                    let value=serde_json::json!({"in":kind,"out":response.as_ref().map(|r|serde_json::to_value(r).unwrap()["type"].clone())});
                    writeln!(&out,"{value}").unwrap();}
                if let Some(response)=response {
                    let connected=matches!(&response,DaemonToWrapper::WorkerAcceptedWithCapabilities {..});
                    push_control_message_until(&writer,&response,Instant::now()+Duration::from_secs(5)).unwrap();
                    if connected {let _=accepted.try_send(());}
                }
            }
            remote_worker_disconnected(&mut state.lock().unwrap(),"live177");
        });
        ready.recv_timeout(Duration::from_secs(90)).expect("origin worker was not connected");
        let private=NativePermissionProof {
            endpoint:config["endpoint"].as_str().unwrap().into(),
            authorization:IdentityCredential::new(config["authorization"].as_str().unwrap().into())};
        let expected=config["expected_thread"].as_str().unwrap();
        let result=exchange(&shared,"origin177",OriginProofOperation::Attest {expected_thread:expected.into(),proof:private.clone()},Duration::from_secs(35)).unwrap();
        let OriginProofResult::Attested {proof}=result else {panic!("attestation expected");};
        assert_eq!(proof.thread_id,expected);assert_eq!(proof.version,2);proof.validate().unwrap();
        let parent=proof.permissions.unwrap();
        assert_eq!(parent.cwd,config["parent_cwd"].as_str().unwrap());
        assert!(!std::path::Path::new(&parent.cwd).exists(),"the coordinator must not have this origin path");
        let source=WorkerSourceBase {cwd:parent.cwd.clone(),expected_base_commit:Some(config["expected_base_commit"].as_str().unwrap().into()),
            workspace:Some(WorkspaceRequest::Shared {}),workload:AutoDelegateWorkload::General};
        let result=exchange(&shared,"origin177",OriginProofOperation::Sources {parent:parent.clone(),source:Some(source)},Duration::from_secs(30)).unwrap();
        assert!(matches!(result,OriginProofResult::Sources {source_error:None,..}));
        if let Some(target)=config["prepare_machine"].as_str() {
            let prepare=OriginProofOperation::PrepareProject {parent:parent.clone(),
                source:WorkerSourceBase {cwd:parent.cwd.clone(),
                    expected_base_commit:Some(config["expected_base_commit"].as_str().unwrap().into()),
                    workspace:Some(WorkspaceRequest::Shared {}),workload:AutoDelegateWorkload::General},
                target_machine:target.into(),project_id:Some("fixture177".into())};
            for _ in 0..2 {
                let result=exchange(&shared,"origin177",prepare.clone(),Duration::from_secs(300)).unwrap();
                assert!(matches!(result,OriginProofResult::ProjectPrepared {preparation_error:None,..}),
                    "private origin preparation must succeed, including an exact replay");
            }
        }
        assert!(exchange(&shared,"origin177",OriginProofOperation::Attest {expected_thread:"other-thread177".into(),proof:private},Duration::from_secs(35)).is_err());
        {
            let st=shared.lock().unwrap();
            assert!(st.delegation_store.tasks().unwrap().is_empty());
            assert!(st.pending_origin_proofs.is_empty());
            let writer=st.connections["live177"].lock().unwrap();
            writer.get_ref().shutdown(std::net::Shutdown::Both).unwrap();
        }
        service.join().unwrap();
        std::fs::remove_file(socket).unwrap();
        std::fs::write(file.parent().unwrap().join("passed"),"origin177_two_hosts_ok").unwrap();
        drop(shared);std::fs::remove_dir_all(root).unwrap();
    }

}
