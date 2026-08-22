//! Registre déclaratif des types d'agents lancés par Bridget.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const DEFAULT_QUEUE_CAPACITY: usize = 32;
const DEFAULT_NOTIFY_TIMEOUT_SECS: u64 = 600;
const MAX_PASS_ENV_ENTRIES: usize = 64;
const MAX_ENV_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AgentDefinition {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub forbidden_env: Vec<String>,
    /// Variables supplémentaires recopiées depuis l'environnement source du
    /// daemon après application de la garde de facturation.
    #[serde(default)]
    pub pass_env: Vec<String>,
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

fn default_protocol() -> String {
    "acp".to_string()
}
fn default_permissions() -> String {
    "allow".to_string()
}
fn default_queue_capacity() -> usize {
    DEFAULT_QUEUE_CAPACITY
}
fn default_notify_timeout_secs() -> u64 {
    DEFAULT_NOTIFY_TIMEOUT_SECS
}

impl AgentRegistry {
    pub fn load() -> Result<Self, String> {
        let source = config_path();
        let mut agents = default_agents();
        if source.exists() {
            let content = std::fs::read_to_string(&source)
                .map_err(|err| format!("impossible de lire {}: {err}", source.display()))?;
            for warning in unknown_key_warnings(&content, &source) {
                eprintln!("avertissement: {warning}");
            }
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
            format!(
                "type d'agent inconnu '{agent_type}' dans {}. Types disponibles : {available}",
                self.source.display()
            )
        })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    /// Alias des lanceurs interactifs historiques. Cette table ne vaut pas
    /// autorisation : `type_for_command` consulte toujours le registre.
    pub fn interactive_alias(command: &str) -> Option<&'static str> {
        match command {
            "codex" => Some("codex"),
            "claude" | "gclaude" | "claude-son" => Some("claude"),
            "gemini" => Some("gemini"),
            _ => None,
        }
    }

    /// Résout une commande du flux `bridget --` vers son type déclaré.
    ///
    /// Les alias interactifs historiques restent acceptés, puis les commandes
    /// du registre sont comparées par leur basename pour tolérer un chemin.
    pub fn type_for_command(&self, command: &str) -> Result<String, String> {
        let basename = command_basename(command);
        if let Some(agent_type) = Self::interactive_alias(basename) {
            self.get(agent_type)?;
            return Ok(agent_type.to_string());
        }
        let matching_types = self
            .agents
            .iter()
            .filter(|(_, definition)| command_basename(&definition.command) == basename)
            .map(|(agent_type, _)| agent_type.as_str())
            .collect::<Vec<_>>();
        if matching_types.len() == 1 {
            return Ok(matching_types[0].to_string());
        }
        if matching_types.len() > 1 {
            return Err(format!(
                "commande ambiguë '{command}' dans {} : {}. Utilisez un type explicite.",
                self.source.display(),
                matching_types.join(", ")
            ));
        }

        let available = self
            .agents
            .iter()
            .map(|(agent_type, definition)| format!("{agent_type} ({})", definition.command))
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!(
            "commande '{command}' non déclarée dans {}. Types/commandes disponibles : {available}",
            self.source.display()
        ))
    }
}

fn command_basename(command: &str) -> &str {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
}

fn unknown_key_warnings(content: &str, source: &Path) -> Vec<String> {
    const KEYS: &[&str] = &[
        "command",
        "args",
        "protocol",
        "forbidden_env",
        "pass_env",
        "permissions",
        "queue_capacity",
        "notify_timeout_secs",
    ];
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };
    let Some(agents) = value.get("agents").and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };
    let mut warnings = Vec::new();
    for (agent_type, definition) in agents {
        let Some(definition) = definition.as_object() else {
            continue;
        };
        for key in definition
            .keys()
            .filter(|key| !KEYS.contains(&key.as_str()))
        {
            warnings.push(format!(
                "clé inconnue '{key}' pour '{agent_type}' dans {}",
                source.display()
            ));
        }
    }
    warnings
}

fn config_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join(".config/bridget/agents.json")
}

fn validate_registry(
    agents: &BTreeMap<String, AgentDefinition>,
    source: &Path,
) -> Result<(), String> {
    for (name, definition) in agents {
        if definition.command.trim().is_empty() {
            return Err(format!(
                "registre invalide {}: command vide pour '{name}'",
                source.display()
            ));
        }
        if !matches!(definition.protocol.as_str(), "acp" | "tmux") {
            return Err(format!(
                "registre invalide {}: protocol invalide pour '{name}'",
                source.display()
            ));
        }
        if !matches!(definition.permissions.as_str(), "allow" | "deny") {
            return Err(format!(
                "registre invalide {}: permissions invalides pour '{name}'",
                source.display()
            ));
        }
        if definition.queue_capacity == 0 {
            return Err(format!(
                "registre invalide {}: queue_capacity nul pour '{name}'",
                source.display()
            ));
        }
        if definition.pass_env.len() > MAX_PASS_ENV_ENTRIES {
            return Err(format!(
                "registre invalide {}: pass_env dépasse {MAX_PASS_ENV_ENTRIES} entrées pour '{name}'",
                source.display()
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for variable in &definition.pass_env {
            if !valid_env_name(variable) {
                return Err(format!(
                    "registre invalide {}: variable pass_env invalide '{variable}' pour '{name}'",
                    source.display()
                ));
            }
            if !seen.insert(variable) {
                return Err(format!(
                    "registre invalide {}: variable pass_env dupliquée '{variable}' pour '{name}'",
                    source.display()
                ));
            }
            if definition.forbidden_env.contains(variable) {
                return Err(format!(
                    "registre invalide {}: '{variable}' est à la fois dans pass_env et forbidden_env pour '{name}'",
                    source.display()
                ));
            }
        }
    }
    Ok(())
}

fn valid_env_name(variable: &str) -> bool {
    let mut bytes = variable.bytes();
    variable.len() <= MAX_ENV_NAME_BYTES
        && bytes
            .next()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

pub(crate) fn allow_api_key_value(value: Option<&str>) -> bool {
    value == Some("1")
}

/// Retourne la première variable interdite présente dans la source. Cette
/// fonction est l'unique garde partagée par le wrapper 007 et le spawn 009.
pub(crate) fn forbidden_environment_variable(
    definition: &AgentDefinition,
    allow_api_key: bool,
    is_present: impl Fn(&str) -> bool,
) -> Option<String> {
    if allow_api_key {
        return None;
    }
    definition
        .forbidden_env
        .iter()
        .find(|variable| is_present(variable))
        .cloned()
}

fn definition(
    command: &str,
    args: &[&str],
    forbidden_env: &[&str],
    pass_env: &[&str],
) -> AgentDefinition {
    AgentDefinition {
        command: command.to_string(),
        args: args.iter().map(ToString::to_string).collect(),
        protocol: "acp".to_string(),
        forbidden_env: forbidden_env.iter().map(ToString::to_string).collect(),
        pass_env: pass_env.iter().map(ToString::to_string).collect(),
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
    }
}

fn default_agents() -> BTreeMap<String, AgentDefinition> {
    BTreeMap::from([
        (
            "codex".to_string(),
            definition(
                "npx",
                &[
                    "@zed-industries/codex-acp@0.16.0",
                    "-c",
                    "model=\"gpt-5.5\"",
                ],
                &["OPENAI_API_KEY", "CODEX_API_KEY"],
                &[
                    "CODEX_HOME",
                    "XDG_CONFIG_HOME",
                    "XDG_CACHE_HOME",
                    "XDG_DATA_HOME",
                    "XDG_STATE_HOME",
                    "SSH_AUTH_SOCK",
                    "NPM_CONFIG_CACHE",
                    "HTTPS_PROXY",
                    "HTTP_PROXY",
                    "NO_PROXY",
                    "SSL_CERT_FILE",
                    "SSL_CERT_DIR",
                ],
            ),
        ),
        (
            "claude".to_string(),
            definition(
                "npx",
                &["@zed-industries/claude-code-acp@0.16.2"],
                &["ANTHROPIC_API_KEY"],
                &[
                    "CLAUDE_CONFIG_DIR",
                    "XDG_CONFIG_HOME",
                    "XDG_CACHE_HOME",
                    "SSH_AUTH_SOCK",
                    "NPM_CONFIG_CACHE",
                    "HTTPS_PROXY",
                    "HTTP_PROXY",
                    "NO_PROXY",
                    "SSL_CERT_FILE",
                    "SSL_CERT_DIR",
                ],
            ),
        ),
        (
            "gemini".to_string(),
            definition(
                "gemini",
                &["--acp"],
                &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
                &["XDG_CONFIG_HOME", "XDG_CACHE_HOME", "HTTPS_PROXY", "HTTP_PROXY", "NO_PROXY"],
            ),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_the_three_priorities() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let codex = registry.get("codex").unwrap();
        assert_eq!(codex.command, "npx");
        assert_eq!(
            codex.args,
            vec![
                "@zed-industries/codex-acp@0.16.0",
                "-c",
                "model=\"gpt-5.5\""
            ]
        );
        assert_eq!(codex.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
        assert_eq!(codex.protocol, "acp");
        assert_eq!(codex.permissions, "allow");
        assert_eq!(codex.queue_capacity, 32);
        assert_eq!(codex.notify_timeout_secs, 600);
        assert!(codex.pass_env.contains(&"CODEX_HOME".to_string()));
        assert!(!codex.pass_env.contains(&"OPENAI_API_KEY".to_string()));
        assert_eq!(registry.get("gemini").unwrap().args, vec!["--acp"]);
    }

    #[test]
    fn user_entry_replaces_its_default() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"custom","protocol":"tmux"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        assert_eq!(registry.get("codex").unwrap().command, "custom");
        assert!(registry.get("codex").unwrap().forbidden_env.is_empty());
    }

    #[test]
    fn empty_command_is_rejected() {
        let result =
            AgentRegistry::from_json(r#"{"agents":{"codex":{"command":""}}}"#, "/tmp/agents.json");
        assert!(result.unwrap_err().contains("command vide"));
    }

    #[test]
    fn invalid_protocol_is_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","protocol":"bad"}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("protocol invalide"));
    }

    #[test]
    fn invalid_permissions_are_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","permissions":"ask"}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("permissions invalides"));
    }

    #[test]
    fn zero_queue_capacity_is_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","queue_capacity":0}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("queue_capacity nul"));
    }

    #[test]
    fn unknown_type_names_its_source_and_choices() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.get("inconnu").unwrap_err();
        assert!(error.contains("/tmp/agents.json"));
        assert!(error.contains("codex"));
    }

    #[test]
    fn unknown_entry_key_is_reported() {
        let warnings = unknown_key_warnings(
            r#"{"agents":{"codex":{"command":"codex","surprise":true}}}"#,
            Path::new("/tmp/agents.json"),
        );
        assert_eq!(
            warnings,
            vec!["clé inconnue 'surprise' pour 'codex' dans /tmp/agents.json"]
        );
    }

    #[test]
    fn valid_entry_keys_produce_no_warning() {
        let warnings = unknown_key_warnings(
            r#"{"agents":{"codex":{"command":"codex","args":[],"protocol":"acp","forbidden_env":[],"pass_env":["CODEX_HOME"],"permissions":"allow","queue_capacity":32,"notify_timeout_secs":600}}}"#,
            Path::new("/tmp/agents.json"),
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn pass_env_est_borne_et_valide_atomiquement() {
        for (json, detail) in [
            (
                r#"{"agents":{"x":{"command":"x","pass_env":["INVALIDE-TIRET"]}}}"#,
                "variable pass_env invalide",
            ),
            (
                r#"{"agents":{"x":{"command":"x","pass_env":["HOME","HOME"]}}}"#,
                "pass_env dupliquée",
            ),
            (
                r#"{"agents":{"x":{"command":"x","pass_env":["API_KEY"],"forbidden_env":["API_KEY"]}}}"#,
                "à la fois dans pass_env et forbidden_env",
            ),
        ] {
            let error = AgentRegistry::from_json(json, "/tmp/agents.json").unwrap_err();
            assert!(error.contains(detail), "erreur inattendue: {error}");
        }
        let entries = (0..=MAX_PASS_ENV_ENTRIES)
            .map(|index| format!("VAR_{index}"))
            .collect::<Vec<_>>();
        let json = serde_json::json!({"agents":{"x":{"command":"x","pass_env":entries}}});
        let error = AgentRegistry::from_json(&json.to_string(), "/tmp/agents.json").unwrap_err();
        assert!(error.contains("pass_env dépasse"));
    }

    #[test]
    fn generic_codex_command_resolves_through_the_registry() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        assert_eq!(registry.type_for_command("codex").unwrap(), "codex");
    }

    #[test]
    fn historic_interactive_aliases_remain_available() {
        for (command, agent_type) in [
            ("codex", "codex"),
            ("claude", "claude"),
            ("gemini", "gemini"),
            ("gclaude", "claude"),
            ("claude-son", "claude"),
        ] {
            assert_eq!(AgentRegistry::interactive_alias(command), Some(agent_type));
        }
    }

    #[test]
    fn ambiguous_command_is_refused_without_map_order_fallback() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.type_for_command("npx").unwrap_err();
        assert!(error.contains("commande ambiguë 'npx'"));
        assert!(error.contains("claude"));
        assert!(error.contains("codex"));
    }

    #[test]
    fn undeclared_generic_command_names_source_and_choices() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.type_for_command("not-declared").unwrap_err();
        assert!(error.contains("/tmp/agents.json"));
        assert!(error.contains("codex (npx)"));
    }
}
