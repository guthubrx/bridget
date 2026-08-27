//! Point d'entrée isolé du rafraîchisseur de politique du greffe.

use bridget_transport::fsutil::write_private_file_atomic;
use bridget_transport::greffe_policy_refresh::{MarkerInventory, refresh_policy};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_INVENTORY_BYTES: u64 = 1024 * 1024;

pub fn run(
    arguments: impl IntoIterator<Item = OsString>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    match execute(arguments.into_iter().collect(), stdout) {
        Ok(()) => 0,
        Err(RunError::Help) => {
            let _ = writeln!(stdout, "{}", usage());
            0
        }
        Err(RunError::Message(message)) => {
            let _ = writeln!(stderr, "bridget-greffe-policy-refresh: {message}");
            2
        }
    }
}

fn execute(arguments: Vec<OsString>, stdout: &mut dyn Write) -> Result<(), RunError> {
    let Some(command) = arguments.first().and_then(|value| value.to_str()) else {
        return Err(RunError::Message(usage().to_string()));
    };
    if command == "--help" || command == "-h" {
        return Err(RunError::Help);
    }
    match command {
        "scan" => execute_scan(&arguments[1..], stdout),
        "refresh" => execute_refresh(&arguments[1..], stdout),
        _ => Err(RunError::Message(format!(
            "commande inconnue {command:?}\n{}",
            usage()
        ))),
    }
}

fn execute_scan(arguments: &[OsString], stdout: &mut dyn Write) -> Result<(), RunError> {
    let mut markers = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index]
            .to_str()
            .ok_or_else(|| RunError::Message("option non UTF-8".to_string()))?;
        match option {
            "--markers" => set_path_once(arguments, &mut index, &mut markers, "--markers")?,
            "--output" => set_path_once(arguments, &mut index, &mut output, "--output")?,
            "--help" | "-h" => return Err(RunError::Help),
            _ => return Err(RunError::Message(format!("option inconnue {option:?}"))),
        }
        index += 1;
    }
    let markers = markers.ok_or_else(|| RunError::Message("--markers est requis".to_string()))?;
    require_absolute(&markers, "--markers")?;
    if let Some(path) = &output {
        require_absolute(path, "--output")?;
    }
    let inventory = crate::mcp_identity::scan_marker_directory(&markers, unix_now()?)
        .map_err(|error| RunError::Message(error.to_string()))?;
    let mut bytes = serde_json::to_vec_pretty(&inventory)
        .map_err(|error| RunError::Message(error.to_string()))?;
    bytes.push(b'\n');
    if let Some(path) = output {
        write_private_file_atomic(&path, &bytes)
            .map_err(|error| RunError::Message(error.to_string()))?;
    } else {
        stdout
            .write_all(&bytes)
            .map_err(|error| RunError::Message(error.to_string()))?;
    }
    Ok(())
}

fn execute_refresh(arguments: &[OsString], stdout: &mut dyn Write) -> Result<(), RunError> {
    let mut policy = None;
    let mut inventories = Vec::new();
    let mut apply = false;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index]
            .to_str()
            .ok_or_else(|| RunError::Message("option non UTF-8".to_string()))?;
        match option {
            "--policy" => set_path_once(arguments, &mut index, &mut policy, "--policy")?,
            "--inventory" => {
                let path = next_path(arguments, &mut index, "--inventory")?;
                inventories.push(path);
            }
            "--apply" => {
                if apply {
                    return Err(RunError::Message("--apply est répété".to_string()));
                }
                apply = true;
            }
            "--help" | "-h" => return Err(RunError::Help),
            _ => return Err(RunError::Message(format!("option inconnue {option:?}"))),
        }
        index += 1;
    }
    let policy = policy.ok_or_else(|| RunError::Message("--policy est requis".to_string()))?;
    require_absolute(&policy, "--policy")?;
    if inventories.is_empty() {
        return Err(RunError::Message(
            "au moins un --inventory est requis".to_string(),
        ));
    }
    let parsed = inventories
        .iter()
        .map(|path| {
            require_absolute(path, "--inventory")?;
            read_inventory(path)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let report = refresh_policy(&policy, &parsed, unix_now()?, apply)
        .map_err(|error| RunError::Message(error.to_string()))?;
    serde_json::to_writer_pretty(&mut *stdout, &report)
        .map_err(|error| RunError::Message(error.to_string()))?;
    writeln!(stdout).map_err(|error| RunError::Message(error.to_string()))?;
    Ok(())
}

fn read_inventory(path: &Path) -> Result<MarkerInventory, RunError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| RunError::Message(format!("inventaire illisible : {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| RunError::Message(format!("inventaire illisible : {error}")))?;
    if !metadata.is_file() || metadata.len() > MAX_INVENTORY_BYTES {
        return Err(RunError::Message(
            "inventaire non régulier ou trop volumineux".to_string(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| RunError::Message(format!("inventaire illisible : {error}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Message(format!("inventaire invalide : {error}")))
}

fn set_path_once(
    arguments: &[OsString],
    index: &mut usize,
    destination: &mut Option<PathBuf>,
    option: &str,
) -> Result<(), RunError> {
    if destination.is_some() {
        return Err(RunError::Message(format!("{option} est répété")));
    }
    *destination = Some(next_path(arguments, index, option)?);
    Ok(())
}

fn next_path(arguments: &[OsString], index: &mut usize, option: &str) -> Result<PathBuf, RunError> {
    *index += 1;
    arguments
        .get(*index)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| RunError::Message(format!("valeur absente après {option}")))
}

fn require_absolute(path: &Path, option: &str) -> Result<(), RunError> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(RunError::Message(format!(
            "{option} exige un chemin absolu"
        )))
    }
}

fn unix_now() -> Result<i64, RunError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RunError::Message("horloge système antérieure à Unix".to_string()))?
        .as_secs();
    i64::try_from(seconds).map_err(|_| RunError::Message("horloge hors plage".to_string()))
}

fn usage() -> &'static str {
    "usage:\n  bridget-greffe-policy-refresh scan --markers CHEMIN_ABSOLU [--output FICHIER_ABSOLU]\n  bridget-greffe-policy-refresh refresh --policy FICHIER_ABSOLU --inventory FICHIER_ABSOLU [--inventory FICHIER_ABSOLU ...] [--apply]"
}

enum RunError {
    Help,
    Message(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invoke(arguments: &[&str]) -> (i32, String, String) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(
            arguments.iter().map(OsString::from),
            &mut stdout,
            &mut stderr,
        );
        (
            code,
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    #[test]
    fn aide_nommee_les_deux_commandes_et_les_chemins_explicites() {
        let (code, stdout, stderr) = invoke(&["--help"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("scan --markers CHEMIN_ABSOLU"));
        assert!(stdout.contains("refresh --policy FICHIER_ABSOLU"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn options_critiques_repetees_ou_relatives_sont_refusees() {
        let (duplicate, _, duplicate_error) =
            invoke(&["scan", "--markers", "/tmp/a", "--markers", "/tmp/b"]);
        assert_eq!(duplicate, 2);
        assert!(duplicate_error.contains("--markers est répété"));

        let (relative, _, relative_error) = invoke(&[
            "refresh",
            "--policy",
            "policy.json",
            "--inventory",
            "/tmp/inventory.json",
        ]);
        assert_eq!(relative, 2);
        assert!(relative_error.contains("--policy exige un chemin absolu"));
    }
}
