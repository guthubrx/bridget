    use super::*;
    use bridget_transport::protocol::{AutoDelegateWorkload, CandidateProbeV2Candidate, ProviderPermissions};
    use bridget_transport::protocol::{RemoteResultDeferredReason, RemoteResultDisposition};
    use std::os::unix::fs::symlink;





    #[test]
    fn spec221_probe_read_does_not_block_native_command_or_release_epoch_slot_early() {
        let fixture=Fixture::new();
        let mut runtime=runtime_fixture(&fixture);
        let (stream,mut peer)=UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let writer=Arc::new(Mutex::new(stream));
        let slot=Arc::new(AtomicBool::new(false));
        let disconnected=Arc::new(AtomicBool::new(false));
        let (started,entered)=mpsc::channel();let (release,blocked)=mpsc::channel();let (done,finished)=mpsc::channel();
        let permit=ProbeReadPermit::claim(&slot).unwrap();
        let reply=WrapperToDaemon::Heartbeat;
        spawn_probe_read(permit,&writer,disconnected.clone(),WrapperToDaemon::Heartbeat,move || {
            started.send(()).unwrap();blocked.recv_timeout(Duration::from_secs(5)).unwrap();
            done.send(()).unwrap();reply
        }).unwrap();
        entered.recv_timeout(Duration::from_secs(1)).unwrap();
        let before=runtime.store.commands().unwrap();
        let request=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"status-while-probe".into(),
            connection_epoch:"probe-epoch".into(),worker:worker(),command:command(),
            request:WorkerRequestOperation::ExecutionStatus {}};
        let response=runtime.handle(&request,"probe-epoch",&disconnected).unwrap_or_else(|reason|
            WorkerResponseOperation::Rejected{reason});
        send_worker_frame(&writer,&WrapperToDaemon::WorkerResponse{response:WorkerResponse {
            version:REMOTE_WORKER_VERSION,correlation_id:request.correlation_id,
            connection_epoch:request.connection_epoch,worker:request.worker,command:request.command,response}}).unwrap();
        let mut reader=std::io::BufReader::new(peer.try_clone().unwrap());
        assert!(matches!(read_inbound172(&mut reader),WrapperToDaemon::WorkerResponse{response}
            if response.correlation_id=="status-while-probe"));
        assert_eq!(serde_json::to_value(runtime.store.commands().unwrap()).unwrap(),serde_json::to_value(before).unwrap());
        // La reconnexion conserve le même créneau. Aucun résultat ancien ne sort.
        disconnected.store(true,Ordering::Release);
        assert!(ProbeReadPermit::claim(&slot).is_none());
        release.send(()).unwrap();finished.recv_timeout(Duration::from_secs(1)).unwrap();
        let deadline=Instant::now()+Duration::from_secs(1);
        while slot.load(Ordering::Acquire) && Instant::now()<deadline {std::thread::yield_now();}
        assert!(!slot.load(Ordering::Acquire));
        assert!(ProbeReadPermit::claim(&slot).is_some());
        peer.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        let mut extra=[0u8;1];assert!(std::io::Read::read(&mut peer,&mut extra).is_err(),"aucun résultat de l'ancienne époque");
    }

    fn probe221_inbound(n:usize)->WorkerInbound {
        WorkerInbound::CandidateProbe(CandidateProbe {
                version:WORKER_CANDIDATE_PROBE_VERSION,correlation_id:format!("probe-{n}"),connection_epoch:"same-epoch".into(),
                worker:worker(),admission_id:"admission".into(),policy_revision:1,parent:ProviderPermissions {
                    version:1,source:"provider_turn".into(),run_id:"inbound-run".into(),provider_session_id:"session".into(),
                    provider_instance_id:"codex".into(),revision:1,driver:"codex_app_server".into(),cwd:"/project".into(),
                    runtime_mode:"full-access".into(),interaction_mode:"default".into(),
                    provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never","sandbox_policy":{"type":"readOnly"}})},
                candidates:vec![CandidateProbeCandidate {candidate_id:"c01".into(),project_id:"project".into(),
                    origin_host:"origin".into(),origin_cwd:"/project".into(),source_root:"/project".into(),
                    remote_root:"/remote".into(),source_root_identity:"/remote/.git".into(),relative_cwd:".".into(),
                    base_commit:None,agent_type:"codex".into(),model:"model".into(),effort:None,
                    posture:Some(SpawnPosture::Discovery),workspace:None}],
            })
    }

    #[test]
    fn spec221_probe_queue_full_keeps_native_lane_and_correlation() {
        let (commands,command_read)=mpsc::sync_channel(8);
        let (probes,probe_read)=mpsc::sync_channel(8);
        let (stream,peer)=UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let writer=Arc::new(Mutex::new(stream));let disconnected=AtomicBool::new(false);
        let cancelled=Mutex::new(BTreeSet::new());
        for n in 0..9 {
            let inbound=probe221_inbound(n);
            dispatch_worker_probe(&probes,inbound,&writer,&disconnected).unwrap();
        }
        let mut reader=std::io::BufReader::new(peer);
        let WrapperToDaemon::CandidateProbeResult{result}=read_inbound172(&mut reader) else {panic!("refus corrélé")};
        assert_eq!(result.correlation_id,"probe-8");assert_eq!(result.connection_epoch,"same-epoch");
        assert!(matches!(result.verdicts[0],CandidateProbeVerdict::Ineligible{reason:CandidateProbeRefusal::ResourceLimit,..}));
        let inbound=inbound172("cancel-priority",WorkerRequestOperation::Cancel{target:WorkerCancelScope::Command{}});
        dispatch_worker_inbound(&commands,inbound,&writer,&disconnected,&cancelled).unwrap();
        assert!(cancelled.lock().unwrap().contains(&json_digest(&command()).unwrap()));
        assert!(matches!(command_read.try_recv().unwrap(),WorkerInbound::Request(request) if request.correlation_id=="cancel-priority"));
        assert_eq!(probe_read.try_iter().count(),8);assert!(!disconnected.load(Ordering::Acquire));
    }

    #[test]
    fn spec221_probe_queue_expiry_refuses_without_waiting_for_active_probe() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);
        let (_sender,receiver)=mpsc::sync_channel(7);let slot=Arc::new(AtomicBool::new(false));
        let _active=ProbeReadPermit::claim(&slot).unwrap();
        let mut lane=ProbeReadLane{receiver,pending:Some((probe221_inbound(10),Instant::now()-WORKER_PROBE_READ_BUDGET-Duration::from_secs(1))),slot};
        let (stream,peer)=UnixStream::pair().unwrap();peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let writer=Arc::new(Mutex::new(stream));let stopped=Arc::new(AtomicBool::new(false));
        lane.drive(&runtime,&writer,&stopped,true).unwrap();
        let mut reader=std::io::BufReader::new(peer);
        let WrapperToDaemon::CandidateProbeResult{result}=read_inbound172(&mut reader) else {panic!("refus avant lecture")};
        assert_eq!(result.correlation_id,"probe-10");
        assert!(matches!(result.verdicts[0],CandidateProbeVerdict::Ineligible{reason:CandidateProbeRefusal::ResourceLimit,..}));
        assert!(lane.pending.is_none());assert!(runtime.store.commands().unwrap().is_empty());
        assert!(lane.slot.load(Ordering::Acquire));
    }

    #[test]
    fn spec221_probe_store_error_is_lazy_and_never_quarantines_unrelated_read() {
        let fixture=Fixture::new();let (store,_,mut request,_)=c19_store_fixture(&fixture);drop(store);
        let runtime=runtime_fixture(&fixture);let context=runtime.probe_read_context();
        let bad=runtime.store.command_path(&command());fs::write(&bad,b"PRIVATE_INVALID_COMMAND").unwrap();
        let before=fs::read(&runtime.store.enrollment).unwrap();
        request.workspace=None;request.base_commit=None;
        let mapping=ProjectMapping{source_host:request.origin_host.clone(),source_root:request.source_root.clone().into(),
            remote_root:request.remote_root.clone().into(),expected_git_identity:request.source_root_identity.clone(),
            mapping_revision:1,additional_root:None};
        assert_eq!(context.probe_workspace_preconditions(&request,&mapping,Path::new(&request.remote_root)),Ok(()));
        assert_eq!(context.active_workspace_roots(),Err(CandidateProbeRefusal::WorkspaceUnavailable));
        assert_eq!(fs::read(&runtime.store.enrollment).unwrap(),before);
        assert_eq!(fs::read(&bad).unwrap(),b"PRIVATE_INVALID_COMMAND");
    }

    #[test]
    fn spec221_probe_context_keeps_active_workspace_guard_without_store_mutation() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);
        let place=placement();let now=worker_now();
        runtime.store.reserve_command(&command(),&"d".repeat(64),&place,now+300,now).unwrap();
        let before=serde_json::to_vec(&runtime.store.commands().unwrap()).unwrap();
        let context=runtime.probe_read_context();
        assert_eq!(context.identity,runtime.identity);
        assert_eq!(context.active_workspace_roots().unwrap(),vec![PathBuf::from(place.execution_cwd)]);
        drop(context);
        assert_eq!(serde_json::to_vec(&runtime.store.commands().unwrap()).unwrap(),before);
    }

    #[test]
    fn spec221_origin_four_parallel_reads_survive_two_inflight_and_remain_bounded() {
        use bridget_transport::protocol::{OriginProofRequest,OriginProofOperation,OriginProofResult,
            CommunicationProjectSource};
        let fixture=Fixture::new();
        let (reader,writer)=UnixStream::pair().unwrap();
        let mut reader=std::io::BufReader::new(reader);
        let writer=Arc::new(Mutex::new(writer));
        let disconnected=Arc::new(AtomicBool::new(false));
        // Deux contrôles déjà en cours : l'ancien dispatcher refusait les quatre suivants.
        let slots=Arc::new(std::sync::atomic::AtomicUsize::new(2));
        let request=|id:&str|OriginProofRequest {
            version:1,correlation_id:id.into(),connection_epoch:"epoch221".into(),
            worker:WorkerIdentity {machine_id:"origin221".into(),worker_id:"worker221".into(),provisioning_revision:1},
            expires_at_ms:crate::origin_proof::now_ms()+10_000,
            request:OriginProofOperation::CommunicationProject {
                root:fixture.0.join("absent").to_string_lossy().into_owned(),
                source:CommunicationProjectSource::Git,worktree:None
            }
        };
        let read=|reader:&mut std::io::BufReader<UnixStream>|{
            let bytes=bridget_transport::jsonl::read_unix_line(reader,MAX_WORKER_FRAME_BYTES,
                bridget_transport::jsonl::LineDeadline::Absolute(Instant::now()+Duration::from_secs(5)))
                .unwrap().unwrap();
            let frame:WrapperToDaemon=serde_json::from_slice(&bytes).unwrap();
            let WrapperToDaemon::OriginProofResponse {response}=frame else {panic!("origin response")};
            response
        };
        for i in 0..4 {
            dispatch_origin_proof(request(&format!("read-{i}")),writer.clone(),disconnected.clone(),
                slots.clone(),fixture.0.join("unused-registry"));
        }
        let mut correlations=std::collections::BTreeSet::new();
        for _ in 0..4 {
            let response=read(&mut reader);
            assert_eq!(response.connection_epoch,"epoch221");
            assert_eq!(response.worker.machine_id,"origin221");
            assert!(matches!(response.response,OriginProofResult::CommunicationProject {..}),
                "chaque contrôle est exécuté, sans preuve partagée ni refus de capacité");
            correlations.insert(response.correlation_id);
        }
        assert_eq!(correlations.len(),4);
        let until=Instant::now()+Duration::from_secs(1);
        while slots.load(Ordering::Acquire)!=2 && Instant::now()<until {std::thread::yield_now();}
        assert_eq!(slots.load(Ordering::Acquire),2);
        let full=Arc::new(std::sync::atomic::AtomicUsize::new(crate::origin_proof::MAX_ORIGIN_PROOFS));
        dispatch_origin_proof(request("full"),writer.clone(),disconnected.clone(),full.clone(),fixture.0.join("unused"));
        assert!(matches!(read(&mut reader).response,
            OriginProofResult::Rejected {reason:WorkerRefusal::ResourceLimit}));
        assert_eq!(full.load(Ordering::Acquire),crate::origin_proof::MAX_ORIGIN_PROOFS);
        full.fetch_sub(1,Ordering::AcqRel);
        dispatch_origin_proof(request("after-release"),writer.clone(),disconnected.clone(),full.clone(),fixture.0.join("unused"));
        assert!(matches!(read(&mut reader).response,OriginProofResult::CommunicationProject {..}));
        let mut prep=request("prep-waits");
        prep.expires_at_ms=crate::origin_proof::now_ms()-1000;
        let parent=ProviderPermissions {version:1,source:"provider_turn".into(),run_id:"run221".into(),
            provider_session_id:"session221".into(),provider_instance_id:"instance221".into(),revision:1,
            driver:"codex_app_server".into(),cwd:"/source".into(),runtime_mode:"full-access".into(),
            interaction_mode:"default".into(),provider_policy:serde_json::json!({"kind":"codex",
                "approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}})};
        prep.request=OriginProofOperation::PrepareProject {source:bridget_transport::protocol::WorkerSourceBase {
            cwd:parent.cwd.clone(),expected_base_commit:Some("a".repeat(40)),
            workspace:Some(bridget_transport::protocol::WorkspaceRequest::Shared {}),
            workload:AutoDelegateWorkload::General},parent,target_machine:"destination".into(),project_id:None,return_context:false};
        prep.validate_shape().unwrap();
        let preparing=Arc::new(std::sync::atomic::AtomicUsize::new(2));
        dispatch_origin_proof(prep.clone(),writer.clone(),disconnected.clone(),preparing,fixture.0.join("unused"));
        assert!(matches!(read(&mut reader).response,OriginProofResult::Rejected {reason:WorkerRefusal::PrepareExpired}),
            "deux préparations reçues ne doivent plus causer un refus de capacité ; le délai refuse avant toute E/S");
        let preparing=Arc::new(std::sync::atomic::AtomicUsize::new(crate::origin_proof::MAX_ORIGIN_PROJECT_PREPARATIONS));
        dispatch_origin_proof(prep,writer,disconnected,preparing.clone(),fixture.0.join("unused"));
        assert!(matches!(read(&mut reader).response,OriginProofResult::Rejected {reason:WorkerRefusal::ResourceLimit}));
        assert_eq!(preparing.load(Ordering::Acquire),crate::origin_proof::MAX_ORIGIN_PROJECT_PREPARATIONS);

    }

    #[test]
    fn spec188_diagnostic_refusal_preserves_identity_and_actionable_unknown() {
        let v=diagnostic_failure("diagnostic_receipt_unavailable",&command());
        assert_eq!(v["command"],serde_json::to_value(command()).unwrap());
        assert!(v["recovery"].as_str().unwrap().contains("ne prouve"));
        assert!(!v["human_action"].as_str().unwrap().is_empty());
        let v=diagnostic_failure("diagnostic_observation_changed",&command());
        assert!(v["recovery"].as_str().unwrap().contains("même tuple"));
    }

    #[test]
    fn spec188_diagnostic_checks_stability_after_early_identity_and_context_errors() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let mut foreign=command();foreign.instance_id="foreign".into();
        let result=read::diagnose_observed(runtime.read_view(),&foreign,|| {
            let mut receipt=runtime.store.command(&command()).unwrap();receipt.updated_at+=1;
            runtime.store.persist(&runtime.store.command_path(&command()),&receipt).unwrap();
        });
        assert_eq!(result.unwrap_err(),"diagnostic_observation_changed");
        let original=fs::read(&runtime.config.registry_path).unwrap();
        fs::write(&runtime.config.registry_path,b"invalid registry").unwrap();
        let result=read::diagnose_local_observed(&runtime.config,&command(),|| {
            fs::write(&runtime.config.registry_path,&original).unwrap();
        });
        assert_eq!(result.unwrap_err(),"diagnostic_observation_changed");
    }
    #[test]
    fn spec188_registry_fifo_and_oversize_are_bounded_without_writer() {
        let fixture=Fixture::new();let p=fixture.0.join("registry-fifo");
        let c=std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe {libc::mkfifo(c.as_ptr(),0o600)},0);
        let (tx,rx)=std::sync::mpsc::channel();let fifo=p.clone();
        let reader=std::thread::spawn(move||tx.send(AgentRegistry::load_from_path(fifo).is_err()).unwrap());
        assert!(rx.recv_timeout(Duration::from_secs(2)).unwrap());reader.join().unwrap();
        fs::remove_file(&p).unwrap();write_private_file_atomic(&p,b"{}").unwrap();
        fs::OpenOptions::new().write(true).open(&p).unwrap().set_len(16*1024*1024+1).unwrap();
        assert!(AgentRegistry::load_from_path(p).err().unwrap().contains("volumineux"));
    }
    fn snapshot188(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out=BTreeMap::new();let mut dirs=vec![root.to_path_buf()];
        while let Some(dir)=dirs.pop() {
            for entry in fs::read_dir(dir).unwrap() {
                let entry=entry.unwrap();let p=entry.path();let m=fs::symlink_metadata(&p).unwrap();
                if m.is_dir() {dirs.push(p)} else if m.is_file() {out.insert(p,fs::read(entry.path()).unwrap());}
            }
        }
        out
    }
    #[test]
    fn spec188_diagnostic_reads_locked_store_without_write_or_terminal_inference() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let before=snapshot188(&fixture.0);
        let result=diagnose_local(&runtime.config,&command()).unwrap();
        assert_eq!(result["scope"],"local_read_only_recheck");
        assert_eq!(result["authoritative"],false);
        assert_eq!(result["check"],"refused");assert_eq!(result["stage"],"preparation");
        assert_eq!(result["reason"],"UnknownExecution");
        assert_eq!(runtime.store.command(&command()).unwrap().phase,WorkerCommandPhase::Reserved);
        assert_eq!(snapshot188(&fixture.0),before,"ni reçu, verrou ni fichier nouveau");
        assert!(!result.to_string().contains("worker162-test-secret"));
    }
    #[test]
    fn spec188_diagnostic_rejects_foreign_identity_and_absent_receipt_without_write() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let before=snapshot188(&fixture.0);
        let mut foreign=command();foreign.instance_id="foreign-instance".into();
        assert_eq!(diagnose_local(&runtime.config,&foreign).unwrap_err(),"diagnostic_identity_mismatch");
        foreign.command_id="missing-command".into();
        assert_eq!(diagnose_local(&runtime.config,&foreign).unwrap_err(),"diagnostic_receipt_unavailable");
        assert_eq!(snapshot188(&fixture.0),before);
    }
    #[test]
    fn spec188_diagnostic_rejects_symlink_and_fifo_before_open() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let p=runtime.store.command_path(&command());let moved=p.with_extension("preserved");fs::rename(&p,&moved).unwrap();
        symlink(&moved,&p).unwrap();
        assert_eq!(diagnose_local(&runtime.config,&command()).unwrap_err(),"diagnostic_file_type_invalid");
        fs::remove_file(&p).unwrap();
        let name=std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe {libc::mkfifo(name.as_ptr(),0o600)},0);
        assert_eq!(diagnose_local(&runtime.config,&command()).unwrap_err(),"diagnostic_file_type_invalid");
        assert!(moved.is_file());
    }
    #[test]
    fn spec188_diagnostic_marks_concurrent_change_instead_of_returning_stale_verdict() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let observed=read::diagnose_observed(runtime.read_view(),&command(),|| {
            let mut receipt=runtime.store.command(&command()).unwrap();receipt.updated_at+=1;
            runtime.store.persist(&runtime.store.command_path(&command()),&receipt).unwrap();
        });
        assert_eq!(observed.unwrap_err(),"diagnostic_observation_changed");
        assert_eq!(runtime.store.command(&command()).unwrap().phase,WorkerCommandPhase::Reserved);
    }
    #[test]
    fn spec188_diagnostic_permission_codes_are_closed() {
        assert_eq!(permission_diagnostic_code("permission_hook_settings_changed"),"permission_hook_settings_changed");
        for secret in ["private token=secret", "permission_secret_token", "permission_hook_settings_changed:secret"] {
            assert_eq!(permission_diagnostic_code(secret),"unknown");
        }
    }

    #[test]
    fn worker172_inbound_r7_failed_signal_can_be_retried() {
        let mut issued = BTreeSet::new();
        let attempts = std::cell::Cell::new(0);
        let first = worker_stop_once(&mut issued, "command", || {
            attempts.set(attempts.get() + 1);
            Err(WorkerRefusal::StoreUnavailable)
        });
        assert_eq!(first, Err(WorkerRefusal::StoreUnavailable));
        assert!(issued.is_empty(), "un refus de signal ne vaut pas envoi");
        worker_stop_once(&mut issued, "command", || {
            attempts.set(attempts.get() + 1);
            Ok(())
        }).unwrap();
        worker_stop_once(&mut issued, "command", || panic!("signal déjà envoyé")).unwrap();
        assert_eq!(attempts.get(), 2);
        assert!(issued.contains("command"));
    }

    #[test]
    fn worker172_inbound_r7_expired_marker_error_is_scoped() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        let first = command();
        let mut second = first.clone();
        second.command_id = "other-expired-command".into();
        second.task_id = "other-expired-task".into();
        runtime.store.reserve_command(&first, &"d".repeat(64), &placement(), now - 1, now - 2).unwrap();
        runtime.store.reserve_command(&second, &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let marker = ManagedMarker { pgid: 9999999, birth: 1, instance_id: first.instance_id.clone(),
            command_id: first.command_id.clone(), generation: first.generation };
        runtime.store.mark_command(&first, &marker, now - 2).unwrap();
        let outcome = runtime.tick();
        eprintln!("{}", serde_json::json!({"case":"expired_marker_error","tick_ok":outcome.is_ok(),
            "other_phase":runtime.store.command(&second).unwrap().phase}));
        assert_eq!(outcome, Ok(()));
        assert_eq!(runtime.store.command(&second).unwrap().phase, WorkerCommandPhase::Unknown);
    }

    #[test]
    fn worker172_inbound_r7_cancel_before_marker_reconciles_unknown() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        runtime.tick().unwrap();
        let phase = runtime.store.command(&command()).unwrap().phase;
        eprintln!("{}", serde_json::json!({"case":"cancel_before_marker","phase":phase}));
        assert_eq!(phase, WorkerCommandPhase::Unknown);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn worker172_inbound_r7_held_abandon_does_not_hide_stop_signal() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let script_path = fixture.0.join("held-bootstrap.py");
        let trace = fixture.0.join("held-term");
        let stop = fixture.0.join("held-stop");
        struct StopGuard(PathBuf);
        impl Drop for StopGuard {
            fn drop(&mut self) { let _ = fs::write(&self.0, b"stop"); }
        }
        let _stop_guard = StopGuard(stop.clone());
        let script = format!(r#"#!/usr/bin/python3
import json,os,signal,sys,time
from pathlib import Path
trace=Path({trace:?})
stop=Path({stop:?})
def term(sig,frame):
 trace.write_text("TERM\n")
 sys.exit(0)
signal.signal(signal.SIGTERM,term)
os.setpgid(0,0)
args=sys.argv
birth=int(Path("/proc/self/stat").read_text().rsplit(")",1)[1].split()[19])
ready={{"event":"bootstrap_ready","pid":os.getpid(),"pgid":os.getpid(),"birth":birth,
 "instance_id":args[args.index("--instance-id")+1],"command_id":args[args.index("--command-id")+1],
 "generation":int(args[args.index("--generation")+1])}}
os.write(101,(json.dumps(ready)+"\n").encode())
while not stop.exists(): time.sleep(.01)
"#, trace=trace.to_string_lossy(),stop=stop.to_string_lossy());
        fs::write(&script_path,script).unwrap();
        fs::set_permissions(&script_path,fs::Permissions::from_mode(0o700)).unwrap();
        let launch = crate::managed_process::ManagedLaunch {
            bootstrap_executable: script_path, wrapper_executable: "/bin/true".into(), wrapper_args: vec![],
            identity: crate::managed_process::ManagedIdentity { instance_id: command().instance_id,
                command_id: command().command_id, generation: command().generation },
            cwd: fixture.0.clone(), env: BTreeMap::new(),
        };
        let mut bootstrap = crate::managed_process::spawn_managed_bootstrap(&launch).unwrap();
        bootstrap.set_ready_timeout(Duration::from_secs(2)).unwrap();
        let marked = bootstrap.wait_ready().unwrap().persist_marker(&runtime.markers,&marker_key(&command())).unwrap();
        let marker = marked.marker().marker().clone();
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        runtime.store.mark_command(&command(),&marker,now).unwrap();
        let parent = ProviderPermissions {
            version:1, source:"native_wrapper".into(),run_id:"run".into(),provider_session_id:"session".into(),
            provider_instance_id:"parent".into(),revision:1,driver:"codex_app_server".into(),
            cwd:"/origin".into(),runtime_mode:"full-access".into(),interaction_mode:"default".into(),
            provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
        };
        let request = WorkerPrepare { service_tier: None,
            machine_id:"cartae".into(),project_id:"project".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin".into(),origin_cwd:"/origin".into(),source_root:"/origin".into(),remote_root:"/remote".into(),
            source_root_identity:"git:held".into(),relative_cwd:".".into(),base_commit:None,agent_type:"codex".into(),
            model:"fixture".into(),effort:None,posture:None,workspace:None,parent,
        };
        let definition = ResolvedAgentDefinition {
            command:"/bin/true".into(),args:vec![],protocol:"codex_app_server".into(),forbidden_env:vec![],
            pass_env:vec![],permissions:"plan".into(),claude_config_dir:None,
            codex_home: None,queue_capacity:1,notify_timeout_secs:60,
            mcp:bridget_transport::protocol::ResolvedMcpDefinition{interactive:"off".into(),acp_session:false},
            capabilities:Default::default(),digest:"a".repeat(64),
        };
        let prepared = PreparedRuntime { routing:None,command:command(),request,
            prepared:WorkerPrepared { token:"held".into(),placement:placement(),definition,child_policy:serde_json::json!({}),
                effective_posture:SpawnPosture::Discovery,prepared_at:now,workspace:None },
            expires:Instant::now()+Duration::from_secs(300) };
        runtime.held.insert(command().command_id,HeldRuntime { prepared,mission_id:"held-mission".into(),
            child:marked,ready_at:now,expires:Instant::now()+Duration::from_secs(300) });
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        for _ in 0..3 {
            runtime.last_maintenance=Instant::now()-Duration::from_secs(2);
            runtime.tick().unwrap();
        }
        let signalled = trace.exists();
        eprintln!("{}",serde_json::json!({"case":"held_stop","signal_observed":signalled}));
        // Sortie garantie même sur RED : le processus de test observe ce fichier.
        fs::write(&stop,b"stop").unwrap();
        if let Some(child)=runtime.abandoned.get_mut(&command().command_id) { child.wait().unwrap(); }
        assert!(signalled,"fermer RELEASE n'est pas une preuve d'envoi de SIGTERM");
    }

    #[test]
    fn worker172_inbound_cancel_maintenance_keeps_other_commands_alive() {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        struct ChildGuard(std::process::Child);
        impl Drop for ChildGuard {
            fn drop(&mut self) { drop(self.0.stdin.take()); let _ = self.0.wait(); }
        }
        let child = ChildGuard(Command::new("/bin/cat").stdin(Stdio::piped())
            .stdout(Stdio::null()).stderr(Stdio::null()).process_group(0).spawn().unwrap());
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        let first = command();
        let mut second = first.clone();
        second.command_id = "other-command".into();
        second.task_id = "other-task".into();
        for identity in [&first, &second] {
            runtime.store.reserve_command(identity, &"d".repeat(64), &placement(), now + 300, now).unwrap();
        }
        let result_slot = slot(24);
        runtime.store.reserve_result_slot(&result_slot, now).unwrap();
        let marker = ManagedMarker {
            pgid: child.0.id(), birth: crate::managed_process::process_birth(child.0.id()).unwrap(), instance_id: first.instance_id.clone(),
            command_id: first.command_id.clone(), generation: first.generation,
        };
        runtime.store.mark_command(&first, &marker, now).unwrap();
        // Le reçu est valide, mais le fichier de marqueur manque : cancel renvoie
        // StoreUnavailable avant tout signal. Ce défaut doit rester à cette commande.
        runtime.cancelled.lock().unwrap().insert(json_digest(&first).unwrap());
        let tick = runtime.tick();
        eprintln!("{}", serde_json::json!({"case":"cancel_maintenance_error",
            "tick_ok":tick.is_ok(),"first_phase":runtime.store.command(&first).unwrap().phase,
            "second_phase":runtime.store.command(&second).unwrap().phase}));
        assert_eq!(tick, Ok(()));
        assert_eq!(runtime.store.command(&second).unwrap().phase, WorkerCommandPhase::Unknown);
        assert!(!matches!(runtime.store.command(&first).unwrap().phase, WorkerCommandPhase::Terminal));
        assert!(runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().closed.is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn worker172_inbound_cancel_maintenance_signals_once_and_waits_for_drain() {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        struct ChildGuard(std::process::Child);
        impl Drop for ChildGuard {
            fn drop(&mut self) { drop(self.0.stdin.take()); let _ = self.0.wait(); }
        }
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let trace = fixture.0.join("term-count");
        let script = "import signal,sys\ndef stop(sig,frame):\n with open(sys.argv[1],'a') as f: f.write('TERM\\n')\nsignal.signal(signal.SIGTERM,stop)\nprint('ready',flush=True)\nsys.stdin.buffer.read()\n";
        let mut child = ChildGuard(Command::new("python3").args(["-u","-c",script])
            .arg(&trace).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
            .process_group(0).spawn().unwrap());
        let mut ready = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(child.0.stdout.take().unwrap()), &mut ready).unwrap();
        assert_eq!(ready.trim(), "ready");
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let result_slot = slot(25);
        runtime.store.reserve_result_slot(&result_slot, now).unwrap();
        let marker = ManagedMarker {
            pgid: child.0.id(), birth: crate::managed_process::process_birth(child.0.id()).unwrap(),
            instance_id: command().instance_id, command_id: command().command_id, generation: command().generation,
        };
        runtime.store.mark_command(&command(), &marker, now).unwrap();
        write_json(&runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command()))), &marker).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        for _ in 0..3 {
            runtime.last_maintenance = Instant::now() - Duration::from_secs(2);
            runtime.tick().unwrap();
        }
        let signals = fs::read_to_string(&trace).unwrap().lines().count();
        let phase = runtime.store.command(&command()).unwrap().phase;
        let open = runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().closed.is_none();
        let live = crate::managed_process::group_exists(marker.pgid).unwrap();
        eprintln!("{}", serde_json::json!({"case":"cancel_signal_once","signals":signals,
            "group_live":live,"phase":phase,"slot_open":open}));
        drop(child.0.stdin.take());
        child.0.wait().unwrap();
        let receipt = runtime.store.command(&command()).unwrap();
        write_json(&drain_proof_path(&runtime.store.root, &command()), &WorkerDrainProof::expected(&receipt).unwrap()).unwrap();
        runtime.last_maintenance = Instant::now() - Duration::from_secs(2);
        runtime.tick().unwrap();
        let closed = runtime.store.read_slot(&result_slot.identity).unwrap().unwrap();
        assert_eq!(signals, 1, "un même arrêt ne doit pas envoyer SIGTERM à chaque tick");
        assert!(live && open && phase != WorkerCommandPhase::Terminal, "pas de clôture fictive tant que le groupe vit");
        assert_eq!(runtime.store.command(&command()).unwrap().phase, WorkerCommandPhase::Terminal);
        assert!(closed.producer_terminal && closed.closed == Some(WorkerSlotClosureProof::Cancelled));
    }

    #[test]
    fn worker172_inbound_expired_reservation_has_no_store_effect() {
        let fixture = Fixture::new();
        let store = fixture.open();
        let now = worker_now();
        store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let result = store.reserve_result_slot_before(&slot(20), now, Some((now - 1) * 1000));
        let slots = store.inventory().unwrap().1;
        eprintln!("{}", serde_json::json!({"case":"expired_reservation","request_deadline_ms":(now-1)*1000,
            "worker_now":now,"command_deadline":now+300,"accepted":result.is_ok(),"slots_written":slots.len()}));
        assert!(result.is_err(), "une réservation déjà expirée doit être refusée");
        assert!(slots.is_empty(), "aucune écriture après échéance");
    }

    #[test]
    fn worker172_inbound_cancelled_reservation_is_refused() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let reserved = slot(21);
        runtime.store.reserve_result_slot(&reserved, now).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        let WorkerInbound::Request(request) = inbound172("cancelled-slot", WorkerRequestOperation::ReserveResultSlot { slot: reserved }) else { unreachable!() };
        let result = runtime.handle(&request, "epoch", &AtomicBool::new(false));
        eprintln!("{}", serde_json::json!({"case":"cancelled_reservation","cancel_marker":true,"accepted":result.is_ok(),
            "open_slots":runtime.store.inventory().unwrap().1.iter().filter(|s|s.closed.is_none()).count()}));
        assert!(result.is_err(), "une annulation déjà observée interdit une réservation");
    }

    #[test]
    fn worker172_inbound_deadline_is_checked_after_queueing() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let (socket, mut peer) = UnixStream::pair().unwrap();
        let mut input = std::io::BufReader::new(socket.try_clone().unwrap());
        let writer = Arc::new(Mutex::new(socket));
        let expires_at_ms = worker_now_millis() + 100;
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "queued-deadline".into(),
            connection_epoch: "epoch".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::ReserveResultSlotUntil { slot: slot(22), expires_at_ms },
        };
        request.validate().unwrap();
        writeln!(peer, "{}", serde_json::to_string(&DaemonToWrapper::WorkerRequest { request }).unwrap()).unwrap();
        let bytes = bridget_transport::jsonl::read_unix_line(&mut input, MAX_WORKER_FRAME_BYTES,
            bridget_transport::jsonl::LineDeadline::Absolute(Instant::now() + Duration::from_secs(1)))
            .unwrap().unwrap();
        let DaemonToWrapper::WorkerRequest { request } = serde_json::from_slice(&bytes).unwrap() else { unreachable!() };
        let (sender, receiver) = mpsc::sync_channel(8);
        let disconnected = AtomicBool::new(false);
        dispatch_worker_inbound(&sender, WorkerInbound::Request(request), &writer, &disconnected, &runtime.cancelled).unwrap();
        std::thread::sleep(Duration::from_millis(150));
        let WorkerInbound::Request(request) = receiver.try_recv().unwrap() else { unreachable!() };
        let response = runtime.handle(&request, "epoch", &disconnected);
        let slots = runtime.store.inventory().unwrap().1;
        eprintln!("{}", serde_json::json!({"case":"queued_deadline","request_deadline_ms":expires_at_ms,
            "worker_now_ms":worker_now_millis(),"accepted":response.is_ok(),"slots_written":slots.len()}));
        assert_eq!(response.unwrap_err(), WorkerRefusal::ResultSlotExpired);
        assert!(slots.is_empty());
        send_worker_frame(&writer, &WrapperToDaemon::WorkerResponse { response: WorkerResponse {
            version: REMOTE_WORKER_VERSION, correlation_id: request.correlation_id.clone(),
            connection_epoch: request.connection_epoch.clone(), worker: request.worker.clone(),
            command: request.command.clone(),
            response: WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResultSlotExpired },
        }}).unwrap();
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut std::io::BufReader::new(peer)) else { unreachable!() };
        response.validate().unwrap();
        assert_eq!(response.correlation_id, "queued-deadline");
        assert_eq!(response.response, WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResultSlotExpired });
    }

    #[test]
    fn worker172_inbound_deadline_is_checked_after_store_lock() {
        let fixture = Fixture::new();
        let store = Arc::new(fixture.open());
        let now = worker_now();
        store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let guard = store.transition.lock().unwrap();
        let delayed = store.clone();
        let expires_at_ms = worker_now_millis() + 100;
        let waiting = std::thread::spawn(move || delayed.reserve_result_slot_before(&slot(23), now, Some(expires_at_ms)));
        std::thread::sleep(Duration::from_millis(150));
        drop(guard);
        assert!(matches!(waiting.join().unwrap(), Err(WorkerStoreError::Invalid(WorkerRefusal::ResultSlotExpired))));
        assert!(store.inventory().unwrap().1.is_empty());
    }

    fn inbound172(correlation: &str, request: WorkerRequestOperation) -> WorkerInbound {
        WorkerInbound::Request(WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: correlation.into(),
            connection_epoch: "epoch".into(), worker: worker(), command: command(), request,
        })
    }

    fn read_inbound172(reader: &mut std::io::BufReader<UnixStream>) -> WrapperToDaemon {
        let bytes = bridget_transport::jsonl::read_unix_line(reader, MAX_WORKER_FRAME_BYTES,
            bridget_transport::jsonl::LineDeadline::Absolute(Instant::now() + Duration::from_secs(1)))
            .unwrap().unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn worker172_inbound_saturation_refuses_slot_without_late_effect() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let (sender, receiver) = mpsc::sync_channel(8);
        for index in 0..8 {
            sender.send(inbound172(&format!("queued-{index}"), WorkerRequestOperation::Reconcile {})).ok().unwrap();
        }
        let (socket, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let mut reader = std::io::BufReader::new(peer);
        let disconnected = AtomicBool::new(false);
        let cancelled = Mutex::new(BTreeSet::new());
        let slot = slot(17);
        dispatch_worker_inbound(&sender,
            inbound172("refused-slot", WorkerRequestOperation::ReserveResultSlot { slot }),
            &writer, &disconnected, &cancelled).unwrap();
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut reader) else { panic!("réponse attendue"); };
        response.validate().unwrap();
        assert_eq!(response.correlation_id, "refused-slot");
        assert_eq!(response.connection_epoch, "epoch");
        assert_eq!(response.command, command());
        assert_eq!(response.response, WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResourceLimit });
        assert!(!disconnected.load(Ordering::Acquire));
        // Le consommateur reste bloqué au-delà du délai maître de trois secondes.
        std::thread::sleep(Duration::from_millis(3100));
        for index in 0..8 {
            let WorkerInbound::Request(request) = receiver.try_recv().unwrap() else { panic!("requête attendue"); };
            assert_eq!(request.correlation_id, format!("queued-{index}"));
            let _ = runtime.handle(&request, "epoch", &disconnected);
        }
        assert!(receiver.try_recv().is_err(), "la réservation refusée ne peut être exécutée plus tard");
        assert!(runtime.store.inventory().unwrap().1.is_empty(), "aucun slot orphelin");
        dispatch_worker_inbound(&sender, inbound172("after-saturation", WorkerRequestOperation::Reconcile {}),
            &writer, &disconnected, &cancelled).unwrap();
        let WorkerInbound::Request(request) = receiver.try_recv().unwrap() else { panic!("requête attendue"); };
        assert_eq!(request.correlation_id, "after-saturation", "le canal reste utilisable");
    }

    #[test]
    fn worker172_inbound_saturation_refuses_probes_and_observes_cancel() {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(inbound172("queued", WorkerRequestOperation::Reconcile {})).ok().unwrap();
        let (socket, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let mut reader = std::io::BufReader::new(peer);
        let disconnected = AtomicBool::new(false);
        let cancelled = Mutex::new(BTreeSet::new());
        let parent = ProviderPermissions {
            version: 1, source: "provider_turn".into(), run_id: "inbound-run".into(),
            provider_session_id: "session".into(), provider_instance_id: "codex".into(),
            revision: 1, driver: "codex_app_server".into(), cwd: "/project".into(),
            runtime_mode: "full-access".into(), interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never","sandbox_policy":{"type":"readOnly"}}),
        };
        let candidate = CandidateProbeCandidate {
            candidate_id: "c01".into(), project_id: "project-162".into(), origin_host: "mac".into(),
            origin_cwd: "/project".into(), source_root: "/project".into(), remote_root: "/remote".into(),
            source_root_identity: "repo".into(), relative_cwd: ".".into(), base_commit: None,
            agent_type: "codex".into(), model: "model".into(), effort: None,
            posture: Some(SpawnPosture::Discovery), workspace: None,
        };
        let probe = CandidateProbe {
            version: WORKER_CANDIDATE_PROBE_VERSION, correlation_id: "probe-v1".into(),
            connection_epoch: "epoch".into(), worker: worker(), admission_id: "admission".into(),
            policy_revision: 1, parent: parent.clone(), candidates: vec![candidate.clone()],
        };
        dispatch_worker_inbound(&sender, WorkerInbound::CandidateProbe(probe.clone()), &writer, &disconnected, &cancelled).unwrap();
        let WrapperToDaemon::CandidateProbeResult { result } = read_inbound172(&mut reader) else { panic!("sonde v1 attendue"); };
        result.validate_shape().unwrap();
        assert_eq!(result.correlation_id, probe.correlation_id);
        assert_eq!(result.connection_epoch, probe.connection_epoch);
        assert_eq!(result.admission_id, probe.admission_id);
        assert_eq!(result.policy_revision, probe.policy_revision);
        assert!(matches!(result.verdicts[0], CandidateProbeVerdict::Ineligible { reason: CandidateProbeRefusal::ResourceLimit, .. }));
        let probe = CandidateProbeV2 { requires_workspace_write:false, required_tools:Vec::new(),
            version: WORKER_CANDIDATE_PROBE_V2_VERSION, correlation_id: "probe-v2".into(),
            connection_epoch: "epoch".into(), worker: worker(), admission_id: "admission".into(),
            policy_revision: 1, parent,
            candidates: vec![CandidateProbeV2Candidate { service_tier: None,
                candidate_id: "c001".into(), project_id: candidate.project_id, origin_host: candidate.origin_host,
                origin_cwd: candidate.origin_cwd, source_root: candidate.source_root, remote_root: candidate.remote_root,
                source_root_identity: candidate.source_root_identity, relative_cwd: candidate.relative_cwd,
                expected_base_commit: None, engine: "codex".into(), connection_id: "codex-cartae".into(),
                agent_type: candidate.agent_type, model: candidate.model, effort: None,
                workload: AutoDelegateWorkload::General, posture: candidate.posture, workspace: None,
            }],
        };
        dispatch_worker_inbound(&sender, WorkerInbound::CandidateProbeV2(probe.clone()), &writer, &disconnected, &cancelled).unwrap();
        let WrapperToDaemon::CandidateProbeResultV2 { result } = read_inbound172(&mut reader) else { panic!("sonde v2 attendue"); };
        result.validate_shape().unwrap();
        assert_eq!(result.correlation_id, probe.correlation_id);
        assert_eq!(result.connection_epoch, probe.connection_epoch);
        assert_eq!(result.admission_id, probe.admission_id);
        assert_eq!(result.policy_revision, probe.policy_revision);
        assert!(matches!(result.verdicts[0], CandidateProbeV2Verdict::Ineligible { reason: CandidateProbeRefusal::ResourceLimit, .. }));
        assert!(result.resources.cpu_percent.is_none() && result.resources.available_memory_percent.is_none());
        dispatch_worker_inbound(&sender, inbound172("cancel", WorkerRequestOperation::Cancel { target: WorkerCancelScope::Command {} }),
            &writer, &disconnected, &cancelled).unwrap();
        assert!(cancelled.lock().unwrap().contains(&json_digest(&command()).unwrap()), "annulation observée sans attendre le consommateur");
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut reader) else { panic!("refus corrélé attendu"); };
        assert_eq!(response.correlation_id, "cancel");
        assert!(!disconnected.load(Ordering::Acquire));
        assert!(matches!(receiver.try_recv().unwrap(), WorkerInbound::Request(_)));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn worker172_inbound_shutdown_never_blocks_or_delivers() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let (socket, _peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let stopped = AtomicBool::new(true);
        let cancelled = Mutex::new(BTreeSet::new());
        assert!(dispatch_worker_inbound(&sender, inbound172("stopped", WorkerRequestOperation::Reconcile {}),
            &writer, &stopped, &cancelled).is_err());
        assert!(receiver.try_recv().is_err());
        drop(receiver);
        assert!(dispatch_worker_inbound(&sender, inbound172("closed", WorkerRequestOperation::Reconcile {}),
            &writer, &AtomicBool::new(false), &cancelled).is_err());
    }

    #[test]
    fn worker172_inbound_late_slot_is_closed_by_terminal_reconciliation() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let late = slot(18);
        runtime.store.reserve_result_slot(&late, 101).unwrap();
        runtime.store.set_phase(&command(), WorkerCommandPhase::Terminal, 102).unwrap();
        runtime.reconcile(&command()).unwrap();
        let slots = runtime.store.inventory().unwrap().1;
        assert_eq!(slots.len(), 1);
        assert!(slots[0].producer_terminal);
        assert_eq!(slots[0].closed, Some(WorkerSlotClosureProof::Cancelled));
    }

    #[test]
    fn native171_probe_projection_guard_reasons_closed_bounded_same_refusal() {
        use crate::native_permissions::{ProjectionGuard, equivalence_fail_at,
            observe_projection171, projection_witness_summary171};
        let correlation = "16679ded-7e80-4918-ad3e-7df3ebcdda5d";
        let admission = "a-86a844fb3499046e4428fc09348a1e50";
        let digest = format!("sha256:{}", "a".repeat(64));
        let policy = serde_json::json!({"launch_context":{"source_equivalence":{
            "version":2,"scope":"origin","profile":{"revision":digest}}}});
        let batch = ProbePermissionDiagnostics::new(correlation, admission);
        let mut seen = std::collections::BTreeSet::new();
        for guard in ProjectionGuard::ALL {
            let (result, observed) = observe_projection171::<()>(|| Err(equivalence_fail_at(guard)));
            assert_eq!(result, Err("permission_hook_equivalence_unavailable".into()));
            assert_eq!(observed, Some(guard));
            assert!(seen.insert(guard.as_str()));
            let mut notes = Vec::new();
            let refusal = map_probe_projection_failure171(result.as_ref().unwrap_err(),
                Some((&batch,"c001")), observed, &policy, |note| notes.push(note.to_owned()));
            assert!(matches!(refusal,WorkerRefusal::PermissionsChanged));
            assert_eq!(notes.len(),1);
            let note = &notes[0];
            assert!(note.len() <= MAX_PROBE_PERMISSION_NOTE_BYTES);
            let v: serde_json::Value = serde_json::from_str(note).unwrap();
            assert_eq!(v.as_object().unwrap().len(),8);
            assert_eq!(v["reason"],guard.as_str());
            assert_eq!(v["code"],"permission_hook_equivalence_unavailable");
            assert_eq!(v["site"],"projection");
            assert_eq!(v["candidate_id"],"c001");
            assert_eq!(v["correlation_id"],correlation);
            assert_eq!(v["admission_id"],admission);
            assert_eq!(v["witness"],serde_json::json!({"shape":"object","version":2,
                "scope":"origin","profile":"object","revision":digest}));
        }
        assert_eq!(seen.len(),ProjectionGuard::ALL.len());
        // No previous call's guard survives, including nested synchronous reads.
        let (unknown, observed) = observe_projection171::<()>(|| Err("permission_hook_equivalence_unavailable".into()));
        assert!(unknown.is_err());assert_eq!(observed,None);
        let (success, observed) = observe_projection171(|| Ok(7));
        assert_eq!(success,Ok(7));assert_eq!(observed,None);
        let (_, observed) = observe_projection171::<()>(|| {
            let _ = equivalence_fail_at(ProjectionGuard::G001);
            let (_, inner) = observe_projection171::<()>(|| Err(equivalence_fail_at(ProjectionGuard::G002)));
            assert_eq!(inner,Some(ProjectionGuard::G002));
            Err("permission_hook_equivalence_unavailable".into())
        });
        assert_eq!(observed,Some(ProjectionGuard::G001));
        let (_, observed) = observe_projection171::<()>(|| Err("permission_hook_equivalence_unavailable".into()));
        assert_eq!(observed,None);
        // Malformed and very large user-controlled witness values become type tags.
        let secret = "SECRET /private/settings --token env=hidden STACK".repeat(2000);
        for witness in [serde_json::Value::Null,serde_json::json!(secret),
            serde_json::json!([secret]),serde_json::json!({"version":secret,"scope":secret,
                "profile":{"revision":secret,"unexported":secret},"binding":secret}),
            serde_json::json!({"version":999,"scope":"other","profile":{"revision":"sha256:BAD"}})] {
            let bad = serde_json::json!({"launch_context":{"source_equivalence":witness}});
            let mut note = None;
            let refusal = map_probe_projection_failure171("permission_hook_equivalence_unavailable",
                Some((&batch,"c001")),None,&bad,|v| note=Some(v.to_owned()));
            assert!(matches!(refusal,WorkerRefusal::PermissionsChanged));
            let note=note.unwrap();assert!(note.len()<=512);
            for forbidden in ["SECRET","/private/","--token","hidden","STACK","unexported","binding"] {
                assert!(!note.contains(forbidden));
            }
            let v:serde_json::Value=serde_json::from_str(&note).unwrap();assert_eq!(v["reason"],"unknown");
            assert_eq!(v["witness"].as_object().unwrap().len(),5);
        }
        let missing=projection_witness_summary171(&serde_json::json!({})).json();
        assert_eq!(missing,serde_json::json!({"shape":"absent","version":"absent",
            "scope":"absent","profile":"absent","revision":"absent"}));
        let cap=ProbePermissionDiagnostics::new(correlation,admission);
        for n in 1..=128 {
            let mut emitted=false;
            let r=map_probe_projection_failure171("permission_hook_equivalence_unavailable",
                Some((&cap,&format!("c{n:03}"))),Some(ProjectionGuard::G001),&policy,|v|{
                    emitted=true;assert!(v.len()<=512);
                });
            assert!(emitted);assert!(matches!(r,WorkerRefusal::PermissionsChanged));
        }
        let mut emitted=false;
        let r=map_probe_projection_failure171("permission_hook_equivalence_unavailable",
            Some((&cap,"c001")),Some(ProjectionGuard::G001),&policy,|_|emitted=true);
        assert!(!emitted);assert!(matches!(r,WorkerRefusal::PermissionsChanged));
        for (corr,adm,candidate) in [("bad",admission,"c001"),(correlation,"a-bad","c001"),
            (correlation,admission,"c01"),(correlation,admission,"c129")] {
            let bad=ProbePermissionDiagnostics::new(corr,adm);let mut emitted=false;
            let r=map_probe_projection_failure171("permission_hook_equivalence_unavailable",
                Some((&bad,candidate)),Some(ProjectionGuard::G001),&policy,|_|emitted=true);
            assert!(!emitted);assert!(matches!(r,WorkerRefusal::PermissionsChanged));
        }
        let mut emitted=false;
        assert!(matches!(map_probe_projection_failure171("permission_hook_equivalence_unavailable",
            None,Some(ProjectionGuard::G001),&policy,|_|emitted=true),WorkerRefusal::PermissionsChanged));
        assert!(!emitted);
        // Another error still uses the unchanged closed-code path, without witness fields.
        let mut note=None;
        let other=map_probe_projection_failure171("unknown SECRET",Some((&batch,"c001")),
            None,&policy,|v|note=Some(v.to_owned()));
        assert!(matches!(other,WorkerRefusal::PermissionsChanged));
        let note=note.unwrap();assert!(!note.contains("SECRET"));
        let value:serde_json::Value=serde_json::from_str(&note).unwrap();
        assert_eq!(value["code"],"unknown");assert!(value.get("witness").is_none());
    }

    #[test]
    fn native171_probe_permission_observation_closed_correlated_bounded_same_verdict() {
        let correlation = "9c9608cf-5f4c-41d5-a6af-1e0a3f0c012c";
        let admission = "a-7e51301c7e8dcfd3718419b98c7523a7";
        let batch = ProbePermissionDiagnostics::new(correlation, admission);
        let sites = [ProbePermissionSite::Projection, ProbePermissionSite::ScopedConstructor,
            ProbePermissionSite::Refresh, ProbePermissionSite::Recheck, ProbePermissionSite::BatchFinish];
        let codes = ["permission_hook_profile_unavailable", "permission_not_inherited",
            "settings_revision_changed", "permission_hook_dependency_changed", "permission_hook_profile_timeout"];
        let mut notes = Vec::new();
        for (index, site) in sites.into_iter().enumerate() {
            let candidate = format!("c{:03}", index + 1);
            let refusal = map_probe_permission_failure(codes[index],
                Some((&batch, &candidate)), site, |note| notes.push(note.to_owned()));
            assert!(matches!(refusal, WorkerRefusal::PermissionsChanged));
            let value: serde_json::Value = serde_json::from_str(notes.last().unwrap()).unwrap();
            assert_eq!(value.as_object().unwrap().len(), 6);
            assert_eq!(value["event"], "worker_probe_permission_refused");
            assert_eq!(value["correlation_id"], correlation);
            assert_eq!(value["admission_id"], admission);
            assert_eq!(value["candidate_id"], candidate);
            assert_eq!(value["site"], site.as_str());
            assert_eq!(value["code"], codes[index]);
            assert!(notes.last().unwrap().len() <= MAX_PROBE_PERMISSION_NOTE_BYTES);
        }
        for opaque in ["permission_source_unavailable SECRET", "--token=SECRET\nSTACK /private/settings",
            "permission_hook_dependency_changed\n{\"env\":\"SECRET\"}", &"SECRET".repeat(100_000)] {
            let refusal = map_probe_permission_failure(opaque, Some((&batch, "c001")),
                ProbePermissionSite::Projection, |note| notes.push(note.to_owned()));
            assert!(matches!(refusal, WorkerRefusal::PermissionsChanged));
            let note = notes.last().unwrap();
            assert!(!note.contains("SECRET") && !note.contains("STACK") && !note.contains("--token") && !note.contains("env"));
            assert_eq!(serde_json::from_str::<serde_json::Value>(note).unwrap()["code"], "unknown");
        }
        for (bad_correlation, bad_admission, bad_candidate) in [
            ("bad", admission, "c001"), (correlation, "a-not-hex", "c001"),
            (correlation, admission, "c01"), (correlation, admission, "c129"),
            ("9c9608cf-5f4c-41d5-a6af-1e0a3f0c012x", admission, "c001"),
            (correlation, "a-gggggggggggggggggggggggggggggggg", "c001"),
        ] {
            let invalid = ProbePermissionDiagnostics::new(bad_correlation, bad_admission);
            assert!(invalid.note(bad_candidate, ProbePermissionSite::Recheck, "settings_revision_changed").is_none());
        }
        let cap = ProbePermissionDiagnostics::new(correlation, admission);
        for index in 0..MAX_PROBE_PERMISSION_NOTES {
            assert!(cap.note(&format!("c{:03}", index + 1), ProbePermissionSite::BatchFinish,
                "permission_hook_equivalence_unavailable").is_some());
        }
        assert!(cap.note("c001", ProbePermissionSite::BatchFinish, "settings_revision_changed").is_none());
        let old_count = notes.len();
        assert!(matches!(map_probe_permission_failure("opaque", None, ProbePermissionSite::Projection,
            |note| notes.push(note.to_owned())), WorkerRefusal::PermissionsChanged));
        assert_eq!(notes.len(), old_count);
        let success: Result<u8, String> = Ok(7);
        assert_eq!(success.map_err(|code| map_probe_permission_failure(&code, Some((&batch, "c001")),
            ProbePermissionSite::Projection, |note| notes.push(note.to_owned()))).unwrap(), 7);
        assert_eq!(notes.len(), old_count);
    }

    #[test]
    fn incident171_startup_outcome_survives_terminal_and_reopen_without_raw_message() {
        use bridget_transport::protocol::{WorkerStartupFailure as Failure, WorkerObservation};
        let fixture = Fixture::new();
        let store = fixture.open(); reserve(&store);
        let before = store.command(&command()).unwrap();
        store.record_startup_outcome(&command(), Some(Failure::CommandMissing), None).unwrap();
        store.record_startup_outcome(&command(), Some(Failure::ExitedBeforeReady), Some(127)).unwrap();
        let after = store.command(&command()).unwrap();
        assert_eq!(after.phase, before.phase);
        assert_eq!(after.native_ready, before.native_ready);
        assert_eq!(after.startup_failed, before.startup_failed);
        drop(store);
        let store = fixture.open();
        let receipt = store.command(&command()).unwrap();
        assert_eq!(receipt.startup_failure, Some(Failure::CommandMissing));
        assert_eq!(receipt.terminal_exit_code, Some(127));
        let observation = WorkerObservation { startup_failure: receipt.startup_failure,
            phase: WorkerCommandPhase::Terminal, native_ready:false, exit_code:receipt.terminal_exit_code,
            observed_at:104, workspace:None, permission_digest:None };
        let encoded=serde_json::to_value(&observation).unwrap();
        let decoded:WorkerObservation=serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded,observation);
        let mut foreign=encoded.clone();foreign["startup_failure"]=serde_json::json!("SECRET arbitrary stderr");
        assert!(serde_json::from_value::<WorkerObservation>(foreign).is_err());
        let mut legacy=encoded;legacy.as_object_mut().unwrap().remove("startup_failure");
        assert!(serde_json::from_value::<WorkerObservation>(legacy).unwrap().startup_failure.is_none());
    }


    #[test]
    fn spec172_worker_reconnects_after_probe_eof_before_sending_credential() {
        use std::io::BufRead;
        use std::os::unix::net::UnixListener;
        let fixture = Fixture::new();
        let runtime = runtime_fixture(&fixture);
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        let listener = UnixListener::bind(&config.master_socket).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config.master_socket,fs::Permissions::from_mode(0o600)).unwrap();
        listener.set_nonblocking(true).unwrap();
        drop(runtime);
        let running = std::thread::spawn(move || run_worker(config));
        fn accept(listener: &UnixListener, running: &std::thread::JoinHandle<Result<(),String>>) -> Option<UnixStream> {
            let until = Instant::now()+Duration::from_secs(4);
            loop {
                match listener.accept() {
                    // macOS : la connexion acceptée hérite du mode non bloquant
                    // de l'écoute ; Linux non. Le banc lit ensuite avec délai.
                    Ok((stream,_)) => { stream.set_nonblocking(false).unwrap(); return Some(stream) },
                    Err(error) if error.kind()==std::io::ErrorKind::WouldBlock => {},
                    Err(error) => panic!("accept: {error}"),
                }
                if running.is_finished() || Instant::now()>=until { return None; }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let Some(first) = accept(&listener,&running) else {
            panic!("pas de première connexion : {:?}",running.join().unwrap());
        };
        first.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut reader = BufReader::new(first);
        let mut line = String::new();reader.read_line(&mut line).unwrap();
        assert!(matches!(serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),WrapperToDaemon::DaemonIdentityRequest));
        drop(reader); // Coupure pendant le redémarrage du maître, avant capacité/credential.
        let Some(mut second) = accept(&listener,&running) else {
            panic!("le worker est sorti au lieu de se reconnecter : {:?}",running.join().unwrap());
        };
        second.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut reader = BufReader::new(second.try_clone().unwrap());
        line.clear();reader.read_line(&mut line).unwrap();
        assert!(matches!(serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),WrapperToDaemon::DaemonIdentityRequest));
        let report = DaemonToWrapper::DaemonIdentityReport {host:"host".into(),db_path:"db".into(),instance_id:"daemon".into(),features:vec!["remote_worker_v1".into()]};
        writeln!(second,"{}",serde_json::to_string(&report).unwrap()).unwrap();
        line.clear();reader.read_line(&mut line).unwrap();
        assert!(matches!(serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),WrapperToDaemon::WorkerHello{..}));
        // Un refus de protocole permanent reste fatal, sans boucle ni downgrade.
        writeln!(second,"{}",serde_json::to_string(&DaemonToWrapper::Nack{id:"stop-test".into(),reason:"closed".into()}).unwrap()).unwrap();
        assert_eq!(running.join().unwrap(),Err("worker_handshake_invalid".into()));
    }


    #[test]
    fn spec221_c19_worker_reader_reopens_receipt_but_rejects_new_prepare_without_effects() {
        use bridget_transport::protocol::{WorkspaceRequest,ProviderPermissions};
        use crate::delegation::{WorkspaceRecord,WorkspaceState,WorkspaceStrategy};
        let fixture=Fixture::new();let store=fixture.open();
        let mut command=command();command.instance_id.clear();command.generation=0;
        let record=WorkspaceRecord {
            strategy:WorkspaceStrategy::CommitView, commit_view:None,state:WorkspaceState::Kept,
            toplevel:"/private/c19-manifests".into(),path:"/private/source-views/fixture".into(),
            branch:"private-manifest".into(),base_commit:"a".repeat(40),
            head_commit:None,branch_commit:None,head_on_branch:None,commit_count:None,
            commits:vec![],truncated:false,dirty:None,
        };
        store.persist(&store.workspace_path(&command),&WorkspacePreparationReceipt {
            version:STORE_VERSION,worker:worker(),command:command.clone(),
            prepare_digest:"a".repeat(64),fenced_previous:None,record:Some(record.clone()),commit_view:None,
        }).unwrap();
        drop(store);
        let reopened=fixture.open();reopened.health().unwrap();
        let receipt=proof198::read_workspace_receipt(&reopened.root,&worker(),&command).unwrap().unwrap();
        assert!(same_workspace_anchor(&record,&serde_json::to_value(receipt.record.unwrap()).unwrap()));
        assert_eq!(reopened.inventory().unwrap().3,1);
        drop(reopened);
        let mut runtime=runtime_fixture(&fixture);
        let before=runtime.store.inventory().unwrap();
        let request=WorkerPrepare { service_tier: None,
            machine_id:"cartae".into(),project_id:"c19".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin".into(),origin_cwd:"/origin/project".into(),source_root:"/origin/project".into(),
            remote_root:"/unavailable/project".into(),source_root_identity:"fixture:c19".into(),relative_cwd:".".into(),
            base_commit:Some("a".repeat(40)),agent_type:"codex".into(),model:"fixture".into(),effort:None,
            posture:Some(SpawnPosture::Discovery),workspace:Some(WorkspaceRequest::CommitView {}),
            parent:ProviderPermissions {
                version:1,source:"native_wrapper".into(),run_id:"run-c19".into(),
                provider_session_id:"session-c19".into(),provider_instance_id:"parent-c19".into(),
                revision:1,driver:"codex_app_server".into(),cwd:"/origin/project".into(),
                runtime_mode:"full-access".into(),interaction_mode:"default".into(),
                provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never",
                    "approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
            },
        };
        // Validation pure reste capable de relire des reçus futurs. Le refus
        // d'exécution est au port, pas dans leur désérialisation.
        request.validate().unwrap();
        assert!(matches!(runtime.prepare(&command,&request),Err(WorkerRefusal::CapabilityRequired)));
        let after=runtime.store.inventory().unwrap();
        assert_eq!((before.2,before.3),(after.2,after.3));
        assert!(runtime.prepared.is_empty());
    }


    fn c19_store_fixture(fixture: &Fixture) -> (
        WorkerStore, WorkerCommandIdentity, WorkerPrepare, crate::workspace_git::ViewIntent,
    ) {
        let source = fixture.0.join("repo");
        create_private_dir(&source).unwrap();
        let source = source.canonicalize().unwrap();
        for args in [vec!["init","-q"],vec!["add","."],vec!["commit","-qm","fixture"]] {
            if args[0] == "add" {
                fs::write(source.join("tracked"),b"committed").unwrap();
                fs::write(source.join("large.bin"),vec![b'x';200000]).unwrap();
            }
            let mut cmd = std::process::Command::new("git");
            for (name, _) in std::env::vars_os().filter(|(name,_)| name.to_string_lossy().starts_with("GIT_")) {
                cmd.env_remove(name);
            }
            let status = cmd.env("GIT_CONFIG_GLOBAL","/dev/null").env("GIT_CONFIG_NOSYSTEM","1")
                .arg("-C").arg(&source)
                .args(["-c","user.name=Fixture","-c","user.email=fixture@example.invalid",
                    "-c","commit.gpgSign=false"]).args(args).status().unwrap();
            assert!(status.success());
        }
        let output = crate::workspace_git::run_git(&source,&["rev-parse","--verify","--end-of-options","HEAD^{commit}"],Duration::from_secs(3)).unwrap();
        assert!(output.success());
        let base = output.stdout.trim().to_owned();
        let data = fixture.0.join("views");
        create_private_dir(&data).unwrap();
        let data = data.canonicalize().unwrap();
        let mut command = command(); command.instance_id.clear(); command.generation=0;
        let request=WorkerPrepare { service_tier: None,
            machine_id:"cartae".into(),project_id:"c19".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin".into(),origin_cwd:"/origin/project".into(),source_root:"/origin/project".into(),
            remote_root:source.to_string_lossy().into(),source_root_identity:"fixture:c19".into(),relative_cwd:".".into(),
            base_commit:Some(base.clone()),agent_type:"codex".into(),model:"fixture".into(),effort:None,
            posture:Some(SpawnPosture::Discovery),
            workspace:Some(bridget_transport::protocol::WorkspaceRequest::CommitView {}),
            parent:ProviderPermissions {
                version:1,source:"native_wrapper".into(),run_id:"run-c19".into(),
                provider_session_id:"session-c19".into(),provider_instance_id:"parent-c19".into(),
                revision:1,driver:"codex_app_server".into(),cwd:"/origin/project".into(),
                runtime_mode:"full-access".into(),interaction_mode:"default".into(),
                provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never",
                    "approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
            },
        };
        request.validate().unwrap();
        let binding = crate::workspace_git::ViewBinding { worker:worker(), command:command.clone(),
            prepare_digest:json_digest(&(&command,&request)).unwrap() };
        let intent = crate::workspace_git::ViewIntent::plan(binding,&source,&data,&base).unwrap();
        let state = fixture.0.join("state");
        create_private_dir(&state).unwrap();
        let store = WorkerStore::open(&state.canonicalize().unwrap().join("store"), worker()).unwrap();
        (store,command,request,intent)
    }

    #[test]
    fn spec221_c19_worker_reserves_view_bytes_before_private_writes_and_keeps_reservation_on_reopen() {
        let fixture=Fixture::new();
        let (store,command,request,intent)=c19_store_fixture(&fixture);
        let before=store.inventory().unwrap();
        store.reserve_view_intent(&command,&request,&intent).unwrap();
        let after=store.inventory().unwrap();
        assert_eq!(after.3,1);
        let receipt=proof198::read_workspace_receipt(&store.root,&worker(),&command).unwrap().unwrap();
        let receipt_bytes=fs::metadata(store.workspace_path(&command)).unwrap().len();
        assert_eq!(after.2-before.2,receipt_bytes+MAX_METADATA_BYTES as u64+intent.reserved_bytes().unwrap());
        assert_eq!(receipt.commit_view.as_ref().unwrap().key,intent.binding.key().unwrap());
        assert!(receipt.record.is_none());
        assert!(!intent.destination().unwrap().exists()); // réservation, pas export implicite
        let proof=store.view_root().join(format!("{}.intent.json",intent.binding.key().unwrap()));
        let original=fs::read(&proof).unwrap();
        store.reserve_view_intent(&command,&request,&intent).unwrap();
        assert_eq!(fs::read(&proof).unwrap(),original);
        let expected=(after.2,after.3);
        let store_root=store.root.clone();
        drop(store);
        let reopened=WorkerStore::open(&store_root,worker()).unwrap();
        let after=reopened.inventory().unwrap();
        assert_eq!((after.2,after.3),expected);
        reopened.reserve_view_intent(&command,&request,&intent).unwrap();
        fs::write(&proof,b"{").unwrap();
        assert!(reopened.reserve_view_intent(&command,&request,&intent).is_err());
        assert_eq!(fs::read(&proof).unwrap(),b"{");
        reopened.health().unwrap(); // refus de cette préparation, pas quarantaine globale
    }

    #[test]
    fn spec221_c19_worker_view_quota_refuses_before_receipt_or_proof_creation() {
        let fixture=Fixture::new();
        let (store,command,request,intent)=c19_store_fixture(&fixture);
        let filler=store.root.join("unrelated-retained-file");
        write_private_file_atomic(&filler,b"").unwrap();
        fs::OpenOptions::new().write(true).open(&filler).unwrap()
            .set_len(MAX_WORKER_STORE_BYTES as u64-2*MAX_METADATA_BYTES as u64).unwrap();
        assert_eq!(store.inventory().unwrap().2,MAX_WORKER_STORE_BYTES as u64);
        assert!(matches!(store.reserve_view_intent(&command,&request,&intent),Err(WorkerStoreError::Quota)));
        assert!(!store.workspace_path(&command).exists());
        assert!(!store.view_root().exists());
        assert!(!intent.destination().unwrap().exists());
        assert_eq!(fs::metadata(&filler).unwrap().len(),MAX_WORKER_STORE_BYTES as u64-2*MAX_METADATA_BYTES as u64);
    }

    #[test]
    fn spec221_c19_worker_view_binding_root_posture_and_unknown_orphans_remain_separate() {
        let fixture=Fixture::new();
        let (store,command,request,mut intent)=c19_store_fixture(&fixture);
        let old_source=intent.source.clone();
        intent.source=fixture.0.canonicalize().unwrap();
        assert!(store.reserve_view_intent(&command,&request,&intent).is_err());
        intent.source=old_source;
        let mut changed=request.clone(); changed.posture=Some(SpawnPosture::Development);
        assert!(store.reserve_view_intent(&command,&changed,&intent).is_err());
        assert!(!store.workspace_path(&command).exists());
        store.reserve_view_intent(&command,&request,&intent).unwrap();
        let original_root=intent.data_root.clone();
        let alternate=fixture.0.join("alternate-views");create_private_dir(&alternate).unwrap();
        intent.data_root=alternate.canonicalize().unwrap();
        assert!(matches!(store.reserve_view_intent(&command,&request,&intent),Err(WorkerStoreError::Conflict)));
        intent.data_root=original_root;
        let unknown="b".repeat(64);
        for suffix in ["intent.json","published.json"] {
            write_private_file_atomic(&store.view_root().join(format!("{unknown}.{suffix}")),b"{}").unwrap();
        }
        write_private_file_atomic(&store.view_root().join("unrecognized"),b"kept").unwrap();
        write_private_file_atomic(&store.view_root().join(format!("{}-garbage",intent.binding.key().unwrap())),b"kept").unwrap();
        let (..,count)=store.inventory().unwrap();
        assert_eq!(count,4); // reçu + un groupe orphelin + deux fichiers non reconnus
        let receipt=WorkspacePreparationReceipt { version:STORE_VERSION,worker:worker(),command,
            prepare_digest:"a".repeat(64),fenced_previous:None,record:None,commit_view:None };
        assert!(serde_json::to_value(&receipt).unwrap().get("commit_view").is_none());
    }


    #[test]
    fn spec221_c19_start_fence_must_not_ignore_live_source_export() {
        let fixture=Fixture::new();
        let (store,prep,request,intent)=c19_store_fixture(&fixture);
        store.reserve_view_intent(&prep,&request,&intent).unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&store.view_root().canonicalize().unwrap()).unwrap();
        let mut env:BTreeMap<String,std::ffi::OsString>=["PATH","HOME","TMPDIR"].iter()
            .filter_map(|key|std::env::var_os(key).map(|value|((*key).into(),value))).collect();
        env.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let mut job=match crate::workspace_git::ViewJob::start(&proofs,&intent,&crate::workspace_git::ViewJobRequest::Export,&std::env::current_exe().unwrap(),&env,
            |marker|store.authorize_view_source(&prep,&intent,marker)
                .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)) {
            Ok(job)=>job,Err((cause,_))=>panic!("source fixture {cause:?}"),
        };
        let ready=store.view_root().join("stall-ready");
        let deadline=Instant::now()+Duration::from_secs(3);
        while !ready.exists() {assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
        let expires=worker_now()+300;
        let fenced=store.fence_unstarted(&command(),&placement(),expires,worker_now());
        let stored=proof198::read_workspace_receipt(&store.root,&worker(),&prep).unwrap().unwrap();
        let owned_marker=stored.commit_view.as_ref().unwrap().source_marker.clone().unwrap();
        assert!(!stored.commit_view.as_ref().unwrap().source_stopped);
        fs::remove_file(store.view_root().join(format!("view-{}.json",intent.binding.key().unwrap()))).unwrap();
        let missing_file=store.fence_unstarted(&command(),&placement(),expires,worker_now());
        // Toujours récolter le helper avant l'assertion RED, sans supprimer
        // de preuve sous son processus vivant.
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(store.view_root().join("allow-exit")).unwrap();
        let deadline=Instant::now()+Duration::from_secs(3);
        loop {
            if matches!(job.poll().unwrap(),crate::workspace_git::JobPoll::Stopped(_)) {break;}
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        }
        eprintln!("c19 live source fence result: {fenced:?}");
        assert!(matches!(fenced,Err(WorkerStoreError::Unknown)),
            "une garde de démarrage ne doit pas ignorer un export vivant");
        assert!(matches!(missing_file,Err(WorkerStoreError::Unknown)),
            "le reçu conserve le PID même si le marqueur fichier manque");
        store.fence_unstarted(&command(),&placement(),expires,worker_now()).unwrap();
        let stopped=proof198::read_workspace_receipt(&store.root,&worker(),&prep).unwrap().unwrap();
        assert!(stopped.commit_view.as_ref().unwrap().source_stopped);
        assert_eq!(stopped.commit_view.as_ref().unwrap().source_marker.as_ref(),Some(&owned_marker));
        assert!(matches!(store.authorize_view_source(&prep,&intent,&owned_marker),
            Err(WorkerStoreError::InvalidTransition)));
        assert!(matches!(store.reserve_view_intent(&prep,&request,&intent),
            Err(WorkerStoreError::InvalidTransition)));
        let store_root=store.root.clone();drop(store);
        let reopened=WorkerStore::open(&store_root,worker()).unwrap();
        reopened.fence_unstarted(&command(),&placement(),expires,worker_now()).unwrap();
    }


    #[test]
    fn spec221_c19_worker_publishes_only_after_source_group_stop_and_private_audit() {
        let fixture=Fixture::new();
        let (store,prep,request,intent)=c19_store_fixture(&fixture);
        store.reserve_view_intent(&prep,&request,&intent).unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&store.view_root().canonicalize().unwrap()).unwrap();
        assert!(matches!(store.publish_view_workspace(&prep,&intent),Err(WorkerStoreError::Unknown)));
        let env:BTreeMap<String,std::ffi::OsString>=["PATH","HOME","TMPDIR"].iter()
            .filter_map(|key|std::env::var_os(key).map(|value|((*key).into(),value))).collect();
        let mut job=match crate::workspace_git::ViewJob::start(&proofs,&intent,&crate::workspace_git::ViewJobRequest::Export,&std::env::current_exe().unwrap(),&env,
            |marker| {
                let mut false_marker=marker.clone();false_marker.birth+=1;
                assert!(matches!(store.authorize_view_source(&prep,&intent,&false_marker),Err(WorkerStoreError::Conflict)));
                store.authorize_view_source(&prep,&intent,marker)
                    .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)
            }) {
                Ok(job)=>job,
                Err((cause,mut retained))=>{
                    if let Some(child)=retained.as_mut() {assert!(child.stop(Duration::from_secs(2)).unwrap());}
                    panic!("source fixture {cause:?}");
                }
            };
        let deadline=Instant::now()+Duration::from_secs(10);
        loop {
            match job.poll().unwrap() {
                crate::workspace_git::JobPoll::Stopped(result)=>{assert_eq!(result,Ok(()));break;}
                _=>{assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
            }
        }
        // La récolte et la suppression du sidecar ne suffisent pas : le reçu
        // conserve encore son marqueur, source_stopped=false jusqu'au constat.
        let previous=proof198::read_workspace_receipt(&store.root,&worker(),&prep).unwrap().unwrap();
        assert!(previous.commit_view.as_ref().unwrap().source_marker.is_some());
        assert!(!previous.commit_view.as_ref().unwrap().source_stopped);
        assert!(!store.view_root().join(format!("view-{}.json",intent.binding.key().unwrap())).exists());
        let record=store.publish_view_workspace(&prep,&intent).unwrap();
        let current=proof198::read_workspace_receipt(&store.root,&worker(),&prep).unwrap().unwrap();
        assert!(current.commit_view.as_ref().unwrap().source_stopped);
        assert_eq!(record.dirty,Some(false));
        assert!(same_workspace_anchor(&record,&serde_json::to_value(current.record.unwrap()).unwrap()));
        let again=store.publish_view_workspace(&prep,&intent).unwrap();
        assert!(same_workspace_anchor(&record,&serde_json::to_value(again).unwrap()));
        let root=store.root.clone();drop(store);
        let reopened=WorkerStore::open(&root,worker()).unwrap();
        assert!(same_workspace_anchor(&record,&serde_json::to_value(
            reopened.publish_view_workspace(&prep,&intent).unwrap()).unwrap()));
        let file=Path::new(&record.path).join("tracked");
        fs::set_permissions(&file,fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&file,b"divergent").unwrap();
        assert!(reopened.publish_view_workspace(&prep,&intent).is_err());
        assert_eq!(fs::read(&file).unwrap(),b"divergent");
    }


    #[test]
    fn spec221_c19_runtime_owns_export_until_durable_publication_and_never_restarts_it() {
        let fixture=Fixture::new();
        let (unused_store,prep,request,intent)=c19_store_fixture(&fixture);
        drop(unused_store);
        let mut runtime=runtime_fixture(&fixture);
        let record=runtime.prepare_view_source(&prep,&request,intent.clone()).unwrap();
        assert!(runtime.view_sources.is_empty());
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        let receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        assert!(receipt.commit_view.as_ref().unwrap().source_stopped);
        let pid=receipt.commit_view.as_ref().unwrap().source_marker.as_ref().unwrap().pgid;
        let replay=runtime.prepare_view_source(&prep,&request,intent.clone()).unwrap();
        assert!(same_workspace_anchor(&record,&serde_json::to_value(replay).unwrap()));
        let receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        assert_eq!(receipt.commit_view.as_ref().unwrap().source_marker.as_ref().unwrap().pgid,pid);
        let config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,
            credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
            registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut reopened=WorkerRuntime::new(config).unwrap();
        let restored=reopened.prepare_view_source(&prep,&request,intent).unwrap();
        assert!(same_workspace_anchor(&record,&serde_json::to_value(restored).unwrap()));
        assert!(reopened.view_sources.is_empty());
    }


    #[test]
    fn spec221_c19_runtime_fence_stops_only_owned_source_after_validating_the_request() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        runtime.store.reserve_view_intent(&prep,&request,&intent).unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap()).unwrap();
        let mut env=runtime.source.clone();env.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let child=match crate::workspace_git::ViewJob::start(&proofs,&intent,&crate::workspace_git::ViewJobRequest::Export,&std::env::current_exe().unwrap(),&env,
            |marker|runtime.store.authorize_view_source(&prep,&intent,marker)
                .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)) {
                Ok(child)=>child,
                Err((cause,mut retained))=>{
                    if let Some(child)=retained.as_mut(){assert!(child.stop(Duration::from_secs(2)).unwrap());}
                    panic!("source fixture {cause:?}");
                }
            };
        let key=marker_key(&prep);
        runtime.view_sources.insert(key.clone(),ViewSourceProcess {binding:intent.binding.clone(),child,materialization:None});
        let root=runtime.store.view_root();
        let until=Instant::now()+Duration::from_secs(3);
        while !root.join("stall-ready").exists(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));}
        assert_eq!(runtime.fence_unstarted(&command(),&placement(),0).unwrap_err(),WorkerRefusal::InvalidEnvelope);
        assert_eq!(runtime.view_sources.get_mut(&key).unwrap().child.poll().unwrap(),crate::workspace_git::JobPoll::Running);
        let mut foreign=command();foreign.task_id="unrelated-task".into();foreign.command_id="unrelated-command".into();
        runtime.fence_unstarted(&foreign,&placement(),worker_now()+300).unwrap();
        assert_eq!(runtime.view_sources.get_mut(&key).unwrap().child.poll().unwrap(),crate::workspace_git::JobPoll::Running);
        let mut other_attempt=command();other_attempt.command_id="other-attempt".into();
        assert_eq!(runtime.fence_unstarted(&other_attempt,&placement(),worker_now()+300).unwrap_err(),WorkerRefusal::UnknownExecution);
        assert!(runtime.cancel_signal_issued.is_empty(),"a different attempt must not signal this source");
        assert_eq!(runtime.view_sources.get_mut(&key).unwrap().child.poll().unwrap(),crate::workspace_git::JobPoll::Running);
        assert_eq!(runtime.fence_unstarted(&command(),&placement(),worker_now()+300).unwrap_err(),WorkerRefusal::UnknownExecution);
        assert_eq!(runtime.view_sources.get_mut(&key).unwrap().child.poll().unwrap(),crate::workspace_git::JobPoll::StopUnconfirmed);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(root.join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty(){
            runtime.collect_view_sources();
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        let stopped=proof198::read_workspace_receipt(&runtime.store.root,&worker(),&prep).unwrap().unwrap();
        assert!(stopped.commit_view.as_ref().unwrap().source_stopped);
        runtime.fence_unstarted(&command(),&placement(),worker_now()+300).unwrap();
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
    }


    #[test]
    fn spec221_c19_restarted_runtime_recovers_receipt_identity_without_second_export() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        runtime.store.reserve_view_intent(&prep,&request,&intent).unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap()).unwrap();
        let mut env=runtime.source.clone();env.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let child=match crate::workspace_git::ViewJob::start(&proofs,&intent,&crate::workspace_git::ViewJobRequest::Export,&std::env::current_exe().unwrap(),&env,
            |marker|runtime.store.authorize_view_source(&prep,&intent,marker)
                .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)) {
                Ok(child)=>child,
                Err((cause,mut retained))=>{
                    if let Some(child)=retained.as_mut(){assert!(child.stop(Duration::from_secs(2)).unwrap());}
                    panic!("source fixture {cause:?}");
                }
            };
        let root=runtime.store.view_root();
        let until=Instant::now()+Duration::from_secs(3);
        while !root.join("stall-ready").exists(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));}
        // Le fixture garde uniquement le Child pour sa récolte finale. Le
        // nouveau worker n'en a ni Child, ni verrou, ni propriétaire en RAM.
        let key=marker_key(&prep);
        runtime.view_sources.insert(key.clone(),ViewSourceProcess {binding:intent.binding.clone(),child,materialization:None});
        let mut held=runtime.view_sources.remove(&key).unwrap();
        let config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,
            credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
            registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut reopened=WorkerRuntime::new(config).unwrap();
        assert!(reopened.view_sources.is_empty());
        let original=reopened.store.checked_view_receipt(&prep,&intent).unwrap();
        let marker=original.commit_view.as_ref().unwrap().source_marker.as_ref().unwrap().clone();
        assert_eq!(reopened.prepare_view_source(&prep,&request,intent.clone()).unwrap_err(),(WorkerRefusal::UnknownExecution,Some(crate::workspace_git::ExportFailure::StopUnconfirmed)));
        assert!(reopened.view_sources.is_empty());
        assert_eq!(reopened.store.checked_view_receipt(&prep,&intent).unwrap().commit_view.as_ref().unwrap().source_marker.as_ref(),Some(&marker));

        // Même PID mais naissance différente : aucun signal, même si le
        // sidecar contient encore l'identité originale du processus vivant.
        let mut wrong=reopened.store.checked_view_receipt(&prep,&intent).unwrap();
        wrong.commit_view.as_mut().unwrap().source_marker.as_mut().unwrap().birth=marker.birth.checked_add(1).unwrap();
        reopened.store.persist(&reopened.store.workspace_path(&prep),&wrong).unwrap();
        assert_eq!(reopened.fence_unstarted(&command(),&placement(),worker_now()+300).unwrap_err(),WorkerRefusal::UnknownExecution);
        assert!(reopened.cancel_signal_issued.is_empty());
        assert_eq!(held.child.poll().unwrap(),crate::workspace_git::JobPoll::Running);
        reopened.store.persist(&reopened.store.workspace_path(&prep),&original).unwrap();
        // Le reçu reste l'autorité lorsque le sidecar a disparu.
        fs::remove_file(root.join(format!("view-{}.json",intent.binding.key().unwrap()))).unwrap();
        assert_eq!(reopened.fence_unstarted(&command(),&placement(),worker_now()+300).unwrap_err(),WorkerRefusal::UnknownExecution);
        let stop_key=format!("view-source-{}",intent.binding.key().unwrap());
        assert_eq!(reopened.cancel_signal_issued,BTreeSet::from([stop_key]));
        assert_eq!(reopened.fence_unstarted(&command(),&placement(),worker_now()+300).unwrap_err(),WorkerRefusal::UnknownExecution);
        assert_eq!(reopened.cancel_signal_issued.len(),1);
        assert!(!reopened.store.checked_view_receipt(&prep,&intent).unwrap().commit_view.as_ref().unwrap().source_stopped);

        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(root.join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        loop {
            if matches!(held.child.poll().unwrap(),crate::workspace_git::JobPoll::Stopped(_)){break;}
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        reopened.fence_unstarted(&command(),&placement(),worker_now()+300).unwrap();
        let stopped=proof198::read_workspace_receipt(&reopened.store.root,&worker(),&prep).unwrap().unwrap();
        assert!(stopped.commit_view.as_ref().unwrap().source_stopped);
        assert_eq!(stopped.commit_view.as_ref().unwrap().source_marker.as_ref(),Some(&marker));
        assert!(!intent.destination().unwrap().exists());
        assert!(reopened.view_sources.is_empty());
    }


    fn c19_preparation_fixture(fixture:&Fixture)
        ->(WorkerRuntime,WorkerCommandIdentity,WorkerPrepare,crate::workspace_git::ViewIntent) {
        let (unused,prep,request,intent)=c19_store_fixture(fixture);drop(unused);
        let mut runtime=runtime_fixture(fixture);
        let source=Path::new(&request.remote_root);
        // Cette fixture garde dépôt et données à côté du runtime. La protection
        // couvre tout le parent de la clé : isoler la clé, sans retirer de garde.
        eprintln!("C19 fixture source_inside_credential_parent={}",
            source.starts_with(runtime.config.credential_path.parent().unwrap()));
        let credentials=fixture.0.join("credentials");create_private_dir(&credentials).unwrap();
        let key=credentials.join("worker-key");
        fs::rename(&runtime.config.credential_path,&key).unwrap();
        runtime.config.credential_path=key;
        let status=std::process::Command::new("git").arg("-C").arg(source)
            .args(["config","--local","remote.origin.url","fixture:c19"]).status().unwrap();
        assert!(status.success());
        write_json(&runtime.config.registry_path,&serde_json::json!({
            "agents":{"codex":{"command":"/bin/echo","args":["app-server"],"protocol":"codex_app_server",
                "capabilities":{"execution_paths":["codex_app_server"],"models":{"fixture":{"efforts":[]}}}}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],
                "mappings":{"c19":{"source_host":"origin","source_root":"/origin/project","remote_root":source,
                    "expected_git_identity":"fixture:c19","mapping_revision":1}}}}
        })).unwrap();
        (runtime,prep,request,intent)
    }

    #[test]
    fn spec221_c05_worker_keeps_the_choice_through_readiness_and_release_recheck() {
        let fixture=Fixture::new();
        let (runtime,command,mut request,_intent)=c19_preparation_fixture(&fixture);
        request.workspace=None;
        request.service_tier=Some("priority".into());
        let parent=request.parent.clone();
        let mut value:serde_json::Value=serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap()).unwrap();
        value["agents"]["codex"]["capabilities"]["models"]["fixture"]["service_tiers"]=serde_json::json!(["priority","default"]);
        write_json(&runtime.config.registry_path,&value).unwrap();
        let registry=runtime.registry().unwrap();
        let mapping=registry.project_mapping("cartae","c19").unwrap();
        let cwd=Path::new(&request.remote_root).canonicalize().unwrap();
        let mut operation=crate::native_permissions::PermissionReadOperation::default();
        let readiness=runtime.readiness_at(&registry,&request,mapping,&cwd,&mut operation,None).unwrap();
        operation.finish().unwrap();
        assert!(readiness.definition.args.iter().any(|arg|arg=="service_tier=\"priority\""));
        assert_eq!(request.parent,parent);
        let placement=RemotePlacement {
            machine_id:request.machine_id.clone(),worker_id:runtime.identity.worker_id.clone(),
            provisioning_revision:request.provisioning_revision,project_id:request.project_id.clone(),
            mapping_revision:request.mapping_revision,origin_host:request.origin_host.clone(),
            origin_cwd:request.origin_cwd.clone(),source_root_identity:request.source_root_identity.clone(),
            relative_cwd:request.relative_cwd.clone(),execution_cwd:cwd.to_string_lossy().into_owned(),
            base_commit:request.base_commit.clone(),definition_digest:readiness.definition.digest.clone(),
            permission_digest:json_digest(&readiness.child_policy).unwrap(),
        };
        let prepared=PreparedRuntime {
            command,request:request.clone(),routing:None,expires:Instant::now()+Duration::from_secs(30),
            prepared:WorkerPrepared {placement,workspace:None,token:"options-test".into(),
                definition:readiness.definition.clone(),child_policy:readiness.child_policy,
                effective_posture:readiness.effective_posture,prepared_at:worker_now()},
        };
        assert_eq!(runtime.recheck_at(&prepared,true).unwrap()
            .resolved_definition("codex").unwrap(),readiness.definition);
        // A future restart reads the same choice from Prepare; no new selection.
        let persisted:WorkerPrepare=serde_json::from_slice(&serde_json::to_vec(&request).unwrap()).unwrap();
        assert_eq!(persisted.service_tier.as_deref(),Some("priority"));
        assert!(runtime.store.commands().unwrap().is_empty());
        value["agents"]["codex"]["capabilities"]["models"]["fixture"]["service_tiers"]=serde_json::json!(["default"]);
        write_json(&runtime.config.registry_path,&value).unwrap();
        assert_eq!(runtime.recheck_at(&prepared,true).unwrap_err(),WorkerRefusal::DefinitionChanged);
        assert_eq!(request.parent,parent);
        assert!(runtime.store.commands().unwrap().is_empty());
    }

    fn c19_plan(intent:&crate::workspace_git::ViewIntent)->crate::workspace_git::ViewPlan {
        crate::workspace_git::ViewPlan::new(intent.binding.clone(),&intent.source,&intent.data_root,
            intent.base_commit(),&intent.source,"fixture:c19").unwrap()
    }

    fn c19_prepared_view_fixture(fixture:&Fixture)
        ->(WorkerRuntime,crate::workspace_git::ViewIntent,PreparedRuntime) {
        let (mut runtime,prep,request,intent)=c19_preparation_fixture(fixture);
        let record=runtime.prepare_view_source(&prep,&request,intent.clone()).unwrap();
        let registry=runtime.registry().unwrap();
        let (mut mapping,_)=runtime.mapping_for(&request,&registry,None).unwrap();
        let (root,cwd)=intent.execution_paths(&mapping.remote_root).unwrap();
        // Même normalisation que poll_prepared_view ; conserver "/." ici
        // fabriquait une divergence de placement textuel dans la fixture.
        mapping.remote_root=root.canonicalize().unwrap();
        let cwd=cwd.canonicalize().unwrap();
        let mut operation=crate::native_permissions::PermissionReadOperation::default();
        // Construire une entrée conservée par les vrais projecteurs de droits,
        // sans lever inspect_candidate/prepare_for ni créer une admission.
        let pinned=registry.clone().pin_delegation_model(&request.agent_type,&request.model,
            request.effort.as_deref()).unwrap();
        let (child_policy,effective_posture)=crate::native_permissions::project_remote_permissions_in_operation(
            &request.parent,pinned.get(&request.agent_type).unwrap(),&mapping,&cwd,
            &runtime.source,request.posture,&runtime.protected(),&mut operation,true,false).unwrap();
        let scoped=pinned.for_inherited_delegation_in_operation(&request.agent_type,
            &request.model,request.effort.as_deref(),&child_policy,&mut operation).unwrap();
        let inspected=CandidateReadiness {
            definition:scoped.resolved_definition(&request.agent_type).unwrap(),
            child_policy,effective_posture,
        };
        operation.finish().unwrap();
        let prepared=PreparedRuntime {
            command:prep.clone(),request:request.clone(),routing:None,
            prepared:WorkerPrepared {
                placement:RemotePlacement {
                    machine_id:request.machine_id.clone(),worker_id:runtime.identity.worker_id.clone(),
                    provisioning_revision:request.provisioning_revision,project_id:request.project_id.clone(),
                    mapping_revision:request.mapping_revision,origin_host:request.origin_host.clone(),
                    origin_cwd:request.origin_cwd.clone(),source_root_identity:request.source_root_identity.clone(),
                    relative_cwd:request.relative_cwd.clone(),execution_cwd:cwd.to_string_lossy().into(),
                    base_commit:request.base_commit.clone(),definition_digest:inspected.definition.digest.clone(),
                    permission_digest:json_digest(&inspected.child_policy).unwrap(),
                },
                workspace:Some(serde_json::to_value(&record).unwrap()),token:"retained-view-fixture".into(),
                definition:inspected.definition,child_policy:inspected.child_policy,
                effective_posture:inspected.effective_posture,prepared_at:worker_now(),
            },
            expires:Instant::now()+Duration::from_secs(30),
        };
        (runtime,intent,prepared)
    }

    #[test]
    fn spec221_c19_inherited_directory_only_in_live_b_never_enters_prepared_view_a() {
        let fixture=Fixture::new();
        let (mut runtime,command,mut request,previous)=c19_preparation_fixture(&fixture);
        request.parent.cwd="/origin/project/.worktrees/source-parent".into();
        request.parent.provider_policy["sandbox_policy"]=serde_json::json!({
            "type":"workspaceWrite","networkAccess":false,
            "writableRoots":["/origin/project/.worktrees/source-parent/notes"],
            "excludeSlashTmp":true,"excludeTmpdirEnvVar":true});
        let source=Path::new(&request.remote_root);
        fs::create_dir(source.join("notes")).unwrap();
        fs::write(source.join("notes/untracked"),b"live-B-only").unwrap();
        let binding=commit_view_preparation_binding(&runtime.identity,&command,&request).unwrap();
        let seed=crate::workspace_git::ViewPlan::new(binding,&previous.source,&previous.data_root,
            request.base_commit.as_deref().unwrap(),&previous.source,"fixture:c19").unwrap();
        let read=runtime.begin_prepared_view(&command,&request,None,seed).unwrap();
        let deadline=Instant::now()+Duration::from_secs(10);
        loop {
            match runtime.poll_prepared_view(&read) {
                Err((WorkerRefusal::PermissionsChanged,None))=>break,
                Ok(None)=>{},
                other=>panic!("aucune publication avec le dossier vivant B : {other:?}"),
            }
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(runtime.prepared.is_empty() && runtime.running.is_empty() && runtime.held.is_empty());
        let receipt=runtime.store.view_receipt_for_binding(&read.source.binding).unwrap();
        assert!(receipt.commit_view.as_ref().unwrap().source_stopped);
        let record=receipt.record.unwrap();
        assert!(!Path::new(&record.path).join("notes").exists());
        assert_eq!(fs::read(source.join("notes/untracked")).unwrap(),b"live-B-only");
    }

    #[test]
    fn spec221_c19_linked_parent_prepares_from_mirror_root_without_linked_destination() {
        let fixture=Fixture::new();
        let (mut runtime,command,mut request,previous)=c19_preparation_fixture(&fixture);
        request.parent.cwd="/origin/project/.worktrees/source-parent".into();
        request.parent.provider_policy["sandbox_policy"]=serde_json::json!({"type":"readOnly","networkAccess":false});
        assert_eq!(request.origin_cwd,request.source_root);
        assert_eq!(request.relative_cwd,".");
        let source=Path::new(&request.remote_root);
        assert!(!source.join(".worktrees/source-parent").exists());
        let before=(fs::read(source.join(".git/HEAD")).unwrap(),
            fs::read(source.join(".git/index")).unwrap(),
            crate::workspace_git::run_git(source,&["worktree","list","--porcelain"],
                Duration::from_secs(3)).unwrap().stdout);
        let parent_before=serde_json::to_vec(&request.parent).unwrap();
        let binding=commit_view_preparation_binding(&runtime.identity,&command,&request).unwrap();
        let seed=crate::workspace_git::ViewPlan::new(binding,&previous.source,&previous.data_root,
            request.base_commit.as_deref().unwrap(),&previous.source,"fixture:c19").unwrap();
        let read=runtime.begin_prepared_view(&command,&request,None,seed).unwrap();
        let deadline=Instant::now()+Duration::from_secs(10);
        let prepared=loop {
            if let Some(prepared)=runtime.poll_prepared_view(&read).unwrap(){break prepared;}
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(prepared.effective_posture,SpawnPosture::Discovery);
        assert_eq!(prepared.child_policy["sandbox_policy"],serde_json::json!({"type":"readOnly","networkAccess":false}));
        assert_eq!(fs::read(Path::new(&prepared.placement.execution_cwd).join("tracked")).unwrap(),b"committed");
        let kept=runtime.prepared.get(&command.command_id).unwrap();
        for release in [false,true] {
            let mut report=None;
            runtime.read_view().recheck_at_with_workspace(kept,release,None,
                &read::ReadTrace::default(),&mut report).unwrap();
            assert_eq!(report.unwrap().unwrap().dirty,Some(false));
        }
        assert_eq!(serde_json::to_vec(&request.parent).unwrap(),parent_before);
        assert!(!source.join(".worktrees/source-parent").exists());
        assert_eq!((fs::read(source.join(".git/HEAD")).unwrap(),
            fs::read(source.join(".git/index")).unwrap(),
            crate::workspace_git::run_git(source,&["worktree","list","--porcelain"],
                Duration::from_secs(3)).unwrap().stdout),before);
        assert!(runtime.held.is_empty() && runtime.running.is_empty());
        assert!(runtime.view_sources.is_empty());
    }

    #[test]
    fn spec221_c19_prepare_begin_publishes_only_after_poll_and_replays_same_token_without_shared_refresh() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let source=Path::new(&request.remote_root);
        fs::write(source.join("tracked"),b"descendant B").unwrap();
        for args in [vec!["add","tracked"],vec!["commit","-qm","descendant"]] {
            assert!(std::process::Command::new("git").arg("-C").arg(source)
                .args(["-c","user.name=Fixture","-c","user.email=fixture@example.invalid","-c","commit.gpgSign=false"])
                .args(args).status().unwrap().success());
        }
        fs::write(source.join("tracked"),b"dirty B").unwrap();
        let index=fs::read(source.join(".git/index")).unwrap();
        let head=crate::workspace_git::run_git(source,&["rev-parse","--verify","--end-of-options","HEAD^{commit}"],Duration::from_secs(3)).unwrap().stdout;
        let started=Instant::now();
        let mut read=runtime.begin_prepared_view(&command,&request,None,c19_plan(&intent)).unwrap();
        assert!(started.elapsed()<Duration::from_secs(3));
        assert!(runtime.prepared.is_empty() && runtime.held.is_empty() && runtime.running.is_empty());
        let deadline=Instant::now()+Duration::from_secs(10);
        let prepared=loop {
            if let Some(prepared)=runtime.poll_prepared_view(&read).unwrap() {break prepared;}
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(prepared.effective_posture,SpawnPosture::Discovery);
        assert_eq!(prepared.placement.base_commit,request.base_commit);
        assert_eq!(prepared.child_policy["sandbox_policy"]["type"],"readOnly");
        let cwd=Path::new(&prepared.placement.execution_cwd);
        assert!(cwd.starts_with(intent.destination().unwrap()));
        assert_eq!(fs::read(cwd.join("tracked")).unwrap(),b"committed");
        assert!(!cwd.join(".git").exists());
        assert_eq!(fs::read(source.join("tracked")).unwrap(),b"dirty B");
        assert_eq!(fs::read(source.join(".git/index")).unwrap(),index);
        assert_eq!(crate::workspace_git::run_git(source,&["rev-parse","--verify","--end-of-options","HEAD^{commit}"],Duration::from_secs(3)).unwrap().stdout,head);
        let receipt=fs::read(runtime.store.workspace_path(&command)).unwrap();
        let replay=loop {
            if let Some(replay)=runtime.poll_prepared_view(&read).unwrap(){break replay;}
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(replay.token,prepared.token);
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),receipt);
        assert!(runtime.view_sources.is_empty());
        runtime.prepared.get_mut(&command.command_id).unwrap().routing=Some(RoutingPreparation {
            connection_id:"different-connection".into(),engine:"codex".into(),
            workload:bridget_transport::protocol::AutoDelegateWorkload::General,
        });
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        runtime.prepared.get_mut(&command.command_id).unwrap().routing=None;
        read.request.base_commit=Some("f".repeat(40));
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        assert_eq!(runtime.prepared[&command.command_id].prepared.token,prepared.token);
        read.request=request.clone();
        runtime.prepared.get_mut(&command.command_id).unwrap().prepared.placement.execution_cwd="/different/view".into();
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        runtime.prepared.get_mut(&command.command_id).unwrap().prepared.placement.execution_cwd=prepared.placement.execution_cwd.clone();
        runtime.prepared.get_mut(&command.command_id).unwrap().prepared.workspace=None;
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        runtime.prepared.get_mut(&command.command_id).unwrap().prepared.workspace=prepared.workspace.clone();
        runtime.prepared.get_mut(&command.command_id).unwrap().expires=Instant::now()-Duration::from_secs(1);
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::PrepareExpired,None));
        assert_eq!(runtime.prepared[&command.command_id].prepared.token,prepared.token);
        assert!(matches!(runtime.prepare(&command,&request),Err(WorkerRefusal::CapabilityRequired)));
        let mut execution=command.clone();execution.instance_id="child-preparation".into();execution.generation=1;
        let now=worker_now();
        runtime.store.reserve_command(&execution,&"d".repeat(64),&prepared.placement,now+120,now).unwrap();
        runtime.prepared.remove(&command.command_id);
        let before=fs::read(runtime.store.workspace_path(&command)).unwrap();
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        assert!(matches!(runtime.begin_prepared_view(&command,&request,None,c19_plan(&intent)),
            Err((WorkerRefusal::CommandMismatch,None))));
        assert!(runtime.prepared.is_empty() && runtime.view_sources.is_empty());
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),before);
        runtime.store.set_phase(&execution,WorkerCommandPhase::Refused,now+1).unwrap();
        runtime.store.collect(now+WORKER_RECEIPT_RETENTION_SECS+100).unwrap();
        assert!(runtime.store.read_command(&execution).unwrap().is_some());
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
    }

    #[test]
    fn spec221_c19_managed_planning_reserves_before_manifest_and_adopts_once_after_stop() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=crate::workspace_git::ViewPlan::new(intent.binding.clone(),&intent.source,&intent.data_root,intent.base_commit(),&intent.source,"fixture:c19").unwrap();
        let before=runtime.store.inventory().unwrap();
        let read=runtime.begin_planned_view_source(&command,&request,seed.clone()).unwrap();
        let receipt=runtime.store.view_receipt_for_binding(&seed.binding).unwrap();
        let reserved=receipt.commit_view.as_ref().unwrap();
        assert!(reserved.source_attempted && reserved.source_marker.is_some() && !reserved.source_stopped);
        assert!(!reserved.planning.as_ref().unwrap().adopted);
        assert_eq!(reserved.intent_digest,seed.proof_digest().unwrap());
        assert_eq!(reserved.reserved_bytes,seed.reserved_bytes());
        assert!(runtime.store.inventory().unwrap().2-before.2>=seed.reserved_bytes());
        let deadline=Instant::now()+Duration::from_secs(10);
        let record=loop {
            if let Some(record)=runtime.poll_view_source(&read).unwrap(){break record;}
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(record.base_commit,intent.base_commit());
        let receipt=runtime.store.checked_view_receipt(&command,&intent).unwrap();
        assert!(receipt.commit_view.as_ref().unwrap().planning.as_ref().unwrap().adopted);
        let stable=fs::read(runtime.store.workspace_path(&command)).unwrap();
        runtime.store.adopt_view_plan(&seed.binding,&intent).unwrap();
        assert_eq!(stable,fs::read(runtime.store.workspace_path(&command)).unwrap());
        // Même graine et délai ; pas de second helper ni nouveau token.
        assert_eq!(runtime.begin_planned_view_source(&command,&request,seed).unwrap().binding,read.binding);
        assert!(runtime.view_sources.is_empty() && runtime.prepared.is_empty() && runtime.running.is_empty());
    }

    #[test]
    fn spec221_c19_managed_planning_crash_after_attempt_never_starts_a_second_helper() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=crate::workspace_git::ViewPlan::new(intent.binding.clone(),&intent.source,&intent.data_root,intent.base_commit(),&intent.source,"fixture:c19").unwrap();
        runtime.store.reserve_view_plan(&command,&request,&seed).unwrap();
        runtime.store.mark_view_source_attempt_binding(&seed.binding).unwrap();
        let before=fs::read(runtime.store.workspace_path(&command)).unwrap();
        assert!(matches!(runtime.begin_planned_view_source(&command,&request,seed),
            Err((WorkerRefusal::UnknownExecution,Some(crate::workspace_git::ExportFailure::StopUnconfirmed)))));
        assert_eq!(before,fs::read(runtime.store.workspace_path(&command)).unwrap());
        assert!(runtime.view_sources.is_empty() && !intent.destination().unwrap().exists());
    }

    #[test]
    fn spec221_c19_deferred_report_keeps_cancel_responsive_and_never_publishes_cancelled_audit() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_REPORT_HOLD".into(),"1".into());
        let read=runtime.begin_prepared_view(&command,&request,None,c19_plan(&intent)).unwrap();
        let deadline=Instant::now()+Duration::from_secs(5);
        while runtime.view_reports.is_empty() {
            assert!(runtime.poll_prepared_view(&read).unwrap().is_none());
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));
        }
        let before=fs::read(runtime.store.workspace_path(&command)).unwrap();
        let marker=runtime.store.view_receipt_for_binding(&read.source.binding).unwrap()
            .commit_view.as_ref().unwrap().source_marker.clone();
        assert!(marker.is_some());
        let started=Instant::now();
        assert!(runtime.poll_prepared_view(&read).unwrap().is_none());
        runtime.cancelled.lock().unwrap().insert(json_digest(&command).unwrap());
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),
            (WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::Cancelled)));
        assert!(started.elapsed()<Duration::from_millis(100));
        assert!(runtime.prepared.is_empty() && runtime.running.is_empty());
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),before);
        assert_eq!(runtime.view_reports.len(),1); // lecture toujours possédée
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-report")).unwrap();
        while !runtime.view_reports.is_empty() {
            runtime.collect_view_sources();
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));
        }
        let receipt=runtime.store.view_receipt_for_binding(&read.source.binding).unwrap();
        assert!(receipt.record.is_none());
        assert_eq!(receipt.commit_view.as_ref().unwrap().source_marker,marker);
        assert!(intent.destination().unwrap().is_dir());
        let budget=runtime.store.inventory().unwrap().2;
        runtime.conserve_commit_view(&command,&read.source.binding.prepare_digest).unwrap();
        assert!(runtime.store.inventory().unwrap().2<=budget);
        assert!(runtime.store.view_receipt_for_binding(&read.source.binding).unwrap().commit_view.unwrap().closed);
        assert!(intent.destination().unwrap().is_dir());
    }

    #[test]
    fn spec221_c19_unattempted_plan_recovers_only_exact_seed_then_conserves_without_erasing_files() {
        let fixture=Fixture::new();
        let (runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=c19_plan(&intent);let key=seed.binding.key().unwrap();
        runtime.store.reserve_view_plan(&command,&request,&seed).unwrap();
        let receipt_path=runtime.store.workspace_path(&command);
        let before=fs::read(&receipt_path).unwrap();
        let path=runtime.store.view_root().join(format!("{key}.plan.json"));
        fs::remove_file(&path).unwrap(); // coupure après reçu, avant graine
        runtime.store.reserve_view_plan(&command,&request,&seed).unwrap();
        assert_eq!(fs::read(&receipt_path).unwrap(),before);
        assert_eq!(crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap())
            .unwrap().load_plan(&key).unwrap(),seed);
        let before_budget=runtime.store.inventory().unwrap().2;
        let seed_bytes=fs::read(&path).unwrap();
        runtime.store.conserve_unpublished_view(&seed.binding).unwrap();
        let receipt=runtime.store.view_receipt_for_binding(&seed.binding).unwrap();
        assert!(receipt.commit_view.as_ref().unwrap().closed && receipt.record.is_none());
        assert!(!receipt.commit_view.as_ref().unwrap().source_attempted);
        assert!(before_budget-runtime.store.inventory().unwrap().2>8*1024*1024-32768);
        assert_eq!(fs::read(&path).unwrap(),seed_bytes);
        runtime.store.collect(worker_now()+WORKER_RECEIPT_RETENTION_SECS+100).unwrap();
        let store_root=runtime.store.root.clone();let identity=runtime.identity.clone();drop(runtime);
        let reopened=WorkerStore::open(&store_root,identity).unwrap();
        assert!(reopened.reserve_view_plan(&command,&request,&seed).is_err());
        let closed=fs::read(&receipt_path).unwrap();
        reopened.conserve_unpublished_view(&seed.binding).unwrap();
        assert_eq!(fs::read(&receipt_path).unwrap(),closed);
    }

    #[test]
    fn spec221_c19_unknown_planning_process_never_releases_reservation_or_claims_conservation() {
        let fixture=Fixture::new();
        let (runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=c19_plan(&intent);
        runtime.store.reserve_view_plan(&command,&request,&seed).unwrap();
        runtime.store.mark_view_source_attempt_binding(&seed.binding).unwrap();
        let before=fs::read(runtime.store.workspace_path(&command)).unwrap();
        let budget=runtime.store.inventory().unwrap().2;
        assert!(matches!(runtime.store.conserve_unpublished_view(&seed.binding),Err(WorkerStoreError::Unknown)));
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),before);
        assert_eq!(runtime.store.inventory().unwrap().2,budget);
    }

    #[test]
    fn spec221_c19_failed_plan_native_conservation_keeps_cause_and_never_exports_again() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=c19_plan(&intent);
        assert!(std::process::Command::new("git").arg("-C").arg(&intent.source)
            .args(["config","remote.origin.url","fixture:changed"]).status().unwrap().success());
        let read=runtime.begin_planned_view_source(&command,&request,seed.clone()).unwrap();
        let deadline=Instant::now()+Duration::from_secs(5);
        loop {
            match runtime.poll_view_source(&read) {
                Ok(None)=>{assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));}
                Err((WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::SourceIdentityChanged)))=>break,
                other=>panic!("unexpected failed plan: {other:?}"),
            }
        }
        assert!(runtime.view_sources.is_empty() && !intent.destination().unwrap().exists());
        assert!(matches!(runtime.conserve_commit_view(&command,&seed.binding.prepare_digest).unwrap(),
            WorkerResponseOperation::CommitViewConserved{..}));
        let receipt=runtime.store.view_receipt_for_binding(&seed.binding).unwrap();
        let view=receipt.commit_view.as_ref().unwrap();
        assert!(view.closed && view.source_stopped && view.source_marker.is_some());
        assert_eq!(view.source_failure,Some(crate::workspace_git::ExportFailure::SourceIdentityChanged));
        assert!(runtime.begin_planned_view_source(&command,&request,seed).is_err());
    }

    #[test]
    fn spec221_c19_snapshot_commit_rechecks_cancel_and_keeps_durable_marker_and_data() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let read=runtime.begin_prepared_view(&command,&request,None,c19_plan(&intent)).unwrap();
        let deadline=Instant::now()+Duration::from_secs(5);
        while runtime.view_reports.is_empty() {
            assert!(runtime.poll_prepared_view(&read).unwrap().is_none());
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));
        }
        let key=marker_key(&command);
        while !runtime.view_reports[&key].reader.is_finished() {
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));
        }
        let (audited_intent,audited)=runtime.view_reports.remove(&key).unwrap().reader.join().unwrap().unwrap();
        let receipt=runtime.store.checked_view_receipt(&command,&audited_intent).unwrap();
        assert!(receipt.record.is_none()); // premier CAS, pas un rejeu déjà publié
        let marker=receipt.commit_view.as_ref().unwrap().source_marker.clone();assert!(marker.is_some());
        let before=fs::read(runtime.store.workspace_path(&command)).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command).unwrap());
        assert!(matches!(runtime.store.publish_view_snapshot(&command,&audited_intent,audited,&runtime.cancelled),
            Err(WorkerStoreError::InvalidTransition)));
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),before);
        let receipt=runtime.store.checked_view_receipt(&command,&audited_intent).unwrap();
        assert_eq!(receipt.commit_view.as_ref().unwrap().source_marker,marker);
        assert!(receipt.record.is_none() && runtime.prepared.is_empty());
        assert_eq!(fs::read(intent.destination().unwrap().join("tracked")).unwrap(),b"committed");
    }

    #[test]
    fn spec221_c19_unknown_manifests_cannot_spawn_parallel_exports_under_existing_store_budget() {
        let fixture=Fixture::new();
        let (runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=c19_plan(&intent);
        // Saturer le même budget partagé sans dépendre de son ancien plafond16Mio.
        let filler=runtime.store.root.join("retained-history");
        write_private_file_atomic(&filler,b"").unwrap();
        fs::OpenOptions::new().write(true).open(&filler).unwrap()
            .set_len((MAX_WORKER_STORE_BYTES as u64).saturating_sub(seed.reserved_bytes()*2)+1).unwrap();
        runtime.store.reserve_view_plan(&command,&request,&seed).unwrap();
        let mut other=command.clone();other.command_id=uuid::Uuid::new_v4().to_string();other.task_id=uuid::Uuid::new_v4().to_string();
        let binding=commit_view_preparation_binding(&runtime.identity,&other,&request).unwrap();
        let other_seed=crate::workspace_git::ViewPlan::new(binding,&intent.source,&intent.data_root,
            intent.base_commit(),&intent.source,"fixture:c19").unwrap();
        assert!(matches!(runtime.store.reserve_view_plan(&other,&request,&other_seed),Err(WorkerStoreError::Quota)));
        assert!(runtime.view_sources.is_empty());
        assert_eq!(runtime.store.view_receipts(None).unwrap().len(),1);
    }

    #[test]
    fn spec221_c19_native_queue_waits_for_unknown_writer_without_renewing_seed_or_exporting() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let mut other=intent.clone();other.binding.command.command_id=uuid::Uuid::new_v4().to_string();
        other.binding.command.task_id=uuid::Uuid::new_v4().to_string();
        other.binding.prepare_digest=json_digest(&(&other.binding.command,&request)).unwrap();
        runtime.store.reserve_view_intent(&other.binding.command,&request,&other).unwrap();
        runtime.store.mark_view_source_attempt(&other.binding.command,&other).unwrap();
        let seed=runtime.native_view_plan(&command,&request).unwrap();
        let digest=seed.proof_digest().unwrap();
        let read=runtime.begin_prepared_view(&command,&request,None,seed).unwrap();
        for _ in 0..3 {
            assert!(runtime.poll_prepared_view(&read).unwrap().is_none());
            assert!(runtime.view_sources.is_empty());
            let state=runtime.store.view_receipt_for_binding(&read.source.binding).unwrap().commit_view.unwrap();
            assert!(!state.source_attempted && state.source_marker.is_none() && !state.source_stopped);
        }
        assert_eq!(runtime.native_view_plan(&command,&request).unwrap().proof_digest().unwrap(),digest);
        runtime.cancelled.lock().unwrap().insert(json_digest(&command).unwrap());
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),
            (WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::Cancelled)));
        assert!(runtime.prepared.is_empty() && runtime.view_sources.is_empty());
    }

    #[test]
    fn spec221_c19_adoption_releases_unknown_manifest_budget_and_allows_next_review() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=c19_plan(&intent);
        let read=runtime.begin_planned_view_source(&command,&request,seed.clone()).unwrap();
        let reserved=runtime.store.inventory().unwrap().2;
        let deadline=Instant::now()+Duration::from_secs(5);
        let first=loop {
            if let Some(record)=runtime.poll_view_source(&read).unwrap(){break record;}
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));
        };
        let after=runtime.store.inventory().unwrap().2;
        assert!(reserved-after>7*1024*1024);
        let receipt=runtime.store.checked_view_receipt(&command,&intent).unwrap();
        assert_eq!(receipt.commit_view.as_ref().unwrap().reserved_bytes,seed.adopted_reserved_bytes(&intent).unwrap());
        let mut next=command.clone();next.task_id="next-review".into();next.command_id="next-review".into();
        let binding=crate::workspace_git::ViewBinding{worker:runtime.identity.clone(),command:next.clone(),
            prepare_digest:json_digest(&(&next,&request)).unwrap()};
        let second=crate::workspace_git::ViewPlan::new(binding,&intent.source,&intent.data_root,
            intent.base_commit(),&intent.source,"fixture:c19").unwrap();
        runtime.store.reserve_view_plan(&next,&request,&second).unwrap();
        assert!(runtime.store.inventory().unwrap().2<=MAX_WORKER_STORE_BYTES as u64);
        assert_eq!(fs::read(Path::new(&first.path).join("tracked")).unwrap(),b"committed");
        assert!(runtime.view_sources.is_empty()); // aucun deuxième modèle ni helper lancé
    }

    fn c19_control_request(runtime:&WorkerRuntime,prepared:&PreparedRuntime)->WorkerRequest {
        let mut command=prepared.command.clone();command.instance_id="view-active-instance".into();command.generation=1;
        WorkerRequest{version:REMOTE_WORKER_VERSION,worker:runtime.identity.clone(),
            connection_epoch:"view-control-epoch".into(),correlation_id:"view-control-only".into(),command,
            request:WorkerRequestOperation::StartDetailed{start:WorkerStart{
                prepared_token:prepared.prepared.token.clone(),child_id:"view-child".into(),mission_id:"view-mission".into(),
                forward_child_requests:false,placement:prepared.prepared.placement.clone(),
                definition:prepared.prepared.definition.clone(),child_policy:prepared.prepared.child_policy.clone(),
                effective_posture:prepared.prepared.effective_posture,deadline_at:worker_now()+60}}}
    }

    #[test]
    fn spec221_c19_control_audit_keeps_cancel_responsive_and_never_starts_after_cancel() {
        let fixture=Fixture::new();
        let (mut runtime,_,prepared)=c19_prepared_view_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_CONTROL_HOLD".into(),"1".into());
        let request=c19_control_request(&runtime,&prepared);
        runtime.prepared.insert(request.command.command_id.clone(),prepared);
        let before=runtime.store.inventory().unwrap().2;
        assert!(runtime.defer_view_control(&request).unwrap());
        assert!(runtime.poll_view_controls("view-control-epoch",&AtomicBool::new(false)).is_empty());
        assert!(runtime.defer_view_control(&request).unwrap());
        assert_eq!(runtime.view_controls.len(),1);
        runtime.cancelled.lock().unwrap().insert(json_digest(&request.command).unwrap());
        let observed=Instant::now();
        let responses=runtime.poll_view_controls("view-control-epoch",&AtomicBool::new(false));
        assert!(observed.elapsed()<Duration::from_millis(250));
        assert_eq!(responses.len(),1);
        assert_eq!(responses[0].correlation_id,request.correlation_id);
        assert!(matches!(responses[0].response,WorkerResponseOperation::StartRejected{
            reason:WorkerRefusal::UnknownExecution,cause:WorkerStartCapacityCause::Unknown}));
        responses[0].validate().unwrap();
        assert_eq!(runtime.view_controls.len(),1,"lecteur retenu jusqu'à sa fin");
        fs::write(runtime.config.state_root.join("allow-view-control"),b"").unwrap();
        let until=Instant::now()+Duration::from_secs(4);
        while !runtime.view_controls.is_empty() {
            assert!(runtime.poll_view_controls("view-control-epoch",&AtomicBool::new(false)).is_empty());
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
        }
        assert!(matches!(runtime.store.command(&request.command),Err(WorkerStoreError::Unknown)));
        assert!(runtime.held.is_empty() && runtime.running.is_empty());
        assert_eq!(runtime.store.inventory().unwrap().2,before);
    }

    #[test]
    fn spec221_c19_control_audit_rechecks_revocation_before_any_start_effect() {
        let fixture=Fixture::new();
        let (mut runtime,_,prepared)=c19_prepared_view_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_CONTROL_HOLD".into(),"1".into());
        let request=c19_control_request(&runtime,&prepared);
        runtime.prepared.insert(request.command.command_id.clone(),prepared);
        assert!(runtime.defer_view_control(&request).unwrap());
        let mut registry:serde_json::Value=serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap()).unwrap();
        registry["machines"]["cartae"]["enabled"]=serde_json::json!(false);
        write_json(&runtime.config.registry_path,&registry).unwrap();
        fs::write(runtime.config.state_root.join("allow-view-control"),b"").unwrap();
        let until=Instant::now()+Duration::from_secs(4);
        let responses=loop {
            let responses=runtime.poll_view_controls("view-control-epoch",&AtomicBool::new(false));
            if !responses.is_empty(){break responses;}
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(responses.len(),1);responses[0].validate().unwrap();
        assert!(matches!(responses[0].response,WorkerResponseOperation::StartRejected{
            reason:WorkerRefusal::Unauthorized|WorkerRefusal::PlacementMismatch,
            cause:WorkerStartCapacityCause::Unknown}));
        assert!(runtime.view_controls.is_empty() && runtime.held.is_empty() && runtime.running.is_empty());
        assert!(matches!(runtime.store.command(&request.command),Err(WorkerStoreError::Unknown)));
    }

    #[test]
    fn spec221_c19_consumed_audit_refuses_foreign_anchor_and_still_rechecks_current_permissions() {
        let fixture=Fixture::new();
        let (runtime,_,prepared)=c19_prepared_view_fixture(&fixture);
        let record:crate::delegation::WorkspaceRecord=serde_json::from_value(prepared.prepared.workspace.clone().unwrap()).unwrap();
        let report=crate::workspace_git::report(&record).unwrap();
        let check=|report:&crate::delegation::WorkspaceRecord|runtime.read_view().recheck_at_with_view_report(
            &prepared,true,None,&read::ReadTrace::default(),&mut None,Some(report));
        assert!(check(&report).is_ok());
        let mut foreign=report.clone();foreign.path="/another/view".into();
        assert!(matches!(check(&foreign),Err(WorkerRefusal::CommandMismatch)));
        let mut registry:serde_json::Value=serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap()).unwrap();
        registry["machines"]["cartae"]["enabled"]=serde_json::json!(false);
        write_json(&runtime.config.registry_path,&registry).unwrap();
        assert!(check(&report).is_err());
    }

    #[test]
    fn spec221_c19_probe_reads_requested_a_without_refreshing_dirty_descendant_b() {
        let fixture=Fixture::new();
        let (runtime,_,mut request,_)=c19_preparation_fixture(&fixture);
        let source=Path::new(&request.remote_root);
        let git=|args:&[&str]| {
            let output=std::process::Command::new("git").arg("-C").arg(source)
                .args(["-c","user.name=Fixture","-c","user.email=fixture@example.invalid","-c","commit.gpgSign=false"])
                .args(args).output().unwrap();
            assert!(output.status.success());output.stdout
        };
        fs::write(source.join("tracked"),b"B").unwrap();git(&["add","tracked"]);git(&["commit","-qm","descendant B"]);
        fs::write(source.join("tracked"),b"dirty B").unwrap();
        let head=git(&["rev-parse","HEAD"]);let index=fs::read(source.join(".git/index")).unwrap();
        let registry=runtime.registry().unwrap();
        let mapping=runtime.mapping_authority(&request,&registry).unwrap();
        assert_eq!(runtime.probe_workspace_preconditions(&request,&mapping,source),Ok(()));
        assert_eq!(git(&["rev-parse","HEAD"]),head);
        assert_eq!(fs::read(source.join(".git/index")).unwrap(),index);
        assert_eq!(fs::read(source.join("tracked")).unwrap(),b"dirty B");
        request.base_commit=Some("f".repeat(40));
        assert!(runtime.probe_workspace_preconditions(&request,&mapping,source).is_err());
        request.posture=Some(SpawnPosture::Development);
        assert_eq!(runtime.probe_workspace_preconditions(&request,&mapping,source),Err(CandidateProbeRefusal::InvalidRequest));
        assert!(runtime.store.view_receipts(None).unwrap().is_empty());
        assert!(runtime.view_sources.is_empty() && runtime.prepared.is_empty());
    }

    #[test]
    fn spec221_c19_deferred_prepare_keeps_one_owner_deadline_and_current_epoch_correlations() {
        let fixture=Fixture::new();
        let (mut runtime,command,prepare,_)=c19_preparation_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_REPORT_HOLD".into(),"1".into());
        let mut request=WorkerRequest{version:REMOTE_WORKER_VERSION,worker:runtime.identity.clone(),
            connection_epoch:"view-epoch-1".into(),correlation_id:"view-correlation-0".into(),
            command:command.clone(),request:WorkerRequestOperation::Prepare{prepare:prepare.clone()}};
        assert!(runtime.defer_view_preparation(&request).unwrap());
        let key=marker_key(&command);
        let expires=runtime.view_preparations[&key].read.expires;
        let until=Instant::now()+Duration::from_secs(6);
        while runtime.view_reports.is_empty() {
            assert!(runtime.poll_view_preparations("view-epoch-1").is_empty());
            assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        }
        let marker=runtime.store.view_receipt_for_binding(
            &runtime.view_preparations[&key].read.source.binding).unwrap().commit_view.unwrap().source_marker;
        for index in 1..8 {
            request.correlation_id=format!("view-correlation-{index}");
            assert!(runtime.defer_view_preparation(&request).unwrap());
        }
        request.correlation_id="view-correlation-overflow".into();
        assert_eq!(runtime.defer_view_preparation(&request).unwrap_err(),
            (WorkerRefusal::ResourceLimit,Some(crate::workspace_git::ExportFailure::OperationCapacity)));
        assert_eq!(runtime.view_preparations.len(),1);
        assert_eq!(runtime.view_preparations[&key].read.expires,expires);
        let mut changed=request.clone();
        if let WorkerRequestOperation::Prepare{prepare}=&mut changed.request {prepare.model="different".into();}
        assert_eq!(runtime.defer_view_preparation(&changed).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        runtime.abandon_held();
        assert!(runtime.poll_view_preparations("view-epoch-2").is_empty());
        request.connection_epoch="view-epoch-2".into();request.correlation_id="new-link-only".into();
        assert!(runtime.defer_view_preparation(&request).unwrap());
        assert_eq!(runtime.view_preparations[&key].read.expires,expires);
        assert_eq!(runtime.store.view_receipt_for_binding(
            &runtime.view_preparations[&key].read.source.binding).unwrap().commit_view.unwrap().source_marker,marker);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-report")).unwrap();
        let responses=loop {
            let responses=runtime.poll_view_preparations("view-epoch-2");
            if !responses.is_empty(){break responses;}
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(responses.len(),1);
        assert_eq!(responses[0].correlation_id,"new-link-only");
        assert_eq!(responses[0].connection_epoch,"view-epoch-2");
        assert!(matches!(responses[0].response,WorkerResponseOperation::Prepared{..}));
        responses[0].validate().unwrap();
        assert!(runtime.view_preparations.is_empty() && runtime.running.is_empty() && runtime.held.is_empty());
    }

    #[test]
    fn spec221_c19_deferred_cancel_returns_correlated_failure_without_publishing_or_blocking_runtime() {
        let fixture=Fixture::new();
        let (mut runtime,command,prepare,_)=c19_preparation_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_REPORT_HOLD".into(),"1".into());
        let request=WorkerRequest{version:REMOTE_WORKER_VERSION,worker:runtime.identity.clone(),
            connection_epoch:"view-epoch".into(),correlation_id:"cancel-view-only".into(),
            command:command.clone(),request:WorkerRequestOperation::PrepareDiagnosed{
                prepare:Box::new(WorkerRequestOperation::Prepare{prepare})}};
        assert!(runtime.defer_view_preparation(&request).unwrap());
        let until=Instant::now()+Duration::from_secs(6);
        while runtime.view_reports.is_empty() {
            assert!(runtime.poll_view_preparations("view-epoch").is_empty());
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
        }
        runtime.cancelled.lock().unwrap().insert(json_digest(&command).unwrap());
        // La boucle reste disponible même si le lecteur de fichiers n'a pas fini.
        let before=Instant::now();
        let responses=runtime.poll_view_preparations("view-epoch");
        assert!(before.elapsed()<Duration::from_millis(250));
        assert_eq!(responses.len(),1);
        assert_eq!(responses[0].correlation_id,request.correlation_id);
        assert!(matches!(responses[0].response,WorkerResponseOperation::CommitViewRejected{
            reason:WorkerRefusal::WorkspacePrepareFailed,failure:crate::workspace_git::ExportFailure::Cancelled}));
        responses[0].validate().unwrap();
        assert!(runtime.prepared.is_empty() && runtime.view_reports.len()==1);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-report")).unwrap();
        while !runtime.view_reports.is_empty() {
            runtime.collect_view_sources();
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
        }
        assert!(runtime.prepared.is_empty());
    }

    #[test]
    fn spec221_continuable_worker_status_is_negotiated_and_never_starts_a_command() {
        let legacy=MasterWorkerFeatures::default();
        assert!(!legacy.admits(&[WorkerCapability::ExecutionStatusV1]));
        let modern=MasterWorkerFeatures {execution_status:true,..MasterWorkerFeatures::default()};
        assert!(modern.admits(&[WorkerCapability::ExecutionStatusV1]));
        {
            use std::io::Write;
            use std::os::unix::process::CommandExt;
            let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);
            reserve(&runtime.store);
            let mut child=std::process::Command::new("sh").args(["-c","read line"])
                .stdin(std::process::Stdio::piped()).process_group(0).spawn().unwrap();
            let birth=crate::managed_process::process_birth(child.id()).unwrap();
            let marker=crate::managed_process::ManagedMarker {pgid:child.id(),birth,
                instance_id:command().instance_id,command_id:command().command_id,generation:command().generation};
            runtime.store.mark_command(&command(),&marker,101).unwrap();
            let permit=WorkerReleasePermit {permit_id:"read-status221".into(),
                command_digest:"d".repeat(64),permission_revision:1,mapping_revision:1,
                provisioning_revision:1,issued_at:101,expires_at:106};
            runtime.store.authorize_release(&command(),&permit,"epoch221",101).unwrap();
            runtime.store.set_phase(&command(),WorkerCommandPhase::Released,102).unwrap();
            runtime.store.native_status(&command(),false).unwrap();
            let before=fs::read(runtime.store.command_path(&command())).unwrap();
            let WorkerResponseOperation::Observation{observation}=runtime.execution_status(&command()).unwrap()
                else {panic!("observation")};
            assert_eq!(observation.phase,WorkerCommandPhase::Released);assert!(observation.native_ready);
            assert_eq!(before,fs::read(runtime.store.command_path(&command())).unwrap());
            assert!(crate::managed_process::process_matches_live(child.id(),birth).unwrap());
            // Fixture de reçu dont la naissance est devenue étrangère.
            let path=runtime.store.command_path(&command());
            let mut altered:serde_json::Value=serde_json::from_slice(&before).unwrap();
            altered["marker"]["birth"]=serde_json::json!(birth.saturating_add(1));
            fs::write(&path,serde_json::to_vec(&altered).unwrap()).unwrap();
            let wrong_birth=fs::read(runtime.store.command_path(&command())).unwrap();
            let WorkerResponseOperation::Observation{observation}=runtime.execution_status(&command()).unwrap()
                else {panic!("observation")};
            assert_eq!(observation.phase,WorkerCommandPhase::Unknown);assert!(!observation.native_ready);
            assert_eq!(wrong_birth,fs::read(runtime.store.command_path(&command())).unwrap());
            fs::write(&path,&before).unwrap();
            let before_exit=fs::read(runtime.store.command_path(&command())).unwrap();
            // Libération normale de notre fixture par stdin, aucun signal.
            child.stdin.take().unwrap().write_all(b"done\n").unwrap();
            let until=Instant::now()+Duration::from_secs(3);
            while crate::managed_process::process_matches_live(child.id(),birth).unwrap_or(false) {
                assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
            }
            let WorkerResponseOperation::Observation{observation}=runtime.execution_status(&command()).unwrap()
                else {panic!("observation")};
            assert_eq!(observation.phase,WorkerCommandPhase::Unknown);assert!(!observation.native_ready);
            assert_eq!(before_exit,fs::read(runtime.store.command_path(&command())).unwrap());
            child.wait().unwrap();
        }
        for phase in [WorkerCommandPhase::Terminal,WorkerCommandPhase::Refused] {
            let fixture=Fixture::new();let mut runtime=runtime_fixture(&fixture);
            reserve(&runtime.store);
            runtime.store.set_phase(&command(),phase,worker_now()).unwrap();
            let request=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"status221".into(),
                connection_epoch:"epoch221".into(),worker:worker(),command:command(),
                request:WorkerRequestOperation::ExecutionStatus {}};
            let response=runtime.handle(&request,"epoch221",&AtomicBool::new(false)).unwrap();
            let WorkerResponseOperation::Observation {observation}=response else {panic!("observation")};
            assert_eq!(observation.phase,phase);
            assert!(observation.workspace.is_none() && observation.permission_digest.is_none());
            assert!(runtime.running.is_empty() && runtime.held.is_empty() && runtime.prepared.is_empty());
            assert_eq!(runtime.store.commands().unwrap().len(),1);
            assert_eq!(runtime.handle(&request,"other-epoch",&AtomicBool::new(false)).unwrap_err(),WorkerRefusal::StaleConnection);
        }
    }

    #[test]
    fn spec221_c19_negotiation_keeps_reader_and_creator_distinct() {
        let reader=MasterWorkerFeatures{workspace_view:true,..MasterWorkerFeatures::default()};
        assert_eq!(reader.capabilities(),vec![WorkerCapability::WorkspaceViewV1]);
        assert!(!reader.admits(&[WorkerCapability::WorkspaceViewV1,WorkerCapability::WorkspaceViewCreateV1]));
        assert!(reader.admits(&[WorkerCapability::WorkspaceViewV1]));
        let creator=MasterWorkerFeatures{workspace_view:true,workspace_view_create:true,..MasterWorkerFeatures::default()};
        assert!(creator.supports(WorkerCapability::WorkspaceViewCreateV1));
        assert!(!creator.admits(&[WorkerCapability::WorkspaceViewCreateV1]));
        assert!(creator.admits(&[WorkerCapability::WorkspaceViewV1,WorkerCapability::WorkspaceViewCreateV1]));
    }

    #[test]
    fn spec221_c19_missing_reservation_closes_only_after_exact_fence_and_survives_collection() {
        let fixture=Fixture::new();
        let (runtime,prepare,request,intent)=c19_preparation_fixture(&fixture);
        let binding=intent.binding.clone();
        let before=runtime.store.inventory().unwrap();
        assert_eq!(runtime.conserve_commit_view(&prepare,&binding.prepare_digest).unwrap_err(),WorkerRefusal::UnknownExecution);
        assert_eq!(runtime.store.inventory().unwrap().2,before.2);
        let mut command=prepare.clone();command.instance_id="never-started".into();command.generation=1;
        runtime.store.fence_unstarted(&command,&placement(),worker_now()+60,worker_now()).unwrap();
        assert!(matches!(runtime.conserve_commit_view(&prepare,&binding.prepare_digest).unwrap(),
            WorkerResponseOperation::CommitViewConserved{..}));
        let receipt=runtime.store.view_receipt_for_binding(&binding).unwrap();
        let view=receipt.commit_view.unwrap();
        assert!(view.closed && view.source_stopped && !view.source_attempted && view.source_marker.is_none());
        assert!(receipt.record.is_none() && !intent.destination().unwrap().exists());
        runtime.store.collect(worker_now()+WORKER_RECEIPT_RETENTION_SECS+100).unwrap();
        assert!(runtime.store.reserve_view_intent(&prepare,&request,&intent).is_err());
        runtime.conserve_commit_view(&prepare,&binding.prepare_digest).unwrap();
        assert!(runtime.store.view_receipt_for_binding(&binding).unwrap().commit_view.unwrap().closed);
    }

    #[test]
    fn spec221_c19_missing_reservation_with_even_empty_orphan_proof_remains_unknown() {
        let fixture=Fixture::new();
        let (runtime,prepare,_,intent)=c19_preparation_fixture(&fixture);
        let mut command=prepare.clone();command.instance_id="never-started".into();command.generation=1;
        runtime.store.fence_unstarted(&command,&placement(),worker_now()+60,worker_now()).unwrap();
        private_dir(&runtime.store.view_root()).unwrap();
        let path=runtime.store.view_root().join(format!("{}.lock",intent.binding.key().unwrap()));
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(),0);
        let before=runtime.store.inventory().unwrap().2;
        assert_eq!(runtime.conserve_commit_view(&prepare,&intent.binding.prepare_digest).unwrap_err(),WorkerRefusal::UnknownExecution);
        assert!(proof198::read_workspace_receipt(&runtime.store.root,&runtime.identity,&prepare).unwrap().is_none());
        assert!(path.exists());
        assert_eq!(runtime.store.inventory().unwrap().2,before);
    }

    #[test]
    fn spec221_c19_prepare_refuses_wrong_mode_or_binding_before_reserving_or_spawning() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let before=runtime.store.inventory().unwrap();
        let mut changed=request.clone();changed.posture=Some(SpawnPosture::Development);
        assert!(matches!(runtime.begin_prepared_view(&command,&changed,None,c19_plan(&intent)),
            Err((WorkerRefusal::InvalidEnvelope,None))));
        let mut bad=c19_plan(&intent);bad.binding.prepare_digest="f".repeat(64);
        assert!(matches!(runtime.begin_prepared_view(&command,&request,None,bad),
            Err((WorkerRefusal::CommandMismatch,None))));
        assert!(!intent.destination().unwrap().exists());
        assert!(runtime.view_sources.is_empty() && runtime.prepared.is_empty());
        let after=runtime.store.inventory().unwrap();assert_eq!((before.2,before.3),(after.2,after.3));
    }

    #[test]
    fn spec221_c19_prepare_rechecks_definition_after_export_and_keeps_view_on_refusal() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let read=runtime.begin_prepared_view(&command,&request,None,c19_plan(&intent)).unwrap();
        let mut registry:serde_json::Value=serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap()).unwrap();
        registry["agents"]["codex"]["command"]=serde_json::json!("/bin/false");
        write_json(&runtime.config.registry_path,&registry).unwrap();
        let deadline=Instant::now()+Duration::from_secs(10);
        loop {
            match runtime.poll_prepared_view(&read) {
                Ok(None)=>{assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
                Err((WorkerRefusal::DefinitionChanged|WorkerRefusal::PermissionsChanged,None))=>break,
                other=>panic!("unexpected preparation result: {other:?}"),
            }
        }
        assert!(runtime.prepared.is_empty() && runtime.running.is_empty() && runtime.held.is_empty());
        assert!(intent.destination().unwrap().is_dir());
        assert_eq!(fs::read(intent.destination().unwrap().join("tracked")).unwrap(),b"committed");
        assert!(proof198::read_workspace_receipt(&runtime.store.root,&runtime.identity,&command).unwrap().unwrap().record.is_some());
    }

    #[test]
    fn spec221_c19_prepare_expired_context_never_renews_a_token_or_loses_owned_helper() {
        let fixture=Fixture::new();
        let (mut runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let mut read=runtime.begin_prepared_view(&command,&request,None,c19_plan(&intent)).unwrap();
        let private=runtime.store.view_root();
        let ready=Instant::now()+Duration::from_secs(3);
        while !private.join("stall-ready").exists() {
            assert!(Instant::now()<ready);std::thread::sleep(Duration::from_millis(10));
        }
        read.expires=Instant::now()-Duration::from_secs(1);
        assert_eq!(runtime.poll_prepared_view(&read).unwrap_err(),
            (WorkerRefusal::PrepareExpired,Some(crate::workspace_git::ExportFailure::StopUnconfirmed)));
        assert!(!runtime.view_sources.is_empty());
        let receipt=runtime.store.view_receipt_for_binding(&read.source.binding).unwrap();
        assert!(!receipt.commit_view.as_ref().unwrap().source_stopped);
        fs::write(private.join("allow-exit"),b"").unwrap();
        assert!(runtime.prepared.is_empty() && runtime.running.is_empty() && runtime.held.is_empty());
        let deadline=Instant::now()+Duration::from_secs(10);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();
            assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(runtime.prepared.is_empty());
        let receipt=runtime.store.view_receipt_for_binding(&read.source.binding).unwrap();
        assert_eq!(receipt.commit_view.as_ref().unwrap().source_failure,
            Some(crate::workspace_git::ExportFailure::PreparationExpired));
    }

    #[test]
    fn spec221_c19_retained_view_rechecks_policy_and_exact_base_without_shared_head() {
        let fixture=Fixture::new();
        let (runtime,intent,mut prepared)=c19_prepared_view_fixture(&fixture);
        let request=prepared.request.clone();let prep=prepared.command.clone();
        let record=runtime.store.checked_view_receipt(&prep,&intent).unwrap().record.unwrap();
        let source=Path::new(&request.remote_root);
        let cwd=PathBuf::from(&prepared.prepared.placement.execution_cwd);
        let private_before=fs::read(runtime.store.workspace_path(&prep)).unwrap();
        // Le Shared peut avancer et être sale ; la vue doit rester sur A.
        fs::write(source.join("tracked"),b"descendant B").unwrap();
        for args in [vec!["add","tracked"],vec!["commit","-qm","descendant"]] {
            assert!(std::process::Command::new("git").arg("-C").arg(source)
                .args(["-c","user.name=Fixture","-c","user.email=fixture@example.invalid","-c","commit.gpgSign=false"])
                .args(args).status().unwrap().success());
        }
        fs::write(source.join("tracked"),b"dirty B").unwrap();
        let source_index=fs::read(source.join(".git/index")).unwrap();
        let source_head=fs::read(source.join(".git/refs/heads").join(
            fs::read_to_string(source.join(".git/HEAD")).unwrap().trim().strip_prefix("ref: refs/heads/").unwrap())).unwrap();
        for before_release in [false,true] {
            let mut report=None;
            let trace=read::ReadTrace::default();
            let outcome=runtime.read_view().recheck_at_with_workspace(&prepared,before_release,None,&trace,&mut report);
            assert!(outcome.is_ok(),"recheck {before_release}, {:?}",outcome.err());
            let report=report.unwrap().unwrap();
            assert_eq!(report.base_commit,request.base_commit.as_ref().unwrap().as_str());
            assert_eq!(report.dirty,Some(false));
            assert!(report.head_commit.is_none());
        }
        assert_eq!(fs::read(cwd.join("tracked")).unwrap(),b"committed");
        assert_eq!(fs::read(source.join("tracked")).unwrap(),b"dirty B");
        assert_eq!(fs::read(source.join(".git/index")).unwrap(),source_index);
        assert_eq!(fs::read(source.join(".git/refs/heads").join(
            fs::read_to_string(source.join(".git/HEAD")).unwrap().trim().strip_prefix("ref: refs/heads/").unwrap())).unwrap(),source_head);
        assert_eq!(fs::read(runtime.store.workspace_path(&prep)).unwrap(),private_before);
        assert!(runtime.held.is_empty() && runtime.running.is_empty());

        // Même reçu mais export encore non soldé : aucun audit/réattribution.
        let mut receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        receipt.commit_view.as_mut().unwrap().source_stopped=false;
        runtime.store.persist(&runtime.store.workspace_path(&prep),&receipt).unwrap();
        let mut report=None;
        assert!(matches!(runtime.read_view().recheck_at_with_workspace(&prepared,true,None,
            &read::ReadTrace::default(),&mut report),Err(WorkerRefusal::UnknownExecution)));
        assert!(report.is_none());
        fs::write(runtime.store.workspace_path(&prep),&private_before).unwrap();

        // Substitution privée cohérente dans les deux messages : le reçu seul
        // ne permet pas de lire un autre store sous le même UID.
        let mut receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        let mut substituted=record.clone();
        substituted.commit_view.as_mut().unwrap().proof_root=fixture.0.join("foreign-store").to_string_lossy().into();
        receipt.record=Some(substituted.clone());
        runtime.store.persist(&runtime.store.workspace_path(&prep),&receipt).unwrap();
        prepared.prepared.workspace=Some(serde_json::to_value(&substituted).unwrap());
        assert!(matches!(runtime.read_view().recheck_at_with_workspace(&prepared,true,None,
            &read::ReadTrace::default(),&mut report),Err(WorkerRefusal::CommandMismatch)));
        assert!(report.is_none());
        fs::write(runtime.store.workspace_path(&prep),&private_before).unwrap();
        prepared.prepared.workspace=Some(serde_json::to_value(&record).unwrap());
        prepared.prepared.effective_posture=SpawnPosture::Development;
        assert!(matches!(runtime.read_view().recheck_at_with_workspace(&prepared,true,None,
            &read::ReadTrace::default(),&mut report),Err(WorkerRefusal::PlacementMismatch)));
        prepared.prepared.effective_posture=SpawnPosture::Discovery;

        fs::set_permissions(cwd.join("tracked"),fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(cwd.join("tracked"),b"divergent view").unwrap();
        assert!(matches!(runtime.read_view().recheck_at_with_workspace(&prepared,true,None,
            &read::ReadTrace::default(),&mut report),Err(WorkerRefusal::PlacementMismatch)));
        assert!(matches!(report,Some(Err(WorkerRefusal::PlacementMismatch))));
        assert_eq!(fs::read(cwd.join("tracked")).unwrap(),b"divergent view");
    }


    fn c19_active_file_fixture(fixture:&Fixture)
        ->(WorkerRuntime,crate::workspace_git::ViewIntent,WorkerResultIdentity) {
        let (runtime,intent,prepared)=c19_prepared_view_fixture(fixture);
        let mut command=prepared.command.clone();command.instance_id="file-reader-instance".into();command.generation=1;
        let now=worker_now();
        runtime.store.reserve_command(&command,&"d".repeat(64),&prepared.prepared.placement,now+300,now).unwrap();
        let mut receipt=runtime.store.read_command(&command).unwrap().unwrap();
        // Simuler l'état natif de la mission pour tester son autorisation.
        // Ce n'est ni une admission ni une qualification fournisseur réelle.
        receipt.phase=WorkerCommandPhase::Released;receipt.native_ready=true;
        receipt.preparation=Some((prepared.request,prepared.prepared));
        receipt.child_id=Some("file-reader-fixture".into());
        runtime.store.persist(&runtime.store.command_path(&command),&receipt).unwrap();
        let execution=WorkerResultIdentity {command,mission_id:"file-mission".into(),round:1,
            execution_id:"file-execution".into(),execution_generation:1};
        runtime.store.reserve_result_slot(&WorkerResultSlot {identity:execution.clone(),admission_digest:"c".repeat(64)},now).unwrap();
        (runtime,intent,execution)
    }

    #[test]
    fn spec221_c19_file_request_requires_exact_live_mission_before_receipt_effects() {
        let fixture=Fixture::new();let (runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let prep=&intent.binding.command;
        let before=fs::read(runtime.store.workspace_path(prep)).unwrap();
        let mut wrong=execution.clone();wrong.mission_id="other-mission".into();
        assert!(runtime.store.begin_materialization(&wrong,"large.bin").is_err());
        wrong=execution.clone();wrong.command.instance_id="other-instance".into();
        assert!(runtime.store.begin_materialization(&wrong,"large.bin").is_err());
        for path in ["../elsewhere","not-in-commit"] {
            assert!(runtime.store.begin_materialization(&execution,path).is_err());
        }
        assert_eq!(fs::read(runtime.store.workspace_path(prep)).unwrap(),before);
        let (_,request)=runtime.store.begin_materialization(&execution,"large.bin").unwrap();
        let pending=fs::read(runtime.store.workspace_path(prep)).unwrap();
        let (_,replay)=runtime.store.begin_materialization(&execution,"large.bin").unwrap();
        assert_eq!(request,replay);assert_eq!(fs::read(runtime.store.workspace_path(prep)).unwrap(),pending);
        assert!(matches!(runtime.store.begin_materialization(&execution,"tracked"),
            Err(WorkerStoreError::Locked)));
        let snapshot=runtime.store.checked_view_receipt(prep,&intent).unwrap();
        assert!(runtime.store.authorize_materialization(&execution,&request,
            snapshot.commit_view.unwrap().source_marker.as_ref().unwrap()).is_err());
        assert_eq!(fs::read(runtime.store.workspace_path(prep)).unwrap(),pending);
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }

    #[test]
    fn spec221_c19_file_marker_is_durable_before_release_and_result_after_group_stop() {
        let fixture=Fixture::new();let (runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let (_,request)=runtime.store.begin_materialization(&execution,"large.bin").unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap()).unwrap();
        let mut job=match crate::workspace_git::ViewJob::start(&proofs,&intent,&request,&std::env::current_exe().unwrap(),
            &runtime.source,|marker|runtime.store.authorize_materialization(&execution,&request,marker)
                .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)) {
                Ok(job)=>job,Err((cause,mut held))=>{
                    if let Some(child)=held.as_mut(){assert!(child.stop(Duration::from_secs(2)).unwrap());}
                    panic!("file fixture {cause:?}");
                }
            };
        let pending=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=pending.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.marker.is_some() && !operation.stopped && operation.available.is_none());
        let until=Instant::now()+Duration::from_secs(10);
        let outcome=loop {
            if let crate::workspace_git::JobPoll::Stopped(result)=job.poll().unwrap(){break result;}
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(outcome,Ok(()));
        assert!(runtime.store.finish_materialization(&execution,&request,Some(outcome)).unwrap());
        let done=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=done.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.stopped);assert_eq!(operation.available,Some(true));assert!(operation.failure.is_none());
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
        let (_,next)=runtime.store.begin_materialization(&execution,"large.bin").unwrap();
        assert!(matches!(next,crate::workspace_git::ViewJobRequest::Materialize {sequence:2,..}));
        let mut closed=runtime.store.read_slot(&execution).unwrap().unwrap();closed.producer_terminal=true;
        runtime.store.persist(&runtime.store.slot_dir(&execution).unwrap().join("receipt.json"),&closed).unwrap();
        assert!(matches!(runtime.store.begin_materialization(&execution,"tracked"),Err(WorkerStoreError::InvalidTransition)));
    }

    #[test]
    fn spec221_c19_live_file_helper_blocks_closure_even_without_its_sidecar() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let (_,request)=runtime.store.begin_materialization(&execution,"large.bin").unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap()).unwrap();
        let mut env=runtime.source.clone();env.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let mut job=match crate::workspace_git::ViewJob::start(&proofs,&intent,&request,&std::env::current_exe().unwrap(),
            &env,|marker|runtime.store.authorize_materialization(&execution,&request,marker)
                .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)) {
                Ok(job)=>job,Err((cause,mut held))=>{
                    if let Some(child)=held.as_mut(){assert!(child.stop(Duration::from_secs(2)).unwrap());}
                    panic!("file fixture {cause:?}");
                }
            };
        let root=runtime.store.view_root();let until=Instant::now()+Duration::from_secs(3);
        while !root.join("stall-ready").exists(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));}
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let marker=receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap().marker.as_ref().unwrap();
        assert!(proofs.remove_stopped_marker(&intent.binding,marker).is_err());
        assert!(root.join(format!("view-{}.json",intent.binding.key().unwrap())).exists());
        fs::remove_file(root.join(format!("view-{}.json",intent.binding.key().unwrap()))).unwrap();
        assert!(!runtime.store.settle_view_sources(&execution.command.task_id).unwrap());
        assert!(!runtime.store.finish_materialization(&execution,&request,Some(Ok(()))).unwrap());
        runtime.stop_owned_view_sources(&execution.command).unwrap();
        assert_eq!(runtime.cancel_signal_issued.len(),1);
        runtime.reconcile_all().unwrap();
        assert_eq!(runtime.cancel_signal_issued.len(),1,"maintenance must retain the issued file stop");
        runtime.stop_owned_view_sources(&execution.command).unwrap();
        assert_eq!(runtime.cancel_signal_issued.len(),1);
        assert!(!runtime.store.settle_view_sources(&execution.command.task_id).unwrap());
        let record=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap().record.unwrap();
        assert!(matches!(runtime.store.report_retained_view(&intent.binding.command,&record),Err(WorkerStoreError::Unknown)));
        // Le producteur simulé est terminé. Cela ne clôt pas son helper et ne
        // permet pas de collecter la preuve de commande après sept jours.
        runtime.store.set_phase(&execution.command,WorkerCommandPhase::Terminal,worker_now()).unwrap();
        let slot=runtime.store.read_slot(&execution).unwrap().unwrap().slot;
        runtime.store.mark_result_producer_terminal(&execution).unwrap();
        runtime.store.close_result_slot(&slot,WorkerSlotClosureProof::NeverDispatched,worker_now()).unwrap();
        assert!(matches!(runtime.reconcile(&execution.command),Err(WorkerRefusal::UnknownExecution)));
        runtime.store.collect(worker_now()+WORKER_RECEIPT_RETENTION_SECS+1000).unwrap();
        assert!(runtime.store.command_path(&execution.command).exists());
        assert!(matches!(runtime.cancel(&execution.command),Err(WorkerRefusal::UnknownExecution)));
        assert_eq!(runtime.cancel_signal_issued.len(),1);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(root.join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        loop {
            if let crate::workspace_git::JobPoll::Stopped(_)=job.poll().unwrap(){break;}
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        // Le faux helper termine sans fichier : exit0 seul ne prouve pas la disponibilité.
        assert!(runtime.store.finish_materialization(&execution,&request,Some(Ok(()))).unwrap());
        let done=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=done.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.stopped);assert_eq!(operation.available,Some(false));
        assert_eq!(operation.failure,Some(crate::workspace_git::ExportFailure::IntegrityFailed));
        assert!(runtime.store.settle_view_sources(&execution.command.task_id).unwrap());
        assert!(runtime.store.begin_materialization(&execution,"tracked").is_err());
        assert!(runtime.store.report_retained_view(&intent.binding.command,&record).is_ok());
        assert!(matches!(runtime.reconcile(&execution.command),
            Ok(WorkerResponseOperation::Observation{observation}) if observation.phase==WorkerCommandPhase::Terminal));
    }


    #[test]
    fn spec221_c19_runtime_materializes_for_live_execution_and_rechecks_each_read() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let command_before=fs::read(runtime.store.command_path(&execution.command)).unwrap();
        runtime.materialize_view_file(&execution,"large.bin").unwrap();
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
        assert!(runtime.view_sources.is_empty());
        assert_eq!(fs::read(runtime.store.command_path(&execution.command)).unwrap(),command_before);
        runtime.materialize_view_file(&execution,"large.bin").unwrap();
        let done=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        assert!(matches!(done.commit_view.as_ref().unwrap().materialization.as_ref().unwrap().request,
            crate::workspace_git::ViewJobRequest::Materialize {sequence:2,..}));
        let file=intent.destination().unwrap().join("large.bin");
        fs::set_permissions(&file,fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&file,b"keep operator work").unwrap();
        assert_eq!(runtime.materialize_view_file(&execution,"large.bin").unwrap_err().1,
            Some(crate::workspace_git::ExportFailure::IntegrityFailed));
        assert_eq!(fs::read(&file).unwrap(),b"keep operator work");
        assert!(runtime.view_sources.is_empty());
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.stopped && operation.available==Some(false));
    }

    #[test]
    fn spec221_c19_runtime_file_cancel_uses_existing_signal_and_keeps_unconfirmed_owner() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let cancelled=runtime.cancelled.clone();
        let key=json_digest(&execution.command).unwrap();
        let root=runtime.store.view_root();
        let sender=std::thread::spawn(move||{
            let until=Instant::now()+Duration::from_secs(4);
            while !root.join("stall-ready").exists(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));}
            cancelled.lock().unwrap().insert(key);
        });
        assert_eq!(runtime.materialize_view_file(&execution,"large.bin").unwrap_err(),
            (WorkerRefusal::UnknownExecution,Some(crate::workspace_git::ExportFailure::StopUnconfirmed)));
        sender.join().unwrap();
        assert_eq!(runtime.view_sources.len(),1);
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        assert!(!receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap().stopped);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty(){
            runtime.collect_view_sources();
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.stopped);assert_eq!(operation.available,Some(false));
        assert_eq!(operation.failure,Some(crate::workspace_git::ExportFailure::Cancelled));
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }


    #[test]
    fn spec221_c19_rejected_release_keeps_proof_then_allows_a_distinct_live_execution() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let (_,request)=runtime.store.begin_materialization(&execution,"large.bin").unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap()).unwrap();
        let mut held=match crate::workspace_git::ViewJob::start(&proofs,&intent,&request,&std::env::current_exe().unwrap(),
            &runtime.source,|marker|{
                runtime.store.mark_result_producer_terminal(&execution).unwrap();
                runtime.store.authorize_materialization(&execution,&request,marker)
                    .map_err(|_|crate::workspace_git::ExportFailure::ProofUnavailable)
            }) {
                Err((crate::workspace_git::ExportFailure::ProofUnavailable,Some(child)))=>child,
                _=>panic!("the ended execution must not release a file helper"),
            };
        let until=Instant::now()+Duration::from_secs(3);
        let result=loop {
            if let crate::workspace_git::JobPoll::Stopped(result)=held.poll().unwrap(){break result;}
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        };
        assert!(runtime.store.finish_materialization(&execution,&request,Some(result)).unwrap());
        drop(held);
        let marker=runtime.store.view_root().join(format!("view-{}.json",intent.binding.key().unwrap()));
        assert!(marker.exists(),"an abandoned bootstrap keeps its proof until scoped cleanup");
        assert!(!intent.destination().unwrap().join("large.bin").exists());
        let mut next=execution.clone();next.round=2;next.execution_generation=2;
        next.execution_id="file-next-execution".into();next.mission_id="file-next-mission".into();
        runtime.store.reserve_result_slot(&WorkerResultSlot {identity:next.clone(),admission_digest:"c".repeat(64)},worker_now()).unwrap();
        runtime.materialize_view_file(&next,"large.bin").unwrap();
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
        assert!(!marker.exists());assert!(runtime.view_sources.is_empty());
        assert!(runtime.store.begin_materialization(&execution,"tracked").is_err());
    }


    #[test]
    fn spec221_c19_live_file_mission_outlives_its_bootstrap_deadline() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let mut current=runtime.store.read_command(&execution.command).unwrap().unwrap();
        current.created_at=worker_now()-1000;current.deadline_at=worker_now()-900;current.updated_at=worker_now()-899;
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        runtime.materialize_view_file(&execution,"large.bin").unwrap();
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
        assert_eq!(runtime.store.read_command(&execution.command).unwrap().unwrap().deadline_at,current.deadline_at);
        assert!(runtime.view_sources.is_empty());
    }

    #[test]
    fn spec221_c19_file_precancel_has_no_reservation_or_process_effect() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.cancelled.lock().unwrap().insert(json_digest(&execution.command).unwrap());
        let before=fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap();
        assert_eq!(runtime.materialize_view_file(&execution,"large.bin").unwrap_err(),
            (WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::Cancelled)));
        assert_eq!(fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap(),before);
        assert!(!intent.destination().unwrap().join("large.bin").exists());
        assert!(runtime.view_sources.is_empty());
    }

    #[test]
    fn spec221_c19_file_unknown_cancel_state_is_not_a_cancel_request() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let cancelled=runtime.cancelled.clone();
        assert!(std::thread::spawn(move||{
            let _held=cancelled.lock().unwrap();panic!("fixture poisons only its own cancel mutex");
        }).join().is_err());
        let before=fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap();
        assert_eq!(runtime.materialize_view_file(&execution,"large.bin").unwrap_err(),
            (WorkerRefusal::StoreUnavailable,None));
        assert_eq!(fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap(),before);
        assert!(!intent.destination().unwrap().join("large.bin").exists());
        assert!(runtime.view_sources.is_empty());
    }

    #[test]
    fn spec221_c19_file_pending_leaves_worker_free_and_collector_stops_ended_mission() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.store.view_root().join("stall-ready").exists() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!runtime.poll_view_file(&read).unwrap());
        let mut other=execution.command.clone();
        other.task_id="other-file-independent-task".into();other.command_id="other-file-independent-command".into();
        other.instance_id="other-file-independent-instance".into();
        let current=runtime.store.command(&execution.command).unwrap();
        runtime.store.reserve_command(&other,&"f".repeat(64),&current.placement,worker_now()+300,worker_now()).unwrap();
        // Sans marqueur ni bootstrap, Reconcile atteste Unknown, jamais
        // un lancement. Il doit pouvoir le faire pendant la lecture en cours.
        let other_observation=runtime.reconcile(&other);
        assert!(matches!(&other_observation,
            Ok(WorkerResponseOperation::Observation{observation}) if observation.phase==WorkerCommandPhase::Unknown),
            "{other_observation:?}");
        let mut wrong=ViewFileRead {binding:read.binding.clone(),execution:read.execution.clone(),request:read.request.clone()};
        wrong.execution.execution_id="foreign-file-execution".into();
        assert_eq!(runtime.poll_view_file(&wrong).unwrap_err(),(WorkerRefusal::CommandMismatch,None));
        assert_eq!(runtime.view_sources.len(),1);
        // Même sans lecteur qui poll, le tick garde le helper et demande son
        // arrêt quand la mission se termine. Ce n'est pas un Cancel utilisateur.
        runtime.store.mark_result_producer_terminal(&execution).unwrap();
        runtime.collect_view_sources();
        assert_eq!(runtime.view_sources.len(),1);
        assert_eq!(runtime.poll_view_file(&read).unwrap_err(),(WorkerRefusal::UnknownExecution,None));
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(10));
        }
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.stopped);assert_eq!(operation.available,Some(false));
        assert_eq!(operation.failure,Some(crate::workspace_git::ExportFailure::ProofUnavailable));
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }

    #[test]
    fn spec221_c19_file_poll_keeps_failure_already_published_by_collection() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.store.view_root().join("stall-ready").exists() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() || !runtime.view_file_audits.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(10));
        }
        let before=fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap();
        for _ in 0..2 {
            assert_eq!(runtime.poll_view_file(&read).unwrap_err(),(WorkerRefusal::WorkspacePrepareFailed,
                Some(crate::workspace_git::ExportFailure::IntegrityFailed)));
            assert_eq!(fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap(),before);
        }
        assert!(runtime.view_sources.is_empty());
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }

    #[test]
    fn spec221_c19_file_wire_reconnection_keeps_one_helper_and_only_current_correlations() {
        let fixture=Fixture::new();let (mut runtime,intent,execution,_)=c19_fenced_file_fixture(&fixture);
        let request=WorkerRequest{version:REMOTE_WORKER_VERSION,worker:runtime.identity.clone(),
            command:execution.command.clone(),connection_epoch:"file-old".into(),correlation_id:"old".into(),
            request:WorkerRequestOperation::ReadCommitViewFile{execution:execution.clone(),path:"large.bin".into()}};
        assert!(runtime.defer_view_file(&request).unwrap());
        let before=runtime.store.view_receipt_for_binding(&intent.binding).unwrap()
            .commit_view.unwrap().materialization.unwrap().request;
        let mut replay=request.clone();replay.connection_epoch="file-new".into();replay.correlation_id="new".into();
        assert!(runtime.defer_view_file(&replay).unwrap());
        assert_eq!(runtime.view_files.len(),1);assert_eq!(runtime.view_sources.len(),1);
        let key=json_digest(&execution.command).unwrap();
        runtime.cancelled.lock().unwrap().insert(key.clone());
        runtime.cancelled.lock().unwrap().insert("unrelated-old-command".into());
        runtime.retain_view_cancellations().unwrap();
        assert_eq!(*runtime.cancelled.lock().unwrap(),BTreeSet::from([key.clone()]));
        // La conservation de l'annulation est vérifiée ; cet essai nominal la
        // retire explicitement avant la première livraison du fichier.
        runtime.cancelled.lock().unwrap().remove(&key);
        let until=Instant::now()+Duration::from_secs(5);
        let responses=loop {
            let responses=runtime.poll_view_files("file-new");
            if !responses.is_empty(){break responses;}
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(responses.len(),1);assert_eq!(responses[0].correlation_id,"new");
        responses[0].validate().unwrap();
        assert!(matches!(&responses[0].response,WorkerResponseOperation::CommitViewFile {
            execution:returned,path,available:true,reason:None,failure:None
        } if returned==&execution && path=="large.bin"));
        let done=runtime.store.view_receipt_for_binding(&intent.binding).unwrap();
        assert_eq!(done.commit_view.unwrap().materialization.unwrap().request,before);
        assert!(runtime.view_sources.is_empty() && runtime.view_file_audits.is_empty());
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
    }

    #[test]
    fn spec221_c19_file_audit_does_not_block_commands_or_publish_before_checks() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_FILE_AUDIT_HOLD".into(),"1".into());
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(5);
        while runtime.view_file_audits.is_empty() {
            assert!(!runtime.poll_view_file(&read).unwrap());assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        }
        let started=Instant::now();
        assert!(!runtime.poll_view_file(&read).unwrap());
        assert!(started.elapsed()<Duration::from_millis(200));
        let receipt=runtime.store.view_receipt_for_binding(&intent.binding).unwrap();
        let operation=receipt.commit_view.unwrap().materialization.unwrap();
        assert!(operation.stopped && operation.available.is_none());
        assert_eq!(runtime.begin_view_file(&execution,"tracked").err().unwrap(),
            (WorkerRefusal::ResourceLimit,Some(crate::workspace_git::ExportFailure::OperationCapacity)));
        let repeated=runtime.begin_view_file(&execution,"large.bin").unwrap();
        assert_eq!(repeated.request,read.request);
        assert!(runtime.view_sources.is_empty(),"aucun nouveau helper pendant l'audit du précédent");
        assert!(!runtime.poll_view_file(&repeated).unwrap());
        let mut independent=execution.command.clone();
        independent.command_id="file-audit-independent-command".into();
        independent.task_id="file-audit-independent-task".into();
        independent.instance_id="file-audit-independent-instance".into();
        let placement=runtime.store.command(&execution.command).unwrap().placement;
        runtime.store.reserve_command(&independent,&"f".repeat(64),&placement,worker_now()+300,worker_now()).unwrap();
        assert!(matches!(runtime.reconcile(&independent),
            Ok(WorkerResponseOperation::Observation{..})));
        runtime.cancelled.lock().unwrap().insert(json_digest(&execution.command).unwrap());
        assert_eq!(runtime.poll_view_file(&read).unwrap_err(),
            (WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::Cancelled)));
        // L'audit reste possédé même après le refus du lecteur.
        assert_eq!(runtime.view_file_audits.len(),1);
        fs::write(runtime.config.state_root.join("allow-file-audit"),b"").unwrap();
        while !runtime.view_file_audits.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(runtime.poll_view_file(&read).unwrap_err(),
            (WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::Cancelled)));
    }

    #[test]
    fn spec221_c19_file_audit_retains_completed_report_until_store_publication_recovers() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_FILE_AUDIT_HOLD".into(),"1".into());
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(5);
        while runtime.view_file_audits.is_empty() {
            assert!(!runtime.poll_view_file(&read).unwrap());assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        }
        // L'audit possède déjà l'arrêt. Sa publication ne peut pas relire
        // le reçu ; les preuves et les données de l'audit restent lisibles.
        let receipt_path=runtime.store.workspace_path(&intent.binding.command);
        fs::set_permissions(&receipt_path,fs::Permissions::from_mode(0o000)).unwrap();
        let denied=fs::File::open(&receipt_path).err().map(|error|error.kind());
        fs::write(runtime.config.state_root.join("allow-file-audit"),b"").unwrap();
        let ready=loop {
            runtime.collect_view_file_audits();
            let ready=runtime.view_file_audits.values().any(|audit|matches!(audit.state,ViewFileAuditState::Ready{..}));
            if ready || runtime.view_file_audits.is_empty() {break ready;}
            if Instant::now()>=until {break false;}
            std::thread::sleep(Duration::from_millis(5));
        };
        fs::set_permissions(&receipt_path,fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(denied,Some(io::ErrorKind::PermissionDenied),"la fixture doit constater le refus réel");
        assert!(ready,"le constat ne doit pas disparaître après une publication refusée");
        assert!(runtime.view_sources.is_empty());
        let receipt=runtime.store.view_receipt_for_binding(&intent.binding).unwrap();
        assert!(receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap().available.is_none());
        assert_eq!(runtime.begin_view_file(&execution,"tracked").err().unwrap(),
            (WorkerRefusal::ResourceLimit,Some(crate::workspace_git::ExportFailure::OperationCapacity)));
        for audit in runtime.view_file_audits.values_mut() {
            if let ViewFileAuditState::Ready{retry_at,..}=&mut audit.state {*retry_at=Instant::now();}
        }
        // Aucun nouveau poll du fichier n'est nécessaire pour finir le CAS.
        runtime.collect_view_sources();
        assert!(runtime.view_file_audits.is_empty() && runtime.view_sources.is_empty());
        let receipt=runtime.store.view_receipt_for_binding(&intent.binding).unwrap();
        let done=receipt.commit_view.unwrap().materialization.unwrap();
        assert_eq!(done.available,Some(true));assert_eq!(done.request,read.request);
        assert!(runtime.poll_view_file(&read).unwrap());
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
    }

    #[test]
    fn spec221_c19_file_audit_restarts_after_durable_stop_without_request_or_new_helper() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let key=format!("file-{}",marker_key(&intent.binding.command));
        let until=Instant::now()+Duration::from_secs(5);
        let result=loop {
            if let crate::workspace_git::JobPoll::Stopped(result)=runtime.view_sources.get_mut(&key).unwrap().child.poll().unwrap() {
                break result;
            }
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
        };
        assert!(runtime.store.stop_materialization(&execution,&read.request,Some(result)).unwrap().is_some());
        runtime.view_sources.remove(&key);
        let before=runtime.store.view_receipt_for_binding(&intent.binding).unwrap().commit_view.unwrap().materialization.unwrap();
        assert!(before.stopped && before.available.is_none());
        assert!(runtime.view_file_audits.is_empty());
        let config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,
            credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
            registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone(),
        };drop(runtime);
        let mut recovered=WorkerRuntime::new(config).unwrap();
        assert!(recovered.view_sources.is_empty() && recovered.view_files.is_empty());
        assert_eq!(recovered.view_file_audits.len(),1,"audit repris depuis le reçu, sans demande du client");
        while !recovered.view_file_audits.is_empty() {
            recovered.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        }
        let done=recovered.store.view_receipt_for_binding(&intent.binding).unwrap().commit_view.unwrap().materialization.unwrap();
        assert!(done.stopped && done.available==Some(true));
        assert_eq!(done.request,before.request);assert_eq!(done.execution,before.execution);
        assert!(recovered.view_sources.is_empty() && recovered.running.is_empty() && recovered.held.is_empty());
        assert_eq!(fs::read(intent.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
    }

    #[test]
    fn spec221_c19_file_deadline_without_in_memory_child_is_not_stop_proof() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_NO_READY".into(),"1".into());
        assert!(runtime.begin_view_file(&execution,"large.bin").is_err());
        let source_key=format!("file-{}",marker_key(&intent.binding.command));
        let held=runtime.view_sources.remove(&source_key).unwrap();
        let request=WorkerRequest{version:REMOTE_WORKER_VERSION,worker:runtime.identity.clone(),
            command:execution.command.clone(),connection_epoch:"file-timeout".into(),correlation_id:"timeout".into(),
            request:WorkerRequestOperation::ReadCommitViewFile{execution:execution.clone(),path:"large.bin".into()}};
        assert!(runtime.defer_view_file(&request).unwrap());
        for pending in runtime.view_files.values_mut(){pending.expires=Instant::now();}
        let responses=runtime.poll_view_files("file-timeout");
        assert_eq!(responses.len(),1);
        assert!(matches!(responses[0].response,WorkerResponseOperation::CommitViewFile {
            available:false,reason:Some(WorkerRefusal::PrepareExpired),
            failure:Some(crate::workspace_git::ExportFailure::StopUnconfirmed),..
        }));
        let receipt=runtime.store.view_receipt_for_binding(&intent.binding).unwrap();
        assert!(!receipt.commit_view.unwrap().materialization.unwrap().stopped);
        runtime.view_sources.insert(source_key,held);
        fs::write(runtime.store.view_root().join("allow-bootstrap-exit"),b"").unwrap();
        let until=Instant::now()+Duration::from_secs(4);
        while !runtime.view_sources.is_empty() || !runtime.view_file_audits.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }

    #[test]
    fn spec221_c19_native_closure_requires_stop_and_result_disposition_and_retains_both_for_cleanup() {
        let fixture=Fixture::new();let (runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let prep=&intent.binding.command;
        assert!(matches!(runtime.store.check_view_native_closure(prep,&intent),Err(WorkerStoreError::InvalidTransition)));
        let mut current=runtime.store.command(&execution.command).unwrap();
        current.phase=WorkerCommandPhase::Terminal;
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        assert!(matches!(runtime.store.check_view_native_closure(prep,&intent),Err(WorkerStoreError::Unknown)));
        // État de commande simulé, jamais un fournisseur lancé : employer
        // le groupe de l'export réellement terminé pour éprouver ces reçus.
        let receipt=runtime.store.checked_view_receipt(prep,&intent).unwrap();
        let mut marker=receipt.commit_view.as_ref().unwrap().source_marker.clone().unwrap();
        marker.instance_id=execution.command.instance_id.clone();marker.command_id=execution.command.command_id.clone();
        marker.generation=execution.command.generation;
        current.marker=Some(marker);
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        if cfg!(target_os="linux") {
            assert!(matches!(runtime.store.check_view_native_closure(prep,&intent),Err(WorkerStoreError::Unknown)));
            let proof=WorkerDrainProof::expected(&current).unwrap();
            runtime.store.persist(&drain_proof_path(&runtime.store.root,&execution.command),&proof).unwrap();
        }
        assert!(matches!(runtime.store.check_view_native_closure(prep,&intent),Err(WorkerStoreError::InvalidTransition)));
        let stored=runtime.store.read_slot(&execution).unwrap().unwrap();
        let outcome=result(&stored.slot,"result retained until parent disposition");
        publish_result_blob(&runtime.store.reservation(&stored.slot).unwrap(),&outcome).unwrap();
        runtime.store.mark_result_producer_terminal(&execution).unwrap();
        runtime.store.close_result_slot(&stored.slot,WorkerSlotClosureProof::Cancelled,worker_now()).unwrap();
        assert!(matches!(runtime.store.check_view_native_closure(prep,&intent),Err(WorkerStoreError::InvalidTransition)),
            "cancelled is not an ACK of a published result");
        runtime.store.record_disposition(&WorkerResultDisposition {identity:execution.clone(),
            result_digest:outcome.digest,disposition:RemoteResultDisposition::Accepted{}},worker_now()).unwrap();
        let before=fs::read(runtime.store.workspace_path(prep)).unwrap();
        runtime.store.check_view_native_closure(prep,&intent).unwrap();
        assert_eq!(fs::read(runtime.store.workspace_path(prep)).unwrap(),before);
        runtime.store.collect(worker_now()+WORKER_RECEIPT_RETENTION_SECS+1).unwrap();
        assert!(runtime.store.read_slot(&execution).unwrap().is_some());
        assert!(runtime.store.command(&execution.command).is_ok());
        runtime.store.check_view_native_closure(prep,&intent).unwrap();
        let mut unsafe_receipt=runtime.store.checked_view_receipt(prep,&intent).unwrap();
        unsafe_receipt.commit_view.as_mut().unwrap().source_stopped=false;
        runtime.store.persist(&runtime.store.workspace_path(prep),&unsafe_receipt).unwrap();
        assert!(matches!(runtime.store.check_view_native_closure(prep,&intent),Err(WorkerStoreError::InvalidTransition)));
        assert!(intent.destination().unwrap().join("tracked").is_file());
        assert!(runtime.store.view_root().join(format!("{}.intent.json",intent.binding.key().unwrap())).is_file());
    }

    #[test]
    fn spec221_c19_native_closure_never_infers_missing_command_history_from_clean_files() {
        let fixture=Fixture::new();let (runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.store.mark_result_producer_terminal(&execution).unwrap();
        let stored=runtime.store.read_slot(&execution).unwrap().unwrap();
        runtime.store.close_result_slot(&stored.slot,WorkerSlotClosureProof::Cancelled,worker_now()).unwrap();
        fs::remove_file(runtime.store.command_path(&execution.command)).unwrap();
        assert!(matches!(runtime.store.check_view_native_closure(&intent.binding.command,&intent),
            Err(WorkerStoreError::Quarantined)));
        assert_eq!(fs::read(intent.destination().unwrap().join("tracked")).unwrap(),b"committed");
    }


    /// La mission est simulée, son helper et son export sont réels. Ne pas
    /// confondre ces fixtures de reçus avec une qualification fournisseur.
    fn c19_close_file_fixture(runtime:&WorkerRuntime,intent:&crate::workspace_git::ViewIntent,
        execution:&WorkerResultIdentity) {
        let mut current=runtime.store.command(&execution.command).unwrap();
        current.phase=WorkerCommandPhase::Terminal;
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,intent).unwrap();
        let mut marker=receipt.commit_view.as_ref().unwrap().source_marker.clone().unwrap();
        marker.instance_id=execution.command.instance_id.clone();
        marker.command_id=execution.command.command_id.clone();
        marker.generation=execution.command.generation;
        current.marker=Some(marker);
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        if cfg!(target_os="linux") {
            let proof=WorkerDrainProof::expected(&current).unwrap();
            runtime.store.persist(&drain_proof_path(&runtime.store.root,&execution.command),&proof).unwrap();
        }
        let stored=runtime.store.read_slot(execution).unwrap().unwrap();
        let outcome=result(&stored.slot,"closed fixture result");
        publish_result_blob(&runtime.store.reservation(&stored.slot).unwrap(),&outcome).unwrap();
        runtime.store.mark_result_producer_terminal(execution).unwrap();
        runtime.store.record_disposition(&WorkerResultDisposition {identity:execution.clone(),
            result_digest:outcome.digest,disposition:RemoteResultDisposition::Accepted{}},worker_now()).unwrap();
    }

    #[test]
    fn spec221_c19_native_conservation_port_requires_exact_binding_stop_and_disposition() {
        let fixture=Fixture::new();
        let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let command=intent.binding.command.clone();
        let before=fs::read(runtime.store.workspace_path(&command)).unwrap();
        let file=intent.destination().unwrap().join("tracked");
        let bytes=fs::read(&file).unwrap();
        let request=WorkerRequest {
            version:REMOTE_WORKER_VERSION,correlation_id:"conserve-view-221".into(),
            connection_epoch:"view-epoch-221".into(),worker:runtime.identity.clone(),
            command:command.clone(),
            request:WorkerRequestOperation::ConserveCommitView {prepare_digest:intent.binding.prepare_digest.clone()},
        };
        request.validate().unwrap();
        let disconnected=AtomicBool::new(false);
        assert!(runtime.handle(&request,"view-epoch-221",&disconnected).is_err());
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),before);
        assert_eq!(fs::read(&file).unwrap(),bytes);
        // Le fournisseur n'est pas lancé par cette fixture ; arrêt et résultat
        // simulés, export et reçus de préparation réels.
        c19_close_file_fixture(&runtime,&intent,&execution);
        let current=fs::read(runtime.store.workspace_path(&command)).unwrap();
        let mut wrong=request.clone();
        wrong.request=WorkerRequestOperation::ConserveCommitView {prepare_digest:"f".repeat(64)};
        assert!(runtime.handle(&wrong,"view-epoch-221",&disconnected).is_err());
        assert_eq!(fs::read(runtime.store.workspace_path(&command)).unwrap(),current);
        let mut foreign=request.clone();foreign.worker.worker_id="foreign-worker".into();
        assert_eq!(runtime.handle(&foreign,"view-epoch-221",&disconnected),Err(WorkerRefusal::StaleConnection));
        let private_intent=runtime.store.view_root().join(format!("{}.intent.json",intent.binding.key().unwrap()));
        let original_intent=fs::read(&private_intent).unwrap();
        let answer=runtime.handle(&request,"view-epoch-221",&disconnected).unwrap();
        assert_eq!(answer,WorkerResponseOperation::CommitViewConserved {prepare_digest:intent.binding.prepare_digest.clone()});
        let wire=WorkerResponse {version:request.version,correlation_id:request.correlation_id.clone(),
            connection_epoch:request.connection_epoch.clone(),worker:request.worker.clone(),
            command:request.command.clone(),response:answer.clone()};
        wire.validate().unwrap();
        assert_eq!(runtime.handle(&request,"view-epoch-221",&disconnected).unwrap(),answer);
        let retained=runtime.store.checked_view_receipt(&command,&intent).unwrap();
        assert!(retained.commit_view.as_ref().unwrap().closed);
        assert_eq!(retained.record.as_ref().unwrap().state,crate::delegation::WorkspaceState::Kept);
        assert_eq!(fs::read(&file).unwrap(),bytes);
        assert!(runtime.view_sources.is_empty());
        assert_eq!(fs::read(&private_intent).unwrap(),original_intent,"aucune réduction ou audit de données implicite");
        runtime.store.collect(worker_now()+WORKER_RECEIPT_RETENTION_SECS+1).unwrap();
        assert!(runtime.store.checked_view_receipt(&command,&intent).unwrap().commit_view.unwrap().closed);
        assert!(runtime.handle(&request,"view-epoch-221",&disconnected).is_ok());
    }

    #[test]
    fn spec221_c19_conservation_capability_never_appears_for_an_old_master() {
        let absent=MasterWorkerFeatures::default();
        assert!(!absent.supports(WorkerCapability::WorkspaceViewV1));
        assert!(!absent.admits(&[WorkerCapability::WorkspaceViewV1]));
        let present=MasterWorkerFeatures {workspace_view:true,..Default::default()};
        assert_eq!(present.capabilities(),vec![WorkerCapability::WorkspaceViewV1]);
        assert!(present.admits(&[WorkerCapability::WorkspaceViewV1]));
        assert!(!present.admits(&[WorkerCapability::WorkspaceViewV1,WorkerCapability::WorkspaceViewV1]));
    }

    #[test]
    fn spec221_c19_closed_view_compaction_restarts_each_phase_without_losing_quota_or_data() {
        let fixture=Fixture::new();let (runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let prep=&intent.binding.command;
        let path=runtime.store.workspace_path(prep);
        let active=fs::read(&path).unwrap();
        assert!(runtime.store.compact_closed_view(prep,&intent).is_err());
        assert_eq!(fs::read(&path).unwrap(),active);
        c19_close_file_fixture(&runtime,&intent,&execution);
        let data_path=intent.destination().unwrap();
        let card_path=data_path.join(".bridget-commit-view.json");
        let before=fs::read(&card_path).unwrap();let meta=fs::metadata(&card_path).unwrap();
        let original=runtime.store.inventory().unwrap().2;
        runtime.store.freeze_view(prep,&intent).unwrap();
        let frozen=runtime.store.checked_view_receipt(prep,&intent).unwrap();
        assert!(frozen.commit_view.as_ref().unwrap().closed);
        assert_eq!(frozen.commit_view.as_ref().unwrap().reserved_bytes,intent.reserved_bytes().unwrap());
        assert!(runtime.store.reserve_view_intent(prep,
            &runtime.store.command(&execution.command).unwrap().preparation.unwrap().0,&intent).is_err());
        // Même avec un reçu d'exécution mal restauré en Released, le tombstone
        // workspace refuse la réouverture avant chargement du manifeste.
        let mut current=runtime.store.command(&execution.command).unwrap();
        current.phase=WorkerCommandPhase::Released;
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        assert!(runtime.store.active_view_execution(&execution).is_err());
        current.phase=WorkerCommandPhase::Terminal;
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        let proofs=crate::workspace_git::ViewProofs::open(&runtime.store.view_root().canonicalize().unwrap()).unwrap();
        // Coupure après remplacement atomique, avant réduction du reçu.
        let reserved=proofs.compact_closed(&intent).unwrap();
        assert_eq!(runtime.store.checked_view_receipt(prep,&intent).unwrap().commit_view.unwrap().reserved_bytes,
            intent.reserved_bytes().unwrap());
        assert!(runtime.store.inventory().unwrap().2>=original.saturating_sub(512));
        runtime.store.compact_closed_view(prep,&intent).unwrap();
        let closed=runtime.store.checked_view_receipt(prep,&intent).unwrap();
        assert_eq!(closed.commit_view.as_ref().unwrap().reserved_bytes,reserved);
        let closed_bytes=fs::read(&path).unwrap();
        runtime.store.compact_closed_view(prep,&intent).unwrap();
        assert_eq!(fs::read(&path).unwrap(),closed_bytes);
        assert_eq!(fs::read(&card_path).unwrap(),before);
        assert_eq!(fs::metadata(&card_path).unwrap().ino(),meta.ino());
        assert_eq!(fs::metadata(&card_path).unwrap().mtime(),meta.mtime());
        assert_eq!(fs::read(data_path.join("tracked")).unwrap(),b"committed");
        assert!(!data_path.join("large.bin").exists());
        assert!(runtime.store.report_retained_view(prep,closed.record.as_ref().unwrap()).is_ok());
        runtime.store.collect(worker_now()+WORKER_RECEIPT_RETENTION_SECS+301).unwrap();
        assert!(runtime.store.read_slot(&execution).unwrap().is_none());
        assert!(runtime.store.command(&execution.command).is_err());
        // Après collecte de l'histoire, la preuve close garde son refus de
        // recréation et son rapport ; la clôture ne se déduit pas d'une absence.
        runtime.store.compact_closed_view(prep,&intent).unwrap();
        assert_eq!(fs::read(&path).unwrap(),closed_bytes);
        assert!(runtime.store.report_retained_view(prep,closed.record.as_ref().unwrap()).is_ok());
    }

    #[test]
    fn spec221_c19_closed_view_keeps_full_reservation_when_card_is_missing_or_modified() {
        let fixture=Fixture::new();let (runtime,intent,execution)=c19_active_file_fixture(&fixture);
        c19_close_file_fixture(&runtime,&intent,&execution);
        let prep=&intent.binding.command;
        let card=intent.destination().unwrap().join(".bridget-commit-view.json");
        let original=fs::read(&card).unwrap();
        fs::write(&card,b"{}").unwrap();
        assert!(runtime.store.compact_closed_view(prep,&intent).is_err());
        let receipt=runtime.store.checked_view_receipt(prep,&intent).unwrap();
        assert!(receipt.commit_view.as_ref().unwrap().closed);
        assert_eq!(receipt.commit_view.as_ref().unwrap().reserved_bytes,intent.reserved_bytes().unwrap());
        assert_eq!(fs::read(&card).unwrap(),b"{}");
        fs::write(&card,&original).unwrap();
        runtime.store.compact_closed_view(prep,&intent).unwrap();
        fs::remove_file(&card).unwrap();
        assert!(runtime.store.report_retained_view(prep,receipt.record.as_ref().unwrap()).is_err());
        assert!(runtime.store.compact_closed_view(prep,&intent).is_err());
        assert_eq!(fs::read(intent.destination().unwrap().join("tracked")).unwrap(),b"committed");
    }


    #[test]
    fn spec221_c19_closed_materialized_view_rejects_late_result_without_data_or_receipt_change() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(10);
        while !runtime.poll_view_file(&read).unwrap() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        c19_close_file_fixture(&runtime,&intent,&execution);
        runtime.store.compact_closed_view(&intent.binding.command,&intent).unwrap();
        let before=fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap();
        let file=intent.destination().unwrap().join("large.bin");
        let bytes=fs::read(&file).unwrap();
        assert!(runtime.store.finish_materialization(&execution,&read.request,
            Some(Err(crate::workspace_git::ExportFailure::Unknown))).is_err());
        assert_eq!(fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap(),before);
        assert_eq!(fs::read(&file).unwrap(),bytes);
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        assert!(runtime.store.report_retained_view(&intent.binding.command,receipt.record.as_ref().unwrap()).is_ok());
    }



    /// La preuve198est réelle ; seul l'état de missionB est simulé pour
    /// éprouver son port de fichiers sans lancer un fournisseur.
    fn c19_fenced_file_fixture(fixture:&Fixture)
        ->(WorkerRuntime,crate::workspace_git::ViewIntent,WorkerResultIdentity,WorkerCommandIdentity) {
        let (runtime,intent,prepared)=c19_prepared_view_fixture(fixture);
        let mut previous=prepared.command.clone();
        previous.instance_id="never-released-original".into();previous.generation=1;
        runtime.store.fence_unstarted(&previous,&prepared.prepared.placement,worker_now()+300,worker_now()).unwrap();
        let mut next=prepared.command.clone();next.command_id=uuid::Uuid::new_v4().to_string();
        runtime.store.bind_fenced_workspace(&previous,&next,&prepared.request,
            &prepared.prepared.placement,prepared.prepared.workspace.as_ref().unwrap()).unwrap();
        let mut command=next.clone();command.instance_id="fenced-reader".into();command.generation=1;
        let now=worker_now();
        runtime.store.reserve_command(&command,&"d".repeat(64),&prepared.prepared.placement,now+300,now).unwrap();
        let mut current=runtime.store.command(&command).unwrap();
        current.phase=WorkerCommandPhase::Released;current.native_ready=true;
        current.preparation=Some((prepared.request,prepared.prepared));
        current.child_id=Some("fenced-file-fixture".into());
        runtime.store.persist(&runtime.store.command_path(&command),&current).unwrap();
        let execution=WorkerResultIdentity {command,mission_id:"fenced-mission".into(),round:1,
            execution_id:"fenced-execution".into(),execution_generation:1};
        runtime.store.reserve_result_slot(&WorkerResultSlot {identity:execution.clone(),admission_digest:"c".repeat(64)},now).unwrap();
        (runtime,intent,execution,previous)
    }

    #[test]
    fn spec221_c19_fenced_prepare_reuses_the_same_export_through_owned_audit_without_creator() {
        let fixture=Fixture::new();
        let (mut runtime,intent,prepared)=c19_prepared_view_fixture(&fixture);
        let mut previous=prepared.command.clone();previous.instance_id="never-released-view".into();previous.generation=1;
        runtime.store.fence_unstarted(&previous,&prepared.prepared.placement,worker_now()+300,worker_now()).unwrap();
        let mut next=prepared.command.clone();next.command_id=uuid::Uuid::new_v4().to_string();
        let original=fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap();
        let files=fs::read_dir(&intent.data_root).unwrap().count();
        let mut request=WorkerRequest{version:REMOTE_WORKER_VERSION,worker:runtime.identity.clone(),command:next.clone(),
            connection_epoch:"fenced-view-epoch".into(),correlation_id:"fenced-view-only".into(),
            request:WorkerRequestOperation::PrepareFenced{previous,prepare:prepared.request.clone(),
                placement:prepared.prepared.placement.clone(),workspace:prepared.prepared.workspace.clone(),routing:None}};
        request.validate().unwrap();
        assert!(request.request.requires_commit_view_reader());
        assert!(!request.request.requires_commit_view_creation());
        // La relecture clôture la préparation existante ; elle ne produit
        // aucune nouvelle extraction, même si ce binaire sait aussi créer.
        assert!(runtime.defer_view_control(&request).unwrap());
        let until=Instant::now()+Duration::from_secs(5);
        let mut observed=Vec::new();
        for round in 0..2 {
            let responses=loop {
                let responses=runtime.poll_view_controls("fenced-view-epoch",&AtomicBool::new(false));
                if !responses.is_empty(){break responses;}
                assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));
            };
            assert_eq!(responses.len(),1);responses[0].validate().unwrap();
            let WorkerResponseOperation::Prepared{prepared:result}=&responses[0].response
                else{panic!("vue réutilisée attendue round={round}, {:?}",responses[0].response)};
            assert_eq!(result.placement,prepared.prepared.placement);
            assert_eq!(result.workspace,prepared.prepared.workspace);
            observed.push(result.token.clone());
            assert_eq!(runtime.prepared[&next.command_id].request,prepared.request);
            if round==0 {
                request.correlation_id="fenced-view-replay".into();
                assert!(runtime.defer_view_control(&request).unwrap());
            }
        }
        assert_eq!(observed[0],observed[1]);
        assert_eq!(fs::read(runtime.store.workspace_path(&intent.binding.command)).unwrap(),original);
        assert_eq!(fs::read_dir(&intent.data_root).unwrap().count(),files);
        assert!(runtime.view_sources.is_empty() && runtime.running.is_empty() && runtime.held.is_empty());
        assert_eq!(original_workspace_at(&runtime.store.root,&runtime.identity,&next).unwrap().path,
            intent.destination().unwrap().to_string_lossy());
    }

    #[test]
    fn spec221_c19_fenced_file_uses_original_anchor_once_and_reports_the_same_view() {
        let fixture=Fixture::new();
        let (mut runtime,intent,execution,_)=c19_fenced_file_fixture(&fixture);
        let before=fs::read_dir(&intent.data_root).unwrap().count();
        let anchor=runtime.store.active_view_execution(&execution).unwrap();
        assert_eq!(anchor.0.binding,intent.binding);
        assert_eq!(anchor.1.command,intent.binding.command);
        assert_eq!(runtime.store.view_receipts(None).unwrap().len(),1,"one reservation, not one per alias");
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(10);
        while !runtime.poll_view_file(&read).unwrap() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(runtime.view_sources.is_empty());
        assert_eq!(fs::read_dir(&intent.data_root).unwrap().count(),before);
        assert!(intent.destination().unwrap().join("large.bin").is_file());
        let anchor=runtime.store.active_view_execution(&execution).unwrap().1;
        assert!(runtime.store.report_retained_view(&execution.command,anchor.record.as_ref().unwrap()).is_ok());
    }

    #[test]
    fn spec221_c19_fenced_file_requires_bind_canon_and_original_unclosed_anchor() {
        let fixture=Fixture::new();
        let (runtime,intent,execution,_)=c19_fenced_file_fixture(&fixture);
        let mut prep=execution.command.clone();prep.instance_id.clear();prep.generation=0;
        let bind=proof198::bind_path(&runtime.store.root,&prep);
        let saved=fs::read(&bind).unwrap();
        fs::remove_file(&bind).unwrap();
        assert!(runtime.store.active_view_execution(&execution).is_err());
        write_private_file_atomic(&bind,&saved).unwrap();
        let mut current=runtime.store.command(&execution.command).unwrap();
        let original=current.preparation.as_ref().unwrap().0.clone();
        current.preparation.as_mut().unwrap().0.base_commit=Some("f".repeat(40));
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        assert!(runtime.store.begin_materialization(&execution,"large.bin").is_err());
        current.preparation.as_mut().unwrap().0=original;
        runtime.store.persist(&runtime.store.command_path(&execution.command),&current).unwrap();
        assert!(runtime.store.active_view_execution(&execution).is_ok());
        let mut anchor=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        anchor.record.as_mut().unwrap().state=crate::delegation::WorkspaceState::Kept;
        anchor.commit_view.as_mut().unwrap().closed=true;
        runtime.store.persist(&runtime.store.workspace_path(&intent.binding.command),&anchor).unwrap();
        assert!(runtime.store.active_view_execution(&execution).is_err());
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }

    #[test]
    fn spec221_c19_fenced_file_cancel_matches_current_operation_not_original_command() {
        let fixture=Fixture::new();
        let (mut runtime,intent,execution,previous)=c19_fenced_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let read=runtime.begin_view_file(&execution,"large.bin").unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.store.view_root().join("stall-ready").exists() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        runtime.cancel_signal_issued.insert(format!("view-source-{}-file",intent.binding.key().unwrap()));
        let prior=runtime.stop_owned_view_sources(&previous);
        let unchanged=runtime.store.active_view_execution(&execution).unwrap().1
            .commit_view.unwrap().materialization.unwrap();
        let stopped=runtime.stop_owned_view_sources(&execution.command);
        let count=runtime.cancel_signal_issued.len();
        let repeated=runtime.stop_owned_view_sources(&execution.command);
        let same_count=runtime.cancel_signal_issued.len();
        let pending=runtime.store.active_view_execution(&execution).unwrap().1
            .commit_view.unwrap().materialization.unwrap();
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(prior.is_ok() && stopped.is_ok() && repeated.is_ok());
        assert!(!unchanged.stopped && !pending.stopped);
        assert_eq!(count,same_count,"one signal per exact operation");
        assert!(count>1,"old generic stop key did not suppress the new operation");
        assert_eq!(runtime.poll_view_file(&read).unwrap_err(),(WorkerRefusal::WorkspacePrepareFailed,
            Some(crate::workspace_git::ExportFailure::Cancelled)));
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }


    #[test]
    fn spec221_c19_source_fence_before_publication_survives_restart_without_reopening() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        let read=runtime.begin_view_source(&prep,&request,intent.clone()).unwrap();
        let until=Instant::now()+Duration::from_secs(5);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(intent.destination().unwrap().join("tracked").is_file());
        let mut full=prep.clone();full.instance_id="never-launched-source".into();full.generation=1;
        runtime.fence_unstarted(&full,&placement(),worker_now()+300).unwrap();
        assert!(runtime.poll_view_source(&read).is_err());
        let receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        assert!(receipt.record.is_none());
        let config=WorkerRuntimeConfig {machine_id:runtime.config.machine_id.clone(),
            worker_id:runtime.config.worker_id.clone(),provisioning_revision:runtime.config.provisioning_revision,
            credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
            registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone()};
        drop(runtime);
        let mut reopened=WorkerRuntime::new(config).unwrap();
        assert!(reopened.cancelled.lock().unwrap().is_empty());
        assert!(reopened.begin_view_source(&prep,&request,intent.clone()).is_err());
        assert!(reopened.view_sources.is_empty());
        assert!(reopened.store.checked_view_receipt(&prep,&intent).unwrap().record.is_none());
        assert!(intent.destination().unwrap().join("tracked").is_file());
    }

    #[test]
    fn spec221_c19_source_pending_polls_use_only_small_receipts_and_do_not_wait() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let read=runtime.begin_view_source(&prep,&request,intent.clone()).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.store.view_root().join("stall-ready").exists() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        let path=runtime.store.view_root().join(format!("{}.intent.json",intent.binding.key().unwrap()));
        let away=runtime.store.view_root().join("temporarily-unavailable");
        fs::rename(&path,&away).unwrap();
        let started=Instant::now();
        let pending=(0..10).map(|_|runtime.poll_view_source(&read)).collect::<Vec<_>>();
        let elapsed=started.elapsed();
        let mut wrong=ViewSourceRead {binding:read.binding.clone()};
        wrong.binding.prepare_digest="f".repeat(64);
        let mismatch=runtime.poll_view_source(&wrong);
        let inventory=runtime.store.inventory();
        fs::rename(&away,&path).unwrap();
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        assert!(pending.iter().all(|result|matches!(result,Ok(None))));
        assert!(elapsed<Duration::from_secs(1),"pending polls {elapsed:?}");
        assert!(matches!(mismatch,Err((WorkerRefusal::CommandMismatch,None))));
        assert!(inventory.is_ok());
        assert!(!intent.destination().unwrap().exists());
        assert!(runtime.poll_view_source(&read).is_err(),"exit zero without publication is not an export");
        assert!(runtime.store.checked_view_receipt(&prep,&intent).unwrap().record.is_none());
    }

    #[test]
    fn spec221_c19_source_failure_survives_collection_and_restart_without_second_spawn() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        fs::rename(&intent.source,fixture.0.join("source-offline")).unwrap();
        let read=runtime.begin_view_source(&prep,&request,intent.clone()).unwrap();
        let until=Instant::now()+Duration::from_secs(5);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        let receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        let view=receipt.commit_view.as_ref().unwrap();
        assert!(view.source_stopped);
        let cause=view.source_failure.expect("closed helper cause survives the lost reader");
        let marker=view.source_marker.clone();
        assert_eq!(runtime.poll_view_source(&read).unwrap_err(),(WorkerRefusal::WorkspacePrepareFailed,Some(cause)));
        let config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,
            credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
            registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut reopened=WorkerRuntime::new(config).unwrap();
        let before=fs::read(reopened.store.workspace_path(&prep)).unwrap();
        let replay=reopened.begin_view_source(&prep,&request,intent.clone());
        assert!(matches!(replay,Err((WorkerRefusal::WorkspacePrepareFailed,Some(actual))) if actual==cause));
        assert!(reopened.view_sources.is_empty());
        assert_eq!(fs::read(reopened.store.workspace_path(&prep)).unwrap(),before);
        assert_eq!(reopened.store.checked_view_receipt(&prep,&intent).unwrap().commit_view.unwrap().source_marker,marker);
        assert!(!intent.destination().unwrap().exists());
    }

    #[test]
    fn spec221_c19_source_cancel_before_begin_has_no_receipt_or_process_effect() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        runtime.cancelled.lock().unwrap().insert(json_digest(&prep).unwrap());
        let result=runtime.begin_view_source(&prep,&request,intent.clone());
        assert!(matches!(result,Err((WorkerRefusal::WorkspacePrepareFailed,Some(crate::workspace_git::ExportFailure::Cancelled)))));
        assert!(!runtime.store.workspace_path(&prep).exists());
        assert!(runtime.view_sources.is_empty());
        assert!(!intent.destination().unwrap().exists());
    }

    #[test]
    fn spec221_c19_source_cancel_is_collected_without_reader_and_preserves_uncertain_stop() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_STALL".into(),"1".into());
        let read=runtime.begin_view_source(&prep,&request,intent.clone()).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.store.view_root().join("stall-ready").exists() {
            assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        runtime.cancelled.lock().unwrap().insert(json_digest(&prep).unwrap());
        runtime.collect_view_sources();
        let pending=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        let held=runtime.view_sources.len();
        let stopped=runtime.store.confirm_view_sources_stopped(&prep.task_id);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(held,1);
        assert!(!pending.commit_view.as_ref().unwrap().source_stopped);
        assert!(!stopped.unwrap());
        let receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        let view=receipt.commit_view.as_ref().unwrap();
        assert!(view.source_stopped);
        assert_eq!(view.source_failure,Some(crate::workspace_git::ExportFailure::Cancelled));
        assert_eq!(runtime.poll_view_source(&read).unwrap_err(),(WorkerRefusal::WorkspacePrepareFailed,
            Some(crate::workspace_git::ExportFailure::Cancelled)));
        assert!(receipt.record.is_none());
        assert!(!intent.destination().unwrap().exists());
    }

    #[test]
    fn spec221_c19_unready_bootstrap_without_marker_is_not_a_stop_proof() {
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_NO_READY".into(),"1".into());
        assert!(runtime.prepare_view_source(&prep,&request,intent.clone()).is_err());
        assert_eq!(runtime.view_sources.len(),1);
        assert!(runtime.store.view_root().join("unready-started").is_file());
        let stopped=runtime.store.confirm_view_sources_stopped(&prep.task_id);
        let receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        let claimed=receipt.commit_view.as_ref().unwrap().source_stopped;
        eprintln!("C19 unready retained={} stop_confirmed={stopped:?} source_stopped={claimed}",
            runtime.view_sources.len());
        let key=marker_key(&prep);
        let held=runtime.view_sources.remove(&key).unwrap();
        let config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,
            credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
            registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut runtime=WorkerRuntime::new(config).unwrap();
        let replay=runtime.prepare_view_source(&prep,&request,intent.clone());
        assert!(matches!(replay,Err((WorkerRefusal::UnknownExecution,
            Some(crate::workspace_git::ExportFailure::StopUnconfirmed)))));
        assert!(runtime.view_sources.is_empty(),"restart must not start another bootstrap");
        runtime.view_sources.insert(key,held);
        // Toujours libérer notre fixture avant les assertions du RED.
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-bootstrap-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(matches!(stopped,Ok(false)));
        assert!(!claimed,"missing READY/marker cannot attest that a retained Child is stopped");
        assert!(!intent.destination().unwrap().exists());
    }

    #[test]
    fn spec221_c19_unready_file_without_marker_stays_unconfirmed_and_is_not_restarted() {
        let fixture=Fixture::new();let (mut runtime,intent,execution)=c19_active_file_fixture(&fixture);
        runtime.source.insert("B221_VIEW_TEST_NO_READY".into(),"1".into());
        assert!(runtime.begin_view_file(&execution,"large.bin").is_err());
        assert_eq!(runtime.view_sources.len(),1);
        let stopped=runtime.store.settle_view_sources(&execution.command.task_id).unwrap();
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let operation=receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(operation.attempted && operation.marker.is_none() && !operation.stopped);
        let key=format!("file-{}",marker_key(&intent.binding.command));
        let held=runtime.view_sources.remove(&key).unwrap();
        let recovered=runtime.begin_view_file(&execution,"large.bin").unwrap();
        assert!(runtime.view_sources.is_empty());
        assert_eq!(runtime.poll_view_file(&recovered).unwrap_err(),
            (WorkerRefusal::UnknownExecution,Some(crate::workspace_git::ExportFailure::StopUnconfirmed)));
        runtime.view_sources.insert(key,held);
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(runtime.store.view_root().join("allow-bootstrap-exit")).unwrap();
        let until=Instant::now()+Duration::from_secs(3);
        while !runtime.view_sources.is_empty() {
            runtime.collect_view_sources();assert!(Instant::now()<until);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!stopped);
        let receipt=runtime.store.checked_view_receipt(&intent.binding.command,&intent).unwrap();
        let done=receipt.commit_view.as_ref().unwrap().materialization.as_ref().unwrap();
        assert!(done.stopped && done.available==Some(false));
        assert_eq!(done.failure,Some(crate::workspace_git::ExportFailure::StartupUnavailable));
        assert!(!intent.destination().unwrap().join("large.bin").exists());
    }

    #[test]
    fn spec221_c19_settled_source_does_not_recheck_an_unrelated_reused_group() {
        use std::os::unix::process::CommandExt;
        let fixture=Fixture::new();
        let (unused,prep,request,intent)=c19_store_fixture(&fixture);drop(unused);
        let mut runtime=runtime_fixture(&fixture);
        let original=runtime.prepare_view_source(&prep,&request,intent.clone()).unwrap();
        let mut unrelated=std::process::Command::new("/bin/cat").stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()).process_group(0).spawn().unwrap();
        let pid=unrelated.id();
        let birth=crate::managed_process::process_birth(pid).unwrap();
        let mut receipt=runtime.store.checked_view_receipt(&prep,&intent).unwrap();
        let view=receipt.commit_view.as_mut().unwrap();
        assert!(view.source_stopped);
        let marker=view.source_marker.as_mut().unwrap();
        marker.pgid=pid;marker.birth=birth.saturating_sub(1).max(1);
        runtime.store.persist(&runtime.store.workspace_path(&prep),&receipt).unwrap();
        let restored=runtime.prepare_view_source(&prep,&request,intent);
        let still_live=unrelated.try_wait().unwrap().is_none();
        drop(unrelated.stdin.take());
        assert!(unrelated.wait().unwrap().success());
        assert!(still_live,"no signal may target the unrelated group");
        assert!(restored.as_ref().is_ok_and(|record|same_workspace_anchor(&original,
            &serde_json::to_value(record).unwrap())),"{restored:?}");
    }

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("bridget-worker162-{}", uuid::Uuid::new_v4()));
            create_private_dir(&path).unwrap();
            Self(path)
        }
        fn root(&self) -> PathBuf {
            self.0.join("worker")
        }
        fn open(&self) -> WorkerStore {
            WorkerStore::open(&self.root(), worker()).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn worker() -> WorkerIdentity {
        WorkerIdentity {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
        }
    }
    fn command() -> WorkerCommandIdentity {
        WorkerCommandIdentity {
            command_id: "command-162".into(),
            task_id: "task-162".into(),
            instance_id: "instance-162".into(),
            generation: 1,
        }
    }
    fn placement() -> RemotePlacement {
        RemotePlacement {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
            project_id: "project-162".into(),
            mapping_revision: 1,
            origin_host: "mac".into(),
            origin_cwd: "/project".into(),
            source_root_identity: "repo".into(),
            relative_cwd: ".".into(),
            execution_cwd: "/project".into(),
            base_commit: None,
            definition_digest: "a".repeat(64),
            permission_digest: "b".repeat(64),
        }
    }
    fn slot(index: usize) -> WorkerResultSlot {
        WorkerResultSlot {
            identity: WorkerResultIdentity {
                command: command(),
                mission_id: format!("mission-{index}"),
                round: 1,
                execution_id: format!("execution-{index}"),
                execution_generation: 1,
            },
            admission_digest: "c".repeat(64),
        }
    }
    fn reserve(store: &WorkerStore) {
        assert!(matches!(
            store
                .reserve_command(&command(), &"d".repeat(64), &placement(), 1000, 100)
                .unwrap(),
            CommandReservation::Created(_)
        ));
    }
    fn result(slot: &WorkerResultSlot, body: &str) -> RemoteResultEnvelope {
        let mut result = RemoteResultEnvelope {
            worker: worker(),
            identity: slot.identity.clone(),
            message_id: "message-162".into(),
            body: body.into(),
            digest: String::new(),
        };
        result.digest = result.canonical_digest().unwrap();
        result
    }

    #[test]
    fn worker162_rejeu_reouvert_ne_recree_jamais_commande() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        drop(store);
        let store = fixture.open();
        assert!(matches!(
            store
                .reserve_command(&command(), &"d".repeat(64), &placement(), 1000, 101)
                .unwrap(),
            CommandReservation::Existing(WorkerCommandReceipt {
                phase: WorkerCommandPhase::Reserved,
                ..
            })
        ));
        assert!(matches!(
            store.reserve_command(&command(), &"e".repeat(64), &placement(), 1000, 101),
            Err(WorkerStoreError::Conflict)
        ));
        store
            .set_phase(&command(), WorkerCommandPhase::Unknown, 102)
            .unwrap();
        assert!(matches!(
            store.set_phase(&command(), WorkerCommandPhase::Released, 103),
            Err(WorkerStoreError::InvalidTransition)
        ));
        assert!(store.reserve_result_slot(&slot(1), 103).is_err());
    }

    #[test]
    fn worker162_verrou_et_enrollment_ne_masquent_pas_disparition() {
        let fixture = Fixture::new();
        let store = fixture.open();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Locked)
        ));
        drop(store);
        fs::remove_dir_all(fixture.root()).unwrap();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
        assert!(!fixture.root().exists());
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
    }

    #[test]
    fn worker162_store_preexistant_corrompu_ou_non_prive_refuse() {
        let fixture = Fixture::new();
        create_private_dir(&fixture.root()).unwrap();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let path = store.command_path(&command());
        fs::write(&path, b"{invalid").unwrap();
        drop(store);
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
        let fixture = Fixture::new();
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
    }

    #[test]
    fn worker172_workspace_refusals_preserve_cause_without_private_details() {
        for (error, expected) in [
            ("workspace_disk_full: fatal: private/path secret-token", WorkerRefusal::DiskFull),
            ("workspace_access_denied: private/path secret-token", WorkerRefusal::FilesystemAccessDenied),
            ("workspace_create_failed: private/path secret-token", WorkerRefusal::WorkspacePrepareFailed),
            ("workspace_create_failed: workspace_disk_full: private/path", WorkerRefusal::WorkspacePrepareFailed),
        ] {
            let refusal = refuse(workspace_failure(error.into()));
            assert_eq!(refusal, expected);
            let encoded = serde_json::to_string(&refusal).unwrap();
            assert!(!encoded.contains("private") && !encoded.contains("secret"));
            assert_eq!(serde_json::from_str::<WorkerRefusal>(&encoded).unwrap(), expected);
        }
        for errno in [libc::ENOSPC, libc::EDQUOT] {
            assert_eq!(refuse(io::Error::from_raw_os_error(errno).into()), WorkerRefusal::DiskFull);
        }
        for errno in [libc::EACCES, libc::EPERM, libc::EROFS] {
            assert_eq!(refuse(io::Error::from_raw_os_error(errno).into()), WorkerRefusal::FilesystemAccessDenied);
        }
        assert_eq!(refuse(io::Error::other("private secret").into()), WorkerRefusal::StoreUnavailable);
        assert_eq!(refuse(WorkerStoreError::Workspace(WorkerRefusal::WorkspacePrepareIncomplete)), WorkerRefusal::WorkspacePrepareIncomplete);
        assert_eq!(refuse(WorkerStoreError::Quota), WorkerRefusal::ResourceLimit);
        assert_eq!(refuse(WorkerStoreError::Quarantined), WorkerRefusal::StoreUnavailable);
    }

    #[test]
    fn worker172_release_apres_attestation_lente_garde_revision_et_expiration() {
        for (now, mapping_revision, accepted) in [(112, 1, true), (162, 1, false), (112, 2, false)] {
            let fixture = Fixture::new();
            let store = fixture.open();
            reserve(&store);
            store.mark_command(&command(), &ManagedMarker {
                pgid:42, birth:7, instance_id:command().instance_id,
                command_id:command().command_id, generation:command().generation,
            }, 101).unwrap();
            let permit = WorkerReleasePermit {
                permit_id:"slow-proof172".into(), command_digest:"d".repeat(64),
                permission_revision:1, mapping_revision, provisioning_revision:1,
                issued_at:102, expires_at:102 + WORKER_RELEASE_LEASE_SECS,
            };
            let result = store.authorize_release(&command(), &permit, "epoch172", now);
            assert_eq!(result.is_ok(), accepted, "{now}/{mapping_revision}: {result:?}");
            assert_eq!(store.command(&command()).unwrap().phase,
                if accepted { WorkerCommandPhase::ReleaseAuthorized } else { WorkerCommandPhase::Marked });
            if accepted {
                assert!(store.authorize_release(&command(), &permit, "other-epoch", now).is_err());
            }
        }
    }

    #[test]
    fn worker162_marqueur_et_permis_precedent_release() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        assert!(
            store
                .set_phase(&command(), WorkerCommandPhase::Released, 101)
                .is_err()
        );
        let mut marker = ManagedMarker {
            pgid: 42,
            birth: 7,
            instance_id: command().instance_id,
            command_id: command().command_id,
            generation: 1,
        };
        marker.generation = 2;
        assert!(store.mark_command(&command(), &marker, 101).is_err());
        marker.generation = 1;
        store.mark_command(&command(), &marker, 101).unwrap();
        let permit = WorkerReleasePermit {
            permit_id: "permit".into(),
            command_digest: "d".repeat(64),
            permission_revision: 1,
            mapping_revision: 1,
            provisioning_revision: 1,
            issued_at: 102,
            expires_at: 107,
        };
        assert!(
            store
                .authorize_release(&command(), &permit, "epoch-1", 107)
                .is_err()
        );
        store
            .authorize_release(&command(), &permit, "epoch-1", 103)
            .unwrap();
        assert!(
            store
                .authorize_release(&command(), &permit, "epoch-2", 103)
                .is_err()
        );
        assert_eq!(
            store.command(&command()).unwrap().phase,
            WorkerCommandPhase::ReleaseAuthorized
        );
        store
            .set_phase(&command(), WorkerCommandPhase::Released, 104)
            .unwrap();
        assert_eq!(store.command(&command()).unwrap().marker, Some(marker));
        assert!(!store.command(&command()).unwrap().native_ready);
        store.native_status(&command(), false).unwrap();
        // Même description de fichier qu’un descripteur hérité entre fork et exec.
        // CLOEXEC ne libère pas le verrou avant exec ; le propriétaire doit le faire.
        use std::os::fd::{FromRawFd, OwnedFd};
        let inherited_fd=unsafe { libc::fcntl(store._lock.as_raw_fd(),libc::F_DUPFD_CLOEXEC,0) };
        assert!(inherited_fd>=0);
        let inherited=unsafe { OwnedFd::from_raw_fd(inherited_fd) };
        drop(store);
        let store = fixture.open();
        drop(inherited);
        assert!(store.command(&command()).unwrap().native_ready);
        store.native_status(&command(), true).unwrap();
        store.native_status(&command(), false).unwrap();
        let receipt = store.command(&command()).unwrap();
        assert!(receipt.startup_failed && !receipt.native_ready);

        let failed_fixture = Fixture::new();
        let failed_store = failed_fixture.open();
        reserve(&failed_store);
        failed_store
            .mark_command(&command(), &receipt.marker.unwrap(), 101)
            .unwrap();
        failed_store
            .authorize_release(&command(), &permit, "epoch-1", 103)
            .unwrap();
        assert!(matches!(
            failed_store.native_status(&command(), false),
            Err(WorkerStoreError::InvalidTransition)
        ));
        failed_store.native_status(&command(), true).unwrap();
        drop(failed_store);
        let failed_store = failed_fixture.open();
        let failed_receipt = failed_store.command(&command()).unwrap();
        assert_eq!(failed_receipt.phase, WorkerCommandPhase::ReleaseAuthorized);
        assert!(failed_receipt.startup_failed && !failed_receipt.native_ready);
        assert!(matches!(
            failed_store.native_status(&command(), false),
            Err(WorkerStoreError::InvalidTransition)
        ));
    }

    #[test]
    fn worker162_blob_immutable_et_relais_apres_reouverture() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let slot = slot(1);
        let reservation = store.reserve_result_slot(&slot, 101).unwrap();
        let metadata =
            fs::read(store.slot_dir(&slot.identity).unwrap().join("receipt.json")).unwrap();
        let result = result(&slot, "résultat exact");
        publish_result_blob(&reservation, &result).unwrap();
        publish_result_blob(&reservation, &result).unwrap();
        assert_eq!(
            metadata,
            fs::read(store.slot_dir(&slot.identity).unwrap().join("receipt.json")).unwrap()
        );
        let resolved = find_result_reservation(
            &fixture.root(),
            &worker(),
            &command(),
            &slot.identity.mission_id,
            &slot.identity.execution_id,
            slot.identity.execution_generation,
        )
        .unwrap();
        assert_eq!(resolved, reservation);
        drop(store);
        let store = fixture.open();
        assert_eq!(
            store.read_result(&slot.identity).unwrap(),
            Some(result.clone())
        );
        let mut divergent = result.clone();
        divergent.body = "autre résultat".into();
        divergent.digest = divergent.canonical_digest().unwrap();
        assert!(matches!(
            publish_result_blob(&reservation, &divergent),
            Err(WorkerStoreError::Conflict)
        ));
        assert!(matches!(
            store.read_result(&slot.identity),
            Err(WorkerStoreError::Quarantined)
        ));
        assert_eq!(
            read_optional::<RemoteResultEnvelope>(
                &reservation.blob_path,
                MAX_WORKER_RESULT_ENVELOPE_BYTES
            )
            .unwrap(),
            Some(result)
        );
        drop(store);
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
    }

    #[test]
    fn spec172_worker_store_conserve_replays_sans_reserver_les_recus_termines() {
        let fixture = Fixture::new();
        let store = fixture.open();
        let mut first = slot(0);
        first.identity.command.command_id = "completed-0".into();
        first.identity.command.task_id = "completed-task-0".into();
        let expected_result = result(&first, "résultat conservé");
        let mut first_receipt = Vec::new();
        for index in 0..120 {
            let mut slot = slot(index);
            slot.identity.command.command_id = format!("completed-{index}");
            slot.identity.command.task_id = format!("completed-task-{index}");
            let command = &slot.identity.command;
            store.reserve_command(command, &"d".repeat(64), &placement(), 1000, 100)
                .unwrap();
            let reservation = store.reserve_result_slot(&slot, 101)
                .unwrap_or_else(|error| panic!("ancien reçu {index}: {error}"));
            let result = result(&slot, "résultat conservé");
            publish_result_blob(&reservation, &result).unwrap();
            store.record_disposition(&WorkerResultDisposition {
                identity: slot.identity.clone(),
                result_digest: result.digest,
                disposition: RemoteResultDisposition::Accepted {},
            }, 102).unwrap();
            store.mark_result_producer_terminal(&slot.identity).unwrap();
            store.close_result_slot(&slot, WorkerSlotClosureProof::Cancelled, 103).unwrap();
            store.set_phase(command, WorkerCommandPhase::Terminal, 104).unwrap();
            if index == 0 {
                first_receipt = fs::read(store.slot_dir(&slot.identity).unwrap().join("receipt.json")).unwrap();
            }
        }
        // Les anciens résultats et leurs reçus restent présents, byte pour byte.
        assert_eq!(store.read_result(&first.identity).unwrap(), Some(expected_result.clone()));
        assert_eq!(first_receipt, fs::read(store.slot_dir(&first.identity).unwrap().join("receipt.json")).unwrap());
        assert_eq!(store.inventory().unwrap().3, 240);
        drop(store);
        let store = fixture.open();
        assert_eq!(store.read_result(&first.identity).unwrap(), Some(expected_result));
        assert!(matches!(
            store.reserve_command(&first.identity.command, &"d".repeat(64), &placement(), 1000, 100).unwrap(),
            CommandReservation::Existing(_)
        ));
        // Les résultats vivants continuent de réserver leur enveloppe complète.
        reserve(&store);
        for index in 0..64 {
            store.reserve_result_slot(&slot(index), 101).unwrap();
        }
        let (_,_,used,_)=store.inventory().unwrap();
        assert!(used<MAX_WORKER_STORE_BYTES as u64);
    }

    #[test]
    fn spec172_worker_store_quota_distincte_dune_panne_de_stockage() {
        assert_eq!(refuse(WorkerStoreError::Quota), WorkerRefusal::ResourceLimit);
        assert_eq!(refuse(WorkerStoreError::Quarantined), WorkerRefusal::StoreUnavailable);
        assert_eq!(refuse(WorkerStoreError::Io(io::Error::other("détail privé"))),
            WorkerRefusal::StoreUnavailable);
    }

    #[test]
    fn spec221_result_capacity_reserves_64_parallel_results_and_preserves_orphan_accounting() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        // Un reçu de commande par mission, comme le vrai parcours d'admission.
        let mut first=None;
        for index in 0..64 {
            let mut slot=slot(index);
            slot.identity.command.command_id=format!("parallel-{index}");
            slot.identity.command.task_id=format!("parallel-task-{index}");
            if index==0 { first=Some(slot.clone()); }
            store.reserve_command(&slot.identity.command,&"d".repeat(64),&placement(),1000,100).unwrap();
            store.reserve_result_slot(&slot,101).unwrap();
        }
        let (_,slots,used,count)=store.inventory().unwrap();
        assert_eq!(slots.len(),64);
        assert!(count>=128 && used<MAX_WORKER_STORE_BYTES as u64);
        // Le prochain slot réutilise une commande déjà réservée : seule
        // sa capacité de retour doit provoquer le refus après remplissage.
        let next=slot(65);
        let before=used;
        let orphan=store.root.join(".orphan.tmp");
        write_private_file_atomic(&orphan,b"").unwrap();
        fs::OpenOptions::new().write(true).open(&orphan).unwrap()
            .set_len(MAX_WORKER_STORE_BYTES as u64-before).unwrap();
        assert_eq!(store.inventory().unwrap().2,MAX_WORKER_STORE_BYTES as u64);
        assert!(matches!(store.reserve_result_slot(&next,101),Err(WorkerStoreError::Quota)));
        assert!(orphan.exists());
        assert!(store.read_slot(&first.unwrap().identity).unwrap().is_some());
    }

    #[test]
    fn worker162_publication_compte_json_et_refuse_symlink_et_temporaire() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let first = slot(1);
        let reservation = store.reserve_result_slot(&first, 101).unwrap();
        let control_body = "\0".repeat(262_144);
        let escaped = result(&first, &control_body);
        assert!(serde_json::to_vec(&escaped).unwrap().len() > control_body.len() * 5);
        publish_result_blob(&reservation, &escaped).unwrap();
        let second = slot(2);
        let reservation = store.reserve_result_slot(&second, 101).unwrap();
        let payload = reservation.blob_path.parent().unwrap();
        write_private_file_atomic(&payload.join(".blob.json.orphan.tmp"), b"incomplete").unwrap();
        assert!(matches!(
            publish_result_blob(&reservation, &result(&second, "result")),
            Err(WorkerStoreError::Quarantined)
        ));
        let third = slot(3);
        let reservation = store.reserve_result_slot(&third, 101).unwrap();
        symlink(fixture.0.join("outside"), &reservation.blob_path).unwrap();
        assert!(matches!(
            publish_result_blob(&reservation, &result(&third, "result")),
            Err(WorkerStoreError::Quarantined)
        ));
        assert!(!fixture.0.join("outside").exists());
    }

    #[test]
    fn worker162_collecte_exige_disposition_producteur_et_cloture() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let slot = slot(1);
        let reservation = runtime.store.reserve_result_slot(&slot, 101).unwrap();
        let result = result(&slot, "result");
        publish_result_blob(&reservation, &result).unwrap();
        let mut disposition = WorkerResultDisposition {
            identity: slot.identity.clone(),
            result_digest: result.digest.clone(),
            disposition: RemoteResultDisposition::Deferred {
                reason: RemoteResultDeferredReason::QuotaHold,
                retry_after: 120,
            },
        };
        runtime.store.record_disposition(&disposition, 110).unwrap();
        assert!(runtime.store.pending_results(119).unwrap().is_empty());
        assert_eq!(runtime.store.pending_results(120).unwrap().len(), 1);
        let empty = self::slot(2);
        runtime.store.reserve_result_slot(&empty, 111).unwrap();
        let reserved_before = runtime.store.inventory().unwrap().2;
        // Simule le crash après la preuve terminale, avant la clôture des slots.
        runtime
            .store
            .set_phase(&command(), WorkerCommandPhase::Terminal, 111)
            .unwrap();
        runtime.reconcile(&command()).unwrap();
        let closed = runtime.store.read_slot(&empty.identity).unwrap().unwrap();
        assert!(closed.producer_terminal);
        assert_eq!(closed.closed, Some(WorkerSlotClosureProof::Cancelled));
        assert!(
            reserved_before - runtime.store.inventory().unwrap().2
                >= (MAX_WORKER_RESULT_ENVELOPE_BYTES - MAX_METADATA_BYTES) as u64
        );
        // La reprise est idempotente : elle ne repousse pas les sept jours.
        runtime.reconcile(&command()).unwrap();
        assert_eq!(
            runtime
                .store
                .read_slot(&empty.identity)
                .unwrap()
                .unwrap()
                .finalized_at,
            closed.finalized_at
        );
        assert_eq!(
            runtime.store.pending_results(120).unwrap(),
            vec![result.clone()]
        );
        let mut next = command();
        next.command_id = "next-command162".into();
        next.instance_id = "next-instance162".into();
        runtime
            .store
            .reserve_command(&next, &"d".repeat(64), &placement(), 1000, 112)
            .unwrap();
        let mut next_slot = self::slot(3);
        next_slot.identity.command = next;
        runtime.store.reserve_result_slot(&next_slot, 112).unwrap();
        runtime
            .store
            .set_phase(
                &next_slot.identity.command,
                WorkerCommandPhase::Refused,
                113,
            )
            .unwrap();
        runtime.reconcile(&next_slot.identity.command).unwrap();
        assert_eq!(
            runtime
                .store
                .command(&next_slot.identity.command)
                .unwrap()
                .phase,
            WorkerCommandPhase::Refused
        );
        assert!(
            runtime
                .store
                .read_slot(&next_slot.identity)
                .unwrap()
                .unwrap()
                .closed
                .is_some()
        );
        assert_eq!(runtime.store.collect(200).unwrap(), 0);
        assert!(reservation.blob_path.exists());
        disposition.disposition = RemoteResultDisposition::Accepted {};
        runtime.store.record_disposition(&disposition, 112).unwrap();
        assert!(runtime.store.pending_results(120).unwrap().is_empty());
        assert_eq!(runtime.store.collect(200).unwrap(), 1);
        assert!(
            runtime
                .store
                .slot_dir(&slot.identity)
                .unwrap()
                .join("receipt.json")
                .exists()
        );
        runtime.store.record_disposition(&disposition, 201).unwrap(); // ACK perdu : compact rejouable.
        runtime
            .store
            .collect(112 + WORKER_RECEIPT_RETENTION_SECS)
            .unwrap();
        assert!(!runtime.store.slot_dir(&slot.identity).unwrap().exists());
        runtime
            .store
            .set_phase(&command(), WorkerCommandPhase::Terminal, 200)
            .unwrap();
        runtime
            .store
            .collect(worker_now() + 1000 + WORKER_RECEIPT_RETENTION_SECS)
            .unwrap();
        assert!(matches!(
            runtime.store.command(&command()),
            Err(WorkerStoreError::Unknown)
        ));
        assert!(
            runtime
                .store
                .reserve_command(&command(), &"d".repeat(64), &placement(), 1000, 1001)
                .is_err()
        );
    }

    #[test]
    fn worker162_slot_caduc_ne_libere_que_sur_preuve_sans_arret_wrapper() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let slot = slot(1);
        let reservation = store.reserve_result_slot(&slot, 101).unwrap();
        store
            .close_result_slot(&slot, WorkerSlotClosureProof::NeverDispatched, 102)
            .unwrap();
        assert!(publish_result_blob(&reservation, &result(&slot, "late")).is_err());
        assert!(store.reserve_result_slot(&slot, 103).is_err());
        assert_eq!(
            store.command(&command()).unwrap().phase,
            WorkerCommandPhase::Reserved
        );
        assert_eq!(store.collect(200).unwrap(), 0);
    }

    #[test]
    fn worker174_collect_distingue_terminal_actif_et_reference() {
        let fixture = Fixture::new();
        let store = fixture.open();
        let terminal = command_174("terminal");
        let neighbour = command_174("terminal-voisin");
        let active = command_174("active");
        let kept_by_slot = command_174("kept");
        for identity in [&terminal, &neighbour, &active, &kept_by_slot] {
            reserve_174(&store, identity);
        }
        let pending = slot_174(&neighbour);
        store.reserve_result_slot(&pending, 100).unwrap();
        let kept_slot = slot_174(&kept_by_slot);
        store.reserve_result_slot(&kept_slot, 100).unwrap();
        store.set_phase(&terminal, WorkerCommandPhase::Terminal, 100).unwrap();
        store.set_phase(&kept_by_slot, WorkerCommandPhase::Terminal, 100).unwrap();
        assert_eq!(store.collect(101 + WORKER_RECEIPT_RETENTION_SECS).unwrap(), 0);
        assert!(matches!(store.command(&terminal), Err(WorkerStoreError::Unknown)));
        assert!(store.command(&neighbour).is_ok());
        assert!(store.command(&active).is_ok());
        assert!(store.command(&kept_by_slot).is_ok());
        assert!(store.read_slot(&pending.identity).unwrap().is_some());
        assert!(store.read_slot(&kept_slot.identity).unwrap().is_some());
    }

    fn command_174(id: &str) -> WorkerCommandIdentity {
        WorkerCommandIdentity {
            command_id: id.into(),
            task_id: "task-174".into(),
            instance_id: "instance-174".into(),
            generation: 1,
        }
    }
    fn reserve_174(store: &WorkerStore, identity: &WorkerCommandIdentity) {
        store
            .reserve_command(identity, &"d".repeat(64), &placement(), 101, 100)
            .unwrap();
    }
    fn slot_174(command: &WorkerCommandIdentity) -> WorkerResultSlot {
        WorkerResultSlot {
            identity: WorkerResultIdentity {
                command: command.clone(),
                mission_id: "mission-174".into(),
                round: 1,
                execution_id: "execution-174".into(),
                execution_generation: 1,
            },
            admission_digest: "c".repeat(64),
        }
    }

    fn runtime_fixture(fixture: &Fixture) -> WorkerRuntime {
        let state = fixture.0.join("runtime");
        private_dir(&state).unwrap();
        let network = fixture.0.join("network");
        private_dir(&network).unwrap();
        let credential = fixture.0.join("credential");
        write_private_file_atomic(&credential, b"worker162-test-secret").unwrap();
        let registry = fixture.0.join("registry.json");
        write_json(&registry, &serde_json::json!({"agents":{},"machines":{"cartae":{
            "enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
            "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],"mappings":{}
        }}})).unwrap();
        WorkerRuntime::new(WorkerRuntimeConfig {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
            credential_path: credential,
            state_root: state,
            registry_path: registry,
            master_socket: network.join("s"),
        })
        .unwrap()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn incident171_cancel_after_exit_requires_matching_drain_and_closes_slots() {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let result_slot = slot(1);
        runtime.store.reserve_result_slot(&result_slot, 101).unwrap();
        let mut child = Command::new("/bin/cat")
            .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null())
            .process_group(0).spawn().unwrap();
        let marker = ManagedMarker {
            pgid: child.id(),
            birth: crate::managed_process::process_birth(child.id()).unwrap(),
            instance_id: command().instance_id,
            command_id: command().command_id,
            generation: command().generation,
        };
        runtime.store.mark_command(&command(), &marker, 101).unwrap();
        write_json(&runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command()))), &marker).unwrap();
        let permit = WorkerReleasePermit {
            permit_id: "cancel-after-exit171".into(), command_digest: "d".repeat(64),
            permission_revision: 1, mapping_revision: 1, provisioning_revision: 1,
            issued_at: 101, expires_at: 106,
        };
        runtime.store.authorize_release(&command(), &permit, "epoch", 101).unwrap();
        drop(child.stdin.take());
        assert!(child.wait().unwrap().success());
        assert!(!crate::managed_process::group_exists(marker.pgid).unwrap());
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Unknown));
        assert!(!runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().producer_terminal);
        let receipt = runtime.store.command(&command()).unwrap();
        let path = drain_proof_path(&runtime.store.root, &command());
        let mut proof = WorkerDrainProof::expected(&receipt).unwrap();
        proof.birth += 1;
        write_json(&path, &proof).unwrap();
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Unknown));
        proof.birth -= 1;
        write_json(&path, &proof).unwrap();
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Terminal));
        let closed = runtime.store.read_slot(&result_slot.identity).unwrap().unwrap();
        assert!(closed.producer_terminal);
        assert_eq!(closed.closed, Some(WorkerSlotClosureProof::Cancelled));
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Terminal));
        assert_eq!(runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().finalized_at, closed.finalized_at);
        let mut foreign = command(); foreign.generation += 1;
        assert_eq!(runtime.cancel(&foreign).unwrap_err(), WorkerRefusal::CommandMismatch);
    }

    #[test]
    fn worker162_runtime_rejeu_reserved_reste_unknown_sans_bootstrap() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let definition = runtime
            .registry()
            .unwrap()
            .resolved_definition("codex")
            .unwrap();
        let mut placement = placement();
        placement.definition_digest = definition.digest.clone();
        let mut start = WorkerStart {
            prepared_token: "token-perdu".into(),
            child_id: "child162".into(),
            mission_id: "mission162".into(),
            forward_child_requests: false,
            placement,
            definition,
            child_policy: serde_json::json!({}),
            effective_posture: bridget_transport::protocol::SpawnPosture::Development,
            deadline_at: worker_now() + 300,
        };
        let digest = worker_command_digest(&worker(), &command(), &start).unwrap();
        runtime
            .store
            .reserve_command(
                &command(),
                &digest,
                &start.placement,
                start.deadline_at,
                worker_now(),
            )
            .unwrap();
        assert!(matches!(
            runtime.reconcile(&command()).unwrap(),
            WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase: WorkerCommandPhase::Unknown,
                    ..
                }
            }
        ));
        let disconnected = AtomicBool::new(false);
        assert!(matches!(
            runtime.start(&command(), &start, &disconnected, &mut WorkerStartCapacityCause::Unknown).unwrap(),
            WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase: WorkerCommandPhase::Unknown,
                    ..
                }
            }
        ));
        assert!(
            runtime.running.is_empty() && runtime.held.is_empty() && runtime.abandoned.is_empty()
        );
        start.child_id = "autre-child".into();
        assert_eq!(
            runtime
                .start(&command(), &start, &disconnected, &mut WorkerStartCapacityCause::Unknown)
                .unwrap_err(),
            WorkerRefusal::CommandMismatch
        );
    }

    #[test]
    fn worker162_reprise_frontieres_durables_ne_reemet_ni_bootstrap_ni_release() {
        use std::os::unix::process::CommandExt;
        use std::process::{Child, Command, Stdio};
        struct EofProcess(Child);
        impl Drop for EofProcess {
            fn drop(&mut self) {
                drop(self.0.stdin.take());
                let _ = self.0.wait();
            }
        }
        // Les EOF et crash avant/après marqueur sont éprouvés par les primitives
        // managed_process. Ici, leur état durable traverse le vrai recover/start.
        for phase in [
            WorkerCommandPhase::Reserved,
            WorkerCommandPhase::Marked,
            WorkerCommandPhase::ReleaseAuthorized,
            WorkerCommandPhase::Released,
        ] {
            let fixture = Fixture::new();
            let mut runtime = runtime_fixture(&fixture);
            // Released a dépassé son délai de bootstrap, pas son mandat métier.
            let admitted_at = worker_now()
                - if phase == WorkerCommandPhase::Released {
                    400
                } else {
                    0
                };
            let definition = runtime
                .registry()
                .unwrap()
                .resolved_definition("codex")
                .unwrap();
            let mut placement = placement();
            placement.definition_digest = definition.digest.clone();
            let start = WorkerStart {
                prepared_token: "token-avant-crash".into(),
                child_id: "child162".into(),
                mission_id: "mission162".into(),
                forward_child_requests: false,
                placement,
                definition,
                child_policy: serde_json::json!({}),
                effective_posture: bridget_transport::protocol::SpawnPosture::Discovery,
                deadline_at: admitted_at + 300,
            };
            let digest = worker_command_digest(&worker(), &command(), &start).unwrap();
            runtime
                .store
                .reserve_command(
                    &command(),
                    &digest,
                    &start.placement,
                    start.deadline_at,
                    admitted_at,
                )
                .unwrap();
            // Un processus possédé reste vivant au passage création→marqueur.
            // Son EOF termine le groupe sans signal ni processus laissé derrière.
            let mut owned = EofProcess(
                Command::new("/bin/cat")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .process_group(0)
                    .spawn()
                    .unwrap(),
            );
            let process = &mut owned.0;
            let marker = ManagedMarker {
                pgid: process.id(),
                birth: crate::managed_process::process_birth(process.id()).unwrap(),
                instance_id: command().instance_id,
                command_id: command().command_id,
                generation: 1,
            };
            let permit = WorkerReleasePermit {
                permit_id: "permit-avant-crash".into(),
                command_digest: digest,
                permission_revision: 1,
                mapping_revision: 1,
                provisioning_revision: 1,
                issued_at: admitted_at,
                expires_at: admitted_at + 5,
            };
            if phase != WorkerCommandPhase::Reserved {
                runtime
                    .store
                    .mark_command(&command(), &marker, admitted_at)
                    .unwrap();
            }
            if matches!(
                phase,
                WorkerCommandPhase::ReleaseAuthorized | WorkerCommandPhase::Released
            ) {
                runtime
                    .store
                    .authorize_release(&command(), &permit, "epoch-avant", admitted_at)
                    .unwrap();
            }
            if phase == WorkerCommandPhase::Released {
                runtime
                    .store
                    .set_phase(&command(), phase, admitted_at)
                    .unwrap();
                runtime.store.native_status(&command(), false).unwrap();
            }
            let mut second = slot(2);
            second.identity.round = 2;
            let second_result = result(&second, "résultat après délai bootstrap");
            if phase == WorkerCommandPhase::Released {
                let mut registry: serde_json::Value =
                    serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap())
                        .unwrap();
                registry["machines"]["cartae"]["mappings"]["project-162"] = serde_json::json!({
                    "source_host":"mac","source_root":"/project","remote_root":"/project",
                    "expected_git_identity":"repo","mapping_revision":1
                });
                write_json(&runtime.config.registry_path, &registry).unwrap();
                let request = WorkerRequest {
                    version: REMOTE_WORKER_VERSION,
                    correlation_id: "second-slot".into(),
                    connection_epoch: "epoch".into(),
                    worker: worker(),
                    command: command(),
                    request: WorkerRequestOperation::ReserveResultSlot {
                        slot: second.clone(),
                    },
                };
                assert!(worker_now() > start.deadline_at);
                assert!(
                    matches!(runtime.handle(&request, "epoch", &AtomicBool::new(false)).unwrap(),
                    WorkerResponseOperation::ResultSlotReserved { slot, .. } if slot == second)
                );
                let reservation = runtime.store.reservation(&second).unwrap();
                publish_result_blob(&reservation, &second_result).unwrap();
                registry["machines"]["cartae"]["enabled"] = false.into();
                write_json(&runtime.config.registry_path, &registry).unwrap();
            } else {
                assert!(matches!(
                    runtime
                        .store
                        .reserve_result_slot(&second, start.deadline_at + 1),
                    Err(WorkerStoreError::InvalidTransition)
                ));
                assert!(matches!(
                    runtime.store.authorize_release(
                        &command(),
                        &permit,
                        "epoch-avant",
                        start.deadline_at + 1
                    ),
                    Err(WorkerStoreError::Invalid(WorkerRefusal::ReleaseExpired))
                ));
            }
            let config = WorkerRuntimeConfig {
                machine_id: runtime.config.machine_id.clone(),
                worker_id: runtime.config.worker_id.clone(),
                provisioning_revision: runtime.config.provisioning_revision,
                credential_path: runtime.config.credential_path.clone(),
                state_root: runtime.config.state_root.clone(),
                registry_path: runtime.config.registry_path.clone(),
                master_socket: runtime.config.master_socket.clone(),
            };
            drop(runtime);
            let mut recovered = WorkerRuntime::new(config).unwrap();
            let expected = if phase == WorkerCommandPhase::Released {
                phase
            } else {
                WorkerCommandPhase::Unknown
            };
            let observed = recovered
                .start(&command(), &start, &AtomicBool::new(false), &mut WorkerStartCapacityCause::Unknown)
                .unwrap();
            assert!(
                matches!(observed, WorkerResponseOperation::Observation { observation }
                if observation.phase == expected && observation.native_ready == (phase == WorkerCommandPhase::Released)),
                "frontière {phase:?}"
            );
            let receipt = recovered.store.command(&command()).unwrap();
            assert_eq!(receipt.phase, expected);
            assert_eq!(
                receipt.marker,
                (phase != WorkerCommandPhase::Reserved).then_some(marker.clone())
            );
            assert!(
                recovered.prepared.is_empty()
                    && recovered.held.is_empty()
                    && recovered.running.is_empty()
            );
            assert_eq!(
                recovered.release(&command(), &permit, "epoch-apres", &AtomicBool::new(false)),
                Err(WorkerRefusal::UnknownExecution)
            );
            if phase == WorkerCommandPhase::Released {
                assert!(recovered.registry().is_ok()); // Authentification conserve la reprise disabled.
                assert_eq!(
                    recovered.registry().unwrap().machine("cartae").unwrap_err(),
                    "machine_disabled"
                );
                assert_eq!(
                    recovered.store.pending_results(worker_now()).unwrap(),
                    vec![second_result.clone()]
                );
                let mut request = WorkerRequest {
                    version: REMOTE_WORKER_VERSION,
                    correlation_id: "second-replay".into(),
                    connection_epoch: "epoch".into(),
                    worker: worker(),
                    command: command(),
                    request: WorkerRequestOperation::ReserveResultSlot {
                        slot: second.clone(),
                    },
                };
                assert!(
                    matches!(recovered.handle(&request, "epoch", &AtomicBool::new(false)).unwrap(),
                    WorkerResponseOperation::ResultSlotReserved { slot, .. } if slot == second)
                );
                let mut third = slot(3);
                third.identity.round = 3;
                request.request = WorkerRequestOperation::ReserveResultSlot { slot: third };
                assert_eq!(
                    recovered.handle(&request, "epoch", &AtomicBool::new(false)),
                    Err(WorkerRefusal::PlacementMismatch)
                );
                let disposition = WorkerResultDisposition {
                    identity: second.identity.clone(),
                    result_digest: second_result.digest.clone(),
                    disposition: RemoteResultDisposition::Accepted {},
                };
                request.request = WorkerRequestOperation::ResultDisposition {
                    result: disposition.clone(),
                };
                assert_eq!(
                    recovered
                        .handle(&request, "epoch", &AtomicBool::new(false))
                        .unwrap(),
                    WorkerResponseOperation::DispositionRecorded {
                        result: disposition
                    }
                );
                assert!(
                    recovered
                        .store
                        .pending_results(worker_now())
                        .unwrap()
                        .is_empty()
                );
            }
            assert!(process.try_wait().unwrap().is_none());
            assert_eq!(
                crate::managed_process::process_birth(process.id()).unwrap(),
                marker.birth
            );
            drop(process.stdin.take());
            assert!(process.wait().unwrap().success());
            #[cfg(target_os = "linux")]
            if matches!(
                phase,
                WorkerCommandPhase::ReleaseAuthorized | WorkerCommandPhase::Released
            ) {
                // Wrapper disparu après RELEASE : ni la reprise ni l'ancien reçu
                // ne prouvent le drainage. La capacité doit rester retenue.
                let observed = recovered.reconcile(&command()).unwrap();
                assert!(
                    matches!(observed, WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Unknown)
                );
                // Même refus pour la branche abandoned après RELEASE possible.
                let mut reaped = Command::new("/bin/true").spawn().unwrap();
                reaped.wait().unwrap();
                recovered.abandoned.insert(command().command_id, reaped);
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Unknown));
                assert!(recovered.abandoned.remove(&command().command_id).is_some());
                let receipt = recovered.store.command(&command()).unwrap();
                let path = drain_proof_path(&recovered.store.root, &command());
                write_private_file_atomic(&path, b"not-json").unwrap();
                assert!(recovered.store.drainage_proven(&receipt).is_err());
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Unknown));
                let mut proof = WorkerDrainProof::expected(&receipt).unwrap();
                proof.birth += 1;
                write_json(&path, &proof).unwrap();
                assert!(!recovered.store.drainage_proven(&receipt).unwrap());
                proof.birth -= 1;
                write_json(&path, &proof).unwrap();
                assert!(recovered.store.drainage_proven(&receipt).unwrap());
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Terminal));
                fs::remove_file(path).unwrap();
                // Les reçus déjà terminaux restent historiques.
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Terminal));
            }
        }
    }

    #[test]
    fn incident171_shared_refresh_exact_commit_preserves_work_and_active_missions() {
        use bridget_transport::protocol::CandidateProbeRefusal as R;
        let fixture = Fixture::new();
        let root = fixture.0.canonicalize().unwrap();
        let origin = root.join("origin"); fs::create_dir(&origin).unwrap();
        let git = |cwd: &Path, args: &[&str]| {
            let out = std::process::Command::new("git").current_dir(cwd)
                .env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1")
                .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgSign=false"])
                .args(args).output().unwrap();
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
            String::from_utf8(out.stdout).unwrap().trim().to_owned()
        };
        git(&origin, &["init", "-q"]);
        fs::write(origin.join("file"), "old\n").unwrap();
        git(&origin, &["add", "."]); git(&origin, &["commit", "-qm", "old"]);
        let before = git(&origin, &["rev-parse", "HEAD"]);
        let project = root.join("project");
        git(&root, &["clone", "-q", "--no-local", origin.to_str().unwrap(), project.to_str().unwrap()]);
        fs::write(origin.join("file"), "new\n").unwrap();
        git(&origin, &["commit", "-qam", "new"]);
        let expected = git(&origin, &["rev-parse", "HEAD"]);
        let runtime = runtime_fixture(&fixture);
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Ok(true));
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), before, "probe must not move HEAD");
        assert!(!project.join(".git/FETCH_HEAD").exists());
        fs::write(project.join("file"), "human work\n").unwrap();
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceDirty));
        assert!(runtime.refresh_shared_workspace(&project, &expected).is_err());
        assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "human work\n");
        fs::write(project.join("file"), "old\n").unwrap();
        fs::write(project.join("untracked"), "keep\n").unwrap();
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceDirty));
        fs::remove_file(project.join("untracked")).unwrap();
        let busy_fixture = Fixture::new(); let busy = runtime_fixture(&busy_fixture);
        let mut placement = placement(); placement.execution_cwd = project.to_string_lossy().into_owned();
        busy.store.reserve_command(&command(), &"d".repeat(64), &placement, 1000, 100).unwrap();
        assert_eq!(busy.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceBusy));
        assert!(busy.refresh_shared_workspace(&project, &expected).is_err());
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), before);
        assert_eq!(runtime.shared_refresh_allowed(&project, &"a".repeat(40)), Err(R::WorkspaceBaseUnavailable));
        let branch = git(&project, &["symbolic-ref", "HEAD"]);
        runtime.refresh_shared_workspace(&project, &expected).unwrap();
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), expected);
        assert_eq!(git(&project, &["symbolic-ref", "HEAD"]), branch);
        assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "new\n");
        assert!(!project.join(".git/FETCH_HEAD").exists());
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Ok(false));
        assert_eq!(runtime.shared_refresh_allowed(&project, &before), Err(R::WorkspaceDiverged));
        fs::write(project.join("file"), "local commit\n").unwrap();
        git(&project, &["commit", "-qam", "local"]);
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceDiverged));
        assert!(runtime.refresh_shared_workspace(&project, &expected).is_err());
        assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "local commit\n");
        // Les codes distincts traversent le protocole fermé existant.
        for reason in [R::WorkspaceDirty, R::WorkspaceBusy, R::WorkspaceDiverged, R::WorkspaceBaseUnavailable] {
            assert_eq!(serde_json::from_value::<R>(serde_json::to_value(reason).unwrap()).unwrap(), reason);
        }
    }

    #[test]
    fn spec172_linked_checkout_prepare_preserves_main() {
        use bridget_transport::protocol::{ProviderPermissions, WorkspaceRequest, CandidateProbeRefusal as R};
        for (diverged, isolated, refresh, parent_at_project_root, native_claude) in [
            (false, false, false, false, false), (true, false, false, false, false),
            (false, true, false, false, false), (true, true, false, false, false),
            (false, false, true, false, false), (false, true, false, true, false),
            (false, true, false, true, true)
        ] {
            let fixture = Fixture::new();
            let project_fixture = Fixture::new();
            let project = project_fixture.0.canonicalize().unwrap();
            let git = |cwd: &Path, args: &[&str]| {
                let out = std::process::Command::new("git").current_dir(cwd)
                    .env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1")
                    .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "-c", "commit.gpgSign=false", "-c", "core.hooksPath=/dev/null"])
                    .args(args).output().unwrap();
                assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
                String::from_utf8(out.stdout).unwrap().trim().to_owned()
            };
            git(&project, &["init", "-q"]);
            git(&project, &["config", "remote.origin.url", "fixture:linked172"]);
            fs::write(project.join(".gitignore"), ".worktrees/\n").unwrap();
            fs::write(project.join("file"), "base\n").unwrap();
            git(&project, &["add", "."]); git(&project, &["commit", "-qm", "base"]);
            let base = git(&project, &["rev-parse", "HEAD"]);
            let linked = project.join(".worktrees/linked172");
            git(&project, &["worktree", "add", "--detach", linked.to_str().unwrap(), &base]);
            fs::write(linked.join("file"), "linked\n").unwrap();
            git(&linked, &["commit", "-qam", "linked"]);
            let expected = git(&linked, &["rev-parse", "HEAD"]);
            if diverged {
                fs::write(project.join("main-only"), "main\n").unwrap();
                git(&project, &["add", "main-only"]); git(&project, &["commit", "-qm", "main"]);
            }
            let main_head = git(&project, &["rev-parse", "HEAD"]);
            if refresh {
                git(&linked, &["checkout", "--detach", &base]);
                assert_eq!(git(&linked, &["rev-parse", "HEAD"]), base);
                assert_ne!(base, expected);
            }
            let mut runtime = runtime_fixture(&fixture);
            fs::create_dir(fixture.0.join(".claude")).unwrap();
            let agent = if native_claude { "claude" } else { "codex" };
            let protocol = if native_claude { "claude_stream_json" } else { "codex_app_server" };
            let args = if native_claude { vec!["--dangerously-skip-permissions"] } else { vec!["app-server"] };
            write_json(&runtime.config.registry_path, &serde_json::json!({
                "agents":{(agent):{"command":"/bin/echo","args":args,"protocol":protocol,
                    "capabilities":{"execution_paths":[protocol],"models":{"fixture-model":{"efforts":["high"]}}}}},
                "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                    "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":[agent],
                    "mappings":{"linked172":{"source_host":"origin172","source_root":"/origin172",
                        "remote_root":project,"expected_git_identity":"fixture:linked172","mapping_revision":1}}}}
            })).unwrap();
            runtime.source.clear();
            runtime.source.insert("HOME".into(), fixture.0.as_os_str().to_owned());
            runtime.source.insert("PATH".into(), OsString::from("/usr/bin:/bin"));
            let mut identity = command();
            identity.task_id = uuid::Uuid::new_v4().to_string();
            identity.instance_id.clear();
            identity.generation = 0;
            let request = WorkerPrepare { service_tier: None,
                machine_id:"cartae".into(),project_id:"linked172".into(),provisioning_revision:1,mapping_revision:1,
                origin_host:"origin172".into(),origin_cwd:"/origin172/.worktrees/linked172".into(),
                source_root:"/origin172".into(),remote_root:project.to_string_lossy().into_owned(),
                source_root_identity:"fixture:linked172".into(),relative_cwd:".worktrees/linked172".into(),
                base_commit:Some(expected.clone()),agent_type:agent.into(),model:"fixture-model".into(),
                effort:Some("high".into()),posture:None,workspace:Some(if isolated {
                    WorkspaceRequest::Worktree { base_ref:Some(expected.clone()), branch:None }
                } else { WorkspaceRequest::Shared {} }),
                parent:ProviderPermissions {
                    version:1,source:"native_wrapper".into(),run_id:"run172".into(),
                    provider_session_id:"session172".into(),provider_instance_id:"parent172".into(),revision:1,
                    driver:"codex_app_server".into(),cwd:if parent_at_project_root {
                        "/origin172".into()
                    } else { "/origin172/.worktrees/linked172".into() },
                    runtime_mode:"full-access".into(),interaction_mode:"default".into(),
                    provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user",
                        "sandbox_policy":if native_claude { serde_json::json!({"type":"dangerFullAccess"}) }
                        else { serde_json::json!({"type":"readOnly","networkAccess":false}) }}),
                },
            };
            let registry = runtime.registry().unwrap();
            let (mapping, cwd) = runtime.mapping_for(&request, &registry, None).unwrap();
            assert_eq!(runtime.probe_workspace_preconditions(&request, &mapping, &cwd), Ok(()));
            if parent_at_project_root {
                let mut outside = request.clone();
                outside.parent.cwd = "/origin172-sibling".into();
                assert!(matches!(runtime.prepare(&identity, &outside),
                    Err(WorkerRefusal::PermissionsChanged)));
                assert!(!linked.join(".worktrees").exists());
            }
            let ready = runtime.prepare(&identity, &request).unwrap();
            if native_claude {
                assert_eq!(ready.child_policy["permission_mode"], "bypassPermissions");
            } else {
                assert_eq!(ready.child_policy["sandbox_policy"], request.parent.provider_policy["sandbox_policy"]);
            }
            if isolated {
                let record: crate::delegation::WorkspaceRecord =
                    serde_json::from_value(ready.workspace.clone().unwrap()).unwrap();
                assert_eq!(ready.placement.execution_cwd, record.path);
                assert_eq!(fs::read_to_string(Path::new(&record.path).join("file")).unwrap(), "linked\n");
            } else { assert_eq!(Path::new(&ready.placement.execution_cwd), linked); }
            runtime.recheck(runtime.prepared.get(&command().command_id).unwrap()).unwrap();
            assert_eq!(git(&project, &["rev-parse", "HEAD"]), main_head, "main must not move");
            assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "base\n");
            assert_eq!(git(&linked, &["rev-parse", "HEAD"]), expected);
            if !isolated {
                let mut stale = request.clone(); stale.base_commit = Some(base);
                assert_eq!(runtime.probe_workspace_preconditions(&stale, &mapping, &cwd), Err(R::WorkspaceDiverged));
            }
            assert!(runtime.store.commands().unwrap().is_empty());
            assert!(runtime.running.is_empty() && runtime.held.is_empty());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn spec221_mapping_change_keeps_report_but_denies_old_authority() {
        use bridget_transport::protocol::{WorkspaceRequest, SpawnPosture};
        use std::os::unix::process::CommandExt;
        let fixture=Fixture::new();
        let project_fixture=Fixture::new();
        let project=project_fixture.0.join("placement-a221");
        fs::create_dir(&project).unwrap();
        let git=|cwd:&Path,args:&[&str]| {
            let out=std::process::Command::new("git").arg("-C").arg(cwd).args(args)
                .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
                .env("GIT_AUTHOR_NAME","Fixture").env("GIT_COMMITTER_NAME","Fixture")
                .env("GIT_AUTHOR_EMAIL","fixture@example.invalid")
                .env("GIT_COMMITTER_EMAIL","fixture@example.invalid").output().unwrap();
            assert!(out.status.success(),"{args:?}: {}",String::from_utf8_lossy(&out.stderr));
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        };
        git(&project,&["init","-q"]);
        git(&project,&["config","remote.origin.url","fixture:placement221"]);
        fs::write(project.join(".gitignore"),".worktrees/\n").unwrap();
        fs::write(project.join("file"),"placement conservé\n").unwrap();
        git(&project,&["add","."]);git(&project,&["commit","-qm","fixture placement"]);
        let base=git(&project,&["rev-parse","HEAD"]);
        let mut runtime=runtime_fixture(&fixture);
        let registry=serde_json::json!({
            "agents":{"codex":{"command":"/bin/echo","args":["app-server"],
                "protocol":"codex_app_server","forbidden_env":[],"capabilities":{"execution_paths":["codex_app_server"],
                    "models":{"fixture-model":{"efforts":["high"]}}}}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],
                "mappings":{"placement221":{"source_host":"origin221","source_root":"/origin221",
                    "remote_root":project,"expected_git_identity":"fixture:placement221","mapping_revision":1}}}}
        });
        write_json(&runtime.config.registry_path,&registry).unwrap();
        runtime.source.clear();
        runtime.source.insert("HOME".into(),fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(),"/usr/bin:/bin".into());
        let mut identity=command();identity.task_id=uuid::Uuid::new_v4().to_string();
        let mut prepare_identity=identity.clone();prepare_identity.instance_id.clear();prepare_identity.generation=0;
        let request=WorkerPrepare { service_tier: None,
            machine_id:"cartae".into(),project_id:"placement221".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin221".into(),origin_cwd:"/origin221".into(),source_root:"/origin221".into(),
            remote_root:project.to_string_lossy().into_owned(),source_root_identity:"fixture:placement221".into(),
            relative_cwd:".".into(),base_commit:Some(base.clone()),agent_type:"codex".into(),
            model:"fixture-model".into(),effort:Some("high".into()),posture:Some(SpawnPosture::Development),
            workspace:Some(WorkspaceRequest::Worktree {base_ref:None,branch:None}),
            parent:ProviderPermissions {version:1,source:"native_wrapper".into(),run_id:"run221".into(),
                provider_session_id:"session221".into(),provider_instance_id:"parent221".into(),revision:1,
                driver:"codex_app_server".into(),cwd:"/origin221".into(),runtime_mode:"full-access".into(),
                interaction_mode:"default".into(),provider_policy:serde_json::json!({"kind":"codex",
                    "approval_policy":"never","approvals_reviewer":"user",
                    "sandbox_policy":{"type":"workspaceWrite","writableRoots":["/origin221"],
                        "networkAccess":false,"excludeTmpdirEnvVar":true,"excludeSlashTmp":true}})}
        };
        let prepared=runtime.prepare(&prepare_identity,&request).unwrap();
        let original:crate::delegation::WorkspaceRecord=serde_json::from_value(prepared.workspace.clone().unwrap()).unwrap();
        runtime.store.reserve_command(&identity,&"c".repeat(64),&prepared.placement,worker_now()+600,worker_now()).unwrap();
        runtime.store.bind_preparation(&identity,runtime.prepared.get(&identity.command_id).unwrap()).unwrap();
        // Le processus privé simule un reçu déjà Released. Aucun fournisseur n'est lancé.
        let mut child=std::process::Command::new("/bin/cat").stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).process_group(0).spawn().unwrap();
        let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut receipt=runtime.store.command(&identity).unwrap();
            receipt.phase=WorkerCommandPhase::Released;receipt.native_ready=true;
            receipt.marker=Some(ManagedMarker {pgid:child.id(),birth:crate::managed_process::process_birth(child.id()).unwrap(),
                command_id:identity.command_id.clone(),instance_id:identity.instance_id.clone(),generation:identity.generation});
            runtime.store.persist(&runtime.store.command_path(&identity),&receipt).unwrap();
            let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
                else {panic!("observation attendue")};
            assert!(observation.permission_digest.is_some());
            let worktree=Path::new(&original.path);
            let source_index=fs::read(project.join(".git/index")).unwrap();
            let worktree_head=git(worktree,&["rev-parse","HEAD"]);
            // Ajouter B sous une autre clé ne remplace jamais l'autorité A.
            let mut with_future=registry.clone();
            let future=project_fixture.0.join("placement-b221");
            let mut mapping_b=with_future["machines"]["cartae"]["mappings"]["placement221"].clone();
            mapping_b["remote_root"]=serde_json::json!(future);
            with_future["machines"]["cartae"]["mappings"]["placement-b221"]=mapping_b;
            // Une préférence future malformée ne doit pas retirer le permis A.
            with_future["project_preferences"]=serde_json::json!([]);
            write_json(&runtime.config.registry_path,&with_future).unwrap();
            let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
                else {panic!("rapport A attendu avec B déclaré")};
            assert!(observation.permission_digest.is_some());
            let retained=runtime.prepared.get(&identity.command_id).unwrap();
            for before_release in [false,true] {
                assert!(runtime.read_view().recheck_at_with_repair(retained,before_release,None,
                    &read::ReadTrace::default()).is_ok(),"A garde son autorité propre");
            }
            let mut future_request=request.clone();
            future_request.project_id="placement-b221".into();
            future_request.remote_root=future.to_string_lossy().into_owned();
            assert_eq!(runtime.mapping_authority(&future_request,&runtime.registry().unwrap()).unwrap().remote_root,future);
            assert!(!future.exists(),"une déclaration ne prépare pas le dossier B");
            let slot=slot_174(&identity);
            let frame=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"slot-a-b221".into(),
                connection_epoch:"epoch221".into(),worker:runtime.identity.clone(),command:identity.clone(),
                request:WorkerRequestOperation::ReserveResultSlot {slot:slot.clone()}};
            let disconnected=AtomicBool::new(false);
            assert!(matches!(runtime.handle(&frame,"epoch221",&disconnected).unwrap(),
                WorkerResponseOperation::ResultSlotReserved {..}));
            let reservation=runtime.store.reserve_result_slot(&slot,worker_now()).unwrap();
            // Revenir à la configuration nominale avant la contre-épreuve d'édition brute.
            write_json(&runtime.config.registry_path,&registry).unwrap();
            let stored=snapshot188(&runtime.config.state_root);
            let saved_registry=fs::read(&runtime.config.registry_path).unwrap();
            let future=project_fixture.0.join("placement-b221");
            let mut changed=registry.clone();
            changed["machines"]["cartae"]["mappings"]["placement221"]["remote_root"]=serde_json::json!(future);
            changed["machines"]["cartae"]["mappings"]["placement221"]["mapping_revision"]=serde_json::json!(2);
            write_json(&runtime.config.registry_path,&changed).unwrap();
            let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
                else {panic!("rapport historique attendu")};
            assert_eq!(observation.workspace.as_ref().unwrap()["path"],original.path);
            assert!(observation.permission_digest.is_none(),"une édition du registre ne vaut pas autorité historique");
            let retained=runtime.prepared.get(&identity.command_id).unwrap();
            for before_release in [false,true] {
                assert!(matches!(runtime.read_view().recheck_at_with_repair(retained,before_release,None,
                    &read::ReadTrace::default()),Err(WorkerRefusal::PlacementMismatch)));
            }
            let mut frame=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"slot221".into(),
                connection_epoch:"epoch221".into(),worker:runtime.identity.clone(),command:identity.clone(),
                request:WorkerRequestOperation::ReserveResultSlot {slot:slot.clone()}};
            let disconnected=AtomicBool::new(false);
            let WorkerResponseOperation::ResultSlotReserved {slot_id,..}=runtime.handle(&frame,"epoch221",&disconnected).unwrap()
                else {panic!("le reçu déjà réservé reste relisible")};
            assert_eq!(slot_id,reservation.slot_id);
            let mut next_slot=slot.clone();next_slot.identity.round=2;next_slot.identity.execution_generation+=1;
            frame.request=WorkerRequestOperation::ReserveResultSlot {slot:next_slot};
            assert_eq!(runtime.handle(&frame,"epoch221",&disconnected),Err(WorkerRefusal::PlacementMismatch));
            assert_eq!(snapshot188(&runtime.config.state_root),stored,"pas de reçu ou slot inventé");
            let mut future_request=request.clone();
            future_request.mapping_revision=2;future_request.remote_root=future.to_string_lossy().into_owned();
            assert_eq!(runtime.mapping_authority(&future_request,&runtime.registry().unwrap()).unwrap().remote_root,future);
            assert!(!future.exists(),"une déclaration n'a créé aucun dossier");
            let config=WorkerRuntimeConfig {
                machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
                provisioning_revision:runtime.config.provisioning_revision,
                credential_path:runtime.config.credential_path.clone(),state_root:runtime.config.state_root.clone(),
                registry_path:runtime.config.registry_path.clone(),master_socket:runtime.config.master_socket.clone(),
            };
            drop(runtime);
            let mut reopened=WorkerRuntime::new(config).unwrap();
            let WorkerResponseOperation::Observation {observation}=reopened.reconcile_attested(&identity).unwrap()
                else {panic!("rapport après réouverture attendu")};
            assert!(observation.workspace.is_some() && observation.permission_digest.is_none());
            assert_eq!(reopened.store.read_slot(&slot.identity).unwrap().unwrap().slot,slot);
            fs::write(&reopened.config.registry_path,&saved_registry).unwrap();
            let WorkerResponseOperation::Observation {observation}=reopened.reconcile_attested(&identity).unwrap()
                else {panic!("retour de la configuration initiale attendu")};
            assert!(observation.permission_digest.is_some());
            let mut revoked:serde_json::Value=serde_json::from_slice(&saved_registry).unwrap();
            revoked["machines"]["cartae"]["enabled"]=serde_json::json!(false);
            write_json(&reopened.config.registry_path,&revoked).unwrap();
            // registry() atteste l'identité du worker ; le permis est contrôlé ensuite.
            let WorkerResponseOperation::Observation {observation}=reopened.reconcile_attested(&identity).unwrap()
                else {panic!("observation technique attendue après révocation")};
            assert!(observation.permission_digest.is_none(),
                "le reçu historique ne doit pas effacer une révocation courante");
            assert!(observation.workspace.is_some(),"le rapport technique reste accessible sans autoriser une remise");
            fs::write(&reopened.config.registry_path,&saved_registry).unwrap();
            assert_eq!(git(&project,&["rev-parse","HEAD"]),base);
            assert_eq!(git(worktree,&["rev-parse","HEAD"]),worktree_head);
            assert_eq!(fs::read(project.join(".git/index")).unwrap(),source_index);
            assert_eq!(fs::read(worktree.join("file")).unwrap(),"placement conservé\n".as_bytes());
        }));
        drop(child.stdin.take());assert!(child.wait().unwrap().success());
        if let Err(error)=outcome {std::panic::resume_unwind(error);}
        // La fin du processus de fixture n'est pas présentée comme une clôture native.
    }

    #[test]
    fn worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications() {
        use bridget_transport::protocol::{ProviderPermissions, SpawnPosture, WorkspaceRequest};
        let fixture = Fixture::new();
        let project_fixture = Fixture::new();
        let project = project_fixture.0.canonicalize().unwrap();
        let git = |cwd: &Path, args: &[&str]| {
            let output = std::process::Command::new("/usr/bin/git")
                .args(args)
                .current_dir(cwd)
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", &fixture.0)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_AUTHOR_NAME", "Fixture")
                .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
                .env("GIT_COMMITTER_NAME", "Fixture")
                .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&project, &["init", "--quiet"]);
        git(
            &project,
            &[
                "config",
                "--local",
                "remote.origin.url",
                "fixture:remote162",
            ],
        );
        fs::write(project.join(".gitignore"), b".worktrees/\n").unwrap();
        fs::write(project.join("base.txt"), b"base\n").unwrap();
        git(&project, &["add", ".gitignore", "base.txt"]);
        git(&project, &["commit", "--quiet", "-m", "fixture initiale"]);
        let base = git(&project, &["rev-parse", "HEAD"]);
        let mut runtime = runtime_fixture(&fixture);
        write_json(&runtime.config.registry_path, &serde_json::json!({
            "agents":{"codex":{"command":"/bin/echo","args":["app-server"],"protocol":"codex_app_server",
                "capabilities":{"execution_paths":["codex_app_server"],"models":{"fixture-model":{"efforts":["high"]}}}}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],
                "mappings":{"project162":{"source_host":"origin162","source_root":"/origin162",
                    "remote_root":project,"expected_git_identity":"fixture:remote162","mapping_revision":1}}}}
        })).unwrap();
        // L'origine est volontairement absente sur le worker.
        runtime.source.clear();
        runtime
            .source
            .insert("HOME".into(), fixture.0.as_os_str().to_owned());
        runtime
            .source
            .insert("PATH".into(), OsString::from("/usr/bin:/bin"));
        let mut identity = command();
        // L'isolation154 dérive son chemin depuis le UUID natif de Task.
        identity.task_id = uuid::Uuid::new_v4().to_string();
        identity.instance_id.clear();
        identity.generation = 0;
        let mut request = WorkerPrepare { service_tier: None,
            machine_id: "cartae".into(),
            project_id: "project162".into(),
            provisioning_revision: 1,
            mapping_revision: 1,
            origin_host: "origin162".into(),
            origin_cwd: "/origin162".into(),
            source_root: "/origin162".into(),
            remote_root: project.to_string_lossy().into_owned(),
            source_root_identity: "fixture:remote162".into(),
            relative_cwd: ".".into(),
            base_commit: Some(base.clone()),
            agent_type: "codex".into(),
            model: "fixture-model".into(),
            effort: Some("high".into()),
            posture: None,
            workspace: None,
            parent: ProviderPermissions {
                version: 1,
                source: "native_wrapper".into(),
                run_id: "run162".into(),
                provider_session_id: "session162".into(),
                provider_instance_id: "parent162".into(),
                revision: 1,
                driver: "codex_app_server".into(),
                cwd: "/origin162".into(),
                runtime_mode: "full-access".into(),
                interaction_mode: "default".into(),
                provider_policy: serde_json::json!({"kind":"codex",
                    "approval_policy":"never","approvals_reviewer":"user",
                    "sandbox_policy":{"type":"readOnly","networkAccess":false}}),
            },
        };
        //210 : vrai refus du profil compact, même handler et aucun fournisseur.
        {
            use bridget_transport::protocol::{WorkspacePrepareCause as C, WorkspacePrepareStage as S};
            git(&project, &["config","bridget.worktreeProfile","invalid210"]);
            let mut inner=request.clone();
            inner.workspace=Some(WorkspaceRequest::Worktree {base_ref:Some(base.clone()),branch:Some("fixture210".into())});
            let mut isolated=identity.clone();
            isolated.task_id=uuid::Uuid::new_v4().to_string(); isolated.command_id=isolated.task_id.clone();
            let frame=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"diagnosed210".into(),
                connection_epoch:"epoch210".into(),worker:worker(),command:isolated.clone(),
                request:WorkerRequestOperation::PrepareDiagnosed {prepare:Box::new(
                    WorkerRequestOperation::Prepare {prepare:inner.clone()})}};
            let disconnected=AtomicBool::new(false);
            let response=runtime.handle(&frame,"epoch210",&disconnected).unwrap();
            assert!(matches!(response,WorkerResponseOperation::WorkspaceRejected {
                reason:WorkerRefusal::WorkspacePrepareFailed,
                failure:bridget_transport::protocol::WorkspacePrepareFailure {stage:S::CompactPlan,cause:C::RepoUnsupported}
            }));
            let response_frame=WorkerResponse {version:REMOTE_WORKER_VERSION,correlation_id:frame.correlation_id.clone(),
                connection_epoch:"epoch210".into(),worker:worker(),command:isolated.clone(),response};
            response_frame.validate().unwrap();
            let wire=serde_json::to_string(&response_frame).unwrap();
            assert!(!wire.contains(project.to_str().unwrap()));
            assert!(!wire.contains("invalid210"));
            assert!(!project.join(".worktrees").join(&isolated.task_id[..8]).exists());
            let mut legacy=frame.clone();
            legacy.command.task_id=uuid::Uuid::new_v4().to_string(); legacy.command.command_id=legacy.command.task_id.clone();
            legacy.request=WorkerRequestOperation::Prepare {prepare:inner};
            assert_eq!(runtime.handle(&legacy,"epoch210",&disconnected),Err(WorkerRefusal::WorkspacePrepareFailed));
            assert!(crate::workspace_failure210::take().is_some());
            let mut stale=frame.clone();stale.connection_epoch="foreign".into();
            assert_eq!(runtime.handle(&stale,"epoch210",&disconnected),Err(WorkerRefusal::StaleConnection));
            assert!(runtime.running.is_empty() && runtime.held.is_empty());
            git(&project, &["config","--unset","bridget.worktreeProfile"]);
        }
        let root_prepared = runtime.prepare(&identity, &request).unwrap();
        assert!(root_prepared.workspace.is_none());
        assert_eq!(root_prepared.effective_posture, SpawnPosture::Discovery);
        assert_eq!(Path::new(&root_prepared.placement.execution_cwd), project);
        // Quota du store avant reçu : le même token reste utilisable après
        // retour de capacité. Aucun reçu ni processus n'est né.
        let quota_file = runtime.store.root.join("quota198");
        let quota = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(&quota_file).unwrap();
        quota.set_len(MAX_WORKER_STORE_BYTES as u64).unwrap();
        let mut quota_command = identity.clone();
        quota_command.instance_id = "instance-quota198".into();
        quota_command.generation = 1;
        let quota_start = WorkerStart {
            prepared_token: root_prepared.token.clone(), child_id: "child-quota198".into(),
            mission_id: "mission-quota198".into(), forward_child_requests: false,
            placement: root_prepared.placement.clone(), definition: root_prepared.definition.clone(),
            child_policy: root_prepared.child_policy.clone(),
            effective_posture: root_prepared.effective_posture, deadline_at: worker_now() + 300,
        };
        let mut capacity_cause = WorkerStartCapacityCause::Unknown;
        assert_eq!(runtime.start(&quota_command, &quota_start, &AtomicBool::new(false),
            &mut capacity_cause).unwrap_err(), WorkerRefusal::ResourceLimit);
        assert_eq!(capacity_cause, WorkerStartCapacityCause::CommandStore);
        assert_eq!(runtime.prepared.get(&identity.command_id).unwrap().prepared.token, root_prepared.token);
        assert!(!runtime.store.command_path(&quota_command).exists());
        assert!(runtime.running.is_empty() && runtime.held.is_empty() && runtime.abandoned.is_empty());
        fs::remove_file(quota_file).unwrap();
        let source_status = git(&project, &["status", "--porcelain=v1"]);
        let mut previous_shared = identity.clone();
        previous_shared.command_id = "previous-shared-198".into();
        previous_shared.instance_id = "old-shared-198".into();
        previous_shared.generation = 1;
        let mut next_shared = identity.clone();
        next_shared.command_id = "next-shared-198".into();
        let mut shared_request = request.clone();
        shared_request.workspace = Some(WorkspaceRequest::Shared {});
        shared_request.parent.revision += 1;
        shared_request.parent.run_id = "new-shared-run-198".into();
        runtime.fence_unstarted(&previous_shared, &root_prepared.placement,
            worker_now() - 1).unwrap();
        let shared = runtime.prepare_fenced(&next_shared, &previous_shared,
            &shared_request, &root_prepared.placement, None).unwrap();
        assert_eq!(shared.placement, root_prepared.placement);
        assert!(shared.workspace.is_none());
        assert!(!runtime.store.workspace_path(&next_shared).exists());
        assert_eq!(git(&project, &["status", "--porcelain=v1"]), source_status);
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), base);
        let mut changed_shared = shared_request.clone();
        changed_shared.base_commit = Some("0".repeat(40));
        assert_eq!(runtime.prepare_fenced(&next_shared, &previous_shared,
            &changed_shared, &root_prepared.placement, None).unwrap_err(),
            WorkerRefusal::PlacementMismatch);
        runtime.recheck(runtime.prepared.get(&next_shared.command_id).unwrap()).unwrap();
        runtime.prepared.remove(&next_shared.command_id);
        fs::remove_file(runtime.store.command_path(&previous_shared)).unwrap();
        request.workspace = Some(WorkspaceRequest::Worktree {
            base_ref: None,
            branch: None,
        });
        let prepared = runtime.prepare(&identity, &request).unwrap();
        let record: crate::delegation::WorkspaceRecord =
            serde_json::from_value(prepared.workspace.clone().unwrap()).unwrap();
        assert_ne!(Path::new(&record.path), project);
        assert_eq!(prepared.placement.execution_cwd, record.path);
        assert_eq!(record.base_commit, base);
        assert_eq!(
            fs::read(Path::new(&record.path).join("base.txt")).unwrap(),
            b"base\n"
        );
        let before = git(&project, &["worktree", "list", "--porcelain"]);
        let replay = runtime.prepare(&identity, &request).unwrap();
        assert_eq!(replay.workspace, prepared.workspace);
        assert_eq!(git(&project, &["worktree", "list", "--porcelain"]), before);
        let original_receipt = fs::read(runtime.store.workspace_path(&identity)).unwrap();
        let worktree_status = git(Path::new(&record.path), &["status", "--porcelain=v1"]);
        let mut previous = identity.clone();
        previous.instance_id = "previous-198".into();
        previous.generation = 1;
        let mut next = identity.clone();
        next.command_id = "fenced-prepare-198".into();
        let mut next_request = request.clone();
        next_request.parent.revision += 1;
        next_request.parent.run_id = "new-run-198".into();
        next_request.parent.source = "provider_turn".into();
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::UnknownExecution);
        let old_now = worker_now();
        runtime.store.reserve_command(&previous, &"a".repeat(64), &prepared.placement,
            old_now + 300, old_now).unwrap();
        runtime.store.set_phase(&previous, WorkerCommandPhase::Refused,
            old_now + 1).unwrap();
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::UnknownExecution);
        assert!(!runtime.store.workspace_path(&next).exists());
        fs::remove_file(runtime.store.command_path(&previous)).unwrap();
        runtime.fence_unstarted(&previous, &prepared.placement, worker_now() + 300).unwrap();
        let mut other_placement = prepared.placement.clone();
        other_placement.mapping_revision += 1;
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &other_placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        let mut other_workspace = prepared.workspace.clone().unwrap();
        other_workspace["branch"] = serde_json::json!("other-branch");
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, Some(&other_workspace)).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        let mut changed_mapping = next_request.clone();
        changed_mapping.mapping_revision += 1;
        assert_eq!(runtime.prepare_fenced(&next, &previous, &changed_mapping,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::PlacementMismatch);
        let mut gained_rights = next_request.clone();
        gained_rights.parent.provider_policy["sandbox_policy"]["networkAccess"] =
            serde_json::json!(true);
        assert!(runtime.prepare_fenced(&next, &previous, &gained_rights,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        assert!(!runtime.store.workspace_path(&next).exists());
        assert_eq!(git(&project, &["worktree", "list", "--porcelain"]), before);
        let mut volatile_workspace = prepared.workspace.clone().unwrap();
        volatile_workspace["dirty"] = serde_json::json!(true);
        volatile_workspace["state"] = serde_json::json!("kept");
        // Sélection168 d'un reçu PrepareV2 : candidat absent ou désactivé refuse avant tout lien.
        let disabled = bridget_transport::protocol::WorkerFencedRouting {
            workload: AutoDelegateWorkload::General,
            connection_id: "connexion-desactivee-198".into(),
            engine: "codex".into(),
            expected_definition_digest: prepared.definition.digest.clone(),
            expected_permission_digest: prepared.placement.permission_digest.clone(),
        };
        assert_eq!(runtime.prepare_fenced_routed(&next, &previous, &next_request,
            &prepared.placement, Some(&volatile_workspace), Some(&disabled)).unwrap_err(),
            WorkerRefusal::CapabilityRequired);
        assert!(!runtime.store.workspace_path(&next).exists());
        assert!(!proof198::bind_path(&runtime.store.root, &next).exists());
        let fresh = runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, Some(&volatile_workspace)).unwrap();
        assert_eq!(fresh.placement, prepared.placement);
        assert_eq!(fresh.workspace, prepared.workspace);
        assert_ne!(fresh.token, prepared.token);
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap().token, fresh.token);
        // ACK Prepare perdu puis nouveau tour équivalent : même preuve, mêmes droits recontrôlés.
        let mut later_turn = next_request.clone();
        later_turn.parent.revision += 1;
        later_turn.parent.run_id = "later-run-198".into();
        later_turn.parent.provider_session_id = "later-session-198".into();
        let replay = runtime.prepare_fenced(&next, &previous, &later_turn,
            &prepared.placement, prepared.workspace.as_ref());
        assert_eq!(replay.unwrap().token, fresh.token);
        assert_eq!(runtime.prepared.get(&next.command_id).unwrap().request.parent,later_turn.parent,
            "Release doit employer la révision actuelle, même quand le token reste identique");
        let proof_before=fs::read(proof198::bind_path(&runtime.store.root,&next)).unwrap();
        let restart_config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,credential_path:runtime.config.credential_path.clone(),
            state_root:runtime.config.state_root.clone(),registry_path:runtime.config.registry_path.clone(),
            master_socket:runtime.config.master_socket.clone()};
        drop(runtime);
        let mut runtime=WorkerRuntime::new(restart_config).unwrap();
        runtime.source.clear();runtime.source.insert("HOME".into(),fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(),OsString::from("/usr/bin:/bin"));
        let fresh=runtime.prepare_fenced(&next,&previous,&later_turn,&prepared.placement,
            prepared.workspace.as_ref()).unwrap();
        assert_eq!(fs::read(proof198::bind_path(&runtime.store.root,&next)).unwrap(),proof_before,
            "ACK perdu puis redémarrage et parent renouvelé : canon durable inchangé");
        let mut other_cwd = next_request.clone();
        other_cwd.parent.cwd.push_str("-autre");
        assert!(runtime.prepare_fenced(&next, &previous, &other_cwd,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        let mut other_driver = next_request.clone();
        other_driver.parent.driver.push_str("-autre");
        assert!(runtime.prepare_fenced(&next, &previous, &other_driver,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        let mut other_policy = next_request.clone();
        other_policy.parent.provider_policy["sandbox_policy"]["networkAccess"] =
            serde_json::json!(true);
        assert!(runtime.prepare_fenced(&next, &previous, &other_policy,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        let bound = proof198::read_workspace_receipt(&runtime.store.root, &runtime.store.worker, &next)
            .unwrap().unwrap();
        assert_eq!(bound.fenced_previous, Some(previous.clone()));
        assert!(bound.record.is_none(), "la nouvelle préparation ne copie pas l'ancre Git");
        let raw: serde_json::Value = serde_json::from_slice(
            &fs::read(runtime.store.workspace_path(&next)).unwrap()).unwrap();
        assert!(raw.get("fenced_previous").is_none(), "reçu workspace strictement au format183");
        assert!(proof198::bind_path(&runtime.store.root, &next).exists());
        assert!(runtime.store.inventory().is_ok());
        assert!(assert_legacy183_readable(&runtime.store.root) >= 1);
        runtime.recheck(runtime.prepared.get(&next.command_id).unwrap()).unwrap();
        let mut launched = next.clone();
        launched.instance_id = "new-instance-198".into();
        launched.generation = 2;
        let start = WorkerStart {
            prepared_token: fresh.token.clone(), child_id: "child-198".into(),
            mission_id: "mission-198".into(), forward_child_requests: false,
            placement: fresh.placement.clone(), definition: fresh.definition.clone(),
            child_policy: fresh.child_policy.clone(), effective_posture: fresh.effective_posture,
            deadline_at: worker_now() + 300,
        };
        let digest = worker_command_digest(&runtime.identity, &launched, &start).unwrap();
        runtime.store.reserve_command(&launched, &digest, &fresh.placement,
            start.deadline_at, worker_now()).unwrap();
        runtime.store.bind_preparation(&launched,
            runtime.prepared.get(&next.command_id).unwrap()).unwrap();
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut runtime = WorkerRuntime::new(config).unwrap();
        runtime.source.clear();
        runtime.source.insert("HOME".into(), fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(), OsString::from("/usr/bin:/bin"));
        let persisted = runtime.store.command(&launched).unwrap();
        let (persisted_request, persisted_prepared) = persisted.preparation.unwrap();
        assert_eq!(persisted_request.parent.run_id, "later-run-198");
        let reconstructed = PreparedRuntime {
            routing: persisted.routing, command: next.clone(), request: persisted_request,
            prepared: persisted_prepared, expires: Instant::now() + Duration::from_secs(30),
        };
        runtime.recheck(&reconstructed).unwrap();
        assert_eq!(fs::read(runtime.store.workspace_path(&identity)).unwrap(), original_receipt);
        assert_eq!(git(&project, &["worktree", "list", "--porcelain"]), before);
        assert_eq!(git(Path::new(&record.path), &["status", "--porcelain=v1"]), worktree_status);
        assert_eq!(fs::read(Path::new(&record.path).join("base.txt")).unwrap(), b"base\n");
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        // Le reste de ce test historique conserve le Prepared initial.
        runtime.prepared.insert(identity.command_id.clone(), PreparedRuntime {
            routing: None, command: identity.clone(), request: request.clone(),
            prepared: prepared.clone(), expires: Instant::now() + Duration::from_secs(30),
        });
        let retained = runtime.prepared.get(&identity.command_id).unwrap();
        runtime.recheck(retained).unwrap();
        fs::write(
            Path::new(&record.path).join("travail.txt"),
            b"travail a conserver\n",
        )
        .unwrap();
        git(Path::new(&record.path), &["add", "travail.txt"]);
        git(
            Path::new(&record.path),
            &["commit", "--quiet", "-m", "fixture travail"],
        );
        runtime.recheck_at(retained, false).unwrap();
        assert_eq!(
            runtime.recheck(retained).unwrap_err(),
            WorkerRefusal::PlacementMismatch
        );
        let report = crate::workspace_git::report(&record).unwrap();
        assert_eq!(report.commit_count, Some(1));
        assert_eq!(report.dirty, Some(false));
        assert_ne!(report.head_commit.as_ref(), Some(&record.base_commit));
        assert!(!project.join("travail.txt").exists());
        let after_work = git(&project, &["worktree", "list", "--porcelain"]);
        assert!(runtime.prepare(&identity, &request).is_err());
        assert_eq!(
            fs::read(Path::new(&record.path).join("travail.txt")).unwrap(),
            b"travail a conserver\n"
        );
        assert_eq!(
            git(&project, &["worktree", "list", "--porcelain"]),
            after_work
        );
        let mut later = next.clone();
        later.command_id = "later-after-head-198".into();
        assert_eq!(runtime.prepare_fenced(&later, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::PlacementMismatch);
        assert!(!runtime.store.workspace_path(&later).exists());
        fs::remove_file(runtime.store.command_path(&previous)).unwrap();
        runtime.recheck_at(&reconstructed, false).unwrap();
        fs::remove_file(runtime.store.command_path(&launched)).unwrap();
        fs::remove_file(runtime.store.workspace_path(&next)).unwrap();
        assert!(runtime.store.commands().unwrap().is_empty());
        assert!(runtime.running.is_empty() && runtime.held.is_empty());
        // Le contrat réseau accepte 256 octets ; le bootstrap historique en
        // accepte 200. Ce refus réel intervient avant spawn, sans injection.
        let mut refused_command = identity.clone();
        refused_command.command_id = "refused-before-birth162".into();
        let mut root_request = request.clone();
        root_request.workspace = None;
        let ready = runtime.prepare(&refused_command, &root_request).unwrap();
        refused_command.instance_id = "i".repeat(201);
        refused_command.generation = 1;
        let start = WorkerStart {
            prepared_token: ready.token,
            child_id: "child162".into(),
            mission_id: "mission162".into(),
            forward_child_requests: false,
            placement: ready.placement,
            definition: ready.definition,
            child_policy: ready.child_policy,
            effective_posture: ready.effective_posture,
            deadline_at: worker_now() + 300,
        };
        assert_eq!(
            runtime
                .start(&refused_command, &start, &AtomicBool::new(false), &mut WorkerStartCapacityCause::Unknown)
                .unwrap_err(),
            WorkerRefusal::StoreUnavailable
        );
        assert_eq!(
            runtime.store.command(&refused_command).unwrap().phase,
            WorkerCommandPhase::Refused
        );
        assert!(
            runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty()
        );
        assert!(matches!(
            runtime.reconcile(&refused_command).unwrap(),
            WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase: WorkerCommandPhase::Refused,
                    ..
                }
            }
        ));
        let mut registry: serde_json::Value =
            serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap()).unwrap();
        registry["machines"]["cartae"]["enabled"] = false.into();
        write_json(&runtime.config.registry_path, &registry).unwrap();
        assert!(runtime.registry().is_ok());
        assert_eq!(
            runtime.prepare(&identity, &request).unwrap_err(),
            WorkerRefusal::PlacementMismatch
        );
    }

    #[test]
    fn worker162_parent_proof_handle_correle_hote_tuple_revision_et_epoch() {
        use bridget_transport::protocol::ProviderPermissions;
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let cwd = fixture.0.canonicalize().unwrap();
        let mut placed = placement();
        placed.execution_cwd = cwd.to_string_lossy().into_owned();
        let now = worker_now();
        runtime
            .store
            .reserve_command(&command(), &"a".repeat(64), &placed, now + 300, now)
            .unwrap();
        // Le reçu représente un parent déjà libéré. Aucun processus n'est créé ici.
        runtime
            .store
            .mark_command(
                &command(),
                &ManagedMarker {
                    pgid: 42,
                    birth: 1,
                    instance_id: command().instance_id,
                    command_id: command().command_id,
                    generation: command().generation,
                },
                now,
            )
            .unwrap();
        runtime
            .store
            .authorize_release(
                &command(),
                &WorkerReleasePermit {
                    permit_id: "parent-proof".into(),
                    command_digest: "a".repeat(64),
                    permission_revision: 1,
                    mapping_revision: 1,
                    provisioning_revision: 1,
                    issued_at: now,
                    expires_at: now + 5,
                },
                "epoch",
                now,
            )
            .unwrap();
        runtime
            .store
            .set_phase(&command(), WorkerCommandPhase::Released, now)
            .unwrap();
        let parent = ProviderPermissions {
            version: 1,
            source: "native_wrapper".into(),
            run_id: "run162".into(),
            provider_session_id: "session162".into(),
            provider_instance_id: command().instance_id,
            revision: 1,
            driver: "codex_app_server".into(),
            cwd: placed.execution_cwd,
            runtime_mode: "full-access".into(),
            interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never",
                "approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
        };
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION,
            correlation_id: "parent-proof162".into(),
            connection_epoch: "epoch".into(),
            worker: worker(),
            command: command(),
            request: WorkerRequestOperation::ParentProof {
                parent: parent.clone(),
                expected_revision: 1,
            },
        };
        let disconnected = AtomicBool::new(false);
        assert_eq!(
            runtime.handle(&request, "epoch", &disconnected).unwrap(),
            WorkerResponseOperation::ParentProof {
                parent: parent.clone(),
                permission_digest: json_digest(&parent).unwrap()
            }
        );
        // Source172 : vrai dépôt sur le worker, jamais lu sur le coordinateur.
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git").arg("-C").arg(&cwd)
                .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgSign=false"])
                .args(args).output().unwrap();
            assert!(output.status.success(), "{output:?}");
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&["init", "-q", "--initial-branch=main"]);
        fs::write(cwd.join("source172"), "base").unwrap();
        git(&["add", "source172"]); git(&["commit", "-qm", "Base172"]);
        let head = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "-qb", "feature172"]);
        fs::write(cwd.join("source172"), "feature").unwrap();
        git(&["commit", "-qam", "Feature172"]);
        let feature = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "-q", "main"]);
        let mut source = bridget_transport::protocol::WorkerSourceBase {
            cwd: parent.cwd.clone(), expected_base_commit: Some(head.clone()),
            workspace: Some(bridget_transport::protocol::WorkspaceRequest::Shared {}),
            workload: bridget_transport::protocol::AutoDelegateWorkload::General,
        };
        let prove = |runtime: &mut WorkerRuntime, source| {
            let mut source_request = request.clone();
            source_request.request = WorkerRequestOperation::ParentSourceProof { parent: parent.clone(), source };
            runtime.handle(&source_request, "epoch", &disconnected).unwrap()
        };
        let valid = WorkerResponseOperation::ParentSourceProof { permission_digest: json_digest(&parent).unwrap(), source_error: None };
        assert_eq!(prove(&mut runtime, source.clone()), valid);
        source.expected_base_commit = Some("b".repeat(40));
        assert!(matches!(prove(&mut runtime, source.clone()), WorkerResponseOperation::ParentSourceProof { source_error: Some(code), .. } if code == "expected_base_commit_mismatch"));
        source.expected_base_commit = None;
        assert!(matches!(prove(&mut runtime, source.clone()), WorkerResponseOperation::ParentSourceProof { source_error: Some(code), .. } if code == "expected_base_commit_required"));
        source.workspace = Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref: Some("feature172".into()), branch: None });
        source.expected_base_commit = Some(feature);
        assert_eq!(prove(&mut runtime, source.clone()), valid);
        source.expected_base_commit = Some(head.clone());
        assert!(matches!(prove(&mut runtime, source), WorkerResponseOperation::ParentSourceProof { source_error: Some(code), .. } if code == "expected_base_commit_mismatch"));
        assert_eq!(git(&["rev-parse", "HEAD"]), head);
        assert!(runtime.running.is_empty() && runtime.held.is_empty());
        let receipt_before = fs::read(runtime.store.command_path(&command())).unwrap();
        let mut stale = request.clone();
        stale.command.generation += 1;
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::StoreUnavailable)
        );
        stale = request.clone();
        stale.connection_epoch = "old".into();
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::StaleConnection)
        );
        stale = request.clone();
        if let WorkerRequestOperation::ParentProof {
            expected_revision, ..
        } = &mut stale.request
        {
            *expected_revision = 2;
        }
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::InvalidEnvelope)
        );
        stale = request.clone();
        if let WorkerRequestOperation::ParentProof { parent, .. } = &mut stale.request {
            parent.cwd = runtime.config.state_root.to_string_lossy().into_owned();
        }
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::Unauthorized)
        );
        assert_eq!(
            fs::read(runtime.store.command_path(&command())).unwrap(),
            receipt_before
        );
        assert!(runtime.running.is_empty() && runtime.held.is_empty());
    }

    #[test]
    fn worker162_runtime_epoch_fence_et_journal_refusent_mauvaise_identite() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        runtime
            .store
            .reserve_command(
                &command(),
                &"a".repeat(64),
                &placement(),
                worker_now() + 300,
                worker_now(),
            )
            .unwrap();
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION,
            correlation_id: "corr".into(),
            connection_epoch: "old".into(),
            worker: worker(),
            command: command(),
            request: WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {},
            },
        };
        assert_eq!(
            runtime
                .handle(&request, "new", &AtomicBool::new(false))
                .unwrap_err(),
            WorkerRefusal::StaleConnection
        );
        assert_eq!(
            runtime.store.command(&command()).unwrap().phase,
            WorkerCommandPhase::Reserved
        );
        let mut hostile = command();
        hostile.instance_id = "../outside".into();
        hostile.command_id = "other".into();
        runtime
            .store
            .reserve_command(
                &hostile,
                &"b".repeat(64),
                &placement(),
                worker_now() + 300,
                worker_now(),
            )
            .unwrap();
        runtime.store.bind_launch(&hostile, "child162").unwrap();
        assert_eq!(
            runtime.journal(&hostile, 0, 1).unwrap_err(),
            WorkerRefusal::JournalUnavailable
        );
        assert!(runtime.running.is_empty());
    }
    #[test]
    fn spec172_workspace_limit_reason_is_closed_and_legacy_safe() {
        use CandidateProbeRefusal as R;
        let reason = workspace_probe_refusal("workspace_limit: private path or stderr");
        assert_eq!(reason, R::WorkspaceLimit);
        assert_eq!(serde_json::to_value(reason).unwrap(), "workspace_limit");
        assert_eq!(workspace_probe_refusal("Git 128: workspace_limit"), R::WorkspaceUnavailable);
        assert_eq!(workspace_probe_reason_for_master(reason, false), R::WorkspaceUnavailable);
        assert_eq!(workspace_probe_reason_for_master(reason, true), R::WorkspaceLimit);
        assert_eq!(workspace_probe_reason_for_master(R::WorkspaceBusy, false), R::WorkspaceBusy);
    }

    #[test]
    fn worker162_maitre_ancien_refuse_avant_hello_et_credential() {
        use std::io::BufRead;
        for supported in [false, true] {
            let (client, server) = UnixStream::pair().unwrap();
            let peer = std::thread::spawn(move || {
                let mut reader = BufReader::new(server.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),
                    WrapperToDaemon::DaemonIdentityRequest
                ));
                let features = if supported {
                    vec!["remote_worker_v1".to_string(), "worker_workspace_diagnostics_v1".to_string()]
                } else {
                    vec![]
                };
                let report = DaemonToWrapper::DaemonIdentityReport {
                    host: "host".into(),
                    db_path: "db".into(),
                    instance_id: "daemon".into(),
                    features,
                };
                let mut server = server;
                server
                    .write_all(
                        bridget_transport::protocol::encode(&report)
                            .unwrap()
                            .as_bytes(),
                    )
                    .unwrap();
                server.write_all(b"\n").unwrap();
                let mut rest = Vec::new();
                reader.read_to_end(&mut rest).unwrap();
                assert!(
                    rest.is_empty(),
                    "aucun WorkerHello/credential avant preuve de capacité"
                );
            });
            let writer = Mutex::new(client.try_clone().unwrap());
            let mut reader = BufReader::new(client);
            let probe = probe_worker_capability(&writer, &mut reader);
            assert_eq!(probe.is_ok(), supported, "sonde: {probe:?}");
            if supported { assert!(probe.unwrap().workspace_diagnostics); }
            drop(writer);
            drop(reader);
            peer.join().unwrap();
        }
    }
    #[cfg(target_os="linux")]
    #[test]
    fn spec185_spec193_worker_repair_publishes_overlay_and_keeps_original_command_and_slots() {
        use bridget_transport::protocol::{WorkspaceRequest,ProviderPermissions,SpawnPosture,WorkspaceRepairRequestV1};
        use std::os::unix::process::CommandExt;
        let fixture=Fixture::new();
        let mut runtime=runtime_fixture(&fixture);
        let project_fixture=Fixture::new();
        let project=project_fixture.0.join("project185");fs::create_dir(&project).unwrap();
        let git=|cwd:&Path,args:&[&str]| {
            let output=std::process::Command::new("git").arg("-C").arg(cwd).args(args)
                .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
                .env("GIT_AUTHOR_NAME","Fixture").env("GIT_COMMITTER_NAME","Fixture")
                .env("GIT_AUTHOR_EMAIL","fixture@example.invalid").env("GIT_COMMITTER_EMAIL","fixture@example.invalid")
                .output().unwrap();
            assert!(output.status.success(),"{args:?}: {}",String::from_utf8_lossy(&output.stderr));
            String::from_utf8(output.stdout).unwrap().trim().to_string()
        };
        git(&project,&["init","-q"]);
        git(&project,&["config","remote.origin.url","fixture:repair185"]);
        fs::write(project.join(".gitignore"),".worktrees/\n").unwrap();
        fs::write(project.join("file"),"conservé\n").unwrap();
        git(&project,&["add","."]);git(&project,&["commit","-qm","base"]);
        let base=git(&project,&["rev-parse","HEAD"]);
        write_json(&runtime.config.registry_path,&serde_json::json!({
            "agents":{"codex":{"command":"/bin/echo","args":["app-server"],"protocol":"codex_app_server",
                "forbidden_env":[],"capabilities":{"execution_paths":["codex_app_server"],
                     "models":{"fixture-model":{"efforts":["high"]}},
                    "observed":{"binary_path":"/bin/echo","binary_version":"fixture185",
                        "binary_digest":digest_bytes(&fs::read("/bin/echo").unwrap()),"contract_version":"codex-app-server-fixture-v1",
                        "operations":["resume"]}}}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],
                "mappings":{"repair185":{"source_host":"origin185","source_root":"/origin185",
                    "remote_root":project,"expected_git_identity":"fixture:repair185","mapping_revision":1}}}}
        })).unwrap();
        runtime.source.clear();
        runtime.source.insert("HOME".into(),fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(),"/usr/bin:/bin".into());
        let mut identity=command();identity.task_id=uuid::Uuid::new_v4().to_string();
        let mut prepare_identity=identity.clone();prepare_identity.instance_id.clear();prepare_identity.generation=0;
        let prepare=WorkerPrepare { service_tier: None,
            machine_id:"cartae".into(),project_id:"repair185".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin185".into(),origin_cwd:"/origin185".into(),source_root:"/origin185".into(),
            remote_root:project.to_string_lossy().into_owned(),source_root_identity:"fixture:repair185".into(),
            relative_cwd:".".into(),base_commit:Some(base.clone()),agent_type:"codex".into(),
            model:"fixture-model".into(),effort:Some("high".into()),posture:Some(SpawnPosture::Development),
            workspace:Some(WorkspaceRequest::Worktree {base_ref:None,branch:None}),
            parent:ProviderPermissions {version:1,source:"native_wrapper".into(),run_id:"run185".into(),
                provider_session_id:"parent-session185".into(),provider_instance_id:"parent185".into(),revision:1,
                driver:"codex_app_server".into(),cwd:"/origin185".into(),runtime_mode:"full-access".into(),
                interaction_mode:"default".into(),provider_policy:serde_json::json!({"kind":"codex",
                    "approval_policy":"never","approvals_reviewer":"user",
                    "sandbox_policy":{"type":"workspaceWrite","writableRoots":["/origin185"],"networkAccess":false,"excludeTmpdirEnvVar":true,"excludeSlashTmp":true}})},
        };
        let prepared=runtime.prepare(&prepare_identity,&prepare).unwrap();
        let original:crate::delegation::WorkspaceRecord=serde_json::from_value(prepared.workspace.clone().unwrap()).unwrap();
        runtime.store.reserve_command(&identity,&"c".repeat(64),&prepared.placement,worker_now()+600,worker_now()).unwrap();
        runtime.store.bind_preparation(&identity,runtime.prepared.get(&identity.command_id).unwrap()).unwrap();
        let mut child=std::process::Command::new("/bin/cat").stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).process_group(0).spawn().unwrap();
        let mut receipt=runtime.store.command(&identity).unwrap();
        receipt.phase=WorkerCommandPhase::Released;receipt.native_ready=true;
        receipt.marker=Some(ManagedMarker {pgid:child.id(),birth:crate::managed_process::process_birth(child.id()).unwrap(),
            command_id:identity.command_id.clone(),instance_id:identity.instance_id.clone(),generation:identity.generation});
        runtime.store.persist(&runtime.store.command_path(&identity),&receipt).unwrap();
        let before188=snapshot188(&runtime.config.state_root);
        let diag188=read::diagnose_observed(runtime.read_view(),&identity,||{}).unwrap();
        assert_eq!(diag188["check"],"passed","{diag188}");
        assert_eq!(diag188["stage"],"complete");
        assert_eq!(snapshot188(&runtime.config.state_root),before188);
        // Mesure du vrai parcours : un seul status Git par réponse, sans cache inter-appels.
        crate::workspace_git::take_status_reads193();
        let measured=Instant::now();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 attendue")};
        let reads=crate::workspace_git::take_status_reads193();
        eprintln!("spec193 normal_status_reads={reads} elapsed_ms={}",measured.elapsed().as_millis());
        assert_eq!(reads,1);
        assert!(observation.permission_digest.is_some());
        assert_eq!(snapshot188(&runtime.config.state_root),before188,"observation sans mutation du store");
        let worktree=PathBuf::from(&original.path);
        fs::write(worktree.join("untracked193"),"changement entre deux appels").unwrap();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 fraîche attendue")};
        assert_eq!(crate::workspace_git::take_status_reads193(),1);
        assert_eq!(observation.workspace.as_ref().unwrap()["dirty"],true);
        fs::remove_file(worktree.join("untracked193")).unwrap();
        let registry_before=fs::read(&runtime.config.registry_path).unwrap();
        let mut changed:serde_json::Value=serde_json::from_slice(&registry_before).unwrap();
        changed["agents"]["codex"]["args"]=serde_json::json!(["app-server","changed193"]);
        write_json(&runtime.config.registry_path,&changed).unwrap();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 refusée attendue")};
        assert_eq!(crate::workspace_git::take_status_reads193(),1);
        assert!(observation.permission_digest.is_none(),"définition modifiée toujours refusée");
        assert!(observation.workspace.is_some(),"le rapport technique reste disponible après refus des droits");
        fs::write(&runtime.config.registry_path,registry_before).unwrap();
        git(&worktree,&["config","filter.blocked193.process","/bin/false"]);
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 Git refusé attendue")};
        assert!(observation.permission_digest.is_none());
        assert!(observation.workspace.is_none(),"aucun ancien rapport après une erreur Git");
        git(&worktree,&["config","--unset","filter.blocked193.process"]);
        let target=project_fixture.0.join("disk185");
        fs::rename(&original.path,&target).unwrap();
        std::os::unix::fs::symlink(&target,&original.path).unwrap();
        let mut request=WorkspaceRepairRequestV1 {request_id:"repair185-worker".into(),
            target_cwd:target.to_string_lossy().into_owned(),expected_head:base};
        let WorkerResponseOperation::WorkspaceRepair {status}=runtime.repair_workspace(&identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("statut attendu")};
        assert_eq!(status.code.as_deref(),Some("workspace_repair_child_upgrade_required"));
        assert_eq!(serde_json::to_value(runtime.store.command(&identity).unwrap()).unwrap(),
            serde_json::to_value(&receipt).unwrap(),"aucun processus ni canon modifié avant capacité");
        private_dir(&runtime.store.root.join("markers")).unwrap();
        write_json(&workspace_repair::capability_path(&runtime.store.root,&identity),
            &workspace_repair::RepairCapability {version:1,worker:runtime.identity.clone(),command:identity.clone(),
                wrapper_pid:child.id(),wrapper_birth:receipt.marker.as_ref().unwrap().birth,thread_id:"thread185".into()}).unwrap();
        let WorkerResponseOperation::WorkspaceRepair {status}=runtime.repair_workspace(&identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("statut attendu")};
        assert_eq!(status.phase,"prepared","{status:?}");
        // Un refus sans effet clôt la tentative ; une nouvelle clé est alors permise.
        let mut refused_status=status.clone();refused_status.phase="refused".into();
        refused_status.code=Some("workspace_repair_busy".into());
        workspace_repair::persist_wrapper_result(&runtime.store.root,&identity,&refused_status).unwrap();
        let WorkerResponseOperation::WorkspaceRepair {status:refused}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("refus durable attendu")};
        assert_eq!(refused.phase,"refused");
        request.request_id="repair185-worker-second".into();
        let WorkerResponseOperation::WorkspaceRepair {status}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("nouvelle préparation attendue")};
        assert_eq!(status.phase,"prepared","{status:?}");
        let retained=runtime.store.command(&identity).unwrap();
        assert_eq!(retained.workspace_repair_history.len(),1);
        assert_eq!(retained.workspace_repair_history[0].phase,"refused");
        let (anchor,source,_)=workspace_repair::proof(&status).unwrap();
        crate::workspace_git::apply_repair_git(&source,&anchor).unwrap();
        // Frontière wrapper simulée : ce test vérifie les contrôles worker.
        // Le test wrapper185 qualifie séparément l'arrêt et la reprise réels.
        let mut resumed=status.clone();resumed.phase="resumed".into();
        workspace_repair::persist_wrapper_result(&runtime.store.root,&identity,&resumed).unwrap();
        let WorkerResponseOperation::WorkspaceRepair {status:observed}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap() else {panic!("reçu observé attendu")};
        assert_eq!(observed.phase,"resumed","un ACK perdu ne rejoue pas le contrôle wrapper");
        assert_eq!(runtime.store.command(&identity).unwrap().workspace_repair.as_ref().unwrap().phase,"prepared",
            "l'observation ne publie aucune liaison");
        let before=runtime.store.command(&identity).unwrap();
        assert_eq!(runtime.repair_workspace(&identity,&request,
            bridget_transport::protocol::WorkspaceRepairActionV1::Commit,worker_now()-1).unwrap_err(),
            WorkerRefusal::PrepareExpired);
        assert_eq!(runtime.store.command(&identity).unwrap(),before,"expiration avant commit sans mutation");
        let WorkerResponseOperation::WorkspaceRepair {status:published}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Commit,worker_now()+60).unwrap() else {panic!("publication attendue")};
        assert_eq!(published.phase,"committed","{published:?}");
        assert!(!workspace_repair::published(&runtime.store.root,&identity,&request));
        let before=runtime.store.command(&identity).unwrap();
        assert_eq!(runtime.repair_workspace(&identity,&request,
            bridget_transport::protocol::WorkspaceRepairActionV1::Publish,worker_now()-1).unwrap_err(),
            WorkerRefusal::PrepareExpired);
        assert_eq!(runtime.store.command(&identity).unwrap(),before,"expiration avant publication sans mutation");
        assert!(!workspace_repair::published(&runtime.store.root,&identity,&request),"barrière conservée");

        let WorkerResponseOperation::WorkspaceRepair {status:published}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Publish,worker_now()+60).unwrap()
            else {panic!("publication attendue")};
        assert_eq!(published.phase,"published","{published:?}");
        let after=runtime.store.command(&identity).unwrap();
        assert_eq!(after.identity,receipt.identity);assert_eq!(after.command_digest,receipt.command_digest);
        assert_eq!(after.placement,receipt.placement);assert_eq!(after.marker,receipt.marker);
        assert_eq!(after.preparation,receipt.preparation);
        assert!(runtime.store.inventory().unwrap().1.is_empty(),"aucun slot métier créé");
        crate::workspace_git::take_status_reads193();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("preuve attendue")};
        assert_eq!(crate::workspace_git::take_status_reads193(),1,"placement réparé sans lectures répétées");
        assert!(observation.native_ready);
        assert_eq!(observation.permission_digest.as_ref(),
            published.prepared.as_ref().map(|v|&v.placement.permission_digest));
        assert_eq!(observation.workspace.as_ref().unwrap()["path"],target.to_string_lossy().as_ref());
        fs::write(target.join("next-round"),"nouveau travail conservé").unwrap();
        git(&target,&["add","next-round"]);git(&target,&["commit","-qm","ronde suivante"]);
        let next_head=git(&target,&["rev-parse","HEAD"]);
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("preuve après commit attendue")};
        assert!(observation.native_ready);
        assert!(observation.permission_digest.is_some(),"un commit après migration doit rester continuable");
        assert_eq!(observation.workspace.as_ref().unwrap()["head_commit"],next_head);
        // Les délégations du parent réparé prouvent les nouveaux droits et le nouveau dossier.
        let parent=ProviderPermissions {cwd:target.to_string_lossy().into_owned(),
            provider_policy:published.prepared.as_ref().unwrap().child_policy.clone(),
            provider_instance_id:identity.instance_id.clone(),..prepare.parent.clone()};
        let request_frame=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"parent-repaired185".into(),
            connection_epoch:"epoch185".into(),worker:runtime.identity.clone(),command:identity.clone(),
            request:WorkerRequestOperation::ParentProof {parent:parent.clone(),expected_revision:parent.revision}};
        let disconnected=AtomicBool::new(false);
        assert!(matches!(runtime.handle(&request_frame,"epoch185",&disconnected).unwrap(),
            WorkerResponseOperation::ParentProof {..}));
        let mut source_frame=request_frame;
        source_frame.request=WorkerRequestOperation::ParentSourceProof {parent:parent.clone(),
            source:bridget_transport::protocol::WorkerSourceBase {cwd:parent.cwd.clone(),
                expected_base_commit:Some(next_head),workspace:Some(WorkspaceRequest::Shared {}),
                workload:bridget_transport::protocol::AutoDelegateWorkload::General}};
        assert!(matches!(runtime.handle(&source_frame,"epoch185",&disconnected).unwrap(),
            WorkerResponseOperation::ParentSourceProof {source_error:None,..}));
        // Une annulation arrivée après les relectures ferme le point atomique de publication.
        let before=runtime.store.command(&identity).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&identity).unwrap());
        assert_eq!(runtime.publish_repair_status(&identity,published.clone(),worker_now()+60).unwrap_err(),WorkerRefusal::Unauthorized);
        assert_eq!(runtime.store.command(&identity).unwrap(),before);
        let WorkerResponseOperation::WorkspaceRepair {status:observed}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Observe,0).unwrap()
            else {panic!("lecture sans effet attendue")};
        assert_eq!(observed.phase,"published","l'annulation n'efface pas le reçu publié");
        let capability_path=workspace_repair::capability_path(&runtime.store.root,&identity);
        let result_path=workspace_repair::result_path(&runtime.store.root,&identity,&request.request_id);
        let refused_path=workspace_repair::result_path(&runtime.store.root,&identity,"repair185-worker");
        runtime.store.collect(worker_now()+2*WORKER_RECEIPT_RETENTION_SECS).unwrap();
        assert!(capability_path.exists()&&result_path.exists()&&refused_path.exists(),"commande active conservée");
        bridget_transport::managed_session::stop_owned_child(&mut child,true,Duration::from_secs(2)).unwrap();
        let mut ended=runtime.store.command(&identity).unwrap();ended.phase=WorkerCommandPhase::Terminal;
        runtime.store.persist(&runtime.store.command_path(&identity),&ended).unwrap();
        runtime.store.collect(worker_now()+2*WORKER_RECEIPT_RETENTION_SECS).unwrap();
        assert!(!capability_path.exists()&&!result_path.exists()&&!refused_path.exists(),"marqueurs collectés avec le reçu");
        assert!(!runtime.store.command_path(&identity).exists());
    }

    #[test]
    fn spec185_capabilities_remain_independent_and_reject_unoffered_origin_proof() {
        let features=MasterWorkerFeatures {workspace_repair:true,..Default::default()};
        assert_eq!(features.capabilities(),vec![WorkerCapability::WorkspaceRepairV1]);
        assert!(features.admits(&[WorkerCapability::WorkspaceRepairV1]));
        assert!(!features.admits(&[WorkerCapability::OriginProofV1]));
        assert!(!features.admits(&[WorkerCapability::WorkspaceRepairV1,WorkerCapability::WorkspaceRepairV1]));
    }

    fn start198(deadline_at: i64) -> WorkerStart {
        WorkerStart {
            prepared_token: "token-198".into(),
            child_id: "child-198".into(),
            mission_id: "mission-198".into(),
            forward_child_requests: false,
            placement: placement(),
            definition: ResolvedAgentDefinition {
                command: "/bin/true".into(), args: vec![], protocol: "codex_app_server".into(),
                forbidden_env: vec![], pass_env: vec![], permissions: "plan".into(),
                claude_config_dir: None,
                codex_home: None, queue_capacity: 1, notify_timeout_secs: 60,
                mcp: bridget_transport::protocol::ResolvedMcpDefinition {
                    interactive: "off".into(), acp_session: false,
                },
                capabilities: Default::default(), digest: "a".repeat(64),
            },
            child_policy: serde_json::json!({}),
            effective_posture: SpawnPosture::Discovery,
            deadline_at,
        }
    }

    /// Schémas copiés de 7478e9e7 (183) : tout champ inconnu ou délai incohérent échoue.
    #[allow(dead_code)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Legacy183Command {
        version: u16, worker: WorkerIdentity, identity: WorkerCommandIdentity,
        command_digest: String, placement: RemotePlacement, deadline_at: i64,
        phase: WorkerCommandPhase, marker: Option<ManagedMarker>,
        permit: Option<WorkerReleasePermit>, release_epoch: Option<String>,
        #[serde(default)] child_id: Option<String>,
        #[serde(default)] native_ready: bool,
        #[serde(default)] startup_failed: bool,
        #[serde(default)] startup_failure: Option<bridget_transport::protocol::WorkerStartupFailure>,
        #[serde(default)] terminal_exit_code: Option<i32>,
        #[serde(default)] preparation: Option<(WorkerPrepare, WorkerPrepared)>,
        #[serde(default)] routing: Option<RoutingPreparation>,
        created_at: i64, updated_at: i64,
    }
    #[allow(dead_code)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Legacy183Workspace {
        version: u16, worker: WorkerIdentity, command: WorkerCommandIdentity,
        prepare_digest: String, record: Option<crate::delegation::WorkspaceRecord>,
    }
    /// Relit tout `commands` et `workspaces` comme le ferait l'inventaire183.
    fn assert_legacy183_readable(root: &Path) -> usize {
        let mut seen = 0;
        for entry in fs::read_dir(root.join("commands")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|ext| ext != "json") { continue; }
            let receipt: Legacy183Command =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert!(receipt.deadline_at > receipt.created_at, "règle183 deadline > created_at");
            seen += 1;
        }
        if let Ok(entries) = fs::read_dir(root.join("workspaces")) {
            for entry in entries {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|ext| ext != "json") { continue; }
                let _: Legacy183Workspace =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                seen += 1;
            }
        }
        seen
    }

    #[test]
    fn spec198_canon_fence_ne_normalise_que_le_renouvellement_provider_turn() {
        let parent = ProviderPermissions {
            version: 1, source: "provider_turn".into(), run_id: "run".into(),
            provider_session_id: "session".into(), provider_instance_id: "parent".into(),
            revision: 1, driver: "codex_app_server".into(), cwd: "/origin".into(),
            runtime_mode: "full-access".into(), interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never"}),
        };
        let a = WorkerPrepare { service_tier: None,
            machine_id: "cartae".into(), project_id: "project".into(), provisioning_revision: 1,
            mapping_revision: 1, origin_host: "origin".into(), origin_cwd: "/origin".into(),
            source_root: "/origin".into(), remote_root: "/remote".into(),
            source_root_identity: "git:held".into(), relative_cwd: ".".into(), base_commit: None,
            agent_type: "codex".into(), model: "fixture".into(), effort: None, posture: None,
            workspace: None, parent,
        };
        let mut b = a.clone();
        b.parent.run_id = "autre".into();
        b.parent.provider_session_id = "autre-session".into();
        b.parent.revision += 7;
        assert_eq!(fenced_prepare_canon(&a), fenced_prepare_canon(&b));
        for mutate in [
            (|r: &mut WorkerPrepare| r.parent.cwd.push('x')) as fn(&mut WorkerPrepare),
            |r| r.parent.driver.push('x'),
            |r| r.parent.provider_instance_id.push('x'),
            |r| r.parent.runtime_mode.push('x'),
            |r| r.parent.provider_policy = serde_json::json!({"autre": true}),
            |r| r.model.push('x'),
        ] {
            let mut c = a.clone();
            mutate(&mut c);
            assert_ne!(fenced_prepare_canon(&a), fenced_prepare_canon(&c));
        }
        let mut native = a.clone();
        native.parent.source = "native_wrapper".into();
        let mut native_b = native.clone();
        native_b.parent.run_id = "autre".into();
        assert_ne!(fenced_prepare_canon(&native), fenced_prepare_canon(&native_b));
    }

    #[test]
    fn spec198_fence_reste_lisible_format183_et_preuve_seule_bloque_start() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let deadline = worker_now() - 1;
        runtime.fence_unstarted(&command(), &placement(), deadline).unwrap();
        let raw: serde_json::Value = serde_json::from_slice(
            &fs::read(runtime.store.command_path(&command())).unwrap()).unwrap();
        assert!(raw.get("pre_start_fence").is_none(), "reçu commande strictement au format183");
        assert!(raw["deadline_at"].as_i64().unwrap() > raw["created_at"].as_i64().unwrap());
        let proof = proof198::read_fence(&runtime.store.root, &runtime.store.worker, &command())
            .unwrap().unwrap();
        assert_eq!(proof.deadline_at, deadline, "le délai historique reste exact dans la preuve");
        assert_eq!(runtime.store.command(&command()).unwrap().deadline_at, deadline);
        assert!(runtime.store.inventory().is_ok());
        assert_eq!(assert_legacy183_readable(&runtime.store.root), 1);

        // Crash après la preuve et avant le reçu : aucun Start neuf, le rejeu termine.
        let other = WorkerCommandIdentity { command_id: "command-198-proof-only".into(), ..command() };
        let created_at = worker_now();
        let digest = proof198::fence_digest(&runtime.store.worker, &other, &placement(), deadline)
            .unwrap();
        proof198::ensure_dir(&runtime.store.root).unwrap();
        runtime.store.persist(&proof198::fence_path(&runtime.store.root, &other),
            &proof198::FenceProof { version: STORE_VERSION, worker: runtime.store.worker.clone(),
                identity: other.clone(), placement: placement(), deadline_at: deadline,
                created_at, digest }).unwrap();
        let (_, _, _, before_orphan) = runtime.store.inventory().unwrap();
        assert!(before_orphan >= 2, "la preuve annexe orpheline compte dans la quota d'entrées");
        assert!(matches!(runtime.store.reserve_command(&other, &"a".repeat(64), &placement(),
            worker_now() + 300, worker_now()), Err(WorkerStoreError::Conflict)));
        assert_eq!(runtime.fence_unstarted(&other, &placement(), deadline + 1).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        assert!(!runtime.store.command_path(&other).exists());
        runtime.fence_unstarted(&other, &placement(), deadline).unwrap();
        let receipt = runtime.store.command(&other).unwrap();
        assert_eq!((receipt.created_at, receipt.deadline_at), (created_at, deadline));
        assert_eq!(receipt.phase, WorkerCommandPhase::Refused);
        assert_eq!(assert_legacy183_readable(&runtime.store.root), 2);
        // Réponse perdue puis redémarrage : même inventaire, aucune quarantaine, rejeu identique.
        let bytes = fs::read(runtime.store.command_path(&other)).unwrap();
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut reopened = WorkerRuntime::new(config).unwrap();
        assert_eq!(reopened.fence_unstarted(&other, &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(reopened.store.command_path(&other)).unwrap(), bytes);
        assert_eq!(reopened.store.command(&command()).unwrap().deadline_at, deadline);
        assert_eq!(assert_legacy183_readable(&reopened.store.root), 2);
    }

    #[test]
    fn spec198_fence_durable_replay_divergence_et_start_tardif() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let deadline = worker_now() - 1;
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        let receipt = runtime.store.command(&command()).unwrap();
        assert_eq!(receipt.phase, WorkerCommandPhase::Refused);
        assert!(receipt.deadline_at < receipt.created_at,
            "le fence conserve le délai historique, même déjà expiré");
        assert!(receipt.pre_start_fence.is_some());
        assert!(receipt.marker.is_none() && receipt.permit.is_none() && receipt.child_id.is_none());
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        assert!(!runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command()))).exists());
        let path = runtime.store.command_path(&command());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation: WorkerObservation {
                phase: WorkerCommandPhase::Refused, ..
            } }));
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline + 1).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        let mut divergent = placement();
        divergent.mapping_revision += 1;
        assert_eq!(runtime.fence_unstarted(&command(), &divergent, deadline).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        assert_eq!(runtime.start(&command(), &start198(deadline), &AtomicBool::new(false),
            &mut WorkerStartCapacityCause::Unknown).unwrap_err(), WorkerRefusal::CommandMismatch);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut reopened = WorkerRuntime::new(config).unwrap();
        assert_eq!(reopened.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(path).unwrap(), bytes);
    }

    #[test]
    fn spec198_fence_quota_refuse_sans_recu_puis_accepte_meme_cle() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let deadline = worker_now() - 1;
        let quota_file = runtime.store.root.join("quota-fence198");
        let quota = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(&quota_file).unwrap();
        quota.set_len(MAX_WORKER_STORE_BYTES as u64).unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap_err(),
            WorkerRefusal::ResourceLimit);
        assert!(!runtime.store.command_path(&command()).exists());
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        fs::remove_file(quota_file).unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(runtime.store.command(&command()).unwrap().phase, WorkerCommandPhase::Refused);
    }

    #[test]
    fn spec198_fence_refuse_tous_reçus_existants_et_marqueur_orphelin() {
        for phase in [WorkerCommandPhase::Reserved, WorkerCommandPhase::Unknown,
            WorkerCommandPhase::Refused] {
            let fixture = Fixture::new();
            let mut runtime = runtime_fixture(&fixture);
            let now = worker_now();
            runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
            if phase != WorkerCommandPhase::Reserved {
                runtime.store.set_phase(&command(), phase, now + 1).unwrap();
            }
            let path = runtime.store.command_path(&command());
            let before = fs::read(&path).unwrap();
            assert_eq!(runtime.fence_unstarted(&command(), &placement(), now + 300).unwrap_err(),
                WorkerRefusal::UnknownExecution);
            assert_eq!(fs::read(path).unwrap(), before);
        }
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let marker = runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command())));
        write_private_file_atomic(&marker, b"orphan").unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), worker_now() - 1).unwrap_err(),
            WorkerRefusal::UnknownExecution);
        assert!(!runtime.store.command_path(&command()).exists());
    }

    #[test]
    fn spec198_file_pleine_distingue_start_detaille_et_wire_historique() {
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "request-198".into(),
            connection_epoch: "epoch-198".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::StartDetailed { start: start198(worker_now() + 300) },
        };
        request.validate().unwrap();
        let wire = serde_json::to_value(&request).unwrap();
        assert_eq!(wire["request"]["operation"], "start_detailed");
        assert_eq!(serde_json::from_value::<WorkerRequest>(wire).unwrap(), request);
        let busy = worker_inbound_busy(WorkerInbound::Request(request.clone()));
        assert!(matches!(busy, WrapperToDaemon::WorkerResponse { response: WorkerResponse {
            response: WorkerResponseOperation::StartRejected {
                reason: WorkerRefusal::ResourceLimit,
                cause: WorkerStartCapacityCause::InboundQueue,
            }, ..
        } }));
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(inbound172("occupant", WorkerRequestOperation::Reconcile {})).unwrap();
        let (socket, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let disconnected = AtomicBool::new(false);
        dispatch_worker_inbound(&sender, WorkerInbound::Request(request.clone()), &writer,
            &disconnected, &Mutex::new(BTreeSet::new())).unwrap();
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut std::io::BufReader::new(peer))
            else { panic!("réponse détaillée attendue"); };
        response.validate().unwrap();
        assert_eq!(response.response, WorkerResponseOperation::StartRejected {
            reason: WorkerRefusal::ResourceLimit, cause: WorkerStartCapacityCause::InboundQueue,
        });
        assert!(matches!(receiver.try_recv().unwrap(), WorkerInbound::Request(WorkerRequest {
            request: WorkerRequestOperation::Reconcile {}, ..
        })));
        assert!(receiver.try_recv().is_err(), "Start refusé ne doit pas rester en file");
        let mut legacy = request;
        legacy.request = WorkerRequestOperation::Start { start: start198(worker_now() + 300) };
        assert_eq!(serde_json::to_value(&legacy).unwrap()["request"]["operation"], "start");
        let busy = worker_inbound_busy(WorkerInbound::Request(legacy));
        assert!(matches!(busy, WrapperToDaemon::WorkerResponse { response: WorkerResponse {
            response: WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResourceLimit }, ..
        } }));
        let features = MasterWorkerFeatures { start_recovery: true, ..Default::default() };
        assert_eq!(features.capabilities(), vec![WorkerCapability::StartRecoveryV1]);
        assert!(features.admits(&[WorkerCapability::StartRecoveryV1]));
        assert!(!MasterWorkerFeatures::default().admits(&[WorkerCapability::StartRecoveryV1]));
        assert_eq!(WorkerStartCapacityCause::PreparationMetadata.code(), "preparation_metadata");
        assert_eq!(serde_json::to_value(WorkerCapability::StartRecoveryV1).unwrap(), "start_recovery_v1");
        let fence = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "fence-198".into(),
            connection_epoch: "epoch-198".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::FenceUnstarted { placement: placement(), deadline_at: 1 },
        };
        fence.validate().unwrap();
        let wire = serde_json::to_value(&fence).unwrap();
        assert_eq!(wire["request"]["operation"], "fence_unstarted");
        assert_eq!(serde_json::from_value::<WorkerRequest>(wire).unwrap(), fence);
        assert!(matches!(worker_inbound_busy(WorkerInbound::Request(fence)),
            WrapperToDaemon::WorkerResponse { response: WorkerResponse {
                response: WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::ResourceLimit,
                }, ..
            } }));
    }

    #[test]
    fn spec198_fence_exige_epoque_et_frame_trop_grande_reste_sans_effet() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let mut request = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "fence-198".into(),
            connection_epoch: "epoch-198".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::FenceUnstarted {
                placement: placement(), deadline_at: worker_now() - 1,
            },
        };
        let connected = AtomicBool::new(false);
        assert_eq!(runtime.handle(&request, "other-epoch", &connected).unwrap_err(),
            WorkerRefusal::StaleConnection);
        assert!(!runtime.store.command_path(&command()).exists());
        assert_eq!(runtime.handle(&request, "epoch-198", &connected).unwrap(),
            WorkerResponseOperation::StartFenced {});
        request.request = WorkerRequestOperation::StartDetailed {
            start: start198(worker_now() + 300),
        };
        if let WorkerRequestOperation::StartDetailed { start } = &mut request.request {
            start.child_policy = serde_json::json!({"large": "x".repeat(65 * 1024)});
        }
        assert_eq!(request.validate(), Err(WorkerRefusal::ResourceLimit));
        assert!(matches!(worker_inbound_busy(WorkerInbound::Request(request.clone())),
            WrapperToDaemon::WorkerResponse { response: WorkerResponse {
                response: WorkerResponseOperation::StartRejected {
                    reason: WorkerRefusal::ResourceLimit,
                    cause: WorkerStartCapacityCause::RequestFrame,
                }, ..
            } }));
        assert_eq!(runtime.handle(&request, "epoch-198", &connected).unwrap(),
            WorkerResponseOperation::StartRejected {
                reason: WorkerRefusal::ResourceLimit,
                cause: WorkerStartCapacityCause::RequestFrame,
            });
        if let WorkerRequestOperation::StartDetailed { start } = &mut request.request {
            start.child_policy = serde_json::json!({});
            start.prepared_token = bridget_transport::protocol::WORKER_INSPECTION_TOKEN.into();
        }
        assert_eq!(runtime.handle(&request, "epoch-198", &connected).unwrap(),
            WorkerResponseOperation::StartRejected {
                reason: WorkerRefusal::InvalidEnvelope,
                cause: WorkerStartCapacityCause::Unknown,
            });
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
    }

    #[test]
    fn spec235_channel_diagnostic_is_closed_and_preserves_rejection() {
        let message=DaemonToWrapper::WorkerRejected{reason:WorkerRefusal::CommandMismatch};
        let fact=unexpected_frame_diagnostic235(&message,&worker(),"epoch");
        assert_eq!(fact["frame_kind"],"master_rejected");assert_eq!(fact["refusal"],"command_mismatch");
        assert_eq!(fact["identity_matches"],serde_json::Value::Null);
        let secret=DaemonToWrapper::AgentTrackingResult{result:serde_json::json!({"private":"PRIVATE-CREDENTIAL"})};
        let fact=unexpected_frame_diagnostic235(&secret,&worker(),"epoch");
        assert_eq!(fact["frame_kind"],"other");assert!(!fact.to_string().contains("PRIVATE"));
        let fact=unexpected_frame_diagnostic235(&DaemonToWrapper::UpdateInProgress,&worker(),"epoch");
        assert_eq!(fact["frame_kind"],"update_in_progress");
    }

    #[test]
    fn spec234_large_view_reservation_reopens_and_recovers_without_receipt_rewrite() {
        let fixture=Fixture::new();let (runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        let seed=c19_plan(&intent);runtime.store.reserve_view_plan(&command,&request,&seed).unwrap();
        let root=runtime.store.root.clone();let worker=runtime.identity.clone();
        let receipt=runtime.store.workspace_path(&command);let bytes=fs::read(&receipt).unwrap();
        // Un reçu de préparation ordinaire conservé ne bloque pas cette reprise.
        let mut retained:WorkspacePreparationReceipt=read_optional(&receipt,MAX_METADATA_BYTES).unwrap().unwrap();
        retained.command.task_id=uuid::Uuid::new_v4().to_string();
        retained.command.command_id=uuid::Uuid::new_v4().to_string();retained.commit_view=None;
        let ordinary=runtime.store.workspace_path(&retained.command);
        write_json(&ordinary,&retained).unwrap();let ordinary_before=fs::read(&ordinary).unwrap();
        assert!(runtime.store.inventory().is_ok());
        runtime.store.quarantine().unwrap();drop(runtime);
        let result=WorkerStore::recover_legacy_view_capacity(&root,worker.clone()).unwrap();
        assert_eq!(result["cause"],"legacy_view_reservation_limit");assert_eq!(result["processes_started"],0);
        assert_eq!(fs::read(&receipt).unwrap(),bytes);
        assert_eq!(fs::read(&ordinary).unwrap(),ordinary_before);
        let reopened=WorkerStore::open(&root,worker).unwrap();assert!(reopened.inventory().is_ok());
        assert_eq!(fs::read(&receipt).unwrap(),bytes);
    }

    #[test]
    fn spec234_recovery_refuses_live_lock_and_missing_witness() {
        let fixture=Fixture::new();let (runtime,_,_,_)=c19_preparation_fixture(&fixture);
        let root=runtime.store.root.clone();let worker=runtime.identity.clone();let enrolled=runtime.store.enrollment.clone();
        runtime.store.quarantine().unwrap();let before=fs::read(&enrolled).unwrap();
        assert!(matches!(WorkerStore::recover_legacy_view_capacity(&root,worker.clone()),Err(WorkerStoreError::Locked)));
        assert_eq!(fs::read(&enrolled).unwrap(),before);drop(runtime);
        assert!(WorkerStore::recover_legacy_view_capacity(&root,worker).is_err());
        assert_eq!(fs::read(&enrolled).unwrap(),before);
    }

    #[test]
    fn spec234_recovery_keeps_unknown_bootstrap_and_wrong_identity_quarantined() {
        let fixture=Fixture::new();let (runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        runtime.store.reserve_view_plan(&command,&request,&c19_plan(&intent)).unwrap();
        runtime.store.mark_view_source_attempt_binding(&intent.binding).unwrap();
        let root=runtime.store.root.clone();let worker=runtime.identity.clone();let enrolled=runtime.store.enrollment.clone();
        runtime.store.quarantine().unwrap();let before=fs::read(&enrolled).unwrap();drop(runtime);
        let mut other=worker.clone();other.worker_id="different-worker".into();
        assert!(WorkerStore::recover_legacy_view_capacity(&root,other).is_err());
        assert!(WorkerStore::recover_legacy_view_capacity(&root,worker).is_err());
        assert_eq!(fs::read(&enrolled).unwrap(),before);
    }

    #[test]
    fn spec234_recovery_refuses_live_orphan_marker_and_invalid_record() {
        let fixture=Fixture::new();let (runtime,command,request,intent)=c19_preparation_fixture(&fixture);
        runtime.store.reserve_view_plan(&command,&request,&c19_plan(&intent)).unwrap();
        let root=runtime.store.root.clone();let worker=runtime.identity.clone();let enrolled=runtime.store.enrollment.clone();
        private_dir(&root.join("markers")).unwrap();
        let marker=root.join("markers/live-orphan.json");
        write_json(&marker,&ManagedMarker{pgid:unsafe{libc::getpgrp()} as u32,birth:1,
            instance_id:command.task_id.clone(),command_id:command.command_id.clone(),generation:1}).unwrap();
        runtime.store.quarantine().unwrap();let before=fs::read(&enrolled).unwrap();drop(runtime);
        assert!(WorkerStore::recover_legacy_view_capacity(&root,worker.clone()).is_err());
        assert_eq!(fs::read(&enrolled).unwrap(),before);
        fs::remove_file(&marker).unwrap(); // fixture only, never an operator recovery.
        let invalid=root.join("commands/invalid.json");write_private_file_atomic(&invalid,b"{").unwrap();
        assert!(WorkerStore::recover_legacy_view_capacity(&root,worker).is_err());
        assert_eq!(fs::read(&enrolled).unwrap(),before);assert_eq!(fs::read(invalid).unwrap(),b"{");
    }

