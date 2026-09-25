//! Carte de reprise du référent — projection courte pour un agent sans contexte.
//!
//! La carte est générée, jamais rédigée à la main. Les chemins vivants viennent
//! de la même résolution que `bridget status` ; une source absente se déclare
//! dans son champ sans faire échouer le reste.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bridget_transport::protocol::{AgentInfo, LedgerMessage, LedgerScope, RequestInfo};

use crate::daemon::{self, DaemonConfig, DaemonStatus};
use crate::ledger;
use crate::store::Store;

/// Âge maximal d'un pin avant qu'il ne soit affiché comme périmé.
pub const PIN_STALE_AFTER: Duration = Duration::from_secs(3600);

/// Chemin relatif par défaut du fichier généré dans le dépôt.
pub const DEFAULT_GENERATED_REL: &str = "docs/carte-reprise.generated.yaml";

/// Chemin relatif par défaut du fichier épinglé dans le dépôt.
pub const DEFAULT_PIN_REL: &str = "docs/carte-reprise.pin.yaml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PinFile {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub couloirs: Vec<PinCouloir>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prochain: Option<PinProchain>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pieges: Option<PinPieges>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinCouloir {
    pub id: String,
    pub owner: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fichiers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinProchain {
    pub geste: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proprietaire: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PinPieges {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actifs_session: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedPin {
    pub pin: PinFile,
    pub path: PathBuf,
    pub modified_at: SystemTime,
}

#[derive(Clone)]
pub struct RepriseSnapshot {
    pub now: SystemTime,
    pub repo: PathBuf,
    pub socket_path: PathBuf,
    /// Base **de cette machine**. Elle ne décrit le daemon que si celui-ci est
    /// local — la carte doit donc toujours l'afficher avec sa machine.
    pub db_path: PathBuf,
    pub binary_cli: Option<PathBuf>,
    pub status: Result<DaemonStatus, String>,
    pub open_requests: Result<Vec<RequestInfo>, String>,
    pub recent_messages: Result<Vec<LedgerMessage>, String>,
    pub git: Result<GitSnapshot, String>,
    pub pin: Option<Result<LoadedPin, String>>,
    /// Trace durable des équipiers absents au dernier redémarrage.
    pub recovery_losses: Result<Option<crate::recovery_trace::RecoveryLossReport>, String>,
}

#[derive(Debug, Clone)]
pub struct GitSnapshot {
    pub head: String,
    pub head_message: String,
    pub worktrees: Vec<GitWorktree>,
}

#[derive(Debug, Clone)]
pub struct GitWorktree {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
    pub dirty: Option<bool>,
}

/// Collecte les sources vivantes. N'échoue jamais en bloc : chaque source
/// porte son propre `Result`.
pub fn collect_snapshot(
    config: &DaemonConfig,
    repo: &Path,
    pin_path: Option<&Path>,
    now: SystemTime,
) -> RepriseSnapshot {
    let socket_path = absolutize(&config.socket_path);
    let db_path = absolutize(&config.db_path);
    let binary_cli = std::env::current_exe().ok().map(|path| absolutize(&path));

    let status = daemon::get_status(config);
    // La flotte décrite doit être celle du daemon interrogé. Tant que sa base
    // n'est pas la nôtre, la trace de reprise locale décrit une AUTRE flotte :
    // on ne la lit pas, et la carte dit pourquoi.
    let recovery_losses = match &status {
        Ok(status) => match recovery_trace_scope(
            status.daemon_host.as_deref(),
            status.daemon_db_path.as_deref(),
            &crate::build_info::local_host(),
            &db_path,
        ) {
            Ok(()) => collect_recovery_losses(&db_path),
            Err(motif) => Err(motif),
        },
        Err(error) => Err(format!("statut daemon indisponible: {error}")),
    };

    let (open_requests, recent_messages) = collect_ledger(config);

    let git = collect_git(repo);
    let pin = pin_path.map(load_pin);
    RepriseSnapshot {
        now,
        repo: absolutize(repo),
        socket_path,
        db_path,
        binary_cli,
        status,
        open_requests,
        recent_messages,
        git,
        pin,
        recovery_losses,
    }
}

/// Marqueur du refus pour INDÉTERMINATION — présent dans ce seul motif.
///
/// Les deux refus partagent la mention « machine non attestée », parce que le
/// motif générique affiche lui aussi l'hôte du daemon. Une assertion sur cette
/// mention ne DISCRIMINE donc rien : il faut une phrase qui n'appartienne qu'à
/// une seule issue, et l'oracle doit exiger l'une ET refuser l'autre.
const MOTIF_INDETERMINATION: &str = "Une machine indéterminée n'est pas une machine";

/// Marqueur du refus pour BASES DIFFÉRENTES — présent dans ce seul motif.
const MOTIF_BASES_DIFFERENTES: &str = "n'est pas celle du daemon";

/// La trace de reprise locale ne décrit la flotte du daemon que si la base du
/// daemon est bien celle d'ici — MACHINE **et** chemin.
///
/// Séparée de la lecture pour être éprouvable dans ses DEUX issues : un test qui
/// ne prouverait que le refus laisserait vivre un mutant rendant la condition
/// toujours fausse, et la trace ne serait plus jamais lue sans que rien ne
/// rougisse.
fn recovery_trace_scope(
    daemon_host: Option<&str>,
    daemon_db_path: Option<&str>,
    local_host: &str,
    local_db_path: &Path,
) -> Result<(), String> {
    if daemon::daemon_store_is_local(daemon_host, daemon_db_path, local_host, local_db_path) {
        return Ok(());
    }
    // Deux refus DIFFERENTS, et le lecteur doit savoir lequel il tient : une
    // machine qu'on n'a pas su nommer ne se corrige pas comme deux machines
    // distinctes. Confondre les deux, c'est envoyer chercher au mauvais endroit.
    let atteste = |host: Option<&str>| host.is_some_and(bridget_core::host_is_attested);
    if !atteste(daemon_host) || !bridget_core::host_is_attested(local_host) {
        return Err(format!(
            "trace de reprise non lisible d'ici : machine non attestée — locale « {} », daemon « {} ». \
             {MOTIF_INDETERMINATION}, et deux indéterminées ne sont pas la même.",
            crate::build_info::describe_host(Some(local_host)),
            crate::build_info::describe_host(daemon_host)
        ));
    }
    Err(format!(
        "trace de reprise non lisible d'ici : la base locale ({} sur {}) {MOTIF_BASES_DIFFERENTES} ({} sur {})",
        local_db_path.display(),
        crate::build_info::describe_host(Some(local_host)),
        daemon_db_path.unwrap_or(crate::build_info::MACHINE_NON_ATTESTEE),
        crate::build_info::describe_host(daemon_host)
    ))
}

fn collect_recovery_losses(
    db_path: &Path,
) -> Result<Option<crate::recovery_trace::RecoveryLossReport>, String> {
    let fleet_path = crate::desired_state::path_for_daemon_db(db_path);
    let path = crate::recovery_trace::report_path(&fleet_path);
    crate::recovery_trace::load_report(&path)
        .map_err(|error| format!("trace de reprise illisible {}: {error}", path.display()))
}

fn collect_ledger(
    config: &DaemonConfig,
) -> (
    Result<Vec<RequestInfo>, String>,
    Result<Vec<LedgerMessage>, String>,
) {
    match send_ledger_both(config) {
        Ok((requests, messages)) => (Ok(requests), Ok(messages)),
        Err(daemon_error) => match Store::open(&config.db_path) {
            Ok(store) => {
                let projection = ledger::read_projection(&store, LedgerScope::Both, 40)
                    .map_err(|error| format!("ledger base illisible: {error}"));
                match projection {
                    Ok(projection) => (Ok(projection.requests), Ok(projection.messages)),
                    Err(error) => (
                        Err(format!("{daemon_error}; {error}")),
                        Err(format!("{daemon_error}; {error}")),
                    ),
                }
            }
            Err(error) => (
                Err(format!("{daemon_error}; base inaccessible: {error}")),
                Err(format!("{daemon_error}; base inaccessible: {error}")),
            ),
        },
    }
}

fn send_ledger_both(
    config: &DaemonConfig,
) -> Result<(Vec<RequestInfo>, Vec<LedgerMessage>), String> {
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::net::UnixStream;

    use bridget_transport::DaemonToWrapper;
    use bridget_transport::protocol::{PresenceMode, WrapperToDaemon, decode, encode};

    if !config.socket_path.exists() {
        return Err("daemon hors ligne (socket absente)".to_string());
    }
    let stream = UnixStream::connect(&config.socket_path)
        .map_err(|error| format!("daemon inaccessible: {error}"))?;
    let mut writer = BufWriter::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut reader = BufReader::new(stream);

    let reg = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        identity_version: 2,
        agent_id: uuid::Uuid::new_v4().to_string(),
        host: Some(crate::build_info::local_host()),
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        // Une connexion CLI éphémère ne tient aucun journal : elle n'est jamais attachable.
        journal_available: Some(false),
    };
    writeln!(writer, "{}", encode(&reg).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    let _registered: DaemonToWrapper = decode(line.trim_end()).map_err(|e| e.to_string())?;

    let command = WrapperToDaemon::LedgerProjection {
        scope: LedgerScope::Both,
        limit: 40,
    };
    writeln!(writer, "{}", encode(&command).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    line.clear();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    match decode(line.trim_end()).map_err(|e| e.to_string())? {
        DaemonToWrapper::LedgerProjection { messages, requests } => Ok((requests, messages)),
        DaemonToWrapper::Nack { reason, .. } => Err(format!("ledger: {reason}")),
        _ => Err("réponse ledger inattendue".to_string()),
    }
}

fn collect_git(repo: &Path) -> Result<GitSnapshot, String> {
    let head = git_output(repo, &["rev-parse", "--short", "main"])
        .or_else(|_| git_output(repo, &["rev-parse", "--short", "HEAD"]))?;
    let head_message = git_output(repo, &["log", "-1", "--format=%s", "main"])
        .or_else(|_| git_output(repo, &["log", "-1", "--format=%s"]))?;
    let porcelain = git_output(repo, &["worktree", "list", "--porcelain"])?;
    let mut worktrees = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_head: Option<String> = None;
    let mut current_branch: Option<String> = None;
    for line in porcelain.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let (Some(path), Some(head)) = (current_path.take(), current_head.take()) {
                worktrees.push(GitWorktree {
                    dirty: git_dirty(&path),
                    path,
                    head,
                    branch: current_branch.take(),
                });
            }
            current_path = Some(path.to_string());
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            current_head = Some(head.chars().take(12).collect());
        } else if let Some(branch) = line.strip_prefix("branch ") {
            current_branch = Some(
                branch
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch)
                    .to_string(),
            );
        } else if line == "detached" {
            current_branch = Some("detached".to_string());
        }
    }
    if let (Some(path), Some(head)) = (current_path, current_head) {
        worktrees.push(GitWorktree {
            dirty: git_dirty(&path),
            path,
            head,
            branch: current_branch,
        });
    }
    Ok(GitSnapshot {
        head,
        head_message,
        worktrees,
    })
}

fn git_dirty(worktree: &str) -> Option<bool> {
    let output = Command::new("git")
        .args(["-C", worktree, "status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!output.stdout.is_empty())
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("git indisponible: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn load_pin(path: &Path) -> Result<LoadedPin, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("pin illisible ({}): {error}", path.display()))?;
    let pin: PinFile = serde_yaml::from_str(&raw)
        .map_err(|error| format!("pin YAML invalide ({}): {error}", path.display()))?;
    let modified_at = fs::metadata(path)
        .and_then(|meta| meta.modified())
        .unwrap_or(UNIX_EPOCH);
    Ok(LoadedPin {
        pin,
        path: absolutize(path),
        modified_at,
    })
}

/// Rend la carte YAML. Ne panique jamais sur une source absente.
pub fn render_card(snapshot: &RepriseSnapshot) -> String {
    let mut out = String::new();
    let genere_a = format_rfc3339(snapshot.now);
    let age_sec = 0u64;
    out.push_str("meta:\n");
    out.push_str(&format!("  genere_a: {genere_a}\n"));
    out.push_str(&format!("  age_sec: {age_sec}\n"));
    out.push_str("  commande_regen: bridget reprise --write\n");
    out.push('\n');

    render_vivant(&mut out, snapshot);
    out.push('\n');
    render_chantier(&mut out, snapshot);
    out.push('\n');
    render_attentes(&mut out, snapshot);
    out.push('\n');
    render_prochain(&mut out, snapshot);
    out.push('\n');
    render_pieges(&mut out, snapshot);
    out.push('\n');
    render_acces(&mut out, snapshot);
    out
}

fn render_vivant(out: &mut String, snapshot: &RepriseSnapshot) {
    out.push_str("# 0 — L'ÉQUIPE EST-ELLE VIVANTE ?\n");
    out.push_str("vivant:\n");
    match &snapshot.status {
        Ok(status) => {
            out.push_str(&format!(
                "  daemon_en_ligne: {}\n",
                if status.running { "true" } else { "false" }
            ));
            // Chemins RÉELLEMENT ceux de status (DaemonConfig résolu), jamais
            // une reconstruction parallèle depuis un HOME inventé. Chacun porte
            // désormais SA machine : `socket` et `db_locale` valent ici, `db`
            // vaut là où le daemon tourne, et les deux diffèrent en fédération.
            out.push_str(&format!(
                "  machine_daemon: {}\n",
                yaml_string(crate::build_info::describe_host(
                    status.daemon_host.as_deref()
                ))
            ));
            out.push_str(&format!(
                "  socket: {}\n",
                yaml_string(&snapshot.socket_path.display().to_string())
            ));
            out.push_str(&format!(
                "  db: {}\n",
                yaml_string(
                    status
                        .daemon_db_path
                        .as_deref()
                        .unwrap_or(crate::build_info::MACHINE_NON_ATTESTEE)
                )
            ));
            out.push_str(&format!(
                "  db_locale: {}\n",
                yaml_string(&snapshot.db_path.display().to_string())
            ));
            out.push_str(&format!(
                "  build_id_daemon: {}\n",
                yaml_string(status.build_id.as_deref().unwrap_or("inconnu"))
            ));
            match &snapshot.binary_cli {
                Some(path) => out.push_str(&format!(
                    "  binaire_cli: {}\n",
                    yaml_string(&path.display().to_string())
                )),
                None => out.push_str("  binaire_cli: indisponible\n"),
            }
            out.push_str(&format!("  agents_connectes: {}\n", status.agents.len()));
            out.push_str("  noms:\n");
            if status.agents.is_empty() {
                if status.running {
                    out.push_str("    - (aucun agent connecté)\n");
                } else {
                    out.push_str("    - indisponible: daemon hors ligne\n");
                }
            } else {
                for agent in status.agents.iter().take(20) {
                    out.push_str(&format!("    - {}\n", yaml_string(&format_agent(agent))));
                }
            }
            render_recovery_losses(out, snapshot);
            out.push_str("  preuve_30s: \"bridget status && bridget who\"\n");
        }
        Err(error) => {
            out.push_str("  daemon_en_ligne: false\n");
            out.push_str(&format!(
                "  machine_daemon: {}\n",
                yaml_string(crate::build_info::MACHINE_NON_ATTESTEE)
            ));
            out.push_str(&format!(
                "  socket: {}\n",
                yaml_string(&snapshot.socket_path.display().to_string())
            ));
            out.push_str(&format!(
                "  db_locale: {}\n",
                yaml_string(&snapshot.db_path.display().to_string())
            ));
            out.push_str("  build_id_daemon: indisponible\n");
            out.push_str("  binaire_cli: indisponible\n");
            out.push_str("  agents_connectes: 0\n");
            out.push_str("  noms:\n");
            out.push_str(&format!("    - indisponible: {}\n", yaml_string(error)));
            render_recovery_losses(out, snapshot);
            out.push_str("  preuve_30s: \"bridget status && bridget who\"\n");
        }
    }
}

fn render_recovery_losses(out: &mut String, snapshot: &RepriseSnapshot) {
    match &snapshot.recovery_losses {
        Ok(None) => {}
        Ok(Some(report)) => {
            out.push_str(&format!("  pertes_reprise_at: {}\n", report.recorded_at));
            out.push_str("  pertes_reprise:\n");
            for absent in &report.absents {
                match absent.detail.as_deref() {
                    Some(detail) if !detail.is_empty() => out.push_str(&format!(
                        "    - name: {}\n      reason: {}\n      detail: {}\n",
                        yaml_string(&absent.name),
                        yaml_string(&absent.reason),
                        yaml_string(detail)
                    )),
                    _ => out.push_str(&format!(
                        "    - name: {}\n      reason: {}\n",
                        yaml_string(&absent.name),
                        yaml_string(&absent.reason)
                    )),
                }
            }
        }
        Err(error) => {
            out.push_str(&format!(
                "  pertes_reprise: indisponible ({})\n",
                yaml_string(error)
            ));
        }
    }
}

fn format_agent(agent: &AgentInfo) -> String {
    format!(
        "{} ({}, {})",
        agent.display_name, agent.agent_type, agent.state
    )
}

fn render_chantier(out: &mut String, snapshot: &RepriseSnapshot) {
    out.push_str("# 1 — OÙ EN EST LE PROJET ?\n");
    out.push_str("chantier:\n");
    match &snapshot.git {
        Ok(git) => {
            out.push_str("  main:\n");
            out.push_str(&format!("    commit: {}\n", yaml_string(&git.head)));
            out.push_str(&format!(
                "    message_1l: {}\n",
                yaml_string(&git.head_message)
            ));
            out.push_str("  worktrees:\n");
            if git.worktrees.is_empty() {
                out.push_str("    - (aucun)\n");
            } else {
                for worktree in &git.worktrees {
                    out.push_str(&format!("    - path: {}\n", yaml_string(&worktree.path)));
                    out.push_str(&format!(
                        "      branch: {}\n",
                        yaml_string(worktree.branch.as_deref().unwrap_or("?"))
                    ));
                    out.push_str(&format!(
                        "      commit_court: {}\n",
                        yaml_string(&worktree.head)
                    ));
                    match worktree.dirty {
                        Some(true) => out.push_str("      dirty: true\n"),
                        Some(false) => out.push_str("      dirty: false\n"),
                        None => out.push_str("      dirty: indisponible\n"),
                    }
                }
            }
        }
        Err(error) => {
            out.push_str(&format!(
                "  main: {}\n",
                yaml_string(&format!("indisponible: {error}"))
            ));
            out.push_str("  worktrees: []\n");
        }
    }

    out.push_str("  couloirs:\n");
    match pin_section(snapshot) {
        PinSection::Ok(loaded) if !loaded.pin.couloirs.is_empty() => {
            for couloir in &loaded.pin.couloirs {
                out.push_str(&format!("    - id: {}\n", yaml_string(&couloir.id)));
                out.push_str(&format!("      owner: {}\n", yaml_string(&couloir.owner)));
                if let Some(worktree) = &couloir.worktree {
                    out.push_str(&format!("      worktree: {}\n", yaml_string(worktree)));
                }
                if !couloir.fichiers.is_empty() {
                    out.push_str("      fichiers:\n");
                    for fichier in &couloir.fichiers {
                        out.push_str(&format!("        - {}\n", yaml_string(fichier)));
                    }
                }
            }
        }
        PinSection::Ok(_) => out.push_str("    - (aucun couloir dans le pin)\n"),
        PinSection::Stale(loaded) => {
            out.push_str(&format!(
                "    - {}\n",
                yaml_string(&format!(
                    "PIN périmé ({} > 1h) — couloirs non fiables jusqu'à mise à jour de {}",
                    format_age(snapshot.now, loaded.modified_at),
                    loaded.path.display()
                ))
            ));
        }
        PinSection::Missing => out.push_str("    - indisponible: pin absent\n"),
        PinSection::Err(error) => out.push_str(&format!(
            "    - {}\n",
            yaml_string(&format!("indisponible: {error}"))
        )),
    }
}

fn render_attentes(out: &mut String, snapshot: &RepriseSnapshot) {
    out.push_str("# 2 — QUI ATTEND QUOI ?\n");
    out.push_str("attentes:\n");
    out.push_str(
        "  # Projection GLOBALE via LedgerProjection/open_requests — pas bridget requests.\n",
    );
    out.push_str("  bridget_ouvertes:\n");
    match &snapshot.open_requests {
        Ok(requests) if requests.is_empty() => {
            out.push_str("    - (aucune demande ouverte)\n");
        }
        Ok(requests) => {
            let messages = snapshot.recent_messages.as_ref().ok();
            for request in requests.iter().take(20) {
                let body = messages
                    .and_then(|entries| {
                        entries
                            .iter()
                            .find(|entry| entry.id == request.id)
                            .map(|entry| truncate(&entry.body, 120))
                    })
                    .unwrap_or_else(|| "(corps absent du ledger récent)".to_string());
                out.push_str(&format!("    - id: {}\n", yaml_string(&request.id)));
                out.push_str(&format!("      de: {}\n", yaml_string(&request.sender)));
                out.push_str(&format!("      vers: {}\n", yaml_string(&request.target)));
                out.push_str(&format!("      etat: {}\n", yaml_string(&request.state)));
                out.push_str(&format!("      deadline_at: {}\n", request.deadline_at));
                out.push_str(&format!(
                    "      extrait_corps_120c: {}\n",
                    yaml_string(&body)
                ));
            }
        }
        Err(error) => {
            out.push_str(&format!(
                "    - {}\n",
                yaml_string(&format!("indisponible: {error}"))
            ));
        }
    }

    out.push_str("  reserves_explicites:\n");
    match (&snapshot.open_requests, &snapshot.recent_messages) {
        (Ok(requests), Ok(messages)) => {
            let open_ids: std::collections::BTreeSet<_> =
                requests.iter().map(|request| request.id.as_str()).collect();
            let mut found = false;
            for message in messages {
                if !open_ids.contains(message.id.as_str()) {
                    continue;
                }
                found = true;
                out.push_str(&format!("    - de: {}\n", yaml_string(&message.sender)));
                out.push_str(&format!("      vers: {}\n", yaml_string(&message.target)));
                out.push_str(&format!(
                    "      pourquoi_120c: {}\n",
                    yaml_string(&truncate(&message.body, 120))
                ));
                out.push_str(&format!("      id_message: {}\n", yaml_string(&message.id)));
            }
            if !found {
                out.push_str(
                    "    - (aucune réserve liée à une demande ouverte dans le ledger récent)\n",
                );
            }
        }
        (Err(error), _) | (_, Err(error)) => {
            out.push_str(&format!(
                "    - {}\n",
                yaml_string(&format!("indisponible: {error}"))
            ));
        }
    }
}

fn render_prochain(out: &mut String, snapshot: &RepriseSnapshot) {
    out.push_str("# 3 — PROCHAIN GESTE + PIÈGES\n");
    out.push_str("prochain:\n");
    match pin_section(snapshot) {
        PinSection::Ok(loaded) => {
            let geste = loaded
                .pin
                .prochain
                .as_ref()
                .map(|prochain| prochain.geste.as_str())
                .unwrap_or("(geste absent du pin)");
            out.push_str(&format!("  geste: {}\n", yaml_string(geste)));
            let proprietaire = loaded
                .pin
                .prochain
                .as_ref()
                .and_then(|prochain| prochain.proprietaire.as_deref())
                .or_else(|| infer_owner(snapshot))
                .unwrap_or("referent");
            out.push_str(&format!("  proprietaire: {}\n", yaml_string(proprietaire)));
        }
        PinSection::Stale(loaded) => {
            out.push_str(&format!(
                "  geste: {}\n",
                yaml_string(&format!(
                    "⚠ PIN périmé ({}) — relire ledger / mettre à jour {}",
                    format_age(snapshot.now, loaded.modified_at),
                    loaded.path.display()
                ))
            ));
            out.push_str(&format!(
                "  proprietaire: {}\n",
                yaml_string(infer_owner(snapshot).unwrap_or("referent"))
            ));
        }
        PinSection::Missing => {
            out.push_str("  geste: indisponible: pin absent\n");
            out.push_str(&format!(
                "  proprietaire: {}\n",
                yaml_string(infer_owner(snapshot).unwrap_or("referent"))
            ));
        }
        PinSection::Err(error) => {
            out.push_str(&format!(
                "  geste: {}\n",
                yaml_string(&format!("indisponible: {error}"))
            ));
            out.push_str("  proprietaire: referent\n");
        }
    }

    out.push_str("  bloque_par:\n");
    match &snapshot.open_requests {
        Ok(requests) if requests.is_empty() => out.push_str("    - (aucune)\n"),
        Ok(requests) => {
            for request in requests.iter().take(10) {
                out.push_str(&format!("    - {}\n", yaml_string(&request.id)));
            }
        }
        Err(error) => out.push_str(&format!(
            "    - {}\n",
            yaml_string(&format!("indisponible: {error}"))
        )),
    }
}

fn render_pieges(out: &mut String, snapshot: &RepriseSnapshot) {
    out.push_str("pieges:\n");
    out.push_str("  regles: docs/regles-chantier.md\n");
    out.push_str("  actifs_session:\n");
    match pin_section(snapshot) {
        PinSection::Ok(loaded) => {
            let actifs = loaded
                .pin
                .pieges
                .as_ref()
                .map(|pieges| pieges.actifs_session.as_slice())
                .unwrap_or(&[]);
            if actifs.is_empty() {
                out.push_str("    - (aucun piège de session dans le pin)\n");
            } else {
                for piege in actifs.iter().take(5) {
                    out.push_str(&format!("    - {}\n", yaml_string(piege)));
                }
            }
        }
        PinSection::Stale(_) => {
            out.push_str("    - PIN périmé — pièges de session non fiables\n");
        }
        PinSection::Missing => out.push_str("    - indisponible: pin absent\n"),
        PinSection::Err(error) => out.push_str(&format!(
            "    - {}\n",
            yaml_string(&format!("indisponible: {error}"))
        )),
    }
    out.push_str(
        "  piege_classique: \"Ne pas conclure mort sans bridget status — socket ≠ docs/\"\n",
    );
}

fn render_acces(out: &mut String, snapshot: &RepriseSnapshot) {
    out.push_str("acces:\n");
    out.push_str(&format!(
        "  repo: {}\n",
        yaml_string(&snapshot.repo.display().to_string())
    ));
    out.push_str(&format!(
        "  socket: {}\n",
        yaml_string(&snapshot.socket_path.display().to_string())
    ));
    out.push_str(&format!(
        "  bridget_db_locale: {}\n",
        yaml_string(&snapshot.db_path.display().to_string())
    ));
}

enum PinSection<'a> {
    Ok(&'a LoadedPin),
    Stale(&'a LoadedPin),
    Missing,
    Err(&'a str),
}

fn pin_section(snapshot: &RepriseSnapshot) -> PinSection<'_> {
    match &snapshot.pin {
        None => PinSection::Missing,
        Some(Err(error)) => PinSection::Err(error),
        Some(Ok(loaded)) => {
            if is_pin_stale(snapshot.now, loaded.modified_at) {
                PinSection::Stale(loaded)
            } else {
                PinSection::Ok(loaded)
            }
        }
    }
}

pub fn is_pin_stale(now: SystemTime, modified_at: SystemTime) -> bool {
    now.duration_since(modified_at)
        .map(|age| age > PIN_STALE_AFTER)
        .unwrap_or(true)
}

fn infer_owner(snapshot: &RepriseSnapshot) -> Option<&str> {
    let Ok(requests) = &snapshot.open_requests else {
        return None;
    };
    if requests.len() == 1 {
        return Some(requests[0].target.as_str());
    }
    None
}

fn format_age(now: SystemTime, then: SystemTime) -> String {
    match now.duration_since(then) {
        Ok(duration) => {
            let secs = duration.as_secs();
            if secs >= 3600 {
                format!("{}h{}", secs / 3600, (secs % 3600) / 60)
            } else {
                format!("{}m", secs / 60)
            }
        }
        Err(_) => "horloge incohérente".to_string(),
    }
}

fn format_rfc3339(time: SystemTime) -> String {
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "1970-01-01T00:00:00Z".to_string();
    };
    let secs = duration.as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let day_secs = secs.rem_euclid(86_400) as u64;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    // Algorithme civil depuis jours Unix (Howard Hinnant), suffisant pour un horodatage.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn truncate(text: &str, max_chars: usize) -> String {
    let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result: String = trimmed.chars().take(max_chars).collect();
    if trimmed.chars().count() > max_chars {
        result.push('…');
    }
    result
}

fn yaml_string(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    let needs_quote = value.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                ':' | '#'
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | ','
                    | '&'
                    | '*'
                    | '!'
                    | '|'
                    | '>'
                    | '\''
                    | '"'
                    | '%'
                    | '@'
                    | '`'
            )
            || value.starts_with('-')
            || value == "true"
            || value == "false"
            || value == "null"
    });
    if needs_quote {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

pub fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

pub fn discover_repo(start: &Path) -> Option<PathBuf> {
    let mut current = absolutize(start);
    loop {
        if current.join("Cargo.toml").is_file() && current.join("docs").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn write_card(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("création {}: {error}", parent.display()))?;
    }
    let tmp = path.with_extension("yaml.tmp");
    fs::write(&tmp, content).map_err(|error| format!("écriture {}: {error}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|error| format!("rename {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn base_snapshot(now: SystemTime) -> RepriseSnapshot {
        RepriseSnapshot {
            now,
            repo: PathBuf::from("/repo"),
            socket_path: PathBuf::from("/resolved/cache/bridget.sock"),
            db_path: PathBuf::from("/resolved/cache/bridget.db"),
            binary_cli: Some(PathBuf::from("/resolved/bin/bridget")),
            status: Ok(DaemonStatus {
                running: true,
                agents_inventory_available: true,
                agents: vec![AgentInfo {
                    agent_id: uuid::Uuid::new_v4().to_string(),
                    display_name: "coderBridget".to_string(),
                    agent_type: "codex".to_string(),
                    connection_id: "conn-1".to_string(),
                    host: "local".to_string(),
                    transport: "unix".to_string(),
                    channel: None,
                    state: "connected".to_string(),
                    last_seen_secs: 0,
                    reconnect_count: 0,
                    mode: None,
                    location: None,
                    os: "macOS".to_string(),
                    domain: None,
                    model: None,
                    effort: None,
                    rate_limits: Default::default(),
                    model_mismatch: None,
                    disk_space: None,
                    provider: None,
                    persistent: None,
                    execution: None,
                    agent_link: None,
                }],
                message_count: Some(3),
                build_id: Some("abc123".to_string()),
                // Le daemon de la fixture tourne AILLEURS : c'est le cas fédéré,
                // le seul où l'attribution est vérifiable.
                daemon_host: Some("poste-alpha".to_string()),
                daemon_db_path: Some("/Users/user/.cache/bridget/bridget.db".to_string()),
            }),
            open_requests: Ok(vec![]),
            recent_messages: Ok(vec![]),
            git: Ok(GitSnapshot {
                head: "deadbeef".to_string(),
                head_message: "test".to_string(),
                worktrees: vec![],
            }),
            pin: None,
            recovery_losses: Ok(None),
        }
    }

    #[test]
    fn bloc_0_expose_les_chemins_resolus_de_status_sans_reconstruire_home() {
        let card = render_card(&base_snapshot(
            SystemTime::UNIX_EPOCH + Duration::from_secs(10),
        ));
        assert!(
            card.contains("socket: /resolved/cache/bridget.sock"),
            "la carte doit recopier le chemin résolu de status: {card}"
        );
        assert!(card.contains("db_locale: /resolved/cache/bridget.db"));
        assert!(
            !card.contains("$HOME") && !card.contains("~/.cache"),
            "aucun chemin reconstruit depuis HOME: {card}"
        );
        assert!(card.contains("build_id_daemon: abc123"));
        assert!(card.contains("coderBridget (codex, connected)"));
        assert!(
            !card.contains("pertes_reprise"),
            "zéro perte ne doit pas faire de bruit: {card}"
        );
    }

    /// POINT 2 — la carte décrivait la flotte à partir d'un fichier qui n'est
    /// pas le sien. Chaque chemin doit désormais porter SA machine.
    ///
    /// La fixture place le daemon sur une AUTRE machine que le client : c'est la
    /// seule configuration où l'attribution est vérifiable.
    ///
    /// Mutant qui tue ce test : réafficher `snapshot.db_path` sous la clé `db`
    /// → l'assertion sur le chemin du daemon meurt, et celle qui interdit de
    /// présenter la base locale comme celle du daemon meurt aussi.
    #[test]
    fn la_carte_attribue_chaque_base_a_sa_machine() {
        let card = render_card(&base_snapshot(
            SystemTime::UNIX_EPOCH + Duration::from_secs(10),
        ));
        assert!(
            card.contains("machine_daemon: poste-alpha"),
            "la carte doit nommer la machine du daemon: {card}"
        );
        assert!(
            card.contains("db: /Users/user/.cache/bridget/bridget.db"),
            "la clé db doit porter la base ATTESTÉE par le daemon: {card}"
        );
        assert!(
            card.contains("db_locale: /resolved/cache/bridget.db"),
            "la base d'ici doit être nommée comme locale: {card}"
        );
        assert!(
            !card.contains("db: /resolved/cache/bridget.db"),
            "la base locale ne doit jamais être présentée comme celle du daemon: {card}"
        );
    }

    /// POINT 2 — la trace de reprise locale décrit une AUTRE flotte quand la
    /// base du daemon n'est pas la nôtre : on ne la lit pas, et on dit pourquoi.
    ///
    /// Mutant qui tue ce test : lire la trace sans la garde → `recovery_losses`
    /// redevient `Ok(..)` et l'assertion meurt.
    #[test]
    fn la_trace_de_reprise_n_est_pas_lue_quand_la_base_est_ailleurs() {
        let config = DaemonConfig {
            socket_path: std::env::temp_dir()
                .join(format!("b89-reprise-{}.sock", uuid::Uuid::new_v4())),
            ..DaemonConfig::default()
        };
        let snapshot = collect_snapshot(
            &config,
            Path::new("/repo"),
            None,
            SystemTime::UNIX_EPOCH + Duration::from_secs(10),
        );
        let error = snapshot
            .recovery_losses
            .expect_err("sans base attestée, la trace locale ne doit pas être lue");
        // Daemon injoignable : rien n'est attesté. Le motif rendu doit être
        // celui de l'INDÉTERMINATION, pas celui de deux machines différentes —
        // les deux n'appellent pas le même geste de la part d'un opérateur.
        assert!(
            error.contains(MOTIF_INDETERMINATION),
            "le refus doit dire pourquoi: {error}"
        );
        assert!(
            !error.contains(MOTIF_BASES_DIFFERENTES),
            "le motif générique ne doit pas être rendu pour une indétermination: {error}"
        );
    }

    #[test]
    fn source_indisponible_se_declare_sans_inventer_ni_echouer_en_bloc() {
        let mut snapshot = base_snapshot(SystemTime::UNIX_EPOCH + Duration::from_secs(10));
        snapshot.status = Ok(DaemonStatus {
            running: false,
            agents: vec![],
            agents_inventory_available: false,
            message_count: None,
            build_id: None,
            daemon_host: None,
            daemon_db_path: None,
        });
        snapshot.open_requests = Err("daemon hors ligne (socket absente)".to_string());
        snapshot.recent_messages = Err("daemon hors ligne (socket absente)".to_string());
        snapshot.git = Err("git indisponible: ENOENT".to_string());

        let card = render_card(&snapshot);
        assert!(card.contains("daemon_en_ligne: false"));
        assert!(card.contains("socket: /resolved/cache/bridget.sock"));
        assert!(card.contains("indisponible: daemon hors ligne"));
        assert!(card.contains("indisponible: git indisponible"));
        assert!(
            !card.contains("inventé") && !card.contains("/tmp/bridget.sock"),
            "pas de valeur inventée: {card}"
        );
    }

    #[test]
    fn sonde_identite_indisponible_ne_devient_pas_absence_d_agents() {
        let mut snapshot = base_snapshot(SystemTime::UNIX_EPOCH + Duration::from_secs(10));
        snapshot.status =
            Err("identité du daemon indisponible: délai de lecture de 2 s dépassé".to_string());

        let card = render_card(&snapshot);
        assert!(card.contains("daemon_en_ligne: false"));
        assert!(card.contains("identité du daemon indisponible"));
        assert!(card.contains("délai de lecture de 2 s dépassé"));
        assert!(!card.contains("(aucun agent connecté)"));
    }

    #[test]
    fn carte_nomme_chaque_absent_de_la_trace_de_reprise() {
        let mut snapshot = base_snapshot(SystemTime::UNIX_EPOCH + Duration::from_secs(10));
        snapshot.recovery_losses = Ok(Some(crate::recovery_trace::RecoveryLossReport {
            schema: 1,
            recorded_at: 42,
            absents: vec![
                crate::recovery_trace::RecoveryLossEntry {
                    name: "cursor3".to_string(),
                    reason: crate::recovery_trace::REASON_NON_PERSISTENT.to_string(),
                    detail: Some("spawn sans --persistent".to_string()),
                },
                crate::recovery_trace::RecoveryLossEntry {
                    name: "cursor8".to_string(),
                    reason: crate::recovery_trace::REASON_QUOTA.to_string(),
                    detail: Some("quota de flotte atteint (8)".to_string()),
                },
            ],
        }));
        let card = render_card(&snapshot);
        assert!(card.contains("name: cursor3"));
        assert!(card.contains("reason: non_persistant"));
        assert!(card.contains("name: cursor8"));
        assert!(card.contains("reason: quota_flotte"));
    }

    #[test]
    fn pin_plus_vieux_d_une_heure_s_affiche_comme_perime() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
        let mut snapshot = base_snapshot(now);
        snapshot.pin = Some(Ok(LoadedPin {
            pin: PinFile {
                prochain: Some(PinProchain {
                    geste: "ne doit plus être affiché comme frais".to_string(),
                    proprietaire: Some("coderBridget".to_string()),
                }),
                couloirs: vec![PinCouloir {
                    id: "A".to_string(),
                    owner: "coderBridget".to_string(),
                    fichiers: vec![],
                    worktree: None,
                }],
                pieges: Some(PinPieges {
                    actifs_session: vec!["piège périmé".to_string()],
                }),
            },
            path: PathBuf::from("/repo/docs/carte-reprise.pin.yaml"),
            modified_at: now - PIN_STALE_AFTER - Duration::from_secs(1),
        }));

        assert!(is_pin_stale(
            now,
            now - PIN_STALE_AFTER - Duration::from_secs(1)
        ));
        let card = render_card(&snapshot);
        assert!(
            card.contains("PIN périmé") || card.contains("⚠ PIN périmé"),
            "pin périmé attendu: {card}"
        );
        assert!(
            !card.contains("ne doit plus être affiché comme frais"),
            "geste frais du pin périmé ne doit pas apparaître: {card}"
        );
    }

    #[test]
    fn pin_frais_expose_le_geste() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
        let mut snapshot = base_snapshot(now);
        snapshot.pin = Some(Ok(LoadedPin {
            pin: PinFile {
                prochain: Some(PinProchain {
                    geste: "livrer carte reprise".to_string(),
                    proprietaire: None,
                }),
                ..PinFile::default()
            },
            path: PathBuf::from("/repo/docs/carte-reprise.pin.yaml"),
            modified_at: now - Duration::from_secs(60),
        }));
        let card = render_card(&snapshot);
        assert!(
            card.contains("geste: \"livrer carte reprise\"")
                || card.contains("geste: livrer carte reprise")
        );
    }

    /// M4 — le contrôle POSITIF qui manquait.
    ///
    /// Un mutant rendant la condition toujours fausse laissait toute la famille
    /// `reprise::tests` verte : le test existant ne prouvait que le REFUS
    /// distant, jamais qu'une trace locale attestée reste lisible. Les deux
    /// issues sont désormais éprouvées sur la même fonction.
    ///
    /// Mutant qui tue ce test : rendre la condition toujours fausse → le premier
    /// cas meurt en affichant le motif de refus qu'il n'aurait pas dû recevoir.
    #[test]
    fn la_trace_est_lue_ici_et_refusee_ailleurs() {
        let chemin = PathBuf::from("/home/utilisateur/.cache/bridget/bridget.db");

        // POSITIF : même machine, même chemin → la trace nous concerne.
        assert_eq!(
            recovery_trace_scope(
                Some("poste-beta"),
                Some("/home/utilisateur/.cache/bridget/bridget.db"),
                "poste-beta",
                &chemin
            ),
            Ok(()),
            "une base attestée locale doit rester lisible"
        );

        // NÉGATIF, et c'est le cas piégeux : MÊME CHEMIN, autre machine.
        let refus = recovery_trace_scope(
            Some("poste-alpha"),
            Some("/home/utilisateur/.cache/bridget/bridget.db"),
            "poste-beta",
            &chemin,
        )
        .expect_err("un daemon distant ne rend pas la trace locale lisible");
        assert!(
            refus.contains("sur poste-beta") && refus.contains("sur poste-alpha"),
            "le motif doit nommer LES DEUX machines : {refus}"
        );
        // Contrôle de séparation dans l'autre sens : deux machines ATTESTÉES et
        // différentes doivent recevoir le motif générique, jamais celui de
        // l'indétermination.
        assert!(
            refus.contains(MOTIF_BASES_DIFFERENTES) && !refus.contains(MOTIF_INDETERMINATION),
            "deux machines nommées ne relèvent pas de l'indétermination : {refus}"
        );

        // Daemon antérieur : non attesté, donc refus explicite.
        let inconnu = recovery_trace_scope(None, None, "poste-beta", &chemin)
            .expect_err("sans attestation, la trace n'est pas réputée locale");
        assert!(
            inconnu.contains(crate::build_info::MACHINE_NON_ATTESTEE),
            "{inconnu}"
        );

        // DEUX HÔTES INDÉTERMINÉS, MÊME CHEMIN — le cas de rc7. Refusé, et le
        // motif doit dire POURQUOI : machine non attestée, pas « deux machines
        // différentes ». Les deux refus n'appellent pas le même geste.
        let repli = recovery_trace_scope(
            Some(bridget_core::HOTE_NON_ATTESTE),
            Some("/home/utilisateur/.cache/bridget/bridget.db"),
            bridget_core::HOTE_NON_ATTESTE,
            &chemin,
        )
        .expect_err("deux machines indéterminées ne sont pas la même machine");
        // LES DEUX CONDITIONS. La mention « machine non attestée » figure AUSSI
        // dans le motif générique, qui affiche l'hôte du daemon : l'exiger seule
        // ne sépare pas les issues. Il faut le motif spécifique PRÉSENT et le
        // motif générique ABSENT — sinon un code qui ne changerait rien du tout
        // passerait le test.
        assert!(
            repli.contains(MOTIF_INDETERMINATION),
            "le motif doit nommer l'indétermination : {repli}"
        );
        assert!(
            !repli.contains(MOTIF_BASES_DIFFERENTES),
            "le motif générique des bases différentes ne doit PAS être rendu ici : {repli}"
        );
        // Rendu unique des DEUX côtés : la machine locale non attestée se lit
        // comme telle, jamais sous sa valeur de repli brute.
        assert!(
            !repli.contains(&format!("sur {}", bridget_core::HOTE_NON_ATTESTE)),
            "la machine locale non attestée doit se lire comme telle : {repli}"
        );
    }
}
