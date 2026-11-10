//! Vérification locale de support. Ni accès fournisseur ni mise à jour implicite.
use crate::cli_table::table_println as println;
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
    // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    let nonblocking =
        // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
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
    if result.0 == "probe_timeout" && !eof && child.try_wait().ok().flatten().is_some() {
        // Le parent a fini mais un descendant garde le flux ouvert : ne pas
        // signaler un PID réutilisable ; conserver l’identité initiale comme diagnostic.
        return (format!("probe_stop_unconfirmed:{}", child.id()), None);
    }
    if child.try_wait().ok().flatten().is_none() {
        // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
        let grace = Instant::now() + Duration::from_secs(1);
        while child.try_wait().ok().flatten().is_none() && Instant::now() < grace {
            std::thread::sleep(Duration::from_millis(10));
        }
        if child.try_wait().ok().flatten().is_none() {
            return (format!("probe_stop_unconfirmed:{}", child.id()), None);
        }
    }
    result
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    minimum: String,
    provenance: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    version: u8,
    requirements: BTreeMap<String, Requirement>,
}
fn policy() -> Policy {
    Policy {
        version: 1,
        requirements: BTreeMap::from([
            (
                "claude".into(),
                Requirement {
                    minimum: minimum("claude").into(),
                    provenance: "provider_minimum_208".into(),
                },
            ),
            (
                "codex".into(),
                Requirement {
                    minimum: minimum("codex").into(),
                    provenance: "child_requests_schema_155".into(),
                },
            ),
        ]),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Client {
    engine: String,
    program: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    probe_pid: Option<u32>,
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
    policy: Policy,
    host: String,
    machine_id: String,
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
    scope: &str,
    mut check: impl FnMut(&str, &str) -> (String, Option<String>),
) -> Result<Value, String> {
    let mut clients = Vec::new();
    for ((engine, program), connections) in entries(registry, scope)? {
        let (mut state, installed) = check(&program, &engine);
        let probe_pid = state
            .strip_prefix("probe_stop_unconfirmed:")
            .and_then(|p| p.parse::<u32>().ok())
            .filter(|p| *p > 0);
        if probe_pid.is_some() {
            state = "probe_stop_unconfirmed".into();
        }
        let required = minimum(&engine).to_string();
        let message = match state.as_str() {
            "compatible" => format!(
                "{engine} {} respecte le minimum pris en charge {required} de cette version Bridget.",
                installed.as_deref().unwrap_or("?")
            ),
            "probe_stop_unconfirmed" => format!(
                "L’arrêt de la sonde --version ou de ses descendants n’est pas confirmé (identifiant initial PID/groupe {}). Aucun lancement métier n’en est déduit.",
                probe_pid.unwrap_or(0)
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
            "probe_stop_unconfirmed"=>format!("Faire inspecter sur {host} l’identifiant initial PID/groupe {} de la sonde du programme {program}, sans supposer que ce PID désigne encore le même processus. Ne pas répéter automatiquement le contrôle, ne pas redémarrer de service et ne pas toucher aux missions métier. Après résolution et décision explicite, demander une nouvelle vérification par bridget updates check.",probe_pid.unwrap_or(0)),
            "outdated"=>format!("Souhaitez-vous autoriser la mise à jour officielle de {engine} sur {host} ? Demander cet accord avant toute modification du programme {program}. Conserver comptes, modèles et gestionnaire d’installation. Après accord, mettre à jour par le mécanisme officiel existant, exécuter bridget clients check, puis qualifier un lancement réel autorisé. Sans accord, conserver Bridget actif et ses missions ; ne pas basculer le candidat."),
            _=>format!("Faire vérifier le programme {program} et sa version sur {host}, puis exécuter bridget clients check. Ne pas installer, changer de compte ou modifier les droits pour masquer une cause inconnue. La bascule reste suspendue ; les missions actuelles continuent."),
        };
        clients.push(Client {
            engine,
            program,
            probe_pid,
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
    serde_json::to_value(Report{version:1,policy:policy(),host:host.into(),machine_id:scope.into(),bridget_build:crate::build_info::BUILD_ID.into(),
        scope:"installed_local_clients".into(),verdict:verdict.into(),registry_file:None,clients,
        authentication_checked:false,model_access_checked:false,
        message:"Contrôle des versions des seuls clients locaux installés ; aucun accès fournisseur vérifié.".into(),
        recovery:"Si une mise à jour est nécessaire, demander l’accord utilisateur pour le programme et la machine indiqués. Conserver les comptes, modèles et gestionnaires d’installation. Refaire ce contrôle après mise à jour, puis qualifier un usage réel autorisé.".into()}).map_err(|_|"client_check_unavailable".into())
}
fn local_registry() -> Result<(AgentRegistry, std::path::PathBuf), String> {
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
    Ok((registry, source))
}
pub(crate) fn machine_scope(registry: &AgentRegistry, source: &Path, host: &str) -> Result<String, String> {
    let Some(policy) = registry.routing_policy() else {
        return Ok(host.into());
    };
    // Le registre dédié du worker porte une seule identité provisionnée.
    // Aucune équivalence n’est déduite des noms DNS ni de leur orthographe.
    let worker = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or("client_machine_scope_unavailable")?
        .join(".config/bridget/agents.json");
    if source == worker && registry.machines().len() == 1 {
        let machine = registry.machines().keys().next().unwrap();
        if policy.connections.iter().any(|c| &c.machine_id == machine) {
            return Ok(machine.clone());
        }
    }
    if policy
        .connections
        .iter()
        .any(|c| c.machine_id == host || c.machine_id == "local")
    {
        return Ok(host.into());
    }
    Err("client_machine_scope_unavailable".into())
}
pub(crate) fn check() -> Result<Value, String> {
    let (registry, source) = local_registry()?;
    let host = crate::build_info::local_host();
    if !bridget_core::host_is_attested(&host) {
        return Err("client_host_unavailable".into());
    }
    let scope = machine_scope(&registry, &source, &host)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut report = report_with(&registry, &host, &scope, |p, e| {
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
        && (report.host == host || report.machine_id == host)
        && bridget_core::host_is_attested(&report.host)
        && atom(&report.machine_id, 256)
        && atom(&report.bridget_build, 256)
        && report
            .registry_file
            .as_deref()
            .is_none_or(|p| atom(p, 4096) && Path::new(p).is_absolute())
        && report.policy.version == 1
        && report.policy.requirements.len() == 2
        && ["claude", "codex"].iter().all(|engine| {
            report.policy.requirements.get(*engine).is_some_and(|r| {
                triplet(&r.minimum).is_some_and(|v| v >= triplet(minimum(engine)).unwrap())
                    && atom(&r.provenance, 128)
            })
        })
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
                && report
                    .policy
                    .requirements
                    .get(&c.engine)
                    .is_some_and(|r| r.minimum == c.minimum)
                && triplet(&c.minimum).is_some()
                && ((c.state == "probe_stop_unconfirmed" && c.probe_pid.is_some_and(|p| p > 0))
                    || (c.state != "probe_stop_unconfirmed" && c.probe_pid.is_none()))
                && matches!(
                    c.state.as_str(),
                    "compatible"
                        | "outdated"
                        | "not_installed"
                        | "launcher_unverified"
                        | "probe_unavailable"
                        | "probe_failed"
                        | "probe_stop_unconfirmed"
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
fn inventory_matches(value: &Value, registry: &AgentRegistry, host: &str, scope: &str) -> bool {
    if !valid_report(value, host) {
        return false;
    }
    let Ok(report) = serde_json::from_value::<Report>(value.clone()) else {
        return false;
    };
    if report.machine_id != scope {
        return false;
    }
    let Ok(expected) = entries(registry, scope) else {
        return false;
    };
    let mut actual = BTreeMap::new();
    for c in report.clients {
        if actual
            .insert((c.engine, c.program), c.connections)
            .is_some()
        {
            return false;
        }
    }
    actual == expected
}
fn validate_local_report(value: &Value) -> bool {
    let Ok((registry, source)) = local_registry() else {
        return false;
    };
    let host = crate::build_info::local_host();
    let Ok(scope) = machine_scope(&registry, &source, &host) else {
        return false;
    };
    value["registry_file"] == json!(source) && inventory_matches(value, &registry, &host, &scope)
}
pub(crate) fn run(args: &[String]) {
    if args == ["policy"] {
        println!("{}", serde_json::to_value(policy()).unwrap());
        return;
    }
    if args == ["validate-report"] {
        let mut raw = Vec::new();
        let valid = std::io::stdin().take(32769).read_to_end(&mut raw).is_ok()
            && raw.len() <= 32768
            && serde_json::from_slice::<Value>(&raw)
                .ok()
                .is_some_and(|v| validate_local_report(&v));
        println!("{}", json!({"valid":valid}));
        crate::cli_table::exit(if valid { 0 } else { 1 });
    }
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
            crate::cli_table::exit(2);
        }
    };
    let result=check().unwrap_or_else(|error| {
        let message=match error.as_str() {
            "client_check_program_limit"=>"client_check_program_limit : plus de16 programmes distincts ; le contrôle automatique est refusé. Faire qualifier ce périmètre local avant la bascule.",
            "client_machine_scope_unavailable"=>"client_machine_scope_unavailable : le registre ne permet pas de rattacher les connexions à cette machine. Vérifier le registre natif du worker ; un inventaire vide ne prouve pas la compatibilité.",
            "client_host_unavailable"=>"client_host_unavailable : la machine locale n’est pas attestée. Vérifier son identité native avant d’en déduire les connexions à contrôler.",
            "client_definition_unverified"=>"client_definition_unverified : moteur et pilote déclarés ne concordent pas. Faire vérifier le registre local avant la bascule.",
            _=>"client_check_unavailable : le contrôle local n’a pas abouti ; sa cause précise reste à vérifier. Aucune version client n’est attestée.",
        };
        json!({"version":1,"policy":policy(),"verdict":"client_check_unavailable",
            "host":crate::build_info::local_host(),"machine_id":crate::build_info::local_host(),"bridget_build":crate::build_info::BUILD_ID,
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
        crate::cli_table::exit(1);
    }
}

#[cfg(test)]
#[path = "client_compatibility_tests.rs"]
mod tests;
