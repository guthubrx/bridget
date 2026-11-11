use super::*;
use crate::observation::{Fact, NoticePayload, Notification, Observations};
use bridget_transport::protocol::{ObservationKind, ObservationRequest};
use std::time::Instant;

// Trois groupes prescrits, même Store SQLite réel ; aucun provider/processus.
fn fixture() -> Store {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;CREATE TABLE observation_subscriptions(singleton INTEGER PRIMARY KEY,payload BLOB NOT NULL);").unwrap();
    ensure_schema(&conn).unwrap();
    Store { conn }
}
fn notice(owner: &str, sub: &str) -> Notification {
    Notification {
        owner: owner.into(),
        body: "texte modele non utilise comme preuve".into(),
        fact_id: uuid::Uuid::new_v4().to_string(),
        subscription_id: sub.into(),
        payload: NoticePayload::Fact(Fact {
            event: ObservationKind::TurnEnded,
            agent: "source".into(),
            host: "hote".into(),
            file: None,
            other_agent: None,
        }),
    }
}

#[test]
fn observation158b_g8_provenance_once_writer_et_transaction() {
    let now = Instant::now();
    let mut observations = Observations::default();
    observations.set_source("primary", "source", vec![ObservationKind::TurnEnded], now);
    assert_eq!(
        observations.request(
            "owner",
            ObservationRequest::Sub {
                event: ObservationKind::TurnEnded,
                agent: Some("source".into()),
                file: None,
                once: true,
                ttl_secs: None
            },
            now,
            100
        )["status"],
        "subscribed"
    );
    let notices = observations.observe(
        Fact {
            event: ObservationKind::TurnEnded,
            agent: "source".into(),
            host: "h".into(),
            file: None,
            other_agent: None,
        },
        now,
    );
    assert_eq!(notices.len(), 1);
    assert!(observations.subscriptions.is_empty());
    assert!(matches!(notices[0].payload, NoticePayload::Fact(_)));
    let mut store = fixture();
    let snapshot = observations.snapshot().unwrap();
    let ids = store
        .save_observation_batch(Some(&snapshot), &notices, 100)
        .unwrap();
    assert_eq!(ids, vec![notices[0].fact_id.clone()]);
    assert_eq!(store.observation_snapshot().unwrap(), snapshot);
    assert_eq!(
        store
            .observation_missed_page("owner", None, None, 100)
            .unwrap()["facts"][0]["phase"],
        "pending_write"
    );
    assert!(
        store
            .finish_observation_write(
                "owner",
                &ids[0],
                ObservationWriteOutcome::FailedBeforeWrite,
                100
            )
            .unwrap()
    );
    assert_eq!(
        store
            .observation_missed_page("owner", None, None, 100)
            .unwrap()["facts"][0]["phase"],
        "missed_before_write"
    );
    assert!(
        !store
            .finish_observation_write("other", &ids[0], ObservationWriteOutcome::Written, 100)
            .unwrap()
    );
    let late = notice("owner", "late");
    store
        .save_observation_batch(None, std::slice::from_ref(&late), 100)
        .unwrap();
    store.maintain_observation_missed(105, false).unwrap();
    assert!(
        !store
            .finish_observation_write(
                "owner",
                &late.fact_id,
                ObservationWriteOutcome::Written,
                106
            )
            .unwrap()
    );
    let page = store
        .observation_missed_page("owner", None, None, 106)
        .unwrap();
    assert_eq!(page["facts"][1]["phase"], "write_unknown");
    let written = notice("owner", "written");
    store
        .save_observation_batch(None, std::slice::from_ref(&written), 107)
        .unwrap();
    assert!(
        store
            .finish_observation_write(
                "owner",
                &written.fact_id,
                ObservationWriteOutcome::Written,
                107
            )
            .unwrap()
    );
    let mut oversized = notice("owner", "oversized");
    if let NoticePayload::Fact(fact) = &mut oversized.payload {
        fact.host = "é".repeat(16384);
    }
    assert!(
        store
            .save_observation_batch(Some(b"[]"), &[oversized], 108)
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .observation_missed_page("owner", None, None, 108)
            .unwrap()["gaps"][0]["count"]
            .as_i64()
            .unwrap()
            > 0
    );
    let before = store.observation_snapshot().unwrap();
    store.conn.execute_batch("CREATE TRIGGER deny_fact BEFORE INSERT ON observation_missed BEGIN SELECT RAISE(ABORT,'fixture store failure');END;").unwrap();
    assert!(
        store
            .save_observation_batch(Some(b"[{}]"), &[notice("owner", "rollback")], 109)
            .is_err()
    );
    assert_eq!(store.observation_snapshot().unwrap(), before);
}

#[test]
fn observation158b_g9_bornes_bytes_ttl_proprietaires_et_lacunes() {
    let mut store = fixture();
    let notices: Vec<_> = (0..65).map(|_| notice("owner", "subscription")).collect();
    store.save_observation_batch(None, &notices, 100).unwrap();
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM observation_missed", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        64
    );
    assert!(
        !store
            .finish_observation_write(
                "owner",
                &notices[0].fact_id,
                ObservationWriteOutcome::Written,
                100
            )
            .unwrap()
    );
    let before = store
        .observation_missed_page("owner", Some(64), None, 100)
        .unwrap();
    assert!(before["gaps"][0]["count"].as_i64().unwrap() >= 1);
    assert!(serde_json::to_vec(&before).unwrap().len() <= 65536);
    assert_eq!(
        before,
        store
            .observation_missed_page("owner", Some(64), None, 100)
            .unwrap()
    );
    let prior_seq = before["facts"].as_array().unwrap().last().unwrap()["fact_seq"]
        .as_u64()
        .unwrap();
    store
        .maintain_observation_missed(100 + RETENTION, true)
        .unwrap();
    let page = store
        .observation_missed_page("owner", None, None, 100 + RETENTION)
        .unwrap();
    assert!(page["facts"].as_array().unwrap().is_empty());
    assert!(
        page["gaps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["kind"] == "global" && g["coverage_evicted"].as_i64().unwrap() > 0)
    );
    store
        .save_observation_batch(None, &[notice("owner", "new")], 101 + RETENTION)
        .unwrap();
    assert!(
        store
            .observation_missed_page("owner", None, None, 101 + RETENTION)
            .unwrap()["facts"][0]["fact_seq"]
            .as_u64()
            .unwrap()
            > prior_seq
    );
    assert!(store.conn.execute("INSERT INTO observation_missed(fact_id,owner,subscription_id,observed_at,deadline_at,retained_until,fact_json,phase)VALUES('bad','owner','bad',1,6,999999,?1,'pending_write')",[vec![b'x';16385]]).is_err());
    // Frontière8192 réelle en une TX, pas8192 appels de production indépendants.
    store.conn.execute_batch("DELETE FROM observation_missed;DELETE FROM observation_missed_owners WHERE kind='owner';").unwrap();
    let tx = store.conn.transaction().unwrap();
    for n in 0..8192 {
        let owner = format!("owner{n}");
        let id = uuid::Uuid::new_v4().to_string();
        tx.execute("INSERT INTO observation_missed_owners(kind,owner,last_fact_or_gap_at)VALUES('owner',?1,100)",[&owner]).unwrap();
        let json=serde_json::to_vec(&json!({"owner":owner,"payload":{"kind":"fact","value":{"event":"turn_ended","agent":"source","host":"h","file":null,"other_agent":null}}})).unwrap();
        tx.execute("INSERT INTO observation_missed(fact_id,owner,subscription_id,observed_at,deadline_at,retained_until,event_name,fact_json,phase)VALUES(?1,?2,?2,100,105,999999,'turn_ended',?3,'pending_write')",params![id,owner,json]).unwrap();
    }
    tx.commit().unwrap();
    assert!(
        store
            .save_observation_batch(None, &[notice("newowner", "newsub")], 101)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM observation_missed_owners WHERE kind='owner'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        8192
    );
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM observation_missed", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        8192
    );
    let own = store
        .observation_missed_page("newowner", None, None, 101)
        .unwrap();
    assert!(own["facts"].as_array().unwrap().is_empty());
    assert!(
        own["gaps"]
            .as_array()
            .unwrap()
            .iter()
            .all(|g| g.get("owner").is_none())
    );
}

#[test]
fn observation158b_g10_retour_cutoff_resume_uuid_et_ack_cas() {
    let mut store = fixture();
    let first = notice("owner", "sub");
    store
        .save_observation_batch(None, std::slice::from_ref(&first), 100)
        .unwrap();
    assert!(
        !store
            .observation_owner_presence("owner", false, 100)
            .unwrap()
    );
    assert!(
        store
            .observation_owner_presence("owner", true, 101)
            .unwrap()
    );
    assert!(
        !store
            .observation_owner_presence("owner", true, 101)
            .unwrap()
    );
    assert!(
        store
            .freeze_observation_summary("owner", 101)
            .unwrap()
            .is_none()
    );
    store.maintain_observation_missed(105, false).unwrap();
    let summary = store
        .freeze_observation_summary("owner", 105)
        .unwrap()
        .unwrap();
    assert!(serde_json::to_vec(&summary).unwrap().len() <= 16384);
    assert_eq!(
        summary.summary_id,
        store
            .freeze_observation_summary("owner", 106)
            .unwrap()
            .unwrap()
            .summary_id
    );
    let second = notice("owner", "sub");
    store
        .save_observation_batch(None, std::slice::from_ref(&second), 105)
        .unwrap();
    store
        .finish_observation_write(
            "owner",
            &second.fact_id,
            ObservationWriteOutcome::FailedBeforeWrite,
            105,
        )
        .unwrap();
    assert!(
        store
            .bind_observation_summary_delivery(&summary.summary_id, "delivery", "instance", 2)
            .unwrap()
    );
    assert!(
        !store
            .confirm_observation_summary(
                "owner",
                &summary.summary_id,
                summary.generation,
                "delivery",
                "other",
                2
            )
            .unwrap()
    );
    assert!(
        store
            .confirm_observation_summary(
                "owner",
                &summary.summary_id,
                summary.generation,
                "delivery",
                "instance",
                2
            )
            .unwrap()
    );
    assert!(
        !store
            .confirm_observation_summary(
                "owner",
                &summary.summary_id,
                summary.generation,
                "delivery",
                "instance",
                2
            )
            .unwrap()
    );
    let cutoff=store.conn.query_row("SELECT confirmed_fact_seq FROM observation_missed_owners WHERE kind='owner' AND owner='owner'",[],|r|r.get::<_,u64>(0)).unwrap();
    assert_eq!(cutoff, summary.fact_seq);
    assert!(
        store
            .freeze_observation_summary("owner", 106)
            .unwrap()
            .is_none()
    );
    store
        .observation_owner_presence("owner", false, 107)
        .unwrap();
    store
        .observation_owner_presence("owner", true, 108)
        .unwrap();
    let next = store
        .freeze_observation_summary("owner", 108)
        .unwrap()
        .unwrap();
    assert_ne!(summary.summary_id, next.summary_id);
    assert!(next.fact_seq > summary.fact_seq);
    assert!(
        store
            .bind_observation_summary_delivery(&next.summary_id, "delivery-next", "instance", 3)
            .unwrap()
    );
    assert!(
        store
            .expire_observation_summary(&next.summary_id, 708)
            .unwrap()
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT confirmed_fact_seq FROM observation_missed_owners WHERE owner='owner'",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        cutoff
    );
    // ACK tardif réel : seul le résumé courant inconclusif reste admissible.
    let current = store.observation_summary_current("owner").unwrap().unwrap();
    assert_eq!(current.summary_id, next.summary_id);
    assert_eq!(current.phase, "outcome_unknown");
    assert_eq!(current.delivery_id.as_deref(), Some("delivery-next"));
    assert!(store.observation_summaries_pending().unwrap().is_empty());
    assert!(
        !store
            .confirm_observation_summary(
                "owner",
                &summary.summary_id,
                summary.generation,
                "delivery",
                "instance",
                2
            )
            .unwrap()
    );
    assert!(
        store
            .confirm_observation_summary(
                "owner",
                &current.summary_id,
                current.generation,
                "delivery-next",
                "instance",
                3
            )
            .unwrap()
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT confirmed_fact_seq FROM observation_missed_owners WHERE owner='owner'",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        next.fact_seq
    );
    // Métadonnée purgée : ni lookup ni ACK ne recrée une ligne.
    store.conn.execute_batch("DELETE FROM observation_missed;DELETE FROM observation_missed_owners WHERE kind='owner';").unwrap();
    assert!(
        store
            .observation_summary_current("owner")
            .unwrap()
            .is_none()
    );
    assert!(
        !store
            .confirm_observation_summary(
                "owner",
                &current.summary_id,
                current.generation,
                "delivery-next",
                "instance",
                3
            )
            .unwrap()
    );
    store
        .save_observation_batch(None, &[notice("owner", "again")], 800)
        .unwrap();
    store
        .finish_observation_write(
            "owner",
            store
                .observation_missed_page("owner", None, None, 800)
                .unwrap()["facts"][0]["fact_id"]
                .as_str()
                .unwrap(),
            ObservationWriteOutcome::FailedBeforeWrite,
            800,
        )
        .unwrap();
    store
        .observation_owner_presence("owner", false, 800)
        .unwrap();
    store
        .observation_owner_presence("owner", true, 801)
        .unwrap();
    let recreated = store
        .freeze_observation_summary("owner", 801)
        .unwrap()
        .unwrap();
    assert_ne!(recreated.key, summary.key);
    // Même owner recréé : son nouvel UUID interdit le callback de l'ancien reçu.
    let current_recreated = store.observation_summary_current("owner").unwrap().unwrap();
    assert_eq!(current_recreated.summary_id, recreated.summary_id);
    assert_ne!(current_recreated.summary_id, current.summary_id);
    assert_eq!(current_recreated.generation, 1);
    assert!(
        !store
            .confirm_observation_summary(
                "owner",
                &current.summary_id,
                current.generation,
                "delivery-next",
                "instance",
                3
            )
            .unwrap()
    );
    assert_eq!(
        store
            .observation_summary_current("owner")
            .unwrap()
            .unwrap()
            .summary_id,
        recreated.summary_id
    );
}
