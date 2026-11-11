use super::*;
use crate::wake_schedules::{WakeSource, calculate_cadence};
use bridget_transport::protocol::{WakeCadence, WakeTarget};
#[test]
#[allow(non_snake_case)] // nom des filtres documentés de la recette 158B/159
fn wake158B_groupe1_sql_limites_physiques_et_annulees() {
    let dir =
        std::env::temp_dir().join(format!("bridget-wake-bounds158B-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    std::fs::set_permissions(
        &dir,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700),
    )
    .unwrap();
    let store = Store::open(&dir.join("wake.sqlite")).unwrap();
    let mut s = WakeSchedule {
        schedule_id: uuid::Uuid::new_v4().to_string(),
        owner: "owner0".into(),
        owner_project: None,
        owner_native: None,
        target: WakeTarget::SelfTarget {},
        cadence: WakeCadence::Interval { seconds: 60 },
        label: String::new(),
        anchor_at: 0,
        next_at: 60,
        timezone: "UTC:0".into(),
        next_calendar_date: None,
        previous_calendar_date: None,
        occurrence: None,
        cancelled_at: None,
    };
    for owner in 0..8 {
        for _ in 0..16 {
            s.owner = format!("owner{owner}");
            s.schedule_id = uuid::Uuid::new_v4().to_string();
            store.create_wake_schedule(&s).unwrap();
        }
    }
    assert_eq!(store.list_wake_schedules(None).unwrap().len(), 128);
    s.schedule_id = uuid::Uuid::new_v4().to_string();
    assert!(
        store
            .create_wake_schedule(&s)
            .unwrap_err()
            .to_string()
            .contains("schedule_limit")
    );
    assert_eq!(store.list_wake_schedules(None).unwrap().len(), 128);
    let first = store.list_wake_schedules(Some("owner7")).unwrap()[0]
        .schedule_id
        .clone();
    let mut held = FrozenWake {
        version: 1,
        source: WakeSource::Scheduled,
        target: s.target.clone(),
        owner: s.owner.clone(),
        owner_project: s.owner_project.clone(),
        owner_native: None,
        due_at: 60,
        issued_at: 60,
        grace_until: 660,
        id: format!("wake:{first}:60"),
        label: String::new(),
        body: String::new(),
        canonical_bytes: Vec::new(),
        thread_operation_id: None,
        delivery_id: None,
        phase: WakePhase::Prepared,
    };
    held.freeze_canonical().unwrap();
    assert!(
        store
            .freeze_wake_occurrence(&first, 60, "UTC:0", "UTC:0", &held, 120, None, None)
            .unwrap()
    );
    assert!(
        store
            .cancel_wake_schedule(Some("foreign"), &first, 1)
            .is_err()
    );
    assert!(
        store
            .cancel_wake_schedule(Some("owner7"), &first, 2)
            .unwrap()
    );
    assert!(
        !store
            .cancel_wake_schedule(Some("owner7"), &first, 3)
            .unwrap()
    );
    assert_eq!(store.list_wake_schedules(None).unwrap().len(), 128);
    assert!(
        store
            .create_wake_schedule(&s)
            .unwrap_err()
            .to_string()
            .contains("schedule_limit")
    );
    assert_eq!(store.list_wake_schedules(None).unwrap().len(), 128);
    assert!(
        store
            .settle_wake_occurrence(&first, &held.id, WakePhase::OwnerUnavailable, 120, None)
            .unwrap()
    );
    store.create_wake_schedule(&s).unwrap();
    assert_eq!(store.list_wake_schedules(None).unwrap().len(), 128);
    assert_eq!(store.list_wake_schedules(Some("owner7")).unwrap().len(), 16);
    assert!(
        store
            .cancel_wake_schedule(Some("owner7"), &first, 4)
            .is_err()
    );
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
#[allow(non_snake_case)] // nom des filtres documentés de la recette 158B/159
fn wake158B_groupe3_intervalle_gel_restart_et_frontieres() {
    let dir =
        std::env::temp_dir().join(format!("bridget-wake-freeze158B-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    std::fs::set_permissions(
        &dir,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700),
    )
    .unwrap();
    let path = dir.join("wake.sqlite");
    let store = Store::open(&path).unwrap();
    let s = WakeSchedule {
        schedule_id: uuid::Uuid::new_v4().to_string(),
        owner: "owner".into(),
        owner_project: None,
        owner_native: None,
        target: WakeTarget::Thread {
            id: uuid::Uuid::new_v4().to_string(),
        },
        cadence: WakeCadence::Interval { seconds: 60 },
        label: "étiquette entière".into(),
        anchor_at: 0,
        next_at: 60,
        timezone: String::new(),
        next_calendar_date: None,
        previous_calendar_date: None,
        occurrence: None,
        cancelled_at: None,
    };
    store.create_wake_schedule(&s).unwrap();
    let calc = calculate_cadence(&s, 601).unwrap();
    assert_eq!(calc.due_at, Some(600));
    assert_eq!(calc.next_at, 660);
    let mut wake = FrozenWake {
        version: 1,
        source: WakeSource::Scheduled,
        target: s.target.clone(),
        owner: s.owner.clone(),
        owner_project: s.owner_project.clone(),
        owner_native: None,
        due_at: 600,
        issued_at: 601,
        grace_until: 1200,
        id: format!("wake:{}:600", s.schedule_id),
        label: s.label.clone(),
        body: s.label.clone(),
        canonical_bytes: Vec::new(),
        thread_operation_id: Some(uuid::Uuid::new_v4().to_string()),
        delivery_id: None,
        phase: WakePhase::Prepared,
    };
    wake.freeze_canonical().unwrap();
    assert!(
        !store
            .freeze_wake_occurrence(
                &s.schedule_id,
                60,
                &calc.fingerprint,
                "obsolete",
                &wake,
                660,
                None,
                None
            )
            .unwrap()
    );
    assert!(
        store
            .wake_schedule(&s.schedule_id)
            .unwrap()
            .unwrap()
            .occurrence
            .is_none()
    );
    assert!(
        store
            .freeze_wake_occurrence(
                &s.schedule_id,
                60,
                &calc.fingerprint,
                &calc.fingerprint,
                &wake,
                660,
                None,
                None
            )
            .unwrap()
    );
    assert!(
        !store
            .freeze_wake_occurrence(
                &s.schedule_id,
                60,
                &calc.fingerprint,
                &calc.fingerprint,
                &wake,
                660,
                None,
                None
            )
            .unwrap()
    );
    assert!(
        !store
            .advance_wake_calendar(
                &s.schedule_id,
                660,
                &calc.fingerprint,
                &calc.fingerprint,
                720,
                None,
                601
            )
            .unwrap()
    );
    drop(store);
    let store = Store::open(&path).unwrap();
    let loaded = store
        .wake_schedule(&s.schedule_id)
        .unwrap()
        .unwrap()
        .occurrence
        .unwrap();
    assert_eq!(loaded, wake);
    assert!(
        store
            .settle_wake_occurrence(
                &s.schedule_id,
                &wake.id,
                WakePhase::OutcomeUnknown,
                1260,
                None
            )
            .unwrap()
    );
    assert!(
        !store
            .settle_wake_occurrence(&s.schedule_id, &wake.id, WakePhase::Confirmed, 1260, None)
            .unwrap()
    );
    let saved = store.wake_schedule(&s.schedule_id).unwrap().unwrap();
    assert_eq!(
        saved.occurrence.unwrap().canonical_bytes,
        wake.canonical_bytes
    );
    assert_eq!(saved.next_at, 1260);
    // Une occurrence fixe bloquée deux jours avance aussi sa date civile.
    let mut fixed = s.clone();
    fixed.schedule_id = uuid::Uuid::new_v4().to_string();
    fixed.cadence = WakeCadence::At {
        time: "09:00".into(),
        weekdays: vec![1, 2, 3, 4, 5, 6, 7],
    };
    fixed.next_at = 1_791_804_000;
    let initial = calculate_cadence(&fixed, fixed.next_at).unwrap();
    fixed.next_at = initial.next_at;
    fixed.timezone = initial.fingerprint.clone();
    fixed.next_calendar_date = initial.next_calendar_date.clone();
    store.create_wake_schedule(&fixed).unwrap();
    let due = calculate_cadence(&fixed, fixed.next_at).unwrap();
    let mut held = wake.clone();
    held.id = format!("wake:{}:{}", fixed.schedule_id, fixed.next_at);
    held.due_at = fixed.next_at;
    held.issued_at = fixed.next_at;
    held.grace_until = fixed.next_at + 600;
    held.thread_operation_id = Some(uuid::Uuid::new_v4().to_string());
    held.phase = WakePhase::Prepared;
    held.freeze_canonical().unwrap();
    assert!(
        store
            .freeze_wake_occurrence(
                &fixed.schedule_id,
                fixed.next_at,
                &due.fingerprint,
                &due.fingerprint,
                &held,
                due.next_at,
                due.calendar_date.as_deref(),
                due.next_calendar_date.as_deref()
            )
            .unwrap()
    );
    let blocked = store.wake_schedule(&fixed.schedule_id).unwrap().unwrap();
    let late = held.due_at + 2 * 86400 + 3600;
    let future = calculate_cadence(&blocked, late).unwrap();
    assert!(future.next_at > late);
    assert_ne!(blocked.next_calendar_date, future.next_calendar_date);
    assert!(
        store
            .settle_wake_occurrence(
                &fixed.schedule_id,
                &held.id,
                WakePhase::OutcomeUnknown,
                future.next_at,
                None
            )
            .is_err()
    );
    assert_eq!(
        store.wake_schedule(&fixed.schedule_id).unwrap().unwrap(),
        blocked
    );
    assert!(
        store
            .settle_wake_occurrence(
                &fixed.schedule_id,
                &held.id,
                WakePhase::OutcomeUnknown,
                future.next_at,
                future.next_calendar_date.as_deref()
            )
            .unwrap()
    );
    let resumed = store.wake_schedule(&fixed.schedule_id).unwrap().unwrap();
    assert_eq!(resumed.next_at, future.next_at);
    assert_eq!(resumed.next_calendar_date, future.next_calendar_date);
    assert_eq!(
        resumed.occurrence.as_ref().unwrap().canonical_bytes,
        held.canonical_bytes
    );
    let next_due = calculate_cadence(&resumed, resumed.next_at + 1).unwrap();
    assert_eq!(next_due.calendar_date, resumed.next_calendar_date);
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}
