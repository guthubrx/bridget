//! Oracles155 T017/T019–T022. Harnais149 réutilisé ; aucun fournisseur lancé.
//! Raccord provisoire sous les tests149 jusqu'au GO du code de production.
#![cfg(test)]
use super::*;
use crate::delegation::ChildRequest;
use bridget_transport::protocol::{
    ChildProviderRequest, ChildProviderRequestId, ChildRequestClosedState, ChildRequestDecision,
    ChildRequestKind,
};

// Fixture propre aux demandes155 : trois familles utilisent la même admission
// réelle149, le bail préparé, la route et la présence effectivement inscrites.
struct RequestFixture155 {
    state: Arc<Mutex<DaemonState>>,
    task: crate::delegation::Task,
    parent_reader: BufReader<UnixStream>,
    child_reader: BufReader<UnixStream>,
    _config: DaemonConfig,
    _managed: Receiver<ManagedSupervisorCommand>,
}

impl RequestFixture155 {
    fn new(label: &str) -> Self {
        Self::with_remote(label, false)
    }

    fn with_remote(label: &str, remote: bool) -> Self {
        Self::with_remote_start(label, remote, true)
    }

    fn with_remote_start(label: &str, remote: bool, started: bool) -> Self {
        let (mut st, config, rx, parent_reader) = fixture_with_reader(label);
        let cwd = st
            .fixture_root
            .as_ref()
            .unwrap()
            .0
            .join("project-149")
            .canonicalize()
            .unwrap();
        insert_fact(
            &mut st,
            codex_fact(
                cwd.to_str().unwrap(),
                json!({"type":"dangerFullAccess"}),
                "on-request",
            ),
            None,
        );
        let mut request = delegate_request(
            "native-test",
            label,
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Development),
        );
        if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
            *child_requests = Some("forward".into());
        }
        let view = handle_locked("conn-1", request, &mut st).unwrap();
        let mut task = st
            .delegation_store
            .get(view["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap();
        let (lease, definition) = if remote {
            use bridget_transport::protocol::RemotePlacement;
            use sha2::Digest;
            let placement = RemotePlacement {
                machine_id: "cartae155".into(),
                worker_id: "worker155".into(),
                provisioning_revision: 1,
                project_id: "project155".into(),
                mapping_revision: 1,
                origin_host: st.host.clone(),
                origin_cwd: task.cwd.clone(),
                source_root_identity: "git155".into(),
                relative_cwd: ".".into(),
                execution_cwd: "/remote155-only/project".into(),
                base_commit: None,
                definition_digest: task.definition.digest.clone(),
                permission_digest: format!(
                    "{:x}",
                    sha2::Sha256::digest(
                        serde_json::to_vec(
                            &task.permission_snapshot.as_ref().unwrap().child_policy
                        )
                        .unwrap()
                    )
                ),
            };
            st.registry = AgentRegistry::from_json(&json!({"machines":{"cartae155":{
                "enabled":true,"worker_id":"worker155","provisioning_revision":1,
                "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["native-test"],
                "mappings":{"project155":{"source_host":st.host,"source_root":cwd,"remote_root":"/remote155-only/project",
                    "expected_git_identity":"git155","mapping_revision":1}}
            }}}).to_string(), "/tmp/remote155-registry.json").unwrap();
            task.cwd = placement.execution_cwd.clone();
            task.remote_placement = Some(placement);
            task.remote_prepared = true;
            if !started {
                st.delegation_store.save(&task).unwrap();
                let (_writer, child_reader) =
                    crate::daemon::presence_tests::control_socket("rq155");
                return Self {
                    state: Arc::new(Mutex::new(st)),
                    task,
                    parent_reader,
                    child_reader,
                    _config: config,
                    _managed: rx,
                };
            }
            let now = unix_timestamp();
            let order = crate::fleet::SpawnOrder {
                remote_placement: task.remote_placement.clone(),
                posture: None,
                agent_type: "native-test".into(),
                project: None,
                requested_name: Some(task.child.clone()),
                cwd: PathBuf::from(&task.cwd),
                persistent: true,
                command_id: task.task_id.clone(),
                issued_at: now,
                deadline_at: now + 300,
                ownership: Some(crate::fleet::SpawnOwnership {
                    parent_instance_id: task.owner_instance.clone(),
                    parent_execution_id: task.parent_execution_id.clone(),
                    objective_id: None,
                    delegation_id: Some(task.task_id.clone()),
                    project: None,
                    role: "native_delegate".into(),
                    max_children: Some(16),
                    max_depth: Some(8),
                }),
            };
            let crate::fleet::SpawnSubmission::Start(lease) =
                st.fleet.request_spawn(&order, now).unwrap()
            else {
                panic!("lease remote155")
            };
            st.fleet
                .mark_starting(&lease, now, &task.definition)
                .unwrap();
            task.child_instance = Some(lease.instance_id.clone());
            st.managed_by_instance
                .insert(lease.instance_id.clone(), lease.command_id.clone());
            st.managed_spawns.insert(
                lease.command_id.clone(),
                ManagedSpawnRecord {
                    lease: lease.clone(),
                    agent_type: "native-test".into(),
                    requester_conns: Vec::new(),
                    wrapper_conn: None,
                    stop: Arc::new(ManagedStopControl::new()),
                },
            );
            (lease, task.definition.clone())
        } else {
            spawn_task(&mut st, &mut task).unwrap();
            let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
                panic!("Start préparé attendu, sans processus")
            };
            (prepared.lease, *prepared.resolved_definition)
        };
        let (writer, child_reader) = crate::daemon::presence_tests::control_socket("rq155");
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "child155",
                2,
                "native-test".into(),
                lease.name.clone(),
                Some(host),
                Some(definition.protocol.clone()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".into()),
                Some(lease.instance_id.clone()),
                None,
                false,
                Some(true),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
        st.connections.insert("child155".into(), writer);
        st.fleet
            .register_connected(&lease, &lease.instance_id, unix_timestamp())
            .unwrap();
        st.managed_spawns
            .get_mut(&task.task_id)
            .unwrap()
            .wrapper_conn = Some("child155".into());
        task.state = "working".into();
        task.mission_deadline_at = Some(unix_timestamp() + 7200);
        st.store
            .create_request(&task.mission, &task.owner, &task.child, 7200)
            .unwrap();
        st.delegation_store.save(&task).unwrap();
        assert_eq!(
            live_connection_identity(&st, "child155"),
            Some((task.child.clone(), task.child_instance.clone().unwrap()))
        );
        Self {
            state: Arc::new(Mutex::new(st)),
            task,
            parent_reader,
            child_reader,
            _config: config,
            _managed: rx,
        }
    }

    fn question(&self, id: i64) -> ChildProviderRequest {
        ChildProviderRequest {
            version: 1,
            provider_session: "sess-child155".into(),
            turn_id: "turn155".into(),
            provider_request_id: ChildProviderRequestId::Number(id),
            kind: ChildRequestKind::UserInput,
            details: json!({"method":"item/tool/requestUserInput","duration":"once","complete":true,"secret":false,
                "effects":{"command":false,"network":false,"file_write":false},"paths_cited":[],
                "questions":[{"id":"q1","header":"Choix","question":"Quelle couleur ?","isSecret":false}],"reason":"Question fixture"}),
            offered: vec![
                ChildRequestDecision::Accept,
                ChildRequestDecision::Decline,
                ChildRequestDecision::Cancel,
            ],
            question_ids: vec!["q1".into()],
            truncated: false,
        }
    }

    fn approval(&self, id: i64) -> ChildProviderRequest {
        let mut request = self.question(id);
        request.kind = ChildRequestKind::ApprovalCommand;
        request.question_ids.clear();
        request.details = json!({"method":"item/commandExecution/requestApproval","kind":"command","duration":"once",
            "complete":true,"secret":false,"effects":{"command":true,"network":false,"file_write":false},
            "command":"printf fixture155","paths_cited":[],"reason":"Commande fixture"});
        request
    }

    fn receive(&self, request: ChildProviderRequest) -> Option<DaemonToWrapper> {
        handle_wrapper_message(
            "child155",
            WrapperToDaemon::ChildRequest { request },
            &self.state,
        )
    }

    fn row(&self, request: &ChildProviderRequest) -> ChildRequest {
        let st = self.state.lock().unwrap();
        let id:String = st.delegation_store.test_connection().query_row(
            "SELECT request_id FROM native_child_requests WHERE task_id=?1 AND provider_session=?2 AND turn_id=?3 AND provider_request_id=?4",
            rusqlite::params![self.task.task_id, request.provider_session, request.turn_id, serde_json::to_string(&request.provider_request_id).unwrap()],
            |row| row.get(0)).expect("une trame liée doit avoir une ligne persistée");
        st.delegation_store.get_child_request(&id).unwrap().unwrap()
    }

    fn decide(&self, row: &ChildRequest) {
        assert!(
            self.state
                .lock()
                .unwrap()
                .delegation_store
                .decide_child_request(
                    &row.request_id,
                    "accept",
                    if row.kind == "user_input" {
                        Some("{\"q1\":[\"bleu\"]}")
                    } else {
                        None
                    },
                    if row.verdict == "human" {
                        "human"
                    } else {
                        "parent"
                    },
                    if row.verdict == "human" {
                        "accepted_by_human"
                    } else {
                        "answered_by_parent"
                    },
                    unix_timestamp()
                )
                .unwrap()
        );
    }

    fn frames(&mut self) -> Vec<DaemonToWrapper> {
        self.child_reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        let mut frames = Vec::new();
        loop {
            let mut line = String::new();
            match self.child_reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => frames.push(decode(line.trim()).unwrap()),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("socket enfant155 : {error}"),
            }
        }
        frames
    }

    fn expired(&self, row: &ChildRequest) {
        self.state
            .lock()
            .unwrap()
            .delegation_store
            .test_connection()
            .execute(
                "UPDATE native_child_requests SET deadline_at=?2 WHERE request_id=?1",
                rusqlite::params![row.request_id, unix_timestamp() - 1],
            )
            .unwrap();
    }
}

#[test]
fn remote162_demande155_recontrole_worker_hors_verrou_et_epoch_avant_reponse() {
    use bridget_transport::protocol::*;
    for replace_epoch in [false, true] {
        let mut fixture = RequestFixture155::with_remote(
            if replace_epoch {
                "r155-epoch"
            } else {
                "r155-ok"
            },
            true,
        );
        let request = fixture.question(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        assert_eq!(row.verdict, "parent");
        fixture.decide(&row);
        let (writer, mut reader) = crate::daemon::presence_tests::control_socket("rw155");
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let placement = fixture.task.remote_placement.clone().unwrap();
        {
            let mut st = fixture.state.lock().unwrap();
            st.connections.insert("worker155".into(), writer);
            st.remote_workers.insert(
                placement.machine_id.clone(),
                RemoteWorkerConnection {
                    identity: WorkerIdentity {
                        machine_id: placement.machine_id.clone(),
                        worker_id: placement.worker_id.clone(),
                        provisioning_revision: 1,
                    },
                    conn_id: "worker155".into(),
                    epoch: "epoch155".into(),
                    capabilities: Vec::new(),
                    last_contact: Instant::now(),
                    last_contact_at: unix_timestamp(),
                    contact: WorkerContactState::Connected,
                },
            );
        }
        let child_writer = fixture.state.lock().unwrap().connections["child155"].clone();
        let child_gate = child_writer.lock().unwrap();
        let shared = fixture.state.clone();
        let worker = std::thread::spawn(move || {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = decode(line.trim()).unwrap() else {
                panic!("Reconcile attendu")
            };
            assert!(matches!(
                request.request,
                WorkerRequestOperation::Reconcile {}
            ));
            assert!(shared.try_lock().is_ok(), "RPC émis sans verrou métier");
            let response = WorkerResponse {
                version: 1,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                command: request.command,
                response: WorkerResponseOperation::Observation {
                    observation: WorkerObservation {
                    startup_failure: None,
                        phase: WorkerCommandPhase::Released,
                        native_ready: true,
                        exit_code: None,
                        observed_at: unix_timestamp(),
                        workspace: None,
                        permission_digest: Some(placement.permission_digest),
                    },
                },
            };
            handle_worker_message(
                &shared,
                "worker155",
                WrapperToDaemon::WorkerResponse { response },
            );
            if replace_epoch {
                shared
                    .lock()
                    .unwrap()
                    .remote_workers
                    .get_mut("cartae155")
                    .unwrap()
                    .epoch = "replaced155".into();
            }
        });
        let tick_state = fixture.state.clone();
        let tick = std::thread::spawn(move || native_child_requests::tick(&tick_state));
        worker.join().unwrap();
        drop(child_gate);
        tick.join().unwrap();
        let answers = fixture
            .frames()
            .into_iter()
            .filter(|frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { .. }))
            .count();
        assert_eq!(answers, usize::from(!replace_epoch));
        assert_eq!(fixture.row(&request).delivery_state, "sent");
        {
            let mut st = fixture.state.lock().unwrap();
            remote_worker_disconnected(&mut st, "worker155");
            let seq = || {
                st.delegation_store
                    .test_connection()
                    .query_row(
                        "SELECT seq FROM native_delegation_projection_meta WHERE singleton=1",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .unwrap()
            };
            let first = seq();
            remote_worker_disconnected(&mut st, "worker155");
            let second = st
                .delegation_store
                .test_connection()
                .query_row(
                    "SELECT seq FROM native_delegation_projection_meta WHERE singleton=1",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap();
            assert_eq!(
                first, second,
                "une seconde fermeture stale ne change pas seq"
            );
        }
        let approval = fixture.approval(2);
        fixture.receive(approval.clone());
        let human = fixture.row(&approval);
        assert_eq!(human.verdict, "human");
        let denied = handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: human.request_id,
                decision: ChildRequestDecision::Accept,
                answer: None,
            },
            &mut fixture.state.lock().unwrap(),
        );
        assert_eq!(denied.unwrap_err(), "parent_cannot_approve");
    }
}

#[test]
fn remote162_release_relit_droits_parent_apres_start_et_refuse_preuve_t3_indisponible() {
    use bridget_transport::protocol::*;
    for change in ["readonly", "unavailable", "t3_unavailable"] {
        let fixture = RequestFixture155::with_remote_start("r155-fresh", true, false);
        let parent = fixture
            .task
            .permission_snapshot
            .as_ref()
            .unwrap()
            .parent
            .clone();
        let prepared = WorkerPrepared {
            placement: fixture.task.remote_placement.clone().unwrap(),
            workspace: None,
            token: "prepared162".into(),
            definition: fixture.task.definition.clone(),
            child_policy: fixture
                .task
                .permission_snapshot
                .as_ref()
                .unwrap()
                .child_policy
                .clone(),
            effective_posture: fixture.task.effective_posture.unwrap(),
            prepared_at: unix_timestamp(),
        };
        let (writer, mut reader) = crate::daemon::presence_tests::control_socket("wf162");
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let worker = remote_worker_identity(&prepared.placement);
        {
            let mut st = fixture.state.lock().unwrap();
            st.connections.insert("worker155".into(), writer);
            st.remote_workers.insert(
                worker.machine_id.clone(),
                RemoteWorkerConnection {
                    identity: worker,
                    conn_id: "worker155".into(),
                    epoch: "epoch155".into(),
                    capabilities: Vec::new(),
                    last_contact: Instant::now(),
                    last_contact_at: unix_timestamp(),
                    contact: WorkerContactState::Connected,
                },
            );
        }
        let shared = fixture.state.clone();
        let server = std::thread::spawn(move || {
            for step in 0..2 {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let DaemonToWrapper::WorkerRequest { request } = decode(line.trim()).unwrap()
                else {
                    panic!("requête worker")
                };
                let response = match request.request {
                    WorkerRequestOperation::Start { .. } if step == 0 => {
                        let mut st = shared.lock().unwrap();
                        if change == "readonly" {
                            let (fact, _) = st.native_permission_facts.get_mut("conn-1").unwrap();
                            fact.revision += 1;
                            fact.provider_policy["sandbox_policy"] = json!({"type":"readOnly"});
                            fact.runtime_mode = "read-only".into();
                        } else if change == "unavailable" {
                            st.native_permission_facts.remove("conn-1");
                            st.native_permission_refusals
                                .insert("conn-1".into(), "permission_source_unavailable".into());
                        }
                        WorkerResponseOperation::ReadyToRelease {
                            ready_at: unix_timestamp(),
                        }
                    }
                    WorkerRequestOperation::ReserveResultSlot { slot } if step == 1 => {
                        WorkerResponseOperation::ResultSlotReserved {
                            slot,
                            slot_id: "slot162".into(),
                        }
                    }
                    other => panic!("aucun Release autorisé : {other:?}"),
                };
                assert!(
                    handle_worker_message(
                        &shared,
                        "worker155",
                        WrapperToDaemon::WorkerResponse {
                            response: WorkerResponse {
                                version: 1,
                                correlation_id: request.correlation_id,
                                connection_epoch: request.connection_epoch,
                                worker: request.worker,
                                command: request.command,
                                response
                            }
                        }
                    )
                    .is_none()
                );
            }
            reader
        });
        let proof = NativePermissionProof {
            endpoint: "http://invalid.example/mcp".into(),
            authorization: IdentityCredential::new("untrusted-fixture".into()),
        };
        let result = start_remote_task(
            &fixture.state,
            &fixture.task.task_id,
            &prepared,
            &parent,
            "conn-1",
            (change == "t3_unavailable").then_some("thread162"),
            (change == "t3_unavailable").then_some(&proof),
        );
        let reason = result.unwrap_err();
        assert!(
            matches!(
                reason.as_str(),
                "permission_changed"
                    | "permission_source_unavailable"
                    | "permission_attestation_unavailable"
            ),
            "{reason}"
        );
        let mut reader = server.join().unwrap();
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        let mut line = String::new();
        assert!(
            reader.read_line(&mut line).is_err(),
            "pas de permis après modification des droits"
        );
        assert_eq!(
            fixture
                .state
                .lock()
                .unwrap()
                .delegation_store
                .get(&fixture.task.task_id)
                .unwrap()
                .unwrap()
                .state,
            "starting"
        );
    }
}

#[test]
fn remote162_remise_refuse_fait_natif_change_sans_exiger_parent_absent_ou_t3() {
    let fixture = RequestFixture155::with_remote("r155-live", true);
    let mut st = fixture.state.lock().unwrap();
    assert!(recheck_remote_dispatch(&st, &fixture.task, &fixture.task).is_ok());
    st.native_permission_facts
        .get_mut("conn-1")
        .unwrap()
        .0
        .revision += 1;
    assert_eq!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).unwrap_err(),
        "permission_not_inherited"
    );
    let instance = st.conn_instances.remove("conn-1").unwrap();
    assert!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).is_ok(),
        "parent absent : autorité admise conservée"
    );
    st.conn_instances.insert("conn-1".into(), instance);
    st.native_permission_facts.remove("conn-1");
    st.native_permission_refusals
        .insert("conn-1".into(), "permission_source_unavailable".into());
    assert_eq!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).unwrap_err(),
        "permission_source_unavailable"
    );
    let mut t3 = fixture.task.clone();
    t3.permission_snapshot.as_mut().unwrap().parent.source = "provider_turn".into();
    assert!(
        recheck_remote_dispatch(&st, &t3, &t3).is_ok(),
        "un refus wrapper ne remplace pas l'autorité T3 déjà admise"
    );
}

#[test]
fn remote162_prepare_posture_omise_conserve_heritage_et_refuse_null() {
    use bridget_transport::protocol::WorkerPrepare;
    let fixture = RequestFixture155::with_remote_start("r155-posture", true, false);
    let task = &fixture.task;
    let placement = task.remote_placement.as_ref().unwrap();
    let prepare = WorkerPrepare {
        machine_id: placement.machine_id.clone(),
        project_id: placement.project_id.clone(),
        provisioning_revision: 1,
        mapping_revision: 1,
        origin_host: placement.origin_host.clone(),
        origin_cwd: placement.origin_cwd.clone(),
        source_root: placement.origin_cwd.clone(),
        remote_root: placement.execution_cwd.clone(),
        source_root_identity: placement.source_root_identity.clone(),
        relative_cwd: ".".into(),
        base_commit: None,
        agent_type: "native-test".into(),
        model: "custom/model".into(),
        effort: None,
        posture: None,
        parent: task.permission_snapshot.as_ref().unwrap().parent.clone(),
        workspace: None,
    };
    let mut json = serde_json::to_value(&prepare).unwrap();
    assert!(json.get("posture").is_none());
    assert_eq!(
        serde_json::from_value::<WorkerPrepare>(json.clone())
            .unwrap()
            .posture,
        None
    );
    json["posture"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<WorkerPrepare>(json.clone()).is_err());
    json["posture"] = serde_json::json!(SpawnPosture::Discovery);
    assert_eq!(
        serde_json::from_value::<WorkerPrepare>(json)
            .unwrap()
            .posture,
        Some(SpawnPosture::Discovery)
    );
}

#[test]
fn remote162_arret_atteste_avant_connected_cloture_demarrage_apres_reprise() {
    use bridget_transport::protocol::*;
    for phase in [WorkerCommandPhase::Terminal, WorkerCommandPhase::Refused] {
        let fixture = RequestFixture155::with_remote("r155-stop", true);
        let execution = format!("execution-{}", fixture.task.mission);
        let (lease, commands_before) = {
            let st = fixture.state.lock().unwrap();
            let mut task = st
                .delegation_store
                .get(&fixture.task.task_id)
                .unwrap()
                .unwrap();
            task.state = "starting".into();
            task.mission_issued_at = None;
            st.delegation_store.save(&task).unwrap();
            st.execution_store
                .admit_starting(&task.mission, &execution, &task.child, unix_timestamp())
                .unwrap();
            (
                st.managed_spawns[&task.task_id].lease.clone(),
                st.idempotency
                    .spawn_commands(&st.fleet.supervisor_scope())
                    .unwrap()
                    .len(),
            )
        };
        // Une connexion absente ne prouve pas l'arrêt, même si le wrapper avait été connu.
        fixture
            .state
            .lock()
            .unwrap()
            .conn_instances
            .remove("child155");
        let (writer, mut reader) = crate::daemon::presence_tests::control_socket("rs162");
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let placement = fixture.task.remote_placement.as_ref().unwrap();
        {
            let mut st = fixture.state.lock().unwrap();
            st.connections.insert("worker155".into(), writer);
            st.remote_workers.insert(
                placement.machine_id.clone(),
                RemoteWorkerConnection {
                    identity: remote_worker_identity(placement),
                    conn_id: "worker155".into(),
                    epoch: "epoch155".into(),
                    capabilities: Vec::new(),
                    last_contact: Instant::now(),
                    last_contact_at: unix_timestamp(),
                    contact: WorkerContactState::Connected,
                },
            );
        }
        tick(&fixture.state); // Rend la main sans attendre une réponse du worker.
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let DaemonToWrapper::WorkerRequest { request } = decode(line.trim()).unwrap() else {
            panic!("sonde Reconcile attendue")
        };
        assert!(matches!(
            request.request,
            WorkerRequestOperation::Reconcile {}
        ));
        let response = WorkerResponse {
            version: 1,
            correlation_id: request.correlation_id,
            connection_epoch: request.connection_epoch,
            worker: request.worker,
            command: request.command,
            response: WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    startup_failure: None,
                    phase: WorkerCommandPhase::Unknown,
                    native_ready: false,
                    exit_code: None,
                    observed_at: unix_timestamp(),
                    workspace: None,
                    permission_digest: None,
                },
            },
        };
        assert!(
            handle_worker_message(
                &fixture.state,
                "worker155",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none()
        );
        {
            let mut st = fixture.state.lock().unwrap();
            assert_eq!(
                st.delegation_store
                    .get(&fixture.task.task_id)
                    .unwrap()
                    .unwrap()
                    .state,
                "starting"
            );
            assert!(
                st.fleet
                    .recovery_candidates()
                    .iter()
                    .any(|candidate| candidate.lease == lease)
            );
            st.managed_spawns.clear();
            st.managed_by_instance.clear();
            assert!(
                reserve_managed_recoveries(&mut st, unix_timestamp())
                    .unwrap()
                    .is_empty()
            );
            assert!(st.managed_spawns.contains_key(&lease.command_id));
        }
        let mut stopped = response.clone();
        if let WorkerResponseOperation::Observation { observation } = &mut stopped.response {
            observation.phase = phase;
        }
        let mut stale = stopped.clone();
        stale.connection_epoch = "obsolete".into();
        assert!(matches!(
            handle_worker_message(
                &fixture.state,
                "worker155",
                WrapperToDaemon::WorkerResponse { response: stale }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::StaleConnection
            })
        ));
        let mut wrong = stopped.clone();
        wrong.command.generation += 1;
        assert!(matches!(
            handle_worker_message(
                &fixture.state,
                "worker155",
                WrapperToDaemon::WorkerResponse { response: wrong }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::CommandMismatch
            })
        ));
        let mut wrong_task = stopped.clone();
        wrong_task.command.task_id = Uuid::new_v4().to_string();
        assert!(matches!(
            handle_worker_message(
                &fixture.state,
                "worker155",
                WrapperToDaemon::WorkerResponse {
                    response: wrong_task
                }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::CommandMismatch
            })
        ));
        assert!(
            handle_worker_message(
                &fixture.state,
                "worker155",
                WrapperToDaemon::WorkerResponse { response: stopped }
            )
            .is_none()
        );
        {
            let mut st = fixture.state.lock().unwrap();
            assert_eq!(
                st.fleet
                    .desired_entry(&fixture.task.child)
                    .unwrap()
                    .unwrap()
                    .lifecycle_state,
                DesiredLifecycleState::Stopped
            );
            // Simulation de reprise : la décision ne doit pas dépendre de ces caches.
            st.managed_spawns.clear();
            st.managed_by_instance.clear();
            st.managed_terminal_instances.clear();
            st.remote_workers.clear();
        }
        tick(&fixture.state);
        {
            let st = fixture.state.lock().unwrap();
            let task = st
                .delegation_store
                .get(&fixture.task.task_id)
                .unwrap()
                .unwrap();
            assert_eq!(task.state, "failed");
            assert_eq!(task.error.as_deref(), Some("remote_start_failed"));
            assert_eq!(
                st.execution_store
                    .execution_snapshot(&execution)
                    .unwrap()
                    .unwrap()
                    .state,
                "unreachable"
            );
            assert!(
                task.cleanup_done,
                "la preuve Stopped durable suffit sans nouveau RPC"
            );
            assert_eq!(
                st.idempotency
                    .spawn_commands(&st.fleet.supervisor_scope())
                    .unwrap()
                    .len(),
                commands_before
            );
        }
        let result = wait_tasks(
            "conn-1",
            &[fixture.task.task_id.clone()],
            Some(1),
            None,
            &fixture.state,
            None,
            Duration::from_millis(1),
        )
        .unwrap();
        assert_eq!(result["ready"][0]["task_id"], fixture.task.task_id);
        assert_eq!(result["timed_out"], false);
    }
}

#[test]
fn remote162_quota157_portee_instance_avant_reprise_distante() {
    use bridget_transport::protocol::{RateLimitFact, RateLimitSource};
    let fixture = RequestFixture155::with_remote("r155-quota", true);
    let mut st = fixture.state.lock().unwrap();
    assert!(recheck_remote_dispatch(&st, &fixture.task, &fixture.task).is_ok());
    let now = unix_timestamp();
    let fact = RateLimitFact {
        credits: None,
        window: "five_hour".into(),
        status: "rate_limit_reached".into(),
        resets_at: Some(now + 600),
        used_percent: Some(100),
    };
    st.provider_limits.record(
        "autre-moteur",
        fact.clone(),
        RateLimitSource::CodexAppServer,
        now,
    );
    assert!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).is_ok(),
        "autre type indépendant"
    );
    // Un autre compte du même profil ne bloque ni n'autorise cet enfant.
    st.provider_limits.record_scoped(
        "native-test",
        "other-account",
        fact.clone(),
        RateLimitSource::CodexAppServer,
        now,
    );
    assert!(recheck_remote_dispatch(&st, &fixture.task, &fixture.task).is_ok());
    let child_instance = fixture.task.child_instance.as_deref().unwrap();
    st.provider_limits.record_scoped(
        "native-test",
        child_instance,
        fact.clone(),
        RateLimitSource::CodexAppServer,
        now,
    );
    assert_eq!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).unwrap_err(),
        "provider_quota_exhausted"
    );
    let credited = RateLimitFact {
        credits: Some(bridget_transport::protocol::CodexCredits {
            has_credits: Some(true),
            unlimited: Some(false),
            spend_control_reached: false,
        }),
        ..fact.clone()
    };
    st.provider_limits.record_scoped(
        "native-test",
        "other-account",
        credited.clone(),
        RateLimitSource::CodexAppServer,
        now,
    );
    assert_eq!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).unwrap_err(),
        "provider_quota_exhausted"
    );
    st.provider_limits.record_scoped(
        "native-test",
        child_instance,
        credited,
        RateLimitSource::CodexAppServer,
        now,
    );
    assert!(recheck_remote_dispatch(&st, &fixture.task, &fixture.task).is_ok());
    st.provider_limits.record_scoped(
        "native-test",
        child_instance,
        RateLimitFact {
            window: "account/credits".into(),
            status: "observed".into(),
            resets_at: None,
            used_percent: None,
            credits: None,
        },
        RateLimitSource::CodexAppServer,
        now + 1,
    );
    assert_eq!(
        recheck_remote_dispatch(&st, &fixture.task, &fixture.task).unwrap_err(),
        "provider_quota_exhausted"
    );
}

#[test]
fn native155_request_tuple_idempotence_conflit_tour_et_identite() {
    let fixture = RequestFixture155::new("tuple155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let first = fixture.row(&request);
    assert_eq!((&first.state[..], &first.verdict[..]), ("open", "parent"));
    fixture.receive(request.clone());
    assert_eq!(fixture.row(&request), first);
    let mut different = request.clone();
    different.details["reason"] = json!("Autre contenu");
    assert!(
        matches!(fixture.receive(different), Some(DaemonToWrapper::Nack { reason, .. }) if reason == "request_conflict")
    );
    let mut next = request.clone();
    next.turn_id = "turn156".into();
    fixture.receive(next.clone());
    assert_ne!(fixture.row(&next).request_id, first.request_id);
    let mut numeric_string = request.clone();
    numeric_string.provider_request_id = ChildProviderRequestId::String("1".into());
    fixture.receive(numeric_string.clone());
    assert_ne!(
        fixture.row(&numeric_string).request_id,
        first.request_id,
        "ID JSON typé conservé"
    );
    let reply = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ChildRequest {
            request: request.clone(),
        },
        &fixture.state,
    );
    assert!(
        matches!(reply, Some(DaemonToWrapper::Nack { reason, .. }) if reason == "request_unlinked")
    );
    let st = fixture.state.lock().unwrap();
    assert_eq!(
        st.delegation_store
            .child_request_counts(&fixture.task.task_id)
            .unwrap(),
        (3, 3)
    );
    let mut terminal = fixture.task.clone();
    terminal.state = "failed".into();
    st.delegation_store.save(&terminal).unwrap();
    drop(st);
    assert!(
        matches!(fixture.receive(fixture.question(9)),Some(DaemonToWrapper::Nack{reason,..}) if reason == "request_unlinked"),
        "tâche terminale rejetée avant admission"
    );
}

#[test]
fn native155_request_verdicts_et_forbidden_sans_inbox() {
    for (label, expected) in [
        ("parent155", "parent"),
        ("human155", "human"),
        ("forbidden155", "forbidden"),
    ] {
        let fixture = RequestFixture155::new(label);
        let mut request = if expected == "parent" {
            fixture.question(1)
        } else {
            fixture.approval(1)
        };
        if expected == "forbidden" {
            request.details["duration"] = json!("session");
        }
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        assert_eq!(row.verdict, expected);
        let st = fixture.state.lock().unwrap();
        let count = crate::human_inbox::open_count(st.store.connection()).unwrap();
        if expected == "human" {
            assert_eq!(count, 1);
            assert!(row.inbox_item_id.is_some());
        } else {
            assert_eq!(count, 0);
        }
        if expected == "forbidden" {
            assert_eq!(row.state, "refused");
            assert_eq!(row.outcome.as_deref(), Some("refused:duration_unsupported"));
        }
    }
}

#[test]
fn native155_request_budgets_ouverte_et_totale() {
    for total_limit in [false, true] {
        let mut fixture = RequestFixture155::new(if total_limit { "total155" } else { "open155" });
        if total_limit {
            // Runtime r2 : ID fournisseur 8 refusé par le disjoncteur149
            // (limite 8 échanges / 180 s), sans ledger ni trame parent.
            // Ce montage teste le budget155 de 32 lignes.
            fixture.state.lock().unwrap().circuit_breaker =
                CircuitBreaker::new(fixture._config.circuit_breaker_window, 64);
        }
        let accepted = if total_limit { 32 } else { 4 };
        for id in 0..accepted {
            let request = fixture.question(id);
            fixture.receive(request.clone());
            let row = fixture.row(&request);
            assert_eq!(row.state, "open");
            // Le parent privé consomme chaque avis ; il ne doit pas saturer
            // sa socket pendant les32 demandes du scénario de budget.
            let _ = crate::daemon::presence_tests::read_control(&mut fixture.parent_reader);
            if total_limit {
                assert!(
                    fixture
                        .state
                        .lock()
                        .unwrap()
                        .delegation_store
                        .close_child_request(
                            &row.request_id,
                            "cancelled",
                            "cancelled_by_parent",
                            unix_timestamp()
                        )
                        .unwrap()
                );
            }
        }
        let refused = fixture.question(accepted);
        let reply = fixture.receive(refused.clone());
        let mut frames = fixture.frames();
        if let Some(reply) = reply {
            frames.push(reply);
        }
        let refusal = frames
            .iter()
            .find_map(|frame| {
                if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                    (answer.provider_request_id == refused.provider_request_id).then_some(answer)
                } else {
                    None
                }
            })
            .expect("refus wire lié au tuple exact, pas un Nack qui laisse l'attente ouverte");
        assert_eq!(refusal.outcome, "refused:request_budget_exhausted");
        assert_eq!(refusal.decision, ChildRequestDecision::Decline);
        assert_eq!(refusal.provider_session, refused.provider_session);
        assert_eq!(refusal.turn_id, refused.turn_id);
        let counts = fixture
            .state
            .lock()
            .unwrap()
            .delegation_store
            .child_request_counts(&fixture.task.task_id)
            .unwrap();
        assert!(
            counts.0 <= 4 && counts.1 <= 32,
            "plafonds persistés {counts:?}"
        );
    }
}

#[test]
fn native155_request_echeances_fenetres_et_decision_tardive() {
    for (human, window) in [(false, 600), (true, 3600)] {
        let fixture = RequestFixture155::new("window155");
        let request = if human {
            fixture.approval(1)
        } else {
            fixture.question(1)
        };
        let before = unix_timestamp();
        fixture.receive(request.clone());
        let after = unix_timestamp();
        let row = fixture.row(&request);
        assert!((before + window..=after + window).contains(&row.deadline_at));
        fixture.expired(&row);
        let response = {
            let mut st = fixture.state.lock().unwrap();
            handle_locked(
                "conn-1",
                serde_json::from_value(
                    json!({"operation":"respond","task_id":fixture.task.task_id,
                "request_id":row.request_id,"decision":"accept","answer":{"q1":["bleu"]}}),
                )
                .unwrap(),
                &mut st,
            )
        };
        assert_eq!(
            response.unwrap_err(),
            "request_expired",
            "échéance avant route"
        );
    }
    for remaining in [29, 90] {
        let fixture = RequestFixture155::new("clip155");
        let mission_deadline = unix_timestamp() + remaining;
        {
            let st = fixture.state.lock().unwrap();
            let mut task = fixture.task.clone();
            task.mission_deadline_at = Some(mission_deadline);
            st.delegation_store.save(&task).unwrap();
        }
        let request = fixture.question(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        if remaining == 29 {
            assert_eq!(row.outcome.as_deref(), Some("refused:deadline_too_close"));
        } else {
            assert_eq!(row.deadline_at, mission_deadline - 30);
        }
    }
}

#[test]
fn native155_request_avis_idempotent() {
    let mut fixture = RequestFixture155::new("notice155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.receive(request.clone());
    native_delegation::tick(&fixture.state);
    let st = fixture.state.lock().unwrap();
    let messages = st.store.recent_messages(100).unwrap();
    let expected = format!("native-request-{}", row.request_id);
    let notices = messages
        .iter()
        .filter(|entry| entry.id == expected)
        .collect::<Vec<_>>();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].target, fixture.task.owner);
    assert!(notices[0].body.contains(&row.request_id));
    drop(st);
    fixture
        .parent_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let mut system_notice = false;
    loop {
        let mut line = String::new();
        match fixture.parent_reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let frame: DaemonToWrapper = decode(line.trim()).unwrap();
                let message = match frame {
                    DaemonToWrapper::Deliver { message, .. }
                    | DaemonToWrapper::DeliverExecution { message, .. }
                    | DaemonToWrapper::DeliverIdempotent { message, .. } => Some(message),
                    _ => None,
                };
                if let Some(message) = message.filter(|message| message.id == expected) {
                    assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
                    system_notice = true;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                break;
            }
            Err(error) => panic!("avis parent155 : {error}"),
        }
    }
    assert!(system_notice, "avis système réellement remis au parent");
}

#[test]
fn native155_human_depot_idempotent_apres_panne() {
    let fixture = RequestFixture155::new("deposit155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item = row.inbox_item_id.unwrap();
    {
        let st = fixture.state.lock().unwrap();
        st.delegation_store
            .test_connection()
            .execute(
                "UPDATE native_child_requests SET inbox_item_id=NULL WHERE request_id=?1",
                [&row.request_id],
            )
            .unwrap();
    }
    fixture.receive(request.clone());
    native_delegation::tick(&fixture.state);
    assert_eq!(
        fixture.row(&request).inbox_item_id.as_deref(),
        Some(item.as_str())
    );
    let st = fixture.state.lock().unwrap();
    assert_eq!(
        crate::human_inbox::open_count(st.store.connection()).unwrap(),
        1
    );
    assert_eq!(
        crate::human_inbox::get(st.store.connection(), &item)
            .unwrap()
            .unwrap()
            .occurrences,
        1,
        "pas de second dépôt pour réparer le lien"
    );
}

#[test]
fn native155_human_item_deja_resolu_reutilise() {
    let fixture = RequestFixture155::new("resolved155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item = row.inbox_item_id.unwrap();
    {
        let st = fixture.state.lock().unwrap();
        crate::human_inbox::resolve(
            st.store.connection(),
            &item,
            "decline",
            "humain",
            unix_timestamp(),
        )
        .unwrap()
        .unwrap();
        st.delegation_store
            .test_connection()
            .execute(
                "UPDATE native_child_requests SET inbox_item_id=NULL WHERE request_id=?1",
                [&row.request_id],
            )
            .unwrap();
    }
    fixture.receive(request.clone());
    assert_eq!(
        fixture.row(&request).inbox_item_id.as_deref(),
        Some(item.as_str())
    );
    let st = fixture.state.lock().unwrap();
    assert_eq!(
        crate::human_inbox::open_count(st.store.connection()).unwrap(),
        0
    );
    let count: i64 = st
        .store
        .connection()
        .query_row("SELECT count(*) FROM human_inbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn native155_lifecycle_renvoi_jusqu_a_accuse() {
    let mut fixture = RequestFixture155::new("resend155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    // Bloquer la socket avant le tick : la lecture indépendante observe le
    // commit SENT alors qu'aucune écriture au wrapper n'est encore possible.
    let writer = fixture.state.lock().unwrap().connections["child155"].clone();
    let gate = writer.lock().unwrap();
    let state = fixture.state.clone();
    let blocked_since = Instant::now();
    let worker = std::thread::spawn(move || native_delegation::tick(&state));
    let reader = rusqlite::Connection::open(&fixture._config.db_path).unwrap();
    reader.busy_timeout(Duration::from_millis(50)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut sent_before_write = false;
    while Instant::now() < deadline {
        let persisted = reader.query_row(
            "SELECT delivery_state,sent_at FROM native_child_requests WHERE request_id=?1",
            [&row.request_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?)),
        );
        if matches!(persisted,Ok((state,Some(_))) if state == "sent") {
            sent_before_write = true;
            break;
        }
        if worker.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let sent_before_timeout = fixture.row(&request);
    while !worker.is_finished() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    if !worker.is_finished() {
        drop(gate);
        worker.join().unwrap();
        panic!("le tick doit rendre la main sous2s malgré le writer verrouillé");
    }
    worker.join().unwrap();
    assert!(
        sent_before_write,
        "SENT committé avant write ; lecture bornée2s"
    );
    assert!(
        blocked_since.elapsed() >= Duration::from_secs(1),
        "le verrou couvre l'expiration du budget d'écriture1s"
    );
    assert_eq!(
        fixture.row(&request),
        sent_before_timeout,
        "sans première écriture, SENT reste inchangé et réessayable"
    );
    assert_eq!(sent_before_timeout.delivery_state, "sent");
    assert!(sent_before_timeout.sent_at.is_some());
    assert!(sent_before_timeout.void_reason.is_none());
    assert!(
        fixture.frames().is_empty(),
        "aucune trame enfant avant libération du writer"
    );
    drop(gate);
    native_delegation::tick(&fixture.state);
    let first = fixture
        .frames()
        .into_iter()
        .filter_map(|frame| {
            if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                Some(answer)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(first.len(), 1);
    assert_eq!(fixture.row(&request).delivery_state, "sent");
    assert!(fixture.row(&request).sent_at.is_some());
    native_delegation::tick(&fixture.state);
    let second = fixture
        .frames()
        .into_iter()
        .filter_map(|frame| {
            if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                Some(answer)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(second, first);
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: row.request_id.clone(),
            applied: true,
            reason: None,
        },
        &fixture.state,
    );
    assert_eq!(
        fixture.row(&request).delivery_state,
        "sent",
        "autre connexion ignorée"
    );
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: row.request_id.clone(),
            applied: true,
            reason: None,
        },
        &fixture.state,
    );
    assert_eq!(fixture.row(&request).delivery_state, "delivered");
    native_delegation::tick(&fixture.state);
    assert!(
        !fixture
            .frames()
            .iter()
            .any(|frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { .. }))
    );
    fixture.receive(request.clone());
    assert_eq!(fixture.row(&request).delivery_state, "delivered");
    assert_eq!(
        fixture
            .state
            .lock()
            .unwrap()
            .delegation_store
            .child_request_counts(&fixture.task.task_id)
            .unwrap()
            .1,
        1,
        "replay terminal sans nouvelle ligne"
    );
}

#[test]
fn native155_lifecycle_accuse_negatif_annule() {
    let mut fixture = RequestFixture155::new("negative155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    native_delegation::tick(&fixture.state);
    fixture.frames();
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: row.request_id.clone(),
            applied: false,
            reason: Some("closed".into()),
        },
        &fixture.state,
    );
    assert_eq!(fixture.row(&request).delivery_state, "void");
    native_delegation::tick(&fixture.state);
    assert!(
        !fixture
            .frames()
            .iter()
            .any(|f| matches!(f, DaemonToWrapper::ChildRequestAnswer { .. }))
    );
}

#[test]
fn native155_lifecycle_enfant_disparu_accord_sent_reste_unknown() {
    let mut fixture = RequestFixture155::new("lost155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    native_delegation::tick(&fixture.state);
    fixture.frames();
    fixture.state.lock().unwrap().connections.remove("child155");
    native_delegation::tick(&fixture.state);
    let persisted = fixture.row(&request);
    assert_eq!(persisted.delivery_state, "unknown");
    assert_eq!(
        persisted.outcome.as_deref(),
        Some("cancelled_outcome_unknown")
    );
    assert_eq!(persisted.decision.as_deref(), Some("accept"));
}

#[test]
fn native155_lifecycle_plantage_entre_sent_et_envoi() {
    let fixture = RequestFixture155::new("crash155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    {
        let mut st = fixture.state.lock().unwrap();
        assert!(
            st.delegation_store
                .claim_child_request(&row.request_id, unix_timestamp())
                .unwrap()
        );
        native_delegation::prepare_restart(&mut st).unwrap();
    }
    let persisted = fixture.row(&request);
    assert_eq!(persisted.delivery_state, "unknown");
    assert_eq!(
        persisted.outcome.as_deref(),
        Some("cancelled_outcome_unknown")
    );
}

#[test]
fn native155_lifecycle_accord_apres_echeance_jamais_envoye() {
    for sent in [false, true] {
        let mut fixture = RequestFixture155::new("late155");
        let request = fixture.approval(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        fixture.decide(&row);
        if sent {
            native_delegation::tick(&fixture.state);
            fixture.frames();
        }
        fixture.expired(&row);
        native_delegation::tick(&fixture.state);
        let frames = fixture.frames();
        assert!(!frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
        assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)),"fermeture requise même pending invalidé");
        if !sent {
            assert_eq!(fixture.row(&request).delivery_state, "void");
        }
        handle_wrapper_message(
            "child155",
            WrapperToDaemon::ChildRequestClosed {
                request_id: row.request_id.clone(),
                state: ChildRequestClosedState::ClosedBeforeApply,
            },
            &fixture.state,
        );
        assert_eq!(fixture.row(&request).delivery_state, "void");
    }
}

#[test]
fn native155_lifecycle_expiration_refus_et_cloture_none_pending_void() {
    for delivery in ["none", "pending", "void"] {
        for approval in [false, true] {
            let mut fixture = RequestFixture155::new("expire155");
            let request = if approval {
                fixture.approval(1)
            } else {
                fixture.question(1)
            };
            fixture.receive(request.clone());
            let row = fixture.row(&request);
            if delivery != "none" {
                fixture.decide(&row);
            }
            if delivery == "void" {
                fixture
                    .state
                    .lock()
                    .unwrap()
                    .delegation_store
                    .void_pending_child_request(&row.request_id, "cancelled_by_parent")
                    .unwrap();
            }
            fixture.expired(&row);
            native_delegation::tick(&fixture.state);
            let frames = fixture.frames();
            assert!(!frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
            assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)));
            native_delegation::tick(&fixture.state);
            assert!(fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)),"Cancel répété tant que Closed absent");
            handle_wrapper_message(
                "child155",
                WrapperToDaemon::ChildRequestClosed {
                    request_id: row.request_id.clone(),
                    state: ChildRequestClosedState::ClosedBeforeApply,
                },
                &fixture.state,
            );
            native_delegation::tick(&fixture.state);
            assert!(!fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
        }
    }
}

#[test]
fn native155_lifecycle_invalidations_avant_renvoi_et_finalisation() {
    for cause in ["cancel", "end", "pause", "revoke", "instance", "restart"] {
        for sent in [false, true] {
            let mut fixture = RequestFixture155::new("close155");
            let request = fixture.approval(1);
            fixture.receive(request.clone());
            let row = fixture.row(&request);
            fixture.decide(&row);
            if sent {
                native_delegation::tick(&fixture.state);
                fixture.frames();
            }
            {
                let mut st = fixture.state.lock().unwrap();
                let mut task = st
                    .delegation_store
                    .get(&fixture.task.task_id)
                    .unwrap()
                    .unwrap();
                match cause {
                    "cancel" => {
                        task.state = "cancelling".into();
                        st.delegation_store.save(&task).unwrap();
                    }
                    "end" => {
                        task.state = "result_available".into();
                        st.delegation_store.save(&task).unwrap();
                    }
                    "pause" => {
                        let control = crate::referent_control::read(st.store.connection()).unwrap();
                        crate::referent_control::set(
                            st.store.connection(),
                            crate::referent_control::ControlMutation {
                                command_id: "pause155",
                                expected_generation: control.generation,
                                paused: Some(true),
                                auto_objectives_cap: None,
                                reason: Some("fixture"),
                                actor: "humain",
                                now: unix_timestamp(),
                                agent_posture: None,
                                auto_reassignment: None,
                            },
                        )
                        .unwrap()
                        .unwrap();
                    }
                    "revoke" => {
                        st.delegation_store
                            .grant_agent(
                                &task.owner,
                                &task.owner_instance,
                                Path::new(&task.cwd),
                                SpawnPosture::Development,
                                true,
                            )
                            .unwrap();
                    }
                    "instance" => {
                        task.child_instance = Some("other155".into());
                        st.delegation_store.save(&task).unwrap();
                    }
                    "restart" => {
                        native_delegation::prepare_restart(&mut st).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
            native_delegation::tick(&fixture.state);
            let frames = fixture.frames();
            assert!(!frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)),"{cause}/{sent}");
            let persisted = fixture.row(&request);
            if cause == "instance" && sent {
                assert_eq!(
                    persisted.delivery_state, "sent",
                    "ancienne connexion encore vivante : attente de Closed"
                );
                assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)));
                handle_wrapper_message(
                    "child155",
                    WrapperToDaemon::ChildRequestClosed {
                        request_id: row.request_id.clone(),
                        state: ChildRequestClosedState::ClosedBeforeApply,
                    },
                    &fixture.state,
                );
                assert_eq!(fixture.row(&request).delivery_state, "void");
            } else if cause == "restart" || cause == "instance" {
                assert!(
                    matches!(persisted.delivery_state.as_str(), "void" | "unknown"),
                    "{cause}/{sent}"
                );
            } else {
                assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)),"{cause}/{sent} attend Closed");
                if cause == "cancel" {
                    let st = fixture.state.lock().unwrap();
                    let task = st
                        .delegation_store
                        .get(&fixture.task.task_id)
                        .unwrap()
                        .unwrap();
                    assert_ne!(task.state, "cancelled", "aucun état final sans fermeture");
                    assert!(!task.cleanup_done);
                }
            }
        }
    }
}

#[test]
fn native155_lifecycle_annulation_course_closed_avant_ou_apres_application() {
    for already_applied in [false, true] {
        let mut fixture = RequestFixture155::new("race155");
        let request = fixture.approval(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        fixture.decide(&row);
        native_delegation::tick(&fixture.state);
        fixture.frames();
        {
            let st = fixture.state.lock().unwrap();
            let mut task = fixture.task.clone();
            task.state = "cancelling".into();
            st.delegation_store.save(&task).unwrap();
        }
        native_delegation::tick(&fixture.state);
        assert!(
            fixture
                .frames()
                .iter()
                .any(|f| matches!(f, DaemonToWrapper::ChildRequestCancel { .. }))
        );
        handle_wrapper_message(
            "child155",
            WrapperToDaemon::ChildRequestClosed {
                request_id: row.request_id.clone(),
                state: if already_applied {
                    ChildRequestClosedState::AlreadyApplied
                } else {
                    ChildRequestClosedState::ClosedBeforeApply
                },
            },
            &fixture.state,
        );
        assert_eq!(
            fixture.row(&request).delivery_state,
            if already_applied { "delivered" } else { "void" }
        );
        fixture.receive(request.clone());
        native_delegation::tick(&fixture.state);
        assert!(!fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
    }
}

#[test]
fn native155_lifecycle_accord_pending_annulation_puis_reconnexion_void() {
    let mut fixture = RequestFixture155::new("reconnect155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    {
        let st = fixture.state.lock().unwrap();
        let mut task = fixture.task.clone();
        task.state = "cancelling".into();
        st.delegation_store.save(&task).unwrap();
    }
    native_delegation::tick(&fixture.state);
    assert_eq!(fixture.row(&request).delivery_state, "void");
    assert!(fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)));
    {
        let mut st = fixture.state.lock().unwrap();
        st.connections.remove("child155");
        let (writer, reader) = crate::daemon::presence_tests::control_socket("re155");
        let host = st.host.clone();
        let agent_type = st.presences[fixture.task.child_instance.as_deref().unwrap()]
            .agent_type
            .clone();
        // Résultat du montage précédent consigné AVANT désinscription : le
        // retrait du writer seul conserve une route déjà prise (Nack register).
        let original = handle_register_with_channel(
            "child155",
            2,
            "native-test".into(),
            fixture.task.child.clone(),
            Some(host),
            Some("codex_app_server".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            fixture.task.child_instance.clone(),
            None,
            false,
            Some(true),
            &mut st,
            false,
        );
        eprintln!("diagnostic reconnexion155 avant désinscription route : {original:?}");
        st.router.unregister_by_conn("child155");
        let host = st.host.clone();
        let registered = handle_register_with_channel(
            "child155",
            2,
            agent_type,
            fixture.task.child.clone(),
            Some(host),
            Some("codex_app_server".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            fixture.task.child_instance.clone(),
            None,
            false,
            Some(true),
            &mut st,
            false,
        );
        assert!(
            matches!(registered, DaemonToWrapper::Registered { .. }),
            "{registered:?}"
        );
        st.connections.insert("child155".into(), writer);
        fixture.child_reader = reader;
    }
    fixture.receive(request.clone());
    native_delegation::tick(&fixture.state);
    assert!(!fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestClosed {
            request_id: row.request_id.clone(),
            state: ChildRequestClosedState::ClosedBeforeApply,
        },
        &fixture.state,
    );
    assert_eq!(fixture.row(&request).delivery_state, "void");
}

#[path = "native_delegation_respond155_tests.rs"]
mod respond155;

#[path = "native_delegation_chain155_tests.rs"]
mod chain155;

#[path = "native_child_requests_human155_tests.rs"]
mod human155;

#[path = "native_delegation_wait155_tests.rs"]
mod wait155;

#[test]
fn guidance156_mission_interrupt_enfant_refuse() {
    let mut fixture = RequestFixture155::new("guidance156_mission_interrupt_enfant_refuse");
    let mut message = bridget_core::BridgetMessage::new(
        &fixture.task.owner,
        &fixture.task.child,
        "mission guidée156",
    );
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response =
        handle_wrapper_message("conn-1", WrapperToDaemon::Send(message), &fixture.state).unwrap();
    assert!(
        matches!(response,DaemonToWrapper::Nack {reason,..} if reason.starts_with("mission_active"))
    );
    fixture
        .child_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(fixture.child_reader.read_line(&mut line).is_err());
}

#[test]
fn guidance156_mission_steer_enfant_admis() {
    let mut fixture = RequestFixture155::new("guidance156_mission_steer_enfant_admis");
    let mut message = bridget_core::BridgetMessage::new(
        &fixture.task.owner,
        &fixture.task.child,
        "mission guidée156",
    );
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let response =
        handle_wrapper_message("conn-1", WrapperToDaemon::Send(message), &fixture.state).unwrap();
    assert!(matches!(response, DaemonToWrapper::Ack { .. }));
    assert!(
        matches!(crate::daemon::presence_tests::read_control(&mut fixture.child_reader),DaemonToWrapper::Deliver {message,mid_turn_ok:Some(true)} if message.intent==Some(bridget_core::MessageIntent::SteerCurrent))
    );
}

#[test]
fn guidance156_control_parent_mission_steer_origine_humaine_forgee() {
    let mut fixture = RequestFixture155::new("guidance156-native-control");
    let mut st = fixture.state.lock().unwrap();
    st.execution_store
        .record_starting(
            "submission-guidance156",
            "execution-guidance156",
            &fixture.task.child,
            unix_timestamp(),
        )
        .unwrap();
    st.execution_store
        .transition_if_current(
            "execution-guidance156",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            unix_timestamp(),
        )
        .unwrap();
    let mut message = bridget_core::BridgetMessage::new(
        &fixture.task.owner,
        &fixture.task.child,
        "corriger sans interruption156",
    );
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    message.origin = Some(bridget_core::MessageOrigin::Human);
    let command = bridget_transport::protocol::ExecutionControlCommand {
        version: 1,
        command_id: "native-control156".into(),
        execution_id: "execution-guidance156".into(),
        generation: 1,
        revision: 1,
        operation: bridget_transport::protocol::ExecutionControlOperation::SteerCurrent,
        message: Some(message),
    };
    let response = crate::daemon::handle_execution_control(
        Some("conn-1"),
        "012_scope_nativecontrol156",
        command,
        &mut st,
    );
    assert!(matches!(
        response,
        DaemonToWrapper::ControlExecutionResult {
            outcome: bridget_transport::protocol::ExecutionControlOutcome::OutcomeUnknown,
            ..
        }
    ));
    drop(st);
    assert!(
        matches!(crate::daemon::presence_tests::read_control(&mut fixture.child_reader),DaemonToWrapper::ControlExecutionDispatch {command,..} if command.message.as_ref().is_some_and(|message|message.origin.is_none() && message.intent==Some(bridget_core::MessageIntent::SteerCurrent)))
    );
}

#[test]
fn guidance156_cancel_parent_mission_sans_interruption() {
    let mut fixture = RequestFixture155::new("guidance156-cancel-native");
    let response = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::CancelRequest {
            id: fixture.task.mission.clone(),
            sender: fixture.task.owner.clone(),
            reason: Some("retrait de la demande156".into()),
        },
        &fixture.state,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled { ref state,.. } if state=="cancelled"),
        "{response:?}"
    );
    // La définition native figée est codex_app_server/Cli : la voie CLI reçoit une notice passive.
    let notification = crate::daemon::presence_tests::read_control(&mut fixture.child_reader);
    assert!(
        matches!(&notification,DaemonToWrapper::Deliver {message,..}
        if message.intent.is_none() && message.origin.is_none()
            && message.body.contains(&fixture.task.mission)
            && message.body.contains("Aucune réponse n'est requise.")),
        "notice native : {notification:?}"
    );
    assert_eq!(
        fixture
            .state
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(&fixture.task.child, unix_timestamp() - 600)
            .unwrap()
            .0,
        0,
        "aucune admission d'interruption dans guidance_events"
    );
    fixture
        .child_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        fixture.child_reader.read_line(&mut line).is_err(),
        "aucune commande d'interruption supplémentaire : {line}"
    );
    assert_eq!(
        fixture
            .state
            .lock()
            .unwrap()
            .store
            .get_request(&fixture.task.mission)
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        fixture
            .state
            .lock()
            .unwrap()
            .delegation_store
            .get(&fixture.task.task_id)
            .unwrap()
            .unwrap()
            .state,
        "working"
    );
}

#[test]
fn spec174_child_delivery_unavailable_is_not_revoked() {
    let fixture = RequestFixture155::with_remote("r174-diagnostic", true);
    let question = fixture.question(174);
    fixture.receive(question.clone());
    let row = fixture.row(&question);
    let mut st = fixture.state.lock().unwrap();
    let now = unix_timestamp();
    assert!(native_child_requests::valid_delivery(&st, &fixture.task, &row, now).is_ok());
    st.native_permission_facts.remove("conn-1");
    st.native_permission_refusals.insert("conn-1".into(), "permission_source_unavailable".into());
    assert_eq!(native_child_requests::valid_delivery(&st, &fixture.task, &row, now), Err("permission_source_unavailable"));
    st.native_permission_refusals.insert("conn-1".into(), "permission_not_inherited".into());
    assert_eq!(native_child_requests::valid_delivery(&st, &fixture.task, &row, now), Err("revoked"));
    st.native_permission_refusals.insert("conn-1".into(), "/private/canary: read failed".into());
    assert_eq!(native_child_requests::valid_delivery(&st, &fixture.task, &row, now), Err("delivery_check_unavailable"));
    st.provider_limits.record_scoped("native-test", fixture.task.child_instance.as_deref().unwrap(),
        bridget_transport::protocol::RateLimitFact { credits: None, window: "five_hour".into(), status: "rate_limit_reached".into(), resets_at: Some(now + 600), used_percent: Some(100) },
        bridget_transport::protocol::RateLimitSource::CodexAppServer, now);
    assert_eq!(native_child_requests::valid_delivery(&st, &fixture.task, &row, now), Err("provider_quota_exhausted"));
    let view = failure("delivery_check_unavailable");
    assert!(view["detail"]["recovery"].as_str().unwrap().contains("bridget_task_status"));
    assert!(!view.to_string().contains("canary"));
}

#[test]
fn spec174_child_answer_invalidated_after_decision_is_visible() {
    let mut fixture = RequestFixture155::with_remote("r174-late-diagnostic", true);
    let question = fixture.question(175);
    fixture.receive(question.clone());
    let row = fixture.row(&question);
    fixture.decide(&row);
    {
        let mut st = fixture.state.lock().unwrap();
        assert_eq!(current_task_view(&st, &fixture.task, true).unwrap()["recent_requests"][0]["delivery_state"], "pending");
        st.native_permission_facts.remove("conn-1");
        st.native_permission_refusals.insert("conn-1".into(), "permission_source_unavailable".into());
    }
    native_child_requests::tick(&fixture.state);
    let row = fixture.row(&question);
    assert_eq!(row.state, "answered");
    assert_eq!(row.outcome.as_deref(), Some("answered_by_parent"));
    assert_eq!(row.delivery_state, "void");
    let view = current_task_view(&fixture.state.lock().unwrap(), &fixture.task, true).unwrap();
    assert_eq!(view["recent_requests"][0]["delivery_error"], "permission_source_unavailable");
    let frames = fixture.frames();
    assert_eq!(frames.len(), 1);
    assert!(matches!(frames[0], DaemonToWrapper::ChildRequestCancel { .. }));
}


#[test]
fn spec184_slow_release_confirmation_keeps_existing_task() {
    use bridget_transport::protocol::*;
    let fixture = RequestFixture155::with_remote_start("r184-slow-release", true, false);
    let parent = fixture.task.permission_snapshot.as_ref().unwrap().parent.clone();
    let prepared = WorkerPrepared {
        placement: fixture.task.remote_placement.clone().unwrap(), workspace: None,
        token: "prepared184".into(), definition: fixture.task.definition.clone(),
        child_policy: fixture.task.permission_snapshot.as_ref().unwrap().child_policy.clone(),
        effective_posture: fixture.task.effective_posture.unwrap(), prepared_at: unix_timestamp(),
    };
    let (writer, mut reader) = crate::daemon::presence_tests::control_socket("wf184");
    reader.get_ref().set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let worker = remote_worker_identity(&prepared.placement);
    {
        let mut st = fixture.state.lock().unwrap();
        st.connections.insert("worker184".into(), writer);
        st.remote_workers.insert(worker.machine_id.clone(), RemoteWorkerConnection {
            identity: worker, conn_id: "worker184".into(), epoch: "epoch184".into(),
            capabilities: vec![WorkerCapability::ReleaseLeaseV2],
            last_contact: Instant::now(), last_contact_at: unix_timestamp(),
            contact: WorkerContactState::Connected,
        });
    }
    let shared = fixture.state.clone();
    let server = std::thread::spawn(move || {
        for step in 0..3 {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = decode(line.trim()).unwrap() else {
                panic!("requête worker attendue");
            };
            let response = match request.request {
                WorkerRequestOperation::Start { .. } if step == 0 =>
                    WorkerResponseOperation::ReadyToRelease { ready_at: unix_timestamp() },
                WorkerRequestOperation::ReserveResultSlot { slot } if step == 1 =>
                    WorkerResponseOperation::ResultSlotReserved { slot, slot_id: "slot184".into() },
                WorkerRequestOperation::Release { permit } if step == 2 => {
                    assert_eq!(permit.expires_at - permit.issued_at, WORKER_RELEASE_LEASE_SECS);
                    // Une revalidation distante peut dépasser l'ancien budget de cinq secondes.
                    std::thread::sleep(Duration::from_secs(6));
                    assert!(unix_timestamp() < permit.expires_at, "le permis doit rester valable");
                    WorkerResponseOperation::Released {}
                }
                other => panic!("séquence de démarrage inattendue : {other:?}"),
            };
            assert!(handle_worker_message(&shared, "worker184", WrapperToDaemon::WorkerResponse {
                response: WorkerResponse { version: REMOTE_WORKER_VERSION,
                    correlation_id: request.correlation_id, connection_epoch: request.connection_epoch,
                    worker: request.worker, command: request.command, response },
            }).is_none());
        }
    });
    let outcome = start_remote_task(&fixture.state, &fixture.task.task_id, &prepared,
        &parent, "conn-1", None, None);
    server.join().unwrap();
    assert!(outcome.is_ok(), "la confirmation lente doit être acceptée : {outcome:?}");
    let st = fixture.state.lock().unwrap();
    assert_eq!(st.delegation_store.tasks().unwrap().len(), 1, "aucune mission dupliquée");
    assert!(st.pending_worker_responses.is_empty());
}

#[test]
fn spec201_master_restart_preserves_remote_child_callbacks_without_stop_proof() {
    for stage in ["open", "pending", "sent"] {
        let fixture = RequestFixture155::with_remote("restart-remote201", true);
        let request = fixture.approval(201);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        if stage != "open" {
            fixture.decide(&row);
        }
        if stage == "sent" {
            assert!(fixture.state.lock().unwrap().delegation_store
                .claim_child_request(&row.request_id, unix_timestamp()).unwrap());
        }
        let before = serde_json::to_value(fixture.row(&request)).unwrap();
        let task_before = serde_json::to_value(fixture.state.lock().unwrap().delegation_store
            .get(&fixture.task.task_id).unwrap().unwrap()).unwrap();
        {
            let mut st = fixture.state.lock().unwrap();
            native_delegation::prepare_restart(&mut st).unwrap();
        }
        assert_eq!(serde_json::to_value(fixture.row(&request)).unwrap(), before, "{stage}");
        assert_eq!(serde_json::to_value(fixture.state.lock().unwrap().delegation_store
            .get(&fixture.task.task_id).unwrap().unwrap()).unwrap(), task_before, "{stage}");
    }
}

#[test]
fn spec201_remote_reconnect_gap_keeps_callbacks_but_not_cancel_or_expiry() {
    // Le tick peut acquitter l’inbox d’une décision humaine déjà durable.
    // Ce seul bit n’est ni la décision ni l’application par le fournisseur.
    let callback = |row: ChildRequest| {
        let mut value = serde_json::to_value(row).unwrap();
        value.as_object_mut().unwrap().remove("inbox_settled");
        value
    };
    for stage in ["open", "pending", "sent"] {
        let fixture = RequestFixture155::with_remote("reconnect-remote201", true);
        let request = fixture.approval(202);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        if stage != "open" { fixture.decide(&row); }
        if stage == "sent" { assert!(fixture.state.lock().unwrap().delegation_store
            .claim_child_request(&row.request_id, unix_timestamp()).unwrap()); }
        let before = callback(fixture.row(&request));
        let route = fixture.state.lock().unwrap().router.unregister_by_conn("child155").unwrap();
        native_child_requests::tick(&fixture.state);
        assert_eq!(callback(fixture.row(&request)), before, "{stage}");
        let persisted = fixture.row(&request);
        let mut st = fixture.state.lock().unwrap();
        let mut current = st.delegation_store.get(&fixture.task.task_id).unwrap().unwrap();
        assert_eq!(native_child_requests::valid_delivery(&st, &current, &persisted, unix_timestamp()), Err("remote_reconnecting"));
        current.state = "cancelling".into();
        assert_eq!(native_child_requests::valid_delivery(&st, &current, &persisted, unix_timestamp()), Err("cancelled"));
        current.state = "working".into();
        assert_eq!(native_child_requests::valid_delivery(&st, &current, &persisted, persisted.deadline_at), Err("expired"));
        st.router.register(&route.agent_id, &route.agent_type, &route.connection_id).unwrap();
        assert_ne!(native_child_requests::valid_delivery(&st, &current, &persisted, unix_timestamp()), Err("remote_reconnecting"));
        let owner_route = st.router.unregister_by_conn("conn-1").unwrap();
        drop(st);
        native_child_requests::tick(&fixture.state);
        assert_eq!(callback(fixture.row(&request)), before, "parent/{stage}");
        let mut st = fixture.state.lock().unwrap();
        st.router.register(&owner_route.agent_id, &owner_route.agent_type, &owner_route.connection_id).unwrap();
        assert_ne!(native_child_requests::valid_delivery(&st, &current, &persisted, unix_timestamp()), Err("remote_reconnecting"));
        st.conn_instances.insert("child155".into(), "foreign-instance201".into());
        assert_eq!(native_child_requests::valid_delivery(&st, &current, &persisted, unix_timestamp()), Err("instance_changed"));
    }
}
