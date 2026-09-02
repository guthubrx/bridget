use bridget_transport::protocol::{
    ProjectBackend, ProjectBindRequest, ProjectBindingProjection, ProjectBindingStatus,
    ProjectRole, ProjectRuntimePolicyReference, SpawnRefusal,
};

#[test]
fn spec_066_docker_contract_is_additive_for_host_history() {
    let legacy = serde_json::from_str::<ProjectBindRequest>(
        r#"{"contract_version":1,"command_id":"old","issued_at":1,"deadline_at":2,"project_id":"project-a","requested_root":"/srv/a","backend":"host"}"#,
    )
    .unwrap();
    assert_eq!(legacy.backend, ProjectBackend::Host);
    assert_eq!(legacy.policy_id, None);
    assert_eq!(legacy.policy_version, None);

    let docker = ProjectBindRequest {
        contract_version: 1,
        command_id: "docker-command".to_string(),
        issued_at: 1,
        deadline_at: 2,
        project_id: "project-a".to_string(),
        requested_root: "/srv/a".to_string(),
        backend: ProjectBackend::Docker,
        policy_id: Some("fixture-local".to_string()),
        policy_version: Some(1),
    };
    let decoded =
        serde_json::from_str::<ProjectBindRequest>(&serde_json::to_string(&docker).unwrap())
            .unwrap();
    assert_eq!(decoded.backend, ProjectBackend::Docker);
    assert_eq!(decoded.policy_id.as_deref(), Some("fixture-local"));

    let policy = ProjectRuntimePolicyReference {
        policy_id: "fixture-local".to_string(),
        policy_version: 1,
        policy_digest: "sha256:abc".to_string(),
        environment_epoch: 3,
    };
    let projection = ProjectBindingProjection {
        project_id: "project-a".to_string(),
        canonical_root: Some("/srv/a".to_string()),
        state: ProjectBindingStatus::Active,
        binding_generation: Some(1),
        backend: Some(ProjectBackend::Docker),
        role: ProjectRole::Standard,
        runtime_policy: Some(policy.clone()),
        reason: None,
        last_audit: None,
        observed_at: 3,
    };
    let decoded = serde_json::from_str::<ProjectBindingProjection>(
        &serde_json::to_string(&projection).unwrap(),
    )
    .unwrap();
    assert_eq!(decoded.runtime_policy, Some(policy));
}

#[test]
fn spec_066_refus_docker_sans_repli_est_structure_et_stable() {
    let refusal = SpawnRefusal::DockerRuntimeUnavailable {
        project_id: "project-066".to_string(),
    };
    let encoded = serde_json::to_string(&refusal).unwrap();
    assert_eq!(
        encoded,
        r#"{"kind":"docker_runtime_unavailable","project_id":"project-066"}"#
    );
    assert_eq!(
        serde_json::from_str::<SpawnRefusal>(&encoded).unwrap(),
        refusal
    );
}
