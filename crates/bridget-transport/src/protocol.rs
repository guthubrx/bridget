//! Définitions des messages JSON du protocole daemon/wrapper.
//!
//! Deux directions :
//! - WrapperToDaemon : ce que le wrapper envoie au daemon
//! - DaemonToWrapper : ce que le daemon envoie au wrapper

use bridget_core::BridgetMessage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Rôle négocié au début d'une connexion persistante avec le daemon.
///
/// L'absence de négociation reste implicitement un wrapper pour préserver les
/// agents 007 déjà déployés. Un client attach doit en revanche s'annoncer
/// explicitement avant toute souscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionRole {
    Wrapper,
    Attach,
    Client,
    Service,
}

/// Mode réel de présence d'un agent.
///
/// Cette information décrit le chemin d'attelage (et non le transport réseau)
/// qui a effectivement enregistré l'agent. Une absence conserve la
/// compatibilité des enregistrements antérieurs au champ et ne doit jamais
/// être interprétée par déduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceMode {
    Acp,
    Tmux,
    Cli,
}

impl PresenceMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Acp => "acp",
            Self::Tmux => "tmux",
            Self::Cli => "cli",
        }
    }
}

/// Version actuellement publiée du contrat idempotent local.
pub const CLIENT_CONTRACT_VERSION: u16 = 1;
/// Version du contrat de service du guichet Maicie.
pub const SERVICE_CONTRACT_VERSION: u16 = 1;
/// Version de l'extension de faits attestés pour la coordination active.
///
/// Elle reste une capacité négociée du contrat de service v1 : les clients 015
/// qui ne la demandent pas ne reçoivent aucune trame 016.
pub const COORDINATION_EVENTS_VERSION: u16 = 1;

/// Capacité optionnelle du client idempotent. L'énumération fermée évite une
/// dégradation silencieuse lorsqu'un client demande une capacité inconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    SendIdempotent,
    Lookup,
}

/// Capacité explicitement négociée par un service local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCapability {
    MaicieGuichet,
    CoordinationEventsV1,
}

/// Refus structurés de la frontière réservée aux services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServiceRefusal {
    RoleHandshakeRequired,
    ServiceRoleRequired,
    NegotiationRequired,
    AlreadyNegotiated,
    UnsupportedVersion { supported_versions: Vec<u16> },
    InvalidIssuerScope,
    ReservedServiceRequired,
    InvalidEnvelope,
    CapabilityRequired,
    MessageOutsideServiceRole,
    TransitionInvalid,
    CanonicalBytesMismatch,
    IdempotencyExpired,
    ClaimStale,
    DeclaredSenderMismatch,
    ReservedTargetRequired,
    InvalidIssuedAt,
    FrameTooLarge,
}

/// Opérations fermées que Bridget peut déposer dans le guichet Maicie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRequestOperation {
    DeliveryReport,
    MissionStatus,
    DeadlineQuestion,
}

/// Charge canonique d'un dépôt de guichet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceRequestPayload {
    DeliveryReport {
        objective_id: String,
        delegation_id: String,
        delivery_hash: String,
        in_reply_to: String,
    },
    Delegation {
        delegation_id: String,
    },
}

/// Issue fermée qu'un service Maicie atteste au guichet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetOutcome {
    Accepted,
    RequestAlreadyTerminal,
    RecipientUnavailable,
    Refused,
}

/// Motif fermé d'un refus déterministe rendu par Maicie après la relève.
///
/// Il décrit une demande bien formée mais impossible à appliquer au registre
/// local. Une corruption du store ou une erreur de transport ne passe jamais
/// par cette voie : ces situations restent des erreurs techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetRefusalReason {
    DelegationMissing,
    RelationInvalid,
    EnvelopeMismatch,
}

/// Fait terminal attesté uniquement par Bridget pour une demande du guichet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetLifecycleState {
    Answered,
    Cancelled,
    TimedOut,
}

/// Fait de coordination transport attesté exclusivement par Bridget.
///
/// Cette énumération est volontairement fermée : Maicie ne déduit jamais une
/// relance d'un texte ou d'une échéance locale, et une valeur future exige une
/// capacité/version explicitement négociée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationEventKind {
    ReminderSent,
}

/// Charge canonique d'une réponse Maicie. L'ordre de déclaration est l'ordre
/// filaire normatif du contrat 015.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GuichetReplyPayload {
    DeliveryReport {
        objective_id: String,
        delegation_id: String,
        delivery_hash: String,
    },
    MissionStatus {
        delegation_id: String,
        objective_id: String,
        coordination_state: GuichetCoordinationState,
        local_delivery: GuichetLocalDelivery,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_observation: Option<GuichetTransportObservation>,
        freshness: GuichetFreshness,
    },
    DeadlineQuestion {
        delegation_id: String,
        duration_class: GuichetDurationClass,
        deadline_at: i64,
    },
    /// Refus fermé sans projection inventée quand les faits locaux demandés
    /// par la requête n'existent pas ou ne sont pas corrélés.
    Refused {
        operation: ServiceRequestOperation,
        reason: GuichetRefusalReason,
    },
}

/// Projection fermée d'une coordination Maicie : Bridget la transporte sans
/// jamais en déduire ni la compléter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetCoordinationState {
    Open,
    EnCoordination,
    AEvaluer,
    Synthetise,
    Clos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetLocalDeliveryState {
    Pending,
    Accepted,
    OutcomeUnknown,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuichetLocalDelivery {
    pub state: GuichetLocalDeliveryState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetTransportState {
    Connected,
    Unavailable,
    Gap,
    Ended,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuichetTransportObservation {
    pub state: GuichetTransportState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_state: Option<String>,
    pub observed_at: i64,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetFreshness {
    Fresh,
    Gap,
    Ended,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetDurationClass {
    Courte,
    Normale,
    Longue,
}

/// Refus structurés de la frontière publique client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientRefusal {
    RoleHandshakeRequired,
    ClientRoleRequired,
    NegotiationRequired,
    AlreadyNegotiated,
    UnsupportedVersion { supported_versions: Vec<u16> },
    InvalidIssuerScope,
    ActiveScopeLimit,
    CapabilityNotNegotiated,
    MessageOutsideClientRole,
}

/// Table fermée des refus d'un ordre de lancement géré par le daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpawnRefusal {
    /// Le daemon joint son instantané du registre : le CLI ne relit jamais le
    /// fichier utilisateur et ne peut donc pas présenter une liste divergente.
    UnknownType {
        #[serde(default)]
        requested_type: String,
        #[serde(default)]
        known_types: Vec<String>,
        #[serde(default)]
        registry: String,
    },
    CommandMissing {
        command: String,
        registry: String,
    },
    /// La définition figée ne déclare pas la capacité indispensable au
    /// lancement demandé. Ce refus intervient avant toute réservation de
    /// lancement neuve et avant tout processus.
    UnsupportedCapability {
        agent_type: String,
        model: String,
        capability: String,
    },
    BillingGuard {
        variable: String,
    },
    NameActive,
    EnvUnfit {
        detail: String,
    },
    CwdGone,
    NegotiationFailed {
        detail: String,
    },
    SpawnTimeout,
    QuotaExceeded {
        limit: usize,
    },
    DaemonRecovering,
    IdempotencyExpired,
}

/// Résultat synchrone et fermé d'un ordre d'arrêt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StopOutcome {
    Stopped,
    StoppedForced { survivors_killed: usize },
    NotManaged,
    NotFound,
    Timeout { state: String },
}

/// Issue calculée d'une opération client idempotente. `OutcomeUnknown` est
/// informatif : il impose un `Lookup` ou le rejeu strict de la même enveloppe,
/// jamais une nouvelle émission avec une nouvelle clé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdempotencyIssue {
    Accepted {
        expires_at: i64,
    },
    Rejected {
        category: String,
        reason: String,
        expires_at: i64,
    },
    OutcomeUnknown {
        expires_at: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery_id: Option<String>,
    },
    EnvelopeMismatch,
    IdempotencyExpired,
    InvalidIssuedAt,
}

/// Fenêtre d'historique demandée par une vue attach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum AttachWindow {
    Today,
    Seq(u64),
    Date(String),
}

/// Refus explicitement typés du plan de contrôle attach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachRefusal {
    AgentUnknown,
    AgentStopped,
    /// Le pilote n'a pas attesté de journal append-only disponible. La
    /// compatibilité attach dépend de ce fait, jamais du protocole du pilote.
    JournalUnavailable,
    /// Variante historique conservée pour les pairs plus anciens. Les
    /// nouveaux refus attach doivent employer `JournalUnavailable`.
    AgentNotAcp,
    WrapperUnavailable,
    CommandQueueSaturated,
    InvalidDate,
    FutureDate,
    DateOutsideRetention,
    ReplyNotAllowed,
    MessageOutsideAttachRole,
}

/// Taille maximale d'un fragment d'événement sur le fil attach.
pub const MAX_ATTACH_SERIALIZED_FRAME_BYTES: usize = 256 * 1024;
/// Borne de charge utile avant encodage base64. Le worker vérifie ensuite la
/// taille JSON réelle afin que la frame filaire reste sous la borne ci-dessus.
pub const MAX_ATTACH_FRAGMENT_BYTES: usize = 190 * 1024;

mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let value = (u32::from(chunk[0]) << 16)
                | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
                | u32::from(*chunk.get(2).unwrap_or(&0));
            encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
            encoded.push(if chunk.len() > 1 {
                TABLE[((value >> 6) & 0x3f) as usize] as char
            } else {
                '='
            });
            encoded.push(if chunk.len() > 2 {
                TABLE[(value & 0x3f) as usize] as char
            } else {
                '='
            });
        }
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        if encoded.len() % 4 != 0 {
            return Err(serde::de::Error::custom("base64 incomplet"));
        }
        let mut bytes = Vec::with_capacity(encoded.len() / 4 * 3);
        for block in encoded.as_bytes().chunks(4) {
            let value = block
                .iter()
                .enumerate()
                .try_fold(0_u32, |value, (index, byte)| {
                    let bits = match byte {
                        b'A'..=b'Z' => byte - b'A',
                        b'a'..=b'z' => byte - b'a' + 26,
                        b'0'..=b'9' => byte - b'0' + 52,
                        b'+' => 62,
                        b'/' => 63,
                        b'=' if index >= 2 => 0,
                        _ => return Err(serde::de::Error::custom("base64 invalide")),
                    };
                    Ok((value << 6) | u32::from(bits))
                })?;
            bytes.push((value >> 16) as u8);
            if block[2] != b'=' {
                bytes.push((value >> 8) as u8);
            }
            if block[3] != b'=' {
                bytes.push(value as u8);
            }
        }
        Ok(bytes)
    }
}

/// Messages envoyés par le wrapper vers le daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WrapperToDaemon {
    /// Négocie un rôle avant l'usage d'une connexion persistante.
    RoleHandshake { role: ConnectionRole },
    /// Négocie le contrat client public, uniquement après RoleAccepted(Client).
    ClientHello {
        contract_version: u16,
        issuer_scope: String,
        capabilities: Vec<ClientCapability>,
    },
    /// Négocie le contrat du service Maicie, uniquement après RoleAccepted(Service).
    ServiceHello {
        version: u16,
        service: String,
        issuer_scope: String,
        capabilities: Vec<ServiceCapability>,
    },
    /// Dépôt durable produit par un wrapper enregistré vers le guichet Maicie.
    #[serde(rename = "service_request")]
    ServiceRequest {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        issued_at: i64,
        from: String,
        to: String,
        operation: ServiceRequestOperation,
        payload: ServiceRequestPayload,
    },
    /// Relève FIFO bornée d'une demande du guichet. T1504 en assure la persistance.
    #[serde(rename = "guichet_claim_next")]
    GuichetClaimNext {
        #[serde(rename = "v")]
        version: u16,
    },
    /// Rejeu strict d'un claim existant, sous son token de lease courant.
    #[serde(rename = "guichet_claim")]
    GuichetClaim {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        claim_token: String,
    },
    /// Consultation bornée d'une demande du guichet.
    #[serde(rename = "guichet_lookup")]
    GuichetLookup {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
    },
    /// Réponse de service corrélée à un claim. T1504 valide et persiste son canon.
    #[serde(rename = "guichet_reply")]
    GuichetReply {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        claim_generation: u64,
        claim_token: String,
        response_message_id: String,
        in_reply_to: String,
        outcome: GuichetOutcome,
        payload: GuichetReplyPayload,
    },
    /// Envoi à clé client. T1205 raccorde cette variante au socle durable.
    SendIdempotent {
        message: BridgetMessage,
        message_id: String,
        issued_at: i64,
    },
    /// Lecture d'une issue à l'intérieur de la portée négociée.
    Lookup {
        operation_kind: String,
        idempotency_key: String,
    },
    /// Accusé durable de remise envoyé exclusivement par un wrapper.
    DeliverAcked {
        delivery_id: String,
        delivery_generation: u64,
    },
    /// Le wrapper a persisté Seen sans pouvoir confirmer l'injection.
    DeliveryIndeterminate {
        delivery_id: String,
        delivery_generation: u64,
    },
    /// Ordre idempotent de lancement d'un équipier supervisé.
    SpawnOrder {
        agent_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        cwd: String,
        persistent: bool,
        command_id: String,
        issued_at: i64,
        deadline_at: i64,
    },
    /// Ordre corrélé d'arrêt d'un équipier supervisé.
    StopOrder { name: String, command_id: String },
    /// Ouvrir un abonnement à la vue d'un équipier.
    Subscribe { agent: String, window: AttachWindow },
    /// Fermer un abonnement sans fermer la connexion attach.
    Unsubscribe { subscription_id: String },
    /// Confirmation du wrapper : le daemon peut alors l'annoncer à la vue.
    Subscribed { subscription_id: String },
    /// Fragment binaire d'une ligne JSONL versionnée.
    JournalFragment {
        subscription_id: String,
        seq: u64,
        offset: u64,
        #[serde(rename = "final")]
        final_fragment: bool,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
    },
    /// Fragment live indépendant des vues ; le daemon le multiplexe vers les
    /// abonnements dont le rejeu est terminé.
    LiveJournalFragment {
        seq: u64,
        offset: u64,
        #[serde(rename = "final")]
        final_fragment: bool,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
    },
    /// Marque la frontière entre le rejeu et le suivi continu.
    SnapshotCaughtUp {
        subscription_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        through_seq: Option<u64>,
    },
    /// Signale une plage volontairement non rendue par une vue lente.
    Gap {
        subscription_id: String,
        from_seq: u64,
        to_seq: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Ligne de journal illisible sans séquence exploitable.
    JournalReadError {
        subscription_id: String,
        line: u64,
        offset: u64,
        reason: String,
    },
    /// Termine un abonnement, sans impliquer la fermeture de connexion.
    End {
        subscription_id: String,
        reason: String,
    },
    /// Refus typé produit par le wrapper lors de la préparation d'un relais.
    AttachRejected {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subscription_id: Option<String>,
        reason: AttachRefusal,
    },
    /// S'enregistrer auprès du daemon.
    Register {
        agent_type: String,
        name: Option<String>,
        #[serde(default)]
        host: Option<String>,
        #[serde(default)]
        transport: Option<String>,
        /// Mode d'attelage réellement emprunté. Son absence représente un
        /// enregistrement historique, jamais un mode à deviner.
        #[serde(default)]
        mode: Option<PresenceMode>,
        /// Localisation interactive connue, au format `session:window.pane`
        /// pour tmux. Elle reste absente lorsqu'elle n'est pas attestée.
        #[serde(default)]
        location: Option<String>,
        #[serde(default)]
        os: Option<String>,
        #[serde(default)]
        instance_id: Option<String>,
        /// Regroupement de travail de l'agent : nom du dépôt d'où il a été
        /// lancé, ou domaine choisi explicitement s'il en existe un.
        #[serde(default)]
        domain: Option<String>,
        /// État ACP re-déclaré après chaque reconnexion du wrapper.
        #[serde(default)]
        turn_in_progress: bool,
        /// `Some(false)` est envoyé par les wrappers qui connaissent
        /// `JournalReady`. L'absence est réservée à la transition des
        /// wrappers historiques, dont le journal ACP est déjà une garantie du
        /// chemin de lancement ; elle ne doit jamais faire rétrograder une
        /// présence attachable lors d'un redémarrage de daemon.
        #[serde(default)]
        journal_available: Option<bool>,
    },
    /// Le pilote a ouvert son journal append-only pour cette connexion. Ce
    /// signal distinct du Register évite de déduire attach du mode ACP.
    JournalReady,
    /// Se désenregistrer.
    Unregister,
    /// Renommer un agent déjà enregistré.
    Rename { current_name: String, name: String },
    /// Envoyer un message à un autre agent.
    Send(BridgetMessage),
    /// Refus terminal asynchrone d'une livraison déjà acquittée par le daemon.
    DeliveryRejected { id: String, reason: String },
    /// Transition dédiée du tour ACP, distincte de l'observation `Runtime`.
    TurnState { in_progress: bool },
    /// Annuler une demande suivie appartenant à l'agent courant.
    CancelRequest {
        id: String,
        sender: String,
        reason: Option<String>,
    },
    /// Lister les demandes suivies de l'agent courant.
    ListRequests { sender: String, limit: u16 },
    /// Projeter le ledger détenu par le daemon, pour un client fédéré qui ne
    /// possède pas sa base SQLite locale.
    LedgerProjection { scope: LedgerScope, limit: u16 },
    /// Signal de vie (périodique).
    Heartbeat,
    /// Demander la liste des agents connectés.
    ListAgents,
    /// Rapporter le modèle et le niveau d'effort courants d'un agent.
    ///
    /// `agent` désigne l'agent observé, et non la connexion émettrice : le hook
    /// Claude et `bridget runtime` passent par le client CLI, dont la connexion
    /// est éphémère et distincte de celle de l'agent. Même motif que `Rename`.
    ///
    /// Une observation est atomique : le couple `(model, effort)` remplace en
    /// bloc l'état connu. Un `effort` absent signifie « observé absent » et
    /// efface la valeur précédente — un modèle sans réglage d'effort ne doit
    /// pas hériter de l'effort du modèle précédent.
    Runtime {
        agent: String,
        model: String,
        #[serde(default)]
        effort: Option<String>,
        source: RuntimeSource,
    },
    /// Rapporter un fait de limite attesté par le pilote d'un agent.
    ///
    /// L'absence de ce message ne permet aucune déduction : une limite inconnue
    /// reste inconnue. Cette observation n'autorise ni refus ni bascule.
    RateLimit {
        agent: String,
        /// Fenêtre opaque du fournisseur, par exemple `five_hour`.
        window: String,
        /// Statut opaque attesté, par exemple `allowed` ou `rejected`.
        status: String,
        /// Instant Unix de retour fourni par le fournisseur, absent si inconnu.
        #[serde(default)]
        resets_at: Option<i64>,
        source: RateLimitSource,
    },
    /// Remplacer le domaine d'un agent, ou revenir au domaine dérivé.
    ///
    /// `domain: None` signifie « réinitialiser » : le daemon reprend alors le
    /// domaine annoncé à l'enregistrement.
    Domain {
        agent: String,
        #[serde(default)]
        domain: Option<String>,
    },
    /// Déclarer la disponibilité d'un agent.
    ///
    /// `until_secs` est un horodatage Unix jusqu'auquel l'agent refuse d'être
    /// dérangé. `None` lève le statut immédiatement. Représenter une échéance
    /// plutôt qu'un booléen rend l'expiration automatique sans tâche de fond.
    Availability {
        agent: String,
        #[serde(default)]
        until_secs: Option<u64>,
    },
}

/// Origine d'une observation de runtime. Énumération fermée : une valeur
/// inconnue rend le message indécodable plutôt que d'entrer dans l'annuaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeSource {
    /// Lu dans le fichier rollout d'un agent Codex.
    #[serde(rename = "codex-rollout")]
    CodexRollout,
    /// Rapporté par le hook Stop d'un agent Claude Code.
    #[serde(rename = "claude-hook")]
    ClaudeHook,
    /// Lu de manière périodique dans le transcript JSONL de Claude Code.
    #[serde(rename = "claude-transcript")]
    ClaudeTranscript,
    /// Déclaré explicitement via `bridget runtime`.
    #[serde(rename = "declared")]
    Declared,
}

impl std::fmt::Display for RuntimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            RuntimeSource::CodexRollout => "codex-rollout",
            RuntimeSource::ClaudeHook => "claude-hook",
            RuntimeSource::ClaudeTranscript => "claude-transcript",
            RuntimeSource::Declared => "declared",
        };
        f.write_str(label)
    }
}

/// Origine d'une observation de limite. Elle est fermée afin qu'un fournisseur
/// inconnu ne puisse pas se faire passer pour une capacité attestée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitSource {
    /// Événement `rate_limit_event` effectivement lu du flux Claude natif.
    #[serde(rename = "claude-stream-json")]
    ClaudeStreamJson,
}

impl std::fmt::Display for RateLimitSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitSource::ClaudeStreamJson => f.write_str("claude-stream-json"),
        }
    }
}

/// Messages envoyés par le daemon vers le wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedMcpDefinition {
    pub interactive: String,
    pub acp_session: bool,
}

/// Matrice déclarative des capacités réellement promises par un adaptateur.
///
/// Elle est portée par la définition résolue et donc par son digest : une
/// reprise ne relit jamais une sonde volatile du pilote pour décider si elle
/// peut démarrer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdapterCapabilities {
    #[serde(default)]
    pub execution_paths: Vec<String>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelCapabilities>,
}

impl Default for AdapterCapabilities {
    fn default() -> Self {
        // Les définitions historiques étaient toutes lancées par ACP. Ce seul
        // chemin reste la valeur de migration ; un modèle explicite demeure
        // absent tant qu'il n'est pas réellement déclaré.
        Self {
            execution_paths: vec!["acp".to_string()],
            models: BTreeMap::new(),
        }
    }
}

/// Capacités opaques déclarées pour un modèle précis, sans substitution.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelCapabilities {
    #[serde(default)]
    pub efforts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedAgentDefinition {
    pub command: String,
    pub args: Vec<String>,
    pub protocol: String,
    pub forbidden_env: Vec<String>,
    /// Noms des variables héritées ; leurs valeurs secrètes ne sont jamais
    /// persistées ni exposées dans la preuve publique.
    pub pass_env: Vec<String>,
    pub permissions: String,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    pub mcp: ResolvedMcpDefinition,
    #[serde(default)]
    pub capabilities: AdapterCapabilities,
    /// SHA-256 hexadécimal de tous les paramètres runtime précédents.
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonToWrapper {
    /// Le rôle demandé est accepté pour cette connexion.
    RoleAccepted { role: ConnectionRole },
    /// Contrat et capacités réellement négociés avec un client public.
    ClientWelcome {
        version: u16,
        #[serde(default = "unknown_build_id")]
        build_id: String,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        capabilities: Vec<ClientCapability>,
    },
    /// Refus motivé de la négociation ou de la matrice client.
    ClientRejected { reason: ClientRefusal },
    /// Contrat et capacité réellement négociés avec un service Maicie.
    ServiceWelcome {
        version: u16,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        capabilities: Vec<ServiceCapability>,
    },
    /// Refus motivé de la négociation ou de la matrice de service.
    ServiceRejected { reason: ServiceRefusal },
    /// Issue durable ou calculée d'une opération du guichet.
    #[serde(rename = "guichet_result")]
    GuichetResult {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        issue: String,
        expires_at: i64,
    },
    /// Une seule demande a été relevée sous une lease durable.
    #[serde(rename = "guichet_claimed")]
    GuichetClaimed {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        #[serde(with = "base64_bytes")]
        canonical_request: Vec<u8>,
        claimed_at: i64,
        claim_generation: u64,
        claim_token: String,
        claim_lease_expires_at: i64,
        expires_at: i64,
    },
    /// Aucun élément FIFO n'est actuellement relevable.
    #[serde(rename = "guichet_empty")]
    GuichetEmpty {
        #[serde(rename = "v")]
        version: u16,
    },
    /// Fait terminal durable, émis exclusivement par Bridget vers le service
    /// Maicie après la transition SQLite correspondante.
    #[serde(rename = "request_lifecycle_event")]
    RequestLifecycleEvent {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        event_id: String,
        request_id: String,
        state: GuichetLifecycleState,
        observed_at: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response_message_id: Option<String>,
    },
    /// Fait non terminal de coordination, envoyé seulement aux services ayant
    /// négocié `coordination_events_v1` en plus de `maicie_guichet`.
    #[serde(rename = "coordination_event")]
    CoordinationEvent {
        #[serde(rename = "v")]
        version: u16,
        event_id: String,
        request_id: String,
        kind: CoordinationEventKind,
        reminder_message_id: String,
        recipient: String,
        generation: u64,
        observed_at: i64,
    },
    /// Issue durable ou calculée d'un `SendIdempotent`.
    IdempotencyResult {
        operation_kind: String,
        idempotency_key: String,
        issue: IdempotencyIssue,
    },
    /// Succès d'un spawn, émis seulement après le `Register` réel.
    SpawnAccepted {
        command_id: String,
        name: String,
        /// `None` n'est toléré que pour le rejeu d'une issue créée avant la
        /// migration du registre résolu ; tout nouveau spawn fournit `Some`.
        definition: Option<ResolvedAgentDefinition>,
    },
    /// Refus terminal et rejouable d'un spawn.
    SpawnRejected {
        command_id: String,
        reason: SpawnRefusal,
    },
    /// Issue synchrone d'un ordre d'arrêt.
    StopResult {
        command_id: String,
        outcome: StopOutcome,
    },
    /// Remise aval réservée au wrapper destinataire.
    DeliverIdempotent {
        delivery_id: String,
        recipient_instance_id: String,
        delivery_generation: u64,
        expires_at: i64,
        message: BridgetMessage,
    },
    /// Souscription du daemon vers le wrapper lecteur du journal.
    Subscribe {
        subscription_id: String,
        agent: String,
        window: AttachWindow,
    },
    /// Désabonnement relayé au wrapper.
    Unsubscribe { subscription_id: String },
    /// Confirmation d'abonnement envoyée à la vue après acceptation wrapper.
    Subscribed { subscription_id: String },
    /// Fragment d'événement relayé à la vue attachée.
    JournalFragment {
        subscription_id: String,
        seq: u64,
        offset: u64,
        #[serde(rename = "final")]
        final_fragment: bool,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
    },
    /// Le rejeu est terminé ; `through_seq` est absent si la fenêtre est vide.
    SnapshotCaughtUp {
        subscription_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        through_seq: Option<u64>,
    },
    /// Lacune de rendu coalescée, émise avant l'événement suivant conservé.
    Gap {
        subscription_id: String,
        from_seq: u64,
        to_seq: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Diagnostic relayé quand une ligne n'a pas de séquence à lacuner.
    JournalReadError {
        subscription_id: String,
        line: u64,
        offset: u64,
        reason: String,
    },
    /// Fin motivée d'un abonnement attach.
    End {
        subscription_id: String,
        reason: String,
    },
    /// Refus typé du plan de contrôle attach.
    AttachRejected {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subscription_id: Option<String>,
        reason: AttachRefusal,
        /// Mode attesté qui motive un refus d'attachement. Absent pour les
        /// refus sans agent ou émis par un wrapper qui ne connaît pas la
        /// présence complète.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<PresenceMode>,
        /// Localisation interactive attestée, uniquement utile pour le mode
        /// tmux. Elle n'est jamais déduite ni reconstruite côté client.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        location: Option<String>,
    },
    /// Confirmation d'enregistrement avec le nom final.
    Registered { name: String },
    /// Confirmation d'un renommage.
    Renamed { old_name: String, name: String },
    /// Livrer un message à l'agent.
    Deliver(BridgetMessage),
    /// Retirer un message de la file du transport, sans l'injecter.
    CancelDelivery { id: String, reason: String },
    /// Acquittement d'un envoi.
    Ack { id: String },
    /// Refus d'un envoi avec raison.
    Nack { id: String, reason: String },
    /// Échec terminal différé d'un envoi attach déjà acquitté.
    DeliveryRejected { id: String, reason: String },
    /// Le daemon s'éteint.
    Disconnect,
    /// Réponse à ListAgents.
    AgentList { agents: Vec<AgentInfo> },
    /// État final d'une annulation.
    RequestCancelled { id: String, state: String },
    /// Liste des demandes suivies accessibles à l'agent courant.
    RequestList { requests: Vec<RequestInfo> },
    /// Projection bornée du ledger, indépendante de tout rendu CLI.
    LedgerProjection {
        messages: Vec<LedgerMessage>,
        requests: Vec<RequestInfo>,
    },
}

fn unknown_build_id() -> String {
    "unknown".to_string()
}

/// Sous-ensembles fermés de la projection de lecture du ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerScope {
    Messages,
    Requests,
    Both,
}

/// Échange stocké par le daemon et exposé aux clients de lecture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerMessage {
    pub id: String,
    pub ts: i64,
    pub sender: String,
    pub target: String,
    pub body: String,
}

impl WrapperToDaemon {
    /// Vérifie la matrice fermée d'une connexion déjà négociée comme attach.
    /// Le handshake est volontairement exclu : il n'est admis qu'avant que le
    /// daemon n'enregistre le rôle de la connexion.
    pub fn attach_refusal(&self) -> Option<AttachRefusal> {
        match self {
            Self::Subscribe { .. } | Self::Unsubscribe { .. } | Self::Heartbeat => None,
            Self::Send(message) if !message.reply => None,
            Self::Send(_) => Some(AttachRefusal::ReplyNotAllowed),
            _ => Some(AttachRefusal::MessageOutsideAttachRole),
        }
    }
}

impl DaemonToWrapper {
    /// Vérifie la matrice de réception du client attach. Le daemon l'emploiera
    /// lors du fan-out : les livraisons réservées au wrapper ne traversent pas
    /// la frontière de rôle.
    pub fn allowed_for_attach(&self) -> bool {
        matches!(
            self,
            Self::RoleAccepted {
                role: ConnectionRole::Attach
            } | Self::Subscribed { .. }
                | Self::JournalFragment { .. }
                | Self::SnapshotCaughtUp { .. }
                | Self::Gap { .. }
                | Self::JournalReadError { .. }
                | Self::End { .. }
                | Self::AttachRejected { .. }
                | Self::Ack { .. }
                | Self::Nack { .. }
                | Self::DeliveryRejected { .. }
        )
    }
}

/// Sérialise un message en ligne JSON (newline-delimited JSON).
pub fn encode<T: Serialize>(msg: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}

/// Désérialise une ligne JSON.
pub fn decode<T: for<'de> Deserialize<'de>>(line: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(line)
}

/// Information sur un agent connecté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub agent_type: String,
    pub connection_id: String,
    pub host: String,
    pub transport: String,
    /// Mode d'attelage attesté. `None` représente une présence historique
    /// dont le mode n'a jamais été annoncé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<PresenceMode>,
    /// Localisation interactive la plus précise attestée, jamais reconstruite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default = "unknown_os")]
    pub os: String,
    pub state: String,
    pub last_seen_secs: u64,
    pub reconnect_count: u32,
    /// Regroupement de travail, `None` si indéterminable.
    #[serde(default)]
    pub domain: Option<String>,
    /// Modèle courant, `None` tant qu'aucune observation n'a eu lieu.
    #[serde(default)]
    pub model: Option<String>,
    /// Niveau d'effort courant. `None` couvre deux cas indiscernables pour un
    /// lecteur : jamais observé, ou observé absent (modèle sans réglage
    /// d'effort). Les deux s'affichent de la même façon.
    #[serde(default)]
    pub effort: Option<String>,
    /// Dernier fait de limite attesté par le pilote, absent si aucun n'a été
    /// observé. Ce champ est purement informatif : il ne modifie pas l'état de
    /// présence ni le routage.
    #[serde(default)]
    pub rate_limit: Option<RateLimitFact>,
}

/// Fait de limite exposé dans l'annuaire. Les chaînes fournisseur restent
/// opaques ; seul `resets_at` manquant signifie explicitement « retour inconnu ».
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitFact {
    pub window: String,
    pub status: String,
    #[serde(default)]
    pub resets_at: Option<i64>,
}

fn unknown_os() -> String {
    "inconnu".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub id: String,
    pub sender: String,
    pub target: String,
    pub state: String,
    pub created_at: i64,
    pub deadline_at: i64,
    pub cancel_reason: Option<String>,
    #[serde(default)]
    pub deferred_reminder_level: Option<u8>,
    #[serde(default)]
    pub deferred_reminder_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVICE_NEGOTIATION_FIXTURE: &str = include_str!(
        "../../../specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl"
    );
    const COORDINATION_EVENTS_FIXTURE: &str = include_str!(
        "../../../specs/016-coordination-active/contracts/fixtures/coordination-events-v1.jsonl"
    );

    #[test]
    fn service_negotiation_v1_emploie_la_fixture_canonique_partagee() {
        let lines = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 5, "la fixture couvre hello, welcome et refus");

        let role: WrapperToDaemon = decode(lines[0]).unwrap();
        assert_eq!(encode(&role).unwrap(), lines[0]);
        assert!(matches!(
            role,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service
            }
        ));

        let accepted: DaemonToWrapper = decode(lines[1]).unwrap();
        assert_eq!(encode(&accepted).unwrap(), lines[1]);
        let hello: WrapperToDaemon = decode(lines[2]).unwrap();
        assert_eq!(encode(&hello).unwrap(), lines[2]);
        assert!(matches!(hello, WrapperToDaemon::ServiceHello { .. }));

        let welcome: DaemonToWrapper = decode(lines[3]).unwrap();
        assert_eq!(encode(&welcome).unwrap(), lines[3]);
        assert!(matches!(welcome, DaemonToWrapper::ServiceWelcome { .. }));

        let rejected: DaemonToWrapper = decode(lines[4]).unwrap();
        assert_eq!(encode(&rejected).unwrap(), lines[4]);
        assert!(matches!(
            rejected,
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::CapabilityRequired
            }
        ));
    }

    #[test]
    fn coordination_events_v1_emploie_la_fixture_canonique_fermee() {
        let lines = COORDINATION_EVENTS_FIXTURE.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 6, "négociation, fait attesté et refus");

        let hello: WrapperToDaemon = decode(lines[2]).unwrap();
        assert_eq!(encode(&hello).unwrap(), lines[2]);
        assert!(matches!(
            hello,
            WrapperToDaemon::ServiceHello { capabilities, .. }
                if capabilities == vec![
                    ServiceCapability::MaicieGuichet,
                    ServiceCapability::CoordinationEventsV1,
                ]
        ));

        let fact: DaemonToWrapper = decode(lines[4]).unwrap();
        assert_eq!(encode(&fact).unwrap(), lines[4]);
        assert!(matches!(
            fact,
            DaemonToWrapper::CoordinationEvent {
                kind: CoordinationEventKind::ReminderSent,
                generation: 1,
                observed_at: 1_787_500_003,
                ..
            }
        ));

        let rejected: DaemonToWrapper = decode(lines[5]).unwrap();
        assert!(matches!(
            rejected,
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::InvalidEnvelope
            }
        ));
    }

    #[test]
    fn test_encode_decode_register() {
        let msg = WrapperToDaemon::Register {
            agent_type: "codex".to_string(),
            name: None,
            host: Some("test-host".to_string()),
            transport: Some("unix".to_string()),
            mode: Some(PresenceMode::Acp),
            location: None,
            os: Some("Linux".to_string()),
            instance_id: Some("instance-test".to_string()),
            domain: Some("bridget".to_string()),
            turn_in_progress: false,
            journal_available: Some(false),
        };
        let json = encode(&msg).unwrap();
        assert!(json.contains("\"type\":\"Register\""));
        let decoded: WrapperToDaemon = decode(&json).unwrap();
        match decoded {
            WrapperToDaemon::Register {
                agent_type,
                name,
                host,
                transport,
                mode,
                location,
                os,
                instance_id,
                domain,
                turn_in_progress,
                journal_available,
            } => {
                assert_eq!(agent_type, "codex");
                assert!(name.is_none());
                assert_eq!(host.as_deref(), Some("test-host"));
                assert_eq!(transport.as_deref(), Some("unix"));
                assert_eq!(mode, Some(PresenceMode::Acp));
                assert_eq!(location, None);
                assert_eq!(os.as_deref(), Some("Linux"));
                assert_eq!(instance_id.as_deref(), Some("instance-test"));
                assert_eq!(domain.as_deref(), Some("bridget"));
                assert!(!turn_in_progress);
                assert_eq!(journal_available, Some(false));
            }
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn register_historique_conserve_un_mode_inconnu() {
        let json =
            r#"{"type":"Register","agent_type":"codex","name":null,"turn_in_progress":false}"#;
        let decoded: WrapperToDaemon = decode(json).unwrap();
        assert!(matches!(
            decoded,
            WrapperToDaemon::Register {
                mode: None,
                location: None,
                ..
            }
        ));
    }

    #[test]
    fn test_encode_decode_deliver() {
        let msg = BridgetMessage::new("claude-1", "codex-1", "hello");
        let dtw = DaemonToWrapper::Deliver(msg.clone());
        let json = encode(&dtw).unwrap();
        assert!(json.contains("\"type\":\"Deliver\""));
        let decoded: DaemonToWrapper = decode(&json).unwrap();
        match decoded {
            DaemonToWrapper::Deliver(m) => {
                assert_eq!(m.from, "claude-1");
                assert_eq!(m.body, "hello");
            }
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn test_encode_decode_send() {
        let msg = BridgetMessage::new("codex-1", "claude-1", "réponse");
        let wtd = WrapperToDaemon::Send(msg);
        let json = encode(&wtd).unwrap();
        let decoded: WrapperToDaemon = decode(&json).unwrap();
        match decoded {
            WrapperToDaemon::Send(m) => assert_eq!(m.body, "réponse"),
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn test_encode_decode_acp_delivery_lifecycle() {
        let cancel = DaemonToWrapper::CancelDelivery {
            id: "message-1".to_string(),
            reason: "échéance".to_string(),
        };
        assert!(matches!(
            decode(&encode(&cancel).unwrap()).unwrap(),
            DaemonToWrapper::CancelDelivery { id, reason } if id == "message-1" && reason == "échéance"
        ));
        let rejected = WrapperToDaemon::DeliveryRejected {
            id: "message-1".to_string(),
            reason: "file ACP pleine".to_string(),
        };
        assert!(matches!(
            decode(&encode(&rejected).unwrap()).unwrap(),
            WrapperToDaemon::DeliveryRejected { id, reason } if id == "message-1" && reason == "file ACP pleine"
        ));
        assert!(matches!(
            decode(&encode(&WrapperToDaemon::TurnState { in_progress: true }).unwrap()).unwrap(),
            WrapperToDaemon::TurnState { in_progress: true }
        ));
    }

    #[test]
    fn test_encode_decode_rename() {
        let msg = WrapperToDaemon::Rename {
            current_name: "codex-1".to_string(),
            name: "analyse".to_string(),
        };
        let json = encode(&msg).unwrap();
        assert!(json.contains("\"type\":\"Rename\""));
        assert!(
            matches!(decode(&json).unwrap(), WrapperToDaemon::Rename { current_name, name } if current_name == "codex-1" && name == "analyse")
        );

        let response = DaemonToWrapper::Renamed {
            old_name: "codex-1".to_string(),
            name: "analyse".to_string(),
        };
        assert!(
            matches!(decode(&encode(&response).unwrap()).unwrap(), DaemonToWrapper::Renamed { old_name, name } if old_name == "codex-1" && name == "analyse")
        );
    }

    #[test]
    fn test_encode_decode_runtime() {
        let msg = WrapperToDaemon::Runtime {
            agent: "agent-2".to_string(),
            model: "claude-opus-5".to_string(),
            effort: Some("high".to_string()),
            source: RuntimeSource::ClaudeHook,
        };
        let json = encode(&msg).unwrap();
        assert!(json.contains("\"type\":\"Runtime\""));
        assert!(json.contains("\"source\":\"claude-hook\""));
        match decode(&json).unwrap() {
            WrapperToDaemon::Runtime {
                agent,
                model,
                effort,
                source,
            } => {
                assert_eq!(agent, "agent-2");
                assert_eq!(model, "claude-opus-5");
                assert_eq!(effort.as_deref(), Some("high"));
                assert_eq!(source, RuntimeSource::ClaudeHook);
            }
            other => panic!("mauvais type: {:?}", other),
        }
    }

    #[test]
    fn runtime_source_claude_transcript_est_stable() {
        let encoded = encode(&RuntimeSource::ClaudeTranscript).unwrap();
        assert_eq!(encoded, "\"claude-transcript\"");
        assert_eq!(
            decode::<RuntimeSource>(&encoded).unwrap(),
            RuntimeSource::ClaudeTranscript
        );
        assert_eq!(
            RuntimeSource::ClaudeTranscript.to_string(),
            "claude-transcript"
        );
    }

    #[test]
    fn test_runtime_effort_absent_est_decodable() {
        // Cas Haiku : le modèle n'expose aucun niveau d'effort.
        let json = r#"{"type":"Runtime","agent":"agent-2","model":"claude-haiku-4-5","source":"codex-rollout"}"#;
        match decode(json).unwrap() {
            WrapperToDaemon::Runtime { effort, .. } => assert!(effort.is_none()),
            other => panic!("mauvais type: {:?}", other),
        }
    }

    #[test]
    fn test_runtime_source_inconnue_est_refusee() {
        let json = r#"{"type":"Runtime","agent":"agent-2","model":"x","source":"inventee"}"#;
        assert!(decode::<WrapperToDaemon>(json).is_err());
    }

    #[test]
    fn test_encode_decode_rate_limit_et_absence_de_retour() {
        let message = WrapperToDaemon::RateLimit {
            agent: "claude-1".to_string(),
            window: "five_hour".to_string(),
            status: "rejected".to_string(),
            resets_at: Some(1_787_572_200),
            source: RateLimitSource::ClaudeStreamJson,
        };
        let encoded = encode(&message).unwrap();
        assert!(encoded.contains("\"type\":\"RateLimit\""));
        assert!(encoded.contains("\"source\":\"claude-stream-json\""));
        assert!(matches!(
            decode(&encoded).unwrap(),
            WrapperToDaemon::RateLimit {
                agent,
                window,
                status,
                resets_at: Some(1_787_572_200),
                source: RateLimitSource::ClaudeStreamJson,
            } if agent == "claude-1" && window == "five_hour" && status == "rejected"
        ));

        let without_reset = r#"{"type":"RateLimit","agent":"claude-1","window":"five_hour","status":"allowed","source":"claude-stream-json"}"#;
        assert!(matches!(
            decode(without_reset).unwrap(),
            WrapperToDaemon::RateLimit {
                resets_at: None,
                ..
            }
        ));
    }

    #[test]
    fn test_agent_info_sans_runtime_reste_decodable() {
        // Compatibilité ascendante : un daemon d'une version antérieure ne
        // sérialise ni model ni effort.
        let json = r#"{"name":"agent-2","agent_type":"claude","connection_id":"conn-1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0}"#;
        let info: AgentInfo = decode(json).unwrap();
        assert!(info.model.is_none());
        assert!(info.effort.is_none());
        assert!(info.rate_limit.is_none());
    }

    #[test]
    fn test_encode_decode_nack() {
        let dtw = DaemonToWrapper::Nack {
            id: "abc123".to_string(),
            reason: "agent introuvable".to_string(),
        };
        let json = encode(&dtw).unwrap();
        let decoded: DaemonToWrapper = decode(&json).unwrap();
        match decoded {
            DaemonToWrapper::Nack { id, reason } => {
                assert_eq!(id, "abc123");
                assert_eq!(reason, "agent introuvable");
            }
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn attach_client_messages_roundtrip() {
        let handshake = WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        };
        assert!(matches!(
            decode(&encode(&handshake).unwrap()).unwrap(),
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            }
        ));

        let subscribe = WrapperToDaemon::Subscribe {
            agent: "codex-1".to_string(),
            window: AttachWindow::Seq(42),
        };
        assert!(matches!(
            decode(&encode(&subscribe).unwrap()).unwrap(),
            WrapperToDaemon::Subscribe {
                agent,
                window: AttachWindow::Seq(42)
            } if agent == "codex-1"
        ));

        let unsubscribe = WrapperToDaemon::Unsubscribe {
            subscription_id: "sub-1".to_string(),
        };
        assert!(matches!(
            decode(&encode(&unsubscribe).unwrap()).unwrap(),
            WrapperToDaemon::Unsubscribe { subscription_id } if subscription_id == "sub-1"
        ));
    }

    #[test]
    fn service_guichet_messages_roundtrip_et_restent_hors_attach() {
        let hello = WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![ServiceCapability::MaicieGuichet],
        };
        assert_eq!(
            encode(&hello).unwrap(),
            SERVICE_NEGOTIATION_FIXTURE.lines().nth(2).unwrap()
        );
        assert!(matches!(
            decode(&encode(&hello).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                capabilities,
                ..
            } if capabilities == vec![ServiceCapability::MaicieGuichet]
        ));

        let reply = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-1".to_string(),
            claim_generation: 3,
            claim_token: "claim-1".to_string(),
            response_message_id: "msg-1".to_string(),
            in_reply_to: "message-1".to_string(),
            outcome: GuichetOutcome::Accepted,
            payload: GuichetReplyPayload::DeliveryReport {
                objective_id: "objective-1".to_string(),
                delegation_id: "delegation-1".to_string(),
                delivery_hash: "0".repeat(64),
            },
        };
        assert!(matches!(
            decode(&encode(&reply).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                claim_generation: 3,
                claim_token,
                ..
            } if claim_token == "claim-1"
        ));
        assert_eq!(
            reply.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );

        let refused = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-refused".to_string(),
            claim_generation: 1,
            claim_token: "claim-refused".to_string(),
            response_message_id: "msg-refused".to_string(),
            in_reply_to: "message-refused".to_string(),
            outcome: GuichetOutcome::Refused,
            payload: GuichetReplyPayload::Refused {
                operation: ServiceRequestOperation::MissionStatus,
                reason: GuichetRefusalReason::DelegationMissing,
            },
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&refused).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                outcome: GuichetOutcome::Refused,
                payload: GuichetReplyPayload::Refused {
                    operation: ServiceRequestOperation::MissionStatus,
                    reason: GuichetRefusalReason::DelegationMissing,
                },
                ..
            }
        ));

        let status = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-status".to_string(),
            claim_generation: 4,
            claim_token: "claim-2".to_string(),
            response_message_id: "message-2".to_string(),
            in_reply_to: "request-status".to_string(),
            outcome: GuichetOutcome::Accepted,
            payload: GuichetReplyPayload::MissionStatus {
                delegation_id: "delegation-1".to_string(),
                objective_id: "objective-1".to_string(),
                coordination_state: GuichetCoordinationState::EnCoordination,
                local_delivery: GuichetLocalDelivery {
                    state: GuichetLocalDeliveryState::Accepted,
                    issue: Some("accepted".to_string()),
                    observed_at: Some(1_787_500_001),
                },
                transport_observation: Some(GuichetTransportObservation {
                    state: GuichetTransportState::Connected,
                    request_state: Some("answered".to_string()),
                    observed_at: 1_787_500_002,
                    source: "attach".to_string(),
                    subscription_id: Some("subscription-1".to_string()),
                    seq: Some(7),
                }),
                freshness: GuichetFreshness::Fresh,
            },
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&status).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                payload: GuichetReplyPayload::MissionStatus {
                    freshness: GuichetFreshness::Fresh,
                    transport_observation: Some(GuichetTransportObservation { seq: Some(7), .. }),
                    ..
                },
                ..
            }
        ));

        let rejected = DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired,
        };
        assert!(matches!(
            decode(&encode(&rejected).unwrap()).unwrap(),
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::CapabilityRequired
            }
        ));
        assert!(!rejected.allowed_for_attach());

        let lifecycle = DaemonToWrapper::RequestLifecycleEvent {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            event_id: "evt-1".to_string(),
            request_id: "request-1".to_string(),
            state: GuichetLifecycleState::Answered,
            observed_at: 1_787_500_000,
            in_reply_to: Some("message-1".to_string()),
            response_message_id: Some("response-1".to_string()),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&lifecycle).unwrap()).unwrap(),
            DaemonToWrapper::RequestLifecycleEvent {
                state: GuichetLifecycleState::Answered,
                in_reply_to: Some(in_reply_to),
                response_message_id: Some(response_message_id),
                ..
            } if in_reply_to == "message-1" && response_message_id == "response-1"
        ));
        assert!(!lifecycle.allowed_for_attach());

        let coordination = DaemonToWrapper::CoordinationEvent {
            version: COORDINATION_EVENTS_VERSION,
            event_id: "evt-reminder-1".to_string(),
            request_id: "request-1".to_string(),
            kind: CoordinationEventKind::ReminderSent,
            reminder_message_id: "message-reminder-1".to_string(),
            recipient: "codex-1".to_string(),
            generation: 1,
            observed_at: 1_787_500_003,
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&coordination).unwrap()).unwrap(),
            DaemonToWrapper::CoordinationEvent {
                version: COORDINATION_EVENTS_VERSION,
                kind: CoordinationEventKind::ReminderSent,
                generation: 1,
                observed_at: 1_787_500_003,
                ..
            }
        ));
        assert!(!coordination.allowed_for_attach());

        let hello_with_coordination = WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![
                ServiceCapability::MaicieGuichet,
                ServiceCapability::CoordinationEventsV1,
            ],
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&hello_with_coordination).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello { capabilities, .. }
                if capabilities == vec![
                    ServiceCapability::MaicieGuichet,
                    ServiceCapability::CoordinationEventsV1,
                ]
        ));
        assert!(decode::<DaemonToWrapper>(
            r#"{"type":"coordination_event","v":1,"event_id":"evt","request_id":"req","kind":"unknown_fact","reminder_message_id":"msg","recipient":"codex","generation":1,"observed_at":1}"#
        )
        .is_err());
    }

    #[test]
    fn client_idempotency_messages_roundtrip_and_stay_outside_attach() {
        let hello = WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
            capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
        };
        assert!(matches!(
            decode(&encode(&hello).unwrap()).unwrap(),
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope,
                capabilities,
            } if issuer_scope == "012_scope_aaaaaaaaaaaa"
                && capabilities == vec![ClientCapability::SendIdempotent, ClientCapability::Lookup]
        ));
        assert_eq!(
            hello.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
        let welcome = DaemonToWrapper::ClientWelcome {
            version: CLIENT_CONTRACT_VERSION,
            build_id: "fixture-build".to_string(),
            horizon_secs: 60,
            issued_at_tolerance_secs: 5,
            capabilities: vec![ClientCapability::Lookup],
        };
        let decoded: DaemonToWrapper = decode(&encode(&welcome).unwrap()).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                build_id,
                capabilities,
                ..
            } if build_id == "fixture-build" && capabilities == vec![ClientCapability::Lookup]
        ));
        let result = DaemonToWrapper::IdempotencyResult {
            operation_kind: "send".to_string(),
            idempotency_key: "message-1".to_string(),
            issue: IdempotencyIssue::OutcomeUnknown {
                expires_at: 123,
                delivery_id: Some("delivery-1".to_string()),
            },
        };
        assert!(matches!(
            decode(&encode(&result).unwrap()).unwrap(),
            DaemonToWrapper::IdempotencyResult {
                operation_kind,
                idempotency_key,
                issue: IdempotencyIssue::OutcomeUnknown {
                    expires_at: 123,
                    delivery_id: Some(delivery_id),
                },
            } if operation_kind == "send" && idempotency_key == "message-1" && delivery_id == "delivery-1"
        ));
        assert!(!welcome.allowed_for_attach());
    }

    #[test]
    fn client_welcome_historique_signale_un_build_id_inconnu() {
        let decoded: DaemonToWrapper = decode(
            r#"{"type":"ClientWelcome","version":1,"horizon_secs":60,"issued_at_tolerance_secs":5,"capabilities":[]}"#,
        )
        .unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::ClientWelcome { build_id, .. } if build_id == "unknown"
        ));
    }

    #[test]
    fn lifecycle_messages_roundtrip_and_stay_outside_attach() {
        let spawn = WrapperToDaemon::SpawnOrder {
            agent_type: "codex".to_string(),
            name: Some("codex-1".to_string()),
            cwd: "/tmp".to_string(),
            persistent: true,
            command_id: "command-1".to_string(),
            issued_at: 100,
            deadline_at: 110,
        };
        assert!(matches!(
            decode(&encode(&spawn).unwrap()).unwrap(),
            WrapperToDaemon::SpawnOrder {
                agent_type,
                name: Some(name),
                command_id,
                ..
            } if agent_type == "codex" && name == "codex-1" && command_id == "command-1"
        ));
        assert_eq!(
            spawn.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
        let rejection = DaemonToWrapper::SpawnRejected {
            command_id: "command-1".to_string(),
            reason: SpawnRefusal::BillingGuard {
                variable: "OPENAI_API_KEY".to_string(),
            },
        };
        assert!(matches!(
            decode(&encode(&rejection).unwrap()).unwrap(),
            DaemonToWrapper::SpawnRejected {
                reason: SpawnRefusal::BillingGuard { variable },
                ..
            } if variable == "OPENAI_API_KEY"
        ));
        assert!(!rejection.allowed_for_attach());
        let stop = DaemonToWrapper::StopResult {
            command_id: "stop-1".to_string(),
            outcome: StopOutcome::StoppedForced {
                survivors_killed: 2,
            },
        };
        assert!(matches!(
            decode(&encode(&stop).unwrap()).unwrap(),
            DaemonToWrapper::StopResult {
                outcome: StopOutcome::StoppedForced {
                    survivors_killed: 2
                },
                ..
            }
        ));
    }

    #[test]
    fn attach_relay_messages_roundtrip() {
        let messages = vec![
            WrapperToDaemon::Subscribed {
                subscription_id: "sub-1".to_string(),
            },
            WrapperToDaemon::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 7,
                offset: 0,
                final_fragment: true,
                bytes: b"{\"v\":1}\n".to_vec(),
            },
            WrapperToDaemon::LiveJournalFragment {
                seq: 8,
                offset: 0,
                final_fragment: true,
                bytes: b"{\"v\":1,\"seq\":8}".to_vec(),
            },
            WrapperToDaemon::SnapshotCaughtUp {
                subscription_id: "sub-1".to_string(),
                through_seq: Some(7),
            },
            WrapperToDaemon::Gap {
                subscription_id: "sub-1".to_string(),
                from_seq: 3,
                to_seq: 4,
                reason: Some("vue lente".to_string()),
            },
            WrapperToDaemon::End {
                subscription_id: "sub-1".to_string(),
                reason: "wrapper arrêté".to_string(),
            },
            WrapperToDaemon::AttachRejected {
                subscription_id: Some("sub-1".to_string()),
                reason: AttachRefusal::CommandQueueSaturated,
            },
            WrapperToDaemon::JournalReadError {
                subscription_id: "sub-1".to_string(),
                line: 3,
                offset: 42,
                reason: "ligne illisible".to_string(),
            },
        ];
        for message in messages {
            assert_eq!(
                encode(&message).unwrap(),
                encode(&decode::<WrapperToDaemon>(&encode(&message).unwrap()).unwrap()).unwrap()
            );
        }
        assert_eq!(MAX_ATTACH_FRAGMENT_BYTES, 190 * 1024);
        assert_eq!(MAX_ATTACH_SERIALIZED_FRAME_BYTES, 256 * 1024);
    }

    #[test]
    fn attach_daemon_messages_roundtrip() {
        let messages = vec![
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach,
            },
            DaemonToWrapper::Subscribe {
                subscription_id: "sub-1".to_string(),
                agent: "codex-1".to_string(),
                window: AttachWindow::Today,
            },
            DaemonToWrapper::Unsubscribe {
                subscription_id: "sub-1".to_string(),
            },
            DaemonToWrapper::Subscribed {
                subscription_id: "sub-1".to_string(),
            },
            DaemonToWrapper::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 8,
                offset: 0,
                final_fragment: true,
                bytes: vec![0, 1, 2],
            },
            DaemonToWrapper::SnapshotCaughtUp {
                subscription_id: "sub-1".to_string(),
                through_seq: None,
            },
            DaemonToWrapper::Gap {
                subscription_id: "sub-1".to_string(),
                from_seq: 6,
                to_seq: 7,
                reason: None,
            },
            DaemonToWrapper::End {
                subscription_id: "sub-1".to_string(),
                reason: "désabonné".to_string(),
            },
            DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::AgentStopped,
                mode: None,
                location: None,
            },
            DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::AgentNotAcp,
                mode: Some(PresenceMode::Tmux),
                location: Some("bridget:4.2".to_string()),
            },
            DaemonToWrapper::JournalReadError {
                subscription_id: "sub-1".to_string(),
                line: 3,
                offset: 42,
                reason: "ligne illisible".to_string(),
            },
            DaemonToWrapper::DeliveryRejected {
                id: "message-1".to_string(),
                reason: "processus arrêté".to_string(),
            },
        ];
        for message in messages {
            let reaches_attach = !matches!(
                message,
                DaemonToWrapper::Subscribe { .. } | DaemonToWrapper::Unsubscribe { .. }
            );
            let json = encode(&message).unwrap();
            let decoded: DaemonToWrapper = decode(&json).unwrap();
            assert_eq!(json, encode(&decoded).unwrap());
            assert_eq!(decoded.allowed_for_attach(), reaches_attach);
        }
    }

    #[test]
    fn attach_rejected_historique_omet_les_details_de_presence() {
        let legacy = r#"{"type":"AttachRejected","reason":"agent_not_acp"}"#;
        let decoded: DaemonToWrapper = decode(legacy).unwrap();
        match decoded {
            DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::AgentNotAcp,
                mode: None,
                location: None,
            } => {}
            other => panic!("message historique inattendu : {other:?}"),
        }
    }

    #[test]
    fn journal_ready_est_un_signal_wrapper_hors_du_role_attach() {
        let signal = WrapperToDaemon::JournalReady;
        let json = encode(&signal).unwrap();
        assert_eq!(json, r#"{"type":"JournalReady"}"#);
        assert!(matches!(
            decode(&json).unwrap(),
            WrapperToDaemon::JournalReady
        ));
        assert_eq!(
            signal.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
    }

    #[test]
    fn spawn_accepted_transporte_la_definition_resolue_complete() {
        let message = DaemonToWrapper::SpawnAccepted {
            command_id: "command-1".to_string(),
            name: "reviewer".to_string(),
            definition: Some(ResolvedAgentDefinition {
                command: "npx".to_string(),
                args: vec!["adapter@1.2.3".to_string()],
                protocol: "acp".to_string(),
                forbidden_env: vec!["API_KEY".to_string()],
                pass_env: vec!["HOME".to_string()],
                permissions: "allow".to_string(),
                queue_capacity: 32,
                notify_timeout_secs: 600,
                mcp: ResolvedMcpDefinition {
                    interactive: "codex".to_string(),
                    acp_session: true,
                },
                capabilities: AdapterCapabilities {
                    execution_paths: vec!["acp".to_string()],
                    models: BTreeMap::from([(
                        "gpt-5.6-terra".to_string(),
                        ModelCapabilities {
                            efforts: vec!["high".to_string()],
                        },
                    )]),
                },
                digest: "a".repeat(64),
            }),
        };
        let json = encode(&message).unwrap();
        let decoded = decode::<DaemonToWrapper>(&json).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::SpawnAccepted { definition: Some(definition), .. }
                if definition.command == "npx"
                    && definition.args == ["adapter@1.2.3"]
                    && definition.forbidden_env == ["API_KEY"]
                    && definition.capabilities.models["gpt-5.6-terra"].efforts == ["high"]
                    && definition.digest == "a".repeat(64)
        ));
        assert!(matches!(
            decode::<DaemonToWrapper>(
                r#"{"type":"SpawnAccepted","command_id":"legacy","name":"ancien"}"#
            )
            .unwrap(),
            DaemonToWrapper::SpawnAccepted {
                definition: None,
                ..
            }
        ));
    }

    #[test]
    fn fragment_base64_reste_sous_la_borne_filaire_pour_des_octets_hostiles() {
        for bytes in [
            vec![0; MAX_ATTACH_FRAGMENT_BYTES],
            vec![0xff; MAX_ATTACH_FRAGMENT_BYTES],
            "équipier".as_bytes().repeat(MAX_ATTACH_FRAGMENT_BYTES / 9),
        ] {
            let message = WrapperToDaemon::JournalFragment {
                subscription_id: "attach-0123456789abcdef".to_string(),
                seq: 7,
                offset: 0,
                final_fragment: true,
                bytes,
            };
            assert!(encode(&message).unwrap().len() <= MAX_ATTACH_SERIALIZED_FRAME_BYTES);
        }
    }

    #[test]
    fn role_attach_refuse_les_messages_wrapper_et_reply_suivi() {
        let wrapper_only = WrapperToDaemon::Runtime {
            agent: "codex-1".to_string(),
            model: "gpt-5.5".to_string(),
            effort: None,
            source: RuntimeSource::Declared,
        };
        assert_eq!(
            wrapper_only.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );

        let mut tracked_send = BridgetMessage::new("forge", "codex-1", "réponds");
        tracked_send.reply = true;
        assert_eq!(
            WrapperToDaemon::Send(tracked_send).attach_refusal(),
            Some(AttachRefusal::ReplyNotAllowed)
        );
        assert!(
            WrapperToDaemon::Send(BridgetMessage::new("humain", "codex-1", "bonjour"))
                .attach_refusal()
                .is_none()
        );
        assert!(!DaemonToWrapper::Deliver(BridgetMessage::new("a", "b", "x")).allowed_for_attach());
    }

    #[test]
    fn issue_differee_attach_reste_correlee_parmi_le_flux() {
        let flux = vec![
            DaemonToWrapper::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 11,
                offset: 0,
                final_fragment: true,
                bytes: b"premier".to_vec(),
            },
            DaemonToWrapper::DeliveryRejected {
                id: "message-humain".to_string(),
                reason: "file pleine".to_string(),
            },
            DaemonToWrapper::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 12,
                offset: 0,
                final_fragment: true,
                bytes: b"second".to_vec(),
            },
        ];
        let decoded = flux
            .into_iter()
            .map(|message| decode::<DaemonToWrapper>(&encode(&message).unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(
            decoded[0],
            DaemonToWrapper::JournalFragment { seq: 11, .. }
        ));
        assert!(
            matches!(decoded[1], DaemonToWrapper::DeliveryRejected { ref id, .. } if id == "message-humain")
        );
        assert!(matches!(
            decoded[2],
            DaemonToWrapper::JournalFragment { seq: 12, .. }
        ));
    }

    #[test]
    fn protocol_007_reste_compatible_sans_handshake() {
        let json =
            r#"{"type":"Register","agent_type":"codex","name":null,"turn_in_progress":false}"#;
        assert!(matches!(
            decode::<WrapperToDaemon>(json).unwrap(),
            WrapperToDaemon::Register { agent_type, .. } if agent_type == "codex"
        ));
    }

    #[test]
    fn refus_type_inconnu_historique_reste_lisible_apres_l_ajout_du_diagnostic() {
        let legacy: SpawnRefusal = serde_json::from_str(r#"{"kind":"unknown_type"}"#).unwrap();
        assert!(matches!(
            legacy,
            SpawnRefusal::UnknownType {
                requested_type,
                known_types,
                registry,
            } if requested_type.is_empty() && known_types.is_empty() && registry.is_empty()
        ));
    }

    #[test]
    fn test_encode_decode_bounded_ledger_projection() {
        let request = WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Both,
            limit: 20,
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&request).unwrap()).unwrap(),
            WrapperToDaemon::LedgerProjection {
                scope: LedgerScope::Both,
                limit: 20
            }
        ));

        let response = DaemonToWrapper::LedgerProjection {
            messages: vec![LedgerMessage {
                id: "m-1".to_string(),
                ts: 42,
                sender: "alice".to_string(),
                target: "bob".to_string(),
                body: "bonjour".to_string(),
            }],
            requests: Vec::new(),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&response).unwrap()).unwrap(),
            DaemonToWrapper::LedgerProjection { messages, requests }
                if messages.len() == 1 && messages[0].id == "m-1" && requests.is_empty()
        ));
    }
}
