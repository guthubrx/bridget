//! Corpus de caractérisation : aucun processus, socket ou environnement utilisateur.
//! Les entrées décrivent les cas ; les sorties sont figées avec le codec de référence.
use bridget_transport::protocol::{DaemonToWrapper, WrapperToDaemon, decode, encode};
use serde::Deserialize;
use serde_json::Value;

const INPUTS: &str = include_str!("../../../fixtures/wire-inputs.jsonl");
const GOLDEN: &str = include_str!("../../../fixtures/wire-reference.jsonl");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    family: String,
    direction: String,
    input: Value,
}

fn encode_case(case: &Case) -> String {
    let input = case.input.to_string();
    match case.direction.as_str() {
        "to_daemon" => encode(&decode::<WrapperToDaemon>(&input).expect(&input)).unwrap(),
        "from_daemon" => encode(&decode::<DaemonToWrapper>(&input).expect(&input)).unwrap(),
        other => panic!("direction inconnue : {other}"),
    }
}

#[test]
fn core_089_wire_matches_reference_bytes() {
    let cases: Vec<Case> = INPUTS
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let golden: Vec<Value> = GOLDEN
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        cases.len(),
        golden.len(),
        "aucun scénario ne peut disparaître"
    );
    assert!(
        cases.len() >= 30,
        "corpus complet, pas seulement le handshake"
    );
    for (case, expected) in cases.iter().zip(&golden) {
        assert_eq!(case.family, expected["family"]);
        assert_eq!(case.direction, expected["direction"]);
        let actual = encode_case(case);
        assert_eq!(
            actual.as_bytes(),
            expected["wire"].as_str().unwrap().as_bytes(),
            "{} : {}",
            case.family,
            case.input
        );
        // Le consommateur décode les octets FIGÉS, pas seulement sa propre émission.
        let wire = expected["wire"].as_str().unwrap();
        let reread = match case.direction.as_str() {
            "to_daemon" => encode(&decode::<WrapperToDaemon>(wire).unwrap()).unwrap(),
            "from_daemon" => encode(&decode::<DaemonToWrapper>(wire).unwrap()).unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(reread.as_bytes(), wire.as_bytes());
    }
}

/// Recette d'origine seulement : ne met jamais à jour les golden automatiquement.
/// Exécuter sur le codec dfa2134 puis archiver la sortie dans Git avant extraction.
#[test]
#[ignore = "matérialisation explicite de la référence, jamais une validation"]
fn core_089_materialize_reference() {
    for line in INPUTS.lines() {
        let case: Case = serde_json::from_str(line).unwrap();
        let wire = encode_case(&case);
        println!(
            "CAPTURE{}",
            serde_json::json!({"family": case.family, "direction": case.direction, "wire": wire})
        );
    }
}
