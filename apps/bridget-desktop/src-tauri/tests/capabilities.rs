#[test]
fn seule_la_coque_locale_recoit_une_capability() {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("../capabilities/main.json")).expect("JSON capability");
    assert_eq!(value["webviews"], serde_json::json!(["main"]));
    let permissions = value["permissions"].as_array().expect("permissions");
    assert!(permissions.iter().any(|value| value == "core:default"));
    assert!(!value.to_string().contains("panel-"));
    assert!(!value.to_string().contains("http://"));
}
