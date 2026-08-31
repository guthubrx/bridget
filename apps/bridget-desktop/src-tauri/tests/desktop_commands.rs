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

#[test]
fn les_reglages_de_contenu_restent_locaux_et_sans_capability_de_panneau() {
    let frontend = include_str!("../../ui/app.js");
    let markup = include_str!("../../ui/index.html");
    let backend = include_str!("../src/lib.rs");

    for identifier in [
        "preferences-external-links",
        "preferences-file-references",
        "preferences-remote-images",
    ] {
        assert!(markup.contains(identifier), "contrôle local absent : {identifier}");
    }
    assert!(frontend.contains("content_security"));
    assert!(backend.contains("__BRIDGET_CONTENT_SECURITY__"));
    assert!(backend.contains("bridget-open"));
    assert!(backend.contains("approved_external_https_url"));
    assert!(backend.contains("window.location.reload()"));
    assert!(!backend.contains("bridget-content-security-updated"));
    assert!(!backend.contains("panel-content-security"));
}

#[test]
fn l_icone_bundled_conserve_le_fond_sombre_et_la_mascotte_agrandie() {
    let source = include_str!("../icons/bridget.svg");
    let icon = include_str!("../icons/icon.svg");

    for asset in [source, icon] {
        assert!(asset.contains("bridget-dark-background"));
        assert!(asset.contains("scale(1.18)"));
    }
    assert!(!include_bytes!("../icons/icon.icns").is_empty());
}
