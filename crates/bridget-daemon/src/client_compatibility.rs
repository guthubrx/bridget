//! Vérification locale de support. Ni accès fournisseur ni mise à jour implicite.
use crate::registry::AgentRegistry;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    os::{fd::AsRawFd, unix::process::CommandExt},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const MAX_PROGRAMS: usize = 16;

fn minimum(engine: &str) -> &'static str {
    // Politique de support209 : preuve208 pour Claude ; schéma de référence ADR155 pour Codex.
    match engine {
        "claude" => "2.1.280",
        _ => "0.162.1",
    }
}
fn triplet(s: &str) -> Option<[u32; 3]> {
    let parts = s.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    let mut out = [0; 3];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty()
            || p.len() > 6
            || !p.bytes().all(|b| b.is_ascii_digit())
            || (p.len() > 1 && p.starts_with('0'))
        {
            return None;
        }
        out[i] = p.parse().ok()?;
    }
    Some(out)
}
fn parse_version(engine: &str, raw: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(raw).ok()?.trim();
    let value = match engine {
        "claude" => text.strip_suffix(" (Claude Code)")?,
        "codex" => text.strip_prefix("codex-cli ")?,
        _ => return None,
    };
    triplet(value)?;
    Some(value.into())
}

fn known_absent(program: &str) -> bool {
    let candidates = if program.contains('/') {
        vec![std::path::PathBuf::from(program)]
    } else if let Some(paths) = std::env::var_os("PATH") {
        std::env::split_paths(&paths)
            .take(129)
            .map(|p| p.join(program))
            .collect::<Vec<_>>()
    } else {
        return false;
    };
    !candidates.is_empty()
        && candidates.len() <= 128
        && candidates.iter().all(|p| {
            std::fs::symlink_metadata(p).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        })
}

/// O(P), P<=16, budget partagé10s, stdout<=4Kio. TERM ne vise que le groupe créé ici.
fn probe(program: &str, engine: &str, deadline: Instant) -> (String, Option<String>) {
    let base = Path::new(program)
        .file_name()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    if !matches!(base, "claude" | "codex" | "gclaude" | "codex-pro") {
        return ("launcher_unverified".into(), None);
    }
    let mut child = match Command::new(program)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env("DISABLE_AUTOUPDATER", "1")
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .process_group(0)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                if e.kind() == std::io::ErrorKind::NotFound && known_absent(program) {
                    "not_installed"
                } else {
                    "probe_unavailable"
                }
                .into(),
                None,
            );
        }
    };
    let mut pipe = child.stdout.take().unwrap();
    let fd = pipe.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    let nonblocking =
        flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0;
    let stop = deadline.min(Instant::now() + Duration::from_secs(2));
    let mut raw = Vec::new();
    let mut buffer = [0; 512];
    let mut eof = false;
    let result = loop {
        if !nonblocking {
            break ("probe_unavailable".into(), None);
        }
        match pipe.read(&mut buffer) {
            Ok(0) => eof = true,
            Ok(n) => {
                raw.extend_from_slice(&buffer[..n]);
                if raw.len() > 4096 {
                    break ("output_limit".into(), None);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break ("probe_unavailable".into(), None),
        }
        match child.try_wait() {
            Ok(Some(status)) if eof => {
                if !status.success() {
                    break ("probe_failed".into(), None);
                }
                break match parse_version(engine, &raw) {
                    Some(v) => (
                        if triplet(&v) < triplet(minimum(engine)) {
                            "outdated"
                        } else {
                            "compatible"
                        }
                        .into(),
                        Some(v),
                    ),
                    None => ("version_unverified".into(), None),
                };
            }
            Err(_) => break ("probe_unavailable".into(), None),
            _ => {}
        }
        if Instant::now() >= stop {
            break ("probe_timeout".into(), None);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if child.try_wait().ok().flatten().is_none() {
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
        let grace = Instant::now() + Duration::from_secs(1);
        while child.try_wait().ok().flatten().is_none() && Instant::now() < grace {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    result
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Client {
    engine: String,
    program: String,
    connections: Vec<String>,
    installed: Option<String>,
    minimum: String,
    state: String,
    message: String,
    recovery: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    version: u8,
    host: String,
    bridget_build: String,
    scope: String,
    verdict: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    registry_file: Option<String>,
    clients: Vec<Client>,
    authentication_checked: bool,
    model_access_checked: bool,
    message: String,
    recovery: String,
}
fn entries(
    registry: &AgentRegistry,
    host: &str,
) -> Result<BTreeMap<(String, String), Vec<String>>, String> {
    let mut out = BTreeMap::new();
    if let Some(policy) = registry.routing_policy() {
        for c in policy
            .connections
            .iter()
            .filter(|c| c.enabled && (c.machine_id == host || c.machine_id == "local"))
        {
            if !matches!(c.engine.as_str(), "claude" | "codex") {
                continue;
            }
            let d = registry.get(&c.agent_type)?;
            let protocol = if c.engine == "claude" {
                "claude_stream_json"
            } else {
                "codex_app_server"
            };
            if d.protocol != protocol {
                return Err("client_definition_unverified".into());
            }
            out.entry((c.engine.clone(), d.command.clone()))
                .or_insert_with(Vec::new)
                .push(c.connection_id.clone());
        }
    } else {
        for name in registry.known_types() {
            let d = registry.get(&name)?;
            let engine = match d.protocol.as_str() {
                "claude_stream_json" => "claude",
                "codex_app_server" => "codex",
                _ => continue,
            };
            out.entry((engine.into(), d.command.clone()))
                .or_insert_with(Vec::new)
                .push(name);
        }
    }
    if out.len() > MAX_PROGRAMS {
        return Err("client_check_program_limit".into());
    }
    Ok(out)
}
fn report_with(
    registry: &AgentRegistry,
    host: &str,
    mut check: impl FnMut(&str, &str) -> (String, Option<String>),
) -> Result<Value, String> {
    let mut clients = Vec::new();
    for ((engine, program), connections) in entries(registry, host)? {
        let (state, installed) = check(&program, &engine);
        let required = minimum(&engine).to_string();
        let message = match state.as_str() {
            "compatible" => format!(
                "{engine} {} respecte le minimum pris en charge {required} de cette version Bridget.",
                installed.as_deref().unwrap_or("?")
            ),
            "not_installed" => format!(
                "{engine} est absent sur cette machine ; il reste indisponible. Son absence n’empêche pas d’installer Bridget seul."
            ),
            "outdated" => format!(
                "{engine} {} est trop ancien ; minimum pris en charge : {required}. La mise à jour Bridget attend une décision utilisateur.",
                installed.as_deref().unwrap_or("?")
            ),
            _ => format!(
                "Version {engine} non vérifiée ({state}). Aucune incompatibilité de compte ou de modèle n’est déduite."
            ),
        };
        let recovery=match state.as_str() {
            "compatible"|"not_installed"=>"Ce contrôle ne garantit ni authentification ni accès aux modèles. Aucune installation ou mission n’a été lancée.".into(),
            "outdated"=>format!("Souhaitez-vous autoriser la mise à jour officielle de {engine} sur {host} ? Demander cet accord avant toute modification du programme {program}. Conserver comptes, modèles et gestionnaire d’installation. Après accord, mettre à jour par le mécanisme officiel existant, exécuter bridget clients check, puis qualifier un lancement réel autorisé. Sans accord, conserver Bridget actif et ses missions ; ne pas basculer le candidat."),
            _=>format!("Faire vérifier le programme {program} et sa version sur {host}, puis exécuter bridget clients check. Ne pas installer, changer de compte ou modifier les droits pour masquer une cause inconnue. La bascule reste suspendue ; les missions actuelles continuent."),
        };
        clients.push(Client {
            engine,
            program,
            connections,
            installed,
            minimum: required,
            state,
            message,
            recovery,
        });
    }
    let verdict = if clients.iter().any(|c| c.state == "outdated") {
        "client_update_required"
    } else if clients
        .iter()
        .any(|c| !matches!(c.state.as_str(), "compatible" | "not_installed"))
    {
        "client_check_unavailable"
    } else {
        "compatible"
    };
    serde_json::to_value(Report{version:1,host:host.into(),bridget_build:crate::build_info::BUILD_ID.into(),
        scope:"installed_local_clients".into(),verdict:verdict.into(),registry_file:None,clients,
        authentication_checked:false,model_access_checked:false,
        message:"Contrôle des versions des seuls clients locaux installés ; aucun accès fournisseur vérifié.".into(),
        recovery:"Si une mise à jour est nécessaire, demander l’accord utilisateur pour le programme et la machine indiqués. Conserver les comptes, modèles et gestionnaires d’installation. Refaire ce contrôle après mise à jour, puis qualifier un usage réel autorisé.".into()}).map_err(|_|"client_check_unavailable".into())
}
pub(crate) fn check() -> Result<Value, String> {
    let namespace = crate::environment::Namespace::from_environment()?;
    let mut source = namespace.root.join("agents.json");
    if let Some(explicit) = std::env::var_os("BRIDGET_CLIENTS_REGISTRY") {
        source = explicit.into();
        if !source.is_absolute() {
            return Err("client_registry_unavailable".into());
        }
    } else if match std::fs::symlink_metadata(&source) {
        Ok(_) => false,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => return Err("client_registry_unavailable".into()),
    } {
        let worker = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .ok_or("client_registry_unavailable")?
            .join(".config/bridget/agents.json");
        match std::fs::symlink_metadata(&worker) {
            Ok(_) => source = worker,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("client_registry_unavailable".into()),
        }
    }
    let registry = AgentRegistry::load_from_path_quiet(source.clone())?;
    let host = crate::build_info::local_host();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut report = report_with(&registry, &host, |p, e| {
        if Instant::now() >= deadline {
            ("probe_budget_elapsed".into(), None)
        } else {
            probe(p, e, deadline)
        }
    })?;
    report["registry_file"] = json!(source);
    Ok(report)
}
/// Un rapport d’update est une donnée d’administration bornée ; jamais une commande.
pub(crate) fn valid_report(value: &Value, host: &str) -> bool {
    let Ok(report) = serde_json::from_value::<Report>(value.clone()) else {
        return false;
    };
    let atom =
        |s: &str, max: usize| !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control);
    let expected_verdict = if report.clients.iter().any(|c| c.state == "outdated") {
        "client_update_required"
    } else if report
        .clients
        .iter()
        .any(|c| !matches!(c.state.as_str(), "compatible" | "not_installed"))
        || (report.clients.is_empty() && report.verdict == "client_check_unavailable")
    {
        "client_check_unavailable"
    } else {
        "compatible"
    };
    report.version == 1
        && report.host == host
        && atom(&report.bridget_build, 256)
        && report
            .registry_file
            .as_deref()
            .is_none_or(|p| atom(p, 4096) && Path::new(p).is_absolute())
        && report.scope == "installed_local_clients"
        && !report.authentication_checked
        && !report.model_access_checked
        && matches!(
            report.verdict.as_str(),
            "compatible" | "client_update_required" | "client_check_unavailable"
        )
        && atom(&report.message, 2048)
        && atom(&report.recovery, 4096)
        && report.clients.len() <= MAX_PROGRAMS
        && value.to_string().len() <= 32768
        && report.verdict == expected_verdict
        && report.clients.iter().all(|c| {
            matches!(c.engine.as_str(), "claude" | "codex")
                && atom(&c.program, 4096)
                && c.connections.len() <= 64
                && c.connections.iter().all(|s| atom(s, 256))
                && c.installed.as_deref().is_none_or(|v| triplet(v).is_some())
                && triplet(&c.minimum).is_some()
                && matches!(
                    c.state.as_str(),
                    "compatible"
                        | "outdated"
                        | "not_installed"
                        | "launcher_unverified"
                        | "probe_unavailable"
                        | "probe_failed"
                        | "probe_timeout"
                        | "output_limit"
                        | "version_unverified"
                        | "probe_budget_elapsed"
                )
                && (match c.state.as_str() {
                    "compatible" => c
                        .installed
                        .as_deref()
                        .is_some_and(|v| triplet(v) >= triplet(&c.minimum)),
                    "outdated" => c
                        .installed
                        .as_deref()
                        .is_some_and(|v| triplet(v) < triplet(&c.minimum)),
                    _ => c.installed.is_none(),
                })
                && atom(&c.message, 2048)
                && atom(&c.recovery, 4096)
        })
}
pub(crate) fn run(args: &[String]) {
    let as_json = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["check"] => false,
        ["check", "--json"] => true,
        _ => {
            eprintln!("Usage : bridget clients check [--json]");
            std::process::exit(2);
        }
    };
    let result=check().unwrap_or_else(|error| {
        let message=match error.as_str() {
            "client_check_program_limit"=>"client_check_program_limit : plus de16 programmes distincts ; le contrôle automatique est refusé. Faire qualifier ce périmètre local avant la bascule.",
            "client_definition_unverified"=>"client_definition_unverified : moteur et pilote déclarés ne concordent pas. Faire vérifier le registre local avant la bascule.",
            _=>"client_check_unavailable : le contrôle local n’a pas abouti ; sa cause précise reste à vérifier. Aucune version client n’est attestée.",
        };
        json!({"version":1,"verdict":"client_check_unavailable",
            "host":crate::build_info::local_host(),"bridget_build":crate::build_info::BUILD_ID,
            "scope":"installed_local_clients","clients":[],"authentication_checked":false,"model_access_checked":false,
            "message":message,
            "recovery":"Vérifier le contexte et le registre natifs de cette machine, puis refaire bridget clients check avant la bascule. Ne pas changer de compte, de droits ou de modèle pour masquer ce refus."})
    });
    if as_json {
        println!("{result}");
    } else {
        println!(
            "Contrôle local des clients : {}",
            result["verdict"]
                .as_str()
                .unwrap_or("client_check_unavailable")
        );
        if let Some(message) = result["message"].as_str() {
            println!("{message}");
        }
        if let Some(recovery) = result["recovery"].as_str() {
            println!("{recovery}");
        }
        if let Some(rows) = result["clients"].as_array() {
            for row in rows {
                println!(
                    "{} — {} : {}\n{}",
                    result["host"].as_str().unwrap_or("?"),
                    row["program"].as_str().unwrap_or("?"),
                    row["message"].as_str().unwrap_or("?"),
                    row["recovery"].as_str().unwrap_or("?")
                );
            }
        }
    }
    let _ = std::io::stdout().flush();
    if result["verdict"] != "compatible" {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn registry() -> AgentRegistry {
        let mut models = serde_json::Map::new();
        for name in ["one", "two", "foreign", "off"] {
            models.insert(name.into(),json!([{"model":"fixture","supported_efforts":[],"default_effort":null,"provenance":"declared"}]));
        }
        AgentRegistry::from_json(&json!({
            "agents":{"fixture":{"command":"/fixture path/claude","protocol":"claude_stream_json"}},
            "machines":{"foreign":{"enabled":true,"worker_id":"foreign-worker","provisioning_revision":1,
                "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["fixture"],"mappings":{}}},
            "routing_policy":{"version":2,"revision":1,"enabled":true,
                "connections":[
                    {"connection_id":"one","machine_id":"local","engine":"claude","agent_type":"fixture","enabled":true},
                    {"connection_id":"two","machine_id":"local","engine":"claude","agent_type":"fixture","enabled":true},
                    {"connection_id":"foreign","machine_id":"foreign","engine":"claude","agent_type":"fixture","enabled":true},
                    {"connection_id":"off","machine_id":"local","engine":"claude","agent_type":"fixture","enabled":false}
                ],"model_catalog":models,"machine_policies":{}
            }}).to_string(),"/tmp/clients209.json").unwrap()
    }
    #[test]
    fn spec209_local_connections_deduplicate_and_require_consent_before_upgrade() {
        let r = registry();
        let mut calls = 0;
        let report = report_with(&r, "here", |program, engine| {
            calls += 1;
            assert_eq!(program, "/fixture path/claude");
            assert_eq!(engine, "claude");
            ("outdated".into(), Some("2.1.251".into()))
        })
        .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(report["clients"].as_array().unwrap().len(), 1);
        assert_eq!(report["clients"][0]["connections"], json!(["one", "two"]));
        assert_eq!(report["verdict"], "client_update_required");
        assert!(
            report["clients"][0]["recovery"]
                .as_str()
                .unwrap()
                .contains("Souhaitez-vous autoriser")
        );
        assert!(!report["authentication_checked"].as_bool().unwrap());
        assert!(valid_report(&report, "here"));
        assert!(!valid_report(&report, "foreign"));
        let mut bad = report.clone();
        bad["verdict"] = json!("compatible");
        assert!(!valid_report(&bad, "here"));
        bad = report.clone();
        bad["clients"][0]["state"] = json!("compatible");
        assert!(!valid_report(&bad, "here"));
        bad = report.clone();
        bad["secret"] = json!("private");
        assert!(!valid_report(&bad, "here"));
    }
    #[test]
    fn spec209_numeric_versions_and_unknown_are_never_guessed() {
        for (engine, text, want) in [
            ("claude", "2.1.312 (Claude Code)", Some("2.1.312")),
            ("codex", "codex-cli 0.163.0", Some("0.163.0")),
            ("claude", "private api_error 400 version 2.1.280", None),
            ("codex", "codex-cli 0.163.0-beta", None),
            ("claude", "02.1.312 (Claude Code)", None),
        ] {
            assert_eq!(parse_version(engine, text.as_bytes()).as_deref(), want);
        }
        assert!(triplet("2.1.280") > triplet("2.1.99"));
        assert!(parse_version("claude", &[255]).is_none());
        for state in ["probe_failed", "version_unverified", "probe_timeout"] {
            let report = report_with(&registry(), "here", |_, _| (state.into(), None)).unwrap();
            assert_eq!(report["verdict"], "client_check_unavailable");
            assert!(valid_report(&report, "here"));
            assert!(!report.to_string().contains("private"));
        }
        let report =
            report_with(&registry(), "here", |_, _| ("not_installed".into(), None)).unwrap();
        assert_eq!(report["verdict"], "compatible");
        assert_eq!(report["clients"][0]["state"], "not_installed");
    }
    #[test]
    fn spec209_update_report_keeps_legacy_wire_and_checks_diagnostics() {
        use bridget_transport::protocol::{UpdateRequest, UpdateStage};
        let report = report_with(&registry(), "here", |_, _| {
            ("outdated".into(), Some("2.1.251".into()))
        })
        .unwrap();
        let q = UpdateRequest::Report {
            host: "here".into(),
            installed: "0.6.3".into(),
            available: Some("0.6.4".into()),
            stage: UpdateStage::ClientUpdateRequired,
            client_compatibility: Some(report.clone()),
        };
        let encoded = serde_json::to_value(q).unwrap();
        assert_eq!(encoded["stage"], "client_update_required");
        let decoded: UpdateRequest = serde_json::from_value(encoded).unwrap();
        let UpdateRequest::Report {
            client_compatibility: Some(checked),
            ..
        } = decoded
        else {
            panic!()
        };
        assert!(valid_report(&checked, "here"));
        let legacy = json!({"action":"report","host":"here","installed":"0.6.3","available":null,"stage":"ready"});
        let old: UpdateRequest = serde_json::from_value(legacy.clone()).unwrap();
        assert_eq!(serde_json::to_value(old).unwrap(), legacy);
        let mut bad = report;
        bad["clients"][0]["installed"] = json!("2.1.312");
        assert!(!valid_report(&bad, "here"));
    }

    #[test]
    fn spec209_real_version_only_probe_is_bounded_and_excludes_raw_errors() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("clients209-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("claude");
        for (body, state, version) in [
            (
                "[ \"$#\" = 1 ] && [ \"$1\" = --version ] && [ \"$DISABLE_AUTOUPDATER\" = 1 ] || exit 9\nprintf '2.1.312 (Claude Code)\\n'",
                "compatible",
                Some("2.1.312"),
            ),
            (
                "printf '2.1.251 (Claude Code)\\n'",
                "outdated",
                Some("2.1.251"),
            ),
            (
                "printf 'PRIVATE-provider-message\\n'",
                "version_unverified",
                None,
            ),
            (
                "printf 'PRIVATE-provider-message' >&2\nexit 1",
                "probe_failed",
                None,
            ),
            (
                "while :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done",
                "output_limit",
                None,
            ),
            ("sleep 10", "probe_timeout", None),
        ] {
            std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
            let start = Instant::now();
            let result = probe(
                path.to_str().unwrap(),
                "claude",
                start + Duration::from_millis(300),
            );
            assert_eq!(result, (state.into(), version.map(str::to_string)));
            assert!(start.elapsed() < Duration::from_secs(2));
        }
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            probe(
                path.to_str().unwrap(),
                "claude",
                Instant::now() + Duration::from_secs(1)
            )
            .0,
            "not_installed"
        );
        assert_eq!(
            probe("/bin/sh", "claude", Instant::now() + Duration::from_secs(1)).0,
            "launcher_unverified"
        );
        std::fs::remove_dir(&dir).unwrap();
    }
}
