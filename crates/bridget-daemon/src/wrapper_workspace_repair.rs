//! Le wrapper reste propriétaire de son fournisseur pendant la réparation185.
use super::*;
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::WorkspaceRepairStatusV1;

struct RetainedRepairSession {
    owner: Mutex<Box<bridget_transport::transport::RetainedStart>>,
    descriptor: ManagedSessionDescriptor,
    connection_id: String,
}
impl bridget_transport::Transport for RetainedRepairSession {
    fn deliver(
        &mut self,
        _: &bridget_core::BridgetMessage,
    ) -> Result<(), bridget_transport::TransportError> {
        Err(bridget_transport::TransportError::AgentDead)
    }
    fn is_alive(&self) -> bool {
        false
    }
    fn connection_id(&self) -> &str {
        &self.connection_id
    }
}
impl ManagedSession for RetainedRepairSession {
    fn descriptor(&self) -> ManagedSessionDescriptor {
        self.descriptor.clone()
    }
    fn process_id(&self) -> u32 {
        self.owner.lock().recover_poison().process_id()
    }
    fn activate_journal(
        &self,
        _: &Path,
        _: &str,
        _: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        Err(std::io::Error::other("reprise fournisseur non confirmée"))
    }
    fn drain_events(&self) -> Vec<bridget_transport::ManagedEvent> {
        Vec::new()
    }
    fn cancel_delivery(&self, _: &str, _: &str) -> bool {
        false
    }
    fn stop(&self) {
        if let Ok(mut owner) = self.owner.try_lock() {
            let _ = owner.stop(Duration::from_secs(4));
        }
    }
    fn is_busy(&self) -> bool {
        false
    }
}

pub(super) struct ApplyOutcome {
    pub status: WorkspaceRepairStatusV1,
    pub pending_receipt: Option<WorkspaceRepairStatusV1>,
}
pub(super) fn persist_pending(
    context: &WorkerResultContext,
    status: &WorkspaceRepairStatusV1,
) -> bool {
    crate::remote_worker::workspace_repair::persist_wrapper_result(
        &context.store_root,
        &context.command,
        status,
    )
    .is_ok()
}

pub(super) fn apply(
    task_id: &str,
    mut status: WorkspaceRepairStatusV1,
    remote: Option<&WorkerResultContext>,
    transport: &mut Box<dyn ManagedSession>,
    agent_type: &str,
    definition: &crate::registry::AgentDefinition,
    native_args: &[String],
    environment: &[(OsString, OsString)],
    mcp_servers: Vec<serde_json::Value>,
    home: &Path,
    name: &str,
    instance: &str,
    socket: &Path,
    cancelled: Option<&NativeTerminationSignal>,
) -> ApplyOutcome {
    let cancelled = || cancelled.is_some_and(NativeTerminationSignal::requested);
    let result = (|| -> Result<(), String> {
        let context = remote.ok_or("workspace_repair_remote_required")?;
        if context.command.task_id != task_id || context.command.instance_id != instance {
            return Err("workspace_repair_identity_unconfirmed".into());
        }
        status
            .request
            .validate()
            .map_err(|_| "workspace_repair_invalid_request")?;
        let (anchor, original, capability) =
            crate::remote_worker::workspace_repair::proof(&status)?;
        if capability.worker != context.worker
            || capability.command != context.command
            || capability.wrapper_pid != std::process::id()
            || crate::managed_process::process_birth(std::process::id()).ok()
                != Some(capability.wrapper_birth)
            || definition.protocol != "codex_app_server"
        {
            return Err("workspace_repair_child_incompatible".into());
        }
        let prepared = status
            .prepared
            .as_ref()
            .ok_or("workspace_repair_proof_missing")?
            .clone();
        let expires = status
            .proof
            .as_ref()
            .and_then(|v| v["expires_at"].as_i64())
            .ok_or("workspace_repair_authority_expired")?;

        let mut checkpoint = crate::remote_worker::workspace_repair::wrapper_result(
            &context.store_root,
            &context.command,
            &status.request,
        )?;
        if let Some(previous) = &checkpoint {
            if previous.prepared != status.prepared || previous.proof != status.proof {
                return Err("workspace_repair_envelope_mismatch".into());
            }
            if previous.phase == "resumed" {
                let current = transport
                    .provider_identity()
                    .ok_or("workspace_repair_session_unconfirmed")?;
                if current.provider_thread_id.as_deref() != Some(capability.thread_id.as_str())
                    || std::env::current_dir().ok().as_deref()
                        != Some(Path::new(&prepared.placement.execution_cwd))
                {
                    return Err("workspace_repair_resume_unconfirmed".into());
                }
                status = previous.clone();
                return Ok(());
            }
            if previous.code.is_some() {
                return Err(previous.code.clone().unwrap());
            }
        }
        if unix_now_secs() >= expires {
            return Err("workspace_repair_authority_expired".into());
        }
        if cancelled() {
            return Err("workspace_repair_cancelled".into());
        }
        if checkpoint.is_none() {
            let identity = transport
                .provider_identity()
                .ok_or("workspace_repair_session_unconfirmed")?;
            if identity.provider_thread_id.as_deref() != Some(capability.thread_id.as_str()) {
                return Err("workspace_repair_session_unconfirmed".into());
            }
            // Revalider le dossier avant la première action sur le fournisseur.
            crate::workspace_git::report_repair(&original, &anchor, false)?;
            let stopped = transport
                .stop_for_workspace_repair(Duration::from_secs(8))
                .map_err(|e| match e {
                    bridget_transport::TransportError::DeliveryFailed(code)
                        if code.starts_with("workspace_repair_") =>
                    {
                        code
                    }
                    error => format!("workspace_repair_stop:{error}"),
                })?;
            if stopped.provider_thread_id.as_deref() != Some(capability.thread_id.as_str()) {
                return Err("workspace_repair_session_unconfirmed".into());
            }
            status.phase = "stopped".into();
            crate::remote_worker::workspace_repair::persist_wrapper_result(
                &context.store_root,
                &context.command,
                &status,
            )?;
            checkpoint = Some(status.clone());
        }
        if checkpoint.as_ref().is_none_or(|v| v.phase != "stopped") {
            return Err("workspace_repair_pending".into());
        }
        if cancelled() {
            return Err("workspace_repair_cancelled".into());
        }
        crate::workspace_git::apply_repair_git(&original, &anchor)?;
        if cancelled() {
            return Err("workspace_repair_cancelled".into());
        }
        // Aucun fil fournisseur ne peut encore employer l'ancien cwd.
        std::env::set_current_dir(&prepared.placement.execution_cwd)
            .map_err(|_| "workspace_repair_destination_changed")?;
        crate::workspace_git::report_repair(&original, &anchor, true)?;
        let mut args = native_args.to_vec();
        crate::native_permissions::apply_child_arguments(
            &definition.protocol,
            &prepared.child_policy,
            &mut args,
        )?;
        let mut env = environment.to_vec();
        env.push((
            crate::native_permissions::CHILD_POLICY_ENV.into(),
            serde_json::to_string(&prepared.child_policy)
                .map_err(|_| "workspace_repair_proof_invalid")?
                .into(),
        ));
        if unix_now_secs() >= expires {
            return Err("workspace_repair_authority_expired".into());
        }
        if cancelled() {
            return Err("workspace_repair_cancelled".into());
        }
        let new = match spawn_managed_session_transport(
            agent_type,
            definition,
            &args,
            &env,
            mcp_servers,
            true,
            home,
            Some(name.into()),
            instance,
            socket,
            Some((&prepared.child_policy, &capability.thread_id)),
        ) {
            Ok(new) => new,
            Err(error) => {
                let code = format!("workspace_repair_resume:{error}");
                if let Ok(error) = error.downcast::<bridget_transport::TransportError>()
                    && let bridget_transport::TransportError::StartUnconfirmed { owner, .. } =
                        *error
                {
                    let descriptor = transport.descriptor();
                    *transport = Box::new(RetainedRepairSession {
                        owner: Mutex::new(owner),
                        descriptor,
                        connection_id: "workspace-repair-unconfirmed".into(),
                    });
                }
                return Err(code);
            }
        };
        // Conserver l'unique transport, même si son attestation suivante échoue.
        *transport = new;
        if cancelled() {
            return Err("workspace_repair_cancelled".into());
        }
        let identity = transport
            .provider_identity()
            .ok_or("workspace_repair_resume_unconfirmed")?;
        if identity.provider_thread_id.as_deref() != Some(capability.thread_id.as_str()) {
            return Err("workspace_repair_resume_unconfirmed".into());
        }
        let (_, policy, _, cwd) = transport
            .provider_permissions()
            .ok_or("workspace_repair_permissions_unconfirmed")?;
        if cwd.as_deref() != Some(prepared.placement.execution_cwd.as_str())
            || ["approval_policy", "approvals_reviewer", "sandbox_policy"]
                .iter()
                .any(|key| policy.get(*key) != prepared.child_policy.get(*key))
        {
            return Err("workspace_repair_permissions_changed".into());
        }
        status.phase = "resumed".into();
        status.code = None;
        // L'écriture finale unique reste hors du traitement des erreurs
        // fournisseur : sa panne ne transforme pas une reprise attestée en refus.
        Ok(())
    })();
    if let Err(mut code) = result {
        // Un fournisseur repris mais non attesté ne reçoit aucun message.
        // Le fermer aussi sous borne, sans lancer de remplaçant.
        if status.phase == "stopped" && transport.is_alive() {
            transport.drain_events();
            if transport
                .stop_for_workspace_repair(Duration::from_secs(8))
                .is_err()
            {
                code = "workspace_repair_stop_unconfirmed".into();
            }
        }
        // Un arrêt non confirmé ferme les remises ; aucun transport de repli.
        if status.phase == "prepared" {
            status.phase = if transport.is_alive() {
                "refused"
            } else {
                "stopped"
            }
            .into();
        }
        status.code = Some(code);
    }
    // Le maître ne voit jamais un refus terminal avant sa persistance.
    // Une erreur de disque ferme les admissions ; le wrapper garde le vrai
    // résultat et ne rejoue aucun effet pendant ses tentatives d'écriture.
    let mut pending_receipt = None;
    if let Some(context) =
        remote.filter(|v| v.command.task_id == task_id && v.command.instance_id == instance)
        && !persist_pending(context, &status)
    {
        pending_receipt = Some(status.clone());
        if status.phase == "refused" {
            status.phase = "prepared".into();
        }
        status.code = Some("workspace_repair_receipt_unconfirmed".into());
    }
    ApplyOutcome {
        status,
        pending_receipt,
    }
}

#[cfg(test)]
#[path = "wrapper_workspace_repair_tests.rs"]
mod tests;
