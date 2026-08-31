use bridget_core::{Execution, ExecutionReason, ExecutionState};

fn apply(execution: &mut Execution, next: ExecutionState, reason: ExecutionReason, at: u64) {
    let state = execution.state;
    let revision = execution.revision;
    execution
        .transition(state, revision, next, reason, at)
        .unwrap();
}

#[test]
fn cycle_runtime_expose_running_attentes_interruption_et_indisponibilite() {
    let mut execution = Execution::queued("execution-1", "submission-1", "agent-1", 1);
    apply(
        &mut execution,
        ExecutionState::Starting,
        ExecutionReason::Delivered,
        1,
    );
    apply(
        &mut execution,
        ExecutionState::Running,
        ExecutionReason::ProviderAccepted,
        2,
    );
    apply(
        &mut execution,
        ExecutionState::WaitingApproval,
        ExecutionReason::PermissionRequired,
        3,
    );
    apply(
        &mut execution,
        ExecutionState::Running,
        ExecutionReason::ProviderAccepted,
        4,
    );
    apply(
        &mut execution,
        ExecutionState::WaitingUserInput,
        ExecutionReason::UserInputRequired,
        5,
    );
    apply(
        &mut execution,
        ExecutionState::Running,
        ExecutionReason::ProviderAccepted,
        6,
    );
    apply(
        &mut execution,
        ExecutionState::Interrupting,
        ExecutionReason::Interrupted,
        7,
    );
    apply(
        &mut execution,
        ExecutionState::Interrupted,
        ExecutionReason::Interrupted,
        8,
    );
    assert!(execution.state.is_terminal());

    let mut unreachable = Execution::queued("execution-2", "submission-2", "agent-1", 1);
    apply(
        &mut unreachable,
        ExecutionState::Starting,
        ExecutionReason::Delivered,
        1,
    );
    apply(
        &mut unreachable,
        ExecutionState::Unreachable,
        ExecutionReason::ProviderUnavailable,
        2,
    );
    assert!(unreachable.state.is_terminal());
}
