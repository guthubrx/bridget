//! Routeur — trouve le destinataire d'un message par nom direct.
//!
//! Gère l'enregistrement/désenregistrement des agents et la résolution
//! des noms. Conserve aussi les compteurs d'auto-incrément par type.

use crate::message::AgentType;
use std::collections::HashMap;

/// Un agent enregistré auprès du daemon.
#[derive(Debug, Clone)]
pub struct RegisteredAgent {
    pub name: String,
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
    InvalidName(String),
    NameTaken(String),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::AgentNotFound(name) => write!(f, "agent introuvable: {}", name),
            RouterError::AgentAmbiguous(name, conns) => {
                write!(f, "nom ambigu '{}': {} connexions", name, conns.len())
            }
            RouterError::HopsExhausted => write!(f, "budget de sauts épuisé (hops=0)"),
            RouterError::SelfSend => write!(f, "auto-envoi interdit"),
            RouterError::InvalidName(name) => write!(f, "nom invalide: {}", name),
            RouterError::NameTaken(name) => write!(f, "nom déjà pris: {}", name),
        }
    }
}

/// Le routeur maintient la table des agents connectés.
pub struct Router {
    /// name -> détails de l'agent
    agents: HashMap<String, RegisteredAgent>,
    /// Compteurs pour l'auto-incrément : "codex" -> 2 (prochain = codex-3)
    counters: HashMap<String, u32>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            agents: HashMap::new(),
            counters: HashMap::new(),
        }
    }

    /// Génère le prochain nom disponible pour un type d'agent.
    /// Si l'utilisateur a fourni un nom explicite, on vérifie juste l'unicité.
    pub fn register(
        &mut self,
        requested_name: Option<&str>,
        agent_type: &AgentType,
        connection_id: &str,
    ) -> Result<String, RouterError> {
        let type_str = agent_type.to_string();

        let name = match requested_name {
            Some(explicit) => {
                // Vérifier l'unicité
                if self.agents.contains_key(explicit) {
                    return Err(RouterError::AgentNotFound(format!(
                        "nom déjà pris: {}",
                        explicit
                    )));
                }
                explicit.to_string()
            }
            None => {
                // Auto-incrément
                let counter = self.counters.entry(type_str.clone()).or_insert(0);
                    loop {
                        *counter += 1;
                        let candidate = format!("{}-{}", type_str, counter);
                        if !self.agents.contains_key(&candidate) {
                            break candidate;
                        }
                    }
            }
        };

        self.agents.insert(
            name.clone(),
            RegisteredAgent {
                name: name.clone(),
                agent_type: agent_type.clone(),
                connection_id: connection_id.to_string(),
            },
        );

        Ok(name)
    }

    /// Désenregistre un agent par sa connexion.
    pub fn unregister_by_conn(&mut self, connection_id: &str) -> Option<RegisteredAgent> {
        let name = self
            .agents
            .iter()
            .find(|(_, a)| a.connection_id == connection_id)
            .map(|(n, _)| n.clone())?;

        self.agents.remove(&name)
    }

    /// Remplace atomiquement le nom d'un agent identifié par sa connexion.
    pub fn rename(&mut self, connection_id: &str, requested_name: &str) -> Result<(String, String), RouterError> {
        let name = requested_name.trim();
        if name.is_empty() || name != requested_name {
            return Err(RouterError::InvalidName(requested_name.to_string()));
        }
        let old_name = self.agents.iter()
            .find(|(_, agent)| agent.connection_id == connection_id)
            .map(|(name, _)| name.clone())
            .ok_or_else(|| RouterError::AgentNotFound(connection_id.to_string()))?;
        if old_name == name {
            return Ok((old_name, name.to_string()));
        }
        if self.agents.contains_key(name) {
            return Err(RouterError::NameTaken(name.to_string()));
        }
        let mut agent = self.agents.remove(&old_name).expect("agent trouvé");
        agent.name = name.to_string();
        self.agents.insert(name.to_string(), agent);
        Ok((old_name, name.to_string()))
    }

    /// Résout un message : vérifie le destinataire, les hops, l'auto-envoi.
    /// Retourne l'action à effectuer.
    pub fn resolve(
        &self,
        _from: &str,
        to: &str,
        hops: i32,
        from_conn: &str,
    ) -> RouterAction {
        // Vérifier le budget de hops
        if hops <= 0 {
            return RouterAction::Reject(RouterError::HopsExhausted);
        }

        // Vérifier l'auto-envoi
        if let Some(agent) = self.agents.get(to) {
            if agent.connection_id == from_conn {
                return RouterAction::Reject(RouterError::SelfSend);
            }
        } else {
            return RouterAction::Reject(RouterError::AgentNotFound(to.to_string()));
        }

        let target = &self.agents[to];
        RouterAction::Deliver {
            target_conn: target.connection_id.clone(),
        }
    }

    /// Liste tous les agents enregistrés.
    pub fn list_agents(&self) -> Vec<&RegisteredAgent> {
        self.agents.values().collect()
    }

    /// Trouve un agent par nom.
    pub fn get_agent(&self, name: &str) -> Option<&RegisteredAgent> {
        self.agents.get(name)
    }

    /// Nombre d'agents connectés.
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

    #[test]
    fn test_register_auto_increment() {
        let mut router = Router::new();
        let n1 = router.register(None, &AgentType::Codex, "conn-1").unwrap();
        assert_eq!(n1, "codex-1");
        let n2 = router.register(None, &AgentType::Codex, "conn-2").unwrap();
        assert_eq!(n2, "codex-2");
        let n3 = router.register(None, &AgentType::Claude, "conn-3").unwrap();
        assert_eq!(n3, "claude-1");
    }

    #[test]
    fn test_register_explicit_name() {
        let mut router = Router::new();
        let name = router.register(Some("analyse"), &AgentType::Codex, "conn-1").unwrap();
        assert_eq!(name, "analyse");
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut router = Router::new();
        router.register(Some("bob"), &AgentType::Codex, "conn-1").unwrap();
        let result = router.register(Some("bob"), &AgentType::Codex, "conn-2");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_deliver() {
        let mut router = Router::new();
        router.register(None, &AgentType::Codex, "conn-1").unwrap();
        router.register(None, &AgentType::Claude, "conn-2").unwrap();

        let action = router.resolve("claude-1", "codex-1", 3, "conn-2");
        assert!(matches!(action, RouterAction::Deliver { target_conn } if target_conn == "conn-1"));
    }

    #[test]
    fn test_resolve_not_found() {
        let router = Router::new();
        let action = router.resolve("a", "ghost", 3, "conn-1");
        assert!(matches!(action, RouterAction::Reject(RouterError::AgentNotFound(_))));
    }

    #[test]
    fn test_resolve_self_send() {
        let mut router = Router::new();
        router.register(None, &AgentType::Codex, "conn-1").unwrap();
        let action = router.resolve("codex-1", "codex-1", 3, "conn-1");
        assert!(matches!(action, RouterAction::Reject(RouterError::SelfSend)));
    }

    #[test]
    fn test_resolve_hops_exhausted() {
        let mut router = Router::new();
        router.register(None, &AgentType::Codex, "conn-1").unwrap();
        router.register(None, &AgentType::Claude, "conn-2").unwrap();
        let action = router.resolve("claude-1", "codex-1", 0, "conn-2");
        assert!(matches!(action, RouterAction::Reject(RouterError::HopsExhausted)));
    }

    #[test]
    fn test_unregister() {
        let mut router = Router::new();
        router.register(None, &AgentType::Codex, "conn-1").unwrap();
        assert_eq!(router.agent_count(), 1);
        let removed = router.unregister_by_conn("conn-1");
        assert!(removed.is_some());
        assert_eq!(router.agent_count(), 0);
    }

    #[test]
    fn test_rename_replaces_the_lookup_key() {
        let mut router = Router::new();
        router.register(Some("avant"), &AgentType::Codex, "conn-1").unwrap();
        assert_eq!(router.rename("conn-1", "apres").unwrap(), ("avant".into(), "apres".into()));
        assert!(router.get_agent("avant").is_none());
        assert_eq!(router.get_agent("apres").unwrap().connection_id, "conn-1");
    }
}
