const BEFORE: &str = include_str!("fixtures/prompts/v1-before.txt");
const AFTER: &str = include_str!("fixtures/prompts/v1-after.txt");

#[test]
fn prompt_mcp_v1_reduit_au_moins_soixante_pourcent_des_scalaires_unicode() {
    let before = BEFORE.chars().count();
    let after = AFTER.chars().count();

    assert!(
        after * 100 <= before * 40,
        "réduction insuffisante: avant={before}, après={after}"
    );
}

#[test]
fn prompt_mcp_v1_ne_conserve_que_la_semantique_des_messages_entrants() {
    assert!(AFTER.contains('💬'));
    assert!(AFTER.contains("reply=yes"));
    assert!(AFTER.contains("reply=no"));
    assert!(!AFTER.contains("bridget send"));
    assert!(!AFTER.contains("bridget who"));
    assert!(!AFTER.contains("Règles ABSOLUES"));
    assert!(!AFTER.contains("Bridget ready"));
}
