use maicie::app::process_guichet_claim;
use maicie::bridget_client::GuichetClaim;
use maicie::store::MaicieStore;
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use uuid::Uuid;

fn claim(operation: &str) -> GuichetClaim {
    let request_id = format!("forbidden-{operation}");
    let bytes = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"scope-0123456789abcdef0123456789abcdef\",\"request_id\":\"{request_id}\",\"issued_at\":1000,\"from\":\"jc6\",\"to\":\"maicie\",\"operation\":\"{operation}\",\"payload\":{{\"approval_id\":\"52000000-0000-4000-8000-000000000002\"}}}}"
    )
    .into_bytes();
    GuichetClaim {
        issuer_scope: "scope-0123456789abcdef0123456789abcdef".into(),
        request_id,
        canonical_request: bytes,
        claimed_at: 1_000,
        claim_generation: 1,
        claim_token: "claim-0123456789abcdef0123456789abcdef".into(),
        claim_lease_expires_at: 1_100,
        expires_at: 1_200,
    }
}

fn assert_forbidden_operation_is_durable(operation: &str) {
    let root = std::env::temp_dir().join(format!(
        "maicie-guichet-forbidden-{operation}-{}",
        Uuid::new_v4()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let result = process_guichet_claim(&mut store, &claim(operation), "response-026", 1_010);
    drop(store);

    let connection = Connection::open(&database).unwrap();
    let refusal_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_refusal_receptions WHERE operation = ?1",
            [operation],
            |row| row.get(0),
        )
        .unwrap();
    let mutation_count: i64 = [
        "objectives",
        "delegations",
        "activation_approvals",
        "activation_outbox",
        "routines",
    ]
    .into_iter()
    .map(|table| {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap()
    })
    .sum();
    drop(connection);
    fs::remove_dir_all(root).unwrap();

    let processed = result.unwrap_or_else(|error| {
        panic!("{operation} doit produire un refus terminal durable : {error}")
    });
    let reply: Value = serde_json::from_slice(&processed.reply_bytes).unwrap();
    assert_eq!(reply["outcome"], "refused");
    assert_eq!(reply["payload"]["kind"], "refused");
    assert_eq!(reply["payload"]["operation"], operation);
    assert_eq!(reply["payload"]["reason"], "operation_not_allowed");
    assert_eq!(
        refusal_count, 1,
        "le refus doit laisser exactement une ligne"
    );
    assert_eq!(mutation_count, 0, "une approbation interdite ne mute rien");
}

#[test]
fn profile_approve_est_refuse_et_persiste_exactement() {
    assert_forbidden_operation_is_durable("profile_approve");
}

#[test]
fn routine_approve_est_refuse_et_persiste_exactement() {
    assert_forbidden_operation_is_durable("routine_approve");
}
