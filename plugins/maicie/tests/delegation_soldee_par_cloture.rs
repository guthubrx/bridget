//! Oracles : clôture d'objectif solde les délégations ouvertes
//! (`soldee_par_cloture`, jamais `terminee`) et terminalise leurs outboxes
//! expédiables dans la même transaction.

use maicie::domain::{
    ClasseDuree, Delegation, EtatDelegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif,
    ObjectifCoordonne, OutboxDelegation,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::store::{MaicieStore, ObjectiveClosureCommitPhase, SCHEMA_VERSION, StoreError};
use rusqlite::Connection;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

const FRAME_LIMIT: usize = 256 * 1024;

#[test]
fn cloture_solde_les_delegations_ouvertes_dans_la_meme_transaction() {
    let fixture = Fixture::new("cloture-solde");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_with_a_evaluer(&mut store, &fixture.database, "a-clore");
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);

    let failed =
        store.close_objective_observed(objective_id, "clôture atomique solde", 2_100, |phase| {
            if phase == ObjectiveClosureCommitPhase::AfterOutboxes {
                return Err(StoreError::Conflict("faute après solde délégations"));
            }
            Ok(())
        });
    assert!(matches!(
        failed,
        Err(StoreError::Conflict("faute après solde délégations"))
    ));
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_ne!(snapshot.objective.etat, EtatObjectif::Clos);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::AEvaluer);
    // Rollback complet : outbox encore expédiable tant que la clôture n'a pas commit.
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);

    store
        .close_objective(objective_id, "clôture atomique solde", 2_100)
        .unwrap();
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snapshot.objective.etat, EtatObjectif::Clos);
    assert_eq!(
        snapshot.delegations[0].etat,
        EtatDelegation::SoldeeParCloture
    );
    assert_ne!(snapshot.delegations[0].etat, EtatDelegation::Terminee);
    // Oracle greffe/action : soldée ⇒ plus aucune enveloppe expédiable.
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
}

#[test]
fn cloture_terminalise_outbox_sinon_oracle_rouge() {
    // Si `terminalize_dispatchable_outboxes_for_settled_delegation` est retiré
    // du solde, ce test meurt : délégation soldée + 1 pending = contradiction.
    let fixture = Fixture::new("cloture-outbox-terminale");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_objective(&mut store, "outbox-doit-mourir");
    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    let message_id = pending[0].message_id;
    let delegation_id = pending[0].delegation_id;

    store
        .close_objective(objective_id, "clôture terminalise outbox", 2_200)
        .unwrap();

    assert!(
        store.pending_delegation_outboxes().unwrap().is_empty(),
        "reprise ne doit plus sélectionner l'enveloppe d'une délégation soldée"
    );
    let connection = Connection::open(&fixture.database).unwrap();
    let (state, terminal): (String, i64) = connection
        .query_row(
            "SELECT state, terminal FROM delegation_outbox WHERE message_id = ?1",
            [message_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(state, "rejected");
    assert_eq!(terminal, 1);
    let delegation_state: String = connection
        .query_row(
            "SELECT state FROM delegations WHERE id = ?1",
            [delegation_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(delegation_state, "soldee_par_cloture");
}

#[test]
fn invariant_aucune_a_evaluer_sous_un_objectif_clos() {
    let fixture = Fixture::new("invariant-global-cloture");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let closed_id = seed_with_a_evaluer(&mut store, &fixture.database, "a-clore-global");
    let _open_id = seed_with_a_evaluer(&mut store, &fixture.database, "controle-positif-ouvert");

    store
        .close_objective(closed_id, "oracle invariant global", 2_300)
        .unwrap();

    let connection = Connection::open(&fixture.database).unwrap();
    let open_a_evaluer: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM delegations d
             JOIN objectives o ON o.id = d.objective_id
             WHERE o.state <> 'clos' AND d.state = 'a_evaluer'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let a_evaluer_under_closed: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM delegations d
             JOIN objectives o ON o.id = d.objective_id
             WHERE o.state = 'clos' AND d.state = 'a_evaluer'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        open_a_evaluer, 1,
        "contrôle positif : l'instrument doit voir la délégation encore ouverte"
    );
    assert_eq!(
        a_evaluer_under_closed, 0,
        "un objectif clos ne doit conserver aucune délégation a_evaluer"
    );
}

#[test]
fn cloture_preserve_terminee_et_annulee() {
    let fixture = Fixture::new("cloture-preserve-terminaux");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_objective(&mut store, "mixte");
    let first = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0)
        .delegations
        .remove(0);
    {
        let connection = Connection::open(&fixture.database).unwrap();
        force_delegation_state(&connection, &first, EtatDelegation::Terminee);
    }
    let second_id = insert_raw_delegation(&fixture.database, objective_id, "bob");
    {
        let connection = Connection::open(&fixture.database).unwrap();
        let second = load_delegation(&connection, second_id);
        force_delegation_state(&connection, &second, EtatDelegation::Annulee);
    }

    store
        .close_objective(objective_id, "clôture mixte", 3_000)
        .unwrap();
    let mut states: Vec<_> = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0)
        .delegations
        .into_iter()
        .map(|d| d.etat)
        .collect();
    states.sort_by_key(|etat| format!("{etat:?}"));
    assert_eq!(
        states,
        vec![EtatDelegation::Annulee, EtatDelegation::Terminee]
    );
}

#[test]
fn migration_v16_solde_les_orphelines_sur_objectifs_clos() {
    let fixture = Fixture::new("migration-v16-orphelines");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_with_a_evaluer(&mut store, &fixture.database, "deja-clos");
    store
        .close_objective(objective_id, "clôture avant downgrade", 4_000)
        .unwrap();
    {
        let connection = Connection::open(&fixture.database).unwrap();
        let delegation = store
            .objective_snapshots(Some(objective_id))
            .unwrap()
            .remove(0)
            .delegations
            .remove(0);
        // Simule une orpheline antérieure au correctif (objectif déjà clos).
        force_delegation_state(&connection, &delegation, EtatDelegation::AEvaluer);
        // Et une outbox encore expédiable (régression pré-correctif greffe/action).
        force_outbox_dispatchable(&connection, delegation.id);
        // Après routines (v15) : orphelines = v16. Downgrade juste avant.
        connection.pragma_update(None, "user_version", 15).unwrap();
        connection
            .execute("DELETE FROM schema_migrations WHERE version = 16", [])
            .unwrap();
    }
    drop(store);

    // Consentement explicite : open() refuse de migrer 15→16.
    let store = MaicieStore::open_and_migrate(&fixture.database).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snapshot.objective.etat, EtatObjectif::Clos);
    assert_eq!(
        snapshot.delegations[0].etat,
        EtatDelegation::SoldeeParCloture
    );
    assert_ne!(snapshot.delegations[0].etat, EtatDelegation::Terminee);
    assert!(
        store.pending_delegation_outboxes().unwrap().is_empty(),
        "migration v15 doit terminaliser l'outbox des orphelines soldées"
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
    objective.id
}

fn seed_with_a_evaluer(store: &mut MaicieStore, database: &Path, label: &str) -> Uuid {
    let objective_id = seed_objective(store, label);
    let delegation = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0)
        .delegations
        .remove(0);
    let connection = Connection::open(database).unwrap();
    force_delegation_state(&connection, &delegation, EtatDelegation::AEvaluer);
    objective_id
}

fn insert_raw_delegation(database: &Path, objective_id: Uuid, participant: &str) -> Uuid {
    let delegation = Delegation::nouvelle(
        objective_id,
        participant,
        "seconde instruction",
        ClasseDuree::Normale,
        "seconde raison",
    )
    .unwrap();
    let id = delegation.id;
    let payload = serde_json::to_vec(&delegation).unwrap();
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "INSERT INTO delegations(id, objective_id, state, payload_json)
             VALUES (?1, ?2, 'creee', ?3)",
            rusqlite::params![id.to_string(), objective_id.to_string(), payload],
        )
        .unwrap();
    id
}

fn load_delegation(connection: &Connection, id: Uuid) -> Delegation {
    let payload: Vec<u8> = connection
        .query_row(
            "SELECT payload_json FROM delegations WHERE id = ?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_slice(&payload).unwrap()
}

fn force_delegation_state(connection: &Connection, delegation: &Delegation, etat: EtatDelegation) {
    let mut forged = delegation.clone();
    forged.etat = etat;
    let payload = serde_json::to_vec(&forged).unwrap();
    let state = match etat {
        EtatDelegation::EnAttentePrerequis => "en_attente_prerequis",
        EtatDelegation::Creee => "creee",
        EtatDelegation::AEvaluer => "a_evaluer",
        EtatDelegation::Terminee => "terminee",
        EtatDelegation::Annulee => "annulee",
        EtatDelegation::SoldeeParCloture => "soldee_par_cloture",
    };
    connection
        .execute(
            "UPDATE delegations SET state = ?1, payload_json = ?2 WHERE id = ?3",
            rusqlite::params![state, payload, delegation.id.to_string()],
        )
        .unwrap();
}

fn force_outbox_dispatchable(connection: &Connection, delegation_id: Uuid) {
    connection
        .execute(
            "UPDATE delegation_outbox
             SET state = 'prepared', terminal = 0, last_issue_json = NULL,
                 issue_observed_at = NULL
             WHERE delegation_id = ?1",
            [delegation_id.to_string()],
        )
        .unwrap();
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("maicie-solde-cloture-{name}-{}", Uuid::new_v4()));
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
