//! Diagnostic unifié `bridget doctor` et exposition des métriques d'exploitation.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn check_entry(name: &str, status: &str, detail: Value, next_action: Option<&str>) -> Value {
    json!({
        "name": name,
        "status": status,
        "detail": detail,
        "next_action": next_action,
    })
}

/// Construit le rapport de diagnostic complet pour un namespace donné.
pub fn collect_doctor_report(state_dir: &Path, socket_path: &Path) -> Value {
    use std::os::unix::fs::MetadataExt;
    let mut checks = Vec::new();
    let mut overall = "ok";
    let mut escalate = |lvl: &'static str| {
        if lvl == "error" || (lvl == "degraded" && overall == "ok") {
            overall = lvl;
        }
    };

    // 1. Permissions du dossier privé BRIDGET_HOME
    match std::fs::symlink_metadata(state_dir) {
        Ok(meta)
            if meta.is_dir()
                && meta.uid() == bridget_transport::fsutil::effective_uid()
                && meta.mode() & 0o077 == 0 =>
        {
            checks.push(check_entry(
                "namespace_permissions",
                "ok",
                json!({"path": state_dir, "mode": format!("{:o}", meta.mode() & 0o777)}),
                None,
            ));
        }
        Ok(meta) => {
            escalate("error");
            checks.push(check_entry(
                "namespace_permissions",
                "error",
                json!({"path": state_dir, "mode": format!("{:o}", meta.mode() & 0o777)}),
                Some("Corriger les droits en 0700 sur le dossier BRIDGET_HOME."),
            ));
        }
        Err(err) => {
            escalate("degraded");
            checks.push(check_entry(
                "namespace_permissions",
                "degraded",
                json!({"path": state_dir, "error": err.to_string()}),
                Some("Initialiser le namespace ou vérifier BRIDGET_HOME."),
            ));
        }
    }

    // 2. Socket daemon
    let socket_live = std::os::unix::net::UnixStream::connect(socket_path).is_ok();
    if socket_live {
        checks.push(check_entry(
            "daemon_socket",
            "ok",
            json!({"path": socket_path, "reachable": true}),
            None,
        ));
    } else {
        escalate("degraded");
        checks.push(check_entry(
            "daemon_socket",
            "degraded",
            json!({"path": socket_path, "reachable": false}),
            Some("Démarrer le service bridget daemon si une session locale est attendue."),
        ));
    }

    // 3. Bases SQLite (mode WAL + schéma lisible)
    for db_name in ["bridget.db", "mobile-devices.db"] {
        let db_path = state_dir.join(db_name);
        if !db_path.is_file() {
            checks.push(check_entry(
                &format!("sqlite:{db_name}"),
                "ok",
                json!({"present": false}),
                None,
            ));
            continue;
        }
        match rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            Ok(conn) => match crate::store_schema::inspect_store_schema(&conn) {
                Ok(info) => {
                    let status = if info.journal_mode.eq_ignore_ascii_case("wal") {
                        "ok"
                    } else {
                        escalate("degraded");
                        "degraded"
                    };
                    checks.push(check_entry(
                        &format!("sqlite:{db_name}"),
                        status,
                        json!({
                            "present": true,
                            "mechanism": info.mechanism,
                            "version": info.version,
                            "journal_mode": info.journal_mode,
                        }),
                        if status == "ok" {
                            None
                        } else {
                            Some("Activer le mode journal WAL à la prochaine ouverture.")
                        },
                    ));
                }
                Err(err) => {
                    escalate("error");
                    checks.push(check_entry(
                        &format!("sqlite:{db_name}"),
                        "error",
                        json!({"error": err.to_string()}),
                        Some("Vérifier l'intégrité SQLite avant toute mise à jour."),
                    ));
                }
            },
            Err(err) => {
                escalate("error");
                checks.push(check_entry(
                    &format!("sqlite:{db_name}"),
                    "error",
                    json!({"error": err.to_string()}),
                    Some("Vérifier les permissions et l'intégrité du fichier SQLite."),
                ));
            }
        }
    }

    // 4. Registre agents.json
    let registry_path = state_dir.join("agents.json");
    match crate::registry::AgentRegistry::load_from_path_quiet(registry_path.clone()) {
        Ok(reg) => {
            checks.push(check_entry(
                "agent_registry",
                "ok",
                json!({"path": registry_path, "agents": reg.len()}),
                None,
            ));
        }
        Err(err) => {
            escalate("error");
            checks.push(check_entry(
                "agent_registry",
                "error",
                json!({"path": registry_path, "error": err}),
                Some("Corriger la syntaxe ou les permissions 0600 d'agents.json."),
            ));
        }
    }

    // 5. Sentinelle & passerelle mobile
    let sentinel = crate::sentinel::status(state_dir);
    let mobile_configured = state_dir.join("mobile.json").is_file();

    json!({
        "version": 1,
        "bridget_version": env!("CARGO_PKG_VERSION"),
        "status": overall,
        "checks": checks,
        "metrics": crate::daemon::get_metrics().snapshot_json(),
        "sentinel": sentinel,
        "mobile": {
            "configured": mobile_configured,
            "devices_db_present": state_dir.join("mobile-devices.db").is_file(),
        }
    })
}

pub fn run(args: &[String]) {
    let mut as_json = false;
    let mut state_override: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                as_json = true;
                i += 1;
            }
            "--state" => {
                let Some(val) = args.get(i + 1) else {
                    eprintln!("bridget doctor: --state exige un chemin");
                    crate::cli_table::exit(2);
                };
                state_override = Some(PathBuf::from(val));
                i += 2;
            }
            other => {
                eprintln!("bridget doctor: argument inconnu: {other}");
                crate::cli_table::exit(2);
            }
        }
    }

    let (state_dir, socket_path) = match state_override {
        Some(dir) => (dir.clone(), dir.join("bridget.sock")),
        None => match crate::environment::Namespace::from_environment() {
            Ok(ns) => (ns.root, ns.socket),
            Err(err) => {
                eprintln!("bridget doctor: namespace indisponible: {err}");
                crate::cli_table::exit(1);
            }
        },
    };

    let report = collect_doctor_report(&state_dir, &socket_path);
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        println!(
            "Bridget doctor ({}) : {}",
            report["bridget_version"].as_str().unwrap_or("?"),
            report["status"].as_str().unwrap_or("unknown")
        );
        if let Some(items) = report["checks"].as_array() {
            for item in items {
                let name = item["name"].as_str().unwrap_or("?");
                let st = item["status"].as_str().unwrap_or("?");
                println!("  [{st}] {name}");
                if let Some(action) = item["next_action"].as_str() {
                    println!("       -> {action}");
                }
            }
        }
    }
    if report["status"] == "error" {
        crate::cli_table::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec306_doctor_reports_private_wal_namespace_and_metrics() {
        let dir = std::env::temp_dir().join(format!("bridget-doc-{}", uuid::Uuid::new_v4()));
        bridget_transport::fsutil::create_private_dir(&dir).unwrap();
        let conn = crate::store_schema::open_private_wal(&dir.join("bridget.db")).unwrap();
        drop(conn);
        let report = collect_doctor_report(&dir, &dir.join("bridget.sock"));
        assert!(report["metrics"]["native_tick_latency_ms"].is_object());
        assert_eq!(report["checks"][0]["status"], "ok");
        assert_eq!(report["checks"][2]["status"], "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
