//! Contrat de transport — ce que tout wrapper doit implémenter.

use bridget_core::BridgetMessage;

/// Erreur de transport.
#[derive(Debug)]
pub enum TransportError {
    /// L'agent CLI n'est plus en vie.
    AgentDead,
    /// Erreur d'I/O (socket, pipe, tmux).
    Io(String),
    /// Le message n'a pas pu être injecté.
    DeliveryFailed(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::AgentDead => write!(f, "agent CLI mort"),
            TransportError::Io(msg) => write!(f, "erreur I/O: {}", msg),
            TransportError::DeliveryFailed(msg) => write!(f, "livraison échouée: {}", msg),
        }
    }
}

impl std::error::Error for TransportError {}

/// Contrat qu'un transport doit implémenter.
/// Le daemon appelle ces méthodes sur chaque wrapper connecté.
pub trait Transport: Send {
    /// Livrer un message à l'agent (l'injecter dans son pane/pipe/socket).
    fn deliver(&mut self, msg: &BridgetMessage) -> Result<(), TransportError>;

    /// Vérifier que l'agent est toujours vivant.
    fn is_alive(&self) -> bool;

    /// Identifiant de connexion (PID, socket path, pane ID, etc.).
    fn connection_id(&self) -> &str;
}
