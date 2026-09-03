use bridget_daemon::execution_store::{
    ContinuationMode, ContinuationOutcome, ContinuationReservation, ExecutionStore,
    ExecutionUsageSample,
};
use bridget_daemon::fleet::{AutonomyBudgetPolicy, AutonomyRuntimeState, evaluate_autonomy_budget};
use bridget_daemon::{
    GovernedContinuation, GovernedContinuationSource, reserve_governed_continuation,
};
use bridget_transport::protocol::{ExecutionBudgetOutcome, UsageTokens};

fn tokens(input: u64, output: u64) -> UsageTokens {
    UsageTokens {
        input_tokens: input,
        output_tokens: output,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
    }
}

fn completed(
    store: &ExecutionStore,
    submission: &str,
    execution: &str,
    agent: &str,
    created_at: i64,
    completed_at: i64,
) {
    store
        .record_starting(submission, execution, agent, created_at)
        .unwrap();
    assert!(matches!(
        store
            .transition_if_current(
                execution,
                "starting",
                0,
                1,
                "completed",
                "completed",
                completed_at
            )
            .unwrap(),
        bridget_daemon::ConditionalTransition::Applied(_)
    ));
}

#[test]
fn agrege_duree_usage_et_descendants_sans_zero_invente() {
    let store = ExecutionStore::open_in_memory().unwrap();
    completed(
        &store,
        "submission-parent",
        "execution-parent",
        "agent-a",
        100,
        110,
    );
    completed(
        &store,
        "submission-child",
        "execution-child",
        "agent-a",
        120,
        130,
    );
    assert_eq!(
        store
            .record_continuation(
                "execution-child",
                1,
                "execution-parent",
                ContinuationMode::Reconstructed,
                Some("resume_not_attested"),
                131,
            )
            .unwrap(),
        ContinuationOutcome::Applied
    );
    assert!(
        store
            .record_execution_usage(&ExecutionUsageSample {
                execution_id: "execution-parent".to_string(),
                generation: 1,
                tokens: tokens(10, 3),
                source: "claude-stream-json".to_string(),
                observed_at: 115,
            })
            .unwrap()
    );
    assert!(
        store
            .record_execution_usage(&ExecutionUsageSample {
                execution_id: "execution-child".to_string(),
                generation: 1,
                tokens: tokens(7, 2),
                source: "claude-stream-json".to_string(),
                observed_at: 132,
            })
            .unwrap()
    );

    let facts = store
        .execution_budget_facts("execution-parent", 200)
        .unwrap()
        .unwrap();
    assert_eq!(facts.duration_secs, 100);
    assert_eq!(facts.descendants, 1);
    assert_eq!(facts.usage.as_ref().map(|usage| usage.turns), Some(2));
    assert_eq!(
        facts.usage.as_ref().map(|usage| usage.facturable_tokens),
        Some(22)
    );

    assert_eq!(
        evaluate_autonomy_budget(
            AutonomyBudgetPolicy {
                max_duration_secs: Some(100),
                ..Default::default()
            },
            &facts,
            AutonomyRuntimeState::Ready,
        ),
        Some(ExecutionBudgetOutcome::BudgetLimit)
    );
    assert_eq!(
        evaluate_autonomy_budget(
            AutonomyBudgetPolicy {
                max_facturable_tokens: Some(22),
                ..Default::default()
            },
            &facts,
            AutonomyRuntimeState::Ready,
        ),
        Some(ExecutionBudgetOutcome::UsageLimit)
    );
    assert_eq!(
        evaluate_autonomy_budget(
            AutonomyBudgetPolicy {
                max_descendants: Some(1),
                ..Default::default()
            },
            &facts,
            AutonomyRuntimeState::Ready,
        ),
        Some(ExecutionBudgetOutcome::Blocked)
    );
    assert_eq!(
        evaluate_autonomy_budget(Default::default(), &facts, AutonomyRuntimeState::Paused),
        Some(ExecutionBudgetOutcome::Paused)
    );
    assert_eq!(
        evaluate_autonomy_budget(Default::default(), &facts, AutonomyRuntimeState::Terminated),
        Some(ExecutionBudgetOutcome::Terminated)
    );
}

#[test]
fn continuation_exige_inactivite_budget_et_reservation_sans_course() {
    let store = ExecutionStore::open_in_memory().unwrap();
    completed(
        &store,
        "submission-parent",
        "execution-parent",
        "agent-a",
        100,
        110,
    );

    assert_eq!(
        reserve_governed_continuation(
            &store,
            Default::default(),
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::InactiveParent,
            "execution-parent",
            1,
            1,
            "continuation-a",
            110,
            120,
        )
        .unwrap(),
        GovernedContinuation::Reserved
    );
    assert_eq!(
        reserve_governed_continuation(
            &store,
            Default::default(),
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::InactiveParent,
            "execution-parent",
            1,
            1,
            "continuation-b",
            110,
            121,
        )
        .unwrap(),
        GovernedContinuation::Reservation(ContinuationReservation::AlreadyReserved)
    );
    assert_eq!(
        reserve_governed_continuation(
            &store,
            Default::default(),
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::InactiveParent,
            "execution-parent",
            1,
            1,
            "continuation-a",
            110,
            122,
        )
        .unwrap(),
        GovernedContinuation::Reservation(ContinuationReservation::Replayed)
    );

    completed(
        &store,
        "submission-budget",
        "execution-budget",
        "agent-b",
        100,
        110,
    );
    assert_eq!(
        reserve_governed_continuation(
            &store,
            AutonomyBudgetPolicy {
                max_duration_secs: Some(1),
                ..Default::default()
            },
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::InactiveParent,
            "execution-budget",
            1,
            1,
            "continuation-budget",
            110,
            120,
        )
        .unwrap(),
        GovernedContinuation::Budget(ExecutionBudgetOutcome::BudgetLimit)
    );

    store
        .record_starting("submission-running", "execution-running", "agent-c", 100)
        .unwrap();
    assert_eq!(
        reserve_governed_continuation(
            &store,
            Default::default(),
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::InactiveParent,
            "execution-running",
            1,
            0,
            "continuation-running",
            100,
            120,
        )
        .unwrap(),
        GovernedContinuation::Reservation(ContinuationReservation::NotInactive)
    );
    assert_eq!(
        reserve_governed_continuation(
            &store,
            Default::default(),
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::RecoveryAfterIdleWrapper,
            "execution-running",
            1,
            0,
            "continuation-recovery-after-idle",
            100,
            120,
        )
        .unwrap(),
        GovernedContinuation::Reserved
    );

    completed(
        &store,
        "submission-race",
        "execution-race",
        "agent-d",
        100,
        110,
    );
    store
        .record_starting(
            "submission-concurrent",
            "execution-concurrent",
            "agent-d",
            115,
        )
        .unwrap();
    assert_eq!(
        reserve_governed_continuation(
            &store,
            Default::default(),
            AutonomyRuntimeState::Ready,
            GovernedContinuationSource::InactiveParent,
            "execution-race",
            1,
            1,
            "continuation-race",
            110,
            120,
        )
        .unwrap(),
        GovernedContinuation::Reservation(ContinuationReservation::ConcurrentExecution)
    );
}
