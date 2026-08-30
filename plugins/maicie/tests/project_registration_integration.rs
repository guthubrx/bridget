use bridget_transport::protocol::{
    ProjectBackend, ProjectBindOutcome, ProjectBindRequest, ProjectBindStatus,
    ProjectRegistryRefusal,
};
use maicie::app::{
    ProjectRegistrationRequest, prepare_project_registration, project_registration_request_bytes,
    resolve_project_registration,
};
use maicie::domain::ProjectIdentityStatus;
use maicie::store::{MaicieStore, ProjectRegistrationIntent, ProjectRegistrationState};
use std::sync::{Arc, Barrier};

fn registration(
    command_id: &str,
    project_id: &str,
    requested_root: &str,
) -> ProjectRegistrationIntent {
    ProjectRegistrationIntent {
        command_id: command_id.to_string(),
        proposed_project_id: project_id.to_string(),
        display_name: format!("Projet {project_id}"),
        requested_root: requested_root.to_string(),
        canonical_payload: format!("registry-v1:{command_id}:{project_id}:{requested_root}")
            .into_bytes(),
        created_at: 1_788_000_000,
        retry_until: 1_788_003_600,
    }
}

fn active_outcome(command_id: &str, project_id: &str) -> ProjectBindOutcome {
    ProjectBindOutcome {
        contract_version: 1,
        command_id: command_id.to_string(),
        project_id: project_id.to_string(),
        status: ProjectBindStatus::Active,
        binding_generation: Some(4),
        backend: Some(ProjectBackend::Host),
        reason: None,
        existing_project_id: None,
        existing_binding_generation: None,
        observed_at: 1_788_000_010,
    }
}

#[test]
fn spec_065_intention_registre_est_idempotente_et_deux_intentions_restent_pending() {
    let path = std::env::temp_dir().join(format!(
        "maicie-project-registration-{}.db",
        uuid::Uuid::new_v4()
    ));
    let mut store = MaicieStore::open(&path).unwrap();
    let first = registration("project-command-1", "project-1", "/srv/projects/shared");
    let prepared = store.prepare_project_registration(&first).unwrap();
    assert_eq!(prepared.state, ProjectRegistrationState::Prepared);
    assert_eq!(
        prepared.identity.status,
        ProjectIdentityStatus::PendingBinding
    );
    assert!(prepared.outbox_pending);

    let replay = store.prepare_project_registration(&first).unwrap();
    assert_eq!(replay, prepared);
    let divergent = ProjectRegistrationIntent {
        canonical_payload: b"divergent".to_vec(),
        ..first.clone()
    };
    assert!(matches!(
        store.prepare_project_registration(&divergent),
        Err(maicie::store::StoreError::EnvelopeMismatch)
    ));
    drop(store);

    let barrier = Arc::new(Barrier::new(3));
    let second = registration("project-command-2", "project-2", "/srv/projects/shared");
    let third = registration("project-command-3", "project-3", "/srv/projects/shared");
    let first_path = path.clone();
    let first_barrier = Arc::clone(&barrier);
    let left = std::thread::spawn(move || {
        first_barrier.wait();
        let mut thread_store = MaicieStore::open(first_path).unwrap();
        thread_store.prepare_project_registration(&second).unwrap()
    });
    let second_path = path.clone();
    let second_barrier = Arc::clone(&barrier);
    let right = std::thread::spawn(move || {
        second_barrier.wait();
        let mut thread_store = MaicieStore::open(second_path).unwrap();
        thread_store.prepare_project_registration(&third).unwrap()
    });
    barrier.wait();
    assert_eq!(
        left.join().unwrap().identity.status,
        ProjectIdentityStatus::PendingBinding
    );
    assert_eq!(
        right.join().unwrap().identity.status,
        ProjectIdentityStatus::PendingBinding
    );

    let store = MaicieStore::open(&path).unwrap();
    let identities = store.project_identities().unwrap();
    assert_eq!(identities.len(), 3);
    assert!(
        identities
            .iter()
            .all(|identity| identity.status == ProjectIdentityStatus::PendingBinding)
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_065_issue_bridget_active_ou_collision_est_durable_par_commande() {
    let path = std::env::temp_dir().join(format!(
        "maicie-project-registration-resolution-{}.db",
        uuid::Uuid::new_v4()
    ));
    let mut store = MaicieStore::open(&path).unwrap();
    let winner = registration(
        "project-command-winner",
        "project-winner",
        "/srv/projects/a",
    );
    store.prepare_project_registration(&winner).unwrap();
    let active = active_outcome(&winner.command_id, &winner.proposed_project_id);

    let resolved = store.resolve_project_registration(&active).unwrap();
    assert_eq!(resolved.state, ProjectRegistrationState::Bound);
    assert_eq!(
        resolved.resolved_project_id.as_deref(),
        Some("project-winner")
    );
    assert_eq!(resolved.identity.status, ProjectIdentityStatus::Active);
    assert_eq!(resolved.outcome.as_ref(), Some(&active));
    assert!(!resolved.outbox_pending);
    assert_eq!(
        store.resolve_project_registration(&active).unwrap(),
        resolved
    );
    drop(store);

    let mut store = MaicieStore::open(&path).unwrap();
    assert_eq!(
        store
            .project_registration("project-command-winner")
            .unwrap(),
        Some(resolved)
    );

    let losing = registration("project-command-loser", "project-loser", "/srv/projects/a");
    store.prepare_project_registration(&losing).unwrap();
    let collision = ProjectBindOutcome {
        contract_version: 1,
        command_id: losing.command_id.clone(),
        project_id: losing.proposed_project_id.clone(),
        status: ProjectBindStatus::RegistrationConflict,
        binding_generation: None,
        backend: None,
        reason: Some(ProjectRegistryRefusal::RootAlreadyBound),
        existing_project_id: Some("project-external".to_string()),
        existing_binding_generation: Some(4),
        observed_at: 1_788_000_020,
    };
    let conflicted = store.resolve_project_registration(&collision).unwrap();
    assert_eq!(conflicted.state, ProjectRegistrationState::Failed);
    assert_eq!(
        conflicted.identity.status,
        ProjectIdentityStatus::RegistrationConflict
    );
    assert_eq!(
        conflicted.resolved_project_id.as_deref(),
        Some("project-external")
    );
    assert_eq!(conflicted.outcome.as_ref(), Some(&collision));
    assert!(!conflicted.outbox_pending);

    let divergent = ProjectBindOutcome {
        observed_at: collision.observed_at + 1,
        ..collision.clone()
    };
    assert!(matches!(
        store.resolve_project_registration(&divergent),
        Err(maicie::store::StoreError::Conflict(_))
    ));

    let refused = registration(
        "project-command-refused",
        "project-refused",
        "/srv/projects/b",
    );
    store.prepare_project_registration(&refused).unwrap();
    let binding_failed = ProjectBindOutcome {
        contract_version: 1,
        command_id: refused.command_id.clone(),
        project_id: refused.proposed_project_id.clone(),
        status: ProjectBindStatus::BindingFailed,
        binding_generation: None,
        backend: None,
        reason: Some(ProjectRegistryRefusal::RootMissing),
        existing_project_id: None,
        existing_binding_generation: None,
        observed_at: 1_788_000_030,
    };
    let failed = store.resolve_project_registration(&binding_failed).unwrap();
    assert_eq!(failed.state, ProjectRegistrationState::Failed);
    assert_eq!(
        failed.identity.status,
        ProjectIdentityStatus::PendingBinding,
        "un refus Bridget ne doit jamais activer l'identité"
    );
    assert_eq!(failed.outcome.as_ref(), Some(&binding_failed));
    assert!(!failed.outbox_pending);
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_065_commande_locale_prepare_puis_reprend_exactement_la_liaison() {
    let path = std::env::temp_dir().join(format!(
        "maicie-project-command-{}.db",
        uuid::Uuid::new_v4()
    ));
    let mut store = MaicieStore::open(&path).unwrap();
    let request = ProjectRegistrationRequest {
        command_id: "project-command-local".to_string(),
        project_id: "project-local".to_string(),
        display_name: "Projet local".to_string(),
        requested_root: "/srv/projects/local".to_string(),
        issued_at: 1_788_000_000,
        deadline_at: 1_788_000_300,
    };
    let prepared = prepare_project_registration(&mut store, &request).unwrap();
    let wire: ProjectBindRequest = serde_json::from_slice(&prepared.canonical_request).unwrap();
    assert_eq!(wire.command_id, request.command_id);
    assert_eq!(wire.project_id, request.project_id);
    assert_eq!(wire.backend, ProjectBackend::Host);
    assert_eq!(
        project_registration_request_bytes(&store, &request.command_id).unwrap(),
        Some(prepared.canonical_request.clone()),
        "la reprise lit les mêmes octets persistés"
    );
    let resolved = resolve_project_registration(
        &mut store,
        &active_outcome(&request.command_id, &request.project_id),
    )
    .unwrap();
    assert_eq!(resolved.state, ProjectRegistrationState::Bound);
    assert_eq!(resolved.identity.status, ProjectIdentityStatus::Active);
    drop(store);
    let _ = std::fs::remove_file(path);
}
