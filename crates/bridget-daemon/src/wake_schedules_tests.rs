use super::*;
#[test]
#[allow(non_snake_case)] // nom des filtres documentés de la recette 158B/159
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
#[allow(non_snake_case)] // nom des filtres documentés de la recette 158B/159
fn wake158B_groupe2_calendrier_trou_fold_tz_et_date() {
    if let Ok(case) = std::env::var("BRIDGET_WAKE_CALENDAR158B_CASE") {
        if case == "zone_identity" {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            let mut winter: libc::tm = unsafe { std::mem::zeroed() };
            winter.tm_year = 126;
            winter.tm_mon = 0;
            winter.tm_mday = 1;
            // SAFETY: les pointeurs passés à timegm désignent des variables locales valides pendant l'appel.
            let epoch = (unsafe { libc::timegm(&mut winter) }) as i64;
            println!("WAKE158B_FP={}", timezone_fingerprint(epoch).unwrap());
            return;
        }
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
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
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            let mut utc: libc::tm = unsafe { std::mem::zeroed() };
            utc.tm_year = y - 1900;
            utc.tm_mon = m - 1;
            utc.tm_mday = d;
            utc.tm_hour = h;
            utc.tm_min = min;
            // SAFETY: les pointeurs passés à timegm désignent des variables locales valides pendant l'appel.
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
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            let mut midnight: libc::tm = unsafe { std::mem::zeroed() };
            midnight.tm_year = 126;
            midnight.tm_mon = 9;
            midnight.tm_mday = 13;
            midnight.tm_hour = 0;
            midnight.tm_min = 1;
            // SAFETY: les pointeurs passés à timegm désignent des variables locales valides pendant l'appel.
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
