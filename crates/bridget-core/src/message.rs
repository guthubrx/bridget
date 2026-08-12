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

/// Message normalisé qui circule entre agents via le daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgetMessage {
    /// UUID court pour déduplication et quarantaine.
    pub id: String,
    /// Nom de l'expéditeur (ex: "claude-1").
    pub from: String,
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
            to: to.into(),
            body: body.into(),
            reply: false,
            hops: default_hops(),
            reply_timeout: None,
        }
    }

    /// Décrémente les hops. Retourne false si le budget est épuisé.
    pub fn decrement_hops(&mut self) -> bool {
        self.hops -= 1;
        self.hops > 0
    }

    /// Génère une clé de contenu pour la déduplication par contenu.
    /// Combine le destinataire + le hash du body.
    pub fn content_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.to.hash(&mut hasher);
        self.body.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
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
}
