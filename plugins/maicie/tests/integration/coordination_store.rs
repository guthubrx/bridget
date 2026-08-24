use maicie::domain::{
    AttenteNotification, ClasseDuree, DefinitionCoordination, Delegation, DependanceDelegation,
    EtatOutboxDelegation, FaitAppartenanceRepli, ModeObjectif, ModeQualificationDependance,
    ObjectifCoordonne, OutboxDelegation, PolitiqueReassignation, TypeEvenementCoordination,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::store::MaicieStore;
use rusqlite::Connection;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

const FRAME_LIMIT: usize = 256 * 1024;

#[test]
fn migration_v7_puis_seconde_ouverture_conservent_l_historique() {
    let fixture = Fixture::new("migration-v8");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objectif = ObjectifCoordonne::nouveau("historique", ModeObjectif::Delegue, 1).unwrap();
    let delegation = create_delegation(&mut store, &objectif, "alice");
    drop(store);

    let connection = Connection::open(&fixture.database).unwrap();
    connection
        .execute_batch(
            "DROP TABLE notification_outbox;
             DROP TABLE active_coordination_decisions;
             DROP TABLE reminder_episodes;
             DROP TABLE delegation_generations;
             DROP TABLE delegation_lineages;
             DROP TABLE reassignment_policies;
             DROP TABLE evaluated_closure_acts;
             DROP TABLE delegation_dependencies;
             DROP TABLE coordination_expectations;
             DROP TABLE coordination_events_v1;",
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 7).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 8", [])
        .unwrap();
    drop(connection);

    let store = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 8);
    assert_eq!(
        store.objective_snapshots(Some(objectif.id)).unwrap()[0].delegations[0].id,
        delegation
    );
    drop(store);
    let connection = Connection::open(&fixture.database).unwrap();
    let coordination_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                 'coordination_events_v1', 'coordination_expectations',
                 'delegation_dependencies', 'evaluated_closure_acts',
                 'reassignment_policies', 'delegation_lineages',
                 'delegation_generations', 'reminder_episodes',
                 'active_coordination_decisions', 'notification_outbox'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(coordination_tables, 10);
    drop(connection);
    let reopened = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 8);
    assert_eq!(
        reopened
            .objective_snapshots(Some(objectif.id))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn snapshot_refuse_cycle_et_reference_exterieure_sans_aucune_mutation() {
    let fixture = Fixture::new("refus-atomique");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objectif = ObjectifCoordonne::nouveau("dag", ModeObjectif::Delegue, 1).unwrap();
    let a = create_delegation(&mut store, &objectif, "alice");
    let b = create_delegation(&mut store, &objectif, "bob");
    let autre = ObjectifCoordonne::nouveau("autre", ModeObjectif::Delegue, 1).unwrap();
    let externe = create_delegation(&mut store, &autre, "carol");

    let cycle = definition(
        objectif.id,
        a,
        vec![edge(objectif.id, a, b), edge(objectif.id, b, a)],
        "bob",
    );
    assert!(store.register_coordination_snapshot(&cycle).is_err());
    assert!(store.coordination_snapshot(objectif.id).unwrap().is_none());

    let cross = definition(objectif.id, a, vec![edge(objectif.id, externe, b)], "bob");
    assert!(store.register_coordination_snapshot(&cross).is_err());
    assert!(store.coordination_snapshot(objectif.id).unwrap().is_none());
}

#[test]
fn snapshot_epingle_politique_lignee_et_index_inverse_sans_relire_la_config() {
    let fixture = Fixture::new("snapshot-immuable");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objectif = ObjectifCoordonne::nouveau("figer", ModeObjectif::Delegue, 1).unwrap();
    let a = create_delegation(&mut store, &objectif, "alice");
    let b = create_delegation(&mut store, &objectif, "bob");
    let original = definition(objectif.id, a, vec![edge(objectif.id, a, b)], "bob");
    store.register_coordination_snapshot(&original).unwrap();
    store.register_coordination_snapshot(&original).unwrap();

    let mut configuration_courante = original.clone();
    configuration_courante.policies[0].seuil_relances = 99;
    configuration_courante.policies[0].chaine_repli[0].participant_id = "carol".into();
    assert!(
        store
            .register_coordination_snapshot(&configuration_courante)
            .is_err()
    );

    let snapshot = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(snapshot.definition, original);
    assert_eq!(snapshot.lineages.len(), 2);
    assert_eq!(snapshot.generations.len(), 2);
    let source = snapshot
        .generations
        .iter()
        .find(|generation| generation.delegation_id == a)
        .unwrap();
    assert_eq!(source.participant_id, "alice");
    assert_eq!(
        source.etat,
        maicie::domain::EtatGenerationDelegation::Ouverte
    );
    let dependent = snapshot
        .generations
        .iter()
        .find(|generation| generation.delegation_id == b)
        .unwrap();
    assert_eq!(dependent.participant_id, "bob");
    assert_eq!(
        dependent.etat,
        maicie::domain::EtatGenerationDelegation::Bloquee
    );
    assert_eq!(snapshot.dependants_by_prerequisite(a), vec![b]);

    let connection = Connection::open(&fixture.database).unwrap();
    let index_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'delegation_dependencies_prerequisite_idx'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(index_count, 1);
}

#[test]
fn politiques_invalides_sont_refusees_avant_toute_mutation_sqlite() {
    let fixture = Fixture::new("policy-refusals");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objectif = ObjectifCoordonne::nouveau("politique", ModeObjectif::Delegue, 1).unwrap();
    let a = create_delegation(&mut store, &objectif, "alice");
    create_delegation(&mut store, &objectif, "bob");

    let base = definition(objectif.id, a, vec![], "bob");
    let mut invalides = Vec::new();
    let mut n_zero = base.clone();
    n_zero.policies[0].seuil_relances = 0;
    invalides.push(n_zero);
    for max_reemissions in [0, 9] {
        let mut invalid = base.clone();
        invalid.policies[0].max_reemissions = max_reemissions;
        invalides.push(invalid);
    }
    let mut absent = base.clone();
    absent.policies[0].chaine_repli[0].participant_id = "inconnu".into();
    invalides.push(absent);
    let mut pilote = base.clone();
    pilote.policies[0].chaine_repli[0].participant_id = maicie::MAICIE_IDENTITY.into();
    pilote.policies[0].chaine_repli[0].est_pilote = false;
    invalides.push(pilote);
    let mut duplicate = base.clone();
    let repeated = duplicate.policies[0].chaine_repli[0].clone();
    duplicate.policies[0].chaine_repli.push(repeated);
    invalides.push(duplicate);
    let mut autre_objectif = base;
    autre_objectif.policies[0].chaine_repli[0].objectif_id = Uuid::new_v4();
    invalides.push(autre_objectif);

    for invalid in invalides {
        assert!(store.register_coordination_snapshot(&invalid).is_err());
        assert!(store.coordination_snapshot(objectif.id).unwrap().is_none());
    }
}

fn definition(
    objectif_id: Uuid,
    delegation_id: Uuid,
    dependencies: Vec<DependanceDelegation>,
    fallback: &str,
) -> DefinitionCoordination {
    DefinitionCoordination {
        objectif_id,
        dependencies,
        policies: vec![PolitiqueReassignation {
            delegation_id,
            objectif_id,
            classe: ClasseDuree::Normale,
            version: 1,
            seuil_relances: 2,
            max_reemissions: 2,
            chaine_repli: vec![FaitAppartenanceRepli {
                objectif_id,
                participant_id: fallback.into(),
                membership_version: 1,
                est_pilote: false,
            }],
        }],
        attentes: vec![AttenteNotification {
            attente_id: Uuid::new_v4(),
            objectif_id,
            delegation_id: None,
            kind: TypeEvenementCoordination::ClotureObjectif,
            recipient: "referent".into(),
            policy_version: 1,
        }],
    }
}

fn edge(objectif_id: Uuid, prerequis_id: Uuid, dependant_id: Uuid) -> DependanceDelegation {
    DependanceDelegation {
        objectif_id,
        prerequis_id,
        dependant_id,
        mode: ModeQualificationDependance::HashGreffe,
    }
}

fn create_delegation(
    store: &mut MaicieStore,
    objectif: &ObjectifCoordonne,
    participant: &str,
) -> Uuid {
    let delegation = Delegation::nouvelle(
        objectif.id,
        participant,
        "instruction",
        ClasseDuree::Normale,
        "raison",
    )
    .unwrap();
    let body = format!("travail pour {participant}").into_bytes();
    let outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: delegation.id,
        target: participant.into(),
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
        objectif.clone(),
        delegation.clone(),
        outbox,
        store.issuer_scope(),
        1_000,
        FRAME_LIMIT,
    )
    .unwrap();
    store.create_prepared_delegation(&prepared).unwrap();
    delegation.id
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("maicie-t1605-{name}-{}", Uuid::new_v4()));
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
