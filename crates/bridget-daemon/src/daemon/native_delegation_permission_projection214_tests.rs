use super::*;
#[test]
fn spec214_candidate_refusal_distinguishes_unknown_translation_from_changed_rights() {
    let reasons = json!([{"kind":"candidate","code":"permissions_changed","candidates":1}]);
    let value = failure(&format!("routing_candidate_unavailable:{reasons}"));
    assert_eq!(value["code"], "routing_candidate_unavailable");
    assert_eq!(
        value["detail"]["permission_projection"]["cause"],
        "unobserved"
    );
    assert!(
        value["detail"]["permission_projection"]["recovery"]
            .as_str()
            .unwrap()
            .contains("cette admission")
    );
    assert!(value["detail"]["permission_projection"]["human_action"].is_string());
    assert_eq!(value["detail"]["reasons"], reasons);
    let unrelated = failure("routing_candidate_unavailable");
    assert!(unrelated["detail"].get("permission_projection").is_none());
}
#[test]
fn spec220_observation_correlation_expiry_definition_and_refresh_preserve_tasks() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::{WorkerCapability, WorkerContactState, WorkerIdentity};
    use bridget_transport::provider_catalogue::*;
    let (shared, config, _rx, _reader, _request, _root) = super::tests::routing168_fixture(1);
    let mut st = shared.lock().unwrap();
    let worker = WorkerIdentity {
        machine_id: "local".into(),
        worker_id: "worker220".into(),
        provisioning_revision: 1,
    };
    st.remote_workers.insert(
        "local".into(),
        RemoteWorkerConnection {
            identity: worker.clone(),
            conn_id: "worker220".into(),
            epoch: "epoch220".into(),
            capabilities: vec![WorkerCapability::ProviderCatalogueV1],
            catalogues: HashMap::new(),
            catalogue_refresh_id: "epoch220".into(),
            catalogue_refresh_at: None,
            contact: WorkerContactState::Connected,
            last_contact: Instant::now(),
            last_contact_at: unix_timestamp(),
        },
    );
    let digest = st
        .registry
        .resolved_definition("native-test")
        .unwrap()
        .digest;
    let original = serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap();
    let update = WorkerCatalogueUpdate {
        version: 1,
        worker,
        connection_epoch: "epoch220".into(),
        refresh_id: "epoch220".into(),
        sequence: 1,
        catalogue: ProviderCatalogue {
            connection_id: "main".into(),
            definition_digest: digest,
            observed_at: unix_timestamp(),
            state: CatalogueState::Observed,
            account_scope: Some("b".repeat(32)),
            cause: None,
            models: vec![CatalogueModel {
                service_tiers_unavailable: false,
                model: "announced-outside-policy".into(),
                supported_efforts: vec![],
                default_effort: None,
                service_tiers: vec![],
                default_service_tier: None,
            }],
        },
    };
    let mut wrong = update.clone();
    wrong.connection_epoch = "foreign".into();
    assert!(catalogue::ingest(&mut st, "worker220", wrong).is_some());
    for n in 0..32 {
        st.remote_workers
            .get_mut("local")
            .unwrap()
            .catalogues
            .insert(
                format!("obsolete-{n}"),
                catalogue::CachedCatalogue {
                    sequence: 1,
                    received: Instant::now(),
                    received_at: unix_timestamp(),
                    value: update.catalogue.clone(),
                },
            );
    }
    assert!(catalogue::ingest(&mut st, "worker220", update.clone()).is_none());
    assert_eq!(st.remote_workers["local"].catalogues.len(), 1);
    let mut mismatch = update.clone();
    mismatch.sequence = 2;
    mismatch.catalogue.definition_digest = "f".repeat(64);
    assert!(catalogue::ingest(&mut st, "worker220", mismatch).is_none());
    let rejected = catalogue::project(&st, &st.registry);
    assert_eq!(rejected[0]["status"], "definition_changed");
    assert_eq!(rejected[0]["models"], json!([]));
    let mut repaired = update.clone();
    repaired.sequence = 3;
    assert!(catalogue::ingest(&mut st, "worker220", repaired).is_none());
    let observed = catalogue::project(&st, &st.registry);
    assert_eq!(observed[0]["status"], "reported_models");
    assert_eq!(observed[0]["models"][0]["declared_in_policy"], false);
    assert_eq!(observed[0]["entitlement"], "unknown");
    assert_eq!(observed[0]["provider_cache"], "not_attested");
    assert_eq!(observed[0]["auth"], "reported_present");
    assert_eq!(observed[0]["account_scope"], "b".repeat(32));
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .observed_at -= 181;
    let stale = catalogue::project(&st, &st.registry);
    assert_eq!(stale[0]["status"], "stale");
    assert!(stale[0]["age_secs"].as_u64().unwrap() >= 181);
    assert!(stale[0]["models"].as_array().unwrap().is_empty());
    assert_eq!(stale[0]["auth"], "not_verified");
    assert!(stale[0]["account_scope"].is_null());
    let advice = stale[0]["next_action"].as_str().unwrap();
    assert!(
        advice.contains("marqueurs privés"),
        "une donnée périmée ne prouve pas l'arrêt de la sonde"
    );
    assert!(advice.contains("changement de version"));
    assert!(advice.contains("autres travaux autorisés"));
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .definition_digest = "c".repeat(64);
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .observed_at = unix_timestamp();
    let changed = catalogue::project(&st, &st.registry);
    assert_eq!(changed[0]["status"], "definition_changed");
    assert_eq!(changed[0]["auth"], "not_verified");
    assert!(changed[0]["account_scope"].is_null());
    let mut signed_out = update.clone();
    signed_out.sequence = 4;
    signed_out.catalogue.state = CatalogueState::SignInRequired;
    signed_out.catalogue.account_scope = None;
    signed_out.catalogue.models.clear();
    assert!(catalogue::ingest(&mut st, "worker220", signed_out).is_none());
    let absent = catalogue::project(&st, &st.registry);
    assert_eq!(absent[0]["status"], "sign_in_required");
    assert_eq!(absent[0]["auth"], "reported_absent");
    assert!(absent[0]["account_scope"].is_null());
    assert_eq!(absent[0]["models"], json!([]));
    {
        let cached = &mut st
            .remote_workers
            .get_mut("local")
            .unwrap()
            .catalogues
            .get_mut("main")
            .unwrap()
            .value;
        *cached = update.catalogue.clone();
        cached.state = CatalogueState::Unavailable;
        cached.models.clear();
        cached.account_scope = None;
        cached.cause = Some(CatalogueCause::StopUnconfirmed);
        cached.observed_at -= 181;
    }
    let blocked = catalogue::project(&st, &st.registry);
    assert_eq!(blocked[0]["status"], "unavailable");
    assert!(
        blocked[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("maintenance")
    );
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogue_refresh_id = "new-refresh".into();
    let mut late = update.clone();
    late.sequence = 2;
    assert!(catalogue::ingest(&mut st, "worker220", late).is_none());
    assert_eq!(st.remote_workers["local"].catalogues["main"].sequence, 4);
    let (a, _b) = UnixStream::pair().unwrap();
    st.connections.insert(
        "worker220".into(),
        Arc::new(Mutex::new(std::io::BufWriter::new(a))),
    );
    let mut controls = Vec::new();
    let result = catalogue::refresh(&mut st, Some("local"), &mut controls).unwrap();
    assert_eq!(result["workers"][0]["status"], "requested");
    assert_eq!(controls.len(), 1);
    assert_eq!(
        st.remote_workers["local"].catalogues["main"].value.cause,
        Some(CatalogueCause::StopUnconfirmed)
    );
    let token = st.remote_workers["local"].catalogue_refresh_id.clone();
    let second = catalogue::refresh(&mut st, Some("local"), &mut controls).unwrap();
    assert_eq!(second["workers"][0]["status"], "already_requested");
    assert_eq!(controls.len(), 1);
    assert_eq!(token, st.remote_workers["local"].catalogue_refresh_id);
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .capabilities
        .clear();
    assert_eq!(
        catalogue::project(&st, &st.registry)[0]["status"],
        "unverified_peer"
    );
    assert_eq!(
        catalogue::project(&st, &st.registry)[0]["probe_constraint"]["state"],
        "stop_unconfirmed"
    );
    let unverified = catalogue::project(&st, &st.registry);
    assert_eq!(unverified[0]["auth"], "not_verified");
    assert!(unverified[0]["account_scope"].is_null());
    assert!(
        unverified[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("marqueurs privés")
    );
    assert!(
        unverified[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("ne confirme aucun arrêt")
    );
    st.remote_workers.remove("local");
    assert_eq!(
        catalogue::project(&st, &st.registry)[0]["status"],
        "observation_source_unavailable"
    );
    let mut raw: Value =
        serde_json::from_str(&std::fs::read_to_string(st.registry.source()).unwrap()).unwrap();
    raw["machines"] = json!({"local":{"enabled":false,"worker_id":"worker220","provisioning_revision":1,"credential_fingerprint":"a".repeat(64),"allowed_agent_types":["native-test"],"mappings":{}}});
    st.registry =
        AgentRegistry::from_json(&raw.to_string(), st.registry.source().to_path_buf()).unwrap();
    let disabled = catalogue::project(&st, &st.registry);
    assert_eq!(disabled[0]["status"], "machine_disabled");
    assert!(
        disabled[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("activation")
    );

    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        original
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

fn observed221_fixture() -> (Arc<Mutex<DaemonState>>, DaemonConfig, i64, Instant) {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::{WorkerCapability, WorkerContactState, WorkerIdentity};
    use bridget_transport::provider_catalogue::*;
    let (shared, config, _rx, _reader, _request, _root) = super::tests::routing168_fixture(1);
    let now = unix_timestamp();
    let clock = {
        let mut st = shared.lock().unwrap();
        let worker = WorkerIdentity {
            machine_id: "local".into(),
            worker_id: "worker221-models".into(),
            provisioning_revision: 1,
        };
        st.remote_workers.insert(
            "local".into(),
            RemoteWorkerConnection {
                identity: worker.clone(),
                conn_id: "worker221-models".into(),
                epoch: "epoch221-models".into(),
                capabilities: vec![WorkerCapability::ProviderCatalogueV1],
                catalogues: HashMap::new(),
                catalogue_refresh_id: "epoch221-models".into(),
                catalogue_refresh_at: None,
                contact: WorkerContactState::Connected,
                last_contact: Instant::now(),
                last_contact_at: now,
            },
        );
        let value = ProviderCatalogue {
            connection_id: "main".into(),
            definition_digest: st
                .registry
                .resolved_definition("native-test")
                .unwrap()
                .digest,
            observed_at: now - 10,
            state: CatalogueState::Observed,
            account_scope: Some("a".repeat(32)),
            cause: None,
            models: vec![
                CatalogueModel {
                    service_tiers_unavailable: false,
                    model: "m000".into(),
                    supported_efforts: vec!["low".into()],
                    default_effort: Some("low".into()),
                    service_tiers: vec![],
                    default_service_tier: None,
                },
                CatalogueModel {
                    service_tiers_unavailable: false,
                    model: "outside-policy".into(),
                    supported_efforts: vec![],
                    default_effort: None,
                    service_tiers: vec![],
                    default_service_tier: None,
                },
            ],
        };
        let update = WorkerCatalogueUpdate {
            version: 1,
            worker,
            connection_epoch: "epoch221-models".into(),
            refresh_id: "epoch221-models".into(),
            sequence: 1,
            catalogue: value,
        };
        assert!(catalogue::ingest(&mut st, "worker221-models", update).is_none());
        st.remote_workers["local"].catalogues["main"].received
    };
    (shared, config, now, clock)
}

#[test]
fn spec221_c07_model_checks_distinguish_announcement_absence_and_expired_observation() {
    use super::super::native_provider_catalogue220 as catalogue;
    let (shared, config, now, clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let tasks = serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap();
    let current = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(current[0]["model_checks"][0]["model"], "m000");
    assert_eq!(
        current[0]["model_checks"][0]["status"],
        "announced_access_unverified"
    );
    assert_eq!(current[0]["model_checks"][0]["entitlement"], "unknown");
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .models
        .retain(|m| m.model != "m000");
    let missing = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(missing[0]["model_checks"][0]["status"], "not_announced");
    assert!(
        missing[0]["model_check_guidance"]["not_announced"]
            .as_str()
            .unwrap()
            .contains("ne prouve pas")
    );
    let expired = catalogue::project_at(
        &st,
        &st.registry,
        now + 181,
        clock + Duration::from_secs(181),
    );
    assert_eq!(expired[0]["model_checks"][0]["status"], "unverified");
    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        tasks
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
#[allow(clippy::assertions_on_constants)] // le drapeau de lancement est une constante : l'assertion garde le test
fn spec221_c05_unsupported_selection_is_guided_and_consumes_no_admission() {
    let (shared, config, _rx, _reader, mut request, _root) = super::tests::routing168_fixture(1);
    let before =
        serde_json::to_vec(&shared.lock().unwrap().delegation_store.tasks().unwrap()).unwrap();
    request.service_tier = Some("priority".into());
    assert!(crate::provider_catalogue220::MODEL_OPTIONS_LAUNCH_ENABLED);
    let error =
        admit_auto_routing("conn-1", &request, None, None, None, None, &shared).unwrap_err();
    assert_eq!(error, "routing_candidate_unavailable:catalogue_no_match");
    let view = routing_failure_admission("conn-1", &request, None, failure(&error), &shared);
    assert_eq!(view["detail"]["admission"]["state"], "not_recorded");
    assert!(view["detail"]["admission"]["task_id"].is_null());
    assert_eq!(view["detail"]["scope"], "this_admission");
    assert!(
        view["message"]
            .as_str()
            .unwrap()
            .contains("autres travaux autorisés")
    );
    assert!(
        view["detail"]["human_action"]
            .as_str()
            .unwrap()
            .contains("utilisateur")
    );
    request.service_tier = Some("bad\nlevel".into());
    assert_eq!(
        admit_auto_routing("conn-1", &request, None, None, None, None, &shared).unwrap_err(),
        "model_options_invalid"
    );
    let invalid = failure("model_options_invalid");
    assert_eq!(invalid["detail"]["cause"], "invalid_service_tier");
    assert!(
        invalid["message"]
            .as_str()
            .unwrap()
            .contains("service_tier")
    );
    assert_eq!(
        serde_json::to_vec(&shared.lock().unwrap().delegation_store.tasks().unwrap()).unwrap(),
        before
    );
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c05_catalogue_options_require_negotiation_and_keep_the_account_context() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::*;
    use bridget_transport::provider_catalogue::*;
    let (shared, config, now, clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let tasks = serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap();
    let mut value = st.remote_workers["local"].catalogues["main"].value.clone();
    value.models[0].service_tiers = vec!["default".into(), "priority".into()];
    value.models[0].default_service_tier = Some("default".into());
    let update = WorkerCatalogueUpdate {
        version: 1,
        worker: st.remote_workers["local"].identity.clone(),
        connection_epoch: "epoch221-models".into(),
        refresh_id: "epoch221-models".into(),
        sequence: 2,
        catalogue: value,
    };
    assert!(update.valid());
    assert!(matches!(
        catalogue::ingest(&mut st, "worker221-models", update.clone()),
        Some(DaemonToWrapper::WorkerRejected {
            reason: WorkerRefusal::CapabilityRequired
        })
    ));
    let old = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(old[0]["model_options_status"], "unverified_peer");
    assert!(old[0]["models"][0].get("service_tiers").is_none());
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .capabilities
        .push(WorkerCapability::ModelOptionsV1);
    assert!(catalogue::ingest(&mut st, "worker221-models", update.clone()).is_none());
    let fresh = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(fresh[0]["model_options_status"], "reported");
    assert_eq!(fresh[0]["model_options_selection"], "explicit_only");
    assert_eq!(
        fresh[0]["models"][0]["service_tiers"],
        json!(["default", "priority"])
    );
    assert_eq!(fresh[0]["models"][0]["default_service_tier"], "default");
    assert_eq!(fresh[0]["account_scope"], old[0]["account_scope"]);
    assert_eq!(fresh[0]["entitlement"], "unknown");
    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        tasks
    );
    let mut legacy = update;
    legacy.sequence = 3;
    legacy.catalogue = legacy.catalogue.legacy_projection();
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .capabilities
        .retain(|c| *c != WorkerCapability::ModelOptionsV1);
    assert!(catalogue::ingest(&mut st, "worker221-models", legacy).is_none());
    let unavailable = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(unavailable[0]["model_options_status"], "unverified_peer");
    assert!(unavailable[0]["models"][0].get("service_tiers").is_none());
    assert_eq!(unavailable[0]["account_scope"], fresh[0]["account_scope"]);
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c05_local_admission_keeps_the_option_in_probe_and_saved_launch() {
    let (shared, config, rx, _reader, mut auto, _root) = super::tests::routing168_fixture(1);
    auto.service_tier = Some("priority".into());
    let mut st = shared.lock().unwrap();
    let path = st.registry.source().to_path_buf();
    let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    raw["agents"]["native-test"]["capabilities"]["models"]["m000"]["service_tiers"] =
        json!(["priority"]);
    raw["routing_policy"]["model_catalog"]["main"][0]["service_tiers"] = json!(["priority"]);
    std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
    st.registry = AgentRegistry::load_from_path(path).unwrap();
    let mut plan = routing_admission_plan(&st.registry, "local", &auto).unwrap();
    let mut observed = super::tests::routing168_observation();
    observed.model_option_account = Some("a".repeat(32));
    observed.definition_digest = st
        .registry
        .resolved_definition("native-test")
        .unwrap()
        .digest;
    plan.observations = vec![observed];
    plan.scores = vec![0];
    let delegated = routing_delegate(
        &auto,
        &plan.candidates[0],
        Some("high".into()),
        "local",
        None,
    );
    let owner = live_connection_identity(&st, "conn-1").unwrap().0;
    let mut receipt = routing_auto_receipt(
        owner,
        &auto,
        serde_json::to_vec(&NativeDelegationRequest::AutoDelegateV2(auto.clone())).unwrap(),
        &plan,
        st.coordinator_epoch,
        unix_timestamp(),
    )
    .unwrap();
    receipt.state = AutoAdmissionState::Chosen;
    receipt.candidate_id = Some("c001".into());
    receipt.delegate_canonical = Some(serde_json::to_vec(&delegated).unwrap());
    receipt.mode = Some(crate::delegation::AutoAdmissionMode::Automatic);
    assert!(
        st.delegation_store
            .insert_auto_admission_receipt(&receipt)
            .unwrap()
    );
    let probe = admit_delegate(
        "conn-1",
        &delegated,
        None,
        None,
        &mut st,
        WorkspacePass::Probe,
        Some(&receipt),
    )
    .unwrap();
    let WorkspaceAdmission::Probe(_, _, definition, _) = probe else {
        panic!("local probe");
    };
    assert!(
        definition
            .args
            .iter()
            .any(|arg| arg == "service_tier=\"priority\""),
        "Probe must preserve the choice passed to worktree Reserve/Commit"
    );
    let shared_view = admit_delegate(
        "conn-1",
        &delegated,
        None,
        None,
        &mut st,
        WorkspacePass::Shared,
        Some(&receipt),
    )
    .unwrap();
    let WorkspaceAdmission::View(view) = shared_view else {
        panic!("local shared");
    };
    let task = st
        .delegation_store
        .get(view["task_id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(task.service_tier, auto.service_tier);
    assert!(
        task.definition
            .args
            .iter()
            .any(|arg| arg == "service_tier=\"priority\"")
    );
    let registry = native_launch_registry(&st, &task).unwrap();
    assert_eq!(
        registry.resolved_definition("native-test").unwrap(),
        task.definition
    );
    assert!(
        rx.try_recv().is_err(),
        "no model or supervisor is started by this test"
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_c05_option_requires_current_model_connection_account_and_profile() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::WorkerCapability;
    let (shared, config, now, _clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let path = st.registry.source().to_path_buf();
    let mut policy: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    policy["agents"]["native-test"]["capabilities"]["models"]["m000"]["service_tiers"] =
        json!(["priority", "default"]);
    policy["routing_policy"]["model_catalog"]["main"][0]["service_tiers"] =
        json!(["priority", "default"]);
    std::fs::write(&path, serde_json::to_vec(&policy).unwrap()).unwrap();
    st.registry = AgentRegistry::load_from_path(path).unwrap();
    let candidate = st
        .registry
        .routing_candidates()
        .unwrap()
        .into_iter()
        .find(|c| c.connection_id == "main" && c.model == "m000")
        .unwrap();
    let tasks = serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap();
    let digest = st
        .registry
        .resolved_definition("native-test")
        .unwrap()
        .digest;
    let worker = st.remote_workers.get_mut("local").unwrap();
    worker.capabilities.push(WorkerCapability::ModelOptionsV1);
    let cached = worker.catalogues.get_mut("main").unwrap();
    cached.value.definition_digest = digest;
    cached.value.models[0].service_tiers = vec!["priority".into()];
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("priority")).unwrap(),
        Some("a".repeat(32))
    );
    {
        let m = &mut st
            .remote_workers
            .get_mut("local")
            .unwrap()
            .catalogues
            .get_mut("main")
            .unwrap()
            .value
            .models[0];
        m.service_tiers.clear();
        m.service_tiers_unavailable = true;
    }
    let partial = catalogue::project(&st, &st.registry);
    assert_eq!(partial[0]["status"], "reported_models");
    assert_eq!(
        partial[0]["models"][0]["model_options_status"],
        "not_observed"
    );
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("priority"))
            .unwrap_err(),
        "model_options_observation_unavailable"
    );
    {
        let m = &mut st
            .remote_workers
            .get_mut("local")
            .unwrap()
            .catalogues
            .get_mut("main")
            .unwrap()
            .value
            .models[0];
        m.service_tiers = vec!["priority".into()];
        m.service_tiers_unavailable = false;
    }

    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, None).unwrap(),
        None
    );
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("default"))
            .unwrap_err(),
        "model_options_not_announced"
    );
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("invented"))
            .unwrap_err(),
        "model_options_not_permitted"
    );
    let mut other = candidate.clone();
    other.connection_id = "other".into();
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &other, Some("priority")).unwrap_err(),
        "model_options_not_permitted"
    );
    let (other, other_config, mut task, _) = super::tests::remote162_capture_fixture();
    let auto: AutoDelegateV2 = serde_json::from_value(json!({"request_id":"frozen-options",
            "task":"Lecture","cwd":task.cwd,"service_tier":"priority"}))
    .unwrap();
    let mut plan = routing_admission_plan(&st.registry, "local", &auto).unwrap();
    let mut fact = super::tests::routing168_observation();
    fact.model_option_account = Some("a".repeat(32));
    fact.connection_epoch = "epoch221-models".into();
    plan.observations = vec![fact];
    plan.scores = vec![0];
    let mut receipt = routing_auto_receipt(
        task.owner.clone(),
        &auto,
        serde_json::to_vec(&NativeDelegationRequest::AutoDelegateV2(auto.clone())).unwrap(),
        &plan,
        st.coordinator_epoch,
        now,
    )
    .unwrap();
    receipt.candidate_id = Some("c001".into());
    task.request = routing_delegate(&auto, &plan.candidates[0], None, "local", None);
    task.service_tier = Some("priority".into());
    assert!(
        st.delegation_store
            .reserve_auto_admission_receipt(&receipt, now)
            .unwrap()
    );
    assert_eq!(recheck_task_model_option(&st, &task), Ok(()));
    let roundtrip: Task = serde_json::from_slice(&serde_json::to_vec(&task).unwrap()).unwrap();
    assert_eq!(roundtrip.service_tier, task.service_tier);
    task.service_tier = Some("default".into());
    assert_eq!(
        recheck_task_model_option(&st, &task).unwrap_err(),
        "auto_admission_invalid"
    );
    task.service_tier = Some("priority".into());
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .account_scope = Some("b".repeat(32));
    assert_eq!(
        recheck_task_model_option(&st, &task).unwrap_err(),
        "model_options_account_changed"
    );
    st.remote_workers.get_mut("local").unwrap().epoch = "reconnected".into();
    assert_eq!(
        recheck_task_model_option(&st, &task).unwrap_err(),
        "model_options_account_changed"
    );
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .account_scope = Some("a".repeat(32));
    assert_eq!(recheck_task_model_option(&st, &task), Ok(()));
    st.remote_workers.get_mut("local").unwrap().epoch = "epoch221-models".into();
    let saved = st
        .delegation_store
        .auto_admission_receipt(&task.owner, &auto.request_id)
        .unwrap()
        .unwrap();
    assert_eq!(saved.metadata, receipt.metadata);
    assert_eq!(saved.canonical, receipt.canonical);
    drop(other);
    super::tests::cleanup(&other_config);
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .account_scope = None;
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("priority"))
            .unwrap_err(),
        "model_options_account_unidentified"
    );
    {
        let cached = &mut st
            .remote_workers
            .get_mut("local")
            .unwrap()
            .catalogues
            .get_mut("main")
            .unwrap()
            .value;
        cached.account_scope = Some("b".repeat(32));
        cached.observed_at = now - 181;
    }
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("priority"))
            .unwrap_err(),
        "model_options_observation_unavailable"
    );
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .catalogues
        .get_mut("main")
        .unwrap()
        .value
        .observed_at = now;
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("priority")).unwrap(),
        Some("b".repeat(32))
    );
    st.remote_workers
        .get_mut("local")
        .unwrap()
        .capabilities
        .retain(|c| *c != WorkerCapability::ModelOptionsV1);
    assert_eq!(
        catalogue::model_option_account(&st, &st.registry, &candidate, Some("priority"))
            .unwrap_err(),
        "model_options_observation_unavailable"
    );
    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        tasks
    );
    for code in [
        "model_options_not_permitted",
        "model_options_launcher_ambiguous",
        "model_options_not_announced",
        "model_options_account_changed",
        "model_options_account_unidentified",
        "model_options_observation_unavailable",
    ] {
        let refusal = failure(code);
        assert_eq!(refusal["detail"]["scope"], "this_admission");
        assert!(
            refusal["message"]
                .as_str()
                .unwrap()
                .contains("autres travaux")
        );
        assert!(refusal["detail"]["human_action"].is_string());
    }
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_l2a_dates_distinguish_source_receipt_and_expire_on_either_clock() {
    use super::super::native_provider_catalogue220 as catalogue;
    let (shared, config, now, clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let at = clock + Duration::from_secs(30);
    st.remote_workers.get_mut("local").unwrap().last_contact = at;
    let value = catalogue::project_at(&st, &st.registry, now, at);
    assert_eq!(value[0]["status"], "reported_models");
    assert_eq!(value[0]["last_observed_at"], now - 10);
    assert!(value[0]["received_at"].as_i64().unwrap() >= now);
    assert_eq!(value[0]["source_age_secs"], 10);
    assert_eq!(value[0]["received_age_secs"], 30);
    assert_eq!(value[0]["age_secs"], 30);
    assert_eq!(value[0]["expires_at"], now + 150);
    assert_eq!(value[0]["provider_cache"], "not_attested");
    assert_eq!(
        value[0]["account_change_detection"]["external_login_without_file_change"],
        "periodic_check_only"
    );
    assert_eq!(
        value[0]["account_change_detection"]["refresh_target_is_guarantee"],
        false
    );
    // Recul de l'horloge locale : l'instant monotone interdit de rajeunir le reçu.
    let old = clock + Duration::from_secs(181);
    st.remote_workers.get_mut("local").unwrap().last_contact = old;
    let expired = catalogue::project_at(&st, &st.registry, now, old);
    assert_eq!(expired[0]["status"], "stale");
    assert_eq!(expired[0]["models"], json!([]));
    assert_eq!(expired[0]["freshness_cause"], "receipt_expired");
    assert!(expired[0]["expires_at"].is_null());
    assert_eq!(expired[0]["auth"], "not_verified");
    assert!(expired[0]["account_scope"].is_null());
    // Source trop ancienne et source trop future ne redeviennent pas actuelles.
    let source_old = catalogue::project_at(&st, &st.registry, now + 200, old);
    assert_eq!(source_old[0]["status"], "stale");
    assert_eq!(
        source_old[0]["freshness_cause"],
        "source_and_receipt_expired"
    );
    st.remote_workers.get_mut("local").unwrap().last_contact = clock;
    let source_future = catalogue::project_at(&st, &st.registry, now - 100, clock);
    assert_eq!(source_future[0]["status"], "stale");
    assert_eq!(source_future[0]["freshness_cause"], "source_clock_ahead");
    assert!(
        source_future[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("horloges")
    );
    assert!(source_future[0]["models"].as_array().unwrap().is_empty());
    let received_old = catalogue::project_at(&st, &st.registry, now + 200, clock);
    assert_eq!(received_old[0]["freshness_cause"], "source_expired");
    let guidance = received_old[0]["next_action"].as_str().unwrap();
    assert!(guidance.contains("horloges") && guidance.contains("remise différée"));
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_l2a_effort_mismatch_is_an_observation_without_mutating_admission() {
    use super::super::native_provider_catalogue220 as catalogue;
    let (shared, config, now, clock) = observed221_fixture();
    let st = shared.lock().unwrap();
    let before = serde_json::to_vec(&st.registry.routing_catalogue(now).unwrap()).unwrap();
    let tasks = serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap();
    let value = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(
        value[0]["models"][0]["policy_effort_status"],
        "not_announced"
    );
    assert_eq!(
        value[0]["models"][0]["declared_efforts_not_announced"],
        json!(["high"])
    );
    assert_eq!(
        value[0]["models"][0]["declared_default_effort_announced"],
        false
    );
    assert_eq!(
        value[0]["models"][1]["policy_effort_status"],
        "outside_policy"
    );
    assert_eq!(value[0]["models"][1]["declared_in_policy"], false);
    assert_eq!(value[0]["entitlement"], "unknown");
    assert_eq!(
        serde_json::to_vec(&st.registry.routing_catalogue(now).unwrap()).unwrap(),
        before
    );
    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        tasks
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_l2a_same_model_on_two_servers_keeps_its_account_and_connection_context() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::*;
    let (shared, config, now, clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let mut raw: Value =
        serde_json::from_str(&std::fs::read_to_string(st.registry.source()).unwrap()).unwrap();
    if raw["machines"].is_null() {
        raw["machines"] = json!({});
    }
    raw["machines"]["other"] = json!({"enabled":true,"worker_id":"worker-other","provisioning_revision":1,
            "credential_fingerprint":"c".repeat(64),"allowed_agent_types":["native-test"],"mappings":{}});
    let connection = json!({"connection_id":"other","machine_id":"other","engine":"codex","agent_type":"native-test","enabled":true});
    raw["routing_policy"]["connections"]
        .as_array_mut()
        .unwrap()
        .push(connection);
    raw["routing_policy"]["model_catalog"]["other"] =
        raw["routing_policy"]["model_catalog"]["main"].clone();
    raw["routing_policy"]["machine_policies"]["other"] =
        raw["routing_policy"]["machine_policies"]["local"].clone();
    std::fs::write(st.registry.source(), raw.to_string()).unwrap();
    st.registry = AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf()).unwrap();
    let mut value = st.remote_workers["local"].catalogues["main"].value.clone();
    value.connection_id = "other".into();
    value.account_scope = Some("b".repeat(32));
    value.observed_at = now - 20;
    value.models[0].supported_efforts.push("high".into());
    let mut cache = HashMap::new();
    cache.insert(
        "other".into(),
        catalogue::CachedCatalogue {
            sequence: 1,
            received: clock,
            received_at: now,
            value,
        },
    );
    st.remote_workers.insert(
        "other".into(),
        RemoteWorkerConnection {
            identity: WorkerIdentity {
                machine_id: "other".into(),
                worker_id: "worker-other".into(),
                provisioning_revision: 1,
            },
            conn_id: "worker-other".into(),
            epoch: "epoch-other".into(),
            capabilities: vec![WorkerCapability::ProviderCatalogueV1],
            catalogues: cache,
            catalogue_refresh_id: "epoch-other".into(),
            catalogue_refresh_at: None,
            contact: WorkerContactState::Connected,
            last_contact: clock,
            last_contact_at: now,
        },
    );
    let projected = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(projected.len(), 2);
    assert_eq!(
        projected[0]["models"][0]["model"],
        projected[1]["models"][0]["model"]
    );
    assert_eq!(projected[0]["machine_id"], "local");
    assert_eq!(projected[0]["connection_id"], "main");
    assert_eq!(projected[0]["account_scope"], "a".repeat(32));
    assert_eq!(
        projected[0]["models"][0]["policy_effort_status"],
        "not_announced"
    );
    assert_eq!(projected[1]["machine_id"], "other");
    assert_eq!(projected[1]["connection_id"], "other");
    assert_eq!(projected[1]["account_scope"], "b".repeat(32));
    assert_eq!(
        projected[1]["models"][0]["policy_effort_status"],
        "announced"
    );
    assert_eq!(projected[0]["last_observed_at"], now - 10);
    assert_eq!(projected[1]["last_observed_at"], now - 20);
    let untouched = serde_json::to_vec(&projected[1]).unwrap();
    let tasks = serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap();
    let mut switched = st.remote_workers["local"].catalogues["main"].value.clone();
    switched.account_scope = Some("c".repeat(32));
    switched.observed_at = now;
    switched.models = vec![bridget_transport::provider_catalogue::CatalogueModel {
        service_tiers_unavailable: false,
        model: "account-new-model".into(),
        supported_efforts: vec!["low".into()],
        default_effort: Some("low".into()),
        service_tiers: vec![],
        default_service_tier: None,
    }];
    let update = bridget_transport::provider_catalogue::WorkerCatalogueUpdate {
        version: 1,
        worker: st.remote_workers["local"].identity.clone(),
        connection_epoch: "epoch221-models".into(),
        refresh_id: "epoch221-models".into(),
        sequence: 2,
        catalogue: switched,
    };
    assert!(catalogue::ingest(&mut st, "worker221-models", update.clone()).is_none());
    let changed = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(changed[0]["account_scope"], "c".repeat(32));
    assert_eq!(changed[0]["models"].as_array().unwrap().len(), 1);
    assert_eq!(changed[0]["models"][0]["model"], "account-new-model");
    assert_eq!(serde_json::to_vec(&changed[1]).unwrap(), untouched);
    let mut late = update;
    late.sequence = 1;
    late.catalogue.account_scope = Some("a".repeat(32));
    // Garder une enveloppe Observed valide : seul son numéro de séquence est ancien.
    late.catalogue.models[0].model = "account-old-model".into();
    assert!(late.valid());
    assert!(catalogue::ingest(&mut st, "worker221-models", late).is_none());
    assert_eq!(
        serde_json::to_vec(&catalogue::project_at(&st, &st.registry, now, clock)).unwrap(),
        serde_json::to_vec(&changed).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        tasks
    );
    let mut unidentified = st.remote_workers["local"].catalogues["main"].value.clone();
    unidentified.account_scope = None;
    unidentified.cause =
        Some(bridget_transport::provider_catalogue::CatalogueCause::AccountUnidentified);
    let update = bridget_transport::provider_catalogue::WorkerCatalogueUpdate {
        version: 1,
        worker: st.remote_workers["local"].identity.clone(),
        connection_epoch: "epoch221-models".into(),
        refresh_id: "epoch221-models".into(),
        sequence: 3,
        catalogue: unidentified,
    };
    assert!(update.valid());
    assert!(catalogue::ingest(&mut st, "worker221-models", update).is_none());
    let unknown = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(unknown[0]["status"], "reported_models");
    assert_eq!(unknown[0]["auth"], "not_verified");
    assert!(unknown[0]["account_scope"].is_null());
    assert_eq!(unknown[0]["models"], changed[0]["models"]);
    assert_eq!(serde_json::to_vec(&unknown[1]).unwrap(), untouched);
    assert_eq!(
        serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
        tasks
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_l2a_maximum_declared_connections_and_observed_models_fit_one_daemon_frame() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::*;
    use bridget_transport::provider_catalogue::*;
    let (shared, config, _rx, _reader, _request, _root) = super::tests::routing168_fixture(1);
    let mut st = shared.lock().unwrap();
    let now = unix_timestamp();
    let clock = Instant::now();
    let mut raw: Value =
        serde_json::from_str(&std::fs::read_to_string(st.registry.source()).unwrap()).unwrap();
    let model = raw["routing_policy"]["model_catalog"]["main"][0].clone();
    let machine_policy = raw["routing_policy"]["machine_policies"]["local"].clone();
    raw["routing_policy"]["connections"] = json!([]);
    raw["routing_policy"]["model_catalog"] = json!({});
    raw["routing_policy"]["machine_policies"] = json!({});
    raw["machines"] = json!({});
    let digest = st
        .registry
        .resolved_definition("native-test")
        .unwrap()
        .digest;
    for machine in 0..4 {
        let id = format!("m{machine}-{}", "s".repeat(240));
        let worker = WorkerIdentity {
            machine_id: id.clone(),
            worker_id: format!("worker-{machine}"),
            provisioning_revision: 1,
        };
        raw["machines"][&id] = json!({"enabled":true,"worker_id":worker.worker_id,"provisioning_revision":1,
                "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["native-test"],"mappings":{}});
        raw["routing_policy"]["machine_policies"][&id] = machine_policy.clone();
        let mut cache = HashMap::new();
        for connection in 0..32 {
            let connection_id = format!("c{machine}-{connection:02}-{}", "c".repeat(240));
            raw["routing_policy"]["connections"].as_array_mut().unwrap().push(json!({
                    "connection_id":connection_id,"machine_id":id,"engine":"codex","agent_type":"native-test","enabled":true}));
            raw["routing_policy"]["model_catalog"][&connection_id] = json!([model.clone()]);
            let value = ProviderCatalogue {
                connection_id: connection_id.clone(),
                definition_digest: digest.clone(),
                observed_at: now,
                state: CatalogueState::Observed,
                account_scope: Some("b".repeat(32)),
                cause: None,
                models: (0..MAX_CATALOGUE_MODELS)
                    .map(|n| CatalogueModel {
                        service_tiers_unavailable: false,
                        model: format!("m{n:03}-{}", "n".repeat(180)),
                        supported_efforts: vec![],
                        default_effort: None,
                        service_tiers: vec![],
                        default_service_tier: None,
                    })
                    .collect(),
            };
            let update = WorkerCatalogueUpdate {
                version: 1,
                worker: worker.clone(),
                connection_epoch: "epoch221".into(),
                refresh_id: "epoch221".into(),
                sequence: 1,
                catalogue: value.clone(),
            };
            assert!(update.valid(), "only valid source envelopes count");
            cache.insert(
                connection_id,
                catalogue::CachedCatalogue {
                    sequence: 1,
                    received: clock,
                    received_at: now,
                    value,
                },
            );
        }
        st.remote_workers.insert(
            id,
            RemoteWorkerConnection {
                identity: worker,
                conn_id: format!("conn-{machine}"),
                epoch: "epoch221".into(),
                capabilities: vec![WorkerCapability::ProviderCatalogueV1],
                catalogues: cache,
                catalogue_refresh_id: "epoch221".into(),
                catalogue_refresh_at: None,
                contact: WorkerContactState::Connected,
                last_contact: clock,
                last_contact_at: now,
            },
        );
    }
    std::fs::write(st.registry.source(), raw.to_string()).unwrap();
    st.registry = AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf()).unwrap();
    let value = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(value.len(), 128);
    assert_eq!(
        value
            .iter()
            .map(|v| v["models"].as_array().unwrap().len())
            .sum::<usize>(),
        128 * MAX_CATALOGUE_MODELS
    );
    let result = catalogue::view(&st).unwrap();
    let mut frame =
        serde_json::to_vec(&DaemonToWrapper::NativeDelegationResult { result }).unwrap();
    frame.push(b'\n');
    let bytes = frame.len();
    eprintln!("catalogue221 maximum valid full frame {bytes} bytes");
    assert!(bytes <= bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES);
    let decoded: DaemonToWrapper = serde_json::from_slice(&frame[..frame.len() - 1]).unwrap();
    let DaemonToWrapper::NativeDelegationResult { result } = decoded else {
        panic!("native result");
    };
    assert_eq!(
        result["connection_catalogues"].as_array().unwrap().len(),
        128
    );
    for worker in st.remote_workers.values_mut() {
        worker.capabilities.push(WorkerCapability::ModelOptionsV1);
        for cached in worker.catalogues.values_mut() {
            for (n, model) in cached.value.models.iter_mut().enumerate() {
                model.model = format!("m{n:03}-{}", "n".repeat(100));
                model.service_tiers = vec!["default".into(), "priority".into(), "flex".into()];
                model.default_service_tier = Some("default".into());
            }
            assert!(
                cached.value.valid(),
                "le maximum reste un catalogue source valide"
            );
        }
    }
    let result = catalogue::view(&st).unwrap();
    assert_eq!(
        result["connection_catalogues"][0]["model_options_status"],
        "reported"
    );
    let mut options_frame =
        serde_json::to_vec(&DaemonToWrapper::NativeDelegationResult { result }).unwrap();
    options_frame.push(b'\n');
    eprintln!(
        "catalogue221 maximum model options frame {} bytes",
        options_frame.len()
    );
    assert!(options_frame.len() <= bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES);
    assert!(
        serde_json::from_slice::<DaemonToWrapper>(&options_frame[..options_frame.len() - 1])
            .is_ok()
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_l2a_stop_constraint_is_kept_on_registry_failure_and_policy_disable() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::provider_catalogue::*;
    let (shared, config, now, clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let path = st.registry.source().to_path_buf();
    let original = std::fs::read_to_string(&path).unwrap();
    let value = st.remote_workers["local"].catalogues["main"].value.clone();
    let worker = st.remote_workers["local"].identity.clone();
    let update = WorkerCatalogueUpdate {
        version: 1,
        worker,
        connection_epoch: "epoch221-models".into(),
        refresh_id: "epoch221-models".into(),
        sequence: 2,
        catalogue: value,
    };
    {
        let cache = st
            .remote_workers
            .get_mut("local")
            .unwrap()
            .catalogues
            .get_mut("main")
            .unwrap();
        cache.value.state = CatalogueState::Unavailable;
        cache.value.models.clear();
        cache.value.account_scope = None;
        cache.value.cause = Some(CatalogueCause::StopUnconfirmed);
    }
    std::fs::write(&path, b"PRIVATE_INVALID_REGISTRY").unwrap();
    assert!(catalogue::ingest(&mut st, "worker221-models", update.clone()).is_none());
    assert_eq!(
        st.remote_workers["local"].catalogues["main"].value.cause,
        Some(CatalogueCause::StopUnconfirmed)
    );
    assert_eq!(
        catalogue::view(&st).unwrap_err(),
        "routing_catalogue_unavailable"
    );
    let mut raw: Value = serde_json::from_str(&original).unwrap();
    raw["routing_policy"]["enabled"] = false.into();
    std::fs::write(&path, raw.to_string()).unwrap();
    assert!(catalogue::ingest(&mut st, "worker221-models", update.clone()).is_none());
    assert_eq!(
        st.remote_workers["local"].catalogues["main"].value.cause,
        Some(CatalogueCause::StopUnconfirmed)
    );
    raw["routing_policy"]["enabled"] = true.into();
    // La politique provisoire retire main mais doit rester valide.
    let saved = raw["routing_policy"]["connections"][0].clone();
    let models = raw["routing_policy"]["model_catalog"]["main"].clone();
    raw["routing_policy"]["connections"][0]["connection_id"] = "temporary".into();
    raw["routing_policy"]["model_catalog"]
        .as_object_mut()
        .unwrap()
        .remove("main");
    raw["routing_policy"]["model_catalog"]["temporary"] = models.clone();
    std::fs::write(&path, raw.to_string()).unwrap();
    let mut absent = update.clone();
    absent.catalogue.connection_id = "temporary".into();
    assert!(catalogue::ingest(&mut st, "worker221-models", absent).is_none());
    assert_eq!(
        st.remote_workers["local"].catalogues["main"].value.cause,
        Some(CatalogueCause::StopUnconfirmed)
    );
    raw["routing_policy"]["connections"][0] = saved;
    raw["routing_policy"]["model_catalog"]
        .as_object_mut()
        .unwrap()
        .remove("temporary");
    raw["routing_policy"]["model_catalog"]["main"] = models;
    raw["agents"]["native-test"]["args"] = json!(["app-server", "--changed-fixture"]);
    std::fs::write(&path, raw.to_string()).unwrap();
    st.registry = AgentRegistry::load_from_path_quiet(path.clone()).unwrap();
    let changed = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(changed[0]["status"], "definition_changed");
    let next = changed[0]["next_action"].as_str().unwrap();
    assert!(
        next.contains("définition")
            && next.contains("qualifier")
            && next.contains("nouvelle observation")
    );
    assert_eq!(changed[0]["probe_constraint"]["state"], "stop_unconfirmed");
    assert!(
        changed[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("marqueur privé")
    );
    raw["routing_policy"]["connections"][0]["enabled"] = false.into();
    std::fs::write(&path, raw.to_string()).unwrap();
    st.registry = AgentRegistry::load_from_path_quiet(path).unwrap();
    let disabled = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(disabled[0]["status"], "disabled");
    assert_eq!(disabled[0]["models"], json!([]));
    assert_eq!(disabled[0]["probe_constraint"]["state"], "stop_unconfirmed");
    assert!(
        disabled[0]["next_action"]
            .as_str()
            .unwrap()
            .contains("arrêt de sonde reste non confirmé")
    );
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec221_l2a_unavailable_global_catalogue_never_implies_probe_stopped() {
    for code in [
        "routing_policy_disabled",
        "routing_policy_unavailable",
        "routing_catalogue_unavailable",
    ] {
        let refusal = failure(code);
        assert_eq!(refusal["code"], code);
        assert_eq!(refusal["detail"]["probe_stop"], "not_verified");
        assert!(
            refusal["message"]
                .as_str()
                .unwrap()
                .contains("ne confirme aucun arrêt")
        );
        assert!(
            refusal["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("marqueurs privés")
        );
        let mut result = json!({});
        catalogue_scope212(&mut result, Err(code.to_owned()), "local");
        assert!(
            result["fleet"]["next_action"]
                .as_str()
                .unwrap()
                .contains("marqueurs privés")
        );
        assert_eq!(result["fleet"]["options"], json!([]));
    }
}

#[test]
fn spec221_l2a_refresh_over_connection_bound_sends_no_control_or_probe() {
    use super::super::native_provider_catalogue220 as catalogue;
    let (shared, config, now, clock) = observed221_fixture();
    let mut st = shared.lock().unwrap();
    let mut raw: Value =
        serde_json::from_str(&std::fs::read_to_string(st.registry.source()).unwrap()).unwrap();
    let model = raw["routing_policy"]["model_catalog"]["main"].clone();
    for n in 0..32 {
        let id = format!("inactive-{n}");
        raw["routing_policy"]["connections"].as_array_mut().unwrap().push(json!({
                "connection_id":id,"machine_id":"local","engine":"codex","agent_type":"native-test","enabled":false}));
        raw["routing_policy"]["model_catalog"][&id] = model.clone();
    }
    std::fs::write(st.registry.source(), raw.to_string()).unwrap();
    st.registry = AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf()).unwrap();
    let (a, b) = UnixStream::pair().unwrap();
    st.connections.insert(
        "worker221-models".into(),
        Arc::new(Mutex::new(std::io::BufWriter::new(a))),
    );
    let before = st.remote_workers["local"].catalogue_refresh_id.clone();
    let mut controls = Vec::new();
    let value = catalogue::refresh(&mut st, Some("local"), &mut controls).unwrap();
    assert_eq!(value["workers"][0]["status"], "connection_limit");
    assert!(
        value["workers"][0]["next_action"]
            .as_str()
            .unwrap()
            .contains("désactivées comprises")
    );
    assert!(controls.is_empty());
    assert_eq!(st.remote_workers["local"].catalogue_refresh_id, before);
    let rows = catalogue::project_at(&st, &st.registry, now, clock);
    assert_eq!(
        rows.iter()
            .find(|row| row["connection_id"] == "main")
            .unwrap()["status"],
        "connection_limit"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["status"] == "disabled")
            .count(),
        32
    );
    drop(b);
    drop(st);
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec220_refresh_requires_native_identity_or_negotiated_same_uid_client() {
    let (shared, config, _rx, _reader, _request, _root) = super::tests::routing168_fixture(1);
    let denied = handle(
        "unknown",
        NativeDelegationRequest::ProviderCatalogueV1 {
            refresh: true,
            machine_id: None,
        },
        &shared,
    );
    assert!(
        matches!(denied,DaemonToWrapper::NativeDelegationResult {result} if result["status"]=="refused")
    );
    {
        let mut st = shared.lock().unwrap();
        st.connection_roles
            .insert("human220".into(), ConnectionRole::Client);
        st.peer_uids.insert(
            "human220".into(),
            bridget_transport::fsutil::effective_uid(),
        );
        st.client_negotiations.insert(
            "human220".into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "test220".into(),
                capabilities: vec![ClientCapability::ProviderCatalogueV1],
            },
        );
    }
    let read = handle(
        "human220",
        NativeDelegationRequest::ProviderCatalogueV1 {
            refresh: false,
            machine_id: None,
        },
        &shared,
    );
    assert!(
        matches!(read,DaemonToWrapper::NativeDelegationResult {result} if result["connection_catalogues"].is_array())
    );
    let filtered = handle(
        "human220",
        NativeDelegationRequest::ProviderCatalogueV1 {
            refresh: false,
            machine_id: Some("different".into()),
        },
        &shared,
    );
    assert!(
        matches!(filtered,DaemonToWrapper::NativeDelegationResult {result} if result["options"]==json!([]) && result["connection_catalogues"]==json!([]))
    );
    {
        let mut st = shared.lock().unwrap();
        let (a, b) = UnixStream::pair().unwrap();
        drop(b);
        st.connections.insert(
            "worker220".into(),
            Arc::new(Mutex::new(std::io::BufWriter::new(a))),
        );
        st.remote_workers.insert(
            "local".into(),
            RemoteWorkerConnection {
                identity: bridget_transport::protocol::WorkerIdentity {
                    machine_id: "local".into(),
                    worker_id: "worker220".into(),
                    provisioning_revision: 1,
                },
                conn_id: "worker220".into(),
                epoch: "epoch".into(),
                capabilities: vec![
                    bridget_transport::protocol::WorkerCapability::ProviderCatalogueV1,
                ],
                catalogues: HashMap::new(),
                catalogue_refresh_id: "epoch".into(),
                catalogue_refresh_at: None,
                contact: bridget_transport::protocol::WorkerContactState::Connected,
                last_contact: Instant::now(),
                last_contact_at: unix_timestamp(),
            },
        );
    }
    let broken = handle(
        "human220",
        NativeDelegationRequest::ProviderCatalogueV1 {
            refresh: true,
            machine_id: Some("local".into()),
        },
        &shared,
    );
    assert!(
        matches!(broken,DaemonToWrapper::NativeDelegationResult {result} if result["status"]=="unconfirmed" && result["code"]=="provider_catalogue_refresh_unconfirmed")
    );
    shared
        .lock()
        .unwrap()
        .client_negotiations
        .get_mut("human220")
        .unwrap()
        .version = 0;
    let stale = handle(
        "human220",
        NativeDelegationRequest::ProviderCatalogueV1 {
            refresh: false,
            machine_id: None,
        },
        &shared,
    );
    assert!(
        matches!(stale,DaemonToWrapper::NativeDelegationResult {result} if result["status"]=="refused")
    );
    drop(shared);
    super::tests::cleanup(&config);
}

#[test]
fn spec220_hello_clears_old_epoch_then_restores_retained_constraint() {
    use super::super::native_provider_catalogue220 as catalogue;
    use bridget_transport::protocol::*;
    use bridget_transport::provider_catalogue::*;
    use sha2::{Digest, Sha256};
    let (shared, config, _rx, _reader, _request, _root) = super::tests::routing168_fixture(1);
    let credential = IdentityCredential::new("worker220-credential".into());
    let worker = WorkerIdentity {
        machine_id: "local".into(),
        worker_id: "worker220".into(),
        provisioning_revision: 1,
    };
    let value = {
        let mut st = shared.lock().unwrap();
        let path = st.registry.source().to_path_buf();
        let mut raw: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        raw["machines"] = json!({"local":{"enabled":true,"worker_id":"worker220","provisioning_revision":1,"credential_fingerprint":format!("{:x}",Sha256::digest(b"worker220-credential")),"allowed_agent_types":["native-test"],"mappings":{}}});
        std::fs::write(&path, raw.to_string()).unwrap();
        st.registry = AgentRegistry::load_from_path_quiet(path).unwrap();
        let value = ProviderCatalogue {
            connection_id: "main".into(),
            definition_digest: st
                .registry
                .resolved_definition("native-test")
                .unwrap()
                .digest,
            observed_at: unix_timestamp(),
            state: CatalogueState::Unavailable,
            account_scope: None,
            models: vec![],
            cause: Some(CatalogueCause::StopUnconfirmed),
        };
        let mut cache = HashMap::new();
        cache.insert(
            "main".into(),
            catalogue::CachedCatalogue {
                sequence: 9,
                received: Instant::now(),
                received_at: unix_timestamp(),
                value: value.clone(),
            },
        );
        st.remote_workers.insert(
            "local".into(),
            RemoteWorkerConnection {
                identity: worker.clone(),
                conn_id: "old-worker".into(),
                epoch: "old-epoch".into(),
                capabilities: vec![WorkerCapability::ProviderCatalogueV1],
                catalogues: cache,
                catalogue_refresh_id: "old-epoch".into(),
                catalogue_refresh_at: None,
                last_contact: Instant::now(),
                last_contact_at: unix_timestamp(),
                contact: WorkerContactState::Connected,
            },
        );
        st.peer_uids.insert(
            "new-worker".into(),
            bridget_transport::fsutil::effective_uid(),
        );
        value
    };
    let accepted = super::super::handle_worker_message(
        &shared,
        "new-worker",
        WrapperToDaemon::WorkerHelloWithCapabilities {
            hello: WorkerHelloWithCapabilities {
                hello: WorkerHello {
                    version: REMOTE_WORKER_VERSION,
                    identity: worker.clone(),
                    credential,
                    build: "220".into(),
                    capability: ClientCapability::RemoteWorkerV1,
                },
                capabilities: vec![WorkerCapability::ProviderCatalogueV1],
            },
        },
    );
    let Some(DaemonToWrapper::WorkerAcceptedWithCapabilities { accepted }) = accepted else {
        panic!("negotiation expected");
    };
    let epoch = accepted.accepted.connection_epoch;
    {
        let st = shared.lock().unwrap();
        assert!(st.remote_workers["local"].catalogues.is_empty());
        let pending = catalogue::project(&st, &st.registry);
        assert_eq!(pending[0]["status"], "not_observed");
        assert!(
            pending[0]["probe_constraint"].is_null(),
            "ne pas inventer un état d'arrêt depuis un cache vide"
        );
        let advice = pending[0]["next_action"].as_str().unwrap();
        assert!(
            advice.contains("marqueurs privés"),
            "la reconnexion réelle doit conserver le prérequis de maintenance"
        );
        assert!(advice.contains("ne confirme aucun arrêt"));
        assert!(advice.contains("changement de version"));
        assert!(advice.contains("autres travaux autorisés"));
    }
    assert!(
        super::super::handle_worker_message(
            &shared,
            "new-worker",
            WrapperToDaemon::ProviderCatalogueUpdate {
                update: WorkerCatalogueUpdate {
                    version: 1,
                    worker,
                    connection_epoch: epoch.clone(),
                    refresh_id: epoch,
                    sequence: 1,
                    catalogue: value
                }
            }
        )
        .is_none()
    );
    {
        let mut st = shared.lock().unwrap();
        let observed = catalogue::project(&st, &st.registry);
        assert_eq!(observed[0]["status"], "unavailable");
        assert_eq!(observed[0]["probe_constraint"]["state"], "stop_unconfirmed");
        let mut raw: Value =
            serde_json::from_str(&std::fs::read_to_string(st.registry.source()).unwrap()).unwrap();
        raw["machines"]["local"]["enabled"] = json!(false);
        st.registry =
            AgentRegistry::from_json(&raw.to_string(), st.registry.source().to_path_buf()).unwrap();
        let disabled = catalogue::project(&st, &st.registry);
        assert_eq!(disabled[0]["status"], "machine_disabled");
        assert_eq!(disabled[0]["probe_constraint"]["state"], "stop_unconfirmed");
    }
    drop(shared);
    super::tests::cleanup(&config);
}
