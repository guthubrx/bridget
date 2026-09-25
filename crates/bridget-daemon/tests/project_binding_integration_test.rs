use bridget_daemon::desired_state::DesiredStateStore;
use bridget_daemon::fleet::{FleetConfig, FleetSupervisor, SpawnOrder, SpawnSubmission};
use bridget_daemon::store::Store as BridgetStore;
use bridget_transport::protocol::{ProjectAdminOperation, ProjectReference};
use std::path::PathBuf;

const NOW: i64 = 1_790_000_000;

fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-spec065-binding-{}",
        uuid::Uuid::new_v4().simple()
    ))
}

#[test]
fn rebind_n_arrete_pas_la_generation_deja_admise_et_ne_double_pas_l_audit() {
    let root = root();
    let initial_root = root.join("initial");
    let rebound_root = root.join("rebound");
    std::fs::create_dir_all(&initial_root).unwrap();
    std::fs::create_dir_all(&rebound_root).unwrap();

    let mut registry = BridgetStore::open(&root.join("registry.db")).unwrap();
    let registered = registry
        .bind_project_registration(
            "register-project-active",
            "project-active",
            initial_root.to_str().unwrap(),
            NOW,
        )
        .unwrap();
    assert_eq!(registered.binding_generation, Some(1));

    let fleet = FleetSupervisor::open(
        &root.join("fleet.db"),
        DesiredStateStore::at_path(root.join("fleet.json")),
        FleetConfig {
            quota: 2,
            persistent_horizon_secs: 3_600,
            ephemeral_horizon_secs: 300,
            issued_at_tolerance_secs: 30,
        },
    )
    .unwrap();
    let project = ProjectReference {
        project_id: "project-active".to_string(),
        binding_generation: 1,
    };
    let lease = match fleet
        .request_spawn(
            &SpawnOrder {
                posture: None,
                agent_type: "fixture".to_string(),
                project: Some(project.clone()),
                requested_name: Some("89000000-0000-4000-8000-000000000401".to_string()),
                cwd: initial_root.clone(),
                persistent: true,
                command_id: "spawn-active-project".to_string(),
                issued_at: NOW,
                deadline_at: NOW + 60,
                ownership: None,
            },
            NOW,
        )
        .unwrap()
    {
        SpawnSubmission::Start(lease) => lease,
        other => panic!("réservation attendue: {other:?}"),
    };
    assert_eq!(lease.project, Some(project.clone()));

    let rebound = registry
        .apply_project_admin_mutation(
            "rebind-project-active",
            ProjectAdminOperation::Rebind,
            "project-active",
            Some(rebound_root.to_str().unwrap()),
            NOW + 1,
        )
        .unwrap();
    assert_eq!(rebound.bindings[0].binding_generation, Some(2));
    assert_eq!(
        registry
            .apply_project_admin_mutation(
                "rebind-project-active",
                ProjectAdminOperation::Rebind,
                "project-active",
                Some(rebound_root.to_str().unwrap()),
                NOW + 2,
            )
            .unwrap(),
        rebound
    );

    let admitted = fleet.recovery_candidates();
    assert_eq!(admitted.len(), 1);
    assert_eq!(admitted[0].lease.project, Some(project));
    assert!(fleet.knows_command("spawn-active-project"));
    assert_eq!(
        registry
            .project_audit_events("project-active")
            .unwrap()
            .len(),
        2,
        "register et rebind, même après replay"
    );

    drop(fleet);
    drop(registry);
    std::fs::remove_dir_all(root).unwrap();
}
