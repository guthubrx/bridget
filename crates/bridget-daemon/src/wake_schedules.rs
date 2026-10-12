//! Domaine des réveils : occurrence figée et calendrier civil borné.
use bridget_transport::protocol::{CommunicationProject, WakeCadence, WakeTarget};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::CStr;
use std::os::unix::fs::MetadataExt;
pub(crate) const CLI_WAKE_SCOPE: &str = "bridget_internal_cli_wake158B_v1";
pub(crate) const SCHEDULED_WAKE_SCOPE: &str = "bridget_internal_scheduled_wake158B_v1";
pub(crate) const MISSED_SUMMARY_SCOPE: &str = "bridget_internal_missed_summary158B_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WakeSource {
    Scheduled,
    CliOnce,
    MissedSummary,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WakePhase {
    Prepared,
    Deferred,
    DispatchingAgent,
    CommittedThread,
    Confirmed,
    OutcomeUnknown,
    MissedBeforeDeparture,
    OwnerUnavailable,
}
impl WakePhase {
    pub(crate) fn terminal(&self) -> bool {
        matches!(
            self,
            Self::CommittedThread
                | Self::Confirmed
                | Self::OutcomeUnknown
                | Self::MissedBeforeDeparture
                | Self::OwnerUnavailable
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeWakeOrigin {
    pub task_id: String,
    pub instance_id: String,
    pub mission: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FrozenWake {
    pub version: u16,
    pub source: WakeSource,
    pub target: WakeTarget,
    pub owner: String,
    pub owner_project: Option<CommunicationProject>,
    pub owner_native: Option<NativeWakeOrigin>,
    pub due_at: i64,
    pub issued_at: i64,
    pub grace_until: i64,
    pub id: String,
    pub label: String,
    pub body: String,
    #[serde(skip)]
    pub canonical_bytes: Vec<u8>,
    pub thread_operation_id: Option<String>,
    pub delivery_id: Option<String>,
    pub phase: WakePhase,
}
impl FrozenWake {
    /// O(B). Ce canon est réservé au domaine CLI158B, jamais canonical_send.
    pub(crate) fn freeze_canonical(&mut self) -> Result<(), &'static str> {
        if self.version != 1
            || self.label.len() > 2048
            || self.grace_until != self.due_at.saturating_add(600)
        {
            return Err("invalid_request");
        }
        let mut immutable = self.clone();
        immutable.phase = WakePhase::Prepared;
        immutable.delivery_id = None;
        self.canonical_bytes = serde_json::to_vec(&immutable).map_err(|_| "invalid_request")?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WakeSchedule {
    pub schedule_id: String,
    pub owner: String,
    pub owner_project: Option<CommunicationProject>,
    pub owner_native: Option<NativeWakeOrigin>,
    pub target: WakeTarget,
    pub cadence: WakeCadence,
    pub label: String,
    pub anchor_at: i64,
    pub next_at: i64,
    pub timezone: String,
    pub next_calendar_date: Option<String>,
    pub previous_calendar_date: Option<String>,
    pub occurrence: Option<FrozenWake>,
    pub cancelled_at: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CalendarSnapshot {
    pub fingerprint: String,
    pub next_at: i64,
    pub due_at: Option<i64>,
    pub calendar_date: Option<String>,
    pub next_calendar_date: Option<String>,
}
/// O(B+j), j<=7 ; UUID et champs métier fermés, aucune résolution de cible.
pub(crate) fn validate_schedule(schedule: &WakeSchedule) -> Result<(), &'static str> {
    if uuid::Uuid::parse_str(&schedule.schedule_id).is_err()
        || schedule.owner.is_empty()
        || schedule.label.len() > 2048
    {
        return Err("invalid_request");
    }
    match &schedule.target {
        WakeTarget::SelfTarget {} => {}
        WakeTarget::Agent { id } | WakeTarget::Thread { id } => {
            if crate::threads::canonical_uuid(id).as_deref() != Some(id) {
                return Err("invalid_request");
            }
        }
    }
    match &schedule.cadence {
        WakeCadence::Interval { seconds } if *seconds >= 60 && *seconds <= i64::MAX as u64 => {
            Ok(())
        }
        WakeCadence::At { time, weekdays } => {
            let bytes = time.as_bytes();
            if bytes.len() != 5
                || bytes[2] != b':'
                || !bytes
                    .iter()
                    .enumerate()
                    .all(|(i, c)| i == 2 || c.is_ascii_digit())
            {
                return Err("invalid_request");
            }
            let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
            let minute = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
            if hour > 23
                || minute > 59
                || weekdays.is_empty()
                || weekdays.len() > 7
                || weekdays.iter().any(|d| !(1..=7).contains(d))
                || weekdays
                    .iter()
                    .enumerate()
                    .any(|(i, d)| weekdays[..i].contains(d))
            {
                Err("invalid_request")
            } else {
                Ok(())
            }
        }
        _ => Err("invalid_request"),
    }
}
fn local(epoch: i64) -> Result<libc::tm, &'static str> {
    let raw: libc::time_t = epoch.try_into().map_err(|_| "calendar_unavailable")?;
    let mut civil: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&raw, &mut civil) }.is_null() {
        return Err("calendar_unavailable");
    }
    if civil.tm_gmtoff.abs() > 86400 || civil.tm_gmtoff % 60 != 0 {
        return Err("calendar_unavailable");
    }
    Ok(civil)
}
/// O(Z), Z identifiant TZ serveur ; metadata hors verrou si TZ est absent.
pub(crate) fn timezone_fingerprint(now: i64) -> Result<String, &'static str> {
    let civil = local(now)?;
    if civil.tm_zone.is_null() {
        return Err("calendar_unavailable");
    }
    let zone = unsafe { CStr::from_ptr(civil.tm_zone) }
        .to_str()
        .map_err(|_| "calendar_unavailable")?;
    let configured = match std::env::var_os("TZ") {
        Some(value) => format!("env:{}", value.to_str().ok_or("calendar_unavailable")?),
        None => {
            let path = std::path::Path::new("/etc/localtime");
            let metadata = std::fs::symlink_metadata(path).map_err(|_| "calendar_unavailable")?;
            let target = if metadata.file_type().is_symlink() {
                std::fs::read_link(path).map_err(|_| "calendar_unavailable")?
            } else {
                path.to_path_buf()
            };
            let actual = std::fs::metadata(path).map_err(|_| "calendar_unavailable")?;
            format!(
                "file:{}:{}:{}:{}:{}:{}:{}:{}",
                target.to_str().ok_or("calendar_unavailable")?,
                metadata.ino(),
                metadata.mtime(),
                metadata.mtime_nsec(),
                actual.ino(),
                actual.mtime(),
                actual.mtime_nsec(),
                actual.len()
            )
        }
    };
    Ok(format!(
        "{}:{}:{:x}",
        zone,
        civil.tm_gmtoff,
        Sha256::digest(configured.as_bytes())
    ))
}
fn date(civil: &libc::tm) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        civil.tm_year + 1900,
        civil.tm_mon + 1,
        civil.tm_mday
    )
}
/// O(2881). Inverse civil exact ; couvre aussi les folds sans changement DST.
fn first_civil_epoch(day: &libc::tm, hour: i32, minute: i32) -> Result<Option<i64>, &'static str> {
    let mut utc = *day;
    utc.tm_hour = hour;
    utc.tm_min = minute;
    utc.tm_sec = 0;
    utc.tm_isdst = 0;
    let center = (unsafe { libc::timegm(&mut utc) }) as i64;
    for step in -1440i64..=1440 {
        let epoch = center
            .checked_add(step * 60)
            .ok_or("calendar_unavailable")?;
        let candidate = local(epoch)?;
        if candidate.tm_year == day.tm_year
            && candidate.tm_mon == day.tm_mon
            && candidate.tm_mday == day.tm_mday
            && candidate.tm_hour == hour
            && candidate.tm_min == minute
            && candidate.tm_sec == 0
        {
            return Ok(Some(epoch));
        }
    }
    Ok(None)
}
/// O(2881*8+B), hors verrou daemon ; l'appelant compare le fingerprint au CAS.
pub(crate) fn calculate_cadence(
    schedule: &WakeSchedule,
    now: i64,
) -> Result<CalendarSnapshot, &'static str> {
    validate_schedule(schedule)?;
    let fingerprint = timezone_fingerprint(now)?;
    match &schedule.cadence {
        WakeCadence::Interval { seconds } => {
            let step = *seconds as i64;
            let passed = now
                .checked_sub(schedule.anchor_at)
                .ok_or("calendar_unavailable")?;
            let last = schedule
                .anchor_at
                .checked_add(
                    passed
                        .div_euclid(step)
                        .checked_mul(step)
                        .ok_or("calendar_unavailable")?,
                )
                .ok_or("calendar_unavailable")?;
            let next = last.checked_add(step).ok_or("calendar_unavailable")?;
            Ok(CalendarSnapshot {
                fingerprint,
                next_at: next,
                due_at: (last >= schedule.next_at && last <= now).then_some(last),
                calendar_date: None,
                next_calendar_date: None,
            })
        }
        WakeCadence::At { time, weekdays } => {
            let hour = time[..2].parse::<i32>().map_err(|_| "invalid_request")?;
            let minute = time[3..].parse::<i32>().map_err(|_| "invalid_request")?;
            let today = local(now)?;
            let mut midnight = today;
            midnight.tm_hour = 0;
            midnight.tm_min = 0;
            midnight.tm_sec = 0;
            let base = (unsafe { libc::timegm(&mut midnight) }) as i64;
            let mut due = None;
            let mut due_date = None;
            let mut next = None;
            let mut next_date = None;
            // L'échéance déjà calculée reste due après minuit et après une
            // nouvelle zone ; seules les échéances futures sont recalculées.
            if schedule.next_at <= now && now.saturating_sub(schedule.next_at) <= 600 {
                if let Some(civil_date) = schedule.next_calendar_date.as_ref() {
                    if schedule.previous_calendar_date.as_deref() != Some(civil_date.as_str()) {
                        due = Some(schedule.next_at);
                        due_date = Some(civil_date.clone());
                    }
                }
            }
            for offset in 0..8 {
                let epoch = base
                    .checked_add(offset * 86400)
                    .ok_or("calendar_unavailable")?;
                let raw: libc::time_t = epoch.try_into().map_err(|_| "calendar_unavailable")?;
                let mut day: libc::tm = unsafe { std::mem::zeroed() };
                if unsafe { libc::gmtime_r(&raw, &mut day) }.is_null() {
                    return Err("calendar_unavailable");
                }
                let weekday = if day.tm_wday == 0 {
                    7
                } else {
                    day.tm_wday as u8
                };
                if !weekdays.contains(&weekday)
                    || schedule.previous_calendar_date.as_deref() == Some(date(&day).as_str())
                {
                    continue;
                }
                let Some(candidate) = first_civil_epoch(&day, hour, minute)? else {
                    continue;
                };
                if candidate <= now {
                    if candidate >= schedule.next_at
                        && now.saturating_sub(candidate) <= 600
                        && due.is_none()
                    {
                        due = Some(candidate);
                        due_date = Some(date(&day));
                    }
                } else {
                    next = Some(candidate);
                    next_date = Some(date(&day));
                    break;
                }
            }
            let next_at = next.ok_or("calendar_unavailable")?;
            if timezone_fingerprint(now)? != fingerprint {
                return Err("calendar_unavailable");
            }
            Ok(CalendarSnapshot {
                fingerprint,
                next_at,
                due_at: due,
                calendar_date: due_date,
                next_calendar_date: next_date,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wake158B_groupe1_domaine_ferme_et_bornes() {
        let mut s = WakeSchedule {
            schedule_id: uuid::Uuid::new_v4().to_string(),
            owner: "owner".into(),
            owner_project: None,
            owner_native: None,
            target: WakeTarget::SelfTarget {},
            cadence: WakeCadence::Interval { seconds: 60 },
            label: "é".repeat(1024),
            anchor_at: 0,
            next_at: 60,
            timezone: "UTC:0".into(),
            next_calendar_date: None,
            previous_calendar_date: None,
            occurrence: None,
            cancelled_at: None,
        };
        assert!(validate_schedule(&s).is_ok());
        s.label.push('a');
        assert!(validate_schedule(&s).is_err());
        s.label.clear();
        s.cadence = WakeCadence::Interval { seconds: 59 };
        assert!(validate_schedule(&s).is_err());
        for (time, days) in [
            ("24:00", vec![1]),
            ("09:60", vec![1]),
            ("09:00", vec![]),
            ("09:00", vec![1, 1]),
            ("09:00", vec![8]),
        ] {
            s.cadence = WakeCadence::At {
                time: time.into(),
                weekdays: days,
            };
            assert!(validate_schedule(&s).is_err());
        }
        let mut value = serde_json::to_value(&s).unwrap();
        value["from"] = serde_json::json!("forged");
        assert!(serde_json::from_value::<WakeSchedule>(value).is_err());
        let mut frozen = FrozenWake {
            version: 1,
            source: WakeSource::Scheduled,
            target: WakeTarget::SelfTarget {},
            owner: "owner".into(),
            owner_project: None,
            owner_native: None,
            due_at: 60,
            issued_at: 60,
            grace_until: 660,
            id: "wake:example:60".into(),
            label: "complet".into(),
            body: "complet".into(),
            canonical_bytes: Vec::new(),
            thread_operation_id: None,
            delivery_id: None,
            phase: WakePhase::Prepared,
        };
        frozen.freeze_canonical().unwrap();
        let original = frozen.canonical_bytes.clone();
        frozen.phase = WakePhase::OutcomeUnknown;
        frozen.delivery_id = Some("delivery".into());
        frozen.freeze_canonical().unwrap();
        assert_eq!(frozen.canonical_bytes, original);
        let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        value["free_owner"] = serde_json::json!("forged");
        assert!(serde_json::from_value::<FrozenWake>(value).is_err());
    }
    #[test]
    fn wake158B_groupe2_calendrier_trou_fold_tz_et_date() {
        if let Ok(case) = std::env::var("BRIDGET_WAKE_CALENDAR158B_CASE") {
            if case == "zone_identity" {
                let mut winter: libc::tm = unsafe { std::mem::zeroed() };
                winter.tm_year = 126;
                winter.tm_mon = 0;
                winter.tm_mday = 1;
                let epoch = (unsafe { libc::timegm(&mut winter) }) as i64;
                println!("WAKE158B_FP={}", timezone_fingerprint(epoch).unwrap());
                return;
            }
            let mut day: libc::tm = unsafe { std::mem::zeroed() };
            let (year, month, day_number, hour, minute, expected) = match case.as_str() {
                "utc" => (2026, 10, 12, 9, 0, Some((2026, 10, 12, 9, 0))),
                "gap" => (2026, 3, 29, 2, 30, None),
                "fold" => (2026, 10, 25, 2, 30, Some((2026, 10, 25, 0, 30))),
                "non_dst" => (2014, 10, 26, 1, 30, Some((2014, 10, 25, 21, 30))),
                _ => panic!("case inconnue"),
            };
            day.tm_year = year - 1900;
            day.tm_mon = month - 1;
            day.tm_mday = day_number;
            let before = std::time::Instant::now();
            let actual = first_civil_epoch(&day, hour, minute).unwrap();
            eprintln!("calendrier158B {case}: {:?}", before.elapsed());
            let expected = expected.map(|(y, m, d, h, min)| {
                let mut utc: libc::tm = unsafe { std::mem::zeroed() };
                utc.tm_year = y - 1900;
                utc.tm_mon = m - 1;
                utc.tm_mday = d;
                utc.tm_hour = h;
                utc.tm_min = min;
                (unsafe { libc::timegm(&mut utc) }) as i64
            });
            assert_eq!(actual, expected);
            if case == "utc" {
                let now = actual.unwrap();
                let day = date(&local(now).unwrap());
                let mut s = WakeSchedule {
                    schedule_id: uuid::Uuid::new_v4().to_string(),
                    owner: "owner".into(),
                    owner_project: None,
                    owner_native: None,
                    target: WakeTarget::SelfTarget {},
                    cadence: WakeCadence::At {
                        time: "09:00".into(),
                        weekdays: vec![1, 2, 3, 4, 5, 6, 7],
                    },
                    label: String::new(),
                    anchor_at: now,
                    next_at: now,
                    timezone: "ancienne-zone".into(),
                    next_calendar_date: None,
                    previous_calendar_date: None,
                    occurrence: None,
                    cancelled_at: None,
                };
                let first = calculate_cadence(&s, now).unwrap();
                assert_eq!(first.due_at, Some(now));
                assert_eq!(first.calendar_date, Some(day.clone()));
                assert_ne!(first.fingerprint, s.timezone);
                assert!(first.next_at > now);
                s.previous_calendar_date = Some(day);
                assert_eq!(calculate_cadence(&s, now).unwrap().due_at, None);
                let mut midnight: libc::tm = unsafe { std::mem::zeroed() };
                midnight.tm_year = 126;
                midnight.tm_mon = 9;
                midnight.tm_mday = 13;
                midnight.tm_hour = 0;
                midnight.tm_min = 1;
                let after_midnight = (unsafe { libc::timegm(&mut midnight) }) as i64;
                s.cadence = WakeCadence::At {
                    time: "23:59".into(),
                    weekdays: vec![1, 2, 3, 4, 5, 6, 7],
                };
                s.next_at = after_midnight - 120;
                s.next_calendar_date = Some("2026-10-12".into());
                s.previous_calendar_date = None;
                let catchup = calculate_cadence(&s, after_midnight).unwrap();
                assert_eq!(catchup.due_at, Some(after_midnight - 120));
                assert_eq!(catchup.calendar_date, Some("2026-10-12".into()));
                assert!(catchup.next_at > after_midnight);
                assert_eq!(
                    calculate_cadence(&s, after_midnight + 481).unwrap().due_at,
                    None
                );
            }
            return;
        }
        for (case, zone) in [
            ("utc", "UTC"),
            ("gap", "Europe/Paris"),
            ("fold", "Europe/Paris"),
            ("non_dst", "Europe/Moscow"),
        ] {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "wake_schedules::tests::wake158B_groupe2_calendrier_trou_fold_tz_et_date",
                    "--nocapture",
                ])
                .env("TZ", zone)
                .env("BRIDGET_WAKE_CALENDAR158B_CASE", case)
                .status()
                .unwrap();
            assert!(status.success(), "{case}");
        }
        let mut fingerprints = Vec::new();
        for zone in ["Europe/Paris", "Africa/Algiers"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "wake_schedules::tests::wake158B_groupe2_calendrier_trou_fold_tz_et_date",
                    "--nocapture",
                ])
                .env("TZ", zone)
                .env("BRIDGET_WAKE_CALENDAR158B_CASE", "zone_identity")
                .output()
                .unwrap();
            assert!(output.status.success());
            let stdout = String::from_utf8(output.stdout).unwrap();
            let fp = stdout
                .lines()
                .find_map(|line| line.strip_prefix("WAKE158B_FP="))
                .unwrap();
            assert!(fp.starts_with("CET:3600:"));
            fingerprints.push(fp.to_string());
        }
        assert_ne!(fingerprints[0], fingerprints[1]);
    }
}
