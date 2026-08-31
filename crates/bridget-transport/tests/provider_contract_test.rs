use bridget_transport::protocol::{ProviderObservation, ProviderOperation};
use serde_json::Value;

const CODEX: &str = include_str!("fixtures/provider-contracts/codex-0.150.1.jsonl");
const CLAUDE: &str = include_str!("fixtures/provider-contracts/claude-2.1.221.jsonl");
const ACP: &str = include_str!("fixtures/provider-contracts/acp-v1.jsonl");

fn frames(fixture: &str) -> Vec<Value> {
    fixture
        .lines()
        .map(|line| serde_json::from_str(line).expect("fixture JSONL canonique"))
        .collect()
}

fn has_method(frames: &[Value], method: &str) -> bool {
    frames.iter().any(|frame| {
        frame
            .get("method")
            .and_then(Value::as_str)
            .is_some_and(|actual| actual == method)
    })
}

#[test]
fn contrats_codex_claude_et_cursor_acp_attestent_leur_interruption_sans_inference() {
    let codex = frames(CODEX);
    let claude = frames(CLAUDE);
    let cursor_acp = frames(ACP);

    assert!(has_method(&codex, "turn/interrupt"));
    assert!(claude.iter().any(|frame| {
        frame.pointer("/type").and_then(Value::as_str) == Some("control_request")
            && frame.pointer("/request/subtype").and_then(Value::as_str) == Some("interrupt")
    }));
    assert!(has_method(&cursor_acp, "session/cancel"));

    for (provider, operations) in [
        (
            "codex",
            vec![ProviderOperation::Interrupt, ProviderOperation::Steer],
        ),
        ("claude", vec![ProviderOperation::Interrupt]),
        ("cursor", vec![ProviderOperation::Interrupt]),
    ] {
        let observation = ProviderObservation {
            binary_path: format!("/fixtures/{provider}"),
            binary_version: "fixture".to_string(),
            binary_digest: "0".repeat(64),
            contract_version: "fixture-v1".to_string(),
            operations,
        };
        assert!(observation.supports(ProviderOperation::Interrupt));
        assert!(!observation.supports(ProviderOperation::Resume));
    }
}

#[test]
fn version_inconnue_capacite_absente_et_evenement_futur_ne_deviennent_pas_supportes() {
    let unknown = ProviderObservation {
        binary_path: "/fixtures/inconnu".to_string(),
        binary_version: "inconnue".to_string(),
        binary_digest: "0".repeat(64),
        contract_version: "inconnu".to_string(),
        operations: Vec::new(),
    };
    assert!(!unknown.supports(ProviderOperation::Interrupt));
    assert!(!unknown.supports(ProviderOperation::Fork));

    let future: Value = serde_json::from_str(
        r#"{"jsonrpc":"2.0","method":"vendor/future","params":{"opaque":true}}"#,
    )
    .unwrap();
    assert_eq!(future["method"], "vendor/future");
    assert!(future.get("result").is_none());
}
