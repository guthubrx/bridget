//! Reprise native198 d'une tâche dont Start a été refusé pour capacité avant tout reçu.
//!
//! Ordre des effets, chacun précédé d'une relecture fraîche de la tâche :
//! plan durable → fence du worker (reçu Refused sans naissance) → invalidation de
//! l'ancienne lease → préparation du MÊME worktree → nouvelle lease → Start/Release.
//! Un plan interrompu reprend à sa phase ; une commande existante est observée,
//! jamais relancée. Aucun nouveau décideur, profil, droit ou tâche n'est créé.
use super::super::origin_proof;
use super::*;
use crate::delegation::{
    MAX_START_RECOVERIES, StartAttemptRecord, StartRecovery, StartRecoveryPhase,
};
use bridget_transport::protocol::{
    ProviderPermissions, WORKER_INSPECTION_TOKEN, WorkerCapability, WorkerCommandIdentity,
    WorkerFencedRouting, WorkerIdentity, WorkerPrepare, WorkerRefusal, WorkerRequestOperation,
    WorkerResponseOperation, WorkspaceRequest,
};
use std::collections::HashSet;
use std::sync::LazyLock;

const FENCE_TIMEOUT: Duration = Duration::from_secs(30);
const PREPARE_TIMEOUT: Duration = Duration::from_secs(30);

/// Sous-codes fermés d'un refus de capacité. `unknown` = le pair négocié n'a pas
/// pu préciser ; l'absence de sous-code désigne un refus historique sans preuve.
const CAUSES: &[&str] = &[
    "unknown",
    "inbound_queue",
    "command_store",
    "preparation_metadata",
    "request_frame",
    "master_pending",
    "master_request_frame",
];

static CLAIMS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Claim exclusive par tâche pendant les E/S ; libérée aussi en cas d'erreur.
struct Claim(String);

impl Claim {
    fn take(task_id: &str) -> Option<Self> {
        CLAIMS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(task_id.to_owned())
            .then(|| Self(task_id.to_owned()))
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        CLAIMS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.0);
    }
}

/// `Some(None)` : refus historique sans cause ; `Some(Some(c))` : cause fermée ;
/// `None` : autre erreur, jamais éligible. O(|error|).
pub(super) fn capacity_refusal(error: &str) -> Option<Option<&str>> {
    let rest = error.strip_prefix("remote_start_failed:remote_start:ResourceLimit")?;
    if rest.is_empty() {
        return Some(None);
    }
    let cause = rest.strip_prefix(":cause=")?;
    CAUSES.contains(&cause).then_some(Some(cause))
}

/// Le quota de métadonnées suit déjà la création du reçu : il ne relève pas
/// de la reprise avant reçu. Un refus ancien reste vérifiable par fence.
pub(super) fn recoverable_refusal(error: &str) -> bool {
    matches!(capacity_refusal(error), Some(cause) if cause != Some("preparation_metadata"))
}

pub(super) fn capacity_message(cause: Option<&str>) -> &'static str {
    match cause {
        None => {
            "Le démarrage distant a été refusé pour capacité. L’ancien retour ne précise ni la cause ni l’état du reçu de commande : file du worker, stockage de commandes ou file du maître restent non distingués."
        }
        Some("inbound_queue") => {
            "Le démarrage distant a été refusé : la file d’entrée du worker était pleine à cet instant. La mission n’a pas été remise."
        }
        Some("command_store") => {
            "Le démarrage distant a été refusé : le stockage de commandes du worker était saturé à cet instant. La mission n’a pas été remise."
        }
        Some("preparation_metadata") => {
            "Le démarrage distant a été refusé : la réservation des métadonnées de préparation du worker a été refusée pour capacité. La mission n’a pas été remise."
        }
        Some("request_frame") => {
            "Le démarrage distant a été refusé : l’enveloppe de la demande dépasse la borne du worker. La mission n’a pas été remise."
        }
        Some("master_pending") => {
            "Le maître n’a pas envoyé le démarrage : sa table de réponses en attente était pleine. Le worker n’a rien reçu."
        }
        Some("master_request_frame") => {
            "Le maître n’a pas envoyé le démarrage : l’enveloppe dépasse sa borne. Le worker n’a rien reçu."
        }
        Some(_) => {
            "Le démarrage distant a été refusé pour capacité ; le worker n’a pas précisé la cause. La mission n’a pas été remise."
        }
    }
}

/// Texte fermé pour les refus de la reprise elle-même. `None` : code étranger.
pub(super) fn refusal_guidance(code: &str) -> Option<(&'static str, &'static str, &'static str)> {
    let base = code.split_once(':').map_or(code, |(base, _)| base);
    Some(match base {
        "start_recovery_capability_required" => (
            "Le worker connecté n’annonce pas la reprise de démarrage (start_recovery_v1).",
            "Conservez la tâche, la request_id et le refus d’origine. Aucun nouveau Start n’a été envoyé à ce pair. Rejouez la même demande après chargement d’un worker qui annonce cette capacité ; ne créez pas de nouvelle tâche.",
            "Une mise à jour du worker par la maintenance native est nécessaire ; demandez à l’utilisateur de l’autoriser, sans redémarrage forcé.",
        ),
        "start_recovery_local_capacity" => (
            "Fleet a refusé localement la réservation avant tout Start distant. La cause fermée figure après les deux-points et la clé est conservée dans l’historique.",
            "Attendez que le propriétaire libère nativement la capacité concernée, puis rejouez exactement la même admission. Bridget réserve une autre tentative dans la même tâche ; aucune commande distante incertaine n’est rejouée.",
            "Si la capacité ne peut pas être libérée dans votre périmètre, demandez à l’utilisateur quelle mission doit être terminée. Ne forcez aucun quota ni arrêt tiers.",
        ),
        "start_recovery_reservation_expired" => (
            "La réservation interrompue a expiré avant tout envoi de Start. Son identité est conservée, sans conclure à un arrêt distant.",
            "Rejouez strictement la même admission. Bridget doit fermer cette réservation auprès du worker avant une nouvelle commande, sans changer la tâche ni le dossier.",
            "Si le worker ne confirme pas la fermeture ou si le budget de reprises est atteint, demandez une décision à l’utilisateur ; ne créez pas de remplaçant.",
        ),
        "start_recovery_exhausted" => (
            "Le nombre de reprises de démarrage admises pour cette tâche est atteint.",
            "Conservez la tâche et son historique de commandes. Relisez son statut ; aucune reprise supplémentaire n’est tentée automatiquement.",
            "Demandez à l’utilisateur s’il faut libérer de la capacité côté worker ou accepter un nouveau parcours pour cette mission.",
        ),
        "start_fence_refused" | "start_fence_invalid" => (
            "Le worker n’a pas fermé durablement l’ancienne commande avant naissance. Aucune nouvelle commande n’a été créée.",
            "Le plan de reprise est conservé tel quel. Rejouez la même demande : le même fence exact sera renvoyé. Le code après les deux-points est la cause renvoyée par le worker ; une commande déjà démarrée ou détenue n’est jamais écrasée.",
            "Si le refus persiste avec CommandMismatch ou UnknownExecution, demandez à l’utilisateur de décider ; la cause côté worker reste à établir.",
        ),
        "start_recovery_lease_unknown" => (
            "L’ancienne réservation Fleet de cette commande est introuvable ; sa fermeture ne peut pas être prouvée.",
            "Conservez le plan de reprise et ne lancez aucun remplaçant. La cause de la disparition de la réservation est inconnue : vérifiez l’inventaire Fleet de cette tâche.",
            "Si l’inventaire ne la retrouve pas, demandez à l’utilisateur de décider du sort de cette commande.",
        ),
        "start_recovery_prepared_changed" | "start_recovery_policy_changed" => (
            "La réattestation du même dossier de travail ne correspond plus au placement, à la définition ou à la politique admis.",
            "Conservez la tâche, son dossier et la clé. Comparez le placement, la définition et la politique figés avec ceux du worker ; ne modifiez rien pour rendre la reprise acceptable.",
            "Si la configuration doit changer, l’utilisateur doit valider cette décision séparément.",
        ),
        "start_recovery_task_changed" => (
            "La tâche a changé (annulation, clôture ou pause) pendant la reprise ; l’effet suivant n’a pas été exécuté.",
            "Relisez le statut de cette même tâche. Une annulation prime : ne relancez pas la commande.",
            "Aucune décision n’est requise sauf si le statut reste incohérent.",
        ),
        _ => return None,
    })
}

/// Vrai si un rejeu strict de ce reçu Admitted doit être confié à `resume`.
/// Lecture seule ; `resume` revalide tout sous claim. O(|request|+|canon|).
pub(super) fn wanted(task: &Task, receipt: &AutoAdmissionReceipt) -> bool {
    if receipt.state != AutoAdmissionState::Admitted
        || receipt.task_id.as_deref() != Some(task.task_id.as_str())
        || serde_json::to_vec(&task.request).ok().as_deref()
            != receipt.delegate_canonical.as_deref()
        || task.remote_placement.is_none()
        || task.state != "starting"
        || task.mission_issued_at.is_some()
    {
        return false;
    }
    match &task.start_recovery {
        Some(plan) if plan.in_progress() => true,
        Some(plan) if plan.phase != StartRecoveryPhase::Closed => false,
        _ => task.error.as_deref().is_some_and(|code| {
            recoverable_refusal(code)
                || code == "remote_start_failed:start_recovery_reservation_expired"
        }),
    }
}

/// Fixe la fin d'une tentative dont le Start de reprise a été refusé pour capacité.
pub(super) fn record_refusal(
    state: &Arc<Mutex<DaemonState>>,
    task_id: &str,
    recovery: Option<&StartRecovery>,
    reason: WorkerRefusal,
    _cause: Option<&str>,
) {
    if recovery.is_none() || reason != WorkerRefusal::ResourceLimit {
        return;
    }
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(Some(mut task)) = st.delegation_store.get(task_id) else {
        return;
    };
    if let Some(plan) = task.start_recovery.as_mut()
        && plan.phase == StartRecoveryPhase::Relaunched
    {
        plan.phase = StartRecoveryPhase::Closed;
        if st.delegation_store.save(&task).is_err() {
            warn!("phase de reprise non persistée après refus de capacité");
        }
    }
}

/// Release confirmé : la reprise est soldée et le refus d'origine n'est plus actuel.
pub(super) fn record_released(
    state: &Arc<Mutex<DaemonState>>,
    task_id: &str,
    recovery: Option<&StartRecovery>,
) {
    if recovery.is_none() {
        return;
    }
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(Some(mut task)) = st.delegation_store.get(task_id) else {
        return;
    };
    if let Some(plan) = task.start_recovery.as_mut() {
        plan.phase = StartRecoveryPhase::Done;
        task.error = None;
        if st.delegation_store.save(&task).is_err() {
            warn!("reprise soldée non persistée");
        }
    }
}

struct Context {
    owner: String,
    parent: ProviderPermissions,
    worker: WorkerIdentity,
    routing: Option<WorkerFencedRouting>,
}

fn lease_of(st: &DaemonState, command: &WorkerCommandIdentity) -> Option<crate::fleet::SpawnLease> {
    st.managed_spawns
        .get(&command.command_id)
        .map(|record| record.lease.clone())
        .or_else(|| {
            st.fleet
                .recovery_candidates()
                .into_iter()
                .find(|candidate| {
                    candidate.lease.command_id == command.command_id
                        && candidate.lease.instance_id == command.instance_id
                        && candidate.lease.generation == command.generation
                })
                .map(|candidate| candidate.lease)
        })
}

pub(super) fn admission_open(st: &DaemonState) -> Result<(), String> {
    let control = crate::referent_control::read(st.store.connection())
        .map_err(|_| "control_state_unavailable")?;
    let halt = st
        .delegation_store
        .delegation_halt()
        .map_err(|_| "control_state_unavailable")?;
    if let crate::referent_control::Admission::Deferred { motif } =
        crate::referent_control::admit_native_delegation(&control, &halt)
    {
        return Err(if motif == "pause" {
            "referent_paused"
        } else {
            motif
        }
        .into());
    }
    Ok(())
}

/// Relecture fraîche avant chaque effet : annulation, clôture, pause/gel et
/// plan inchangé priment. O(log t).
fn fresh(
    state: &Arc<Mutex<DaemonState>>,
    task_id: &str,
    phases: &[StartRecoveryPhase],
) -> Result<(Task, StartRecovery), String> {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    let plan = task.start_recovery.clone();
    match plan {
        Some(plan)
            if task.state == "starting"
                && task.mission_issued_at.is_none()
                && !task.guard_released
                && !task.cleanup_done
                && task.result.is_none()
                && task.round_no() == 1
                && phases.contains(&plan.phase) =>
        {
            admission_open(&st)?;
            Ok((task, plan))
        }
        _ => Err("start_recovery_task_changed".into()),
    }
}

fn peer_has_recovery(st: &DaemonState, machine: &str) -> bool {
    st.remote_workers.get(machine).is_some_and(|peer| {
        peer.capabilities
            .contains(&WorkerCapability::StartRecoveryV1)
    })
}

/// Le reçu V2 porte son choix initial, sans nouveau décideur ni sélection.
fn frozen_start_routing(
    task: &Task,
    receipt: &AutoAdmissionReceipt,
) -> Result<Option<WorkerFencedRouting>, String> {
    if receipt.metadata_version != 2 {
        return Ok(None);
    }
    let auto = recovered_routing_request(receipt)?;
    let plan = frozen_routing_plan(receipt)?;
    let index = routing_choice_index(
        &plan,
        receipt
            .candidate_id
            .as_deref()
            .ok_or("auto_admission_invalid")?,
    )?;
    let candidate = &plan.candidates[index];
    let observed = &plan.observations[index];
    let snapshot: RoutingAdmissionSnapshot =
        serde_json::from_slice(&receipt.metadata).map_err(|_| "auto_admission_invalid")?;
    let decision = snapshot.decision.ok_or("auto_admission_invalid")?;
    let NativeDelegationRequest::Delegate {
        machine_id,
        agent_type,
        model,
        effort,
        cwd,
        ..
    } = &task.request
    else {
        return Err("invalid_record".into());
    };
    let placement = task
        .remote_placement
        .as_ref()
        .ok_or("remote_placement_missing")?;
    // La sonde figée décrit le profil initial. Quand le décideur a changé
    // l'effort, sa définition diffère légitimement de celle attestée à Prepare.
    // Le tuple Task/placement porte cette définition d'exécution admise ; le
    // worker la revalide encore sur le même dossier avant Start et Release.
    let initial_definition_matches = if decision.effort == candidate.default_effort {
        observed.definition_digest == placement.definition_digest
    } else {
        AgentRegistry::from_resolved(agent_type, &task.definition)
            .and_then(|registry| registry.pin_delegation_model(agent_type, model, candidate.default_effort.as_deref()))
            .and_then(|registry| registry.resolved_definition(agent_type))
            .is_ok_and(|initial| initial.digest == observed.definition_digest)
    };
    if let Some(code) = routing_change_error("start_recovery", &[
        ("machine", machine_id.as_deref() != Some(candidate.machine_id.as_str())
            || decision.machine_id != candidate.machine_id),
        ("profile", *agent_type != candidate.agent_type
            || decision.candidate_id != receipt.candidate_id.as_deref().unwrap_or_default()
            || decision.connection_id != candidate.connection_id || decision.engine != candidate.engine),
        ("model", *model != candidate.model || decision.model != candidate.model),
        ("effort", *effort != decision.effort),
        ("canonical", *cwd != auto.cwd),
        ("base", auto.expected_base_commit != placement.base_commit),
        ("definition", task.definition.digest != placement.definition_digest || !initial_definition_matches),
        ("permissions", observed.permission_digest != placement.permission_digest),
    ]) { return Err(code); }
    Ok(Some(WorkerFencedRouting {
        workload: auto.workload,
        connection_id: candidate.connection_id.clone(),
        engine: candidate.engine.clone(),
        expected_definition_digest: placement.definition_digest.clone(),
        expected_permission_digest: placement.permission_digest.clone(),
    }))
}

/// Gardes statiques, sous verrou, avant tout effet de cette tentative.
fn gather(
    state: &Arc<Mutex<DaemonState>>,
    conn: &str,
    task_id: &str,
    receipt: &AutoAdmissionReceipt,
    attested: Option<&ProviderPermissions>,
    thread: Option<&str>,
) -> Result<Result<Context, Value>, String> {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    recheck_attested_thread(&st, conn, thread)?;
    let (owner, instance) = live_connection_identity(&st, conn).ok_or("identity_unavailable")?;
    let task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    if !wanted(&task, receipt) {
        // Autre rejeu, autre reprise ou état modifié : simple lecture.
        return Ok(Err(current_task_view(&st, &task, true)?));
    }
    if !can_own_task(&st, &task, &owner, &instance)? {
        return Err("task_unavailable".into());
    }
    let NativeDelegationRequest::Delegate {
        request_id,
        agent_type,
        machine_id: Some(machine),
        ..
    } = &task.request
    else {
        return Err("invalid_record".into());
    };
    let placement = task
        .remote_placement
        .as_ref()
        .ok_or("remote_placement_missing")?;
    let parent = parent_fact(&st, &owner, &instance, Some(request_id), attested)?
        .ok_or("permission_attestation_unavailable")?;
    let snapshot = task
        .permission_snapshot
        .as_ref()
        .ok_or("permission_not_inherited")?;
    if !continuation_parent_matches(Some(&snapshot.parent), Some(&parent)) {
        return Err("permission_changed".into());
    }
    let provision = st.registry.machine(machine)?;
    let mapping = st
        .registry
        .project_mapping(machine, &placement.project_id)?;
    if !provision.permits_agent_type(agent_type)
        || provision.worker_id != placement.worker_id
        || provision.provisioning_revision != placement.provisioning_revision
        || mapping.mapping_revision != placement.mapping_revision
    {
        return Err("remote_placement_changed".into());
    }
    if task.round_no() != 1
        || !task.rounds.is_empty()
        || task.guard_released
        || task.cleanup_done
        || task.result.is_some()
        || !task.remote_result_slots.is_empty()
        || !task.remote_result_receipts.is_empty()
        || !task.remote_prepared
        || task
            .workspace_repairs
            .last()
            .is_some_and(|repair| repair.blocks_dispatch())
    {
        return Err("start_recovery_task_changed".into());
    }
    recheck_remote_dispatch(&st, &task, &task)?;
    admission_open(&st)?;
    if !peer_has_recovery(&st, machine) {
        return Err("start_recovery_capability_required".into());
    }
    let history = task
        .start_recovery
        .as_ref()
        .map_or(0, |plan| plan.history.len() + 1);
    if task
        .start_recovery
        .as_ref()
        .is_none_or(|plan| !plan.in_progress())
        && history >= MAX_START_RECOVERIES
    {
        return Err("start_recovery_exhausted".into());
    }
    Ok(Ok(Context {
        owner,
        parent,
        worker: remote_worker_identity(placement),
        routing: frozen_start_routing(&task, receipt)?,
    }))
}

/// Écrit l'intention AVANT toute E/S : un crash retrouve le même plan.
fn persist_plan(state: &Arc<Mutex<DaemonState>>, task_id: &str) -> Result<(), String> {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let mut task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    if task
        .start_recovery
        .as_ref()
        .is_some_and(|plan| plan.in_progress())
    {
        return Ok(());
    }
    if task.state != "starting" || task.mission_issued_at.is_some() {
        return Err("start_recovery_task_changed".into());
    }
    admission_open(&st)?;
    let previous = remote_command_identity(&st, &task)
        .map_err(|e| format!("start_recovery_lease_unknown:{e:?}"))?;
    let lease = lease_of(&st, &previous).ok_or("start_recovery_lease_unknown")?;
    let (history, previous_issued_at, local_refusals) = match task.start_recovery.take() {
        Some(old) => {
            // Cause fermée de la tentative qui se termine ; "historical" = refus sans sous-code.
            let cause = task
                .error
                .as_deref()
                .and_then(capacity_refusal)
                .map(|cause| cause.unwrap_or("historical").to_owned());
            let mut history = old.history.clone();
            history.push(StartAttemptRecord {
                command: old.previous.clone(),
                issued_at: old.previous_issued_at,
                deadline_at: old.previous_deadline_at,
                cause,
            });
            (history, old.issued_at, old.local_refusals)
        }
        None => (
            Vec::new(),
            task.launch_issued_at.unwrap_or(task.created_at),
            Vec::new(),
        ),
    };
    if history.len() >= MAX_START_RECOVERIES {
        return Err("start_recovery_exhausted".into());
    }
    let attempt = history.len() + 1;
    task.start_recovery = Some(StartRecovery {
        phase: StartRecoveryPhase::Planned,
        previous,
        previous_issued_at,
        previous_deadline_at: lease.deadline_at,
        new_command_id: format!("{}-start-{attempt}", task.task_id),
        issued_at: unix_timestamp(),
        history,
        local_refusals,
    });
    st.delegation_store.save(&task)
}

fn set_phase(
    state: &Arc<Mutex<DaemonState>>,
    task_id: &str,
    from: StartRecoveryPhase,
    to: StartRecoveryPhase,
) -> Result<(), String> {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let mut task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    match task.start_recovery.as_mut() {
        Some(plan) if plan.phase == from && task.state == "starting" => plan.phase = to,
        _ => return Err("start_recovery_task_changed".into()),
    }
    st.delegation_store.save(&task)
}

/// Fence exact, rejouable : seul `StartFenced` avance. Tout autre retour garde la phase.
fn fence(state: &Arc<Mutex<DaemonState>>, task_id: &str, ctx: &Context) -> Result<(), String> {
    let (task, plan) = fresh(state, task_id, &[StartRecoveryPhase::Planned])?;
    let placement = task
        .remote_placement
        .clone()
        .ok_or("remote_placement_missing")?;
    let response = remote_worker_roundtrip(
        state,
        &ctx.worker,
        &plan.previous,
        WorkerRequestOperation::FenceUnstarted {
            placement,
            deadline_at: plan.previous_deadline_at,
        },
        Instant::now() + FENCE_TIMEOUT,
    )
    .map_err(|reason| format!("start_fence_refused:{reason:?}"))?;
    if !matches!(response, WorkerResponseOperation::StartFenced {}) {
        return Err("start_fence_invalid".into());
    }
    set_phase(
        state,
        task_id,
        StartRecoveryPhase::Planned,
        StartRecoveryPhase::Fenced,
    )
}

/// Invalide l'ancienne lease Fleet ; idempotent après crash (entrée déjà Stopped).
fn invalidate_previous(state: &Arc<Mutex<DaemonState>>, task_id: &str) -> Result<(), String> {
    let (task, plan) = fresh(state, task_id, &[StartRecoveryPhase::Fenced])?;
    {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        match lease_of(&st, &plan.previous) {
            Some(lease) => {
                st.fleet
                    .invalidate_for_stop(&lease)
                    .map_err(|e| e.to_string())?;
                st.managed_spawns.remove(&plan.previous.command_id);
                st.managed_by_instance.remove(&plan.previous.instance_id);
                st.managed_terminal_instances
                    .insert(plan.previous.instance_id.clone());
            }
            None => {
                let stopped = st
                    .fleet
                    .desired_entry(&task.child)
                    .map_err(|e| e.to_string())?
                    .is_some_and(|entry| entry.lifecycle_state == DesiredLifecycleState::Stopped);
                if !stopped {
                    return Err("start_recovery_lease_unknown".into());
                }
            }
        }
    }
    set_phase(
        state,
        task_id,
        StartRecoveryPhase::Fenced,
        StartRecoveryPhase::Invalidated,
    )
}

/// Reconstruit WorkerPrepare depuis la requête figée et le mapping frais ; les
/// droits sont ceux du parent actuel, jamais plus larges.
fn rebuild_prepare(
    st: &DaemonState,
    task: &Task,
    owner: &str,
    parent: &ProviderPermissions,
) -> Result<WorkerPrepare, String> {
    let NativeDelegationRequest::Delegate {
        machine_id: Some(machine),
        project_id: Some(project_id),
        agent_type,
        model,
        effort,
        cwd,
        posture,
        workspace,
        ..
    } = &task.request
    else {
        return Err("invalid_record".into());
    };
    let placement = task
        .remote_placement
        .as_ref()
        .ok_or("remote_placement_missing")?;
    let provision = st.registry.machine(machine)?;
    let original = st.registry.project_mapping(machine, project_id)?;
    let origin_host = origin_proof::host_for_owner(st, owner)?;
    let mapping = original
        .for_origin(machine, &origin_host)
        .ok_or("remote_origin_host_mismatch")?;
    let offset = Path::new(cwd)
        .strip_prefix(&mapping.source_root)
        .map_err(|_| "remote_mapping_mismatch")?;
    let relative = if offset.as_os_str().is_empty() {
        ".".into()
    } else {
        offset.to_string_lossy().into_owned()
    };
    Ok(WorkerPrepare {
        machine_id: machine.clone(),
        project_id: project_id.clone(),
        provisioning_revision: provision.provisioning_revision,
        mapping_revision: mapping.mapping_revision,
        origin_host: mapping.source_host.clone(),
        origin_cwd: cwd.clone(),
        source_root: mapping.source_root.to_string_lossy().into_owned(),
        remote_root: mapping.remote_root.to_string_lossy().into_owned(),
        source_root_identity: mapping.expected_git_identity.clone(),
        relative_cwd: relative,
        base_commit: placement.base_commit.clone(),
        agent_type: agent_type.clone(),
        model: model.clone(),
        effort: effort.clone(),
        posture: *posture,
        parent: parent.clone(),
        workspace: workspace.clone(),
    })
}

/// Réattestation du même worktree puis écriture du snapshot parent ACTUEL, avant
/// tout nouveau Start : sans elle, Release strict refuserait un parent équivalent.
fn prepare_same_worktree(
    state: &Arc<Mutex<DaemonState>>,
    task_id: &str,
    ctx: &Context,
) -> Result<bridget_transport::protocol::WorkerPrepared, String> {
    let (task, plan) = fresh(
        state,
        task_id,
        &[
            StartRecoveryPhase::Invalidated,
            StartRecoveryPhase::Prepared,
        ],
    )?;
    let (prepare, workspace) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let prepare = rebuild_prepare(&st, &task, &ctx.owner, &ctx.parent)?;
        let workspace = match (&prepare.workspace, &task.workspace) {
            (Some(WorkspaceRequest::Worktree { .. }|WorkspaceRequest::CommitView {}), Some(record)) => {
                Some(serde_json::to_value(record).map_err(|_| "remote_workspace_invalid")?)
            }
            (Some(WorkspaceRequest::Worktree { .. }|WorkspaceRequest::CommitView {}), None) => {
                return Err("remote_workspace_missing".into());
            }
            _ => None,
        };
        (prepare, workspace)
    };
    let placement = task
        .remote_placement
        .clone()
        .ok_or("remote_placement_missing")?;
    let provisional = WorkerCommandIdentity {
        command_id: plan.new_command_id.clone(),
        task_id: task.task_id.clone(),
        instance_id: String::new(),
        generation: 0,
    };
    let response = remote_worker_roundtrip(
        state,
        &ctx.worker,
        &provisional,
        WorkerRequestOperation::PrepareFenced {
            previous: plan.previous.clone(),
            prepare,
            placement: placement.clone(),
            workspace: workspace.clone(),
            routing: ctx.routing.clone(),
        },
        Instant::now() + PREPARE_TIMEOUT,
    )
    .map_err(|reason| format!("remote_prepare:{reason:?}"))?;
    let WorkerResponseOperation::Prepared { prepared } = response else {
        return Err("remote_prepare_invalid".into());
    };
    let snapshot_policy = task
        .permission_snapshot
        .as_ref()
        .map(|snapshot| snapshot.child_policy.clone())
        .ok_or("permission_not_inherited")?;
    let workspace_matches = match (&task.workspace, &prepared.workspace) {
        (Some(frozen), Some(observed)) => {
            crate::remote_worker::same_workspace_anchor(frozen, observed)
        }
        (None, None) => true,
        _ => false,
    };
    if prepared.token == WORKER_INSPECTION_TOKEN
        || prepared.placement != placement
        || prepared.definition != task.definition
        || Some(prepared.effective_posture) != task.effective_posture
        || !workspace_matches
    {
        return Err("start_recovery_prepared_changed".into());
    }
    if prepared.child_policy != snapshot_policy {
        return Err("start_recovery_policy_changed".into());
    }
    // Revalider chaque retour d'E/S, puis écrire snapshot et phase d'un seul save.
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let mut task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    let unchanged = task.state == "starting"
        && task.mission_issued_at.is_none()
        && task.remote_placement.as_ref() == Some(&placement)
        && task.start_recovery.as_ref().is_some_and(|current| {
            current.new_command_id == plan.new_command_id
                && matches!(
                    current.phase,
                    StartRecoveryPhase::Invalidated | StartRecoveryPhase::Prepared
                )
        });
    if !unchanged {
        return Err("start_recovery_task_changed".into());
    }
    admission_open(&st)?;
    task.permission_snapshot = Some(crate::native_permissions::snapshot(
        &task.owner,
        &task.owner_instance,
        ctx.parent.clone(),
        prepared.child_policy.clone(),
        Path::new(&task.cwd),
    ));
    if let Some(current) = task.start_recovery.as_mut() {
        current.phase = StartRecoveryPhase::Prepared;
    }
    st.delegation_store.save(&task)?;
    Ok(prepared)
}

/// Une issue locale de quota est connue sans exécution distante : archiver
/// sa clé puis préparer une clé nouvelle, toujours dans la même tâche.
pub(super) fn local_reservation_refusal(
    st: &DaemonState,
    task: &mut Task,
    issue: &crate::idempotency::SpawnCommandIssue,
) -> Result<String, String> {
    let crate::idempotency::SpawnCommandIssue::Failed { category, .. } = issue else {
        return Err("start_recovery_lease_unknown".into());
    };
    if !matches!(
        category.as_str(),
        "quota_exceeded" | "children_quota_exceeded" | "spawn_timeout"
    ) {
        return Err("start_recovery_lease_unknown".into());
    }
    let plan = task
        .start_recovery
        .as_mut()
        .ok_or("start_recovery_task_changed")?;
    if plan.phase != StartRecoveryPhase::Prepared {
        return Err("start_recovery_task_changed".into());
    }
    if plan.local_refusals.len() >= MAX_START_RECOVERIES {
        return Err("start_recovery_exhausted".into());
    }
    plan.local_refusals
        .push(crate::delegation::StartLocalRefusal {
            command_id: plan.new_command_id.clone(),
            issued_at: plan.issued_at,
            code: category.clone(),
        });
    plan.new_command_id = format!(
        "{}-local-{}",
        plan.local_refusals[0].command_id,
        plan.local_refusals.len()
    );
    plan.issued_at = unix_timestamp();
    task.error = Some(format!(
        "remote_start_failed:start_recovery_local_capacity:{category}"
    ));
    st.delegation_store.save(task)?;
    Ok(format!("start_recovery_local_capacity:{category}"))
}

/// Le plan peut attendre sans consommer son futur bail. L'heure du canon est
/// fixée durablement juste avant sa première réservation, jamais prolongée ensuite.
fn launch_plan(state: &Arc<Mutex<DaemonState>>, task_id: &str) -> Result<StartRecovery, String> {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let mut task = st
        .delegation_store
        .get(task_id)?
        .ok_or("task_unavailable")?;
    let mut plan = task
        .start_recovery
        .clone()
        .ok_or("start_recovery_task_changed")?;
    if plan.phase != StartRecoveryPhase::Prepared
        || task.guard_released
        || task.cleanup_done
        || task.state != "starting"
        || task.mission_issued_at.is_some()
    {
        return Err("start_recovery_task_changed".into());
    }
    admission_open(&st)?;
    if !st.fleet.knows_command(&plan.new_command_id) {
        plan.issued_at = unix_timestamp();
        task.start_recovery = Some(plan.clone());
        st.delegation_store.save(&task)?;
    } else if let Some(candidate) = st
        .fleet
        .recovery_candidates()
        .into_iter()
        .find(|c| c.lease.command_id == plan.new_command_id)
    {
        if candidate.lease.deadline_at <= unix_timestamp() {
            // Prepared est durable : aucun Start de cette commande n'a pu partir.
            // Conserver le tuple réservé ; la prochaine reprise devra le fermer
            // par le worker avant toute nouvelle commande. Aucune preuve Terminal.
            if candidate.lease.name != task.child
                || candidate.lease.remote_placement != task.remote_placement
            {
                return Err("start_recovery_lease_unknown".into());
            }
            task.child_instance = Some(candidate.lease.instance_id);
            plan.phase = StartRecoveryPhase::Closed;
            task.start_recovery = Some(plan);
            task.error = Some("remote_start_failed:start_recovery_reservation_expired".into());
            st.delegation_store.save(&task)?;
            return Err("start_recovery_reservation_expired".into());
        }
    } else {
        let issue = st
            .idempotency
            .spawn_commands(&st.fleet.supervisor_scope())
            .map_err(|_| "start_recovery_lease_unknown")?
            .into_iter()
            .find(|command| command.command_id == plan.new_command_id)
            .and_then(|command| command.issue)
            .ok_or("start_recovery_lease_unknown")?;
        let code = local_reservation_refusal(&st, &mut task, &issue)?;
        return Err(code);
    }
    Ok(plan)
}

/// Point d'entrée du rejeu strict d'un reçu Admitted au même canon.
pub(super) fn resume(
    state: &Arc<Mutex<DaemonState>>,
    conn: &str,
    task_id: &str,
    receipt: &AutoAdmissionReceipt,
    attested: Option<&ProviderPermissions>,
    thread: Option<&str>,
    permission_proof: Option<&bridget_transport::protocol::NativePermissionProof>,
) -> Result<Value, String> {
    let Some(_claim) = Claim::take(task_id) else {
        // Une reprise tourne déjà : observer, jamais lancer de second fournisseur.
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let task = st
            .delegation_store
            .get(task_id)?
            .ok_or("task_unavailable")?;
        return current_task_view(&st, &task, true);
    };
    let ctx = match gather(state, conn, task_id, receipt, attested, thread)? {
        Ok(ctx) => ctx,
        Err(view) => return Ok(view),
    };
    // Preuve native fraîche hors mutex ; relue après chaque retour d'E/S.
    let attest = |ctx: &Context| -> Result<(), String> {
        match (thread, permission_proof) {
            (Some(expected), Some(proof)) => {
                let fresh = origin_proof::reattest(conn, proof, state)
                    .map_err(|_| "permission_attestation_unavailable")?;
                if fresh.thread_id != expected || fresh.permissions.as_ref() != Some(&ctx.parent) {
                    return Err("permission_changed".into());
                }
            }
            (None, None) => {}
            _ => return Err("permission_attestation_unavailable".into()),
        }
        remote_parent_proof(state, &ctx.owner, &ctx.parent).map(|_| ())
    };
    attest(&ctx)?;
    persist_plan(state, task_id)?;
    let phase = fresh(
        state,
        task_id,
        &[
            StartRecoveryPhase::Planned,
            StartRecoveryPhase::Fenced,
            StartRecoveryPhase::Invalidated,
            StartRecoveryPhase::Prepared,
        ],
    )?
    .1
    .phase;
    if phase == StartRecoveryPhase::Planned {
        fence(state, task_id, &ctx)?;
        attest(&ctx)?;
    }
    if matches!(
        phase,
        StartRecoveryPhase::Planned | StartRecoveryPhase::Fenced
    ) {
        invalidate_previous(state, task_id)?;
    }
    let prepared = prepare_same_worktree(state, task_id, &ctx)?;
    attest(&ctx)?;
    let plan = launch_plan(state, task_id)?;
    let outcome = start_remote_task_with(
        state,
        task_id,
        &prepared,
        &ctx.parent,
        conn,
        thread,
        permission_proof,
        Some(&plan),
    );
    remote_admission_outcome(state, task_id, outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_refusal_accepts_only_closed_codes() {
        let base = "remote_start_failed:remote_start:ResourceLimit";
        assert_eq!(capacity_refusal(base), Some(None));
        for cause in CAUSES {
            assert_eq!(
                capacity_refusal(&format!("{base}:cause={cause}")),
                Some(Some(*cause))
            );
        }
        for other in [
            "remote_start_failed:remote_start:WorkerUnavailable",
            "remote_start_failed:remote_release:ResourceLimit",
            "remote_start_failed:remote_result_slot:ResourceLimit",
            "remote_start_failed:remote_start:ResourceLimit:cause=secret",
            "remote_start_failed:remote_start:ResourceLimitX",
            "remote_start:ResourceLimit",
        ] {
            assert_eq!(capacity_refusal(other), None, "{other}");
        }
    }

    #[test]
    fn guidance_covers_every_recovery_refusal_and_ignores_foreign_codes() {
        for code in [
            "start_recovery_capability_required",
            "start_recovery_exhausted",
            "start_fence_refused:CommandMismatch",
            "start_fence_invalid",
            "start_recovery_lease_unknown",
            "start_recovery_prepared_changed",
            "start_recovery_policy_changed",
            "start_recovery_task_changed",
        ] {
            let (message, recovery, human) = refusal_guidance(code).expect(code);
            assert!(!message.is_empty() && !recovery.is_empty() && !human.is_empty());
        }
        assert!(refusal_guidance("permission_changed").is_none());
    }

    fn plan(phase: StartRecoveryPhase) -> StartRecovery {
        StartRecovery {
            phase,
            previous: WorkerCommandIdentity {
                command_id: "task".into(),
                task_id: "task".into(),
                instance_id: "instance".into(),
                generation: 1,
            },
            previous_issued_at: 10,
            previous_deadline_at: 310,
            new_command_id: "task-start-1".into(),
            issued_at: 400,
            history: Vec::new(),
            local_refusals: Vec::new(),
        }
    }

    #[test]
    fn only_pre_lease_phases_hide_the_task_from_the_tick() {
        use StartRecoveryPhase::*;
        for (phase, in_progress) in [
            (Planned, true),
            (Fenced, true),
            (Invalidated, true),
            (Prepared, true),
            (Relaunched, false),
            (Closed, false),
            (Done, false),
        ] {
            assert_eq!(plan(phase).in_progress(), in_progress, "{phase:?}");
        }
    }

    #[test]
    fn plan_roundtrips_without_losing_the_old_tuple_or_new_command() {
        let mut original = plan(StartRecoveryPhase::Fenced);
        original.history.push(StartAttemptRecord {
            command: original.previous.clone(),
            issued_at: 10,
            deadline_at: 310,
            cause: Some("historical".into()),
        });
        let encoded = serde_json::to_string(&original).unwrap();
        assert_eq!(
            serde_json::from_str::<StartRecovery>(&encoded).unwrap(),
            original
        );
        assert!(encoded.contains("\"phase\":\"fenced\""));
        assert!(serde_json::from_str::<StartRecovery>(&encoded.replace("}", ",\"x\":1}")).is_err());
    }

    #[test]
    fn claim_is_exclusive_and_released_on_drop() {
        let first = Claim::take("task-claim-test").expect("première claim");
        assert!(Claim::take("task-claim-test").is_none());
        drop(first);
        assert!(Claim::take("task-claim-test").is_some());
    }
}

#[cfg(test)]
#[path = "native_start_recovery198_tests.rs"]
mod admission_tests;

#[cfg(test)]
#[path = "native_cancel201_tests.rs"]
mod cancel201_tests;
