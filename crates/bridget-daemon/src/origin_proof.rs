//! Contrôle d'origine fermé : mêmes lecteurs locaux, aucun shell ni nouvel acteur.
use bridget_transport::protocol::*;

/// Même borne que les attentes du maître. Une preuve reste indépendante par
/// appel ; aucune réutilisation de droits ou session HTTP partagée.
pub(crate) const MAX_ORIGIN_PROOFS: usize = 32;
/// Préparations déjà reçues, actives ou en attente du verrou de projet.
/// L'échéance de la requête couvre aussi l'attente ; aucune remise métier.
pub(crate) const MAX_ORIGIN_PROJECT_PREPARATIONS: usize = 16;

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}
pub(crate) fn execute(
    request: &OriginProofRequest,
    registry: &std::path::Path,
) -> OriginProofResult {
    execute_with_context(
        request,
        now_ms,
        |proof| crate::t3code_mcp::reattest(proof).map_err(attestation_worker_refusal),
        Some(registry),
    )
}

fn attestation_worker_refusal(error: crate::mcp_identity::IdentityError) -> WorkerRefusal {
    use crate::mcp_identity::IdentityError;
    match error {
        IdentityError::T3SessionUnavailableAt(detail)
            if detail.starts_with("permission_attestation_unavailable :")
                || detail.starts_with("parent_not_active :") =>
        {
            WorkerRefusal::PermissionsChanged
        }
        IdentityError::T3SessionUnavailableAt(detail)
            if detail.starts_with("attestation_tool_refused :") =>
        {
            WorkerRefusal::Unauthorized
        }
        _ => WorkerRefusal::WorkerUnavailable,
    }
}

#[cfg(test)]
mod attestation240_tests {
    use super::*;
    #[test]
    fn spec240_typed_attestation_refusal_is_distinct_from_transport_failure() {
        use crate::mcp_identity::IdentityError;
        for detail in [
            "permission_attestation_unavailable : refus natif",
            "parent_not_active : tour terminé",
        ] {
            assert_eq!(
                attestation_worker_refusal(IdentityError::T3SessionUnavailableAt(detail)),
                WorkerRefusal::PermissionsChanged
            );
        }
        assert_eq!(
            attestation_worker_refusal(IdentityError::T3SessionUnavailableAt(
                "attestation_tool_refused : motif inconnu"
            )),
            WorkerRefusal::Unauthorized
        );
        assert_eq!(
            attestation_worker_refusal(IdentityError::T3SessionUnavailable),
            WorkerRefusal::WorkerUnavailable
        );
        assert_eq!(
            attestation_worker_refusal(IdentityError::T3SessionUnavailableAt(
                "unrecognized detail"
            )),
            WorkerRefusal::WorkerUnavailable
        );
    }
}
#[cfg(test)]
fn execute_with(
    request: &OriginProofRequest,
    now: impl Fn() -> i64,
    attest: impl FnOnce(&NativePermissionProof) -> Result<OriginSessionProof, WorkerRefusal>,
) -> OriginProofResult {
    execute_with_context(request, now, attest, None)
}
fn execute_with_context(
    request: &OriginProofRequest,
    now: impl Fn() -> i64,
    attest: impl FnOnce(&NativePermissionProof) -> Result<OriginSessionProof, WorkerRefusal>,
    registry: Option<&std::path::Path>,
) -> OriginProofResult {
    let check = || {
        let current = now();
        if request.expires_at_ms <= current
            || request.expires_at_ms.saturating_sub(current)
                > if matches!(
                    &request.request,
                    OriginProofOperation::PrepareProject { .. }
                ) {
                    600_000
                } else {
                    45_000
                }
        {
            Err(WorkerRefusal::PrepareExpired)
        } else {
            Ok(())
        }
    };
    let result = (|| {
        request.validate_shape()?;
        check()?;
        let result = match &request.request {
            OriginProofOperation::Catalogue { parent, blocked } => {
                let before =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let registry = crate::registry::AgentRegistry::load_from_path(
                    registry.ok_or(WorkerRefusal::Unauthorized)?.to_path_buf(),
                )
                .map_err(|_| WorkerRefusal::DefinitionChanged)?;
                let entries = crate::daemon::catalogue_entries177(
                    &registry,
                    &crate::lifecycle::source_environment(),
                    &crate::provider_limits::ProviderLimits::default(),
                    "origin-catalogue",
                    &parent.provider_instance_id,
                    Some(parent),
                    Some(std::path::Path::new(&parent.cwd)),
                    blocked,
                )
                .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let after =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                if before != after {
                    return Err(WorkerRefusal::PermissionsChanged);
                }
                OriginProofResult::Catalogue {
                    permission_digest: after,
                    entries,
                }
            }
            OriginProofOperation::CommunicationProject {
                root,
                source,
                worktree,
            } => {
                let project = crate::communication::resolve_communication_project(
                    root,
                    &request.worker.machine_id,
                    *source,
                    worktree.as_deref(),
                );
                OriginProofResult::CommunicationProject { project }
            }
            OriginProofOperation::Attest {
                expected_thread,
                proof,
            } => {
                let proof = attest(proof)?;
                if &proof.thread_id != expected_thread {
                    return Err(WorkerRefusal::Unauthorized);
                }
                OriginProofResult::Attested { proof }
            }
            OriginProofOperation::PrepareProject {
                parent,
                source,
                target_machine,
                project_id,
                return_context,
            } => {
                let permission_digest =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let registry = registry.ok_or(WorkerRefusal::Unauthorized)?;
                let preparation = (|| {
                    let selected =
                        crate::lifecycle::source_root(parent, std::path::Path::new(&source.cwd))?;
                    let root = std::fs::canonicalize(&selected)
                        .map_err(|_| "project_preparation_path".to_owned())?;
                    let cwd = std::fs::canonicalize(&source.cwd)
                        .map_err(|_| "project_preparation_path".to_owned())?;
                    if ((!crate::native_permissions::unrestricted_source_scope(parent))
                        && (root != selected || cwd.to_string_lossy() != source.cwd))
                        || !cwd.starts_with(&root)
                    {
                        return Err("cwd_outside_parent_project".to_owned());
                    }
                    crate::lifecycle::validate_parent_source_base(parent, source)
                        .map_err(closed_source_error)?;
                    let local_worker_registry = source
                        .is_local_shared_directory(&request.worker.machine_id, target_machine)
                        || (request.worker.machine_id == *target_machine
                            && matches!(source.workspace, Some(WorkspaceRequest::CommitView {})));
                    if !local_worker_registry
                        && registry
                            .parent()
                            .is_none_or(|root| !crate::project_preparation::has_link(root))
                    {
                        return Err("project_preparation_link_missing".into());
                    }
                    let budget = request
                        .expires_at_ms
                        .saturating_sub(now())
                        .saturating_sub(2000);
                    if budget <= 0 {
                        return Err("project_preparation_timeout".into());
                    }
                    let before = crate::registry::AgentRegistry::load_from_path_quiet(
                        registry.to_path_buf(),
                    )
                    .map_err(|_| "project_preparation_registry_changed".to_owned())?;
                    let expected = before.machine(target_machine)?;
                    let payload = serde_json::json!({"registry":registry,"machine_id":target_machine,
                        "correlation_id":request.correlation_id,
                        "origin_host":request.worker.machine_id,"source_root":selected,
                        "cwd":source.cwd,"commit":source.expected_base_commit,"project_id":project_id,
                        "origin_worker_registry":local_worker_registry,
                        "objects_only":matches!(source.workspace,Some(WorkspaceRequest::CommitView {}))});
                    let prepared = crate::project_preparation::prepare_receipt_until(
                        &payload,
                        std::time::Instant::now() + std::time::Duration::from_millis(budget as u64),
                    )?;
                    // Sans négociation, conserver exactement la réponse historique.
                    if !return_context {
                        return Ok(None);
                    }
                    let current = crate::registry::AgentRegistry::load_from_path_quiet(
                        registry.to_path_buf(),
                    )
                    .map_err(|_| "project_preparation_registry_changed".to_owned())?;
                    let target = current.machine(target_machine)?;
                    if target.worker_id != expected.worker_id
                        || target.provisioning_revision != expected.provisioning_revision
                        || target.credential_fingerprint != expected.credential_fingerprint
                    {
                        return Err("project_preparation_machine_mismatch".to_owned());
                    }
                    let project_id = prepared["project_id"]
                        .as_str()
                        .ok_or("project_preparation_context_missing")?
                        .to_owned();
                    let mapping: crate::registry::ProjectMapping =
                        serde_json::from_value(prepared["mapping"].clone())
                            .map_err(|_| "project_preparation_context_missing".to_owned())?;
                    if target.mappings.get(&project_id) != Some(&mapping) {
                        return Err("project_preparation_mapping_changed".to_owned());
                    }
                    Ok::<_, String>(Some(PreparedOriginProject {
                        target: WorkerIdentity {
                            machine_id: target_machine.clone(),
                            worker_id: target.worker_id.clone(),
                            provisioning_revision: target.provisioning_revision,
                        },
                        credential_fingerprint: target.credential_fingerprint.clone(),
                        project_id,
                        mapping: prepared["mapping"].clone(),
                    }))
                })();
                let (project, preparation_error) = match preparation {
                    Ok(project) => (project, None),
                    Err(code) => (None, Some(code)),
                };
                let after =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                if permission_digest != after {
                    return Err(WorkerRefusal::PermissionsChanged);
                }
                OriginProofResult::ProjectPrepared {
                    permission_digest,
                    preparation_error,
                    project,
                }
            }
            OriginProofOperation::Sources { parent, source } => {
                let permission_digest =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let source_error = source.as_ref().and_then(|source| {
                    (|| {
                        let root = std::fs::canonicalize(crate::lifecycle::source_root(
                            parent,
                            std::path::Path::new(&source.cwd),
                        )?)
                        .map_err(|_| "workspace_preflight_technical".to_owned())?;
                        let cwd = std::fs::canonicalize(&source.cwd)
                            .map_err(|_| "workspace_preflight_technical".to_owned())?;
                        if !cwd.starts_with(&root) {
                            return Err("cwd_outside_parent_project".to_owned());
                        }
                        let mut source = source.clone();
                        source.cwd = cwd.to_string_lossy().into_owned();
                        crate::lifecycle::validate_parent_source_base(parent, &source)
                            .map_err(closed_source_error)
                    })()
                    .err()
                });
                let after =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                if permission_digest != after {
                    return Err(WorkerRefusal::PermissionsChanged);
                }
                OriginProofResult::Sources {
                    permission_digest,
                    source_error,
                }
            }
        };
        check()?;
        Ok(result)
    })();
    result.unwrap_or_else(|reason| OriginProofResult::Rejected { reason })
}

fn closed_source_error(code: String) -> String {
    let prefix = code.split(':').next().unwrap_or("");
    if !prefix.is_empty()
        && prefix.len() <= 128
        && prefix.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
    {
        prefix.to_owned()
    } else {
        "workspace_preflight_technical".into()
    }
}

#[cfg(test)]
#[path = "origin_proof_tests.rs"]
mod tests;
