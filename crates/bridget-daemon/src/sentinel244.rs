//! État public des contrôles d'infrastructure ; aucune donnée de mission métier.
use serde_json::{Value, json};
use std::io::Read;
use std::path::Path;

fn public_code(value: &Value) -> Value {
    let Some(raw) = value.as_str() else {
        return Value::Null;
    };
    let code = raw.split(':').next().unwrap_or("");
    if serde_json::from_value::<bridget_transport::protocol::WorkerRefusal>(json!(code)).is_ok() {
        return json!(code);
    }
    match code {
        "resource_saturated"
        | "provider_quota_exhausted"
        | "referent_paused"
        | "delegations_halted"
        | "routing_candidate_unavailable"
        | "routing_connection_changed"
        | "permission_attestation_unavailable"
        | "origin_proof_rejected"
        | "remote_start_failed"
        | "project_mapping_unknown"
        | "project_mapping_mismatch"
        | "cwd_outside_parent_project"
        | "expected_base_commit_mismatch"
        | "program_check_unavailable"
        | "sentinel_communication_unconfirmed"
        | "sentinel_catalogue_empty"
        | "sentinel_controller_stop_unconfirmed"
        | "sentinel_contract_changed"
        | "sentinel_execution_unconfirmed"
        | "sentinel_runner_failed" => json!(code),
        _ => json!("unobserved"),
    }
}

pub(crate) fn read_remote(socket: &Path) -> Result<Value, String> {
    let mut connection = crate::communication::client::DaemonConnection::connect(socket)
        .map_err(|e| e.to_string())?;
    match connection.exchange(&bridget_transport::WrapperToDaemon::DaemonIdentityRequest).map_err(|e|e.to_string())? {
        bridget_transport::DaemonToWrapper::DaemonIdentityReport{features,..} if features.iter().any(|f|f=="sentinel_status_v1")=>{},
        _=>return Err("sentinel_not_active: charger la sentinelle sur le maître Bridget ; aucun redémarrage du client nécessaire".into()),
    }
    match connection
        .exchange(&bridget_transport::WrapperToDaemon::SentinelStatusRead)
        .map_err(|e| e.to_string())?
    {
        bridget_transport::DaemonToWrapper::SentinelStatus { report } => Ok(report),
        _ => Err("sentinel_status_unavailable".into()),
    }
}

pub(crate) fn status(state: &Path) -> Value {
    let path = state.join("sentinel.json");
    let read = (|| -> Result<Value, ()> {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)
            .map_err(|_| ())?;
        let meta = file.metadata().map_err(|_| ())?;
        if !meta.is_file() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
            return Err(());
        }
        let mut data = Vec::new();
        file.take(1024 * 1024 + 1)
            .read_to_end(&mut data)
            .map_err(|_| ())?;
        if data.len() > 1024 * 1024 {
            return Err(());
        }
        serde_json::from_slice(&data).map_err(|_| ())
    })();
    let report = match read {
        Ok(v) => v,
        Err(_) => {
            return json!({"version":1,"status":if path.exists(){"unavailable"}else{"not_observed"}});
        }
    };
    let cases =
        report["cases"]
            .as_array()
            .map(|rows| {
                rows.iter().map(|row|{
        json!({"machine":row["machine"],"engine":row["engine"],"fixture":row["fixture"],
            "strategy":row["strategy"],"status":row["status"],"code":public_code(&row["code"])})
    }).collect::<Vec<_>>()
            })
            .unwrap_or_default();
    json!({"version":1,"status":report["status"],"started_at":report["started_at"],
        "finished_at":report["finished_at"],"scope":report["scope"],"code":public_code(&report["code"]),"cases":cases})
}

pub(crate) fn run(binary: &Path, state: &Path, reason: &str) -> Result<i32, String> {
    if !matches!(reason, "manual" | "activation" | "tick") {
        return Err("sentinel_reason_invalid".into());
    }
    let snapshot = crate::host_resources::HostResourceCollector::default().collect(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        None,
    );
    std::process::Command::new("python3")
        .arg("-c")
        .arg(include_str!("../../../scripts/sentinel.py"))
        .env(
            "BRIDGET_SENTINEL_RESOURCE_SNAPSHOT",
            serde_json::to_string(&snapshot).map_err(|e| e.to_string())?,
        )
        .env(
            "BRIDGET_SENTINEL_PREPARE_SOURCE",
            include_str!("../../../scripts/prepare-project.py"),
        )
        .arg("--binary")
        .arg(binary)
        .arg("--state")
        .arg(state)
        .arg("--reason")
        .arg(reason)
        .status()
        .map(|s| s.code().unwrap_or(1))
        .map_err(|e| format!("sentinel_runner_unavailable: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec244_statut_sentinelle_ne_publie_aucun_contenu_metier() {
        let root = std::env::temp_dir().join(format!("sentinel244-{}", uuid::Uuid::new_v4()));
        bridget_transport::fsutil::create_private_dir(&root).unwrap();
        assert_eq!(status(&root)["status"], "not_observed");
        bridget_transport::fsutil::write_private_file_atomic(&root.join("sentinel.json"),br#"{"version":1,"status":"failed","secret":"private","cases":[{"machine":"sys","engine":"codex","fixture":"new","strategy":"commit_view","status":"failed","code":"worker_unavailable","task_id":"private","result":"private"}]}"#).unwrap();
        let view = status(&root).to_string();
        assert!(!view.contains("private"));
        assert!(view.contains("worker_unavailable"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
