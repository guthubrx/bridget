use maicie::config::ProfileConfig;
use maicie::profiles::{ProfileError, ResolvedAgentDefinition, approval_view, load_profiles};
use serde::Deserialize;

#[derive(Deserialize)]
struct ProfileFixture {
    profile: ProfileConfig,
    resolved_definition: ResolvedAgentDefinition,
}

fn profile() -> ProfileConfig {
    fixture().profile
}

fn definition() -> ResolvedAgentDefinition {
    fixture().resolved_definition
}

fn fixture() -> ProfileFixture {
    serde_json::from_str(include_str!("../fixtures/profiles/claude-review.json"))
        .expect("fixture profil valide")
}

fn codex_fixture() -> ProfileFixture {
    serde_json::from_str(include_str!("../fixtures/profiles/codex-code.json"))
        .expect("fixture profil codex valide")
}

#[test]
fn charge_un_profil_avec_tags_exacts_et_reference_spawn_obligatoire() {
    let profiles = load_profiles(&[profile()]).unwrap();

    assert_eq!(profiles[0].tags, ["review", "security"]);
    assert_eq!(profiles[0].spawn_order_ref, "agents/claude-review");
    assert_eq!(profiles[0].agent_type, "claude");
    assert_eq!(profiles[0].model, "claude-fable-5");
    assert_eq!(profiles[0].effort, "raisonnement-renforce");
}

#[test]
fn refuse_un_profil_sans_champ_de_gouvernance_ou_spawn_order() {
    for field in ["agent_type", "model", "effort"] {
        let mut invalid = profile();
        match field {
            "agent_type" => invalid.agent_type = None,
            "model" => invalid.model = None,
            "effort" => invalid.effort = None,
            _ => unreachable!(),
        }
        assert!(matches!(
            load_profiles(&[invalid]),
            Err(ProfileError::MissingField { field: actual, .. }) if actual == field
        ));
    }
    let mut invalid = profile();
    invalid.spawn_order_ref.clear();
    assert!(matches!(
        load_profiles(&[invalid]),
        Err(ProfileError::MissingField {
            field: "spawn_order_ref",
            ..
        })
    ));
}

#[test]
fn ecran_d_approbation_conserve_args_verbatim_et_effort_opaque() {
    let loaded = load_profiles(&[profile()]).unwrap().remove(0);
    let resolved = definition();
    let view = approval_view(loaded, resolved.clone()).unwrap();

    assert_eq!(view.args, resolved.args);
    assert_eq!(view.profile.effort, "raisonnement-renforce");
    assert_eq!(view.forbidden_env, ["ANTHROPIC_API_KEY"]);
    assert_eq!(view.definition_digest, "a".repeat(64));
}

#[test]
fn refuse_une_definition_resolue_sans_garde_de_facturation() {
    let loaded = load_profiles(&[profile()]).unwrap().remove(0);
    let mut unresolved = definition();
    unresolved.forbidden_env.clear();

    assert!(matches!(
        approval_view(loaded, unresolved),
        Err(ProfileError::ResolvedDefinition("forbidden_env vide"))
    ));
}

#[test]
fn chaque_variante_de_fixture_protege_ses_variables_de_facturation() {
    for fixture in [fixture(), codex_fixture()] {
        let loaded = load_profiles(&[fixture.profile]).unwrap().remove(0);
        let view = approval_view(loaded, fixture.resolved_definition).unwrap();
        assert!(
            !view.forbidden_env.is_empty(),
            "chaque variante doit déclarer forbidden_env"
        );
    }
}
