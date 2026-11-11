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

/// Fenêtre humaine d'une tâche. Le chemin est résolu uniquement par le daemon.
pub(crate) fn lineage_journal_page(
    directory: &Path,
    task_id: &str,
    after: u64,
    limit: u32,
) -> Result<serde_json::Value, bridget_transport::protocol::HumanLineageError> {
    use bridget_transport::journal::{
        IncrementalJournalReader, JournalReadItem, current_host_date, resolve_window,
    };
    use bridget_transport::protocol::HumanLineageError as E;
    if !(1..=100).contains(&limit) {
        return Err(E::InvalidRequest);
    }
    crate::environment::validate_private_directory_if_present(directory)
        .map_err(|_| E::JournalUnavailable)?;
    let window = resolve_window(
        directory,
        &AttachWindow::Seq(after.saturating_add(1)),
        &current_host_date(),
    )
    .map_err(|_| E::JournalUnavailable)?;
    if window.files.is_empty() {
        return Err(E::JournalUnavailable);
    }
    if window.files.len() > 256 {
        return Err(E::ResourceLimit);
    }
    let mut events = Vec::new();
    let mut bytes = 0usize;
    let mut next = after;
    let mut gap = None;
    let mut scanned = 0usize;
    for file in window.files {
        crate::environment::validate_state_file(&file, false).map_err(|_| E::JournalUnavailable)?;
        let mut reader = IncrementalJournalReader::new(file);
        loop {
            let before = reader.next_offset();
            let items = reader
                .read_chunk(16 * 1024)
                .map_err(|_| E::JournalUnavailable)?;
            scanned += reader.next_offset().saturating_sub(before) as usize;
            if scanned > 64 * 1024 * 1024 {
                return Err(E::ResourceLimit);
            }
            for item in items {
                match item {
                    JournalReadItem::Event(entry) if entry.seq > after => {
                        if events.len() >= limit as usize
                            || (!events.is_empty() && bytes + entry.bytes.len() > 16 * 1024)
                        {
                            return Ok(lineage_journal_result(task_id, events, next, false, gap));
                        }
                        if entry.seq > next.saturating_add(1) {
                            gap = Some(
                                serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":entry.seq-1,"reason":"journal_gap"}),
                            );
                            return Ok(lineage_journal_result(
                                task_id,
                                events,
                                entry.seq - 1,
                                false,
                                gap,
                            ));
                        }
                        next = entry.seq;
                        if entry.bytes.len() > 16 * 1024 {
                            gap = Some(
                                serde_json::json!({"from_seq":entry.seq,"to_seq":entry.seq,"reason":"entry_too_large"}),
                            );
                            return Ok(lineage_journal_result(task_id, events, next, false, gap));
                        }
                        let value: serde_json::Value = serde_json::from_slice(&entry.bytes)
                            .map_err(|_| E::JournalUnavailable)?;
                        if value["seq"].as_u64() != Some(entry.seq) {
                            return Err(E::JournalUnavailable);
                        }
                        bytes += entry.bytes.len();
                        events.push(value);
                    }
                    JournalReadItem::Unreadable(_) => return Err(E::JournalUnavailable),
                    JournalReadItem::Oversized { seq: Some(seq), .. } if seq > after => {
                        gap = Some(
                            serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":seq,"reason":"entry_too_large"}),
                        );
                        return Ok(lineage_journal_result(
                            task_id,
                            events,
                            next.max(seq),
                            false,
                            gap,
                        ));
                    }
                    JournalReadItem::Oversized { seq: None, .. } => {
                        return Err(E::JournalUnavailable);
                    }
                    _ => {}
                }
            }
            if reader.next_offset() == before {
                break;
            }
        }
    }
    Ok(lineage_journal_result(task_id, events, next, true, gap))
}

pub(crate) fn lineage_journal_result(
    task_id: &str,
    events: Vec<serde_json::Value>,
    next: u64,
    caught_up: bool,
    gap: Option<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({"version":1,"status":"ok","task_id":task_id,"events":events,"next_seq":next,"caught_up":caught_up,"gap":gap})
}

/// Réutilise la négociation/reconstruction attach et le vrai AttachRelay du wrapper.
pub(crate) fn lineage_journal_follow(
    socket: &Path,
    child: &str,
    task_id: &str,
    after: u64,
    _limit: u32,
    mut opened: impl FnMut(&UnixStream) -> Result<(), bridget_transport::protocol::HumanLineageError>,
    mut emit: impl FnMut(
        serde_json::Value,
    ) -> Result<(), bridget_transport::protocol::HumanLineageError>,
) -> Result<(), bridget_transport::protocol::HumanLineageError> {
    use crate::communication::client::DaemonConnection;
    use bridget_transport::protocol::HumanLineageError as E;
    let mut connection = DaemonConnection::connect(socket).map_err(|_| E::JournalUnavailable)?;
    if !matches!(
        connection
            .exchange(&WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            })
            .map_err(|_| E::JournalUnavailable)?,
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        }
    ) {
        return Err(E::JournalUnavailable);
    }
    opened(connection.read_stream())?;
    let window = AttachWindow::Seq(after.saturating_add(1));
    let mut state = AttachClientState::new(window.clone());
    state.subscription_requested();
    let mut response = connection
        .exchange(&WrapperToDaemon::Subscribe {
            agent: child.into(),
            window,
        })
        .map_err(|_| E::JournalUnavailable)?;
    let mut next = after;
    let mut caught_up = false;
    loop {
        let outcome = state
            .dispatch(response)
            .map_err(|_| E::JournalUnavailable)?;
        if outcome.rejected.is_some() || outcome.reconnect || outcome.resubscribe.is_some() {
            return Err(E::JournalUnavailable);
        }
        for event in outcome.events {
            match event {
                AttachEvent::Journal { seq, bytes, .. } if seq > next => {
                    let gap = if seq > next.saturating_add(1) {
                        Some(
                            serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":seq-1,"reason":"journal_gap"}),
                        )
                    } else {
                        None
                    };
                    next = seq;
                    if bytes.len() > 16 * 1024 {
                        emit(lineage_journal_result(
                            task_id,
                            Vec::new(),
                            next,
                            caught_up,
                            Some(
                                serde_json::json!({"from_seq":seq,"to_seq":seq,"reason":"entry_too_large"}),
                            ),
                        ))?;
                    } else {
                        let value: serde_json::Value =
                            serde_json::from_slice(&bytes).map_err(|_| E::JournalUnavailable)?;
                        if value["seq"].as_u64() != Some(seq) {
                            return Err(E::JournalUnavailable);
                        }
                        emit(lineage_journal_result(
                            task_id,
                            vec![value],
                            next,
                            caught_up,
                            gap,
                        ))?;
                    }
                }
                AttachEvent::Gap {
                    from_seq,
                    to_seq,
                    reason,
                } => {
                    // Le wrapper recharge cette plage depuis son journal froid.
                    // Ne pas avancer le curseur avant les fragments rejoués.
                    if reason.as_deref() == Some("live_feed_overrun") {
                        caught_up = false;
                        continue;
                    }
                    next = next.max(to_seq);
                    emit(lineage_journal_result(
                        task_id,
                        Vec::new(),
                        next,
                        caught_up,
                        Some(
                            serde_json::json!({"from_seq":from_seq,"to_seq":to_seq,"reason":reason.unwrap_or_else(||"journal_gap".into()).chars().take(160).collect::<String>()}),
                        ),
                    ))?;
                }
                AttachEvent::SnapshotCaughtUp { through_seq } => {
                    if state.reassembly.is_some() {
                        return Err(E::JournalUnavailable);
                    }
                    next = next.max(through_seq.unwrap_or(next));
                    caught_up = true;
                    emit(lineage_journal_result(
                        task_id,
                        Vec::new(),
                        next,
                        true,
                        None,
                    ))?;
                }
                AttachEvent::JournalReadError { .. } => return Err(E::JournalUnavailable),
                AttachEvent::End { .. } => {
                    return if caught_up {
                        Ok(())
                    } else {
                        Err(E::JournalUnavailable)
                    };
                }
                _ => {}
            }
        }
        response = connection
            .read_stream_response(MAX_ATTACH_SERIALIZED_FRAME_BYTES)
            .map_err(|_| E::JournalUnavailable)?;
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JournalRequest {
    pub agent: String,
    pub tail: Option<u64>,
    pub from_seq: Option<u64>,
    pub to: Option<String>,
    #[serde(default)]
    pub reply: bool,
    #[serde(
        default,
        deserialize_with = "crate::communication::client::deserialize_project_reason"
    )]
    pub cross_project_reason: Option<String>,
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
            || (self.cross_project_reason.is_some() && self.to.is_none())
        {
            return Err(ClientError::InvalidParams(
                "tail/from_seq exclusifs ; reply et cross_project_reason exigent to non vide"
                    .into(),
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
    #[serde(skip)]
    snapshot_caught_up: bool,
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
            snapshot_caught_up: false,
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
                    excerpt.snapshot_caught_up = true;
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
/// O(octets), instantané Attach existant <=60 Kio ; page mobile <=16 Kio et 100 entrées.
pub(crate) fn lineage_journal_remote_page(
    socket: &Path,
    agent: &str,
    after: u64,
    limit: u32,
) -> Result<serde_json::Value, bridget_transport::protocol::HumanLineageError> {
    use bridget_transport::protocol::HumanLineageError as E;
    if !(1..=100).contains(&limit) {
        return Err(E::InvalidRequest);
    }
    let excerpt = read_journal_excerpt(socket, agent, AttachWindow::Seq(after.saturating_add(1)))
        .map_err(|_| E::JournalUnavailable)?;
    if excerpt
        .notices
        .iter()
        .any(|n| !n.starts_with("journal_gap:") && !n.starts_with("excerpt_limit:"))
    {
        return Err(E::JournalUnavailable);
    }
    let mut events = Vec::new();
    let mut bytes = 0usize;
    let mut next = after;
    for event in excerpt.entries {
        let seq = event["seq"].as_u64().ok_or(E::JournalUnavailable)?;
        if seq <= next {
            continue;
        }
        if seq > next.saturating_add(1) {
            return Ok(lineage_journal_result(
                agent,
                events,
                seq - 1,
                false,
                Some(
                    serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":seq-1,"reason":"journal_gap"}),
                ),
            ));
        }
        let size = serde_json::to_vec(&event)
            .map_err(|_| E::JournalUnavailable)?
            .len();
        if events.len() >= limit as usize || !events.is_empty() && bytes + size > 16 * 1024 {
            return Ok(lineage_journal_result(agent, events, next, false, None));
        }
        if size > 16 * 1024 {
            return Ok(lineage_journal_result(
                agent,
                events,
                seq,
                false,
                Some(serde_json::json!({"from_seq":seq,"to_seq":seq,"reason":"entry_too_large"})),
            ));
        }
        bytes += size;
        next = seq;
        events.push(event);
    }
    if let Some(through) = excerpt.through_seq.filter(|v| *v > next) {
        return Ok(lineage_journal_result(
            agent,
            events,
            through,
            true,
            Some(
                serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":through,"reason":"journal_gap"}),
            ),
        ));
    }
    if events.is_empty()
        && let Some(seq) = excerpt.next_seq.filter(|v| *v > next)
    {
        return Ok(lineage_journal_result(
            agent,
            events,
            seq,
            false,
            Some(
                serde_json::json!({"from_seq":next.saturating_add(1),"to_seq":seq,"reason":"entry_too_large"}),
            ),
        ));
    }
    // Une confirmation sans séquence signifie un instantané vide, pas un suivi terminé.
    Ok(lineage_journal_result(
        agent,
        events,
        next,
        excerpt.snapshot_caught_up,
        None,
    ))
}

/// Lit une seule entrée de fin via Attach, sans charger l’historique dans le client.
pub(crate) fn journal_tail_start(
    socket: &Path,
    agent: &str,
    limit: u32,
) -> Result<u64, bridget_transport::protocol::HumanLineageError> {
    use bridget_transport::protocol::HumanLineageError as E;
    if !(1..=50).contains(&limit) {
        return Err(E::InvalidRequest);
    }
    let excerpt = read_journal_excerpt(socket, agent, AttachWindow::Tail(1))
        .map_err(|_| E::JournalUnavailable)?;
    if excerpt
        .notices
        .iter()
        .any(|n| !n.starts_with("excerpt_limit:"))
    {
        return Err(E::JournalUnavailable);
    }
    Ok(excerpt
        .last_seq
        .or(excerpt.through_seq)
        .or(excerpt.next_seq)
        .unwrap_or(0)
        .saturating_sub(limit as u64))
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
struct RawTerminal {
    fd: RawFd,
    original: libc::termios,
    keyboard_fd: Option<RawFd>,
    restored: bool,
}

#[expect(
    dead_code,
    reason = "code réservé à la boucle T806b, conservé par décision D12 (programme 260) : API utilisée par la boucle T806b et les pseudo-TTY de T806a"
)]
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
        // SAFETY: isatty ne déréférence aucun pointeur ; ses arguments sont des entiers.
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
        // SAFETY: les pointeurs passés à tcgetattr désignent des variables locales valides pendant l'appel.
        if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
            return Err(format!(
                "lecture termios impossible: {}",
                std::io::Error::last_os_error()
            ));
        }
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
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
        // SAFETY: les pointeurs passés à tcsetattr désignent des variables locales valides pendant l'appel.
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
            // SAFETY: les pointeurs passés à tcsetattr désignent des variables locales valides pendant l'appel.
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
            // SAFETY: les pointeurs passés à tcsetattr désignent des variables locales valides pendant l'appel.
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
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
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
            DaemonToWrapper::Ack { id, .. } if self.pending_send.contains_key(&id) => {
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
    // SAFETY: isatty ne déréférence aucun pointeur ; ses arguments sont des entiers.
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
    // SAFETY: les pointeurs passés à ioctl désignent des variables locales valides pendant l'appel.
    if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, size.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
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
        // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
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
                // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
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
                // SAFETY: `more` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
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
        // SAFETY: les pointeurs passés à localtime_r désignent des variables locales valides pendant l'appel.
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
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        ..unsafe { std::mem::zeroed() }
    };
    // SAFETY: les pointeurs passés à timegm désignent des variables locales valides pendant l'appel.
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
        turn_attest: false,
        queue_fallback: false,
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
#[path = "attach_tests.rs"]
mod tests;
