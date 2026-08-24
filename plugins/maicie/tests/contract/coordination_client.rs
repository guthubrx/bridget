use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Copie consommateur du corpus 015 gelé : elle reste en octets afin que le
/// futur client 016 ne puisse pas lui substituer un DTO local avant G-1601.
const UPSTREAM_015_FRAMES: &[u8] = include_bytes!(
    "../../../../specs/016-coordination-active/contracts/fixtures/service-frames-v1.jsonl"
);
const CONSUMER_015_FRAMES: &[u8] =
    include_bytes!("../fixtures/coordination-active/service-frames-v1.jsonl");
const OBSERVATION_ORACLES: &str =
    include_str!("../fixtures/coordination-active/observation-oracles-v1.json");
const UPSTREAM_015_SHA256: &str =
    "42e0ea32690e267221804a8af9aee538162f7c141a9cffacf6d6a44a9a8cda1f";

#[test]
fn relit_le_corpus_015_octet_pour_octet_sans_dto_016_local() {
    assert_eq!(CONSUMER_015_FRAMES, UPSTREAM_015_FRAMES);
    assert!(CONSUMER_015_FRAMES.ends_with(b"\n"));
    assert_eq!(hex_sha256(CONSUMER_015_FRAMES), UPSTREAM_015_SHA256);

    // Mutation discriminante : modifier un seul octet rendrait l'oracle rouge,
    // même si le JSON restait syntaxiquement valide.
    let mut mutated = CONSUMER_015_FRAMES.to_vec();
    mutated[0] ^= 1;
    assert_ne!(mutated, UPSTREAM_015_FRAMES);
}

#[test]
fn le_corpus_015_expose_les_depots_releve_et_terminaux_attestes() {
    let frames = parse_jsonl(CONSUMER_015_FRAMES);
    assert_eq!(frames.len(), 7, "les sept trames normatives sont requises");

    assert_eq!(frames[0], delivery_report_frame());
    assert_eq!(frames[1], mission_status_frame());
    assert_eq!(frames[2], deadline_question_frame());
    assert_eq!(frames[3], json!({"type":"guichet_claim_next","v":1}));
    assert_eq!(frames[4], lifecycle_frame("evt-answered", "answered", 1));
    assert_eq!(frames[5], lifecycle_frame("evt-cancelled", "cancelled", 2));
    assert_eq!(frames[6], lifecycle_frame("evt-timeout", "timed_out", 3));
}

#[test]
fn oracles_de_fraicheur_restent_explicitement_hors_des_octets_015() {
    let oracle: Value =
        serde_json::from_str(OBSERVATION_ORACLES).expect("oracle de fraîcheur JSON valide");
    assert_eq!(oracle["source"], "oracle_test_016_non_filaire");
    assert_eq!(oracle["upstream_015_bytes"], false);

    let observations = oracle["observations"]
        .as_array()
        .expect("liste d'observations");
    assert_eq!(observations.len(), 3);
    assert_eq!(
        observations[0],
        json!({"kind":"Fresh","expected":"attested"})
    );
    assert_eq!(
        observations[1],
        json!({"kind":"Gap","expected":"unavailable_without_inference"})
    );
    assert_eq!(
        observations[2],
        json!({"kind":"Unavailable","expected":"unavailable_without_inference"})
    );
}

fn parse_jsonl(bytes: &[u8]) -> Vec<Value> {
    std::str::from_utf8(bytes)
        .expect("fixture UTF-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("trame JSON normative"))
        .collect()
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn delivery_report_frame() -> Value {
    json!({
        "type":"service_request",
        "v":1,
        "issuer_scope":"015_scope_0123456789abcdef0123456789abcdef",
        "request_id":"req-delivery",
        "issued_at":1787500000_i64,
        "from":"codex-1",
        "to":"maicie",
        "operation":"delivery_report",
        "payload":{
            "objective_id":"obj-01",
            "delegation_id":"del-01",
            "delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "in_reply_to":"msg-01"
        }
    })
}

fn mission_status_frame() -> Value {
    json!({
        "type":"service_request",
        "v":1,
        "issuer_scope":"015_scope_0123456789abcdef0123456789abcdef",
        "request_id":"req-status",
        "issued_at":1787500000_i64,
        "from":"codex-1",
        "to":"maicie",
        "operation":"mission_status",
        "payload":{"delegation_id":"del-01"}
    })
}

fn deadline_question_frame() -> Value {
    json!({
        "type":"service_request",
        "v":1,
        "issuer_scope":"015_scope_0123456789abcdef0123456789abcdef",
        "request_id":"req-deadline",
        "issued_at":1787500000_i64,
        "from":"codex-1",
        "to":"maicie",
        "operation":"deadline_question",
        "payload":{"delegation_id":"del-01"}
    })
}

fn lifecycle_frame(event_id: &str, state: &str, offset: i64) -> Value {
    let mut frame = json!({
        "type":"request_lifecycle_event",
        "v":1,
        "issuer_scope":"015_scope_0123456789abcdef0123456789abcdef",
        "event_id":event_id,
        "request_id":"req-delivery",
        "state":state,
        "observed_at":1_787_500_000_i64 + offset
    });
    if state == "answered" {
        frame["in_reply_to"] = json!("msg-01");
        frame["response_message_id"] = json!("msg-02");
    }
    frame
}
