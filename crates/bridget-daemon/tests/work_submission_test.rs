use bridget_daemon::{ExecutionStore, QueuedSubmission};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn admission_est_idempotente_fifo_par_priorite_et_reprise_apres_redemarrage() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-work-submission-{stamp}.db"));
    let store = ExecutionStore::open(&path).unwrap();
    assert!(store.admit_submission("low", "agent-a", 1, 30).unwrap());
    assert!(
        store
            .admit_submission("high-late", "agent-a", 9, 20)
            .unwrap()
    );
    assert!(
        store
            .admit_submission("high-early", "agent-a", 9, 10)
            .unwrap()
    );
    assert!(
        !store
            .admit_submission("high-early", "agent-a", 9, 10)
            .unwrap()
    );
    drop(store);

    let restarted = ExecutionStore::open(&path).unwrap();
    assert_eq!(
        restarted.take_next_submission("agent-a").unwrap(),
        Some(QueuedSubmission {
            submission_id: "high-early".to_string(),
            target_agent: "agent-a".to_string(),
            priority: 9,
            enqueued_at: 10,
            message: None,
        })
    );
    assert_eq!(
        restarted
            .take_next_submission("agent-a")
            .unwrap()
            .unwrap()
            .submission_id,
        "high-late"
    );
    assert_eq!(
        restarted
            .take_next_submission("agent-a")
            .unwrap()
            .unwrap()
            .submission_id,
        "low"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn queue_persiste_le_message_exact_pour_un_declenchement_apres_redemarrage() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-work-message-{stamp}.db"));
    let store = ExecutionStore::open(&path).unwrap();
    let mut message = bridget_core::BridgetMessage::new("humain", "agent-a", "à faire plus tard");
    message.id = "queued-message".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::QueueOnly);
    message.references = vec!["delegation-1".to_string()];
    assert!(store.admit_message_submission(&message, 4, 10).unwrap());
    drop(store);

    let restarted = ExecutionStore::open(&path).unwrap();
    let queued = restarted.take_next_submission("agent-a").unwrap().unwrap();
    assert_eq!(queued.message, Some(message));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn demarrage_persiste_le_message_exact_pour_reconciliation_apres_redemarrage() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bridget-start-message-{stamp}.db"));
    let store = ExecutionStore::open(&path).unwrap();
    let mut message = bridget_core::BridgetMessage::new("humain", "agent-a", "démarrer maintenant");
    message.id = "starting-message".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    assert!(
        store
            .admit_starting_message(&message, "execution-starting", 10)
            .unwrap()
    );
    drop(store);

    let restarted = ExecutionStore::open(&path).unwrap();
    assert_eq!(
        restarted.execution_message("execution-starting").unwrap(),
        Some(message)
    );
    std::fs::remove_file(path).unwrap();
}
