//! Consultation humaine des tâches natives, sans identité d'agent fournie.
use super::ClientCapability;
use serde::{Deserialize, Serialize};

pub const HUMAN_LINEAGE_VERSION: u16 = 1;
pub const HUMAN_LINEAGE_PLACEMENT_VERSION: u16 = 2;
pub const HUMAN_LINEAGE_DECISION_VERSION: u16 = 3;
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
                version @ (HUMAN_LINEAGE_PLACEMENT_VERSION | HUMAN_LINEAGE_DECISION_VERSION),
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
}
