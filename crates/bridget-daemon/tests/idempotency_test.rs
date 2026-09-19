use bridget_daemon::idempotency::{
    IdempotencyKey, IdempotencyStore, LookupResult, OperationKind, Reservation,
};

const NOW: i64 = 1_000_000;

fn key() -> IdempotencyKey {
    IdempotencyKey::new(
        "012_scope_aaaaaaaaaaaa",
        OperationKind::Send,
        "external-message",
    )
    .unwrap()
}

#[test]
fn public_api_expires_an_unpurged_record() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        store.reserve(&key(), b"canon", NOW, 10, NOW, 30).unwrap(),
        Reservation::Prepared { .. }
    ));
    assert_eq!(
        store.lookup(&key(), NOW + 10).unwrap(),
        LookupResult::IdempotencyExpired
    );
    assert_eq!(
        store
            .reserve(&key(), b"canon", NOW, 10, NOW + 10, 30)
            .unwrap(),
        Reservation::IdempotencyExpired
    );
}

#[test]
fn public_api_calculates_expiry_from_issued_at_and_horizon() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert_eq!(
        store
            .reserve(&key(), b"canon", NOW - 20, 10, NOW, 30)
            .unwrap(),
        Reservation::IdempotencyExpired
    );
}
