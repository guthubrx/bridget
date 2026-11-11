fn worker162_identity() -> WorkerIdentity {
    WorkerIdentity {
        machine_id: "cartae".into(),
        worker_id: "worker-cartae".into(),
        provisioning_revision: 1,
    }
}

fn worker162_command() -> WorkerCommandIdentity {
    WorkerCommandIdentity {
        command_id: "command".into(),
        task_id: "task".into(),
        instance_id: "instance".into(),
        generation: 1,
    }
}

fn worker162_result_identity() -> WorkerResultIdentity {
    WorkerResultIdentity {
        command: worker162_command(),
        mission_id: "mission".into(),
        round: 0,
        execution_id: "execution-mission".into(),
        execution_generation: 1,
    }
}

#[test]
fn remote172_source_proof_shape_bornee_et_parent_historique_intact() {
    let parent = ProviderPermissions {
        version: 1,
        source: "native_wrapper".into(),
        run_id: "mission".into(),
        provider_session_id: "session".into(),
        provider_instance_id: "instance".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: "/remote/project".into(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never", "approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
    };
    let source = WorkerSourceBase {
        cwd: parent.cwd.clone(),
        expected_base_commit: Some("a".repeat(40)),
        workspace: Some(WorkspaceRequest::Shared {}),
        workload: AutoDelegateWorkload::General,
    };
    let mut request = WorkerRequest {
        version: REMOTE_WORKER_VERSION,
        correlation_id: "source-proof".into(),
        connection_epoch: "epoch".into(),
        worker: worker162_identity(),
        command: worker162_command(),
        request: WorkerRequestOperation::ParentSourceProof {
            parent: parent.clone(),
            source: source.clone(),
        },
    };
    assert_eq!(request.validate(), Ok(()));
    for invalid in ["/other", "/remote/project/../other"] {
        if let WorkerRequestOperation::ParentSourceProof { source, .. } = &mut request.request {
            source.cwd = invalid.into();
        }
        assert_eq!(request.validate(), Err(WorkerRefusal::InvalidEnvelope));
    }
    request.request = WorkerRequestOperation::ParentSourceProof {
        parent: parent.clone(),
        source: WorkerSourceBase {
            expected_base_commit: Some("x".repeat(40)),
            ..source
        },
    };
    assert_eq!(request.validate(), Err(WorkerRefusal::InvalidEnvelope));
    let old = WorkerRequestOperation::ParentProof {
        parent: parent.clone(),
        expected_revision: 1,
    };
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::json!({"operation":"parent_proof","parent":parent,"expected_revision":1})
    );
}

#[test]
fn remote162_handshake_role_distinct_capacite_et_secret_masque() {
    let hello = WorkerHello {
        version: 1,
        identity: worker162_identity(),
        credential: IdentityCredential::new("secret-worker-prive".into()),
        build: "build162".into(),
        capability: ClientCapability::RemoteWorkerV1,
    };
    assert_eq!(hello.validate(), Ok(()));
    let wire = encode(&WrapperToDaemon::WorkerHello {
        hello: hello.clone(),
    })
    .unwrap();
    let decoded: WrapperToDaemon = decode(&wire).unwrap();
    assert!(matches!(decoded, WrapperToDaemon::WorkerHello { .. }));
    assert!(!format!("{decoded:?}").contains("secret-worker-prive"));
    assert_eq!(encode(&ConnectionRole::Worker).unwrap(), "\"worker\"");
    assert_eq!(
        encode(&ClientCapability::RemoteWorkerV1).unwrap(),
        "\"remote_worker_v1\""
    );
    let mut wrong = hello.clone();
    wrong.version = 2;
    assert_eq!(wrong.validate(), Err(WorkerRefusal::UnsupportedVersion));
    wrong = hello;
    wrong.capability = ClientCapability::SendIdempotent;
    assert_eq!(wrong.validate(), Err(WorkerRefusal::CapabilityRequired));
    // Les négociations locales existantes ne gagnent aucun champ implicite.
    let local = r#"{"type":"RoleHandshake","role":"service"}"#;
    assert_eq!(
        encode(&decode::<WrapperToDaemon>(local).unwrap()).unwrap(),
        local
    );
}

#[test]
fn remote162_contrat_ferme_et_slot_correlations_avant_dispatch() {
    let slot = WorkerResultSlot {
        identity: worker162_result_identity(),
        admission_digest: "a".repeat(64),
    };
    let mut request = WorkerRequest {
        version: 1,
        correlation_id: "correlation".into(),
        connection_epoch: "epoch".into(),
        worker: worker162_identity(),
        command: worker162_command(),
        request: WorkerRequestOperation::ReserveResultSlot { slot: slot.clone() },
    };
    assert_eq!(request.validate(), Ok(()));
    let wire = encode(&request).unwrap();
    assert_eq!(decode::<WorkerRequest>(&wire).unwrap(), request);
    let mut extra = serde_json::to_value(&request).unwrap();
    extra["request"]["local_fallback"] = serde_json::json!(true);
    assert!(serde_json::from_value::<WorkerRequest>(extra).is_err());
    let mut unknown = serde_json::to_value(&request).unwrap();
    unknown["request"]["operation"] = serde_json::json!("spawn_without_receipt");
    assert!(serde_json::from_value::<WorkerRequest>(unknown).is_err());
    request.command.generation = 2;
    assert_eq!(request.validate(), Err(WorkerRefusal::InvalidEnvelope));
    request.command = worker162_command();
    request.request = WorkerRequestOperation::Cancel {
        target: WorkerCancelScope::ResultSlot {
            slot,
            proof: WorkerSlotClosureProof::NeverDispatched,
        },
    };
    assert_eq!(request.validate(), Ok(()));
    request.connection_epoch.clear();
    assert_eq!(request.validate(), Err(WorkerRefusal::InvalidEnvelope));
}

#[test]
fn remote162_resultat_borne_digest_canon_et_execution_quota() {
    let mut result = RemoteResultEnvelope {
        worker: worker162_identity(),
        identity: worker162_result_identity(),
        message_id: "result-message".into(),
        body: "résultat".into(),
        digest: String::new(),
    };
    result.digest = result.canonical_digest().unwrap();
    assert_eq!(result.validate(), Ok(()));
    let first = result.clone();
    result.identity.execution_id = "execution-quota-reprise".into();
    assert_eq!(result.validate(), Err(WorkerRefusal::InvalidEnvelope));
    result = first;
    result.body = "x".repeat(MAX_WORKER_RESULT_BODY_BYTES);
    result.digest = result.canonical_digest().unwrap();
    assert_eq!(result.validate(), Ok(()));
    result.body.push('x');
    result.digest = result.canonical_digest().unwrap();
    assert_eq!(result.validate(), Err(WorkerRefusal::InvalidEnvelope));
    // Le budget compte l'encodage JSON réel, y compris les échappements.
    result.body = "\u{0001}".repeat(MAX_WORKER_RESULT_BODY_BYTES);
    result.digest = result.canonical_digest().unwrap();
    assert!(encode(&result).unwrap().len() > MAX_WORKER_RESULT_BODY_BYTES * 5);
    assert_eq!(result.validate(), Ok(()));
    assert!(encode(&result).unwrap().len() <= MAX_WORKER_RESULT_ENVELOPE_BYTES);
}

#[test]
fn remote162_dispositions_fermees_sans_faux_succes_quota() {
    let final_states = [
        RemoteResultDisposition::Accepted {},
        RemoteResultDisposition::DiscardedTerminal {
            reason: RemoteResultDiscardReason::Cancelled,
        },
        RemoteResultDisposition::RejectedConflict {},
    ];
    for disposition in final_states {
        assert!(disposition.is_final());
        let wire = encode(&disposition).unwrap();
        assert_eq!(
            decode::<RemoteResultDisposition>(&wire).unwrap(),
            disposition
        );
    }
    let deferred = RemoteResultDisposition::Deferred {
        reason: RemoteResultDeferredReason::QuotaHold,
        retry_after: 123,
    };
    assert!(!deferred.is_final());
    assert!(
        decode::<RemoteResultDisposition>(
            r#"{"status":"discarded_terminal","reason":"quota_hold"}"#
        )
        .is_err()
    );
    assert!(decode::<RemoteResultDisposition>(r#"{"status":"accepted","ignored":true}"#).is_err());
    assert!(
        decode::<RemoteResultDisposition>(r#"{"status":"rejected_conflict","ignored":true}"#)
            .is_err()
    );
    assert!(
        decode::<WorkerRequestOperation>(r#"{"operation":"reconcile","ignored":true}"#).is_err()
    );
    assert!(
        decode::<WorkerResponseOperation>(r#"{"operation":"released","ignored":true}"#).is_err()
    );
    assert!(decode::<WorkerCancelScope>(r#"{"scope":"command","ignored":true}"#).is_err());
}

#[test]
fn remote162_placement_chemins_revisions_et_permis_bornes() {
    let mut placement = RemotePlacement {
        machine_id: "cartae".into(),
        worker_id: "worker-cartae".into(),
        provisioning_revision: 1,
        project_id: "project".into(),
        mapping_revision: 1,
        origin_host: "mac".into(),
        origin_cwd: "/Users/moi/project".into(),
        source_root_identity: "git:project".into(),
        relative_cwd: ".".into(),
        execution_cwd: "/home/moi/project".into(),
        base_commit: Some("a".repeat(40)),
        definition_digest: "b".repeat(64),
        permission_digest: "c".repeat(64),
    };
    assert_eq!(placement.validate(), Ok(()));
    assert!(placement.matches_worker(&worker162_identity()));
    placement.relative_cwd = "src/../../outside".into();
    assert_eq!(placement.validate(), Err(WorkerRefusal::InvalidEnvelope));
    placement.relative_cwd = ".".into();
    placement.mapping_revision = HUMAN_LINEAGE_MAX_SEQ + 1;
    assert_eq!(placement.validate(), Err(WorkerRefusal::InvalidEnvelope));
    let mut permit = WorkerReleasePermit {
        permit_id: "permit".into(),
        command_digest: "a".repeat(64),
        permission_revision: 1,
        mapping_revision: 1,
        provisioning_revision: 1,
        issued_at: 100,
        expires_at: 105,
    };
    assert_eq!(permit.validate(), Ok(()));
    permit.expires_at = 100 + WORKER_RELEASE_LEASE_SECS;
    assert_eq!(permit.validate(), Ok(()));
    permit.expires_at += 1;
    assert_eq!(permit.validate(), Err(WorkerRefusal::InvalidEnvelope));
    permit.issued_at = i64::MAX;
    permit.expires_at = i64::MIN;
    assert_eq!(permit.validate(), Err(WorkerRefusal::InvalidEnvelope));
}

#[test]
fn remote162_journal_et_reponse_bornes_sans_chemin_fourni() {
    let request = WorkerRequest {
        version: 1,
        correlation_id: "journal".into(),
        connection_epoch: "epoch".into(),
        worker: worker162_identity(),
        command: worker162_command(),
        request: WorkerRequestOperation::JournalPage {
            after_seq: 0,
            limit: 200,
        },
    };
    assert_eq!(request.validate(), Ok(()));
    let mut value = serde_json::to_value(&request).unwrap();
    value["request"]["path"] = serde_json::json!("/other-agent/journal");
    assert!(serde_json::from_value::<WorkerRequest>(value).is_err());
    let mut response = WorkerResponse {
        version: 1,
        correlation_id: "journal".into(),
        connection_epoch: "epoch".into(),
        worker: worker162_identity(),
        command: worker162_command(),
        response: WorkerResponseOperation::JournalPage {
            fragments: vec![WorkerJournalFragment {
                seq: 1,
                offset: 0,
                final_fragment: true,
                bytes: vec![b'x'],
            }],
            through_seq: Some(1),
            more: false,
        },
    };
    assert_eq!(response.validate(), Ok(()));
    if let WorkerResponseOperation::JournalPage { fragments, .. } = &mut response.response {
        fragments[0].bytes = vec![b'x'; MAX_ATTACH_FRAGMENT_BYTES + 1];
    }
    assert_eq!(response.validate(), Err(WorkerRefusal::InvalidEnvelope));
}

#[test]
fn spec221_c05_auto_selection_preserves_the_legacy_canon_and_rejects_null() {
    let input = serde_json::json!({"request_id":"canonical-options","task":"Lire","cwd":"/tmp"});
    let request: AutoDelegateV2 = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(
        serde_json::to_string(&request).unwrap(),
        r#"{"request_id":"canonical-options","workload":"general","task":"Lire","cwd":"/tmp"}"#
    );
    assert!(request.service_tier.is_none());
    let mut explicit = input.clone();
    explicit["service_tier"] = serde_json::json!("priority");
    let selected: AutoDelegateV2 = serde_json::from_value(explicit).unwrap();
    assert_eq!(selected.service_tier.as_deref(), Some("priority"));
    assert_eq!(
        serde_json::to_value(selected).unwrap()["service_tier"],
        "priority"
    );
    let mut invalid = input;
    invalid["service_tier"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<AutoDelegateV2>(invalid).is_err());
}

#[test]
fn spec168_prepare_v2_contrat_ferme_et_prepare_v1_inchange() {
    let worker = worker162_identity();
    let mut command = worker162_command();
    command.instance_id.clear();
    command.generation = 0;
    let prepare = WorkerPrepare {
        service_tier: None,
        machine_id: worker.machine_id.clone(),
        project_id: "project168".into(),
        provisioning_revision: 1,
        mapping_revision: 1,
        origin_host: "origin168".into(),
        origin_cwd: "/origin/project".into(),
        source_root: "/origin/project".into(),
        remote_root: "/remote/project".into(),
        source_root_identity: "repo168".into(),
        relative_cwd: ".".into(),
        base_commit: Some("a".repeat(40)),
        agent_type: "codex".into(),
        model: "gpt-5.6-terra".into(),
        effort: Some("high".into()),
        posture: Some(SpawnPosture::Discovery),
        workspace: Some(WorkspaceRequest::Shared {}),
        parent: ProviderPermissions {
            version: 1,
            source: "native_wrapper".into(),
            run_id: "run168".into(),
            provider_session_id: "session168".into(),
            provider_instance_id: "parent168".into(),
            revision: 1,
            driver: "codex_app_server".into(),
            cwd: "/origin/project".into(),
            runtime_mode: "full-access".into(),
            interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
        },
    };
    let mut request = WorkerRequest {
        version: 1,
        correlation_id: "prepare168".into(),
        connection_epoch: "epoch168".into(),
        worker,
        command,
        request: WorkerRequestOperation::Prepare {
            prepare: prepare.clone(),
        },
    };
    assert_eq!(request.validate(), Ok(()));
    let legacy = serde_json::to_value(&request).unwrap();
    {
        let mut diagnosed = request.clone();
        diagnosed.request = WorkerRequestOperation::PrepareDiagnosed {
            prepare: Box::new(request.request.clone()),
        };
        diagnosed.validate().unwrap();
        let wire = serde_json::to_vec(&diagnosed).unwrap();
        assert_eq!(
            serde_json::from_slice::<WorkerRequest>(&wire).unwrap(),
            diagnosed
        );
        let mut nested = diagnosed.clone();
        nested.request = WorkerRequestOperation::PrepareDiagnosed {
            prepare: Box::new(diagnosed.request.clone()),
        };
        assert_eq!(nested.validate(), Err(WorkerRefusal::InvalidEnvelope));
        let mut no_authority = diagnosed;
        no_authority.request = WorkerRequestOperation::PrepareDiagnosed {
            prepare: Box::new(WorkerRequestOperation::Reconcile {}),
        };
        assert_eq!(no_authority.validate(), Err(WorkerRefusal::InvalidEnvelope));
        let f = WorkspacePrepareFailure {
            stage: WorkspacePrepareStage::CompactPlan,
            cause: WorkspacePrepareCause::GitTimeout,
        };
        assert!(f.validate(WorkerRefusal::WorkspacePrepareFailed).is_ok());
        assert_eq!(
            f.validate(WorkerRefusal::DiskFull),
            Err(WorkerRefusal::InvalidEnvelope)
        );
    }
    assert_eq!(legacy["request"]["operation"], "prepare");
    assert!(legacy["request"].get("connection_id").is_none());
    request.request = WorkerRequestOperation::PrepareV2 {
        prepare,
        inspect_only: false,
        workload: AutoDelegateWorkload::General,
        connection_id: "connection168".into(),
        engine: "codex".into(),
        expected_definition_digest: "a".repeat(64),
        expected_permission_digest: "b".repeat(64),
    };
    assert_eq!(request.validate(), Ok(()));
    let current = serde_json::to_value(&request).unwrap();
    assert_eq!(current["request"]["prepare"], legacy["request"]["prepare"]);
    assert_eq!(current["request"]["operation"], "prepare_v2");
    assert!(current["request"].get("inspect_only").is_none());
    let mut inspection = current.clone();
    inspection["request"]["inspect_only"] = serde_json::json!(true);
    assert_eq!(
        serde_json::from_value::<WorkerRequest>(inspection.clone())
            .unwrap()
            .validate(),
        Err(WorkerRefusal::InvalidEnvelope),
        "Shared ne peut pas emprunter le contrôle Worktree sans effets"
    );
    inspection["request"]["prepare"]["workspace"] =
        serde_json::json!({"strategy":"worktree","base_ref":"HEAD"});
    let decoded = serde_json::from_value::<WorkerRequest>(inspection.clone()).unwrap();
    assert_eq!(decoded.validate(), Ok(()));
    assert_eq!(serde_json::to_value(decoded).unwrap(), inspection);
    assert_eq!(
        serde_json::from_value::<WorkerRequest>(current.clone()).unwrap(),
        request
    );
    for (field, value) in [
        ("connection_id", serde_json::json!("")),
        ("engine", serde_json::json!("other")),
        ("expected_definition_digest", serde_json::json!("bad")),
        ("expected_permission_digest", serde_json::json!("bad")),
    ] {
        let mut invalid = current.clone();
        invalid["request"][field] = value;
        assert!(
            serde_json::from_value::<WorkerRequest>(invalid)
                .unwrap()
                .validate()
                .is_err(),
            "{field}"
        );
    }
    let mut unknown = current.clone();
    unknown["request"]["credential_path"] = serde_json::json!("/private");
    assert!(serde_json::from_value::<WorkerRequest>(unknown).is_err());
    let mut fenced = serde_json::from_value::<WorkerRequest>(current.clone()).unwrap();
    fenced.command.command_id = "new-prepare198".into();
    let WorkerRequestOperation::PrepareV2 { mut prepare, .. } = fenced.request else {
        unreachable!()
    };
    prepare.workspace = Some(WorkspaceRequest::Worktree {
        base_ref: None,
        branch: None,
    });
    let placement = RemotePlacement {
        machine_id: fenced.worker.machine_id.clone(),
        worker_id: fenced.worker.worker_id.clone(),
        provisioning_revision: fenced.worker.provisioning_revision,
        project_id: prepare.project_id.clone(),
        mapping_revision: prepare.mapping_revision,
        origin_host: prepare.origin_host.clone(),
        origin_cwd: prepare.origin_cwd.clone(),
        source_root_identity: prepare.source_root_identity.clone(),
        relative_cwd: prepare.relative_cwd.clone(),
        execution_cwd: "/remote/project/.worktrees/existing198".into(),
        base_commit: prepare.base_commit.clone(),
        definition_digest: "a".repeat(64),
        permission_digest: "b".repeat(64),
    };
    fenced.request = WorkerRequestOperation::PrepareFenced {
        previous: worker162_command(),
        prepare,
        placement,
        workspace: Some(serde_json::json!({"path":"/remote/project/.worktrees/existing198"})),
        routing: None,
    };
    fenced.validate().unwrap();
    let wire = serde_json::to_value(&fenced).unwrap();
    assert!(
        wire["request"].get("routing").is_none(),
        "reçu162 : aucun champ neuf"
    );
    let routing = WorkerFencedRouting {
        workload: AutoDelegateWorkload::General,
        connection_id: "connexion-198".into(),
        engine: "codex".into(),
        expected_definition_digest: "a".repeat(64),
        expected_permission_digest: "b".repeat(64),
    };
    let plain = fenced.clone();
    if let WorkerRequestOperation::PrepareFenced { routing: slot, .. } = &mut fenced.request {
        *slot = Some(routing.clone());
    }
    fenced.validate().unwrap();
    let routed = serde_json::to_value(&fenced).unwrap();
    assert_eq!(
        serde_json::from_value::<WorkerRequest>(routed).unwrap(),
        fenced
    );
    if let WorkerRequestOperation::PrepareFenced { routing: slot, .. } = &mut fenced.request {
        slot.as_mut().unwrap().engine = "autre".into();
    }
    assert_eq!(fenced.validate(), Err(WorkerRefusal::InvalidEnvelope));
    fenced = plain;
    assert_eq!(wire["request"]["operation"], "prepare_fenced");
    assert_eq!(
        serde_json::from_value::<WorkerRequest>(wire).unwrap(),
        fenced
    );
    if let WorkerRequestOperation::PrepareFenced { workspace, .. } = &mut fenced.request {
        *workspace = None;
    }
    assert_eq!(fenced.validate(), Err(WorkerRefusal::InvalidEnvelope));
    if let WorkerRequestOperation::PrepareFenced { prepare, .. } = &mut fenced.request {
        prepare.workspace = Some(WorkspaceRequest::Shared {});
    }
    fenced.validate().unwrap();
    assert_eq!(
        serde_json::to_value(&fenced).unwrap()["request"]["operation"],
        "prepare_fenced"
    );
    assert_eq!(legacy["request"]["operation"], "prepare");
    let mut missing = current;
    missing["request"]
        .as_object_mut()
        .unwrap()
        .remove("workload");
    assert!(serde_json::from_value::<WorkerRequest>(missing).is_err());
    request.command.instance_id = "instance".into();
    request.command.generation = 1;
    assert_eq!(request.validate(), Err(WorkerRefusal::InvalidEnvelope));
}

#[test]
fn spec198_start_capacity_cause_fermee_et_master_pending() {
    assert_eq!(
        WorkerStartCapacityCause::RequestFrame.code(),
        "request_frame"
    );
    assert_eq!(
        serde_json::to_value(WorkerStartCapacityCause::RequestFrame).unwrap(),
        "request_frame"
    );
    assert_eq!(
        WorkerStartCapacityCause::MasterPending.code(),
        "master_pending"
    );
    assert_eq!(
        serde_json::to_value(WorkerStartCapacityCause::MasterPending).unwrap(),
        "master_pending"
    );
    assert_eq!(
        serde_json::from_value::<WorkerStartCapacityCause>(serde_json::json!("master_pending"))
            .unwrap(),
        WorkerStartCapacityCause::MasterPending
    );
    assert!(
        serde_json::from_value::<WorkerStartCapacityCause>(serde_json::json!(
            "future_unknown_cause"
        ))
        .is_err()
    );
}

#[test]
fn remote162_observation_recontrole_borne_et_refus_prepare() {
    let mut response = WorkerResponse {
        version: 1,
        correlation_id: "reconcile".into(),
        connection_epoch: "epoch".into(),
        worker: worker162_identity(),
        command: worker162_command(),
        response: WorkerResponseOperation::Observation {
            observation: WorkerObservation {
                startup_failure: None,
                phase: WorkerCommandPhase::Released,
                native_ready: true,
                exit_code: None,
                observed_at: 100,
                workspace: Some(serde_json::json!({"path":"/workspace"})),
                permission_digest: Some("a".repeat(64)),
            },
        },
    };
    assert_eq!(response.validate(), Ok(()));
    if let WorkerResponseOperation::Observation { observation } = &mut response.response {
        observation.workspace = Some(serde_json::json!({"path":"x".repeat(65536)}));
    }
    assert_eq!(response.validate(), Err(WorkerRefusal::ResourceLimit));
    response.response = WorkerResponseOperation::Rejected {
        reason: WorkerRefusal::PermissionsChanged,
    };
    response.command.instance_id.clear();
    response.command.generation = 0;
    assert_eq!(response.validate(), Ok(()));
    response.response = WorkerResponseOperation::Released {};
    assert_eq!(response.validate(), Err(WorkerRefusal::InvalidEnvelope));
}

#[test]
fn spec099_credential_optionnel_compatible_et_masque_dans_debug() {
    use super::{DaemonToWrapper, IdentityCredential, WrapperToDaemon};
    let old: DaemonToWrapper =
        serde_json::from_str(r#"{"type":"Registered","agent_id":"agent"}"#).unwrap();
    assert!(matches!(
        old,
        DaemonToWrapper::Registered {
            credential: None,
            ..
        }
    ));
    let secret = "ne-doit-jamais-apparaitre-dans-les-logs";
    let credential = IdentityCredential::new(secret.into());
    let registration = WrapperToDaemon::RegisterAuxiliary {
        agent_id: "agent".into(),
        instance_id: "instance".into(),
        credential: credential.clone(),
    };
    let response = DaemonToWrapper::Registered {
        agent_id: "agent".into(),
        credential: Some(credential),
    };
    assert!(!format!("{registration:?} {response:?}").contains(secret));
    assert!(
        serde_json::to_string(&registration)
            .unwrap()
            .contains(secret)
    );
}

#[test]
fn spec091_selection_fermee_bornee_et_sans_permissions() {
    use super::{RuntimeSelection, RuntimeSelectionOutcome};
    let source = r#"{"model":"gpt-5.6-terra","effort":"medium"}"#;
    let selection: RuntimeSelection = serde_json::from_str(source).unwrap();
    assert!(selection.valid());
    assert_eq!(serde_json::to_string(&selection).unwrap(), source);
    for source in [
        r#"{"model":"x","effort":"high","approval":"never"}"#,
        r#"{"model":"x"}"#,
    ] {
        assert!(serde_json::from_str::<RuntimeSelection>(source).is_err());
    }
    for model in [
        String::new(),
        "x".repeat(129),
        "x\u{1b}[2J".into(),
        "x y".into(),
    ] {
        assert!(
            !RuntimeSelection {
                model,
                effort: "high".into()
            }
            .valid()
        );
    }
    assert!(
        serde_json::from_str::<RuntimeSelectionOutcome>(
            r#"{"status":"refused","reason":"unknown"}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<RuntimeSelectionOutcome>(
            r#"{"status":"outcome_unknown","applied":true}"#
        )
        .is_err()
    );
}
use super::*;

#[test]
fn terminal_session_ready_contrat_filaire() {
    let bytes = r#"{"type":"TerminalSessionReady"}"#;
    let decoded: WrapperToDaemon = decode(bytes).unwrap();
    assert!(matches!(decoded, WrapperToDaemon::TerminalSessionReady));
    assert_eq!(encode(&decoded).unwrap(), bytes);
}

const SERVICE_NEGOTIATION_FIXTURE: &str = include_str!(
    "../../../specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl"
);
const COORDINATION_EVENTS_FIXTURE: &str = include_str!(
    "../../../specs/016-coordination-active/contracts/fixtures/coordination-events-v1.jsonl"
);
const COORDINATION_STREAM_FIXTURE: &str = include_str!(
    "../../../specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl"
);

#[test]
fn contrat_revue_valide_ref_et_sha_fermes() {
    let valid = ReviewTarget {
        target_ref: "origin/fix/review/sub".to_string(),
        expected_head: "a".repeat(40),
    };
    assert_eq!(
        valid.remote_and_branch(),
        Some(("origin", "fix/review/sub"))
    );
    assert!(valid.is_valid());
    for target_ref in ["origin", "./main", "origin/../main", "-origin/main"] {
        assert!(
            !ReviewTarget {
                target_ref: target_ref.to_string(),
                expected_head: "a".repeat(40),
            }
            .is_valid(),
            "référence interdite acceptée : {target_ref}"
        );
    }
    assert!(
        !ReviewTarget {
            target_ref: "origin/main".to_string(),
            expected_head: "A".repeat(40),
        }
        .is_valid()
    );
}

#[test]
fn delivery_report_ordinaire_conserve_ses_octets_sans_bloc_revue() {
    let payload = ServiceRequestPayload::DeliveryReport {
        objective_id: "objective-1".to_string(),
        delegation_id: "delegation-1".to_string(),
        delivery_hash: "0".repeat(64),
        in_reply_to: "message-1".to_string(),
        review_verdict: None,
    };
    assert_eq!(
        serde_json::to_string(&payload).unwrap(),
        format!(
            "{{\"objective_id\":\"objective-1\",\"delegation_id\":\"delegation-1\",\"delivery_hash\":\"{}\",\"in_reply_to\":\"message-1\"}}",
            "0".repeat(64)
        )
    );
}

#[test]
fn delegation_historique_reste_v1_et_cible_de_revue_exige_v2() {
    let ordinary = ServiceRequestPayload::Delegate {
        goal: "relire le lot".to_string(),
        review_target: None,
        explicit_target: None,
        required_tags: Vec::new(),
        duration: GuichetDurationClass::Courte,
        suite: ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin: None,
        focus: None,
    };
    assert_eq!(
        ordinary.required_contract_version(),
        SERVICE_CONTRACT_VERSION
    );
    assert!(
        !serde_json::to_string(&ordinary)
            .unwrap()
            .contains("review_target")
    );

    let targeted = ServiceRequestPayload::Delegate {
        goal: "relire le lot".to_string(),
        review_target: Some(ReviewTarget {
            target_ref: "origin/session-047-verdict-tete-reecrite".to_string(),
            expected_head: "a".repeat(40),
        }),
        explicit_target: None,
        required_tags: Vec::new(),
        duration: GuichetDurationClass::Courte,
        suite: ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin: None,
        focus: None,
    };
    assert_eq!(
        targeted.required_contract_version(),
        REVIEW_DELEGATE_CONTRACT_VERSION
    );
    let wire = serde_json::to_string(&targeted).unwrap();
    assert!(wire.contains(r#""target_ref":"origin/session-047-verdict-tete-reecrite""#));
    assert_eq!(
        serde_json::from_str::<ServiceRequestPayload>(&wire).unwrap(),
        targeted
    );
}

#[test]
fn service_negotiation_v1_emploie_la_fixture_canonique_partagee() {
    let lines = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 5, "la fixture couvre hello, welcome et refus");

    let role: WrapperToDaemon = decode(lines[0]).unwrap();
    assert_eq!(encode(&role).unwrap(), lines[0]);
    assert!(matches!(
        role,
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Service
        }
    ));

    let accepted: DaemonToWrapper = decode(lines[1]).unwrap();
    assert_eq!(encode(&accepted).unwrap(), lines[1]);
    let hello: WrapperToDaemon = decode(lines[2]).unwrap();
    assert_eq!(encode(&hello).unwrap(), lines[2]);
    assert!(matches!(hello, WrapperToDaemon::ServiceHello { .. }));

    let welcome: DaemonToWrapper = decode(lines[3]).unwrap();
    assert_eq!(encode(&welcome).unwrap(), lines[3]);
    assert!(matches!(welcome, DaemonToWrapper::ServiceWelcome { .. }));

    let rejected: DaemonToWrapper = decode(lines[4]).unwrap();
    assert_eq!(encode(&rejected).unwrap(), lines[4]);
    assert!(matches!(
        rejected,
        DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired
        }
    ));
}

#[test]
fn coordination_stream_v2_emploie_la_fixture_canonique_fermee() {
    let lines = COORDINATION_STREAM_FIXTURE.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 10);
    for index in [0, 2, 4] {
        let frame: WrapperToDaemon = decode(lines[index]).unwrap();
        assert_eq!(encode(&frame).unwrap(), lines[index]);
    }
    for index in [1, 3, 5, 6, 7, 8, 9] {
        let frame: DaemonToWrapper = decode(lines[index]).unwrap();
        assert_eq!(encode(&frame).unwrap(), lines[index]);
    }
    assert!(matches!(
        decode::<WrapperToDaemon>(lines[4]).unwrap(),
        WrapperToDaemon::CoordinationSubscribe {
            version: COORDINATION_STREAM_VERSION,
            after_cursor: None,
        }
    ));
    assert!(matches!(
        decode::<DaemonToWrapper>(lines[7]).unwrap(),
        DaemonToWrapper::CoordinationGap {
            from_cursor: 2,
            to_cursor: 3,
            ..
        }
    ));
}

#[test]
fn coordination_events_v1_emploie_la_fixture_canonique_fermee() {
    let lines = COORDINATION_EVENTS_FIXTURE.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 6, "négociation, fait attesté et refus");

    let hello: WrapperToDaemon = decode(lines[2]).unwrap();
    assert_eq!(encode(&hello).unwrap(), lines[2]);
    assert!(matches!(
        hello,
        WrapperToDaemon::ServiceHello { capabilities, .. }
            if capabilities == vec![
                ServiceCapability::GuichetV1,
                ServiceCapability::CoordinationEventsV1,
            ]
    ));

    let fact: DaemonToWrapper = decode(lines[4]).unwrap();
    assert_eq!(encode(&fact).unwrap(), lines[4]);
    assert!(matches!(
        fact,
        DaemonToWrapper::CoordinationEvent {
            kind: CoordinationEventKind::ReminderSent,
            generation: 1,
            observed_at: 1_787_500_003,
            ..
        }
    ));

    let rejected: DaemonToWrapper = decode(lines[5]).unwrap();
    assert!(matches!(
        rejected,
        DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::InvalidEnvelope
        }
    ));
}

// T016 : JSON public compilable avant les nouveaux variants.
#[test]
fn edit159_protocol_trames_et_capacite_allers_retours() {
    let capability: ClientCapability = decode(r#""queue_edit_v1""#).unwrap();
    assert_eq!(encode(&capability).unwrap(), r#""queue_edit_v1""#);
    for action in ["cancel", "reorder", "promote"] {
        for before in [serde_json::Value::Null, serde_json::json!("avant")] {
            let wire = serde_json::json!({"type":"QueueEditRequest","request_id":"r159",
                    "target":"agent","action":action,"message_id":"message159","before_message_id":before});
            let request: WrapperToDaemon = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(request).unwrap(), wire);
        }
    }
    for wire in [
        serde_json::json!({"type":"QueueEditResult","request_id":"r159","state":"applied"}),
        serde_json::json!({"type":"QueueEditResult","request_id":"r159","state":"refused","refusal":"not_held"}),
        serde_json::json!({"type":"QueueEditResult","request_id":"r159","state":"applied","effect":"unconfirmed"}),
    ] {
        let result: DaemonToWrapper = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(result).unwrap(), wire);
    }
}

#[test]
fn edit159_protocol_trames_historiques_et_register_octets_inchanges() {
    let legacy = r#"{"type":"Register","identity_version":2,"agent_type":"codex","agent_id":"550e8400-e29b-41d4-a716-446655440000","host":"test-host","transport":"unix","channel":"unix","mode":"acp","location":null,"os":"Linux","instance_id":"instance-test","domain":"bridget","turn_in_progress":false,"journal_available":false}"#;
    let old: WrapperToDaemon = decode(legacy).unwrap();
    assert_eq!(encode(&old).unwrap(), legacy);
    let mut wire: serde_json::Value = serde_json::from_str(legacy).unwrap();
    wire["queue_fallback"] = serde_json::json!(false);
    let old: WrapperToDaemon = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(encode(&old).unwrap(), legacy);
    wire["queue_fallback"] = serde_json::json!(true);
    let new: WrapperToDaemon = serde_json::from_value(wire).unwrap();
    assert_eq!(serde_json::to_value(new).unwrap()["queue_fallback"], true);
    let list = r#"{"type":"QueueList","target":"codex-1"}"#;
    let unchanged: WrapperToDaemon = decode(list).unwrap();
    assert_eq!(encode(&unchanged).unwrap(), list);
    guidance156_protocol_trames_historiques_byte_identiques();
}

#[test]
fn held159_turnstate_register_sans_turn_attest_octets_inchanges() {
    // API JSON publique : absence/faux conservent les octets158B, vrai reste attesté.
    let legacy = r#"{"type":"Register","identity_version":2,"agent_type":"codex","agent_id":"550e8400-e29b-41d4-a716-446655440000","host":"test-host","transport":"unix","channel":"unix","mode":"acp","location":null,"os":"Linux","instance_id":"instance-test","domain":"bridget","turn_in_progress":false,"journal_available":false}"#;
    let register: WrapperToDaemon = decode(legacy).unwrap();
    assert_eq!(encode(&register).unwrap(), legacy);
    let mut value: serde_json::Value = serde_json::from_str(legacy).unwrap();
    value["turn_attest"] = serde_json::json!(false);
    let register: WrapperToDaemon = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(encode(&register).unwrap(), legacy, "faux doit être omis");
    value["turn_attest"] = serde_json::json!(true);
    let register: WrapperToDaemon = serde_json::from_value(value).unwrap();
    let encoded: serde_json::Value = serde_json::from_str(&encode(&register).unwrap()).unwrap();
    assert_eq!(
        encoded["turn_attest"],
        serde_json::json!(true),
        "déclaration vraie conservée sur le fil"
    );
}

#[test]
fn test_encode_decode_register() {
    let msg = WrapperToDaemon::Register {
        identity_version: 2,
        agent_type: "codex".to_string(),
        agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        host: Some("test-host".to_string()),
        transport: Some("unix".to_string()),
        channel: ChannelReport::Known("unix".to_string()),
        mode: Some(PresenceMode::Acp),
        location: None,
        os: Some("Linux".to_string()),
        instance_id: Some("instance-test".to_string()),
        domain: Some("bridget".to_string()),
        turn_in_progress: false,
        journal_available: Some(false),
        turn_attest: false,
        queue_fallback: false,
    };
    let json = encode(&msg).unwrap();
    assert!(json.contains("\"type\":\"Register\""));
    let decoded: WrapperToDaemon = decode(&json).unwrap();
    match decoded {
        WrapperToDaemon::Register {
            identity_version,
            agent_type,
            agent_id,
            host,
            transport,
            channel,
            mode,
            location,
            os,
            instance_id,
            domain,
            turn_in_progress,
            journal_available,
            turn_attest,
            queue_fallback,
        } => {
            assert_eq!(identity_version, 2);
            assert_eq!(agent_type, "codex");
            assert_eq!(agent_id, "550e8400-e29b-41d4-a716-446655440000");
            assert_eq!(host.as_deref(), Some("test-host"));
            assert_eq!(transport.as_deref(), Some("unix"));
            assert_eq!(channel.as_deref(), Some("unix"));
            assert_eq!(mode, Some(PresenceMode::Acp));
            assert_eq!(location, None);
            assert_eq!(os.as_deref(), Some("Linux"));
            assert_eq!(instance_id.as_deref(), Some("instance-test"));
            assert_eq!(domain.as_deref(), Some("bridget"));
            assert!(!turn_in_progress);
            assert_eq!(journal_available, Some(false));
            assert!(!turn_attest);
            assert!(!queue_fallback);
        }
        _ => panic!("mauvais type"),
    }
}

#[test]
fn register_historique_est_refuse_sans_contrat_identite_v2() {
    let json = r#"{\"type\":\"Register\",\"agent_type\":\"codex\",\"name\":null}"#;
    assert!(decode::<WrapperToDaemon>(json).is_err());
}

#[test]
fn register_v2_conserve_la_distinction_de_canal() {
    let base = "\"type\":\"Register\",\"identity_version\":2,\"agent_type\":\"ui\",\"agent_id\":\"550e8400-e29b-41d4-a716-446655440000\"";
    let omitted: WrapperToDaemon = decode(&format!("{{{base}}}")).unwrap();
    assert!(matches!(
        omitted,
        WrapperToDaemon::Register {
            channel: ChannelReport::Omitted,
            ..
        }
    ));
    let unknown: WrapperToDaemon = decode(&format!("{{{base},\"channel\":null}}")).unwrap();
    assert!(matches!(
        unknown,
        WrapperToDaemon::Register {
            channel: ChannelReport::Unknown,
            ..
        }
    ));
    let known: WrapperToDaemon = decode(&format!("{{{base},\"channel\":\"ssh-unix\"}}")).unwrap();
    assert!(
        matches!(known, WrapperToDaemon::Register { channel: ChannelReport::Known(value), .. } if value == "ssh-unix")
    );
}

// T003 : oracles JSON compilables AVANT les nouveaux variants/champs.
// Les octets155 sont des constantes indépendantes de l'implémentation156.
#[test]
fn guidance156_protocol_trames_historiques_byte_identiques() {
    for wire in [
        r#"{"type":"Deliver","id":"m","from":"a","to":"b","body":"x","reply":false,"hops":4,"from_declared":false}"#,
        r#"{"type":"DeliverIdempotent","delivery_id":"d","recipient_instance_id":"i","delivery_generation":1,"expires_at":1900000000,"message":{"id":"m","from":"a","to":"b","body":"x","reply":false,"hops":4,"from_declared":false}}"#,
        r#"{"type":"CancelDelivery","id":"m","reason":"cancelled"}"#,
    ] {
        let frame: DaemonToWrapper = decode(wire).unwrap();
        assert_eq!(
            encode(&frame).unwrap(),
            wire,
            "omission des flags156 : mêmes octets155"
        );
    }
}

#[test]
fn guidance156_protocol_queue_et_refus_allers_retours() {
    let capability: ClientCapability =
        decode(r#""queue_list_v1""#).expect("capacité156 fermée connue");
    assert_eq!(encode(&capability).unwrap(), r#""queue_list_v1""#);
    let refusal: ExecutionControlRefusal =
        decode(r#""not_permitted""#).expect("refus156 fermé connu");
    assert_eq!(encode(&refusal).unwrap(), r#""not_permitted""#);
    let wire = r#"{"type":"QueueList","target":"codex-1"}"#;
    let request: WrapperToDaemon = decode(wire).expect("lecture de file156");
    assert_eq!(encode(&request).unwrap(), wire);
    for queue_bound in [serde_json::json!(64), serde_json::Value::Null] {
        let wire = serde_json::json!({"type":"QueueListResult","entries":[{
                "message_id":"m","from":"a","intent":"steer","reply":false,"age_secs":12,
                "preview":"travail cité","position":1,"state":"pending_or_injecting"
            }],"queue_bound":queue_bound});
        let response: DaemonToWrapper = decode(&wire.to_string()).expect("résultat de file156");
        assert_eq!(
            serde_json::to_value(&response).unwrap(),
            wire,
            "queue_bound null reste explicite"
        );
        assert!(!response.allowed_for_attach());
    }
}

#[test]
fn guidance156_protocol_flags_allers_retours() {
    for allowed in [false, true] {
        for wire in [
            serde_json::json!({"type":"Deliver","id":"m","from":"a","to":"b","body":"x",
                    "reply":false,"hops":4,"from_declared":false,"mid_turn_ok":allowed}),
            serde_json::json!({"type":"DeliverIdempotent","delivery_id":"d","recipient_instance_id":"i",
                    "delivery_generation":1,"expires_at":1900000000,
                    "message":{"id":"m","from":"a","to":"b","body":"x","reply":false,"hops":4,"from_declared":false},
                    "mid_turn_ok":allowed}),
            serde_json::json!({"type":"CancelDelivery","id":"m","reason":"cancelled","interrupt_active":allowed}),
        ] {
            let frame: DaemonToWrapper = decode(&wire.to_string()).unwrap();
            assert_eq!(
                serde_json::to_value(frame).unwrap(),
                wire,
                "un flag156 explicite doit survivre au décodage"
            );
        }
    }
}

#[test]
fn test_encode_decode_deliver() {
    let msg = BridgetMessage::new("claude-1", "codex-1", "hello");
    let dtw = DaemonToWrapper::Deliver {
        message: msg.clone(),
        mid_turn_ok: None,
    };
    let json = encode(&dtw).unwrap();
    assert!(json.contains("\"type\":\"Deliver\""));
    let decoded: DaemonToWrapper = decode(&json).unwrap();
    match decoded {
        DaemonToWrapper::Deliver { message: m, .. } => {
            assert_eq!(m.from, "claude-1");
            assert_eq!(m.body, "hello");
        }
        _ => panic!("mauvais type"),
    }
}

#[test]
fn spec_079_deliver_idempotent_projette_le_contexte_execution_optionnel() {
    let message = BridgetMessage::new("humain", "codex-1", "reprends ce travail");
    let frame = DaemonToWrapper::DeliverIdempotent {
        delivery_id: "delivery-079".to_string(),
        recipient_instance_id: "instance-079".to_string(),
        delivery_generation: 7,
        expires_at: 1_900_000_000,
        message,
        mid_turn_ok: None,
        execution: Some(ExecutionDeliveryContext {
            execution_id: "execution-079".to_string(),
            generation: 2,
            revision: 0,
        }),
    };

    let encoded = encode(&frame).unwrap();
    let decoded: DaemonToWrapper = decode(&encoded).unwrap();
    assert!(matches!(
        decoded,
        DaemonToWrapper::DeliverIdempotent {
            execution: Some(ExecutionDeliveryContext {
                execution_id,
                generation: 2,
                revision: 0,
            }),
            ..
        } if execution_id == "execution-079"
    ));
}

#[test]
fn spec_079_deliver_idempotent_historique_sans_contexte_reste_decodable() {
    let json = r#"{"type":"DeliverIdempotent","delivery_id":"delivery-old","recipient_instance_id":"instance-old","delivery_generation":1,"expires_at":1900000000,"message":{"id":"message-old","from":"humain","to":"codex-1","body":"historique","reply":false,"references":[],"timestamp":"2026-08-31T00:00:00Z"}}"#;

    let decoded: DaemonToWrapper = decode(json).unwrap();
    assert!(matches!(
        decoded,
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            execution: None,
            ..
        } if delivery_id == "delivery-old"
    ));
}

#[test]
fn test_encode_decode_send() {
    let msg = BridgetMessage::new("codex-1", "claude-1", "réponse");
    let wtd = WrapperToDaemon::Send(msg);
    let json = encode(&wtd).unwrap();
    let decoded: WrapperToDaemon = decode(&json).unwrap();
    match decoded {
        WrapperToDaemon::Send(m) => assert_eq!(m.body, "réponse"),
        _ => panic!("mauvais type"),
    }
}

#[test]
fn native160_protocole_refus_sans_correlation_octets_inchanges() {
    let wire = br#"{"type":"DeliveryRejected","id":"message160","reason":"limite"}"#;
    let decoded: WrapperToDaemon = serde_json::from_slice(wire).unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), wire);
    assert!(matches!(
        decoded,
        WrapperToDaemon::DeliveryRejected {
            execution_id: None,
            generation: None,
            ..
        }
    ));
}

#[test]
fn native160_protocole_refus_correlation_et_ancien_decodeur() {
    let wire = br#"{"type":"DeliveryRejected","id":"message160","reason":"limite","execution_id":"execution160","generation":2}"#;
    let decoded: WrapperToDaemon = serde_json::from_slice(wire).unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), wire);
    assert!(matches!(decoded, WrapperToDaemon::DeliveryRejected {
            execution_id: Some(ref id), generation: Some(2), ..
        } if id == "execution160"));
    // Projection de cette variante historique : même tag, sans deny_unknown_fields.
    // Aucun ancien binaire daemon n'est exécuté par ce test.
    #[derive(serde::Deserialize)]
    #[serde(tag = "type")]
    enum LegacyWrapper {
        DeliveryRejected { id: String, reason: String },
    }
    let old: LegacyWrapper = serde_json::from_slice(wire).unwrap();
    assert!(matches!(old, LegacyWrapper::DeliveryRejected { id, reason }
            if id == "message160" && reason == "limite"));
}

#[test]
fn test_encode_decode_acp_delivery_lifecycle() {
    let cancel = DaemonToWrapper::CancelDelivery {
        id: "message-1".to_string(),
        reason: "échéance".to_string(),
        interrupt_active: None,
    };
    assert!(matches!(
        decode(&encode(&cancel).unwrap()).unwrap(),
        DaemonToWrapper::CancelDelivery { id, reason, .. } if id == "message-1" && reason == "échéance"
    ));
    let rejected = WrapperToDaemon::DeliveryRejected {
        id: "message-1".to_string(),
        reason: "file ACP pleine".to_string(),
        execution_id: None,
        generation: None,
    };
    assert!(matches!(
        decode(&encode(&rejected).unwrap()).unwrap(),
        WrapperToDaemon::DeliveryRejected { id, reason, .. } if id == "message-1" && reason == "file ACP pleine"
    ));
    assert!(matches!(
        decode(&encode(&WrapperToDaemon::TurnState { in_progress: true }).unwrap()).unwrap(),
        WrapperToDaemon::TurnState { in_progress: true }
    ));
}

#[test]
fn test_encode_decode_runtime() {
    let msg = WrapperToDaemon::Runtime {
        agent: "agent-2".to_string(),
        model: "claude-opus-5".to_string(),
        effort: Some("high".to_string()),
        source: RuntimeSource::ClaudeHook,
    };
    let json = encode(&msg).unwrap();
    assert!(json.contains("\"type\":\"Runtime\""));
    assert!(json.contains("\"source\":\"claude-hook\""));
    match decode(&json).unwrap() {
        WrapperToDaemon::Runtime {
            agent,
            model,
            effort,
            source,
        } => {
            assert_eq!(agent, "agent-2");
            assert_eq!(model, "claude-opus-5");
            assert_eq!(effort.as_deref(), Some("high"));
            assert_eq!(source, RuntimeSource::ClaudeHook);
        }
        other => panic!("mauvais type: {:?}", other),
    }
}

#[test]
fn runtime_source_claude_transcript_est_stable() {
    let encoded = encode(&RuntimeSource::ClaudeTranscript).unwrap();
    assert_eq!(encoded, "\"claude-transcript\"");
    assert_eq!(
        decode::<RuntimeSource>(&encoded).unwrap(),
        RuntimeSource::ClaudeTranscript
    );
    assert_eq!(
        RuntimeSource::ClaudeTranscript.to_string(),
        "claude-transcript"
    );
}

#[test]
fn runtime_et_limite_codex_app_server_sont_fermes() {
    let runtime = encode(&RuntimeSource::CodexAppServer).unwrap();
    assert_eq!(runtime, "\"codex-app-server\"");
    assert_eq!(
        decode::<RuntimeSource>(&runtime).unwrap(),
        RuntimeSource::CodexAppServer
    );
    let limit = encode(&RateLimitSource::CodexAppServer).unwrap();
    assert_eq!(limit, "\"codex-app-server\"");
    assert_eq!(
        decode::<RateLimitSource>(&limit).unwrap(),
        RateLimitSource::CodexAppServer
    );
    assert!(decode::<RuntimeSource>("\"codex-app-server-futur\"").is_err());
    assert!(decode::<RateLimitSource>("\"codex-app-server-futur\"").is_err());
}

#[test]
fn test_runtime_effort_absent_est_decodable() {
    // Cas Haiku : le modèle n'expose aucun niveau d'effort.
    let json = r#"{"type":"Runtime","agent":"agent-2","model":"claude-haiku-4-5","source":"codex-rollout"}"#;
    match decode(json).unwrap() {
        WrapperToDaemon::Runtime { effort, .. } => assert!(effort.is_none()),
        other => panic!("mauvais type: {:?}", other),
    }
}

#[test]
fn test_runtime_source_inconnue_est_refusee() {
    let json = r#"{"type":"Runtime","agent":"agent-2","model":"x","source":"inventee"}"#;
    assert!(decode::<WrapperToDaemon>(json).is_err());
}

#[test]
fn test_encode_decode_rate_limit_et_absence_de_retour() {
    let message = WrapperToDaemon::RateLimit {
        credits: None,
        agent: "claude-1".to_string(),
        window: "five_hour".to_string(),
        status: "rejected".to_string(),
        resets_at: Some(1_787_572_200),
        used_percent: Some(100),
        source: RateLimitSource::ClaudeStreamJson,
    };
    let encoded = encode(&message).unwrap();
    assert!(encoded.contains("\"type\":\"RateLimit\""));
    assert!(encoded.contains("\"source\":\"claude-stream-json\""));
    assert!(encoded.contains("\"used_percent\":100"));
    assert!(
        !encoded.contains("\"credits\""),
        "les messages legacy gardent leurs octets"
    );
    let credited = WrapperToDaemon::RateLimit {
        agent: "codex".into(),
        window: "weekly".into(),
        status: "rate_limit_reached".into(),
        resets_at: Some(500),
        used_percent: Some(100),
        source: RateLimitSource::CodexAppServer,
        credits: Some(CodexCredits {
            has_credits: Some(true),
            unlimited: Some(false),
            spend_control_reached: false,
        }),
    };
    assert!(
        matches!(decode(&encode(&credited).unwrap()).unwrap(), WrapperToDaemon::RateLimit { credits:Some(credits),.. } if credits.usable())
    );
    assert!(matches!(
        decode(&encoded).unwrap(),
        WrapperToDaemon::RateLimit {
            credits: None,
            agent,
            window,
            status,
            resets_at: Some(1_787_572_200),
            used_percent: Some(100),
            source: RateLimitSource::ClaudeStreamJson,
        } if agent == "claude-1" && window == "five_hour" && status == "rejected"
    ));

    let without_reset = r#"{"type":"RateLimit","agent":"claude-1","window":"five_hour","status":"allowed","source":"claude-stream-json"}"#;
    assert!(matches!(
        decode(without_reset).unwrap(),
        WrapperToDaemon::RateLimit {
            resets_at: None,
            used_percent: None,
            ..
        }
    ));
}

#[test]
fn test_encode_decode_usage_et_fenetre_inconnue() {
    let sample = WrapperToDaemon::Usage {
        agent: "claude-1".to_string(),
        input_tokens: 2,
        execution_id: Some("execution-1".to_string()),
        execution_generation: Some(1),
        output_tokens: 175,
        cache_creation_input_tokens: 40_804,
        cache_read_input_tokens: 13_907,
        provider_kind: Some("claude".to_string()),
        source: UsageSource::ClaudeStreamJson,
    };
    let encoded = encode(&sample).unwrap();
    assert!(encoded.contains("\"type\":\"Usage\""));
    assert!(encoded.contains("\"source\":\"claude-stream-json\""));
    assert!(matches!(
        decode(&encoded).unwrap(),
        WrapperToDaemon::Usage {
            input_tokens: 2,
            execution_id: Some(execution_id),
            execution_generation: Some(1),
            output_tokens: 175,
            cache_creation_input_tokens: 40_804,
            cache_read_input_tokens: 13_907,
            source: UsageSource::ClaudeStreamJson,
            ..
        } if execution_id == "execution-1"
    ));

    let window = WrapperToDaemon::UsageWindow {
        agent: "claude-1".to_string(),
        from_secs: 10,
        to_secs: 20,
    };
    let encoded_window = encode(&window).unwrap();
    assert!(encoded_window.contains("\"type\":\"UsageWindow\""));

    let unknown = r#"{"type":"UsageWindowResult","agent":"tmux-1"}"#;
    assert!(matches!(
        decode(unknown).unwrap(),
        DaemonToWrapper::UsageWindowResult {
            aggregate: None,
            ..
        }
    ));

    let attested = DaemonToWrapper::UsageWindowResult {
        agent: "claude-1".to_string(),
        aggregate: Some(UsageAggregate {
            turns: 1,
            input_tokens: 2,
            output_tokens: 175,
            cache_creation_input_tokens: 40_804,
            cache_read_input_tokens: 13_907,
            facturable_tokens: 40_981,
        }),
    };
    let encoded_attested = encode(&attested).unwrap();
    assert!(encoded_attested.contains("\"facturable_tokens\":40981"));
    assert!(!encoded_attested.contains("\"facturable_tokens\":0"));
}

#[test]
fn test_agent_info_sans_runtime_reste_decodable() {
    // Compatibilité ascendante : un daemon d'une version antérieure ne
    // sérialise ni model ni effort.
    let json = r#"{"agent_id":"550e8400-e29b-41d4-a716-446655440002","display_name":"Agent 2","agent_type":"claude","connection_id":"conn-1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0}"#;
    let info: AgentInfo = decode(json).unwrap();
    assert!(info.model.is_none());
    assert!(info.effort.is_none());
    assert!(info.channel.is_none());
    assert!(info.rate_limits.is_empty());
    assert!(info.model_mismatch.is_none());
    assert!(info.disk_space.is_none());
}

#[test]
fn fait_espace_disque_post_enregistrement_garde_sa_valeur_attestee() {
    let message = WrapperToDaemon::DiskSpace {
        fact: DiskSpaceFact {
            volume: "/".to_string(),
            free_bytes: 47_300_000_000,
            observed_at_unix: 1_788_000_000,
        },
    };
    let encoded = encode(&message).unwrap();
    assert!(matches!(
        decode::<WrapperToDaemon>(&encoded).unwrap(),
        WrapperToDaemon::DiskSpace { fact }
            if fact.volume == "/"
                && fact.free_bytes == 47_300_000_000
                && fact.observed_at_unix == 1_788_000_000
    ));
}

#[test]
fn spec_024_agent_info_expose_protocole_et_canal_independants() {
    let json = r#"{"agent_id":"550e8400-e29b-41d4-a716-446655440003","display_name":"Lab Agent","agent_type":"codex","connection_id":"conn-1",
            "host":"lab-host","transport":"tmux","channel":"ssh-unix","mode":"tmux",
            "os":"Linux","state":"connected","last_seen_secs":0,"reconnect_count":0}"#;
    let info: AgentInfo = decode(json).unwrap();
    assert_eq!(info.transport, "tmux");
    assert_eq!(info.channel.as_deref(), Some("ssh-unix"));

    let encoded = encode(&info).unwrap();
    assert!(encoded.contains(r#""transport":"tmux""#));
    assert!(encoded.contains(r#""channel":"ssh-unix""#));
}

#[test]
fn fait_mono_fenetre_ancien_se_relit_en_vec_sans_inventer() {
    // Ancien fil : un seul champ `rate_limit`. Doit devenir Vec d'1 élément
    // avec la fenêtre attestée telle quelle — aucune fenêtre inventée.
    let json = r#"{
            "agent_id":"550e8400-e29b-41d4-a716-446655440004","display_name":"Claude","agent_type":"claude","connection_id":"c1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0,
            "rate_limit":{"window":"five_hour","status":"rejected","resets_at":1787572200}
        }"#;
    let info: AgentInfo = decode(json).unwrap();
    assert_eq!(info.rate_limits.len(), 1);
    assert_eq!(info.rate_limits[0].window, "five_hour");
    assert_eq!(info.rate_limits[0].status, "rejected");
    assert_eq!(info.rate_limits[0].resets_at, Some(1_787_572_200));
    assert_eq!(info.rate_limits[0].used_percent, None);

    // Nouveau fil : `rate_limits` gagne ; l'ancien champ s'il coexiste est ignoré.
    let both = r#"{
            "agent_id":"550e8400-e29b-41d4-a716-446655440004","display_name":"Claude","agent_type":"claude","connection_id":"c1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0,
            "rate_limits":[{"window":"seven_day","status":"allowed","used_percent":61}],
            "rate_limit":{"window":"five_hour","status":"rejected"}
        }"#;
    let prefer_new: AgentInfo = decode(both).unwrap();
    assert_eq!(prefer_new.rate_limits.len(), 1);
    assert_eq!(prefer_new.rate_limits[0].window, "seven_day");

    // Sérialisation neuve : uniquement `rate_limits`, jamais `rate_limit`.
    let encoded = encode(&prefer_new).unwrap();
    assert!(encoded.contains("\"rate_limits\""));
    assert!(!encoded.contains("\"rate_limit\":"));
}

#[test]
fn test_encode_decode_served_model() {
    let message = WrapperToDaemon::ServedModel {
        agent: "claude-1".to_string(),
        model: "claude-opus-4-6".to_string(),
    };
    let encoded = encode(&message).unwrap();
    assert!(encoded.contains("\"type\":\"ServedModel\""));
    assert!(matches!(
        decode(&encoded).unwrap(),
        WrapperToDaemon::ServedModel { agent, model }
            if agent == "claude-1" && model == "claude-opus-4-6"
    ));
}

#[test]
fn model_mismatch_observe_exige_les_deux_etiquettes_distinctes() {
    assert!(ModelMismatchFact::observe(None, Some("claude-opus-4-6")).is_none());
    assert!(ModelMismatchFact::observe(Some("claude-opus-5"), None).is_none());
    assert!(ModelMismatchFact::observe(Some("claude-opus-5"), Some("claude-opus-5")).is_none());
    let gap = ModelMismatchFact::observe(Some("claude-opus-5"), Some("claude-opus-4-6"))
        .expect("écart attesté");
    assert_eq!(gap.pinned, "claude-opus-5");
    assert_eq!(gap.served, "claude-opus-4-6");
}

#[test]
fn test_encode_decode_nack() {
    let dtw = DaemonToWrapper::Nack {
        id: "abc123".to_string(),
        reason: "agent introuvable".to_string(),
    };
    let json = encode(&dtw).unwrap();
    let decoded: DaemonToWrapper = decode(&json).unwrap();
    match decoded {
        DaemonToWrapper::Nack { id, reason } => {
            assert_eq!(id, "abc123");
            assert_eq!(reason, "agent introuvable");
        }
        _ => panic!("mauvais type"),
    }
}

#[test]
fn attach_client_messages_roundtrip() {
    let handshake = WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Attach,
    };
    assert!(matches!(
        decode(&encode(&handshake).unwrap()).unwrap(),
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach
        }
    ));

    let subscribe = WrapperToDaemon::Subscribe {
        agent: "codex-1".to_string(),
        window: AttachWindow::Seq(42),
    };
    assert!(matches!(
        decode(&encode(&subscribe).unwrap()).unwrap(),
        WrapperToDaemon::Subscribe {
            agent,
            window: AttachWindow::Seq(42)
        } if agent == "codex-1"
    ));

    let unsubscribe = WrapperToDaemon::Unsubscribe {
        subscription_id: "sub-1".to_string(),
    };
    assert!(matches!(
        decode(&encode(&unsubscribe).unwrap()).unwrap(),
        WrapperToDaemon::Unsubscribe { subscription_id } if subscription_id == "sub-1"
    ));
}

#[test]
fn service_guichet_messages_roundtrip_et_restent_hors_attach() {
    let hello = WrapperToDaemon::ServiceHello {
        version: SERVICE_CONTRACT_VERSION,
        service: "guichet".to_string(),
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        capabilities: vec![ServiceCapability::GuichetV1],
    };
    assert_eq!(
        encode(&hello).unwrap(),
        SERVICE_NEGOTIATION_FIXTURE.lines().nth(2).unwrap()
    );
    assert!(matches!(
        decode(&encode(&hello).unwrap()).unwrap(),
        WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            capabilities,
            ..
        } if capabilities == vec![ServiceCapability::GuichetV1]
    ));

    let reply = WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "req-1".to_string(),
        claim_generation: 3,
        claim_token: "claim-1".to_string(),
        response_message_id: "msg-1".to_string(),
        in_reply_to: "message-1".to_string(),
        outcome: GuichetOutcome::Accepted,
        payload: GuichetReplyPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "0".repeat(64),
            review_verdict: None,
        },
    };
    assert!(matches!(
        decode(&encode(&reply).unwrap()).unwrap(),
        WrapperToDaemon::GuichetReply {
            claim_generation: 3,
            claim_token,
            ..
        } if claim_token == "claim-1"
    ));
    assert_eq!(
        reply.attach_refusal(),
        Some(AttachRefusal::MessageOutsideAttachRole)
    );

    let waiting = WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "req-waiting".to_string(),
        claim_generation: 3,
        claim_token: "claim-waiting".to_string(),
        response_message_id: "msg-waiting".to_string(),
        in_reply_to: "message-waiting".to_string(),
        outcome: GuichetOutcome::Accepted,
        payload: GuichetReplyPayload::Delegate {
            status: GuichetDelegateMutationStatus::WaitingForAgent,
            objective_id: Some("objective-waiting".to_string()),
            delegation_id: None,
            message_id: None,
            participant: None,
            candidates: Vec::new(),
            waiting_on_prerequisites: true,
            replayed: false,
        },
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&waiting).unwrap()).unwrap(),
        WrapperToDaemon::GuichetReply {
            payload: GuichetReplyPayload::Delegate {
                status: GuichetDelegateMutationStatus::WaitingForAgent,
                objective_id: Some(objective_id),
                delegation_id: None,
                waiting_on_prerequisites: true,
                ..
            },
            ..
        } if objective_id == "objective-waiting"
    ));

    let refused = WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "req-refused".to_string(),
        claim_generation: 1,
        claim_token: "claim-refused".to_string(),
        response_message_id: "msg-refused".to_string(),
        in_reply_to: "message-refused".to_string(),
        outcome: GuichetOutcome::Refused,
        payload: GuichetReplyPayload::Refused {
            operation: ServiceRequestOperation::MissionStatus,
            reason: GuichetRefusalReason::DelegationMissing,
        },
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&refused).unwrap()).unwrap(),
        WrapperToDaemon::GuichetReply {
            outcome: GuichetOutcome::Refused,
            payload: GuichetReplyPayload::Refused {
                operation: ServiceRequestOperation::MissionStatus,
                reason: GuichetRefusalReason::DelegationMissing,
            },
            ..
        }
    ));

    let status = WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "req-status".to_string(),
        claim_generation: 4,
        claim_token: "claim-2".to_string(),
        response_message_id: "message-2".to_string(),
        in_reply_to: "request-status".to_string(),
        outcome: GuichetOutcome::Accepted,
        payload: GuichetReplyPayload::MissionStatus {
            delegation_id: "delegation-1".to_string(),
            objective_id: "objective-1".to_string(),
            coordination_state: GuichetCoordinationState::EnCoordination,
            local_delivery: GuichetLocalDelivery {
                state: GuichetLocalDeliveryState::Accepted,
                issue: Some("accepted".to_string()),
                observed_at: Some(1_787_500_001),
            },
            transport_observation: Some(GuichetTransportObservation {
                state: GuichetTransportState::Connected,
                request_state: Some("answered".to_string()),
                observed_at: 1_787_500_002,
                source: "attach".to_string(),
                subscription_id: Some("subscription-1".to_string()),
                seq: Some(7),
            }),
            freshness: GuichetFreshness::Fresh,
        },
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&status).unwrap()).unwrap(),
        WrapperToDaemon::GuichetReply {
            payload: GuichetReplyPayload::MissionStatus {
                freshness: GuichetFreshness::Fresh,
                transport_observation: Some(GuichetTransportObservation { seq: Some(7), .. }),
                ..
            },
            ..
        }
    ));

    let rejected = DaemonToWrapper::ServiceRejected {
        reason: ServiceRefusal::CapabilityRequired,
    };
    assert!(matches!(
        decode(&encode(&rejected).unwrap()).unwrap(),
        DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired
        }
    ));
    assert!(!rejected.allowed_for_attach());

    let lifecycle = DaemonToWrapper::RequestLifecycleEvent {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        event_id: "evt-1".to_string(),
        request_id: "request-1".to_string(),
        state: GuichetLifecycleState::Answered,
        observed_at: 1_787_500_000,
        in_reply_to: Some("message-1".to_string()),
        response_message_id: Some("response-1".to_string()),
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(&encode(&lifecycle).unwrap()).unwrap(),
        DaemonToWrapper::RequestLifecycleEvent {
            state: GuichetLifecycleState::Answered,
            in_reply_to: Some(in_reply_to),
            response_message_id: Some(response_message_id),
            ..
        } if in_reply_to == "message-1" && response_message_id == "response-1"
    ));
    assert!(!lifecycle.allowed_for_attach());

    let coordination = DaemonToWrapper::CoordinationEvent {
        version: COORDINATION_EVENTS_VERSION,
        event_id: "evt-reminder-1".to_string(),
        request_id: "request-1".to_string(),
        kind: CoordinationEventKind::ReminderSent,
        reminder_message_id: "message-reminder-1".to_string(),
        recipient: "codex-1".to_string(),
        generation: 1,
        observed_at: 1_787_500_003,
        cursor: None,
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(&encode(&coordination).unwrap()).unwrap(),
        DaemonToWrapper::CoordinationEvent {
            version: COORDINATION_EVENTS_VERSION,
            kind: CoordinationEventKind::ReminderSent,
            generation: 1,
            observed_at: 1_787_500_003,
            ..
        }
    ));
    assert!(!coordination.allowed_for_attach());

    let hello_with_coordination = WrapperToDaemon::ServiceHello {
        version: SERVICE_CONTRACT_VERSION,
        service: "guichet".to_string(),
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        capabilities: vec![
            ServiceCapability::GuichetV1,
            ServiceCapability::CoordinationEventsV1,
        ],
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&hello_with_coordination).unwrap()).unwrap(),
        WrapperToDaemon::ServiceHello { capabilities, .. }
            if capabilities == vec![
                ServiceCapability::GuichetV1,
                ServiceCapability::CoordinationEventsV1,
            ]
    ));
    assert!(decode::<DaemonToWrapper>(
            r#"{"type":"coordination_event","v":1,"event_id":"evt","request_id":"req","kind":"unknown_fact","reminder_message_id":"msg","recipient":"codex","generation":1,"observed_at":1}"#
        )
        .is_err());
}

#[test]
fn mutations_du_greffe_et_resultat_terminal_font_un_roundtrip_ferme() {
    let mutations = [
        (
            ServiceRequestOperation::RegistreAdd,
            ServiceRequestPayload::RegistreAdd {
                line: "kind=add id=constat-1".to_string(),
            },
            "registre_add",
        ),
        (
            ServiceRequestOperation::ObjectiveClose,
            ServiceRequestPayload::ObjectiveClose {
                objective_id: "objective-1".to_string(),
                reason: "objectif atteint".to_string(),
            },
            "objective_close",
        ),
    ];
    for (operation, payload, wire_name) in mutations {
        let request = WrapperToDaemon::ServiceRequest {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "026_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: format!("request-{wire_name}"),
            issued_at: 1_787_824_000,
            from: "jc2".to_string(),
            to: "guichet".to_string(),
            operation,
            payload: payload.clone(),
        };
        let wire = encode(&request).unwrap();
        assert!(wire.contains(&format!(r#""operation":"{wire_name}""#)));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ServiceRequest {
                operation: decoded_operation,
                payload: decoded_payload,
                ..
            } if decoded_operation == operation && decoded_payload == payload
        ));
    }

    let result = DaemonToWrapper::GuichetResult {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "026_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "request-close".to_string(),
        issue: "accepted".to_string(),
        expires_at: 1_787_824_060,
        payload: Some(GuichetReplyPayload::ObjectiveClose {
            objective_id: "objective-1".to_string(),
            decision_id: "decision-1".to_string(),
            replayed: false,
        }),
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(&encode(&result).unwrap()).unwrap(),
        DaemonToWrapper::GuichetResult {
            issue,
            payload: Some(GuichetReplyPayload::ObjectiveClose {
                objective_id,
                decision_id,
                replayed: false,
            }),
            ..
        } if issue == "accepted"
            && objective_id == "objective-1"
            && decision_id == "decision-1"
    ));

    let historical: DaemonToWrapper = decode(
            r#"{"type":"guichet_result","v":1,"issuer_scope":"026_scope_0123456789abcdef0123456789abcdef","request_id":"request-old","issue":"queued","expires_at":1787824060}"#,
        )
        .unwrap();
    assert!(matches!(
        historical,
        DaemonToWrapper::GuichetResult { payload: None, .. }
    ));
    assert!(matches!(
        decode::<DaemonToWrapper>(
            r#"{"type":"ServiceRejected","reason":{"kind":"greffe_authorization_denied"}}"#
        )
        .unwrap(),
        DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::GreffeAuthorizationDenied
        }
    ));
}

#[test]
fn motif_compose_de_revue_fait_un_roundtrip_filaire_exact() {
    let reply = WrapperToDaemon::GuichetReply {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "req-refused-compound".to_string(),
        claim_generation: 2,
        claim_token: "claim-refused-compound".to_string(),
        response_message_id: "msg-refused-compound".to_string(),
        in_reply_to: "message-refused-compound".to_string(),
        outcome: GuichetOutcome::Refused,
        payload: GuichetReplyPayload::Refused {
            operation: ServiceRequestOperation::DeliveryReport,
            reason: GuichetRefusalReason::TargetHeadMovedAndMeasuredHeadMismatch,
        },
    };

    let wire = encode(&reply).unwrap();
    assert_eq!(
        wire,
        r#"{"type":"guichet_reply","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-refused-compound","claim_generation":2,"claim_token":"claim-refused-compound","response_message_id":"msg-refused-compound","in_reply_to":"message-refused-compound","outcome":"refused","payload":{"kind":"refused","operation":"delivery_report","reason":"target_head_moved_and_measured_head_mismatch"}}"#
    );
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::GuichetReply {
            outcome: GuichetOutcome::Refused,
            payload: GuichetReplyPayload::Refused {
                operation: ServiceRequestOperation::DeliveryReport,
                reason: GuichetRefusalReason::TargetHeadMovedAndMeasuredHeadMismatch,
            },
            ..
        }
    ));
}

#[test]
fn client_idempotency_messages_roundtrip_and_stay_outside_attach() {
    let hello = WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
        capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
    };
    assert!(matches!(
        decode(&encode(&hello).unwrap()).unwrap(),
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope,
            capabilities,
        } if issuer_scope == "012_scope_aaaaaaaaaaaa"
            && capabilities == vec![ClientCapability::SendIdempotent, ClientCapability::Lookup]
    ));
    assert_eq!(
        hello.attach_refusal(),
        Some(AttachRefusal::MessageOutsideAttachRole)
    );
    let welcome = DaemonToWrapper::ClientWelcome {
        version: CLIENT_CONTRACT_VERSION,
        build_id: "fixture-build".to_string(),
        horizon_secs: 60,
        issued_at_tolerance_secs: 5,
        capabilities: vec![ClientCapability::Lookup],
    };
    let decoded: DaemonToWrapper = decode(&encode(&welcome).unwrap()).unwrap();
    assert!(matches!(
        decoded,
        DaemonToWrapper::ClientWelcome {
            version: CLIENT_CONTRACT_VERSION,
            build_id,
            capabilities,
            ..
        } if build_id == "fixture-build" && capabilities == vec![ClientCapability::Lookup]
    ));
    let result = DaemonToWrapper::IdempotencyResult {
        project_warnings: Vec::new(),
        operation_kind: "send".to_string(),
        idempotency_key: "message-1".to_string(),
        issue: IdempotencyIssue::OutcomeUnknown {
            expires_at: 123,
            delivery_id: Some("delivery-1".to_string()),
        },
    };
    assert!(matches!(
        decode(&encode(&result).unwrap()).unwrap(),
        DaemonToWrapper::IdempotencyResult {
            operation_kind,
            idempotency_key,
            issue: IdempotencyIssue::OutcomeUnknown {
                expires_at: 123,
                delivery_id: Some(delivery_id),
            },
            ..
        } if operation_kind == "send" && idempotency_key == "message-1" && delivery_id == "delivery-1"
    ));
    assert!(!welcome.allowed_for_attach());
}

/// La machine, la base et l'instance du daemon voyagent par un message
/// DÉDIÉ. Les trois valeurs doivent survivre au tour du fil : l'instance
/// reste distincte du build et du chemin local.
///
/// Mutant qui tue ce test : renvoyer `db_path` à la place de
/// `instance_id` → la dernière assertion meurt.
#[test]
fn le_daemon_atteste_sa_machine_sa_base_et_son_instance() {
    let report = DaemonToWrapper::DaemonIdentityReport {
        host: "monordinateur".to_string(),
        db_path: "/Users/moi/.cache/bridget/bridget.db".to_string(),
        instance_id: "daemon-7f0c01d2".to_string(),
        features: Vec::new(),
    };
    match decode::<DaemonToWrapper>(&encode(&report).unwrap()).unwrap() {
        DaemonToWrapper::DaemonIdentityReport {
            host,
            db_path,
            instance_id,
            ..
        } => {
            assert_eq!(host, "monordinateur");
            assert_eq!(db_path, "/Users/moi/.cache/bridget/bridget.db");
            assert_eq!(instance_id, "daemon-7f0c01d2");
        }
        other => panic!("variante inattendue: {other:?}"),
    }
}

/// Les deux hôtes du refus survivent au tour du fil ET restent distincts :
/// un oracle qui vérifierait seulement leur présence passerait aussi si le
/// producteur écrivait deux fois la même machine.
///
/// Mutant qui tue ce test : sérialiser `requested_from` à partir de
/// `searched_on` → l'assertion d'inégalité meurt.
#[test]
fn cwd_gone_transporte_les_deux_machines_et_les_distingue() {
    let refusal = SpawnRefusal::CwdGone {
        searched_on: "monordinateur".to_string(),
        requested_from: "cartae".to_string(),
    };
    let json = serde_json::to_string(&refusal).expect("sérialisable");
    let decoded: SpawnRefusal = serde_json::from_str(&json).expect("décodable");
    match decoded {
        SpawnRefusal::CwdGone {
            searched_on,
            requested_from,
        } => {
            assert_eq!(searched_on, "monordinateur");
            assert_eq!(requested_from, "cartae");
            assert_ne!(searched_on, requested_from);
        }
        other => panic!("variante inattendue: {other:?}"),
    }

    // Refus produit par un daemon antérieur : champs vides, jamais devinés.
    let ancien: SpawnRefusal =
        serde_json::from_str(r#"{"kind":"cwd_gone"}"#).expect("refus historique décodable");
    assert!(matches!(
        ancien,
        SpawnRefusal::CwdGone {
            searched_on,
            requested_from,
        } if searched_on.is_empty() && requested_from.is_empty()
    ));
}

#[test]
fn client_welcome_historique_signale_un_build_id_inconnu() {
    let decoded: DaemonToWrapper = decode(
            r#"{"type":"ClientWelcome","version":1,"horizon_secs":60,"issued_at_tolerance_secs":5,"capabilities":[]}"#,
        )
        .unwrap();
    assert!(matches!(
        decoded,
        DaemonToWrapper::ClientWelcome { build_id, .. } if build_id == "unknown"
    ));
}

#[test]
fn lifecycle_messages_roundtrip_and_stay_outside_attach() {
    let spawn = WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "codex".to_string(),
        project: Some(ProjectReference {
            project_id: "project-1".to_string(),
            binding_generation: 2,
        }),
        ownership: Some(SpawnOwnership {
            parent_instance_id: "instance-parent".to_string(),
            parent_execution_id: Some("execution-parent".to_string()),
            objective_id: Some("objective-1".to_string()),
            delegation_id: Some("delegation-1".to_string()),
            project: Some(ProjectReference {
                project_id: "project-1".to_string(),
                binding_generation: 2,
            }),
            role: "verification".to_string(),
            max_children: Some(3),
            max_depth: Some(2),
        }),
        agent_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        cwd: "/tmp".to_string(),
        persistent: true,
        command_id: "command-1".to_string(),
        issued_at: 100,
        deadline_at: 110,
    };
    assert!(matches!(
        decode(&encode(&spawn).unwrap()).unwrap(),
        WrapperToDaemon::SpawnOrder {
            agent_type,
            agent_id: Some(agent_id),
            command_id,
            ownership: Some(ownership),
            ..
        } if agent_type == "codex"
            && agent_id == "550e8400-e29b-41d4-a716-446655440000"
            && command_id == "command-1"
            && ownership.parent_instance_id == "instance-parent"
    ));
    assert_eq!(
        spawn.attach_refusal(),
        Some(AttachRefusal::MessageOutsideAttachRole)
    );
    let wait = WrapperToDaemon::WaitAgentLinks {
        after_cursor: Some(41),
        timeout_ms: 3_000,
    };
    assert!(matches!(
        decode(&encode(&wait).unwrap()).unwrap(),
        WrapperToDaemon::WaitAgentLinks {
            after_cursor: Some(41),
            timeout_ms: 3_000,
        }
    ));
    assert_eq!(
        wait.attach_refusal(),
        Some(AttachRefusal::MessageOutsideAttachRole)
    );
    let events = DaemonToWrapper::AgentLinkEvents {
        events: vec![AgentLinkEventFrame {
            cursor: 42,
            event_id: "link-1:2".to_string(),
            link_id: "link-1".to_string(),
            child_instance_id: "child-1".to_string(),
            state: "orphaned".to_string(),
            observed_at: 1_788_000_000,
            project: Some(ProjectReference {
                project_id: "project-1".to_string(),
                binding_generation: 2,
            }),
        }],
        through_cursor: Some(42),
        timed_out: false,
    };
    assert!(matches!(
        decode(&encode(&events).unwrap()).unwrap(),
        DaemonToWrapper::AgentLinkEvents {
            through_cursor: Some(42),
            timed_out: false,
            events,
        } if events.len() == 1 && events[0].state == "orphaned"
    ));
    let rejection = DaemonToWrapper::SpawnRejected {
        command_id: "command-1".to_string(),
        reason: SpawnRefusal::BillingGuard {
            variable: "OPENAI_API_KEY".to_string(),
        },
    };
    assert!(matches!(
        decode(&encode(&rejection).unwrap()).unwrap(),
        DaemonToWrapper::SpawnRejected {
            reason: SpawnRefusal::BillingGuard { variable },
            ..
        } if variable == "OPENAI_API_KEY"
    ));
    assert!(!rejection.allowed_for_attach());
    let stop = DaemonToWrapper::StopResult {
        command_id: "stop-1".to_string(),
        outcome: StopOutcome::StoppedForced {
            survivors_killed: 2,
        },
    };
    assert!(matches!(
        decode(&encode(&stop).unwrap()).unwrap(),
        DaemonToWrapper::StopResult {
            outcome: StopOutcome::StoppedForced {
                survivors_killed: 2
            },
            ..
        }
    ));
    let relaunch = DaemonToWrapper::RelaunchResult {
        command_id: "relaunch-1".to_string(),
        outcome: RelaunchOutcome::Started {
            agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            generation: 12,
        },
    };
    assert!(matches!(
        decode(&encode(&relaunch).unwrap()).unwrap(),
        DaemonToWrapper::RelaunchResult {
            outcome: RelaunchOutcome::Started { generation: 12, .. },
            ..
        }
    ));
    let decommission = DaemonToWrapper::DecommissionResult {
        command_id: "decommission-1".to_string(),
        outcome: DecommissionOutcome::DecommissionedForced {
            survivors_killed: 3,
        },
    };
    assert!(matches!(
        decode(&encode(&decommission).unwrap()).unwrap(),
        DaemonToWrapper::DecommissionResult {
            outcome: DecommissionOutcome::DecommissionedForced {
                survivors_killed: 3
            },
            ..
        }
    ));
    let adoption = DaemonToWrapper::AdoptStoppedResult {
        command_id: "adopt-1".to_string(),
        outcome: AdoptStoppedOutcome::Adopted { generation: 7 },
    };
    assert!(matches!(
        decode(&encode(&adoption).unwrap()).unwrap(),
        DaemonToWrapper::AdoptStoppedResult {
            outcome: AdoptStoppedOutcome::Adopted { generation: 7 },
            ..
        }
    ));
}

#[test]
fn attach_relay_messages_roundtrip() {
    let messages = vec![
        WrapperToDaemon::Subscribed {
            subscription_id: "sub-1".to_string(),
        },
        WrapperToDaemon::JournalFragment {
            subscription_id: "sub-1".to_string(),
            seq: 7,
            offset: 0,
            final_fragment: true,
            bytes: b"{\"v\":1}\n".to_vec(),
        },
        WrapperToDaemon::LiveJournalFragment {
            seq: 8,
            offset: 0,
            final_fragment: true,
            bytes: b"{\"v\":1,\"seq\":8}".to_vec(),
        },
        WrapperToDaemon::SnapshotCaughtUp {
            subscription_id: "sub-1".to_string(),
            through_seq: Some(7),
        },
        WrapperToDaemon::Gap {
            subscription_id: "sub-1".to_string(),
            from_seq: 3,
            to_seq: 4,
            reason: Some("vue lente".to_string()),
        },
        WrapperToDaemon::End {
            subscription_id: "sub-1".to_string(),
            reason: "wrapper arrêté".to_string(),
        },
        WrapperToDaemon::AttachRejected {
            subscription_id: Some("sub-1".to_string()),
            reason: AttachRefusal::CommandQueueSaturated,
        },
        WrapperToDaemon::JournalReadError {
            subscription_id: "sub-1".to_string(),
            line: 3,
            offset: 42,
            reason: "ligne illisible".to_string(),
        },
    ];
    for message in messages {
        assert_eq!(
            encode(&message).unwrap(),
            encode(&decode::<WrapperToDaemon>(&encode(&message).unwrap()).unwrap()).unwrap()
        );
    }
    assert_eq!(MAX_ATTACH_FRAGMENT_BYTES, 190 * 1024);
    assert_eq!(MAX_ATTACH_SERIALIZED_FRAME_BYTES, 256 * 1024);
}

#[test]
fn attach_daemon_messages_roundtrip() {
    let messages = vec![
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach,
        },
        DaemonToWrapper::Subscribe {
            subscription_id: "sub-1".to_string(),
            agent: "codex-1".to_string(),
            window: AttachWindow::Today,
        },
        DaemonToWrapper::Unsubscribe {
            subscription_id: "sub-1".to_string(),
        },
        DaemonToWrapper::Subscribed {
            subscription_id: "sub-1".to_string(),
        },
        DaemonToWrapper::JournalFragment {
            subscription_id: "sub-1".to_string(),
            seq: 8,
            offset: 0,
            final_fragment: true,
            bytes: vec![0, 1, 2],
        },
        DaemonToWrapper::SnapshotCaughtUp {
            subscription_id: "sub-1".to_string(),
            through_seq: None,
        },
        DaemonToWrapper::Gap {
            subscription_id: "sub-1".to_string(),
            from_seq: 6,
            to_seq: 7,
            reason: None,
        },
        DaemonToWrapper::End {
            subscription_id: "sub-1".to_string(),
            reason: "désabonné".to_string(),
        },
        DaemonToWrapper::AttachRejected {
            subscription_id: None,
            reason: AttachRefusal::AgentStopped,
            mode: None,
            location: None,
        },
        DaemonToWrapper::AttachRejected {
            subscription_id: None,
            reason: AttachRefusal::AgentNotAcp,
            mode: Some(PresenceMode::Tmux),
            location: Some("bridget:4.2".to_string()),
        },
        DaemonToWrapper::JournalReadError {
            subscription_id: "sub-1".to_string(),
            line: 3,
            offset: 42,
            reason: "ligne illisible".to_string(),
        },
        DaemonToWrapper::DeliveryRejected {
            id: "message-1".to_string(),
            reason: "processus arrêté".to_string(),
        },
    ];
    for message in messages {
        let reaches_attach = !matches!(
            message,
            DaemonToWrapper::Subscribe { .. } | DaemonToWrapper::Unsubscribe { .. }
        );
        let json = encode(&message).unwrap();
        let decoded: DaemonToWrapper = decode(&json).unwrap();
        assert_eq!(json, encode(&decoded).unwrap());
        assert_eq!(decoded.allowed_for_attach(), reaches_attach);
    }
}

#[test]
fn attach_rejected_historique_omet_les_details_de_presence() {
    let legacy = r#"{"type":"AttachRejected","reason":"agent_not_acp"}"#;
    let decoded: DaemonToWrapper = decode(legacy).unwrap();
    match decoded {
        DaemonToWrapper::AttachRejected {
            subscription_id: None,
            reason: AttachRefusal::AgentNotAcp,
            mode: None,
            location: None,
        } => {}
        other => panic!("message historique inattendu : {other:?}"),
    }
}

#[test]
fn journal_ready_est_un_signal_wrapper_hors_du_role_attach() {
    let signal = WrapperToDaemon::JournalReady {
        receipt_probe: false,
    };
    let json = encode(&signal).unwrap();
    assert_eq!(json, r#"{"type":"JournalReady"}"#);
    assert!(matches!(
        decode(&json).unwrap(),
        WrapperToDaemon::JournalReady { .. }
    ));
    assert_eq!(
        signal.attach_refusal(),
        Some(AttachRefusal::MessageOutsideAttachRole)
    );
}

#[test]
fn spec244_sonde_recu_est_annoncee_sans_casser_journal_ready_historique() {
    #[derive(serde::Deserialize)]
    #[serde(tag = "type")]
    enum Legacy {
        JournalReady,
    }
    let wire = encode(&WrapperToDaemon::JournalReady {
        receipt_probe: true,
    })
    .unwrap();
    assert!(matches!(
        serde_json::from_str::<Legacy>(&wire).unwrap(),
        Legacy::JournalReady
    ));
    assert!(matches!(
        decode(r#"{"type":"JournalReady"}"#).unwrap(),
        WrapperToDaemon::JournalReady {
            receipt_probe: false
        }
    ));
}

#[test]
fn spawn_accepted_transporte_la_definition_resolue_complete() {
    let message = DaemonToWrapper::SpawnAccepted {
        command_id: "command-1".to_string(),
        agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        definition: Some(ResolvedAgentDefinition {
            command: "npx".to_string(),
            args: vec!["adapter@1.2.3".to_string()],
            protocol: "acp".to_string(),
            forbidden_env: vec!["API_KEY".to_string()],
            pass_env: vec!["HOME".to_string()],
            claude_config_dir: None,
            codex_home: None,
            permissions: "allow".to_string(),
            queue_capacity: 32,
            notify_timeout_secs: 600,
            mcp: ResolvedMcpDefinition {
                interactive: "codex".to_string(),
                acp_session: true,
            },
            capabilities: AdapterCapabilities {
                account_profiles: false,
                execution_paths: vec!["acp".to_string()],
                models: BTreeMap::from([(
                    "gpt-5.6-terra".to_string(),
                    ModelCapabilities {
                        service_tiers: Vec::new(),
                        efforts: vec!["high".to_string()],
                    },
                )]),
                observed: None,
            },
            digest: "a".repeat(64),
        }),
    };
    let json = encode(&message).unwrap();
    let decoded = decode::<DaemonToWrapper>(&json).unwrap();
    assert!(matches!(
        decoded,
        DaemonToWrapper::SpawnAccepted { definition: Some(definition), .. }
            if definition.command == "npx"
                && definition.args == ["adapter@1.2.3"]
                && definition.forbidden_env == ["API_KEY"]
                && definition.capabilities.models["gpt-5.6-terra"].efforts == ["high"]
                && definition.digest == "a".repeat(64)
    ));
    assert!(matches!(
            decode::<DaemonToWrapper>(
                r#"{"type":"SpawnAccepted","command_id":"legacy","agent_id":"550e8400-e29b-41d4-a716-446655440000"}"#
            )
            .unwrap(),
            DaemonToWrapper::SpawnAccepted {
                definition: None,
                ..
            }
        ));
}

#[test]
fn fragment_base64_reste_sous_la_borne_filaire_pour_des_octets_hostiles() {
    for bytes in [
        vec![0; MAX_ATTACH_FRAGMENT_BYTES],
        vec![0xff; MAX_ATTACH_FRAGMENT_BYTES],
        "équipier".as_bytes().repeat(MAX_ATTACH_FRAGMENT_BYTES / 9),
    ] {
        let message = WrapperToDaemon::JournalFragment {
            subscription_id: "attach-0123456789abcdef".to_string(),
            seq: 7,
            offset: 0,
            final_fragment: true,
            bytes,
        };
        assert!(encode(&message).unwrap().len() <= MAX_ATTACH_SERIALIZED_FRAME_BYTES);
    }
}

#[test]
fn role_attach_refuse_les_messages_wrapper_et_reply_suivi() {
    assert!(WrapperToDaemon::ListAgents.attach_refusal().is_none());
    assert!(DaemonToWrapper::AgentList { agents: vec![] }.allowed_for_attach());
    let wrapper_only = WrapperToDaemon::Runtime {
        agent: "codex-1".to_string(),
        model: "gpt-5.5".to_string(),
        effort: None,
        source: RuntimeSource::Declared,
    };
    assert_eq!(
        wrapper_only.attach_refusal(),
        Some(AttachRefusal::MessageOutsideAttachRole)
    );

    let mut tracked_send = BridgetMessage::new("forge", "codex-1", "réponds");
    tracked_send.reply = true;
    assert_eq!(
        WrapperToDaemon::Send(tracked_send).attach_refusal(),
        Some(AttachRefusal::ReplyNotAllowed)
    );
    assert!(
        WrapperToDaemon::Send(BridgetMessage::new("humain", "codex-1", "bonjour"))
            .attach_refusal()
            .is_none()
    );
    assert!(
        !DaemonToWrapper::Deliver {
            message: BridgetMessage::new("a", "b", "x"),
            mid_turn_ok: None
        }
        .allowed_for_attach()
    );
}

#[test]
fn issue_differee_attach_reste_correlee_parmi_le_flux() {
    let flux = vec![
        DaemonToWrapper::JournalFragment {
            subscription_id: "sub-1".to_string(),
            seq: 11,
            offset: 0,
            final_fragment: true,
            bytes: b"premier".to_vec(),
        },
        DaemonToWrapper::DeliveryRejected {
            id: "message-humain".to_string(),
            reason: "file pleine".to_string(),
        },
        DaemonToWrapper::JournalFragment {
            subscription_id: "sub-1".to_string(),
            seq: 12,
            offset: 0,
            final_fragment: true,
            bytes: b"second".to_vec(),
        },
    ];
    let decoded = flux
        .into_iter()
        .map(|message| decode::<DaemonToWrapper>(&encode(&message).unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        decoded[0],
        DaemonToWrapper::JournalFragment { seq: 11, .. }
    ));
    assert!(
        matches!(decoded[1], DaemonToWrapper::DeliveryRejected { ref id, .. } if id == "message-humain")
    );
    assert!(matches!(
        decoded[2],
        DaemonToWrapper::JournalFragment { seq: 12, .. }
    ));
}

#[test]
fn protocol_007_register_legacy_est_refuse() {
    let json = r#"{\"type\":\"Register\",\"agent_type\":\"codex\",\"name\":null}"#;
    assert!(decode::<WrapperToDaemon>(json).is_err());
}

#[test]
fn refus_type_inconnu_historique_reste_lisible_apres_l_ajout_du_diagnostic() {
    let legacy: SpawnRefusal = serde_json::from_str(r#"{"kind":"unknown_type"}"#).unwrap();
    assert!(matches!(
        legacy,
        SpawnRefusal::UnknownType {
            requested_type,
            known_types,
            registry,
        } if requested_type.is_empty() && known_types.is_empty() && registry.is_empty()
    ));
}

#[test]
fn test_encode_decode_bounded_ledger_projection() {
    let request = WrapperToDaemon::LedgerProjection {
        scope: LedgerScope::Both,
        limit: 20,
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&request).unwrap()).unwrap(),
        WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Both,
            limit: 20
        }
    ));

    let response = DaemonToWrapper::LedgerProjection {
        messages: vec![LedgerMessage {
            id: "m-1".to_string(),
            ts: 42,
            sender: "alice".to_string(),
            target: "bob".to_string(),
            body: "bonjour".to_string(),
            delivery_status: Some(LedgerDeliveryStatus::EnVol),
        }],
        requests: Vec::new(),
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(&encode(&response).unwrap()).unwrap(),
        DaemonToWrapper::LedgerProjection { messages, requests }
            if messages.len() == 1
                && messages[0].id == "m-1"
                && messages[0].delivery_status == Some(LedgerDeliveryStatus::EnVol)
                && requests.is_empty()
    ));
}

/// Oracle : les phases nommées ont une correspondance. Meurt si l'on
/// retire un arm (état SQL sans rendu lisible au ledger).
#[test]
fn from_phase_garde_indetermine_parmi_les_trois_etats() {
    assert_eq!(
        LedgerDeliveryStatus::from_phase("dispatching"),
        Some(LedgerDeliveryStatus::EnVol)
    );
    assert_eq!(
        LedgerDeliveryStatus::from_phase("acked"),
        Some(LedgerDeliveryStatus::Recu)
    );
    assert_eq!(
        LedgerDeliveryStatus::from_phase("indeterminate"),
        Some(LedgerDeliveryStatus::Indetermine),
        "retirer cette correspondance laisse la quarantaine muette au ledger"
    );
    assert_eq!(
        LedgerDeliveryStatus::from_phase("orphaned"),
        Some(LedgerDeliveryStatus::Orphelin),
        "orphelin doit se rendre au ledger, distinct de indéterminé"
    );
    assert_eq!(LedgerDeliveryStatus::Indetermine.label_fr(), "indéterminé");
    assert_eq!(LedgerDeliveryStatus::Orphelin.label_fr(), "orphelin");
    assert_eq!(LedgerDeliveryStatus::from_phase("autre"), None);
}
#[test]
fn execution_control_contract_roundtrip_et_version_explicit() {
    let mut message = BridgetMessage::new("humain", "agent-fixture", "corrige ce point");
    message.id = "message-steer-fixture".to_string();
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "control-fixture".to_string(),
        execution_id: "execution-fixture".to_string(),
        generation: 7,
        revision: 3,
        operation: ExecutionControlOperation::SteerCurrent,
        message: Some(message),
    };
    let wire = encode(&command).unwrap();
    let decoded: ExecutionControlCommand = decode(&wire).unwrap();
    assert_eq!(decoded, command);
    assert!(wire.contains("\"version\":1"));
}

#[test]
fn execution_control_refusal_reste_ferme_sur_le_fil() {
    let refusal = ExecutionControlRefusal::CapabilityUnavailable;
    assert_eq!(
        serde_json::to_string(&refusal).unwrap(),
        "\"capability_unavailable\""
    );
    assert!(serde_json::from_str::<ExecutionControlRefusal>("\"unknown\"").is_err());
}
#[test]
fn spec_068_trames_incident_deleguees_sont_fermees_et_redacted() {
    let emitted = WrapperToDaemon::DelegatedRuntimeEvent {
        execution_id: "execution-child-42".to_string(),
        kind: DelegatedRuntimeEventKind::Warning,
        code: "unsupported_provider_request".to_string(),
        reference: "sha256:ab12".to_string(),
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&emitted).unwrap()).unwrap(),
        WrapperToDaemon::DelegatedRuntimeEvent {
            execution_id,
            kind: DelegatedRuntimeEventKind::Warning,
            code,
            reference,
        } if execution_id == "execution-child-42"
            && code == "unsupported_provider_request"
            && reference == "sha256:ab12"
    ));

    let delivery = DaemonToWrapper::DelegatedRuntimeEvent {
        event: DelegatedRuntimeEventFrame {
            cursor: 7,
            event_id: "runtime-link-1-execution-child-42-warning".to_string(),
            link_id: "link-1".to_string(),
            child_instance_id: "instance-child".to_string(),
            child_execution_id: "execution-child-42".to_string(),
            kind: DelegatedRuntimeEventKind::Warning,
            code: "unsupported_provider_request".to_string(),
            reference: "sha256:ab12".to_string(),
            observed_at: 1_788_000_000,
            project: Some(ProjectReference {
                project_id: "project-1".to_string(),
                binding_generation: 2,
            }),
        },
    };
    let wire = encode(&delivery).unwrap();
    assert!(matches!(
        decode::<DaemonToWrapper>(&wire).unwrap(),
        DaemonToWrapper::DelegatedRuntimeEvent { event }
            if event.cursor == 7
                && event.kind == DelegatedRuntimeEventKind::Warning
                && event.code == "unsupported_provider_request"
                && event.reference == "sha256:ab12"
    ));
    assert!(!wire.contains("params"));
    assert!(!wire.contains("secret"));

    let acknowledgement = WrapperToDaemon::DelegatedRuntimeEventAcknowledged {
        event_id: "runtime-link-1-execution-child-42-warning".to_string(),
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&acknowledgement).unwrap()).unwrap(),
        WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id }
            if event_id == "runtime-link-1-execution-child-42-warning"
    ));
    assert!(serde_json::from_str::<DelegatedRuntimeEventKind>("\"other\"").is_err());
}

#[test]
fn spec_065_registre_projet_est_ferme_directionnel_et_negocie() {
    let request = ProjectBindRequest {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: "project-command-1".to_string(),
        issued_at: 1_787_997_600,
        deadline_at: 1_787_998_200,
        project_id: "project-opaque-1".to_string(),
        requested_root: "/srv/projects/fixture".to_string(),
        backend: ProjectBackend::Host,
        policy_id: None,
        policy_version: None,
    };
    let message = WrapperToDaemon::ProjectRegistryRequest {
        request: request.clone(),
    };
    let wire = encode(&message).unwrap();
    assert!(wire.contains(r#""type":"project_registry_request""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::ProjectRegistryRequest { request: decoded }
            if decoded == request
    ));
    assert!(serde_json::from_str::<ProjectBindRequest>(
            r#"{"contract_version":1,"command_id":"project-command-1","issued_at":1,"deadline_at":2,"project_id":"project-opaque-1","requested_root":"/srv/projects/fixture","backend":"host","unexpected":true}"#
        )
        .is_err());

    let hello = WrapperToDaemon::ServiceHello {
        version: SERVICE_CONTRACT_VERSION,
        service: "guichet".to_string(),
        issuer_scope: "065_scope_0123456789abcdef0123456789abcdef".to_string(),
        capabilities: vec![ServiceCapability::ProjectRegistryV1],
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&hello).unwrap()).unwrap(),
        WrapperToDaemon::ServiceHello {
            service,
            capabilities,
            ..
        } if service == "guichet" && capabilities == vec![ServiceCapability::ProjectRegistryV1]
    ));

    let conflict = ProjectBindOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: "project-command-2".to_string(),
        project_id: "project-loser".to_string(),
        status: ProjectBindStatus::RegistrationConflict,
        binding_generation: None,
        backend: None,
        runtime_policy: None,
        reason: Some(ProjectRegistryRefusal::RootAlreadyBound),
        existing_project_id: Some("project-winner".to_string()),
        existing_binding_generation: Some(4),
        observed_at: 1_787_997_601,
    };
    let outcome = DaemonToWrapper::ProjectRegistryOutcome {
        outcome: conflict.clone(),
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(&encode(&outcome).unwrap()).unwrap(),
        DaemonToWrapper::ProjectRegistryOutcome { outcome: decoded }
            if decoded == conflict
    ));

    let admin_request = ProjectAdminRequest {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: "project-rebind-1".to_string(),
        issued_at: 1_787_997_602,
        deadline_at: 1_787_998_202,
        operation: ProjectAdminOperation::Rebind,
        project_id: Some("project-winner".to_string()),
        requested_root: Some("/srv/projects/other".to_string()),
    };
    let admin_wire = encode(&WrapperToDaemon::ProjectRegistryAdminRequest {
        request: admin_request.clone(),
    })
    .unwrap();
    assert!(admin_wire.contains(r#""type":"project_registry_admin_request""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&admin_wire).unwrap(),
        WrapperToDaemon::ProjectRegistryAdminRequest { request }
            if request == admin_request
    ));
    let admin_outcome = ProjectAdminOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: admin_request.command_id.clone(),
        operation: ProjectAdminOperation::Rebind,
        bindings: vec![ProjectBindingProjection {
            project_id: "project-winner".to_string(),
            canonical_root: None,
            state: ProjectBindingStatus::Active,
            binding_generation: Some(2),
            backend: Some(ProjectBackend::Host),
            role: ProjectRole::Standard,
            runtime_policy: None,
            reason: None,
            last_audit: None,
            observed_at: 1_787_997_603,
        }],
        reason: None,
        observed_at: 1_787_997_603,
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(
            &encode(&DaemonToWrapper::ProjectRegistryAdminOutcome {
                outcome: admin_outcome.clone(),
            })
            .unwrap()
        )
        .unwrap(),
        DaemonToWrapper::ProjectRegistryAdminOutcome { outcome }
            if outcome == admin_outcome
    ));

    let version_refusal = ProjectRegistryRefusal::ProjectRegistryVersionUnsupported;
    assert_eq!(
        serde_json::to_string(&version_refusal).unwrap(),
        "\"project_registry_version_unsupported\""
    );
    assert_eq!(
        serde_json::to_string(&ProjectRegistryRefusal::PeerUidMismatch).unwrap(),
        "\"peer_uid_mismatch\""
    );
    assert!(serde_json::from_str::<ProjectRegistryRefusal>("\"unknown\"").is_err());
}
#[test]
fn spec_066_runtime_local_est_versionne_ferme_et_sans_chemin_hote() {
    let request = ProjectRuntimeRequest {
        contract_version: 1,
        command_id: "runtime-command-1".to_string(),
        issued_at: 1_788_000_000,
        deadline_at: 1_788_000_060,
        operation: ProjectRuntimeOperation::Prepare,
        project_id: "project-066".to_string(),
        expected_binding_generation: None,
        policy_id: None,
        policy_version: None,
        profile: None,
    };
    let wire = encode(&WrapperToDaemon::ProjectRuntimeRequest {
        request: request.clone(),
    })
    .unwrap();
    assert!(wire.contains(r#""type":"project_runtime_request""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::ProjectRuntimeRequest { request: decoded } if decoded == request
    ));
    assert!(serde_json::from_str::<ProjectRuntimeRequest>(
            r#"{"contract_version":1,"command_id":"runtime-command-1","issued_at":1,"deadline_at":2,"operation":"prepare","project_id":"project-066","path":"/srv/private"}"#
        )
        .is_err());

    let outcome = ProjectRuntimeOutcome {
        contract_version: 1,
        command_id: request.command_id.clone(),
        project_id: request.project_id.clone(),
        operation: request.operation,
        binding_generation: Some(2),
        state: Some("ready".to_string()),
        runtime_policy: Some(ProjectRuntimePolicyReference {
            policy_id: "fixture-local".to_string(),
            policy_version: 1,
            policy_digest: format!("sha256:{}", "a".repeat(64)),
            environment_epoch: 3,
        }),
        last_reason: Some("runtime_exec_lost".to_string()),
        reason: None,
        observed_at: 1_788_000_001,
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(
            &encode(&DaemonToWrapper::ProjectRuntimeOutcome {
                outcome: outcome.clone(),
            })
            .unwrap()
        )
        .unwrap(),
        DaemonToWrapper::ProjectRuntimeOutcome { outcome: decoded } if decoded == outcome
    ));
}

#[test]
fn spec_085_activate_docker_exige_une_reference_fermee_et_reste_versionne() {
    let request = ProjectRuntimeRequest {
        contract_version: PROJECT_RUNTIME_CONTRACT_VERSION,
        command_id: "activate-docker-085".to_string(),
        issued_at: 1,
        deadline_at: 2,
        operation: ProjectRuntimeOperation::ActivateDocker,
        project_id: "project-085".to_string(),
        expected_binding_generation: Some(7),
        policy_id: Some("production-linux-amd64".to_string()),
        policy_version: Some(1),
        profile: None,
    };
    let wire = serde_json::to_string(&request).unwrap();
    assert!(wire.contains("\"operation\":\"activate_docker\""));
    assert!(wire.contains("\"expected_binding_generation\":7"));
    assert!(wire.contains("\"policy_id\":\"production-linux-amd64\""));
    assert!(serde_json::from_str::<ProjectRuntimeRequest>(
            r#"{"contract_version":1,"command_id":"x","issued_at":1,"deadline_at":2,"operation":"activate_docker","project_id":"p","image":"latest"}"#
        ).is_err());
}
#[test]
fn spec_066_handshake_ingress_est_ferme_et_versionne() {
    let hello = RuntimeIngressHandshake {
        contract_version: RUNTIME_INGRESS_CONTRACT_VERSION,
        project_id: "project-066".to_string(),
        binding_generation: 2,
        container_id: "a".repeat(64),
        environment_epoch: 3,
        agent_generation: 4,
        instance_id: "00000000-0000-4000-8000-000000000066".to_string(),
    };
    let wire = encode(&WrapperToDaemon::RuntimeIngressHello {
        hello: hello.clone(),
    })
    .unwrap();
    assert!(wire.contains(r#""type":"runtime_ingress_hello""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::RuntimeIngressHello { hello: decoded } if decoded == hello
    ));
    assert!(serde_json::from_str::<RuntimeIngressHandshake>(
            r#"{"contract_version":1,"project_id":"project-066","binding_generation":2,"container_id":"abc","environment_epoch":3,"agent_generation":4,"host_path":"/srv/private"}"#
        )
        .is_err());
    assert_eq!(
        serde_json::to_string(&RuntimeIngressRefusal::EnvironmentEpochStale).unwrap(),
        "\"environment_epoch_stale\""
    );
}
#[test]
fn spec_079_contrat_ronde_projet_est_ferme_versionne_et_rejouable() {
    let request = ProjectRoundRequest {
        contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
        command_id: "round-enable-1".to_string(),
        issued_at: 1_788_000_000,
        deadline_at: 1_788_000_060,
        operation: ProjectRoundOperation::Enable,
        project_id: Some("project-079".to_string()),
        binding_generation: Some(4),
    };
    let wire = encode(&WrapperToDaemon::ProjectRoundRequest {
        request: request.clone(),
    })
    .unwrap();
    assert!(wire.contains(r#""type":"project_round_request""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::ProjectRoundRequest { request: decoded } if decoded == request
    ));
    assert!(serde_json::from_str::<ProjectRoundRequest>(
            r#"{"contract_version":1,"command_id":"round-enable-1","issued_at":1,"deadline_at":2,"operation":"enable","project_id":"project-079","binding_generation":4,"provider":"codex"}"#
        )
        .is_err());

    let legacy_projection: ProjectRoundProjection = serde_json::from_str(
            r#"{"project_id":"project-079","binding_generation":4,"active":true,"configured":true,"enabled":true,"revision":1,"updated_at":1788000000}"#,
        )
        .unwrap();
    assert_eq!(legacy_projection.last_occurrence_at, None);
    assert_eq!(legacy_projection.last_dispatch_state, None);
    assert_eq!(legacy_projection.last_dispatch_observed_at, None);

    let observed_projection = ProjectRoundProjection {
        last_occurrence_at: Some(1_788_000_000),
        last_dispatch_state: Some(ProjectRoundDispatchState::Deposited),
        last_dispatch_observed_at: Some(1_788_000_001),
        ..legacy_projection
    };
    let projection_wire = serde_json::to_string(&observed_projection).unwrap();
    assert!(projection_wire.contains(r#""last_dispatch_state":"deposited""#));
    assert_eq!(
        serde_json::from_str::<ProjectRoundProjection>(&projection_wire).unwrap(),
        observed_projection
    );

    let dispatch = ProjectRoundDispatchRequest {
        contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
        occurrence_at: 1_788_000_000,
        project: ProjectReference {
            project_id: "project-079".to_string(),
            binding_generation: 4,
        },
    };
    let wire = encode(&WrapperToDaemon::ProjectRoundDispatch {
        request: dispatch.clone(),
    })
    .unwrap();
    assert!(wire.contains(r#""type":"project_round_dispatch""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::ProjectRoundDispatch { request: decoded } if decoded == dispatch
    ));
    assert!(serde_json::from_str::<ProjectRoundDispatchRequest>(
            r#"{"contract_version":1,"occurrence_at":1788000000,"project":{"project_id":"project-079","binding_generation":4},"cwd":"/srv/private"}"#
        )
        .is_err());

    let outcome = ProjectRoundDispatchOutcome {
        contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
        occurrence_at: dispatch.occurrence_at,
        project: dispatch.project,
        issue: Some(IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_788_604_800,
            delivery_id: Some("delivery-079".to_string()),
        }),
        reason: None,
        observed_at: 1_788_000_001,
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(
            &encode(&DaemonToWrapper::ProjectRoundDispatchOutcome {
                outcome: outcome.clone(),
            })
            .unwrap()
        )
        .unwrap(),
        DaemonToWrapper::ProjectRoundDispatchOutcome { outcome: decoded }
            if decoded == outcome
    ));
}

#[test]
fn spec_086_role_projet_est_ferme_et_ancien_projection_reste_standard() {
    let legacy: ProjectBindingProjection =
        serde_json::from_str(r#"{"project_id":"legacy","state":"active","observed_at":1}"#)
            .unwrap();
    assert_eq!(legacy.role, ProjectRole::Standard);
    let system = ProjectBindingProjection {
        project_id: "bridget-system".to_string(),
        canonical_root: None,
        state: ProjectBindingStatus::Active,
        binding_generation: Some(1),
        backend: Some(ProjectBackend::Docker),
        role: ProjectRole::BridgetSystem,
        runtime_policy: None,
        reason: None,
        last_audit: None,
        observed_at: 2,
    };
    let wire = serde_json::to_string(&system).unwrap();
    assert!(wire.contains(r#""role":"bridget_system""#));
    assert_eq!(
        serde_json::from_str::<ProjectBindingProjection>(&wire).unwrap(),
        system
    );
}

#[test]
fn spec_086_contrat_systeme_est_separe_du_registre_standard() {
    let request = ProjectSystemRequest {
        contract_version: PROJECT_SYSTEM_CONTRACT_VERSION,
        command_id: "system-declare-086".to_string(),
        issued_at: 1,
        deadline_at: 2,
        operation: ProjectSystemOperation::Declare,
        project_id: "bridget-system".to_string(),
        expected_binding_generation: 3,
        runtime_policy_id: None,
        runtime_policy_version: None,
        expected_setting_generation: None,
        requested_mode: None,
    };
    let wire = encode(&WrapperToDaemon::ProjectSystemRequest {
        request: request.clone(),
    })
    .unwrap();
    assert!(wire.contains(r#""type":"project_system_request""#));
    assert!(matches!(
        decode::<WrapperToDaemon>(&wire).unwrap(),
        WrapperToDaemon::ProjectSystemRequest { request: decoded } if decoded == request
    ));
    assert!(serde_json::from_str::<ProjectSystemRequest>(
            r#"{"contract_version":1,"command_id":"x","issued_at":1,"deadline_at":2,"operation":"declare","project_id":"project","expected_binding_generation":1,"root":"/tmp"}"#,
        )
        .is_err());
}
