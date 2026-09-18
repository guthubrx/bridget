//! bridget-core — logique pure du protocole bridget.
//!
//! Zéro I/O, zéro réseau. Tout est testable sans socket ni SQLite.

pub mod circuit_breaker;
pub mod dedup;
pub mod envelope;
pub mod execution;
pub mod host;
pub mod message;
pub mod router;
pub mod text_guards;

pub use circuit_breaker::CircuitBreaker;
pub use dedup::Deduplicator;
pub use envelope::{EnvelopeGuard, wrap_envelope};
pub use execution::{
    Execution, ExecutionReason, ExecutionState, ExecutionTransitionError, SubmissionState,
    WorkSubmission,
};
pub use host::{HOTE_NON_ATTESTE, host_is_attested, local_host};
pub use message::{AgentType, BridgetMessage, MessageIntent, MessageOrigin, ThreadNotice};
pub use router::{Router, RouterAction, RouterError};
pub use text_guards::{is_disallowed_control, is_format_character};
