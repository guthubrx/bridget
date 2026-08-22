//! Journal JSONL versionné des sessions ACP.

use serde::Serialize;
use serde_json::Value;
use crate::acp::AcpEvent;
use crate::protocol::AttachWindow;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const WRITER_QUEUE_CAPACITY: usize = 256;
const MAX_INCREMENTAL_LINE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct JournalEntry {
    v: u8,
    seq: u64,
    ts: String,
    session_id: String,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_id: Option<String>,
    payload: Value,
}

impl JournalEntry {
    fn new(sequence: u64, timestamp: &str, session_id: &str, event: &str, message_id: Option<&str>, payload: Value) -> Self {
        Self {
            v: 1,
            seq: sequence,
            ts: timestamp.to_string(),
            session_id: session_id.to_string(),
            event: event.to_string(),
            message_id: message_id.map(str::to_owned),
            payload,
        }
    }
}

enum WriterCommand {
    Entry(JournalEntry),
    Stop(mpsc::Sender<()>),
}

/// Propriétaire unique des E/S du journal. Les threads ACP n'y déposent que
/// des événements, afin qu'un disque lent ne bloque jamais stdout ou un tour.
#[derive(Clone)]
pub struct JournalWriter {
    sender: mpsc::SyncSender<WriterCommand>,
    failure: Arc<Mutex<Option<String>>>,
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl JournalWriter {
    pub fn start(
        root: impl AsRef<Path>,
        agent: &str,
        session_id: &str,
        events: Arc<Mutex<std::collections::VecDeque<AcpEvent>>>,
    ) -> std::io::Result<Self> {
        let mut journal = SessionJournal::new(root, agent, session_id)?;
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let failure = Arc::new(Mutex::new(None));
        let thread_failure = failure.clone();
        let handle = thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                match command {
                    WriterCommand::Entry(entry) => {
                        if let Err(error) = journal.append_entry(entry) {
                            let detail = format!("écriture du journal ACP impossible: {error}");
                            *thread_failure.lock().unwrap_or_else(|poison| poison.into_inner()) = Some(detail.clone());
                            events
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner())
                                .push_back(AcpEvent::JournalFailed { detail });
                            break;
                        }
                    }
                    WriterCommand::Stop(done) => {
                        let _ = done.send(());
                        break;
                    }
                }
            }
        });
        Ok(Self {
            sender,
            failure,
            handle: Arc::new(Mutex::new(Some(handle))),
        })
    }

    pub fn enqueue(&self, event: &str, message_id: Option<&str>, payload: Value) -> Result<(), String> {
        if let Some(error) = self.failure.lock().unwrap_or_else(|poison| poison.into_inner()).clone() {
            return Err(error);
        }
        let (_, timestamp) = now_date_and_timestamp();
        let entry = JournalEntry::new(0, &timestamp, "", event, message_id, payload);
        self.sender.try_send(WriterCommand::Entry(entry)).map_err(|error| match error {
            mpsc::TrySendError::Full(_) => "journal ACP saturé".to_string(),
            mpsc::TrySendError::Disconnected(_) => self.failure.lock().unwrap_or_else(|poison| poison.into_inner()).clone().unwrap_or_else(|| "journal ACP arrêté".to_string()),
        })
    }

    pub fn stop(&self) {
        let Some(handle) = self.handle.lock().unwrap_or_else(|poison| poison.into_inner()).take() else {
            return;
        };
        let (done_sender, done_receiver) = mpsc::channel();
        if self.sender.send(WriterCommand::Stop(done_sender)).is_ok() {
            let _ = done_receiver.recv();
        }
        let _ = handle.join();
    }

    #[cfg(test)]
    pub(crate) fn saturated_for_test() -> Self {
        let (sender, receiver) = mpsc::sync_channel(0);
        // Le récepteur reste volontairement vivant mais ne lit jamais :
        // try_send doit donc échouer immédiatement avec Full.
        std::mem::forget(receiver);
        Self {
            sender,
            failure: Arc::new(Mutex::new(None)),
            handle: Arc::new(Mutex::new(None)),
        }
    }
}

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
        let entry = JournalEntry::new(sequence, timestamp, &self.session_id, event, message_id, payload);
        self.append_entry_at(date, entry)?;
        self.next_seq = sequence.saturating_add(1);
        Ok(sequence)
    }

    fn append_entry(&mut self, mut entry: JournalEntry) -> std::io::Result<()> {
        let (date, timestamp) = now_date_and_timestamp();
        entry.seq = self.next_seq;
        entry.ts = timestamp;
        entry.session_id.clone_from(&self.session_id);
        self.append_entry_at(&date, entry)?;
        self.next_seq = self.next_seq.saturating_add(1);
        Ok(())
    }

    fn append_entry_at(&mut self, date: &str, entry: JournalEntry) -> std::io::Result<()> {
        let path = self.directory.join(format!("{date}.jsonl"));
        isolate_partial_tail(&path)?;
        let mut file = OpenOptions::new().create(true).append(true).mode(0o600).open(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        writeln!(file, "{}", serde_json::to_string(&entry)?)?;
        file.flush()?;
        Ok(())
    }
}

pub fn valid_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path).ok().into_iter()
        .flat_map(|content| content.lines().map(str::to_owned).collect::<Vec<_>>())
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .filter(|value| value.get("v").and_then(Value::as_u64) == Some(1))
        .collect()
}

/// Une ligne v1 lisible, accompagnée de sa position dans le fichier source.
#[derive(Debug, Clone, PartialEq)]
pub struct JournalReadEvent {
    pub seq: u64,
    pub offset: u64,
    pub line: u64,
    pub bytes: Vec<u8>,
}

/// Diagnostic non bloquant d'une ligne qui ne peut pas participer au flux.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalUnreadableLine {
    pub line: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JournalReadItem {
    Event(JournalReadEvent),
    Unreadable(JournalUnreadableLine),
    Oversized { seq: Option<u64>, offset: u64, line: u64 },
}

/// Lecteur incrémental : sa mémoire est bornée par une tranche plus la seule
/// ligne finale incomplète. Celle-ci n'est jamais interprétée avant son newline.
pub struct IncrementalJournalReader {
    path: PathBuf,
    next_offset: u64,
    next_line: u64,
    pending_offset: u64,
    pending: Vec<u8>,
    discarding: Option<(u64, u64, Option<u64>)>,
}

impl IncrementalJournalReader {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            next_offset: 0,
            next_line: 1,
            pending_offset: 0,
            pending: Vec::new(),
            discarding: None,
        }
    }

    pub fn next_offset(&self) -> u64 { self.next_offset }
    #[cfg(test)]
    fn buffered_len(&self) -> usize { self.pending.len() }

    /// Lit au plus `max_bytes` octets nouveaux. Une queue partielle est gardée
    /// pour l'appel suivant et ne produit donc jamais un faux événement.
    pub fn read_chunk(&mut self, max_bytes: usize) -> std::io::Result<Vec<JournalReadItem>> {
        assert!(max_bytes > 0, "une tranche de journal doit être non nulle");
        let mut file = match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        file.seek(SeekFrom::Start(self.next_offset))?;
        let mut chunk = vec![0; max_bytes];
        let count = file.read(&mut chunk)?;
        chunk.truncate(count);
        self.next_offset = self.next_offset.saturating_add(count as u64);
        if let Some((_offset, _line, _seq)) = self.discarding {
            if let Some(end) = chunk.iter().position(|byte| *byte == b'\n') {
                self.pending_offset = self.next_offset.saturating_sub(count as u64).saturating_add((end + 1) as u64);
                self.next_line = self.next_line.saturating_add(1);
                self.discarding = None;
                self.pending.extend_from_slice(&chunk[end + 1..]);
            } else { return Ok(Vec::new()); }
        } else { self.pending.extend_from_slice(&chunk); }

        let mut items = Vec::new();
        if self.pending.len() > MAX_INCREMENTAL_LINE_BYTES {
            let offset = self.pending_offset;
            let line = self.next_line;
            let seq = sequence_prefix(&self.pending);
            items.push(JournalReadItem::Oversized { seq, offset, line });
            if let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
                self.pending.drain(..=end);
                self.pending_offset = self.pending_offset.saturating_add((end + 1) as u64);
                self.next_line = self.next_line.saturating_add(1);
            } else {
                self.pending.clear();
                self.discarding = Some((offset, line, seq));
                return Ok(items);
            }
        }
        while let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
            let mut line = self.pending.drain(..=end).collect::<Vec<_>>();
            line.pop();
            let offset = self.pending_offset;
            let line_number = self.next_line;
            self.pending_offset = self.pending_offset.saturating_add((end + 1) as u64);
            self.next_line = self.next_line.saturating_add(1);
            match serde_json::from_slice::<Value>(&line) {
                Ok(value) if value.get("v").and_then(Value::as_u64) == Some(1) => {
                    match value.get("seq").and_then(Value::as_u64) {
                        Some(seq) => items.push(JournalReadItem::Event(JournalReadEvent {
                            seq, offset, line: line_number, bytes: line,
                        })),
                        None => items.push(JournalReadItem::Unreadable(JournalUnreadableLine {
                            line: line_number, offset,
                        })),
                    }
                }
                _ => items.push(JournalReadItem::Unreadable(JournalUnreadableLine {
                    line: line_number, offset,
                })),
            }
        }
        Ok(items)
    }
}

fn sequence_prefix(bytes: &[u8]) -> Option<u64> {
    let prefix = &bytes[..bytes.len().min(1024)];
    let marker = b"\"seq\":";
    let start = prefix.windows(marker.len()).position(|window| window == marker)? + marker.len();
    let end = prefix[start..].iter().position(|byte| !byte.is_ascii_digit()).unwrap_or(prefix.len() - start);
    std::str::from_utf8(&prefix[start..start + end]).ok()?.parse().ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalWindowError {
    InvalidDate,
    FutureDate,
    DateOutsideRetention,
}

/// Fenêtre résolue par l'hôte du wrapper. `from_seq` est inclusif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedJournalWindow {
    pub files: Vec<PathBuf>,
    pub from_seq: Option<u64>,
}

/// Résout `Today` et `Date` dans le calendrier de l'hôte du wrapper, passé
/// explicitement par l'appelant afin de ne jamais emprunter le fuseau du client.
pub fn resolve_window(
    directory: &Path,
    window: &AttachWindow,
    host_today: &str,
) -> Result<ResolvedJournalWindow, JournalWindowError> {
    let mut files = fs::read_dir(directory)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "jsonl"))
        .collect::<Vec<_>>();
    files.sort();
    match window {
        AttachWindow::Seq(seq) => Ok(ResolvedJournalWindow { files, from_seq: Some(*seq) }),
        AttachWindow::Today => Ok(ResolvedJournalWindow {
            files: files.into_iter().filter(|path| file_date(path) == Some(host_today)).collect(),
            from_seq: None,
        }),
        AttachWindow::Date(date) => {
            if !is_date(date) { return Err(JournalWindowError::InvalidDate); }
            if date.as_str() > host_today { return Err(JournalWindowError::FutureDate); }
            let selected = files.into_iter().filter(|path| file_date(path) == Some(date)).collect::<Vec<_>>();
            if selected.is_empty() { return Err(JournalWindowError::DateOutsideRetention); }
            Ok(ResolvedJournalWindow { files: selected, from_seq: None })
        }
    }
}

fn file_date(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|stem| stem.to_str())
}

fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
        && value[5..7].parse::<u8>().is_ok_and(|month| (1..=12).contains(&month))
        && value[8..10].parse::<u8>().is_ok_and(|day| (1..=31).contains(&day))
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

/// Empêche qu'un nouvel événement soit concaténé à une queue incomplète issue
/// d'un crash. Le fragment reste lisible comme ligne invalide, sans corrompre
/// l'événement appendé qui le suit.
fn isolate_partial_tail(path: &Path) -> std::io::Result<()> {
    let Ok(mut file) = OpenOptions::new().read(true).write(true).open(path) else {
        return Ok(());
    };
    let length = file.metadata()?.len();
    if length != 0 {
        file.seek(SeekFrom::End(-1))?;
        let mut last_byte = [0];
        file.read_exact(&mut last_byte)?;
        if last_byte[0] == b'\n' {
            return Ok(());
        }
        file.seek(SeekFrom::End(0))?;
        file.write_all(b"\n")?;
        file.flush()?;
    }
    Ok(())
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
    use serde_json::json;

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
    fn partial_tail_is_isolated_before_a_same_day_append() {
        let root = root("same-day-tail");
        let directory = root.join("codex-1");
        create_private_dir(&directory).unwrap();
        let path = directory.join("2026-08-22.jsonl");
        fs::write(&path, "{\"v\":1,\"seq\":4}\n{\"v\":1,\"seq\":").unwrap();
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        assert_eq!(journal.append_at("2026-08-22", "2026-08-22T01:00:00Z", "error", None, json!({"reason":"reprise"})).unwrap(), 5);
        let entries = valid_events(&path);
        assert_eq!(entries.iter().map(|entry| entry["seq"].as_u64()).collect::<Vec<_>>(), vec![Some(4), Some(5)]);
        assert_eq!(entries[1]["payload"]["reason"], "reprise");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writer_omits_an_absent_message_id() {
        let root = root("optional-message-id");
        let writer = JournalWriter::start(&root, "codex-1", "session-1", Arc::new(Mutex::new(std::collections::VecDeque::new()))).unwrap();
        writer.enqueue("error", None, json!({"reason":"global"})).unwrap();
        writer.stop();
        let path = std::fs::read_dir(root.join("codex-1")).unwrap().next().unwrap().unwrap().path();
        let event = valid_events(&path).pop().unwrap();
        assert!(event.get("message_id").is_none());
        assert_eq!(event["payload"], json!({"reason":"global"}));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writer_emits_a_terminal_event_when_an_append_fails() {
        let root = root("write-failure");
        let events = Arc::new(Mutex::new(std::collections::VecDeque::new()));
        let writer = JournalWriter::start(&root, "codex-1", "session-1", events.clone()).unwrap();
        fs::remove_dir_all(root.join("codex-1")).unwrap();
        writer.enqueue("error", None, json!({"reason":"test"})).unwrap();
        for _ in 0..20 {
            if events.lock().unwrap().iter().any(|event| matches!(event, AcpEvent::JournalFailed { .. })) {
                writer.stop();
                fs::remove_dir_all(root).unwrap();
                return;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("l'échec d'écriture du journal n'est pas devenu terminal");
    }

    #[test]
    fn versioned_reader_fixtures_skip_partial_and_corrupt_lines() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        assert_eq!(valid_events(&fixtures.join("complete-turn.jsonl")).len(), 3);
        assert_eq!(valid_events(&fixtures.join("error.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("permission.jsonl")).len(), 1);
        let rotation = fixtures.join("rotation");
        let mut rotated = std::fs::read_dir(rotation).unwrap()
            .filter_map(Result::ok).flat_map(|entry| valid_events(&entry.path()))
            .collect::<Vec<_>>();
        rotated.sort_by_key(|event| event["seq"].as_u64());
        assert_eq!(rotated.iter().map(|event| event["seq"].as_u64()).collect::<Vec<_>>(), vec![Some(5), Some(6)]);
        assert_eq!(valid_events(&fixtures.join("partial-tail.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("corrupt-line.jsonl")).len(), 1);
    }

    #[test]
    fn lecteur_incremental_garde_la_queue_partielle_et_signale_la_ligne_corrompue() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        let mut partial = IncrementalJournalReader::new(fixtures.join("partial-tail.jsonl"));
        let mut partial_items = Vec::new();
        for _ in 0..32 { partial_items.extend(partial.read_chunk(7).unwrap()); }
        assert_eq!(partial_items.iter().filter(|item| matches!(item, JournalReadItem::Event(_))).count(), 1);
        assert_eq!(partial_items.iter().filter(|item| matches!(item, JournalReadItem::Unreadable(_))).count(), 1);

        let root = root("tail-without-newline");
        create_private_dir(&root).unwrap();
        let path = root.join("journal.jsonl");
        fs::write(&path, b"{\"v\":1,\"seq\":9").unwrap();
        assert!(IncrementalJournalReader::new(&path).read_chunk(64).unwrap().is_empty());
        fs::remove_dir_all(root).unwrap();

        let mut corrupt = IncrementalJournalReader::new(fixtures.join("corrupt-line.jsonl"));
        let items = corrupt.read_chunk(16 * 1024).unwrap();
        assert!(matches!(items[0], JournalReadItem::Event(_)));
        assert!(matches!(items[1], JournalReadItem::Unreadable(JournalUnreadableLine { line: 2, offset }) if offset > 0));
    }

    #[test]
    fn lecteur_borne_une_ligne_surdimensionnee_et_reprend_apres_newline() {
        let root = root("oversized"); create_private_dir(&root).unwrap(); let path = root.join("x.jsonl");
        fs::write(&path, [b"{\"v\":1,\"seq\":42,\"x\":\"".as_slice(), &vec![b'x'; MAX_INCREMENTAL_LINE_BYTES + 32], b"\"}\n{\"v\":1,\"seq\":43}\n"].concat()).unwrap();
        let mut reader = IncrementalJournalReader::new(&path); let mut seen = Vec::new();
        for _ in 0..80 { seen.extend(reader.read_chunk(65_536).unwrap()); assert!(reader.buffered_len() <= MAX_INCREMENTAL_LINE_BYTES); }
        assert!(matches!(seen.iter().find(|item| matches!(item, JournalReadItem::Oversized { .. })), Some(JournalReadItem::Oversized { seq: Some(42), .. })));
        assert!(matches!(seen.last(), Some(JournalReadItem::Event(JournalReadEvent { seq: 43, .. }))));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fenetres_resolvent_rotation_seq_inclusif_et_vide() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        let rotation = resolve_window(&fixtures.join("rotation"), &AttachWindow::Seq(5), "2026-08-24").unwrap();
        assert_eq!(rotation.files.len(), 2);
        assert_eq!(rotation.from_seq, Some(5));
        let events = rotation.files.iter().flat_map(|path| valid_events(path)).collect::<Vec<_>>();
        assert_eq!(events.iter().filter_map(|event| event["seq"].as_u64()).collect::<Vec<_>>(), vec![5, 6]);

        let root = root("empty-window");
        create_private_dir(&root).unwrap();
        assert!(resolve_window(&root, &AttachWindow::Today, "2026-08-22").unwrap().files.is_empty());
        assert_eq!(resolve_window(&root, &AttachWindow::Date("2026-08-23".to_string()), "2026-08-22"), Err(JournalWindowError::FutureDate));
        assert_eq!(resolve_window(&root, &AttachWindow::Date("bad".to_string()), "2026-08-22"), Err(JournalWindowError::InvalidDate));
        assert_eq!(resolve_window(&root, &AttachWindow::Date("2026-08-21".to_string()), "2026-08-22"), Err(JournalWindowError::DateOutsideRetention));
        fs::remove_dir_all(root).unwrap();
    }
}
