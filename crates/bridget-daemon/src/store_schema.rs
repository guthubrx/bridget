//! Préflight partagé, lecture seule, avant les migrations des deux magasins.
use rusqlite::{Connection, OptionalExtension};

pub const MAX_IDEMPOTENCY_SCHEMA_VERSION: i64 = 6;

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
