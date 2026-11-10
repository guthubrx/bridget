//! Consultation humaine des tâches natives, sans identité d'agent fournie.
use super::ClientCapability;
use serde::{Deserialize, Serialize};

pub const HUMAN_LINEAGE_VERSION: u16 = 1;
pub const HUMAN_LINEAGE_PLACEMENT_VERSION: u16 = 2;
pub const HUMAN_LINEAGE_DECISION_VERSION: u16 = 3;
pub const HUMAN_LINEAGE_ROUTING_VERSION: u16 = 4;
pub const HUMAN_LINEAGE_MAX_SEQ: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanLineageRequest {
    pub version: u16,
    pub t3_thread_id: String,
    pub project_root: String,
    pub request: HumanLineageAction,
}

impl HumanLineageRequest {
    /// Seules les lectures paginées négocient la projection enrichie. Les
    /// capacités, erreurs, journaux et événements watch gardent leur contrat v1.
    pub fn validate_response_version(&self) -> Result<u16, HumanLineageError> {
        match (self.version, &self.request) {
            (HUMAN_LINEAGE_VERSION, _) => Ok(HUMAN_LINEAGE_VERSION),
            (
                version @ (HUMAN_LINEAGE_PLACEMENT_VERSION
                | HUMAN_LINEAGE_DECISION_VERSION
                | HUMAN_LINEAGE_ROUTING_VERSION),
                HumanLineageAction::List { .. } | HumanLineageAction::Show { .. },
            ) => Ok(version),
            _ => Err(HumanLineageError::UnsupportedVersion),
        }
    }
}

/// Fait de contact du worker, distinct du statut métier de la tâche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerContactState {
    Connected,
    Stale,
    Unknown,
}

/// Uniquement dans les succès list/show v2 ; jamais ajouté à la sortie v1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineagePlacementV2 {
    pub machine_id: String,
    pub display_name: Option<String>,
    pub worker_id: String,
    /// Observation du maître à la lecture de cette page, pas un snapshot gelé.
    pub last_contact_at: Option<i64>,
    pub contact_state: WorkerContactState,
}

/// Mode conservé avec l'admission, sans autorité supplémentaire pour le lecteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageDecisionModeV3 {
    Explicit,
    Automatic,
    Fallback,
}

/// Présent seulement dans les réponses list/show v3. `None` signifie que la
/// tâche historique ne possède pas de reçu de décision, pas un mode inventé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDecisionV3 {
    pub mode: LineageDecisionModeV3,
    #[serde(
        deserialize_with = "deserialize_auto_candidate_id",
        serialize_with = "serialize_auto_candidate_id"
    )]
    pub candidate_id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_auto_admission_reason",
        serialize_with = "serialize_auto_admission_reason"
    )]
    pub reason: Option<String>,
}

/// Projection168 v4. Elle reste absente d'une lecture v3 afin de préserver la
/// compatibilité des lecteurs historiques et de leurs IDs c01..c08.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDecisionV4 {
    #[serde(
        deserialize_with = "deserialize_routing_candidate_id",
        serialize_with = "serialize_routing_candidate_id"
    )]
    pub candidate_id: String,
    pub connection_id: String,
    pub machine_id: String,
    pub engine: String,
    pub model: String,
    pub effort: Option<String>,
    pub source: LineageDecisionSourceV4,
    /// Faits du classement d'origine, jamais relus sur l'hôte à la consultation.
    /// L'absence conserve la lecture des reçus v4 antérieurs à cette projection.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_decision_facts",
        serialize_with = "serialize_decision_facts"
    )]
    pub facts: Option<LineageDecisionFactsV4>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_auto_admission_reason",
        serialize_with = "serialize_auto_admission_reason"
    )]
    pub reason: Option<String>,
}

/// Une seule cible : aucune matrice privée, chaîne libre, chemin ou credential.
pub const MAX_LINEAGE_DECISION_FACTS_BYTES: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDecisionFactsV4 {
    pub scored_at: i64,
    pub resources: LineageDecisionResourcesV4,
    pub compilation_capacity: Option<u8>,
    pub continuity: LineageDecisionContinuityV4,
    /// Faits de la seule connexion portée par LineageDecisionV4.connection_id.
    pub quota: LineageDecisionQuotaV4,
    pub score_terms: LineageDecisionScoreTermsV4,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDecisionResourcesV4 {
    pub observed_at: Option<i64>,
    pub age_secs: Option<u64>,
    pub fresh: bool,
    pub cpu_percent: Option<u8>,
    pub available_memory_percent: Option<u8>,
    pub normalized_load_percent: Option<u8>,
    pub compilation_slots: Option<u16>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageDecisionContinuityV4 {
    Local,
    PinnedBase,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDecisionQuotaV4 {
    pub observed_at: Option<i64>,
    pub age_secs: Option<u64>,
    pub used_percent: Option<u8>,
    pub resets_at: Option<i64>,
    /// État attesté utile de cette connexion ; aucun solde ni identité de compte.
    /// None conserve les snapshots antérieurs qui n'enregistraient pas ce fait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credits: Option<LineageDecisionCreditsV4>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageDecisionCreditsV4 {
    Usable,
    Unusable,
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageDecisionScoreTermsV4 {
    pub cpu: i16,
    pub memory: i16,
    pub load: i16,
    pub continuity: i16,
    pub compilation: i16,
    pub quota_remaining: i16,
    pub quota_reset: i16,
    /// Valeur signée : 0 ou -100, ajoutée aux autres termes.
    pub saturation_penalty: i16,
    pub total: i16,
}

impl LineageDecisionFactsV4 {
    /// Même validation lors de l'écriture des métadonnées et de leur lecture.
    pub fn validate(&self) -> Result<(), &'static str> {
        let time_valid = |at: i64| (0..=HUMAN_LINEAGE_MAX_SEQ as i64).contains(&at);
        let age_valid = |at: Option<i64>, age: Option<u64>| match (at, age) {
            (None, None) => true,
            (Some(at), Some(age)) => {
                time_valid(at) && age == self.scored_at.saturating_sub(at).max(0) as u64
            }
            _ => false,
        };
        let r = &self.resources;
        let q = &self.quota;
        let t = &self.score_terms;
        let percentages = [
            r.cpu_percent,
            r.available_memory_percent,
            r.normalized_load_percent,
            q.used_percent,
        ];
        let total = i32::from(t.cpu)
            + i32::from(t.memory)
            + i32::from(t.load)
            + i32::from(t.continuity)
            + i32::from(t.compilation)
            + i32::from(t.quota_remaining)
            + i32::from(t.quota_reset)
            + i32::from(t.saturation_penalty);
        if !time_valid(self.scored_at)
            || !age_valid(r.observed_at, r.age_secs)
            || !age_valid(q.observed_at, q.age_secs)
            || (r.fresh && r.observed_at.is_none())
            || percentages.into_iter().flatten().any(|v| v > 100)
            || r.compilation_slots.is_some_and(|v| v > 10)
            || self.compilation_capacity.is_some_and(|v| v > 10)
            || q.resets_at.is_some_and(|at| !time_valid(at))
            || !(0..=20).contains(&t.cpu)
            || !(0..=20).contains(&t.memory)
            || !(0..=20).contains(&t.load)
            || !(0..=20).contains(&t.continuity)
            || !(0..=10).contains(&t.compilation)
            || !(0..=5).contains(&t.quota_remaining)
            || !(0..=3).contains(&t.quota_reset)
            || !matches!(t.saturation_penalty, 0 | -100)
            || total != i32::from(t.total)
        {
            return Err("decision_facts_invalid");
        }
        let bytes = serde_json::to_vec(self).map_err(|_| "decision_facts_invalid")?;
        if bytes.len() > MAX_LINEAGE_DECISION_FACTS_BYTES {
            return Err("decision_facts_too_large");
        }
        Ok(())
    }
}

fn deserialize_decision_facts<'de, D>(
    deserializer: D,
) -> Result<Option<LineageDecisionFactsV4>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let facts = Option::<LineageDecisionFactsV4>::deserialize(deserializer)?;
    if let Some(facts) = &facts {
        facts.validate().map_err(serde::de::Error::custom)?;
    }
    Ok(facts)
}

fn serialize_decision_facts<S>(
    facts: &Option<LineageDecisionFactsV4>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(facts) = facts {
        facts.validate().map_err(serde::ser::Error::custom)?;
    }
    facts.serialize(serializer)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageDecisionSourceV4 {
    Explicit,
    Single,
    DecisionService,
    LightweightModel,
    Deterministic,
}

fn deserialize_routing_candidate_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if super::is_routing_candidate_id(&value) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("routing_candidate_id_invalid"))
    }
}

fn serialize_routing_candidate_id<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if super::is_routing_candidate_id(value) {
        serializer.serialize_str(value)
    } else {
        Err(serde::ser::Error::custom("routing_candidate_id_invalid"))
    }
}

fn deserialize_auto_candidate_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if super::is_auto_candidate_id(&value) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("candidate_id_invalid"))
    }
}

fn serialize_auto_candidate_id<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if super::is_auto_candidate_id(value) {
        serializer.serialize_str(value)
    } else {
        Err(serde::ser::Error::custom("candidate_id_invalid"))
    }
}

fn deserialize_auto_admission_reason<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if value.as_deref().is_none_or(super::is_auto_admission_reason) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("decision_reason_invalid"))
    }
}

fn serialize_auto_admission_reason<S>(
    value: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if value.as_deref().is_none_or(super::is_auto_admission_reason) {
        value.serialize(serializer)
    } else {
        Err(serde::ser::Error::custom("decision_reason_invalid"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum HumanLineageAction {
    List {
        limit: u32,
        cursor: Option<String>,
    },
    Show {
        task_id: String,
        offset: u32,
        limit: u32,
    },
    Journal {
        task_id: String,
        after_seq: u64,
        limit: u32,
        follow: bool,
    },
    Watch,
    Cancel {
        task_id: String,
        request_id: String,
    },
    /// Composition d'un tour enfant. `expected_round` et `request_id` gardent
    /// la corrélation ; l'identité d'agent ne vient jamais du client.
    Compose {
        task_id: String,
        expected_round: u32,
        request_id: String,
        operation: HumanComposeOperation,
    },
}

/// Réponse `compose` réussie. Tous les champs sont présents, `null` compris ;
/// les refus d'autorité restent `HumanLineageError`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanComposeResponseV1 {
    pub version: u16,
    pub status: HumanComposeStatus,
    pub request_id: String,
    pub task_id: String,
    pub round: u32,
    pub outcome: HumanComposeOutcome,
    #[serde(deserialize_with = "present_nullable")]
    pub code: Option<String>,
    pub message: String,
    pub state: String,
    #[serde(deserialize_with = "present_nullable")]
    pub selection: Option<HumanComposeSelection>,
    pub models: Vec<HumanComposeModel>,
    pub capabilities: HumanComposeCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanComposeStatus {
    Ok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanComposeOutcome {
    Observed,
    Accepted,
    Selected,
    Refused,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanComposeSelection {
    pub model: String,
    #[serde(deserialize_with = "present_nullable")]
    pub effort: Option<String>,
}

/// Un champ nullable doit être présent dans la réponse fermée : serde
/// accepterait autrement son absence comme `null`.
fn present_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanComposeModel {
    pub id: String,
    pub name: String,
    pub efforts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanComposeCapabilities {
    pub send: bool,
    pub steer: bool,
    pub interrupt: bool,
    pub select_runtime: bool,
    pub attachments: bool,
}

/// Taille maximale du texte d'un envoi humain, en octets UTF-8.
pub const HUMAN_COMPOSE_BODY_MAX_BYTES: usize = 65536;

/// Opérations humaines fermées. Les champs d'une autre opération sont refusés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HumanComposeOperation {
    Read {},
    Send { body: String, intent: HumanComposeSendIntent },
    SelectRuntime {
        model: String,
        #[serde(deserialize_with = "present_nullable")]
        effort: Option<String>,
    },
    Interrupt {},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanComposeSendIntent { Queue, Steer }

impl HumanComposeOperation {
    /// Bornes locales ; l'existence du tour et des capacités relève du daemon.
    /// La lecture de ronde utilise `expected_round=0` sans effet ; toute
    /// mutation exige une ronde constatée strictement positive.
    pub fn bounded(&self, expected_round: u32) -> bool {
        match self {
            Self::Read {} => expected_round == 0,
            _ if expected_round == 0 => false,
            Self::Interrupt {} => true,
            Self::Send { body, .. } => {
                !body.trim().is_empty()
                    && body.len() <= HUMAN_COMPOSE_BODY_MAX_BYTES
                    && !body.chars().any(|c| c != '\n' && c != '\t' && c.is_control())
            }
            // Même règle que `RuntimeSelection::valid` du plan de contrôle.
            Self::SelectRuntime { model, effort } => std::iter::once(model)
                .chain(effort.iter()).all(|s| {
                    !s.is_empty() && s.len() <= 128
                        && s.chars().all(|c| !c.is_control() && !c.is_whitespace())
                }),
        }
    }
}

impl HumanLineageAction {
    pub fn capability(&self) -> ClientCapability {
        match self {
            Self::Watch => ClientCapability::HumanLineageWatchV1,
            Self::Cancel { .. } => ClientCapability::HumanLineageCancelV1,
            Self::Compose { .. } => ClientCapability::HumanLineageComposeV1,
            _ => ClientCapability::HumanLineageViewV1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum HumanLineageWatchEvent {
    Ready {
        version: u16,
        generation: String,
        seq: u64,
    },
    Changed {
        version: u16,
        generation: String,
        seq: u64,
    },
    Resync {
        version: u16,
        generation: String,
        seq: u64,
    },
    Error {
        version: u16,
        code: HumanLineageError,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanLineageError {
    UnsupportedVersion,
    InvalidRequest,
    BindingUnavailable,
    ProjectMismatch,
    TaskUnavailable,
    SnapshotChanged,
    JournalUnavailable,
    ResultOffsetInvalid,
    EnvelopeMismatch,
    ResourceLimit,
    StoreUnavailable,
    /// Composition reconnue mais pas encore branchée au daemon : refus explicite,
    /// jamais un succès ni un envoi partiel.
    ComposeUnavailable,
}

impl HumanLineageError {
    pub fn result(self) -> serde_json::Value {
        serde_json::json!({"version":1,"status":"error","code":self,
            "retryable":self == Self::StoreUnavailable})
    }
}

#[cfg(test)]
#[path = "lineage_protocol_tests.rs"]
mod tests;
