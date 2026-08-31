use bridget_desktop::profile::ProfileDraft;

#[test]
fn le_frontend_ne_peut_pas_fournir_de_commande_url_ou_secret_libre() {
    for payload in [
        r#"{"kind":"ssh","label":"X","host":"example.test","port":22,"user":"moi","identity":{"source":"agent"},"command":"sh -c whoami"}"#,
        r#"{"kind":"ssh","label":"X","host":"example.test","port":22,"user":"moi","identity":{"source":"agent"},"relay_url":"http://127.0.0.1/?token=secret"}"#,
        r#"{"kind":"ssh","label":"X","host":"example.test","port":22,"user":"moi","identity":{"source":"agent"},"token":"secret"}"#,
    ] {
        assert!(
            serde_json::from_str::<ProfileDraft>(payload).is_err(),
            "{payload}"
        );
    }
}

#[test]
fn les_actions_de_carte_sont_delegatees_au_listing_de_profils() {
    let frontend = include_str!("../../ui/app.js");
    assert!(frontend.contains("elements.list.addEventListener(\"click\""));
    for action in [
        "edit",
        "delete",
        "connect",
        "open",
        "disconnect",
    ] {
        assert!(
            frontend.contains(&format!("button.dataset.action === \"{action}\"")),
            "action de carte absente : {action}"
        );
    }
    assert!(frontend.contains("void connectProfile(profile)"));
    assert!(!frontend.contains("data-action=\"settings\""));
    assert!(!frontend.contains("openServerSettings"));
}

#[test]
fn un_seul_serveur_est_affiche_a_la_fois() {
    let frontend = include_str!("../../ui/app.js");
    let backend = include_str!("../src/lib.rs");
    assert!(
        frontend.contains("openPanelProfiles.clear(); openPanelProfiles.add(panel.profile_id)")
    );
    assert!(backend.contains("close_open_panels(&app, &state)?;"));
    assert!(backend.contains("PhysicalSize::new(size.width.max(1), size.height.max(1))"));
}
