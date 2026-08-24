use maicie::domain::{
    ClasseDuree, DefinitionCoordination, DependanceDelegation, DomainError, EvenementCoordination,
    FaitAppartenanceRepli, FraicheurCoordination, MAX_COORDINATION_EDGES,
    ModeQualificationDependance, PolitiqueReassignation, TypeEvenementCoordination,
};
use std::collections::BTreeSet;
use uuid::Uuid;

#[test]
fn les_evenements_de_cycle_ne_qualifient_jamais_une_arete() {
    for kind in [
        TypeEvenementCoordination::Answered,
        TypeEvenementCoordination::Cancelled,
        TypeEvenementCoordination::TimedOut,
        TypeEvenementCoordination::ReminderSent,
    ] {
        let event = EvenementCoordination {
            event_id: format!("event-{kind:?}"),
            objectif_id: Uuid::new_v4(),
            delegation_id: Some(Uuid::new_v4()),
            generation: Some(1),
            kind,
            observed_at: 1,
            freshness: FraicheurCoordination::Fresh,
        };
        assert!(!event.peut_qualifier_une_arete());
    }
    let report = EvenementCoordination {
        event_id: "delivery-report".into(),
        objectif_id: Uuid::new_v4(),
        delegation_id: Some(Uuid::new_v4()),
        generation: Some(1),
        kind: TypeEvenementCoordination::DeliveryReport,
        observed_at: 1,
        freshness: FraicheurCoordination::Gap,
    };
    assert!(!report.peut_qualifier_une_arete());
}

#[test]
fn politique_refuse_seuil_borne_pilote_absent_et_doublon() {
    let objectif_id = Uuid::new_v4();
    let delegation_id = Uuid::new_v4();
    let participants = BTreeSet::from(["alice".to_string(), "bob".to_string()]);
    let mut policy = PolitiqueReassignation {
        delegation_id,
        objectif_id,
        classe: ClasseDuree::Normale,
        version: 1,
        seuil_relances: 1,
        max_reemissions: 2,
        chaine_repli: vec![membership(objectif_id, "bob", false)],
    };
    policy.verifier(&participants).unwrap();

    policy.seuil_relances = 0;
    assert_eq!(
        policy.verifier(&participants),
        Err(DomainError::DonneeInvalide(
            "politique sans version ou seuil"
        ))
    );
    policy.seuil_relances = 1;
    policy.max_reemissions = 9;
    assert!(policy.verifier(&participants).is_err());
    policy.max_reemissions = 2;
    policy.chaine_repli = vec![membership(objectif_id, "pilote", true)];
    assert!(policy.verifier(&participants).is_err());
    policy.chaine_repli = vec![membership(objectif_id, "inconnu", false)];
    assert!(policy.verifier(&participants).is_err());
    policy.chaine_repli = vec![
        membership(objectif_id, "bob", false),
        membership(objectif_id, "bob", false),
    ];
    assert!(policy.verifier(&participants).is_err());
}

#[test]
fn definition_refuse_doublon_inter_objectif_et_borne_avant_store() {
    let objectif_id = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let edge = DependanceDelegation {
        objectif_id,
        prerequis_id: a,
        dependant_id: b,
        mode: ModeQualificationDependance::HashGreffe,
    };
    let mut definition = DefinitionCoordination {
        objectif_id,
        dependencies: vec![edge.clone(), edge],
        policies: vec![],
        attentes: vec![],
    };
    assert!(definition.verifier_bornes().is_err());

    definition.dependencies = vec![DependanceDelegation {
        objectif_id: Uuid::new_v4(),
        prerequis_id: a,
        dependant_id: b,
        mode: ModeQualificationDependance::HashGreffe,
    }];
    assert!(definition.verifier_bornes().is_err());

    definition.dependencies = (0..=MAX_COORDINATION_EDGES)
        .map(|_| DependanceDelegation {
            objectif_id,
            prerequis_id: Uuid::new_v4(),
            dependant_id: Uuid::new_v4(),
            mode: ModeQualificationDependance::HashGreffe,
        })
        .collect();
    assert!(definition.verifier_bornes().is_err());
}

fn membership(objectif_id: Uuid, participant_id: &str, est_pilote: bool) -> FaitAppartenanceRepli {
    FaitAppartenanceRepli {
        objectif_id,
        participant_id: participant_id.to_string(),
        membership_version: 1,
        est_pilote,
    }
}
