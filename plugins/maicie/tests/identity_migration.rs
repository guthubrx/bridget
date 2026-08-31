use maicie::store::MaicieStore;
use std::collections::BTreeMap;

#[test]
fn migration_maicie_recrit_les_cibles_outbox_sans_reassignation_implicite() {
    let root = std::env::temp_dir().join(format!(
        "maicie-identity-integration-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(unix)]
    std::fs::set_permissions(&root, std::os::unix::fs::PermissionsExt::from_mode(0o700)).unwrap();
    let db = root.join("maicie.db");
    let mut store = MaicieStore::open(&db).unwrap();
    let connection = rusqlite::Connection::open(&db).unwrap();
    connection
        .execute(
            "INSERT INTO objectives (id, state, payload_json) VALUES (?1, ?2, ?3)",
            rusqlite::params!["objective-1", "ouverte", b"{}".to_vec()],
        )
        .unwrap();
    connection.execute(
        "INSERT INTO delegations (id, objective_id, state, payload_json) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["delegation-1", "objective-1", "creee", br#"{"participant":"agent-historique"}"#.to_vec()],
    ).unwrap();
    connection
        .execute(
            "INSERT INTO delegation_outbox (
            message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
            body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash, message_bytes,
            state, retry_until, dedup_retained_until
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                "message-1",
                "delegation-1",
                "objective-1",
                "scope",
                1_i64,
                "agent-historique",
                b"body".to_vec(),
                0_i64,
                60_i64,
                61_i64,
                b"hash".to_vec(),
                br#"{"to":"agent-historique"}"#.to_vec(),
                "prepared",
                1_i64,
                2_i64
            ],
        )
        .unwrap();
    drop(connection);

    let mut mapping = BTreeMap::new();
    mapping.insert(
        "agent-historique".to_string(),
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
    );
    store.migrate_agent_participants(&mapping).unwrap();

    let mut retarget = BTreeMap::new();
    retarget.insert(
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        "ignored".to_string(),
    );
    let blocked = retarget.keys().cloned().collect();
    store.mark_agents_requires_retarget(&blocked).unwrap();
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());

    let connection = rusqlite::Connection::open(&db).unwrap();
    let target: String = connection
        .query_row(
            "SELECT target FROM delegation_outbox WHERE message_id = 'message-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(target, "550e8400-e29b-41d4-a716-446655440000");
    std::fs::remove_dir_all(root).unwrap();
}
