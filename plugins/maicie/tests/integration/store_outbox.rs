use maicie::bridget_client::{IdempotencyIssue, PublicMessage};
use maicie::domain::{
    ClasseDuree, Delegation, EtatDelegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif,
    ObjectifCoordonne, OutboxDelegation,
};
use maicie::outbox::{stable_body_hash, PreparedDelegation, StoreCommitPhase, MAX_MESSAGE_BYTES};
use maicie::store::{DelegationRecoveryEntry, LocalFailureReason, MaicieStore};
use serde_json::json;
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
const FRAME_LIMIT: usize = 256 * 1024;
const CHILD_MODE: &str = "MAICIE_T006_CHILD_MODE";
const CHILD_ROOT: &str = "MAICIE_T006_CHILD_ROOT";

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

#[test]
fn migrations_idempotentes_et_base_privee() {
    let root = unique_root("migration");
    let database = root.join("maicie.sqlite3");
    let first_scope = {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), 6);
        store.issuer_scope().to_string()
    };
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 2", [])
        .unwrap();
    drop(connection);
    let reopened = MaicieStore::open(&database).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 6);
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
    connection.pragma_update(None, "user_version", 7).unwrap();
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
fn enveloppe_locale_corrompue_devient_un_rejet_terminal_et_n_est_jamais_reprise() {
    let root = unique_root("local-failure");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    let objective_id = prepared.objective.id;
    let delegation_id = prepared.delegation.id;
    store.create_prepared_delegation(&prepared).unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE delegation_outbox SET body_bytes = ?1 WHERE message_id = ?2",
            rusqlite::params![b"corrompu".as_slice(), MESSAGE_ID],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    let entries = store.delegation_recovery_entries().unwrap();
    assert_eq!(
        entries,
        vec![DelegationRecoveryEntry::LocalFailure {
            objective_id,
            delegation_id,
            message_id: uuid(MESSAGE_ID),
            reason: LocalFailureReason::InvalidEnvelope,
        }]
    );
    store
        .record_local_failure_at(
            uuid(MESSAGE_ID),
            LocalFailureReason::InvalidEnvelope,
            1_010,
        )
        .unwrap();
    store
        .record_local_failure_at(
            uuid(MESSAGE_ID),
            LocalFailureReason::InvalidEnvelope,
            1_010,
        )
        .unwrap();
    assert!(store.delegation_recovery_entries().unwrap().is_empty());
    let snapshots = store.objective_snapshots(Some(objective_id)).unwrap();
    assert_eq!(snapshots[0].objective.etat, EtatObjectif::AEvaluer);
    assert_eq!(snapshots[0].delegations[0].etat, EtatDelegation::AEvaluer);
    assert_eq!(snapshots[0].decisions.len(), 1);
    assert_eq!(snapshots[0].decisions[0].id, uuid(MESSAGE_ID));
    assert_eq!(
        snapshots[0].decisions[0].kind,
        maicie::domain::TypeDecision::ConstaterIssue
    );
    assert_eq!(
        snapshots[0].decisions[0].etat,
        maicie::domain::EtatDecision::Appliquee
    );
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    let stored: (String, i64, Vec<u8>, String, String) = connection
        .query_row(
            "SELECT state, terminal, last_issue_json, objective_id, delegation_id\n\
             FROM delegation_outbox WHERE message_id = ?1",
            [MESSAGE_ID],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(stored.0, "rejected");
    assert_eq!(stored.1, 1);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&stored.2).unwrap(),
        json!({"local":"invalid_envelope"})
    );
    assert_eq!(stored.3, objective_id.to_string());
    assert_eq!(stored.4, delegation_id.to_string());
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_v1_convertit_un_refus_terminal_historique_en_rejected() {
    let root = unique_root("migration-rejected");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE delegation_outbox\n\
             SET state = 'outcome_unknown', terminal = 1,\n\
                 last_issue_json = ?1, issue_observed_at = 1010\n\
             WHERE message_id = ?2",
            rusqlite::params![br#"{"kind":"invalid_issued_at"}"#.as_slice(), MESSAGE_ID],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 2", [])
        .unwrap();
    drop(connection);

    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 6);
    let snapshot = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
    assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Rejected);
    assert_eq!(snapshot.last_issue.unwrap()["kind"], "invalid_issued_at");
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_v2_vers_v6_conserve_les_donnees_historiques_et_cree_les_tables_requises() {
    let root = unique_root("migration-activation");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let prepared = fixture(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute("DROP INDEX activation_outbox_pending_idx", [])
        .unwrap();
    connection
        .execute("DROP TABLE activation_outbox", [])
        .unwrap();
    connection
        .execute("DROP TABLE activation_approvals", [])
        .unwrap();
    connection
        .execute("DROP TABLE coordination_decisions", [])
        .unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 3", [])
        .unwrap();
    drop(connection);

    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 6);
    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].message_bytes, prepared.message_bytes);
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    let required_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master\n\
             WHERE type = 'table'\n\
               AND name IN ('coordination_decisions', 'activation_approvals', 'activation_outbox', 'delegate_idempotency', 'conversation_records')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(required_tables, 5);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_v3_vers_v6_ajoute_les_preuves_et_la_reservation_delegate() {
    let root = unique_root("migration-activation-issue");
    let database = root.join("maicie.sqlite3");
    drop(MaicieStore::open(&database).unwrap());

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute("DROP INDEX activation_outbox_pending_idx", [])
        .unwrap();
    connection
        .execute("DROP TABLE activation_outbox", [])
        .unwrap();
    connection
        .execute_batch(
            "CREATE TABLE activation_outbox (
                 command_id TEXT PRIMARY KEY REFERENCES activation_approvals(command_id),
                 approval_id TEXT NOT NULL UNIQUE REFERENCES activation_approvals(id),
                 spawn_order_bytes BLOB NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('dispatching','outcome_unknown','applied')),
                 retry_until INTEGER NOT NULL,
                 dedup_retained_until INTEGER NOT NULL,
                 terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))
             );
             CREATE INDEX activation_outbox_pending_idx
                 ON activation_outbox(terminal, state, retry_until, command_id);",
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 3).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 4", [])
        .unwrap();
    drop(connection);

    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 6);
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    let columns: Vec<String> = connection
        .prepare("PRAGMA table_info(activation_outbox)")
        .unwrap()
        .query_map([], |row| row.get(1))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(columns.contains(&"last_issue_json".to_string()));
    assert!(columns.contains(&"issue_observed_at".to_string()));
    let delegate_table: String = connection
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'delegate_idempotency'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(delegate_table, "delegate_idempotency");
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn prepare_refuse_une_trame_finale_poison_et_des_champs_hors_contrat() {
    let root = unique_root("wire-bound");
    let database = root.join("maicie.sqlite3");
    let store = MaicieStore::open(&database).unwrap();
    let base = fixture(store.issuer_scope());

    let empty_message = PublicMessage {
        id: base.outbox.message_id.to_string(),
        from: maicie::MAICIE_IDENTITY.to_string(),
        to: base.outbox.target.clone(),
        body: String::new(),
        reply: base.outbox.reply,
        hops: 4,
        reply_timeout: Some(base.outbox.timeout_secs),
        deadline_at: u64::try_from(base.outbox.deadline_contractuelle).ok(),
        in_reply_to: None,
    };
    let replay_overhead = serde_json::to_vec(&json!({
        "type": "SendIdempotent",
        "message": empty_message,
        "message_id": base.outbox.message_id,
        "issued_at": base.issued_at,
    }))
    .unwrap()
    .len();
    let body = vec![b'x'; MAX_MESSAGE_BYTES - replay_overhead - 1];
    let mut at_limit = base.outbox.clone();
    at_limit.body_bytes = body.clone();
    at_limit.body_hash = stable_body_hash(&body);
    assert!(PreparedDelegation::new(
        base.objective.clone(),
        base.delegation.clone(),
        at_limit,
        store.issuer_scope(),
        base.issued_at,
        FRAME_LIMIT,
    )
    .is_ok());

    let mut body_over_limit = body;
    body_over_limit.push(b'x');
    let mut oversized = base.outbox.clone();
    oversized.body_bytes = body_over_limit.clone();
    oversized.body_hash = stable_body_hash(&body_over_limit);
    assert!(PreparedDelegation::new(
        base.objective.clone(),
        base.delegation.clone(),
        oversized,
        store.issuer_scope(),
        base.issued_at,
        FRAME_LIMIT,
    )
    .is_err());

    let base_message: PublicMessage = serde_json::from_slice(&base.message_bytes).unwrap();
    let runtime_frame_bytes = serde_json::to_vec(&json!({
        "type": "SendIdempotent",
        "message": base_message,
        "message_id": base.outbox.message_id,
        "issued_at": base.issued_at,
    }))
    .unwrap()
    .len()
        + 1;
    assert!(PreparedDelegation::new(
        base.objective.clone(),
        base.delegation.clone(),
        base.outbox.clone(),
        store.issuer_scope(),
        base.issued_at,
        runtime_frame_bytes - 1,
    )
    .is_err());

    let mut timeout = base.outbox.clone();
    timeout.timeout_secs = 7 * 24 * 60 * 60 + 1;
    assert!(PreparedDelegation::new(
        base.objective.clone(),
        base.delegation.clone(),
        timeout,
        store.issuer_scope(),
        base.issued_at,
        FRAME_LIMIT,
    )
    .is_err());

    let mut wrong_hops = base.clone();
    let mut message: PublicMessage = serde_json::from_slice(&wrong_hops.message_bytes).unwrap();
    message.hops = 3;
    wrong_hops.message_bytes = serde_json::to_vec(&message).unwrap();
    assert!(wrong_hops.validate().is_err());
    fs::remove_dir_all(root).unwrap();
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
    let public_message = pending[0].public_message().unwrap();
    assert_eq!(public_message.body.as_bytes(), BODY);
    assert_eq!(public_message.from, maicie::MAICIE_IDENTITY);
    assert_ne!(pending[0].issuer_scope, public_message.from);

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
        FRAME_LIMIT,
    )
    .unwrap();
    store.create_prepared_delegation(&second).unwrap();
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn un_objectif_clos_ne_peut_pas_etre_rouvert_par_un_nouvel_upsert() {
    let root = unique_root("closed-objective");
    let database = root.join("maicie.sqlite3");
    let (prepared, scope) = {
        let mut store = MaicieStore::open(&database).unwrap();
        let prepared = fixture(store.issuer_scope());
        store.create_prepared_delegation(&prepared).unwrap();
        (prepared, store.issuer_scope().to_string())
    };

    let mut closed = prepared.objective.clone();
    closed.clore(1_001).unwrap();
    let closed_json = serde_json::to_vec(&closed).unwrap();
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE objectives SET state = 'clos', payload_json = ?1 WHERE id = ?2",
            rusqlite::params![closed_json, closed.id.to_string()],
        )
        .unwrap();
    drop(connection);

    let mut second_delegation = prepared.delegation.clone();
    second_delegation.id = uuid("40000000-0000-4000-8000-000000000004");
    let mut second_outbox = prepared.outbox.clone();
    second_outbox.message_id = uuid("50000000-0000-4000-8000-000000000005");
    second_outbox.delegation_id = second_delegation.id;
    let second = PreparedDelegation::new(
        prepared.objective,
        second_delegation,
        second_outbox,
        &scope,
        1_000,
        FRAME_LIMIT,
    )
    .unwrap();
    let mut store = MaicieStore::open(&database).unwrap();
    assert!(store.create_prepared_delegation(&second).is_err());
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    let stored: (String, Vec<u8>) = connection
        .query_row(
            "SELECT state, payload_json FROM objectives WHERE id = ?1",
            [closed.id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(stored.0, "clos");
    assert_eq!(
        serde_json::from_slice::<ObjectifCoordonne>(&stored.1).unwrap(),
        closed
    );
    let outbox_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM delegation_outbox", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(outbox_count, 1);
    drop(connection);
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
    assert!(store
        .record_lookup_issue(
            uuid(MESSAGE_ID),
            &IdempotencyIssue::IdempotencyExpired,
            1_013,
        )
        .is_err());
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    let terminal = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
    assert_eq!(terminal.outbox.state, EtatOutboxDelegation::Accepted);
    assert_eq!(terminal.last_issue.unwrap()["kind"], "accepted");
    assert_eq!(terminal.issue_observed_at, Some(1_011));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tous_les_refus_durables_convergent_vers_rejected_terminal() {
    let cases = [
        (
            IdempotencyIssue::Rejected {
                category: "policy".to_string(),
                reason: "refus explicite".to_string(),
                expires_at: 1_100,
            },
            "rejected",
        ),
        (IdempotencyIssue::EnvelopeMismatch, "envelope_mismatch"),
        (IdempotencyIssue::IdempotencyExpired, "idempotency_expired"),
        (IdempotencyIssue::InvalidIssuedAt, "invalid_issued_at"),
    ];
    for (index, (issue, expected_kind)) in cases.into_iter().enumerate() {
        let root = unique_root(&format!("rejected-{index}"));
        let database = root.join("maicie.sqlite3");
        let mut store = MaicieStore::open(&database).unwrap();
        let prepared = fixture(store.issuer_scope());
        store.create_prepared_delegation(&prepared).unwrap();
        store
            .record_lookup_issue(uuid(MESSAGE_ID), &issue, 1_010)
            .unwrap();
        store
            .record_lookup_issue(uuid(MESSAGE_ID), &issue, 1_011)
            .unwrap();

        assert!(store.pending_delegation_outboxes().unwrap().is_empty());
        let snapshot = store.recovery_snapshot(uuid(MESSAGE_ID)).unwrap().unwrap();
        assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Rejected);
        assert_eq!(snapshot.last_issue.unwrap()["kind"], expected_kind);
        assert_eq!(snapshot.issue_observed_at, Some(1_010));
        assert!(store
            .record_transport_uncertainty(uuid(MESSAGE_ID), 1_012)
            .is_err());
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
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
        let replay = store
            .lookup_delegate_replay("crash-reservation", b"delegate-v1")
            .unwrap();
        assert_eq!(replay.is_some(), committed, "réservation {phase}");
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
                .lookup_or_reserve_delegate_observed(
                    "crash-reservation",
                    b"delegate-v1",
                    &prepared,
                    |phase| {
                        if phase == barrier {
                            block_at_barrier(&mode);
                        }
                        Ok(())
                    },
                )
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
    PreparedDelegation::new(
        objective,
        delegation,
        outbox,
        issuer_scope,
        1_000,
        FRAME_LIMIT,
    )
    .unwrap()
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
