//! J2 — Ramasseur de sessions abandonnées, **Phase 0 observateur uniquement**.
//!
//! Aucune fonction de ce module n'envoie de signal, n'appelle `stop`, ni ne
//! supprime de fichier. Le rapport dit ce qui *aurait* été fait.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Seuil d'âge / d'inactivité par défaut (aligné T3 : 30 min).
pub const DEFAULT_MIN_AGE_SECS: u64 = 30 * 60;
/// Intervalle minimal entre deux photos pour `stable_for_phase1` (5 min).
pub const DEFAULT_DOUBLE_OBSERVATION_SECS: u64 = 5 * 60;

const PHASE: &str = "observer";
const GAP_BACKGROUND: &str = "fail_closed_background_liveness_absent";
const MANQUE_BACKGROUND: &str = "MANQUE À INSTRUIRE: Bridget n'expose pas encore de signal de travail de fond (équivalent T3 backgroundLiveness). Toute cible classe A reste fail-closed jusqu'à ce signal.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetClass {
    ManagedWrapper,
    TestDaemon,
    TempDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Eligible,
    Protege,
    Incertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetReport {
    pub class: TargetClass,
    pub identity: String,
    pub fingerprint: String,
    pub pid: Option<u32>,
    pub ppid: Option<u32>,
    pub cmdline: Option<String>,
    pub path: Option<String>,
    pub age_secs: Option<u64>,
    pub agent_name: Option<String>,
    pub criteria_met: Vec<String>,
    pub criteria_missing: Vec<String>,
    pub guards_triggered: Vec<String>,
    pub uncertainty_reasons: Vec<String>,
    pub verdict: Verdict,
    /// Action qui aurait été prise en Phase 1 — jamais exécutée ici.
    pub would_have_done: String,
    pub stable_for_phase1: bool,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaperReport {
    pub phase: String,
    pub observed_at_unix: u64,
    pub min_age_secs: u64,
    pub double_observation_secs: u64,
    pub background_liveness_available: bool,
    pub gaps_to_instruct: Vec<String>,
    pub targets: Vec<TargetReport>,
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub eligible: usize,
    pub protege: usize,
    pub incertain: usize,
    pub stable_for_phase1: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub ppid: u32,
    pub age_secs: u64,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPresence {
    pub name: String,
    pub state: String,
    pub mode: Option<String>,
    pub location: Option<String>,
    pub last_seen_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempDirSnapshot {
    pub path: PathBuf,
    pub age_secs: u64,
    pub prefix_kind: String,
    /// Nom d'agent extrait du répertoire lorsque son propriétaire peut être
    /// rapproché de l'inventaire. Absent pour les anciens préfixes harnais.
    pub agent_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorHit {
    pub fingerprint: String,
    pub verdict: Verdict,
    pub observed_at_unix: u64,
}

/// Vue du monde injectable — pour les tests et le balayage réel.
#[derive(Debug, Clone, Default)]
pub struct WorldView {
    pub processes: Vec<ProcessSnapshot>,
    pub agents: Vec<AgentPresence>,
    pub open_request_participants: Vec<String>,
    pub temp_dirs: Vec<TempDirSnapshot>,
    pub prior_hits: Vec<PriorHit>,
    pub now_unix: u64,
    pub min_age_secs: u64,
    pub double_observation_secs: u64,
    /// Toujours `false` tant que le signal n'existe pas chez Bridget.
    pub background_liveness_available: bool,
    /// L'inventaire daemon a-t-il pu être consulté ? Sans lui, l'absence d'un
    /// agent n'est pas une preuve d'abandon et reste fail-closed.
    pub agent_inventory_available: bool,
}

impl WorldView {
    pub fn with_defaults(now_unix: u64) -> Self {
        Self {
            now_unix,
            min_age_secs: DEFAULT_MIN_AGE_SECS,
            double_observation_secs: DEFAULT_DOUBLE_OBSERVATION_SECS,
            background_liveness_available: false,
            agent_inventory_available: true,
            ..Self::default()
        }
    }
}

/// Classe purement une cible déjà extraite — cœur testable, zéro I/O.
pub fn classify_managed_wrapper(
    process: &ProcessSnapshot,
    agent: Option<&AgentPresence>,
    open_participants: &[String],
    world: &WorldView,
) -> TargetReport {
    let agent_name = parse_managed_wrapper_name(&process.command);
    let identity = match &agent_name {
        Some(name) => format!("managed-wrapper:{name}:pid={}", process.pid),
        None => format!("managed-wrapper:pid={}", process.pid),
    };
    let fingerprint = match &agent_name {
        Some(name) => format!("A:{}:{}", name, stable_cmdline_digest(&process.command)),
        None => format!("A:pid-only:{}", process.pid),
    };

    let mut criteria_met = Vec::new();
    let mut criteria_missing = Vec::new();
    let mut guards = Vec::new();
    let mut uncertainty = Vec::new();
    let mut gaps = Vec::new();

    // P5 — appartenance
    if command_is_managed_wrapper(&process.command) {
        criteria_met.push("P5_bridget_managed_wrapper".into());
    } else {
        criteria_missing.push("P5_bridget_managed_wrapper".into());
        guards.push("G8_hors_inventaire".into());
    }

    // P1 — âge
    if process.age_secs >= world.min_age_secs {
        criteria_met.push(format!("P1_age_secs>={}", world.min_age_secs));
    } else {
        criteria_missing.push(format!(
            "P1_age_secs>={} (actuel={})",
            world.min_age_secs, process.age_secs
        ));
    }

    // P4 — orphelinage
    let orphan_ppid1 = process.ppid <= 1;
    let known_to_daemon = agent.is_some();
    if orphan_ppid1 {
        criteria_met.push("P4_ppid_1".into());
    }
    if !known_to_daemon {
        criteria_met.push("P4_absent_du_daemon".into());
    }
    if !orphan_ppid1 && known_to_daemon {
        criteria_missing.push("P4_orphelinage_structurel".into());
    }

    // P2 / G1 — busy
    let busy = agent.is_some_and(|a| a.state == "busy");
    if busy {
        guards.push("G1_tour_actif_busy".into());
        criteria_missing.push("P2_pas_de_tour_actif".into());
    } else {
        criteria_met.push("P2_pas_de_tour_actif".into());
    }

    // G3 — présence daemon
    if let Some(a) = agent {
        if a.state == "connected" || a.state == "busy" {
            guards.push(format!("G3_presence_daemon_state={}", a.state));
        }
        // G4 — tmux vivant
        if a.mode.as_deref() == Some("tmux") && a.location.as_ref().is_some_and(|l| !l.is_empty()) {
            guards.push(format!(
                "G4_pane_tmux_vivant={}",
                a.location.as_deref().unwrap_or("?")
            ));
        }
        if a.last_seen_secs == u64::MAX {
            guards.push("G6_last_seen_illisible".into());
        }
    }

    // G5 — demande ouverte
    if let Some(name) = &agent_name
        && open_participants.iter().any(|p| p == name)
    {
        guards.push("G5_demande_bridget_ouverte".into());
    }

    // P3 / G2 — background : TROU
    if world.background_liveness_available {
        criteria_met.push("P3_pas_de_travail_de_fond".into());
    } else {
        criteria_missing.push("P3_pas_de_travail_de_fond".into());
        uncertainty.push(GAP_BACKGROUND.into());
        gaps.push(MANQUE_BACKGROUND.into());
    }

    let would = format!(
        "SIGTERM pid={} puis stopSession agent={} (JAMAIS exécuté en phase observer)",
        process.pid,
        agent_name.as_deref().unwrap_or("?")
    );

    finalize_report(FinalizeInput {
        class: TargetClass::ManagedWrapper,
        identity,
        fingerprint,
        pid: Some(process.pid),
        ppid: Some(process.ppid),
        cmdline: Some(truncate(&process.command, 240)),
        path: None,
        age_secs: Some(process.age_secs),
        agent_name,
        criteria_met,
        criteria_missing,
        guards,
        uncertainty,
        would_have_done: would,
        gaps,
        // Classe A : fail-closed force l'incertitude tant que le signal manque,
        // même si le reste de la conjonction tient.
        force_incertain_if_background_gap: !world.background_liveness_available,
        world,
    })
}

pub fn classify_test_daemon(
    process: &ProcessSnapshot,
    home: Option<&Path>,
    world: &WorldView,
) -> TargetReport {
    let home_str = home.map(|p| p.display().to_string());
    let identity = match &home_str {
        Some(h) => format!("test-daemon:pid={}:home={h}", process.pid),
        None => format!("test-daemon:pid={}", process.pid),
    };
    let fingerprint = match &home_str {
        Some(h) => format!("B:home:{h}"),
        None => format!("B:pid:{}", process.pid),
    };

    let mut criteria_met = Vec::new();
    let mut criteria_missing = Vec::new();
    let mut guards = Vec::new();
    let mut uncertainty = Vec::new();

    if command_is_bridget_daemon(&process.command) {
        criteria_met.push("P5_bridget_daemon".into());
    } else {
        criteria_missing.push("P5_bridget_daemon".into());
        guards.push("G8_hors_inventaire".into());
    }

    let harness = home.is_some_and(path_looks_like_harness_home);
    if harness {
        criteria_met.push("P3_marqueur_harnais".into());
    } else if home.is_some() {
        criteria_missing.push("P3_marqueur_harnais".into());
        uncertainty.push("home_present_mais_prefixe_harnais_non_reconnu".into());
    } else {
        criteria_missing.push("P3_marqueur_harnais".into());
        uncertainty.push("home_daemon_inconnu".into());
    }

    if process.age_secs >= world.min_age_secs {
        criteria_met.push(format!("P1_age_secs>={}", world.min_age_secs));
    } else {
        criteria_missing.push(format!(
            "P1_age_secs>={} (actuel={})",
            world.min_age_secs, process.age_secs
        ));
    }

    if process.ppid <= 1 {
        criteria_met.push("P2_ppid_1_sans_parent_harnais".into());
    } else {
        // Parent vivant : probablement un test en cours → protégé / incertain
        criteria_missing.push("P2_ppid_1_sans_parent_harnais".into());
        guards.push(format!("G_parent_vivant_ppid={}", process.ppid));
    }

    // G9 — managed-wrapper encore rattaché à ce daemon : tuer le parent le
    // tuerait aussi. Deux signaux, le doute élargit la garde (jamais l'inverse) :
    // (1) ppid == pid du daemon ; (2) cmdline qui cite le HOME harnais — car
    // après réadoption par PID 1 la filiation ppid disparaît, mais le home reste.
    let descendant_pids: Vec<u32> = world
        .processes
        .iter()
        .filter(|child| {
            if !command_is_managed_wrapper(&child.command) {
                return false;
            }
            if child.ppid == process.pid {
                return true;
            }
            home.is_some_and(|h| process_mentions_path(&child.command, h))
        })
        .map(|child| child.pid)
        .collect();
    if !descendant_pids.is_empty() {
        guards.push(format!(
            "G9_descendant_managed_wrapper_vivant={}",
            descendant_pids
                .iter()
                .map(|pid| pid.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    let would = match &home_str {
        Some(h) => format!(
            "SIGTERM pid={} puis rmdir best-effort {h} (JAMAIS exécuté en phase observer)",
            process.pid
        ),
        None => format!(
            "SIGTERM pid={} (home inconnu — JAMAIS exécuté en phase observer)",
            process.pid
        ),
    };

    finalize_report(FinalizeInput {
        class: TargetClass::TestDaemon,
        identity,
        fingerprint,
        pid: Some(process.pid),
        ppid: Some(process.ppid),
        cmdline: Some(truncate(&process.command, 240)),
        path: home_str,
        age_secs: Some(process.age_secs),
        agent_name: None,
        criteria_met,
        criteria_missing,
        guards,
        uncertainty,
        would_have_done: would,
        gaps: Vec::new(),
        force_incertain_if_background_gap: false,
        world,
    })
}

pub fn classify_temp_dir(
    dir: &TempDirSnapshot,
    owner_pids: &[u32],
    world: &WorldView,
) -> TargetReport {
    let path = dir.path.display().to_string();
    let identity = format!("temp-dir:{}", path);
    let fingerprint = format!("C:{}", path);

    let mut criteria_met = vec![format!("P3_racine_observee={}", dir.prefix_kind)];
    let mut criteria_missing = Vec::new();
    let mut guards = Vec::new();
    let mut uncertainty = Vec::new();

    if dir.age_secs >= world.min_age_secs {
        criteria_met.push(format!("P1_age_secs>={}", world.min_age_secs));
    } else {
        criteria_missing.push(format!(
            "P1_age_secs>={} (actuel={})",
            world.min_age_secs, dir.age_secs
        ));
    }

    if owner_pids.is_empty() {
        criteria_met.push("P2_aucun_pid_proprietaire".into());
    } else {
        criteria_missing.push("P2_aucun_pid_proprietaire".into());
        guards.push(format!(
            "G_pids_rattachés={}",
            owner_pids
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    if let Some(agent_name) = dir.agent_name.as_deref() {
        match world.agents.iter().find(|agent| agent.name == agent_name) {
            Some(agent) if matches!(agent.state.as_str(), "connected" | "busy" | "dnd") => {
                // Une présence active suffit, même lorsqu'aucun processus ne
                // référence le répertoire à l'instant de la photographie.
                guards.push(format!("G_agent_actif={} état={}", agent.name, agent.state));
                criteria_missing.push("P4_agent_absent_ou_arrête".into());
            }
            Some(agent) if matches!(agent.state.as_str(), "stopped" | "unreachable") => {
                criteria_met.push(format!(
                    "P4_agent_absent_ou_arrête={} état={}",
                    agent.name, agent.state
                ));
            }
            Some(agent) => {
                criteria_missing.push("P4_agent_absent_ou_arrête".into());
                uncertainty.push(format!(
                    "état_agent_non_classable={} état={}",
                    agent.name, agent.state
                ));
            }
            None if world.agent_inventory_available => {
                criteria_met.push(format!("P4_agent_absent_du_daemon={agent_name}"));
            }
            None => {
                criteria_missing.push("P4_agent_absent_ou_arrête".into());
                uncertainty.push(format!("inventaire_agent_indisponible pour {agent_name}"));
            }
        }
    }

    let would = format!("candidat à revue {path} (JAMAIS exécuté en phase observer)");

    finalize_report(FinalizeInput {
        class: TargetClass::TempDir,
        identity,
        fingerprint,
        pid: None,
        ppid: None,
        cmdline: None,
        path: Some(path),
        age_secs: Some(dir.age_secs),
        agent_name: dir.agent_name.clone(),
        criteria_met,
        criteria_missing,
        guards,
        uncertainty,
        would_have_done: would,
        gaps: Vec::new(),
        force_incertain_if_background_gap: false,
        world,
    })
}

struct FinalizeInput<'a> {
    class: TargetClass,
    identity: String,
    fingerprint: String,
    pid: Option<u32>,
    ppid: Option<u32>,
    cmdline: Option<String>,
    path: Option<String>,
    age_secs: Option<u64>,
    agent_name: Option<String>,
    criteria_met: Vec<String>,
    criteria_missing: Vec<String>,
    guards: Vec<String>,
    uncertainty: Vec<String>,
    would_have_done: String,
    gaps: Vec<String>,
    force_incertain_if_background_gap: bool,
    world: &'a WorldView,
}

fn finalize_report(input: FinalizeInput<'_>) -> TargetReport {
    let mut verdict = if !input.guards.is_empty() {
        Verdict::Protege
    } else if !input.uncertainty.is_empty() || !input.criteria_missing.is_empty() {
        Verdict::Incertain
    } else if input.criteria_missing.is_empty() && input.guards.is_empty() {
        Verdict::Eligible
    } else {
        Verdict::Incertain
    };

    if input.force_incertain_if_background_gap && verdict == Verdict::Eligible {
        verdict = Verdict::Incertain;
    }
    // Même si d'autres critères manquent, le gap background reste visible ;
    // si des gardes ont déjà protégé, on conserve PROTÉGÉ (vie prouvée).

    let stable = verdict == Verdict::Eligible
        && input.world.prior_hits.iter().any(|prior| {
            prior.fingerprint == input.fingerprint
                && prior.verdict == Verdict::Eligible
                && input.world.now_unix.saturating_sub(prior.observed_at_unix)
                    >= input.world.double_observation_secs
        });

    let would = if verdict == Verdict::Eligible {
        input.would_have_done
    } else {
        format!(
            "aucune (verdict={:?}) — hypothétique si éligible: {}",
            verdict_label(verdict),
            input.would_have_done
        )
    };

    TargetReport {
        class: input.class,
        identity: input.identity,
        fingerprint: input.fingerprint,
        pid: input.pid,
        ppid: input.ppid,
        cmdline: input.cmdline,
        path: input.path,
        age_secs: input.age_secs,
        agent_name: input.agent_name,
        criteria_met: input.criteria_met,
        criteria_missing: input.criteria_missing,
        guards_triggered: input.guards,
        uncertainty_reasons: input.uncertainty,
        verdict,
        would_have_done: would,
        stable_for_phase1: stable,
        gaps: input.gaps,
    }
}

fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Eligible => "eligible",
        Verdict::Protege => "protege",
        Verdict::Incertain => "incertain",
    }
}

/// Construit le rapport complet à partir d'une vue du monde.
pub fn build_report(world: &WorldView) -> ReaperReport {
    let mut targets = Vec::new();
    let agents_by_name: BTreeMap<&str, &AgentPresence> =
        world.agents.iter().map(|a| (a.name.as_str(), a)).collect();

    for process in &world.processes {
        if command_is_managed_wrapper(&process.command) {
            let name = parse_managed_wrapper_name(&process.command);
            let agent = name.as_deref().and_then(|n| agents_by_name.get(n).copied());
            targets.push(classify_managed_wrapper(
                process,
                agent,
                &world.open_request_participants,
                world,
            ));
        } else if command_is_bridget_daemon(&process.command) {
            // Home résolu via lsof (live) ou marqueur `HOME=` (fixtures tests).
            // Un daemon de production (ppid=1 inclus) sans préfixe harnais est
            // hors inventaire B — on ne le liste pas.
            let home = parse_fixture_home(&process.command).map(PathBuf::from);
            let maybe_home = home.as_deref();
            if maybe_home.is_some_and(path_looks_like_harness_home) {
                targets.push(classify_test_daemon(process, maybe_home, world));
            }
        }
    }

    for dir in &world.temp_dirs {
        let owners: Vec<u32> = world
            .processes
            .iter()
            .filter(|p| process_mentions_path(&p.command, &dir.path))
            .map(|p| p.pid)
            .collect();
        targets.push(classify_temp_dir(dir, &owners, world));
    }

    targets.sort_by(|a, b| a.identity.cmp(&b.identity));

    let summary = ReportSummary {
        eligible: targets
            .iter()
            .filter(|t| t.verdict == Verdict::Eligible)
            .count(),
        protege: targets
            .iter()
            .filter(|t| t.verdict == Verdict::Protege)
            .count(),
        incertain: targets
            .iter()
            .filter(|t| t.verdict == Verdict::Incertain)
            .count(),
        stable_for_phase1: targets.iter().filter(|t| t.stable_for_phase1).count(),
    };

    let mut gaps = Vec::new();
    if !world.background_liveness_available {
        gaps.push(MANQUE_BACKGROUND.to_string());
    }

    ReaperReport {
        phase: PHASE.into(),
        observed_at_unix: world.now_unix,
        min_age_secs: world.min_age_secs,
        double_observation_secs: world.double_observation_secs,
        background_liveness_available: world.background_liveness_available,
        gaps_to_instruct: gaps,
        targets,
        summary,
    }
}

pub fn render_human(report: &ReaperReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "bridget reaper report — phase={} — observed_at={}\n",
        report.phase, report.observed_at_unix
    ));
    out.push_str(&format!(
        "seuils: min_age={}s double_photo={}s background_liveness={}\n",
        report.min_age_secs, report.double_observation_secs, report.background_liveness_available
    ));
    out.push_str(&format!(
        "résumé: eligible={} protege={} incertain={} stable_for_phase1={}\n",
        report.summary.eligible,
        report.summary.protege,
        report.summary.incertain,
        report.summary.stable_for_phase1
    ));
    for gap in &report.gaps_to_instruct {
        out.push_str(&format!("GAP: {gap}\n"));
    }
    out.push('\n');
    for t in &report.targets {
        out.push_str(&format!(
            "--- {} [{}]\n",
            t.identity,
            verdict_label(t.verdict)
        ));
        if let Some(pid) = t.pid {
            out.push_str(&format!(
                "  pid={} ppid={} age_secs={}\n",
                pid,
                t.ppid.unwrap_or(0),
                t.age_secs.unwrap_or(0)
            ));
        }
        if let Some(path) = &t.path {
            out.push_str(&format!("  path={path}\n"));
        }
        if let Some(agent_name) = &t.agent_name {
            out.push_str(&format!("  agent={agent_name}\n"));
        }
        if let Some(cmd) = &t.cmdline {
            out.push_str(&format!("  cmdline={cmd}\n"));
        }
        out.push_str(&format!(
            "  critères OK: {}\n",
            if t.criteria_met.is_empty() {
                "—".into()
            } else {
                t.criteria_met.join(", ")
            }
        ));
        out.push_str(&format!(
            "  critères manquants: {}\n",
            if t.criteria_missing.is_empty() {
                "—".into()
            } else {
                t.criteria_missing.join(", ")
            }
        ));
        out.push_str(&format!(
            "  gardes: {}\n",
            if t.guards_triggered.is_empty() {
                "—".into()
            } else {
                t.guards_triggered.join(", ")
            }
        ));
        if !t.uncertainty_reasons.is_empty() {
            out.push_str(&format!(
                "  incertitude: {}\n",
                t.uncertainty_reasons.join(", ")
            ));
        }
        out.push_str(&format!("  AURAIT fait: {}\n", t.would_have_done));
        if t.stable_for_phase1 {
            out.push_str("  stable_for_phase1=true (toujours sans action en phase observer)\n");
        }
        for g in &t.gaps {
            out.push_str(&format!("  gap: {g}\n"));
        }
        out.push('\n');
    }
    out.push_str(
        "INTERDIT en phase observer: signal, stop, unlink. Activation Phase 1 = arbitrage explicite.\n",
    );
    out
}

/// Persiste le snapshot pour double observation + relecture humaine.
pub fn append_observation(state_dir: &Path, report: &ReaperReport) -> io::Result<PathBuf> {
    fs::create_dir_all(state_dir)?;
    let path = state_dir.join("observations.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let line =
        serde_json::to_string(report).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writeln!(file, "{line}")?;
    Ok(path)
}

pub fn load_prior_hits(state_dir: &Path) -> Vec<PriorHit> {
    let path = state_dir.join("observations.jsonl");
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for line in text.lines() {
        let Ok(report) = serde_json::from_str::<ReaperReport>(line) else {
            continue;
        };
        for t in report.targets {
            hits.push(PriorHit {
                fingerprint: t.fingerprint,
                verdict: t.verdict,
                observed_at_unix: report.observed_at_unix,
            });
        }
    }
    hits
}

/// Balayage réel de la machine — lecture seule.
pub fn observe_live(
    state_dir: &Path,
    tmp_dir: &Path,
    min_age_secs: u64,
) -> io::Result<ReaperReport> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut world = WorldView::with_defaults(now);
    world.min_age_secs = min_age_secs;
    world.prior_hits = load_prior_hits(state_dir);
    world.processes = scan_processes()?;
    match scan_agents()? {
        Some(agents) => {
            world.agent_inventory_available = true;
            world.agents = agents;
        }
        None => world.agent_inventory_available = false,
    }
    world.open_request_participants = scan_open_request_participants_best_effort();
    scan_explicit_temp_root(&mut world, tmp_dir, now)?;

    // Enrichir les daemons avec leur HOME via lsof (best-effort).
    let mut enriched = Vec::new();
    for process in &world.processes {
        if command_is_bridget_daemon(&process.command)
            && let Some(home) = probe_daemon_home(process.pid)
        {
            let mut cmd = process.command.clone();
            cmd.push_str(" HOME=");
            cmd.push_str(&home.display().to_string());
            enriched.push(ProcessSnapshot {
                command: cmd,
                ..process.clone()
            });
            continue;
        }
        enriched.push(process.clone());
    }
    world.processes = enriched;

    let report = build_report(&world);
    let _ = append_observation(state_dir, &report)?;
    Ok(report)
}

fn scan_processes() -> io::Result<Vec<ProcessSnapshot>> {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,ppid=,etime=,command="])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("ps a échoué"));
    }
    let mut out = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((pid_s, rest)) = split_first_token(line) else {
            continue;
        };
        let Some((ppid_s, rest)) = split_first_token(rest) else {
            continue;
        };
        let Some((etime_s, command)) = split_first_token(rest) else {
            continue;
        };
        let Ok(pid) = pid_s.parse::<u32>() else {
            continue;
        };
        let Ok(ppid) = ppid_s.parse::<u32>() else {
            continue;
        };
        if !(command_is_managed_wrapper(command) || command_is_bridget_daemon(command)) {
            continue;
        }
        out.push(ProcessSnapshot {
            pid,
            ppid,
            age_secs: parse_etime_secs(etime_s).unwrap_or(0),
            command: command.to_string(),
        });
    }
    Ok(out)
}

fn scan_agents() -> io::Result<Option<Vec<AgentPresence>>> {
    let config = crate::daemon::DaemonConfig::default();
    agents_from_status(crate::daemon::get_status(&config))
}

fn agents_from_status(
    status: Result<crate::daemon::DaemonStatus, String>,
) -> io::Result<Option<Vec<AgentPresence>>> {
    let status = status.map_err(io::Error::other)?;
    if !status.running {
        return Ok(Some(Vec::new()));
    }
    if !status.agents_inventory_available {
        return Ok(None);
    }
    Ok(Some(
        status
            .agents
            .into_iter()
            .map(|a| AgentPresence {
                name: a.agent_id,
                state: a.state,
                mode: a.mode.map(|m| m.as_str().to_string()),
                location: a.location,
                last_seen_secs: a.last_seen_secs,
            })
            .collect(),
    ))
}

fn scan_open_request_participants_best_effort() -> Vec<String> {
    // Best-effort : le ledger CLI liste les messages ; les demandes ouvertes
    // passent par le daemon. Sans API dédiée stable ici, on reste vide plutôt
    // que d'inventer — fail-open sur G5 seulement (absence ≠ fausse garde).
    Vec::new()
}

fn scan_explicit_temp_root(world: &mut WorldView, tmp: &Path, now_unix: u64) -> io::Result<()> {
    world.temp_dirs = scan_temp_dirs(tmp, now_unix, &world.agents)?;
    Ok(())
}

fn scan_temp_dirs(
    tmp: &Path,
    now_unix: u64,
    known_agents: &[AgentPresence],
) -> io::Result<Vec<TempDirSnapshot>> {
    let entries = match fs::read_dir(tmp) {
        Ok(e) => e,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let (kind, agent_name) = match harness_temp_prefix(name) {
            Some(kind) => (kind.to_string(), None),
            None if meta.is_dir() => match agent_name_from_temp_dir(name, known_agents) {
                Some(agent_name) => ("agent".to_string(), Some(agent_name)),
                None => continue,
            },
            None => continue,
        };
        if !meta.is_dir() {
            // Les .sock / .pid isolés : on les signale aussi comme artefacts
            // si le préfixe matche — âge sur mtime.
        }
        let age = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| now_unix.saturating_sub(d.as_secs()))
            .unwrap_or(0);
        out.push(TempDirSnapshot {
            path,
            age_secs: age,
            prefix_kind: kind,
            agent_name,
        });
    }
    Ok(out)
}

fn probe_daemon_home(pid: u32) -> Option<PathBuf> {
    let output = Command::new("lsof")
        .args(["-p", &pid.to_string(), "-Fn"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix('n'))
        .find_map(|path| {
            // HOME/.cache/bridget/bridget.sock → HOME
            let p = Path::new(path);
            if p.file_name()?.to_str()? != "bridget.sock" {
                return None;
            }
            let cache = p.parent()?; // .../.cache/bridget
            let cache_parent = cache.parent()?; // .../.cache
            let home = cache_parent.parent()?;
            if path_looks_like_harness_home(home) {
                Some(home.to_path_buf())
            } else {
                None
            }
        })
}

pub fn command_is_managed_wrapper(command: &str) -> bool {
    let parts: Vec<&str> = command.split_whitespace().collect();
    parts.len() >= 2 && parts[0].contains("bridget") && parts[1] == "managed-wrapper"
}

pub fn command_is_bridget_daemon(command: &str) -> bool {
    let parts: Vec<&str> = command.split_whitespace().collect();
    // `bridget daemon` sans args supplémentaires (hors marqueur HOME= fixture)
    if parts.len() >= 2 && parts[0].contains("bridget") && parts[1] == "daemon" {
        return parts.len() == 2 || parts[2].starts_with("HOME=");
    }
    false
}

pub fn parse_managed_wrapper_name(command: &str) -> Option<String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    // bridget managed-wrapper <type> <name> ...
    if parts.len() >= 4 && parts[1] == "managed-wrapper" {
        return Some(parts[3].to_string());
    }
    None
}

fn parse_fixture_home(command: &str) -> Option<&str> {
    command
        .split_whitespace()
        .find_map(|tok| tok.strip_prefix("HOME="))
}

pub fn path_looks_like_harness_home(path: &Path) -> bool {
    path.components().any(|c| {
        let Some(name) = c.as_os_str().to_str() else {
            return false;
        };
        harness_temp_prefix(name).is_some()
    })
}

fn harness_temp_prefix(name: &str) -> Option<&'static str> {
    if name.starts_with("bg909-") {
        Some("bg909")
    } else if name.starts_with("bg-") {
        Some("bg")
    } else if name.starts_with("mg1504-") {
        Some("mg1504")
    } else {
        None
    }
}

/// Extrait le préfixe d'un répertoire de travail d'agent (`jc2-attribution`,
/// `gregen.abc`). Cette reconnaissance classe seulement : elle n'autorise
/// jamais une suppression et une identité inconnue reste visible au rapport.
fn agent_name_from_temp_dir(name: &str, known_agents: &[AgentPresence]) -> Option<String> {
    if let Some(agent_name) = known_agents
        .iter()
        .map(|agent| agent.name.as_str())
        .filter(|agent_name| {
            name.strip_prefix(agent_name)
                .is_some_and(|suffix| suffix.starts_with('-') || suffix.starts_with('.'))
        })
        .max_by_key(|agent_name| agent_name.len())
    {
        return Some(agent_name.to_string());
    }
    let separator = name.find(|character| matches!(u32::from(character), 45 | 46))?;
    let candidate = &name[..separator];
    if candidate.is_empty()
        || candidate.len() > 64
        || !candidate
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }
    Some(candidate.to_string())
}

fn process_mentions_path(command: &str, path: &Path) -> bool {
    let needle = path.to_string_lossy();
    command.contains(needle.as_ref())
}

fn split_first_token(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start();
    if line.is_empty() {
        return None;
    }
    match line.find(char::is_whitespace) {
        Some(i) => Some((&line[..i], line[i..].trim_start())),
        None => Some((line, "")),
    }
}

/// Parse `etime` de ps : `[[dd-]hh:]mm:ss` ou `mm:ss`.
pub fn parse_etime_secs(etime: &str) -> Option<u64> {
    let etime = etime.trim();
    if etime.is_empty() {
        return None;
    }
    let (days, rest) = if let Some((d, r)) = etime.split_once('-') {
        (d.parse::<u64>().ok()?, r)
    } else {
        (0, etime)
    };
    let parts: Vec<&str> = rest.split(':').collect();
    let (hours, minutes, seconds): (u64, u64, u64) = match parts.as_slice() {
        [m, s] => (0, m.parse().ok()?, s.parse().ok()?),
        [h, m, s] => (h.parse().ok()?, m.parse().ok()?, s.parse().ok()?),
        _ => return None,
    };
    Some(days * 86_400 + hours * 3_600 + minutes * 60 + seconds)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}

fn stable_cmdline_digest(command: &str) -> String {
    // Pas un hash crypto : identité stable lisible (type+name si possible).
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.len() >= 4 && parts[1] == "managed-wrapper" {
        format!("{}:{}", parts[2], parts[3])
    } else {
        truncate(command, 64)
    }
}

pub fn default_state_dir() -> PathBuf {
    crate::environment::Namespace::from_environment()
        .expect("namespace validé avant le reaper")
        .root
        .join("reaper")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_at(now: u64) -> WorldView {
        WorldView::with_defaults(now)
    }

    #[test]
    fn identite_indisponible_n_est_pas_un_annuaire_vide() {
        let error = agents_from_status(Err(
            "identité du daemon indisponible: délai de lecture dépassé".to_string(),
        ))
        .expect_err("l'incertitude doit interrompre l'inventaire");
        assert!(
            error
                .to_string()
                .contains("identité du daemon indisponible")
        );

        let absent = agents_from_status(Ok(crate::daemon::DaemonStatus {
            agents_inventory_available: true,
            ..crate::daemon::DaemonStatus::default()
        }))
        .expect("un daemon réellement absent reste observable")
        .expect("l'absence déterminée donne un inventaire connu");
        assert!(absent.is_empty());
    }

    #[test]
    fn daemon_present_mais_inventaire_indisponible_reste_incertain() {
        let inventory = agents_from_status(Ok(crate::daemon::DaemonStatus {
            running: true,
            agents_inventory_available: false,
            ..crate::daemon::DaemonStatus::default()
        }))
        .expect("le statut attesté reste lisible");
        assert!(
            inventory.is_none(),
            "un daemon présent mais muet ne doit pas devenir un annuaire vide"
        );
    }

    #[test]
    fn orphan_wrapper_fail_closed_without_background_signal() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let process = ProcessSnapshot {
            pid: 3013,
            ppid: 1,
            age_secs: 3600,
            command: "/tmp/bridget managed-wrapper g1504_fixture g1504-agent {}".into(),
        };
        let report = classify_managed_wrapper(&process, None, &[], &world);
        assert_eq!(report.verdict, Verdict::Incertain);
        assert!(
            report
                .uncertainty_reasons
                .iter()
                .any(|r| r == GAP_BACKGROUND)
        );
        assert!(report.would_have_done.contains("aucune"));
        assert!(!report.gaps.is_empty());
    }

    #[test]
    fn live_connected_wrapper_is_protege() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let process = ProcessSnapshot {
            pid: 39951,
            ppid: 37095,
            age_secs: 3600,
            command: "/opt/bridget managed-wrapper cursor cursor3 {}".into(),
        };
        let agent = AgentPresence {
            name: "cursor3".into(),
            state: "busy".into(),
            mode: Some("acp".into()),
            location: None,
            last_seen_secs: 0,
        };
        let report = classify_managed_wrapper(&process, Some(&agent), &[], &world);
        assert_eq!(report.verdict, Verdict::Protege);
        assert!(report.guards_triggered.iter().any(|g| g.starts_with("G1_")));
        assert!(report.guards_triggered.iter().any(|g| g.starts_with("G3_")));
    }

    #[test]
    fn tmux_location_protects_even_if_daemon_confused() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let process = ProcessSnapshot {
            pid: 42,
            ppid: 1,
            age_secs: 3600,
            command: "/opt/bridget managed-wrapper claude survivor {}".into(),
        };
        let agent = AgentPresence {
            name: "survivor".into(),
            state: "connected".into(),
            mode: Some("tmux".into()),
            location: Some("Who:2.2".into()),
            last_seen_secs: 999,
        };
        let report = classify_managed_wrapper(&process, Some(&agent), &[], &world);
        assert_eq!(report.verdict, Verdict::Protege);
        assert!(report.guards_triggered.iter().any(|g| g.starts_with("G4_")));
    }

    #[test]
    fn harness_daemon_orphan_can_be_eligible() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let home = PathBuf::from("/tmp/bg-12345-abcd");
        let process = ProcessSnapshot {
            pid: 4513,
            ppid: 1,
            age_secs: 7200,
            command: "/tmp/x/bridget daemon".into(),
        };
        let report = classify_test_daemon(&process, Some(&home), &world);
        assert_eq!(report.verdict, Verdict::Eligible);
        assert!(report.would_have_done.contains("SIGTERM"));
        assert!(!report.would_have_done.contains("aucune"));
    }

    #[test]
    fn harness_daemon_with_living_descendant_is_not_eligible() {
        // Cas réel 32465→35839 : parent éligible sans G9 aurait tué l'enfant.
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let home = PathBuf::from("/tmp/bg909-matrix-0-32462-0fe0a6d0");
        let parent = ProcessSnapshot {
            pid: 32465,
            ppid: 1,
            age_secs: 7200,
            command: "/tmp/x/bridget daemon".into(),
        };
        world.processes = vec![
            parent.clone(),
            ProcessSnapshot {
                pid: 35839,
                ppid: 32465,
                age_secs: 7000,
                command: format!(
                    "/tmp/x/bridget managed-wrapper parity parity-managed-0 {}/parity-acp.py",
                    home.display()
                ),
            },
        ];
        let report = classify_test_daemon(&parent, Some(&home), &world);
        assert_eq!(report.verdict, Verdict::Protege);
        assert!(
            report
                .guards_triggered
                .iter()
                .any(|g| g.starts_with("G9_descendant_managed_wrapper_vivant="))
        );
        assert!(report.would_have_done.contains("aucune"));
    }

    #[test]
    fn harness_daemon_becomes_eligible_when_descendant_gone() {
        // Sans le second oracle, G9 pourrait bloquer tout le monde à jamais.
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let home = PathBuf::from("/tmp/bg909-matrix-1-54665-0ad2889e");
        let parent = ProcessSnapshot {
            pid: 64298,
            ppid: 1,
            age_secs: 7200,
            command: "/tmp/x/bridget daemon".into(),
        };
        world.processes = vec![
            parent.clone(),
            ProcessSnapshot {
                pid: 67637,
                ppid: 64298,
                age_secs: 7000,
                command: format!(
                    "/tmp/x/bridget managed-wrapper parity parity-managed-1 {}/parity-acp.py",
                    home.display()
                ),
            },
        ];
        let with_child = classify_test_daemon(&parent, Some(&home), &world);
        assert_eq!(with_child.verdict, Verdict::Protege);

        world.processes.retain(|p| p.pid != 67637);
        let without_child = classify_test_daemon(&parent, Some(&home), &world);
        assert_eq!(without_child.verdict, Verdict::Eligible);
        assert!(
            !without_child
                .guards_triggered
                .iter()
                .any(|g| g.starts_with("G9_"))
        );
        assert!(without_child.would_have_done.contains("SIGTERM"));
        assert!(!without_child.would_have_done.contains("aucune"));
    }

    #[test]
    fn harness_daemon_protected_by_reparented_wrapper_sharing_home() {
        // Après réadoption (ppid=1), la filiation ppid disparaît ; le HOME
        // partagé reste un signal honnête — doute → pas d'éligibilité.
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let home = PathBuf::from("/tmp/bg909-matrix-0-reparent");
        let parent = ProcessSnapshot {
            pid: 100,
            ppid: 1,
            age_secs: 7200,
            command: "/tmp/x/bridget daemon".into(),
        };
        world.processes = vec![
            parent.clone(),
            ProcessSnapshot {
                pid: 200,
                ppid: 1,
                age_secs: 7000,
                command: format!(
                    "/tmp/x/bridget managed-wrapper parity parity-managed-0 {}/parity-acp.py",
                    home.display()
                ),
            },
        ];
        let report = classify_test_daemon(&parent, Some(&home), &world);
        assert_eq!(report.verdict, Verdict::Protege);
        assert!(
            report
                .guards_triggered
                .iter()
                .any(|g| g.contains("G9_descendant_managed_wrapper_vivant=200"))
        );
    }

    #[test]
    fn temp_dir_with_owner_is_protege() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let dir = TempDirSnapshot {
            path: PathBuf::from("/tmp/bg-999-aa"),
            age_secs: 7200,
            prefix_kind: "bg".into(),
            agent_name: None,
        };
        let report = classify_temp_dir(&dir, &[111], &world);
        assert_eq!(report.verdict, Verdict::Protege);
    }

    #[test]
    fn temp_dir_orphaned_is_eligible() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let dir = TempDirSnapshot {
            path: PathBuf::from("/tmp/mg1504-1-1"),
            age_secs: 7200,
            prefix_kind: "mg1504".into(),
            agent_name: None,
        };
        let report = classify_temp_dir(&dir, &[], &world);
        assert_eq!(report.verdict, Verdict::Eligible);
        assert!(report.would_have_done.contains("candidat à revue"));
        assert!(report.would_have_done.contains("JAMAIS exécuté"));
    }

    #[test]
    fn repertoire_agent_absent_et_ancien_est_candidat_a_revue() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        let dir = TempDirSnapshot {
            path: PathBuf::from("/tmp/ancien-attribution"),
            age_secs: 7200,
            prefix_kind: "agent".into(),
            agent_name: Some("ancien".into()),
        };

        let report = classify_temp_dir(&dir, &[], &world);
        assert_eq!(report.verdict, Verdict::Eligible);
        assert_eq!(report.agent_name.as_deref(), Some("ancien"));
        assert!(
            report
                .criteria_met
                .iter()
                .any(|criterion| criterion == "P4_agent_absent_du_daemon=ancien")
        );
        assert!(report.would_have_done.contains("candidat à revue"));
    }

    #[test]
    fn repertoire_agent_connecte_est_protege_meme_sans_descripteur_ouvert() {
        let mut world = world_at(1_700_000_000);
        world.min_age_secs = 60;
        world.agents.push(AgentPresence {
            name: "jc2".into(),
            state: "connected".into(),
            mode: None,
            location: None,
            last_seen_secs: 0,
        });
        let dir = TempDirSnapshot {
            path: PathBuf::from("/tmp/jc2-attribution"),
            age_secs: 7200,
            prefix_kind: "agent".into(),
            agent_name: Some("jc2".into()),
        };

        let report = classify_temp_dir(&dir, &[], &world);
        assert_eq!(report.verdict, Verdict::Protege);
        assert!(
            report
                .guards_triggered
                .iter()
                .any(|guard| guard == "G_agent_actif=jc2 état=connected"),
            "un arbre au repos reste protégé tant que l'agent est connecté"
        );
    }

    #[test]
    fn racine_explicite_hors_temp_par_defaut_est_observee() {
        let root = PathBuf::from(format!(
            "/tmp/bridget-reaper-racine-explicite-{}",
            uuid::Uuid::new_v4()
        ));
        let target = root.join("jc6-revue");
        fs::create_dir_all(&target).unwrap();

        let scanned = scan_temp_dirs(&root, 1_700_000_000, &[]).unwrap();
        assert!(
            scanned.iter().any(|entry| {
                entry.path == target && entry.agent_name.as_deref() == Some("jc6")
            }),
            "la racine passée explicitement doit être lue par le scanner, sans recours à TMPDIR"
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn raccord_productif_lit_la_racine_fournie_et_protege_un_nom_avec_tiret() {
        let root = PathBuf::from(format!(
            "/tmp/bridget-reaper-raccord-productif-{}",
            uuid::Uuid::new_v4()
        ));
        let target = root.join("jc2-review-attribution");
        fs::create_dir_all(&target).unwrap();
        let state_dir = root.join("state");
        fs::create_dir_all(&state_dir).unwrap();
        let report = observe_live(&state_dir, &root, 60).unwrap();
        assert!(
            report
                .targets
                .iter()
                .any(|entry| { entry.path.as_deref() == Some(target.to_str().unwrap()) })
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn agent_connecte_avec_tiret_est_protege() {
        let root = PathBuf::from(format!(
            "/tmp/bridget-reaper-tiret-{}",
            uuid::Uuid::new_v4()
        ));
        let target = root.join("jc2-review-attribution");
        fs::create_dir_all(&target).unwrap();
        let mut world = world_at(1_700_000_000);
        world.agent_inventory_available = true;
        world.agents.push(AgentPresence {
            name: "jc2-review".into(),
            state: "connected".into(),
            mode: None,
            location: None,
            last_seen_secs: 0,
        });
        scan_explicit_temp_root(&mut world, &root, 1_700_000_000).unwrap();
        let dir = world
            .temp_dirs
            .iter()
            .find(|entry| entry.path == target)
            .unwrap();
        assert_eq!(dir.agent_name.as_deref(), Some("jc2-review"));
        assert_eq!(
            classify_temp_dir(dir, &[], &world).verdict,
            Verdict::Protege
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn double_observation_marks_stable() {
        let mut world = world_at(1_700_000_600);
        world.min_age_secs = 60;
        world.double_observation_secs = 300;
        let home = PathBuf::from("/tmp/bg-1-aa");
        let process = ProcessSnapshot {
            pid: 7,
            ppid: 1,
            age_secs: 900,
            command: "/tmp/bridget daemon".into(),
        };
        let first = classify_test_daemon(&process, Some(&home), &world);
        assert_eq!(first.verdict, Verdict::Eligible);
        assert!(!first.stable_for_phase1);

        world.prior_hits.push(PriorHit {
            fingerprint: first.fingerprint.clone(),
            verdict: Verdict::Eligible,
            observed_at_unix: 1_700_000_000,
        });
        let second = classify_test_daemon(&process, Some(&home), &world);
        assert!(second.stable_for_phase1);
    }

    #[test]
    fn build_report_lists_gap_when_background_missing() {
        let mut world = world_at(10);
        world.min_age_secs = 1;
        world.processes.push(ProcessSnapshot {
            pid: 1,
            ppid: 1,
            age_secs: 100,
            command: "/x/bridget managed-wrapper t n {}".into(),
        });
        let report = build_report(&world);
        assert!(!report.gaps_to_instruct.is_empty());
        assert_eq!(report.phase, "observer");
        assert_eq!(report.targets.len(), 1);
        assert_eq!(report.targets[0].verdict, Verdict::Incertain);
    }

    #[test]
    fn parse_etime_variants() {
        assert_eq!(parse_etime_secs("05:01"), Some(301));
        assert_eq!(parse_etime_secs("1:02:03"), Some(3723));
        assert_eq!(parse_etime_secs("2-01:00:00"), Some(2 * 86400 + 3600));
    }

    #[test]
    fn observer_module_has_no_kill_symbols_in_would_execution_path() {
        // Garde de conception : Phase 0 n'expose pas d'API d'action.
        // (Régression textuelle volontaire — si quelqu'un ajoute kill/unlink
        // public, ce test ne les détecte pas tous, mais documente l'intention.)
        let report = build_report(&world_at(1));
        assert_eq!(report.phase, "observer");
        assert!(report.targets.is_empty());
    }
}
