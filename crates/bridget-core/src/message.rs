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
    /// Dépôt durable uniquement : ne crée jamais de remise au fournisseur.
    StoreOnly,
}

/// Provenance bornée d'un message émis par un sous-agent interne.
///
/// Le parent reste l'expéditeur routable. `child_ref` est une empreinte opaque,
/// jamais l'identifiant de session natif du fournisseur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelegatedOrigin {
    pub provider: String,
    pub child_ref: String,
}

/// Message normalisé qui circule entre agents via le daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgetMessage {
    /// Choix volontaire interprojets. Absent conserve le contrat historique.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present_reason"
    )]
    pub cross_project_reason: Option<String>,
    /// UUID court pour déduplication et quarantaine.
    pub id: String,
    /// Identifiant opaque de l'expéditeur. Il sert exclusivement au routage.
    pub from: String,
    /// Nom de présentation de l'expéditeur, résolu par le daemon au moment de
    /// la remise. Cette valeur n'est jamais une clé de routage : le wrapper
    /// l'emploie seulement pour construire le prompt fournisseur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_display_name: Option<String>,
    /// Titre facultatif de présentation d'un fil, résolu pour son membre à la
    /// remise seulement. Sans rôle dans le routage, le journal ou le canon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_display_title: Option<String>,
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
    /// Provenance informative posée par la façade MCP déléguée. Elle ne change
    /// ni l'identité routable, ni la portée d'idempotence du parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegated_origin: Option<DelegatedOrigin>,
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
            cross_project_reason: None,
            id,
            from: from.into(),
            from_display_name: None,
            thread_display_title: None,
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
            delegated_origin: None,
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

    /// Libellé de présentation. L'identité routable reste toujours `from`.
    /// Complexité : O(1).
    pub fn sender_label(&self) -> String {
        let sender = self
            .from_display_name
            .as_deref()
            .map(str::trim)
            .filter(|display_name| !display_name.is_empty() && *display_name != self.from.as_str())
            .map_or_else(
                || self.from.clone(),
                |display_name| format!("{display_name} ({})", self.from),
            );
        match &self.delegated_origin {
            Some(origin) => format!(
                "{} (via sous-agent {} {})",
                sender, origin.provider, origin.child_ref
            ),
            None => sender,
        }
    }
}

fn deserialize_present_reason<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(de).map(Some)
}

#[cfg(test)]
#[path = "message_tests.rs"]
mod tests;
