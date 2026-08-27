//! Oracles F36 (suite obligatoire) + F37 (classement + déblocage à la clôture).

use maicie::app::{
    DelegateError, DelegateRequest, DelegateResult, DelegationCandidate, close, delegate,
};
use maicie::config::DurationClasses;
use maicie::domain::{ClasseDuree, EtatDelegation, MotifRefusDelegationLocale, SuiteObjective};
use maicie::store::MaicieStore;
use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-f36-f37-{label}-{}", Uuid::new_v4()))
}

fn durations() -> DurationClasses {
    DurationClasses {
        short_secs: 30,
        normal_secs: 60,
        long_secs: 90,
    }
}

fn candidate(name: &str) -> DelegationCandidate {
    DelegationCandidate {
        name: name.to_string(),
        tags: Vec::new(),
        available: true,
        dnd: false,
    }
}

fn base_request<'a>(
    goal: &'a str,
    suite: SuiteObjective,
    depends_on: &'a [Uuid],
    references: &'a [Uuid],
    key: &'a str,
) -> DelegateRequest<'a> {
    DelegateRequest {
        goal,
        explicit_target: Some("prospective"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: false,
        constat_id: None,
        review_target: None,
        suite,
        depends_on,
        references,
        idempotency_key: key,
        now: 1_787_570_000,
        retry_until: 1_787_570_100,
        dedup_retained_until: 1_787_570_200,
        max_frame_bytes: 256 * 1024,
    }
}

fn create_seed(store: &mut MaicieStore, goal: &str, key: &str) -> Uuid {
    let result = delegate(
        store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(goal, SuiteObjective::Aucune, &[], &[], key),
    )
    .unwrap();
    let DelegateResult::Created(created) = result else {
        panic!("création attendue");
    };
    created.objective_id
}

#[test]
fn f37_goal_citant_objectif_connu_sans_classement_refuse() {
    let root = root("cite-sans-classement");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let known = create_seed(&mut store, "prérequis", "seed-1");
    let goal = format!("enchaîne sur {known}");
    let error = delegate(
        &mut store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(&goal, SuiteObjective::Aucune, &[], &[], "cite-1"),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        DelegateError::ContrainteRefusee {
            motif: MotifRefusDelegationLocale::SuiteAucuneAvecCitationNonClassee,
            objectif_cite,
            refus_durables: 1,
        } if objectif_cite == known
    ));

    // Une seconde invocation est un second refus : chacune laisse sa propre
    // ligne, même si aucun objectif ni outbox n'a été créé entre les deux.
    let second = delegate(
        &mut store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(&goal, SuiteObjective::Aucune, &[], &[], "cite-2"),
    )
    .unwrap_err();
    assert!(matches!(
        second,
        DelegateError::ContrainteRefusee {
            motif: MotifRefusDelegationLocale::SuiteAucuneAvecCitationNonClassee,
            objectif_cite,
            refus_durables: 2,
        } if objectif_cite == known
    ));
    assert_eq!(
        store
            .local_delegate_refusal_counts()
            .unwrap()
            .suite_aucune_avec_citation_non_classee,
        2
    );
    drop(store);

    let reopened = MaicieStore::open(&database).unwrap();
    assert_eq!(
        reopened
            .local_delegate_refusal_counts()
            .unwrap()
            .suite_aucune_avec_citation_non_classee,
        2,
        "le compteur survit à la fermeture du processus"
    );
    drop(reopened);

    let connection = Connection::open(&database).unwrap();
    let rows = connection
        .prepare("SELECT reason, cited_objective_id FROM local_delegate_refusals ORDER BY sequence")
        .unwrap()
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        rows,
        vec![
            (
                "suite_none_with_unclassified_citation".to_string(),
                known.to_string(),
            ),
            (
                "suite_none_with_unclassified_citation".to_string(),
                known.to_string(),
            ),
        ],
        "exactement une ligne typée par refus"
    );
    assert!(
        connection
            .execute(
                "INSERT INTO local_delegate_refusals(observed_at, reason, cited_objective_id)
                 VALUES (?1, ?2, ?3)",
                params![1_787_570_001_i64, "raison_libre", known.to_string()],
            )
            .is_err(),
        "le schéma refuse tout motif hors de l'ensemble fermé"
    );
    assert!(
        connection
            .execute(
                "UPDATE local_delegate_refusals SET observed_at = observed_at + 1",
                [],
            )
            .is_err(),
        "le journal des refus est append-only"
    );
    assert!(
        connection
            .execute("DELETE FROM local_delegate_refusals", [])
            .is_err(),
        "une ligne de refus durable ne peut pas être supprimée"
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn f37_reference_accepte_sans_creer_d_arete() {
    let root = root("reference-sans-arete");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let known = create_seed(&mut store, "contexte", "seed-ref");
    let goal = format!("contexte seulement {known}");
    let refs = [known];
    let DelegateResult::Created(created) = delegate(
        &mut store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(&goal, SuiteObjective::Aucune, &[], &refs, "ref-1"),
    )
    .unwrap() else {
        panic!("création attendue");
    };
    assert!(!created.waiting_on_prerequisites);
    assert!(created.message_id.is_some());
    assert!(store.dependents_of_objective(known).unwrap().is_empty());
    let snap = store
        .objective_snapshots(Some(created.objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snap.objective.references, vec![known]);
    assert!(snap.objective.depends_on.is_empty());
    assert_eq!(
        store
            .local_delegate_refusal_counts()
            .unwrap()
            .suite_aucune_avec_citation_non_classee,
        0,
        "une citation classée reste cohérente sans ajouter de champ libre"
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn f37_depends_on_cree_arete_et_deblocage_a_la_cloture() {
    let root = root("depends-deblocage");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let prereq = create_seed(&mut store, "lot amont", "seed-dep");
    let goal = format!("dépend de {prereq}");
    let deps = [prereq];
    let DelegateResult::Created(created) = delegate(
        &mut store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(&goal, SuiteObjective::Aucune, &deps, &[], "dep-1"),
    )
    .unwrap() else {
        panic!("création attendue");
    };
    assert!(created.waiting_on_prerequisites);
    assert!(created.message_id.is_none());
    assert_eq!(
        store.dependents_of_objective(prereq).unwrap(),
        vec![created.objective_id]
    );
    assert!(
        store
            .pending_delegation_outboxes()
            .unwrap()
            .iter()
            .all(|outbox| outbox.delegation_id != created.delegation_id),
        "aucune outbox pour la délégation en attente"
    );
    let waiting = store
        .objective_snapshots(Some(created.objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(
        waiting.delegations[0].etat,
        EtatDelegation::EnAttentePrerequis
    );

    close(&mut store, prereq, "lot amont livré", 1_787_570_050).unwrap();

    let released = store
        .objective_snapshots(Some(created.objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(released.delegations[0].etat, EtatDelegation::Creee);
    let pending = store.pending_delegation_outboxes().unwrap();
    let outbox = pending
        .iter()
        .find(|outbox| outbox.delegation_id == created.delegation_id)
        .expect("outbox créée au déblocage");
    let message = outbox.public_message().unwrap();
    let expected_body = format!(
        "{goal}\n\nIDENTIFIANTS DU MANDAT\n\
         objective_id  : {}\n\
         delegation_id : {}\n\
         message_id    : {}",
        created.objective_id, created.delegation_id, outbox.message_id
    );
    assert_eq!(message.id, outbox.message_id.to_string());
    assert_eq!(message.body, expected_body);
    let notifications = store.pending_notification_outboxes().unwrap();
    assert!(
        notifications
            .iter()
            .any(|n| n.recipient == "prospective" && n.objectif_id == created.objective_id),
        "notification de déblocage attendue"
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn f37_cycle_mutuel_refuse_sans_seconde_arete() {
    let root = root("cycle-ab");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let a = create_seed(&mut store, "lot A", "seed-cycle-a");
    let b = create_seed(&mut store, "lot B", "seed-cycle-b");
    store.register_objective_dependencies(a, &[b]).unwrap();
    let error = store
        .register_objective_dependencies(b, &[a])
        .expect_err("cycle A↔B");
    assert!(
        matches!(error, maicie::store::StoreError::Invalid(reason) if reason.contains("cycle")),
        "{error:?}"
    );
    assert_eq!(
        store.dependents_of_objective(a).unwrap(),
        Vec::<Uuid>::new()
    );
    assert_eq!(store.dependents_of_objective(b).unwrap(), vec![a]);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn f37_uuid_inconnu_ignore_sans_refus() {
    let root = root("uuid-inconnu");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let ghost = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let goal = format!("hash-like {ghost} dans le mandat");
    let result = delegate(
        &mut store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(&goal, SuiteObjective::Aucune, &[], &[], "ghost-1"),
    )
    .unwrap();
    assert!(matches!(result, DelegateResult::Created(_)));
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn f37_cloture_sans_dependant_zero_bruit() {
    let root = root("cloture-silencieuse");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let alone = create_seed(&mut store, "sans suite", "seed-alone");
    assert!(store.dependents_of_objective(alone).unwrap().is_empty());
    let before_notifications = store.pending_notification_outboxes().unwrap().len();
    let before_outboxes = store.pending_delegation_outboxes().unwrap().len();
    assert!(
        before_outboxes >= 1,
        "la graine crée au moins une enveloppe préparée"
    );
    close(&mut store, alone, "clôture isolée", 1_787_570_060).unwrap();
    assert_eq!(
        store.pending_notification_outboxes().unwrap().len(),
        before_notifications,
        "aucune notification F37 inventée"
    );
    // Pas de bruit de déblocage : aucune nouvelle enveloppe. La mission initiale
    // n'est plus expédiable (soldée + outbox terminale, même transaction).
    assert!(
        store.pending_delegation_outboxes().unwrap().is_empty(),
        "aucune outbox de déblocage inventée ; reprise vide après solde"
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn f36_suite_journalisee_sur_objectif() {
    let root = root("suite-journal");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let next = create_seed(&mut store, "prochaine étape", "seed-suite");
    let DelegateResult::Created(created) = delegate(
        &mut store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &base_request(
            "travail courant",
            SuiteObjective::Objectif(next),
            &[],
            &[],
            "suite-1",
        ),
    )
    .unwrap() else {
        panic!("création attendue");
    };
    let snap = store
        .objective_snapshots(Some(created.objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snap.objective.suite, Some(SuiteObjective::Objectif(next)));
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
