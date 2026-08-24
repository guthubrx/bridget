use bridget_transport::protocol::WrapperToDaemon;
use maicie::bridget_client::{GuichetClaim, PublicMessage};
use maicie::domain::guichet::{RequeteGuichet, parse_claim};
use maicie::domain::{
    AttenteNotification, ClasseDuree, DecisionCoordination, DefinitionCoordination, Delegation,
    DependanceDelegation, EntreeReductionCoordination, EtatDecision, EtatGenerationDelegation,
    EtatObjectif, EtatOutboxDelegation, EvaluationCloture, EvenementCoordination,
    FaitAppartenanceRepli, FaitReassignation, FraicheurCoordination, GenerationDelegation,
    IssueClotureEvaluee, LotReassignation, ModeObjectif, ModeQualificationDependance,
    ObjectifCoordonne, OutboxDelegation, PolitiqueReassignation, TypeDecision,
    TypeEvenementAttendu, TypeFaitReassignation,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::store::{
    CoordinationCommitPhase, GuichetCommitPhase, MaicieStore, ObjectiveClosureCommitPhase,
    ReassignmentCommitPhase, StoreError,
};
use rusqlite::{Connection, ErrorCode, params};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

const FRAME_LIMIT: usize = 256 * 1024;

#[test]
fn migration_v8_main_vers_v11_puis_seconde_ouverture_conservent_l_historique() {
    let fixture = Fixture::new("migration-main-v8");
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
             DROP TABLE tracked_request_outbox;
             DROP TABLE reassignment_reductions;
             DROP TABLE reassignment_events;
             DROP TABLE evaluated_closure_acts;
             DROP TABLE delegation_dependencies;
             DROP TABLE coordination_expectations;
             DROP TABLE coordination_events;",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO guichet_refusal_receptions(
                 issuer_scope,request_id,canonical_request_bytes,operation,reason,
                 response_message_id,reply_bytes,claim_generation,claim_token,processed_at
             ) VALUES (?1,?2,?3,'mission_status','delegation_missing',?4,?5,1,?6,?7)",
            params![
                "scope-main-v8-0123456789abcdef",
                "request-main-v8",
                b"canon-main-v8".as_slice(),
                "response-main-v8",
                b"reply-main-v8".as_slice(),
                "claim-main-v8-0123456789abcdef",
                1_787_501_001_i64,
            ],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 8).unwrap();
    connection
        .execute(
            "DELETE FROM schema_migrations WHERE version IN (9, 10, 11)",
            [],
        )
        .unwrap();
    drop(connection);

    let store = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 11);
    assert_eq!(
        store.objective_snapshots(Some(objectif.id)).unwrap()[0].delegations[0].id,
        delegation
    );
    drop(store);
    let connection = Connection::open(&fixture.database).unwrap();
    let coordination_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                 'coordination_events', 'coordination_expectations',
                 'delegation_dependencies', 'evaluated_closure_acts',
                 'reassignment_policies', 'delegation_lineages',
                 'delegation_generations', 'reminder_episodes',
                 'active_coordination_decisions', 'notification_outbox',
                 'reassignment_events', 'reassignment_reductions',
                 'tracked_request_outbox'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(coordination_tables, 13);
    let refusal: (String, Vec<u8>) = connection
        .query_row(
            "SELECT reason, reply_bytes FROM guichet_refusal_receptions
             WHERE issuer_scope='scope-main-v8-0123456789abcdef'
               AND request_id='request-main-v8'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        refusal,
        ("delegation_missing".to_string(), b"reply-main-v8".to_vec())
    );
    drop(connection);
    let reopened = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 11);
    assert_eq!(
        reopened
            .objective_snapshots(Some(objectif.id))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn migration_v9_refuse_de_rejouer_une_notification_sans_horodatage_atteste() {
    let fixture = Fixture::new("migration-v9-notification");
    drop(MaicieStore::open(&fixture.database).unwrap());
    let connection = Connection::open(&fixture.database).unwrap();
    connection
        .execute_batch(
            "DROP TABLE notification_outbox;
             CREATE TABLE notification_outbox (
                 message_id TEXT PRIMARY KEY,
                 idempotency_key TEXT NOT NULL UNIQUE,
                 objective_id TEXT NOT NULL,
                 delegation_id TEXT,
                 generation INTEGER,
                 event_id TEXT NOT NULL,
                 policy_version INTEGER NOT NULL,
                 recipient TEXT NOT NULL,
                 message_bytes BLOB NOT NULL,
                 state TEXT NOT NULL,
                 last_issue_json BLOB,
                 terminal INTEGER NOT NULL
             );",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO notification_outbox(
                 message_id,idempotency_key,objective_id,delegation_id,generation,
                 event_id,policy_version,recipient,message_bytes,state,terminal
             ) VALUES (?1,?2,?3,NULL,NULL,?4,1,?5,?6,'prepared',0)",
            params![
                Uuid::new_v4().to_string(),
                "legacy-notification-key",
                Uuid::new_v4().to_string(),
                "legacy-event",
                "alice",
                b"octets-historiques".as_slice(),
            ],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 9).unwrap();
    connection
        .execute(
            "DELETE FROM schema_migrations WHERE version IN (10, 11)",
            [],
        )
        .unwrap();
    drop(connection);

    let store = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 11);
    assert!(matches!(
        store.pending_notification_outboxes(),
        Err(StoreError::Corrupt(
            "notification historique sans issued_at"
        ))
    ));
    drop(store);

    let connection = Connection::open(&fixture.database).unwrap();
    let issued_at: Option<i64> = connection
        .query_row("SELECT issued_at FROM notification_outbox", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(issued_at, None);
}

#[test]
fn cent_clotures_a_trois_destinataires_produisent_trois_cents_cles_uniques() {
    let fixture = Fixture::new("clotures-300");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    for index in 0..100 {
        let objective_id = seed_with_closure_recipients(&mut store, index, 3);
        store
            .close_objective(objective_id, "clôture explicite", 1_787_501_000 + index)
            .unwrap();
    }
    let pending = store.pending_notification_outboxes().unwrap();
    assert_eq!(pending.len(), 300);
    assert_eq!(
        pending
            .iter()
            .map(|outbox| outbox.message_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        300
    );
    assert_eq!(
        pending
            .iter()
            .map(|outbox| outbox.idempotency_key.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        300
    );
    assert!(pending.iter().all(|outbox| outbox.issued_at > 0));
}

#[test]
fn cloture_et_notifications_sont_une_seule_transaction_rejouable() {
    let fixture = Fixture::new("cloture-rollback");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_with_closure_recipients(&mut store, 1, 3);
    let failed =
        store.close_objective_observed(objective_id, "clôture atomique", 1_787_502_000, |phase| {
            if phase == ObjectiveClosureCommitPhase::AfterOutboxes {
                return Err(StoreError::Conflict("faute après notifications"));
            }
            Ok(())
        });
    assert!(matches!(
        failed,
        Err(StoreError::Conflict("faute après notifications"))
    ));
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_ne!(snapshot.objective.etat, EtatObjectif::Clos);
    assert!(snapshot.decisions.is_empty());
    assert!(store.pending_notification_outboxes().unwrap().is_empty());

    store
        .close_objective(objective_id, "clôture atomique", 1_787_502_000)
        .unwrap();
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snapshot.objective.etat, EtatObjectif::Clos);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 3);
}

#[test]
fn toute_cloture_hors_primitive_est_refusee_et_un_texte_termine_ne_notifie_pas() {
    let fixture = Fixture::new("cloture-bypass");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let objective_id = seed_with_closure_recipients(&mut store, 1, 1);
    let snapshot = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    let mut forged = snapshot.objective.clone();
    forged.clore(1_787_503_000).unwrap();
    let close_decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind: TypeDecision::Cloturer,
        proposee_par: "maicie".to_string(),
        etat: EtatDecision::Appliquee,
        motif: "contournement tenté".to_string(),
    };
    assert!(matches!(
        store.apply_objective_decision(&close_decision, Some(&forged), snapshot.objective.etat,),
        Err(StoreError::Invalid("clôture réservée à close_objective"))
    ));

    let text_only = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind: TypeDecision::AjouterParticipant,
        proposee_par: "maicie".to_string(),
        etat: EtatDecision::Appliquee,
        motif: "travail terminé".to_string(),
    };
    store
        .apply_objective_decision(&text_only, None, snapshot.objective.etat)
        .unwrap();
    let after = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_ne!(after.objective.etat, EtatObjectif::Clos);
    assert_eq!(after.decisions, vec![text_only]);
    assert!(store.pending_notification_outboxes().unwrap().is_empty());

    let mut terminal_at_creation =
        ObjectifCoordonne::nouveau("terminal forgé", ModeObjectif::Delegue, 1_787_503_100).unwrap();
    terminal_at_creation.clore(1_787_503_101).unwrap();
    let delegation = Delegation::nouvelle(
        terminal_at_creation.id,
        "alice",
        "instruction",
        ClasseDuree::Normale,
        "raison",
    )
    .unwrap();
    let body = b"message".to_vec();
    let outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: delegation.id,
        target: "alice".to_string(),
        body_hash: stable_body_hash(&body),
        body_bytes: body,
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_787_503_200,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_787_503_150,
        dedup_retained_until: 1_787_503_300,
    };
    let prepared = PreparedDelegation::new(
        terminal_at_creation.clone(),
        delegation,
        outbox,
        store.issuer_scope(),
        1_787_503_110,
        FRAME_LIMIT,
    );
    assert!(prepared.is_err());
    assert!(
        store
            .objective_snapshots(Some(terminal_at_creation.id))
            .unwrap()
            .is_empty()
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
fn losange_ouvre_et_notifie_une_fois_apres_le_dernier_prerequis() {
    let fixture = Fixture::new("f28-losange");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut objectif = ObjectifCoordonne::nouveau("losange", ModeObjectif::Delegue, 1).unwrap();
    objectif
        .transition(EtatObjectif::EnCoordination, 2)
        .unwrap();
    let a = create_delegation(&mut store, &objectif, "alice");
    let b = create_delegation(&mut store, &objectif, "bob");
    let dependant = create_delegation(&mut store, &objectif, "carol");
    let request_a = initial_request_id(&store, a);
    let request_b = initial_request_id(&store, b);
    let snapshot = definition(
        objectif.id,
        a,
        vec![
            edge(objectif.id, a, dependant),
            edge(objectif.id, b, dependant),
        ],
        "bob",
    );
    store.register_coordination_snapshot(&snapshot).unwrap();

    graft_delivery(
        &mut store,
        DeliveryFixture {
            objective_id: objectif.id,
            delegation_id: a,
            participant: "alice",
            in_reply_to: &request_a,
            request_id: "f28-report-a",
            response_id: "f28-response-a",
            hash: &"aa".repeat(32),
            now: 1_787_500_100,
        },
    );
    let after_first = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(
        active_state(&after_first, dependant),
        EtatGenerationDelegation::Bloquee
    );
    assert!(store.pending_notification_outboxes().unwrap().is_empty());

    let second = DeliveryFixture {
        objective_id: objectif.id,
        delegation_id: b,
        participant: "bob",
        in_reply_to: &request_b,
        request_id: "f28-report-b",
        response_id: "f28-response-b",
        hash: &"bb".repeat(32),
        now: 1_787_500_101,
    };
    graft_delivery(&mut store, second);
    graft_delivery(&mut store, second);
    let opened = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(
        active_state(&opened, dependant),
        EtatGenerationDelegation::Ouverte
    );
    let notifications = store.pending_notification_outboxes().unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].recipient, "carol");
    drop(store);
    let connection = Connection::open(&fixture.database).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM active_coordination_decisions WHERE kind='ouvrir'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
}

#[test]
fn chaine_f28_ouvre_les_trois_missions_dans_l_ordre_declare() {
    let fixture = Fixture::new("f28-chaine");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut objectif = ObjectifCoordonne::nouveau("chaîne", ModeObjectif::Delegue, 1).unwrap();
    objectif
        .transition(EtatObjectif::EnCoordination, 2)
        .unwrap();
    let a = create_delegation(&mut store, &objectif, "alice");
    let b = create_delegation(&mut store, &objectif, "bob");
    let c = create_delegation(&mut store, &objectif, "carol");
    let request_a = initial_request_id(&store, a);
    let request_b = initial_request_id(&store, b);
    store
        .register_coordination_snapshot(&definition(
            objectif.id,
            a,
            vec![edge(objectif.id, a, b), edge(objectif.id, b, c)],
            "bob",
        ))
        .unwrap();

    graft_delivery(
        &mut store,
        DeliveryFixture {
            objective_id: objectif.id,
            delegation_id: a,
            participant: "alice",
            in_reply_to: &request_a,
            request_id: "f28-chain-a",
            response_id: "f28-chain-response-a",
            hash: &"78".repeat(32),
            now: 1_787_500_100,
        },
    );
    let first = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(active_state(&first, b), EtatGenerationDelegation::Ouverte);
    assert_eq!(active_state(&first, c), EtatGenerationDelegation::Bloquee);
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 1);

    graft_delivery(
        &mut store,
        DeliveryFixture {
            objective_id: objectif.id,
            delegation_id: b,
            participant: "bob",
            in_reply_to: &request_b,
            request_id: "f28-chain-b",
            response_id: "f28-chain-response-b",
            hash: &"9a".repeat(32),
            now: 1_787_500_101,
        },
    );
    let second = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(active_state(&second, b), EtatGenerationDelegation::Ouverte);
    assert_eq!(active_state(&second, c), EtatGenerationDelegation::Ouverte);
    let notifications = store.pending_notification_outboxes().unwrap();
    assert_eq!(notifications.len(), 2);
    let mut recipients = notifications
        .iter()
        .map(|notification| notification.recipient.as_str())
        .collect::<Vec<_>>();
    recipients.sort_unstable();
    assert_eq!(recipients, vec!["bob", "carol"]);
}

#[test]
fn deux_derniers_prerequis_concurrents_ouvrent_un_seul_dependant() {
    let fixture = Fixture::new("f28-losange-concurrent");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut objectif =
        ObjectifCoordonne::nouveau("losange concurrent", ModeObjectif::Delegue, 1).unwrap();
    objectif
        .transition(EtatObjectif::EnCoordination, 2)
        .unwrap();
    let a = create_delegation(&mut store, &objectif, "alice");
    let b = create_delegation(&mut store, &objectif, "bob");
    let dependant = create_delegation(&mut store, &objectif, "carol");
    let request_a = initial_request_id(&store, a);
    let request_b = initial_request_id(&store, b);
    store
        .register_coordination_snapshot(&definition(
            objectif.id,
            a,
            vec![
                edge(objectif.id, a, dependant),
                edge(objectif.id, b, dependant),
            ],
            "bob",
        ))
        .unwrap();
    drop(store);

    let claim_a = delivery_claim(DeliveryFixture {
        objective_id: objectif.id,
        delegation_id: a,
        participant: "alice",
        in_reply_to: &request_a,
        request_id: "f28-concurrent-a",
        response_id: "f28-concurrent-response-a",
        hash: &"34".repeat(32),
        now: 1_787_500_100,
    });
    let claim_b = delivery_claim(DeliveryFixture {
        objective_id: objectif.id,
        delegation_id: b,
        participant: "bob",
        in_reply_to: &request_b,
        request_id: "f28-concurrent-b",
        response_id: "f28-concurrent-response-b",
        hash: &"56".repeat(32),
        now: 1_787_500_101,
    });
    let database_a = fixture.database.clone();
    let (locked_tx, locked_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let first = thread::spawn(move || {
        let canonical = parse_claim(&claim_a).unwrap();
        let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
            panic!("rapport attendu")
        };
        let mut store = MaicieStore::open(database_a).unwrap();
        store.graft_delivery_report_observed(
            &claim_a,
            &canonical,
            report,
            "f28-concurrent-response-a",
            1_787_500_100,
            |phase| {
                if phase == GuichetCommitPhase::AfterDecisionInsert {
                    locked_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                }
                Ok(())
            },
        )
    });
    locked_rx.recv_timeout(Duration::from_secs(3)).unwrap();

    let database_b = fixture.database.clone();
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let second = thread::spawn(move || {
        let canonical = parse_claim(&claim_b).unwrap();
        let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
            panic!("rapport attendu")
        };
        let mut store = MaicieStore::open(database_b).unwrap();
        result_tx
            .send(store.graft_delivery_report(
                &claim_b,
                &canonical,
                report,
                "f28-concurrent-response-b",
                1_787_500_101,
            ))
            .unwrap();
    });
    assert!(result_rx.recv_timeout(Duration::from_millis(100)).is_err());
    release_tx.send(()).unwrap();
    first.join().unwrap().unwrap();
    result_rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();
    second.join().unwrap();

    let store = MaicieStore::open(&fixture.database).unwrap();
    let snapshot = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(
        active_state(&snapshot, dependant),
        EtatGenerationDelegation::Ouverte
    );
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 1);
    drop(store);
    let connection = Connection::open(&fixture.database).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM active_coordination_decisions WHERE kind='ouvrir'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
}

#[test]
fn mode_strict_attend_l_acte_evalue_portant_le_hash_greffe() {
    let fixture = Fixture::new("f28-strict");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut objectif = ObjectifCoordonne::nouveau("strict", ModeObjectif::Delegue, 1).unwrap();
    objectif
        .transition(EtatObjectif::EnCoordination, 2)
        .unwrap();
    let prerequis = create_delegation(&mut store, &objectif, "alice");
    let dependant = create_delegation(&mut store, &objectif, "bob");
    let request_id = initial_request_id(&store, prerequis);
    let mut strict = edge(objectif.id, prerequis, dependant);
    strict.mode = ModeQualificationDependance::ClotureEvalueeExigee;
    store
        .register_coordination_snapshot(&definition(objectif.id, prerequis, vec![strict], "bob"))
        .unwrap();
    let delivery_hash = "cd".repeat(32);
    graft_delivery(
        &mut store,
        DeliveryFixture {
            objective_id: objectif.id,
            delegation_id: prerequis,
            participant: "alice",
            in_reply_to: &request_id,
            request_id: "f28-report-strict",
            response_id: "f28-response-strict",
            hash: &delivery_hash,
            now: 1_787_500_100,
        },
    );
    let hash_only = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(
        active_state(&hash_only, dependant),
        EtatGenerationDelegation::Bloquee
    );
    assert!(store.pending_notification_outboxes().unwrap().is_empty());

    store
        .apply_coordination_reduction(&EntreeReductionCoordination::ClotureEvaluee(
            EvaluationCloture {
                event_id: "f28-evaluation-strict".to_string(),
                objectif_id: objectif.id,
                delegation_id: prerequis,
                generation: 1,
                delivery_hash,
                issue: IssueClotureEvaluee::LivraisonValidee,
                evaluated_at: 1_787_500_101,
            },
        ))
        .unwrap();
    let evaluated = store.coordination_snapshot(objectif.id).unwrap().unwrap();
    assert_eq!(
        active_state(&evaluated, dependant),
        EtatGenerationDelegation::Ouverte
    );
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 1);
}

#[test]
fn declaration_f28_apres_un_fait_qualifiant_est_refusee_sans_snapshot() {
    let fixture = Fixture::new("f28-retroactive");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut objectif = ObjectifCoordonne::nouveau("rétroactif", ModeObjectif::Delegue, 1).unwrap();
    objectif
        .transition(EtatObjectif::EnCoordination, 2)
        .unwrap();
    let prerequis = create_delegation(&mut store, &objectif, "alice");
    let dependant = create_delegation(&mut store, &objectif, "bob");
    let request_id = initial_request_id(&store, prerequis);
    graft_delivery(
        &mut store,
        DeliveryFixture {
            objective_id: objectif.id,
            delegation_id: prerequis,
            participant: "alice",
            in_reply_to: &request_id,
            request_id: "f28-report-before-definition",
            response_id: "f28-response-before-definition",
            hash: &"ef".repeat(32),
            now: 1_787_500_100,
        },
    );
    let retroactive = definition(
        objectif.id,
        prerequis,
        vec![edge(objectif.id, prerequis, dependant)],
        "bob",
    );
    assert!(matches!(
        store.register_coordination_snapshot(&retroactive),
        Err(StoreError::Invalid(
            "dépendance déclarée après fait qualifiant"
        ))
    ));
    assert!(store.coordination_snapshot(objectif.id).unwrap().is_none());
}

#[test]
fn fautes_f28_annulent_ensemble_recu_decision_ouverture_et_notification() {
    for phase in [
        GuichetCommitPhase::AfterDependentDecision,
        GuichetCommitPhase::AfterDependentTransition,
        GuichetCommitPhase::AfterDependentOutbox,
    ] {
        let fixture = Fixture::new(&format!("f28-rollback-{phase:?}"));
        let mut store = MaicieStore::open(&fixture.database).unwrap();
        let mut objectif =
            ObjectifCoordonne::nouveau("rollback", ModeObjectif::Delegue, 1).unwrap();
        objectif
            .transition(EtatObjectif::EnCoordination, 2)
            .unwrap();
        let prerequis = create_delegation(&mut store, &objectif, "alice");
        let dependant = create_delegation(&mut store, &objectif, "bob");
        let request_id = initial_request_id(&store, prerequis);
        store
            .register_coordination_snapshot(&definition(
                objectif.id,
                prerequis,
                vec![edge(objectif.id, prerequis, dependant)],
                "bob",
            ))
            .unwrap();
        let claim = delivery_claim(DeliveryFixture {
            objective_id: objectif.id,
            delegation_id: prerequis,
            participant: "alice",
            in_reply_to: &request_id,
            request_id: "f28-report-rollback",
            response_id: "f28-response-rollback",
            hash: &"12".repeat(32),
            now: 1_787_500_100,
        });
        let canonical = parse_claim(&claim).unwrap();
        let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
            panic!("rapport attendu")
        };
        let failed = store.graft_delivery_report_observed(
            &claim,
            &canonical,
            report,
            "f28-response-rollback",
            1_787_500_100,
            |observed| {
                if observed == phase {
                    return Err(StoreError::Conflict("faute F28 injectée"));
                }
                Ok(())
            },
        );
        assert!(matches!(
            failed,
            Err(StoreError::Conflict("faute F28 injectée"))
        ));
        drop(store);
        let connection = Connection::open(&fixture.database).unwrap();
        assert_eq!(table_count(&connection, "guichet_receptions"), 0);
        assert_eq!(table_count(&connection, "active_coordination_decisions"), 0);
        assert_eq!(table_count(&connection, "notification_outbox"), 0);
        let state: String = connection
            .query_row(
                "SELECT state FROM delegation_generations WHERE delegation_id=?1",
                [dependant.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "bloquee");
    }
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

#[test]
fn reduction_est_idempotente_et_l_acte_evalue_est_produit_dans_la_transaction() {
    let fixture = Fixture::new("reduction-idempotente");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
    let input = evaluated_input(objectif_id, delegation_id, "evaluation-idempotente");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let first = store.apply_coordination_reduction(&input).unwrap();
    let replay = store.apply_coordination_reduction(&input).unwrap();
    assert!(!first.replayed);
    assert!(replay.replayed);
    assert_eq!(first.reduction, replay.reduction);
    drop(store);

    let connection = Connection::open(&fixture.database).unwrap();
    assert_eq!(table_count(&connection, "active_coordination_decisions"), 1);
    assert_eq!(table_count(&connection, "evaluated_closure_acts"), 1);
    assert_eq!(table_count(&connection, "notification_outbox"), 0);
}

#[test]
fn faute_entre_decision_et_transition_annule_toutes_les_ecritures() {
    for phase in [
        CoordinationCommitPhase::AfterDecisionInsert,
        CoordinationCommitPhase::AfterTransition,
    ] {
        let fixture = Fixture::new(&format!("rollback-{phase:?}"));
        let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
        let input = evaluated_input(objectif_id, delegation_id, &format!("event-{phase:?}"));
        let mut store = MaicieStore::open(&fixture.database).unwrap();
        let result = store.apply_coordination_reduction_observed(&input, |observed| {
            if observed == phase {
                return Err(StoreError::Conflict("faute transactionnelle injectée"));
            }
            Ok(())
        });
        assert!(matches!(
            result,
            Err(StoreError::Conflict("faute transactionnelle injectée"))
        ));
        drop(store);

        let connection = Connection::open(&fixture.database).unwrap();
        for table in [
            "coordination_events",
            "active_coordination_decisions",
            "evaluated_closure_acts",
            "notification_outbox",
        ] {
            assert_eq!(
                table_count(&connection, table),
                0,
                "écriture partielle visible dans {table} à {phase:?}"
            );
        }
    }
}

#[test]
fn deux_ecrivains_se_concurrencent_reellement_et_un_seul_commit_gagne() {
    let fixture = Fixture::new("course-reelle");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "codex-1");
    let input = EntreeReductionCoordination::EvenementAtteste {
        objectif_id,
        delegation_id,
        generation: 1,
        evenement: fixture_event(FraicheurCoordination::Fresh),
    };
    let mut first_store = MaicieStore::open(&fixture.database).unwrap();
    let second_store = MaicieStore::open(&fixture.database).unwrap();
    let first_input = input.clone();
    let second_input = input.clone();
    let (locked_tx, locked_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let first = thread::spawn(move || {
        first_store.apply_coordination_reduction_observed(&first_input, |phase| {
            if phase == CoordinationCommitPhase::AfterDecisionInsert {
                locked_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            }
            Ok(())
        })
    });
    locked_rx.recv_timeout(Duration::from_secs(3)).unwrap();

    let (attempt_tx, attempt_rx) = mpsc::sync_channel(1);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let second = thread::spawn(move || {
        let mut second_store = second_store;
        attempt_tx.send(()).unwrap();
        result_tx
            .send(second_store.apply_coordination_reduction(&second_input))
            .unwrap();
    });
    attempt_rx.recv_timeout(Duration::from_secs(3)).unwrap();

    let locked_connection = Connection::open(&fixture.database).unwrap();
    locked_connection.busy_timeout(Duration::ZERO).unwrap();
    let busy = locked_connection.execute(
        "UPDATE objectives SET state = state WHERE id = ?1",
        [objectif_id.to_string()],
    );
    assert!(matches!(
        busy,
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == ErrorCode::DatabaseBusy
    ));
    assert!(result_rx.recv_timeout(Duration::from_millis(100)).is_err());

    release_tx.send(()).unwrap();
    let first_result = first.join().unwrap().unwrap();
    let second_result = result_rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();
    second.join().unwrap();
    assert_ne!(first_result.replayed, second_result.replayed);
    assert_eq!(first_result.reduction, second_result.reduction);

    let connection = Connection::open(&fixture.database).unwrap();
    assert_eq!(table_count(&connection, "coordination_events"), 1);
    assert_eq!(table_count(&connection, "active_coordination_decisions"), 1);
}

#[test]
fn evenement_d_une_generation_inactive_est_refuse_avant_toute_decision() {
    let fixture = Fixture::new("generation-obsolete");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "codex-1");
    let connection = Connection::open(&fixture.database).unwrap();
    let payload: Vec<u8> = connection
        .query_row(
            "SELECT payload_json FROM delegation_generations
             WHERE delegation_id = ?1 AND generation = 1",
            [delegation_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    let mut previous: GenerationDelegation = serde_json::from_slice(&payload).unwrap();
    previous.etat = EtatGenerationDelegation::Reassignee;
    let mut active = previous.clone();
    active.generation = 2;
    active.etat = EtatGenerationDelegation::Ouverte;
    active.generation_precedente = Some(1);
    active.trigger_event_id = Some("generation-2".to_string());
    connection.execute_batch("BEGIN IMMEDIATE").unwrap();
    connection
        .execute(
            "UPDATE delegation_generations SET state='reassignee', payload_json=?1
             WHERE delegation_id=?2 AND generation=1",
            params![
                serde_json::to_vec(&previous).unwrap(),
                delegation_id.to_string()
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO delegation_generations(
                 delegation_id,objective_id,generation,participant_id,state,payload_json
             ) VALUES (?1,?2,2,?3,'ouverte',?4)",
            params![
                delegation_id.to_string(),
                objectif_id.to_string(),
                active.participant_id,
                serde_json::to_vec(&active).unwrap(),
            ],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE delegation_lineages SET active_generation=2, payload_json=?1
             WHERE delegation_id=?2",
            params![
                serde_json::to_vec(&maicie::domain::LigneeDelegation {
                    delegation_id,
                    objectif_id,
                    generation_active: 2,
                })
                .unwrap(),
                delegation_id.to_string(),
            ],
        )
        .unwrap();
    connection.execute_batch("COMMIT").unwrap();
    drop(connection);

    let input = EntreeReductionCoordination::EvenementAtteste {
        objectif_id,
        delegation_id,
        generation: 1,
        evenement: fixture_event(FraicheurCoordination::Fresh),
    };
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    assert!(store.apply_coordination_reduction(&input).is_err());
    drop(store);
    let connection = Connection::open(&fixture.database).unwrap();
    assert_eq!(table_count(&connection, "coordination_events"), 0);
    assert_eq!(table_count(&connection, "active_coordination_decisions"), 0);
}

#[test]
fn seuil_reassigne_vers_la_chaine_epinglee_et_prepare_quatre_effets_atomiques() {
    let fixture = Fixture::new("reassignation-atomique");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let request_id = initial_request_id(&store, delegation_id);
    let lot = reassignment_lot(
        objectif_id,
        delegation_id,
        1,
        vec![
            reassignment_fact(
                "reminder-1",
                &request_id,
                TypeFaitReassignation::ReminderSent,
            ),
            reassignment_fact(
                "reminder-2",
                &request_id,
                TypeFaitReassignation::ReminderSent,
            ),
        ],
    );
    let first = store.apply_reassignment_batch(&lot).unwrap();
    assert!(!first.replayed);
    assert_eq!(
        first.reduction.source.etat,
        EtatGenerationDelegation::Reassignee
    );
    let successor = first.reduction.successeur.as_ref().unwrap();
    assert_eq!(successor.participant_id, "bob");
    assert_eq!(successor.generation, 2);
    let request_outboxes = store.pending_tracked_request_outboxes().unwrap();
    assert_eq!(request_outboxes.len(), 2);
    let cancel = request_outboxes
        .iter()
        .find(|outbox| outbox.kind == maicie::domain::TypeEffetDemandeSuivie::Annuler)
        .unwrap();
    assert!(matches!(
        serde_json::from_slice::<WrapperToDaemon>(&cancel.message_bytes).unwrap(),
        WrapperToDaemon::CancelRequest { id, sender, .. }
            if id == request_id && sender == maicie::MAICIE_IDENTITY
    ));
    let create = request_outboxes
        .iter()
        .find(|outbox| outbox.kind == maicie::domain::TypeEffetDemandeSuivie::Creer)
        .unwrap();
    let message: PublicMessage = serde_json::from_slice(&create.message_bytes).unwrap();
    assert_eq!(message.id, create.request_id);
    assert_eq!(message.to, "bob");
    assert!(message.reply);
    assert_eq!(message.deadline_at, Some(1_787_500_200));
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 2);

    let replay = store.apply_reassignment_batch(&lot).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.reduction, first.reduction);
    assert_eq!(store.pending_tracked_request_outboxes().unwrap().len(), 2);
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 2);
    let snapshot = store.coordination_snapshot(objectif_id).unwrap().unwrap();
    assert_eq!(snapshot.lineages[0].generation_active, 2);
    assert_eq!(snapshot.generations.len(), 2);
}

#[test]
fn faute_a_chaque_frontiere_f29_annule_generation_demandes_et_notifications() {
    for phase in [
        ReassignmentCommitPhase::AfterDecision,
        ReassignmentCommitPhase::AfterGenerations,
        ReassignmentCommitPhase::AfterRequestOutboxes,
        ReassignmentCommitPhase::AfterNotifications,
    ] {
        let fixture = Fixture::new(&format!("reassignation-rollback-{phase:?}"));
        let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
        let store = MaicieStore::open(&fixture.database).unwrap();
        let request_id = initial_request_id(&store, delegation_id);
        drop(store);
        let initial_connection = Connection::open(&fixture.database).unwrap();
        let initial_episode = reminder_episode_snapshot(&initial_connection, delegation_id);
        drop(initial_connection);
        let lot = reassignment_lot(
            objectif_id,
            delegation_id,
            1,
            vec![reassignment_fact(
                "timeout-source",
                &request_id,
                TypeFaitReassignation::TimedOut,
            )],
        );
        let mut store = MaicieStore::open(&fixture.database).unwrap();
        let failed = store.apply_reassignment_batch_observed(&lot, |observed| {
            if observed == phase {
                return Err(StoreError::Conflict("faute F29 injectée"));
            }
            Ok(())
        });
        assert!(matches!(
            failed,
            Err(StoreError::Conflict("faute F29 injectée"))
        ));
        drop(store);
        let connection = Connection::open(&fixture.database).unwrap();
        assert_eq!(table_count(&connection, "reassignment_events"), 0);
        assert_eq!(table_count(&connection, "reassignment_reductions"), 0);
        assert_eq!(
            table_count(&connection, "active_coordination_decisions"),
            0,
            "une décision F29 a fui le rollback à {phase:?}"
        );
        assert_eq!(
            reminder_episode_snapshot(&connection, delegation_id),
            initial_episode,
            "un épisode F29 a changé malgré le rollback à {phase:?}"
        );
        assert_eq!(table_count(&connection, "tracked_request_outbox"), 0);
        assert_eq!(table_count(&connection, "notification_outbox"), 0);
        assert_eq!(table_count(&connection, "delegation_generations"), 1);
        let active_generation: i64 = connection
            .query_row(
                "SELECT active_generation FROM delegation_lineages WHERE delegation_id = ?1",
                [delegation_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            active_generation, 1,
            "la lignée F29 a avancé malgré le rollback à {phase:?}"
        );
        let state: String = connection
            .query_row(
                "SELECT state FROM delegation_generations WHERE delegation_id = ?1",
                [delegation_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "ouverte");
    }
}

#[test]
fn borne_m_deux_cree_deux_reemissions_puis_un_unique_successeur() {
    let fixture = Fixture::new("reassignation-borne-m");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut request_id = initial_request_id(&store, delegation_id);
    for ordinal in 1..=2 {
        let lot = reassignment_lot(
            objectif_id,
            delegation_id,
            1,
            vec![reassignment_fact(
                &format!("answered-{ordinal}"),
                &request_id,
                TypeFaitReassignation::Answered,
            )],
        );
        let result = store.apply_reassignment_batch(&lot).unwrap();
        assert!(result.reduction.successeur.is_none());
        request_id = result
            .reduction
            .episode_successeur
            .as_ref()
            .unwrap()
            .request_id
            .clone();
    }
    let third = reassignment_lot(
        objectif_id,
        delegation_id,
        1,
        vec![reassignment_fact(
            "answered-3",
            &request_id,
            TypeFaitReassignation::Answered,
        )],
    );
    let result = store.apply_reassignment_batch(&third).unwrap();
    assert_eq!(result.reduction.successeur.unwrap().participant_id, "bob");
    assert_eq!(store.pending_tracked_request_outboxes().unwrap().len(), 4);
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 2);
}

#[test]
fn annulation_administrative_inhibe_definitivement_timeout_et_successeur() {
    let fixture = Fixture::new("reassignation-annulation");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let request_id = initial_request_id(&store, delegation_id);
    let cancel = reassignment_lot(
        objectif_id,
        delegation_id,
        1,
        vec![reassignment_fact(
            "admin-cancel",
            &request_id,
            TypeFaitReassignation::AnnulationAdministrative,
        )],
    );
    store.apply_reassignment_batch(&cancel).unwrap();
    let late = reassignment_lot(
        objectif_id,
        delegation_id,
        1,
        vec![reassignment_fact(
            "late-timeout",
            &request_id,
            TypeFaitReassignation::TimedOut,
        )],
    );
    let late_result = store.apply_reassignment_batch(&late).unwrap();
    assert!(late_result.reduction.successeur.is_none());
    assert!(late_result.reduction.effets_demandes.is_empty());
    assert_eq!(store.pending_tracked_request_outboxes().unwrap().len(), 1);
    assert_eq!(store.pending_notification_outboxes().unwrap().len(), 1);
}

#[test]
fn deux_ecrivains_arbitrent_le_meme_timeout_avec_une_contention_prouvee() {
    let fixture = Fixture::new("reassignation-course");
    let (objectif_id, delegation_id) = seed_coordination(&fixture.database, "alice");
    let first_store = MaicieStore::open(&fixture.database).unwrap();
    let second_store = MaicieStore::open(&fixture.database).unwrap();
    let request_id = initial_request_id(&first_store, delegation_id);
    let lot = reassignment_lot(
        objectif_id,
        delegation_id,
        1,
        vec![reassignment_fact(
            "timeout-concurrent",
            &request_id,
            TypeFaitReassignation::TimedOut,
        )],
    );
    let first_lot = lot.clone();
    let second_lot = lot;
    let (locked_tx, locked_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let first = thread::spawn(move || {
        let mut store = first_store;
        store.apply_reassignment_batch_observed(&first_lot, |phase| {
            if phase == ReassignmentCommitPhase::AfterDecision {
                locked_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            }
            Ok(())
        })
    });
    locked_rx.recv_timeout(Duration::from_secs(3)).unwrap();

    let probe = Connection::open(&fixture.database).unwrap();
    probe.busy_timeout(Duration::ZERO).unwrap();
    let busy = probe.execute_batch("BEGIN IMMEDIATE").unwrap_err();
    assert_eq!(busy.sqlite_error_code(), Some(ErrorCode::DatabaseBusy));
    drop(probe);

    let second = thread::spawn(move || {
        let mut store = second_store;
        store.apply_reassignment_batch(&second_lot)
    });
    release_tx.send(()).unwrap();
    let first_result = first.join().unwrap().unwrap();
    let second_result = second.join().unwrap().unwrap();
    assert_ne!(first_result.replayed, second_result.replayed);
    let connection = Connection::open(&fixture.database).unwrap();
    assert_eq!(table_count(&connection, "reassignment_reductions"), 1);
    assert_eq!(table_count(&connection, "tracked_request_outbox"), 2);
    assert_eq!(table_count(&connection, "notification_outbox"), 2);
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
            kind: TypeEvenementAttendu::ClotureObjectif,
            recipient: "referent".into(),
            policy_version: 1,
        }],
    }
}

fn seed_with_closure_recipients(
    store: &mut MaicieStore,
    index: i64,
    recipient_count: usize,
) -> Uuid {
    let objective = ObjectifCoordonne::nouveau(
        format!("objectif-{index}"),
        ModeObjectif::Delegue,
        1_787_500_000 + index,
    )
    .unwrap();
    let source = create_delegation(store, &objective, "alice");
    create_delegation(store, &objective, "bob");
    let mut snapshot = definition(objective.id, source, Vec::new(), "bob");
    snapshot.attentes = (0..recipient_count)
        .map(|recipient| AttenteNotification {
            attente_id: Uuid::new_v4(),
            objectif_id: objective.id,
            delegation_id: None,
            kind: TypeEvenementAttendu::ClotureObjectif,
            recipient: format!("recipient-{recipient}"),
            policy_version: 1,
        })
        .collect();
    store.register_coordination_snapshot(&snapshot).unwrap();
    objective.id
}

fn seed_coordination(database: &PathBuf, participant: &str) -> (Uuid, Uuid) {
    let mut store = MaicieStore::open(database).unwrap();
    let objective = ObjectifCoordonne::nouveau("réducteur", ModeObjectif::Delegue, 1).unwrap();
    let source = create_delegation(&mut store, &objective, participant);
    create_delegation(&mut store, &objective, "bob");
    let snapshot = definition(objective.id, source, vec![], "bob");
    store.register_coordination_snapshot(&snapshot).unwrap();
    (objective.id, source)
}

fn evaluated_input(
    objectif_id: Uuid,
    delegation_id: Uuid,
    event_id: &str,
) -> EntreeReductionCoordination {
    EntreeReductionCoordination::ClotureEvaluee(EvaluationCloture {
        event_id: event_id.to_string(),
        objectif_id,
        delegation_id,
        generation: 1,
        delivery_hash: "cd".repeat(32),
        issue: IssueClotureEvaluee::LivraisonValidee,
        evaluated_at: 1_787_500_100,
    })
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

fn table_count(connection: &Connection, table: &str) -> i64 {
    connection
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

#[derive(Debug, Eq, PartialEq)]
struct ReminderEpisodeSnapshot {
    generation: i64,
    request_id: String,
    request_ordinal: i64,
    reminder_count: i64,
    reemissions_used: i64,
    state: String,
    payload_json: Vec<u8>,
}

fn reminder_episode_snapshot(
    connection: &Connection,
    delegation_id: Uuid,
) -> Vec<ReminderEpisodeSnapshot> {
    let mut statement = connection
        .prepare(
            "SELECT generation,request_id,request_ordinal,reminder_count,
                    reemissions_used,state,payload_json
             FROM reminder_episodes WHERE delegation_id=?1 ORDER BY request_ordinal",
        )
        .unwrap();
    statement
        .query_map([delegation_id.to_string()], |row| {
            Ok(ReminderEpisodeSnapshot {
                generation: row.get(0)?,
                request_id: row.get(1)?,
                request_ordinal: row.get(2)?,
                reminder_count: row.get(3)?,
                reemissions_used: row.get(4)?,
                state: row.get(5)?,
                payload_json: row.get(6)?,
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
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

fn initial_request_id(store: &MaicieStore, delegation_id: Uuid) -> String {
    store
        .pending_delegation_outboxes()
        .unwrap()
        .into_iter()
        .find(|outbox| outbox.delegation_id == delegation_id)
        .unwrap()
        .message_id
        .to_string()
}

fn reassignment_fact(
    event_id: &str,
    request_id: &str,
    kind: TypeFaitReassignation,
) -> FaitReassignation {
    FaitReassignation {
        event_id: event_id.to_string(),
        request_id: request_id.to_string(),
        kind,
        observed_at: 1_787_500_100,
        freshness: FraicheurCoordination::Fresh,
        delivery_hash: None,
    }
}

fn reassignment_lot(
    objectif_id: Uuid,
    delegation_id: Uuid,
    generation: u64,
    faits: Vec<FaitReassignation>,
) -> LotReassignation {
    LotReassignation {
        objectif_id,
        delegation_id,
        generation,
        issued_at: 1_787_500_100,
        next_deadline_at: 1_787_500_200,
        faits,
    }
}

#[derive(Clone, Copy)]
struct DeliveryFixture<'a> {
    objective_id: Uuid,
    delegation_id: Uuid,
    participant: &'a str,
    in_reply_to: &'a str,
    request_id: &'a str,
    response_id: &'a str,
    hash: &'a str,
    now: i64,
}

fn delivery_claim(fixture: DeliveryFixture<'_>) -> GuichetClaim {
    let issuer_scope = "scope-f28-0123456789abcdef0123456789abcdef";
    let canonical_request = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"{issuer_scope}\",\"request_id\":\"{}\",\"issued_at\":{},\"from\":\"{}\",\"to\":\"maicie\",\"operation\":\"delivery_report\",\"payload\":{{\"objective_id\":\"{}\",\"delegation_id\":\"{}\",\"delivery_hash\":\"{}\",\"in_reply_to\":\"{}\"}}}}",
        fixture.request_id,
        fixture.now,
        fixture.participant,
        fixture.objective_id,
        fixture.delegation_id,
        fixture.hash,
        fixture.in_reply_to,
    )
    .into_bytes();
    GuichetClaim {
        issuer_scope: issuer_scope.to_string(),
        request_id: fixture.request_id.to_string(),
        canonical_request,
        claimed_at: fixture.now,
        claim_generation: 1,
        claim_token: format!("claim-f28-{}-0123456789abcdef", fixture.request_id),
        claim_lease_expires_at: fixture.now + 100,
        expires_at: fixture.now + 200,
    }
}

fn graft_delivery(store: &mut MaicieStore, fixture: DeliveryFixture<'_>) {
    let claim = delivery_claim(fixture);
    let canonical = parse_claim(&claim).unwrap();
    let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
        panic!("rapport attendu")
    };
    store
        .graft_delivery_report(&claim, &canonical, report, fixture.response_id, fixture.now)
        .unwrap();
}

fn active_state(
    snapshot: &maicie::store::StoredCoordinationSnapshot,
    delegation_id: Uuid,
) -> EtatGenerationDelegation {
    snapshot
        .generations
        .iter()
        .find(|generation| generation.delegation_id == delegation_id)
        .unwrap()
        .etat
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
