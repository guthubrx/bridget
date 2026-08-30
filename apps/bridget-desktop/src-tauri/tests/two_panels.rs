use bridget_desktop::panels::{MAXIMUM_OPEN_PANELS, PanelRegistry};

#[test]
fn deux_panneaux_restent_isoles_lorsque_le_premier_est_ferme() {
    let mut panels = PanelRegistry::default();
    let first = panels
        .open("alpha", "http://127.0.0.1:39101/?token=fixture-alpha")
        .expect("premier panneau");
    let second = panels
        .open("beta", "http://127.0.0.1:39102/?token=fixture-beta")
        .expect("second panneau");
    assert_eq!(panels.panels().count(), MAXIMUM_OPEN_PANELS);
    assert_eq!(
        panels.close(&first.label).expect("fermeture").profile_id,
        "alpha"
    );
    let remaining = panels.panels().collect::<Vec<_>>();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].label, second.label);
    assert_eq!(remaining[0].profile_id, "beta");
}
