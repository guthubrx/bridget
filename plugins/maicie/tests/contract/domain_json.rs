use maicie::domain::{
    ActivationOutbox, ApprobationActivation, ClasseDuree, DecisionCoordination, Delegation,
    EtatActivationOutbox, EtatActivationProfil, EtatDecision, EtatDelegation, EtatFlux,
    EtatObjectif, EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne, OutboxDelegation,
    ProfilEquipe, SnapshotTransport, SourceSnapshot, TypeDecision,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CorpusDomaine {
    objectif: ObjectifCoordonne,
    delegation: Delegation,
    outbox_delegation: OutboxDelegation,
    snapshot_transport: SnapshotTransport,
    decision_coordination: DecisionCoordination,
    approbation_activation: ApprobationActivation,
    profil_equipe: ProfilEquipe,
    activation_outbox: ActivationOutbox,
}

#[test]
fn corpus_json_v1_du_domaine_est_fige_octet_pour_octet() {
    let fixture = include_str!("../fixtures/domain-v1.json");
    let corpus: CorpusDomaine = serde_json::from_str(fixture).expect("fixture domaine v1 valide");

    let rendered = serde_json::to_string_pretty(&corpus).expect("corpus domaine sérialisable");
    assert_eq!(format!("{rendered}\n"), fixture);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MatriceEnumsPersistes {
    mode_objectif: Vec<ModeObjectif>,
    etat_objectif: Vec<EtatObjectif>,
    classe_duree: Vec<ClasseDuree>,
    etat_delegation: Vec<EtatDelegation>,
    etat_outbox_delegation: Vec<EtatOutboxDelegation>,
    source_snapshot: Vec<SourceSnapshot>,
    etat_flux: Vec<EtatFlux>,
    etat_activation_profil: Vec<EtatActivationProfil>,
    etat_activation_outbox: Vec<EtatActivationOutbox>,
    type_decision: Vec<TypeDecision>,
    etat_decision: Vec<EtatDecision>,
}

#[test]
fn matrice_json_v1_couvre_toutes_les_variantes_persistes_dans_les_deux_sens() {
    let fixture = include_str!("../fixtures/domain-enums-v1.json");
    let corpus: MatriceEnumsPersistes =
        serde_json::from_str(fixture).expect("fixture enums domaine v1 valide");

    let expected = MatriceEnumsPersistes {
        mode_objectif: all_mode_objectif(),
        etat_objectif: all_etat_objectif(),
        classe_duree: all_classe_duree(),
        etat_delegation: all_etat_delegation(),
        etat_outbox_delegation: all_etat_outbox_delegation(),
        source_snapshot: all_source_snapshot(),
        etat_flux: all_etat_flux(),
        etat_activation_profil: all_etat_activation_profil(),
        etat_activation_outbox: all_etat_activation_outbox(),
        type_decision: all_type_decision(),
        etat_decision: all_etat_decision(),
    };

    assert_eq!(corpus, expected, "lecture de la matrice v1");
    let rendered = serde_json::to_string_pretty(&expected).expect("matrice domaine sérialisable");
    assert_eq!(
        format!("{rendered}\n"),
        fixture,
        "écriture de la matrice v1"
    );
}

fn all_mode_objectif() -> Vec<ModeObjectif> {
    exhaustive_mode_objectif(ModeObjectif::Collaboratif);
    vec![ModeObjectif::Collaboratif, ModeObjectif::Delegue]
}

fn exhaustive_mode_objectif(value: ModeObjectif) {
    match value {
        ModeObjectif::Collaboratif | ModeObjectif::Delegue => {}
    }
}

fn all_etat_objectif() -> Vec<EtatObjectif> {
    exhaustive_etat_objectif(EtatObjectif::Ouvert);
    vec![
        EtatObjectif::Ouvert,
        EtatObjectif::EnCoordination,
        EtatObjectif::AEvaluer,
        EtatObjectif::Synthetise,
        EtatObjectif::Clos,
    ]
}

fn exhaustive_etat_objectif(value: EtatObjectif) {
    match value {
        EtatObjectif::Ouvert
        | EtatObjectif::EnCoordination
        | EtatObjectif::AEvaluer
        | EtatObjectif::Synthetise
        | EtatObjectif::Clos => {}
    }
}

fn all_classe_duree() -> Vec<ClasseDuree> {
    exhaustive_classe_duree(ClasseDuree::Courte);
    vec![
        ClasseDuree::Courte,
        ClasseDuree::Normale,
        ClasseDuree::Longue,
    ]
}

fn exhaustive_classe_duree(value: ClasseDuree) {
    match value {
        ClasseDuree::Courte | ClasseDuree::Normale | ClasseDuree::Longue => {}
    }
}

fn all_etat_delegation() -> Vec<EtatDelegation> {
    exhaustive_etat_delegation(EtatDelegation::Creee);
    vec![
        EtatDelegation::Creee,
        EtatDelegation::AEvaluer,
        EtatDelegation::Terminee,
        EtatDelegation::Annulee,
    ]
}

fn exhaustive_etat_delegation(value: EtatDelegation) {
    match value {
        EtatDelegation::Creee
        | EtatDelegation::AEvaluer
        | EtatDelegation::Terminee
        | EtatDelegation::Annulee => {}
    }
}

fn all_etat_outbox_delegation() -> Vec<EtatOutboxDelegation> {
    exhaustive_etat_outbox_delegation(EtatOutboxDelegation::Prepared);
    vec![
        EtatOutboxDelegation::Prepared,
        EtatOutboxDelegation::OutcomeUnknown,
        EtatOutboxDelegation::Accepted,
        EtatOutboxDelegation::Rejected,
    ]
}

fn exhaustive_etat_outbox_delegation(value: EtatOutboxDelegation) {
    match value {
        EtatOutboxDelegation::Prepared
        | EtatOutboxDelegation::OutcomeUnknown
        | EtatOutboxDelegation::Accepted
        | EtatOutboxDelegation::Rejected => {}
    }
}

fn all_source_snapshot() -> Vec<SourceSnapshot> {
    exhaustive_source_snapshot(SourceSnapshot::Bridget);
    vec![SourceSnapshot::Bridget, SourceSnapshot::AcpSubscription]
}

fn exhaustive_source_snapshot(value: SourceSnapshot) {
    match value {
        SourceSnapshot::Bridget | SourceSnapshot::AcpSubscription => {}
    }
}

fn all_etat_flux() -> Vec<EtatFlux> {
    exhaustive_etat_flux(EtatFlux::Fresh);
    vec![
        EtatFlux::Fresh,
        EtatFlux::Gap,
        EtatFlux::Ended,
        EtatFlux::Unavailable,
    ]
}

fn exhaustive_etat_flux(value: EtatFlux) {
    match value {
        EtatFlux::Fresh | EtatFlux::Gap | EtatFlux::Ended | EtatFlux::Unavailable => {}
    }
}

fn all_etat_activation_profil() -> Vec<EtatActivationProfil> {
    exhaustive_etat_activation_profil(EtatActivationProfil::Inactif);
    vec![
        EtatActivationProfil::Inactif,
        EtatActivationProfil::Proposition,
        EtatActivationProfil::Approuve,
        EtatActivationProfil::Lance,
        EtatActivationProfil::Connecte,
    ]
}

fn exhaustive_etat_activation_profil(value: EtatActivationProfil) {
    match value {
        EtatActivationProfil::Inactif
        | EtatActivationProfil::Proposition
        | EtatActivationProfil::Approuve
        | EtatActivationProfil::Lance
        | EtatActivationProfil::Connecte => {}
    }
}

fn all_etat_activation_outbox() -> Vec<EtatActivationOutbox> {
    exhaustive_etat_activation_outbox(EtatActivationOutbox::Dispatching);
    vec![
        EtatActivationOutbox::Dispatching,
        EtatActivationOutbox::OutcomeUnknown,
        EtatActivationOutbox::Applied,
    ]
}

fn exhaustive_etat_activation_outbox(value: EtatActivationOutbox) {
    match value {
        EtatActivationOutbox::Dispatching
        | EtatActivationOutbox::OutcomeUnknown
        | EtatActivationOutbox::Applied => {}
    }
}

fn all_type_decision() -> Vec<TypeDecision> {
    exhaustive_type_decision(TypeDecision::AjouterParticipant);
    vec![
        TypeDecision::AjouterParticipant,
        TypeDecision::RetirerParticipant,
        TypeDecision::ConstaterIssue,
        TypeDecision::Relancer,
        TypeDecision::ReveillerProfil,
        TypeDecision::Cloturer,
    ]
}

fn exhaustive_type_decision(value: TypeDecision) {
    match value {
        TypeDecision::AjouterParticipant
        | TypeDecision::RetirerParticipant
        | TypeDecision::ConstaterIssue
        | TypeDecision::Relancer
        | TypeDecision::ReveillerProfil
        | TypeDecision::Cloturer => {}
    }
}

fn all_etat_decision() -> Vec<EtatDecision> {
    exhaustive_etat_decision(EtatDecision::Proposee);
    vec![
        EtatDecision::Proposee,
        EtatDecision::Approuvee,
        EtatDecision::Refusee,
        EtatDecision::Appliquee,
    ]
}

fn exhaustive_etat_decision(value: EtatDecision) {
    match value {
        EtatDecision::Proposee
        | EtatDecision::Approuvee
        | EtatDecision::Refusee
        | EtatDecision::Appliquee => {}
    }
}
