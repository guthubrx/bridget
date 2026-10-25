//! Faits sanitaires de droits. Aucun credential ne rejoint un snapshot durable.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderPermissions {
    pub version: u16,
    pub source: String,
    pub run_id: String,
    pub provider_session_id: String,
    pub provider_instance_id: String,
    pub revision: u64,
    pub driver: String,
    pub cwd: String,
    pub runtime_mode: String,
    pub interaction_mode: String,
    pub provider_policy: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativePermissionSnapshot {
    pub version: u16,
    pub source: String,
    pub owner_agent_id: String,
    pub owner_instance_id: String,
    pub parent: ProviderPermissions,
    pub child_policy: Value,
    pub project_cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePermissionProof {
    pub endpoint: String,
    pub authorization: super::IdentityCredential,
}

pub fn validate_permissions(fact: &ProviderPermissions, t3: bool) -> Result<(), String> {
    let fail = || "permission_attestation_unavailable".to_string();
    let atom = |s: &str, max: usize| {
        !s.is_empty() && s.len() <= max && !s.chars().any(bridget_core::is_disallowed_control)
    };
    if fact.version != 1
        || fact.revision == 0
        || fact.revision > super::HUMAN_LINEAGE_MAX_SEQ
        || fact.source
            != if t3 {
                "provider_turn"
            } else {
                "native_wrapper"
            }
        || [
            &fact.run_id,
            &fact.provider_session_id,
            &fact.provider_instance_id,
        ]
        .iter()
        .any(|s| !atom(s, 256))
        || !atom(&fact.cwd, 4096)
        || !std::path::Path::new(&fact.cwd).is_absolute()
        || !matches!(
            fact.runtime_mode.as_str(),
            "approval-required" | "auto-accept-edits" | "auto" | "full-access"
        )
        || !matches!(fact.interaction_mode.as_str(), "default" | "plan")
    {
        return Err(fail());
    }
    let policy = &fact.provider_policy;
    let keys = |v: &Value, required: &[&str], optional: &[&str]| -> bool {
        v.as_object().is_some_and(|o| {
            required.iter().all(|k| o.contains_key(*k))
                && o.keys()
                    .all(|k| required.contains(&k.as_str()) || optional.contains(&k.as_str()))
        })
    };
    let array = |v: &Value, paths: bool| {
        v.as_array().is_some_and(|a| {
            a.len() <= 128
                && a.iter().all(|v| {
                    v.as_str().is_some_and(|s| {
                        atom(s, if paths { 4096 } else { 512 })
                            && (!paths || std::path::Path::new(s).is_absolute())
                    })
                })
        })
    };
    match fact.driver.as_str() {
        "codex_app_server" => {
            if policy["kind"] != "codex"
                || !keys(
                    policy,
                    &[
                        "kind",
                        "approval_policy",
                        "approvals_reviewer",
                        "sandbox_policy",
                    ],
                    &[],
                )
            {
                return Err(fail());
            }
            let approval = &policy["approval_policy"];
            if !approval
                .as_str()
                .is_some_and(|s| matches!(s, "untrusted" | "on-request" | "never"))
            {
                if !keys(approval, &["granular"], &[])
                    || !keys(
                        &approval["granular"],
                        &["mcp_elicitations", "rules", "sandbox_approval"],
                        &["request_permissions", "skill_approval"],
                    )
                    || approval["granular"]
                        .as_object()
                        .is_none_or(|o| o.values().any(|v| !v.is_boolean()))
                {
                    return Err(fail());
                }
            }
            if !policy["approvals_reviewer"]
                .as_str()
                .is_some_and(|s| matches!(s, "user" | "auto_review" | "guardian_subagent"))
            {
                return Err(fail());
            }
            let sandbox = &policy["sandbox_policy"];
            match sandbox["type"].as_str() {
                Some("dangerFullAccess") if keys(sandbox, &["type"], &[]) => {}
                Some("readOnly")
                    if keys(sandbox, &["type"], &["networkAccess"])
                        && sandbox.get("networkAccess").is_none_or(Value::is_boolean) => {}
                Some("workspaceWrite")
                    if keys(
                        sandbox,
                        &["type"],
                        &[
                            "networkAccess",
                            "writableRoots",
                            "excludeSlashTmp",
                            "excludeTmpdirEnvVar",
                        ],
                    ) && ["networkAccess", "excludeSlashTmp", "excludeTmpdirEnvVar"]
                        .iter()
                        .all(|k| sandbox.get(k).is_none_or(Value::is_boolean))
                        && sandbox.get("writableRoots").is_none_or(|v| array(v, true)) => {}
                Some("externalSandbox")
                    if keys(sandbox, &["type"], &["networkAccess"])
                        && sandbox.get("networkAccess").is_none_or(|v| {
                            v.as_str()
                                .is_some_and(|s| matches!(s, "restricted" | "enabled"))
                        }) => {}
                _ => return Err(fail()),
            }
        }
        "claude_stream_json" => {
            if policy["kind"] != "claude"
                || !keys(
                    policy,
                    &[
                        "kind",
                        "permission_mode",
                        "tools",
                        "permission_callback",
                        "settings_sources",
                    ],
                    &[
                        "allowed_tools",
                        "disallowed_tools",
                        "additional_directories",
                        "allow_dangerously_skip_permissions",
                        "launch_context",
                    ],
                )
            {
                return Err(fail());
            }
            if !policy["permission_mode"].as_str().is_some_and(|s| {
                matches!(
                    s,
                    "default" | "acceptEdits" | "bypassPermissions" | "plan" | "dontAsk" | "auto"
                )
            }) {
                return Err(fail());
            }
            if !(array(&policy["tools"], false)
                || (keys(&policy["tools"], &["type", "preset"], &[])
                    && policy["tools"]["type"] == "preset"
                    && policy["tools"]["preset"] == "claude_code"))
            {
                return Err(fail());
            }
            for (field, paths) in [
                ("allowed_tools", false),
                ("disallowed_tools", false),
                ("additional_directories", true),
            ] {
                if policy.get(field).is_some_and(|v| !array(v, paths)) {
                    return Err(fail());
                }
            }
            if policy
                .get("allow_dangerously_skip_permissions")
                .is_some_and(|v| !v.is_boolean())
                || policy["settings_sources"] != "provider_default"
            {
                return Err(fail());
            }
            let callback = &policy["permission_callback"];
            if !keys(callback, &["kind", "tool_approval", "plan_exit"], &[])
                || callback["kind"] != if t3 { "t3_runtime" } else { "native_wrapper" }
                || !callback["tool_approval"]
                    .as_str()
                    .is_some_and(|s| matches!(s, "prompt" | "allow"))
                || callback["plan_exit"] != "deny"
            {
                return Err(fail());
            }
            if let Some(context) = policy.get("launch_context") {
                if !keys(
                    context,
                    &[
                        "cli_path",
                        "cli_revision",
                        "resolved_cli_path",
                        "resolved_cli_revision",
                        "config_dir",
                        "settings_sources",
                        "permission_sources",
                    ],
                    &[
                        "attestation_version",
                        "hook_execution",
                        "source_equivalence",
                        "settings_overrides",
                    ],
                ) {
                    return Err(fail());
                }
                for field in ["cli_path", "resolved_cli_path", "config_dir"] {
                    if !context[field]
                        .as_str()
                        .is_some_and(|s| atom(s, 4096) && std::path::Path::new(s).is_absolute())
                    {
                        return Err(fail());
                    }
                }
                let digest = |v: &Value| {
                    v.as_str().is_some_and(|s| {
                        s.len() == 71
                            && s.starts_with("sha256:")
                            && s[7..]
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    })
                };
                if !digest(&context["cli_revision"]) || !digest(&context["resolved_cli_revision"]) {
                    return Err(fail());
                }
                if context["settings_sources"] != "provider_default"
                    && !context["settings_sources"].as_array().is_some_and(|a| {
                        a.len() <= 3
                            && a.iter().all(|v| {
                                v.as_str()
                                    .is_some_and(|s| matches!(s, "user" | "project" | "local"))
                            })
                            && a.iter().enumerate().all(|(i, v)| !a[..i].contains(v))
                    })
                {
                    return Err(fail());
                }
                let attestation_version = context
                    .get("attestation_version")
                    .and_then(Value::as_u64)
                    .unwrap_or(1);
                if !matches!(attestation_version, 1 | 2)
                    || (attestation_version == 1 && context.get("attestation_version").is_some())
                    || !context["permission_sources"].as_array().is_some_and(|a| {
                        a.len() <= if attestation_version == 2 { 128 } else { 32 }
                            && a.iter().all(|s| {
                                keys(s, &["kind", "path", "revision"], &[])
                                    && s["kind"].as_str().is_some_and(|s| {
                                        matches!(
                                            s,
                                            "user" | "project" | "local" | "managed" | "cli"
                                        ) || (attestation_version == 2 && s == "hook")
                                    })
                                    && s["path"].as_str().is_some_and(|p| {
                                        atom(p, 4096) && std::path::Path::new(p).is_absolute()
                                    })
                                    && (s["revision"] == "absent" || digest(&s["revision"]))
                                    && (s["kind"] != "hook" || digest(&s["revision"]))
                            })
                    })
                {
                    return Err(fail());
                }
                if attestation_version == 2
                    && serde_json::to_vec(context).map_err(|_| fail())?.len() > 48 * 1024
                {
                    return Err(fail());
                }
                if let Some(execution) = context.get("hook_execution") {
                    if !matches!(execution.as_str(), Some("native_same_host" | "native_remote_equivalent" | "native_remote_guarded"))
                        || attestation_version != 2
                        || !context["permission_sources"]
                            .as_array()
                            .is_some_and(|sources| {
                                sources.iter().any(|source| source["kind"] == "hook")
                            })
                    {
                        return Err(fail());
                    }
                } else if context["permission_sources"]
                    .as_array()
                    .is_some_and(|sources| sources.iter().any(|source| source["kind"] == "hook"))
                {
                    return Err(fail());
                }
                validate_permission_source_equivalence(context)?;
                if let Some(overrides) = context.get("settings_overrides") {
                    if !keys(overrides, &[], &["permissions"]) {
                        return Err("permission_source_unavailable".into());
                    }
                    if let Some(p) = overrides.get("permissions") {
                        if !keys(
                            p,
                            &[],
                            &[
                                "allow",
                                "ask",
                                "deny",
                                "defaultMode",
                                "additionalDirectories",
                                "disableBypassPermissionsMode",
                            ],
                        ) || ["allow", "ask", "deny"]
                            .iter()
                            .any(|key| p.get(key).is_some_and(|v| !array(v, false)))
                            || p.get("additionalDirectories")
                                .is_some_and(|v| !array(v, true))
                            || p.get("defaultMode").is_some_and(|v| {
                                !v.as_str().is_some_and(|s| {
                                    matches!(
                                        s,
                                        "default"
                                            | "acceptEdits"
                                            | "bypassPermissions"
                                            | "plan"
                                            | "dontAsk"
                                            | "auto"
                                    )
                                })
                            })
                            || p.get("disableBypassPermissionsMode")
                                .is_some_and(|v| v != "disable")
                        {
                            return Err("permission_source_unavailable".into());
                        }
                    }
                }
            }
        }
        _ => return Err(fail()),
    }
    if serde_json::to_vec(fact).map_err(|_| fail())?.len() > 64 * 1024 {
        return Err(fail());
    }
    Ok(())
}

/// Reprise fournisseur interne : les entrées figées sont recontrôlées avant
/// l'effet, même si le transport remplace lui-même un processus de reprise.
pub fn recheck_frozen_inputs(
    command: &str,
    environment: &[(String, String)],
) -> Result<(), String> {
    use std::path::{Path, PathBuf};
    if let Ok(raw) = std::env::var("BRIDGET_NATIVE_PERMISSION_SOURCES") {
        let contexts: Vec<Value> =
            serde_json::from_str(&raw).map_err(|_| "permission_attestation_unavailable")?;
        if contexts.len() > 2 {
            return Err("permission_attestation_unavailable".into());
        }
        for context in contexts {
            recheck_permission_context_sources(&context)?;
        }
    }
    let raw = std::env::var("BRIDGET_NATIVE_CHILD_POLICY").ok();
    let Some(raw) = raw else { return Ok(()) };
    let policy: Value =
        serde_json::from_str(&raw).map_err(|_| "permission_attestation_unavailable")?;
    let Some(context) = policy.get("launch_context") else {
        return Ok(());
    };
    let changed = || "settings_revision_changed".to_string();
    let path = environment
        .iter()
        .find(|(name, _)| name == "PATH")
        .map(|(_, value)| value.clone())
        .or_else(|| std::env::var("PATH").ok())
        .ok_or_else(changed)?;
    let resolve = |name: &str| -> Result<PathBuf, String> {
        let p = Path::new(name);
        let target = if p.is_absolute() {
            p.to_owned()
        } else {
            if p.components().count() != 1 {
                return Err(changed());
            }
            use std::os::unix::fs::PermissionsExt;
            std::env::split_paths(&path)
                .filter(|p| p.is_absolute())
                .map(|p| p.join(name))
                .find(|p| {
                    p.metadata()
                        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                })
                .ok_or_else(changed)?
        };
        std::fs::canonicalize(target).map_err(|_| changed())
    };
    let launcher = resolve(command)?;
    if launcher.to_str() != context["cli_path"].as_str()
        || permission_source_revision(&launcher, 512 * 1024 * 1024).map_err(|_| changed())?
            != context["cli_revision"]
    {
        return Err(changed());
    }
    let target = if context["cli_revision"]
        == "sha256:dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e"
    {
        resolve("claude")?
    } else {
        launcher
    };
    if target.to_str() != context["resolved_cli_path"].as_str()
        || permission_source_revision(&target, 512 * 1024 * 1024).map_err(|_| changed())?
            != context["resolved_cli_revision"]
    {
        return Err(changed());
    }
    recheck_permission_context_sources(context)
}

pub fn recheck_permission_context_sources(context: &Value) -> Result<(), String> {
    recheck_permission_context_sources_inner(context, None)
}

/// Réemploie uniquement une preuve issue du vérificateur fermé. Les lectures
/// des révisions et des sources ci-dessous restent intégralement exécutées.
pub fn recheck_permission_context_sources_with_profile(
    context: &Value, verified: &super::native_hook_profile171::VerifiedGuardedHookProfile,
) -> Result<(), String> {
    recheck_permission_context_sources_inner(context, Some(verified))
}

// Déduplication limitée à cette relecture. Chaque appel repart du disque ;
// les deux révisions attendues sont comparées, même pour un chemin identique.
fn recheck_cli_revisions(
    context: &Value,
    mut revision: impl FnMut(&std::path::Path) -> Result<String, String>,
) -> Result<(), String> {
    use std::path::Path;
    let changed = || "settings_revision_changed".to_string();
    let launcher = Path::new(context["cli_path"].as_str().ok_or_else(changed)?);
    let cli = Path::new(context["resolved_cli_path"].as_str().ok_or_else(changed)?);
    let launcher_revision = revision(launcher).map_err(|_| changed())?;
    if launcher_revision != context["cli_revision"] {
        return Err(changed());
    }
    let cli_revision = if cli == launcher {
        launcher_revision
    } else {
        revision(cli).map_err(|_| changed())?
    };
    if cli_revision != context["resolved_cli_revision"] {
        return Err(changed());
    }
    Ok(())
}

fn recheck_permission_context_sources_inner(
    context: &Value, verified: Option<&super::native_hook_profile171::VerifiedGuardedHookProfile>,
) -> Result<(), String> {
    use std::path::Path;
    let changed = || "settings_revision_changed".to_string();
    let attestation_version = context
        .get("attestation_version")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    if !matches!(attestation_version, 1 | 2)
        || (attestation_version == 1 && context.get("attestation_version").is_some())
        || (attestation_version == 2
            && serde_json::to_vec(context).map_err(|_| changed())?.len() > 48 * 1024)
    {
        return Err(changed());
    }
    recheck_cli_revisions(context, |path| {
        permission_source_revision(path, 512 * 1024 * 1024)
    })?;
    let sources = context["permission_sources"]
        .as_array()
        .filter(|s| s.len() <= if attestation_version == 2 { 128 } else { 32 })
        .ok_or_else(changed)?;
    let has_hooks = sources.iter().any(|source| source["kind"] == "hook");
    match (has_hooks, context.get("hook_execution")) {
        (true, Some(value)) if matches!(value.as_str(), Some("native_same_host" | "native_remote_equivalent" | "native_remote_guarded")) && attestation_version == 2 => {}
        (false, None) => {}
        _ => return Err(changed()),
    }
    validate_permission_source_equivalence(context).map_err(|_| changed())?;
    // Recontrôler les dépendances sur cet hôte seulement ; l'origine est
    // reconstituée indépendamment par le maître avec son parent authentifié.
    if context.pointer("/source_equivalence/version") == Some(&Value::from(2)) {
        if let Some(verified) = verified { verified.profile(context)?; }
        else { super::native_hook_profile171::recheck_guarded_hook_profile(context)?; }
    } else if let Some(witness) = context.get("source_equivalence") {
        use std::os::unix::fs::MetadataExt;
        let manifest = if witness["scope"] == "remote" { &witness["target"] } else { &witness["manifest"] };
        for row in manifest["rows"].as_array().ok_or_else(changed)? {
            for dependency in row["dependencies"].as_array().ok_or_else(changed)? {
                let path = Path::new(dependency["path"].as_str().ok_or_else(changed)?);
                let m = std::fs::metadata(path).map_err(|_| changed())?;
                if !m.is_file() || m.uid() != 0 || m.gid() != 0 || m.mode() & 0o022 != 0
                    || m.mode() & 0o7777 != dependency["mode"].as_u64().ok_or_else(changed)? as u32
                    || permission_source_revision_v2(path, 512 * 1024 * 1024).map_err(|_| changed())? != dependency["revision"] {
                    return Err(changed());
                }
            }
        }
    }
    for source in sources {
        let kind = source["kind"].as_str().ok_or_else(changed)?;
        if kind == "hook" && (attestation_version != 2 || source["revision"] == "absent") {
            return Err(changed());
        }
        if attestation_version == 1
            && std::fs::symlink_metadata(source["path"].as_str().ok_or_else(changed)?)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(changed());
        }
        let actual = (if source["kind"] == "hook" {
            permission_hook_source_revision
        } else if context.get("attestation_version") == Some(&Value::from(2)) {
            permission_source_revision_v2
        } else {
            permission_source_revision
        })(
            Path::new(source["path"].as_str().ok_or_else(changed)?),
            16 * 1024 * 1024,
        )
        .map_err(|_| changed())?;
        if actual != source["revision"] {
            // Only a native destination capture can carry this semantic digest.
            // Origin/legacy captures keep their raw revision and still fail changes.
            let native_marketplace = attestation_version == 2 && kind == "user"
                && context.get("hook_execution").is_none()
                && context.get("source_equivalence").is_none()
                && context["config_dir"].as_str().is_some_and(|root|
                    Path::new(root).join("plugins/known_marketplaces.json") ==
                    Path::new(source["path"].as_str().unwrap_or("")));
            if !native_marketplace || permission_marketplace_revision171(
                Path::new(source["path"].as_str().ok_or_else(changed)?), 16 * 1024 * 1024)
                .map_err(|_| changed())? != source["revision"] {
                return Err(changed());
            }
        }
    }
    Ok(())
}

/// Options de droits du CLI réellement lancé. Aucun JSON MCP ne les fournit.
pub fn claude_launch_policy_options(args: &[String], policy: &mut Value) -> Result<(), String> {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        let (flag, inline) = arg
            .split_once('=')
            .map(|(f, v)| (f, Some(v)))
            .unwrap_or((arg.as_str(), None));
        let field = match flag {
            "--tools" => Some("tools"),
            "--allowedTools" | "--allowed-tools" => Some("allowed_tools"),
            "--disallowedTools" | "--disallowed-tools" => Some("disallowed_tools"),
            "--add-dir" => Some("additional_directories"),
            _ => None,
        };
        if let Some(field) = field {
            let mut values = Vec::new();
            if let Some(value) = inline {
                values.extend(
                    value
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned),
                );
            } else {
                index += 1;
                while index < args.len() && !args[index].starts_with('-') {
                    values.extend(
                        args[index]
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned),
                    );
                    index += 1;
                }
                index = index.saturating_sub(1);
            }
            if values.len() > 128
                || values.iter().any(|v| {
                    v.len()
                        > if field == "additional_directories" {
                            4096
                        } else {
                            512
                        }
                        || v.chars().any(bridget_core::is_disallowed_control)
                        || (field == "additional_directories"
                            && !std::path::Path::new(v).is_absolute())
                })
            {
                return Err("permission_source_unavailable".into());
            }
            if field == "tools" && values == ["default"] {
                policy[field] = serde_json::json!({"type":"preset","preset":"claude_code"});
            } else {
                policy[field] = serde_json::json!(values);
            }
        } else if flag == "--allow-dangerously-skip-permissions" {
            policy["allow_dangerously_skip_permissions"] = Value::Bool(true);
        }
        index += 1;
    }
    Ok(())
}

pub fn permission_source_revision(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let unavailable = || "permission_source_unavailable".to_string();
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok("absent".into()),
        Err(_) => return Err(unavailable()),
    };
    if metadata.file_type().is_symlink() {
        return Err(unavailable());
    }
    if metadata.is_dir() {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|_| unavailable())? {
            let entry = entry.map_err(|_| unavailable())?;
            let name = entry.file_name().into_string().map_err(|_| unavailable())?;
            if entries.len() >= 256
                || name.chars().any(bridget_core::is_disallowed_control)
                || !entry.file_type().map_err(|_| unavailable())?.is_file()
            {
                return Err(unavailable());
            }
            entries.push((name, permission_source_revision(&entry.path(), max_bytes)?));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let values = entries
            .into_iter()
            .map(|(name, revision)| serde_json::json!({"name":name,"revision":revision}))
            .collect::<Vec<_>>();
        return Ok(format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&values).map_err(|_| unavailable())?)
        ));
    }
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(unavailable());
    }
    let mut file = std::fs::File::open(path).map_err(|_| unavailable())?;
    let mut digest = Sha256::new();
    let mut bytes = [0u8; 65536];
    let mut consumed = 0u64;
    loop {
        let n = file.read(&mut bytes).map_err(|_| unavailable())?;
        if n == 0 {
            break;
        }
        consumed = consumed.saturating_add(n as u64);
        if consumed > max_bytes {
            return Err(unavailable());
        }
        digest.update(&bytes[..n]);
    }
    Ok(format!("sha256:{:x}", digest.finalize()))
}

// Native Claude refreshes lastUpdated during startup. Preserve all other
// marketplace fields and the canonical file binding, not this refresh clock.
pub fn permission_marketplace_revision171(path: &std::path::Path, max_bytes: u64) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let fail = || "permission_source_unavailable".to_owned();
    let before = permission_source_revision_v2(path, max_bytes)?;
    if before == "absent" { return Ok(before); }
    let m = std::fs::symlink_metadata(path).map_err(|_| fail())?;
    if !m.is_file() || std::fs::canonicalize(path).map_err(|_| fail())? != path {
        return Ok(before); // Preserve the existing exact contract for links.
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path).map_err(|_| fail())?.take(max_bytes + 1)
        .read_to_end(&mut bytes).map_err(|_| fail())?;
    if bytes.len() as u64 > max_bytes { return Err(fail()); }
    let mut value: Value = serde_json::from_slice(&bytes).map_err(|_| fail())?;
    for row in value.as_object_mut().ok_or_else(fail)?.values_mut() {
        let row = row.as_object_mut().ok_or_else(fail)?;
        if let Some(stamp) = row.get("lastUpdated") {
            if !stamp.as_str().is_some_and(|s| s.len() <= 64) { return Err(fail()); }
            row.remove("lastUpdated");
        }
    }
    if permission_source_revision_v2(path, max_bytes)? != before {
        return Err("settings_revision_changed".into());
    }
    let binding = serde_json::json!({"kind":"native_marketplace171", "path":path, "value":value});
    Ok(format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(&binding).map_err(|_| fail())?)))
}

pub fn permission_source_revision_v2(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let unavailable = || "permission_source_unavailable".to_string();
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok("absent".into()),
        Err(_) => return Err(unavailable()),
    };
    if metadata.file_type().is_symlink() {
        return symlink_permission_source_revision_v2(path, max_bytes, 0, &mut Vec::new());
    }
    if metadata.is_dir() {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|_| unavailable())? {
            let entry = entry.map_err(|_| unavailable())?;
            let name = entry.file_name().into_string().map_err(|_| unavailable())?;
            if entries.len() >= 256
                || name.chars().any(bridget_core::is_disallowed_control)
                || (!entry.file_type().map_err(|_| unavailable())?.is_file()
                    && !entry.file_type().map_err(|_| unavailable())?.is_symlink())
            {
                return Err(unavailable());
            }
            entries.push((
                name,
                permission_source_revision_v2(&entry.path(), max_bytes)?,
            ));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let values = entries
            .into_iter()
            .map(|(name, revision)| serde_json::json!({"name":name,"revision":revision}))
            .collect::<Vec<_>>();
        return Ok(format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&values).map_err(|_| unavailable())?)
        ));
    }
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(unavailable());
    }
    let canonical = std::fs::canonicalize(path).map_err(|_| unavailable())?;
    let mut file = std::fs::File::open(path).map_err(|_| unavailable())?;
    let mut digest = Sha256::new();
    let mut bytes = [0u8; 65536];
    let mut consumed = 0u64;
    loop {
        let n = file.read(&mut bytes).map_err(|_| unavailable())?;
        if n == 0 {
            break;
        }
        consumed = consumed.saturating_add(n as u64);
        if consumed > max_bytes {
            return Err(unavailable());
        }
        digest.update(&bytes[..n]);
    }
    use std::os::unix::fs::MetadataExt;
    let after = std::fs::symlink_metadata(path).map_err(|_| unavailable())?;
    if !after.is_file()
        || metadata.dev() != after.dev()
        || metadata.ino() != after.ino()
        || metadata.len() != after.len()
        || metadata.mtime() != after.mtime()
        || metadata.mtime_nsec() != after.mtime_nsec()
        || metadata.ctime() != after.ctime()
        || metadata.ctime_nsec() != after.ctime_nsec()
        || std::fs::canonicalize(path).map_err(|_| unavailable())? != canonical
    {
        return Err("settings_revision_changed".into());
    }
    let revision = format!("sha256:{:x}", digest.finalize());
    if canonical == path {
        return Ok(revision);
    }
    let binding = serde_json::json!({
        "content_revision": revision, "kind": "resolved",
        "logical_path": path.to_str().ok_or_else(unavailable)?,
        "target_path": canonical.to_str().ok_or_else(unavailable)?,
    });
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(&binding).map_err(|_| unavailable())?)
    ))
}

pub fn permission_hook_source_revision(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::os::unix::fs::MetadataExt;
    let unavailable = || "permission_source_unavailable".to_string();
    let before = std::fs::metadata(path).map_err(|_| unavailable())?;
    if !before.is_file() {
        return Err(unavailable());
    }
    let content = permission_source_revision_v2(path, max_bytes)?;
    let after = std::fs::metadata(path).map_err(|_| unavailable())?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mode() != after.mode()
        || before.uid() != after.uid()
        || before.gid() != after.gid()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
    {
        return Err("settings_revision_changed".into());
    }
    let facts = serde_json::json!({
        "content_revision": content, "gid": before.gid(),
        "mode": before.mode() & 0o7777, "uid": before.uid(),
    });
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(&facts).map_err(|_| unavailable())?)
    ))
}

fn symlink_permission_source_revision_v2(
    path: &std::path::Path,
    max_bytes: u64,
    depth: u8,
    seen: &mut Vec<std::path::PathBuf>,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let unavailable = || "permission_source_unavailable".to_string();
    if depth >= 8 || seen.iter().any(|entry| entry == path) {
        return Err(unavailable());
    }
    let target = std::fs::read_link(path).map_err(|_| unavailable())?;
    let resolved = if target.is_absolute() {
        target.clone()
    } else {
        path.parent().ok_or_else(unavailable)?.join(&target)
    };
    let metadata = std::fs::symlink_metadata(&resolved).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "permission_source_dangling".to_string()
        } else {
            unavailable()
        }
    })?;
    let canonical = std::fs::canonicalize(&resolved).map_err(|_| unavailable())?;
    seen.push(path.to_path_buf());
    let content_revision = if metadata.is_dir() {
        return Err(unavailable());
    } else if metadata.file_type().is_symlink() {
        symlink_permission_source_revision_v2(&resolved, max_bytes, depth + 1, seen)?
    } else {
        permission_source_revision_v2(&resolved, max_bytes)?
    };
    seen.pop();
    if std::fs::read_link(path).map_err(|_| unavailable())? != target
        || std::fs::canonicalize(&resolved).map_err(|_| unavailable())? != canonical
    {
        return Err("settings_revision_changed".into());
    }
    let value = serde_json::json!({
        "link_target": target.to_str().ok_or_else(unavailable)?,
        "kind": "symlink",
        "logical_path": path.to_string_lossy(),
        "target_path": canonical.to_string_lossy(),
        "content_revision": content_revision,
    });
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(&value).map_err(|_| unavailable())?)
    ))
}

fn canonical_json(value: &Value) -> Result<Vec<u8>, serde_json::Error> {
    match value {
        Value::Null => Ok(b"null".to_vec()),
        Value::Bool(value) => Ok(value.to_string().into_bytes()),
        Value::Number(value) => Ok(value.to_string().into_bytes()),
        Value::String(value) => serde_json::to_vec(value),
        Value::Array(values) => {
            let mut output = Vec::from([b'[']);
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                output.extend(canonical_json(value)?);
            }
            output.push(b']');
            Ok(output)
        }
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort();
            let mut output = Vec::from([b'{']);
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                output.extend(serde_json::to_vec(key)?);
                output.push(b':');
                output.extend(canonical_json(&values[*key])?);
            }
            output.push(b'}');
            Ok(output)
        }
    }
}


/// Private 171 witness. Equality concerns every declared source, never a parent
/// digest alone. No executable content or command text is rewritten.
fn equivalence_keys(value: &Value, names: &[&str]) -> bool {
    value.as_object().is_some_and(|o| o.len() == names.len()
        && names.iter().all(|name| o.contains_key(*name)))
}
fn equivalence_digest_value(value: &Value) -> bool {
    value.as_str().is_some_and(|s| s.len() == 71 && s.starts_with("sha256:")
        && s[7..].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}
fn equivalence_path(value: &Value) -> bool {
    value.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 4096
        && !s.chars().any(char::is_control) && std::path::Path::new(s).is_absolute()
        && !std::path::Path::new(s).components().any(|c| matches!(c, std::path::Component::ParentDir)))
}
fn equivalence_manifest_valid(manifest: &Value) -> bool {
    if !equivalence_keys(manifest, &["project_root", "settings_sources", "settings_overrides", "rows"])
        || !equivalence_path(&manifest["project_root"]) { return false; }
    let selected = &manifest["settings_sources"];
    if selected != "provider_default" && !selected.as_array().is_some_and(|a|
        a.len() <= 3 && a.iter().all(|v| matches!(v.as_str(), Some("user" | "project" | "local")))
        && a.iter().enumerate().all(|(i, v)| !a[..i].contains(v))) { return false; }
    // Inline overrides are not silently merged or promoted into remote evidence.
    if !manifest["settings_overrides"].is_null() { return false; }
    manifest["rows"].as_array().is_some_and(|rows| rows.len() <= 128 && rows.iter().all(|row| {
        if !equivalence_keys(row, &["kind", "path", "revision", "selector", "content_revision", "mode", "owner", "dependencies"])
            || !matches!(row["kind"].as_str(), Some("user" | "project" | "local" | "managed" | "cli" | "hook"))
            || !equivalence_path(&row["path"]) { return false; }
        if row["revision"] == "absent" {
            return row["content_revision"] == "absent" && row["selector"].is_null()
                && row["mode"] == 0 && row["owner"] == "absent"
                && row["dependencies"].as_array().is_some_and(Vec::is_empty);
        }
        equivalence_digest_value(&row["revision"]) && equivalence_digest_value(&row["content_revision"])
            && row["selector"].as_str().is_some_and(|s| !s.is_empty() && s.len() <= 4096 && !s.chars().any(char::is_control))
            && row["mode"].as_u64().is_some_and(|m| m <= 0o7777)
            && matches!(row["owner"].as_str(), Some("profile" | "system"))
            && row["dependencies"].as_array().is_some_and(|deps| deps.len() <= 16 && deps.iter().all(|dep|
                equivalence_keys(dep, &["path", "revision", "mode", "owner"])
                && equivalence_path(&dep["path"]) && equivalence_digest_value(&dep["revision"])
                && dep["mode"].as_u64().is_some_and(|m| m <= 0o7777) && dep["owner"] == "system"))
    }))
}

/// Exhaustive accounting: absent sources remain explicit pairs, not discarded.
/// Present sources need a unique native selector and byte-identical controls,
/// modes, owner role and declared dependency revisions on the two hosts.
pub fn permission_source_equivalence_pairs(origin: &Value, target: &Value) -> Result<Value, String> {
    let fail = || "permission_mapping_unavailable".to_owned();
    if !equivalence_manifest_valid(origin) || !equivalence_manifest_valid(target)
        || origin["settings_sources"] != target["settings_sources"]
        || origin["settings_overrides"] != target["settings_overrides"] { return Err(fail()); }
    let a = origin["rows"].as_array().ok_or_else(fail)?;
    let b = target["rows"].as_array().ok_or_else(fail)?;
    let mut used = std::collections::BTreeSet::new();
    let mut selectors = std::collections::BTreeSet::new();
    let mut pairs = Vec::new();
    for (i, row) in a.iter().enumerate() {
        if row["revision"] == "absent" {
            pairs.push(serde_json::json!({"origin":i,"target":null,"disposition":"origin_absent"}));
            continue;
        }
        let selector = (row["kind"].as_str().ok_or_else(fail)?, row["selector"].as_str().ok_or_else(fail)?);
        if !selectors.insert(selector) { return Err(fail()); }
        let matches = b.iter().enumerate().filter(|(_, r)| r["revision"] != "absent"
            && r["kind"] == row["kind"] && r["selector"] == row["selector"]).collect::<Vec<_>>();
        if matches.len() != 1 { return Err(fail()); }
        let (j, other) = matches[0];
        if !used.insert(j) || ["content_revision", "mode", "owner", "dependencies"].iter().any(|key| row[*key] != other[*key]) {
            return Err(fail());
        }
        pairs.push(serde_json::json!({"origin":i,"target":j,"disposition":"equal"}));
    }
    for (j, row) in b.iter().enumerate() {
        if used.contains(&j) { continue; }
        if row["revision"] != "absent" { return Err(fail()); }
        pairs.push(serde_json::json!({"origin":null,"target":j,"disposition":"target_absent"}));
    }
    Ok(Value::Array(pairs))
}

pub fn validate_permission_source_equivalence(context: &Value) -> Result<(), String> {
    let fail = || "permission_attestation_unavailable".to_owned();
    let Some(witness) = context.get("source_equivalence") else {
        if matches!(context["hook_execution"].as_str(), Some("native_remote_equivalent" | "native_remote_guarded")) { return Err(fail()); }
        return Ok(());
    };
    if witness["version"] == 2 { return super::native_hook_profile171::validate_guarded_hook_witness(context); }
    if context["attestation_version"] != 2 || witness["version"] != 1 { return Err(fail()); }
    let manifest = match witness["scope"].as_str() {
        Some("origin") => {
            if !equivalence_keys(witness, &["version", "scope", "manifest"])
                || context["hook_execution"] != "native_same_host" { return Err(fail()); }
            &witness["manifest"]
        }
        Some("remote") => {
            if !equivalence_keys(witness, &["version", "scope", "origin", "target", "binding", "pairs"])
                || context["hook_execution"] != "native_remote_equivalent" { return Err(fail()); }
            let binding = &witness["binding"];
            if !equivalence_keys(binding, &["parent_revision", "parent_run_id", "parent_session_id", "parent_instance_id",
                "parent_digest", "child_policy_digest", "source_host", "source_root", "remote_root", "mapping_revision", "source_identity"])
                || !binding["parent_revision"].as_u64().is_some_and(|v| v > 0)
                || !binding["mapping_revision"].as_u64().is_some_and(|v| v > 0)
                || !equivalence_path(&binding["source_root"]) || !equivalence_path(&binding["remote_root"])
                || !equivalence_digest_value(&binding["parent_digest"]) || !equivalence_digest_value(&binding["child_policy_digest"])
                || ["parent_run_id","parent_session_id","parent_instance_id","source_host","source_identity"].iter().any(|key|
                    !binding[*key].as_str().is_some_and(|s| !s.is_empty() && s.len() <= 4096 && !s.chars().any(char::is_control)))
                || witness["origin"]["project_root"] != binding["source_root"]
                || witness["target"]["project_root"] != binding["remote_root"]
                || permission_source_equivalence_pairs(&witness["origin"], &witness["target"]).map_err(|_| fail())? != witness["pairs"] {
                return Err(fail());
            }
            &witness["target"]
        }
        _ => return Err(fail()),
    };
    if !equivalence_manifest_valid(manifest)
        || manifest["settings_sources"] != context["settings_sources"]
        || manifest["settings_overrides"] != context.get("settings_overrides").cloned().unwrap_or(Value::Null) { return Err(fail()); }
    let rows = manifest["rows"].as_array().ok_or_else(fail)?;
    let sources = context["permission_sources"].as_array().ok_or_else(fail)?;
    if rows.len() != sources.len() || rows.iter().zip(sources).any(|(row, source)|
        ["kind","path","revision"].iter().any(|key| row[*key] != source[*key])) {
        return Err(fail());
    }
    Ok(())
}

#[cfg(test)]
mod executable_revision183_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn native183_cli_unique_lu_une_fois_par_controle_et_attentes_distinctes() {
        let mut context = serde_json::json!({
            "cli_path":"/cli", "cli_revision":"revision-a",
            "resolved_cli_path":"/cli", "resolved_cli_revision":"revision-a"
        });
        let mut reads = 0;
        recheck_cli_revisions(&context, |path| {
            assert_eq!(path, Path::new("/cli"));
            reads += 1;
            Ok("revision-a".into())
        }).unwrap();
        assert_eq!(reads, 1);
        context["resolved_cli_revision"] = Value::from("revision-b");
        assert_eq!(recheck_cli_revisions(&context, |_| Ok("revision-a".into())),
            Err("settings_revision_changed".into()));
        context["resolved_cli_path"] = Value::from("/wrapper-target");
        let mut paths = Vec::new();
        recheck_cli_revisions(&context, |path| {
            paths.push(path.to_path_buf());
            Ok(if path == Path::new("/cli") { "revision-a" } else { "revision-b" }.into())
        }).unwrap();
        assert_eq!(paths, [Path::new("/cli"), Path::new("/wrapper-target")]);
    }

    #[test]
    fn native183_cli_modifie_entre_deux_controles_reste_refuse() {
        let path = std::env::temp_dir().join(format!("bridget-cli183-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"original").unwrap();
        let revision = permission_source_revision(&path, 512).unwrap();
        let context = serde_json::json!({
            "cli_path":path, "cli_revision":revision,
            "resolved_cli_path":path, "resolved_cli_revision":revision
        });
        recheck_cli_revisions(&context, |p| permission_source_revision(p, 512)).unwrap();
        std::fs::write(&path, b"modified").unwrap();
        assert_eq!(recheck_cli_revisions(&context, |p| permission_source_revision(p, 512)),
            Err("settings_revision_changed".into()));
        std::fs::remove_file(&path).unwrap();
        assert_eq!(recheck_cli_revisions(&context, |p| permission_source_revision(p, 512)),
            Err("settings_revision_changed".into()));
    }
}
