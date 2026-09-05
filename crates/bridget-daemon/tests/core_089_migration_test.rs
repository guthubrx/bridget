//! Copie volontaire d'une base historique ; aucune lecture de la base utilisateur.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_daemon::idempotency::{IdempotencyKey, IdempotencyStore, LookupResult, OperationKind};
use bridget_daemon::store::Store;
use fixture::*;
use rusqlite::{Connection, OpenFlags, params, types::Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn snapshot(path: &Path) -> Vec<(String, Vec<Vec<Value>>)> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let names = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    names
        .into_iter()
        .map(|name| {
            let mut query = conn
                .prepare(&format!(
                    "SELECT * FROM \"{}\" ORDER BY rowid",
                    name.replace('"', "\"\"")
                ))
                .unwrap();
            let columns = query.column_count();
            let rows = query
                .query_map([], |r| {
                    (0..columns)
                        .map(|index| r.get(index))
                        .collect::<Result<Vec<Value>, _>>()
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (name, rows)
        })
        .collect()
}

#[test]
fn migration_sur_copie_garde_ids_canons_terminaux_et_source_octet_identique() {
    let root = test_root("migration-copy");
    let source = root.join("source-historique.db");
    private_write(&source, b"").unwrap();
    let old = Connection::open(&source).unwrap();
    old.execute_batch(include_str!("fixtures/core_089/idempotency-v1.sql"))
        .unwrap();
    let canonical = b"bridget/client-send/v1\0  bytes historiques\n\xff";
    let payload = b"{ \"champ_inconnu\" : true, \"texte\": \"  intact  \" }\n";
    for (id, phase, message_bytes) in [
        ("accepted", "acked", Some(payload.as_slice())),
        ("in-flight", "dispatching", Some(payload.as_slice())),
        ("missing-payload", "dispatching", None),
    ] {
        old.execute("INSERT INTO idempotency_records VALUES (?1,'send',?2,?3,?4,?5,NULL,NULL,1000,9999999999)",
            params![SCOPE,id,canonical.as_slice(),if id=="accepted" {"terminal"} else {"dispatching"},(id=="accepted").then_some("accepted")]).unwrap();
        old.execute("INSERT INTO send_deliveries VALUES (?1,?2,'send',?1,'instance-historique',3,?3,9999999999,?4)",
            params![id,SCOPE,phase,message_bytes]).unwrap();
    }
    drop(old);
    fs::set_permissions(&source, fs::Permissions::from_mode(0o400)).unwrap();
    let source_bytes = fs::read(&source).unwrap();
    let target = root.join("state/bridget.db");
    fs::copy(&source, &target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    let mut stable = None;
    for _ in 0..3 {
        let store = Store::open(&target).unwrap();
        let idem = IdempotencyStore::open(&target).unwrap();
        let key = IdempotencyKey::new(SCOPE, OperationKind::Send, "accepted").unwrap();
        assert_eq!(
            idem.lookup(&key, 1001).unwrap(),
            LookupResult::Accepted {
                expires_at: 9999999999
            }
        );
        drop(idem);
        drop(store);
        let conn = Connection::open_with_flags(&target, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let records = conn.prepare("SELECT idempotency_key,canonical_bytes FROM idempotency_records ORDER BY idempotency_key").unwrap()
            .query_map([], |r| Ok((r.get::<_,String>(0)?,r.get::<_,Vec<u8>>(1)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(
            records
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>(),
            ["accepted", "in-flight", "missing-payload"]
        );
        assert!(records.iter().all(|(_, bytes)| bytes == canonical));
        for (id, expected) in [
            ("accepted", "acked"),
            ("in-flight", "dispatching"),
            ("missing-payload", "indeterminate"),
        ] {
            let (phase, bytes): (String, Option<Vec<u8>>) = conn
                .query_row(
                    "SELECT phase,message_bytes FROM send_deliveries WHERE delivery_id=?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(phase, expected);
            assert_eq!(
                bytes.as_deref(),
                (id != "missing-payload").then_some(payload.as_slice())
            );
        }
        drop(conn);
        let observed = snapshot(&target);
        if let Some(before) = &stable {
            assert_eq!(
                &observed, before,
                "la seconde migration ne réécrit pas les faits"
            );
        }
        stable = Some(observed);
        assert_eq!(
            fs::read(&source).unwrap(),
            source_bytes,
            "source jamais ouverte en écriture"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn un_schema_futur_est_refuse_avant_toute_migration() {
    for component in ["store", "idempotency", "daemon"] {
        let root = test_root("migration-future");
        let database = root.join("state/bridget.db");
        private_write(&database, b"").unwrap();
        let conn = Connection::open(&database).unwrap();
        conn.execute_batch("CREATE TABLE idempotency_schema_migrations(version INTEGER PRIMARY KEY); INSERT INTO idempotency_schema_migrations VALUES(999); CREATE TABLE future_sentinel(bytes BLOB); INSERT INTO future_sentinel VALUES(X'00FF0A');").unwrap();
        drop(conn);
        let before = fs::read(&database).unwrap();
        let error = match component {
            "store" => Store::open(&database).err().map(|e| e.to_string()),
            "daemon" => {
                let output = run_isolated(&root, &["daemon"], false);
                assert!(!output.status.success());
                assert!(!socket(&root).exists(), "aucune ouverture aux clients");
                Some(output_text(&output))
            }
            _ => IdempotencyStore::open(&database)
                .err()
                .map(|e| e.to_string()),
        };
        assert!(error.is_some(), "{component} a accepté un schéma inconnu");
        assert!(error.unwrap().contains("999"));
        assert_eq!(
            fs::read(&database).unwrap(),
            before,
            "aucune mutation avant refus du schéma futur"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
