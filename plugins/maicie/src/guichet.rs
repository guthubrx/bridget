//! Enveloppes métier strictes du guichet Maicie.
//!
//! Le transport Bridget valide capacité, claim et canon filaire. Ce module
//! refuse néanmoins toute opération ou charge hors matrice avant que le store
//! Maicie ne soit consulté.

use super::{EtatRequeteGuichet, IssueGreffe, MotifRefusGreffe, OperationGuichet};
use crate::bridget_client::{GuichetClaim, GuichetLifecycleEvent};
use bridget_transport::protocol::ReviewVerdictEvidence;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

const MAX_IDENTIFIER_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequeteGuichet {
    DeliveryReport(RapportLivraison),
    MissionStatus { delegation_id: Uuid },
    DeadlineQuestion { delegation_id: Uuid },
}

impl RequeteGuichet {
    pub fn operation(&self) -> OperationGuichet {
        match self {
            Self::DeliveryReport(_) => OperationGuichet::DeliveryReport,
            Self::MissionStatus { .. } => OperationGuichet::MissionStatus,
            Self::DeadlineQuestion { .. } => OperationGuichet::DeadlineQuestion,
        }
    }

    pub fn in_reply_to(&self, request_id: &str) -> String {
        match self {
            Self::DeliveryReport(report) => report.in_reply_to.clone(),
            Self::MissionStatus { .. } | Self::DeadlineQuestion { .. } => request_id.to_string(),
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

pub fn parse_claim(claim: &GuichetClaim) -> Result<RequeteCanonique, GuichetDomainError> {
    let wire: ServiceRequestWire = serde_json::from_slice(&claim.canonical_request)
        .map_err(|_| GuichetDomainError::InvalidEnvelope("JSON ou champs invalides"))?;
    if wire.kind != "service_request" || wire.v != 1 || wire.to != "maicie" {
        return Err(GuichetDomainError::InvalidEnvelope(
            "type, version ou cible invalide",
        ));
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
    let operation = match request.request.operation() {
        OperationGuichet::DeliveryReport => "delivery_report",
        OperationGuichet::MissionStatus => "mission_status",
        OperationGuichet::DeadlineQuestion => "deadline_question",
    };
    let reason = match reason {
        MotifRefusGreffe::DelegationAbsente => "delegation_missing",
        MotifRefusGreffe::RelationsInvalides => "relation_invalid",
        MotifRefusGreffe::EnveloppeDivergente => "envelope_mismatch",
        MotifRefusGreffe::VerdictRevueRequis => "review_verdict_required",
        MotifRefusGreffe::VerdictRevueInattendu => "review_verdict_unexpected",
        MotifRefusGreffe::MandatRevueDivergent => "review_mandate_mismatch",
        MotifRefusGreffe::TeteCibleDeplacee => "target_head_moved",
        MotifRefusGreffe::TeteCibleDeplaceeEtTeteMesureeDivergente => {
            "target_head_moved_and_measured_head_mismatch"
        }
        MotifRefusGreffe::TeteMesureeDivergente => "measured_head_mismatch",
    };
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

fn ensure_canonical<T: Serialize>(
    original: &[u8],
    wire: &ServiceRequestWire,
    operation: &'static str,
    payload: &T,
) -> Result<(), GuichetDomainError> {
    let canonical = serde_json::to_vec(&CanonicalServiceRequest {
        kind: "service_request",
        v: 1,
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
