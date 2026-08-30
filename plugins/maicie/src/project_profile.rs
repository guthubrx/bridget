use crate::config::{ProfileConfig, ProjectProfileConfig};
use crate::domain::DomainError;
use bridget_transport::protocol::{
    ProjectProfileAgent, ProjectResourceRef, ResolvedProjectProfile,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectProfileStatus {
    Proposed,
    Resolved,
    Approved,
    Active,
    Stale,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfile {
    pub profile_id: String,
    pub project_id: String,
    pub binding_generation: u64,
    pub runtime_policy_version: u64,
    pub policy_digest: String,
    pub agent_profile_ids: Vec<String>,
    pub agents: Vec<ProjectProfileAgent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<ResolvedProjectProfile>,
    pub extensions: Vec<ProjectResourceRef>,
    pub secrets: Vec<ProjectResourceRef>,
    pub required_capabilities: Vec<String>,
    pub generation: u64,
    pub status: ProjectProfileStatus,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectProfileApproval {
    pub profile_id: String,
    pub project_id: String,
    pub binding_generation: u64,
    pub runtime_policy_version: u64,
    pub policy_digest: String,
    pub generation: u64,
    pub profile_digest: String,
    pub actor: String,
    pub approved_at: i64,
}

impl ProjectProfile {
    #[allow(clippy::too_many_arguments)]
    pub fn propose(
        profile_id: String,
        project_id: String,
        binding_generation: u64,
        runtime_policy_version: u64,
        policy_digest: String,
        agent_profile_ids: Vec<String>,
        agents: Vec<ProjectProfileAgent>,
        extensions: Vec<ProjectResourceRef>,
        secrets: Vec<ProjectResourceRef>,
        required_capabilities: Vec<String>,
        now: i64,
    ) -> Result<Self, DomainError> {
        if profile_id.trim().is_empty()
            || project_id.trim().is_empty()
            || binding_generation == 0
            || runtime_policy_version == 0
            || policy_digest.trim().is_empty()
            || agent_profile_ids.is_empty()
            || agents.is_empty()
            || agents.len() != agent_profile_ids.len()
            || now < 0
        {
            return Err(DomainError::DonneeInvalide("profil projet invalide"));
        }
        Ok(Self {
            profile_id,
            project_id,
            binding_generation,
            runtime_policy_version,
            policy_digest,
            agent_profile_ids,
            agents,
            resolved: None,
            generation: 1,
            extensions,
            secrets,
            required_capabilities,
            status: ProjectProfileStatus::Proposed,
            updated_at: now,
        })
    }
    pub fn from_config(
        config: &ProjectProfileConfig,
        profiles: &[ProfileConfig],
        now: i64,
    ) -> Result<Self, DomainError> {
        let agents = config
            .agent_profile_ids
            .iter()
            .map(|profile_id| {
                let profile = profiles
                    .iter()
                    .find(|profile| profile.id == *profile_id)
                    .ok_or(DomainError::DonneeInvalide("profil agent introuvable"))?;
                let required = |value: &Option<String>| {
                    value
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                        .map(ToOwned::to_owned)
                        .ok_or(DomainError::DonneeInvalide("profil agent incomplet"))
                };
                Ok(ProjectProfileAgent {
                    profile_id: profile.id.clone(),
                    agent_type: required(&profile.agent_type)?,
                    model: required(&profile.model)?,
                    effort: required(&profile.effort)?,
                    tools: profile.tools.clone(),
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;
        Self::propose(
            config.id.clone(),
            config.project_id.clone(),
            config.binding_generation,
            config.runtime_policy_version,
            config.policy_digest.clone(),
            config.agent_profile_ids.clone(),
            agents,
            config.extensions.clone(),
            config.secrets.clone(),
            config.required_capabilities.clone(),
            now,
        )
    }

    pub fn digest(&self) -> Result<String, DomainError> {
        let canonical = serde_json::to_vec(&(
            &self.profile_id,
            &self.project_id,
            self.binding_generation,
            self.runtime_policy_version,
            &self.policy_digest,
            &self.agent_profile_ids,
            &self.agents,
            &self.extensions,
            &self.secrets,
            &self.required_capabilities,
            self.generation,
        ))
        .map_err(|_| DomainError::DonneeInvalide("profil projet non serialisable"))?;
        Ok(format!("sha256:{:x}", Sha256::digest(canonical)))
    }

    pub fn proposal(
        &self,
        command_id: String,
    ) -> Result<bridget_transport::protocol::ProjectProfileProposal, DomainError> {
        if self.status != ProjectProfileStatus::Proposed || command_id.trim().is_empty() {
            return Err(DomainError::TransitionInterdite);
        }
        Ok(bridget_transport::protocol::ProjectProfileProposal {
            contract_version: bridget_transport::protocol::PROJECT_PROFILE_CONTRACT_VERSION,
            command_id,
            project: bridget_transport::protocol::ProjectReference {
                project_id: self.project_id.clone(),
                binding_generation: self.binding_generation,
            },
            profile_id: self.profile_id.clone(),
            binding_generation: self.binding_generation,
            runtime_policy_version: self.runtime_policy_version,
            policy_digest: self.policy_digest.clone(),
            generation: self.generation,
            agent_profile_ids: self.agent_profile_ids.clone(),
            agents: self.agents.clone(),
            extensions: self.extensions.clone(),
            secrets: self.secrets.clone(),
            required_capabilities: self.required_capabilities.clone(),
            profile_digest: self.digest()?,
        })
    }

    pub fn record_resolution(
        &mut self,
        resolved: ResolvedProjectProfile,
        now: i64,
    ) -> Result<(), DomainError> {
        if self.status != ProjectProfileStatus::Proposed
            || now < self.updated_at
            || resolved.proposal.profile_digest != self.digest()?
            || resolved.proposal.profile_id != self.profile_id
            || resolved.proposal.project.project_id != self.project_id
            || resolved.proposal.binding_generation != self.binding_generation
            || resolved.proposal.runtime_policy_version != self.runtime_policy_version
            || resolved.proposal.policy_digest != self.policy_digest
            || resolved.proposal.agents != self.agents
            || resolved.validate().is_err()
        {
            return Err(DomainError::TransitionInterdite);
        }
        self.resolved = Some(resolved);
        self.status = ProjectProfileStatus::Resolved;
        self.updated_at = now;
        Ok(())
    }

    pub fn approve(
        &mut self,
        resolved_digest: String,
        now: i64,
    ) -> Result<ProjectProfileApproval, DomainError> {
        if self.status != ProjectProfileStatus::Resolved
            || self
                .resolved
                .as_ref()
                .map(|profile| profile.resolved_digest.as_str())
                != Some(resolved_digest.as_str())
            || now < self.updated_at
        {
            return Err(DomainError::TransitionInterdite);
        }
        self.status = ProjectProfileStatus::Approved;
        self.updated_at = now;
        Ok(ProjectProfileApproval {
            profile_id: self.profile_id.clone(),
            project_id: self.project_id.clone(),
            binding_generation: self.binding_generation,
            runtime_policy_version: self.runtime_policy_version,
            policy_digest: self.policy_digest.clone(),
            generation: self.generation,
            profile_digest: resolved_digest,
            actor: "local_human".to_string(),
            approved_at: now,
        })
    }

    pub fn mark_stale_if_runtime_changed(
        &mut self,
        binding_generation: u64,
        runtime_policy_version: u64,
        policy_digest: &str,
        now: i64,
    ) -> Result<(), DomainError> {
        if binding_generation == 0 || policy_digest.trim().is_empty() || now < self.updated_at {
            return Err(DomainError::DonneeInvalide("transition profil invalide"));
        }
        if self.binding_generation != binding_generation
            || self.runtime_policy_version != runtime_policy_version
            || self.policy_digest != policy_digest
        {
            self.status = ProjectProfileStatus::Stale;
            self.updated_at = now;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_agent() -> ProjectProfileAgent {
        ProjectProfileAgent {
            profile_id: "coordinator".to_string(),
            agent_type: "codex".to_string(),
            model: "gpt-5".to_string(),
            effort: "high".to_string(),
            tools: vec!["shell".to_string()],
        }
    }

    fn fixture_resolution(profile: &ProjectProfile) -> ResolvedProjectProfile {
        let proposal = profile.proposal("request-a".to_string()).unwrap();
        let definition = bridget_transport::protocol::ResolvedAgentDefinition {
            command: "/bin/test".to_string(),
            args: Vec::new(),
            protocol: "acp".to_string(),
            forbidden_env: vec!["PATH".to_string()],
            pass_env: vec!["PROJECT_TOKEN".to_string()],
            permissions: "default".to_string(),
            claude_config_dir: None,
            queue_capacity: 1,
            notify_timeout_secs: 1,
            mcp: bridget_transport::protocol::ResolvedMcpDefinition {
                interactive: "stdio".to_string(),
                acp_session: true,
            },
            capabilities: bridget_transport::protocol::AdapterCapabilities::default(),
            digest: "a".repeat(64),
        };
        ResolvedProjectProfile::from_resolution(
            proposal.clone(),
            Vec::new(),
            vec![bridget_transport::protocol::ResolvedProjectAgent {
                agent: proposal.agents[0].clone(),
                definition,
            }],
            bridget_transport::protocol::ProjectRuntimeView {
                policy_id: "policy-a".to_string(),
                policy_version: 2,
                policy_digest: "sha256:policy-a".to_string(),
                image_reference: "example.invalid/runtime@sha256:abc".to_string(),
                run_as_uid: 1000,
                run_as_gid: 1000,
                cpu_limit_milli: 1000,
                memory_limit_bytes: 1024,
                pids_limit: 64,
                tmpfs: vec!["/tmp".to_string()],
                network_mode: "bridge".to_string(),
            },
        )
        .unwrap()
    }

    #[test]
    fn spec_067_profil_devient_stale_si_runtime_diverge() {
        let mut profile = ProjectProfile::propose(
            "profile-a".to_string(),
            "project-a".to_string(),
            4,
            2,
            "sha256:policy-a".to_string(),
            vec!["coordinator".to_string()],
            vec![fixture_agent()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            100,
        )
        .unwrap();
        let resolved = fixture_resolution(&profile);
        let resolved_digest = resolved.resolved_digest.clone();
        profile.record_resolution(resolved, 101).unwrap();
        let approval = profile.approve(resolved_digest, 102).unwrap();
        assert_eq!(profile.status, ProjectProfileStatus::Approved);
        assert_eq!(approval.binding_generation, 4);
        profile
            .mark_stale_if_runtime_changed(5, 2, "sha256:policy-a", 103)
            .unwrap();
        assert_eq!(profile.status, ProjectProfileStatus::Stale);
    }
    #[test]
    fn spec_067_generation_ressource_invalide_le_digest_du_profil() {
        let mut profile = ProjectProfile::propose(
            "profile-a".to_string(),
            "project-a".to_string(),
            4,
            2,
            "sha256:policy-a".to_string(),
            vec!["coordinator".to_string()],
            vec![fixture_agent()],
            vec![ProjectResourceRef {
                resource_id: "extension-a".to_string(),
                kind: bridget_transport::protocol::ProjectResourceKind::Extension,
                source_ref: "catalog:extension-a".to_string(),
                destination: "/opt/bridget/extensions/extension-a".to_string(),
                version: Some("v1".to_string()),
                content_digest: Some("sha256:extension".to_string()),
                generation: 1,
            }],
            Vec::new(),
            vec!["acp".to_string()],
            100,
        )
        .unwrap();
        let first = profile.digest().unwrap();
        profile.extensions[0].generation = 2;
        assert_ne!(first, profile.digest().unwrap());
    }

    #[test]
    fn spec_067_approbation_locale_persiste_un_profil_sans_valeur() {
        let path = std::env::temp_dir().join(format!(
            "maicie-project-profile-{}.db",
            uuid::Uuid::new_v4()
        ));
        let mut store = crate::store::MaicieStore::open(&path).unwrap();
        let profile = ProjectProfile::propose(
            "profile-a".to_string(),
            "project-a".to_string(),
            4,
            2,
            "sha256:policy-a".to_string(),
            vec!["coordinator".to_string()],
            vec![fixture_agent()],
            Vec::new(),
            Vec::new(),
            vec!["acp".to_string()],
            100,
        )
        .unwrap();
        let resolved = fixture_resolution(&profile);
        let digest = resolved.resolved_digest.clone();
        store.save_project_profile(&profile).unwrap();
        store
            .record_project_profile_resolution("profile-a", resolved, 101)
            .unwrap();
        let (approved, approval) = store
            .approve_project_profile("profile-a", digest.clone(), 102)
            .unwrap();
        assert_eq!(approved.status, ProjectProfileStatus::Approved);
        assert_eq!(approval.actor, "local_human");
        assert_eq!(approval.profile_digest, digest);
        assert_eq!(store.project_profile("profile-a").unwrap(), Some(approved));
        drop(store);
        let _ = std::fs::remove_file(path);
    }
}
