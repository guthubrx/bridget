//! Oracles : migration Maicie sous consentement explicite.
//!
//! Une base au schéma N-1 ne migre plus à l'ouverture silencieuse. Le flag /
//! `open_and_migrate` reste le seul consentement. Créer une base neuve
//! (user_version 0 ET sqlite_master utilisateur vide) n'est pas une migration.
//! Une base peuplée avec user_version remis à 0 est une porte déguisée : refus.
//!
//! Leçon : un oracle de version qui code la version en dur meurt à chaque
//! migration ; il doit lire `store::SCHEMA_VERSION`.

use maicie::store::{MaicieStore, SCHEMA_VERSION, StoreError};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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

/// Empreinte du contenu schéma : version + sqlite_master + schema_migrations.
/// Un mutant qui muterait les tables en laissant user_version intact doit
/// faire rougir l'oracle — la propriété nommée est « rien d'écrit », pas
/// seulement « uv inchangé ».
#[derive(Debug, Clone, PartialEq, Eq)]
struct SchemaSnapshot {
    user_version: i64,
    master_digest: String,
    migrations: Vec<(i64, i64)>,
}

fn schema_snapshot(path: &Path) -> SchemaSnapshot {
    let connection = rusqlite::Connection::open(path).unwrap();
    let user_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    let mut master_rows = connection
        .prepare(
            "SELECT type, name, tbl_name, IFNULL(sql, '')\n\
             FROM sqlite_master\n\
             WHERE name NOT LIKE 'sqlite_%'\n\
             ORDER BY type, name",
        )
        .unwrap()
        .query_map([], |row| {
            Ok(format!(
                "{}|{}|{}|{}",
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    master_rows.sort();
    let master_digest = master_rows.join("\n");

    let migrations_exist: bool = connection
        .query_row(
            "SELECT EXISTS(\n\
                 SELECT 1 FROM sqlite_master\n\
                 WHERE type = 'table' AND name = 'schema_migrations'\n\
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let migrations = if migrations_exist {
        connection
            .prepare("SELECT version, applied_at FROM schema_migrations ORDER BY version")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    } else {
        Vec::new()
    };

    SchemaSnapshot {
        user_version,
        master_digest,
        migrations,
    }
}

fn assert_refusal_leaves_schema_untouched(avant: &SchemaSnapshot, apres: &SchemaSnapshot) {
    assert_eq!(
        avant.user_version, apres.user_version,
        "refus : user_version doit rester inchangé"
    );
    assert_eq!(
        avant.master_digest, apres.master_digest,
        "refus : sqlite_master doit rester inchangé (aucune table créée/altérée)"
    );
    assert_eq!(
        avant.migrations, apres.migrations,
        "refus : schema_migrations doit rester inchangé (aucune ligne écrite)"
    );
}

fn assert_migration_required(erreur: &StoreError, found: i64) {
    let message = format!("{erreur}");
    match erreur {
        StoreError::MigrationRequired {
            found: got,
            supported,
        } => {
            assert_eq!(*got, found);
            assert_eq!(*supported, SCHEMA_VERSION);
        }
        other => panic!("attendu MigrationRequired, reçu {other}"),
    }
    assert!(
        message.contains(&format!("schéma SQLite {found}"))
            && message.contains(&format!("attend {SCHEMA_VERSION}"))
            && message.contains("maicie migrate --config <chemin>"),
        "message parlant attendu, reçu : {message}"
    );
}

#[test]
fn base_anterieure_sans_flag_refuse_parlant_sans_ecriture() {
    let root = unique_root("refus");
    let database = root.join("maicie.sqlite3");
    {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .pragma_update(None, "user_version", SCHEMA_VERSION - 1)
        .unwrap();
    drop(connection);
    let avant = schema_snapshot(&database);
    assert_eq!(avant.user_version, SCHEMA_VERSION - 1);

    let erreur = match MaicieStore::open(&database) {
        Ok(_) => panic!("ouverture sans consentement aurait dû refuser"),
        Err(error) => error,
    };
    assert_migration_required(&erreur, SCHEMA_VERSION - 1);
    let apres = schema_snapshot(&database);
    assert_refusal_leaves_schema_untouched(&avant, &apres);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn base_peuplee_user_version_zero_refuse_sans_mutation() {
    // Porte v0 : une base peuplée dont on remet user_version=0 ne doit PAS
    // être traitée comme un bootstrap. Sans --migrate : refus + schéma intact.
    let root = unique_root("porte-v0");
    let database = root.join("maicie.sqlite3");
    {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.pragma_update(None, "user_version", 0).unwrap();
    drop(connection);
    let avant = schema_snapshot(&database);
    assert_eq!(avant.user_version, 0);
    assert!(
        !avant.migrations.is_empty() || !avant.master_digest.is_empty(),
        "précondition : base peuplée"
    );

    let erreur = match MaicieStore::open(&database) {
        Ok(_) => panic!("porte v0 : bootstrap silencieux aurait dû être refusé"),
        Err(error) => error,
    };
    assert_migration_required(&erreur, 0);
    let apres = schema_snapshot(&database);
    assert_refusal_leaves_schema_untouched(&avant, &apres);

    // Avec consentement, la même base avance.
    let store = MaicieStore::open_and_migrate(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    drop(store);
    assert_eq!(schema_snapshot(&database).user_version, SCHEMA_VERSION);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn base_anterieure_avec_consentement_est_migree() {
    let root = unique_root("migrate");
    let database = root.join("maicie.sqlite3");
    {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .pragma_update(None, "user_version", SCHEMA_VERSION - 1)
        .unwrap();
    drop(connection);
    assert_eq!(schema_snapshot(&database).user_version, SCHEMA_VERSION - 1);

    let store = MaicieStore::open_and_migrate(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    drop(store);
    assert_eq!(schema_snapshot(&database).user_version, SCHEMA_VERSION);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn base_neuve_est_cree_sans_flag() {
    let root = unique_root("neuve");
    let database = root.join("maicie.sqlite3");
    assert!(!database.exists());
    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    assert!(database.is_file());
    drop(store);
    assert_eq!(schema_snapshot(&database).user_version, SCHEMA_VERSION);
    fs::remove_dir_all(root).unwrap();
}
