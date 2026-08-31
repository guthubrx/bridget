use bridget_daemon::project_runtime::{
    ImageReferenceKind, ProjectEnvironment, ProjectEnvironmentState, ProjectRuntimePolicy,
    RuntimeIngressExpectation, bind_runtime_ingress,
};
use bridget_transport::protocol::{
    RUNTIME_INGRESS_CONTRACT_VERSION, RuntimeIngressHandshake, RuntimeIngressRefusal,
    WrapperToDaemon, decode, encode,
};
use std::path::PathBuf;

fn policy(state_root_parent: PathBuf) -> ProjectRuntimePolicy {
    ProjectRuntimePolicy {
        policy_id: "ingress-test".to_string(),
        policy_version: 1,
        image_reference_kind: ImageReferenceKind::LocalImageId,
        image_reference: format!("sha256:{}", "a".repeat(64)),
        run_as_uid: unsafe { libc::geteuid() },
        run_as_gid: unsafe { libc::getegid() },
        cpu_limit: 1.0,
        memory_limit_bytes: 128 * 1024 * 1024,
        pids_limit: 64,
        tmpfs: vec!["/tmp".to_string()],
        network_mode: "bridge".to_string(),
        runtime_launcher: None,
        runtime_executables: std::collections::BTreeMap::new(),
        state_root_parent,
        digest: format!("sha256:{}", "b".repeat(64)),
    }
}

fn ready_environment(
    project_id: &str,
    binding_generation: u64,
    container_id: char,
    policy: &ProjectRuntimePolicy,
) -> ProjectEnvironment {
    let mut environment =
        ProjectEnvironment::absent(project_id, binding_generation, policy).unwrap();
    environment
        .transition(ProjectEnvironmentState::Creating, None)
        .unwrap();
    environment.container_id = Some(container_id.to_string().repeat(64));
    environment
        .transition(ProjectEnvironmentState::Ready, None)
        .unwrap();
    environment
}

fn hello(expectation: &RuntimeIngressExpectation) -> RuntimeIngressHandshake {
    RuntimeIngressHandshake {
        contract_version: RUNTIME_INGRESS_CONTRACT_VERSION,
        project_id: expectation.project_id.clone(),
        binding_generation: expectation.binding_generation,
        container_id: expectation.container_id.clone(),
        environment_epoch: expectation.environment_epoch,
        agent_generation: expectation.agent_generation,
        instance_id: expectation.instance_id.clone(),
    }
}

#[test]
fn spec_066_ingress_prive_isole_projets_et_refuse_les_identites_forgees() {
    let root = PathBuf::from("/tmp").join(format!("r66-ingress-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).unwrap();
    let policy = policy(root.clone());
    let environment_a = ready_environment("project-a", 2, 'a', &policy);
    let environment_b = ready_environment("project-b", 3, 'b', &policy);
    let initial_a = bind_runtime_ingress(&policy, &environment_a).unwrap();
    let socket_a = initial_a.socket_path.clone();
    let endpoint_b = bind_runtime_ingress(&policy, &environment_b).unwrap();
    assert_ne!(socket_a, endpoint_b.socket_path);
    assert!(socket_a.exists());
    assert!(endpoint_b.socket_path.exists());

    drop(initial_a);
    let rebound_a = bind_runtime_ingress(&policy, &environment_a).unwrap();
    assert_eq!(rebound_a.socket_path, socket_a);

    let expectation_a = RuntimeIngressExpectation::from_environment(
        &environment_a,
        7,
        "00000000-0000-4000-8000-000000000066",
    )
    .unwrap();
    let expectation_b = RuntimeIngressExpectation::from_environment(
        &environment_b,
        8,
        "10000000-0000-4000-8000-000000000066",
    )
    .unwrap();
    let accepted = hello(&expectation_a);
    assert_eq!(expectation_a.validate(&accepted), Ok(()));
    assert_eq!(
        expectation_b.validate(&accepted),
        Err(RuntimeIngressRefusal::IdentityMismatch),
        "un wrapper de A ne peut jamais utiliser l'ingress de B"
    );

    let mut forged = accepted.clone();
    forged.agent_generation += 1;
    assert_eq!(
        expectation_a.validate(&forged),
        Err(RuntimeIngressRefusal::GenerationMismatch)
    );
    forged.agent_generation = accepted.agent_generation;
    forged.environment_epoch += 1;
    assert_eq!(
        expectation_a.validate(&forged),
        Err(RuntimeIngressRefusal::EnvironmentEpochStale)
    );
    forged.environment_epoch = accepted.environment_epoch;
    forged.instance_id = "20000000-0000-4000-8000-000000000066".to_string();
    assert_eq!(
        expectation_a.validate(&forged),
        Err(RuntimeIngressRefusal::IdentityMismatch)
    );

    let acknowledgement = WrapperToDaemon::DelegatedRuntimeEventAcknowledged {
        event_id: "runtime-event-066".to_string(),
    };
    assert!(matches!(
        decode::<WrapperToDaemon>(&encode(&acknowledgement).unwrap()).unwrap(),
        WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id }
            if event_id == "runtime-event-066"
    ));

    drop(rebound_a);
    drop(endpoint_b);
    let _ = std::fs::remove_dir_all(root);
}
