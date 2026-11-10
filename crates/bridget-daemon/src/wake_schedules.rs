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
    #[allow(clippy::useless_conversion)] // time_t varie selon la plateforme
    let raw: libc::time_t = epoch.try_into().map_err(|_| "calendar_unavailable")?;
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    let mut civil: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: les pointeurs passés à localtime_r désignent des variables locales valides pendant l'appel.
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
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
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
    // SAFETY: les pointeurs passés à timegm désignent des variables locales valides pendant l'appel.
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
            // SAFETY: les pointeurs passés à timegm désignent des variables locales valides pendant l'appel.
            let base = (unsafe { libc::timegm(&mut midnight) }) as i64;
            let mut due = None;
            let mut due_date = None;
            let mut next = None;
            let mut next_date = None;
            // L'échéance déjà calculée reste due après minuit et après une
            // nouvelle zone ; seules les échéances futures sont recalculées.
            if schedule.next_at <= now && now.saturating_sub(schedule.next_at) <= 600
                && let Some(civil_date) = schedule.next_calendar_date.as_ref()
                    && schedule.previous_calendar_date.as_deref() != Some(civil_date.as_str()) {
                        due = Some(schedule.next_at);
                        due_date = Some(civil_date.clone());
                    }
            for offset in 0..8 {
                let epoch = base
                    .checked_add(offset * 86400)
                    .ok_or("calendar_unavailable")?;
                #[allow(clippy::useless_conversion)] // time_t varie selon la plateforme
                let raw: libc::time_t = epoch.try_into().map_err(|_| "calendar_unavailable")?;
                // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                let mut day: libc::tm = unsafe { std::mem::zeroed() };
                // SAFETY: les pointeurs passés à gmtime_r désignent des variables locales valides pendant l'appel.
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
#[path = "wake_schedules_tests.rs"]
mod tests;
