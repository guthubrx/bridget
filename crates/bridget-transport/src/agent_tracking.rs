//! Organisation humaine des agents, distincte de leur présence et de leurs missions.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentTrackingView {
    pub dormant: bool,
    pub synchronize: bool,
    pub revision: u64,
    pub changed_at: i64,
    pub changed_by: String,
    pub synchronization_pending: bool,
    #[serde(default)]
    pub synchronization_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentTrackingRequest {
    Set {
        request_id: String,
        agent_id: String,
        dormant: bool,
        #[serde(default)]
        synchronize: Option<bool>,
        #[serde(default)]
        expected_revision: Option<u64>,
    },
    Sync {
        source_host: String,
        observations: Vec<AgentTrackingObservation>,
        confirmations: Vec<AgentTrackingConfirmation>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentTrackingConfirmation {
    pub thread_id: String,
    pub operation_id: String,
    pub attempt: u32,
    pub outcome: AgentTrackingSyncOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentTrackingSyncOutcome {
    Applied { sequence: u64 },
    Rejected {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentTrackingObservation {
    pub thread_id: String,
    pub source_revision: String,
    pub source_sequence: u64,
    /// Seules les valeurs settled/active sont des gestes explicites du client.
    /// None couvre notamment un nouveau message, qui ne réveille pas Bridget.
    pub explicit_state: Option<String>,
    pub can_settle: bool,
}
