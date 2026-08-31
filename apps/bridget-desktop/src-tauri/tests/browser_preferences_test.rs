use bridget_desktop::preferences_store::{
    BrowserPanelStateV1, DesktopPreferences, PreferencesStore,
};
use uuid::Uuid;

#[test]
fn browser_state_est_local_borne_et_migre_depuis_v4() {
    let root = std::env::temp_dir().join(format!("bridget-browser-preferences-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("preferences.json");
    std::fs::write(&path, br#"{ "version": 4, "preferences": { "display_name": "", "color_scheme": "system", "timezone": "system", "font_size_px": 16 } }"#).unwrap();
    let store = PreferencesStore::new(&path);
    let migrated = store.load().unwrap();
    assert!(!migrated.browser_panel.right_panel_visible);
    assert!(!migrated.browser_panel.external_content_fetch_enabled);
    assert_eq!(migrated.browser_panel.browser_session_mode, "persistent");

    let invalid = DesktopPreferences {
        browser_panel: BrowserPanelStateV1 {
            active_tab: "cookies".to_owned(),
            ..BrowserPanelStateV1::default()
        },
        ..DesktopPreferences::default()
    };
    assert!(store.save(invalid).is_err());
    let _ = std::fs::remove_dir_all(root);
}
