//! bridget-core — logique pure du protocole bridget.
//!
//! Zéro I/O, zéro réseau. Tout est testable sans socket ni SQLite.

pub mod message;
pub mod envelope;
pub mod router;
pub mod circuit_breaker;
pub mod dedup;

pub use message::{BridgetMessage, AgentType};
pub use envelope::{wrap_envelope, EnvelopeGuard};
pub use router::{Router, RouterAction, RouterError};
pub use circuit_breaker::CircuitBreaker;
pub use dedup::Deduplicator;
