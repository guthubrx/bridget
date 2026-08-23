//! Client de vue attach : connexion persistante, reprise et rendu sûr du journal.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    AttachRefusal, AttachWindow, ConnectionRole, DaemonToWrapper,
    MAX_ATTACH_SERIALIZED_FRAME_BYTES, WrapperToDaemon, decode, encode,
};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::mem::MaybeUninit;
use std::net::Shutdown;
use std::os::fd::RawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const MAX_REASSEMBLY_BYTES: usize = 4 * 1024 * 1024;
const MAX_RENDERED_EVENT_CHARS: usize = 16 * 1024;
const MAX_RENDERED_LABEL_CHARS: usize = 160;
const MAX_CONSECUTIVE_COMBINING_MARKS: usize = 8;
const RECONNECT_DELAY: Duration = Duration::from_millis(250);
const RETIRED_SUBSCRIPTIONS_LIMIT: usize = 64;
const SEND_ISSUE_TIMEOUT: Duration = Duration::from_secs(60);
const INPUT_POLL_TIMEOUT_MILLIS: i32 = 100;
const RENDER_COMMAND_CAPACITY: usize = 64;
const MAX_TURN_BLOCK_BYTES: usize = 64 * 1024;
const MAX_TURN_BLOCK_LINES: usize = 400;

/// Garde minimale du terminal d'entrée. Le mode désactive le traitement des
/// signaux afin que Ctrl-C arrive comme l'octet `0x03` à la boucle de saisie.
/// Ainsi la boucle peut restaurer le terminal avant de quitter.
#[allow(dead_code)] // La boucle de saisie T806b en devient le propriétaire.
struct RawTerminal {
    fd: RawFd,
    original: libc::termios,
    restored: bool,
}

#[allow(dead_code)] // API utilisée par la boucle T806b et les pseudo-TTY de T806a.
impl RawTerminal {
    fn enable_for_fd(fd: RawFd) -> Result<Option<Self>, String> {
        let is_tty = unsafe { libc::isatty(fd) };
        if is_tty == 0 {
            return Ok(None);
        }
        if is_tty < 0 {
            return Err(format!(
                "détection du terminal impossible: {}",
                std::io::Error::last_os_error()
            ));
        }

        let mut original = MaybeUninit::<libc::termios>::uninit();
        if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
            return Err(format!(
                "lecture termios impossible: {}",
                std::io::Error::last_os_error()
            ));
        }
        let original = unsafe { original.assume_init() };
        let mut raw = original;
        let disabled = (libc::ICANON | libc::ECHO | libc::ISIG) as libc::tcflag_t;
        raw.c_lflag &= !disabled;
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
            return Err(format!(
                "activation du mode raw impossible: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(Some(Self {
            fd,
            original,
            restored: false,
        }))
    }

    fn restore(&mut self) -> Result<(), String> {
        if self.restored {
            return Ok(());
        }
        if unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.original) } != 0 {
            return Err(format!(
                "restauration termios impossible: {}",
                std::io::Error::last_os_error()
            ));
        }
        self.restored = true;
        Ok(())
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[allow(dead_code)] // Le mode dégradé sera appelé par la boucle T806b.
fn with_raw_terminal<T>(
    fd: RawFd,
    operation: impl FnOnce(bool) -> Result<T, String>,
) -> Result<T, String> {
    let mut terminal = RawTerminal::enable_for_fd(fd)?;
    let result = operation(terminal.is_some());
    if let Some(raw) = terminal.as_mut() {
        raw.restore()?;
    }
    result
}

/// Événements neutres produits par la couche de transport puis projetés par le
/// renderer sécurisé de ce module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachEvent {
    Subscribed {
        subscription_id: String,
    },
    Journal {
        seq: u64,
        bytes: Vec<u8>,
        live: bool,
    },
    SnapshotCaughtUp {
        through_seq: Option<u64>,
    },
    Gap {
        from_seq: u64,
        to_seq: u64,
        reason: Option<String>,
    },
    JournalReadError {
        line: u64,
        offset: u64,
    },
    End {
        reason: String,
    },
    SendAcknowledged {
        message_id: String,
    },
    SendRejected {
        message_id: String,
        delayed: bool,
        reason: String,
    },
}

#[derive(Debug, Default)]
struct DispatchOutcome {
    events: Vec<AttachEvent>,
    resubscribe: Option<AttachWindow>,
    reconnect: bool,
    rejected: Option<AttachRefusal>,
}

#[derive(Debug)]
struct Reassembly {
    seq: u64,
    next_offset: u64,
    bytes: Vec<u8>,
    dropping: bool,
}

impl Reassembly {
    fn new(seq: u64, dropping: bool) -> Self {
        Self {
            seq,
            next_offset: 0,
            bytes: Vec::new(),
            dropping,
        }
    }
}

/// État strictement local d'une invocation attach.
#[derive(Debug)]
struct AttachClientState {
    initial_window: AttachWindow,
    subscription_id: Option<String>,
    retired_subscriptions: VecDeque<String>,
    waiting_for_subscription: bool,
    last_seq: Option<u64>,
    caught_up: bool,
    pending_send: HashMap<String, String>,
    pending_send_expirations: HashMap<String, Instant>,
    reassembly: Option<Reassembly>,
    #[cfg(test)]
    event_observer: Option<mpsc::Sender<()>>,
}

impl AttachClientState {
    fn new(initial_window: AttachWindow) -> Self {
        Self {
            initial_window,
            subscription_id: None,
            retired_subscriptions: VecDeque::new(),
            waiting_for_subscription: false,
            last_seq: None,
            caught_up: false,
            pending_send: HashMap::new(),
            pending_send_expirations: HashMap::new(),
            reassembly: None,
            #[cfg(test)]
            event_observer: None,
        }
    }

    fn resume_window(&self) -> AttachWindow {
        self.last_seq
            .map(|seq| AttachWindow::Seq(seq.saturating_add(1)))
            .unwrap_or_else(|| self.initial_window.clone())
    }

    fn subscription_requested(&mut self) -> AttachWindow {
        self.subscription_id = None;
        self.waiting_for_subscription = true;
        self.caught_up = false;
        self.reassembly = None;
        self.resume_window()
    }

    fn connection_closed(&mut self) {
        if let Some(subscription_id) = self.subscription_id.take() {
            self.retire(subscription_id);
        }
        self.waiting_for_subscription = false;
        self.caught_up = false;
        self.reassembly = None;
        self.pending_send.clear();
        self.pending_send_expirations.clear();
    }

    fn is_current(&self, subscription_id: &str) -> bool {
        self.subscription_id.as_deref() == Some(subscription_id)
    }

    fn retire(&mut self, subscription_id: String) {
        if self.retired_subscriptions.back() != Some(&subscription_id) {
            self.retired_subscriptions.push_back(subscription_id);
        }
        while self.retired_subscriptions.len() > RETIRED_SUBSCRIPTIONS_LIMIT {
            self.retired_subscriptions.pop_front();
        }
    }

    fn dispatch(&mut self, message: DaemonToWrapper) -> Result<DispatchOutcome, String> {
        let mut outcome = DispatchOutcome::default();
        match message {
            DaemonToWrapper::Subscribed { subscription_id } => {
                if self.waiting_for_subscription
                    && !self.retired_subscriptions.contains(&subscription_id)
                {
                    self.subscription_id = Some(subscription_id.clone());
                    self.waiting_for_subscription = false;
                    outcome
                        .events
                        .push(AttachEvent::Subscribed { subscription_id });
                }
            }
            DaemonToWrapper::JournalFragment {
                subscription_id,
                seq,
                offset,
                final_fragment,
                bytes,
            } if self.is_current(&subscription_id) => {
                outcome
                    .events
                    .extend(self.accept_fragment(seq, offset, final_fragment, bytes));
            }
            DaemonToWrapper::SnapshotCaughtUp {
                subscription_id,
                through_seq,
            } if self.is_current(&subscription_id) => {
                self.caught_up = true;
                outcome
                    .events
                    .push(AttachEvent::SnapshotCaughtUp { through_seq });
            }
            DaemonToWrapper::Gap {
                subscription_id,
                from_seq,
                to_seq,
                reason,
            } if self.is_current(&subscription_id) => {
                if self
                    .reassembly
                    .as_ref()
                    .is_some_and(|partial| (from_seq..=to_seq).contains(&partial.seq))
                {
                    self.reassembly = None;
                }
                outcome.events.push(AttachEvent::Gap {
                    from_seq,
                    to_seq,
                    reason,
                });
            }
            DaemonToWrapper::JournalReadError {
                subscription_id,
                line,
                offset,
                ..
            } if self.is_current(&subscription_id) => {
                outcome
                    .events
                    .push(AttachEvent::JournalReadError { line, offset });
            }
            DaemonToWrapper::End {
                subscription_id,
                reason,
            } if self.is_current(&subscription_id) => {
                self.retire(subscription_id);
                self.subscription_id = None;
                self.waiting_for_subscription = true;
                self.caught_up = false;
                self.reassembly = None;
                outcome.events.push(AttachEvent::End { reason });
                outcome.resubscribe = Some(self.resume_window());
            }
            DaemonToWrapper::AttachRejected {
                subscription_id,
                reason,
            } => {
                if !subscription_id
                    .as_ref()
                    .is_some_and(|id| self.retired_subscriptions.contains(id))
                {
                    outcome.rejected = Some(reason);
                }
            }
            DaemonToWrapper::Ack { id } if self.pending_send.contains_key(&id) => {
                outcome
                    .events
                    .push(AttachEvent::SendAcknowledged { message_id: id });
            }
            DaemonToWrapper::Nack { id, reason } if self.remove_pending_send(&id) => {
                outcome.events.push(AttachEvent::SendRejected {
                    message_id: id,
                    delayed: false,
                    reason,
                });
            }
            DaemonToWrapper::DeliveryRejected { id, reason } if self.remove_pending_send(&id) => {
                outcome.events.push(AttachEvent::SendRejected {
                    message_id: id,
                    delayed: true,
                    reason,
                });
            }
            DaemonToWrapper::Disconnect => outcome.reconnect = true,
            message if message.allowed_for_attach() => {}
            _ => return Err("message interdit sur une connexion attach".to_string()),
        }
        Ok(outcome)
    }

    fn track_send(&mut self, message_id: String, body: String, now: Instant) {
        self.pending_send.insert(message_id.clone(), body);
        self.pending_send_expirations
            .insert(message_id, now + SEND_ISSUE_TIMEOUT);
    }

    fn remove_pending_send(&mut self, message_id: &str) -> bool {
        self.pending_send_expirations.remove(message_id);
        self.pending_send.remove(message_id).is_some()
    }

    fn expire_pending_sends(&mut self, now: Instant) -> Vec<AttachEvent> {
        let expired = self
            .pending_send_expirations
            .iter()
            .filter(|(_, expires_at)| **expires_at <= now)
            .map(|(message_id, _)| message_id.clone())
            .collect::<Vec<_>>();
        expired
            .into_iter()
            .filter_map(|message_id| {
                self.remove_pending_send(&message_id)
                    .then_some(AttachEvent::SendRejected {
                        message_id,
                        delayed: true,
                        reason: "délai d'issue dépassé".to_string(),
                    })
            })
            .collect()
    }

    /// Assemble une charge en O(nombre d'octets), avec une rétention bornée à
    /// `MAX_REASSEMBLY_BYTES`, puis consomme sans stocker jusqu'au fragment final.
    fn accept_fragment(
        &mut self,
        seq: u64,
        offset: u64,
        final_fragment: bool,
        bytes: Vec<u8>,
    ) -> Vec<AttachEvent> {
        let mut events = Vec::new();
        if self
            .reassembly
            .as_ref()
            .is_some_and(|partial| partial.seq != seq)
        {
            let abandoned = self.reassembly.take().expect("assemblage présent");
            if !abandoned.dropping {
                events.push(AttachEvent::Gap {
                    from_seq: abandoned.seq,
                    to_seq: abandoned.seq,
                    reason: Some("fragment_missing".to_string()),
                });
            }
        }

        if self.reassembly.is_none() {
            let dropping = offset != 0;
            self.reassembly = Some(Reassembly::new(seq, dropping));
            if dropping {
                events.push(AttachEvent::Gap {
                    from_seq: seq,
                    to_seq: seq,
                    reason: Some("fragment_missing".to_string()),
                });
            }
        }

        let partial = self.reassembly.as_mut().expect("assemblage initialisé");
        if !partial.dropping && offset != partial.next_offset {
            partial.bytes.clear();
            partial.dropping = true;
            events.push(AttachEvent::Gap {
                from_seq: seq,
                to_seq: seq,
                reason: Some("fragment_missing".to_string()),
            });
        }

        if !partial.dropping {
            let exceeds_limit = partial
                .bytes
                .len()
                .checked_add(bytes.len())
                .is_none_or(|size| size > MAX_REASSEMBLY_BYTES);
            if exceeds_limit {
                partial.bytes.clear();
                partial.dropping = true;
                events.push(AttachEvent::Gap {
                    from_seq: seq,
                    to_seq: seq,
                    reason: Some("event_too_large".to_string()),
                });
            } else {
                partial.bytes.extend_from_slice(&bytes);
            }
        }
        partial.next_offset = offset.saturating_add(bytes.len() as u64);

        if final_fragment {
            let completed = self.reassembly.take().expect("assemblage présent");
            if !completed.dropping && self.last_seq.is_none_or(|last| seq > last) {
                self.last_seq = Some(seq);
                events.push(AttachEvent::Journal {
                    seq,
                    bytes: completed.bytes,
                    live: self.caught_up,
                });
            }
        }
        events
    }
}

struct AttachConnection {
    reader: BufReader<UnixStream>,
    writer: Arc<Mutex<BufWriter<UnixStream>>>,
}

impl AttachConnection {
    fn connect(socket_path: &Path) -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path)
            .map_err(|error| format!("connexion au daemon impossible: {error}"))?;
        let reader_stream = stream
            .try_clone()
            .map_err(|error| format!("duplication de la socket impossible: {error}"))?;
        let connection = Self {
            reader: BufReader::new(reader_stream),
            writer: Arc::new(Mutex::new(BufWriter::new(stream))),
        };
        connection.send(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        })?;
        Ok(connection)
    }

    fn send(&self, message: &WrapperToDaemon) -> Result<(), String> {
        write_socket_message(&self.writer, message)
    }

    fn read(&mut self) -> Result<Option<DaemonToWrapper>, String> {
        read_daemon_message(&mut self.reader)
    }

    fn accept_role(&mut self) -> Result<(), String> {
        match self.read()? {
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach,
            }) => Ok(()),
            Some(DaemonToWrapper::AttachRejected { reason, .. }) => {
                Err(format!("rôle attach refusé: {reason:?}"))
            }
            Some(_) => Err("réponse inattendue pendant la négociation attach".to_string()),
            None => Err("daemon déconnecté pendant la négociation attach".to_string()),
        }
    }

    fn subscribe(&self, agent: &str, window: AttachWindow) -> Result<(), String> {
        self.send(&WrapperToDaemon::Subscribe {
            agent: agent.to_string(),
            window,
        })
    }
}

fn write_socket_message(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    message: &WrapperToDaemon,
) -> Result<(), String> {
    let line = encode(message).map_err(|error| format!("encodage attach impossible: {error}"))?;
    let mut writer = writer
        .lock()
        .map_err(|_| "writer attach empoisonné".to_string())?;
    writer
        .write_all(line.as_bytes())
        .and_then(|()| writer.write_all(b"\n"))
        .and_then(|()| writer.flush())
        .map_err(|error| format!("écriture attach impossible: {error}"))
}

fn read_bounded_frame(reader: &mut BufReader<UnixStream>) -> Result<Option<Vec<u8>>, String> {
    let mut frame = Vec::new();
    loop {
        let (consumed, complete, overflow) = {
            let available = reader
                .fill_buf()
                .map_err(|error| format!("lecture attach impossible: {error}"))?;
            if available.is_empty() {
                if frame.is_empty() {
                    return Ok(None);
                }
                return Err("frame attach tronquée avant newline".to_string());
            }
            let newline = available.iter().position(|byte| *byte == b'\n');
            let payload_len = newline.unwrap_or(available.len());
            let overflow = frame
                .len()
                .checked_add(payload_len)
                .is_none_or(|size| size > MAX_ATTACH_SERIALIZED_FRAME_BYTES);
            if !overflow {
                frame.extend_from_slice(&available[..payload_len]);
            }
            (
                newline.map_or(available.len(), |index| index + 1),
                newline.is_some(),
                overflow,
            )
        };
        reader.consume(consumed);
        if overflow {
            if !complete {
                discard_until_newline(reader)?;
            }
            return Err(format!(
                "frame attach trop volumineuse (max {MAX_ATTACH_SERIALIZED_FRAME_BYTES} octets)"
            ));
        }
        if complete {
            return Ok(Some(frame));
        }
    }
}

fn discard_until_newline(reader: &mut BufReader<UnixStream>) -> Result<(), String> {
    loop {
        let (consumed, complete) = {
            let available = reader
                .fill_buf()
                .map_err(|error| format!("lecture attach impossible: {error}"))?;
            if available.is_empty() {
                return Ok(());
            }
            match available.iter().position(|byte| *byte == b'\n') {
                Some(index) => (index + 1, true),
                None => (available.len(), false),
            }
        };
        reader.consume(consumed);
        if complete {
            return Ok(());
        }
    }
}

/// Lance une vue attach persistante. La mémoire croît au plus comme un
/// événement réassemblé (4 Mio) plus la petite table des envois de l'invocation.
pub fn run(agent: &str, initial_window: AttachWindow, socket_path: &Path) -> Result<(), String> {
    with_raw_terminal(libc::STDIN_FILENO, |raw_terminal| {
        run_with_input(
            agent,
            initial_window,
            socket_path,
            raw_terminal,
            is_terminal(libc::STDOUT_FILENO),
        )
    })
}

fn is_terminal(fd: RawFd) -> bool {
    unsafe { libc::isatty(fd) == 1 }
}

fn run_with_input(
    agent: &str,
    initial_window: AttachWindow,
    socket_path: &Path,
    raw_terminal: bool,
    tty_output: bool,
) -> Result<(), String> {
    let mut state = AttachClientState::new(initial_window);
    let mut connected_once = false;

    loop {
        let attempt = AttachConnection::connect(socket_path).and_then(|mut connection| {
            connection.accept_role()?;
            let window = state.subscription_requested();
            connection.subscribe(agent, window)?;
            Ok(connection)
        });
        let connection = match attempt {
            Ok(connection) => connection,
            Err(error) if !connected_once => return Err(error),
            Err(_) => {
                thread::sleep(RECONNECT_DELAY);
                continue;
            }
        };
        connected_once = true;
        if !drive_interactive(
            connection,
            &mut state,
            agent,
            socket_path,
            libc::STDIN_FILENO,
            raw_terminal,
            tty_output,
        )? {
            return Ok(());
        }

        state.connection_closed();
        thread::sleep(RECONNECT_DELAY);
    }
}

#[derive(Debug)]
enum ReaderStatus {
    Closed,
    Failed(String),
}

#[derive(Debug, Default)]
struct InputBuffer {
    bytes: Vec<u8>,
}

impl InputBuffer {
    fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn erase_last(&mut self) -> bool {
        let Some(last_start) = std::str::from_utf8(&self.bytes)
            .ok()
            .and_then(|text| text.char_indices().next_back().map(|(index, _)| index))
        else {
            return self.bytes.pop().is_some();
        };
        self.bytes.truncate(last_start);
        true
    }

    fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }

    fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn display(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TurnKey {
    session_id: String,
    message_id: String,
}

#[derive(Debug)]
struct TurnBlock {
    key: TurnKey,
    header: Vec<String>,
    response: String,
    details: Vec<String>,
    stored_bytes: usize,
    stored_lines: usize,
    omitted_lines: usize,
}

impl TurnBlock {
    fn new(key: TurnKey, header: String) -> Self {
        let header = header.lines().map(str::to_owned).collect::<Vec<_>>();
        Self {
            key,
            stored_bytes: header.iter().map(String::len).sum(),
            stored_lines: header.len(),
            header,
            response: String::new(),
            details: Vec::new(),
            omitted_lines: 0,
        }
    }

    fn append_response(&mut self, content: &str) {
        self.append_bounded(content, true);
    }

    fn append_detail(&mut self, detail: &str) {
        self.append_bounded(detail, false);
    }

    fn append_bounded(&mut self, value: &str, response: bool) {
        let additional_lines = value.bytes().filter(|byte| *byte == b'\n').count()
            + usize::from(response && self.response.is_empty() && !value.is_empty())
            + usize::from(!response && !value.is_empty());
        if self.stored_bytes.saturating_add(value.len()) <= MAX_TURN_BLOCK_BYTES
            && self.stored_lines.saturating_add(additional_lines) <= MAX_TURN_BLOCK_LINES
        {
            if response {
                self.response.push_str(value);
            } else {
                self.details.extend(value.lines().map(str::to_owned));
            }
            self.stored_bytes += value.len();
            self.stored_lines += additional_lines;
            return;
        }
        self.omitted_lines = self
            .omitted_lines
            .saturating_add(value.lines().count().max(1));
    }

    fn lines(&self, agent: &str) -> Vec<String> {
        let mut lines = self.header.clone();
        if !self.response.is_empty() {
            lines.extend(
                render_prefixed(&format!("{agent} →"), &self.response)
                    .lines()
                    .map(str::to_owned),
            );
        }
        lines.extend(self.details.iter().cloned());
        if self.omitted_lines > 0 {
            lines.push(format!("… tronqué, {} ligne(s)", self.omitted_lines));
        }
        lines
    }
}

struct JournalRenderRecord {
    key: Option<TurnKey>,
    event: String,
    text: Option<String>,
    terminal: bool,
    rendered: String,
}

fn journal_render_record(bytes: &[u8], agent: &str) -> Option<JournalRenderRecord> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).ok()?;
    if value.get("v").and_then(serde_json::Value::as_u64) != Some(1) {
        return None;
    }
    let event = value.get("event")?.as_str()?.to_string();
    let payload = value.get("payload").unwrap_or(&serde_json::Value::Null);
    let key = value
        .get("session_id")
        .and_then(serde_json::Value::as_str)
        .zip(value.get("message_id").and_then(serde_json::Value::as_str))
        .map(|(session_id, message_id)| TurnKey {
            session_id: session_id.to_string(),
            message_id: message_id.to_string(),
        });
    let text = (event == "update"
        && payload.get("kind").and_then(serde_json::Value::as_str) == Some("text"))
    .then(|| {
        payload
            .get("content")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    Some(JournalRenderRecord {
        key,
        event,
        text,
        terminal: payload
            .get("terminal")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        rendered: render_journal_event(bytes, agent),
    })
}

enum RendererCommand {
    Event(AttachEvent),
    InputChanged(Vec<u8>),
}

enum RendererControl {
    Stop,
}

#[derive(Clone)]
struct RendererSender {
    commands: mpsc::SyncSender<RendererCommand>,
    input_dirty: Arc<AtomicBool>,
    tty_output: bool,
}

impl RendererSender {
    fn event(&self, event: AttachEvent) {
        let _ = self.commands.send(RendererCommand::Event(event));
    }

    fn input_changed(&self, legacy_bytes: &[u8]) {
        if self.tty_output {
            self.input_dirty.store(true, Ordering::Release);
            let _ = self
                .commands
                .try_send(RendererCommand::InputChanged(Vec::new()));
        } else {
            let _ = self
                .commands
                .send(RendererCommand::InputChanged(legacy_bytes.to_vec()));
        }
    }
}

struct RendererThread {
    sender: RendererSender,
    control: mpsc::Sender<RendererControl>,
    handle: thread::JoinHandle<()>,
}

impl RendererThread {
    fn spawn(
        input: Arc<Mutex<InputBuffer>>,
        agent: String,
        raw_terminal: bool,
        tty_output: bool,
    ) -> Self {
        let (commands, command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        let (control, control_rx) = mpsc::channel();
        let input_dirty = Arc::new(AtomicBool::new(false));
        let sender = RendererSender {
            commands,
            input_dirty: input_dirty.clone(),
            tty_output,
        };
        let handle = thread::spawn(move || {
            let stdout = std::io::stdout();
            let mut output = stdout.lock();
            renderer_loop(
                &mut output,
                BlockRenderer::new(agent, raw_terminal, tty_output),
                input,
                input_dirty,
                command_rx,
                control_rx,
            );
        });
        Self {
            sender,
            control,
            handle,
        }
    }

    fn sender(&self) -> RendererSender {
        self.sender.clone()
    }

    fn stop(self) {
        let _ = self.control.send(RendererControl::Stop);
        let _ = self.handle.join();
    }
}

fn renderer_loop(
    output: &mut impl Write,
    mut renderer: BlockRenderer,
    input: Arc<Mutex<InputBuffer>>,
    input_dirty: Arc<AtomicBool>,
    commands: mpsc::Receiver<RendererCommand>,
    control: mpsc::Receiver<RendererControl>,
) {
    loop {
        if matches!(control.try_recv(), Ok(RendererControl::Stop)) {
            while let Ok(command) = commands.try_recv() {
                renderer.apply(command, &input, output);
            }
            break;
        }
        match commands.recv_timeout(Duration::from_millis(10)) {
            Ok(command) => renderer.apply(command, &input, output),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if input_dirty.swap(false, Ordering::AcqRel) && renderer.tty_output {
            renderer.redraw(&input_snapshot(&input), output);
        }
    }
    renderer.finish(&input_snapshot(&input), output);
}

fn input_snapshot(input: &Arc<Mutex<InputBuffer>>) -> String {
    input
        .lock()
        .map(|input| input.display())
        .unwrap_or_default()
}

struct BlockRenderer {
    agent: String,
    raw_terminal: bool,
    tty_output: bool,
    current: Option<TurnBlock>,
    rendered_rows: usize,
}

impl BlockRenderer {
    fn new(agent: String, raw_terminal: bool, tty_output: bool) -> Self {
        Self {
            agent,
            raw_terminal,
            tty_output,
            current: None,
            rendered_rows: 0,
        }
    }

    fn apply(
        &mut self,
        command: RendererCommand,
        input: &Arc<Mutex<InputBuffer>>,
        output: &mut impl Write,
    ) {
        let input = input_snapshot(input);
        match command {
            RendererCommand::Event(event) => self.render_event(&event, &input, output),
            RendererCommand::InputChanged(_bytes) if self.tty_output => self.redraw(&input, output),
            RendererCommand::InputChanged(bytes) => {
                let _ = output.write_all(&bytes);
                let _ = output.flush();
            }
        }
    }

    fn render_event(&mut self, event: &AttachEvent, input: &str, output: &mut impl Write) {
        if !self.tty_output {
            if self.raw_terminal {
                let _ = output.write_all(b"\r\n");
            }
            let _ = writeln!(output, "{}", render_attach_event(event, &self.agent));
            if self.raw_terminal {
                let _ = write!(output, "> {input}");
            }
            let _ = output.flush();
            return;
        }
        match event {
            AttachEvent::Journal { bytes, live, .. } => {
                self.render_journal(bytes, *live, input, output)
            }
            AttachEvent::SnapshotCaughtUp { .. } if self.current.is_some() => {
                let marker = render_attach_event(event, &self.agent);
                if let Some(block) = self.current.as_mut() {
                    block.append_detail(&marker);
                }
                self.redraw(input, output);
            }
            AttachEvent::Gap { .. }
            | AttachEvent::JournalReadError { .. }
            | AttachEvent::End { .. } => {
                self.flush_incomplete(input, output);
                self.emit_standalone(&render_attach_event(event, &self.agent), input, output);
            }
            _ => self.emit_standalone(&render_attach_event(event, &self.agent), input, output),
        }
    }

    fn render_journal(&mut self, bytes: &[u8], live: bool, input: &str, output: &mut impl Write) {
        let Some(record) = journal_render_record(bytes, &self.agent) else {
            self.flush_incomplete(input, output);
            self.emit_standalone(&render_journal_event(bytes, &self.agent), input, output);
            return;
        };
        let Some(key) = record.key else {
            self.flush_incomplete(input, output);
            self.emit_standalone(&record.rendered, input, output);
            return;
        };
        let starts_block = self.current.as_ref().map(|block| &block.key) != Some(&key);
        if starts_block {
            self.flush_incomplete(input, output);
            let header = if record.event == "turn_start" {
                record.rendered.clone()
            } else {
                format!("??:?? {} →", self.agent)
            };
            self.current = Some(TurnBlock::new(key.clone(), header));
        }
        if record.event == "turn_start" {
            if live {
                self.redraw(input, output);
            }
            return;
        }
        if let Some(block) = self.current.as_mut() {
            if let Some(text) = record.text {
                block.append_response(&text);
            } else {
                block.append_detail(&record.rendered);
            }
        }
        if record.event == "turn_end" || record.terminal {
            self.flush_complete(input, output);
        } else if live {
            self.redraw(input, output);
        }
    }

    fn flush_incomplete(&mut self, input: &str, output: &mut impl Write) {
        if let Some(block) = self.current.as_mut() {
            block.append_detail("[tour incomplet]");
        }
        self.flush_complete(input, output);
    }

    fn flush_complete(&mut self, input: &str, output: &mut impl Write) {
        let Some(block) = self.current.take() else {
            return;
        };
        self.clear(output);
        write_terminal_lines(output, &block.lines(&self.agent));
        if self.raw_terminal {
            let _ = write!(output, "> {input}");
            self.rendered_rows = 1;
        }
        let _ = output.flush();
    }

    fn emit_standalone(&mut self, rendered: &str, input: &str, output: &mut impl Write) {
        self.clear(output);
        write_terminal_lines(
            output,
            &rendered.lines().map(str::to_owned).collect::<Vec<_>>(),
        );
        self.draw_active(input, output);
        let _ = output.flush();
    }

    fn redraw(&mut self, input: &str, output: &mut impl Write) {
        self.clear(output);
        self.draw_active(input, output);
        let _ = output.flush();
    }

    fn draw_active(&mut self, input: &str, output: &mut impl Write) {
        let mut rows = 0;
        if let Some(block) = self.current.as_ref() {
            let lines = block.lines(&self.agent);
            rows += lines.len();
            write_visual_lines(output, &lines);
            if self.raw_terminal {
                let _ = output.write_all(b"\r\n");
            }
        }
        if self.raw_terminal {
            let _ = write!(output, "> {input}");
            rows += 1;
        }
        self.rendered_rows = rows;
    }

    fn clear(&mut self, output: &mut impl Write) {
        for index in 0..self.rendered_rows {
            let _ = output.write_all(b"\r\x1b[2K");
            if index + 1 < self.rendered_rows {
                let _ = output.write_all(b"\x1b[1A");
            }
        }
        self.rendered_rows = 0;
    }

    fn finish(&mut self, input: &str, output: &mut impl Write) {
        self.flush_incomplete(input, output);
        if self.rendered_rows > 0 {
            let _ = output.write_all(b"\r\n");
            self.rendered_rows = 0;
        }
        let _ = output.flush();
    }
}

fn write_visual_lines(output: &mut impl Write, lines: &[String]) {
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            let _ = output.write_all(b"\r\n");
        }
        let _ = output.write_all(line.as_bytes());
    }
}

fn write_terminal_lines(output: &mut impl Write, lines: &[String]) {
    write_visual_lines(output, lines);
    if !lines.is_empty() {
        let _ = output.write_all(b"\r\n");
    }
}

/// Maintient la saisie locale pendant qu'un seul lecteur socket traite le flux
/// attach. Le writer reste partagé et sérialisé avec les resouscriptions du
/// lecteur ; ainsi `Send` et les issues différées passent par la même connexion.
fn drive_interactive(
    connection: AttachConnection,
    client_state: &mut AttachClientState,
    agent: &str,
    socket_path: &Path,
    input_fd: RawFd,
    raw_terminal: bool,
    tty_output: bool,
) -> Result<bool, String> {
    let AttachConnection { reader, writer } = connection;
    let shared_state = Arc::new(Mutex::new(std::mem::replace(
        client_state,
        AttachClientState::new(AttachWindow::Today),
    )));
    let input = Arc::new(Mutex::new(InputBuffer::default()));
    let renderer =
        RendererThread::spawn(input.clone(), agent.to_string(), raw_terminal, tty_output);
    let renderer_sender = renderer.sender();
    let (status_tx, status_rx) = mpsc::channel();
    let reader_handle = spawn_attach_reader(
        reader,
        writer.clone(),
        shared_state.clone(),
        renderer_sender.clone(),
        agent.to_string(),
        socket_path.to_path_buf(),
        status_tx,
    );

    let mut reconnect = true;
    let mut reader_failure = None;
    loop {
        match status_rx.try_recv() {
            Ok(ReaderStatus::Closed) => break,
            Ok(ReaderStatus::Failed(error)) => {
                reconnect = false;
                reader_failure = Some(error);
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => break,
            Err(mpsc::TryRecvError::Empty) => {}
        }

        let mut pollfd = libc::pollfd {
            fd: input_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let poll_result = unsafe { libc::poll(&mut pollfd, 1, INPUT_POLL_TIMEOUT_MILLIS) };
        if poll_result < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                reconnect = false;
                break;
            }
            continue;
        }
        if poll_result == 0 {
            render_expired_sends(&shared_state, &renderer_sender);
            continue;
        }
        if pollfd.revents & libc::POLLIN != 0 {
            // Un pseudo-terminal peut signaler POLLIN|POLLHUP pour ses
            // derniers octets. Les consommer avant de conclure à EOF évite de
            // perdre le dernier Send d'une saisie terminée par fermeture.
            loop {
                let mut byte = 0_u8;
                let read = unsafe {
                    libc::read(input_fd, (&mut byte as *mut u8).cast::<libc::c_void>(), 1)
                };
                if read == 0 {
                    reconnect = false;
                    break;
                }
                if read < 0 {
                    let error = std::io::Error::last_os_error();
                    if error.kind() != std::io::ErrorKind::Interrupted {
                        reconnect = false;
                    }
                    break;
                }
                if !handle_input_byte(
                    byte,
                    &shared_state,
                    &input,
                    &renderer_sender,
                    &writer,
                    agent,
                )? {
                    reconnect = false;
                    break;
                }
                let mut more = libc::pollfd {
                    fd: input_fd,
                    events: libc::POLLIN,
                    revents: 0,
                };
                if unsafe { libc::poll(&mut more, 1, 0) } <= 0 || more.revents & libc::POLLIN == 0 {
                    break;
                }
            }
            if !reconnect {
                break;
            }
        }
        if pollfd.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
            // Hors TTY, stdin peut être fermé par le lanceur sans que la vue
            // elle-même soit terminée : le flux socket reste alors observable.
            // En raw mode, Ctrl-D est traité comme un octet avant ce palier.
            if raw_terminal {
                reconnect = false;
                break;
            }
            render_expired_sends(&shared_state, &renderer_sender);
            continue;
        }
        render_expired_sends(&shared_state, &renderer_sender);
    }

    close_attach_socket(&writer);
    let _ = reader_handle.join();
    renderer.stop();
    let mut recovered = Arc::try_unwrap(shared_state)
        .map_err(|_| "état attach encore partagé à la fermeture".to_string())?
        .into_inner()
        .map_err(|_| "état attach empoisonné".to_string())?;
    recovered.connection_closed();
    *client_state = recovered;
    match reader_failure {
        Some(error) => Err(error),
        None => Ok(reconnect),
    }
}

fn spawn_attach_reader(
    mut reader: BufReader<UnixStream>,
    writer: Arc<Mutex<BufWriter<UnixStream>>>,
    state: Arc<Mutex<AttachClientState>>,
    renderer: RendererSender,
    agent: String,
    socket_path: std::path::PathBuf,
    status_tx: mpsc::Sender<ReaderStatus>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let message = match read_daemon_message(&mut reader) {
                Ok(Some(message)) => message,
                Ok(None) => {
                    let _ = status_tx.send(ReaderStatus::Closed);
                    return;
                }
                Err(error) => {
                    let _ = status_tx.send(ReaderStatus::Failed(error));
                    return;
                }
            };
            #[cfg(test)]
            let journal_rendered = matches!(&message, DaemonToWrapper::JournalFragment { .. });
            let outcome = match state
                .lock()
                .map_err(|_| "état attach empoisonné".to_string())
                .and_then(|mut state| state.dispatch(message))
            {
                Ok(outcome) => outcome,
                Err(error) => {
                    let _ = status_tx.send(ReaderStatus::Failed(error));
                    return;
                }
            };
            for event in outcome.events {
                renderer.event(event);
            }
            #[cfg(test)]
            if journal_rendered
                && let Some(observer) = state
                    .lock()
                    .ok()
                    .and_then(|state| state.event_observer.clone())
            {
                let _ = observer.send(());
            }
            if let Some(reason) = outcome.rejected {
                let attachable_agents = if reason == AttachRefusal::AgentUnknown {
                    list_attachable_agents(&socket_path)
                } else {
                    Vec::new()
                };
                let _ = status_tx.send(ReaderStatus::Failed(attach_refusal_message(
                    &reason,
                    &agent,
                    &attachable_agents,
                )));
                return;
            }
            if let Some(window) = outcome.resubscribe
                && let Err(error) = write_socket_message(
                    &writer,
                    &WrapperToDaemon::Subscribe {
                        agent: agent.clone(),
                        window,
                    },
                )
            {
                let _ = status_tx.send(ReaderStatus::Failed(error));
                return;
            }
            if outcome.reconnect {
                let _ = status_tx.send(ReaderStatus::Closed);
                return;
            }
        }
    })
}

fn read_daemon_message(
    reader: &mut BufReader<UnixStream>,
) -> Result<Option<DaemonToWrapper>, String> {
    loop {
        let Some(frame) = read_bounded_frame(reader)? else {
            return Ok(None);
        };
        let line =
            std::str::from_utf8(&frame).map_err(|_| "réponse attach non UTF-8".to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        return decode(line)
            .map(Some)
            .map_err(|error| format!("réponse attach invalide: {error}"));
    }
}

fn handle_input_byte(
    byte: u8,
    state: &Arc<Mutex<AttachClientState>>,
    input: &Arc<Mutex<InputBuffer>>,
    renderer: &RendererSender,
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    agent: &str,
) -> Result<bool, String> {
    match byte {
        0x03 => return Ok(false),
        0x04 => {
            if input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?
                .is_empty()
            {
                return Ok(false);
            }
        }
        b'\r' | b'\n' => {
            let bytes = input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?
                .take();
            renderer.input_changed(b"\r\n");
            if bytes.is_empty() {
                return Ok(true);
            }
            let body = String::from_utf8(bytes)
                .map_err(|_| "saisie invalide : UTF-8 attendu".to_string())?;
            let message = BridgetMessage::new("humain", agent, body.clone());
            let message_id = message.id.clone();
            {
                let mut state = state
                    .lock()
                    .map_err(|_| "état attach empoisonné".to_string())?;
                state.track_send(message_id.clone(), body, Instant::now());
            }
            if let Err(error) = write_socket_message(writer, &WrapperToDaemon::Send(message)) {
                let _ = state
                    .lock()
                    .map(|mut state| state.remove_pending_send(&message_id));
                return Err(error);
            }
            return Ok(true);
        }
        0x08 | 0x7f => {
            let erased = input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?
                .erase_last();
            if erased {
                renderer.input_changed(b"\x08 \x08");
            }
        }
        byte if byte >= 0x20 => {
            input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?
                .push(byte);
            renderer.input_changed(&[byte]);
        }
        _ => {}
    }
    Ok(true)
}

fn render_expired_sends(state: &Arc<Mutex<AttachClientState>>, renderer: &RendererSender) {
    let events = state
        .lock()
        .map(|mut state| state.expire_pending_sends(Instant::now()))
        .unwrap_or_default();
    for event in events {
        renderer.event(event);
    }
}

fn close_attach_socket(writer: &Arc<Mutex<BufWriter<UnixStream>>>) {
    if let Ok(writer) = writer.lock() {
        let _ = writer.get_ref().shutdown(Shutdown::Both);
    }
}

#[cfg(test)]
fn drive_connection(
    connection: &mut AttachConnection,
    state: &mut AttachClientState,
    agent: &str,
    socket_path: &Path,
    on_event: &mut impl FnMut(&AttachEvent),
) -> Result<(), String> {
    while let Ok(Some(message)) = connection.read() {
        let outcome = state.dispatch(message)?;
        for event in outcome.events {
            on_event(&event);
        }
        if let Some(reason) = outcome.rejected {
            let attachable_agents = if reason == AttachRefusal::AgentUnknown {
                list_attachable_agents(socket_path)
            } else {
                Vec::new()
            };
            return Err(attach_refusal_message(&reason, agent, &attachable_agents));
        }
        if let Some(window) = outcome.resubscribe
            && connection.subscribe(agent, window).is_err()
        {
            return Ok(());
        }
        if outcome.reconnect {
            return Ok(());
        }
    }
    Ok(())
}

/// Rend un événement sans jamais laisser les données du journal produire des
/// contrôles de terminal. Les séquences ANSI éventuelles appartiendront au
/// renderer lui-même, jamais au contenu reçu.
fn render_attach_event(event: &AttachEvent, agent: &str) -> String {
    match event {
        AttachEvent::Subscribed { .. } => "attach: abonnement actif".to_string(),
        AttachEvent::Journal { bytes, .. } => render_journal_event(bytes, agent),
        AttachEvent::SnapshotCaughtUp {
            through_seq: Some(seq),
        } => {
            format!("attach: historique rattrapé jusqu’à {seq}")
        }
        AttachEvent::SnapshotCaughtUp { through_seq: None } => {
            "attach: en attente du premier événement".to_string()
        }
        AttachEvent::Gap {
            from_seq,
            to_seq,
            reason,
        } => {
            let count = to_seq.saturating_sub(*from_seq).saturating_add(1);
            let reason = reason
                .as_deref()
                .map(|value| format!(" — {}", sanitize_inline(value)))
                .unwrap_or_default();
            format!(
                "attach: +{count} événement(s) non affiché(s), rejouables (seq {from_seq}→{to_seq}){reason}"
            )
        }
        AttachEvent::JournalReadError { line, offset } => {
            format!("attach: ligne {line}, offset {offset} : illisible, seq inconnu")
        }
        AttachEvent::End { reason } => format!(
            "attach: abonnement terminé ({}) — resynchronisation",
            sanitize_inline(reason)
        ),
        AttachEvent::SendAcknowledged { message_id } => {
            format!("attach: envoi {} accepté", sanitize_inline(message_id))
        }
        AttachEvent::SendRejected {
            message_id,
            delayed,
            reason,
        } => {
            let phase = if *delayed { "différé" } else { "immédiat" };
            format!(
                "attach: envoi {} rejeté ({phase}) : {}",
                sanitize_inline(message_id),
                sanitize_inline(reason)
            )
        }
    }
}

fn render_journal_event(bytes: &[u8], agent: &str) -> String {
    let value = match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(value) => value,
        Err(_) => return "??:?? [erreur] événement journal illisible".to_string(),
    };
    if value.get("v").and_then(serde_json::Value::as_u64) != Some(1) {
        return "??:?? [erreur] version de journal non prise en charge".to_string();
    }

    let timestamp = short_timestamp(value.get("ts").and_then(serde_json::Value::as_str));
    let event = value
        .get("event")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("inconnu");
    let payload = value.get("payload").unwrap_or(&serde_json::Value::Null);
    let (label, content) = match event {
        "turn_start" => (
            format!(
                "{} →",
                payload
                    .get("from")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("humain")
            ),
            payload
                .get("body")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("message sans corps")
                .to_string(),
        ),
        "update" if payload.get("kind").and_then(serde_json::Value::as_str) == Some("text") => (
            format!("{agent} →"),
            payload
                .get("content")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
        ),
        "update"
            if payload.get("kind").and_then(serde_json::Value::as_str) == Some("tool_call") =>
        {
            let tool = payload
                .get("title")
                .or_else(|| payload.get("name"))
                .or_else(|| payload.get("tool_kind"))
                .or_else(|| payload.get("tool"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("inconnu");
            (
                format!("[outil] {tool}"),
                payload
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .filter(|summary| !summary.is_empty())
                    .unwrap_or("appel demandé")
                    .to_string(),
            )
        }
        "permission" => (
            format!(
                "[permission] {}",
                payload
                    .get("tool")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("outil inconnu")
            ),
            permission_summary(payload),
        ),
        "turn_end" => ("[fin]".to_string(), turn_end_summary(payload)),
        "error" => (
            "[erreur]".to_string(),
            payload
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("erreur sans motif")
                .to_string(),
        ),
        _ => (
            format!("[événement] {event}"),
            "payload v1 non pris en charge".to_string(),
        ),
    };

    render_prefixed(&format!("{timestamp} {label}"), &content)
}

fn permission_summary(payload: &serde_json::Value) -> String {
    match payload
        .pointer("/decision/outcome")
        .and_then(serde_json::Value::as_str)
    {
        Some("selected") => format!(
            "autorisation décidée : {}",
            payload
                .pointer("/decision/option_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("option inconnue")
        ),
        Some("cancelled") => "autorisation refusée".to_string(),
        Some(outcome) => format!("décision : {outcome}"),
        None => "décision absente".to_string(),
    }
}

fn turn_end_summary(payload: &serde_json::Value) -> String {
    let stop_reason = payload
        .get("stop_reason")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("inconnu");
    match payload.get("routed_to").and_then(serde_json::Value::as_str) {
        Some(target) => format!("tour terminé : {stop_reason} — réponse vers {target}"),
        None => format!("tour terminé : {stop_reason}"),
    }
}

fn short_timestamp(timestamp: Option<&str>) -> &str {
    timestamp
        .and_then(|value| value.get(11..16))
        .filter(|value| {
            value.as_bytes().get(2) == Some(&b':')
                && value
                    .bytes()
                    .enumerate()
                    .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
        })
        .unwrap_or("??:??")
}

#[derive(Debug)]
struct SanitizedText {
    text: String,
    truncated: bool,
}

fn sanitize_data(value: &str, limit: usize) -> SanitizedText {
    let mut text = String::new();
    let mut output_chars = 0usize;
    let mut combining_run = 0usize;
    let mut combining_overflow_visible = false;
    let mut truncated = false;

    for character in value.chars() {
        let representation = if character == '\n' || character == '\t' {
            combining_run = 0;
            combining_overflow_visible = false;
            character.to_string()
        } else if is_combining_mark(character) {
            combining_run += 1;
            if combining_run <= MAX_CONSECUTIVE_COMBINING_MARKS {
                character.to_string()
            } else if !combining_overflow_visible {
                combining_overflow_visible = true;
                "·".to_string()
            } else {
                continue;
            }
        } else {
            combining_run = 0;
            combining_overflow_visible = false;
            visible_character(character)
        };
        let representation_chars = representation.chars().count();
        if output_chars.saturating_add(representation_chars) > limit {
            truncated = true;
            break;
        }
        text.push_str(&representation);
        output_chars += representation_chars;
    }

    SanitizedText { text, truncated }
}

fn visible_character(character: char) -> String {
    match character {
        '\u{001b}' => "␛".to_string(),
        '\u{0008}' => "␈".to_string(),
        '\r' => "␍".to_string(),
        '\u{007f}' => "␡".to_string(),
        character if is_format_character(character) => "·".to_string(),
        character if character.is_control() && (character as u32) <= 0x1f => {
            char::from_u32(0x2400 + character as u32)
                .unwrap_or('�')
                .to_string()
        }
        character if character.is_control() => format!("<U+{:04X}>", character as u32),
        character => character.to_string(),
    }
}

fn is_format_character(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{061c}'
            | '\u{06dd}'
            | '\u{070f}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08e2}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
            | '\u{110bd}'
            | '\u{110cd}'
            | '\u{13430}'..='\u{1345f}'
            | '\u{1bca0}'..='\u{1bca3}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0001}'
            | '\u{e0020}'..='\u{e007f}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{e0100}'..='\u{e01ef}'
    ) || matches!(character, '\u{0600}'..='\u{0605}' | '\u{2065}')
}

fn is_combining_mark(character: char) -> bool {
    matches!(
        character,
        '\u{0300}'..='\u{036f}'
            | '\u{1ab0}'..='\u{1aff}'
            | '\u{1dc0}'..='\u{1dff}'
            | '\u{20d0}'..='\u{20ff}'
            | '\u{fe20}'..='\u{fe2f}'
    )
}

fn sanitize_inline(value: &str) -> String {
    let sanitized = sanitize_data(value, MAX_RENDERED_LABEL_CHARS);
    let mut text = sanitized.text.replace('\n', "↵").replace('\t', "⇥");
    if sanitized.truncated {
        text.push('…');
    }
    text
}

fn render_prefixed(prefix: &str, content: &str) -> String {
    let prefix = sanitize_inline(prefix);
    let indent = " ".repeat(prefix.chars().count().saturating_add(1));
    let sanitized = sanitize_data(content, MAX_RENDERED_EVENT_CHARS);
    let truncation_marker = "… [affichage tronqué]";
    let reserved = truncation_marker.chars().count();
    let content_limit = MAX_RENDERED_EVENT_CHARS.saturating_sub(reserved);
    let mut rendered = format!("{prefix} ");
    let mut rendered_chars = rendered.chars().count();
    let mut truncated = sanitized.truncated;

    for character in sanitized.text.chars() {
        let addition = if character == '\n' {
            format!("\n{indent}")
        } else {
            character.to_string()
        };
        let addition_chars = addition.chars().count();
        if rendered_chars.saturating_add(addition_chars) > content_limit {
            truncated = true;
            break;
        }
        rendered.push_str(&addition);
        rendered_chars += addition_chars;
    }
    if truncated {
        rendered.push_str(truncation_marker);
    }
    rendered
}

fn attach_refusal_message(
    reason: &AttachRefusal,
    agent: &str,
    attachable_agents: &[String],
) -> String {
    match reason {
        AttachRefusal::AgentUnknown => {
            if attachable_agents.is_empty() {
                format!(
                    "équipier « {} » inconnu ; aucun équipier ACP n’est actuellement attachable",
                    sanitize_inline(agent)
                )
            } else {
                format!(
                    "équipier « {} » inconnu ; équipiers attachables : {}",
                    sanitize_inline(agent),
                    attachable_agents
                        .iter()
                        .map(|name| sanitize_inline(name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        AttachRefusal::AgentNotAcp => format!(
            "« {} » est un agent interactif tmux ; ouvrez son pane dans la session tmux au lieu d’utiliser attach",
            sanitize_inline(agent)
        ),
        AttachRefusal::WrapperUnavailable => format!(
            "le wrapper ACP de « {} » est indisponible",
            sanitize_inline(agent)
        ),
        AttachRefusal::CommandQueueSaturated => {
            "le relais de journal est saturé ; réessayez dans un instant".to_string()
        }
        AttachRefusal::InvalidDate => {
            "date attach invalide (format attendu : AAAA-MM-JJ)".to_string()
        }
        AttachRefusal::FutureDate => {
            "la date attach est dans le futur sur l’hôte du journal".to_string()
        }
        AttachRefusal::DateOutsideRetention => {
            "la date attach est hors de la rétention du journal".to_string()
        }
        AttachRefusal::ReplyNotAllowed => {
            "les envois depuis attach doivent utiliser reply=false".to_string()
        }
        AttachRefusal::MessageOutsideAttachRole => {
            "message interdit pour une connexion attach".to_string()
        }
    }
}

fn list_attachable_agents(socket_path: &Path) -> Vec<String> {
    let Ok(stream) = UnixStream::connect(socket_path) else {
        return Vec::new();
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let Ok(read_stream) = stream.try_clone() else {
        return Vec::new();
    };
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    let probe_name = format!("attach-list-{}", std::process::id());
    let register = WrapperToDaemon::Register {
        agent_type: "attach-list".to_string(),
        name: Some(probe_name),
        host: None,
        transport: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
    };
    if write_plain_message(&mut writer, &register).is_err() {
        return Vec::new();
    }
    let mut line = String::new();
    if reader
        .read_line(&mut line)
        .ok()
        .filter(|read| *read > 0)
        .is_none()
    {
        return Vec::new();
    }
    line.clear();
    if write_plain_message(&mut writer, &WrapperToDaemon::ListAgents).is_err()
        || reader
            .read_line(&mut line)
            .ok()
            .filter(|read| *read > 0)
            .is_none()
    {
        return Vec::new();
    }
    let mut agents = match decode::<DaemonToWrapper>(line.trim_end()) {
        Ok(DaemonToWrapper::AgentList { agents }) => agents
            .into_iter()
            .filter(|agent| agent.transport == "acp" && agent.state != "unreachable")
            .map(|agent| agent.name)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let _ = write_plain_message(&mut writer, &WrapperToDaemon::Unregister);
    agents.sort();
    agents.dedup();
    agents
}

fn write_plain_message(
    writer: &mut BufWriter<UnixStream>,
    message: &WrapperToDaemon,
) -> Result<(), String> {
    let line = encode(message).map_err(|error| error.to_string())?;
    writeln!(writer, "{line}")
        .and_then(|()| writer.flush())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridget_transport::protocol::MAX_ATTACH_FRAGMENT_BYTES;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Write};
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::net::UnixListener;
    use std::sync::mpsc;

    fn test_renderer_sender(tty_output: bool) -> RendererSender {
        let (commands, _command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        RendererSender {
            commands,
            input_dirty: Arc::new(AtomicBool::new(false)),
            tty_output,
        }
    }

    struct PseudoTerminal {
        master: RawFd,
        slave: RawFd,
    }

    impl PseudoTerminal {
        fn open() -> Self {
            let mut master = -1;
            let mut slave = -1;
            assert_eq!(
                unsafe {
                    libc::openpty(
                        &mut master,
                        &mut slave,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                },
                0,
                "openpty: {}",
                std::io::Error::last_os_error()
            );
            Self { master, slave }
        }

        fn attrs(&self) -> libc::termios {
            let mut attributes = MaybeUninit::<libc::termios>::uninit();
            assert_eq!(
                unsafe { libc::tcgetattr(self.slave, attributes.as_mut_ptr()) },
                0,
                "tcgetattr: {}",
                std::io::Error::last_os_error()
            );
            unsafe { attributes.assume_init() }
        }

        fn close_master(&mut self) {
            if self.master >= 0 {
                assert_eq!(unsafe { libc::close(self.master) }, 0);
                self.master = -1;
            }
        }
    }

    impl Drop for PseudoTerminal {
        fn drop(&mut self) {
            for fd in [self.master, self.slave] {
                if fd >= 0 {
                    unsafe { libc::close(fd) };
                }
            }
        }
    }

    fn assert_terminal_restored(before: &libc::termios, after: &libc::termios) {
        // Le pilote pseudo-TTY macOS peut rétablir ses bits ECHOCTL propres
        // après une lecture. Les seuls bits mutés par notre garde doivent
        // donc retrouver leur valeur initiale ; ils prouvent la restauration.
        let raw_bits = (libc::ICANON | libc::ECHO | libc::ISIG) as libc::tcflag_t;
        assert_eq!(after.c_lflag & raw_bits, before.c_lflag & raw_bits);
        assert_eq!(after.c_cc[libc::VMIN], before.c_cc[libc::VMIN]);
        assert_eq!(after.c_cc[libc::VTIME], before.c_cc[libc::VTIME]);
    }

    fn fragment(
        subscription_id: &str,
        seq: u64,
        offset: usize,
        final_fragment: bool,
        bytes: Vec<u8>,
    ) -> DaemonToWrapper {
        DaemonToWrapper::JournalFragment {
            subscription_id: subscription_id.to_string(),
            seq,
            offset: offset as u64,
            final_fragment,
            bytes,
        }
    }

    fn subscribe(state: &mut AttachClientState, id: &str) {
        state.subscription_requested();
        let result = state
            .dispatch(DaemonToWrapper::Subscribed {
                subscription_id: id.to_string(),
            })
            .unwrap();
        assert_eq!(
            result.events,
            vec![AttachEvent::Subscribed {
                subscription_id: id.to_string()
            }]
        );
    }

    fn journal_record(seq: u64, event: &str, payload: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "v": 1,
            "seq": seq,
            "ts": "2026-08-23T09:07:00Z",
            "session_id": "session-1",
            "message_id": "message-1",
            "event": event,
            "payload": payload,
        }))
        .unwrap()
    }

    #[test]
    fn matrice_stdin_stdout_selectionne_le_renderer_par_stdout_seul() {
        for (stdin_tty, stdout_tty) in [(false, false), (true, false), (false, true), (true, true)]
        {
            let renderer = BlockRenderer::new("codex-1".to_string(), stdin_tty, stdout_tty);
            assert_eq!(renderer.tty_output, stdout_tty);
            assert_eq!(renderer.raw_terminal, stdin_tty);
        }
    }

    #[test]
    fn stdout_non_tty_reste_identique_octet_pour_octet_quel_que_soit_stdin() {
        let event = AttachEvent::Gap {
            from_seq: 4,
            to_seq: 5,
            reason: Some("vue lente".to_string()),
        };
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"frappe".to_vec(),
        }));
        let rendered = render_attach_event(&event, "codex-1");

        let mut piped_stdin = BlockRenderer::new("codex-1".to_string(), false, false);
        let mut piped_output = Vec::new();
        piped_stdin.apply(
            RendererCommand::Event(event.clone()),
            &input,
            &mut piped_output,
        );
        assert_eq!(piped_output, format!("{rendered}\n").as_bytes());

        let mut tty_stdin = BlockRenderer::new("codex-1".to_string(), true, false);
        let mut tty_output = Vec::new();
        tty_stdin.apply(RendererCommand::Event(event), &input, &mut tty_output);
        assert_eq!(tty_output, format!("\r\n{rendered}\n> frappe").as_bytes());
    }

    #[test]
    fn tour_tty_reste_un_bloc_de_la_bascule_replay_jusqu_a_la_fin_live() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), true, true);
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 1,
                bytes: journal_record(1, "turn_start", json!({"from":"humain","body":"Question"})),
                live: false,
            }),
            &input,
            &mut output,
        );
        let key = renderer.current.as_ref().unwrap().key.clone();
        renderer.apply(
            RendererCommand::Event(AttachEvent::SnapshotCaughtUp {
                through_seq: Some(1),
            }),
            &input,
            &mut output,
        );
        assert_eq!(renderer.current.as_ref().unwrap().key, key);
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 2,
                bytes: journal_record(2, "update", json!({"kind":"text","content":"Réponse live"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        assert_eq!(renderer.current.as_ref().unwrap().response, "Réponse live");
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 3,
                bytes: journal_record(3, "turn_end", json!({"stop_reason":"end_turn"})),
                live: true,
            }),
            &input,
            &mut output,
        );

        assert!(renderer.current.is_none());
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Question"));
        assert!(output.contains("Réponse live"));
        assert!(output.contains("historique rattrapé jusqu’à 1"));
        assert!(output.contains("tour terminé : end_turn"));
    }

    #[test]
    fn rattrapage_tty_n_ecrit_chaque_bloc_qu_a_sa_cloture() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        let mut output = Vec::new();
        let fixture = include_str!("../tests/fixtures/attach-replay-compact.jsonl");

        for line in fixture.lines() {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            let before = output.len();
            renderer.apply(
                RendererCommand::Event(AttachEvent::Journal {
                    seq: value["seq"].as_u64().unwrap(),
                    bytes: line.as_bytes().to_vec(),
                    live: false,
                }),
                &input,
                &mut output,
            );
            if value["event"] == "turn_end" {
                assert!(output.len() > before, "le bloc clos doit être rendu");
                assert!(renderer.current.is_none());
            } else {
                assert_eq!(
                    output.len(),
                    before,
                    "le rejeu ne doit pas redessiner un tour encore ouvert"
                );
            }
        }

        renderer.apply(
            RendererCommand::Event(AttachEvent::SnapshotCaughtUp {
                through_seq: Some(7),
            }),
            &input,
            &mut output,
        );
        let output = String::from_utf8(output).unwrap();
        for expected in [
            "Premier tour",
            "réponse compacte",
            "Second tour",
            "déjà clos",
            "historique rattrapé jusqu’à 7",
        ] {
            assert_eq!(
                output.matches(expected).count(),
                1,
                "sortie compacte manquante ou dupliquée : {expected}"
            );
        }
    }

    #[test]
    fn perte_de_correlation_apres_snapshot_evacuant_le_tour_incomplet() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 1,
                bytes: journal_record(1, "turn_start", json!({"from":"humain","body":"Question"})),
                live: false,
            }),
            &input,
            &mut output,
        );
        renderer.apply(
            RendererCommand::Event(AttachEvent::SnapshotCaughtUp {
                through_seq: Some(1),
            }),
            &input,
            &mut output,
        );
        let journal_sans_correlation = serde_json::to_vec(&json!({
            "v": 1,
            "seq": 2,
            "ts": "2026-08-23T09:07:01Z",
            "event": "error",
            "payload": {"reason":"corrélation absente"},
        }))
        .unwrap();
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 2,
                bytes: journal_sans_correlation,
                live: true,
            }),
            &input,
            &mut output,
        );

        assert!(renderer.current.is_none());
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("historique rattrapé jusqu’à 1"));
        assert!(output.contains("[tour incomplet]"));
        assert!(output.contains("corrélation absente"));
    }

    #[test]
    fn diagnostic_non_terminal_ne_ferme_pas_le_bloc() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (2, "error", json!({"reason":"diagnostic transitoire"})),
            (3, "update", json!({"kind":"text","content":"suite"})),
        ] {
            renderer.apply(
                RendererCommand::Event(AttachEvent::Journal {
                    seq,
                    bytes: journal_record(seq, event, payload),
                    live: true,
                }),
                &input,
                &mut output,
            );
        }
        let current = renderer.current.as_ref().unwrap();
        assert_eq!(current.response, "suite");
        assert!(
            current
                .details
                .iter()
                .any(|line| line.contains("diagnostic transitoire"))
        );
    }

    #[test]
    fn gap_end_et_ligne_corrompue_evacuant_un_tour_le_signalent_incomplet() {
        let boundaries = [
            (
                AttachEvent::Gap {
                    from_seq: 2,
                    to_seq: 3,
                    reason: Some("vue lente".to_string()),
                },
                "non affiché",
            ),
            (
                AttachEvent::End {
                    reason: "source_replaced".to_string(),
                },
                "abonnement terminé",
            ),
            (
                AttachEvent::Journal {
                    seq: 2,
                    bytes: b"{invalide".to_vec(),
                    live: true,
                },
                "événement journal illisible",
            ),
        ];
        for (boundary, expected) in boundaries {
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
            let mut output = Vec::new();
            renderer.apply(
                RendererCommand::Event(AttachEvent::Journal {
                    seq: 1,
                    bytes: journal_record(
                        1,
                        "turn_start",
                        json!({"from":"humain","body":"Question"}),
                    ),
                    live: false,
                }),
                &input,
                &mut output,
            );
            renderer.apply(RendererCommand::Event(boundary), &input, &mut output);

            assert!(renderer.current.is_none());
            let output = String::from_utf8(output).unwrap();
            assert!(output.contains("[tour incomplet]"));
            assert!(output.contains(expected));
        }
    }

    #[test]
    fn champ_terminal_explicite_ferme_le_bloc_meme_sur_un_diagnostic() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (
                2,
                "error",
                json!({"reason":"transport arrêté","terminal":true}),
            ),
        ] {
            renderer.apply(
                RendererCommand::Event(AttachEvent::Journal {
                    seq,
                    bytes: journal_record(seq, event, payload),
                    live: true,
                }),
                &input,
                &mut output,
            );
        }
        assert!(renderer.current.is_none());
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("transport arrêté")
        );
    }

    #[test]
    fn bloc_borne_affiche_sa_troncature() {
        let mut block = TurnBlock::new(
            TurnKey {
                session_id: "session".to_string(),
                message_id: "message".to_string(),
            },
            "09:07 humain → Question".to_string(),
        );
        for _ in 0..500 {
            block.append_response(&format!("{}\n", "x".repeat(200)));
        }

        assert!(block.stored_bytes <= MAX_TURN_BLOCK_BYTES);
        assert!(block.stored_lines <= MAX_TURN_BLOCK_LINES);
        assert!(block.omitted_lines > 0);
        assert!(
            block
                .lines("codex-1")
                .last()
                .unwrap()
                .starts_with("… tronqué,")
        );
    }

    #[test]
    fn saturation_coalesce_la_saisie_sans_perdre_ctrl_c_ni_la_restauration() {
        let pseudo_terminal = PseudoTerminal::open();
        let before = pseudo_terminal.attrs();
        let (commands, command_rx) = mpsc::sync_channel(1);
        let dirty = Arc::new(AtomicBool::new(false));
        let sender = RendererSender {
            commands,
            input_dirty: dirty.clone(),
            tty_output: true,
        };
        sender
            .commands
            .send(RendererCommand::Event(AttachEvent::Subscribed {
                subscription_id: "sub".to_string(),
            }))
            .unwrap();
        sender.input_changed(b"x");
        assert!(dirty.load(Ordering::Acquire));

        with_raw_terminal(pseudo_terminal.slave, |raw_terminal| {
            assert!(raw_terminal);
            let (write_stream, _) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            assert!(!handle_input_byte(
                0x03, &state, &input, &sender, &writer, "codex-1",
            )?);
            let (control_tx, control_rx) = mpsc::channel();
            control_tx.send(RendererControl::Stop).unwrap();
            assert!(matches!(control_rx.try_recv(), Ok(RendererControl::Stop)));
            Ok(())
        })
        .unwrap();
        assert_terminal_restored(&before, &pseudo_terminal.attrs());
        assert!(matches!(
            command_rx.try_recv(),
            Ok(RendererCommand::Event(_))
        ));
        assert!(matches!(
            command_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }

    #[test]
    fn reassemble_un_evenement_fragmenté_en_rejeu_puis_en_live_sans_boucle() {
        let mut state = AttachClientState::new(AttachWindow::Today);
        subscribe(&mut state, "sub-1");
        let size = MAX_ATTACH_FRAGMENT_BYTES + 80_000;
        let payload = vec![b'a'; size];

        let first = state
            .dispatch(fragment(
                "sub-1",
                1,
                0,
                false,
                payload[..MAX_ATTACH_FRAGMENT_BYTES].to_vec(),
            ))
            .unwrap();
        assert!(first.events.is_empty());
        let replay = state
            .dispatch(fragment(
                "sub-1",
                1,
                MAX_ATTACH_FRAGMENT_BYTES,
                true,
                payload[MAX_ATTACH_FRAGMENT_BYTES..].to_vec(),
            ))
            .unwrap();
        assert_eq!(
            replay.events,
            vec![AttachEvent::Journal {
                seq: 1,
                bytes: payload.clone(),
                live: false,
            }]
        );

        state
            .dispatch(DaemonToWrapper::SnapshotCaughtUp {
                subscription_id: "sub-1".to_string(),
                through_seq: Some(1),
            })
            .unwrap();
        let live_first = state
            .dispatch(fragment(
                "sub-1",
                2,
                0,
                false,
                payload[..MAX_ATTACH_FRAGMENT_BYTES].to_vec(),
            ))
            .unwrap();
        assert!(live_first.events.is_empty());
        let live = state
            .dispatch(fragment(
                "sub-1",
                2,
                MAX_ATTACH_FRAGMENT_BYTES,
                true,
                payload[MAX_ATTACH_FRAGMENT_BYTES..].to_vec(),
            ))
            .unwrap();
        assert_eq!(
            live.events,
            vec![AttachEvent::Journal {
                seq: 2,
                bytes: payload,
                live: true,
            }]
        );
        assert_eq!(state.last_seq, Some(2));
    }

    #[test]
    fn abandonne_un_evenement_de_plus_de_quatre_mio_avec_un_seul_gap() {
        let mut state = AttachClientState::new(AttachWindow::Today);
        subscribe(&mut state, "sub-1");
        let chunk = vec![b'x'; MAX_ATTACH_FRAGMENT_BYTES];
        let mut offset = 0;
        let mut events = Vec::new();
        while offset <= MAX_REASSEMBLY_BYTES + MAX_ATTACH_FRAGMENT_BYTES {
            let final_fragment = offset > MAX_REASSEMBLY_BYTES;
            events.extend(
                state
                    .dispatch(fragment("sub-1", 7, offset, final_fragment, chunk.clone()))
                    .unwrap()
                    .events,
            );
            offset += chunk.len();
            if final_fragment {
                break;
            }
        }

        assert_eq!(
            events,
            vec![AttachEvent::Gap {
                from_seq: 7,
                to_seq: 7,
                reason: Some("event_too_large".to_string()),
            }]
        );
        assert!(state.reassembly.is_none());
        assert_eq!(state.last_seq, None);
    }

    #[test]
    fn snapshot_vide_end_reprend_le_selecteur_initial_et_ignore_l_ancienne_generation() {
        let mut state = AttachClientState::new(AttachWindow::Date("2026-08-22".to_string()));
        subscribe(&mut state, "ancienne");
        state
            .dispatch(DaemonToWrapper::SnapshotCaughtUp {
                subscription_id: "ancienne".to_string(),
                through_seq: None,
            })
            .unwrap();
        state
            .pending_send
            .insert("message-1".to_string(), "texte".to_string());

        let end = state
            .dispatch(DaemonToWrapper::End {
                subscription_id: "ancienne".to_string(),
                reason: "source_replaced".to_string(),
            })
            .unwrap();
        assert_eq!(
            end.resubscribe,
            Some(AttachWindow::Date("2026-08-22".to_string()))
        );
        assert!(state.pending_send.contains_key("message-1"));

        assert!(
            state
                .dispatch(fragment("ancienne", 1, 0, true, b"ancien".to_vec()))
                .unwrap()
                .events
                .is_empty()
        );
        let subscribed = state
            .dispatch(DaemonToWrapper::Subscribed {
                subscription_id: "nouvelle".to_string(),
            })
            .unwrap();
        assert_eq!(subscribed.events.len(), 1);
        let first = state
            .dispatch(fragment("nouvelle", 1, 0, true, b"premier".to_vec()))
            .unwrap();
        assert_eq!(
            first.events,
            vec![AttachEvent::Journal {
                seq: 1,
                bytes: b"premier".to_vec(),
                live: false,
            }]
        );
        assert!(
            state
                .dispatch(fragment("ancienne", 2, 0, true, b"retard".to_vec()))
                .unwrap()
                .events
                .is_empty()
        );
    }

    #[test]
    fn reconnexion_reprend_a_la_sequence_suivante_et_purge_les_envois() {
        let mut state = AttachClientState::new(AttachWindow::Today);
        subscribe(&mut state, "sub-1");
        state
            .dispatch(fragment("sub-1", 41, 0, true, b"event".to_vec()))
            .unwrap();
        state
            .pending_send
            .insert("message-1".to_string(), "texte".to_string());

        state.connection_closed();

        assert_eq!(state.subscription_requested(), AttachWindow::Seq(42));
        assert!(state.pending_send.is_empty());
    }

    #[test]
    fn ack_ne_purge_pas_un_envoi_mais_le_rejet_tardif_oui_meme_apres_end() {
        let mut state = AttachClientState::new(AttachWindow::Today);
        subscribe(&mut state, "sub-1");
        state
            .pending_send
            .insert("message-1".to_string(), "texte".to_string());

        state
            .dispatch(DaemonToWrapper::Ack {
                id: "message-1".to_string(),
            })
            .unwrap();
        state
            .dispatch(DaemonToWrapper::End {
                subscription_id: "sub-1".to_string(),
                reason: "arrêt".to_string(),
            })
            .unwrap();
        assert!(state.pending_send.contains_key("message-1"));
        let rejected = state
            .dispatch(DaemonToWrapper::DeliveryRejected {
                id: "message-1".to_string(),
                reason: "transport arrêté".to_string(),
            })
            .unwrap();

        assert_eq!(
            rejected.events,
            vec![AttachEvent::SendRejected {
                message_id: "message-1".to_string(),
                delayed: true,
                reason: "transport arrêté".to_string(),
            }]
        );
        assert!(state.pending_send.is_empty());
    }

    #[test]
    fn saisie_attach_envoie_reply_false_et_conserve_la_correlation_jusqu_au_rejet_tardif() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(false);

        for byte in b"bonjour\n" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1",).unwrap()
            );
        }

        let mut reader = BufReader::new(read_stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu sur la connexion attach");
        };
        assert_eq!(message.from, "humain");
        assert_eq!(message.to, "codex-1");
        assert_eq!(message.body, "bonjour");
        assert!(!message.reply);
        let message_id = message.id;

        let acknowledged = state
            .lock()
            .unwrap()
            .dispatch(DaemonToWrapper::Ack {
                id: message_id.clone(),
            })
            .unwrap();
        assert_eq!(
            acknowledged.events,
            vec![AttachEvent::SendAcknowledged {
                message_id: message_id.clone()
            }]
        );
        assert!(state.lock().unwrap().pending_send.contains_key(&message_id));

        state
            .lock()
            .unwrap()
            .dispatch(DaemonToWrapper::End {
                subscription_id: "ancienne-generation".to_string(),
                reason: "wrapper parti".to_string(),
            })
            .unwrap();
        let rejected = state
            .lock()
            .unwrap()
            .dispatch(DaemonToWrapper::DeliveryRejected {
                id: message_id.clone(),
                reason: "file ACP pleine".to_string(),
            })
            .unwrap();
        assert_eq!(
            rejected.events,
            vec![AttachEvent::SendRejected {
                message_id: message_id.clone(),
                delayed: true,
                reason: "file ACP pleine".to_string(),
            }]
        );
        assert!(!state.lock().unwrap().pending_send.contains_key(&message_id));
    }

    #[test]
    fn evenement_hostile_ne_modifie_jamais_la_saisie_partielle() {
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"r\xc3\xa9ponse en cours".to_vec(),
        }));
        let hostile = include_bytes!("../tests/fixtures/attach-hostile.jsonl");
        let event = AttachEvent::Journal {
            seq: 9,
            bytes: hostile.strip_suffix(b"\n").unwrap().to_vec(),
            live: true,
        };
        let expected = input.lock().unwrap().bytes.clone();
        let mut renderer = BlockRenderer::new("codex-1".to_string(), true, true);
        let mut output = Vec::new();
        renderer.apply(RendererCommand::Event(event), &input, &mut output);
        assert_eq!(input.lock().unwrap().bytes, expected);
    }

    #[test]
    fn expiration_d_un_envoi_est_terminale_et_affiche_son_motif() {
        let mut state = AttachClientState::new(AttachWindow::Today);
        let now = Instant::now();
        state.track_send("message-expire".to_string(), "texte".to_string(), now);
        let events =
            state.expire_pending_sends(now + SEND_ISSUE_TIMEOUT + Duration::from_millis(1));
        assert_eq!(
            events,
            vec![AttachEvent::SendRejected {
                message_id: "message-expire".to_string(),
                delayed: true,
                reason: "délai d'issue dépassé".to_string(),
            }]
        );
        assert!(state.pending_send.is_empty());
    }

    #[test]
    fn ctrl_c_et_eof_sur_tampon_vide_quittent_proprement_la_boucle_de_saisie() {
        let (write_stream, _) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(false);
        assert!(!handle_input_byte(0x03, &state, &input, &renderer, &writer, "codex-1",).unwrap());
        assert!(!handle_input_byte(0x04, &state, &input, &renderer, &writer, "codex-1",).unwrap());
    }

    #[test]
    fn pseudo_tty_polin_hup_livre_le_dernier_send_et_restaure_le_terminal() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let (client_stream, server_stream) = UnixStream::pair().unwrap();
        let connection = AttachConnection {
            reader: BufReader::new(client_stream.try_clone().unwrap()),
            writer: Arc::new(Mutex::new(BufWriter::new(client_stream))),
        };
        let (sent_tx, sent_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let mut reader = BufReader::new(server_stream.try_clone().unwrap());
            let mut writer = BufWriter::new(server_stream);
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::JournalFragment {
                    subscription_id: "sub-pty".to_string(),
                    seq: 7,
                    offset: 0,
                    final_fragment: true,
                    bytes: br#"{"v":1,"seq":7,"event":"update"}"#.to_vec(),
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let message = decode::<WrapperToDaemon>(line.trim_end()).unwrap();
            sent_tx.send(()).unwrap();
            message
        });

        let (mut input_writer, input_reader) = UnixStream::pair().unwrap();
        let (input_start_tx, input_start_rx) = mpsc::channel();
        let (event_rendered_tx, event_rendered_rx) = mpsc::channel();
        let input_writer = thread::spawn(move || {
            input_start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            event_rendered_rx
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
            assert_eq!(input_writer.write(b"dernier envoi\n").unwrap(), 14);
            input_writer.shutdown(Shutdown::Write).unwrap();
        });
        let mut state = AttachClientState::new(AttachWindow::Today);
        state.event_observer = Some(event_rendered_tx);
        state.subscription_requested();
        state
            .dispatch(DaemonToWrapper::Subscribed {
                subscription_id: "sub-pty".to_string(),
            })
            .unwrap();

        let result = with_raw_terminal(pseudo_tty.slave, |raw_terminal| {
            input_start_tx.send(()).unwrap();
            drive_interactive(
                connection,
                &mut state,
                "codex-1",
                Path::new("/tmp/bridget-attach-pty-unused.sock"),
                input_reader.as_raw_fd(),
                raw_terminal,
                true,
            )
        });
        assert!(result.is_ok());
        input_writer.join().unwrap();
        sent_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_terminal_restored(&before, &pseudo_tty.attrs());
        assert_eq!(state.last_seq, Some(7), "l'événement est traité avant EOF");
        assert!(matches!(
            server.join().unwrap(),
            WrapperToDaemon::Send(message)
                if message.body == "dernier envoi"
                    && message.to == "codex-1"
                    && !message.reply
        ));
    }

    #[test]
    fn retour_arriere_retire_un_scalaire_utf8_entier() {
        let mut input = InputBuffer {
            bytes: "réponse".as_bytes().to_vec(),
        };
        assert!(input.erase_last());
        assert_eq!(input.display(), "répons");
    }

    #[test]
    fn negocie_le_role_attach() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let socket_path =
            Path::new("/tmp").join(format!("bg-h-{}-{nonce}.sock", std::process::id()));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let (observed_tx, observed_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            observed_tx
                .send(decode::<WrapperToDaemon>(line.trim_end()).unwrap())
                .unwrap();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Attach
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });

        let mut connection = AttachConnection::connect(&socket_path).unwrap();
        connection.accept_role().unwrap();
        assert!(matches!(
            observed_rx.recv().unwrap(),
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            }
        ));
        server.join().unwrap();
        std::fs::remove_file(socket_path).unwrap();
    }

    #[test]
    fn writer_concurrent_produit_des_frames_entieres() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let workers = (0..4)
            .map(|_| {
                let writer = writer.clone();
                thread::spawn(move || {
                    for _ in 0..32 {
                        write_socket_message(&writer, &WrapperToDaemon::Heartbeat).unwrap();
                    }
                })
            })
            .collect::<Vec<_>>();
        let reader = thread::spawn(move || {
            let mut reader = BufReader::new(read_stream);
            for _ in 0..128 {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::Heartbeat
                ));
            }
        });

        for worker in workers {
            worker.join().unwrap();
        }
        reader.join().unwrap();
    }

    #[test]
    fn socket_snapshot_vide_end_reabonne_et_livre_le_premier_evenement_une_fois() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let socket_path =
            Path::new("/tmp").join(format!("bg-r-{}-{nonce}.sock", std::process::id()));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut read_client = || {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                decode::<WrapperToDaemon>(line.trim_end()).unwrap()
            };
            let mut send = |message: DaemonToWrapper| {
                writeln!(writer, "{}", encode(&message).unwrap()).unwrap();
                writer.flush().unwrap();
            };

            assert!(matches!(
                read_client(),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach
                }
            ));
            send(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach,
            });
            assert!(matches!(
                read_client(),
                WrapperToDaemon::Subscribe {
                    window: AttachWindow::Date(ref date),
                    ..
                } if date == "2026-08-22"
            ));
            send(DaemonToWrapper::Subscribed {
                subscription_id: "ancienne".to_string(),
            });
            send(DaemonToWrapper::SnapshotCaughtUp {
                subscription_id: "ancienne".to_string(),
                through_seq: None,
            });
            send(DaemonToWrapper::End {
                subscription_id: "ancienne".to_string(),
                reason: "source_replaced".to_string(),
            });
            assert!(matches!(
                read_client(),
                WrapperToDaemon::Subscribe {
                    window: AttachWindow::Date(ref date),
                    ..
                } if date == "2026-08-22"
            ));
            send(fragment("ancienne", 1, 0, true, b"retard".to_vec()));
            send(DaemonToWrapper::Subscribed {
                subscription_id: "nouvelle".to_string(),
            });
            send(fragment("nouvelle", 1, 0, true, b"premier".to_vec()));
            send(DaemonToWrapper::Disconnect);
        });

        let initial_window = AttachWindow::Date("2026-08-22".to_string());
        let mut state = AttachClientState::new(initial_window);
        let mut connection = AttachConnection::connect(&socket_path).unwrap();
        connection.accept_role().unwrap();
        let window = state.subscription_requested();
        connection.subscribe("codex-1", window).unwrap();
        let mut events = Vec::new();
        drive_connection(
            &mut connection,
            &mut state,
            "codex-1",
            &socket_path,
            &mut |event| {
                events.push(event.clone());
            },
        )
        .unwrap();

        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, AttachEvent::Journal { .. }))
                .count(),
            1
        );
        assert!(events.iter().any(|event| matches!(
            event,
            AttachEvent::Journal { seq: 1, bytes, .. } if bytes == b"premier"
        )));
        assert_eq!(state.last_seq, Some(1));
        server.join().unwrap();
        std::fs::remove_file(socket_path).unwrap();
    }

    #[test]
    fn reconnexion_socket_reprend_exactement_a_last_seq_plus_un() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let socket_path =
            Path::new("/tmp").join(format!("bg-c-{}-{nonce}.sock", std::process::id()));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            for generation in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Attach
                    }
                ));
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Attach
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();

                line.clear();
                reader.read_line(&mut line).unwrap();
                let subscription = decode::<WrapperToDaemon>(line.trim_end()).unwrap();
                if generation == 0 {
                    assert!(matches!(
                        subscription,
                        WrapperToDaemon::Subscribe {
                            window: AttachWindow::Today,
                            ..
                        }
                    ));
                    for message in [
                        DaemonToWrapper::Subscribed {
                            subscription_id: "sub-1".to_string(),
                        },
                        fragment("sub-1", 5, 0, true, b"event".to_vec()),
                    ] {
                        writeln!(writer, "{}", encode(&message).unwrap()).unwrap();
                    }
                    writer.flush().unwrap();
                } else {
                    assert!(matches!(
                        subscription,
                        WrapperToDaemon::Subscribe {
                            window: AttachWindow::Seq(6),
                            ..
                        }
                    ));
                    writeln!(
                        writer,
                        "{}",
                        encode(&DaemonToWrapper::AttachRejected {
                            subscription_id: None,
                            reason: AttachRefusal::WrapperUnavailable,
                        })
                        .unwrap()
                    )
                    .unwrap();
                    writer.flush().unwrap();
                }
            }
        });

        let error = run("codex-1", AttachWindow::Today, &socket_path).unwrap_err();
        assert!(error.contains("wrapper ACP de « codex-1 » est indisponible"));
        server.join().unwrap();
        std::fs::remove_file(socket_path).unwrap();
    }

    #[test]
    fn borne_une_frame_socket_et_consomme_sa_fin() {
        let (reader_stream, mut writer_stream) = UnixStream::pair().unwrap();
        let writer = thread::spawn(move || {
            writer_stream
                .write_all(&vec![b'x'; MAX_ATTACH_SERIALIZED_FRAME_BYTES + 1])
                .unwrap();
            writer_stream.write_all(b"\n{}\n").unwrap();
        });
        let mut reader = BufReader::new(reader_stream);

        let error = read_bounded_frame(&mut reader).unwrap_err();
        assert!(error.contains("trop volumineuse"));
        assert_eq!(
            read_bounded_frame(&mut reader).unwrap(),
            Some(b"{}".to_vec())
        );
        writer.join().unwrap();
    }

    #[test]
    fn borne_la_memoire_des_generations_retires() {
        let mut state = AttachClientState::new(AttachWindow::Today);
        for index in 0..RETIRED_SUBSCRIPTIONS_LIMIT + 10 {
            state.retire(format!("sub-{index}"));
        }
        assert_eq!(
            state.retired_subscriptions.len(),
            RETIRED_SUBSCRIPTIONS_LIMIT
        );
        assert!(!state.retired_subscriptions.contains(&"sub-0".to_string()));
        assert!(
            state
                .retired_subscriptions
                .contains(&format!("sub-{}", RETIRED_SUBSCRIPTIONS_LIMIT + 9))
        );
    }

    #[test]
    fn rend_les_payloads_v1_des_fixtures_gelee_sans_json_brut() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../bridget-transport/tests/fixtures/journal");
        let complete = std::fs::read_to_string(fixture_root.join("complete-turn.jsonl")).unwrap();
        let rendered = complete
            .lines()
            .map(|line| render_journal_event(line.as_bytes(), "codex-1"))
            .collect::<Vec<_>>();
        assert_eq!(rendered[0], "00:00 alice → bonjour intégral");
        assert_eq!(rendered[1], "00:00 codex-1 → réponse");
        assert_eq!(
            rendered[2],
            "00:00 [fin] tour terminé : end_turn — réponse vers alice"
        );

        let permission = std::fs::read_to_string(fixture_root.join("permission.jsonl")).unwrap();
        assert_eq!(
            render_journal_event(permission.trim_end().as_bytes(), "codex-1"),
            "00:00 [permission] écrire autorisation décidée : allow-1"
        );
        let error = std::fs::read_to_string(fixture_root.join("error.jsonl")).unwrap();
        assert_eq!(
            render_journal_event(error.trim_end().as_bytes(), "codex-1"),
            "00:00 [erreur] équipier arrêté"
        );
    }

    #[test]
    fn neutralise_la_fixture_hostile_et_indente_les_fausses_lignes() {
        let hostile = include_bytes!("../tests/fixtures/attach-hostile.jsonl");
        let rendered = render_journal_event(hostile.strip_suffix(b"\n").unwrap(), "codex-1");

        assert!(rendered.contains("␛[2J"));
        assert!(rendered.contains("␛]0;pwned␇"));
        assert!(rendered.contains("<U+0085>"));
        assert!(rendered.contains("bidi ·"));
        assert!(rendered.contains("zero ·"));
        assert!(rendered.contains("CR␍"));
        assert!(rendered.contains("backspace␈"));
        assert!(!rendered.contains('\u{001b}'));
        assert!(!rendered.contains('\u{0085}'));
        assert!(!rendered.contains('\u{202e}'));
        assert!(!rendered.contains('\u{200b}'));
        assert!(!rendered.contains('\r'));
        assert!(!rendered.contains('\u{0008}'));
        let continuation_indent = " ".repeat("10:42 codex-1 →".chars().count() + 1);
        assert!(rendered.lines().nth(1).is_some_and(|line| {
            line.starts_with(&continuation_indent) && line.contains("10:42 [erreur] forgée")
        }));
        assert!(rendered.contains(&format!("e{}· fin", "\u{0301}".repeat(8))));
        assert!(
            rendered
                .chars()
                .all(|character| character == '\n' || character == '\t' || !character.is_control())
        );
    }

    #[test]
    fn golden_delta_transport_etiquette_title_name_kind_et_assainit_le_titre() {
        let fixture = include_str!("../tests/fixtures/attach-tools-hostile.jsonl");
        let rendered = fixture
            .lines()
            .map(|line| render_journal_event(line.as_bytes(), "codex-1"))
            .collect::<Vec<_>>();

        assert_eq!(
            rendered.iter().map(String::as_str).collect::<Vec<_>>(),
            [
                "10:00 [outil] Read src/main.rs lecture",
                "10:00 [outil] Bash cargo test --workspace tests",
                "10:00 [outil] quantum_wrench kind inconnu",
                "10:00 [outil] ␛[2J␛]0;pwned␇ ·gnahc titre hostile",
            ]
        );
        assert!(rendered.iter().all(|line| !line.contains('\u{001b}')));
        assert!(rendered.iter().all(|line| !line.contains('\u{202e}')));
    }

    #[test]
    fn borne_et_signale_la_longueur_rendue() {
        let value = serde_json::json!({
            "v": 1,
            "seq": 1,
            "ts": "2026-08-22T10:42:00Z",
            "session_id": "session",
            "event": "update",
            "payload": { "kind": "text", "content": "x".repeat(MAX_RENDERED_EVENT_CHARS * 4) }
        });
        let rendered = render_journal_event(value.to_string().as_bytes(), "codex-1");
        assert!(rendered.chars().count() <= MAX_RENDERED_EVENT_CHARS);
        assert!(rendered.ends_with("… [affichage tronqué]"));
    }

    #[test]
    fn explique_les_refus_non_acp_et_nom_inconnu() {
        let non_acp = attach_refusal_message(&AttachRefusal::AgentNotAcp, "claude-1", &[]);
        assert!(non_acp.contains("interactif tmux"));
        assert!(non_acp.contains("pane"));

        let unknown = attach_refusal_message(
            &AttachRefusal::AgentUnknown,
            "absent",
            &["claude-acp".to_string(), "codex-acp".to_string()],
        );
        assert!(unknown.contains("équipier « absent » inconnu"));
        assert!(unknown.contains("claude-acp, codex-acp"));
    }

    #[test]
    fn raw_mode_lit_ctrl_c_comme_octet_et_restaure_le_pseudo_tty() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let mut raw = RawTerminal::enable_for_fd(pseudo_tty.slave)
            .unwrap()
            .expect("pseudo-TTY détecté");
        let active = pseudo_tty.attrs();
        assert_eq!(active.c_lflag & (libc::ICANON | libc::ECHO | libc::ISIG), 0);

        assert_eq!(
            unsafe { libc::write(pseudo_tty.master, [0x03_u8].as_ptr().cast(), 1) },
            1
        );
        let mut byte = 0_u8;
        assert_eq!(
            unsafe { libc::read(pseudo_tty.slave, (&mut byte as *mut u8).cast(), 1) },
            1
        );
        assert_eq!(byte, 0x03, "Ctrl-C doit arriver à la boucle de saisie");

        raw.restore().unwrap();
        assert_terminal_restored(&before, &pseudo_tty.attrs());
    }

    #[test]
    fn raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty() {
        let mut pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let raw = RawTerminal::enable_for_fd(pseudo_tty.slave)
            .unwrap()
            .expect("pseudo-TTY détecté");
        pseudo_tty.close_master();
        let mut byte = 0_u8;
        let read = unsafe { libc::read(pseudo_tty.slave, (&mut byte as *mut u8).cast(), 1) };
        assert!(read <= 0, "le pseudo-TTY fermé doit signaler EOF ou EIO");
        drop(raw);
        assert_terminal_restored(&before, &pseudo_tty.attrs());
    }

    #[test]
    fn garde_raw_restaure_le_terminal_sur_erreur_et_degrade_un_pipe() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let result = with_raw_terminal(pseudo_tty.slave, |interactive| {
            assert!(interactive);
            Err::<(), _>("erreur de boucle simulée".to_string())
        });
        assert_eq!(result.unwrap_err(), "erreur de boucle simulée");
        assert_terminal_restored(&before, &pseudo_tty.attrs());

        let mut pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
        let result = with_raw_terminal(pipe[0], |interactive| {
            assert!(!interactive, "un pipe ne passe jamais en raw mode");
            Ok::<_, String>(())
        });
        assert!(result.is_ok());
        assert_eq!(unsafe { libc::close(pipe[0]) }, 0);
        assert_eq!(unsafe { libc::close(pipe[1]) }, 0);
    }

    #[test]
    fn garde_raw_restaure_le_terminal_apres_panic() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = with_raw_terminal(pseudo_tty.slave, |_interactive| -> Result<(), String> {
                panic!("panic de boucle simulée")
            });
        }));
        assert!(panic.is_err());
        assert_terminal_restored(&before, &pseudo_tty.attrs());
    }
}
