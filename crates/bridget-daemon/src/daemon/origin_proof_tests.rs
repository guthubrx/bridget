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
    st.peer_uids.insert(
        "worker177".into(),
        bridget_transport::fsutil::effective_uid(),
    );
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
fn spec229_authenticated_origin_prepares_coordinator_without_task_and_rejects_mismatch() {
    use serde_json::json;
    use sha2::Digest;
    use std::io::BufRead;
    let (shared, root, _) = fixture();
    let canonical = root.canonicalize().unwrap();
    let path = canonical.join("agents.json");
    let owner = crate::t3code::stable_uuid("parent229");
    let instance = crate::t3code::stable_uuid("parent-instance229");
    let (primary, _primary_peer) = UnixStream::pair().unwrap();
    {
        let mut st = shared.lock().unwrap();
        st.connections.insert(
            "primary229".into(),
            Arc::new(Mutex::new(BufWriter::new(primary))),
        );
        assert!(matches!(
            handle_register_with_channel(
                "primary229",
                2,
                "parent229".into(),
                owner.clone(),
                Some("origin177".into()),
                Some("t3code".into()),
                ChannelReport::Known("ssh-unix".into()),
                Some(PresenceMode::Cli),
                None,
                Some("fixture".into()),
                Some(instance),
                None,
                false,
                Some(false),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
    }
    let target = WorkerIdentity {
        machine_id: "target229".into(),
        worker_id: "worker229".into(),
        provisioning_revision: 1,
    };
    let parent = ProviderPermissions {
        version: 1,
        source: "provider_turn".into(),
        run_id: "run229".into(),
        provider_session_id: "session229".into(),
        provider_instance_id: "instance229".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: canonical.to_string_lossy().into_owned(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: json!({"kind":"codex","approval_policy":"never",
                "approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
    };
    let digest = format!(
        "{:x}",
        sha2::Sha256::digest(serde_json::to_vec(&parent).unwrap())
    );
    let source = WorkerSourceBase {
        cwd: parent.cwd.clone(),
        expected_base_commit: Some("a".repeat(40)),
        workspace: Some(WorkspaceRequest::Shared {}),
        workload: AutoDelegateWorkload::General,
    };
    let mapping = json!({"source_host":"origin177","source_root":canonical,
            "remote_root":"/tmp/target229/repository","expected_git_identity":"fixture229","mapping_revision":1});
    let project = PreparedOriginProject {
        target: target.clone(),
        credential_fingerprint: "a".repeat(64),
        project_id: "project229".into(),
        mapping: mapping.clone(),
    };
    let (writer, peer) = UnixStream::pair().unwrap();
    shared.lock().unwrap().connections.insert(
        "worker177".into(),
        Arc::new(Mutex::new(BufWriter::new(writer))),
    );
    let worker_state = shared.clone();
    let fake = std::thread::spawn(move || {
        peer.set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut reader = std::io::BufReader::new(peer);
        for case in 0..6 {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::OriginProofRequest { request } =
                serde_json::from_str(&line).unwrap()
            else {
                panic!("origin request")
            };
            request.validate_shape().unwrap();
            assert!(
                matches!(request.request,OriginProofOperation::PrepareProject {return_context,..} if return_context==(case<4))
            );
            let mut context = project.clone();
            if case == 1 {
                context.target.provisioning_revision += 1;
            }
            if case == 3 {
                context.mapping["remote_root"] = json!("relative");
            }
            receive(
                &mut worker_state.lock().unwrap(),
                "worker177",
                OriginProofResponse {
                    version: 1,
                    correlation_id: request.correlation_id,
                    connection_epoch: request.connection_epoch,
                    worker: request.worker,
                    response: OriginProofResult::ProjectPrepared {
                        permission_digest: digest.clone(),
                        preparation_error: None,
                        project: (case < 4).then_some(context),
                    },
                },
            );
        }
    });
    for (case, expected) in [
        Ok(()),
        Err("project_preparation_machine_mismatch"),
        Err("project_mapping_mismatch"),
        Err("project_mapping_invalid"),
        Err("project_preparation_context_missing"),
        Ok(()),
    ]
    .into_iter()
    .enumerate()
    {
        let config = json!({"machines":{
            "origin177":{"enabled":true,"worker_id":"worker-origin177","provisioning_revision":1,
                "credential_fingerprint":format!("{:x}",sha2::Sha256::digest(b"fixture177-worker-key")),"mappings":{}},
            "target229":{"enabled":true,"worker_id":target.worker_id,"provisioning_revision":1,
                "credential_fingerprint":"a".repeat(64),"mappings":if case==5 {json!({"project229":mapping})}else{json!({})}}
        }});
        bridget_transport::fsutil::write_private_file_atomic(
            &path,
            &serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        {
            let mut st = shared.lock().unwrap();
            st.registry = AgentRegistry::load_from_path_quiet(path.clone()).unwrap();
            st.remote_workers.get_mut("origin177").unwrap().capabilities = if case < 4 {
                vec![
                    WorkerCapability::OriginProofV1,
                    WorkerCapability::OriginProjectContextV1,
                ]
            } else {
                vec![WorkerCapability::OriginProofV1]
            };
        }
        let result = prepare_project(
            &shared,
            &owner,
            &parent,
            source.clone(),
            "target229",
            (case == 2).then_some("required229"),
            Instant::now() + Duration::from_secs(5),
        );
        assert_eq!(
            result.as_ref().map(|_| ()).map_err(String::as_str),
            expected,
            "case {case}"
        );
        let registry = AgentRegistry::load_from_path_quiet(path.clone()).unwrap();
        assert_eq!(
            registry.machine("target229").unwrap().mappings.len(),
            usize::from(case == 0 || case == 5)
        );
        assert!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .tasks()
                .unwrap()
                .is_empty()
        );
    }
    fake.join().unwrap();
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn spec221_c19_origin_reader_never_receives_creation_source_or_prepare() {
    let (shared, root, _identity) = fixture();
    let (writer, peer) = UnixStream::pair().unwrap();
    shared.lock().unwrap().connections.insert(
        "worker177".into(),
        Arc::new(Mutex::new(BufWriter::new(writer))),
    );
    let parent = ProviderPermissions {
        version: 1,
        source: "provider_turn".into(),
        run_id: "run".into(),
        provider_session_id: "session".into(),
        provider_instance_id: "instance".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: root.to_string_lossy().into_owned(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never",
                "sandbox_policy":{"type":"dangerFullAccess"}}),
    };
    let source = WorkerSourceBase {
        cwd: parent.cwd.clone(),
        expected_base_commit: Some("a".repeat(40)),
        workspace: Some(WorkspaceRequest::CommitView {}),
        workload: AutoDelegateWorkload::General,
    };
    for operation in [
        OriginProofOperation::Sources {
            parent: parent.clone(),
            source: Some(source.clone()),
        },
        OriginProofOperation::PrepareProject {
            parent: parent.clone(),
            source,
            target_machine: "target".into(),
            project_id: Some("project".into()),
            return_context: false,
        },
    ] {
        assert_eq!(
            exchange(&shared, "origin177", operation, Duration::from_secs(1)),
            Err(if crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED {
                "origin_commit_view_unsupported"
            } else {
                "workspace_commit_view_not_enabled"
            }
            .into())
        );
    }
    peer.set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let mut reader = std::io::BufReader::new(peer);
    let err = std::io::Read::read(&mut reader, &mut [0]).unwrap_err();
    assert!(matches!(
        err.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    ));
    assert!(shared.lock().unwrap().pending_origin_proofs.is_empty());
    assert!(
        shared
            .lock()
            .unwrap()
            .delegation_store
            .tasks()
            .unwrap()
            .is_empty()
    );
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec246_mobile177_foreign_project_requires_worker_proof_and_matching_primary_host() {
    use std::io::BufRead;
    let (shared, root, _identity) = fixture();
    let thread = "foreign-project177";
    let owner = crate::t3code::stable_uuid(thread);
    let instance = crate::t3code::stable_uuid(&format!("instance:{thread}"));
    let (primary, _primary_peer) = UnixStream::pair().unwrap();
    {
        let mut st = shared.lock().unwrap();
        st.connections.insert(
            "primary177".into(),
            Arc::new(Mutex::new(BufWriter::new(primary))),
        );
        assert!(matches!(
            handle_register_with_channel(
                "primary177",
                2,
                "fixture".into(),
                owner.clone(),
                Some("origin177".into()),
                Some("t3code".into()),
                ChannelReport::Known("ssh-unix".into()),
                Some(PresenceMode::Cli),
                None,
                Some("fixture".into()),
                Some(instance.clone()),
                None,
                false,
                Some(false),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
    }
    let (writer, peer) = UnixStream::pair().unwrap();
    shared.lock().unwrap().connections.insert(
        "worker177".into(),
        Arc::new(Mutex::new(BufWriter::new(writer))),
    );
    let for_worker = shared.clone();
    let fake = std::thread::spawn(move || {
        peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut reader = std::io::BufReader::new(peer);
        for _ in 0..3 {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::OriginProofRequest { request } =
                serde_json::from_str(&line).unwrap()
            else {
                panic!("origin request")
            };
            assert!(matches!(
                &request.request,
                OriginProofOperation::CommunicationProject {
                    source: CommunicationProjectSource::T3,
                    ..
                }
            ));
            let response = crate::origin_proof::execute(&request, std::path::Path::new("/unused"));
            assert!(matches!(
                &response,
                OriginProofResult::CommunicationProject { project: Some(_) }
            ));
            receive(
                &mut for_worker.lock().unwrap(),
                "worker177",
                OriginProofResponse {
                    version: 1,
                    correlation_id: request.correlation_id,
                    connection_epoch: request.connection_epoch,
                    worker: request.worker,
                    response,
                },
            );
        }
    });
    assert!(matches!(
        announce_communication_project(
            &shared,
            "primary177",
            root.to_str().unwrap(),
            CommunicationProjectSource::Git,
            "origin177",
            None
        ),
        DaemonToWrapper::ProjectContextResult { project: None, .. }
    ));
    let result = announce_communication_project(
        &shared,
        "primary177",
        root.to_str().unwrap(),
        CommunicationProjectSource::T3,
        "origin177",
        None,
    );
    let human_project = human_request_project(&shared, thread, root.to_str().unwrap());
    assert!(human_project.is_some());
    assert!(matches!(
        result,
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
    assert!(
        handle_wrapper_message(
            "primary177",
            WrapperToDaemon::T3ThreadBindingFact {
                version: HUMAN_THREAD_VIEW_VERSION,
                t3_thread_id: thread.into()
            },
            &shared
        )
        .is_none()
    );
    {
        let mut st = shared.lock().unwrap();
        st.connection_roles
            .insert("human246".into(), ConnectionRole::Client);
        st.client_negotiations.insert(
            "human246".into(),
            NegotiatedClient {
                version: 1,
                issuer_scope: "fixture246".into(),
                capabilities: vec![ClientCapability::HumanLineageViewV1],
            },
        );
    }
    let response = handle_wrapper_message(
        "human246",
        WrapperToDaemon::HumanLineage {
            request: HumanLineageRequest {
                version: 4,
                t3_thread_id: thread.into(),
                project_root: root.to_string_lossy().into_owned(),
                request: HumanLineageAction::List {
                    limit: 10,
                    cursor: None,
                },
            },
        },
        &shared,
    )
    .unwrap();
    let value = serde_json::to_value(response).unwrap();
    assert_eq!(
        value["result"]["status"], "ok",
        "le chemin de l’hôte propriétaire doit être résolu par son worker : {value}"
    );
    fake.join().unwrap();
    {
        let mut st = shared.lock().unwrap();
        let project = agent_communication_project(&st, &owner);
        assert_eq!(
            human_authority_guard(&st, thread, &project),
            Ok(("primary177".into(), owner.clone()))
        );
        let mut forged = project.clone().unwrap();
        forged.host = "other177".into();
        assert!(human_authority_guard(&st, thread, &Some(forged)).is_err());
        let local = st.host.clone();
        st.presences.get_mut(&instance).unwrap().host = local.clone();
        assert!(human_authority_guard(&st, thread, &project).is_err());
        st.presences.get_mut(&instance).unwrap().host = "origin177".into();
        st.conn_hosts.insert("primary177".into(), local);
        assert!(human_authority_guard(&st, thread, &project).is_err());
        st.conn_hosts
            .insert("primary177".into(), "origin177".into());
        remote_worker_disconnected(&mut st, "worker177");
        assert!(human_authority_guard(&st, thread, &project).is_err());
        assert_eq!(
            agent_communication_project(&st, &owner),
            project,
            "known scope must not become unknown on worker loss"
        );
        st.remote_workers.clear();
    }
    let retained = announce_communication_project(
        &shared,
        "primary177",
        root.to_str().unwrap(),
        CommunicationProjectSource::T3,
        "origin177",
        None,
    );
    assert!(matches!(
        retained,
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
    {
        let st = shared.lock().unwrap();
        let known = agent_communication_project(&st, &owner).unwrap();
        let other = CommunicationProject {
            host: known.host.clone(),
            root: "/other-project177".into(),
        };
        assert!(
            crate::communication::project_scope(Some(&known), Some(&other), "recipient177", None)
                .is_err()
        );
        assert!(!st.t3_thread_bindings.contains_key("primary177"));
        assert!(human_authority_guard(&st, thread, &Some(known)).is_err());
    }
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
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
        let DaemonToWrapper::OriginProofRequest { request } = serde_json::from_str(&line).unwrap()
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
fn mobile177_origin_live_two_hosts_no_task_or_provider_launch() {
    use std::io::BufRead;
    let file =
        std::env::var_os("BRIDGET177_ORIGIN_LIVE").expect("explicit private fixture file required");
    let file = PathBuf::from(file);
    crate::environment::validate_state_file(&file, false).unwrap();
    let config: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    let (shared, root, _identity) = fixture();
    // The live worker must authenticate itself on this real Unix/SSH connection.
    {
        let mut st = shared.lock().unwrap();
        st.remote_workers.clear();
        st.connection_roles.clear();
        st.peer_uids.clear();
    }
    let socket = file.parent().unwrap().join("s");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(file.parent().unwrap().join("ready"), "listening").unwrap();
    let state = shared.clone();
    let witness = file.parent().unwrap().join("wire-events.jsonl");
    let (accepted, ready) = mpsc::sync_channel(1);
    let service = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let uid = unix_peer_uid(&stream).unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(stream.try_clone().unwrap())));
        {
            let mut st = state.lock().unwrap();
            st.connections.insert("live177".into(), writer.clone());
            st.peer_uids.insert("live177".into(), uid);
        }
        let mut reader = std::io::BufReader::new(stream);
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }
            let frame: WrapperToDaemon = serde_json::from_str(&line).unwrap();
            let kind = serde_json::to_value(&frame).unwrap()["type"].clone();
            let response = handle_wrapper_message("live177", frame, &state);
            {
                use std::io::Write;
                let out = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&witness)
                    .unwrap();
                let value = serde_json::json!({"in":kind,"out":response.as_ref().map(|r|serde_json::to_value(r).unwrap()["type"].clone())});
                writeln!(&out, "{value}").unwrap();
            }
            if let Some(response) = response {
                let connected = matches!(
                    &response,
                    DaemonToWrapper::WorkerAcceptedWithCapabilities { .. }
                );
                push_control_message_until(
                    &writer,
                    &response,
                    Instant::now() + Duration::from_secs(5),
                )
                .unwrap();
                if connected {
                    let _ = accepted.try_send(());
                }
            }
        }
        remote_worker_disconnected(&mut state.lock().unwrap(), "live177");
    });
    ready
        .recv_timeout(Duration::from_secs(90))
        .expect("origin worker was not connected");
    let private = NativePermissionProof {
        endpoint: config["endpoint"].as_str().unwrap().into(),
        authorization: IdentityCredential::new(config["authorization"].as_str().unwrap().into()),
    };
    let expected = config["expected_thread"].as_str().unwrap();
    let result = exchange(
        &shared,
        "origin177",
        OriginProofOperation::Attest {
            expected_thread: expected.into(),
            proof: private.clone(),
        },
        Duration::from_secs(35),
    )
    .unwrap();
    let OriginProofResult::Attested { proof } = result else {
        panic!("attestation expected");
    };
    assert_eq!(proof.thread_id, expected);
    assert_eq!(proof.version, 2);
    proof.validate().unwrap();
    let parent = proof.permissions.unwrap();
    assert_eq!(parent.cwd, config["parent_cwd"].as_str().unwrap());
    assert!(
        !std::path::Path::new(&parent.cwd).exists(),
        "the coordinator must not have this origin path"
    );
    let source = WorkerSourceBase {
        cwd: parent.cwd.clone(),
        expected_base_commit: Some(config["expected_base_commit"].as_str().unwrap().into()),
        workspace: Some(WorkspaceRequest::Shared {}),
        workload: AutoDelegateWorkload::General,
    };
    let result = exchange(
        &shared,
        "origin177",
        OriginProofOperation::Sources {
            parent: parent.clone(),
            source: Some(source),
        },
        Duration::from_secs(30),
    )
    .unwrap();
    assert!(matches!(
        result,
        OriginProofResult::Sources {
            source_error: None,
            ..
        }
    ));
    if let Some(target) = config["prepare_machine"].as_str() {
        let prepare = OriginProofOperation::PrepareProject {
            parent: parent.clone(),
            source: WorkerSourceBase {
                cwd: parent.cwd.clone(),
                expected_base_commit: Some(config["expected_base_commit"].as_str().unwrap().into()),
                workspace: Some(WorkspaceRequest::Shared {}),
                workload: AutoDelegateWorkload::General,
            },
            target_machine: target.into(),
            project_id: Some("fixture177".into()),
            return_context: false,
        };
        for _ in 0..2 {
            let result = exchange(
                &shared,
                "origin177",
                prepare.clone(),
                Duration::from_secs(300),
            )
            .unwrap();
            assert!(
                matches!(
                    result,
                    OriginProofResult::ProjectPrepared {
                        preparation_error: None,
                        ..
                    }
                ),
                "private origin preparation must succeed, including an exact replay"
            );
        }
    }
    assert!(
        exchange(
            &shared,
            "origin177",
            OriginProofOperation::Attest {
                expected_thread: "other-thread177".into(),
                proof: private
            },
            Duration::from_secs(35)
        )
        .is_err()
    );
    {
        let st = shared.lock().unwrap();
        assert!(st.delegation_store.tasks().unwrap().is_empty());
        assert!(st.pending_origin_proofs.is_empty());
        let writer = st.connections["live177"].lock().unwrap();
        writer.get_ref().shutdown(std::net::Shutdown::Both).unwrap();
    }
    service.join().unwrap();
    std::fs::remove_file(socket).unwrap();
    std::fs::write(
        file.parent().unwrap().join("passed"),
        "origin177_two_hosts_ok",
    )
    .unwrap();
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
}
