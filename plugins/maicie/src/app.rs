//! Cas d'usage d'orchestration explicites.
//!
//! Cette couche ne consulte ni socket ni processus : le CLI fournit un
//! annuaire factuel, puis l'application choisit de façon déterministe avant
//! d'écrire l'agrégat objectif/délégation/outbox dans le store privé.

use crate::bridget_client::{GuichetClaim, GuichetLifecycleEvent};
use crate::catalogue::{
    ArbitrationLink, AttestedClosure, CatalogueError, CatalogueJournal, ReconcileReport,
};
use crate::citation::unclassified_known_citations;
use crate::config::{CoordinationPoliciesConfig, DurationClasses};
use crate::domain::guichet::{
    GuichetDomainError, ProjectionCoordinationState, ProjectionDurationClass, ProjectionFreshness,
    ProjectionLocalDelivery, ProjectionLocalDeliveryState, ProjectionReply,
    ProjectionTransportObservation, ProjectionTransportState, RequeteCanonique, RequeteGuichet,
    parse_claim, parse_lifecycle_event,
};
use crate::domain::{
    ActivationOutbox, ApprobationActivation, ClasseDuree, CoutMissionAgent, DecisionCoordination,
    DefinitionCoordination, Delegation, EntreeReductionCoordination, EtatDecision, EtatFlux,
    EtatObjectif, EtatOutboxDelegation, EtatRequeteGuichet, EvenementCoordination,
    FaitAppartenanceRepli, FaitReassignation, FraicheurCoordination, ModeObjectif,
    MotifRefusGreffe, ObjectifCoordonne, OutboxDelegation, PolitiqueReassignation,
    SnapshotTransport, SourceSnapshot, SuiteObjective, TypeDecision, TypeFaitReassignation,
};
use crate::outbox::{PreparedDelegation, stable_body_hash};
pub use crate::store::GuichetLifecycleResult;
use crate::store::{
    ActivationApprovalRequest, CoordinationCommitPhase, DeferredDispatchParams,
    DelegateReservation, GuichetProjectionFacts, MaicieStore, ObjectiveSnapshot, StoreError,
    StoredDelegateResult, StoredGuichetReply,
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

/// Résultat applicatif d'une relève. Les octets de réponse sont exactement
/// ceux du reçu durable ; ils ne doivent jamais être reconstruits par le CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetProcessResult {
    pub request_id: String,
    pub objective_id: Option<Uuid>,
    pub delegation_id: Option<Uuid>,
    pub response_message_id: String,
    pub reply_bytes: Vec<u8>,
    pub replayed: bool,
    pub refusal_reason: Option<MotifRefusGreffe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuichetError {
    InvalidEnvelope(String),
    UnsupportedOperation,
    EnvelopeMismatch,
    Store(String),
}

impl fmt::Display for GuichetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEnvelope(reason) => {
                write!(formatter, "requête guichet invalide : {reason}")
            }
            Self::UnsupportedOperation => {
                formatter.write_str("opération guichet non prise en charge")
            }
            Self::EnvelopeMismatch => formatter.write_str("enveloppe guichet divergente"),
            Self::Store(reason) => write!(formatter, "greffe guichet impossible : {reason}"),
        }
    }
}

/// Traite une relève sans I/O réseau. En l'absence d'observation transport,
/// `mission_status` rend explicitement une fraîcheur `unavailable`.
pub fn process_guichet_claim(
    store: &mut MaicieStore,
    claim: &GuichetClaim,
    response_message_id: &str,
    now: i64,
) -> Result<GuichetProcessResult, GuichetError> {
    let canonical = parse_claim(claim).map_err(guichet_domain_error)?;
    let stored = match &canonical.request {
        RequeteGuichet::DeliveryReport(report) => {
            store.graft_delivery_report(claim, &canonical, report, response_message_id, now)
        }
        RequeteGuichet::MissionStatus { .. } => process_mission_status_canonical(
            store,
            claim,
            &canonical,
            response_message_id,
            None,
            now,
        ),
        RequeteGuichet::DeadlineQuestion { .. } => {
            process_deadline_question_canonical(store, claim, &canonical, response_message_id, now)
        }
    };
    match stored {
        Ok(stored) => Ok(guichet_process_result(stored, None)),
        Err(error) => {
            let Some(reason) = deterministic_refusal_reason(&error) else {
                return Err(guichet_store_error(error));
            };
            let stored = store
                .persist_guichet_refusal(claim, &canonical, response_message_id, now, reason)
                .map_err(guichet_store_error)?;
            Ok(guichet_process_result(stored, Some(reason)))
        }
    }
}

/// Produit et persiste un statut de mission en gardant les deux vérités
/// séparées : le registre local d'un côté, l'observation Bridget attestée de
/// l'autre. `None` ne signifie jamais absence d'activité, mais indisponibilité.
pub fn process_mission_status_claim(
    store: &mut MaicieStore,
    claim: &GuichetClaim,
    response_message_id: &str,
    transport: Option<&SnapshotTransport>,
    now: i64,
) -> Result<GuichetProcessResult, GuichetError> {
    let canonical = parse_claim(claim).map_err(guichet_domain_error)?;
    process_mission_status_canonical(
        store,
        claim,
        &canonical,
        response_message_id,
        transport,
        now,
    )
    .map(|stored| guichet_process_result(stored, None))
    .map_err(guichet_store_error)
}

/// Produit et persiste l'échéance contractuelle sans créer de timer, de
/// relance ou de qualification implicite du retard.
pub fn process_deadline_question_claim(
    store: &mut MaicieStore,
    claim: &GuichetClaim,
    response_message_id: &str,
    now: i64,
) -> Result<GuichetProcessResult, GuichetError> {
    let canonical = parse_claim(claim).map_err(guichet_domain_error)?;
    process_deadline_question_canonical(store, claim, &canonical, response_message_id, now)
        .map(|stored| guichet_process_result(stored, None))
        .map_err(guichet_store_error)
}

fn process_mission_status_canonical(
    store: &mut MaicieStore,
    claim: &GuichetClaim,
    canonical: &RequeteCanonique,
    response_message_id: &str,
    transport: Option<&SnapshotTransport>,
    now: i64,
) -> Result<StoredGuichetReply, StoreError> {
    let RequeteGuichet::MissionStatus { delegation_id } = canonical.request else {
        return Err(StoreError::Invalid("projection mission_status incohérente"));
    };
    let stored =
        store.persist_guichet_projection(claim, canonical, response_message_id, now, |facts| {
            let (transport_observation, freshness) = transport_projection(facts, transport)?;
            Ok(ProjectionReply::MissionStatus {
                delegation_id: delegation_id.to_string(),
                objective_id: facts.objective.id.to_string(),
                coordination_state: coordination_projection(facts.objective.etat),
                local_delivery: local_delivery_projection(facts)?,
                transport_observation,
                freshness,
            })
        })?;
    Ok(stored)
}

fn process_deadline_question_canonical(
    store: &mut MaicieStore,
    claim: &GuichetClaim,
    canonical: &RequeteCanonique,
    response_message_id: &str,
    now: i64,
) -> Result<StoredGuichetReply, StoreError> {
    let RequeteGuichet::DeadlineQuestion { delegation_id } = canonical.request else {
        return Err(StoreError::Invalid(
            "projection deadline_question incohérente",
        ));
    };
    let stored =
        store.persist_guichet_projection(claim, canonical, response_message_id, now, |facts| {
            Ok(ProjectionReply::DeadlineQuestion {
                delegation_id: delegation_id.to_string(),
                duration_class: duration_projection(facts.delegation.duree),
                deadline_at: facts.deadline_at,
            })
        })?;
    Ok(stored)
}

fn local_delivery_projection(
    facts: &GuichetProjectionFacts,
) -> Result<ProjectionLocalDelivery, StoreError> {
    let issue = facts
        .local_delivery
        .issue
        .as_ref()
        .map(|value| {
            value
                .get("kind")
                .and_then(|kind| kind.as_str())
                .filter(|kind| !kind.is_empty())
                .map(str::to_string)
                .ok_or(StoreError::Corrupt("issue locale sans kind attesté"))
        })
        .transpose()?;
    let state = match facts.local_delivery.state {
        EtatOutboxDelegation::Prepared => ProjectionLocalDeliveryState::Pending,
        EtatOutboxDelegation::Accepted => ProjectionLocalDeliveryState::Accepted,
        EtatOutboxDelegation::OutcomeUnknown => ProjectionLocalDeliveryState::OutcomeUnknown,
        EtatOutboxDelegation::Rejected => ProjectionLocalDeliveryState::Rejected,
    };
    Ok(ProjectionLocalDelivery {
        state,
        issue,
        observed_at: facts.local_delivery.observed_at,
    })
}

fn transport_projection(
    facts: &GuichetProjectionFacts,
    transport: Option<&SnapshotTransport>,
) -> Result<(Option<ProjectionTransportObservation>, ProjectionFreshness), StoreError> {
    let Some(transport) = transport else {
        return Ok((None, ProjectionFreshness::Unavailable));
    };
    transport
        .verifier()
        .map_err(|_| StoreError::Invalid("snapshot transport invalide"))?;
    if transport.message_id != facts.local_delivery.message_id {
        return Err(StoreError::Invalid("snapshot transport non corrélé"));
    }
    let (state, freshness) = match transport.stream_state {
        EtatFlux::Fresh => (
            ProjectionTransportState::Connected,
            ProjectionFreshness::Fresh,
        ),
        EtatFlux::Gap => (ProjectionTransportState::Gap, ProjectionFreshness::Gap),
        EtatFlux::Ended => (ProjectionTransportState::Ended, ProjectionFreshness::Ended),
        EtatFlux::Unavailable => (
            ProjectionTransportState::Unavailable,
            ProjectionFreshness::Unavailable,
        ),
    };
    let source = match transport.source {
        SourceSnapshot::Bridget => "bridget",
        SourceSnapshot::AcpSubscription => "acp_subscription",
    };
    Ok((
        Some(ProjectionTransportObservation {
            state,
            request_state: transport.request_state.clone(),
            observed_at: transport.observed_at,
            source: source.to_string(),
            subscription_id: transport.subscription_id.clone(),
            seq: transport.seq,
        }),
        freshness,
    ))
}

fn coordination_projection(state: EtatObjectif) -> ProjectionCoordinationState {
    match state {
        EtatObjectif::Ouvert => ProjectionCoordinationState::Open,
        EtatObjectif::EnCoordination => ProjectionCoordinationState::EnCoordination,
        EtatObjectif::AEvaluer => ProjectionCoordinationState::AEvaluer,
        EtatObjectif::Synthetise => ProjectionCoordinationState::Synthetise,
        EtatObjectif::Clos => ProjectionCoordinationState::Clos,
    }
}

fn duration_projection(duration: ClasseDuree) -> ProjectionDurationClass {
    match duration {
        ClasseDuree::Courte => ProjectionDurationClass::Courte,
        ClasseDuree::Normale => ProjectionDurationClass::Normale,
        ClasseDuree::Longue => ProjectionDurationClass::Longue,
    }
}

fn guichet_process_result(
    stored: StoredGuichetReply,
    refusal_reason: Option<MotifRefusGreffe>,
) -> GuichetProcessResult {
    GuichetProcessResult {
        request_id: stored.reception.request_id,
        objective_id: stored.reception.objective_id,
        delegation_id: stored.reception.delegation_id,
        response_message_id: stored.reception.response_message_id,
        reply_bytes: stored.reception.reply_bytes,
        replayed: stored.replayed,
        refusal_reason,
    }
}

fn deterministic_refusal_reason(error: &StoreError) -> Option<MotifRefusGreffe> {
    match error {
        StoreError::NotFound(_) => Some(MotifRefusGreffe::DelegationAbsente),
        StoreError::EnvelopeMismatch => Some(MotifRefusGreffe::EnveloppeDivergente),
        StoreError::Invalid("relations du rapport invalides")
        | StoreError::Invalid("relations de projection invalides") => {
            Some(MotifRefusGreffe::RelationsInvalides)
        }
        _ => None,
    }
}

/// Enregistre le fait terminal poussé par Bridget. Cette voie ne crée aucune
/// décision : le rapport structuré reste l'unique source de l'effet métier.
pub fn record_guichet_lifecycle_event(
    store: &mut MaicieStore,
    event: &GuichetLifecycleEvent,
) -> Result<GuichetLifecycleResult, GuichetError> {
    let event = parse_lifecycle_event(event).map_err(guichet_domain_error)?;
    let result = store
        .record_guichet_lifecycle_event(&event)
        .map_err(guichet_store_error)?;
    let kind = match event.state {
        EtatRequeteGuichet::Answered => TypeFaitReassignation::Answered,
        EtatRequeteGuichet::Cancelled => TypeFaitReassignation::AnnulationAdministrative,
        EtatRequeteGuichet::TimedOut => TypeFaitReassignation::TimedOut,
    };
    let fact = FaitReassignation {
        event_id: event.event_id.clone(),
        request_id: event.request_id.clone(),
        kind,
        observed_at: event.observed_at,
        freshness: FraicheurCoordination::Fresh,
        delivery_hash: None,
    };
    match store.apply_reassignment_fact(fact) {
        Ok(_) => {}
        // Tous les terminaux du guichet ne décrivent pas une demande suivie
        // F29. Leur fait reste durable sans qu'une délégation soit inventée.
        Err(StoreError::NotFound(_)) => {}
        Err(error) => return Err(guichet_store_error(error)),
    }
    Ok(result)
}

/// Applique un rappel cursé seulement après la frontière SnapshotCaughtUp.
/// Le curseur, la décision F29 et ses outboxes partagent le même commit.
///
/// Un `reminder_sent` pour une demande inconnue du greffe (hors F29) est
/// journalisé puis sauté : le curseur avance, aucune délégation n'est inventée.
/// Même doctrine que [`record_guichet_lifecycle_event`] face à `NotFound`.
pub fn apply_attested_coordination_event(
    store: &mut MaicieStore,
    canonical_bytes: &[u8],
    observer: impl FnMut(CoordinationCommitPhase) -> Result<(), StoreError>,
) -> Result<(), GuichetError> {
    let event =
        EvenementCoordination::depuis_trame_attestee(canonical_bytes, FraicheurCoordination::Fresh)
            .map_err(|_| {
                GuichetError::InvalidEnvelope("trame de coordination invalide".to_string())
            })?;
    let context = match store.reassignment_request_context(event.request_id()) {
        Ok(context) => context,
        // Bridget peut pousser des rappels pour des conversations hors Maicie
        // (ex. agent → référent). Les traiter comme F29 empoisonne toute commande.
        Err(StoreError::NotFound(reason)) => {
            eprintln!(
                "avertissement: reminder_sent ignoré — demande suivie F29 inconnue \
                 (request_id={}, event_id={}, cursor={}): {reason}",
                event.request_id(),
                event.event_id(),
                event.cursor(),
            );
            store
                .acknowledge_untracked_coordination_event(&event)
                .map_err(guichet_store_error)?;
            return Ok(());
        }
        Err(error) => return Err(guichet_store_error(error)),
    };
    if context.participant != event.recipient() {
        return Err(GuichetError::InvalidEnvelope(
            "destinataire du rappel et demande suivie divergents".to_string(),
        ));
    }
    let fact = FaitReassignation {
        event_id: event.event_id().to_string(),
        request_id: event.request_id().to_string(),
        kind: TypeFaitReassignation::ReminderSent,
        observed_at: event.observed_at(),
        freshness: FraicheurCoordination::Fresh,
        delivery_hash: None,
    };
    let input = EntreeReductionCoordination::EvenementAtteste {
        objectif_id: context.objectif_id,
        delegation_id: context.delegation_id,
        generation: context.generation,
        evenement: event,
    };
    store
        .apply_attested_coordination_and_reassignment(&input, fact, observer)
        .map_err(guichet_store_error)?;
    Ok(())
}

fn guichet_domain_error(error: GuichetDomainError) -> GuichetError {
    match error {
        GuichetDomainError::UnsupportedOperation => GuichetError::UnsupportedOperation,
        GuichetDomainError::CanonicalBytesMismatch => GuichetError::EnvelopeMismatch,
        GuichetDomainError::InvalidEnvelope(reason) => {
            GuichetError::InvalidEnvelope(reason.to_string())
        }
    }
}

fn guichet_store_error(error: StoreError) -> GuichetError {
    match error {
        StoreError::EnvelopeMismatch => GuichetError::EnvelopeMismatch,
        StoreError::Invalid(reason) | StoreError::NotFound(reason) => {
            GuichetError::InvalidEnvelope(reason.to_string())
        }
        other => GuichetError::Store(other.to_string()),
    }
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
                    usage: "maicie delegate --goal <texte> --suite aucune| <objectif> [--to <agent>] [--depends-on <id>] [--reference <id>] [--duration courte|normale|longue]",
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
    /// Identifiant exact du constat motivant cette délégation, s'il est déclaré
    /// à la construction. Absent : délégation ordinaire sans lien d'arbitrage.
    pub constat_id: Option<&'a str>,
    /// F36 — suite obligatoire (`Aucune` ou objectif nommé).
    pub suite: SuiteObjective,
    /// F37 — prérequis objectifs (arêtes OBJECTIF→OBJECTIF).
    pub depends_on: &'a [Uuid],
    /// F37 — citations de contexte sans couplage.
    pub references: &'a [Uuid],
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
    /// Absent tant que la délégation est `EnAttentePrerequis`.
    pub message_id: Option<Uuid>,
    pub participant: String,
    /// Classe explicitement choisie, affichée sans l'interpréter comme un
    /// état de retard local.
    pub duration: ClasseDuree,
    pub timeout_secs: u64,
    /// Échéance contractuelle déjà envoyée à Bridget ; Maicie l'affiche mais
    /// ne déclenche aucune action quand elle est atteinte.
    pub deadline_contractuelle: i64,
    pub replayed: bool,
    /// `true` si aucune outbox n'existe encore (attente de prérequis).
    pub waiting_on_prerequisites: bool,
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
/// Sans coûts fournis, chaque agent délégué porte « inconnu » (jamais zéro).
pub fn close(
    store: &mut MaicieStore,
    objective_id: Uuid,
    reason: &str,
    now: i64,
) -> Result<DecisionCoordination, ObjectiveError> {
    close_with_costs(store, objective_id, reason, now, Vec::new())
}

/// Clôture en portant les coûts attestés (ou inconnus) fournis par l'appelant.
/// Les agents absents des overrides restent « inconnu » — jamais un zéro inventé.
pub fn close_with_costs(
    store: &mut MaicieStore,
    objective_id: Uuid,
    reason: &str,
    now: i64,
    costs: Vec<CoutMissionAgent>,
) -> Result<DecisionCoordination, ObjectiveError> {
    if reason.trim().is_empty() || now <= 0 {
        return Err(ObjectiveError::Invalid("motif ou horodatage absent"));
    }
    if costs.is_empty() {
        store
            .close_objective(objective_id, reason, now)
            .map_err(objective_store_error)
    } else {
        store
            .close_objective_with_costs(objective_id, reason, now, costs)
            .map_err(objective_store_error)
    }
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
    validate_suite_and_citations(store, request)?;
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
    objective.suite = Some(request.suite.clone());
    objective.depends_on = request.depends_on.to_vec();
    objective.references = request.references.to_vec();

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
    .and_then(|delegation| match request.constat_id {
        Some(constat_id) => delegation.pour_constat(constat_id),
        None => Ok(delegation),
    })
    .map_err(|_| DelegateError::Invalid("délégation invalide"))?;

    let waiting = {
        let mut needs_wait = false;
        for id in request.depends_on {
            if !store.objective_is_closed(*id).map_err(store_error)? {
                needs_wait = true;
                break;
            }
        }
        needs_wait
    };

    if waiting {
        let waiting_delegation = delegation
            .en_attente_de_prerequis()
            .map_err(|_| DelegateError::Invalid("délégation en attente invalide"))?;
        let deferred = DeferredDispatchParams {
            reply: request.reply,
            timeout_secs,
            retry_until: request.retry_until,
            dedup_retained_until: request.dedup_retained_until,
            max_frame_bytes: request.max_frame_bytes,
            deadline_contractuelle: deadline,
            issuer_scope: store.issuer_scope().to_string(),
        };
        match store
            .lookup_or_reserve_waiting_delegate(
                request.idempotency_key,
                &canonical_request_bytes,
                &objective,
                &waiting_delegation,
                &deferred,
            )
            .map_err(store_error)?
        {
            DelegateReservation::Created => Ok(DelegateResult::Created(DelegationCreated {
                objective_id: objective.id,
                delegation_id: waiting_delegation.id,
                message_id: None,
                participant: selected,
                duration: request.duration,
                timeout_secs,
                deadline_contractuelle: deadline,
                replayed: false,
                waiting_on_prerequisites: true,
            })),
            DelegateReservation::Replay(stored) => {
                Ok(DelegateResult::Created(created_from_stored(stored, true)))
            }
        }
    } else {
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
            .lookup_or_reserve_delegate(
                request.idempotency_key,
                &canonical_request_bytes,
                &prepared,
            )
            .map_err(store_error)?
        {
            DelegateReservation::Created => {
                if !request.depends_on.is_empty() {
                    store
                        .register_objective_dependencies(objective.id, request.depends_on)
                        .map_err(store_error)?;
                }
                Ok(DelegateResult::Created(DelegationCreated {
                    objective_id: objective.id,
                    delegation_id: delegation.id,
                    message_id: Some(message_id),
                    participant: selected,
                    duration: request.duration,
                    timeout_secs,
                    deadline_contractuelle: deadline,
                    replayed: false,
                    waiting_on_prerequisites: false,
                }))
            }
            DelegateReservation::Replay(stored) => {
                Ok(DelegateResult::Created(created_from_stored(stored, true)))
            }
        }
    }
}

fn validate_suite_and_citations(
    store: &MaicieStore,
    request: &DelegateRequest<'_>,
) -> Result<(), DelegateError> {
    if let SuiteObjective::Objectif(suite_id) = request.suite {
        let known = store
            .existing_objective_ids(&[suite_id])
            .map_err(store_error)?;
        if known.is_empty() {
            return Err(DelegateError::Invalid("objectif --suite inconnu"));
        }
    }
    let mut overlap = BTreeSet::new();
    for id in request.depends_on {
        if !overlap.insert(*id) {
            return Err(DelegateError::Invalid("--depends-on dupliqué"));
        }
    }
    for id in request.references {
        if !overlap.insert(*id) {
            return Err(DelegateError::Invalid(
                "citation classée à la fois --depends-on et --reference",
            ));
        }
    }
    for id in request.depends_on {
        let known = store.existing_objective_ids(&[*id]).map_err(store_error)?;
        if known.is_empty() {
            return Err(DelegateError::Invalid("objectif --depends-on inconnu"));
        }
    }
    for id in request.references {
        let known = store.existing_objective_ids(&[*id]).map_err(store_error)?;
        if known.is_empty() {
            return Err(DelegateError::Invalid("objectif --reference inconnu"));
        }
    }
    let cited = crate::citation::extract_uuids(request.goal);
    let known = store.existing_objective_ids(&cited).map_err(store_error)?;
    let missing =
        unclassified_known_citations(request.goal, &known, request.depends_on, request.references);
    if !missing.is_empty() {
        return Err(DelegateError::Invalid(
            "citation d'objectif non classée (--depends-on ou --reference)",
        ));
    }
    Ok(())
}

/// Fige la politique configurée après la création durable de la délégation et
/// avant son dispatch. Une définition déjà présente gagne : un rejeu ne relit
/// jamais une configuration modifiée pour réinterpréter l'historique.
pub fn pin_coordination_policy(
    store: &mut MaicieStore,
    policies: &CoordinationPoliciesConfig,
    created: &DelegationCreated,
) -> Result<(), DelegateError> {
    if store
        .coordination_snapshot(created.objective_id)
        .map_err(store_error)?
        .is_some()
    {
        return Ok(());
    }

    let configured = policies.for_duration(created.duration);
    let policy = PolitiqueReassignation {
        delegation_id: created.delegation_id,
        objectif_id: created.objective_id,
        classe: created.duration,
        version: configured.version,
        seuil_relances: configured.reminder_threshold,
        max_reemissions: configured.max_reemissions,
        chaine_repli: configured
            .fallback_chain
            .iter()
            .map(|candidate| FaitAppartenanceRepli {
                objectif_id: created.objective_id,
                participant_id: candidate.participant_id.clone(),
                membership_version: candidate.membership_version,
                est_pilote: false,
            })
            .collect(),
    };
    store
        .register_coordination_snapshot(&DefinitionCoordination {
            objectif_id: created.objective_id,
            dependencies: Vec::new(),
            policies: vec![policy],
            attentes: Vec::new(),
        })
        .map_err(store_error)
}

#[derive(Serialize)]
struct CanonicalDelegateRequest<'a> {
    v: u8,
    goal: &'a str,
    explicit_target: Option<&'a str>,
    required_tags: Vec<&'a str>,
    duration: &'static str,
    reply: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    constat_id: Option<&'a str>,
    suite: CanonicalSuite<'a>,
    depends_on: Vec<String>,
    references: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum CanonicalSuite<'a> {
    Aucune,
    Objectif(&'a str),
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
    let mut depends_on = request
        .depends_on
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>();
    depends_on.sort_unstable();
    let mut references = request
        .references
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>();
    references.sort_unstable();
    let suite_id;
    let suite = match &request.suite {
        SuiteObjective::Aucune => CanonicalSuite::Aucune,
        SuiteObjective::Objectif(id) => {
            suite_id = id.to_string();
            CanonicalSuite::Objectif(&suite_id)
        }
    };
    serde_json::to_vec(&CanonicalDelegateRequest {
        v: 2,
        goal: request.goal,
        explicit_target: request.explicit_target,
        required_tags,
        duration: duration_name(request.duration),
        reply: request.reply,
        constat_id: request.constat_id,
        suite,
        depends_on,
        references,
    })
    .map_err(|_| DelegateError::Invalid("commande delegate non sérialisable"))
}

/// Réconcilie le journal catalogue contre les faits durables du store.
///
/// Lit uniquement `delegation_arbitration_links()` et les objectifs `Clos` :
/// aucune horloge locale, aucune homonymie, aucune invention de lien. Une
/// délégation ordinaire ou une clôture sans lien n'écrit rien.
pub fn reconcile_catalogue_from_store(
    store: &MaicieStore,
    journal: &mut CatalogueJournal,
) -> Result<ReconcileReport, CatalogueReconcileError> {
    let links = store
        .delegation_arbitration_links()
        .map_err(CatalogueReconcileError::Store)?
        .into_iter()
        .map(|link| ArbitrationLink {
            constat_id: link.constat_id,
            objective_id: link.objectif_id.to_string(),
        })
        .collect::<Vec<_>>();
    let closures = attested_closures_from_store(store)?;
    journal
        .reconcile_attested_closures(&links, &closures)
        .map_err(CatalogueReconcileError::Catalogue)
}

fn attested_closures_from_store(
    store: &MaicieStore,
) -> Result<Vec<AttestedClosure>, CatalogueReconcileError> {
    let mut closures = Vec::new();
    for snapshot in store
        .objective_snapshots(None)
        .map_err(CatalogueReconcileError::Store)?
    {
        if snapshot.objective.etat != EtatObjectif::Clos {
            continue;
        }
        closures.push(AttestedClosure {
            objective_id: snapshot.objective.id.to_string(),
            observed_at: unix_secs_to_rfc3339_z(snapshot.objective.mis_a_jour_at).ok_or(
                CatalogueReconcileError::Invalid("horodatage de clôture hors borne"),
            )?,
        });
    }
    Ok(closures)
}

/// Erreurs fermées du raccord store → journal pour les clôtures attestées.
#[derive(Debug)]
pub enum CatalogueReconcileError {
    Store(StoreError),
    Catalogue(CatalogueError),
    Invalid(&'static str),
}

impl fmt::Display for CatalogueReconcileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "réconciliation catalogue : {error}"),
            Self::Catalogue(error) => write!(formatter, "réconciliation catalogue : {error}"),
            Self::Invalid(reason) => write!(formatter, "réconciliation catalogue : {reason}"),
        }
    }
}

impl std::error::Error for CatalogueReconcileError {}

/// Convertit un horodatage Unix durable (secondes) en RFC 3339 UTC (`…Z`).
///
/// Le journal n'accepte que des horodatages à fuseau explicite ; le store
/// conserve des secondes Unix. La conversion est pure et déterministe.
pub fn unix_secs_to_rfc3339_z(secs: i64) -> Option<String> {
    if secs < 0 {
        return None;
    }
    let days = secs / 86_400;
    let tod = (secs % 86_400) as u32;
    let hour = tod / 3_600;
    let minute = (tod % 3_600) / 60;
    let second = tod % 60;
    let (year, month, day) = civil_from_days_since_unix_epoch(days);
    if !(1..=9999).contains(&year) {
        return None;
    }
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

/// Algorithme civil de Howard Hinnant : jours depuis 1970-01-01 → (Y, M, D).
fn civil_from_days_since_unix_epoch(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
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
        waiting_on_prerequisites: stored.message_id.is_none(),
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
