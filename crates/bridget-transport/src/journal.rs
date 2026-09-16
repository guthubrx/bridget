//! Journal JSONL versionné des sessions ACP.

use crate::acp::{AcpEvent, AcpEventQueue};
use crate::fsutil::create_private_dir;
use crate::protocol::AttachWindow;
use serde::Serialize;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
#[cfg(any(test, feature = "test-support"))]
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "test-support")]
use std::collections::HashMap;
#[cfg(feature = "test-support")]
use std::sync::{OnceLock, Weak};

#[cfg(feature = "test-support")]
type AppendSamples = Arc<Mutex<Vec<AppendLatencySample>>>;
#[cfg(feature = "test-support")]
type AppendProbeRegistry = Mutex<HashMap<PathBuf, AppendProbeRegistration>>;
#[cfg(feature = "test-support")]
struct AppendProbeRegistration {
    samples: Weak<Mutex<Vec<AppendLatencySample>>>,
    all_events: bool,
}

/// Signal minimal vers un adaptateur lorsque le journal append-only devient
/// non fiable. Le journal reste indépendant du protocole fournisseur.
pub type JournalFailureSink = Arc<dyn Fn(String) + Send + Sync>;

/// Observation post-flush limitée aux bancs d'intégration. `completed_at` est
/// pris immédiatement après l'écriture complète de la ligne JSONL.
#[cfg(feature = "test-support")]
#[derive(Debug, Clone, Copy)]
pub struct AppendLatencySample {
    pub seq: u64,
    pub completed_at: Instant,
    pub elapsed: Duration,
}

const WRITER_QUEUE_CAPACITY: usize = 256;
const MAX_INCREMENTAL_LINE_BYTES: usize = 4 * 1024 * 1024;
const LIVE_FEED_CAPACITY_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const TURN_FAILED_KIND: &str = "turn_failed";

/// Marque une erreur comme borne de fin sans déduire sa nature de `reason`.
///
/// Le détail reste libre ; seul ce code fermé est consommable par les outils
/// de pilotage pour décider qu'un tour s'est terminé en échec. Un payload
/// contraire au schéma objet reste volontairement non marqué : le lecteur le
/// classera indéterminé au lieu de faire confiance à une borne mal formée.
pub(crate) fn with_turn_failed_kind(mut payload: Value) -> Value {
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "terminal_kind".to_string(),
            Value::String(TURN_FAILED_KIND.to_string()),
        );
    }
    payload
}

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
    fn new(
        sequence: u64,
        timestamp: &str,
        session_id: &str,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalLiveEvent {
    pub seq: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalLiveBatch {
    pub gap: Option<(u64, u64)>,
    pub events: Vec<JournalLiveEvent>,
}

/// Tampon borné des entrées confirmées après leur flush. Le journal reste la
/// source du rejeu ; ce flux évite seulement de relire son fichier en régime
/// live. Une perte par saturation reste observable comme une plage `gap`.
#[derive(Clone)]
pub struct JournalLiveFeed {
    observation_sender: mpsc::SyncSender<crate::protocol::WrapperToDaemon>,
    observation_receiver: Arc<Mutex<mpsc::Receiver<crate::protocol::WrapperToDaemon>>>,
    observation_dropped: Arc<AtomicU64>,
    sender: mpsc::SyncSender<JournalLiveEvent>,
    receiver: Arc<Mutex<mpsc::Receiver<JournalLiveEvent>>>,
    queued_bytes: Arc<AtomicUsize>,
    dropped_through: Arc<AtomicU64>,
    latest_seq: Arc<AtomicU64>,
    capacity_bytes: usize,
}

impl Default for JournalLiveFeed {
    fn default() -> Self {
        Self::new(LIVE_FEED_CAPACITY_BYTES)
    }
}

impl JournalLiveFeed {
    pub fn new(capacity_bytes: usize) -> Self {
        assert!(
            capacity_bytes > 0,
            "le relais live doit avoir une capacité non nulle"
        );
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let (observation_sender, observation_receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        Self {
            observation_sender,
            observation_receiver: Arc::new(Mutex::new(observation_receiver)),
            observation_dropped: Arc::new(AtomicU64::new(0)),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            queued_bytes: Arc::new(AtomicUsize::new(0)),
            dropped_through: Arc::new(AtomicU64::new(0)),
            latest_seq: Arc::new(AtomicU64::new(0)),
            capacity_bytes,
        }
    }

    fn publish(&self, event: JournalLiveEvent) {
        self.latest_seq.fetch_max(event.seq, Ordering::SeqCst);
        let length = event.bytes.len();
        let reserved = self
            .queued_bytes
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |queued| {
                queued
                    .checked_add(length)
                    .filter(|total| *total <= self.capacity_bytes)
            })
            .is_ok();
        if !reserved {
            self.dropped_through.fetch_max(event.seq, Ordering::SeqCst);
            return;
        }
        if self.sender.try_send(event).is_err() {
            self.queued_bytes.fetch_sub(length, Ordering::SeqCst);
            self.dropped_through
                .fetch_max(self.latest_seq.load(Ordering::SeqCst), Ordering::SeqCst);
        }
    }

    pub fn latest_seq(&self) -> Option<u64> {
        let latest = self.latest_seq.load(Ordering::SeqCst);
        (latest > 0).then_some(latest)
    }

    /// Flux indépendant d'attach : seuls les petits faits structurés y passent.
    /// O(n), n<=256 ; la source n'attend jamais un consommateur.
    pub fn take_observations(&self) -> (Vec<crate::protocol::WrapperToDaemon>, u64) {
        let receiver = self
            .observation_receiver
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        (
            receiver.try_iter().take(WRITER_QUEUE_CAPACITY).collect(),
            self.observation_dropped.swap(0, Ordering::Relaxed),
        )
    }

    pub fn after(&self, cursor: Option<u64>) -> JournalLiveBatch {
        let receiver = self
            .receiver
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let expected = cursor.map_or(1, |seq| seq.saturating_add(1));
        let dropped = self.dropped_through.load(Ordering::SeqCst);
        let gap = (dropped >= expected).then_some((expected, dropped));
        let mut events = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            self.queued_bytes
                .fetch_sub(event.bytes.len(), Ordering::SeqCst);
            if cursor.is_none_or(|seq| event.seq > seq) {
                events.push(event);
            }
        }
        JournalLiveBatch { gap, events }
    }
}

/// Propriétaire unique des E/S du journal. Les threads ACP n'y déposent que
/// des événements, afin qu'un disque lent ne bloque jamais stdout ou un tour.
#[derive(Clone)]
pub struct JournalWriter {
    sender: mpsc::SyncSender<WriterCommand>,
    failure: Arc<Mutex<Option<String>>>,
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    #[cfg(test)]
    append_latencies: Arc<Mutex<Vec<Duration>>>,
}

#[cfg(feature = "test-support")]
fn append_probes() -> &'static AppendProbeRegistry {
    static PROBES: OnceLock<AppendProbeRegistry> = OnceLock::new();
    PROBES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Sonde de test ciblée par racine de journal. Elle n'existe pas dans les
/// builds de production et n'intercepte donc aucun autre journal concurrent.
#[cfg(feature = "test-support")]
pub struct AppendLatencyProbe {
    root: PathBuf,
    samples: AppendSamples,
}

#[cfg(feature = "test-support")]
impl AppendLatencyProbe {
    pub fn install(root: impl AsRef<Path>) -> Self {
        Self::install_with_scope(root, false)
    }

    /// Recette 089 : chaque fragment complet a sa borne post-append, sans
    /// modifier l'échantillonnage historique turn_start/turn_end de SC-005.
    pub fn install_all_events(root: impl AsRef<Path>) -> Self {
        Self::install_with_scope(root, true)
    }

    fn install_with_scope(root: impl AsRef<Path>, all_events: bool) -> Self {
        let root = root.as_ref().to_path_buf();
        let samples = Arc::new(Mutex::new(Vec::new()));
        append_probes()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(
                root.clone(),
                AppendProbeRegistration {
                    samples: Arc::downgrade(&samples),
                    all_events,
                },
            );
        Self { root, samples }
    }

    pub fn sample_count(&self) -> usize {
        self.samples
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .len()
    }

    pub fn take(&self) -> Vec<Duration> {
        std::mem::take(
            &mut *self
                .samples
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
        .into_iter()
        .map(|sample| sample.elapsed)
        .collect()
    }

    /// Prélève les observations complètes, dont la borne temporelle après le
    /// flush, pour corréler append et rendu dans les bancs attach.
    pub fn take_samples(&self) -> Vec<AppendLatencySample> {
        std::mem::take(
            &mut *self
                .samples
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
    }
}

#[cfg(feature = "test-support")]
impl Drop for AppendLatencyProbe {
    fn drop(&mut self) {
        let mut probes = append_probes()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if probes
            .get(&self.root)
            .and_then(|registration| registration.samples.upgrade())
            .is_some_and(|samples| Arc::ptr_eq(&samples, &self.samples))
        {
            probes.remove(&self.root);
        }
    }
}

impl JournalWriter {
    pub fn start(
        root: impl AsRef<Path>,
        agent: &str,
        session_id: &str,
        events: Arc<Mutex<AcpEventQueue>>,
    ) -> std::io::Result<Self> {
        Self::start_with_live_feed(root, agent, session_id, events, None)
    }

    pub fn start_with_live_feed(
        root: impl AsRef<Path>,
        agent: &str,
        session_id: &str,
        events: Arc<Mutex<AcpEventQueue>>,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<Self> {
        let failure_events = events.clone();
        Self::start_with_live_feed_and_failure(
            root,
            agent,
            session_id,
            Arc::new(move |detail| {
                failure_events
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .push_back(AcpEvent::JournalFailed { detail });
            }),
            live_feed,
        )
    }

    /// Variante neutre pour les pilotes gérés qui ne parlent pas ACP.
    pub fn start_with_live_feed_and_failure(
        root: impl AsRef<Path>,
        agent: &str,
        session_id: &str,
        on_failure: JournalFailureSink,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<Self> {
        let root = root.as_ref();
        let mut journal = SessionJournal::new(root, agent, session_id)?;
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let failure = Arc::new(Mutex::new(None));
        let thread_failure = failure.clone();
        #[cfg(test)]
        let append_latencies = Arc::new(Mutex::new(Vec::new()));
        #[cfg(test)]
        let thread_append_latencies = append_latencies.clone();
        #[cfg(feature = "test-support")]
        let thread_probe = append_probes()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(root)
            .and_then(|registration| {
                registration
                    .samples
                    .upgrade()
                    .map(|samples| (samples, registration.all_events))
            });
        let handle = thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                match command {
                    WriterCommand::Entry(entry) => {
                        let observation = observation_metadata(&entry);
                        #[cfg(any(test, feature = "test-support"))]
                        let started = Instant::now();
                        #[cfg(feature = "test-support")]
                        let is_turn_boundary =
                            matches!(entry.event.as_str(), "turn_start" | "turn_end");
                        let live_event = match journal.append_entry(entry) {
                            Ok(event) => event,
                            Err(error) => {
                                let detail = format!("écriture du journal ACP impossible: {error}");
                                *thread_failure
                                    .lock()
                                    .unwrap_or_else(|poison| poison.into_inner()) =
                                    Some(detail.clone());
                                on_failure(detail);
                                break;
                            }
                        };
                        #[cfg(any(test, feature = "test-support"))]
                        let append_elapsed = started.elapsed();
                        #[cfg(feature = "test-support")]
                        let completed_at = Instant::now();
                        #[cfg(feature = "test-support")]
                        let sequence = live_event.seq;
                        if let Some(feed) = &live_feed {
                            if let Some((event, file)) = observation
                                && feed
                                    .observation_sender
                                    .try_send(crate::protocol::WrapperToDaemon::ObservedActivity {
                                        seq: live_event.seq,
                                        event,
                                        file,
                                    })
                                    .is_err()
                            {
                                feed.observation_dropped.fetch_add(1, Ordering::Relaxed);
                            }
                            feed.publish(live_event);
                        }
                        #[cfg(test)]
                        thread_append_latencies
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner())
                            .push(append_elapsed);
                        #[cfg(feature = "test-support")]
                        if let Some((samples, all_events)) = &thread_probe
                            && (is_turn_boundary || *all_events)
                        {
                            samples
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner())
                                .push(AppendLatencySample {
                                    seq: sequence,
                                    completed_at,
                                    elapsed: append_elapsed,
                                });
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
            #[cfg(test)]
            append_latencies,
        })
    }

    pub fn enqueue(
        &self,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) -> Result<(), String> {
        // Contrat d'écriture : un kind d'update hors vocabulaire est refusé ici,
        // point unique traversé par ACP, Claude et Codex — jamais silencieux.
        crate::act_kind::validate_journal_write(event, &payload)?;
        if let Some(error) = self
            .failure
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
        {
            return Err(error);
        }
        let (_, timestamp) = now_date_and_timestamp();
        let entry = JournalEntry::new(0, &timestamp, "", event, message_id, payload);
        self.sender
            .try_send(WriterCommand::Entry(entry))
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => "journal ACP saturé".to_string(),
                mpsc::TrySendError::Disconnected(_) => self
                    .failure
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .clone()
                    .unwrap_or_else(|| "journal ACP arrêté".to_string()),
            })
    }

    pub fn stop(&self) {
        let Some(handle) = self
            .handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        else {
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
            append_latencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[cfg(test)]
    pub(crate) fn take_append_latencies(&self) -> Vec<Duration> {
        std::mem::take(
            &mut *self
                .append_latencies
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
    }
}

fn observation_metadata(
    entry: &JournalEntry,
) -> Option<(crate::protocol::ObservationKind, Option<String>)> {
    use crate::protocol::ObservationKind;
    // Les tours provoqués par nos notifications ne créent pas d'autres
    // observations. La corrélation réutilise l'id du message, pas son texte.
    if entry
        .message_id
        .as_deref()
        .is_some_and(|id| id.starts_with("bridget-observation:"))
    {
        return None;
    }
    // Les adaptateurs natifs/T3 sans corrélation avec le message déclencheur
    // n'exposent pas cette observation : leur simple état idle est ambigu.
    if entry.event == "turn_end" && entry.payload.get("stop_reason").is_some() {
        return Some((ObservationKind::TurnEnded, None));
    }
    if entry.event == "permission"
        || (entry.event == "update"
            && entry.payload.get("kind").and_then(Value::as_str) == Some("approval"))
    {
        return Some((ObservationKind::PermissionRequired, None));
    }
    if entry.event == "update"
        && entry
            .payload
            .get("write_confirmed")
            .and_then(Value::as_bool)
            == Some(true)
        && entry.payload.get("kind").and_then(Value::as_str) == Some("file")
    {
        let path = entry.payload.get("text")?.as_str()?;
        if Path::new(path).is_absolute()
            && path.len() <= 4096
            && !path.chars().any(char::is_control)
        {
            return Some((ObservationKind::FileWritten, Some(path.into())));
        }
    }
    None
}

/// Métadonnée commune à Claude, Codex et ACP, uniquement après succès outil.
/// O(longueur chemin), sans lecture du fichier ni exposition de son contenu.
pub fn confirmed_write_payload(path: &str, cwd: &Path) -> Option<Value> {
    if path.is_empty() || path.len() > 4096 || path.chars().any(char::is_control) {
        return None;
    }
    let absolute = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };
    let absolute = absolute.to_str()?;
    if !Path::new(absolute).is_absolute() || absolute.len() > 4096 {
        return None;
    }
    Some(serde_json::json!({"kind":"file","text":absolute,"write_confirmed":true}))
}

/// Ne reconnaît que des outils écrivains connus ; jamais de parsing shell.
pub fn tool_write_path<'a>(name: &str, input: &'a Value) -> Option<&'a str> {
    if !matches!(
        name,
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" | "write_file" | "replace"
    ) {
        return None;
    }
    input
        .get("file_path")
        .or_else(|| input.get("path"))
        .or_else(|| input.get("notebook_path"))?
        .as_str()
        .filter(|p| !p.is_empty() && p.len() <= 4096 && !p.chars().any(char::is_control))
}

pub struct SessionJournal {
    directory: PathBuf,
    session_id: String,
    next_seq: u64,
    open_file: Option<(String, File)>,
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
            open_file: None,
        })
    }

    pub fn append(
        &mut self,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) -> std::io::Result<u64> {
        let (date, timestamp) = now_date_and_timestamp();
        self.append_at(&date, &timestamp, event, message_id, payload)
    }

    pub fn append_at(
        &mut self,
        date: &str,
        timestamp: &str,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) -> std::io::Result<u64> {
        crate::act_kind::validate_journal_write(event, &payload)
            .map_err(|detail| std::io::Error::new(std::io::ErrorKind::InvalidInput, detail))?;
        let sequence = self.next_seq;
        let entry = JournalEntry::new(
            sequence,
            timestamp,
            &self.session_id,
            event,
            message_id,
            payload,
        );
        self.append_entry_at(date, entry)?;
        self.next_seq = sequence.saturating_add(1);
        Ok(sequence)
    }

    /// Chemin test-only qui pose une entrée **sans** passer par `enqueue` /
    /// `append_at` — simule un futur `WriterCommand::Entry` rendu accessible.
    #[cfg(test)]
    pub(crate) fn append_entry_bypassing_public_guards_for_test(
        &mut self,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) -> std::io::Result<JournalLiveEvent> {
        let (_, timestamp) = now_date_and_timestamp();
        let entry = JournalEntry::new(0, &timestamp, "", event, message_id, payload);
        self.append_entry(entry)
    }

    fn append_entry(&mut self, mut entry: JournalEntry) -> std::io::Result<JournalLiveEvent> {
        // Dette 1 — défense en profondeur : le thread writer n'a que ce point
        // avant le disque. enqueue filtre déjà, mais WriterCommand::Entry est
        // aujourd'hui privé ; un lot qui l'exposerait ouvrirait le trou sans
        // qu'aucun test enqueue ne bronche (motif
        // validation-sur-le-chemin-officiel-ne-protege-rien).
        crate::act_kind::validate_journal_write(&entry.event, &entry.payload)
            .map_err(|detail| std::io::Error::new(std::io::ErrorKind::InvalidInput, detail))?;
        let (date, timestamp) = now_date_and_timestamp();
        entry.seq = self.next_seq;
        entry.ts = timestamp;
        entry.session_id.clone_from(&self.session_id);
        let seq = entry.seq;
        let bytes = self.append_entry_at(&date, entry)?;
        self.next_seq = self.next_seq.saturating_add(1);
        Ok(JournalLiveEvent { seq, bytes })
    }

    fn append_entry_at(&mut self, date: &str, entry: JournalEntry) -> std::io::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&entry)?;
        if self
            .open_file
            .as_ref()
            .is_none_or(|(open_date, _)| open_date != date)
        {
            let path = self.directory.join(format!("{date}.jsonl"));
            isolate_partial_tail(&path)?;
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o600)
                .open(&path)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            self.open_file = Some((date.to_string(), file));
        }
        let file = &mut self.open_file.as_mut().expect("fichier journal ouvert").1;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.flush()?;
        Ok(bytes)
    }
}

pub fn valid_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .ok()
        .into_iter()
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
    Oversized {
        seq: Option<u64>,
        offset: u64,
        line: u64,
    },
}

/// Identité du fichier réellement ouvert pour une tranche de lecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JournalSourceIdentity {
    pub dev: u64,
    pub ino: u64,
    pub len: u64,
}

impl JournalSourceIdentity {
    pub fn same_file(self, other: Self) -> bool {
        self.dev == other.dev && self.ino == other.ino
    }
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

    pub fn next_offset(&self) -> u64 {
        self.next_offset
    }
    #[cfg(test)]
    fn buffered_len(&self) -> usize {
        self.pending.len()
    }

    /// Lit au plus `max_bytes` octets nouveaux. Une queue partielle est gardée
    /// pour l'appel suivant et ne produit donc jamais un faux événement.
    pub fn read_chunk(&mut self, max_bytes: usize) -> std::io::Result<Vec<JournalReadItem>> {
        self.read_chunk_with_source(max_bytes)
            .map(|(items, _)| items)
    }

    /// Lit une tranche et expose l'identité du descripteur effectivement lu.
    /// L'appelant peut ainsi rejeter une tranche issue d'un inode remplacé sans
    /// jamais la mettre en file.
    pub fn read_chunk_with_source(
        &mut self,
        max_bytes: usize,
    ) -> std::io::Result<(Vec<JournalReadItem>, Option<JournalSourceIdentity>)> {
        assert!(max_bytes > 0, "une tranche de journal doit être non nulle");
        let mut file = match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok((Vec::new(), None));
            }
            Err(error) => return Err(error),
        };
        let metadata = file.metadata()?;
        let source = JournalSourceIdentity {
            dev: metadata.dev(),
            ino: metadata.ino(),
            len: metadata.len(),
        };
        file.seek(SeekFrom::Start(self.next_offset))?;
        let mut chunk = vec![0; max_bytes];
        let count = file.read(&mut chunk)?;
        chunk.truncate(count);
        self.next_offset = self.next_offset.saturating_add(count as u64);
        if let Some((_offset, _line, _seq)) = self.discarding {
            if let Some(end) = chunk.iter().position(|byte| *byte == b'\n') {
                self.pending_offset = self
                    .next_offset
                    .saturating_sub(count as u64)
                    .saturating_add((end + 1) as u64);
                self.next_line = self.next_line.saturating_add(1);
                self.discarding = None;
                self.pending.extend_from_slice(&chunk[end + 1..]);
            } else {
                return Ok((Vec::new(), Some(source)));
            }
        } else {
            self.pending.extend_from_slice(&chunk);
        }

        let mut items = Vec::new();
        loop {
            let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') else {
                if self.pending.len() > MAX_INCREMENTAL_LINE_BYTES {
                    let offset = self.pending_offset;
                    let line = self.next_line;
                    let seq = sequence_prefix(&self.pending);
                    items.push(JournalReadItem::Oversized { seq, offset, line });
                    self.pending.clear();
                    self.discarding = Some((offset, line, seq));
                }
                break;
            };
            if end > MAX_INCREMENTAL_LINE_BYTES {
                let offset = self.pending_offset;
                let line = self.next_line;
                let seq = sequence_prefix(&self.pending);
                items.push(JournalReadItem::Oversized { seq, offset, line });
                self.pending.drain(..=end);
                self.pending_offset = self.pending_offset.saturating_add((end + 1) as u64);
                self.next_line = self.next_line.saturating_add(1);
                continue;
            }
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
                            seq,
                            offset,
                            line: line_number,
                            bytes: line,
                        })),
                        None => items.push(JournalReadItem::Unreadable(JournalUnreadableLine {
                            line: line_number,
                            offset,
                        })),
                    }
                }
                _ => items.push(JournalReadItem::Unreadable(JournalUnreadableLine {
                    line: line_number,
                    offset,
                })),
            }
        }
        Ok((items, Some(source)))
    }
}

fn sequence_prefix(bytes: &[u8]) -> Option<u64> {
    let prefix = &bytes[..bytes.len().min(1024)];
    let marker = b"\"seq\":";
    let start = prefix
        .windows(marker.len())
        .position(|window| window == marker)?
        + marker.len();
    let end = prefix[start..]
        .iter()
        .position(|byte| !byte.is_ascii_digit())
        .unwrap_or(prefix.len() - start);
    std::str::from_utf8(&prefix[start..start + end])
        .ok()?
        .parse()
        .ok()
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
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "jsonl")
        })
        .collect::<Vec<_>>();
    files.sort();
    match window {
        AttachWindow::Seq(seq) => Ok(ResolvedJournalWindow {
            files,
            from_seq: Some(*seq),
        }),
        AttachWindow::Tail(count) => {
            let count = (*count).max(1);
            let last = last_sequence_in_files(&files);
            let from_seq = if last == 0 {
                0
            } else {
                last.saturating_sub(count.saturating_sub(1))
            };
            Ok(ResolvedJournalWindow {
                files,
                from_seq: Some(from_seq),
            })
        }
        AttachWindow::Today => Ok(ResolvedJournalWindow {
            files: files
                .into_iter()
                .filter(|path| file_date(path) == Some(host_today))
                .collect(),
            from_seq: None,
        }),
        AttachWindow::Date(date) => {
            if !is_date(date) {
                return Err(JournalWindowError::InvalidDate);
            }
            if date.as_str() > host_today {
                return Err(JournalWindowError::FutureDate);
            }
            let selected = files
                .into_iter()
                .filter(|path| file_date(path) == Some(date))
                .collect::<Vec<_>>();
            if selected.is_empty() {
                return Err(JournalWindowError::DateOutsideRetention);
            }
            Ok(ResolvedJournalWindow {
                files: selected,
                from_seq: None,
            })
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
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
        && matches!(
            (
                value[0..4].parse::<i32>(),
                value[5..7].parse::<u8>(),
                value[8..10].parse::<u8>(),
            ),
            (Ok(year), Ok(month), Ok(day)) if day >= 1 && day <= days_in_month(year, month)
        )
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn last_sequence(directory: &Path) -> u64 {
    let files = fs::read_dir(directory)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    last_sequence_in_files(&files)
}

fn last_sequence_in_files(files: &[PathBuf]) -> u64 {
    files
        .iter()
        .flat_map(|path| valid_events(path))
        .filter_map(|value| value.get("seq").and_then(Value::as_u64))
        .max()
        .unwrap_or(0)
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

/// Date courante du processus qui possède le journal.
pub fn current_host_date() -> String {
    now_date_and_timestamp().0
}

fn now_date_and_timestamp() -> (String, String) {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let (year, month, day, hour, minute, second) = civil_time(seconds);
    (
        format!("{year:04}-{month:02}-{day:02}"),
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"),
    )
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
    (
        year,
        month as u32,
        day as u32,
        day_seconds / 3600,
        day_seconds / 60 % 60,
        day_seconds % 60,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn spec100_observation_requires_confirmed_structured_write() {
        use super::*;
        let payload = confirmed_write_payload("src/lib.rs", Path::new("/project")).unwrap();
        let entry = JournalEntry::new(1, "now", "s", "update", None, payload);
        assert_eq!(
            observation_metadata(&entry),
            Some((
                crate::protocol::ObservationKind::FileWritten,
                Some("/project/src/lib.rs".into())
            ))
        );
        let attempt = JournalEntry::new(
            2,
            "now",
            "s",
            "update",
            None,
            serde_json::json!({"kind":"file","text":"/project/src/lib.rs"}),
        );
        assert!(observation_metadata(&attempt).is_none());
        assert!(tool_write_path("Read", &serde_json::json!({"file_path":"/p/x"})).is_none());
        assert!(tool_write_path("Bash", &serde_json::json!({"command":"echo x > /p/x"})).is_none());
        assert!(confirmed_write_payload("x", Path::new("relative")).is_none());
    }

    #[test]
    fn spec100_observation_feed_is_independent_and_bounded() {
        use super::*;
        let feed = JournalLiveFeed::default();
        for seq in 1..=WRITER_QUEUE_CAPACITY {
            feed.observation_sender
                .try_send(crate::protocol::WrapperToDaemon::ObservedActivity {
                    seq: seq as u64,
                    event: crate::protocol::ObservationKind::PermissionRequired,
                    file: None,
                })
                .unwrap();
        }
        assert!(
            feed.observation_sender
                .try_send(crate::protocol::WrapperToDaemon::ObservedActivity {
                    seq: 999,
                    event: crate::protocol::ObservationKind::PermissionRequired,
                    file: None,
                })
                .is_err()
        );
        let (events, _) = feed.take_observations();
        assert_eq!(events.len(), WRITER_QUEUE_CAPACITY);
        assert!(feed.after(None).events.is_empty());
    }

    #[test]
    fn spec100_post_flush_turns_do_not_reobserve_notifications() {
        let root = root("spec100-post-flush");
        let feed = JournalLiveFeed::default();
        let writer = JournalWriter::start_with_live_feed(
            &root,
            "agent",
            "session",
            Arc::new(Mutex::new(AcpEventQueue::default())),
            Some(feed.clone()),
        )
        .unwrap();
        writer
            .enqueue(
                "turn_end",
                Some("human"),
                json!({"stop_reason":"cancelled"}),
            )
            .unwrap();
        writer
            .enqueue(
                "turn_end",
                Some("bridget-observation:123"),
                json!({"stop_reason":"completed"}),
            )
            .unwrap();
        writer
            .enqueue("turn_end", Some("unattributed-native"), json!({}))
            .unwrap();
        writer
            .enqueue(
                "update",
                Some("bridget-observation:123"),
                confirmed_write_payload("/p/x", Path::new("/p")).unwrap(),
            )
            .unwrap();
        writer
            .enqueue(
                "update",
                Some("human"),
                confirmed_write_payload("/p/x", Path::new("/p")).unwrap(),
            )
            .unwrap();
        writer.stop();
        let (facts, lost) = feed.take_observations();
        assert_eq!(lost, 0);
        assert_eq!(facts.len(), 2);
        assert!(matches!(
            facts[0],
            crate::protocol::WrapperToDaemon::ObservedActivity {
                seq: 1,
                event: crate::protocol::ObservationKind::TurnEnded,
                ..
            }
        ));
        assert!(matches!(
            facts[1],
            crate::protocol::WrapperToDaemon::ObservedActivity {
                seq: 5,
                event: crate::protocol::ObservationKind::FileWritten,
                ..
            }
        ));
        assert_eq!(
            feed.after(None).events.len(),
            5,
            "le journal attach n'est pas consommé par les observations"
        );
    }
    use super::*;
    use serde_json::json;

    fn root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bridget-journal-{name}-{}", std::process::id()))
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn sonde_complete_ne_modifie_pas_les_deux_bornes_historiques() {
        for (all, suffix, expected) in [
            (false, "probe-boundaries", vec![1, 3]),
            (true, "probe-all", vec![1, 2, 3]),
        ] {
            let root = root(suffix);
            let probe = if all {
                AppendLatencyProbe::install_all_events(&root)
            } else {
                AppendLatencyProbe::install(&root)
            };
            let writer = JournalWriter::start(
                &root,
                "probe",
                "session",
                Arc::new(Mutex::new(AcpEventQueue::default())),
            )
            .unwrap();
            for event in ["turn_start", "reasoning", "turn_end"] {
                writer.enqueue(event, Some("message"), json!({})).unwrap();
            }
            writer.stop();
            // Mutation all_events toujours vrai : le premier oracle reçoit
            // [1,2,3] ; toujours faux : le second perd la séquence 2.
            assert_eq!(
                probe
                    .take_samples()
                    .iter()
                    .map(|s| s.seq)
                    .collect::<Vec<_>>(),
                expected
            );
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn sequence_survives_rotation_and_payload_is_versioned() {
        let root = root("rotation");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        assert_eq!(
            journal
                .append_at(
                    "2026-08-22",
                    "2026-08-22T00:00:00Z",
                    "turn_start",
                    Some("m1"),
                    json!({"from":"alice","reply":true,"body":"bonjour"})
                )
                .unwrap(),
            1
        );
        assert_eq!(
            journal
                .append_at(
                    "2026-08-23",
                    "2026-08-23T00:00:00Z",
                    "turn_end",
                    Some("m1"),
                    json!({"stop_reason":"end_turn","routed_to":"alice"})
                )
                .unwrap(),
            2
        );
        let event = valid_events(&root.join("codex-1/2026-08-23.jsonl"))
            .pop()
            .unwrap();
        assert_eq!(event["v"], 1);
        assert_eq!(event["seq"], 2);
        assert_eq!(event["payload"]["routed_to"], "alice");
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(root.join("codex-1/2026-08-23.jsonl"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn next_sequence_recovers_from_normal_empty_partial_and_corrupt_tails() {
        for (name, contents, expected) in [
            ("normal", "{\"v\":1,\"seq\":4}\n", 5),
            ("empty", "", 1),
            ("partial", "{\"v\":1,\"seq\":4}\n{\"v\":1,\"seq\":", 5),
            ("corrupt", "{\"v\":1,\"seq\":4}\nnot-json\n", 5),
        ] {
            let root = root(name);
            let directory = root.join("codex-1");
            create_private_dir(&directory).unwrap();
            fs::write(directory.join("2026-08-22.jsonl"), contents).unwrap();
            let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
            assert_eq!(
                journal
                    .append_at(
                        "2026-08-23",
                        "2026-08-23T00:00:00Z",
                        "error",
                        None,
                        json!({"reason":"test"})
                    )
                    .unwrap(),
                expected
            );
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
        assert_eq!(
            journal
                .append_at(
                    "2026-08-22",
                    "2026-08-22T01:00:00Z",
                    "error",
                    None,
                    json!({"reason":"reprise"})
                )
                .unwrap(),
            5
        );
        let entries = valid_events(&path);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry["seq"].as_u64())
                .collect::<Vec<_>>(),
            vec![Some(4), Some(5)]
        );
        assert_eq!(entries[1]["payload"]["reason"], "reprise");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writer_omits_an_absent_message_id() {
        let root = root("optional-message-id");
        let writer = JournalWriter::start(
            &root,
            "codex-1",
            "session-1",
            Arc::new(Mutex::new(AcpEventQueue::default())),
        )
        .unwrap();
        writer
            .enqueue("error", None, json!({"reason":"global"}))
            .unwrap();
        writer.stop();
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let event = valid_events(&path).pop().unwrap();
        assert!(event.get("message_id").is_none());
        assert_eq!(event["payload"], json!({"reason":"global"}));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_journal_writer_refuse_un_kind_hors_vocabulaire() {
        let root = root("act-kind-contract");
        let writer = JournalWriter::start(
            &root,
            "codex-1",
            "session-1",
            Arc::new(Mutex::new(AcpEventQueue::default())),
        )
        .unwrap();
        writer
            .enqueue("update", Some("m1"), json!({"kind":"tool","text":"Read"}))
            .expect("écriture tool valide doit passer d'abord");
        let rejected = writer.enqueue(
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            rejected
                .as_ref()
                .is_err_and(|detail| detail.contains("hors vocabulaire")),
            "TEMOIN: écriture hors vocabulaire doit être refusée, got {rejected:?}"
        );
        writer.stop();
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = valid_events(&path);
        assert_eq!(
            entries.len(),
            1,
            "le kind refusé ne doit jamais atteindre le JSONL"
        );
        assert_eq!(entries[0]["payload"]["kind"], "tool");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_session_journal_append_at_refuse_un_kind_hors_vocabulaire() {
        // Bouche le trou « mutant append_at-retirée » : enqueue seul ne suffit
        // pas — append_at est aussi un point d'entrée public.
        let root = root("act-kind-append-at");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        journal
            .append("update", Some("m1"), json!({"kind":"tool","text":"Read"}))
            .expect("écriture tool valide doit passer d'abord");
        let rejected = journal.append(
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            rejected.as_ref().is_err_and(|err| {
                err.kind() == std::io::ErrorKind::InvalidInput
                    && err.to_string().contains("hors vocabulaire")
            }),
            "TEMOIN append_at: hors vocabulaire doit être refusé, got {rejected:?}"
        );
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = valid_events(&path);
        assert_eq!(
            entries.len(),
            1,
            "le kind refusé ne doit jamais atteindre le JSONL"
        );
        assert_eq!(entries[0]["payload"]["kind"], "tool");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_append_entry_refuse_meme_si_enqueue_contourne() {
        // Dette 1 : garde sur le chemin writer. Preuve d'abord qu'un cas valide
        // passe — sinon l'oracle d'absence tourne sur une projection vide.
        let root = root("act-kind-append-entry");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        journal
            .append_entry_bypassing_public_guards_for_test(
                "update",
                Some("m1"),
                json!({"kind":"tool","text":"Read"}),
            )
            .expect("append_entry tool valide doit passer d'abord");
        let rejected = journal.append_entry_bypassing_public_guards_for_test(
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            rejected.as_ref().is_err_and(|err| {
                err.kind() == std::io::ErrorKind::InvalidInput
                    && err.to_string().contains("hors vocabulaire")
            }),
            "TEMOIN append_entry: hors vocabulaire doit être refusé même hors enqueue, got {rejected:?}"
        );
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = valid_events(&path);
        assert_eq!(
            entries.len(),
            1,
            "le kind refusé ne doit jamais atteindre le JSONL"
        );
        assert_eq!(entries[0]["payload"]["kind"], "tool");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_retire_garde_append_entry_tue_TEMOIN_append_entry() {
        // Simule le trou d'avant la dette 1 : append_entry écrit sans valider.
        fn broken_append_entry_no_guard(
            journal: &mut SessionJournal,
            event: &str,
            message_id: Option<&str>,
            payload: Value,
        ) -> std::io::Result<JournalLiveEvent> {
            let (_, timestamp) = now_date_and_timestamp();
            let mut entry = JournalEntry::new(0, &timestamp, "", event, message_id, payload);
            let (date, timestamp) = now_date_and_timestamp();
            entry.seq = journal.next_seq;
            entry.ts = timestamp;
            entry.session_id.clone_from(&journal.session_id);
            let seq = entry.seq;
            let bytes = journal.append_entry_at(&date, entry)?;
            journal.next_seq = journal.next_seq.saturating_add(1);
            Ok(JournalLiveEvent { seq, bytes })
        }
        let root = root("act-kind-append-entry-mutant");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        broken_append_entry_no_guard(
            &mut journal,
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        )
        .expect("le mutant laisse passer");
        let healthy = journal.append_entry_bypassing_public_guards_for_test(
            "update",
            Some("m2"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            healthy
                .as_ref()
                .is_err_and(|err| err.to_string().contains("hors vocabulaire")),
            "TEMOIN_append_entry doit mourir si la garde writer est retirée: {healthy:?}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writer_emits_a_terminal_event_when_an_append_fails() {
        let root = root("write-failure");
        let events = Arc::new(Mutex::new(AcpEventQueue::default()));
        let writer = JournalWriter::start(&root, "codex-1", "session-1", events.clone()).unwrap();
        fs::remove_dir_all(root.join("codex-1")).unwrap();
        writer
            .enqueue("error", None, json!({"reason":"test"}))
            .unwrap();
        for _ in 0..20 {
            if events
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(event, AcpEvent::JournalFailed { .. }))
            {
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
        let mut rotated = std::fs::read_dir(rotation)
            .unwrap()
            .filter_map(Result::ok)
            .flat_map(|entry| valid_events(&entry.path()))
            .collect::<Vec<_>>();
        rotated.sort_by_key(|event| event["seq"].as_u64());
        assert_eq!(
            rotated
                .iter()
                .map(|event| event["seq"].as_u64())
                .collect::<Vec<_>>(),
            vec![Some(5), Some(6)]
        );
        assert_eq!(valid_events(&fixtures.join("partial-tail.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("corrupt-line.jsonl")).len(), 1);
    }

    #[test]
    fn lecteur_incremental_garde_la_queue_partielle_et_signale_la_ligne_corrompue() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        let mut partial = IncrementalJournalReader::new(fixtures.join("partial-tail.jsonl"));
        let mut partial_items = Vec::new();
        for _ in 0..32 {
            partial_items.extend(partial.read_chunk(7).unwrap());
        }
        assert_eq!(
            partial_items
                .iter()
                .filter(|item| matches!(item, JournalReadItem::Event(_)))
                .count(),
            1
        );
        assert_eq!(
            partial_items
                .iter()
                .filter(|item| matches!(item, JournalReadItem::Unreadable(_)))
                .count(),
            1
        );

        let root = root("tail-without-newline");
        create_private_dir(&root).unwrap();
        let path = root.join("journal.jsonl");
        fs::write(&path, b"{\"v\":1,\"seq\":9").unwrap();
        assert!(
            IncrementalJournalReader::new(&path)
                .read_chunk(64)
                .unwrap()
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();

        let mut corrupt = IncrementalJournalReader::new(fixtures.join("corrupt-line.jsonl"));
        let items = corrupt.read_chunk(16 * 1024).unwrap();
        assert!(matches!(items[0], JournalReadItem::Event(_)));
        assert!(
            matches!(items[1], JournalReadItem::Unreadable(JournalUnreadableLine { line: 2, offset }) if offset > 0)
        );
    }

    #[test]
    fn lecteur_borne_une_ligne_surdimensionnee_et_reprend_apres_newline() {
        let root = root("oversized");
        create_private_dir(&root).unwrap();
        let path = root.join("x.jsonl");
        fs::write(
            &path,
            [
                b"{\"v\":1,\"seq\":42,\"x\":\"".as_slice(),
                &vec![b'x'; MAX_INCREMENTAL_LINE_BYTES + 32],
                b"\"}\n{\"v\":1,\"seq\":43}\n",
            ]
            .concat(),
        )
        .unwrap();
        let mut reader = IncrementalJournalReader::new(&path);
        let mut seen = Vec::new();
        for _ in 0..80 {
            seen.extend(reader.read_chunk(65_536).unwrap());
            assert!(reader.buffered_len() <= MAX_INCREMENTAL_LINE_BYTES);
        }
        assert!(matches!(
            seen.iter()
                .find(|item| matches!(item, JournalReadItem::Oversized { .. })),
            Some(JournalReadItem::Oversized { seq: Some(42), .. })
        ));
        assert!(matches!(
            seen.last(),
            Some(JournalReadItem::Event(JournalReadEvent { seq: 43, .. }))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lecteur_mesure_la_borne_sur_la_premiere_ligne_et_non_le_tampon_entier() {
        fn valid_line(seq: u64, length: usize) -> Vec<u8> {
            let prefix = format!("{{\"v\":1,\"seq\":{seq},\"body\":\"").into_bytes();
            let suffix = b"\"}";
            assert!(length >= prefix.len() + suffix.len());
            [
                prefix.as_slice(),
                &vec![b'x'; length - prefix.len() - suffix.len()],
                suffix,
            ]
            .concat()
        }

        let root = root("incremental-boundaries");
        create_private_dir(&root).unwrap();

        let near = root.join("near.jsonl");
        let mut near_contents = valid_line(40, MAX_INCREMENTAL_LINE_BYTES - 8);
        near_contents.extend_from_slice(b"\n{\"v\":1,\"seq\":41}\n");
        fs::write(&near, near_contents).unwrap();
        let mut near_reader = IncrementalJournalReader::new(&near);
        let near_items = near_reader
            .read_chunk(MAX_INCREMENTAL_LINE_BYTES + 64)
            .unwrap();
        assert!(
            near_items
                .iter()
                .all(|item| !matches!(item, JournalReadItem::Oversized { .. }))
        );
        assert_eq!(
            near_items
                .iter()
                .filter_map(|item| match item {
                    JournalReadItem::Event(event) => Some(event.seq),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![40, 41]
        );

        let exact = root.join("exact.jsonl");
        let mut exact_contents = valid_line(42, MAX_INCREMENTAL_LINE_BYTES);
        exact_contents.push(b'\n');
        fs::write(&exact, exact_contents).unwrap();
        let exact_items = IncrementalJournalReader::new(&exact)
            .read_chunk(MAX_INCREMENTAL_LINE_BYTES + 1)
            .unwrap();
        assert!(matches!(
            exact_items.as_slice(),
            [JournalReadItem::Event(JournalReadEvent { seq: 42, .. })]
        ));

        let too_large = root.join("too-large.jsonl");
        let mut too_large_contents = valid_line(43, MAX_INCREMENTAL_LINE_BYTES + 1);
        too_large_contents.push(b'\n');
        fs::write(&too_large, too_large_contents).unwrap();
        let too_large_items = IncrementalJournalReader::new(&too_large)
            .read_chunk(MAX_INCREMENTAL_LINE_BYTES + 2)
            .unwrap();
        assert!(matches!(
            too_large_items.as_slice(),
            [JournalReadItem::Oversized { seq: Some(43), .. }]
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fenetres_resolvent_rotation_seq_inclusif_et_vide() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        let rotation = resolve_window(
            &fixtures.join("rotation"),
            &AttachWindow::Seq(5),
            "2026-08-24",
        )
        .unwrap();
        assert_eq!(rotation.files.len(), 2);
        assert_eq!(rotation.from_seq, Some(5));

        let mut first = IncrementalJournalReader::new(&rotation.files[0]);
        let mut first_items = Vec::new();
        while first.next_offset() < fs::metadata(&rotation.files[0]).unwrap().len() {
            first_items.extend(first.read_chunk(9).unwrap());
        }
        assert_eq!(
            first.next_offset(),
            fs::metadata(&rotation.files[0]).unwrap().len()
        );
        assert!(
            matches!(first_items.as_slice(), [JournalReadItem::Event(event)] if event.seq == 5 && event.offset == 0 && event.line == 1)
        );
        assert!(
            first.read_chunk(9).unwrap().is_empty(),
            "le premier fichier est consommé avant la rotation"
        );

        let mut second = IncrementalJournalReader::new(&rotation.files[1]);
        let mut second_items = Vec::new();
        while second.next_offset() < fs::metadata(&rotation.files[1]).unwrap().len() {
            second_items.extend(second.read_chunk(9).unwrap());
        }
        assert_eq!(
            second.next_offset(),
            fs::metadata(&rotation.files[1]).unwrap().len()
        );
        assert!(
            matches!(second_items.as_slice(), [JournalReadItem::Event(event)] if event.seq == 6 && event.offset == 0 && event.line == 1)
        );
        assert_eq!(
            first_items
                .iter()
                .chain(second_items.iter())
                .filter_map(|item| match item {
                    JournalReadItem::Event(event) => Some(event.seq),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![5, 6]
        );

        let partial_tail = fixtures.join("partial-tail.jsonl");
        let mut old = IncrementalJournalReader::new(&partial_tail);
        let mut old_items = Vec::new();
        while old.next_offset() < fs::metadata(&partial_tail).unwrap().len() {
            old_items.extend(old.read_chunk(7).unwrap());
        }
        assert!(
            matches!(old_items.first(), Some(JournalReadItem::Event(event)) if event.seq == 7 && event.offset == 0)
        );
        assert!(matches!(
            old_items.get(1),
            Some(JournalReadItem::Unreadable(_))
        ));
        assert!(
            old.read_chunk(7).unwrap().is_empty(),
            "la queue partielle ancienne ne bloque pas la rotation"
        );
        let partial_root = root("partial-tail-before-next");
        create_private_dir(&partial_root).unwrap();
        let path_after_partial = partial_root.join("2026-08-25.jsonl");
        fs::write(&path_after_partial, b"{\"v\":1,\"seq\":8}\n").unwrap();
        let mut next = IncrementalJournalReader::new(path_after_partial);
        let next_items = next.read_chunk(32).unwrap();
        assert!(
            matches!(next_items.as_slice(), [JournalReadItem::Event(event)] if event.seq == 8 && event.offset == 0)
        );
        fs::remove_dir_all(partial_root).unwrap();

        let empty_root = root("empty-window");
        create_private_dir(&empty_root).unwrap();
        assert!(
            resolve_window(&empty_root, &AttachWindow::Today, "2026-08-22")
                .unwrap()
                .files
                .is_empty()
        );
        assert_eq!(
            resolve_window(
                &empty_root,
                &AttachWindow::Date("2026-08-23".to_string()),
                "2026-08-22"
            ),
            Err(JournalWindowError::FutureDate)
        );
        assert_eq!(
            resolve_window(
                &empty_root,
                &AttachWindow::Date("bad".to_string()),
                "2026-08-22"
            ),
            Err(JournalWindowError::InvalidDate)
        );
        assert_eq!(
            resolve_window(
                &empty_root,
                &AttachWindow::Date("2026-08-21".to_string()),
                "2026-08-22"
            ),
            Err(JournalWindowError::DateOutsideRetention)
        );
        fs::remove_dir_all(&empty_root).unwrap();

        let tail_root = root("tail-window");
        create_private_dir(&tail_root).unwrap();
        let mut lines = String::new();
        for seq in 1..=100 {
            lines.push_str(&format!("{{\"v\":1,\"seq\":{seq}}}\n"));
        }
        fs::write(tail_root.join("2026-08-22.jsonl"), lines).unwrap();
        let tail = resolve_window(&tail_root, &AttachWindow::Tail(30), "2026-08-22").unwrap();
        assert_eq!(tail.from_seq, Some(71));
        let older = resolve_window(&tail_root, &AttachWindow::Seq(21), "2026-08-22").unwrap();
        assert_eq!(older.from_seq, Some(21));
        assert!(
            older.from_seq.unwrap() < tail.from_seq.unwrap(),
            "from_seq plus bas doit donner l'ancien au-delà du Tail"
        );
        fs::remove_dir_all(tail_root).unwrap();

        assert!(!is_date("2026-02-31"));
        assert!(is_date("2024-02-29"));
        assert!(!is_date("2026-02-29"));
        assert!(!is_date("2026-04-31"));
    }
}
