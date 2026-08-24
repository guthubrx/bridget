//! Oracles politique 31 — registre des plages (comparaison exacte de noms).

use maicie::domain::{
    ClasseDuree, Delegation, EtatDelegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif,
    ObjectifCoordonne, OutboxDelegation,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::store::{MaicieStore, StoreError};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

const FRAME_LIMIT: usize = 256 * 1024;

#[test]
fn chevauchement_refuse_avec_titulaire_nomme() {
    let fixture = Fixture::new("chevauchement");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let titulaire = seed_objective(&mut store, "titulaire");
    let concurrent = seed_objective(&mut store, "concurrent");

    store
        .reserve_resource_range("migration:v14", titulaire, 1_000)
        .unwrap();

    let erreur = store
        .reserve_resource_range("migration:v14", concurrent, 1_001)
        .unwrap_err();
    let message = format!("{erreur}");
    match erreur {
        StoreError::ResourceHeld {
            resource,
            holder_objective_id,
        } => {
            assert_eq!(resource, "migration:v14");
            assert_eq!(holder_objective_id, titulaire);
        }
        other => panic!("attendu ResourceHeld, reçu {other}"),
    }
    assert!(message.contains(&titulaire.to_string()));
}

#[test]
fn liberation_a_la_cloture_prouvee_meme_transaction() {
    let fixture = Fixture::new("liberation-cloture");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_objective(&mut store, "a-clore");
    store
        .reserve_resource_range("protocol.rs:Usage*", objective_id, 2_000)
        .unwrap();
    assert_eq!(store.list_resource_ranges().unwrap().len(), 1);

    // Crash injecté après notifications (et donc après libération plages dans
    // la même transaction) : rien ne doit rester durable.
    let failed =
        store.close_objective_observed(objective_id, "clôture atomique plages", 2_100, |phase| {
            if phase == maicie::store::ObjectiveClosureCommitPhase::AfterOutboxes {
                return Err(StoreError::Conflict("faute après libération plages"));
            }
            Ok(())
        });
    assert!(matches!(
        failed,
        Err(StoreError::Conflict("faute après libération plages"))
    ));
    assert_eq!(
        store.list_resource_ranges().unwrap().len(),
        1,
        "rollback doit conserver la réservation"
    );
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_ne!(snapshot.objective.etat, EtatObjectif::Clos);

    store
        .close_objective(objective_id, "clôture atomique plages", 2_100)
        .unwrap();
    assert!(
        store.list_resource_ranges().unwrap().is_empty(),
        "clôture commitée doit libérer la plage"
    );
}

#[test]
fn ressources_distinctes_coexistent() {
    let fixture = Fixture::new("coexistence");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let a = seed_objective(&mut store, "obj-a");
    let b = seed_objective(&mut store, "obj-b");

    store
        .reserve_resource_range("fichier:wrapper.rs", a, 3_000)
        .unwrap();
    store
        .reserve_resource_range("migration:v14", b, 3_001)
        .unwrap();
    // Noms distincts même proches : coexistence.
    store
        .reserve_resource_range("migration:v14 ", a, 3_002)
        .unwrap();

    let listed = store.list_resource_ranges().unwrap();
    assert_eq!(listed.len(), 3);
    assert_eq!(listed[0].resource, "fichier:wrapper.rs");
    assert_eq!(listed[0].objective_id, a);
    assert_eq!(listed[1].resource, "migration:v14");
    assert_eq!(listed[1].objective_id, b);
    assert_eq!(listed[2].resource, "migration:v14 ");
    assert_eq!(listed[2].objective_id, a);
}

#[test]
fn rejeu_meme_titulaire_conserve_reserved_at_du_premier() {
    // Tue MUTANT-A : branche holder==objective_id remplacée par Err.
    let fixture = Fixture::new("rejeu-idempotent");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let titulaire = seed_objective(&mut store, "rejeu");

    let premier = store
        .reserve_resource_range("migration:v14", titulaire, 4_000)
        .unwrap();
    assert_eq!(premier.reserved_at, 4_000);

    let second = store
        .reserve_resource_range("migration:v14", titulaire, 4_999)
        .unwrap();
    assert_eq!(
        second.reserved_at, 4_000,
        "le rejeu doit conserver reserved_at du PREMIER, pas celui du second appel"
    );
    assert_eq!(second.objective_id, titulaire);
    assert_eq!(second.resource, "migration:v14");

    let listed = store.list_resource_ranges().unwrap();
    assert_eq!(listed.len(), 1, "une seule ligne après rejeu");
    assert_eq!(listed[0].reserved_at, 4_000);
    assert_eq!(listed[0].objective_id, titulaire);
}

#[test]
fn casse_distingue_deux_ressources_accordees_a_deux_objectifs() {
    // Tue MUTANT-B : COLLATE NOCASE qui fusionnerait migration:v14 et Migration:V14.
    let fixture = Fixture::new("casse-exacte");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let bas = seed_objective(&mut store, "bas");
    let haut = seed_objective(&mut store, "haut");

    store
        .reserve_resource_range("migration:v14", bas, 5_000)
        .unwrap();
    store
        .reserve_resource_range("Migration:V14", haut, 5_001)
        .unwrap();

    let listed = store.list_resource_ranges().unwrap();
    assert_eq!(
        listed.len(),
        2,
        "la casse DISTINGUE : deux lignes accordées, pas un chevauchement"
    );
    let noms: Vec<&str> = listed.iter().map(|r| r.resource.as_str()).collect();
    assert!(noms.contains(&"migration:v14"));
    assert!(noms.contains(&"Migration:V14"));
    assert_eq!(
        listed
            .iter()
            .find(|r| r.resource == "migration:v14")
            .unwrap()
            .objective_id,
        bas
    );
    assert_eq!(
        listed
            .iter()
            .find(|r| r.resource == "Migration:V14")
            .unwrap()
            .objective_id,
        haut
    );
}

fn seed_objective(store: &mut MaicieStore, label: &str) -> Uuid {
    let objective = ObjectifCoordonne::nouveau(label, ModeObjectif::Delegue, 1).unwrap();
    let delegation = Delegation::nouvelle(
        objective.id,
        "alice",
        "instruction",
        ClasseDuree::Normale,
        "raison",
    )
    .unwrap();
    let body = format!("travail {label}").into_bytes();
    let outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: delegation.id,
        target: "alice".into(),
        body_hash: stable_body_hash(&body),
        body_bytes: body,
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_100,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_050,
        dedup_retained_until: 1_200,
    };
    let prepared = PreparedDelegation::new(
        objective.clone(),
        delegation,
        outbox,
        store.issuer_scope(),
        1_000,
        FRAME_LIMIT,
    )
    .unwrap();
    store.create_prepared_delegation(&prepared).unwrap();
    assert_eq!(prepared.delegation.etat, EtatDelegation::Creee);
    objective.id
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("maicie-plage-{name}-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            database: root.join("maicie.sqlite3"),
            root,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
