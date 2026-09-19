//! Recette explicite : vrai Codex, vrai daemon, fournisseur HTTP privé.
//! Aucun secret, abonnement ni daemon utilisateur n'est utilisé.
#[test]
#[ignore = "exige le CLI Codex installé ; HTTP synthétique, aucune inférence facturée"]
fn spec091_selection_native_conserve_fil_historique_et_configuration() {
    let codex = std::env::var("BRIDGET_TEST_CODEX_BIN")
        .unwrap_or_else(|_| "/opt/homebrew/bin/codex".into());
    let result = std::process::Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/runtime_selection_091.py"
        ))
        .arg(env!("CARGO_BIN_EXE_bridget"))
        .arg(codex)
        .output()
        .expect("recette native 091");
    assert!(
        result.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"result\": \"PASS\""));
}
