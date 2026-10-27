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
    /// Arrêt incomplet après échec de négociation : propriété conservée.
    StartUnconfirmed {reason:String,owner:Box<RetainedStart>},
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::AgentDead => write!(f, "agent CLI mort"),
            TransportError::Io(msg) => write!(f, "erreur I/O: {}", msg),
            TransportError::DeliveryFailed(msg) => write!(f, "livraison échouée: {}", msg),
            TransportError::StartUnconfirmed {reason,owner} => write!(f,
                "arrêt du fournisseur non confirmé (PID {}) : {}",owner.process_id(),reason),
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

/// Ressources d'un lancement dont l'arrêt n'est pas encore prouvé.
/// Le wrapper doit les conserver et fermer les admissions, jamais relancer.
#[derive(Debug)]
pub struct RetainedStart {
    child: std::process::Child,
    reader: Option<std::thread::JoinHandle<()>>,
}
impl RetainedStart {
    pub(crate) fn new(child: std::process::Child, reader: std::thread::JoinHandle<()>) -> Self {
        Self {child,reader:Some(reader)}
    }
    pub fn process_id(&self) -> u32 {self.child.id()}
    pub fn stop(&mut self,budget:std::time::Duration)->std::io::Result<()> {
        let deadline=std::time::Instant::now()+budget;
        crate::managed_session::stop_owned_child(&mut self.child,true,budget)?;
        while self.reader.as_ref().is_some_and(|v|!v.is_finished()) {
            if std::time::Instant::now()>=deadline {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut,"lecture fournisseur non arrêtée"));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if let Some(reader)=self.reader.take() {
            reader.join().map_err(|_|std::io::Error::other("lecture fournisseur en échec"))?;
        }
        Ok(())
    }
}
