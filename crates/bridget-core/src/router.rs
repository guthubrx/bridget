//! Routeur - trouve le destinataire d'un message par agent_id.
//!
//! Gère l'enregistrement et le désenregistrement des agents. Une identité
//! humaine relève du profil; le routeur ne connaît que l'identifiant opaque.

use crate::message::AgentType;
use std::collections::HashMap;
use uuid::{Uuid, Version};

/// Valide un agent_id UUID v4 canonique. Les principaux non agents ne passent
/// jamais par le routeur.
pub fn validate_agent_id(agent_id: &str) -> Result<(), String> {
    let parsed = Uuid::parse_str(agent_id)
        .map_err(|_| "agent_id doit être un UUID v4 canonique".to_string())?;
    if parsed.get_version() != Some(Version::Random) || parsed.hyphenated().to_string() != agent_id
    {
        return Err("agent_id doit être un UUID v4 canonique".to_string());
    }
    Ok(())
}

/// Valide un libellé technique (type, domaine, option CLI), sans lui donner de rôle d’identité.
pub fn validate_technical_label(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 100 {
        return Err("libellé technique vide ou trop long".to_string());
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("libellé technique invalide".to_string());
    }
    Ok(())
}

/// Un agent enregistré auprès du daemon.
#[derive(Debug, Clone)]
pub struct RegisteredAgent {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub connection_id: String,
}

/// Action que le routeur demande au daemon d'effectuer.
#[derive(Debug)]
pub enum RouterAction {
    /// Livrer le message à cet agent.
    Deliver { target_conn: String },
    /// Message rejeté avec une raison.
    Reject(RouterError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    AgentNotFound(String),
    AgentAmbiguous(String, Vec<String>),
    HopsExhausted,
    SelfSend,
    InvalidAgentId(String),
    AgentIdTaken(String),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentNotFound(agent_id) => write!(formatter, "agent introuvable: {agent_id}"),
            Self::AgentAmbiguous(agent_id, connections) => {
                write!(
                    formatter,
                    "agent_id ambigu {agent_id:?}: {} connexions",
                    connections.len()
                )
            }
            Self::HopsExhausted => write!(formatter, "budget de sauts épuisé (hops=0)"),
            Self::SelfSend => write!(formatter, "auto-envoi interdit"),
            Self::InvalidAgentId(agent_id) => write!(formatter, "agent_id invalide: {agent_id}"),
            Self::AgentIdTaken(agent_id) => write!(formatter, "agent_id déjà connecté: {agent_id}"),
        }
    }
}

/// Le routeur maintient la table des agents connectés.
pub struct Router {
    /// agent_id -> détails de l'agent
    agents: HashMap<String, RegisteredAgent>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Enregistre un agent sous un agent_id préalablement créé par Bridget.
    pub fn register(
        &mut self,
        agent_id: &str,
        agent_type: &AgentType,
        connection_id: &str,
    ) -> Result<(), RouterError> {
        validate_agent_id(agent_id)
            .map_err(|_| RouterError::InvalidAgentId(agent_id.to_string()))?;
        if self.agents.contains_key(agent_id) {
            return Err(RouterError::AgentIdTaken(agent_id.to_string()));
        }
        self.agents.insert(
            agent_id.to_string(),
            RegisteredAgent {
                agent_id: agent_id.to_string(),
                agent_type: agent_type.clone(),
                connection_id: connection_id.to_string(),
            },
        );
        Ok(())
    }

    /// Désenregistre un agent par sa connexion.
    pub fn unregister_by_conn(&mut self, connection_id: &str) -> Option<RegisteredAgent> {
        let agent_id = self
            .agents
            .iter()
            .find(|(_, agent)| agent.connection_id == connection_id)
            .map(|(agent_id, _)| agent_id.clone())?;
        self.agents.remove(&agent_id)
    }

    /// Résout un message : vérifie le destinataire, les hops et l'auto-envoi.
    pub fn resolve(
        &self,
        _from_agent_id: &str,
        to_agent_id: &str,
        hops: i32,
        from_conn: &str,
    ) -> RouterAction {
        if hops <= 0 {
            return RouterAction::Reject(RouterError::HopsExhausted);
        }
        let Some(target) = self.agents.get(to_agent_id) else {
            return RouterAction::Reject(RouterError::AgentNotFound(to_agent_id.to_string()));
        };
        if target.connection_id == from_conn {
            return RouterAction::Reject(RouterError::SelfSend);
        }
        RouterAction::Deliver {
            target_conn: target.connection_id.clone(),
        }
    }

    /// Liste tous les agents enregistrés.
    pub fn list_agents(&self) -> Vec<&RegisteredAgent> {
        self.agents.values().collect()
    }

    /// Trouve un agent par agent_id.
    pub fn get_agent(&self, agent_id: &str) -> Option<&RegisteredAgent> {
        self.agents.get(agent_id)
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "550e8400-e29b-41d4-a716-446655440000";
    const B: &str = "550e8400-e29b-41d4-a716-446655440001";

    #[test]
    fn register_requires_a_canonical_v4_agent_id() {
        let mut router = Router::new();
        router
            .register(A, &AgentType::Codex, "conn-1")
            .expect("UUID v4 accepté");
        assert!(matches!(
            router.register("codex-1", &AgentType::Codex, "conn-2"),
            Err(RouterError::InvalidAgentId(_))
        ));
        assert_eq!(router.agent_count(), 1);
    }

    #[test]
    fn register_rejects_duplicate_agent_id() {
        let mut router = Router::new();
        router.register(A, &AgentType::Codex, "conn-1").unwrap();
        assert!(matches!(
            router.register(A, &AgentType::Claude, "conn-2"),
            Err(RouterError::AgentIdTaken(_))
        ));
    }

    #[test]
    fn resolve_delivers_by_agent_id() {
        let mut router = Router::new();
        router.register(A, &AgentType::Codex, "conn-1").unwrap();
        router.register(B, &AgentType::Claude, "conn-2").unwrap();

        let action = router.resolve(B, A, 3, "conn-2");
        assert!(matches!(action, RouterAction::Deliver { target_conn } if target_conn == "conn-1"));
    }

    #[test]
    fn resolve_refuses_legacy_name() {
        let router = Router::new();
        let action = router.resolve(A, "codex-1", 3, "conn-1");
        assert!(matches!(
            action,
            RouterAction::Reject(RouterError::AgentNotFound(target)) if target == "codex-1"
        ));
    }

    #[test]
    fn resolve_refuses_self_send() {
        let mut router = Router::new();
        router.register(A, &AgentType::Codex, "conn-1").unwrap();
        assert!(matches!(
            router.resolve(A, A, 3, "conn-1"),
            RouterAction::Reject(RouterError::SelfSend)
        ));
    }

    #[test]
    fn resolve_refuses_exhausted_hops() {
        let mut router = Router::new();
        router.register(A, &AgentType::Codex, "conn-1").unwrap();
        router.register(B, &AgentType::Claude, "conn-2").unwrap();
        assert!(matches!(
            router.resolve(B, A, 0, "conn-2"),
            RouterAction::Reject(RouterError::HopsExhausted)
        ));
    }

    #[test]
    fn unregister_removes_by_connection() {
        let mut router = Router::new();
        router.register(A, &AgentType::Codex, "conn-1").unwrap();
        assert!(router.unregister_by_conn("conn-1").is_some());
        assert_eq!(router.agent_count(), 0);
    }

    #[test]
    fn v4_validation_refuses_uuid_v1_and_noncanonical_values() {
        assert!(validate_agent_id(A).is_ok());
        assert!(validate_agent_id("550e8400-e29b-11d4-a716-446655440000").is_err());
        assert!(validate_agent_id("550E8400-E29B-41D4-A716-446655440000").is_err());
    }
}
