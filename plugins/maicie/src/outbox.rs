//! Enveloppes durables de delegation Maicie.
//!
//! Ce module prepare les octets immuables avant toute I/O Bridget. Le store
//! demeure l'unique autorite de transition ; le futur reconciliateur T008 ne
//! recevra que des snapshots complets, sans resolution implicite de cible.

use crate::bridget_client::{BridgetClientError, PublicMessage, validate_send_idempotent_frame};
use crate::domain::{
    Delegation, EtatDelegation, EtatObjectif, EtatOutboxDelegation, ObjectifCoordonne,
    OutboxDelegation,
};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

pub const MAX_MESSAGE_BYTES: usize = 256 * 1024;

/// Agregat cree dans une transaction unique : decision, delegation et outbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedDelegation {
    pub objective: ObjectifCoordonne,
    pub delegation: Delegation,
    pub outbox: OutboxDelegation,
    pub issuer_scope: String,
    pub issued_at: i64,
    pub message_bytes: Vec<u8>,
}

impl PreparedDelegation {
    /// Construit les octets filaires une seule fois, avant toute persistence ou
    /// connexion Bridget. La borne est celle du client runtime qui remettra
    /// cette outbox ; aucune valeur implicite ne peut diverger de ce transport.
    pub fn new(
        objective: ObjectifCoordonne,
        delegation: Delegation,
        outbox: OutboxDelegation,
        issuer_scope: impl Into<String>,
        issued_at: i64,
        max_frame_bytes: usize,
    ) -> Result<Self, OutboxError> {
        let body = std::str::from_utf8(&outbox.body_bytes)
            .map_err(|_| OutboxError::Invalid("body_bytes doit etre UTF-8"))?;
        let message = PublicMessage {
            id: outbox.message_id.to_string(),
            from: crate::MAICIE_IDENTITY.to_string(),
            to: outbox.target.clone(),
            body: body.to_string(),
            reply: outbox.reply,
            hops: 4,
            reply_timeout: outbox.reply.then_some(outbox.timeout_secs),
            deadline_at: u64::try_from(outbox.deadline_contractuelle).ok(),
            in_reply_to: None,
        };
        let message_bytes = serde_json::to_vec(&message).map_err(OutboxError::Encode)?;
        validate_send_idempotent_frame(
            &message,
            &outbox.message_id.to_string(),
            issued_at,
            max_frame_bytes,
        )
        .map_err(OutboxError::Frame)?;
        let prepared = Self {
            objective,
            delegation,
            outbox,
            issuer_scope: issuer_scope.into(),
            issued_at,
            message_bytes,
        };
        prepared.validate()?;
        Ok(prepared)
    }

    pub fn validate(&self) -> Result<(), OutboxError> {
        self.outbox
            .verifier()
            .map_err(|_| OutboxError::Invalid("outbox domaine invalide"))?;
        if !matches!(
            self.objective.etat,
            EtatObjectif::Ouvert | EtatObjectif::EnCoordination
        ) {
            return Err(OutboxError::Invalid(
                "délégation autorisée seulement pour un objectif ouvert ou en coordination",
            ));
        }
        if self.delegation.etat != EtatDelegation::Creee
            || self.outbox.etat != EtatOutboxDelegation::Prepared
        {
            return Err(OutboxError::Invalid(
                "delegation et outbox doivent etre preparees",
            ));
        }
        if self.delegation.objectif_id != self.objective.id
            || self.outbox.delegation_id != self.delegation.id
        {
            return Err(OutboxError::Invalid("correlation locale incoherente"));
        }
        if self.outbox.target != self.delegation.participant {
            return Err(OutboxError::Invalid("cible et participant divergent"));
        }
        if self.issuer_scope.trim().is_empty() || self.issuer_scope.len() > 256 {
            return Err(OutboxError::Invalid("issuer_scope invalide"));
        }
        if self.issued_at <= 0
            || self.outbox.timeout_secs == 0
            || self.outbox.timeout_secs > crate::config::MAX_TIMEOUT_SECS
            || self.outbox.deadline_contractuelle <= self.issued_at
            || self.outbox.retry_until < self.issued_at
        {
            return Err(OutboxError::Invalid("echeances de l'enveloppe invalides"));
        }
        if self.outbox.body_bytes.len() > MAX_MESSAGE_BYTES
            || self.message_bytes.is_empty()
            || self.message_bytes.len() > MAX_MESSAGE_BYTES
        {
            return Err(OutboxError::Invalid("enveloppe hors borne"));
        }
        if self.outbox.body_hash != stable_body_hash(&self.outbox.body_bytes) {
            return Err(OutboxError::Invalid("body_hash divergent"));
        }

        let message: PublicMessage =
            serde_json::from_slice(&self.message_bytes).map_err(OutboxError::Decode)?;
        let body = std::str::from_utf8(&self.outbox.body_bytes)
            .map_err(|_| OutboxError::Invalid("body_bytes doit etre UTF-8"))?;
        if message.id != self.outbox.message_id.to_string()
            || message.from != crate::MAICIE_IDENTITY
            || message.to != self.outbox.target
            || message.body != body
            || message.reply != self.outbox.reply
            || message.hops != 4
            || message.reply_timeout != self.outbox.reply.then_some(self.outbox.timeout_secs)
            || message.deadline_at != u64::try_from(self.outbox.deadline_contractuelle).ok()
            || message.in_reply_to.is_some()
        {
            return Err(OutboxError::Invalid("enveloppe filaire divergente"));
        }
        Ok(())
    }
}

/// Snapshot complet donne a T008 pour lookup puis replay exact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingDelegationOutbox {
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    pub message_id: Uuid,
    pub issuer_scope: String,
    pub issued_at: i64,
    pub target: String,
    pub body_bytes: Vec<u8>,
    pub reply: bool,
    pub timeout_secs: u64,
    pub deadline_contractuelle: i64,
    pub body_hash: Vec<u8>,
    pub message_bytes: Vec<u8>,
    pub state: EtatOutboxDelegation,
    pub attempted_at: Option<i64>,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
}

impl PendingDelegationOutbox {
    pub fn public_message(&self) -> Result<PublicMessage, OutboxError> {
        serde_json::from_slice(&self.message_bytes).map_err(OutboxError::Decode)
    }

    /// Refuse toute reprise dont les octets durables ne correspondent plus à
    /// l'enveloppe indexée. Une corruption ne devient jamais un nouvel envoi.
    pub fn validate(&self) -> Result<(), OutboxError> {
        if self.issuer_scope.trim().is_empty()
            || self.issuer_scope.len() > 256
            || self.issued_at <= 0
            || self.timeout_secs == 0
            || self.timeout_secs > crate::config::MAX_TIMEOUT_SECS
            || self.deadline_contractuelle <= self.issued_at
            || self.retry_until < self.issued_at
            || self.retry_until > self.dedup_retained_until
            || self.body_bytes.is_empty()
            || self.body_bytes.len() > MAX_MESSAGE_BYTES
            || self.message_bytes.is_empty()
            || self.message_bytes.len() > MAX_MESSAGE_BYTES
        {
            return Err(OutboxError::Invalid("snapshot de reprise invalide"));
        }
        if self.body_hash != stable_body_hash(&self.body_bytes) {
            return Err(OutboxError::Invalid("body_hash durable divergent"));
        }
        let body = std::str::from_utf8(&self.body_bytes)
            .map_err(|_| OutboxError::Invalid("body_bytes doit etre UTF-8"))?;
        let message = self.public_message()?;
        if message.id != self.message_id.to_string()
            || message.from != crate::MAICIE_IDENTITY
            || message.to != self.target
            || message.body != body
            || message.reply != self.reply
            || message.hops != 4
            || message.reply_timeout != self.reply.then_some(self.timeout_secs)
            || message.deadline_at != u64::try_from(self.deadline_contractuelle).ok()
            || message.in_reply_to.is_some()
        {
            return Err(OutboxError::Invalid("enveloppe durable et index divergent"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecoverySnapshot {
    pub outbox: PendingDelegationOutbox,
    pub objective_state: EtatObjectif,
    pub delegation_state: EtatDelegation,
    pub last_issue: Option<Value>,
    pub issue_observed_at: Option<i64>,
}

/// Frontieres observables de la transaction locale, utilisees par les tests
/// de crash qui tuent un vrai processus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreCommitPhase {
    BeforeCommit,
    AfterCommit,
}

/// Empreinte stable non cryptographique : detection de corruption accidentelle,
/// jamais preuve d'integrite hostile.
pub fn stable_body_hash(bytes: &[u8]) -> Vec<u8> {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash.to_be_bytes().to_vec()
}

#[derive(Debug)]
pub enum OutboxError {
    Invalid(&'static str),
    Encode(serde_json::Error),
    Decode(serde_json::Error),
    Frame(BridgetClientError),
}

impl fmt::Display for OutboxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "outbox invalide : {reason}"),
            Self::Encode(source) => write!(formatter, "encodage outbox impossible : {source}"),
            Self::Decode(source) => write!(formatter, "decodage outbox impossible : {source}"),
            Self::Frame(source) => write!(formatter, "trame outbox impossible : {source}"),
        }
    }
}

impl std::error::Error for OutboxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode(source) | Self::Decode(source) => Some(source),
            Self::Frame(source) => Some(source),
            Self::Invalid(_) => None,
        }
    }
}
