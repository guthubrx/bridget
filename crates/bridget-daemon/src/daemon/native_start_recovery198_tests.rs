//! Tests des gardes et de la clôture, sans fournisseur ni store réel.
use super::*;
use super::super::tests::{cleanup, continuation172_parent, fixture as parent_fixture, tache_151};
use bridget_transport::protocol::{WorkerContactState, WorkerResponse, REMOTE_WORKER_VERSION};

fn fixture() -> (Arc<Mutex<DaemonState>>, DaemonConfig, Task, AutoAdmissionReceipt, ProviderPermissions) {
    let (mut st,config,_rx,_reader)=parent_fixture("r198");
    let mut task=tache_151(&mut st,"start198");
    task.remote_prepared=true;
    task.remote_placement=Some(bridget_transport::protocol::RemotePlacement {
        machine_id:"cartae".into(),worker_id:"worker-cartae".into(),provisioning_revision:1,
        project_id:"project".into(),mapping_revision:1,origin_host:st.host.clone(),origin_cwd:task.cwd.clone(),
        source_root_identity:"git:fixture162".into(),relative_cwd:".".into(),execution_cwd:"/tmp".into(),
        base_commit:Some("a".repeat(40)),definition_digest:task.definition.digest.clone(),permission_digest:"b".repeat(64),
    });
    if let NativeDelegationRequest::Delegate { machine_id, project_id, .. } = &mut task.request {
        *machine_id = Some("cartae".into());
        *project_id = Some("project".into());
    }
    if let NativeDelegationRequest::Delegate { workspace, .. } = &mut task.request {
        *workspace=Some(WorkspaceRequest::Shared {});
    }
    let root = st.fixture_root.as_ref().unwrap().0.clone();
    st.registry = AgentRegistry::from_json(&json!({
        "agents":{"native-test":{"command":"/bin/sh","args":["app-server"],
            "protocol":"codex_app_server","forbidden_env":[],"capabilities":{
                "execution_paths":["codex_app_server"],"models":{"custom/model":{"efforts":["high"]}}}}},
        "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
            "credential_fingerprint":"c".repeat(64),"allowed_agent_types":["native-test"],
            "mappings":{"project":{"source_host":st.host,"source_root":task.cwd,
                "remote_root":"/tmp","expected_git_identity":"git:fixture162","mapping_revision":1}}}}
    }).to_string(), &root.join("registry198.json")).unwrap();
    let source_cwd=task.cwd.clone();
    task.cwd=task.remote_placement.as_ref().unwrap().execution_cwd.clone();
    task.remote_placement.as_mut().unwrap().origin_host=st.host.clone();
    task.remote_placement.as_mut().unwrap().base_commit=Some("a".repeat(40));
    let mut parent = continuation172_parent();
    parent.cwd=source_cwd;
    parent.provider_instance_id=task.owner_instance.clone();
    st.native_permission_facts.insert("conn-1".into(),(parent.clone(),None));
    task.permission_snapshot = Some(crate::native_permissions::snapshot(
        &task.owner, &task.owner_instance, parent.clone(),
        json!({"kind":"codex","approval_policy":"never","sandbox_policy":{"type":"readOnly"}}),
        Path::new(&task.cwd)));
    // Simuler une naissance jamais envoyée avec un bail déjà expiré.
    // Les temps sont passés explicitement aux APIs Fleet, sans changer l'horloge.
    task.remote_placement.as_mut().unwrap().definition_digest=task.definition.digest.clone();
    task.remote_placement.as_mut().unwrap().permission_digest=format!("{:x}",sha2::Sha256::digest(
        serde_json::to_vec(&task.permission_snapshot.as_ref().unwrap().child_policy).unwrap()));
    let issued=unix_timestamp()-600;
    let order=FleetSpawnOrder {
        posture:None,agent_type:"native-test".into(),project:None,requested_name:Some(task.child.clone()),
        cwd:PathBuf::from(&task.cwd),persistent:true,command_id:task.task_id.clone(),
        issued_at:issued,deadline_at:issued+300,
        ownership:Some(SpawnOwnership {parent_instance_id:task.owner_instance.clone(),parent_execution_id:None,
            objective_id:None,delegation_id:Some(task.task_id.clone()),project:None,role:"native_delegate".into(),
            max_children:Some(16),max_depth:Some(8)}),remote_placement:task.remote_placement.clone(),
    };
    let submission=if st.fleet.desired_entry(&task.child).unwrap().is_some() {
        st.fleet.request_relaunch(&order,issued)
    } else {st.fleet.request_spawn(&order,issued)}.unwrap();
    let crate::fleet::SpawnSubmission::Start(lease)=submission else {panic!("lease198")};
    st.fleet.mark_starting(&lease,issued,&task.definition).unwrap();
    task.child_instance=Some(lease.instance_id);task.launch_issued_at=Some(issued);
    task.state = "starting".into();
    task.error = Some("remote_start_failed:remote_start:ResourceLimit".into());
    task.mission_issued_at = None;
    task.mission_deadline_at = None;
    task.remote_result_slots.clear();
    task.remote_result_receipts.clear();
    st.delegation_store.save(&task).unwrap();
    task = st.delegation_store.get(&task.task_id).unwrap().unwrap();
    let request_id = match &task.request { NativeDelegationRequest::Delegate {request_id,..} => request_id.clone(), _ => unreachable!() };
    let now = unix_timestamp();
    let receipt = AutoAdmissionReceipt {
        owner:task.owner.clone(),request_id,canonical:vec![],metadata:b"{}".to_vec(),metadata_version:2,
        state:AutoAdmissionState::Admitted,coordinator_epoch:st.coordinator_epoch,revision:0,
        policy_revision:1,policy_digest:"d".repeat(64),candidate_id:Some("c001".into()),
        delegate_canonical:Some(serde_json::to_vec(&task.request).unwrap()),mode:None,reason:None,
        task_id:Some(task.task_id.clone()),refusal_code:None,created_at:now,updated_at:now,
    };
    st.remote_workers.insert("cartae".into(), super::super::super::RemoteWorkerConnection {
        identity:remote_worker_identity(task.remote_placement.as_ref().unwrap()),
        conn_id:"worker198".into(),epoch:"epoch198".into(),capabilities:vec![WorkerCapability::StartRecoveryV1],
        last_contact:Instant::now(),last_contact_at:now,contact:WorkerContactState::Connected,
    });
    (Arc::new(Mutex::new(st)),config,task,receipt,parent)
}

fn saved(state:&Arc<Mutex<DaemonState>>, id:&str)->Value {
    serde_json::to_value(state.lock().unwrap().delegation_store.get(id).unwrap().unwrap()).unwrap()
}

#[test]
fn spec198_gather_accepts_current_equivalent_parent_without_mutating() {
    let (state,config,task,receipt,mut parent)=fixture();
    let before=saved(&state,&task.task_id);
    parent.run_id="new-run198".into();parent.revision+=1;
    assert!(matches!(gather(&state,"conn-1",&task.task_id,&receipt,Some(&parent),None),Ok(Ok(_))));
    assert_eq!(saved(&state,&task.task_id),before);
    drop(state);cleanup(&config);
}

#[test]
fn spec198_gather_foreign_owner_and_changed_policy_refuse_without_plan() {
    for foreign in [true,false] {
        let (state,config,mut task,receipt,mut parent)=fixture();
        if foreign {task.owner="foreign198".into();state.lock().unwrap().delegation_store.save(&task).unwrap();}
        else {parent.provider_policy["approval_policy"]=json!("on-request");}
        let before=saved(&state,&task.task_id);
        let code=gather(&state,"conn-1",&task.task_id,&receipt,Some(&parent),None).err().unwrap();
        assert_eq!(code,if foreign {"task_unavailable"} else {"permission_changed"});
        assert_eq!(saved(&state,&task.task_id),before);
        drop(state);cleanup(&config);
    }
}

#[test]
fn spec198_halt_revocation_and_old_worker_refuse_without_plan() {
    for case in 0..3 {
        let (state,config,task,receipt,parent)=fixture();
        {
            let mut st=state.lock().unwrap();
            match case {
                0 => {st.delegation_store.set_delegation_halt("halt198",true,"cli",Some("fixture"),||Ok(0),unix_timestamp()).unwrap();},
                1 => {st.delegation_store.grant(&task.owner_instance,Path::new(&task.cwd),SpawnPosture::Development,true).unwrap();},
                _ => {st.remote_workers.get_mut("cartae").unwrap().capabilities.clear();},
            }
        }
        let before=saved(&state,&task.task_id);
        let code=gather(&state,"conn-1",&task.task_id,&receipt,Some(&parent),None).err().unwrap();
        assert_eq!(code,match case {0=>"delegations_halted",1=>"delegation_grant_required",_=>"start_recovery_capability_required"});
        assert_eq!(saved(&state,&task.task_id),before);
        drop(state);cleanup(&config);
    }
}

#[test]
fn spec198_unknown_start_or_release_never_enters_recovery() {
    let (state,config,mut task,receipt,_)=fixture();
    for error in ["remote_start_failed:remote_start:WorkerUnavailable","remote_start_failed:remote_release:ResourceLimit", "remote_start_failed:remote_start:ResourceLimit:cause=preparation_metadata"] {
        task.error=Some(error.into());
        assert!(!wanted(&task,&receipt));
        let view=task_view(&task,None,true);
        assert!(!view["error_recovery"].as_str().unwrap_or("").contains("ferme d’abord"));
    }
    task.error=Some("remote_start_failed:remote_start:ResourceLimit".into());
    let mut changed=receipt.clone();changed.delegate_canonical=Some(b"{}".to_vec());
    assert!(!wanted(&task,&changed));
    drop(state);cleanup(&config);
}

#[test]
fn spec198_persisted_plan_replays_and_cancellation_blocks_fence() {
    let (state,config,task,_,_)=fixture();
    persist_plan(&state,&task.task_id).unwrap();
    let before=saved(&state,&task.task_id);
    persist_plan(&state,&task.task_id).unwrap();
    assert_eq!(saved(&state,&task.task_id),before);
    {
        let st=state.lock().unwrap();let mut cancelled=st.delegation_store.get(&task.task_id).unwrap().unwrap();
        cancelled.state="cancelling".into();st.delegation_store.save(&cancelled).unwrap();
    }
    let after=saved(&state,&task.task_id);
    let result=fresh(&state,&task.task_id,&[StartRecoveryPhase::Planned]);
    assert_eq!(result.err().as_deref(),Some("start_recovery_task_changed"));
    assert_eq!(saved(&state,&task.task_id),after);
    drop(state);cleanup(&config);
}

#[test]
fn spec198_fence_ack_moves_same_plan_before_any_new_lease() {
    let (state,config,task,_,parent)=fixture();
    persist_plan(&state,&task.task_id).unwrap();
    let original=saved(&state,&task.task_id);
    let (socket,peer)=UnixStream::pair().unwrap();peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    state.lock().unwrap().connections.insert("worker198".into(),Arc::new(Mutex::new(std::io::BufWriter::new(socket))));
    let responder=state.clone();let id=task.task_id.clone();
    let listener=thread::spawn(move || {
        let mut reader=std::io::BufReader::new(peer);let mut line=String::new();
        std::io::BufRead::read_line(&mut reader,&mut line).unwrap();
        let DaemonToWrapper::WorkerRequest {request}=serde_json::from_str(&line).unwrap() else {panic!("worker198 request")};
        assert!(matches!(request.request,WorkerRequestOperation::FenceUnstarted{..}));
        assert_eq!(saved(&responder,&id)["start_recovery"]["phase"],"planned");
        let response=WorkerResponse{version:REMOTE_WORKER_VERSION,correlation_id:request.correlation_id,
            connection_epoch:request.connection_epoch,worker:request.worker,command:request.command,
            response:WorkerResponseOperation::StartFenced{}};
        assert!(super::super::super::handle_worker_message(&responder,"worker198",WrapperToDaemon::WorkerResponse{response:response.clone()}).is_none());
        response
    });
    let ctx=Context{owner:task.owner.clone(),parent,worker:remote_worker_identity(task.remote_placement.as_ref().unwrap())};
    fence(&state,&task.task_id,&ctx).unwrap();let response=listener.join().unwrap();
    let before_late=saved(&state,&task.task_id);
    assert!(super::super::super::handle_worker_message(&state,"worker198",WrapperToDaemon::WorkerResponse{response:response.clone()}).is_none());
    assert_eq!(saved(&state,&task.task_id),before_late);
    let mut stale=response.clone();stale.connection_epoch="stale198".into();
    assert!(super::super::super::handle_worker_message(&state,"worker198",WrapperToDaemon::WorkerResponse{response:stale}).is_some());
    let mut foreign=response;foreign.command.instance_id="foreign198".into();
    assert!(super::super::super::handle_worker_message(&state,"worker198",WrapperToDaemon::WorkerResponse{response:foreign}).is_some());
    let after=saved(&state,&task.task_id);
    assert_eq!(after["start_recovery"]["phase"],"fenced");
    assert_eq!(after["child_instance"],original["child_instance"]);
    assert_eq!(after["mission"],original["mission"]);
    assert_eq!(after["round"],original["round"]);
    assert_eq!(state.lock().unwrap().delegation_store.tasks().unwrap().len(),1);
    drop(state);cleanup(&config);
}

#[test]
fn spec198_pause_blocks_before_plan_or_worker_request() {
    let (state,config,task,receipt,parent)=fixture();
    {
        let st=state.lock().unwrap();
        let current=crate::referent_control::read(st.store.connection()).unwrap();
        crate::referent_control::set(st.store.connection(),crate::referent_control::ControlMutation {
            command_id:"pause198",expected_generation:current.generation,paused:Some(true),
            auto_objectives_cap:None,reason:Some("fixture"),actor:"cli",now:unix_timestamp(),
            agent_posture:None,auto_reassignment:None,
        }).unwrap().unwrap();
    }
    let before=saved(&state,&task.task_id);
    assert_eq!(gather(&state,"conn-1",&task.task_id,&receipt,Some(&parent),None).err().as_deref(),Some("referent_paused"));
    assert_eq!(saved(&state,&task.task_id),before);
    assert!(state.lock().unwrap().pending_worker_responses.is_empty());
    drop(state);cleanup(&config);
}

#[test]
fn spec198_local_capacity_keeps_master_cause_and_sends_nothing() {
    use bridget_transport::protocol::{WorkerStart,MAX_WORKER_FRAME_BYTES,MAX_WORKER_STORE_ENTRIES};
    for oversized in [false,true] {
        let (state,config,task,_,_)=fixture();
        let (socket,peer)=UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        let command=remote_command_identity(&state.lock().unwrap(),&task).unwrap();
        let worker=remote_worker_identity(task.remote_placement.as_ref().unwrap());
        let mut policy=task.permission_snapshot.as_ref().unwrap().child_policy.clone();
        if oversized {policy["oversized_fixture"]=json!("x".repeat(MAX_WORKER_FRAME_BYTES));}
        let start=WorkerStart {prepared_token:"token198".into(),child_id:task.child.clone(),
            mission_id:task.mission.clone(),forward_child_requests:false,
            placement:task.remote_placement.clone().unwrap(),definition:task.definition.clone(),
            child_policy:policy,effective_posture:SpawnPosture::Discovery,deadline_at:unix_timestamp()+300};
        {
            let mut st=state.lock().unwrap();
            st.connections.insert("worker198".into(),Arc::new(Mutex::new(std::io::BufWriter::new(socket))));
            if !oversized {
                for n in 0..MAX_WORKER_STORE_ENTRIES {
                    let (tx,_rx)=mpsc::sync_channel(1);
                    st.pending_worker_responses.insert(format!("pending198-{n}"),super::super::super::PendingWorkerResponse {
                        conn_id:"worker198".into(),epoch:"epoch198".into(),worker:worker.clone(),
                        command:command.clone(),reply:tx,
                    });
                }
            }
        }
        assert_eq!(remote_worker_roundtrip(&state,&worker,&command,
            WorkerRequestOperation::StartDetailed {start},Instant::now()+Duration::from_secs(1)).err(),Some(WorkerRefusal::ResourceLimit));
        assert_eq!(super::super::super::take_local_roundtrip_limit(),Some(if oversized {"master_request_frame"} else {"master_pending"}));
        let mut reader=std::io::BufReader::new(peer);let mut line=String::new();
        assert!(std::io::BufRead::read_line(&mut reader,&mut line).is_err());
        assert!(line.is_empty());drop(state);cleanup(&config);
    }
}

#[test]
fn spec198_restart_preserves_the_pending_recovery_and_never_spawns() {
    let (state,config,task,_,_)=fixture();
    persist_plan(&state,&task.task_id).unwrap();
    let before=saved(&state,&task.task_id);
    {
        let mut st=state.lock().unwrap();
        assert!(reserve_managed_recoveries(&mut st,unix_timestamp()).unwrap().is_empty());
    }
    assert_eq!(saved(&state,&task.task_id),before);
    drop(state);cleanup(&config);
}

#[test]
fn spec198_recovery_cannot_reuse_an_expired_new_lease_as_current_time() {
    use bridget_transport::protocol::WorkerPrepared;
    let (state,config,task,_,parent)=fixture();
    persist_plan(&state,&task.task_id).unwrap();
    set_phase(&state,&task.task_id,StartRecoveryPhase::Planned,StartRecoveryPhase::Fenced).unwrap();
    invalidate_previous(&state,&task.task_id).unwrap();
    let plan={
        let st=state.lock().unwrap();let mut task=st.delegation_store.get(&task.task_id).unwrap().unwrap();
        let plan=task.start_recovery.as_mut().unwrap();plan.phase=StartRecoveryPhase::Prepared;
        plan.issued_at=unix_timestamp()-600;let plan=plan.clone();st.delegation_store.save(&task).unwrap();plan
    };
    let prepared=WorkerPrepared {workspace:None,placement:task.remote_placement.clone().unwrap(),
        token:"prepared198".into(),definition:task.definition.clone(),
        child_policy:task.permission_snapshot.as_ref().unwrap().child_policy.clone(),
        effective_posture:task.effective_posture.unwrap(),prepared_at:unix_timestamp()};
    let result=start_remote_task_with(&state,&task.task_id,&prepared,&parent,"conn-1",None,None,Some(&plan));
    assert!(result.is_err());
    let st=state.lock().unwrap();
    assert!(st.managed_spawns.is_empty(),"une échéance passée ne crée aucun bootstrap");
    assert!(st.pending_worker_responses.is_empty(),"aucun Start ne doit être envoyé");
    drop(st);drop(state);cleanup(&config);
}

#[test]
fn spec198_expired_original_lease_resumes_same_task_once_through_native_responses() {
    use bridget_transport::protocol::WorkerPrepared;
    let (state,config,task,receipt,parent)=fixture();
    let original=saved(&state,&task.task_id);
    let old=remote_command_identity(&state.lock().unwrap(),&task).unwrap();
    assert!(lease_of(&state.lock().unwrap(),&old).unwrap().deadline_at<unix_timestamp());
    let (socket,peer)=UnixStream::pair().unwrap();peer.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    state.lock().unwrap().connections.insert("worker198".into(),Arc::new(Mutex::new(std::io::BufWriter::new(socket))));
    let responder=state.clone();let expected=task.clone();
    let listener=thread::spawn(move || {
        let mut reader=std::io::BufReader::new(peer);let mut started=None;let mut calls=Vec::new();
        for _ in 0..5 {
            let mut line=String::new();
            match std::io::BufRead::read_line(&mut reader,&mut line) {
                Ok(0) | Err(_) => {eprintln!("séquence198 interrompue après {calls:?}");return calls;},
                Ok(_) => {},
            }
            let DaemonToWrapper::WorkerRequest {request}=serde_json::from_str(&line).unwrap() else {panic!("requête native198")};
            request.validate().unwrap();
            let operation=match &request.request {
                WorkerRequestOperation::FenceUnstarted {..} => {
                    calls.push("fence");assert_eq!(request.command,old);
                    WorkerResponseOperation::StartFenced{}
                },
                WorkerRequestOperation::PrepareFenced {previous,placement,..} => {
                    calls.push("prepare");assert_eq!(previous,&old);
                    assert_eq!(placement,expected.remote_placement.as_ref().unwrap());
                    WorkerResponseOperation::Prepared {prepared:WorkerPrepared {workspace:None,
                        placement:placement.clone(),token:"prepared198".into(),definition:expected.definition.clone(),
                        child_policy:expected.permission_snapshot.as_ref().unwrap().child_policy.clone(),
                        effective_posture:expected.effective_posture.unwrap(),prepared_at:unix_timestamp()}}
                },
                WorkerRequestOperation::StartDetailed {start} => {
                    calls.push("start");assert_ne!(request.command.command_id,old.command_id);
                    assert_ne!(request.command.instance_id,old.instance_id);
                    assert_eq!(request.command.task_id,old.task_id);
                    assert!(start.deadline_at>unix_timestamp());
                    assert_eq!(start.child_id,expected.child);assert_eq!(start.mission_id,expected.mission);
                    started=Some((request.command.clone(),start.clone()));
                    WorkerResponseOperation::ReadyToRelease {ready_at:unix_timestamp()}
                },
                WorkerRequestOperation::ReserveResultSlot {slot}
                | WorkerRequestOperation::ReserveResultSlotUntil {slot,..} => {
                    calls.push("slot");
                    WorkerResponseOperation::ResultSlotReserved {slot:slot.clone(),slot_id:"slot198".into()}
                },
                WorkerRequestOperation::Release {permit} => {
                    calls.push("release");let (command,start)=started.as_ref().unwrap();
                    assert_eq!(permit.command_digest,crate::remote_worker::worker_command_digest(&request.worker,command,start).unwrap());
                    WorkerResponseOperation::Released{}
                },
                _ => panic!("opération imprévue198"),
            };
            let response=WorkerResponse {version:REMOTE_WORKER_VERSION,correlation_id:request.correlation_id,
                connection_epoch:request.connection_epoch,worker:request.worker,command:request.command,response:operation};
            assert!(super::super::super::handle_worker_message(&responder,"worker198",WrapperToDaemon::WorkerResponse{response}).is_none());
        }
        calls
    });
    let result=resume(&state,"conn-1",&task.task_id,&receipt,Some(&parent),None,None);
    if let Err(code)=&result {eprintln!("refus de la fixture198 avant fin du parcours : {code}");}
    let calls=listener.join().unwrap();
    assert!(result.as_ref().is_ok_and(|value|value["error"].is_null()),"réponse198={result:?}, opérations={calls:?}");
    assert_eq!(calls,vec!["fence","prepare","start","slot","release"]);
    let after=saved(&state,&task.task_id);
    for key in ["task_id","owner","child","mission","round","cwd","request","workspace"] {assert_eq!(after[key],original[key],"{key}");}
    assert_eq!(after["start_recovery"]["phase"],"done");
    assert_ne!(after["child_instance"],original["child_instance"]);
    assert_eq!(state.lock().unwrap().delegation_store.tasks().unwrap().len(),1);
    let replay=resume(&state,"conn-1",&task.task_id,&receipt,Some(&parent),None,None).unwrap();
    assert_eq!(replay["task_id"],json!(task.task_id));
    assert_eq!(saved(&state,&task.task_id),after);
    drop(state);cleanup(&config);
}
