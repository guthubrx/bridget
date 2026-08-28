//! Supports communs aux scénarios de migration historique.
//!
//! Les tests qui abaissent une base courante doivent aussi restaurer ce DDL :
//! changer seulement `user_version` fabriquerait un prédécesseur impossible.

use maicie::install_publish::{PublishedMigrationOpenError, republish_exe_before_migrate};
use maicie::store::{MaicieStore, StoreError};
use rusqlite::Connection;
use std::path::Path;

/// Publie le binaire testé avant de consentir à la migration, puis rouvre le
/// store courant. Les erreurs de publication relèvent du montage ; les erreurs
/// du store restent observables par les scénarios de corruption historique.
pub fn open_after_published_migration(database: &Path) -> Result<MaicieStore, StoreError> {
    let source = Path::new(env!("CARGO_BIN_EXE_maicie"));
    let installed = database.with_extension("test-published-maicie");
    let publication = republish_exe_before_migrate(source, &installed, database)
        .unwrap_or_else(|error| panic!("republication du migrateur de test impossible: {error}"));
    match publication.open() {
        Ok(migrated) => {
            drop(migrated);
            MaicieStore::open(database)
        }
        Err(PublishedMigrationOpenError::Store(error)) => Err(error),
        Err(PublishedMigrationOpenError::Install(error)) => {
            panic!("ouverture publiée du migrateur de test impossible: {error}")
        }
    }
}

pub fn rebuild_v19_guichet_receptions(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "ALTER TABLE guichet_receptions RENAME TO guichet_receptions_open;
         CREATE TABLE guichet_receptions (
             issuer_scope TEXT NOT NULL,
             request_id TEXT NOT NULL,
             operation TEXT NOT NULL CHECK(operation IN ('delivery_report','mission_status','deadline_question')),
             canonical_request_bytes BLOB NOT NULL,
             objective_id TEXT,
             delegation_id TEXT,
             delivery_hash TEXT,
             in_reply_to TEXT,
             response_message_id TEXT NOT NULL,
             claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
             claim_token TEXT NOT NULL,
             outcome TEXT NOT NULL CHECK(outcome IN ('accepted','request_already_terminal')),
             reply_bytes BLOB NOT NULL,
             decision_id TEXT,
             processed_at INTEGER NOT NULL,
             PRIMARY KEY(issuer_scope, request_id),
             UNIQUE(in_reply_to, response_message_id)
         );
         INSERT INTO guichet_receptions(
             issuer_scope,request_id,operation,canonical_request_bytes,
             objective_id,delegation_id,delivery_hash,in_reply_to,response_message_id,
             claim_generation,claim_token,outcome,reply_bytes,decision_id,processed_at
         )
         SELECT issuer_scope,request_id,operation,canonical_request_bytes,
                objective_id,delegation_id,delivery_hash,in_reply_to,response_message_id,
                claim_generation,claim_token,outcome,reply_bytes,decision_id,processed_at
         FROM guichet_receptions_open;
         DROP TABLE guichet_receptions_open;",
    )
}
