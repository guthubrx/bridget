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
        | "usage_limit"
        | "provider_quota_exhausted"
        | "referent_paused"
        | "delegations_halted"
        | "routing_candidate_unavailable"
        | "routing_connection_changed"
        | "permission_attestation_unavailable"
        | "origin_proof_rejected"
        | "remote_start_failed"
        | "workspace_requires_development"
        | "permission_not_inherited"
        | "project_mapping_unknown"
        | "project_mapping_mismatch"
        | "project_preparation_link_unavailable"
        | "project_preparation_git_required"
        | "cwd_outside_parent_project"
        | "expected_base_commit_mismatch"
        | "program_check_unavailable"
        | "sentinel_command_surviving"
        | "sentinel_communication_unconfirmed"
        | "sentinel_catalogue_empty"
        | "sentinel_controller_stop_unconfirmed"
        | "sentinel_contract_changed"
        | "sentinel_execution_unconfirmed"
        | "sentinel_runner_failed" => json!(code),
        _ => json!("unobserved"),
    }
}

/// Dernier refus observé, sans admission ni consommation de clé. Ce cache
/// n'autorise aucun lancement et ne remplace jamais le store des tâches.
pub(crate) fn record_admission_observation(
    db: &rusqlite::Connection,
    owner: &str,
    request: &str,
    admission: &str,
    code: &str,
    now: i64,
) -> rusqlite::Result<()> {
    if !matches!(admission, "not_recorded" | "refused") {
        return Ok(());
    }
    let code = public_code(&json!(code));
    db.execute_batch("CREATE TABLE IF NOT EXISTS admission_observations244(owner TEXT NOT NULL,request_id TEXT NOT NULL,admission TEXT NOT NULL,code TEXT NOT NULL,observed_at INTEGER NOT NULL,PRIMARY KEY(owner,request_id))")?;
    db.execute("INSERT INTO admission_observations244 VALUES(?1,?2,?3,?4,?5) ON CONFLICT(owner,request_id) DO UPDATE SET admission=excluded.admission,code=excluded.code,observed_at=excluded.observed_at",
        rusqlite::params![owner,request,admission,code.as_str().unwrap_or("unobserved"),now])?;
    db.execute(
        "DELETE FROM admission_observations244 WHERE observed_at < ?1",
        [now.saturating_sub(7 * 86400)],
    )?;
    Ok(())
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
        if !meta.is_file()
            || meta.uid() != bridget_transport::fsutil::effective_uid()
            || meta.mode() & 0o077 != 0
        {
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
    let alert = &report["maintenance_alert"];
    let alert_state = alert["state"].as_str().filter(|s| {
        matches!(
            *s,
            "pending" | "submitted" | "unknown" | "confirmed" | "terminal" | "resolved"
        )
    });
    let maintenance_alert=alert_state.map(|state|json!({"state":state,
        "observed_at":alert["observed_at"].as_i64(),"last_checked_at":alert["last_checked_at"].as_i64(),
        "next_check_at":alert["next_check_at"].as_i64(),
        "next_action":if state=="unknown" || state=="submitted" {"Bridget relit le sort du même envoi au prochain contrôle, sans nouvelle notification."}
            else if state=="confirmed" {"Le mainteneur a reçu le signalement ; les autres travaux autorisés peuvent continuer."}
            else {"Le signalement est conservé pour diagnostic de maintenance ; aucun remplacement de mission n’est créé."}}));
    json!({"version":1,"status":report["status"],"started_at":report["started_at"],
        "finished_at":report["finished_at"],"scope":report["scope"],"code":public_code(&report["code"]),"cases":cases,"maintenance_alert":maintenance_alert})
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
    fn spec244_refus_avant_admission_est_observe_sans_creer_de_mission() {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        record_admission_observation(
            &db,
            "parent",
            "requete",
            "not_recorded",
            "project_mapping_unknown",
            100,
        )
        .unwrap();
        let row:(String,String)=db.query_row("SELECT admission,code FROM admission_observations244 WHERE owner='parent' AND request_id='requete'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(
            row,
            ("not_recorded".into(), "project_mapping_unknown".into())
        );
        record_admission_observation(
            &db,
            "parent",
            "hors-git",
            "not_recorded",
            "project_preparation_git_required",
            100,
        )
        .unwrap();
        assert_eq!(
            db.query_row(
                "SELECT code FROM admission_observations244 WHERE request_id='hors-git'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "project_preparation_git_required"
        );
        record_admission_observation(
            &db,
            "parent",
            "requete",
            "pending",
            "provider_quota_exhausted",
            101,
        )
        .unwrap();
        assert_eq!(
            db.query_row(
                "SELECT observed_at FROM admission_observations244",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            100
        );
        record_admission_observation(
            &db,
            "autre",
            "requete",
            "refused",
            "secret non autorisé",
            102,
        )
        .unwrap();
        assert_eq!(
            db.query_row(
                "SELECT code FROM admission_observations244 WHERE owner='autre'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "unobserved"
        );
        assert_eq!(
            db.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            1,
            "aucune tâche ni réservation créée"
        );
    }

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
