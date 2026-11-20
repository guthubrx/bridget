//! État public des contrôles d'infrastructure ; aucune donnée de mission métier.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn read_private_bytes(path: &Path, limit: usize) -> Result<Vec<u8>, ()> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| ())?;
    let meta = file.metadata().map_err(|_| ())?;
    if !meta.is_file()
        || meta.uid() != bridget_transport::fsutil::effective_uid()
        || meta.mode() & 0o077 != 0
    {
        return Err(());
    }
    let mut data = Vec::new();
    file.take((limit + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|_| ())?;
    if data.len() > limit {
        return Err(());
    }
    Ok(data)
}

fn is_valid_host(host: &str) -> bool {
    let bytes = host.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
        && !host.contains("..")
}

fn is_valid_user(user: &str) -> bool {
    let bytes = user.as_bytes();
    !bytes.is_empty()
        && (bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

fn is_safe_absolute(raw: &str) -> bool {
    raw.starts_with('/')
        && !raw.contains('\0')
        && !raw.contains('\n')
        && !raw.contains('\r')
        && !raw.split('/').any(|part| part == "..")
}

fn no_links(path: &Path) -> Result<PathBuf, ()> {
    if !path.is_absolute() {
        return Err(());
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Ok(meta) = std::fs::symlink_metadata(&current)
            && meta.file_type().is_symlink()
        {
            return Err(());
        }
    }
    Ok(current)
}

/// Retourne les liens de fédération natifs vérifiés pour un namespace donné.
/// Remplace l'ancien chargement dynamique par `exec(BRIDGET_SENTINEL_PREPARE_SOURCE)`.
pub(crate) fn installed_links(namespace: &Path) -> Vec<BTreeMap<String, String>> {
    use std::os::unix::fs::MetadataExt;
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    let base = home.join(".local/share/bridget-federation");
    let Ok(entries) = std::fs::read_dir(&base) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|entry| entry.path()))
        .collect();
    dirs.sort();
    let expected_uid = bridget_transport::fsutil::effective_uid();
    let mut links = Vec::new();
    for directory in dirs {
        let Ok(meta) = std::fs::symlink_metadata(&directory) else {
            continue;
        };
        if !meta.is_dir() || meta.uid() != expected_uid || meta.mode() & 0o077 != 0 {
            continue;
        }
        let config_path = directory.join("config");
        let receipt_path = directory.join("receipt");
        let (Ok(raw_config), Ok(raw_receipt)) = (
            read_private_bytes(&config_path, 65_536),
            read_private_bytes(&receipt_path, 65_536),
        ) else {
            continue;
        };
        let (Ok(config_str), Ok(receipt_str)) = (
            String::from_utf8(raw_config.clone()),
            String::from_utf8(raw_receipt),
        ) else {
            continue;
        };
        let values: BTreeMap<String, String> = config_str
            .lines()
            .filter_map(|line| {
                line.split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect();
        let receipt: BTreeMap<String, String> = receipt_str
            .lines()
            .filter_map(|line| {
                line.split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect();

        if values.get("version").map(String::as_str) != Some("1")
            || values.get("root").map(String::as_str) != namespace.to_str()
        {
            continue;
        }
        if receipt.get("config").map(String::as_str) != config_path.to_str()
            || receipt.get("config_sha256").map(String::as_str)
                != Some(sha256_hex(&raw_config).as_str())
        {
            continue;
        }
        let mut receipt_ok = true;
        for name in ["runner", "service"] {
            let Some(target_str) = receipt.get(name) else {
                receipt_ok = false;
                break;
            };
            let Some(expected_hash) = receipt.get(&format!("{name}_sha256")) else {
                receipt_ok = false;
                break;
            };
            let Ok(target) = no_links(Path::new(target_str)) else {
                receipt_ok = false;
                break;
            };
            let Ok(meta) = std::fs::metadata(&target) else {
                receipt_ok = false;
                break;
            };
            let Ok(bytes) = std::fs::read(&target) else {
                receipt_ok = false;
                break;
            };
            if !meta.is_file() || meta.uid() != expected_uid || sha256_hex(&bytes) != *expected_hash
            {
                receipt_ok = false;
                break;
            }
        }
        if !receipt_ok {
            continue;
        }
        let host_ok = values.get("host").is_some_and(|h| is_valid_host(h));
        let user_ok = values.get("user").is_some_and(|u| is_valid_user(u));
        let port_ok = values
            .get("port")
            .and_then(|p| p.parse::<u16>().ok())
            .is_some_and(|p| p >= 1);
        let paths_ok = ["identity", "known_hosts", "remote_root"]
            .iter()
            .all(|k| values.get(*k).is_some_and(|v| is_safe_absolute(v)));
        if host_ok && user_ok && port_ok && paths_ok {
            links.push(values);
        }
    }
    links
}

pub(crate) fn run_federation_links(args: &[String]) -> Result<Value, String> {
    let mut state: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--state" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "federation links: --state exige un chemin".to_string())?;
                state = Some(PathBuf::from(value));
                i += 2;
            }
            "--json" => {
                i += 1;
            }
            other => {
                return Err(format!("federation links: argument inconnu: {other}"));
            }
        }
    }
    let namespace = match state {
        Some(path) => path,
        None => {
            crate::environment::Namespace::from_environment()
                .map_err(|e| format!("federation links: namespace indisponible: {e}"))?
                .root
        }
    };
    Ok(json!(installed_links(&namespace)))
}

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

    #[test]
    fn spec306_run_federation_links_returns_json_array_for_empty_state() {
        let root = std::env::temp_dir().join(format!("fedlinks-{}", uuid::Uuid::new_v4()));
        bridget_transport::fsutil::create_private_dir(&root).unwrap();
        let out = run_federation_links(&[
            "--state".to_string(),
            root.to_string_lossy().to_string(),
            "--json".to_string(),
        ])
        .unwrap();
        assert!(out.is_array());
        std::fs::remove_dir_all(root).unwrap();
    }
}
