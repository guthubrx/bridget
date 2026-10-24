//! Préparation du projet avant l'admission distante, indépendante de T3.
//! Le helper réutilise exclusivement la fédération SSH installée et vérifiée.
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::Path;
use std::os::unix::process::CommandExt;
use std::time::{Duration, Instant};
use std::process::{Command, Stdio};
use serde_json::Value;

const SCRIPT: &str = include_str!("../../../scripts/prepare-project.py");

/// Filtre bon marché ; le helper vérifie ensuite permissions et reçus complets.
pub(crate) fn has_link(namespace: &Path) -> bool {
    let Some(home) = std::env::var_os("HOME") else { return false; };
    let root = Path::new(&home).join(".local/share/bridget-federation");
    let Ok(entries) = fs::read_dir(root) else { return false; };
    let expected = format!("root={}", namespace.display());
    entries.flatten().any(|entry| fs::read_to_string(entry.path().join("config"))
        .is_ok_and(|text| text.lines().any(|line| line == expected)))
}

pub(crate) fn prepare(request: &Value) -> Result<(), String> {
    prepare_until(request, Instant::now() + Duration::from_secs(600))
}

pub(crate) fn prepare_until(request: &Value, deadline: Instant) -> Result<(), String> {
    prepare_script(request, SCRIPT, deadline)
}

fn prepare_script(request: &Value, script_content: &str, deadline: Instant) -> Result<(), String> {
    if Instant::now() >= deadline { return Err("project_preparation_timeout".into()); }
    let base = fs::canonicalize(std::env::temp_dir())
        .map_err(|_| "project_preparation_temp_unavailable")?;
    let directory = base.join(format!("bridget-prepare-{}", uuid::Uuid::new_v4()));
    fs::DirBuilder::new().mode(0o700).create(&directory)
        .map_err(|_| "project_preparation_temp_unavailable")?;
    let result = (|| {
        let script = directory.join("prepare.py");
        let input = directory.join("request.json");
        let output = directory.join("response.json");
        use std::io::Write;
        for (path, bytes) in [(&script, script_content.as_bytes().to_vec()),
            (&input, serde_json::to_vec(request).map_err(|_| "project_preparation_invalid")?)] {
            let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600)
                .open(path).map_err(|_| "project_preparation_temp_unavailable")?;
            file.write_all(&bytes).map_err(|_| "project_preparation_temp_unavailable")?;
        }
        let out = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(&output).map_err(|_| "project_preparation_temp_unavailable")?;
        let mut child = Command::new("python3").arg(&script)
            .stdin(fs::File::open(&input).map_err(|_| "project_preparation_temp_unavailable")?)
            .stdout(out).stderr(Stdio::null()).process_group(0).spawn()
            .map_err(|_| "project_preparation_python_unavailable")?;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                _ => {
                    // Le groupe est créé et détenu ici ; aucun PID externe ni SIGKILL.
                    let stopped = bridget_transport::managed_session::stop_owned_child(
                        &mut child, true, Duration::from_secs(2));
                    return Err(if stopped.is_ok() { "project_preparation_timeout" }
                        else { "project_preparation_stop_unconfirmed" }.into());
                }
            }
        };
        if !status.success() { return Err("project_preparation_failed".into()); }
        let mut data = Vec::new();
        fs::File::open(&output).map_err(|_| "project_preparation_failed")?
            .take(1024 * 1024 + 1).read_to_end(&mut data)
            .map_err(|_| "project_preparation_failed")?;
        if data.len() > 1024 * 1024 { return Err("project_preparation_output_limit".into()); }
        let answer: Value = serde_json::from_slice(&data).map_err(|_| "project_preparation_failed")?;
        if answer["ok"] == true { return Ok(()); }
        let code = answer["error"].as_str().unwrap_or("project_preparation_failed");
        if code.len() > 128 || !code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            return Err("project_preparation_failed".into());
        }
        Err(code.to_owned())
    })();
    // Un enfant survivant garde sa fixture privée ; aucun dépôt n'est supprimé.
    if result.as_ref().is_err_and(|code| code == "project_preparation_stop_unconfirmed") {
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
    fn mobile177_origin_preparation_helper_is_bounded_and_keeps_closed_errors() {
        let started = Instant::now();
        let result = prepare_script(&serde_json::json!({}), "import time\ntime.sleep(60)\n",
            started + Duration::from_millis(100));
        assert_eq!(result, Err("project_preparation_timeout".into()));
        assert!(started.elapsed() < Duration::from_secs(3));
        let result = prepare_script(&serde_json::json!({}),
            "import json\nprint(json.dumps({'ok':False,'error':'project_preparation_busy'}))\n",
            Instant::now() + Duration::from_secs(3));
        assert_eq!(result, Err("project_preparation_busy".into()));
        let result = prepare_script(&serde_json::json!({}),
            "print('{\"ok\":false,\"error\":\"private stderr path\"}')\n",
            Instant::now() + Duration::from_secs(3));
        assert_eq!(result, Err("project_preparation_failed".into()));
    }
}
