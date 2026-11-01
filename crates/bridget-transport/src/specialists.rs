//! Contrat fermé222 des spécialistes ; aucune donnée de compte ni autorité déclarée.
use crate::protocol::SpawnPosture;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistScope {
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub transversal: bool,
}
impl SpecialistScope {
    pub fn permits(&self, project: &str) -> bool {
        self.transversal || self.projects.iter().any(|p| p == project)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistBudget {
    pub max_duration_secs: Option<u64>,
    pub max_facturable_tokens: Option<u64>,
    pub max_descendants: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistProcedure {
    pub name: String,
    pub instructions: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpecialistRequirement {
    Tool { name: String },
    Capability { name: String },
    Program { name: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistPreferences {
    pub machine_id: Option<String>,
    pub engine: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistTeamMember {
    pub key: String,
    pub role_id: String,
    pub deliverable: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistDefinition {
    pub profile_id: String,
    pub name: String,
    pub kind: SpecialistDefinitionKind,
    pub instructions: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub procedures: Vec<SpecialistProcedure>,
    #[serde(default)]
    pub requirements: Vec<SpecialistRequirement>,
    #[serde(default)]
    pub preferences: SpecialistPreferences,
    #[serde(default)]
    pub members: Vec<SpecialistTeamMember>,
    pub provenance: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistDefinitionKind {
    Profile,
    Team,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistLaunchPolicy {
    #[serde(default)]
    pub prepare_requirements: bool,
    pub agent_type: String,
    pub cwd: String,
    pub posture: SpawnPosture,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialistRole {
    pub role_id: String,
    pub label: String,
    pub scope: SpecialistScope,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistMemoryKind {
    Fact,
    Decision,
    Hypothesis,
    ReportedResult,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpecialistSource {
    Message {
        message_id: String,
        in_reply_to: String,
        digest: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        part: Option<u32>,
    },
    Task {
        task_id: String,
        round: u64,
        digest: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        part: Option<u32>,
    },
    Journal {
        agent_id: String,
        file: String,
        from_seq: u64,
        to_seq: u64,
        digest: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        part: Option<u32>,
    },
    Statement {
        request_id: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpecialistRequest {
    RequestTemporary {
        version: u16,
        request_id: String,
        profile_id: String,
        profile_revision: u64,
        delegation: Box<crate::protocol::AutoDelegateV2>,
    },
    RequestApproval {
        version: u16,
        request_id: String,
        operation: Box<SpecialistRequest>,
    },
    CaptureHistory {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        project_id: Option<String>,
        after_seq: Option<u64>,
        limit: Option<u32>,
    },
    Profiles {
        version: u16,
        cursor: Option<String>,
        limit: Option<u32>,
    },
    ProfileVersions {
        version: u16,
        profile_id: String,
        cursor: Option<String>,
        limit: Option<u32>,
    },
    AttachProfile {
        version: u16,
        request_id: String,
        profile_id: String,
        profile_revision: u64,
        expected_revision: u64,
    },
    List {
        version: u16,
        cursor: Option<String>,
        limit: Option<u32>,
    },
    Search {
        version: u16,
        role_id: String,
        project_id: Option<String>,
        cursor: Option<String>,
        limit: Option<u32>,
    },
    Get {
        version: u16,
        specialist_agent_id: String,
    },
    DeclareRoles {
        version: u16,
        request_id: String,
        expected_revision: u64,
        roles: Vec<SpecialistRole>,
    },
    ProposeProfile {
        version: u16,
        request_id: String,
        definition: SpecialistDefinition,
    },
    ProposeRevision {
        version: u16,
        request_id: String,
        expected_revision: u64,
        definition: SpecialistDefinition,
    },
    ApproveRevision {
        version: u16,
        request_id: String,
        profile_id: String,
        revision: u64,
    },
    Activate {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        profile_id: String,
        profile_revision: u64,
        expected_revision: u64,
        scope: SpecialistScope,
        durable: bool,
        auto_start: bool,
        mandate: Option<String>,
        budget: Option<SpecialistBudget>,
        launch_policy: Option<SpecialistLaunchPolicy>,
    },
    Disable {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        expected_revision: u64,
    },
    ConfigureMandate {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        expected_revision: u64,
        mandate: String,
        budget: Option<SpecialistBudget>,
        scope: SpecialistScope,
        auto_start: bool,
        launch_policy: Option<SpecialistLaunchPolicy>,
    },
    Revoke {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        expected_revision: u64,
    },
    DesignateRole {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        role_id: String,
        official: bool,
    },
    MemoryVersions {
        version: u16,
        memory_id: String,
        cursor: Option<u64>,
        limit: Option<u32>,
    },
    MemoryRecall {
        version: u16,
        specialist_agent_id: String,
        project_id: String,
        query: String,
        cursor: Option<String>,
        limit: Option<u32>,
    },
    MemoryWrite {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        project_id: String,
        shared: bool,
        kind: SpecialistMemoryKind,
        text: String,
        source: SpecialistSource,
    },
    MemoryCorrect {
        version: u16,
        request_id: String,
        memory_id: String,
        expected_revision: u64,
        text: String,
        kind: SpecialistMemoryKind,
    },
    MemoryForget {
        version: u16,
        request_id: String,
        memory_id: String,
        expected_revision: u64,
    },
    Export {
        version: u16,
        profile_id: String,
        revision: u64,
    },
    Import {
        version: u16,
        request_id: String,
        definition: SpecialistDefinition,
    },
    Convert {
        version: u16,
        request_id: String,
        specialist_agent_id: String,
        profile_id: String,
        profile_revision: u64,
        expected_revision: u64,
    },
    RequestRole {
        version: u16,
        request_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        specialist_agent_id: Option<String>,
        role_id: String,
        project_id: String,
        task: String,
    },
    RequestStatus {
        version: u16,
        request_id: String,
    },
    RequestTeam {
        version: u16,
        request_id: String,
        profile_id: String,
        revision: u64,
        project_id: String,
        task: String,
    },
}
impl SpecialistRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        let value = serde_json::to_value(self).map_err(|_| "invalid_request")?;
        if value["version"] != 1 {
            return Err("specialists_version_unsupported");
        }
        if serde_json::to_vec(&value)
            .map_err(|_| "invalid_request")?
            .len()
            > 64 * 1024
        {
            return Err("invalid_request");
        }
        if let Some(id) = value.get("request_id").and_then(|v| v.as_str()) {
            if id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) {
                return Err("invalid_request");
            }
        }
        Ok(())
    }
    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Self::Profiles { .. }
                | Self::ProfileVersions { .. }
                | Self::List { .. }
                | Self::Search { .. }
                | Self::Get { .. }
                | Self::MemoryVersions { .. }
                | Self::MemoryRecall { .. }
                | Self::Export { .. }
                | Self::RequestStatus { .. }
        )
    }
    pub fn requires_human(&self) -> bool {
        matches!(
            self,
            Self::ApproveRevision { .. }
                | Self::Activate { .. }
                | Self::Disable { .. }
                | Self::ConfigureMandate { .. }
                | Self::Revoke { .. }
                | Self::DesignateRole { .. }
                | Self::Import { .. }
                | Self::Convert { .. }
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec222_request_rejects_unknown_authority_and_version() {
        let q: SpecialistRequest =
            serde_json::from_str(r#"{"action":"list","version":1}"#).unwrap();
        assert!(q.validate().is_ok());
        assert!(
            serde_json::from_str::<SpecialistRequest>(
                r#"{"action":"list","version":1,"is_human":true}"#
            )
            .is_err()
        );
        let q: SpecialistRequest =
            serde_json::from_str(r#"{"action":"list","version":2}"#).unwrap();
        assert_eq!(q.validate(), Err("specialists_version_unsupported"));
        assert_eq!(SpecialistBudget::default().max_facturable_tokens, None);
    }
}
