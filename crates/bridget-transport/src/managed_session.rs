//! Frontière commune des sessions d'équipiers gérés.
//!
//! Elle reste volontairement plus étroite que les protocoles fournisseurs :
//! le wrapper ne consomme que la livraison, les événements, le journal et le
//! cycle de vie. Les adaptateurs conservent leurs détails propres derrière
//! cette frontière.

use crate::journal::JournalLiveFeed;
use crate::protocol::PresenceMode;
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
    ClaudeStreamJson,
    CodexAppServer,
}

/// Provenance des octets portés par un événement.
///
/// Une ligne effectivement lue sur stdout ne doit jamais être confondue avec
/// un fait interne produit par le wrapper (annulation locale, échec du
/// journal, etc.). Les adaptateurs gardent ainsi la frontière d'observation
/// honnête, y compris pour une notification fournisseur inconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedEventOrigin {
    SourceLine,
    Internal,
}

/// Identité de présence attestée par le pilote de session.
///
/// Le wrapper ne déduit jamais ce descripteur du protocole qu'il implémente :
/// il le transporte aussi bien au premier Register qu'à chaque reconnexion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedSessionDescriptor {
    pub transport: String,
    pub mode: PresenceMode,
    pub location: Option<String>,
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
    pub origin: ManagedEventOrigin,
    pub raw: Vec<u8>,
    pub kind: ManagedEventKind,
}

impl ManagedEvent {
    pub fn source_line(source: ManagedEventSource, raw: Vec<u8>, kind: ManagedEventKind) -> Self {
        Self {
            source,
            origin: ManagedEventOrigin::SourceLine,
            raw,
            kind,
        }
    }

    pub fn internal(source: ManagedEventSource, raw: Vec<u8>, kind: ManagedEventKind) -> Self {
        Self {
            source,
            origin: ManagedEventOrigin::Internal,
            raw,
            kind,
        }
    }
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
        terminal: ManagedTerminal,
    },
    DeliveryRejected {
        message_id: String,
        reason: String,
    },
    /// Modèle et effort effectivement retournés par le pilote. L'effort
    /// absent reste une absence attestée, jamais un réglage par défaut local.
    RuntimeObserved {
        model: String,
        effort: Option<String>,
    },
    /// Limite fournisseur effectivement annoncée par le pilote. Le wrapper la
    /// transmet telle quelle au daemon ; il n'en tire aucune décision locale.
    RateLimitObserved {
        window: String,
        status: String,
        resets_at: Option<i64>,
    },
    /// Modèle réellement annoncé par le flux natif. L'absence de cet événement
    /// n'autorise aucun verdict : un flux muet reste sans écart.
    ModelObserved {
        model: String,
    },
    /// Consommation d'un tour effectivement lue du flux natif. Les compteurs
    /// absents ou incomplets ne deviennent jamais un zéro inventé : le pilote
    /// n'émet cet événement que lorsque le schéma attesté est entier.
    UsageObserved {
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
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

/// Issue terminale normalisée par l'adaptateur au bord fournisseur.
///
/// La couche commune ne connaît ni `stopReason` ACP ni une chaîne native
/// particulière. Le détail d'un refus reste attesté, mais sa sémantique est
/// fermée avant d'atteindre le wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedTerminal {
    Completed,
    Cancelled,
    Failed { detail: String },
}

/// Contrat commun d'une session enfant gérée.
///
/// `Transport` porte la livraison et l'état de vie ; ce trait ajoute
/// exclusivement les opérations de session dont le wrapper a besoin. Il ne
/// déclare ni modèle, ni quota, ni sémantique de protocole fournisseur.
pub trait ManagedSession: Transport {
    fn descriptor(&self) -> ManagedSessionDescriptor;
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
