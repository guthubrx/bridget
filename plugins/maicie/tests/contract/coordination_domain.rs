use maicie::domain::{
    ClasseDuree, DefinitionCoordination, DependanceDelegation, DomainError, EvenementCoordination,
    FaitAppartenanceRepli, FraicheurCoordination, MAX_COORDINATION_EDGES, MAX_COORDINATION_NODES,
    ModeQualificationDependance, PolitiqueReassignation,
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
