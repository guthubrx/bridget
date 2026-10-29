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
        if let Some(context) = parent.provider_policy.get("launch_context") {
            bridget_transport::protocol::recheck_permission_context_sources(context)?;
        } else if !authenticated_t3_tool_scope171(parent) {
            // T3 attests native SDK rights without a file-transplant context.
            // A managed wrapper still owes its frozen launch/source evidence.
            return Err(unavailable());
        }
        // The origin attests the caller's actual sources. A remote provider
        // uses its own configuration; no equivalence of personal installations
        // is required to authenticate this caller.
    }
    use sha2::{Digest, Sha256};
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(parent).map_err(|_| unavailable())?)
    ))
}

// Only the two SDK-injected read rules observed on the authenticated T3 port.
// This is an exact closed value, never a prefix-based MCP permission grant.
fn sdk_settings171() -> Value {
    json!({"permissions":{"allow":["mcp__bridget__bridget_capabilities","mcp__bridget__bridget_task_status"]}})
}

struct AuthenticatedSdkInputs171<'a> { settings: &'a Value, environment: &'a Value }
impl<'a> AuthenticatedSdkInputs171<'a> {
    fn from_parent(parent: &'a ProviderPermissions) -> Result<Option<Self>, String> {
        let Some(settings) = parent.provider_policy.pointer("/launch_context/settings_overrides") else { return Ok(None); };
        let p = &parent.provider_policy;
        if p.pointer("/launch_context/source_equivalence/version") != Some(&json!(2))
            || p.pointer("/launch_context/source_equivalence/scope") != Some(&json!("origin")) {
            return Ok(None); // Legacy CLI-derived overrides keep their existing path.
        }
        bridget_transport::protocol::validate_permissions(parent, true)?;
        if parent.source != "provider_turn" || parent.driver != "claude_stream_json"
            || parent.run_id.is_empty() || parent.provider_session_id.is_empty() || parent.provider_instance_id.is_empty()
            || p["permission_callback"] != json!({"kind":"t3_runtime","tool_approval":"allow","plan_exit":"deny"})
            || p["allowed_tools"] != json!(["mcp__t3-code__*"])
            || p.pointer("/launch_context/attestation_version") != Some(&json!(2))
            || p.pointer("/launch_context/hook_execution") != Some(&json!("native_same_host"))
            || p.pointer("/launch_context/source_equivalence/version") != Some(&json!(2))
            || p.pointer("/launch_context/source_equivalence/scope") != Some(&json!("origin"))
            || *settings != sdk_settings171()
        { return Err("permission_mapping_unavailable".into()); }
        Ok(Some(Self { settings, environment: &p["launch_context"]["source_equivalence"]["profile"]["origin_environment"] }))
    }
}

// Le port T3 atteste aussi les sessions natives sans profil de transplantation.
// Conserver sa règle exacte, sans donner au fils une connexion T3 du parent.
fn authenticated_t3_tool_scope171(parent: &ProviderPermissions) -> bool {
    parent.source == "provider_turn" && parent.driver == "claude_stream_json"
        && !parent.run_id.is_empty() && !parent.provider_session_id.is_empty()
        && !parent.provider_instance_id.is_empty()
        && bridget_transport::protocol::validate_permissions(parent, true).is_ok()
        && parent.provider_policy["kind"] == "claude"
        && parent.provider_policy["permission_callback"]
            == json!({"kind":"t3_runtime","tool_approval":"allow","plan_exit":"deny"})
}

/// Cas fermé du parent natif T3 : sa seule règle locale d’approbation MCP
/// n’est pas une restriction sur les outils de fichiers et n’est jamais exportée.
/// Les profils de hooks, règles privées et callbacks tiers restent non traduits.
fn native_t3_codex_scope214(parent: &ProviderPermissions, definition: &AgentDefinition) -> bool {
    let policy = &parent.provider_policy;
    authenticated_t3_tool_scope171(parent)
        && definition.protocol == "codex_app_server"
        && definition.mcp.interactive == "codex"
        && parent.runtime_mode == "full-access"
        && policy["permission_mode"] == "bypassPermissions"
        && policy["tools"] == json!({"type":"preset","preset":"claude_code"})
        && policy["allowed_tools"] == json!(["mcp__t3-code__*"])
        && policy.get("disallowed_tools").is_none_or(|v| v.as_array().is_some_and(Vec::is_empty))
        && policy["settings_sources"] == "provider_default"
        && policy.get("launch_context").is_none()
}

fn require_isolated_claude_mount171(definition: &AgentDefinition) -> Result<(), String> {
    if definition.protocol != "claude_stream_json" || definition.mcp.interactive != "claude"
        || definition.args.iter().any(|a| matches!(a.split('=').next(), Some("--mcp-config" | "--strict-mcp-config"))) {
        return Err("permission_mapping_unavailable".into());
    }
    Ok(())
}

fn portable_claude_rules(parent: &ProviderPermissions) -> bool {
    let policy = &parent.provider_policy;
    let sdk = match AuthenticatedSdkInputs171::from_parent(parent) { Ok(value) => value, Err(_) => return false };
    ["allowed_tools", "disallowed_tools"].iter().all(|field| {
        policy.get(*field).is_none_or(|value| value.as_array().is_some_and(|rules| rules.iter().all(|rule| {
            rule.as_str().is_some_and(|name| !name.is_empty() &&
                (name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                 || (*field == "allowed_tools" && authenticated_t3_tool_scope171(parent) && name == "mcp__t3-code__*")))
        })))
    }) && (sdk.is_some() || policy.pointer("/launch_context/settings_overrides").is_none())
}

// The existing managed wrapper mounts only its child-scoped Bridget server,
// using --strict-mcp-config. T3 names remain unchanged and cannot select that
// server. No parent T3 credential or authority is forwarded to the child.
fn require_sdk_native_mount171(context: &Value, definition: &AgentDefinition) -> Result<(), String> {
    if context.get("settings_overrides").is_none() { return Ok(()); }
    if context["settings_overrides"] != sdk_settings171()
        || definition.protocol != "claude_stream_json" || definition.mcp.interactive != "claude"
        || definition.args.iter().any(|a| matches!(a.split('=').next(), Some("--mcp-config" | "--strict-mcp-config")))
    { return Err("permission_mapping_unavailable".into()); }
    Ok(())
}

fn recheck_parent_context171(parent: &ProviderPermissions, definition: &AgentDefinition,
    cwd: &Path, source: &SourceEnvironment) -> Result<(), String> {
    let sdk = AuthenticatedSdkInputs171::from_parent(parent)?;
    let context = parent.provider_policy.get("launch_context").ok_or_else(unavailable)?;
    let mut environment = build_environment(definition, source).map_err(|_| changed())?;
    if let Some(sdk) = &sdk {
        // These are the authenticated parent's three closed hook facts, not
        // the target definition's launch environment. The same-operation
        // verified profile checks every value again before reconstruction.
        for key in ["TMUX", "TMUX_PANE", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"] {
            match &sdk.environment[key] {
                Value::Null => { environment.remove(key); }
                Value::String(value) => { environment.insert(key.into(), std::ffi::OsString::from(value.as_str())); }
                _ => return Err("permission_hook_environment_unavailable".into()),
            }
        }
    }
    let mut operation = PermissionReadOperation::default();
    recheck_context_with_sdk171(context, definition, cwd, &environment, &mut operation, sdk.as_ref())?;
    operation.finish()
}

fn parent_recheck_environment171(parent: &ProviderPermissions, definition: &AgentDefinition,
    source: &SourceEnvironment) -> Result<SourceEnvironment, String> {
    let sdk = AuthenticatedSdkInputs171::from_parent(parent)?;
    parent.provider_policy.get("launch_context").ok_or_else(unavailable)?;
    let mut environment = build_environment(definition, source).map_err(|_| changed())?;
    if let Some(sdk) = &sdk {
        for key in ["TMUX", "TMUX_PANE", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"] {
            match &sdk.environment[key] {
                Value::Null => { environment.remove(key); }
                Value::String(value) => { environment.insert(key.into(), std::ffi::OsString::from(value.as_str())); }
                _ => return Err("permission_hook_environment_unavailable".into()),
            }
        }
    }
    Ok(environment)
}

fn recheck_parent_context_in_operation171(parent: &ProviderPermissions, definition: &AgentDefinition,
    cwd: &Path, environment: &SourceEnvironment, operation: &mut PermissionReadOperation) -> Result<(), String> {
    let sdk = AuthenticatedSdkInputs171::from_parent(parent)?;
    let context = parent.provider_policy.get("launch_context").ok_or_else(unavailable)?;
    recheck_context_with_sdk171(context, definition, cwd, environment, operation, sdk.as_ref())
}

/// A stack-owned catalogue scope. It shares only the opaque verified profile,
/// never an Ok policy/recheck from one definition with another definition.
/// Every entry still builds its own environment and recaptures its native CLI,
/// settings, sources and SDK inputs. Different effective environments cannot
/// share a token. The full authenticated parent and cwd are exact bindings.
pub(crate) struct CataloguePermissionRead171 {
    binding: Option<(ProviderPermissions, PathBuf)>,
    groups: Vec<(SourceEnvironment, PermissionReadOperation)>,
}
impl CataloguePermissionRead171 {
    pub(crate) fn new(parent: Option<&ProviderPermissions>, cwd: Option<&Path>) -> Self {
        Self { binding: parent.zip(cwd).map(|(parent, cwd)| (parent.clone(), cwd.to_owned())), groups: Vec::new() }
    }
    fn recheck(&mut self, parent: &ProviderPermissions, definition: &AgentDefinition,
        cwd: &Path, source: &SourceEnvironment) -> Result<(), String> {
        bridget_transport::protocol::validate_permissions(parent, parent.source == "provider_turn")?;
        if self.binding.as_ref().is_none_or(|(bound, root)| bound != parent || root != cwd) {
            return Err("permission_attestation_unavailable".into());
        }
        let environment = parent_recheck_environment171(parent, definition, source)?;
        let position = match self.groups.iter().position(|(bound, _)| bound == &environment) {
            Some(position) => position,
            None => {
                if self.groups.len() >= 128 { return Err(unavailable()); }
                self.groups.push((environment.clone(), PermissionReadOperation::default()));
                self.groups.len() - 1
            }
        };
        // No cross-definition cached Result: each definition takes every native
        // comparison. Only profile verification reads are operation-local.
        recheck_parent_context_in_operation171(parent, definition, cwd, &environment, &mut self.groups[position].1)
    }
    /// Consuming the scope gates publication. Any fresh finish failure returns
    /// an error instead of publishing an already-built favourable catalogue.
    pub(crate) fn finish<T>(mut self, response: T) -> Result<T, String> {
        for (_, operation) in &mut self.groups { operation.finish()?; }
        // Legacy sources have no opaque profile. They keep their existing
        // per-entry checks and receive a fresh source check before publication.
        if !self.groups.is_empty() {
            let (parent, _) = self.binding.as_ref().ok_or_else(unavailable)?;
            let context = parent.provider_policy.get("launch_context").ok_or_else(unavailable)?;
            if context.pointer("/source_equivalence/version") != Some(&json!(2)) {
                bridget_transport::protocol::recheck_permission_context_sources(context)?;
            }
        }
        Ok(response)
    }
}

fn mapped_permission_suffix171<'a>(original: &'a Path, source_root: &Path) -> Result<&'a Path, String> {
    if original.components().any(|part| matches!(part, std::path::Component::ParentDir)) {
        return Err("permission_mapping_unavailable".into());
    }
    original.strip_prefix(source_root).map_err(|_| "permission_mapping_unavailable".into())
}

fn mapped_permission_directory171(mapping: &crate::registry::ProjectMapping, original: &Path) -> Result<PathBuf, String> {
    crate::registry::validate_additional_mapping171(mapping).map_err(|_| "permission_mapping_unavailable")?;
    if let Ok(suffix) = mapped_permission_suffix171(original, &mapping.source_root) {
        return Ok(mapping.remote_root.join(suffix));
    }
    if let Some(extra) = &mapping.additional_root {
        let suffix = mapped_permission_suffix171(original, &extra.source_root)?;
        return Ok(extra.remote_root.join(suffix));
    }
    Err("permission_mapping_unavailable".into())
}

fn mapped_existing_permission_directory171(mapping: &crate::registry::ProjectMapping,
    original: &Path, protected: &[PathBuf]) -> Result<PathBuf, String> {
    let root = crate::registry::validate_unprotected_local_path(&mapping.remote_root, protected)?;
    let selected = mapped_permission_directory171(mapping, original)?;
    let translated = crate::registry::validate_unprotected_local_path(&selected, protected)?;
    let permitted = translated.starts_with(&root) || mapping.additional_root.as_ref()
        .is_some_and(|extra| translated.starts_with(&extra.remote_root));
    if !permitted || !translated.is_dir() { return Err("permission_mapping_unavailable".into()); }
    Ok(translated)
}

fn project_policy_directories171(policy: &mut Value, mapping: &crate::registry::ProjectMapping,
    protected: &[PathBuf]) -> Result<(), String> {
    let mut projected = policy.clone();
    for pointer in ["/sandbox_policy/writableRoots", "/additional_directories"] {
        if let Some(paths) = projected.pointer_mut(pointer) {
            let mapped = paths.as_array().ok_or("permission_mapping_unavailable")?.iter()
                .map(|path| mapped_existing_permission_directory171(mapping,
                    Path::new(path.as_str().ok_or("permission_mapping_unavailable")?), protected))
                .collect::<Result<Vec<_>, String>>()?;
            *paths = serde_json::to_value(mapped).map_err(|_| unavailable())?;
        }
    }
    *policy = projected;
    Ok(())
}

fn verify_permission_directory171(path: &Path, uid: u32, mode: u32, protected: &[PathBuf]) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let canonical = crate::registry::validate_unprotected_local_path(path, protected)?;
    if canonical != path || uid != unsafe { libc::geteuid() } { return Err("permission_mapping_unavailable".into()); }
    for ancestor in path.ancestors() {
        let m = std::fs::symlink_metadata(ancestor).map_err(|_| "permission_mapping_unavailable")?;
        if !m.is_dir() || m.file_type().is_symlink()
            || std::fs::canonicalize(ancestor).map_err(|_| "permission_mapping_unavailable")? != ancestor {
            return Err("permission_mapping_unavailable".into());
        }
    }
    let m = std::fs::symlink_metadata(path).map_err(|_| "permission_mapping_unavailable")?;
    if m.uid() != uid || m.mode() & 0o7777 != mode { return Err("permission_mapping_unavailable".into()); }
    Ok(())
}

fn check_profile_additional_root171(profile: &Value, mapping: &crate::registry::ProjectMapping) -> Result<(), String> {
    let expected = mapping.additional_root.as_ref().map(serde_json::to_value).transpose().map_err(|_| "permission_hook_equivalence_unavailable".to_owned())?;
    if profile["mapping"].get("additional_root") != expected.as_ref()
        || profile["mapping"]["mapping_revision"] != mapping.mapping_revision
        || profile["mapping"]["source_host"] != mapping.source_host
        || profile["mapping"]["source_root"] != json!(mapping.source_root)
        || profile["mapping"]["remote_root"] != json!(mapping.remote_root)
        || profile["mapping"]["source_identity"] != mapping.expected_git_identity {
        return Err("permission_hook_equivalence_unavailable".to_owned());
    }
    Ok(())
}

fn check_additional_registry_authority171(profile: &Value, registry: &crate::registry::AgentRegistry) -> Result<(), String> {
    let matches: Vec<_> = registry.machines().values().filter(|machine| machine.enabled)
        .flat_map(|machine| machine.mappings.values())
        .filter(|mapping| profile["mapping"]["source_host"] == mapping.source_host
            && profile["mapping"]["source_root"] == json!(mapping.source_root)
            && profile["mapping"]["remote_root"] == json!(mapping.remote_root)
            && profile["mapping"]["source_identity"] == mapping.expected_git_identity
            && profile["mapping"]["mapping_revision"] == mapping.mapping_revision).collect();
    if matches.len() != 1 { return Err("permission_hook_equivalence_unavailable".into()); }
    check_profile_additional_root171(profile, matches[0])
}

fn check_parent_additional_root171(parent: &ProviderPermissions, profile: &Value) -> Result<(), String> {
    if let Some(extra) = profile["mapping"].get("additional_root") {
        let root = extra["source_root"].as_str().ok_or_else(|| "permission_hook_equivalence_unavailable".to_owned())?;
        if !parent.provider_policy["additional_directories"].as_array().is_some_and(|paths|
            paths.iter().filter(|path| path.as_str() == Some(root)).count() == 1) {
            return Err("permission_hook_equivalence_unavailable".to_owned());
        }
    }
    Ok(())
}

/// Projection sur le worker destinataire, après preuve parent authentifiée par
/// le maître. Aucun fichier d'origine n'est lu ici. Le mapping est lui-même une
/// autorité administrative revalidée par le caller, pas une donnée du modèle.
/// Une seule sonde groupée ou phase native. Aucun résultat ne survit à cette
/// valeur locale ; chaque frontière relit intégralement les dépendances.
#[derive(Default)]
pub(crate) struct PermissionReadOperation {
    profiles: Vec<(std::sync::Arc<bridget_transport::protocol::VerifiedGuardedHookProfile>, Value)>,
}
impl PermissionReadOperation {
    fn profile(&mut self, context: &Value) -> Result<std::sync::Arc<bridget_transport::protocol::VerifiedGuardedHookProfile>, String> {
        if let Some((verified, last)) = self.profiles.iter_mut().find(|(v,_)| v.same_profile(context)) {
            verified.profile(context)?;
            *last = context.clone();
            return Ok(verified.clone());
        }
        if self.profiles.len() >= 128 { return Err(equivalence_fail_at(ProjectionGuard::G002)); }
        let verified = std::sync::Arc::new(bridget_transport::protocol::VerifiedGuardedHookProfile::verify(context)?);
        verified.profile(context)?;
        self.profiles.push((verified.clone(), context.clone()));
        Ok(verified)
    }
    pub(crate) fn finish(&mut self) -> Result<(), String> {
        for (_, context) in self.profiles.drain(..) {
            bridget_transport::protocol::recheck_guarded_hook_profile(&context)?;
        }
        Ok(())
    }
}

pub(crate) fn project_remote_permissions(
    parent: &ProviderPermissions,
    definition: &AgentDefinition,
    mapping: &crate::registry::ProjectMapping,
    execution_cwd: &Path,
    source: &SourceEnvironment,
    posture: Option<SpawnPosture>,
    protected: &[PathBuf],
) -> Result<(Value, SpawnPosture), String> {
    let mut operation = PermissionReadOperation::default();
    let result = project_remote_permissions_in_operation(parent, definition, mapping, execution_cwd, source, posture, protected, &mut operation)?;
    operation.finish()?;
    Ok(result)
}

pub(crate) fn project_remote_permissions_in_operation(
    parent: &ProviderPermissions,
    definition: &AgentDefinition,
    mapping: &crate::registry::ProjectMapping,
    execution_cwd: &Path,
    source: &SourceEnvironment,
    posture: Option<SpawnPosture>,
    protected: &[PathBuf],
    operation: &mut PermissionReadOperation,
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
    let project_path = |origin: &str| mapped_existing_permission_directory171(mapping, Path::new(origin), protected);
    mapped_permission_suffix171(Path::new(&parent.cwd), &mapping.source_root)?;
    if let Some(extra) = &mapping.additional_root {
        crate::registry::validate_additional_mapping171(mapping)?;
        verify_permission_directory171(&extra.remote_root, extra.remote_uid, extra.remote_mode, protected)?;
    }
    let cwd = crate::registry::validate_unprotected_local_path(execution_cwd, protected)?;
    if !cwd.starts_with(project_path(&parent.cwd)?) || !cwd.is_dir() {
        return Err("permission_mapping_unavailable".into());
    }
    let mut translated = parent.clone();
    translated.cwd = cwd.to_string_lossy().into_owned();
    // Explicit tool restrictions remain portable constraints. Personal launch
    // settings, plugins and source paths are not copied to another machine.
    if parent.driver == "claude_stream_json" && !portable_claude_rules(parent) {
        return Err("permission_mapping_unavailable".into());
    }
    if parent.driver == "claude_stream_json"
        && parent.provider_policy["allowed_tools"].as_array().is_some_and(|rules|
            rules.iter().any(|rule| rule == "mcp__t3-code__*")) {
        if !native_t3_codex_scope214(parent, definition) {
            require_isolated_claude_mount171(definition)?;
        }
    }
    project_policy_directories171(&mut translated.provider_policy, mapping, protected)?;
    let prepared = if definition.protocol == "claude_stream_json" {
        Some(capture_native_target_context(definition, &cwd, source)?)
    } else { None };
    let (mut policy, effective) =
        child_policy_reader_with_capture(&translated, definition, &cwd, source, true, prepared)(posture)?;
    if definition.protocol == "claude_stream_json" {
        merge_native_target_arguments(&mut policy, definition, &protected)?;
    }
    if definition.protocol == "claude_stream_json" && effective == SpawnPosture::Development {
        // The destination's configured mode is authoritative. A full-access
        // parent cannot turn a normally restricted destination into a bypass.
        let mode = native_target_mode(definition)?;
        if mode == "plan" { return Err("permission_not_inherited".into()); }
        policy["permission_mode"] = json!(mode);
    }
    recheck_remote_child_permissions(
        &policy,
        definition,
        &cwd,
        source,
        protected,
        Some(mapping),
        Some(parent),
    )?;
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
    mapping: Option<&crate::registry::ProjectMapping>,
    origin_parent: Option<&ProviderPermissions>,
) -> Result<(), String> {
    let mut operation = PermissionReadOperation::default();
    let result = recheck_remote_child_permissions_in_operation(policy, definition, cwd, source, protected, mapping, origin_parent, &mut operation)?;
    operation.finish()?;
    Ok(result)
}

pub(crate) fn recheck_remote_child_permissions_in_operation(
    policy: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
    protected: &[PathBuf],
    mapping: Option<&crate::registry::ProjectMapping>,
    origin_parent: Option<&ProviderPermissions>,
    operation: &mut PermissionReadOperation,
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
        if context.get("hook_execution").is_some() && context.get("settings_overrides").is_some() {
            require_sdk_native_mount171(context, definition)?;
            let parent = origin_parent.ok_or("permission_mapping_unavailable")?;
            let sdk = AuthenticatedSdkInputs171::from_parent(parent)?.ok_or("permission_mapping_unavailable")?;
            if context.get("settings_overrides") != Some(sdk.settings)
                || policy["allowed_tools"] != parent.provider_policy["allowed_tools"] {
                return Err("permission_mapping_unavailable".into());
            }
        }
        if context.get("hook_execution").is_none() {
            check_native_target_directories(context, protected)?;
            bridget_transport::protocol::recheck_permission_context_sources(context)?;
            if *context != capture_native_target_context(definition, cwd, source)? {
                return Err(changed());
            }
        } else {
            // Compatibility for already admitted work using the older route.
            recheck_context_in_operation(context, definition, cwd, source, operation)?;
        }
        if context["hook_execution"] == "native_remote_guarded" {
            let witness = &context["source_equivalence"];
            let mapping = mapping.ok_or_else(|| equivalence_fail_at(ProjectionGuard::G004))?;
            let parent = origin_parent.ok_or_else(|| equivalence_fail_at(ProjectionGuard::G005))?;
            check_equivalence_mapping(&witness["binding"], mapping)?;
            if witness["binding"] != mapping_equivalence_binding(parent, policy, mapping)?
                || witness["profile"] != parent.provider_policy["launch_context"]["source_equivalence"]["profile"]
                || witness["origin_sources"] != parent.provider_policy["launch_context"]["permission_sources"] {
                return Err(equivalence_fail_at(ProjectionGuard::G006));
            }
            let verified = operation.profile(context)?;
            let profile = verified.profile(context)?;
            check_profile_additional_root171(profile, mapping)?;
            check_parent_additional_root171(parent, profile)?;
            if let Some(extra) = &mapping.additional_root {
                verify_permission_directory171(&extra.remote_root, extra.remote_uid, extra.remote_mode, protected)?;
            }
        } else if context["hook_execution"] == "native_remote_equivalent" {
            let witness = &context["source_equivalence"];
            let mapping = mapping.ok_or_else(|| equivalence_fail_at(ProjectionGuard::G007))?;
            let parent = origin_parent.ok_or_else(|| equivalence_fail_at(ProjectionGuard::G008))?;
            check_equivalence_mapping(&witness["binding"], mapping)?;
            if witness["binding"] != mapping_equivalence_binding(parent, policy, mapping)?
                || witness["origin"]
                    != *origin_equivalence(&parent.provider_policy["launch_context"])?
            {
                return Err(equivalence_fail_at(ProjectionGuard::G009));
            }
        } else if context.get("hook_execution").is_some() {
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
    bridget_transport::protocol::permission_source_revision_v2(path, 16 * 1024 * 1024)
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

/// An absolute configured native Claude is its own launch authority. The
/// registry chooses the command; PATH still resolves bare commands and hooks.
fn configured_absolute_claude171(command: &str, launcher: &Path) -> Result<bool, String> {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let logical = Path::new(command);
    if !logical.is_absolute() || logical.file_name().and_then(|s| s.to_str()) != Some("claude") {
        return Ok(false);
    }
    if logical.components().any(|p| matches!(p, std::path::Component::ParentDir)) {
        return Err(unavailable());
    }
    let mut file = std::fs::File::open(launcher).map_err(|_| unavailable())?;
    let metadata = file.metadata().map_err(|_| unavailable())?;
    let owner = unsafe { libc::geteuid() };
    if !metadata.is_file() || (metadata.uid() != owner && metadata.uid() != 0)
        || metadata.mode() & 0o022 != 0 || metadata.mode() & 0o111 == 0 {
        return Err(unavailable());
    }
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|_| unavailable())?;
    if !matches!(magic, [0x7f, b'E', b'L', b'F']
        | [0xcf, 0xfa, 0xed, 0xfe] | [0xce, 0xfa, 0xed, 0xfe]
        | [0xfe, 0xed, 0xfa, 0xcf] | [0xfe, 0xed, 0xfa, 0xce]
        | [0xca, 0xfe, 0xba, 0xbe] | [0xbe, 0xba, 0xfe, 0xca]
        | [0xca, 0xfe, 0xba, 0xbf] | [0xbf, 0xba, 0xfe, 0xca]) {
        return Err(unavailable());
    }
    if std::fs::canonicalize(logical).map_err(|_| unavailable())? != launcher {
        return Err(unavailable());
    }
    Ok(true)
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

fn project_root(cwd: &Path) -> PathBuf {
    cwd.ancestors()
        .find(|path| path.join(".git").exists())
        .unwrap_or(cwd)
        .to_owned()
}

fn hook_words(command: &str, project: &Path, plugin: &Path) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in command.chars() {
        if escaped {
            word.push(character);
            escaped = false;
        } else if character == '\\' && quote != Some('\'') {
            escaped = true;
        } else if quote == Some(character) {
            quote = None;
        } else if quote.is_none() && (character == '\'' || character == '"') {
            quote = Some(character);
        } else if quote.is_none() && character.is_whitespace() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else {
            word.push(character);
        }
    }
    if escaped || quote.is_some() {
        return Err(unavailable());
    }
    if !word.is_empty() {
        words.push(word);
    }
    let replace = |word: String| {
        word.replace("${CLAUDE_PROJECT_DIR}", &project.to_string_lossy())
            .replace("$CLAUDE_PROJECT_DIR", &project.to_string_lossy())
            .replace("${CLAUDE_PLUGIN_ROOT}", &plugin.to_string_lossy())
            .replace("$CLAUDE_PLUGIN_ROOT", &plugin.to_string_lossy())
    };
    Ok(words.into_iter().map(replace).collect())
}

fn hook_commands(value: &Value, commands: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(command) = object.get("command").and_then(Value::as_str) {
                commands.push(command.to_owned());
            }
            for child in object.values() {
                hook_commands(child, commands);
            }
        }
        Value::Array(values) => values
            .iter()
            .for_each(|child| hook_commands(child, commands)),
        _ => {}
    }
}

fn append_hook_sources(
    value: &Value,
    declaration: &Path,
    project: &Path,
    plugin: &Path,
    env: &SourceEnvironment,
    sources: &mut Vec<(String, PathBuf)>,
) -> Result<bool, String> {
    let Some(hooks) = value.get("hooks") else {
        return Ok(false);
    };
    // Selection is rechecked; the original native provider still applies it.
    // This is not a claim of hermetic shell dependencies or remote portability.
    sources.push((
        "hook".into(),
        std::fs::canonicalize(declaration).map_err(|_| unavailable())?,
    ));
    let mut commands = Vec::new();
    hook_commands(hooks, &mut commands);
    for command in commands {
        if command
            .chars()
            .any(|c| matches!(c, ';' | '&' | '|' | '<' | '>' | '\x60' | '\n' | '\r'))
            || command.contains("$(")
        {
            continue;
        }
        let Ok(words) = hook_words(&command, project, plugin) else {
            continue;
        };
        if words.iter().any(|word| word.contains('$')) {
            continue;
        }
        if let Some(command) = words.first() {
            let builtin = [
                ":", "cd", "printf", "read", "exit", "return", "set", "unset", "export", "local",
                "declare", "typeset",
            ];
            if command.contains('=') {
                continue;
            }
            if !command.contains('/') && !builtin.contains(&command.as_str()) {
                resolved(command, env)?;
            }
        }
        for (index, word) in words.iter().enumerate() {
            let direct = index == 0 && (word.contains('/') || word.starts_with('.'));
            let script = index > 0
                && ["sh", "bash", "zsh", "py", "js", "mjs", "cjs", "rb", "pl"]
                    .iter()
                    .any(|extension| {
                        Path::new(word).extension().and_then(|v| v.to_str()) == Some(*extension)
                    });
            if !direct && !script {
                continue;
            }
            let logical = if Path::new(word).is_absolute() {
                PathBuf::from(word)
            } else {
                project.join(word)
            };
            let real = std::fs::canonicalize(&logical).map_err(|_| unavailable())?;
            let metadata = std::fs::metadata(&real).map_err(|_| unavailable())?;
            if !metadata.is_file() {
                return Err(unavailable());
            }
            use std::os::unix::fs::PermissionsExt;
            if direct && metadata.permissions().mode() & 0o111 == 0 {
                return Err(unavailable());
            }
            let home = env.get("HOME").map(PathBuf::from).ok_or_else(unavailable)?;
            // `real` est résolu : comparer à des racines résolues aussi. Sinon
            // un HOME via lien (macOS : /tmp -> /private/tmp) refuse à tort.
            let under = |root: &Path| {
                real.starts_with(root)
                    || std::fs::canonicalize(root).is_ok_and(|root| real.starts_with(root))
            };
            let owned = under(&home) || under(project) || under(plugin);
            use std::os::unix::fs::MetadataExt;
            let standard = ["/bin", "/usr/bin", "/usr/local/bin"]
                .iter()
                .any(|base| real.starts_with(base))
                && metadata.uid() == 0;
            if !owned && !standard {
                return Err(unavailable());
            }
            if standard && !owned {
                continue;
            }
            sources.push(("hook".into(), logical));
        }
    }
    Ok(true)
}

fn security_source_is_observable(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }
    let value = read_json_source(path)?;
    if ["policyHelper", "policyHelpers", "processWrapper"]
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
    capture_context_with_env_impl(definition, cwd, env, false, false)
}

// Only the destination's own CLI grants are used; parent tool denials and
// explicit tool reductions remain constraints on the mission.
fn merge_native_target_arguments(policy: &mut Value, definition: &AgentDefinition,
    protected: &[PathBuf]) -> Result<(), String> {
    policy.as_object_mut().ok_or_else(unavailable)?.remove("allowed_tools");
    let mut index = 0;
    while index < definition.args.len() {
        let arg = &definition.args[index];
        let (flag, inline) = arg.split_once('=').map_or((arg.as_str(), None), |(a,b)| (a,Some(b)));
        if !matches!(flag, "--tools" | "--allowedTools" | "--allowed-tools" |
            "--disallowedTools" | "--disallowed-tools" | "--add-dir") { index += 1; continue; }
        let mut values = Vec::new();
        if let Some(value) = inline { values.push(value.to_owned()); }
        else {
            index += 1;
            while index < definition.args.len() && !definition.args[index].starts_with('-') {
                values.push(definition.args[index].clone()); index += 1;
            }
            if index > 0 { index -= 1; }
        }
        if values.is_empty() { return Err("permission_mapping_unavailable".into()); }
        if flag == "--add-dir" {
            for value in values {
                crate::registry::validate_unprotected_local_path(Path::new(&value), protected)?;
                let paths = policy.as_object_mut().ok_or_else(unavailable)?
                    .entry("additional_directories").or_insert_with(|| json!([]))
                    .as_array_mut().ok_or_else(unavailable)?;
                if !paths.contains(&json!(value)) { paths.push(json!(value)); }
            }
        } else {
            let rules: Vec<Value> = values.iter().flat_map(|v| v.split(','))
                .filter(|v| !v.is_empty()).map(|v| json!(v)).collect();
            match flag {
                "--tools" => {
                    if rules == vec![json!("default")] { /* Native preset; retain the parent's reduction. */ }
                    else if let Some(parent_tools) = policy["tools"].as_array() {
                        policy["tools"] = json!(rules.into_iter().filter(|r| parent_tools.contains(r)).collect::<Vec<_>>());
                    } else { policy["tools"] = json!(rules); }
                }
                "--allowedTools" | "--allowed-tools" => { policy["allowed_tools"] = json!(rules); }
                _ => {
                    let denies = policy.as_object_mut().ok_or_else(unavailable)?
                        .entry("disallowed_tools").or_insert_with(|| json!([]))
                        .as_array_mut().ok_or_else(unavailable)?;
                    for rule in rules { if !denies.contains(&rule) { denies.push(rule); } }
                }
            }
        }
        index += 1;
    }
    Ok(())
}

fn check_native_target_directories(context: &Value, protected: &[PathBuf]) -> Result<(), String> {
    for source in context["permission_sources"].as_array().ok_or_else(unavailable)? {
        let path = Path::new(source["path"].as_str().ok_or_else(unavailable)?);
        if source["revision"] == "absent" || !path.is_file() || path.extension().and_then(|e|e.to_str()) != Some("json") { continue; }
        let settings = read_json_source(path)?;
        if let Some(paths) = settings.pointer("/permissions/additionalDirectories") {
            for path in paths.as_array().ok_or_else(unavailable)? {
                crate::registry::validate_unprotected_local_path(Path::new(path.as_str().ok_or_else(unavailable)?), protected)?;
            }
        }
    }
    Ok(())
}

fn capture_native_target_context(
    definition: &AgentDefinition, cwd: &Path, source: &SourceEnvironment,
) -> Result<Value, String> {
    let env = build_environment(definition, source).map_err(|_| unavailable())?;
    capture_context_with_env_impl(definition, cwd, &env, false, true)
}

fn native_target_mode(definition: &AgentDefinition) -> Result<&str, String> {
    let mut mode = if definition.args.iter().any(|a| a == "--dangerously-skip-permissions") {
        "bypassPermissions"
    } else { "default" };
    let mut args = definition.args.iter();
    while let Some(arg) = args.next() {
        if arg == "--permission-mode" { mode = args.next().ok_or_else(unavailable)?; }
        else if let Some(value) = arg.strip_prefix("--permission-mode=") { mode = value; }
    }
    if !matches!(mode, "default" | "plan" | "acceptEdits" | "bypassPermissions" | "auto" | "dontAsk") {
        return Err("permission_mapping_unavailable".into());
    }
    Ok(mode)
}

fn capture_context_with_env_impl(
    definition: &AgentDefinition, cwd: &Path, env: &SourceEnvironment,
    guarded_selection: bool, native_target: bool,
) -> Result<Value, String> {
    if definition.protocol != "claude_stream_json" {
        return Err(unavailable());
    }
    // Sources du projet sous le dossier résolu, comme la racine du projet et
    // le dossier de configuration : un cwd via lien (~/10.Scripts) ne doit
    // pas produire des chemins hors de la racine résolue (garde G054).
    let resolved_cwd = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let cwd = resolved_cwd.as_path();
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
    let cli = if native_target {
        // The authenticated destination registry chooses its launcher. Its
        // name and installation path need not match any origin computer.
        use std::os::unix::fs::MetadataExt;
        let m = std::fs::metadata(&launcher).map_err(|_| unavailable())?;
        if !m.is_file() || (m.uid() != unsafe { libc::geteuid() } && m.uid() != 0)
            || m.mode() & 0o022 != 0 || m.mode() & 0o111 == 0 { return Err(unavailable()); }
        launcher.clone()
    } else if launcher_revision == TRANSPARENT_GCLAUDE {
        // The pinned transparent wrapper still needs its actual PATH target.
        resolved("claude", env)?
    } else if configured_absolute_claude171(&definition.command, &launcher)? {
        launcher.clone()
    } else {
        let direct_cli = resolved("claude", env)?;
        // Bare commands and unknown launchers retain their existing contract.
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
    if !native_target && n == 2 && head == *b"#!" {
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
    if guarded_selection {
        if selected != json!([]) { return Err(equivalence_fail_at(ProjectionGuard::G010)); }
        paths.retain(|(_, p)| p.starts_with(&managed));
    }
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
        if guarded_selection { break; }
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
    let project = project_root(cwd);
    let mut hook_sources = Vec::new();
    let settings = config.join("settings.json");
    let settings: Value = if !guarded_selection && settings.exists() {
        read_json_source(&settings)?
    } else {
        Value::Null
    };
    if native_target && settings.get("enabledPlugins").is_some() {
        // These metadata stay local. The provider decides which plugins load
        // and reports its own missing-plugin warnings; no copied origin cache.
        paths.push(("user", config.join("plugins/installed_plugins.json")));
        paths.push(("user", config.join("plugins/known_marketplaces.json")));
    }
    if let Some(enabled) = settings.get("enabledPlugins").and_then(Value::as_object).filter(|_| !native_target) {
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
            let manifest: Value = if source_revision(&manifest_path)? != "absent" {
                read_json_source(&manifest_path)?
            } else {
                // Native Claude allows a missing plugin.json: the selected
                // marketplace entry supplies the manifest. Pin both sources.
                let (plugin_name, marketplace_name) =
                    name.rsplit_once('@').ok_or_else(unavailable)?;
                let known_path = config.join("plugins/known_marketplaces.json");
                let known = read_json_source(&known_path)?;
                let location = PathBuf::from(
                    known
                        .get(marketplace_name)
                        .and_then(|entry| entry.get("installLocation"))
                        .and_then(Value::as_str)
                        .ok_or_else(unavailable)?,
                );
                if !location.is_absolute() {
                    return Err(unavailable());
                }
                let declaration = location.join(".claude-plugin/marketplace.json");
                let marketplace = read_json_source(&declaration)?;
                let plugins = marketplace
                    .get("plugins")
                    .and_then(Value::as_array)
                    .filter(|plugins| plugins.len() <= 1024)
                    .ok_or_else(unavailable)?;
                let mut selected = plugins.iter().filter(|entry| entry["name"] == plugin_name);
                let plugin = selected.next().ok_or_else(unavailable)?.clone();
                if selected.next().is_some() {
                    return Err(unavailable());
                }
                append_hook_sources(
                    &plugin,
                    &declaration,
                    &project,
                    &path,
                    env,
                    &mut hook_sources,
                )?;
                paths.push(("user", known_path));
                paths.push(("user", declaration));
                plugin
            };
            if ["policyHelper", "policyHelpers", "processWrapper"]
                .iter()
                .any(|key| manifest.get(*key).is_some())
            {
                return Err(unavailable());
            }
            paths.push(("user", manifest_path));
            for suffix in ["hooks/hooks.json", "settings.json", ".mcp.json"] {
                let source = path.join(suffix);
                paths.push(("user", source));
            }
        }
    }
    if paths.len() > 128 {
        return Err(unavailable());
    }
    let mut sources = Vec::new();
    for (kind, path) in paths {
        security_source_is_observable(&path)?;
        if !native_target && path != config.join("settings.json")
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
        if !native_target && path.is_dir() {
            for entry in std::fs::read_dir(&path).map_err(|_| unavailable())? {
                let entry = entry.map_err(|_| unavailable())?.path();
                security_source_is_observable(&entry)?;
                if entry.is_file()
                    && entry.extension().and_then(|extension| extension.to_str()) == Some("json")
                {
                    let value = read_json_source(&entry)?;
                    append_hook_sources(
                        &value,
                        &entry,
                        &project,
                        &project,
                        env,
                        &mut hook_sources,
                    )?;
                }
            }
        }
        let revision = if native_target && kind == "user"
            && path == config.join("plugins/known_marketplaces.json") {
            bridget_transport::protocol::permission_marketplace_revision171(&path, 16 * 1024 * 1024)?
        } else { source_revision(&path)? };
        sources.push(json!({"kind":kind,"path":path,"revision":revision}));
        if !native_target && path.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("json")
        {
            let value = read_json_source(&path)?;
            let plugin = path
                .ancestors()
                .find(|ancestor| ancestor.join(".claude-plugin/plugin.json").is_file())
                .unwrap_or(&project);
            append_hook_sources(&value, &path, &project, plugin, env, &mut hook_sources)?;
        }
    }
    for (kind, path) in hook_sources {
        let revision =
            bridget_transport::protocol::permission_hook_source_revision(&path, 16 * 1024 * 1024)?;
        if revision == "absent" {
            return Err(unavailable());
        }
        sources.push(json!({
            "kind": kind,
            "path": path,
            "revision": revision,
        }));
    }
    sources.sort_by(|left, right| {
        left["kind"]
            .as_str()
            .cmp(&right["kind"].as_str())
            .then_with(|| left["path"].as_str().cmp(&right["path"].as_str()))
    });
    sources.dedup_by(|left, right| left["kind"] == right["kind"] && left["path"] == right["path"]);
    if sources.len() > 128 {
        return Err(unavailable());
    }
    let has_hooks = sources.iter().any(|source| source["kind"] == "hook");
    // Une seule capture peut désigner deux fois le même fichier. Aucune
    // révision n'est conservée pour la capture ou la revalidation suivante.
    let cli_revision = if cli == launcher {
        launcher_revision.clone()
    } else {
        executable_revision(&cli)?
    };
    let mut context = json!({"attestation_version":2,"cli_path":launcher,"cli_revision":launcher_revision,"resolved_cli_path":cli,"resolved_cli_revision":cli_revision,"config_dir":config,"settings_sources":selected,"permission_sources":sources});
    if has_hooks {
        context["hook_execution"] = json!("native_same_host");
        if let Ok(manifest) = source_equivalence_manifest(&context, &project) {
            let mut with_witness = context.clone();
            with_witness["source_equivalence"] =
                json!({"version":1,"scope":"origin","manifest":manifest});
            if serde_json::to_vec(&with_witness)
                .map_err(|_| unavailable())?
                .len()
                <= 48 * 1024
            {
                context = with_witness;
            }
        }
    }
    if let Some(overrides) = overrides {
        context["settings_overrides"] = overrides;
    }
    if serde_json::to_vec(&context)
        .map_err(|_| unavailable())?
        .len()
        > 48 * 1024
    {
        return Err(unavailable());
    }
    Ok(context)
}

pub(crate) fn recheck_context(
    context: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
) -> Result<(), String> {
    let mut operation = PermissionReadOperation::default();
    let result = recheck_context_in_operation(context, definition, cwd, source, &mut operation)?;
    operation.finish()?;
    Ok(result)
}

pub(crate) fn recheck_context_in_operation(
    context: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
    operation: &mut PermissionReadOperation,
) -> Result<(), String> {
    let environment = build_environment(definition, source).map_err(|_| changed())?;
    recheck_context_with_env_in_operation(context, definition, cwd, &environment, operation)
}

pub(crate) fn recheck_context_with_env(
    context: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    environment: &SourceEnvironment,
) -> Result<(), String> {
    let mut operation = PermissionReadOperation::default();
    let result = recheck_context_with_env_in_operation(context, definition, cwd, environment, &mut operation)?;
    operation.finish()?;
    Ok(result)
}

pub(crate) fn recheck_context_with_env_in_operation(
    context: &Value,
    definition: &AgentDefinition,
    cwd: &Path,
    environment: &SourceEnvironment,
    operation: &mut PermissionReadOperation,
) -> Result<(), String> {
    recheck_context_with_sdk171(context, definition, cwd, environment, operation, None)
}

fn recheck_context_with_sdk171(
    context: &Value, definition: &AgentDefinition, cwd: &Path,
    environment: &SourceEnvironment, operation: &mut PermissionReadOperation,
    sdk: Option<&AuthenticatedSdkInputs171<'_>>,
) -> Result<(), String> {
    let verified_profile = if context.pointer("/source_equivalence/version") == Some(&json!(2)) {
        let verified = operation.profile(context)?;
        bridget_transport::protocol::recheck_permission_context_sources_with_profile(context, &verified)?;
        Some(verified)
    } else {
        bridget_transport::protocol::recheck_permission_context_sources(context)?;
        None
    };
    let scoped;
    let definition = if context["hook_execution"] == "native_remote_guarded" {
        let verified = operation.profile(context)?;
        scoped = guarded_profile_definition(definition, verified.profile(context)?, context)?;
        &scoped
    } else { definition };
    let mut captured = if context["hook_execution"] == "native_remote_guarded" {
        capture_guarded_context(definition, context, cwd, environment, operation).map_err(|_| changed())?
    } else if context.get("hook_execution").is_none() && context.get("source_equivalence").is_none() {
        // The wrapper must recheck the same native destination capture used at
        // admission, rather than reconstructing an origin-equivalent profile.
        capture_context_with_env_impl(definition, cwd, environment, false, true).map_err(|_| changed())?
    } else { capture_context_with_env(definition, cwd, environment).map_err(|_| changed())? };
    if let Some(sdk) = sdk {
        // SDK query settings are a provider-turn fact, not registry CLI args.
        // Never replace a native CLI selector or an independently captured override.
        if captured.get("settings_overrides").is_some()
            || definition.args.iter().any(|a| a == "--settings" || a.starts_with("--settings="))
            || context.get("settings_overrides") != Some(sdk.settings)
        { return Err(changed()); }
        captured["settings_overrides"] = sdk.settings.clone();
    }
    let legacy = context.get("attestation_version").is_none();
    if legacy && captured.get("hook_execution").is_some() {
        return Err(changed());
    }
    for key in [
        "cli_path",
        "cli_revision",
        "resolved_cli_path",
        "resolved_cli_revision",
        "config_dir",
        "settings_sources",
    ] {
        if context.get(key) != captured.get(key) {
            return Err(changed());
        }
    }
    if context["hook_execution"] == "native_same_host"
        && context.pointer("/source_equivalence/version") == Some(&json!(2))
        && context.pointer("/source_equivalence/scope") == Some(&json!("origin"))
    {
        if captured.get("settings_overrides") != context.get("settings_overrides") {
            return Err(changed());
        }
        if environment.keys().any(|key| ["SECURITY_", "SECURITY_GUIDANCE_", "SECURITY_WARNINGS_", "SG_",
            "ENABLE_PATTERN_RULES", "ENABLE_CODE_SECURITY_REVIEW", "ENABLE_STOP_REVIEW", "ENABLE_COMMIT_REVIEW"]
            .iter().any(|prefix| key.starts_with(prefix))) {
            return Err("permission_hook_environment_unavailable".into());
        }
        let env_value = |key: &str| -> Result<Value, String> {
            match environment.get(key) {
                None => Ok(Value::Null),
                Some(value) => value.to_str().map(|s| json!(s))
                    .ok_or_else(|| "permission_hook_environment_unavailable".to_owned()),
            }
        };
        let environment = json!({
            "TMUX":env_value("TMUX")?,
            "TMUX_PANE":env_value("TMUX_PANE")?,
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC":env_value("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC")?,
        });
        captured["source_equivalence"] = verified_profile.as_ref().ok_or_else(|| "permission_hook_equivalence_unavailable".to_owned())?
            .origin_witness_for(&captured, &project_root(cwd), &environment)?;
    }
    if context["hook_execution"] == "native_remote_guarded" {
        operation.profile(context)?;
    } else if context["hook_execution"] == "native_remote_equivalent" {
        let actual = origin_equivalence(&captured)?;
        if context["source_equivalence"]["target"] != *actual {
            return Err(changed());
        }
    } else if context.get("hook_execution") != captured.get("hook_execution")
        || context.get("source_equivalence") != captured.get("source_equivalence")
    {
        return Err(changed());
    }
    if !legacy && context.get("attestation_version") != captured.get("attestation_version") {
        return Err(changed());
    }
    let normalize = |value: &Value| -> Result<Vec<Value>, String> {
        let mut sources = value.as_array().ok_or_else(changed)?.clone();
        sources.sort_by(|a, b| {
            a["kind"]
                .as_str()
                .cmp(&b["kind"].as_str())
                .then_with(|| a["path"].as_str().cmp(&b["path"].as_str()))
        });
        Ok(sources)
    };
    if normalize(&context["permission_sources"])? != normalize(&captured["permission_sources"])? {
        return Err(changed());
    }
    Ok(())
}

fn claude_inputs_without_rules(context: &Value) -> Result<bool, String> {
    // The common recheck selects the native hook digest (content + ownership +
    // mode). A verified hook is still a rule, not an empty cross-provider policy.
    bridget_transport::protocol::recheck_permission_context_sources(context)?;
    let sources = context["permission_sources"]
        .as_array()
        .ok_or_else(unavailable)?;
    if sources.iter().any(|source| source["kind"] == "hook") {
        return Ok(false);
    }
    for source in sources {
        let path = Path::new(source["path"].as_str().ok_or_else(unavailable)?);
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
    child_policy_reader_with_capture(parent, definition, cwd, source, origin_attested, None)
}

fn child_policy_reader_with_capture<'a>(
    parent: &'a ProviderPermissions, definition: &'a AgentDefinition,
    cwd: &'a Path, source: &'a SourceEnvironment, origin_attested: bool,
    prepared: Option<Value>,
) -> impl FnMut(Option<SpawnPosture>) -> Result<(Value, SpawnPosture), String> + 'a {
    child_policy_reader_catalogue_scope171(parent, definition, cwd, source, origin_attested, prepared, None)
}

pub(crate) fn child_policy_reader_for_catalogue171<'a>(
    parent: &'a ProviderPermissions, definition: &'a AgentDefinition,
    cwd: &'a Path, source: &'a SourceEnvironment,
    catalogue: Option<&'a std::cell::RefCell<CataloguePermissionRead171>>,
) -> impl FnMut(Option<SpawnPosture>) -> Result<(Value, SpawnPosture), String> + 'a {
    child_policy_reader_catalogue_scope171(parent, definition, cwd, source, false, None, catalogue)
}

fn child_policy_reader_catalogue_scope171<'a>(
    parent: &'a ProviderPermissions, definition: &'a AgentDefinition,
    cwd: &'a Path, source: &'a SourceEnvironment, origin_attested: bool,
    prepared: Option<Value>, catalogue: Option<&'a std::cell::RefCell<CataloguePermissionRead171>>,
) -> impl FnMut(Option<SpawnPosture>) -> Result<(Value, SpawnPosture), String> + 'a {
    let mut captured: Option<Result<Value, String>> = prepared.map(Ok);
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
                    if AuthenticatedSdkInputs171::from_parent(parent)?.is_some() {
                        require_sdk_native_mount171(&policy["launch_context"], definition)?;
                    }
                    rechecked
                        .get_or_insert_with(|| match catalogue { Some(scope) => scope.borrow_mut().recheck(parent, definition, cwd, source), None => recheck_parent_context171(parent, definition, cwd, source) })
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
                // Seul le fait natif authentifié sans profil local peut omettre
                // launch_context. Ne pas déduire cette exception d’un nom d’outil seul.
                let native_scope = origin_attested && native_t3_codex_scope214(parent, definition);
                if native_scope {
                    policy.as_object_mut().unwrap().remove("allowed_tools");
                } else {
                    let context = policy.get("launch_context").ok_or("permission_source_unavailable")?;
                    if !origin_attested && !without_rules
                        .get_or_insert_with(|| claude_inputs_without_rules(context)).clone()? {
                        return Err("permission_mapping_unavailable".into());
                    }
                }
                if policy["disallowed_tools"]
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
    let mut operation = PermissionReadOperation::default();
    let result = apply_child_arguments_in_operation(protocol, policy, args, &mut operation)?;
    operation.finish()?;
    Ok(result)
}

pub(crate) fn apply_child_arguments_in_operation(
    protocol: &str,
    policy: &Value,
    args: &mut Vec<String>,
    operation: &mut PermissionReadOperation,
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
        if protocol == "claude_stream_json"
            && policy.pointer("/launch_context/hook_execution") == Some(&json!("native_remote_guarded"))
            && matches!(flag, "--plugin-dir" | "--settings") {
            index += if arg.contains('=') { 1 } else { 2 };
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
            kept.extend(["--permission-mode".into(), mode.into()]);
            // Native Claude print mode uses its configured permission mode.
            // --permission-prompts is not supported by Claude 2.1.251;
            // retain it only for the legacy attested launch contract.
            if policy.pointer("/launch_context/hook_execution").is_some() {
                kept.extend(["--permission-prompts".into(), "none".into()]);
            }
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
            if policy.pointer("/launch_context/hook_execution") == Some(&json!("native_remote_guarded")) {
                let verified = operation.profile(&policy["launch_context"])?;
                let profile = verified.profile(&policy["launch_context"])?;
                let settings = match guarded_sdk_settings171(profile, &policy["launch_context"])? {
                    Some(value) => serde_json::to_string(&value).map_err(|_| unavailable())?,
                    None => profile["settings"]["target"]["path"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G011))?.to_owned(),
                };
                kept.extend(["--settings".into(), settings]);
                for plugin in profile["target_controls"]["plugins"].as_array().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G012))? {
                    kept.extend(["--plugin-dir".into(), plugin["private_root"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G013))?.into()]);
                }
            } else if let Some(value) = policy.pointer("/launch_context/settings_overrides") {
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

// 171 private witness: unknown execution dependencies remain closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectionGuard {
    G001,
    G002,
    G003,
    G004,
    G005,
    G006,
    G007,
    G008,
    G009,
    G010,
    G011,
    G012,
    G013,
    G014,
    G015,
    G016,
    G017,
    G018,
    G019,
    G020,
    G021,
    G022,
    G023,
    G024,
    G025,
    G026,
    G027,
    G028,
    G029,
    G030,
    G031,
    G032,
    G033,
    G034,
    G035,
    G036,
    G037,
    G038,
    G039,
    G040,
    G041,
    G042,
    G043,
    G044,
    G045,
    G046,
    G047,
    G048,
    G049,
    G050,
    G051,
    G052,
    G053,
    G054,
    G055,
    G056,
    G057,
    G058,
    G059,
    G060,
    G061,
    G062,
    G063,
    G064,
    G065,
    G066,
    G067,
    G068,
    G069,
    G070,
    G071,
    G072,
    G073,
    G074,
    G075,
    G076,
    G077,
    G078,
    G079,
    G080,
    G081,
    G082,
    G083,
}
impl ProjectionGuard {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 83] = [Self::G001,Self::G002,Self::G003,Self::G004,Self::G005,Self::G006,Self::G007,Self::G008,Self::G009,Self::G010,Self::G011,Self::G012,Self::G013,Self::G014,Self::G015,Self::G016,Self::G017,Self::G018,Self::G019,Self::G020,Self::G021,Self::G022,Self::G023,Self::G024,Self::G025,Self::G026,Self::G027,Self::G028,Self::G029,Self::G030,Self::G031,Self::G032,Self::G033,Self::G034,Self::G035,Self::G036,Self::G037,Self::G038,Self::G039,Self::G040,Self::G041,Self::G042,Self::G043,Self::G044,Self::G045,Self::G046,Self::G047,Self::G048,Self::G049,Self::G050,Self::G051,Self::G052,Self::G053,Self::G054,Self::G055,Self::G056,Self::G057,Self::G058,Self::G059,Self::G060,Self::G061,Self::G062,Self::G063,Self::G064,Self::G065,Self::G066,Self::G067,Self::G068,Self::G069,Self::G070,Self::G071,Self::G072,Self::G073,Self::G074,Self::G075,Self::G076,Self::G077,Self::G078,Self::G079,Self::G080,Self::G081,Self::G082,Self::G083];
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::G001 => "g001",
            Self::G002 => "g002",
            Self::G003 => "g003",
            Self::G004 => "g004",
            Self::G005 => "g005",
            Self::G006 => "g006",
            Self::G007 => "g007",
            Self::G008 => "g008",
            Self::G009 => "g009",
            Self::G010 => "g010",
            Self::G011 => "g011",
            Self::G012 => "g012",
            Self::G013 => "g013",
            Self::G014 => "g014",
            Self::G015 => "g015",
            Self::G016 => "g016",
            Self::G017 => "g017",
            Self::G018 => "g018",
            Self::G019 => "g019",
            Self::G020 => "g020",
            Self::G021 => "g021",
            Self::G022 => "g022",
            Self::G023 => "g023",
            Self::G024 => "g024",
            Self::G025 => "g025",
            Self::G026 => "g026",
            Self::G027 => "g027",
            Self::G028 => "g028",
            Self::G029 => "g029",
            Self::G030 => "g030",
            Self::G031 => "g031",
            Self::G032 => "g032",
            Self::G033 => "g033",
            Self::G034 => "g034",
            Self::G035 => "g035",
            Self::G036 => "g036",
            Self::G037 => "g037",
            Self::G038 => "g038",
            Self::G039 => "g039",
            Self::G040 => "g040",
            Self::G041 => "g041",
            Self::G042 => "g042",
            Self::G043 => "g043",
            Self::G044 => "g044",
            Self::G045 => "g045",
            Self::G046 => "g046",
            Self::G047 => "g047",
            Self::G048 => "g048",
            Self::G049 => "g049",
            Self::G050 => "g050",
            Self::G051 => "g051",
            Self::G052 => "g052",
            Self::G053 => "g053",
            Self::G054 => "g054",
            Self::G055 => "g055",
            Self::G056 => "g056",
            Self::G057 => "g057",
            Self::G058 => "g058",
            Self::G059 => "g059",
            Self::G060 => "g060",
            Self::G061 => "g061",
            Self::G062 => "g062",
            Self::G063 => "g063",
            Self::G064 => "g064",
            Self::G065 => "g065",
            Self::G066 => "g066",
            Self::G067 => "g067",
            Self::G068 => "g068",
            Self::G069 => "g069",
            Self::G070 => "g070",
            Self::G071 => "g071",
            Self::G072 => "g072",
            Self::G073 => "g073",
            Self::G074 => "g074",
            Self::G075 => "g075",
            Self::G076 => "g076",
            Self::G077 => "g077",
            Self::G078 => "g078",
            Self::G079 => "g079",
            Self::G080 => "g080",
            Self::G081 => "g081",
            Self::G082 => "g082",
            Self::G083 => "g083",
        }
    }
}

// Closed observation only. The slot is active on this synchronous call stack;
// it contains one enum, never a policy, and Drop restores it on every exit.
thread_local! {
    static PROJECTION_GUARD171: std::cell::RefCell<Option<Option<ProjectionGuard>>> =
        const { std::cell::RefCell::new(None) };
}
struct ProjectionGuardScope171(Option<Option<ProjectionGuard>>);
impl Drop for ProjectionGuardScope171 {
    fn drop(&mut self) {
        PROJECTION_GUARD171.with(|slot| { slot.replace(self.0); });
    }
}
pub(crate) fn observe_projection171<T>(
    read: impl FnOnce() -> Result<T, String>,
) -> (Result<T, String>, Option<ProjectionGuard>) {
    let _scope = ProjectionGuardScope171(PROJECTION_GUARD171.with(|slot| slot.replace(Some(None))));
    let result = read();
    let guard = if result.as_ref().err().is_some_and(|e| e == "permission_hook_equivalence_unavailable") {
        PROJECTION_GUARD171.with(|slot| (*slot.borrow()).flatten())
    } else { None };
    (result, guard)
}
pub(crate) fn equivalence_fail_at(guard: ProjectionGuard) -> String {
    PROJECTION_GUARD171.with(|slot| {
        if let Some(current) = slot.borrow_mut().as_mut() { *current = Some(guard); }
    });
    "permission_hook_equivalence_unavailable".into()
}

// Only the witness shape and an already valid digest can leave this reader.
// No arbitrary string, serialized body, path or environment value is retained.
pub(crate) struct ProjectionWitnessSummary171 {
    shape: &'static str,
    version: Value,
    scope: Value,
    profile: &'static str,
    revision: Value,
}
fn projection_value_kind171(value: Option<&Value>) -> &'static str {
    match value {
        None => "absent", Some(Value::Null) => "null", Some(Value::Bool(_)) => "boolean",
        Some(Value::Number(_)) => "number", Some(Value::String(_)) => "string",
        Some(Value::Array(_)) => "array", Some(Value::Object(_)) => "object",
    }
}
pub(crate) fn projection_witness_summary171(policy: &Value) -> ProjectionWitnessSummary171 {
    let witness = policy.get("launch_context").and_then(|c| c.get("source_equivalence"));
    let version = witness.and_then(|w| w.get("version"));
    let scope = witness.and_then(|w| w.get("scope"));
    let profile = witness.and_then(|w| w.get("profile"));
    let revision = profile.and_then(|p| p.get("revision"));
    ProjectionWitnessSummary171 {
        shape: projection_value_kind171(witness),
        version: match version.and_then(Value::as_u64) {
            Some(v @ (1 | 2)) => json!(v), _ => json!(projection_value_kind171(version)),
        },
        scope: match scope.and_then(Value::as_str) {
            Some("origin") => json!("origin"), Some("remote") => json!("remote"),
            _ => json!(projection_value_kind171(scope)),
        },
        profile: projection_value_kind171(profile),
        revision: match revision.and_then(Value::as_str) {
            Some(s) if s.len() == 71 && s.starts_with("sha256:")
                && s[7..].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) => json!(s),
            _ => json!(projection_value_kind171(revision)),
        },
    }
}
impl ProjectionWitnessSummary171 {
    pub(crate) fn json(&self) -> Value {
        json!({"shape":self.shape,"version":self.version,"scope":self.scope,
               "profile":self.profile,"revision":self.revision})
    }
}

fn equivalence_sha(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn equivalence_policy_digest(policy: &Value) -> Result<String, String> {
    let mut fields = policy.clone();
    fields
        .as_object_mut()
        .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G014))?
        .remove("launch_context");
    Ok(equivalence_sha(
        &serde_json::to_vec(&fields).map_err(|_| equivalence_fail_at(ProjectionGuard::G015))?,
    ))
}
fn equivalence_read(path: &Path) -> Result<(Vec<u8>, std::fs::Metadata), String> {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let file = std::fs::File::open(path).map_err(|_| equivalence_fail_at(ProjectionGuard::G016))?;
    let a = file.metadata().map_err(|_| equivalence_fail_at(ProjectionGuard::G017))?;
    if !a.is_file() || a.len() > 4 * 1024 * 1024 {
        return Err(equivalence_fail_at(ProjectionGuard::G018));
    }
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| equivalence_fail_at(ProjectionGuard::G019))?;
    let b = std::fs::metadata(path).map_err(|_| equivalence_fail_at(ProjectionGuard::G020))?;
    if bytes.len() > 4 * 1024 * 1024
        || a.dev() != b.dev()
        || a.ino() != b.ino()
        || a.len() != b.len()
        || a.mtime() != b.mtime()
        || a.mtime_nsec() != b.mtime_nsec()
        || a.ctime() != b.ctime()
        || a.ctime_nsec() != b.ctime_nsec()
        || a.mode() != b.mode()
        || a.uid() != b.uid()
        || a.gid() != b.gid()
    {
        return Err(equivalence_fail_at(ProjectionGuard::G021));
    }
    Ok((bytes, b))
}
fn equivalence_script_dependencies(path: &Path) -> Result<Vec<Value>, String> {
    use std::os::unix::fs::MetadataExt;
    let (bytes, stat) = equivalence_read(path)?;
    if stat.mode() & 0o111 == 0 {
        return Err(equivalence_fail_at(ProjectionGuard::G022));
    }
    // This deliberately small complete grammar has no hidden program calls.
    // General Bash programs, including guard148, need another qualified
    // dependency declaration; their hashes alone do not qualify them.
    if ![
        b"#!/bin/sh\nexit 0\n".as_slice(),
        b"#!/bin/sh\nexit 2\n".as_slice(),
    ]
    .contains(&bytes.as_slice())
    {
        return Err("permission_hook_dependencies_unavailable".into());
    }
    let interpreter = Path::new("/bin/sh");
    let m = std::fs::metadata(interpreter).map_err(|_| equivalence_fail_at(ProjectionGuard::G023))?;
    if !m.is_file()
        || m.uid() != 0
        || m.gid() != 0
        || m.mode() & 0o111 == 0
        || m.mode() & 0o022 != 0
    {
        return Err(equivalence_fail_at(ProjectionGuard::G024));
    }
    let revision =
        bridget_transport::protocol::permission_source_revision_v2(interpreter, 512 * 1024 * 1024)?;
    Ok(vec![
        json!({"path":interpreter,"revision":revision,"mode":m.mode() & 0o7777,"owner":"system"}),
    ])
}
fn equivalence_command(command: &str, project: &Path, context: &Value) -> Result<(), String> {
    if command
        .chars()
        .any(|c| matches!(c, ';' | '&' | '|' | '<' | '>' | '\x60' | '\n' | '\r'))
        || command.contains("$(")
        || command.contains("CLAUDE_PLUGIN_ROOT")
    {
        return Err(equivalence_fail_at(ProjectionGuard::G025));
    }
    let words = hook_words(command, project, project)?;
    if words.is_empty() || words.iter().any(|w| w.contains('$')) {
        return Err(equivalence_fail_at(ProjectionGuard::G026));
    }
    let path = Path::new(&words[0]);
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        || !path.starts_with(project)
    {
        return Err(equivalence_fail_at(ProjectionGuard::G027));
    }
    let real = std::fs::canonicalize(path).map_err(|_| equivalence_fail_at(ProjectionGuard::G028))?;
    if !real.starts_with(project)
        || !context["permission_sources"]
            .as_array()
            .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G029))?
            .iter()
            .any(|s| {
                s["kind"] == "hook"
                    && s["path"]
                        .as_str()
                        .is_some_and(|p| Path::new(p) == path || Path::new(p) == real)
            })
    {
        return Err(equivalence_fail_at(ProjectionGuard::G030));
    }
    equivalence_script_dependencies(path)?;
    Ok(())
}
fn equivalence_settings(value: &Value, project: &Path, context: &Value) -> Result<(), String> {
    let object = value.as_object().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G031))?;
    if object.keys().any(|k| {
        !matches!(
            k.as_str(),
            "permissions" | "hooks" | "model" | "modelPricing" | "effortLevel"
        )
    }) {
        return Err(equivalence_fail_at(ProjectionGuard::G032));
    }
    if let Some(p) = object.get("permissions") {
        let p = p.as_object().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G033))?;
        if p.keys().any(|k| {
            !matches!(
                k.as_str(),
                "allow"
                    | "ask"
                    | "deny"
                    | "defaultMode"
                    | "additionalDirectories"
                    | "disableBypassPermissionsMode"
            )
        }) {
            return Err(equivalence_fail_at(ProjectionGuard::G034));
        }
        for key in ["allow", "ask", "deny"] {
            if p.get(key).is_some_and(|v| {
                !v.as_array().is_some_and(|a| {
                    a.len() <= 128
                        && a.iter().all(|v| {
                            v.as_str().is_some_and(|s| {
                                !s.is_empty()
                                    && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                            })
                        })
                })
            }) {
                return Err(equivalence_fail_at(ProjectionGuard::G035));
            }
        }
        if p.get("additionalDirectories")
            .is_some_and(|v| !v.as_array().is_some_and(Vec::is_empty))
            || p.get("defaultMode").is_some_and(|v| {
                !matches!(
                    v.as_str(),
                    Some(
                        "default"
                            | "acceptEdits"
                            | "bypassPermissions"
                            | "plan"
                            | "dontAsk"
                            | "auto"
                    )
                )
            })
            || p.get("disableBypassPermissionsMode")
                .is_some_and(|v| v != "disable")
        {
            return Err(equivalence_fail_at(ProjectionGuard::G036));
        }
    }
    if let Some(hooks) = object.get("hooks") {
        for (event, groups) in hooks.as_object().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G037))? {
            if !matches!(
                event.as_str(),
                "PreToolUse"
                    | "PostToolUse"
                    | "SessionStart"
                    | "SessionEnd"
                    | "Stop"
                    | "UserPromptSubmit"
            ) {
                return Err(equivalence_fail_at(ProjectionGuard::G038));
            }
            for group in groups
                .as_array()
                .filter(|a| a.len() <= 128)
                .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G039))?
            {
                let group = group.as_object().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G040))?;
                if group
                    .keys()
                    .any(|k| !matches!(k.as_str(), "matcher" | "hooks"))
                    || group.get("matcher").is_some_and(|v| !v.is_string())
                {
                    return Err(equivalence_fail_at(ProjectionGuard::G041));
                }
                for hook in group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .filter(|a| a.len() <= 128)
                    .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G042))?
                {
                    let hook = hook.as_object().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G043))?;
                    if hook
                        .keys()
                        .any(|k| !matches!(k.as_str(), "type" | "command" | "timeout"))
                        || hook.get("type") != Some(&json!("command"))
                        || hook
                            .get("timeout")
                            .is_some_and(|v| !v.as_u64().is_some_and(|n| n <= 600))
                    {
                        return Err(equivalence_fail_at(ProjectionGuard::G044));
                    }
                    equivalence_command(
                        hook.get("command")
                            .and_then(Value::as_str)
                            .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G045))?,
                        project,
                        context,
                    )?;
                }
            }
        }
    }
    Ok(())
}
fn equivalence_selector(
    kind: &str,
    path: &Path,
    context: &Value,
    project: &Path,
) -> Result<String, String> {
    if kind == "hook" && path.extension().and_then(|s| s.to_str()) == Some("json") {
        let canonical = std::fs::canonicalize(path).map_err(|_| equivalence_fail_at(ProjectionGuard::G046))?;
        let mut selected = Vec::new();
        for row in context["permission_sources"]
            .as_array()
            .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G047))?
        {
            if row["kind"] == "hook" || row["revision"] == "absent" {
                continue;
            }
            let p = Path::new(row["path"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G048))?);
            if std::fs::canonicalize(p).is_ok_and(|p| p == canonical) {
                selected.push(equivalence_selector(
                    row["kind"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G049))?,
                    p,
                    context,
                    project,
                )?);
            }
        }
        if selected.len() != 1 {
            return Err(equivalence_fail_at(ProjectionGuard::G050));
        }
        return Ok(format!("hook-declaration:{}", selected[0]));
    }
    if let Ok(relative) = path.strip_prefix(project) {
        return Ok(format!(
            "project:{}",
            relative.to_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G051))?
        ));
    }
    let config = Path::new(
        context["config_dir"]
            .as_str()
            .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G052))?,
    );
    if let Ok(relative) = path.strip_prefix(config) {
        if matches!(kind, "user" | "managed") {
            return Ok(format!(
                "native-profile:{}",
                relative.to_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G053))?
            ));
        }
    }
    if kind == "managed"
        && path.file_name().and_then(|p| p.to_str()) == Some("managed-settings.json")
    {
        return Ok("native-managed:managed-settings.json".into());
    }
    Err(equivalence_fail_at(ProjectionGuard::G054))
}
fn source_equivalence_manifest(context: &Value, project: &Path) -> Result<Value, String> {
    use std::os::unix::fs::MetadataExt;
    bridget_transport::protocol::recheck_permission_context_sources(context)?;
    if context["attestation_version"] != 2 || context.get("settings_overrides").is_some() {
        return Err(equivalence_fail_at(ProjectionGuard::G055));
    }
    let project = std::fs::canonicalize(project).map_err(|_| equivalence_fail_at(ProjectionGuard::G056))?;
    let profile = std::fs::metadata(
        context["config_dir"]
            .as_str()
            .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G057))?,
    )
    .map_err(|_| equivalence_fail_at(ProjectionGuard::G058))?;
    if profile.uid() != unsafe { libc::geteuid() } {
        return Err(equivalence_fail_at(ProjectionGuard::G059));
    }
    let mut rows = Vec::new();
    for source in context["permission_sources"]
        .as_array()
        .filter(|a| a.len() <= 128)
        .ok_or_else(|| equivalence_fail_at(ProjectionGuard::G060))?
    {
        let kind = source["kind"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G061))?;
        let path = Path::new(source["path"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G062))?);
        if source["revision"] == "absent" {
            rows.push(json!({"kind":kind,"path":path,"revision":"absent","selector":null,"content_revision":"absent","mode":0,"owner":"absent","dependencies":[]}));
            continue;
        }
        let selector = equivalence_selector(kind, path, context, &project)?;
        let (bytes, m) = equivalence_read(path)?;
        let owner = if m.uid() == profile.uid() && m.gid() == profile.gid() {
            "profile"
        } else if kind == "managed" && m.uid() == 0 && (m.gid() == 0 || m.mode() & 0o022 == 0) {
            // macOS installe ses réglages administrés en root:admin 0644 :
            // seul root peut les modifier, comme root:root sous Linux.
            "system"
        } else {
            return Err(equivalence_fail_at(ProjectionGuard::G063));
        };
        let dependencies =
            if kind == "hook" && path.extension().and_then(|s| s.to_str()) != Some("json") {
                equivalence_script_dependencies(path)?
            } else {
                equivalence_settings(
                    &serde_json::from_slice(&bytes).map_err(|_| equivalence_fail_at(ProjectionGuard::G064))?,
                    &project,
                    context,
                )?;
                Vec::new()
            };
        let revision = if kind == "hook" {
            bridget_transport::protocol::permission_hook_source_revision(path, 16 * 1024 * 1024)?
        } else {
            source_revision(path)?
        };
        if revision != source["revision"] {
            return Err(changed());
        }
        rows.push(json!({"kind":kind,"path":path,"revision":revision,"selector":selector,"content_revision":equivalence_sha(&bytes),
            "mode":m.mode() & 0o7777,"owner":owner,"dependencies":dependencies}));
    }
    Ok(
        json!({"project_root":project,"settings_sources":context["settings_sources"],"settings_overrides":null,"rows":rows}),
    )
}
fn session_start_sdk_bootstrap_declared(context: &Value) -> bool {
    context["permission_sources"].as_array().is_some_and(|sources| {
        sources.iter().any(|source| {
            source["kind"] == "hook"
                && source["revision"].as_str().is_some_and(|r| r != "absent")
                && source["path"].as_str().is_some_and(|name| {
                    Path::new(name).file_name().and_then(|s| s.to_str())
                        == Some("ensure_agent_sdk.py")
                        && equivalence_read(Path::new(name)).is_ok_and(|(bytes, _)| {
                            equivalence_sha(&bytes)
                                == "sha256:e682d82026521106ec00267078fb4e2190b3a0a501362fc9f4a0753f2d7a1b56"
                        })
                })
        })
    })
}
fn origin_equivalence(context: &Value) -> Result<&Value, String> {
    bridget_transport::protocol::validate_permission_source_equivalence(context)?;
    let witness = context.get("source_equivalence").ok_or_else(|| {
        // A missing closure is not proof that a package is absent. Name the
        // unresolved bootstrap without starting it or accepting its hooks.
        if session_start_sdk_bootstrap_declared(context) {
            "permission_session_start_sdk_dependency_unattested".into()
        } else {
            equivalence_fail_at(ProjectionGuard::G065)
        }
    })?;
    if witness["scope"] != "origin" {
        return Err(equivalence_fail_at(ProjectionGuard::G066));
    }
    Ok(&witness["manifest"])
}
fn mapping_equivalence_binding(
    parent: &ProviderPermissions,
    policy: &Value,
    mapping: &crate::registry::ProjectMapping,
) -> Result<Value, String> {
    Ok(
        json!({"parent_revision":parent.revision,"parent_run_id":parent.run_id,"parent_session_id":parent.provider_session_id,
        "parent_instance_id":parent.provider_instance_id,"parent_digest":equivalence_sha(&serde_json::to_vec(parent).map_err(|_| equivalence_fail_at(ProjectionGuard::G067))?),
        "child_policy_digest":equivalence_policy_digest(policy)?,"source_host":mapping.source_host,"source_root":mapping.source_root,
        "remote_root":mapping.remote_root,"mapping_revision":mapping.mapping_revision,"source_identity":mapping.expected_git_identity}),
    )
}
fn check_equivalence_mapping(
    binding: &Value,
    mapping: &crate::registry::ProjectMapping,
) -> Result<(), String> {
    if binding["source_host"] != mapping.source_host
        || binding["source_root"] != json!(mapping.source_root)
        || binding["remote_root"] != json!(mapping.remote_root)
        || binding["mapping_revision"] != mapping.mapping_revision
        || binding["source_identity"] != mapping.expected_git_identity
    {
        return Err(equivalence_fail_at(ProjectionGuard::G068));
    }
    Ok(())
}
fn guarded_sdk_settings171(profile: &Value, context: &Value) -> Result<Option<Value>, String> {
    let Some(sdk) = context.get("settings_overrides") else { return Ok(None); };
    if *sdk != sdk_settings171() { return Err("permission_mapping_unavailable".into()); }
    let path = Path::new(profile["settings"]["target"]["path"].as_str().ok_or_else(unavailable)?);
    let base = read_json_source(path)?;
    Ok(Some(compose_sdk_settings171(&base, sdk)?))
}
fn compose_sdk_settings171(base: &Value, sdk: &Value) -> Result<Value, String> {
    if *sdk != sdk_settings171() || !base.is_object() { return Err("permission_mapping_unavailable".into()); }
    let mut result = base.clone();
    let permissions = result.as_object_mut().ok_or_else(unavailable)?
        .entry("permissions").or_insert_with(|| json!({})).as_object_mut().ok_or_else(unavailable)?;
    let allow = permissions.entry("allow").or_insert_with(|| json!([])).as_array_mut().ok_or_else(unavailable)?;
    if allow.iter().any(|v| !v.is_string()) { return Err(unavailable()); }
    for rule in sdk["permissions"]["allow"].as_array().ok_or_else(unavailable)? {
        if !allow.contains(rule) { allow.push(rule.clone()); }
    }
    // All native denies/asks, modes, hooks, and other settings stay byte-for-value.
    Ok(result)
}
fn validate_guarded_settings_args171(definition: &AgentDefinition, profile: &Value, context: &Value) -> Result<(), String> {
    let Some(expected) = guarded_sdk_settings171(profile, context)? else { return Ok(()); };
    require_sdk_native_mount171(context, definition)?;
    let selected = profile["settings"]["target"]["path"].as_str().ok_or_else(unavailable)?;
    let mut count = 0;
    for (i,arg) in definition.args.iter().enumerate() {
        let value = if arg == "--settings" { Some(definition.args.get(i+1).ok_or_else(unavailable)?.as_str()) }
            else { arg.strip_prefix("--settings=") };
        if let Some(value) = value {
            count += 1;
            if value != selected && serde_json::from_str::<Value>(value).ok().as_ref() != Some(&expected) {
                return Err(changed());
            }
        }
    }
    if count > 1 { return Err(changed()); }
    Ok(())
}
fn guarded_profile_definition(definition: &AgentDefinition, profile: &Value, context: &Value) -> Result<AgentDefinition, String> {
    let selected = profile["settings"]["target"]["path"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G069))?;
    validate_guarded_settings_args171(definition, profile, context)?;
    let mut scoped = definition.clone();
    let mut args = Vec::new();
    let mut index = 0;
    while index < scoped.args.len() {
        let arg = &scoped.args[index];
        let flag = arg.split('=').next().unwrap_or(arg);
        if matches!(flag, "--settings" | "--setting-sources" | "--plugin-dir") {
            index += if arg.contains('=') { 1 } else { 2 };
        } else { args.push(arg.clone()); index += 1; }
    }
    args.extend(["--settings".into(), selected.into(), "--setting-sources".into(), "".into()]);
    for plugin in profile["target_controls"]["plugins"].as_array().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G070))? {
        args.extend(["--plugin-dir".into(), plugin["private_root"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G071))?.into()]);
    }
    scoped.args = args;
    Ok(scoped)
}

fn prepare_guarded_remote_context(
    parent: &ProviderPermissions, policy: &Value, definition: &AgentDefinition,
    mapping: &crate::registry::ProjectMapping, cwd: &Path, source: &SourceEnvironment,
    operation: &mut PermissionReadOperation,
) -> Result<Value, String> {
    // A registry-provided source selector is not silently replaced. This
    // route only accepts the default native selection captured in its proof.
    if definition.args.iter().any(|a| matches!(a.split('=').next(),
        Some("--settings" | "--setting-sources" | "--plugin-dir" | "--agent" | "--agents"))) {
        return Err(equivalence_fail_at(ProjectionGuard::G072));
    }
    let origin = &parent.provider_policy["launch_context"];
    let mut context = origin.clone();
    context["hook_execution"] = json!("native_remote_guarded");
    context["settings_sources"] = json!([]);
    context["source_equivalence"] = json!({"version":2,"scope":"remote",
        "profile":origin["source_equivalence"]["profile"],
        "origin_sources":origin["permission_sources"],
        "binding":mapping_equivalence_binding(parent, policy, mapping)?});
    let profile = bridget_transport::protocol::read_guarded_hook_profile_manifest(&context)?;
    check_profile_additional_root171(&profile, mapping)?;
    check_parent_additional_root171(parent, &profile)?;
    context["permission_sources"] = profile["target_controls"]["sources"].clone();
    // Reads actual target declarations and every privately reconstructed
    // native plugin, before any target permission context is captured.
    operation.profile(&context)?;
    let environment = build_environment(definition, source).map_err(|_| unavailable())?;
    capture_guarded_context(definition, &context, cwd, &environment, operation)
}

fn capture_guarded_context(
    definition: &AgentDefinition, context: &Value, cwd: &Path,
    environment: &SourceEnvironment,
    operation: &mut PermissionReadOperation,
) -> Result<Value, String> {
    let verified = operation.profile(context)?;
    let profile = verified.profile(context)?;
    let config = environment.get("CLAUDE_CONFIG_DIR").map(PathBuf::from)
        .or_else(|| environment.get("HOME").map(|p| PathBuf::from(p).join(".claude")))
        .ok_or_else(unavailable)?;
    if std::fs::canonicalize(&config).map_err(|_| unavailable())?
        != PathBuf::from(profile["target_controls"]["config_dir"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G073))?) {
        return Err(equivalence_fail_at(ProjectionGuard::G074));
    }
    let scoped = guarded_profile_definition(definition, profile, context)?;
    let mut reader = scoped.clone();
    let mut args = Vec::new(); let mut index = 0;
    while index < reader.args.len() {
        if reader.args[index] == "--plugin-dir" { index += 2; }
        else { args.push(reader.args[index].clone()); index += 1; }
    }
    reader.args = args;
    let mut captured = capture_context_with_env_impl(&reader, cwd, environment, true, false)?;
    let sources = captured["permission_sources"].as_array_mut().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G075))?;
    for plugin in profile["target_controls"]["plugins"].as_array().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G076))? {
        let root = Path::new(plugin["private_root"].as_str().ok_or_else(|| equivalence_fail_at(ProjectionGuard::G077))?);
        for relative in [".claude-plugin/plugin.json", "hooks/hooks.json", "settings.json", ".mcp.json"] {
            let path = root.join(relative);
            security_source_is_observable(&path)?;
            sources.push(json!({"kind":"user","path":path,"revision":source_revision(&path)?}));
            if path.is_file() {
                let value = read_json_source(&path)?;
                let mut hooks = Vec::new();
                append_hook_sources(&value, &path, cwd, root, environment, &mut hooks)?;
                for (kind,path) in hooks {
                    let revision = bridget_transport::protocol::permission_hook_source_revision(&path,16*1024*1024)?;
                    if revision == "absent" { return Err(unavailable()); }
                    sources.push(json!({"kind":kind,"path":path,"revision":revision}));
                }
            }
        }
    }
    sources.sort_by(|a,b| (a["kind"].as_str(),a["path"].as_str()).cmp(&(b["kind"].as_str(),b["path"].as_str())));
    sources.dedup_by(|a,b| a["kind"]==b["kind"] && a["path"]==b["path"]);
    if sources.len()>128 { return Err(unavailable()); }
    if let Some(sdk) = context.get("settings_overrides") {
        if *sdk != sdk_settings171() || captured.get("settings_overrides").is_some() { return Err(changed()); }
        captured["settings_overrides"] = sdk.clone();
    }
    captured["hook_execution"] = json!("native_remote_guarded");
    captured["source_equivalence"] = context["source_equivalence"].clone();
    operation.profile(&captured)?;
    Ok(captured)
}

fn attach_guarded_remote_profile(
    parent: &ProviderPermissions, policy: &mut Value, definition: &AgentDefinition,
    mapping: &crate::registry::ProjectMapping, cwd: &Path, source: &SourceEnvironment,
    operation: &mut PermissionReadOperation,
) -> Result<(), String> {
    let origin = &parent.provider_policy["launch_context"];
    bridget_transport::protocol::validate_guarded_hook_witness(origin)?;
    let descriptor = origin["source_equivalence"]["profile"].clone();
    let binding = mapping_equivalence_binding(parent, policy, mapping)?;
    let mut context = policy["launch_context"].clone();
    context["hook_execution"] = json!("native_remote_guarded");
    context["source_equivalence"] = json!({"version":2,"scope":"remote","profile":descriptor,
        "binding":binding,"origin_sources":origin["permission_sources"]});
    // This independent target check is required before settings may be passed
    // to the native CLI. No origin file is opened on the target.
    let verified = operation.profile(&context)?;
    let profile = verified.profile(&context)?;
    check_profile_additional_root171(profile, mapping)?;
    check_parent_additional_root171(parent, profile)?;
    if profile["mapping"]["remote_root"] != json!(mapping.remote_root)
        || profile["mapping"]["source_root"] != json!(mapping.source_root) { return Err(equivalence_fail_at(ProjectionGuard::G078)); }
    context["settings_sources"] = json!([]);
    policy["launch_context"] = context;
    // Capture the exact --settings file and --setting-sources reduction that
    // the native child will receive. Discovery tools/modes are unchanged.
    let mut scoped = definition.clone();
    apply_child_arguments_in_operation(&scoped.protocol, policy, &mut scoped.args, operation)?;
    let environment = build_environment(&scoped, source).map_err(|_| unavailable())?;
    let mut actual = capture_guarded_context(&scoped, &policy["launch_context"], cwd, &environment, operation)?;
    actual["hook_execution"] = json!("native_remote_guarded");
    actual["source_equivalence"] = policy["launch_context"]["source_equivalence"].clone();
    policy["launch_context"] = actual;
    Ok(())
}
fn attach_remote_equivalence(
    context: &mut Value,
    origin: &Value,
    binding: Value,
) -> Result<(), String> {
    let target = origin_equivalence(context)?.clone();
    if origin["project_root"] != binding["source_root"]
        || target["project_root"] != binding["remote_root"]
    {
        return Err(equivalence_fail_at(ProjectionGuard::G079));
    }
    let pairs = bridget_transport::protocol::permission_source_equivalence_pairs(origin, &target)?;
    context["hook_execution"] = json!("native_remote_equivalent");
    context["source_equivalence"] = json!({"version":1,"scope":"remote","origin":origin,"target":target,"binding":binding,"pairs":pairs});
    if serde_json::to_vec(context)
        .map_err(|_| equivalence_fail_at(ProjectionGuard::G080))?
        .len()
        > 48 * 1024
    {
        return Err(equivalence_fail_at(ProjectionGuard::G081));
    }
    bridget_transport::protocol::validate_permission_source_equivalence(context)
}
pub(crate) fn refresh_remote_permissions_context(
    policy: &mut Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
    mapping: &crate::registry::ProjectMapping,
) -> Result<(), String> {
    let mut operation = PermissionReadOperation::default();
    let result = refresh_remote_permissions_context_in_operation(policy, definition, cwd, source, mapping, &mut operation)?;
    operation.finish()?;
    Ok(result)
}

pub(crate) fn refresh_remote_permissions_context_in_operation(
    policy: &mut Value,
    definition: &AgentDefinition,
    cwd: &Path,
    source: &SourceEnvironment,
    mapping: &crate::registry::ProjectMapping,
    operation: &mut PermissionReadOperation,
) -> Result<(), String> {
    let previous = policy
        .get("launch_context")
        .ok_or_else(unavailable)?
        .clone();
    let scoped;
    let definition = if previous["hook_execution"] == "native_remote_guarded" {
        let verified = operation.profile(&previous)?;
        scoped = guarded_profile_definition(definition, verified.profile(&previous)?, &previous)?;
        &scoped
    } else { definition };
    let mut captured = if previous["hook_execution"] == "native_remote_guarded" {
        let environment = build_environment(definition, source).map_err(|_| unavailable())?;
        capture_guarded_context(definition, &previous, cwd, &environment, operation)?
    } else if previous.get("hook_execution").is_none() {
        capture_native_target_context(definition, cwd, source)?
    } else { capture_context(definition, cwd, source)? };
    if previous["hook_execution"] == "native_remote_guarded" {
        let witness = &previous["source_equivalence"];
        check_equivalence_mapping(&witness["binding"], mapping)?;
        let verified = operation.profile(&previous)?;
        check_profile_additional_root171(verified.profile(&previous)?, mapping)?;
        if witness["binding"]["child_policy_digest"] != equivalence_policy_digest(policy)? { return Err(equivalence_fail_at(ProjectionGuard::G082)); }
        captured["hook_execution"] = json!("native_remote_guarded");
        captured["source_equivalence"] = witness.clone();
        operation.profile(&captured)?;
    } else if previous["hook_execution"] == "native_remote_equivalent" {
        bridget_transport::protocol::validate_permission_source_equivalence(&previous)?;
        let witness = &previous["source_equivalence"];
        check_equivalence_mapping(&witness["binding"], mapping)?;
        if witness["binding"]["child_policy_digest"] != equivalence_policy_digest(policy)? {
            return Err(equivalence_fail_at(ProjectionGuard::G083));
        }
        attach_remote_equivalence(
            &mut captured,
            &witness["origin"],
            witness["binding"].clone(),
        )?;
    }
    policy["launch_context"] = captured;
    Ok(())
}

#[cfg(test)]
#[path = "native_permissions149_tests.rs"]
mod native_permissions149_tests;
