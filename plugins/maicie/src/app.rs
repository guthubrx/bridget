//! Cas d'usage d'orchestration explicites.
//!
//! Cette couche ne consulte ni socket ni processus : le CLI fournit un
//! annuaire factuel, puis l'application choisit de façon déterministe avant
//! d'écrire l'agrégat objectif/délégation/outbox dans le store privé.

use crate::config::DurationClasses;
use crate::domain::{
    ActivationOutbox, ApprobationActivation, ClasseDuree, DecisionCoordination, Delegation,
    EtatDecision, EtatObjectif, EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne,
    OutboxDelegation, TypeDecision,
};
use crate::outbox::{PreparedDelegation, stable_body_hash};
use crate::store::{
    ActivationApprovalRequest, DelegateReservation, MaicieStore, ObjectiveSnapshot, StoreError,
    StoredDelegateResult,
};
use serde::Serialize;
use std::collections::BTreeSet;
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

/// Enregistrement immuable d'un message libre réellement destiné à Maicie.
/// Il conserve le corps tel quel, sans en déduire une intention d'objectif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationRecord {
    pub sender: String,
    pub body: String,
}

/// Aide locale structurée : elle décrit la seule commande mutante disponible,
/// mais ne construit ni n'émet jamais un message Bridget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationHelp {
    pub command: &'static str,
    pub usage: &'static str,
}

/// Résultat fermé du traitement d'un message direct. Ce type ne contient
/// aucun ordre de transport : une conversation ne peut pas devenir une
/// délégation ou une réponse réseau implicite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectMessageHandling {
    IgnoredOutsideMaicie,
    Conversation {
        record: ConversationRecord,
        help: ConversationHelp,
    },
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

/// Traite un message libre destiné à Maicie comme une conversation locale
/// immuable. La création d'objectif reste exclusivement derrière la commande
/// explicite [`delegate`].
pub fn handle_direct_message(
    store: &mut MaicieStore,
    message: &DirectBridgetMessage<'_>,
    maicie_identity: &str,
) -> Result<DirectMessageHandling, ObjectiveError> {
    match route_direct_message(message, maicie_identity) {
        DirectMessageRoute::OutsideMaicie => Ok(DirectMessageHandling::IgnoredOutsideMaicie),
        DirectMessageRoute::AddressedToMaicie => {
            let record = ConversationRecord {
                sender: message.from.to_string(),
                body: message.body.to_string(),
            };
            store
                .record_conversation(&record)
                .map_err(objective_store_error)?;
            Ok(DirectMessageHandling::Conversation {
                record,
                help: ConversationHelp {
                    command: "maicie delegate",
                    usage: "maicie delegate --goal <texte> [--to <agent>] [--duration courte|normale|longue]",
                },
            })
        }
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
    /// Classe explicitement choisie, affichée sans l'interpréter comme un
    /// état de retard local.
    pub duration: ClasseDuree,
    pub timeout_secs: u64,
    /// Échéance contractuelle déjà envoyée à Bridget ; Maicie l'affiche mais
    /// ne déclenche aucune action quand elle est atteinte.
    pub deadline_contractuelle: i64,
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

/// Proposition locale d'activation d'un profil absent. Le digest résolu est
/// fourni par la surface publique Bridget : aucun registre ni fichier Bridget
/// n'est relu par Maicie pour compléter cette approbation.
pub struct ProfileActivationProposalRequest<'a> {
    pub objective_id: Uuid,
    pub profile_id: &'a str,
    /// Type d'agent Bridget déclaré par le profil. Il est distinct du slug de
    /// profil et devient une partie des octets exacts du SpawnOrder approuvé.
    pub agent_type: &'a str,
    /// SHA-256 du profil déclaré et validé par la couche profils.
    pub profile_hash: &'a [u8],
    /// SHA-256 hexadécimal de la définition Bridget résolue et figée.
    pub resolved_definition_digest: &'a str,
    pub context_scope: &'a str,
    pub cwd: &'a str,
    pub persistent: bool,
    pub now: i64,
    pub spawn_deadline_at: i64,
    pub approval_expires_at: i64,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
    pub reason: &'a str,
}

/// Résultat durable de la proposition. Les octets du SpawnOrder sont produits
/// une seule fois avec le `command_id` créé avant toute I/O, puis deviennent
/// les paramètres approuvés et la future charge immuable de l'outbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileActivationProposal {
    pub decision: DecisionCoordination,
    pub approval: ApprobationActivation,
    pub spawn_order_bytes: Vec<u8>,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
}

/// Approbation exclusivement locale. L'acteur n'est pas paramétrable : le
/// domaine persiste toujours `local_human`, sans voie Bridget ou MCP pour le
/// fabriquer ou le remplacer.
pub struct LocalProfileApproval<'a> {
    pub approval_id: Uuid,
    pub now: i64,
    pub profile_hash: &'a [u8],
    pub resolved_definition_digest: &'a str,
}

/// Erreurs fermées du cycle de proposition et d'approbation de profil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileActivationError {
    Invalid(&'static str),
    Store(String),
}

impl fmt::Display for ProfileActivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "activation de profil invalide : {reason}"),
            Self::Store(reason) => write!(formatter, "stockage d'activation impossible : {reason}"),
        }
    }
}

impl std::error::Error for ProfileActivationError {}

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

/// Persiste une décision de réveil et son approbation mono-usage. Cette étape
/// ne contacte pas Bridget et ne crée donc encore aucune `ActivationOutbox`.
pub fn propose_profile_activation(
    store: &mut MaicieStore,
    request: &ProfileActivationProposalRequest<'_>,
) -> Result<ProfileActivationProposal, ProfileActivationError> {
    validate_profile_activation_proposal(request)?;
    let context_hash = definition_digest_bytes(request.resolved_definition_digest)?;
    let command_id = Uuid::new_v4();
    let spawn_order_bytes = approved_spawn_order_bytes(request, command_id)?;
    let parameters = String::from_utf8(spawn_order_bytes.clone())
        .map_err(|_| ProfileActivationError::Invalid("SpawnOrder non UTF-8"))?;
    let decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: request.objective_id,
        kind: TypeDecision::ReveillerProfil,
        proposee_par: MAICIE_PILOT.to_string(),
        etat: EtatDecision::Proposee,
        motif: request.reason.to_string(),
    };
    let approval = ApprobationActivation {
        id: Uuid::new_v4(),
        command_id,
        objective_id: request.objective_id,
        profile_id: request.profile_id.to_string(),
        profile_hash: request.profile_hash.to_vec(),
        // Le hash de contexte est exactement le digest de définition résolue
        // fourni par Bridget : T023 le revalidera avant toute émission.
        context_hash,
        context_scope: request.context_scope.to_string(),
        parameters,
        actor: "local_human".to_string(),
        expires_at: request.approval_expires_at,
        consumed_at: None,
    };
    store
        .create_activation_proposal(&decision, &approval)
        .map_err(profile_activation_store_error)?;
    Ok(ProfileActivationProposal {
        decision,
        approval,
        spawn_order_bytes,
        retry_until: request.retry_until,
        dedup_retained_until: request.dedup_retained_until,
    })
}

/// Approuve localement une proposition déjà persistée. L'acteur est scellé
/// dans [`propose_profile_activation`], et les hashes courants sont comparés
/// dans la transaction qui écrit l'`ActivationOutbox`.
pub fn approve_profile_activation(
    store: &mut MaicieStore,
    proposal: &ProfileActivationProposal,
    approval: &LocalProfileApproval<'_>,
) -> Result<ActivationOutbox, ProfileActivationError> {
    if approval.approval_id != proposal.approval.id {
        return Err(ProfileActivationError::Invalid("approbation divergente"));
    }
    let context_hash = definition_digest_bytes(approval.resolved_definition_digest)?;
    store
        .approve_activation(
            approval.approval_id,
            &ActivationApprovalRequest {
                now: approval.now,
                profile_hash: approval.profile_hash,
                context_hash: &context_hash,
                spawn_order_bytes: &proposal.spawn_order_bytes,
                retry_until: proposal.retry_until,
                dedup_retained_until: proposal.dedup_retained_until,
            },
        )
        .map_err(profile_activation_store_error)
}

/// Recharge une proposition durable pour la commande locale `profile approve`.
/// Les bytes du SpawnOrder viennent exclusivement de l'approbation persistée ;
/// le CLI ne peut ni les éditer ni en produire une seconde version.
pub fn stored_profile_activation_proposal(
    store: &MaicieStore,
    approval_id: Uuid,
    retry_until: i64,
    dedup_retained_until: i64,
) -> Result<ProfileActivationProposal, ProfileActivationError> {
    if retry_until <= 0 || retry_until > dedup_retained_until {
        return Err(ProfileActivationError::Invalid(
            "horizon de reprise invalide",
        ));
    }
    let (decision, approval) = store
        .activation_proposal(approval_id)
        .map_err(profile_activation_store_error)?;
    Ok(ProfileActivationProposal {
        spawn_order_bytes: approval.parameters.as_bytes().to_vec(),
        decision,
        approval,
        retry_until,
        dedup_retained_until,
    })
}

/// Lit les données de coordination disponibles localement. Les sources
/// Bridget et ACP restent explicitement inconnues jusqu'à T018.
pub fn status(
    store: &MaicieStore,
    objective_id: Option<Uuid>,
) -> Result<Vec<ObjectiveSnapshot>, ObjectiveError> {
    let snapshots = store
        .objective_snapshots(objective_id)
        .map_err(objective_store_error)?;
    match objective_id {
        Some(id) if snapshots.is_empty() => return Err(ObjectiveError::NotFound(id)),
        _ => {}
    }
    Ok(snapshots)
}

/// Extrait les seuls agents auxquels la coordination locale a effectivement
/// délégué. Cette projection pure borne la capture Attach du CLI : un
/// `status` n'abonne jamais Maicie à l'activité d'agents hors de ses objectifs.
pub fn delegated_participants(snapshots: &[ObjectiveSnapshot]) -> Vec<String> {
    snapshots
        .iter()
        .flat_map(|snapshot| {
            snapshot
                .delegations
                .iter()
                .map(|delegation| &delegation.participant)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .cloned()
        .collect()
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
    store
        .close_objective(objective_id, reason, now)
        .map_err(objective_store_error)
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
    let objective = one_objective(store, objective_id)?.objective;
    if objective.etat == EtatObjectif::Clos {
        return Err(ObjectiveError::Invalid("objectif déjà clos"));
    }
    let decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind,
        proposee_par: MAICIE_PILOT.to_string(),
        etat: EtatDecision::Appliquee,
        motif: format!("{reason}: {participant}"),
    };
    store
        .apply_objective_decision(&decision, None, objective.etat)
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
        StoreError::Invalid("objectif déjà clos") => {
            ObjectiveError::Invalid("objectif déjà clos")
        }
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

    let timeout_secs = timeout_for_duration(request.duration, durations);
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
            duration: request.duration,
            timeout_secs,
            deadline_contractuelle: deadline,
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
        duration: stored.duration,
        timeout_secs: stored.timeout_secs,
        deadline_contractuelle: stored.deadline_contractuelle,
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

/// Projection pure de la classe configurée vers le timeout public Bridget.
/// Aucun timer, aucune relance et aucune transition de coordination n'en
/// découlent : l'échéance est un fait contractuel affichable seulement.
pub fn timeout_for_duration(duration: ClasseDuree, durations: DurationClasses) -> u64 {
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

#[derive(Serialize)]
struct ApprovedSpawnOrder<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    agent_type: &'a str,
    name: Option<&'a str>,
    cwd: &'a str,
    persistent: bool,
    command_id: String,
    issued_at: i64,
    deadline_at: i64,
}

fn validate_profile_activation_proposal(
    request: &ProfileActivationProposalRequest<'_>,
) -> Result<(), ProfileActivationError> {
    if request.objective_id.is_nil()
        || request.profile_id.trim().is_empty()
        || request.agent_type.trim().is_empty()
        || request.profile_hash.len() != 32
        || request.context_scope.trim().is_empty()
        || request.cwd.trim().is_empty()
        || request.reason.trim().is_empty()
        || request.now <= 0
        || request.spawn_deadline_at <= request.now
        || request.approval_expires_at <= request.now
        || request.retry_until < request.now
        || request.retry_until > request.dedup_retained_until
    {
        return Err(ProfileActivationError::Invalid("proposition incomplète"));
    }
    Ok(())
}

fn approved_spawn_order_bytes(
    request: &ProfileActivationProposalRequest<'_>,
    command_id: Uuid,
) -> Result<Vec<u8>, ProfileActivationError> {
    serde_json::to_vec(&ApprovedSpawnOrder {
        kind: "SpawnOrder",
        agent_type: request.agent_type,
        name: None,
        cwd: request.cwd,
        persistent: request.persistent,
        command_id: command_id.to_string(),
        issued_at: request.now,
        deadline_at: request.spawn_deadline_at,
    })
    .map_err(|_| ProfileActivationError::Invalid("SpawnOrder non sérialisable"))
}

fn definition_digest_bytes(digest: &str) -> Result<Vec<u8>, ProfileActivationError> {
    if digest.len() != 64 {
        return Err(ProfileActivationError::Invalid(
            "digest de définition invalide",
        ));
    }
    digest
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(value: u8) -> Result<u8, ProfileActivationError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(ProfileActivationError::Invalid(
            "digest de définition invalide",
        )),
    }
}

fn profile_activation_store_error(error: StoreError) -> ProfileActivationError {
    ProfileActivationError::Store(error.to_string())
}
