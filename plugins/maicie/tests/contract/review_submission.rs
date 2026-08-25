use maicie::review::{
    CriticalityMap, EvidenceKind, F38_FIXED_REGIME, FileChange, RepositorySnapshot,
    ReviewDecisionError, ReviewDeviationState, ReviewDirection, ReviewLotSubmitPayload,
    ReviewRegime, ReviewRegimeSelectPayload, ReviewSubmissionControl, ReviewSubmissionError,
    ReviewSubmissionState, TrackedPath, calculate_criticality, create_review_submission,
    decide_review_regime,
};
use serde_json::json;

const BASE: &str = "1111111111111111111111111111111111111111";
const HEAD: &str = "2222222222222222222222222222222222222222";

fn map_for(path: &str, patch: &str, content: &str) -> CriticalityMap {
    calculate_criticality(&RepositorySnapshot {
        paths: vec![TrackedPath {
            path: path.to_string(),
            line_count: 20,
        }],
        changes: vec![FileChange {
            old_path: Some(path.to_string()),
            new_path: Some(path.to_string()),
            patch: patch.to_string(),
            head_content: Some(content.to_string()),
        }],
        contracts: Vec::new(),
        open_findings: Vec::new(),
    })
    .unwrap()
}

fn ordinary_map() -> CriticalityMap {
    map_for("src/render.rs", "+pub fn render() {}", "pub fn render() {}")
}

fn critical_map() -> CriticalityMap {
    map_for(
        "src/model.rs",
        "+const SCHEMA_VERSION: i64 = 20;",
        "const SCHEMA_VERSION: i64 = 20;",
    )
}

fn submit_payload() -> ReviewLotSubmitPayload {
    ReviewLotSubmitPayload {
        project_id: "bridget".to_string(),
        branch_ref: "refs/remotes/origin/fix/example".to_string(),
        base: BASE.to_string(),
        head: HEAD.to_string(),
    }
}

fn selection(submission_id: &str, retained_regime: ReviewRegime) -> ReviewRegimeSelectPayload {
    ReviewRegimeSelectPayload {
        submission_id: submission_id.to_string(),
        retained_regime,
    }
}

#[test]
fn t2511a_identifiant_soumission_est_deterministe_et_couvre_la_paire_complete() {
    let map = ordinary_map();
    let baseline = create_review_submission(&submit_payload(), "author-1", &map).unwrap();
    let replay = create_review_submission(&submit_payload(), "author-1", &map).unwrap();

    assert_eq!(baseline.submission_id, replay.submission_id);
    assert_eq!(baseline.submission_id.len(), 64);
    assert!(
        baseline
            .submission_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );

    let variants = [
        ReviewLotSubmitPayload {
            project_id: "cartae".to_string(),
            ..submit_payload()
        },
        ReviewLotSubmitPayload {
            branch_ref: "refs/heads/fix/example".to_string(),
            ..submit_payload()
        },
        ReviewLotSubmitPayload {
            base: "3333333333333333333333333333333333333333".to_string(),
            ..submit_payload()
        },
        ReviewLotSubmitPayload {
            head: "4444444444444444444444444444444444444444".to_string(),
            ..submit_payload()
        },
    ];
    for variant in variants {
        let changed = create_review_submission(&variant, "author-1", &map).unwrap();
        assert_ne!(
            baseline.submission_id, changed.submission_id,
            "chaque coordonnée canonique doit participer à l'identifiant"
        );
    }

    let other_author = create_review_submission(&submit_payload(), "author-2", &map).unwrap();
    assert_eq!(
        baseline.submission_id, other_author.submission_id,
        "l'auteur est un fait attesté, pas une coordonnée Git du lot"
    );

    let invalid_cases = [
        (
            ReviewLotSubmitPayload {
                project_id: String::new(),
                ..submit_payload()
            },
            "author-1",
            "project_id",
        ),
        (
            ReviewLotSubmitPayload {
                branch_ref: "fix/example".to_string(),
                ..submit_payload()
            },
            "author-1",
            "branch_ref",
        ),
        (
            ReviewLotSubmitPayload {
                base: "A".repeat(40),
                ..submit_payload()
            },
            "author-1",
            "base",
        ),
        (
            ReviewLotSubmitPayload {
                head: "2".repeat(39),
                ..submit_payload()
            },
            "author-1",
            "head",
        ),
        (submit_payload(), "", "author_id"),
    ];
    for (payload, author_id, field) in invalid_cases {
        assert_eq!(
            create_review_submission(&payload, author_id, &map),
            Err(ReviewSubmissionError::InvalidField(field))
        );
    }
}

#[test]
fn t2511a_soumission_ordinaire_attend_une_decision_sans_valeur_par_defaut() {
    let submission =
        create_review_submission(&submit_payload(), "author-1", &ordinary_map()).unwrap();

    assert_eq!(submission.proposed_regime, ReviewRegime::Simple);
    assert_eq!(submission.retained_regime, None);
    assert_eq!(submission.state(), ReviewSubmissionState::AwaitingDecision);
    assert_eq!(
        submission.control,
        ReviewSubmissionControl::ReferentSelectionRequired
    );
}

#[test]
fn t2511a_referent_peut_confirmer_durcir_ou_alleger_et_seul_un_ecart_ouvre() {
    let cases = [
        (ReviewRegime::JuryOnePlusOne, ReviewDirection::Same, false),
        (
            ReviewRegime::JuryTwoByTwo,
            ReviewDirection::Strengthened,
            true,
        ),
        (ReviewRegime::Simple, ReviewDirection::Lightened, true),
    ];

    for (retained, expected_direction, expects_deviation) in cases {
        let mut submission =
            create_review_submission(&submit_payload(), "author-1", &critical_map()).unwrap();
        let payload = selection(&submission.submission_id, retained);
        let decision =
            decide_review_regime(&mut submission, "referent-1", "referent-1", &payload).unwrap();

        assert_eq!(decision.direction, expected_direction);
        assert_eq!(decision.retained_regime, retained);
        assert_eq!(decision.deviation.is_some(), expects_deviation);
        if let Some(deviation) = decision.deviation {
            assert_eq!(deviation.direction, expected_direction);
            assert_eq!(deviation.state, ReviewDeviationState::Open);
        }
        assert_eq!(submission.retained_regime, Some(retained));
        assert_eq!(
            submission.state(),
            ReviewSubmissionState::DecisionRecordedElectionUnavailable
        );
    }
}

#[test]
fn t2511a_acteur_ou_lot_inexact_est_refuse_sans_muter_la_soumission() {
    let mut submission =
        create_review_submission(&submit_payload(), "author-1", &critical_map()).unwrap();
    let initial = submission.clone();
    let payload = selection(&submission.submission_id, ReviewRegime::JuryTwoByTwo);

    assert_eq!(
        decide_review_regime(&mut submission, "intrus", "referent-1", &payload),
        Err(ReviewDecisionError::ReferentMismatch)
    );
    assert_eq!(submission, initial);

    let wrong_lot = selection(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ReviewRegime::JuryTwoByTwo,
    );
    assert_eq!(
        decide_review_regime(&mut submission, "referent-1", "referent-1", &wrong_lot),
        Err(ReviewDecisionError::SubmissionMismatch)
    );
    assert_eq!(submission, initial);
}

#[test]
fn t2511a_une_decision_ne_peut_pas_etre_remplacee() {
    let mut submission =
        create_review_submission(&submit_payload(), "author-1", &critical_map()).unwrap();
    let first = selection(&submission.submission_id, ReviewRegime::JuryTwoByTwo);
    decide_review_regime(&mut submission, "referent-1", "referent-1", &first).unwrap();
    let recorded = submission.clone();
    let replacement = selection(&submission.submission_id, ReviewRegime::Simple);

    assert_eq!(
        decide_review_regime(&mut submission, "referent-1", "referent-1", &replacement,),
        Err(ReviewDecisionError::DecisionAlreadyRecorded)
    );
    assert_eq!(submission, recorded);
}

#[test]
fn t2511a_regime_du_noyau_est_deja_fixe_et_ne_se_remplace_pas_par_lot() {
    let mut map = map_for(
        "plugins/maicie/src/review.rs",
        "+const RULE: u8 = 4;",
        "const RULE: u8 = 4;",
    );
    assert!(map.zones.iter().any(|zone| {
        zone.evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::SelfProtection)
    }));
    map.proposed_regime = ReviewRegime::Simple;
    let mut submission = create_review_submission(&submit_payload(), "author-1", &map).unwrap();

    assert_eq!(submission.proposed_regime, F38_FIXED_REGIME);
    assert_eq!(submission.retained_regime, Some(F38_FIXED_REGIME));
    assert_eq!(
        submission.control,
        ReviewSubmissionControl::FixedByReferentDecision
    );
    assert_eq!(
        submission.state(),
        ReviewSubmissionState::DecisionRecordedElectionUnavailable
    );

    let payload = selection(&submission.submission_id, ReviewRegime::Simple);
    assert_eq!(
        decide_review_regime(&mut submission, "referent-1", "referent-1", &payload,),
        Err(ReviewDecisionError::SelfRegimeFixed)
    );
}

#[test]
fn t2511a_contrats_json_refusent_champs_libres_chemins_et_decision_omise() {
    let with_motive = json!({
        "submission_id": "a".repeat(64),
        "retained_regime": "revue_simple",
        "motif": "aucune"
    });
    assert!(
        serde_json::from_value::<ReviewRegimeSelectPayload>(with_motive).is_err(),
        "un motif libre doit mourir au décodage"
    );
    assert!(
        serde_json::from_value::<ReviewRegimeSelectPayload>(json!({
            "submission_id": "a".repeat(64)
        }))
        .is_err(),
        "le régime retenu ne doit avoir aucune valeur par défaut"
    );

    for forbidden in [
        "repository_root",
        "critical_paths",
        "reason",
        "justification",
    ] {
        let mut value = json!({
            "project_id": "bridget",
            "branch_ref": "refs/remotes/origin/fix/example",
            "base": BASE,
            "head": HEAD
        });
        value[forbidden] = json!("valeur-gratuite");
        assert!(
            serde_json::from_value::<ReviewLotSubmitPayload>(value).is_err(),
            "le champ interdit {forbidden} doit être refusé"
        );
    }
}
