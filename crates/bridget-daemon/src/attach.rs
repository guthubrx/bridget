//! Client de vue attach : connexion persistante, reprise et rendu sûr du journal.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    AgentInfo, AttachRefusal, AttachWindow, ConnectionRole, DaemonToWrapper,
    MAX_ATTACH_SERIALIZED_FRAME_BYTES, PresenceMode, WrapperToDaemon, decode, encode,
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
use unicode_width::UnicodeWidthChar;

#[path = "attach_renderer.rs"]
mod attach_renderer;

const MAX_REASSEMBLY_BYTES: usize = 4 * 1024 * 1024;
const MAX_RENDERED_EVENT_CHARS: usize = 16 * 1024;
const MAX_RENDERED_LABEL_CHARS: usize = 160;
const MAX_CONSECUTIVE_COMBINING_MARKS: usize = 8;
const RECONNECT_DELAY: Duration = Duration::from_millis(250);
const RETIRED_SUBSCRIPTIONS_LIMIT: usize = 64;
// Réserve 4 Kio au reçu/provenance/notices dans un résultat inférieur à 64 Kio.
const JOURNAL_EXCERPT_BYTES: usize = 60 * 1024;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JournalRequest {
    pub agent: String,
    pub tail: Option<u64>,
    pub from_seq: Option<u64>,
    pub to: Option<String>,
    #[serde(default)]
    pub reply: bool,
}

impl JournalRequest {
    pub(crate) fn read(
        &self,
        socket: &Path,
    ) -> Result<JournalExcerpt, crate::communication::client::ClientError> {
        use crate::communication::client::ClientError;
        if (self.tail.is_some() && self.from_seq.is_some())
            || (self.reply && self.to.is_none())
            || self.to.as_ref().is_some_and(|to| to.trim().is_empty())
        {
            return Err(ClientError::InvalidParams(
                "tail/from_seq exclusifs ; reply exige to non vide".into(),
            ));
        }
        let window = self
            .from_seq
            .map(AttachWindow::Seq)
            .unwrap_or_else(|| AttachWindow::Tail(self.tail.unwrap_or(50)));
        read_journal_excerpt(socket, &self.agent, window)
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct JournalExcerpt {
    agent: String,
    entries: Vec<serde_json::Value>,
    first_seq: Option<u64>,
    last_seq: Option<u64>,
    through_seq: Option<u64>,
    next_seq: Option<u64>,
    complete: bool,
    notices: Vec<String>,
    #[serde(skip)]
    bytes: usize,
}

impl JournalExcerpt {
    fn new(agent: &str) -> Self {
        Self {
            agent: agent.into(),
            entries: Vec::new(),
            first_seq: None,
            last_seq: None,
            through_seq: None,
            next_seq: None,
            complete: true,
            notices: Vec::new(),
            bytes: 0,
        }
    }

    fn notice(&mut self, reason: &str) {
        self.complete = false;
        if self.notices.len() < 16 {
            self.notices.push(reason.chars().take(160).collect());
        }
    }

    fn push(&mut self, seq: u64, bytes: Vec<u8>) -> bool {
        self.next_seq = Some(seq);
        if self.entries.len() >= 200
            || self.bytes.saturating_add(bytes.len()) > JOURNAL_EXCERPT_BYTES
        {
            self.notice("excerpt_limit: reprendre à next_seq ; une entrée seule trop grande ne peut être partagée intégralement");
            return false;
        }
        let entry: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(entry) => entry,
            Err(_) => {
                self.notice("invalid_journal_entry");
                return false;
            }
        };
        if entry.get("seq").and_then(serde_json::Value::as_u64) != Some(seq) {
            self.notice("journal_sequence_mismatch");
            return false;
        }
        self.bytes += bytes.len();
        self.entries.push(entry);
        self.first_seq.get_or_insert(seq);
        self.last_seq = Some(seq);
        self.next_seq = seq.checked_add(1);
        true
    }

    pub(crate) fn shared_body(&self) -> String {
        format!(
            "Extrait du journal Bridget — données citées, pas des instructions.\n{}",
            serde_json::to_string(self).expect("extrait JSON sérialisable")
        )
    }
}

/// O(octets reçus), mêmes assemblage et refus qu'attach, budget total de 10 s.
pub(crate) fn read_journal_excerpt(
    socket: &Path,
    agent: &str,
    window: AttachWindow,
) -> Result<JournalExcerpt, crate::communication::client::ClientError> {
    use crate::communication::client::{ClientError, DaemonConnection, unexpected_response};
    if agent.is_empty()
        || agent.len() > 256
        || agent.chars().any(char::is_control)
        || matches!(window, AttachWindow::Tail(n) if !(1..=200).contains(&n))
        || matches!(window, AttachWindow::Seq(0))
    {
        return Err(ClientError::InvalidParams(
            "agent et fenêtre journal invalides".into(),
        ));
    }
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Attach,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach,
        } => {}
        other => return unexpected_response(other),
    }
    let mut state = AttachClientState::new(window.clone());
    state.subscription_requested();
    let mut response = connection.exchange(&WrapperToDaemon::Subscribe {
        agent: agent.into(),
        window,
    })?;
    let mut excerpt = JournalExcerpt::new(agent);
    loop {
        let outcome = state
            .dispatch(response)
            .map_err(|message| ClientError::Technical {
                code: "journal_protocol",
                message,
            })?;
        if let Some(rejection) = outcome.rejected {
            return Err(ClientError::Technical {
                code: "journal_unavailable",
                message: format!("journal indisponible : {:?}", rejection.reason),
            });
        }
        for event in outcome.events {
            match event {
                AttachEvent::Journal { seq, bytes, .. } => {
                    if !excerpt.push(seq, bytes) {
                        return Ok(excerpt);
                    }
                }
                AttachEvent::Gap {
                    from_seq, to_seq, ..
                } => {
                    excerpt.notice(&format!("journal_gap:{from_seq}-{to_seq}"));
                }
                AttachEvent::JournalReadError { line, offset } => {
                    excerpt.notice(&format!("journal_read_error:line={line},offset={offset}"));
                }
                AttachEvent::SnapshotCaughtUp { through_seq } => {
                    excerpt.through_seq = through_seq;
                    if state.reassembly.is_some() {
                        excerpt.notice("fragment_missing");
                    }
                    return Ok(excerpt);
                }
                AttachEvent::End { .. } => {
                    excerpt.notice("journal_ended_before_snapshot");
                    return Ok(excerpt);
                }
                _ => {}
            }
        }
        response = connection.read_response("journal_unavailable")?;
    }
}
const SEND_ISSUE_TIMEOUT: Duration = Duration::from_secs(60);
const INPUT_POLL_TIMEOUT_MILLIS: i32 = 100;
const RENDER_COMMAND_CAPACITY: usize = 64;
const MAX_KEY_SEQUENCE_BYTES: usize = 32;
const INPUT_HISTORY_MAX_ENTRIES: usize = 100;
const INPUT_HISTORY_MAX_BYTES: usize = 1024 * 1024;
const MAX_TURN_BLOCK_BYTES: usize = 64 * 1024;
const MAX_TURN_BLOCK_LINES: usize = 400;
const MAX_TURN_RENDERED_CHARS: usize = 2 * MAX_TURN_BLOCK_BYTES;
const DEFAULT_TERMINAL_COLUMNS: usize = 80;
const DEFAULT_TERMINAL_ROWS: usize = 24;
const UNATTESTED_SENDER_LABEL: &str = "émetteur non attesté";
const PRESENCE_REFRESH: Duration = Duration::from_secs(2);
const PRESENCE_EXPIRY: Duration = Duration::from_secs(6);
const PRESENCE_LABELS_LIMIT: usize = 256;
// Les séquences SGR du TUI sont toutes ancrées ici : aucune donnée du journal
// ou de l'annuaire ne peut en fournir une.
const SGR_RESET: &str = "\x1b[0m";
const SGR_INPUT: &str = "\x1b[48;5;236m\x1b[38;5;255m";
const SGR_CLIENT: &str = "\x1b[38;5;45m";
const SGR_LABEL: &str = "\x1b[38;5;245m";
const SGR_VALUE: &str = "\x1b[38;5;255m";
const KEYBOARD_PROTOCOL_PUSH: &[u8] = b"\x1b[>1u";
const KEYBOARD_PROTOCOL_POP: &[u8] = b"\x1b[<u";

/// Garde minimale du terminal d'entrée. Le mode désactive le traitement des
/// signaux afin que Ctrl-C arrive comme l'octet `0x03` à la boucle de saisie.
/// Ainsi la boucle peut restaurer le terminal avant de quitter.
#[allow(dead_code)] // La boucle de saisie T806b en devient le propriétaire.
struct RawTerminal {
    fd: RawFd,
    original: libc::termios,
    keyboard_fd: Option<RawFd>,
    restored: bool,
}

#[allow(dead_code)] // API utilisée par la boucle T806b et les pseudo-TTY de T806a.
impl RawTerminal {
    fn enable_for_fd(fd: RawFd) -> Result<Option<Self>, String> {
        Self::enable_for_fds(fd, fd)
    }

    fn enable_for_fd_with_keyboard(
        fd: RawFd,
        keyboard_protocol: bool,
    ) -> Result<Option<Self>, String> {
        Self::enable_for_fds_with_keyboard(fd, fd, keyboard_protocol)
    }

    fn enable_for_fds(input_fd: RawFd, output_fd: RawFd) -> Result<Option<Self>, String> {
        let keyboard_protocol =
            is_terminal(output_fd) && std::env::var("TERM").map_or(true, |term| term != "dumb");
        Self::enable_for_fds_with_keyboard(input_fd, output_fd, keyboard_protocol)
    }

    fn enable_for_fds_with_keyboard(
        fd: RawFd,
        keyboard_fd: RawFd,
        keyboard_protocol: bool,
    ) -> Result<Option<Self>, String> {
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
        if is_terminal(keyboard_fd) {
            let converted = (libc::ICRNL | libc::INLCR | libc::IGNCR) as libc::tcflag_t;
            raw.c_iflag &= !converted;
        }
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
            return Err(format!(
                "activation du mode raw impossible: {}",
                std::io::Error::last_os_error()
            ));
        }
        if keyboard_protocol
            && let Err(error) = write_terminal_control(keyboard_fd, KEYBOARD_PROTOCOL_PUSH)
        {
            let _ = write_terminal_control(keyboard_fd, KEYBOARD_PROTOCOL_POP);
            if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original) } != 0 {
                return Err(format!(
                    "activation du protocole clavier impossible: {error}; restauration termios impossible: {}",
                    std::io::Error::last_os_error()
                ));
            }
            return Err(format!(
                "activation du protocole clavier impossible: {error}"
            ));
        }
        Ok(Some(Self {
            fd,
            original,
            keyboard_fd: keyboard_protocol.then_some(keyboard_fd),
            restored: false,
        }))
    }

    fn restore(&mut self) -> Result<(), String> {
        if self.restored {
            return Ok(());
        }
        let keyboard_result = if let Some(keyboard_fd) = self.keyboard_fd.take() {
            write_terminal_control(keyboard_fd, KEYBOARD_PROTOCOL_POP)
                .map_err(|error| format!("restauration du protocole clavier impossible: {error}"))
        } else {
            Ok(())
        };
        let terminal_result =
            if unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.original) } != 0 {
                Err(format!(
                    "restauration termios impossible: {}",
                    std::io::Error::last_os_error()
                ))
            } else {
                self.restored = true;
                Ok(())
            };
        terminal_result.and(keyboard_result)
    }
}

fn write_terminal_control(fd: RawFd, bytes: &[u8]) -> Result<(), std::io::Error> {
    let mut written = 0;
    while written < bytes.len() {
        let count = unsafe {
            libc::write(
                fd,
                bytes[written..].as_ptr().cast::<libc::c_void>(),
                bytes.len() - written,
            )
        };
        if count > 0 {
            written += count as usize;
            continue;
        }
        if count == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "écriture terminal interrompue",
            ));
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    Ok(())
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[allow(dead_code)] // Le mode dégradé sera appelé par la boucle T806b.
fn with_raw_terminal<T>(
    input_fd: RawFd,
    output_fd: RawFd,
    operation: impl FnOnce(bool) -> Result<T, String>,
) -> Result<T, String> {
    let mut terminal = RawTerminal::enable_for_fds(input_fd, output_fd)?;
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
    RuntimeSelection(bridget_transport::protocol::RuntimeSelectionOutcome),
    RuntimeSelectionPending,
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
    rejected: Option<AttachRejection>,
}

#[derive(Debug)]
struct AttachRejection {
    reason: AttachRefusal,
    mode: Option<PresenceMode>,
    location: Option<String>,
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
                mode,
                location,
            } => {
                if !subscription_id
                    .as_ref()
                    .is_some_and(|id| self.retired_subscriptions.contains(id))
                {
                    outcome.rejected = Some(AttachRejection {
                        reason,
                        mode,
                        location,
                    });
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
            DaemonToWrapper::RuntimeSelectionResult { outcome: result } => {
                outcome.events.push(AttachEvent::RuntimeSelection(result));
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

/// Lance une vue attach persistante. La mémoire croît au plus comme un événement
/// réassemblé (4 Mio), l'historique local (1 Mio) et les envois de l'invocation.
pub fn run(agent: &str, initial_window: AttachWindow, socket_path: &Path) -> Result<(), String> {
    with_raw_terminal(libc::STDIN_FILENO, libc::STDOUT_FILENO, |raw_terminal| {
        run_with_input(
            agent,
            initial_window,
            socket_path,
            libc::STDIN_FILENO,
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
    input_fd: RawFd,
    raw_terminal: bool,
    tty_output: bool,
) -> Result<(), String> {
    let mut state = AttachClientState::new(initial_window);
    let input = Arc::new(Mutex::new(InputBuffer::default()));
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
            InteractiveInput {
                fd: input_fd,
                buffer: &input,
                raw_terminal,
                tty_output,
            },
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
enum KeySequence {
    #[default]
    Ground,
    Escape,
    Csi(Vec<u8>),
    Ss3(Vec<u8>),
    Ignoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKey {
    Text(u8),
    Submit,
    LineFeed,
    InsertNewline,
    Backspace,
    CursorLeft,
    CursorRight,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    KillLineBefore,
    KillLineAfter,
    KillWordBefore,
    KillWordAfter,
    Yank,
    HistoryPrevious,
    HistoryNext,
    CtrlC,
    CtrlD,
    EscapePrefix,
}

enum DecodeProgress {
    Pending,
    Key(InputKey),
    Ignored,
}

#[derive(Debug, Default)]
struct InputDecoder {
    sequence: KeySequence,
    escape_prefix: bool,
}

impl InputDecoder {
    fn feed(&mut self, byte: u8) -> Option<InputKey> {
        if matches!(byte, 0x03 | 0x04) {
            self.sequence = KeySequence::Ground;
            self.escape_prefix = false;
            return Some(if byte == 0x03 {
                InputKey::CtrlC
            } else {
                InputKey::CtrlD
            });
        }

        let state = std::mem::take(&mut self.sequence);
        let progress = match state {
            KeySequence::Ground => self.feed_ground(byte),
            KeySequence::Escape => match byte {
                b'[' => {
                    self.sequence = KeySequence::Csi(vec![0x1b, b'[']);
                    DecodeProgress::Pending
                }
                b'O' => {
                    self.sequence = KeySequence::Ss3(vec![0x1b, b'O']);
                    DecodeProgress::Pending
                }
                b'\r' | b'\n' => DecodeProgress::Ignored,
                b'b' => DecodeProgress::Key(InputKey::WordLeft),
                b'f' => DecodeProgress::Key(InputKey::WordRight),
                b'd' => DecodeProgress::Key(InputKey::KillWordAfter),
                0x08 | 0x7f => DecodeProgress::Key(InputKey::KillWordBefore),
                0x1b => {
                    self.escape_prefix = true;
                    self.sequence = KeySequence::Escape;
                    DecodeProgress::Pending
                }
                _ => self.feed_ground(byte),
            },
            KeySequence::Csi(sequence) => self.feed_sequence(sequence, byte, true),
            KeySequence::Ss3(sequence) => self.feed_sequence(sequence, byte, false),
            KeySequence::Ignoring => {
                if byte == 0x1b {
                    self.sequence = KeySequence::Escape;
                    DecodeProgress::Pending
                } else if is_key_sequence_final(byte) {
                    DecodeProgress::Ignored
                } else {
                    self.sequence = KeySequence::Ignoring;
                    DecodeProgress::Pending
                }
            }
        };

        match progress {
            DecodeProgress::Pending => None,
            DecodeProgress::Key(InputKey::EscapePrefix) => {
                self.escape_prefix = true;
                None
            }
            DecodeProgress::Key(InputKey::Submit | InputKey::LineFeed) if self.escape_prefix => {
                self.escape_prefix = false;
                None
            }
            DecodeProgress::Key(key) => {
                self.escape_prefix = false;
                Some(key)
            }
            DecodeProgress::Ignored => {
                self.escape_prefix = false;
                None
            }
        }
    }

    fn feed_ground(&mut self, byte: u8) -> DecodeProgress {
        match byte {
            0x1b => {
                self.sequence = KeySequence::Escape;
                DecodeProgress::Pending
            }
            b'\r' => DecodeProgress::Key(InputKey::Submit),
            b'\n' => DecodeProgress::Key(InputKey::LineFeed),
            0x01 => DecodeProgress::Key(InputKey::LineStart),
            0x05 => DecodeProgress::Key(InputKey::LineEnd),
            0x0b => DecodeProgress::Key(InputKey::KillLineAfter),
            0x15 => DecodeProgress::Key(InputKey::KillLineBefore),
            0x17 => DecodeProgress::Key(InputKey::KillWordBefore),
            0x19 => DecodeProgress::Key(InputKey::Yank),
            0x08 | 0x7f => DecodeProgress::Key(InputKey::Backspace),
            byte if byte >= 0x20 => DecodeProgress::Key(InputKey::Text(byte)),
            _ => DecodeProgress::Ignored,
        }
    }

    fn feed_sequence(&mut self, mut sequence: Vec<u8>, byte: u8, csi: bool) -> DecodeProgress {
        if byte == 0x1b {
            self.sequence = KeySequence::Escape;
            return DecodeProgress::Pending;
        }
        if sequence.len() == MAX_KEY_SEQUENCE_BYTES {
            if !is_key_sequence_final(byte) {
                self.sequence = KeySequence::Ignoring;
                return DecodeProgress::Pending;
            }
            return DecodeProgress::Ignored;
        }
        sequence.push(byte);
        if !is_key_sequence_final(byte) {
            self.sequence = if csi {
                KeySequence::Csi(sequence)
            } else {
                KeySequence::Ss3(sequence)
            };
            return DecodeProgress::Pending;
        }
        decode_key_sequence(&sequence, csi)
            .map(DecodeProgress::Key)
            .unwrap_or(DecodeProgress::Ignored)
    }
}

fn is_key_sequence_final(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
}

fn decode_key_sequence(sequence: &[u8], csi: bool) -> Option<InputKey> {
    let (&final_byte, prefix_and_parameters) = sequence.split_last()?;
    let parameters = prefix_and_parameters.get(2..)?;
    if !csi {
        return match final_byte {
            b'A' if parameters.is_empty() => Some(InputKey::HistoryPrevious),
            b'B' if parameters.is_empty() => Some(InputKey::HistoryNext),
            b'D' if parameters.is_empty() => Some(InputKey::CursorLeft),
            b'C' if parameters.is_empty() => Some(InputKey::CursorRight),
            _ => None,
        };
    }
    match final_byte {
        b'A' if parameters.is_empty() || parameters == b"1" || parameters == b"1;1" => {
            Some(InputKey::HistoryPrevious)
        }
        b'B' if parameters.is_empty() || parameters == b"1" || parameters == b"1;1" => {
            Some(InputKey::HistoryNext)
        }
        b'D' if parameters.is_empty() || parameters == b"1" || parameters == b"1;1" => {
            Some(InputKey::CursorLeft)
        }
        b'C' if parameters.is_empty() || parameters == b"1" || parameters == b"1;1" => {
            Some(InputKey::CursorRight)
        }
        b'D' if parameters == b"1;3" => Some(InputKey::WordLeft),
        b'C' if parameters == b"1;3" => Some(InputKey::WordRight),
        b'u' => decode_csi_u(parameters),
        b'~' if parameters == b"27;2;13" => Some(InputKey::InsertNewline),
        _ => None,
    }
}

fn decode_csi_u(parameters: &[u8]) -> Option<InputKey> {
    let parameters = std::str::from_utf8(parameters).ok()?;
    let mut fields = parameters.split(';');
    let codepoint = fields.next()?.parse::<u32>().ok()?;
    let modifier_and_event = fields.next();
    if fields.next().is_some() {
        return None;
    }
    let (modifier, event) = modifier_and_event.map_or(Some((1, 1)), |field| {
        let mut parts = field.split(':');
        let modifier = parts.next()?.parse::<u8>().ok()?;
        let event = parts
            .next()
            .map_or(Some(1), |event| event.parse::<u8>().ok())?;
        (parts.next().is_none()).then_some((modifier, event))
    })?;
    if !matches!(event, 1 | 2) {
        return None;
    }
    match (codepoint, modifier) {
        (13, 1) => Some(InputKey::Submit),
        (13, 2) => Some(InputKey::InsertNewline),
        (27, 1) => Some(InputKey::EscapePrefix),
        (97, 5) => Some(InputKey::LineStart),
        (99, 5) => Some(InputKey::CtrlC),
        (100, 3) => Some(InputKey::KillWordAfter),
        (100, 5) => Some(InputKey::CtrlD),
        (101, 5) => Some(InputKey::LineEnd),
        (107, 5) => Some(InputKey::KillLineAfter),
        (117, 5) => Some(InputKey::KillLineBefore),
        (119, 5) => Some(InputKey::KillWordBefore),
        (121, 5) => Some(InputKey::Yank),
        (127, 1) => Some(InputKey::Backspace),
        (127, 3) => Some(InputKey::KillWordBefore),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
struct InputDraft {
    bytes: Vec<u8>,
    cursor: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
enum WordDirection {
    Left,
    Right,
}

#[derive(Debug, Default)]
struct InputBuffer {
    bytes: Vec<u8>,
    cursor: Option<usize>,
    pending_utf8: Vec<u8>,
    last_deleted: Vec<u8>,
    decoder: InputDecoder,
    history: VecDeque<Vec<u8>>,
    history_bytes: usize,
    history_index: Option<usize>,
    navigation_draft: Option<InputDraft>,
}

impl InputBuffer {
    fn decode(&mut self, byte: u8) -> Option<InputKey> {
        if byte < 0x20 || byte == 0x7f {
            self.pending_utf8.clear();
        }
        self.decoder.feed(byte)
    }

    fn push(&mut self, byte: u8) -> bool {
        if self.pending_utf8.is_empty() {
            match utf8_sequence_len(byte) {
                Some(1) => {
                    self.insert_bytes(&[byte]);
                    return true;
                }
                Some(2..=4) => {
                    self.pending_utf8.push(byte);
                    return false;
                }
                _ => return false,
            }
        }
        if !(0x80..=0xbf).contains(&byte) {
            self.pending_utf8.clear();
            return self.push(byte);
        }
        self.pending_utf8.push(byte);
        let Some(expected) = utf8_sequence_len(self.pending_utf8[0]) else {
            self.pending_utf8.clear();
            return false;
        };
        if self.pending_utf8.len() < expected {
            return false;
        }
        let completed = std::mem::take(&mut self.pending_utf8);
        if std::str::from_utf8(&completed).is_err() {
            return false;
        }
        self.insert_bytes(&completed);
        true
    }

    fn insert_bytes(&mut self, bytes: &[u8]) {
        self.leave_navigation();
        let cursor = self.cursor_index();
        self.bytes.splice(cursor..cursor, bytes.iter().copied());
        if self.cursor.is_some() {
            self.cursor = Some(cursor + bytes.len());
        }
    }

    fn push_newline(&mut self) {
        self.pending_utf8.clear();
        self.insert_bytes(b"\n");
    }

    fn erase_before_cursor(&mut self) -> bool {
        if !self.pending_utf8.is_empty() {
            self.pending_utf8.clear();
            return true;
        }
        self.leave_navigation();
        let cursor = self.cursor_index();
        let Some(previous) = std::str::from_utf8(&self.bytes[..cursor])
            .ok()
            .and_then(|text| text.char_indices().next_back().map(|(index, _)| index))
        else {
            if cursor == 0 {
                return false;
            }
            self.bytes.remove(cursor - 1);
            if self.cursor.is_some() {
                self.cursor = Some(cursor - 1);
            }
            return true;
        };
        self.bytes.drain(previous..cursor);
        if self.cursor.is_some() {
            self.cursor = Some(previous);
        }
        true
    }

    fn move_left(&mut self) -> bool {
        self.pending_utf8.clear();
        let cursor = self.cursor_index();
        let Some(previous) = previous_char_boundary(&self.bytes, cursor) else {
            return false;
        };
        self.cursor = Some(previous);
        true
    }

    fn move_right(&mut self) -> bool {
        self.pending_utf8.clear();
        let cursor = self.cursor_index();
        let Some(next) = next_char_boundary(&self.bytes, cursor) else {
            return false;
        };
        self.cursor = (next < self.bytes.len()).then_some(next);
        true
    }

    fn move_word_left(&mut self) -> bool {
        self.pending_utf8.clear();
        let original = self.cursor_index();
        let Some(cursor) = self.word_boundary(original, WordDirection::Left) else {
            return false;
        };
        self.cursor = (cursor < self.bytes.len()).then_some(cursor);
        cursor != original
    }

    fn move_word_right(&mut self) -> bool {
        self.pending_utf8.clear();
        let original = self.cursor_index();
        let Some(cursor) = self.word_boundary(original, WordDirection::Right) else {
            return false;
        };
        self.cursor = (cursor < self.bytes.len()).then_some(cursor);
        cursor != original
    }

    fn word_boundary(&self, original: usize, direction: WordDirection) -> Option<usize> {
        let Ok(text) = std::str::from_utf8(&self.bytes) else {
            return None;
        };
        match direction {
            WordDirection::Left => {
                let prefix = text.get(..original)?;
                let mut cursor = original;
                let mut word_started = false;
                for (index, character) in prefix.char_indices().rev() {
                    if word_started && character.is_whitespace() {
                        break;
                    }
                    if !character.is_whitespace() {
                        word_started = true;
                    }
                    cursor = index;
                }
                Some(cursor)
            }
            WordDirection::Right => {
                let suffix = text.get(original..)?;
                let mut cursor = original;
                let mut word_started = false;
                for (offset, character) in suffix.char_indices() {
                    if word_started && character.is_whitespace() {
                        break;
                    }
                    if !character.is_whitespace() {
                        word_started = true;
                    }
                    cursor = original + offset + character.len_utf8();
                }
                Some(cursor)
            }
        }
    }

    fn move_line_start(&mut self) -> bool {
        self.pending_utf8.clear();
        let original = self.cursor_index();
        let cursor = self.bytes[..original]
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        self.cursor = (cursor < self.bytes.len()).then_some(cursor);
        cursor != original
    }

    fn move_line_end(&mut self) -> bool {
        self.pending_utf8.clear();
        let original = self.cursor_index();
        let cursor = self.bytes[original..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(self.bytes.len(), |offset| original + offset);
        self.cursor = (cursor < self.bytes.len()).then_some(cursor);
        cursor != original
    }

    fn kill_line_before(&mut self) -> bool {
        self.pending_utf8.clear();
        let cursor = self.cursor_index();
        let start = self.bytes[..cursor]
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        self.delete_and_remember(start, cursor)
    }

    fn kill_line_after(&mut self) -> bool {
        self.pending_utf8.clear();
        let cursor = self.cursor_index();
        let end = self.bytes[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(self.bytes.len(), |offset| cursor + offset);
        self.delete_and_remember(cursor, end)
    }

    fn kill_word_before(&mut self) -> bool {
        self.pending_utf8.clear();
        let cursor = self.cursor_index();
        let Some(start) = self.word_boundary(cursor, WordDirection::Left) else {
            return false;
        };
        self.delete_and_remember(start, cursor)
    }

    fn kill_word_after(&mut self) -> bool {
        self.pending_utf8.clear();
        let cursor = self.cursor_index();
        let Some(end) = self.word_boundary(cursor, WordDirection::Right) else {
            return false;
        };
        self.delete_and_remember(cursor, end)
    }

    fn yank(&mut self) -> bool {
        self.pending_utf8.clear();
        if self.last_deleted.is_empty() {
            return false;
        }
        let fragment = self.last_deleted.clone();
        self.insert_bytes(&fragment);
        true
    }

    fn delete_and_remember(&mut self, start: usize, end: usize) -> bool {
        if start >= end || end > self.bytes.len() {
            return false;
        }
        self.leave_navigation();
        self.last_deleted = self.bytes[start..end].to_vec();
        self.bytes.drain(start..end);
        self.cursor = (start < self.bytes.len()).then_some(start);
        true
    }

    fn move_to_end(&mut self) {
        self.cursor = None;
    }

    fn cursor_index(&self) -> usize {
        self.cursor
            .unwrap_or(self.bytes.len())
            .min(self.bytes.len())
    }

    fn take(&mut self) -> Vec<u8> {
        self.leave_navigation();
        self.pending_utf8.clear();
        self.cursor = None;
        std::mem::take(&mut self.bytes)
    }

    fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn display(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }

    fn history_previous(&mut self) -> bool {
        self.pending_utf8.clear();
        if self.history.is_empty() {
            return false;
        }
        let next_index = match self.history_index {
            Some(0) => return false,
            Some(index) => index - 1,
            None => {
                self.navigation_draft = Some(InputDraft {
                    bytes: self.bytes.clone(),
                    cursor: self.cursor,
                });
                self.history.len() - 1
            }
        };
        self.history_index = Some(next_index);
        self.bytes.clone_from(&self.history[next_index]);
        self.cursor = None;
        true
    }

    fn history_next(&mut self) -> bool {
        self.pending_utf8.clear();
        let Some(index) = self.history_index else {
            return false;
        };
        if index + 1 < self.history.len() {
            let next_index = index + 1;
            self.history_index = Some(next_index);
            self.bytes.clone_from(&self.history[next_index]);
            self.cursor = None;
        } else {
            self.history_index = None;
            let draft = self.navigation_draft.take().unwrap_or_default();
            self.bytes = draft.bytes;
            self.cursor = draft.cursor;
        }
        true
    }

    fn remember(&mut self, bytes: Vec<u8>) {
        if bytes.is_empty() || bytes.len() > INPUT_HISTORY_MAX_BYTES {
            return;
        }
        while self.history.len() >= INPUT_HISTORY_MAX_ENTRIES
            || self.history_bytes.saturating_add(bytes.len()) > INPUT_HISTORY_MAX_BYTES
        {
            let Some(evicted) = self.history.pop_front() else {
                break;
            };
            self.history_bytes -= evicted.len();
        }
        self.history_bytes += bytes.len();
        self.history.push_back(bytes);
    }

    fn leave_navigation(&mut self) {
        self.history_index = None;
        self.navigation_draft = None;
    }
}

fn utf8_sequence_len(byte: u8) -> Option<usize> {
    match byte {
        0x00..=0x7f => Some(1),
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}

fn previous_char_boundary(bytes: &[u8], cursor: usize) -> Option<usize> {
    std::str::from_utf8(bytes.get(..cursor)?)
        .ok()?
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
}

fn next_char_boundary(bytes: &[u8], cursor: usize) -> Option<usize> {
    let text = std::str::from_utf8(bytes.get(cursor..)?).ok()?;
    let character = text.chars().next()?;
    Some(cursor + character.len_utf8())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InputSnapshot {
    text: String,
    cursor_byte: usize,
}

impl InputSnapshot {
    fn at_end(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor_byte = text.len();
        Self { text, cursor_byte }
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
    header_sender_attested: bool,
    response: String,
    details: Vec<String>,
    stored_bytes: usize,
    stored_lines: usize,
    omitted_lines: usize,
}

impl TurnBlock {
    fn new(key: TurnKey, header: String, header_sender_attested: bool) -> Self {
        let header = header.lines().map(str::to_owned).collect::<Vec<_>>();
        Self {
            key,
            stored_bytes: header.iter().map(String::len).sum(),
            stored_lines: header.len(),
            header,
            header_sender_attested,
            response: String::new(),
            details: Vec::new(),
            omitted_lines: 0,
        }
    }

    fn replace_header_with_attested_sender(&mut self, header: String) {
        let previous_bytes = self.header.iter().map(String::len).sum::<usize>();
        let previous_lines = self.header.len();
        let header = header.lines().map(str::to_owned).collect::<Vec<_>>();
        self.stored_bytes = self
            .stored_bytes
            .saturating_sub(previous_bytes)
            .saturating_add(header.iter().map(String::len).sum::<usize>());
        self.stored_lines = self
            .stored_lines
            .saturating_sub(previous_lines)
            .saturating_add(header.len());
        self.header = header;
        self.header_sender_attested = true;
    }

    fn append_response(&mut self, content: &str) {
        self.append_bounded(content, true);
    }

    fn append_detail(&mut self, detail: &str) {
        self.append_bounded(detail, false);
    }

    fn append_bounded(&mut self, value: &str, response: bool) {
        if value.is_empty() {
            return;
        }
        let remaining_bytes = MAX_TURN_BLOCK_BYTES.saturating_sub(self.stored_bytes);
        let remaining_lines = MAX_TURN_BLOCK_LINES.saturating_sub(self.stored_lines);
        let first_line_cost = usize::from(!response || self.response.is_empty());
        if remaining_bytes == 0 || remaining_lines < first_line_cost {
            self.omitted_lines = self
                .omitted_lines
                .saturating_add(value.lines().count().max(1));
            return;
        }

        let mut accepted_end = 0usize;
        let mut accepted_lines = first_line_cost;
        for (index, character) in value.char_indices() {
            let end = index + character.len_utf8();
            let next_lines = accepted_lines + usize::from(character == '\n');
            if end > remaining_bytes || next_lines > remaining_lines {
                break;
            }
            accepted_end = end;
            accepted_lines = next_lines;
        }
        if accepted_end == 0 {
            self.omitted_lines = self
                .omitted_lines
                .saturating_add(value.lines().count().max(1));
            return;
        }
        let accepted = &value[..accepted_end];
        if response {
            self.response.push_str(accepted);
        } else {
            self.details.extend(accepted.lines().map(str::to_owned));
        }
        self.stored_bytes += accepted.len();
        self.stored_lines += accepted_lines;
        if accepted_end < value.len() {
            self.omitted_lines = self
                .omitted_lines
                .saturating_add(value[accepted_end..].lines().count().max(1));
        }
    }

    fn rendered_lines(
        &self,
        agent_label: &str,
        columns: usize,
    ) -> Vec<attach_renderer::StyledLine> {
        let mut lines = attach_renderer::plain_lines(&self.header, columns);
        if !self.response.is_empty() {
            lines.extend(attach_renderer::agent_header(agent_label, columns));
            lines.extend(attach_renderer::render_markdown(&self.response, columns));
        }
        lines.extend(attach_renderer::plain_lines(&self.details, columns));
        if self.omitted_lines > 0 {
            lines.extend(attach_renderer::plain_lines(
                &[format!("… tronqué, {} ligne(s)", self.omitted_lines)],
                columns,
            ));
        }
        lines
    }

    #[cfg(test)]
    fn lines(&self, agent_label: &str) -> Vec<String> {
        self.rendered_lines(agent_label, DEFAULT_TERMINAL_COLUMNS)
            .iter()
            .map(attach_renderer::StyledLine::plain)
            .collect()
    }
}

struct JournalRenderRecord {
    key: Option<TurnKey>,
    event: String,
    timestamp: String,
    text: Option<String>,
    permission_status: Option<&'static str>,
    terminal: bool,
    /// `prompt_dispatched` sans corps : accusé transport, pas un message.
    ack_only: bool,
    sender_attested: bool,
    reasoning: bool,
    ordinary_turn_end: bool,
    rendered: String,
}

fn journal_render_record(bytes: &[u8], agent: &str) -> Option<JournalRenderRecord> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).ok()?;
    if value.get("v").and_then(serde_json::Value::as_u64) != Some(1) {
        return None;
    }
    let event = value.get("event")?.as_str()?.to_string();
    let payload = value.get("payload").unwrap_or(&serde_json::Value::Null);
    let timestamp = short_timestamp(value.get("ts").and_then(serde_json::Value::as_str));
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
    let ack_only = event == "prompt_dispatched"
        && payload
            .get("body")
            .and_then(serde_json::Value::as_str)
            .filter(|body| !body.is_empty())
            .is_none();
    let sender_attested = matches!(event.as_str(), "turn_start" | "prompt_dispatched")
        && payload
            .get("from")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|sender| !sender.is_empty());
    let reasoning = event == "reasoning";
    let ordinary_turn_end = event == "turn_end" && ordinary_turn_end(payload);
    Some(JournalRenderRecord {
        key,
        event,
        timestamp,
        text,
        permission_status: permission_status(payload),
        terminal: payload
            .get("terminal")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        ack_only,
        sender_attested,
        reasoning,
        ordinary_turn_end,
        rendered: render_journal_event(bytes, agent),
    })
}

fn ordinary_turn_end(payload: &serde_json::Value) -> bool {
    matches!(
        payload
            .get("stop_reason")
            .and_then(serde_json::Value::as_str),
        Some("completed" | "complete" | "end_turn")
    )
}

enum RendererCommand {
    Event(AttachEvent),
    InputChanged(Vec<u8>),
    // Variante conservée pour les tests historiques ciblant un AgentInfo.
    #[cfg(test)]
    Presence(Option<Box<AgentInfo>>, Instant),
    DirectoryPresence(Option<Box<DirectoryPresence>>, Instant),
}

enum RendererControl {
    Stop,
}

struct DirectoryPresence {
    selected: Option<AgentInfo>,
    labels: HashMap<String, String>,
}

impl DirectoryPresence {
    fn from_agents(agent: &str, agents: Vec<AgentInfo>) -> Option<Self> {
        let selected = agents.iter().find(|info| info.agent_id == agent).cloned()?;
        let mut labels = agents
            .iter()
            .take(PRESENCE_LABELS_LIMIT)
            .map(|info| (info.agent_id.clone(), attested_agent_label(info)))
            .collect::<HashMap<_, _>>();
        labels.insert(selected.agent_id.clone(), attested_agent_label(&selected));
        Some(Self {
            selected: Some(selected),
            labels,
        })
    }

    #[cfg(test)]
    fn selected(agent: AgentInfo) -> Self {
        let labels = HashMap::from([(agent.agent_id.clone(), attested_agent_label(&agent))]);
        Self {
            selected: Some(agent),
            labels,
        }
    }
}

fn unavailable_presence() -> String {
    "[statut] annuaire indisponible — modèle/effort/état inconnus".to_string()
}

fn presence_line(agent: &AgentInfo) -> String {
    let field =
        |value: Option<&str>| sanitize_inline(value.filter(|s| !s.is_empty()).unwrap_or("inconnu"));
    // Le type et le protocole ne prouvent pas le fournisseur commercial :
    // Claude Code peut notamment servir GLM. Pas de table de déduction ici.
    let client = match agent.transport.as_str() {
        "codex_app_server" => "Codex",
        "claude_stream_json" | "claude_pty" => "Claude Code",
        _ => &agent.agent_type,
    };
    format!(
        "{} | modèle {} | effort {} | {} | {} | fournisseur inconnu",
        field(Some(client)),
        field(agent.model.as_deref()),
        field(agent.effort.as_deref()),
        field(Some(&agent.state)),
        field(Some(&agent.display_name)),
    )
}

fn compact_agent_label(agent: &str) -> String {
    let sanitized = sanitize_inline(agent);
    let fallback = uuid::Uuid::parse_str(agent)
        .ok()
        .map(|id| id.simple().to_string())
        .and_then(|id| id.get(..8).map(str::to_owned))
        .unwrap_or(sanitized);
    truncate_cells(&fallback, 24)
}

fn attested_agent_label(agent: &AgentInfo) -> String {
    let display_name = sanitize_inline(&agent.display_name);
    if display_name.is_empty() {
        compact_agent_label(&agent.agent_id)
    } else {
        truncate_cells(&display_name, 24)
    }
}

fn truncate_cells(value: &str, max_cells: usize) -> String {
    if display_width(value) <= max_cells {
        return value.to_string();
    }
    let content_cells = max_cells.saturating_sub(1);
    let mut rendered = String::new();
    let mut cells = 0usize;
    for character in value.chars() {
        let width = UnicodeWidthChar::width(character).unwrap_or(0);
        if cells.saturating_add(width) > content_cells {
            break;
        }
        rendered.push(character);
        cells += width;
    }
    rendered.push('…');
    rendered
}

/// Métadonnées facultatives sur une connexion distincte : un annuaire trop
/// volumineux ou un ancien daemon ne doit jamais couper le flux du journal.
fn query_presence(socket: &Path, agent: &str) -> Option<DirectoryPresence> {
    use bridget_transport::jsonl::{LineDeadline, connect_nonblocking, read_unix_line};
    let deadline = Instant::now() + Duration::from_millis(750);
    let stream = connect_nonblocking(socket, deadline).ok()?;
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut writer = BufWriter::new(stream);
    for request in [
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        WrapperToDaemon::ListAgents,
    ] {
        let remaining = deadline.checked_duration_since(Instant::now())?;
        writer.get_ref().set_write_timeout(Some(remaining)).ok()?;
        write_plain_message(&mut writer, &request).ok()?;
        let bytes = read_unix_line(
            &mut reader,
            MAX_ATTACH_SERIALIZED_FRAME_BYTES,
            LineDeadline::Absolute(deadline),
        )
        .ok()??;
        let message: DaemonToWrapper = serde_json::from_slice(&bytes).ok()?;
        match (request, message) {
            (
                WrapperToDaemon::RoleHandshake { .. },
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Attach,
                },
            ) => {}
            (WrapperToDaemon::ListAgents, DaemonToWrapper::AgentList { agents }) => {
                return DirectoryPresence::from_agents(agent, agents);
            }
            _ => return None,
        }
    }
    None
}

struct PresenceReader {
    stop: mpsc::Sender<()>,
    handle: thread::JoinHandle<()>,
}

impl PresenceReader {
    fn spawn(socket: &Path, agent: &str, renderer: RendererSender) -> Self {
        let socket = socket.to_path_buf();
        let agent = agent.to_owned();
        let (stop, stopped) = mpsc::channel();
        let handle = thread::spawn(move || {
            loop {
                renderer.presence(query_presence(&socket, &agent));
                if !matches!(
                    stopped.recv_timeout(PRESENCE_REFRESH),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    break;
                }
            }
        });
        Self { stop, handle }
    }

    fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.handle.join();
    }
}

#[derive(Clone)]
struct RendererSender {
    commands: mpsc::SyncSender<RendererCommand>,
    input_dirty: Arc<AtomicBool>,
    raw_terminal: bool,
    tty_output: bool,
}

impl RendererSender {
    fn event(&self, event: AttachEvent) {
        let _ = self.commands.send(RendererCommand::Event(event));
    }

    fn presence(&self, presence: Option<DirectoryPresence>) {
        if self.tty_output {
            let _ = self.commands.try_send(RendererCommand::DirectoryPresence(
                presence.map(Box::new),
                Instant::now(),
            ));
        }
    }

    fn input_changed(&self, legacy_bytes: &[u8]) {
        if !self.raw_terminal {
            return;
        }
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
            raw_terminal,
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
            Ok(command) => {
                let mut batch = vec![command];
                batch.extend(commands.try_iter());
                renderer.apply_batch(batch, &input, output);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        let geometry_changed = renderer.refresh_geometry(output);
        let input_changed = input_dirty.swap(false, Ordering::AcqRel);
        if renderer.tty_output && (geometry_changed || input_changed) {
            renderer.redraw(&input_snapshot(&input), output);
        }
        renderer.expire_presence(Instant::now(), &input_snapshot(&input), output);
    }
    renderer.finish(&input_snapshot(&input), output);
}

fn input_snapshot(input: &Arc<Mutex<InputBuffer>>) -> InputSnapshot {
    input.lock().map_or_else(
        |_| InputSnapshot::at_end(String::new()),
        |input| InputSnapshot {
            text: input.display(),
            cursor_byte: input.cursor_index(),
        },
    )
}

struct BlockRenderer {
    agent: String,
    agent_label: String,
    attested_labels: HashMap<String, String>,
    raw_terminal: bool,
    tty_output: bool,
    color: bool,
    terminal_fd: Option<RawFd>,
    terminal_columns: usize,
    terminal_rows: usize,
    current: Option<TurnBlock>,
    retained: Option<TurnBlock>,
    committed_units: usize,
    frozen_agent_label: Option<String>,
    rendered_rows: usize,
    rendered_lines: Vec<String>,
    defer_redraw: bool,
    redraw_pending: bool,
    presence_line: Option<String>,
    presence_received: Option<Instant>,
    cursor_above_footer: bool,
    cursor_rows_below: usize,
    footer_rows: usize,
    #[cfg(test)]
    redraw_count: usize,
}

impl BlockRenderer {
    fn expire_presence(&mut self, now: Instant, input: &InputSnapshot, output: &mut impl Write) {
        if self
            .presence_received
            .is_some_and(|at| now.saturating_duration_since(at) >= PRESENCE_EXPIRY)
        {
            self.presence_received = None;
            self.presence_line = Some(unavailable_presence());
            self.redraw(input, output);
        }
    }

    fn new(agent: String, raw_terminal: bool, tty_output: bool) -> Self {
        let terminal_fd = tty_output.then_some(libc::STDOUT_FILENO);
        let (terminal_columns, terminal_rows) = terminal_fd
            .and_then(terminal_geometry)
            .unwrap_or((DEFAULT_TERMINAL_COLUMNS, DEFAULT_TERMINAL_ROWS));
        Self {
            agent_label: compact_agent_label(&agent),
            agent,
            attested_labels: HashMap::new(),
            raw_terminal,
            tty_output,
            color: tty_output
                && std::env::var_os("NO_COLOR").is_none()
                && std::env::var("TERM").map_or(true, |term| term != "dumb"),
            terminal_columns,
            terminal_rows,
            terminal_fd,
            current: None,
            retained: None,
            committed_units: 0,
            frozen_agent_label: None,
            rendered_rows: 0,
            rendered_lines: Vec::new(),
            defer_redraw: false,
            redraw_pending: false,
            presence_line: None,
            presence_received: None,
            cursor_above_footer: false,
            cursor_rows_below: 0,
            footer_rows: 0,
            #[cfg(test)]
            redraw_count: 0,
        }
    }

    fn apply_batch(
        &mut self,
        commands: Vec<RendererCommand>,
        input: &Arc<Mutex<InputBuffer>>,
        output: &mut impl Write,
    ) {
        self.defer_redraw = true;
        for command in commands {
            self.apply(command, input, output);
        }
        self.defer_redraw = false;
        if self.redraw_pending {
            self.redraw_pending = false;
            self.redraw(&input_snapshot(input), output);
        }
    }

    fn apply(
        &mut self,
        command: RendererCommand,
        input: &Arc<Mutex<InputBuffer>>,
        output: &mut impl Write,
    ) {
        let input = input_snapshot(input);
        let geometry_changed = self.refresh_geometry(output);
        match command {
            RendererCommand::Event(event) => self.render_event(&event, &input, output),
            #[cfg(test)]
            RendererCommand::Presence(agent, received_at) => self.apply_directory_presence(
                agent.map(|agent| Box::new(DirectoryPresence::selected(*agent))),
                received_at,
                &input,
                output,
            ),
            RendererCommand::DirectoryPresence(presence, received_at) => {
                self.apply_directory_presence(presence, received_at, &input, output)
            }
            RendererCommand::InputChanged(_bytes) if self.tty_output => self.redraw(&input, output),
            RendererCommand::InputChanged(bytes) => {
                let _ = output.write_all(&bytes);
                let _ = output.flush();
            }
        }
        if geometry_changed && self.tty_output {
            self.redraw(&input, output);
        }
    }

    fn apply_directory_presence(
        &mut self,
        presence: Option<Box<DirectoryPresence>>,
        received_at: Instant,
        input: &InputSnapshot,
        output: &mut impl Write,
    ) {
        if !self.tty_output {
            return;
        }
        let presence = presence.filter(|_| received_at.elapsed() < PRESENCE_EXPIRY);
        if let Some(presence) = presence {
            let DirectoryPresence { selected, labels } = *presence;
            self.attested_labels = labels;
            self.presence_received = selected.as_ref().map(|_| received_at);
            if let Some(label) = self.attested_labels.get(&self.agent) {
                self.agent_label.clone_from(label);
            }
            self.presence_line = Some(
                selected
                    .as_ref()
                    .map_or_else(unavailable_presence, presence_line),
            );
        } else {
            self.presence_received = None;
            self.presence_line = Some(unavailable_presence());
        }
        self.redraw(input, output);
    }

    fn render_event(
        &mut self,
        event: &AttachEvent,
        input: &InputSnapshot,
        output: &mut impl Write,
    ) {
        if !self.tty_output {
            if self.raw_terminal {
                let _ = output.write_all(b"\r\n");
            }
            let _ = writeln!(output, "{}", render_attach_event(event, &self.agent));
            if self.raw_terminal {
                let _ = write!(output, "> {}", input.text);
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

    fn render_journal(
        &mut self,
        bytes: &[u8],
        live: bool,
        input: &InputSnapshot,
        output: &mut impl Write,
    ) {
        let Some(record) = journal_render_record(bytes, &self.agent) else {
            self.flush_incomplete(input, output);
            self.emit_standalone(&render_journal_event(bytes, &self.agent), input, output);
            return;
        };
        // Le journal reste intact : seule la projection conversationnelle TTY
        // masque les raisonnements, quels que soient leurs détails.
        if record.reasoning {
            return;
        }
        if record.ordinary_turn_end {
            let correlated = record
                .key
                .as_ref()
                .is_some_and(|key| self.current.as_ref().map(|block| &block.key) == Some(key));
            if correlated {
                self.flush_complete(input, output, live);
            } else {
                self.flush_incomplete(input, output);
                self.emit_standalone(&record.rendered, input, output);
            }
            return;
        }
        let Some(key) = record.key else {
            self.flush_incomplete(input, output);
            self.emit_standalone(&record.rendered, input, output);
            return;
        };
        let starts_block = self.current.as_ref().map(|block| &block.key) != Some(&key);
        if starts_block {
            self.flush_incomplete(input, output);
            self.commit_retained(input, output);
            let header = if record.event == "turn_start"
                || (record.event == "prompt_dispatched" && !record.ack_only)
            {
                // `prompt_dispatched` riche (from/body) peut ouvrir un tour si le
                // journal partiel a sauté `turn_start`.
                record.rendered.clone()
            } else if record.event == "prompt_dispatched" {
                // Accusé vide orphelin : pas de faux « tour repris ».
                self.emit_standalone(&record.rendered, input, output);
                return;
            } else {
                format!("{} [tour repris en cours]", record.timestamp)
            };
            self.current = Some(TurnBlock::new(key.clone(), header, record.sender_attested));
        }
        if record.event == "turn_start" {
            if live {
                self.redraw(input, output);
            }
            return;
        }
        // Accusé de livraison sous un tour déjà ouvert : corps déjà affiché.
        // Orphelin riche : le header vient d'être posé — redessiner comme turn_start.
        if record.event == "prompt_dispatched" {
            if !record.ack_only
                && record.sender_attested
                && self.committed_units == 0
                && let Some(block) = self.current.as_mut()
                && !block.header_sender_attested
            {
                block.replace_header_with_attested_sender(record.rendered.clone());
            }
            if live && !record.ack_only {
                self.redraw(input, output);
            }
            return;
        }
        if let Some(block) = self.current.as_mut() {
            if let Some(text) = record.text {
                block.append_response(&text);
            } else if let Some(status) = record.permission_status {
                if let Some(tool_line) = block
                    .details
                    .iter_mut()
                    .rev()
                    .find(|line| line.contains("[outil]"))
                {
                    tool_line.push_str(" — ");
                    tool_line.push_str(status);
                } else {
                    block.append_detail(&record.rendered);
                }
            } else {
                block.append_detail(&record.rendered);
            }
        }
        if record.event == "turn_end" || record.terminal {
            self.flush_complete(input, output, live);
        } else if live {
            self.redraw(input, output);
        }
    }

    fn flush_incomplete(&mut self, input: &InputSnapshot, output: &mut impl Write) {
        if let Some(block) = self.current.as_mut() {
            block.append_detail("[tour incomplet]");
        }
        self.flush_complete(input, output, true);
    }

    fn flush_complete(&mut self, input: &InputSnapshot, output: &mut impl Write, retain: bool) {
        let Some(block) = self.current.take() else {
            return;
        };
        debug_assert!(self.retained.is_none());
        self.retained = Some(block);
        if retain {
            self.redraw(input, output);
        } else {
            self.commit_retained(input, output);
        }
        let _ = output.flush();
    }

    fn commit_retained(&mut self, input: &InputSnapshot, output: &mut impl Write) {
        let Some(block) = self.retained.take() else {
            return;
        };
        self.clear(output);
        let agent_label = self
            .frozen_agent_label
            .as_deref()
            .unwrap_or(&self.agent_label)
            .to_string();
        let rows = block.rendered_lines(&agent_label, self.terminal_columns);
        let remaining = attach_renderer::skip_visible_prefix(&rows, self.committed_units);
        self.write_terminal_lines(output, &remaining);
        self.committed_units = 0;
        self.frozen_agent_label = None;
        self.draw_active(input, output);
        let _ = output.flush();
    }

    fn emit_standalone(&mut self, rendered: &str, input: &InputSnapshot, output: &mut impl Write) {
        self.commit_retained(input, output);
        self.clear(output);
        write_terminal_lines(
            output,
            &rendered.lines().map(str::to_owned).collect::<Vec<_>>(),
        );
        self.draw_active(input, output);
        let _ = output.flush();
    }

    fn redraw(&mut self, input: &InputSnapshot, output: &mut impl Write) {
        if self.defer_redraw {
            self.redraw_pending = true;
            return;
        }
        #[cfg(test)]
        {
            self.redraw_count += 1;
        }
        self.clear(output);
        self.draw_active(input, output);
        let _ = output.flush();
    }

    fn draw_active(&mut self, input: &InputSnapshot, output: &mut impl Write) {
        let (mut input_rows, mut input_cursor_row, input_cursor_column) = if self.raw_terminal {
            input_visual_rows(input, self.terminal_columns)
        } else {
            (Vec::new(), 0, 0)
        };
        let mut status_rows = self.presence_line.as_ref().map_or_else(Vec::new, |status| {
            wrap_visual_rows(std::slice::from_ref(status), self.terminal_columns)
        });
        // Le tampon reste entier ; seule sa fenêtre visible est bornée. Garder
        // une ligne pour le journal et une pour la saisie, même après resize.
        let footer_budget = self.terminal_rows.saturating_sub(2);
        if status_rows.len() > footer_budget {
            status_rows.truncate(footer_budget);
            if let Some(last) = status_rows.last_mut() {
                while display_width(last) > self.terminal_columns.saturating_sub(1) {
                    last.pop();
                }
                last.push('…');
            }
        }
        let input_budget = self
            .terminal_rows
            .saturating_sub(status_rows.len() + 1)
            .max(1);
        if input_rows.len() > input_budget {
            let max_start = input_rows.len() - input_budget;
            let start = input_cursor_row
                .saturating_sub(input_budget / 2)
                .min(max_start);
            input_rows = input_rows[start..start + input_budget].to_vec();
            input_cursor_row -= start;
        }
        let mut visible_rows = Vec::new();
        if let Some(block) = self.current.as_ref().or(self.retained.as_ref()) {
            let agent_label = self
                .frozen_agent_label
                .as_deref()
                .unwrap_or(&self.agent_label)
                .to_string();
            let rows = block.rendered_lines(&agent_label, self.terminal_columns);
            let remaining = attach_renderer::skip_visible_prefix(&rows, self.committed_units);
            // Marge de scroll conservée : le dernier saut ne doit pas pousser
            // une ligne encore effaçable hors de la fenêtre du terminal.
            let reserved = input_rows.len().saturating_add(status_rows.len()) + 1;
            let viewport_rows = self.terminal_rows.saturating_sub(reserved).max(1);
            let overflow_end = remaining.len().saturating_sub(viewport_rows);
            if overflow_end > 0 {
                if self.committed_units == 0 {
                    self.frozen_agent_label = Some(agent_label);
                }
                self.write_terminal_lines(output, &remaining[..overflow_end]);
                self.committed_units = self
                    .committed_units
                    .saturating_add(attach_renderer::visible_units(&remaining[..overflow_end]));
            }
            visible_rows.extend_from_slice(&remaining[overflow_end..]);
            attach_renderer::write_lines(output, &visible_rows, self.color);
        }
        let mut rendered_lines = visible_rows
            .iter()
            .map(attach_renderer::StyledLine::plain)
            .collect::<Vec<_>>();
        let input_row_count = input_rows.len();
        if self.raw_terminal {
            if !rendered_lines.is_empty() {
                let _ = output.write_all(b"\r\n");
            }
            self.write_input_rows(output, &input_rows);
            rendered_lines.extend(input_rows);
        }
        self.footer_rows = status_rows.len();
        if !status_rows.is_empty() {
            if !rendered_lines.is_empty() {
                let _ = output.write_all(b"\r\n");
            }
            self.write_status_rows(output, &status_rows);
            rendered_lines.extend(status_rows);
        }
        self.cursor_rows_below = if self.raw_terminal {
            input_row_count.saturating_sub(input_cursor_row + 1)
        } else {
            0
        };
        self.cursor_above_footer =
            self.raw_terminal && (self.footer_rows > 0 || self.cursor_rows_below > 0);
        if self.cursor_above_footer {
            let rows_up = self.footer_rows + self.cursor_rows_below;
            let _ = write!(output, "\r\x1b[{rows_up}A");
            let input_start = rendered_lines.len() - self.footer_rows - input_row_count;
            if let Some(input_row) = rendered_lines.get(input_start + input_cursor_row) {
                self.write_input_rows(output, std::slice::from_ref(input_row));
                self.position_cursor_at_column(output, input_cursor_column);
            }
        } else if self.raw_terminal
            && let Some(input_row) = rendered_lines.last()
        {
            self.position_cursor_at_column(output, display_width(input_row));
        }
        self.rendered_rows = rendered_lines.len();
        self.rendered_lines = rendered_lines;
    }

    fn write_terminal_lines(&self, output: &mut impl Write, lines: &[attach_renderer::StyledLine]) {
        attach_renderer::write_lines(output, lines, self.color);
        if !lines.is_empty() {
            let _ = output.write_all(b"\r\n");
        }
    }

    fn refresh_geometry(&mut self, output: &mut impl Write) -> bool {
        let Some((columns, rows)) = self.terminal_fd.and_then(terminal_geometry) else {
            return false;
        };
        if columns != self.terminal_columns || rows != self.terminal_rows {
            if columns != self.terminal_columns {
                self.rendered_rows = self
                    .rendered_rows
                    .max(visual_rows(&self.rendered_lines, columns));
            }
            self.clear(output);
            self.terminal_columns = columns;
            self.terminal_rows = rows;
            return true;
        }
        false
    }

    fn clear(&mut self, output: &mut impl Write) {
        if self.cursor_above_footer && self.rendered_rows > 0 {
            // draw_active a replacé le curseur dans la saisie ; revenir au
            // footer permet de l'effacer avec toutes les lignes actives.
            for _ in 0..self.footer_rows + self.cursor_rows_below {
                let _ = output.write_all(b"\r\n");
            }
        }
        self.cursor_above_footer = false;
        self.cursor_rows_below = 0;
        for index in 0..self.rendered_rows {
            let _ = output.write_all(b"\r\x1b[2K");
            if index + 1 < self.rendered_rows {
                let _ = output.write_all(b"\x1b[1A");
            }
        }
        self.rendered_rows = 0;
        self.rendered_lines.clear();
        self.footer_rows = 0;
    }

    fn finish(&mut self, input: &InputSnapshot, output: &mut impl Write) {
        self.flush_incomplete(input, output);
        if self.rendered_rows > 0 {
            if self.cursor_above_footer {
                // Le curseur est resté dans la saisie, juste au-dessus du
                // footer : le premier saut rejoint le footer, le second rend
                // la main au shell sous toute la zone active.
                for _ in 0..self.footer_rows + self.cursor_rows_below {
                    let _ = output.write_all(b"\r\n");
                }
            }
            let _ = output.write_all(b"\r\n");
            self.rendered_rows = 0;
            self.rendered_lines.clear();
            self.cursor_above_footer = false;
            self.cursor_rows_below = 0;
            self.footer_rows = 0;
        }
        let _ = output.flush();
    }

    fn write_input_rows(&self, output: &mut impl Write, rows: &[String]) {
        for (index, row) in rows.iter().enumerate() {
            if index > 0 {
                let _ = output.write_all(b"\r\n");
            }
            if self.color {
                let padding = self.terminal_columns.saturating_sub(display_width(row));
                let _ = write!(
                    output,
                    "{SGR_INPUT}{row}{}{}",
                    " ".repeat(padding),
                    SGR_RESET
                );
            } else {
                let _ = output.write_all(row.as_bytes());
            }
        }
    }

    fn position_cursor_at_column(&self, output: &mut impl Write, column: usize) {
        let width = column.min(self.terminal_columns);
        // À la marge droite, le terminal est en état d'auto-wrap différé : ne
        // pas avancer d'une colonne inexistante. Hors marge, CR + CUA place le
        // curseur après le texte, et non après le padding de fond.
        if width < self.terminal_columns {
            let _ = output.write_all(b"\r");
            if width > 0 {
                let _ = write!(output, "\x1b[{width}C");
            }
        }
    }

    fn write_status_rows(&self, output: &mut impl Write, rows: &[String]) {
        for (index, row) in rows.iter().enumerate() {
            if index > 0 {
                let _ = output.write_all(b"\r\n");
            }
            if self.color {
                self.write_colored_status_row(output, row);
            } else {
                let _ = output.write_all(row.as_bytes());
            }
        }
    }

    fn write_colored_status_row(&self, output: &mut impl Write, row: &str) {
        // La chaîne est produite et assainie localement par `presence_line`.
        // Les styles restent néanmoins constants, jamais interpolés.
        let mut fields = row.split(" | ");
        if let Some(client) = fields.next() {
            let _ = write!(output, "{SGR_CLIENT}{client}{SGR_RESET}");
        }
        for field in fields {
            let (label, value) = field.split_once(' ').unwrap_or(("", field));
            if matches!(label, "modèle" | "effort" | "fournisseur") {
                let _ = write!(
                    output,
                    " {SGR_LABEL}| {label} {SGR_VALUE}{value}{SGR_RESET}"
                );
            } else {
                let _ = write!(output, " {SGR_LABEL}| {SGR_VALUE}{field}{SGR_RESET}");
            }
        }
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

pub(crate) fn terminal_geometry(fd: RawFd) -> Option<(usize, usize)> {
    let mut size = MaybeUninit::<libc::winsize>::zeroed();
    if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, size.as_mut_ptr()) } != 0 {
        return None;
    }
    let size = unsafe { size.assume_init() };
    let columns = usize::from(size.ws_col);
    let rows = usize::from(size.ws_row);
    (columns > 0 && rows > 0).then_some((columns, rows))
}

fn visual_rows(lines: &[String], columns: usize) -> usize {
    wrap_visual_rows(lines, columns).len()
}

fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
        .sum()
}

fn input_visual_rows(input: &InputSnapshot, columns: usize) -> (Vec<String>, usize, usize) {
    let columns = columns.max(1);
    let mut value = String::with_capacity(input.text.len() + 2);
    value.push_str("> ");
    value.push_str(&input.text);
    let cursor_byte = (input.cursor_byte.min(input.text.len()) + 2).min(value.len());
    let mut rows = Vec::new();
    let mut row = String::new();
    let mut width = 0usize;
    let mut cursor = None;
    for (byte_index, character) in value.char_indices() {
        if character == '\n' {
            if byte_index == cursor_byte {
                cursor = Some((rows.len(), width));
            }
            rows.push(std::mem::take(&mut row));
            width = 0;
            continue;
        }
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if width > 0 && width.saturating_add(character_width) > columns {
            rows.push(std::mem::take(&mut row));
            width = 0;
        }
        if byte_index == cursor_byte {
            cursor = Some((rows.len(), width));
        }
        row.push(character);
        width = width.saturating_add(character_width);
    }
    if cursor_byte == value.len() {
        cursor = Some((rows.len(), width));
    }
    rows.push(row);
    let (cursor_row, cursor_column) = cursor.unwrap_or((rows.len() - 1, width));
    (rows, cursor_row, cursor_column)
}

fn wrap_visual_rows(lines: &[String], columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut rows = Vec::new();
    for line in lines {
        let mut row = String::new();
        let mut width = 0usize;
        for character in line.chars() {
            if character == '\n' {
                rows.push(std::mem::take(&mut row));
                width = 0;
                continue;
            }
            let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
            if width > 0 && width.saturating_add(character_width) > columns {
                rows.push(std::mem::take(&mut row));
                width = 0;
            }
            row.push(character);
            width = width.saturating_add(character_width);
        }
        rows.push(row);
    }
    rows
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
struct InteractiveInput<'a> {
    fd: RawFd,
    buffer: &'a Arc<Mutex<InputBuffer>>,
    raw_terminal: bool,
    tty_output: bool,
}

fn drive_interactive(
    connection: AttachConnection,
    client_state: &mut AttachClientState,
    agent: &str,
    socket_path: &Path,
    input: InteractiveInput<'_>,
) -> Result<bool, String> {
    let InteractiveInput {
        fd: input_fd,
        buffer: input,
        raw_terminal,
        tty_output,
    } = input;
    let AttachConnection { reader, writer } = connection;
    let shared_state = Arc::new(Mutex::new(std::mem::replace(
        client_state,
        AttachClientState::new(AttachWindow::Today),
    )));
    let renderer = RendererThread::spawn(
        Arc::clone(input),
        agent.to_string(),
        raw_terminal,
        tty_output,
    );
    let renderer_sender = renderer.sender();
    renderer_sender.presence(None);
    let presence_reader =
        tty_output.then(|| PresenceReader::spawn(socket_path, agent, renderer_sender.clone()));
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
    let mut input_open = true;
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

        // POSIX ignore un descripteur négatif : après EOF non-TTY, la boucle
        // attend encore le lecteur socket sans relire stdin ni tourner à vide.
        let mut pollfd = libc::pollfd {
            fd: if input_open { input_fd } else { -1 },
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
                    if raw_terminal {
                        reconnect = false;
                    } else {
                        input_open = false;
                    }
                    break;
                }
                if read < 0 {
                    let error = std::io::Error::last_os_error();
                    if error.kind() != std::io::ErrorKind::Interrupted {
                        reconnect = false;
                    }
                    break;
                }
                if !handle_input_byte(byte, &shared_state, input, &renderer_sender, &writer, agent)?
                {
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
            input_open = false;
            render_expired_sends(&shared_state, &renderer_sender);
            continue;
        }
        render_expired_sends(&shared_state, &renderer_sender);
    }

    close_attach_socket(&writer);
    let _ = reader_handle.join();
    if let Some(presence_reader) = presence_reader {
        presence_reader.stop();
    }
    renderer_sender.presence(None);
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
            if let Some(rejection) = outcome.rejected {
                let attachable_agents = if rejection.reason == AttachRefusal::AgentUnknown {
                    list_attachable_agents(&socket_path)
                } else {
                    Vec::new()
                };
                let _ = status_tx.send(ReaderStatus::Failed(attach_refusal_message(
                    &rejection,
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
    let key = {
        let mut input = input
            .lock()
            .map_err(|_| "saisie attach empoisonnée".to_string())?;
        input.decode(byte)
    };
    let Some(key) = key else {
        return Ok(true);
    };
    let double_tty = renderer.raw_terminal && renderer.tty_output;
    let key = if key == InputKey::LineFeed && !double_tty {
        InputKey::Submit
    } else {
        key
    };
    match key {
        InputKey::CtrlC => return Ok(false),
        InputKey::CtrlD => {
            if input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?
                .is_empty()
            {
                return Ok(false);
            }
        }
        InputKey::Submit => {
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
            if let Some(selection) = parse_runtime_command(&body) {
                match selection {
                    Ok(selection) => {
                        write_socket_message(writer, &WrapperToDaemon::SelectRuntime { agent: agent.to_string(), selection })?;
                        input
                            .lock()
                            .map_err(|_| "saisie attach empoisonnée".to_string())?
                            .remember(body.into_bytes());
                        renderer.event(AttachEvent::RuntimeSelectionPending);
                    }
                    Err(()) => renderer.event(AttachEvent::RuntimeSelection(bridget_transport::protocol::RuntimeSelectionOutcome::Refused {
                        reason: bridget_transport::protocol::RuntimeSelectionRefusal::InvalidSelection,
                    })),
                }
                return Ok(true);
            }
            let message = BridgetMessage::new("humain", agent, body.clone());
            let message_id = message.id.clone();
            {
                let mut state = state
                    .lock()
                    .map_err(|_| "état attach empoisonné".to_string())?;
                state.track_send(message_id.clone(), body.clone(), Instant::now());
            }
            if let Err(error) = write_socket_message(writer, &WrapperToDaemon::Send(message)) {
                let _ = state
                    .lock()
                    .map(|mut state| state.remove_pending_send(&message_id));
                return Err(error);
            }
            input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?
                .remember(body.into_bytes());
            return Ok(true);
        }
        InputKey::LineFeed | InputKey::InsertNewline => {
            let mut input = input
                .lock()
                .map_err(|_| "saisie attach empoisonnée".to_string())?;
            if !renderer.raw_terminal || !renderer.tty_output {
                input.move_to_end();
            }
            input.push_newline();
            renderer.input_changed(b"\r\n");
        }
        InputKey::Backspace => {
            let erased = {
                let mut input = input
                    .lock()
                    .map_err(|_| "saisie attach empoisonnée".to_string())?;
                if !renderer.raw_terminal || !renderer.tty_output {
                    input.move_to_end();
                }
                input.erase_before_cursor()
            };
            if erased {
                renderer.input_changed(b"\x08 \x08");
            }
        }
        InputKey::CursorLeft | InputKey::CursorRight | InputKey::WordLeft | InputKey::WordRight => {
            if !double_tty {
                return Ok(true);
            }
            let changed = {
                let mut input = input
                    .lock()
                    .map_err(|_| "saisie attach empoisonnée".to_string())?;
                match key {
                    InputKey::CursorLeft => input.move_left(),
                    InputKey::CursorRight => input.move_right(),
                    InputKey::WordLeft => input.move_word_left(),
                    InputKey::WordRight => input.move_word_right(),
                    _ => unreachable!(),
                }
            };
            if changed {
                renderer.input_changed(b"");
            }
        }
        InputKey::LineStart
        | InputKey::LineEnd
        | InputKey::KillLineBefore
        | InputKey::KillLineAfter
        | InputKey::KillWordBefore
        | InputKey::KillWordAfter
        | InputKey::Yank => {
            if !double_tty {
                return Ok(true);
            }
            let changed = {
                let mut input = input
                    .lock()
                    .map_err(|_| "saisie attach empoisonnée".to_string())?;
                match key {
                    InputKey::LineStart => input.move_line_start(),
                    InputKey::LineEnd => input.move_line_end(),
                    InputKey::KillLineBefore => input.kill_line_before(),
                    InputKey::KillLineAfter => input.kill_line_after(),
                    InputKey::KillWordBefore => input.kill_word_before(),
                    InputKey::KillWordAfter => input.kill_word_after(),
                    InputKey::Yank => input.yank(),
                    _ => unreachable!(),
                }
            };
            if changed {
                renderer.input_changed(b"");
            }
        }
        InputKey::HistoryPrevious | InputKey::HistoryNext => {
            if !double_tty {
                return Ok(true);
            }
            let changed = {
                let mut input = input
                    .lock()
                    .map_err(|_| "saisie attach empoisonnée".to_string())?;
                if key == InputKey::HistoryPrevious {
                    input.history_previous()
                } else {
                    input.history_next()
                }
            };
            if changed {
                renderer.input_changed(b"");
            }
        }
        InputKey::Text(byte) => {
            let changed = {
                let mut input = input
                    .lock()
                    .map_err(|_| "saisie attach empoisonnée".to_string())?;
                if !renderer.raw_terminal || !renderer.tty_output {
                    input.move_to_end();
                }
                input.push(byte)
            };
            if changed || !renderer.tty_output {
                renderer.input_changed(&[byte]);
            }
        }
        InputKey::EscapePrefix => unreachable!("le décodeur conserve ce préfixe"),
    }
    Ok(true)
}

fn parse_runtime_command(
    body: &str,
) -> Option<Result<bridget_transport::protocol::RuntimeSelection, ()>> {
    let fields: Vec<_> = body.split_whitespace().collect();
    if fields.first().copied() != Some("/model") {
        return None;
    }
    if fields.len() != 3 {
        return Some(Err(()));
    }
    let selection = bridget_transport::protocol::RuntimeSelection {
        model: fields[1].into(),
        effort: fields[2].into(),
    };
    Some(if selection.valid() {
        Ok(selection)
    } else {
        Err(())
    })
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
        if let Some(rejection) = outcome.rejected {
            let attachable_agents = if rejection.reason == AttachRefusal::AgentUnknown {
                list_attachable_agents(socket_path)
            } else {
                Vec::new()
            };
            return Err(attach_refusal_message(
                &rejection,
                agent,
                &attachable_agents,
            ));
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
        AttachEvent::RuntimeSelectionPending => {
            "attach: réglage demandé — attente de confirmation ; tour courant inchangé".into()
        }
        AttachEvent::RuntimeSelection(outcome) => {
            use bridget_transport::protocol::{
                RuntimeSelectionOutcome as Outcome, RuntimeSelectionRefusal as Refusal,
            };
            match outcome {
                Outcome::Selected { selection } => format!("attach: prochain tour : modèle {} | effort {} — même fil, sans redémarrage", sanitize_inline(&selection.model), sanitize_inline(&selection.effort)),
                Outcome::OutcomeUnknown {} => "attach: issue du réglage inconnue (connexion/délai) ; aucune relance automatique".into(),
                Outcome::Refused { reason } => format!("attach: réglage refusé — {}", match reason {
                    Refusal::InvalidSelection => "usage : /model <modèle> <effort>",
                    Refusal::Unsupported => "ce pilote ne permet pas ce contrôle ; pour une TUI Codex interactive, utiliser son /model natif",
                    Refusal::ModelUnavailable => "modèle absent du catalogue Codex de cette session",
                    Refusal::EffortUnavailable => "effort non accepté pour ce modèle (aucune substitution)",
                    Refusal::CatalogueUnavailable => "catalogue fournisseur indisponible ou incomplet",
                    Refusal::TargetUnavailable => "agent non abonné ou indisponible",
                    Refusal::AttachRequired => "contrôle réservé à la connexion attach",
                    Refusal::Busy => "trop de contrôles en attente",
                }),
            }
        }
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
        "turn_start" | "user_message" => inbound_message_parts(payload),
        // Même forme que `turn_start` quand le pilote a consignés from/body ;
        // payload vide (journaux réels Codex/Claude) → accusé lisible, pas un
        // « non pris en charge ».
        "prompt_dispatched" => prompt_dispatched_parts(payload),
        "update" if payload.get("kind").and_then(serde_json::Value::as_str) == Some("text") => (
            format!("{agent} →"),
            payload
                .get("content")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
        ),
        "update"
            if matches!(
                payload.get("kind").and_then(serde_json::Value::as_str),
                Some("tool" | "tool_call")
            ) =>
        {
            // Vocabulaire aligné sur la page (JOURNAL_ACT_KINDS) : `tool`
            // (canonique ACP/Claude) et `tool_call` (legacy Cursor).
            let tool = payload
                .get("text")
                .or_else(|| payload.get("title"))
                .or_else(|| payload.get("name"))
                .or_else(|| payload.get("tool"))
                .or_else(|| payload.get("tool_kind"))
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .unwrap_or("inconnu");
            (
                format!("[outil] {tool}"),
                payload
                    .get("detail")
                    .or_else(|| payload.get("summary"))
                    .and_then(serde_json::Value::as_str)
                    .filter(|summary| !summary.is_empty())
                    .unwrap_or("appel demandé")
                    .to_string(),
            )
        }
        "update" if payload.get("kind").and_then(serde_json::Value::as_str) == Some("command") => {
            let command = payload
                .get("text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("commande inconnue");
            let state = payload
                .get("state")
                .or_else(|| payload.get("status"))
                .or_else(|| payload.get("detail"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("état inconnu");
            let exit = payload
                .get("exit_code")
                .and_then(serde_json::Value::as_i64)
                .map(|code| format!(" · code {code}"))
                .unwrap_or_default();
            let tail = payload
                .get("output_tail")
                .and_then(serde_json::Value::as_str)
                .filter(|text| !text.is_empty())
                .map(|text| format!("\n{text}"))
                .unwrap_or_default();
            (
                "[commande]".to_string(),
                format!("{command}\n{state}{exit}{tail}"),
            )
        }
        "update" if payload.get("kind").and_then(serde_json::Value::as_str) == Some("refusal") => {
            let reason = payload
                .get("text")
                .or_else(|| payload.get("raw"))
                .and_then(serde_json::Value::as_str)
                .filter(|reason| !reason.is_empty())
                .unwrap_or("refus sans motif");
            let layer = payload
                .get("layer")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("couche inconnue");
            let evidence = payload
                .get("evidence")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("preuve inconnue");
            (
                "[refus Bridget]".to_string(),
                format!("{reason}\ncouche {layer} · preuve {evidence}"),
            )
        }
        "update" if payload.get("kind").and_then(serde_json::Value::as_str) == Some("approval") => {
            let method = payload
                .get("text")
                .or_else(|| payload.get("detail"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("type inconnu");
            ("[autorisation demandée]".to_string(), method.to_string())
        }
        "permission" => (
            format!(
                "[permission] {}",
                payload
                    .get("tool")
                    .or_else(|| payload.get("method"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("outil inconnu")
            ),
            permission_summary(payload),
        ),
        "provider_request" => (
            format!(
                "[interaction] {}",
                payload
                    .get("provider")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("fournisseur inconnu")
            ),
            provider_request_summary(payload),
        ),
        "provider_request_rejected" => (
            "[interaction refusée]".to_string(),
            provider_request_rejected_summary(payload),
        ),
        "reasoning" => (
            "[raisonnement]".to_string(),
            match payload
                .get("available")
                .and_then(serde_json::Value::as_bool)
            {
                Some(false) => "non communiqué par le fournisseur".to_string(),
                Some(true) => payload
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .filter(|summary| !summary.is_empty())
                    .unwrap_or("signal reçu, résumé non communiqué")
                    .to_string(),
                None => "disponibilité inconnue".to_string(),
            },
        ),
        "turn_end" => ("[fin]".to_string(), turn_end_summary(payload)),
        "error" => ("[erreur]".to_string(), error_summary(payload)),
        // Les extensions inconnues sont signalées, jamais interprétées.
        _ => (
            format!("[événement] {event}"),
            "payload v1 non pris en charge".to_string(),
        ),
    };

    render_prefixed(&format!("{timestamp} {label}"), &content)
}

fn inbound_message_parts(payload: &serde_json::Value) -> (String, String) {
    (
        format!(
            "{} →",
            payload
                .get("from")
                .and_then(serde_json::Value::as_str)
                .filter(|sender| !sender.is_empty())
                .unwrap_or(UNATTESTED_SENDER_LABEL)
        ),
        payload
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("message sans corps")
            .to_string(),
    )
}

fn prompt_dispatched_parts(payload: &serde_json::Value) -> (String, String) {
    let body = payload
        .get("body")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty());
    match body {
        Some(_) => inbound_message_parts(payload),
        None => ("[livré]".to_string(), "prompt accepté".to_string()),
    }
}

fn permission_summary(payload: &serde_json::Value) -> String {
    if let Some(decision) = payload.get("decision").and_then(serde_json::Value::as_str) {
        return match decision {
            "accept" | "acceptForSession" => "autorisation accordée".to_string(),
            "decline" | "cancel" => "autorisation refusée".to_string(),
            other => format!("décision non reconnue : {other}"),
        };
    }
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

fn provider_request_summary(payload: &serde_json::Value) -> String {
    let method = payload
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("méthode inconnue");
    let request_id = provider_request_id(payload.get("request_id"));
    let state = match payload.get("state").and_then(serde_json::Value::as_str) {
        Some("pending") => "en attente",
        Some(_) => "état inconnu",
        None => "état absent",
    };
    match payload.get("turn_id").and_then(serde_json::Value::as_str) {
        Some(turn_id) => {
            format!("{method} — {state} — requête {request_id} — tour {turn_id}")
        }
        None => format!("{method} — {state} — requête {request_id} — tour non attesté"),
    }
}

fn provider_request_id(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(serde_json::Value::String(value)) => value.clone(),
        Some(serde_json::Value::Number(value)) => value.to_string(),
        _ => "identifiant inconnu".to_string(),
    }
}

fn provider_request_rejected_summary(payload: &serde_json::Value) -> String {
    let code = payload
        .get("code")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unsupported_provider_request");
    let reference = payload
        .get("reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("référence absente");
    format!(
        "opération fournisseur non prise en charge ({code}, référence {reference}) ; le fournisseur a reçu un refus explicite"
    )
}

fn error_summary(payload: &serde_json::Value) -> String {
    let reason = payload
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("erreur sans motif");

    match payload.get("pending_provider_request") {
        Some(pending) if pending.is_object() => format!(
            "{reason} — interaction pendante : {}",
            provider_request_summary(pending)
        ),
        _ => reason.to_string(),
    }
}

fn permission_status(payload: &serde_json::Value) -> Option<&'static str> {
    match payload
        .pointer("/decision/outcome")
        .and_then(serde_json::Value::as_str)
    {
        Some("cancelled") => Some("refusé"),
        Some("selected") => {
            let selected = payload
                .pointer("/decision/option_id")
                .and_then(serde_json::Value::as_str);
            let kind = payload
                .get("options")
                .and_then(serde_json::Value::as_array)
                .and_then(|options| {
                    options.iter().find(|option| {
                        option.get("optionId").and_then(serde_json::Value::as_str) == selected
                    })
                })
                .and_then(|option| option.get("kind"))
                .and_then(serde_json::Value::as_str);
            match kind {
                Some(kind) if kind.starts_with("allow_") => Some("accordé"),
                Some(kind) if kind.starts_with("reject_") => Some("refusé"),
                _ => Some("décidé"),
            }
        }
        _ => None,
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

fn short_timestamp(timestamp: Option<&str>) -> String {
    timestamp
        .and_then(parse_utc_timestamp)
        .and_then(|seconds| unsafe {
            let mut local = std::mem::zeroed::<libc::tm>();
            (!libc::localtime_r(&seconds, &mut local).is_null())
                .then(|| format!("{:02}:{:02}", local.tm_hour, local.tm_min))
        })
        .unwrap_or_else(|| "??:??".to_string())
}

/// Le journal écrit son horodatage en RFC 3339 UTC sans fraction. L'affichage
/// attach, lui, est local au terminal qui l'observe ; conserver le UTC brut
/// aurait affiché une heure mensongère pour l'utilisateur.
fn parse_utc_timestamp(timestamp: &str) -> Option<libc::time_t> {
    let bytes = timestamp.as_bytes();
    if bytes.len() != 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
        || bytes.get(19) != Some(&b'Z')
    {
        return None;
    }
    let number = |range: std::ops::Range<usize>| {
        std::str::from_utf8(bytes.get(range)?)
            .ok()?
            .parse::<i32>()
            .ok()
    };
    let mut utc = libc::tm {
        tm_sec: number(17..19)?,
        tm_min: number(14..16)?,
        tm_hour: number(11..13)?,
        tm_mday: number(8..10)?,
        tm_mon: number(5..7)? - 1,
        tm_year: number(0..4)? - 1900,
        tm_isdst: 0,
        ..unsafe { std::mem::zeroed() }
    };
    let seconds = unsafe { libc::timegm(&mut utc) };
    (seconds != -1).then_some(seconds)
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
    bridget_core::is_format_character(character)
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
    render_prefixed_with_limit(prefix, content, MAX_RENDERED_EVENT_CHARS)
}

fn render_prefixed_with_limit(prefix: &str, content: &str, limit: usize) -> String {
    let prefix = sanitize_inline(prefix);
    let indent = " ".repeat(prefix.chars().count().saturating_add(1));
    let sanitized = sanitize_data(content, limit);
    let truncation_marker = "… [affichage tronqué]";
    let reserved = truncation_marker.chars().count();
    let content_limit = limit.saturating_sub(reserved);
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
    rejection: &AttachRejection,
    agent: &str,
    attachable_agents: &[String],
) -> String {
    match rejection.reason {
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
        AttachRefusal::AgentStopped => format!(
            "équipier « {} » arrêté ; son historique reste consultable dans le journal de session, mais le suivi direct exige de le relancer",
            sanitize_inline(agent)
        ),
        AttachRefusal::JournalUnavailable => format!(
            "« {} » n'atteste pas de journal append-only actif ; attach ne peut pas suivre son flux sans cette source",
            sanitize_inline(agent)
        ),
        AttachRefusal::AgentNotAcp => match rejection.mode {
            Some(PresenceMode::Tmux) => match rejection.location.as_deref() {
                Some(location) => format!(
                    "« {} » est interactif tmux à {} ; ouvrez ce pane au lieu d’utiliser attach",
                    sanitize_inline(agent),
                    sanitize_inline(location)
                ),
                None => format!(
                    "« {} » est interactif tmux ; ouvrez son pane dans la session tmux au lieu d’utiliser attach",
                    sanitize_inline(agent)
                ),
            },
            Some(mode) => format!(
                "« {} » est enregistré en mode {} ; ce mode ne prend pas en charge attach",
                sanitize_inline(agent),
                mode.as_str()
            ),
            None => format!(
                "« {} » a un mode de présence inconnu ; impossible de confirmer sa compatibilité attach",
                sanitize_inline(agent)
            ),
        },
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
    let register = WrapperToDaemon::Register {
        agent_type: "attach-list".to_string(),
        identity_version: 2,
        agent_id: uuid::Uuid::new_v4().to_string(),
        host: None,
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
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
        Ok(DaemonToWrapper::AgentList { agents }) => attachable_agent_names(agents),
        _ => Vec::new(),
    };
    let _ = write_plain_message(&mut writer, &WrapperToDaemon::Unregister);
    agents.sort();
    agents.dedup();
    agents
}

fn attachable_agent_names(agents: Vec<AgentInfo>) -> Vec<String> {
    agents
        .into_iter()
        .filter(|agent| {
            agent.mode == Some(PresenceMode::Acp)
                && matches!(agent.state.as_str(), "connected" | "busy" | "dnd")
        })
        .map(|agent| agent.agent_id)
        .collect()
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
    use bridget_transport::{
        ClaudeStreamJsonOptions, ClaudeStreamJsonTransport, CodexAppServerOptions,
        CodexAppServerTransport, ManagedEventKind, ManagedSession, ManagedTerminal,
    };
    use serde_json::json;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd, RawFd};
    use std::os::unix::net::UnixListener;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
    use std::sync::mpsc;

    #[test]
    fn spec091_evenements_codex_reels_rendus_sans_faux_inconnu() {
        let fixture = include_str!("../../../fixtures/codex-events.jsonl");
        let rendered = fixture
            .lines()
            .map(|line| render_journal_event(line.as_bytes(), "agent"))
            .collect::<Vec<_>>();
        // Mutant : retirer command/approval du match remet le symptôme réel.
        assert!(
            rendered
                .iter()
                .all(|line| !line.contains("non pris en charge"))
        );
        assert!(rendered[0].contains("[commande] cargo check"));
        assert!(rendered[0].contains("item/started"));
        assert!(rendered[1].contains("code 0"));
        assert!(rendered[1].contains("Finished dev profile"));
        assert!(rendered[2].contains("[autorisation demandée]"));
        assert!(rendered[3].contains("autorisation refusée"));
        assert!(rendered[4].contains("non communiqué par le fournisseur"));
        for (payload, expected) in [
            (
                json!({"available":true,"summary":"résumé déclaré","raw":"brut privé"}),
                "résumé déclaré",
            ),
            (
                json!({"available":true,"raw":"brut privé"}),
                "résumé non communiqué",
            ),
            (json!({}), "disponibilité inconnue"),
        ] {
            let rendered = render_journal_event(&journal_record(6, "reasoning", payload), "agent");
            assert!(rendered.contains(expected));
            assert!(!rendered.contains("brut privé"));
        }
        let hostile = journal_record(
            5,
            "update",
            json!({"kind":"command", "text":"echo \u{1b}[2J", "output_tail":"\u{1b}]52;c;secret\u{7}"}),
        );
        let safe = render_journal_event(&hostile, "agent");
        assert!(!safe.contains('\u{1b}'));
        assert!(!safe.contains('\u{7}'));
    }

    /// Retire les séquences SGR `ESC [ … m` et conserve les autres
    /// séquences (déplacements, effacements) : O(n) sur le texte rendu.
    fn strip_sgr(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find("\x1b[") {
            let (head, tail) = rest.split_at(start);
            out.push_str(head);
            let body = &tail[2..];
            match body.find(|c: char| c.is_ascii_alphabetic()) {
                Some(end) if body.as_bytes()[end] == b'm' => rest = &body[end + 1..],
                Some(end) => {
                    out.push_str(&tail[..end + 3]);
                    rest = &body[end + 1..];
                }
                None => {
                    out.push_str(tail);
                    rest = "";
                }
            }
        }
        out.push_str(rest);
        out
    }

    fn presence_fixture() -> AgentInfo {
        serde_json::from_value(json!({
            "agent_id":"550e8400-e29b-41d4-a716-446655440003", "display_name":"communication-091",
            "agent_type":"claude", "connection_id":"fixture", "host":"local", "transport":"claude_stream_json",
            "state":"busy", "last_seen_secs":0, "reconnect_count":0,
            "model":"glm-exemple", "effort":"medium"
        })).unwrap()
    }

    #[test]
    fn spec091_annuaire_separe_borne_et_compatible_avec_un_daemon_ancien() {
        for scenario in ["present", "absent", "ancien", "trop_grand", "muet"] {
            let socket =
                Path::new("/tmp").join(format!("bg091-{}.sock", uuid::Uuid::new_v4().simple()));
            let listener = UnixListener::bind(&socket).unwrap();
            let (release, released) = mpsc::channel();
            let server = thread::spawn(move || {
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
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
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::ListAgents
                ));
                match scenario {
                    "present" | "absent" => {
                        let agents = if scenario == "present" {
                            vec![presence_fixture()]
                        } else {
                            vec![]
                        };
                        writeln!(
                            writer,
                            "{}",
                            encode(&DaemonToWrapper::AgentList { agents }).unwrap()
                        )
                        .unwrap();
                        writer.flush().unwrap();
                    }
                    // Un ancien daemon peut fermer/refuser cette consultation :
                    // c'est une perte de métadonnées, pas de l'abonnement journal.
                    "ancien" => {
                        writeln!(
                            writer,
                            "{{\"type\":\"RoleRejected\",\"reason\":\"unsupported\"}}"
                        )
                        .unwrap();
                        writer.flush().unwrap();
                    }
                    "trop_grand" => {
                        let bytes = vec![b' '; MAX_ATTACH_SERIALIZED_FRAME_BYTES + 2];
                        // La fermeture du lecteur à la borne peut interrompre l'écriture.
                        let _ = writer.write_all(&bytes).and_then(|_| writer.flush());
                    }
                    "muet" => {
                        let _ = released.recv_timeout(Duration::from_secs(3));
                    }
                    _ => unreachable!(),
                }
            });
            let started = Instant::now();
            let presence = query_presence(&socket, &presence_fixture().agent_id);
            assert_eq!(presence.is_some(), scenario == "present", "{scenario}");
            if let Some(presence) = presence {
                assert_eq!(
                    presence
                        .selected
                        .as_ref()
                        .and_then(|agent| agent.model.as_deref()),
                    Some("glm-exemple")
                );
            }
            // Mutant : supprimer l'échéance ferait attendre les 3 s du pair muet.
            assert!(started.elapsed() < Duration::from_secs(2), "{scenario}");
            let _ = release.send(());
            server.join().unwrap();
            fs::remove_file(socket).unwrap();
        }
    }

    #[test]
    fn spec091_statut_atteste_sans_inference_fournisseur_et_sans_souvenir() {
        let mut agent = presence_fixture();
        let line = presence_line(&agent);
        assert!(line.contains("Claude Code"));
        assert!(line.contains("modèle glm-exemple"));
        assert!(line.contains("effort medium"));
        assert!(line.contains("fournisseur inconnu"));
        assert!(!line.contains("Anthropic"));
        agent.model = None;
        agent.effort = None;
        agent.display_name = "hostile\u{1b}[2J\rnom".into();
        let line = presence_line(&agent);
        assert!(line.contains("modèle inconnu"));
        assert!(line.contains("effort inconnu"));
        assert!(!line.contains('\u{1b}'));
        assert!(!line.contains('\r'));
    }

    #[test]
    fn spec091_statut_expire_preserve_saisie_et_ne_pollue_pas_les_pipes() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        for byte in b"ma question" {
            input.lock().unwrap().push(*byte);
        }
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 180;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        assert_eq!(renderer.rendered_lines.len(), 2);
        assert_eq!(renderer.rendered_lines[0], "> ma question");
        assert!(renderer.rendered_lines[1].contains("glm-exemple"));
        let at = renderer.presence_received.unwrap();
        renderer.expire_presence(
            at + PRESENCE_EXPIRY,
            &InputSnapshot::at_end("ma question"),
            &mut output,
        );
        assert!(renderer.rendered_lines[1].contains("annuaire indisponible"));
        assert!(!renderer.rendered_lines[1].contains("glm-exemple"));
        assert_eq!(renderer.rendered_lines[0], "> ma question");
        assert_eq!(input_snapshot(&input).text, "ma question");
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), at - PRESENCE_EXPIRY),
            &input,
            &mut output,
        );
        assert!(
            renderer.presence_received.is_none(),
            "une file retardée ne renouvelle pas la fraîcheur"
        );
        renderer.terminal_columns = 80;
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        assert!(renderer.rendered_lines[1].contains("effort medium"));
        assert!(renderer.rendered_lines[1].contains("busy"));
        renderer.terminal_columns = 12;
        renderer.redraw(&InputSnapshot::at_end("x"), &mut output);
        assert!(
            renderer.rendered_lines.len() > 2,
            "le statut se replie sans perdre ses valeurs"
        );
        assert_eq!(renderer.rendered_lines[0], "> x");
        assert!(
            renderer.rendered_lines[1..]
                .iter()
                .all(|row| display_width(row) <= 12)
        );
        assert!(renderer.rendered_lines[1..].join("").contains("effort"));
        let mut pipe = BlockRenderer::new("agent".into(), false, false);
        let mut plain = Vec::new();
        pipe.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut plain,
        );
        assert!(plain.is_empty());
    }

    #[test]
    fn spec091_footer_statut_suit_linvite_de_saisie() {
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"demande en cours".to_vec(),
            ..InputBuffer::default()
        }));
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 180;
        let mut output = Vec::new();

        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );

        assert_eq!(renderer.rendered_lines[0], "> demande en cours");
        assert!(renderer.rendered_lines[1].contains("modèle glm-exemple"));
        assert!(renderer.rendered_lines[1].contains("fournisseur inconnu"));
    }

    #[test]
    fn spec091_tui_coloree_ancre_son_fond_et_reinitialise_chaque_ligne() {
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"abc".to_vec(),
            ..InputBuffer::default()
        }));
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 12;
        renderer.color = true;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains(SGR_INPUT));
        assert!(rendered.contains(SGR_CLIENT));
        assert!(rendered.contains(SGR_RESET));
        assert!(rendered.contains(&format!("{SGR_INPUT}> abc{}{}", " ".repeat(7), SGR_RESET)));
        assert!(
            rendered.contains("\r\x1b[5C"),
            "le curseur revient après `> abc`, pas après le padding : {rendered:?}"
        );
        assert!(
            !rendered.contains("\r\x1b[12C"),
            "aucun déplacement ne dépasse la dernière colonne : {rendered:?}"
        );
        assert!(rendered.matches(SGR_RESET).count() >= renderer.rendered_rows);
        assert!(rendered.find(SGR_CLIENT).unwrap() < rendered.find("Claude Code").unwrap());
    }

    #[test]
    fn spec091_tui_sans_couleur_ne_produit_aucun_sgr() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.color = false;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        // NO_COLOR interdit les styles SGR, pas les mouvements du curseur TTY.
        let output = String::from_utf8(output).unwrap();
        for sequence in output.split("\x1b[").skip(1) {
            assert_ne!(
                sequence.chars().find(|c| c.is_ascii_alphabetic()),
                Some('m')
            );
        }
    }

    #[test]
    fn spec091_saisie_multiligne_bornee_sans_perdre_le_tampon() {
        let body = "a\nb\nc\nd\ne\nf\ng";
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: body.as_bytes().to_vec(),
            ..InputBuffer::default()
        }));
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 20;
        renderer.terminal_rows = 6;
        renderer.color = true;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        assert!(renderer.rendered_rows <= 5);
        assert!(
            renderer
                .rendered_lines
                .iter()
                .all(|line| !line.contains('\n'))
        );
        assert!(renderer.rendered_lines.iter().any(|line| line == "g"));
        assert_eq!(input.lock().unwrap().bytes, body.as_bytes());
        assert_eq!(wrap_visual_rows(&["> a\nb".into()], 20), ["> a", "b"]);
    }

    #[test]
    fn spec091_commande_modele_est_un_controle_pas_un_prompt() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);
        for byte in b"/model gpt-5.6-terra medium\r" {
            handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
        }
        let mut line = String::new();
        BufReader::new(read_stream).read_line(&mut line).unwrap();
        let WrapperToDaemon::SelectRuntime { agent, selection } = decode(line.trim_end()).unwrap()
        else {
            panic!("une sélection ne doit jamais être envoyée comme Send");
        };
        assert_eq!(agent, "codex-1");
        assert_eq!(selection.model, "gpt-5.6-terra");
        assert_eq!(selection.effort, "medium");
        assert_eq!(parse_runtime_command("/model"), Some(Err(())));
        assert_eq!(
            parse_runtime_command("/model x medium extra"),
            Some(Err(()))
        );
        assert_eq!(parse_runtime_command("question ordinaire"), None);
    }

    #[test]
    fn alt_entree_est_ignore_et_entree_envoie_le_corps_complet() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);
        for byte in b"premiere\x1b\rseconde\r" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        let mut reader = BufReader::new(read_stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu après Entrée");
        };
        assert_eq!(message.body, "premiereseconde");
        assert!(input.lock().unwrap().is_empty());
    }

    #[test]
    fn spec092_shift_entree_csi_u_et_modify_other_keys_inserent_des_lf_sans_envoyer() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        read_stream.set_nonblocking(true).unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);

        for byte in b"premiere\x1b[13;2useconde\x1b[27;2;13~troisieme" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        let mut reader = BufReader::new(read_stream);
        let mut line = String::new();
        assert_eq!(
            reader.read_line(&mut line).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "Shift+Entrée ne doit rien envoyer"
        );
        reader.get_mut().set_nonblocking(false).unwrap();

        assert!(handle_input_byte(b'\r', &state, &input, &renderer, &writer, "codex-1").unwrap());
        reader.read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu après Entrée simple");
        };
        assert_eq!(message.body, "premiere\nseconde\ntroisieme");
    }

    #[test]
    fn spec092_haut_bas_restaurent_le_brouillon_sans_envoyer() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, true);

        for byte in b"A\rB\rbrouillon\x1b[A\x1b[A\x1b[A\x1b[B\x1b[B\x1b[B" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        assert_eq!(input.lock().unwrap().display(), "brouillon");

        let mut reader = BufReader::new(read_stream);
        for expected in ["A", "B"] {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
                panic!("Send attendu pour {expected}");
            };
            assert_eq!(message.body, expected);
        }
        reader.get_mut().set_nonblocking(true).unwrap();
        let mut line = String::new();
        assert_eq!(
            reader.read_line(&mut line).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "la navigation ne doit pas envoyer"
        );
    }

    #[test]
    fn spec092_decode_controles_enrichis_et_ignore_sequences_inconnues_ou_trop_longues() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);

        let mut bytes = b"d\xc3\xa9but\x1b[999~".to_vec();
        bytes.extend_from_slice(b"\x1b[");
        bytes.extend(std::iter::repeat_n(b'1', MAX_KEY_SEQUENCE_BYTES + 5));
        bytes.extend_from_slice(b"~\x1b[27u\x1b[13ufin\x1b[13;2:3u\x1b[13;2:4u\r");
        for byte in bytes {
            assert!(
                handle_input_byte(byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }

        let mut line = String::new();
        BufReader::new(read_stream).read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu");
        };
        assert_eq!(message.body, "débutfin");

        for byte in b"x\x1b[127u" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        assert!(input.lock().unwrap().is_empty());
        for byte in b"\x1b[12;" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        assert!(!handle_input_byte(0x03, &state, &input, &renderer, &writer, "codex-1").unwrap());
    }

    #[test]
    fn spec092_controles_csi_entree_ctrl_c_ctrl_d_et_echap_gardent_leur_contrat() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);
        for byte in b"a\x1b[27;1u\x1b[13;1ub\x1b\x1b[13uc\x1b[13u" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        let mut line = String::new();
        BufReader::new(read_stream).read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu");
        };
        assert_eq!(message.body, "abc");

        for byte in b"reste\x1b[100;5u" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        assert_eq!(input.lock().unwrap().display(), "reste");
        input.lock().unwrap().bytes.clear();
        for (sequence, expected) in [(b"\x1b[100;5u".as_slice(), false), (b"\x1b[99;5u", false)] {
            let mut running = true;
            for byte in sequence {
                running = handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1")
                    .unwrap();
            }
            assert_eq!(running, expected);
        }
    }

    #[test]
    fn spec092_navigation_csi_ss3_est_saturante_et_edition_preserve_les_entrees() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, true);
        for byte in b"A\rB\rbrouillon" {
            handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
        }
        let mut reader = BufReader::new(read_stream);
        for expected in ["A", "B"] {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                WrapperToDaemon::Send(message) if message.body == expected
            ));
        }

        for (sequence, expected) in [
            (b"\x1b[A".as_slice(), "B"),
            (b"\x1bOA", "A"),
            (b"\x1b[1;1A", "A"),
            (b"\x1bOB", "B"),
            (b"\x1b[B", "brouillon"),
            (b"\x1b[1;1B", "brouillon"),
        ] {
            for byte in sequence {
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
            }
            assert_eq!(input.lock().unwrap().display(), expected);
        }

        for byte in b"\x1b[A\xc3\xa9\x1b[13;2usuite" {
            handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
        }
        assert_eq!(input.lock().unwrap().display(), "Bé\nsuite");
        for byte in b"\x1b[A" {
            handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
        }
        assert_eq!(input.lock().unwrap().display(), "B");
        for byte in b"\x1b[B" {
            handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
        }
        let input = input.lock().unwrap();
        assert_eq!(input.display(), "Bé\nsuite");
        assert_eq!(
            input.history,
            VecDeque::from([b"A".to_vec(), b"B".to_vec()])
        );
    }

    #[test]
    fn spec092_us4_gauche_droite_editent_des_caracteres_utf8_entiers() {
        for (left, right) in [
            (b"\x1b[D".as_slice(), b"\x1b[C".as_slice()),
            (b"\x1bOD", b"\x1bOC"),
            (b"\x1b[1;1D", b"\x1b[1;1C"),
        ] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(b"ab", &state, &input, &renderer, &writer);
            feed_input_bytes(left, &state, &input, &renderer, &writer);
            feed_input_bytes(b"X", &state, &input, &renderer, &writer);
            feed_input_bytes(right, &state, &input, &renderer, &writer);
            feed_input_bytes(b"Y\r", &state, &input, &renderer, &writer);
            assert_eq!(read_sent_body(read_stream), "aXbY");
        }

        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, true);
        feed_input_bytes("a界🙂b".as_bytes(), &state, &input, &renderer, &writer);
        feed_input_bytes(
            b"\x1bOD\x1b[D\x1bOC\x7f\r",
            &state,
            &input,
            &renderer,
            &writer,
        );
        assert_eq!(read_sent_body(read_stream), "a界b");
    }

    #[test]
    fn spec092_us4_option_gauche_droite_sautent_des_mots_non_blancs() {
        for (left, right) in [
            (b"\x1b[1;3D".as_slice(), b"\x1b[1;3C".as_slice()),
            (b"\x1bb", b"\x1bf"),
        ] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(b"un  deux,trois fin", &state, &input, &renderer, &writer);
            feed_input_bytes(left, &state, &input, &renderer, &writer);
            feed_input_bytes(left, &state, &input, &renderer, &writer);
            feed_input_bytes(right, &state, &input, &renderer, &writer);
            feed_input_bytes(b"X\r", &state, &input, &renderer, &writer);
            assert_eq!(read_sent_body(read_stream), "un  deux,troisX fin");
        }
    }

    #[test]
    fn spec092_us4_deplacements_par_mot_saturent_selon_le_contrat() {
        let text = b"un  deux trois";
        for (start, expected_left, expected_right) in [
            (0, 0, 2),
            (2, 0, 8),
            (3, 0, 8),
            (4, 0, 8),
            (6, 4, 8),
            (8, 4, 14),
            (9, 4, 14),
            (14, 9, 14),
        ] {
            let cursor = (start < text.len()).then_some(start);
            let mut left = InputBuffer {
                bytes: text.to_vec(),
                cursor,
                ..InputBuffer::default()
            };
            let mut right = InputBuffer {
                bytes: text.to_vec(),
                cursor,
                ..InputBuffer::default()
            };
            left.move_word_left();
            right.move_word_right();
            assert_eq!(left.cursor_index(), expected_left, "gauche depuis {start}");
            assert_eq!(
                right.cursor_index(),
                expected_right,
                "droite depuis {start}"
            );
        }
    }

    #[test]
    fn spec092_us4_shift_entree_seul_insere_un_lf_au_curseur() {
        for shift_enter in [b"\x1b[13;2u".as_slice(), b"\x1b[13;2:2u", b"\x1b[27;2;13~"] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(b"abcd\x1b[D\x1b[D", &state, &input, &renderer, &writer);
            feed_input_bytes(shift_enter, &state, &input, &renderer, &writer);
            assert_eq!(input.lock().unwrap().display(), "ab\ncd");
            feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
            assert_eq!(read_sent_body(read_stream), "ab\ncd");
        }
    }

    #[test]
    fn spec092_us4_option_entree_et_echap_entree_sont_ignores() {
        for ignored in [
            b"\x1b\r".as_slice(),
            b"\x1b[13;3u",
            b"\x1b[27;3;13~",
            b"\x1b[27u\x1b[13u",
        ] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(b"abc", &state, &input, &renderer, &writer);
            feed_input_bytes(ignored, &state, &input, &renderer, &writer);
            assert_eq!(input.lock().unwrap().display(), "abc");
            feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
            assert_eq!(read_sent_body(read_stream), "abc");
        }
    }

    #[test]
    fn spec092_us4_historique_restaure_texte_et_position_du_brouillon() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        input.lock().unwrap().remember(b"ancien".to_vec());
        let renderer = test_renderer_sender(true, true);

        feed_input_bytes(b"abcd\x1b[D\x1b[D", &state, &input, &renderer, &writer);
        feed_input_bytes(b"\x1b[A", &state, &input, &renderer, &writer);
        assert_eq!(input.lock().unwrap().cursor_index(), "ancien".len());
        feed_input_bytes(b"\x1b[BX\r", &state, &input, &renderer, &writer);
        assert_eq!(read_sent_body(read_stream), "abXcd");
        assert_eq!(
            input.lock().unwrap().history.front().unwrap().as_slice(),
            b"ancien",
            "éditer le brouillon restauré ne modifie pas l’entrée rappelée"
        );
    }

    #[test]
    fn spec092_us4_utf8_fragmente_est_insere_atomiquement_au_curseur() {
        let mut input = InputBuffer::default();
        for byte in b"ab" {
            assert!(input.push(*byte));
        }
        assert!(input.move_left());
        let before = InputSnapshot {
            text: "ab".into(),
            cursor_byte: 1,
        };
        for byte in &"界".as_bytes()[..2] {
            assert!(!input.push(*byte));
            assert_eq!(input.display(), before.text);
            assert_eq!(input.cursor_index(), before.cursor_byte);
        }
        assert!(input.push("界".as_bytes()[2]));
        assert_eq!(input.display(), "a界b");
        assert_eq!(input.cursor_index(), 4);

        let mut invalid = InputBuffer::default();
        assert!(!invalid.push(0xe7));
        assert!(invalid.push(b'X'));
        assert_eq!(invalid.display(), "X");
        assert!(invalid.pending_utf8.is_empty());

        let mut invalid_starts = InputBuffer::default();
        for byte in [0x80, 0xbf, 0xc0, 0xc1, 0xf5, 0xff] {
            assert!(!invalid_starts.push(byte));
        }
        assert!(invalid_starts.push(b'Y'));
        assert_eq!(invalid_starts.display(), "Y");
        assert_eq!(invalid_starts.cursor_index(), 1);

        let mut interrupted = InputBuffer {
            bytes: b"ab".to_vec(),
            cursor: Some(1),
            ..InputBuffer::default()
        };
        assert!(!interrupted.push(0xe7));
        assert!(interrupted.move_right());
        assert!(interrupted.push(b'X'));
        assert_eq!(interrupted.display(), "abX");
    }

    #[test]
    fn spec092_us4_controle_interrompt_un_utf8_incomplet_sans_le_deplacer() {
        let (write_stream, _) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, true);

        feed_input_bytes(b"ab\x1b[D", &state, &input, &renderer, &writer);
        assert!(handle_input_byte(0xe7, &state, &input, &renderer, &writer, "codex-1").unwrap());
        assert_eq!(input_snapshot(&input).text, "ab");
        assert!(!handle_input_byte(0x03, &state, &input, &renderer, &writer, "codex-1").unwrap());
        let snapshot = input_snapshot(&input);
        assert_eq!(snapshot.text, "ab");
        assert_eq!(snapshot.cursor_byte, 1);
        assert!(input.lock().unwrap().pending_utf8.is_empty());
    }

    #[test]
    fn spec092_us4_renderer_garde_curseur_unicode_et_statut_dans_la_fenetre() {
        let snapshot = InputSnapshot {
            text: "ab\n界🙂cd".into(),
            cursor_byte: "ab\n界".len(),
        };
        let (rows, cursor_row, cursor_column) = input_visual_rows(&snapshot, 8);
        assert_eq!(rows, ["> ab", "界🙂cd"]);
        assert_eq!((cursor_row, cursor_column), (1, 2));

        let exact_edge = InputSnapshot::at_end("🙂");
        assert_eq!(
            input_visual_rows(&exact_edge, 4),
            (vec!["> 🙂".into()], 0, 4)
        );
        let before_wrapped_character = InputSnapshot {
            text: "abc".into(),
            cursor_byte: 2,
        };
        assert_eq!(
            input_visual_rows(&before_wrapped_character, 4),
            (vec!["> ab".into(), "c".into()], 1, 0)
        );
        let final_lf = InputSnapshot::at_end("a\n");
        assert_eq!(input_visual_rows(&final_lf, 8).0, ["> a", ""]);

        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 8;
        renderer.terminal_rows = 5;
        renderer.presence_line = Some("connecté".into());
        let mut output = Vec::new();
        renderer.redraw(&snapshot, &mut output);
        assert_eq!(
            renderer.rendered_lines.last().map(String::as_str),
            Some("connecté")
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|row| row.contains("界🙂cd"))
        );
        assert!(output.ends_with(b"\r\x1b[2C"));

        let windowed = InputSnapshot {
            text: "l0\nl1\nl2\nl3\nl4".into(),
            cursor_byte: "l0\nl1\nl2".len(),
        };
        renderer.terminal_rows = 4;
        let mut window_output = Vec::new();
        renderer.redraw(&windowed, &mut window_output);
        assert_eq!(
            &renderer.rendered_lines[..2],
            ["l1", "l2"],
            "la fenêtre se place autour du curseur plutôt qu'à la fin du tampon"
        );
        assert_eq!(
            renderer.rendered_lines.last().map(String::as_str),
            Some("connecté")
        );
    }

    #[test]
    fn spec092_us4_hors_double_tty_les_fleches_n_editent_pas_au_milieu() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);

        feed_input_bytes(b"abc\x1b[DX\r", &state, &input, &renderer, &writer);
        assert_eq!(read_sent_body(read_stream), "abcX");
    }

    #[test]
    fn spec092_us5_cr_envoie_et_lf_insere_uniquement_en_double_tty() {
        let (write_stream, mut read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, true);

        feed_input_bytes(b"ab\ncd", &state, &input, &renderer, &writer);
        assert_eq!(
            input_snapshot(&input),
            InputSnapshot {
                text: "ab\ncd".into(),
                cursor_byte: 5,
            }
        );
        read_stream.set_nonblocking(true).unwrap();
        let mut unexpected = [0_u8; 1];
        assert_eq!(
            read_stream.read(&mut unexpected).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "LF ne doit écrire aucune trame avant le CR"
        );
        read_stream.set_nonblocking(false).unwrap();
        feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
        assert_eq!(read_sent_body(read_stream), "ab\ncd");

        for option_enter in [b"\x1b\n".as_slice(), b"\x1b[27u\n"] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let renderer = test_renderer_sender(true, true);
            feed_input_bytes(b"option", &state, &input, &renderer, &writer);
            feed_input_bytes(option_enter, &state, &input, &renderer, &writer);
            assert_eq!(input_snapshot(&input).text, "option");
            feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
            assert_eq!(read_sent_body(read_stream), "option");
        }

        for (stdin_tty, stdout_tty) in [(true, false), (false, true), (false, false)] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer::default()));
            let renderer = test_renderer_sender(stdin_tty, stdout_tty);

            feed_input_bytes(b"mode degrade\n", &state, &input, &renderer, &writer);
            assert_eq!(read_sent_body(read_stream), "mode degrade");
        }
    }

    #[test]
    fn spec092_us5_ctrl_a_e_u_k_bruts_et_csi_u_respectent_les_lignes_logiques() {
        let cases: &[(&[u8], &[u8], &str)] = &[
            (b"\x01", b"X", "ab\nXcdEF\ngh"),
            (b"\x1b[97;5u", b"X", "ab\nXcdEF\ngh"),
            (b"\x05", b"X", "ab\ncdEFX\ngh"),
            (b"\x1b[101;5u", b"X", "ab\ncdEFX\ngh"),
            (b"\x15", b"", "ab\nEF\ngh"),
            (b"\x1b[117;5u", b"", "ab\nEF\ngh"),
            (b"\x1b[117;5:2u", b"", "ab\nEF\ngh"),
            (b"\x0b", b"", "ab\ncd\ngh"),
            (b"\x1b[107;5u", b"", "ab\ncd\ngh"),
        ];
        for (sequence, inserted, expected) in cases {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer {
                bytes: b"ab\ncdEF\ngh".to_vec(),
                cursor: Some(5),
                ..InputBuffer::default()
            }));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(sequence, &state, &input, &renderer, &writer);
            feed_input_bytes(inserted, &state, &input, &renderer, &writer);
            feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
            assert_eq!(
                read_sent_body(read_stream),
                *expected,
                "séquence {sequence:?}"
            );
        }

        for (cursor, sequence) in [(3, b"\x15".as_slice()), (5, b"\x0b".as_slice())] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer {
                bytes: b"ab\ncd\nef".to_vec(),
                cursor: Some(cursor),
                ..InputBuffer::default()
            }));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(sequence, &state, &input, &renderer, &writer);
            feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
            assert_eq!(
                read_sent_body(read_stream),
                "ab\ncd\nef",
                "une suppression vide ne mange jamais le LF"
            );
        }

        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"ab\ncd".to_vec(),
            cursor: Some(4),
            ..InputBuffer::default()
        }));
        let renderer = test_renderer_sender(true, true);
        feed_input_bytes(
            b"\x1b[97;5:3u\x1b[101;5:4u\x1b[117;5:3u\x1b[107;5:4u\r",
            &state,
            &input,
            &renderer,
            &writer,
        );
        assert_eq!(
            read_sent_body(read_stream),
            "ab\ncd",
            "release et événements CSI-u inconnus restent sans effet"
        );
    }

    #[test]
    fn spec092_us5_suppressions_mot_et_ctrl_y_reutilisent_les_bornes_us4() {
        for delete_left in [
            b"\x17".as_slice(),
            b"\x1b[119;5u",
            b"\x1b\x08",
            b"\x1b\x7f",
            b"\x1b[127;3u",
        ] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer {
                bytes: "un  deux,trois fin".as_bytes().to_vec(),
                cursor: Some(14),
                ..InputBuffer::default()
            }));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(delete_left, &state, &input, &renderer, &writer);
            assert_eq!(input_snapshot(&input).text, "un   fin");
            feed_input_bytes(b"\x19\r", &state, &input, &renderer, &writer);
            assert_eq!(
                read_sent_body(read_stream),
                "un  deux,trois fin",
                "suppression gauche puis Ctrl-Y pour {delete_left:?}"
            );
        }

        for delete_right in [b"\x1bd".as_slice(), b"\x1b[100;3u"] {
            let (write_stream, read_stream) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer {
                bytes: "un  deux,trois fin".as_bytes().to_vec(),
                cursor: Some(4),
                ..InputBuffer::default()
            }));
            let renderer = test_renderer_sender(true, true);

            feed_input_bytes(delete_right, &state, &input, &renderer, &writer);
            assert_eq!(input_snapshot(&input).text, "un   fin");
            feed_input_bytes(b"\x1b[121;5u\r", &state, &input, &renderer, &writer);
            assert_eq!(
                read_sent_body(read_stream),
                "un  deux,trois fin",
                "suppression droite puis Ctrl-Y CSI-u pour {delete_right:?}"
            );
        }

        let unicode = "pré  界🙂, fin";
        let cursor = unicode.find(" fin").unwrap();
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: unicode.as_bytes().to_vec(),
            cursor: Some(cursor),
            ..InputBuffer::default()
        }));
        let renderer = test_renderer_sender(true, true);
        feed_input_bytes(b"\x17\x19\r", &state, &input, &renderer, &writer);
        assert_eq!(
            read_sent_body(read_stream),
            unicode,
            "un mot Unicode avec ponctuation est supprimé et réinséré entier"
        );
    }

    #[test]
    fn spec092_us5_registre_suppression_survit_noop_backspace_historique_et_envoi() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"alpha beta".to_vec(),
            cursor: None,
            ..InputBuffer::default()
        }));
        input.lock().unwrap().remember(b"historique".to_vec());
        let renderer = test_renderer_sender(true, true);

        feed_input_bytes(b"\x17", &state, &input, &renderer, &writer);
        assert_eq!(input_snapshot(&input).text, "alpha ");
        feed_input_bytes(b"\x05\x0b", &state, &input, &renderer, &writer);
        feed_input_bytes(b"\x7f\x19\x19", &state, &input, &renderer, &writer);
        assert_eq!(input_snapshot(&input).text, "alphabetabeta");
        feed_input_bytes(b"\x1b[A\x1b[B\r", &state, &input, &renderer, &writer);
        assert_eq!(read_sent_body(read_stream), "alphabetabeta");

        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        feed_input_bytes(b">\x19\r", &state, &input, &renderer, &writer);
        assert_eq!(
            read_sent_body(read_stream),
            ">beta",
            "le registre reste dans InputBuffer après l'envoi, comme après reconnexion"
        );
    }

    #[test]
    fn spec092_us5_ctrl_u_k_remplacent_le_registre_par_des_fragments_unicode_sans_lf() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"ancien".to_vec(),
            ..InputBuffer::default()
        }));
        let renderer = test_renderer_sender(true, true);

        feed_input_bytes(b"\x17", &state, &input, &renderer, &writer);
        feed_input_bytes(
            "pré界 milieu\nsuite".as_bytes(),
            &state,
            &input,
            &renderer,
            &writer,
        );
        input.lock().unwrap().cursor = Some("pré界".len());

        feed_input_bytes(b"\x15", &state, &input, &renderer, &writer);
        {
            let input = input.lock().unwrap();
            assert_eq!(input.display(), " milieu\nsuite");
            assert_eq!(input.last_deleted.as_slice(), "pré界".as_bytes());
        }
        feed_input_bytes(b"\x19", &state, &input, &renderer, &writer);
        assert_eq!(input_snapshot(&input).text, "pré界 milieu\nsuite");

        feed_input_bytes(b"\x0b", &state, &input, &renderer, &writer);
        {
            let input = input.lock().unwrap();
            assert_eq!(input.display(), "pré界\nsuite");
            assert_eq!(input.last_deleted.as_slice(), b" milieu");
        }
        feed_input_bytes(b"\x19\r", &state, &input, &renderer, &writer);
        assert_eq!(read_sent_body(read_stream), "pré界 milieu\nsuite");
    }

    #[test]
    fn spec092_us5_controles_editeur_sont_inertes_hors_double_tty() {
        let controls: &[&[u8]] = &[
            b"\x01",
            b"\x05",
            b"\x15",
            b"\x0b",
            b"\x17",
            b"\x19",
            b"\x1b\x08",
            b"\x1b\x7f",
            b"\x1bd",
            b"\x1b[97;5u",
            b"\x1b[101;5u",
            b"\x1b[117;5u",
            b"\x1b[107;5u",
            b"\x1b[119;5u",
            b"\x1b[121;5u",
            b"\x1b[127;3u",
            b"\x1b[100;3u",
        ];
        for (stdin_tty, stdout_tty) in [(true, false), (false, true), (false, false)] {
            for control in controls {
                let (write_stream, read_stream) = UnixStream::pair().unwrap();
                let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
                let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
                let input = Arc::new(Mutex::new(InputBuffer {
                    bytes: b"abcd".to_vec(),
                    cursor: Some(2),
                    ..InputBuffer::default()
                }));
                let renderer = test_renderer_sender(stdin_tty, stdout_tty);

                feed_input_bytes(control, &state, &input, &renderer, &writer);
                assert_eq!(input_snapshot(&input).text, "abcd");
                feed_input_bytes(b"\r", &state, &input, &renderer, &writer);
                assert_eq!(read_sent_body(read_stream), "abcd");
            }
        }
    }

    #[test]
    fn spec092_fleches_hors_double_tty_sont_consommees_sans_effet_renderer() {
        for (stdin_tty, stdout_tty) in [(true, false), (false, true), (false, false)] {
            let (commands, command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
            let renderer = RendererSender {
                commands,
                input_dirty: Arc::new(AtomicBool::new(false)),
                raw_terminal: stdin_tty,
                tty_output: stdout_tty,
            };
            let (write_stream, _) = UnixStream::pair().unwrap();
            let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
            let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
            let input = Arc::new(Mutex::new(InputBuffer {
                bytes: b"brouillon".to_vec(),
                ..InputBuffer::default()
            }));
            input.lock().unwrap().remember(b"ancien".to_vec());

            for byte in b"\x1b[A\x1bOB" {
                handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
            }

            assert_eq!(input.lock().unwrap().display(), "brouillon");
            assert!(matches!(
                command_rx.try_recv(),
                Err(mpsc::TryRecvError::Empty)
            ));
        }
    }

    #[test]
    fn spec092_historique_borne_conserve_les_doublons_et_refuse_une_entree_geante() {
        let mut input = InputBuffer::default();
        for index in 0..=INPUT_HISTORY_MAX_ENTRIES {
            input.remember(index.to_string().into_bytes());
        }
        assert_eq!(input.history.len(), INPUT_HISTORY_MAX_ENTRIES);
        assert_eq!(input.history.front().unwrap().as_slice(), b"1");
        assert_eq!(input.history.back().unwrap().as_slice(), b"100");
        assert_eq!(
            input.history_bytes,
            input.history.iter().map(Vec::len).sum::<usize>()
        );

        input.remember(b"100".to_vec());
        assert_eq!(input.history.back().unwrap().as_slice(), b"100");
        assert_eq!(input.history[input.history.len() - 2].as_slice(), b"100");
        let before = input.history.clone();
        input.remember(vec![b'x'; INPUT_HISTORY_MAX_BYTES + 1]);
        assert_eq!(input.history, before, "l'entrée géante n'évince rien");

        let mut byte_bounded = InputBuffer::default();
        byte_bounded.remember(vec![b'a'; 600_000]);
        byte_bounded.remember(vec![b'b'; 600_000]);
        assert_eq!(byte_bounded.history.len(), 1);
        assert_eq!(byte_bounded.history_bytes, 600_000);
    }

    #[test]
    fn spec092_memoire_suit_uniquement_les_ecritures_reussies() {
        let (write_stream, mut read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);
        for byte in b"/model gpt-5.6-sol high\r/model incomplet\r\r" {
            handle_input_byte(*byte, &state, &input, &renderer, &writer, "codex-1").unwrap();
        }
        let mut line = String::new();
        BufReader::new(&mut read_stream)
            .read_line(&mut line)
            .unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
            WrapperToDaemon::SelectRuntime { .. }
        ));
        assert_eq!(
            input.lock().unwrap().history,
            VecDeque::from([b"/model gpt-5.6-sol high".to_vec()])
        );

        let (failed_stream, failed_peer) = UnixStream::pair().unwrap();
        drop(failed_peer);
        let failed_writer = Arc::new(Mutex::new(BufWriter::new(failed_stream)));
        let failed_input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"non memorise".to_vec(),
            ..InputBuffer::default()
        }));
        assert!(
            handle_input_byte(
                b'\r',
                &state,
                &failed_input,
                &renderer,
                &failed_writer,
                "codex-1",
            )
            .is_err()
        );
        assert!(failed_input.lock().unwrap().history.is_empty());
        assert!(InputBuffer::default().history.is_empty());
    }

    #[test]
    fn spec092_message_geant_est_envoye_sans_evincer_l_historique() {
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let reader = thread::spawn(move || {
            let mut line = String::new();
            BufReader::new(read_stream).read_line(&mut line).unwrap();
            decode::<WrapperToDaemon>(line.trim_end()).unwrap()
        });
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: vec![b'x'; INPUT_HISTORY_MAX_BYTES + 1],
            ..InputBuffer::default()
        }));
        input.lock().unwrap().remember(b"precedent".to_vec());
        let renderer = test_renderer_sender(false, false);

        assert!(handle_input_byte(b'\r', &state, &input, &renderer, &writer, "codex-1").unwrap());
        let WrapperToDaemon::Send(message) = reader.join().unwrap() else {
            panic!("Send attendu");
        };
        assert_eq!(message.body.len(), INPUT_HISTORY_MAX_BYTES + 1);
        assert_eq!(
            input.lock().unwrap().history,
            VecDeque::from([b"precedent".to_vec()])
        );
    }

    #[test]
    fn spec092_drive_reutilise_l_historique_apres_reconnexion_et_nouvelle_invocation_est_vide() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut state = AttachClientState::new(AttachWindow::Today);

        let (client_stream, server_stream) = UnixStream::pair().unwrap();
        let connection = AttachConnection {
            reader: BufReader::new(client_stream.try_clone().unwrap()),
            writer: Arc::new(Mutex::new(BufWriter::new(client_stream))),
        };
        let server = thread::spawn(move || {
            let mut line = String::new();
            BufReader::new(server_stream).read_line(&mut line).unwrap();
            decode::<WrapperToDaemon>(line.trim_end()).unwrap()
        });
        let (mut first_input_writer, first_input_reader) = UnixStream::pair().unwrap();
        first_input_writer.write_all(b"A\r").unwrap();
        first_input_writer.flush().unwrap();
        assert!(
            drive_interactive(
                connection,
                &mut state,
                "codex-1",
                Path::new("/tmp/bridget-attach-092-reconnect-unused.sock"),
                InteractiveInput {
                    fd: first_input_reader.as_raw_fd(),
                    buffer: &input,
                    raw_terminal: true,
                    tty_output: true,
                },
            )
            .unwrap(),
            "la fermeture du daemon demande une reconnexion"
        );
        assert!(matches!(
            server.join().unwrap(),
            WrapperToDaemon::Send(message) if message.body == "A"
        ));
        assert_eq!(
            input.lock().unwrap().history,
            VecDeque::from([b"A".to_vec()])
        );

        let (client_stream, _server_stream) = UnixStream::pair().unwrap();
        let connection = AttachConnection {
            reader: BufReader::new(client_stream.try_clone().unwrap()),
            writer: Arc::new(Mutex::new(BufWriter::new(client_stream))),
        };
        let (mut second_input_writer, second_input_reader) = UnixStream::pair().unwrap();
        second_input_writer.write_all(b"\x1b[A\x03").unwrap();
        second_input_writer.flush().unwrap();
        assert!(
            !drive_interactive(
                connection,
                &mut state,
                "codex-1",
                Path::new("/tmp/bridget-attach-092-reconnect-unused.sock"),
                InteractiveInput {
                    fd: second_input_reader.as_raw_fd(),
                    buffer: &input,
                    raw_terminal: true,
                    tty_output: true,
                },
            )
            .unwrap(),
            "Ctrl-C termine la seconde connexion"
        );
        assert_eq!(input.lock().unwrap().display(), "A");
        assert!(InputBuffer::default().history.is_empty());
    }

    #[test]
    fn spec091_footer_rend_le_curseur_a_la_saisie_et_lefface_au_redessin() {
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"demande en cours".to_vec(),
            ..InputBuffer::default()
        }));
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 180;
        let mut output = Vec::new();

        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        let rendered = String::from_utf8(output).unwrap();
        // L'oracle porte sur le texte et les mouvements de curseur : les
        // couleurs SGR du footer et de l'invite sont retirées avant comparaison.
        let sans_sgr = strip_sgr(&rendered);
        let footer = "fournisseur inconnu";
        let footer_end = sans_sgr.find(footer).unwrap() + footer.len();
        assert!(
            sans_sgr[footer_end..].contains("\r\x1b[1A> demande en cours"),
            "le curseur doit revenir à la saisie après le footer : {rendered:?}"
        );

        let mut redraw = Vec::new();
        renderer.redraw(&InputSnapshot::at_end("demande en cours"), &mut redraw);
        assert!(
            redraw.starts_with(b"\r\n\r\x1b[2K"),
            "le redraw doit d'abord rejoindre puis effacer le footer : {redraw:?}"
        );
    }

    #[test]
    fn spec091_premiere_presence_ne_decale_pas_leffacement_dune_saisie_sans_footer() {
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"demande en cours".to_vec(),
            ..InputBuffer::default()
        }));
        let mut renderer = BlockRenderer::new("agent".into(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 180;
        let mut initial = Vec::new();
        renderer.redraw(&InputSnapshot::at_end("demande en cours"), &mut initial);
        assert!(!renderer.cursor_above_footer);

        let mut first_presence = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut first_presence,
        );

        assert!(
            first_presence.starts_with(b"\r\x1b[2K"),
            "une première présence ne doit pas descendre sous une saisie sans footer : {first_presence:?}"
        );
        assert!(renderer.cursor_above_footer);
    }

    #[test]
    fn spec091_finish_rend_la_main_sous_le_footer_ou_la_saisie() {
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"demande en cours".to_vec(),
            ..InputBuffer::default()
        }));
        let mut with_footer = BlockRenderer::new("agent".into(), true, true);
        with_footer.terminal_fd = None;
        with_footer.terminal_columns = 180;
        let mut footer_output = Vec::new();
        with_footer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut footer_output,
        );
        let footer_before_finish = footer_output.len();
        with_footer.finish(
            &InputSnapshot::at_end("demande en cours"),
            &mut footer_output,
        );
        assert_eq!(&footer_output[footer_before_finish..], b"\r\n\r\n");

        let mut without_footer = BlockRenderer::new("agent".into(), true, true);
        without_footer.terminal_fd = None;
        without_footer.terminal_columns = 180;
        let mut input_output = Vec::new();
        without_footer.redraw(
            &InputSnapshot::at_end("demande en cours"),
            &mut input_output,
        );
        let input_before_finish = input_output.len();
        without_footer.finish(
            &InputSnapshot::at_end("demande en cours"),
            &mut input_output,
        );
        assert_eq!(&input_output[input_before_finish..], b"\r\n");
    }

    fn test_renderer_sender(raw_terminal: bool, tty_output: bool) -> RendererSender {
        let (commands, _command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        RendererSender {
            commands,
            input_dirty: Arc::new(AtomicBool::new(false)),
            raw_terminal,
            tty_output,
        }
    }

    fn feed_input_bytes(
        bytes: &[u8],
        state: &Arc<Mutex<AttachClientState>>,
        input: &Arc<Mutex<InputBuffer>>,
        renderer: &RendererSender,
        writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    ) {
        for byte in bytes {
            assert!(handle_input_byte(*byte, state, input, renderer, writer, "codex-1").unwrap());
        }
    }

    fn read_sent_body(stream: UnixStream) -> String {
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu");
        };
        message.body
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

        fn set_size(&self, rows: u16, columns: u16) {
            let size = libc::winsize {
                ws_row: rows,
                ws_col: columns,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            assert_eq!(
                unsafe { libc::ioctl(self.slave, libc::TIOCSWINSZ, &size) },
                0,
                "TIOCSWINSZ: {}",
                std::io::Error::last_os_error()
            );
        }

        fn slave_writer(&self) -> File {
            let fd = unsafe { libc::dup(self.slave) };
            assert!(fd >= 0, "dup slave: {}", std::io::Error::last_os_error());
            unsafe { File::from_raw_fd(fd) }
        }

        fn slave_reader(&self) -> File {
            let path = unsafe { libc::ttyname(self.slave) };
            assert!(
                !path.is_null(),
                "ttyname: {}",
                std::io::Error::last_os_error()
            );
            let fd = unsafe { libc::open(path, libc::O_RDONLY | libc::O_NOCTTY) };
            assert!(
                fd >= 0,
                "ouverture en lecture seule du slave: {}",
                std::io::Error::last_os_error()
            );
            unsafe { File::from_raw_fd(fd) }
        }

        fn read_available(&self) -> Vec<u8> {
            let fd = unsafe { libc::dup(self.master) };
            assert!(fd >= 0, "dup master: {}", std::io::Error::last_os_error());
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            assert!(flags >= 0);
            assert_eq!(
                unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) },
                0
            );
            let mut file = unsafe { File::from_raw_fd(fd) };
            let mut output = Vec::new();
            loop {
                let mut chunk = [0_u8; 4096];
                match file.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(count) => output.extend_from_slice(&chunk[..count]),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(error) => panic!("lecture pseudo-TTY: {error}"),
                }
            }
            output
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
        assert_eq!(after.c_iflag, before.c_iflag);
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

    #[test]
    fn spec100_excerpt_preserves_unicode_and_marks_limit() {
        let mut excerpt = JournalExcerpt::new("agent-a");
        assert!(excerpt.push(1, br#"{"seq":1,"text":"hello"}"#.to_vec()));
        let unicode = serde_json::to_vec(&json!({"seq":2,"text":"équipier 🦀"})).unwrap();
        assert!(excerpt.push(2, unicode));
        assert_eq!(excerpt.entries[1]["text"], "équipier 🦀");
        assert!(!excerpt.push(3, vec![b'x'; JOURNAL_EXCERPT_BYTES + 1]));
        assert!(!excerpt.complete);
        assert_eq!(excerpt.next_seq, Some(3));
        assert_eq!(excerpt.entries.len(), 2);
        let mut window = JournalExcerpt::new("agent-a");
        for seq in 11..=210 {
            assert!(window.push(
                seq,
                serde_json::to_vec(&json!({"seq":seq,"text":"é 🦀"})).unwrap()
            ));
            if seq == 60 {
                assert_eq!(window.entries.len(), 50);
                assert_eq!((window.first_seq, window.last_seq), (Some(11), Some(60)));
            }
        }
        assert!(!window.push(211, serde_json::to_vec(&json!({"seq":211})).unwrap()));
        assert_eq!(window.entries.len(), 200);
        assert_eq!(window.next_seq, Some(211));
        assert!(window.shared_body().len() < 64 * 1024);
    }

    #[test]
    fn spec100_excerpt_rejects_corrupt_or_mismatched_entry() {
        let mut excerpt = JournalExcerpt::new("agent-a");
        assert!(!excerpt.push(3, br#"{"seq":4}"#.to_vec()));
        assert!(!excerpt.complete);
        assert!(excerpt.entries.is_empty());
    }

    #[test]
    fn spec100_excerpt_socket_fragments_unicode_gap_and_invalid_options() {
        use std::os::unix::net::UnixListener;
        let root = std::env::temp_dir().join(format!("bg100-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&root).unwrap();
        let socket = root.join("s");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            for response in [
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Attach,
                },
                DaemonToWrapper::Subscribed {
                    subscription_id: "s".into(),
                },
            ] {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                decode::<WrapperToDaemon>(line.trim()).unwrap();
                writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
            }
            let bytes = serde_json::to_vec(&json!({"seq":3,"text":"équipier 🦀"})).unwrap();
            let split = bytes.iter().position(|b| *b == 0xf0).unwrap() + 1;
            for response in [
                DaemonToWrapper::Gap {
                    subscription_id: "s".into(),
                    from_seq: 1,
                    to_seq: 2,
                    reason: Some("retention".into()),
                },
                fragment("s", 3, 0, false, bytes[..split].to_vec()),
                fragment("s", 3, split, true, bytes[split..].to_vec()),
                DaemonToWrapper::SnapshotCaughtUp {
                    subscription_id: "s".into(),
                    through_seq: Some(3),
                },
            ] {
                writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
            }
        });
        let excerpt = read_journal_excerpt(&socket, "agent", AttachWindow::Seq(1)).unwrap();
        server.join().unwrap();
        assert_eq!(excerpt.entries[0]["text"], "équipier 🦀");
        assert_eq!(excerpt.next_seq, Some(4));
        assert_eq!(excerpt.through_seq, Some(3));
        assert!(!excerpt.complete);
        assert!(excerpt.notices.iter().any(|n| n.contains("journal_gap")));
        assert!(
            read_journal_excerpt(&root.join("absent"), "agent", AttachWindow::Tail(1)).is_err()
        );
        for value in [
            json!({"agent":"a","tail":1,"from_seq":2}),
            json!({"agent":"a","reply":true}),
            json!({"agent":"a","tail":201}),
            json!({"agent":"a","from_seq":0}),
        ] {
            let request: JournalRequest = serde_json::from_value(value).unwrap();
            assert!(matches!(
                request.read(&socket),
                Err(crate::communication::client::ClientError::InvalidParams(_))
            ));
        }
    }

    #[test]
    fn spec100_excerpt_silent_server_exhausts_shared_budget() {
        use std::os::unix::net::UnixListener;
        let root = std::env::temp_dir().join(format!("b100-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&root).unwrap();
        let socket = root.join("s");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(15)))
                .unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            // Aucune réponse : le client doit fermer au terme de son budget,
            // sans attendre une fin de journal que ce serveur ne fournira pas.
            line.clear();
            assert_eq!(reader.read_line(&mut line).unwrap(), 0);
        });
        let start = Instant::now();
        assert!(read_journal_excerpt(&socket, "agent", AttachWindow::Tail(1)).is_err());
        assert!(start.elapsed() < Duration::from_secs(14));
        server.join().unwrap();
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
        journal_record_for_turn(seq, "session-1", "message-1", event, payload)
    }

    fn journal_record_for_turn(
        seq: u64,
        session_id: &str,
        message_id: &str,
        event: &str,
        payload: serde_json::Value,
    ) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "v": 1,
            "seq": seq,
            "ts": "2026-08-23T09:07:00Z",
            "session_id": session_id,
            "message_id": message_id,
            "event": event,
            "payload": payload,
        }))
        .unwrap()
    }

    static SENDER_ATTRIBUTION_SEQ: AtomicU64 = AtomicU64::new(0);

    fn sender_attribution_root(pilot: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-attach-sender-attribution-{pilot}-{}-{}",
            std::process::id(),
            SENDER_ATTRIBUTION_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
        ))
    }

    fn turn_start_from_real_pilot<S: ManagedSession>(
        transport: &mut S,
        root: &std::path::Path,
        agent: &str,
        message_id: &str,
    ) -> (serde_json::Value, String) {
        fs::create_dir_all(root).expect("racine de journal du témoin");
        transport
            .activate_journal(root, agent, None)
            .expect("activation du JournalWriter");
        let mut message = BridgetMessage::new("jc2", "bridget", "TRANCHE SPEC 052 POUSSEE");
        message.id = message_id.to_string();
        transport.deliver(&message).expect("livraison au pilote");

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
        assert!(
            finished,
            "le pilote doit terminer avant la lecture du journal"
        );
        transport.stop();

        let journal_path = fs::read_dir(root.join(agent))
            .expect("répertoire JournalWriter")
            .next()
            .expect("fichier JSONL")
            .expect("entrée JSONL")
            .path();
        let line = fs::read_to_string(journal_path)
            .expect("lecture JSONL")
            .lines()
            .find(|line| {
                serde_json::from_str::<serde_json::Value>(line)
                    .ok()
                    .is_some_and(|record| record["event"] == "turn_start")
            })
            .expect("turn_start écrit par le pilote")
            .to_string();
        let record = serde_json::from_str(&line).expect("turn_start JSON valide");
        let rendered = render_journal_event(line.as_bytes(), agent);
        fs::remove_dir_all(root).expect("nettoyage du témoin");
        (record, rendered)
    }

    fn assert_sender_attribution(record: serde_json::Value, rendered: String) {
        assert_eq!(
            record
                .pointer("/payload/from")
                .and_then(serde_json::Value::as_str),
            Some("jc2"),
            "turn_start doit persister l’émetteur réel avant tout enrichissement ultérieur"
        );
        assert!(
            rendered.contains("jc2 →"),
            "Attach doit rendre l’émetteur persistant, reçu {rendered:?}"
        );
        assert!(
            !rendered.contains("humain →"),
            "Attach ne doit pas appliquer son repli humain, reçu {rendered:?}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_codex_turn_start_attribue_le_sender_reel_jusqu_a_Attach() {
        let root = sender_attribution_root("codex");
        let options = CodexAppServerOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                r#"while IFS= read -r line; do
                    case "$line" in
                        *'"method":"initialize"'*) printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"linux"}}' ;;
                        *'"method":"thread/start"'*) printf '%s\n' '{"id":2,"result":{"thread":{"id":"thread-attribution"}}}' ;;
                        *'"method":"account/rateLimits/read"'*) printf '%s\n' '{"id":3,"result":{"rateLimits":{}}}' ;;
                        *'"method":"turn/start"'*)
                            printf '%s\n' '{"id":4,"result":{"turn":{"id":"turn-attribution"}}}'
                            printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-attribution","turnId":"turn-attribution","itemId":"item","delta":"ok"}}'
                            printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-attribution","turn":{"id":"turn-attribution","status":"completed","items":[]}}}'
                            ;;
                    esac
                done"#
                    .to_string(),
            ],
            queue_capacity: 1,
            notify_timeout_secs: 2,
            model: None,
            permissions: "allow".to_string(),
            provider_observation: None,
            thread_bootstrap: Default::default(),
            dynamic_tool_handler: None,
        };
        let mut transport = CodexAppServerTransport::spawn(options).expect("pilote Codex");
        let (record, rendered) =
            turn_start_from_real_pilot(&mut transport, &root, "codex-attribution", "codex-055");
        assert_sender_attribution(record, rendered);
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_claude_turn_start_attribue_le_sender_reel_jusqu_a_Attach() {
        let root = sender_attribution_root("claude");
        let options = ClaudeStreamJsonOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                concat!(
                    "while IFS= read -r line; do ",
                    "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                    "done"
                )
                .to_string(),
            ],
            provider_kind: "claude".to_string(),
            queue_capacity: 1,
            notify_timeout_secs: 2,
            session_store_root: None,
            agent_name: None,
            provider_observation: None,
        };
        let mut transport = ClaudeStreamJsonTransport::spawn(options).expect("pilote Claude");
        let (record, rendered) =
            turn_start_from_real_pilot(&mut transport, &root, "claude-attribution", "claude-055");
        assert_sender_attribution(record, rendered);
    }

    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_prompt_dispatched_enrichi_remplace_l_entete_legacy_sans_from() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-legacy".to_string(), false, true);
        renderer.terminal_fd = None;
        let mut output = Vec::new();
        for (seq, message_id, event, payload) in [
            (
                1,
                "legacy-enriched",
                "turn_start",
                json!({"body":"TRANCHE SPEC 052 POUSSEE"}),
            ),
            (
                2,
                "legacy-enriched",
                "prompt_dispatched",
                json!({"from":"jc2","body":"TRANCHE SPEC 052 POUSSEE"}),
            ),
            (
                3,
                "legacy-enriched",
                "turn_end",
                json!({"stop_reason":"end_turn"}),
            ),
            (
                4,
                "legacy-unattested",
                "turn_start",
                json!({"body":"ARCHIVE SANS EMETTEUR"}),
            ),
            (
                5,
                "legacy-unattested",
                "turn_end",
                json!({"stop_reason":"end_turn"}),
            ),
        ] {
            renderer.apply(
                RendererCommand::Event(AttachEvent::Journal {
                    seq,
                    bytes: journal_record_for_turn(
                        seq,
                        "session-legacy",
                        message_id,
                        event,
                        payload,
                    ),
                    live: false,
                }),
                &input,
                &mut output,
            );
        }

        let rendered = String::from_utf8(output).unwrap();
        assert!(
            rendered.contains("jc2 → TRANCHE SPEC 052 POUSSEE"),
            "le prompt enrichi doit remplacer l’émetteur de l’en-tête : {rendered:?}"
        );
        assert!(
            !rendered.contains("humain → TRANCHE SPEC 052 POUSSEE"),
            "le repli historique ne doit plus survivre à l’enrichissement : {rendered:?}"
        );
        assert!(
            rendered.contains("émetteur non attesté → ARCHIVE SANS EMETTEUR"),
            "une provenance absente doit rester explicitement non attestée : {rendered:?}"
        );
        assert!(
            !rendered.contains("humain → ARCHIVE SANS EMETTEUR"),
            "une provenance absente ne doit jamais devenir humaine : {rendered:?}"
        );
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
    fn stdin_et_stdout_pipes_conservent_le_golden_sans_echo_de_saisie() {
        let (commands, command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        let renderer_sender = RendererSender {
            commands,
            input_dirty: Arc::new(AtomicBool::new(false)),
            raw_terminal: false,
            tty_output: false,
        };
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let (socket, _peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(socket)));
        for byte in b"commande pipe\n" {
            assert!(
                handle_input_byte(*byte, &state, &input, &renderer_sender, &writer, "codex-1",)
                    .unwrap()
            );
        }
        assert!(
            command_rx.try_recv().is_err(),
            "stdin non-TTY ne doit jamais produire d'écho sur stdout non-TTY"
        );

        let event = AttachEvent::Subscribed {
            subscription_id: "sub-pipe".to_string(),
        };
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, false);
        let mut output = Vec::new();
        renderer.apply(RendererCommand::Event(event.clone()), &input, &mut output);
        assert_eq!(
            output,
            format!("{}\n", render_attach_event(&event, "codex-1")).into_bytes()
        );
    }

    #[test]
    fn bloc_live_nefface_que_la_fenetre_visible_du_terminal() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 40;
        renderer.terminal_rows = 8;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 1,
                bytes: journal_record(1, "turn_start", json!({"from":"humain","body":"Question"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        for seq in 2..=20 {
            renderer.apply(
                RendererCommand::Event(AttachEvent::Journal {
                    seq,
                    bytes: journal_record(
                        seq,
                        "update",
                        json!({"kind":"tool_call","title":format!("outil-{seq}"),"summary":"exécuté"}),
                    ),
                    live: true,
                }),
                &input,
                &mut output,
            );
            assert!(
                renderer.rendered_rows < renderer.terminal_rows,
                "le rendu effaçable dépasse la fenêtre visible"
            );
        }
        assert!(
            renderer.committed_units > 0,
            "les lignes sorties doivent être figées dans le scrollback"
        );
    }

    #[test]
    fn lot_de_chunks_live_ne_declenche_qu_un_redessin() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        renderer.terminal_fd = None;
        let mut commands = vec![RendererCommand::Event(AttachEvent::Journal {
            seq: 1,
            bytes: journal_record(1, "turn_start", json!({"from":"humain","body":"Question"})),
            live: true,
        })];
        commands.extend((2..=32).map(|seq| {
            RendererCommand::Event(AttachEvent::Journal {
                seq,
                bytes: journal_record(seq, "update", json!({"kind":"text","content":"mot "})),
                live: true,
            })
        }));
        let mut output = Vec::new();
        renderer.apply_batch(commands, &input, &mut output);
        assert_eq!(renderer.redraw_count, 1);
        assert_eq!(
            renderer
                .current
                .as_ref()
                .unwrap()
                .response
                .matches("mot ")
                .count(),
            31
        );
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
            ..InputBuffer::default()
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
        assert!(renderer.retained.is_some());
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Question"));
        assert!(output.contains("Réponse live"));
        assert!(output.contains("historique rattrapé jusqu’à 1"));
        assert!(!output.contains("tour terminé : end_turn"));
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
    fn sc001_trois_tours_deterministes_tiennent_dans_un_ecran_80_par_24() {
        let started_at = Instant::now();
        let mut events = Vec::new();
        let mut seq = 1_u64;
        for turn in 1..=3 {
            let session_id = format!("session-sc001-{turn}");
            let message_id = format!("message-sc001-{turn}");
            events.push(AttachEvent::Journal {
                seq,
                bytes: journal_record_for_turn(
                    seq,
                    &session_id,
                    &message_id,
                    "turn_start",
                    json!({"from":"humain","body":format!("Question déterministe {turn}")}),
                ),
                live: true,
            });
            seq += 1;
            for chunk in 0..20 {
                if turn == 2 && chunk == 10 {
                    events.push(AttachEvent::Journal {
                        seq,
                        bytes: journal_record_for_turn(
                            seq,
                            &session_id,
                            &message_id,
                            "update",
                            json!({
                                "kind":"tool_call",
                                "title":"Read src/main.rs",
                                "name":"read_file",
                                "tool_kind":"read",
                                "summary":"lecture"
                            }),
                        ),
                        live: true,
                    });
                    seq += 1;
                }
                events.push(AttachEvent::Journal {
                    seq,
                    bytes: journal_record_for_turn(
                        seq,
                        &session_id,
                        &message_id,
                        "update",
                        json!({"kind":"text","content":"mot "}),
                    ),
                    live: true,
                });
                seq += 1;
            }
            events.push(AttachEvent::Journal {
                seq,
                bytes: journal_record_for_turn(
                    seq,
                    &session_id,
                    &message_id,
                    "turn_end",
                    json!({"stop_reason":"end_turn"}),
                ),
                live: true,
            });
            seq += 1;
        }

        let legacy_lines = events
            .iter()
            .map(|event| render_attach_event(event, "codex-bench").lines().count())
            .sum::<usize>();
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-bench".to_string(), false, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 80;
        let mut output = Vec::new();
        let mut compact_rows = 0;
        for event in events {
            if let AttachEvent::Journal { bytes, .. } = &event
                && let Some(record) = journal_render_record(bytes, "codex-bench")
                && record.event == "turn_end"
            {
                let mut final_lines = renderer
                    .current
                    .as_ref()
                    .expect("bloc courant avant turn_end")
                    .lines("codex-bench");
                final_lines.push(record.rendered);
                compact_rows += visual_rows(&final_lines, 80);
            }
            renderer.apply(RendererCommand::Event(event), &input, &mut output);
        }

        eprintln!(
            "SC-001 80x24, 3 tours dont un outil : ancien={legacy_lines} lignes, blocs={compact_rows} lignes visuelles"
        );
        assert!(legacy_lines > 60, "baseline historique={legacy_lines}");
        assert!(compact_rows <= 24, "rendu compact={compact_rows}");
        assert!(String::from_utf8_lossy(&output).contains("[outil] Read src/main.rs"));
        assert!(
            started_at.elapsed() < Duration::from_secs(2),
            "le banc visuel a dépassé son budget global de 2 s"
        );
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
    fn permission_complete_la_ligne_de_loutil_dans_le_meme_bloc() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        renderer.terminal_fd = None;
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (
                2,
                "update",
                json!({"kind":"tool_call","title":"Read src/main.rs","summary":"lecture"}),
            ),
            (
                3,
                "permission",
                json!({
                    "tool":"Read src/main.rs",
                    "options":[{"optionId":"allow-1","kind":"allow_once"}],
                    "decision":{"outcome":"selected","option_id":"allow-1"}
                }),
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
        let details = &renderer.current.as_ref().unwrap().details;
        assert!(details.iter().any(|line| {
            line.contains("[outil] Read src/main.rs") && line.ends_with("— accordé")
        }));
        assert_eq!(
            details
                .iter()
                .filter(|line| line.contains("[permission]"))
                .count(),
            0,
            "la décision corrélée ne doit pas créer un mur de statut séparé"
        );
    }

    #[test]
    fn reprise_en_cours_de_tour_conserve_lheure_du_premier_evenement() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 2,
                bytes: journal_record(2, "update", json!({"kind":"text","content":"suite"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        assert_eq!(
            renderer.current.as_ref().unwrap().header,
            [format!(
                "{} [tour repris en cours]",
                short_timestamp(Some("2026-08-23T09:07:01Z"))
            )]
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
    fn spec093_tty_rend_markdown_sous_un_label_atteste_sans_indent_uuid() {
        let agent_id = "550e8400-e29b-41d4-a716-446655440003";
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new(agent_id.to_string(), true, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 80;
        renderer.terminal_rows = 40;
        renderer.color = true;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        for (seq, event, payload) in [
            (
                1,
                "turn_start",
                json!({"from":"humain","body":"Montre-moi"}),
            ),
            (
                2,
                "update",
                json!({"kind":"text","content":"# Titre\n\n- **un**\n- *deux*\n\n```rust\nlet x = 1;\n```"}),
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

        assert_eq!(
            renderer.agent, agent_id,
            "l'identité de routage ne change pas"
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line == "communication-091 ›")
        );
        assert!(renderer.rendered_lines.iter().any(|line| line == "Titre"));
        assert!(renderer.rendered_lines.iter().any(|line| line == "• un"));
        assert!(renderer.rendered_lines.iter().any(|line| line == "• deux"));
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line == "  let x = 1;")
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .all(|line| !line.contains("```"))
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .all(|line| !line.starts_with(&" ".repeat(37)))
        );
        assert!(output.windows(2).any(|bytes| bytes == b"\x1b["));
    }

    #[test]
    fn spec093_label_inconnu_reste_un_uuid_court_cosmetique() {
        let agent_id = "550e8400-e29b-41d4-a716-446655440003";
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new(agent_id.to_string(), false, true);
        renderer.terminal_fd = None;
        renderer.terminal_rows = 20;
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (2, "update", json!({"kind":"text","content":"réponse"})),
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
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line == "550e8400 ›")
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .all(|line| !line.contains(agent_id))
        );
        assert_eq!(
            renderer.agent, agent_id,
            "le label ne devient jamais une adresse"
        );
    }

    #[test]
    fn spec093_tty_masque_reasoning_et_fin_reussie_mais_garde_les_echecs() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        renderer.terminal_fd = None;
        renderer.terminal_rows = 40;
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (
                2,
                "reasoning",
                json!({"available":true,"summary":"secret de raisonnement"}),
            ),
            (
                3,
                "update",
                json!({
                    "kind":"refusal",
                    "text":"sandbox interdit",
                    "layer":"provider_sandbox",
                    "evidence":"output_and_exit"
                }),
            ),
            (4, "update", json!({"kind":"text","content":"Réponse"})),
            (5, "turn_end", json!({"stop_reason":"completed"})),
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
        let retained = renderer
            .retained
            .as_ref()
            .expect("dernière réponse retenue");
        assert!(
            retained
                .details
                .iter()
                .all(|line| !line.contains("raisonnement"))
        );
        assert!(
            retained.details.iter().any(|line| {
                line.contains("[refus Bridget]") && line.contains("sandbox interdit")
            })
        );
        let visible = String::from_utf8_lossy(&output);
        assert!(!visible.contains("secret de raisonnement"));
        assert!(!visible.contains("tour terminé : completed"));

        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 6,
                bytes: journal_record(6, "turn_end", json!({"stop_reason":"interrupted"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        assert!(String::from_utf8_lossy(&output).contains("interrupted"));

        let mut diagnostic = BlockRenderer::new("codex-1".to_string(), false, false);
        let mut diagnostic_output = Vec::new();
        diagnostic.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 7,
                bytes: journal_record(
                    7,
                    "reasoning",
                    json!({"available":true,"summary":"diagnostic conservé"}),
                ),
                live: true,
            }),
            &input,
            &mut diagnostic_output,
        );
        assert!(String::from_utf8_lossy(&diagnostic_output).contains("diagnostic conservé"));
    }

    #[test]
    fn spec093_fin_reussie_non_correlee_reste_visible_et_evacue_le_tour_incomplet() {
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new("codex-1".to_string(), false, true);
        renderer.terminal_fd = None;
        renderer.terminal_rows = 40;
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (
                2,
                "update",
                json!({"kind":"text","content":"Réponse partielle"}),
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
        let uncorrelated = serde_json::to_vec(&json!({
            "v": 1,
            "seq": 3,
            "ts": "2026-08-23T09:07:00Z",
            "event": "turn_end",
            "payload": {"stop_reason":"completed"},
        }))
        .unwrap();
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 3,
                bytes: uncorrelated,
                live: true,
            }),
            &input,
            &mut output,
        );
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 4,
                bytes: journal_record_for_turn(
                    4,
                    "session-2",
                    "message-2",
                    "turn_start",
                    json!({"from":"humain","body":"Suite"}),
                ),
                live: true,
            }),
            &input,
            &mut output,
        );

        let visible = String::from_utf8_lossy(&output);
        assert!(visible.contains("tour incomplet"));
        assert!(visible.contains("completed"));
        assert_eq!(
            renderer
                .current
                .as_ref()
                .map(|block| block.key.session_id.as_str()),
            Some("session-2")
        );
    }

    #[test]
    fn spec093_resize_sans_evenement_replie_derniere_reponse_et_preserve_footer() {
        let pseudo_tty = PseudoTerminal::open();
        pseudo_tty.set_size(30, 100);
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"brouillon intact".to_vec(),
            ..InputBuffer::default()
        }));
        let mut renderer = BlockRenderer::new(
            "550e8400-e29b-41d4-a716-446655440003".to_string(),
            true,
            true,
        );
        renderer.terminal_fd = Some(pseudo_tty.slave);
        renderer.terminal_columns = 100;
        renderer.terminal_rows = 30;
        renderer.color = true;
        let mut output = Vec::new();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        let response = format!("# Largeur\n\n{}", "mot ".repeat(35));
        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (2, "update", json!({"kind":"text","content":response})),
            (3, "turn_end", json!({"stop_reason":"end_turn"})),
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
        assert!(renderer.retained.is_some());
        let wide_rows = renderer.rendered_lines.len();

        pseudo_tty.set_size(30, 45);
        assert!(renderer.refresh_geometry(&mut output));
        renderer.redraw(&input_snapshot(&input), &mut output);
        let narrow_rows = renderer.rendered_lines.len();
        assert!(narrow_rows > wide_rows);
        assert!(
            renderer
                .rendered_lines
                .iter()
                .all(|line| display_width(line) <= 45)
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line.contains("brouillon intact"))
        );
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line.contains("modèle glm-exemple"))
        );

        pseudo_tty.set_size(30, 100);
        assert!(renderer.refresh_geometry(&mut output));
        renderer.redraw(&input_snapshot(&input), &mut output);
        assert_eq!(input.lock().unwrap().display(), "brouillon intact");
        assert!(renderer.retained.is_some());
        assert!(renderer.rendered_lines.len() < narrow_rows);

        pseudo_tty.set_size(30, 45);
        assert!(renderer.refresh_geometry(&mut output));
        renderer.redraw(&input_snapshot(&input), &mut output);
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 4,
                bytes: journal_record(4, "turn_start", json!({"from":"humain","body":"Suite"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 5,
                bytes: journal_record(
                    5,
                    "update",
                    json!({"kind":"text","content":"nouveau ".repeat(12)}),
                ),
                live: true,
            }),
            &input,
            &mut output,
        );
        assert!(renderer.current.is_some());
        assert!(renderer.retained.is_none());
        assert!(
            renderer
                .rendered_lines
                .iter()
                .all(|line| display_width(line) <= 45)
        );
    }

    #[test]
    fn spec093_bloc_deja_evacue_fige_label_et_header_logiques() {
        let agent_id = "550e8400-e29b-41d4-a716-446655440003";
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let mut renderer = BlockRenderer::new(agent_id.to_string(), false, true);
        renderer.terminal_fd = None;
        renderer.terminal_columns = 32;
        renderer.terminal_rows = 5;
        let mut output = Vec::new();
        for (seq, event, payload) in [
            (1, "turn_start", json!({})),
            (
                2,
                "update",
                json!({"kind":"text","content":"sentinelle ".repeat(40)}),
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
        assert!(renderer.committed_units > 0);
        let frozen = renderer
            .frozen_agent_label
            .clone()
            .expect("label figé au premier débordement");
        let logical_before = renderer
            .current
            .as_ref()
            .unwrap()
            .rendered_lines(&frozen, renderer.terminal_columns);

        let mut presence = presence_fixture();
        presence.display_name = "nom-attesté-beaucoup-plus-long".to_string();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence)), Instant::now()),
            &input,
            &mut output,
        );
        let mut presence = presence_fixture();
        presence.display_name = "court".to_string();
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence)), Instant::now()),
            &input,
            &mut output,
        );
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 3,
                bytes: journal_record(
                    3,
                    "prompt_dispatched",
                    json!({"from":"humain","body":"en-tête attesté tardif"}),
                ),
                live: true,
            }),
            &input,
            &mut output,
        );

        assert_eq!(
            renderer.frozen_agent_label.as_deref(),
            Some(frozen.as_str())
        );
        assert_eq!(renderer.agent_label, "court");
        assert_eq!(
            renderer
                .current
                .as_ref()
                .unwrap()
                .rendered_lines(&frozen, renderer.terminal_columns),
            logical_before,
            "la queue logique ne doit pas se décaler après engagement"
        );
    }

    #[test]
    fn spec093_renderer_loop_reagit_au_resize_sans_commande_ni_frappe() {
        let pseudo_tty = PseudoTerminal::open();
        pseudo_tty.set_size(12, 100);
        let mut output = pseudo_tty.slave_writer();
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let dirty = Arc::new(AtomicBool::new(false));
        let (commands, command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        let (control, control_rx) = mpsc::channel();
        let mut renderer = BlockRenderer::new("codex-1".to_string(), true, true);
        renderer.terminal_fd = Some(pseudo_tty.slave);
        renderer.terminal_columns = 100;
        renderer.terminal_rows = 12;
        let handle = thread::spawn(move || {
            renderer_loop(&mut output, renderer, input, dirty, command_rx, control_rx);
        });

        for (seq, event, payload) in [
            (1, "turn_start", json!({"from":"humain","body":"Question"})),
            (
                2,
                "update",
                json!({"kind":"text","content":"réponse redimensionnable ".repeat(6)}),
            ),
        ] {
            commands
                .send(RendererCommand::Event(AttachEvent::Journal {
                    seq,
                    bytes: journal_record(seq, event, payload),
                    live: true,
                }))
                .unwrap();
        }

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut initial = Vec::new();
        while !initial
            .windows("réponse redimensionnable".len())
            .any(|bytes| bytes == "réponse redimensionnable".as_bytes())
        {
            initial.extend(pseudo_tty.read_available());
            assert!(Instant::now() < deadline, "rendu initial absent du PTY");
            thread::yield_now();
        }
        initial.extend(pseudo_tty.read_available());

        // À partir d'ici, TIOCSWINSZ est la seule entrée avant l'observation.
        pseudo_tty.set_size(12, 45);
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut resized = Vec::new();
        while !(resized.windows(4).any(|bytes| bytes == b"\x1b[2K")
            && resized
                .windows("réponse".len())
                .any(|bytes| bytes == "réponse".as_bytes()))
        {
            resized.extend(pseudo_tty.read_available());
            assert!(
                Instant::now() < deadline,
                "aucun repaint autonome après resize"
            );
            thread::yield_now();
        }
        assert!(
            resized
                .windows("réponse".len())
                .any(|bytes| bytes == "réponse".as_bytes()),
            "le repaint doit reprojeter la réponse, pas seulement effacer; octets réels: {resized:?}; texte lossy: {:?}",
            String::from_utf8_lossy(&resized)
        );

        control.send(RendererControl::Stop).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn bloc_borne_affiche_sa_troncature() {
        let mut block = TurnBlock::new(
            TurnKey {
                session_id: "session".to_string(),
                message_id: "message".to_string(),
            },
            "09:07 humain → Question".to_string(),
            true,
        );
        for _ in 0..500 {
            block.append_response(&format!("{}\n", "x".repeat(200)));
        }

        assert!(block.stored_bytes <= MAX_TURN_BLOCK_BYTES);
        assert!(block.stored_lines <= MAX_TURN_BLOCK_LINES);
        assert!(block.omitted_lines > 0);
        let lines = block.lines("codex-1");
        assert!(lines.last().unwrap().starts_with("… tronqué,"));
        assert!(
            lines.iter().all(|line| !line.contains("affichage tronqué")),
            "le bloc ne doit porter qu'un seul marqueur de troncature"
        );
        assert!(
            block.response.len() > 60 * 1024,
            "la capacité utile de 64 Kio doit être consommée avant la troncature"
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
            raw_terminal: true,
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

        with_raw_terminal(
            pseudo_terminal.slave,
            pseudo_terminal.slave,
            |raw_terminal| {
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
            },
        )
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
        let renderer = test_renderer_sender(true, false);

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
            ..InputBuffer::default()
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
    fn pseudo_tty_garde_la_saisie_en_bas_pendant_le_flux_et_un_resize() {
        let pseudo_tty = PseudoTerminal::open();
        pseudo_tty.set_size(12, 40);
        let mut output = pseudo_tty.slave_writer();
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let (socket, _) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(socket)));
        let (commands, command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        let sender = RendererSender {
            commands,
            input_dirty: Arc::new(AtomicBool::new(false)),
            raw_terminal: true,
            tty_output: true,
        };
        let mut renderer = BlockRenderer::new("codex-1".to_string(), true, true);
        renderer.terminal_fd = Some(pseudo_tty.slave);
        (renderer.terminal_columns, renderer.terminal_rows) =
            terminal_geometry(pseudo_tty.slave).unwrap();
        let mut rendered = Vec::new();

        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 1,
                bytes: journal_record(
                    1,
                    "turn_start",
                    json!({"from":"humain","body":"Question entrante"}),
                ),
                live: true,
            }),
            &input,
            &mut output,
        );
        rendered.extend(pseudo_tty.read_available());
        for byte in b"commande partielle" {
            assert!(handle_input_byte(*byte, &state, &input, &sender, &writer, "codex-1").unwrap());
        }
        while let Ok(command) = command_rx.try_recv() {
            renderer.apply(command, &input, &mut output);
            rendered.extend(pseudo_tty.read_available());
        }
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 2,
                bytes: journal_record(2, "update", json!({"kind":"text","content":"réponse live"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        rendered.extend(pseudo_tty.read_available());
        assert_eq!(input.lock().unwrap().display(), "commande partielle");

        pseudo_tty.set_size(12, 18);
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 3,
                bytes: journal_record(
                    3,
                    "update",
                    json!({"kind":"text","content":" après resize"}),
                ),
                live: true,
            }),
            &input,
            &mut output,
        );
        rendered.extend(pseudo_tty.read_available());
        assert_eq!(renderer.terminal_columns, 18);
        assert!(renderer.current.is_some(), "le resize ne clôt pas le tour");
        assert_eq!(input.lock().unwrap().display(), "commande partielle");

        let wide_text = "界界界界界界界界界🙂";
        assert_eq!(
            visual_rows(&[wide_text.to_string()], 18),
            2,
            "dix scalaires larges occupent vingt cellules"
        );
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 4,
                bytes: journal_record(4, "update", json!({"kind":"text","content":wide_text})),
                live: true,
            }),
            &input,
            &mut output,
        );
        rendered.extend(pseudo_tty.read_available());
        let wide_rows = renderer.rendered_rows;
        renderer.apply(
            RendererCommand::Event(AttachEvent::Journal {
                seq: 5,
                bytes: journal_record(5, "update", json!({"kind":"text","content":" fin"})),
                live: true,
            }),
            &input,
            &mut output,
        );
        let cleared_wide_render = pseudo_tty.read_available();
        assert_eq!(
            cleared_wide_render
                .windows(4)
                .filter(|window| *window == b"\x1b[2K")
                .count(),
            wide_rows,
            "chaque ligne visuelle CJK/emoji doit être effacée avant le redessin"
        );
        rendered.extend(cleared_wide_render);

        assert!(rendered.windows(4).any(|window| window == b"\x1b[2K"));
        let rendered_text = String::from_utf8_lossy(&rendered);
        for fragment in ["réponse live", "après resize", "界界界界", "🙂", " fin"] {
            assert!(
                rendered_text.contains(fragment),
                "fragment absent après wrapping/resize : {fragment}"
            );
        }
        assert!(
            renderer
                .rendered_lines
                .ends_with(&["> commande partiel".into(), "le".into()])
        );
        assert!(
            rendered.ends_with(b"\r\x1b[2C"),
            "curseur après `le`, avant le padding"
        );
    }

    #[test]
    fn spec092_rappel_redessine_le_tampon_tty_avec_statut_et_curseur() {
        let pseudo_tty = PseudoTerminal::open();
        pseudo_tty.set_size(8, 40);
        let mut output = pseudo_tty.slave_writer();
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"brouillon".to_vec(),
            ..InputBuffer::default()
        }));
        {
            let mut input = input.lock().unwrap();
            input.remember(b"A".to_vec());
            input.remember("réponse\nsuite".as_bytes().to_vec());
        }
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let (socket, _) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(socket)));
        let (commands, command_rx) = mpsc::sync_channel(RENDER_COMMAND_CAPACITY);
        let sender = RendererSender {
            commands,
            input_dirty: Arc::new(AtomicBool::new(false)),
            raw_terminal: true,
            tty_output: true,
        };
        let mut renderer = BlockRenderer::new("codex-1".to_string(), true, true);
        renderer.terminal_fd = Some(pseudo_tty.slave);
        (renderer.terminal_columns, renderer.terminal_rows) =
            terminal_geometry(pseudo_tty.slave).unwrap();
        renderer.color = true;
        renderer.apply(
            RendererCommand::Presence(Some(Box::new(presence_fixture())), Instant::now()),
            &input,
            &mut output,
        );
        let _ = pseudo_tty.read_available();

        for byte in b"\x1b[A" {
            handle_input_byte(*byte, &state, &input, &sender, &writer, "codex-1").unwrap();
        }
        while let Ok(command) = command_rx.try_recv() {
            renderer.apply(command, &input, &mut output);
        }
        let rendered = pseudo_tty.read_available();
        assert_eq!(input.lock().unwrap().display(), "réponse\nsuite");
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line == "> réponse")
        );
        assert!(renderer.rendered_lines.iter().any(|line| line == "suite"));
        assert!(
            renderer
                .rendered_lines
                .iter()
                .any(|line| line.contains("Claude Code"))
        );
        assert!(
            rendered
                .windows(SGR_INPUT.len())
                .any(|bytes| bytes == SGR_INPUT.as_bytes())
        );
        assert!(rendered.ends_with(b"\r\x1b[5C"));
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
        let renderer = test_renderer_sender(true, false);
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

            loop {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                let message = decode::<WrapperToDaemon>(line.trim_end()).unwrap();
                if matches!(message, WrapperToDaemon::ListAgents) {
                    // La métadonnée de statut n'est pas le Send attendu :
                    // l'oracle de dernière ligne reste strictement inchangé.
                    writeln!(
                        writer,
                        "{}",
                        encode(&DaemonToWrapper::AgentList { agents: vec![] }).unwrap()
                    )
                    .unwrap();
                    writer.flush().unwrap();
                    continue;
                }
                sent_tx.send(()).unwrap();
                break message;
            }
        });

        let (mut input_writer, input_reader) = UnixStream::pair().unwrap();
        let (input_start_tx, input_start_rx) = mpsc::channel();
        let (event_rendered_tx, event_rendered_rx) = mpsc::channel();
        let input_writer = thread::spawn(move || {
            input_start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            event_rendered_rx
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
            assert_eq!(input_writer.write(b"dernier envoi\r").unwrap(), 14);
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
        let input = Arc::new(Mutex::new(InputBuffer::default()));

        let result = with_raw_terminal(pseudo_tty.slave, pseudo_tty.slave, |raw_terminal| {
            input_start_tx.send(()).unwrap();
            drive_interactive(
                connection,
                &mut state,
                "codex-1",
                Path::new("/tmp/bridget-attach-pty-unused.sock"),
                InteractiveInput {
                    fd: input_reader.as_raw_fd(),
                    buffer: &input,
                    raw_terminal,
                    tty_output: true,
                },
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
            ..InputBuffer::default()
        };
        assert!(input.erase_before_cursor());
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
                            mode: None,
                            location: None,
                        })
                        .unwrap()
                    )
                    .unwrap();
                    writer.flush().unwrap();
                }
            }
        });

        let input = File::open("/dev/null").unwrap();
        let error = run_with_input(
            "codex-1",
            AttachWindow::Today,
            &socket_path,
            input.as_raw_fd(),
            false,
            false,
        )
        .unwrap_err();
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
        // Delta FR-1405 déclaré : seul le préfixe horaire devient local ; le
        // reste de ce golden non-TTY reste comparé octet pour octet.
        let midnight = short_timestamp(Some("2026-08-22T00:00:00Z"));
        assert_eq!(rendered[0], format!("{midnight} alice → bonjour intégral"));
        assert_eq!(rendered[1], format!("{midnight} codex-1 → réponse"));
        assert_eq!(
            rendered[2],
            format!("{midnight} [fin] tour terminé : end_turn — réponse vers alice")
        );

        let permission = std::fs::read_to_string(fixture_root.join("permission.jsonl")).unwrap();
        let permission_time = short_timestamp(Some("2026-08-23T00:00:00Z"));
        assert_eq!(
            render_journal_event(permission.trim_end().as_bytes(), "codex-1"),
            format!("{permission_time} [permission] écrire autorisation décidée : allow-1")
        );
        let error = std::fs::read_to_string(fixture_root.join("error.jsonl")).unwrap();
        let error_time = short_timestamp(Some("2026-08-23T00:00:00Z"));
        assert_eq!(
            render_journal_event(error.trim_end().as_bytes(), "codex-1"),
            format!("{error_time} [erreur] équipier arrêté")
        );
    }

    #[test]
    fn rend_prompt_dispatched_v1_riche_comme_un_message_humain() {
        // Forme v1 réelle attendue après enrichissement des pilotes (from+body),
        // alignée sur le style `turn_start` déjà lisible pour les mandats.
        let line = journal_record(
            42,
            "prompt_dispatched",
            json!({"from": "humain", "body": "salut"}),
        );
        let stamp = short_timestamp(Some("2026-08-23T09:07:00Z"));
        assert_eq!(
            render_journal_event(&line, "coder2"),
            format!("{stamp} humain → salut")
        );
    }

    #[test]
    fn rend_prompt_dispatched_payload_vide_comme_accuse_lisible() {
        // Journaux Codex/Claude observés le 2026-08-24 : payload {}.
        let line = r#"{"v":1,"seq":5845,"ts":"2026-08-24T12:00:43Z","session_id":"s","event":"prompt_dispatched","message_id":"5f793b236d094","payload":{}}"#;
        let rendered = render_journal_event(line.as_bytes(), "coder2");
        let stamp = short_timestamp(Some("2026-08-24T12:00:43Z"));
        assert_eq!(rendered, format!("{stamp} [livré] prompt accepté"));
        assert!(!rendered.contains("non pris en charge"));
    }

    #[test]
    fn test_019_rend_requete_fournisseur_et_correlation_de_lecheance() {
        let request_ref = format!("sha256:{}", "a".repeat(64));
        let turn_ref = format!("sha256:{}", "b".repeat(64));
        let pending = json!({
            "provider": "codex",
            "method": "item/commandExecution/requestApproval",
            "request_id": request_ref,
            "turn_id": turn_ref,
            "state": "pending"
        });
        let request = journal_record(1, "provider_request", pending.clone());
        let error = journal_record(
            2,
            "error",
            json!({
                "reason": "échéance Codex dépassée",
                "pending_provider_request": pending
            }),
        );
        let stamp = short_timestamp(Some("2026-08-23T09:07:00Z"));

        assert_eq!(
            render_journal_event(&request, "coder2"),
            format!(
                "{stamp} [interaction] codex item/commandExecution/requestApproval — en attente — requête {request_ref} — tour {turn_ref}"
            )
        );
        assert_eq!(
            render_journal_event(&error, "coder2"),
            format!(
                "{stamp} [erreur] échéance Codex dépassée — interaction pendante : item/commandExecution/requestApproval — en attente — requête {request_ref} — tour {turn_ref}"
            )
        );
    }

    #[test]
    fn payload_v1_inconnu_reste_honnete_sans_crash() {
        let line = r#"{"v":1,"seq":1,"ts":"2026-08-24T12:00:00Z","session_id":"s","event":"vendor_future","message_id":"m","payload":{"x":1}}"#;
        let rendered = render_journal_event(line.as_bytes(), "coder2");
        let stamp = short_timestamp(Some("2026-08-24T12:00:00Z"));
        assert_eq!(
            rendered,
            format!("{stamp} [événement] vendor_future payload v1 non pris en charge")
        );
    }

    #[test]
    fn update_kind_inconnu_reste_brut_volontairement() {
        let line = r#"{"v":1,"seq":1,"ts":"2026-08-24T12:00:00Z","session_id":"s","event":"update","message_id":"m","payload":{"kind":"thinking","content":"..."}}"#;
        let rendered = render_journal_event(line.as_bytes(), "coder2");
        assert!(rendered.contains("[événement] update"));
        assert!(rendered.contains("payload v1 non pris en charge"));
    }

    /// Propriété : un acte `kind=tool` présent au journal EST affiché dans
    /// l'attach (pas « non pris en charge »). Mutant : égalité stricte à
    /// `tool_call` seule → left contient « non pris en charge », right le nom.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_attach_affiche_un_acte_kind_tool_present_au_journal() {
        let line = r#"{"v":1,"seq":1,"ts":"2026-08-26T18:30:00Z","session_id":"s","event":"update","message_id":"m","payload":{"kind":"tool","text":"Read","tool":"Read","detail":"{\"file_path\":\"/tmp/demo.toml\",\"limit\":1}"}}"#;
        let rendered = render_journal_event(line.as_bytes(), "claude");
        let stamp = short_timestamp(Some("2026-08-26T18:30:00Z"));
        assert_eq!(
            rendered,
            format!("{stamp} [outil] Read {{\"file_path\":\"/tmp/demo.toml\",\"limit\":1}}"),
            "acte tool présent → visible dans attach ; reçu {rendered:?}"
        );
        assert!(
            !rendered.contains("non pris en charge"),
            "ne doit pas tomber dans le bras générique, reçu {rendered:?}"
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
        let hostile_time = short_timestamp(Some("2026-08-22T10:42:00Z"));
        let continuation_indent =
            " ".repeat(format!("{hostile_time} codex-1 →").chars().count() + 1);
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
            rendered,
            vec![
                format!(
                    "{} [outil] Read src/main.rs lecture",
                    short_timestamp(Some("2026-08-23T10:00:00Z"))
                ),
                format!(
                    "{} [outil] Bash cargo test --workspace tests",
                    short_timestamp(Some("2026-08-23T10:00:01Z"))
                ),
                format!(
                    "{} [outil] quantum_wrench kind inconnu",
                    short_timestamp(Some("2026-08-23T10:00:02Z"))
                ),
                format!(
                    "{} [outil] ␛[2J␛]0;pwned␇ ·gnahc titre hostile",
                    short_timestamp(Some("2026-08-23T10:00:03Z"))
                ),
            ]
        );
        assert!(rendered.iter().all(|line| !line.contains('\u{001b}')));
        assert!(rendered.iter().all(|line| !line.contains('\u{202e}')));
    }

    #[test]
    fn golden_delta_tool_call_id_herite_le_titre_sur_toutes_les_mises_a_jour() {
        let fixture = include_str!("../tests/fixtures/attach-tool-call-correlation.jsonl");
        let rendered = fixture
            .lines()
            .map(|line| render_journal_event(line.as_bytes(), "codex-1"))
            .collect::<Vec<_>>();

        // Delta FR-1404/FR-1405 déclaré : les quatre lignes v1 portent le
        // titre corrélé et seule leur heure passe en fuseau local.
        assert_eq!(
            rendered,
            vec![
                format!(
                    "{} [outil] Read src/main.rs lecture",
                    short_timestamp(Some("2026-08-23T10:00:00Z"))
                ),
                format!(
                    "{} [outil] Read src/main.rs analyse",
                    short_timestamp(Some("2026-08-23T10:00:01Z"))
                ),
                format!(
                    "{} [outil] Read src/main.rs lecture du résultat",
                    short_timestamp(Some("2026-08-23T10:00:02Z"))
                ),
                format!(
                    "{} [outil] Read src/main.rs terminé",
                    short_timestamp(Some("2026-08-23T10:00:03Z"))
                ),
            ]
        );
        assert!(rendered.iter().all(|line| !line.contains("inconnu")));
    }

    #[test]
    fn rend_l_heure_locale_dans_un_processus_a_tz_forcee() {
        const PROBE: &str = "BRIDGET_ATTACH_LOCAL_TIME_PROBE";
        if std::env::var_os(PROBE).is_some() {
            assert_eq!(
                short_timestamp(Some("2026-08-23T10:00:00Z")),
                "12:00",
                "Europe/Paris applique l'heure d'été au journal UTC"
            );
            return;
        }

        // `TZ` est global au processus C. La sonde enfant évite une course
        // avec les autres tests parallèles tout en vérifiant le vrai chemin
        // libc::localtime_r employé par le renderer.
        let current_test_binary = std::env::current_exe().unwrap();
        let output = Command::new(current_test_binary)
            .args([
                "--exact",
                "attach::tests::rend_l_heure_locale_dans_un_processus_a_tz_forcee",
                "--nocapture",
            ])
            .env(PROBE, "1")
            .env("TZ", "Europe/Paris")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "sonde TZ enfant en échec : {}",
            String::from_utf8_lossy(&output.stderr)
        );
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
        let sans_journal = AttachRejection {
            reason: AttachRefusal::JournalUnavailable,
            mode: Some(PresenceMode::Acp),
            location: None,
        };
        assert!(
            attach_refusal_message(&sans_journal, "agent-1", &[]).contains("journal append-only")
        );

        let tmux = AttachRejection {
            reason: AttachRefusal::AgentNotAcp,
            mode: Some(PresenceMode::Tmux),
            location: Some("bridget:4.2".to_string()),
        };
        let non_acp = attach_refusal_message(&tmux, "claude-1", &[]);
        assert!(non_acp.contains("interactif tmux"));
        assert!(non_acp.contains("bridget:4.2"));

        let cli = AttachRejection {
            reason: AttachRefusal::AgentNotAcp,
            mode: Some(PresenceMode::Cli),
            location: None,
        };
        assert!(attach_refusal_message(&cli, "cli-1", &[]).contains("mode cli"));

        let unknown_mode = AttachRejection {
            reason: AttachRefusal::AgentNotAcp,
            mode: None,
            location: None,
        };
        assert!(
            attach_refusal_message(&unknown_mode, "legacy-1", &[])
                .contains("mode de présence inconnu")
        );

        let unknown = attach_refusal_message(
            &AttachRejection {
                reason: AttachRefusal::AgentUnknown,
                mode: None,
                location: None,
            },
            "absent",
            &["claude-review".to_string(), "codex-native".to_string()],
        );
        assert!(unknown.contains("équipier « absent » inconnu"));
        assert!(unknown.contains("claude-review, codex-native"));

        let stopped = attach_refusal_message(
            &AttachRejection {
                reason: AttachRefusal::AgentStopped,
                mode: None,
                location: None,
            },
            "codex-1",
            &[],
        );
        assert!(stopped.contains("équipier « codex-1 » arrêté"));
        assert!(stopped.contains("historique"));

        let agent =
            |name: &str, transport: &str, mode: Option<PresenceMode>, state: &str| AgentInfo {
                agent_id: uuid::Uuid::new_v4().to_string(),
                display_name: name.to_string(),
                agent_type: "fixture".to_string(),
                connection_id: format!("conn-{name}"),
                host: "local".to_string(),
                transport: transport.to_string(),
                channel: None,
                mode,
                location: None,
                os: "test".to_string(),
                state: state.to_string(),
                last_seen_secs: 0,
                reconnect_count: 0,
                domain: None,
                model: None,
                effort: None,
                rate_limits: Default::default(),
                model_mismatch: None,
                disk_space: None,
                provider: None,
                persistent: None,
                execution: None,
                agent_link: None,
            };
        let agents = vec![
            agent("connected", "acp", Some(PresenceMode::Acp), "connected"),
            agent("busy", "acp", Some(PresenceMode::Acp), "busy"),
            agent("dnd", "acp", Some(PresenceMode::Acp), "dnd"),
            agent("stopped", "acp", Some(PresenceMode::Acp), "stopped"),
            agent("unreachable", "acp", Some(PresenceMode::Acp), "unreachable"),
            agent("recovering", "acp", Some(PresenceMode::Acp), "recovering"),
            agent("tmux", "acp", Some(PresenceMode::Tmux), "connected"),
            agent("historique", "acp", None, "connected"),
            agent("acp-sur-unix", "unix", Some(PresenceMode::Acp), "connected"),
        ];
        // La sélection retourne les identités routables, pas les noms affichés.
        // Oracle indépendant : connecté/busy/dnd et ACP sur Unix sont les
        // seuls éléments attendus, même si les autres annoncent transport ACP.
        let expected = [0, 1, 2, 8].map(|index| agents[index].agent_id.clone());
        assert_eq!(attachable_agent_names(agents), expected);
    }

    #[test]
    fn spec092_mode_clavier_push_pop_est_symetrique_inerte_et_absent_en_mode_degrade() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let mut raw = RawTerminal::enable_for_fd_with_keyboard(pseudo_tty.slave, true)
            .unwrap()
            .expect("pseudo-TTY détecté");
        assert_eq!(pseudo_tty.read_available(), KEYBOARD_PROTOCOL_PUSH);
        raw.restore().unwrap();
        assert_eq!(pseudo_tty.read_available(), KEYBOARD_PROTOCOL_POP);
        raw.restore().unwrap();
        assert!(pseudo_tty.read_available().is_empty());
        assert_terminal_restored(&before, &pseudo_tty.attrs());

        let dumb_tty = PseudoTerminal::open();
        let mut dumb = RawTerminal::enable_for_fd_with_keyboard(dumb_tty.slave, false)
            .unwrap()
            .expect("pseudo-TTY détecté");
        assert!(dumb_tty.read_available().is_empty());
        dumb.restore().unwrap();
        assert!(dumb_tty.read_available().is_empty());

        let mut pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
        assert!(
            RawTerminal::enable_for_fd_with_keyboard(pipe[0], true)
                .unwrap()
                .is_none()
        );
        assert_eq!(unsafe { libc::close(pipe[0]) }, 0);
        assert_eq!(unsafe { libc::close(pipe[1]) }, 0);
    }

    #[test]
    fn spec092_mode_clavier_exige_stdin_et_stdout_tty() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let mut output_pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(output_pipe.as_mut_ptr()) }, 0);

        let mut raw = RawTerminal::enable_for_fds(pseudo_tty.slave, output_pipe[1])
            .unwrap()
            .expect("stdin pseudo-TTY détecté");
        assert!(pseudo_tty.read_available().is_empty());
        raw.restore().unwrap();
        assert!(pseudo_tty.read_available().is_empty());
        assert_terminal_restored(&before, &pseudo_tty.attrs());
        let flags = unsafe { libc::fcntl(output_pipe[0], libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(output_pipe[0], libc::F_SETFL, flags | libc::O_NONBLOCK) },
            0
        );
        let mut byte = 0_u8;
        assert_eq!(
            unsafe { libc::read(output_pipe[0], (&mut byte as *mut u8).cast(), 1) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().kind(),
            std::io::ErrorKind::WouldBlock,
            "stdout redirigé ne reçoit aucun contrôle clavier"
        );

        assert_eq!(unsafe { libc::close(output_pipe[0]) }, 0);
        assert_eq!(unsafe { libc::close(output_pipe[1]) }, 0);
    }

    #[test]
    fn spec092_us5_garde_double_tty_preserve_cr_lf_et_restaure_les_conversions() {
        let pseudo_tty = PseudoTerminal::open();
        let mut configured = pseudo_tty.attrs();
        let conversions = (libc::ICRNL | libc::INLCR | libc::IGNCR) as libc::tcflag_t;
        configured.c_iflag |= conversions;
        assert_eq!(
            unsafe { libc::tcsetattr(pseudo_tty.slave, libc::TCSANOW, &configured) },
            0,
            "préparation termios: {}",
            std::io::Error::last_os_error()
        );

        let mut raw =
            RawTerminal::enable_for_fds_with_keyboard(pseudo_tty.slave, pseudo_tty.slave, false)
                .unwrap()
                .expect("double pseudo-TTY détecté sans dépendre du mode Kitty");
        assert_eq!(
            pseudo_tty.attrs().c_iflag & conversions,
            0,
            "la garde double-TTY doit neutraliser les trois conversions CR/LF"
        );
        assert_eq!(
            unsafe { libc::write(pseudo_tty.master, b"\n\r".as_ptr().cast(), 2) },
            2
        );
        let mut observed = [0_u8; 2];
        let mut reader = pseudo_tty.slave_reader();
        reader.read_exact(&mut observed).unwrap();
        assert_eq!(observed, *b"\n\r");

        let (write_stream, mut read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer {
            bytes: b"pty".to_vec(),
            ..InputBuffer::default()
        }));
        let renderer = test_renderer_sender(true, true);
        assert!(
            handle_input_byte(observed[0], &state, &input, &renderer, &writer, "codex-1").unwrap()
        );
        assert_eq!(input_snapshot(&input).text, "pty\n");
        read_stream.set_nonblocking(true).unwrap();
        let mut unexpected = [0_u8; 1];
        assert_eq!(
            read_stream.read(&mut unexpected).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        read_stream.set_nonblocking(false).unwrap();
        assert!(
            handle_input_byte(observed[1], &state, &input, &renderer, &writer, "codex-1").unwrap()
        );
        assert_eq!(read_sent_body(read_stream), "pty\n");
        raw.restore().unwrap();
        assert_eq!(pseudo_tty.attrs().c_iflag, configured.c_iflag);
        assert_terminal_restored(&configured, &pseudo_tty.attrs());

        let redirected = PseudoTerminal::open();
        let mut redirected_configured = redirected.attrs();
        redirected_configured.c_iflag |= conversions;
        assert_eq!(
            unsafe { libc::tcsetattr(redirected.slave, libc::TCSANOW, &redirected_configured,) },
            0
        );
        let mut output_pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(output_pipe.as_mut_ptr()) }, 0);
        let mut redirected_raw = RawTerminal::enable_for_fds(redirected.slave, output_pipe[1])
            .unwrap()
            .expect("stdin TTY détecté avec stdout redirigé");
        assert_eq!(redirected.attrs().c_iflag & conversions, conversions);
        redirected_raw.restore().unwrap();
        assert_eq!(redirected.attrs().c_iflag, redirected_configured.c_iflag);
        assert_eq!(unsafe { libc::close(output_pipe[0]) }, 0);
        assert_eq!(unsafe { libc::close(output_pipe[1]) }, 0);
    }

    #[test]
    fn spec092_mode_clavier_ecrit_sur_stdout_et_restaure_apres_echec_activation() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let input = pseudo_tty.slave_reader();
        let input_flags = unsafe { libc::fcntl(input.as_raw_fd(), libc::F_GETFL) };
        assert!(input_flags >= 0);
        assert_eq!(input_flags & libc::O_ACCMODE, libc::O_RDONLY);

        let mut raw =
            RawTerminal::enable_for_fds_with_keyboard(input.as_raw_fd(), pseudo_tty.slave, true)
                .unwrap()
                .expect("stdin pseudo-TTY en lecture seule détecté");
        assert_eq!(pseudo_tty.read_available(), KEYBOARD_PROTOCOL_PUSH);
        raw.restore().unwrap();
        assert_eq!(pseudo_tty.read_available(), KEYBOARD_PROTOCOL_POP);
        assert_terminal_restored(&before, &pseudo_tty.attrs());

        let mut invalid_output = [-1; 2];
        assert_eq!(unsafe { libc::pipe(invalid_output.as_mut_ptr()) }, 0);
        let error =
            RawTerminal::enable_for_fds_with_keyboard(pseudo_tty.slave, invalid_output[0], true)
                .err()
                .expect("le fd de lecture refuse l'écriture du push clavier");
        assert!(error.contains("activation du protocole clavier impossible"));
        assert_terminal_restored(&before, &pseudo_tty.attrs());
        assert_eq!(unsafe { libc::close(invalid_output[0]) }, 0);
        assert_eq!(unsafe { libc::close(invalid_output[1]) }, 0);
    }

    #[test]
    fn spec092_pseudo_tty_livre_shift_entree_fragmente_au_vrai_handler() {
        let pseudo_tty = PseudoTerminal::open();
        let mut raw = RawTerminal::enable_for_fd_with_keyboard(pseudo_tty.slave, true)
            .unwrap()
            .expect("pseudo-TTY détecté");
        assert_eq!(pseudo_tty.read_available(), KEYBOARD_PROTOCOL_PUSH);
        let (write_stream, read_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);

        for injected in b"premiere\x1b[13;2useconde\r" {
            let injected = [*injected];
            assert_eq!(
                unsafe { libc::write(pseudo_tty.master, injected.as_ptr().cast(), 1) },
                1
            );
            let mut observed = 0_u8;
            assert_eq!(
                unsafe { libc::read(pseudo_tty.slave, (&mut observed as *mut u8).cast(), 1) },
                1
            );
            assert!(
                handle_input_byte(observed, &state, &input, &renderer, &writer, "codex-1").unwrap()
            );
        }
        let mut line = String::new();
        BufReader::new(read_stream).read_line(&mut line).unwrap();
        let WrapperToDaemon::Send(message) = decode(line.trim_end()).unwrap() else {
            panic!("Send attendu");
        };
        assert_eq!(message.body, "premiere\nseconde");
        raw.restore().unwrap();
        assert_eq!(pseudo_tty.read_available(), KEYBOARD_PROTOCOL_POP);
    }

    #[test]
    fn spec092_mode_clavier_lit_ctrl_c_et_restaure_push_pop() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let mut raw = RawTerminal::enable_for_fd_with_keyboard(pseudo_tty.slave, true)
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
        assert_eq!(
            pseudo_tty.read_available(),
            [KEYBOARD_PROTOCOL_PUSH, KEYBOARD_PROTOCOL_POP].concat()
        );
    }

    #[test]
    fn spec092_mode_clavier_restaure_push_pop_apres_eof() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let (write_stream, _) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));
        let state = Arc::new(Mutex::new(AttachClientState::new(AttachWindow::Today)));
        let input = Arc::new(Mutex::new(InputBuffer::default()));
        let renderer = test_renderer_sender(true, false);

        {
            let _raw = RawTerminal::enable_for_fd_with_keyboard(pseudo_tty.slave, true)
                .unwrap()
                .expect("pseudo-TTY détecté");
            assert_eq!(
                unsafe { libc::write(pseudo_tty.master, [0x04_u8].as_ptr().cast(), 1) },
                1
            );
            let mut byte = 0_u8;
            assert_eq!(
                unsafe { libc::read(pseudo_tty.slave, (&mut byte as *mut u8).cast(), 1) },
                1
            );
            assert_eq!(byte, 0x04, "Ctrl-D doit arriver à la boucle de saisie");
            assert!(
                !handle_input_byte(byte, &state, &input, &renderer, &writer, "codex-1",).unwrap()
            );
        }
        assert_terminal_restored(&before, &pseudo_tty.attrs());
        assert_eq!(
            pseudo_tty.read_available(),
            [KEYBOARD_PROTOCOL_PUSH, KEYBOARD_PROTOCOL_POP].concat()
        );
    }

    #[test]
    fn spec092_mode_clavier_restaure_push_pop_sur_erreur_et_degrade_un_pipe() {
        let pseudo_tty = PseudoTerminal::open();
        let before = pseudo_tty.attrs();
        let result = (|| {
            let _raw = RawTerminal::enable_for_fd_with_keyboard(pseudo_tty.slave, true)?
                .expect("pseudo-TTY détecté");
            Err::<(), _>("erreur de boucle simulée".to_string())
        })();
        assert_eq!(result.unwrap_err(), "erreur de boucle simulée");
        assert_terminal_restored(&before, &pseudo_tty.attrs());
        assert_eq!(
            pseudo_tty.read_available(),
            [KEYBOARD_PROTOCOL_PUSH, KEYBOARD_PROTOCOL_POP].concat()
        );

        let mut pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
        let result = with_raw_terminal(pipe[0], pipe[1], |interactive| {
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
            let _ = with_raw_terminal(
                pseudo_tty.slave,
                pseudo_tty.slave,
                |_interactive| -> Result<(), String> { panic!("panic de boucle simulée") },
            );
        }));
        assert!(panic.is_err());
        assert_terminal_restored(&before, &pseudo_tty.attrs());
    }
}
