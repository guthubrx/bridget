//! bridget-transport — protocole de communication daemon/wrapper.
//!
//! Définit les messages JSON qui circulent sur la socket locale.

pub mod acp;
pub mod journal;
pub mod protocol;
pub mod tmux;
pub mod transport;

pub use acp::{AcpEvent, AcpOptions, AcpTransport, TurnState};
pub use protocol::{
    AttachRefusal, AttachWindow, ConnectionRole, DaemonToWrapper, MAX_ATTACH_FRAGMENT_BYTES,
    MAX_ATTACH_SERIALIZED_FRAME_BYTES, WrapperToDaemon,
};
pub use tmux::TmuxTransport;
pub use transport::{Transport, TransportError};
