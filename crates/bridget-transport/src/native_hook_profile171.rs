//! Closed SPEC171 profile descriptor. Native transport, not a second RPC.
use serde_json::{Value, json};
use std::{path::Path, process::{Command, Stdio}, io::{Read, Write}, time::{Duration, Instant}};

const CHECKER: &str = include_str!("../../../scripts/real_hook_profile171.py");
fn fail() -> String { "permission_hook_profile_unavailable".into() }
fn keys(v: &Value, names: &[&str]) -> bool {
    v.as_object().is_some_and(|o| o.len() == names.len() && names.iter().all(|n| o.contains_key(*n)))
}
fn path(v: &Value) -> bool {
    v.as_str().is_some_and(|s| Path::new(s).is_absolute() && s.len() <= 4096
        && !s.chars().any(char::is_control)
        && !Path::new(s).components().any(|p| matches!(p, std::path::Component::ParentDir)))
}
fn digest(v: &Value) -> bool {
    v.as_str().is_some_and(|s| s.len() == 71 && s.starts_with("sha256:")
        && s[7..].bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)))
}

pub fn validate_guarded_hook_witness(context: &Value) -> Result<(), String> {
    let w = &context["source_equivalence"];
    let origin = w["scope"] == "origin";
    if context["attestation_version"] != 2 || w["version"] != 2
        || !keys(w, if origin { &["version","scope","profile"] }
                    else { &["version","scope","profile","binding","origin_sources"] })
        || (!origin && w["scope"] != "remote")
        || context["hook_execution"] != if origin { "native_same_host" } else { "native_remote_guarded" }
    { return Err(fail()); }
    let p = &w["profile"];
    if !keys(p, &["origin_path","target_path","revision","origin_python","target_python","origin_python_revision","target_python_revision","origin_environment"])
        || ["origin_path","target_path","origin_python","target_python"].iter().any(|k| !path(&p[*k]))
        || ["revision","origin_python_revision","target_python_revision"].iter().any(|k| !digest(&p[*k]))
    { return Err(fail()); }
    let e = &p["origin_environment"];
    if !keys(e, &["TMUX","TMUX_PANE","CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"])
        || ["TMUX","TMUX_PANE"].iter().any(|k| !e[*k].is_null() && !e[*k].as_str().is_some_and(|s| s.len() <= 4096 && !s.chars().any(char::is_control)))
        || (!e["CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"].is_null() && e["CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"] != "1") {
        return Err(fail());
    }
    if !origin {
        let b = &w["binding"];
        if !keys(b, &["parent_revision","parent_run_id","parent_session_id","parent_instance_id",
                     "parent_digest","child_policy_digest","source_host","source_root","remote_root",
                     "mapping_revision","source_identity"])
            || !b["parent_revision"].as_u64().is_some_and(|n| n > 0)
            || !b["mapping_revision"].as_u64().is_some_and(|n| n > 0)
            || !digest(&b["parent_digest"]) || !digest(&b["child_policy_digest"])
            || !path(&b["source_root"]) || !path(&b["remote_root"])
            || ["parent_run_id","parent_session_id","parent_instance_id","source_host","source_identity"].iter()
                .any(|k| !b[*k].as_str().is_some_and(|s| !s.is_empty() && s.len() <= 4096 && !s.chars().any(char::is_control)))
            || !w["origin_sources"].as_array().is_some_and(|a| !a.is_empty() && a.len() <= 128)
        { return Err(fail()); }
    }
    Ok(())
}

/// The checker is compiled into the transport. A manifest cannot choose its
/// own verifier or pass executable code. Python and every dependency are pinned.
pub fn read_guarded_hook_profile_manifest(context: &Value) -> Result<Value, String> {
    use sha2::{Digest, Sha256};
    use std::os::unix::fs::MetadataExt;
    validate_guarded_hook_witness(context)?;
    let w = &context["source_equivalence"];
    let side = if w["scope"] == "origin" { "origin" } else { "target" };
    let descriptor = &w["profile"];
    let manifest_path = descriptor[format!("{side}_path")].as_str().ok_or_else(fail)?;
    let m = std::fs::metadata(manifest_path).map_err(|_| fail())?;
    if !m.is_file() || m.len() > 8*1024*1024 || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
        return Err(fail());
    }
    let bytes = std::fs::read(manifest_path).map_err(|_| fail())?;
    if json!(format!("sha256:{:x}", Sha256::digest(&bytes))) != descriptor["revision"] { return Err(fail()); }
    let profile: Value = serde_json::from_slice(&bytes).map_err(|_| fail())?;
    let expected_sources = if side == "origin" { &context["permission_sources"] } else { &w["origin_sources"] };
    if profile["wrapper_revision"] != json!(format!("sha256:{:x}", Sha256::digest(CHECKER.as_bytes())))
        || profile["version"] != 2 || profile["kind"] != "native_guarded_hooks"
        || profile["origin_sources"] != *expected_sources { return Err(fail()); }
    if side == "target" {
        for (a,b) in [("source_host","source_host"),("source_root","source_root"),("remote_root","remote_root"),
                      ("mapping_revision","mapping_revision"),("source_identity","source_identity")] {
            if profile["mapping"][a] != w["binding"][b] { return Err(fail()); }
        }
    }
    Ok(profile)
}

/// Preuve locale non forgeable, limitée à la pile de l'opération native.
/// Elle ne dispense jamais le propriétaire de sa vérification finale fraîche.
pub struct VerifiedGuardedHookProfile {
    profile: std::sync::Arc<Value>,
    descriptor: Value,
    scope: Value,
    allowed_sources: std::collections::BTreeSet<String>,
}
impl VerifiedGuardedHookProfile {
    pub fn verify(context: &Value) -> Result<Self, String> {
        let profile = recheck_guarded_hook_profile(context)?;
        let mut allowed_sources = std::collections::BTreeSet::new();
        if let Some(rows) = profile["files"].as_array() {
            for row in rows { if let Some(p) = row["variants"]["target"]["path"].as_str() { allowed_sources.insert(p.to_owned()); } }
        }
        if let Some(rows) = profile["target_controls"]["sources"].as_array() {
            for row in rows { if let Some(p) = row["path"].as_str() { allowed_sources.insert(p.to_owned()); } }
        }
        if let Some(p) = profile["settings"]["target"]["path"].as_str() { allowed_sources.insert(p.to_owned()); }
        if let Some(plugins) = profile["target_controls"]["plugins"].as_array() {
            for plugin in plugins {
                let root = plugin["private_root"].as_str().ok_or_else(fail)?;
                for name in [".claude-plugin/plugin.json", "hooks/hooks.json", "settings.json", ".mcp.json"] {
                    allowed_sources.insert(Path::new(root).join(name).to_string_lossy().into_owned());
                }
            }
        }
        Ok(Self { profile: std::sync::Arc::new(profile), descriptor: context["source_equivalence"]["profile"].clone(),
            scope: context["source_equivalence"]["scope"].clone(), allowed_sources })
    }
    /// Rebuild an origin witness only from this operation's independently
    /// verified descriptor and exactly recaptured native authority sources.
    pub fn origin_witness_for(
        &self, captured: &Value, project: &Path, environment: &Value,
    ) -> Result<Value, String> {
        if self.scope != "origin" || captured["attestation_version"] != 2
            || captured["hook_execution"] != "native_same_host"
            || captured["permission_sources"] != self.profile["origin_sources"]
            || self.profile["mapping"]["source_root"] != json!(project)
            || self.profile["hosts"]["origin"]["project"] != json!(project)
        { return Err("permission_hook_equivalence_unavailable".into()); }
        if *environment != self.profile["hosts"]["origin"]["environment"]
            || *environment != self.descriptor["origin_environment"]
        { return Err("permission_hook_environment_unavailable".into()); }
        let witness = json!({"version":2,"scope":"origin","profile":self.descriptor});
        let mut reconciled = captured.clone();
        reconciled["source_equivalence"] = witness.clone();
        // The opaque token checks its descriptor, scope and source provenance.
        self.profile(&reconciled)?;
        Ok(witness)
    }
    pub fn same_profile(&self, context: &Value) -> bool {
        self.descriptor == context["source_equivalence"]["profile"] && self.scope == context["source_equivalence"]["scope"]
    }
    /// Construction pure : toutes les autorités de chaque contexte sont
    /// comparées au profil déjà vérifié, sans exécuter un second vérificateur.
    pub fn profile<'a>(&'a self, context: &Value) -> Result<&'a Value, String> {
        validate_guarded_hook_witness(context)?;
        if !self.same_profile(context) { return Err(fail()); }
        let w = &context["source_equivalence"];
        if w["scope"] == "origin" {
            if self.profile["origin_sources"] != context["permission_sources"] { return Err(fail()); }
        } else {
            if self.profile["origin_sources"] != w["origin_sources"] { return Err(fail()); }
            for (a,b) in [("source_host","source_host"),("source_root","source_root"),("remote_root","remote_root"),
                          ("mapping_revision","mapping_revision"),("source_identity","source_identity")] {
                if self.profile["mapping"][a] != w["binding"][b] { return Err(fail()); }
            }
            for source in context["permission_sources"].as_array().ok_or_else(fail)? {
                if !source["path"].as_str().is_some_and(|p| self.allowed_sources.contains(p)) {
                    return Err("permission_hook_target_controls_unavailable".into());
                }
            }
        }
        Ok(&self.profile)
    }
}

pub fn recheck_guarded_hook_profile(context: &Value) -> Result<Value, String> {
    let profile = read_guarded_hook_profile_manifest(context)?;
    let w = &context["source_equivalence"];
    let side = if w["scope"] == "origin" { "origin" } else { "target" };
    let descriptor = &w["profile"];
    let manifest_path = descriptor[format!("{side}_path")].as_str().ok_or_else(fail)?;
    let python = descriptor[format!("{side}_python")].as_str().ok_or_else(fail)?;
    if crate::protocol::permission_source_revision_v2(Path::new(python), 512*1024*1024)?
        != descriptor[format!("{side}_python_revision")] { return Err(fail()); }
    let mut child = Command::new(python).args(["-I","-B","-c",CHECKER,"verify","--manifest",manifest_path,
                    "--revision",descriptor["revision"].as_str().ok_or_else(fail)?,"--side",side])
        .env_clear().env("PATH","/usr/bin:/bin").env("LANG","C.UTF-8")
        .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().map_err(|_| fail())?;
    if let Some(mut input) = child.stdin.take() {
        if input.write_all(&serde_json::to_vec(context).map_err(|_| fail())?).is_err() {
            let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }; let _ = child.wait(); return Err(fail());
        }
    }
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(profile),
            Ok(Some(_)) => {
                let mut raw = Vec::new();
                if let Some(stderr) = child.stderr.take() { let _ = stderr.take(4097).read_to_end(&mut raw); }
                return Err(guarded_hook_refusal(&raw));
            },
            Err(_) => { let _=unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }; let _=child.wait(); return Err(fail()); }
            _ if Instant::now() >= deadline => { let _=unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }; let _=child.wait(); return Err("permission_hook_profile_timeout".into()); }
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn guarded_hook_refusal(raw: &[u8]) -> String {
    if raw.len() > 4096 { return fail(); }
    let parsed: Value = match serde_json::from_slice(raw) { Ok(v) => v, Err(_) => return fail() };
    let code = parsed["code"].as_str().unwrap_or("");
    if parsed["status"] != "refused" { return fail(); }
    match code {
        "permission_hook_target_controls_unavailable" | "permission_hook_target_controls_changed" | "permission_hook_unattested_settings_input" | "permission_hook_capture_changed" | "permission_hook_capture_unavailable" | "permission_hook_command_unavailable" | "permission_hook_control_changed" | "permission_hook_dependency_changed" | "permission_hook_dependency_unavailable" | "permission_hook_environment_unavailable" | "permission_hook_event_changed" | "permission_hook_execution_unavailable" | "permission_hook_git_config_changed" | "permission_hook_git_state_unavailable" | "permission_hook_input_budget" | "permission_hook_input_outside_project" | "permission_hook_input_unavailable" | "permission_hook_matcher_changed" | "permission_hook_path_program_changed" | "permission_hook_permissions_changed" | "permission_hook_permissions_unavailable" | "permission_hook_platform_unavailable" | "permission_hook_profile_budget" | "permission_hook_profile_changed" | "permission_hook_profile_incomplete" | "permission_hook_profile_path" | "permission_hook_profile_schema" | "permission_hook_prompt_changed" | "permission_hook_recipe_source_changed" | "permission_hook_recipe_unavailable" | "permission_hook_selection_changed" | "permission_hook_selection_unavailable" | "permission_hook_settings_changed" | "permission_hook_type_unavailable" | "permission_hook_unknown_dynamic_dependency" | "permission_hook_unknown_path_program" | "permission_hook_unmapped_permission_path" | "permission_hook_unported_ralph_state" | "permission_hook_unported_skills_repository" | "permission_hook_unported_skills_write" | "permission_hook_unported_tmux_state" | "permission_hook_unqualified_branch" | "permission_hook_unqualified_html_branch" | "permission_hook_unqualified_security_change" | "permission_hook_unqualified_security_commit" | "permission_hook_unqualified_security_edit" | "permission_hook_wrapper_changed" | "permission_hook_wrapper_unavailable" | "permission_mapping_unavailable" | "permission_source_unavailable" | "settings_revision_changed" => {
            log::warn!("bridget_hook_profile_refused code={}", code);
            code.into()
        },
        _ => fail(),
    }
}
