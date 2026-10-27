//! Réparation185 : reçus séparés du canon de naissance, aucun nouveau Start.
use super::*;
use bridget_transport::protocol::{
    WorkspaceRepairActionV1, WorkspaceRepairRequestV1, WorkspaceRepairStatusV1,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepairCapability {
    pub version: u16,
    pub worker: WorkerIdentity,
    pub command: WorkerCommandIdentity,
    pub wrapper_pid: u32,
    pub wrapper_birth: u64,
    pub thread_id: String,
}

pub(crate) fn capability_path(root: &Path, command: &WorkerCommandIdentity) -> PathBuf {
    root.join("markers")
        .join(format!("{}.repair-capability", marker_key(command)))
}
pub(crate) fn result_path(
    root: &Path,
    command: &WorkerCommandIdentity,
    request_id: &str,
) -> PathBuf {
    root.join("markers").join(format!(
        "{}.repair-{}",
        marker_key(command),
        digest_bytes(request_id.as_bytes())
    ))
}
pub(crate) fn publish_capability(
    root: &Path,
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
    thread_id: &str,
) -> Result<(), String> {
    let value = RepairCapability {
        version: 1,
        worker: worker.clone(),
        command: command.clone(),
        wrapper_pid: std::process::id(),
        wrapper_birth: crate::managed_process::process_birth(std::process::id())
            .map_err(|e| e.to_string())?,
        thread_id: thread_id.into(),
    };
    write_json(&capability_path(root, command), &value).map_err(|e| e.to_string())
}

pub(crate) fn proof(
    status: &WorkspaceRepairStatusV1,
) -> Result<
    (
        crate::workspace_git::RepairAnchor,
        crate::delegation::WorkspaceRecord,
        RepairCapability,
    ),
    String,
> {
    let value = status
        .proof
        .as_ref()
        .ok_or("workspace_repair_proof_missing")?;
    Ok((
        serde_json::from_value(value["anchor"].clone())
            .map_err(|_| "workspace_repair_proof_invalid")?,
        serde_json::from_value(value["original_workspace"].clone())
            .map_err(|_| "workspace_repair_proof_invalid")?,
        serde_json::from_value(value["capability"].clone())
            .map_err(|_| "workspace_repair_proof_invalid")?,
    ))
}

pub(crate) fn persist_wrapper_result(
    root: &Path,
    command: &WorkerCommandIdentity,
    status: &WorkspaceRepairStatusV1,
) -> Result<(), String> {
    write_json(
        &result_path(root, command, &status.request.request_id),
        status,
    )
    .map_err(|e| e.to_string())
}
pub(crate) fn wrapper_result(
    root: &Path,
    command: &WorkerCommandIdentity,
    request: &WorkspaceRepairRequestV1,
) -> Result<Option<WorkspaceRepairStatusV1>, String> {
    let value: Option<WorkspaceRepairStatusV1> = read_optional(
        &result_path(root, command, &request.request_id),
        MAX_METADATA_BYTES,
    )
    .map_err(|e| e.to_string())?;
    if value.as_ref().is_some_and(|v| v.request != *request) {
        return Err("workspace_repair_envelope_mismatch".into());
    }
    Ok(value)
}

fn remap_policy(value: &mut serde_json::Value, old: &Path, target: &Path) {
    match value {
        serde_json::Value::String(text) => {
            if let Ok(offset) = Path::new(text).strip_prefix(old) {
                *text = if offset.as_os_str().is_empty() {
                    target.to_path_buf()
                } else {
                    target.join(offset)
                }
                .to_string_lossy()
                .into_owned();
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                remap_policy(item, old, target);
            }
        }
        serde_json::Value::Object(items) => {
            for item in items.values_mut() {
                remap_policy(item, old, target);
            }
        }
        _ => {}
    }
}
impl WorkerRuntime {
    fn save_repair_status(
        &self,
        command: &WorkerCommandIdentity,
        status: WorkspaceRepairStatusV1,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let _guard = self
            .store
            .transition
            .lock()
            .map_err(|_| WorkerRefusal::StoreUnavailable)?;
        self.store.health().map_err(refuse)?;
        let mut receipt = self
            .store
            .read_command(command)
            .map_err(refuse)?
            .ok_or(WorkerRefusal::UnknownExecution)?;
        // Le canon, le marqueur, la génération et la phase restent intacts.
        if let Some(previous) = receipt.workspace_repair.take() {
            if previous.request.request_id != status.request.request_id {
                if previous.phase != "refused" || receipt.workspace_repair_history.len() >= 7 {
                    return Err(WorkerRefusal::InvalidEnvelope);
                }
                receipt.workspace_repair_history.push(previous);
            }
        }
        receipt.workspace_repair = Some(status.clone());
        receipt.updated_at = worker_now();
        if serde_json::to_vec(&receipt).map_err(unavailable)?.len() > MAX_METADATA_BYTES {
            return Err(WorkerRefusal::ResourceLimit);
        }
        self.store
            .persist(&self.store.command_path(command), &receipt)
            .map_err(refuse)?;
        Ok(WorkerResponseOperation::WorkspaceRepair { status })
    }

    pub(super) fn publish_repair_status(
        &self,
        command: &WorkerCommandIdentity,
        status: WorkspaceRepairStatusV1,
        expires_at: i64,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        // Même verrou que dispatch_worker_inbound : l'annulation reçue et
        // la publication durable possèdent un ordre unique, après les relectures.
        let cancelled = self
            .cancelled
            .lock()
            .map_err(|_| WorkerRefusal::StoreUnavailable)?;
        if cancelled.contains(&json_digest(command).map_err(unavailable)?) {
            return Err(WorkerRefusal::Unauthorized);
        }
        let now = worker_now();
        if now >= expires_at || expires_at > now + 60 {
            return Err(WorkerRefusal::PrepareExpired);
        }
        self.save_repair_status(command, status)
    }

    pub(super) fn repair_workspace(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkspaceRepairRequestV1,
        action: WorkspaceRepairActionV1,
        expires_at: i64,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        request.validate()?;
        let mut receipt = self.store.command(command).map_err(refuse)?;
        // Lecture pure : un résultat publié reste réconciliable après arrêt/expiration.
        if action == WorkspaceRepairActionV1::Observe {
            let status = receipt
                .workspace_repair
                .iter()
                .chain(receipt.workspace_repair_history.iter())
                .find(|v| v.request.request_id == request.request_id)
                .ok_or(WorkerRefusal::UnknownExecution)?;
            if status.request != *request {
                return Err(WorkerRefusal::CommandMismatch);
            }
            return Ok(WorkerResponseOperation::WorkspaceRepair {
                status: status.clone(),
            });
        }

        // Un refus sans effet du wrapper est durable avant une autre clé.
        if let Some(previous) = receipt
            .workspace_repair
            .as_ref()
            .filter(|v| v.phase == "prepared")
        {
            if let Some(observed) =
                wrapper_result(&self.store.root, command, &previous.request).map_err(unavailable)?
            {
                if observed.phase == "refused"
                    && observed.prepared == previous.prepared
                    && observed.proof == previous.proof
                {
                    self.save_repair_status(command, observed)?;
                    receipt = self.store.command(command).map_err(refuse)?;
                }
            }
        }
        let refused = |code: &str| WorkerResponseOperation::WorkspaceRepair {
            status: WorkspaceRepairStatusV1 {
                request: request.clone(),
                phase: "refused".into(),
                prepared: None,
                proof: None,
                code: Some(code.into()),
            },
        };
        if !cfg!(target_os = "linux") {
            return Ok(refused("workspace_repair_platform_unsupported"));
        }
        if receipt.phase != WorkerCommandPhase::Released
            || !receipt.native_ready
            || receipt.startup_failed
        {
            return Ok(refused("workspace_repair_execution_unconfirmed"));
        }
        if self.cancelled.lock().map_or(true, |v| {
            v.contains(&json_digest(command).unwrap_or_default())
        }) {
            return Ok(refused("workspace_repair_cancelled"));
        }
        if let Some(previous) = receipt
            .workspace_repair_history
            .iter()
            .find(|v| v.request.request_id == request.request_id)
        {
            if previous.request != *request {
                return Ok(refused("workspace_repair_envelope_mismatch"));
            }
            return Ok(WorkerResponseOperation::WorkspaceRepair {
                status: previous.clone(),
            });
        }
        if let Some(previous) = receipt
            .workspace_repair
            .as_ref()
            .filter(|v| v.request.request_id == request.request_id || v.phase != "refused")
        {
            if previous.request != *request {
                return Ok(refused(
                    if previous.request.request_id == request.request_id {
                        "workspace_repair_envelope_mismatch"
                    } else {
                        "workspace_repair_already_recorded"
                    },
                ));
            }
            if previous.phase == "published" {
                return Ok(WorkerResponseOperation::WorkspaceRepair {
                    status: previous.clone(),
                });
            }
            if action == WorkspaceRepairActionV1::Prepare {
                // Observation seulement : un ACK perdu se réconcilie depuis le
                // blob durable, sans réveiller un wrapper déjà mis en barrière.
                if let Some(observed) =
                    wrapper_result(&self.store.root, command, request).map_err(unavailable)?
                {
                    if observed.prepared != previous.prepared || observed.proof != previous.proof {
                        return Err(WorkerRefusal::CommandMismatch);
                    }
                    return Ok(WorkerResponseOperation::WorkspaceRepair { status: observed });
                }
                return Ok(WorkerResponseOperation::WorkspaceRepair {
                    status: previous.clone(),
                });
            }
            if previous.phase == "committed" && action == WorkspaceRepairActionV1::Commit {
                return Ok(WorkerResponseOperation::WorkspaceRepair {
                    status: previous.clone(),
                });
            }
            if action == WorkspaceRepairActionV1::Publish && previous.phase != "committed" {
                return Ok(refused("workspace_repair_not_committed"));
            }
            let (anchor, original, capability) =
                proof(previous).map_err(|_| WorkerRefusal::CommandMismatch)?;
            let observed =
                wrapper_result(&self.store.root, command, request).map_err(unavailable)?;
            let Some(mut observed) = observed else {
                return Ok(WorkerResponseOperation::WorkspaceRepair {
                    status: previous.clone(),
                });
            };
            if observed.phase != "resumed"
                || observed.prepared != previous.prepared
                || observed.proof != previous.proof
                || observed.code.is_some()
            {
                if observed.phase == "refused" {
                    return self.save_repair_status(command, observed);
                }
                return Ok(refused("workspace_repair_resume_unconfirmed"));
            }
            if capability.worker != self.identity
                || capability.command != *command
                || crate::managed_process::process_birth(capability.wrapper_pid).ok()
                    != Some(capability.wrapper_birth)
                || receipt.marker.as_ref().is_none_or(|m| {
                    m.pgid != capability.wrapper_pid || m.birth != capability.wrapper_birth
                })
            {
                return Ok(refused("workspace_repair_wrapper_changed"));
            }
            crate::workspace_git::report_repair(&original, &anchor, true)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            let (request, prepared) = receipt
                .preparation
                .clone()
                .ok_or(WorkerRefusal::UnknownExecution)?;
            let mut retained = PreparedRuntime {
                routing: receipt.routing.clone(),
                command: WorkerCommandIdentity {
                    instance_id: String::new(),
                    generation: 0,
                    ..command.clone()
                },
                request,
                prepared: previous
                    .prepared
                    .clone()
                    .ok_or(WorkerRefusal::CommandMismatch)?,
                expires: Instant::now(),
            };
            // Reconstituer l'ancienne définition admise avant de vérifier la
            // nouvelle projection. Ni modèle ni effort ne viennent du caller.
            let registry = self.registry()?;
            let original_scoped = registry
                .for_inherited_delegation(
                    &retained.request.agent_type,
                    &retained.request.model,
                    retained.request.effort.as_deref(),
                    &prepared.child_policy,
                )
                .map_err(|_| WorkerRefusal::DefinitionChanged)?;
            if original_scoped
                .resolved_definition(&retained.request.agent_type)
                .map_err(|_| WorkerRefusal::DefinitionChanged)?
                != prepared.definition
            {
                return Err(WorkerRefusal::DefinitionChanged);
            }
            self.recheck_at_repaired(&mut retained, &anchor)
                .map_err(|_| WorkerRefusal::PermissionsChanged)?;
            observed.phase = if action == WorkspaceRepairActionV1::Publish {
                "published"
            } else {
                "committed"
            }
            .into();
            return self.publish_repair_status(command, observed, expires_at);
        }
        if action != WorkspaceRepairActionV1::Prepare {
            return Ok(refused("workspace_repair_not_prepared"));
        }
        if worker_now() >= expires_at || expires_at > worker_now() + 60 {
            return Ok(refused("workspace_repair_authority_expired"));
        }
        let admitted = receipt
            .preparation
            .as_ref()
            .ok_or(WorkerRefusal::UnknownExecution)?;
        if admitted.1.definition.protocol != "codex_app_server" {
            return Ok(refused("workspace_repair_adapter_unsupported"));
        }
        if !admitted
            .1
            .definition
            .capabilities
            .observed
            .as_ref()
            .is_some_and(|v| v.supports(bridget_transport::protocol::ProviderOperation::Resume))
        {
            return Ok(refused("workspace_repair_resume_unattested"));
        }
        let capability: Option<RepairCapability> = read_optional(
            &capability_path(&self.store.root, command),
            MAX_METADATA_BYTES,
        )
        .map_err(refuse)?;
        let Some(capability) = capability else {
            return Ok(refused("workspace_repair_child_upgrade_required"));
        };
        if capability.version != 1
            || capability.command != *command
            || capability.worker != self.identity
            || capability.thread_id.is_empty()
            || receipt.marker.as_ref().is_none_or(|m| {
                m.pgid != capability.wrapper_pid || m.birth != capability.wrapper_birth
            })
            || crate::managed_process::process_birth(capability.wrapper_pid).ok()
                != Some(capability.wrapper_birth)
        {
            return Ok(refused("workspace_repair_wrapper_changed"));
        }
        let (prepare, mut prepared) = receipt
            .preparation
            .clone()
            .ok_or(WorkerRefusal::UnknownExecution)?;
        let original: crate::delegation::WorkspaceRecord = serde_json::from_value(
            prepared
                .workspace
                .clone()
                .ok_or(WorkerRefusal::PlacementMismatch)?,
        )
        .map_err(unavailable)?;
        let target = crate::registry::validate_unprotected_local_path(
            Path::new(&request.target_cwd),
            &self.protected(),
        )
        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        let (anchor, record) = match crate::workspace_git::inspect_repair(
            &original,
            &target,
            &request.expected_head,
        ) {
            Ok(value) => value,
            Err(code) => return Ok(refused(&code)),
        };
        let registry = self.registry()?;
        if let Some(routing) = &receipt.routing {
            self.check_routing_connection(&registry, &prepare, routing)?;
        }
        let (mut mapping, cwd) = self.mapping_for(
            &prepare,
            &registry,
            receipt.routing.as_ref().map(|v| v.workload),
        )?;
        let execution_cwd = self.remap_prepared_worktree(&mut mapping, &cwd, &record)?;
        let definition = registry
            .get(&prepare.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        let original_scoped = registry
            .for_inherited_delegation(
                &prepare.agent_type,
                &prepare.model,
                prepare.effort.as_deref(),
                &prepared.child_policy,
            )
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        if original_scoped
            .resolved_definition(&prepare.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?
            != prepared.definition
        {
            return Err(WorkerRefusal::DefinitionChanged);
        }
        let (policy, posture) = match crate::native_permissions::project_remote_permissions(
            &prepare.parent,
            definition,
            &mapping,
            &execution_cwd,
            &self.source,
            prepare.posture,
            &self.protected(),
        ) {
            Ok(value) => value,
            Err(_) => return Ok(refused("workspace_repair_permissions_unconfirmed")),
        };
        let mut equivalent = prepared.child_policy.clone();
        remap_policy(&mut equivalent, Path::new(&original.path), &target);
        if policy != equivalent || posture != prepared.effective_posture {
            return Ok(refused("workspace_repair_permissions_changed"));
        }
        let projected = registry
            .for_inherited_delegation(
                &prepare.agent_type,
                &prepare.model,
                prepare.effort.as_deref(),
                &policy,
            )
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        prepared.definition = projected
            .resolved_definition(&prepare.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        prepared.placement.definition_digest = prepared.definition.digest.clone();
        prepared.placement.execution_cwd = execution_cwd.to_string_lossy().into_owned();
        prepared.placement.permission_digest = json_digest(&policy).map_err(refuse)?;
        prepared.child_policy = policy;
        prepared.workspace = Some(serde_json::to_value(record).map_err(unavailable)?);
        let status = WorkspaceRepairStatusV1 {
            request: request.clone(),
            phase: "prepared".into(),
            prepared: Some(prepared),
            proof: Some(serde_json::json!({
                "anchor":anchor,"original_workspace":original,"capability":capability,"expires_at":expires_at})),
            code: None,
        };
        // Réserver les blobs techniques avant de donner un contrôle au wrapper.
        self.store
            .admit((3 * MAX_METADATA_BYTES) as u64, 0)
            .map_err(refuse)?;
        self.save_repair_status(command, status)
    }

    pub(super) fn recheck_at_repaired(
        &self,
        retained: &mut PreparedRuntime,
        anchor: &crate::workspace_git::RepairAnchor,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at_with_repair(retained, false, Some(anchor))
    }
}

/// Lecture de la publication worker : ne rejoue ni Git ni fournisseur.
pub(crate) fn published(
    root: &Path,
    command: &WorkerCommandIdentity,
    request: &WorkspaceRepairRequestV1,
) -> bool {
    let receipt: Result<Option<WorkerCommandReceipt>, _> = read_optional(
        &root
            .join("commands")
            .join(format!("{}.json", marker_key(command))),
        MAX_METADATA_BYTES,
    );
    receipt
        .ok()
        .flatten()
        .filter(|v| v.identity == *command)
        .and_then(|v| v.workspace_repair)
        .is_some_and(|v| v.request == *request && v.phase == "published")
}

pub(super) fn collect_markers(root: &Path, receipt: &WorkerCommandReceipt) -> io::Result<()> {
    let markers = root.join("markers");
    match fs::symlink_metadata(&markers) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
        Ok(_) => check_private(&markers, true).map_err(|e| io::Error::other(e.to_string()))?,
    }
    let mut paths = vec![capability_path(root, &receipt.identity)];
    for status in receipt
        .workspace_repair
        .iter()
        .chain(receipt.workspace_repair_history.iter())
    {
        paths.push(result_path(
            root,
            &receipt.identity,
            &status.request.request_id,
        ));
    }
    for path in paths {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    File::open(root.join("markers"))?.sync_all()
}
