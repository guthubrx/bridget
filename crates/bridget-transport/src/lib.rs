//! bridget-transport — protocole de communication daemon/wrapper.
//!
//! Définit les messages JSON qui circulent sur la socket locale.

pub mod protocol;
pub mod transport;
pub mod tmux;

pub use protocol::{WrapperToDaemon, DaemonToWrapper};
pub use transport::{Transport, TransportError};
pub use tmux::TmuxTransport;
