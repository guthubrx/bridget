//! Registre déclaratif des types d'agents lancés par Bridget.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const DEFAULT_QUEUE_CAPACITY: usize = 32;
const DEFAULT_NOTIFY_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AgentDefinition {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub forbidden_env: Vec<String>,
    #[serde(default = "default_permissions")]
    pub permissions: String,
    #[serde(default = "default_queue_capacity")]
    pub queue_capacity: usize,
    #[serde(default = "default_notify_timeout_secs")]
    pub notify_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AgentRegistryFile {
    #[serde(default)]
    pub agents: BTreeMap<String, AgentDefinition>,
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: BTreeMap<String, AgentDefinition>,
    source: PathBuf,
}

fn default_protocol() -> String { "acp".to_string() }
fn default_permissions() -> String { "allow".to_string() }
fn default_queue_capacity() -> usize { DEFAULT_QUEUE_CAPACITY }
fn default_notify_timeout_secs() -> u64 { DEFAULT_NOTIFY_TIMEOUT_SECS }

impl AgentRegistry {
    pub fn load() -> Result<Self, String> {
        let source = config_path();
        let mut agents = default_agents();
        if source.exists() {
            let content = std::fs::read_to_string(&source)
                .map_err(|err| format!("impossible de lire {}: {err}", source.display()))?;
            warn_unknown_keys(&content, &source);
            let user: AgentRegistryFile = serde_json::from_str(&content)
                .map_err(|err| format!("registre invalide {}: {err}", source.display()))?;
            validate_registry(&user.agents, &source)?;
            agents.extend(user.agents);
        }
        Ok(Self { agents, source })
    }

    pub fn from_json(content: &str, source: impl Into<PathBuf>) -> Result<Self, String> {
        let source = source.into();
        let user: AgentRegistryFile = serde_json::from_str(content)
            .map_err(|err| format!("registre invalide {}: {err}", source.display()))?;
        validate_registry(&user.agents, &source)?;
        let mut agents = default_agents();
        agents.extend(user.agents);
        Ok(Self { agents, source })
    }

    pub fn get(&self, agent_type: &str) -> Result<&AgentDefinition, String> {
        self.agents.get(agent_type).ok_or_else(|| {
            let available = self.agents.keys().cloned().collect::<Vec<_>>().join(", ");
            format!("type d'agent inconnu '{agent_type}' dans {}. Types disponibles : {available}", self.source.display())
        })
    }

    pub fn launcher_type(command: &str) -> Option<&'static str> {
        match command {
            "codex" => Some("codex"),
            "claude" | "gclaude" => Some("claude"),
            "gemini" => Some("gemini"),
            _ => None,
        }
    }
}

fn warn_unknown_keys(content: &str, source: &Path) {
    const KEYS: &[&str] = &[
        "command",
        "args",
        "protocol",
        "forbidden_env",
        "permissions",
        "queue_capacity",
        "notify_timeout_secs",
    ];
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return;
    };
    let Some(agents) = value.get("agents").and_then(serde_json::Value::as_object) else {
        return;
    };
    for (agent_type, definition) in agents {
        let Some(definition) = definition.as_object() else {
            continue;
        };
        for key in definition.keys().filter(|key| !KEYS.contains(&key.as_str())) {
            eprintln!("avertissement: clé inconnue '{key}' pour '{agent_type}' dans {}", source.display());
        }
    }
}

fn config_path() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp")).join(".config/bridget/agents.json")
}

fn validate_registry(agents: &BTreeMap<String, AgentDefinition>, source: &Path) -> Result<(), String> {
    for (name, definition) in agents {
        if definition.command.trim().is_empty() {
            return Err(format!("registre invalide {}: command vide pour '{name}'", source.display()));
        }
        if !matches!(definition.protocol.as_str(), "acp" | "tmux") {
            return Err(format!("registre invalide {}: protocol invalide pour '{name}'", source.display()));
        }
        if !matches!(definition.permissions.as_str(), "allow" | "deny") {
            return Err(format!("registre invalide {}: permissions invalides pour '{name}'", source.display()));
        }
        if definition.queue_capacity == 0 {
            return Err(format!("registre invalide {}: queue_capacity nul pour '{name}'", source.display()));
        }
    }
    Ok(())
}

fn definition(command: &str, args: &[&str], forbidden_env: &[&str]) -> AgentDefinition {
    AgentDefinition {
        command: command.to_string(),
        args: args.iter().map(ToString::to_string).collect(),
        protocol: "acp".to_string(),
        forbidden_env: forbidden_env.iter().map(ToString::to_string).collect(),
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
    }
}

fn default_agents() -> BTreeMap<String, AgentDefinition> {
    BTreeMap::from([
        ("codex".to_string(), definition("npx", &["@zed-industries/codex-acp@0.16.0", "-c", "model=\"gpt-5.5\""], &["OPENAI_API_KEY", "CODEX_API_KEY"])),
        ("claude".to_string(), definition("npx", &["@zed-industries/claude-code-acp@0.16.2"], &["ANTHROPIC_API_KEY"])),
        ("gemini".to_string(), definition("gemini", &["--acp"], &["GEMINI_API_KEY", "GOOGLE_API_KEY"])),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_the_three_priorities() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        assert_eq!(registry.get("codex").unwrap().command, "npx");
        assert_eq!(registry.get("codex").unwrap().queue_capacity, 32);
        assert_eq!(registry.get("gemini").unwrap().args, vec!["--acp"]);
    }

    #[test]
    fn user_entry_replaces_its_default() {
        let registry = AgentRegistry::from_json(r#"{"agents":{"codex":{"command":"custom","protocol":"tmux"}}}"#, "/tmp/agents.json").unwrap();
        assert_eq!(registry.get("codex").unwrap().command, "custom");
        assert!(registry.get("codex").unwrap().forbidden_env.is_empty());
    }

    #[test]
    fn invalid_entry_is_rejected() {
        let result = AgentRegistry::from_json(r#"{"agents":{"codex":{"command":"","protocol":"bad"}}}"#, "/tmp/agents.json");
        assert!(result.unwrap_err().contains("command vide"));
    }

    #[test]
    fn unknown_type_names_its_source_and_choices() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.get("inconnu").unwrap_err();
        assert!(error.contains("/tmp/agents.json"));
        assert!(error.contains("codex"));
    }
}
