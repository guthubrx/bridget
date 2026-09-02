//! Enveloppes métier strictes du guichet Maicie.
//!
//! Le transport Bridget valide capacité, claim et canon filaire. Ce module
//! refuse néanmoins toute opération ou charge hors matrice avant que le store
//! Maicie ne soit consulté.

use super::{
    ClasseDuree, EtatRequeteGuichet, IssueGreffe, MotifRefusGreffe, OperationGuichet,
    SuiteObjective,
};
use crate::bridget_client::{GuichetClaim, GuichetLifecycleEvent};
use bridget_transport::protocol::{
    DelegateFocus, DelegateOrigin, GuichetDurationClass, ReviewTarget, ReviewVerdictEvidence,
    ServiceRequestPayload, ServiceSuiteDeclaration,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;
use uuid::Uuid;

const MAX_IDENTIFIER_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequeteGuichet {
    DeliveryReport(RapportLivraison),
    MissionStatus { delegation_id: Uuid },
    DeadlineQuestion { delegation_id: Uuid },
    Delegate(DemandeDelegation),
    RegistreAdd(DemandeRegistreAdd),
    ObjectiveClose(DemandeObjectiveClose),
}

impl RequeteGuichet {
    pub fn operation(&self) -> OperationGuichet {
        match self {
            Self::DeliveryReport(_) => OperationGuichet::DeliveryReport,
            Self::MissionStatus { .. } => OperationGuichet::MissionStatus,
            Self::DeadlineQuestion { .. } => OperationGuichet::DeadlineQuestion,
            Self::Delegate(_) => OperationGuichet::Delegate,
            Self::RegistreAdd(_) => OperationGuichet::RegistreAdd,
            Self::ObjectiveClose(_) => OperationGuichet::ObjectiveClose,
        }
    }

    pub fn in_reply_to(&self, request_id: &str) -> String {
        match self {
            Self::DeliveryReport(report) => report.in_reply_to.clone(),
            Self::MissionStatus { .. }
            | Self::DeadlineQuestion { .. }
            | Self::Delegate(_)
            | Self::RegistreAdd(_)
            | Self::ObjectiveClose(_) => request_id.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequeteCanonique {
    pub issuer_scope: String,
    pub request_id: String,
    pub issued_at: i64,
    pub from: String,
    pub request: RequeteGuichet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RapportLivraison {
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    pub delivery_hash: String,
    pub in_reply_to: String,
    pub review_verdict: Option<ReviewVerdictEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandeDelegation {
    pub goal: String,
    pub explicit_target: Option<String>,
    pub required_tags: Vec<String>,
    pub duration: ClasseDuree,
    pub review_target: Option<ReviewTarget>,
    pub suite: SuiteObjective,
    pub depends_on: Vec<Uuid>,
    pub references: Vec<Uuid>,
    /// SPEC-087 : origine fabriquée par le daemon pour le principal humain.
    pub origin: Option<DelegateOrigin>,
    /// SPEC-087 : demande de focus, valide seulement avec une origine humaine.
    pub focus: Option<DelegateFocus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandeRegistreAdd {
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandeObjectiveClose {
    pub objective_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvenementCycleGuichet {
    pub issuer_scope: String,
    pub event_id: String,
    pub request_id: String,
    pub state: EtatRequeteGuichet,
    pub observed_at: i64,
    pub in_reply_to: Option<String>,
    pub response_message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuichetDomainError {
    InvalidEnvelope(&'static str),
    CanonicalBytesMismatch,
    UnsupportedOperation,
}

impl fmt::Display for GuichetDomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEnvelope(reason) => {
                write!(formatter, "enveloppe guichet invalide : {reason}")
            }
            Self::CanonicalBytesMismatch => {
                formatter.write_str("octets canoniques guichet divergents")
            }
            Self::UnsupportedOperation => formatter.write_str("opération guichet non admise"),
        }
    }
}

impl std::error::Error for GuichetDomainError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceRequestWire {
    #[serde(rename = "type")]
    kind: String,
    v: u8,
    issuer_scope: String,
    request_id: String,
    issued_at: i64,
    from: String,
    to: String,
    operation: String,
    payload: Value,
}

#[derive(Debug, Serialize)]
struct CanonicalServiceRequest<'a, T: Serialize> {
    #[serde(rename = "type")]
    kind: &'static str,
    v: u8,
    issuer_scope: &'a str,
    request_id: &'a str,
    issued_at: i64,
    from: &'a str,
    to: &'static str,
    operation: &'static str,
    payload: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeliveryReportPayload {
    objective_id: String,
    delegation_id: String,
    delivery_hash: String,
    in_reply_to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    review_verdict: Option<ReviewVerdictEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DelegationPayload {
    delegation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DelegatePayload {
    goal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    explicit_target: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_tags: Vec<String>,
    duration: GuichetDurationClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    review_target: Option<ReviewTarget>,
    suite: ServiceSuiteDeclaration,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    references: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    origin: Option<DelegateOrigin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    focus: Option<DelegateFocus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistreAddPayload {
    line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectiveClosePayload {
    objective_id: String,
    reason: String,
}

pub fn parse_claim(claim: &GuichetClaim) -> Result<RequeteCanonique, GuichetDomainError> {
    let wire: ServiceRequestWire = serde_json::from_slice(&claim.canonical_request)
        .map_err(|_| GuichetDomainError::InvalidEnvelope("JSON ou champs invalides"))?;
    if wire.kind != "service_request" || wire.to != "maicie" {
        return Err(GuichetDomainError::InvalidEnvelope(
            "type, version ou cible invalide",
        ));
    }
    match (wire.v, wire.operation.as_str()) {
        (1, _) | (2, "delegate") => {}
        _ => {
            return Err(GuichetDomainError::InvalidEnvelope(
                "version incompatible avec l’opération",
            ));
        }
    }
    validate_identifier(&wire.issuer_scope)?;
    validate_identifier(&wire.request_id)?;
    validate_identifier(&wire.from)?;
    if wire.issued_at <= 0
        || wire.issuer_scope != claim.issuer_scope
        || wire.request_id != claim.request_id
    {
        return Err(GuichetDomainError::InvalidEnvelope(
            "claim et requête divergents",
        ));
    }

    let request = match wire.operation.as_str() {
        "delivery_report" => {
            let payload: DeliveryReportPayload = serde_json::from_value(wire.payload.clone())
                .map_err(|_| {
                    GuichetDomainError::InvalidEnvelope("rapport de livraison invalide")
                })?;
            validate_identifier(&payload.in_reply_to)?;
            if payload.delivery_hash.len() != 64
                || !payload
                    .delivery_hash
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "hash de livraison invalide",
                ));
            }
            if payload
                .review_verdict
                .as_ref()
                .is_some_and(|evidence| !evidence.is_valid())
            {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "attestation Git de revue invalide",
                ));
            }
            let objective_id = parse_uuid(&payload.objective_id)?;
            let delegation_id = parse_uuid(&payload.delegation_id)?;
            ensure_canonical(&claim.canonical_request, &wire, "delivery_report", &payload)?;
            RequeteGuichet::DeliveryReport(RapportLivraison {
                objective_id,
                delegation_id,
                delivery_hash: payload.delivery_hash,
                in_reply_to: payload.in_reply_to,
                review_verdict: payload.review_verdict,
            })
        }
        "mission_status" => {
            let payload: DelegationPayload = serde_json::from_value(wire.payload.clone())
                .map_err(|_| GuichetDomainError::InvalidEnvelope("statut de mission invalide"))?;
            let delegation_id = parse_uuid(&payload.delegation_id)?;
            ensure_canonical(&claim.canonical_request, &wire, "mission_status", &payload)?;
            RequeteGuichet::MissionStatus { delegation_id }
        }
        "deadline_question" => {
            let payload: DelegationPayload = serde_json::from_value(wire.payload.clone())
                .map_err(|_| GuichetDomainError::InvalidEnvelope("question d’échéance invalide"))?;
            let delegation_id = parse_uuid(&payload.delegation_id)?;
            ensure_canonical(
                &claim.canonical_request,
                &wire,
                "deadline_question",
                &payload,
            )?;
            RequeteGuichet::DeadlineQuestion { delegation_id }
        }
        "delegate" => {
            let payload: DelegatePayload = serde_json::from_value(wire.payload.clone())
                .map_err(|_| GuichetDomainError::InvalidEnvelope("délégation invalide"))?;
            // SPEC-087 : origine et focus voyagent en v2, jamais en v1.
            if wire.v == 1 && (payload.origin.is_some() || payload.focus.is_some()) {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "origine ou focus exigent la version 2",
                ));
            }
            match (wire.v, payload.review_target.as_ref()) {
                (1, None) => {}
                (2, None) if payload.origin.is_some() || payload.focus.is_some() => {}
                (2, Some(target)) if target.is_valid() => {}
                (1, Some(_)) => {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "cible de revue interdite en version historique",
                    ));
                }
                (2, None) => {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "cible de revue absente en version 2",
                    ));
                }
                (2, Some(_)) => {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "cible de revue invalide",
                    ));
                }
                _ => {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "version incompatible avec la délégation",
                    ));
                }
            }
            if payload.goal.trim().is_empty() || payload.goal.len() > 16 * 1024 {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "but de délégation invalide",
                ));
            }
            if let Some(target) = &payload.explicit_target {
                validate_identifier(target)?;
            }
            if payload.required_tags.len() > 32
                || payload
                    .required_tags
                    .iter()
                    .any(|tag| validate_identifier(tag).is_err())
                || payload.required_tags.iter().collect::<BTreeSet<_>>().len()
                    != payload.required_tags.len()
            {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "tags de délégation invalides",
                ));
            }
            if payload.depends_on.len() > 100 || payload.references.len() > 100 {
                return Err(GuichetDomainError::InvalidEnvelope("relations hors borne"));
            }
            let depends_on = payload
                .depends_on
                .iter()
                .map(|value| parse_uuid(value))
                .collect::<Result<Vec<_>, _>>()?;
            let references = payload
                .references
                .iter()
                .map(|value| parse_uuid(value))
                .collect::<Result<Vec<_>, _>>()?;
            let mut relations = BTreeSet::new();
            if depends_on
                .iter()
                .chain(&references)
                .any(|id| !relations.insert(*id))
            {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "relation de délégation dupliquée",
                ));
            }
            let suite = match &payload.suite {
                ServiceSuiteDeclaration::Aucune => SuiteObjective::Aucune,
                ServiceSuiteDeclaration::Objectif { objective_id } => {
                    SuiteObjective::Objectif(parse_uuid(objective_id)?)
                }
            };
            let duration = match payload.duration {
                GuichetDurationClass::Courte => ClasseDuree::Courte,
                GuichetDurationClass::Normale => ClasseDuree::Normale,
                GuichetDurationClass::Longue => ClasseDuree::Longue,
            };
            // La forme canonique appartient au contrat filaire public. Le
            // type privé ci-dessus garde le décodage strict, mais son ordre de
            // champs ne doit jamais créer une seconde canonisation.
            let canonical_payload = ServiceRequestPayload::Delegate {
                goal: payload.goal.clone(),
                review_target: payload.review_target.clone(),
                explicit_target: payload.explicit_target.clone(),
                required_tags: payload.required_tags.clone(),
                duration: payload.duration,
                suite: payload.suite.clone(),
                depends_on: payload.depends_on.clone(),
                references: payload.references.clone(),
                origin: payload.origin.clone(),
                focus: payload.focus.clone(),
            };
            ensure_canonical(
                &claim.canonical_request,
                &wire,
                "delegate",
                &canonical_payload,
            )?;
            RequeteGuichet::Delegate(DemandeDelegation {
                goal: payload.goal,
                explicit_target: payload.explicit_target,
                required_tags: payload.required_tags,
                duration,
                review_target: payload.review_target,
                suite,
                depends_on,
                references,
                origin: payload.origin,
                focus: payload.focus,
            })
        }
        "registre_add" => {
            let payload: RegistreAddPayload = serde_json::from_value(wire.payload.clone())
                .map_err(|_| GuichetDomainError::InvalidEnvelope("entrée de registre invalide"))?;
            if payload.line.trim().is_empty() || payload.line.len() > 48 * 1024 {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "ligne de registre hors borne",
                ));
            }
            ensure_canonical(&claim.canonical_request, &wire, "registre_add", &payload)?;
            RequeteGuichet::RegistreAdd(DemandeRegistreAdd { line: payload.line })
        }
        "objective_close" => {
            let payload: ObjectiveClosePayload = serde_json::from_value(wire.payload.clone())
                .map_err(|_| GuichetDomainError::InvalidEnvelope("clôture d'objectif invalide"))?;
            let objective_id = parse_uuid(&payload.objective_id)?;
            if payload.reason.trim().is_empty() || payload.reason.len() > 16 * 1024 {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "motif de clôture hors borne",
                ));
            }
            ensure_canonical(&claim.canonical_request, &wire, "objective_close", &payload)?;
            RequeteGuichet::ObjectiveClose(DemandeObjectiveClose {
                objective_id,
                reason: payload.reason,
            })
        }
        _ => return Err(GuichetDomainError::UnsupportedOperation),
    };

    Ok(RequeteCanonique {
        issuer_scope: wire.issuer_scope,
        request_id: wire.request_id,
        issued_at: wire.issued_at,
        from: wire.from,
        request,
    })
}

pub fn parse_lifecycle_event(
    event: &GuichetLifecycleEvent,
) -> Result<EvenementCycleGuichet, GuichetDomainError> {
    validate_identifier(&event.issuer_scope)?;
    validate_identifier(&event.event_id)?;
    validate_identifier(&event.request_id)?;
    if event.observed_at <= 0 {
        return Err(GuichetDomainError::InvalidEnvelope(
            "horodatage d’événement invalide",
        ));
    }
    let state = match event.state.as_str() {
        "answered" => EtatRequeteGuichet::Answered,
        "cancelled" => EtatRequeteGuichet::Cancelled,
        "timed_out" => EtatRequeteGuichet::TimedOut,
        _ => {
            return Err(GuichetDomainError::InvalidEnvelope(
                "état terminal invalide",
            ));
        }
    };
    match (&event.in_reply_to, &event.response_message_id) {
        (Some(in_reply_to), Some(response_message_id)) => {
            validate_identifier(in_reply_to)?;
            validate_identifier(response_message_id)?;
        }
        (None, None) => {}
        _ => {
            return Err(GuichetDomainError::InvalidEnvelope(
                "corrélation terminale partielle",
            ));
        }
    }
    Ok(EvenementCycleGuichet {
        issuer_scope: event.issuer_scope.clone(),
        event_id: event.event_id.clone(),
        request_id: event.request_id.clone(),
        state,
        observed_at: event.observed_at,
        in_reply_to: event.in_reply_to.clone(),
        response_message_id: event.response_message_id.clone(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionCoordinationState {
    Open,
    EnCoordination,
    AEvaluer,
    Synthetise,
    Clos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionLocalDeliveryState {
    Pending,
    Accepted,
    OutcomeUnknown,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionLocalDelivery {
    pub state: ProjectionLocalDeliveryState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionTransportState {
    Connected,
    Unavailable,
    Gap,
    Ended,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionTransportObservation {
    pub state: ProjectionTransportState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_state: Option<String>,
    pub observed_at: i64,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionFreshness {
    Fresh,
    Gap,
    Ended,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionDurationClass {
    Courte,
    Normale,
    Longue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectionReply {
    MissionStatus {
        delegation_id: String,
        objective_id: String,
        coordination_state: ProjectionCoordinationState,
        local_delivery: ProjectionLocalDelivery,
        #[serde(skip_serializing_if = "Option::is_none")]
        transport_observation: Option<ProjectionTransportObservation>,
        freshness: ProjectionFreshness,
    },
    DeadlineQuestion {
        delegation_id: String,
        duration_class: ProjectionDurationClass,
        deadline_at: i64,
    },
}

impl ProjectionReply {
    pub fn operation(&self) -> OperationGuichet {
        match self {
            Self::MissionStatus { .. } => OperationGuichet::MissionStatus,
            Self::DeadlineQuestion { .. } => OperationGuichet::DeadlineQuestion,
        }
    }

    pub fn delegation_id(&self) -> Result<Uuid, GuichetDomainError> {
        let value = match self {
            Self::MissionStatus { delegation_id, .. }
            | Self::DeadlineQuestion { delegation_id, .. } => delegation_id,
        };
        parse_uuid(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegateMutationStatus {
    Created,
    SelectionRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistreAddMutationStatus {
    Appended,
    IdempotentNoop,
}

/// Réponse terminale d'une mutation du greffe. Les identifiants absents dans
/// `selection_required` restent réellement absents ; aucun UUID n'est inventé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MutationReply {
    Delegate {
        status: DelegateMutationStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        objective_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delegation_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        participant: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        candidates: Vec<String>,
        waiting_on_prerequisites: bool,
        replayed: bool,
    },
    RegistreAdd {
        status: RegistreAddMutationStatus,
        constat_id: String,
    },
    ObjectiveClose {
        objective_id: String,
        decision_id: String,
        replayed: bool,
    },
}

impl MutationReply {
    pub fn operation(&self) -> OperationGuichet {
        match self {
            Self::Delegate { .. } => OperationGuichet::Delegate,
            Self::RegistreAdd { .. } => OperationGuichet::RegistreAdd,
            Self::ObjectiveClose { .. } => OperationGuichet::ObjectiveClose,
        }
    }

    pub fn objective_id(&self) -> Result<Option<Uuid>, GuichetDomainError> {
        match self {
            Self::Delegate { objective_id, .. } => {
                objective_id.as_deref().map(parse_uuid).transpose()
            }
            Self::RegistreAdd { .. } => Ok(None),
            Self::ObjectiveClose { objective_id, .. } => parse_uuid(objective_id).map(Some),
        }
    }

    pub fn delegation_id(&self) -> Result<Option<Uuid>, GuichetDomainError> {
        match self {
            Self::Delegate { delegation_id, .. } => {
                delegation_id.as_deref().map(parse_uuid).transpose()
            }
            Self::RegistreAdd { .. } | Self::ObjectiveClose { .. } => Ok(None),
        }
    }

    pub fn decision_id(&self) -> Result<Option<Uuid>, GuichetDomainError> {
        match self {
            Self::ObjectiveClose { decision_id, .. } => parse_uuid(decision_id).map(Some),
            Self::Delegate { .. } | Self::RegistreAdd { .. } => Ok(None),
        }
    }
}

#[derive(Serialize)]
struct GuichetReplyWire<'a, T: Serialize> {
    #[serde(rename = "type")]
    kind: &'static str,
    v: u8,
    issuer_scope: &'a str,
    request_id: &'a str,
    claim_generation: u64,
    claim_token: &'a str,
    response_message_id: &'a str,
    in_reply_to: &'a str,
    outcome: &'static str,
    payload: &'a T,
}

#[derive(Serialize)]
struct DeliveryReplyPayload<'a> {
    kind: &'static str,
    objective_id: String,
    delegation_id: String,
    delivery_hash: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_verdict: Option<&'a ReviewVerdictEvidence>,
}

#[derive(Serialize)]
struct RefusalReplyPayload {
    kind: &'static str,
    operation: &'static str,
    reason: &'static str,
}

pub fn delivery_reply_bytes(
    claim: &GuichetClaim,
    report: &RapportLivraison,
    response_message_id: &str,
    issue: IssueGreffe,
) -> Result<Vec<u8>, GuichetDomainError> {
    validate_identifier(response_message_id)?;
    validate_identifier(&claim.claim_token)?;
    let outcome = match issue {
        IssueGreffe::Accepted => "accepted",
        IssueGreffe::DemandeDejaTerminale => "request_already_terminal",
        IssueGreffe::Refusee => "refused",
    };
    let payload = DeliveryReplyPayload {
        kind: "delivery_report",
        objective_id: report.objective_id.to_string(),
        delegation_id: report.delegation_id.to_string(),
        delivery_hash: &report.delivery_hash,
        review_verdict: report.review_verdict.as_ref(),
    };
    serde_json::to_vec(&GuichetReplyWire {
        kind: "guichet_reply",
        v: 1,
        issuer_scope: &claim.issuer_scope,
        request_id: &claim.request_id,
        claim_generation: claim.claim_generation,
        claim_token: &claim.claim_token,
        response_message_id,
        in_reply_to: &report.in_reply_to,
        outcome,
        payload: &payload,
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("réponse non sérialisable"))
}

/// Produit le reçu fermé d'un claim bien formé mais impossible à appliquer au
/// registre local. Aucun fait métier n'est inventé : la charge ne contient que
/// l'opération demandée et le motif attesté par la greffe.
pub fn refusal_reply_bytes(
    claim: &GuichetClaim,
    response_message_id: &str,
    request: &RequeteCanonique,
    reason: MotifRefusGreffe,
) -> Result<Vec<u8>, GuichetDomainError> {
    validate_identifier(response_message_id)?;
    validate_identifier(&claim.claim_token)?;
    let operation = request.request.operation().as_sql();
    let reason = reason.as_sql();
    let in_reply_to = request.request.in_reply_to(&request.request_id);
    serde_json::to_vec(&GuichetReplyWire {
        kind: "guichet_reply",
        v: 1,
        issuer_scope: &claim.issuer_scope,
        request_id: &claim.request_id,
        claim_generation: claim.claim_generation,
        claim_token: &claim.claim_token,
        response_message_id,
        in_reply_to: &in_reply_to,
        outcome: "refused",
        payload: &RefusalReplyPayload {
            kind: "refused",
            operation,
            reason,
        },
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("refus non sérialisable"))
}

pub fn projection_reply_bytes(
    claim: &GuichetClaim,
    response_message_id: &str,
    projection: &ProjectionReply,
) -> Result<Vec<u8>, GuichetDomainError> {
    validate_identifier(response_message_id)?;
    validate_identifier(&claim.claim_token)?;
    validate_projection(projection)?;
    serde_json::to_vec(&GuichetReplyWire {
        kind: "guichet_reply",
        v: 1,
        issuer_scope: &claim.issuer_scope,
        request_id: &claim.request_id,
        claim_generation: claim.claim_generation,
        claim_token: &claim.claim_token,
        response_message_id,
        in_reply_to: &claim.request_id,
        outcome: "accepted",
        payload: projection,
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("projection non sérialisable"))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredProjectionReplyWire {
    #[serde(rename = "type")]
    kind: String,
    v: u8,
    issuer_scope: String,
    request_id: String,
    claim_generation: u64,
    claim_token: String,
    response_message_id: String,
    in_reply_to: String,
    outcome: String,
    payload: ProjectionReply,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredMutationReplyWire {
    #[serde(rename = "type")]
    kind: String,
    v: u8,
    issuer_scope: String,
    request_id: String,
    claim_generation: u64,
    claim_token: String,
    response_message_id: String,
    in_reply_to: String,
    outcome: String,
    payload: MutationReply,
}

pub fn mutation_reply_bytes(
    claim: &GuichetClaim,
    response_message_id: &str,
    reply: &MutationReply,
) -> Result<Vec<u8>, GuichetDomainError> {
    validate_identifier(response_message_id)?;
    validate_identifier(&claim.claim_token)?;
    validate_mutation_reply(reply)?;
    serde_json::to_vec(&GuichetReplyWire {
        kind: "guichet_reply",
        v: 1,
        issuer_scope: &claim.issuer_scope,
        request_id: &claim.request_id,
        claim_generation: claim.claim_generation,
        claim_token: &claim.claim_token,
        response_message_id,
        in_reply_to: &claim.request_id,
        outcome: "accepted",
        payload: reply,
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("réponse de mutation non sérialisable"))
}

/// Régénère seulement l'enveloppe de lease d'une mutation déjà durable.
pub fn reclaim_mutation_reply_bytes(
    claim: &GuichetClaim,
    stored_reply: &[u8],
) -> Result<(Vec<u8>, MutationReply, String), GuichetDomainError> {
    let stored: StoredMutationReplyWire = serde_json::from_slice(stored_reply)
        .map_err(|_| GuichetDomainError::InvalidEnvelope("mutation durable invalide"))?;
    if stored.kind != "guichet_reply"
        || stored.v != 1
        || stored.issuer_scope != claim.issuer_scope
        || stored.request_id != claim.request_id
        || stored.in_reply_to != claim.request_id
        || stored.outcome != "accepted"
        || stored.claim_generation == 0
        || stored.claim_token.is_empty()
    {
        return Err(GuichetDomainError::InvalidEnvelope(
            "enveloppe de mutation durable divergente",
        ));
    }
    validate_identifier(&stored.response_message_id)?;
    validate_mutation_reply(&stored.payload)?;
    let bytes = mutation_reply_bytes(claim, &stored.response_message_id, &stored.payload)?;
    Ok((bytes, stored.payload, stored.response_message_id))
}

fn validate_mutation_reply(reply: &MutationReply) -> Result<(), GuichetDomainError> {
    match reply {
        MutationReply::Delegate {
            status,
            objective_id,
            delegation_id,
            message_id,
            participant,
            candidates,
            waiting_on_prerequisites,
            ..
        } => match status {
            DelegateMutationStatus::Created => {
                let objective_id =
                    objective_id
                        .as_deref()
                        .ok_or(GuichetDomainError::InvalidEnvelope(
                            "délégation sans objectif",
                        ))?;
                let delegation_id =
                    delegation_id
                        .as_deref()
                        .ok_or(GuichetDomainError::InvalidEnvelope(
                            "délégation sans identifiant",
                        ))?;
                parse_uuid(objective_id)?;
                parse_uuid(delegation_id)?;
                if let Some(message_id) = message_id {
                    parse_uuid(message_id)?;
                }
                if participant
                    .as_deref()
                    .is_none_or(|value| validate_identifier(value).is_err())
                    || !candidates.is_empty()
                    || (*waiting_on_prerequisites && message_id.is_some())
                {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "résultat de délégation incohérent",
                    ));
                }
            }
            DelegateMutationStatus::SelectionRequired => {
                if objective_id.is_some()
                    || delegation_id.is_some()
                    || message_id.is_some()
                    || participant.is_some()
                    || *waiting_on_prerequisites
                    || candidates.len() > 128
                    || candidates
                        .iter()
                        .any(|candidate| validate_identifier(candidate).is_err())
                {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "sélection de délégation incohérente",
                    ));
                }
            }
        },
        MutationReply::RegistreAdd { constat_id, .. } => validate_identifier(constat_id)?,
        MutationReply::ObjectiveClose {
            objective_id,
            decision_id,
            ..
        } => {
            parse_uuid(objective_id)?;
            parse_uuid(decision_id)?;
        }
    }
    Ok(())
}

/// Régénère uniquement l'enveloppe de claim d'une projection déjà durable.
/// Le payload métier et le response_message_id restent ceux du premier reçu.
pub fn reclaim_projection_reply_bytes(
    claim: &GuichetClaim,
    stored_reply: &[u8],
) -> Result<(Vec<u8>, ProjectionReply, String), GuichetDomainError> {
    let stored: StoredProjectionReplyWire = serde_json::from_slice(stored_reply)
        .map_err(|_| GuichetDomainError::InvalidEnvelope("projection durable illisible"))?;
    if stored.kind != "guichet_reply"
        || stored.v != 1
        || stored.issuer_scope != claim.issuer_scope
        || stored.request_id != claim.request_id
        || stored.in_reply_to != claim.request_id
        || stored.outcome != "accepted"
        || stored.claim_generation >= claim.claim_generation
        || stored.claim_token.is_empty()
    {
        return Err(GuichetDomainError::InvalidEnvelope(
            "projection durable divergente",
        ));
    }
    let bytes = projection_reply_bytes(claim, &stored.response_message_id, &stored.payload)?;
    Ok((bytes, stored.payload, stored.response_message_id))
}

fn validate_projection(projection: &ProjectionReply) -> Result<(), GuichetDomainError> {
    match projection {
        ProjectionReply::MissionStatus {
            delegation_id,
            objective_id,
            local_delivery,
            transport_observation,
            ..
        } => {
            parse_uuid(delegation_id)?;
            parse_uuid(objective_id)?;
            if let Some(issue) = &local_delivery.issue {
                validate_identifier(issue)?;
            }
            if local_delivery.observed_at.is_some_and(|value| value <= 0) {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "horodatage de remise invalide",
                ));
            }
            if let Some(observation) = transport_observation {
                if observation.observed_at <= 0 || observation.source.is_empty() {
                    return Err(GuichetDomainError::InvalidEnvelope(
                        "observation transport invalide",
                    ));
                }
                validate_identifier(&observation.source)?;
                if let Some(request_state) = &observation.request_state {
                    validate_identifier(request_state)?;
                }
                if let Some(subscription_id) = &observation.subscription_id {
                    validate_identifier(subscription_id)?;
                }
            }
        }
        ProjectionReply::DeadlineQuestion {
            delegation_id,
            deadline_at,
            ..
        } => {
            parse_uuid(delegation_id)?;
            if *deadline_at <= 0 {
                return Err(GuichetDomainError::InvalidEnvelope(
                    "échéance contractuelle invalide",
                ));
            }
        }
    }
    Ok(())
}

/// SPEC-087 : octets canoniques de la requête de délégation **sans** son
/// origine, tels que le daemon les a hachés avant de fabriquer l'attestation.
/// Le hash ne se définit jamais sur un document qui porte déjà la preuve.
pub fn canonical_delegate_bytes_without_origin(
    canonical: &RequeteCanonique,
    request: &DemandeDelegation,
) -> Result<Vec<u8>, GuichetDomainError> {
    let duration = match request.duration {
        ClasseDuree::Courte => GuichetDurationClass::Courte,
        ClasseDuree::Normale => GuichetDurationClass::Normale,
        ClasseDuree::Longue => GuichetDurationClass::Longue,
    };
    let suite = match &request.suite {
        SuiteObjective::Aucune => ServiceSuiteDeclaration::Aucune,
        SuiteObjective::Objectif(id) => ServiceSuiteDeclaration::Objectif {
            objective_id: id.to_string(),
        },
    };
    let payload = ServiceRequestPayload::Delegate {
        goal: request.goal.clone(),
        review_target: request.review_target.clone(),
        explicit_target: request.explicit_target.clone(),
        required_tags: request.required_tags.clone(),
        duration,
        suite,
        depends_on: request.depends_on.iter().map(ToString::to_string).collect(),
        references: request.references.iter().map(ToString::to_string).collect(),
        origin: None,
        focus: request.focus.clone(),
    };
    serde_json::to_vec(&CanonicalServiceRequest {
        kind: "service_request",
        v: 2,
        issuer_scope: &canonical.issuer_scope,
        request_id: &canonical.request_id,
        issued_at: canonical.issued_at,
        from: &canonical.from,
        to: "maicie",
        operation: "delegate",
        payload,
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("requête non sérialisable"))
}

fn ensure_canonical<T: Serialize>(
    original: &[u8],
    wire: &ServiceRequestWire,
    operation: &'static str,
    payload: &T,
) -> Result<(), GuichetDomainError> {
    let canonical = serde_json::to_vec(&CanonicalServiceRequest {
        kind: "service_request",
        v: wire.v,
        issuer_scope: &wire.issuer_scope,
        request_id: &wire.request_id,
        issued_at: wire.issued_at,
        from: &wire.from,
        to: "maicie",
        operation,
        payload,
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("requête non sérialisable"))?;
    if canonical == original {
        Ok(())
    } else {
        Err(GuichetDomainError::CanonicalBytesMismatch)
    }
}

fn validate_identifier(value: &str) -> Result<(), GuichetDomainError> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(GuichetDomainError::InvalidEnvelope("identifiant invalide"));
    }
    Ok(())
}

fn parse_uuid(value: &str) -> Result<Uuid, GuichetDomainError> {
    if value.len() > MAX_IDENTIFIER_BYTES {
        return Err(GuichetDomainError::InvalidEnvelope("référence trop longue"));
    }
    Uuid::parse_str(value).map_err(|_| GuichetDomainError::InvalidEnvelope("référence invalide"))
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    fn claim(canonical_request: Vec<u8>, generation: u64, token: &str) -> GuichetClaim {
        GuichetClaim {
            issuer_scope: "scope-test".to_string(),
            request_id: "request-test".to_string(),
            canonical_request,
            authorization_attestation: None,
            claimed_at: 1_000,
            claim_generation: generation,
            claim_token: token.to_string(),
            claim_lease_expires_at: 1_060,
            expires_at: 2_000,
        }
    }

    fn canonical<T: Serialize>(operation: &'static str, payload: &T) -> Vec<u8> {
        canonical_version(1, operation, payload)
    }

    fn canonical_version<T: Serialize>(
        version: u8,
        operation: &'static str,
        payload: &T,
    ) -> Vec<u8> {
        serde_json::to_vec(&CanonicalServiceRequest {
            kind: "service_request",
            v: version,
            issuer_scope: "scope-test",
            request_id: "request-test",
            issued_at: 1_000,
            from: "agent-test",
            to: "maicie",
            operation,
            payload,
        })
        .unwrap()
    }

    fn delegate_payload(review_target: Option<ReviewTarget>) -> ServiceRequestPayload {
        ServiceRequestPayload::Delegate {
            goal: "relire la tête gelée".to_string(),
            review_target,
            explicit_target: Some("reviewer".to_string()),
            required_tags: vec!["review".to_string()],
            duration: GuichetDurationClass::Normale,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: Vec::new(),
            references: Vec::new(),
            origin: None,
            focus: None,
        }
    }

    #[test]
    fn spec_047_version_de_revue_ne_peut_pas_perdre_sa_cible() {
        let target = ReviewTarget {
            target_ref: "origin/session-047-verdict-tete-reecrite".to_string(),
            expected_head: "a".repeat(40),
        };
        let modern_payload = delegate_payload(Some(target.clone()));
        let modern = claim(
            canonical_version(2, "delegate", &modern_payload),
            1,
            "token-modern",
        );
        let RequeteGuichet::Delegate(parsed) = parse_claim(&modern).unwrap().request else {
            panic!("délégation moderne attendue")
        };
        assert_eq!(parsed.review_target, Some(target.clone()));

        let legacy_with_target = claim(
            canonical("delegate", &modern_payload),
            1,
            "token-legacy-target",
        );
        assert!(matches!(
            parse_claim(&legacy_with_target),
            Err(GuichetDomainError::InvalidEnvelope(
                "cible de revue interdite en version historique"
            ))
        ));

        let modern_without_target = delegate_payload(None);
        let modern_without_target = claim(
            canonical_version(2, "delegate", &modern_without_target),
            1,
            "token-modern-empty",
        );
        assert!(matches!(
            parse_claim(&modern_without_target),
            Err(GuichetDomainError::InvalidEnvelope(
                "cible de revue absente en version 2"
            ))
        ));

        let status = DelegationPayload {
            delegation_id: Uuid::new_v4().to_string(),
        };
        let modern_status = claim(
            canonical_version(2, "mission_status", &status),
            1,
            "token-modern-status",
        );
        assert!(matches!(
            parse_claim(&modern_status),
            Err(GuichetDomainError::InvalidEnvelope(
                "version incompatible avec l’opération"
            ))
        ));
    }

    #[test]
    fn spec_026_parse_les_deux_nouvelles_mutations_sans_chemin_ni_principal() {
        let registre_payload = RegistreAddPayload {
            line: r#"{"v":1,"kind":"add"}"#.to_string(),
        };
        let registre = claim(canonical("registre_add", &registre_payload), 1, "token-a");
        assert!(matches!(
            parse_claim(&registre).unwrap().request,
            RequeteGuichet::RegistreAdd(DemandeRegistreAdd { line })
                if line == registre_payload.line
        ));

        let objective_id = Uuid::new_v4();
        let close_payload = ObjectiveClosePayload {
            objective_id: objective_id.to_string(),
            reason: "preuve centrale attestée".to_string(),
        };
        let close = claim(canonical("objective_close", &close_payload), 1, "token-b");
        assert!(matches!(
            parse_claim(&close).unwrap().request,
            RequeteGuichet::ObjectiveClose(DemandeObjectiveClose { objective_id: parsed, reason })
                if parsed == objective_id && reason == close_payload.reason
        ));
    }

    #[test]
    fn spec_026_reclaim_mutation_renouvelle_la_lease_sans_reconstruire_le_payload() {
        let first = claim(Vec::new(), 1, "token-first");
        let reply = MutationReply::Delegate {
            status: DelegateMutationStatus::Created,
            objective_id: Some(Uuid::new_v4().to_string()),
            delegation_id: Some(Uuid::new_v4().to_string()),
            message_id: Some(Uuid::new_v4().to_string()),
            participant: Some("agent-test".to_string()),
            candidates: Vec::new(),
            waiting_on_prerequisites: false,
            replayed: false,
        };
        let stored = mutation_reply_bytes(&first, "response-test", &reply).unwrap();
        let replay = claim(Vec::new(), 2, "token-second");
        let (bytes, parsed, response_message_id) =
            reclaim_mutation_reply_bytes(&replay, &stored).unwrap();

        assert_eq!(parsed, reply);
        assert_eq!(response_message_id, "response-test");
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["claim_generation"], 2);
        assert_eq!(value["claim_token"], "token-second");
        assert_eq!(value["payload"], serde_json::to_value(reply).unwrap());
    }

    #[test]
    fn spec_026_selection_requise_n_invente_aucun_identifiant() {
        let current = claim(Vec::new(), 1, "token-selection");
        let reply = MutationReply::Delegate {
            status: DelegateMutationStatus::SelectionRequired,
            objective_id: None,
            delegation_id: None,
            message_id: None,
            participant: None,
            candidates: vec!["agent-a".to_string(), "agent-b".to_string()],
            waiting_on_prerequisites: false,
            replayed: false,
        };
        let bytes = mutation_reply_bytes(&current, "response-selection", &reply).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(value["payload"].get("objective_id").is_none());
        assert!(value["payload"].get("delegation_id").is_none());
        assert!(value["payload"].get("message_id").is_none());
    }
}
