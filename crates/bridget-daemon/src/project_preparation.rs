//! Préparation du projet avant l'admission distante, indépendante de T3.
//! Le helper réutilise exclusivement la fédération SSH installée et vérifiée.
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SCRIPT: &str = include_str!("../../../scripts/prepare-project.py");

fn io_code(error: std::io::Error, fallback: &'static str) -> String {
    match error.raw_os_error() {
        Some(libc::ENOSPC) => "project_preparation_disk_full",
        Some(libc::EDQUOT) => "project_preparation_disk_quota",
        _ => fallback,
    }
    .into()
}

/// Filtre bon marché ; le helper vérifie ensuite permissions et reçus complets.
pub(crate) fn has_link(namespace: &Path) -> bool {
    let Some(home) = std::env::var_os("HOME") else {
        return false;
    };
    let root = Path::new(&home).join(".local/share/bridget-federation");
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    let expected = format!("root={}", namespace.display());
    entries.flatten().any(|entry| {
        fs::read_to_string(entry.path().join("config"))
            .is_ok_and(|text| text.lines().any(|line| line == expected))
    })
}

pub(crate) fn prepare_until(request: &Value, deadline: Instant) -> Result<(), String> {
    prepare_receipt_until(request, deadline).map(|_| ())
}

pub(crate) fn prepare_receipt_until(request: &Value, deadline: Instant) -> Result<Value, String> {
    prepare_script(request, SCRIPT, deadline)
}

fn prepare_script(
    request: &Value,
    script_content: &str,
    deadline: Instant,
) -> Result<Value, String> {
    if Instant::now() >= deadline {
        return Err("project_preparation_timeout".into());
    }
    let base = fs::canonicalize(std::env::temp_dir())
        .map_err(|error| io_code(error, "project_preparation_temp_unavailable"))?;
    let directory = base.join(format!("bridget-prepare-{}", uuid::Uuid::new_v4()));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory)
        .map_err(|error| io_code(error, "project_preparation_temp_unavailable"))?;
    let result = (|| {
        let script = directory.join("prepare.py");
        let input = directory.join("request.json");
        let output = directory.join("response.json");
        use std::io::Write;
        for (path, bytes) in [
            (&script, script_content.as_bytes().to_vec()),
            (
                &input,
                serde_json::to_vec(request).map_err(|_| "project_preparation_invalid")?,
            ),
        ] {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .map_err(|error| io_code(error, "project_preparation_temp_unavailable"))?;
            file.write_all(&bytes)
                .map_err(|error| io_code(error, "project_preparation_temp_unavailable"))?;
        }
        let out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&output)
            .map_err(|error| io_code(error, "project_preparation_temp_unavailable"))?;
        let mut child = Command::new("python3")
            .arg(&script)
            .stdin(
                fs::File::open(&input)
                    .map_err(|error| io_code(error, "project_preparation_temp_unavailable"))?,
            )
            .stdout(out)
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|error| io_code(error, "project_preparation_python_unavailable"))?;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20))
                }
                _ => {
                    // Le groupe est créé et détenu ici ; aucun PID externe ni SIGKILL.
                    let stopped = bridget_transport::managed_session::stop_owned_child(
                        &mut child,
                        true,
                        Duration::from_secs(2),
                    );
                    return Err(if stopped.is_ok() {
                        "project_preparation_timeout"
                    } else {
                        "project_preparation_stop_unconfirmed"
                    }
                    .into());
                }
            }
        };
        if !status.success() {
            return Err("project_preparation_failed".into());
        }
        let mut data = Vec::new();
        fs::File::open(&output)
            .map_err(|error| io_code(error, "project_preparation_failed"))?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut data)
            .map_err(|error| io_code(error, "project_preparation_failed"))?;
        if data.len() > 1024 * 1024 {
            return Err("project_preparation_output_limit".into());
        }
        let answer: Value =
            serde_json::from_slice(&data).map_err(|_| "project_preparation_failed")?;
        if answer["ok"] == true {
            return Ok(answer["result"].clone());
        }
        let code = answer["error"]
            .as_str()
            .unwrap_or("project_preparation_failed");
        if code.len() > 128 || !code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            return Err("project_preparation_failed".into());
        }
        if code == "project_preparation_process_failed"
            && let Some(detail) = answer.get("detail").and_then(Value::as_object)
            && detail.len() == 2
            && matches!(
                detail.get("operation").and_then(Value::as_str),
                Some("inspect" | "prepare")
            )
            && detail
                .get("exit_code")
                .and_then(Value::as_i64)
                .is_some_and(|n| n != 0 && (-255..=255).contains(&n))
        {
            let correlation = request
                .get("correlation_id")
                .and_then(Value::as_str)
                .filter(|value| uuid::Uuid::parse_str(value).is_ok());
            eprintln!(
                "bridget_project_preparation {}",
                serde_json::json!({
                    "version":1,"scope":"this_request","code":code,
                    "correlation_id":correlation,"operation":detail["operation"],
                    "exit_code":detail["exit_code"],"cause":"unobserved"
                })
            );
        }
        Err(code.to_owned())
    })();
    // Un enfant survivant garde sa fixture privée ; aucun dépôt n'est supprimé.
    if result
        .as_ref()
        .is_err_and(|code| code == "project_preparation_stop_unconfirmed")
    {
        return result;
    }
    for name in ["prepare.py", "request.json", "response.json"] {
        let _ = fs::remove_file(directory.join(name));
    }
    let _ = fs::remove_dir(directory);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec271_transport_diagnostic_preserves_legacy_wire_shape() {
        use bridget_transport::protocol::{OriginProofResponse, OriginProofResult, WorkerIdentity};
        let script = "print('{\"ok\":false,\"error\":\"project_preparation_process_failed\",\"detail\":{\"operation\":\"inspect\",\"exit_code\":255}}')";
        let request = serde_json::json!({"correlation_id":"27100000-0000-4000-8000-000000000001"});
        let error =
            prepare_script(&request, script, Instant::now() + Duration::from_secs(5)).unwrap_err();
        assert_eq!(error, "project_preparation_process_failed");
        let response = OriginProofResponse {
            version: 1,
            correlation_id: "correlation271".into(),
            connection_epoch: "epoch271".into(),
            worker: WorkerIdentity {
                machine_id: "fixture271".into(),
                worker_id: "worker271".into(),
                provisioning_revision: 1,
            },
            response: OriginProofResult::ProjectPrepared {
                permission_digest: "a".repeat(64),
                preparation_error: Some(error),
                project: None,
            },
        };
        assert_eq!(response.validate_shape(), Ok(()));
        let mut incompatible = response;
        if let OriginProofResult::ProjectPrepared {
            preparation_error, ..
        } = &mut incompatible.response
        {
            *preparation_error = Some(
                "project_preparation_transport_failed:{\"operation\":\"inspect\",\"exit_code\":255}"
                    .into(),
            );
        }
        assert!(
            incompatible.validate_shape().is_err(),
            "ne pas élargir le protocole pour faire passer le diagnostic"
        );
    }

    #[test]
    fn spec229_preparation_context_crosses_wire_and_reaches_coordinator_registry() {
        use bridget_transport::protocol::*;
        use serde_json::json;
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("bridget229-{}", uuid::Uuid::new_v4()));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let path = root.join("agents.json");
        let origin = root.join("source");
        let target = WorkerIdentity {
            machine_id: "target229".into(),
            worker_id: "worker229".into(),
            provisioning_revision: 1,
        };
        let authority = json!({"worker_id":target.worker_id,"provisioning_revision":1,"credential_fingerprint":"a".repeat(64)});
        let mut machine = authority.clone();
        machine["enabled"] = json!(true);
        machine["mappings"] = json!({});
        let config = json!({"machines":{"target229":machine}});
        bridget_transport::fsutil::write_private_file_atomic(
            &path,
            &serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        assert!(
            crate::registry::AgentRegistry::load_from_path_quiet(path.clone())
                .unwrap()
                .select_project_mapping("target229", None, "origin229", &origin)
                .is_err()
        );
        let mapping = json!({"source_host":"origin229","source_root":origin,
            "remote_root":"/tmp/target229/repository","expected_git_identity":"/tmp/target229/mirror.git","mapping_revision":1});
        let reply = OriginProofResponse {
            version: 1,
            correlation_id: "proof229".into(),
            connection_epoch: "epoch229".into(),
            worker: WorkerIdentity {
                machine_id: "origin229".into(),
                worker_id: "origin-worker229".into(),
                provisioning_revision: 1,
            },
            response: OriginProofResult::ProjectPrepared {
                permission_digest: "b".repeat(64),
                preparation_error: None,
                project: Some(PreparedOriginProject {
                    target,
                    credential_fingerprint: "a".repeat(64),
                    project_id: "project229".into(),
                    mapping,
                }),
            },
        };
        let wire = serde_json::to_vec(&reply).unwrap();
        let decoded: OriginProofResponse = serde_json::from_slice(&wire).unwrap();
        decoded.validate_shape().unwrap();
        let OriginProofResult::ProjectPrepared {
            project: Some(project),
            ..
        } = decoded.response
        else {
            panic!("context lost")
        };
        let returned=prepare_receipt_until(&json!({"operation":"publish_prepared","registry":path,
            "machine_id":project.target.machine_id,"authority":authority,"origin_host":"origin229",
            "source_root":origin,"cwd":origin,"project_id":project.project_id,"mapping":project.mapping}),
            Instant::now()+Duration::from_secs(3)).unwrap();
        assert_eq!(returned["project_id"], "project229");
        let registry = crate::registry::AgentRegistry::load_from_path_quiet(path.clone()).unwrap();
        assert_eq!(
            registry
                .select_project_mapping("target229", None, "origin229", &origin)
                .unwrap()
                .0,
            "project229"
        );
        let old: OriginProofResult = serde_json::from_value(json!({"operation":"project_prepared",
            "permission_digest":"b".repeat(64),"preparation_error":null}))
        .unwrap();
        assert!(matches!(
            old,
            OriginProofResult::ProjectPrepared { project: None, .. }
        ));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn spec191_storage_errno_and_helper_result_stay_closed() {
        for (errno, expected) in [
            (libc::ENOSPC, "project_preparation_disk_full"),
            (libc::EDQUOT, "project_preparation_disk_quota"),
            (libc::EACCES, "project_preparation_failed"),
        ] {
            assert_eq!(
                io_code(
                    std::io::Error::from_raw_os_error(errno),
                    "project_preparation_failed"
                ),
                expected
            );
        }
        assert_eq!(
            io_code(
                std::io::Error::other("No space left on device /private/secret"),
                "project_preparation_failed"
            ),
            "project_preparation_failed"
        );
        let result = prepare_script(
            &serde_json::json!({}),
            "import json\nprint(json.dumps({'ok':False,'error':'project_preparation_disk_full'}))\n",
            Instant::now() + Duration::from_secs(3),
        );
        assert_eq!(result, Err("project_preparation_disk_full".into()));
    }
    #[test]
    fn mobile177_origin_preparation_helper_is_bounded_and_keeps_closed_errors() {
        let started = Instant::now();
        let result = prepare_script(
            &serde_json::json!({}),
            "import time\ntime.sleep(60)\n",
            started + Duration::from_millis(100),
        );
        assert_eq!(result, Err("project_preparation_timeout".into()));
        assert!(started.elapsed() < Duration::from_secs(3));
        let result = prepare_script(
            &serde_json::json!({}),
            "import json\nprint(json.dumps({'ok':False,'error':'project_preparation_busy'}))\n",
            Instant::now() + Duration::from_secs(3),
        );
        assert_eq!(result, Err("project_preparation_busy".into()));
        let result = prepare_script(
            &serde_json::json!({}),
            "print('{\"ok\":false,\"error\":\"private stderr path\"}')\n",
            Instant::now() + Duration::from_secs(3),
        );
        assert_eq!(result, Err("project_preparation_failed".into()));
    }
}
