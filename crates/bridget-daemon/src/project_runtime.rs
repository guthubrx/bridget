//! Runtime Docker borné et persistant par projet.
//!
//! Ce module ne possède ni flotte, ni identité agent, ni protocole provider. Il
//! centralise seulement l'état d'environnement, la politique hôte fermée et la
//! construction déterministe des arguments Docker.

use bridget_core::router::validate_agent_id;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[path = "project_resources.rs"]
pub mod project_resources;
pub use bridget_transport::protocol::PROJECT_RUNTIME_CONTRACT_VERSION;
pub use project_resources::{ProjectResourceCatalog, ProjectResourceSource};
pub const CONTAINER_STATE_ROOT: &str = "/var/lib/bridget-project";
pub const CONTAINER_HOME: &str = "/var/lib/bridget-project/home";
pub const CONTAINER_INGRESS_DIRECTORY: &str = "/run/bridget/runtime";
pub const CONTAINER_INGRESS_SOCKET: &str = "/run/bridget/runtime/bridget.sock";
pub const CONTAINER_PROCESS_ENV_DIRECTORY: &str = "/run/bridget/secrets/process-env";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEnvironmentState {
    Absent,
    Creating,
    Ready,
    Running,
    Stopping,
    Stopped,
    Degraded,
    RecreateRequired,
}

impl ProjectEnvironmentState {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Absent, Self::Creating)
                | (
                    Self::Creating,
                    Self::Ready | Self::Degraded | Self::RecreateRequired
                )
                | (
                    Self::Ready,
                    Self::Running
                        | Self::Stopping
                        | Self::Stopped
                        | Self::RecreateRequired
                        | Self::Degraded
                )
                | (
                    Self::Running,
                    Self::Ready | Self::Stopping | Self::Degraded | Self::RecreateRequired
                )
                | (Self::Stopping, Self::Stopped | Self::Degraded)
                | (
                    Self::Stopped,
                    Self::Creating | Self::Absent | Self::RecreateRequired
                )
                | (
                    Self::Degraded,
                    Self::Creating | Self::Stopping | Self::Stopped | Self::RecreateRequired
                )
                | (
                    Self::RecreateRequired,
                    Self::Stopping | Self::Stopped | Self::Creating | Self::Absent
                )
        )
    }

    pub fn admits_spawn(self) -> bool {
        matches!(self, Self::Ready | Self::Running)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeIssue {
    InvalidTransition {
        from: ProjectEnvironmentState,
        to: ProjectEnvironmentState,
    },
    SpawnNotAdmitted(ProjectEnvironmentState),
    EnvironmentEpochStale {
        expected: u64,
        observed: u64,
    },
    RuntimePolicyChanged,
    PolicyUnavailable,
    PolicyInvalid(String),
    ImageNotPinned,
    ForbiddenMount(String),
    UnsupportedWorktreeLayout,
    InvalidProjectId,
    DockerUnavailable,
    DockerTimeout,
    DockerCommandFailed,
    DockerOutputInvalid,
    ContainerAttestationInvalid,
    RuntimeUserIncompatible,
    RuntimeExecutableUnavailable,
    RuntimeLaunchInvalid(String),
}

impl fmt::Display for RuntimeIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(formatter, "transition runtime invalide: {from:?} -> {to:?}")
            }
            Self::SpawnNotAdmitted(state) => {
                write!(formatter, "admission Docker refusée dans l'état {state:?}")
            }
            Self::EnvironmentEpochStale { expected, observed } => {
                write!(
                    formatter,
                    "epoch runtime obsolète: attendu {expected}, observé {observed}"
                )
            }
            Self::RuntimePolicyChanged => write!(formatter, "runtime_policy_changed"),
            Self::PolicyUnavailable => write!(formatter, "runtime_policy_config_unavailable"),
            Self::PolicyInvalid(reason) => write!(formatter, "policy_invalid: {reason}"),
            Self::ImageNotPinned => write!(formatter, "image_not_pinned"),
            Self::ForbiddenMount(path) => write!(formatter, "forbidden_mount: {path}"),
            Self::UnsupportedWorktreeLayout => write!(formatter, "unsupported_worktree_layout"),
            Self::InvalidProjectId => write!(formatter, "identifiant projet runtime invalide"),
            Self::DockerUnavailable => write!(formatter, "docker_unavailable"),
            Self::DockerTimeout => write!(formatter, "docker_timeout"),
            Self::DockerCommandFailed => write!(formatter, "docker_command_failed"),
            Self::DockerOutputInvalid => write!(formatter, "docker_output_invalid"),
            Self::ContainerAttestationInvalid => write!(formatter, "container_attestation_invalid"),
            Self::RuntimeUserIncompatible => write!(formatter, "runtime_user_incompatible"),
            Self::RuntimeExecutableUnavailable => {
                write!(formatter, "runtime_executable_unavailable")
            }
            Self::RuntimeLaunchInvalid(reason) => {
                write!(formatter, "runtime_launch_invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for RuntimeIssue {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectEnvironment {
    pub project_id: String,
    pub binding_generation: u64,
    pub state: ProjectEnvironmentState,
    pub environment_epoch: u64,
    pub container_id: Option<String>,
    pub policy_id: String,
    pub policy_version: u64,
    pub policy_digest: String,
    pub image_reference: String,
    pub run_as_uid: u32,
    pub run_as_gid: u32,
    pub last_reason: Option<String>,
}

impl ProjectEnvironment {
    pub fn absent(
        project_id: impl Into<String>,
        binding_generation: u64,
        policy: &ProjectRuntimePolicy,
    ) -> Result<Self, RuntimeIssue> {
        let project_id = project_id.into();
        if !is_valid_project_id(&project_id) || binding_generation == 0 {
            return Err(RuntimeIssue::InvalidProjectId);
        }
        Ok(Self {
            project_id,
            binding_generation,
            state: ProjectEnvironmentState::Absent,
            environment_epoch: 1,
            container_id: None,
            policy_id: policy.policy_id.clone(),
            policy_version: policy.policy_version,
            policy_digest: policy.digest.clone(),
            image_reference: policy.image_reference.clone(),
            run_as_uid: policy.run_as_uid,
            run_as_gid: policy.run_as_gid,
            last_reason: None,
        })
    }

    pub fn transition(
        &mut self,
        next: ProjectEnvironmentState,
        reason: Option<String>,
    ) -> Result<(), RuntimeIssue> {
        if !self.state.can_transition_to(next) {
            return Err(RuntimeIssue::InvalidTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        self.last_reason = reason;
        if matches!(next, ProjectEnvironmentState::Absent) {
            self.container_id = None;
        }
        Ok(())
    }

    pub fn reserve_spawn(&self) -> Result<RuntimeReservation, RuntimeIssue> {
        if !self.state.admits_spawn() {
            return Err(RuntimeIssue::SpawnNotAdmitted(self.state));
        }
        Ok(RuntimeReservation {
            project_id: self.project_id.clone(),
            binding_generation: self.binding_generation,
            environment_epoch: self.environment_epoch,
        })
    }

    pub fn assert_reservation(&self, reservation: &RuntimeReservation) -> Result<(), RuntimeIssue> {
        if reservation.project_id != self.project_id
            || reservation.binding_generation != self.binding_generation
            || reservation.environment_epoch != self.environment_epoch
        {
            return Err(RuntimeIssue::EnvironmentEpochStale {
                expected: self.environment_epoch,
                observed: reservation.environment_epoch,
            });
        }
        if !self.state.admits_spawn() {
            return Err(RuntimeIssue::SpawnNotAdmitted(self.state));
        }
        Ok(())
    }

    pub fn apply_runtime_policy(
        &mut self,
        policy: &ProjectRuntimePolicy,
    ) -> Result<(), RuntimeIssue> {
        let changed = self.policy_id != policy.policy_id
            || self.policy_version != policy.policy_version
            || self.policy_digest != policy.digest
            || self.image_reference != policy.image_reference
            || self.run_as_uid != policy.run_as_uid
            || self.run_as_gid != policy.run_as_gid;
        if !changed {
            return Ok(());
        }
        self.environment_epoch = self
            .environment_epoch
            .checked_add(1)
            .ok_or_else(|| RuntimeIssue::PolicyInvalid("environment_epoch épuisé".to_string()))?;
        self.policy_id = policy.policy_id.clone();
        self.policy_version = policy.policy_version;
        self.policy_digest = policy.digest.clone();
        self.image_reference = policy.image_reference.clone();
        self.run_as_uid = policy.run_as_uid;
        self.run_as_gid = policy.run_as_gid;
        self.state = ProjectEnvironmentState::RecreateRequired;
        self.last_reason = Some("runtime_policy_changed".to_string());
        Err(RuntimeIssue::RuntimePolicyChanged)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReservation {
    pub project_id: String,
    pub binding_generation: u64,
    pub environment_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageReferenceKind {
    RegistryDigest,
    LocalImageId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeExecutableDefinition {
    pub agent_type: String,
    pub provider_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRuntimeExecution {
    pub wrapper_executable: String,
    pub provider_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRuntimePolicyConfig {
    pub contract_version: u16,
    pub state_root_parent: PathBuf,
    pub policies: Vec<ProjectRuntimePolicyDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRuntimePolicyDefinition {
    pub policy_id: String,
    pub policy_version: u64,
    pub image_reference_kind: ImageReferenceKind,
    pub image_reference: String,
    pub run_as_uid: u32,
    pub run_as_gid: u32,
    pub cpu_limit: f64,
    pub memory_limit_bytes: u64,
    pub pids_limit: u32,
    pub tmpfs: Vec<String>,
    pub network_mode: String,
    /// Binaire Bridget présent dans l'image, absent tant que le runtime ne
    /// lance aucun agent. Sa présence est requise dès qu'une commande agent est déclarée.
    #[serde(default)]
    pub runtime_launcher: Option<String>,
    /// Mapping fermé type d'agent -> commande fournisseur interne à l'image.
    #[serde(default)]
    pub runtime_executables: Vec<RuntimeExecutableDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectRuntimePolicy {
    pub policy_id: String,
    pub policy_version: u64,
    pub image_reference_kind: ImageReferenceKind,
    pub image_reference: String,
    pub run_as_uid: u32,
    pub run_as_gid: u32,
    pub cpu_limit: f64,
    pub memory_limit_bytes: u64,
    pub pids_limit: u32,
    pub tmpfs: Vec<String>,
    pub network_mode: String,
    pub runtime_launcher: Option<String>,
    pub runtime_executables: BTreeMap<String, String>,
    pub state_root_parent: PathBuf,
    pub digest: String,
}

impl ProjectRuntimePolicyConfig {
    pub fn load(path: &Path) -> Result<Self, RuntimeIssue> {
        validate_policy_file_permissions(path)?;
        let bytes = fs::read(path).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|error| RuntimeIssue::PolicyInvalid(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), RuntimeIssue> {
        if self.contract_version != PROJECT_RUNTIME_CONTRACT_VERSION {
            return Err(RuntimeIssue::PolicyInvalid(
                "version de contrat runtime inconnue".to_string(),
            ));
        }
        if !self.state_root_parent.is_absolute() {
            return Err(RuntimeIssue::PolicyInvalid(
                "state_root_parent doit être absolu".to_string(),
            ));
        }
        if self.policies.is_empty() {
            return Err(RuntimeIssue::PolicyInvalid(
                "aucune politique runtime".to_string(),
            ));
        }
        for (index, policy) in self.policies.iter().enumerate() {
            policy.validate()?;
            if self.policies[..index].iter().any(|previous| {
                previous.policy_id == policy.policy_id
                    && previous.policy_version == policy.policy_version
            }) {
                return Err(RuntimeIssue::PolicyInvalid(
                    "policy_id et policy_version dupliqués".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn resolve(
        &self,
        policy_id: &str,
        policy_version: u64,
    ) -> Result<ProjectRuntimePolicy, RuntimeIssue> {
        let definition = self
            .policies
            .iter()
            .find(|policy| policy.policy_id == policy_id && policy.policy_version == policy_version)
            .ok_or(RuntimeIssue::PolicyUnavailable)?;
        definition.validate()?;
        let canonical = serde_json::to_vec(definition)
            .map_err(|error| RuntimeIssue::PolicyInvalid(error.to_string()))?;
        Ok(ProjectRuntimePolicy {
            policy_id: definition.policy_id.clone(),
            policy_version: definition.policy_version,
            image_reference_kind: definition.image_reference_kind.clone(),
            image_reference: definition.image_reference.clone(),
            run_as_uid: definition.run_as_uid,
            run_as_gid: definition.run_as_gid,
            cpu_limit: definition.cpu_limit,
            memory_limit_bytes: definition.memory_limit_bytes,
            pids_limit: definition.pids_limit,
            tmpfs: definition.tmpfs.clone(),
            network_mode: definition.network_mode.clone(),
            runtime_launcher: definition.runtime_launcher.clone(),
            runtime_executables: definition
                .runtime_executables
                .iter()
                .map(|entry| (entry.agent_type.clone(), entry.provider_command.clone()))
                .collect(),
            state_root_parent: self.state_root_parent.clone(),
            digest: format!("sha256:{:x}", Sha256::digest(canonical)),
        })
    }
}

impl ProjectRuntimePolicy {
    /// Retourne uniquement une paire d'exécutables explicitement déclarée dans
    /// la politique hôte attestée. L'absence ferme le lancement Docker.
    pub fn runtime_execution(
        &self,
        agent_type: &str,
    ) -> Result<ProjectRuntimeExecution, RuntimeIssue> {
        let wrapper_executable = self
            .runtime_launcher
            .as_ref()
            .ok_or(RuntimeIssue::RuntimeExecutableUnavailable)?;
        let provider_command = self
            .runtime_executables
            .get(agent_type)
            .ok_or(RuntimeIssue::RuntimeExecutableUnavailable)?;
        Ok(ProjectRuntimeExecution {
            wrapper_executable: wrapper_executable.clone(),
            provider_command: provider_command.clone(),
        })
    }
}
/// Digest la politique runtime et les attestations de ressources déjà résolues.
pub fn project_profile_runtime_digest(
    policy: &ProjectRuntimePolicy,
    profile: &bridget_transport::protocol::ResolvedProjectProfile,
) -> Result<String, RuntimeIssue> {
    profile
        .validate()
        .map_err(|reason| RuntimeIssue::PolicyInvalid(reason.to_string()))?;
    if profile.proposal.runtime_policy_version != policy.policy_version
        || profile.proposal.policy_digest != policy.digest
    {
        return Err(RuntimeIssue::RuntimePolicyChanged);
    }
    let canonical =
        serde_json::to_vec(&(policy.policy_version, &policy.digest, &profile.resources))
            .map_err(|error| RuntimeIssue::PolicyInvalid(error.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical)))
}

impl ProjectRuntimePolicyDefinition {
    fn validate(&self) -> Result<(), RuntimeIssue> {
        if self.policy_id.trim().is_empty()
            || self.policy_id.len() > 128
            || self.policy_version == 0
            || self.run_as_uid == 0
            || self.run_as_gid == 0
            || !self.cpu_limit.is_finite()
            || self.cpu_limit <= 0.0
            || self.memory_limit_bytes == 0
            || self.pids_limit == 0
            || self.network_mode != "bridge"
            || self.tmpfs.iter().any(|path| path != "/tmp")
        {
            return Err(RuntimeIssue::PolicyInvalid(
                "champ de politique invalide".to_string(),
            ));
        }
        validate_image_reference(&self.image_reference_kind, &self.image_reference)?;
        if self.runtime_launcher.is_none() && !self.runtime_executables.is_empty() {
            return Err(RuntimeIssue::PolicyInvalid(
                "runtime_launcher requis pour les executables".to_string(),
            ));
        }
        if let Some(runtime_launcher) = self.runtime_launcher.as_deref() {
            validate_runtime_executable_path(runtime_launcher)?;
        }
        for (index, executable) in self.runtime_executables.iter().enumerate() {
            if !is_valid_runtime_agent_type(&executable.agent_type)
                || self.runtime_executables[..index]
                    .iter()
                    .any(|previous| previous.agent_type == executable.agent_type)
            {
                return Err(RuntimeIssue::PolicyInvalid(
                    "agent_type runtime invalide ou duplique".to_string(),
                ));
            }
            validate_runtime_executable_path(&executable.provider_command)?;
        }
        Ok(())
    }
}

#[cfg(unix)]
fn validate_runtime_executable_path(path: &str) -> Result<(), RuntimeIssue> {
    let path = Path::new(path);
    if !path.is_absolute()
        || path.as_os_str().is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
        || path
            .to_string_lossy()
            .bytes()
            .any(|byte| byte.is_ascii_whitespace())
    {
        return Err(RuntimeIssue::PolicyInvalid(
            "executable runtime invalide".to_string(),
        ));
    }
    Ok(())
}

fn is_valid_runtime_agent_type(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn validate_policy_file_permissions(path: &Path) -> Result<(), RuntimeIssue> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let metadata = fs::symlink_metadata(path).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RuntimeIssue::PolicyInvalid(
            "le document de politique doit être un fichier régulier".to_string(),
        ));
    }
    if metadata.uid() != unsafe { libc::geteuid() } || metadata.permissions().mode() & 0o022 != 0 {
        return Err(RuntimeIssue::PolicyInvalid(
            "propriétaire ou permissions de politique invalides".to_string(),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_policy_file_permissions(path: &Path) -> Result<(), RuntimeIssue> {
    if path.is_file() {
        Ok(())
    } else {
        Err(RuntimeIssue::PolicyUnavailable)
    }
}

pub fn runtime_state_root(
    policy: &ProjectRuntimePolicy,
    project_id: &str,
) -> Result<PathBuf, RuntimeIssue> {
    if !is_valid_project_id(project_id) {
        return Err(RuntimeIssue::InvalidProjectId);
    }
    Ok(policy.state_root_parent.join(project_id))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMount {
    pub host_path: PathBuf,
    pub container_path: String,
    pub writable: bool,
}

/// Métadonnée non secrète transmise au wrapper après admission ingress.
/// Le chemin désigne un fichier privé déjà monté dans le conteneur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessEnvBinding {
    pub variable: String,
    pub container_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectProfileRuntimeAdmission {
    pub mounts: Vec<ProjectMount>,
    pub process_env: Vec<ProcessEnvBinding>,
    pub resolved_digest: String,
    pub profile: bridget_transport::protocol::ResolvedProjectProfile,
}

fn process_env_container_file(resource_id: &str) -> String {
    let digest = Sha256::digest(resource_id.as_bytes());
    format!("{CONTAINER_PROCESS_ENV_DIRECTORY}/{digest:x}")
}

/// Résout les références approuvées et ne construit que des montages bind en
/// lecture seule. Une référence process-env ne constitue jamais un montage Docker.
pub fn resolve_project_resource_mounts(
    catalog: &ProjectResourceCatalog,
    project_id: &str,
    references: &[bridget_transport::protocol::ProjectResourceRef],
) -> Result<
    (
        Vec<ProjectMount>,
        Vec<bridget_transport::protocol::ResolvedProjectResource>,
    ),
    RuntimeIssue,
> {
    let resources = catalog.resolve_refs(project_id, references)?;
    let mut mounts = Vec::new();
    for resource in &resources {
        if resource.reference.kind
            == bridget_transport::protocol::ProjectResourceKind::SecretProcessEnv
        {
            continue;
        }
        let source = catalog.source_for(project_id, &resource.reference.source_ref)?;
        let mount = ProjectMount {
            host_path: source.canonical_path.clone(),
            container_path: resource.reference.destination.clone(),
            writable: false,
        };
        validate_mount(&mount)?;
        mounts.push(mount);
    }
    Ok((mounts, resources))
}

/// Le backend hôte conserve son comportement historique, mais refuse toute
/// composition nécessitant une ressource Docker.
pub fn validate_project_profile_backend(
    backend: bridget_transport::protocol::ProjectBackend,
    references: &[bridget_transport::protocol::ProjectResourceRef],
) -> Result<(), RuntimeIssue> {
    if backend == bridget_transport::protocol::ProjectBackend::Host && !references.is_empty() {
        return Err(RuntimeIssue::PolicyInvalid(
            "project_profile_requires_docker".to_string(),
        ));
    }
    Ok(())
}

/**
 * Admits an approved profile by reattesting every source before Docker creation.
 * A changed generation or source stamp closes the next spawn without stopping
 * historical executions.
 */
pub fn admit_project_profile_runtime(
    catalog: &ProjectResourceCatalog,
    policy: &ProjectRuntimePolicy,
    backend: bridget_transport::protocol::ProjectBackend,
    profile: &bridget_transport::protocol::ResolvedProjectProfile,
) -> Result<ProjectProfileRuntimeAdmission, RuntimeIssue> {
    profile
        .validate()
        .map_err(|reason| RuntimeIssue::PolicyInvalid(reason.to_string()))?;
    let references = profile
        .proposal
        .extensions
        .iter()
        .chain(&profile.proposal.secrets)
        .cloned()
        .collect::<Vec<_>>();
    validate_project_profile_backend(backend, &references)?;
    if profile.proposal.runtime_policy_version != policy.policy_version
        || profile.proposal.policy_digest != policy.digest
    {
        return Err(RuntimeIssue::RuntimePolicyChanged);
    }
    let resources = catalog.resolve_refs(&profile.proposal.project.project_id, &references)?;
    let current = bridget_transport::protocol::ResolvedProjectProfile::from_resolution(
        profile.proposal.clone(),
        resources,
        profile.agents.clone(),
        profile.runtime.clone(),
    )
    .map_err(RuntimeIssue::PolicyInvalid)?;
    if current.resolved_digest != profile.resolved_digest {
        return Err(RuntimeIssue::PolicyInvalid(
            "project_profile_resource_stamp_changed".to_string(),
        ));
    }
    let _runtime_digest = project_profile_runtime_digest(policy, &current)?;
    let mut mounts = Vec::new();
    let mut process_env = Vec::new();
    for resource in &current.resources {
        let source = catalog.source_for(
            &current.proposal.project.project_id,
            &resource.reference.source_ref,
        )?;
        let container_path = if resource.reference.kind
            == bridget_transport::protocol::ProjectResourceKind::SecretProcessEnv
        {
            let container_file = process_env_container_file(&resource.reference.resource_id);
            process_env.push(ProcessEnvBinding {
                variable: resource.reference.destination.clone(),
                container_file: container_file.clone(),
            });
            container_file
        } else {
            resource.reference.destination.clone()
        };
        let mount = ProjectMount {
            host_path: source.canonical_path.clone(),
            container_path,
            writable: false,
        };
        validate_mount(&mount)?;
        mounts.push(mount);
    }
    Ok(ProjectProfileRuntimeAdmission {
        mounts,
        process_env,
        resolved_digest: current.resolved_digest.clone(),
        profile: current,
    })
}
/// Endpoint Unix privé monté dans un seul environnement Docker. Son répertoire
/// est monté en lecture seule: le daemon peut rétablir la socket après un
/// redémarrage sans changer la vue du conteneur.
pub struct RuntimeIngressEndpoint {
    pub mount_directory: PathBuf,
    pub socket_path: PathBuf,
    pub binding_generation: u64,
    pub listener: std::os::unix::net::UnixListener,
}

/// Attentes durables comparées au premier frame reçu sur l'ingress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIngressExpectation {
    pub project_id: String,
    pub binding_generation: u64,
    pub container_id: String,
    pub environment_epoch: u64,
    pub agent_generation: u64,
    pub instance_id: String,
    pub run_as_uid: u32,
}

impl RuntimeIngressExpectation {
    pub fn from_environment(
        environment: &ProjectEnvironment,
        agent_generation: u64,
        instance_id: &str,
    ) -> Result<Self, RuntimeIssue> {
        let container_id = environment
            .container_id
            .clone()
            .filter(|container_id| is_safe_container_id(container_id))
            .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
        if agent_generation == 0 || uuid::Uuid::parse_str(instance_id).is_err() {
            return Err(RuntimeIssue::InvalidProjectId);
        }
        Ok(Self {
            project_id: environment.project_id.clone(),
            binding_generation: environment.binding_generation,
            container_id,
            run_as_uid: environment.run_as_uid,
            environment_epoch: environment.environment_epoch,
            agent_generation,
            instance_id: instance_id.to_string(),
        })
    }

    pub fn validate(
        &self,
        handshake: &bridget_transport::protocol::RuntimeIngressHandshake,
    ) -> Result<(), bridget_transport::protocol::RuntimeIngressRefusal> {
        use bridget_transport::protocol::{
            RUNTIME_INGRESS_CONTRACT_VERSION, RuntimeIngressRefusal,
        };

        if handshake.contract_version != RUNTIME_INGRESS_CONTRACT_VERSION {
            return Err(RuntimeIngressRefusal::InvalidContract);
        }
        if handshake.project_id != self.project_id
            || handshake.container_id != self.container_id
            || handshake.instance_id != self.instance_id
        {
            return Err(RuntimeIngressRefusal::IdentityMismatch);
        }
        if handshake.binding_generation != self.binding_generation
            || handshake.agent_generation != self.agent_generation
        {
            return Err(RuntimeIngressRefusal::GenerationMismatch);
        }
        if handshake.environment_epoch != self.environment_epoch {
            return Err(RuntimeIngressRefusal::EnvironmentEpochStale);
        }
        Ok(())
    }
}

/// Crée ou recrée l'endpoint déterministe d'une génération attestée. Un ancien
/// socket n'est retiré que s'il est bien une socket privée appartenant à l'UID
/// du daemon, jamais par recherche large dans le système de fichiers.
pub fn bind_runtime_ingress(
    policy: &ProjectRuntimePolicy,
    environment: &ProjectEnvironment,
) -> Result<RuntimeIngressEndpoint, RuntimeIssue> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

    ensure_runtime_state_root(policy, &environment.project_id)?;
    let root = runtime_state_root(policy, &environment.project_id)?;
    let runtime_directory = root.join("runtime");
    fs::create_dir_all(&runtime_directory).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    fs::set_permissions(&runtime_directory, fs::Permissions::from_mode(0o700))
        .map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    let directory = root
        .join("runtime")
        .join(environment.binding_generation.to_string());
    fs::create_dir_all(&directory).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    let metadata = fs::metadata(&directory).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(RuntimeIssue::PolicyInvalid(
            "répertoire ingress non privé".to_string(),
        ));
    }
    let socket_path = directory.join("bridget.sock");
    if socket_path.exists() {
        let existing =
            fs::symlink_metadata(&socket_path).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
        if !existing.file_type().is_socket()
            || existing.uid() != unsafe { libc::geteuid() }
            || existing.permissions().mode() & 0o077 != 0
        {
            return Err(RuntimeIssue::PolicyInvalid(
                "socket ingress existante invalide".to_string(),
            ));
        }
        fs::remove_file(&socket_path).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    }
    let listener = std::os::unix::net::UnixListener::bind(&socket_path)
        .map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))
        .map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    Ok(RuntimeIngressEndpoint {
        mount_directory: directory,
        binding_generation: environment.binding_generation,
        socket_path,
        listener,
    })
}

/// Résout uniquement les worktrees annoncés par le dépôt du projet. Un dépôt
/// étranger ou un chemin non absolu ferme la préparation au lieu d'élargir un
/// montage Docker au hasard.
pub fn resolve_project_mounts(
    project_root: &Path,
    state_root: &Path,
) -> Result<Vec<ProjectMount>, RuntimeIssue> {
    let root = fs::canonicalize(project_root)
        .map_err(|_| RuntimeIssue::ForbiddenMount(project_root.display().to_string()))?;
    if !root.is_dir() || !root.is_absolute() {
        return Err(RuntimeIssue::UnsupportedWorktreeLayout);
    }
    let mut hosts = BTreeSet::from([root.clone()]);
    let common_dir = match git_text(
        &root,
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ) {
        Ok(common_dir) => PathBuf::from(common_dir.trim()),
        Err(_) => return project_mounts_from_hosts(hosts, state_root),
    };
    let output = git_text(&root, ["worktree", "list", "--porcelain"])?;
    for candidate in parse_worktree_list(&output)? {
        let candidate =
            fs::canonicalize(candidate).map_err(|_| RuntimeIssue::UnsupportedWorktreeLayout)?;
        let candidate_common = PathBuf::from(
            git_text(
                &candidate,
                ["rev-parse", "--path-format=absolute", "--git-common-dir"],
            )?
            .trim(),
        );
        if candidate_common != common_dir {
            return Err(RuntimeIssue::UnsupportedWorktreeLayout);
        }
        hosts.insert(candidate);
    }
    project_mounts_from_hosts(hosts, state_root)
}

fn project_mounts_from_hosts(
    hosts: BTreeSet<PathBuf>,
    state_root: &Path,
) -> Result<Vec<ProjectMount>, RuntimeIssue> {
    let mut mounts = hosts
        .into_iter()
        .map(|host_path| ProjectMount {
            container_path: host_path.to_string_lossy().into_owned(),
            host_path,
            writable: true,
        })
        .collect::<Vec<_>>();
    mounts.push(ProjectMount {
        host_path: state_root.to_path_buf(),
        container_path: CONTAINER_STATE_ROOT.to_string(),
        writable: true,
    });
    for mount in &mounts {
        validate_mount(mount)?;
    }
    Ok(mounts)
}

fn parse_worktree_list(output: &str) -> Result<Vec<PathBuf>, RuntimeIssue> {
    let mut worktrees = Vec::new();
    for record in output.split("\n\n") {
        let Some(first) = record.lines().next() else {
            continue;
        };
        let Some(path) = first.strip_prefix("worktree ") else {
            return Err(RuntimeIssue::UnsupportedWorktreeLayout);
        };
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(RuntimeIssue::UnsupportedWorktreeLayout);
        }
        worktrees.push(path);
    }
    if worktrees.is_empty() {
        return Err(RuntimeIssue::UnsupportedWorktreeLayout);
    }
    Ok(worktrees)
}

fn git_text<const N: usize>(root: &Path, args: [&str; N]) -> Result<String, RuntimeIssue> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|_| RuntimeIssue::UnsupportedWorktreeLayout)?;
    if !output.status.success() || output.stdout.len() > 64 * 1024 {
        return Err(RuntimeIssue::UnsupportedWorktreeLayout);
    }
    String::from_utf8(output.stdout).map_err(|_| RuntimeIssue::UnsupportedWorktreeLayout)
}

pub fn docker_create_arguments(
    environment: &ProjectEnvironment,
    policy: &ProjectRuntimePolicy,
    mounts: &[ProjectMount],
) -> Result<Vec<String>, RuntimeIssue> {
    if environment.state != ProjectEnvironmentState::Creating {
        return Err(RuntimeIssue::SpawnNotAdmitted(environment.state));
    }
    if environment.project_id.is_empty() || environment.binding_generation == 0 {
        return Err(RuntimeIssue::InvalidProjectId);
    }
    let mut arguments = vec![
        "create".to_string(),
        "--name".to_string(),
        container_name(&environment.project_id),
        "--label".to_string(),
        format!("bridget.project_id={}", environment.project_id),
        "--label".to_string(),
        format!(
            "bridget.binding_generation={}",
            environment.binding_generation
        ),
        "--label".to_string(),
        format!("bridget.policy_digest={}", policy.digest),
        "--label".to_string(),
        format!("bridget.runtime_contract={PROJECT_RUNTIME_CONTRACT_VERSION}"),
        "--read-only".to_string(),
        "--cap-drop=ALL".to_string(),
        "--security-opt=no-new-privileges".to_string(),
        "--user".to_string(),
        format!("{}:{}", policy.run_as_uid, policy.run_as_gid),
        "--network".to_string(),
        "bridge".to_string(),
        "--cpus".to_string(),
        policy.cpu_limit.to_string(),
        "--memory".to_string(),
        policy.memory_limit_bytes.to_string(),
        "--pids-limit".to_string(),
        policy.pids_limit.to_string(),
        "--env".to_string(),
        format!("HOME={CONTAINER_HOME}"),
        "--env".to_string(),
        format!("XDG_CONFIG_HOME={CONTAINER_STATE_ROOT}/xdg/config"),
        "--env".to_string(),
        format!("XDG_CACHE_HOME={CONTAINER_STATE_ROOT}/xdg/cache"),
        "--env".to_string(),
        format!("XDG_DATA_HOME={CONTAINER_STATE_ROOT}/xdg/data"),
        "--env".to_string(),
        format!("XDG_STATE_HOME={CONTAINER_STATE_ROOT}/xdg/state"),
        "--env".to_string(),
        format!("BRIDGET_RUNTIME_SOCKET={CONTAINER_INGRESS_SOCKET}"),
    ];
    for tmpfs in &policy.tmpfs {
        arguments.push("--tmpfs".to_string());
        arguments.push(tmpfs.clone());
    }
    for mount in mounts {
        validate_mount(mount)?;
        let readonly = if mount.writable { "" } else { ",readonly" };
        arguments.push("--mount".to_string());
        arguments.push(format!(
            "type=bind,src={},dst={}{}",
            mount.host_path.display(),
            mount.container_path,
            readonly
        ));
    }
    arguments.push(policy.image_reference.clone());
    arguments.push("sleep".to_string());
    arguments.push("infinity".to_string());
    Ok(arguments)
}

/// Commande `docker exec` construite par le daemon après admission du lease.
/// Ses valeurs viennent exclusivement du binding persistant, de la politique
/// hôte et de la définition déjà figée par la saga de flotte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerRuntimeLaunch {
    pub reservation: RuntimeReservation,
    pub container_id: String,
    pub run_as_uid: u32,
    pub run_as_gid: u32,
    pub execution: ProjectRuntimeExecution,
    pub exec_id: String,
    pub agent_type: String,
    pub agent_id: String,
    pub instance_id: String,
    pub agent_generation: u64,
    pub cwd: PathBuf,
    pub resolved_definition_json: String,
    pub process_env: Vec<ProcessEnvBinding>,
}

impl DockerRuntimeLaunch {
    pub fn docker_stop_arguments(&self) -> Result<Vec<String>, RuntimeIssue> {
        if !is_container_id(&self.container_id) || uuid::Uuid::parse_str(&self.instance_id).is_err()
        {
            return Err(RuntimeIssue::RuntimeLaunchInvalid(
                "identité runtime invalide pour l'arrêt".to_string(),
            ));
        }
        validate_runtime_executable_path(&self.execution.wrapper_executable)?;
        Ok(vec![
            "exec".to_string(),
            "--user".to_string(),
            format!("{}:{}", self.run_as_uid, self.run_as_gid),
            self.container_id.clone(),
            self.execution.wrapper_executable.clone(),
            "managed-runtime-stop".to_string(),
            self.instance_id.clone(),
        ])
    }

    pub fn docker_exec_arguments(&self) -> Result<Vec<String>, RuntimeIssue> {
        if !is_valid_runtime_agent_type(&self.agent_type)
            || validate_agent_id(&self.agent_id).is_err()
            || self.agent_generation == 0
            || !self.cwd.is_absolute()
            || self.resolved_definition_json.is_empty()
            || self.resolved_definition_json.len() > 64 * 1024
            || !is_container_id(&self.container_id)
            || uuid::Uuid::parse_str(&self.instance_id).is_err()
            || uuid::Uuid::parse_str(&self.exec_id).is_err()
        {
            return Err(RuntimeIssue::RuntimeLaunchInvalid(
                "identité ou charge utile runtime invalide".to_string(),
            ));
        }
        validate_runtime_executable_path(&self.execution.wrapper_executable)?;
        validate_runtime_executable_path(&self.execution.provider_command)?;
        for binding in &self.process_env {
            if !is_valid_process_env_binding(binding) {
                return Err(RuntimeIssue::RuntimeLaunchInvalid(
                    "liaison process-env invalide".to_string(),
                ));
            }
        }
        let process_env_json = serde_json::to_string(&self.process_env).map_err(|_| {
            RuntimeIssue::RuntimeLaunchInvalid("liaison process-env invalide".to_string())
        })?;
        let mut arguments = vec![
            "exec".to_string(),
            "--user".to_string(),
            format!("{}:{}", self.run_as_uid, self.run_as_gid),
            "--workdir".to_string(),
            self.cwd.display().to_string(),
            "--env".to_string(),
            format!("BRIDGET_RUNTIME_SOCKET={CONTAINER_INGRESS_SOCKET}"),
            "--env".to_string(),
            format!("BRIDGET_RUNTIME_PROJECT_ID={}", self.reservation.project_id),
            "--env".to_string(),
            format!(
                "BRIDGET_RUNTIME_BINDING_GENERATION={}",
                self.reservation.binding_generation
            ),
            "--env".to_string(),
            format!("BRIDGET_RUNTIME_CONTAINER_ID={}", self.container_id),
            "--env".to_string(),
            format!(
                "BRIDGET_RUNTIME_ENVIRONMENT_EPOCH={}",
                self.reservation.environment_epoch
            ),
            "--env".to_string(),
            format!("BRIDGET_RUNTIME_AGENT_GENERATION={}", self.agent_generation),
            "--env".to_string(),
            format!("BRIDGET_RUNTIME_INSTANCE_ID={}", self.instance_id),
        ];
        if !self.process_env.is_empty() {
            arguments.push("--env".to_string());
            arguments.push(format!(
                "BRIDGET_RUNTIME_SECRET_ENV_FILES={process_env_json}"
            ));
        }
        arguments.extend([
            self.container_id.clone(),
            self.execution.wrapper_executable.clone(),
            "managed-runtime-wrapper".to_string(),
            self.agent_type.clone(),
            self.agent_id.clone(),
            self.execution.provider_command.clone(),
            self.resolved_definition_json.clone(),
        ]);
        Ok(arguments)
    }
    pub fn durable_execution(&self) -> crate::desired_state::ContainerAgentExecution {
        crate::desired_state::ContainerAgentExecution {
            agent_instance_id: self.instance_id.clone(),
            generation: self.agent_generation,
            project_id: self.reservation.project_id.clone(),
            binding_generation: self.reservation.binding_generation,
            environment_epoch: self.reservation.environment_epoch,
            container_id: self.container_id.clone(),
            exec_id: self.exec_id.clone(),
            cwd: self.cwd.clone(),
            state: crate::desired_state::ContainerAgentExecutionState::Starting,
            provider_identity: self.execution.provider_command.clone(),
        }
    }
}

fn is_valid_process_env_binding(binding: &ProcessEnvBinding) -> bool {
    !binding.variable.is_empty()
        && binding
            .variable
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        && binding
            .container_file
            .starts_with(CONTAINER_PROCESS_ENV_DIRECTORY)
        && binding.container_file.len() == CONTAINER_PROCESS_ENV_DIRECTORY.len() + 65
        && binding.container_file.as_bytes()[CONTAINER_PROCESS_ENV_DIRECTORY.len()] == b'/'
}

fn is_container_id(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Debug, Clone)]
pub struct DockerCli {
    executable: PathBuf,
    timeout: Duration,
}

impl DockerCli {
    pub fn new(executable: PathBuf, timeout: Duration) -> Self {
        Self {
            executable,
            timeout,
        }
    }

    /// Démarre un wrapper dans le conteneur sans shell ni environnement hôte.
    /// Le processus retourné est long-vivant et reste à superviser par le
    /// daemon ; les appels de préparation bornés utilisent `invoke_text`.
    pub fn spawn_runtime(
        &self,
        launch: &DockerRuntimeLaunch,
    ) -> Result<std::process::Child, RuntimeIssue> {
        self.spawn_runtime_with_stderr(launch, Stdio::null())
    }

    /// Variante du superviseur: stderr reste dans le journal privé de
    /// l'instance, au lieu d'être laissé dans un tube que personne ne lit.
    pub fn spawn_runtime_with_stderr(
        &self,
        launch: &DockerRuntimeLaunch,
        stderr: Stdio,
    ) -> Result<std::process::Child, RuntimeIssue> {
        let arguments = launch.docker_exec_arguments()?;
        Command::new(&self.executable)
            .args(&arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .map_err(|_| RuntimeIssue::DockerUnavailable)
    }

    /// Demande au lanceur déjà présent dans l'image de terminer l'instance
    /// attestée. Tuer le client `docker exec` ne suffit pas: Docker laisse la
    /// commande interne active après la déconnexion du client.
    pub fn stop_runtime(&self, launch: &DockerRuntimeLaunch) -> Result<(), RuntimeIssue> {
        let arguments = launch.docker_stop_arguments()?;
        let mut child = Command::new(&self.executable)
            .args(&arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| RuntimeIssue::DockerUnavailable)?;
        let started_at = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(_)) => return Err(RuntimeIssue::DockerCommandFailed),
                Ok(None) if started_at.elapsed() >= self.timeout => {
                    return Err(RuntimeIssue::DockerTimeout);
                }
                Ok(None) => thread::sleep(Duration::from_millis(5)),
                Err(_) => return Err(RuntimeIssue::DockerCommandFailed),
            }
        }
    }

    pub fn invoke_text(
        &self,
        arguments: &[&str],
        environment: &[(&str, &str)],
    ) -> Result<Vec<u8>, RuntimeIssue> {
        if arguments.is_empty() {
            return Err(RuntimeIssue::DockerCommandFailed);
        }
        let mut child = Command::new(&self.executable)
            .args(arguments)
            .envs(environment.iter().copied())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| RuntimeIssue::DockerUnavailable)?;
        let started_at = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child
                        .wait_with_output()
                        .map_err(|_| RuntimeIssue::DockerCommandFailed)?;
                    if !status.success() {
                        return Err(RuntimeIssue::DockerCommandFailed);
                    }
                    if output.stdout.len() > 64 * 1024 {
                        return Err(RuntimeIssue::DockerOutputInvalid);
                    }
                    return Ok(output.stdout);
                }
                Ok(None) if started_at.elapsed() >= self.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(RuntimeIssue::DockerTimeout);
                }
                Ok(None) => thread::sleep(Duration::from_millis(5)),
                Err(_) => return Err(RuntimeIssue::DockerCommandFailed),
            }
        }
    }

    pub fn invoke_json(
        &self,
        arguments: &[&str],
        environment: &[(&str, &str)],
    ) -> Result<serde_json::Value, RuntimeIssue> {
        let output = self.invoke_text(arguments, environment)?;
        serde_json::from_slice(&output).map_err(|_| RuntimeIssue::DockerOutputInvalid)
    }
}

/// Les bind mounts privés appartiennent à l'UID/GID du daemon. Une politique
/// qui demanderait une autre identité ne peut pas être préparée honnêtement :
/// elle échouerait dans le conteneur ou ouvrirait les droits du state root.
fn assert_runtime_user_compatible(policy: &ProjectRuntimePolicy) -> Result<(), RuntimeIssue> {
    if policy.run_as_uid != unsafe { libc::geteuid() }
        || policy.run_as_gid != unsafe { libc::getegid() }
    {
        return Err(RuntimeIssue::RuntimeUserIncompatible);
    }
    Ok(())
}

pub fn preflight_runtime(
    docker: &DockerCli,
    policy: &ProjectRuntimePolicy,
) -> Result<String, RuntimeIssue> {
    docker.invoke_json(&["version", "--format", "{{json .}}"], &[])?;
    assert_runtime_user_compatible(policy)?;
    let image = docker.invoke_json(
        &[
            "image",
            "inspect",
            "--format",
            "{{json .}}",
            &policy.image_reference,
        ],
        &[],
    )?;
    let image_id = image
        .pointer("/Id")
        .and_then(serde_json::Value::as_str)
        .filter(|value| value.starts_with("sha256:"))
        .ok_or(RuntimeIssue::ImageNotPinned)?;
    Ok(image_id.to_string())
}

/// Convertit une observation Docker bornée en raison publique fermée. Les
/// détails Docker ne sont jamais conservés ni exposés à l'interface.
pub fn classify_runtime_failure(
    inspection: Option<&serde_json::Value>,
    docker_reachable: bool,
) -> &'static str {
    if !docker_reachable {
        return "docker_restarted";
    }
    if inspection.and_then(|value| value.pointer("/State/OOMKilled"))
        == Some(&serde_json::Value::Bool(true))
    {
        return "resource_oom";
    }
    if inspection
        .and_then(|value| value.pointer("/State/Error"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|error| error.to_ascii_lowercase().contains("pid"))
    {
        return "resource_pid_limit";
    }
    "runtime_exec_lost"
}

pub fn prepare_environment(
    docker: &DockerCli,
    environment: &mut ProjectEnvironment,
    policy: &ProjectRuntimePolicy,
    mounts: &[ProjectMount],
) -> Result<serde_json::Value, RuntimeIssue> {
    ensure_runtime_state_root(policy, &environment.project_id)?;
    let resolved_image_id = preflight_runtime(docker, policy)?;
    if environment.state == ProjectEnvironmentState::Absent {
        environment.transition(ProjectEnvironmentState::Creating, None)?;
    } else if environment.state != ProjectEnvironmentState::Creating {
        return Err(RuntimeIssue::SpawnNotAdmitted(environment.state));
    }
    let create_arguments = docker_create_arguments(environment, policy, mounts)?;
    let create_references = create_arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let created = docker.invoke_text(&create_references, &[]);
    let container_id = match created {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => return fail_prepare(environment, RuntimeIssue::DockerOutputInvalid),
        },
        Err(error) => return fail_prepare(environment, error),
    };
    environment.container_id = Some(container_id.clone());
    if let Err(error) = docker.invoke_text(&["start", &container_id], &[]) {
        let _ = docker.invoke_text(&["rm", "--force", &container_id], &[]);
        return fail_prepare(environment, error);
    }
    let inspection =
        match docker.invoke_json(&["inspect", "--format", "{{json .}}", &container_id], &[]) {
            Ok(inspection) => inspection,
            Err(error) => return fail_prepare(environment, error),
        };
    if let Err(error) =
        attest_container(&inspection, environment, policy, mounts, &resolved_image_id)
    {
        return fail_prepare(environment, error);
    }
    environment.transition(ProjectEnvironmentState::Ready, None)?;
    Ok(inspection)
}

/// Arrête uniquement le conteneur attesté. L'état durable devient
/// `stopped` et l'identifiant reste disponible pour une suppression explicite.
pub fn stop_environment(
    docker: &DockerCli,
    environment: &mut ProjectEnvironment,
) -> Result<(), RuntimeIssue> {
    let Some(container_id) = environment.container_id.clone() else {
        return match environment.state {
            ProjectEnvironmentState::Absent | ProjectEnvironmentState::Stopped => Ok(()),
            state => Err(RuntimeIssue::InvalidTransition {
                from: state,
                to: ProjectEnvironmentState::Stopped,
            }),
        };
    };
    if !is_safe_container_id(&container_id) {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    match environment.state {
        ProjectEnvironmentState::Stopped => return Ok(()),
        ProjectEnvironmentState::Ready
        | ProjectEnvironmentState::Running
        | ProjectEnvironmentState::RecreateRequired
        | ProjectEnvironmentState::Degraded => {
            environment.transition(ProjectEnvironmentState::Stopping, None)?;
        }
        ProjectEnvironmentState::Stopping => {
            return Err(RuntimeIssue::InvalidTransition {
                from: ProjectEnvironmentState::Stopping,
                to: ProjectEnvironmentState::Stopped,
            });
        }
        ProjectEnvironmentState::Absent | ProjectEnvironmentState::Creating => {
            return Err(RuntimeIssue::InvalidTransition {
                from: environment.state,
                to: ProjectEnvironmentState::Stopped,
            });
        }
    }
    if let Err(error) = docker.invoke_text(&["stop", &container_id], &[]) {
        return fail_prepare(environment, error);
    }
    environment.transition(ProjectEnvironmentState::Stopped, None)
}

/// Supprime uniquement un conteneur déjà arrêté et attesté. Il n'existe aucune
/// recherche par nom ni nettoyage global.
pub fn remove_environment(
    docker: &DockerCli,
    environment: &mut ProjectEnvironment,
) -> Result<(), RuntimeIssue> {
    let Some(container_id) = environment.container_id.clone() else {
        return match environment.state {
            ProjectEnvironmentState::Absent => Ok(()),
            state => Err(RuntimeIssue::InvalidTransition {
                from: state,
                to: ProjectEnvironmentState::Absent,
            }),
        };
    };
    if !is_safe_container_id(&container_id) {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    if environment.state != ProjectEnvironmentState::Stopped {
        return Err(RuntimeIssue::InvalidTransition {
            from: environment.state,
            to: ProjectEnvironmentState::Absent,
        });
    }
    if let Err(error) = docker.invoke_text(&["rm", &container_id], &[]) {
        return fail_prepare(environment, error);
    }
    environment.transition(ProjectEnvironmentState::Absent, None)
}

/// Arrête puis supprime uniquement le conteneur attesté par son identifiant
/// durable. Il n'existe aucune recherche par nom ni nettoyage global.
pub fn stop_remove_environment(
    docker: &DockerCli,
    environment: &mut ProjectEnvironment,
) -> Result<(), RuntimeIssue> {
    let Some(container_id) = environment.container_id.clone() else {
        if environment.state != ProjectEnvironmentState::Absent {
            environment.transition(ProjectEnvironmentState::Absent, None)?;
        }
        return Ok(());
    };
    if !is_safe_container_id(&container_id) {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    match environment.state {
        ProjectEnvironmentState::Ready | ProjectEnvironmentState::Running => {
            environment.transition(ProjectEnvironmentState::Stopping, None)?;
        }
        ProjectEnvironmentState::RecreateRequired | ProjectEnvironmentState::Degraded => {
            environment.transition(ProjectEnvironmentState::Stopping, None)?;
        }
        ProjectEnvironmentState::Stopped | ProjectEnvironmentState::Stopping => {}
        ProjectEnvironmentState::Absent | ProjectEnvironmentState::Creating => {
            return Err(RuntimeIssue::InvalidTransition {
                from: environment.state,
                to: ProjectEnvironmentState::Stopped,
            });
        }
    }
    if environment.state == ProjectEnvironmentState::Stopping {
        if let Err(error) = docker.invoke_text(&["stop", &container_id], &[]) {
            return fail_prepare(environment, error);
        }
        environment.transition(ProjectEnvironmentState::Stopped, None)?;
    }
    if let Err(error) = docker.invoke_text(&["rm", &container_id], &[]) {
        return fail_prepare(environment, error);
    }
    environment.transition(ProjectEnvironmentState::Absent, None)
}

fn fail_prepare<T>(
    environment: &mut ProjectEnvironment,
    error: RuntimeIssue,
) -> Result<T, RuntimeIssue> {
    let target = if matches!(error, RuntimeIssue::ContainerAttestationInvalid) {
        ProjectEnvironmentState::RecreateRequired
    } else {
        ProjectEnvironmentState::Degraded
    };
    if environment.state == ProjectEnvironmentState::Creating {
        let _ = environment.transition(target, Some(error.to_string()));
    } else if environment.state == ProjectEnvironmentState::Stopping {
        let _ = environment.transition(ProjectEnvironmentState::Degraded, Some(error.to_string()));
    }
    Err(error)
}

fn attest_container(
    inspection: &serde_json::Value,
    environment: &ProjectEnvironment,
    policy: &ProjectRuntimePolicy,
    mounts: &[ProjectMount],
    resolved_image_id: &str,
) -> Result<(), RuntimeIssue> {
    let labels = inspection
        .pointer("/Config/Labels")
        .and_then(serde_json::Value::as_object)
        .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
    let expected = [
        ("bridget.project_id", environment.project_id.as_str()),
        ("bridget.binding_generation", ""),
        ("bridget.policy_digest", policy.digest.as_str()),
        ("bridget.runtime_contract", "1"),
    ];
    for (name, expected_value) in expected {
        let value = labels
            .get(name)
            .and_then(serde_json::Value::as_str)
            .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
        let expected_value = if name == "bridget.binding_generation" {
            environment.binding_generation.to_string()
        } else {
            expected_value.to_string()
        };
        if value != expected_value {
            return Err(RuntimeIssue::ContainerAttestationInvalid);
        }
    }
    let expected_user = format!("{}:{}", policy.run_as_uid, policy.run_as_gid);
    if inspection.pointer("/HostConfig/ReadonlyRootfs") != Some(&serde_json::Value::Bool(true))
        || inspection
            .pointer("/Config/User")
            .and_then(serde_json::Value::as_str)
            != Some(expected_user.as_str())
        || inspection
            .pointer("/HostConfig/NetworkMode")
            .and_then(serde_json::Value::as_str)
            != Some("bridge")
        || inspection
            .pointer("/Image")
            .and_then(serde_json::Value::as_str)
            != Some(resolved_image_id)
        || inspection
            .pointer("/HostConfig/Memory")
            .and_then(serde_json::Value::as_i64)
            != Some(policy.memory_limit_bytes as i64)
        || inspection
            .pointer("/HostConfig/PidsLimit")
            .and_then(serde_json::Value::as_i64)
            != Some(policy.pids_limit as i64)
        || inspection
            .pointer("/HostConfig/NanoCpus")
            .and_then(serde_json::Value::as_i64)
            != Some((policy.cpu_limit * 1_000_000_000.0) as i64)
    {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    if !has_no_port_bindings(inspection.pointer("/HostConfig/PortBindings")) {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    let cap_drop = inspection
        .pointer("/HostConfig/CapDrop")
        .and_then(serde_json::Value::as_array)
        .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
    if cap_drop.len() != 1 || cap_drop[0].as_str() != Some("ALL") {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    let security = inspection
        .pointer("/HostConfig/SecurityOpt")
        .and_then(serde_json::Value::as_array)
        .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
    if !security
        .iter()
        .any(|value| value.as_str() == Some("no-new-privileges"))
    {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    let configured_env = inspection
        .pointer("/Config/Env")
        .and_then(serde_json::Value::as_array)
        .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
    for expected_env in [
        format!("HOME={CONTAINER_HOME}"),
        format!("XDG_CONFIG_HOME={CONTAINER_STATE_ROOT}/xdg/config"),
        format!("XDG_CACHE_HOME={CONTAINER_STATE_ROOT}/xdg/cache"),
        format!("BRIDGET_RUNTIME_SOCKET={CONTAINER_INGRESS_SOCKET}"),
        format!("XDG_DATA_HOME={CONTAINER_STATE_ROOT}/xdg/data"),
        format!("XDG_STATE_HOME={CONTAINER_STATE_ROOT}/xdg/state"),
    ] {
        if !configured_env
            .iter()
            .any(|entry| entry.as_str() == Some(expected_env.as_str()))
        {
            return Err(RuntimeIssue::ContainerAttestationInvalid);
        }
    }
    let inspected_mounts = inspection
        .pointer("/Mounts")
        .and_then(serde_json::Value::as_array)
        .ok_or(RuntimeIssue::ContainerAttestationInvalid)?;
    if inspected_mounts.len() != mounts.len() {
        return Err(RuntimeIssue::ContainerAttestationInvalid);
    }
    for expected_mount in mounts {
        validate_mount(expected_mount)?;
        let found = inspected_mounts.iter().any(|mount| {
            mount.get("Type").and_then(serde_json::Value::as_str) == Some("bind")
                && mount.get("Source").and_then(serde_json::Value::as_str)
                    == Some(expected_mount.host_path.to_string_lossy().as_ref())
                && mount.get("Destination").and_then(serde_json::Value::as_str)
                    == Some(expected_mount.container_path.as_str())
                && mount.get("RW").and_then(serde_json::Value::as_bool)
                    == Some(expected_mount.writable)
        });
        if !found {
            return Err(RuntimeIssue::ContainerAttestationInvalid);
        }
    }
    Ok(())
}

fn ensure_runtime_state_root(
    policy: &ProjectRuntimePolicy,
    project_id: &str,
) -> Result<(), RuntimeIssue> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let parent = &policy.state_root_parent;
    if !parent.is_absolute() || parent == Path::new("/") {
        return Err(RuntimeIssue::PolicyInvalid(
            "state root parent invalide".to_string(),
        ));
    }
    fs::create_dir_all(parent).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    let root = runtime_state_root(policy, project_id)?;
    fs::create_dir_all(&root).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    let metadata = fs::metadata(&root).map_err(|_| RuntimeIssue::PolicyUnavailable)?;
    if metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
        || !metadata.is_dir()
    {
        return Err(RuntimeIssue::PolicyInvalid(
            "state root non prive".to_string(),
        ));
    }
    Ok(())
}

fn has_no_port_bindings(value: Option<&serde_json::Value>) -> bool {
    matches!(value, None | Some(serde_json::Value::Null))
        || value.is_some_and(|value| value.as_object().is_some_and(|ports| ports.is_empty()))
}

fn is_safe_container_id(value: &str) -> bool {
    value.len() >= 12 && value.len() <= 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_mount(mount: &ProjectMount) -> Result<(), RuntimeIssue> {
    if !mount.host_path.is_absolute()
        || mount.host_path == Path::new("/")
        || !mount.container_path.starts_with('/')
        || mount.container_path == "/"
        || mount.host_path.starts_with("/home/moi/.cache")
        || mount.host_path == Path::new("/var/run/docker.sock")
    {
        return Err(RuntimeIssue::ForbiddenMount(
            mount.host_path.display().to_string(),
        ));
    }
    Ok(())
}

fn validate_image_reference(
    kind: &ImageReferenceKind,
    reference: &str,
) -> Result<(), RuntimeIssue> {
    let is_digest = |value: &str| {
        value.len() == 71
            && value.starts_with("sha256:")
            && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
    };
    match kind {
        ImageReferenceKind::RegistryDigest
            if reference.contains("@sha256:")
                && reference
                    .split_once('@')
                    .is_some_and(|(repository, digest)| {
                        !repository.is_empty() && is_digest(digest)
                    }) =>
        {
            Ok(())
        }
        ImageReferenceKind::LocalImageId if is_digest(reference) => Ok(()),
        _ => Err(RuntimeIssue::ImageNotPinned),
    }
}

fn container_name(project_id: &str) -> String {
    let digest = Sha256::digest(project_id.as_bytes());
    format!("bridget-project-{}", &format!("{digest:x}")[..16])
}

fn is_valid_project_id(project_id: &str) -> bool {
    !project_id.is_empty()
        && project_id.len() <= 128
        && project_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn definition() -> ProjectRuntimePolicyDefinition {
        ProjectRuntimePolicyDefinition {
            policy_id: "fixture-local".to_string(),
            policy_version: 1,
            image_reference_kind: ImageReferenceKind::LocalImageId,
            image_reference: format!("sha256:{}", "a".repeat(64)),
            run_as_uid: 1002,
            run_as_gid: 1002,
            cpu_limit: 2.0,
            memory_limit_bytes: 4_294_967_296,
            pids_limit: 512,
            tmpfs: vec!["/tmp".to_string()],
            network_mode: "bridge".to_string(),
            runtime_launcher: Some("/usr/local/bin/bridget".to_string()),
            runtime_executables: vec![RuntimeExecutableDefinition {
                agent_type: "fixture".to_string(),
                provider_command: "/usr/local/bin/fixture-agent".to_string(),
            }],
        }
    }

    fn policy() -> ProjectRuntimePolicy {
        let config = ProjectRuntimePolicyConfig {
            contract_version: PROJECT_RUNTIME_CONTRACT_VERSION,
            state_root_parent: PathBuf::from("/tmp/bridget-project-runtime-tests"),
            policies: vec![definition()],
        };
        config.resolve("fixture-local", 1).unwrap()
    }

    #[test]
    fn project_environment_refuses_invalid_transitions_and_reserves_current_epoch() {
        let policy = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 2, &policy).unwrap();
        assert!(matches!(
            environment.transition(ProjectEnvironmentState::Ready, None),
            Err(RuntimeIssue::InvalidTransition { .. })
        ));
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        environment
            .transition(ProjectEnvironmentState::Ready, None)
            .unwrap();
        let reservation = environment.reserve_spawn().unwrap();
        assert_eq!(reservation.environment_epoch, 1);
        environment
            .transition(ProjectEnvironmentState::Running, None)
            .unwrap();
        environment.assert_reservation(&reservation).unwrap();
    }

    #[test]
    fn policy_change_requires_recreation_and_invalidates_reservation() {
        let old = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 1, &old).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        environment
            .transition(ProjectEnvironmentState::Ready, None)
            .unwrap();
        let reservation = environment.reserve_spawn().unwrap();
        let mut changed = old.clone();
        changed.policy_version = 2;
        changed.digest = "sha256:changed".to_string();
        assert_eq!(
            environment.apply_runtime_policy(&changed),
            Err(RuntimeIssue::RuntimePolicyChanged)
        );
        assert_eq!(environment.state, ProjectEnvironmentState::RecreateRequired);
        assert_eq!(environment.environment_epoch, 2);
        assert!(matches!(
            environment.assert_reservation(&reservation),
            Err(RuntimeIssue::EnvironmentEpochStale { .. })
        ));
    }

    #[test]
    fn absent_policy_file_closes_only_docker_runtime() {
        let absent = Path::new("/tmp/bridget-runtime-policy-absent.json");
        assert!(matches!(
            ProjectRuntimePolicyConfig::load(absent),
            Err(RuntimeIssue::PolicyUnavailable)
        ));
    }

    #[test]
    fn policy_config_rejects_mutable_images_and_invalid_runtime_values() {
        let mut invalid = definition();
        invalid.image_reference = "fixture:latest".to_string();
        assert_eq!(invalid.validate(), Err(RuntimeIssue::ImageNotPinned));
        invalid.image_reference = format!("sha256:{}", "b".repeat(64));
        invalid.run_as_uid = 0;
        assert!(matches!(
            invalid.validate(),
            Err(RuntimeIssue::PolicyInvalid(_))
        ));
    }

    #[test]
    fn docker_arguments_are_closed_and_do_not_use_a_shell() {
        let policy = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 1, &policy).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        let root = PathBuf::from("/tmp/project-a");
        let state = runtime_state_root(&policy, "project-a").unwrap();
        let arguments = docker_create_arguments(
            &environment,
            &policy,
            &[
                ProjectMount {
                    host_path: root.clone(),
                    container_path: root.display().to_string(),
                    writable: true,
                },
                ProjectMount {
                    host_path: state,
                    container_path: CONTAINER_STATE_ROOT.to_string(),
                    writable: true,
                },
            ],
        )
        .unwrap();
        assert_eq!(arguments.first().map(String::as_str), Some("create"));
        assert!(arguments.iter().any(|argument| argument == "--read-only"));
        assert!(
            arguments
                .iter()
                .any(|argument| argument == "--cap-drop=ALL")
        );
        assert!(
            arguments
                .iter()
                .any(|argument| argument == "--security-opt=no-new-privileges")
        );
        assert!(
            arguments
                .iter()
                .any(|argument| argument == &format!("HOME={CONTAINER_HOME}"))
        );
        assert!(
            arguments.iter().any(|argument| argument
                == &format!("BRIDGET_RUNTIME_SOCKET={CONTAINER_INGRESS_SOCKET}"))
        );
        assert!(!arguments.iter().any(|argument| argument.contains("sh -c")));
    }

    #[test]
    fn docker_arguments_refuse_host_root_and_docker_socket_mounts() {
        let policy = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 1, &policy).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        let root_mount = ProjectMount {
            host_path: PathBuf::from("/"),
            container_path: "/workspace".to_string(),
            writable: true,
        };
        assert!(matches!(
            docker_create_arguments(&environment, &policy, &[root_mount]),
            Err(RuntimeIssue::ForbiddenMount(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn policy_loader_rejects_group_writable_file() {
        let path = std::env::temp_dir().join(format!(
            "bridget-policy-{}-{}.json",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let document = serde_json::json!({
            "contract_version": 1,
            "state_root_parent": "/tmp/bridget-state",
            "policies": [{
                "policy_id": "fixture-local",
                "policy_version": 1,
                "image_reference_kind": "local_image_id",
                "image_reference": format!("sha256:{}", "c".repeat(64)),
                "run_as_uid": 1002,
                "run_as_gid": 1002,
                "cpu_limit": 1.0,
                "memory_limit_bytes": 1024,
                "pids_limit": 64,
                "tmpfs": ["/tmp"],
                "network_mode": "bridge"
            }]
        });
        fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o660)).unwrap();
        assert!(matches!(
            ProjectRuntimePolicyConfig::load(&path),
            Err(RuntimeIssue::PolicyInvalid(_))
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn docker_fixture_covers_timeout_hostile_json_image_and_daemon_failure() {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docker/docker");
        let docker = DockerCli::new(fixture, std::time::Duration::from_millis(40));
        assert!(
            docker
                .invoke_json(&["version"], &[("BRIDGET_DOCKER_FIXTURE_MODE", "ok")])
                .is_ok()
        );
        assert!(matches!(
            docker.invoke_json(
                &["inspect"],
                &[("BRIDGET_DOCKER_FIXTURE_MODE", "hostile-json")]
            ),
            Err(RuntimeIssue::DockerOutputInvalid)
        ));
        assert!(matches!(
            docker.invoke_json(
                &["image", "inspect", "missing"],
                &[("BRIDGET_DOCKER_FIXTURE_MODE", "image-missing")]
            ),
            Err(RuntimeIssue::DockerCommandFailed)
        ));
        assert!(matches!(
            docker.invoke_json(
                &["version"],
                &[("BRIDGET_DOCKER_FIXTURE_MODE", "daemon-unavailable")]
            ),
            Err(RuntimeIssue::DockerCommandFailed)
        ));
        assert!(matches!(
            docker.invoke_json(
                &["version"],
                &[
                    ("BRIDGET_DOCKER_FIXTURE_MODE", "timeout"),
                    ("BRIDGET_DOCKER_FIXTURE_TIMEOUT_SECS", "1"),
                ]
            ),
            Err(RuntimeIssue::DockerTimeout)
        ));
    }
    #[test]
    fn attestation_divergente_impose_recreate_required() {
        let policy = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 1, &policy).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        let result: Result<(), RuntimeIssue> =
            fail_prepare(&mut environment, RuntimeIssue::ContainerAttestationInvalid);
        assert_eq!(result, Err(RuntimeIssue::ContainerAttestationInvalid));
        assert_eq!(environment.state, ProjectEnvironmentState::RecreateRequired);
        assert_eq!(
            environment.last_reason.as_deref(),
            Some("container_attestation_invalid")
        );
    }
    #[test]
    fn environnement_a_recreer_est_arrete_et_supprime_sans_recherche_globale() {
        let policy = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 1, &policy).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        environment
            .transition(ProjectEnvironmentState::RecreateRequired, None)
            .unwrap();
        environment.container_id = Some("a".repeat(64));

        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docker/docker");
        let docker = DockerCli::new(fixture, std::time::Duration::from_secs(1));
        stop_remove_environment(&docker, &mut environment).unwrap();

        assert_eq!(environment.state, ProjectEnvironmentState::Absent);
        assert_eq!(environment.container_id, None);
    }

    #[test]
    fn arret_et_suppression_sont_deux_transitions_locales_distinctes() {
        let policy = policy();
        let mut environment = ProjectEnvironment::absent("project-a", 1, &policy).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        environment.container_id = Some("a".repeat(64));
        environment
            .transition(ProjectEnvironmentState::Ready, None)
            .unwrap();

        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docker/docker");
        let docker = DockerCli::new(fixture, std::time::Duration::from_secs(1));

        assert!(matches!(
            remove_environment(&docker, &mut environment),
            Err(RuntimeIssue::InvalidTransition { .. })
        ));
        assert_eq!(environment.state, ProjectEnvironmentState::Ready);
        assert!(environment.container_id.is_some());

        stop_environment(&docker, &mut environment).unwrap();
        assert_eq!(environment.state, ProjectEnvironmentState::Stopped);
        assert!(environment.container_id.is_some());

        remove_environment(&docker, &mut environment).unwrap();
        assert_eq!(environment.state, ProjectEnvironmentState::Absent);
        assert_eq!(environment.container_id, None);
    }
    #[test]
    fn ingress_prive_est_deterministe_et_refuse_toute_identite_divergente() {
        let mut policy = policy();
        let state_parent = std::env::temp_dir().join(format!(
            "bridget-066-ingress-{}-{}",
            std::process::id(),
            unix_now_nanos()
        ));
        policy.state_root_parent = state_parent.clone();
        let mut environment = ProjectEnvironment::absent("project-a", 2, &policy).unwrap();
        environment
            .transition(ProjectEnvironmentState::Creating, None)
            .unwrap();
        environment.container_id = Some("a".repeat(64));
        environment
            .transition(ProjectEnvironmentState::Ready, None)
            .unwrap();

        let expectation = RuntimeIngressExpectation::from_environment(
            &environment,
            7,
            "00000000-0000-4000-8000-000000000066",
        )
        .unwrap();
        let endpoint = bind_runtime_ingress(&policy, &environment).unwrap();
        assert_eq!(
            endpoint.socket_path,
            state_parent.join("project-a/runtime/2/bridget.sock")
        );
        assert_eq!(
            fs::metadata(state_parent.join("project-a/runtime"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&endpoint.socket_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        let handshake = bridget_transport::protocol::RuntimeIngressHandshake {
            contract_version: bridget_transport::protocol::RUNTIME_INGRESS_CONTRACT_VERSION,
            project_id: "project-a".to_string(),
            binding_generation: 2,
            container_id: "a".repeat(64),
            environment_epoch: 1,
            agent_generation: 7,
            instance_id: "00000000-0000-4000-8000-000000000066".to_string(),
        };
        assert_eq!(expectation.validate(&handshake), Ok(()));

        let mut foreign_project = handshake.clone();
        foreign_project.project_id = "project-b".to_string();
        assert_eq!(
            expectation.validate(&foreign_project),
            Err(bridget_transport::protocol::RuntimeIngressRefusal::IdentityMismatch)
        );
        let mut stale_epoch = handshake;
        stale_epoch.environment_epoch = 2;
        assert_eq!(
            expectation.validate(&stale_epoch),
            Err(bridget_transport::protocol::RuntimeIngressRefusal::EnvironmentEpochStale)
        );

        drop(endpoint);
        let _ = fs::remove_dir_all(state_parent);
    }

    fn unix_now_nanos() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
    #[test]
    fn preflight_refuse_un_uid_ou_gid_incompatible_avec_le_state_root_prive() {
        let mut incompatible = policy();
        incompatible.run_as_uid = incompatible.run_as_uid.saturating_add(1);
        assert_eq!(
            assert_runtime_user_compatible(&incompatible),
            Err(RuntimeIssue::RuntimeUserIncompatible)
        );
    }
    #[test]
    fn politique_runtime_resout_uniquement_un_executable_interne_declare() {
        let policy = policy();
        assert_eq!(
            policy.runtime_execution("fixture").unwrap(),
            ProjectRuntimeExecution {
                wrapper_executable: "/usr/local/bin/bridget".to_string(),
                provider_command: "/usr/local/bin/fixture-agent".to_string(),
            }
        );
        assert_eq!(
            policy.runtime_execution("codex"),
            Err(RuntimeIssue::RuntimeExecutableUnavailable)
        );
    }

    #[test]
    fn docker_exec_runtime_transporte_uniquement_les_identites_attestees() {
        let launch = DockerRuntimeLaunch {
            reservation: RuntimeReservation {
                project_id: "project-066".to_string(),
                binding_generation: 2,
                environment_epoch: 3,
            },
            container_id: "a".repeat(64),
            run_as_uid: 1002,
            run_as_gid: 1002,
            execution: ProjectRuntimeExecution {
                wrapper_executable: "/usr/local/bin/bridget".to_string(),
                provider_command: "/usr/local/bin/fixture-agent".to_string(),
            },
            exec_id: uuid::Uuid::new_v4().to_string(),
            agent_type: "fixture".to_string(),
            agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            instance_id: uuid::Uuid::new_v4().to_string(),
            agent_generation: 7,
            cwd: PathBuf::from("/workspace/project-066"),
            resolved_definition_json: r#"{"command":"/not-from-host","digest":"frozen"}"#
                .to_string(),
            process_env: vec![ProcessEnvBinding {
                variable: "PROJECT_TOKEN".to_string(),
                container_file: format!("{CONTAINER_PROCESS_ENV_DIRECTORY}/{}", "a".repeat(64)),
            }],
        };

        let arguments = launch.docker_exec_arguments().unwrap();
        assert_eq!(arguments.first().map(String::as_str), Some("exec"));
        assert!(
            arguments
                .windows(2)
                .any(|pair| { pair == ["--env", "BRIDGET_RUNTIME_PROJECT_ID=project-066"] })
        );
        assert!(
            arguments
                .windows(2)
                .any(|pair| { pair == ["--env", "BRIDGET_RUNTIME_ENVIRONMENT_EPOCH=3"] })
        );
        assert!(
            arguments
                .windows(2)
                .any(|pair| { pair == ["--env", "BRIDGET_RUNTIME_AGENT_GENERATION=7"] })
        );
        assert!(arguments.iter().all(|argument| argument != "sh"));
        assert!(
            arguments
                .iter()
                .any(|argument| argument.contains("PROJECT_TOKEN"))
        );
        assert!(
            arguments
                .iter()
                .all(|argument| !argument.contains("S067_SYNTHETIC_SECRET"))
        );
        assert_eq!(arguments[arguments.len() - 5], "managed-runtime-wrapper");
        assert_eq!(
            arguments[arguments.len() - 3],
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            arguments[arguments.len() - 2],
            "/usr/local/bin/fixture-agent"
        );

        let mut invalid = launch;
        invalid.container_id = "not-a-container".to_string();
        assert!(matches!(
            invalid.docker_exec_arguments(),
            Err(RuntimeIssue::RuntimeLaunchInvalid(_))
        ));
    }

    #[test]
    fn spec_066_raisons_runtime_oom_pid_exec_et_redemarrage_sont_fermees() {
        let oom = serde_json::json!({"State": {"OOMKilled": true}});
        assert_eq!(classify_runtime_failure(Some(&oom), true), "resource_oom");
        let pid_limit = serde_json::json!({"State": {"Error": "pids limit reached"}});
        assert_eq!(
            classify_runtime_failure(Some(&pid_limit), true),
            "resource_pid_limit"
        );
        assert_eq!(classify_runtime_failure(None, true), "runtime_exec_lost");
        assert_eq!(classify_runtime_failure(None, false), "docker_restarted");
    }
}
