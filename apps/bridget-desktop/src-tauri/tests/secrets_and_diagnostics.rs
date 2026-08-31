use bridget_desktop::connection::{ConnectionError, relay_url};
use bridget_desktop::profile::{
    ConnectionProfile, ProfileCapability, RelayEndpoint, SshIdentityRef,
};

#[test]
fn profils_et_diagnostics_ne_divulguent_ni_cle_ni_jeton() {
    let profile = ConnectionProfile::Ssh {
        id: "redacted".into(),
        label: "Serveur sûr".into(),
        host: "relay.example.test".into(),
        port: 2222,
        user: "operator".into(),
        identity: SshIdentityRef::File {
            path: "/Users/operator/.ssh/id_ed25519".into(),
        },
        host_fingerprint: Some("SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
        capabilities: vec![ProfileCapability::Ui],
    };
    let endpoint = RelayEndpoint::new(17888, "test-relay-token".into()).expect("endpoint");
    let stored = serde_json::to_string(&profile).expect("profil JSON");
    let diagnostic = format!("{:?} {}", endpoint, ConnectionError::RelayUnavailable);
    for forbidden in [
        "PRIVATE KEY",
        "BEGIN OPENSSH",
        "test-relay-token",
        "?token=",
    ] {
        assert!(!stored.contains(forbidden), "profil: {forbidden}");
        assert!(!diagnostic.contains(forbidden), "diagnostic: {forbidden}");
    }
    let relay_only_in_memory = relay_url(39174, &endpoint);
    assert!(relay_only_in_memory.contains("test-relay-token"));
}

#[test]
fn les_preferences_de_contenu_ne_transportent_ni_jeton_ni_url_de_tunnel() {
    let source = include_str!("../src/preferences_store.rs");
    for forbidden in ["token", "relay_url", "tunnel_url", "private_key"] {
        assert!(
            !source.contains(forbidden),
            "une préférence de contenu ne doit pas contenir : {forbidden}"
        );
    }
    assert!(source.contains("external_links"));
    assert!(source.contains("file_references"));
    assert!(source.contains("remote_images"));
}
