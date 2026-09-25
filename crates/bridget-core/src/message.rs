//! Types de messages qui circulent dans le protocole bridget.

use serde::{Deserialize, Serialize};

/// Type d'agent CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Codex,
    Claude,
    Gemini,
    Shell,
    Custom(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Codex => write!(f, "codex"),
            AgentType::Claude => write!(f, "claude"),
            AgentType::Gemini => write!(f, "gemini"),
            AgentType::Shell => write!(f, "shell"),
            AgentType::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "codex" => Ok(AgentType::Codex),
            "claude" => Ok(AgentType::Claude),
            "gemini" => Ok(AgentType::Gemini),
            "shell" => Ok(AgentType::Shell),
            other => Ok(AgentType::Custom(other.to_string())),
        }
    }
}

/// Source déclarée d une soumission. Absente dans un message historique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageOrigin {
    Human,
    Agent,
    Routine,
    System,
}

/// Effet demandé lors de la remise. L adaptateur ne le déduit jamais du texte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageIntent {
    QueueOnly,
    TriggerTurn,
    SteerCurrent,
    InterruptAndStart,
    ControlOnly,
}

/// Message normalisé qui circule entre agents via le daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgetMessage {
    /// UUID court pour déduplication et quarantaine.
    pub id: String,
    /// Identifiant opaque de l'expéditeur. Il sert exclusivement au routage.
    pub from: String,
    /// Nom de présentation de l'expéditeur, résolu par le daemon au moment de
    /// la remise. Cette valeur n'est jamais une clé de routage : le wrapper
    /// l'emploie seulement pour construire le prompt fournisseur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_display_name: Option<String>,
    /// Nom du destinataire (ex: "codex-2").
    pub to: String,
    /// Texte du message.
    pub body: String,
    /// true = une réponse est attendue, false = affirmation/acusé.
    #[serde(default)]
    pub reply: bool,
    /// Sauts restants avant coupure. Décrémenté à chaque transfert.
    #[serde(default = "default_hops")]
    pub hops: i32,
    /// Timeout en secondes pour une réponse (si reply=true).
    /// Le daemon relance le destinataire à T/3, 2T/3, puis notifie l'émetteur à T.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_timeout: Option<u64>,
    /// Échéance Unix absolue transmise au transport destinataire. Le daemon
    /// reste l'autorité de cycle de vie ; le transport refuse aussi un tour
    /// qui arriverait après cette échéance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_at: Option<u64>,
    /// Identifiant de la demande suivie à laquelle ce message répond.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    /// Vrai quand l'émetteur s'est nommé explicitement (`send --from`). Le
    /// daemon ne remplace alors jamais ce nom en silence : il le conserve s'il
    /// est adressable, sinon il refuse l'envoi. Absent du flux = faux, ce qui
    /// laisse le comportement par défaut inchangé.
    #[serde(default)]
    pub from_declared: bool,
    /// Origine déclarée. Absente pour les producteurs historiques.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
    /// Intention de remise, distincte du contenu du message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<MessageIntent>,
    /// Références durables optionnelles de mission ou délégation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
    /// Session 102 : alerte typée d'un fil inter-agents, construite par le seul
    /// chemin interne du daemon. Absente des messages directs et omise à la
    /// sérialisation ; un client ne peut pas la forger par `send`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_notice: Option<ThreadNotice>,
}

/// Métadonnée d'une sollicitation de fil (session 102) : le destinataire lit
/// lui-même les nouveautés ; l'alerte ne porte ni historique ni titre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadNotice {
    pub version: u16,
    pub thread_id: String,
    pub through_seq: u64,
    pub generation: u64,
}

fn default_hops() -> i32 {
    4
}

impl BridgetMessage {
    /// Crée un nouveau message avec un ID généré et les hops par défaut.
    pub fn new(from: impl Into<String>, to: impl Into<String>, body: impl Into<String>) -> Self {
        let id = uuid::Uuid::new_v4()
            .to_string()
            .replace('-', "")
            .chars()
            .take(13)
            .collect::<String>();
        BridgetMessage {
            id,
            from: from.into(),
            from_display_name: None,
            to: to.into(),
            body: body.into(),
            reply: false,
            hops: default_hops(),
            reply_timeout: None,
            deadline_at: None,
            in_reply_to: None,
            from_declared: false,
            origin: None,
            intent: None,
            references: Vec::new(),
            thread_notice: None,
        }
    }

    /// Décrémente les hops. Retourne false si le budget est épuisé.
    pub fn decrement_hops(&mut self) -> bool {
        self.hops -= 1;
        self.hops > 0
    }

    /// Génère une clé de contenu pour la déduplication par contenu.
    /// Combine le destinataire, le corps et la demande référencée.
    /// Deux réponses textuellement identiques à des demandes distinctes ne sont
    /// pas des doublons : leur `in_reply_to` porte une sémantique métier.
    pub fn content_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.to.hash(&mut hasher);
        self.body.hash(&mut hasher);
        self.in_reply_to.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn spec102_thread_notice_absente_par_defaut_et_omise() {
        let message = super::BridgetMessage::new("a", "b", "corps");
        assert!(message.thread_notice.is_none());
        let json = serde_json::to_string(&message).unwrap();
        assert!(!json.contains("thread_notice"));
        let old: super::BridgetMessage =
            serde_json::from_str(r#"{"id":"m","from":"a","to":"b","body":"x"}"#).unwrap();
        assert!(old.thread_notice.is_none());
        let typed: super::BridgetMessage = serde_json::from_str(
            r#"{"id":"m","from":"bridget","to":"b","body":"x","thread_notice":{"version":1,"thread_id":"t","through_seq":3,"generation":2}}"#,
        )
        .unwrap();
        assert_eq!(typed.thread_notice.as_ref().unwrap().through_seq, 3);
        assert!(
            serde_json::from_str::<super::BridgetMessage>(
                r#"{"id":"m","from":"a","to":"b","body":"x","thread_notice":{"version":1,"thread_id":"t","through_seq":3,"generation":2,"body":"forge"}}"#
            )
            .is_err(),
            "champ inconnu refusé dans la notice"
        );
    }

    use super::*;

    #[test]
    fn test_message_new_generates_id() {
        let msg = BridgetMessage::new("claude", "codex", "hello");
        assert!(!msg.id.is_empty());
        assert_eq!(msg.id.len(), 13);
        assert_eq!(msg.from, "claude");
        assert_eq!(msg.to, "codex");
        assert_eq!(msg.body, "hello");
        assert!(!msg.reply);
        assert_eq!(msg.hops, 4);
    }

    #[test]
    fn content_key_keeps_distinct_tracked_replies_distinct() {
        let mut first = BridgetMessage::new("codex", "alice", "même réponse");
        first.in_reply_to = Some("request-1".to_string());
        let mut second = first.clone();
        second.in_reply_to = Some("request-2".to_string());
        assert_ne!(first.content_key(), second.content_key());
    }

    #[test]
    fn test_decrement_hops() {
        let mut msg = BridgetMessage::new("a", "b", "x");
        assert!(msg.decrement_hops()); // 4 -> 3
        assert!(msg.decrement_hops()); // 3 -> 2
        assert!(msg.decrement_hops()); // 2 -> 1
        assert!(!msg.decrement_hops()); // 1 -> 0, budget épuisé
    }

    #[test]
    fn test_content_key_stable() {
        let msg1 = BridgetMessage::new("a", "codex", "hello");
        let msg2 = BridgetMessage::new("b", "codex", "hello");
        // Même destinataire + même body = même clé, même si expéditeur différent
        assert_eq!(msg1.content_key(), msg2.content_key());
    }

    #[test]
    fn test_content_key_differs_on_body() {
        let msg1 = BridgetMessage::new("a", "codex", "hello");
        let msg2 = BridgetMessage::new("a", "codex", "world");
        assert_ne!(msg1.content_key(), msg2.content_key());
    }

    #[test]
    fn test_content_key_differs_on_target() {
        let msg1 = BridgetMessage::new("a", "codex", "hello");
        let msg2 = BridgetMessage::new("a", "claude", "hello");
        assert_ne!(msg1.content_key(), msg2.content_key());
    }

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Codex.to_string(), "codex");
        assert_eq!(AgentType::Claude.to_string(), "claude");
    }

    #[test]
    fn test_message_json_roundtrip() {
        let msg = BridgetMessage::new("claude-1", "codex-1", "test message");
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: BridgetMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.id, decoded.id);
        assert_eq!(msg.from, decoded.from);
        assert_eq!(msg.body, decoded.body);
    }

    #[test]
    fn message_historique_sans_champs_de_controle_reste_acceptable() {
        let historical = r#"{"id":"legacy","from":"a","to":"b","body":"x"}"#;
        let decoded: BridgetMessage = serde_json::from_str(historical).unwrap();
        assert_eq!(decoded.origin, None);
        assert_eq!(decoded.intent, None);
        assert!(decoded.references.is_empty());
        let reencoded = serde_json::to_value(decoded).unwrap();
        assert!(reencoded.get("origin").is_none());
        assert!(reencoded.get("intent").is_none());
        assert!(reencoded.get("references").is_none());
    }

    #[test]
    fn message_controle_serialise_origine_intention_et_references() {
        let mut msg = BridgetMessage::new("humain", "agent", "travail");
        msg.origin = Some(MessageOrigin::Human);
        msg.intent = Some(MessageIntent::InterruptAndStart);
        msg.references = vec!["objective-1".to_string(), "delegation-2".to_string()];
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: BridgetMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.origin, Some(MessageOrigin::Human));
        assert_eq!(decoded.intent, Some(MessageIntent::InterruptAndStart));
        assert_eq!(decoded.references, ["objective-1", "delegation-2"]);
    }

    #[test]
    fn enums_de_controle_sont_stables_sur_le_fil() {
        assert_eq!(
            serde_json::to_string(&MessageIntent::QueueOnly).unwrap(),
            "\"queue_only\""
        );
        assert_eq!(
            serde_json::to_string(&MessageOrigin::Routine).unwrap(),
            "\"routine\""
        );
    }
}
