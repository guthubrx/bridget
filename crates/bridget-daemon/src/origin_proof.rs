//! Contrôle d'origine fermé : mêmes lecteurs locaux, aucun shell ni nouvel acteur.
use bridget_transport::protocol::*;

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}
pub(crate) fn execute(request: &OriginProofRequest, registry: &std::path::Path) -> OriginProofResult {
    execute_with_context(request, now_ms, |proof| {
        crate::t3code_mcp::reattest(proof).map_err(|_| WorkerRefusal::PermissionsChanged)
    }, Some(registry))
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
            || request.expires_at_ms.saturating_sub(current) > if matches!(
                &request.request, OriginProofOperation::PrepareProject {..}) {600_000} else {45_000}
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
            OriginProofOperation::Catalogue {parent,blocked} => {
                let before=crate::native_permissions::prove_parent_permissions(parent,parent.revision)
                    .map_err(|_|WorkerRefusal::PermissionsChanged)?;
                let registry=crate::registry::AgentRegistry::load_from_path(registry.ok_or(WorkerRefusal::Unauthorized)?.to_path_buf())
                    .map_err(|_|WorkerRefusal::DefinitionChanged)?;
                let entries=crate::daemon::catalogue_entries177(&registry,&crate::lifecycle::source_environment(),
                    &crate::provider_limits::ProviderLimits::default(),"origin-catalogue",&parent.provider_instance_id,
                    Some(parent),Some(std::path::Path::new(&parent.cwd)),blocked).map_err(|_|WorkerRefusal::PermissionsChanged)?;
                let after=crate::native_permissions::prove_parent_permissions(parent,parent.revision)
                    .map_err(|_|WorkerRefusal::PermissionsChanged)?;
                if before!=after {return Err(WorkerRefusal::PermissionsChanged);}
                OriginProofResult::Catalogue {permission_digest:after,entries}
            }
            OriginProofOperation::CommunicationProject {root,source,worktree} => {
                let project=crate::communication::resolve_communication_project(root,&request.worker.machine_id,*source,worktree.as_deref());
                OriginProofResult::CommunicationProject {project}
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
            OriginProofOperation::PrepareProject {parent,source,target_machine,project_id} => {
                let permission_digest = crate::native_permissions::prove_parent_permissions(parent,parent.revision)
                    .map_err(|_|WorkerRefusal::PermissionsChanged)?;
                let registry = registry.ok_or(WorkerRefusal::Unauthorized)?;
                let preparation_error = (|| {
                    let root=std::fs::canonicalize(&parent.cwd).map_err(|_|"project_preparation_path".to_owned())?;
                    let cwd=std::fs::canonicalize(&source.cwd).map_err(|_|"project_preparation_path".to_owned())?;
                    if root.to_string_lossy()!=parent.cwd || cwd.to_string_lossy()!=source.cwd || !cwd.starts_with(&root) {
                        return Err("cwd_outside_parent_project".to_owned());
                    }
                    crate::lifecycle::validate_source_base(source).map_err(closed_source_error)?;
                    if registry.parent().is_none_or(|root| !crate::project_preparation::has_link(root)) {
                        return Err("project_preparation_link_missing".into());
                    }
                    let budget=request.expires_at_ms.saturating_sub(now()).saturating_sub(2000);
                    if budget<=0 {return Err("project_preparation_timeout".into());}
                    let payload=serde_json::json!({"registry":registry,"machine_id":target_machine,
                        "origin_host":request.worker.machine_id,"source_root":parent.cwd,
                        "cwd":source.cwd,"commit":source.expected_base_commit,"project_id":project_id});
                    crate::project_preparation::prepare_until(&payload,
                        std::time::Instant::now()+std::time::Duration::from_millis(budget as u64))
                })().err();
                let after=crate::native_permissions::prove_parent_permissions(parent,parent.revision)
                    .map_err(|_|WorkerRefusal::PermissionsChanged)?;
                if permission_digest!=after {return Err(WorkerRefusal::PermissionsChanged);}
                OriginProofResult::ProjectPrepared {permission_digest,preparation_error}
            }
            OriginProofOperation::Sources { parent, source } => {
                let permission_digest =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let source_error = source.as_ref().and_then(|source| {
                    (|| {
                        let root = std::fs::canonicalize(&parent.cwd)
                            .map_err(|_| "workspace_preflight_technical".to_owned())?;
                        let cwd = std::fs::canonicalize(&source.cwd)
                            .map_err(|_| "workspace_preflight_technical".to_owned())?;
                        if !cwd.starts_with(&root) {
                            return Err("cwd_outside_parent_project".to_owned());
                        }
                        let mut source = source.clone();
                        source.cwd = cwd.to_string_lossy().into_owned();
                        crate::lifecycle::validate_source_base(&source).map_err(closed_source_error)
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

fn closed_source_error(code:String)->String {
    let prefix=code.split(':').next().unwrap_or("");
    if !prefix.is_empty() && prefix.len()<=128 && prefix.bytes().all(|b|b.is_ascii_lowercase()||b==b'_') {
        prefix.to_owned()
    } else {"workspace_preflight_technical".into()}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> OriginProofRequest {
        OriginProofRequest {
            version: 1,
            correlation_id: uuid::Uuid::new_v4().to_string(),
            connection_epoch: "epoch177".into(),
            worker: WorkerIdentity {
                machine_id: "origin177".into(),
                worker_id: uuid::Uuid::new_v4().to_string(),
                provisioning_revision: 1,
            },
            expires_at_ms: 20_000,
            request: OriginProofOperation::Attest {
                expected_thread: "thread177".into(),
                proof: NativePermissionProof {
                    endpoint: "http://127.0.0.1:1/mcp".into(),
                    authorization: IdentityCredential::new("Bearer private-177".into()),
                },
            },
        }
    }
    fn proof(thread: &str) -> OriginSessionProof {
        OriginSessionProof {
            version: 1,
            environment_id: "environment177".into(),
            thread_id: thread.into(),
            provider_session_id: "session177".into(),
            provider_instance_id: "instance177".into(),
            permissions: None,
        }
    }
    #[test]
    fn mobile177_origin_attestation_requires_expected_thread_and_live_deadline() {
        let r = request();
        assert!(matches!(
            execute_with(&r, || 10_000, |_| Ok(proof("thread177"))),
            OriginProofResult::Attested { .. }
        ));
        assert!(matches!(
            execute_with(&r, || 10_000, |_| Ok(proof("other"))),
            OriginProofResult::Rejected {
                reason: WorkerRefusal::Unauthorized
            }
        ));
        assert!(matches!(
            execute_with(
                &r,
                || 20_000,
                |_| panic!("expired request must not call provider")
            ),
            OriginProofResult::Rejected {
                reason: WorkerRefusal::PrepareExpired
            }
        ));
        let clock = std::cell::Cell::new(10_000);
        assert!(matches!(
            execute_with(
                &r,
                || clock.get(),
                |_| {
                    clock.set(20_001);
                    Ok(proof("thread177"))
                }
            ),
            OriginProofResult::Rejected {
                reason: WorkerRefusal::PrepareExpired
            }
        ));
        assert!(!format!("{:?}", r).contains("private-177"));
    }
    #[test]
    fn mobile177_origin_sources_read_real_git_and_reject_escape_without_launching(){
        use std::os::unix::fs::{DirBuilderExt,symlink};
        let root=std::env::temp_dir().canonicalize().unwrap().join(format!("bridget177-origin-git-{}",uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let source=root.join("source");std::fs::create_dir(&source).unwrap();
        let git=|args:&[&str]| {
            let result=std::process::Command::new("git").arg("-C").arg(&source).args(args).output().unwrap();
            assert!(result.status.success(),"fixture Git command failed");
            String::from_utf8(result.stdout).unwrap().trim().to_owned()
        };
        git(&["init","-q","-b","main"]);std::fs::write(source.join("file"),b"fixture177").unwrap();
        git(&["add","file"]);
        git(&["-c","user.name=Fixture","-c","user.email=fixture@example.com","commit","-qm","Fixture177"]);
        let head=git(&["rev-parse","HEAD"]);
        let parent=ProviderPermissions {version:1,source:"provider_turn".into(),run_id:"run177".into(),
            provider_session_id:"session177".into(),provider_instance_id:"instance177".into(),revision:1,driver:"codex_app_server".into(),
            cwd:source.to_string_lossy().into_owned(),runtime_mode:"full-access".into(),interaction_mode:"default".into(),
            provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}})};
        let mut envelope=request();envelope.request=OriginProofOperation::Sources {parent:parent.clone(),source:Some(WorkerSourceBase {
            cwd:parent.cwd.clone(),expected_base_commit:Some(head.clone()),workspace:Some(WorkspaceRequest::Shared {}),
            workload:AutoDelegateWorkload::General})};
        let prove=|request:&OriginProofRequest|execute_with(request,||10_000,|_|panic!("source check does not create a provider session"));
        assert!(matches!(prove(&envelope),OriginProofResult::Sources {source_error:None,..}));
        if let OriginProofOperation::Sources {source:Some(source),..}=&mut envelope.request {source.expected_base_commit=Some("b".repeat(40));}
        assert!(matches!(prove(&envelope),OriginProofResult::Sources {source_error:Some(code),..} if code=="expected_base_commit_mismatch"));
        let outside=root.join("outside");std::fs::create_dir(&outside).unwrap();symlink(&outside,source.join("escape")).unwrap();
        if let OriginProofOperation::Sources {source:Some(input),..}=&mut envelope.request {input.cwd=source.join("escape").to_string_lossy().into_owned();}
        assert!(matches!(prove(&envelope),OriginProofResult::Sources {source_error:Some(code),..} if code=="cwd_outside_parent_project"));
        envelope.request=OriginProofOperation::PrepareProject {parent:parent.clone(),
            source:WorkerSourceBase {cwd:parent.cwd.clone(),expected_base_commit:Some(head.clone()),
                workspace:Some(WorkspaceRequest::Shared {}),workload:AutoDelegateWorkload::General},
            target_machine:"target177".into(),project_id:None};
        assert!(matches!(execute_with_context(&envelope,||10_000,|_|panic!("no provider"),
            Some(&root.join("agents.json"))),OriginProofResult::ProjectPrepared {preparation_error:Some(code),..}
            if code=="project_preparation_link_missing"));
        assert!(matches!(prove(&envelope),OriginProofResult::Rejected {reason:WorkerRefusal::Unauthorized}));
        if let OriginProofOperation::PrepareProject {source,..}=&mut envelope.request {
            source.expected_base_commit=Some("b".repeat(40));
        }
        assert!(matches!(execute_with_context(&envelope,||10_000,|_|panic!("no provider"),
            Some(&root.join("agents.json"))),OriginProofResult::ProjectPrepared {preparation_error:Some(code),..}
            if code=="expected_base_commit_mismatch"));
        if let OriginProofOperation::PrepareProject {target_machine,..}=&mut envelope.request {
            *target_machine="origin177".into();
        }
        assert!(envelope.validate_shape().is_err());
        envelope.request=OriginProofOperation::Catalogue {parent:parent.clone(),blocked:vec![]};
        assert!(matches!(prove(&envelope),OriginProofResult::Rejected {reason:WorkerRefusal::Unauthorized}));
        let catalogue=execute_with_context(&envelope,||10_000,|_|panic!("catalogue does not launch a provider"),
            Some(&root.join("agents.json")));
        let OriginProofResult::Catalogue {permission_digest,entries}=catalogue else {panic!("origin catalogue refused");};
        assert_eq!(permission_digest,crate::native_permissions::prove_parent_permissions(&parent,parent.revision).unwrap());
        assert!(!entries.is_empty());
        assert!(entries.len()<=128);
        assert!(entries.iter().all(|entry|entry["agent_type"].is_string()));
        assert!(!root.join("agents.json").exists());
        assert!(!root.join("bridget.db").exists());
        assert_eq!(git(&["rev-parse","HEAD"]),head);
        assert!(!source.join(".worktrees").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

}
