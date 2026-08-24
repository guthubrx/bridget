//! Oracles : migration Maicie sous consentement explicite.
//!
//! Une base au schéma N-1 ne migre plus à l'ouverture silencieuse. Le flag /
//! `open_and_migrate` reste le seul consentement. Créer une base neuve n'est
//! pas une migration.

use maicie::store::{MaicieStore, StoreError};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn unique_root(label: &str) -> PathBuf {
    let n = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("maicie-migration-consent-{label}-{n}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let mut perms = fs::metadata(&root).unwrap().permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&root, perms).unwrap();
    root
}

fn user_version(path: &std::path::Path) -> i64 {
    let connection = rusqlite::Connection::open(path).unwrap();
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn base_anterieure_sans_flag_refuse_parlant_sans_ecriture() {
    let root = unique_root("refus");
    let database = root.join("maicie.sqlite3");
    {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), 14);
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.pragma_update(None, "user_version", 13).unwrap();
    drop(connection);
    assert_eq!(user_version(&database), 13);

    let erreur = match MaicieStore::open(&database) {
        Ok(_) => panic!("ouverture sans consentement aurait dû refuser"),
        Err(error) => error,
    };
    let message = format!("{erreur}");
    match &erreur {
        StoreError::MigrationRequired { found, supported } => {
            assert_eq!(*found, 13);
            assert_eq!(*supported, 14);
        }
        other => panic!("attendu MigrationRequired, reçu {other}"),
    }
    assert!(
        message.contains("schéma SQLite 13")
            && message.contains("attend 14")
            && message.contains("maicie migrate --config <chemin>"),
        "message parlant attendu, reçu : {message}"
    );
    assert_eq!(
        user_version(&database),
        13,
        "refus sans flag ne doit rien écrire"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn base_anterieure_avec_consentement_est_migree() {
    let root = unique_root("migrate");
    let database = root.join("maicie.sqlite3");
    {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), 14);
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.pragma_update(None, "user_version", 13).unwrap();
    drop(connection);
    assert_eq!(user_version(&database), 13);

    let store = MaicieStore::open_and_migrate(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 14);
    drop(store);
    assert_eq!(user_version(&database), 14);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn base_neuve_est_cree_sans_flag() {
    let root = unique_root("neuve");
    let database = root.join("maicie.sqlite3");
    assert!(!database.exists());
    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 14);
    assert!(database.is_file());
    drop(store);
    assert_eq!(user_version(&database), 14);
    fs::remove_dir_all(root).unwrap();
}
