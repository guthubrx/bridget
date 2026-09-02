//! SPEC-087, versant Maicie : oracles du store et de la relève.
//!
//! Chaque test nomme une propriété. Les gardes de pause sont éprouvées par
//! effet observable (ligne différée, outbox absente), jamais par présence.

use maicie::app::{
    DelegateRequest, DelegateResult, DelegationCandidate, close, delegate,
    open_focus_waiting_for_agent,
};
use maicie::config::DurationClasses;
use maicie::control::ControlSnapshot;
use maicie::domain::{
    AttestationConsumption, ClasseDuree, FaitReassignation, FraicheurCoordination,
    HumanRequestOriginAttestation, ObjectiveOpeningPermit, ObservedHumanMessage, SuiteObjective,
    TypeFaitReassignation, human_message_content_seal,
};
use maicie::reconcile::{
    HumanInboxReconcilePhase, ReconcileAction, ReconcileError, reconcile_focus_waiting_agents,
    reconcile_human_inbox_observed_with_limits, reconcile_human_inbox_with_limits,
    reconcile_startup_with_limits,
};
use maicie::store::{HumanDecisionApplication, MaicieStore, SCHEMA_VERSION, StoreError};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

#[path = "support/historical_guichet_receptions.rs"]
#[allow(dead_code)]
mod historical_guichet_receptions;

struct RootGuard {
    path: PathBuf,
}

impl RootGuard {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!("maicie-087-{label}-{}", Uuid::new_v4()));
        // Le store exige un répertoire privé : il le crée lui-même en 0700.
        Self { path }
    }
}

impl Drop for RootGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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

fn active(cap: u32) -> ControlSnapshot {
    ControlSnapshot::Read {
        paused: false,
        auto_objectives_cap: cap,
        inbox_open_count: 0,
        read_at: 1,
    }
}

fn paused() -> ControlSnapshot {
    ControlSnapshot::Read {
        paused: true,
        auto_objectives_cap: 5,
        inbox_open_count: 0,
        read_at: 1,
    }
}

fn human_focus_permit(store: &MaicieStore, message_id: &str) -> ObjectiveOpeningPermit {
    let observed = ObservedHumanMessage {
        message_id: message_id.to_string(),
        ts: 1_000,
        sender: "humain".to_string(),
        target: "maicie".to_string(),
        body: "Traite cette priorité".to_string(),
    };
    let scope = store.issuer_scope().to_string();
    let canonical_request_sha256 = "a".repeat(64);
    ObjectiveOpeningPermit::human_request(
        HumanRequestOriginAttestation {
            version: 1,
            issuer_scope: scope.clone(),
            canonical_request_sha256: canonical_request_sha256.clone(),
            signature: human_message_content_seal(&observed),
        },
        &observed,
        &scope,
        &canonical_request_sha256,
        AttestationConsumption::NeverConsumed,
    )
    .unwrap()
}

fn human_inbox_limits() -> maicie::bridget_client::BridgetClientLimits {
    maicie::bridget_client::BridgetClientLimits {
        connect_timeout: std::time::Duration::from_secs(2),
        io_timeout: std::time::Duration::from_secs(2),
        max_frame_bytes: 64 * 1024,
    }
}

fn read_json(reader: &mut BufReader<UnixStream>) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) {
    serde_json::to_writer(&mut *writer, &value).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
}

fn serve_one_human_decision(
    socket: PathBuf,
    decision_id: &'static str,
    expect_ack: bool,
) -> (thread::JoinHandle<()>, mpsc::Receiver<()>) {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let server = thread::spawn(move || {
        let listener = UnixListener::bind(socket).unwrap();
        ready_tx.send(()).unwrap();
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        assert_eq!(read_json(&mut reader)["type"], "RoleHandshake");
        write_json(
            &mut writer,
            serde_json::json!({"type":"RoleAccepted","role":"service"}),
        );
        assert_eq!(read_json(&mut reader)["type"], "ServiceHello");
        write_json(
            &mut writer,
            serde_json::json!({"type":"ServiceWelcome","version":1,"horizon_secs":60,
                "issued_at_tolerance_secs":5,"capabilities":["human_inbox_v1"]}),
        );
        assert_eq!(read_json(&mut reader)["type"], "human_inbox_decisions");
        write_json(
            &mut writer,
            serde_json::json!({"type":"human_inbox_decisions_batch","decisions":[{
                "decision_id":decision_id,"item_id":"item-1","dedup_key":"k1",
                "kind":"chain_exhausted","subject":{},"choice":"ack","at":1000
            }]}),
        );
        if expect_ack {
            assert_eq!(
                read_json(&mut reader),
                serde_json::json!({"type":"human_inbox_ack","version":1,"decision_id":decision_id}),
            );
            write_json(
                &mut writer,
                serde_json::json!({"type":"human_inbox_acked","decision_id":decision_id,"acked_at":1001}),
            );
        }
    });
    (server, ready_rx)
}

fn open_auto(
    store: &mut MaicieStore,
    key: &str,
    now: i64,
    depends_on: &[Uuid],
) -> (Uuid, Uuid, Option<Uuid>) {
    let request = DelegateRequest {
        goal: "travail automatique",
        opening_permit: ObjectiveOpeningPermit::auto_generated(),
        explicit_target: Some("prospective"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: false,
        constat_id: None,
        review_target: None,
        suite: SuiteObjective::Aucune,
        depends_on,
        references: &[],
        idempotency_key: key,
        now,
        retry_until: now + 90,
        dedup_retained_until: now + 90,
        max_frame_bytes: 256 * 1024,
    };
    match delegate(
        store,
        durations(),
        "maicie",
        &[candidate("prospective")],
        &request,
    )
    .unwrap()
    {
        DelegateResult::Created(created) => (
            created.objective_id,
            created.delegation_id,
            created.message_id,
        ),
        other => panic!("création attendue : {other:?}"),
    }
}

/// Propriété : une base au schéma précédent migre vers v24 avec ses quatre
/// objets, et une base v24 vierge s'ouvre. La base « v23 » est fabriquée par
/// le code courant puis ramenée à la version précédente sans ses objets v24,
/// faute de binaire v23 disponible sur le banc.
#[test]
fn spec_087_migration_v24_ajoute_focus_consommations_decisions_et_motif() {
    let guard = RootGuard::new("migration");
    let database = guard.path.join("maicie.sqlite3");
    {
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(SCHEMA_VERSION, 24);
    }
    {
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection
            .execute_batch(
                "DROP TABLE focus_queue; DROP TABLE human_origin_consumptions;
                 DROP TABLE human_decisions_applied; DROP TABLE human_inbox_outbox;
                 ALTER TABLE deferred_delegation_dispatch DROP COLUMN deferred_reason;
                 DELETE FROM schema_migrations WHERE version = 24;
                 PRAGMA user_version = 23;",
            )
            .unwrap();
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 23);
    }
    let refused = MaicieStore::open(&database).err();
    assert!(
        matches!(
            refused,
            Some(StoreError::MigrationRequired {
                found: 23,
                supported: 24
            })
        ),
        "une base v23 exige le consentement : {refused:?}"
    );
    let mut store =
        historical_guichet_receptions::open_after_published_migration(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 24);
    store.set_control_snapshot(active(5));
    let (objective_id, _, _) = open_auto(&mut store, "post-migration", 1_000, &[]);
    store.focus_enqueue(objective_id, true, 1_000).unwrap();
    assert_eq!(store.focus_active().unwrap(), Some(objective_id));
    assert!(
        store
            .consume_human_origin("m1", objective_id, 1_000)
            .unwrap()
    );
    assert_eq!(store.deferred_dispatch_reasons().unwrap(), Vec::new());
}

/// Propriété : la file de focus n'a qu'une tête ; remplacer recule l'ancien,
/// mettre en file l'ajoute derrière, fermer promeut.
#[test]
fn spec_087_file_de_focus_une_tete_et_une_file() {
    let guard = RootGuard::new("focus");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(10));
    let (a, _, _) = open_auto(&mut store, "a", 1_000, &[]);
    let (b, _, _) = open_auto(&mut store, "b", 1_001, &[]);
    let (c, _, _) = open_auto(&mut store, "c", 1_002, &[]);
    store.focus_enqueue(a, true, 1_000).unwrap();
    store.focus_enqueue(b, false, 1_001).unwrap();
    assert_eq!(store.focus_active().unwrap(), Some(a));
    assert_eq!(
        store.focus_queue().unwrap(),
        vec![(a, 0), (b, 1)],
        "b est en file derrière a"
    );
    store.focus_enqueue(c, true, 1_002).unwrap();
    assert_eq!(store.focus_queue().unwrap(), vec![(c, 0), (a, 1), (b, 2)]);
    assert_eq!(store.focus_close_current().unwrap(), Some(a));
    assert_eq!(store.focus_queue().unwrap(), vec![(a, 0), (b, 1)]);
    store.focus_remove(b).unwrap();
    assert_eq!(store.focus_queue().unwrap(), vec![(a, 0)]);
    assert_eq!(store.focus_close_current().unwrap(), None);
    assert_eq!(store.focus_active().unwrap(), None);
}

/// Propriété : un focus humain sans agent disponible reste ouvert, sans
/// délégation fictive, puis produit un unique item durable après la durée
/// normale. Avant l'échéance, il ne déclenche rien.
#[test]
fn spec_087_focus_sans_agent_attend_puis_avertit_le_referent() {
    let guard = RootGuard::new("focus-waiting");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    let permit = human_focus_permit(&store, "hmo-focus-waiting");
    let request = DelegateRequest {
        goal: "Réparer l'import CSV",
        opening_permit: permit,
        explicit_target: None,
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: false,
        constat_id: None,
        review_target: None,
        suite: SuiteObjective::Aucune,
        depends_on: &[],
        references: &[],
        idempotency_key: "focus-waiting",
        now: 1_000,
        retry_until: 1_090,
        dedup_retained_until: 1_090,
        max_frame_bytes: 256 * 1024,
    };
    let waiting =
        open_focus_waiting_for_agent(&mut store, &request, "hmo-focus-waiting", true).unwrap();
    assert_eq!(store.focus_active().unwrap(), Some(waiting.objective_id));
    assert!(
        store
            .objective_snapshots(Some(waiting.objective_id))
            .unwrap()[0]
            .delegations
            .is_empty(),
        "aucune délégation ni outbox fictive ne doit être créée"
    );
    assert_eq!(
        reconcile_focus_waiting_agents(&mut store, durations().normal_secs, 1_059).unwrap(),
        0
    );
    assert!(store.pending_human_inbox().unwrap().is_empty());
    assert_eq!(
        reconcile_focus_waiting_agents(&mut store, durations().normal_secs, 1_060).unwrap(),
        1
    );
    let pending = store.pending_human_inbox().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].dedup_key,
        format!("focus-waiting:{}", waiting.objective_id)
    );
    assert_eq!(pending[0].kind, "focus_waiting_agent");
    assert_eq!(
        reconcile_focus_waiting_agents(&mut store, durations().normal_secs, 1_061).unwrap(),
        0,
        "le même focus ne doit pas déposer deux alertes"
    );
}

/// Propriété : un message humain n'ouvre jamais deux fois.
#[test]
fn spec_087_usage_unique_de_l_attestation_humaine() {
    let guard = RootGuard::new("consommation");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(10));
    let (objective_id, _, _) = open_auto(&mut store, "x", 1_000, &[]);
    assert_eq!(
        store.human_origin_consumption("hmo-1").unwrap(),
        AttestationConsumption::NeverConsumed
    );
    assert!(
        store
            .consume_human_origin("hmo-1", objective_id, 1_000)
            .unwrap()
    );
    assert!(
        !store
            .consume_human_origin("hmo-1", objective_id, 1_001)
            .unwrap()
    );
    assert_eq!(
        store.human_origin_consumption("hmo-1").unwrap(),
        AttestationConsumption::AlreadyConsumed
    );
}

/// Propriété : une décision est appliquée une fois, dans une transaction qui
/// l'enregistre ; un rejeu ne réapplique rien ; `cancel` annule la délégation
/// visée ; un choix inconnu reste non appliqué.
#[test]
fn spec_087_decisions_humaines_appliquees_une_seule_fois() {
    let guard = RootGuard::new("decisions");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(10));
    let (_, delegation_id, _) = open_auto(&mut store, "d", 1_000, &[]);
    assert_eq!(
        store
            .apply_human_decision("dec-1", "item-1", "ack", None, 1_010)
            .unwrap(),
        HumanDecisionApplication::Applied
    );
    assert_eq!(
        store
            .apply_human_decision("dec-1", "item-1", "ack", None, 1_011)
            .unwrap(),
        HumanDecisionApplication::Replayed
    );
    assert!(store.human_decision_applied("dec-1").unwrap());
    assert_eq!(
        store
            .apply_human_decision("dec-2", "item-2", "cancel", Some(delegation_id), 1_020)
            .unwrap(),
        HumanDecisionApplication::Applied
    );
    assert!(
        store.pending_delegation_outboxes().unwrap().is_empty(),
        "l'annulation retire l'outbox en attente"
    );
    assert_eq!(
        store
            .apply_human_decision("dec-budget", "item-budget", "raise_budget", None, 1_025)
            .unwrap(),
        HumanDecisionApplication::Applied,
        "le plafond reste côté daemon, mais la décision est acquittable"
    );
    assert!(store.human_decision_applied("dec-budget").unwrap());
    assert_eq!(
        store
            .apply_human_decision("dec-3", "item-3", "reassign:agent-x", None, 1_030)
            .unwrap(),
        HumanDecisionApplication::Unsupported
    );
    assert!(!store.human_decision_applied("dec-3").unwrap());
}

/// Propriété : en pause, la clôture d'un prérequis ne matérialise pas l'outbox
/// du dépendant ; la ligne différée garde le motif et repart à la levée.
/// Mutant : forcer la garde à admettre fait apparaître l'outbox en pause.
#[test]
fn spec_087_le_deblocage_de_dependance_attend_la_levee_de_la_pause() {
    let guard = RootGuard::new("dependance");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(10));
    let (prerequisite, _, _) = open_auto(&mut store, "prereq", 1_000, &[]);
    let (dependent, dependent_delegation, message) =
        open_auto(&mut store, "dependant", 1_001, &[prerequisite]);
    assert_eq!(message, None, "le dépendant attend son prérequis");
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);
    store.set_control_snapshot(paused());
    close(&mut store, prerequisite, "prérequis fini", 1_100).unwrap();
    // La clôture solde l'outbox du prérequis ; en pause, celle du dépendant
    // n'apparaît pas.
    assert_eq!(
        store.pending_delegation_outboxes().unwrap().len(),
        0,
        "aucune outbox neuve en pause"
    );
    assert_eq!(
        store.deferred_dispatch_reasons().unwrap(),
        vec![(dependent_delegation, "pause".to_string())]
    );
    assert_eq!(store.release_ready_dependents(1_200).unwrap(), 0);
    store.set_control_snapshot(active(10));
    assert_eq!(store.release_ready_dependents(1_300).unwrap(), 1);
    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].objective_id, dependent);
    assert!(store.deferred_dispatch_reasons().unwrap().is_empty());
}

/// Propriété : en pause ou en état inconnu, aucune réassignation n'est
/// réduite ; le fait reste au guichet pour la relève suivante.
#[test]
fn spec_087_la_reassignation_est_differee_avant_toute_reduction() {
    let guard = RootGuard::new("reassignation");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    let fact = || FaitReassignation {
        event_id: "evt-1".to_string(),
        request_id: "req-1".to_string(),
        kind: TypeFaitReassignation::TimedOut,
        observed_at: 1_000,
        freshness: FraicheurCoordination::Fresh,
        delivery_hash: None,
    };
    store.set_control_snapshot(paused());
    assert!(matches!(
        store.apply_reassignment_fact(fact()),
        Err(StoreError::Conflict(
            "réassignation différée : pause du référent"
        ))
    ));
    store.set_control_snapshot(ControlSnapshot::Unknown);
    assert!(matches!(
        store.apply_reassignment_fact(fact()),
        Err(StoreError::Conflict(
            "réassignation différée : état de contrôle inconnu"
        ))
    ));
    // Contrôle positif : hors pause, le fait est instruit (ici inconnu, donc
    // NotFound), ce qui prouve que la garde n'est pas un refus systématique.
    store.set_control_snapshot(active(5));
    assert!(matches!(
        store.apply_reassignment_fact(fact()),
        Err(StoreError::NotFound(_))
    ));
}

/// Propriété : en pause, une outbox d'origine automatique n'est même pas
/// tentée (aucune connexion) ; hors pause, elle l'est.
#[test]
fn spec_087_la_releve_ne_tente_pas_les_outboxes_automatiques_en_pause() {
    let guard = RootGuard::new("releve");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(10));
    let (objective_id, _, message_id) = open_auto(&mut store, "o", 1_000, &[]);
    let socket = guard.path.join("absent.sock");
    let limits = maicie::bridget_client::BridgetClientLimits {
        connect_timeout: std::time::Duration::from_millis(50),
        io_timeout: std::time::Duration::from_millis(50),
        max_frame_bytes: 64 * 1024,
    };
    store.set_control_snapshot(paused());
    let report = reconcile_startup_with_limits(&mut store, &socket, limits).unwrap();
    assert!(
        matches!(
            report.actions.as_slice(),
            [ReconcileAction::Differee { objective_id: o, message_id: m, motif: "pause" }]
                if *o == objective_id && Some(*m) == message_id
        ),
        "{:?}",
        report.actions
    );
    store.set_control_snapshot(active(10));
    let report = reconcile_startup_with_limits(&mut store, &socket, limits).unwrap();
    assert!(
        matches!(
            report.actions.as_slice(),
            [ReconcileAction::TransportIndisponible { .. }]
        ),
        "hors pause la socket est tentée : {:?}",
        report.actions
    );
}

/// Propriété : le compte du budget ne voit que les objectifs ouverts d'origine
/// automatique ; une clôture le fait redescendre.
#[test]
fn spec_087_le_compte_du_budget_suit_les_objectifs_automatiques_ouverts() {
    let guard = RootGuard::new("budget");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(2));
    let (a, _, _) = open_auto(&mut store, "a", 1_000, &[]);
    let (_, _, _) = open_auto(&mut store, "b", 1_001, &[]);
    assert_eq!(store.count_open_auto_generated_objectives().unwrap(), 2);
    let request = DelegateRequest {
        goal: "troisième",
        opening_permit: ObjectiveOpeningPermit::auto_generated(),
        explicit_target: Some("prospective"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: false,
        constat_id: None,
        review_target: None,
        suite: SuiteObjective::Aucune,
        depends_on: &[],
        references: &[],
        idempotency_key: "c",
        now: 1_002,
        retry_until: 1_100,
        dedup_retained_until: 1_100,
        max_frame_bytes: 256 * 1024,
    };
    assert!(matches!(
        delegate(
            &mut store,
            durations(),
            "maicie",
            &[candidate("prospective")],
            &request
        ),
        Err(maicie::app::DelegateError::BudgetReached { cap: 2, open: 2 })
    ));
    close(&mut store, a, "fini", 1_100).unwrap();
    assert_eq!(store.count_open_auto_generated_objectives().unwrap(), 1);
    assert!(matches!(
        delegate(
            &mut store,
            durations(),
            "maicie",
            &[candidate("prospective")],
            &request
        ),
        Ok(DelegateResult::Created(_))
    ));
}

/// Propriété : le dépôt vers la boîte est durable et idempotent par clé, et la
/// référence `focus:<id>` s'ajoute au message d'une outbox encore préparée.
#[test]
fn spec_087_depot_humain_durable_et_reference_focus() {
    let guard = RootGuard::new("depot");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    store.set_control_snapshot(active(10));
    assert!(
        store
            .enqueue_human_inbox(
                "k1",
                "chain_exhausted",
                "{}",
                r#"{"summary":"s"}"#,
                &["ack".to_string()],
                1_000
            )
            .unwrap()
    );
    assert!(
        !store
            .enqueue_human_inbox(
                "k1",
                "chain_exhausted",
                "{}",
                "{}",
                &["ack".to_string()],
                1_001
            )
            .unwrap()
    );
    assert_eq!(store.pending_human_inbox().unwrap().len(), 1);
    store.mark_human_inbox_deposited("k1").unwrap();
    assert!(store.pending_human_inbox().unwrap().is_empty());
    assert!(
        store
            .enqueue_human_inbox("", "x", "{}", "{}", &["ack".to_string()], 1_000)
            .is_err()
    );

    let (objective_id, _, _) = open_auto(&mut store, "f", 1_000, &[]);
    assert_eq!(
        store
            .add_focus_reference_to_pending_outboxes(objective_id)
            .unwrap(),
        1
    );
    let pending = store.pending_delegation_outboxes().unwrap();
    let message: serde_json::Value = serde_json::from_slice(&pending[0].message_bytes).unwrap();
    assert_eq!(
        message["references"],
        serde_json::json!([format!("focus:{objective_id}")])
    );
    assert_eq!(
        store
            .add_focus_reference_to_pending_outboxes(objective_id)
            .unwrap(),
        0,
        "idempotent"
    );
}

/// Propriété : un arrêt entre la relève et l'effet ne consomme pas la
/// décision. Elle revient ensuite et n'est acquittée qu'après le commit.
#[test]
fn spec_087_releve_humaine_rejoue_apres_crash_avant_effet() {
    let guard = RootGuard::new("human-decision-crash");
    let mut store = MaicieStore::open(guard.path.join("maicie.sqlite3")).unwrap();
    let socket = guard.path.join("human.sock");
    let (first, first_ready) = serve_one_human_decision(socket.clone(), "decision-crash-1", false);
    first_ready.recv().unwrap();
    let first_result = reconcile_human_inbox_observed_with_limits(
        &mut store,
        &socket,
        human_inbox_limits(),
        1_000,
        |phase| {
            assert_eq!(phase, HumanInboxReconcilePhase::AfterFetchBeforeApply);
            Err(ReconcileError::InvalidSnapshot("crash simulé avant effet"))
        },
    );
    assert!(
        matches!(
            first_result,
            Err(ReconcileError::InvalidSnapshot("crash simulé avant effet"))
        ),
        "résultat réel : {first_result:?}"
    );
    first.join().unwrap();
    assert!(!store.human_decision_applied("decision-crash-1").unwrap());
    fs::remove_file(&socket).unwrap();

    let (second, second_ready) = serve_one_human_decision(socket.clone(), "decision-crash-1", true);
    second_ready.recv().unwrap();
    let report =
        reconcile_human_inbox_with_limits(&mut store, &socket, human_inbox_limits(), 1_001)
            .unwrap();
    second.join().unwrap();
    assert_eq!(
        (report.applied, report.acked, report.unsupported),
        (1, 1, 0)
    );
    assert!(store.human_decision_applied("decision-crash-1").unwrap());
}
