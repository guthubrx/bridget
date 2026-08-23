use maicie::domain::{
    ActivationOutbox, ApprobationActivation, Delegation, ObjectifCoordonne, OutboxDelegation,
    ProfilEquipe, SnapshotTransport,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CorpusDomaine {
    objectif: ObjectifCoordonne,
    delegation: Delegation,
    outbox_delegation: OutboxDelegation,
    snapshot_transport: SnapshotTransport,
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
