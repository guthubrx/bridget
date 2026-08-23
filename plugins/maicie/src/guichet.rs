//! Enveloppes métier strictes du guichet Maicie.
//!
//! Le transport Bridget valide capacité, claim et canon filaire. Ce module
//! refuse néanmoins toute opération ou charge hors matrice avant que le store
//! Maicie ne soit consulté.

use super::{EtatRequeteGuichet, IssueGreffe, OperationGuichet};
use crate::bridget_client::{GuichetClaim, GuichetLifecycleEvent};
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
            let objective_id = parse_uuid(&payload.objective_id)?;
            let delegation_id = parse_uuid(&payload.delegation_id)?;
            ensure_canonical(&claim.canonical_request, &wire, "delivery_report", &payload)?;
            RequeteGuichet::DeliveryReport(RapportLivraison {
                objective_id,
                delegation_id,
                delivery_hash: payload.delivery_hash,
                in_reply_to: payload.in_reply_to,
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

#[derive(Serialize)]
struct GuichetReplyWire<'a> {
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
    payload: DeliveryReplyPayload<'a>,
}

#[derive(Serialize)]
struct DeliveryReplyPayload<'a> {
    kind: &'static str,
    objective_id: String,
    delegation_id: String,
    delivery_hash: &'a str,
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
        payload: DeliveryReplyPayload {
            kind: "delivery_report",
            objective_id: report.objective_id.to_string(),
            delegation_id: report.delegation_id.to_string(),
            delivery_hash: &report.delivery_hash,
        },
    })
    .map_err(|_| GuichetDomainError::InvalidEnvelope("réponse non sérialisable"))
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
