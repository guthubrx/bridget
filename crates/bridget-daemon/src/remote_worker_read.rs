//! Lectures communes au worker vivant et au diagnostic local. Aucun store écrivain.
use super::*;
use std::cell::{Cell, RefCell};

pub(super) struct WorkerReadView<'a> {
    pub(super) config: &'a WorkerRuntimeConfig,
    pub(super) identity: &'a WorkerIdentity,
    pub(super) credential: &'a IdentityCredential,
    pub(super) source: &'a crate::lifecycle::SourceEnvironment,
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
    let registry = AgentRegistry::load_from_path(config.registry_path.clone())?;
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
    let mut source = crate::lifecycle::source_environment();
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
    fn workspace_path(&self, command: &WorkerCommandIdentity) -> PathBuf {
        self.config
            .state_root
            .join("store/workspaces")
            .join(format!("{}.json", marker_key(command)))
    }
    pub(super) fn registry(&self) -> Result<AgentRegistry, WorkerRefusal> {
        let registry = AgentRegistry::load_from_path(self.config.registry_path.clone())
            .map_err(|_| WorkerRefusal::Unauthorized)?;
        let provision = registry
            .machines()
            .get(&self.identity.machine_id)
            .ok_or(WorkerRefusal::Unauthorized)?;
        if provision.worker_id != self.identity.worker_id
            || provision.provisioning_revision != self.identity.provisioning_revision
            || !provision.matches_credential(&self.credential)
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
    pub(super) fn mapping_paths_and_git(
        &self,
        request: &WorkerPrepare,
        mapping: ProjectMapping,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
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
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        trace.at("registry");
        let registry = self.registry()?;
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
        if matches!(
            &prepared.request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
        ) {
            trace.at("workspace_receipt");
            let receipt: WorkspacePreparationReceipt =
                read_optional(&self.workspace_path(&prepared.command), MAX_METADATA_BYTES)
                    .map_err(refuse)?
                    .ok_or(WorkerRefusal::UnknownExecution)?;
            if receipt.worker != *self.identity
                || receipt.command != prepared.command
                || receipt.prepare_digest
                    != json_digest(&(&prepared.command, &prepared.request)).map_err(refuse)?
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            trace.at("workspace_report");
            let original = receipt.record.ok_or(WorkerRefusal::UnknownExecution)?;
            let record = if let Some(anchor) = repair {
                crate::workspace_git::report_repaired_workspace(&original, anchor)
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?
            } else {
                original
            };
            let report = if let Some(anchor) = repair {
                crate::workspace_git::report_repaired_workspace(&record, anchor)
            } else {
                crate::workspace_git::report(&record)
            }
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if before_release
                && (report.head_commit.as_ref() != Some(&record.base_commit)
                    || report.head_on_branch != Some(true))
            {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            trace.at("workspace_mapping");
            cwd = self.remap_prepared_worktree(&mut mapping, &cwd, &record)?;
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
        crate::lifecycle::launch_preflight(&scoped, &prepared.request.agent_type, &self.source)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        trace.at("child_permissions");
        crate::native_permissions::recheck_remote_child_permissions_in_operation(
            &prepared.prepared.child_policy,
            scoped
                .get(&prepared.request.agent_type)
                .map_err(|_| WorkerRefusal::DefinitionChanged)?,
            &cwd,
            &self.source,
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
pub(crate) fn diagnose_local(
    config: &WorkerRuntimeConfig,
    command: &WorkerCommandIdentity,
) -> Result<serde_json::Value, String> {
    command
        .validate()
        .map_err(|_| "diagnostic_identity_invalid")?;
    let (identity, credential, source) =
        inputs(config).map_err(|_| "diagnostic_context_unavailable")?;
    diagnose_observed(
        WorkerReadView {
            config,
            identity: &identity,
            credential: &credential,
            source: &source,
        },
        command,
        || {},
    )
}

fn stamp(path: &Path) -> Result<Option<(u64, u64, u64, i64, i64, i64, i64)>, String> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_file() && !m.file_type().is_symlink() => Ok(Some((
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        ))),
        Ok(_) => Err("diagnostic_file_type_invalid".into()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("diagnostic_file_unavailable".into()),
    }
}

pub(super) fn diagnose_observed(
    view: WorkerReadView<'_>,
    command: &WorkerCommandIdentity,
    after_check: impl FnOnce(),
) -> Result<serde_json::Value, String> {
    let config = view.config;
    let identity = view.identity;
    command
        .validate()
        .map_err(|_| "diagnostic_identity_invalid")?;
    let root = config.state_root.join("store");
    let parent = root.parent().ok_or("diagnostic_store_unavailable")?;
    let enrollment = parent.join(format!(
        ".worker-{}.enrolled",
        digest_bytes(root.as_os_str().as_encoded_bytes())
    ));
    let path = root
        .join("commands")
        .join(format!("{}.json", marker_key(command)));
    let workspace = root
        .join("workspaces")
        .join(format!("{}.json", marker_key(command)));
    let paths = [
        &path,
        &workspace,
        &enrollment,
        &config.registry_path,
        &config.credential_path,
    ];
    let before = paths
        .iter()
        .map(|p| stamp(p))
        .collect::<Result<Vec<_>, _>>()?;
    check_private(&root, true).map_err(|_| "diagnostic_store_unavailable")?;
    let enrolled: Enrollment = read_optional(&enrollment, MAX_METADATA_BYTES)
        .map_err(|_| "diagnostic_enrollment_invalid")?
        .ok_or("diagnostic_enrollment_unavailable")?;
    if enrolled.version != STORE_VERSION || enrolled.worker != *identity || enrolled.quarantined {
        return Err("diagnostic_enrollment_invalid".into());
    }
    let receipt: WorkerCommandReceipt = read_optional(&path, MAX_METADATA_BYTES)
        .map_err(|_| "diagnostic_receipt_invalid")?
        .ok_or("diagnostic_receipt_unavailable")?;
    if receipt.version != STORE_VERSION
        || receipt.worker != *identity
        || receipt.identity != *command
        || !receipt.placement.matches_worker(&identity)
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
    after_check();
    let after = paths
        .iter()
        .map(|p| stamp(p))
        .collect::<Result<Vec<_>, _>>()?;
    if before != after {
        return Err("diagnostic_observation_changed".into());
    }
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
    serde_json::json!({"status":"refused","code":code,"command":command,
        "message":message,"recovery":recovery,"human_action":human_action,
        "scope":"local_read_only_recheck","authoritative":false})
}
