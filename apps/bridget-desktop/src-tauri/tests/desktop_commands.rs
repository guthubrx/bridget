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
    let frontend = include_str!("../../ui/fleet-app.js");
    assert!(frontend.contains("invoke(\"fleet_snapshot\")"));
    assert!(frontend.contains("void openAgent(agent)"));
    assert!(frontend.contains("void openProjectAction(source.source_id, pendingProjectAction)"));
    assert!(frontend.contains("await invoke(\"connection_close\""));
    assert!(frontend.contains("await connectProfile(profile)"));
    assert!(frontend.contains("source_id: agent.source_id"));
    assert!(frontend.contains("desktop_action: action"));
}

#[test]
fn un_seul_serveur_est_affiche_a_la_fois() {
    let frontend = include_str!("../../ui/fleet-app.js");
    let backend = include_str!("../src/lib.rs");
    assert!(frontend.contains("source_id: agent.source_id"));
    assert!(backend.contains("close_open_panels(&app, &state)?;"));
    assert!(backend.contains("const DESKTOP_SHELL_WIDTH: u32 = 520;"));
    assert!(backend.contains("PhysicalSize::new(panel_width, size.height.max(1))"));
}

#[test]
fn flotte_desktop_ne_restitue_ni_secret_ni_chemin_d_origine() {
    let backend = include_str!("../src/lib.rs");
    let fleet = include_str!("../src/fleet.rs");
    assert!(backend.contains("fn fleet_snapshot"));
    assert!(backend.contains("connected_remote_targets(&profiles, &sessions)"));
    assert!(backend.contains("if local.source.error.is_none()"));
    assert!(fleet.contains("pub struct DesktopFleetSnapshotV1"));
    assert!(fleet.contains("canonical_path_n_est_pas_un_champ_de_sortie"));
    assert!(!fleet.contains("canonical_path: String"));
}

#[test]
fn les_reglages_de_contenu_restent_locaux_et_sans_capability_de_panneau() {
    let frontend = include_str!("../../ui/fleet-app.js");
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
