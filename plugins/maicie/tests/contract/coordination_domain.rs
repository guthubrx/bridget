use maicie::domain::{
    ClasseDuree, DefinitionCoordination, DependanceDelegation, DomainError,
    EntreeReductionCoordination, EtatGenerationDelegation, EvaluationCloture,
    EvenementCoordination, FaitAppartenanceRepli, FraicheurCoordination, GenerationDelegation,
    IssueClotureEvaluee, MAX_COORDINATION_EDGES, MAX_COORDINATION_NODES,
    ModeQualificationDependance, PolitiqueReassignation, TransitionCoordinationActive,
    TypeDecisionCoordinationActive, reduire_coordination,
};
use std::collections::BTreeSet;
use uuid::Uuid;

#[test]
fn la_frontiere_a_parse_le_dto_public_et_conserve_les_octets_attestes() {
    const STREAM_A: &[u8] = include_bytes!(
        "../../../../specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl"
    );
    let event_bytes = STREAM_A
        .split(|byte| *byte == b'\n')
        .nth(5)
        .expect("événement A dans le corpus gelé");
    let event =
        EvenementCoordination::depuis_trame_attestee(event_bytes, FraicheurCoordination::Fresh)
            .unwrap();
    assert_eq!(event.canonical_bytes(), event_bytes);
    assert_eq!(event.event_id(), "evt-reminder-1");
    assert_eq!(event.request_id(), "request-1");
    assert_eq!(event.reminder_message_id(), "message-reminder-1");
    assert_eq!(event.recipient(), "codex-1");
    assert_eq!(event.generation(), 1);
    assert_eq!(event.observed_at(), 1_787_500_003);
    assert_eq!(event.cursor(), 1);
    assert_eq!(event.freshness(), FraicheurCoordination::Fresh);

    // Mutation discriminante : le type fermé public devient inconnu. Le fait
    // est refusé à la frontière, avant que T1606 puisse calculer une décision.
    let mut mutated = event_bytes.to_vec();
    let marker = b"reminder_sent";
    let offset = mutated
        .windows(marker.len())
        .position(|window| window == marker)
        .expect("kind public dans la fixture");
    mutated[offset + marker.len() - 1] = b'x';
    assert!(
        EvenementCoordination::depuis_trame_attestee(&mutated, FraicheurCoordination::Fresh)
            .is_err()
    );
}

#[test]
fn reducteur_repete_cent_fois_les_memes_octets_et_refuse_une_fraicheur_incomplete() {
    let objectif_id = Uuid::new_v4();
    let delegation_id = Uuid::new_v4();
    let generation = generation(objectif_id, delegation_id, "codex-1");
    let policy = policy(objectif_id, delegation_id, "bob");
    let event = fixture_event(FraicheurCoordination::Fresh);
    let input = EntreeReductionCoordination::EvenementAtteste {
        objectif_id,
        delegation_id,
        evenement: event,
    };
    let expected =
        serde_json::to_vec(&reduire_coordination(&generation, &policy, &input).unwrap()).unwrap();
    for _ in 0..100 {
        let replay = reduire_coordination(&generation, &policy, &input).unwrap();
        assert_eq!(serde_json::to_vec(&replay).unwrap(), expected);
        assert_eq!(replay.decision.kind, TypeDecisionCoordinationActive::Aucun);
        assert_eq!(replay.transition, TransitionCoordinationActive::Aucune);
    }

    let incomplete = EntreeReductionCoordination::EvenementAtteste {
        objectif_id,
        delegation_id,
        evenement: fixture_event(FraicheurCoordination::Gap),
    };
    assert_eq!(
        reduire_coordination(&generation, &policy, &incomplete),
        Err(DomainError::DonneeInvalide(
            "observation de coordination incomplète"
        ))
    );
}

#[test]
fn reducteur_est_le_seul_producteur_de_l_acte_de_cloture_evaluee() {
    let objectif_id = Uuid::new_v4();
    let delegation_id = Uuid::new_v4();
    let generation = generation(objectif_id, delegation_id, "alice");
    let policy = policy(objectif_id, delegation_id, "bob");
    let input = EntreeReductionCoordination::ClotureEvaluee(EvaluationCloture {
        event_id: "evaluation-1".to_string(),
        objectif_id,
        delegation_id,
        generation: 1,
        delivery_hash: "ab".repeat(32),
        issue: IssueClotureEvaluee::LivraisonValidee,
        evaluated_at: 1_787_500_100,
    });
    let first = reduire_coordination(&generation, &policy, &input).unwrap();
    let expected = serde_json::to_vec(&first).unwrap();
    for _ in 0..100 {
        assert_eq!(
            serde_json::to_vec(&reduire_coordination(&generation, &policy, &input).unwrap())
                .unwrap(),
            expected
        );
    }
    let TransitionCoordinationActive::ClotureEvaluee(act) = first.transition else {
        panic!("acte de clôture attendu")
    };
    assert_eq!(act.objectif_id(), objectif_id);
    assert_eq!(act.delegation_id(), delegation_id);
    assert_eq!(act.generation(), 1);
    assert_eq!(act.delivery_hash(), "ab".repeat(32));
    assert_eq!(
        act.issue_qualifiante(),
        IssueClotureEvaluee::LivraisonValidee
    );
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

#[test]
fn borne_de_noeuds_accepte_cent_et_refuse_cent_un_independamment_des_aretes() {
    let objectif_id = Uuid::new_v4();
    let accepted = chain_definition(objectif_id, MAX_COORDINATION_NODES);
    assert_eq!(accepted.dependencies.len(), 99);
    accepted.verifier_bornes().unwrap();

    let rejected = chain_definition(objectif_id, MAX_COORDINATION_NODES + 1);
    assert_eq!(rejected.dependencies.len(), 100);
    assert!(rejected.dependencies.len() < MAX_COORDINATION_EDGES);
    assert_eq!(
        rejected.verifier_bornes(),
        Err(DomainError::DonneeInvalide(
            "graphe de coordination hors borne"
        ))
    );
}

fn chain_definition(objectif_id: Uuid, nodes: usize) -> DefinitionCoordination {
    let ids: Vec<Uuid> = (1..=nodes)
        .map(|index| Uuid::from_u128(index as u128))
        .collect();
    DefinitionCoordination {
        objectif_id,
        dependencies: ids
            .windows(2)
            .map(|pair| DependanceDelegation {
                objectif_id,
                prerequis_id: pair[0],
                dependant_id: pair[1],
                mode: ModeQualificationDependance::HashGreffe,
            })
            .collect(),
        policies: vec![],
        attentes: vec![],
    }
}

fn membership(objectif_id: Uuid, participant_id: &str, est_pilote: bool) -> FaitAppartenanceRepli {
    FaitAppartenanceRepli {
        objectif_id,
        participant_id: participant_id.to_string(),
        membership_version: 1,
        est_pilote,
    }
}

fn generation(
    objectif_id: Uuid,
    delegation_id: Uuid,
    participant_id: &str,
) -> GenerationDelegation {
    GenerationDelegation {
        delegation_id,
        objectif_id,
        generation: 1,
        participant_id: participant_id.to_string(),
        etat: EtatGenerationDelegation::Ouverte,
        generation_precedente: None,
        trigger_event_id: None,
    }
}

fn policy(objectif_id: Uuid, delegation_id: Uuid, fallback: &str) -> PolitiqueReassignation {
    PolitiqueReassignation {
        delegation_id,
        objectif_id,
        classe: ClasseDuree::Normale,
        version: 1,
        seuil_relances: 2,
        max_reemissions: 2,
        chaine_repli: vec![membership(objectif_id, fallback, false)],
    }
}

fn fixture_event(freshness: FraicheurCoordination) -> EvenementCoordination {
    const STREAM_A: &[u8] = include_bytes!(
        "../../../../specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl"
    );
    let event_bytes = STREAM_A
        .split(|byte| *byte == b'\n')
        .nth(5)
        .expect("événement A dans le corpus gelé");
    EvenementCoordination::depuis_trame_attestee(event_bytes, freshness).unwrap()
}
