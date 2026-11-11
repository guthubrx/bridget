use super::*;
use std::os::unix::fs::PermissionsExt;
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU64, Ordering},
};
use std::thread;

const NOW: i64 = 1_000_000;
static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn test_store(quota: ReceiptQuota) -> (ReceiptStore, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "bridget-receipts-{}-{}",
        std::process::id(),
        NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    let store = ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", quota).unwrap();
    (store, root)
}

#[test]
fn seen_then_acked_replays_the_ack_without_another_injection() {
    let (store, root) = test_store(ReceiptQuota::default());
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.acknowledge("delivery_1", NOW).unwrap(),
        ReceiptDecision::Acked
    );
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Acked
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn seen_without_ack_is_indeterminate_and_never_requests_a_second_injection() {
    let (store, root) = test_store(ReceiptQuota::default());
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn quota_saturation_is_a_terminal_refusal_before_a_seen_write() {
    let (store, root) = test_store(ReceiptQuota {
        max_entries: 1,
        max_bytes: 1024,
        horizon_max_secs: 60,
    });
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.receive("delivery_2", NOW + 60, NOW).unwrap(),
        ReceiptDecision::RejectedQuota
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_corrupted_acked_receipt_quarantines_without_reinjection() {
    let (store, root) = test_store(ReceiptQuota::default());
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.acknowledge("delivery_1", NOW).unwrap(),
        ReceiptDecision::Acked
    );
    fs::write(store.entry_path("delivery_1"), b"broken").unwrap();
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    assert!(store.quarantine_path().exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn disappearance_after_initialization_is_fail_closed() {
    let (store, root) = test_store(ReceiptQuota::default());
    fs::remove_dir_all(&store.directory).unwrap();
    assert_eq!(
        store.receive("delivery_1", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    assert!(!store.directory.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn deleting_an_enrolled_instance_directory_remains_fail_closed_after_reopen() {
    let (store, root) = test_store(ReceiptQuota::default());
    assert_eq!(
        store.receive("delivery_old", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.acknowledge("delivery_old", NOW).unwrap(),
        ReceiptDecision::Acked
    );
    let directory = store.directory.clone();
    drop(store);

    fs::remove_dir_all(directory).unwrap();
    let reopened =
        ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", ReceiptQuota::default()).unwrap();
    assert_eq!(
        reopened.receive("delivery_old", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    assert_eq!(
        reopened.receive("delivery_new", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_sentinel_or_recreated_directory_is_fail_closed_after_initialization() {
    let (store, root) = test_store(ReceiptQuota::default());
    assert_eq!(
        store.receive("delivery_old", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    fs::remove_file(store.max_expires_path()).unwrap();
    assert_eq!(
        store.receive("delivery_old", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    assert_eq!(
        store.receive("delivery_new", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    drop(store);
    let reopened =
        ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", ReceiptQuota::default()).unwrap();
    assert_eq!(
        reopened.receive("delivery_old", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    assert_eq!(
        reopened.receive("delivery_new", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );

    let directory = reopened.directory.clone();
    drop(reopened);
    fs::remove_dir_all(&directory).unwrap();
    create_private_dir(&directory).unwrap();
    let recreated =
        ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", ReceiptQuota::default()).unwrap();
    assert_eq!(
        recreated.receive("delivery_old", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    assert_eq!(
        recreated.receive("delivery_new", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Indeterminate
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn concurrent_delivery_reserves_once_and_serializes_quota() {
    let (store, root) = test_store(ReceiptQuota {
        max_entries: 1,
        max_bytes: 1024,
        horizon_max_secs: 60,
    });
    let store = Arc::new(store);
    let barrier = Arc::new(Barrier::new(3));
    let mut same_delivery = Vec::new();
    for _ in 0..2 {
        let store = store.clone();
        let barrier = barrier.clone();
        same_delivery.push(thread::spawn(move || {
            barrier.wait();
            store.receive("delivery_same", NOW + 60, NOW).unwrap()
        }));
    }
    barrier.wait();
    let decisions: Vec<_> = same_delivery
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == ReceiptDecision::Inject)
            .count(),
        1
    );
    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == ReceiptDecision::Indeterminate)
            .count(),
        1
    );

    let quota_store = Arc::new(
        ReceiptStore::open(
            &root,
            "instance_012_bbbbbbbbbbbb",
            ReceiptQuota {
                max_entries: 1,
                max_bytes: 1024,
                horizon_max_secs: 60,
            },
        )
        .unwrap(),
    );
    let quota_barrier = Arc::new(Barrier::new(3));
    let mut distinct_delivery = Vec::new();
    for delivery_id in ["delivery_other_1", "delivery_other_2"] {
        let store = quota_store.clone();
        let barrier = quota_barrier.clone();
        distinct_delivery.push(thread::spawn(move || {
            barrier.wait();
            store.receive(delivery_id, NOW + 60, NOW).unwrap()
        }));
    }
    quota_barrier.wait();
    let decisions: Vec<_> = distinct_delivery
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == ReceiptDecision::Inject)
            .count(),
        1
    );
    assert_eq!(
        decisions
            .iter()
            .filter(|decision| **decision == ReceiptDecision::RejectedQuota)
            .count(),
        1
    );
    drop(store);
    drop(quota_store);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn only_one_handle_can_open_an_instance_at_once() {
    let root = std::env::temp_dir().join(format!(
        "bridget-receipts-handle-{}-{}",
        std::process::id(),
        NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let root = root.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let opened =
                ReceiptStore::open(&root, "instance_012_cccccccccccc", ReceiptQuota::default());
            barrier.wait();
            let accepted = opened.is_ok();
            drop(opened);
            accepted
        }));
    }
    barrier.wait();
    let opened_count = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(|opened| *opened)
        .count();
    assert_eq!(opened_count, 1);
    match ReceiptStore::open(&root, "instance_012_cccccccccccc", ReceiptQuota::default()) {
        Ok(store) => drop(store),
        Err(error) => panic!("le verrou doit être libéré après les deux handles: {error}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn quota_reserves_the_final_acked_size_at_its_exact_boundary() {
    let expires_at = NOW + 60;
    let final_size = ReceiptEntry::Acked { expires_at }.serialize().len() as u64;
    let (store, root) = test_store(ReceiptQuota {
        max_entries: 2,
        max_bytes: final_size,
        horizon_max_secs: 60,
    });
    assert_eq!(
        store.receive("delivery_1", expires_at, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.acknowledge("delivery_1", NOW).unwrap(),
        ReceiptDecision::Acked
    );
    assert_eq!(
        store.receive("delivery_2", expires_at, NOW).unwrap(),
        ReceiptDecision::RejectedQuota
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn private_state_permissions_and_compaction_are_enforced() {
    let (store, root) = test_store(ReceiptQuota::default());
    assert_eq!(
        store.receive("delivery_1", NOW + 1, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        store.receive("delivery_2", NOW + 60, NOW).unwrap(),
        ReceiptDecision::Inject
    );
    assert_eq!(
        fs::metadata(&store.directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(store.entry_path("delivery_1"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(store.purge_expired(NOW + 1).unwrap(), 1);
    assert_eq!(store.read_max_expires_at().unwrap(), NOW + 60);
    let _ = fs::remove_dir_all(root);
}
