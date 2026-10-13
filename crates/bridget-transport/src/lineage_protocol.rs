//! Consultation humaine des tâches natives, sans identité d'agent fournie.
use super::ClientCapability;
use serde::{Deserialize, Serialize};

pub const HUMAN_LINEAGE_VERSION: u16 = 1;
pub const HUMAN_LINEAGE_PLACEMENT_VERSION: u16 = 2;
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
                HUMAN_LINEAGE_PLACEMENT_VERSION,
                HumanLineageAction::List { .. } | HumanLineageAction::Show { .. },
            ) => Ok(HUMAN_LINEAGE_PLACEMENT_VERSION),
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
        request.version = 3;
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
}
