//! Orchestration autonome de graphes orientés acycliques (DAG) de missions Bridget.
//!
//! Permet à un chef d'orchestre de déclarer un enchaînement (`A -> B + C -> D`) en un seul
//! appel (`bridget_delegate_graph`) puis de s'endormir sans ronde de garde périodique :
//! le daemon suit l'état durable de chaque étape, déclenche automatiquement les étapes
//! suivantes en parallèle dès que leurs dépendances sont terminées, propage les échecs selon
//! `on_failure`, et réveille le parent une seule fois avec le bilan consolidé.

use super::*;
use crate::convergence_metrics::ConvergenceMetricRawRow;
use crate::delegation::{
    NativeGraphBlackboardEntry, NativeGraphBudgetUsage, NativeGraphConvergenceIteration,
    NativeGraphConvergenceState, NativeGraphEscalationCandidate, NativeGraphMapItemRecord,
    NativeGraphMapState, NativeGraphMapWorkerSlot, NativeGraphRecord, NativeGraphStepRecord,
};
use bridget_transport::protocol::{
    AutoDelegateV2, ConvergenceMetricsPolicyUpdate, DelegateGraphRequest, GraphBudget,
    GraphConvergenceEscalation, GraphDeterministicGate, GraphFailurePolicy, GraphMapSpec,
    GraphModelTier, GraphNotifyPolicy, GraphStepCondition, MAX_CONVERGENCE_ITERATIONS,
    MAX_ESCALATION_CANDIDATES, MAX_ESCALATION_CANDIDATES_WITHOUT_GATE,
    MAX_ESCALATION_EXTRA_ITERATIONS, MAX_GRAPH_BUDGET_DURATION_SECS,
    MAX_GRAPH_BUDGET_FRONTIER_ROUNDS, MAX_GRAPH_BUDGET_ROUNDS, MAX_GRAPH_MAP_CONCURRENCY,
    MAX_GRAPH_MAP_ITEM_BYTES, MAX_GRAPH_MAP_ITEMS, MAX_TOTAL_CONVERGENCE_ITERATIONS,
    MIN_CONVERGENCE_ITERATIONS, MIN_ESCALATION_EXTRA_ITERATIONS, MIN_GRAPH_BUDGET_DURATION_SECS,
    MIN_GRAPH_MAP_ITEMS, NativeDelegationRequest, ProviderPermissions, SpawnPosture,
    WorkspaceRequest,
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
const MAX_BLACKBOARD_KEYS: usize = 32;
const MAX_BLACKBOARD_KEY_LEN: usize = 64;
const MAX_BLACKBOARD_VALUE_BYTES: usize = 4096;
const MAX_BLACKBOARD_READS_PER_STEP: usize = 16;
const MAX_BLACKBOARD_WRITES_PER_STEP: usize = 8;

/// Issues d'échec autorisées dans `when_outcomes` (jamais `"cancelled"` : une annulation humaine
/// ne déclenche jamais une branche de repli automatique).
pub(crate) const ALLOWED_WHEN_OUTCOMES: &[&str] = &[
    "failed",
    "stagnated",
    "max_iterations_reached",
    "escalated",
    "gate_unavailable",
    "verdict_unparseable",
    "budget_exhausted",
];

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
        "completed"
            | "failed"
            | "cancelled"
            | "skipped_dependency_failed"
            | "skipped_condition_not_met"
    )
}

/// Issue normalisée d'une dépendance en échec pour l'évaluation des branches de repli.
/// Renvoie `None` pour `completed`, `cancelled` (annulation humaine) et les états sautés.
fn step_dependency_failure_outcome(dep: &NativeGraphStepRecord) -> Option<&str> {
    if dep.state != "failed" {
        return None;
    }
    if let Some(outcome) = dep
        .convergence_state
        .as_ref()
        .and_then(|cs| cs.outcome.as_deref())
    {
        return Some(outcome);
    }
    if dep
        .error
        .as_deref()
        .is_some_and(|e| e.starts_with("budget_exhausted"))
    {
        return Some("budget_exhausted");
    }
    Some("failed")
}

fn valid_blackboard_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_BLACKBOARD_KEY_LEN
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn validate_graph_budget(budget: &GraphBudget) -> Result<(), (String, String)> {
    if let Some(total) = budget.max_total_rounds
        && !(1..=MAX_GRAPH_BUDGET_ROUNDS).contains(&total)
    {
        return Err((
            "graph_invalid".into(),
            format!(
                "budget.max_total_rounds doit être compris entre 1 et {MAX_GRAPH_BUDGET_ROUNDS}."
            ),
        ));
    }
    if let Some(frontier) = budget.max_frontier_rounds
        && frontier > MAX_GRAPH_BUDGET_FRONTIER_ROUNDS
    {
        return Err((
            "graph_invalid".into(),
            format!(
                "budget.max_frontier_rounds doit être compris entre 0 et {MAX_GRAPH_BUDGET_FRONTIER_ROUNDS}."
            ),
        ));
    }
    if let Some(secs) = budget.max_duration_secs
        && !(MIN_GRAPH_BUDGET_DURATION_SECS..=MAX_GRAPH_BUDGET_DURATION_SECS).contains(&secs)
    {
        return Err((
            "graph_invalid".into(),
            format!(
                "budget.max_duration_secs doit être compris entre {MIN_GRAPH_BUDGET_DURATION_SECS} et {MAX_GRAPH_BUDGET_DURATION_SECS}."
            ),
        ));
    }
    if let (Some(total), Some(frontier)) = (budget.max_total_rounds, budget.max_frontier_rounds)
        && frontier > total
    {
        return Err((
            "graph_invalid".into(),
            format!(
                "budget.max_frontier_rounds ({frontier}) ne peut pas dépasser budget.max_total_rounds ({total})."
            ),
        ));
    }
    Ok(())
}

/// Valide la structure DAG (1..=16 étapes, `step_id` uniques, dépendances existantes,
/// absence de cycles via tri topologique de Kahn, portes déterministes, escalade, branches de repli)
/// et renvoie l'ordre topologique déterministe.
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

    if let Some(budget) = &request.budget {
        validate_graph_budget(budget)?;
    }

    let has_fallback_branch = request
        .steps
        .iter()
        .any(|s| s.when != GraphStepCondition::AllSucceeded);
    if has_fallback_branch && request.on_failure == GraphFailurePolicy::CancelGraph {
        return Err((
            "graph_invalid".into(),
            "on_failure='cancel_graph' est incompatible avec une étape conditionnelle (when != 'all_succeeded') : utilisez on_failure='stop_dependents' pour permettre aux branches de repli de s'exécuter.".into(),
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
        if step.blackboard_reads.len() > MAX_BLACKBOARD_READS_PER_STEP {
            return Err((
                "graph_invalid".into(),
                format!(
                    "étape '{}' : blackboard_reads accepte au plus {MAX_BLACKBOARD_READS_PER_STEP} clés.",
                    step.step_id
                ),
            ));
        }
        for key in &step.blackboard_reads {
            if !valid_blackboard_key(key) {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : clé blackboard_reads invalide '{key}'.",
                        step.step_id
                    ),
                ));
            }
        }
        if step.blackboard_writes.len() > MAX_BLACKBOARD_WRITES_PER_STEP {
            return Err((
                "graph_invalid".into(),
                format!(
                    "étape '{}' : blackboard_writes accepte au plus {MAX_BLACKBOARD_WRITES_PER_STEP} clés.",
                    step.step_id
                ),
            ));
        }
        for key in &step.blackboard_writes {
            if !valid_blackboard_key(key) {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : clé blackboard_writes invalide '{key}'.",
                        step.step_id
                    ),
                ));
            }
        }
        match step.when {
            GraphStepCondition::AllSucceeded => {
                if !step.when_outcomes.is_empty() {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : when_outcomes ne peut être renseigné qu'avec when='on_outcome'.",
                            step.step_id
                        ),
                    ));
                }
            }
            GraphStepCondition::AnyFailed => {
                if step.depends_on.is_empty() {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : when='any_failed' exige au moins une dépendance dans depends_on.",
                            step.step_id
                        ),
                    ));
                }
                if !step.when_outcomes.is_empty() {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : when_outcomes ne peut être renseigné qu'avec when='on_outcome'.",
                            step.step_id
                        ),
                    ));
                }
            }
            GraphStepCondition::OnOutcome => {
                if step.depends_on.is_empty() {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : when='on_outcome' exige au moins une dépendance dans depends_on.",
                            step.step_id
                        ),
                    ));
                }
                if step.when_outcomes.is_empty() || step.when_outcomes.len() > 8 {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : when='on_outcome' exige 1 à 8 issues dans when_outcomes.",
                            step.step_id
                        ),
                    ));
                }
                for outcome in &step.when_outcomes {
                    if !ALLOWED_WHEN_OUTCOMES.contains(&outcome.as_str()) {
                        return Err((
                            "graph_invalid".into(),
                            format!(
                                "étape '{}' : when_outcomes contient une issue invalide '{}' (autorisées : {}).",
                                step.step_id,
                                outcome,
                                ALLOWED_WHEN_OUTCOMES.join(", ")
                            ),
                        ));
                    }
                }
            }
        }
        if step.map.is_some() && step.convergence.is_some() {
            return Err((
                "graph_invalid".into(),
                format!(
                    "étape '{}' : map et convergence sont mutuellement exclusifs sur une même étape.",
                    step.step_id
                ),
            ));
        }
        if let Some(map) = &step.map {
            if !(MIN_GRAPH_MAP_ITEMS..=MAX_GRAPH_MAP_ITEMS).contains(&map.items.len()) {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : map.items doit contenir entre {MIN_GRAPH_MAP_ITEMS} et {MAX_GRAPH_MAP_ITEMS} éléments.",
                        step.step_id
                    ),
                ));
            }
            for (item_idx, item) in map.items.iter().enumerate() {
                if item.trim().is_empty() || item.len() > MAX_GRAPH_MAP_ITEM_BYTES {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : map.items[{item_idx}] doit contenir entre 1 et {MAX_GRAPH_MAP_ITEM_BYTES} octets.",
                            step.step_id
                        ),
                    ));
                }
            }
            if !(1..=MAX_GRAPH_MAP_CONCURRENCY).contains(&map.max_concurrency) {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : map.max_concurrency doit être compris entre 1 et {MAX_GRAPH_MAP_CONCURRENCY}.",
                        step.step_id
                    ),
                ));
            }
            if map.max_concurrency > 1
                && step.delegation.posture != Some(SpawnPosture::Discovery)
                && matches!(step.delegation.workspace, Some(WorkspaceRequest::Shared {}))
            {
                return Err((
                    "graph_invalid".into(),
                    format!(
                        "étape '{}' : map avec max_concurrency > 1 en écriture est interdit avec workspace='shared' (risque de collision concurrente). Utilisez workspace='worktree', posture='discovery' ou max_concurrency=1.",
                        step.step_id
                    ),
                ));
            }
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
            if let Some(gate) = &conv.gate {
                crate::deterministic_gate::validate_gate_spec(gate).map_err(|msg| {
                    (
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : convergence.gate invalide : {msg}",
                            step.step_id
                        ),
                    )
                })?;
                if step.delegation.posture == Some(SpawnPosture::Discovery) {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : convergence.gate exige une étape productrice en posture 'development' (interdit en 'discovery').",
                            step.step_id
                        ),
                    ));
                }
            }
            if let Some(esc) = &conv.escalation {
                validate_escalation_spec(
                    &step.step_id,
                    conv.max_iterations,
                    conv.stop_on_stagnation,
                    conv.gate.is_some(),
                    esc,
                )?;
                if esc.candidates > 1
                    && step.delegation.posture != Some(SpawnPosture::Discovery)
                    && matches!(step.delegation.workspace, Some(WorkspaceRequest::Shared {}))
                {
                    return Err((
                        "graph_invalid".into(),
                        format!(
                            "étape '{}' : convergence.escalation.candidates={} exige une isolation par worktree (interdit avec workspace='shared' hors posture 'discovery').",
                            step.step_id, esc.candidates
                        ),
                    ));
                }
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

pub(crate) fn validate_escalation_spec(
    step_id: &str,
    base_max_iterations: u8,
    stop_on_stagnation: bool,
    has_gate: bool,
    esc: &GraphConvergenceEscalation,
) -> Result<(), (String, String)> {
    if !(1..=MAX_ESCALATION_CANDIDATES).contains(&esc.candidates) {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : convergence.escalation.candidates doit être compris entre 1 et {MAX_ESCALATION_CANDIDATES}."
            ),
        ));
    }
    if !has_gate && esc.candidates > MAX_ESCALATION_CANDIDATES_WITHOUT_GATE {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : convergence.escalation.candidates={} exige une porte déterministe convergence.gate (plafonné à {MAX_ESCALATION_CANDIDATES_WITHOUT_GATE} sans porte).",
                esc.candidates
            ),
        ));
    }
    let has_override = esc.engine.as_deref().is_some_and(|s| !s.trim().is_empty())
        || esc.model.as_deref().is_some_and(|s| !s.trim().is_empty())
        || esc.effort.as_deref().is_some_and(|s| !s.trim().is_empty())
        || esc
            .machine_id
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
        || esc.candidates > 1;
    if !has_override {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : convergence.escalation exige au moins une surcharge (engine, model, effort, machine_id ou candidates > 1)."
            ),
        ));
    }
    if !(MIN_ESCALATION_EXTRA_ITERATIONS..=MAX_ESCALATION_EXTRA_ITERATIONS)
        .contains(&esc.extra_iterations)
    {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : convergence.escalation.extra_iterations doit être compris entre {MIN_ESCALATION_EXTRA_ITERATIONS} et {MAX_ESCALATION_EXTRA_ITERATIONS}."
            ),
        ));
    }
    if base_max_iterations.saturating_add(esc.extra_iterations) > MAX_TOTAL_CONVERGENCE_ITERATIONS {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : max_iterations ({base_max_iterations}) + extra_iterations ({}) dépasse la borne totale de {MAX_TOTAL_CONVERGENCE_ITERATIONS}.",
                esc.extra_iterations
            ),
        ));
    }
    if !esc.on_stagnation && !esc.on_max_iterations {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : convergence.escalation exige au moins un déclencheur (on_stagnation ou on_max_iterations)."
            ),
        ));
    }
    if !stop_on_stagnation && esc.on_stagnation {
        return Err((
            "graph_invalid".into(),
            format!(
                "étape '{step_id}' : convergence.escalation.on_stagnation=true est incompatible avec stop_on_stagnation=false."
            ),
        ));
    }
    Ok(())
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
/// (`produce` ou `verify`) d'une étape de convergence bornée, isolée par palier `tier` (`0` ou `1`).
pub(crate) fn convergence_step_request_id(
    graph_request_id: &str,
    step_id: &str,
    tier: u8,
    iteration: u8,
    role: &str,
) -> String {
    let tier_seg = if tier == 0 {
        String::new()
    } else {
        format!(":t{tier}")
    };
    let raw = format!("{graph_request_id}:{step_id}{tier_seg}:i{iteration}:{role}");
    if raw.len() <= 128 {
        return raw;
    }
    let digest = format!("{:x}", Sha256::digest(raw.as_bytes()));
    let suffix = format!("{tier_seg}:i{iteration}:{role}:{}", &digest[..12]);
    let prefix_len = 128usize.saturating_sub(suffix.len());
    let prefix: String = raw.chars().take(prefix_len).collect();
    format!("{prefix}{suffix}")
}

/// Clé déterministe et bornée (`<= 128` octets) pour chaque élément d'une étape Map-Reduce.
pub(crate) fn map_item_request_id(
    graph_request_id: &str,
    step_id: &str,
    item_index: usize,
    mode: &str,
) -> String {
    let raw = format!("{graph_request_id}:{step_id}:map:{item_index}:{mode}");
    if raw.len() <= 128 {
        return raw;
    }
    let digest = format!("{:x}", Sha256::digest(raw.as_bytes()));
    let suffix = format!(":map:{item_index}:{mode}:{}", &digest[..12]);
    let prefix_len = 128usize.saturating_sub(suffix.len());
    let prefix: String = raw.chars().take(prefix_len).collect();
    format!("{prefix}{suffix}")
}

pub(crate) fn init_map_state(spec: &GraphMapSpec) -> NativeGraphMapState {
    let items = spec
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| NativeGraphMapItemRecord {
            index,
            item: item.clone(),
            state: "pending".into(),
            worker_slot: None,
            task_id: None,
            expected_round: 1,
            reused_worker: false,
            result_excerpt: None,
            error: None,
            started_at: None,
            finished_at: None,
        })
        .collect();
    let worker_slots = (0..spec.max_concurrency)
        .map(|slot_id| NativeGraphMapWorkerSlot {
            slot_id,
            task_id: None,
            busy: false,
            rounds_completed: 0,
        })
        .collect();
    NativeGraphMapState {
        max_concurrency: spec.max_concurrency,
        reuse_workers: spec.reuse_workers,
        fail_fast: spec.fail_fast,
        items,
        worker_slots,
        reused_worker_hits: 0,
    }
}

/// Construit la consigne d'un élément Map-Reduce en gardant le préfixe stable en tête
/// pour une session neuve (`!is_continue`) ou en envoyant un delta court sur un worker chaud.
pub(crate) fn build_map_item_task(
    base_task: &str,
    item: &str,
    index: usize,
    total: usize,
    is_continue: bool,
) -> String {
    let ordinal = index.saturating_add(1);
    if is_continue {
        let body = format!(
            "## Élément suivant du lot Map-Reduce ({ordinal}/{total})\n\
Traite uniquement cet élément en appliquant exactement les mêmes règles et format de sortie que précédemment :\n\
{item}\n"
        );
        return if body.len() <= MAX_TASK_BYTES {
            body
        } else {
            let cut = body.floor_char_boundary(MAX_TASK_BYTES);
            body[..cut].to_string()
        };
    }
    let header = "## Consigne de lot Map-Reduce (préfixe stable)\n";
    let tail = format!(
        "\n\n## Élément assigné ({ordinal}/{total})\n\
Traite uniquement cet élément :\n\
{item}\n"
    );
    let body = format!("{header}{base_task}{tail}");
    if body.len() <= MAX_TASK_BYTES {
        body
    } else {
        // Préserver intégralement la ligne « Élément assigné » en pied : borner uniquement base_task en octets UTF-8.
        let budget = MAX_TASK_BYTES.saturating_sub(header.len() + tail.len() + 8);
        let cut = base_task.floor_char_boundary(budget.min(base_task.len()));
        format!("{header}{}\n...{tail}", &base_task[..cut])
    }
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
        if trimmed.starts_with('{')
            && trimmed.ends_with('}')
            && let Some(parsed) = try_parse_verdict_json(trimmed)
        {
            return parsed;
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
                        && let Some(parsed) = try_parse_verdict_json(&text[start..=end])
                    {
                        return parsed;
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

/// Extrait le SHA40 produit depuis un `factual_summary` (schéma réel `/git/head_commit`
/// ou champ plat `commit` pour rétrocompatibilité tests).
fn extract_commit_from_factual_summary(fs: &Value) -> Option<&str> {
    fs.pointer("/git/head_commit")
        .and_then(Value::as_str)
        .filter(|sha| is_valid_git_sha40(sha))
        .or_else(|| {
            fs.get("commit")
                .and_then(Value::as_str)
                .filter(|sha| is_valid_git_sha40(sha))
        })
}

/// Épingle automatiquement `workspace.base_ref = expected_base_commit` lorsqu'une étape
/// ou un vérificateur utilise `strategy = "worktree"` sans `base_ref` explicite.
/// Empêche `expected_base_commit_mismatch` lorsque `main` avance pendant l'exécution d'un graphe.
fn pin_worktree_base_ref(delegation: &mut AutoDelegateV2) {
    if let (Some(WorkspaceRequest::Worktree { base_ref, .. }), Some(sha)) = (
        &mut delegation.workspace,
        delegation.expected_base_commit.as_deref(),
    ) && base_ref.is_none()
        && is_valid_git_sha40(sha)
    {
        *base_ref = Some(sha.to_string());
    }
}

fn apply_commit_to_delegation(delegation: &mut AutoDelegateV2, commit: Option<&str>) {
    let Some(commit) = commit.filter(|sha| is_valid_git_sha40(sha)) else {
        pin_worktree_base_ref(delegation);
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

fn step_is_frontier_launch(
    step_tier: Option<GraphModelTier>,
    escalation_tier: u8,
    delegation: &AutoDelegateV2,
) -> bool {
    if escalation_tier > 0 || step_tier == Some(GraphModelTier::Frontier) {
        return true;
    }
    if step_tier == Some(GraphModelTier::Light) {
        return false;
    }
    delegation
        .model
        .as_deref()
        .is_some_and(super::native_delegation::is_frontier_model_name)
}

fn check_graph_budget(
    graph: &NativeGraphRecord,
    is_frontier: bool,
    now: i64,
) -> Result<(), &'static str> {
    let Some(budget) = graph.budget.as_ref() else {
        return Ok(());
    };
    if let Some(max_secs) = budget.max_duration_secs
        && now.saturating_sub(graph.created_at) >= max_secs as i64
    {
        return Err("max_duration_secs");
    }
    if let Some(max_total) = budget.max_total_rounds
        && graph.budget_usage.total_rounds_used >= max_total
    {
        return Err("max_total_rounds");
    }
    if is_frontier
        && let Some(max_frontier) = budget.max_frontier_rounds
        && graph.budget_usage.frontier_rounds_used >= max_frontier
    {
        return Err("max_frontier_rounds");
    }
    Ok(())
}

fn consume_graph_budget(graph: &mut NativeGraphRecord, is_frontier: bool) {
    graph.budget_usage.total_rounds_used = graph.budget_usage.total_rounds_used.saturating_add(1);
    if is_frontier {
        graph.budget_usage.frontier_rounds_used =
            graph.budget_usage.frontier_rounds_used.saturating_add(1);
    }
}

fn fail_step_budget_exhausted(graph: &mut NativeGraphRecord, idx: usize, reason: &str, now: i64) {
    graph.budget_usage.exhausted_reason = Some(reason.to_string());
    let step = &mut graph.steps[idx];
    step.state = "failed".into();
    step.error = Some(format!("budget_exhausted:{reason}"));
    step.finished_at = Some(now);
    if let Some(cs) = step.convergence_state.as_mut() {
        if let Some(iter_rec) = cs.iterations.last_mut() {
            iter_rec.finished_at.get_or_insert(now);
        }
        cs.phase = "done".into();
        cs.outcome = Some("budget_exhausted".into());
    }
}

/// Extrait un éventuel objet `{"blackboard":{...}}` en fin de réponse d'une étape
/// (ligne JSON autonome ou bloc markdown ```json), sans appel MCP supplémentaire.
pub(crate) fn extract_blackboard_updates(text: &str) -> Option<serde_json::Map<String, Value>> {
    let parse_candidate = |raw: &str| -> Option<serde_json::Map<String, Value>> {
        let trimmed = raw.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return None;
        }
        let val = serde_json::from_str::<Value>(trimmed).ok()?;
        val.get("blackboard").and_then(Value::as_object).cloned()
    };

    for line in text.lines().rev() {
        if let Some(map) = parse_candidate(line) {
            return Some(map);
        }
    }
    let mut in_fence = false;
    let mut buf = String::new();
    let mut last_found = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_fence {
                if let Some(map) = parse_candidate(&buf) {
                    last_found = Some(map);
                }
                buf.clear();
                in_fence = false;
            } else {
                in_fence = true;
                buf.clear();
            }
            continue;
        }
        if in_fence {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    last_found
}

/// Valide et applique les écritures tableau noir émises par une étape terminée.
/// Une étape doit déclarer explicitement les clés autorisées dans `blackboard_writes`
/// (liste vide = aucune écriture acceptée) ; plafond de 32 clés et 4 Ko par valeur.
pub(crate) fn apply_blackboard_updates(
    graph: &mut NativeGraphRecord,
    idx: usize,
    text: &str,
    now: i64,
) -> u32 {
    let allowed = graph.steps[idx].blackboard_writes.clone();
    if allowed.is_empty() {
        return 0;
    }
    let Some(updates) = extract_blackboard_updates(text) else {
        return 0;
    };
    let step_id = graph.steps[idx].step_id.clone();
    let mut applied = 0u32;
    for (key, value) in updates {
        if !valid_blackboard_key(&key) {
            continue;
        }
        if !allowed.iter().any(|k| k == &key) {
            continue;
        }
        let encoded_len = serde_json::to_vec(&value).map_or(usize::MAX, |v| v.len());
        if encoded_len > MAX_BLACKBOARD_VALUE_BYTES {
            continue;
        }
        if !graph.blackboard.contains_key(&key) && graph.blackboard.len() >= MAX_BLACKBOARD_KEYS {
            continue;
        }
        graph.blackboard.insert(
            key,
            NativeGraphBlackboardEntry {
                value,
                updated_by_step: step_id.clone(),
                updated_at: now,
            },
        );
        applied = applied.saturating_add(1);
    }
    graph.steps[idx].blackboard_writes_applied = graph.steps[idx]
        .blackboard_writes_applied
        .saturating_add(applied);
    applied
}

/// Prompt vérificateur ordonné pour le Prompt Caching (Spec 316) :
/// le bloc invariant (`Contrat de verdict` + `Consigne de vérification`) est placé
/// strictement en tête, et le contexte variable de l'itération courante en pied.
fn build_verifier_task(
    step: &NativeGraphStepRecord,
    verify: &AutoDelegateV2,
    iteration: u8,
    max_iterations: u8,
) -> String {
    let contract = "## Contrat de verdict fermé (obligatoire en fin de réponse)\n\
Termine ta réponse par UNE ligne JSON autonome respectant exactement l'un des trois schémas suivants :\n\
{\"verdict\":\"approve\",\"reason\":\"...\"}\n\
{\"verdict\":\"revise\",\"reason\":\"...\",\"blocking_points\":[\"...\",\"...\"]}\n\
{\"verdict\":\"escalate\",\"reason\":\"...\"}\n\
- `approve` : livrable conforme, clôt la boucle et débloque les étapes suivantes.\n\
- `revise` : défauts corrigeables dans le périmètre ; le producteur reprendra avec tes `blocking_points`.\n\
- `escalate` : blocage hors périmètre, ambiguïté produit ou arbitrage humain requis ; arrêt immédiat vers le parent.\n";
    let invariant_prefix = format!(
        "{contract}\n## Consigne de vérification (étape `{}`)\n{}\n",
        step.step_id, verify.task
    );

    let mut variable_tail =
        format!("\n## Contexte de l'itération courante ({iteration}/{max_iterations})\n");
    if let Some(cs) = step.convergence_state.as_ref()
        && let Some(tid) = cs.producer_task_id.as_deref()
    {
        variable_tail.push_str(&format!("- Producteur : task_id={tid}"));
        if let Some(commit) = cs.last_producer_commit.as_deref() {
            variable_tail.push_str(&format!(", commit={commit}"));
        }
        variable_tail.push('\n');
    }
    if let Some(fs) = step.factual_summary.as_ref()
        && let Some(files) = fs.get("files_changed").and_then(Value::as_u64)
    {
        variable_tail.push_str(&format!("- Fichiers modifiés déclarés : {files}\n"));
    }
    if let Some(excerpt) = step.result_excerpt.as_deref()
        && !excerpt.trim().is_empty()
    {
        let clipped: String = excerpt.trim().chars().take(MAX_DEP_EXCERPT_CHARS).collect();
        variable_tail.push_str(&format!("- Extrait producteur : {clipped}\n"));
    }

    let body = format!("{invariant_prefix}{variable_tail}");
    if body.len() <= MAX_TASK_BYTES {
        body
    } else {
        let budget = MAX_TASK_BYTES.saturating_sub(invariant_prefix.len() + 16);
        let mut clipped_tail = String::new();
        for ch in variable_tail.chars() {
            if clipped_tail.len() + ch.len_utf8() > budget {
                break;
            }
            clipped_tail.push(ch);
        }
        format!("{invariant_prefix}{clipped_tail}\n...")
    }
}

/// Prompt vérificateur dédié à un candidat parallèle `Best-of-N` d'escalade (préfixe stable en tête).
fn build_candidate_verifier_task(
    step: &NativeGraphStepRecord,
    verify: &AutoDelegateV2,
    cand: &NativeGraphEscalationCandidate,
    total_candidates: usize,
) -> String {
    let contract = "## Contrat de verdict fermé (obligatoire en fin de réponse)\n\
Termine ta réponse par UNE ligne JSON autonome respectant exactement l'un des trois schémas suivants :\n\
{\"verdict\":\"approve\",\"reason\":\"...\"}\n\
{\"verdict\":\"revise\",\"reason\":\"...\",\"blocking_points\":[\"...\",\"...\"]}\n\
{\"verdict\":\"escalate\",\"reason\":\"...\"}\n\
- `approve` : livrable conforme, clôt la boucle et débloque les étapes suivantes.\n\
- `revise` : défauts corrigeables dans le périmètre ; le producteur reprendra avec tes `blocking_points`.\n\
- `escalate` : blocage hors périmètre, ambiguïté produit ou arbitrage humain requis ; arrêt immédiat vers le parent.\n";
    let invariant_prefix = format!(
        "{contract}\n## Consigne de vérification (étape `{}`)\n{}\n",
        step.step_id, verify.task
    );
    let mut variable_tail = format!(
        "\n## Contexte du candidat parallèle d'escalade ({}/{total_candidates})\n",
        cand.candidate_id.saturating_add(1)
    );
    if let Some(tid) = cand.producer_task_id.as_deref() {
        variable_tail.push_str(&format!("- Producteur candidat : task_id={tid}"));
        if let Some(commit) = cand.producer_commit.as_deref() {
            variable_tail.push_str(&format!(", commit={commit}"));
        }
        variable_tail.push('\n');
    }
    if let Some(excerpt) = cand.result_excerpt.as_deref()
        && !excerpt.trim().is_empty()
    {
        let clipped: String = excerpt.trim().chars().take(MAX_DEP_EXCERPT_CHARS).collect();
        variable_tail.push_str(&format!("- Extrait candidat : {clipped}\n"));
    }
    let body = format!("{invariant_prefix}{variable_tail}");
    if body.len() <= MAX_TASK_BYTES {
        body
    } else {
        let budget = MAX_TASK_BYTES.saturating_sub(invariant_prefix.len() + 16);
        let mut clipped_tail = String::new();
        for ch in variable_tail.chars() {
            if clipped_tail.len() + ch.len_utf8() > budget {
                break;
            }
            clipped_tail.push(ch);
        }
        format!("{invariant_prefix}{clipped_tail}\n...")
    }
}

/// Prompt de reprise corrective : pour une session neuve (`for_fallback_new_task`),
/// la consigne initiale est placée EN TÊTE afin de préserver le préfixe de cache KV.
fn build_producer_revision_task(
    step_id: &str,
    tier: u8,
    iteration: u8,
    max_iterations: u8,
    reason: &str,
    blocking_points: &[String],
    original_task: &str,
    for_fallback_new_task: bool,
    from_gate: bool,
) -> String {
    let source = if from_gate {
        "La porte déterministe (vérification objective 0 jeton) a échoué."
    } else {
        "Le vérificateur indépendant a demandé une correction (`verdict=revise`)."
    };
    let tier_note = if tier > 0 {
        format!(
            "\nEscalade de modèle activée (palier {tier}) : le palier précédent a stagné ou atteint son plafond. Repars proprement depuis le commit courant en analysant la cause racine.\n"
        )
    } else {
        String::new()
    };
    let mut revision = format!(
        "## Reprise corrective : étape `{step_id}` (palier {tier}, itération {iteration}/{max_iterations})\n{source}{tier_note}\nMotif : {reason}\n"
    );
    if !blocking_points.is_empty() {
        revision.push_str(if from_gate {
            "Sortie de la porte déterministe :\n"
        } else {
            "Points bloquants à corriger :\n"
        });
        for point in blocking_points {
            revision.push_str(&format!("- {point}\n"));
        }
    }
    revision.push_str(
        "Corrige précisément ces points bloquants, vérifie par des tests ciblés et produis un nouveau commit avant de rendre ton bilan.\n",
    );

    let out = if for_fallback_new_task {
        format!("## Consigne de cette étape\n{original_task}\n\n{revision}")
    } else {
        revision
    };
    if out.len() <= MAX_TASK_BYTES {
        out
    } else {
        out.chars().take(MAX_TASK_BYTES).collect()
    }
}

/// Construit la consigne finale d'une étape en plaçant le bloc invariant en tête
/// (Prompt Caching, Spec 316) et les données variables en pied (clés ciblées du tableau noir
/// + résultats factuels des dépendances), borné à `MAX_TASK_BYTES`.
pub(crate) fn build_enriched_step_task(
    step: &NativeGraphStepRecord,
    all_steps: &[NativeGraphStepRecord],
    blackboard: &BTreeMap<String, NativeGraphBlackboardEntry>,
) -> (String, u32) {
    let mut invariant = format!("## Consigne de cette étape\n{}", step.delegation.task);
    if !step.blackboard_writes.is_empty() {
        invariant.push_str(&format!(
            "\n\n## Contrat tableau noir (optionnel en fin de réponse)\n\
Si tu produis des faits structurés utiles aux étapes suivantes, termine par une ligne JSON autonome contenant uniquement les clés autorisées ({}) :\n\
{{\"blackboard\":{{\"{}\": ...}}}}\n",
            step.blackboard_writes.join(", "),
            step.blackboard_writes[0]
        ));
    }

    let mut variable_tail = String::new();
    let mut bb_reads_injected = 0u32;
    if !step.blackboard_reads.is_empty() {
        let mut bb_lines = Vec::new();
        for key in &step.blackboard_reads {
            if let Some(entry) = blackboard.get(key)
                && let Ok(encoded) = serde_json::to_string(&entry.value)
            {
                bb_lines.push(format!(
                    "- `{key}` (par `{}`) : {encoded}",
                    entry.updated_by_step
                ));
                bb_reads_injected = bb_reads_injected.saturating_add(1);
            }
        }
        if !bb_lines.is_empty() {
            variable_tail.push_str("\n\n## Tableau noir du graphe (clés sélectionnées)\n");
            for line in bb_lines {
                variable_tail.push_str(&line);
                variable_tail.push('\n');
            }
        }
    }

    if step.pass_dependency_results && !step.depends_on.is_empty() {
        let by_id: BTreeMap<&str, &NativeGraphStepRecord> =
            all_steps.iter().map(|s| (s.step_id.as_str(), s)).collect();
        variable_tail.push_str("\n\n## Résultats des étapes préalables du graphe Bridget\n");
        for dep_id in &step.depends_on {
            let Some(dep) = by_id.get(dep_id.as_str()) else {
                continue;
            };
            variable_tail.push_str(&format!("- Étape `{}` : état={}", dep.step_id, dep.state));
            if let Some(tid) = dep.task_id.as_deref() {
                variable_tail.push_str(&format!(", task_id={tid}"));
            }
            if let Some(fs) = dep.factual_summary.as_ref() {
                if let Some(commit) = extract_commit_from_factual_summary(fs)
                    .or_else(|| fs.get("commit").and_then(Value::as_str))
                {
                    variable_tail.push_str(&format!(", commit={commit}"));
                }
                if let Some(files) = fs
                    .get("files_changed")
                    .and_then(Value::as_u64)
                    .or_else(|| fs.pointer("/git/commit_count").and_then(Value::as_u64))
                {
                    variable_tail.push_str(&format!(", fichiers_modifies={files}"));
                }
            }
            if let Some(cs) = dep.convergence_state.as_ref() {
                if let Some(outcome) = cs.outcome.as_deref() {
                    variable_tail.push_str(&format!(", outcome={outcome}"));
                }
                if cs.tier_escalated {
                    variable_tail.push_str(", tier_escalated=true");
                }
            }
            variable_tail.push('\n');
            if let Some(err) = dep.error.as_deref()
                && !err.trim().is_empty()
            {
                let clipped: String = err.trim().chars().take(400).collect();
                variable_tail.push_str(&format!("  Erreur : {clipped}\n"));
            }
            if dep.state == "failed"
                && let Some(last_iter) = dep
                    .convergence_state
                    .as_ref()
                    .and_then(|cs| cs.iterations.last())
                && !last_iter.blocking_points.is_empty()
            {
                variable_tail.push_str("  Derniers points bloquants :\n");
                for pt in last_iter.blocking_points.iter().take(6) {
                    let clipped: String = pt.chars().take(300).collect();
                    variable_tail.push_str(&format!("  - {clipped}\n"));
                }
            }
            if let Some(excerpt) = dep.result_excerpt.as_deref()
                && !excerpt.trim().is_empty()
            {
                let limit = if dep.map_state.is_some() {
                    MAX_DEP_EXCERPT_CHARS * 3
                } else {
                    MAX_DEP_EXCERPT_CHARS
                };
                let clipped: String = excerpt.trim().chars().take(limit).collect();
                variable_tail.push_str(&format!("  Extrait : {clipped}\n"));
            }
        }
    }

    if variable_tail.is_empty() {
        if step.blackboard_writes.is_empty() {
            return (step.delegation.task.clone(), 0);
        }
        return (invariant, 0);
    }

    let combined = format!("{invariant}{variable_tail}");
    if combined.len() <= MAX_TASK_BYTES {
        (combined, bb_reads_injected)
    } else {
        // Ne jamais tronquer la consigne métier en tête : raccourcir uniquement le pied variable.
        let budget = MAX_TASK_BYTES.saturating_sub(invariant.len() + 16);
        if budget < 64 {
            (invariant, 0)
        } else {
            let mut trimmed_tail = String::new();
            for ch in variable_tail.chars() {
                if trimmed_tail.len() + ch.len_utf8() > budget {
                    break;
                }
                trimmed_tail.push(ch);
            }
            (format!("{invariant}{trimmed_tail}\n..."), bb_reads_injected)
        }
    }
}

/// Propage les échecs, annulations et conditions de branches de repli sur les étapes du graphe
/// et renvoie la liste des `task_id` d'étapes actives à annuler si `on_failure == CancelGraph`.
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
                        for prior in &cs.prior_producer_task_ids {
                            if !to_cancel_tasks.contains(prior) {
                                to_cancel_tasks.push(prior.clone());
                            }
                        }
                        for cand in &cs.escalation_candidates {
                            if let Some(ptid) = cand.producer_task_id.as_ref()
                                && !to_cancel_tasks.contains(ptid)
                            {
                                to_cancel_tasks.push(ptid.clone());
                            }
                            if let Some(vtid) = cand.verifier_task_id.as_ref()
                                && !to_cancel_tasks.contains(vtid)
                            {
                                to_cancel_tasks.push(vtid.clone());
                            }
                        }
                    }
                    if let Some(ms) = step.map_state.as_ref() {
                        for item in &ms.items {
                            if matches!(item.state.as_str(), "launching" | "running")
                                && let Some(tid) = item.task_id.as_ref()
                                && !to_cancel_tasks.contains(tid)
                            {
                                to_cancel_tasks.push(tid.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        // Point fixe pour propager `skipped_dependency_failed` et `skipped_condition_not_met`
        // en respectant strictement la condition `when` de chaque étape (aucun blocage).
        loop {
            let snapshots: BTreeMap<String, (String, Option<String>)> = graph
                .steps
                .iter()
                .map(|s| {
                    (
                        s.step_id.clone(),
                        (
                            s.state.clone(),
                            step_dependency_failure_outcome(s).map(str::to_string),
                        ),
                    )
                })
                .collect();
            let mut changed = false;
            for step in &mut graph.steps {
                if step.state != "waiting_dependencies" {
                    continue;
                }
                match step.when {
                    GraphStepCondition::AllSucceeded => {
                        if let Some(bad_dep) = step.depends_on.iter().find(|d| {
                            snapshots.get(*d).is_some_and(|(st, _)| {
                                matches!(
                                    st.as_str(),
                                    "failed" | "cancelled" | "skipped_dependency_failed"
                                )
                            })
                        }) {
                            step.state = "skipped_dependency_failed".into();
                            step.error = Some(format!("dependency_failed:{bad_dep}"));
                            step.finished_at = Some(now);
                            changed = true;
                        } else if !step.depends_on.is_empty()
                            && step.depends_on.iter().all(|d| {
                                snapshots.get(d).is_some_and(|(st, _)| step_is_terminal(st))
                            })
                            && step.depends_on.iter().any(|d| {
                                snapshots
                                    .get(d)
                                    .is_some_and(|(st, _)| st == "skipped_condition_not_met")
                            })
                        {
                            step.state = "skipped_condition_not_met".into();
                            step.error = None;
                            step.finished_at = Some(now);
                            changed = true;
                        }
                    }
                    GraphStepCondition::AnyFailed => {
                        let all_deps_terminal = step
                            .depends_on
                            .iter()
                            .all(|d| snapshots.get(d).is_some_and(|(st, _)| step_is_terminal(st)));
                        if all_deps_terminal {
                            let any_dep_failed = step
                                .depends_on
                                .iter()
                                .any(|d| snapshots.get(d).is_some_and(|(st, _)| st == "failed"));
                            if !any_dep_failed {
                                step.state = "skipped_condition_not_met".into();
                                step.error = None;
                                step.finished_at = Some(now);
                                changed = true;
                            }
                        }
                    }
                    GraphStepCondition::OnOutcome => {
                        let all_deps_terminal = step
                            .depends_on
                            .iter()
                            .all(|d| snapshots.get(d).is_some_and(|(st, _)| step_is_terminal(st)));
                        if all_deps_terminal {
                            let matched = step.depends_on.iter().any(|d| {
                                snapshots.get(d).is_some_and(|(_, outcome)| {
                                    outcome
                                        .as_deref()
                                        .is_some_and(|o| step.when_outcomes.iter().any(|w| w == o))
                                })
                            });
                            if !matched {
                                step.state = "skipped_condition_not_met".into();
                                step.error = None;
                                step.finished_at = Some(now);
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    if graph.steps.iter().all(|s| step_is_terminal(&s.state)) {
        let all_completed_or_unused_fallback = graph
            .steps
            .iter()
            .all(|s| matches!(s.state.as_str(), "completed" | "skipped_condition_not_met"))
            && graph.steps.iter().any(|s| s.state == "completed");
        let any_completed = graph.steps.iter().any(|s| s.state == "completed");
        let all_cancelled = graph.steps.iter().all(|s| s.state == "cancelled");
        graph.state = if all_completed_or_unused_fallback {
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

/// Renvoie les `step_id` en attente dont les dépendances satisfont la condition `when`.
pub(crate) fn ready_waiting_steps(graph: &NativeGraphRecord) -> Vec<String> {
    let by_id: BTreeMap<&str, &NativeGraphStepRecord> = graph
        .steps
        .iter()
        .map(|s| (s.step_id.as_str(), s))
        .collect();
    graph
        .steps
        .iter()
        .filter(|s| {
            if s.state != "waiting_dependencies" {
                return false;
            }
            match s.when {
                GraphStepCondition::AllSucceeded => s.depends_on.iter().all(|d| {
                    by_id
                        .get(d.as_str())
                        .is_some_and(|dep| dep.state == "completed")
                }),
                GraphStepCondition::AnyFailed => {
                    !s.depends_on.is_empty()
                        && s.depends_on.iter().all(|d| {
                            by_id
                                .get(d.as_str())
                                .is_some_and(|dep| step_is_terminal(&dep.state))
                        })
                        && s.depends_on.iter().any(|d| {
                            by_id
                                .get(d.as_str())
                                .is_some_and(|dep| dep.state == "failed")
                        })
                }
                GraphStepCondition::OnOutcome => {
                    !s.depends_on.is_empty()
                        && s.depends_on.iter().all(|d| {
                            by_id
                                .get(d.as_str())
                                .is_some_and(|dep| step_is_terminal(&dep.state))
                        })
                        && s.depends_on.iter().any(|d| {
                            by_id
                                .get(d.as_str())
                                .and_then(|dep| step_dependency_failure_outcome(dep))
                                .is_some_and(|o| s.when_outcomes.iter().any(|w| w == o))
                        })
                }
            }
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
    let skipped_condition_not_met = graph
        .steps
        .iter()
        .filter(|s| s.state == "skipped_condition_not_met")
        .count();
    let fallback_steps_completed = graph
        .steps
        .iter()
        .filter(|s| s.when != GraphStepCondition::AllSucceeded && s.state == "completed")
        .count();
    let fallback_recovered =
        fallback_steps_completed > 0 && graph.steps.iter().any(|s| s.state == "failed");
    let cancelled = graph
        .steps
        .iter()
        .filter(|s| s.state == "cancelled")
        .count();
    let mut gate_runs_total = 0u32;
    let mut gate_failures_total = 0u32;
    let mut gate_saved_verifier_calls_total = 0u32;
    let mut tier_escalated_steps = 0usize;
    let mut stagnation_saved_iterations_total = 0u32;
    let mut map_reused_worker_hits_total = 0u32;
    let mut escalation_candidates_cancelled_total = 0u32;
    for s in &graph.steps {
        if let Some(cs) = s.convergence_state.as_ref() {
            gate_runs_total = gate_runs_total.saturating_add(cs.gate_runs);
            gate_failures_total = gate_failures_total.saturating_add(cs.gate_failures);
            gate_saved_verifier_calls_total =
                gate_saved_verifier_calls_total.saturating_add(cs.gate_saved_verifier_calls);
            if cs.tier_escalated {
                tier_escalated_steps += 1;
            }
            stagnation_saved_iterations_total =
                stagnation_saved_iterations_total.saturating_add(cs.stagnation_saved_iterations);
            escalation_candidates_cancelled_total =
                escalation_candidates_cancelled_total.saturating_add(cs.candidates_cancelled);
        }
        if let Some(ms) = s.map_state.as_ref() {
            map_reused_worker_hits_total =
                map_reused_worker_hits_total.saturating_add(ms.reused_worker_hits);
        }
    }
    let steps: Vec<Value> = graph
        .steps
        .iter()
        .map(|s| {
            json!({
                "step_id": s.step_id,
                "depends_on": s.depends_on,
                "when": s.when,
                "when_outcomes": s.when_outcomes,
                "model_tier": s.model_tier,
                "blackboard_reads": s.blackboard_reads,
                "blackboard_writes": s.blackboard_writes,
                "state": s.state,
                "task_id": s.task_id,
                "error": s.error,
                "factual_summary": s.factual_summary,
                "result_excerpt": s.result_excerpt,
                "convergence": s.convergence_state,
                "map": s.map_state,
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
        "bridget_build_id": if graph.bridget_build_id.is_empty() { Value::Null } else { json!(graph.bridget_build_id) },
        "state": graph.state,
        "fallback_recovered": fallback_recovered,
        "on_failure": graph.on_failure,
        "notify_policy": graph.notify_policy,
        "budget": graph.budget,
        "budget_usage": graph.budget_usage,
        "blackboard": graph.blackboard,
        "topological_order": graph.topological_order,
        "counts": {
            "total": total,
            "completed": completed,
            "running": running,
            "waiting_dependencies": waiting,
            "failed_or_skipped": failed,
            "skipped_condition_not_met": skipped_condition_not_met,
            "fallback_steps_completed": fallback_steps_completed,
            "cancelled": cancelled,
        },
        "metrics_summary": {
            "gate_runs": gate_runs_total,
            "gate_failures": gate_failures_total,
            "gate_saved_verifier_calls": gate_saved_verifier_calls_total,
            "tier_escalated_steps": tier_escalated_steps,
            "stagnation_saved_iterations": stagnation_saved_iterations_total,
            "map_reused_worker_hits": map_reused_worker_hits_total,
            "escalation_candidates_cancelled": escalation_candidates_cancelled_total,
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
    let fallback_recovered = graph
        .steps
        .iter()
        .any(|s| s.when != GraphStepCondition::AllSucceeded && s.state == "completed")
        && graph.steps.iter().any(|s| s.state == "failed");
    let mut out = format!(
        "Graphe Bridget `{}` ({}) terminé — état : `{}`{}.\n",
        title,
        graph.graph_id,
        graph.state,
        if fallback_recovered {
            " (repli réussi)"
        } else {
            ""
        }
    );
    for step in &graph.steps {
        out.push_str(&format!("- `{}` : {}", step.step_id, step.state));
        if let Some(tid) = step.task_id.as_deref() {
            out.push_str(&format!(" (task_id={tid})"));
        }
        if let Some(commit) = step.factual_summary.as_ref().and_then(|v| {
            extract_commit_from_factual_summary(v)
                .or_else(|| v.get("commit").and_then(Value::as_str))
        }) {
            out.push_str(&format!(" · commit={commit}"));
        }
        if let Some(cs) = step.convergence_state.as_ref() {
            let outcome = cs.outcome.as_deref().unwrap_or(cs.phase.as_str());
            out.push_str(&format!(
                " · convergence={outcome} ({}/{})",
                cs.current_iteration, cs.max_iterations
            ));
            if cs.gate_runs > 0 {
                out.push_str(&format!(
                    " · gate={}/{} (verif_epargnees={})",
                    cs.gate_runs.saturating_sub(cs.gate_failures),
                    cs.gate_runs,
                    cs.gate_saved_verifier_calls
                ));
            }
            if cs.tier_escalated {
                out.push_str(" · escalade_modele=palier1");
            }
            if !cs.escalation_candidates.is_empty() {
                let winner = cs
                    .winning_candidate_id
                    .map(|id| format!("#{}", id.saturating_add(1)))
                    .unwrap_or_else(|| "aucun".into());
                out.push_str(&format!(
                    " · best_of_n={}/{} (gagnant={winner}, annules={})",
                    cs.escalation_candidates.len(),
                    cs.escalation_candidates.len(),
                    cs.candidates_cancelled
                ));
            }
        }
        if let Some(ms) = step.map_state.as_ref() {
            let done = ms.items.iter().filter(|i| i.state == "completed").count();
            out.push_str(&format!(
                " · map={done}/{} (workers_reutilises={})",
                ms.items.len(),
                ms.reused_worker_hits
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
        if request
            .steps
            .iter()
            .any(|s| s.convergence.as_ref().is_some_and(|c| c.gate.is_some()))
            && !crate::native_permissions::permits_command_execution(
                &attested.provider_policy,
                true,
            )
        {
            return graph_failure(
                "permission_not_inherited",
                "convergence.gate exige un parent attesté autorisé à exécuter des commandes en posture development (refusé en lecture seule / discovery).",
            );
        }
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
                pin_worktree_base_ref(&mut delegation);
                delegation.request_id = if s.convergence.is_some() {
                    convergence_step_request_id(&request.request_id, &s.step_id, 0, 1, "produce")
                } else {
                    step_request_id(&request.request_id, &s.step_id, &delegation.request_id)
                };
                if s.convergence.is_some() {
                    ensure_convergence_producer_keep_alive(&mut delegation);
                }
                let mut convergence = s.convergence.clone();
                if let Some(conv) = convergence.as_mut() {
                    pin_worktree_base_ref(&mut conv.verify);
                }
                let convergence_state =
                    s.convergence
                        .as_ref()
                        .map(|conv| NativeGraphConvergenceState {
                            current_iteration: 1,
                            max_iterations: conv.max_iterations,
                            total_max_iterations: conv.max_iterations,
                            stop_on_stagnation: conv.stop_on_stagnation,
                            phase: "waiting".into(),
                            escalation_tier: 0,
                            tier_escalated: false,
                            escalated_from_reason: None,
                            producer_task_id: None,
                            prior_producer_task_ids: Vec::new(),
                            producer_expected_round: 1,
                            verifier_task_id: None,
                            last_producer_commit: None,
                            last_blocking_fingerprint: None,
                            seen_commits: Vec::new(),
                            seen_blocking_fingerprints: Vec::new(),
                            gate_runs: 0,
                            gate_failures: 0,
                            gate_saved_verifier_calls: 0,
                            stagnation_saved_iterations: 0,
                            metrics_recorded: false,
                            outcome: None,
                            iterations: Vec::new(),
                            escalation_candidates: Vec::new(),
                            winning_candidate_id: None,
                            candidates_cancelled: 0,
                        });
                let is_root = s.depends_on.is_empty() && s.when == GraphStepCondition::AllSucceeded;
                NativeGraphStepRecord {
                    step_id: s.step_id.clone(),
                    depends_on: s.depends_on.clone(),
                    pass_dependency_results: s.pass_dependency_results,
                    when: s.when,
                    when_outcomes: s.when_outcomes.clone(),
                    model_tier: s.model_tier,
                    blackboard_reads: s.blackboard_reads.clone(),
                    blackboard_writes: s.blackboard_writes.clone(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation,
                    convergence,
                    convergence_state,
                    map: s.map.clone(),
                    map_state: None,
                    // Les racines sont marquées `launching` avant de relâcher le verrou afin que
                    // `tick` ne puisse jamais les lancer en double ni écraser `budget_usage`.
                    state: if is_root {
                        "launching".into()
                    } else {
                        "waiting_dependencies".into()
                    },
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
            .filter_map(|(i, s)| {
                (s.depends_on.is_empty() && s.when == GraphStepCondition::AllSucceeded).then_some(i)
            })
            .collect();
        let graph = NativeGraphRecord {
            graph_id: format!("graph-{}", uuid::Uuid::new_v4().simple()),
            owner: owner.clone(),
            owner_instance: instance,
            request_id: request.request_id.clone(),
            title: request.title.clone(),
            bridget_build_id: crate::build_info::BUILD_ID.to_string(),
            on_failure: request.on_failure,
            notify_policy: request.notify_policy,
            budget: request.budget.clone(),
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
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
        if let Some(map_spec) = graph.steps[idx].map.clone() {
            if let Some(tier) = graph.steps[idx].model_tier {
                let st = state.lock().recover_poison();
                super::native_delegation::resolve_model_tier_pub(
                    &st,
                    &mut graph.steps[idx].delegation,
                    tier,
                );
            }
            let (base_task, bb_reads) =
                build_enriched_step_task(&graph.steps[idx], &graph.steps, &graph.blackboard);
            graph.steps[idx].delegation.task = base_task;
            graph.steps[idx].blackboard_reads_injected = bb_reads;
            graph.steps[idx].map_state = Some(init_map_state(&map_spec));
            graph.steps[idx].state = "running".into();
            graph.steps[idx].started_at = Some(unix_timestamp());
            continue;
        }
        let mut step_delegation = graph.steps[idx].delegation.clone();
        if let Some(tier) = graph.steps[idx].model_tier {
            let st = state.lock().recover_poison();
            super::native_delegation::resolve_model_tier_pub(&st, &mut step_delegation, tier);
        }
        let is_frontier = step_is_frontier_launch(graph.steps[idx].model_tier, 0, &step_delegation);
        let now_check = unix_timestamp();
        if let Err(reason) = check_graph_budget(&graph, is_frontier, now_check) {
            fail_step_budget_exhausted(&mut graph, idx, reason, now_check);
            continue;
        }
        consume_graph_budget(&mut graph, is_frontier);
        let (task_text, bb_reads) =
            build_enriched_step_task(&graph.steps[idx], &graph.steps, &graph.blackboard);
        step_delegation.task = task_text;
        graph.steps[idx].blackboard_reads_injected = bb_reads;
        graph.steps[idx].delegation.engine = step_delegation.engine.clone();
        graph.steps[idx].delegation.model = step_delegation.model.clone();
        graph.steps[idx].delegation.effort = step_delegation.effort.clone();
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
        persist_graph_with_pending_metrics(&st, &mut graph, now);
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
    let st = state.lock().recover_poison();
    apply_admission_response_locked(
        graph,
        idx,
        step_id,
        response,
        suppress_step_notices,
        &st,
        unix_timestamp(),
    );
}

fn apply_admission_response_locked(
    graph: &mut NativeGraphRecord,
    idx: usize,
    step_id: &str,
    response: DaemonToWrapper,
    suppress_step_notices: bool,
    st: &DaemonState,
    now: i64,
) {
    match response {
        DaemonToWrapper::NativeDelegationResult { result } => {
            if result["status"] != "refused"
                && let Some(task_id) = result["task_id"].as_str()
            {
                graph.steps[idx].state = "running".into();
                graph.steps[idx].task_id = Some(task_id.to_string());
                graph.steps[idx].started_at.get_or_insert(now);
                let prod_engine = graph.steps[idx].delegation.engine.clone();
                let prod_model = graph.steps[idx].delegation.model.clone();
                if let Some(cs) = graph.steps[idx].convergence_state.as_mut() {
                    cs.phase = "producing".into();
                    cs.producer_task_id = Some(task_id.to_string());
                    cs.producer_expected_round = 1;
                    cs.iterations.push(NativeGraphConvergenceIteration {
                        iteration: cs.current_iteration,
                        tier: cs.escalation_tier,
                        producer_task_id: Some(task_id.to_string()),
                        producer_round: 1,
                        producer_commit: None,
                        producer_engine: prod_engine,
                        producer_model: prod_model,
                        gate_passed: None,
                        gate_exit_code: None,
                        gate_timed_out: false,
                        gate_duration_ms: None,
                        gate_output_excerpt: None,
                        verifier_task_id: None,
                        verifier_engine: None,
                        verifier_model: None,
                        verifier_orthogonal: None,
                        verdict: None,
                        reason: None,
                        blocking_points: Vec::new(),
                        blocking_fingerprint: None,
                        started_at: Some(now),
                        finished_at: None,
                    });
                }
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
    let _ = sync_steps_from_tasks(&st, &mut graph, unix_timestamp());
    graph_view(&graph)
}

fn require_local_control_principal(st: &DaemonState, conn: &str) -> Result<(), Value> {
    if st.connection_roles.get(conn) != Some(&ConnectionRole::Client)
        || control_client_actor(st, conn).is_none()
        || st.conn_names.contains_key(conn)
        || !st.client_negotiations.get(conn).is_some_and(|client| {
            client
                .capabilities
                .contains(&ClientCapability::ControlStateV1)
        })
    {
        return Err(graph_failure(
            "human_principal_required",
            "Cette opération d'administration des métriques de graphe est réservée à la CLI locale du principal humain.",
        ));
    }
    Ok(())
}

/// Lecture consolidée des métriques durables de convergence (tous paliers Time-Machine :
/// brut + journalier + mensuel) pour `bridget graphs metrics`.
pub(super) fn handle_metrics(
    conn: &str,
    project: Option<&str>,
    window_days: Option<u32>,
    compare: Option<(&str, &str)>,
    state: &Arc<Mutex<DaemonState>>,
) -> Value {
    let st = state.lock().recover_poison();
    if let Err(refusal) = require_local_control_principal(&st, conn) {
        return refusal;
    }
    let now = unix_timestamp();
    match st
        .delegation_store
        .query_convergence_metrics(project, window_days, compare, now)
    {
        Ok(report) => json!({
            "version": 1,
            "status": "ok",
            "report": report,
        }),
        Err(error) => graph_failure("metrics_query_failed", &error),
    }
}

/// Lecture, mise à jour et/ou exécution immédiate de la politique Time-Machine d'agrégation
/// et de purge (`bridget graphs metrics-policy`).
pub(super) fn handle_metrics_policy(
    conn: &str,
    update: Option<ConvergenceMetricsPolicyUpdate>,
    run_now: bool,
    state: &Arc<Mutex<DaemonState>>,
) -> Value {
    let mut st = state.lock().recover_poison();
    if let Err(refusal) = require_local_control_principal(&st, conn) {
        return refusal;
    }
    let now = unix_timestamp();
    let policy = if let Some(patch) = update {
        match st
            .delegation_store
            .update_convergence_metrics_policy(&patch, now)
        {
            Ok(p) => p,
            Err(error) => return graph_failure("metrics_policy_invalid", &error),
        }
    } else {
        match st.delegation_store.load_convergence_metrics_policy() {
            Ok(p) => p,
            Err(error) => return graph_failure("metrics_policy_unavailable", &error),
        }
    };
    let rollup = if run_now {
        match st
            .delegation_store
            .rollup_and_purge_convergence_metrics(now)
        {
            Ok(r) => Some(r),
            Err(error) => return graph_failure("metrics_rollup_failed", &error),
        }
    } else {
        None
    };
    json!({
        "version": 1,
        "status": "ok",
        "policy": policy,
        "rollup": rollup,
    })
}

/// Collecte les métriques de toutes les étapes de convergence nouvellement terminales
fn collect_step_usage_agents(
    st: &DaemonState,
    step: &NativeGraphStepRecord,
) -> Vec<(String, Option<String>)> {
    let mut task_ids: Vec<&str> = Vec::new();
    if let Some(tid) = step.task_id.as_deref() {
        task_ids.push(tid);
    }
    if let Some(cs) = step.convergence_state.as_ref() {
        if let Some(tid) = cs.producer_task_id.as_deref() {
            task_ids.push(tid);
        }
        if let Some(tid) = cs.verifier_task_id.as_deref() {
            task_ids.push(tid);
        }
        for tid in &cs.prior_producer_task_ids {
            task_ids.push(tid.as_str());
        }
        for it in &cs.iterations {
            if let Some(tid) = it.producer_task_id.as_deref() {
                task_ids.push(tid);
            }
            if let Some(tid) = it.verifier_task_id.as_deref() {
                task_ids.push(tid);
            }
        }
        for cand in &cs.escalation_candidates {
            if let Some(tid) = cand.producer_task_id.as_deref() {
                task_ids.push(tid);
            }
            if let Some(tid) = cand.verifier_task_id.as_deref() {
                task_ids.push(tid);
            }
        }
    }
    if let Some(ms) = step.map_state.as_ref() {
        for item in &ms.items {
            if let Some(tid) = item.task_id.as_deref() {
                task_ids.push(tid);
            }
        }
    }
    let mut seen_tasks = BTreeSet::new();
    let mut agents = Vec::new();
    for tid in task_ids {
        if !seen_tasks.insert(tid) {
            continue;
        }
        if let Ok(Some(task)) = st.delegation_store.get(tid)
            && !task.child.is_empty()
        {
            agents.push((task.child, task.child_instance));
        }
    }
    agents
}

/// Collecte les étapes terminales (convergées ou simples) dont la métrique n'a pas encore été
/// enregistrée, joint leurs tokens attestés depuis `execution_usage_totals` et le
/// `bridget_build_id` du graphe, puis persiste atomiquement graphe + métriques.
fn persist_graph_with_pending_metrics(st: &DaemonState, graph: &mut NativeGraphRecord, now: i64) {
    let project = graph
        .communication_project
        .as_ref()
        .map(|p| p.root.clone())
        .unwrap_or_default();
    let build_id = if graph.bridget_build_id.is_empty() {
        crate::build_info::BUILD_ID.to_string()
    } else {
        graph.bridget_build_id.clone()
    };
    let mut rows = Vec::new();
    for step in &mut graph.steps {
        if !step_is_terminal(&step.state) {
            continue;
        }
        if matches!(
            step.state.as_str(),
            "skipped_dependency_failed" | "skipped_condition_not_met"
        ) {
            continue;
        }
        let already_recorded = step.metrics_recorded
            || step
                .convergence_state
                .as_ref()
                .is_some_and(|cs| cs.metrics_recorded);
        if already_recorded {
            continue;
        }
        let usage_agents = collect_step_usage_agents(st, step);
        let usage_since = step.started_at.unwrap_or(graph.created_at);
        let usage = match st.store.sum_task_usage_tokens(&usage_agents, usage_since) {
            Ok(tokens) => tokens,
            Err(error) => {
                warn!(
                    "graphe {} étape {}: lecture tokens échouée ({error})",
                    graph.graph_id, step.step_id
                );
                Default::default()
            }
        };
        let producer_engine = step
            .delegation
            .engine
            .as_deref()
            .map(super::native_delegation::normalize_engine_family)
            .filter(|s| !s.is_empty())
            .unwrap_or("auto")
            .to_string();
        let producer_model = step.delegation.model.clone().unwrap_or_default();
        let model_tier = step
            .model_tier
            .map(|t| t.as_str().to_string())
            .unwrap_or_default();
        let started = step.started_at.unwrap_or(graph.created_at);
        let finished = step.finished_at.unwrap_or(now);
        let duration_secs = finished.saturating_sub(started).max(0);

        if let Some(cs) = step.convergence_state.as_mut() {
            let outcome = cs.outcome.clone().unwrap_or_else(|| step.state.clone());
            let verifier_engine = cs
                .iterations
                .iter()
                .rev()
                .find_map(|it| it.verifier_engine.as_deref())
                .or_else(|| {
                    step.convergence
                        .as_ref()
                        .and_then(|c| c.verify.engine.as_deref())
                })
                .map(super::native_delegation::normalize_engine_family)
                .filter(|s| !s.is_empty())
                .unwrap_or("auto")
                .to_string();
            let verifier_model = cs
                .iterations
                .iter()
                .rev()
                .find_map(|it| it.verifier_model.clone())
                .or_else(|| {
                    step.convergence
                        .as_ref()
                        .and_then(|c| c.verify.model.clone())
                })
                .unwrap_or_default();
            let verifier_orthogonal = cs
                .iterations
                .iter()
                .any(|it| it.verifier_orthogonal == Some(true));
            let tier1_iterations = cs.iterations.iter().filter(|it| it.tier == 0).count() as u32;
            let tier2_iterations = cs.iterations.iter().filter(|it| it.tier > 0).count() as u32;
            let iterations_used = (cs.iterations.len() as u32)
                .max(u32::from(cs.current_iteration))
                .min(u32::from(MAX_TOTAL_CONVERGENCE_ITERATIONS));
            let gate_configured = step.convergence.as_ref().is_some_and(|c| c.gate.is_some());
            rows.push(ConvergenceMetricRawRow {
                graph_id: graph.graph_id.clone(),
                step_id: step.step_id.clone(),
                project: project.clone(),
                bridget_build_id: build_id.clone(),
                model_tier,
                producer_engine,
                producer_model,
                verifier_engine,
                verifier_model,
                verifier_orthogonal,
                outcome,
                iterations_used,
                tier1_iterations,
                tier2_iterations,
                tier_escalated: cs.tier_escalated,
                gate_configured,
                gate_runs: cs.gate_runs,
                gate_failures: cs.gate_failures,
                gate_saved_verifier_calls: cs.gate_saved_verifier_calls,
                stagnation_saved_iterations: cs.stagnation_saved_iterations,
                duration_secs,
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_creation_input_tokens: usage.cache_creation_input_tokens,
                cache_read_input_tokens: usage.cache_read_input_tokens,
                blackboard_reads: step.blackboard_reads_injected,
                blackboard_writes: step.blackboard_writes_applied,
                finished_at: finished,
            });
            cs.metrics_recorded = true;
            step.metrics_recorded = true;
        } else {
            let outcome = if step
                .error
                .as_deref()
                .is_some_and(|e| e.starts_with("budget_exhausted"))
            {
                "budget_exhausted".to_string()
            } else {
                step.state.clone()
            };
            rows.push(ConvergenceMetricRawRow {
                graph_id: graph.graph_id.clone(),
                step_id: step.step_id.clone(),
                project: project.clone(),
                bridget_build_id: build_id.clone(),
                model_tier,
                producer_engine,
                producer_model,
                verifier_engine: String::new(),
                verifier_model: String::new(),
                verifier_orthogonal: false,
                outcome,
                iterations_used: 1,
                tier1_iterations: 1,
                tier2_iterations: 0,
                tier_escalated: false,
                gate_configured: false,
                gate_runs: 0,
                gate_failures: 0,
                gate_saved_verifier_calls: 0,
                stagnation_saved_iterations: 0,
                duration_secs,
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_creation_input_tokens: usage.cache_creation_input_tokens,
                cache_read_input_tokens: usage.cache_read_input_tokens,
                blackboard_reads: step.blackboard_reads_injected,
                blackboard_writes: step.blackboard_writes_applied,
                finished_at: finished,
            });
            step.metrics_recorded = true;
        }
    }
    if rows.is_empty() {
        let _ = st.delegation_store.save_graph(graph);
    } else if let Err(error) = st.delegation_store.save_graph_with_metrics(graph, &rows) {
        warn!(
            "graphe {}: échec sauvegarde atomique graphe+métriques : {error}",
            graph.graph_id
        );
    }
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
                        let mut candidates: Vec<String> = Vec::new();
                        if let Some(ptid) = cs.producer_task_id.as_ref() {
                            candidates.push(ptid.clone());
                        }
                        if let Some(vtid) = cs.verifier_task_id.as_ref() {
                            candidates.push(vtid.clone());
                        }
                        for prior in &cs.prior_producer_task_ids {
                            candidates.push(prior.clone());
                        }
                        for cand in &mut cs.escalation_candidates {
                            if let Some(ptid) = cand.producer_task_id.as_ref() {
                                candidates.push(ptid.clone());
                            }
                            if let Some(vtid) = cand.verifier_task_id.as_ref() {
                                candidates.push(vtid.clone());
                            }
                            if !candidate_is_terminal(&cand.phase) {
                                cand.phase = "cancelled".into();
                            }
                        }
                        for cid in candidates {
                            if !tasks_to_cancel.contains(&cid)
                                && st
                                    .delegation_store
                                    .get(&cid)
                                    .ok()
                                    .flatten()
                                    .is_some_and(|t| t.state == "working")
                            {
                                tasks_to_cancel.push(cid);
                            }
                        }
                        cs.phase = "done".into();
                        cs.outcome.get_or_insert_with(|| "cancelled".into());
                    }
                    if let Some(ms) = step.map_state.as_mut() {
                        for item in &mut ms.items {
                            if let Some(tid) = item.task_id.as_ref()
                                && !tasks_to_cancel.contains(tid)
                                && st
                                    .delegation_store
                                    .get(tid)
                                    .ok()
                                    .flatten()
                                    .is_some_and(|t| t.state == "working")
                            {
                                tasks_to_cancel.push(tid.clone());
                            }
                            if matches!(item.state.as_str(), "pending" | "launching" | "running") {
                                item.state = "cancelled".into();
                                item.finished_at.get_or_insert(now);
                            }
                        }
                        for slot in &mut ms.worker_slots {
                            slot.busy = false;
                        }
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
        persist_graph_with_pending_metrics(&st, &mut graph, now);
        (graph, tasks_to_cancel)
    };

    for tid in running_tasks {
        let _ = cancel_task_for_graph(&graph, &tid, state);
    }
    let done_at = unix_timestamp();
    graph.updated_at = done_at;
    {
        let st = state.lock().recover_poison();
        persist_graph_with_pending_metrics(&st, &mut graph, done_at);
    }
    graph_view(&graph)
}

fn candidate_is_terminal(phase: &str) -> bool {
    matches!(phase, "approved" | "rejected" | "failed" | "cancelled")
}

fn map_item_is_terminal(state: &str) -> bool {
    matches!(state, "completed" | "failed" | "cancelled" | "skipped")
}

/// Finalise une étape Map-Reduce dès que tous ses éléments sont terminaux et produit
/// un extrait agrégé lisible pour les étapes aval (`reduce`).
pub(crate) fn finalize_map_step_if_done(step: &mut NativeGraphStepRecord, now: i64) -> bool {
    let Some(ms) = step.map_state.as_ref() else {
        return false;
    };
    if !ms.items.iter().all(|it| map_item_is_terminal(&it.state)) {
        return false;
    }
    let total = ms.items.len();
    let mut completed = 0usize;
    let mut cancelled = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    let mut lines = Vec::with_capacity(total);
    let mut first_task_id: Option<String> = None;
    for item in &ms.items {
        if first_task_id.is_none() && item.task_id.is_some() {
            first_task_id = item.task_id.clone();
        }
        match item.state.as_str() {
            "completed" => completed += 1,
            "cancelled" => cancelled += 1,
            "skipped" => skipped += 1,
            _ => failed += 1,
        }
        let detail = item
            .result_excerpt
            .as_deref()
            .or(item.error.as_deref())
            .unwrap_or("")
            .replace('\n', " ");
        let clipped_detail: String = detail.chars().take(200).collect();
        let clipped_item: String = item.item.chars().take(120).collect();
        lines.push(format!(
            "[{}/{total}] {clipped_item} : {} | {clipped_detail}",
            item.index.saturating_add(1),
            item.state
        ));
    }
    let summary: String = lines.join("\n").chars().take(4096).collect();
    step.result_excerpt = Some(summary);
    if step.task_id.is_none() {
        step.task_id = first_task_id;
    }
    step.finished_at = Some(now);
    if completed == total {
        step.state = "completed".into();
        step.error = None;
    } else if cancelled == total {
        step.state = "cancelled".into();
        step.error = Some("map_cancelled".into());
    } else {
        step.state = "failed".into();
        step.error = Some(if skipped > 0 {
            format!("map_items_failed:{failed}/{total} (skipped={skipped})")
        } else {
            format!("map_items_failed:{failed}/{total}")
        });
    }
    true
}

pub(crate) fn apply_map_item_completed(
    graph: &mut NativeGraphRecord,
    idx: usize,
    item_index: usize,
    raw_result: Option<&str>,
    keep_warm: bool,
    now: i64,
) {
    let slot_id = {
        let Some(ms) = graph.steps[idx].map_state.as_mut() else {
            return;
        };
        let Some(item) = ms.items.get_mut(item_index) else {
            return;
        };
        item.state = "completed".into();
        item.finished_at = Some(now);
        item.error = None;
        item.result_excerpt = raw_result.map(|r| r.chars().take(400).collect());
        item.worker_slot
    };
    if let Some(sid) = slot_id
        && let Some(ms) = graph.steps[idx].map_state.as_mut()
        && let Some(slot) = ms.worker_slots.iter_mut().find(|s| s.slot_id == sid)
    {
        slot.busy = false;
        slot.rounds_completed = slot.rounds_completed.saturating_add(1);
        // Borne native de 8 rondes par tâche : renouveler proprement avant la limite.
        if !ms.reuse_workers || !keep_warm || slot.rounds_completed >= 7 {
            slot.task_id = None;
        }
    }
    if let Some(raw) = raw_result {
        apply_blackboard_updates(graph, idx, raw, now);
    }
    finalize_map_step_if_done(&mut graph.steps[idx], now);
}

pub(crate) fn apply_map_item_failed(
    graph: &mut NativeGraphRecord,
    idx: usize,
    item_index: usize,
    cancelled: bool,
    error: Option<String>,
    now: i64,
) {
    let Some(ms) = graph.steps[idx].map_state.as_mut() else {
        return;
    };
    let slot_id = if let Some(item) = ms.items.get_mut(item_index) {
        item.state = if cancelled {
            "cancelled".into()
        } else {
            "failed".into()
        };
        item.finished_at = Some(now);
        item.error = error.or_else(|| {
            Some(if cancelled {
                "task_cancelled".into()
            } else {
                "map_item_failed".into()
            })
        });
        item.worker_slot
    } else {
        None
    };
    if let Some(sid) = slot_id
        && let Some(slot) = ms.worker_slots.iter_mut().find(|s| s.slot_id == sid)
    {
        slot.busy = false;
        slot.task_id = None;
    }
    if ms.fail_fast {
        for other in &mut ms.items {
            if other.state == "pending" {
                other.state = "skipped".into();
                other.finished_at = Some(now);
                other.error = Some("map_fail_fast".into());
            }
        }
    }
    finalize_map_step_if_done(&mut graph.steps[idx], now);
}

fn sync_map_step_from_tasks(
    st: &DaemonState,
    graph: &mut NativeGraphRecord,
    idx: usize,
    now: i64,
) -> bool {
    let running_items: Vec<(usize, String, u32)> = graph.steps[idx]
        .map_state
        .as_ref()
        .map(|ms| {
            ms.items
                .iter()
                .filter(|it| it.state == "running")
                .filter_map(|it| {
                    it.task_id
                        .as_ref()
                        .map(|tid| (it.index, tid.clone(), it.expected_round))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut changed = false;
    for (item_index, task_id, expected_round) in running_items {
        let Ok(Some(task)) = st.delegation_store.get(&task_id) else {
            continue;
        };
        match task.state.as_str() {
            "result_available" if task.round >= expected_round => {
                let keep_warm = task.idle_deadline_at.is_some_and(|dl| dl > now);
                apply_map_item_completed(
                    graph,
                    idx,
                    item_index,
                    task.result.as_deref(),
                    keep_warm,
                    now,
                );
                changed = true;
            }
            "failed" => {
                apply_map_item_failed(graph, idx, item_index, false, task.error.clone(), now);
                changed = true;
            }
            "cancelled" => {
                apply_map_item_failed(
                    graph,
                    idx,
                    item_index,
                    true,
                    Some("task_cancelled".into()),
                    now,
                );
                changed = true;
            }
            _ => {}
        }
    }
    if graph.steps[idx].state == "running" && finalize_map_step_if_done(&mut graph.steps[idx], now)
    {
        changed = true;
    }
    changed
}

fn cancel_other_candidates(
    cs: &mut NativeGraphConvergenceState,
    keep_candidate: Option<u8>,
) -> Vec<String> {
    let mut to_cancel = Vec::new();
    for cand in &mut cs.escalation_candidates {
        if keep_candidate == Some(cand.candidate_id) || candidate_is_terminal(&cand.phase) {
            continue;
        }
        if let Some(ptid) = cand.producer_task_id.as_ref()
            && !to_cancel.contains(ptid)
        {
            to_cancel.push(ptid.clone());
        }
        if let Some(vtid) = cand.verifier_task_id.as_ref()
            && !to_cancel.contains(vtid)
        {
            to_cancel.push(vtid.clone());
        }
        cand.phase = "cancelled".into();
        cand.reason
            .get_or_insert_with(|| "cancelled_after_winner".into());
        cs.candidates_cancelled = cs.candidates_cancelled.saturating_add(1);
    }
    to_cancel
}

fn finalize_candidates_if_all_failed(step: &mut NativeGraphStepRecord, now: i64) {
    let Some(cs) = step.convergence_state.as_mut() else {
        return;
    };
    if step.state != "running" || cs.phase != "escalating_candidates" {
        return;
    }
    if cs.winning_candidate_id.is_none()
        && !cs.escalation_candidates.is_empty()
        && cs
            .escalation_candidates
            .iter()
            .all(|c| candidate_is_terminal(&c.phase))
    {
        cs.phase = "done".into();
        cs.outcome = Some("max_iterations_reached".into());
        step.state = "failed".into();
        step.finished_at = Some(now);
        step.error = Some(
            "escalation_best_of_n_all_rejected: aucun candidat parallèle d'escalade n'a été approuvé."
                .into(),
        );
    }
}

pub(crate) fn apply_escalation_candidate_producer_result(
    step: &mut NativeGraphStepRecord,
    candidate_id: u8,
    producer_commit: Option<String>,
    excerpt: Option<String>,
    fs: Value,
    now: i64,
) {
    let has_gate = step.convergence.as_ref().is_some_and(|c| c.gate.is_some());
    if let Some(cs) = step.convergence_state.as_mut()
        && let Some(cand) = cs
            .escalation_candidates
            .iter_mut()
            .find(|c| c.candidate_id == candidate_id)
    {
        if candidate_is_terminal(&cand.phase) {
            return;
        }
        cand.producer_commit = producer_commit;
        cand.result_excerpt = excerpt;
        cand.factual_summary = Some(fs);
        if has_gate {
            if cand.producer_commit.is_none() {
                cand.phase = "failed".into();
                cand.verdict = Some("gate_unavailable".into());
                cand.reason =
                    Some("commit Git attesté absent pour la porte déterministe du candidat".into());
            } else {
                cand.phase = "pending_gate".into();
            }
        } else {
            cand.phase = "pending_verify".into();
        }
    }
    finalize_candidates_if_all_failed(step, now);
}

pub(crate) fn apply_escalation_candidate_gate(
    step: &mut NativeGraphStepRecord,
    candidate_id: u8,
    outcome: crate::deterministic_gate::GateExecOutput,
    now: i64,
) {
    if let Some(cs) = step.convergence_state.as_mut()
        && let Some(cand) = cs
            .escalation_candidates
            .iter_mut()
            .find(|c| c.candidate_id == candidate_id)
    {
        if candidate_is_terminal(&cand.phase) {
            return;
        }
        cs.gate_runs = cs.gate_runs.saturating_add(1);
        if outcome.unavailable_reason.is_none()
            && outcome.passed
            && !outcome.timed_out
            && outcome.exit_code == Some(0)
        {
            cand.gate_passed = Some(true);
            cand.phase = "pending_verify".into();
        } else {
            cs.gate_failures = cs.gate_failures.saturating_add(1);
            if outcome.unavailable_reason.is_none() {
                cs.gate_saved_verifier_calls = cs.gate_saved_verifier_calls.saturating_add(1);
            }
            cand.gate_passed = Some(false);
            cand.verdict = Some(
                if outcome.unavailable_reason.is_some() {
                    "gate_unavailable"
                } else {
                    "gate_failed"
                }
                .into(),
            );
            cand.reason = outcome.unavailable_reason.or_else(|| {
                let txt = outcome.output_excerpt.trim();
                if txt.is_empty() {
                    Some(format!("porte en échec (code {:?})", outcome.exit_code))
                } else {
                    Some(txt.chars().take(400).collect())
                }
            });
            cand.phase = "rejected".into();
        }
    }
    finalize_candidates_if_all_failed(step, now);
}

/// Applique le verdict d'un vérificateur sur un candidat `Best-of-N`.
/// Dès le premier `Approve` (ou `Escalate`), clôt l'étape et renvoie les `task_id`
/// des autres candidats encore actifs à annuler immédiatement.
pub(crate) fn apply_escalation_candidate_verdict(
    step: &mut NativeGraphStepRecord,
    candidate_id: u8,
    parsed: ParsedVerdict,
    now: i64,
) -> Vec<String> {
    let mut to_cancel = Vec::new();
    let Some(cs) = step.convergence_state.as_mut() else {
        return to_cancel;
    };
    if cs.phase != "escalating_candidates" {
        return to_cancel;
    }
    let Some(pos) = cs
        .escalation_candidates
        .iter()
        .position(|c| c.candidate_id == candidate_id)
    else {
        return to_cancel;
    };
    if candidate_is_terminal(&cs.escalation_candidates[pos].phase) {
        return to_cancel;
    }
    match parsed {
        ParsedVerdict::Approve { reason } => {
            cs.escalation_candidates[pos].phase = "approved".into();
            cs.escalation_candidates[pos].verdict = Some("approve".into());
            cs.escalation_candidates[pos].reason = Some(reason);
            let win_ptid = cs.escalation_candidates[pos].producer_task_id.clone();
            let win_commit = cs.escalation_candidates[pos].producer_commit.clone();
            let win_excerpt = cs.escalation_candidates[pos].result_excerpt.clone();
            let win_fs = cs.escalation_candidates[pos].factual_summary.clone();
            to_cancel = cancel_other_candidates(cs, Some(candidate_id));
            cs.winning_candidate_id = Some(candidate_id);
            cs.producer_task_id = win_ptid.clone();
            cs.last_producer_commit = win_commit;
            cs.phase = "done".into();
            cs.outcome = Some("approved".into());
            step.state = "completed".into();
            step.finished_at = Some(now);
            step.error = None;
            step.task_id = win_ptid;
            step.result_excerpt = win_excerpt;
            step.factual_summary = win_fs;
            to_cancel
        }
        ParsedVerdict::Escalate { reason } => {
            cs.escalation_candidates[pos].phase = "rejected".into();
            cs.escalation_candidates[pos].verdict = Some("escalate".into());
            cs.escalation_candidates[pos].reason = Some(reason.clone());
            to_cancel = cancel_other_candidates(cs, Some(candidate_id));
            cs.phase = "done".into();
            cs.outcome = Some("escalated".into());
            step.state = "failed".into();
            step.finished_at = Some(now);
            step.error = Some(format!("convergence_escalated: {reason}"));
            to_cancel
        }
        ParsedVerdict::Revise { reason, .. } => {
            cs.escalation_candidates[pos].phase = "rejected".into();
            cs.escalation_candidates[pos].verdict = Some("revise".into());
            cs.escalation_candidates[pos].reason = Some(reason);
            finalize_candidates_if_all_failed(step, now);
            to_cancel
        }
        ParsedVerdict::Unparseable => {
            cs.escalation_candidates[pos].phase = "rejected".into();
            cs.escalation_candidates[pos].verdict = Some("unparseable".into());
            finalize_candidates_if_all_failed(step, now);
            to_cancel
        }
    }
}

fn sync_escalation_candidates_from_tasks(
    st: &DaemonState,
    step: &mut NativeGraphStepRecord,
    now: i64,
) -> (bool, Vec<String>) {
    let snapshots: Vec<(u8, String, Option<String>, Option<String>)> = step
        .convergence_state
        .as_ref()
        .map(|cs| {
            cs.escalation_candidates
                .iter()
                .map(|c| {
                    (
                        c.candidate_id,
                        c.phase.clone(),
                        c.producer_task_id.clone(),
                        c.verifier_task_id.clone(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let mut changed = false;
    let mut to_cancel = Vec::new();
    for (cid, phase, ptid, vtid) in snapshots {
        if step.state != "running" {
            break;
        }
        match phase.as_str() {
            "producing" => {
                let Some(tid) = ptid.as_deref() else { continue };
                let Ok(Some(task)) = st.delegation_store.get(tid) else {
                    continue;
                };
                match task.state.as_str() {
                    "result_available" => {
                        let fs = super::native_delegation::factual_summary(
                            &task,
                            task.result.as_ref().map(String::len),
                        );
                        let commit = task
                            .workspace
                            .as_ref()
                            .and_then(|ws| ws.head_commit.as_deref())
                            .filter(|sha| is_valid_git_sha40(sha))
                            .or_else(|| extract_commit_from_factual_summary(&fs))
                            .map(str::to_owned);
                        let excerpt = task
                            .result
                            .as_deref()
                            .map(|r| r.chars().take(MAX_DEP_EXCERPT_CHARS).collect());
                        apply_escalation_candidate_producer_result(
                            step, cid, commit, excerpt, fs, now,
                        );
                        changed = true;
                    }
                    "failed" | "cancelled" => {
                        if let Some(cs) = step.convergence_state.as_mut()
                            && let Some(c) = cs
                                .escalation_candidates
                                .iter_mut()
                                .find(|c| c.candidate_id == cid)
                        {
                            c.phase = task.state.clone();
                            c.reason = task.error.clone();
                        }
                        finalize_candidates_if_all_failed(step, now);
                        changed = true;
                    }
                    _ => {}
                }
            }
            "verifying" => {
                let Some(tid) = vtid.as_deref() else { continue };
                let Ok(Some(task)) = st.delegation_store.get(tid) else {
                    continue;
                };
                match task.state.as_str() {
                    "result_available" => {
                        let parsed =
                            parse_convergence_verdict(task.result.as_deref().unwrap_or(""));
                        let cancelled = apply_escalation_candidate_verdict(step, cid, parsed, now);
                        to_cancel.extend(cancelled);
                        changed = true;
                    }
                    "failed" | "cancelled" => {
                        if let Some(cs) = step.convergence_state.as_mut()
                            && let Some(c) = cs
                                .escalation_candidates
                                .iter_mut()
                                .find(|c| c.candidate_id == cid)
                        {
                            c.phase = task.state.clone();
                            c.reason = task.error.clone();
                        }
                        finalize_candidates_if_all_failed(step, now);
                        changed = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (changed, to_cancel)
}

fn sync_steps_from_tasks(
    st: &DaemonState,
    graph: &mut NativeGraphRecord,
    now: i64,
) -> (bool, Vec<String>) {
    let mut changed = false;
    let mut to_cancel = Vec::new();
    for idx in 0..graph.steps.len() {
        if graph.steps[idx].state != "running" {
            continue;
        }
        if graph.steps[idx].map_state.is_some() {
            if sync_map_step_from_tasks(st, graph, idx, now) {
                changed = true;
            }
            continue;
        }
        if graph.steps[idx].convergence.is_some() {
            let prev_state = graph.steps[idx].state.clone();
            let is_best_of_n = graph.steps[idx]
                .convergence_state
                .as_ref()
                .is_some_and(|cs| cs.phase == "escalating_candidates");
            let step_changed = if is_best_of_n {
                let (c, cancels) =
                    sync_escalation_candidates_from_tasks(st, &mut graph.steps[idx], now);
                to_cancel.extend(cancels);
                c
            } else {
                sync_convergence_step_from_tasks(st, &mut graph.steps[idx], now)
            };
            if step_changed {
                changed = true;
                if prev_state != "completed"
                    && graph.steps[idx].state == "completed"
                    && let Some(ptid) = graph.steps[idx].task_id.as_deref()
                    && let Ok(Some(ptask)) = st.delegation_store.get(ptid)
                    && let Some(raw_res) = ptask.result.as_deref()
                {
                    apply_blackboard_updates(graph, idx, raw_res, now);
                }
            }
            continue;
        }
        let Some(task_id) = graph.steps[idx].task_id.clone() else {
            continue;
        };
        let Ok(Some(task)) = st.delegation_store.get(&task_id) else {
            continue;
        };
        match task.state.as_str() {
            "result_available" => {
                graph.steps[idx].state = "completed".into();
                graph.steps[idx].finished_at = Some(now);
                graph.steps[idx].error = None;
                graph.steps[idx].result_excerpt = task
                    .result
                    .as_deref()
                    .map(|r| r.chars().take(MAX_DEP_EXCERPT_CHARS).collect());
                graph.steps[idx].factual_summary = Some(super::native_delegation::factual_summary(
                    &task,
                    task.result.as_ref().map(String::len),
                ));
                if let Some(raw_res) = task.result.as_deref() {
                    apply_blackboard_updates(graph, idx, raw_res, now);
                }
                changed = true;
            }
            "failed" => {
                graph.steps[idx].state = "failed".into();
                graph.steps[idx].finished_at = Some(now);
                graph.steps[idx].error = task.error.clone().or_else(|| Some("task_failed".into()));
                graph.steps[idx].factual_summary =
                    Some(super::native_delegation::factual_summary(&task, None));
                changed = true;
            }
            "cancelled" => {
                graph.steps[idx].state = "cancelled".into();
                graph.steps[idx].finished_at = Some(now);
                graph.steps[idx].error = Some("task_cancelled".into());
                changed = true;
            }
            _ => {}
        }
    }
    (changed, to_cancel)
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
                    let producer_commit = task
                        .workspace
                        .as_ref()
                        .and_then(|ws| ws.head_commit.as_deref())
                        .filter(|sha| is_valid_git_sha40(sha))
                        .or_else(|| extract_commit_from_factual_summary(&fs))
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

/// Tente une escalade unique et bornée vers le palier 1 (modèle/moteur/effort/machine plus fort)
/// sur `stagnated` ou `max_iterations_reached`. Ne se déclenche JAMAIS sur `verdict == "escalate"`.
/// Si l'escalade n'est pas configurée ou déjà consommée (`tier_escalated == true`), termine
/// l'étape en échec avec `trigger` comme `outcome` et renvoie `false`.
fn try_trigger_escalation(
    step: &mut NativeGraphStepRecord,
    trigger: &str,
    reason: String,
    now: i64,
) -> bool {
    let escalation_cfg = step.convergence.as_ref().and_then(|c| c.escalation.clone());
    let Some(cs) = step.convergence_state.as_mut() else {
        return false;
    };
    let eligible = escalation_cfg.as_ref().is_some_and(|esc| {
        !cs.tier_escalated
            && cs.escalation_tier == 0
            && ((trigger == "stagnated" && esc.on_stagnation)
                || (trigger == "max_iterations_reached" && esc.on_max_iterations))
    });
    if eligible && let Some(esc) = escalation_cfg {
        if let Some(old_ptid) = cs.producer_task_id.take()
            && !cs.prior_producer_task_ids.contains(&old_ptid)
        {
            cs.prior_producer_task_ids.push(old_ptid);
        }
        cs.verifier_task_id = None;
        cs.escalation_tier = 1;
        cs.tier_escalated = true;
        cs.escalated_from_reason = Some(trigger.to_string());
        cs.max_iterations = cs
            .max_iterations
            .saturating_add(esc.extra_iterations)
            .min(MAX_TOTAL_CONVERGENCE_ITERATIONS);
        cs.total_max_iterations = cs.max_iterations;
        cs.seen_commits.clear();
        cs.seen_blocking_fingerprints.clear();
        cs.last_blocking_fingerprint = None;
        cs.current_iteration = cs.current_iteration.saturating_add(1);
        if esc.candidates > 1 {
            cs.winning_candidate_id = None;
            cs.escalation_candidates = (0..esc.candidates)
                .map(|cid| NativeGraphEscalationCandidate {
                    candidate_id: cid,
                    phase: "pending_produce".into(),
                    engine: None,
                    model: None,
                    producer_task_id: None,
                    producer_commit: None,
                    verifier_task_id: None,
                    gate_passed: None,
                    verdict: None,
                    reason: None,
                    result_excerpt: None,
                    factual_summary: None,
                })
                .collect();
            cs.phase = "escalating_candidates".into();
        } else {
            cs.phase = "pending_produce".into();
        }
        if let Some(iter_rec) = cs.iterations.last_mut() {
            if iter_rec.reason.is_none() {
                iter_rec.reason = Some(reason);
            }
            iter_rec.finished_at.get_or_insert(now);
        }
        return true;
    }

    if trigger == "stagnated" {
        cs.stagnation_saved_iterations =
            u32::from(cs.max_iterations.saturating_sub(cs.current_iteration));
    }
    if let Some(iter_rec) = cs.iterations.last_mut() {
        iter_rec.finished_at.get_or_insert(now);
    }
    cs.phase = "done".into();
    cs.outcome = Some(trigger.into());
    step.state = "failed".into();
    step.finished_at = Some(now);
    step.error = Some(reason);
    false
}

/// Applique la remise d'une itération producteur sur une étape de convergence,
/// détecte la stagnation / oscillation (`A -> B -> A`) sur l'ensemble des commits vus
/// dans le palier courant, puis bascule vers `pending_gate` (si porte déterministe)
/// ou `pending_verify`.
pub(crate) fn apply_producer_iteration_result(
    step: &mut NativeGraphStepRecord,
    task_round: u32,
    producer_commit: Option<String>,
    excerpt: Option<String>,
    fs: Value,
    now: i64,
) {
    let has_gate = step.convergence.as_ref().is_some_and(|c| c.gate.is_some());
    let Some(cs) = step.convergence_state.as_mut() else {
        return;
    };
    if let Some(iter_rec) = cs.iterations.last_mut() {
        iter_rec.producer_round = task_round;
        iter_rec.producer_commit = producer_commit.clone();
    }
    step.result_excerpt = excerpt;
    step.factual_summary = Some(fs);

    // Détection de stagnation / oscillation #1 : commit déjà vu dans le palier courant
    // (couvre à la fois deux tours consécutifs identiques ET une oscillation A -> B -> A).
    // Le repli `last_producer_commit` n'est utilisé que pour les anciens enregistrements Spec 313
    // désérialisés avec `seen_commits` vide au palier 0.
    let repeated_commit = producer_commit.as_deref().is_some_and(|commit| {
        cs.seen_commits.iter().any(|c| c == commit)
            || (!cs.tier_escalated
                && cs.seen_commits.is_empty()
                && cs.current_iteration >= 2
                && cs.last_producer_commit.as_deref() == Some(commit))
    });
    if cs.stop_on_stagnation
        && repeated_commit
        && let Some(commit) = producer_commit.as_deref()
    {
        let err = format!(
            "convergence_stagnated: commit identique ou oscillant détecté à l'itération {} ({commit})",
            cs.current_iteration
        );
        try_trigger_escalation(step, "stagnated", err, now);
        return;
    }

    if let Some(commit) = producer_commit {
        if !cs.seen_commits.iter().any(|c| c == &commit) {
            cs.seen_commits.push(commit.clone());
        }
        cs.last_producer_commit = Some(commit);
    }

    if has_gate {
        if cs.last_producer_commit.is_none() {
            if let Some(iter_rec) = cs.iterations.last_mut() {
                iter_rec.finished_at = Some(now);
            }
            cs.phase = "done".into();
            cs.outcome = Some("gate_unavailable".into());
            step.state = "failed".into();
            step.finished_at = Some(now);
            step.error = Some(
                "convergence_gate_unavailable: le producteur n'a pas fourni de commit Git attesté pour exécuter la porte déterministe."
                    .into(),
            );
            return;
        }
        cs.phase = "pending_gate".into();
    } else {
        cs.phase = "pending_verify".into();
    }
}

/// Applique le résultat d'une porte déterministe (0 jeton LLM) sur une étape de convergence.
/// - `unavailable_reason.is_some()` : échec fail-closed (`outcome = "gate_unavailable"`).
/// - `passed == true` : porte réussie -> passe à `pending_verify`.
/// - sinon : court-circuite le vérificateur LLM (`gate_saved_verifier_calls += 1`),
///   vérifie stagnation/oscillation + plafond + escalade éventuelle, sinon renvoie au producteur.
pub(crate) fn apply_gate_result(
    step: &mut NativeGraphStepRecord,
    outcome: crate::deterministic_gate::GateExecOutput,
    now: i64,
) {
    let Some(cs) = step.convergence_state.as_mut() else {
        return;
    };
    cs.gate_runs = cs.gate_runs.saturating_add(1);
    if let Some(err) = outcome.unavailable_reason {
        if let Some(iter_rec) = cs.iterations.last_mut() {
            iter_rec.gate_passed = Some(false);
            iter_rec.gate_duration_ms = Some(outcome.duration_ms);
            iter_rec.verdict = Some("gate_unavailable".into());
            iter_rec.reason = Some(err.clone());
            iter_rec.finished_at = Some(now);
        }
        cs.phase = "done".into();
        cs.outcome = Some("gate_unavailable".into());
        step.state = "failed".into();
        step.finished_at = Some(now);
        step.error = Some(format!("convergence_gate_unavailable: {err}"));
        return;
    }
    if outcome.passed && !outcome.timed_out && outcome.exit_code == Some(0) {
        if let Some(iter_rec) = cs.iterations.last_mut() {
            iter_rec.gate_passed = Some(true);
            iter_rec.gate_exit_code = Some(0);
            iter_rec.gate_timed_out = false;
            iter_rec.gate_duration_ms = Some(outcome.duration_ms);
            if !outcome.output_excerpt.trim().is_empty() {
                iter_rec.gate_output_excerpt = Some(outcome.output_excerpt);
            }
        }
        cs.phase = "pending_verify".into();
        return;
    }
    cs.gate_failures = cs.gate_failures.saturating_add(1);
    cs.gate_saved_verifier_calls = cs.gate_saved_verifier_calls.saturating_add(1);
    let fp = crate::deterministic_gate::gate_failure_fingerprint(
        outcome.exit_code,
        outcome.timed_out,
        &outcome.output_excerpt,
    );
    let reason = if outcome.timed_out {
        format!(
            "porte déterministe expirée après {} ms",
            outcome.duration_ms
        )
    } else {
        format!(
            "porte déterministe en échec (code {:?}, {} ms)",
            outcome.exit_code, outcome.duration_ms
        )
    };
    let excerpt = outcome.output_excerpt.clone();
    if let Some(iter_rec) = cs.iterations.last_mut() {
        iter_rec.gate_passed = Some(false);
        iter_rec.gate_exit_code = outcome.exit_code;
        iter_rec.gate_timed_out = outcome.timed_out;
        iter_rec.gate_duration_ms = Some(outcome.duration_ms);
        iter_rec.gate_output_excerpt = Some(excerpt.clone());
        iter_rec.verdict = Some("gate_failed".into());
        iter_rec.reason = Some(reason.clone());
        iter_rec.blocking_points = if excerpt.trim().is_empty() {
            vec![reason.clone()]
        } else {
            vec![excerpt]
        };
        iter_rec.blocking_fingerprint = Some(fp.clone());
        iter_rec.finished_at = Some(now);
    }

    let repeated_fp = cs.seen_blocking_fingerprints.iter().any(|s| s == &fp)
        || (cs.current_iteration >= 2
            && cs.last_blocking_fingerprint.as_deref() == Some(fp.as_str()));
    if cs.stop_on_stagnation && repeated_fp {
        let err = format!(
            "convergence_stagnated: même échec de porte déterministe répété à l'itération {} ({reason})",
            cs.current_iteration
        );
        try_trigger_escalation(step, "stagnated", err, now);
        return;
    }
    if cs.current_iteration >= cs.max_iterations {
        let err = format!(
            "convergence_max_iterations_reached: {} itérations sans passage de la porte ({reason})",
            cs.max_iterations
        );
        try_trigger_escalation(step, "max_iterations_reached", err, now);
        return;
    }
    if !cs.seen_blocking_fingerprints.iter().any(|s| s == &fp) {
        cs.seen_blocking_fingerprints.push(fp.clone());
    }
    cs.last_blocking_fingerprint = Some(fp);
    cs.current_iteration = cs.current_iteration.saturating_add(1);
    cs.phase = "pending_produce".into();
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
            // Un verdict `escalate` court-circuite TOUJOURS l'escalade de modèle intra-étape
            // et remonte immédiatement au parent / humain.
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
            // Détection de stagnation / oscillation #2 : empreinte bloquante déjà vue dans le palier.
            let repeated_fp = cs
                .seen_blocking_fingerprints
                .iter()
                .any(|s| s == &fingerprint)
                || (cs.current_iteration >= 2
                    && cs.last_blocking_fingerprint.as_deref() == Some(fingerprint.as_str()));
            if cs.stop_on_stagnation && repeated_fp {
                let err = format!(
                    "convergence_stagnated: motifs bloquants identiques ou oscillants détectés à l'itération {} ({reason})",
                    cs.current_iteration
                );
                try_trigger_escalation(step, "stagnated", err, now);
                return;
            }
            if cs.current_iteration >= cs.max_iterations {
                let err = format!(
                    "convergence_max_iterations_reached: {} itérations sans approbation ({reason})",
                    cs.max_iterations
                );
                try_trigger_escalation(step, "max_iterations_reached", err, now);
                return;
            }
            if !cs
                .seen_blocking_fingerprints
                .iter()
                .any(|s| s == &fingerprint)
            {
                cs.seen_blocking_fingerprints.push(fingerprint.clone());
            }
            cs.last_blocking_fingerprint = Some(fingerprint);
            cs.current_iteration = cs.current_iteration.saturating_add(1);
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
    ConvergenceGate {
        graph: NativeGraphRecord,
        idx: usize,
        tier: u8,
        iteration: u8,
        producer_task_id: String,
        expected_commit: String,
        gate: GraphDeterministicGate,
    },
    ConvergenceVerify {
        graph: NativeGraphRecord,
        idx: usize,
        tier: u8,
        iteration: u8,
        delegation: AutoDelegateV2,
    },
    ConvergenceNextProduce {
        graph: NativeGraphRecord,
        idx: usize,
        tier: u8,
        iteration: u8,
        continue_task_id: Option<String>,
        continue_request_id: String,
        continue_task: String,
        fallback_delegation: AutoDelegateV2,
    },
    MapItemDispatch {
        graph: NativeGraphRecord,
        idx: usize,
        item_index: usize,
        slot_id: u8,
        continue_task_id: Option<String>,
        continue_request_id: String,
        continue_task: String,
        fallback_delegation: AutoDelegateV2,
    },
    EscalationCandidateProduce {
        graph: NativeGraphRecord,
        idx: usize,
        candidate_id: u8,
        delegation: AutoDelegateV2,
    },
    EscalationCandidateGate {
        graph: NativeGraphRecord,
        idx: usize,
        candidate_id: u8,
        producer_task_id: String,
        expected_commit: String,
        gate: GraphDeterministicGate,
    },
    EscalationCandidateVerify {
        graph: NativeGraphRecord,
        idx: usize,
        candidate_id: u8,
        delegation: AutoDelegateV2,
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
            let (mut dirty, sync_cancels) = sync_steps_from_tasks(&st, &mut graph, now);
            for tid in sync_cancels {
                cancellations.push((graph.clone(), tid));
            }
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
                    if !st.graph_launching_steps.insert(launch_key.clone()) {
                        continue;
                    }
                    if let Some(idx) = graph.steps.iter().position(|s| s.step_id == step_id) {
                        if let Some(map_spec) = graph.steps[idx].map.clone() {
                            st.graph_launching_steps.remove(&launch_key);
                            if let Some(tier) = graph.steps[idx].model_tier {
                                super::native_delegation::resolve_model_tier_pub(
                                    &st,
                                    &mut graph.steps[idx].delegation,
                                    tier,
                                );
                            }
                            let (enriched_task, bb_reads) = build_enriched_step_task(
                                &graph.steps[idx],
                                &graph.steps,
                                &graph.blackboard,
                            );
                            graph.steps[idx].delegation.task = enriched_task;
                            graph.steps[idx].blackboard_reads_injected = bb_reads;
                            graph.steps[idx].map_state = Some(init_map_state(&map_spec));
                            graph.steps[idx].state = "running".into();
                            graph.steps[idx].started_at = Some(now);
                            dirty = true;
                            continue;
                        }
                        let mut delegation = graph.steps[idx].delegation.clone();
                        if let Some(tier) = graph.steps[idx].model_tier {
                            super::native_delegation::resolve_model_tier_pub(
                                &st,
                                &mut delegation,
                                tier,
                            );
                        }
                        let is_frontier =
                            step_is_frontier_launch(graph.steps[idx].model_tier, 0, &delegation);
                        if let Err(reason) = check_graph_budget(&graph, is_frontier, now) {
                            st.graph_launching_steps.remove(&launch_key);
                            fail_step_budget_exhausted(&mut graph, idx, reason, now);
                            dirty = true;
                            continue;
                        }
                        consume_graph_budget(&mut graph, is_frontier);
                        let (enriched_task, bb_reads) = build_enriched_step_task(
                            &graph.steps[idx],
                            &graph.steps,
                            &graph.blackboard,
                        );
                        pin_worktree_base_ref(&mut delegation);
                        delegation.task = enriched_task;
                        if graph.steps[idx].convergence.is_some() {
                            ensure_convergence_producer_keep_alive(&mut delegation);
                        }
                        graph.steps[idx].blackboard_reads_injected = bb_reads;
                        graph.steps[idx].delegation.engine = delegation.engine.clone();
                        graph.steps[idx].delegation.model = delegation.model.clone();
                        graph.steps[idx].delegation.effort = delegation.effort.clone();
                        graph.steps[idx].state = "launching".into();
                        dirty = true;
                        launches.push(GraphLaunchJob::StepStart {
                            graph: graph.clone(),
                            idx,
                            delegation,
                        });
                    }
                }

                // Ordonnancement Map-Reduce borné sur pool de workers chauds (Spec 317).
                for idx in 0..graph.steps.len() {
                    if graph.steps[idx].state != "running" || graph.steps[idx].map_state.is_none() {
                        continue;
                    }
                    let step_id = graph.steps[idx].step_id.clone();
                    // Reprise après redémarrage ou échec de spawn : tout item resté en `launching`
                    // sans clé active dans `graph_launching_steps` repasse `pending` et libère son slot.
                    if let Some(ms) = graph.steps[idx].map_state.as_mut() {
                        let mut freed_slots = Vec::new();
                        for it in &mut ms.items {
                            if it.state == "launching" {
                                let key =
                                    format!("{}:{}:map:item:{}", graph.graph_id, step_id, it.index);
                                if !st.graph_launching_steps.contains(&key) {
                                    it.state = "pending".into();
                                    if let Some(sid) = it.worker_slot.take() {
                                        freed_slots.push(sid);
                                    }
                                    dirty = true;
                                }
                            }
                        }
                        for sid in freed_slots {
                            if let Some(slot) =
                                ms.worker_slots.iter_mut().find(|s| s.slot_id == sid)
                            {
                                slot.busy = false;
                            }
                        }
                    }
                    let total_items = graph.steps[idx]
                        .map_state
                        .as_ref()
                        .map_or(0, |ms| ms.items.len());
                    let is_frontier = step_is_frontier_launch(
                        graph.steps[idx].model_tier,
                        0,
                        &graph.steps[idx].delegation,
                    );
                    loop {
                        let next_pair = graph.steps[idx].map_state.as_ref().and_then(|ms| {
                            let slot = ms.worker_slots.iter().find(|s| !s.busy)?;
                            let item = ms.items.iter().find(|it| it.state == "pending")?;
                            Some((
                                slot.slot_id,
                                if ms.reuse_workers {
                                    slot.task_id.clone()
                                } else {
                                    None
                                },
                                item.index,
                                item.item.clone(),
                                ms.reuse_workers,
                            ))
                        });
                        let Some((slot_id, continue_task_id, item_index, item_text, reuse_workers)) =
                            next_pair
                        else {
                            break;
                        };
                        let launch_key =
                            format!("{}:{}:map:item:{}", graph.graph_id, step_id, item_index);
                        if !st.graph_launching_steps.insert(launch_key.clone()) {
                            break;
                        }
                        if let Err(reason) = check_graph_budget(&graph, is_frontier, now) {
                            st.graph_launching_steps.remove(&launch_key);
                            if let Some(ms) = graph.steps[idx].map_state.as_mut() {
                                for it in &mut ms.items {
                                    if it.state == "pending" {
                                        it.state = "failed".into();
                                        it.finished_at = Some(now);
                                        it.error = Some(format!("budget_exhausted:{reason}"));
                                    }
                                }
                            }
                            let any_active =
                                graph.steps[idx].map_state.as_ref().is_some_and(|ms| {
                                    ms.items.iter().any(|it| {
                                        matches!(it.state.as_str(), "launching" | "running")
                                    })
                                });
                            if !any_active {
                                fail_step_budget_exhausted(&mut graph, idx, reason, now);
                            }
                            dirty = true;
                            break;
                        }
                        consume_graph_budget(&mut graph, is_frontier);
                        let continue_request_id =
                            map_item_request_id(&graph.request_id, &step_id, item_index, "cont");
                        let continue_task = build_map_item_task(
                            &graph.steps[idx].delegation.task,
                            &item_text,
                            item_index,
                            total_items,
                            true,
                        );
                        let mut fallback_del = graph.steps[idx].delegation.clone();
                        fallback_del.request_id =
                            map_item_request_id(&graph.request_id, &step_id, item_index, "new");
                        fallback_del.task = build_map_item_task(
                            &graph.steps[idx].delegation.task,
                            &item_text,
                            item_index,
                            total_items,
                            false,
                        );
                        if reuse_workers {
                            ensure_convergence_producer_keep_alive(&mut fallback_del);
                        }
                        pin_worktree_base_ref(&mut fallback_del);
                        if let Some(ms) = graph.steps[idx].map_state.as_mut() {
                            if let Some(slot) =
                                ms.worker_slots.iter_mut().find(|s| s.slot_id == slot_id)
                            {
                                slot.busy = true;
                            }
                            if let Some(item) = ms.items.get_mut(item_index) {
                                item.state = "launching".into();
                                item.worker_slot = Some(slot_id);
                                item.started_at = Some(now);
                            }
                        }
                        dirty = true;
                        launches.push(GraphLaunchJob::MapItemDispatch {
                            graph: graph.clone(),
                            idx,
                            item_index,
                            slot_id,
                            continue_task_id,
                            continue_request_id,
                            continue_task,
                            fallback_delegation: fallback_del,
                        });
                    }
                }

                // Transitions internes des étapes de convergence (`pending_gate` / `pending_verify` / `pending_produce`).
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
                    if matches!(cs.phase.as_str(), "pending_gate" | "launching_gate")
                        && let Some(gate) = conv.gate.clone()
                    {
                        let launch_key = format!(
                            "{}:{}:gate:t{}:i{}",
                            graph.graph_id, step_id, cs.escalation_tier, cs.current_iteration
                        );
                        if !st.graph_launching_steps.insert(launch_key) {
                            continue;
                        }
                        let (Some(producer_task_id), Some(expected_commit)) =
                            (cs.producer_task_id.clone(), cs.last_producer_commit.clone())
                        else {
                            apply_gate_result(
                                &mut graph.steps[idx],
                                crate::deterministic_gate::GateExecOutput {
                                    passed: false,
                                    exit_code: None,
                                    timed_out: false,
                                    duration_ms: 0,
                                    output_excerpt: String::new(),
                                    unavailable_reason: Some(
                                        "tâche productrice ou commit absent pour la porte déterministe"
                                            .into(),
                                    ),
                                },
                                now,
                            );
                            dirty = true;
                            continue;
                        };
                        if let Some(state_mut) = graph.steps[idx].convergence_state.as_mut() {
                            state_mut.phase = "launching_gate".into();
                        }
                        dirty = true;
                        launches.push(GraphLaunchJob::ConvergenceGate {
                            graph: graph.clone(),
                            idx,
                            tier: cs.escalation_tier,
                            iteration: cs.current_iteration,
                            producer_task_id,
                            expected_commit,
                            gate,
                        });
                    } else if matches!(cs.phase.as_str(), "pending_verify" | "launching_verify") {
                        let launch_key = format!(
                            "{}:{}:verify:t{}:i{}",
                            graph.graph_id, step_id, cs.escalation_tier, cs.current_iteration
                        );
                        if !st.graph_launching_steps.insert(launch_key.clone()) {
                            continue;
                        }
                        let mut verify_del = conv.verify.clone();
                        let verify_is_frontier = step_is_frontier_launch(None, 0, &verify_del);
                        if let Err(reason) = check_graph_budget(&graph, verify_is_frontier, now) {
                            st.graph_launching_steps.remove(&launch_key);
                            fail_step_budget_exhausted(&mut graph, idx, reason, now);
                            dirty = true;
                            continue;
                        }
                        consume_graph_budget(&mut graph, verify_is_frontier);
                        verify_del.request_id = convergence_step_request_id(
                            &graph.request_id,
                            &step_id,
                            cs.escalation_tier,
                            cs.current_iteration,
                            "verify",
                        );
                        // Vérificateur orthogonal (`different_provider: true`) : préférer un moteur
                        // sain différent de la famille du producteur, avec repli gracieux.
                        let mut orthogonal_flag: Option<bool> = None;
                        if conv.different_provider {
                            let prod_engine_raw = cs
                                .producer_task_id
                                .as_deref()
                                .and_then(|ptid| st.delegation_store.get(ptid).ok().flatten())
                                .map(|t| t.definition.command)
                                .or_else(|| graph.steps[idx].delegation.engine.clone());
                            let prod_fam = prod_engine_raw
                                .as_deref()
                                .map(super::native_delegation::normalize_engine_family)
                                .unwrap_or("");
                            let verify_fam = verify_del
                                .engine
                                .as_deref()
                                .map(super::native_delegation::normalize_engine_family)
                                .unwrap_or("");
                            if prod_fam.is_empty() {
                                orthogonal_flag = Some(false);
                            } else if !verify_fam.is_empty() && verify_fam != prod_fam {
                                orthogonal_flag = Some(true);
                            } else if let Some(orth) =
                                super::native_delegation::pick_orthogonal_engine_pub(
                                    &st,
                                    prod_engine_raw.as_deref(),
                                )
                            {
                                verify_del.engine = Some(orth);
                                verify_del.model = None;
                                verify_del.effort = None;
                                orthogonal_flag = Some(true);
                            } else {
                                orthogonal_flag = Some(false);
                            }
                        }
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
                            if let Some(iter_rec) = state_mut.iterations.last_mut() {
                                iter_rec.verifier_engine = verify_del.engine.clone();
                                iter_rec.verifier_model = verify_del.model.clone();
                                if orthogonal_flag.is_some() {
                                    iter_rec.verifier_orthogonal = orthogonal_flag;
                                }
                            }
                        }
                        dirty = true;
                        launches.push(GraphLaunchJob::ConvergenceVerify {
                            graph: graph.clone(),
                            idx,
                            tier: cs.escalation_tier,
                            iteration: cs.current_iteration,
                            delegation: verify_del,
                        });
                    } else if matches!(cs.phase.as_str(), "pending_produce" | "launching_produce") {
                        let launch_key = format!(
                            "{}:{}:produce:t{}:i{}",
                            graph.graph_id, step_id, cs.escalation_tier, cs.current_iteration
                        );
                        if !st.graph_launching_steps.insert(launch_key.clone()) {
                            continue;
                        }
                        let produce_is_frontier = step_is_frontier_launch(
                            graph.steps[idx].model_tier,
                            cs.escalation_tier,
                            &graph.steps[idx].delegation,
                        );
                        if let Err(reason) = check_graph_budget(&graph, produce_is_frontier, now) {
                            st.graph_launching_steps.remove(&launch_key);
                            fail_step_budget_exhausted(&mut graph, idx, reason, now);
                            dirty = true;
                            continue;
                        }
                        consume_graph_budget(&mut graph, produce_is_frontier);
                        let (last_reason, last_points, from_gate) = cs
                            .iterations
                            .last()
                            .map(|it| {
                                (
                                    it.reason
                                        .as_deref()
                                        .unwrap_or("corrections demandées")
                                        .to_string(),
                                    it.blocking_points.clone(),
                                    it.verdict.as_deref() == Some("gate_failed"),
                                )
                            })
                            .unwrap_or_else(|| ("corrections demandées".into(), Vec::new(), false));
                        let continue_request_id = convergence_step_request_id(
                            &graph.request_id,
                            &step_id,
                            cs.escalation_tier,
                            cs.current_iteration,
                            "produce",
                        );
                        let continue_task = build_producer_revision_task(
                            &step_id,
                            cs.escalation_tier,
                            cs.current_iteration,
                            cs.max_iterations,
                            &last_reason,
                            &last_points,
                            &graph.steps[idx].delegation.task,
                            false,
                            from_gate,
                        );
                        let mut fallback_del = graph.steps[idx].delegation.clone();
                        // Appliquer les surcharges d'escalade sur tout lancement neuf du palier 1.
                        if cs.escalation_tier > 0
                            && let Some(esc) = conv.escalation.as_ref()
                        {
                            if let Some(eng) = esc.engine.as_ref() {
                                fallback_del.engine = Some(eng.clone());
                                if esc.model.is_none() {
                                    fallback_del.model = None;
                                }
                            }
                            if let Some(model) = esc.model.as_ref() {
                                fallback_del.model = Some(model.clone());
                            }
                            if let Some(effort) = esc.effort.as_ref() {
                                fallback_del.effort = Some(effort.clone());
                            }
                            if let Some(machine_id) = esc.machine_id.as_ref() {
                                fallback_del.machine_id = Some(machine_id.clone());
                            }
                        }
                        fallback_del.request_id = convergence_step_request_id(
                            &graph.request_id,
                            &step_id,
                            cs.escalation_tier,
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
                            cs.escalation_tier,
                            cs.current_iteration,
                            cs.max_iterations,
                            &last_reason,
                            &last_points,
                            &graph.steps[idx].delegation.task,
                            true,
                            from_gate,
                        );
                        if let Some(state_mut) = graph.steps[idx].convergence_state.as_mut() {
                            state_mut.phase = "launching_produce".into();
                        }
                        dirty = true;
                        launches.push(GraphLaunchJob::ConvergenceNextProduce {
                            graph: graph.clone(),
                            idx,
                            tier: cs.escalation_tier,
                            iteration: cs.current_iteration,
                            continue_task_id: cs.producer_task_id.clone(),
                            continue_request_id,
                            continue_task,
                            fallback_delegation: fallback_del,
                        });
                    } else if cs.phase == "escalating_candidates" {
                        // Escalade Best-of-N bornée (Spec 317) : candidats indépendants en parallèle,
                        // diversifiés par fournisseur sain, filtrés par gate avant verify.
                        let (last_reason, last_points, from_gate) = cs
                            .iterations
                            .last()
                            .map(|it| {
                                (
                                    it.reason
                                        .as_deref()
                                        .unwrap_or("escalade parallèle sur blocage")
                                        .to_string(),
                                    it.blocking_points.clone(),
                                    it.verdict.as_deref() == Some("gate_failed"),
                                )
                            })
                            .unwrap_or_else(|| {
                                ("escalade parallèle sur blocage".into(), Vec::new(), false)
                            });
                        let mut used_families: Vec<String> = cs
                            .escalation_candidates
                            .iter()
                            .filter_map(|c| c.engine.as_deref())
                            .map(super::native_delegation::normalize_engine_family)
                            .filter(|f| !f.is_empty())
                            .map(str::to_string)
                            .collect();
                        let total_cands = cs.escalation_candidates.len();
                        for c_idx in 0..total_cands {
                            if graph.steps[idx].state != "running" {
                                break;
                            }
                            let cand = match graph.steps[idx]
                                .convergence_state
                                .as_ref()
                                .and_then(|s| s.escalation_candidates.get(c_idx))
                            {
                                Some(c) => c.clone(),
                                None => break,
                            };
                            let cid = cand.candidate_id;
                            match cand.phase.as_str() {
                                "launching_produce" => {
                                    let launch_key = format!(
                                        "{}:{}:esc_cand:{}:produce",
                                        graph.graph_id, step_id, cid
                                    );
                                    if !st.graph_launching_steps.contains(&launch_key)
                                        && let Some(cs_mut) =
                                            graph.steps[idx].convergence_state.as_mut()
                                        && let Some(c_mut) =
                                            cs_mut.escalation_candidates.get_mut(c_idx)
                                    {
                                        c_mut.phase = "pending_produce".into();
                                        dirty = true;
                                    }
                                }
                                "launching_gate" => {
                                    let launch_key = format!(
                                        "{}:{}:esc_cand:{}:gate",
                                        graph.graph_id, step_id, cid
                                    );
                                    if !st.graph_launching_steps.contains(&launch_key)
                                        && let Some(cs_mut) =
                                            graph.steps[idx].convergence_state.as_mut()
                                        && let Some(c_mut) =
                                            cs_mut.escalation_candidates.get_mut(c_idx)
                                    {
                                        c_mut.phase = "pending_gate".into();
                                        dirty = true;
                                    }
                                }
                                "launching_verify" => {
                                    let launch_key = format!(
                                        "{}:{}:esc_cand:{}:verify",
                                        graph.graph_id, step_id, cid
                                    );
                                    if !st.graph_launching_steps.contains(&launch_key)
                                        && let Some(cs_mut) =
                                            graph.steps[idx].convergence_state.as_mut()
                                        && let Some(c_mut) =
                                            cs_mut.escalation_candidates.get_mut(c_idx)
                                    {
                                        c_mut.phase = "pending_verify".into();
                                        dirty = true;
                                    }
                                }
                                "pending_produce" => {
                                    let launch_key = format!(
                                        "{}:{}:esc_cand:{}:produce",
                                        graph.graph_id, step_id, cid
                                    );
                                    if !st.graph_launching_steps.insert(launch_key.clone()) {
                                        continue;
                                    }
                                    if let Err(reason) = check_graph_budget(&graph, true, now) {
                                        st.graph_launching_steps.remove(&launch_key);
                                        if let Some(cs_mut) =
                                            graph.steps[idx].convergence_state.as_mut()
                                            && let Some(c_mut) =
                                                cs_mut.escalation_candidates.get_mut(c_idx)
                                        {
                                            c_mut.phase = "cancelled".into();
                                            c_mut.reason =
                                                Some(format!("budget_exhausted:{reason}"));
                                        }
                                        let all_done = graph.steps[idx]
                                            .convergence_state
                                            .as_ref()
                                            .is_some_and(|s| {
                                                s.escalation_candidates
                                                    .iter()
                                                    .all(|c| candidate_is_terminal(&c.phase))
                                            });
                                        if all_done {
                                            fail_step_budget_exhausted(
                                                &mut graph, idx, reason, now,
                                            );
                                        }
                                        dirty = true;
                                        continue;
                                    }
                                    consume_graph_budget(&mut graph, true);
                                    let mut cand_del = graph.steps[idx].delegation.clone();
                                    if let Some(esc) = conv.escalation.as_ref() {
                                        if let Some(eng) = esc.engine.as_ref() {
                                            cand_del.engine = Some(eng.clone());
                                            if esc.model.is_none() {
                                                cand_del.model = None;
                                            }
                                        }
                                        if let Some(model) = esc.model.as_ref() {
                                            cand_del.model = Some(model.clone());
                                        }
                                        if let Some(effort) = esc.effort.as_ref() {
                                            cand_del.effort = Some(effort.clone());
                                        }
                                        if let Some(machine_id) = esc.machine_id.as_ref() {
                                            cand_del.machine_id = Some(machine_id.clone());
                                        }
                                    }
                                    if cid > 0 {
                                        let excluded: Vec<&str> =
                                            used_families.iter().map(String::as_str).collect();
                                        if let Some(diverse) = super::native_delegation::pick_diverse_healthy_engine_pub(
                                            &st,
                                            &excluded,
                                        ) {
                                            cand_del.engine = Some(diverse.clone());
                                            cand_del.model = None;
                                            cand_del.effort = None;
                                            used_families.push(diverse);
                                        }
                                    } else if let Some(fam) = cand_del
                                        .engine
                                        .as_deref()
                                        .map(super::native_delegation::normalize_engine_family)
                                        .filter(|f| !f.is_empty())
                                    {
                                        if !used_families.iter().any(|u| u == fam) {
                                            used_families.push(fam.to_string());
                                        }
                                    }
                                    // Garantir des worktrees isolés sans collision de branche entre candidats.
                                    match &mut cand_del.workspace {
                                        Some(WorkspaceRequest::Worktree { branch, .. }) => {
                                            *branch = None;
                                        }
                                        None if cand_del.expected_base_commit.is_some()
                                            && cand_del.posture
                                                != Some(SpawnPosture::Discovery) =>
                                        {
                                            cand_del.workspace = Some(WorkspaceRequest::Worktree {
                                                base_ref: cs.last_producer_commit.clone().or_else(
                                                    || cand_del.expected_base_commit.clone(),
                                                ),
                                                branch: None,
                                            });
                                        }
                                        _ => {}
                                    }
                                    apply_commit_to_delegation(
                                        &mut cand_del,
                                        cs.last_producer_commit.as_deref(),
                                    );
                                    cand_del.request_id = convergence_step_request_id(
                                        &graph.request_id,
                                        &step_id,
                                        1,
                                        cs.current_iteration,
                                        &format!("cand{cid}-prod"),
                                    );
                                    let mut cand_task = build_producer_revision_task(
                                        &step_id,
                                        1,
                                        cs.current_iteration,
                                        cs.max_iterations,
                                        &last_reason,
                                        &last_points,
                                        &graph.steps[idx].delegation.task,
                                        true,
                                        from_gate,
                                    );
                                    cand_task.push_str(&format!(
                                        "\nCandidat parallèle Best-of-N #{}/{total_cands} : explore une approche propre et autonome.\n",
                                        cid.saturating_add(1)
                                    ));
                                    cand_del.task = cand_task;
                                    if let Some(cs_mut) =
                                        graph.steps[idx].convergence_state.as_mut()
                                        && let Some(c_mut) =
                                            cs_mut.escalation_candidates.get_mut(c_idx)
                                    {
                                        c_mut.phase = "launching_produce".into();
                                        c_mut.engine = cand_del.engine.clone();
                                        c_mut.model = cand_del.model.clone();
                                    }
                                    dirty = true;
                                    launches.push(GraphLaunchJob::EscalationCandidateProduce {
                                        graph: graph.clone(),
                                        idx,
                                        candidate_id: cid,
                                        delegation: cand_del,
                                    });
                                }
                                "pending_gate" => {
                                    let Some(gate) = conv.gate.clone() else {
                                        if let Some(cs_mut) =
                                            graph.steps[idx].convergence_state.as_mut()
                                            && let Some(c_mut) =
                                                cs_mut.escalation_candidates.get_mut(c_idx)
                                        {
                                            c_mut.phase = "pending_verify".into();
                                        }
                                        dirty = true;
                                        continue;
                                    };
                                    let launch_key = format!(
                                        "{}:{}:esc_cand:{}:gate",
                                        graph.graph_id, step_id, cid
                                    );
                                    if !st.graph_launching_steps.insert(launch_key.clone()) {
                                        continue;
                                    }
                                    let (Some(ptid), Some(commit)) = (
                                        cand.producer_task_id.clone(),
                                        cand.producer_commit.clone(),
                                    ) else {
                                        st.graph_launching_steps.remove(&launch_key);
                                        apply_escalation_candidate_gate(
                                            &mut graph.steps[idx],
                                            cid,
                                            crate::deterministic_gate::GateExecOutput {
                                                passed: false,
                                                exit_code: None,
                                                timed_out: false,
                                                duration_ms: 0,
                                                output_excerpt: String::new(),
                                                unavailable_reason: Some(
                                                    "tâche ou commit candidat absent pour la porte déterministe"
                                                        .into(),
                                                ),
                                            },
                                            now,
                                        );
                                        dirty = true;
                                        continue;
                                    };
                                    if let Some(cs_mut) =
                                        graph.steps[idx].convergence_state.as_mut()
                                        && let Some(c_mut) =
                                            cs_mut.escalation_candidates.get_mut(c_idx)
                                    {
                                        c_mut.phase = "launching_gate".into();
                                    }
                                    dirty = true;
                                    launches.push(GraphLaunchJob::EscalationCandidateGate {
                                        graph: graph.clone(),
                                        idx,
                                        candidate_id: cid,
                                        producer_task_id: ptid,
                                        expected_commit: commit,
                                        gate,
                                    });
                                }
                                "pending_verify" => {
                                    let launch_key = format!(
                                        "{}:{}:esc_cand:{}:verify",
                                        graph.graph_id, step_id, cid
                                    );
                                    if !st.graph_launching_steps.insert(launch_key.clone()) {
                                        continue;
                                    }
                                    let mut verify_del = conv.verify.clone();
                                    let verify_is_frontier =
                                        step_is_frontier_launch(None, 0, &verify_del);
                                    if let Err(reason) =
                                        check_graph_budget(&graph, verify_is_frontier, now)
                                    {
                                        st.graph_launching_steps.remove(&launch_key);
                                        if let Some(cs_mut) =
                                            graph.steps[idx].convergence_state.as_mut()
                                            && let Some(c_mut) =
                                                cs_mut.escalation_candidates.get_mut(c_idx)
                                        {
                                            c_mut.phase = "cancelled".into();
                                            c_mut.reason =
                                                Some(format!("budget_exhausted:{reason}"));
                                        }
                                        let all_done = graph.steps[idx]
                                            .convergence_state
                                            .as_ref()
                                            .is_some_and(|s| {
                                                s.escalation_candidates
                                                    .iter()
                                                    .all(|c| candidate_is_terminal(&c.phase))
                                            });
                                        if all_done {
                                            fail_step_budget_exhausted(
                                                &mut graph, idx, reason, now,
                                            );
                                        }
                                        dirty = true;
                                        continue;
                                    }
                                    consume_graph_budget(&mut graph, verify_is_frontier);
                                    verify_del.request_id = convergence_step_request_id(
                                        &graph.request_id,
                                        &step_id,
                                        1,
                                        cs.current_iteration,
                                        &format!("cand{cid}-ver"),
                                    );
                                    if conv.different_provider {
                                        let prod_engine_raw = cand
                                            .producer_task_id
                                            .as_deref()
                                            .and_then(|ptid| {
                                                st.delegation_store.get(ptid).ok().flatten()
                                            })
                                            .map(|t| t.definition.command)
                                            .or_else(|| cand.engine.clone());
                                        let prod_fam = prod_engine_raw
                                            .as_deref()
                                            .map(super::native_delegation::normalize_engine_family)
                                            .unwrap_or("");
                                        let verify_fam = verify_del
                                            .engine
                                            .as_deref()
                                            .map(super::native_delegation::normalize_engine_family)
                                            .unwrap_or("");
                                        if !prod_fam.is_empty()
                                            && (verify_fam.is_empty() || verify_fam == prod_fam)
                                            && let Some(orth) =
                                                super::native_delegation::pick_orthogonal_engine_pub(
                                                    &st,
                                                    prod_engine_raw.as_deref(),
                                                )
                                        {
                                            verify_del.engine = Some(orth);
                                            verify_del.model = None;
                                            verify_del.effort = None;
                                        }
                                    }
                                    apply_commit_to_delegation(
                                        &mut verify_del,
                                        cand.producer_commit.as_deref(),
                                    );
                                    verify_del.task = build_candidate_verifier_task(
                                        &graph.steps[idx],
                                        &conv.verify,
                                        &cand,
                                        total_cands,
                                    );
                                    if let Some(cs_mut) =
                                        graph.steps[idx].convergence_state.as_mut()
                                        && let Some(c_mut) =
                                            cs_mut.escalation_candidates.get_mut(c_idx)
                                    {
                                        c_mut.phase = "launching_verify".into();
                                    }
                                    dirty = true;
                                    launches.push(GraphLaunchJob::EscalationCandidateVerify {
                                        graph: graph.clone(),
                                        idx,
                                        candidate_id: cid,
                                        delegation: verify_del,
                                    });
                                }
                                _ => {}
                            }
                        }
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
                persist_graph_with_pending_metrics(&st, &mut graph, now);
            }
        }
    }

    if !controls.is_empty() {
        let _ = execute_controls(controls);
    }

    for (graph, task_id) in cancellations {
        let _ = cancel_task_for_graph(&graph, &task_id, state);
    }

    struct LaunchKeyGuard {
        state: Arc<Mutex<DaemonState>>,
        key: String,
    }
    impl Drop for LaunchKeyGuard {
        fn drop(&mut self) {
            let mut st = self.state.lock().recover_poison();
            st.graph_launching_steps.remove(&self.key);
        }
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
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
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
                            let now = unix_timestamp();
                            apply_admission_response_locked(
                                &mut current,
                                pos,
                                &step_id,
                                response,
                                suppress,
                                &st,
                                now,
                            );
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            persist_graph_with_pending_metrics(&st, &mut current, now);
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
            GraphLaunchJob::ConvergenceGate {
                graph: graph_snapshot,
                idx,
                tier,
                iteration,
                producer_task_id,
                expected_commit,
                gate,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let _ = thread::Builder::new()
                    .name(format!("bridget-graph-gate-{}", &step_id))
                    .spawn(move || {
                        let launch_key = format!("{graph_id}:{step_id}:gate:t{tier}:i{iteration}");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let gate_res =
                            super::native_delegation::run_deterministic_gate_for_task_pub(
                                &state_bg,
                                &producer_task_id,
                                &expected_commit,
                                &gate.argv,
                                gate.timeout_secs,
                            );
                        let mut st = state_bg.lock().recover_poison();
                        st.graph_launching_steps.remove(&launch_key);
                        if let Ok(Some(mut current)) = st.delegation_store.get_graph(&graph_id)
                            && let Some(pos) =
                                current.steps.iter().position(|s| s.step_id == step_id)
                            && current.steps[pos].state == "running"
                            && current.steps[pos]
                                .convergence_state
                                .as_ref()
                                .is_some_and(|cs| cs.phase == "launching_gate")
                        {
                            let now = unix_timestamp();
                            apply_gate_result(&mut current.steps[pos], gate_res, now);
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            persist_graph_with_pending_metrics(&st, &mut current, now);
                        }
                    });
            }
            GraphLaunchJob::ConvergenceVerify {
                graph: graph_snapshot,
                idx,
                tier,
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
                        let launch_key =
                            format!("{graph_id}:{step_id}:verify:t{tier}:i{iteration}");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let exec_tag = format!("{step_id}-verify-t{tier}-i{iteration}");
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
                                    let hint = if code == "expected_base_commit_mismatch"
                                        || code == "base_commit_not_found"
                                    {
                                        " Conseil : épinglez workspace.base_ref au même SHA que expected_base_commit (ou utilisez strategy='commit_view') ; si le producteur a tourné sur une machine distante différente de l'hôte d'origine, poussez ou placez producteur/vérificateur sur l'hôte source."
                                    } else {
                                        ""
                                    };
                                    current.steps[pos].state = "failed".into();
                                    current.steps[pos].error = Some(
                                        format!("convergence_verify_failed:{code}: {msg}{hint}")
                                            .chars()
                                            .take(640)
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
                            persist_graph_with_pending_metrics(&st, &mut current, now);
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
                tier,
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
                        let launch_key =
                            format!("{graph_id}:{step_id}:produce:t{tier}:i{iteration}");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let exec_tag = format!("{step_id}-produce-t{tier}-i{iteration}");
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
                                let prod_engine = current.steps[pos].delegation.engine.clone();
                                let prod_model = current.steps[pos].delegation.model.clone();
                                if let Some(cs) = current.steps[pos].convergence_state.as_mut() {
                                    cs.phase = "producing".into();
                                    cs.producer_task_id = Some(ptid.clone());
                                    cs.producer_expected_round = expected_round;
                                    cs.verifier_task_id = None;
                                    cs.iterations.push(NativeGraphConvergenceIteration {
                                        iteration,
                                        tier: cs.escalation_tier,
                                        producer_task_id: Some(ptid),
                                        producer_round: expected_round,
                                        producer_commit: None,
                                        producer_engine: prod_engine,
                                        producer_model: prod_model,
                                        gate_passed: None,
                                        gate_exit_code: None,
                                        gate_timed_out: false,
                                        gate_duration_ms: None,
                                        gate_output_excerpt: None,
                                        verifier_task_id: None,
                                        verifier_engine: None,
                                        verifier_model: None,
                                        verifier_orthogonal: None,
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
                                        let prod_engine = current.steps[pos]
                                            .convergence
                                            .as_ref()
                                            .and_then(|c| c.escalation.as_ref())
                                            .filter(|_| tier > 0)
                                            .and_then(|e| e.engine.clone())
                                            .or_else(|| {
                                                current.steps[pos].delegation.engine.clone()
                                            });
                                        let prod_model = current.steps[pos]
                                            .convergence
                                            .as_ref()
                                            .and_then(|c| c.escalation.as_ref())
                                            .filter(|_| tier > 0)
                                            .and_then(|e| e.model.clone())
                                            .or_else(|| {
                                                current.steps[pos].delegation.model.clone()
                                            });
                                        if let Some(cs) =
                                            current.steps[pos].convergence_state.as_mut()
                                        {
                                            cs.phase = "producing".into();
                                            cs.producer_task_id = Some(new_tid.clone());
                                            cs.producer_expected_round = 1;
                                            cs.verifier_task_id = None;
                                            cs.iterations.push(NativeGraphConvergenceIteration {
                                                iteration,
                                                tier: cs.escalation_tier,
                                                producer_task_id: Some(new_tid.clone()),
                                                producer_round: 1,
                                                producer_commit: None,
                                                producer_engine: prod_engine,
                                                producer_model: prod_model,
                                                gate_passed: None,
                                                gate_exit_code: None,
                                                gate_timed_out: false,
                                                gate_duration_ms: None,
                                                gate_output_excerpt: None,
                                                verifier_task_id: None,
                                                verifier_engine: None,
                                                verifier_model: None,
                                                verifier_orthogonal: None,
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
                            persist_graph_with_pending_metrics(&st, &mut current, now);
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
            GraphLaunchJob::MapItemDispatch {
                graph: graph_snapshot,
                idx,
                item_index,
                slot_id,
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
                    .name(format!("bridget-map-{}-{}", &step_id, item_index))
                    .spawn(move || {
                        let launch_key = format!("{graph_id}:{step_id}:map:item:{item_index}");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let exec_tag = format!("{step_id}-map-{item_index}");
                        let mut continued: Option<(String, u32)> = None;
                        let mut fallback_response: Option<DaemonToWrapper> = None;

                        match prepare_exec_conn(&graph_snapshot, &exec_tag, &state_bg) {
                            Ok((exec_conn, _guard)) => {
                                if let Some(tid) = continue_task_id.as_deref() {
                                    let cont_resp = super::native_delegation::handle_with_fact_pub(
                                        &exec_conn,
                                        NativeDelegationRequest::Continue {
                                            task_id: tid.to_string(),
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
                                        let round =
                                            result["round"].as_u64().map(|r| r as u32).unwrap_or(2);
                                        continued = Some((tid.to_string(), round));
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
                                .map_state
                                .as_ref()
                                .and_then(|ms| ms.items.get(item_index))
                                .is_some_and(|it| it.state == "launching")
                        {
                            let now = unix_timestamp();
                            if let Some((tid, expected_round)) = continued {
                                if let Some(ms) = current.steps[pos].map_state.as_mut() {
                                    if let Some(it) = ms.items.get_mut(item_index) {
                                        it.state = "running".into();
                                        it.task_id = Some(tid.clone());
                                        it.expected_round = expected_round;
                                        it.reused_worker = true;
                                    }
                                    if let Some(slot) =
                                        ms.worker_slots.iter_mut().find(|s| s.slot_id == slot_id)
                                    {
                                        slot.task_id = Some(tid);
                                    }
                                    ms.reused_worker_hits = ms.reused_worker_hits.saturating_add(1);
                                }
                            } else if let Some(response) = fallback_response {
                                match response {
                                    DaemonToWrapper::NativeDelegationResult { result }
                                        if result["status"] != "refused"
                                            && result["task_id"].as_str().is_some() =>
                                    {
                                        let new_tid =
                                            result["task_id"].as_str().unwrap().to_string();
                                        if let Some(ms) = current.steps[pos].map_state.as_mut() {
                                            if let Some(it) = ms.items.get_mut(item_index) {
                                                it.state = "running".into();
                                                it.task_id = Some(new_tid.clone());
                                                it.expected_round = 1;
                                                it.reused_worker = false;
                                            }
                                            if let Some(slot) = ms
                                                .worker_slots
                                                .iter_mut()
                                                .find(|s| s.slot_id == slot_id)
                                            {
                                                slot.task_id = Some(new_tid.clone());
                                                slot.rounds_completed = 0;
                                            }
                                        }
                                        let _ = st.delegation_store.bind_graph_step_task(
                                            &new_tid, &graph_id, &step_id, suppress,
                                        );
                                    }
                                    DaemonToWrapper::NativeDelegationResult { result } => {
                                        let code = result["code"]
                                            .as_str()
                                            .or_else(|| result["error"].as_str())
                                            .unwrap_or("map_admission_refused");
                                        let msg = result["message"].as_str().unwrap_or(code);
                                        apply_map_item_failed(
                                            &mut current,
                                            pos,
                                            item_index,
                                            false,
                                            Some(
                                                format!("{code}: {msg}")
                                                    .chars()
                                                    .take(512)
                                                    .collect(),
                                            ),
                                            now,
                                        );
                                    }
                                    other => {
                                        apply_map_item_failed(
                                            &mut current,
                                            pos,
                                            item_index,
                                            false,
                                            Some(
                                                format!("map_unexpected:{other:?}")
                                                    .chars()
                                                    .take(256)
                                                    .collect(),
                                            ),
                                            now,
                                        );
                                    }
                                }
                            }
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            persist_graph_with_pending_metrics(&st, &mut current, now);
                        } else {
                            let orphan_tid = continued.map(|(tid, _)| tid).or_else(|| {
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
            GraphLaunchJob::EscalationCandidateProduce {
                graph: graph_snapshot,
                idx,
                candidate_id,
                delegation,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let suppress =
                    graph_snapshot.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
                let _ = thread::Builder::new()
                    .name(format!("bridget-esc-prod-{}-c{}", &step_id, candidate_id))
                    .spawn(move || {
                        let launch_key =
                            format!("{graph_id}:{step_id}:esc_cand:{candidate_id}:produce");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let exec_tag = format!("{step_id}-esc-c{candidate_id}-prod");
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
                                .is_some_and(|cs| {
                                    cs.phase == "escalating_candidates"
                                        && cs.escalation_candidates.iter().any(|c| {
                                            c.candidate_id == candidate_id
                                                && c.phase == "launching_produce"
                                        })
                                })
                        {
                            let now = unix_timestamp();
                            match response {
                                DaemonToWrapper::NativeDelegationResult { result }
                                    if result["status"] != "refused"
                                        && result["task_id"].as_str().is_some() =>
                                {
                                    let tid = result["task_id"].as_str().unwrap().to_string();
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                        && let Some(cand) = cs
                                            .escalation_candidates
                                            .iter_mut()
                                            .find(|c| c.candidate_id == candidate_id)
                                    {
                                        cand.phase = "producing".into();
                                        cand.producer_task_id = Some(tid.clone());
                                    }
                                    let _ = st
                                        .delegation_store
                                        .bind_graph_step_task(&tid, &graph_id, &step_id, suppress);
                                }
                                DaemonToWrapper::NativeDelegationResult { result } => {
                                    let code = result["code"]
                                        .as_str()
                                        .or_else(|| result["error"].as_str())
                                        .unwrap_or("candidate_produce_refused");
                                    let msg = result["message"].as_str().unwrap_or(code);
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                        && let Some(cand) = cs
                                            .escalation_candidates
                                            .iter_mut()
                                            .find(|c| c.candidate_id == candidate_id)
                                    {
                                        cand.phase = "failed".into();
                                        cand.reason = Some(
                                            format!("{code}: {msg}").chars().take(400).collect(),
                                        );
                                    }
                                    finalize_candidates_if_all_failed(&mut current.steps[pos], now);
                                }
                                other => {
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                        && let Some(cand) = cs
                                            .escalation_candidates
                                            .iter_mut()
                                            .find(|c| c.candidate_id == candidate_id)
                                    {
                                        cand.phase = "failed".into();
                                        cand.reason = Some(
                                            format!("candidate_produce_unexpected:{other:?}")
                                                .chars()
                                                .take(256)
                                                .collect(),
                                        );
                                    }
                                    finalize_candidates_if_all_failed(&mut current.steps[pos], now);
                                }
                            }
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            persist_graph_with_pending_metrics(&st, &mut current, now);
                        } else if let DaemonToWrapper::NativeDelegationResult { result } = response
                            && result["status"] != "refused"
                            && let Some(orphan_tid) = result["task_id"].as_str().map(str::to_owned)
                        {
                            drop(st);
                            let _ = cancel_task_for_graph(&graph_snapshot, &orphan_tid, &state_bg);
                        }
                    });
            }
            GraphLaunchJob::EscalationCandidateGate {
                graph: graph_snapshot,
                idx,
                candidate_id,
                producer_task_id,
                expected_commit,
                gate,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let _ = thread::Builder::new()
                    .name(format!("bridget-esc-gate-{}-c{}", &step_id, candidate_id))
                    .spawn(move || {
                        let launch_key =
                            format!("{graph_id}:{step_id}:esc_cand:{candidate_id}:gate");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let gate_res =
                            super::native_delegation::run_deterministic_gate_for_task_pub(
                                &state_bg,
                                &producer_task_id,
                                &expected_commit,
                                &gate.argv,
                                gate.timeout_secs,
                            );
                        let mut st = state_bg.lock().recover_poison();
                        st.graph_launching_steps.remove(&launch_key);
                        if let Ok(Some(mut current)) = st.delegation_store.get_graph(&graph_id)
                            && let Some(pos) =
                                current.steps.iter().position(|s| s.step_id == step_id)
                            && current.steps[pos].state == "running"
                            && current.steps[pos]
                                .convergence_state
                                .as_ref()
                                .is_some_and(|cs| {
                                    cs.phase == "escalating_candidates"
                                        && cs.escalation_candidates.iter().any(|c| {
                                            c.candidate_id == candidate_id
                                                && c.phase == "launching_gate"
                                        })
                                })
                        {
                            let now = unix_timestamp();
                            apply_escalation_candidate_gate(
                                &mut current.steps[pos],
                                candidate_id,
                                gate_res,
                                now,
                            );
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            persist_graph_with_pending_metrics(&st, &mut current, now);
                        }
                    });
            }
            GraphLaunchJob::EscalationCandidateVerify {
                graph: graph_snapshot,
                idx,
                candidate_id,
                delegation,
            } => {
                let graph_id = graph_snapshot.graph_id.clone();
                let step_id = graph_snapshot.steps[idx].step_id.clone();
                let suppress =
                    graph_snapshot.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
                let _ = thread::Builder::new()
                    .name(format!("bridget-esc-ver-{}-c{}", &step_id, candidate_id))
                    .spawn(move || {
                        let launch_key =
                            format!("{graph_id}:{step_id}:esc_cand:{candidate_id}:verify");
                        let _key_guard = LaunchKeyGuard {
                            state: Arc::clone(&state_bg),
                            key: launch_key.clone(),
                        };
                        let exec_tag = format!("{step_id}-esc-c{candidate_id}-ver");
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
                                .is_some_and(|cs| {
                                    cs.phase == "escalating_candidates"
                                        && cs.escalation_candidates.iter().any(|c| {
                                            c.candidate_id == candidate_id
                                                && c.phase == "launching_verify"
                                        })
                                })
                        {
                            let now = unix_timestamp();
                            match response {
                                DaemonToWrapper::NativeDelegationResult { result }
                                    if result["status"] != "refused"
                                        && result["task_id"].as_str().is_some() =>
                                {
                                    let tid = result["task_id"].as_str().unwrap().to_string();
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                        && let Some(cand) = cs
                                            .escalation_candidates
                                            .iter_mut()
                                            .find(|c| c.candidate_id == candidate_id)
                                    {
                                        cand.phase = "verifying".into();
                                        cand.verifier_task_id = Some(tid.clone());
                                    }
                                    let _ = st
                                        .delegation_store
                                        .bind_graph_step_task(&tid, &graph_id, &step_id, suppress);
                                }
                                DaemonToWrapper::NativeDelegationResult { result } => {
                                    let code = result["code"]
                                        .as_str()
                                        .or_else(|| result["error"].as_str())
                                        .unwrap_or("candidate_verify_refused");
                                    let msg = result["message"].as_str().unwrap_or(code);
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                        && let Some(cand) = cs
                                            .escalation_candidates
                                            .iter_mut()
                                            .find(|c| c.candidate_id == candidate_id)
                                    {
                                        cand.phase = "failed".into();
                                        cand.reason = Some(
                                            format!("{code}: {msg}").chars().take(400).collect(),
                                        );
                                    }
                                    finalize_candidates_if_all_failed(&mut current.steps[pos], now);
                                }
                                other => {
                                    if let Some(cs) = current.steps[pos].convergence_state.as_mut()
                                        && let Some(cand) = cs
                                            .escalation_candidates
                                            .iter_mut()
                                            .find(|c| c.candidate_id == candidate_id)
                                    {
                                        cand.phase = "failed".into();
                                        cand.reason = Some(
                                            format!("candidate_verify_unexpected:{other:?}")
                                                .chars()
                                                .take(256)
                                                .collect(),
                                        );
                                    }
                                    finalize_candidates_if_all_failed(&mut current.steps[pos], now);
                                }
                            }
                            let _ = propagate_graph_state(&mut current, now);
                            current.updated_at = now;
                            persist_graph_with_pending_metrics(&st, &mut current, now);
                        } else if let DaemonToWrapper::NativeDelegationResult { result } = response
                            && result["status"] != "refused"
                            && let Some(orphan_tid) = result["task_id"].as_str().map(str::to_owned)
                        {
                            drop(st);
                            let _ = cancel_task_for_graph(&graph_snapshot, &orphan_tid, &state_bg);
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
            when: GraphStepCondition::AllSucceeded,
            when_outcomes: Vec::new(),
            model_tier: None,
            blackboard_reads: Vec::new(),
            blackboard_writes: Vec::new(),
            delegation: sample_auto(task),
            convergence: None,
            map: None,
        }
    }

    #[test]
    fn dag_diamond_topological_order_and_parallel_unlock() {
        let req = DelegateGraphRequest {
            request_id: "dag-1".into(),
            title: Some("Build puis tests parallèles puis synthèse".into()),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
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
            bridget_build_id: String::new(),
            on_failure: req.on_failure,
            notify_policy: req.notify_policy,
            budget: None,
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
            state: "running".into(),
            topological_order: order,
            steps: req
                .steps
                .into_iter()
                .map(|s| NativeGraphStepRecord {
                    step_id: s.step_id,
                    depends_on: s.depends_on,
                    pass_dependency_results: s.pass_dependency_results,
                    when: s.when,
                    when_outcomes: s.when_outcomes,
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: s.delegation,
                    convergence: s.convergence,
                    convergence_state: None,
                    map: s.map,
                    map_state: None,
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
        let (enriched, _) = build_enriched_step_task(b, &graph.steps, &graph.blackboard);
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
            budget: None,
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
            budget: None,
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
            bridget_build_id: String::new(),
            on_failure: policy,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
            state: "running".into(),
            topological_order: vec!["A".into(), "B".into(), "C".into(), "X".into()],
            steps: vec![
                NativeGraphStepRecord {
                    step_id: "A".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("A"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("B"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("C"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("X"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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
            bridget_build_id: String::new(),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
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
            gate: None,
            escalation: None,
            different_provider: false,
        });
        let req = DelegateGraphRequest {
            request_id: "conv-bad-min".into(),
            title: None,
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
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
        let derived = convergence_step_request_id(&long_req, &long_step, 0, 3, "verify");
        assert!(derived.len() <= 128, "len={}", derived.len());
        let derived_t1 = convergence_step_request_id(&long_req, &long_step, 1, 3, "verify");
        assert!(derived_t1.len() <= 128, "len={}", derived_t1.len());
        assert_ne!(derived, derived_t1);
    }

    fn mk_test_iter(
        iteration: u8,
        tier: u8,
        producer_task_id: &str,
        producer_round: u32,
        producer_commit: Option<String>,
        verifier_task_id: Option<&str>,
    ) -> NativeGraphConvergenceIteration {
        NativeGraphConvergenceIteration {
            iteration,
            tier,
            producer_task_id: Some(producer_task_id.into()),
            producer_round,
            producer_commit,
            producer_engine: None,
            producer_model: None,
            gate_passed: None,
            gate_exit_code: None,
            gate_timed_out: false,
            gate_duration_ms: None,
            gate_output_excerpt: None,
            verifier_task_id: verifier_task_id.map(str::to_string),
            verifier_engine: None,
            verifier_model: None,
            verifier_orthogonal: None,
            verdict: None,
            reason: None,
            blocking_points: Vec::new(),
            blocking_fingerprint: None,
            started_at: Some(10),
            finished_at: None,
        }
    }

    #[test]
    fn convergence_state_machine_revise_approve_stagnation_escalate_and_max_rounds() {
        let mk_step = |max_iterations: u8, stop_on_stagnation: bool| NativeGraphStepRecord {
            step_id: "build-review".into(),
            depends_on: vec![],
            pass_dependency_results: true,
            when: GraphStepCondition::AllSucceeded,
            when_outcomes: Vec::new(),
            model_tier: None,
            blackboard_reads: Vec::new(),
            blackboard_writes: Vec::new(),
            blackboard_reads_injected: 0,
            blackboard_writes_applied: 0,
            metrics_recorded: false,
            delegation: sample_auto("impl"),
            convergence: Some(GraphConvergenceLoop {
                verify: sample_auto("review"),
                max_iterations,
                stop_on_stagnation,
                gate: None,
                escalation: None,
                different_provider: false,
            }),
            convergence_state: Some(NativeGraphConvergenceState {
                current_iteration: 1,
                max_iterations,
                total_max_iterations: max_iterations,
                stop_on_stagnation,
                phase: "verifying".into(),
                escalation_tier: 0,
                tier_escalated: false,
                escalated_from_reason: None,
                producer_task_id: Some("prod-1".into()),
                prior_producer_task_ids: Vec::new(),
                producer_expected_round: 1,
                verifier_task_id: Some("ver-1".into()),
                last_producer_commit: Some("a".repeat(40)),
                last_blocking_fingerprint: None,
                seen_commits: vec!["a".repeat(40)],
                seen_blocking_fingerprints: Vec::new(),
                gate_runs: 0,
                gate_failures: 0,
                gate_saved_verifier_calls: 0,
                stagnation_saved_iterations: 0,
                metrics_recorded: false,
                outcome: None,
                iterations: vec![mk_test_iter(
                    1,
                    0,
                    "prod-1",
                    1,
                    Some("a".repeat(40)),
                    Some("ver-1"),
                )],
                escalation_candidates: Vec::new(),
                winning_candidate_id: None,
                candidates_cancelled: 0,
            }),
            map: None,
            map_state: None,
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
            .push(mk_test_iter(
                2,
                0,
                "prod-1",
                2,
                Some("b".repeat(40)),
                Some("ver-2"),
            ));
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
            .push(mk_test_iter(
                2,
                0,
                "prod-1",
                2,
                Some("c".repeat(40)),
                Some("ver-2"),
            ));
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
            bridget_build_id: String::new(),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
            state: "running".into(),
            topological_order: vec!["A".into(), "B".into(), "D".into()],
            steps: vec![
                NativeGraphStepRecord {
                    step_id: "A".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("prep"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("deploy"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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

    #[test]
    fn spec314_gate_escalation_anti_oscillation_and_fallback_branches() {
        use crate::deterministic_gate::GateExecOutput;

        let mk_conv_step = |gate: Option<GraphDeterministicGate>,
                            escalation: Option<GraphConvergenceEscalation>,
                            max_iterations: u8| NativeGraphStepRecord {
            step_id: "impl".into(),
            depends_on: vec![],
            pass_dependency_results: true,
            when: GraphStepCondition::AllSucceeded,
            when_outcomes: Vec::new(),
            model_tier: None,
            blackboard_reads: Vec::new(),
            blackboard_writes: Vec::new(),
            blackboard_reads_injected: 0,
            blackboard_writes_applied: 0,
            metrics_recorded: false,
            delegation: sample_auto("impl"),
            convergence: Some(GraphConvergenceLoop {
                verify: sample_auto("review"),
                max_iterations,
                stop_on_stagnation: true,
                gate,
                escalation,
                different_provider: true,
            }),
            convergence_state: Some(NativeGraphConvergenceState {
                current_iteration: 1,
                max_iterations,
                total_max_iterations: max_iterations,
                stop_on_stagnation: true,
                phase: "producing".into(),
                escalation_tier: 0,
                tier_escalated: false,
                escalated_from_reason: None,
                producer_task_id: Some("prod-t0".into()),
                prior_producer_task_ids: Vec::new(),
                producer_expected_round: 1,
                verifier_task_id: None,
                last_producer_commit: None,
                last_blocking_fingerprint: None,
                seen_commits: Vec::new(),
                seen_blocking_fingerprints: Vec::new(),
                gate_runs: 0,
                gate_failures: 0,
                gate_saved_verifier_calls: 0,
                stagnation_saved_iterations: 0,
                metrics_recorded: false,
                outcome: None,
                iterations: vec![mk_test_iter(1, 0, "prod-t0", 1, None, None)],
                escalation_candidates: Vec::new(),
                winning_candidate_id: None,
                candidates_cancelled: 0,
            }),
            map: None,
            map_state: None,
            state: "running".into(),
            task_id: Some("prod-t0".into()),
            error: None,
            result_excerpt: None,
            factual_summary: None,
            started_at: Some(10),
            finished_at: None,
        };

        // 1) Porte déterministe : échec tour 1 (0 appel vérificateur) -> reprise -> réussite tour 2 -> pending_verify.
        let gate_cfg = GraphDeterministicGate {
            argv: vec!["cargo".into(), "test".into()],
            timeout_secs: 60,
        };
        let mut s = mk_conv_step(Some(gate_cfg.clone()), None, 3);
        apply_producer_iteration_result(
            &mut s,
            1,
            Some("1".repeat(40)),
            Some("v1".into()),
            json!({"commit": "1".repeat(40)}),
            20,
        );
        assert_eq!(s.convergence_state.as_ref().unwrap().phase, "pending_gate");
        apply_gate_result(
            &mut s,
            GateExecOutput {
                passed: false,
                exit_code: Some(101),
                timed_out: false,
                duration_ms: 420,
                output_excerpt: "test foo FAILED in 0.42s".into(),
                unavailable_reason: None,
            },
            21,
        );
        let cs = s.convergence_state.as_ref().unwrap();
        assert_eq!(cs.phase, "pending_produce");
        assert_eq!(cs.current_iteration, 2);
        assert_eq!(cs.gate_runs, 1);
        assert_eq!(cs.gate_failures, 1);
        assert_eq!(cs.gate_saved_verifier_calls, 1);
        // Tour 2 : nouveau commit -> porte verte -> pending_verify.
        s.convergence_state.as_mut().unwrap().phase = "producing".into();
        s.convergence_state
            .as_mut()
            .unwrap()
            .iterations
            .push(mk_test_iter(2, 0, "prod-t0", 2, None, None));
        apply_producer_iteration_result(
            &mut s,
            2,
            Some("2".repeat(40)),
            Some("v2".into()),
            json!({"commit": "2".repeat(40)}),
            35,
        );
        assert_eq!(s.convergence_state.as_ref().unwrap().phase, "pending_gate");
        apply_gate_result(
            &mut s,
            GateExecOutput {
                passed: true,
                exit_code: Some(0),
                timed_out: false,
                duration_ms: 310,
                output_excerpt: "ok".into(),
                unavailable_reason: None,
            },
            36,
        );
        assert_eq!(
            s.convergence_state.as_ref().unwrap().phase,
            "pending_verify"
        );
        assert_eq!(s.convergence_state.as_ref().unwrap().gate_runs, 2);
        assert_eq!(s.convergence_state.as_ref().unwrap().gate_failures, 1);

        // 2) Porte indisponible (fail-closed) -> outcome = gate_unavailable.
        let mut s_unavail = mk_conv_step(Some(gate_cfg.clone()), None, 3);
        apply_producer_iteration_result(&mut s_unavail, 1, None, None, json!({}), 20);
        assert_eq!(s_unavail.state, "failed");
        assert_eq!(
            s_unavail
                .convergence_state
                .as_ref()
                .unwrap()
                .outcome
                .as_deref(),
            Some("gate_unavailable")
        );

        // 3) Anti-oscillation A -> B -> A + Escalade palier 0 -> palier 1 + borne dure.
        let esc_cfg = GraphConvergenceEscalation {
            engine: Some("claude".into()),
            model: Some("claude-opus-4-6".into()),
            effort: Some("high".into()),
            machine_id: None,
            extra_iterations: 2,
            on_stagnation: true,
            on_max_iterations: true,
            candidates: 1,
        };
        let mut osc = mk_conv_step(None, Some(esc_cfg.clone()), 4);
        // Tour 1 : commit A -> revise fp1
        apply_producer_iteration_result(&mut osc, 1, Some("a".repeat(40)), None, json!({}), 15);
        osc.convergence_state.as_mut().unwrap().phase = "verifying".into();
        apply_verifier_verdict(
            &mut osc,
            parse_convergence_verdict(
                "{\"verdict\":\"revise\",\"reason\":\"r1\",\"blocking_points\":[\"p1\"]}",
            ),
            20,
        );
        // Tour 2 : commit B -> revise fp2
        osc.convergence_state.as_mut().unwrap().phase = "producing".into();
        apply_producer_iteration_result(&mut osc, 2, Some("b".repeat(40)), None, json!({}), 25);
        osc.convergence_state.as_mut().unwrap().phase = "verifying".into();
        apply_verifier_verdict(
            &mut osc,
            parse_convergence_verdict(
                "{\"verdict\":\"revise\",\"reason\":\"r2\",\"blocking_points\":[\"p2\"]}",
            ),
            30,
        );
        // Tour 3 : retour au commit A (oscillation A->B->A) -> déclenche escalade palier 1 !
        osc.convergence_state.as_mut().unwrap().phase = "producing".into();
        apply_producer_iteration_result(&mut osc, 3, Some("a".repeat(40)), None, json!({}), 35);
        assert_eq!(osc.state, "running");
        let cs_osc = osc.convergence_state.as_ref().unwrap();
        assert_eq!(cs_osc.escalation_tier, 1);
        assert!(cs_osc.tier_escalated);
        assert_eq!(cs_osc.max_iterations, 6);
        assert_eq!(cs_osc.current_iteration, 4);
        assert!(cs_osc.producer_task_id.is_none());
        assert_eq!(cs_osc.prior_producer_task_ids, vec!["prod-t0".to_string()]);
        assert!(cs_osc.seen_commits.is_empty());
        assert_eq!(cs_osc.phase, "pending_produce");

        // 4) Verdict verifier == "escalate" court-circuite TOUJOURS l'escalade de modèle !
        let mut v_esc = mk_conv_step(None, Some(esc_cfg), 3);
        v_esc.convergence_state.as_mut().unwrap().phase = "verifying".into();
        apply_verifier_verdict(
            &mut v_esc,
            ParsedVerdict::Escalate {
                reason: "décision métier".into(),
            },
            40,
        );
        assert_eq!(v_esc.state, "failed");
        assert!(!v_esc.convergence_state.as_ref().unwrap().tier_escalated);
        assert_eq!(
            v_esc.convergence_state.as_ref().unwrap().outcome.as_deref(),
            Some("escalated")
        );

        // 5) Validation DAG : cancel_graph + fallback interdit ; cancelled interdit dans when_outcomes.
        let mut fb_step = sample_step("B_repli", &["A"], "plan B");
        fb_step.when = GraphStepCondition::OnOutcome;
        fb_step.when_outcomes = vec!["stagnated".into()];
        let bad_cancel_req = DelegateGraphRequest {
            request_id: "r-fb-cancel".into(),
            title: None,
            on_failure: GraphFailurePolicy::CancelGraph,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            steps: vec![sample_step("A", &[], "plan A"), fb_step.clone()],
        };
        assert_eq!(
            validate_and_order_graph(&bad_cancel_req).unwrap_err().0,
            "graph_invalid"
        );
        let mut bad_cancelled = fb_step.clone();
        bad_cancelled.when_outcomes = vec!["cancelled".into()];
        let bad_cancelled_req = DelegateGraphRequest {
            request_id: "r-fb-cancelled".into(),
            title: None,
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            steps: vec![sample_step("A", &[], "plan A"), bad_cancelled],
        };
        assert_eq!(
            validate_and_order_graph(&bad_cancelled_req).unwrap_err().0,
            "graph_invalid"
        );

        // 6) Exécution conditionnelle : nominal (A réussit) -> B_repli et son aval C sautés -> graphe completed !
        let mk_fb_graph = |a_state: &str, a_outcome: Option<&str>| NativeGraphRecord {
            graph_id: "g-fb".into(),
            request_id: "r-fb".into(),
            owner: "o".into(),
            owner_instance: "i".into(),
            parent_thread: None,
            communication_project: Some(bridget_transport::protocol::CommunicationProject {
                host: "local".into(),
                root: "bridget".into(),
            }),
            attested_permissions: None,
            title: None,
            bridget_build_id: String::new(),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
            state: "running".into(),
            topological_order: vec!["A".into(), "B_repli".into(), "C_post_repli".into()],
            steps: vec![
                NativeGraphStepRecord {
                    step_id: "A".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("plan A"),
                    convergence: None,
                    convergence_state: a_outcome.map(|o| NativeGraphConvergenceState {
                        current_iteration: 2,
                        max_iterations: 2,
                        total_max_iterations: 2,
                        stop_on_stagnation: true,
                        phase: "done".into(),
                        escalation_tier: 0,
                        tier_escalated: false,
                        escalated_from_reason: None,
                        producer_task_id: None,
                        prior_producer_task_ids: Vec::new(),
                        producer_expected_round: 1,
                        verifier_task_id: None,
                        last_producer_commit: None,
                        last_blocking_fingerprint: None,
                        seen_commits: Vec::new(),
                        seen_blocking_fingerprints: Vec::new(),
                        gate_runs: 0,
                        gate_failures: 0,
                        gate_saved_verifier_calls: 0,
                        stagnation_saved_iterations: 0,
                        metrics_recorded: false,
                        outcome: Some(o.into()),
                        iterations: Vec::new(),
                        escalation_candidates: Vec::new(),
                        winning_candidate_id: None,
                        candidates_cancelled: 0,
                    }),
                    map: None,
                    map_state: None,
                    state: a_state.into(),
                    task_id: Some("t-a".into()),
                    error: (a_state == "failed").then(|| "stagnation".into()),
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: Some(1),
                    finished_at: Some(5),
                },
                NativeGraphStepRecord {
                    step_id: "B_repli".into(),
                    depends_on: vec!["A".into()],
                    pass_dependency_results: true,
                    when: GraphStepCondition::OnOutcome,
                    when_outcomes: vec!["stagnated".into()],
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("plan B"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                },
                NativeGraphStepRecord {
                    step_id: "C_post_repli".into(),
                    depends_on: vec!["B_repli".into()],
                    pass_dependency_results: true,
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("post repli"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
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
            updated_at: 5,
        };

        let mut g_nom = mk_fb_graph("completed", None);
        propagate_graph_state(&mut g_nom, 10);
        assert_eq!(g_nom.steps[1].state, "skipped_condition_not_met");
        assert_eq!(g_nom.steps[2].state, "skipped_condition_not_met");
        assert_eq!(g_nom.state, "completed");

        // A échoue sur stagnated -> B_repli devient prêt ! Une fois B_repli + C_post_repli terminés,
        // le graphe termine en partial_failure avec fallback_recovered=true.
        let mut g_fb = mk_fb_graph("failed", Some("stagnated"));
        propagate_graph_state(&mut g_fb, 10);
        assert_eq!(ready_waiting_steps(&g_fb), vec!["B_repli"]);
        let (enriched, _) = build_enriched_step_task(&g_fb.steps[1], &g_fb.steps, &g_fb.blackboard);
        assert!(enriched.contains("outcome=stagnated"));
        assert!(enriched.contains("Erreur : stagnation"));
        g_fb.steps[1].state = "completed".into();
        g_fb.steps[2].state = "completed".into();
        propagate_graph_state(&mut g_fb, 20);
        assert_eq!(g_fb.state, "partial_failure");
        let view = graph_view(&g_fb);
        assert_eq!(view["fallback_recovered"], true);
        assert_eq!(view["counts"]["fallback_steps_completed"], 1);

        // Annulation humaine de A ne déclenche JAMAIS B_repli (even with AnyFailed).
        let mut g_cancel = mk_fb_graph("cancelled", None);
        g_cancel.steps[1].when = GraphStepCondition::AnyFailed;
        g_cancel.steps[1].when_outcomes.clear();
        propagate_graph_state(&mut g_cancel, 10);
        assert!(ready_waiting_steps(&g_cancel).is_empty());
        assert_eq!(g_cancel.steps[1].state, "skipped_condition_not_met");
        assert_eq!(g_cancel.steps[2].state, "skipped_condition_not_met");
    }

    #[test]
    fn spec314_worktree_auto_pin_and_nested_git_head_commit_extraction() {
        let sha_base = "4ed3cbee959cb27c480c3219a173e184eb721ec9";
        let sha_prod = "84e7d95990efd5ca9c0a732e7f6424d2c2c6f09b";

        // 1) Extraction du commit producteur depuis la vraie structure `factual_summary.git.head_commit`.
        let real_fs = json!({
            "git": {
                "strategy": "worktree",
                "base_commit": sha_base,
                "head_commit": sha_prod,
                "commit_count": 1
            }
        });
        assert_eq!(
            extract_commit_from_factual_summary(&real_fs),
            Some(sha_prod)
        );

        // 2) Auto-épinglage de `workspace.base_ref` quand `worktree` est déclaré sans `base_ref`
        //    (évite `expected_base_commit_mismatch` quand `main` avance pendant le graphe).
        let mut verify_del = sample_auto("review");
        verify_del.expected_base_commit = Some(sha_base.into());
        verify_del.workspace = Some(WorkspaceRequest::Worktree {
            base_ref: None,
            branch: None,
        });
        apply_commit_to_delegation(&mut verify_del, None);
        assert_eq!(
            verify_del.workspace,
            Some(WorkspaceRequest::Worktree {
                base_ref: Some(sha_base.into()),
                branch: None,
            })
        );

        // 3) Quand le producteur fournit `sha_prod`, verify est ré-ancré sur `sha_prod` (SHA + base_ref).
        apply_commit_to_delegation(&mut verify_del, Some(sha_prod));
        assert_eq!(verify_del.expected_base_commit.as_deref(), Some(sha_prod));
        assert_eq!(
            verify_del.workspace,
            Some(WorkspaceRequest::Worktree {
                base_ref: Some(sha_prod.into()),
                branch: None,
            })
        );
    }

    #[test]
    fn spec316_prefixe_stable_budgets_durs_blackboard_et_palier_cout() {
        // 1) Prompt caching : `build_verifier_task` place le bloc invariant strictement en tête.
        let mut step_v = NativeGraphStepRecord {
            step_id: "impl".into(),
            depends_on: vec![],
            pass_dependency_results: true,
            when: GraphStepCondition::AllSucceeded,
            when_outcomes: Vec::new(),
            model_tier: Some(GraphModelTier::Standard),
            blackboard_reads: Vec::new(),
            blackboard_writes: vec!["api_contract".into()],
            blackboard_reads_injected: 0,
            blackboard_writes_applied: 0,
            metrics_recorded: false,
            delegation: sample_auto("Implémenter l'endpoint /v2"),
            convergence: None,
            convergence_state: None,
            map: None,
            map_state: None,
            state: "running".into(),
            task_id: Some("t1".into()),
            error: None,
            result_excerpt: Some("extrait iter 1".into()),
            factual_summary: None,
            started_at: Some(10),
            finished_at: None,
        };
        let verify = sample_auto("Vérifier sécurité et tests unitaires");
        let p1 = build_verifier_task(&step_v, &verify, 1, 3);
        step_v.result_excerpt = Some("extrait iter 2 complètement différent".into());
        let p2 = build_verifier_task(&step_v, &verify, 2, 3);
        let marker = "\n## Contexte de l'itération courante";
        let head1 = p1.split(marker).next().unwrap();
        let head2 = p2.split(marker).next().unwrap();
        assert_eq!(
            head1, head2,
            "le préfixe invariant doit être bit-à-bit identique entre les itérations"
        );
        assert!(head1.starts_with("## Contrat de verdict fermé"));
        assert!(head1.contains("Vérifier sécurité et tests unitaires"));

        // `build_producer_revision_task` en nouvelle session conserve la consigne initiale en tête.
        let rev_new = build_producer_revision_task(
            "impl",
            1,
            2,
            4,
            "test cassé",
            &["fixer assert".into()],
            "Consigne initiale fixe",
            true,
            false,
        );
        assert!(rev_new.starts_with("## Consigne de cette étape\nConsigne initiale fixe"));

        // 2) Budgets durs : plafond frontière épuisé autorise encore un repli `light` !
        let mut g = NativeGraphRecord {
            graph_id: "g-budget".into(),
            owner: "o".into(),
            owner_instance: "i".into(),
            request_id: "r-budget".into(),
            title: None,
            bridget_build_id: "build316".into(),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: Some(GraphBudget {
                max_total_rounds: Some(3),
                max_frontier_rounds: Some(1),
                max_duration_secs: Some(600),
            }),
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
            state: "running".into(),
            topological_order: vec!["A".into(), "B_light".into()],
            steps: vec![
                step_v.clone(),
                NativeGraphStepRecord {
                    step_id: "B_light".into(),
                    depends_on: vec!["impl".into()],
                    pass_dependency_results: false,
                    when: GraphStepCondition::OnOutcome,
                    when_outcomes: vec!["budget_exhausted".into()],
                    model_tier: Some(GraphModelTier::Light),
                    blackboard_reads: vec!["api_contract".into()],
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("Plan B léger"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                },
            ],
            attested_permissions: None,
            parent_thread: None,
            communication_project: None,
            notice_sent: false,
            canonical: vec![1],
            created_at: 1_000,
            updated_at: 1_000,
        };
        assert!(check_graph_budget(&g, true, 1_010).is_ok());
        consume_graph_budget(&mut g, true);
        // 2e appel frontière refusé (`max_frontier_rounds = 1`)...
        assert_eq!(
            check_graph_budget(&g, true, 1_020),
            Err("max_frontier_rounds")
        );
        // ...mais un appel non-frontière (`light`) reste autorisé !
        assert!(check_graph_budget(&g, false, 1_020).is_ok());
        // Durée dépassée refuse tout.
        assert_eq!(
            check_graph_budget(&g, false, 1_600),
            Err("max_duration_secs")
        );
        // Quand l'étape 0 échoue sur budget_exhausted, B_light devient prête.
        fail_step_budget_exhausted(&mut g, 0, "max_frontier_rounds", 1_025);
        propagate_graph_state(&mut g, 1_025);
        assert_eq!(ready_waiting_steps(&g), vec!["B_light"]);

        // 3) Tableau noir structuré : seules les clés autorisées passent et sont injectées en pied.
        let output_text =
            "Travail fini.\n{\"blackboard\":{\"api_contract\":{\"v\":2},\"unauthorized\":true}}";
        let applied = apply_blackboard_updates(&mut g, 0, output_text, 1_030);
        assert_eq!(applied, 1);
        assert!(g.blackboard.contains_key("api_contract"));
        assert!(!g.blackboard.contains_key("unauthorized"));
        let (prompt_b, reads_count) =
            build_enriched_step_task(&g.steps[1], &g.steps, &g.blackboard);
        assert_eq!(reads_count, 1);
        assert!(prompt_b.starts_with("## Consigne de cette étape\nPlan B léger"));
        assert!(prompt_b.contains("## Tableau noir du graphe"));
        assert!(prompt_b.contains("`api_contract`"));
        assert!(!prompt_b.contains("Résultats des étapes préalables"));

        // Liste `blackboard_writes` vide sur B_light : aucune écriture acceptée.
        let denied =
            apply_blackboard_updates(&mut g, 1, "{\"blackboard\":{\"sneaky\":true}}", 1_035);
        assert_eq!(denied, 0);
        assert!(!g.blackboard.contains_key("sneaky"));

        // 4) Classification par jetons (évite le faux positif `mini` dans `gemini`).
        assert_eq!(
            super::native_delegation::classify_catalog_model_tier("gemini-2.5-pro"),
            GraphModelTier::Frontier
        );
        assert!(super::native_delegation::is_frontier_model_name(
            "gemini-2.5-pro"
        ));
        assert_eq!(
            super::native_delegation::classify_catalog_model_tier("gemini-2.5-flash"),
            GraphModelTier::Light
        );
        assert_eq!(
            super::native_delegation::classify_catalog_model_tier("gpt-5-mini"),
            GraphModelTier::Light
        );
        assert_eq!(
            super::native_delegation::classify_catalog_model_tier("claude-opus-4-6"),
            GraphModelTier::Frontier
        );
        assert_eq!(
            super::native_delegation::classify_catalog_model_tier("claude-sonnet-4-6"),
            GraphModelTier::Standard
        );
    }

    #[test]
    fn spec317_map_reduce_workers_chauds_et_best_of_n_escalade() {
        use crate::deterministic_gate::GateExecOutput;

        // 1) Validation : map + convergence mutuellement exclusifs.
        let mut both = sample_step("M", &[], "Traiter le fichier");
        both.map = Some(GraphMapSpec {
            items: vec!["a.rs".into(), "b.rs".into()],
            max_concurrency: 2,
            reuse_workers: true,
            fail_fast: false,
        });
        both.convergence = Some(GraphConvergenceLoop {
            verify: sample_auto("Vérifier"),
            max_iterations: 2,
            stop_on_stagnation: true,
            gate: None,
            escalation: None,
            different_provider: false,
        });
        let req_both = DelegateGraphRequest {
            request_id: "map-conv-conflict".into(),
            title: None,
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            steps: vec![both],
        };
        assert_eq!(
            validate_and_order_graph(&req_both).unwrap_err().0,
            "graph_invalid"
        );

        // 2) Validation : map.items borné (min 2, max 16) et max_concurrency (1..=4).
        let mut too_few = sample_step("M", &[], "Analyser");
        too_few.map = Some(GraphMapSpec {
            items: vec!["seul.rs".into()],
            max_concurrency: 2,
            reuse_workers: true,
            fail_fast: false,
        });
        assert_eq!(
            validate_and_order_graph(&DelegateGraphRequest {
                request_id: "map-min".into(),
                title: None,
                on_failure: GraphFailurePolicy::StopDependents,
                notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
                budget: None,
                steps: vec![too_few],
            })
            .unwrap_err()
            .0,
            "graph_invalid"
        );

        // 3) Validation Best-of-N : candidates=3 refusé sans gate, autorisé avec gate ; candidates=2 sans gate OK.
        let esc3 = GraphConvergenceEscalation {
            engine: None,
            model: None,
            effort: None,
            machine_id: None,
            extra_iterations: 1,
            on_stagnation: true,
            on_max_iterations: true,
            candidates: 3,
        };
        assert_eq!(
            validate_escalation_spec("S", 2, true, false, &esc3)
                .unwrap_err()
                .0,
            "graph_invalid"
        );
        assert!(validate_escalation_spec("S", 2, true, true, &esc3).is_ok());
        let mut esc2 = esc3.clone();
        esc2.candidates = 2;
        assert!(validate_escalation_spec("S", 2, true, false, &esc2).is_ok());

        // 4) Map-Reduce : préfixe stable en tête pour session neuve, delta court sur Continue,
        //    réutilisation des slots chauds, agrégation pour reduce, et fail_fast.
        let p_new = build_map_item_task("Consigne stable", "mod_a.rs", 0, 3, false);
        let p_new2 = build_map_item_task("Consigne stable", "mod_b.rs", 1, 3, false);
        let prefix_marker = "\n\n## Élément assigné";
        assert_eq!(
            p_new.split(prefix_marker).next().unwrap(),
            p_new2.split(prefix_marker).next().unwrap()
        );
        let p_cont = build_map_item_task("Consigne stable", "mod_c.rs", 2, 3, true);
        assert!(p_cont.starts_with("## Élément suivant du lot Map-Reduce (3/3)"));

        let map_spec = GraphMapSpec {
            items: vec!["a.rs".into(), "b.rs".into(), "c.rs".into()],
            max_concurrency: 2,
            reuse_workers: true,
            fail_fast: false,
        };
        let mut map_graph = NativeGraphRecord {
            graph_id: "g-map".into(),
            owner: "o".into(),
            owner_instance: "i".into(),
            request_id: "r-map".into(),
            title: None,
            bridget_build_id: "build317".into(),
            on_failure: GraphFailurePolicy::StopDependents,
            notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
            budget: None,
            budget_usage: NativeGraphBudgetUsage::default(),
            blackboard: BTreeMap::new(),
            state: "running".into(),
            topological_order: vec!["map_step".into(), "reduce_step".into()],
            steps: vec![
                NativeGraphStepRecord {
                    step_id: "map_step".into(),
                    depends_on: vec![],
                    pass_dependency_results: true,
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: Some(GraphModelTier::Light),
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("Auditer chaque module"),
                    convergence: None,
                    convergence_state: None,
                    map: Some(map_spec.clone()),
                    map_state: Some(init_map_state(&map_spec)),
                    state: "running".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: Some(10),
                    finished_at: None,
                },
                NativeGraphStepRecord {
                    step_id: "reduce_step".into(),
                    depends_on: vec!["map_step".into()],
                    pass_dependency_results: true,
                    when: GraphStepCondition::AllSucceeded,
                    when_outcomes: Vec::new(),
                    model_tier: None,
                    blackboard_reads: Vec::new(),
                    blackboard_writes: Vec::new(),
                    blackboard_reads_injected: 0,
                    blackboard_writes_applied: 0,
                    metrics_recorded: false,
                    delegation: sample_auto("Synthétiser l'audit"),
                    convergence: None,
                    convergence_state: None,
                    map: None,
                    map_state: None,
                    state: "waiting_dependencies".into(),
                    task_id: None,
                    error: None,
                    result_excerpt: None,
                    factual_summary: None,
                    started_at: None,
                    finished_at: None,
                },
            ],
            attested_permissions: None,
            parent_thread: None,
            communication_project: None,
            notice_sent: false,
            canonical: vec![1],
            created_at: 10,
            updated_at: 10,
        };
        // Simuler items 0 et 1 sur slots 0 et 1 -> item 0 termine et garde le worker chaud sur slot 0.
        {
            let ms = map_graph.steps[0].map_state.as_mut().unwrap();
            ms.worker_slots[0].busy = true;
            ms.worker_slots[0].task_id = Some("worker-0".into());
            ms.items[0].state = "running".into();
            ms.items[0].worker_slot = Some(0);
            ms.items[0].task_id = Some("worker-0".into());
            ms.worker_slots[1].busy = true;
            ms.worker_slots[1].task_id = Some("worker-1".into());
            ms.items[1].state = "running".into();
            ms.items[1].worker_slot = Some(1);
            ms.items[1].task_id = Some("worker-1".into());
        }
        apply_map_item_completed(&mut map_graph, 0, 0, Some("a.rs OK"), true, 20);
        {
            let ms = map_graph.steps[0].map_state.as_ref().unwrap();
            assert!(!ms.worker_slots[0].busy);
            assert_eq!(ms.worker_slots[0].task_id.as_deref(), Some("worker-0"));
            assert_eq!(ms.worker_slots[0].rounds_completed, 1);
            assert_eq!(map_graph.steps[0].state, "running");
        }
        // Item 2 réutilise slot 0 (`worker-0`), puis items 1 et 2 terminent -> map_step completed -> reduce_step prêt !
        {
            let ms = map_graph.steps[0].map_state.as_mut().unwrap();
            ms.worker_slots[0].busy = true;
            ms.items[2].state = "running".into();
            ms.items[2].worker_slot = Some(0);
            ms.items[2].task_id = Some("worker-0".into());
            ms.items[2].reused_worker = true;
            ms.reused_worker_hits += 1;
        }
        apply_map_item_completed(&mut map_graph, 0, 1, Some("b.rs OK"), true, 25);
        apply_map_item_completed(&mut map_graph, 0, 2, Some("c.rs OK"), true, 30);
        assert_eq!(map_graph.steps[0].state, "completed");
        assert_eq!(ready_waiting_steps(&map_graph), vec!["reduce_step"]);
        let (reduce_prompt, _) =
            build_enriched_step_task(&map_graph.steps[1], &map_graph.steps, &map_graph.blackboard);
        assert!(reduce_prompt.contains("[1/3] a.rs : completed | a.rs OK"));
        assert!(reduce_prompt.contains("[3/3] c.rs : completed | c.rs OK"));

        // 5) Map fail_fast saute les items encore pending dès le premier échec.
        let mut ff_graph = map_graph.clone();
        ff_graph.steps[0].state = "running".into();
        let mut ff_spec = map_spec;
        ff_spec.fail_fast = true;
        ff_graph.steps[0].map_state = Some(init_map_state(&ff_spec));
        ff_graph.steps[0].map_state.as_mut().unwrap().items[0].state = "running".into();
        apply_map_item_failed(&mut ff_graph, 0, 0, false, Some("boom".into()), 40);
        let ff_ms = ff_graph.steps[0].map_state.as_ref().unwrap();
        assert_eq!(ff_ms.items[0].state, "failed");
        assert_eq!(ff_ms.items[1].state, "skipped");
        assert_eq!(ff_ms.items[2].state, "skipped");
        assert_eq!(ff_graph.steps[0].state, "failed");
        assert_eq!(
            ff_graph.steps[0].error.as_deref(),
            Some("map_items_failed:1/3 (skipped=2)")
        );

        // 5b) Validation P1-1 : workspace='shared' interdit pour Map max_concurrency>1 en écriture
        //     et pour Best-of-N candidates>1 hors discovery ; troncature P2-1 garde la ligne item intacte.
        let mut shared_map = sample_step("M_shared", &[], "Modifier");
        shared_map.delegation.workspace = Some(WorkspaceRequest::Shared {});
        shared_map.map = Some(GraphMapSpec {
            items: vec!["a.rs".into(), "b.rs".into()],
            max_concurrency: 2,
            reuse_workers: true,
            fail_fast: false,
        });
        assert_eq!(
            validate_and_order_graph(&DelegateGraphRequest {
                request_id: "map-shared-bad".into(),
                title: None,
                on_failure: GraphFailurePolicy::StopDependents,
                notify_policy: GraphNotifyPolicy::OnGraphDoneOrFailure,
                budget: None,
                steps: vec![shared_map],
            })
            .unwrap_err()
            .0,
            "graph_invalid"
        );
        let huge_base = "é".repeat(40_000);
        let clipped_prompt = build_map_item_task(&huge_base, "cible_critique.rs", 0, 2, false);
        assert!(clipped_prompt.len() <= MAX_TASK_BYTES);
        assert!(clipped_prompt.contains("cible_critique.rs"));

        // 6) Escalade Best-of-N (3 candidats + porte déterministe) :
        //    candidat 0 rejeté par la porte (0 appel vérificateur), candidat 1 approuvé ->
        //    gagne immédiatement et annule candidat 2 encore actif !
        let mut bn_step = NativeGraphStepRecord {
            step_id: "hard_fix".into(),
            depends_on: vec![],
            pass_dependency_results: true,
            when: GraphStepCondition::AllSucceeded,
            when_outcomes: Vec::new(),
            model_tier: None,
            blackboard_reads: Vec::new(),
            blackboard_writes: Vec::new(),
            blackboard_reads_injected: 0,
            blackboard_writes_applied: 0,
            metrics_recorded: false,
            delegation: sample_auto("Corriger le bug complexe"),
            convergence: Some(GraphConvergenceLoop {
                verify: sample_auto("Vérifier la correction"),
                max_iterations: 2,
                stop_on_stagnation: true,
                gate: Some(GraphDeterministicGate {
                    argv: vec!["cargo".into(), "test".into()],
                    timeout_secs: 60,
                }),
                escalation: Some(esc3),
                different_provider: true,
            }),
            convergence_state: Some(NativeGraphConvergenceState {
                current_iteration: 2,
                max_iterations: 2,
                total_max_iterations: 2,
                stop_on_stagnation: true,
                phase: "verifying".into(),
                escalation_tier: 0,
                tier_escalated: false,
                escalated_from_reason: None,
                producer_task_id: Some("prod-t0".into()),
                prior_producer_task_ids: Vec::new(),
                producer_expected_round: 2,
                verifier_task_id: Some("ver-t0".into()),
                last_producer_commit: Some("a".repeat(40)),
                last_blocking_fingerprint: None,
                seen_commits: vec!["a".repeat(40)],
                seen_blocking_fingerprints: Vec::new(),
                gate_runs: 0,
                gate_failures: 0,
                gate_saved_verifier_calls: 0,
                stagnation_saved_iterations: 0,
                metrics_recorded: false,
                outcome: None,
                iterations: vec![mk_test_iter(
                    2,
                    0,
                    "prod-t0",
                    2,
                    Some("a".repeat(40)),
                    Some("ver-t0"),
                )],
                escalation_candidates: Vec::new(),
                winning_candidate_id: None,
                candidates_cancelled: 0,
            }),
            map: None,
            map_state: None,
            state: "running".into(),
            task_id: Some("ver-t0".into()),
            error: None,
            result_excerpt: None,
            factual_summary: None,
            started_at: Some(10),
            finished_at: None,
        };
        // Plafond atteint au palier 0 -> déclenche escalade Best-of-N (3 candidats).
        apply_verifier_verdict(
            &mut bn_step,
            parse_convergence_verdict(
                "{\"verdict\":\"revise\",\"reason\":\"cas limite manquant\",\"blocking_points\":[\"edge\"]}",
            ),
            50,
        );
        let cs = bn_step.convergence_state.as_mut().unwrap();
        assert_eq!(cs.phase, "escalating_candidates");
        assert!(cs.tier_escalated);
        assert_eq!(cs.escalation_candidates.len(), 3);
        // Simuler les 3 producteurs lancés.
        cs.escalation_candidates[0].phase = "producing".into();
        cs.escalation_candidates[0].producer_task_id = Some("cand-0-prod".into());
        cs.escalation_candidates[1].phase = "producing".into();
        cs.escalation_candidates[1].producer_task_id = Some("cand-1-prod".into());
        cs.escalation_candidates[2].phase = "producing".into();
        cs.escalation_candidates[2].producer_task_id = Some("cand-2-prod".into());

        // Candidat 0 termine mais échoue à la porte déterministe -> rejeté sans vérificateur.
        apply_escalation_candidate_producer_result(
            &mut bn_step,
            0,
            Some("1".repeat(40)),
            Some("essai 0".into()),
            json!({"commit": "1".repeat(40)}),
            60,
        );
        assert_eq!(
            bn_step
                .convergence_state
                .as_ref()
                .unwrap()
                .escalation_candidates[0]
                .phase,
            "pending_gate"
        );
        apply_escalation_candidate_gate(
            &mut bn_step,
            0,
            GateExecOutput {
                passed: false,
                exit_code: Some(1),
                timed_out: false,
                duration_ms: 100,
                output_excerpt: "test failed".into(),
                unavailable_reason: None,
            },
            61,
        );
        assert_eq!(
            bn_step
                .convergence_state
                .as_ref()
                .unwrap()
                .escalation_candidates[0]
                .phase,
            "rejected"
        );
        assert_eq!(
            bn_step
                .convergence_state
                .as_ref()
                .unwrap()
                .gate_saved_verifier_calls,
            1
        );

        // Candidat 1 passe la porte et reçoit `approve` -> gagne immédiatement et annule candidat 2 !
        apply_escalation_candidate_producer_result(
            &mut bn_step,
            1,
            Some("2".repeat(40)),
            Some("correctif propre".into()),
            json!({"commit": "2".repeat(40)}),
            65,
        );
        apply_escalation_candidate_gate(
            &mut bn_step,
            1,
            GateExecOutput {
                passed: true,
                exit_code: Some(0),
                timed_out: false,
                duration_ms: 120,
                output_excerpt: "ok".into(),
                unavailable_reason: None,
            },
            66,
        );
        assert_eq!(
            bn_step
                .convergence_state
                .as_ref()
                .unwrap()
                .escalation_candidates[1]
                .phase,
            "pending_verify"
        );
        bn_step
            .convergence_state
            .as_mut()
            .unwrap()
            .escalation_candidates[1]
            .phase = "verifying".into();
        bn_step
            .convergence_state
            .as_mut()
            .unwrap()
            .escalation_candidates[1]
            .verifier_task_id = Some("cand-1-ver".into());
        let to_cancel = apply_escalation_candidate_verdict(
            &mut bn_step,
            1,
            ParsedVerdict::Approve {
                reason: "fix complet".into(),
            },
            70,
        );
        assert_eq!(to_cancel, vec!["cand-2-prod".to_string()]);
        assert_eq!(bn_step.state, "completed");
        assert_eq!(bn_step.task_id.as_deref(), Some("cand-1-prod"));
        let cs_done = bn_step.convergence_state.as_ref().unwrap();
        assert_eq!(cs_done.outcome.as_deref(), Some("approved"));
        assert_eq!(cs_done.winning_candidate_id, Some(1));
        assert_eq!(cs_done.candidates_cancelled, 1);
        assert_eq!(cs_done.escalation_candidates[2].phase, "cancelled");
    }
}
