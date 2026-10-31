//! Possession C19 : le bootstrap et le marqueur précèdent tout export.
//! Ce module ne crée aucune Task, ne choisit aucun fournisseur et ne remplace
//! ni la réservation du store ni l'audit au Release.
use super::*;
use crate::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarker, ManagedMarkerStore, ManagedStatus,
    ManagedStatusReporter, ManagedStopResult, RunningManagedChild,
};
use std::ffi::OsString;

pub(crate) use bridget_transport::protocol::CommitViewFailure as ExportFailure;
pub(crate) fn failure(error: &str) -> ExportFailure {
    match error.split(':').next().unwrap_or("") {
        "workspace_disk_full" => return ExportFailure::DiskFull,
        "workspace_access_denied" => return ExportFailure::AccessDenied,
        _ => {}
    }
    match error.strip_prefix("workspace_commit_view_unavailable:cause=") {
        Some("git_timeout" | "view_link_timeout" | "view_audit_timeout") => ExportFailure::TimedOut,
        Some("preparation_expired") => ExportFailure::PreparationExpired,
        Some("git_object_unavailable"|"commit_unavailable") => ExportFailure::ObjectUnavailable,
        Some("destination_inside_source") => ExportFailure::DestinationInsideRepository,
        Some("destination_volume_mismatch") => ExportFailure::DestinationVolumeMismatch,
        Some("destination_unavailable") => ExportFailure::DestinationUnavailable,
        Some("destination_invalid") => ExportFailure::DestinationUnsafe,
        Some("storage_space_unavailable") => ExportFailure::StorageSpaceUnknown,
        Some("link_outside_view") => ExportFailure::LinkOutsideView,
        Some("link_cycle") => ExportFailure::LinkCycle,
        Some(
            "path_unsupported"
            | "link_unsupported"
,
        ) => ExportFailure::UnsupportedPath,
        Some("view_integrity_failed" | "view_identity_changed" | "git_object_integrity_failed") => {
            ExportFailure::IntegrityFailed
        }
        Some("byte_limit" | "view_entry_limit" | "link_metadata_limit" | "proof_byte_limit" | "tree_limit" | "entry_limit") => {
            ExportFailure::ByteLimit
        }
        Some("export_busy") => ExportFailure::OperationCapacity,
        Some("source_identity_changed") => ExportFailure::SourceIdentityChanged,
        Some("source_unavailable" | "source_inventory_unavailable") => {
            ExportFailure::SourceUnavailable
        }
        Some(
            "proof_unavailable"
            | "proof_invalid"
            | "proof_binding_mismatch"
            | "preparation_incomplete"
            | "view_closed",
        ) => ExportFailure::ProofUnavailable,
        Some("git_stop_unconfirmed") => ExportFailure::StopUnconfirmed,
        _ => ExportFailure::Unknown,
    }
}
/// Le contrôleur natif doit authentifier et durcir cette requête avant RELEASE.
/// Ce type porte une opération de fichiers, jamais une nouvelle mission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ViewJobRequest {
    Export,
    Materialize {
        command: bridget_transport::protocol::WorkerCommandIdentity,
        sequence: u64,
        path: String,
    },
}
impl ViewJobRequest {
    pub(crate) fn identity_for_binding(
        &self,
        binding: &ViewBinding,
    ) -> Result<ManagedIdentity, String> {
        let key = binding.key()?;
        match self {
            Self::Export => Ok(ManagedIdentity {
                instance_id: binding.command.task_id.clone(),
                command_id: binding.command.command_id.clone(),
                generation: 1,
            }),
            Self::Materialize {
                command,
                sequence,
                path,
            } => {
                command
                    .validate()
                    .map_err(|_| refusal("proof_binding_invalid"))?;
                // Le contrôleur prouve la chaîne198 avant BEGIN/RELEASE.
                // Cette identité pure ne donne aucune autorité à une commande.
                if command.task_id != binding.command.task_id || *sequence == 0 || !path_valid(path)
                {
                    return Err(refusal("proof_binding_invalid"));
                }
                Ok(ManagedIdentity {
                    instance_id: command.instance_id.clone(),
                    command_id: view_digest(
                        &serde_json::to_vec(&(key, command, sequence, path))
                            .map_err(|_| refusal("proof_binding_invalid"))?,
                    ),
                    generation: command.generation,
                })
            }
        }
    }
    pub(crate) fn identity(&self, intent: &ViewIntent) -> Result<ManagedIdentity, String> {
        let identity = self.identity_for_binding(&intent.binding)?;
        if let Self::Materialize { path, .. } = self
            && !intent
                .manifest
                .entries
                .iter()
                .any(|entry| entry.path == *path && entry.mode != "120000")
        {
            return Err(refusal("proof_binding_invalid"));
        }
        Ok(identity)
    }
    fn encoded(&self) -> Result<String, String> {
        let encoded = serde_json::to_string(self).map_err(|_| refusal("proof_binding_invalid"))?;
        if encoded.len() > 8192 {
            return Err(refusal("proof_byte_limit"));
        }
        Ok(encoded)
    }
}
impl ViewProofs {
    fn job_identity(
        &self,
        intent: &ViewIntent,
        request: &ViewJobRequest,
    ) -> Result<ManagedIdentity, String> {
        self.require_live_intent(intent)?;
        let identity = request.identity(intent)?;
        if let ViewJobRequest::Materialize { command, .. } = request
            && command.command_id != intent.binding.command.command_id
            && !crate::remote_worker::view_file_alias_matches(&self.path, &intent.binding, request)
        {
            return Err(refusal("proof_binding_mismatch"));
        }
        Ok(identity)
    }
}
fn marker_name(intent: &ViewIntent) -> Result<String, String> {
    Ok(format!("view-{}", intent.binding.key()?))
}

/// Le sous-mode refuse un appel ordinaire avant de lire une preuve. L'identité
/// et le FD sont ceux du bootstrap ; aucune valeur de notice ne fait autorité.
pub(crate) fn run_commit_view_plan(args:&[String])->Result<(),String> {
    let mut reporter=ManagedStatusReporter::from_environment()
        .map_err(|_|refusal("managed_context_required"))?
        .ok_or_else(||refusal("managed_context_required"))?;
    let result=(||{
        let [root,key]=args else {return Err(refusal("proof_binding_invalid"));};
        let proofs=ViewProofs::open(Path::new(root))?;
        let seed=proofs.load_plan(key)?;
        if reporter.identity()!=&ViewJobRequest::Export.identity_for_binding(&seed.binding)? {
            return Err(refusal("proof_binding_mismatch"));
        }
        let intent=seed.planned_intent()?;
        seed.matches(&intent)?;seed.live()?;
        proofs.write_intent(&intent)?;
        export(&intent.source,&intent.manifest,&intent.destination()?)?;
        seed.check_source()?;
        proofs.publish(&intent)?;
        seed.live()
    })();
    match result {
        Ok(())=>{reporter.startup_succeeded();Ok(())}
        Err(error)=>{
            let reason=serde_json::to_string(&failure(&error)).map_err(|_|refusal("proof_invalid"))?;
            let _=reporter.startup_failed("workspace_commit_view",reason);
            Err(error)
        }
    }
}
pub(crate) fn run_commit_view_export(args: &[String]) -> Result<(), String> {
    run_commit_view_files(args, false)
}
pub(crate) fn run_commit_view_materialize(args: &[String]) -> Result<(), String> {
    run_commit_view_files(args, true)
}
fn run_commit_view_files(args: &[String], file: bool) -> Result<(), String> {
    let mut reporter = ManagedStatusReporter::from_environment()
        .map_err(|_| refusal("managed_context_required"))?
        .ok_or_else(|| refusal("managed_context_required"))?;
    let result = (|| {
        let (root, key, request) = match (file, args) {
            (false, [root, key]) => (root, key, ViewJobRequest::Export),
            (true, [root, key, encoded]) if encoded.len() <= 8192 => {
                let request: ViewJobRequest =
                    serde_json::from_str(encoded).map_err(|_| refusal("proof_binding_invalid"))?;
                if !matches!(request, ViewJobRequest::Materialize { .. }) {
                    return Err(refusal("proof_binding_invalid"));
                }
                (root, key, request)
            }
            _ => return Err(refusal("proof_binding_invalid")),
        };
        let proofs = ViewProofs::open(Path::new(root))?;
        let intent = proofs.load(key)?;
        if reporter.identity() != &proofs.job_identity(&intent, &request)? {
            return Err(refusal("proof_binding_mismatch"));
        }
        let destination = intent.destination()?;
        match request {
            ViewJobRequest::Export => {
                if fs::symlink_metadata(&destination).is_ok() {
                    return Err(refusal("preparation_incomplete"));
                }
                export(&intent.source, &intent.manifest, &destination)?;
                proofs.publish(&intent)
            }
            ViewJobRequest::Materialize { path, .. } => {
                proofs.recover(&intent.binding, &intent.source, &intent.data_root)?;
                materialize(&intent.source, &intent.manifest, &destination, &path)?;
                audit(&intent.manifest, &destination)
            }
        }
    })();
    match result {
        Ok(()) => {
            reporter.startup_succeeded();
            Ok(())
        }
        Err(error) => {
            let reason =
                serde_json::to_string(&failure(&error)).map_err(|_| refusal("proof_invalid"))?;
            let _ = reporter.startup_failed("workspace_commit_view", reason);
            Err(error)
        }
    }
}

enum Process {
    BeforeRelease(Child),
    Running(RunningManagedChild),
}
#[must_use = "Conserver le propriétaire tant que l'arrêt n'est pas confirmé."]
pub(crate) struct ViewJob {
    process: Process,
    _lease: ViewLease,
    failure: Option<ExportFailure>,
    deadline: Instant,
    stop_requested: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum JobPoll {
    Running,
    Stopped(Result<(), ExportFailure>),
    StopUnconfirmed,
}
impl ViewJob {
    /// L'appelant a durci le reçu et réservé intention, publication, verrou et
    /// marqueur. Un échec peut rendre un Child à conserver, jamais à oublier.
    #[cfg(test)]
    pub(crate) fn start(
        proofs: &ViewProofs,
        intent: &ViewIntent,
        request: &ViewJobRequest,
        executable: &Path,
        environment: &BTreeMap<String, OsString>,
        authorize_release: impl FnOnce(&ManagedMarker) -> Result<(), ExportFailure>,
    ) -> Result<Self, (ExportFailure, Option<Self>)> {
        Self::start_registered(
            proofs,
            intent,
            request,
            executable,
            environment,
            || Ok(()),
            authorize_release,
        )
    }

    pub(crate) fn start_registered(
        proofs: &ViewProofs,
        intent: &ViewIntent,
        request: &ViewJobRequest,
        executable: &Path,
        environment: &BTreeMap<String, OsString>,
        before_spawn: impl FnOnce() -> Result<(), ExportFailure>,
        authorize_release: impl FnOnce(&ManagedMarker) -> Result<(), ExportFailure>,
    ) -> Result<Self, (ExportFailure, Option<Self>)> {
        let plain = |cause| (cause, None);
        let identity = proofs
            .job_identity(intent, request)
            .map_err(|e| plain(failure(&e)))?;
        if matches!(request, ViewJobRequest::Export) {
            if fs::symlink_metadata(intent.destination().map_err(|e| plain(failure(&e)))?).is_ok() {
                return Err(plain(ExportFailure::ProofUnavailable));
            }
        } else {
            proofs.publication(intent).map_err(|e| plain(failure(&e)))?;
        }
        let args=match request {
            ViewJobRequest::Export=>vec!["workspace-view-export".into(),proofs.path.to_string_lossy().into(),intent.binding.key().map_err(|e|plain(failure(&e)))?],
            ViewJobRequest::Materialize{..}=>vec!["workspace-view-materialize".into(),proofs.path.to_string_lossy().into(),intent.binding.key().map_err(|e|plain(failure(&e)))?,
                request.encoded().map_err(|e|plain(failure(&e)))?],
        };
        Self::start_launch(proofs,&intent.binding,identity,args,executable,environment,before_spawn,authorize_release)
    }

    pub(crate) fn start_plan_registered(
        proofs:&ViewProofs,seed:&ViewPlan,executable:&Path,environment:&BTreeMap<String,OsString>,
        before_spawn:impl FnOnce()->Result<(),ExportFailure>,
        authorize_release:impl FnOnce(&ManagedMarker)->Result<(),ExportFailure>,
    )->Result<Self,(ExportFailure,Option<Self>)> {
        let plain=|cause|(cause,None);
        seed.live().map_err(|e|plain(failure(&e)))?;
        let key=seed.binding.key().map_err(|e|plain(failure(&e)))?;
        if proofs.load_plan(&key).map_err(|e|plain(failure(&e)))?!=*seed
            || proofs.read(&format!("{key}.intent.json"),MAX_INTENT).map_err(|e|plain(failure(&e)))?.is_some()
            || fs::symlink_metadata(seed.data_root.join(&key)).is_ok() {
            return Err(plain(ExportFailure::ProofUnavailable));
        }
        let identity=ViewJobRequest::Export.identity_for_binding(&seed.binding).map_err(|e|plain(failure(&e)))?;
        Self::start_launch(proofs,&seed.binding,identity,
            vec!["workspace-view-plan".into(),proofs.path.to_string_lossy().into(),key],
            executable,environment,before_spawn,authorize_release)
    }

    fn start_launch(
        proofs:&ViewProofs,binding:&ViewBinding,identity:ManagedIdentity,args:Vec<String>,
        executable:&Path,environment:&BTreeMap<String,OsString>,
        before_spawn:impl FnOnce()->Result<(),ExportFailure>,
        authorize_release:impl FnOnce(&ManagedMarker)->Result<(),ExportFailure>,
    )->Result<Self,(ExportFailure,Option<Self>)> {
        let plain=|cause|(cause,None);
        let key=binding.key().map_err(|e|plain(failure(&e)))?;
        let lease=proofs.lease(&key).map_err(|e|plain(failure(&e)))?;
        let name=format!("view-{key}");
        if proofs.read(&format!("{name}.json"),2048).map_err(|e|plain(failure(&e)))?.is_some() {
            return Err(plain(ExportFailure::StopUnconfirmed));
        }
        let markers=ManagedMarkerStore::at_directory(&proofs.path);
        let mut env=BTreeMap::new();
        for name in ["PATH","HOME","TMPDIR"] {
            if let Some(value)=environment.get(name) {env.insert(name.into(),value.clone());}
        }
        env.insert("LC_ALL".into(),OsString::from("C"));
        #[allow(unused_mut)]
        let mut launch=ManagedLaunch {bootstrap_executable:executable.into(),wrapper_executable:executable.into(),
            wrapper_args:args,identity,cwd:proofs.path.clone(),env};
        #[cfg(test)]
        let native_recipe=environment.get("B221_VIEW_NATIVE_BINARY").is_some_and(|value|value=="1");
        #[cfg(test)] if !native_recipe {
            let planning=launch.wrapper_args[0]=="workspace-view-plan";
            let encoded=if !planning && launch.wrapper_args.len()==4 {launch.wrapper_args[3].clone()} else {
                ViewJobRequest::Export.encoded().unwrap()
            };
            launch.wrapper_args=vec!["--exact".into(),"workspace_git::commit_view::tests::c19_export_child".into(),
                "--ignored".into(),"--nocapture".into(),"--test-threads=1".into()];
            launch.env.insert("B221_VIEW_PROOFS".into(),proofs.path.as_os_str().into());
            launch.env.insert("B221_VIEW_KEY".into(),key.into());
            launch.env.insert("B221_VIEW_REQUEST".into(),encoded.into());
            if planning {launch.env.insert("B221_VIEW_PLAN".into(),"1".into());}
            for name in ["B221_VIEW_TEST_STALL","B221_VIEW_TEST_NO_READY"] {
                if let Some(value)=environment.get(name) {launch.env.insert(name.into(),value.clone());}
            }
        }
        // Le reçu doit distinguer une réservation sans processus du trou
        // après fork et avant READY/marqueur, même si ce contrôleur disparaît.
        before_spawn().map_err(plain)?;
        #[cfg(not(test))]
        let bootstrap = crate::managed_process::spawn_managed_bootstrap(&launch)
            .map_err(|_| plain(ExportFailure::StartupUnavailable))?;
        #[cfg(test)]
        let bootstrap = if native_recipe {
            crate::managed_process::spawn_managed_bootstrap(&launch)
                .map_err(|_|plain(ExportFailure::StartupUnavailable))?
        } else {crate::managed_process::tests::spawn_catalogue_test_bootstrap(&launch)};
        let deadline = Instant::now() + Duration::from_secs(2);
        let ready = match bootstrap
            .wait_ready_or_cancel_retained(|| Instant::now() >= deadline, Duration::from_millis(10))
        {
            Ok(Some(ready)) => ready,
            Ok(None) => return Err(plain(ExportFailure::StartupUnavailable)),
            Err((_, child)) => {
                return Err((
                    ExportFailure::StartupUnavailable,
                    Some(Self {
                        process: Process::BeforeRelease(child),
                        _lease: lease,
                        failure: Some(ExportFailure::StartupUnavailable),
                        deadline: Instant::now() + DEADLINE,
                        stop_requested: false,
                    }),
                ));
            }
        };
        if !ready.belongs_to(&launch.identity) {
            return Err((
                ExportFailure::StartupUnavailable,
                Some(Self {
                    process: Process::BeforeRelease(ready.abandon()),
                    _lease: lease,
                    failure: Some(ExportFailure::StartupUnavailable),
                    deadline: Instant::now() + DEADLINE,
                    stop_requested: false,
                }),
            ));
        }
        let marked = match ready.persist_marker_retained(&markers, &name) {
            Ok(marked) => marked,
            Err((_, child)) => {
                return Err((
                    ExportFailure::ProofUnavailable,
                    Some(Self {
                        process: Process::BeforeRelease(child),
                        _lease: lease,
                        failure: Some(ExportFailure::ProofUnavailable),
                        deadline: Instant::now() + DEADLINE,
                        stop_requested: false,
                    }),
                ));
            }
        };
        if let Err(cause) = authorize_release(marked.marker().marker()) {
            let (child, _) = marked.abandon();
            return Err((
                cause,
                Some(Self {
                    process: Process::BeforeRelease(child),
                    _lease: lease,
                    failure: Some(cause),
                    deadline: Instant::now() + DEADLINE,
                    stop_requested: false,
                }),
            ));
        }
        let mut child = match marked.release_retained() {
            Ok(child) => child,
            Err((_, child)) => {
                return Err((
                    ExportFailure::StopUnconfirmed,
                    Some(Self {
                        process: Process::Running(child),
                        _lease: lease,
                        failure: Some(ExportFailure::StopUnconfirmed),
                        deadline: Instant::now() + DEADLINE,
                        stop_requested: false,
                    }),
                ));
            }
        };
        let mut cause = None;
        if child.set_status_nonblocking().is_err() {
            cause = Some(ExportFailure::StartupUnavailable);
        }
        Ok(Self {
            process: Process::Running(child),
            _lease: lease,
            failure: cause,
            deadline: Instant::now() + DEADLINE,
            stop_requested: false,
        })
    }

    /// La fin du leader n'est jamais assimilée à celle du groupe. La réussite
    /// de processus ne dispense pas l'appelant de contrôler la publication.
    pub(crate) fn poll(&mut self) -> Result<JobPoll, String> {
        if Instant::now() >= self.deadline && !self.stop_requested {
            self.stop(Duration::ZERO)?;
        }
        let pending = if self.stop_requested {
            JobPoll::StopUnconfirmed
        } else {
            JobPoll::Running
        };
        match &mut self.process {
            Process::BeforeRelease(child) => match child.try_wait().map_err(compact::io_failure)? {
                None => Ok(pending),
                Some(_) => Ok(JobPoll::Stopped(Err(self
                    .failure
                    .unwrap_or(ExportFailure::StartupUnavailable)))),
            },
            Process::Running(child) => {
                match child.try_status() {
                    Ok(Some(ManagedStatus::StartupFailed {
                        kind,
                        reason,
                        instance_id,
                        command_id,
                        generation,
                    })) if kind == "workspace_commit_view"
                        && instance_id == child.marker().marker().instance_id
                        && command_id == child.marker().marker().command_id
                        && generation == child.marker().marker().generation =>
                    {
                        self.failure =
                            Some(serde_json::from_str(&reason).unwrap_or(ExportFailure::Unknown));
                    }
                    Ok(None) => {}
                    _ => {
                        self.failure = Some(ExportFailure::Unknown);
                    }
                }
                let Some(status) = child.try_wait().map_err(compact::io_failure)? else {
                    return Ok(pending);
                };
                if !child
                    .reap_terminated_group()
                    .map_err(|_| refusal("git_stop_unconfirmed"))?
                {
                    return Ok(pending);
                }
                Ok(JobPoll::Stopped(match self.failure {
                    Some(cause) => Err(cause),
                    None if status.success() => Ok(()),
                    None => Err(ExportFailure::Unknown),
                }))
            }
        }
    }

    pub(crate) fn cancel(&mut self, grace: Duration) -> Result<bool, String> {
        self.abort(ExportFailure::Cancelled, grace)
    }

    /// Refus du contrôleur distinct d'une annulation utilisateur. Un arrêt
    /// déjà demandé n'émet pas un nouveau signal à chaque tick du worker.
    pub(crate) fn abort(&mut self, cause: ExportFailure, grace: Duration) -> Result<bool, String> {
        self.failure.get_or_insert(cause);
        if self.stop_requested {
            return self
                .poll()
                .map(|state| matches!(state, JobPoll::Stopped(_)));
        }
        self.stop(grace)
    }

    /// Arrêt du seul groupe détenu. Pas de SIGKILL. false garde le Child,
    /// le verrou et le marqueur ; le worker doit conserver cette instance.
    pub(crate) fn stop(&mut self, grace: Duration) -> Result<bool, String> {
        self.stop_requested = true;
        self.failure.get_or_insert(ExportFailure::TimedOut);
        match &mut self.process {
            Process::Running(child) => child
                .stop_group(Duration::ZERO, grace, Duration::from_millis(10))
                .map(|result| !matches!(result, ManagedStopResult::Timeout))
                .map_err(|_| refusal("git_stop_unconfirmed")),
            Process::BeforeRelease(child) => {
                if child.try_wait().map_err(compact::io_failure)?.is_some() {
                    return Ok(true);
                }
                if unsafe { libc::kill(child.id() as i32, libc::SIGTERM) } != 0
                    && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
                {
                    return Err(refusal("git_stop_unconfirmed"));
                }
                let deadline = Instant::now() + grace;
                while Instant::now() < deadline {
                    if child.try_wait().map_err(compact::io_failure)?.is_some() {
                        return Ok(true);
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(false)
            }
        }
    }
}

/// Relecture non destructive après perte du handle : aucun signal par un PID
/// déclaré. Un groupe vivant ou une identité ambiguë reste retenu. La future
/// clôture native décide du retrait après cette preuve, jamais un lecteur.
pub(crate) fn source_group_stopped(
    proofs: &ViewProofs,
    intent: &ViewIntent,
) -> Result<bool, String> {
    let Some(marker) = proofs.source_marker(&intent.binding)? else {
        return Ok(true);
    };
    // Même un leader disparu ou recyclé ne prouve pas l'arrêt de descendants.
    crate::managed_process::group_exists(marker.pgid)
        .map(|exists| !exists)
        .map_err(|_| refusal("git_stop_unconfirmed"))
}
