#[test]
fn les_surfaces_non_fiables_n_exposent_ni_ipc_ni_secret() {
    let browser_capability = include_str!("../capabilities/browser-panel.json");
    let frame_capability = include_str!("../capabilities/artifact-frame.json");
    let host = include_str!("../../../../crates/bridget-daemon/assets/ui/artifact-sandbox-host.js");
    for capability in [browser_capability, frame_capability] {
        assert!(capability.contains("\"permissions\": []"));
        assert!(!capability.contains("core:"));
        assert!(!capability.contains("shell:"));
    }
    assert!(host.contains("sandbox\", \"allow-scripts\""));
    assert!(host.contains("connect-src 'none'"));
    assert!(!host.contains("__TAURI__"));
    assert!(!host.contains("invoke("));
    assert!(!host.contains("token"));
}
