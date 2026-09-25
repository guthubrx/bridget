use bridget_daemon::ExecutionStore;
use bridget_daemon::execution_store::{ContinuationMode, ContinuationOutcome};

#[test]
fn continuation_preserve_ascendance_et_refuse_generation_ou_parent_incompatibles() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-parent", "execution-parent", "agent-a", 10)
        .unwrap();
    store
        .record_starting("submission-child", "execution-child", "agent-a", 11)
        .unwrap();
    store
        .record_starting(
            "submission-parent-other",
            "execution-parent-autre",
            "agent-a",
            11,
        )
        .unwrap();

    assert_eq!(
        store
            .record_continuation(
                "execution-child",
                1,
                "execution-parent",
                ContinuationMode::Native,
                None,
                12,
            )
            .unwrap(),
        ContinuationOutcome::Applied,
    );
    assert_eq!(
        store
            .record_continuation(
                "execution-child",
                1,
                "execution-parent",
                ContinuationMode::Native,
                None,
                13,
            )
            .unwrap(),
        ContinuationOutcome::Applied,
        "le même fait est idempotent",
    );
    assert_eq!(
        store
            .record_continuation(
                "execution-child",
                2,
                "execution-parent",
                ContinuationMode::Native,
                None,
                14,
            )
            .unwrap(),
        ContinuationOutcome::GenerationMismatch,
    );
    assert_eq!(
        store
            .record_continuation(
                "execution-child",
                1,
                "execution-parent-autre",
                ContinuationMode::Forked,
                None,
                15,
            )
            .unwrap(),
        ContinuationOutcome::Incompatible,
    );
}

#[test]
fn reconstruction_est_explicite_et_ne_se_fait_pas_passer_pour_natif() {
    let store = ExecutionStore::open_in_memory().unwrap();
    store
        .record_starting("submission-parent", "execution-parent", "agent-a", 10)
        .unwrap();
    store
        .record_starting("submission-child", "execution-child", "agent-a", 11)
        .unwrap();

    assert_eq!(
        store
            .record_continuation(
                "execution-child",
                1,
                "execution-parent",
                ContinuationMode::Reconstructed,
                Some("resume_not_attested"),
                12,
            )
            .unwrap(),
        ContinuationOutcome::Applied,
    );
    let continuation = store.continuation_for("execution-child").unwrap().unwrap();
    assert_eq!(continuation.mode, ContinuationMode::Reconstructed);
    assert_eq!(continuation.reason.as_deref(), Some("resume_not_attested"));
    assert_ne!(continuation.mode, ContinuationMode::Native);
    let summary = store.agent_execution_summaries().unwrap();
    assert_eq!(
        summary["agent-a"].continuation_mode.as_deref(),
        Some("reconstructed")
    );
}
