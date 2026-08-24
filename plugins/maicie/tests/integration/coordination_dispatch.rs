use bridget_transport::DaemonToWrapper;
use bridget_transport::protocol::{CoordinationEventKind, GuichetLifecycleState};
use maicie::bridget_client::BridgetClientLimits;
use maicie::domain::{
    AttenteNotification, ClasseDuree, DefinitionCoordination, Delegation, DependanceDelegation,
    EtatGenerationDelegation, EtatObjectif, EtatOutboxDelegation, FaitAppartenanceRepli,
    ModeObjectif, ModeQualificationDependance, ObjectifCoordonne, OutboxDelegation,
    PolitiqueReassignation, TypeEvenementAttendu,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::reconcile::{
    CoordinationReconcileAction, CoordinationReconcilePhase, NotificationReconcileAction,
    NotificationReconcilePhase, reconcile_coordination_startup_observed_with_limits,
    reconcile_coordination_startup_with_forbidden_f28_mutation_for_test,
    reconcile_coordination_startup_with_limits,
    reconcile_notification_startup_observed_with_limits,
    reconcile_notification_startup_with_limits,
};
use maicie::store::{MaicieStore, StoreError};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);
const CHILD_MODE: &str = "MAICIE_T1608_CRASH_MODE";
const CHILD_DATABASE: &str = "MAICIE_T1608_CRASH_DATABASE";
const CHILD_SOCKET: &str = "MAICIE_T1608_CRASH_SOCKET";
const CHILD_BARRIER: &str = "MAICIE_T1608_CRASH_BARRIER";
const COORDINATION_CHILD_DATABASE: &str = "MAICIE_T1610_CRASH_DATABASE";
const COORDINATION_CHILD_SOCKET: &str = "MAICIE_T1610_CRASH_SOCKET";
const COORDINATION_CHILD_BARRIER: &str = "MAICIE_T1610_CRASH_BARRIER";

#[test]
fn releve_coordination_applique_un_snapshot_frais_une_seule_fois() {
    let fixture = Fixture::new("coordination-fresh");
    let seed = seed_coordination_stream(&fixture.database_path, false);
    let listener = fixture.bind();
    let request_id = seed.request_id.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("première relève attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        write_coordination_event(&mut writer, &request_id, 1, "evt-rappel-1");
        write_coordination_snapshot(&mut writer, Some(1));

        let (stream, _) = listener.accept().expect("seconde relève attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, Some(1));
        write_coordination_snapshot(&mut writer, Some(1));
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let first = reconcile_coordination_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("première relève");
    assert!(
        first
            .actions
            .contains(&CoordinationReconcileAction::EvenementApplique { cursor: 1 })
    );
    assert_eq!(store.coordination_cursor().unwrap(), Some(1));
    assert_eq!(
        table_count(&fixture.database_path, "coordination_events"),
        1
    );
    assert_eq!(
        table_count(&fixture.database_path, "reassignment_events"),
        1
    );

    let second = reconcile_coordination_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("rejeu depuis curseur");
    assert!(!second.actions.iter().any(|action| matches!(
        action,
        CoordinationReconcileAction::EvenementApplique { .. }
    )));
    assert_eq!(
        table_count(&fixture.database_path, "coordination_events"),
        1
    );
    assert_eq!(
        table_count(&fixture.database_path, "reassignment_events"),
        1
    );
    server.join().expect("serveur coordination terminé");
}

#[test]
fn gap_avant_snapshot_interdit_tout_effet_metier() {
    let fixture = Fixture::new("coordination-gap");
    let seed = seed_coordination_stream(&fixture.database_path, false);
    let listener = fixture.bind();
    let request_id = seed.request_id.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("relève attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        write_coordination_event(&mut writer, &request_id, 1, "evt-non-frais");
        write_json(
            &mut writer,
            json!({
                "type":"coordination_gap",
                "v":2,
                "from_cursor":1,
                "to_cursor":1,
                "reason":"fixture_gap"
            }),
        );
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_coordination_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("Gap reste une observation");
    assert!(matches!(
        report.actions.as_slice(),
        [CoordinationReconcileAction::Gap { reason, .. }] if reason == "fixture_gap"
    ));
    assert_eq!(store.coordination_cursor().unwrap(), None);
    assert_eq!(
        table_count(&fixture.database_path, "coordination_events"),
        0
    );
    assert_eq!(
        table_count(&fixture.database_path, "reassignment_events"),
        0
    );
    server.join().expect("serveur Gap terminé");
}

#[test]
fn refus_canonique_de_la_releve_reste_non_fatal_et_visible() {
    let fixture = Fixture::new("coordination-canonical-rejected");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("relève attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        write_json(
            &mut writer,
            json!({
                "type":"ServiceRejected",
                "reason":{"kind":"canonical_bytes_mismatch"}
            }),
        );
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_coordination_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("un refus de relève ne doit pas interrompre la commande Maicie");
    assert!(matches!(
        report.actions.as_slice(),
        [CoordinationReconcileAction::Unavailable { reason }]
            if reason.contains("canonical_bytes_mismatch")
    ));
    assert_eq!(store.coordination_cursor().unwrap(), None);
    server.join().expect("serveur refus terminé");
}

#[test]
fn releve_coordination_bornee_a_512_refuse_un_lot_incomplet_sans_mutation() {
    let fixture = Fixture::new("coordination-bound");
    let seed = seed_coordination_stream(&fixture.database_path, false);
    let listener = fixture.bind();
    let request_id = seed.request_id.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("relève bornée attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        for cursor in 1..=512 {
            write_coordination_event(
                &mut writer,
                &request_id,
                cursor,
                &format!("evt-borne-{cursor}"),
            );
        }
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_coordination_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("la borne devient un résultat visible");
    assert_eq!(
        report.actions,
        vec![CoordinationReconcileAction::BudgetEpuise]
    );
    assert_eq!(store.coordination_cursor().unwrap(), None);
    assert_eq!(
        table_count(&fixture.database_path, "coordination_events"),
        0
    );
    assert_eq!(
        table_count(&fixture.database_path, "reassignment_events"),
        0
    );
    server.join().expect("serveur borne terminé");
}

#[test]
fn terminal_transport_reassigne_sans_qualifier_une_arete_f28() {
    let (state, pending_requests) = run_terminal_transport_case("coordination-terminal-f29", false);
    assert_eq!(
        state,
        EtatGenerationDelegation::Bloquee,
        "timed_out alimente F29 mais ne qualifie jamais l'arête F28"
    );
    assert_eq!(
        pending_requests, 2,
        "F29 écrit annulation source et demande successeur"
    );
}

#[test]
fn oracle_refuse_le_mutant_qui_route_un_terminal_transport_vers_f28() {
    let (state, _) = run_terminal_transport_case("coordination-terminal-mutant", true);
    let oracle_failed = std::panic::catch_unwind(|| {
        assert_eq!(
            state,
            EtatGenerationDelegation::Bloquee,
            "un terminal transport ne doit jamais ouvrir F28"
        );
    });
    assert!(
        oracle_failed.is_err(),
        "l'oracle doit tuer la mutation qui transforme timed_out en fait F28"
    );
}

fn run_terminal_transport_case(
    label: &str,
    inject_forbidden_f28_route: bool,
) -> (EtatGenerationDelegation, usize) {
    let fixture = Fixture::new(label);
    let seed = seed_coordination_stream(&fixture.database_path, true);
    let listener = fixture.bind();
    let request_id = seed.request_id.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("relève attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        write_json(
            &mut writer,
            serde_json::to_value(DaemonToWrapper::RequestLifecycleEvent {
                version: 1,
                issuer_scope: "scope-coordination-fixture-0123456789".to_string(),
                event_id: "evt-timeout-source".to_string(),
                request_id,
                state: GuichetLifecycleState::TimedOut,
                observed_at: ISSUED_AT + 10,
                in_reply_to: None,
                response_message_id: None,
            })
            .expect("terminal sérialisable"),
        );
        write_coordination_snapshot(&mut writer, None);
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = if inject_forbidden_f28_route {
        reconcile_coordination_startup_with_forbidden_f28_mutation_for_test(
            &mut store,
            fixture.socket_path(),
            limits(Duration::from_secs(2)),
        )
    } else {
        reconcile_coordination_startup_with_limits(
            &mut store,
            fixture.socket_path(),
            limits(Duration::from_secs(2)),
        )
    }
    .expect("terminal F29 appliqué");
    assert!(report.actions.iter().any(|action| matches!(
        action,
        CoordinationReconcileAction::TerminalApplique { state, .. } if state == "timed_out"
    )));
    let snapshot = store
        .coordination_snapshot(seed.objective_id)
        .unwrap()
        .expect("snapshot coordination");
    let dependant = seed.dependant_id.expect("dépendant fixture");
    let state = snapshot
        .generations
        .iter()
        .find(|generation| generation.delegation_id == dependant)
        .expect("génération dépendante")
        .etat;
    let pending_requests = store.pending_tracked_request_outboxes().unwrap().len();
    server.join().expect("serveur terminal terminé");
    (state, pending_requests)
}

#[test]
fn faute_f29_du_chemin_combine_annule_curseur_evenements_generations_et_boites() {
    for phase in [
        CoordinationReconcilePhase::AfterDecision,
        CoordinationReconcilePhase::AfterGenerations,
        CoordinationReconcilePhase::AfterRequestOutboxes,
        CoordinationReconcilePhase::AfterNotifications,
    ] {
        let fixture = Fixture::new(&format!("coordination-f29-rollback-{phase:?}"));
        // Un seuil de un garantit que chaque frontière observée suit de vraies
        // écritures F29 ; aucune phase n'est une vacuole sans effet à annuler.
        let seed = seed_coordination_stream_with_threshold(&fixture.database_path, false, 1);
        let initial_generations = table_count(&fixture.database_path, "delegation_generations");
        let initial_episode = reminder_episode(&fixture.database_path, seed.source_id);
        let listener = fixture.bind();
        let request_id = seed.request_id.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("relève fautive attendue");
            let (mut reader, mut writer) = split(stream);
            complete_coordination_handshake(&mut reader, &mut writer, None);
            write_coordination_event(&mut writer, &request_id, 1, "evt-f29-rollback");
            write_coordination_snapshot(&mut writer, Some(1));
        });

        let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
        let failed = reconcile_coordination_startup_observed_with_limits(
            &mut store,
            fixture.socket_path(),
            limits(Duration::from_secs(2)),
            |observed| {
                if observed == phase {
                    return Err(StoreError::Conflict("faute F29 combinée injectée"));
                }
                Ok(())
            },
        );
        assert!(
            failed.is_err(),
            "la faute {phase:?} doit sortir du point d'entrée réel"
        );
        server.join().expect("serveur fautif terminé");
        drop(store);

        assert_eq!(coordination_cursor(&fixture.database_path), None);
        assert_eq!(
            table_count(&fixture.database_path, "coordination_events"),
            0
        );
        assert_eq!(
            table_count(&fixture.database_path, "reassignment_events"),
            0
        );
        assert_eq!(
            table_count(&fixture.database_path, "reassignment_reductions"),
            0
        );
        assert_eq!(
            table_count(&fixture.database_path, "active_coordination_decisions"),
            0
        );
        assert_eq!(
            table_count(&fixture.database_path, "tracked_request_outbox"),
            0
        );
        assert_eq!(
            table_count(&fixture.database_path, "notification_outbox"),
            0
        );
        assert_eq!(
            table_count(&fixture.database_path, "delegation_generations"),
            initial_generations,
            "aucune génération ne doit fuir à {phase:?}"
        );
        assert_eq!(active_generation(&fixture.database_path, seed.source_id), 1);
        assert_eq!(
            reminder_episode(&fixture.database_path, seed.source_id),
            initial_episode,
            "l'épisode de relance doit rester identique à {phase:?}"
        );
    }
}

#[test]
fn sigkill_pendant_application_rejoue_sans_double_effet() {
    let fixture = Fixture::new("coordination-crash");
    let seed = seed_coordination_stream(&fixture.database_path, false);
    let listener = fixture.bind();
    let request_id = seed.request_id.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("relève enfant attendue");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        write_coordination_event(&mut writer, &request_id, 1, "evt-crash-replay");
        write_coordination_snapshot(&mut writer, Some(1));
    });
    let (mut child, mut barrier, barrier_path) = spawn_coordination_crash_child(&fixture);
    wait_barrier(&mut barrier, "before_store_commit");
    child.kill().expect("SIGKILL enfant réel");
    child.wait().expect("wait enfant réel");
    server.join().expect("serveur enfant terminé");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    fs::remove_file(fixture.socket_path()).expect("socket crash supprimée");
    assert_eq!(
        table_count(&fixture.database_path, "coordination_events"),
        0
    );
    assert_eq!(
        table_count(&fixture.database_path, "reassignment_events"),
        0
    );

    let listener = fixture.bind();
    let request_id = seed.request_id;
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("rejeu attendu");
        let (mut reader, mut writer) = split(stream);
        complete_coordination_handshake(&mut reader, &mut writer, None);
        write_coordination_event(&mut writer, &request_id, 1, "evt-crash-replay");
        write_coordination_snapshot(&mut writer, Some(1));
    });
    let mut store = MaicieStore::open(&fixture.database_path).expect("store repris");
    reconcile_coordination_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("rejeu après crash");
    assert_eq!(store.coordination_cursor().unwrap(), Some(1));
    assert_eq!(
        table_count(&fixture.database_path, "coordination_events"),
        1
    );
    assert_eq!(
        table_count(&fixture.database_path, "reassignment_events"),
        1
    );
    server.join().expect("serveur rejeu terminé");
}

#[test]
fn coordination_crash_child() {
    let Some(database) = std::env::var_os(COORDINATION_CHILD_DATABASE) else {
        return;
    };
    let socket = PathBuf::from(
        std::env::var_os(COORDINATION_CHILD_SOCKET).expect("socket coordination enfant"),
    );
    let barrier = PathBuf::from(
        std::env::var_os(COORDINATION_CHILD_BARRIER).expect("barrière coordination enfant"),
    );
    let mut store = MaicieStore::open(database).expect("store coordination enfant");
    reconcile_coordination_startup_observed_with_limits(
        &mut store,
        socket,
        limits(Duration::from_secs(3)),
        |phase| {
            if phase == CoordinationReconcilePhase::BeforeStoreCommit {
                block_at_barrier(&barrier, "before_store_commit");
            }
            Ok(())
        },
    )
    .expect("relève coordination enfant");
}

#[test]
fn notification_rejoue_les_octets_durables_et_ne_les_emet_qu_une_fois() {
    let fixture = Fixture::new("exact-bytes");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);

    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion notification attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);

        let mut actual = Vec::new();
        reader
            .read_until(b'\n', &mut actual)
            .expect("rejeu notification lu");
        assert_eq!(
            actual, expected,
            "le dispatcher doit transmettre les bytes persistés, sans re-sérialisation"
        );
        write_json(
            &mut writer,
            json!({
                "type":"IdempotencyResult",
                "operation_kind":"send",
                "idempotency_key":message_id.to_string(),
                "issue":{"kind":"accepted","expires_at":1_787_600_000_i64}
            }),
        );
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(300)),
    )
    .expect("reprise notification");
    assert!(matches!(
        report.actions.as_slice(),
        [NotificationReconcileAction::Issue { message_id: actual, .. }] if *actual == message_id
    ));
    server.join().expect("serveur notification termine");
    assert!(
        store
            .pending_notification_outboxes()
            .expect("pending relues")
            .is_empty(),
        "Accepted consomme durablement la boîte"
    );

    // Mutation discriminante : réémettre une notification déjà accepted
    // ouvrirait une seconde connexion. Le listener fermé rendrait ce second
    // passage visible comme une erreur au lieu d'un faux vert silencieux.
    let second = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(80)),
    )
    .expect("aucune boîte terminale ne doit être réémise");
    assert!(second.actions.is_empty());
}

#[test]
fn cloture_reelle_emet_exactement_trois_notifications_et_jamais_six() {
    let fixture = Fixture::new("three-notifications");
    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let objective_id = seed_closure_with_recipients(&mut store, 3);
    store
        .close_objective(objective_id, "clôture attestée", ISSUED_AT)
        .expect("clôture transactionnelle");
    let expected = store
        .pending_notification_outboxes()
        .expect("trois boîtes durables")
        .into_iter()
        .map(|outbox| {
            (
                outbox.message_id,
                replay_frame(&outbox.message_bytes, outbox.message_id, outbox.issued_at),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(expected.len(), 3, "T1607 crée une boîte par destinataire");

    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let mut received = std::collections::BTreeMap::new();
        for _ in 0..3 {
            let (stream, _) = listener.accept().expect("connexion notification attendue");
            let (mut reader, mut writer) = split(stream);
            complete_client_handshake(&mut reader, &mut writer);
            let mut actual = Vec::new();
            reader
                .read_until(b'\n', &mut actual)
                .expect("notification lue");
            let request: serde_json::Value = serde_json::from_slice(&actual).expect("JSON replay");
            let message_id =
                Uuid::parse_str(request["message_id"].as_str().expect("message_id filaire"))
                    .expect("UUID filaire");
            received.insert(message_id, actual);
            write_accepted(&mut writer, message_id);
        }
        received
    });

    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("dispatch de clôture");
    let received = server.join().expect("serveur de clôture");
    assert_eq!(
        received, expected,
        "chaque notification conserve ses octets exacts"
    );
    assert_eq!(report.actions.len(), 3);
    assert!(
        store
            .pending_notification_outboxes()
            .expect("boîtes relues")
            .is_empty()
    );

    // Mutation discriminante : si Accepted ne rendait pas l'outbox terminale,
    // ce second passage tenterait une quatrième connexion vers le listener
    // fermé et ne pourrait donc pas rester vert par simple équivalence JSON.
    let second = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(80)),
    )
    .expect("aucune notification terminale ne doit repartir");
    assert!(
        second.actions.is_empty(),
        "trois acceptations ne deviennent jamais six"
    );
}

#[test]
fn notification_partage_une_echeance_absolue_entre_handshake_et_rejeu() {
    let fixture = Fixture::new("absolute-deadline");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);

    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion notification attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"RoleHandshake","role":"client"})
        );
        thread::sleep(Duration::from_millis(170));
        write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
        let _hello = read_json(&mut reader);
        thread::sleep(Duration::from_millis(170));
        // Une échéance absolue correcte a déjà expiré. Un client mutant qui
        // renouvelle l'échéance par phase recevrait ce welcome et enverrait
        // alors SendIdempotent : l'assertion ci-dessous le rend rouge.
        let _ = write_json_if_open(
            &mut writer,
            json!({
                "type":"ClientWelcome",
                "version":1,
                "horizon_secs":3600,
                "issued_at_tolerance_secs":30,
                "capabilities":["send_idempotent","lookup"]
            }),
        );
        reader
            .get_mut()
            .set_nonblocking(true)
            .expect("lecture serveur non bloquante");
        let until = Instant::now() + Duration::from_millis(250);
        loop {
            let mut unexpected = String::new();
            match reader.read_line(&mut unexpected) {
                Ok(0) if Instant::now() >= until => break,
                Ok(0) => thread::sleep(Duration::from_millis(5)),
                Err(error) if error.kind() == ErrorKind::WouldBlock && Instant::now() < until => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Ok(_) => panic!(
                    "un délai réinitialisé par phase aurait envoyé une trame après le budget : {unexpected}"
                ),
                Err(error) => panic!("lecture serveur inattendue : {error}"),
            }
        }
    });

    let mut store = MaicieStore::open(&fixture.database_path).expect("store ouvert");
    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_millis(300)),
    )
    .expect("budget global respecté");
    assert!(matches!(
        report.actions.as_slice(),
        [NotificationReconcileAction::TransportIndisponible { message_id: actual, .. }]
            if *actual == message_id
    ));
    server.join().expect("serveur budget termine");
    assert_eq!(
        store
            .pending_notification_outboxes()
            .expect("pending relues")
            .len(),
        1,
        "une échéance dépassée ne consomme jamais la notification"
    );
}

#[test]
fn crash_reel_apres_write_ou_ack_rejoue_la_meme_notification_sans_doublon_local() {
    crash_avant_socket_apres_commit_laisse_la_boite_durable();
    crash_apres_write_avant_ack_rejoue_exactement();
    crash_apres_ack_avant_consommation_rejoue_exactement();
}

#[test]
fn notification_crash_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let database = PathBuf::from(std::env::var_os(CHILD_DATABASE).expect("base enfant"));
    let socket = PathBuf::from(std::env::var_os(CHILD_SOCKET).expect("socket enfant"));
    let barrier = PathBuf::from(std::env::var_os(CHILD_BARRIER).expect("barrière enfant"));
    let phase = match mode.as_str() {
        "before_socket" => NotificationReconcilePhase::BeforeSocket,
        "after_write" => NotificationReconcilePhase::AfterWriteBeforeAck,
        "after_ack" => NotificationReconcilePhase::AfterIssueBeforeStoreCommit,
        other => panic!("mode enfant inconnu : {other}"),
    };
    let mut store = MaicieStore::open(database).expect("store enfant");
    reconcile_notification_startup_observed_with_limits(
        &mut store,
        socket,
        limits(Duration::from_secs(2)),
        |observed| {
            if observed == phase {
                block_at_barrier(&barrier, &mode);
            }
            Ok(())
        },
    )
    .expect("reprise enfant");
}

fn crash_avant_socket_apres_commit_laisse_la_boite_durable() {
    let fixture = Fixture::new("crash-before-socket");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);
    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);

    // La boîte est déjà commitée (frontière T1607), mais aucun listener
    // n'existe. Le jalon prouve qu'un kill avant socket ne peut produire I/O.
    let (mut child, mut barrier, barrier_path) = spawn_crash_child("before_socket", &fixture);
    wait_barrier(&mut barrier, "before_socket");
    child.kill().expect("kill enfant réel");
    child.wait().expect("wait enfant réel");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    let store = MaicieStore::open(&fixture.database_path).expect("store après crash avant socket");
    assert_eq!(
        store
            .pending_notification_outboxes()
            .expect("outbox durable")
            .len(),
        1,
        "le kill avant I/O ne consomme pas une boîte déjà commitée"
    );
    drop(store);

    replay_after_crash(&fixture, message_id, &expected);
}

fn crash_apres_write_avant_ack_rejoue_exactement() {
    let fixture = Fixture::new("crash-after-write");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);
    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);

    let listener = fixture.bind();
    let first_expected = expected.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion enfant attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);
        let mut actual = Vec::new();
        reader
            .read_until(b'\n', &mut actual)
            .expect("write enfant lu");
        assert_eq!(
            actual, first_expected,
            "le write avant crash utilise les bytes durables"
        );
        let mut eof = [0_u8; 1];
        assert_eq!(
            reader.get_mut().read(&mut eof).expect("EOF enfant"),
            0,
            "le serveur garde la socket ouverte : seul le kill enfant coupe avant ACK"
        );
        let _ = writer.flush();
    });
    let (mut child, mut barrier, barrier_path) = spawn_crash_child("after_write", &fixture);
    wait_barrier(&mut barrier, "after_write");
    child.kill().expect("kill enfant réel");
    child.wait().expect("wait enfant réel");
    server.join().expect("serveur crash write");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    fs::remove_file(fixture.socket_path()).expect("socket crash supprimée");

    replay_after_crash(&fixture, message_id, &expected);
}

fn crash_apres_ack_avant_consommation_rejoue_exactement() {
    let fixture = Fixture::new("crash-after-ack");
    let message_id = Uuid::new_v4();
    let message_bytes = message_bytes(message_id);
    insert_notification(&fixture.database_path, message_id, &message_bytes);
    let expected = replay_frame(&message_bytes, message_id, ISSUED_AT);

    let listener = fixture.bind();
    let first_expected = expected.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion enfant attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);
        let mut actual = Vec::new();
        reader
            .read_until(b'\n', &mut actual)
            .expect("send enfant lu");
        assert_eq!(actual, first_expected, "l'ACK porte sur la trame exacte");
        write_accepted(&mut writer, message_id);
    });
    let (mut child, mut barrier, barrier_path) = spawn_crash_child("after_ack", &fixture);
    wait_barrier(&mut barrier, "after_ack");
    child.kill().expect("kill enfant réel");
    child.wait().expect("wait enfant réel");
    server.join().expect("serveur crash ACK");
    fs::remove_file(&barrier_path).expect("barrière supprimée");
    fs::remove_file(fixture.socket_path()).expect("socket crash supprimée");

    replay_after_crash(&fixture, message_id, &expected);
}

fn replay_after_crash(fixture: &Fixture, message_id: Uuid, expected: &[u8]) {
    let listener = fixture.bind();
    let expected = expected.to_vec();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("reprise attendue");
        let (mut reader, mut writer) = split(stream);
        complete_client_handshake(&mut reader, &mut writer);
        let mut actual = Vec::new();
        reader.read_until(b'\n', &mut actual).expect("rejeu lu");
        assert_eq!(
            actual, expected,
            "un crash ne change ni clé ni bytes de notification"
        );
        write_accepted(&mut writer, message_id);
    });
    let mut store = MaicieStore::open(&fixture.database_path).expect("store repris");
    let report = reconcile_notification_startup_with_limits(
        &mut store,
        fixture.socket_path(),
        limits(Duration::from_secs(2)),
    )
    .expect("reprise après crash");
    assert!(matches!(
        report.actions.as_slice(),
        [NotificationReconcileAction::Issue { message_id: actual, .. }] if *actual == message_id
    ));
    assert!(
        store
            .pending_notification_outboxes()
            .expect("outbox relue")
            .is_empty(),
        "un seul Accepted durable consomme la notification après le crash"
    );
    server.join().expect("serveur reprise termine");
}

const ISSUED_AT: i64 = 1_787_500_000;

fn limits(timeout: Duration) -> BridgetClientLimits {
    BridgetClientLimits {
        connect_timeout: timeout,
        io_timeout: timeout,
        max_frame_bytes: 64 * 1024,
    }
}

fn message_bytes(message_id: Uuid) -> Vec<u8> {
    let mut bytes = b"{\"id\":\"".to_vec();
    bytes.extend_from_slice(message_id.to_string().as_bytes());
    bytes.extend_from_slice(
        r#"","from":"maicie","to":"alice","body":"clôture attestée","reply":false,"hops":4,"extension_future":{"opaque":true}}"#
            .as_bytes(),
    );
    bytes
}

fn replay_frame(message_bytes: &[u8], message_id: Uuid, issued_at: i64) -> Vec<u8> {
    let mut frame = br#"{"type":"SendIdempotent","message":"#.to_vec();
    frame.extend_from_slice(message_bytes);
    frame.extend_from_slice(br#","message_id":"#);
    frame.extend_from_slice(
        serde_json::to_string(&message_id.to_string())
            .unwrap()
            .as_bytes(),
    );
    frame.extend_from_slice(br#","issued_at":"#);
    frame.extend_from_slice(issued_at.to_string().as_bytes());
    frame.extend_from_slice(b"}\n");
    frame
}

fn insert_notification(database_path: &PathBuf, message_id: Uuid, message_bytes: &[u8]) {
    let _store = MaicieStore::open(database_path).expect("schéma Maicie créé");
    let connection = Connection::open(database_path).expect("base fixture ouverte");
    // Le harnais isole le dispatcher du producteur T1607 : il ne fabrique pas
    // de clôture de coordination, mais insère une boîte déjà durable. La FK
    // vers l'objectif est donc explicitement hors de l'oracle C.
    connection
        .execute_batch("PRAGMA foreign_keys = OFF;")
        .expect("FK fixture désactivées");
    connection
        .execute(
            "INSERT INTO notification_outbox(
                 message_id, idempotency_key, issued_at, objective_id, delegation_id,
                 generation, event_id, policy_version, recipient, message_bytes,
                 state, last_issue_json, terminal
             ) VALUES(?1, ?2, ?3, ?4, NULL, NULL, ?5, 1, 'alice', ?6, 'prepared', NULL, 0)",
            params![
                message_id.to_string(),
                format!("notification:{message_id}"),
                ISSUED_AT,
                Uuid::new_v4().to_string(),
                format!("event:{message_id}"),
                message_bytes,
            ],
        )
        .expect("notification durable insérée");
}

fn complete_client_handshake(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
) {
    assert_eq!(
        read_json(reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(writer, json!({"type":"RoleAccepted","role":"client"}));
    let hello = read_json(reader);
    assert_eq!(hello["type"], "ClientHello");
    write_json(
        writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().expect("clone socket")),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("ligne client");
    serde_json::from_str(&line).expect("JSON client")
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).expect("écriture serveur");
    writer.flush().expect("flush serveur");
}

fn write_json_if_open(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) -> bool {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).is_ok() && writer.flush().is_ok()
}

fn write_accepted(writer: &mut BufWriter<UnixStream>, message_id: Uuid) {
    write_json(
        writer,
        json!({
            "type":"IdempotencyResult",
            "operation_kind":"send",
            "idempotency_key":message_id.to_string(),
            "issue":{"kind":"accepted","expires_at":1_787_600_000_i64}
        }),
    );
}

fn complete_coordination_handshake(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
    expected_cursor: Option<u64>,
) {
    assert_eq!(
        read_json(reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(hello["version"], 1);
    assert_eq!(hello["service"], "maicie");
    assert_eq!(
        hello["capabilities"],
        json!(["maicie_guichet", "coordination_events_v2"])
    );
    write_json(
        writer,
        json!({
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet","coordination_events_v2"]
        }),
    );
    let mut subscribe = String::new();
    reader
        .read_line(&mut subscribe)
        .expect("trame coordination_subscribe");
    let expected = match expected_cursor {
        Some(cursor) => {
            format!("{{\"type\":\"coordination_subscribe\",\"v\":2,\"after_cursor\":{cursor}}}")
        }
        None => "{\"type\":\"coordination_subscribe\",\"v\":2}".to_string(),
    };
    assert_eq!(subscribe.trim_end(), expected);
}

fn write_coordination_event(
    writer: &mut BufWriter<UnixStream>,
    request_id: &str,
    cursor: u64,
    event_id: &str,
) {
    let frame = DaemonToWrapper::CoordinationEvent {
        version: 2,
        event_id: event_id.to_string(),
        request_id: request_id.to_string(),
        kind: CoordinationEventKind::ReminderSent,
        reminder_message_id: format!("rappel-{cursor}"),
        recipient: "alice".to_string(),
        generation: 1,
        observed_at: ISSUED_AT + i64::try_from(cursor).expect("curseur borne"),
        cursor: Some(cursor),
    };
    writer
        .write_all(&serde_json::to_vec(&frame).expect("événement sérialisable"))
        .expect("écriture événement");
    writer.write_all(b"\n").expect("fin de trame événement");
    writer.flush().expect("flush événement");
}

fn write_coordination_snapshot(writer: &mut BufWriter<UnixStream>, through_cursor: Option<u64>) {
    write_json(
        writer,
        serde_json::to_value(DaemonToWrapper::CoordinationSnapshotCaughtUp {
            version: 2,
            through_cursor,
        })
        .expect("snapshot sérialisable"),
    );
}

fn spawn_coordination_crash_child(fixture: &Fixture) -> (Child, UnixStream, PathBuf) {
    let barrier_path = fixture.root.join("coordination-crash.barrier.sock");
    let listener = UnixListener::bind(&barrier_path).expect("barrière coordination parent");
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        sender
            .send(listener.accept().map(|pair| pair.0))
            .expect("barrière coordination connectée");
    });
    let mut child = Command::new(std::env::current_exe().expect("binaire test"))
        .arg("--exact")
        .arg("coordination_dispatch::coordination_crash_child")
        .arg("--nocapture")
        .env(COORDINATION_CHILD_DATABASE, &fixture.database_path)
        .env(COORDINATION_CHILD_SOCKET, &fixture.socket)
        .env(COORDINATION_CHILD_BARRIER, &barrier_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("enfant coordination lancé");
    let barrier = match receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(stream)) => stream,
        other => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("barrière coordination absente : {other:?}");
        }
    };
    (child, barrier, barrier_path)
}

fn spawn_crash_child(mode: &str, fixture: &Fixture) -> (Child, UnixStream, PathBuf) {
    let barrier_path = fixture.root.join(format!("{mode}.barrier.sock"));
    let listener = UnixListener::bind(&barrier_path).expect("barrière parent");
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        sender
            .send(listener.accept().map(|pair| pair.0))
            .expect("barrière connectée");
    });
    let mut child = Command::new(std::env::current_exe().expect("binaire test"))
        .arg("--exact")
        .arg("coordination_dispatch::notification_crash_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env(CHILD_DATABASE, &fixture.database_path)
        .env(CHILD_SOCKET, &fixture.socket)
        .env(CHILD_BARRIER, &barrier_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("enfant crash lancé");
    let barrier = match receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(stream)) => stream,
        other => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("barrière enfant absente : {other:?}");
        }
    };
    (child, barrier, barrier_path)
}

fn wait_barrier(stream: &mut UnixStream, phase: &str) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout barrière");
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .expect("ligne barrière");
    assert_eq!(line.trim(), format!("BARRIER:{phase}"));
}

fn block_at_barrier(path: &PathBuf, phase: &str) {
    let mut stream = UnixStream::connect(path).expect("barrière enfant jointe");
    writeln!(stream, "BARRIER:{phase}").expect("jalon enfant écrit");
    stream.flush().expect("jalon enfant flush");
    let mut release = [0_u8; 1];
    let _ = stream.read(&mut release);
}

fn seed_closure_with_recipients(store: &mut MaicieStore, recipients: usize) -> Uuid {
    let objective =
        ObjectifCoordonne::nouveau("clôture à notifier", ModeObjectif::Delegue, ISSUED_AT - 10)
            .expect("objectif fixture");
    let delegation = create_delegation(store, &objective, "alice");
    create_delegation(store, &objective, "bob");
    let definition = DefinitionCoordination {
        objectif_id: objective.id,
        dependencies: Vec::new(),
        policies: vec![PolitiqueReassignation {
            delegation_id: delegation,
            objectif_id: objective.id,
            classe: ClasseDuree::Normale,
            version: 1,
            seuil_relances: 2,
            max_reemissions: 2,
            chaine_repli: vec![FaitAppartenanceRepli {
                objectif_id: objective.id,
                participant_id: "bob".to_string(),
                membership_version: 1,
                est_pilote: false,
            }],
        }],
        attentes: (0..recipients)
            .map(|index| AttenteNotification {
                attente_id: Uuid::new_v4(),
                objectif_id: objective.id,
                delegation_id: None,
                kind: TypeEvenementAttendu::ClotureObjectif,
                recipient: format!("recipient-{index}"),
                policy_version: 1,
            })
            .collect(),
    };
    store
        .register_coordination_snapshot(&definition)
        .expect("définition de clôture");
    objective.id
}

struct CoordinationSeed {
    objective_id: Uuid,
    source_id: Uuid,
    request_id: String,
    dependant_id: Option<Uuid>,
}

fn seed_coordination_stream(database_path: &PathBuf, with_dependency: bool) -> CoordinationSeed {
    seed_coordination_stream_with_threshold(database_path, with_dependency, 2)
}

fn seed_coordination_stream_with_threshold(
    database_path: &PathBuf,
    with_dependency: bool,
    reminder_threshold: u32,
) -> CoordinationSeed {
    let mut store = MaicieStore::open(database_path).expect("store de semence");
    let mut objective =
        ObjectifCoordonne::nouveau("relève de coordination", ModeObjectif::Delegue, ISSUED_AT)
            .expect("objectif fixture");
    objective
        .transition(EtatObjectif::EnCoordination, ISSUED_AT + 1)
        .expect("objectif coordonné");
    let (source, request_id) = create_delegation_with_message(&mut store, &objective, "alice");
    create_delegation(&mut store, &objective, "bob");
    let dependant = with_dependency.then(|| create_delegation(&mut store, &objective, "carol"));
    let dependencies = dependant
        .map(|dependant_id| {
            vec![DependanceDelegation {
                objectif_id: objective.id,
                prerequis_id: source,
                dependant_id,
                mode: ModeQualificationDependance::HashGreffe,
            }]
        })
        .unwrap_or_default();
    store
        .register_coordination_snapshot(&DefinitionCoordination {
            objectif_id: objective.id,
            dependencies,
            policies: vec![PolitiqueReassignation {
                delegation_id: source,
                objectif_id: objective.id,
                classe: ClasseDuree::Normale,
                version: 1,
                seuil_relances: reminder_threshold,
                max_reemissions: 2,
                chaine_repli: vec![FaitAppartenanceRepli {
                    objectif_id: objective.id,
                    participant_id: "bob".to_string(),
                    membership_version: 1,
                    est_pilote: false,
                }],
            }],
            attentes: Vec::new(),
        })
        .expect("snapshot coordination inscrit");
    CoordinationSeed {
        objective_id: objective.id,
        source_id: source,
        request_id: request_id.to_string(),
        dependant_id: dependant,
    }
}

fn create_delegation(store: &mut MaicieStore, objective: &ObjectifCoordonne, target: &str) -> Uuid {
    create_delegation_with_message(store, objective, target).0
}

fn create_delegation_with_message(
    store: &mut MaicieStore,
    objective: &ObjectifCoordonne,
    target: &str,
) -> (Uuid, Uuid) {
    let delegation = Delegation::nouvelle(
        objective.id,
        target,
        "instruction de fixture",
        ClasseDuree::Normale,
        "raison de fixture",
    )
    .expect("délégation fixture");
    let body = b"fixture".to_vec();
    let message_id = Uuid::new_v4();
    let outbox = OutboxDelegation {
        message_id,
        delegation_id: delegation.id,
        target: target.to_string(),
        body_hash: stable_body_hash(&body),
        body_bytes: body,
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: ISSUED_AT + 60,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: ISSUED_AT + 30,
        dedup_retained_until: ISSUED_AT + 300,
    };
    let prepared = PreparedDelegation::new(
        objective.clone(),
        delegation.clone(),
        outbox,
        store.issuer_scope(),
        ISSUED_AT,
        64 * 1024,
    )
    .expect("outbox fixture");
    store
        .create_prepared_delegation(&prepared)
        .expect("délégation persistée");
    (delegation.id, message_id)
}

fn table_count(database_path: &PathBuf, table: &str) -> i64 {
    Connection::open(database_path)
        .expect("base observable")
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .expect("compteur observable")
}

fn coordination_cursor(database_path: &PathBuf) -> Option<i64> {
    Connection::open(database_path)
        .expect("base observable")
        .query_row("SELECT MAX(cursor) FROM coordination_events", [], |row| {
            row.get(0)
        })
        .expect("curseur observable")
}

fn active_generation(database_path: &PathBuf, delegation_id: Uuid) -> i64 {
    Connection::open(database_path)
        .expect("base observable")
        .query_row(
            "SELECT active_generation FROM delegation_lineages WHERE delegation_id=?1",
            [delegation_id.to_string()],
            |row| row.get(0),
        )
        .expect("génération active observable")
}

fn reminder_episode(database_path: &PathBuf, delegation_id: Uuid) -> Option<Vec<u8>> {
    Connection::open(database_path)
        .expect("base observable")
        .query_row(
            "SELECT payload_json FROM reminder_episodes WHERE delegation_id=?1",
            [delegation_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .expect("épisode observable")
}

struct Fixture {
    root: PathBuf,
    database_path: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let _ = label;
        let root = std::env::temp_dir().join(format!("md-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&root).expect("répertoire fixture");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("permissions fixture");
        Self {
            database_path: root.join("maicie.sqlite"),
            socket: root.join("bridget.sock"),
            root,
        }
    }

    fn bind(&self) -> UnixListener {
        UnixListener::bind(&self.socket).expect("socket fixture")
    }

    fn socket_path(&self) -> &PathBuf {
        &self.socket
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
