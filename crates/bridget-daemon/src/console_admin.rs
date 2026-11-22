//! Routes d'administration de la console locale Bridget :
//! - Paramètres (`automanaged`, synchronisation flotte, politique de routage gardée)
//! - Machines (inventaire multi-hôtes, ajout/édition `~/.ssh/config`, réparations ciblées)
//! - Déploiement / mise à jour contrôlée des 5 clients CLI officiels
//!   (`claude`, `codex`, `antigravity`, `cursor`, `gemini`).

use crate::accounts_connect::{load_fleet_connections_with_machines, valid_machine_id};
use crate::console_ui::{
    ConsoleState, check_private_auth, error_response, json_with_headers, resolve_registry_authority,
};
use crate::registry::AgentRegistry;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Json, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SSH_CONNECT_TIMEOUT: &str = "4";
const PROBE_TIMEOUT_SECS: u64 = 10;
const INSTALL_TIMEOUT_SECS: u64 = 240;
const MIN_INTERVAL_SECS: u64 = 900;
const MAX_INTERVAL_SECS: u64 = 604_800;
const PREVIEW_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub(crate) struct BoundRoutingPreview {
    pub session_id: String,
    pub registry_path: PathBuf,
    pub expected_sha256: String,
    pub mutation: RoutingMutation,
    pub expires_at: Instant,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RoutingMutation {
    SetGlobalEnabled {
        enabled: bool,
    },
    SetMachineEnabled {
        machine_id: String,
        enabled: bool,
    },
    SetMachineCapacity {
        machine_id: String,
        capacity: u32,
    },
    SetConnectionEnabled {
        connection_id: String,
        enabled: bool,
    },
}

#[derive(Deserialize)]
pub(crate) struct AutomanagedUpdateRequest {
    pub cli_updates: Option<bool>,
    pub model_sync: Option<bool>,
    pub interval_secs: Option<u64>,
    #[serde(default = "default_true")]
    pub propagate_remotes: bool,
}

#[derive(Deserialize)]
pub(crate) struct SyncFleetRequest {
    #[serde(default = "default_true")]
    pub cli: bool,
    #[serde(default = "default_true")]
    pub models: bool,
}

#[derive(Deserialize)]
pub(crate) struct RoutingApplyRequest {
    pub preview_id: String,
}

#[derive(Deserialize)]
pub(crate) struct SshAddRequest {
    pub alias: String,
    pub hostname: String,
    pub user: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub ipv4_only: bool,
    #[serde(default = "default_true")]
    pub accept_host_key: bool,
}

#[derive(Deserialize)]
pub(crate) struct MachineRepairRequest {
    pub machine_id: String,
    pub action: String,
}

#[derive(Deserialize)]
pub(crate) struct ClientDeployRequest {
    pub machine_id: String,
    pub client_id: String,
    #[serde(default)]
    pub mode: String, // "install" ou "update"
}

fn default_true() -> bool {
    true
}
fn default_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CliCatalogItem {
    pub id: &'static str,
    pub label: &'static str,
    pub binary: &'static str,
    pub install_summary: &'static str,
    pub update_summary: &'static str,
    pub login_hint: &'static str,
}

pub(crate) const CLI_CATALOG: &[CliCatalogItem] = &[
    CliCatalogItem {
        id: "claude",
        label: "Claude Code",
        binary: "claude",
        install_summary: "Installateur officiel https://claude.ai/install.sh (bash)",
        update_summary: "claude update",
        login_hint: "claude auth login",
    },
    CliCatalogItem {
        id: "codex",
        label: "Codex",
        binary: "codex",
        install_summary: "Installateur officiel https://chatgpt.com/codex/install.sh (sh)",
        update_summary: "codex update (ou brew upgrade --cask codex sur macOS Homebrew)",
        login_hint: "codex login",
    },
    CliCatalogItem {
        id: "antigravity",
        label: "Antigravity",
        binary: "agy",
        install_summary: "Installateur officiel https://antigravity.google/cli/install.sh (sh)",
        update_summary: "agy update",
        login_hint: "agy acp --login",
    },
    CliCatalogItem {
        id: "cursor",
        label: "Cursor CLI",
        binary: "cursor-agent",
        install_summary: "Installateur officiel https://cursor.com/install (bash)",
        update_summary: "Ré-exécution officielle https://cursor.com/install",
        login_hint: "cursor-agent login",
    },
    CliCatalogItem {
        id: "gemini",
        label: "Gemini CLI",
        binary: "gemini",
        install_summary: "npm install --global --prefix \"$HOME/.local\" @google/gemini-cli (Node >= 20)",
        update_summary: "npm install --global --prefix \"$HOME/.local\" @google/gemini-cli@latest",
        login_hint: "gemini",
    },
];

pub(crate) fn catalog_item(client_id: &str) -> Option<&'static CliCatalogItem> {
    CLI_CATALOG.iter().find(|c| c.id == client_id)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn wait_child_bounded(mut child: Child, timeout_secs: u64) -> Result<std::process::Output, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return child.wait_with_output().map_err(|e| e.to_string());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("délai dépassé".into());
        }
        std::thread::sleep(Duration::from_millis(40));
    }
}

fn local_machine_name() -> String {
    let host = crate::build_info::local_host();
    if host.is_empty() {
        "local".to_string()
    } else {
        host
    }
}

pub(crate) fn valid_ssh_hostname(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && !host.starts_with('-')
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-_:".contains(&b))
}

pub(crate) fn valid_ssh_user(user: &str) -> bool {
    !user.is_empty()
        && user.len() <= 64
        && !user.starts_with('-')
        && user
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

pub(crate) fn parse_ssh_config_hosts(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut current: Vec<String> = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        let lower = key.to_ascii_lowercase();
        if lower == "host" {
            current = parts
                .filter(|h| !h.contains('*') && !h.contains('?') && valid_machine_id(h))
                .map(str::to_string)
                .collect();
            for h in &current {
                out.entry(h.clone()).or_default();
            }
        } else if lower == "match" {
            current.clear();
        } else if !current.is_empty() {
            let val = parts.collect::<Vec<_>>().join(" ");
            if !val.is_empty() {
                for h in &current {
                    out.entry(h.clone())
                        .or_default()
                        .insert(lower.clone(), val.clone());
                }
            }
        }
    }
    out
}

pub(crate) fn upsert_ssh_config_file(
    config_path: &Path,
    alias: &str,
    hostname: &str,
    user: &str,
    port: u16,
    ipv4_only: bool,
) -> Result<(), String> {
    if !valid_machine_id(alias) {
        return Err("alias SSH invalide (lettres, chiffres, ., _, - uniquement)".into());
    }
    if !valid_ssh_hostname(hostname) {
        return Err("adresse ou nom d'hôte invalide".into());
    }
    if !valid_ssh_user(user) {
        return Err("nom d'utilisateur SSH invalide".into());
    }
    if port == 0 {
        return Err("port SSH invalide".into());
    }
    let parent = config_path
        .parent()
        .ok_or("dossier parent .ssh introuvable")?;
    fs::create_dir_all(parent).map_err(|e| format!("création .ssh impossible : {e}"))?;
    let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));

    let existing = fs::read_to_string(config_path).unwrap_or_default();
    // Retire un bloc précédent strictement dédié à cet alias (`Host <alias>`).
    let mut kept_lines: Vec<&str> = Vec::new();
    let mut skipping = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("host ") || lower.starts_with("match ") {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if lower.starts_with("host ") && tokens.len() == 2 && tokens[1] == alias {
                skipping = true;
                continue;
            }
            skipping = false;
        }
        if !skipping {
            kept_lines.push(line);
        }
    }
    while kept_lines.last().is_some_and(|l| l.trim().is_empty()) {
        kept_lines.pop();
    }
    let mut next = kept_lines.join("\n");
    if !next.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(&format!("# Ajouté via Bridget Console ({alias})\n"));
    next.push_str(&format!("Host {alias}\n"));
    next.push_str(&format!("    HostName {hostname}\n"));
    if ipv4_only {
        next.push_str("    AddressFamily inet\n");
    }
    next.push_str(&format!("    User {user}\n"));
    next.push_str(&format!("    Port {port}\n"));
    next.push_str("    ServerAliveInterval 30\n");
    next.push_str("    ServerAliveCountMax 3\n");

    let tmp = parent.join(format!(
        ".config.bridget-{}.tmp",
        uuid::Uuid::new_v4().simple()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&tmp)
        .map_err(|e| format!("écriture temporaire ~/.ssh/config impossible : {e}"))?;
    file.write_all(next.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("synchronisation ~/.ssh/config impossible : {e}"))?;
    drop(file);
    fs::rename(&tmp, config_path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("remplacement ~/.ssh/config impossible : {e}")
    })?;
    let _ = fs::set_permissions(config_path, fs::Permissions::from_mode(0o600));
    Ok(())
}

// ---------------------------------------------------------------------------
// Lot A — Paramètres Bridget (automanaged + politique de routage)
// ---------------------------------------------------------------------------

fn resolve_target_registry(state: &ConsoleState) -> Result<PathBuf, String> {
    if let Some(p) = state.registry_override_ref() {
        return Ok(p);
    }
    crate::registry::config_path()
}

pub(crate) async fn route_api_settings_get(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, false) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let reg_res = resolve_target_registry(&state);
    let attested = state.attested_default_registry_ref();
    let res = tokio::task::spawn_blocking(move || -> Value {
        let auto = crate::automanaged::status_payload();
        let local_m = local_machine_name();
        let routing = match reg_res {
            Ok(path) => match crate::registry::read_private_registry(&path) {
                Ok(raw) => match serde_json::from_str::<Value>(&raw) {
                    Ok(val) => {
                        let auth = resolve_registry_authority(
                            &path,
                            &local_m,
                            true,
                            &local_m,
                            attested.as_deref(),
                        );
                        let rp = val.get("routing_policy").cloned().unwrap_or(Value::Null);
                        json!({
                            "registry_path": path.to_string_lossy(),
                            "registry_sha256": sha256_hex(raw.as_bytes()),
                            "owner_kind": auth.owner_kind,
                            "owner_badge": auth.owner_badge,
                            "writable": auth.writable,
                            "routing_policy": rp
                        })
                    }
                    Err(e) => json!({ "error": format!("JSON invalide : {e}"), "writable": false }),
                },
                Err(e) => json!({ "error": e, "writable": false }),
            },
            Err(e) => json!({ "error": e, "writable": false }),
        };
        json!({
            "local_machine": local_m,
            "bridget_version": env!("CARGO_PKG_VERSION"),
            "bridget_build_id": crate::build_info::BUILD_ID,
            "automanaged": auto,
            "routing": routing
        })
    })
    .await;
    match res {
        Ok(val) => json_with_headers(StatusCode::OK, val),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "settings_read_failed",
            &e.to_string(),
            "Refusé : lecture des paramètres interrompue",
        ),
    }
}

pub(crate) async fn route_api_settings_automanaged(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<AutomanagedUpdateRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    if let Some(secs) = req.interval_secs
        && !(MIN_INTERVAL_SECS..=MAX_INTERVAL_SECS).contains(&secs)
    {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_interval",
            "La fréquence doit être comprise entre 15 minutes (900 s) et 7 jours (604800 s).",
            "Refusé : fréquence hors bornes",
        );
    }
    let reg_override = state.registry_override_ref();
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let mut cfg = crate::automanaged::load_config();
        if let Some(v) = req.cli_updates {
            cfg.cli_updates = v;
        }
        if let Some(v) = req.model_sync {
            cfg.model_sync = v;
        }
        if let Some(secs) = req.interval_secs {
            cfg.interval_secs = secs;
        }
        crate::automanaged::save_config(&cfg)?;

        let mut propagated = Vec::new();
        if req.propagate_remotes
            && let Ok((local_m, known, _)) =
                load_fleet_connections_with_machines(reg_override.as_deref())
        {
            let cli_cmd = if cfg.cli_updates {
                "on --cli"
            } else {
                "off --cli"
            };
            let mod_cmd = if cfg.model_sync {
                "on --models"
            } else {
                "off --models"
            };
            let remote_sh = format!(
                "\"$HOME/.local/bin/bridget\" automanaged {cli_cmd} >/dev/null 2>&1 || true; \
                 \"$HOME/.local/bin/bridget\" automanaged {mod_cmd} >/dev/null 2>&1 || true"
            );
            let handles: Vec<_> = known
                .into_iter()
                .filter(|m| m != &local_m && valid_machine_id(m))
                .map(|host| {
                    let sh = remote_sh.clone();
                    std::thread::spawn(move || {
                        let ok = Command::new("ssh")
                            .args([
                                "-o",
                                "BatchMode=yes",
                                "-o",
                                &format!("ConnectTimeout={SSH_CONNECT_TIMEOUT}"),
                                &host,
                                &sh,
                            ])
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .is_ok_and(|s| s.success());
                        json!({ "machine_id": host, "propagated": ok })
                    })
                })
                .collect();
            for h in handles {
                if let Ok(item) = h.join() {
                    propagated.push(item);
                }
            }
        }
        Ok(json!({
            "automanaged": crate::automanaged::status_payload(),
            "propagated": propagated,
            "ui_state": "Enregistré et relu"
        }))
    })
    .await;
    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "automanaged_save_failed",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "automanaged_join_error",
            &e.to_string(),
            "Refusé : écriture interrompue",
        ),
    }
}

pub(crate) async fn route_api_settings_sync(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<SyncFleetRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let reg_override = state.registry_override_ref();
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let local_state =
            crate::automanaged::execute_sync(req.cli, req.models, reg_override.as_deref(), false)?;
        let mut remotes = Vec::new();
        if let Ok((local_m, known, _)) =
            load_fleet_connections_with_machines(reg_override.as_deref())
        {
            let mut flags = Vec::new();
            if req.cli {
                flags.push("--cli");
            }
            if req.models {
                flags.push("--models");
            }
            let flag_str = flags.join(" ");
            let remote_sh = format!(
                "\"$HOME/.local/bin/bridget\" automanaged sync {flag_str} --json 2>/dev/null"
            );
            let handles: Vec<_> = known
                .into_iter()
                .filter(|m| m != &local_m && valid_machine_id(m))
                .map(|host| {
                    let sh = remote_sh.clone();
                    std::thread::spawn(move || {
                        let out = Command::new("ssh")
                            .args([
                                "-o",
                                "BatchMode=yes",
                                "-o",
                                &format!("ConnectTimeout={SSH_CONNECT_TIMEOUT}"),
                                &host,
                                &sh,
                            ])
                            .stdin(Stdio::null())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .spawn()
                            .map_err(|e| e.to_string())
                            .and_then(|c| wait_child_bounded(c, 90));
                        match out {
                            Ok(o) if o.status.success() => {
                                let parsed = serde_json::from_slice::<Value>(&o.stdout).ok();
                                json!({ "machine_id": host, "status": "ok", "state": parsed })
                            }
                            _ => json!({ "machine_id": host, "status": "unreachable" }),
                        }
                    })
                })
                .collect();
            for h in handles {
                if let Ok(item) = h.join() {
                    remotes.push(item);
                }
            }
        }
        Ok(json!({
            "local": local_state,
            "remotes": remotes,
            "ui_state": "Synchronisation flotte terminée"
        }))
    })
    .await;
    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "sync_failed",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "sync_join_error",
            &e.to_string(),
            "Refusé : synchronisation interrompue",
        ),
    }
}

pub(crate) fn apply_routing_mutation_to_value(
    val: &mut Value,
    mutation: &RoutingMutation,
) -> Result<(String, Vec<String>), String> {
    let rp = val
        .get_mut("routing_policy")
        .and_then(Value::as_object_mut)
        .ok_or("section routing_policy absente du registre")?;
    let summary = match mutation {
        RoutingMutation::SetGlobalEnabled { enabled } => {
            rp.insert("enabled".into(), json!(enabled));
            (
                if *enabled {
                    "Activer le routage automatique".into()
                } else {
                    "Mettre en pause le routage automatique".into()
                },
                vec![
                    "routing_policy.enabled".into(),
                    "routing_policy.revision".into(),
                ],
            )
        }
        RoutingMutation::SetMachineEnabled {
            machine_id,
            enabled,
        } => {
            let policies = rp
                .get_mut("machine_policies")
                .and_then(Value::as_object_mut)
                .ok_or("machine_policies absent")?;
            let entry = policies
                .get_mut(machine_id)
                .and_then(Value::as_object_mut)
                .ok_or_else(|| {
                    format!("machine introuvable dans machine_policies : {machine_id}")
                })?;
            entry.insert("enabled".into(), json!(enabled));
            (
                format!(
                    "{} la machine {machine_id}",
                    if *enabled { "Activer" } else { "Désactiver" }
                ),
                vec![
                    format!("routing_policy.machine_policies.{machine_id}.enabled"),
                    "routing_policy.revision".into(),
                ],
            )
        }
        RoutingMutation::SetMachineCapacity {
            machine_id,
            capacity,
        } => {
            if !(1..=16).contains(capacity) {
                return Err("la capacité de compilation doit être comprise entre 1 et 16".into());
            }
            let policies = rp
                .get_mut("machine_policies")
                .and_then(Value::as_object_mut)
                .ok_or("machine_policies absent")?;
            let entry = policies
                .get_mut(machine_id)
                .and_then(Value::as_object_mut)
                .ok_or_else(|| {
                    format!("machine introuvable dans machine_policies : {machine_id}")
                })?;
            entry.insert("compilation_capacity".into(), json!(capacity));
            (
                format!("Régler la capacité de {machine_id} à {capacity}"),
                vec![
                    format!("routing_policy.machine_policies.{machine_id}.compilation_capacity"),
                    "routing_policy.revision".into(),
                ],
            )
        }
        RoutingMutation::SetConnectionEnabled {
            connection_id,
            enabled,
        } => {
            let conns = rp
                .get_mut("connections")
                .and_then(Value::as_array_mut)
                .ok_or("connections absent")?;
            let row = conns
                .iter_mut()
                .find(|c| c.get("connection_id").and_then(Value::as_str) == Some(connection_id))
                .and_then(Value::as_object_mut)
                .ok_or_else(|| format!("connexion introuvable : {connection_id}"))?;
            row.insert("enabled".into(), json!(enabled));
            (
                format!(
                    "{} la connexion {connection_id}",
                    if *enabled { "Activer" } else { "Désactiver" }
                ),
                vec![
                    format!("routing_policy.connections[{connection_id}].enabled"),
                    "routing_policy.revision".into(),
                ],
            )
        }
    };
    let next_rev = rp
        .get("revision")
        .and_then(Value::as_u64)
        .and_then(|r| r.checked_add(1))
        .ok_or("révision routing_policy invalide")?;
    rp.insert("revision".into(), json!(next_rev));
    Ok(summary)
}

pub(crate) async fn route_api_routing_preview(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(mutation): Json<RoutingMutation>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let reg_res = resolve_target_registry(&state);
    let attested = state.attested_default_registry_ref();
    let mutation_for_worker = mutation.clone();
    let res = tokio::task::spawn_blocking(
        move || -> Result<(PathBuf, String, String, String, Vec<String>), (StatusCode, String, String)> {
            let reg_path = reg_res.map_err(|e| (StatusCode::BAD_REQUEST, "registry_unavailable".into(), e))?;
            let local_m = local_machine_name();
            let auth = resolve_registry_authority(&reg_path, &local_m, true, &local_m, attested.as_deref());
            if !auth.writable {
                return Err((
                    StatusCode::FORBIDDEN,
                    "registry_read_only".into(),
                    format!("Registre en lecture seule ({}) : modification refusée.", auth.owner_badge),
                ));
            }
            let raw = crate::registry::read_private_registry(&reg_path)
                .map_err(|e| (StatusCode::BAD_REQUEST, "registry_unreadable".into(), e))?;
            let expected_sha = sha256_hex(raw.as_bytes());
            let mut candidate: Value = serde_json::from_str(&raw)
                .map_err(|e| (StatusCode::BAD_REQUEST, "registry_invalid".into(), e.to_string()))?;
            let (summary, fields) = apply_routing_mutation_to_value(&mut candidate, &mutation_for_worker)
                .map_err(|e| (StatusCode::BAD_REQUEST, "routing_mutation_invalid".into(), e))?;
            let candidate_text = serde_json::to_string_pretty(&candidate)
                .map_err(|e| (StatusCode::BAD_REQUEST, "registry_serialize_error".into(), e.to_string()))?;
            AgentRegistry::from_json(&candidate_text, &reg_path)
                .map_err(|e| (StatusCode::BAD_REQUEST, "registry_validation_failed".into(), e))?;
            Ok((reg_path, expected_sha, auth.owner_badge, summary, fields))
        },
    )
    .await;

    match res {
        Ok(Ok((reg_path, expected_sha, owner_badge, summary, fields))) => {
            let preview_id = uuid::Uuid::new_v4().simple().to_string();
            let bound = BoundRoutingPreview {
                session_id: state.session_id_ref().to_string(),
                registry_path: reg_path.clone(),
                expected_sha256: expected_sha.clone(),
                mutation,
                expires_at: Instant::now() + PREVIEW_TTL,
            };
            {
                let mut map = state.routing_previews_lock();
                let now = Instant::now();
                map.retain(|_, p| p.expires_at > now);
                map.insert(preview_id.clone(), bound);
            }
            json_with_headers(
                StatusCode::OK,
                json!({
                    "preview_id": preview_id,
                    "summary": summary,
                    "owner_badge": owner_badge,
                    "registry_path": reg_path.to_string_lossy(),
                    "expected_sha256": expected_sha,
                    "fields_touched": fields
                }),
            )
        }
        Ok(Err((status, code, msg))) => {
            error_response(status, &code, &msg, &format!("Refusé : {msg}"))
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "routing_preview_join",
            &e.to_string(),
            "Refusé : aperçu interrompu",
        ),
    }
}

pub(crate) async fn route_api_routing_apply(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<RoutingApplyRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let bound = state.routing_previews_lock().remove(&req.preview_id);
    let Some(bound) = bound else {
        return error_response(
            StatusCode::CONFLICT,
            "preview_missing_or_consumed",
            "Aperçu absent ou déjà utilisé : rechargez l'aperçu avant de confirmer.",
            "Conflit : rechargez l'aperçu",
        );
    };
    if bound.session_id != state.session_id_ref() || Instant::now() >= bound.expires_at {
        return error_response(
            StatusCode::CONFLICT,
            "preview_expired",
            "Aperçu expiré : rechargez l'aperçu avant de confirmer.",
            "Conflit : rechargez l'aperçu",
        );
    }
    let attested = state.attested_default_registry_ref();
    let res = tokio::task::spawn_blocking(
        move || -> Result<Value, (StatusCode, String, String, String)> {
            let local_m = local_machine_name();
            let auth = resolve_registry_authority(
                &bound.registry_path,
                &local_m,
                true,
                &local_m,
                attested.as_deref(),
            );
            if !auth.writable {
                let msg = format!("Autorité registre perdue ({}).", auth.owner_badge);
                return Err((
                    StatusCode::FORBIDDEN,
                    "registry_read_only".into(),
                    msg.clone(),
                    format!("Refusé : {msg}"),
                ));
            }
            let parent = bound.registry_path.parent().ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "registry_unsafe".into(),
                    "Dossier registre introuvable.".into(),
                    "Refusé : registre invalide".into(),
                )
            })?;
            let lock_path = parent.join("project-preparation.lock");
            let lock_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .mode(0o600)
                .open(&lock_path)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "lock_failed".into(),
                        e.to_string(),
                        "Refusé : verrou indisponible".into(),
                    )
                })?;
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                // SAFETY: flock reçoit un descripteur valide ouvert dans ce scope.
                let rc =
                    unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
                if rc == 0 {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err((
                        StatusCode::CONFLICT,
                        "registry_busy".into(),
                        "Registre occupé par une autre opération.".into(),
                        "Conflit : réessayez dans un instant".into(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            let original =
                crate::registry::read_private_registry(&bound.registry_path).map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        "registry_unreadable".into(),
                        e,
                        "Refusé : registre illisible".into(),
                    )
                })?;
            if sha256_hex(original.as_bytes()) != bound.expected_sha256 {
                return Err((
                    StatusCode::CONFLICT,
                    "account_registry_changed".into(),
                    "Le registre a changé depuis l'aperçu ; rechargez l'aperçu avant de confirmer."
                        .into(),
                    "Conflit : rechargez l'aperçu".into(),
                ));
            }
            let mut next: Value = serde_json::from_str(&original).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    "registry_invalid".into(),
                    e.to_string(),
                    "Refusé : JSON invalide".into(),
                )
            })?;
            let (summary, fields) = apply_routing_mutation_to_value(&mut next, &bound.mutation)
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        "routing_mutation_invalid".into(),
                        e.clone(),
                        format!("Refusé : {e}"),
                    )
                })?;
            let next_text = serde_json::to_string_pretty(&next).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    "registry_serialize_error".into(),
                    e.to_string(),
                    "Refusé : sérialisation impossible".into(),
                )
            })?;
            AgentRegistry::from_json(&next_text, &bound.registry_path).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    "registry_validation_failed".into(),
                    e.clone(),
                    format!("Refusé : {e}"),
                )
            })?;

            // Sauvegarde atomique dans account-registry-backups/
            let backups = parent.join("account-registry-backups");
            crate::environment::ensure_private_directory(&backups).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "backup_dir_failed".into(),
                    e,
                    "Refusé : dossier sauvegarde indisponible".into(),
                )
            })?;
            let backup_file = backups.join(format!("{}.json", bound.expected_sha256));
            if !backup_file.exists() {
                let mut bf = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                    .open(&backup_file)
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "backup_write_failed".into(),
                            e.to_string(),
                            "Refusé : sauvegarde impossible".into(),
                        )
                    })?;
                bf.write_all(original.as_bytes())
                    .and_then(|_| bf.sync_all())
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "backup_sync_failed".into(),
                            e.to_string(),
                            "Refusé : sauvegarde incomplète".into(),
                        )
                    })?;
            }
            let tmp = parent.join(format!(".routing-{}.tmp", uuid::Uuid::new_v4().simple()));
            let mut tf = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(&tmp)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "tmp_write_failed".into(),
                        e.to_string(),
                        "Refusé : écriture temporaire impossible".into(),
                    )
                })?;
            tf.write_all(next_text.as_bytes())
                .and_then(|_| tf.sync_all())
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "tmp_sync_failed".into(),
                        e.to_string(),
                        "Refusé : synchronisation impossible".into(),
                    )
                })?;
            drop(tf);
            fs::rename(&tmp, &bound.registry_path).map_err(|e| {
                let _ = fs::remove_file(&tmp);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "rename_failed".into(),
                    e.to_string(),
                    "Refusé : remplacement impossible".into(),
                )
            })?;
            if let Ok(dir) = File::open(parent) {
                let _ = dir.sync_all();
            }
            drop(lock_file);

            // Relecture disque
            let reread =
                crate::registry::read_private_registry(&bound.registry_path).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "readback_failed".into(),
                        e,
                        "Résultat incertain : relisez avant de réessayer".into(),
                    )
                })?;
            AgentRegistry::from_json(&reread, &bound.registry_path).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "readback_invalid".into(),
                    e,
                    "Résultat incertain : relisez avant de réessayer".into(),
                )
            })?;
            Ok(json!({
                "status": "applied_local",
                "summary": summary,
                "fields_touched": fields,
                "registry_sha256": sha256_hex(reread.as_bytes()),
                "backup": backup_file.to_string_lossy(),
                "ui_state": "Enregistré et relu — prise en compte à confirmer"
            }))
        },
    )
    .await;

    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err((status, code, msg, ui_state))) => error_response(status, &code, &msg, &ui_state),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "routing_apply_join",
            &e.to_string(),
            "Résultat incertain : relisez avant de réessayer",
        ),
    }
}

// ---------------------------------------------------------------------------
// Lot B — Machines & déploiement des 5 clients CLI officiels
// ---------------------------------------------------------------------------

fn build_machine_inventory_script() -> &'static str {
    r#"
export PATH="$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
os=$(uname -s 2>/dev/null || echo unknown)
arch=$(uname -m 2>/dev/null || echo unknown)
hn=$(hostname 2>/dev/null || echo unknown)
bpath=$(command -v bridget 2>/dev/null || true)
bver=""
if [ -n "$bpath" ]; then bver=$("$bpath" --version 2>/dev/null | head -n 1 || true); fi
worker="absent"
if ps aux 2>/dev/null | grep -E 'bridget[[:space:]]+worker' | grep -v grep >/dev/null 2>&1; then
  worker="running"
fi
codex_brew="ok"
if [ "$os" = "Darwin" ] && [ -d /opt/homebrew/bin ] && [ ! -e /opt/homebrew/bin/codex ] && [ -x "$HOME/.local/bin/codex" ]; then
  codex_brew="missing_link"
fi
printf 'META\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$os" "$arch" "$hn" "$bpath" "$bver" "$worker" "$codex_brew"
for pair in "claude:claude" "codex:codex" "antigravity:agy" "cursor:cursor-agent" "gemini:gemini"; do
  cid=${pair%%:*}
  bin=${pair#*:}
  p=$(command -v "$bin" 2>/dev/null || true)
  if [ -n "$p" ]; then
    v=$("$p" --version 2>/dev/null | head -n 1 | tr '\t\r\n' '   ' || true)
    printf 'CLI\t%s\tpresent\t%s\t%s\n' "$cid" "$p" "$v"
  else
    printf 'CLI\t%s\tabsent\t\t\n' "$cid"
  fi
done
"#
}

fn probe_one_machine(
    machine_id: String,
    is_local: bool,
    ssh_entry: Option<BTreeMap<String, String>>,
    local_os: &str,
    local_arch: &str,
) -> Value {
    let script = build_machine_inventory_script();
    let out = if is_local {
        Command::new("bash")
            .args(["-lc", script])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())
            .and_then(|c| wait_child_bounded(c, PROBE_TIMEOUT_SECS))
    } else if valid_machine_id(&machine_id) {
        Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                &format!("ConnectTimeout={SSH_CONNECT_TIMEOUT}"),
                &machine_id,
                "bash -s",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())
            .and_then(|mut c| {
                if let Some(mut stdin) = c.stdin.take() {
                    let _ = stdin.write_all(script.as_bytes());
                }
                wait_child_bounded(c, PROBE_TIMEOUT_SECS)
            })
    } else {
        Err("identifiant machine invalide".into())
    };

    let ssh_configured = is_local || ssh_entry.is_some();
    let ssh_host = ssh_entry.as_ref().and_then(|m| m.get("hostname").cloned());
    let ssh_user = ssh_entry.as_ref().and_then(|m| m.get("user").cloned());
    let ssh_port = ssh_entry
        .as_ref()
        .and_then(|m| m.get("port").and_then(|p| p.parse::<u16>().ok()));

    let mut reachable = false;
    let mut os = String::new();
    let mut arch = String::new();
    let mut hostname = String::new();
    let mut bridget_path = String::new();
    let mut bridget_version = String::new();
    let mut worker_state = "unknown".to_string();
    let mut codex_brew_status = "ok".to_string();
    let mut clients: Vec<Value> = Vec::new();

    if let Ok(output) = out
        && output.status.success()
    {
        reachable = true;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            match parts.first().copied() {
                Some("META") if parts.len() >= 8 => {
                    os = parts[1].to_string();
                    arch = parts[2].to_string();
                    hostname = parts[3].to_string();
                    bridget_path = parts[4].to_string();
                    bridget_version = parts[5].to_string();
                    worker_state = parts[6].to_string();
                    codex_brew_status = parts[7].to_string();
                }
                Some("CLI") if parts.len() >= 5 => {
                    let cid = parts[1];
                    if let Some(cat) = catalog_item(cid) {
                        clients.push(json!({
                            "id": cat.id,
                            "label": cat.label,
                            "binary": cat.binary,
                            "installed": parts[2] == "present",
                            "path": parts[3],
                            "version": parts[4].trim(),
                            "install_summary": cat.install_summary,
                            "update_summary": cat.update_summary,
                            "login_hint": cat.login_hint,
                        }));
                    }
                }
                _ => {}
            }
        }
    }
    if clients.is_empty() {
        for cat in CLI_CATALOG {
            clients.push(json!({
                "id": cat.id,
                "label": cat.label,
                "binary": cat.binary,
                "installed": false,
                "path": "",
                "version": "",
                "install_summary": cat.install_summary,
                "update_summary": cat.update_summary,
                "login_hint": cat.login_hint,
            }));
        }
    }

    let can_sync_bridget = !is_local && reachable && os == local_os && arch == local_arch;
    let mut suggested_repairs: Vec<Value> = Vec::new();
    if !is_local && !reachable {
        suggested_repairs.push(json!({
            "action": "configure_ssh",
            "label": if ssh_configured { "Vérifier l'adresse SSH" } else { "Configurer l'alias SSH" },
            "detail": "Renseigner l'hôte, l'utilisateur et IPv4 dans ~/.ssh/config."
        }));
    }
    if codex_brew_status == "missing_link" {
        suggested_repairs.push(json!({
            "action": "fix_codex_link",
            "label": "Créer le lien /opt/homebrew/bin/codex",
            "detail": "Relie ~/.local/bin/codex au chemin Homebrew attendu par le registre natif."
        }));
    }
    if can_sync_bridget {
        suggested_repairs.push(json!({
            "action": "sync_bridget",
            "label": format!("Déployer Bridget {} et relancer le worker", env!("CARGO_PKG_VERSION")),
            "detail": "Copie le binaire local compatible (même OS/architecture) puis redémarre le worker."
        }));
    }
    if !is_local && reachable && worker_state != "running" {
        suggested_repairs.push(json!({
            "action": "restart_worker",
            "label": "Redémarrer le worker Bridget",
            "detail": "Relance le service worker (launchd ou systemd utilisateur)."
        }));
    }

    json!({
        "machine_id": machine_id,
        "is_local": is_local,
        "reachable": reachable,
        "ssh_configured": ssh_configured,
        "ssh_hostname": ssh_host,
        "ssh_user": ssh_user,
        "ssh_port": ssh_port,
        "os": os,
        "arch": arch,
        "hostname": hostname,
        "bridget_path": bridget_path,
        "bridget_version": bridget_version,
        "worker_state": worker_state,
        "can_sync_bridget": can_sync_bridget,
        "suggested_repairs": suggested_repairs,
        "clients": clients
    })
}

pub(crate) async fn route_api_machines_get(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, false) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let reg_override = state.registry_override_ref();
    let res = tokio::task::spawn_blocking(move || -> Value {
        let local_m = local_machine_name();
        let ssh_path = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".ssh/config"))
            .ok();
        let ssh_hosts = ssh_path
            .as_deref()
            .and_then(|p| fs::read_to_string(p).ok())
            .map(|t| parse_ssh_config_hosts(&t))
            .unwrap_or_default();

        let mut all_machines: BTreeSet<String> = BTreeSet::new();
        all_machines.insert(local_m.clone());
        if let Ok((_, known, _)) = load_fleet_connections_with_machines(reg_override.as_deref()) {
            for m in known {
                all_machines.insert(m);
            }
        }
        // Ajoute les hosts `~/.ssh/config` qui ne sont pas des alias secondaires connus.
        let alias_skip: BTreeSet<&str> = [
            "cartae.app",
            "sysle",
            "sys-le-2",
            "ksstor",
            "ks-stor",
            "orb",
        ]
        .into_iter()
        .collect();
        for h in ssh_hosts.keys() {
            if !alias_skip.contains(h.as_str()) {
                all_machines.insert(h.clone());
            }
        }

        let local_os = if cfg!(target_os = "macos") {
            "Darwin"
        } else {
            "Linux"
        };
        let local_arch = std::env::consts::ARCH;
        let local_uname_arch = match local_arch {
            "aarch64" => "arm64",
            other => other,
        };

        let handles: Vec<_> = all_machines
            .into_iter()
            .map(|m| {
                let is_local = m == local_m;
                let ssh_entry = ssh_hosts.get(&m).cloned();
                let los = local_os.to_string();
                let larch = local_uname_arch.to_string();
                std::thread::spawn(move || probe_one_machine(m, is_local, ssh_entry, &los, &larch))
            })
            .collect();
        let mut machines = Vec::new();
        for h in handles {
            if let Ok(item) = h.join() {
                machines.push(item);
            }
        }
        machines.sort_by(|a, b| {
            let a_loc = a["is_local"].as_bool().unwrap_or(false);
            let b_loc = b["is_local"].as_bool().unwrap_or(false);
            b_loc
                .cmp(&a_loc)
                .then_with(|| a["machine_id"].as_str().cmp(&b["machine_id"].as_str()))
        });
        json!({
            "local_machine": local_m,
            "local_bridget_version": env!("CARGO_PKG_VERSION"),
            "catalog": CLI_CATALOG,
            "machines": machines
        })
    })
    .await;
    match res {
        Ok(val) => json_with_headers(StatusCode::OK, val),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "machines_probe_failed",
            &e.to_string(),
            "Refusé : inventaire des machines interrompu",
        ),
    }
}

pub(crate) async fn route_api_machines_ssh_add(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<SshAddRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let home = std::env::var("HOME").map_err(|_| "variable HOME absente".to_string())?;
        let ssh_config = PathBuf::from(&home).join(".ssh/config");
        upsert_ssh_config_file(
            &ssh_config,
            req.alias.trim(),
            req.hostname.trim(),
            req.user.trim(),
            req.port,
            req.ipv4_only,
        )?;
        let mut args = vec![
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            "ConnectTimeout=5".to_string(),
        ];
        if req.accept_host_key {
            args.push("-o".into());
            args.push("StrictHostKeyChecking=accept-new".into());
        }
        args.push(req.alias.trim().to_string());
        args.push("hostname && uname -sm".into());

        let probe = Command::new("ssh")
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())
            .and_then(|c| wait_child_bounded(c, 8));

        match probe {
            Ok(out) if out.status.success() => {
                let info = String::from_utf8_lossy(&out.stdout).trim().replace('\n', " · ");
                Ok(json!({
                    "status": "connected",
                    "alias": req.alias.trim(),
                    "detail": info,
                    "ui_state": format!("Machine {} enregistrée et joignable ({info})", req.alias.trim())
                }))
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let copy_cmd = format!(
                    "ssh-copy-id -p {} {}@{}",
                    req.port,
                    req.user.trim(),
                    req.hostname.trim()
                );
                Ok(json!({
                    "status": "ssh_saved_unreachable",
                    "alias": req.alias.trim(),
                    "detail": err,
                    "copy_key_hint": copy_cmd,
                    "ui_state": format!("Alias {} enregistré dans ~/.ssh/config, mais accès SSH à autoriser", req.alias.trim())
                }))
            }
            Err(e) => Ok(json!({
                "status": "ssh_saved_unreachable",
                "alias": req.alias.trim(),
                "detail": e,
                "ui_state": format!("Alias {} enregistré dans ~/.ssh/config (test SSH expiré)", req.alias.trim())
            })),
        }
    })
    .await;
    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "ssh_config_invalid",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ssh_add_join_error",
            &e.to_string(),
            "Refusé : opération interrompue",
        ),
    }
}

pub(crate) async fn route_api_machines_repair(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<MachineRepairRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    if !valid_machine_id(&req.machine_id) {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_machine_id",
            "Identifiant de machine invalide.",
            "Refusé : identifiant de machine invalide",
        );
    }
    if !matches!(
        req.action.as_str(),
        "fix_codex_link" | "restart_worker" | "sync_bridget"
    ) {
        return error_response(
            StatusCode::BAD_REQUEST,
            "unknown_repair_action",
            "Action de réparation non reconnue.",
            "Refusé : action inconnue",
        );
    }
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let local_m = local_machine_name();
        let is_local = req.machine_id == local_m;
        let m = req.machine_id.clone();

        let run_remote = |script: &str| -> Result<String, String> {
            let out = if is_local {
                Command::new("bash")
                    .args(["-lc", script])
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| e.to_string())
                    .and_then(|c| wait_child_bounded(c, 30))?
            } else {
                Command::new("ssh")
                    .args([
                        "-o",
                        "BatchMode=yes",
                        "-o",
                        &format!("ConnectTimeout={SSH_CONNECT_TIMEOUT}"),
                        &m,
                        script,
                    ])
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| e.to_string())
                    .and_then(|c| wait_child_bounded(c, 30))?
            };
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if out.status.success() {
                Ok(combined.trim().to_string())
            } else {
                Err(if combined.trim().is_empty() {
                    format!("échec commande (code {:?})", out.status.code())
                } else {
                    combined.trim().to_string()
                })
            }
        };

        match req.action.as_str() {
            "fix_codex_link" => {
                let script = "if [ -d /opt/homebrew/bin ] && [ -x \"$HOME/.local/bin/codex\" ]; then \
                    ln -sfn \"$HOME/.local/bin/codex\" /opt/homebrew/bin/codex && /opt/homebrew/bin/codex --version; \
                    else echo 'Codex introuvable dans ~/.local/bin ou /opt/homebrew/bin absent' >&2; exit 1; fi";
                let detail = run_remote(script)?;
                Ok(json!({
                    "status": "ok",
                    "action": "fix_codex_link",
                    "detail": detail,
                    "ui_state": format!("Lien Codex réparé sur {m} ({detail})")
                }))
            }
            "restart_worker" => {
                let label = format!("com.bridget.worker.{m}");
                let script = format!(
                    "if [ \"$(uname -s)\" = Darwin ]; then \
                       launchctl kickstart -k \"gui/$(id -u)/{label}\" && echo 'worker launchd redémarré'; \
                     else \
                       systemctl --user restart bridget-worker.service && echo 'worker systemd redémarré'; \
                     fi"
                );
                let detail = run_remote(&script)?;
                Ok(json!({
                    "status": "ok",
                    "action": "restart_worker",
                    "detail": detail,
                    "ui_state": format!("Worker Bridget redémarré sur {m}")
                }))
            }
            "sync_bridget" => {
                if is_local {
                    return Err("la machine locale exécute déjà cette version".into());
                }
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let local_os = if cfg!(target_os = "macos") { "Darwin" } else { "Linux" };
                let local_arch = match std::env::consts::ARCH {
                    "aarch64" => "arm64",
                    other => other,
                };
                let remote_info = run_remote("uname -sm")?;
                let expected = format!("{local_os} {local_arch}");
                if remote_info.trim() != expected {
                    return Err(format!(
                        "architecture différente ({remote_info} vs {expected}) : utiliser un build compatible"
                    ));
                }
                let tag = crate::build_info::BUILD_ID;
                let safe_tag = if valid_machine_id(tag) {
                    tag.to_string()
                } else {
                    format!("console-{}", env!("CARGO_PKG_VERSION"))
                };
                run_remote(&format!(
                    "mkdir -p \"$HOME/.local/lib/bridget-communication/{safe_tag}/bin\" && chmod 700 \"$HOME/.local/lib/bridget-communication/{safe_tag}\""
                ))?;
                let dest = format!("{m}:.local/lib/bridget-communication/{safe_tag}/bin/bridget.new");
                let scp = Command::new("scp")
                    .args(["-q", "-o", "BatchMode=yes"])
                    .arg(&exe)
                    .arg(&dest)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| e.to_string())
                    .and_then(|c| wait_child_bounded(c, 60))?;
                if !scp.status.success() {
                    return Err(format!(
                        "copie binaire échouée : {}",
                        String::from_utf8_lossy(&scp.stderr).trim()
                    ));
                }
                let label = format!("com.bridget.worker.{m}");
                let activate = format!(
                    "chmod 755 \"$HOME/.local/lib/bridget-communication/{safe_tag}/bin/bridget.new\" && \
                     mv -f \"$HOME/.local/lib/bridget-communication/{safe_tag}/bin/bridget.new\" \"$HOME/.local/lib/bridget-communication/{safe_tag}/bin/bridget\" && \
                     ln -sfn \"$HOME/.local/lib/bridget-communication/{safe_tag}/bin/bridget\" \"$HOME/.local/bin/bridget\" && \
                     if [ \"$(uname -s)\" = Darwin ]; then launchctl kickstart -k \"gui/$(id -u)/{label}\" >/dev/null 2>&1 || true; \
                     else systemctl --user restart bridget-worker.service >/dev/null 2>&1 || true; fi && \
                     \"$HOME/.local/bin/bridget\" --version"
                );
                let ver = run_remote(&activate)?;
                Ok(json!({
                    "status": "ok",
                    "action": "sync_bridget",
                    "detail": ver,
                    "ui_state": format!("Bridget déployé sur {m} ({ver}) et worker redémarré")
                }))
            }
            _ => Err("action inconnue".into()),
        }
    })
    .await;

    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "repair_failed",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "repair_join_error",
            &e.to_string(),
            "Refusé : réparation interrompue",
        ),
    }
}

pub(crate) fn build_client_deploy_script(
    client_id: &str,
    update_mode: bool,
) -> Result<String, String> {
    let header = "set -e\nexport PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH\"\n";
    let body = match (client_id, update_mode) {
        ("claude", true) => "claude update && claude --version",
        ("claude", false) => {
            "tmp=$(mktemp)\ntrap 'rm -f \"$tmp\"' EXIT\ncurl -fsSL https://claude.ai/install.sh -o \"$tmp\"\nbash \"$tmp\" </dev/null\nexport PATH=\"$HOME/.local/bin:$PATH\"\nclaude --version"
        }
        ("codex", true) => {
            "if [ \"$(uname -s)\" = Darwin ] && command -v brew >/dev/null 2>&1 && brew list --cask codex >/dev/null 2>&1; then \
               brew upgrade --cask codex; \
             else \
               codex update; \
             fi && codex --version"
        }
        ("codex", false) => {
            "tmp=$(mktemp)\ntrap 'rm -f \"$tmp\"' EXIT\ncurl -fsSL https://chatgpt.com/codex/install.sh -o \"$tmp\"\nsh \"$tmp\" </dev/null\nexport PATH=\"$HOME/.local/bin:/opt/homebrew/bin:$PATH\"\nif [ \"$(uname -s)\" = Darwin ] && [ -d /opt/homebrew/bin ] && [ ! -e /opt/homebrew/bin/codex ] && [ -x \"$HOME/.local/bin/codex\" ]; then ln -s \"$HOME/.local/bin/codex\" /opt/homebrew/bin/codex; fi\ncodex --version"
        }
        ("antigravity", true) => "agy update && agy --version",
        ("antigravity", false) => {
            "tmp=$(mktemp)\ntrap 'rm -f \"$tmp\"' EXIT\ncurl -fsSL https://antigravity.google/cli/install.sh -o \"$tmp\"\nsh \"$tmp\" </dev/null\nexport PATH=\"$HOME/.local/bin:$PATH\"\nagy --version"
        }
        ("cursor", _) => {
            "tmp=$(mktemp)\ntrap 'rm -f \"$tmp\"' EXIT\ncurl -fsSL https://cursor.com/install -o \"$tmp\"\nbash \"$tmp\" </dev/null\nexport PATH=\"$HOME/.local/bin:$PATH\"\ncursor-agent --version"
        }
        ("gemini", update) => {
            let pkg = if update {
                "@google/gemini-cli@latest"
            } else {
                "@google/gemini-cli"
            };
            return Ok(format!(
                "{header}command -v node >/dev/null 2>&1 || {{ echo 'Node.js 20+ requis pour Gemini CLI' >&2; exit 1; }}\n\
                 node -e 'process.exit(Number(process.versions.node.split(\".\")[0]) >= 20 ? 0 : 1)' || {{ echo 'Node.js 20+ requis pour Gemini CLI' >&2; exit 1; }}\n\
                 npm install --global --prefix \"$HOME/.local\" {pkg} </dev/null\n\
                 gemini --version\n"
            ));
        }
        _ => {
            return Err(format!(
                "client CLI inconnu ou non pris en charge : {client_id}"
            ));
        }
    };
    Ok(format!("{header}{body}\n"))
}

pub(crate) async fn route_api_clients_deploy(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<ClientDeployRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    if !valid_machine_id(&req.machine_id) {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_machine_id",
            "Identifiant de machine invalide.",
            "Refusé : machine invalide",
        );
    }
    let Some(cat) = catalog_item(&req.client_id) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "unknown_client",
            "Client CLI inconnu (autorisés : claude, codex, antigravity, cursor, gemini).",
            "Refusé : client CLI non catalogué",
        );
    };
    let update_mode = req.mode == "update";
    let script = match build_client_deploy_script(cat.id, update_mode) {
        Ok(s) => s,
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_client",
                &e,
                &format!("Refusé : {e}"),
            );
        }
    };

    let machine = req.machine_id.clone();
    let label = cat.label;
    let login_hint = cat.login_hint;
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let local_m = local_machine_name();
        let is_local = machine == local_m;
        let out = if is_local {
            Command::new("bash")
                .args(["-lc", &script])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())
                .and_then(|c| wait_child_bounded(c, INSTALL_TIMEOUT_SECS))?
        } else {
            Command::new("ssh")
                .args([
                    "-o",
                    "BatchMode=yes",
                    "-o",
                    &format!("ConnectTimeout={SSH_CONNECT_TIMEOUT}"),
                    &machine,
                    "bash -s",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())
                .and_then(|mut c| {
                    if let Some(mut stdin) = c.stdin.take() {
                        let _ = stdin.write_all(script.as_bytes());
                    }
                    wait_child_bounded(c, INSTALL_TIMEOUT_SECS)
                })?
        };
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let tail_log = format!("{stdout}\n{stderr}")
            .chars()
            .rev()
            .take(4000)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        if out.status.success() {
            let version_line = stdout
                .lines()
                .last()
                .unwrap_or("installé")
                .trim()
                .to_string();
            let verb = if update_mode {
                "mis à jour"
            } else {
                "installé"
            };
            Ok(json!({
                "status": "ok",
                "machine_id": machine,
                "client_id": cat.id,
                "version": version_line,
                "login_hint": login_hint,
                "log": tail_log,
                "ui_state": format!("{label} {verb} sur {machine} ({version_line})")
            }))
        } else {
            Err(format!(
                "échec {} de {label} sur {machine} : {}",
                if update_mode {
                    "mise à jour"
                } else {
                    "installation"
                },
                if !stderr.is_empty() {
                    stderr.lines().last().unwrap_or(&stderr).to_string()
                } else {
                    format!("code {:?}", out.status.code())
                }
            ))
        }
    })
    .await;

    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "client_deploy_failed",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "client_deploy_join",
            &e.to_string(),
            "Refusé : déploiement interrompu",
        ),
    }
}

#[cfg(test)]
#[path = "console_admin_tests.rs"]
mod tests;
