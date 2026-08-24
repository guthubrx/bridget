//! Registre déclaratif des types d'agents lancés par Bridget.

use bridget_transport::{
    AdapterCapabilities, ModelCapabilities, ResolvedAgentDefinition, ResolvedMcpDefinition,
    SpawnRefusal,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const DEFAULT_QUEUE_CAPACITY: usize = 32;
const DEFAULT_NOTIFY_TIMEOUT_SECS: u64 = 600;
// launchd démarre le daemon avec un PATH minimal : les pilotes embarqués ne
// doivent pas dépendre de la configuration interactive de l'utilisateur.
const NATIVE_CODEX_COMMAND: &str = "/opt/homebrew/bin/codex";
/// Chemin absolu figé du CLI Claude (launchd / PATH minimal).
pub const NATIVE_CLAUDE_COMMAND: &str = "/Users/moi/.local/bin/claude";
const MAX_PASS_ENV_ENTRIES: usize = 64;
const MAX_ENV_NAME_BYTES: usize = 128;
const MAX_CAPABILITY_VALUE_CHARS: usize = 100;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct McpDefinition {
    #[serde(default = "default_interactive_mcp")]
    pub interactive: String,
    #[serde(default)]
    pub acp_session: bool,
}

impl Default for McpDefinition {
    fn default() -> Self {
        Self {
            interactive: default_interactive_mcp(),
            acp_session: false,
        }
    }
}

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
    /// Branchement MCP éphémère, déclaré par type et jamais par une
    /// configuration utilisateur persistante.
    #[serde(default)]
    pub mcp: McpDefinition,
    /// Matrice déclarative qui autorise un lancement sans interroger le
    /// pilote. Son contenu est figé avec le reste de la définition résolue.
    #[serde(default = "default_capabilities")]
    pub capabilities: AdapterCapabilities,
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
fn default_interactive_mcp() -> String {
    "none".to_string()
}

fn default_capabilities() -> AdapterCapabilities {
    // Compatibilité des registres historiques : ACP était déjà le seul chemin
    // réellement lancé par ce daemon. Un modèle explicite, lui, reste refusé
    // tant qu'il n'est pas déclaré dans la matrice.
    AdapterCapabilities::default()
}

impl AgentRegistry {
    pub fn load() -> Result<Self, String> {
        Self::load_from_path(config_path())
    }

    fn load_from_path(source: PathBuf) -> Result<Self, String> {
        let mut agents = default_agents();
        match std::fs::symlink_metadata(&source) {
            Ok(_) => {
                let content = read_private_registry(&source)?;
                for warning in registry_warnings(&content, &source, &agents) {
                    eprintln!("avertissement: {warning}");
                }
                let user: AgentRegistryFile = serde_json::from_str(&content)
                    .map_err(|err| format!("registre invalide {}: {err}", source.display()))?;
                validate_registry(&user.agents, &source)?;
                agents.extend(user.agents);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "impossible d'inspecter {}: {error}",
                    source.display()
                ));
            }
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

    pub fn resolved_definition(&self, agent_type: &str) -> Result<ResolvedAgentDefinition, String> {
        resolved_definition(self.get(agent_type)?)
    }

    /// Reconstruit un registre à une seule entrée depuis la définition figée
    /// par la saga. Le digest est revérifié avant tout lancement : le wrapper
    /// géré ne consulte donc jamais le registre utilisateur courant.
    pub fn from_resolved(
        agent_type: &str,
        resolved: &ResolvedAgentDefinition,
    ) -> Result<Self, String> {
        let mut definition = AgentDefinition {
            command: resolved.command.clone(),
            args: resolved.args.clone(),
            protocol: resolved.protocol.clone(),
            forbidden_env: resolved.forbidden_env.clone(),
            pass_env: resolved.pass_env.clone(),
            permissions: resolved.permissions.clone(),
            queue_capacity: resolved.queue_capacity,
            notify_timeout_secs: resolved.notify_timeout_secs,
            mcp: McpDefinition {
                interactive: resolved.mcp.interactive.clone(),
                acp_session: resolved.mcp.acp_session,
            },
            capabilities: resolved.capabilities.clone(),
        };
        let expected = resolved_definition(&definition)?;
        let legacy_digest = expected.digest != resolved.digest
            && legacy_resolved_digest(&definition)? == resolved.digest;
        if expected.digest != resolved.digest && !legacy_digest {
            return Err("digest de la définition figée invalide".to_string());
        }
        if legacy_digest {
            definition.capabilities = legacy_capabilities_for(&definition);
        }
        let source = PathBuf::from("<définition-figée>");
        let agents = BTreeMap::from([(agent_type.to_string(), definition)]);
        validate_registry(&agents, &source)?;
        Ok(Self { agents, source })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    /// Instantané ordonné des types effectivement chargés par le daemon.
    pub fn known_types(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
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

fn legacy_capabilities_for(definition: &AgentDefinition) -> AdapterCapabilities {
    let mut capabilities = AdapterCapabilities::default();
    if let Some((model, effort)) = runtime_model_and_effort(&definition.args) {
        capabilities.models.insert(
            model,
            ModelCapabilities {
                efforts: effort.into_iter().collect(),
            },
        );
    }
    capabilities
}

/// Paquets npm `@zed-industries/{codex,claude-code}-acp` : pont tiers figé
/// retiré en G10 (ADR 010). L'ACP générique (`cursor-agent acp`, `gemini
/// --acp`, fixtures ACP) reste autorisé.
///
/// Normalise avant comparaison : basename du chemin + retrait du suffixe
/// npm `@version`, pour couvrir aussi les chemins absolus vivants du projet
/// (`/opt/homebrew/bin/codex-acp`, `./node_modules/.bin/…`, `codex-acp@0.16.0`).
fn normalize_bridge_token(token: &str) -> String {
    let lowered = token.to_ascii_lowercase();
    let base = Path::new(&lowered)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(lowered.as_str());
    match base.rfind('@').filter(|&index| index > 0) {
        Some(index) => base[..index].to_string(),
        None => base.to_string(),
    }
}

fn zed_bridge_token(token: &str) -> Option<&'static str> {
    let normalized = normalize_bridge_token(token);
    let lowered = token.to_ascii_lowercase();
    if normalized == "codex-acp" || lowered.contains("@zed-industries/codex-acp") {
        Some("@zed-industries/codex-acp")
    } else if normalized == "claude-code-acp"
        || normalized == "claude-acp"
        || lowered.contains("@zed-industries/claude-code-acp")
    {
        Some("@zed-industries/claude-code-acp")
    } else {
        None
    }
}

/// Refuse un lancement qui ciblerait encore le pont Zed pour Codex ou Claude.
/// Ne touche pas le protocole ACP lui-même : seuls les paquets tiers figés
/// sont exclus.
pub(crate) fn reject_retired_zed_bridge(definition: &AgentDefinition) -> Result<(), SpawnRefusal> {
    let mut tokens = vec![definition.command.as_str()];
    tokens.extend(definition.args.iter().map(String::as_str));
    for token in tokens {
        if let Some(package) = zed_bridge_token(token) {
            return Err(SpawnRefusal::EnvUnfit {
                detail: format!(
                    "pont Zed retiré (G10, 2026-08-24) : le paquet {package} n'est plus un chemin de lancement ; utiliser le pilote natif (codex app-server / claude_stream_json) — ACP reste pour cursor et gemini"
                ),
            });
        }
    }
    Ok(())
}

/// Vérifie une demande de lancement contre la matrice persistable du registre.
/// Aucune sonde du pilote n'est exécutée : le même ordre rejoué conserve donc
/// la même décision après un crash ou une évolution externe.
pub(crate) fn validate_launch_capabilities(
    agent_type: &str,
    definition: &AgentDefinition,
) -> Result<(), SpawnRefusal> {
    reject_retired_zed_bridge(definition)?;
    let (model, effort) = runtime_labels(&definition.args);
    let model_label = model.clone().unwrap_or_else(|| "<non déclaré>".to_string());
    if !definition
        .capabilities
        .execution_paths
        .iter()
        .any(|path| path == &definition.protocol)
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model: model_label,
            capability: format!("chemin d'exécution '{}'", definition.protocol),
        });
    }
    if model
        .as_deref()
        .is_some_and(|value| !valid_capability_value(value))
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model: model_label,
            capability: "étiquette de modèle valide".to_string(),
        });
    }
    if effort
        .as_deref()
        .is_some_and(|value| !valid_capability_value(value))
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model: model_label,
            capability: "étiquette d'effort valide".to_string(),
        });
    }
    let Some(model) = model else {
        if effort.is_some() {
            return Err(SpawnRefusal::UnsupportedCapability {
                agent_type: agent_type.to_string(),
                model: model_label,
                capability: "modèle explicite requis pour l'effort".to_string(),
            });
        }
        return Ok(());
    };
    let Some(model_capabilities) = definition.capabilities.models.get(&model) else {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model,
            capability: "modèle pris en charge par l'adaptateur".to_string(),
        });
    };
    if let Some(effort) = effort
        && !model_capabilities
            .efforts
            .iter()
            .any(|declared| declared == &effort)
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model,
            capability: format!("effort '{effort}' accepté par le modèle"),
        });
    }
    Ok(())
}

/// Extrait les étiquettes opaques du lancement sans les normaliser ni les
/// substituer. La projection de l'annuaire et la garde de lancement partagent
/// ainsi exactement la même lecture des arguments.
pub(crate) fn runtime_model_and_effort(args: &[String]) -> Option<(String, Option<String>)> {
    let (model, effort) = runtime_labels(args);
    let model = model.filter(|value| valid_capability_value(value))?;
    let effort = effort.filter(|value| valid_capability_value(value));
    Some((model, effort))
}

fn runtime_labels(args: &[String]) -> (Option<String>, Option<String>) {
    let mut model = None;
    let mut effort = None;
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if matches!(argument.as_str(), "--model" | "--effort" | "--effort-level") {
            if let Some(value) = args.get(index + 1) {
                let value = unquote_runtime_value(value);
                if argument == "--model" {
                    model = Some(value);
                } else {
                    effort = Some(value);
                }
            }
            index += 2;
            continue;
        }
        if let Some(value) = runtime_assignment(argument, "model") {
            model = Some(unquote_runtime_value(value));
        }
        if let Some(value) = runtime_assignment(argument, "model_reasoning_effort")
            .or_else(|| runtime_assignment(argument, "effort"))
        {
            effort = Some(unquote_runtime_value(value));
        }
        index += 1;
    }
    (model, effort)
}

fn runtime_assignment<'a>(argument: &'a str, key: &str) -> Option<&'a str> {
    argument
        .strip_prefix(key)
        .and_then(|suffix| suffix.strip_prefix('='))
}

fn unquote_runtime_value(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn valid_capability_value(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_CAPABILITY_VALUE_CHARS
        && !value.chars().any(char::is_control)
}

fn command_basename(command: &str) -> &str {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
}

fn registry_warnings(
    content: &str,
    source: &Path,
    defaults: &BTreeMap<String, AgentDefinition>,
) -> Vec<String> {
    const KEYS: &[&str] = &[
        "command",
        "args",
        "protocol",
        "forbidden_env",
        "pass_env",
        "permissions",
        "queue_capacity",
        "notify_timeout_secs",
        "mcp",
        "capabilities",
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
        if !definition.contains_key("forbidden_env")
            && defaults
                .get(agent_type)
                .is_some_and(|default| !default.forbidden_env.is_empty())
        {
            warnings.push(format!(
                "'{agent_type}' remplace un défaut protégé sans déclarer forbidden_env dans {}",
                source.display()
            ));
        }
    }
    warnings
}

fn read_private_registry(source: &Path) -> Result<String, String> {
    let link_metadata = std::fs::symlink_metadata(source)
        .map_err(|err| format!("impossible d'inspecter {}: {err}", source.display()))?;
    if link_metadata.file_type().is_symlink() {
        return Err(format!(
            "registre refusé {}: les liens symboliques sont interdits",
            source.display()
        ));
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(source)
        .map_err(|err| {
            format!(
                "impossible d'ouvrir {} sans suivre de lien: {err}",
                source.display()
            )
        })?;
    let metadata = file
        .metadata()
        .map_err(|err| format!("impossible d'inspecter {}: {err}", source.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "registre refusé {}: fichier régulier requis",
            source.display()
        ));
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(format!(
            "registre refusé {}: propriétaire inattendu",
            source.display()
        ));
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(format!(
            "registre refusé {}: permissions {mode:04o}, attendu 0600 ou plus restrictif",
            source.display()
        ));
    }
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content)
        .map_err(|err| format!("impossible de lire {}: {err}", source.display()))?;
    Ok(content)
}

#[derive(Serialize)]
struct CanonicalResolvedDefinition<'a> {
    command: &'a str,
    args: &'a [String],
    protocol: &'a str,
    forbidden_env: &'a [String],
    pass_env: &'a [String],
    permissions: &'a str,
    queue_capacity: usize,
    notify_timeout_secs: u64,
    mcp: CanonicalResolvedMcpDefinition<'a>,
    capabilities: &'a AdapterCapabilities,
}

#[derive(Serialize)]
struct CanonicalResolvedMcpDefinition<'a> {
    interactive: &'a str,
    acp_session: bool,
}

/// Forme de digest publiée avant la matrice L1. Elle n'est acceptée qu'à la
/// lecture d'une génération déjà persistée, puis la prochaine transition
/// réécrit la définition complète avec les capacités déclaratives.
#[derive(Serialize)]
struct LegacyCanonicalResolvedDefinition<'a> {
    command: &'a str,
    args: &'a [String],
    protocol: &'a str,
    forbidden_env: &'a [String],
    pass_env: &'a [String],
    permissions: &'a str,
    queue_capacity: usize,
    notify_timeout_secs: u64,
    mcp: CanonicalResolvedMcpDefinition<'a>,
}

fn legacy_resolved_digest(definition: &AgentDefinition) -> Result<String, String> {
    let canonical = LegacyCanonicalResolvedDefinition {
        command: &definition.command,
        args: &definition.args,
        protocol: &definition.protocol,
        forbidden_env: &definition.forbidden_env,
        pass_env: &definition.pass_env,
        permissions: &definition.permissions,
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: CanonicalResolvedMcpDefinition {
            interactive: &definition.mcp.interactive,
            acp_session: definition.mcp.acp_session,
        },
    };
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| format!("définition historique impossible à sérialiser: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn resolved_definition(definition: &AgentDefinition) -> Result<ResolvedAgentDefinition, String> {
    let canonical = CanonicalResolvedDefinition {
        command: &definition.command,
        args: &definition.args,
        protocol: &definition.protocol,
        forbidden_env: &definition.forbidden_env,
        pass_env: &definition.pass_env,
        permissions: &definition.permissions,
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: CanonicalResolvedMcpDefinition {
            interactive: &definition.mcp.interactive,
            acp_session: definition.mcp.acp_session,
        },
        capabilities: &definition.capabilities,
    };
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| format!("définition résolue impossible à sérialiser: {err}"))?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    Ok(ResolvedAgentDefinition {
        command: definition.command.clone(),
        args: definition.args.clone(),
        protocol: definition.protocol.clone(),
        forbidden_env: definition.forbidden_env.clone(),
        pass_env: definition.pass_env.clone(),
        permissions: definition.permissions.clone(),
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: ResolvedMcpDefinition {
            interactive: definition.mcp.interactive.clone(),
            acp_session: definition.mcp.acp_session,
        },
        capabilities: definition.capabilities.clone(),
        digest,
    })
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
        if !matches!(
            definition.protocol.as_str(),
            "acp" | "claude_stream_json" | "codex_app_server" | "tmux"
        ) {
            return Err(format!(
                "registre invalide {}: protocol invalide pour '{name}'",
                source.display()
            ));
        }
        validate_capabilities(name, &definition.capabilities, source)?;
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
        if !matches!(
            definition.mcp.interactive.as_str(),
            "none" | "claude" | "codex" | "unsupported"
        ) {
            return Err(format!(
                "registre invalide {}: mcp.interactive invalide pour '{name}'",
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

fn validate_capabilities(
    name: &str,
    capabilities: &AdapterCapabilities,
    source: &Path,
) -> Result<(), String> {
    let mut paths = std::collections::HashSet::new();
    for path in &capabilities.execution_paths {
        if !valid_capability_value(path) {
            return Err(format!(
                "registre invalide {}: chemin de capacité invalide pour '{name}'",
                source.display()
            ));
        }
        if !paths.insert(path) {
            return Err(format!(
                "registre invalide {}: chemin de capacité dupliqué '{path}' pour '{name}'",
                source.display()
            ));
        }
    }
    for (model, model_capabilities) in &capabilities.models {
        if !valid_capability_value(model) {
            return Err(format!(
                "registre invalide {}: modèle de capacité invalide pour '{name}'",
                source.display()
            ));
        }
        let mut efforts = std::collections::HashSet::new();
        for effort in &model_capabilities.efforts {
            if !valid_capability_value(effort) {
                return Err(format!(
                    "registre invalide {}: effort de capacité invalide pour '{name}'",
                    source.display()
                ));
            }
            if !efforts.insert(effort) {
                return Err(format!(
                    "registre invalide {}: effort de capacité dupliqué '{effort}' pour '{name}'",
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
    mcp_interactive: &str,
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
        mcp: McpDefinition {
            interactive: mcp_interactive.to_string(),
            acp_session: true,
        },
        capabilities: default_capabilities(),
    }
}

fn native_claude_definition() -> AgentDefinition {
    AgentDefinition {
        command: NATIVE_CLAUDE_COMMAND.to_string(),
        // Même contrat que le wrapper interactif (wrapper.rs) : sans ces
        // flags le flux stream-json émet des demandes d'outil auxquelles le
        // pilote géré ne répond pas — tours clos, zéro outil.
        args: vec![
            "--model".to_string(),
            "claude-opus-5".to_string(),
            "--dangerously-skip-permissions".to_string(),
            "--permission-mode".to_string(),
            "bypassPermissions".to_string(),
        ],
        protocol: "claude_stream_json".to_string(),
        forbidden_env: vec!["ANTHROPIC_API_KEY".to_string()],
        pass_env: [
            "CLAUDE_CONFIG_DIR",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "SSH_AUTH_SOCK",
            "HTTPS_PROXY",
            "HTTP_PROXY",
            "NO_PROXY",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: "claude".to_string(),
            acp_session: false,
        },
        capabilities: AdapterCapabilities {
            execution_paths: vec!["claude_stream_json".to_string()],
            models: BTreeMap::from([("claude-opus-5".to_string(), ModelCapabilities::default())]),
        },
    }
}

fn native_codex_definition() -> AgentDefinition {
    AgentDefinition {
        command: NATIVE_CODEX_COMMAND.to_string(),
        args: vec![
            "-c".to_string(),
            "model=\"gpt-5.6-terra\"".to_string(),
            "app-server".to_string(),
        ],
        protocol: "codex_app_server".to_string(),
        forbidden_env: vec!["OPENAI_API_KEY".to_string(), "CODEX_API_KEY".to_string()],
        pass_env: [
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
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: "codex".to_string(),
            acp_session: false,
        },
        capabilities: AdapterCapabilities {
            execution_paths: vec!["codex_app_server".to_string()],
            models: BTreeMap::from([("gpt-5.6-terra".to_string(), ModelCapabilities::default())]),
        },
    }
}

fn native_cursor_definition() -> AgentDefinition {
    // Le CLI Cursor choisit le modèle en interne : on affiche l'étiquette
    // honnête `auto`, jamais un nom inventé. Sans `--model`, who reste vide.
    AgentDefinition {
        command: "cursor-agent".to_string(),
        args: vec![
            "--model".to_string(),
            "auto".to_string(),
            "acp".to_string(),
        ],
        protocol: "acp".to_string(),
        forbidden_env: vec![
            "CURSOR_API_KEY".to_string(),
            "OPENAI_API_KEY".to_string(),
            "ANTHROPIC_API_KEY".to_string(),
        ],
        pass_env: Vec::new(),
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: "none".to_string(),
            acp_session: true,
        },
        capabilities: AdapterCapabilities {
            execution_paths: vec!["acp".to_string()],
            models: BTreeMap::from([("auto".to_string(), ModelCapabilities::default())]),
        },
    }
}

fn default_agents() -> BTreeMap<String, AgentDefinition> {
    BTreeMap::from([
        ("codex".to_string(), native_codex_definition()),
        ("claude".to_string(), native_claude_definition()),
        ("cursor".to_string(), native_cursor_definition()),
        (
            "gemini".to_string(),
            definition(
                "gemini",
                &["--acp"],
                &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
                &[
                    "XDG_CONFIG_HOME",
                    "XDG_CACHE_HOME",
                    "HTTPS_PROXY",
                    "HTTP_PROXY",
                    "NO_PROXY",
                ],
                "unsupported",
            ),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn test_root(label: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("bridget-registry-{label}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_registry(path: &Path, mode: u32) {
        std::fs::write(path, "{\"agents\":{}}").unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[test]
    fn defaults_cover_the_three_priorities() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let codex = registry.get("codex").unwrap();
        assert_eq!(codex.command, NATIVE_CODEX_COMMAND);
        assert!(Path::new(&codex.command).is_absolute());
        assert_eq!(
            codex.args,
            vec!["-c", "model=\"gpt-5.6-terra\"", "app-server"]
        );
        assert_eq!(codex.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
        assert_eq!(codex.protocol, "codex_app_server");
        assert_eq!(codex.permissions, "allow");
        assert_eq!(codex.queue_capacity, 32);
        assert_eq!(codex.notify_timeout_secs, 600);
        assert!(codex.pass_env.contains(&"CODEX_HOME".to_string()));
        assert!(!codex.pass_env.contains(&"OPENAI_API_KEY".to_string()));
        assert_eq!(registry.get("gemini").unwrap().args, vec!["--acp"]);
        assert_eq!(codex.mcp.interactive, "codex");
        assert!(!codex.mcp.acp_session);
        assert_eq!(codex.capabilities.execution_paths, vec!["codex_app_server"]);
        assert_eq!(
            codex.capabilities.models.get("gpt-5.6-terra"),
            Some(&ModelCapabilities::default())
        );
        let claude = registry.get("claude").unwrap();
        assert_eq!(claude.command, NATIVE_CLAUDE_COMMAND);
        assert!(Path::new(&claude.command).is_absolute());
        assert_eq!(claude.protocol, "claude_stream_json");
        assert_eq!(
            claude.args,
            [
                "--model",
                "claude-opus-5",
                "--dangerously-skip-permissions",
                "--permission-mode",
                "bypassPermissions",
            ]
        );
        assert!(!claude.mcp.acp_session);
        assert_eq!(
            claude.capabilities.execution_paths,
            vec!["claude_stream_json"]
        );
        assert_eq!(
            claude.capabilities.models.get("claude-opus-5"),
            Some(&ModelCapabilities::default())
        );
        assert_eq!(
            registry.get("gemini").unwrap().mcp.interactive,
            "unsupported"
        );
        let cursor = registry.get("cursor").unwrap();
        assert_eq!(cursor.command, "cursor-agent");
        assert_eq!(cursor.args, vec!["--model", "auto", "acp"]);
        assert_eq!(cursor.protocol, "acp");
        assert_eq!(
            cursor.capabilities.models.get("auto"),
            Some(&ModelCapabilities::default())
        );
        assert_eq!(
            runtime_model_and_effort(&cursor.args),
            Some(("auto".to_string(), None))
        );
    }

    #[test]
    fn etiquette_modele_auto_lue_depuis_les_args_cursor() {
        // Sans --model, who reste vide — c'est le trou constaté sur les
        // anciennes définitions. Avec --model auto, l'étiquette est honnête.
        let observed = runtime_model_and_effort(&[
            "--model".to_string(),
            "auto".to_string(),
            "acp".to_string(),
        ]);
        assert_eq!(observed, Some(("auto".to_string(), None)));
        assert!(
            runtime_model_and_effort(&["acp".to_string()]).is_none(),
            "sans --model, who reste vide"
        );
    }

    #[test]
    fn bypass_permissions_change_le_digest_de_la_definition_claude() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let with_bypass = registry.resolved_definition("claude").unwrap();
        let mut without = registry.get("claude").unwrap().clone();
        without.args = vec!["--model".to_string(), "claude-opus-5".to_string()];
        let old = resolved_definition(&without).unwrap();
        assert_ne!(
            with_bypass.digest, old.digest,
            "ajouter les flags bypass doit changer le digest figé"
        );
        assert!(
            with_bypass
                .args
                .iter()
                .any(|a| a == "--dangerously-skip-permissions")
        );
        assert!(with_bypass.args.iter().any(|a| a == "bypassPermissions"));
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
    fn remplacement_sans_forbidden_env_signale_la_perte_de_garde() {
        let warnings = registry_warnings(
            r#"{"agents":{"codex":{"command":"custom"}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents(),
        );
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("codex"));
        assert!(warnings[0].contains("sans déclarer forbidden_env"));

        let explicit = registry_warnings(
            r#"{"agents":{"codex":{"command":"custom","forbidden_env":[]}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents(),
        );
        assert!(explicit.is_empty());
    }

    #[test]
    fn definition_resolue_est_complete_et_son_digest_est_deterministe() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let first = registry.resolved_definition("codex").unwrap();
        let second = registry.resolved_definition("codex").unwrap();
        assert_eq!(first, second);
        assert_eq!(first.command, NATIVE_CODEX_COMMAND);
        assert_eq!(
            first.args,
            vec!["-c", "model=\"gpt-5.6-terra\"", "app-server"]
        );
        assert_eq!(first.protocol, "codex_app_server");
        assert_eq!(first.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
        assert!(first.pass_env.contains(&"CODEX_HOME".to_string()));
        assert_eq!(first.permissions, "allow");
        assert_eq!(first.queue_capacity, 32);
        assert_eq!(first.notify_timeout_secs, 600);
        assert_eq!(first.mcp.interactive, "codex");
        assert!(!first.mcp.acp_session);
        assert_eq!(
            first.capabilities.models.get("gpt-5.6-terra"),
            Some(&ModelCapabilities::default())
        );
        assert_eq!(first.digest.len(), 64);

        let baseline = default_agents().remove("codex").unwrap();
        let mut mutations = Vec::new();
        let mut changed = baseline.clone();
        changed.command = "other".to_string();
        mutations.push(("command", changed));
        let mut changed = baseline.clone();
        changed.args.push("other".to_string());
        mutations.push(("args", changed));
        let mut changed = baseline.clone();
        changed.protocol = "tmux".to_string();
        mutations.push(("protocol", changed));
        let mut changed = baseline.clone();
        changed.forbidden_env.push("OTHER_KEY".to_string());
        mutations.push(("forbidden_env", changed));
        let mut changed = baseline.clone();
        changed.pass_env.push("OTHER_HOME".to_string());
        mutations.push(("pass_env", changed));
        let mut changed = baseline.clone();
        changed.permissions = "deny".to_string();
        mutations.push(("permissions", changed));
        let mut changed = baseline.clone();
        changed.queue_capacity += 1;
        mutations.push(("queue_capacity", changed));
        let mut changed = baseline.clone();
        changed.notify_timeout_secs += 1;
        mutations.push(("notify_timeout_secs", changed));
        let mut changed = baseline.clone();
        changed.mcp.interactive = "claude".to_string();
        mutations.push(("mcp.interactive", changed));
        let mut changed = baseline.clone();
        changed.mcp.acp_session = !changed.mcp.acp_session;
        mutations.push(("mcp.acp_session", changed));
        let mut changed = baseline.clone();
        changed.capabilities.models.insert(
            "gpt-5.6-terra".to_string(),
            ModelCapabilities {
                efforts: vec!["high".to_string()],
            },
        );
        mutations.push(("capabilities.models", changed));
        for (field, changed) in mutations {
            assert_ne!(
                first.digest,
                resolved_definition(&changed).unwrap().digest,
                "le digest doit changer avec {field}"
            );
        }
        assert!(AgentRegistry::from_resolved("codex", &first).is_ok());
        let mut forged = first;
        forged.queue_capacity += 1;
        assert!(AgentRegistry::from_resolved("codex", &forged).is_err());
    }

    #[test]
    fn definition_figee_historique_reste_reprise_avec_la_capacite_acp_de_migration() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let definition = registry.get("codex").unwrap().clone();
        let mut legacy = resolved_definition(&definition).unwrap();
        legacy.digest = legacy_resolved_digest(&definition).unwrap();
        let mut value = serde_json::to_value(&legacy).unwrap();
        value.as_object_mut().unwrap().remove("capabilities");
        let decoded: ResolvedAgentDefinition = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.capabilities, AdapterCapabilities::default());
        assert!(AgentRegistry::from_resolved("codex", &decoded).is_ok());
    }

    #[test]
    fn lecture_privee_refuse_permissions_larges_et_accepte_0600() {
        let root = test_root("permissions");
        let path = root.join("agents.json");
        write_registry(&path, 0o644);
        let error = AgentRegistry::load_from_path(path.clone()).unwrap_err();
        assert!(error.contains("permissions 0644"));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(AgentRegistry::load_from_path(path).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lecture_privee_refuse_un_lien_symbolique() {
        let root = test_root("symlink");
        let target = root.join("target.json");
        let link = root.join("agents.json");
        write_registry(&target, 0o600);
        symlink(&target, &link).unwrap();
        let error = AgentRegistry::load_from_path(link).unwrap_err();
        assert!(error.contains("liens symboliques sont interdits"));
        std::fs::remove_dir_all(root).unwrap();
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
    fn invalid_mcp_interactive_is_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","mcp":{"interactive":"global-file"}}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("mcp.interactive invalide"));
    }

    #[test]
    fn matrice_de_capacites_refuse_les_champs_inconnus_et_les_doublons() {
        let unknown = AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","capabilities":{"execution_paths":["acp"],"future":true}}}}"#,
            "/tmp/agents.json",
        );
        assert!(unknown.unwrap_err().contains("unknown field `future`"));

        let duplicate = AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","capabilities":{"execution_paths":["acp","acp"]}}}}"#,
            "/tmp/agents.json",
        );
        assert!(
            duplicate
                .unwrap_err()
                .contains("chemin de capacité dupliqué")
        );
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
        let warnings = registry_warnings(
            r#"{"agents":{"codex":{"command":"codex","forbidden_env":[],"surprise":true}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents(),
        );
        assert_eq!(
            warnings,
            vec!["clé inconnue 'surprise' pour 'codex' dans /tmp/agents.json"]
        );
    }

    #[test]
    fn valid_entry_keys_produce_no_warning() {
        let warnings = registry_warnings(
            r#"{"agents":{"codex":{"command":"codex","args":[],"protocol":"acp","forbidden_env":[],"pass_env":["CODEX_HOME"],"permissions":"allow","queue_capacity":32,"notify_timeout_secs":600}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents(),
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
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"one":{"command":"bridge"},"two":{"command":"bridge"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let error = registry.type_for_command("bridge").unwrap_err();
        assert!(error.contains("commande ambiguë 'bridge'"));
        assert!(error.contains("one"));
        assert!(error.contains("two"));
    }

    #[test]
    fn undeclared_generic_command_names_source_and_choices() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.type_for_command("not-declared").unwrap_err();
        assert!(error.contains("/tmp/agents.json"));
        assert!(error.contains(&format!("codex ({NATIVE_CODEX_COMMAND})")));
    }

    #[test]
    fn pont_zed_codex_est_refuse_avant_lancement() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"legacy":{"command":"npx","args":["@zed-industries/codex-acp@0.16.0","-c","model=\"gpt-5.6-terra\""],"protocol":"acp"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let definition = registry.get("legacy").unwrap();
        let refusal = reject_retired_zed_bridge(definition).unwrap_err();
        assert!(matches!(
            refusal,
            SpawnRefusal::EnvUnfit { detail } if detail.contains("@zed-industries/codex-acp")
        ));
        assert!(matches!(
            validate_launch_capabilities("legacy", definition),
            Err(SpawnRefusal::EnvUnfit { .. })
        ));
    }

    #[test]
    fn pont_zed_refuse_les_quatre_formes_de_chemin_vivantes() {
        // C1 revue G10 : basename + retrait @version — les formes qui
        // échappaient à la comparaison naïve `== "codex-acp"`.
        let forms = [
            (
                "/opt/homebrew/bin/codex-acp",
                Vec::<&str>::new(),
                "@zed-industries/codex-acp",
            ),
            (
                "/Users/moi/.local/bin/claude-code-acp",
                vec![],
                "@zed-industries/claude-code-acp",
            ),
            (
                "./node_modules/.bin/codex-acp",
                vec![],
                "@zed-industries/codex-acp",
            ),
            ("npx", vec!["codex-acp@0.16.0"], "@zed-industries/codex-acp"),
        ];
        for (command, args, package) in forms {
            let args_json = serde_json::to_string(&args).unwrap();
            let json = format!(
                r#"{{"agents":{{"legacy":{{"command":"{command}","args":{args_json},"protocol":"acp"}}}}}}"#
            );
            let registry = AgentRegistry::from_json(&json, "/tmp/agents.json").unwrap();
            let definition = registry.get("legacy").unwrap();
            let refusal = reject_retired_zed_bridge(definition).unwrap_err();
            assert!(
                matches!(
                    &refusal,
                    SpawnRefusal::EnvUnfit { detail } if detail.contains(package)
                ),
                "forme non refusée: command={command} args={args:?} → {refusal:?}"
            );
        }
    }

    #[test]
    fn pont_zed_claude_est_refuse_avant_lancement() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"legacy":{"command":"claude-code-acp","protocol":"acp"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let definition = registry.get("legacy").unwrap();
        let refusal = reject_retired_zed_bridge(definition).unwrap_err();
        assert!(matches!(
            refusal,
            SpawnRefusal::EnvUnfit { detail } if detail.contains("@zed-industries/claude-code-acp")
        ));
    }

    #[test]
    fn acp_generique_cursor_et_gemini_restent_admis() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{
                "cursor":{"command":"cursor-agent","args":["--model","auto","acp"],"protocol":"acp","capabilities":{"execution_paths":["acp"],"models":{"auto":{"efforts":[]}}}},
                "gemini":{"command":"gemini","args":["--acp"],"protocol":"acp"}
            }}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        reject_retired_zed_bridge(registry.get("cursor").unwrap()).unwrap();
        reject_retired_zed_bridge(registry.get("gemini").unwrap()).unwrap();
        validate_launch_capabilities("cursor", registry.get("cursor").unwrap()).unwrap();
        // gemini sans modèle explicite : la matrice accepte le chemin seul
        validate_launch_capabilities("gemini", registry.get("gemini").unwrap()).unwrap();
    }

    #[test]
    fn defauts_natifs_codex_et_claude_ne_passent_pas_par_zed() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        for name in ["codex", "claude"] {
            let definition = registry.get(name).unwrap();
            reject_retired_zed_bridge(definition).unwrap();
            assert!(!definition.command.contains("npx"));
            assert!(
                !definition
                    .args
                    .iter()
                    .any(|arg| zed_bridge_token(arg).is_some())
            );
            assert_ne!(definition.protocol, "acp");
        }
    }
}
