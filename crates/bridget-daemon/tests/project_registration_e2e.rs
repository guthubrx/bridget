use bridget_daemon::daemon::{self, DaemonConfig};
use bridget_transport::protocol::ProjectBindStatus;
use maicie::app::{
    ProjectRegistrationRequest, prepare_project_registration, project_registration_request_bytes,
    resolve_project_registration,
};
use maicie::bridget_client::ProjectRegistryClient;
use maicie::domain::ProjectIdentityStatus;
use maicie::store::MaicieStore;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn now() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge Unix")
            .as_secs(),
    )
    .expect("horodatage i64")
}

fn test_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-spec065-project-e2e-{}",
        uuid::Uuid::new_v4().simple()
    ))
}

fn wait_for_socket(socket: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if UnixStream::connect(socket).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("daemon de test absent: {}", socket.display());
}

fn request(
    command_id: &str,
    project_id: &str,
    name: &str,
    root: &Path,
    issued_at: i64,
) -> ProjectRegistrationRequest {
    ProjectRegistrationRequest {
        command_id: command_id.to_string(),
        project_id: project_id.to_string(),
        display_name: name.to_string(),
        requested_root: root.display().to_string(),
        issued_at,
        deadline_at: issued_at + 120,
    }
}

#[test]
fn spec_065_reprise_apres_crash_et_collision_alias_ne_creent_qu_un_actif() {
    let root = test_root();
    std::fs::create_dir_all(&root).expect("racine de test");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
        .expect("permissions racine de test");
    let allowed_root = root.join("allowed");
    let project_root = allowed_root.join("project");
    let alias_root = allowed_root.join("project-alias");
    std::fs::create_dir_all(&project_root).expect("racine projet");
    symlink(&project_root, &alias_root).expect("alias projet");
    let policy_path = root.join("project-policy.json");
    std::fs::write(
        &policy_path,
        format!(
            "{{\"contract_version\":1,\"allowed_project_roots\":[\"{}\"]}}",
            allowed_root.display()
        ),
    )
    .expect("politique projet");
    std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600))
        .expect("permissions politique");

    let socket = root.join("bridget.sock");
    let daemon_config = DaemonConfig {
        socket_path: socket.clone(),
        db_path: root.join("bridget.db"),
        log_path: root.join("bridget.log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
        project_root_policy_path: Some(policy_path),
    };
    thread::spawn(move || {
        let _ = daemon::run(daemon_config);
    });
    wait_for_socket(&socket);

    let maicie_db = root.join("maicie.db");
    let issued_at = now();
    let first_request = request(
        "project-command-crash",
        "project-winner",
        "Projet gagnant",
        &project_root,
        issued_at,
    );
    let mut store = MaicieStore::open(&maicie_db).expect("store Maicie");
    let prepared = prepare_project_registration(&mut store, &first_request).expect("préparation");
    let persisted_bytes = prepared.canonical_request.clone();
    drop(store);

    // Crash après l'écriture Maicie/outbox, avant toute I/O : la reprise
    // retrouve les octets exacts sans produire une seconde intention.
    let store = MaicieStore::open(&maicie_db).expect("reprise store Maicie");
    assert_eq!(
        project_registration_request_bytes(&store, &first_request.command_id).unwrap(),
        Some(persisted_bytes.clone())
    );
    let mut client = ProjectRegistryClient::connect(&socket, store.issuer_scope())
        .expect("connexion registre projet");
    let daemon_outcome = client
        .bind_exact_bytes(&persisted_bytes)
        .expect("liaison Bridget");
    assert_eq!(daemon_outcome.status, ProjectBindStatus::Active);
    drop(client);
    drop(store);

    // Crash après l'écriture durable Bridget, avant que Maicie persiste son
    // accusé. Le même command_id doit relire l'issue, pas créer une liaison.
    let mut store = MaicieStore::open(&maicie_db).expect("store après accusé perdu");
    let replay_bytes = project_registration_request_bytes(&store, &first_request.command_id)
        .unwrap()
        .expect("outbox persistée");
    let mut client = ProjectRegistryClient::connect(&socket, store.issuer_scope())
        .expect("reconnexion registre projet");
    let replay_outcome = client
        .bind_exact_bytes(&replay_bytes)
        .expect("rejeu Bridget");
    assert_eq!(replay_outcome, daemon_outcome);
    let winner =
        resolve_project_registration(&mut store, &replay_outcome).expect("résolution Maicie");
    assert_eq!(winner.identity.status, ProjectIdentityStatus::Active);

    let loser_request = request(
        "project-command-alias",
        "project-loser",
        "Projet perdant",
        &alias_root,
        issued_at,
    );
    let loser =
        prepare_project_registration(&mut store, &loser_request).expect("préparation perdante");
    let collision = client
        .bind_exact_bytes(&loser.canonical_request)
        .expect("collision canonique");
    assert_eq!(collision.status, ProjectBindStatus::RegistrationConflict);
    assert_eq!(
        collision.existing_project_id.as_deref(),
        Some("project-winner")
    );
    let loser = resolve_project_registration(&mut store, &collision).expect("résolution collision");
    assert_eq!(
        loser.identity.status,
        ProjectIdentityStatus::RegistrationConflict
    );
    assert_eq!(
        store
            .project_identities()
            .unwrap()
            .iter()
            .filter(|identity| identity.status == ProjectIdentityStatus::Active)
            .count(),
        1,
        "les alias ne doivent jamais créer une seconde identité active"
    );
}
