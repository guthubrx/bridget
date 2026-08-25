//! Routeur — trouve le destinataire d'un message par nom direct.
//!
//! Gère l'enregistrement/désenregistrement des agents et la résolution
//! des noms. Conserve aussi les compteurs d'auto-incrément par type.

use crate::message::AgentType;
use std::collections::HashMap;

const MAX_AGENT_NAME_LENGTH: usize = 100;

/// Valide l'identité rendue dans les vues humaines et protocolaires.
///
/// La grammaire ASCII fermée écarte à la source les contrôles de ligne,
/// les caractères de format bidi et les différences de largeur d'affichage.
pub fn validate_agent_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("nom d'agent vide".to_string());
    }
    if name.len() > MAX_AGENT_NAME_LENGTH {
        return Err(format!(
            "nom d'agent trop long (max {MAX_AGENT_NAME_LENGTH} caractères)"
        ));
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(
            "nom d'agent contient des caractères invalides (ASCII alphanumérique, -, _ uniquement)"
                .to_string(),
        );
    }
    Ok(())
}

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
                validate_agent_name(explicit).map_err(RouterError::InvalidName)?;
                if self.agents.contains_key(explicit) {
                    return Err(RouterError::AgentNotFound(format!(
                        "nom déjà pris: {}",
                        explicit
                    )));
                }
                explicit.to_string()
            }
            None => {
                validate_agent_name(&type_str).map_err(RouterError::InvalidName)?;
                let mut next_counter = self.counters.get(&type_str).copied().unwrap_or(0);
                let candidate = loop {
                    next_counter = next_counter.checked_add(1).ok_or_else(|| {
                        RouterError::InvalidName("compteur de noms épuisé".to_string())
                    })?;
                    let candidate = format!("{}-{}", type_str, next_counter);
                    validate_agent_name(&candidate).map_err(RouterError::InvalidName)?;
                    if !self.agents.contains_key(&candidate) {
                        break candidate;
                    }
                };
                self.counters.insert(type_str, next_counter);
                candidate
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
    pub fn rename(
        &mut self,
        connection_id: &str,
        requested_name: &str,
    ) -> Result<(String, String), RouterError> {
        validate_agent_name(requested_name).map_err(RouterError::InvalidName)?;
        let name = requested_name;
        let old_name = self
            .agents
            .iter()
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
    pub fn resolve(&self, _from: &str, to: &str, hops: i32, from_conn: &str) -> RouterAction {
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
        let name = router
            .register(Some("analyse"), &AgentType::Codex, "conn-1")
            .unwrap();
        assert_eq!(name, "analyse");
    }

    fn assert_register_refuses(name: &str) {
        let mut router = Router::new();
        assert!(
            matches!(
                router.register(Some(name), &AgentType::Codex, "conn-1"),
                Err(RouterError::InvalidName(_))
            ),
            "Register ne doit jamais accepter l'identité {name:?}"
        );
        assert_eq!(router.agent_count(), 0);
    }

    fn assert_rename_refuses(name: &str) {
        let mut router = Router::new();
        router
            .register(Some("avant"), &AgentType::Codex, "conn-1")
            .unwrap();
        assert!(
            matches!(
                router.rename("conn-1", name),
                Err(RouterError::InvalidName(_))
            ),
            "Rename ne doit jamais accepter l'identité {name:?}"
        );
        assert!(router.get_agent("avant").is_some());
    }

    #[test]
    fn register_refuse_un_nom_avec_lf() {
        assert_register_refuses("relec\nadmin");
    }

    #[test]
    fn rename_refuse_un_nom_avec_lf() {
        assert_rename_refuses("relec\nadmin");
    }

    #[test]
    fn register_refuse_un_nom_vide() {
        assert_register_refuses("");
    }

    #[test]
    fn rename_refuse_un_nom_vide() {
        assert_rename_refuses("");
    }

    #[test]
    fn register_refuse_un_nom_bidi() {
        assert_register_refuses("relec\u{202e}nimda");
    }

    #[test]
    fn rename_refuse_un_nom_bidi() {
        assert_rename_refuses("relec\u{202e}nimda");
    }

    #[test]
    fn register_refuse_nul_et_non_ascii() {
        assert_register_refuses("relec\0admin");
        assert_register_refuses("分析");
    }

    #[test]
    fn register_refuse_un_nom_auto_derive_d_un_type_invalide() {
        let mut router = Router::new();
        assert!(matches!(
            router.register(None, &AgentType::Custom("rel\nadmin".to_string()), "conn-1"),
            Err(RouterError::InvalidName(_))
        ));
        assert_eq!(router.agent_count(), 0);
        assert!(
            router.counters.is_empty(),
            "un Register refusé ne doit laisser aucun compteur piloté par l'entrée"
        );
    }

    #[test]
    fn register_et_rename_refusent_un_nom_de_plus_de_cent_octets() {
        let name = "a".repeat(101);
        assert_register_refuses(&name);
        assert_rename_refuses(&name);
    }

    #[test]
    fn rename_refuse_nul_et_non_ascii() {
        assert_rename_refuses("relec\0admin");
        assert_rename_refuses("分析");
    }

    #[test]
    fn register_et_rename_acceptent_un_nom_ascii_ordinaire() {
        let mut router = Router::new();
        assert_eq!(
            router
                .register(Some("relec-6_test"), &AgentType::Codex, "conn-1")
                .unwrap(),
            "relec-6_test"
        );
        assert_eq!(
            router.rename("conn-1", "relec-6-final").unwrap(),
            ("relec-6_test".into(), "relec-6-final".into())
        );
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut router = Router::new();
        router
            .register(Some("bob"), &AgentType::Codex, "conn-1")
            .unwrap();
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
        assert!(matches!(
            action,
            RouterAction::Reject(RouterError::AgentNotFound(_))
        ));
    }

    #[test]
    fn test_resolve_self_send() {
        let mut router = Router::new();
        router.register(None, &AgentType::Codex, "conn-1").unwrap();
        let action = router.resolve("codex-1", "codex-1", 3, "conn-1");
        assert!(matches!(
            action,
            RouterAction::Reject(RouterError::SelfSend)
        ));
    }

    #[test]
    fn test_resolve_hops_exhausted() {
        let mut router = Router::new();
        router.register(None, &AgentType::Codex, "conn-1").unwrap();
        router.register(None, &AgentType::Claude, "conn-2").unwrap();
        let action = router.resolve("claude-1", "codex-1", 0, "conn-2");
        assert!(matches!(
            action,
            RouterAction::Reject(RouterError::HopsExhausted)
        ));
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
        router
            .register(Some("avant"), &AgentType::Codex, "conn-1")
            .unwrap();
        assert_eq!(
            router.rename("conn-1", "apres").unwrap(),
            ("avant".into(), "apres".into())
        );
        assert!(router.get_agent("avant").is_none());
        assert_eq!(router.get_agent("apres").unwrap().connection_id, "conn-1");
    }
}
