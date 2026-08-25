use maicie::store::{MaicieStore, SCHEMA_VERSION, StoreError};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const EXPECTED_SCHEMA_VERSION: i64 = 19;

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("maicie-v19-{label}-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn hash_file(path: &Path) -> Vec<u8> {
    Sha256::digest(fs::read(path).unwrap()).to_vec()
}

fn schema_snapshot(path: &Path) -> (i64, Vec<(String, String, Option<String>)>) {
    let connection = Connection::open(path).unwrap();
    let version = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    let schema = connection
        .prepare(
            "SELECT type,name,sql FROM sqlite_master
             WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name",
        )
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    (version, schema)
}

fn rebuild_closed_refusal_table(connection: &Connection, include_compound_reason: bool) {
    let compound = if include_compound_reason {
        ",'target_head_moved_and_measured_head_mismatch'"
    } else {
        ""
    };
    connection
        .execute_batch(&format!(
            "ALTER TABLE guichet_refusal_receptions RENAME TO refusal_open;
             CREATE TABLE guichet_refusal_receptions (
                 issuer_scope TEXT NOT NULL,
                 request_id TEXT NOT NULL,
                 canonical_request_bytes BLOB NOT NULL,
                 operation TEXT NOT NULL CHECK(operation IN (
                     'delivery_report','mission_status','deadline_question'
                 )),
                 reason TEXT NOT NULL CHECK(reason IN (
                     'delegation_missing','relation_invalid','envelope_mismatch',
                     'review_verdict_required','review_verdict_unexpected',
                     'review_mandate_mismatch','target_head_moved','measured_head_mismatch'{compound}
                 )),
                 response_message_id TEXT NOT NULL,
                 reply_bytes BLOB NOT NULL,
                 claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
                 claim_token TEXT NOT NULL,
                 processed_at INTEGER NOT NULL,
                 PRIMARY KEY(issuer_scope,request_id,canonical_request_bytes)
             );
             INSERT INTO guichet_refusal_receptions SELECT * FROM refusal_open;
             DROP TABLE refusal_open;"
        ))
        .unwrap();
}

fn downgrade_to_v18(path: &Path) {
    drop(MaicieStore::open(path).unwrap());
    let connection = Connection::open(path).unwrap();
    connection
        .execute(
            "INSERT INTO guichet_refusal_receptions VALUES(
                 'scope-history','request-history',X'0102','delivery_report',
                 'delegation_missing','response-history',X'0304',1,'claim-history',1000
             )",
            [],
        )
        .unwrap();
    rebuild_closed_refusal_table(&connection, true);
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 19", [])
        .unwrap();
    connection.pragma_update(None, "user_version", 18).unwrap();
}

fn downgrade_to_v14(path: &Path) {
    drop(MaicieStore::open(path).unwrap());
    let connection = Connection::open(path).unwrap();
    rebuild_closed_refusal_table(&connection, false);
    connection
        .execute_batch(
            "DROP TRIGGER local_delegate_refusals_append_only_update;
             DROP TRIGGER local_delegate_refusals_append_only_delete;
             DROP INDEX local_delegate_refusals_reason_idx;
             DROP TABLE local_delegate_refusals;
             DROP TABLE routine_occurrences;
             DROP TABLE routines;
             DELETE FROM schema_migrations WHERE version > 14;
             PRAGMA user_version = 14;",
        )
        .unwrap();
}

#[test]
fn v19_succede_a_la_v18_reelle_et_conserve_les_refus_octet_par_octet() {
    let root = root("real-v18");
    let database = root.join("maicie.sqlite3");
    downgrade_to_v18(&database);

    let store = MaicieStore::open_and_migrate(&database).unwrap();
    assert_eq!(SCHEMA_VERSION, EXPECTED_SCHEMA_VERSION);
    assert_eq!(store.schema_version().unwrap(), EXPECTED_SCHEMA_VERSION);
    drop(store);

    let connection = Connection::open(&database).unwrap();
    let historical: (String, String, Vec<u8>, Vec<u8>) = connection
        .query_row(
            "SELECT operation,reason,canonical_request_bytes,reply_bytes
             FROM guichet_refusal_receptions WHERE request_id='request-history'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        historical,
        (
            "delivery_report".to_string(),
            "delegation_missing".to_string(),
            vec![1, 2],
            vec![3, 4],
        )
    );
    let table_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type='table' AND name='guichet_refusal_receptions'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!table_sql.contains("CHECK(operation"));
    assert!(!table_sql.contains("CHECK(reason"));
    let probe_rows: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_refusal_receptions
             WHERE issuer_scope='maicie-v19-shape'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        probe_rows, 0,
        "la sonde v19 doit être intégralement rollbackée"
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn v19_refuse_une_estampille_v18_sans_son_ddl_et_ne_mute_rien() {
    let root = root("false-v18");
    let database = root.join("maicie.sqlite3");
    downgrade_to_v18(&database);
    let connection = Connection::open(&database).unwrap();
    connection
        .execute_batch(
            "DROP TRIGGER local_delegate_refusals_append_only_update;
             DROP TRIGGER local_delegate_refusals_append_only_delete;
             DROP INDEX local_delegate_refusals_reason_idx;
             DROP TABLE local_delegate_refusals;",
        )
        .unwrap();
    drop(connection);
    let before = schema_snapshot(&database);

    let error = match MaicieStore::open_and_migrate(&database) {
        Ok(_) => panic!("le numéro v18 seul ne doit pas autoriser v19"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        StoreError::Corrupt("forme v18 du greffe local incompatible avec v19")
    ));
    assert_eq!(schema_snapshot(&database), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn v19_refuse_son_propre_numero_si_le_check_v18_subsiste() {
    let root = root("false-v19");
    let database = root.join("maicie.sqlite3");
    downgrade_to_v18(&database);
    let connection = Connection::open(&database).unwrap();
    connection
        .execute(
            "INSERT INTO schema_migrations(version,applied_at) VALUES(19,1000)",
            [],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 19).unwrap();
    drop(connection);
    let before = schema_snapshot(&database);

    let error = match MaicieStore::open(&database) {
        Ok(_) => panic!("le numéro v19 seul ne doit pas masquer le CHECK v18"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        StoreError::Corrupt("forme v19 des refus fédérés incomplète")
    ));
    assert_eq!(schema_snapshot(&database), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn parcours_prive_v14_v17_v18_v19_ne_mute_jamais_sa_source() {
    let root = root("production-path");
    let source = root.join("source-v14.sqlite3");
    let copy = root.join("copy-v14.sqlite3");
    downgrade_to_v14(&source);
    let source_before = hash_file(&source);
    fs::copy(&source, &copy).unwrap();
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o600)).unwrap();
    let copy_before = hash_file(&copy);
    assert_eq!(source_before, copy_before);

    let store = MaicieStore::open_and_migrate(&copy).unwrap();
    assert_eq!(store.schema_version().unwrap(), EXPECTED_SCHEMA_VERSION);
    drop(store);

    assert_eq!(hash_file(&source), source_before);
    assert_ne!(hash_file(&copy), copy_before);
    assert_eq!(schema_snapshot(&source).0, 14);
    assert_eq!(schema_snapshot(&copy).0, EXPECTED_SCHEMA_VERSION);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bootstrap_vide_exerce_separement_tous_les_paliers_jusqu_a_v19() {
    let root = root("bootstrap");
    let database = root.join("maicie.sqlite3");
    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), EXPECTED_SCHEMA_VERSION);
    drop(store);
    let connection = Connection::open(&database).unwrap();
    let migrations: Vec<i64> = connection
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(migrations.last(), Some(&EXPECTED_SCHEMA_VERSION));
    assert!(migrations.windows(2).all(|pair| pair[1] == pair[0] + 1));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}
