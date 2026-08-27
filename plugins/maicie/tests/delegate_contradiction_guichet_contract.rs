use bridget_transport::protocol::{
    GuichetOutcome, GuichetRefusalReason, GuichetReplyPayload, ServiceRequestOperation,
    WrapperToDaemon, decode,
};
use maicie::app::{
    DelegateRequest, DelegateResult, DelegationCandidate, GuichetError, delegate,
    process_guichet_claim,
};
use maicie::bridget_client::GuichetClaim;
use maicie::config::DurationClasses;
use maicie::domain::{ClasseDuree, MotifRefusGreffe, OperationGuichet, SuiteObjective};
use maicie::store::MaicieStore;
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const ISSUER_SCOPE: &str = "scope-0123456789abcdef0123456789abcdef";
const CLAIM_TOKEN: &str = "claim-0123456789abcdef0123456789abcdef";

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "maicie-delegate-guichet-{label}-{}",
        Uuid::new_v4()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn seed_objective(store: &mut MaicieStore) -> Uuid {
    let request = DelegateRequest {
        goal: "objectif préalable",
        explicit_target: Some("prospective"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: false,
        constat_id: None,
        review_target: None,
        suite: SuiteObjective::Aucune,
        depends_on: &[],
        references: &[],
        idempotency_key: "seed-guichet-contradiction",
        now: 1_787_671_000,
        retry_until: 1_787_671_100,
        dedup_retained_until: 1_787_671_200,
        max_frame_bytes: 256 * 1024,
    };
    let candidates = [DelegationCandidate {
        name: "prospective".to_string(),
        tags: Vec::new(),
        available: true,
        dnd: false,
    }];
    let durations = DurationClasses {
        short_secs: 30,
        normal_secs: 60,
        long_secs: 90,
    };
    let DelegateResult::Created(created) =
        delegate(store, durations, "maicie", &candidates, &request).unwrap()
    else {
        panic!("création préalable attendue")
    };
    created.objective_id
}

fn claim(request_id: &str, goal: &str) -> GuichetClaim {
    let goal = serde_json::to_string(goal).unwrap();
    let canonical_request = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"{ISSUER_SCOPE}\",\"request_id\":\"{request_id}\",\"issued_at\":1787671000,\"from\":\"jc6\",\"to\":\"maicie\",\"operation\":\"delegate\",\"payload\":{{\"goal\":{goal},\"explicit_target\":\"prospective\",\"duration\":\"normale\",\"suite\":{{\"kind\":\"aucune\"}}}}}}"
    )
    .into_bytes();
    GuichetClaim {
        issuer_scope: ISSUER_SCOPE.to_string(),
        request_id: request_id.to_string(),
        canonical_request,
        authorization_attestation: None,
        claimed_at: 1_787_671_001,
        claim_generation: 1,
        claim_token: CLAIM_TOKEN.to_string(),
        claim_lease_expires_at: 1_787_671_100,
        expires_at: 1_787_671_200,
    }
}

fn refusal_row(database: &Path, request_id: &str) -> (String, String, i64) {
    Connection::open(database)
        .unwrap()
        .query_row(
            "SELECT operation, reason, COUNT(*)
             FROM guichet_refusal_receptions
             WHERE request_id = ?1
             GROUP BY operation, reason",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap()
}

#[test]
fn suite_aucune_et_uuid_connu_non_classe_sont_refuses_et_persistes_exactement() {
    let root = root("uuid-known");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let objective_id = seed_objective(&mut store);
    let claim = claim(
        "request-contradiction-uuid",
        &format!("ce lot est la suite de {objective_id}"),
    );

    let first = process_guichet_claim(&mut store, &claim, "response-026", 1_787_671_010)
        .expect("la contradiction attribuable doit produire un reçu terminal");
    let replay = process_guichet_claim(&mut store, &claim, "response-ignored", 1_787_671_011)
        .expect("le même claim doit relire le reçu durable");

    assert!(!first.replayed);
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);
    let reply: Value = serde_json::from_slice(&first.reply_bytes).unwrap();
    assert_eq!(reply["outcome"], "refused");
    assert_eq!(reply["payload"]["operation"], "delegate");
    assert_eq!(
        reply["payload"]["reason"],
        "suite_none_with_unclassified_citation"
    );
    let reply_text = std::str::from_utf8(&first.reply_bytes).unwrap();
    assert!(matches!(
        decode::<WrapperToDaemon>(reply_text).unwrap(),
        WrapperToDaemon::GuichetReply {
            outcome: GuichetOutcome::Refused,
            payload: GuichetReplyPayload::Refused {
                operation: ServiceRequestOperation::Delegate,
                reason: GuichetRefusalReason::SuiteNoneWithUnclassifiedCitation,
            },
            ..
        }
    ));
    assert_eq!(
        refusal_row(&database, "request-contradiction-uuid"),
        (
            "delegate".to_string(),
            "suite_none_with_unclassified_citation".to_string(),
            1,
        )
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn branche_ou_sha_seuls_ne_sont_pas_declares_comme_contradiction() {
    let root = root("branch-sha-limit");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let claim = claim(
        "request-branch-sha-limit",
        "dépend de session-021-verdict-sha-mesure à 2623772",
    );

    let result = process_guichet_claim(&mut store, &claim, "response-026-limit", 1_787_671_010)
        .expect("la limite déterministe doit rester un refus d'opération, pas une contradiction");
    let reply: Value = serde_json::from_slice(&result.reply_bytes).unwrap();
    assert_eq!(reply["outcome"], "refused");
    assert_eq!(reply["payload"]["operation"], "delegate");
    assert_eq!(reply["payload"]["reason"], "operation_not_available");
    assert_eq!(
        refusal_row(&database, "request-branch-sha-limit"),
        (
            "delegate".to_string(),
            "operation_not_available".to_string(),
            1,
        )
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn vocabulaires_rust_font_un_round_trip_exact_et_exhaustif() {
    for &operation in OperationGuichet::ALL {
        assert_eq!(
            OperationGuichet::parse_sql(operation.as_sql()),
            Some(operation)
        );
    }
    for &reason in MotifRefusGreffe::ALL {
        assert_eq!(MotifRefusGreffe::parse_sql(reason.as_sql()), Some(reason));
    }
}

#[test]
fn valeur_sql_inconnue_est_refusee_a_la_premiere_lecture() {
    let root = root("unknown-sql");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let objective_id = seed_objective(&mut store);
    let claim = claim(
        "request-corrupt-operation",
        &format!("ce lot est la suite de {objective_id}"),
    );
    process_guichet_claim(&mut store, &claim, "response-corrupt", 1_787_671_010).unwrap();
    drop(store);

    let connection = Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE guichet_refusal_receptions SET operation='future_operation'
             WHERE request_id='request-corrupt-operation'",
            [],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    let error = process_guichet_claim(&mut store, &claim, "ignored", 1_787_671_011).unwrap_err();
    assert_eq!(
        error,
        GuichetError::Store("store corrompu : opération guichet inconnue".to_string())
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
