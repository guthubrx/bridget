use bridget_desktop::panels::{PanelError, PanelRegistry, is_browser_target};

#[test]
fn navigation_browser_reste_https_ou_publication_locale_et_le_panneau_est_unique() {
    assert!(is_browser_target("https://example.org/rapport"));
    assert!(is_browser_target(
        "http://127.0.0.1:17888/v1/artifacts/detail"
    ));
    assert!(is_browser_target("bridget://browser-home"));
    for target in [
        "file:///tmp/no",
        "http://example.org",
        "javascript:alert(1)",
    ] {
        assert!(!is_browser_target(target), "{target}");
    }
    let mut registry = PanelRegistry::default();
    registry.open_browser("https://example.org/a").unwrap();
    registry.open_browser("https://example.org/b").unwrap();
    assert_eq!(registry.browser().unwrap().target, "https://example.org/b");
    assert!(matches!(
        registry.open_browser("file:///tmp/no"),
        Err(PanelError::InvalidBrowserUrl)
    ));
}

#[test]
fn surface_browser_existante_est_rechargee_vers_le_mode_browser_bridget() {
    let backend = include_str!("../src/lib.rs");
    let relay = include_str!("../../../../crates/bridget-daemon/src/ui.rs");

    assert!(backend.contains("if let Some(webview) = app.get_webview(&browser.label)"));
    assert!(backend.contains("webview.navigate(url).map_err(as_message)?;"));
    assert!(backend.contains("jamais conserver une seconde conversation dans le volet"));
    assert!(backend.contains("url.set_path(\"/browser-panel\");"));
    assert!(backend.contains("window.__BRIDGET_BROWSER_PANEL__ = true;"));
    assert!(backend.contains("open_browser_home_surface(&app, &state)?;"));
    assert!(relay.contains("(\"GET\", \"/browser-panel\")"));
}
