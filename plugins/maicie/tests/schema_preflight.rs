//! Oracles du gate d'activation : connaître la compatibilité du greffe sans
//! créer, migrer, réconcilier ni toucher son contenu SQLite.

use maicie::store::{MaicieStore, SCHEMA_VERSION, StoreError};
use rusqlite::Connection;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn unique_root(label: &str) -> PathBuf {
    let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "maicie-schema-preflight-{label}-{}-{serial}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn physical_snapshot(database: &Path) -> Vec<(String, Option<Vec<u8>>)> {
    ["", "-wal", "-shm"]
        .into_iter()
        .map(|suffix| {
            let path = PathBuf::from(format!("{}{suffix}", database.display()));
            (
                suffix.to_string(),
                if path.exists() {
                    Some(fs::read(path).unwrap())
                } else {
                    None
                },
            )
        })
        .collect()
}

fn assert_physically_unchanged(
    label: &str,
    before: &[(String, Option<Vec<u8>>)],
    after: &[(String, Option<Vec<u8>>)],
) {
    for ((before_suffix, before_bytes), (after_suffix, after_bytes)) in
        before.iter().zip(after.iter())
    {
        assert_eq!(before_suffix, after_suffix);
        if before_bytes == after_bytes {
            continue;
        }
        let first_difference = before_bytes
            .as_deref()
            .unwrap_or_default()
            .iter()
            .zip(after_bytes.as_deref().unwrap_or_default())
            .position(|(before, after)| before != after);
        panic!(
            "{label}: sidecar '{before_suffix}' modifié (avant={:?} octets, après={:?} octets, premier écart={first_difference:?})",
            before_bytes.as_ref().map(Vec::len),
            after_bytes.as_ref().map(Vec::len),
        );
    }
}

fn write_config(root: &Path, database: &Path) -> PathBuf {
    let config = root.join("config.json");
    let payload = serde_json::json!({
        "version": 1,
        "bridget_socket": root.join("bridget.sock"),
        "database_path": database,
        "durations": {
            "short_secs": 30,
            "normal_secs": 300,
            "long_secs": 3_600
        },
        "profiles": []
    });
    fs::write(&config, serde_json::to_vec(&payload).unwrap()).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    config
}

#[test]
fn preflight_base_absente_autorise_bootstrap_sans_creer_de_fichier() {
    let root = unique_root("absente");
    let database = root.join("maicie.sqlite3");

    let report = MaicieStore::schema_preflight(&database).unwrap();

    assert_eq!(report.database_version, None);
    assert_eq!(report.supported_version, SCHEMA_VERSION);
    assert!(report.bootstrap_required);
    assert!(
        !database.exists(),
        "le préflight ne crée pas la base absente"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preflight_version_exacte_est_une_lecture_physiquement_neutre() {
    let root = unique_root("exacte");
    let database = root.join("maicie.sqlite3");
    drop(MaicieStore::open(&database).unwrap());
    let before = physical_snapshot(&database);

    let report = MaicieStore::schema_preflight(&database).unwrap();

    assert_eq!(report.database_version, Some(SCHEMA_VERSION));
    assert!(!report.bootstrap_required);
    assert_physically_unchanged("version exacte", &before, &physical_snapshot(&database));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preflight_refuse_les_deux_sens_sans_modifier_le_greffe() {
    for (label, version) in [
        ("ancienne", SCHEMA_VERSION - 1),
        ("future", SCHEMA_VERSION + 1),
    ] {
        let root = unique_root(label);
        let database = root.join("maicie.sqlite3");
        drop(MaicieStore::open(&database).unwrap());
        let connection = Connection::open(&database).unwrap();
        connection
            .pragma_update(None, "user_version", version)
            .unwrap();
        drop(connection);
        let before = physical_snapshot(&database);

        let error = MaicieStore::schema_preflight(&database).unwrap_err();

        if version < SCHEMA_VERSION {
            assert!(matches!(
                error,
                StoreError::MigrationRequired {
                    found,
                    supported
                } if found == version && supported == SCHEMA_VERSION
            ));
        } else {
            assert!(matches!(
                error,
                StoreError::UnsupportedSchema {
                    found,
                    supported
                } if found == version && supported == SCHEMA_VERSION
            ));
        }
        assert_physically_unchanged(label, &before, &physical_snapshot(&database));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn commande_preflight_refuse_avant_ecriture_avec_exit_store() {
    let root = unique_root("cli-ancienne");
    let database = root.join("maicie.sqlite3");
    drop(MaicieStore::open(&database).unwrap());
    let connection = Connection::open(&database).unwrap();
    connection
        .pragma_update(None, "user_version", SCHEMA_VERSION - 1)
        .unwrap();
    drop(connection);
    let config = write_config(&root, &database);
    let before = physical_snapshot(&database);

    let output = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args(["preflight", "--config"])
        .arg(&config)
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(6));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "store");
    assert_physically_unchanged("CLI ancienne", &before, &physical_snapshot(&database));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preflight_lit_la_version_commitee_dans_le_wal_sans_toucher_les_sidecars() {
    let root = unique_root("wal");
    let database = root.join("maicie.sqlite3");
    drop(MaicieStore::open(&database).unwrap());
    let writer = Connection::open(&database).unwrap();
    writer
        .execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;")
        .unwrap();
    writer
        .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap();
    assert!(PathBuf::from(format!("{}-wal", database.display())).exists());
    let before = physical_snapshot(&database);

    let error = MaicieStore::schema_preflight(&database).unwrap_err();

    assert!(matches!(
        error,
        StoreError::UnsupportedSchema {
            found,
            supported
        } if found == SCHEMA_VERSION + 1 && supported == SCHEMA_VERSION
    ));
    assert_physically_unchanged("WAL vivant", &before, &physical_snapshot(&database));
    drop(writer);
    fs::remove_dir_all(root).unwrap();
}
