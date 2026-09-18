//! Le noyau compile sans le composant métier et sans ses types, même en test.

#[test]
fn le_noyau_n_a_aucune_dependance_service_compagnon_meme_en_test() {
    for manifest in [
        include_str!("../Cargo.toml"),
        include_str!("../../../Cargo.toml"),
    ] {
        assert!(
            !manifest
                .lines()
                .any(|line| line.trim_start().starts_with("service ="))
        );
        assert!(!manifest.contains("\"plugins/service-compagnon\""));
    }
    // Examiner aussi les fixtures : retirer seulement la dépendance production
    // ne ferme pas la frontière si un test importe encore le magasin métier.
    for (name, source) in [
        ("wrapper", include_str!("../src/wrapper.rs")),
        ("daemon", include_str!("../src/daemon.rs")),
        ("migration", include_str!("../src/identity_migration.rs")),
        ("reprise", include_str!("../src/reprise.rs")),
        ("migration test", include_str!("identity_migration_test.rs")),
    ] {
        assert!(
            !source.contains("service_compagnon::"),
            "{name}: import privé"
        );
        assert!(!source.contains("GuichetStore"), "{name}: magasin privé");
    }
}

#[test]
fn la_reprise_ne_consulte_ni_ne_lance_la_coordination() {
    let wrapper = include_str!("../src/wrapper.rs");
    let reprise = include_str!("../src/reprise.rs");
    for source in [wrapper, reprise] {
        for seam in [
            "managed_resume_mission",
            "collect_service_compagnon",
            ".config/service-compagnon",
            "ResumeStance",
        ] {
            assert!(!source.contains(seam), "couture métier implicite: {seam}");
        }
    }
}
