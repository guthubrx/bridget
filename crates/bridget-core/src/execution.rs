//! Machine pure des soumissions et exécutions Bridget.
//!
//! Ce module ne pilote aucun fournisseur. Il exprime les transitions durables
//! afin que le daemon puisse les persister et les comparer atomiquement.

use crate::MessageIntent;
use serde::{Deserialize, Serialize};

/// Etat durable d une soumission avant et pendant son admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionState {
    Prepared,
    Admitted,
    Queued,
    BoundToExecution,
    Rejected,
    Cancelled,
}

impl SubmissionState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Rejected | Self::Cancelled)
    }
}

/// Etat runtime canonique. Il ne décrit ni le métier le service compagnon ni le transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Queued,
    Starting,
    Running,
    WaitingApproval,
    WaitingUserInput,
    Interrupting,
    Interrupted,
    Completed,
    Failed,
    Unreachable,
}

impl ExecutionState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Interrupted | Self::Completed | Self::Failed | Self::Unreachable
        )
    }
}

/// Motif fermé associé à une transition. Aucun corps de message ne passe ici.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionReason {
    Accepted,
    Delivered,
    ProviderAccepted,
    MessageVisible,
    Completed,
    Interrupted,
    ProviderFailed,
    ProviderUnavailable,
    DeadlineExceeded,
    PermissionRequired,
    UserInputRequired,
    CapabilityMissing,
    StaleEvent,
    RevisionMismatch,
    InvalidTransition,
}

/// Erreur de transition pure, rendue persistable par le daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionTransitionError {
    StateMismatch {
        expected: ExecutionState,
        actual: ExecutionState,
    },
    RevisionMismatch {
        expected: u64,
        actual: u64,
    },
    TerminalState {
        state: ExecutionState,
    },
    InvalidTransition {
        from: ExecutionState,
        to: ExecutionState,
    },
}

/// Travail logique accepté par Bridget. Les références le service compagnon sont opaques.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkSubmission {
    pub submission_id: String,
    pub origin: Option<crate::MessageOrigin>,
    pub intent: Option<MessageIntent>,
    pub priority_class: String,
    pub objective_id: Option<String>,
    pub delegation_id: Option<String>,
    pub requested_agent: String,
    pub pinned_instance_id: Option<String>,
    pub generation: Option<u64>,
    pub payload_digest: String,
    pub delivery_deadline_at: Option<u64>,
    pub execution_deadline_at: Option<u64>,
    pub fallback_policy: Option<String>,
    pub created_at: u64,
    pub state: SubmissionState,
    pub revision: u64,
}

impl WorkSubmission {
    pub fn prepared(
        submission_id: impl Into<String>,
        requested_agent: impl Into<String>,
        payload_digest: impl Into<String>,
        created_at: u64,
    ) -> Self {
        Self {
            submission_id: submission_id.into(),
            origin: None,
            intent: None,
            priority_class: "normal".to_string(),
            objective_id: None,
            delegation_id: None,
            requested_agent: requested_agent.into(),
            pinned_instance_id: None,
            generation: None,
            payload_digest: payload_digest.into(),
            delivery_deadline_at: None,
            execution_deadline_at: None,
            fallback_policy: None,
            created_at,
            state: SubmissionState::Prepared,
            revision: 0,
        }
    }
}

/// Cycle runtime durable distinct de la soumission et de la livraison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Execution {
    pub execution_id: String,
    pub submission_id: String,
    pub agent_instance_id: String,
    pub generation: u64,
    pub state: ExecutionState,
    pub reason: ExecutionReason,
    pub started_at: Option<u64>,
    pub last_progress_at: Option<u64>,
    pub ended_at: Option<u64>,
    pub provider_binding_id: Option<String>,
    pub revision: u64,
}

impl Execution {
    pub fn queued(
        execution_id: impl Into<String>,
        submission_id: impl Into<String>,
        agent_instance_id: impl Into<String>,
        generation: u64,
    ) -> Self {
        Self {
            execution_id: execution_id.into(),
            submission_id: submission_id.into(),
            agent_instance_id: agent_instance_id.into(),
            generation,
            state: ExecutionState::Queued,
            reason: ExecutionReason::Accepted,
            started_at: None,
            last_progress_at: None,
            ended_at: None,
            provider_binding_id: None,
            revision: 0,
        }
    }

    /// Applique une transition compare-and-set. Les états terminaux sont
    /// monotones : même une réconciliation tardive ne les rouvre pas.
    pub fn transition(
        &mut self,
        expected_state: ExecutionState,
        expected_revision: u64,
        next_state: ExecutionState,
        reason: ExecutionReason,
        observed_at: u64,
    ) -> Result<(), ExecutionTransitionError> {
        if self.revision != expected_revision {
            return Err(ExecutionTransitionError::RevisionMismatch {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        if self.state != expected_state {
            return Err(ExecutionTransitionError::StateMismatch {
                expected: expected_state,
                actual: self.state,
            });
        }
        if self.state.is_terminal() {
            return Err(ExecutionTransitionError::TerminalState { state: self.state });
        }
        if !transition_allowed(self.state, next_state) {
            return Err(ExecutionTransitionError::InvalidTransition {
                from: self.state,
                to: next_state,
            });
        }

        if matches!(next_state, ExecutionState::Starting) && self.started_at.is_none() {
            self.started_at = Some(observed_at);
        }
        if next_state.is_terminal() {
            self.ended_at = Some(observed_at);
        }
        self.state = next_state;
        self.reason = reason;
        self.last_progress_at = Some(observed_at);
        self.revision += 1;
        Ok(())
    }
}

pub const fn transition_allowed(from: ExecutionState, to: ExecutionState) -> bool {
    match from {
        ExecutionState::Queued => {
            matches!(to, ExecutionState::Starting | ExecutionState::Unreachable)
        }
        ExecutionState::Starting => {
            matches!(
                to,
                ExecutionState::Running | ExecutionState::Failed | ExecutionState::Unreachable
            )
        }
        ExecutionState::Running => matches!(
            to,
            ExecutionState::WaitingApproval
                | ExecutionState::WaitingUserInput
                | ExecutionState::Interrupting
                | ExecutionState::Completed
                | ExecutionState::Failed
                | ExecutionState::Unreachable
        ),
        ExecutionState::WaitingApproval | ExecutionState::WaitingUserInput => {
            matches!(
                to,
                ExecutionState::Running | ExecutionState::Failed | ExecutionState::Unreachable
            )
        }
        ExecutionState::Interrupting => {
            matches!(
                to,
                ExecutionState::Interrupted | ExecutionState::Failed | ExecutionState::Unreachable
            )
        }
        ExecutionState::Interrupted
        | ExecutionState::Completed
        | ExecutionState::Failed
        | ExecutionState::Unreachable => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_attendue_est_comparee_sur_etat_et_revision() {
        let mut execution = Execution::queued("e1", "s1", "agent1", 4);
        let error = execution
            .transition(
                ExecutionState::Running,
                0,
                ExecutionState::Completed,
                ExecutionReason::Completed,
                10,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ExecutionTransitionError::StateMismatch {
                expected: ExecutionState::Running,
                actual: ExecutionState::Queued
            }
        ));
        let error = execution
            .transition(
                ExecutionState::Queued,
                1,
                ExecutionState::Starting,
                ExecutionReason::Delivered,
                10,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ExecutionTransitionError::RevisionMismatch {
                expected: 1,
                actual: 0
            }
        ));
    }

    #[test]
    fn etat_terminal_reste_monotone_face_a_un_evenement_tardif() {
        let mut execution = Execution::queued("e1", "s1", "agent1", 4);
        execution
            .transition(
                ExecutionState::Queued,
                0,
                ExecutionState::Starting,
                ExecutionReason::Delivered,
                10,
            )
            .unwrap();
        execution
            .transition(
                ExecutionState::Starting,
                1,
                ExecutionState::Running,
                ExecutionReason::ProviderAccepted,
                11,
            )
            .unwrap();
        execution
            .transition(
                ExecutionState::Running,
                2,
                ExecutionState::Completed,
                ExecutionReason::Completed,
                12,
            )
            .unwrap();
        let error = execution
            .transition(
                ExecutionState::Completed,
                3,
                ExecutionState::Running,
                ExecutionReason::StaleEvent,
                13,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ExecutionTransitionError::TerminalState {
                state: ExecutionState::Completed
            }
        ));
        assert_eq!(execution.ended_at, Some(12));
    }

    #[test]
    fn attente_autorisation_repart_vers_running_sans_creer_un_nouveau_cycle() {
        let mut execution = Execution::queued("e1", "s1", "agent1", 4);
        execution
            .transition(
                ExecutionState::Queued,
                0,
                ExecutionState::Starting,
                ExecutionReason::Delivered,
                10,
            )
            .unwrap();
        execution
            .transition(
                ExecutionState::Starting,
                1,
                ExecutionState::Running,
                ExecutionReason::ProviderAccepted,
                11,
            )
            .unwrap();
        execution
            .transition(
                ExecutionState::Running,
                2,
                ExecutionState::WaitingApproval,
                ExecutionReason::PermissionRequired,
                12,
            )
            .unwrap();
        execution
            .transition(
                ExecutionState::WaitingApproval,
                3,
                ExecutionState::Running,
                ExecutionReason::ProviderAccepted,
                13,
            )
            .unwrap();
        assert_eq!(execution.state, ExecutionState::Running);
        assert_eq!(execution.revision, 4);
        assert_eq!(execution.started_at, Some(10));
    }

    #[test]
    fn submission_prepared_ne_deduit_ni_origine_ni_intention() {
        let submission = WorkSubmission::prepared("s1", "agent1", "digest", 10);
        assert_eq!(submission.state, SubmissionState::Prepared);
        assert_eq!(submission.origin, None);
        assert_eq!(submission.intent, None);
        assert_eq!(submission.revision, 0);
    }
}
