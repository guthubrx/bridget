//! Consultation humaine des tâches natives, sans identité d'agent fournie.
use super::ClientCapability;
use serde::{Deserialize, Serialize};

pub const HUMAN_LINEAGE_VERSION: u16 = 1;
pub const HUMAN_LINEAGE_MAX_SEQ: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanLineageRequest {
    pub version: u16,
    pub t3_thread_id: String,
    pub project_root: String,
    pub request: HumanLineageAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum HumanLineageAction {
    List { limit: u32, cursor: Option<String> },
    Show { task_id: String, offset: u32, limit: u32 },
    Journal { task_id: String, after_seq: u64, limit: u32, follow: bool },
    Watch,
    Cancel { task_id: String, request_id: String },
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
    Ready { version: u16, generation: String, seq: u64 },
    Changed { version: u16, generation: String, seq: u64 },
    Resync { version: u16, generation: String, seq: u64 },
    Error { version: u16, code: HumanLineageError },
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
