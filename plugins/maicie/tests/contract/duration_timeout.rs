use maicie::app::{
    delegate, status, timeout_for_duration, DelegateRequest, DelegateResult, DelegationCandidate,
};
use maicie::config::DurationClasses;
use maicie::domain::{ClasseDuree, EtatObjectif, EtatOutboxDelegation};
use maicie::store::MaicieStore;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn root() -> PathBuf {
    std::env::temp_dir().join(format!("maicie-duration-timeout-{}", Uuid::new_v4()))
}

fn durations() -> DurationClasses {
    DurationClasses {
        short_secs: 17,
        normal_secs: 43,
        long_secs: 101,
    }
}

#[test]
fn les_trois_classes_produisent_le_timeout_et_l_echeance_contractuelle_persistes() {
    let root = root();
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidate = DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec![],
        available: true,
        dnd: false,
    };
    let now = 10_000;
    let cases = [
        (ClasseDuree::Courte, 17_u64),
        (ClasseDuree::Normale, 43_u64),
        (ClasseDuree::Longue, 101_u64),
    ];

    for (index, (duration, expected_timeout)) in cases.into_iter().enumerate() {
        assert_eq!(
            timeout_for_duration(duration, durations()),
            expected_timeout
        );
        let request = DelegateRequest {
            goal: "vérifier les délais passifs",
            explicit_target: Some("prospective"),
            required_tags: &[],
            duration,
            reply: false,
            idempotency_key: match index {
                0 => "duration-short",
                1 => "duration-normal",
                _ => "duration-long",
            },
            now,
            retry_until: now + 200,
            dedup_retained_until: now + 300,
            max_frame_bytes: 256 * 1024,
        };
        let DelegateResult::Created(created) = delegate(
            &mut store,
            durations(),
            "maicie",
            std::slice::from_ref(&candidate),
            &request,
        )
        .unwrap() else {
            panic!("création de délégation attendue")
        };

        assert_eq!(created.duration, duration);
        assert_eq!(created.timeout_secs, expected_timeout);
        assert_eq!(
            created.deadline_contractuelle,
            now + i64::try_from(expected_timeout).unwrap()
        );
        let snapshot = store
            .recovery_snapshot(created.message_id)
            .unwrap()
            .unwrap();
        assert_eq!(snapshot.outbox.timeout_secs, expected_timeout);
        assert_eq!(
            snapshot.outbox.deadline_contractuelle,
            created.deadline_contractuelle
        );
        assert_eq!(snapshot.outbox.state, EtatOutboxDelegation::Prepared);
    }

    let before_consultation = status(&store, None).unwrap();
    let after_consultation = status(&store, None).unwrap();
    assert_eq!(before_consultation, after_consultation);
    assert!(after_consultation
        .iter()
        .all(|snapshot| snapshot.objective.etat == EtatObjectif::EnCoordination));
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 3);

    drop(store);
    fs::remove_dir_all(root).unwrap();
}
