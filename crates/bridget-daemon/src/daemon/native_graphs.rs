//! Orchestration autonome de graphes orientés acycliques (DAG) de missions Bridget.
//!
//! Permet à un chef d'orchestre de déclarer un enchaînement (`A -> B + C -> D`) en un seul
//! appel (`bridget_delegate_graph`) puis de s'endormir sans ronde de garde périodique :
//! le daemon suit l'état durable de chaque étape, déclenche automatiquement les étapes
//! suivantes en parallèle dès que leurs dépendances sont terminées, propage les échecs selon
//! `on_failure`, et réveille le parent une seule fois avec le bilan consolidé.

use super::*;
use crate::delegation::{NativeGraphRecord, NativeGraphStepRecord};
use bridget_transport::protocol::{
    AutoDelegateV2, DelegateGraphRequest, GraphFailurePolicy, GraphNotifyPolicy,
    NativeDelegationRequest, ProviderPermissions,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) const MAX_GRAPH_STEPS: usize = 16;
const MAX_STEP_ID_LEN: usize = 64;
const MAX_DEP_EXCERPT_CHARS: usize = 1200;
const MAX_TASK_BYTES: usize = 65_536;

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

/// Construit la consigne finale d'une étape dépendante en y préfixant un contexte court et
/// factuel des résultats de ses dépendances terminées (borné à `MAX_TASK_BYTES`).
pub(crate) fn build_enriched_step_task(
    step: &NativeGraphStepRecord,
    all_steps: &[NativeGraphStepRecord],
) -> String {
    if !step.pass_dependency_results || step.depends_on.is_empty() {
        return step.delegation.task.clone();
    }
    let by_id: BTreeMap<&str, &NativeGraphStepRecord> = all_steps
        .iter()
        .map(|s| (s.step_id.as_str(), s))
        .collect();
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
            format!("{trimmed_header}\n...\n\n## Consigne de cette étape\n{}", step.delegation.task)
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
    let completed = graph.steps.iter().filter(|s| s.state == "completed").count();
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
    let cancelled = graph.steps.iter().filter(|s| s.state == "cancelled").count();
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
                delegation.request_id =
                    step_request_id(&request.request_id, &s.step_id, &delegation.request_id);
                NativeGraphStepRecord {
                    step_id: s.step_id.clone(),
                    depends_on: s.depends_on.clone(),
                    pass_dependency_results: s.pass_dependency_results,
                    delegation,
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
                graph.steps[idx].started_at = Some(now);
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
            }
        }
        other => {
            graph.steps[idx].state = "failed".into();
            graph.steps[idx].error = Some(format!("unexpected_admission_response:{other:?}").chars().take(256).collect());
            graph.steps[idx].finished_at = Some(now);
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

/// Appelé à chaque cadence du daemon (`native_delegation::tick`).
/// Coût quasi nul quand aucun graphe n'est actif.
pub(super) fn tick(state: &Arc<Mutex<DaemonState>>) {
    let mut launches: Vec<(NativeGraphRecord, usize, AutoDelegateV2)> = Vec::new();
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
                    let launch_key = format!("{}:{}", graph.graph_id, step_id);
                    if !st.graph_launching_steps.insert(launch_key) {
                        continue;
                    }
                    if let Some(idx) = graph.steps.iter().position(|s| s.step_id == step_id) {
                        let enriched_task = build_enriched_step_task(&graph.steps[idx], &graph.steps);
                        let mut delegation = graph.steps[idx].delegation.clone();
                        delegation.task = enriched_task;
                        graph.steps[idx].state = "launching".into();
                        dirty = true;
                        launches.push((graph.clone(), idx, delegation));
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

    for (graph_snapshot, idx, delegation) in launches {
        let state_bg = Arc::clone(state);
        let graph_id = graph_snapshot.graph_id.clone();
        let step_id = graph_snapshot.steps[idx].step_id.clone();
        let suppress = graph_snapshot.notify_policy == GraphNotifyPolicy::OnGraphDoneOrFailure;
        let _ = thread::Builder::new()
            .name(format!("bridget-graph-{}", &step_id))
            .spawn(move || {
                let launch_key = format!("{graph_id}:{step_id}");
                let response = match prepare_exec_conn(&graph_snapshot, &step_id, &state_bg) {
                    Ok((exec_conn, _guard)) => super::native_delegation::handle_with_fact_pub(
                        &exec_conn,
                        NativeDelegationRequest::AutoDelegateV2(delegation),
                        graph_snapshot.attested_permissions.clone(),
                        None,
                        None,
                        &state_bg,
                        None,
                    ),
                    Err(code) => DaemonToWrapper::NativeDelegationResult {
                        result: graph_failure(&code, "Contexte parent révoqué ou indisponible."),
                    },
                };

                {
                    let mut st = state_bg.lock().recover_poison();
                    st.graph_launching_steps.remove(&launch_key);
                    if let Ok(Some(mut current)) = st.delegation_store.get_graph(&graph_id)
                        && let Some(pos) = current.steps.iter().position(|s| s.step_id == step_id)
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
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delegation::DelegationStore;
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
            agent_type: None,
        }
    }

    fn sample_step(id: &str, deps: &[&str], task: &str) -> DelegateGraphStep {
        DelegateGraphStep {
            step_id: id.into(),
            depends_on: deps.iter().map(|s| (*s).into()).collect(),
            pass_dependency_results: true,
            delegation: sample_auto(task),
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
        assert!(enriched.contains("Étape `A` : état=completed, task_id=task-a, commit=deadbeef, fichiers_modifies=3"));
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
}
