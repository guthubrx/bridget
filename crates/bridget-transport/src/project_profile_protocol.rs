use super::{ProjectBackend, ProjectReference, ResolvedAgentDefinition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectResourceKind {
    Extension,
    SecretFile,
    SecretDirectory,
    SecretProcessEnv,
}

pub const PROJECT_PROFILE_CONTRACT_VERSION: u16 = 1;

/// Host resolution request before a local le service compagnon approval.
/// The proposal carries declarative references only, never a secret value or
/// a source revision claimed by the client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfileRequest {
    pub proposal: ProjectProfileProposal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectProfileRefusal {
    InvalidContract,
    InvalidProfile,
    ProjectNotFound,
    ProjectNotDocker,
    BindingGenerationMismatch,
    RuntimePolicyMismatch,
    CatalogUnavailable,
    ResourceRejected,
    PeerUidMismatch,
}

/// Resolution result without a host path or secret content. No profile plus a
/// reason is a closed refusal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfileOutcome {
    pub contract_version: u16,
    pub command_id: String,
    pub project: ProjectReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<ProjectBackend>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<ResolvedProjectProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectProfileRefusal>,
    pub observed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectResourceRef {
    pub resource_id: String,
    pub kind: ProjectResourceKind,
    pub source_ref: String,
    pub destination: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedProjectResource {
    pub reference: ProjectResourceRef,
    pub source_revision: u64,
    pub attestation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfileAgent {
    pub profile_id: String,
    pub agent_type: String,
    pub model: String,
    pub effort: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedProjectAgent {
    pub agent: ProjectProfileAgent,
    pub definition: ResolvedAgentDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRuntimeView {
    pub policy_id: String,
    pub policy_version: u64,
    pub policy_digest: String,
    pub image_reference: String,
    pub run_as_uid: u32,
    pub run_as_gid: u32,
    pub cpu_limit_milli: u64,
    pub memory_limit_bytes: u64,
    pub pids_limit: u32,
    pub tmpfs: Vec<String>,
    pub network_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfileProposal {
    pub contract_version: u16,
    pub command_id: String,
    pub project: ProjectReference,
    pub profile_id: String,
    pub binding_generation: u64,
    pub runtime_policy_version: u64,
    pub policy_digest: String,
    pub generation: u64,
    pub agent_profile_ids: Vec<String>,
    pub agents: Vec<ProjectProfileAgent>,
    pub extensions: Vec<ProjectResourceRef>,
    pub secrets: Vec<ProjectResourceRef>,
    pub required_capabilities: Vec<String>,
    pub profile_digest: String,
}
impl ProjectProfileProposal {
    pub fn validate(&self) -> Result<(), &str> {
        if self.contract_version != 1
            || self.command_id.trim().is_empty()
            || self.profile_id.trim().is_empty()
            || self.binding_generation == 0
            || self.binding_generation != self.project.binding_generation
            || self.runtime_policy_version == 0
            || self.policy_digest.trim().is_empty()
            || self.generation == 0
            || self.agent_profile_ids.is_empty()
            || self.agents.is_empty()
            || self.agents.len() != self.agent_profile_ids.len()
        {
            return Err("proposition profil projet invalide");
        }
        let mut profile_ids = HashSet::new();
        for agent in &self.agents {
            if agent.profile_id.trim().is_empty()
                || agent.agent_type.trim().is_empty()
                || agent.model.trim().is_empty()
                || agent.effort.trim().is_empty()
                || !profile_ids.insert(agent.profile_id.as_str())
                || !self
                    .agent_profile_ids
                    .iter()
                    .any(|id| id == &agent.profile_id)
            {
                return Err("agent profil invalide");
            }
        }
        let mut destinations = HashSet::new();
        let mut resource_ids = HashSet::new();
        let extension_count = self.extensions.len();
        for (position, resource) in self.extensions.iter().chain(&self.secrets).enumerate() {
            if (position < extension_count) != (resource.kind == ProjectResourceKind::Extension) {
                return Err("groupe ressource invalide");
            }
            if resource.resource_id.trim().is_empty()
                || resource.source_ref.trim().is_empty()
                || resource.destination.trim().is_empty()
                || resource.generation == 0
            {
                return Err("reference ressource invalide");
            }
            if matches!(resource.kind, ProjectResourceKind::Extension)
                && (resource.version.as_deref().unwrap_or_default().is_empty()
                    || resource
                        .content_digest
                        .as_deref()
                        .unwrap_or_default()
                        .is_empty())
            {
                return Err("extension sans version ou digest");
            }
            if !resource_ids.insert(resource.resource_id.as_str()) {
                return Err("identifiant ressource duplique");
            }
            if !destinations.insert(resource.destination.as_str()) {
                return Err("destination ressource dupliquee");
            }
        }
        if self
            .required_capabilities
            .iter()
            .any(|capability| capability.trim().is_empty())
        {
            return Err("capability requise invalide");
        }
        let mut capabilities = HashSet::new();
        if self
            .required_capabilities
            .iter()
            .any(|capability| !capabilities.insert(capability.as_str()))
        {
            return Err("capability requise dupliquee");
        }
        if self.profile_digest.trim().is_empty() {
            return Err("digest profil absent");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedProjectProfile {
    pub proposal: ProjectProfileProposal,
    pub resources: Vec<ResolvedProjectResource>,
    pub agents: Vec<ResolvedProjectAgent>,
    pub runtime: ProjectRuntimeView,
    pub resolved_digest: String,
}
impl ResolvedProjectProfile {
    pub fn from_resolution(
        proposal: ProjectProfileProposal,
        resources: Vec<ResolvedProjectResource>,
        agents: Vec<ResolvedProjectAgent>,
        runtime: ProjectRuntimeView,
    ) -> Result<Self, String> {
        proposal.validate().map_err(str::to_string)?;
        let canonical = serde_json::to_vec(&(&proposal, &resources, &agents, &runtime))
            .map_err(|error| error.to_string())?;
        let profile = Self {
            proposal,
            resources,
            agents,
            runtime,
            resolved_digest: format!("sha256:{:x}", Sha256::digest(canonical)),
        };
        profile.validate().map_err(str::to_string)?;
        Ok(profile)
    }

    pub fn validate(&self) -> Result<(), &str> {
        self.proposal.validate()?;
        if self.runtime.policy_id.trim().is_empty()
            || self.runtime.policy_version != self.proposal.runtime_policy_version
            || self.runtime.policy_digest != self.proposal.policy_digest
            || self.runtime.image_reference.trim().is_empty()
            || self.runtime.run_as_uid == 0
            || self.runtime.run_as_gid == 0
            || self.runtime.cpu_limit_milli == 0
            || self.runtime.memory_limit_bytes == 0
            || self.runtime.pids_limit == 0
            || self.runtime.network_mode != "bridge"
        {
            return Err("vue runtime invalide");
        }
        if self.resolved_digest.trim().is_empty() {
            return Err("digest resolution absent");
        }
        let canonical =
            serde_json::to_vec(&(&self.proposal, &self.resources, &self.agents, &self.runtime))
                .map_err(|_| "resolution invalide")?;
        if self.resolved_digest != format!("sha256:{:x}", Sha256::digest(canonical)) {
            return Err("digest resolution divergent");
        }
        if self.agents.len() != self.proposal.agents.len() {
            return Err("nombre de definitions agents divergent");
        }
        for resolved in &self.agents {
            if resolved.definition.digest.trim().is_empty()
                || !self.proposal.agents.contains(&resolved.agent)
            {
                return Err("definition agent resolue invalide");
            }
        }
        let expected = self.proposal.extensions.len() + self.proposal.secrets.len();
        if self.resources.len() != expected {
            return Err("nombre de ressources resolues divergent");
        }
        for resource in &self.resources {
            if resource.source_revision == 0
                || resource.attestation_digest.trim().is_empty()
                || !self
                    .proposal
                    .extensions
                    .iter()
                    .chain(&self.proposal.secrets)
                    .any(|reference| reference == &resource.reference)
            {
                return Err("attestation ressource invalide");
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_067_contrat_commun_accepte_les_quatre_fournisseurs_sans_heuristique() {
        for agent_type in ["codex", "claude", "cursor-acp", "fixture"] {
            let proposal = ProjectProfileProposal {
                contract_version: 1,
                command_id: "command-a".to_string(),
                project: ProjectReference {
                    project_id: "project-a".to_string(),
                    binding_generation: 4,
                },
                profile_id: "profile-a".to_string(),
                binding_generation: 4,
                runtime_policy_version: 2,
                policy_digest: "sha256:policy".to_string(),
                generation: 1,
                agent_profile_ids: vec!["coordinator".to_string()],
                agents: vec![ProjectProfileAgent {
                    profile_id: "coordinator".to_string(),
                    agent_type: agent_type.to_string(),
                    model: "declared-model".to_string(),
                    effort: "declared-effort".to_string(),
                    tools: vec!["shell".to_string()],
                }],
                extensions: Vec::new(),
                secrets: Vec::new(),
                required_capabilities: vec!["acp".to_string()],
                profile_digest: "sha256:profile".to_string(),
            };
            assert!(proposal.validate().is_ok(), "{agent_type}");
        }
    }

    #[test]
    fn spec_067_proposition_ne_porte_pas_de_revision_source() {
        let proposal = ProjectProfileProposal {
            contract_version: 1,
            command_id: "command-a".to_string(),
            project: ProjectReference {
                project_id: "project-a".to_string(),
                binding_generation: 4,
            },
            profile_id: "profile-a".to_string(),
            binding_generation: 4,
            runtime_policy_version: 2,
            policy_digest: "sha256:policy".to_string(),
            generation: 1,
            agent_profile_ids: vec!["coordinator".to_string()],
            agents: vec![ProjectProfileAgent {
                profile_id: "coordinator".to_string(),
                agent_type: "codex".to_string(),
                model: "gpt-5".to_string(),
                effort: "high".to_string(),
                tools: vec!["shell".to_string()],
            }],
            extensions: Vec::new(),
            secrets: Vec::new(),
            required_capabilities: vec!["acp".to_string()],
            profile_digest: "sha256:profile".to_string(),
        };
        let wire = serde_json::to_string(&proposal).unwrap();
        assert!(!wire.contains("source_revision"));
        assert_eq!(
            serde_json::from_str::<ProjectProfileProposal>(&wire).unwrap(),
            proposal
        );
        assert!(proposal.validate().is_ok());
    }
}
