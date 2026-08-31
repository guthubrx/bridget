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
fn le_volet_agents_reste_la_surface_distante_autoritaire() {
    let frontend = include_str!("../../ui/fleet-app.js");
    let connection = include_str!("../src/connection.rs");
    let remote_ui = include_str!("../../../../crates/bridget-daemon/assets/ui/app.js");
    assert!(frontend.contains("invoke(\"fleet_snapshot\")"));
    assert!(frontend.contains("async function openSource(sourceId, projectId = null)"));
    assert!(frontend.contains("project_id: projectId"));
    assert!(frontend.contains("void openPanelAction(source.source_id, pendingPanelAction)"));
    assert!(frontend.contains("await invoke(\"connection_close\""));
    assert!(frontend.contains("await connectProfile(profile)"));
    assert!(frontend.contains("desktop_action: action"));
    assert!(!connection.contains("agent_menu"));
    assert!(remote_ui.contains("const renderAgentButton = (agent) =>"));
    assert!(remote_ui.contains("shell.addEventListener(\"contextmenu\""));
}

#[test]
fn un_seul_serveur_est_affiche_a_la_fois() {
    let frontend = include_str!("../../ui/fleet-app.js");
    let backend = include_str!("../src/lib.rs");
    assert!(frontend.contains("source_id: sourceId"));
    assert!(backend.contains("close_open_panels(&app, &state)?;"));
    assert!(backend.contains("const DESKTOP_SHELL_WIDTH: u32 = 200;"));
    assert!(backend.contains("PhysicalSize::new(panel_width, size.height.max(1))"));
}

#[test]
fn la_fenetre_principale_est_rendue_visible_au_demarrage() {
    let backend = include_str!("../src/lib.rs");

    assert!(backend.contains("let main = main_window(&handle)?;"));
    assert!(backend.contains("main.show().map_err(as_message)?;"));
    assert!(backend.contains("main.set_focus().map_err(as_message)?;"));
}

#[test]
fn la_coque_desktop_ne_garde_que_le_filtre_sources() {
    let markup = include_str!("../../ui/index.html");
    let stylesheet = include_str!("../../ui/fleet-desktop.css");

    assert!(markup.contains("<h1 id=\"fleet-title\">Projets</h1>"));
    assert!(stylesheet.contains("grid-template-columns: 12.5rem;"));
    assert!(stylesheet.contains("width: min(100vw, 12.5rem);"));
    assert!(stylesheet.contains("#fleet-shell .fleet-sources-pane button"));
    assert!(!markup.contains("fleet-agents-pane"));
    assert!(stylesheet.contains("background: var(--fleet-surface-hover);"));
}

#[test]
fn le_menu_desktop_restitue_le_centre_de_controle_existant_sans_le_remplacer() {
    let markup = include_str!("../../ui/index.html");
    let frontend = include_str!("../../ui/fleet-app.js");
    let stylesheet = include_str!("../../ui/fleet-desktop.css");

    assert!(markup.contains("id=\"settings-launcher\""));
    assert!(markup.contains("data-open-remote-view=\"settings\""));
    assert!(markup.contains("data-open-remote-view=\"usage\""));
    assert!(markup.contains("id=\"settings-dialog\""));
    assert!(markup.contains("data-settings-section=\"servers\""));
    assert!(!markup.contains("id=\"preferences-dialog\""));
    assert!(!markup.contains("id=\"server-dialog\""));
    assert!(frontend.contains("void beginPanelAction(button.dataset.openRemoteView)"));
    assert!(frontend.contains("openSettings(\"servers\")"));
    assert!(stylesheet.contains(".settings-dialog"));
    assert!(stylesheet.contains(".settings-layout"));
    assert!(stylesheet.contains("#settings-dialog .settings-sidebar button"));
    assert!(stylesheet.contains("left: 0;"));
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
        assert!(
            markup.contains(identifier),
            "contrôle local absent : {identifier}"
        );
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
