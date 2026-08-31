use bridget_daemon::agent_profile::AgentProfileStore;
use bridget_daemon::desired_state::{DesiredEquipier, DesiredLifecycleState, DesiredStateStore};
use bridget_daemon::identity_migration::{IdentityMigrationPaths, apply, plan};
use bridget_daemon::store::Store;

#[test]
fn migration_reindexe_ledger_et_supprime_alias_legacy() {
    let root = std::env::temp_dir().join(format!(
        "bridget-identity-integration-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
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
        maicie_config: None,
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
fn migration_conserve_un_agent_declare_hors_flotte_et_reindexe_les_fichiers() {
    let root = std::env::temp_dir().join(format!(
        "bridget-identity-config-fleet-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let db = root.join("bridget.db");
    let fleet = root.join("fleet.json");
    let maicie_db = root.join("maicie.db");
    let maicie_config = root.join("maicie.json");
    drop(Store::open(&db).unwrap());
    drop(maicie::store::MaicieStore::open(&maicie_db).unwrap());

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

    std::fs::write(
        &maicie_config,
        serde_json::json!({
            "version": 1,
            "bridget_socket": root.join("bridget.sock"),
            "database_path": maicie_db,
            "durations": { "short_secs": 60, "normal_secs": 120, "long_secs": 180 },
            "profiles": [{
                "id": "profil-declare",
                "agent_name": "agent-declare",
                "display_name": "Agent déclaré",
                "tags": [],
                "personality_ref": "personality/declaratif",
                "tools": [],
                "spawn_order_ref": "spawn/declaratif"
            }],
            "project_profiles": []
        })
        .to_string(),
    )
    .unwrap();

    let connection = rusqlite::Connection::open(&maicie_db).unwrap();
    connection
        .execute(
            "INSERT INTO objectives (id, state, payload_json) VALUES (?1, ?2, ?3)",
            rusqlite::params!["objective-retire", "ouverte", b"{}".to_vec()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO delegations (id, objective_id, state, payload_json) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                "delegation-retire",
                "objective-retire",
                "creee",
                br#"{"participant":"agent-retire"}"#.to_vec(),
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO delegation_outbox (
                message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
                body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash, message_bytes,
                state, retry_until, dedup_retained_until
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                "message-retire",
                "delegation-retire",
                "objective-retire",
                "scope",
                1_i64,
                "agent-retire",
                b"body".to_vec(),
                0_i64,
                60_i64,
                61_i64,
                b"hash".to_vec(),
                br#"{"to":"agent-retire"}"#.to_vec(),
                "prepared",
                1_i64,
                2_i64,
            ],
        )
        .unwrap();
    drop(connection);

    let migration = plan(IdentityMigrationPaths {
        bridget_db: db,
        fleet_path: fleet.clone(),
        maicie_config: Some(maicie_config.clone()),
    })
    .unwrap();
    let agent_id = migration.mapping.get("agent-declare").cloned().unwrap();
    let retired_agent_id = migration.mapping.get("agent-retire").cloned().unwrap();
    assert_eq!(migration.requires_retarget.len(), 1);
    assert!(migration.requires_retarget.contains(&retired_agent_id));
    apply(&migration).unwrap();

    assert!(
        DesiredStateStore::at_path(&fleet)
            .load()
            .unwrap()
            .equipiers
            .contains_key(&agent_id)
    );
    let config = maicie::config::MaicieConfig::load(&maicie_config).unwrap();
    assert_eq!(
        config.profiles[0].agent_id.as_deref(),
        Some(agent_id.as_str())
    );
    assert!(
        !std::fs::read_to_string(&maicie_config)
            .unwrap()
            .contains("agent_name")
    );
    assert!(
        maicie::store::MaicieStore::open(&maicie_db)
            .unwrap()
            .pending_delegation_outboxes()
            .unwrap()
            .is_empty()
    );

    std::fs::remove_dir_all(root).unwrap();
}
