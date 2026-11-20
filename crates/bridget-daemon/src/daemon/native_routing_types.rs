//! Types de planification et d'observation du routage natif (extraits de `native_delegation.rs`).

use crate::decision_service::RoutingDecisionFact;
use crate::delegation::AutoAdmissionDecisionLevel;
use crate::registry::{
    AdmissionPolicy, CandidateProfile, DecisionService, RoutingCandidate, RoutingPolicy,
};
use bridget_transport::protocol::{
    LineageDecisionContinuityV4, LineageDecisionQuotaV4, LineageDecisionResourcesV4,
    LineageDecisionScoreTermsV4,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Plan immuable dérivé d'une lecture fraîche du registre.
#[derive(Debug, Clone)]
pub(super) struct AutoAdmissionPlan {
    pub(super) policy: AdmissionPolicy,
    pub(super) policy_digest: String,
    pub(super) candidates: Vec<CandidateProfile>,
    pub(super) decision_service: Option<DecisionService>,
}

/// Le reçu conserve le seul plan qu'une reprise est autorisée à consulter :
/// le registre courant ne peut ni déplacer un choix, ni rouvrir un appel HTTP.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AutoAdmissionSnapshot {
    pub(super) policy: AdmissionPolicy,
    pub(super) candidates: Vec<CandidateProfile>,
    pub(super) decision_service: Option<DecisionService>,
}

/// Faits privés du probe, jamais exposés dans le catalogue public.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingCandidateObservation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) model_option_account: Option<String>,
    pub(super) definition_digest: String,
    pub(super) permission_digest: String,
    pub(super) mapping_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) project_id: Option<String>,
    pub(super) connection_epoch: String,
    pub(super) observed_at: i64,
    pub(super) received_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingMachineObservation {
    pub(super) epoch: String,
    pub(super) resources: bridget_transport::protocol::HostResourceSnapshotV1,
}

/// Faits partagés, calculés une seule fois par machine/connexion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingMachineScoreFact {
    pub(super) scored_at: i64,
    pub(super) resources: LineageDecisionResourcesV4,
    pub(super) compilation_capacity: Option<u8>,
    pub(super) continuity: LineageDecisionContinuityV4,
    pub(super) terms: LineageDecisionScoreTermsV4,
    pub(super) saturated: bool,
    pub(super) known_available: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingConnectionScoreFact {
    pub(super) quota: LineageDecisionQuotaV4,
    pub(super) remaining: i16,
    pub(super) reset: i16,
}

/// Snapshot168 autonome : matrice, faits, ordre et repli sont durcis avant
/// toute consultation. La source détaillée est écrite atomiquement au Chosen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingAdmissionSnapshot {
    pub(super) policy: RoutingPolicy,
    pub(super) candidates: Vec<RoutingCandidate>,
    pub(super) decision_level: AutoAdmissionDecisionLevel,
    pub(super) decision: Option<bridget_transport::protocol::LineageDecisionV4>,
    pub(super) coordinator_machine_id: String,
    pub(super) decision_service: Option<DecisionService>,
    pub(super) observations: Vec<RoutingCandidateObservation>,
    pub(super) machines: BTreeMap<String, RoutingMachineObservation>,
    pub(super) connections: Vec<RoutingDecisionFact>,
    #[serde(default)]
    pub(super) machine_score_facts: BTreeMap<String, RoutingMachineScoreFact>,
    #[serde(default)]
    pub(super) connection_score_facts: BTreeMap<String, RoutingConnectionScoreFact>,
    pub(super) scores: Vec<i32>,
    pub(super) fallback_id: String,
    pub(super) lightweight_definition_digest: Option<String>,
    pub(super) decision_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct RoutingAdmissionPlan {
    pub(super) policy: RoutingPolicy,
    pub(super) candidates: Vec<RoutingCandidate>,
    pub(super) decision_level: AutoAdmissionDecisionLevel,
    pub(super) decision_service: Option<DecisionService>,
    pub(super) policy_digest: String,
    pub(super) coordinator_machine_id: String,
    pub(super) observations: Vec<RoutingCandidateObservation>,
    pub(super) machines: BTreeMap<String, RoutingMachineObservation>,
    pub(super) connections: Vec<RoutingDecisionFact>,
    pub(super) machine_score_facts: BTreeMap<String, RoutingMachineScoreFact>,
    pub(super) connection_score_facts: BTreeMap<String, RoutingConnectionScoreFact>,
    pub(super) scores: Vec<i32>,
    pub(super) fallback_id: String,
    pub(super) lightweight_definition_digest: Option<String>,
    pub(super) decision_notes: Vec<String>,
}
