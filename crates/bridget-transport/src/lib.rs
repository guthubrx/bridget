//! bridget-transport — protocole de communication daemon/wrapper.
//!
//! Définit les messages JSON qui circulent sur la socket locale.

pub mod acp;
pub mod claude_stream_json;
pub mod codex_app_server;
pub mod fsutil;
pub mod journal;
pub mod managed_session;
pub mod protocol;
pub mod tmux;
pub mod transport;

pub use acp::{AcpEvent, AcpEventQueue, AcpOptions, AcpTransport, TurnState};
pub use claude_stream_json::{ClaudeStreamJsonOptions, ClaudeStreamJsonTransport};
pub use codex_app_server::{CodexAppServerOptions, CodexAppServerTransport};
pub use managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource, ManagedSession,
    ManagedSessionDescriptor, ManagedTerminal,
};
pub use protocol::{
    AdapterCapabilities, AttachRefusal, AttachWindow, ChannelReport, ConnectionRole,
    DaemonToWrapper, LedgerDeliveryStatus, LedgerMessage, MAX_ATTACH_FRAGMENT_BYTES,
    MAX_ATTACH_SERIALIZED_FRAME_BYTES, ModelCapabilities, ResolvedAgentDefinition,
    ResolvedMcpDefinition, SpawnRefusal, StopOutcome, WrapperToDaemon,
};
pub use tmux::TmuxTransport;
pub use transport::{Transport, TransportError};
