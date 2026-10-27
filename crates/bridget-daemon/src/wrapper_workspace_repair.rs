//! Le wrapper reste propriétaire de son fournisseur pendant la réparation185.
use super::*;
use bridget_transport::protocol::WorkspaceRepairStatusV1;

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
) -> WorkspaceRepairStatusV1 {
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
        if unix_now_secs() >= expires {
            return Err("workspace_repair_authority_expired".into());
        }

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
                .map_err(|e| format!("workspace_repair_stop:{e}"))?;
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
        crate::workspace_git::apply_repair_git(&original, &anchor)?;
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
        let new = spawn_managed_session_transport(
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
        )
        .map_err(|e| format!("workspace_repair_resume:{e}"))?;
        // Conserver l'unique transport, même si son attestation suivante échoue.
        *transport = new;
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
        crate::remote_worker::workspace_repair::persist_wrapper_result(
            &context.store_root,
            &context.command,
            &status,
        )?;
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
        if let Some(context) = remote {
            let _ = crate::remote_worker::workspace_repair::persist_wrapper_result(
                &context.store_root,
                &context.command,
                &status,
            );
        }
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;

    fn git(root: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().into()
    }

    #[test]
    fn spec185_wrapper_repairs_deleted_cwd_and_resumes_same_thread_without_business_turn() {
        const KEY: &str = "BRIDGET_REPAIR185_FIXTURE_ROOT";
        if let Some(root) = std::env::var_os(KEY) {
            exercise(Path::new(&root));
            return;
        }
        let root = std::env::temp_dir().join(format!("wrapper-repair185-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let log = std::fs::File::create(root.join("test.log")).unwrap();
        let mut child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","wrapper::workspace_repair::tests::spec185_wrapper_repairs_deleted_cwd_and_resumes_same_thread_without_business_turn","--nocapture"])
            .env(KEY,&root).env_remove(crate::native_permissions::CHILD_POLICY_ENV)
            .env_remove(crate::native_permissions::CHILD_SOURCES_ENV)
            .env_remove("GIT_DIR").env_remove("GIT_WORK_TREE").env_remove("GIT_COMMON_DIR")
            .stdout(log.try_clone().unwrap()).stderr(log).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "fixture185 hors délai, PID {}",
                child.id()
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        let diagnostic = std::fs::read_to_string(root.join("test.log")).unwrap();
        let result = status.success();
        // Le fils garde ses traces jusqu'à ce reçu ; aucune donnée métier réelle.
        if result {
            std::fs::remove_dir_all(&root).unwrap();
        }
        assert!(
            result,
            "fixture conservée {} : {diagnostic}",
            root.display()
        );
    }

    fn exercise(root: &Path) {
        let repo = root.join("repo");
        std::fs::create_dir(&repo).unwrap();
        git(&repo, &["init", "-q"]);
        std::fs::write(repo.join(".gitignore"), ".worktrees/\n").unwrap();
        std::fs::write(repo.join("tracked"), "conservé\n").unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-qm", "base"]);
        let original = crate::workspace_git::create(
            &crate::workspace_git::preflight(&repo, None, None).unwrap(),
            &uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let original_path = Path::new(&original.path);
        std::fs::write(original_path.join("untracked"), "conservé aussi\n").unwrap();
        let head = git(original_path, &["rev-parse", "HEAD"]);
        let trace = root.join("rpc.jsonl");
        let script = root.join("provider.py");
        std::fs::write(&script,r#"
import json,sys,os
for line in sys.stdin:
 q=json.loads(line)
 with open(sys.argv[1],"a") as log: log.write(line)
 method=q.get("method")
 if "id" not in q: continue
 result={}
 if method=="initialize": result={"userAgent":"fixture185","codexHome":"/tmp","platformFamily":"unix","platformOs":"linux"}
 if method=="config/read": result={"config":{"approval_policy":"never","sandbox_mode":"read-only"}}
 if method in ("thread/start","thread/resume"):
  result={"thread":{"id":"same-thread185"},"cwd":q.get("params",{}).get("cwd",os.getcwd()),
   "approvalPolicy":"never","approvalsReviewer":"user","sandboxPolicy":{"type":"readOnly","networkAccess":False}}
 if method=="turn/start": raise RuntimeError("aucun tour métier autorisé")
 print(json.dumps({"id":q["id"],"result":result}),flush=True)
"#).unwrap();
        let registry=crate::registry::AgentRegistry::from_json(&json!({"agents":{"repair185":{
            "command":"/usr/bin/python3","args":["-u",script,trace],"protocol":"codex_app_server",
            "permissions":"deny","forbidden_env":[],
            "capabilities":{"execution_paths":["codex_app_server"],"models":{"fixture185":{"efforts":["high"]}},
                "observed":{"binary_path":"/usr/bin/python3","binary_version":"fixture185",
                    "binary_digest":"0".repeat(64),"contract_version":"codex-app-server-fixture-v1",
                    "operations":["resume"]}}
        }}}).to_string(),&root.join("registry.json")).unwrap();
        let definition = registry.get("repair185").unwrap();
        std::env::set_current_dir(original_path).unwrap();
        let env: Vec<(OsString, OsString)> = Vec::new();
        let mut transport = spawn_managed_session_transport(
            "repair185",
            definition,
            &definition.args,
            &env,
            Vec::new(),
            true,
            root,
            Some("child185".into()),
            "instance185",
            &root.join("bridget.sock"),
            None,
        )
        .unwrap();
        transport.drain_events();
        let old_pid = transport.process_id();
        // Copier puis supprimer simule le changement d'inode du déplacement
        // inter-volumes, pendant que le cwd du pilote est encore vivant.
        let target = root.join("disk185");
        std::fs::create_dir(&target).unwrap();
        for item in std::fs::read_dir(original_path).unwrap() {
            let item = item.unwrap();
            assert!(item.file_type().unwrap().is_file());
            std::fs::copy(item.path(), target.join(item.file_name())).unwrap();
        }
        std::fs::remove_dir_all(original_path).unwrap();
        std::os::unix::fs::symlink(&target, original_path).unwrap();
        assert!(std::env::current_dir().is_err());
        let (anchor, record) =
            crate::workspace_git::inspect_repair(&original, &target, &head).unwrap();
        let store = root.join("worker");
        crate::environment::ensure_private_directory(&store.join("markers")).unwrap();
        let context = WorkerResultContext {
            worker: bridget_transport::protocol::WorkerIdentity {
                machine_id: "fixture185".into(),
                worker_id: "worker185".into(),
                provisioning_revision: 1,
            },
            command: bridget_transport::protocol::WorkerCommandIdentity {
                command_id: "command185".into(),
                task_id: "task185".into(),
                instance_id: "instance185".into(),
                generation: 1,
            },
            store_root: store,
        };
        let capability = crate::remote_worker::workspace_repair::RepairCapability {
            version: 1,
            worker: context.worker.clone(),
            command: context.command.clone(),
            wrapper_pid: std::process::id(),
            wrapper_birth: crate::managed_process::process_birth(std::process::id()).unwrap(),
            thread_id: "same-thread185".into(),
        };
        let policy = json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user",
            "sandbox_policy":{"type":"readOnly","networkAccess":false}});
        let placement = bridget_transport::protocol::RemotePlacement {
            machine_id: "fixture185".into(),
            worker_id: "worker185".into(),
            provisioning_revision: 1,
            project_id: "fixture185".into(),
            mapping_revision: 1,
            origin_host: "origin185".into(),
            origin_cwd: "/source185".into(),
            source_root_identity: "fixture185".into(),
            relative_cwd: ".".into(),
            execution_cwd: target.to_string_lossy().into_owned(),
            base_commit: Some(head.clone()),
            definition_digest: "0".repeat(64),
            permission_digest: "0".repeat(64),
        };
        let status = WorkspaceRepairStatusV1 {
            request: bridget_transport::protocol::WorkspaceRepairRequestV1 {
                request_id: "repair185".into(),
                target_cwd: target.to_string_lossy().into_owned(),
                expected_head: head,
            },
            phase: "prepared".into(),
            code: None,
            prepared: Some(bridget_transport::protocol::WorkerPrepared {
                placement,
                workspace: Some(serde_json::to_value(record).unwrap()),
                token: "unused185".into(),
                definition: registry.resolved_definition("repair185").unwrap(),
                child_policy: policy,
                effective_posture: bridget_transport::protocol::SpawnPosture::Discovery,
                prepared_at: unix_now_secs(),
            }),
            proof: Some(
                json!({"anchor":anchor,"original_workspace":original,"capability":capability,
                "expires_at":unix_now_secs()+60}),
            ),
        };
        let result = apply(
            "task185",
            status.clone(),
            Some(&context),
            &mut transport,
            "repair185",
            definition,
            &definition.args,
            &env,
            Vec::new(),
            root,
            "child185",
            "instance185",
            &root.join("bridget.sock"),
        );
        assert_eq!(result.phase, "resumed", "{result:?}");
        assert_eq!(result.code, None);
        assert_ne!(transport.process_id(), old_pid);
        assert_eq!(
            transport
                .provider_identity()
                .unwrap()
                .provider_thread_id
                .as_deref(),
            Some("same-thread185")
        );
        assert_eq!(std::env::current_dir().unwrap(), target);
        assert_eq!(
            std::fs::read_to_string(target.join("untracked")).unwrap(),
            "conservé aussi\n"
        );
        let replay = apply(
            "task185",
            status,
            Some(&context),
            &mut transport,
            "repair185",
            definition,
            &definition.args,
            &env,
            Vec::new(),
            root,
            "child185",
            "instance185",
            &root.join("bridget.sock"),
        );
        assert_eq!(replay.phase, "resumed", "{replay:?}");
        let rpc = std::fs::read_to_string(&trace).unwrap();
        assert_eq!(
            rpc.lines()
                .filter(|v| v.contains("\"method\":\"thread/start\""))
                .count(),
            1
        );
        assert_eq!(
            rpc.lines()
                .filter(|v| v.contains("\"method\":\"thread/resume\""))
                .count(),
            1
        );
        assert!(!rpc.contains("\"method\":\"turn/start\""));
        transport.stop();
        std::env::set_current_dir(root).unwrap();
    }
}
