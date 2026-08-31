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

#[test]
fn le_panneau_relais_ne_recoit_que_l_autorisation_native_de_notification() {
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../capabilities/relay-notification-permission.json"
    ))
    .expect("JSON capability relais");

    assert_eq!(value["webviews"], serde_json::json!(["panel-*"]));
    assert_eq!(
        value["remote"]["urls"],
        serde_json::json!(["http://127.0.0.1:*"])
    );
    assert_eq!(
        value["permissions"],
        serde_json::json!([
            "notification:allow-is-permission-granted",
            "notification:allow-request-permission"
        ])
    );
    assert!(!value.to_string().contains("core:"));
    assert!(!value.to_string().contains("allow-notify"));
}
