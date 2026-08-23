use maicie::bridget_client::SpawnOutcome;
use maicie::domain::{
    ApprobationActivation, DecisionCoordination, EtatActivationOutbox, EtatDecision, TypeDecision,
};
use maicie::outbox::StoreCommitPhase;
use maicie::store::{ActivationApprovalRequest, MaicieStore, StoreError};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

const CHILD_MODE: &str = "MAICIE_T007_CHILD_MODE";
const CHILD_ROOT: &str = "MAICIE_T007_CHILD_ROOT";
const BARRIER: &str = "MAICIE_T007_BARRIER";
const SPAWN_BYTES: &[u8] = br#"{"type":"SpawnOrder","agent_type":"codex","name":null,"cwd":"/tmp/maicie","persistent":false,"command_id":"33333333-3333-4333-8333-333333333333","issued_at":10,"deadline_at":100}"#;
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

#[test]
fn approbation_et_activation_sont_atomiques_et_les_octets_restent_immuables() {
    let root = unique_root("atomique");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let (decision, approval) = proposal();
    store
        .create_activation_proposal(&decision, &approval)
        .unwrap();

    assert!(matches!(
        store.approve_activation(approval.id, &request_at(100, &[7], &[9], SPAWN_BYTES)),
        Err(StoreError::Domain(_))
    ));
    assert!(store.pending_activation_outboxes().unwrap().is_empty());

    for divergent in [
        String::from_utf8(SPAWN_BYTES.to_vec()).unwrap().replace(
            "33333333-3333-4333-8333-333333333333",
            "55555555-5555-4555-8555-555555555555",
        ),
        String::from_utf8(SPAWN_BYTES.to_vec())
            .unwrap()
            .replace("\"agent_type\":\"codex\"", "\"agent_type\":\"claude\""),
    ] {
        assert!(matches!(
            store.approve_activation(
                approval.id,
                &request_at(10, &[7], &[9], divergent.as_bytes())
            ),
            Err(StoreError::Invalid(_))
        ));
        assert!(store.pending_activation_outboxes().unwrap().is_empty());
    }

    assert!(matches!(
        store.approve_activation(approval.id, &request_at(10, &[8], &[9], SPAWN_BYTES)),
        Err(StoreError::Domain(_))
    ));
    assert!(store.pending_activation_outboxes().unwrap().is_empty());

    let activation = store
        .approve_activation(approval.id, &request_at(10, &[7], &[9], SPAWN_BYTES))
        .unwrap();
    assert_eq!(activation.command_id, approval.command_id);
    assert_eq!(activation.spawn_order_bytes, SPAWN_BYTES);
    assert_eq!(activation.etat, EtatActivationOutbox::Dispatching);

    let mut mutable_input = SPAWN_BYTES.to_vec();
    mutable_input[1] = b'X';
    let result = store.approve_activation(approval.id, &request_at(10, &[7], &[9], &mutable_input));
    assert!(
        matches!(
            &result,
            Err(StoreError::Invalid(_)) | Err(StoreError::Json(_)) | Err(StoreError::Conflict(_))
        ),
        "{result:?}"
    );
    let pending = store.pending_activation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].activation.spawn_order_bytes, SPAWN_BYTES);
    assert_eq!(pending[0].approval.consumed_at, None);

    assert!(matches!(
        store.record_activation_outcome(
            approval.command_id,
            &SpawnOutcome::Accepted {
                command_id: "55555555-5555-4555-8555-555555555555".to_string(),
                name: "codex-1".to_string(),
            },
            20,
        ),
        Err(StoreError::Invalid(_))
    ));
    assert_eq!(store.pending_activation_outboxes().unwrap().len(), 1);

    assert!(matches!(
        store.record_activation_outcome(
            approval.command_id,
            &SpawnOutcome::Idempotency(maicie::bridget_client::IdempotencyIssue::OutcomeUnknown {
                expires_at: 30,
                delivery_id: None
            }),
            20,
        ),
        Ok(())
    ));
    assert_eq!(store.pending_activation_outboxes().unwrap().len(), 1);
    store
        .record_activation_outcome(
            approval.command_id,
            &SpawnOutcome::Accepted {
                command_id: approval.command_id.to_string(),
                name: "codex-1".to_string(),
            },
            21,
        )
        .unwrap();
    store
        .record_activation_outcome(
            approval.command_id,
            &SpawnOutcome::Accepted {
                command_id: approval.command_id.to_string(),
                name: "codex-1".to_string(),
            },
            22,
        )
        .unwrap();
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    let (state, consumed_at, bytes): (String, i64, Vec<u8>) = connection
        .query_row(
            "SELECT a.state, a.consumed_at, o.spawn_order_bytes\n\
             FROM activation_approvals a\n\
             JOIN activation_outbox o ON o.approval_id = a.id\n\
             WHERE a.id = ?1",
            [approval.id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(state, "consumed");
    assert_eq!(consumed_at, 21);
    assert_eq!(bytes, SPAWN_BYTES);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relations_corrompues_refusent_les_deux_transitions_activation() {
    let root = unique_root("relations-corrompues");
    let database = root.join("maicie.sqlite3");
    let (decision, approval) = proposal();
    let mut store = MaicieStore::open(&database).unwrap();
    store
        .create_activation_proposal(&decision, &approval)
        .unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE activation_approvals SET objective_id = '00000000-0000-4000-8000-000000000000'\n\
             WHERE id = ?1",
            [approval.id.to_string()],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    assert!(matches!(
        store.approve_activation(approval.id, &request_at(10, &[7], &[9], SPAWN_BYTES)),
        Err(StoreError::Corrupt(_))
    ));
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE activation_approvals\n\
             SET objective_id = ?1\n\
             WHERE id = ?2",
            [approval.objective_id.to_string(), approval.id.to_string()],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    store
        .approve_activation(approval.id, &request_at(10, &[7], &[9], SPAWN_BYTES))
        .unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = OFF;")
        .unwrap();
    connection
        .execute(
            "UPDATE activation_approvals\n\
             SET command_id = '55555555-5555-4555-8555-555555555555'\n\
             WHERE id = ?1",
            [approval.id.to_string()],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    assert!(matches!(
        store.record_activation_outcome(
            approval.command_id,
            &SpawnOutcome::Accepted {
                command_id: approval.command_id.to_string(),
                name: "codex-1".to_string(),
            },
            20,
        ),
        Err(StoreError::Corrupt(_))
    ));
    assert_eq!(store.pending_activation_outboxes().unwrap().len(), 1);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn crash_reel_autour_de_l_approbation_ne_publie_jamais_un_etat_partiel() {
    for (phase, committed) in [("before_commit", false), ("after_commit", true)] {
        let root = unique_root(phase);
        let (mut child, mut barrier, socket_path) = spawn_child(phase, &root);
        wait_barrier(&mut barrier, phase);
        child.kill().unwrap();
        child.wait().unwrap();
        fs::remove_file(socket_path).unwrap();

        let store = MaicieStore::open(root.join("maicie.sqlite3")).unwrap();
        let pending = store.pending_activation_outboxes().unwrap();
        assert_eq!(pending.len(), usize::from(committed), "frontière {phase}");
        if committed {
            assert_eq!(pending[0].activation.command_id, command_id());
            assert_eq!(pending[0].activation.spawn_order_bytes, SPAWN_BYTES);
            assert_eq!(pending[0].approval.consumed_at, None);
        }
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn approval_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).unwrap());
    let mut store = MaicieStore::open(root.join("maicie.sqlite3")).unwrap();
    let (decision, approval) = proposal();
    store
        .create_activation_proposal(&decision, &approval)
        .unwrap();
    let expected = match mode.as_str() {
        "before_commit" => StoreCommitPhase::BeforeCommit,
        "after_commit" => StoreCommitPhase::AfterCommit,
        other => panic!("mode enfant inconnu: {other}"),
    };
    store
        .approve_activation_observed(
            approval.id,
            &request_at(10, &[7], &[9], SPAWN_BYTES),
            |phase| {
                if phase == expected {
                    block_at_barrier(&mode);
                }
                Ok(())
            },
        )
        .unwrap();
}

fn proposal() -> (DecisionCoordination, ApprobationActivation) {
    let objective_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
    let decision = DecisionCoordination {
        id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
        objectif_id: objective_id,
        kind: TypeDecision::ReveillerProfil,
        proposee_par: "maicie".to_string(),
        etat: EtatDecision::Proposee,
        motif: "profil requis par une décision explicite".to_string(),
    };
    let approval = ApprobationActivation {
        id: Uuid::parse_str("44444444-4444-4444-8444-444444444444").unwrap(),
        command_id: command_id(),
        objective_id,
        profile_id: "codex".to_string(),
        profile_hash: vec![7],
        context_hash: vec![9],
        context_scope: "objectif:11111111-1111-4111-8111-111111111111".to_string(),
        parameters: String::from_utf8(SPAWN_BYTES.to_vec()).unwrap(),
        actor: "local_human".to_string(),
        expires_at: 100,
        consumed_at: None,
    };
    (decision, approval)
}

fn command_id() -> Uuid {
    Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap()
}

fn request_at<'a>(
    now: i64,
    profile_hash: &'a [u8],
    context_hash: &'a [u8],
    spawn_order_bytes: &'a [u8],
) -> ActivationApprovalRequest<'a> {
    ActivationApprovalRequest {
        now,
        profile_hash,
        context_hash,
        spawn_order_bytes,
        retry_until: 150,
        dedup_retained_until: 200,
    }
}

fn spawn_child(mode: &str, root: &Path) -> (Child, UnixStream, PathBuf) {
    let socket_path = root.with_extension("barrier.sock");
    let _ = fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).unwrap();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || sender.send(listener.accept().map(|pair| pair.0)).unwrap());
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("approval_atomicity::approval_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_ROOT, root)
        .env(BARRIER, &socket_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let stream = match receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(stream)) => stream,
        other => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("connexion à la barrière impossible : {other:?}");
        }
    };
    (child, stream, socket_path)
}

fn wait_barrier(stream: &mut UnixStream, expected: &str) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).unwrap();
    assert_eq!(line.trim(), format!("BARRIER:{expected}"));
}

fn block_at_barrier(name: &str) {
    let socket_path = std::env::var_os(BARRIER).unwrap();
    let mut socket = UnixStream::connect(socket_path).unwrap();
    socket
        .write_all(format!("BARRIER:{name}\n").as_bytes())
        .unwrap();
    socket.flush().unwrap();
    let mut byte = [0_u8; 1];
    socket.read_exact(&mut byte).unwrap();
}

fn unique_root(label: &str) -> PathBuf {
    let suffix = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(format!(
        "/tmp/maicie-t007-{}-{suffix}-{label}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}
