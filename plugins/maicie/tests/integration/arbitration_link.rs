use maicie::domain::{
    ClasseDuree, Delegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne,
    OutboxDelegation,
};
use maicie::outbox::{PreparedDelegation, StoreCommitPhase, stable_body_hash};
use maicie::store::{MaicieStore, StoreError};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

const FRAME_LIMIT: usize = 256 * 1024;

#[test]
fn lien_est_un_fait_du_document_delegation_et_survit_a_la_reouverture() {
    let fixture = Fixture::new("durable");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let linked = prepared(&store, "même libellé", Some("review:R-42"));
    let ordinary = prepared(&store, "même libellé", None);
    store.create_prepared_delegation(&linked).unwrap();
    store.create_prepared_delegation(&ordinary).unwrap();

    let links = store.delegation_arbitration_links().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].constat_id, "review:R-42");
    assert_eq!(links[0].objectif_id, linked.objective.id);
    assert_eq!(links[0].delegation_id, linked.delegation.id);
    assert_ne!(links[0].objectif_id, ordinary.objective.id);

    drop(store);
    let reopened = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(reopened.delegation_arbitration_links().unwrap(), links);
}

#[test]
fn erreur_avant_commit_ne_laisse_ni_delegation_ni_lien() {
    let fixture = Fixture::new("rollback");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let linked = prepared(&store, "transaction", Some("gate:G1704"));

    let result = store.create_prepared_delegation_observed(&linked, |phase| match phase {
        StoreCommitPhase::BeforeCommit => Err(StoreError::Conflict("barrière avant commit")),
        StoreCommitPhase::AfterCommit => Ok(()),
    });
    assert!(result.is_err());
    assert!(store.delegation_arbitration_links().unwrap().is_empty());
    assert!(
        store
            .objective_snapshots(Some(linked.objective.id))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn index_divergent_est_refuse_plutot_que_deviné_par_homonymie() {
    let fixture = Fixture::new("divergence");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let linked = prepared(&store, "même libellé", Some("incident:I-7"));
    let other = prepared(&store, "même libellé", None);
    store.create_prepared_delegation(&linked).unwrap();
    store.create_prepared_delegation(&other).unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&fixture.database).unwrap();
    connection
        .execute(
            "UPDATE delegations SET objective_id = ?1 WHERE id = ?2",
            [
                other.objective.id.to_string(),
                linked.delegation.id.to_string(),
            ],
        )
        .unwrap();
    drop(connection);

    let reopened = MaicieStore::open(&fixture.database).unwrap();
    assert!(reopened.delegation_arbitration_links().is_err());
}

#[test]
fn delegation_historique_sans_constat_reste_compatible_et_ne_cree_aucun_lien() {
    let objective_id = Uuid::new_v4();
    let historical = format!(
        r#"{{"id":"{}","objectif_id":"{}","participant":"agent","instruction":"travail","duree":"normale","etat":"creee","raison":"historique"}}"#,
        Uuid::new_v4(),
        objective_id
    );
    let delegation: Delegation = serde_json::from_str(&historical).unwrap();
    assert_eq!(delegation.constat_id, None);
    assert_eq!(delegation.lien_arbitrage().unwrap(), None);
    assert!(
        !serde_json::to_string(&delegation)
            .unwrap()
            .contains("constat_id")
    );
}

#[test]
fn constat_id_est_exact_non_vide_et_ne_peut_pas_etre_remplace() {
    let objective_id = Uuid::new_v4();
    let base = Delegation::nouvelle(
        objective_id,
        "agent",
        "travail",
        ClasseDuree::Normale,
        "arbitrage humain",
    )
    .unwrap();
    assert!(base.clone().pour_constat(" constat ").is_err());
    assert!(base.clone().pour_constat("ligne\nsuivante").is_err());

    let linked = base.pour_constat("review:R-42").unwrap();
    assert!(linked.pour_constat("review:R-43").is_err());
}

fn prepared(store: &MaicieStore, goal: &str, constat_id: Option<&str>) -> PreparedDelegation {
    let mut objective = ObjectifCoordonne::nouveau(goal, ModeObjectif::Delegue, 1_000).unwrap();
    objective
        .transition(EtatObjectif::EnCoordination, 1_001)
        .unwrap();
    let delegation = Delegation::nouvelle(
        objective.id,
        "agent",
        goal,
        ClasseDuree::Normale,
        "arbitrage humain",
    )
    .and_then(|delegation| match constat_id {
        Some(id) => delegation.pour_constat(id),
        None => Ok(delegation),
    })
    .unwrap();
    let body = goal.as_bytes().to_vec();
    let outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: delegation.id,
        target: delegation.participant.clone(),
        body_hash: stable_body_hash(&body),
        body_bytes: body,
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_061,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_050,
        dedup_retained_until: 1_100,
    };
    PreparedDelegation::new(
        objective,
        delegation,
        outbox,
        store.issuer_scope(),
        1_001,
        FRAME_LIMIT,
    )
    .unwrap()
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "maicie-arbitration-link-{label}-{}",
            Uuid::new_v4()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.db");
        Self { root, database }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
