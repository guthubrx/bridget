//! Programmations sur l'unique connexion Store ; aucune remise réseau ici.
use super::{Store, StoreError};
use crate::wake_schedules::{FrozenWake, WakePhase, WakeSchedule, validate_schedule};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub(crate) fn ensure_schema(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wake_schedules(
        schedule_id TEXT PRIMARY KEY, owner TEXT NOT NULL, next_at INTEGER NOT NULL,
        cancelled_at INTEGER, payload TEXT NOT NULL CHECK(json_valid(payload)));
        CREATE INDEX IF NOT EXISTS wake_schedules_owner ON wake_schedules(owner,schedule_id);
        CREATE INDEX IF NOT EXISTS wake_schedules_due ON wake_schedules(cancelled_at,next_at);",
    )
    .map_err(StoreError::Sqlite)
}
fn decode(payload: &str) -> Result<WakeSchedule, StoreError> {
    let mut schedule: WakeSchedule = serde_json::from_str(payload)
        .map_err(|_| StoreError::Invariant("wake schedule illisible"))?;
    validate_schedule(&schedule).map_err(StoreError::Invariant)?;
    if let Some(wake) = schedule.occurrence.as_mut() {
        wake.freeze_canonical().map_err(StoreError::Invariant)?;
    }
    Ok(schedule)
}
impl Store {
    /// O(128+B), admission et retrait des seules annulées soldées dans une TX.
    pub(crate) fn create_wake_schedule(&self, schedule: &WakeSchedule) -> Result<(), StoreError> {
        validate_schedule(schedule).map_err(StoreError::Invariant)?;
        if schedule.cancelled_at.is_some()
            || schedule.occurrence.is_some()
            || (matches!(
                &schedule.cadence,
                bridget_transport::protocol::WakeCadence::At { .. }
            ) && schedule.next_calendar_date.is_none())
        {
            return Err(StoreError::Invariant("invalid_request"));
        }
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        for owner_only in [true, false] {
            loop {
                let count: i64 = if owner_only {
                    tx.query_row(
                        "SELECT count(*) FROM wake_schedules WHERE owner=?1",
                        [&schedule.owner],
                        |r| r.get(0),
                    )
                } else {
                    tx.query_row("SELECT count(*) FROM wake_schedules", [], |r| r.get(0))
                }
                .map_err(StoreError::Sqlite)?;
                if count < if owner_only { 16 } else { 128 } {
                    break;
                }
                let removable:Option<String>=tx.query_row("SELECT schedule_id FROM wake_schedules WHERE cancelled_at IS NOT NULL AND (?1=0 OR owner=?2) AND (json_extract(payload,'$.occurrence') IS NULL OR json_extract(payload,'$.occurrence.phase') IN ('committed_thread','confirmed','outcome_unknown','missed_before_departure','owner_unavailable')) ORDER BY cancelled_at,schedule_id LIMIT 1",params![owner_only as i32,schedule.owner],|r|r.get(0)).optional().map_err(StoreError::Sqlite)?;
                let id = removable.ok_or(StoreError::Invariant("schedule_limit"))?;
                tx.execute("DELETE FROM wake_schedules WHERE schedule_id=?1", [id])
                    .map_err(StoreError::Sqlite)?;
            }
        }
        let payload = serde_json::to_string(schedule)
            .map_err(|_| StoreError::Invariant("invalid_request"))?;
        tx.execute("INSERT INTO wake_schedules(schedule_id,owner,next_at,cancelled_at,payload) VALUES(?1,?2,?3,NULL,?4)",params![schedule.schedule_id,schedule.owner,schedule.next_at,payload]).map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)
    }
    /// O(128+B). owner=None réservé au chemin humain attesté, jamais MCP.
    pub(crate) fn list_wake_schedules(
        &self,
        owner: Option<&str>,
    ) -> Result<Vec<WakeSchedule>, StoreError> {
        let mut statement=self.conn.prepare("SELECT payload FROM wake_schedules WHERE (?1 IS NULL OR owner=?1) ORDER BY next_at,schedule_id LIMIT 128").map_err(StoreError::Sqlite)?;
        let rows = statement
            .query_map([owner], |r| r.get::<_, String>(0))
            .map_err(StoreError::Sqlite)?;
        rows.map(|row| decode(&row.map_err(StoreError::Sqlite)?))
            .collect()
    }
    pub(crate) fn wake_schedule(&self, id: &str) -> Result<Option<WakeSchedule>, StoreError> {
        self.conn
            .query_row(
                "SELECT payload FROM wake_schedules WHERE schedule_id=?1",
                [id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(StoreError::Sqlite)?
            .map(|p| decode(&p))
            .transpose()
    }
    /// O(log128+B). Annule le futur ; n'invente aucun résultat d'une remise.
    pub(crate) fn cancel_wake_schedule(
        &self,
        owner: Option<&str>,
        id: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let payload: String = tx
            .query_row(
                "SELECT payload FROM wake_schedules WHERE schedule_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(StoreError::Sqlite)?
            .ok_or(StoreError::Invariant("target_unknown"))?;
        let mut schedule = decode(&payload)?;
        if owner.is_some_and(|owner| owner != schedule.owner) {
            return Err(StoreError::Invariant("not_owner"));
        }
        if schedule.cancelled_at.is_some() {
            return Ok(false);
        }
        schedule.cancelled_at = Some(now);
        let payload = serde_json::to_string(&schedule)
            .map_err(|_| StoreError::Invariant("invalid_request"))?;
        tx.execute(
            "UPDATE wake_schedules SET cancelled_at=?1,payload=?2 WHERE schedule_id=?3",
            params![now, payload, id],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(true)
    }
    /// O(log128+B). Avancement sans gel : saut ancien ou nouvelle zone future.
    pub(crate) fn advance_wake_calendar(
        &self,
        id: &str,
        expected_next_at: i64,
        calculated_tz: &str,
        current_tz: &str,
        next_at: i64,
        next_calendar_date: Option<&str>,
        now: i64,
    ) -> Result<bool, StoreError> {
        if calculated_tz != current_tz || next_at <= now {
            return Ok(false);
        }
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let payload:Option<String>=tx.query_row("SELECT payload FROM wake_schedules WHERE schedule_id=?1 AND next_at=?2 AND cancelled_at IS NULL",params![id,expected_next_at],|r|r.get(0)).optional().map_err(StoreError::Sqlite)?;
        let Some(payload) = payload else {
            return Ok(false);
        };
        let mut schedule = decode(&payload)?;
        if schedule
            .occurrence
            .as_ref()
            .is_some_and(|w| !w.phase.terminal())
        {
            return Ok(false);
        }
        if matches!(
            &schedule.cadence,
            bridget_transport::protocol::WakeCadence::At { .. }
        ) && next_calendar_date.is_none()
        {
            return Err(StoreError::Invariant("calendar_unavailable"));
        }
        schedule.next_at = next_at;
        schedule.next_calendar_date = next_calendar_date.map(str::to_owned);
        schedule.timezone = current_tz.into();
        let payload = serde_json::to_string(&schedule)
            .map_err(|_| StoreError::Invariant("invalid_request"))?;
        let changed=tx.execute("UPDATE wake_schedules SET next_at=?1,payload=?2 WHERE schedule_id=?3 AND next_at=?4 AND cancelled_at IS NULL",params![next_at,payload,id,expected_next_at]).map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }
    /// O(log128+B). CAS de l'échéance ET du canon ; fingerprint observé au puits.
    pub(crate) fn freeze_wake_occurrence(
        &self,
        id: &str,
        expected_next_at: i64,
        calculated_tz: &str,
        current_tz: &str,
        wake: &FrozenWake,
        next_at: i64,
        calendar_date: Option<&str>,
        next_calendar_date: Option<&str>,
    ) -> Result<bool, StoreError> {
        if calculated_tz != current_tz {
            return Ok(false);
        }
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let payload:Option<String>=tx.query_row("SELECT payload FROM wake_schedules WHERE schedule_id=?1 AND next_at=?2 AND cancelled_at IS NULL",params![id,expected_next_at],|r|r.get(0)).optional().map_err(StoreError::Sqlite)?;
        let Some(payload) = payload else {
            return Ok(false);
        };
        let mut schedule = decode(&payload)?;
        if schedule
            .occurrence
            .as_ref()
            .is_some_and(|w| !w.phase.terminal())
        {
            return Ok(false);
        }
        if wake.source != crate::wake_schedules::WakeSource::Scheduled
            || wake.owner != schedule.owner
            || wake.target != schedule.target
            || wake.owner_project != schedule.owner_project
            || wake.owner_native != schedule.owner_native
            || wake.label != schedule.label
            || wake.due_at < expected_next_at
            || next_at <= wake.due_at
            || wake.phase != WakePhase::Prepared
            || wake.id != format!("wake:{}:{}", id, wake.due_at)
        {
            return Err(StoreError::Invariant("invalid_request"));
        }
        let mut frozen = wake.clone();
        frozen.freeze_canonical().map_err(StoreError::Invariant)?;
        if !wake.canonical_bytes.is_empty() && frozen.canonical_bytes != wake.canonical_bytes {
            return Err(StoreError::Invariant("envelope_mismatch"));
        }
        schedule.occurrence = Some(frozen);
        if matches!(
            &schedule.cadence,
            bridget_transport::protocol::WakeCadence::At { .. }
        ) && next_calendar_date.is_none()
        {
            return Err(StoreError::Invariant("calendar_unavailable"));
        }
        schedule.next_at = next_at;
        schedule.next_calendar_date = next_calendar_date.map(str::to_owned);
        schedule.timezone = current_tz.into();
        if let Some(day) = calendar_date {
            schedule.previous_calendar_date = Some(day.into());
        }
        let payload = serde_json::to_string(&schedule)
            .map_err(|_| StoreError::Invariant("invalid_request"))?;
        let changed=tx.execute("UPDATE wake_schedules SET next_at=?1,payload=?2 WHERE schedule_id=?3 AND next_at=?4 AND cancelled_at IS NULL",params![next_at,payload,id,expected_next_at]).map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }
    /// O(log128+B). Seule la preuve courante peut avancer la phase et next_at.
    pub(crate) fn settle_wake_occurrence(
        &self,
        id: &str,
        key: &str,
        phase: WakePhase,
        next_at: i64,
        next_calendar_date: Option<&str>,
    ) -> Result<bool, StoreError> {
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let payload: Option<String> = tx
            .query_row(
                "SELECT payload FROM wake_schedules WHERE schedule_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        let Some(payload) = payload else {
            return Ok(false);
        };
        let mut schedule = decode(&payload)?;
        let Some(wake) = schedule.occurrence.as_mut() else {
            return Ok(false);
        };
        if wake.id != key || wake.phase.terminal() {
            return Ok(false);
        }
        if phase == WakePhase::Prepared
            || (wake.phase == WakePhase::DispatchingAgent
                && !matches!(phase, WakePhase::Confirmed | WakePhase::OutcomeUnknown))
        {
            return Err(StoreError::Invariant("invalid_request"));
        }
        wake.phase = phase;
        if wake.phase.terminal() {
            if next_at <= wake.due_at || next_at < schedule.next_at {
                return Err(StoreError::Invariant("invalid_request"));
            }
            if matches!(
                &schedule.cadence,
                bridget_transport::protocol::WakeCadence::At { .. }
            ) && next_calendar_date.is_none()
            {
                return Err(StoreError::Invariant("calendar_unavailable"));
            }
            schedule.next_at = next_at;
            schedule.next_calendar_date = next_calendar_date.map(str::to_owned);
        }
        let payload = serde_json::to_string(&schedule)
            .map_err(|_| StoreError::Invariant("invalid_request"))?;
        tx.execute(
            "UPDATE wake_schedules SET next_at=?1,payload=?2 WHERE schedule_id=?3",
            params![schedule.next_at, payload, id],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wake_schedules::{WakeSource, calculate_cadence};
    use bridget_transport::protocol::{WakeCadence, WakeTarget};
    #[test]
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
}
