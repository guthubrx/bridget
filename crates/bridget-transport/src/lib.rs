//! bridget-transport — protocole de communication daemon/wrapper.
//!
//! Définit les messages JSON qui circulent sur la socket locale.

pub mod acp;
pub mod act_kind;
pub mod claude_provider_session;
pub mod claude_stream_json;
pub mod codex_app_server;
mod codex_socket;
pub mod fsutil;
pub mod greffe_authorization;
pub mod greffe_policy_refresh;
pub mod journal;
pub mod jsonl;
pub mod managed_session;
pub mod protocol;
pub mod pty;
pub mod refusals;
pub mod tmux;
pub mod transport;

pub use act_kind::{JournalUpdateKind, parse_update_kind, validate_journal_write};

pub use acp::{AcpEvent, AcpEventQueue, AcpOptions, AcpTransport, TurnState};
pub use claude_provider_session::{
    CLAUDE_RESUME_FAILED_PREFIX, ProviderSessionStore, ResumeFailure, ResumeFailureKind,
};
pub use claude_stream_json::{ClaudeStreamJsonOptions, ClaudeStreamJsonTransport};
pub use codex_app_server::{CodexAppServerOptions, CodexAppServerTransport};
pub use managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource,
    ManagedProviderIdentity, ManagedSession, ManagedSessionDescriptor, ManagedTerminal,
    ManagedWaitState,
};
pub use protocol::{
    AdapterCapabilities, AttachRefusal, AttachWindow, ChannelReport, ConnectionRole,
    DaemonToWrapper, LedgerDeliveryStatus, LedgerMessage, MAX_ATTACH_FRAGMENT_BYTES,
    MAX_ATTACH_SERIALIZED_FRAME_BYTES, ModelCapabilities, ResolvedAgentDefinition,
    ResolvedMcpDefinition, SpawnRefusal, StopOutcome, WrapperToDaemon,
};
pub use pty::PtyTransport;
pub use tmux::TmuxTransport;
pub use transport::{Transport, TransportError};
