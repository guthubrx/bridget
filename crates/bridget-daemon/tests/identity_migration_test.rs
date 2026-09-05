use bridget_daemon::agent_profile::AgentProfileStore;
use bridget_daemon::desired_state::{DesiredEquipier, DesiredLifecycleState, DesiredStateStore};
use bridget_daemon::identity_migration::{IdentityMigrationPaths, apply, plan};
use bridget_daemon::store::Store;
use std::os::unix::fs::DirBuilderExt;

#[test]
fn migration_reindexe_ledger_et_supprime_alias_legacy() {
    let root = std::env::temp_dir().join(format!(
        "bridget-identity-integration-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let db = root.join("bridget.db");
    let fleet = root.join("fleet.json");
    drop(Store::open(&db).unwrap());
    let connection = rusqlite::Connection::open(&db).unwrap();
    connection.execute(
        "INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["message-1", 1_i64, "agent-historique", "human", "bonjour", "agent-historique:human"],
    ).unwrap();

    let migration = plan(IdentityMigrationPaths {
        bridget_db: db.clone(),
        fleet_path: fleet,
    })
    .unwrap();
    let agent_id = migration.mapping.get("agent-historique").cloned().unwrap();
    apply(&migration).unwrap();

    let store = Store::open(&db).unwrap();
    let messages = store.participant_messages("human", 10).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].sender, agent_id);
    assert!(
        AgentProfileStore::open(&db)
            .unwrap()
            .legacy_routing_map()
            .unwrap()
            .is_empty()
    );
    let connection = rusqlite::Connection::open(&db).unwrap();
    let alias_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('agent_routing_aliases', 'identity_migration_aliases')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        alias_tables, 0,
        "aucune table de routage legacy ne subsiste"
    );
    let routing_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('agent_identities') WHERE name = 'current_routing_name'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(routing_columns, 0, "la colonne legacy doit être supprimée");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_reindexe_la_flotte_et_sauvegarde_les_sources() {
    let root = std::env::temp_dir().join(format!(
        "bridget-identity-config-fleet-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let db = root.join("bridget.db");
    let fleet = root.join("fleet.json");
    drop(Store::open(&db).unwrap());

    DesiredStateStore::at_path(&fleet)
        .upsert(
            "agent-declare".to_string(),
            DesiredEquipier {
                agent_type: "codex".to_string(),
                cwd: root.clone(),
                command_id: "migration-command".to_string(),
                generation: 1,
                created: "2026-08-31T00:00:00Z".to_string(),
                persistent: true,
                lifecycle_state: DesiredLifecycleState::Running,
                resolved_definition: None,
                domain: None,
                project: None,
                agent_link: None,
                runtime_execution: None,
            },
        )
        .unwrap();

    let migration = plan(IdentityMigrationPaths {
        bridget_db: db,
        fleet_path: fleet.clone(),
    })
    .unwrap();
    let agent_id = migration.mapping.get("agent-declare").cloned().unwrap();
    let before = std::fs::read(&fleet).unwrap();
    let outcome = apply(&migration).unwrap();
    assert_eq!(outcome.backup_paths.len(), 2);
    let fleet_backup = outcome
        .backup_paths
        .iter()
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("fleet.json.")
        })
        .unwrap();
    assert_eq!(std::fs::read(fleet_backup).unwrap(), before);
    assert!(
        DesiredStateStore::at_path(&fleet)
            .load()
            .unwrap()
            .equipiers
            .contains_key(&agent_id)
    );
    std::fs::remove_dir_all(root).unwrap();
}
