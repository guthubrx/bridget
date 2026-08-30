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
