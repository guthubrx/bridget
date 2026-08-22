//! Journal JSONL versionné des sessions ACP.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SessionJournal {
    directory: PathBuf,
    session_id: String,
    next_seq: u64,
}

impl SessionJournal {
    pub fn new(root: impl AsRef<Path>, agent: &str, session_id: &str) -> std::io::Result<Self> {
        let root = root.as_ref();
        create_private_dir(root)?;
        let directory = root.join(agent);
        create_private_dir(&directory)?;
        Ok(Self {
            next_seq: last_sequence(&directory).saturating_add(1),
            directory,
            session_id: session_id.to_string(),
        })
    }

    pub fn append(&mut self, event: &str, message_id: Option<&str>, payload: Value) -> std::io::Result<u64> {
        let (date, timestamp) = now_date_and_timestamp();
        self.append_at(&date, &timestamp, event, message_id, payload)
    }

    pub fn append_at(&mut self, date: &str, timestamp: &str, event: &str, message_id: Option<&str>, payload: Value) -> std::io::Result<u64> {
        let sequence = self.next_seq;
        let entry = json!({
            "v": 1, "seq": sequence, "ts": timestamp, "session_id": self.session_id,
            "event": event, "message_id": message_id, "payload": payload,
        });
        let path = self.directory.join(format!("{date}.jsonl"));
        let mut file = OpenOptions::new().create(true).append(true).mode(0o600).open(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        writeln!(file, "{entry}")?;
        file.flush()?;
        self.next_seq = sequence.saturating_add(1);
        Ok(sequence)
    }
}

pub fn valid_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path).ok().into_iter()
        .flat_map(|content| content.lines().map(str::to_owned).collect::<Vec<_>>())
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .filter(|value| value.get("v").and_then(Value::as_u64) == Some(1))
        .collect()
}

fn create_private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

fn last_sequence(directory: &Path) -> u64 {
    fs::read_dir(directory).ok().into_iter().flatten().filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .flat_map(|entry| valid_events(&entry.path()))
        .filter_map(|value| value.get("seq").and_then(Value::as_u64)).max().unwrap_or(0)
}

fn now_date_and_timestamp() -> (String, String) {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let (year, month, day, hour, minute, second) = civil_time(seconds);
    (format!("{year:04}-{month:02}-{day:02}"), format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"))
}

fn civil_time(seconds: i64) -> (i64, u32, u32, i64, i64, i64) {
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year, month as u32, day as u32, day_seconds / 3600, day_seconds / 60 % 60, day_seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(name: &str) -> PathBuf { std::env::temp_dir().join(format!("bridget-journal-{name}-{}", std::process::id())) }

    #[test]
    fn sequence_survives_rotation_and_payload_is_versioned() {
        let root = root("rotation");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        assert_eq!(journal.append_at("2026-08-22", "2026-08-22T00:00:00Z", "turn_start", Some("m1"), json!({"from":"alice","reply":true,"body":"bonjour"})).unwrap(), 1);
        assert_eq!(journal.append_at("2026-08-23", "2026-08-23T00:00:00Z", "turn_end", Some("m1"), json!({"stop_reason":"end_turn","routed_to":"alice"})).unwrap(), 2);
        let event = valid_events(&root.join("codex-1/2026-08-23.jsonl")).pop().unwrap();
        assert_eq!(event["v"], 1);
        assert_eq!(event["seq"], 2);
        assert_eq!(event["payload"]["routed_to"], "alice");
        assert_eq!(fs::metadata(&root).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(root.join("codex-1/2026-08-23.jsonl")).unwrap().permissions().mode() & 0o777, 0o600);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn next_sequence_recovers_from_normal_empty_partial_and_corrupt_tails() {
        for (name, contents, expected) in [
            ("normal", "{\"v\":1,\"seq\":4}\n", 5), ("empty", "", 1),
            ("partial", "{\"v\":1,\"seq\":4}\n{\"v\":1,\"seq\":", 5),
            ("corrupt", "{\"v\":1,\"seq\":4}\nnot-json\n", 5),
        ] {
            let root = root(name);
            let directory = root.join("codex-1");
            create_private_dir(&directory).unwrap();
            fs::write(directory.join("2026-08-22.jsonl"), contents).unwrap();
            let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
            assert_eq!(journal.append_at("2026-08-23", "2026-08-23T00:00:00Z", "error", None, json!({"reason":"test"})).unwrap(), expected);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn versioned_reader_fixtures_skip_partial_and_corrupt_lines() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        assert_eq!(valid_events(&fixtures.join("complete-turn.jsonl")).len(), 3);
        assert_eq!(valid_events(&fixtures.join("error.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("permission.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("rotation.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("partial-tail.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("corrupt-line.jsonl")).len(), 1);
    }
}
