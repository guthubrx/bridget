//! Définitions des messages JSON du protocole daemon/wrapper.
//!
//! Deux directions :
//! - WrapperToDaemon : ce que le wrapper envoie au daemon
//! - DaemonToWrapper : ce que le daemon envoie au wrapper

use bridget_core::BridgetMessage;
use serde::{Deserialize, Serialize};

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
}

/// Version actuellement publiée du contrat idempotent local.
pub const CLIENT_CONTRACT_VERSION: u16 = 1;

/// Capacité optionnelle du client idempotent. L'énumération fermée évite une
/// dégradation silencieuse lorsqu'un client demande une capacité inconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    SendIdempotent,
    Lookup,
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
    UnknownType,
    CommandMissing { command: String, registry: String },
    BillingGuard { variable: String },
    NameActive,
    EnvUnfit { detail: String },
    CwdGone,
    NegotiationFailed { detail: String },
    SpawnTimeout,
    QuotaExceeded { limit: usize },
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
    },
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
    ListRequests { sender: String },
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
    /// Déclaré explicitement via `bridget runtime`.
    #[serde(rename = "declared")]
    Declared,
}

impl std::fmt::Display for RuntimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            RuntimeSource::CodexRollout => "codex-rollout",
            RuntimeSource::ClaudeHook => "claude-hook",
            RuntimeSource::Declared => "declared",
        };
        f.write_str(label)
    }
}

/// Messages envoyés par le daemon vers le wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonToWrapper {
    /// Le rôle demandé est accepté pour cette connexion.
    RoleAccepted { role: ConnectionRole },
    /// Contrat et capacités réellement négociés avec un client public.
    ClientWelcome {
        version: u16,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        capabilities: Vec<ClientCapability>,
    },
    /// Refus motivé de la négociation ou de la matrice client.
    ClientRejected { reason: ClientRefusal },
    /// Issue durable ou calculée d'un `SendIdempotent`.
    IdempotencyResult {
        operation_kind: String,
        idempotency_key: String,
        issue: IdempotencyIssue,
    },
    /// Succès d'un spawn, émis seulement après le `Register` réel.
    SpawnAccepted { command_id: String, name: String },
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
}

fn unknown_os() -> String {
    "inconnu".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub id: String,
    pub target: String,
    pub state: String,
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

    #[test]
    fn test_encode_decode_register() {
        let msg = WrapperToDaemon::Register {
            agent_type: "codex".to_string(),
            name: None,
            host: Some("test-host".to_string()),
            transport: Some("unix".to_string()),
            os: Some("Linux".to_string()),
            instance_id: Some("instance-test".to_string()),
            domain: Some("bridget".to_string()),
            turn_in_progress: false,
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
                os,
                instance_id,
                domain,
                turn_in_progress,
            } => {
                assert_eq!(agent_type, "codex");
                assert!(name.is_none());
                assert_eq!(host.as_deref(), Some("test-host"));
                assert_eq!(transport.as_deref(), Some("unix"));
                assert_eq!(os.as_deref(), Some("Linux"));
                assert_eq!(instance_id.as_deref(), Some("instance-test"));
                assert_eq!(domain.as_deref(), Some("bridget"));
                assert!(!turn_in_progress);
            }
            _ => panic!("mauvais type"),
        }
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
    fn test_agent_info_sans_runtime_reste_decodable() {
        // Compatibilité ascendante : un daemon d'une version antérieure ne
        // sérialise ni model ni effort.
        let json = r#"{"name":"agent-2","agent_type":"claude","connection_id":"conn-1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0}"#;
        let info: AgentInfo = decode(json).unwrap();
        assert!(info.model.is_none());
        assert!(info.effort.is_none());
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
            horizon_secs: 60,
            issued_at_tolerance_secs: 5,
            capabilities: vec![ClientCapability::Lookup],
        };
        let decoded: DaemonToWrapper = decode(&encode(&welcome).unwrap()).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                capabilities,
                ..
            } if capabilities == vec![ClientCapability::Lookup]
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
                reason: AttachRefusal::AgentNotAcp,
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
}
