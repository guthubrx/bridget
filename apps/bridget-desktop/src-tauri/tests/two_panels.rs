use bridget_desktop::panels::{MAXIMUM_OPEN_PANELS, PanelRegistry};

#[test]
fn un_seul_panneau_est_conserve_dans_le_registre() {
    let mut panels = PanelRegistry::default();
    let first = panels
        .open("alpha", "http://127.0.0.1:39101/?token=fixture-alpha")
        .expect("premier panneau");
    assert!(
        panels
            .open("beta", "http://127.0.0.1:39102/?token=fixture-beta")
            .is_err()
    );
    assert_eq!(panels.panels().count(), MAXIMUM_OPEN_PANELS);
    assert_eq!(
        panels.close(&first.label).expect("fermeture").profile_id,
        "alpha"
    );
    assert_eq!(panels.panels().count(), 0);
}
