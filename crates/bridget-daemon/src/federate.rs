//! Façade minimale vers le gestionnaire de fédération embarqué.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SCRIPT: &str = include_str!("../../../scripts/federate-ssh.sh");

fn cleanup(directory: &Path, script: &Path) -> Result<(), String> {
    if script.exists() {
        fs::remove_file(script)
            .map_err(|error| format!("suppression {} : {error}", script.display()))?;
    }
    fs::remove_dir(directory)
        .map_err(|error| format!("suppression {} : {error}", directory.display()))
}

fn create_script() -> Result<(PathBuf, PathBuf), String> {
    let temp_root = fs::canonicalize(std::env::temp_dir())
        .map_err(|error| format!("répertoire temporaire indisponible : {error}"))?;
    let directory = temp_root.join(format!(
        "bridget-federate-{}",
        uuid::Uuid::new_v4().simple()
    ));
    DirBuilder::new()
        .mode(0o700)
        .create(&directory)
        .map_err(|error| format!("création {} : {error}", directory.display()))?;

    let metadata = fs::symlink_metadata(&directory)
        .map_err(|error| format!("inspection {} : {error}", directory.display()))?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o700
        || fs::canonicalize(&directory).ok().as_deref() != Some(directory.as_path())
    {
        let _ = fs::remove_dir(&directory);
        return Err("répertoire temporaire privé canonique 0700 requis".into());
    }

    let script = directory.join("federate-ssh.sh");
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o700)
        .open(&script)
    {
        Ok(file) => file,
        Err(error) => {
            let _ = fs::remove_dir(&directory);
            return Err(format!("création {} : {error}", script.display()));
        }
    };
    if let Err(error) = file
        .set_permissions(fs::Permissions::from_mode(0o700))
        .and_then(|()| file.write_all(SCRIPT.as_bytes()))
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = cleanup(&directory, &script);
        return Err(format!("écriture {} : {error}", script.display()));
    }
    drop(file);
    Ok((directory, script))
}

fn invoke(arguments: &[String]) -> Result<i32, String> {
    let (directory, script) = create_script()?;
    let result = Command::new("/bin/bash")
        .arg(&script)
        .arg("cli")
        .args(arguments)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    let cleanup_result = cleanup(&directory, &script);
    let status = result.map_err(|error| format!("/bin/bash indisponible : {error}"))?;
    cleanup_result?;
    Ok(status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1)))
}

pub fn run(arguments: &[String]) -> ! {
    match invoke(arguments) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("bridget federate: {error}");
            std::process::exit(1);
        }
    }
}
