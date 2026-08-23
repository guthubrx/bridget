use maicie::bridget_client::IdempotencyIssue;
use maicie::domain::{
    ClasseDuree, Delegation, EtatDelegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif,
    ObjectifCoordonne, OutboxDelegation,
};
use maicie::outbox::{PreparedDelegation, StoreCommitPhase, stable_body_hash};
use maicie::store::MaicieStore;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

const OBJECTIVE_ID: &str = "10000000-0000-4000-8000-000000000001";
const DELEGATION_ID: &str = "20000000-0000-4000-8000-000000000002";
const MESSAGE_ID: &str = "30000000-0000-4000-8000-000000000003";
const BODY: &[u8] = b"Inspecte le chemin critique, puis reponds avec les preuves.";
const CHILD_MODE: &str = "MAICIE_T006_CHILD_MODE";
const CHILD_ROOT: &str = "MAICIE_T006_CHILD_ROOT";

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

#[test]
fn migrations_idempotentes_et_base_privee() {
    let root = unique_root("migration");
    let database = root.join("maicie.sqlite3");
    let first_scope = {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), 1);
        store.issuer_scope().to_string()
    };
    let reopened = MaicieStore::open(&database).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 1);
    assert_eq!(reopened.issuer_scope(), first_scope);
    assert_eq!(mode(&root), 0o700);
    assert_eq!(mode(&database), 0o600);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn schema_futur_et_enveloppe_corrompue_sont_refuses_fail_closed() {
    let future_root = unique_root("future-schema");
    let future_database = future_root.join("maicie.sqlite3");
    drop(MaicieStore::open(&future_database).unwrap());
    let connection = rusqlite::Connection::open(&future_database).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    assert!(MaicieStore::open(&future_database).is_err());
    fs::remove_dir_all(future_root).unwrap();

    let corrupt_root = unique_root("corrupt-envelope");
    let corrupt_database = corrupt_root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&corrupt_database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();
    drop(store);
    let connection = rusqlite::Connection::open(&corrupt_database).unwrap();
    connection
        .execute(
            "UPDATE delegation_outbox SET body_bytes = ?1 WHERE message_id = ?2",
            rusqlite::params![b"corrompu".as_slice(), MESSAGE_ID],
        )
        .unwrap();
    drop(connection);
    let store = MaicieStore::open(&corrupt_database).unwrap();
    assert!(store.pending_delegation_outboxes().is_err());
    assert!(store.recovery_snapshot(uuid(MESSAGE_ID)).is_err());
    fs::remove_dir_all(corrupt_root).unwrap();
}

#[test]
fn transaction_unique_expose_l_enveloppe_exacte_et_le_snapshot() {
    let root = unique_root("exact");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].message_id, uuid(MESSAGE_ID));
    assert_eq!(pending[0].issuer_scope, store.issuer_scope());
    assert_eq!(pending[0].issued_at, 1_000);
    assert_eq!(pending[0].body_bytes, BODY);
    assert_eq!(pending[0].message_bytes, prepared.message_bytes);
    assert_eq!(pending[0].public_message().unwrap().body.as_bytes(), BODY);

    let snapshot = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
    assert_eq!(snapshot.objective_state, EtatObjectif::EnCoordination);
    assert_eq!(snapshot.delegation_state, EtatDelegation::Creee);
    assert_eq!(snapshot.outbox, pending[0]);
    assert!(snapshot.last_issue.is_none());

    let duplicate = store.create_prepared_delegation(&prepared);
    assert!(duplicate.is_err());
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);

    let mut second_delegation = prepared.delegation.clone();
    second_delegation.id = uuid("40000000-0000-4000-8000-000000000004");
    let mut second_outbox = prepared.outbox.clone();
    second_outbox.message_id = uuid("50000000-0000-4000-8000-000000000005");
    second_outbox.delegation_id = second_delegation.id;
    let second = PreparedDelegation::new(
        prepared.objective.clone(),
        second_delegation,
        second_outbox,
        store.issuer_scope(),
        1_000,
    )
    .unwrap();
    store.create_prepared_delegation(&second).unwrap();
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issue_et_incertitude_sont_des_transitions_transactionnelles() {
    let root = unique_root("issues");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    store
        .record_transport_uncertainty(uuid(MESSAGE_ID), 1_010)
        .unwrap();
    let uncertain = store.pending_delegation_outboxes().unwrap();
    assert_eq!(uncertain[0].state, EtatOutboxDelegation::OutcomeUnknown);
    assert_eq!(uncertain[0].attempted_at, Some(1_010));

    store
        .record_lookup_issue(
            uuid(MESSAGE_ID),
            &IdempotencyIssue::Accepted { expires_at: 1_100 },
            1_011,
        )
        .unwrap();
    store
        .record_lookup_issue(
            uuid(MESSAGE_ID),
            &IdempotencyIssue::Accepted { expires_at: 1_100 },
            1_012,
        )
        .unwrap();
    assert!(
        store
            .record_lookup_issue(
                uuid(MESSAGE_ID),
                &IdempotencyIssue::IdempotencyExpired,
                1_013,
            )
            .is_err()
    );
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    let terminal = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
    assert_eq!(terminal.outbox.state, EtatOutboxDelegation::Accepted);
    assert_eq!(terminal.last_issue.unwrap()["kind"], "accepted");
    assert_eq!(terminal.issue_observed_at, Some(1_011));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn crash_reel_avant_et_apres_commit_respecte_l_atomicite() {
    for (phase, committed) in [("before_commit", false), ("after_commit", true)] {
        let root = unique_root(phase);
        let (mut child, mut barrier, socket_path) = spawn_child(phase, &root);
        wait_barrier(&mut barrier, phase);
        child.kill().unwrap();
        child.wait().unwrap();
        fs::remove_file(socket_path).unwrap();

        let store = MaicieStore::open(root.join("maicie.sqlite3")).unwrap();
        let snapshot = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap();
        assert_eq!(snapshot.is_some(), committed, "frontière {phase}");
        if let Some(snapshot) = snapshot {
            assert_eq!(
                snapshot.outbox.message_bytes,
                fixture(store.issuer_scope()).message_bytes
            );
            assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Prepared);
        }
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn crash_reel_avant_et_apres_handoff_conserve_un_replay_exact() {
    for phase in ["before_send", "after_send"] {
        let root = unique_root(phase);
        let database = root.join("maicie.sqlite3");
        let expected = {
            let mut store = MaicieStore::open(&database).unwrap();
            let prepared = fixture(store.issuer_scope());
            store.create_prepared_delegation(&prepared).unwrap();
            prepared.message_bytes
        };

        let (mut child, mut barrier, socket_path) = spawn_child(phase, &root);
        wait_barrier(&mut barrier, phase);
        child.kill().unwrap();
        child.wait().unwrap();
        fs::remove_file(socket_path).unwrap();

        let ledger = root.join("transport-ledger.bin");
        if phase == "before_send" {
            assert!(!ledger.exists());
        } else {
            assert_eq!(fs::read(&ledger).unwrap(), expected);
        }
        let mut store = MaicieStore::open(&database).unwrap();
        let pending = store.pending_delegation_outboxes().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].message_bytes, expected);
        assert_eq!(pending[0].message_id, uuid(MESSAGE_ID));

        if phase == "after_send" {
            store
                .record_transport_uncertainty(uuid(MESSAGE_ID), 1_020)
                .unwrap();
            store
                .record_lookup_issue(
                    uuid(MESSAGE_ID),
                    &IdempotencyIssue::Accepted { expires_at: 1_100 },
                    1_021,
                )
                .unwrap();
            assert!(store.pending_delegation_outboxes().unwrap().is_empty());
            assert_eq!(fs::read(&ledger).unwrap(), expected);
        }
        fs::remove_dir_all(root).unwrap();
    }
}

/// Sous-processus du crash-test. Sans variables dédiées, le test est neutre.
#[test]
fn crash_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).unwrap());
    let database = root.join("maicie.sqlite3");
    match mode.as_str() {
        "before_commit" | "after_commit" => {
            let mut store = MaicieStore::open(&database).unwrap();
            let prepared = fixture(store.issuer_scope());
            let barrier = if mode == "before_commit" {
                StoreCommitPhase::BeforeCommit
            } else {
                StoreCommitPhase::AfterCommit
            };
            store
                .create_prepared_delegation_observed(&prepared, |phase| {
                    if phase == barrier {
                        block_at_barrier(&mode);
                    }
                    Ok(())
                })
                .unwrap();
        }
        "before_send" | "after_send" => {
            let store = MaicieStore::open(&database).unwrap();
            let pending = store.pending_delegation_outboxes().unwrap();
            assert_eq!(pending.len(), 1);
            if mode == "before_send" {
                block_at_barrier(&mode);
            }
            let ledger = root.join("transport-ledger.bin");
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(ledger)
                .unwrap();
            file.write_all(&pending[0].message_bytes).unwrap();
            file.sync_all().unwrap();
            block_at_barrier(&mode);
        }
        other => panic!("mode enfant inconnu: {other}"),
    }
}

fn fixture(issuer_scope: &str) -> PreparedDelegation {
    let objective = ObjectifCoordonne {
        id: uuid(OBJECTIVE_ID),
        but: "Auditer le chemin critique".to_string(),
        mode: ModeObjectif::Delegue,
        etat: EtatObjectif::EnCoordination,
        cree_at: 1_000,
        mis_a_jour_at: 1_000,
        synthese: None,
        decision_en_attente_id: None,
    };
    let delegation = Delegation {
        id: uuid(DELEGATION_ID),
        objectif_id: objective.id,
        participant: "prospective".to_string(),
        instruction: String::from_utf8(BODY.to_vec()).unwrap(),
        duree: ClasseDuree::Normale,
        etat: EtatDelegation::Creee,
        raison: "expertise déclarée".to_string(),
    };
    let outbox = OutboxDelegation {
        message_id: uuid(MESSAGE_ID),
        delegation_id: delegation.id,
        target: delegation.participant.clone(),
        body_bytes: BODY.to_vec(),
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_060,
        body_hash: stable_body_hash(BODY),
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_050,
        dedup_retained_until: 1_100,
    };
    PreparedDelegation::new(objective, delegation, outbox, issuer_scope, 1_000).unwrap()
}

fn spawn_child(mode: &str, root: &Path) -> (Child, UnixStream, PathBuf) {
    let socket_path = root.with_extension("barrier.sock");
    let _ = fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).unwrap();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        sender.send(listener.accept().map(|pair| pair.0)).unwrap();
    });
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("store_outbox::crash_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_ROOT, root)
        .env("MAICIE_T006_BARRIER", &socket_path)
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
    let socket_path = std::env::var_os("MAICIE_T006_BARRIER").unwrap();
    let mut socket = UnixStream::connect(socket_path).unwrap();
    socket
        .write_all(format!("BARRIER:{name}\n").as_bytes())
        .unwrap();
    socket.flush().unwrap();
    let mut byte = [0_u8; 1];
    socket.read_exact(&mut byte).unwrap();
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn unique_root(label: &str) -> PathBuf {
    let suffix = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(format!(
        "/tmp/maicie-t006-{}-{suffix}-{label}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}
