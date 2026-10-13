//! Projection sanitaire des entrées fournisseur, sans fusion de settings maison.
use crate::lifecycle::{SourceEnvironment, build_environment};
use crate::registry::AgentDefinition;
use bridget_transport::protocol::{NativePermissionSnapshot, ProviderPermissions, SpawnPosture};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Sources administrées du système réellement exécutant. Aucun chemin d'un
/// autre OS ne constitue une preuve d'absence de politique sur cette machine.
fn managed_settings_root(platform: &str) -> Result<PathBuf, String> {
    match platform {
        "linux" => Ok(PathBuf::from("/etc/claude-code")),
        "macos" => Ok(PathBuf::from("/Library/Application Support/ClaudeCode")),
        _ => Err("permission_source_unavailable".into()),
    }
}

/// Preuve de contenu seulement : le caller doit attester connexion, parent,
/// révocation et hôte AVANT d'appeler ce lecteur sur l'hôte d'origine.
pub(crate) fn prove_parent_permissions(
    parent: &ProviderPermissions,
    expected_revision: u64,
) -> Result<String, String> {
    bridget_transport::protocol::validate_permissions(parent, parent.source == "provider_turn")?;
    if parent.revision != expected_revision {
        return Err("permission_revision_changed".into());
    }
    if !std::fs::canonicalize(&parent.cwd).is_ok_and(|path| path.is_dir()) {
        return Err("permission_source_unavailable".into());
    }
    if parent.driver == "claude_stream_json" {
        let context = parent
            .provider_policy
            .get("launch_context")
            .ok_or_else(unavailable)?;
        bridget_transport::protocol::recheck_permission_context_sources(context)?;
        // On ne transporte pas une fusion de settings. Des règles opaques
        // dépendantes de leurs fichiers d'origine ne sont pas exportables.
        if !claude_inputs_without_rules(context)? || !portable_claude_rules(&parent.provider_policy)
        {
            return Err("permission_mapping_unavailable".into());
        }
    }
    use sha2::{Digest, Sha256};
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(parent).map_err(|_| unavailable())?)
    ))
}

fn portable_claude_rules(policy: &Value) -> bool {
    ["allowed_tools", "disallowed_tools"].iter().all(|field| {
        policy.get(*field).is_none_or(|value| {
            value.as_array().is_some_and(|rules| {
                rules.iter().all(|rule| {
                    rule.as_str().is_some_and(|name| {
                        !name.is_empty()
                            && name
                                .bytes()
                                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    })
                })
            })
        })
    }) && policy
        .pointer("/launch_context/settings_overrides")
        .is_none()
}

/// Projection sur le worker destinataire, après preuve parent authentifiée par
/// le maître. Aucun fichier d'origine n'est lu ici. Le mapping est lui-même une
/// autorité administrative revalidée par le caller, pas une donnée du modèle.
pub(crate) fn project_remote_permissions(
    parent: &ProviderPermissions,
    definition: &AgentDefinition,
    mapping: &crate::registry::ProjectMapping,
    execution_cwd: &Path,
    source: &SourceEnvironment,
    posture: Option<SpawnPosture>,
    protected: &[PathBuf],
) -> Result<(Value, SpawnPosture), String> {
    bridget_transport::protocol::validate_permissions(parent, parent.source == "provider_turn")?;
    let root = crate::registry::validate_unprotected_local_path(&mapping.remote_root, protected)?;
    if !root.is_dir()
        || !mapping.source_root.is_absolute()
        || mapping
            .source_root
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("permission_mapping_unavailable".into());
    }
    // strip_prefix agit sur les composants et ne confond pas project/project-x.
    let project_path = |origin: &str| -> Result<PathBuf, String> {
        let original = Path::new(origin);
        if original
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err("permission_mapping_unavailable".into());
        }
        let suffix = original
            .strip_prefix(&mapping.source_root)
            .map_err(|_| "permission_mapping_unavailable")?;
        let translated =
            crate::registry::validate_unprotected_local_path(&root.join(suffix), protected)?;
        if !translated.starts_with(&root) || !translated.is_dir() {
            return Err("permission_mapping_unavailable".into());
        }
        Ok(translated)
    };
    let cwd = crate::registry::validate_unprotected_local_path(execution_cwd, protected)?;
    if !cwd.starts_with(project_path(&parent.cwd)?) || !cwd.is_dir() {
        return Err("permission_mapping_unavailable".into());
    }
    let mut translated = parent.clone();
    translated.cwd = cwd.to_string_lossy().into_owned();
    if parent.driver == "claude_stream_json" && !portable_claude_rules(&parent.provider_policy) {
        return Err("permission_mapping_unavailable".into());
    }
    for pointer in ["/sandbox_policy/writableRoots", "/additional_directories"] {
        if let Some(paths) = translated.provider_policy.pointer_mut(pointer) {
            let mapped = paths
                .as_array()
                .ok_or("permission_mapping_unavailable")?
                .iter()
                .map(|path| project_path(path.as_str().ok_or("permission_mapping_unavailable")?))
                .collect::<Result<Vec<_>, String>>()?;
            *paths = serde_json::to_value(mapped).map_err(|_| unavailable())?;
        }
    }
    let (policy, effective) =
        child_policy_reader_mode(&translated, definition, &cwd, source, true)(posture)?;
    recheck_remote_child_permissions(&policy, definition, &cwd, source, protected)?;
    Ok((policy, effective))
}

/// Relecture locale avant RELEASE/reprise. La preuve origine reste au maître.
/// Le namespace du worker n'est jamais ajouté à la politique fournisseur pour
/// permettre l'écriture du slot : seul le wrapper d'infrastructure l'écrit.
pub(crate) fn recheck_remote_child_permissions(
    policy: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
    protected: &[PathBuf],
) -> Result<(), String> {
    crate::registry::validate_unprotected_local_path(cwd, protected)?;
    for pointer in ["/sandbox_policy/writableRoots", "/additional_directories"] {
        if let Some(paths) = policy.pointer(pointer) {
            for path in paths.as_array().ok_or("permission_mapping_unavailable")? {
                crate::registry::validate_unprotected_local_path(
                    Path::new(path.as_str().ok_or("permission_mapping_unavailable")?),
                    protected,
                )?;
            }
        }
    }
    if definition.protocol == "claude_stream_json" {
        let context = policy.get("launch_context").ok_or_else(unavailable)?;
        recheck_context(context, definition, cwd, source)?;
        // Les settings distants peuvent ajouter des droits/racines malgré des
        // arguments sûrs. Tant que leur projection n'est pas démontrable, on
        // refuse leur portée opaque plutôt que de fusionner ou l'ignorer.
        if !claude_inputs_without_rules(context)? {
            return Err("permission_mapping_unavailable".into());
        }
    }
    Ok(())
}

/// O(d+q+p), d détails, q questions, p politiques ; aucune lecture de fichier.
/// Les effets sont normalisés par le transport. Les droits restent ceux149.
pub(crate) fn classify_request(
    frozen: &NativePermissionSnapshot,
    request: &bridget_transport::protocol::ChildProviderRequest,
    parent_available: bool,
) -> (
    bridget_transport::protocol::ChildRequestVerdict,
    Option<&'static str>,
) {
    use bridget_transport::protocol::{
        ChildRequestDecision, ChildRequestKind, ChildRequestVerdict,
    };
    let deny = |code| (ChildRequestVerdict::Forbidden, Some(code));
    if !parent_available {
        return deny("parent_unavailable");
    }
    if request.kind == ChildRequestKind::ElicitationForm {
        return deny(if request.details["mode"] == "url" {
            "url_mode_unsupported"
        } else {
            "approval_method_unsupported"
        });
    }
    let method = request.details["method"].as_str();
    let supported = match request.kind {
        ChildRequestKind::UserInput => method == Some("item/tool/requestUserInput"),
        ChildRequestKind::ApprovalCommand => {
            method == Some("item/commandExecution/requestApproval")
                && request.details["kind"] == "command"
        }
        // Effet synthétique établi du classifier ; le transport155 refuse les
        // vraies demandes fileChange0.162.1 avant tout événement forward.
        ChildRequestKind::ApprovalFile => method == Some("item/fileChange/requestApproval"),
        ChildRequestKind::ElicitationForm => false,
    };
    if !supported {
        return deny("approval_method_unsupported");
    }
    let Some(details) = request.details.as_object() else {
        return deny("details_incomplete");
    };
    if request.version != 1
        || request.truncated
        || details.get("complete") != Some(&Value::Bool(true))
        || details.keys().any(|key| {
            !matches!(
                key.as_str(),
                "method"
                    | "kind"
                    | "duration"
                    | "effects"
                    | "complete"
                    | "secret"
                    | "command"
                    | "cwd"
                    | "paths_cited"
                    | "questions"
                    | "mode"
                    | "reason"
            )
        })
        || details
            .get("reason")
            .is_some_and(|reason| !reason.is_string() && !reason.is_null())
        || details.get("cwd").is_some_and(|cwd| match cwd {
            Value::Null => false,
            Value::String(cwd) => cwd.len() > 1024,
            _ => true,
        })
    {
        return deny("details_incomplete");
    }
    let Some(effects) = details.get("effects").and_then(Value::as_object) else {
        return deny("details_incomplete");
    };
    if effects.len() != 3
        || ["command", "network", "file_write"]
            .iter()
            .any(|key| !effects.get(*key).is_some_and(Value::is_boolean))
    {
        return deny("details_incomplete");
    }
    let command = effects["command"] == true;
    let network = effects["network"] == true;
    let file_write = effects["file_write"] == true;
    if request.details["duration"] != "once" {
        return deny("duration_unsupported");
    }
    if frozen.version != 1
        || frozen.source != frozen.parent.source
        || frozen.owner_agent_id.is_empty()
        || frozen.owner_instance_id != frozen.parent.provider_instance_id
        || !Path::new(&frozen.project_cwd).is_absolute()
        || bridget_transport::protocol::validate_permissions(
            &frozen.parent,
            frozen.parent.source == "provider_turn",
        )
        .is_err()
        || frozen.parent.provider_policy["approvals_reviewer"] != "user"
    {
        return deny("parent_limit_unknown");
    }
    // Valider le schéma complet de la politique enfant avec le validateur pur
    // existant, sans recapturer le contexte ni relire les fichiers149.
    let mut child = frozen.parent.clone();
    child.driver = "codex_app_server".into();
    child.provider_policy = frozen.child_policy.clone();
    if bridget_transport::protocol::validate_permissions(&child, child.source == "provider_turn")
        .is_err()
        || child.provider_policy["approvals_reviewer"] != "user"
    {
        return deny("parent_limit_unknown");
    }
    if request.kind == ChildRequestKind::UserInput {
        if command || network || file_write {
            return deny("details_incomplete");
        }
        let Some(questions) = details.get("questions").and_then(Value::as_array) else {
            return deny("details_incomplete");
        };
        if questions.is_empty() || questions.len() != request.question_ids.len() {
            return deny("details_incomplete");
        }
        let mut ids = std::collections::HashSet::new();
        let mut secret = false;
        for (index, question) in questions.iter().enumerate() {
            let Some(question) = question.as_object() else {
                return deny("details_incomplete");
            };
            if question.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "id" | "header" | "question" | "isSecret" | "isOther" | "options"
                )
            }) || ["id", "header", "question"]
                .iter()
                .any(|key| !question.get(*key).is_some_and(Value::is_string))
                || ["isSecret", "isOther"]
                    .iter()
                    .any(|key| question.get(*key).is_some_and(|value| !value.is_boolean()))
            {
                return deny("details_incomplete");
            }
            let id = question["id"].as_str().unwrap_or("");
            if id.is_empty() || request.question_ids[index] != id || !ids.insert(id) {
                return deny("details_incomplete");
            }
            if let Some(options) = question.get("options").filter(|value| !value.is_null()) {
                let Some(options) = options.as_array() else {
                    return deny("details_incomplete");
                };
                if options.iter().any(|option| {
                    !option.as_object().is_some_and(|option| {
                        option.len() == 2
                            && ["label", "description"]
                                .iter()
                                .all(|key| option.get(*key).is_some_and(Value::is_string))
                    })
                }) {
                    return deny("details_incomplete");
                }
            }
            // Défaut false normatif UserInput, jamais déduit du texte libre.
            secret |= question.get("isSecret") == Some(&Value::Bool(true));
        }
        if secret {
            return deny("secret_not_forwardable");
        }
        if details.get("secret") != Some(&Value::Bool(false)) {
            return deny("details_incomplete");
        }
        return (ChildRequestVerdict::Parent, None);
    }
    if details.get("secret") != Some(&Value::Bool(false))
        || !request.offered.contains(&ChildRequestDecision::Accept)
        || !request.offered.iter().any(|decision| {
            matches!(
                decision,
                ChildRequestDecision::Decline | ChildRequestDecision::Cancel
            )
        })
    {
        return deny("details_incomplete");
    }
    if child.provider_policy["sandbox_policy"]["type"] == "readOnly" {
        return deny("child_restricted");
    }
    let sandbox = &frozen.parent.provider_policy["sandbox_policy"];
    let full = sandbox["type"] == "dangerFullAccess";
    if !full && sandbox["type"] != "workspaceWrite" {
        return deny("parent_limit_unknown");
    }
    if file_write && !full {
        return deny("file_confinement_unproven");
    }
    if network && !full && sandbox.get("networkAccess") != Some(&Value::Bool(true)) {
        return deny("network_not_granted");
    }
    if command && !full {
        return deny("command_unbounded");
    }
    if !command && !network && !file_write {
        return deny("details_incomplete");
    }
    (ChildRequestVerdict::Human, None)
}

/// O(b), b octets parcourus des64 premières valeurs ; sortie<=4096octets UTF-8.
/// Affichage seulement : aucune permission, résolution ni lecture de chemin.
pub(crate) fn format_cited_paths(paths: &[String]) -> String {
    const LIMIT: usize = 4096;
    const TRUNCATED: &str = " ; … liste tronquée";
    let mut result = String::from("Chemins cités (non vérifiés) : ");
    let budget = LIMIT - TRUNCATED.len();
    let mut truncated = paths.len() > 64;
    for (index, path) in paths.iter().take(64).enumerate() {
        if index > 0 {
            if result.len() + 3 > budget {
                truncated = true;
                break;
            }
            result.push_str(" ; ");
        }
        for character in path
            .chars()
            .filter(|character| !bridget_core::is_disallowed_control(*character))
        {
            if result.len() + character.len_utf8() > budget {
                truncated = true;
                break;
            }
            result.push(character);
        }
        if result.len() >= budget {
            truncated |= index + 1 < paths.len();
            break;
        }
    }
    if truncated {
        result.push_str(TRUNCATED);
    }
    result
}

pub(crate) const CHILD_POLICY_ENV: &str = "BRIDGET_NATIVE_CHILD_POLICY";
pub(crate) const CHILD_SOURCES_ENV: &str = "BRIDGET_NATIVE_PERMISSION_SOURCES";
const TRANSPARENT_GCLAUDE: &str =
    "sha256:dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e";
fn unavailable() -> String {
    "permission_source_unavailable".into()
}
fn changed() -> String {
    "settings_revision_changed".into()
}

pub(crate) fn source_revision(path: &Path) -> Result<String, String> {
    bridget_transport::protocol::permission_source_revision(path, 16 * 1024 * 1024)
}

fn resolved(command: &str, env: &SourceEnvironment) -> Result<PathBuf, String> {
    let path = Path::new(command);
    let candidate = if path.is_absolute() {
        path.to_owned()
    } else {
        if path.components().count() != 1 {
            return Err(unavailable());
        }
        let search = env.get("PATH").ok_or_else(unavailable)?;
        std::env::split_paths(search)
            .filter(|p| p.is_absolute())
            .map(|p| p.join(command))
            .find(|p| {
                use std::os::unix::fs::PermissionsExt;
                p.metadata()
                    .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            })
            .ok_or_else(unavailable)?
    };
    std::fs::canonicalize(candidate).map_err(|_| unavailable())
}

fn executable_revision(path: &Path) -> Result<String, String> {
    if !path.metadata().is_ok_and(|m| m.is_file()) {
        return Err(unavailable());
    }
    bridget_transport::protocol::permission_source_revision(path, 512 * 1024 * 1024)
}

fn selected_sources(args: &[String]) -> Result<Value, String> {
    let mut selected = Value::String("provider_default".into());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        let value = if arg == "--setting-sources" {
            index += 1;
            args.get(index).map(String::as_str)
        } else {
            arg.strip_prefix("--setting-sources=")
        };
        if let Some(value) = value {
            let sources: Vec<_> = value.split(',').filter(|s| !s.is_empty()).collect();
            if sources
                .iter()
                .any(|s| !matches!(*s, "user" | "project" | "local"))
                || sources.len() > 3
            {
                return Err(unavailable());
            }
            selected = json!(sources);
        }
        index += 1;
    }
    Ok(selected)
}

fn read_json_source(path: &Path) -> Result<Value, String> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|_| unavailable())?;
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(unavailable());
    }
    serde_json::from_slice(&bytes).map_err(|_| unavailable())
}

fn security_source_is_observable(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }
    let value = read_json_source(path)?;
    if ["policyHelper", "policyHelpers", "processWrapper", "hooks"]
        .iter()
        .any(|key| value.get(*key).is_some())
    {
        return Err(unavailable());
    }
    Ok(())
}

/// Références exactes du profil sélectionné. Le fournisseur reste responsable
/// de fusionner ces entrées ; Bridget ne publie que leurs chemins et révisions.
pub(crate) fn capture_context(
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
) -> Result<Value, String> {
    let env = build_environment(definition, source).map_err(|_| unavailable())?;
    capture_context_with_env(definition, cwd, &env)
}

pub(crate) fn capture_context_with_env(
    definition: &AgentDefinition,
    cwd: &Path,
    env: &SourceEnvironment,
) -> Result<Value, String> {
    if definition.protocol != "claude_stream_json" {
        return Err(unavailable());
    }
    if definition.args.iter().any(|arg| {
        matches!(
            arg.split('=').next(),
            Some("--agent" | "--agents" | "--plugin-dir")
        )
    }) {
        return Err(unavailable());
    }
    if env
        .keys()
        .any(|key| matches!(key.as_str(), "BASH_ENV" | "ENV") || key.starts_with("BASH_FUNC_"))
    {
        return Err(unavailable());
    }
    let launcher = resolved(&definition.command, env)?;
    let launcher_revision = executable_revision(&launcher)?;
    let direct_cli = resolved("claude", env)?;
    let cli = if launcher_revision == TRANSPARENT_GCLAUDE {
        direct_cli
    } else {
        // A direct provider CLI is the command resolved by the owner's PATH.
        // An arbitrary binary launcher is not proof that flags are forwarded.
        if launcher != direct_cli {
            return Err(unavailable());
        }
        launcher.clone()
    };
    // Une chaîne shell inconnue n'est pas une capacité de lancement prouvée.
    use std::io::Read;
    let mut head = [0u8; 2];
    let n = std::fs::File::open(&cli)
        .map_err(|_| unavailable())?
        .read(&mut head)
        .map_err(|_| unavailable())?;
    if n == 2 && head == *b"#!" {
        return Err(unavailable());
    }
    let config = env
        .get("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            env.get("HOME")
                .map(|home| PathBuf::from(home).join(".claude"))
        })
        .ok_or_else(unavailable)?;
    let config = std::fs::canonicalize(config).map_err(|_| unavailable())?;
    let selected = selected_sources(&definition.args)?;
    let managed = managed_settings_root(std::env::consts::OS)?;
    let mut paths = vec![
        ("user", config.join("settings.json")),
        ("managed", config.join("remote-settings.json")),
        ("managed", managed.join("managed-settings.json")),
        ("managed", managed.join("managed-settings.d")),
    ];
    // Les politiques MDM ne sont pas assimilées aux settings fichiers.
    if cfg!(target_os = "macos") {
        let home = env.get("HOME").map(PathBuf::from).ok_or_else(unavailable)?;
        for path in [
            PathBuf::from("/Library/Managed Preferences/com.anthropic.claudecode.plist"),
            home.join("Library/Preferences/com.anthropic.claudecode.plist"),
        ] {
            if path.exists() {
                return Err(unavailable());
            }
            paths.push(("managed", path));
        }
    }
    for ancestor in cwd.ancestors() {
        paths.push(("project", ancestor.join(".claude/settings.json")));
        paths.push(("local", ancestor.join(".claude/settings.local.json")));
        if paths.len() > 24 {
            return Err(unavailable());
        }
        if ancestor.join(".git").exists() {
            break;
        }
    }
    let mut overrides = None;
    let mut index = 0;
    while index < definition.args.len() {
        let arg = &definition.args[index];
        let inline = if arg == "--settings" {
            index += 1;
            definition.args.get(index).map(String::as_str)
        } else {
            arg.strip_prefix("--settings=")
        };
        if let Some(value) = inline {
            if value.starts_with('{') {
                let parsed: Value = serde_json::from_str(value).map_err(|_| unavailable())?;
                if !parsed
                    .as_object()
                    .is_some_and(|o| o.keys().all(|k| k == "permissions"))
                {
                    return Err(unavailable());
                }
                overrides = Some(parsed);
            } else {
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err(unavailable());
                }
                paths.push(("cli", path));
            }
        }
        index += 1;
    }
    let settings = config.join("settings.json");
    let settings: Value = if settings.exists() {
        read_json_source(&settings)?
    } else {
        Value::Null
    };
    if let Some(enabled) = settings.get("enabledPlugins").and_then(Value::as_object) {
        let installed_path = config.join("plugins/installed_plugins.json");
        paths.push(("user", installed_path.clone()));
        let installed: Value = read_json_source(&installed_path)?;
        let mut names = enabled
            .iter()
            .filter_map(|(name, value)| {
                if value.as_bool() == Some(true) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        names.sort();
        for name in names {
            let entries = installed["plugins"][name.as_str()]
                .as_array()
                .ok_or_else(unavailable)?;
            let entries = entries
                .iter()
                .filter(|e| e["scope"] == "user")
                .collect::<Vec<_>>();
            if entries.len() != 1 {
                return Err(unavailable());
            }
            let path = PathBuf::from(entries[0]["installPath"].as_str().ok_or_else(unavailable)?);
            if !path.is_absolute() {
                return Err(unavailable());
            }
            let path = std::fs::canonicalize(path).map_err(|_| unavailable())?;
            if !path.starts_with(config.join("plugins/cache")) {
                return Err(unavailable());
            }
            let manifest_path = path.join(".claude-plugin/plugin.json");
            let manifest: Value = read_json_source(&manifest_path)?;
            if [
                "hooks",
                "settings",
                "permissions",
                "policyHelper",
                "policyHelpers",
                "processWrapper",
                "mcpServers",
                "lspServers",
            ]
            .iter()
            .any(|key| manifest.get(*key).is_some())
            {
                return Err(unavailable());
            }
            paths.push(("user", manifest_path));
            for suffix in ["hooks/hooks.json", "settings.json", ".mcp.json"] {
                let source = path.join(suffix);
                if source.exists() {
                    return Err(unavailable());
                }
                paths.push(("user", source));
            }
        }
    }
    if paths.len() > 32 {
        return Err(unavailable());
    }
    let mut sources = Vec::new();
    for (kind, path) in paths {
        security_source_is_observable(&path)?;
        if path != config.join("settings.json")
            && path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    matches!(
                        name,
                        "settings.json"
                            | "settings.local.json"
                            | "managed-settings.json"
                            | "remote-settings.json"
                    )
                })
        {
            if read_json_source(&path)?.get("enabledPlugins").is_some() {
                return Err(unavailable());
            }
        }
        if path.is_dir() {
            for entry in std::fs::read_dir(&path).map_err(|_| unavailable())? {
                security_source_is_observable(&entry.map_err(|_| unavailable())?.path())?;
            }
        }
        sources.push(json!({"kind":kind,"path":path,"revision":source_revision(&path)?}));
    }
    let mut context = json!({"cli_path":launcher,"cli_revision":launcher_revision,"resolved_cli_path":cli,"resolved_cli_revision":executable_revision(&cli)?,"config_dir":config,"settings_sources":selected,"permission_sources":sources});
    if let Some(overrides) = overrides {
        context["settings_overrides"] = overrides;
    }
    Ok(context)
}

pub(crate) fn recheck_context(
    context: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
) -> Result<(), String> {
    let environment = build_environment(definition, source).map_err(|_| changed())?;
    recheck_context_with_env(context, definition, cwd, &environment)
}

pub(crate) fn recheck_context_with_env(
    context: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    environment: &SourceEnvironment,
) -> Result<(), String> {
    let captured = capture_context_with_env(definition, cwd, environment).map_err(|_| changed())?;
    // Les overrides ajoutés par le montage MCP natif ne remplacent pas les
    // entrées de droits du parent. Leurs règles restent vérifiées séparément.
    for key in [
        "cli_path",
        "cli_revision",
        "resolved_cli_path",
        "resolved_cli_revision",
        "config_dir",
        "settings_sources",
        "permission_sources",
    ] {
        if context.get(key) != captured.get(key) {
            return Err(changed());
        }
    }
    if let Some(sources) = context["permission_sources"].as_array() {
        for source in sources {
            let path = source["path"].as_str().ok_or_else(changed)?;
            if source_revision(Path::new(path)).map_err(|_| changed())? != source["revision"] {
                return Err(changed());
            }
        }
    }
    Ok(())
}

fn claude_inputs_without_rules(context: &Value) -> Result<bool, String> {
    let sources = context["permission_sources"]
        .as_array()
        .ok_or_else(unavailable)?;
    for source in sources {
        let path = Path::new(source["path"].as_str().ok_or_else(unavailable)?);
        if source_revision(path)? != source["revision"] {
            return Err(changed());
        }
        if source["revision"] == "absent" {
            continue;
        }
        let settings_paths = if path.is_dir() {
            std::fs::read_dir(path)
                .map_err(|_| unavailable())?
                .take(257)
                .map(|entry| entry.map(|entry| entry.path()).map_err(|_| unavailable()))
                .collect::<Result<Vec<_>, _>>()?
        } else if matches!(source["kind"].as_str(), Some("managed" | "cli"))
            || path
                .file_name()
                .and_then(|p| p.to_str())
                .is_some_and(|name| {
                    matches!(
                        name,
                        "settings.json"
                            | "settings.local.json"
                            | "managed-settings.json"
                            | "remote-settings.json"
                    )
                })
        {
            vec![path.to_owned()]
        } else {
            continue;
        };
        if settings_paths.len() > 256 {
            return Err(unavailable());
        }
        for settings_path in settings_paths {
            let settings = read_json_source(&settings_path)?;
            if !settings.as_object().is_some_and(|o| {
                o.keys().all(|key| {
                    matches!(
                        key.as_str(),
                        "modelPricing"
                            | "model"
                            | "effortLevel"
                            | "enabledPlugins"
                            | "extraKnownMarketplaces"
                    )
                })
            }) {
                return Ok(false);
            }
        }
    }
    if context.get("settings_overrides").is_some() {
        return Ok(false);
    }
    Ok(true)
}

/// Admission fraîche : aucun résultat de lecture ne survit à cet appel.
pub(crate) fn child_policy(
    parent: &ProviderPermissions,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
    posture: Option<SpawnPosture>,
) -> Result<(Value, SpawnPosture), String> {
    child_policy_reader(parent, definition, cwd, source)(posture)
}

/// R8 : un lecteur lié aux mêmes parent/définition/cwd/env pour une seule entrée.
/// O(B+3P), B octets des sources lus une fois, P politique dérivée par posture.
/// Chaque posture exécute les branches communes ; un refus inherit ne ferme pas discovery.
pub(crate) fn child_policy_reader<'a>(
    parent: &'a ProviderPermissions,
    definition: &'a AgentDefinition,
    cwd: &'a Path,
    source: &'a SourceEnvironment,
) -> impl FnMut(Option<SpawnPosture>) -> Result<(Value, SpawnPosture), String> + 'a {
    child_policy_reader_mode(parent, definition, cwd, source, false)
}

fn child_policy_reader_mode<'a>(
    parent: &'a ProviderPermissions,
    definition: &'a AgentDefinition,
    cwd: &'a Path,
    source: &'a SourceEnvironment,
    origin_attested: bool,
) -> impl FnMut(Option<SpawnPosture>) -> Result<(Value, SpawnPosture), String> + 'a {
    let mut captured: Option<Result<Value, String>> = None;
    let mut rechecked: Option<Result<(), String>> = None;
    let mut without_rules: Option<Result<bool, String>> = None;
    move |posture| {
        bridget_transport::protocol::validate_permissions(
            parent,
            parent.source == "provider_turn",
        )?;
        let mut policy = parent.provider_policy.clone();
        let readonly = posture == Some(SpawnPosture::Discovery);
        match (parent.driver.as_str(), definition.protocol.as_str()) {
            ("codex_app_server", "codex_app_server") => {
                if policy["sandbox_policy"]["type"] == "externalSandbox" {
                    return Err("provider_confinement_unavailable".into());
                }
                if readonly {
                    policy["sandbox_policy"] = json!({"type":"readOnly","networkAccess":false});
                }
                if posture == Some(SpawnPosture::Development)
                    && policy["sandbox_policy"]["type"] == "readOnly"
                {
                    return Err("permission_not_inherited".into());
                }
                // La politique réelle est conservée. Le transport refuse toute
                // extension nécessitant une approbation ; il n'ouvre pas de UI.
            }
            ("codex_app_server", "claude_stream_json") => {
                let inherited_readonly = policy["sandbox_policy"]["type"] == "readOnly";
                if inherited_readonly
                    && !readonly
                    && policy["sandbox_policy"].get("networkAccess") != Some(&Value::Bool(false))
                {
                    return Err("permission_mapping_unavailable".into());
                }
                let read = readonly || inherited_readonly;
                if !read && policy["sandbox_policy"]["type"] != "dangerFullAccess" {
                    return Err("provider_confinement_unavailable".into());
                }
                if !read && policy["approval_policy"] != "never" {
                    return Err("permission_mapping_unavailable".into());
                }
                policy = json!({"kind":"claude","permission_mode":if read{"plan"}else{"bypassPermissions"},"tools":{"type":"preset","preset":"claude_code"},"permission_callback":{"kind":"native_wrapper","tool_approval":"allow","plan_exit":"deny"},"settings_sources":"provider_default","launch_context":captured.get_or_insert_with(|| capture_context(definition,cwd,source)).clone()?});
                if read {
                    policy["tools"] = json!(["Read", "Glob", "Grep"]);
                    policy["disallowed_tools"] = json!([
                        "Bash",
                        "Edit",
                        "Write",
                        "NotebookEdit",
                        "WebFetch",
                        "WebSearch"
                    ]);
                }
            }
            ("claude_stream_json", "claude_stream_json") => {
                if origin_attested {
                    policy["launch_context"] = captured
                        .get_or_insert_with(|| capture_context(definition, cwd, source))
                        .clone()?;
                } else {
                    let context = policy
                        .get("launch_context")
                        .ok_or("permission_source_unavailable")?;
                    rechecked
                        .get_or_insert_with(|| recheck_context(context, definition, cwd, source))
                        .clone()?;
                }
                if Path::new(&parent.cwd) != cwd {
                    return Err("permission_mapping_unavailable".into());
                }
                if !readonly
                    && policy["permission_callback"]["kind"] == "t3_runtime"
                    && policy["permission_callback"]["tool_approval"] == "allow"
                    && !policy["permission_mode"]
                        .as_str()
                        .is_some_and(|m| matches!(m, "bypassPermissions" | "auto"))
                {
                    return Err("permission_mapping_unavailable".into());
                }
                if readonly {
                    let available = policy["tools"].as_array();
                    let tools: Vec<&str> = ["Read", "Glob", "Grep"]
                        .into_iter()
                        .filter(|name| {
                            available.is_none_or(|tools| {
                                tools.iter().any(|tool| tool.as_str() == Some(name))
                            }) && policy["disallowed_tools"].as_array().is_none_or(|tools| {
                                !tools.iter().any(|tool| tool.as_str() == Some(name))
                            })
                        })
                        .collect();
                    policy["permission_mode"] = json!("plan");
                    policy["tools"] = json!(tools);
                    let mut denied = policy["disallowed_tools"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default();
                    for name in [
                        "Bash",
                        "Edit",
                        "Write",
                        "NotebookEdit",
                        "WebFetch",
                        "WebSearch",
                    ] {
                        if !denied.iter().any(|tool| tool == name) {
                            denied.push(json!(name));
                        }
                    }
                    policy["disallowed_tools"] = json!(denied);
                    // Existing allowed rules remain below the tool inventory and
                    // never add a tool removed by the discovery reduction.
                } else if posture == Some(SpawnPosture::Development)
                    && policy["permission_mode"] == "plan"
                {
                    return Err("permission_not_inherited".into());
                }
                // The inherited inputs enforce rules in Claude; none cannot approve
                // an action which requires the parent's human mediation.
                policy["permission_callback"]["kind"] = json!("native_wrapper");
            }
            ("claude_stream_json", "codex_app_server") => {
                let context = policy
                    .get("launch_context")
                    .ok_or("permission_source_unavailable")?;
                if (!origin_attested
                    && !without_rules
                        .get_or_insert_with(|| claude_inputs_without_rules(context))
                        .clone()?)
                    || policy["disallowed_tools"]
                        .as_array()
                        .is_some_and(|a| !a.is_empty())
                {
                    return Err("permission_mapping_unavailable".into());
                }
                let tools = &policy["tools"];
                // An empty built-in inventory must never gain Codex file tools.
                // Nonempty auto-approval rules have no exact Codex translation.
                if tools.as_array().is_some_and(Vec::is_empty)
                    || policy["allowed_tools"]
                        .as_array()
                        .is_some_and(|rules| !rules.is_empty())
                {
                    return Err("permission_mapping_unavailable".into());
                }
                let full = policy["permission_mode"] == "bypassPermissions"
                    && policy["permission_callback"]["tool_approval"] == "allow"
                    && (tools["preset"] == "claude_code"
                        || tools
                            .as_array()
                            .is_some_and(|a| a.iter().any(|t| t == "Bash")));
                let read = readonly
                    || (policy["permission_mode"] == "plan"
                        && tools.as_array().is_some_and(|a| {
                            a.iter().all(|t| {
                                t.as_str()
                                    .is_some_and(|t| matches!(t, "Read" | "Glob" | "Grep"))
                            })
                        }));
                if !full && !read {
                    return Err("permission_mapping_unavailable".into());
                }
                policy = json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":if read{json!({"type":"readOnly","networkAccess":false})}else{json!({"type":"dangerFullAccess"})}});
            }
            _ => return Err("permission_mapping_unavailable".into()),
        }
        let only_read_tools = policy["tools"].as_array().is_some_and(|tools| {
            tools.iter().all(|tool| {
                tool.as_str()
                    .is_some_and(|name| matches!(name, "Read" | "Glob" | "Grep"))
            })
        });
        let effective = if readonly
            || policy["sandbox_policy"]["type"] == "readOnly"
            || policy["permission_mode"] == "plan"
            || only_read_tools
        {
            SpawnPosture::Discovery
        } else {
            SpawnPosture::Development
        };
        if posture == Some(SpawnPosture::Development) && effective == SpawnPosture::Discovery {
            return Err("permission_not_inherited".into());
        }
        Ok((policy, effective))
    }
}

pub(crate) fn apply_child_arguments(
    protocol: &str,
    policy: &Value,
    args: &mut Vec<String>,
) -> Result<(), String> {
    let mut kept = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        let flag = arg.split('=').next().unwrap_or(arg);
        if matches!(
            flag,
            "--permission-mode" | "--permission-prompts" | "--sandbox" | "--ask-for-approval"
        ) {
            index += if arg.contains('=') { 1 } else { 2 };
            continue;
        }
        if matches!(
            flag,
            "--tools"
                | "--allowedTools"
                | "--allowed-tools"
                | "--disallowedTools"
                | "--disallowed-tools"
                | "--add-dir"
        ) {
            index += 1;
            if !arg.contains('=') {
                while index < args.len() && !args[index].starts_with('-') {
                    index += 1;
                }
            }
            continue;
        }
        if protocol == "claude_stream_json" && matches!(flag, "--setting-sources" | "--settings") {
            // Preserve a selected settings file; sanitised inline overrides are
            // rebuilt from the frozen parent policy below.
            let value = arg
                .split_once('=')
                .map(|(_, v)| v)
                .or_else(|| args.get(index + 1).map(String::as_str));
            if flag == "--setting-sources" || value.is_some_and(|v| v.trim_start().starts_with('{'))
            {
                index += if arg.contains('=') { 1 } else { 2 };
                continue;
            }
        }
        if matches!(
            arg.as_str(),
            "--dangerously-skip-permissions"
                | "--allow-dangerously-skip-permissions"
                | "--yolo"
                | "--restricted"
                | "--full-auto"
        ) {
            index += 1;
            continue;
        }
        if arg.starts_with("--permission-mode=") || arg.starts_with("--permission-prompts=") {
            index += 1;
            continue;
        }
        if matches!(arg.as_str(), "-c" | "--config")
            && args.get(index + 1).is_some_and(|v| {
                [
                    "approval_policy=",
                    "sandbox_mode=",
                    "sandbox_workspace_write.",
                ]
                .iter()
                .any(|k| v.starts_with(k))
            })
        {
            index += 2;
            continue;
        }
        if [
            "approval_policy=",
            "sandbox_mode=",
            "sandbox_workspace_write.",
        ]
        .iter()
        .any(|k| arg.starts_with(k))
        {
            index += 1;
            continue;
        }
        kept.push(arg.clone());
        index += 1;
    }
    match protocol {
        "codex_app_server" => {} // Exact app-server policy accompanies each turn.
        "claude_stream_json" => {
            if policy
                .pointer("/launch_context/settings_overrides")
                .is_some()
                && kept
                    .iter()
                    .any(|arg| arg == "--settings" || arg.starts_with("--settings="))
            {
                return Err("permission_source_unavailable".into());
            }
            let mode = policy["permission_mode"]
                .as_str()
                .ok_or("permission_mapping_unavailable")?;
            kept.extend([
                "--permission-mode".into(),
                mode.into(),
                "--permission-prompts".into(),
                "none".into(),
            ]);
            // --restricted drops user/project/local settings. Preserve the
            // frozen inputs and enforce the reduction with tools and denies.
            if policy["allow_dangerously_skip_permissions"] == true {
                kept.push("--allow-dangerously-skip-permissions".into());
            }
            if let Some(tools) = policy["tools"].as_array() {
                kept.extend([
                    "--tools".into(),
                    tools
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(","),
                ]);
            }
            for (field, flag) in [
                ("allowed_tools", "--allowedTools"),
                ("disallowed_tools", "--disallowedTools"),
            ] {
                if let Some(values) = policy[field].as_array() {
                    kept.extend([
                        flag.into(),
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(","),
                    ]);
                }
            }
            if let Some(values) = policy["additional_directories"].as_array() {
                for value in values {
                    kept.extend([
                        "--add-dir".into(),
                        value
                            .as_str()
                            .ok_or("permission_mapping_unavailable")?
                            .into(),
                    ]);
                }
            }
            if let Some(value) = policy.pointer("/launch_context/settings_overrides") {
                kept.extend([
                    "--settings".into(),
                    serde_json::to_string(value).map_err(|_| unavailable())?,
                ]);
            }
            if let Some(selected) = policy
                .pointer("/launch_context/settings_sources")
                .and_then(Value::as_array)
            {
                kept.extend([
                    "--setting-sources".into(),
                    selected
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(","),
                ]);
            }
        }
        _ => return Err("permission_mapping_unavailable".into()),
    }
    *args = kept;
    Ok(())
}

pub(crate) fn snapshot(
    owner: &str,
    instance: &str,
    parent: ProviderPermissions,
    child_policy: Value,
    cwd: &Path,
) -> NativePermissionSnapshot {
    NativePermissionSnapshot {
        version: 1,
        source: parent.source.clone(),
        owner_agent_id: owner.into(),
        owner_instance_id: instance.into(),
        parent,
        child_policy,
        project_cwd: cwd.to_string_lossy().into_owned(),
    }
}

pub(crate) fn snapshot_contexts(snapshot: &NativePermissionSnapshot) -> Vec<Value> {
    let mut contexts = Vec::new();
    for context in [
        snapshot.parent.provider_policy.get("launch_context"),
        snapshot.child_policy.get("launch_context"),
    ]
    .into_iter()
    .flatten()
    {
        if !contexts.contains(context) {
            contexts.push(context.clone());
        }
    }
    contexts
}

pub(crate) fn runtime_mode(policy: &Value) -> &'static str {
    match policy["permission_mode"].as_str() {
        Some("default" | "plan") => "approval-required",
        Some("acceptEdits") => "auto-accept-edits",
        Some("bypassPermissions") => "full-access",
        Some("auto" | "dontAsk") => "auto",
        _ => {
            if policy["sandbox_policy"]["type"] == "dangerFullAccess"
                && policy["approval_policy"] == "never"
            {
                "full-access"
            } else if policy["approval_policy"] == "never" {
                "auto"
            } else {
                "approval-required"
            }
        }
    }
}

pub(crate) fn observed_fact(
    instance: &str,
    definition: &AgentDefinition,
    context: Option<&Value>,
    transport: &dyn bridget_transport::ManagedSession,
) -> Result<Option<ProviderPermissions>, String> {
    let Some((session, mut policy, revision, provider_cwd)) = transport.provider_permissions()
    else {
        return Ok(None);
    };
    let cwd = match provider_cwd {
        Some(cwd) => PathBuf::from(cwd),
        None if definition.protocol == "claude_stream_json" => {
            std::env::current_dir().map_err(|_| unavailable())?
        }
        None => return Err("permission_attestation_unavailable".into()),
    };
    if definition.protocol == "claude_stream_json" {
        policy["launch_context"] = context.cloned().ok_or_else(unavailable)?;
    }
    let readonly =
        policy["permission_mode"] == "plan" || policy["sandbox_policy"]["type"] == "readOnly";
    let runtime = runtime_mode(&policy);
    let fact = ProviderPermissions {
        version: 1,
        source: "native_wrapper".into(),
        run_id: transport
            .provider_identity()
            .and_then(|i| i.active_turn_id)
            .unwrap_or_else(|| session.clone()),
        provider_session_id: session,
        provider_instance_id: instance.into(),
        revision,
        driver: definition.protocol.clone(),
        cwd: cwd.to_string_lossy().into_owned(),
        runtime_mode: runtime.into(),
        interaction_mode: if readonly { "plan" } else { "default" }.into(),
        provider_policy: policy,
    };
    bridget_transport::protocol::validate_permissions(&fact, false)?;
    Ok(Some(fact))
}

#[cfg(test)]
#[path = "native_permissions149_tests.rs"]
mod native_permissions149_tests;
