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

fn serialize_routing_candidate_id<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
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

fn serialize_auto_candidate_id<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
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
}

impl HumanLineageAction {
    pub fn capability(&self) -> ClientCapability {
        match self {
            Self::Watch => ClientCapability::HumanLineageWatchV1,
            Self::Cancel { .. } => ClientCapability::HumanLineageCancelV1,
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
}

impl HumanLineageError {
    pub fn result(self) -> serde_json::Value {
        serde_json::json!({"version":1,"status":"error","code":self,
            "retryable":self == Self::StoreUnavailable})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote162_lineage_version_de_projection_seulement_pour_list_show() {
        let mut request = HumanLineageRequest {
            version: 2,
            t3_thread_id: "thread".into(),
            project_root: "/project".into(),
            request: HumanLineageAction::List {
                limit: 20,
                cursor: None,
            },
        };
        assert_eq!(request.validate_response_version(), Ok(2));
        assert_eq!(
            request.request.capability(),
            ClientCapability::HumanLineageViewV1
        );
        request.request = HumanLineageAction::Show {
            task_id: "task".into(),
            offset: 0,
            limit: 20,
        };
        assert_eq!(request.validate_response_version(), Ok(2));
        request.request = HumanLineageAction::Watch;
        assert_eq!(
            request.validate_response_version(),
            Err(HumanLineageError::UnsupportedVersion)
        );
        request.version = 1;
        assert_eq!(request.validate_response_version(), Ok(1));
        request.request = HumanLineageAction::List {
            limit: 20,
            cursor: None,
        };
        request.version = HUMAN_LINEAGE_DECISION_VERSION;
        assert_eq!(
            request.validate_response_version(),
            Ok(HUMAN_LINEAGE_DECISION_VERSION)
        );
        request.request = HumanLineageAction::Watch;
        assert_eq!(
            request.validate_response_version(),
            Err(HumanLineageError::UnsupportedVersion)
        );
        assert_eq!(HumanLineageError::UnsupportedVersion.result()["version"], 1);
    }

    #[test]
    fn remote162_lineage_wire_v1_inchange_et_placement_v2_ferme() {
        let wire = r#"{"version":1,"t3_thread_id":"t","project_root":"/p","request":{"action":"list","limit":20,"cursor":null}}"#;
        let request: HumanLineageRequest = serde_json::from_str(wire).unwrap();
        assert_eq!(serde_json::to_string(&request).unwrap(), wire);
        let placement = serde_json::json!({"machine_id":"cartae","display_name":null,
            "worker_id":"worker","last_contact_at":null,"contact_state":"unknown"});
        let decoded: LineagePlacementV2 = serde_json::from_value(placement.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), placement);
        let mut unknown = placement;
        unknown["credential"] = serde_json::json!("forbidden");
        assert!(serde_json::from_value::<LineagePlacementV2>(unknown).is_err());
    }

    #[test]
    fn spec163_lineage_v3_decision_est_ferme_et_v1_v2_restent_distincts() {
        let summary = serde_json::json!({
            "mode":"automatic", "candidate_id":"c01", "reason":"profil adapté"
        });
        let decoded: LineageDecisionV3 = serde_json::from_value(summary.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), summary);
        let mut unknown = summary;
        unknown["endpoint"] = serde_json::json!("forbidden");
        assert!(serde_json::from_value::<LineageDecisionV3>(unknown).is_err());
        assert!(
            serde_json::from_value::<LineageDecisionV3>(serde_json::json!({
                "mode":"automatic", "candidate_id":"profile-secret"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<LineageDecisionV3>(serde_json::json!({
                "mode":"automatic", "candidate_id":"c08", "reason":"x".repeat(257)
            }))
            .is_err()
        );
        assert_eq!(
            serde_json::to_value(LineageDecisionV3 {
                mode: LineageDecisionModeV3::Fallback,
                candidate_id: "c08".into(),
                reason: None,
            })
            .unwrap(),
            serde_json::json!({"mode":"fallback","candidate_id":"c08"})
        );
    }

    #[test]
    fn spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3() {
        let value = serde_json::json!({
            "candidate_id":"c128", "connection_id":"conn", "machine_id":"cartae",
            "engine":"codex", "model":"gpt-6.1-sol", "effort":null,
            "source":"deterministic"
        });
        let decoded: LineageDecisionV4 = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), value);
        assert!(
            decoded.facts.is_none(),
            "ancien reçu v4 conservé sans faits inventés"
        );
        let facts = serde_json::json!({
            "scored_at":1000,
            "resources":{"observed_at":990,"age_secs":10,"fresh":true,
                "cpu_percent":40,"available_memory_percent":50,
                "normalized_load_percent":20,"compilation_slots":2},
            "compilation_capacity":8,"continuity":"pinned_base",
            "quota":{"observed_at":995,"age_secs":5,"used_percent":20,"resets_at":1200},
            "score_terms":{"cpu":12,"memory":10,"load":16,"continuity":10,
                "compilation":8,"quota_remaining":4,"quota_reset":3,
                "saturation_penalty":0,"total":63}
        });
        let mut with_facts = value.clone();
        with_facts["facts"] = facts;
        let decoded: LineageDecisionV4 = serde_json::from_value(with_facts.clone()).unwrap();
        assert!(
            decoded.reason.is_none(),
            "les faits locaux ne dépendent pas d'une reason"
        );
        assert!(
            serde_json::to_vec(decoded.facts.as_ref().unwrap())
                .unwrap()
                .len()
                <= MAX_LINEAGE_DECISION_FACTS_BYTES
        );
        assert_eq!(serde_json::to_value(&decoded).unwrap(), with_facts);
        for (field, bad) in [
            (
                "resources",
                serde_json::json!({"observed_at":990,"age_secs":10,
                "fresh":true,"cpu_percent":101,"available_memory_percent":50,
                "normalized_load_percent":20,"compilation_slots":2}),
            ),
            (
                "quota",
                serde_json::json!({"observed_at":995,"age_secs":6,
                "used_percent":20,"resets_at":1200}),
            ),
            ("compilation_capacity", serde_json::json!(11)),
        ] {
            let mut invalid = with_facts.clone();
            invalid["facts"][field] = bad;
            assert!(
                serde_json::from_value::<LineageDecisionV4>(invalid).is_err(),
                "{field}"
            );
        }
        let mut unknown_key = with_facts.clone();
        unknown_key["facts"]["resources"]["credential"] = serde_json::json!("forbidden");
        assert!(serde_json::from_value::<LineageDecisionV4>(unknown_key).is_err());
        let mut invalid_total = decoded.clone();
        invalid_total.facts.as_mut().unwrap().score_terms.total += 1;
        assert!(
            serde_json::to_value(invalid_total).is_err(),
            "l'écriture des metadata applique la même validation que leur lecture"
        );

        let mut unknown_facts = with_facts.clone();
        unknown_facts["facts"]["resources"] = serde_json::json!({
            "observed_at":null,"age_secs":null,"fresh":false,"cpu_percent":null,
            "available_memory_percent":null,"normalized_load_percent":null,
            "compilation_slots":null
        });
        unknown_facts["facts"]["quota"] = serde_json::json!({
            "observed_at":null,"age_secs":null,"used_percent":null,"resets_at":null
        });
        unknown_facts["facts"]["score_terms"] = serde_json::json!({
            "cpu":0,"memory":0,"load":0,"continuity":10,"compilation":8,
            "quota_remaining":0,"quota_reset":0,"saturation_penalty":0,"total":18
        });
        let unknown_decoded: LineageDecisionV4 =
            serde_json::from_value(unknown_facts.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(unknown_decoded).unwrap(),
            unknown_facts,
            "inconnu reste null, jamais une mesure libre à zéro"
        );

        for credits in ["usable", "unusable", "unknown"] {
            let mut exhausted = with_facts.clone();
            exhausted["facts"]["quota"]["used_percent"] = serde_json::json!(100);
            exhausted["facts"]["quota"]["credits"] = serde_json::json!(credits);
            exhausted["facts"]["score_terms"]["quota_remaining"] = serde_json::json!(0);
            exhausted["facts"]["score_terms"]["total"] = serde_json::json!(59);
            let decoded: LineageDecisionV4 = serde_json::from_value(exhausted.clone()).unwrap();
            assert_eq!(
                serde_json::to_value(decoded).unwrap(),
                exhausted,
                "quota inclus à100 et état de crédits sont deux faits distincts : {credits}"
            );
        }
        let mut invalid_credits = with_facts.clone();
        invalid_credits["facts"]["quota"]["credits"] = serde_json::json!({"balance":"opaque"});
        assert!(serde_json::from_value::<LineageDecisionV4>(invalid_credits).is_err());

        let mut bad_slots = with_facts.clone();
        bad_slots["facts"]["resources"]["compilation_slots"] = serde_json::json!(11);
        assert!(
            serde_json::from_value::<LineageDecisionV4>(bad_slots).is_err(),
            "compilation_slots=11 est refusé à la lecture"
        );
        let mut bad_slots = decoded.clone();
        bad_slots
            .facts
            .as_mut()
            .unwrap()
            .resources
            .compilation_slots = Some(11);
        assert!(
            serde_json::to_value(bad_slots).is_err(),
            "compilation_slots=11 est aussi refusé à l'écriture metadata"
        );

        let mut stale = unknown_facts.clone();
        stale["facts"]["resources"]["observed_at"] = serde_json::json!(1);
        stale["facts"]["resources"]["age_secs"] = serde_json::json!(999);
        stale["facts"]["resources"]["compilation_slots"] = serde_json::json!(2);
        stale["facts"]["quota"]["credits"] = serde_json::json!("unknown");
        let decoded: LineageDecisionV4 = serde_json::from_value(stale.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            stale,
            "date/âge périmés conservés, nombres inconnus null et capacité déclarée intacte"
        );
        assert_eq!(stale["facts"]["compilation_capacity"], 8);
        assert!(stale["facts"]["resources"]["cpu_percent"].is_null());

        let mut unknown = value;
        unknown["credential"] = serde_json::json!("forbidden");
        assert!(serde_json::from_value::<LineageDecisionV4>(unknown).is_err());
        assert!(
            serde_json::from_value::<LineageDecisionV4>(serde_json::json!({
                "candidate_id":"c08", "connection_id":"conn", "machine_id":"cartae",
                "engine":"codex", "model":"gpt", "effort":null, "source":"explicit"
            }))
            .is_err()
        );
        let request = HumanLineageRequest {
            version: HUMAN_LINEAGE_ROUTING_VERSION,
            t3_thread_id: "t".into(),
            project_root: "/p".into(),
            request: HumanLineageAction::List {
                limit: 1,
                cursor: None,
            },
        };
        assert_eq!(
            request.validate_response_version(),
            Ok(HUMAN_LINEAGE_ROUTING_VERSION)
        );
    }
}
