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
