//! Garde de frontière : Bridget consomme uniquement le contrat JSON public.

#[test]
fn le_daemon_n_a_pas_de_dependance_maicie_en_production() {
    let manifest = include_str!("../Cargo.toml");
    let production_dependencies = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("manifest sans section dev");
    assert!(
        !production_dependencies
            .lines()
            .any(|line| line.trim_start().starts_with("maicie =")),
        "Maicie ne doit être disponible que pour les fixtures de test"
    );

    for (name, source) in [
        ("ui", include_str!("../src/ui.rs")),
        ("wrapper", include_str!("../src/wrapper.rs")),
        ("daemon", include_str!("../src/daemon.rs")),
        ("projection", include_str!("../src/mission_projection.rs")),
    ] {
        let productive = source.split("\n#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !productive.contains("maicie::"),
            "{name} ne doit pas importer les types privés Maicie"
        );
        assert!(
            !productive.contains("MaicieStore"),
            "{name} ne doit pas ouvrir le magasin privé Maicie"
        );
    }
}
