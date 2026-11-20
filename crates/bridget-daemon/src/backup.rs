//! Sauvegarde cohérente à chaud et restauration vérifiée en bac à sable de `BRIDGET_HOME`.
//!
//! Utilise `VACUUM INTO` de SQLite (bundled) pour capturer des clichés transactionnels
//! cohérents sans interrompre le daemon, avec empreintes SHA-256 et contrôle `integrity_check`.

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const BACKUP_FORMAT_VERSION: u32 = 1;
const MANIFEST_FILE: &str = "backup-manifest.json";
const SQLITE_FILES: &[&str] = &["bridget.db", "mobile-devices.db"];
const CONFIG_FILES: &[&str] = &["agents.json", "mobile.json", "sentinel-config.json"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub relative_path: String,
    pub kind: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub format_version: u32,
    pub created_at_ms: u64,
    pub bridget_version: String,
    pub entries: Vec<BackupEntry>,
}

fn sha256_file(path: &Path) -> Result<(u64, String), String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("lecture impossible {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok((bytes.len() as u64, format!("{:x}", hasher.finalize())))
}

fn verify_sqlite_integrity(path: &Path) -> Result<(), String> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("ouverture SQLite {}: {e}", path.display()))?;
    let status: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| format!("integrity_check {}: {e}", path.display()))?;
    if status != "ok" {
        return Err(format!(
            "intégrité SQLite invalide pour {}: {status}",
            path.display()
        ));
    }
    Ok(())
}

fn is_safe_relative_name(raw: &str) -> bool {
    let path = Path::new(raw);
    !raw.is_empty()
        && !path.is_absolute()
        && path.components().all(|c| matches!(c, Component::Normal(_)))
}

/// Crée une sauvegarde cohérente à chaud de `state_dir` dans `dest_dir`.
pub fn create_backup(state_dir: &Path, dest_dir: &Path) -> Result<BackupManifest, String> {
    if dest_dir.exists() {
        let mut iter = std::fs::read_dir(dest_dir)
            .map_err(|e| format!("inspection destination {}: {e}", dest_dir.display()))?;
        if iter.next().is_some() {
            return Err(format!(
                "backup_destination_not_empty: {} doit être vide ou inexistant",
                dest_dir.display()
            ));
        }
    } else {
        bridget_transport::fsutil::create_private_dir(dest_dir)
            .map_err(|e| format!("création destination {}: {e}", dest_dir.display()))?;
    }

    let mut entries = Vec::new();
    for db_name in SQLITE_FILES {
        let src = state_dir.join(db_name);
        if !src.is_file() {
            continue;
        }
        let dst = dest_dir.join(db_name);
        let conn = Connection::open_with_flags(&src, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| format!("ouverture source {}: {e}", src.display()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| format!("busy_timeout {}: {e}", src.display()))?;
        let dst_str = dst
            .to_str()
            .ok_or_else(|| format!("chemin non UTF-8: {}", dst.display()))?;
        conn.execute("VACUUM INTO ?1", [dst_str])
            .map_err(|e| format!("VACUUM INTO {}: {e}", dst.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o600));
        }
        verify_sqlite_integrity(&dst)?;
        let (bytes, sha256) = sha256_file(&dst)?;
        entries.push(BackupEntry {
            relative_path: (*db_name).to_string(),
            kind: "sqlite_hot_copy".to_string(),
            bytes,
            sha256,
        });
    }

    for cfg_name in CONFIG_FILES {
        let src = state_dir.join(cfg_name);
        if !src.is_file() {
            continue;
        }
        let content =
            std::fs::read(&src).map_err(|e| format!("lecture config {}: {e}", src.display()))?;
        let dst = dest_dir.join(cfg_name);
        bridget_transport::fsutil::write_private_file_atomic(&dst, &content)
            .map_err(|e| format!("écriture config {}: {e}", dst.display()))?;
        let (bytes, sha256) = sha256_file(&dst)?;
        entries.push(BackupEntry {
            relative_path: (*cfg_name).to_string(),
            kind: "config_file".to_string(),
            bytes,
            sha256,
        });
    }

    let created_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let manifest = BackupManifest {
        format_version: BACKUP_FORMAT_VERSION,
        created_at_ms,
        bridget_version: env!("CARGO_PKG_VERSION").to_string(),
        entries,
    };
    let encoded = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| format!("sérialisation manifeste: {e}"))?;
    bridget_transport::fsutil::write_private_file_atomic(&dest_dir.join(MANIFEST_FILE), &encoded)
        .map_err(|e| format!("écriture manifeste: {e}"))?;
    Ok(manifest)
}

/// Vérifie une sauvegarde en bac à sable temporaire sans jamais modifier `BRIDGET_HOME`.
pub fn verify_restore(src_dir: &Path) -> Result<Value, String> {
    let manifest_path = src_dir.join(MANIFEST_FILE);
    let raw = std::fs::read(&manifest_path)
        .map_err(|e| format!("manifeste introuvable {}: {e}", manifest_path.display()))?;
    let manifest: BackupManifest = serde_json::from_slice(&raw)
        .map_err(|e| format!("manifeste invalide {}: {e}", manifest_path.display()))?;
    if manifest.format_version != BACKUP_FORMAT_VERSION {
        return Err(format!(
            "backup_format_unsupported: version {}",
            manifest.format_version
        ));
    }

    let sandbox =
        std::env::temp_dir().join(format!("bridget-restore-verify-{}", uuid::Uuid::new_v4()));
    bridget_transport::fsutil::create_private_dir(&sandbox)
        .map_err(|e| format!("création bac à sable: {e}"))?;

    let verification = (|| -> Result<Value, String> {
        let mut verified = 0usize;
        for entry in &manifest.entries {
            if !is_safe_relative_name(&entry.relative_path) {
                return Err(format!(
                    "backup_path_invalid: chemin interdit {}",
                    entry.relative_path
                ));
            }
            let file_path = src_dir.join(&entry.relative_path);
            let (bytes, sha256) = sha256_file(&file_path)?;
            if bytes != entry.bytes || sha256 != entry.sha256 {
                return Err(format!(
                    "backup_checksum_mismatch: {} corrompu",
                    entry.relative_path
                ));
            }
            let target = sandbox.join(&entry.relative_path);
            std::fs::copy(&file_path, &target)
                .map_err(|e| format!("copie bac à sable {}: {e}", entry.relative_path))?;
            if entry.kind == "sqlite_hot_copy" {
                verify_sqlite_integrity(&target)?;
                if entry.relative_path == "bridget.db" {
                    crate::store_schema::validate_existing(&target)
                        .map_err(|e| format!("schéma incompatible {}: {e}", entry.relative_path))?;
                }
            }
            verified += 1;
        }
        Ok(json!({
            "status": "ok",
            "mode": "verify_sandbox",
            "format_version": manifest.format_version,
            "bridget_version": manifest.bridget_version,
            "verified_entries": verified,
        }))
    })();

    let _ = std::fs::remove_dir_all(&sandbox);
    verification
}

pub fn run_backup(args: &[String]) {
    let mut dest: Option<PathBuf> = None;
    let mut state: Option<PathBuf> = None;
    let mut as_json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--state" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("bridget backup: --state exige un chemin");
                    crate::cli_table::exit(2);
                };
                state = Some(PathBuf::from(v));
                i += 2;
            }
            "--json" => {
                as_json = true;
                i += 1;
            }
            arg if !arg.starts_with('-') && dest.is_none() => {
                dest = Some(PathBuf::from(arg));
                i += 1;
            }
            other => {
                eprintln!("bridget backup: argument inconnu: {other}");
                crate::cli_table::exit(2);
            }
        }
    }
    let Some(dest_dir) = dest else {
        eprintln!("Usage : bridget backup <dossier-destination> [--state <dossier>] [--json]");
        crate::cli_table::exit(2);
    };
    let state_dir = match state {
        Some(p) => p,
        None => match crate::environment::Namespace::from_environment() {
            Ok(ns) => ns.root,
            Err(e) => {
                eprintln!("bridget backup: namespace indisponible: {e}");
                crate::cli_table::exit(1);
            }
        },
    };
    match create_backup(&state_dir, &dest_dir) {
        Ok(manifest) => {
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": "ok",
                        "destination": dest_dir,
                        "manifest": manifest,
                    }))
                    .unwrap_or_default()
                );
            } else {
                println!(
                    "OK: sauvegarde cohérente créée dans {} ({} fichiers)",
                    dest_dir.display(),
                    manifest.entries.len()
                );
            }
        }
        Err(error) => {
            eprintln!("bridget backup: {error}");
            crate::cli_table::exit(1);
        }
    }
}

pub fn run_restore(args: &[String]) {
    let mut src: Option<PathBuf> = None;
    let mut verify = false;
    let mut as_json = false;
    for arg in args {
        match arg.as_str() {
            "--verify" => verify = true,
            "--json" => as_json = true,
            value if !value.starts_with('-') && src.is_none() => {
                src = Some(PathBuf::from(value));
            }
            other => {
                eprintln!("bridget restore: argument inconnu: {other}");
                crate::cli_table::exit(2);
            }
        }
    }
    let Some(src_dir) = src else {
        eprintln!("Usage : bridget restore --verify <dossier-sauvegarde> [--json]");
        crate::cli_table::exit(2);
    };
    if !verify {
        eprintln!(
            "bridget restore: passer explicitement --verify pour contrôler en bac à sable sans écraser BRIDGET_HOME actif"
        );
        crate::cli_table::exit(2);
    }
    match verify_restore(&src_dir) {
        Ok(report) => {
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                );
            } else {
                println!(
                    "OK: sauvegarde vérifiée en bac à sable ({} entrées valides)",
                    report["verified_entries"].as_u64().unwrap_or(0)
                );
            }
        }
        Err(error) => {
            eprintln!("bridget restore: {error}");
            crate::cli_table::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec306_hot_backup_and_sandbox_restore_verify_roundtrip_and_detect_corruption() {
        let root = std::env::temp_dir().join(format!("bridget-bkp-{}", uuid::Uuid::new_v4()));
        let state = root.join("state");
        let dest = root.join("backup");
        bridget_transport::fsutil::create_private_dir(&state).unwrap();

        let conn = crate::store_schema::open_private_wal(&state.join("bridget.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE idempotency_schema_migrations(version INTEGER PRIMARY KEY);
             INSERT INTO idempotency_schema_migrations(version) VALUES (8);
             CREATE TABLE sample(id INTEGER PRIMARY KEY, body TEXT);
             INSERT INTO sample(id, body) VALUES (1, 'ok');",
        )
        .unwrap();
        bridget_transport::fsutil::write_private_file_atomic(
            &state.join("agents.json"),
            br#"{"agents":{}}"#,
        )
        .unwrap();

        let manifest = create_backup(&state, &dest).unwrap();
        assert_eq!(manifest.entries.len(), 2);

        let report = verify_restore(&dest).unwrap();
        assert_eq!(report["status"], "ok");
        assert_eq!(report["verified_entries"], 2);

        // Corrompre un fichier doit faire échouer la vérification.
        std::fs::write(dest.join("agents.json"), b"corrompu").unwrap();
        let err = verify_restore(&dest).unwrap_err();
        assert!(err.contains("backup_checksum_mismatch"), "{err}");

        drop(conn);
        let _ = std::fs::remove_dir_all(&root);
    }
}
