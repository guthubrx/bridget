//! Client de vue attach : connexion persistante, reprise et rendu sûr du journal.

use bridget_transport::protocol::{
    AttachRefusal, AttachWindow, ConnectionRole, DaemonToWrapper,
    MAX_ATTACH_SERIALIZED_FRAME_BYTES, WrapperToDaemon, decode, encode,
};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const MAX_REASSEMBLY_BYTES: usize = 4 * 1024 * 1024;
const MAX_RENDERED_EVENT_CHARS: usize = 16 * 1024;
const MAX_RENDERED_LABEL_CHARS: usize = 160;
const MAX_CONSECUTIVE_COMBINING_MARKS: usize = 8;
const RECONNECT_DELAY: Duration = Duration::from_millis(250);
const RETIRED_SUBSCRIPTIONS_LIMIT: usize = 64;

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
    reassembly: Option<Reassembly>,
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
            reassembly: None,
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
            DaemonToWrapper::Nack { id, .. } if self.pending_send.remove(&id).is_some() => {
                outcome.events.push(AttachEvent::SendRejected {
                    message_id: id,
                    delayed: false,
                });
            }
            DaemonToWrapper::DeliveryRejected { id, .. }
                if self.pending_send.remove(&id).is_some() =>
            {
                outcome.events.push(AttachEvent::SendRejected {
                    message_id: id,
                    delayed: true,
                });
            }
            DaemonToWrapper::Disconnect => outcome.reconnect = true,
            message if message.allowed_for_attach() => {}
            _ => return Err("message interdit sur une connexion attach".to_string()),
        }
        Ok(outcome)
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
        loop {
            let Some(frame) = read_bounded_frame(&mut self.reader)? else {
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
    let mut state = AttachClientState::new(initial_window);
    let mut connected_once = false;

    loop {
        let attempt = AttachConnection::connect(socket_path).and_then(|mut connection| {
            connection.accept_role()?;
            let window = state.subscription_requested();
            connection.subscribe(agent, window)?;
            Ok(connection)
        });
        let mut connection = match attempt {
            Ok(connection) => connection,
            Err(error) if !connected_once => return Err(error),
            Err(_) => {
                thread::sleep(RECONNECT_DELAY);
                continue;
            }
        };
        connected_once = true;
        let mut renderer = |event: &AttachEvent| print_transport_event(event, agent);
        drive_connection(
            &mut connection,
            &mut state,
            agent,
            socket_path,
            &mut renderer,
        )?;

        state.connection_closed();
        thread::sleep(RECONNECT_DELAY);
    }
}

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

fn print_transport_event(event: &AttachEvent, agent: &str) {
    let rendered = render_attach_event(event, agent);
    match event {
        AttachEvent::Journal { .. } => println!("{rendered}"),
        _ => eprintln!("{rendered}"),
    }
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
        } => {
            let phase = if *delayed { "différé" } else { "immédiat" };
            format!(
                "attach: envoi {} rejeté ({phase})",
                sanitize_inline(message_id)
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
            (
                format!(
                    "[outil] {}",
                    payload
                        .get("tool")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("inconnu")
                ),
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
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::sync::mpsc;

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
            }]
        );
        assert!(state.pending_send.is_empty());
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
}
