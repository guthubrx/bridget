use bridget_daemon::execution_store::{
    ContinuationMode, ControlCommandStatus, ControlReservation, ExecutionRecoveryOutcome,
    ProviderBindingOutcome,
};
use bridget_daemon::{ConditionalTransition, ExecutionStore};
use bridget_transport::protocol::{
    ExecutionProviderContext, ProjectReference, ProviderObservation, ProviderOperation,
};
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn migration_execution_store_est_additive_et_idempotente() {
    let store = ExecutionStore::open_in_memory().expect("magasin en mémoire");
    assert_eq!(store.schema_version().expect("version"), 10);
}

#[test]
fn migration_execution_store_garde_les_tables_heritees_et_rejoue_sans_effet() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-execution-store-{stamp}.db"));
    {
        let legacy = Connection::open(&path).unwrap();
        legacy.execute_batch("CREATE TABLE legacy_messages (id TEXT PRIMARY KEY); INSERT INTO legacy_messages VALUES ('m-1');").unwrap();
    }
    let first = ExecutionStore::open(&path).expect("migration additive");
    assert_eq!(first.schema_version().unwrap(), 10);
    drop(first);
    let second = ExecutionStore::open(&path).expect("migration idempotente");
    assert_eq!(second.schema_version().unwrap(), 10);
    drop(second);
    let legacy = Connection::open(&path).unwrap();
    let preserved: i64 = legacy
        .query_row("SELECT COUNT(*) FROM legacy_messages", [], |row| row.get(0))
        .unwrap();
    assert_eq!(preserved, 1);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn migration_execution_store_complete_les_lignes_heritees_sans_les_effacer() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-execution-legacy-{stamp}.db"));
    let legacy = Connection::open(&path).unwrap();
    legacy.execute_batch("CREATE TABLE executions (execution_id TEXT PRIMARY KEY, submission_id TEXT NOT NULL, state TEXT NOT NULL); INSERT INTO executions VALUES ('e-1', 's-1', 'running');").unwrap();
    drop(legacy);
    let store = ExecutionStore::open(&path).expect("migration héritée");
    assert_eq!(store.schema_version().unwrap(), 10);
    drop(store);
    let check = Connection::open(&path).unwrap();
    let revision_columns: i64 = check
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('executions') WHERE name = 'revision'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(revision_columns, 1);
    let preserved: i64 = check
        .query_row(
            "SELECT COUNT(*) FROM executions WHERE execution_id = 'e-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(preserved, 1);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn reprise_apres_crash_retrouve_les_executions_non_terminales() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-execution-crash-{stamp}.db"));
    let store = ExecutionStore::open(&path).unwrap();
    store
        .record_starting("submission-1", "execution-1", "agent-a", 42)
        .unwrap();
    drop(store);
    let recovered = ExecutionStore::open(&path).unwrap();
    assert_eq!(
        recovered.recoverable_execution_ids().unwrap(),
        vec!["execution-1"]
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn spec_079_reconstruction_conserve_message_soumission_projet_et_lignee() {
    let store = ExecutionStore::open_in_memory().unwrap();
    let project = ProjectReference {
        project_id: "project-079".to_string(),
        binding_generation: 7,
    };
    let mut message =
        bridget_core::BridgetMessage::new("humain", "coordinateur", "reprends exactement ceci");
    message.id = "message-079".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    assert!(
        store
            .admit_starting_message_for_project(
                &message,
                "execution-079-parent",
                Some(&project),
                40,
            )
            .unwrap()
    );
    store
        .transition_if_current(
            "execution-079-parent",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            41,
        )
        .unwrap();

    let outcome = store
        .reconstruct_active_for_agent(
            "coordinateur",
            "instance-reprise",
            "execution-079-enfant",
            50,
        )
        .unwrap();
    let reconstructed = match outcome {
        ExecutionRecoveryOutcome::Reconstructed(value) => value,
        other => panic!("reconstruction attendue, reçu {other:?}"),
    };
    assert_eq!(reconstructed.parent_execution_id, "execution-079-parent");
    assert_eq!(reconstructed.message, message);
    assert_eq!(reconstructed.snapshot.project, Some(project));
    assert_eq!(reconstructed.snapshot.generation, 2);
    assert_eq!(reconstructed.snapshot.state, "starting");
    assert!(matches!(
        store.execution_snapshot("execution-079-parent").unwrap(),
        Some(snapshot) if snapshot.state == "unreachable" && snapshot.revision == 2
    ));
    let continuation = store
        .continuation_for("execution-079-enfant")
        .unwrap()
        .unwrap();
    assert_eq!(continuation.parent_execution_id, "execution-079-parent");
    assert_eq!(continuation.mode, ContinuationMode::Reconstructed);
}

#[test]
fn spec_079_payload_absent_ferme_le_parent_sans_inventer() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-legacy", "execution-legacy", "coordinateur", 40)
        .unwrap();
    assert!(matches!(
        store
            .reconstruct_active_for_agent(
                "coordinateur",
                "instance-reprise",
                "execution-ne-doit-pas-exister",
                50,
            )
            .unwrap(),
        ExecutionRecoveryOutcome::PayloadUnavailable {
            parent_execution_id
        } if parent_execution_id == "execution-legacy"
    ));
    assert!(matches!(
        store.execution_snapshot("execution-legacy").unwrap(),
        Some(snapshot) if snapshot.state == "unreachable" && snapshot.revision == 1
    ));
    assert_eq!(
        store
            .execution_snapshot("execution-ne-doit-pas-exister")
            .unwrap(),
        None
    );
}

#[test]
fn spec_079_deux_actifs_refusent_une_reconstruction_aveugle() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-a", "execution-a", "coordinateur", 40)
        .unwrap();
    store
        .record_starting("submission-b", "execution-b", "coordinateur", 41)
        .unwrap();
    assert!(matches!(
        store
            .reconstruct_active_for_agent(
                "coordinateur",
                "instance-reprise",
                "execution-enfant",
                50,
            )
            .unwrap(),
        ExecutionRecoveryOutcome::Ambiguous { execution_ids }
            if execution_ids == vec!["execution-a", "execution-b"]
    ));
    assert_eq!(store.recoverable_execution_ids().unwrap().len(), 2);
}

#[test]
fn reference_projet_du_snapshot_survit_au_redemarrage_sans_reduction() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-execution-project-{stamp}.db"));
    let project = ProjectReference {
        project_id: "project-snapshot".to_string(),
        binding_generation: 2,
    };
    let store = ExecutionStore::open(&path).unwrap();
    store
        .record_starting_for_project(
            "submission-project",
            "execution-project",
            "agent-a",
            Some(&project),
            42,
        )
        .unwrap();
    drop(store);

    let reopened = ExecutionStore::open(&path).unwrap();
    assert_eq!(
        reopened
            .execution_snapshot("execution-project")
            .unwrap()
            .unwrap()
            .project,
        Some(project)
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn transition_conditionnelle_refuse_un_evenement_tardif() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-1", "execution-1", "agent-a", 42)
        .unwrap();
    assert!(matches!(
        store.transition_if_current("execution-1", "starting", 0, 1, "running", "provider_accepted", 43),
        Ok(ConditionalTransition::Applied(snapshot)) if snapshot.revision == 1 && snapshot.state == "running"
    ));
    assert!(matches!(
        store.transition_if_current("execution-1", "starting", 0, 1, "running", "provider_accepted", 44),
        Ok(ConditionalTransition::Rejected(snapshot)) if snapshot.revision == 1 && snapshot.generation == 1 && snapshot.state == "running"
    ));
}

#[test]
fn demarrage_sans_preuve_devient_injoignable_a_sa_borne() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-1", "execution-1", "agent-a", 10)
        .unwrap();

    assert_eq!(store.expire_starting_before(10, 20).unwrap(), 0);
    assert_eq!(store.expire_starting_before(11, 20).unwrap(), 1);

    assert!(matches!(
        store.execution_snapshot("execution-1"),
        Ok(Some(snapshot)) if snapshot.state == "unreachable" && snapshot.revision == 1
    ));
    assert_eq!(store.expire_starting_before(11, 21).unwrap(), 0);
}

#[test]
fn projection_par_agent_distingue_file_et_attente_durable() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store.admit_submission("queued", "agent-a", 0, 8).unwrap();
    store
        .record_starting("started", "execution-1", "agent-a", 10)
        .unwrap();
    store
        .transition_if_current(
            "execution-1",
            "starting",
            0,
            1,
            "waiting_approval",
            "approval_requested",
            11,
        )
        .unwrap();
    let summaries = store.agent_execution_summaries().unwrap();
    let summary = summaries.get("agent-a").unwrap();
    assert_eq!(summary.state.as_deref(), Some("waiting_approval"));
    assert_eq!(summary.updated_at, Some(11));
    assert_eq!(summary.queue_depth, 1);
}

#[test]
fn admission_de_declenchement_ne_cree_pas_de_faux_element_de_file() {
    let store = ExecutionStore::open_in_memory().unwrap();
    assert!(
        store
            .admit_starting("trigger", "execution-trigger", "agent-a", 10)
            .unwrap()
    );
    assert!(
        !store
            .admit_starting("trigger", "execution-trigger", "agent-a", 11)
            .unwrap()
    );
    let summary = store
        .agent_execution_summaries()
        .unwrap()
        .remove("agent-a")
        .unwrap();
    assert_eq!(summary.state.as_deref(), Some("starting"));
    assert_eq!(summary.queue_depth, 0);
    assert_eq!(store.take_next_submission("agent-a").unwrap(), None);
}

#[test]
fn commande_de_controle_rejouee_ne_declenche_pas_deux_actions() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-1", "execution-1", "agent-a", 10)
        .unwrap();
    assert_eq!(
        store
            .reserve_control_command(
                "client:alice",
                "commande-1",
                "execution-1",
                b"commande-canonique",
                11,
                40
            )
            .unwrap(),
        ControlReservation::New
    );
    assert!(
        store
            .mark_control_dispatched("client:alice", "commande-1", 12)
            .unwrap()
    );
    assert!(matches!(
        store
            .reserve_control_command("client:alice", "commande-1", "execution-1", b"commande-canonique", 13, 40)
            .unwrap(),
        ControlReservation::Replayed(record)
            if record.status == ControlCommandStatus::Dispatched
    ));
    assert_eq!(
        store
            .reserve_control_command(
                "client:alice",
                "commande-1",
                "execution-1",
                b"commande-differente",
                13,
                40
            )
            .unwrap(),
        ControlReservation::EnvelopeMismatch
    );
    assert!(
        store
            .resolve_control_command("client:alice", "commande-1", true, None, 14)
            .unwrap()
    );
    assert!(matches!(
        store.lookup_control_command("client:alice", "commande-1", 15).unwrap(),
        Some(record) if record.status == ControlCommandStatus::Accepted
    ));
}

fn provider_context(thread: Option<&str>, turn: Option<&str>) -> ExecutionProviderContext {
    ExecutionProviderContext {
        execution_id: "execution-1".to_string(),
        generation: 1,
        provider_kind: "cursor".to_string(),
        execution_path: "acp".to_string(),
        observation: ProviderObservation {
            binary_path: "/opt/cursor-agent".to_string(),
            binary_version: "1.2.3".to_string(),
            binary_digest: "a".repeat(64),
            contract_version: "acp-v1".to_string(),
            operations: vec![ProviderOperation::Interrupt],
        },
        provider_session_id: Some("session-1".to_string()),
        provider_thread_id: thread.map(str::to_string),
        provider_turn_id: turn.map(str::to_string),
        observed_at: 20,
    }
}

#[test]
fn provider_binding_verifie_generation_thread_et_tour_sans_branche_cursor() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-1", "execution-1", "agent-a", 10)
        .unwrap();

    assert_eq!(
        store
            .record_provider_context(&provider_context(Some("thread-1"), Some("turn-1")))
            .unwrap(),
        ProviderBindingOutcome::Applied
    );
    assert_eq!(
        store
            .record_provider_context(&provider_context(Some("thread-1"), Some("turn-1")))
            .unwrap(),
        ProviderBindingOutcome::Applied,
        "la même observation est idempotente"
    );

    let mut stale = provider_context(Some("thread-1"), Some("turn-1"));
    stale.generation = 2;
    assert_eq!(
        store.record_provider_context(&stale).unwrap(),
        ProviderBindingOutcome::GenerationMismatch
    );
    assert_eq!(
        store
            .record_provider_context(&provider_context(Some("thread-autre"), Some("turn-1")))
            .unwrap(),
        ProviderBindingOutcome::Incompatible,
        "un thread divergent ne peut pas reprendre l'exécution"
    );
    assert_eq!(
        store
            .record_provider_context(&provider_context(Some("thread-1"), Some("turn-autre")))
            .unwrap(),
        ProviderBindingOutcome::Incompatible,
        "un tour divergent ne peut pas piloter la même exécution"
    );
}
