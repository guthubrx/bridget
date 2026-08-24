//! Frontière commune des sessions d'équipiers gérés.
//!
//! Elle reste volontairement plus étroite que les protocoles fournisseurs :
//! le wrapper ne consomme que la livraison, les événements, le journal et le
//! cycle de vie. Les adaptateurs conservent leurs détails propres derrière
//! cette frontière.

use crate::journal::JournalLiveFeed;
use crate::transport::Transport;
use bridget_core::BridgetMessage;
use std::path::Path;

/// Source attestée d'un événement de session.
///
/// La v1 ne contient que la source déjà réellement consommée par Bridget.
/// Les futurs pilotes n'ajoutent une variante qu'au moment où ils émettent un
/// événement public : ne pas anticiper leur vocabulaire ici.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedEventSource {
    Acp,
}

/// Projection neutre des événements que le wrapper consomme aujourd'hui.
///
/// Chaque événement conserve les octets émis par son pilote à cette frontière
/// de session. Le wrapper ne traite donc jamais une variante ACP directement
/// et un adaptateur futur peut préserver ses bytes sans les réduire à une
/// chaîne de diagnostic.
#[derive(Debug, Clone)]
pub struct ManagedEvent {
    pub source: ManagedEventSource,
    pub raw: Vec<u8>,
    pub kind: ManagedEventKind,
}

#[derive(Debug, Clone)]
pub enum ManagedEventKind {
    TurnStarted {
        message_id: String,
    },
    PromptDispatched {
        message_id: String,
    },
    TurnFinished {
        message: BridgetMessage,
        response: String,
        stop_reason: String,
    },
    DeliveryRejected {
        message_id: String,
        reason: String,
    },
    Update {
        detail: String,
    },
    Error {
        detail: String,
    },
    JournalFailed {
        detail: String,
    },
}

/// Contrat commun d'une session enfant gérée.
///
/// `Transport` porte la livraison et l'état de vie ; ce trait ajoute
/// exclusivement les opérations de session dont le wrapper a besoin. Il ne
/// déclare ni modèle, ni quota, ni sémantique de protocole fournisseur.
pub trait ManagedSession: Transport {
    fn process_id(&self) -> u32;
    fn activate_journal(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()>;
    fn drain_events(&self) -> Vec<ManagedEvent>;
    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool;
    fn stop(&self);
    fn is_busy(&self) -> bool;
}
