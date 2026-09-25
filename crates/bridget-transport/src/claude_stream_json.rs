//! Pilote natif Claude Code en JSONL `stream-json`.
//!
//! Les lignes du fournisseur restent brutes jusqu'à la frontière commune. Ce
//! module ne projette ni modèle, ni quota, ni consommation : G5 ne couvre que
//! la session gérée, le journal et son cycle de vie.

use crate::claude_provider_session::{
    ProviderSessionStore, classify_resume_failure, prepare_launch_args,
    session_id_from_system_init, strip_resume_arg,
};
use crate::journal::{JournalFailureSink, JournalLiveFeed, JournalWriter, with_turn_failed_kind};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource,
    ManagedProviderIdentity, ManagedSession, ManagedSessionDescriptor, ManagedTerminal,
};
use crate::protocol::{PresenceMode, ProviderObservation};
use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, MessageIntent, MessageOrigin};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Délai absolu daemon avant Connected (`DEFAULT_SPAWN_TIMEOUT_SECS` dans cli).
pub const DAEMON_SPAWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Contexte de validation `--resume` déplacée hors du chemin spawn (reader).
struct ResumeBootstrap {
    attempted_id: String,
    command: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
    inherit_stderr: bool,
    writer: Writer,
    child: Arc<Mutex<Child>>,
}

type Writer = Arc<Mutex<Option<ChildStdin>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;
type SessionStoreHandle = Arc<Mutex<Option<ProviderSessionStore>>>;

#[derive(Debug, Clone)]
pub struct ClaudeStreamJsonOptions {
    pub command: String,
    /// Arguments de définition, dont l'éventuel `--model` demandé.
    pub args: Vec<String>,
    /// Type déclaré par le registre, conservé dans les événements et les contextes.
    pub provider_kind: String,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    /// Racine durable `~/.cache/bridget/sessions` (même arbre que le journal).
    /// Survit à la mort du wrapper et au redémarrage du daemon.
    pub session_store_root: Option<PathBuf>,
    /// Baseline relevée avant lancement, absente pour un registre historique.
    pub provider_observation: Option<ProviderObservation>,
    /// Nom d'agent sous lequel lire/écrire `claude_provider_session`.
    pub agent_name: Option<String>,
}

struct ActiveTurn {
    message_id: String,
    completion: mpsc::Sender<ManagedTerminal>,
    response: String,
    /// Nombre d'événements journal `update` texte déjà écrits pour ce tour.
    text_updates: usize,
    /// Issue fournisseur (`end_turn`, …) — absente ⇒ le fil affiche « inconnu ».
    stop_reason: Option<String>,
}

struct QueueState {
    messages: VecDeque<BridgetMessage>,
    active: Option<ActiveTurn>,
    closed: bool,
    /// `request_id` du `control_request` d'interruption émis et non encore
    /// corrélé par son `control_response`. Porté par l'état de file parce que
    /// l'émetteur (`cancel_delivery`) et le lecteur le partagent déjà.
    pending_interrupt: Option<String>,
}

pub struct ClaudeStreamJsonTransport {
    connection_id: String,
    alive: Arc<AtomicBool>,
    shutdown_started: AtomicBool,
    busy: Arc<AtomicBool>,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    queue_capacity: usize,
    writer: Writer,
    child: Arc<Mutex<Child>>,
    provider_kind: String,
    provider_observation: Option<ProviderObservation>,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    journal: Journal,
    session_store: SessionStoreHandle,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl ClaudeStreamJsonTransport {
    pub fn spawn(options: ClaudeStreamJsonOptions) -> Result<Self, TransportError> {
        Self::spawn_with_environment(options, &[], false)
    }

    pub fn spawn_inheriting_stderr_with_environment(
        options: ClaudeStreamJsonOptions,
        environment: &[(String, String)],
    ) -> Result<Self, TransportError> {
        Self::spawn_with_environment(options, environment, true)
    }

    pub fn spawn_with_environment(
        mut options: ClaudeStreamJsonOptions,
        environment: &[(String, String)],
        inherit_stderr: bool,
    ) -> Result<Self, TransportError> {
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue Claude de capacité nulle".to_string(),
            ));
        }
        let session_store = match (&options.session_store_root, &options.agent_name) {
            (Some(root), Some(agent)) if !agent.trim().is_empty() => {
                Some(ProviderSessionStore::new(root.clone(), agent.clone()))
            }
            _ => None,
        };
        let attempted_resume = prepare_launch_args(&mut options.args, session_store.as_ref());
        ensure_stream_arguments(&mut options.args)?;

        // Ne pas attendre la première trame ici : le daemon accorde 10 s avant
        // Connected et une reprise réelle peut dépasser ce plafond (~10,5 s).
        // La validation `--resume` vit dans le reader, après le spawn.
        let spawned = spawn_claude_child(&options, environment, inherit_stderr)?
            .ok_or_else(|| TransportError::Io("impossible de lancer Claude".to_string()))?;

        let pid = spawned.child.id();
        let writer = Arc::new(Mutex::new(Some(spawned.stdin)));
        let child = Arc::new(Mutex::new(spawned.child));
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
                pending_interrupt: None,
            }),
            Condvar::new(),
        ));
        let alive = Arc::new(AtomicBool::new(true));
        let busy = Arc::new(AtomicBool::new(false));
        let private_profile_instructions = Arc::new(Mutex::new(None));
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let journal = Arc::new(Mutex::new(None));
        let session_store_handle = Arc::new(Mutex::new(session_store));
        let provider_kind = options.provider_kind.clone();
        let pinned_model = pinned_model_from_args(&options.args);
        let resume_bootstrap = attempted_resume.map(|attempted_id| ResumeBootstrap {
            attempted_id,
            command: options.command.clone(),
            args: options.args.clone(),
            environment: environment.to_vec(),
            inherit_stderr,
            writer: writer.clone(),
            child: child.clone(),
        });
        let reader_handle = spawn_reader(
            spawned.stdout,
            Vec::new(),
            queue.clone(),
            events.clone(),
            alive.clone(),
            journal.clone(),
            session_store_handle.clone(),
            provider_kind.clone(),
            pinned_model,
            resume_bootstrap,
        );
        let worker_handle = spawn_worker(
            queue.clone(),
            writer.clone(),
            events.clone(),
            journal.clone(),
            alive.clone(),
            busy.clone(),
            private_profile_instructions.clone(),
            Duration::from_secs(options.notify_timeout_secs),
        );
        push_internal(
            &events,
            ManagedEventKind::ProviderContextObserved {
                identity: ManagedProviderIdentity {
                    provider_kind: provider_kind.clone(),
                    provider_session_id: None,
                    provider_thread_id: None,
                    active_turn_id: None,
                    provider_item_id: None,
                    capabilities_revision: options
                        .provider_observation
                        .as_ref()
                        .map(|observation| observation.contract_version.clone()),
                    provider_observation: options.provider_observation.clone(),
                },
            },
        );
        Ok(Self {
            connection_id: format!("claude-stream-json-{pid}"),
            alive,
            shutdown_started: AtomicBool::new(false),
            busy,
            private_profile_instructions,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            provider_kind,
            provider_observation: options.provider_observation,
            child,
            events,
            journal,
            session_store: session_store_handle,
            reader_handle: Mutex::new(Some(reader_handle)),
            worker_handle: Mutex::new(Some(worker_handle)),
        })
    }

    fn push_internal(&self, kind: ManagedEventKind) {
        push_internal(&self.events, kind);
    }

    fn shutdown(&self) {
        if self.shutdown_started.swap(true, Ordering::SeqCst) {
            return;
        }
        self.alive.store(false, Ordering::SeqCst);
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        queue.closed = true;
        while let Some(message) = queue.messages.pop_front() {
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id,
                reason: "équipier Claude arrêté".to_string(),
            });
        }
        if let Some(active) = queue.active.take() {
            let _ = active.completion.send(ManagedTerminal::Cancelled);
        }
        wake.notify_all();
        drop(queue);
        self.writer
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take();
        let mut child = self
            .child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        terminate_group(&mut child);
        drop(child);
        if let Some(handle) = self
            .reader_handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        {
            let _ = handle.join();
        }
        if let Some(handle) = self
            .worker_handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        {
            let _ = handle.join();
        }
        if let Some(journal) = self
            .journal
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        {
            journal.stop();
        }
    }
}

impl Drop for ClaudeStreamJsonTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Transport for ClaudeStreamJsonTransport {
    fn deliver(&mut self, message: &BridgetMessage) -> Result<(), TransportError> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(TransportError::AgentDead);
        }
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        if message.intent == Some(MessageIntent::SteerCurrent) {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "pilotage Claude indisponible".to_string(),
            });
            return Ok(());
        }
        if queue.closed || queue.messages.len() >= self.queue_capacity {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "file Claude pleine".to_string(),
            });
            return Ok(());
        }
        queue.messages.push_back(message.clone());
        // Le relais UI marque l'origine humaine sans choisir le protocole.
        // Claude conserve la FIFO et ne cible que le tour actif capturé.
        let active_message_id = ((message.intent == Some(MessageIntent::InterruptAndStart)
            || message.origin == Some(MessageOrigin::Human))
            && queue.pending_interrupt.is_none())
        .then(|| {
            queue
                .active
                .as_ref()
                .map(|active| active.message_id.clone())
        })
        .flatten();
        if let Some(active_message_id) = active_message_id {
            drop(queue);
            wake.notify_one();
            let _ = self.cancel_delivery(&active_message_id, "interruption explicite");
        } else {
            wake.notify_one();
        }
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

impl ManagedSession for ClaudeStreamJsonTransport {
    fn set_private_profile_instructions(
        &mut self,
        instructions: &str,
    ) -> Result<(), TransportError> {
        *self
            .private_profile_instructions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(instructions.to_string());
        Ok(())
    }

    fn descriptor(&self) -> ManagedSessionDescriptor {
        ManagedSessionDescriptor {
            transport: "stdio".to_string(),
            mode: PresenceMode::Cli,
            location: None,
        }
    }

    fn provider_identity(&self) -> Option<ManagedProviderIdentity> {
        let session_id = self
            .session_store
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .as_ref()
            .and_then(ProviderSessionStore::load);
        Some(ManagedProviderIdentity {
            provider_kind: self.provider_kind.clone(),
            provider_session_id: session_id,
            provider_thread_id: None,
            active_turn_id: None,
            provider_item_id: None,
            capabilities_revision: self
                .provider_observation
                .as_ref()
                .map(|observation| observation.contract_version.clone()),
            provider_observation: self.provider_observation.clone(),
        })
    }

    fn process_id(&self) -> u32 {
        self.child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .id()
    }

    fn activate_journal(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        let events = self.events.clone();
        let failure: JournalFailureSink = Arc::new(move |detail| {
            push_internal(&events, ManagedEventKind::JournalFailed { detail });
        });
        *self
            .journal
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) =
            Some(JournalWriter::start_with_live_feed_and_failure(
                root,
                agent,
                self.connection_id(),
                failure,
                live_feed,
            )?);
        // Même racine que le journal : le session_id fournisseur survit ainsi
        // à la mort du wrapper et au redémarrage du daemon.
        *self
            .session_store
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) =
            Some(ProviderSessionStore::new(root, agent));
        Ok(())
    }

    fn drain_events(&self) -> Vec<ManagedEvent> {
        self.events
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .drain(..)
            .collect()
    }

    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        if queue
            .active
            .as_ref()
            .is_some_and(|active| active.message_id == message_id)
        {
            // Le contrôle d'interruption Claude est désormais attesté (mesures
            // du 28/08) : on coupe le tour au lieu de tuer l'agent, qui reste
            // vivant et enchaîne. Le terminal `aborted_*` clôt la livraison ;
            // l'échéance du tour (`recv_timeout`) reste le repli si le
            // fournisseur ne rend ni accusé ni terminal.
            let request_id = format!("bridget-interrupt-{message_id}");
            if write_control_interrupt(&self.writer, &request_id).is_ok() {
                queue.pending_interrupt = Some(request_id.clone());
                drop(queue);
                wake.notify_one();
                record_or_terminal(
                    &self.journal,
                    &self.events,
                    "interrupt_requested",
                    Some(message_id),
                    json!({ "request_id": request_id, "reason": reason }),
                );
                return true;
            }
            // Repli historique : sans canal d'écriture, la seule fin honnête
            // reste de terminer la livraison et d'arrêter le groupe.
            let active = queue.active.take().expect("tour Claude actif");
            let _ = active.completion.send(ManagedTerminal::Cancelled);
            wake.notify_one();
            self.alive.store(false, Ordering::SeqCst);
            self.writer
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .take();
            let mut child = self
                .child
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            terminate_group(&mut child);
            return true;
        }
        let Some(position) = queue
            .messages
            .iter()
            .position(|message| message.id == message_id)
        else {
            return false;
        };
        let message = queue
            .messages
            .remove(position)
            .expect("message Claude présent");
        wake.notify_one();
        drop(queue);
        self.push_internal(ManagedEventKind::DeliveryRejected {
            message_id: message.id,
            reason: reason.to_string(),
        });
        true
    }

    fn stop(&self) {
        self.shutdown();
    }

    fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }
}

fn ensure_stream_arguments(args: &mut Vec<String>) -> Result<(), TransportError> {
    if args.iter().any(|argument| {
        matches!(argument.as_str(), "--input-format" | "--output-format")
            || argument.starts_with("--input-format=")
            || argument.starts_with("--output-format=")
    }) {
        return Err(TransportError::DeliveryFailed(
            "le pilote Claude impose ses formats stream-json".to_string(),
        ));
    }
    args.extend(
        [
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--include-partial-messages",
            "--verbose",
        ]
        .into_iter()
        .map(str::to_string),
    );
    Ok(())
}

struct SpawnedClaude {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

/// Lance le binaire Claude. La validation `--resume` n'est pas faite ici :
/// voir `bootstrap_resume_in_reader`.
fn spawn_claude_child(
    options: &ClaudeStreamJsonOptions,
    environment: &[(String, String)],
    inherit_stderr: bool,
) -> Result<Option<SpawnedClaude>, TransportError> {
    let mut command = Command::new(&options.command);
    command
        .args(&options.args)
        .envs(environment.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(if inherit_stderr {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .process_group(0);
    let mut child = command
        .spawn()
        .map_err(|error| TransportError::Io(format!("impossible de lancer Claude: {error}")))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| TransportError::Io("stdin Claude absent".to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| TransportError::Io("stdout Claude absent".to_string()))?;
    Ok(Some(SpawnedClaude {
        child,
        stdin,
        stdout,
    }))
}

/// Repli sans `--resume` après échec nommé.
fn relaunch_after_resume_failure(
    mut bootstrap: ResumeBootstrap,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    session_store: &SessionStoreHandle,
    notice: String,
) -> (ChildStdout, Vec<String>) {
    if let Some(store) = session_store
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .as_ref()
    {
        let _ = store.clear();
    }
    push_internal(events, ManagedEventKind::Error { detail: notice });
    {
        let mut child = bootstrap
            .child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        terminate_group(&mut child);
    }
    strip_resume_arg(&mut bootstrap.args);
    let Ok(Some(spawned)) = spawn_claude_command(
        &bootstrap.command,
        &bootstrap.args,
        &bootstrap.environment,
        bootstrap.inherit_stderr,
    ) else {
        let fallback = Command::new("/bin/true")
            .stdout(Stdio::piped())
            .spawn()
            .ok()
            .and_then(|mut child| child.stdout.take());
        return (fallback.expect("repli Claude indisponible"), Vec::new());
    };
    {
        let mut writer = bootstrap
            .writer
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        *writer = Some(spawned.stdin);
    }
    {
        let mut child = bootstrap
            .child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        *child = spawned.child;
    }
    (spawned.stdout, Vec::new())
}

/// Lit la première trame `--resume` dans le reader (hors chemin spawn). En
/// cas d'échec : message nommé, effacement de l'id, repli sans `--resume`.
fn bootstrap_resume_in_reader(
    stdout: ChildStdout,
    bootstrap: ResumeBootstrap,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    session_store: &SessionStoreHandle,
) -> (ChildStdout, Vec<String>) {
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    let read = reader.read_line(&mut first_line);
    let attempted_id = bootstrap.attempted_id.clone();

    match read {
        Ok(0) => relaunch_after_resume_failure(
            bootstrap,
            events,
            session_store,
            format!(
                "reprise Claude impossible: conversation introuvable pour l'identifiant {attempted_id} — démarrage d'une session neuve (stdout vide après --resume)"
            ),
        ),
        Ok(_) => {
            let trimmed = first_line.trim_end_matches(['\r', '\n']).to_string();
            if let Ok(value) = serde_json::from_str::<Value>(&trimmed) {
                if let Some(failure) = classify_resume_failure(&value, &attempted_id) {
                    return relaunch_after_resume_failure(
                        bootstrap,
                        events,
                        session_store,
                        failure.named_message(),
                    );
                }
                if let Some(session_id) = session_id_from_system_init(&value)
                    && let Some(store) = session_store
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .as_ref()
                {
                    let _ = store.store(&session_id);
                }
            }
            (reader.into_inner(), vec![trimmed])
        }
        Err(error) => relaunch_after_resume_failure(
            bootstrap,
            events,
            session_store,
            format!(
                "reprise Claude impossible: lecture initiale impossible pour l'identifiant {attempted_id} — démarrage d'une session neuve ({error})"
            ),
        ),
    }
}

fn spawn_claude_command(
    command_path: &str,
    args: &[String],
    environment: &[(String, String)],
    inherit_stderr: bool,
) -> Result<Option<SpawnedClaude>, TransportError> {
    spawn_claude_child(
        &ClaudeStreamJsonOptions {
            command: command_path.to_string(),
            args: args.to_vec(),
            provider_kind: "claude".to_string(),
            queue_capacity: 1,
            notify_timeout_secs: 1,
            provider_observation: None,
            session_store_root: None,
            agent_name: None,
        },
        environment,
        inherit_stderr,
    )
}

fn terminate_group(child: &mut Child) {
    // `spawn_with_environment` crée ce groupe avec l'identifiant du fils :
    // le signal ne peut donc toucher ni le wrapper ni un autre équipier.
    let _ = unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
    let _ = child.wait();
}

#[allow(clippy::too_many_arguments)]
fn spawn_worker(
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    writer: Writer,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    journal: Journal,
    alive: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    notify_timeout: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let message = {
                let (lock, wake) = &*queue;
                let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
                while state.messages.is_empty() && !state.closed {
                    state = wake
                        .wait(state)
                        .unwrap_or_else(|poison| poison.into_inner());
                }
                if state.closed {
                    return;
                }
                state.messages.pop_front().expect("file Claude non vide")
            };
            if !alive.load(Ordering::SeqCst) {
                push_internal(
                    &events,
                    ManagedEventKind::DeliveryRejected {
                        message_id: message.id,
                        reason: "équipier Claude arrêté".to_string(),
                    },
                );
                continue;
            }
            let (completion, receiver) = mpsc::channel();
            queue
                .0
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .active = Some(ActiveTurn {
                message_id: message.id.clone(),
                completion,
                response: String::new(),
                text_updates: 0,
                stop_reason: None,
            });
            busy.store(true, Ordering::SeqCst);
            push_internal(
                &events,
                ManagedEventKind::TurnStarted {
                    message_id: message.id.clone(),
                },
            );
            record_or_terminal(
                &journal,
                &events,
                "turn_start",
                Some(&message.id),
                json!({
                    "from": &message.from,
                    "body": message.body,
                }),
            );
            let instructions = private_profile_instructions
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .clone();
            let terminal = match write_input(&writer, &message, instructions.as_deref()) {
                Ok(()) => {
                    push_internal(
                        &events,
                        ManagedEventKind::PromptDispatched {
                            message_id: message.id.clone(),
                        },
                    );
                    record_or_terminal(
                        &journal,
                        &events,
                        "prompt_dispatched",
                        Some(&message.id),
                        json!({
                            "from": &message.from,
                            "body": &message.body,
                        }),
                    );
                    let deadline = message
                        .deadline_at
                        .map(|seconds| UNIX_EPOCH + Duration::from_secs(seconds))
                        .unwrap_or_else(|| SystemTime::now() + notify_timeout);
                    let remaining = deadline
                        .duration_since(SystemTime::now())
                        .unwrap_or(Duration::ZERO);
                    receiver
                        .recv_timeout(remaining)
                        .unwrap_or(ManagedTerminal::Failed {
                            detail: "échéance Claude dépassée".to_string(),
                        })
                }
                Err(error) => ManagedTerminal::Failed {
                    detail: error.to_string(),
                },
            };
            let (response, stop_reason) = {
                let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                match state.active.take() {
                    Some(active) => (active.response, active.stop_reason),
                    None => (String::new(), None),
                }
            };
            let event = match &terminal {
                ManagedTerminal::Completed | ManagedTerminal::Cancelled => {
                    ManagedEventKind::TurnFinished {
                        message: message.clone(),
                        response,
                        terminal: terminal.clone(),
                    }
                }
                ManagedTerminal::Failed { detail } => ManagedEventKind::DeliveryRejected {
                    message_id: message.id.clone(),
                    reason: detail.clone(),
                },
            };
            if matches!(event, ManagedEventKind::TurnFinished { .. }) {
                let stop_reason = stop_reason.unwrap_or_else(|| match &terminal {
                    ManagedTerminal::Cancelled => "cancelled".to_string(),
                    ManagedTerminal::Completed => "completed".to_string(),
                    ManagedTerminal::Failed { detail } => detail.clone(),
                });
                let mut payload = json!({ "stop_reason": stop_reason });
                if message.reply {
                    payload["routed_to"] = json!(&message.from);
                }
                record_or_terminal(&journal, &events, "turn_end", Some(&message.id), payload);
            } else if let ManagedEventKind::DeliveryRejected { reason, .. } = &event {
                // Même borne que ACP/Codex : une échéance DOIT écrire error
                // avec terminal_kind=turn_failed, sinon bridget-idle classe
                // encore OCCUPE un mort des mains.
                record_or_terminal(
                    &journal,
                    &events,
                    "error",
                    Some(&message.id),
                    with_turn_failed_kind(json!({ "reason": reason })),
                );
            }
            push_internal(&events, event);
            busy.store(false, Ordering::SeqCst);
        }
    })
}

fn private_prompt(instructions: Option<&str>, body: &str) -> String {
    let Some(instructions) = instructions
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return body.to_string();
    };
    format!("[Instructions individuelles Bridget]\n{instructions}\n\n[Demande]\n{body}")
}

fn write_input(
    writer: &Writer,
    message: &BridgetMessage,
    instructions: Option<&str>,
) -> Result<(), TransportError> {
    // Même contrat géré qu'ACP : une réponse finale, uniquement si demandée.
    let prompt = private_prompt(instructions, &crate::acp::prompt_for(message));
    let frame = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [{ "type": "text", "text": prompt }],
        },
    });
    let mut writer = writer
        .lock()
        .map_err(|error| TransportError::Io(error.to_string()))?;
    let writer = writer.as_mut().ok_or(TransportError::AgentDead)?;
    writeln!(writer, "{frame}").map_err(|error| TransportError::Io(error.to_string()))?;
    writer
        .flush()
        .map_err(|error| TransportError::Io(error.to_string()))
}

/// Trame de contrôle mesurée le 28/08 : `control_request` / `interrupt` sur
/// l'entrée standard. Claude accuse par `control_response` en 3 ms puis rend un
/// terminal `aborted_tools` ou `aborted_streaming` en 25 ms, sous-processus
/// réellement arrêté, sans aucun signal POSIX. Le tour suivant repart.
fn write_control_interrupt(writer: &Writer, request_id: &str) -> Result<(), TransportError> {
    let frame = json!({
        "type": "control_request",
        "request_id": request_id,
        "request": { "subtype": "interrupt" },
    });
    let mut writer = writer
        .lock()
        .map_err(|error| TransportError::Io(error.to_string()))?;
    let writer = writer.as_mut().ok_or(TransportError::AgentDead)?;
    writeln!(writer, "{frame}").map_err(|error| TransportError::Io(error.to_string()))?;
    writer
        .flush()
        .map_err(|error| TransportError::Io(error.to_string()))
}

/// Terminaux d'interruption Claude. Distincts d'un échec : le tour est coupé à
/// la demande, l'agent reste vivant et enchaîne.
fn interrupt_terminal_reason(reason: &str) -> bool {
    matches!(reason, "aborted_tools" | "aborted_streaming")
}

#[allow(clippy::too_many_arguments)]
fn spawn_reader(
    stdout: ChildStdout,
    prefetch: Vec<String>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    alive: Arc<AtomicBool>,
    journal: Journal,
    session_store: SessionStoreHandle,
    provider_kind: String,
    pinned_model: Option<String>,
    resume_bootstrap: Option<ResumeBootstrap>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (stdout, prefetch) = if let Some(bootstrap) = resume_bootstrap {
            bootstrap_resume_in_reader(stdout, bootstrap, &events, &session_store)
        } else {
            (stdout, prefetch)
        };
        let lines = prefetch
            .into_iter()
            .map(Ok)
            .chain(BufReader::new(stdout).lines());
        let mut pending_writes = std::collections::HashMap::new();
        let observation_cwd = std::env::current_dir().ok();
        for line in lines {
            let Ok(line) = line else { break };
            let raw = line.as_bytes().to_vec();
            let value = match serde_json::from_str::<Value>(&line) {
                Ok(value) => value,
                Err(_) => {
                    push_source(
                        &events,
                        raw,
                        ManagedEventKind::Error {
                            detail: "ligne Claude stream-json invalide".to_string(),
                        },
                    );
                    continue;
                }
            };
            if let Some(session_id) = session_id_from_system_init(&value) {
                if let Some(store) = session_store
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .as_ref()
                {
                    let _ = store.store(&session_id);
                }
                push_source(
                    &events,
                    raw.clone(),
                    ManagedEventKind::ProviderContextObserved {
                        identity: ManagedProviderIdentity {
                            provider_kind: provider_kind.clone(),
                            provider_session_id: Some(session_id),
                            provider_thread_id: None,
                            active_turn_id: None,
                            provider_item_id: None,
                            capabilities_revision: None,
                            provider_observation: None,
                        },
                    },
                );
            }
            let kind = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("inconnu");
            for path in confirmed_tool_writes(&value, &mut pending_writes) {
                let message_id = queue
                    .0
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .active
                    .as_ref()
                    .map(|a| a.message_id.clone());
                if let (Some(message_id), Some(cwd)) = (message_id, observation_cwd.as_deref())
                    && let Some(payload) = crate::journal::confirmed_write_payload(&path, cwd)
                {
                    record_or_terminal(&journal, &events, "update", Some(&message_id), payload);
                }
            }
            // (A) Retranscription : chaque delta texte → journal `update`.
            // Sans cela le fil ne voit que « a travaillé Ns ».
            if let Some(delta) = value.pointer("/event/delta/text").and_then(Value::as_str) {
                let message_id = {
                    let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                    if let Some(active) = state.active.as_mut() {
                        active.response.push_str(delta);
                        active.text_updates = active.text_updates.saturating_add(1);
                        Some(active.message_id.clone())
                    } else {
                        None
                    }
                };
                if let Some(message_id) = message_id {
                    record_or_terminal(
                        &journal,
                        &events,
                        "update",
                        Some(&message_id),
                        json!({ "kind": "text", "content": delta }),
                    );
                }
            }
            // (B) Repli : message `assistant` complet si aucun delta n'a été
            // journalisé (stdout sans include-partial, ou forme agrégée seule).
            //
            // Anti-doublon assistant↔deltas — écarté par écrit, pas par garde
            // supplémentaire : `ensure_stream_arguments` impose
            // `--include-partial-messages`. Sous ce contrat CLI/SDK, les
            // `stream_event` (deltas) arrivent pendant la génération, PUIS le
            // message `assistant` agrégé, PUIS `result`. L'ordre inverse
            // (assistant puis deltas) n'est pas produit ; la garde
            // `text_updates == 0` couvre donc le seul ordre réel
            // (deltas→assistant→result, ou assistant→result sans partial).
            // Une fixture assistant-puis-deltas testerait un fantôme de
            // protocole, pas un trou du pilote.
            if kind == "control_response" {
                // Forme mesurée le 28/08 : {"type":"control_response",
                // "response":{"subtype":"success","request_id":…,
                // "response":{"still_queued":[]}}}. Corrélation stricte : seul
                // l'accusé portant le `request_id` émis solde l'interruption.
                // Un accusé étranger est ignoré plutôt que d'éteindre une
                // interruption qui n'est pas la sienne.
                if let Some(acked) = value
                    .pointer("/response/request_id")
                    .and_then(Value::as_str)
                {
                    let matched = {
                        let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                        if state.pending_interrupt.as_deref() == Some(acked) {
                            state.pending_interrupt = None;
                            true
                        } else {
                            false
                        }
                    };
                    if matched {
                        record_or_terminal(
                            &journal,
                            &events,
                            "interrupt_acked",
                            None,
                            json!({
                                "request_id": acked,
                                "subtype": value
                                    .pointer("/response/subtype")
                                    .and_then(Value::as_str),
                            }),
                        );
                    }
                }
                continue;
            }
            if kind == "assistant" {
                // TOOL : le content_block_start arrive tôt avec name mais
                // input={}. On journalise ici le bloc assistant qui porte
                // name + input complet (mesure /tmp/flux-outil.jsonl L19).
                // Pas de tool_result : un Read de fichier peut saturer le
                // journal ; le fil garde nom + arguments, pas le rendu.
                record_tool_uses_from_assistant(&journal, &events, &queue, &value);
                let assistant_text = value
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .map(|blocks| {
                        blocks
                            .iter()
                            .filter_map(|block| {
                                if block.get("type").and_then(Value::as_str) == Some("text") {
                                    block.get("text").and_then(Value::as_str)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_default();
                if !assistant_text.is_empty() {
                    let message_id = {
                        let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                        if let Some(active) = state.active.as_mut() {
                            if active.text_updates == 0 {
                                if active.response.is_empty() {
                                    active.response.push_str(&assistant_text);
                                }
                                active.text_updates = active.text_updates.saturating_add(1);
                                Some(active.message_id.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    if let Some(message_id) = message_id {
                        record_or_terminal(
                            &journal,
                            &events,
                            "update",
                            Some(&message_id),
                            json!({ "kind": "text", "content": assistant_text }),
                        );
                    }
                }
            }
            if kind == "result" {
                let terminal = if value.get("is_error").and_then(Value::as_bool) == Some(false)
                    && value.get("terminal_reason").and_then(Value::as_str) == Some("completed")
                {
                    ManagedTerminal::Completed
                } else if value
                    .get("terminal_reason")
                    .and_then(Value::as_str)
                    .is_some_and(interrupt_terminal_reason)
                {
                    // `aborted_tools` / `aborted_streaming` : le tour a été
                    // coupé à la demande. C'est une annulation, pas un échec —
                    // sans quoi bridget-idle classerait l'agent en bloqué.
                    ManagedTerminal::Cancelled
                } else {
                    ManagedTerminal::Failed {
                        detail: value
                            .get("terminal_reason")
                            .and_then(Value::as_str)
                            .unwrap_or("terminal Claude inconnu")
                            .to_string(),
                    }
                };
                // (C) Issue attestée : `terminal_reason` du résultat Claude
                // (ex. "completed"), pas le champ stop_reason fournisseur
                // (ex. "end_turn") — attach lit payload.stop_reason.
                let stop_reason = value
                    .get("terminal_reason")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|reason| !reason.is_empty())
                    .map(str::to_string)
                    .or_else(|| {
                        value
                            .get("stop_reason")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|reason| !reason.is_empty())
                            .map(str::to_string)
                    });
                let pending_result_update = {
                    let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                    if let Some(active) = state.active.as_mut() {
                        active.stop_reason = stop_reason;
                        // (B) Repli result-only : texte final dans le fil SSI
                        // aucun update texte n'a déjà été journalisé (pas de
                        // duplication après deltas).
                        if active.text_updates == 0
                            && let Some(result) = value.get("result").and_then(Value::as_str)
                            && !result.is_empty()
                        {
                            if active.response.is_empty() {
                                active.response.push_str(result);
                            }
                            active.text_updates = 1;
                            Some((active.message_id.clone(), result.to_string()))
                        } else {
                            if active.response.is_empty()
                                && let Some(result) = value.get("result").and_then(Value::as_str)
                            {
                                active.response.push_str(result);
                            }
                            None
                        }
                    } else {
                        None
                    }
                };
                if let Some((message_id, result)) = pending_result_update {
                    // (B) Réponse finale absente des deltas → journaliser `result`.
                    record_or_terminal(
                        &journal,
                        &events,
                        "update",
                        Some(&message_id),
                        json!({ "kind": "text", "content": result }),
                    );
                }
                let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                if let Some(active) = state.active.as_mut() {
                    let _ = active.completion.send(terminal);
                }
            }
            let managed_event = if kind == "rate_limit_event" {
                rate_limit_event(&value).unwrap_or_else(|| ManagedEventKind::Update {
                    detail: "événement Claude rate_limit_event incomplet".to_string(),
                })
            } else if let Some(served) = served_model_from_claude(&value) {
                maybe_record_mismatch(&journal, &events, pinned_model.as_deref(), &served);
                ManagedEventKind::ModelObserved { model: served }
            } else if let Some(usage) = usage_event(&value) {
                usage
            } else {
                ManagedEventKind::Update {
                    detail: format!("événement Claude: {kind}"),
                }
            };
            push_source(&events, raw, managed_event);
        }
        alive.store(false, Ordering::SeqCst);
        let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(active) = state.active.take() {
            let _ = active.completion.send(ManagedTerminal::Failed {
                detail: "stdout Claude fermé".to_string(),
            });
        }
        queue.1.notify_all();
    })
}

/// Extrait uniquement un fait complet du schéma `rate_limit_event` attesté par
/// Claude. Une date absente reste `None` : aucune heure de retour n'est déduite.
fn pinned_model_from_args(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == "--model")
        .map(|pair| {
            pair[1]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|model| !model.is_empty())
}

/// Le modèle servi n'est lu que sur `system/init`. Un autre événement, même
/// porteur d'un champ `model`, ne constitue pas un verdict.
fn served_model_from_claude(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("system") {
        return None;
    }
    if value.get("subtype").and_then(Value::as_str) != Some("init") {
        return None;
    }
    value
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_string)
}

fn maybe_record_mismatch(
    journal: &Journal,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    pinned: Option<&str>,
    served: &str,
) {
    let Some(pinned) = pinned.filter(|pinned| *pinned != served) else {
        return;
    };
    record_or_terminal(
        journal,
        events,
        "model_mismatch",
        None,
        json!({ "pinned": pinned, "served": served }),
    );
}

fn rate_limit_event(value: &Value) -> Option<ManagedEventKind> {
    let info = value.get("rate_limit_info")?;
    let window = info.get("rateLimitType")?.as_str()?.trim();
    let status = info.get("status")?.as_str()?.trim();
    if window.is_empty() || status.is_empty() {
        return None;
    }
    Some(ManagedEventKind::RateLimitObserved {
        window: window.to_string(),
        status: status.to_string(),
        resets_at: info.get("resetsAt").and_then(Value::as_i64),
        used_percent: percent_0_100(info.get("usedPercent").or_else(|| info.get("utilization"))),
    })
}

fn percent_0_100(value: Option<&Value>) -> Option<u8> {
    let pct = value.and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))?;
    if (0.0..=100.0).contains(&pct) {
        Some(pct.round() as u8)
    } else {
        None
    }
}

/// Extrait la consommation d'un tour depuis l'événement terminal `result`.
///
/// Un tour Claude émet souvent `assistant` (usage sous `/message/usage`) puis
/// `result` (usage à la racine) avec les mêmes compteurs. Ne lire que `result`
/// : un seul échantillon par tour, jamais un facturable doublé. Schéma attesté
/// complet requis ; un champ manquant → aucun fait (jamais zéro inventé).
fn usage_event(value: &Value) -> Option<ManagedEventKind> {
    if value.get("type").and_then(Value::as_str) != Some("result") {
        return None;
    }
    let usage = value
        .get("usage")
        .or_else(|| value.pointer("/message/usage"))?;
    let input_tokens = usage.get("input_tokens")?.as_u64()?;
    let output_tokens = usage.get("output_tokens")?.as_u64()?;
    let cache_creation_input_tokens = usage.get("cache_creation_input_tokens")?.as_u64()?;
    let cache_read_input_tokens = usage.get("cache_read_input_tokens")?.as_u64()?;
    Some(ManagedEventKind::UsageObserved {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens,
        cache_read_input_tokens,
    })
}

/// Plafond du champ `detail` (input sérialisé). Au-delà : troncature + «…».
/// Choix étroit : garder le journal lisible si un outil reçoit un gros blob.
const TOOL_INPUT_DETAIL_MAX: usize = 512;

fn confirmed_tool_writes(
    value: &Value,
    pending: &mut std::collections::HashMap<String, Option<String>>,
) -> Vec<String> {
    let kind = value.get("type").and_then(Value::as_str);
    if kind == Some("result") {
        pending.clear();
        return vec![];
    }
    let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
        return vec![];
    };
    let mut paths = vec![];
    for block in blocks {
        if kind == Some("assistant")
            && block.get("type").and_then(Value::as_str) == Some("tool_use")
        {
            let Some(id) = block
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| id.len() <= 256)
            else {
                continue;
            };
            let name = block.get("name").and_then(Value::as_str).unwrap_or("");
            if let Some(path) = crate::journal::tool_write_path(name, &block["input"]) {
                if pending.len() >= 256 && !pending.contains_key(id) {
                    log::warn!("observation_gap: trop d'écritures Claude dans ce tour");
                    continue;
                }
                pending
                    .entry(id.into())
                    .or_insert_with(|| Some(path.into()));
            }
        } else if kind == Some("user")
            && block.get("type").and_then(Value::as_str) == Some("tool_result")
            && let Some(id) = block.get("tool_use_id").and_then(Value::as_str)
            && let Some(path) = pending.get_mut(id).and_then(Option::take)
            && block.get("is_error").and_then(Value::as_bool) != Some(true)
        {
            paths.push(path);
        }
    }
    paths
}

/// Journalise chaque bloc `tool_use` d'un message assistant.
/// Vocabulaire imposé : `kind=tool` (pas `tool_call`).
fn record_tool_uses_from_assistant(
    journal: &Journal,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    value: &Value,
) {
    let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
        return;
    };
    let message_id = {
        let state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
        state
            .active
            .as_ref()
            .map(|active| active.message_id.clone())
    };
    let Some(message_id) = message_id else {
        return;
    };
    for block in blocks {
        if block.get("type").and_then(Value::as_str) != Some("tool_use") {
            continue;
        }
        let Some(name) = block
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        let input = block.get("input").cloned().unwrap_or(Value::Null);
        let detail = compact_tool_input(&input);
        record_or_terminal(
            journal,
            events,
            "update",
            Some(&message_id),
            json!({
                "kind": crate::act_kind::pilot_kind_str(crate::JournalUpdateKind::Tool),
                "text": name,
                "tool": name,
                "detail": detail,
            }),
        );
    }
}

fn compact_tool_input(input: &Value) -> String {
    let raw = match input {
        Value::Null => String::new(),
        other => other.to_string(),
    };
    if raw.chars().count() <= TOOL_INPUT_DETAIL_MAX {
        return raw;
    }
    let truncated: String = raw.chars().take(TOOL_INPUT_DETAIL_MAX).collect();
    format!("{truncated}…")
}

fn record(
    journal: &Journal,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) -> Result<(), String> {
    journal
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .as_ref()
        .map_or(Ok(()), |journal| {
            journal.enqueue(event, message_id, payload)
        })
}

/// Miroir ACP `record_or_terminal` : un échec d'enqueue (dont kind hors
/// vocabulaire) devient `JournalFailed` — visible, jamais avalé en silence.
fn record_or_terminal(
    journal: &Journal,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) {
    if let Err(detail) = record(journal, event, message_id, payload) {
        push_internal(events, ManagedEventKind::JournalFailed { detail });
    }
}

fn push_internal(events: &Arc<Mutex<VecDeque<ManagedEvent>>>, kind: ManagedEventKind) {
    let raw = serde_json::to_vec(&json!({ "kind": "claude_internal" }))
        .expect("fait interne Claude sérialisable");
    events
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .push_back(ManagedEvent::internal(
            ManagedEventSource::ClaudeStreamJson,
            raw,
            kind,
        ));
}

fn push_source(events: &Arc<Mutex<VecDeque<ManagedEvent>>>, raw: Vec<u8>, kind: ManagedEventKind) {
    events
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .push_back(ManagedEvent {
            source: ManagedEventSource::ClaudeStreamJson,
            origin: ManagedEventOrigin::SourceLine,
            raw,
            kind,
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn spec100_claude_requires_success_and_keeps_path_before_truncation() {
        let mut pending = std::collections::HashMap::new();
        let tool = serde_json::json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"w","name":"Write","input":{"content":"x".repeat(2000),"file_path":"/project/x"}}]}});
        assert!(super::confirmed_tool_writes(&tool, &mut pending).is_empty());
        let done = serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"w","is_error":false}]}});
        assert_eq!(
            super::confirmed_tool_writes(&done, &mut pending),
            vec!["/project/x"]
        );
        assert!(super::confirmed_tool_writes(&done, &mut pending).is_empty());
        super::confirmed_tool_writes(&tool, &mut pending);
        assert!(
            super::confirmed_tool_writes(&done, &mut pending).is_empty(),
            "une répétition assistant/résultat ne recrée pas l'écriture"
        );
        super::confirmed_tool_writes(&serde_json::json!({"type":"result"}), &mut pending);
        super::confirmed_tool_writes(&tool, &mut pending);
        let failed = serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"w","is_error":true}]}});
        assert!(super::confirmed_tool_writes(&failed, &mut pending).is_empty());
        assert!(super::confirmed_tool_writes(&done, &mut pending).is_empty());
    }
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
    use std::time::Instant;

    static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn spec105_claude_trame_transmet_le_contrat_et_le_corps() {
        for requested in [false, true] {
            let mut child = Command::new("/bin/cat")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let writer = Arc::new(Mutex::new(child.stdin.take()));
            let mut message = BridgetMessage::new("alice", "bob", "Texte utile\nligne suivante");
            message.reply = requested;
            message.in_reply_to = Some("parent".into());
            write_input(&writer, &message, Some("Instruction privée")).unwrap();
            drop(writer);
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            let frame: Value = serde_json::from_slice(&output.stdout).unwrap();
            let prompt = frame["message"]["content"][0]["text"].as_str().unwrap();
            assert!(prompt.contains("Instruction privée"));
            assert!(prompt.contains(&message.id));
            assert!(prompt.contains("alice"));
            assert!(prompt.contains("parent"));
            assert!(prompt.ends_with(&message.body));
            assert_eq!(
                prompt.contains("Aucune réponse inter-agent attendue"),
                !requested
            );
            assert_eq!(prompt.contains("relaie automatiquement"), requested);
        }
    }

    fn options() -> ClaudeStreamJsonOptions {
        ClaudeStreamJsonOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                concat!(
                    "while IFS= read -r line; do ",
                    "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"status\",\"status\":\"requesting\"}'; ",
                    "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"réponse Claude\"}}}'; ",
                    "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"réponse Claude\"}'; ",
                    "done"
                )
                .to_string(),
            ],
            provider_kind: "claude".to_string(),
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            session_store_root: None,
            agent_name: None,
        }
    }

    #[test]
    fn provider_kind_reste_celui_declare_par_le_registre() {
        for provider_kind in ["claude", "anthropic", "glm", "deepseek"] {
            let mut configured = options();
            configured.provider_kind = provider_kind.to_string();
            let transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
            let contexts = transport
                .drain_events()
                .into_iter()
                .filter_map(|event| match event.kind {
                    ManagedEventKind::ProviderContextObserved { identity } => {
                        Some(identity.provider_kind)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(contexts, vec![provider_kind.to_string()]);
            assert_eq!(
                transport.provider_identity().unwrap().provider_kind,
                provider_kind
            );
            transport.stop();
        }
    }

    fn message(id: &str) -> BridgetMessage {
        let mut message = BridgetMessage::new("bridget", "claude", "mission");
        message.id = id.to_string();
        message.reply = true;
        message
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_claude_refus_update_hors_vocabulaire_emet_JournalFailed() {
        // Preuve d'abord : écriture valide passe sans JournalFailed.
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-act-kind-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::SeqCst)
        ));
        fs::create_dir_all(&root).unwrap();
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let journal = Arc::new(Mutex::new(Some(
            JournalWriter::start(
                &root,
                "claude-1",
                "session-1",
                Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
            )
            .unwrap(),
        )));
        record_or_terminal(
            &journal,
            &events,
            "update",
            Some("m1"),
            json!({"kind":"tool","text":"Read"}),
        );
        assert!(
            events.lock().unwrap().is_empty(),
            "écriture tool valide ne doit pas émettre JournalFailed"
        );
        record_or_terminal(
            &journal,
            &events,
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        let drained: Vec<_> = events.lock().unwrap().drain(..).collect();
        assert!(
            drained.iter().any(|event| matches!(
                &event.kind,
                ManagedEventKind::JournalFailed { detail }
                    if detail.contains("hors vocabulaire")
            )),
            "refus hors vocabulaire doit être visible via JournalFailed, got {drained:?}"
        );
        if let Some(writer) = journal.lock().unwrap().take() {
            writer.stop();
        }
        let _ = fs::remove_dir_all(root);
    }

    /// Dette 3 — oracle bout-en-bout : le PILOTE (tool_use → record_tool_uses)
    /// écrit un kind interdit ; l'événement terminal JournalFailed apparaît
    /// dans drain_events. Pas un appel direct à record_or_terminal.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_claude_pilote_kind_interdit_emet_JournalFailed_bout_en_bout() {
        // Preuve d'abord : cas valide → tool au journal, PAS de JournalFailed.
        let (events_ok, journal_ok) = {
            let _suite = crate::act_kind::pilot_kind_suite_lock();
            claude_tool_turn_managed_events()
        };
        assert!(
            events_ok
                .iter()
                .all(|event| { !matches!(event.kind, ManagedEventKind::JournalFailed { .. }) }),
            "cas valide ne doit pas émettre JournalFailed, got {events_ok:?}"
        );
        assert!(
            journal_ok
                .iter()
                .any(|event| { event["event"] == "update" && event["payload"]["kind"] == "tool" }),
            "cas valide doit journaliser kind=tool, got {journal_ok:?}"
        );

        // Chemin réel : provider tool_use → pilote → enqueue refuse → JournalFailed.
        let (events_bad, journal_bad) = crate::act_kind::with_forced_pilot_update_kind(
            "intent",
            claude_tool_turn_managed_events,
        );
        assert!(
            events_bad.iter().any(|event| matches!(
                &event.kind,
                ManagedEventKind::JournalFailed { detail }
                    if detail.contains("hors vocabulaire")
            )),
            "kind interdit du pilote doit être VISIBLE via JournalFailed, got {events_bad:?}"
        );
        assert!(
            journal_bad
                .iter()
                .all(|event| { event["payload"]["kind"] != "intent" }),
            "le kind refusé ne doit jamais atteindre le JSONL, got {journal_bad:?}"
        );
    }

    fn claude_tool_turn_managed_events() -> (Vec<ManagedEvent>, Vec<Value>) {
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-e2e-act-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let mut opts = options();
        opts.args[1] = concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_demo\",\"name\":\"Read\",\"input\":{\"file_path\":\"/tmp/demo.toml\",\"limit\":1}}]}}'; ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"ok\"}'; ",
            "done"
        )
        .to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(opts).unwrap();
        transport
            .activate_journal(&root, "claude-e2e", None)
            .unwrap();
        transport.deliver(&message("claude-e2e-1")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Completed,
                        ..
                    }
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Drainer le reste (JournalFailed peut arriver juste avant/après fin).
        events.extend(transport.drain_events());
        transport.stop();
        let journal_path = fs::read_dir(root.join("claude-e2e"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let journal_events = crate::journal::valid_events(&journal_path);
        let _ = fs::remove_dir_all(&root);
        (events, journal_events)
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_avale_err_record_tue_TEMOIN_claude_JournalFailed() {
        fn broken_swallow(
            journal: &Journal,
            _events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
            event: &str,
            message_id: Option<&str>,
            payload: Value,
        ) {
            let _ = record(journal, event, message_id, payload);
        }
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-act-kind-mutant-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::SeqCst)
        ));
        fs::create_dir_all(&root).unwrap();
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let journal = Arc::new(Mutex::new(Some(
            JournalWriter::start(
                &root,
                "claude-1",
                "session-1",
                Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
            )
            .unwrap(),
        )));
        broken_swallow(
            &journal,
            &events,
            "update",
            Some("m1"),
            json!({"kind":"intent"}),
        );
        assert!(
            events.lock().unwrap().is_empty(),
            "le mutant avale l'erreur"
        );
        record_or_terminal(
            &journal,
            &events,
            "update",
            Some("m1"),
            json!({"kind":"intent"}),
        );
        assert!(
            events.lock().unwrap().iter().any(|event| matches!(
                &event.kind,
                ManagedEventKind::JournalFailed { detail }
                    if detail.contains("hors vocabulaire")
            )),
            "TEMOIN_claude_JournalFailed doit mourir si Err est avalée"
        );
        if let Some(writer) = journal.lock().unwrap().take() {
            writer.stop();
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn session_native_preserve_les_lignes_et_termine_le_tour() {
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-native-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut transport = ClaudeStreamJsonTransport::spawn(options()).unwrap();
        transport
            .activate_journal(&root, "claude-native", None)
            .unwrap();
        transport.deliver(&message("claude-1")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        ref response,
                        terminal: ManagedTerminal::Completed,
                        ..
                    } if response == "réponse Claude"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(event.kind, ManagedEventKind::PromptDispatched { ref message_id } if message_id == "claude-1")
        }));
        let raw = events
            .iter()
            .find(|event| event.raw.starts_with(br#"{"type":"stream_event""#))
            .expect("notification stream-event source");
        assert_eq!(raw.source, ManagedEventSource::ClaudeStreamJson);
        assert_eq!(raw.origin, ManagedEventOrigin::SourceLine);
        // Même ligne que le fournisseur historique dfa2134, sans sa seule
        // terminaison LF. Une reconstruction sémantique ne suffit pas.
        assert_eq!(
            raw.raw.as_slice(),
            include_bytes!("../../../fixtures/native-claude-delta.jsonl")
                .strip_suffix(b"\n")
                .expect("fixture JSONL terminée par LF")
        );
        transport.stop();
        let _ = fs::remove_dir_all(root);
    }

    /// (A) Témoin retranscription : deltas → `update` journal avec TEXTE EXACT.
    /// Mutant : retirer `record(..., "update", ...)` sur `/event/delta/text`,
    /// ou vider `content` (`content:""`) — A doit mourir (présence ≠ propriété).
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_A_claude_stream_json_retranscrit_les_deltas_en_update() {
        let events = journal_fixture_deltas();
        let updates: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update")
            .collect();
        assert!(
            updates
                .iter()
                .all(|event| event["payload"]["kind"] == "text")
        );
        let contents: Vec<&str> = updates
            .iter()
            .filter_map(|event| event["payload"]["content"].as_str())
            .collect();
        // Contenu en dur (comme B) : cardinal+kind seuls laissent passer content:"".
        assert_eq!(
            contents,
            vec!["BONJOUR ", "JE SUIS VIVANT"],
            "deux deltas exacts, pas de troisième update issu du result: {contents:?} via {events:?}"
        );
        assert!(
            updates
                .iter()
                .all(|event| event["message_id"] == "claude-abc-1"),
            "même message_id que le tour, reçu {updates:?}"
        );
    }

    /// (B) Témoin M1 : flux result-only — le TEXTE FINAL EXACT dans le fil,
    /// même message_id. Mutant : retirer le repli result→record(update)
    /// (text_updates==0) → left=[] / right=["REPONSE FINALE"].
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_B_claude_stream_json_reponse_finale_result_only_dans_le_fil() {
        let events = journal_fixture_result_only();
        let updates: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update")
            .collect();
        let contents: Vec<&str> = updates
            .iter()
            .filter_map(|event| event["payload"]["content"].as_str())
            .collect();
        assert_eq!(
            contents,
            vec!["REPONSE FINALE"],
            "repli result-only → update exact, reçu {contents:?} via {events:?}"
        );
        assert!(
            updates
                .iter()
                .all(|event| event["message_id"] == "claude-abc-1"),
            "même message_id que le tour, reçu {updates:?}"
        );
    }

    /// (C) Témoin M2 : turn_end.payload.stop_reason = issue attestée
    /// (`completed` = terminal_reason fournisseur). Mutant : turn_end {} →
    /// attach affiche « inconnu » (left=Null / right="completed").
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_C_claude_stream_json_turn_end_porte_stop_reason_completed() {
        let events = journal_fixture_result_only();
        let turn_end = events
            .iter()
            .find(|event| event["event"] == "turn_end")
            .expect("turn_end attendu");
        assert_eq!(
            turn_end["payload"]["stop_reason"].as_str(),
            Some("completed"),
            "stop_reason manquant → attach affiche « inconnu », payload={}",
            turn_end["payload"]
        );
        assert_eq!(
            turn_end["payload"]["routed_to"].as_str(),
            Some("bridget"),
            "reply=true doit porter routed_to, payload={}",
            turn_end["payload"]
        );
    }

    /// (TOOL) Appel d'outil Claude → update kind=tool avec NOM + CONTENU input.
    /// Source : message assistant (input complet), pas content_block_start (input {}).
    /// Mutant : retirer record_tool_uses_from_assistant, ou vider text/detail → meurt.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_TOOL_claude_stream_json_retranscrit_tool_use_nom_et_contenu() {
        let events = journal_fixture_tool_use();
        let tools: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "tool")
            .collect();
        assert_eq!(
            tools.len(),
            1,
            "un seul acte tool attendu, reçu {tools:?} via {events:?}"
        );
        assert_eq!(
            tools[0]["payload"]["text"].as_str(),
            Some("Read"),
            "nom d'outil en dur, payload={}",
            tools[0]["payload"]
        );
        assert_eq!(
            tools[0]["payload"]["tool"].as_str(),
            Some("Read"),
            "champ tool en dur, payload={}",
            tools[0]["payload"]
        );
        assert_eq!(
            tools[0]["payload"]["detail"].as_str(),
            Some(r#"{"file_path":"/tmp/demo.toml","limit":1}"#),
            "input complet en dur (pas une présence vide), payload={}",
            tools[0]["payload"]
        );
        assert_eq!(
            tools[0]["message_id"].as_str(),
            Some("claude-abc-1"),
            "même message_id, reçu {tools:?}"
        );
    }

    fn journal_fixture_deltas() -> Vec<Value> {
        journal_fixture(concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"BONJOUR \"}}}'; ",
            "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"JE SUIS VIVANT\"}}}'; ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"BONJOUR JE SUIS VIVANT\"}'; ",
            "done"
        ))
    }

    fn journal_fixture_result_only() -> Vec<Value> {
        journal_fixture(concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"REPONSE FINALE\"}'; ",
            "done"
        ))
    }

    fn journal_fixture_tool_use() -> Vec<Value> {
        let _suite = crate::act_kind::pilot_kind_suite_lock();
        // Forme mesurée L19 de /tmp/flux-outil.jsonl : assistant + tool_use + input.
        journal_fixture(concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_demo\",\"name\":\"Read\",\"input\":{\"file_path\":\"/tmp/demo.toml\",\"limit\":1}}]}}'; ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"ok\"}'; ",
            "done"
        ))
    }

    fn journal_fixture(provider_script: &str) -> Vec<Value> {
        // Compteur atomique : sous parallelisme, SystemTime::nanos peut
        // coincider et un remove_dir_all efface le journal d'un autre témoin.
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-abc-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let mut opts = options();
        opts.args[1] = provider_script.to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(opts).unwrap();
        transport
            .activate_journal(&root, "claude-abc", None)
            .unwrap();
        transport.deliver(&message("claude-abc-1")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut finished = false;
        while Instant::now() < deadline {
            finished |= transport.drain_events().iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Completed,
                        ..
                    }
                )
            });
            if finished {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(finished, "le tour Claude n'a pas terminé");
        transport.stop();
        let journal_path = fs::read_dir(root.join("claude-abc"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let events = crate::journal::valid_events(&journal_path);
        let _ = fs::remove_dir_all(&root);
        events
    }

    #[test]
    fn formats_fournisseur_imposes_ne_peuvent_pas_etre_ecrases() {
        let mut args = vec!["--output-format".to_string(), "json".to_string()];
        assert!(ensure_stream_arguments(&mut args).is_err());
    }

    /// Contrat changé le 28/08, sur mesures : jusqu'ici, faute de trame
    /// d'interruption attestée, l'annulation d'un tour actif tuait le groupe de
    /// processus. Claude accuse un `control_request` en 3 ms et rend un
    /// terminal `aborted_tools` en 25 ms, sous-processus arrêté sans signal
    /// POSIX, tour suivant réussi en 1,594 s. On coupe donc le tour SANS tuer
    /// l'agent — c'est déjà ce que fait Codex (`codex_app_server.rs`), qui
    /// signale et laisse le worker émettre `turn/interrupt`.
    ///
    /// Le faux fournisseur répond ici exactement les formes mesurées.
    #[test]
    fn message_humain_actif_interrompt_claude_et_declenche_la_remise() {
        let mut slow = options();
        slow.args[1] = concat!(
            "while IFS= read -r line; do case \"$line\" in ",
            "*control_request*) ",
            "rid=$(printf '%s' \"$line\" | sed 's/.*\"request_id\":\"\\([^\"]*\\)\".*/\\1/'); ",
            "printf '{\"type\":\"control_response\",\"response\":{\"subtype\":\"success\",",
            "\"request_id\":\"%s\",\"response\":{\"still_queued\":[]}}}\\n' \"$rid\"; ",
            "printf '{\"type\":\"result\",\"is_error\":true,",
            "\"terminal_reason\":\"aborted_tools\"}\\n';; ",
            "*) : ;; esac; done"
        )
        .to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(slow).unwrap();
        transport.deliver(&message("claude-actif")).unwrap();
        let start_deadline = Instant::now() + Duration::from_secs(2);
        while !transport.is_busy() && Instant::now() < start_deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            transport.is_busy(),
            "le tour Claude initial doit être actif"
        );

        let mut human = message("claude-humain");
        human.from = "superviseur".to_string();
        human.origin = Some(MessageOrigin::Human);
        transport.deliver(&human).unwrap();

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut interrupted = false;
        let mut human_dispatched = false;
        while Instant::now() < deadline {
            for event in transport.drain_events() {
                interrupted |= matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        ref message,
                        terminal: ManagedTerminal::Cancelled,
                        ..
                    } if message.id == "claude-actif"
                );
                human_dispatched |= matches!(
                    event.kind,
                    ManagedEventKind::PromptDispatched { ref message_id }
                        if message_id == "claude-humain"
                );
            }
            if interrupted && human_dispatched {
                assert!(
                    transport.is_alive(),
                    "Claude doit rester utilisable après interruption"
                );
                transport.stop();
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        transport.stop();
        panic!(
            "le message humain devait interrompre le tour actif puis être remis, interruption={interrupted}, remis={human_dispatched}"
        );
    }

    #[test]
    fn interruption_active_coupe_le_tour_sans_tuer_l_agent_claude() {
        let mut slow = options();
        slow.args[1] = concat!(
            "while IFS= read -r line; do case \"$line\" in ",
            "*control_request*) ",
            "rid=$(printf '%s' \"$line\" | sed 's/.*\"request_id\":\"\\([^\"]*\\)\".*/\\1/'); ",
            "printf '{\"type\":\"control_response\",\"response\":{\"subtype\":\"success\",",
            "\"request_id\":\"%s\",\"response\":{\"still_queued\":[]}}}\\n' \"$rid\"; ",
            "printf '{\"type\":\"result\",\"is_error\":true,",
            "\"terminal_reason\":\"aborted_tools\"}\\n';; ",
            "*) : ;; esac; done"
        )
        .to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(slow).unwrap();
        transport.deliver(&message("claude-cancel")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !transport.is_busy() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(transport.is_busy());
        assert!(transport.cancel_delivery("claude-cancel", "annulé par le daemon"));

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Cancelled,
                        ..
                    }
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Assertion métier 1 : `aborted_tools` est un terminal d'annulation,
        // pas un échec — sinon l'agent serait classé bloqué par la ronde.
        assert!(
            events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Cancelled,
                        ..
                    }
                )
            }),
            "terminal d'interruption absent : {events:?}"
        );
        // Assertion métier 2 : c'est tout l'objet de la trame — l'agent
        // survit à l'interruption et peut enchaîner le tour suivant.
        assert!(
            transport.is_alive(),
            "l'agent a été tué au lieu d'être interrompu"
        );
        transport.stop();
    }

    #[test]
    fn terminal_fournisseur_en_erreur_refuse_la_livraison() {
        let mut failing = options();
        failing.args[1] = concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":true,",
            "\"terminal_reason\":\"api_error\",\"result\":\"authentification refusée\"}'; ",
            "done"
        )
        .to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(failing).unwrap();
        transport.deliver(&message("claude-reject")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == "claude-reject" && reason == "api_error"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "claude-reject" && reason == "api_error"
            )
        }));
        transport.stop();
    }

    #[test]
    fn rate_limit_event_reel_devient_un_fait_sans_inventer_de_retour() {
        let event: Value = serde_json::from_str(
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"rejected","resetsAt":1787572200,"rateLimitType":"five_hour"}}"#,
        )
        .unwrap();
        assert!(matches!(
            rate_limit_event(&event),
            Some(ManagedEventKind::RateLimitObserved {
                ref window,
                ref status,
                resets_at: Some(1_787_572_200),
                used_percent: None,
            }) if window == "five_hour" && status == "rejected"
        ));

        let no_reset: Value = serde_json::from_str(
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","rateLimitType":"five_hour"}}"#,
        )
        .unwrap();
        assert!(matches!(
            rate_limit_event(&no_reset),
            Some(ManagedEventKind::RateLimitObserved {
                resets_at: None,
                used_percent: None,
                ..
            })
        ));
    }

    #[test]
    fn init_annonce_un_modele_distinct_et_le_journalise() {
        let event: Value =
            serde_json::from_str(r#"{"type":"system","subtype":"init","model":"claude-opus-4-6"}"#)
                .unwrap();
        assert_eq!(
            served_model_from_claude(&event).as_deref(),
            Some("claude-opus-4-6")
        );

        let mut mismatch = options();
        mismatch.args = vec![
            "-c".to_string(),
            concat!(
                "while IFS= read -r line; do ",
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-4-6\"}'; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                "done"
            )
            .to_string(),
            "--model".to_string(),
            "claude-opus-5".to_string(),
        ];
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-mismatch-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut transport = ClaudeStreamJsonTransport::spawn(mismatch).unwrap();
        transport
            .activate_journal(&root, "claude-mismatch", None)
            .unwrap();
        transport.deliver(&message("claude-gap")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-4-6"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-4-6"
            )
        }));
        transport.stop();
        let contents = fs::read_dir(root.join("claude-mismatch"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<String>();
        assert!(
            contents.contains("\"event\":\"model_mismatch\""),
            "journal sans écart: {contents}"
        );
        assert!(contents.contains("claude-opus-5"));
        assert!(contents.contains("claude-opus-4-6"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn flux_muet_ou_modele_identique_ne_produit_aucun_ecart() {
        assert!(
            served_model_from_claude(
                &serde_json::from_str(
                    r#"{"type":"system","subtype":"status","status":"requesting"}"#
                )
                .unwrap()
            )
            .is_none()
        );
        assert!(
            served_model_from_claude(
                &serde_json::from_str(r#"{"type":"system","subtype":"init"}"#).unwrap()
            )
            .is_none()
        );

        let mut matching = options();
        matching.args = vec![
            "-c".to_string(),
            concat!(
                "while IFS= read -r line; do ",
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-5\"}'; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                "done"
            )
            .to_string(),
            "--model".to_string(),
            "claude-opus-5".to_string(),
        ];
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-match-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut transport = ClaudeStreamJsonTransport::spawn(matching).unwrap();
        transport
            .activate_journal(&root, "claude-match", None)
            .unwrap();
        transport.deliver(&message("claude-ok")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events
                .iter()
                .any(|event| matches!(event.kind, ManagedEventKind::ModelObserved { .. }))
            {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-5"
            )
        }));
        transport.stop();
        let contents = fs::read_dir(root.join("claude-match"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<String>();
        assert!(
            !contents.contains("model_mismatch"),
            "écart inventé: {contents}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn usage_result_reel_devient_un_fait_complet() {
        let event: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}"#,
        )
        .unwrap();
        assert!(matches!(
            usage_event(&event),
            Some(ManagedEventKind::UsageObserved {
                input_tokens: 2,
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
            })
        ));
    }

    #[test]
    fn usage_incomplet_ne_devient_pas_un_zero_invente() {
        let event: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":10}}"#,
        )
        .unwrap();
        assert!(usage_event(&event).is_none());
    }

    #[test]
    fn usage_assistant_puis_result_ne_compte_qu_un_echantillon() {
        let assistant: Value = serde_json::from_str(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#,
        )
        .unwrap();
        let result: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}"#,
        )
        .unwrap();
        let counted = [assistant, result]
            .iter()
            .filter(|event| usage_event(event).is_some())
            .count();
        assert_eq!(
            counted, 1,
            "un tour assistant+result ne doit poser qu'un échantillon"
        );
        assert!(usage_event(&serde_json::from_str(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#,
        )
        .unwrap())
        .is_none());
    }

    /// P2 bout-en-bout : `--resume` fantôme → message nommé + session neuve.
    /// Mutant : avaler l'échec sans Error event → oracle meurt seul.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_P2_spawn_repli_nomme_apres_echec_resume() {
        use crate::claude_provider_session::CLAUDE_RESUME_FAILED_PREFIX;
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-resume-fail-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let fake = root.join("fake-claude");
        fs::write(
            &fake,
            r#"#!/bin/sh
if [ "$1" = "--resume" ]; then
  printf '%s\n' '{"type":"result","subtype":"error_during_execution","is_error":true,"errors":["No conversation found with session ID: 00000000-0000-0000-0000-000000000000"]}'
  exit 1
fi
printf '%s\n' '{"type":"system","subtype":"init","session_id":"fresh-session-999","model":"claude-opus-5"}'
while IFS= read -r line; do
  printf '%s\n' '{"type":"result","is_error":false,"terminal_reason":"completed","result":"ok"}'
done
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&fake).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&fake, permissions).unwrap();
        }
        let store = crate::claude_provider_session::ProviderSessionStore::new(&root, "agent-x");
        store.store("00000000-0000-0000-0000-000000000000").unwrap();
        let options = ClaudeStreamJsonOptions {
            command: fake.to_string_lossy().into_owned(),
            args: Vec::new(),
            provider_kind: "claude".to_string(),
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            session_store_root: Some(root.clone()),
            agent_name: Some("agent-x".to_string()),
        };
        let transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    &event.kind,
                    ManagedEventKind::Error { detail }
                        if detail.starts_with(CLAUDE_RESUME_FAILED_PREFIX)
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let notice = events.iter().find_map(|event| match &event.kind {
            ManagedEventKind::Error { detail } => Some(detail.clone()),
            _ => None,
        });
        let notice = notice.expect("le message d'échec de reprise doit exister");
        assert!(
            notice.starts_with(CLAUDE_RESUME_FAILED_PREFIX),
            "échec non nommé: {notice}"
        );
        assert!(
            notice.contains("conversation introuvable"),
            "cause absente: {notice}"
        );
        assert!(
            notice.contains("00000000-0000-0000-0000-000000000000"),
            "identifiant absent: {notice}"
        );
        let fresh = crate::claude_provider_session::ProviderSessionStore::new(&root, "agent-x");
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if fresh.load().as_deref() == Some("fresh-session-999") {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(fresh.load().as_deref(), Some("fresh-session-999"));
        assert_ne!(
            fresh.load().as_deref(),
            Some("00000000-0000-0000-0000-000000000000")
        );
        transport.stop();
        let _ = fs::remove_dir_all(root);
    }

    /// P4 — session persistée : le spawn rend la main avant le plafond daemon.
    /// Mutant : remettre read_line dans spawn_claude_child → oracle meurt seul.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_P4_session_persistee_ne_bloque_pas_le_lancement_sous_delai_daemon() {
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-resume-timeout-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let fake = root.join("fake-claude");
        fs::write(
            &fake,
            r#"#!/bin/sh
resume=0
for arg in "$@"; do [ "$arg" = "--resume" ] && resume=1; done
if [ "$resume" = 1 ]; then sleep 15; fi
printf '%s\n' '{"type":"system","subtype":"init","session_id":"slow-or-fast","model":"claude-opus-5"}'
exit 0
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&fake).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&fake, permissions).unwrap();
        }
        let options_neuf = ClaudeStreamJsonOptions {
            command: fake.to_string_lossy().into_owned(),
            args: Vec::new(),
            provider_kind: "claude".to_string(),
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            session_store_root: Some(root.clone()),
            agent_name: Some("agent-neuf".to_string()),
        };
        let t0 = Instant::now();
        let t_neuf = ClaudeStreamJsonTransport::spawn(options_neuf).expect("sans session");
        assert!(t0.elapsed() < DAEMON_SPAWN_TIMEOUT);
        t_neuf.stop();

        let store = crate::claude_provider_session::ProviderSessionStore::new(&root, "agent-lent");
        store.store("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
        let options_lent = ClaudeStreamJsonOptions {
            command: fake.to_string_lossy().into_owned(),
            args: Vec::new(),
            provider_kind: "claude".to_string(),
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            session_store_root: Some(root.clone()),
            agent_name: Some("agent-lent".to_string()),
        };
        let t1 = Instant::now();
        let t_lent = ClaudeStreamJsonTransport::spawn(options_lent).expect("avec session");
        assert!(
            t1.elapsed() < DAEMON_SPAWN_TIMEOUT,
            "spawn bloqué: {:?}",
            t1.elapsed()
        );
        assert!(
            t1.elapsed() < Duration::from_secs(2),
            "spawn trop lent: {:?}",
            t1.elapsed()
        );
        t_lent.stop();
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn consigne_privee_preserve_le_corps_visible_du_message() {
        let body = "Demande utilisateur visible.";
        let prompt = private_prompt(Some("Privilégie les sources attestées."), body);

        assert!(prompt.contains("Privilégie les sources attestées."));
        assert!(prompt.ends_with(body));
        assert_eq!(body, "Demande utilisateur visible.");
        assert_eq!(private_prompt(None, body), body);
    }
}
