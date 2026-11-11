//! Lectures communes au worker vivant et au diagnostic local. Aucun store écrivain.
use super::*;
use std::cell::{Cell, RefCell};

/// Un relevé par lot, commandes fermées hors du dépôt. Aucun stdout/stderr
/// retenu ; le résultat ne contient que l'outil et son état constaté.
pub(crate) fn qualify_workload_tools(
    source: &crate::lifecycle::SourceEnvironment,
    tools: &[bridget_transport::protocol::WorkloadTool],
    forbidden_roots: &[PathBuf],
    deadline: Instant,
) -> Vec<bridget_transport::protocol::WorkloadToolObservation> {
    use bridget_transport::protocol::{
        WorkloadTool as T, WorkloadToolObservation, WorkloadToolState as S,
    };
    tools
        .iter()
        .map(|tool| {
            let (name, args): (&str, &[&str]) = match tool {
                T::Git => ("git", &["--version"]),
                T::Cargo => ("cargo", &["--version"]),
                T::Node => ("node", &["--version"]),
                T::Python => ("python3", &["--version"]),
                T::Npm => ("npm", &["--version"]),
                T::Pnpm => ("pnpm", &["--version"]),
                T::Make => ("make", &["--version"]),
                T::Docker => ("docker", &["version", "--format", "{{.Server.Version}}"]),
                T::DockerCompose => ("docker", &["compose", "version"]),
            };
            let state = (|| {
                if Instant::now() >= deadline {
                    return S::Unknown;
                }
                let Some(path) = source
                    .get("PATH")
                    .and_then(|p| p.to_str())
                    .into_iter()
                    .flat_map(|p| p.split(':'))
                    .map(Path::new)
                    .filter(|p| p.is_absolute())
                    .map(|p| p.join(name))
                    .filter_map(|p| p.canonicalize().ok())
                    .filter(|p| !forbidden_roots.iter().any(|root| p.starts_with(root)))
                    .find(|p| {
                        p.metadata()
                            .is_ok_and(|m| m.is_file() && m.mode() & 0o111 != 0)
                    })
                else {
                    return S::Absent;
                };
                let mut child = match std::process::Command::new(path)
                    .args(args)
                    .current_dir("/")
                    .env_clear()
                    .envs(source)
                    // Les shims peuvent installer une version sur un simple --version.
                    // La qualification observe l'installation existante seulement.
                    .env("RUSTUP_AUTO_INSTALL", "0")
                    .env("COREPACK_ENABLE_NETWORK", "0")
                    .env("COREPACK_DEFAULT_TO_LATEST", "0")
                    .env("COREPACK_ENV_FILE", "0")
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(_) => return S::Unusable,
                };
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            return if status.success() {
                                S::Available
                            } else {
                                S::Unusable
                            };
                        }
                        Ok(None) if Instant::now() < deadline => {
                            std::thread::sleep(Duration::from_millis(5))
                        }
                        _ => {
                            let _ = crate::workspace_git::stop_git(&mut child);
                            return S::Unknown;
                        }
                    }
                }
            })();
            WorkloadToolObservation { tool: *tool, state }
        })
        .collect()
}

pub(super) struct WorkerReadView<'a> {
    pub(super) config: &'a WorkerRuntimeConfig,
    pub(super) identity: &'a WorkerIdentity,
    pub(super) credential: &'a IdentityCredential,
    pub(super) source: &'a crate::lifecycle::SourceEnvironment,
    pub(super) quiet: bool,
}
#[derive(Default)]
pub(super) struct ReadTrace {
    stage: Cell<&'static str>,
    cause: RefCell<Option<String>>,
}
impl ReadTrace {
    fn at(&self, stage: &'static str) {
        self.stage.set(stage);
    }
    fn permission(&self, code: &str) {
        *self.cause.borrow_mut() = Some(permission_diagnostic_code(code).to_owned());
    }
}
pub(super) fn inputs(
    config: &WorkerRuntimeConfig,
) -> Result<
    (
        WorkerIdentity,
        IdentityCredential,
        crate::lifecycle::SourceEnvironment,
    ),
    String,
> {
    inputs_observed(config, false, crate::lifecycle::source_environment())
}
fn inputs_observed(
    config: &WorkerRuntimeConfig,
    quiet: bool,
    mut source: crate::lifecycle::SourceEnvironment,
) -> Result<
    (
        WorkerIdentity,
        IdentityCredential,
        crate::lifecycle::SourceEnvironment,
    ),
    String,
> {
    let identity = WorkerIdentity {
        machine_id: config.machine_id.clone(),
        worker_id: config.worker_id.clone(),
        provisioning_revision: config.provisioning_revision,
    };
    identity.validate().map_err(|_| "worker_identity_invalid")?;
    for path in [
        &config.credential_path,
        &config.state_root,
        &config.registry_path,
        &config.master_socket,
    ] {
        if !path.is_absolute() {
            return Err("worker_path_not_absolute".into());
        }
    }
    check_private(&config.state_root, true).map_err(|e| e.to_string())?;
    check_private(
        config
            .credential_path
            .parent()
            .ok_or("worker_credential_parent")?,
        true,
    )
    .map_err(|e| e.to_string())?;
    check_private(&config.credential_path, false).map_err(|e| e.to_string())?;
    let mut secret = String::new();
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(&config.credential_path)
        .map_err(|_| "worker_credential_unavailable")?
        .take(4097)
        .read_to_string(&mut secret)
        .map_err(|_| "worker_credential_invalid")?;
    if secret.len() > 4096 || secret.trim().is_empty() {
        return Err("worker_credential_invalid".into());
    }
    let credential = IdentityCredential::new(secret.trim_end_matches(['\r', '\n']).to_owned());
    let registry = if quiet {
        AgentRegistry::load_from_path_quiet(config.registry_path.clone())
    } else {
        AgentRegistry::load_from_path(config.registry_path.clone())
    }?;
    let provision = registry
        .machines()
        .get(&identity.machine_id)
        .ok_or("worker_machine_unknown")?;
    if provision.worker_id != identity.worker_id
        || provision.provisioning_revision != identity.provisioning_revision
        || !provision.matches_credential(&credential)
    {
        return Err("worker_provisioning_mismatch".into());
    }
    let namespace = crate::environment::Namespace::resolve(
        config.master_socket.parent().map(Path::to_path_buf),
        Some(config.master_socket.clone()),
        source.get("HOME").map(PathBuf::from),
    )?;
    for (name, value) in namespace.child_environment() {
        source.insert(name.to_string_lossy().into_owned(), value);
    }
    Ok((identity, credential, source))
}
impl WorkerReadView<'_> {
    pub(super) fn registry(&self) -> Result<AgentRegistry, WorkerRefusal> {
        let registry = if self.quiet {
            AgentRegistry::load_from_path_quiet(self.config.registry_path.clone())
        } else {
            AgentRegistry::load_from_path(self.config.registry_path.clone())
        }
        .map_err(|_| WorkerRefusal::Unauthorized)?;
        let provision = registry
            .machines()
            .get(&self.identity.machine_id)
            .ok_or(WorkerRefusal::Unauthorized)?;
        if provision.worker_id != self.identity.worker_id
            || provision.provisioning_revision != self.identity.provisioning_revision
            || !provision.matches_credential(self.credential)
        {
            return Err(WorkerRefusal::Unauthorized);
        }
        Ok(registry)
    }
    pub(super) fn protected(&self) -> Vec<PathBuf> {
        vec![
            self.config.credential_path.clone(),
            self.config.credential_path.parent().unwrap().into(),
            self.config.state_root.clone(),
        ]
    }
    pub(super) fn mapping_for(
        &self,
        request: &WorkerPrepare,
        registry: &AgentRegistry,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        let mapping = self.mapping_authority(request, registry)?;
        self.mapping_paths_and_git(request, mapping, workload)
    }
    pub(super) fn mapping_authority(
        &self,
        request: &WorkerPrepare,
        registry: &AgentRegistry,
    ) -> Result<ProjectMapping, WorkerRefusal> {
        let provision = registry
            .machine(&self.identity.machine_id)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if request.machine_id != self.identity.machine_id
            || request.provisioning_revision != self.identity.provisioning_revision
            || !provision.permits_agent_type(&request.agent_type)
        {
            return Err(WorkerRefusal::Unauthorized);
        }
        let mapping = registry
            .project_mapping(&request.machine_id, &request.project_id)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?
            .for_origin(&request.machine_id, &request.origin_host)
            .ok_or(WorkerRefusal::PlacementMismatch)?;
        if mapping.mapping_revision != request.mapping_revision
            || mapping.source_host != request.origin_host
            || mapping.source_root != Path::new(&request.source_root)
            || mapping.remote_root != Path::new(&request.remote_root)
            || mapping.expected_git_identity != request.source_root_identity
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        // Le côté origine reste une suite de composants approuvée. Aucun stat
        // ni canonicalize sur un chemin appartenant à l'autre machine.
        let origin = mapping.source_root.join(&request.relative_cwd);
        if origin != Path::new(&request.origin_cwd) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok(mapping)
    }
    /// Chemins seuls : les contrôles Git C19 sont exécutés par le helper
    /// possédé. Ne pas transformer une erreur Git en absence de dépôt.
    pub(super) fn mapping_paths(
        &self,
        request: &WorkerPrepare,
        mapping: ProjectMapping,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        let protected = self.protected();
        let root =
            crate::registry::validate_unprotected_local_path(&mapping.remote_root, &protected)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        let cwd = crate::registry::validate_unprotected_local_path(
            &root.join(&request.relative_cwd),
            &protected,
        )
        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !cwd.starts_with(&root) || !cwd.is_dir() {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok((mapping, cwd))
    }
    pub(super) fn mapping_paths_and_git(
        &self,
        request: &WorkerPrepare,
        mapping: ProjectMapping,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        let (mapping, cwd) = self.mapping_paths(request, mapping)?;
        let root = mapping.remote_root.clone();
        if let Some(workload) = workload {
            let git = Self::git_markers_present(&cwd)?;
            let worktree = matches!(
                &request.workspace,
                Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
            );
            if !git {
                if request.base_commit.is_some()
                    || worktree
                    || workload == bridget_transport::protocol::AutoDelegateWorkload::Compilation
                {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
                // HorsGit attesté : l'autorité est le mapping explicite et les
                // mêmes chemins/protections/droits, jamais un échec Git supposé.
                return Ok((mapping, cwd));
            }
            if request.base_commit.is_none() {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        let origin = crate::workspace_git::run_git(
            &root,
            &[
                "config",
                "--local",
                "--no-includes",
                "--get",
                "remote.origin.url",
            ],
            crate::workspace_git::READ_TIMEOUT,
        )
        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !origin.success()
            || origin.truncated
            || origin.stdout.trim_end_matches(['\r', '\n']) != mapping.expected_git_identity
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok((mapping, cwd))
    }
    pub(super) fn git_markers_present(cwd: &Path) -> Result<bool, WorkerRefusal> {
        for (depth, ancestor) in cwd.ancestors().enumerate() {
            if depth >= 256 {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            match fs::symlink_metadata(ancestor.join(".git")) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
                Ok(metadata) if metadata.is_file() || metadata.is_dir() => return Ok(true),
                Ok(_) => return Err(WorkerRefusal::PlacementMismatch),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => return Err(WorkerRefusal::PlacementMismatch),
            }
            let mut bare = 0;
            for name in ["HEAD", "objects", "refs"] {
                match fs::symlink_metadata(ancestor.join(name)) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        return Err(WorkerRefusal::PlacementMismatch);
                    }
                    Ok(_) => bare += 1,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(_) => return Err(WorkerRefusal::PlacementMismatch),
                }
            }
            if bare == 3 {
                return Ok(true);
            }
            if bare != 0 {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        Ok(false)
    }
    pub(super) fn check_routing_connection(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        routing: &RoutingPreparation,
    ) -> Result<(), WorkerRefusal> {
        let policy = registry
            .routing_policy()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        if !policy.enabled
            || !policy
                .machine_policies
                .get(&self.identity.machine_id)
                .is_some_and(|policy| policy.enabled)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        let candidates = registry
            .routing_candidates()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        if !candidates.iter().any(|entry| {
            entry.enabled
                && entry.machine_id == self.identity.machine_id
                && entry.connection_id == routing.connection_id
                && entry.engine == routing.engine
                && entry.agent_type == request.agent_type
                && entry.model == request.model
                && request
                    .service_tier
                    .as_ref()
                    .is_none_or(|tier| entry.service_tiers.contains(tier))
                && request
                    .effort
                    .as_ref()
                    .map_or(entry.default_effort.is_none(), |effort| {
                        entry.supported_efforts.contains(effort)
                    })
        }) {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        Ok(())
    }
    pub(super) fn remap_prepared_worktree(
        &self,
        mapping: &mut ProjectMapping,
        cwd: &Path,
        record: &crate::delegation::WorkspaceRecord,
    ) -> Result<PathBuf, WorkerRefusal> {
        let top = Path::new(&record.toplevel);
        let next;
        if let Ok(offset) = mapping.remote_root.strip_prefix(top) {
            next = Path::new(&record.path).join(
                cwd.strip_prefix(top)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?,
            );
            mapping.remote_root = Path::new(&record.path).join(offset);
        } else {
            top.strip_prefix(&mapping.remote_root)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            next = Path::new(&record.path).join(
                cwd.strip_prefix(top)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?,
            );
            // Un worktree contenu conserve la correspondance projet attestée.
        }
        let next = crate::registry::validate_unprotected_local_path(&next, &self.protected())
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !next.starts_with(&mapping.remote_root) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok(next)
    }
    pub(super) fn recheck_at_with_repair(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
        repair: Option<&crate::workspace_git::RepairAnchor>,
        trace: &ReadTrace,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at_with_workspace(prepared, before_release, repair, trace, &mut None)
    }

    /// Le rapport appartient à cet appel seulement. Il est produit depuis le reçu
    /// authentifié, même si une vérification ultérieure des droits refuse.
    pub(super) fn recheck_at_with_workspace(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
        repair: Option<&crate::workspace_git::RepairAnchor>,
        trace: &ReadTrace,
        workspace_report: &mut Option<Result<crate::delegation::WorkspaceRecord, WorkerRefusal>>,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at_with_view_report(
            prepared,
            before_release,
            repair,
            trace,
            workspace_report,
            None,
        )
    }

    /// Le rapport fourni est le résultat consommé d'un lecteur privé possédé
    /// par le runtime. Il ne remplace aucune autorité ni relecture de droits.
    pub(super) fn recheck_at_with_view_report(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
        repair: Option<&crate::workspace_git::RepairAnchor>,
        trace: &ReadTrace,
        workspace_report: &mut Option<Result<crate::delegation::WorkspaceRecord, WorkerRefusal>>,
        view_report: Option<&crate::delegation::WorkspaceRecord>,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        *workspace_report = None;
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        trace.at("registry");
        let registry = self
            .registry()?
            .pin_delegation_service_tier(
                &prepared.request.agent_type,
                &prepared.request.model,
                prepared.request.service_tier.as_deref(),
            )
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        if let Some(routing) = &prepared.routing {
            trace.at("routing");
            self.check_routing_connection(&registry, &prepared.request, routing)?;
        }
        trace.at("mapping");
        let (mut mapping, mut cwd) = self.mapping_for(
            &prepared.request,
            &registry,
            prepared.routing.as_ref().map(|routing| routing.workload),
        )?;
        let is_view = matches!(
            prepared.request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        );
        if is_view
            || matches!(
                &prepared.request.workspace,
                Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
            )
        {
            trace.at("workspace_receipt");
            let receipt = proof::read_workspace_receipt(
                &self.config.state_root.join("store"),
                self.identity,
                &prepared.command,
            )
            .map_err(refuse)?
            .ok_or(WorkerRefusal::UnknownExecution)?;
            let expected_digest = if receipt.fenced_previous.is_some() {
                fenced_prepare_digest(&prepared.command, &prepared.request)
            } else {
                json_digest(&(&prepared.command, &prepared.request))
            }
            .map_err(refuse)?;
            if receipt.worker != *self.identity
                || receipt.command != prepared.command
                || receipt.prepare_digest != expected_digest
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            let mut view_anchor = None;
            if is_view {
                if repair.is_some()
                    || prepared.prepared.effective_posture != SpawnPosture::Discovery
                {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
                let anchor = view_anchor_receipt_at(
                    &self.config.state_root.join("store"),
                    self.identity,
                    &prepared.command,
                    Some(&prepared.request),
                )
                .map_err(refuse)?;
                let view = anchor
                    .commit_view
                    .as_ref()
                    .ok_or(WorkerRefusal::CommandMismatch)?;
                if view.closed {
                    return Err(WorkerRefusal::InvalidEnvelope);
                }
                if !view.source_stopped
                    || view.source_marker.is_none()
                    || view
                        .materialization
                        .as_ref()
                        .is_some_and(|operation| !operation.stopped)
                {
                    return Err(WorkerRefusal::UnknownExecution);
                }
                view_anchor = Some(anchor);
            }
            trace.at("workspace_report");
            let original = match (receipt.record, receipt.fenced_previous) {
                (Some(record), None) => record,
                (None, Some(_)) => original_workspace_at(
                    &self.config.state_root.join("store"),
                    self.identity,
                    &prepared.command,
                )
                .map_err(refuse_fenced_workspace)?,
                _ => return Err(WorkerRefusal::CommandMismatch),
            };
            // Sous réparation185, le prepared porte légitimement le nouveau chemin ;
            // report_repaired_workspace valide déjà l'ancre contre l'original.
            if !prepared
                .prepared
                .workspace
                .as_ref()
                .is_some_and(|value| repair.is_some() || same_workspace_anchor(&original, value))
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            let mut view_intent = None;
            if is_view {
                let reference = original
                    .commit_view
                    .as_ref()
                    .ok_or(WorkerRefusal::CommandMismatch)?;
                let root = self
                    .config
                    .state_root
                    .join("store")
                    .join("view-manifests")
                    .canonicalize()
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                if !matches!(
                    original.strategy,
                    crate::delegation::WorkspaceStrategy::CommitView
                ) || reference.binding.worker != *self.identity
                    || reference.binding.command != view_anchor.as_ref().unwrap().command
                    || reference.binding.prepare_digest
                        != view_anchor.as_ref().unwrap().prepare_digest
                    || Path::new(&reference.proof_root) != root
                    || Path::new(&original.toplevel) != cwd
                    || prepared.request.base_commit.as_deref()
                        != Some(original.base_commit.as_str())
                {
                    return Err(WorkerRefusal::CommandMismatch);
                }
                let proofs = crate::workspace_git::ViewProofs::open(&root)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                let intent = proofs
                    .load(
                        &reference
                            .binding
                            .key()
                            .map_err(|_| WorkerRefusal::CommandMismatch)?,
                    )
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                if intent
                    .proof_digest()
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?
                    != reference.intent_digest
                {
                    return Err(WorkerRefusal::CommandMismatch);
                }
                view_intent = Some(intent);
            } else if !matches!(
                original.strategy,
                crate::delegation::WorkspaceStrategy::Worktree
            ) || original.commit_view.is_some()
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            let report = if let Some(report) = view_report {
                if !is_view
                    || repair.is_some()
                    || !same_workspace_anchor(
                        &original,
                        &serde_json::to_value(report)
                            .map_err(|_| WorkerRefusal::PlacementMismatch)?,
                    )
                {
                    return Err(WorkerRefusal::CommandMismatch);
                }
                Ok(report.clone())
            } else if let Some(anchor) = repair {
                crate::workspace_git::report_repaired_workspace(&original, anchor)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)
            } else {
                crate::workspace_git::report(&original)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)
            };
            // Some(Err) garde aussi la tentative refusée : aucun second rapport
            // n'est lancé à l'intérieur de ce même contrôle après un échec Git.
            *workspace_report = Some(report.clone());
            let report = report?;
            let record = if repair.is_some() { &report } else { &original };
            if before_release
                && if is_view {
                    report.dirty != Some(false)
                } else {
                    report.head_commit.as_ref() != Some(&record.base_commit)
                        || report.head_on_branch != Some(true)
                }
            {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            trace.at("workspace_mapping");
            if let Some(intent) = view_intent {
                let (root, next) = intent
                    .execution_paths(&mapping.remote_root)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                let protected = self.protected();
                mapping.remote_root =
                    crate::registry::validate_unprotected_local_path(&root, &protected)
                        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                cwd = crate::registry::validate_unprotected_local_path(&next, &protected)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                if !cwd.starts_with(&mapping.remote_root) {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
            } else {
                cwd = self.remap_prepared_worktree(&mut mapping, &cwd, record)?;
            }
        } else if before_release
            && let Some(base) = prepared.prepared.placement.base_commit.as_deref()
        {
            let observed = crate::workspace_git::run_git(
                &cwd,
                &["rev-parse", "--verify", "--end-of-options", "HEAD^{commit}"],
                crate::workspace_git::READ_TIMEOUT,
            )
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if !observed.success() || observed.truncated || observed.stdout.trim() != base {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        trace.at("execution_cwd");
        if cwd != Path::new(&prepared.prepared.placement.execution_cwd) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        trace.at("definition_constructor");
        let scoped = registry
            .for_inherited_delegation_in_operation(
                &prepared.request.agent_type,
                &prepared.request.model,
                prepared.request.effort.as_deref(),
                &prepared.prepared.child_policy,
                &mut operation,
            )
            .map_err(|code| {
                trace.permission(&code);
                WorkerRefusal::DefinitionChanged
            })?;
        trace.at("definition_identity");
        if scoped
            .resolved_definition(&prepared.request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?
            != prepared.prepared.definition
        {
            return Err(WorkerRefusal::DefinitionChanged);
        }
        trace.at("launch_preflight");
        crate::lifecycle::launch_preflight(&scoped, &prepared.request.agent_type, self.source)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        trace.at("child_permissions");
        crate::native_permissions::recheck_remote_child_permissions_in_operation(
            &prepared.prepared.child_policy,
            scoped
                .get(&prepared.request.agent_type)
                .map_err(|_| WorkerRefusal::DefinitionChanged)?,
            &cwd,
            self.source,
            &self.protected(),
            Some(&mapping),
            Some(&prepared.request.parent),
            &mut operation,
        )
        .map_err(|code| {
            trace.permission(&code);
            WorkerRefusal::PermissionsChanged
        })?;
        trace.at("input_consistency");
        operation.finish().map_err(|code| {
            trace.permission(&code);
            WorkerRefusal::PermissionsChanged
        })?;
        trace.at("complete");
        Ok(scoped)
    }
}

/// Observation locale seulement : aucun WorkerStore écrivain ni runtime vivant.
/// Le même environnement que le worker est requis pour interpréter les refus.
#[cfg(test)]
pub(crate) fn diagnose_local(
    config: &WorkerRuntimeConfig,
    command: &WorkerCommandIdentity,
) -> Result<serde_json::Value, String> {
    diagnose_local_observed(config, command, || {})
}

#[cfg(any(test, target_os = "linux"))]
fn diagnostic_paths(
    config: &WorkerRuntimeConfig,
    command: &WorkerCommandIdentity,
) -> Result<Vec<PathBuf>, String> {
    command
        .validate()
        .map_err(|_| "diagnostic_identity_invalid")?;
    for p in [
        &config.state_root,
        &config.credential_path,
        &config.registry_path,
        &config.master_socket,
    ] {
        if !p.is_absolute()
            || p.components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err("diagnostic_context_unavailable".into());
        }
    }
    let root = config.state_root.join("store");
    let enrollment = config.state_root.join(format!(
        ".worker-{}.enrolled",
        digest_bytes(root.as_os_str().as_encoded_bytes())
    ));
    Ok(vec![
        root.join("commands")
            .join(format!("{}.json", marker_key(command))),
        root.join("workspaces")
            .join(format!("{}.json", marker_key(command))),
        enrollment,
        config.registry_path.clone(),
        config.credential_path.clone(),
    ])
}
#[cfg(any(test, target_os = "linux"))]
#[derive(PartialEq, Eq)]
struct ReadStamp {
    dev: u64,
    ino: u64,
    len: u64,
    mode: u32,
    mtime: (i64, i64),
    ctime: (i64, i64),
}
#[cfg(any(test, target_os = "linux"))]
fn stamp(path: &Path) -> Result<Option<ReadStamp>, String> {
    match fs::symlink_metadata(path) {
        Ok(m) => Ok(Some(ReadStamp {
            dev: m.dev(),
            ino: m.ino(),
            len: m.len(),
            mode: m.mode(),
            mtime: (m.mtime(), m.mtime_nsec()),
            ctime: (m.ctime(), m.ctime_nsec()),
        })),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("diagnostic_file_unavailable".into()),
    }
}
#[cfg(any(test, target_os = "linux"))]
#[allow(clippy::unnecessary_cast)] // libc::S_IFMT est un u16 sous macOS et un u32 sous Linux
fn observe_stably(
    paths: &[PathBuf],
    read: impl FnOnce() -> Result<serde_json::Value, String>,
    after_check: impl FnOnce(),
) -> Result<serde_json::Value, String> {
    let before = paths
        .iter()
        .map(|p| stamp(p))
        .collect::<Result<Vec<_>, _>>()?;
    let outcome = if before
        .iter()
        .flatten()
        .any(|m| u64::from(m.mode) & u64::from(libc::S_IFMT) != u64::from(libc::S_IFREG))
    {
        Err("diagnostic_file_type_invalid".into())
    } else {
        read()
    };
    after_check();
    let after = paths
        .iter()
        .map(|p| stamp(p))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "diagnostic_stability_unavailable")?;
    if before != after {
        return Err("diagnostic_observation_changed".into());
    }
    outcome
}
#[cfg(test)]
pub(super) fn diagnose_local_observed(
    config: &WorkerRuntimeConfig,
    command: &WorkerCommandIdentity,
    after_check: impl FnOnce(),
) -> Result<serde_json::Value, String> {
    diagnose_with_source(
        config,
        command,
        crate::lifecycle::source_environment(),
        after_check,
    )
}
#[cfg(any(test, target_os = "linux"))]
fn diagnose_with_source(
    config: &WorkerRuntimeConfig,
    command: &WorkerCommandIdentity,
    source: crate::lifecycle::SourceEnvironment,
    after_check: impl FnOnce(),
) -> Result<serde_json::Value, String> {
    let paths = diagnostic_paths(config, command)?;
    observe_stably(
        &paths,
        || {
            let (identity, credential, source) = inputs_observed(config, true, source)
                .map_err(|_| "diagnostic_context_unavailable")?;
            diagnose_inner(
                WorkerReadView {
                    config,
                    identity: &identity,
                    credential: &credential,
                    source: &source,
                    quiet: true,
                },
                command,
            )
        },
        after_check,
    )
}
#[cfg(test)]
pub(super) fn diagnose_observed(
    view: WorkerReadView<'_>,
    command: &WorkerCommandIdentity,
    after_check: impl FnOnce(),
) -> Result<serde_json::Value, String> {
    let paths = diagnostic_paths(view.config, command)?;
    observe_stably(&paths, || diagnose_inner(view, command), after_check)
}
#[cfg(any(test, target_os = "linux"))]
fn diagnose_inner(
    view: WorkerReadView<'_>,
    command: &WorkerCommandIdentity,
) -> Result<serde_json::Value, String> {
    let config = view.config;
    let identity = view.identity;
    let paths = diagnostic_paths(config, command)?;
    let path = &paths[0];
    let enrollment = &paths[2];
    let root = config.state_root.join("store");
    check_private(&root, true).map_err(|_| "diagnostic_store_unavailable")?;
    let enrolled: Enrollment = read_optional(enrollment, MAX_METADATA_BYTES)
        .map_err(|_| "diagnostic_enrollment_invalid")?
        .ok_or("diagnostic_enrollment_unavailable")?;
    if enrolled.version != STORE_VERSION || enrolled.worker != *identity || enrolled.quarantined {
        return Err("diagnostic_enrollment_invalid".into());
    }
    let receipt: WorkerCommandReceipt = read_optional(path, MAX_METADATA_BYTES)
        .map_err(|_| "diagnostic_receipt_invalid")?
        .ok_or("diagnostic_receipt_unavailable")?;
    if receipt.version != STORE_VERSION
        || receipt.worker != *identity
        || receipt.identity != *command
        || !receipt.placement.matches_worker(identity)
    {
        return Err("diagnostic_identity_mismatch".into());
    }
    let trace = ReadTrace::default();
    trace.at("preparation");
    let outcome = (|| -> Result<(), WorkerRefusal> {
        let (request, prepared) = receipt
            .preparation
            .clone()
            .ok_or(WorkerRefusal::UnknownExecution)?;
        if prepared.placement != receipt.placement
            || json_digest(&prepared.child_policy).map_err(refuse)?
                != receipt.placement.permission_digest
        {
            return Err(WorkerRefusal::CommandMismatch);
        }
        if receipt.phase != WorkerCommandPhase::Released
            || !receipt.native_ready
            || receipt.startup_failed
            || receipt
                .workspace_repair
                .as_ref()
                .is_some_and(|r| r.blocks_dispatch())
        {
            trace.at("execution_not_ready");
            return Err(WorkerRefusal::UnknownExecution);
        }
        let mut provisional = command.clone();
        provisional.instance_id.clear();
        provisional.generation = 0;
        let mut retained = PreparedRuntime {
            routing: receipt.routing.clone(),
            command: provisional,
            request,
            prepared,
            expires: Instant::now(),
        };
        let repair = receipt
            .workspace_repair
            .as_ref()
            .filter(|r| r.phase == "published");
        let anchor = repair
            .map(|r| workspace_repair::proof(r).map(|p| p.0))
            .transpose()
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if let Some(prepared) = repair.and_then(|r| r.prepared.as_ref()) {
            retained.prepared = prepared.clone();
        }
        view.recheck_at_with_repair(&retained, false, anchor.as_ref(), &trace)?;
        Ok(())
    })();
    let reason = outcome.as_ref().err().map(|r| format!("{r:?}"));
    let recovery = match trace.stage.get() {
        "mapping" | "workspace_receipt" | "workspace_report" | "workspace_mapping"
        | "execution_cwd" => {
            "Vérifiez le mapping et l’ancrage Git de ce worktree dans le dépôt admis. Conservez les fichiers et le reçu ; un déplacement, une autre identité Git ou un lien ne répare pas l’autorité enregistrée."
        }
        "definition_constructor" | "definition_identity" | "launch_preflight" => {
            "Comparez la définition du moteur et du modèle configurés avec celle admise. Le diagnostic ne permet pas de choisir un autre moteur ou d’élargir les droits."
        }
        "child_permissions" | "input_consistency" => {
            "Vérifiez le code de cause fermé indiqué et les sources locales de la politique héritée. Conservez la tâche ; une modification légitime de cette politique exige le parcours natif prévu."
        }
        _ => {
            "Conservez cette tâche et ses fichiers. Comparez ce contrôle local avec le worker vivant, dans le même environnement. Ce diagnostic ne remet aucune mission et ne remplace pas l’attestation du maître."
        }
    };

    Ok(
        serde_json::json!({"version":1,"scope":"local_read_only_recheck","authoritative":false,
        "command":command,"worker":identity,"phase":receipt.phase,"native_ready":receipt.native_ready,
        "check":if outcome.is_ok(){"passed"}else{"refused"},"stage":trace.stage.get(),
        "reason":reason,"cause":trace.cause.borrow().clone(),"observed_at":worker_now(),
        "environment":"diagnostic_process",
        "recovery":recovery,"human_action":if outcome.is_err(){Some("Si la vérification dépasse votre périmètre, demandez au responsable du worker une décision sur les sources ou leur récupération, en conservant ce tuple et ce code.")}else{None}}),
    )
}

pub(crate) fn diagnostic_failure(code: &str, command: &WorkerCommandIdentity) -> serde_json::Value {
    let (message, recovery, human_action) = match code {
        "delegated_mcp_only" => (
            "Ce processus délégué ne peut pas utiliser le lecteur local du worker.",
            "Rendez le besoin de diagnostic au parent par sa façade native, en conservant la tâche et sa clé.",
            "Le parent ou le responsable du worker doit prendre en charge ce contrôle dans son propre périmètre.",
        ),
        "namespace_configuration_invalid"
        | "identity_not_found"
        | "legacy_marker"
        | "t3_session_unavailable" => (
            "Le contexte du lecteur n’est pas résolu de manière sûre.",
            "Vérifiez le namespace configuré et les preuves natives de ce processus. Conservez les identités d’origine.",
            "Si ce rattachement ne peut pas être vérifié, demandez au responsable de le rétablir.",
        ),
        "diagnostic_worker_binding_unsupported" => (
            "Le rattachement du lecteur au worker vivant requiert Linux.",
            "Exécutez ce diagnostic sur l’hôte Linux du worker, avec ses options exactes.",
            "Pour un autre système, faites qualifier un parcours adapté par le responsable.",
        ),
        "diagnostic_worker_binding_unavailable"
        | "diagnostic_worker_binding_mismatch"
        | "diagnostic_worker_binding_ambiguous"
        | "diagnostic_worker_binding_changed" => (
            "Le détenteur vivant du store et ses options ne sont pas vérifiés de manière univoque.",
            "Vérifiez le worker Linux qui détient le verrou, son PID et ses sept options. Aucun reçu ni verrou ne doit être recréé pour ce contrôle.",
            "Si le processus ou ses informations noyau restent invérifiables, demandez une inspection au responsable ; aucune fin de tâche n’est déduite.",
        ),
        "diagnostic_tool_environment_mismatch" => (
            "Le lecteur et le worker ne sélectionnent pas leurs outils avec le même PATH.",
            "Faites vérifier le PATH du worker vivant par son responsable, puis utilisez ce seul réglage pour le lecteur. Ne recopiez aucune variable d’identité ou secret. Le contrôle métier n’a pas été exécuté.",
            "Si ce réglage ne peut pas être vérifié, demandez au responsable du worker d’effectuer le diagnostic dans son contexte d’outils.",
        ),
        "diagnostic_worker_environment_unavailable" | "diagnostic_worker_environment_changed" => (
            "L’environnement source du worker vivant est inaccessible ou instable.",
            "Vérifiez le processus détenteur du verrou et son accès procfs ; aucune variable d’identité du lecteur ne doit être remplacée.",
            "Si cette observation reste impossible, demandez une inspection au responsable du worker ; ce refus ne prouve pas une fin de tâche.",
        ),
        "diagnostic_identity_invalid" | "diagnostic_identity_mismatch" => (
            "Le tuple demandé ne correspond pas à un reçu valide de ce worker.",
            "Comparez commande, tâche, instance et génération avec l’admission d’origine ; conservez la clé et le reçu existants.",
            "Si les identités durables se contredisent, demandez au responsable du worker de résoudre ce rattachement.",
        ),
        "diagnostic_receipt_unavailable" => (
            "Le reçu de commande demandé est absent du store indiqué.",
            "Vérifiez le store et l’identité du worker d’origine. Ce manque ne prouve ni l’arrêt ni l’absence de lancement.",
            "Si le reçu demeure introuvable, demandez une décision de récupération au responsable ; conservez la tâche.",
        ),
        "diagnostic_observation_changed" => (
            "Un fichier observé a changé pendant le contrôle.",
            "Après la fin de l’opération qui écrit ces fichiers, relisez exactement le même tuple dans le même environnement.",
            "Si les changements persistent, faites examiner les écritures concurrentes par le responsable du worker.",
        ),
        "diagnostic_stability_unavailable" => (
            "La stabilité finale des fichiers n’a pas pu être vérifiée.",
            "Conservez le tuple et le code ; vérifiez l’accès aux fichiers avant une nouvelle observation.",
            "Si l’accès reste invérifiable, faites examiner le stockage par son responsable.",
        ),
        "diagnostic_file_type_invalid" => (
            "Une entrée attendue comme fichier ordinaire est d’un autre type.",
            "Inspectez les chemins de configuration et de reçus indiqués dans votre commande, sans remplacer leurs fichiers.",
            "Le responsable du stockage doit décider de la remise en état si cette entrée a été remplacée.",
        ),
        "diagnostic_context_unavailable" => (
            "La configuration locale du diagnostic n’a pas pu être vérifiée.",
            "Vérifiez l’identité provisionnée, les sept chemins/options du worker et son environnement réel. La cause détaillée n’est pas établie par ce refus.",
            "Si ces entrées ne sont pas accessibles dans votre périmètre, demandez leur vérification au responsable du worker.",
        ),
        _ => (
            "Les données locales requises ne permettent pas un diagnostic complet.",
            "Conservez ce code et le tuple. Vérifiez le store d’origine, son enrôlement et les fichiers privés ; la vérification manquante reste non résolue.",
            "Demandez au responsable du worker d’inspecter les données ou de décider une récupération conservant la tâche.",
        ),
    };
    let stage = match code {
        "diagnostic_identity_invalid" | "diagnostic_identity_mismatch" => "identity",
        "diagnostic_receipt_unavailable" | "diagnostic_receipt_invalid" => "receipt",
        "diagnostic_context_unavailable" => "context",
        "diagnostic_enrollment_invalid" | "diagnostic_enrollment_unavailable" => "enrollment",
        "namespace_configuration_invalid" | "delegated_mcp_only" | "identity_not_found" => "caller",
        "diagnostic_tool_environment_mismatch" => "tool_environment",
        code if code.starts_with("diagnostic_worker_environment_") => "worker_environment",
        code if code.starts_with("diagnostic_worker_binding_") => "worker_binding",
        _ => "observation",
    };
    let cause = match code {
        "diagnostic_receipt_unavailable" => "receipt_missing",
        "diagnostic_identity_mismatch" => "command_tuple_mismatch",
        "diagnostic_file_type_invalid" => "not_regular_file",
        "diagnostic_observation_changed" => "observed_files_changed",
        "namespace_configuration_invalid" => "namespace_invalid",
        "delegated_mcp_only" => "delegated_cli_denied",
        _ => "unknown",
    };
    serde_json::json!({"version":1,"status":"refused","code":code,"command":command,"stage":stage,
        "cause":cause,"environment":"diagnostic_process",
        "message":message,"recovery":recovery,"human_action":human_action,
        "scope":"local_read_only_recheck","authoritative":false})
}

#[cfg(target_os = "linux")]
fn live_environment(
    binding: &LiveWorkerBinding,
) -> Result<crate::lifecycle::SourceEnvironment, String> {
    use std::os::unix::ffi::OsStringExt;
    if crate::managed_process::process_birth(binding.pid).ok() != Some(binding.birth) {
        return Err("diagnostic_worker_binding_changed".into());
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(format!("/proc/{}/environ", binding.pid))
        .map_err(|_| "diagnostic_worker_environment_unavailable")?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "diagnostic_worker_environment_unavailable")?;
    if bytes.len() > 1024 * 1024 {
        return Err("diagnostic_worker_environment_unavailable".into());
    }
    let mut source = crate::lifecycle::SourceEnvironment::new();
    for field in bytes.split(|b| *b == 0).filter(|v| !v.is_empty()) {
        let i = field
            .iter()
            .position(|b| *b == b'=')
            .ok_or("diagnostic_worker_environment_unavailable")?;
        let name = std::str::from_utf8(&field[..i])
            .map_err(|_| "diagnostic_worker_environment_unavailable")?;
        source.insert(
            name.to_owned(),
            std::ffi::OsString::from_vec(field[i + 1..].to_vec()),
        );
    }
    Ok(source)
}
#[cfg(target_os = "linux")]
pub(crate) fn diagnose_live_local(
    config: &WorkerRuntimeConfig,
    command: &WorkerCommandIdentity,
    binding: &LiveWorkerBinding,
) -> Result<serde_json::Value, String> {
    if live_worker_binding(config).ok().as_ref() != Some(binding) {
        return Err("diagnostic_worker_binding_changed".into());
    }
    let environment = live_environment(binding)?;
    // Git est lancé par le contrôleur historique dans le processus lecteur.
    // Ne jamais présenter son refus comme comparable si PATH sélectionne
    // un autre outil. Aucune variable n'est modifiée pour franchir cette garde.
    if environment.get("PATH") != std::env::var_os("PATH").as_ref() {
        return Err("diagnostic_tool_environment_mismatch".into());
    }
    let outcome = diagnose_with_source(config, command, environment.clone(), || {});
    if live_worker_binding(config).ok().as_ref() != Some(binding) {
        return Err("diagnostic_worker_binding_changed".into());
    }
    if live_environment(binding)? != environment {
        return Err("diagnostic_worker_environment_changed".into());
    }
    outcome.map(|mut value| {
        value["source_environment"] = serde_json::json!("live_worker_observed");
        value["worker_pid"] = serde_json::json!(binding.pid);
        value["worker_birth"] = serde_json::json!(binding.birth);
        value
    })
}
#[cfg(not(target_os = "linux"))]
pub(crate) fn diagnose_live_local(
    _: &WorkerRuntimeConfig,
    _: &WorkerCommandIdentity,
    _: &LiveWorkerBinding,
) -> Result<serde_json::Value, String> {
    Err("diagnostic_worker_binding_unsupported".into())
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LiveWorkerBinding {
    pub(crate) pid: u32,
    pub(crate) birth: u64,
    pub(crate) namespace_root: PathBuf,
}
#[cfg(not(target_os = "linux"))]
pub(crate) fn live_worker_binding(
    _: &WorkerRuntimeConfig,
) -> Result<LiveWorkerBinding, &'static str> {
    Err("diagnostic_worker_binding_unsupported")
}
#[cfg(target_os = "linux")]
pub(crate) fn live_worker_binding(
    config: &WorkerRuntimeConfig,
) -> Result<LiveWorkerBinding, &'static str> {
    fn bounded(path: &Path, max: u64) -> Result<Vec<u8>, &'static str> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
            .open(path)
            .map_err(|_| "diagnostic_worker_binding_unavailable")?;
        let mut bytes = Vec::new();
        file.take(max + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "diagnostic_worker_binding_unavailable")?;
        if bytes.len() as u64 > max {
            return Err("diagnostic_worker_binding_unavailable");
        }
        Ok(bytes)
    }
    let root = config.state_root.join("store");
    let key = digest_bytes(root.as_os_str().as_encoded_bytes());
    let lock = config.state_root.join(format!(".worker-{key}.lock"));
    check_private(&config.state_root, true).map_err(|_| "diagnostic_worker_binding_unavailable")?;
    check_private(&lock, false).map_err(|_| "diagnostic_worker_binding_unavailable")?;
    let m = fs::symlink_metadata(&lock).map_err(|_| "diagnostic_worker_binding_unavailable")?;
    if !m.is_file() || m.nlink() != 1 {
        return Err("diagnostic_worker_binding_unavailable");
    }
    let locks = bounded(Path::new("/proc/locks"), 2 * 1024 * 1024)?;
    let locks = std::str::from_utf8(&locks).map_err(|_| "diagnostic_worker_binding_unavailable")?;
    let mut owner = None;
    for line in locks.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 6 || fields[1] != "FLOCK" || fields[3] != "WRITE" {
            continue;
        }
        let dev = fields[5].split(':').collect::<Vec<_>>();
        if dev.len() != 3
            || u64::from_str_radix(dev[0], 16).ok() != Some(libc::major(m.dev()) as u64)
            || u64::from_str_radix(dev[1], 16).ok() != Some(libc::minor(m.dev()) as u64)
            || dev[2].parse::<u64>().ok() != Some(m.ino())
        {
            continue;
        }
        let pid = fields[4]
            .parse::<u32>()
            .ok()
            .filter(|p| *p > 1)
            .ok_or("diagnostic_worker_binding_unavailable")?;
        if owner.replace(pid).is_some() {
            return Err("diagnostic_worker_binding_ambiguous");
        }
    }
    let pid = owner.ok_or("diagnostic_worker_binding_unavailable")?;
    let process = PathBuf::from(format!("/proc/{pid}"));
    if fs::metadata(&process)
        .map_err(|_| "diagnostic_worker_binding_unavailable")?
        .uid()
        != bridget_transport::fsutil::effective_uid()
    {
        return Err("diagnostic_worker_binding_unavailable");
    }
    let birth = crate::managed_process::process_birth(pid)
        .map_err(|_| "diagnostic_worker_binding_unavailable")?;
    let args = bounded(&process.join("cmdline"), 32768)?;
    let args = args
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .collect::<Vec<_>>();
    if args.len() != 16 || args[1] != b"worker" {
        return Err("diagnostic_worker_binding_mismatch");
    }
    let mut options = BTreeMap::new();
    for pair in args[2..].chunks_exact(2) {
        let key = std::str::from_utf8(pair[0]).map_err(|_| "diagnostic_worker_binding_mismatch")?;
        if options.insert(key, pair[1]).is_some() {
            return Err("diagnostic_worker_binding_mismatch");
        }
    }
    for (key, value) in [
        ("--machine-id", config.machine_id.as_bytes()),
        ("--worker-id", config.worker_id.as_bytes()),
        (
            "--credential-file",
            config.credential_path.as_os_str().as_encoded_bytes(),
        ),
        (
            "--state-root",
            config.state_root.as_os_str().as_encoded_bytes(),
        ),
        (
            "--registry-file",
            config.registry_path.as_os_str().as_encoded_bytes(),
        ),
        (
            "--master-socket",
            config.master_socket.as_os_str().as_encoded_bytes(),
        ),
    ] {
        if options.get(key).copied() != Some(value) {
            return Err("diagnostic_worker_binding_mismatch");
        }
    }
    if options
        .get("--provisioning-revision")
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse::<u64>().ok())
        != Some(config.provisioning_revision)
        || crate::managed_process::process_birth(pid).ok() != Some(birth)
    {
        return Err("diagnostic_worker_binding_mismatch");
    }
    Ok(LiveWorkerBinding {
        pid,
        birth,
        namespace_root: config
            .master_socket
            .parent()
            .ok_or("diagnostic_worker_binding_mismatch")?
            .to_path_buf(),
    })
}

impl WorkerReadView<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn inspect_candidate(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
        provider_accounts_negotiated: bool,
    ) -> Result<CandidateReadiness, WorkerRefusal> {
        if registry
            .get(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?
            .capabilities
            .account_profiles
            && !provider_accounts_negotiated
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) && (!crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED
            || request.posture != Some(SpawnPosture::Discovery))
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        self.readiness_at(registry, request, mapping, cwd, operation, diagnostics)
    }
    pub(super) fn readiness_at(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    ) -> Result<CandidateReadiness, WorkerRefusal> {
        let pinned = registry
            .clone()
            .pin_delegation_model(
                &request.agent_type,
                &request.model,
                request.effort.as_deref(),
            )
            .and_then(|registry| {
                registry.pin_delegation_service_tier(
                    &request.agent_type,
                    &request.model,
                    request.service_tier.as_deref(),
                )
            })
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        crate::lifecycle::launch_preflight(&pinned, &request.agent_type, self.source)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        let projection_definition = pinned
            .get(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        let (projection, projection_guard) =
            crate::native_permissions::observe_projection171(|| {
                crate::native_permissions::project_remote_permissions_in_operation(
                    &request.parent,
                    projection_definition,
                    mapping,
                    cwd,
                    self.source,
                    request.posture,
                    &self.protected(),
                    operation,
                    matches!(
                        request.workspace,
                        Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
                    ),
                    request.origin_host == self.identity.machine_id,
                )
            });
        let (mut policy, effective_posture) = projection.map_err(|code| {
            map_probe_projection_failure171(
                &code,
                diagnostics,
                projection_guard,
                &request.parent.provider_policy,
                |note| eprintln!("bridget_probe_permission_refusal {note}"),
            )
        })?;
        let scoped = pinned
            .for_inherited_delegation_in_operation(
                &request.agent_type,
                &request.model,
                request.effort.as_deref(),
                &policy,
                operation,
            )
            .map_err(|code| {
                observe_probe_permission_failure(
                    &code,
                    diagnostics,
                    ProbePermissionSite::ScopedConstructor,
                )
            })?;
        let definition = scoped
            .get(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        // Les arguments effectivement contrôlés sont capturés avant le digest
        // de droits. La capture reste une lecture de configuration locale.
        if definition.protocol == "claude_stream_json" {
            crate::native_permissions::refresh_remote_permissions_context_in_operation(
                &mut policy,
                definition,
                cwd,
                self.source,
                mapping,
                operation,
            )
            .map_err(|code| {
                observe_probe_permission_failure(&code, diagnostics, ProbePermissionSite::Refresh)
            })?;
        }
        crate::native_permissions::recheck_remote_child_permissions_in_operation(
            &policy,
            definition,
            cwd,
            self.source,
            &self.protected(),
            Some(mapping),
            Some(&request.parent),
            operation,
        )
        .map_err(|code| {
            observe_probe_permission_failure(&code, diagnostics, ProbePermissionSite::Recheck)
        })?;
        let definition = scoped
            .resolved_definition(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        Ok(CandidateReadiness {
            definition,
            child_policy: policy,
            effective_posture,
        })
    }
    pub(super) fn checkout_root(
        &self,
        mapping: &ProjectMapping,
        cwd: &Path,
    ) -> Result<PathBuf, WorkerRefusal> {
        let git_path = |dir: &Path, flag: &str| -> Result<PathBuf, WorkerRefusal> {
            let output = crate::workspace_git::run_git(
                dir,
                &["rev-parse", flag],
                crate::workspace_git::READ_TIMEOUT,
            )
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if !output.success() || output.truncated {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            let path = Path::new(output.stdout.trim_end_matches(['\r', '\n']));
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                dir.join(path)
            };
            fs::canonicalize(absolute).map_err(|_| WorkerRefusal::PlacementMismatch)
        };
        let root = git_path(cwd, "--show-toplevel")?;
        if (!root.starts_with(&mapping.remote_root) && !mapping.remote_root.starts_with(&root))
            || !cwd.starts_with(&root)
            || git_path(&root, "--git-common-dir")?
                != git_path(&mapping.remote_root, "--git-common-dir")?
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        // Une correspondance limitee a un sous-dossier ne donne pas le droit
        // de rafraichir le checkout entier : garder sa borne historique.
        let authorized = if root.starts_with(&mapping.remote_root) {
            &root
        } else {
            &mapping.remote_root
        };
        crate::registry::validate_unprotected_local_path(authorized, &self.protected())
            .map_err(|_| WorkerRefusal::PlacementMismatch)
    }
    pub(super) fn probe_workspace_preconditions(
        &self,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
        shared: impl FnOnce(&Path, &str) -> Result<(), CandidateProbeRefusal>,
    ) -> Result<(), CandidateProbeRefusal> {
        if matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) {
            if request.posture != Some(SpawnPosture::Discovery) {
                return Err(CandidateProbeRefusal::InvalidRequest);
            }
            let base = request
                .base_commit
                .as_deref()
                .ok_or(CandidateProbeRefusal::WorkspaceBaseUnavailable)?;
            // Lire A ne qualifie ni ne rafraîchit HEAD B. Le miroir local est
            // lu selon le contrat existant, sans fetch pendant la sonde.
            return match crate::workspace_git::remote_source_commit(cwd, Some(base)) {
                Ok(actual) if actual == base => Ok(()),
                Err(crate::workspace_git::WorkspacePreflightError::Refused(error))
                    if error == "workspace_base_unavailable: référence de base inconnue" =>
                {
                    Err(CandidateProbeRefusal::WorkspaceBaseUnavailable)
                }
                _ => Err(CandidateProbeRefusal::WorkspaceUnavailable),
            };
        }
        if let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        {
            let root = self
                .checkout_root(mapping, cwd)
                .map_err(|_| CandidateProbeRefusal::MappingUnavailable)?;
            let preflight = crate::workspace_git::preflight_remote(
                &root,
                base_ref.as_deref(),
                branch.as_deref(),
            )
            .map_err(|error| workspace_probe_refusal(&error))?;
            if request
                .base_commit
                .as_ref()
                .is_some_and(|expected| expected != &preflight.base_commit)
            {
                return Err(CandidateProbeRefusal::WorkspaceUnavailable);
            }
            return Ok(());
        }
        if let Some(base) = request.base_commit.as_deref() {
            let root = self
                .checkout_root(mapping, cwd)
                .map_err(|_| CandidateProbeRefusal::MappingUnavailable)?;
            shared(&root, base)?;
        }
        Ok(())
    }
}

/// Lecture isolée : aucun store écrivain, token Prepared, Child ou runtime.
/// L'occupation est une photographie prudente ; Prepare relit toujours le
/// registre, les droits et les gardes vivantes avant toute mutation.
pub(super) struct ProbeReadContext {
    pub(super) config: WorkerRuntimeConfig,
    pub(super) identity: WorkerIdentity,
    pub(super) credential: IdentityCredential,
    pub(super) source: crate::lifecycle::SourceEnvironment,
    pub(super) prepared: Vec<PathBuf>,
    pub(super) command_roots: RefCell<Option<Result<Vec<PathBuf>, CandidateProbeRefusal>>>,
    pub(super) provider_accounts_negotiated: bool,
    pub(super) permission_reasons_negotiated: bool,
    pub(super) host_resources: crate::host_resources::HostResourceCollector,
}
impl ProbeReadContext {
    fn read_view(&self) -> WorkerReadView<'_> {
        WorkerReadView {
            config: &self.config,
            identity: &self.identity,
            credential: &self.credential,
            source: &self.source,
            quiet: false,
        }
    }
    fn compilation_capacity(&self, registry: &AgentRegistry) -> Option<u8> {
        registry
            .routing_policy()?
            .machine_policies
            .get(&self.identity.machine_id)?
            .compilation_capacity
    }
    pub(super) fn active_workspace_roots(&self) -> Result<Vec<PathBuf>, CandidateProbeRefusal> {
        let mut cache = self.command_roots.borrow_mut();
        let commands = cache
            .get_or_insert_with(|| self.read_view().active_command_roots())
            .as_ref()
            .map_err(|r| *r)?;
        let mut paths = self.prepared.clone();
        paths.extend(commands.iter().cloned());
        Ok(paths)
    }
    pub(super) fn probe_workspace_preconditions(
        &self,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
    ) -> Result<(), CandidateProbeRefusal> {
        self.read_view()
            .probe_workspace_preconditions(request, mapping, cwd, |root, base| {
                if crate::workspace_git::shared_refresh_needed(root, base)? {
                    shared_paths_available(root, &self.active_workspace_roots()?)?;
                }
                Ok(())
            })
    }
    fn candidate_prepare(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        mapping_revision: u64,
    ) -> WorkerPrepare {
        WorkerPrepare {
            service_tier: None,
            machine_id: self.identity.machine_id.clone(),
            project_id: candidate.project_id.clone(),
            provisioning_revision: self.identity.provisioning_revision,
            // La révision n'est jamais fournie par le décideur. Elle provient
            // de la lecture fraîche du registre worker et est renvoyée dans le
            // verdict afin que le maître fige le fait réellement observé.
            mapping_revision,
            origin_host: candidate.origin_host.clone(),
            origin_cwd: candidate.origin_cwd.clone(),
            source_root: candidate.source_root.clone(),
            remote_root: candidate.remote_root.clone(),
            source_root_identity: candidate.source_root_identity.clone(),
            relative_cwd: candidate.relative_cwd.clone(),
            base_commit: candidate.base_commit.clone(),
            agent_type: candidate.agent_type.clone(),
            model: candidate.model.clone(),
            effort: candidate.effort.clone(),
            posture: candidate.posture,
            parent: probe.parent.clone(),
            workspace: candidate.workspace.clone(),
        }
    }

    fn probe_one_candidate_for(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        registry: &AgentRegistry,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
        command_write: Option<bool>,
        service_tier: Option<&str>,
        git_facts: Option<&mut ProbeGitFacts>,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    ) -> CandidateProbeVerdict {
        let mapping_revision =
            match registry.project_mapping(&self.identity.machine_id, &candidate.project_id) {
                Ok(mapping) => mapping.mapping_revision,
                Err(_) => {
                    return CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::MappingUnavailable,
                    };
                }
            };
        let mut request = self.candidate_prepare(probe, candidate, mapping_revision);
        request.service_tier = service_tier.map(str::to_owned);
        if request.validate().is_err() {
            return CandidateProbeVerdict::Ineligible {
                candidate_id: candidate.candidate_id.clone(),
                reason: CandidateProbeRefusal::InvalidRequest,
            };
        }
        // Cette autorité reste individuelle, même si les faits Git du projet
        // sont déjà connus pour un autre modèle ou agent du même lot.
        let mapping = match self.read_view().mapping_authority(&request, registry) {
            Ok(mapping) => mapping,
            Err(_) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::MappingUnavailable,
                };
            }
        };
        let read_git_facts = || {
            let (mapping, cwd) = self
                .read_view()
                .mapping_paths_and_git(&request, mapping, workload)
                .map_err(|_| CandidateProbeRefusal::MappingUnavailable)?;
            self.probe_workspace_preconditions(&request, &mapping, &cwd)?;
            Ok((mapping, cwd))
        };
        let checked = if let Some(cache) = git_facts {
            // Worker/provisioning/epoch/registre/protections sont constants dans
            // ce seul lot. Tout fait variable de mapping/base figure dans la clé.
            let key = match serde_json::to_vec(&(
                &request.project_id,
                request.mapping_revision,
                &request.origin_host,
                &request.origin_cwd,
                &request.source_root,
                &request.remote_root,
                &request.source_root_identity,
                &request.relative_cwd,
                &request.workspace,
                &request.base_commit,
                workload,
            )) {
                Ok(key) => key,
                Err(_) => {
                    return CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::InvalidRequest,
                    };
                }
            };
            // Le refus commun est lui aussi mémorisé : une base absente ne
            // provoque pas 128 préflights. Aucun résultat ne quitte ce lot.
            cache.entry(key).or_insert_with(read_git_facts).clone()
        } else {
            read_git_facts()
        };
        let (mapping, cwd) = match checked {
            Ok(values) => values,
            Err(reason) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason,
                };
            }
        };
        let (checked_readiness, program_failure) = crate::provider_program_compat::capture(|| {
            self.read_view().inspect_candidate(
                registry,
                &request,
                &mapping,
                &cwd,
                operation,
                diagnostics,
                self.provider_accounts_negotiated,
            )
        });
        let readiness = match checked_readiness {
            Ok(readiness) => readiness,
            Err(WorkerRefusal::DefinitionChanged) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: match program_failure {
                        Some(crate::provider_program_compat::UpgradeRefusal::Pending) => {
                            CandidateProbeRefusal::ProgramUpgradeCheckPending
                        }
                        Some(crate::provider_program_compat::UpgradeRefusal::Incompatible) => {
                            CandidateProbeRefusal::ProgramProtocolIncompatible
                        }
                        Some(crate::provider_program_compat::UpgradeRefusal::Unavailable) => {
                            CandidateProbeRefusal::ProgramCheckUnavailable
                        }
                        None => CandidateProbeRefusal::DefinitionUnavailable,
                    },
                };
            }
            Err(WorkerRefusal::PermissionsChanged) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::PermissionsChanged,
                };
            }
            Err(_) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::CapabilityUnavailable,
                };
            }
        };
        if command_write.is_some_and(|writes| {
            !crate::native_permissions::permits_command_execution(&readiness.child_policy, writes)
        }) {
            return CandidateProbeVerdict::Ineligible {
                candidate_id: candidate.candidate_id.clone(),
                reason: CandidateProbeRefusal::CommandExecutionUnavailable,
            };
        }
        let permission_digest = match json_digest(&readiness.child_policy) {
            Ok(digest) => digest,
            Err(_) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::CapabilityUnavailable,
                };
            }
        };
        CandidateProbeVerdict::Eligible {
            candidate_id: candidate.candidate_id.clone(),
            mapping_revision: mapping.mapping_revision,
            definition_digest: readiness.definition.digest,
            permission_digest,
            observed_at: worker_now(),
        }
    }

    pub(super) fn candidate_probe(
        &self,
        probe: &CandidateProbe,
        disconnected: &AtomicBool,
        deadline: Instant,
    ) -> Result<CandidateProbeResult, WorkerRefusal> {
        probe.validate_shape()?;
        if probe.worker != self.identity {
            return Err(WorkerRefusal::StaleConnection);
        }
        let registry = self.read_view().registry();
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let mut verdicts: Vec<CandidateProbeVerdict> = probe
            .candidates
            .iter()
            .map(|candidate| {
                if disconnected.load(Ordering::Acquire) || Instant::now() >= deadline {
                    CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::ResourceLimit,
                    }
                } else {
                    match &registry {
                        Ok(registry) => self.probe_one_candidate_for(
                            probe,
                            candidate,
                            registry,
                            None,
                            None,
                            None,
                            None,
                            &mut operation,
                            None,
                        ),
                        Err(_) => CandidateProbeVerdict::Ineligible {
                            candidate_id: candidate.candidate_id.clone(),
                            reason: CandidateProbeRefusal::CapabilityUnavailable,
                        },
                    }
                }
            })
            .collect();
        if operation.finish().is_err() {
            for verdict in &mut verdicts {
                if let CandidateProbeVerdict::Eligible { candidate_id, .. } = verdict {
                    *verdict = CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate_id.clone(),
                        reason: CandidateProbeRefusal::PermissionsChanged,
                    };
                }
            }
        }
        if Instant::now() >= deadline {
            for verdict in &mut verdicts {
                if let CandidateProbeVerdict::Eligible { candidate_id, .. } = verdict {
                    *verdict = CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate_id.clone(),
                        reason: CandidateProbeRefusal::ResourceLimit,
                    };
                }
            }
        }
        Ok(CandidateProbeResult {
            version: WORKER_CANDIDATE_PROBE_VERSION,
            correlation_id: probe.correlation_id.clone(),
            connection_epoch: probe.connection_epoch.clone(),
            worker: self.identity.clone(),
            admission_id: probe.admission_id.clone(),
            policy_revision: probe.policy_revision,
            verdicts,
        })
    }

    pub(super) fn candidate_probe_v2(
        &mut self,
        probe: &CandidateProbeV2,
        disconnected: &AtomicBool,
        deadline: Instant,
    ) -> Result<CandidateProbeV2Result, WorkerRefusal> {
        probe.validate_shape()?;
        let started = Instant::now();
        if probe.worker != self.identity {
            return Err(WorkerRefusal::StaleConnection);
        }
        let registry = self.read_view().registry()?;
        let capacity = self.compilation_capacity(&registry);
        let candidates = registry
            .routing_candidates()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        let policy = registry
            .routing_policy()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        let enabled = policy.enabled
            && policy
                .machine_policies
                .get(&self.identity.machine_id)
                .is_some_and(|policy| policy.enabled);
        let index = candidates
            .iter()
            .map(|entry| {
                (
                    (
                        entry.machine_id.as_str(),
                        entry.connection_id.as_str(),
                        entry.engine.as_str(),
                        entry.agent_type.as_str(),
                        entry.model.as_str(),
                    ),
                    entry,
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        let forbidden_roots = registry
            .machines()
            .get(&self.identity.machine_id)
            .into_iter()
            .flat_map(|machine| machine.mappings.values())
            .map(|mapping| mapping.remote_root.clone())
            .collect::<Vec<_>>();
        let tool_observations = qualify_workload_tools(
            &self.source,
            &probe.required_tools,
            &forbidden_roots,
            deadline.min(Instant::now() + Duration::from_secs(3)),
        );
        let tools_available = tool_observations
            .iter()
            .all(|fact| fact.state == bridget_transport::protocol::WorkloadToolState::Available);
        let mut git_facts = ProbeGitFacts::new();
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let diagnostics =
            ProbePermissionDiagnostics::new(&probe.correlation_id, &probe.admission_id);
        let mut timings = Vec::with_capacity(probe.candidates.len());
        let mut verdicts: Vec<CandidateProbeV2Verdict> = probe
            .candidates
            .iter()
            .map(|candidate| {
                if disconnected.load(Ordering::Acquire) || Instant::now() >= deadline {
                    return CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::ResourceLimit,
                    };
                }
                if !tools_available {
                    return CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::WorkloadToolUnavailable,
                    };
                }
                let candidate_started = Instant::now();
                let registered = enabled
                    && index
                        .get(&(
                            self.identity.machine_id.as_str(),
                            candidate.connection_id.as_str(),
                            candidate.engine.as_str(),
                            candidate.agent_type.as_str(),
                            candidate.model.as_str(),
                        ))
                        .is_some_and(|entry| {
                            entry.enabled
                                && candidate
                                    .service_tier
                                    .as_ref()
                                    .is_none_or(|tier| entry.service_tiers.contains(tier))
                                && candidate
                                    .effort
                                    .as_ref()
                                    .map_or(entry.default_effort.is_none(), |effort| {
                                        entry.supported_efforts.contains(effort)
                                    })
                        });
                if !registered {
                    return CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::CapabilityUnavailable,
                    };
                }
                let legacy = CandidateProbeCandidate {
                    candidate_id: "c01".into(),
                    project_id: candidate.project_id.clone(),
                    origin_host: candidate.origin_host.clone(),
                    origin_cwd: candidate.origin_cwd.clone(),
                    source_root: candidate.source_root.clone(),
                    remote_root: candidate.remote_root.clone(),
                    source_root_identity: candidate.source_root_identity.clone(),
                    relative_cwd: candidate.relative_cwd.clone(),
                    base_commit: candidate.expected_base_commit.clone(),
                    agent_type: candidate.agent_type.clone(),
                    model: candidate.model.clone(),
                    effort: candidate.effort.clone(),
                    posture: candidate.posture,
                    workspace: candidate.workspace.clone(),
                };
                let legacy_probe = CandidateProbe {
                    version: WORKER_CANDIDATE_PROBE_VERSION,
                    correlation_id: probe.correlation_id.clone(),
                    connection_epoch: probe.connection_epoch.clone(),
                    worker: probe.worker.clone(),
                    admission_id: probe.admission_id.clone(),
                    policy_revision: probe.policy_revision,
                    parent: probe.parent.clone(),
                    candidates: vec![legacy],
                };
                let verdict = match self.probe_one_candidate_for(
                    &legacy_probe,
                    &legacy_probe.candidates[0],
                    &registry,
                    Some(candidate.workload),
                    (!probe.required_tools.is_empty()).then_some(probe.requires_workspace_write),
                    candidate.service_tier.as_deref(),
                    Some(&mut git_facts),
                    &mut operation,
                    Some((&diagnostics, candidate.candidate_id.as_str())),
                ) {
                    CandidateProbeVerdict::Eligible {
                        mapping_revision,
                        definition_digest,
                        permission_digest,
                        observed_at,
                        ..
                    } => CandidateProbeV2Verdict::Eligible {
                        candidate_id: candidate.candidate_id.clone(),
                        mapping_revision,
                        definition_digest,
                        permission_digest,
                        observed_at,
                    },
                    CandidateProbeVerdict::Ineligible { reason, .. } => {
                        CandidateProbeV2Verdict::Ineligible {
                            candidate_id: candidate.candidate_id.clone(),
                            reason: diagnostics.refusal(
                                &candidate.candidate_id,
                                reason,
                                self.permission_reasons_negotiated,
                            ),
                        }
                    }
                };
                timings.push((
                    candidate.candidate_id.clone(),
                    candidate_started.elapsed().as_millis() as u64,
                ));
                verdict
            })
            .collect();
        if let Err(code) = operation.finish() {
            for verdict in &mut verdicts {
                if let CandidateProbeV2Verdict::Eligible { candidate_id, .. } = verdict {
                    observe_probe_permission_failure(
                        &code,
                        Some((&diagnostics, candidate_id)),
                        ProbePermissionSite::BatchFinish,
                    );
                    *verdict = CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate_id.clone(),
                        reason: diagnostics.refusal(
                            candidate_id,
                            CandidateProbeRefusal::PermissionsChanged,
                            self.permission_reasons_negotiated,
                        ),
                    };
                }
            }
        }
        // Les contrôles Git peuvent durer : observer la charge à la remise,
        // pour ne pas rendre périmé un échantillon pris avant leur lecture.
        let resources = self.host_resources.collect(worker_now(), capacity);
        eprintln!(
            "bridget_worker_probe172 {}",
            serde_json::json!({
            "correlation_id":probe.correlation_id,"admission_id":probe.admission_id,
            "candidates":probe.candidates.len(),"elapsed_ms":started.elapsed().as_millis() as u64,
            "candidate_elapsed_ms":timings})
        );
        if Instant::now() >= deadline {
            for verdict in &mut verdicts {
                if let CandidateProbeV2Verdict::Eligible { candidate_id, .. } = verdict {
                    *verdict = CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate_id.clone(),
                        reason: CandidateProbeRefusal::ResourceLimit,
                    };
                }
            }
        }
        Ok(CandidateProbeV2Result {
            tool_observations,
            version: WORKER_CANDIDATE_PROBE_V2_VERSION,
            correlation_id: probe.correlation_id.clone(),
            connection_epoch: probe.connection_epoch.clone(),
            worker: self.identity.clone(),
            admission_id: probe.admission_id.clone(),
            policy_revision: probe.policy_revision,
            resources,
            verdicts,
        })
    }
}

pub(super) fn read_worker_command(
    root: &Path,
    worker: &WorkerIdentity,
    identity: &WorkerCommandIdentity,
) -> Result<Option<WorkerCommandReceipt>, WorkerStoreError> {
    identity.validate()?;
    let mut result: Option<WorkerCommandReceipt> = read_optional(
        &root
            .join("commands")
            .join(format!("{}.json", marker_key(identity))),
        MAX_METADATA_BYTES,
    )?;
    if let Some(receipt) = result.as_mut()
        && receipt.worker == *worker
        && receipt.identity == *identity
    {
        proof::overlay_fence(root, worker, receipt)?;
    }
    if let Some(receipt) = &result {
        if receipt.worker != *worker
            || receipt.version != STORE_VERSION
            || receipt.identity != *identity
        {
            return Err(WorkerStoreError::Conflict);
        }
        receipt.placement.validate()?;
        let expected_fence = receipt
            .pre_start_fence
            .as_ref()
            .map(|_| proof::fence_digest(worker, identity, &receipt.placement, receipt.deadline_at))
            .transpose()?;
        if !receipt.placement.matches_worker(worker)
            || !valid_digest(&receipt.command_digest)
            || receipt.created_at < 0
            || (receipt.pre_start_fence.is_none() && receipt.deadline_at <= receipt.created_at)
            || receipt.pre_start_fence.as_ref().is_some_and(|digest| {
                !valid_digest(digest)
                    || Some(digest) != expected_fence.as_ref()
                    || *digest != receipt.command_digest
                    || receipt.phase != WorkerCommandPhase::Refused
                    || receipt.marker.is_some()
                    || receipt.permit.is_some()
                    || receipt.child_id.is_some()
                    || receipt.preparation.is_some()
                    || receipt.routing.is_some()
                    || receipt.release_epoch.is_some()
                    || receipt.native_ready
                    || receipt.startup_failed
                    || receipt.startup_failure.is_some()
                    || receipt.terminal_exit_code.is_some()
            })
        {
            return Err(WorkerStoreError::Quarantined);
        }
    }
    Ok(result)
}

/// La même décision pour Prepare vivant et pour sonde. Aucun chemin ne sort.
pub(super) fn shared_paths_available(
    root: &Path,
    paths: &[PathBuf],
) -> Result<(), CandidateProbeRefusal> {
    if paths
        .iter()
        .any(|cwd| cwd.starts_with(root) || root.starts_with(cwd))
    {
        Err(CandidateProbeRefusal::WorkspaceBusy)
    } else {
        Ok(())
    }
}
impl WorkerReadView<'_> {
    pub(super) fn active_command_roots(&self) -> Result<Vec<PathBuf>, CandidateProbeRefusal> {
        let read = || -> Result<Vec<PathBuf>, WorkerStoreError> {
            let root = self.config.state_root.join("store");
            for directory in [
                &root,
                &root.join("commands"),
                &root.join("slots"),
                &root.join("workspaces"),
            ] {
                check_private(directory, true)?;
            }
            let enrollment = self.config.state_root.join(format!(
                ".worker-{}.enrolled",
                digest_bytes(root.as_os_str().as_encoded_bytes())
            ));
            let enrolled: Enrollment = read_optional(&enrollment, MAX_METADATA_BYTES)?
                .ok_or(WorkerStoreError::Quarantined)?;
            if enrolled.version != STORE_VERSION
                || enrolled.worker != *self.identity
                || enrolled.quarantined
            {
                return Err(WorkerStoreError::Quarantined);
            }
            let mut paths = Vec::new();
            for (count, entry) in fs::read_dir(root.join("commands"))?.enumerate() {
                if count >= MAX_WORKER_STORE_ENTRIES {
                    return Err(WorkerStoreError::Quota);
                }
                let path = entry?.path();
                if path.extension().is_none_or(|e| e != "json") {
                    continue;
                }
                let header: WorkerCommandReceipt = read_optional(&path, MAX_METADATA_BYTES)?
                    .ok_or(WorkerStoreError::Quarantined)?;
                if path
                    != root
                        .join("commands")
                        .join(format!("{}.json", marker_key(&header.identity)))
                {
                    return Err(WorkerStoreError::Quarantined);
                }
                let command = read_worker_command(&root, self.identity, &header.identity)?
                    .ok_or(WorkerStoreError::Quarantined)?;
                if !matches!(
                    command.phase,
                    WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                ) {
                    paths.push(PathBuf::from(command.placement.execution_cwd));
                }
            }
            Ok(paths)
        };
        read().map_err(|_| CandidateProbeRefusal::WorkspaceUnavailable)
    }
}
