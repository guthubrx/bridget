//! Préflight partagé, lecture seule, avant les migrations des deux magasins.
use rusqlite::{Connection, OptionalExtension};

pub const MAX_IDEMPOTENCY_SCHEMA_VERSION: i64 = 8;

#[derive(Debug)]
pub enum SchemaError {
    Io(std::io::Error),
    UnsupportedVersion { found: i64, supported: i64 },
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "inspection du fichier de base : {error}"),
            Self::UnsupportedVersion { found, supported } => write!(
                f,
                "schéma idempotence {found} non pris en charge (versions 1..={supported}) ; aucune migration"
            ),
            Self::Sqlite(error) => write!(f, "inspection du schéma : {error}"),
        }
    }
}
impl std::error::Error for SchemaError {}

/// Le bootstrap appelle ce préflight après validation UID/type du chemin,
/// avant socket, récupération des groupes et toute ouverture migrante.
pub(crate) fn validate_existing(path: &std::path::Path) -> Result<(), SchemaError> {
    if !path.try_exists().map_err(SchemaError::Io)? {
        return Ok(());
    }
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(SchemaError::Sqlite)?;
    validate(&conn)
}

/// Aucun CREATE, PRAGMA d'écriture ni réparation : une base d'une version
/// future n'est pas une base ancienne à compléter. L'absence de table est le
/// format historique v1, pris en charge par les migrations existantes.
pub(crate) fn validate(conn: &Connection) -> Result<(), SchemaError> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='idempotency_schema_migrations')",
        [], |r| r.get(0)).map_err(SchemaError::Sqlite)?;
    if !exists {
        return Ok(());
    }
    let unknown: Option<i64> = conn.query_row(
        "SELECT version FROM idempotency_schema_migrations WHERE version < 1 OR version > ?1 ORDER BY version DESC LIMIT 1",
        [MAX_IDEMPOTENCY_SCHEMA_VERSION], |r| r.get(0)).optional().map_err(SchemaError::Sqlite)?;
    if let Some(found) = unknown {
        return Err(SchemaError::UnsupportedVersion {
            found,
            supported: MAX_IDEMPOTENCY_SCHEMA_VERSION,
        });
    }
    Ok(())
}

/// Ouvre ou crée un fichier SQLite privé (`0600`, `O_NOFOLLOW`) configuré en mode
/// journal concurrent `WAL` avec `busy_timeout` et clés étrangères actives (ADR 260).
pub(crate) fn open_private_wal(path: &std::path::Path) -> Result<Connection, SchemaError> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink()
            || !meta.is_file()
            || meta.uid() != bridget_transport::fsutil::effective_uid()
            || meta.mode() & 0o077 != 0
        {
            return Err(SchemaError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fichier SQLite privé 0600 possédé par le compte requis",
            )));
        }
    } else {
        std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(SchemaError::Io)?;
    }
    let conn = Connection::open(path).map_err(SchemaError::Sqlite)?;
    conn.busy_timeout(std::time::Duration::from_secs(2))
        .map_err(SchemaError::Sqlite)?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "foreign_keys", "ON");
    Ok(conn)
}

/// Résumé unifié de versionnement d'un magasin SQLite (ADR 260, option C).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoreSchemaInfo {
    pub mechanism: &'static str,
    pub version: i64,
    pub journal_mode: String,
}

/// Lit la version exposée par n'importe quel magasin SQLite Bridget sans écrire.
pub(crate) fn inspect_store_schema(conn: &Connection) -> Result<StoreSchemaInfo, SchemaError> {
    let journal_mode: String = conn
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .map_err(SchemaError::Sqlite)?;
    let has_idem: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='idempotency_schema_migrations')",
            [],
            |r| r.get(0),
        )
        .map_err(SchemaError::Sqlite)?;
    if has_idem {
        let ver: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM idempotency_schema_migrations",
                [],
                |r| r.get(0),
            )
            .map_err(SchemaError::Sqlite)?;
        return Ok(StoreSchemaInfo {
            mechanism: "idempotency_schema_migrations",
            version: ver,
            journal_mode,
        });
    }
    let user_version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(SchemaError::Sqlite)?;
    if user_version > 0 {
        return Ok(StoreSchemaInfo {
            mechanism: "pragma_user_version",
            version: user_version,
            journal_mode,
        });
    }
    Ok(StoreSchemaInfo {
        mechanism: "implicit_v1",
        version: 1,
        journal_mode,
    })
}

/// T003 : préflight en lecture seule ; l’ancien plafond6 devra refuser cette v7.
#[cfg(test)]
#[test]
fn tracking230_migration_precontrole_accepte_7_et_8_refuse_9() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE idempotency_schema_migrations(version INTEGER PRIMARY KEY);
        INSERT INTO idempotency_schema_migrations(version) VALUES (7);",
    )
    .unwrap();
    let before = conn.total_changes();
    assert!(
        validate(&conn).is_ok(),
        "v7 doit être admise avant les migrations"
    );
    assert_eq!(
        conn.total_changes(),
        before,
        "préflight ne modifie pas la base"
    );
    conn.execute("UPDATE idempotency_schema_migrations SET version=8", [])
        .unwrap();
    assert!(
        validate(&conn).is_ok(),
        "le lecteur du sommeil doit comprendre v8"
    );
    conn.execute("UPDATE idempotency_schema_migrations SET version=9", [])
        .unwrap();
    let before = conn.total_changes();
    assert!(
        matches!(
            validate(&conn),
            Err(SchemaError::UnsupportedVersion {
                found: 9,
                supported: 8
            })
        ),
        "v9 doit rester fermée ; elle ne doit être ni migrée ni réparée"
    );
    assert_eq!(conn.total_changes(), before);
    assert_eq!(
        conn.query_row(
            "SELECT version FROM idempotency_schema_migrations",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        9
    );
}

#[cfg(test)]
#[test]
fn spec306_open_private_wal_and_inspect_schema_cover_all_stores() {
    let dir = std::env::temp_dir().join(format!("bridget-wal-{}", uuid::Uuid::new_v4()));
    bridget_transport::fsutil::create_private_dir(&dir).unwrap();
    let db = dir.join("test.db");
    let conn = open_private_wal(&db).unwrap();
    let info = inspect_store_schema(&conn).unwrap();
    assert_eq!(info.journal_mode.to_lowercase(), "wal");
    assert_eq!(info.version, 1);

    conn.pragma_update(None, "user_version", 2i64).unwrap();
    let info_v2 = inspect_store_schema(&conn).unwrap();
    assert_eq!(info_v2.mechanism, "pragma_user_version");
    assert_eq!(info_v2.version, 2);

    conn.execute_batch(
        "CREATE TABLE idempotency_schema_migrations(version INTEGER PRIMARY KEY);
         INSERT INTO idempotency_schema_migrations(version) VALUES (8);",
    )
    .unwrap();
    let info_idem = inspect_store_schema(&conn).unwrap();
    assert_eq!(info_idem.mechanism, "idempotency_schema_migrations");
    assert_eq!(info_idem.version, 8);
    drop(conn);
    let _ = std::fs::remove_dir_all(&dir);
}
