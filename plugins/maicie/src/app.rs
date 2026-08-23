//! Cas d'usage d'orchestration explicites.
//!
//! Cette couche ne consulte ni socket ni processus : le CLI fournit un
//! annuaire factuel, puis l'application choisit de façon déterministe avant
//! d'écrire l'agrégat objectif/délégation/outbox dans le store privé.

use crate::config::DurationClasses;
use crate::domain::{
    ClasseDuree, DecisionCoordination, Delegation, EtatDecision, EtatObjectif,
    EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne, OutboxDelegation, TypeDecision,
};
use crate::outbox::{PreparedDelegation, stable_body_hash};
use crate::store::{
    DelegateReservation, MaicieStore, ObjectiveSnapshot, StoreError, StoredDelegateResult,
};
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

/// Message Bridget entrant traité à la frontière de coordination. Il ne porte
/// volontairement aucun accès au store ni au client réseau : une conversation
/// directe ne peut donc pas devenir une mutation ou un envoi Maicie par effet
/// de bord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectBridgetMessage<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub body: &'a str,
}

/// Routage fermé des messages directs. Seuls les messages explicitement
/// adressés à Maicie pourront être affichés par le cas d'usage conversationnel
/// suivant ; tous les autres restent hors du domaine Maicie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectMessageRoute {
    OutsideMaicie,
    AddressedToMaicie,
}

/// Garde structurel de la frontière conversationnelle : cette fonction est
/// pure et ne peut ni créer objectif/délégation, ni construire un envoi
/// Bridget. Une délégation reste exclusivement créée par [`delegate`].
pub fn route_direct_message(
    message: &DirectBridgetMessage<'_>,
    maicie_identity: &str,
) -> DirectMessageRoute {
    if message.to == maicie_identity {
        DirectMessageRoute::AddressedToMaicie
    } else {
        DirectMessageRoute::OutsideMaicie
    }
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

/// Erreurs des commandes explicites sur les objectifs. Le CLI les rend sans
/// les réinterpréter, afin que la décision reste portée par le cas d'usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectiveError {
    Invalid(&'static str),
    NotFound(Uuid),
    Store(String),
}

impl fmt::Display for ObjectiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "objectif invalide : {reason}"),
            Self::NotFound(id) => write!(formatter, "objectif introuvable : {id}"),
            Self::Store(reason) => write!(formatter, "stockage impossible : {reason}"),
        }
    }
}

impl std::error::Error for ObjectiveError {}

/// Lit les données de coordination disponibles localement. Les sources
/// Bridget et ACP restent explicitement inconnues jusqu'à T018.
pub fn status(
    store: &MaicieStore,
    objective_id: Option<Uuid>,
) -> Result<Vec<ObjectiveSnapshot>, ObjectiveError> {
    let snapshots = store
        .objective_snapshots(objective_id)
        .map_err(objective_store_error)?;
    if let Some(id) = objective_id
        && snapshots.is_empty()
    {
        return Err(ObjectiveError::NotFound(id));
    }
    Ok(snapshots)
}

/// Enregistre l'ajout explicite d'un participant comme décision appliquée.
/// La création d'une nouvelle délégation et de son outbox reste exclusivement
/// le cas d'usage `delegate` : aucune I/O Bridget implicite n'est possible ici.
pub fn add_participant(
    store: &mut MaicieStore,
    objective_id: Uuid,
    participant: &str,
) -> Result<DecisionCoordination, ObjectiveError> {
    apply_participant_decision(
        store,
        objective_id,
        participant,
        TypeDecision::AjouterParticipant,
        "ajout explicite de participant",
    )
}

/// Enregistre le retrait explicite et motivé d'un participant. Cette décision
/// ne prétend pas annuler une remise Bridget : ce contrat reste explicite et
/// sera raccordé par le cas d'usage d'annulation dédié.
pub fn remove_participant(
    store: &mut MaicieStore,
    objective_id: Uuid,
    participant: &str,
    reason: &str,
) -> Result<DecisionCoordination, ObjectiveError> {
    if reason.trim().is_empty() {
        return Err(ObjectiveError::Invalid("motif de retrait obligatoire"));
    }
    apply_participant_decision(
        store,
        objective_id,
        participant,
        TypeDecision::RetirerParticipant,
        reason,
    )
}

/// Retourne une agrégation purement factuelle de l'état corrélé disponible.
/// Aucune sortie d'agent n'est interprétée et aucun état objectif ne change.
pub fn summarize(
    store: &MaicieStore,
    objective_id: Uuid,
) -> Result<ObjectiveSnapshot, ObjectiveError> {
    one_objective(store, objective_id)
}

/// Clôture explicitement l'objectif et écrit son audit dans la même
/// transaction SQLite ; aucune issue Bridget ne peut provoquer cette action.
pub fn close(
    store: &mut MaicieStore,
    objective_id: Uuid,
    reason: &str,
    now: i64,
) -> Result<DecisionCoordination, ObjectiveError> {
    if reason.trim().is_empty() || now <= 0 {
        return Err(ObjectiveError::Invalid("motif ou horodatage absent"));
    }
    let mut objective = one_objective(store, objective_id)?.objective;
    objective
        .clore(now)
        .map_err(|_| ObjectiveError::Invalid("objectif déjà clos"))?;
    let decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind: TypeDecision::Cloturer,
        proposee_par: MAICIE_PILOT.to_string(),
        etat: EtatDecision::Appliquee,
        motif: reason.to_string(),
    };
    store
        .apply_objective_decision(&decision, Some(&objective))
        .map_err(objective_store_error)?;
    Ok(decision)
}

const MAICIE_PILOT: &str = "maicie";

fn apply_participant_decision(
    store: &mut MaicieStore,
    objective_id: Uuid,
    participant: &str,
    kind: TypeDecision,
    reason: &str,
) -> Result<DecisionCoordination, ObjectiveError> {
    if participant.trim().is_empty() {
        return Err(ObjectiveError::Invalid("participant obligatoire"));
    }
    let _ = one_objective(store, objective_id)?;
    let decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind,
        proposee_par: MAICIE_PILOT.to_string(),
        etat: EtatDecision::Appliquee,
        motif: format!("{reason}: {participant}"),
    };
    store
        .apply_objective_decision(&decision, None)
        .map_err(objective_store_error)?;
    Ok(decision)
}

fn one_objective(
    store: &MaicieStore,
    objective_id: Uuid,
) -> Result<ObjectiveSnapshot, ObjectiveError> {
    status(store, Some(objective_id)).map(|mut snapshots| snapshots.remove(0))
}

fn objective_store_error(error: StoreError) -> ObjectiveError {
    match error {
        StoreError::NotFound(_) => ObjectiveError::NotFound(Uuid::nil()),
        other => ObjectiveError::Store(other.to_string()),
    }
}

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
    let eligible = candidates
        .iter()
        .filter(|candidate| candidate.name != pilot_name && candidate.available && !candidate.dnd)
        .collect::<Vec<_>>();
    let selected = match request.explicit_target {
        Some(target) => eligible
            .iter()
            .find(|candidate| candidate.name == target)
            .map(|candidate| candidate.name.clone())
            .ok_or_else(|| DelegateError::TargetUnavailable(target.to_string()))?,
        None => {
            let mut matching = eligible
                .iter()
                .filter(|candidate| tags_equal(&candidate.tags, request.required_tags))
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
