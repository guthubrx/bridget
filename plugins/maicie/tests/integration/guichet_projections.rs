use maicie::app::{
    DelegateRequest, DelegateResult, DelegationCandidate, GuichetError, delegate,
    process_deadline_question_claim, process_guichet_claim, process_mission_status_claim,
};
use maicie::bridget_client::{GuichetClaim, IdempotencyIssue};
use maicie::config::DurationClasses;
use maicie::domain::guichet::{ProjectionReply, parse_claim};
use maicie::domain::{ClasseDuree, EtatFlux, SnapshotTransport, SourceSnapshot};
use maicie::store::{MaicieStore, StoreError};
use rusqlite::{Connection, ErrorCode};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "maicie-guichet-projection-{label}-{}",
        Uuid::new_v4()
    ))
}

fn seed(database: &Path) -> maicie::app::DelegationCreated {
    let mut store = MaicieStore::open(database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec!["rust".to_string()],
        available: true,
        dnd: false,
    }];
    let request = DelegateRequest {
        goal: "observer la projection",
        explicit_target: Some("prospective"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: true,
        idempotency_key: "guichet-projection-seed",
        now: 900,
        retry_until: 1_100,
        dedup_retained_until: 1_200,
        max_frame_bytes: 256 * 1024,
    };
    let DelegateResult::Created(created) = delegate(
        &mut store,
        DurationClasses {
            short_secs: 30,
            normal_secs: 60,
            long_secs: 90,
        },
        "maicie",
        &candidates,
        &request,
    )
    .unwrap() else {
        panic!("délégation attendue")
    };
    created
}

fn query_claim(
    operation: &str,
    request_id: &str,
    delegation_id: Uuid,
    generation: u64,
) -> GuichetClaim {
    let canonical_request = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"scope-0123456789abcdef0123456789abcdef\",\"request_id\":\"{request_id}\",\"issued_at\":1000,\"from\":\"prospective\",\"to\":\"maicie\",\"operation\":\"{operation}\",\"payload\":{{\"delegation_id\":\"{delegation_id}\"}}}}"
    )
    .into_bytes();
    GuichetClaim {
        issuer_scope: "scope-0123456789abcdef0123456789abcdef".to_string(),
        request_id: request_id.to_string(),
        canonical_request,
        claimed_at: 1_000,
        claim_generation: generation,
        claim_token: format!("claim-{generation:032}"),
        claim_lease_expires_at: 1_100,
        expires_at: 1_200,
    }
}

#[test]
fn mission_status_separe_remise_transport_et_fraicheur_et_rejoue_les_octets() {
    let root = root("mission");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let mut store = MaicieStore::open(&database).unwrap();
    store
        .record_lookup_issue(
            created.message_id,
            &IdempotencyIssue::Accepted { expires_at: 1_200 },
            1_005,
        )
        .unwrap();
    let claim = query_claim(
        "mission_status",
        "request-mission-status",
        created.delegation_id,
        1,
    );
    let transport = SnapshotTransport {
        message_id: created.message_id,
        request_state: Some("open".to_string()),
        observed_at: 1_006,
        source: SourceSnapshot::AcpSubscription,
        subscription_id: Some("subscription-1".to_string()),
        seq: Some(42),
        stream_state: EtatFlux::Fresh,
    };
    let first = process_mission_status_claim(
        &mut store,
        &claim,
        "response-mission-status",
        Some(&transport),
        1_010,
    )
    .unwrap();
    let value: Value = serde_json::from_slice(&first.reply_bytes).unwrap();
    assert_eq!(value["payload"]["kind"], "mission_status");
    assert_eq!(value["payload"]["coordination_state"], "en_coordination");
    assert_eq!(value["payload"]["local_delivery"]["state"], "accepted");
    assert_eq!(value["payload"]["local_delivery"]["issue"], "accepted");
    assert_eq!(value["payload"]["local_delivery"]["observed_at"], 1_005);
    assert_eq!(
        value["payload"]["transport_observation"]["state"],
        "connected"
    );
    assert_eq!(
        value["payload"]["transport_observation"]["source"],
        "acp_subscription"
    );
    assert_eq!(value["payload"]["transport_observation"]["seq"], 42);
    assert_eq!(value["payload"]["freshness"], "fresh");

    let changed_observation = SnapshotTransport {
        stream_state: EtatFlux::Gap,
        observed_at: 1_011,
        ..transport
    };
    let replay = process_mission_status_claim(
        &mut store,
        &claim,
        "ignored-on-replay",
        Some(&changed_observation),
        1_012,
    )
    .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);

    let gap_claim = query_claim(
        "mission_status",
        "request-mission-gap",
        created.delegation_id,
        1,
    );
    let gap = process_mission_status_claim(
        &mut store,
        &gap_claim,
        "response-mission-gap",
        Some(&changed_observation),
        1_012,
    )
    .unwrap();
    let gap_value: Value = serde_json::from_slice(&gap.reply_bytes).unwrap();
    assert_eq!(gap_value["payload"]["freshness"], "gap");
    assert_eq!(
        gap_value["payload"]["transport_observation"]["state"],
        "gap"
    );
    assert_eq!(gap_value["payload"]["local_delivery"]["state"], "accepted");

    let next_generation = query_claim(
        "mission_status",
        "request-mission-status",
        created.delegation_id,
        2,
    );
    let reclaimed = process_mission_status_claim(
        &mut store,
        &next_generation,
        "ignored-on-reclaim",
        Some(&changed_observation),
        1_013,
    )
    .unwrap();
    let reclaimed_value: Value = serde_json::from_slice(&reclaimed.reply_bytes).unwrap();
    assert!(reclaimed.replayed);
    assert_ne!(reclaimed.reply_bytes, first.reply_bytes);
    assert_eq!(reclaimed_value["claim_generation"], 2);
    assert_eq!(
        reclaimed_value["response_message_id"],
        "response-mission-status"
    );
    assert_eq!(reclaimed_value["payload"], value["payload"]);
    assert_eq!(reclaimed_value["payload"]["freshness"], "fresh");
    drop(store);
    let connection = Connection::open(&database).unwrap();
    let objective_state: String = connection
        .query_row(
            "SELECT state FROM objectives WHERE id = ?1",
            [created.objective_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(objective_state, "en_coordination");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mission_status_sans_observation_est_indisponible_et_refuse_une_fausse_correlation() {
    let root = root("unavailable");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = query_claim(
        "mission_status",
        "request-mission-unavailable",
        created.delegation_id,
        1,
    );
    let mut store = MaicieStore::open(&database).unwrap();
    let result =
        process_guichet_claim(&mut store, &claim, "response-mission-unavailable", 1_010).unwrap();
    let value: Value = serde_json::from_slice(&result.reply_bytes).unwrap();
    assert_eq!(value["payload"]["freshness"], "unavailable");
    assert!(value["payload"].get("transport_observation").is_none());

    let divergent_claim = query_claim(
        "mission_status",
        "request-mission-divergent",
        created.delegation_id,
        1,
    );
    let unrelated = SnapshotTransport {
        message_id: Uuid::new_v4(),
        request_state: None,
        observed_at: 1_011,
        source: SourceSnapshot::Bridget,
        subscription_id: None,
        seq: None,
        stream_state: EtatFlux::Fresh,
    };
    assert!(matches!(
        process_mission_status_claim(
            &mut store,
            &divergent_claim,
            "response-divergent",
            Some(&unrelated),
            1_012,
        ),
        Err(GuichetError::InvalidEnvelope(_))
    ));
    drop(store);
    let connection = Connection::open(&database).unwrap();
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_receptions WHERE request_id = 'request-mission-divergent'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deadline_question_est_passive_factuelle_et_durable() {
    let root = root("deadline");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = query_claim(
        "deadline_question",
        "request-deadline",
        created.delegation_id,
        1,
    );
    let mut store = MaicieStore::open(&database).unwrap();
    let first =
        process_deadline_question_claim(&mut store, &claim, "response-deadline", 5_000).unwrap();
    let value: Value = serde_json::from_slice(&first.reply_bytes).unwrap();
    assert_eq!(value["payload"]["kind"], "deadline_question");
    assert_eq!(value["payload"]["duration_class"], "normale");
    assert_eq!(value["payload"]["deadline_at"], 960);
    let rendered = String::from_utf8(first.reply_bytes.clone()).unwrap();
    for forbidden in ["bloqué", "terminé", "en retard"] {
        assert!(!rendered.contains(forbidden));
    }
    let replay = process_guichet_claim(&mut store, &claim, "ignored", 5_100).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);
    assert_eq!(replay.response_message_id, "response-deadline");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn projection_verrouille_lecture_et_recu_puis_reprend_une_faute_sans_doublon() {
    let root = root("atomic");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = query_claim(
        "deadline_question",
        "request-deadline-atomic",
        created.delegation_id,
        1,
    );
    let canonical = parse_claim(&claim).unwrap();
    let mut store = MaicieStore::open(&database).unwrap();
    let (facts_read_tx, facts_read_rx) = mpsc::channel();
    let (mutation_tx, mutation_rx) = mpsc::channel();
    let mutation_database = database.clone();
    let objective_id = created.objective_id;
    let mutator = thread::spawn(move || {
        let connection = Connection::open(mutation_database).unwrap();
        connection.busy_timeout(Duration::ZERO).unwrap();
        facts_read_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let result = connection.execute(
            "UPDATE objectives SET payload_json = payload_json WHERE id = ?1",
            [objective_id.to_string()],
        );
        mutation_tx.send(result).unwrap();
    });

    let injected = store.persist_guichet_projection(
        &claim,
        &canonical,
        "response-deadline-atomic",
        5_000,
        |_facts| {
            facts_read_tx.send(()).unwrap();
            let mutation = mutation_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(matches!(
                mutation,
                Err(rusqlite::Error::SqliteFailure(error, _))
                    if error.code == ErrorCode::DatabaseBusy
            ));
            Err::<ProjectionReply, StoreError>(StoreError::Invalid("faute injectée avant reçu"))
        },
    );
    assert!(matches!(
        injected,
        Err(StoreError::Invalid("faute injectée avant reçu"))
    ));
    mutator.join().unwrap();

    let connection = Connection::open(&database).unwrap();
    let receipts_after_failure: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_receptions WHERE request_id = ?1",
            [&claim.request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(receipts_after_failure, 0);
    drop(connection);

    let first =
        process_deadline_question_claim(&mut store, &claim, "response-deadline-atomic", 5_001)
            .unwrap();
    let replay = process_deadline_question_claim(&mut store, &claim, "ignored", 5_002).unwrap();
    assert!(!first.replayed);
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);

    drop(store);
    let connection = Connection::open(&database).unwrap();
    let unique_receipt: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_receptions WHERE request_id = ?1",
            [&claim.request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(unique_receipt, 1);
    let value: Value = serde_json::from_slice(&first.reply_bytes).unwrap();
    assert_eq!(value["payload"]["duration_class"], "normale");
    assert_eq!(value["payload"]["deadline_at"], 960);
    fs::remove_dir_all(root).unwrap();
}
