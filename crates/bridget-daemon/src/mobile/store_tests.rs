use super::*;
use uuid::Uuid;

fn scope() -> Scope {
    Scope {
        all_machines: false,
        machines: vec!["cartae".into()],
        all_projects: true,
        projects: vec![],
        operations: vec![Operation::Read, Operation::Message],
    }
}
fn key() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
fn attempt() -> String {
    Uuid::new_v4().to_string()
}

#[test]
fn spec239_specialists_accepts_exact_legacy_global_administrator_only() {
    let full = Scope {
        all_machines: true,
        machines: vec![],
        all_projects: true,
        projects: vec![],
        operations: vec![
            Operation::Read,
            Operation::Message,
            Operation::TaskRead,
            Operation::TaskContinue,
            Operation::TaskCancel,
            Operation::TaskRespond,
            Operation::DevicesManage,
        ],
    };
    let device = |scope: Scope, created_at| Device {
        device_id: Uuid::new_v4().to_string(),
        label: "Test".into(),
        scope,
        created_at,
        expires_at: i64::MAX,
        revoked_at: None,
    };
    let allows = |scope: Scope| {
        device(scope, SPECIALISTS_MANAGEMENT_INTRODUCED_AT - 1).permits_specialists_management()
    };
    assert!(allows(full.clone()));
    assert!(
        !device(full.clone(), SPECIALISTS_MANAGEMENT_INTRODUCED_AT)
            .permits_specialists_management()
    );
    assert!(!device(full.clone(), 0).permits_specialists_management());
    let mut restricted = full.clone();
    restricted.all_machines = false;
    assert!(!allows(restricted));
    restricted = full.clone();
    restricted.all_projects = false;
    assert!(!allows(restricted));
    restricted = full.clone();
    restricted.operations = vec![Operation::Read, Operation::DevicesManage];
    assert!(!allows(restricted));
    for op in full.operations.iter() {
        restricted = full.clone();
        restricted.operations.retain(|value| value != op);
        assert!(!allows(restricted), "missing {op:?}");
    }
    restricted = full.clone();
    restricted.operations[0] = Operation::Message;
    assert!(!allows(restricted), "duplicate is not the complete scope");
    restricted = scope();
    restricted.all_machines = true;
    restricted.operations.push(Operation::SpecialistsManage);
    restricted.all_projects = false;
    assert!(
        device(restricted, SPECIALISTS_MANAGEMENT_INTRODUCED_AT + 1)
            .permits_specialists_management(),
        "explicit approval retains its project boundary after the cutoff"
    );
}

#[test]
fn mobile177_pairing_is_single_use_but_lost_response_reconciles_same_device() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let device_key = key();
    let id = attempt();
    let first = store
        .pair(&grant.credential, &id, &device_key, "Phone", 101)
        .unwrap();
    let replay = store
        .pair(&grant.credential, &id, &device_key, "Phone", 102)
        .unwrap();
    assert_eq!(first, replay);
    assert_eq!(store.authenticate(&device_key, 103).unwrap(), first);
    assert_eq!(
        store.pair(&grant.credential, &attempt(), &key(), "Other", 103),
        Err(StoreError::PairingUnavailable)
    );
}

#[test]
fn mobile177_expired_pairing_does_not_create_an_authorization() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let device_key = key();
    assert_eq!(
        store.pair(
            &grant.credential,
            &attempt(),
            &device_key,
            "Phone",
            grant.expires_at
        ),
        Err(StoreError::PairingUnavailable)
    );
    assert_eq!(
        store.authenticate(&device_key, 401),
        Err(StoreError::AuthorizationRequired)
    );
}

#[test]
fn mobile177_revocation_blocks_reads_replays_and_renewals() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let device_key = key();
    let id = attempt();
    let device = store
        .pair(&grant.credential, &id, &device_key, "Phone", 101)
        .unwrap();
    assert!(store.revoke(&device.device_id, 102).unwrap());
    assert_eq!(
        store.authenticate(&device_key, 103),
        Err(StoreError::DeviceRevoked)
    );
    assert_eq!(
        store.pair(&grant.credential, &id, &device_key, "Phone", 103),
        Err(StoreError::DeviceRevoked)
    );
    assert_eq!(
        store.renew(&device_key, &key(), &attempt(), 103),
        Err(StoreError::DeviceRevoked)
    );
}

#[test]
fn mobile177_scope_is_explicit_and_does_not_expand_from_empty_lists() {
    let s = scope();
    assert!(s.permits(Operation::Read, "cartae", None));
    assert!(!s.permits(Operation::Read, "other", None));
    assert!(!s.permits(Operation::TaskCreate, "cartae", None));
    let mut empty = s.clone();
    empty.machines.clear();
    assert!(!empty.permits(Operation::Read, "cartae", None));
    empty.all_machines = true;
    empty.all_projects = false;
    assert!(!empty.permits(Operation::Read, "cartae", None));
}

#[test]
fn mobile177_credential_expires_and_cannot_renew_after_expiry() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let device_key = key();
    let d = store
        .pair(&grant.credential, &attempt(), &device_key, "Phone", 101)
        .unwrap();
    assert_eq!(
        store.authenticate(&device_key, d.expires_at),
        Err(StoreError::CredentialExpired)
    );
    assert_eq!(
        store.renew(&device_key, &key(), &attempt(), d.expires_at),
        Err(StoreError::CredentialExpired)
    );
}

#[test]
fn mobile177_renewal_recovers_lost_response_without_resurrecting_old_key() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let old = key();
    let d = store
        .pair(&grant.credential, &attempt(), &old, "Phone", 101)
        .unwrap();
    let new = key();
    let request = attempt();
    let changed = store.renew(&old, &new, &request, 200).unwrap();
    let replay = store.renew(&old, &new, &request, 201).unwrap();
    assert_eq!(changed, replay);
    assert_eq!(changed.device_id, d.device_id);
    assert_eq!(store.authenticate(&new, 201).unwrap(), changed);
    assert_eq!(
        store.authenticate(&old, 201),
        Err(StoreError::AuthorizationRequired)
    );
    assert_eq!(
        store.renew(&old, &key(), &request, 201),
        Err(StoreError::EnvelopeMismatch)
    );
}

#[test]
fn mobile177_store_never_persists_raw_pairing_or_device_keys() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let device_key = key();
    store
        .pair(&grant.credential, &attempt(), &device_key, "Phone", 101)
        .unwrap();
    let dump = format!(
        "{:?}",
        store
            .conn
            .query_row(
                "SELECT credential_hash,scope FROM mobile_pairings LIMIT 1",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            )
            .unwrap()
    );
    assert!(!dump.contains(&grant.credential) && !dump.contains(&device_key));
    let stored: String = store
        .conn
        .query_row("SELECT key_hash FROM mobile_devices LIMIT 1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_ne!(stored, device_key);
    assert_eq!(stored.len(), 64);
}

#[test]
fn mobile177_replay_cannot_change_label_or_key() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let device_key = key();
    let id = attempt();
    store
        .pair(&grant.credential, &id, &device_key, "Phone", 101)
        .unwrap();
    assert_eq!(
        store.pair(&grant.credential, &id, &device_key, "Changed", 102),
        Err(StoreError::PairingUnavailable)
    );
    assert_eq!(
        store.pair(&grant.credential, &id, &key(), "Phone", 102),
        Err(StoreError::PairingUnavailable)
    );
}
#[test]
fn mobile177_private_store_survives_restart_without_raw_credentials() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("bridget177-{}", Uuid::new_v4()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let path = root.join("mobile-devices.db");
    let (credential, device_key, device) = {
        let mut store = Store::open(&path).unwrap();
        let grant = store.issue_pairing(&scope(), 100).unwrap();
        let key = key();
        let device = store
            .pair(&grant.credential, &attempt(), &key, "Phone", 101)
            .unwrap();
        (grant.credential, key, device)
    };
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let data = std::fs::read(&path).unwrap();
    assert!(
        !data
            .windows(credential.len())
            .any(|w| w == credential.as_bytes())
    );
    assert!(
        !data
            .windows(device_key.len())
            .any(|w| w == device_key.as_bytes())
    );
    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.authenticate(&device_key, 102).unwrap(), device);
    drop(reopened);
    let link = root.join("linked.db");
    symlink(&path, &link).unwrap();
    assert!(matches!(
        Store::open(&link),
        Err(StoreError::StorageUnavailable)
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn mobile177_device_limit_releases_only_explicitly_revoked_capacity() {
    let mut store = Store::memory();
    let mut first = String::new();
    for i in 0..MAX_DEVICES {
        let grant = store.issue_pairing(&scope(), 100).unwrap();
        let d = store
            .pair(
                &grant.credential,
                &attempt(),
                &key(),
                &format!("Phone {i}"),
                101,
            )
            .unwrap();
        if i == 0 {
            first = d.device_id;
        }
    }
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let id = attempt();
    let device_key = key();
    assert_eq!(
        store.pair(&grant.credential, &id, &device_key, "Extra", 102),
        Err(StoreError::DeviceLimit)
    );
    assert!(store.revoke(&first, 103).unwrap());
    assert!(
        store
            .pair(&grant.credential, &id, &device_key, "Extra", 104)
            .is_ok()
    );
    assert_eq!(store.devices().unwrap().len(), 17);
}

#[test]
fn mobile177_two_consumers_cannot_share_one_pairing() {
    use std::os::unix::fs::DirBuilderExt;
    use std::sync::{Arc, Barrier};
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("bridget177-{}", Uuid::new_v4()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let path = root.join("mobile-devices.db");
    let mut first = Store::open(&path).unwrap();
    let grant = first.issue_pairing(&scope(), 100).unwrap();
    let second = Store::open(&path).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let spawn = |mut store: Store, secret: String, barrier: Arc<Barrier>| {
        std::thread::spawn(move || {
            barrier.wait();
            store.pair(&secret, &attempt(), &key(), "Phone", 101)
        })
    };
    let one = spawn(first, grant.credential.clone(), barrier.clone());
    let two = spawn(second, grant.credential, barrier);
    let results = [one.join().unwrap(), two.join().unwrap()];
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Err(StoreError::PairingUnavailable)))
            .count(),
        1
    );
    let store = Store::open(&path).unwrap();
    assert_eq!(store.devices().unwrap().len(), 1);
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn mobile177_notification_registration_is_explicit_revocable_and_bounded() {
    let mut store = Store::memory();
    let mut rights = scope();
    rights.all_machines = true;
    rights.machines.clear();
    rights.all_projects = true;
    rights.projects.clear();
    rights.operations = vec![Operation::Read, Operation::TaskRead];
    let grant = store.issue_pairing(&rights, 100).unwrap();
    let device = store
        .pair(&grant.credential, &attempt(), &key(), "Phone", 101)
        .unwrap();
    let token = "receiver_token_test_123456789";
    assert!(store.notification_targets(102).unwrap().is_empty());
    store.set_notifications(&device, Some(token), 102).unwrap();
    store.set_notifications(&device, Some(token), 102).unwrap();
    assert_eq!(
        store.notification_targets(103).unwrap(),
        vec![(device.device_id.clone(), token.into())]
    );
    assert!(
        store
            .set_notifications(&device, Some("invalid\nreceiver_token_test"), 103)
            .is_err()
    );
    store.set_notifications(&device, None, 104).unwrap();
    assert!(store.notification_targets(105).unwrap().is_empty());
    store.set_notifications(&device, Some(token), 106).unwrap();
    store.revoke(&device.device_id, 107).unwrap();
    assert!(store.notification_targets(108).unwrap().is_empty());
    assert!(store.set_notifications(&device, Some(token), 108).is_err());
    assert!(
        store
            .notification_targets(101 + DEVICE_TTL)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn mobile177_notification_reassociation_moves_receiver_without_moving_authority() {
    let mut store = Store::memory();
    let mut rights = scope();
    rights.all_machines = true;
    rights.machines.clear();
    rights.operations = vec![Operation::Read, Operation::TaskRead];
    let old_key = key();
    let new_key = key();
    let grant = store.issue_pairing(&rights, 100).unwrap();
    let old = store
        .pair(&grant.credential, &attempt(), &old_key, "Phone", 101)
        .unwrap();
    let grant = store.issue_pairing(&rights, 102).unwrap();
    let new = store
        .pair(&grant.credential, &attempt(), &new_key, "Phone again", 103)
        .unwrap();
    let token = "receiver_reassociation_123456789";
    store.set_notifications(&old, Some(token), 104).unwrap();
    store
        .set_notifications(&new, Some("receiver_previous_token_123456789"), 104)
        .unwrap();
    store.set_notifications(&new, Some(token), 105).unwrap();
    assert_eq!(
        store.notification_targets(106).unwrap(),
        vec![(new.device_id.clone(), token.into())]
    );
    assert_eq!(store.authenticate(&old_key, 106).unwrap(), old);
    assert_eq!(store.authenticate(&new_key, 106).unwrap(), new);
    store.revoke(&old.device_id, 107).unwrap();
    assert!(store.set_notifications(&old, Some(token), 108).is_err());
    assert_eq!(
        store.notification_targets(108).unwrap(),
        vec![(new.device_id.clone(), token.into())]
    );
}

#[test]
fn mobile177_notification_schema_upgrades_without_changing_device_authority() {
    let mut store = Store::memory();
    let grant = store.issue_pairing(&scope(), 100).unwrap();
    let credential = key();
    let device = store
        .pair(&grant.credential, &attempt(), &credential, "Phone", 101)
        .unwrap();
    store
        .conn
        .execute_batch("DROP TABLE mobile_notifications; PRAGMA user_version=1;")
        .unwrap();
    let reopened = Store::from_connection(store.conn).unwrap();
    assert_eq!(reopened.authenticate(&credential, 102).unwrap(), device);
    assert!(reopened.notification_targets(102).unwrap().is_empty());
    assert!(
        reopened
            .set_notifications(&device, Some("receiver_token_test_123456789"), 102)
            .is_err()
    );
}
