//! Consommation factuelle du flux public Attach de Bridget.
//!
//! Ce module ne connaît ni journal local, ni socket interne, ni état métier
//! Bridget. Il ne transforme que les lignes JSONL versionnées reçues via
//! `Subscribe` en observations corrélées. Une lacune ou une fin de flux reste
//! explicitement observable : aucune conclusion d'état d'agent n'en découle.

use crate::bridget_client::{
    AttachWindow, BridgetClient, BridgetClientError, Subscription, SubscriptionEvent,
};
use crate::domain::{EtatFlux, SourceSnapshot};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fmt;

/// Borne de réassemblage imposée par le contrat Attach session 008.
pub const MAX_REASSEMBLED_EVENT_BYTES: usize = 4 * 1024 * 1024;

/// Vocabulaire fermé des faits runtime affichables par le MVP Maicie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeNature {
    Disponibilite,
    Tour,
    Outil,
    Idle,
    PermissionAutoDecidee,
}

/// Fait brut, daté et corrélé issu d'une ligne journal v1 publique.
///
/// `details` exclut notamment les corps de messages et de réponses : Maicie
/// doit constater une activité, pas recopier le journal ACP ni l'interpréter.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeObservation {
    pub agent: String,
    pub source: SourceSnapshot,
    pub nature: RuntimeNature,
    pub observed_at: String,
    pub session_id: String,
    pub message_id: Option<String>,
    pub subscription_id: String,
    pub seq: u64,
    pub proof_ref: String,
    pub stream_state: EtatFlux,
    pub details: Value,
}

/// Signaux non ambigus du plan de contrôle Attach.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeSignal {
    Observation(RuntimeObservation),
    SnapshotCaughtUp {
        subscription_id: String,
        through_seq: Option<u64>,
    },
    Gap {
        subscription_id: String,
        from_seq: u64,
        to_seq: u64,
        reason: Option<String>,
    },
    JournalReadError {
        subscription_id: String,
        line: u64,
        offset: u64,
        reason: String,
    },
    End {
        subscription_id: String,
        reason: String,
    },
}

#[derive(Debug)]
pub enum RuntimeError {
    Transport(BridgetClientError),
    Protocol(String),
    Fragment(String),
    Journal(String),
    Ended,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => {
                write!(formatter, "flux Attach Bridget indisponible: {error}")
            }
            Self::Protocol(detail) => write!(formatter, "protocole Attach invalide: {detail}"),
            Self::Fragment(detail) => write!(formatter, "fragment Attach invalide: {detail}"),
            Self::Journal(detail) => write!(formatter, "ligne journal v1 invalide: {detail}"),
            Self::Ended => formatter.write_str("abonnement Attach déjà terminé"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<BridgetClientError> for RuntimeError {
    fn from(error: BridgetClientError) -> Self {
        Self::Transport(error)
    }
}

#[derive(Debug)]
struct PendingFragments {
    seq: u64,
    next_offset: u64,
    bytes: Vec<u8>,
}

/// Consommateur d'une génération unique d'abonnement Attach.
pub struct RuntimeSubscription {
    agent: String,
    initial_window: AttachWindow,
    subscription_id: String,
    subscription: Subscription,
    pending: Option<PendingFragments>,
    discarding_seq: Option<u64>,
    last_seq: Option<u64>,
    stream_state: EtatFlux,
}

impl RuntimeSubscription {
    /// Ouvre la vue publique et attend sa confirmation explicite. Aucun état
    /// runtime n'est créé avant le `Subscribed` du daemon.
    pub fn open(
        client: &BridgetClient,
        agent: impl Into<String>,
        initial_window: AttachWindow,
    ) -> Result<Self, RuntimeError> {
        let agent = agent.into();
        if agent.trim().is_empty() {
            return Err(RuntimeError::Protocol("agent Attach vide".to_string()));
        }
        let mut subscription = client.subscribe(&agent, initial_window.clone())?;
        let subscription_id = match subscription.next_event()? {
            SubscriptionEvent::Subscribed { subscription_id } => subscription_id,
            event => {
                return Err(RuntimeError::Protocol(format!(
                    "Subscribed attendu avant tout flux, reçu {event:?}"
                )));
            }
        };
        Ok(Self {
            agent,
            initial_window,
            subscription_id,
            subscription,
            pending: None,
            discarding_seq: None,
            last_seq: None,
            stream_state: EtatFlux::Unavailable,
        })
    }

    pub fn subscription_id(&self) -> &str {
        &self.subscription_id
    }

    pub fn stream_state(&self) -> EtatFlux {
        self.stream_state
    }

    /// Détermine le sélecteur de reprise sans convertir lui-même les dates ou
    /// « aujourd'hui » : cette résolution reste du ressort du wrapper Bridget.
    pub fn resume_window(&self) -> Result<AttachWindow, RuntimeError> {
        match self.last_seq {
            Some(sequence) => sequence
                .checked_add(1)
                .map(AttachWindow::Seq)
                .ok_or_else(|| {
                    RuntimeError::Protocol("seq maximal sans reprise possible".to_string())
                }),
            None => Ok(self.initial_window.clone()),
        }
    }

    /// Lit jusqu'au prochain fait ou signal utile. Les trames d'une ancienne
    /// génération et les continuations d'un événement trop grand sont
    /// volontairement consommées sans devenir des observations.
    pub fn next_signal(&mut self) -> Result<RuntimeSignal, RuntimeError> {
        if self.stream_state == EtatFlux::Ended {
            return Err(RuntimeError::Ended);
        }

        loop {
            let event = self.subscription.next_event()?;
            if !matches_subscription(&event, &self.subscription_id) {
                continue;
            }
            if let Some(signal) = self.handle_event(event)? {
                return Ok(signal);
            }
        }
    }

    fn handle_event(
        &mut self,
        event: SubscriptionEvent,
    ) -> Result<Option<RuntimeSignal>, RuntimeError> {
        match event {
            SubscriptionEvent::Subscribed { .. } => Err(RuntimeError::Protocol(
                "Subscribed répété dans une génération active".to_string(),
            )),
            SubscriptionEvent::JournalFragment {
                subscription_id,
                seq,
                offset,
                final_fragment,
                bytes,
            } => self.handle_fragment(subscription_id, seq, offset, final_fragment, bytes),
            SubscriptionEvent::SnapshotCaughtUp {
                subscription_id,
                through_seq,
            } => {
                if self.stream_state == EtatFlux::Unavailable {
                    self.stream_state = EtatFlux::Fresh;
                }
                Ok(Some(RuntimeSignal::SnapshotCaughtUp {
                    subscription_id,
                    through_seq,
                }))
            }
            SubscriptionEvent::Gap {
                subscription_id,
                from_seq,
                to_seq,
                reason,
            } => {
                if from_seq > to_seq {
                    return Err(RuntimeError::Protocol("Gap inversé".to_string()));
                }
                self.pending = None;
                self.discarding_seq = None;
                self.stream_state = EtatFlux::Gap;
                Ok(Some(RuntimeSignal::Gap {
                    subscription_id,
                    from_seq,
                    to_seq,
                    reason,
                }))
            }
            SubscriptionEvent::JournalReadError {
                subscription_id,
                line,
                offset,
                reason,
            } => {
                self.pending = None;
                self.discarding_seq = None;
                self.stream_state = EtatFlux::Gap;
                Ok(Some(RuntimeSignal::JournalReadError {
                    subscription_id,
                    line,
                    offset,
                    reason,
                }))
            }
            SubscriptionEvent::End {
                subscription_id,
                reason,
            } => {
                self.pending = None;
                self.discarding_seq = None;
                self.stream_state = EtatFlux::Ended;
                Ok(Some(RuntimeSignal::End {
                    subscription_id,
                    reason,
                }))
            }
        }
    }

    fn handle_fragment(
        &mut self,
        subscription_id: String,
        seq: u64,
        offset: u64,
        final_fragment: bool,
        bytes: Vec<u8>,
    ) -> Result<Option<RuntimeSignal>, RuntimeError> {
        if let Some(discarding_seq) = self.discarding_seq {
            if seq != discarding_seq {
                return Err(RuntimeError::Fragment(format!(
                    "seq {seq} reçu avant la fin du seq abandonné {discarding_seq}"
                )));
            }
            if final_fragment {
                self.discarding_seq = None;
            }
            return Ok(None);
        }

        let pending = match self.pending.take() {
            Some(pending) => {
                if pending.seq != seq || pending.next_offset != offset {
                    return Err(RuntimeError::Fragment(format!(
                        "fragment seq={seq} offset={offset} non contigu après seq={} offset={}",
                        pending.seq, pending.next_offset
                    )));
                }
                pending
            }
            None => {
                if offset != 0 {
                    return Err(RuntimeError::Fragment(format!(
                        "premier fragment seq={seq} avec offset non nul {offset}"
                    )));
                }
                PendingFragments {
                    seq,
                    next_offset: 0,
                    bytes: Vec::new(),
                }
            }
        };

        if pending.bytes.len().saturating_add(bytes.len()) > MAX_REASSEMBLED_EVENT_BYTES {
            self.stream_state = EtatFlux::Gap;
            if !final_fragment {
                self.discarding_seq = Some(seq);
            }
            return Ok(Some(RuntimeSignal::Gap {
                subscription_id,
                from_seq: seq,
                to_seq: seq,
                reason: Some("event_too_large".to_string()),
            }));
        }

        let mut pending = pending;
        pending.next_offset = pending
            .next_offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| {
                RuntimeError::Fragment("taille de fragment hors plage u64".to_string())
            })?)
            .ok_or_else(|| RuntimeError::Fragment("offset de fragment débordé".to_string()))?;
        pending.bytes.extend_from_slice(&bytes);

        if !final_fragment {
            self.pending = Some(pending);
            return Ok(None);
        }

        let mut observation =
            observation_from_journal_line(&self.agent, &subscription_id, seq, &pending.bytes)?;
        if let Some(previous) = self.last_seq
            && seq <= previous
        {
            return Err(RuntimeError::Protocol(format!(
                "seq non strictement croissant: {seq} après {previous}"
            )));
        }
        self.last_seq = Some(seq);
        observation.stream_state = self.stream_state;
        // Une ligne valide après une lacune ne répare pas rétroactivement le
        // flux : seule une nouvelle souscription, avec son propre
        // `SnapshotCaughtUp`, peut établir une vue fraîche. Cette génération
        // conserve donc explicitement `Gap` jusqu'à sa fin.
        Ok(Some(RuntimeSignal::Observation(observation)))
    }
}

fn matches_subscription(event: &SubscriptionEvent, expected: &str) -> bool {
    match event {
        SubscriptionEvent::Subscribed { subscription_id }
        | SubscriptionEvent::SnapshotCaughtUp {
            subscription_id, ..
        }
        | SubscriptionEvent::Gap {
            subscription_id, ..
        }
        | SubscriptionEvent::JournalReadError {
            subscription_id, ..
        }
        | SubscriptionEvent::End {
            subscription_id, ..
        }
        | SubscriptionEvent::JournalFragment {
            subscription_id, ..
        } => subscription_id == expected,
    }
}

#[derive(Debug, Deserialize)]
struct JournalEntryV1 {
    v: u8,
    seq: u64,
    ts: String,
    session_id: String,
    event: String,
    #[serde(default)]
    message_id: Option<String>,
    payload: Value,
}

fn observation_from_journal_line(
    agent: &str,
    subscription_id: &str,
    fragment_seq: u64,
    bytes: &[u8],
) -> Result<RuntimeObservation, RuntimeError> {
    let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let entry: JournalEntryV1 = serde_json::from_slice(bytes)
        .map_err(|error| RuntimeError::Journal(format!("JSON v1 illisible: {error}")))?;
    if entry.v != 1 {
        return Err(RuntimeError::Journal(format!(
            "version v{} non prise en charge",
            entry.v
        )));
    }
    if entry.seq != fragment_seq {
        return Err(RuntimeError::Journal(format!(
            "seq de ligne {} différent du fragment {fragment_seq}",
            entry.seq
        )));
    }
    if entry.ts.trim().is_empty() || entry.session_id.trim().is_empty() {
        return Err(RuntimeError::Journal(
            "timestamp ou session_id vide".to_string(),
        ));
    }
    let (nature, details) = minimized_details(&entry.event, &entry.payload)?;
    Ok(RuntimeObservation {
        agent: agent.to_string(),
        source: SourceSnapshot::AcpSubscription,
        nature,
        observed_at: entry.ts,
        session_id: entry.session_id,
        message_id: entry.message_id,
        subscription_id: subscription_id.to_string(),
        seq: entry.seq,
        proof_ref: format!("{subscription_id}:{}", entry.seq),
        stream_state: EtatFlux::Fresh,
        details,
    })
}

fn minimized_details(event: &str, payload: &Value) -> Result<(RuntimeNature, Value), RuntimeError> {
    let object = payload
        .as_object()
        .ok_or_else(|| RuntimeError::Journal(format!("payload {event} non objet")))?;
    match event {
        "turn_start" => {
            let from = required_payload_string(object, "from", event)?;
            let reply = required_payload_bool(object, "reply", event)?;
            Ok((
                RuntimeNature::Tour,
                json!({"event": event, "from": from, "reply": reply}),
            ))
        }
        "update" => {
            let kind = required_payload_string(object, "kind", event)?;
            let nature = match kind.as_str() {
                "text" => RuntimeNature::Tour,
                "tool_call" => RuntimeNature::Outil,
                _ => {
                    return Err(RuntimeError::Journal(format!("kind update inconnu {kind}")));
                }
            };
            Ok((nature, json!({"event": event, "kind": kind})))
        }
        "permission" => {
            let decision = object
                .get("decision")
                .filter(|value| value.is_object())
                .ok_or_else(|| {
                    RuntimeError::Journal("permission sans décision typée".to_string())
                })?;
            let outcome = decision
                .get("outcome")
                .and_then(Value::as_str)
                .filter(|outcome| matches!(*outcome, "selected" | "cancelled"))
                .ok_or_else(|| RuntimeError::Journal("outcome permission invalide".to_string()))?;
            let mut details = serde_json::Map::new();
            details.insert("event".to_string(), Value::String(event.to_string()));
            details.insert("outcome".to_string(), Value::String(outcome.to_string()));
            if let Some(option_id) = decision.get("option_id").and_then(Value::as_str) {
                details.insert(
                    "option_id".to_string(),
                    Value::String(option_id.to_string()),
                );
            }
            Ok((RuntimeNature::PermissionAutoDecidee, Value::Object(details)))
        }
        "turn_end" => {
            let stop_reason = required_payload_string(object, "stop_reason", event)?;
            let mut details = serde_json::Map::new();
            details.insert("event".to_string(), Value::String(event.to_string()));
            details.insert("stop_reason".to_string(), Value::String(stop_reason));
            if let Some(routed_to) = object.get("routed_to").and_then(Value::as_str) {
                details.insert(
                    "routed_to".to_string(),
                    Value::String(routed_to.to_string()),
                );
            }
            Ok((RuntimeNature::Tour, Value::Object(details)))
        }
        "error" => {
            let reason = required_payload_string(object, "reason", event)?;
            // Le journal atteste seulement l'erreur d'un tour : il ne permet
            // pas de déduire que l'agent est devenu indisponible.
            Ok((
                RuntimeNature::Tour,
                json!({"event": event, "reason": reason}),
            ))
        }
        _ => Err(RuntimeError::Journal(format!("event v1 inconnu {event}"))),
    }
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
    event: &str,
) -> Result<String, RuntimeError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| RuntimeError::Journal(format!("{event}.{field} absent ou invalide")))
}

fn required_payload_bool(
    payload: &serde_json::Map<String, Value>,
    field: &str,
    event: &str,
) -> Result<bool, RuntimeError> {
    payload
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| RuntimeError::Journal(format!("{event}.{field} absent ou invalide")))
}
