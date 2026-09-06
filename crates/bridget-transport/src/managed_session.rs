//! Frontière commune des sessions d'équipiers gérés.
//!
//! Elle reste volontairement plus étroite que les protocoles fournisseurs :
//! le wrapper ne consomme que la livraison, les événements, le journal et le
//! cycle de vie. Les adaptateurs conservent leurs détails propres derrière
//! cette frontière.

use crate::journal::JournalLiveFeed;
use crate::protocol::{PresenceMode, ProviderObservation, ProviderOperation};
use crate::transport::{Transport, TransportError};
use bridget_core::BridgetMessage;
use std::path::Path;

/// Arrêt borné d'un enfant DÉTENU depuis son lancement. Le groupe n'est
/// utilisable que si l'appelant l'a créé avec process_group(0). Aucun SIGKILL,
/// aucun succès inventé : un survivant reste identifié dans l'erreur.
pub fn stop_owned_child(
    child: &mut std::process::Child,
    process_group: bool,
    budget: std::time::Duration,
) -> std::io::Result<()> {
    let pid = child.id() as i32;
    let target = if process_group { -pid } else { pid };
    let started = std::time::Instant::now();
    for (index, signal) in [libc::SIGTERM, libc::SIGINT].into_iter().enumerate() {
        let exited = child.try_wait()?.is_some();
        if exited
            && (!process_group
                || unsafe { libc::kill(target, 0) } < 0
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH))
        {
            return Ok(());
        }
        if unsafe { libc::kill(target, signal) } < 0
            && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
        {
            return Err(std::io::Error::last_os_error());
        }
        let deadline = started + budget.mul_f64((index + 1) as f64 / 2.0);
        while std::time::Instant::now() < deadline {
            let exited = child.try_wait()?.is_some();
            if exited
                && (!process_group
                    || unsafe { libc::kill(target, 0) } < 0
                        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH))
            {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("arrêt non confirmé du processus/groupe détenu {pid} après TERM et INT"),
    ))
}

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

/// Identité et corrélations réellement observées par une session fournisseur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedProviderIdentity {
    pub provider_kind: String,
    pub provider_session_id: Option<String>,
    pub provider_thread_id: Option<String>,
    pub active_turn_id: Option<String>,
    pub provider_item_id: Option<String>,
    pub capabilities_revision: Option<String>,
    /// Baseline attestée du binaire, absente tant qu'aucune sonde versionnée
    /// ne l'a réellement fournie à la session.
    pub provider_observation: Option<ProviderObservation>,
}

/// Attente explicite d une décision externe, distincte d un tour actif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedWaitState {
    Approval { request_id: String },
    UserInput { request_id: String },
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
    /// Activité attestée sans message Bridget (tour humain natif).
    ActivityObserved {
        in_progress: bool,
    },
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
        /// Pourcentage consommé si le fournisseur l'atteste ; sinon absent.
        used_percent: Option<u8>,
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
    /// Corrélations attestées sans faire dériver un état local par le wrapper.
    ProviderContextObserved {
        identity: ManagedProviderIdentity,
    },
    /// Attente explicite d autorisation ou de saisie, corrélée au fournisseur.
    Waiting {
        state: ManagedWaitState,
    },
    Update {
        detail: String,
    },
    /// Incident récupérable projeté par un adaptateur. Il ne transporte ni
    /// corps fournisseur ni argument d'outil: seuls un code stable et une
    /// référence pseudonymisée franchissent la frontière commune.
    Diagnostic {
        code: String,
        reference: String,
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

/// Issue fermée d'une demande de continuité. `Reconstructed` est un repli
/// déclaré: il ne signifie jamais que le fil fournisseur a été repris.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedContinuation {
    Native {
        provider_thread_id: String,
    },
    Forked {
        provider_thread_id: String,
        parent_thread_id: String,
    },
    Reconstructed {
        reason: String,
    },
    Refused {
        reason: String,
    },
}

fn continuation_fallback(
    observation: Option<ProviderObservation>,
    operation: ProviderOperation,
) -> ManagedContinuation {
    if observation.is_none_or(|observation| !observation.supports(operation)) {
        return ManagedContinuation::Reconstructed {
            reason: match operation {
                ProviderOperation::Resume => "resume_not_attested".to_string(),
                ProviderOperation::Fork => "fork_not_attested".to_string(),
                _ => "continuation_not_attested".to_string(),
            },
        };
    }
    ManagedContinuation::Refused {
        reason: "continuation_not_implemented".to_string(),
    }
}

/// Contrat commun d'une session enfant gérée.
///
/// `Transport` porte la livraison et l'état de vie ; ce trait ajoute
/// exclusivement les opérations de session dont le wrapper a besoin. Il ne
/// déclare ni modèle, ni quota, ni sémantique de protocole fournisseur.
pub trait ManagedSession: Transport {
    /// Choix explicite de session, sans tour caché. Le défaut refuse : aucun
    /// adaptateur ne simule une commande de modèle par un prompt.
    fn select_runtime(
        &mut self,
        _selection: crate::protocol::RuntimeSelection,
    ) -> crate::protocol::RuntimeSelectionOutcome {
        crate::protocol::RuntimeSelectionOutcome::Refused {
            reason: crate::protocol::RuntimeSelectionRefusal::Unsupported,
        }
    }
    /// Consigne individuelle conservée uniquement par le pilote en mémoire.
    ///
    /// Elle ne devient jamais un `BridgetMessage`: les implémentations qui la
    /// prennent en charge l'ajoutent à la prochaine vraie demande fournisseur,
    /// sans l'écrire dans le journal Bridget ni l'exposer comme conversation.
    fn set_private_profile_instructions(
        &mut self,
        _instructions: &str,
    ) -> Result<(), TransportError> {
        Err(TransportError::DeliveryFailed(
            "contexte privé de profil non pris en charge".to_string(),
        ))
    }

    /// Reprise neutre: sans contrat attesté, le seul repli est explicitement
    /// reconstruit et l'implémentation ne peut pas appeler le fournisseur.
    fn resume_thread(&self, _provider_thread_id: &str) -> ManagedContinuation {
        continuation_fallback(
            self.provider_identity()
                .and_then(|identity| identity.provider_observation),
            ProviderOperation::Resume,
        )
    }

    /// Bifurcation neutre, soumise à la même fermeture par capacité.
    fn fork_thread(&self, _provider_thread_id: &str) -> ManagedContinuation {
        continuation_fallback(
            self.provider_identity()
                .and_then(|identity| identity.provider_observation),
            ProviderOperation::Fork,
        )
    }

    fn descriptor(&self) -> ManagedSessionDescriptor;
    fn process_id(&self) -> u32;
    fn activate_journal(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()>;
    fn drain_events(&self) -> Vec<ManagedEvent>;
    /// Une implémentation qui ne peut pas attester son identité la laisse
    /// absente : le consommateur doit refuser les opérations qui la requièrent.
    fn provider_identity(&self) -> Option<ManagedProviderIdentity> {
        None
    }

    /// La capacité se lit uniquement depuis la baseline associée à cette
    /// session. Son absence ferme le contrôle : le nom du fournisseur ne vaut
    /// pas capacité et un registre historique ne peut pas injecter un prompt.
    fn supports_operation(&self, operation: ProviderOperation) -> bool {
        self.provider_identity()
            .and_then(|identity| identity.provider_observation)
            .is_some_and(|observation| observation.supports(operation))
    }

    /// Compatibilité locale avec les appelants historiques de steering.
    fn supports_steering(&self) -> bool {
        self.supports_operation(ProviderOperation::Steer)
    }
    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool;
    fn stop(&self);
    fn is_busy(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enfant_reel_ignore_term_puis_int_ferme_le_groupe_sous_borne() {
        use std::io::{BufRead, BufReader};
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};
        let mut child = Command::new("/usr/bin/python3").args(["-c",
            "import signal,sys; signal.signal(signal.SIGTERM, signal.SIG_IGN); signal.signal(signal.SIGINT, lambda *_: sys.exit(0)); print('READY',flush=True); signal.pause()"])
            .stdout(Stdio::piped()).stderr(Stdio::null()).process_group(0).spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let _ = tx.send(BufReader::new(stdout).lines().next());
        });
        let ready = rx.recv_timeout(Duration::from_secs(3));
        let started = Instant::now();
        let result = stop_owned_child(&mut child, true, Duration::from_secs(2));
        reader.join().unwrap();
        assert_eq!(ready.unwrap().unwrap().unwrap(), "READY");
        result.unwrap();
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(child.try_wait().unwrap().is_some());
        // Mutation : TERM seul laisserait ce véritable enfant vivant ;
        // wait() sans borne immobiliserait ce test au lieu de confirmer INT.
        assert!(unsafe { libc::kill(-(child.id() as i32), 0) } < 0);
    }

    #[test]
    fn contexte_fournisseur_et_attente_restent_deux_faits_distincts() {
        let identity = ManagedProviderIdentity {
            provider_kind: "codex".to_string(),
            provider_session_id: Some("session-1".to_string()),
            provider_thread_id: Some("thread-1".to_string()),
            active_turn_id: Some("turn-1".to_string()),
            provider_item_id: Some("item-1".to_string()),
            capabilities_revision: Some("contrat-1".to_string()),
            provider_observation: None,
        };
        let event = ManagedEvent::internal(
            ManagedEventSource::CodexAppServer,
            Vec::new(),
            ManagedEventKind::ProviderContextObserved { identity },
        );
        assert!(matches!(
            event.kind,
            ManagedEventKind::ProviderContextObserved { identity }
                if identity.active_turn_id.as_deref() == Some("turn-1")
        ));
        let wait = ManagedWaitState::Approval {
            request_id: "approval-1".to_string(),
        };
        assert!(
            matches!(wait, ManagedWaitState::Approval { request_id } if request_id == "approval-1")
        );
    }
    #[test]
    fn reprise_sans_capacite_est_reconstruite_et_capacite_non_implantee_est_refusee() {
        assert!(matches!(
            continuation_fallback(None, ProviderOperation::Resume),
            ManagedContinuation::Reconstructed { reason } if reason == "resume_not_attested"
        ));
        let observed = ProviderObservation {
            binary_path: "/fixture/codex".to_string(),
            binary_version: "fixture".to_string(),
            binary_digest: "a".repeat(64),
            contract_version: "fixture-v1".to_string(),
            operations: vec![ProviderOperation::Resume],
        };
        assert!(matches!(
            continuation_fallback(Some(observed), ProviderOperation::Resume),
            ManagedContinuation::Refused { reason } if reason == "continuation_not_implemented"
        ));
    }
}
