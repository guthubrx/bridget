//! Cas d'usage d'orchestration explicites.
//!
//! Cette couche ne consulte ni socket ni processus : le CLI fournit un
//! annuaire factuel, puis l'application choisit de façon déterministe avant
//! d'écrire l'agrégat objectif/délégation/outbox dans le store privé.

use crate::config::DurationClasses;
use crate::domain::{
    ClasseDuree, Delegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne,
    OutboxDelegation,
};
use crate::outbox::{stable_body_hash, PreparedDelegation};
use crate::store::{DelegateReservation, MaicieStore, StoreError, StoredDelegateResult};
use serde::Serialize;
use std::fmt;
use uuid::Uuid;

/// Fait d'annuaire minimal consommé par la sélection déterministe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationCandidate {
    pub name: String,
    pub tags: Vec<String>,
    pub available: bool,
    pub dnd: bool,
}

/// Entrée complète du cas d'usage : aucune valeur de durée n'est implicite.
pub struct DelegateRequest<'a> {
    pub goal: &'a str,
    pub explicit_target: Option<&'a str>,
    pub required_tags: &'a [String],
    pub duration: ClasseDuree,
    pub reply: bool,
    /// Clé opaque fournie par le client. Elle est l'identité durable de la
    /// commande : un rejeu avec les mêmes octets canoniques rend les mêmes IDs.
    pub idempotency_key: &'a str,
    pub now: i64,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
    pub max_frame_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationCreated {
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    pub message_id: Uuid,
    pub participant: String,
    pub timeout_secs: u64,
    pub replayed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegateResult {
    Created(DelegationCreated),
    Candidates(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegateError {
    Invalid(&'static str),
    TargetUnavailable(String),
    EnvelopeMismatch,
    Store(String),
}

impl fmt::Display for DelegateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "délégation invalide : {reason}"),
            Self::TargetUnavailable(target) => write!(formatter, "cible indisponible : {target}"),
            Self::EnvelopeMismatch => write!(formatter, "commande idempotente divergente"),
            Self::Store(reason) => write!(formatter, "stockage impossible : {reason}"),
        }
    }
}

impl std::error::Error for DelegateError {}

/// Sélectionne puis persiste une délégation, sans effectuer d'I/O Bridget.
///
/// Une cible explicite ne passe pas par l'inférence de tags. Sans cible, les
/// seuls candidats sont disponibles, hors DND, distincts du pilote Maicie et
/// dont l'ensemble de tags est strictement égal à la demande.
pub fn delegate(
    store: &mut MaicieStore,
    durations: DurationClasses,
    pilot_name: &str,
    candidates: &[DelegationCandidate],
    request: &DelegateRequest<'_>,
) -> Result<DelegateResult, DelegateError> {
    if request.goal.trim().is_empty() || request.now <= 0 {
        return Err(DelegateError::Invalid("objectif ou horodatage absent"));
    }
    let canonical_request_bytes = canonical_request_bytes(request)?;
    if let Some(stored) = store
        .lookup_delegate_replay(request.idempotency_key, &canonical_request_bytes)
        .map_err(store_error)?
    {
        return Ok(DelegateResult::Created(created_from_stored(stored, true)));
    }
    let selected = match request.explicit_target {
        Some(target) => candidates
            .iter()
            .find(|candidate| candidate.name == target)
            .filter(|candidate| {
                candidate.name != pilot_name && candidate.available && !candidate.dnd
            })
            .map(|candidate| candidate.name.clone())
            .ok_or_else(|| DelegateError::TargetUnavailable(target.to_string()))?,
        None => {
            let mut matching = candidates
                .iter()
                .filter(|candidate| {
                    candidate.name != pilot_name
                        && candidate.available
                        && !candidate.dnd
                        && tags_equal(&candidate.tags, request.required_tags)
                })
                .map(|candidate| candidate.name.clone())
                .collect::<Vec<_>>();
            matching.sort();
            matching.dedup();
            if matching.len() != 1 {
                return Ok(DelegateResult::Candidates(matching));
            }
            matching.remove(0)
        }
    };

    let timeout_secs = timeout_for(request.duration, durations);
    let deadline = request
        .now
        .checked_add(
            i64::try_from(timeout_secs)
                .map_err(|_| DelegateError::Invalid("timeout hors borne"))?,
        )
        .ok_or(DelegateError::Invalid("échéance hors borne"))?;
    if request.retry_until < request.now || request.retry_until > request.dedup_retained_until {
        return Err(DelegateError::Invalid("horizon de reprise invalide"));
    }
    let mut objective =
        ObjectifCoordonne::nouveau(request.goal, ModeObjectif::Delegue, request.now)
            .map_err(|_| DelegateError::Invalid("objectif invalide"))?;
    objective
        .transition(EtatObjectif::EnCoordination, request.now)
        .map_err(|_| DelegateError::Invalid("transition objectif invalide"))?;
    let reason = if request.explicit_target.is_some() {
        "cible explicite"
    } else {
        "égalité stricte des tags"
    };
    let delegation = Delegation::nouvelle(
        objective.id,
        &selected,
        request.goal,
        request.duration,
        reason,
    )
    .map_err(|_| DelegateError::Invalid("délégation invalide"))?;
    let body_bytes = request.goal.as_bytes().to_vec();
    let outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: delegation.id,
        target: selected.clone(),
        body_hash: stable_body_hash(&body_bytes),
        body_bytes,
        reply: request.reply,
        timeout_secs,
        deadline_contractuelle: deadline,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: request.retry_until,
        dedup_retained_until: request.dedup_retained_until,
    };
    let message_id = outbox.message_id;
    let prepared = PreparedDelegation::new(
        objective.clone(),
        delegation.clone(),
        outbox,
        store.issuer_scope(),
        request.now,
        request.max_frame_bytes,
    )
    .map_err(|_| DelegateError::Invalid("enveloppe de délégation invalide"))?;
    match store
        .lookup_or_reserve_delegate(request.idempotency_key, &canonical_request_bytes, &prepared)
        .map_err(store_error)?
    {
        DelegateReservation::Created => Ok(DelegateResult::Created(DelegationCreated {
            objective_id: objective.id,
            delegation_id: delegation.id,
            message_id,
            participant: selected,
            timeout_secs,
            replayed: false,
        })),
        DelegateReservation::Replay(stored) => {
            Ok(DelegateResult::Created(created_from_stored(stored, true)))
        }
    }
}

#[derive(Serialize)]
struct CanonicalDelegateRequest<'a> {
    v: u8,
    goal: &'a str,
    explicit_target: Option<&'a str>,
    required_tags: Vec<&'a str>,
    duration: &'static str,
    reply: bool,
}

fn canonical_request_bytes(request: &DelegateRequest<'_>) -> Result<Vec<u8>, DelegateError> {
    if request.idempotency_key.is_empty()
        || request.idempotency_key.len() > 128
        || request.idempotency_key.chars().any(char::is_control)
    {
        return Err(DelegateError::Invalid("clé d'idempotence invalide"));
    }
    let mut required_tags = request
        .required_tags
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    required_tags.sort_unstable();
    if required_tags.iter().any(|tag| tag.is_empty())
        || required_tags.windows(2).any(|tags| tags[0] == tags[1])
    {
        return Err(DelegateError::Invalid("tags requis invalides"));
    }
    serde_json::to_vec(&CanonicalDelegateRequest {
        v: 1,
        goal: request.goal,
        explicit_target: request.explicit_target,
        required_tags,
        duration: duration_name(request.duration),
        reply: request.reply,
    })
    .map_err(|_| DelegateError::Invalid("commande delegate non sérialisable"))
}

fn duration_name(duration: ClasseDuree) -> &'static str {
    match duration {
        ClasseDuree::Courte => "courte",
        ClasseDuree::Normale => "normale",
        ClasseDuree::Longue => "longue",
    }
}

fn created_from_stored(stored: StoredDelegateResult, replayed: bool) -> DelegationCreated {
    DelegationCreated {
        objective_id: stored.objective_id,
        delegation_id: stored.delegation_id,
        message_id: stored.message_id,
        participant: stored.participant,
        timeout_secs: stored.timeout_secs,
        replayed,
    }
}

fn tags_equal(left: &[String], right: &[String]) -> bool {
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    left.sort();
    right.sort();
    left == right
}

fn timeout_for(duration: ClasseDuree, durations: DurationClasses) -> u64 {
    match duration {
        ClasseDuree::Courte => durations.short_secs,
        ClasseDuree::Normale => durations.normal_secs,
        ClasseDuree::Longue => durations.long_secs,
    }
}

fn store_error(error: StoreError) -> DelegateError {
    match error {
        StoreError::EnvelopeMismatch => DelegateError::EnvelopeMismatch,
        other => DelegateError::Store(other.to_string()),
    }
}
