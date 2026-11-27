//! Orchestration autonome de graphes orientés acycliques (DAG) de missions Bridget.
//!
//! Permet à un chef d'orchestre de déclarer un enchaînement (`A -> B + C -> D`) en un seul
//! appel (`bridget_delegate_graph`) puis de s'endormir sans ronde de garde périodique :
//! le daemon suit l'état durable de chaque étape, déclenche automatiquement les étapes
//! suivantes en parallèle dès que leurs dépendances sont terminées, propage les échecs selon
//! `on_failure`, et réveille le parent une seule fois avec le bilan consolidé.

use super::*;
use crate::delegation::{
    NativeGraphConvergenceIteration, NativeGraphConvergenceState, NativeGraphRecord,
    NativeGraphStepRecord,
};
use bridget_transport::protocol::{
    AutoDelegateV2, DelegateGraphRequest, GraphFailurePolicy, GraphNotifyPolicy,
    MAX_CONVERGENCE_ITERATIONS, MIN_CONVERGENCE_ITERATIONS, NativeDelegationRequest,
    ProviderPermissions, WorkspaceRequest,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) const MAX_GRAPH_STEPS: usize = 16;
const MAX_STEP_ID_LEN: usize = 64;
const MAX_DEP_EXCERPT_CHARS: usize = 1200;
const MAX_TASK_BYTES: usize = 65_536;
const CONVERGENCE_KEEP_ALIVE_SECS: u64 = 900;
const MAX_KEEP_ALIVE_SECS: u64 = 7200;

fn valid_step_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_STEP_ID_LEN
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn step_is_terminal(state: &str) -> bool {
    matches!(
        state,
        "completed" | "failed" | "cancelled" | "skipped_dependency_failed"
    )
}

/// Valide la structure DAG (1..=16 étapes, `step_id` uniques, dépendances existantes,
/// absence de cycles via tri topologique de Kahn) et renvoie l'ordre topologique déterministe.
pub(crate) fn validate_and_order_graph(
    request: &DelegateGraphRequest,
) -> Result<Vec<String>, (String, String)> {
    if request.request_id.trim().is_empty() || request.request_id.len() > 128 {
        return Err((
            "graph_invalid".into(),
            "request_id doit contenir entre 1 et 128 caractères.".into(),
        ));
    }
    if request.steps.is_empty() || request.steps.len() > MAX_GRAPH_STEPS {
        return Err((
            "graph_invalid".into(),
            format!("steps doit contenir entre 1 et {MAX_GRAPH_STEPS} étapes."),
        ));
    }

    let mut seen = BTreeSet::new();
    for step in &request.steps {
        if !valid_step_id(&step.step_id) {
            return Err((
                "graph_invalid".into(),
                format!(
                    "step_id invalide '{}' : 1 à {MAX_STEP_ID_LEN} caractères [A-Za-z0-9_.-].",
                    step.step_id
                ),
            ));
        }
        if !seen.insert(step.step_id.as_str()) {
            return Err((
                "graph_invalid".into(),
                format!("step_id dupliqué dans le graphe : '{}'.", step.step_id),
            ));
        }
        if step.delegation.task.trim().is_empty() || step.delegation.task.len() > MAX_TASK_BYTES {
            return Err((
                "graph_invalid".into(),
                format!(
                    "étape '{}' : la consigne task est vide ou dépasse {MAX_TASK_BYTES} octets.",
                    step.step_id
                ),
            ));
        }
        if step.delegation.cwd.trim().is_empty() {
            return Err((
                "graph_invalid".into(),
                format!("étape '{}' : cwd est obligatoire.", step.step_id),
            ));
        }
        if let Some(conv) = &step.convergence {
            if !(MIN_CONVERGENCE_ITERATIONS..=MAX_CONVERGENCE_ITERATIONS)
                .contains(&conv.max_iterations)
            {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : convergence.max_iterations doit être compris entre {} et {}.",
                        step.step_id, MIN_CONVERGENCE_ITERATIONS, MAX_CONVERGENCE_ITERATIONS
                    ),
                ));
            }
            if conv.verify.task.trim().is_empty() || conv.verify.task.len() > MAX_TASK_BYTES {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : convergence.verify.task est vide ou dépasse {MAX_TASK_BYTES} octets.",
                        step.step_id
                    ),
                ));
            }
            if conv.verify.cwd.trim().is_empty() {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : convergence.verify.cwd est obligatoire.",
                        step.step_id
                    ),
                ));
            }
            if let Some(sha) = conv.verify.expected_base_commit.as_deref()
                && (sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()))
            {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : convergence.verify.expected_base_commit exige un SHA complet de 40 caractères hexadécimaux.",
                        step.step_id
                    ),
                ));
            }
        }
    }

    let mut indegree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for step in &request.steps {
        indegree.insert(step.step_id.as_str(), 0);
        dependents.insert(step.step_id.as_str(), Vec::new());
    }
    for step in &request.steps {
        let mut step_deps = BTreeSet::new();
        for dep in &step.depends_on {
            if dep == &step.step_id {
                return Err((
                    "graph_cycle_detected".into(),
                    format!("auto-dépendance interdite sur l'étape '{}'.", step.step_id),
                ));
            }
            if !seen.contains(dep.as_str()) {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' dépend d'une étape inconnue '{}'.",
                        step.step_id, dep
                    ),
                ));
            }
            if step_deps.insert(dep.as_str()) {
                *indegree.get_mut(step.step_id.as_str()).unwrap() += 1;
                dependents
                    .get_mut(dep.as_str())
                    .unwrap()
                    .push(step.step_id.as_str());
            }
        }
    }

    let mut ready: VecDeque<&str> = indegree
        .iter()
        .filter_map(|(&id, &deg)| (deg == 0).then_some(id))
        .collect();
    let mut order = Vec::with_capacity(request.steps.len());
    while let Some(node) = ready.pop_front() {
        order.push(node.to_string());
        if let Some(children) = dependents.get(node) {
            for &child in children {
                let deg = indegree.get_mut(child).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    ready.push_back(child);
                }
            }
        }
    }

    if order.len() != request.steps.len() {
        let cyclic: Vec<&str> = indegree
            .into_iter()
            .filter_map(|(id, deg)| (deg > 0).then_some(id))
            .collect();
        return Err((
            "graph_cycle_detected".into(),
            format!(
                "Cycle détecté dans le graphe sur les étapes : {}.",
                cyclic.join(", ")
            ),
        ));
    }

    Ok(order)
}

/// Dérive la `request_id` déterministe d'une étape à partir de la clé du graphe si l'étape
/// n'en fournit pas ou reprend la clé brute.
pub(crate) fn step_request_id(graph_request_id: &str, step_id: &str, explicit: &str) -> String {
    let raw = if explicit.trim().is_empty() {
        format!("{graph_request_id}:{step_id}")
    } else {
        explicit.trim().to_string()
    };
    if raw.len() <= 128 {
        return raw;
    }
    let suffix = format!(":{step_id}");
    let prefix_len = 128usize.saturating_sub(suffix.len());
    let prefix: String = raw.chars().take(prefix_len).collect();
    format!("{prefix}{suffix}")
}

/// Clé déterministe et bornée (`<= 128` octets) pour chaque sous-tâche d'itération
/// (`produce` ou `verify`) d'une étape de convergence bornée.
pub(crate) fn convergence_step_request_id(
    graph_request_id: &str,
    step_id: &str,
    iteration: u8,
    role: &str,
) -> String {
    let raw = format!("{graph_request_id}:{step_id}:i{iteration}:{role}");
    if raw.len() <= 128 {
        return raw;
    }
    let digest = format!("{:x}", Sha256::digest(raw.as_bytes()));
    let suffix = format!(":i{iteration}:{role}:{}", &digest[..12]);
    let prefix_len = 128usize.saturating_sub(suffix.len());
    let prefix: String = raw.chars().take(prefix_len).collect();
    format!("{prefix}{suffix}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedVerdict {
    Approve {
        reason: String,
    },
    Revise {
        reason: String,
        blocking_points: Vec<String>,
        fingerprint: String,
    },
    Escalate {
        reason: String,
    },
    Unparseable,
}

fn normalize_text_for_fingerprint(value: &str) -> String {
    value
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

fn blocking_fingerprint(reason: &str, blocking_points: &[String]) -> String {
    let mut normalized: Vec<String> = blocking_points
        .iter()
        .map(|item| normalize_text_for_fingerprint(item))
        .filter(|item| !item.is_empty())
        .collect();
    normalized.sort();
    normalized.dedup();
    let payload = if normalized.is_empty() {
        normalize_text_for_fingerprint(reason)
    } else {
        normalized.join("\n")
    };
    format!("{:x}", Sha256::digest(payload.as_bytes()))
}

fn try_parse_verdict_json(candidate: &str) -> Option<ParsedVerdict> {
    let value: Value = serde_json::from_str(candidate.trim()).ok()?;
    let obj = value.as_object()?;
    let verdict = obj.get("verdict").and_then(Value::as_str)?.trim();
    let reason = obj
        .get("reason")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("motif non précisé")
        .chars()
        .take(512)
        .collect::<String>();
    match verdict {
        "approve" => Some(ParsedVerdict::Approve { reason }),
        "escalate" => Some(ParsedVerdict::Escalate { reason }),
        "revise" => {
            let blocking_points: Vec<String> = obj
                .get("blocking_points")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .take(16)
                        .map(|s| s.chars().take(400).collect())
                        .collect()
                })
                .unwrap_or_default();
            let fingerprint = blocking_fingerprint(&reason, &blocking_points);
            Some(ParsedVerdict::Revise {
                reason,
                blocking_points,
                fingerprint,
            })
        }
        _ => None,
    }
}

/// Extrait en mode fail-closed le dernier objet JSON fermé contenant `"verdict"`
/// (`approve` | `revise` | `escalate`) depuis la sortie du vérificateur.
/// Tolère les blocs Markdown ```` ```json ```` entourant la ligne JSON finale.
/// Complexité O(L) sur la longueur de la sortie.
pub(crate) fn parse_convergence_verdict(text: &str) -> ParsedVerdict {
    // 1) Parcours des lignes depuis la fin : cas nominal (une ligne JSON autonome).
    for line in text.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("```") {
            continue;
        }
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Some(parsed) = try_parse_verdict_json(trimmed) {
                return parsed;
            }
        }
    }
    // 2) Repli borné : dernier objet JSON multi-lignes entre `{` et `}` en fin de réponse.
    if let Some(end) = text.rfind('}') {
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;
        let bytes = text.as_bytes();
        // Parcourir dans l'ordre sur les 8 derniers Kio au plus pour rester O(1) mémoire.
        let window_start = text.floor_char_boundary(end.saturating_sub(8192));
        let mut starts: Vec<usize> = Vec::new();
        for (offset, ch) in text[window_start..=end].char_indices() {
            let idx = window_start + offset;
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }
            match ch {
                '"' => in_string = true,
                '{' => {
                    if depth == 0 {
                        starts.push(idx);
                    }
                    depth += 1;
                }
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0
                        && let Some(start) = starts.last().copied()
                        && idx == end
                        && bytes.get(start) == Some(&b'{')
                    {
                        if let Some(parsed) = try_parse_verdict_json(&text[start..=end]) {
                            return parsed;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    ParsedVerdict::Unparseable
}

fn is_valid_git_sha40(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn apply_commit_to_delegation(delegation: &mut AutoDelegateV2, commit: Option<&str>) {
    let Some(commit) = commit.filter(|sha| is_valid_git_sha40(sha)) else {
        return;
    };
    match &mut delegation.workspace {
        Some(WorkspaceRequest::CommitView {}) => {
            delegation.expected_base_commit = Some(commit.to_string());
        }
        Some(WorkspaceRequest::Worktree { base_ref, .. }) => {
            delegation.expected_base_commit = Some(commit.to_string());
            *base_ref = Some(commit.to_string());
        }
        _ => {
            if delegation.expected_base_commit.is_some() {
                delegation.expected_base_commit = Some(commit.to_string());
            }
        }
    }
}

fn ensure_convergence_producer_keep_alive(delegation: &mut AutoDelegateV2) {
    let current = delegation
        .keep_alive_secs
        .unwrap_or(CONVERGENCE_KEEP_ALIVE_SECS);
    delegation.keep_alive_secs =
        Some(current.clamp(CONVERGENCE_KEEP_ALIVE_SECS, MAX_KEEP_ALIVE_SECS));
}

fn build_verifier_task(
    step: &NativeGraphStepRecord,
    verify: &AutoDelegateV2,
    iteration: u8,
    max_iterations: u8,
) -> String {
    let mut header = format!(
        "## Vérification de convergence — étape `{}` (itération {}/{})\n",
        step.step_id, iteration, max_iterations
    );
    if let Some(cs) = step.convergence_state.as_ref() {
        if let Some(tid) = cs.producer_task_id.as_deref() {
            header.push_str(&format!("- Producteur : task_id={tid}"));
            if let Some(commit) = cs.last_producer_commit.as_deref() {
                header.push_str(&format!(", commit={commit}"));
            }
            header.push('\n');
        }
    }
    if let Some(fs) = step.factual_summary.as_ref()
        && let Some(files) = fs.get("files_changed").and_then(Value::as_u64)
    {
        header.push_str(&format!("- Fichiers modifiés déclarés : {files}\n"));
    }
    if let Some(excerpt) = step.result_excerpt.as_deref()
        && !excerpt.trim().is_empty()
    {
        let clipped: String = excerpt.trim().chars().take(MAX_DEP_EXCERPT_CHARS).collect();
        header.push_str(&format!("- Extrait producteur : {clipped}\n"));
    }
    let contract = "\n## Contrat de verdict fermé (obligatoire en fin de réponse)\n\
Termine ta réponse par UNE ligne JSON autonome respectant exactement l'un des trois schémas suivants :\n\
{\"verdict\":\"approve\",\"reason\":\"...\"}\n\
{\"verdict\":\"revise\",\"reason\":\"...\",\"blocking_points\":[\"...\",\"...\"]}\n\
{\"verdict\":\"escalate\",\"reason\":\"...\"}\n\
- `approve` : livrable conforme, clôt la boucle et débloque les étapes suivantes.\n\
- `revise` : défauts corrigeables dans le périmètre ; le producteur reprendra avec tes `blocking_points`.\n\
- `escalate` : blocage hors périmètre, ambiguïté produit ou arbitrage humain requis ; arrêt immédiat vers le parent.\n";
    let body = format!(
        "{header}\n## Consigne de vérification\n{}{contract}",
        verify.task
    );
    if body.len() <= MAX_TASK_BYTES {
        body
    } else {
        let suffix = format!("\n## Consigne de vérification\n{}{contract}", verify.task);
        let budget = MAX_TASK_BYTES.saturating_sub(suffix.len() + 16);
        let prefix: String = header.chars().take(budget).collect();
        format!("{prefix}\n...{suffix}")
    }
}

fn build_producer_revision_task(
    step_id: &str,
    iteration: u8,
    max_iterations: u8,
    reason: &str,
    blocking_points: &[String],
    original_task: &str,
    for_fallback_new_task: bool,
) -> String {
    let mut out = format!(
        "## Reprise corrective — étape `{step_id}` (itération {iteration}/{max_iterations})\n\
Le vérificateur indépendant a demandé une correction (`verdict=revise`).\n\
Motif : {reason}\n"
    );
    if !blocking_points.is_empty() {
        out.push_str("Points bloquants à corriger :\n");
        for point in blocking_points {
            out.push_str(&format!("- {point}\n"));
        }
    }
    out.push_str(
        "Corrige précisément ces points bloquants, vérifie par des tests ciblés et produis un nouveau commit avant de rendre ton bilan.\n",
    );
    if for_fallback_new_task {
        out.push_str("\n## Rappel de la consigne initiale\n");
        out.push_str(original_task);
    }
    if out.len() <= MAX_TASK_BYTES {
        out
    } else {
        out.chars().take(MAX_TASK_BYTES).collect()
    }
}

/// Construit la consigne finale d'une étape dépendante en y préfixant un contexte court et
/// factuel des résultats de ses dépendances terminées (borné à `MAX_TASK_BYTES`).
pub(crate) fn build_enriched_step_task(
    step: &NativeGraphStepRecord,
    all_steps: &[NativeGraphStepRecord],
) -> String {
    if !step.pass_dependency_results || step.depends_on.is_empty() {
        return step.delegation.task.clone();
    }
    let by_id: BTreeMap<&str, &NativeGraphStepRecord> =
        all_steps.iter().map(|s| (s.step_id.as_str(), s)).collect();
    let mut header = String::from("## Résultats des étapes préalables du graphe Bridget\n");
    for dep_id in &step.depends_on {
        let Some(dep) = by_id.get(dep_id.as_str()) else {
            continue;
        };
        header.push_str(&format!("- Étape `{}` : état={}", dep.step_id, dep.state));
        if let Some(tid) = dep.task_id.as_deref() {
            header.push_str(&format!(", task_id={tid}"));
        }
        if let Some(fs) = dep.factual_summary.as_ref() {
            if let Some(commit) = fs.get("commit").and_then(Value::as_str) {
                header.push_str(&format!(", commit={commit}"));
            }
            if let Some(files) = fs.get("files_changed").and_then(Value::as_u64) {
                header.push_str(&format!(", fichiers_modifies={files}"));
            }
        }
        header.push('\n');
        if let Some(excerpt) = dep.result_excerpt.as_deref()
            && !excerpt.trim().is_empty()
        {
            let clipped: String = excerpt.trim().chars().take(MAX_DEP_EXCERPT_CHARS).collect();
            header.push_str(&format!("  Extrait : {clipped}\n"));
        }
    }
    header.push_str("\n## Consigne de cette étape\n");
    let combined = format!("{header}{}", step.delegation.task);
    if combined.len() <= MAX_TASK_BYTES {
        combined
    } else {
        // Ne jamais tronquer la consigne métier de l'étape : raccourcir uniquement le préambule.
        let budget = MAX_TASK_BYTES.saturating_sub(step.delegation.task.len() + 64);
        if budget < 64 {
            step.delegation.task.clone()
        } else {
            let mut trimmed_header = String::new();
            for ch in header.chars() {
                if trimmed_header.len() + ch.len_utf8() > budget {
                    break;
                }
                trimmed_header.push(ch);
            }
            format!(
                "{trimmed_header}\n...\n\n## Consigne de cette étape\n{}",
                step.delegation.task
            )
        }
    }
}

/// Propage les échecs/annulations sur les étapes du graphe et renvoie la liste des `task_id`
/// d'étapes actives à annuler immédiatement si `on_failure == CancelGraph`.
pub(crate) fn propagate_graph_state(graph: &mut NativeGraphRecord, now: i64) -> Vec<String> {
    let mut to_cancel_tasks = Vec::new();
    let any_failed = graph
        .steps
        .iter()
        .any(|s| matches!(s.state.as_str(), "failed" | "cancelled"));

    if any_failed && graph.on_failure == GraphFailurePolicy::CancelGraph {
        for step in &mut graph.steps {
            match step.state.as_str() {
                "waiting_dependencies" | "launching" => {
                    step.state = "cancelled".into();
                    step.finished_at.get_or_insert(now);
                    if step.error.is_none() {
                        step.error = Some("graph_cancelled_on_failure".into());
                    }
                }
                "running" => {
                    if let Some(tid) = step.task_id.clone() {
                        to_cancel_tasks.push(tid);
                    }
                    if let Some(cs) = step.convergence_state.as_ref() {
                        if let Some(ptid) = cs.producer_task_id.as_ref()
                            && !to_cancel_tasks.contains(ptid)
                        {
                            to_cancel_tasks.push(ptid.clone());
                        }
                        if let Some(vtid) = cs.verifier_task_id.as_ref()
                            && !to_cancel_tasks.contains(vtid)
                        {
                            to_cancel_tasks.push(vtid.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        // Point fixe pour propager `skipped_dependency_failed` sur toute la descendance transitive.
        loop {
            let failed_ids: BTreeSet<String> = graph
                .steps
                .iter()
                .filter(|s| {
                    matches!(
                        s.state.as_str(),
                        "failed" | "cancelled" | "skipped_dependency_failed"
                    )
                })
                .map(|s| s.step_id.clone())
                .collect();
            let mut changed = false;
            for step in &mut graph.steps {
                if step.state == "waiting_dependencies"
                    && let Some(bad_dep) = step.depends_on.iter().find(|d| failed_ids.contains(*d))
                {
                    step.state = "skipped_dependency_failed".into();
                    step.error = Some(format!("dependency_failed:{bad_dep}"));
                    step.finished_at = Some(now);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    if graph.steps.iter().all(|s| step_is_terminal(&s.state)) {
        let all_completed = graph.steps.iter().all(|s| s.state == "completed");
        let any_completed = graph.steps.iter().any(|s| s.state == "completed");
        let all_cancelled = graph.steps.iter().all(|s| s.state == "cancelled");
        graph.state = if all_completed {
            "completed".into()
        } else if all_cancelled {
            "cancelled".into()
        } else if any_completed {
            "partial_failure".into()
        } else {
            "failed".into()
        };
    }

    to_cancel_tasks
}

/// Renvoie les `step_id` en attente dont TOUTES les dépendances sont `completed`.
pub(crate) fn ready_waiting_steps(graph: &NativeGraphRecord) -> Vec<String> {
    let completed: BTreeSet<&str> = graph
        .steps
        .iter()
        .filter(|s| s.state == "completed")
        .map(|s| s.step_id.as_str())
        .collect();
    graph
        .steps
        .iter()
        .filter(|s| {
            s.state == "waiting_dependencies"
                && s.depends_on.iter().all(|d| completed.contains(d.as_str()))
        })
        .map(|s| s.step_id.clone())
        .collect()
}

pub(crate) fn graph_view(graph: &NativeGraphRecord) -> Value {
    let total = graph.steps.len();
    let completed = graph
        .steps
        .iter()
        .filter(|s| s.state == "completed")
        .count();
    let running = graph
        .steps
        .iter()
        .filter(|s| matches!(s.state.as_str(), "running" | "launching"))
        .count();
    let waiting = graph
        .steps
        .iter()
        .filter(|s| s.state == "waiting_dependencies")
        .count();
    let failed = graph
        .steps
        .iter()
        .filter(|s| matches!(s.state.as_str(), "failed" | "skipped_dependency_failed"))
        .count();
    let cancelled = graph
        .steps
        .iter()
        .filter(|s| s.state == "cancelled")
        .count();
    let steps: Vec<Value> = graph
        .steps
        .iter()
        .map(|s| {
            json!({
                "step_id": s.step_id,
                "depends_on": s.depends_on,
                "state": s.state,
                "task_id": s.task_id,
                "error": s.error,
                "factual_summary": s.factual_summary,
                "result_excerpt": s.result_excerpt,
                "convergence": s.convergence_state,
                "started_at": s.started_at,
                "finished_at": s.finished_at,
            })
        })
        .collect();
    let next_action = if graph.state == "running" {
        "Graphe pris en charge par Bridget : inutile de lancer une ronde de garde ou du polling. Vous serez réveillé automatiquement à la fin du graphe ou sur échec. Utilisez bridget_graph_status pour un point ponctuel."
    } else {
        "Graphe terminé : consultez le bilan des étapes ci-dessus ou bridget_task_status(task_id) pour le détail complet d'une étape."
    };
    json!({
        "version": 1,
        "status": "ok",
        "graph_id": graph.graph_id,
        "request_id": graph.request_id,
        "title": graph.title,
        "state": graph.state,
        "on_failure": graph.on_failure,
        "notify_policy": graph.notify_policy,
        "topological_order": graph.topological_order,
        "counts": {
            "total": total,
            "completed": completed,
            "running": running,
            "waiting_dependencies": waiting,
            "failed_or_skipped": failed,
            "cancelled": cancelled,
        },
        "steps": steps,
        "notice_sent": graph.notice_sent,
        "created_at": graph.created_at,
        "updated_at": graph.updated_at,
        "next_action": next_action,
    })
}

fn graph_failure(code: &str, message: &str) -> Value {
    json!({
        "version": 1,
        "status": "refused",
        "code": code,
        "scope": "this_request",
        "message": message,
        "next_action": "Corrigez la déclaration du graphe (étapes, dépendances, absence de cycle) avant de resoumettre. Les autres travaux autorisés suivent leur propre état.",
    })
}

fn format_graph_summary_message(graph: &NativeGraphRecord) -> String {
    let title = graph
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(&graph.request_id);
    let mut out = format!(
        "Graphe Bridget `{}` ({}) terminé — état : `{}`.\n",
        title, graph.graph_id, graph.state
    );
    for step in &graph.steps {
        out.push_str(&format!("- `{}` : {}", step.step_id, step.state));
        if let Some(tid) = step.task_id.as_deref() {
            out.push_str(&format!(" (task_id={tid})"));
        }
        if let Some(commit) = step
            .factual_summary
            .as_ref()
            .and_then(|v| v.get("commit"))
            .and_then(Value::as_str)
        {
            out.push_str(&format!(" · commit={commit}"));
        }
        if let Some(cs) = step.convergence_state.as_ref() {
            let outcome = cs.outcome.as_deref().unwrap_or(cs.phase.as_str());
            out.push_str(&format!(
                " · convergence={outcome} ({}/{})",
                cs.current_iteration, cs.max_iterations
            ));
        }
        if let Some(err) = step.error.as_deref() {
            out.push_str(&format!(" · erreur={err}"));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "Consultez `bridget_graph_status(graph_id=\"{}\")` ou `bridget_task_status(task_id)` pour le détail complet.",
        graph.graph_id
    ));
    out
}

/// Soumission d'un nouveau graphe de missions (`bridget_delegate_graph`).
pub(super) fn handle_submit(
    conn: &str,
    request: DelegateGraphRequest,
    fact: Option<ProviderPermissions>,
    thread: Option<&str>,
    permission_proof: Option<&bridget_transport::protocol::NativePermissionProof>,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
) -> Value {
    let topological_order = match validate_and_order_graph(&request) {
        Ok(order) => order,
        Err((code, msg)) => return graph_failure(&code, &msg),
    };
    let canonical = match serde_json::to_vec(&request) {
        Ok(bytes) => bytes,
        Err(error) => return graph_failure("graph_invalid", &error.to_string()),
    };

    let now = unix_timestamp();
    let (mut graph, root_indices) = {
        let st = state.lock().recover_poison();
        if let Err(code) = super::native_delegation::recheck_attested_thread_pub(&st, conn, thread)
        {
            return graph_failure(&code, "Attestation du fil parent indisponible.");
        }
        let Some((owner, instance)) = live_connection_identity(&st, conn) else {
            return graph_failure(
                "identity_unavailable",
                "Identité de connexion parent indisponible pour soumettre le graphe.",
            );
        };
        let attested = match super::native_delegation::parent_fact_pub(
            &st,
            &owner,
            &instance,
            Some(&request.request_id),
            fact.as_ref(),
        ) {
            Ok(Some(perms)) => perms,
            Ok(None) => {
                return graph_failure(
                    "permission_attestation_unavailable",
                    "Permissions attestées du parent introuvables.",
                );
            }
            Err(code) => return graph_failure(&code, &format!("Refus d'autorisation : {code}")),
        };
        if let Ok(Some(existing)) = st
            .delegation_store
            .graph_by_owner_request(&owner, &request.request_id)
        {
            if existing.canonical != canonical {
                return graph_failure(
                    "envelope_mismatch",
                    "Cette request_id appartient déjà à un graphe de paramètres différents.",
                );
            }
            return graph_view(&existing);
        }

        let steps: Vec<NativeGraphStepRecord> = request
            .steps
            .iter()
            .map(|s| {
                let mut delegation = s.delegation.clone();
                delegation.request_id = if s.convergence.is_some() {
                    convergence_step_request_id(&request.request_id, &s.step_id, 1, "produce")
                } else {
                    step_request_id(&request.request_id, &s.step_id, &delegation.request_id)
                };
                if s.convergence.is_some() {
                    ensure_convergence_producer_keep_alive(&mut delegation);
                }
                let convergence_state =
                    s.convergence
                        .as_ref()
                        .map(|conv| NativeGraphConvergenceState {
                            current_iteration: 1,
                            max_iterations: conv.max_iterations,
                            stop_on_stagnation: conv.stop_on_stagnation,
                            phase: "waiting".into(),
                            producer_task_id: None,
                            producer_expected_round: 1,
                            verifier_task_id: None,
                            last_producer_commit: None,
                            last_blocking_fingerprint: None,
                            outcome: None,
                            iterations: Vec::new(),
                        });
                NativeGraphStepRecord {
                    step_id: s.step_id.clone(),
                    depends_on: s.depends_on.clone(),
                    pass_dependency_results: s.pass_dependency_results,
                    delegation,
                    convergence: s.convergence.clone(),
                    convergence_state,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                }
            })
            .collect();
        let root_indices: Vec<usize> = steps
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.depends_on.is_empty().then_some(i))
            .collect();
        let graph = NativeGraphRecord {
            graph_id: format!("graph-{}", uuid::Uuid::new_v4().simple()),
            owner: owner.clone(),
            owner_instance: instance,
            request_id: request.request_id.clone(),
            title: request.title.clone(),
            on_failure: request.on_failure,
            notify_policy: request.notify_policy,
            state: "running".into(),
            topological_order,
            steps,
            attested_permissions: Some(attested),
            parent_thread: thread.map(str::to_owned),
            communication_project: agent_communication_project(&st, &owner),
            notice_sent: false,
            canonical,
            created_at: now,
            updated_at: now,
        };
        if let Err(error) = st.delegation_store.save_graph(&graph) {
            return graph_failure("graph_store_unavailable", &error);
        }
        (graph, root_indices)
    };

    // Admettre immédiatement les étapes racines (sans dépendance) sur la connexion vivante.
    let suppress_step_notices = graph.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
    for idx in root_indices {
        let step_delegation = graph.steps[idx].delegation.clone();
        let step_id = graph.steps[idx].step_id.clone();
        let response = super::native_delegation::handle_with_fact_pub(
            conn,
            NativeDelegationRequest::AutoDelegateV2(step_delegation),
            fact.clone(),
            thread,
            permission_proof,
            state,
            peer,
        );
        apply_admission_response(
            &mut graph,
            idx,
            &step_id,
            response,
            suppress_step_notices,
            state,
        );
    }

    let now = unix_timestamp();
    let to_cancel = propagate_graph_state(&mut graph, now);
    graph.updated_at = now;
    {
        let st = state.lock().recover_poison();
        let _ = st.delegation_store.save_graph(&graph);
    }
    for task_id in to_cancel {
        let _ = cancel_task_for_graph(&graph, &task_id, state);
    }

    graph_view(&graph)
}

fn apply_admission_response(
    graph: &mut NativeGraphRecord,
    idx: usize,
    step_id: &str,
    response: DaemonToWrapper,
    suppress_step_notices: bool,
    state: &Arc<Mutex<DaemonState>>,
) {
    let now = unix_timestamp();
    match response {
        DaemonToWrapper::NativeDelegationResult { result } => {
            if result["status"] != "refused"
                && let Some(task_id) = result["task_id"].as_str()
            {
                graph.steps[idx].state = "running".into();
                graph.steps[idx].task_id = Some(task_id.to_string());
                graph.steps[idx].started_at.get_or_insert(now);
                if let Some(cs) = graph.steps[idx].convergence_state.as_mut() {
                    cs.phase = "producing".into();
                    cs.producer_task_id = Some(task_id.to_string());
                    cs.producer_expected_round = 1;
                    cs.iterations.push(NativeGraphConvergenceIteration {
                        iteration: cs.current_iteration,
                        producer_task_id: Some(task_id.to_string()),
                        producer_round: 1,
                        producer_commit: None,
                        verifier_task_id: None,
                        verdict: None,
                        reason: None,
                        blocking_points: Vec::new(),
                        blocking_fingerprint: None,
                        started_at: Some(now),
                        finished_at: None,
                    });
                }
                let st = state.lock().recover_poison();
                let _ = st.delegation_store.bind_graph_step_task(
                    task_id,
                    &graph.graph_id,
                    step_id,
                    suppress_step_notices,
                );
            } else {
                let code = result["code"]
                    .as_str()
                    .or_else(|| result["error"].as_str())
                    .unwrap_or("admission_refused");
                let msg = result["message"].as_str().unwrap_or(code);
                graph.steps[idx].state = "failed".into();
                graph.steps[idx].error = Some(format!("{code}: {msg}").chars().take(512).collect());
                graph.steps[idx].finished_at = Some(now);
                if let Some(cs) = graph.steps[idx].convergence_state.as_mut() {
                    cs.phase = "done".into();
                    cs.outcome = Some("failed".into());
                }
            }
        }
        other => {
            graph.steps[idx].state = "failed".into();
            graph.steps[idx].error = Some(
                format!("unexpected_admission_response:{other:?}")
                    .chars()
                    .take(256)
                    .collect(),
            );
            graph.steps[idx].finished_at = Some(now);
            if let Some(cs) = graph.steps[idx].convergence_state.as_mut() {
                cs.phase = "done".into();
                cs.outcome = Some("failed".into());
            }
        }
    }
}

/// Lecture de l'état d'un graphe appartenant au parent (`bridget_graph_status`).
pub(super) fn handle_status(
    conn: &str,
    graph_id: &str,
    thread: Option<&str>,
    state: &Arc<Mutex<DaemonState>>,
) -> Value {
    let st = state.lock().recover_poison();
    if let Err(code) = super::native_delegation::recheck_attested_thread_pub(&st, conn, thread) {
        return graph_failure(&code, "Attestation du fil parent indisponible.");
    }
    let Some((owner, _)) = live_connection_identity(&st, conn) else {
        return graph_failure("identity_unavailable", "Identité parent indisponible.");
    };
    let Ok(Some(mut graph)) = st.delegation_store.get_graph(graph_id) else {
        return graph_failure("graph_not_found", "Graphe introuvable.");
    };
    if graph.owner != owner {
        return graph_failure("graph_not_found", "Graphe introuvable.");
    }
    sync_steps_from_tasks(&st, &mut graph, unix_timestamp());
    graph_view(&graph)
}

/// Annulation d'un graphe appartenant au parent (`bridget_graph_cancel`).
pub(super) fn handle_cancel(
    conn: &str,
    graph_id: &str,
    thread: Option<&str>,
    state: &Arc<Mutex<DaemonState>>,
) -> Value {
    let (mut graph, running_tasks) = {
        let st = state.lock().recover_poison();
        if let Err(code) = super::native_delegation::recheck_attested_thread_pub(&st, conn, thread)
        {
            return graph_failure(&code, "Attestation du fil parent indisponible.");
        }
        let Some((owner, _)) = live_connection_identity(&st, conn) else {
            return graph_failure("identity_unavailable", "Identité parent indisponible.");
        };
        let Ok(Some(mut graph)) = st.delegation_store.get_graph(graph_id) else {
            return graph_failure("graph_not_found", "Graphe introuvable.");
        };
        if graph.owner != owner {
            return graph_failure("graph_not_found", "Graphe introuvable.");
        }
        let now = unix_timestamp();
        let mut tasks_to_cancel = Vec::new();
        for step in &mut graph.steps {
            match step.state.as_str() {
                "waiting_dependencies" | "launching" => {
                    step.state = "cancelled".into();
                    step.error.get_or_insert_with(|| "graph_cancelled".into());
                    step.finished_at = Some(now);
                }
                "running" => {
                    if let Some(tid) = step.task_id.clone() {
                        tasks_to_cancel.push(tid);
                    }
                    if let Some(cs) = step.convergence_state.as_mut() {
                        if let Some(ptid) = cs.producer_task_id.as_ref()
                            && !tasks_to_cancel.contains(ptid)
                            && st
                                .delegation_store
                                .get(ptid)
                                .ok()
                                .flatten()
                                .is_some_and(|t| t.state == "working")
                        {
                            tasks_to_cancel.push(ptid.clone());
                        }
                        if let Some(vtid) = cs.verifier_task_id.as_ref()
                            && !tasks_to_cancel.contains(vtid)
                            && st
                                .delegation_store
                                .get(vtid)
                                .ok()
                                .flatten()
                                .is_some_and(|t| t.state == "working")
                        {
                            tasks_to_cancel.push(vtid.clone());
                        }
                        cs.phase = "done".into();
                        cs.outcome.get_or_insert_with(|| "cancelled".into());
                    }
                    step.state = "cancelled".into();
                    step.error.get_or_insert_with(|| "graph_cancelled".into());
                    step.finished_at = Some(now);
                }
                _ => {}
            }
        }
        graph.state = "cancelled".into();
        graph.updated_at = now;
        let _ = st.delegation_store.save_graph(&graph);
        (graph, tasks_to_cancel)
    };

    for tid in running_tasks {
        let _ = cancel_task_for_graph(&graph, &tid, state);
    }
    graph.updated_at = unix_timestamp();
    {
        let st = state.lock().recover_poison();
        let _ = st.delegation_store.save_graph(&graph);
    }
    graph_view(&graph)
}

fn sync_steps_from_tasks(st: &DaemonState, graph: &mut NativeGraphRecord, now: i64) -> bool {
    let mut changed = false;
    for step in &mut graph.steps {
        if step.state != "running" {
            continue;
        }
        if step.convergence.is_some() {
            if sync_convergence_step_from_tasks(st, step, now) {
                changed = true;
            }
            continue;
        }
        let Some(task_id) = step.task_id.as_deref() else {
            continue;
        };
        let Ok(Some(task)) = st.delegation_store.get(task_id) else {
            continue;
        };
        match task.state.as_str() {
            "result_available" => {
                step.state = "completed".into();
                step.finished_at = Some(now);
                step.error = None;
                step.result_excerpt = task
                    .result
                    .as_deref()
                    .map(|r| r.chars().take(MAX_DEP_EXCERPT_CHARS).collect());
                step.factual_summary = Some(super::native_delegation::factual_summary(
                    &task,
                    task.result.as_ref().map(String::len),
                ));
                changed = true;
            }
            "failed" => {
                step.state = "failed".into();
                step.finished_at = Some(now);
                step.error = task.error.clone().or_else(|| Some("task_failed".into()));
                step.factual_summary = Some(super::native_delegation::factual_summary(&task, None));
                changed = true;
            }
            "cancelled" => {
                step.state = "cancelled".into();
                step.finished_at = Some(now);
                step.error = Some("task_cancelled".into());
                changed = true;
            }
            _ => {}
        }
    }
    changed
}

/// Synchronise une étape de convergence bornée (`producing` / `verifying`) depuis
/// les tâches durables stockées dans `DelegationStore`, sans appel bloquant sous verrou.
pub(crate) fn sync_convergence_step_from_tasks(
    st: &DaemonState,
    step: &mut NativeGraphStepRecord,
    now: i64,
) -> bool {
    let Some(cs) = step.convergence_state.as_mut() else {
        return false;
    };
    match cs.phase.as_str() {
        "producing" => {
            let Some(task_id) = cs.producer_task_id.as_deref().or(step.task_id.as_deref()) else {
                return false;
            };
            let Ok(Some(task)) = st.delegation_store.get(task_id) else {
                return false;
            };
            match task.state.as_str() {
                "result_available" if task.round >= cs.producer_expected_round => {
                    let fs = super::native_delegation::factual_summary(
                        &task,
                        task.result.as_ref().map(String::len),
                    );
                    let producer_commit = fs
                        .get("commit")
                        .and_then(Value::as_str)
                        .filter(|sha| is_valid_git_sha40(sha))
                        .map(str::to_owned);
                    let excerpt = task
                        .result
                        .as_deref()
                        .map(|r| r.chars().take(MAX_DEP_EXCERPT_CHARS).collect());
                    apply_producer_iteration_result(
                        step,
                        task.round,
                        producer_commit,
                        excerpt,
                        fs,
                        now,
                    );
                    true
                }
                "failed" | "cancelled" => {
                    if let Some(iter_rec) = cs.iterations.last_mut() {
                        iter_rec.finished_at = Some(now);
                    }
                    cs.phase = "done".into();
                    cs.outcome = Some(if task.state == "cancelled" {
                        "cancelled".into()
                    } else {
                        "failed".into()
                    });
                    step.state = task.state.clone();
                    step.finished_at = Some(now);
                    step.error = task.error.clone().or_else(|| {
                        Some(if task.state == "cancelled" {
                            "task_cancelled".into()
                        } else {
                            "producer_task_failed".into()
                        })
                    });
                    step.factual_summary =
                        Some(super::native_delegation::factual_summary(&task, None));
                    true
                }
                _ => false,
            }
        }
        "verifying" => {
            let Some(task_id) = cs.verifier_task_id.as_deref().or(step.task_id.as_deref()) else {
                return false;
            };
            let Ok(Some(task)) = st.delegation_store.get(task_id) else {
                return false;
            };
            match task.state.as_str() {
                "result_available" => {
                    let parsed = parse_convergence_verdict(task.result.as_deref().unwrap_or(""));
                    apply_verifier_verdict(step, parsed, now);
                    true
                }
                "failed" | "cancelled" => {
                    if let Some(iter_rec) = cs.iterations.last_mut() {
                        iter_rec.finished_at = Some(now);
                    }
                    cs.phase = "done".into();
                    cs.outcome = Some(if task.state == "cancelled" {
                        "cancelled".into()
                    } else {
                        "failed".into()
                    });
                    step.state = task.state.clone();
                    step.finished_at = Some(now);
                    step.error = task.error.clone().or_else(|| {
                        Some(if task.state == "cancelled" {
                            "verifier_task_cancelled".into()
                        } else {
                            "verifier_task_failed".into()
                        })
                    });
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Applique la remise d'une itération producteur sur une étape de convergence
/// et déclenche l'arrêt sur stagnation si le commit produit est identique au tour précédent.
pub(crate) fn apply_producer_iteration_result(
    step: &mut NativeGraphStepRecord,
    task_round: u32,
    producer_commit: Option<String>,
    excerpt: Option<String>,
    fs: Value,
    now: i64,
) {
    let Some(cs) = step.convergence_state.as_mut() else {
        return;
    };
    if let Some(iter_rec) = cs.iterations.last_mut() {
        iter_rec.producer_round = task_round;
        iter_rec.producer_commit = producer_commit.clone();
    }
    step.result_excerpt = excerpt;
    step.factual_summary = Some(fs);

    // Détection de stagnation #1 : même commit non vide sur 2 tours consécutifs.
    if cs.stop_on_stagnation
        && cs.current_iteration >= 2
        && let Some(commit) = producer_commit.as_deref()
        && cs.last_producer_commit.as_deref() == Some(commit)
    {
        if let Some(iter_rec) = cs.iterations.last_mut() {
            iter_rec.finished_at = Some(now);
        }
        cs.phase = "done".into();
        cs.outcome = Some("stagnated".into());
        step.state = "failed".into();
        step.finished_at = Some(now);
        step.error = Some(format!(
            "convergence_stagnated: commit identique sur les itérations {} et {} ({commit})",
            cs.current_iteration - 1,
            cs.current_iteration
        ));
        return;
    }

    if producer_commit.is_some() {
        cs.last_producer_commit = producer_commit;
    }
    cs.phase = "pending_verify".into();
}

/// Applique la transition déterministe d'un verdict de vérificateur sur une étape de convergence.
pub(crate) fn apply_verifier_verdict(
    step: &mut NativeGraphStepRecord,
    parsed: ParsedVerdict,
    now: i64,
) {
    let Some(cs) = step.convergence_state.as_mut() else {
        return;
    };
    if let Some(iter_rec) = cs.iterations.last_mut() {
        iter_rec.finished_at = Some(now);
    }
    match parsed {
        ParsedVerdict::Approve { reason } => {
            if let Some(iter_rec) = cs.iterations.last_mut() {
                iter_rec.verdict = Some("approve".into());
                iter_rec.reason = Some(reason);
            }
            cs.phase = "done".into();
            cs.outcome = Some("approved".into());
            step.state = "completed".into();
            step.finished_at = Some(now);
            step.error = None;
            // Exposer la tâche productrice validée comme task_id final pour les étapes aval.
            if let Some(ptid) = cs.producer_task_id.clone() {
                step.task_id = Some(ptid);
            }
        }
        ParsedVerdict::Escalate { reason } => {
            if let Some(iter_rec) = cs.iterations.last_mut() {
                iter_rec.verdict = Some("escalate".into());
                iter_rec.reason = Some(reason.clone());
            }
            cs.phase = "done".into();
            cs.outcome = Some("escalated".into());
            step.state = "failed".into();
            step.finished_at = Some(now);
            step.error = Some(format!("convergence_escalated: {reason}"));
        }
        ParsedVerdict::Revise {
            reason,
            blocking_points,
            fingerprint,
        } => {
            if let Some(iter_rec) = cs.iterations.last_mut() {
                iter_rec.verdict = Some("revise".into());
                iter_rec.reason = Some(reason.clone());
                iter_rec.blocking_points = blocking_points;
                iter_rec.blocking_fingerprint = Some(fingerprint.clone());
            }
            // Détection de stagnation #2 : mêmes motifs bloquants normalisés deux tours de suite.
            if cs.stop_on_stagnation
                && cs.current_iteration >= 2
                && cs.last_blocking_fingerprint.as_deref() == Some(fingerprint.as_str())
            {
                cs.phase = "done".into();
                cs.outcome = Some("stagnated".into());
                step.state = "failed".into();
                step.finished_at = Some(now);
                step.error = Some(format!(
                    "convergence_stagnated: motifs bloquants identiques sur les itérations {} et {} ({reason})",
                    cs.current_iteration - 1,
                    cs.current_iteration
                ));
                return;
            }
            if cs.current_iteration >= cs.max_iterations {
                cs.phase = "done".into();
                cs.outcome = Some("max_iterations_reached".into());
                step.state = "failed".into();
                step.finished_at = Some(now);
                step.error = Some(format!(
                    "convergence_max_iterations_reached: {} itérations sans approbation ({reason})",
                    cs.max_iterations
                ));
                return;
            }
            cs.last_blocking_fingerprint = Some(fingerprint);
            cs.current_iteration += 1;
            cs.phase = "pending_produce".into();
        }
        ParsedVerdict::Unparseable => {
            if let Some(iter_rec) = cs.iterations.last_mut() {
                iter_rec.verdict = Some("unparseable".into());
            }
            cs.phase = "done".into();
            cs.outcome = Some("verdict_unparseable".into());
            step.state = "failed".into();
            step.finished_at = Some(now);
            step.error = Some(
                "convergence_verdict_unparseable: le vérificateur n'a pas rendu de ligne JSON fermée {\"verdict\":\"approve\"|\"revise\"|\"escalate\"}"
                    .into(),
            );
        }
    }
}

struct GraphExecIdentityGuard {
    state: Arc<Mutex<DaemonState>>,
    conn_id: String,
    installed: bool,
}

impl Drop for GraphExecIdentityGuard {
    fn drop(&mut self) {
        if self.installed {
            let mut st = self.state.lock().recover_poison();
            st.graph_exec_identities.remove(&self.conn_id);
        }
    }
}

fn prepare_exec_conn(
    graph: &NativeGraphRecord,
    step_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<(String, GraphExecIdentityGuard), String> {
    let mut st = state.lock().recover_poison();
    if st.delegation_store.revoked_agent(&graph.owner)?
        || st.delegation_store.revoked(&graph.owner_instance)?
    {
        return Err("delegation_grant_required".into());
    }
    if let Some(route) = st.router.get_agent(&graph.owner)
        && live_connection_identity(&st, &route.connection_id)
            == Some((graph.owner.clone(), graph.owner_instance.clone()))
    {
        let conn = route.connection_id.clone();
        return Ok((
            conn.clone(),
            GraphExecIdentityGuard {
                state: Arc::clone(state),
                conn_id: conn,
                installed: false,
            },
        ));
    }
    let synthetic = format!("graph-exec:{}:{}", graph.graph_id, step_id);
    st.graph_exec_identities.insert(
        synthetic.clone(),
        (
            graph.owner.clone(),
            graph.owner_instance.clone(),
            graph.communication_project.clone(),
        ),
    );
    Ok((
        synthetic.clone(),
        GraphExecIdentityGuard {
            state: Arc::clone(state),
            conn_id: synthetic,
            installed: true,
        },
    ))
}

fn cancel_task_for_graph(
    graph: &NativeGraphRecord,
    task_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Result<(), String> {
    let (exec_conn, _guard) = prepare_exec_conn(graph, &format!("cancel-{task_id}"), state)?;
    let _ = super::native_delegation::handle_with_fact_pub(
        &exec_conn,
        NativeDelegationRequest::Cancel {
            task_id: task_id.to_string(),
        },
        graph.attested_permissions.clone(),
        None,
        None,
        state,
        None,
    );
    Ok(())
}

enum GraphLaunchJob {
    StepStart {
        graph: NativeGraphRecord,
        idx: usize,
        delegation: AutoDelegateV2,
    },
    ConvergenceVerify {
        graph: NativeGraphRecord,
        idx: usize,
        iteration: u8,
        delegation: AutoDelegateV2,
    },
    ConvergenceNextProduce {
        graph: NativeGraphRecord,
        idx: usize,
        iteration: u8,
        continue_task_id: Option<String>,
        continue_request_id: String,
        continue_task: String,
        fallback_delegation: AutoDelegateV2,
    },
}

/// Appelé à chaque cadence du daemon (`native_delegation::tick`).
/// Coût quasi nul quand aucun graphe n'est actif.
pub(super) fn tick(state: &Arc<Mutex<DaemonState>>) {
    let mut launches: Vec<GraphLaunchJob> = Vec::new();
    let mut cancellations: Vec<(NativeGraphRecord, String)> = Vec::new();
    let mut controls = Vec::new();

    {
        let mut st = state.lock().recover_poison();
        let control = match crate::referent_control::read(st.store.connection()) {
            Ok(c) => c,
            Err(_) => return,
        };
        let Ok(graphs) = st.delegation_store.active_graphs() else {
            return;
        };
        if graphs.is_empty() {
            return;
        }
        let now = unix_timestamp();

        for mut graph in graphs {
            let mut dirty = sync_steps_from_tasks(&st, &mut graph, now);
            let prev_state = graph.state.clone();
            let to_cancel = propagate_graph_state(&mut graph, now);
            if graph.state != prev_state || !to_cancel.is_empty() {
                dirty = true;
            }
            for tid in to_cancel {
                cancellations.push((graph.clone(), tid));
            }

            if graph.state == "running" && !control.paused {
                let ready_ids = ready_waiting_steps(&graph);
                for step_id in ready_ids {
                    let launch_key = format!("{}:{}:start:1", graph.graph_id, step_id);
                    if !st.graph_launching_steps.insert(launch_key) {
                        continue;
                    }
                    if let Some(idx) = graph.steps.iter().position(|s| s.step_id == step_id) {
                        let enriched_task =
                            build_enriched_step_task(&graph.steps[idx], &graph.steps);
                        let mut delegation = graph.steps[idx].delegation.clone();
                        delegation.task = enriched_task;
                        if graph.steps[idx].convergence.is_some() {
                            ensure_convergence_producer_keep_alive(&mut delegation);
                        }
                        graph.steps[idx].state = "launching".into();
                        dirty = true;
                        launches.push(GraphLaunchJob::StepStart {
                            graph: graph.clone(),
                            idx,
                            delegation,
                        });
                    }
                }

                // Transitions internes des étapes de convergence (`pending_verify` / `pending_produce`).
                for idx in 0..graph.steps.len() {
                    if graph.steps[idx].state != "running" {
                        continue;
                    }
                    let Some(conv) = graph.steps[idx].convergence.clone() else {
                        continue;
                    };
                    let Some(cs) = graph.steps[idx].convergence_state.clone() else {
                        continue;
                    };
                    let step_id = graph.steps[idx].step_id.clone();
                    if matches!(cs.phase.as_str(), "pending_verify" | "launching_verify") {
                        let launch_key = format!(
                            "{}:{}:verify:{}",
                            graph.graph_id, step_id, cs.current_iteration
                        );
                        if !st.graph_launching_steps.insert(launch_key) {
                            continue;
                        }
                        let mut verify_del = conv.verify.clone();
                        verify_del.request_id = convergence_step_request_id(
                            &graph.request_id,
                            &step_id,
                            cs.current_iteration,
                            "verify",
                        );
                        apply_commit_to_delegation(
                            &mut verify_del,
                            cs.last_producer_commit.as_deref(),
                        );
                        verify_del.task = build_verifier_task(
                            &graph.steps[idx],
                            &conv.verify,
                            cs.current_iteration,
                            cs.max_iterations,
                        );
                        if let Some(state_mut) = graph.steps[idx].convergence_state.as_mut() {
                            state_mut.phase = "launching_verify".into();
                        }
                        dirty = true;
                        launches.push(GraphLaunchJob::ConvergenceVerify {
                            graph: graph.clone(),
                            idx,
                            iteration: cs.current_iteration,
                            delegation: verify_del,
                        });
                    } else if matches!(cs.phase.as_str(), "pending_produce" | "launching_produce") {
                        let launch_key = format!(
                            "{}:{}:produce:{}",
                            graph.graph_id, step_id, cs.current_iteration
                        );
                        if !st.graph_launching_steps.insert(launch_key) {
                            continue;
                        }
                        let (last_reason, last_points) = cs
                            .iterations
                            .last()
                            .map(|it| {
                                (
                                    it.reason
                                        .as_deref()
                                        .unwrap_or("corrections demandées par le vérificateur")
                                        .to_string(),
                                    it.blocking_points.clone(),
                                )
                            })
                            .unwrap_or_else(|| ("corrections demandées".into(), Vec::new()));
                        let continue_request_id = convergence_step_request_id(
                            &graph.request_id,
                            &step_id,
                            cs.current_iteration,
                            "produce",
                        );
                        let continue_task = build_producer_revision_task(
                            &step_id,
                            cs.current_iteration,
                            cs.max_iterations,
                            &last_reason,
                            &last_points,
                            &graph.steps[idx].delegation.task,
                            false,
                        );
                        let mut fallback_del = graph.steps[idx].delegation.clone();
                        fallback_del.request_id = convergence_step_request_id(
                            &graph.request_id,
                            &step_id,
                            cs.current_iteration,
                            "produce-new",
                        );
                        ensure_convergence_producer_keep_alive(&mut fallback_del);
                        apply_commit_to_delegation(
                            &mut fallback_del,
                            cs.last_producer_commit.as_deref(),
                        );
                        fallback_del.task = build_producer_revision_task(
                            &step_id,
                            cs.current_iteration,
                            cs.max_iterations,
                            &last_reason,
                            &last_points,
                            &graph.steps[idx].delegation.task,
                            true,
                        );
                        if let Some(state_mut) = graph.steps[idx].convergence_state.as_mut() {
                            state_mut.phase = "launching_produce".into();
                        }
                        dirty = true;
                        launches.push(GraphLaunchJob::ConvergenceNextProduce {
                            graph: graph.clone(),
                            idx,
                            iteration: cs.current_iteration,
                            continue_task_id: cs.producer_task_id.clone(),
                            continue_request_id,
                            continue_task,
                            fallback_delegation: fallback_del,
                        });
                    }
                }
            }

            // Envoi unique consolidé au parent dès que le graphe atteint un état terminal.
            if graph.state != "running" && !graph.notice_sent {
                let parent_live = st
                    .router
                    .get_agent(&graph.owner)
                    .and_then(|route| st.conn_instances.get(&route.connection_id))
                    == Some(&graph.owner_instance)
                    && st
                        .presences
                        .get(&graph.owner_instance)
                        .is_some_and(|p| p.state == "connected");
                if parent_live {
                    let prefix = if graph.state == "completed" {
                        "native-graph-result"
                    } else {
                        "native-graph-failure"
                    };
                    let mut notice = bridget_core::BridgetMessage::new(
                        &graph.owner,
                        &graph.owner,
                        format_graph_summary_message(&graph),
                    );
                    notice.id = format!("{prefix}-{}", graph.graph_id);
                    notice.origin = Some(bridget_core::MessageOrigin::System);
                    let outcome = super::native_delegation::send_as(
                        &mut st,
                        &graph.owner,
                        &graph.owner_instance,
                        &graph.graph_id,
                        notice,
                        graph.created_at,
                        &mut controls,
                    );
                    if matches!(
                        outcome,
                        DaemonToWrapper::IdempotencyResult {
                            issue: IdempotencyIssue::Accepted { .. }
                                | IdempotencyIssue::OutcomeUnknown { .. },
                            ..
                        }
                    ) {
                        graph.notice_sent = true;
                        dirty = true;
                    }
                }
            }

            if dirty {
                graph.updated_at = now;
                let _ = st.delegation_store.save_graph(&graph);
            }
        }
    }

    if !controls.is_empty() {
        let _ = execute_controls(controls);
    }

    for (graph, task_id) in cancellations {
        let _ = cancel_task_for_graph(&graph, &task_id, state);
    }

    for job in launches {
        let state_bg = Arc::clone(state);
        match job {
            GraphLaunchJob::StepStart {
                graph: graph_snapshot,
                idx,
                delegation,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let suppress =
                    graph_snapshot.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
                let _ = thread::Builder::new()
                    .name(format!("bridget-graph-{}", &step_id))
                    .spawn(move || {
                        let launch_key = format!("{graph_id}:{step_id}:start:1");
                        let response = match prepare_exec_conn(&graph_snapshot, &step_id, &state_bg)
                        {
                            Ok((exec_conn, _guard)) => {
                                super::native_delegation::handle_with_fact_pub(
                                    &exec_conn,
                                    NativeDelegationRequest::AutoDelegateV2(delegation),
                                    graph_snapshot.attested_permissions.clone(),
                                    None,
                                    None,
                                    &state_bg,
                                    None,
                                )
                            }
                            Err(code) => DaemonToWrapper::NativeDelegationResult {
                                result: graph_failure(
                                    &code,
                                    "Contexte parent révoqué ou indisponible.",
                                ),
                            },
                        };

                        let mut st = state_bg.lock().recover_poison();
                        st.graph_launching_steps.remove(&launch_key);
                        if let Ok(Some(mut current)) = st.delegation_store.get_graph(&graph_id)
                            && let Some(pos) =
                                current.steps.iter().position(|s| s.step_id == step_id)
                            && current.steps[pos].state == "launching"
                        {
                            drop(st);
                            apply_admission_response(
                                &mut current,
                                pos,
                                &step_id,
                                response,
                                suppress,
                                &state_bg,
                            );
                            let now = unix_timestamp();
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            let st2 = state_bg.lock().recover_poison();
                            let _ = st2.delegation_store.save_graph(&current);
                        }
                    });
            }
            GraphLaunchJob::ConvergenceVerify {
                graph: graph_snapshot,
                idx,
                iteration,
                delegation,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let suppress =
                    graph_snapshot.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
                let _ = thread::Builder::new()
                    .name(format!("bridget-graph-verify-{}", &step_id))
                    .spawn(move || {
                        let launch_key = format!("{graph_id}:{step_id}:verify:{iteration}");
                        let exec_tag = format!("{step_id}-verify-i{iteration}");
                        let response =
                            match prepare_exec_conn(&graph_snapshot, &exec_tag, &state_bg) {
                                Ok((exec_conn, _guard)) => {
                                    super::native_delegation::handle_with_fact_pub(
                                        &exec_conn,
                                        NativeDelegationRequest::AutoDelegateV2(delegation),
                                        graph_snapshot.attested_permissions.clone(),
                                        None,
                                        None,
                                        &state_bg,
                                        None,
                                    )
                                }
                                Err(code) => DaemonToWrapper::NativeDelegationResult {
                                    result: graph_failure(
                                        &code,
                                        "Contexte parent révoqué ou indisponible.",
                                    ),
                                },
                            };

                        let mut st = state_bg.lock().recover_poison();
                        st.graph_launching_steps.remove(&launch_key);
                        if let Ok(Some(mut current)) = st.delegation_store.get_graph(&graph_id)
                            && let Some(pos) =
                                current.steps.iter().position(|s| s.step_id == step_id)
                            && current.steps[pos].state == "running"
                            && current.steps[pos]
                                .convergence_state
                                .as_ref()
                                .is_some_and(|cs| cs.phase == "launching_verify")
                        {
                            let now = unix_timestamp();
                            match response {
                                DaemonToWrapper::NativeDelegationResult { result }
                                    if result["status"] != "refused"
                                        && result["task_id"].as_str().is_some() =>
                                {
                                    let task_id = result["task_id"].as_str().unwrap().to_string();
                                    current.steps[pos].task_id = Some(task_id.clone());
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                    {
                                        cs.phase = "verifying".into();
                                        cs.verifier_task_id = Some(task_id.clone());
                                        if let Some(iter_rec) = cs.iterations.last_mut() {
                                            iter_rec.verifier_task_id = Some(task_id.clone());
                                        }
                                    }
                                    let _ = st.delegation_store.bind_graph_step_task(
                                        &task_id, &graph_id, &step_id, suppress,
                                    );
                                }
                                DaemonToWrapper::NativeDelegationResult { result } => {
                                    let code = result["code"]
                                        .as_str()
                                        .or_else(|| result["error"].as_str())
                                        .unwrap_or("verifier_admission_refused");
                                    let msg = result["message"].as_str().unwrap_or(code);
                                    current.steps[pos].state = "failed".into();
                                    current.steps[pos].error = Some(
                                        format!("convergence_verify_failed:{code}: {msg}")
                                            .chars()
                                            .take(512)
                                            .collect(),
                                    );
                                    current.steps[pos].finished_at = Some(now);
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                    {
                                        cs.phase = "done".into();
                                        cs.outcome = Some("failed".into());
                                    }
                                }
                                other => {
                                    current.steps[pos].state = "failed".into();
                                    current.steps[pos].error = Some(
                                        format!("convergence_verify_unexpected:{other:?}")
                                            .chars()
                                            .take(256)
                                            .collect(),
                                    );
                                    current.steps[pos].finished_at = Some(now);
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                    {
                                        cs.phase = "done".into();
                                        cs.outcome = Some("failed".into());
                                    }
                                }
                            }
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            let _ = st.delegation_store.save_graph(&current);
                        } else if let DaemonToWrapper::NativeDelegationResult { result } = response
                            && result["status"] != "refused"
                            && let Some(orphan_tid) = result["task_id"].as_str().map(str::to_owned)
                        {
                            // Étape ou graphe annulé pendant l'admission : éviter toute tâche orpheline.
                            drop(st);
                            let _ = cancel_task_for_graph(&graph_snapshot, &orphan_tid, &state_bg);
                        }
                    });
            }
            GraphLaunchJob::ConvergenceNextProduce {
                graph: graph_snapshot,
                idx,
                iteration,
                continue_task_id,
                continue_request_id,
                continue_task,
                fallback_delegation,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let suppress =
                    graph_snapshot.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
                let _ = thread::Builder::new()
                    .name(format!("bridget-graph-produce-{}", &step_id))
                    .spawn(move || {
                        let launch_key = format!("{graph_id}:{step_id}:produce:{iteration}");
                        let exec_tag = format!("{step_id}-produce-i{iteration}");
                        let mut continued: Option<(String, u32)> = None;
                        let mut fallback_response: Option<DaemonToWrapper> = None;

                        match prepare_exec_conn(&graph_snapshot, &exec_tag, &state_bg) {
                            Ok((exec_conn, _guard)) => {
                                if let Some(ptid) = continue_task_id.as_deref() {
                                    let cont_resp = super::native_delegation::handle_with_fact_pub(
                                        &exec_conn,
                                        NativeDelegationRequest::Continue {
                                            task_id: ptid.to_string(),
                                            request_id: continue_request_id,
                                            task: continue_task,
                                            brief: None,
                                        },
                                        graph_snapshot.attested_permissions.clone(),
                                        None,
                                        None,
                                        &state_bg,
                                        None,
                                    );
                                    if let DaemonToWrapper::NativeDelegationResult { result } =
                                        cont_resp
                                        && result["status"] != "refused"
                                    {
                                        let round = result["round"]
                                            .as_u64()
                                            .map(|r| r as u32)
                                            .unwrap_or(iteration as u32);
                                        continued = Some((ptid.to_string(), round));
                                    }
                                }
                                if continued.is_none() {
                                    fallback_response =
                                        Some(super::native_delegation::handle_with_fact_pub(
                                            &exec_conn,
                                            NativeDelegationRequest::AutoDelegateV2(
                                                fallback_delegation,
                                            ),
                                            graph_snapshot.attested_permissions.clone(),
                                            None,
                                            None,
                                            &state_bg,
                                            None,
                                        ));
                                }
                            }
                            Err(code) => {
                                fallback_response = Some(DaemonToWrapper::NativeDelegationResult {
                                    result: graph_failure(
                                        &code,
                                        "Contexte parent révoqué ou indisponible.",
                                    ),
                                });
                            }
                        }

                        let mut st = state_bg.lock().recover_poison();
                        st.graph_launching_steps.remove(&launch_key);
                        if let Ok(Some(mut current)) = st.delegation_store.get_graph(&graph_id)
                            && let Some(pos) =
                                current.steps.iter().position(|s| s.step_id == step_id)
                            && current.steps[pos].state == "running"
                            && current.steps[pos]
                                .convergence_state
                                .as_ref()
                                .is_some_and(|cs| cs.phase == "launching_produce")
                        {
                            let now = unix_timestamp();
                            if let Some((ptid, expected_round)) = continued {
                                current.steps[pos].task_id = Some(ptid.clone());
                                if let Some(cs) = current.steps[pos].convergence_state.as_mut() {
                                    cs.phase = "producing".into();
                                    cs.producer_task_id = Some(ptid.clone());
                                    cs.producer_expected_round = expected_round;
                                    cs.verifier_task_id = None;
                                    cs.iterations.push(NativeGraphConvergenceIteration {
                                        iteration,
                                        producer_task_id: Some(ptid),
                                        producer_round: expected_round,
                                        producer_commit: None,
                                        verifier_task_id: None,
                                        verdict: None,
                                        reason: None,
                                        blocking_points: Vec::new(),
                                        blocking_fingerprint: None,
                                        started_at: Some(now),
                                        finished_at: None,
                                    });
                                }
                            } else if let Some(response) = fallback_response {
                                match response {
                                    DaemonToWrapper::NativeDelegationResult { result }
                                        if result["status"] != "refused"
                                            && result["task_id"].as_str().is_some() =>
                                    {
                                        let new_tid =
                                            result["task_id"].as_str().unwrap().to_string();
                                        current.steps[pos].task_id = Some(new_tid.clone());
                                        if let Some(cs) =
                                            current.steps[pos].convergence_state.as_mut()
                                        {
                                            cs.phase = "producing".into();
                                            cs.producer_task_id = Some(new_tid.clone());
                                            cs.producer_expected_round = 1;
                                            cs.verifier_task_id = None;
                                            cs.iterations.push(NativeGraphConvergenceIteration {
                                                iteration,
                                                producer_task_id: Some(new_tid.clone()),
                                                producer_round: 1,
                                                producer_commit: None,
                                                verifier_task_id: None,
                                                verdict: None,
                                                reason: None,
                                                blocking_points: Vec::new(),
                                                blocking_fingerprint: None,
                                                started_at: Some(now),
                                                finished_at: None,
                                            });
                                        }
                                        let _ = st.delegation_store.bind_graph_step_task(
                                            &new_tid, &graph_id, &step_id, suppress,
                                        );
                                    }
                                    DaemonToWrapper::NativeDelegationResult { result } => {
                                        let code = result["code"]
                                            .as_str()
                                            .or_else(|| result["error"].as_str())
                                            .unwrap_or("producer_relaunch_refused");
                                        let msg = result["message"].as_str().unwrap_or(code);
                                        current.steps[pos].state = "failed".into();
                                        current.steps[pos].error = Some(
                                            format!("convergence_produce_failed:{code}: {msg}")
                                                .chars()
                                                .take(512)
                                                .collect(),
                                        );
                                        current.steps[pos].finished_at = Some(now);
                                        if let Some(cs) =
                                            current.steps[pos].convergence_state.as_mut()
                                        {
                                            cs.phase = "done".into();
                                            cs.outcome = Some("failed".into());
                                        }
                                    }
                                    other => {
                                        current.steps[pos].state = "failed".into();
                                        current.steps[pos].error = Some(
                                            format!("convergence_produce_unexpected:{other:?}")
                                                .chars()
                                                .take(256)
                                                .collect(),
                                        );
                                        current.steps[pos].finished_at = Some(now);
                                        if let Some(cs) =
                                            current.steps[pos].convergence_state.as_mut()
                                        {
                                            cs.phase = "done".into();
                                            cs.outcome = Some("failed".into());
                                        }
                                    }
                                }
                            }
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            let _ = st.delegation_store.save_graph(&current);
                        } else {
                            let orphan_tid = continued.map(|(ptid, _)| ptid).or_else(|| {
                                fallback_response.and_then(|resp| match resp {
                                    DaemonToWrapper::NativeDelegationResult { result }
                                        if result["status"] != "refused" =>
                                    {
                                        result["task_id"].as_str().map(str::to_owned)
                                    }
                                    _ => None,
                                })
                            });
                            drop(st);
                            if let Some(tid) = orphan_tid {
                                let _ = cancel_task_for_graph(&graph_snapshot, &tid, &state_bg);
                            }
                        }
                    });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delegation::DelegationStore;
    use bridget_transport::protocol::GraphConvergenceLoop;
    use bridget_transport::protocol::{AutoDelegateWorkload, DelegateGraphStep};

    fn sample_auto(task: &str) -> AutoDelegateV2 {
        AutoDelegateV2 {
            request_id: String::new(),
            machine_id: None,
            project_id: None,
            engine: Some("codex".into()),
            connection_id: None,
            model: None,
            effort: None,
            service_tier: None,
            workload: AutoDelegateWorkload::General,
            expected_base_commit: None,
            task: task.into(),
            cwd: "/tmp/demo".into(),
            posture: None,
            reply_timeout: None,
            workspace: None,
            keep_alive_secs: None,
            brief: None,
            child_requests: None,
            urgent: None,
            deliverable: None,
            quota_fallback_authorized: None,
            resume_on_limit: None,
            agent_type: None,
        }
    }

    fn sample_step(id: &str, deps: &[&str], task: &str) -> DelegateGraphStep {
        DelegateGraphStep {
            step_id: id.into(),
            depends_on: deps.iter().map(|s| (*s).into()).collect(),
            pass_dependency_results: true,
            delegation: sample_auto(task),
            convergence: None,
        }
    }

    #[test]
    fn dag_diamond_topological_order_and_parallel_unlock() {
        let req = DelegateGraphRequest {
            request_id: "dag-1".into(),
            title: Some("Build puis tests parallèles puis synthèse".into()),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            steps: vec![
                sample_step("D", &["B", "C"], "Synthèse finale"),
                sample_step("B", &["A"], "Tests unitaires"),
                sample_step("C", &["A"], "Lint + clippy"),
                sample_step("A", &[], "Compilation initiale"),
            ],
        };
        let order = validate_and_order_graph(&req).expect("DAG valide");
        assert_eq!(order[0], "A");
        assert_eq!(order[3], "D");
        assert!(order[1..3].contains(&"B".to_string()));
        assert!(order[1..3].contains(&"C".to_string()));

        let mut graph = NativeGraphRecord {
            graph_id: "graph-test".into(),
            owner: "owner-1".into(),
            owner_instance: "inst-1".into(),
            request_id: "dag-1".into(),
            title: req.title.clone(),
            on_failure: req.on_failure,
            notify_policy: req.notify_policy,
            state: "running".into(),
            topological_order: order,
            steps: req
                .steps
                .into_iter()
                .map(|s| NativeGraphStepRecord {
                    step_id: s.step_id,
                    depends_on: s.depends_on,
                    pass_dependency_results: s.pass_dependency_results,
                    delegation: s.delegation,
                    convergence: s.convergence,
                    convergence_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                })
                .collect(),
            attested_permissions: None,
            parent_thread: None,
            communication_project: None,
            notice_sent: false,
            canonical: vec![1],
            created_at: 100,
            updated_at: 100,
        };

        assert_eq!(ready_waiting_steps(&graph), vec!["A"]);
        // A termine : B et C se débloquent en parallèle !
        let a = graph.steps.iter_mut().find(|s| s.step_id == "A").unwrap();
        a.state = "completed".into();
        a.task_id = Some("task-a".into());
        a.result_excerpt = Some("WASM compilé avec succès".into());
        a.factual_summary = Some(json!({"commit": "deadbeef", "files_changed": 3}));
        assert_eq!(ready_waiting_steps(&graph), vec!["B", "C"]);

        // Vérifier l'enrichissement de la consigne de B avec le résultat factuel de A.
        let b = graph.steps.iter().find(|s| s.step_id == "B").unwrap();
        let enriched = build_enriched_step_task(b, &graph.steps);
        assert!(enriched.contains(
            "Étape `A` : état=completed, task_id=task-a, commit=deadbeef, fichiers_modifies=3"
        ));
        assert!(enriched.contains("WASM compilé avec succès"));
        assert!(enriched.contains("Tests unitaires"));
    }

    #[test]
    fn dag_rejects_cycles_and_unknown_dependencies() {
        let cycle = DelegateGraphRequest {
            request_id: "dag-cycle".into(),
            title: None,
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            steps: vec![
                sample_step("A", &["B"], "step A"),
                sample_step("B", &["A"], "step B"),
            ],
        };
        let err = validate_and_order_graph(&cycle).unwrap_err();
        assert_eq!(err.0, "graph_cycle_detected");

        let unknown = DelegateGraphRequest {
            request_id: "dag-unknown".into(),
            title: None,
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            steps: vec![sample_step("A", &["Z"], "step A")],
        };
        let err = validate_and_order_graph(&unknown).unwrap_err();
        assert_eq!(err.0, "graph_invalid");
    }

    #[test]
    fn failure_propagation_stop_dependents_vs_cancel_graph() {
        let mk = |policy: GraphFailurePolicy| NativeGraphRecord {
            graph_id: "graph-fail".into(),
            owner: "owner-1".into(),
            owner_instance: "inst-1".into(),
            request_id: "req-fail".into(),
            title: None,
            on_failure: policy,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            state: "running".into(),
            topological_order: vec!["A".into(), "B".into(), "C".into(), "X".into()],
            steps: vec![
                NativeGraphStepRecord {
                    step_id: "A".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    delegation: sample_auto("A"),
                    convergence: None,
                    convergence_state: None,
                    state: "failed".into(),
                    task_id: Some("t-a".into()),
                    error: Some("boom".into()),
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: Some(1),
                    finished_at: Some(2),
                },
                NativeGraphStepRecord {
                    step_id: "B".into(),
                    depends_on: vec!["A".into()],
                    pass_dependency_results: true,
                    delegation: sample_auto("B"),
                    convergence: None,
                    convergence_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                },
                NativeGraphStepRecord {
                    step_id: "C".into(),
                    depends_on: vec!["B".into()],
                    pass_dependency_results: true,
                    delegation: sample_auto("C"),
                    convergence: None,
                    convergence_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                },
                NativeGraphStepRecord {
                    step_id: "X".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    delegation: sample_auto("X"),
                    convergence: None,
                    convergence_state: None,
                    state: "running".into(),
                    task_id: Some("t-x".into()),
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: Some(1),
                    finished_at: None,
                },
            ],
            attested_permissions: None,
            parent_thread: None,
            communication_project: None,
            notice_sent: false,
            canonical: vec![1],
            created_at: 1,
            updated_at: 1,
        };

        // StopDependents : B et C passent skipped_dependency_failed, X continue !
        let mut g1 = mk(GraphFailurePolicy::StopDependents);
        let cancelled = propagate_graph_state(&mut g1, 10);
        assert!(cancelled.is_empty());
        assert_eq!(g1.steps[1].state, "skipped_dependency_failed");
        assert_eq!(g1.steps[2].state, "skipped_dependency_failed");
        assert_eq!(g1.steps[3].state, "running");
        assert_eq!(g1.state, "running");

        // CancelGraph : X est demandé en annulation, B et C passent cancelled !
        let mut g2 = mk(GraphFailurePolicy::CancelGraph);
        let cancelled = propagate_graph_state(&mut g2, 10);
        assert_eq!(cancelled, vec!["t-x"]);
        assert_eq!(g2.steps[1].state, "cancelled");
        assert_eq!(g2.steps[2].state, "cancelled");
    }

    #[test]
    fn sqlite_graph_store_roundtrip_and_notice_suppression() {
        let dir = std::env::temp_dir().join(format!(
            "bridget-graph-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let store = DelegationStore::open(&dir.join("bridget.db")).unwrap();
        let rec = NativeGraphRecord {
            graph_id: "graph-sql-1".into(),
            owner: "agent-a".into(),
            owner_instance: "inst-a".into(),
            request_id: "req-sql-1".into(),
            title: Some("Pipeline".into()),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            state: "running".into(),
            topological_order: vec!["A".into()],
            steps: vec![],
            attested_permissions: None,
            parent_thread: None,
            communication_project: None,
            notice_sent: false,
            canonical: b"{\"a\":1}".to_vec(),
            created_at: 10,
            updated_at: 10,
        };
        store.save_graph(&rec).unwrap();
        let loaded = store.get_graph("graph-sql-1").unwrap().unwrap();
        assert_eq!(loaded.title.as_deref(), Some("Pipeline"));
        let by_req = store
            .graph_by_owner_request("agent-a", "req-sql-1")
            .unwrap()
            .unwrap();
        assert_eq!(by_req.graph_id, "graph-sql-1");
        assert_eq!(store.active_graphs().unwrap().len(), 1);

        assert!(!store.graph_suppresses_step_notice("task-42").unwrap());
        store
            .bind_graph_step_task("task-42", "graph-sql-1", "A", true)
            .unwrap();
        assert!(store.graph_suppresses_step_notice("task-42").unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn convergence_validation_bounds_and_verdict_parser() {
        let mut bad = sample_step("B", &[], "impl");
        bad.convergence = Some(GraphConvergenceLoop {
            verify: sample_auto("review"),
            max_iterations: 1,
            stop_on_stagnation: true,
        });
        let req = DelegateGraphRequest {
            request_id: "conv-bad-min".into(),
            title: None,
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            steps: vec![bad],
        };
        assert_eq!(
            validate_and_order_graph(&req).unwrap_err().0,
            "graph_invalid"
        );

        // Verdict approve en ligne autonome et sous bloc markdown json.
        assert_eq!(
            parse_convergence_verdict(
                "Analyse OK.\n{\"verdict\":\"approve\",\"reason\":\"tests verts\"}"
            ),
            ParsedVerdict::Approve {
                reason: "tests verts".into()
            }
        );
        let fenced = "Revue complète.\n```json\n{\"verdict\":\"revise\",\"reason\":\"cas limite\",\"blocking_points\":[\"Ajouter test nil\",\"Fixer clippy\"]}\n```\n";
        let ParsedVerdict::Revise {
            reason,
            blocking_points,
            fingerprint: fp1,
        } = parse_convergence_verdict(fenced)
        else {
            panic!("revise attendu");
        };
        assert_eq!(reason, "cas limite");
        assert_eq!(blocking_points.len(), 2);
        // Même motifs dans un autre ordre + casse/espaces -> même empreinte de stagnation.
        let ParsedVerdict::Revise {
            fingerprint: fp2, ..
        } = parse_convergence_verdict(
            "{\"verdict\":\"revise\",\"reason\":\"autre\",\"blocking_points\":[\"  fixer   clippy \",\"AJOUTER TEST NIL\"]}",
        )
        else {
            panic!("revise attendu");
        };
        assert_eq!(fp1, fp2);

        // Escalate et fail-closed sur texte libre sans JSON fermé.
        assert_eq!(
            parse_convergence_verdict(
                "{\"verdict\":\"escalate\",\"reason\":\"arbitrage produit requis\"}"
            ),
            ParsedVerdict::Escalate {
                reason: "arbitrage produit requis".into()
            }
        );
        assert_eq!(
            parse_convergence_verdict("Tout semble APPROUVE mais sans objet JSON fermé."),
            ParsedVerdict::Unparseable
        );
        assert_eq!(
            parse_convergence_verdict("{\"verdict\":\"unknown\",\"reason\":\"x\"}"),
            ParsedVerdict::Unparseable
        );

        // Clé d'itération toujours <= 128 octets même avec longs identifiants.
        let long_req = "r".repeat(120);
        let long_step = "s".repeat(60);
        let derived = convergence_step_request_id(&long_req, &long_step, 3, "verify");
        assert!(derived.len() <= 128, "len={}", derived.len());
    }

    #[test]
    fn convergence_state_machine_revise_approve_stagnation_escalate_and_max_rounds() {
        let mk_step = |max_iterations: u8, stop_on_stagnation: bool| NativeGraphStepRecord {
            step_id: "build-review".into(),
            depends_on: vec![],
            pass_dependency_results: true,
            delegation: sample_auto("impl"),
            convergence: Some(GraphConvergenceLoop {
                verify: sample_auto("review"),
                max_iterations,
                stop_on_stagnation,
            }),
            convergence_state: Some(NativeGraphConvergenceState {
                current_iteration: 1,
                max_iterations,
                stop_on_stagnation,
                phase: "verifying".into(),
                producer_task_id: Some("prod-1".into()),
                producer_expected_round: 1,
                verifier_task_id: Some("ver-1".into()),
                last_producer_commit: Some("a".repeat(40)),
                last_blocking_fingerprint: None,
                outcome: None,
                iterations: vec![NativeGraphConvergenceIteration {
                    iteration: 1,
                    producer_task_id: Some("prod-1".into()),
                    producer_round: 1,
                    producer_commit: Some("a".repeat(40)),
                    verifier_task_id: Some("ver-1".into()),
                    verdict: None,
                    reason: None,
                    blocking_points: Vec::new(),
                    blocking_fingerprint: None,
                    started_at: Some(10),
                    finished_at: None,
                }],
            }),
            state: "running".into(),
            task_id: Some("ver-1".into()),
            error: None,
            result_excerpt: Some("code prêt".into()),
            factual_summary: Some(json!({"commit": "a".repeat(40), "files_changed": 2})),
            started_at: Some(10),
            finished_at: None,
        };

        // 1) Nominal 2 tours : tour 1 revise -> tour 2 approve -> completed + task_id producteur.
        let mut step = mk_step(3, true);
        let v1 = parse_convergence_verdict(
            "{\"verdict\":\"revise\",\"reason\":\"manque test\",\"blocking_points\":[\"test_edge\"]}",
        );
        apply_verifier_verdict(&mut step, v1, 20);
        assert_eq!(step.state, "running");
        assert_eq!(
            step.convergence_state.as_ref().unwrap().current_iteration,
            2
        );
        assert_eq!(
            step.convergence_state.as_ref().unwrap().phase,
            "pending_produce"
        );
        // Simuler tour 2 terminé et vérifié -> approve.
        step.convergence_state.as_mut().unwrap().phase = "verifying".into();
        step.convergence_state
            .as_mut()
            .unwrap()
            .iterations
            .push(NativeGraphConvergenceIteration {
                iteration: 2,
                producer_task_id: Some("prod-1".into()),
                producer_round: 2,
                producer_commit: Some("b".repeat(40)),
                verifier_task_id: Some("ver-2".into()),
                verdict: None,
                reason: None,
                blocking_points: Vec::new(),
                blocking_fingerprint: None,
                started_at: Some(25),
                finished_at: None,
            });
        apply_verifier_verdict(
            &mut step,
            ParsedVerdict::Approve {
                reason: "conforme".into(),
            },
            30,
        );
        assert_eq!(step.state, "completed");
        assert_eq!(step.task_id.as_deref(), Some("prod-1"));
        assert_eq!(
            step.convergence_state.as_ref().unwrap().outcome.as_deref(),
            Some("approved")
        );

        // 2) Stagnation sur mêmes blocking_points dès le tour 2 (avant max_iterations=4).
        let mut stag = mk_step(4, true);
        let same_revise = parse_convergence_verdict(
            "{\"verdict\":\"revise\",\"reason\":\"toujours cassé\",\"blocking_points\":[\"erreur X\"]}",
        );
        apply_verifier_verdict(&mut stag, same_revise.clone(), 20);
        assert_eq!(
            stag.convergence_state.as_ref().unwrap().current_iteration,
            2
        );
        stag.convergence_state.as_mut().unwrap().phase = "verifying".into();
        stag.convergence_state
            .as_mut()
            .unwrap()
            .iterations
            .push(NativeGraphConvergenceIteration {
                iteration: 2,
                producer_task_id: Some("prod-1".into()),
                producer_round: 2,
                producer_commit: Some("c".repeat(40)),
                verifier_task_id: Some("ver-2".into()),
                verdict: None,
                reason: None,
                blocking_points: Vec::new(),
                blocking_fingerprint: None,
                started_at: Some(25),
                finished_at: None,
            });
        apply_verifier_verdict(&mut stag, same_revise, 30);
        assert_eq!(stag.state, "failed");
        assert_eq!(
            stag.convergence_state.as_ref().unwrap().outcome.as_deref(),
            Some("stagnated")
        );
        assert!(
            stag.error
                .as_deref()
                .unwrap()
                .contains("convergence_stagnated")
        );

        // 3) Escalade immédiate au tour 1.
        let mut esc = mk_step(3, true);
        apply_verifier_verdict(
            &mut esc,
            ParsedVerdict::Escalate {
                reason: "spec ambiguë".into(),
            },
            20,
        );
        assert_eq!(esc.state, "failed");
        assert_eq!(
            esc.convergence_state.as_ref().unwrap().outcome.as_deref(),
            Some("escalated")
        );
        assert!(
            esc.error
                .as_deref()
                .unwrap()
                .contains("convergence_escalated")
        );

        // 4) Plafond max_iterations=2 avec motifs distincts -> max_iterations_reached.
        let mut capped = mk_step(2, true);
        apply_verifier_verdict(
            &mut capped,
            parse_convergence_verdict(
                "{\"verdict\":\"revise\",\"reason\":\"pb 1\",\"blocking_points\":[\"a\"]}",
            ),
            20,
        );
        assert_eq!(capped.state, "running");
        capped.convergence_state.as_mut().unwrap().phase = "verifying".into();
        apply_verifier_verdict(
            &mut capped,
            parse_convergence_verdict(
                "{\"verdict\":\"revise\",\"reason\":\"pb 2\",\"blocking_points\":[\"b\"]}",
            ),
            30,
        );
        assert_eq!(capped.state, "failed");
        assert_eq!(
            capped
                .convergence_state
                .as_ref()
                .unwrap()
                .outcome
                .as_deref(),
            Some("max_iterations_reached")
        );

        // 5) Verdict illisible -> fail-closed verdict_unparseable.
        let mut bad = mk_step(3, true);
        apply_verifier_verdict(&mut bad, ParsedVerdict::Unparseable, 20);
        assert_eq!(bad.state, "failed");
        assert_eq!(
            bad.convergence_state.as_ref().unwrap().outcome.as_deref(),
            Some("verdict_unparseable")
        );

        // 6) DAG A -> B(convergence) -> D : D attend l'approbation de B et est sauté sur stagnation.
        let mut dag_ok = NativeGraphRecord {
            graph_id: "g-conv-dag".into(),
            request_id: "r-conv-dag".into(),
            owner: "owner".into(),
            owner_instance: "inst".into(),
            parent_thread: None,
            communication_project: None,
            attested_permissions: None,
            title: Some("A->B(conv)->D".into()),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            state: "running".into(),
            topological_order: vec!["A".into(), "B".into(), "D".into()],
            steps: vec![
                NativeGraphStepRecord {
                    step_id: "A".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    delegation: sample_auto("prep"),
                    convergence: None,
                    convergence_state: None,
                    state: "completed".into(),
                    task_id: Some("t-a".into()),
                    error: None,
                    result_excerpt: Some("prep ok".into()),
                    factual_summary: None,
                    started_at: Some(1),
                    finished_at: Some(5),
                },
                {
                    let mut b = mk_step(3, true);
                    b.step_id = "B".into();
                    b.depends_on = vec!["A".into()];
                    b
                },
                NativeGraphStepRecord {
                    step_id: "D".into(),
                    depends_on: vec!["B".into()],
                    pass_dependency_results: true,
                    delegation: sample_auto("deploy"),
                    convergence: None,
                    convergence_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                },
            ],
            canonical: vec![1],
            notice_sent: false,
            created_at: 1,
            updated_at: 10,
        };
        assert!(ready_waiting_steps(&dag_ok).is_empty());
        apply_verifier_verdict(
            &mut dag_ok.steps[1],
            ParsedVerdict::Approve {
                reason: "ok".into(),
            },
            25,
        );
        assert_eq!(ready_waiting_steps(&dag_ok), vec!["D"]);

        // 7) on_failure=cancel_graph annule producteur ET vérificateur d'une étape en phase verifying.
        let mut dag_cancel = dag_ok;
        dag_cancel.on_failure = GraphFailurePolicy::CancelGraph;
        dag_cancel.steps[0].state = "failed".into();
        dag_cancel.steps[1] = mk_step(3, true);
        dag_cancel.steps[1].step_id = "B".into();
        let to_cancel = propagate_graph_state(&mut dag_cancel, 40);
        assert!(to_cancel.contains(&"ver-1".to_string()));
        assert!(to_cancel.contains(&"prod-1".to_string()));
        assert_eq!(dag_cancel.steps[2].state, "cancelled");

        // 8) Stagnation #1 : même commit produit au tour 2 -> arrêt immédiat avant vérificateur.
        let mut commit_stag = mk_step(4, true);
        let cs = commit_stag.convergence_state.as_mut().unwrap();
        cs.current_iteration = 2;
        cs.phase = "producing".into();
        cs.last_producer_commit = Some("a".repeat(40));
        apply_producer_iteration_result(
            &mut commit_stag,
            2,
            Some("a".repeat(40)),
            Some("inchangé".into()),
            json!({"commit": "a".repeat(40)}),
            50,
        );
        assert_eq!(commit_stag.state, "failed");
        assert_eq!(
            commit_stag
                .convergence_state
                .as_ref()
                .unwrap()
                .outcome
                .as_deref(),
            Some("stagnated")
        );
        assert!(
            commit_stag
                .error
                .as_deref()
                .unwrap()
                .contains("commit identique")
        );
    }
}
