//! Compatibilité du stockage des anciennes liaisons projet.
//! Aucun moteur, lancement ou réinterprétation Docker→hôte : les anciennes
//! lignes et migrations restent relisibles, hors du chemin d'admission.
use super::{Store, StoreError};
use crate::project_compat::{
    DogfoodingBridgetMode, DogfoodingBridgetState, ProjectEnvironmentState,
};
use bridget_transport::protocol::{
    PROJECT_REGISTRY_CONTRACT_VERSION, PROJECT_ROUND_POLICY_CONTRACT_VERSION,
    ProjectAdminOperation, ProjectAdminOutcome, ProjectAuditOperationKind, ProjectAuditOutcomeKind,
    ProjectAuditProjection, ProjectBackend, ProjectBindOutcome, ProjectBindStatus,
    ProjectBindingProjection, ProjectBindingStatus, ProjectRegistryRefusal, ProjectRole,
    ProjectRoundDispatchState, ProjectRoundOperation, ProjectRoundOutcome, ProjectRoundProjection,
    ProjectRoundRefusal, ProjectRuntimePolicyReference,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::path::Path;

/// État technique d'une liaison projet, détenu par Bridget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectBindingState {
    PendingBinding,
    Active,
    Disabled,
    PathMissing,
    BindingFailed,
}

impl ProjectBindingState {
    fn as_db(self) -> &'static str {
        match self {
            Self::PendingBinding => "pending_binding",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::PathMissing => "path_missing",
            Self::BindingFailed => "binding_failed",
        }
    }

    fn from_db(value: &str) -> Result<Self, StoreError> {
        match value {
            "pending_binding" => Ok(Self::PendingBinding),
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            "path_missing" => Ok(Self::PathMissing),
            "binding_failed" => Ok(Self::BindingFailed),
            _ => Err(StoreError::Invariant("état de liaison projet inconnu")),
        }
    }
}

/// Liaison technique entre une identité opaque et une racine canonique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBinding {
    pub project_id: String,
    pub canonical_root: String,
    pub backend: ProjectBackend,
    pub state: ProjectBindingState,
    pub generation: u64,
    pub bound_at: i64,
    pub updated_at: i64,
    pub last_reason: Option<ProjectRegistryRefusal>,
    pub runtime: Option<ProjectRuntimeBinding>,
}

impl ProjectBinding {
    pub fn active(
        project_id: String,
        canonical_root: String,
        backend: ProjectBackend,
        observed_at: i64,
    ) -> Result<Self, StoreError> {
        let binding = Self {
            project_id,
            canonical_root,
            backend,
            state: ProjectBindingState::Active,
            generation: 1,
            bound_at: observed_at,
            updated_at: observed_at,
            last_reason: None,
            runtime: None,
        };
        binding.validate()?;
        Ok(binding)
    }

    pub fn docker(
        project_id: String,
        canonical_root: String,
        runtime: ProjectRuntimeBinding,
        observed_at: i64,
    ) -> Result<Self, StoreError> {
        let binding = Self {
            project_id,
            canonical_root,
            backend: ProjectBackend::Docker,
            state: ProjectBindingState::Active,
            generation: 1,
            bound_at: observed_at,
            updated_at: observed_at,
            last_reason: None,
            runtime: Some(runtime),
        };
        binding.validate()?;
        Ok(binding)
    }

    fn validate(&self) -> Result<(), StoreError> {
        if self.project_id.trim().is_empty()
            || self.canonical_root.trim().is_empty()
            || !Path::new(&self.canonical_root).is_absolute()
            || self.generation == 0
            || self.bound_at < 0
            || self.updated_at < self.bound_at
        {
            return Err(StoreError::Invariant("liaison projet invalide"));
        }
        match (self.backend, self.runtime.as_ref()) {
            (ProjectBackend::Host, None) => {}
            (ProjectBackend::Docker, Some(runtime)) => runtime.validate()?,
            _ => {
                return Err(StoreError::Invariant(
                    "backend et runtime projet incohérents",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRuntimeBinding {
    pub state: ProjectEnvironmentState,
    pub policy_id: String,
    pub policy_version: u64,
    pub policy_digest: String,
    pub image_reference: String,
    pub resolved_image_id: Option<String>,
    pub run_as_uid: u32,
    pub run_as_gid: u32,
    pub environment_epoch: u64,
    pub topology_digest: String,
    pub container_id: Option<String>,
    pub last_reason: Option<String>,
}

impl ProjectRuntimeBinding {
    pub fn policy_reference(&self) -> ProjectRuntimePolicyReference {
        ProjectRuntimePolicyReference {
            policy_id: self.policy_id.clone(),
            policy_version: self.policy_version,
            policy_digest: self.policy_digest.clone(),
            environment_epoch: self.environment_epoch,
        }
    }

    fn validate(&self) -> Result<(), StoreError> {
        if self.policy_id.trim().is_empty()
            || self.policy_version == 0
            || !self.policy_digest.starts_with("sha256:")
            || self.image_reference.trim().is_empty()
            || self.run_as_uid == 0
            || self.run_as_gid == 0
            || self.environment_epoch == 0
            || !self.topology_digest.starts_with("sha256:")
        {
            return Err(StoreError::Invariant("runtime projet docker invalide"));
        }
        Ok(())
    }
}

/// Mutation auditée d'une liaison technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAuditOperation {
    Register,
    Rebind,
    Activate,
    Disable,
    ReviewProjectReconcile,
}

impl ProjectAuditOperation {
    fn as_db(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Rebind => "rebind",
            Self::Activate => "activate",
            Self::Disable => "disable",
            Self::ReviewProjectReconcile => "review_project_reconcile",
        }
    }

    fn from_db(value: &str) -> Result<Self, StoreError> {
        match value {
            "register" => Ok(Self::Register),
            "rebind" => Ok(Self::Rebind),
            "activate" => Ok(Self::Activate),
            "disable" => Ok(Self::Disable),
            "review_project_reconcile" => Ok(Self::ReviewProjectReconcile),
            _ => Err(StoreError::Invariant("opération audit projet inconnue")),
        }
    }
}

/// Issue fermée d'un événement d'audit, sans contenu de racine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectAuditOutcome {
    Applied,
    Refused { reason: ProjectRegistryRefusal },
}

/// Trace transactionnelle d'une mutation réelle de liaison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAuditEvent {
    pub audit_event_id: String,
    pub command_id: String,
    pub project_id: String,
    pub operation: ProjectAuditOperation,
    pub binding_generation: u64,
    pub outcome: ProjectAuditOutcome,
    pub previous_root_reference: Option<String>,
    pub observed_at: i64,
}

impl ProjectAuditEvent {
    pub fn for_mutation(
        command_id: &str,
        project_id: &str,
        operation: ProjectAuditOperation,
        binding_generation: u64,
        outcome: ProjectAuditOutcome,
        previous_root: Option<&str>,
        observed_at: i64,
    ) -> Self {
        Self {
            audit_event_id: project_audit_event_id(command_id, operation, binding_generation),
            command_id: command_id.to_string(),
            project_id: project_id.to_string(),
            operation,
            binding_generation,
            outcome,
            previous_root_reference: previous_root.map(project_root_reference),
            observed_at,
        }
    }

    fn validate(&self) -> Result<(), StoreError> {
        if self.audit_event_id.trim().is_empty()
            || self.command_id.trim().is_empty()
            || self.project_id.trim().is_empty()
            || self.binding_generation == 0
            || self.observed_at < 0
            || self
                .previous_root_reference
                .as_deref()
                .is_some_and(|reference| !is_sha256_reference(reference))
        {
            return Err(StoreError::Invariant("événement audit projet invalide"));
        }
        if self.audit_event_id
            != project_audit_event_id(&self.command_id, self.operation, self.binding_generation)
        {
            return Err(StoreError::Invariant(
                "identifiant audit projet non déterministe",
            ));
        }
        Ok(())
    }
}

impl Store {
    /// SQLite ne sait pas étendre la contrainte CHECK d'une table. La migration
    /// reconstruit donc l'audit dans une transaction sans modifier les événements
    /// existants ni leur ordre. Elle est idempotente pour les bases neuves.
    pub(super) fn ensure_project_audit_schema(conn: &Connection) -> Result<(), StoreError> {
        let schema: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'project_audit_events'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        if schema
            .as_deref()
            .is_some_and(|definition| definition.contains("'activate'"))
        {
            return Ok(());
        }
        conn.execute_batch(
            "BEGIN IMMEDIATE;
             CREATE TABLE project_audit_events_next (
                 audit_event_id TEXT PRIMARY KEY,
                 command_id TEXT NOT NULL,
                 project_id TEXT NOT NULL,
                 operation TEXT NOT NULL CHECK (operation IN (
                     'register', 'rebind', 'activate', 'disable', 'review_project_reconcile'
                 )),
                 binding_generation INTEGER NOT NULL CHECK (binding_generation > 0),
                 outcome_json BLOB NOT NULL,
                 previous_root_reference TEXT,
                 observed_at INTEGER NOT NULL,
                 UNIQUE (command_id, operation, binding_generation)
             );
             INSERT INTO project_audit_events_next (
                 audit_event_id, command_id, project_id, operation, binding_generation,
                 outcome_json, previous_root_reference, observed_at
             )
             SELECT audit_event_id, command_id, project_id, operation, binding_generation,
                    outcome_json, previous_root_reference, observed_at
             FROM project_audit_events;
             DROP TABLE project_audit_events;
             ALTER TABLE project_audit_events_next RENAME TO project_audit_events;
             CREATE INDEX idx_project_audit_events_project_observed
                 ON project_audit_events(project_id, observed_at, audit_event_id);
             COMMIT;",
        )
        .map_err(StoreError::Sqlite)
    }

    /// Insère une liaison déjà validée par la frontière daemon.
    pub fn insert_project_binding(&mut self, binding: &ProjectBinding) -> Result<(), StoreError> {
        binding.validate()?;
        let runtime = binding.runtime.as_ref();
        let policy_version = runtime
            .map(|runtime| i64::try_from(runtime.policy_version))
            .transpose()
            .map_err(|_| StoreError::Invariant("version policy runtime invalide"))?;
        let environment_epoch = runtime
            .map(|runtime| i64::try_from(runtime.environment_epoch))
            .transpose()
            .map_err(|_| StoreError::Invariant("epoch runtime invalide"))?
            .unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO project_bindings (
                     project_id, canonical_root, backend, state, generation,
                     bound_at, updated_at, last_reason, runtime_state, runtime_last_reason, policy_id,
                     policy_version, policy_digest, image_reference, resolved_image_id,
                     run_as_uid, run_as_gid, environment_epoch, topology_digest, container_id
                 ) VALUES (
                     ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
                 )",
                params![
                    &binding.project_id,
                    &binding.canonical_root,
                    match binding.backend {
                        ProjectBackend::Host => "host",
                        ProjectBackend::Docker => "docker",
                    },
                    binding.state.as_db(),
                    binding.generation as i64,
                    binding.bound_at,
                    binding.updated_at,
                    binding.last_reason.map(project_registry_refusal_name),
                    runtime.map(|runtime| project_environment_state_name(runtime.state)),
                    runtime.and_then(|runtime| runtime.last_reason.as_deref()),
                    runtime.map(|runtime| runtime.policy_id.as_str()),
                    policy_version,
                    runtime.map(|runtime| runtime.policy_digest.as_str()),
                    runtime.map(|runtime| runtime.image_reference.as_str()),
                    runtime.and_then(|runtime| runtime.resolved_image_id.as_deref()),
                    runtime.map(|runtime| i64::from(runtime.run_as_uid)),
                    runtime.map(|runtime| i64::from(runtime.run_as_gid)),
                    environment_epoch,
                    runtime
                        .map(|runtime| runtime.topology_digest.as_str())
                        .unwrap_or("sha256:legacy"),
                    runtime.and_then(|runtime| runtime.container_id.as_deref()),
                ],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Admet une liaison initiale dans une transaction unique avec sa trace
    /// d'audit. Le résultat est retenu par `command_id` afin qu'un retry après
    /// crash retrouve exactement l'issue déjà rendue.
    pub fn bind_project_registration(
        &mut self,
        command_id: &str,
        project_id: &str,
        canonical_root: &str,
        observed_at: i64,
    ) -> Result<ProjectBindOutcome, StoreError> {
        if command_id.trim().is_empty()
            || project_id.trim().is_empty()
            || canonical_root.trim().is_empty()
            || !Path::new(canonical_root).is_absolute()
            || observed_at < 0
        {
            return Err(StoreError::Invariant("demande de liaison projet invalide"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        if let Some(existing) = project_binding_attempt_for_command(&tx, command_id)? {
            let outcome: ProjectBindOutcome = serde_json::from_slice(&existing.outcome_json)
                .map_err(|_| StoreError::Invariant("issue liaison projet corrompue"))?;
            if existing.project_id != project_id
                || existing.canonical_root != canonical_root
                || outcome.backend != Some(ProjectBackend::Host)
                || outcome.runtime_policy.is_some()
            {
                return Err(StoreError::ProjectRegistryRefusal(
                    ProjectRegistryRefusal::EnvelopeMismatch,
                ));
            }
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(outcome);
        }

        let outcome = if let Some(existing) = project_binding_for_project(&tx, project_id)? {
            if existing.state == ProjectBindingState::Active
                && existing.canonical_root == canonical_root
            {
                project_binding_active_outcome(
                    command_id,
                    project_id,
                    existing.generation,
                    observed_at,
                )
            } else {
                project_binding_failed_outcome(
                    command_id,
                    project_id,
                    if existing.state == ProjectBindingState::Disabled {
                        ProjectRegistryRefusal::ProjectDisabled
                    } else {
                        ProjectRegistryRefusal::RebindRequired
                    },
                    observed_at,
                )
            }
        } else if let Some(existing) = project_binding_for_root(&tx, canonical_root)? {
            ProjectBindOutcome {
                contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
                command_id: command_id.to_string(),
                project_id: project_id.to_string(),
                status: ProjectBindStatus::RegistrationConflict,
                binding_generation: None,
                backend: None,
                runtime_policy: None,
                reason: Some(ProjectRegistryRefusal::RootAlreadyBound),
                existing_project_id: Some(existing.project_id),
                existing_binding_generation: Some(existing.generation),
                observed_at,
            }
        } else {
            let binding = ProjectBinding::active(
                project_id.to_string(),
                canonical_root.to_string(),
                ProjectBackend::Host,
                observed_at,
            )?;
            tx.execute(
                "INSERT INTO project_bindings (
                     project_id, canonical_root, backend, state, generation,
                     bound_at, updated_at, last_reason
                 ) VALUES (?1, ?2, 'host', 'active', 1, ?3, ?3, NULL)",
                params![project_id, canonical_root, observed_at],
            )
            .map_err(StoreError::Sqlite)?;
            let audit = ProjectAuditEvent::for_mutation(
                command_id,
                project_id,
                ProjectAuditOperation::Register,
                binding.generation,
                ProjectAuditOutcome::Applied,
                None,
                observed_at,
            );
            record_project_audit_event_in_transaction(&tx, &audit)?;
            project_binding_active_outcome(command_id, project_id, binding.generation, observed_at)
        };
        let outcome_json = serde_json::to_vec(&outcome)
            .map_err(|_| StoreError::Invariant("issue liaison projet non sérialisable"))?;
        tx.execute(
            "INSERT INTO project_binding_attempts (
                 command_id, project_id, canonical_root, outcome_json, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                command_id,
                project_id,
                canonical_root,
                outcome_json,
                observed_at
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(outcome)
    }

    pub fn bind_project_docker_registration(
        &mut self,
        command_id: &str,
        project_id: &str,
        canonical_root: &str,
        runtime: ProjectRuntimeBinding,
        observed_at: i64,
    ) -> Result<ProjectBindOutcome, StoreError> {
        if command_id.trim().is_empty()
            || project_id.trim().is_empty()
            || canonical_root.trim().is_empty()
            || !Path::new(canonical_root).is_absolute()
            || observed_at < 0
        {
            return Err(StoreError::Invariant("demande Docker projet invalide"));
        }
        runtime.validate()?;
        let expected_runtime = runtime.policy_reference();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        if let Some(existing) = project_binding_attempt_for_command(&tx, command_id)? {
            let outcome: ProjectBindOutcome = serde_json::from_slice(&existing.outcome_json)
                .map_err(|_| StoreError::Invariant("issue liaison projet corrompue"))?;
            if existing.project_id != project_id
                || existing.canonical_root != canonical_root
                || outcome.backend != Some(ProjectBackend::Docker)
                || outcome.runtime_policy.as_ref() != Some(&expected_runtime)
            {
                return Err(StoreError::ProjectRegistryRefusal(
                    ProjectRegistryRefusal::EnvelopeMismatch,
                ));
            }
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(outcome);
        }

        let outcome = if let Some(existing) = project_binding_for_project(&tx, project_id)? {
            if existing.backend == ProjectBackend::Docker
                && existing.state == ProjectBindingState::Active
                && existing.canonical_root == canonical_root
                && existing.runtime.as_ref() == Some(&runtime)
            {
                project_binding_active_outcome_for_binding(command_id, &existing, observed_at)
            } else {
                project_binding_failed_outcome(
                    command_id,
                    project_id,
                    if existing.state == ProjectBindingState::Disabled {
                        ProjectRegistryRefusal::ProjectDisabled
                    } else {
                        ProjectRegistryRefusal::RebindRequired
                    },
                    observed_at,
                )
            }
        } else if let Some(existing) = project_binding_for_root(&tx, canonical_root)? {
            ProjectBindOutcome {
                contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
                command_id: command_id.to_string(),
                project_id: project_id.to_string(),
                status: ProjectBindStatus::RegistrationConflict,
                binding_generation: None,
                backend: None,
                runtime_policy: None,
                reason: Some(ProjectRegistryRefusal::RootAlreadyBound),
                existing_project_id: Some(existing.project_id),
                existing_binding_generation: Some(existing.generation),
                observed_at,
            }
        } else {
            let binding = ProjectBinding::docker(
                project_id.to_string(),
                canonical_root.to_string(),
                runtime,
                observed_at,
            )?;
            let runtime = binding
                .runtime
                .as_ref()
                .ok_or(StoreError::Invariant("runtime Docker absent"))?;
            tx.execute(
                "INSERT INTO project_bindings (
                     project_id, canonical_root, backend, state, generation,
                     bound_at, updated_at, last_reason, runtime_state, runtime_last_reason, policy_id,
                     policy_version, policy_digest, image_reference, resolved_image_id,
                     run_as_uid, run_as_gid, environment_epoch, topology_digest, container_id
                 ) VALUES (
                     ?1, ?2, 'docker', ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                     ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19
                 )",
                params![
                    &binding.project_id,
                    &binding.canonical_root,
                    binding.state.as_db(),
                    binding.generation as i64,
                    binding.bound_at,
                    binding.updated_at,
                    binding.last_reason.map(project_registry_refusal_name),
                    project_environment_state_name(runtime.state),
                    runtime.last_reason.as_deref(),
                    &runtime.policy_id,
                    i64::try_from(runtime.policy_version)
                        .map_err(|_| StoreError::Invariant("version policy runtime invalide"))?,
                    &runtime.policy_digest,
                    &runtime.image_reference,
                    runtime.resolved_image_id.as_deref(),
                    i64::from(runtime.run_as_uid),
                    i64::from(runtime.run_as_gid),
                    i64::try_from(runtime.environment_epoch)
                        .map_err(|_| StoreError::Invariant("epoch runtime invalide"))?,
                    &runtime.topology_digest,
                    runtime.container_id.as_deref(),
                ],
            )
            .map_err(StoreError::Sqlite)?;
            let audit = ProjectAuditEvent::for_mutation(
                command_id,
                project_id,
                ProjectAuditOperation::Register,
                binding.generation,
                ProjectAuditOutcome::Applied,
                None,
                observed_at,
            );
            record_project_audit_event_in_transaction(&tx, &audit)?;
            project_binding_active_outcome_for_binding(command_id, &binding, observed_at)
        };
        let outcome_json = serde_json::to_vec(&outcome)
            .map_err(|_| StoreError::Invariant("issue liaison Docker non sérialisable"))?;
        tx.execute(
            "INSERT INTO project_binding_attempts (
                 command_id, project_id, canonical_root, outcome_json, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                command_id,
                project_id,
                canonical_root,
                outcome_json,
                observed_at
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(outcome)
    }

    pub fn update_project_runtime(
        &mut self,
        project_id: &str,
        runtime: &ProjectRuntimeBinding,
        observed_at: i64,
    ) -> Result<ProjectBinding, StoreError> {
        runtime.validate()?;
        let transaction = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let existing = project_binding_for_project(&transaction, project_id)?
            .ok_or(StoreError::Invariant("liaison projet absente"))?;
        if existing.backend != ProjectBackend::Docker {
            return Err(StoreError::Invariant("runtime Docker sur liaison host"));
        }
        let previous = existing
            .runtime
            .as_ref()
            .ok_or(StoreError::Invariant("runtime Docker absent"))?;
        if runtime.environment_epoch < previous.environment_epoch
            || observed_at < existing.updated_at
        {
            return Err(StoreError::Invariant("mise a jour runtime obsolete"));
        }
        let policy_version = i64::try_from(runtime.policy_version)
            .map_err(|_| StoreError::Invariant("version policy runtime invalide"))?;
        let environment_epoch = i64::try_from(runtime.environment_epoch)
            .map_err(|_| StoreError::Invariant("epoch runtime invalide"))?;
        transaction
            .execute(
                "UPDATE project_bindings
                 SET runtime_state = ?1, runtime_last_reason = ?2, policy_id = ?3, policy_version = ?4,
                     policy_digest = ?5, image_reference = ?6, resolved_image_id = ?7,
                     run_as_uid = ?8, run_as_gid = ?9, environment_epoch = ?10,
                     topology_digest = ?11, container_id = ?12, updated_at = ?13
                 WHERE project_id = ?14",
                params![
                    project_environment_state_name(runtime.state),
                    runtime.last_reason.as_deref(),
                    &runtime.policy_id,
                    policy_version,
                    &runtime.policy_digest,
                    &runtime.image_reference,
                    runtime.resolved_image_id.as_deref(),
                    i64::from(runtime.run_as_uid),
                    i64::from(runtime.run_as_gid),
                    environment_epoch,
                    &runtime.topology_digest,
                    runtime.container_id.as_deref(),
                    observed_at,
                    project_id,
                ],
            )
            .map_err(StoreError::Sqlite)?;
        let updated = project_binding_for_project(&transaction, project_id)?
            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
        transaction.commit().map_err(StoreError::Sqlite)?;
        Ok(updated)
    }

    /// Conserve une cause runtime déjà classée par le daemon, sans détail
    /// Docker brut, et la rend disponible à la projection UI.
    pub fn record_project_runtime_failure(
        &mut self,
        project_id: &str,
        reason: &'static str,
        observed_at: i64,
    ) -> Result<ProjectBinding, StoreError> {
        let binding = self
            .project_binding(project_id)?
            .ok_or(StoreError::Invariant("liaison projet absente"))?;
        let mut runtime = binding
            .runtime
            .ok_or(StoreError::Invariant("runtime Docker absent"))?;
        runtime.last_reason = Some(reason.to_string());
        self.update_project_runtime(project_id, &runtime, observed_at)
    }

    /// Bascule explicitement une liaison Docker vers le backend hôte après que
    /// le lifecycle a supprimé le conteneur. La génération change afin que toute
    /// réservation Docker antérieure soit définitivement périmée.
    pub fn switch_project_backend_to_host(
        &mut self,
        project_id: &str,
        observed_at: i64,
    ) -> Result<ProjectBinding, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let binding = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet absente"))?;
        if observed_at < binding.updated_at {
            return Err(StoreError::Invariant("bascule backend obsolète"));
        }
        if binding.backend == ProjectBackend::Host {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(binding);
        }
        if binding.state != ProjectBindingState::Active {
            return Err(StoreError::Invariant("liaison Docker non active"));
        }
        let generation = binding
            .generation
            .checked_add(1)
            .ok_or(StoreError::Invariant("génération liaison épuisée"))?;
        tx.execute(
            "UPDATE project_bindings
             SET backend = 'host', state = 'active', generation = ?1, bound_at = ?2,
                 updated_at = ?2, last_reason = NULL, runtime_state = NULL,
                 policy_id = NULL, policy_version = NULL, policy_digest = NULL,
                 image_reference = NULL, resolved_image_id = NULL, run_as_uid = NULL,
                 run_as_gid = NULL, environment_epoch = 0, topology_digest = 'sha256:legacy', container_id = NULL
             WHERE project_id = ?3",
            params![generation as i64, observed_at, project_id],
        )
        .map_err(StoreError::Sqlite)?;
        let switched = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(switched)
    }

    /// Publie une liaison Docker déjà préparée et attestée. La préparation du
    /// conteneur se fait hors transaction ; cette méthode est le seul point où
    /// le projet quitte Host. Une erreur laisse donc la liaison Host intacte.
    pub fn activate_project_docker_binding(
        &mut self,
        command_id: &str,
        project_id: &str,
        expected_generation: u64,
        runtime: ProjectRuntimeBinding,
        observed_at: i64,
    ) -> Result<ProjectBinding, StoreError> {
        if command_id.trim().is_empty()
            || project_id.trim().is_empty()
            || expected_generation == 0
            || observed_at < 0
        {
            return Err(StoreError::Invariant("activation Docker projet invalide"));
        }
        runtime.validate()?;
        let expected_runtime = runtime.policy_reference();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        if let Some(existing) = project_binding_attempt_for_command(&tx, command_id)? {
            let outcome: ProjectBindOutcome = serde_json::from_slice(&existing.outcome_json)
                .map_err(|_| StoreError::Invariant("issue activation Docker corrompue"))?;
            if existing.project_id != project_id
                || outcome.backend != Some(ProjectBackend::Docker)
                || outcome.binding_generation != expected_generation.checked_add(1)
                || outcome.runtime_policy.as_ref() != Some(&expected_runtime)
            {
                return Err(StoreError::ProjectRegistryRefusal(
                    ProjectRegistryRefusal::EnvelopeMismatch,
                ));
            }
            let binding = project_binding_for_project(&tx, project_id)?
                .ok_or(StoreError::Invariant("liaison projet absente"))?;
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(binding);
        }
        let existing = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet absente"))?;
        if existing.backend != ProjectBackend::Host
            || existing.state != ProjectBindingState::Active
            || existing.generation != expected_generation
        {
            return Err(StoreError::ProjectRegistryRefusal(
                ProjectRegistryRefusal::RebindRequired,
            ));
        }
        let generation = expected_generation
            .checked_add(1)
            .ok_or(StoreError::Invariant("génération liaison épuisée"))?;
        tx.execute(
            "UPDATE project_bindings
             SET backend = 'docker', state = 'active', generation = ?1, updated_at = ?2,
                 last_reason = NULL, runtime_state = ?3, runtime_last_reason = ?4,
                 policy_id = ?5, policy_version = ?6, policy_digest = ?7,
                 image_reference = ?8, resolved_image_id = ?9, run_as_uid = ?10,
                 run_as_gid = ?11, environment_epoch = ?12, topology_digest = ?13,
                 container_id = ?14
             WHERE project_id = ?15",
            params![
                generation as i64,
                observed_at,
                project_environment_state_name(runtime.state),
                runtime.last_reason.as_deref(),
                &runtime.policy_id,
                i64::try_from(runtime.policy_version)
                    .map_err(|_| StoreError::Invariant("version policy runtime invalide"))?,
                &runtime.policy_digest,
                &runtime.image_reference,
                runtime.resolved_image_id.as_deref(),
                i64::from(runtime.run_as_uid),
                i64::from(runtime.run_as_gid),
                i64::try_from(runtime.environment_epoch)
                    .map_err(|_| StoreError::Invariant("epoch runtime invalide"))?,
                &runtime.topology_digest,
                runtime.container_id.as_deref(),
                project_id,
            ],
        )
        .map_err(StoreError::Sqlite)?;
        let activated = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
        let audit = ProjectAuditEvent::for_mutation(
            command_id,
            project_id,
            ProjectAuditOperation::Activate,
            generation,
            ProjectAuditOutcome::Applied,
            None,
            observed_at,
        );
        record_project_audit_event_in_transaction(&tx, &audit)?;
        let outcome =
            project_binding_active_outcome_for_binding(command_id, &activated, observed_at);
        let outcome_json = serde_json::to_vec(&outcome)
            .map_err(|_| StoreError::Invariant("issue activation Docker non sérialisable"))?;
        tx.execute(
            "INSERT INTO project_binding_attempts (
                 command_id, project_id, canonical_root, outcome_json, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                command_id,
                project_id,
                &activated.canonical_root,
                outcome_json,
                observed_at,
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(activated)
    }

    /// Déclare l'unique projet système Bridget. Cette transition ne modifie ni
    /// le backend ni la racine de la liaison : la politique expert distincte
    /// reste nécessaire avant tout montage écrivable.
    pub fn declare_bridget_system_project(
        &mut self,
        project_id: &str,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if project_id.trim().is_empty() || observed_at < 0 {
            return Err(StoreError::Invariant("projet système invalide"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let binding = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet système absente"))?;
        if binding.state != ProjectBindingState::Active {
            return Err(StoreError::Invariant("liaison projet système non active"));
        }
        let existing = tx
            .query_row(
                "SELECT project_id FROM project_system_roles WHERE singleton = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        match existing {
            Some(existing) if existing == project_id => {}
            Some(_) => return Err(StoreError::Invariant("projet système déjà déclaré")),
            None => {
                tx.execute(
                    "INSERT INTO project_system_roles (singleton, project_id, declared_at)
                     VALUES (1, ?1, ?2)",
                    params![project_id, observed_at],
                )
                .map_err(StoreError::Sqlite)?;
            }
        }
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(())
    }

    pub fn project_role(&self, project_id: &str) -> Result<ProjectRole, StoreError> {
        project_role_for_project(&self.conn, project_id)
    }

    /// L'absence de ligne est volontairement projetée en `disabled` : une
    /// installation antérieure ne reçoit jamais des mounts inscriptibles lors
    /// de sa première migration.
    pub fn dogfooding_bridget_state(
        &self,
        project_id: &str,
        binding_generation: u64,
    ) -> Result<DogfoodingBridgetState, StoreError> {
        if project_role_for_project(&self.conn, project_id)? != ProjectRole::BridgetSystem {
            return Err(StoreError::Invariant("réglage réservé au projet système"));
        }
        let persisted = self
            .conn
            .query_row(
                "SELECT project_id, binding_generation, setting_generation, mode
                 FROM project_system_dogfooding WHERE singleton = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        match persisted {
            None => Ok(DogfoodingBridgetState {
                project_id: project_id.to_string(),
                binding_generation,
                setting_generation: 1,
                mode: DogfoodingBridgetMode::Disabled,
            }),
            Some((stored_project, stored_binding, setting_generation, mode))
                if stored_project == project_id
                    && stored_binding >= 1
                    && setting_generation >= 1 =>
            {
                let mode = match mode.as_str() {
                    "disabled" => DogfoodingBridgetMode::Disabled,
                    "enabled" => DogfoodingBridgetMode::Enabled,
                    _ => return Err(StoreError::Invariant("mode dogfooding inconnu")),
                };
                Ok(DogfoodingBridgetState {
                    project_id: stored_project,
                    binding_generation: stored_binding as u64,
                    setting_generation: setting_generation as u64,
                    mode,
                })
            }
            Some(_) => Err(StoreError::Invariant("réglage dogfooding incohérent")),
        }
    }

    /// Persiste uniquement une transition déjà attestée par le daemon. La
    /// comparaison de générations maintient l'apply idempotent et empêche une
    /// publication tardive après une recréation concurrente.
    pub fn apply_dogfooding_bridget_state(
        &mut self,
        previous: &DogfoodingBridgetState,
        next: &DogfoodingBridgetState,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if observed_at < 0
            || previous.project_id != next.project_id
            || previous.binding_generation != next.binding_generation
            || next.setting_generation < previous.setting_generation
        {
            return Err(StoreError::Invariant("transition dogfooding invalide"));
        }
        let current =
            self.dogfooding_bridget_state(&previous.project_id, previous.binding_generation)?;
        if current != *previous {
            return Err(StoreError::Invariant("transition dogfooding obsolète"));
        }
        let mode = match next.mode {
            DogfoodingBridgetMode::Disabled => "disabled",
            DogfoodingBridgetMode::Enabled => "enabled",
        };
        self.conn
            .execute(
                "INSERT INTO project_system_dogfooding
                 (singleton, project_id, binding_generation, setting_generation, mode, updated_at)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(singleton) DO UPDATE SET
                   project_id = excluded.project_id,
                   binding_generation = excluded.binding_generation,
                   setting_generation = excluded.setting_generation,
                   mode = excluded.mode,
                   updated_at = excluded.updated_at",
                params![
                    next.project_id,
                    next.binding_generation as i64,
                    next.setting_generation as i64,
                    mode,
                    observed_at
                ],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Réserve un linked worktree déjà attesté par le daemon. La persistance
    /// ne crée jamais de worktree et ne permet jamais le checkout principal.
    pub fn acquire_system_worktree_lease(
        &mut self,
        project_id: &str,
        agent_id: &str,
        canonical_worktree: &str,
        binding_generation: u64,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if project_id.trim().is_empty()
            || agent_id.trim().is_empty()
            || !Path::new(canonical_worktree).is_absolute()
            || binding_generation == 0
            || observed_at < 0
        {
            return Err(StoreError::Invariant("lease worktree invalide"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        if project_role_for_project(&tx, project_id)? != ProjectRole::BridgetSystem {
            return Err(StoreError::Invariant("lease reservee au projet système"));
        }
        let binding = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet système absente"))?;
        if binding.generation != binding_generation
            || binding.canonical_root == canonical_worktree
            || binding.state != ProjectBindingState::Active
        {
            return Err(StoreError::Invariant(
                "worktree ou génération non admissible",
            ));
        }
        let existing = tx
            .query_row(
                "SELECT project_id, agent_id, binding_generation
                 FROM project_system_worktree_leases WHERE canonical_worktree = ?1",
                [canonical_worktree],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        match existing {
            Some((existing_project, existing_agent, existing_generation))
                if existing_project == project_id
                    && existing_agent == agent_id
                    && existing_generation == binding_generation as i64 => {}
            Some(_) => return Err(StoreError::Invariant("worktree déjà attribué")),
            None => {
                tx.execute(
                    "INSERT INTO project_system_worktree_leases
                     (canonical_worktree, project_id, agent_id, binding_generation, acquired_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        canonical_worktree,
                        project_id,
                        agent_id,
                        binding_generation as i64,
                        observed_at
                    ],
                )
                .map_err(StoreError::Sqlite)?;
            }
        }
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(())
    }

    pub fn release_system_worktree_lease(
        &mut self,
        project_id: &str,
        agent_id: &str,
        canonical_worktree: &str,
    ) -> Result<bool, StoreError> {
        let changed = self
            .conn
            .execute(
                "DELETE FROM project_system_worktree_leases
                 WHERE canonical_worktree = ?1 AND project_id = ?2 AND agent_id = ?3",
                params![canonical_worktree, project_id, agent_id],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }

    /// Liste bornée des worktrees actuellement attribués au projet système.
    /// Le daemon l'utilise pour calculer une topologie de mounts complète avant
    /// une recréation : il n'infère jamais un worktree depuis un chemin agent.
    pub fn system_worktree_leases(&self, project_id: &str) -> Result<Vec<String>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT canonical_worktree FROM project_system_worktree_leases
                 WHERE project_id = ?1 ORDER BY canonical_worktree ASC LIMIT 64",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map([project_id], |row| row.get::<_, String>(0))
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)
    }

    pub fn project_binding(&self, project_id: &str) -> Result<Option<ProjectBinding>, StoreError> {
        project_binding_for_project(&self.conn, project_id)
    }

    /// Consulte une liaison sans exposer sa racine canonique à l'appelant.
    pub fn project_binding_projection_for_project(
        &self,
        project_id: &str,
        observed_at: i64,
    ) -> Result<Option<ProjectBindingProjection>, StoreError> {
        self.project_binding(project_id)?
            .map(|binding| project_binding_projection(&self.conn, &binding, observed_at))
            .transpose()
    }

    pub fn project_binding_for_root(
        &self,
        canonical_root: &str,
    ) -> Result<Option<ProjectBinding>, StoreError> {
        project_binding_for_root(&self.conn, canonical_root)
    }

    /// Conserve l'identité de la liaison lorsque le chemin disparaît.
    pub fn mark_project_binding_path_missing(
        &mut self,
        project_id: &str,
        observed_at: i64,
    ) -> Result<ProjectBinding, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let Some(binding) = project_binding_for_project(&tx, project_id)? else {
            return Err(StoreError::Invariant("liaison projet absente"));
        };
        if observed_at < binding.updated_at {
            return Err(StoreError::Invariant("observation liaison antérieure"));
        }
        if binding.state == ProjectBindingState::Disabled {
            return Err(StoreError::Invariant("liaison projet désactivée"));
        }
        if binding.state != ProjectBindingState::PathMissing {
            tx.execute(
                "UPDATE project_bindings
                 SET state = 'path_missing', updated_at = ?1, last_reason = ?2
                 WHERE project_id = ?3",
                params![
                    observed_at,
                    project_registry_refusal_name(ProjectRegistryRefusal::RootMissing),
                    project_id
                ],
            )
            .map_err(StoreError::Sqlite)?;
        }
        let updated = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(updated)
    }

    /// Effectue un rebind explicite, sans toucher au contenu de la racine.
    pub fn rebind_project_binding(
        &mut self,
        project_id: &str,
        canonical_root: &str,
        observed_at: i64,
    ) -> Result<ProjectBinding, StoreError> {
        if canonical_root.trim().is_empty() || !Path::new(canonical_root).is_absolute() {
            return Err(StoreError::Invariant("racine de rebind invalide"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let Some(binding) = project_binding_for_project(&tx, project_id)? else {
            return Err(StoreError::Invariant("liaison projet absente"));
        };
        if binding.state == ProjectBindingState::Disabled {
            return Err(StoreError::Invariant("rebind d'une liaison désactivée"));
        }
        if observed_at < binding.updated_at {
            return Err(StoreError::Invariant("rebind antérieur à la liaison"));
        }
        if binding.state != ProjectBindingState::Active || binding.canonical_root != canonical_root
        {
            let generation = binding
                .generation
                .checked_add(1)
                .ok_or(StoreError::Invariant("génération liaison épuisée"))?;
            let rebind_runtime_epoch = binding
                .runtime
                .as_ref()
                .map(|runtime| {
                    runtime
                        .environment_epoch
                        .checked_add(1)
                        .ok_or(StoreError::Invariant("epoch runtime épuisé"))
                })
                .transpose()?
                .map(|epoch| {
                    i64::try_from(epoch)
                        .map_err(|_| StoreError::Invariant("epoch runtime invalide"))
                })
                .transpose()?;
            tx.execute(
                "UPDATE project_bindings
                 SET canonical_root = ?1, state = 'active', generation = ?2,
                     bound_at = ?3, updated_at = ?3, last_reason = NULL,
                     runtime_state = COALESCE(?4, runtime_state),
                     environment_epoch = COALESCE(?5, environment_epoch)
                 WHERE project_id = ?6",
                params![
                    canonical_root,
                    generation as i64,
                    observed_at,
                    rebind_runtime_epoch.as_ref().map(|_| "recreate_required"),
                    rebind_runtime_epoch,
                    project_id,
                ],
            )
            .map_err(StoreError::Sqlite)?;
        }
        let rebound = project_binding_for_project(&tx, project_id)?
            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(rebound)
    }

    /// Retourne les liaisons comme projections publiques sans chemin hôte.
    /// Complexité : O(n), avec n le nombre de liaisons retenues pour la liste.
    pub fn project_binding_projections(
        &self,
        observed_at: i64,
    ) -> Result<Vec<ProjectBindingProjection>, StoreError> {
        let bindings = {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT project_id, canonical_root, backend, state, generation,
                            bound_at, updated_at, last_reason, runtime_state, runtime_last_reason, policy_id,
                            policy_version, policy_digest, image_reference, resolved_image_id,
                            run_as_uid, run_as_gid, environment_epoch, topology_digest, container_id
                     FROM project_bindings ORDER BY project_id ASC",
                )
                .map_err(StoreError::Sqlite)?;
            statement
                .query_map([], project_binding_from_row)
                .map_err(StoreError::Sqlite)?
                .map(|row| {
                    row.map_err(StoreError::Sqlite)
                        .and_then(decode_project_binding)
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        bindings
            .iter()
            .map(|binding| project_binding_projection(&self.conn, binding, observed_at))
            .collect()
    }

    /// Projette la politique effective d'un projet. L'absence de ligne est
    /// volontairement une désactivation non configurée.
    pub fn project_round_policy_for_project(
        &self,
        project_id: &str,
        observed_at: i64,
    ) -> Result<Option<ProjectRoundProjection>, StoreError> {
        let Some(binding) = self.project_binding(project_id)? else {
            return Ok(None);
        };
        project_round_projection(&self.conn, &binding, observed_at).map(Some)
    }

    /// Liste toutes les liaisons et leur politique effective, y compris les
    /// projets inactifs et les politiques absentes.
    pub fn project_round_policies(
        &self,
        observed_at: i64,
    ) -> Result<Vec<ProjectRoundProjection>, StoreError> {
        let bindings = self.project_binding_projections(observed_at)?;
        bindings
            .into_iter()
            .map(|projection| {
                let binding = self
                    .project_binding(&projection.project_id)?
                    .ok_or(StoreError::Invariant("liaison projet disparue"))?;
                project_round_projection(&self.conn, &binding, observed_at)
            })
            .collect()
    }

    /// Cibles du scheduler unique. Une ancienne génération reste stockée mais
    /// ne peut jamais redevenir active implicitement après rebind.
    pub fn enabled_project_round_policies(
        &self,
        observed_at: i64,
    ) -> Result<Vec<ProjectRoundProjection>, StoreError> {
        Ok(self
            .project_round_policies(observed_at)?
            .into_iter()
            .filter(|policy| policy.active && policy.configured && policy.enabled)
            .collect())
    }

    /// Enregistre en O(1) le dernier dispatch admis sur la génération exacte.
    /// Une occurrence plus ancienne est un no-op afin qu'un rejeu retardé ne
    /// puisse jamais faire régresser l'observation présentée à l'opérateur.
    pub fn record_project_round_dispatch(
        &mut self,
        project_id: &str,
        binding_generation: u64,
        occurrence_at: i64,
        state: ProjectRoundDispatchState,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if project_id.trim().is_empty()
            || binding_generation == 0
            || binding_generation > i64::MAX as u64
            || occurrence_at < 0
            || observed_at < occurrence_at
        {
            return Err(StoreError::Invariant("observation de ronde invalide"));
        }
        let exists = self
            .conn
            .query_row(
                "SELECT 1 FROM project_round_policies
                 WHERE project_id = ?1 AND binding_generation = ?2",
                params![project_id, binding_generation as i64],
                |_| Ok(()),
            )
            .optional()
            .map_err(StoreError::Sqlite)?
            .is_some();
        if !exists {
            return Err(StoreError::ProjectRoundRefusal(
                ProjectRoundRefusal::PolicyDisabled,
            ));
        }
        self.conn
            .execute(
                "UPDATE project_round_policies
                 SET last_occurrence_at = ?1,
                     last_dispatch_state = ?2,
                     last_dispatch_observed_at = ?3
                 WHERE project_id = ?4 AND binding_generation = ?5
                   AND (last_occurrence_at IS NULL OR last_occurrence_at <= ?1)",
                params![
                    occurrence_at,
                    project_round_dispatch_state_name(state),
                    observed_at,
                    project_id,
                    binding_generation as i64
                ],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Applique une décision de ronde épinglée à la génération active et
    /// mémorise l'issue exacte par command_id.
    pub fn apply_project_round_mutation(
        &mut self,
        command_id: &str,
        operation: ProjectRoundOperation,
        project_id: &str,
        binding_generation: u64,
        observed_at: i64,
    ) -> Result<ProjectRoundOutcome, StoreError> {
        let desired_enabled = match operation {
            ProjectRoundOperation::Enable => true,
            ProjectRoundOperation::Disable => false,
            ProjectRoundOperation::List | ProjectRoundOperation::Status => {
                return Err(StoreError::Invariant("opération de ronde non mutante"));
            }
        };
        if command_id.trim().is_empty()
            || project_id.trim().is_empty()
            || binding_generation == 0
            || binding_generation > i64::MAX as u64
            || observed_at < 0
        {
            return Err(StoreError::Invariant("mutation de ronde invalide"));
        }
        let canonical_bytes = format!(
            "project-round-policy-v1|{}|{}|{}",
            project_round_operation_name(operation),
            project_id,
            binding_generation
        )
        .into_bytes();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;

        if let Some((stored_canonical, outcome_json)) = tx
            .query_row(
                "SELECT canonical_bytes, outcome_json
                 FROM project_round_commands WHERE command_id = ?1",
                [command_id],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()
            .map_err(StoreError::Sqlite)?
        {
            if stored_canonical != canonical_bytes {
                return Err(StoreError::ProjectRoundRefusal(
                    ProjectRoundRefusal::EnvelopeMismatch,
                ));
            }
            let outcome = serde_json::from_slice(&outcome_json)
                .map_err(|_| StoreError::Invariant("issue de ronde corrompue"))?;
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(outcome);
        }

        let outcome = match project_binding_for_project(&tx, project_id)? {
            None => project_round_failure(
                command_id,
                operation,
                ProjectRoundRefusal::ProjectNotFound,
                observed_at,
            ),
            Some(binding) if binding.state != ProjectBindingState::Active => project_round_failure(
                command_id,
                operation,
                ProjectRoundRefusal::ProjectInactive,
                observed_at,
            ),
            Some(binding) if binding.generation != binding_generation => project_round_failure(
                command_id,
                operation,
                ProjectRoundRefusal::BindingGenerationMismatch,
                observed_at,
            ),
            Some(binding) => {
                let stored = tx
                    .query_row(
                        "SELECT enabled, revision
                         FROM project_round_policies
                         WHERE project_id = ?1 AND binding_generation = ?2",
                        params![project_id, binding_generation as i64],
                        |row| Ok((row.get::<_, bool>(0)?, row.get::<_, u64>(1)?)),
                    )
                    .optional()
                    .map_err(StoreError::Sqlite)?;
                match stored {
                    None => {
                        tx.execute(
                            "INSERT INTO project_round_policies (
                                project_id, binding_generation, enabled,
                                revision, updated_at, command_id
                             ) VALUES (?1, ?2, ?3, 1, ?4, ?5)",
                            params![
                                project_id,
                                binding_generation as i64,
                                desired_enabled,
                                observed_at,
                                command_id
                            ],
                        )
                        .map_err(StoreError::Sqlite)?;
                    }
                    Some((enabled, revision)) if enabled != desired_enabled => {
                        let next_revision = revision
                            .checked_add(1)
                            .ok_or(StoreError::Invariant("révision de ronde épuisée"))?;
                        tx.execute(
                            "UPDATE project_round_policies
                             SET enabled = ?1, revision = ?2,
                                 updated_at = ?3, command_id = ?4
                             WHERE project_id = ?5 AND binding_generation = ?6",
                            params![
                                desired_enabled,
                                next_revision,
                                observed_at,
                                command_id,
                                project_id,
                                binding_generation as i64
                            ],
                        )
                        .map_err(StoreError::Sqlite)?;
                    }
                    Some(_) => {}
                }
                let projection = project_round_projection(&tx, &binding, observed_at)?;
                ProjectRoundOutcome {
                    contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
                    command_id: command_id.to_string(),
                    operation,
                    policies: vec![projection],
                    reason: None,
                    observed_at,
                }
            }
        };
        let outcome_json = serde_json::to_vec(&outcome)
            .map_err(|_| StoreError::Invariant("issue de ronde non sérialisable"))?;
        tx.execute(
            "INSERT INTO project_round_commands (
                command_id, operation, project_id, binding_generation,
                canonical_bytes, outcome_json, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                command_id,
                project_round_operation_name(operation),
                project_id,
                binding_generation as i64,
                canonical_bytes,
                outcome_json,
                observed_at
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(outcome)
    }

    /// Applique rebind ou disable avec son audit dans la transaction qui porte
    /// la mutation. L'issue complète est mémorisée par command_id, de sorte
    /// qu'un retry ne peut ni changer de racine ni écrire un second audit.
    pub fn apply_project_admin_mutation(
        &mut self,
        command_id: &str,
        operation: ProjectAdminOperation,
        project_id: &str,
        canonical_root: Option<&str>,
        observed_at: i64,
    ) -> Result<ProjectAdminOutcome, StoreError> {
        if command_id.trim().is_empty() || project_id.trim().is_empty() || observed_at < 0 {
            return Err(StoreError::Invariant(
                "mutation projet administrative invalide",
            ));
        }
        if !matches!(
            operation,
            ProjectAdminOperation::Activate
                | ProjectAdminOperation::Rebind
                | ProjectAdminOperation::Disable
                | ProjectAdminOperation::ReviewProjectReconcile
        ) {
            return Err(StoreError::Invariant("opération projet non mutante"));
        }
        if matches!(
            operation,
            ProjectAdminOperation::Activate
                | ProjectAdminOperation::Rebind
                | ProjectAdminOperation::ReviewProjectReconcile
        ) && !canonical_root
            .is_some_and(|root| !root.trim().is_empty() && Path::new(root).is_absolute())
        {
            return Err(StoreError::Invariant("racine de mutation invalide"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        if let Some(attempt) = project_admin_attempt_for_command(&tx, command_id)? {
            if attempt.operation != project_admin_operation_name(operation)
                || attempt.project_id.as_deref() != Some(project_id)
                || attempt.canonical_root.as_deref() != canonical_root
            {
                return Err(StoreError::ProjectRegistryRefusal(
                    ProjectRegistryRefusal::EnvelopeMismatch,
                ));
            }
            let outcome = serde_json::from_slice(&attempt.outcome_json)
                .map_err(|_| StoreError::Invariant("issue administrative projet corrompue"))?;
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(outcome);
        }

        let outcome = match project_binding_for_project(&tx, project_id)? {
            None => project_admin_failure(
                command_id,
                operation,
                ProjectRegistryRefusal::InvalidProjectId,
                observed_at,
            ),
            Some(binding) if observed_at < binding.updated_at => project_admin_failure(
                command_id,
                operation,
                ProjectRegistryRefusal::IdempotencyExpired,
                observed_at,
            ),
            Some(binding) => match operation {
                ProjectAdminOperation::Rebind | ProjectAdminOperation::ReviewProjectReconcile => {
                    let canonical_root = canonical_root.expect("racine rebind validée");
                    if binding.state == ProjectBindingState::Disabled {
                        project_admin_failure(
                            command_id,
                            operation,
                            ProjectRegistryRefusal::ProjectDisabled,
                            observed_at,
                        )
                    } else if binding.state == ProjectBindingState::Active
                        && binding.canonical_root == canonical_root
                    {
                        if operation == ProjectAdminOperation::ReviewProjectReconcile {
                            let audit = ProjectAuditEvent::for_mutation(
                                command_id,
                                project_id,
                                ProjectAuditOperation::ReviewProjectReconcile,
                                binding.generation,
                                ProjectAuditOutcome::Applied,
                                Some(&binding.canonical_root),
                                observed_at,
                            );
                            record_project_audit_event_in_transaction(&tx, &audit)?;
                        }
                        project_admin_success(&tx, command_id, operation, &binding, observed_at)?
                    } else if let Some(owner) = project_binding_for_root(&tx, canonical_root)?
                        && owner.project_id != project_id
                    {
                        project_admin_failure(
                            command_id,
                            operation,
                            ProjectRegistryRefusal::RootAlreadyBound,
                            observed_at,
                        )
                    } else {
                        let generation = binding
                            .generation
                            .checked_add(1)
                            .ok_or(StoreError::Invariant("génération liaison épuisée"))?;
                        let rebind_runtime_epoch = binding
                            .runtime
                            .as_ref()
                            .map(|runtime| {
                                runtime
                                    .environment_epoch
                                    .checked_add(1)
                                    .ok_or(StoreError::Invariant("epoch runtime épuisé"))
                            })
                            .transpose()?
                            .map(|epoch| {
                                i64::try_from(epoch)
                                    .map_err(|_| StoreError::Invariant("epoch runtime invalide"))
                            })
                            .transpose()?;
                        tx.execute(
                            "UPDATE project_bindings
                             SET canonical_root = ?1, state = 'active', generation = ?2,
                                 bound_at = ?3, updated_at = ?3, last_reason = NULL,
                                 runtime_state = COALESCE(?4, runtime_state),
                                 environment_epoch = COALESCE(?5, environment_epoch)
                             WHERE project_id = ?6",
                            params![
                                canonical_root,
                                generation as i64,
                                observed_at,
                                rebind_runtime_epoch.as_ref().map(|_| "recreate_required"),
                                rebind_runtime_epoch,
                                project_id,
                            ],
                        )
                        .map_err(StoreError::Sqlite)?;
                        let rebound = project_binding_for_project(&tx, project_id)?
                            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
                        let audit = ProjectAuditEvent::for_mutation(
                            command_id,
                            project_id,
                            if operation == ProjectAdminOperation::ReviewProjectReconcile {
                                ProjectAuditOperation::ReviewProjectReconcile
                            } else {
                                ProjectAuditOperation::Rebind
                            },
                            rebound.generation,
                            ProjectAuditOutcome::Applied,
                            Some(&binding.canonical_root),
                            observed_at,
                        );
                        record_project_audit_event_in_transaction(&tx, &audit)?;
                        project_admin_success(&tx, command_id, operation, &rebound, observed_at)?
                    }
                }
                ProjectAdminOperation::Activate => {
                    let canonical_root = canonical_root.expect("racine activation validée");
                    if binding.state == ProjectBindingState::Active {
                        if binding.canonical_root == canonical_root {
                            project_admin_success(
                                &tx,
                                command_id,
                                operation,
                                &binding,
                                observed_at,
                            )?
                        } else {
                            project_admin_failure(
                                command_id,
                                operation,
                                ProjectRegistryRefusal::RootAlreadyBound,
                                observed_at,
                            )
                        }
                    } else if binding.state != ProjectBindingState::Disabled {
                        project_admin_failure(
                            command_id,
                            operation,
                            ProjectRegistryRefusal::ProjectDisabled,
                            observed_at,
                        )
                    } else if let Some(owner) = project_binding_for_root(&tx, canonical_root)?
                        && owner.project_id != project_id
                    {
                        project_admin_failure(
                            command_id,
                            operation,
                            ProjectRegistryRefusal::RootAlreadyBound,
                            observed_at,
                        )
                    } else {
                        tx.execute(
                            "UPDATE project_bindings
                             SET canonical_root = ?1, state = 'active', updated_at = ?2,
                                 bound_at = ?2, last_reason = NULL
                             WHERE project_id = ?3",
                            params![canonical_root, observed_at, project_id],
                        )
                        .map_err(StoreError::Sqlite)?;
                        let activated = project_binding_for_project(&tx, project_id)?
                            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
                        let audit = ProjectAuditEvent::for_mutation(
                            command_id,
                            project_id,
                            ProjectAuditOperation::Activate,
                            activated.generation,
                            ProjectAuditOutcome::Applied,
                            Some(&binding.canonical_root),
                            observed_at,
                        );
                        record_project_audit_event_in_transaction(&tx, &audit)?;
                        project_admin_success(&tx, command_id, operation, &activated, observed_at)?
                    }
                }
                ProjectAdminOperation::Disable => {
                    if binding.state == ProjectBindingState::Disabled {
                        project_admin_success(&tx, command_id, operation, &binding, observed_at)?
                    } else {
                        tx.execute(
                            "UPDATE project_bindings
                             SET state = 'disabled', updated_at = ?1, last_reason = NULL
                             WHERE project_id = ?2",
                            params![observed_at, project_id],
                        )
                        .map_err(StoreError::Sqlite)?;
                        let disabled = project_binding_for_project(&tx, project_id)?
                            .ok_or(StoreError::Invariant("liaison projet disparue"))?;
                        let audit = ProjectAuditEvent::for_mutation(
                            command_id,
                            project_id,
                            ProjectAuditOperation::Disable,
                            disabled.generation,
                            ProjectAuditOutcome::Applied,
                            Some(&binding.canonical_root),
                            observed_at,
                        );
                        record_project_audit_event_in_transaction(&tx, &audit)?;
                        project_admin_success(&tx, command_id, operation, &disabled, observed_at)?
                    }
                }
                ProjectAdminOperation::List | ProjectAdminOperation::Status => unreachable!(),
            },
        };
        let outcome_json = serde_json::to_vec(&outcome)
            .map_err(|_| StoreError::Invariant("issue administrative projet non sérialisable"))?;
        tx.execute(
            "INSERT INTO project_admin_attempts (
                 command_id, operation, project_id, canonical_root, outcome_json, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                command_id,
                project_admin_operation_name(operation),
                project_id,
                canonical_root,
                outcome_json,
                observed_at,
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(outcome)
    }

    /// Écrit exactement un audit déterministe pour une mutation effective.
    pub fn record_project_audit_event(
        &mut self,
        event: &ProjectAuditEvent,
    ) -> Result<bool, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let changed = record_project_audit_event_in_transaction(&tx, event)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(changed)
    }

    pub fn project_audit_events(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectAuditEvent>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT audit_event_id, command_id, project_id, operation,
                        binding_generation, outcome_json, previous_root_reference, observed_at
                 FROM project_audit_events
                 WHERE project_id = ?1
                 ORDER BY observed_at ASC, audit_event_id ASC",
            )
            .map_err(StoreError::Sqlite)?;
        let rows = statement
            .query_map([project_id], project_audit_event_from_row)
            .map_err(StoreError::Sqlite)?;
        rows.map(|row| {
            row.map_err(StoreError::Sqlite)
                .and_then(decode_project_audit_event)
        })
        .collect()
    }
}

pub(super) fn ensure_project_bindings_runtime_schema(conn: &Connection) -> Result<(), StoreError> {
    let mut statement = conn
        .prepare("PRAGMA table_info(project_bindings)")
        .map_err(StoreError::Sqlite)?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(StoreError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sqlite)?;
    if !columns.iter().any(|column| column == "environment_epoch") {
        conn.execute_batch(
        "BEGIN IMMEDIATE;
         DROP INDEX IF EXISTS idx_project_bindings_live_root;
         ALTER TABLE project_bindings RENAME TO project_bindings_065;
         CREATE TABLE project_bindings (
             project_id TEXT PRIMARY KEY,
             canonical_root TEXT NOT NULL,
             backend TEXT NOT NULL CHECK (backend IN (\x27host\x27, \x27docker\x27)),
             state TEXT NOT NULL CHECK (state IN (
                 \x27pending_binding\x27, \x27active\x27, \x27disabled\x27, \x27path_missing\x27, \x27binding_failed\x27
             )),
             generation INTEGER NOT NULL CHECK (generation > 0),
             bound_at INTEGER NOT NULL,
             updated_at INTEGER NOT NULL,
             last_reason TEXT,
             runtime_state TEXT,
             runtime_last_reason TEXT,
             policy_id TEXT,
             policy_version INTEGER,
             policy_digest TEXT,
             image_reference TEXT,
             resolved_image_id TEXT,
             run_as_uid INTEGER,
             run_as_gid INTEGER,
             environment_epoch INTEGER NOT NULL DEFAULT 0,
             topology_digest TEXT NOT NULL DEFAULT 'sha256:legacy',
             container_id TEXT
         );
         INSERT INTO project_bindings (
             project_id, canonical_root, backend, state, generation, bound_at,
             updated_at, last_reason, runtime_state, runtime_last_reason, policy_id, policy_version,
             policy_digest, image_reference, resolved_image_id, run_as_uid,
             run_as_gid, environment_epoch, topology_digest, container_id
         )
         SELECT project_id, canonical_root, \x27host\x27, state, generation, bound_at,
                updated_at, last_reason, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                NULL, NULL, 0, 'sha256:legacy', NULL
         FROM project_bindings_065;
         DROP TABLE project_bindings_065;
         CREATE UNIQUE INDEX idx_project_bindings_live_root
             ON project_bindings(canonical_root) WHERE state != \x27disabled\x27;
         COMMIT;",
    )
    .map_err(StoreError::Sqlite)?;
        return Ok(());
    }
    if !columns.iter().any(|column| column == "runtime_last_reason") {
        conn.execute(
            "ALTER TABLE project_bindings ADD COLUMN runtime_last_reason TEXT",
            [],
        )
        .map_err(StoreError::Sqlite)?;
    }
    if !columns.iter().any(|column| column == "topology_digest") {
        conn.execute(
            "ALTER TABLE project_bindings ADD COLUMN topology_digest TEXT NOT NULL DEFAULT 'sha256:legacy'",
            [],
        )
        .map_err(StoreError::Sqlite)?;
    }
    Ok(())
}
#[derive(Debug)]
struct StoredProjectBinding {
    project_id: String,
    canonical_root: String,
    backend: String,
    state: String,
    generation: i64,
    bound_at: i64,
    updated_at: i64,
    last_reason: Option<String>,
    runtime_state: Option<String>,
    runtime_last_reason: Option<String>,
    policy_id: Option<String>,
    policy_version: Option<i64>,
    policy_digest: Option<String>,
    image_reference: Option<String>,
    resolved_image_id: Option<String>,
    run_as_uid: Option<i64>,
    run_as_gid: Option<i64>,
    environment_epoch: i64,
    topology_digest: String,
    container_id: Option<String>,
}

fn project_binding_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredProjectBinding> {
    Ok(StoredProjectBinding {
        project_id: row.get(0)?,
        canonical_root: row.get(1)?,
        backend: row.get(2)?,
        state: row.get(3)?,
        generation: row.get(4)?,
        bound_at: row.get(5)?,
        updated_at: row.get(6)?,
        last_reason: row.get(7)?,
        runtime_state: row.get(8)?,
        runtime_last_reason: row.get(9)?,
        policy_id: row.get(10)?,
        policy_version: row.get(11)?,
        policy_digest: row.get(12)?,
        image_reference: row.get(13)?,
        resolved_image_id: row.get(14)?,
        run_as_uid: row.get(15)?,
        run_as_gid: row.get(16)?,
        environment_epoch: row.get(17)?,
        topology_digest: row.get(18)?,
        container_id: row.get(19)?,
    })
}

fn project_environment_state_name(state: ProjectEnvironmentState) -> &'static str {
    match state {
        ProjectEnvironmentState::Absent => "absent",
        ProjectEnvironmentState::Creating => "creating",
        ProjectEnvironmentState::Ready => "ready",
        ProjectEnvironmentState::Running => "running",
        ProjectEnvironmentState::Stopping => "stopping",
        ProjectEnvironmentState::Stopped => "stopped",
        ProjectEnvironmentState::Degraded => "degraded",
        ProjectEnvironmentState::RecreateRequired => "recreate_required",
    }
}

fn project_environment_state_from_name(value: &str) -> Result<ProjectEnvironmentState, StoreError> {
    match value {
        "absent" => Ok(ProjectEnvironmentState::Absent),
        "creating" => Ok(ProjectEnvironmentState::Creating),
        "ready" => Ok(ProjectEnvironmentState::Ready),
        "running" => Ok(ProjectEnvironmentState::Running),
        "stopping" => Ok(ProjectEnvironmentState::Stopping),
        "stopped" => Ok(ProjectEnvironmentState::Stopped),
        "degraded" => Ok(ProjectEnvironmentState::Degraded),
        "recreate_required" => Ok(ProjectEnvironmentState::RecreateRequired),
        _ => Err(StoreError::Invariant("etat runtime projet inconnu")),
    }
}

fn decode_project_binding(row: StoredProjectBinding) -> Result<ProjectBinding, StoreError> {
    let (backend, runtime) = match row.backend.as_str() {
        "host" => {
            if row.runtime_state.is_some()
                || row.runtime_last_reason.is_some()
                || row.policy_id.is_some()
                || row.policy_version.is_some()
                || row.policy_digest.is_some()
                || row.image_reference.is_some()
                || row.resolved_image_id.is_some()
                || row.run_as_uid.is_some()
                || row.run_as_gid.is_some()
                || row.environment_epoch != 0
                || row.topology_digest != "sha256:legacy"
                || row.container_id.is_some()
            {
                return Err(StoreError::Invariant("runtime docker sur backend host"));
            }
            (ProjectBackend::Host, None)
        }
        "docker" => {
            let runtime_state = row
                .runtime_state
                .as_deref()
                .ok_or(StoreError::Invariant("etat runtime docker absent"))?;
            let policy_id = row
                .policy_id
                .ok_or(StoreError::Invariant("policy runtime absente"))?;
            let policy_version = u64::try_from(
                row.policy_version
                    .ok_or(StoreError::Invariant("version policy runtime absente"))?,
            )
            .map_err(|_| StoreError::Invariant("version policy runtime invalide"))?;
            let policy_digest = row
                .policy_digest
                .ok_or(StoreError::Invariant("digest policy runtime absent"))?;
            let image_reference = row
                .image_reference
                .ok_or(StoreError::Invariant("image runtime absente"))?;
            let run_as_uid = u32::try_from(
                row.run_as_uid
                    .ok_or(StoreError::Invariant("uid runtime absent"))?,
            )
            .map_err(|_| StoreError::Invariant("uid runtime invalide"))?;
            let run_as_gid = u32::try_from(
                row.run_as_gid
                    .ok_or(StoreError::Invariant("gid runtime absent"))?,
            )
            .map_err(|_| StoreError::Invariant("gid runtime invalide"))?;
            let environment_epoch = u64::try_from(row.environment_epoch)
                .map_err(|_| StoreError::Invariant("epoch runtime invalide"))?;
            (
                ProjectBackend::Docker,
                Some(ProjectRuntimeBinding {
                    state: project_environment_state_from_name(runtime_state)?,
                    policy_id,
                    policy_version,
                    policy_digest,
                    image_reference,
                    resolved_image_id: row.resolved_image_id,
                    run_as_uid,
                    run_as_gid,
                    environment_epoch,
                    topology_digest: row.topology_digest,
                    container_id: row.container_id,
                    last_reason: row.runtime_last_reason,
                }),
            )
        }
        _ => return Err(StoreError::Invariant("backend projet inconnu")),
    };
    let generation = u64::try_from(row.generation)
        .map_err(|_| StoreError::Invariant("génération liaison invalide"))?;
    let last_reason = row
        .last_reason
        .as_deref()
        .map(project_registry_refusal_from_name)
        .transpose()?;
    let binding = ProjectBinding {
        project_id: row.project_id,
        canonical_root: row.canonical_root,
        backend,
        state: ProjectBindingState::from_db(&row.state)?,
        generation,
        bound_at: row.bound_at,
        updated_at: row.updated_at,
        last_reason,
        runtime,
    };
    binding.validate()?;
    Ok(binding)
}

fn project_binding_for_project(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<ProjectBinding>, StoreError> {
    let stored = conn
        .query_row(
            "SELECT project_id, canonical_root, backend, state, generation,
                    bound_at, updated_at, last_reason, runtime_state, runtime_last_reason, policy_id,
                    policy_version, policy_digest, image_reference, resolved_image_id,
                    run_as_uid, run_as_gid, environment_epoch, topology_digest, container_id
             FROM project_bindings WHERE project_id = ?1",
            [project_id],
            project_binding_from_row,
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    stored.map(decode_project_binding).transpose()
}

fn project_binding_for_root(
    conn: &Connection,
    canonical_root: &str,
) -> Result<Option<ProjectBinding>, StoreError> {
    let stored = conn
        .query_row(
            "SELECT project_id, canonical_root, backend, state, generation,
                    bound_at, updated_at, last_reason, runtime_state, runtime_last_reason, policy_id,
                    policy_version, policy_digest, image_reference, resolved_image_id,
                    run_as_uid, run_as_gid, environment_epoch, topology_digest, container_id
             FROM project_bindings
             WHERE canonical_root = ?1 AND state != 'disabled'",
            [canonical_root],
            project_binding_from_row,
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    stored.map(decode_project_binding).transpose()
}

#[derive(Debug)]
struct StoredProjectBindingAttempt {
    project_id: String,
    canonical_root: String,
    outcome_json: Vec<u8>,
}

#[derive(Debug)]
struct StoredProjectAdminAttempt {
    operation: String,
    project_id: Option<String>,
    canonical_root: Option<String>,
    outcome_json: Vec<u8>,
}

fn project_admin_attempt_for_command(
    conn: &Connection,
    command_id: &str,
) -> Result<Option<StoredProjectAdminAttempt>, StoreError> {
    conn.query_row(
        "SELECT operation, project_id, canonical_root, outcome_json
         FROM project_admin_attempts WHERE command_id = ?1",
        [command_id],
        |row| {
            Ok(StoredProjectAdminAttempt {
                operation: row.get(0)?,
                project_id: row.get(1)?,
                canonical_root: row.get(2)?,
                outcome_json: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(StoreError::Sqlite)
}

fn project_admin_operation_name(operation: ProjectAdminOperation) -> &'static str {
    match operation {
        ProjectAdminOperation::List => "list",
        ProjectAdminOperation::Status => "status",
        ProjectAdminOperation::Rebind => "rebind",
        ProjectAdminOperation::Activate => "activate",
        ProjectAdminOperation::Disable => "disable",
        ProjectAdminOperation::ReviewProjectReconcile => "review_project_reconcile",
    }
}

fn project_round_operation_name(operation: ProjectRoundOperation) -> &'static str {
    match operation {
        ProjectRoundOperation::List => "list",
        ProjectRoundOperation::Status => "status",
        ProjectRoundOperation::Enable => "enable",
        ProjectRoundOperation::Disable => "disable",
    }
}

fn project_round_dispatch_state_name(state: ProjectRoundDispatchState) -> &'static str {
    match state {
        ProjectRoundDispatchState::Deposited => "deposited",
        ProjectRoundDispatchState::Refused => "refused",
        ProjectRoundDispatchState::Indeterminate => "indeterminate",
    }
}

fn parse_project_round_dispatch_state(
    value: Option<String>,
) -> Result<Option<ProjectRoundDispatchState>, StoreError> {
    match value.as_deref() {
        None => Ok(None),
        Some("deposited") => Ok(Some(ProjectRoundDispatchState::Deposited)),
        Some("refused") => Ok(Some(ProjectRoundDispatchState::Refused)),
        Some("indeterminate") => Ok(Some(ProjectRoundDispatchState::Indeterminate)),
        Some(_) => Err(StoreError::Invariant("état de dispatch de ronde corrompu")),
    }
}

fn project_round_projection(
    conn: &Connection,
    binding: &ProjectBinding,
    observed_at: i64,
) -> Result<ProjectRoundProjection, StoreError> {
    let stored = conn
        .query_row(
            "SELECT enabled, revision, updated_at,
                    last_occurrence_at, last_dispatch_state,
                    last_dispatch_observed_at
             FROM project_round_policies
             WHERE project_id = ?1 AND binding_generation = ?2",
            params![binding.project_id, binding.generation as i64],
            |row| {
                Ok((
                    row.get::<_, bool>(0)?,
                    row.get::<_, u64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    let active = binding.state == ProjectBindingState::Active;
    let (
        configured,
        enabled,
        revision,
        updated_at,
        last_occurrence_at,
        last_dispatch_state,
        last_dispatch_observed_at,
    ) = match stored {
        Some((
            enabled,
            revision,
            updated_at,
            last_occurrence_at,
            last_dispatch_state,
            last_dispatch_observed_at,
        )) => (
            true,
            active && enabled,
            revision,
            updated_at,
            last_occurrence_at,
            parse_project_round_dispatch_state(last_dispatch_state)?,
            last_dispatch_observed_at,
        ),
        None => (
            false,
            false,
            0,
            binding.updated_at.min(observed_at),
            None,
            None,
            None,
        ),
    };
    Ok(ProjectRoundProjection {
        project_id: binding.project_id.clone(),
        binding_generation: Some(binding.generation),
        active,
        configured,
        enabled,
        revision,
        updated_at,
        last_occurrence_at,
        last_dispatch_state,
        last_dispatch_observed_at,
    })
}

fn project_round_failure(
    command_id: &str,
    operation: ProjectRoundOperation,
    reason: ProjectRoundRefusal,
    observed_at: i64,
) -> ProjectRoundOutcome {
    ProjectRoundOutcome {
        contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
        command_id: command_id.to_string(),
        operation,
        policies: Vec::new(),
        reason: Some(reason),
        observed_at,
    }
}

fn project_role_for_project(
    conn: &Connection,
    project_id: &str,
) -> Result<ProjectRole, StoreError> {
    let declared = conn
        .query_row(
            "SELECT 1 FROM project_system_roles WHERE singleton = 1 AND project_id = ?1",
            [project_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(StoreError::Sqlite)?
        .is_some();
    Ok(if declared {
        ProjectRole::BridgetSystem
    } else {
        ProjectRole::Standard
    })
}

fn project_binding_projection(
    conn: &Connection,
    binding: &ProjectBinding,
    observed_at: i64,
) -> Result<ProjectBindingProjection, StoreError> {
    let state = match binding.state {
        ProjectBindingState::Active => ProjectBindingStatus::Active,
        ProjectBindingState::Disabled => ProjectBindingStatus::Disabled,
        ProjectBindingState::PathMissing => ProjectBindingStatus::PathMissing,
        ProjectBindingState::PendingBinding => ProjectBindingStatus::PendingBinding,
        ProjectBindingState::BindingFailed => ProjectBindingStatus::BindingFailed,
    };
    Ok(ProjectBindingProjection {
        project_id: binding.project_id.clone(),
        canonical_root: Some(binding.canonical_root.clone()),
        state,
        binding_generation: Some(binding.generation),
        backend: Some(binding.backend),
        role: project_role_for_project(conn, &binding.project_id)?,
        runtime_policy: binding
            .runtime
            .as_ref()
            .map(ProjectRuntimeBinding::policy_reference),
        reason: binding.last_reason,
        last_audit: project_audit_projection_for_project(conn, &binding.project_id)?,
        observed_at,
    })
}

fn project_admin_success(
    conn: &Connection,
    command_id: &str,
    operation: ProjectAdminOperation,
    binding: &ProjectBinding,
    observed_at: i64,
) -> Result<ProjectAdminOutcome, StoreError> {
    Ok(ProjectAdminOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: command_id.to_string(),
        operation,
        bindings: vec![project_binding_projection(conn, binding, observed_at)?],
        reason: None,
        observed_at,
    })
}

fn project_admin_failure(
    command_id: &str,
    operation: ProjectAdminOperation,
    reason: ProjectRegistryRefusal,
    observed_at: i64,
) -> ProjectAdminOutcome {
    ProjectAdminOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: command_id.to_string(),
        operation,
        bindings: Vec::new(),
        reason: Some(reason),
        observed_at,
    }
}

fn project_binding_attempt_for_command(
    conn: &Connection,
    command_id: &str,
) -> Result<Option<StoredProjectBindingAttempt>, StoreError> {
    conn.query_row(
        "SELECT project_id, canonical_root, outcome_json
         FROM project_binding_attempts WHERE command_id = ?1",
        [command_id],
        |row| {
            Ok(StoredProjectBindingAttempt {
                project_id: row.get(0)?,
                canonical_root: row.get(1)?,
                outcome_json: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(StoreError::Sqlite)
}

fn project_binding_active_outcome_for_binding(
    command_id: &str,
    binding: &ProjectBinding,
    observed_at: i64,
) -> ProjectBindOutcome {
    ProjectBindOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: command_id.to_string(),
        project_id: binding.project_id.clone(),
        status: ProjectBindStatus::Active,
        binding_generation: Some(binding.generation),
        backend: Some(binding.backend),
        runtime_policy: binding
            .runtime
            .as_ref()
            .map(ProjectRuntimeBinding::policy_reference),
        reason: None,
        existing_project_id: None,
        existing_binding_generation: None,
        observed_at,
    }
}

fn project_binding_active_outcome(
    command_id: &str,
    project_id: &str,
    generation: u64,
    observed_at: i64,
) -> ProjectBindOutcome {
    ProjectBindOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: command_id.to_string(),
        project_id: project_id.to_string(),
        status: ProjectBindStatus::Active,
        binding_generation: Some(generation),
        backend: Some(ProjectBackend::Host),
        runtime_policy: None,
        reason: None,
        existing_project_id: None,
        existing_binding_generation: None,
        observed_at,
    }
}

fn project_binding_failed_outcome(
    command_id: &str,
    project_id: &str,
    reason: ProjectRegistryRefusal,
    observed_at: i64,
) -> ProjectBindOutcome {
    ProjectBindOutcome {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: command_id.to_string(),
        project_id: project_id.to_string(),
        status: ProjectBindStatus::BindingFailed,
        binding_generation: None,
        backend: None,
        runtime_policy: None,
        reason: Some(reason),
        existing_project_id: None,
        existing_binding_generation: None,
        observed_at,
    }
}

fn project_registry_refusal_name(reason: ProjectRegistryRefusal) -> &'static str {
    match reason {
        ProjectRegistryRefusal::InvalidContract => "invalid_contract",
        ProjectRegistryRefusal::InvalidProjectId => "invalid_project_id",
        ProjectRegistryRefusal::InvalidAbsoluteRoot => "invalid_absolute_root",
        ProjectRegistryRefusal::RootMissing => "root_missing",
        ProjectRegistryRefusal::RootNotDirectory => "root_not_directory",
        ProjectRegistryRefusal::RootOutsideAllowedPrefixes => "root_outside_allowed_prefixes",
        ProjectRegistryRefusal::RootTooBroad => "root_too_broad",
        ProjectRegistryRefusal::RootAlreadyBound => "root_already_bound",
        ProjectRegistryRefusal::ProjectAlreadyBoundElsewhere => "project_already_bound_elsewhere",
        ProjectRegistryRefusal::RebindRequired => "rebind_required",
        ProjectRegistryRefusal::ProjectDisabled => "project_disabled",
        ProjectRegistryRefusal::EnvelopeMismatch => "envelope_mismatch",
        ProjectRegistryRefusal::IdempotencyExpired => "idempotency_expired",
        ProjectRegistryRefusal::StoreUnavailable => "store_unavailable",
        ProjectRegistryRefusal::RegistrationConflict => "registration_conflict",
        ProjectRegistryRefusal::ProjectRegistryCapabilityMissing => {
            "project_registry_capability_missing"
        }
        ProjectRegistryRefusal::ProjectRegistryVersionUnsupported => {
            "project_registry_version_unsupported"
        }
        ProjectRegistryRefusal::LocalOperatorRequired => "local_operator_required",
        ProjectRegistryRefusal::PeerUidMismatch => "peer_uid_mismatch",
        ProjectRegistryRefusal::ProjectRootPolicyUnavailable => "project_root_policy_unavailable",
        ProjectRegistryRefusal::ProjectRootPolicyInvalid => "project_root_policy_invalid",
        ProjectRegistryRefusal::ProjectRootPolicyPermissionsInvalid => {
            "project_root_policy_permissions_invalid"
        }
    }
}

fn project_registry_refusal_from_name(value: &str) -> Result<ProjectRegistryRefusal, StoreError> {
    match value {
        "invalid_contract" => Ok(ProjectRegistryRefusal::InvalidContract),
        "invalid_project_id" => Ok(ProjectRegistryRefusal::InvalidProjectId),
        "invalid_absolute_root" => Ok(ProjectRegistryRefusal::InvalidAbsoluteRoot),
        "root_missing" => Ok(ProjectRegistryRefusal::RootMissing),
        "root_not_directory" => Ok(ProjectRegistryRefusal::RootNotDirectory),
        "root_outside_allowed_prefixes" => Ok(ProjectRegistryRefusal::RootOutsideAllowedPrefixes),
        "root_too_broad" => Ok(ProjectRegistryRefusal::RootTooBroad),
        "root_already_bound" => Ok(ProjectRegistryRefusal::RootAlreadyBound),
        "project_already_bound_elsewhere" => {
            Ok(ProjectRegistryRefusal::ProjectAlreadyBoundElsewhere)
        }
        "rebind_required" => Ok(ProjectRegistryRefusal::RebindRequired),
        "project_disabled" => Ok(ProjectRegistryRefusal::ProjectDisabled),
        "envelope_mismatch" => Ok(ProjectRegistryRefusal::EnvelopeMismatch),
        "idempotency_expired" => Ok(ProjectRegistryRefusal::IdempotencyExpired),
        "store_unavailable" => Ok(ProjectRegistryRefusal::StoreUnavailable),
        "registration_conflict" => Ok(ProjectRegistryRefusal::RegistrationConflict),
        "project_registry_capability_missing" => {
            Ok(ProjectRegistryRefusal::ProjectRegistryCapabilityMissing)
        }
        "project_registry_version_unsupported" => {
            Ok(ProjectRegistryRefusal::ProjectRegistryVersionUnsupported)
        }
        "local_operator_required" => Ok(ProjectRegistryRefusal::LocalOperatorRequired),
        "peer_uid_mismatch" => Ok(ProjectRegistryRefusal::PeerUidMismatch),
        "project_root_policy_unavailable" => {
            Ok(ProjectRegistryRefusal::ProjectRootPolicyUnavailable)
        }
        "project_root_policy_invalid" => Ok(ProjectRegistryRefusal::ProjectRootPolicyInvalid),
        "project_root_policy_permissions_invalid" => {
            Ok(ProjectRegistryRefusal::ProjectRootPolicyPermissionsInvalid)
        }
        _ => Err(StoreError::Invariant("raison liaison projet inconnue")),
    }
}

#[derive(Debug)]
struct StoredProjectAuditEvent {
    audit_event_id: String,
    command_id: String,
    project_id: String,
    operation: String,
    binding_generation: i64,
    outcome_json: Vec<u8>,
    previous_root_reference: Option<String>,
    observed_at: i64,
}

fn project_audit_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StoredProjectAuditEvent> {
    Ok(StoredProjectAuditEvent {
        audit_event_id: row.get(0)?,
        command_id: row.get(1)?,
        project_id: row.get(2)?,
        operation: row.get(3)?,
        binding_generation: row.get(4)?,
        outcome_json: row.get(5)?,
        previous_root_reference: row.get(6)?,
        observed_at: row.get(7)?,
    })
}

fn decode_project_audit_event(
    row: StoredProjectAuditEvent,
) -> Result<ProjectAuditEvent, StoreError> {
    let event = ProjectAuditEvent {
        audit_event_id: row.audit_event_id,
        command_id: row.command_id,
        project_id: row.project_id,
        operation: ProjectAuditOperation::from_db(&row.operation)?,
        binding_generation: u64::try_from(row.binding_generation)
            .map_err(|_| StoreError::Invariant("génération audit projet invalide"))?,
        outcome: serde_json::from_slice(&row.outcome_json)
            .map_err(|_| StoreError::Invariant("issue audit projet corrompue"))?,
        previous_root_reference: row.previous_root_reference,
        observed_at: row.observed_at,
    };
    event.validate()?;
    Ok(event)
}

fn project_audit_projection_for_project(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<ProjectAuditProjection>, StoreError> {
    let stored = conn
        .query_row(
            "SELECT audit_event_id, command_id, project_id, operation,
                    binding_generation, outcome_json, previous_root_reference, observed_at
             FROM project_audit_events
             WHERE project_id = ?1
             ORDER BY observed_at DESC, audit_event_id DESC
             LIMIT 1",
            [project_id],
            project_audit_event_from_row,
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    let Some(event) = stored.map(decode_project_audit_event).transpose()? else {
        return Ok(None);
    };
    let operation = match event.operation {
        ProjectAuditOperation::Register => ProjectAuditOperationKind::Register,
        ProjectAuditOperation::Rebind => ProjectAuditOperationKind::Rebind,
        ProjectAuditOperation::Activate => ProjectAuditOperationKind::Activate,
        ProjectAuditOperation::Disable => ProjectAuditOperationKind::Disable,
        ProjectAuditOperation::ReviewProjectReconcile => {
            ProjectAuditOperationKind::ReviewProjectReconcile
        }
    };
    let (outcome, reason) = match event.outcome {
        ProjectAuditOutcome::Applied => (ProjectAuditOutcomeKind::Applied, None),
        ProjectAuditOutcome::Refused { reason } => (ProjectAuditOutcomeKind::Refused, Some(reason)),
    };
    Ok(Some(ProjectAuditProjection {
        operation,
        outcome,
        binding_generation: event.binding_generation,
        reason,
        observed_at: event.observed_at,
    }))
}

fn project_audit_event_for_id(
    conn: &Connection,
    audit_event_id: &str,
) -> Result<Option<ProjectAuditEvent>, StoreError> {
    let stored = conn
        .query_row(
            "SELECT audit_event_id, command_id, project_id, operation,
                    binding_generation, outcome_json, previous_root_reference, observed_at
             FROM project_audit_events WHERE audit_event_id = ?1",
            [audit_event_id],
            project_audit_event_from_row,
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    stored.map(decode_project_audit_event).transpose()
}

fn record_project_audit_event_in_transaction(
    connection: &Connection,
    event: &ProjectAuditEvent,
) -> Result<bool, StoreError> {
    event.validate()?;
    let outcome = serde_json::to_vec(&event.outcome)
        .map_err(|_| StoreError::Invariant("issue audit projet non sérialisable"))?;
    let changed = connection
        .execute(
            "INSERT INTO project_audit_events (
                 audit_event_id, command_id, project_id, operation,
                 binding_generation, outcome_json, previous_root_reference, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(audit_event_id) DO NOTHING",
            params![
                event.audit_event_id,
                event.command_id,
                event.project_id,
                event.operation.as_db(),
                event.binding_generation as i64,
                outcome,
                event.previous_root_reference,
                event.observed_at,
            ],
        )
        .map_err(StoreError::Sqlite)?;
    if changed == 0 {
        let existing = project_audit_event_for_id(connection, &event.audit_event_id)?
            .ok_or(StoreError::Invariant("audit projet absent après conflit"))?;
        if existing != *event {
            return Err(StoreError::Invariant("collision audit projet divergente"));
        }
    }
    Ok(changed == 1)
}

fn project_audit_event_id(
    command_id: &str,
    operation: ProjectAuditOperation,
    binding_generation: u64,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"bridget/project-audit-event/v1");
    for field in [command_id.as_bytes(), operation.as_db().as_bytes()] {
        digest.update((field.len() as u64).to_be_bytes());
        digest.update(field);
    }
    digest.update(binding_generation.to_be_bytes());
    format!("sha256:{:x}", digest.finalize())
}

fn project_root_reference(root: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"bridget/project-root-reference/v1");
    digest.update((root.len() as u64).to_be_bytes());
    digest.update(root.as_bytes());
    format!("sha256:{:x}", digest.finalize())
}

fn is_sha256_reference(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
