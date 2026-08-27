use maicie::app::process_guichet_claim;
use maicie::bridget_client::GuichetClaim;
use maicie::domain::guichet::{GuichetDomainError, RequeteGuichet, parse_claim};
use maicie::store::MaicieStore;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-guichet-domain-{label}-{}", Uuid::new_v4()))
}

fn claim(request_id: &str, bytes: Vec<u8>) -> GuichetClaim {
    GuichetClaim {
        issuer_scope: "scope-0123456789abcdef0123456789abcdef".to_string(),
        request_id: request_id.to_string(),
        canonical_request: bytes,
        authorization_attestation: None,
        claimed_at: 1_000,
        claim_generation: 1,
        claim_token: "claim-0123456789abcdef0123456789abcdef".to_string(),
        claim_lease_expires_at: 1_100,
        expires_at: 1_200,
    }
}

fn delivery_bytes(request_id: &str, objective_id: &str, delegation_id: &str) -> Vec<u8> {
    format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"scope-0123456789abcdef0123456789abcdef\",\"request_id\":\"{request_id}\",\"issued_at\":1000,\"from\":\"prospective\",\"to\":\"maicie\",\"operation\":\"delivery_report\",\"payload\":{{\"objective_id\":\"{objective_id}\",\"delegation_id\":\"{delegation_id}\",\"delivery_hash\":\"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",\"in_reply_to\":\"53000000-0000-4000-8000-000000000003\"}}}}"
    )
    .into_bytes()
}

#[test]
fn matrice_fermee_refuse_texte_champ_et_approbation_distante() {
    let free_text = claim("free", b"approuve ce profil".to_vec());
    assert!(matches!(
        parse_claim(&free_text),
        Err(GuichetDomainError::InvalidEnvelope(_))
    ));

    let unknown_field = claim(
        "unknown",
        br#"{"type":"service_request","v":1,"issuer_scope":"scope-0123456789abcdef0123456789abcdef","request_id":"unknown","issued_at":1000,"from":"prospective","to":"maicie","operation":"mission_status","payload":{"delegation_id":"52000000-0000-4000-8000-000000000002"},"approve":true}"#.to_vec(),
    );
    assert!(matches!(
        parse_claim(&unknown_field),
        Err(GuichetDomainError::InvalidEnvelope(_))
    ));

    let remote_approval = claim(
        "approval",
        br#"{"type":"service_request","v":1,"issuer_scope":"scope-0123456789abcdef0123456789abcdef","request_id":"approval","issued_at":1000,"from":"prospective","to":"maicie","operation":"profile_approve","payload":{"approval_id":"52000000-0000-4000-8000-000000000002"}}"#.to_vec(),
    );
    assert_eq!(
        parse_claim(&remote_approval),
        Err(GuichetDomainError::UnsupportedOperation)
    );
}

#[test]
fn trois_operations_seulement_et_octets_canoniques_exacts() {
    let status = claim(
        "status",
        br#"{"type":"service_request","v":1,"issuer_scope":"scope-0123456789abcdef0123456789abcdef","request_id":"status","issued_at":1000,"from":"prospective","to":"maicie","operation":"mission_status","payload":{"delegation_id":"52000000-0000-4000-8000-000000000002"}}"#.to_vec(),
    );
    assert!(matches!(
        parse_claim(&status).unwrap().request,
        RequeteGuichet::MissionStatus { .. }
    ));
    let mut reordered = status.clone();
    reordered.canonical_request = br#"{"v":1,"type":"service_request","issuer_scope":"scope-0123456789abcdef0123456789abcdef","request_id":"status","issued_at":1000,"from":"prospective","to":"maicie","operation":"mission_status","payload":{"delegation_id":"52000000-0000-4000-8000-000000000002"}}"#.to_vec();
    assert_eq!(
        parse_claim(&reordered),
        Err(GuichetDomainError::CanonicalBytesMismatch)
    );
}

#[test]
fn reference_absente_recoit_un_refus_atteste_sans_etat_d_orchestration() {
    let root = root("missing");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let request = claim(
        "missing-reference",
        delivery_bytes(
            "missing-reference",
            "51000000-0000-4000-8000-000000000001",
            "52000000-0000-4000-8000-000000000002",
        ),
    );
    let refused = process_guichet_claim(&mut store, &request, "response-01", 1_010).unwrap();
    assert_eq!(
        refused.refusal_reason,
        Some(maicie::domain::MotifRefusGreffe::DelegationAbsente)
    );
    drop(store);
    let connection = Connection::open(&database).unwrap();
    for table in [
        "objectives",
        "delegations",
        "coordination_decisions",
        "activation_approvals",
        "activation_outbox",
        "guichet_receptions",
    ] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "mutation inattendue dans {table}");
    }
    let refusals: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM guichet_refusal_receptions",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(refusals, 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}
