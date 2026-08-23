use maicie::{
    domain::{EtatFlux, SourceSnapshot},
    telemetry::{write_event, TelemetryEvent, TelemetryKind},
};
use serde_json::json;
use uuid::Uuid;

fn event() -> TelemetryEvent {
    TelemetryEvent {
        observed_at: 1_787_464_900,
        objective_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        delegation_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
        message_id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
        source: SourceSnapshot::AcpSubscription,
        freshness: EtatFlux::Gap,
        event: TelemetryKind::Correlation,
    }
}

#[test]
fn journalise_une_ligne_structuree_et_correlee() {
    let mut output = Vec::new();
    write_event(&mut output, &event()).unwrap();

    let line = String::from_utf8(output).unwrap();
    assert_eq!(
        line,
        concat!(
            r#"{"observed_at":1787464900,"objective_id":"11111111-1111-1111-1111-111111111111","delegation_id":"22222222-2222-2222-2222-222222222222","message_id":"33333333-3333-3333-3333-333333333333","source":"acp_subscription","freshness":"gap","event":"correlation"}"#,
            "\n",
        )
    );

    let decoded: serde_json::Value = serde_json::from_str(line.trim_end()).unwrap();
    assert_eq!(
        decoded,
        json!({
            "observed_at": 1_787_464_900,
            "objective_id": "11111111-1111-1111-1111-111111111111",
            "delegation_id": "22222222-2222-2222-2222-222222222222",
            "message_id": "33333333-3333-3333-3333-333333333333",
            "source": "acp_subscription",
            "freshness": "gap",
            "event": "correlation",
        })
    );
}

#[test]
fn refuse_un_corps_non_prevu_par_le_schema_de_telemetrie() {
    let mut value = serde_json::to_value(event()).unwrap();
    value["body"] = json!("contenu qui ne doit pas etre journalise");

    assert!(serde_json::from_value::<TelemetryEvent>(value).is_err());
}
