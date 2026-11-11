//! État technique du worker 162. Aucune tâche métier ni réservation Fleet.
//!
//! Un seul worker écrit les reçus. Chaque wrapper publie un seul blob dans son
//! slot réservé ; il n'ouvre jamais le verrou du store ou ses métadonnées.

use crate::managed_process::ManagedMarker;
use bridget_transport::fsutil::{create_private_dir, write_private_file_atomic};
use bridget_transport::protocol::{
    HostResourceSnapshotV1, MAX_WORKER_RESULT_ENVELOPE_BYTES, MAX_WORKER_STORE_BYTES,
    MAX_WORKER_STORE_ENTRIES, RemotePlacement, RemoteResultEnvelope, WORKER_RECEIPT_RETENTION_SECS,
    WORKER_RELEASE_LEASE_SECS, WorkerCommandIdentity, WorkerCommandPhase, WorkerIdentity,
    WorkerRefusal, WorkerReleasePermit, WorkerResultDisposition, WorkerResultIdentity,
    WorkerResultSlot, WorkerSlotClosureProof, WorkerStartCapacityCause,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[path = "remote_worker_capacity.rs"]
mod capacity;
#[path = "remote_worker_proof.rs"]
mod proof;
#[path = "remote_worker_read.rs"]
mod read;
pub(crate) use capacity::recover_view_capacity;
#[cfg(test)]
use read::diagnose_local;
pub(crate) use read::{
    LiveWorkerBinding, diagnose_live_local, diagnostic_failure, live_worker_binding,
    qualify_workload_tools,
};

#[path = "remote_workspace_repair.rs"]
pub(crate) mod workspace_repair;

const MAX_METADATA_BYTES: usize = 64 * 1024;
const STORE_VERSION: u16 = 1;
const DRAIN_PROOF_RESERVATION: u64 = 4096;
const MAX_DRAIN_PROOF_BYTES: usize = 2048;

// SPEC171: bounded diagnostics for the existing V2 probe only. The same
// UUID/admission/candidate filters and 128 notes / 512 bytes as the master.
const MAX_PROBE_PERMISSION_NOTES: usize = 128;
const MAX_PROBE_PERMISSION_NOTE_BYTES: usize = 512;

#[derive(Clone, Copy)]
enum ProbePermissionSite {
    Projection,
    ScopedConstructor,
    Refresh,
    Recheck,
    BatchFinish,
}
impl ProbePermissionSite {
    fn as_str(self) -> &'static str {
        match self {
            Self::Projection => "projection",
            Self::ScopedConstructor => "scoped-constructor",
            Self::Refresh => "refresh",
            Self::Recheck => "recheck",
            Self::BatchFinish => "batch-finish",
        }
    }
}

fn permission_diagnostic_code(error: &str) -> &str {
    match error {
        "effort_unavailable"
        | "machine_path_protected"
        | "machine_path_unverifiable"
        | "model_unavailable"
        | "permission_attestation_unavailable"
        | "permission_hook_capture_changed"
        | "permission_hook_capture_unavailable"
        | "permission_hook_command_unavailable"
        | "permission_hook_control_changed"
        | "permission_hook_dependencies_unavailable"
        | "permission_hook_dependency_changed"
        | "permission_hook_dependency_unavailable"
        | "permission_hook_environment_unavailable"
        | "permission_hook_equivalence_unavailable"
        | "permission_hook_event_changed"
        | "permission_hook_execution_unavailable"
        | "permission_hook_git_config_changed"
        | "permission_hook_git_state_unavailable"
        | "permission_hook_input_budget"
        | "permission_hook_input_outside_project"
        | "permission_hook_input_unavailable"
        | "permission_hook_matcher_changed"
        | "permission_hook_path_program_changed"
        | "permission_hook_permissions_changed"
        | "permission_hook_permissions_unavailable"
        | "permission_hook_platform_unavailable"
        | "permission_hook_profile_budget"
        | "permission_hook_profile_changed"
        | "permission_hook_profile_incomplete"
        | "permission_hook_profile_path"
        | "permission_hook_profile_schema"
        | "permission_hook_profile_timeout"
        | "permission_hook_profile_unavailable"
        | "permission_hook_prompt_changed"
        | "permission_hook_recipe_source_changed"
        | "permission_hook_recipe_unavailable"
        | "permission_hook_selection_changed"
        | "permission_hook_selection_unavailable"
        | "permission_hook_settings_changed"
        | "permission_hook_target_controls_changed"
        | "permission_hook_target_controls_unavailable"
        | "permission_hook_type_unavailable"
        | "permission_hook_unattested_settings_input"
        | "permission_hook_unknown_dynamic_dependency"
        | "permission_hook_unknown_path_program"
        | "permission_hook_unmapped_permission_path"
        | "permission_hook_unported_ralph_state"
        | "permission_hook_unported_skills_repository"
        | "permission_hook_unported_skills_write"
        | "permission_hook_unported_tmux_state"
        | "permission_hook_unqualified_branch"
        | "permission_hook_unqualified_html_branch"
        | "permission_hook_unqualified_security_change"
        | "permission_hook_unqualified_security_commit"
        | "permission_hook_unqualified_security_edit"
        | "permission_hook_wrapper_changed"
        | "permission_hook_wrapper_unavailable"
        | "permission_mapping_unavailable"
        | "permission_not_inherited"
        | "permission_revision_changed"
        | "permission_session_start_sdk_dependency_unattested"
        | "permission_source_unavailable"
        | "settings_revision_changed" => error,
        _ => "unknown",
    }
}

struct ProbePermissionDiagnostics<'a> {
    correlation_id: &'a str,
    admission_id: &'a str,
    emitted: std::cell::Cell<usize>,
    protected_candidates: std::cell::RefCell<BTreeSet<String>>,
}
impl<'a> ProbePermissionDiagnostics<'a> {
    fn new(correlation_id: &'a str, admission_id: &'a str) -> Self {
        Self {
            correlation_id,
            admission_id,
            emitted: std::cell::Cell::new(0),
            protected_candidates: std::cell::RefCell::new(BTreeSet::new()),
        }
    }
    fn refusal(
        &self,
        candidate_id: &str,
        reason: CandidateProbeRefusal,
        negotiated: bool,
    ) -> CandidateProbeRefusal {
        if negotiated
            && reason == CandidateProbeRefusal::PermissionsChanged
            && self.protected_candidates.borrow().contains(candidate_id)
        {
            CandidateProbeRefusal::PermissionPathProtected
        } else {
            reason
        }
    }
    fn note(&self, candidate_id: &str, site: ProbePermissionSite, error: &str) -> Option<String> {
        self.note_with_witness(candidate_id, site, error, None)
    }
    fn note_with_witness(
        &self,
        candidate_id: &str,
        site: ProbePermissionSite,
        error: &str,
        detail: Option<(
            Option<crate::native_permissions::ProjectionGuard>,
            &crate::native_permissions::ProjectionWitnessSummary171,
        )>,
    ) -> Option<String> {
        if self.correlation_id.len() != 36
            || uuid::Uuid::parse_str(self.correlation_id).is_err()
            || self.admission_id.len() != 34
            || !self.admission_id.starts_with("a-")
            || !self.admission_id[2..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || !bridget_transport::protocol::is_routing_candidate_id(candidate_id)
        {
            return None;
        }
        // Exact closed codes only, never an error message or a prefix match.
        let code = permission_diagnostic_code(error);
        if code == "machine_path_protected" {
            self.protected_candidates
                .borrow_mut()
                .insert(candidate_id.to_owned());
        }
        if self.emitted.get() >= MAX_PROBE_PERMISSION_NOTES {
            return None;
        }
        let mut value = serde_json::json!({"event":"worker_probe_permission_refused",
            "correlation_id":self.correlation_id,"admission_id":self.admission_id,
            "candidate_id":candidate_id,"site":site.as_str(),"code":code});
        if let Some((guard, summary)) = detail {
            value["reason"] = serde_json::json!(guard.map(|g| g.as_str()).unwrap_or("unknown"));
            value["witness"] = summary.json();
        }
        let note = value.to_string();
        if note.len() > MAX_PROBE_PERMISSION_NOTE_BYTES {
            return None;
        }
        self.emitted.set(self.emitted.get() + 1);
        Some(note)
    }
}

fn map_probe_permission_failure(
    error: &str,
    diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    site: ProbePermissionSite,
    emit: impl FnOnce(&str),
) -> WorkerRefusal {
    if let Some((batch, candidate_id)) = diagnostics
        && let Some(note) = batch.note(candidate_id, site, error)
    {
        emit(&note);
    }
    WorkerRefusal::PermissionsChanged
}
fn observe_probe_permission_failure(
    error: &str,
    diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    site: ProbePermissionSite,
) -> WorkerRefusal {
    map_probe_permission_failure(error, diagnostics, site, |note| {
        eprintln!("bridget_probe_permission_refusal {note}")
    })
}

// The internal reason augments the existing note; the external refusal is unchanged.
fn map_probe_projection_failure171(
    error: &str,
    diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    guard: Option<crate::native_permissions::ProjectionGuard>,
    parent_policy: &serde_json::Value,
    emit: impl FnOnce(&str),
) -> WorkerRefusal {
    if error != "permission_hook_equivalence_unavailable" {
        return map_probe_permission_failure(
            error,
            diagnostics,
            ProbePermissionSite::Projection,
            emit,
        );
    }
    if let Some((batch, candidate_id)) = diagnostics {
        let summary = crate::native_permissions::projection_witness_summary171(parent_policy);
        if let Some(note) = batch.note_with_witness(
            candidate_id,
            ProbePermissionSite::Projection,
            error,
            Some((guard, &summary)),
        ) {
            emit(&note);
        }
    }
    WorkerRefusal::PermissionsChanged
}

#[derive(Debug)]
pub enum WorkerStoreError {
    Io(io::Error),
    Invalid(WorkerRefusal),
    Locked,
    Quarantined,
    Conflict,
    Quota,
    Unknown,
    InvalidTransition,
    Workspace(WorkerRefusal),
}

impl From<io::Error> for WorkerStoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<WorkerRefusal> for WorkerStoreError {
    fn from(value: WorkerRefusal) -> Self {
        Self::Invalid(value)
    }
}

impl std::fmt::Display for WorkerStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(_) => f.write_str("worker_store_io"),
            Self::Invalid(_) => f.write_str("worker_store_invalid"),
            Self::Locked => f.write_str("worker_store_locked"),
            Self::Quarantined => f.write_str("worker_store_quarantined"),
            Self::Conflict => f.write_str("worker_store_conflict"),
            Self::Quota => f.write_str("worker_store_quota"),
            Self::Unknown => f.write_str("worker_store_unknown"),
            Self::InvalidTransition => f.write_str("worker_store_transition"),
            Self::Workspace(_) => f.write_str("worker_workspace_failure"),
        }
    }
}
impl std::error::Error for WorkerStoreError {}

/// Sélection168 opaque conservée dans le reçu privé pour les rechecks frais.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RoutingPreparation {
    connection_id: String,
    engine: String,
    workload: bridget_transport::protocol::AutoDelegateWorkload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkerCommandReceipt {
    pub version: u16,
    pub worker: WorkerIdentity,
    pub identity: WorkerCommandIdentity,
    pub command_digest: String,
    /// Empreinte fermée du fence pré-Start, lue depuis la preuve annexe198.
    /// Jamais sérialisée : le reçu reste strictement au format183.
    #[serde(skip)]
    pub pre_start_fence: Option<String>,
    pub placement: RemotePlacement,
    pub deadline_at: i64,
    pub phase: WorkerCommandPhase,
    pub marker: Option<ManagedMarker>,
    pub permit: Option<WorkerReleasePermit>,
    pub release_epoch: Option<String>,
    #[serde(default)]
    pub child_id: Option<String>,
    #[serde(default)]
    pub native_ready: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_repair: Option<bridget_transport::protocol::WorkspaceRepairStatusV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspace_repair_history: Vec<bridget_transport::protocol::WorkspaceRepairStatusV1>,
    #[serde(default)]
    pub startup_failed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_failure: Option<bridget_transport::protocol::WorkerStartupFailure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_exit_code: Option<i32>,
    /// Canon préparé conservé avant bootstrap pour les rechecks après reprise.
    #[serde(default)]
    preparation: Option<(WorkerPrepare, WorkerPrepared)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    routing: Option<RoutingPreparation>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl WorkerCommandReceipt {
    fn effective_placement(&self) -> &RemotePlacement {
        self.workspace_repair
            .as_ref()
            .filter(|v| v.phase == "published")
            .and_then(|v| v.prepared.as_ref())
            .map(|v| &v.placement)
            .unwrap_or(&self.placement)
    }
}

/// Attestation compacte : les digests lient les identités complètes sans secrets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerDrainProof {
    version: u16,
    worker_digest: String,
    command_digest: String,
    marker_digest: String,
    pgid: u32,
    birth: u64,
}

impl WorkerDrainProof {
    fn expected(receipt: &WorkerCommandReceipt) -> Result<Self, WorkerStoreError> {
        let marker = receipt.marker.as_ref().ok_or(WorkerStoreError::Unknown)?;
        Ok(Self {
            version: STORE_VERSION,
            worker_digest: json_digest(&receipt.worker)?,
            command_digest: json_digest(&receipt.identity)?,
            marker_digest: json_digest(marker)?,
            pgid: marker.pgid,
            birth: marker.birth,
        })
    }
}

fn command_key(identity: &WorkerCommandIdentity) -> (&str, &str, &str, u64) {
    (
        &identity.command_id,
        &identity.task_id,
        &identity.instance_id,
        identity.generation,
    )
}

fn drain_proof_path(root: &Path, command: &WorkerCommandIdentity) -> PathBuf {
    root.join("markers")
        .join(format!("{}.drained", marker_key(command)))
}

/// O(c+p), c ≤ 64 groupes de contrôle du sous-arbre, p processus listés.
/// Vrai seulement si tout le sous-arbre cgroup v2 du worker ne contient que
/// lui-même. Un processus ne quitte pas ce sous-arbre sans écrire dans un
/// autre cgroup.procs ; les fournisseurs lancés par Bridget ne le font pas.
#[cfg(target_os = "linux")]
fn service_cgroup_holds_only_self() -> bool {
    let Ok(text) = fs::read_to_string("/proc/self/cgroup") else {
        return false;
    };
    let Some(path) = text.lines().find_map(|line| line.strip_prefix("0::")) else {
        return false;
    };
    // La racine ou un cgroup partagé (session, scope) n'est pas le service.
    if !path.ends_with(".service") {
        return false;
    }
    let me = std::process::id().to_string();
    let mut pending = vec![Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/'))];
    let mut visited = 0;
    while let Some(dir) = pending.pop() {
        visited += 1;
        if visited > 64 {
            return false;
        }
        let Ok(procs) = fs::read_to_string(dir.join("cgroup.procs")) else {
            return false;
        };
        if procs.lines().any(|pid| pid.trim() != me) {
            return false;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            return false;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                pending.push(entry.path());
            }
        }
    }
    visited > 0
}

#[cfg(not(target_os = "linux"))]
fn service_cgroup_holds_only_self() -> bool {
    false
}

/// Lecture seule par le wrapper, avant fournisseur. Le bootstrap fait exec ;
/// vérifier néanmoins le marqueur exact au lieu de supposer PID == PGID.
#[cfg(target_os = "linux")]
pub(crate) fn prepare_drain_proof(
    root: &Path,
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
) -> Result<WorkerDrainProof, WorkerStoreError> {
    check_private(root, true)?;
    let path = root
        .join("commands")
        .join(format!("{}.json", marker_key(command)));
    let receipt: WorkerCommandReceipt =
        read_optional(&path, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Unknown)?;
    let marker: ManagedMarker = read_optional(
        &root
            .join("markers")
            .join(format!("{}.json", marker_key(command))),
        MAX_METADATA_BYTES,
    )?
    .ok_or(WorkerStoreError::Unknown)?;
    if receipt.worker != *worker
        || receipt.identity != *command
        || receipt.marker.as_ref() != Some(&marker)
        || marker.instance_id != command.instance_id
        || marker.command_id != command.command_id
        || marker.generation != command.generation
        || marker.pgid != std::process::id()
        || crate::managed_process::process_birth(marker.pgid)? != marker.birth
        || receipt.release_epoch.is_none()
        || receipt.permit.is_none()
    {
        return Err(WorkerStoreError::Conflict);
    }
    WorkerDrainProof::expected(&receipt)
}

#[cfg(target_os = "linux")]
pub(crate) fn publish_drain_proof(
    root: &Path,
    command: &WorkerCommandIdentity,
    proof: &WorkerDrainProof,
) -> Result<(), WorkerStoreError> {
    let path = drain_proof_path(root, command);
    if let Some(existing) = read_optional::<WorkerDrainProof>(&path, MAX_DRAIN_PROOF_BYTES)? {
        return if existing == *proof {
            Ok(())
        } else {
            Err(WorkerStoreError::Conflict)
        };
    }
    let bytes = serde_json::to_vec(proof).map_err(|_| WorkerStoreError::Quarantined)?;
    if bytes.len() > MAX_DRAIN_PROOF_BYTES {
        return Err(WorkerStoreError::Quota);
    }
    check_private(path.parent().ok_or(WorkerStoreError::Unknown)?, true)?;
    write_private_file_atomic(&path, &bytes)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandReservation {
    Created(WorkerCommandReceipt),
    /// Même un reçu Reserved ne permet jamais une seconde création.
    Existing(WorkerCommandReceipt),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SlotReceipt {
    version: u16,
    worker: WorkerIdentity,
    slot: WorkerResultSlot,
    disposition: Option<WorkerResultDisposition>,
    producer_terminal: bool,
    closed: Option<WorkerSlotClosureProof>,
    finalized_at: Option<i64>,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkerResultReservation {
    pub worker: WorkerIdentity,
    pub slot: WorkerResultSlot,
    pub slot_id: String,
    pub blob_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Enrollment {
    version: u16,
    worker: WorkerIdentity,
    quarantined: bool,
}

/// Trace technique de la création154, pas une tâche ni une seconde Fleet.
/// record absent après crash signifie issue inconnue, jamais autorisation de créer.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspacePreparationReceipt {
    version: u16,
    worker: WorkerIdentity,
    command: WorkerCommandIdentity,
    prepare_digest: String,
    /// Lien198 lu depuis la preuve annexe ; jamais écrit dans ce reçu format183.
    #[serde(skip)]
    fenced_previous: Option<WorkerCommandIdentity>,
    record: Option<crate::delegation::WorkspaceRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commit_view: Option<ViewReservation>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewReservation {
    key: String,
    intent_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    planning: Option<ViewPlanning>,
    reserved_bytes: u64,
    #[serde(default)]
    source_marker: Option<ManagedMarker>,
    #[serde(default)]
    source_stopped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_failure: Option<crate::workspace_git::ExportFailure>,
    #[serde(default)]
    source_attempted: bool,
    /// Monotone : la clôture native interdit tout nouveau démarrage ou export.
    /// Elle permet la collecte de l'histoire, jamais le retrait des données.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    closed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    materialization: Option<ViewMaterialization>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewPlanning {
    seed_digest: String,
    adopted: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewMaterialization {
    execution: WorkerResultIdentity,
    request: crate::workspace_git::ViewJobRequest,
    marker: Option<ManagedMarker>,
    #[serde(default)]
    attempted: bool,
    stopped: bool,
    /// true exige le contrôle exact du fichier ; false ne prouve pas son absence.
    available: Option<bool>,
    failure: Option<crate::workspace_git::ExportFailure>,
}
impl ViewReservation {
    fn validate(&self, receipt: &WorkspacePreparationReceipt) -> Result<(), WorkerStoreError> {
        let binding = crate::workspace_git::ViewBinding {
            worker: receipt.worker.clone(),
            command: receipt.command.clone(),
            prepare_digest: receipt.prepare_digest.clone(),
        };
        if binding.key().ok().as_deref() != Some(self.key.as_str())
            || !valid_digest(&self.intent_digest)
            || self.reserved_bytes < 5120
            || self.reserved_bytes > crate::workspace_git::MAX_VIEW_RESERVATION_BYTES
            || self.planning.as_ref().is_some_and(|plan| {
                !valid_digest(&plan.seed_digest)
                    || (!plan.adopted
                        && (receipt.record.is_some() || self.materialization.is_some()))
            })
        {
            return Err(WorkerStoreError::Quarantined);
        }
        if self.closed
            && (!self.source_stopped
                || (receipt.record.is_some()
                    && (self.source_marker.is_none()
                        || receipt.record.as_ref().is_some_and(|record| {
                            record.state != crate::delegation::WorkspaceState::Kept
                        })))
                || (receipt.record.is_none()
                    && (self.materialization.is_some()
                        || (self.source_attempted
                            && self.source_marker.is_none()
                            && self.source_failure.is_none())))
                || self
                    .materialization
                    .as_ref()
                    .is_some_and(|operation| !operation.stopped))
        {
            return Err(WorkerStoreError::Quarantined);
        }
        if self.source_failure.is_some() && !self.source_stopped {
            return Err(WorkerStoreError::Quarantined);
        }
        if self.source_marker.is_some() && !self.source_attempted {
            return Err(WorkerStoreError::Quarantined);
        }
        if self.source_marker.as_ref().is_some_and(|marker| {
            marker.instance_id != receipt.command.task_id
                || marker.command_id != receipt.command.command_id
                || marker.generation != 1
                || marker.pgid == 0
                || marker.birth == 0
        }) {
            return Err(WorkerStoreError::Quarantined);
        }
        if let Some(operation) = &self.materialization {
            operation.execution.validate()?;
            let crate::workspace_git::ViewJobRequest::Materialize { command, .. } =
                &operation.request
            else {
                return Err(WorkerStoreError::Quarantined);
            };
            let expected = operation
                .request
                .identity_for_binding(&binding)
                .map_err(|_| WorkerStoreError::Quarantined)?;
            if !self.source_stopped
                || self.source_marker.is_none()
                || (operation.marker.is_some() && !operation.attempted)
                || command != &operation.execution.command
                || operation.marker.as_ref().is_some_and(|marker| {
                    marker.instance_id != expected.instance_id
                        || marker.command_id != expected.command_id
                        || marker.generation != expected.generation
                        || marker.pgid == 0
                        || marker.birth == 0
                })
                || (!operation.stopped
                    && (operation.available.is_some() || operation.failure.is_some()))
                || (operation.available == Some(true) && operation.failure.is_some())
            {
                return Err(WorkerStoreError::Quarantined);
            }
        }
        Ok(())
    }
}

/// Canon de la seule preuve de préparation fencée : un nouveau tour
/// `provider_turn` renouvelle run, session et révision sans changer les droits.
/// Driver, cwd, politique, source, version et instance restent stricts ;
/// `native_wrapper` n'est jamais normalisé. Les droits réels sont recalculés
/// avec le parent courant avant toute préparation.
fn fenced_prepare_canon(request: &WorkerPrepare) -> WorkerPrepare {
    let mut canon = request.clone();
    if canon.parent.source == "provider_turn" {
        canon.parent.run_id.clear();
        canon.parent.provider_session_id.clear();
        canon.parent.revision = 0;
    }
    canon
}

fn fenced_prepare_digest(
    command: &WorkerCommandIdentity,
    request: &WorkerPrepare,
) -> Result<String, WorkerStoreError> {
    json_digest(&(command, fenced_prepare_canon(request)))
}

pub(crate) fn same_workspace_anchor(
    record: &crate::delegation::WorkspaceRecord,
    workspace: &serde_json::Value,
) -> bool {
    let Ok(sent) = serde_json::from_value::<crate::delegation::WorkspaceRecord>(workspace.clone())
    else {
        return false;
    };
    matches!(
        (&record.strategy, &sent.strategy),
        (
            crate::delegation::WorkspaceStrategy::Worktree,
            crate::delegation::WorkspaceStrategy::Worktree
        ) | (
            crate::delegation::WorkspaceStrategy::CommitView,
            crate::delegation::WorkspaceStrategy::CommitView
        )
    ) && record.path == sent.path
        && record.toplevel == sent.toplevel
        && record.branch == sent.branch
        && record.base_commit == sent.base_commit
        && record.commit_view == sent.commit_view
}

fn exact_fence_at(
    root: &Path,
    worker: &WorkerIdentity,
    previous: &WorkerCommandIdentity,
    placement: &RemotePlacement,
) -> Result<(), WorkerStoreError> {
    let path = root
        .join("commands")
        .join(format!("{}.json", marker_key(previous)));
    let mut receipt: WorkerCommandReceipt =
        read_optional(&path, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Unknown)?;
    proof::overlay_fence(root, worker, &mut receipt)?;
    let digest = proof::fence_digest(worker, previous, &receipt.placement, receipt.deadline_at)?;
    if receipt.version != STORE_VERSION
        || receipt.worker != *worker
        || receipt.identity != *previous
        || receipt.placement != *placement
        || receipt.deadline_at <= 0
        || receipt.created_at < 0
        || receipt.pre_start_fence.as_deref() != Some(digest.as_str())
        || receipt.command_digest != digest
        || receipt.phase != WorkerCommandPhase::Refused
        || receipt.marker.is_some()
        || receipt.permit.is_some()
        || receipt.child_id.is_some()
        || receipt.preparation.is_some()
        || receipt.routing.is_some()
        || receipt.release_epoch.is_some()
        || receipt.native_ready
        || receipt.startup_failed
        || receipt.startup_failure.is_some()
        || receipt.terminal_exit_code.is_some()
        || receipt.workspace_repair.is_some()
        || !receipt.workspace_repair_history.is_empty()
    {
        return Err(WorkerStoreError::Conflict);
    }
    Ok(())
}

/// Les reçus de reprise ne contiennent que le lien vers le fence précédent.
/// Une chaîne de reprises se termine toujours sur la création Git originale.
fn original_workspace_at(
    root: &Path,
    worker: &WorkerIdentity,
    first: &WorkerCommandIdentity,
) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
    let mut command = first.clone();
    let mut visited = BTreeSet::new();
    for _ in 0..MAX_WORKER_STORE_ENTRIES {
        if !visited.insert(command.command_id.clone()) {
            return Err(WorkerStoreError::Conflict);
        }
        let receipt = proof::read_workspace_receipt(root, worker, &command)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt.version != STORE_VERSION
            || receipt.worker != *worker
            || receipt.command != command
            || !valid_digest(&receipt.prepare_digest)
        {
            return Err(WorkerStoreError::Conflict);
        }
        match (receipt.record, receipt.fenced_previous) {
            (Some(record), None) => return Ok(record),
            (None, Some(previous)) => {
                if previous.validate().is_err()
                    || previous.task_id != command.task_id
                    || previous.command_id == command.command_id
                {
                    return Err(WorkerStoreError::Conflict);
                }
                // Le lien a été écrit seulement après preuve du fence exact.
                // Son ancien reçu peut ensuite être collecté pendant qu'une
                // nouvelle commande utilise encore ce worktree.
                command = previous;
                command.instance_id.clear();
                command.generation = 0;
            }
            _ => return Err(WorkerStoreError::Conflict),
        }
    }
    Err(WorkerStoreError::Conflict)
}

/// Même autorité198 pour lecteur et writer : B remonte sa chaîne privée
/// jusqu'à A. O(k), k borné par le store ; aucun manifeste ni E/S Git.
fn view_anchor_receipt_at(
    root: &Path,
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
    request: Option<&WorkerPrepare>,
) -> Result<WorkspacePreparationReceipt, WorkerStoreError> {
    command.validate_preparation()?;
    let current =
        proof::read_workspace_receipt(root, worker, command)?.ok_or(WorkerStoreError::Unknown)?;
    if current.worker != *worker || current.command != *command || current.version != STORE_VERSION
    {
        return Err(WorkerStoreError::Conflict);
    }
    if let Some(request) = request {
        let expected = if current.fenced_previous.is_some() {
            fenced_prepare_digest(command, request)?
        } else {
            json_digest(&(command, request))?
        };
        if current.prepare_digest != expected {
            return Err(WorkerStoreError::Conflict);
        }
    }
    let original = original_workspace_at(root, worker, command)?;
    let reference = original
        .commit_view
        .as_ref()
        .ok_or(WorkerStoreError::Conflict)?;
    if !matches!(
        original.strategy,
        crate::delegation::WorkspaceStrategy::CommitView
    ) || reference.binding.worker != *worker
        || reference.binding.command.task_id != command.task_id
    {
        return Err(WorkerStoreError::Conflict);
    }
    let anchor = proof::read_workspace_receipt(root, worker, &reference.binding.command)?
        .ok_or(WorkerStoreError::Unknown)?;
    let view = anchor
        .commit_view
        .as_ref()
        .ok_or(WorkerStoreError::Unknown)?;
    view.validate(&anchor)?;
    if anchor.version != STORE_VERSION
        || anchor.worker != *worker
        || anchor.command != reference.binding.command
        || anchor.prepare_digest != reference.binding.prepare_digest
        || anchor.fenced_previous.is_some()
        || view.intent_digest != reference.intent_digest
        || anchor.record.as_ref().is_none_or(|record| {
            !same_workspace_anchor(
                record,
                &serde_json::to_value(&original).unwrap_or(serde_json::Value::Null),
            )
        })
    {
        return Err(WorkerStoreError::Conflict);
    }
    Ok(anchor)
}

/// Contrôle lecture seule de l'alias avant fork et dans le helper géré.
/// Une autre commande de même Task n'est pas suffisante sans BindProof198.
pub(crate) fn view_file_alias_matches(
    proof_root: &Path,
    binding: &crate::workspace_git::ViewBinding,
    request: &crate::workspace_git::ViewJobRequest,
) -> bool {
    let crate::workspace_git::ViewJobRequest::Materialize { command, .. } = request else {
        return false;
    };
    if command.validate().is_err() || command.task_id != binding.command.task_id {
        return false;
    }
    let Some(root) = proof_root.parent() else {
        return false;
    };
    if root.join("view-manifests") != proof_root || check_private(root, true).is_err() {
        return false;
    }
    let mut prepare = command.clone();
    prepare.instance_id.clear();
    prepare.generation = 0;
    let Ok(anchor) = view_anchor_receipt_at(root, &binding.worker, &prepare, None) else {
        return false;
    };
    let Some(view) = &anchor.commit_view else {
        return false;
    };
    anchor.command == binding.command
        && anchor.prepare_digest == binding.prepare_digest
        && !view.closed
        && view.source_stopped
        && view.source_failure.is_none()
        && view.materialization.as_ref().is_some_and(|operation| {
            operation.execution.command == *command
                && operation.request == *request
                && !operation.stopped
        })
        && anchor
            .record
            .as_ref()
            .and_then(|record| record.commit_view.as_ref())
            .is_some_and(|reference| Path::new(&reference.proof_root) == proof_root)
}

pub struct WorkerStore {
    root: PathBuf,
    enrollment: PathBuf,
    worker: WorkerIdentity,
    _lock: WorkerFileLock,
    transition: Mutex<()>,
}

impl WorkerStore {
    /// Le parent est provisionné privé ; ne jamais chmod un parent partagé.
    /// Enrollment et verrou restent hors de root pour détecter sa disparition.
    pub fn open(root: &Path, worker: WorkerIdentity) -> Result<Self, WorkerStoreError> {
        worker.validate()?;
        if !root.is_absolute() {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope));
        }
        let parent = root.parent().ok_or(WorkerStoreError::Unknown)?;
        check_private(parent, true)?;
        let key = digest_bytes(root.as_os_str().as_encoded_bytes());
        let enrollment = parent.join(format!(".worker-{key}.enrolled"));
        let lock = lock_file(&parent.join(format!(".worker-{key}.lock")), true)?;
        let enrolled: Option<Enrollment> = read_optional(&enrollment, MAX_METADATA_BYTES)?;
        let present = fs::symlink_metadata(root).is_ok();
        match enrolled {
            None if !present => {
                write_json(
                    &enrollment,
                    &Enrollment {
                        version: STORE_VERSION,
                        worker: worker.clone(),
                        quarantined: false,
                    },
                )?;
                private_dir(root)?;
                private_dir(&root.join("commands"))?;
                private_dir(&root.join("slots"))?;
                private_dir(&root.join("workspaces"))?;
            }
            Some(record) if record.worker != worker => return Err(WorkerStoreError::Conflict),
            Some(record)
                if record.version == STORE_VERSION
                    && record.worker == worker
                    && !record.quarantined
                    && present => {}
            _ => {
                write_json(
                    &enrollment,
                    &Enrollment {
                        version: STORE_VERSION,
                        worker,
                        quarantined: true,
                    },
                )?;
                return Err(WorkerStoreError::Quarantined);
            }
        }
        let store = Self {
            root: root.into(),
            enrollment,
            worker,
            _lock: lock,
            transition: Mutex::new(()),
        };
        store.health()?;
        // Lit et valide les reçus au redémarrage. Aucun état ne vaut autorisation
        // de reconstruire le pipe RELEASE ou de recréer un bootstrap.
        if store.inventory().is_err() {
            store.quarantine()?;
            return Err(WorkerStoreError::Quarantined);
        }
        Ok(store)
    }

    pub fn reserve_command(
        &self,
        identity: &WorkerCommandIdentity,
        command_digest: &str,
        placement: &RemotePlacement,
        deadline_at: i64,
        now: i64,
    ) -> Result<CommandReservation, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        identity.validate()?;
        placement.validate()?;
        if !placement.matches_worker(&self.worker)
            || !valid_digest(command_digest)
            || now < 0
            || deadline_at <= now
        {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::PlacementMismatch));
        }
        if let Some(receipt) = self.read_command(identity)? {
            if receipt.pre_start_fence.is_some() {
                return Err(WorkerStoreError::Conflict);
            }
            if receipt.identity != *identity
                || receipt.command_digest != command_digest
                || receipt.placement != *placement
                || receipt.deadline_at != deadline_at
            {
                return Err(WorkerStoreError::Conflict);
            }
            return Ok(CommandReservation::Existing(receipt));
        }
        // Une preuve de fence198 sans reçu (crash) interdit encore tout Start.
        if proof::read_fence(&self.root, &self.worker, identity)?.is_some() {
            return Err(WorkerStoreError::Conflict);
        }
        self.admit(MAX_METADATA_BYTES as u64 * 2 + DRAIN_PROOF_RESERVATION, 1)?;
        let receipt = WorkerCommandReceipt {
            version: STORE_VERSION,
            worker: self.worker.clone(),
            identity: identity.clone(),
            command_digest: command_digest.into(),
            pre_start_fence: None,
            placement: placement.clone(),
            deadline_at,
            phase: WorkerCommandPhase::Reserved,
            marker: None,
            permit: None,
            release_epoch: None,
            child_id: None,
            native_ready: false,
            workspace_repair: None,
            workspace_repair_history: Vec::new(),
            startup_failed: false,
            startup_failure: None,
            terminal_exit_code: None,
            preparation: None,
            routing: None,
            created_at: now,
            updated_at: now,
        };
        self.persist(&self.command_path(identity), &receipt)?;
        Ok(CommandReservation::Created(receipt))
    }

    /// O(n) via admit/inventory ; n est borné par MAX_WORKER_STORE_ENTRIES.
    /// Crée un refus durable sans transformer un reçu Start déjà existant.
    fn fence_unstarted(
        &self,
        identity: &WorkerCommandIdentity,
        placement: &RemotePlacement,
        deadline_at: i64,
        now: i64,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        identity.validate()?;
        placement.validate()?;
        if !placement.matches_worker(&self.worker) || deadline_at <= 0 || now < 0 {
            return Err(WorkerRefusal::InvalidEnvelope.into());
        }
        let digest = proof::fence_digest(&self.worker, identity, placement, deadline_at)?;
        // La preuve est écrite avant le reçu ; seule elle reste après un crash.
        let existing = proof::read_fence(&self.root, &self.worker, identity)?;
        if existing.as_ref().is_some_and(|proof| {
            proof.digest != digest
                || proof.placement != *placement
                || proof.deadline_at != deadline_at
        }) {
            return Err(WorkerStoreError::Conflict);
        }
        if let Some(receipt) = self.read_command(identity)? {
            return if receipt.pre_start_fence.as_deref() == Some(digest.as_str())
                && receipt.command_digest == digest
                && receipt.placement == *placement
                && receipt.deadline_at == deadline_at
                && receipt.phase == WorkerCommandPhase::Refused
            {
                Ok(())
            } else if receipt.pre_start_fence.is_some() || receipt.placement != *placement {
                Err(WorkerStoreError::Conflict)
            } else {
                Err(WorkerStoreError::Unknown)
            };
        }
        match fs::symlink_metadata(
            self.root
                .join("markers")
                .join(format!("{}.json", marker_key(identity))),
        ) {
            Ok(_) => return Err(WorkerStoreError::Unknown),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        if !self.confirm_view_sources_stopped(&identity.task_id)? {
            return Err(WorkerStoreError::Unknown);
        }
        let proof = match existing {
            Some(proof) => {
                self.admit(MAX_METADATA_BYTES as u64 * 2 + DRAIN_PROOF_RESERVATION, 1)?;
                proof
            }
            None => {
                self.admit(MAX_METADATA_BYTES as u64 * 3 + DRAIN_PROOF_RESERVATION, 2)?;
                let proof = proof::FenceProof {
                    version: STORE_VERSION,
                    worker: self.worker.clone(),
                    identity: identity.clone(),
                    placement: placement.clone(),
                    deadline_at,
                    created_at: now,
                    digest: digest.clone(),
                };
                proof::ensure_dir(&self.root)?;
                self.persist(&proof::fence_path(&self.root, identity), &proof)?;
                proof
            }
        };
        let receipt = WorkerCommandReceipt {
            version: STORE_VERSION,
            worker: self.worker.clone(),
            identity: identity.clone(),
            command_digest: digest,
            pre_start_fence: None,
            placement: placement.clone(),
            deadline_at: proof::legacy_deadline(proof.deadline_at, proof.created_at),
            phase: WorkerCommandPhase::Refused,
            marker: None,
            permit: None,
            release_epoch: None,
            child_id: None,
            native_ready: false,
            workspace_repair: None,
            workspace_repair_history: Vec::new(),
            startup_failed: false,
            startup_failure: None,
            terminal_exit_code: None,
            preparation: None,
            routing: None,
            created_at: proof.created_at,
            updated_at: proof.created_at,
        };
        self.persist(&self.command_path(identity), &receipt)
    }

    pub fn command(
        &self,
        identity: &WorkerCommandIdentity,
    ) -> Result<WorkerCommandReceipt, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        self.read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)
    }

    pub fn commands(&self) -> Result<Vec<WorkerCommandReceipt>, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        Ok(self.inventory()?.0)
    }

    fn bind_launch(
        &self,
        identity: &WorkerCommandIdentity,
        child_id: &str,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        if !safe_component(child_id) {
            return Err(WorkerRefusal::InvalidEnvelope.into());
        }
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt
            .child_id
            .as_deref()
            .is_some_and(|value| value != child_id)
            || receipt.phase != WorkerCommandPhase::Reserved
        {
            return Err(WorkerStoreError::Conflict);
        }
        receipt.child_id = Some(child_id.into());
        self.persist(&self.command_path(identity), &receipt)
    }

    fn bind_preparation(
        &self,
        identity: &WorkerCommandIdentity,
        prepared: &PreparedRuntime,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        let canon = (prepared.request.clone(), prepared.prepared.clone());
        if receipt.phase != WorkerCommandPhase::Reserved
            || receipt.placement != prepared.prepared.placement
            || prepared.command.command_id != identity.command_id
            || prepared.command.task_id != identity.task_id
            || receipt
                .preparation
                .as_ref()
                .is_some_and(|previous| previous != &canon || receipt.routing != prepared.routing)
        {
            return Err(WorkerStoreError::Conflict);
        }
        receipt.preparation = Some(canon);
        receipt.routing = prepared.routing.clone();
        // Refus avant bootstrap si le canon excède la réservation de métadonnées.
        let bytes = serde_json::to_vec(&receipt).map_err(|_| WorkerStoreError::Quota)?;
        if bytes.len() > MAX_METADATA_BYTES {
            return Err(WorkerStoreError::Quota);
        }
        self.admit(bytes.len() as u64, 0)?;
        self.persist(&self.command_path(identity), &receipt)
    }

    fn record_startup_outcome(
        &self,
        identity: &WorkerCommandIdentity,
        failure: Option<bridget_transport::protocol::WorkerStartupFailure>,
        exit_code: Option<i32>,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        let before = (receipt.startup_failure, receipt.terminal_exit_code);
        if receipt.startup_failure.is_none() {
            receipt.startup_failure = failure;
        }
        if exit_code.is_some() {
            receipt.terminal_exit_code = exit_code;
        }
        if before != (receipt.startup_failure, receipt.terminal_exit_code) {
            self.persist(&self.command_path(identity), &receipt)?;
            eprintln!(
                "bridget_remote_start171 {}",
                serde_json::json!({
                    "task_id":identity.task_id,"command_id":identity.command_id,
                    "generation":identity.generation,"failure":receipt.startup_failure,
                    "exit_code":receipt.terminal_exit_code
                })
            );
        }
        Ok(())
    }

    fn native_status(
        &self,
        identity: &WorkerCommandIdentity,
        failed: bool,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        // Une erreur de RELEASE peut précéder sa confirmation durable. Seul
        // l'échec est attestable dans cet état ; jamais NativeReady ni Released.
        if receipt.marker.is_none()
            || !(receipt.phase == WorkerCommandPhase::Released
                || (failed && receipt.phase == WorkerCommandPhase::ReleaseAuthorized))
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        receipt.startup_failed |= failed;
        receipt.native_ready = !receipt.startup_failed;
        self.persist(&self.command_path(identity), &receipt)
    }

    pub fn mark_command(
        &self,
        identity: &WorkerCommandIdentity,
        marker: &ManagedMarker,
        now: i64,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        if marker.command_id != identity.command_id
            || marker.instance_id != identity.instance_id
            || marker.generation != identity.generation
            || marker.pgid == 0
            || marker.birth == 0
        {
            return Err(WorkerStoreError::Conflict);
        }
        if receipt.phase == WorkerCommandPhase::Marked && receipt.marker.as_ref() == Some(marker) {
            return Ok(());
        }
        if receipt.phase != WorkerCommandPhase::Reserved {
            return Err(WorkerStoreError::InvalidTransition);
        }
        receipt.marker = Some(marker.clone());
        receipt.phase = WorkerCommandPhase::Marked;
        receipt.updated_at = now;
        self.persist(&self.command_path(identity), &receipt)
    }

    pub fn authorize_release(
        &self,
        identity: &WorkerCommandIdentity,
        permit: &WorkerReleasePermit,
        connection_epoch: &str,
        now: i64,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        permit.validate()?;
        if connection_epoch.is_empty()
            || connection_epoch.len() > 256
            || connection_epoch.chars().any(char::is_control)
        {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::StaleConnection));
        }
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        if permit.command_digest != receipt.command_digest
            || permit.mapping_revision != receipt.placement.mapping_revision
            || permit.provisioning_revision != self.worker.provisioning_revision
            || now < permit.issued_at
            || now >= permit.expires_at
            || now >= receipt.deadline_at
        {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::ReleaseExpired));
        }
        if receipt.phase == WorkerCommandPhase::ReleaseAuthorized
            && receipt.permit.as_ref() == Some(permit)
            && receipt.release_epoch.as_deref() == Some(connection_epoch)
        {
            return Ok(());
        }
        if receipt.phase != WorkerCommandPhase::Marked {
            return Err(WorkerStoreError::InvalidTransition);
        }
        receipt.permit = Some(permit.clone());
        receipt.release_epoch = Some(connection_epoch.into());
        receipt.phase = WorkerCommandPhase::ReleaseAuthorized;
        receipt.updated_at = now;
        self.persist(&self.command_path(identity), &receipt)
    }

    /// Le runtime doit fournir la preuve de processus avant Terminal/Refused.
    /// Unknown n'autorise aucun retour à Reserved, Marked ou Released.
    pub fn set_phase(
        &self,
        identity: &WorkerCommandIdentity,
        phase: WorkerCommandPhase,
        now: i64,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self
            .read_command(identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt.phase == phase {
            return Ok(());
        }
        if matches!(
            receipt.phase,
            WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
        ) || !(matches!(
            phase,
            WorkerCommandPhase::Terminal
                | WorkerCommandPhase::Refused
                | WorkerCommandPhase::Unknown
        ) || receipt.phase == WorkerCommandPhase::ReleaseAuthorized
            && phase == WorkerCommandPhase::Released)
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        receipt.phase = phase;
        receipt.updated_at = now;
        self.persist(&self.command_path(identity), &receipt)
    }

    pub fn reserve_result_slot(
        &self,
        slot: &WorkerResultSlot,
        now: i64,
    ) -> Result<WorkerResultReservation, WorkerStoreError> {
        self.reserve_result_slot_before(slot, now, None)
    }

    pub fn reserve_result_slot_before(
        &self,
        slot: &WorkerResultSlot,
        now: i64,
        expires_at_ms: Option<i64>,
    ) -> Result<WorkerResultReservation, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        if expires_at_ms.is_some_and(|deadline| worker_now_millis() >= deadline) {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::ResultSlotExpired));
        }
        slot.validate()?;
        let command = self
            .read_command(&slot.identity.command)?
            .ok_or(WorkerStoreError::Unknown)?;
        if let Some(existing) = self.read_slot(&slot.identity)? {
            if existing.slot != *slot {
                return Err(WorkerStoreError::Conflict);
            }
            if existing.closed.is_some() {
                return Err(WorkerStoreError::InvalidTransition);
            }
            return self.reservation(slot);
        }
        if matches!(
            command.phase,
            WorkerCommandPhase::Terminal
                | WorkerCommandPhase::Refused
                | WorkerCommandPhase::Unknown
        ) || now < 0
            || (command.phase != WorkerCommandPhase::Released && now >= command.deadline_at)
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        self.admit(
            MAX_WORKER_RESULT_ENVELOPE_BYTES as u64 + 2 * MAX_METADATA_BYTES as u64,
            1,
        )?;
        if expires_at_ms.is_some_and(|deadline| worker_now_millis() >= deadline) {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::ResultSlotExpired));
        }
        let directory = self.slot_dir(&slot.identity)?;
        // Le reçu est d'abord durable ; une création partielle reste inconnue,
        // jamais une nouvelle admission au prochain démarrage.
        private_dir(&directory)?;
        let receipt = SlotReceipt {
            version: STORE_VERSION,
            worker: self.worker.clone(),
            slot: slot.clone(),
            disposition: None,
            producer_terminal: false,
            closed: None,
            finalized_at: None,
            created_at: now,
        };
        self.persist(&directory.join("receipt.json"), &receipt)?;
        private_dir(&directory.join("payload"))?;
        drop(lock_file(&directory.join("payload/publish.lock"), true)?);
        self.reservation(slot)
    }

    pub fn read_result(
        &self,
        identity: &WorkerResultIdentity,
    ) -> Result<Option<RemoteResultEnvelope>, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let receipt = self.read_slot(identity)?.ok_or(WorkerStoreError::Unknown)?;
        self.read_blob(&receipt)
    }

    pub fn record_disposition(
        &self,
        result: &WorkerResultDisposition,
        now: i64,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        result.validate()?;
        let mut receipt = self
            .read_slot(&result.identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt
            .disposition
            .as_ref()
            .is_some_and(|old| old.disposition.is_final())
        {
            return if receipt.disposition.as_ref() == Some(result) {
                Ok(())
            } else {
                Err(WorkerStoreError::Conflict)
            };
        }
        let blob = self.read_blob(&receipt)?.ok_or(WorkerStoreError::Unknown)?;
        if blob.digest != result.result_digest {
            return Err(WorkerStoreError::Conflict);
        }
        if result.disposition.is_final() {
            receipt.finalized_at = Some(now);
        }
        receipt.disposition = Some(result.clone());
        self.persist(
            &self.slot_dir(&result.identity)?.join("receipt.json"),
            &receipt,
        )
    }

    /// Relais borné après reconnexion ou mort du wrapper. Aucun lancement ni
    /// décision métier ; les reports demandés par le maître restent respectés.
    pub fn pending_results(&self, now: i64) -> Result<Vec<RemoteResultEnvelope>, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let (_, slots, _, _) = self.inventory()?;
        let mut results = Vec::new();
        for slot in slots {
            if slot.disposition.as_ref().is_some_and(|record| {
                record.disposition.is_final()
                    || matches!(record.disposition, bridget_transport::protocol::RemoteResultDisposition::Deferred { retry_after, .. } if now < retry_after)
            }) { continue; }
            if let Some(result) = self.read_blob(&slot)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    /// Attestation fournie par le runtime après observation terminale du
    /// producteur exact, jamais à partir d'une simple déconnexion.
    pub fn mark_result_producer_terminal(
        &self,
        identity: &WorkerResultIdentity,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self.read_slot(identity)?.ok_or(WorkerStoreError::Unknown)?;
        receipt.producer_terminal = true;
        self.persist(&self.slot_dir(identity)?.join("receipt.json"), &receipt)
    }

    pub fn close_result_slot(
        &self,
        slot: &WorkerResultSlot,
        proof: WorkerSlotClosureProof,
        now: i64,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let _publication = lock_file(
            &self.slot_dir(&slot.identity)?.join("payload/publish.lock"),
            false,
        )?;
        let mut receipt = self
            .read_slot(&slot.identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt.slot != *slot {
            return Err(WorkerStoreError::Conflict);
        }
        if receipt.closed.is_some() && receipt.closed != Some(proof) {
            return Err(WorkerStoreError::Conflict);
        }
        if proof == WorkerSlotClosureProof::NeverDispatched {
            if self.read_blob(&receipt)?.is_some() {
                return Err(WorkerStoreError::Conflict);
            }
            receipt.producer_terminal = true;
            receipt.finalized_at = Some(now);
        }
        if receipt.disposition.is_none() && self.read_blob(&receipt)?.is_none() {
            receipt.finalized_at = Some(now);
        }
        receipt.closed = Some(proof);
        self.persist(
            &self.slot_dir(&slot.identity)?.join("receipt.json"),
            &receipt,
        )
    }

    /// Collecte des blobs seulement sous trois preuves. Les reçus compacts
    /// sont conservés sept jours ; aucune collecte d'un résultat différé.
    pub fn collect(&self, now: i64) -> Result<usize, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let (commands, slots, _, _) = self.inventory()?;
        // Les commandes et slots d'une vue conservée restent nécessaires au
        // retrait. Ne pas perdre sa preuve de clôture après sept jours.
        let retained_views = self
            .view_receipts(None)?
            .into_iter()
            .filter(|receipt| receipt.commit_view.as_ref().is_none_or(|view| !view.closed))
            .map(|receipt| receipt.command.task_id)
            .collect::<HashSet<_>>();
        let terminal_commands: HashSet<_> = commands
            .iter()
            .filter(|c| {
                matches!(
                    c.phase,
                    WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                )
            })
            .map(|c| command_key(&c.identity))
            .collect();
        let mut removed = 0;
        for slot in slots {
            let command_terminal =
                terminal_commands.contains(&command_key(&slot.slot.identity.command));
            let final_disposition = slot
                .disposition
                .as_ref()
                .is_some_and(|d| d.disposition.is_final());
            let never_dispatched = slot.closed == Some(WorkerSlotClosureProof::NeverDispatched);
            let closed_without_result = slot.closed.is_some()
                && slot.disposition.is_none()
                && self.read_blob(&slot)?.is_none();
            if !slot.producer_terminal
                || !(command_terminal || slot.closed.is_some())
                || !(final_disposition || never_dispatched || closed_without_result)
            {
                continue;
            }
            let directory = self.slot_dir(&slot.slot.identity)?;
            let payload = directory.join("payload");
            let _publication = lock_file(&payload.join("publish.lock"), false)?;
            let blob = payload.join("blob.json");
            if fs::symlink_metadata(&blob).is_ok() {
                self.read_blob(&slot)?;
                fs::remove_file(&blob)?;
                File::open(&payload)?.sync_all()?;
                removed += 1;
            }
            // Garder le compact au-delà de sept jours en présence d'orphelins,
            // au lieu d'effacer une preuve ambiguë ou de déborder une réservation.
            if slot
                .finalized_at
                .is_some_and(|at| now.saturating_sub(at) >= WORKER_RECEIPT_RETENTION_SECS)
                && !retained_views.contains(&slot.slot.identity.command.task_id)
                && fs::read_dir(&payload)?.count() == 1
            {
                fs::remove_file(payload.join("publish.lock"))?;
                fs::remove_dir(&payload)?;
                fs::remove_file(directory.join("receipt.json"))?;
                fs::remove_dir(&directory)?;
                File::open(self.root.join("slots"))?.sync_all()?;
            }
        }
        // Aucun ancien canon non expiré ne perd sa preuve d'admission.
        let (_, remaining_slots, _, _) = self.inventory()?;
        let referenced: HashSet<_> = remaining_slots
            .iter()
            .map(|slot| command_key(&slot.slot.identity.command))
            .collect();
        for command in commands {
            if matches!(
                command.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) && !retained_views.contains(&command.identity.task_id)
                && now.saturating_sub(command.deadline_at.max(command.updated_at))
                    >= WORKER_RECEIPT_RETENTION_SECS
                && !referenced.contains(&command_key(&command.identity))
            {
                let proof = drain_proof_path(&self.root, &command.identity);
                match fs::remove_file(&proof) {
                    Ok(()) => File::open(self.root.join("markers"))?.sync_all()?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                workspace_repair::collect_markers(&self.root, &command)?;
                // Preuve annexe d'abord : un crash laisse au pire un refus ordinaire.
                if command.pre_start_fence.is_some() {
                    fs::remove_file(proof::fence_path(&self.root, &command.identity))?;
                    File::open(self.root.join("start198"))?.sync_all()?;
                }
                fs::remove_file(self.command_path(&command.identity))?;
                File::open(self.root.join("commands"))?.sync_all()?;
            }
        }
        Ok(removed)
    }

    fn drainage_proven(&self, receipt: &WorkerCommandReceipt) -> Result<bool, WorkerStoreError> {
        // Sur les autres hôtes ce correctif ne change pas le superviseur local.
        if !cfg!(target_os = "linux") {
            return Ok(true);
        }
        // Pas de RELEASE possible : la récolte du bootstrap demeure suffisante.
        if receipt.permit.is_none()
            && receipt.release_epoch.is_none()
            && !receipt.native_ready
            && !receipt.startup_failed
            && matches!(
                receipt.phase,
                WorkerCommandPhase::Reserved
                    | WorkerCommandPhase::Marked
                    | WorkerCommandPhase::Unknown
            )
        {
            return Ok(true);
        }
        let expected = WorkerDrainProof::expected(receipt)?;
        Ok(read_optional::<WorkerDrainProof>(
            &drain_proof_path(&self.root, &receipt.identity),
            MAX_DRAIN_PROOF_BYTES,
        )?
        .as_ref()
            == Some(&expected))
    }

    fn command_path(&self, identity: &WorkerCommandIdentity) -> PathBuf {
        self.root.join("commands").join(format!(
            "{}.json",
            digest_bytes(identity.command_id.as_bytes())
        ))
    }

    fn slot_dir(&self, identity: &WorkerResultIdentity) -> Result<PathBuf, WorkerStoreError> {
        Ok(self.root.join("slots").join(json_digest(identity)?))
    }

    fn reservation(
        &self,
        slot: &WorkerResultSlot,
    ) -> Result<WorkerResultReservation, WorkerStoreError> {
        Ok(WorkerResultReservation {
            worker: self.worker.clone(),
            slot: slot.clone(),
            slot_id: json_digest(&slot.identity)?,
            blob_path: self.slot_dir(&slot.identity)?.join("payload/blob.json"),
        })
    }

    fn health(&self) -> Result<(), WorkerStoreError> {
        let record: Enrollment = read_optional(&self.enrollment, MAX_METADATA_BYTES)?
            .ok_or(WorkerStoreError::Quarantined)?;
        if record.version != STORE_VERSION || record.worker != self.worker || record.quarantined {
            return Err(WorkerStoreError::Quarantined);
        }
        for directory in [
            &self.root,
            &self.root.join("commands"),
            &self.root.join("slots"),
            &self.root.join("workspaces"),
        ] {
            if check_private(directory, true).is_err() {
                self.quarantine()?;
                return Err(WorkerStoreError::Quarantined);
            }
        }
        Ok(())
    }

    fn quarantine(&self) -> Result<(), WorkerStoreError> {
        write_json(
            &self.enrollment,
            &Enrollment {
                version: STORE_VERSION,
                worker: self.worker.clone(),
                quarantined: true,
            },
        )
    }

    fn read_command(
        &self,
        identity: &WorkerCommandIdentity,
    ) -> Result<Option<WorkerCommandReceipt>, WorkerStoreError> {
        read::read_worker_command(&self.root, &self.worker, identity)
    }

    fn read_slot(
        &self,
        identity: &WorkerResultIdentity,
    ) -> Result<Option<SlotReceipt>, WorkerStoreError> {
        identity.validate()?;
        let directory = self.slot_dir(identity)?;
        let result: Option<SlotReceipt> =
            read_optional(&directory.join("receipt.json"), MAX_METADATA_BYTES)?;
        if let Some(receipt) = &result {
            if receipt.worker != self.worker
                || receipt.version != STORE_VERSION
                || receipt.slot.identity != *identity
            {
                return Err(WorkerStoreError::Conflict);
            }
            receipt.slot.validate()?;
            if let Some(disposition) = &receipt.disposition {
                disposition.validate()?;
                if disposition.identity != receipt.slot.identity
                    || disposition.disposition.is_final() != receipt.finalized_at.is_some()
                {
                    return Err(WorkerStoreError::Quarantined);
                }
            }
        } else if fs::symlink_metadata(directory).is_ok() {
            return Err(WorkerStoreError::Quarantined);
        }
        Ok(result)
    }

    fn read_blob(
        &self,
        slot: &SlotReceipt,
    ) -> Result<Option<RemoteResultEnvelope>, WorkerStoreError> {
        if fs::symlink_metadata(self.slot_dir(&slot.slot.identity)?.join("payload/conflict"))
            .is_ok()
        {
            self.quarantine()?;
            return Err(WorkerStoreError::Quarantined);
        }
        let blob: Option<RemoteResultEnvelope> = read_optional(
            &self
                .slot_dir(&slot.slot.identity)?
                .join("payload/blob.json"),
            MAX_WORKER_RESULT_ENVELOPE_BYTES,
        )?;
        if let Some(blob) = &blob {
            blob.validate()?;
            if blob.worker != self.worker || blob.identity != slot.slot.identity {
                return Err(WorkerStoreError::Conflict);
            }
        }
        Ok(blob)
    }

    fn persist<T: Serialize>(&self, path: &Path, value: &T) -> Result<(), WorkerStoreError> {
        match write_json(path, value) {
            Ok(()) => Ok(()),
            Err(error) => {
                let _ = self.quarantine();
                Err(error)
            }
        }
    }

    fn admit(&self, additional: u64, entries: usize) -> Result<(), WorkerStoreError> {
        let (_, _, used, count) = self.inventory()?;
        if count.saturating_add(entries) > MAX_WORKER_STORE_ENTRIES
            || used.saturating_add(additional) > MAX_WORKER_STORE_BYTES as u64
        {
            return Err(WorkerStoreError::Quota);
        }
        Ok(())
    }

    /// Chaque payload actif réserve 2 Mio, temporaire de publication inclus.
    /// Les reçus en cours gardent 64 Kio pour leur croissance ; les reçus
    /// soldés comptent leurs octets réels. Deux remplacements de métadonnées
    /// sont réservés globalement, car les transitions sont sérialisées.
    /// Les fichiers orphelins comptent leurs octets, jamais zéro.
    fn inventory(
        &self,
    ) -> Result<(Vec<WorkerCommandReceipt>, Vec<SlotReceipt>, u64, usize), WorkerStoreError> {
        let mut commands = Vec::new();
        let mut slots = Vec::new();
        let mut used =
            private_tree_bytes(&self.root)?.saturating_add(2 * MAX_METADATA_BYTES as u64);
        for entry in fs::read_dir(self.root.join("commands"))? {
            let path = entry?.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let command: WorkerCommandReceipt =
                read_optional(&path, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Quarantined)?;
            if self.command_path(&command.identity) != path {
                return Err(WorkerStoreError::Quarantined);
            }
            let command = self
                .read_command(&command.identity)?
                .ok_or(WorkerStoreError::Quarantined)?;
            // Terminal/Refused ne peut plus recevoir de préparation ni de permis.
            // Les 4 Kio gardent la preuve de drainage et les détails de sortie tardifs.
            used = used.saturating_add(DRAIN_PROOF_RESERVATION);
            if !matches!(
                command.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) {
                used = used.saturating_add(MAX_METADATA_BYTES as u64);
            }
            commands.push(command);
        }
        for entry in fs::read_dir(self.root.join("slots"))? {
            let path = entry?.path();
            check_private(&path, true)?;
            let slot: SlotReceipt = read_optional(&path.join("receipt.json"), MAX_METADATA_BYTES)?
                .ok_or(WorkerStoreError::Quarantined)?;
            if self.slot_dir(&slot.slot.identity)? != path
                || self.read_command(&slot.slot.identity.command)?.is_none()
            {
                return Err(WorkerStoreError::Quarantined);
            }
            self.read_slot(&slot.slot.identity)?;
            let payload_bytes = private_tree_bytes(&path.join("payload"))?;
            let final_disposition = slot
                .disposition
                .as_ref()
                .is_some_and(|d| d.disposition.is_final());
            if !(final_disposition || slot.closed.is_some() && slot.producer_terminal) {
                used = used.saturating_add(
                    (MAX_WORKER_RESULT_ENVELOPE_BYTES as u64).saturating_sub(payload_bytes),
                );
            }
            let settled = slot.closed.is_some()
                && slot.producer_terminal
                && (final_disposition || self.read_blob(&slot)?.is_none());
            if !settled {
                used = used.saturating_add(MAX_METADATA_BYTES as u64);
            }
            slots.push(slot);
        }
        let mut workspaces = 0;
        let mut view_keys = BTreeSet::new();
        if self.root.join("workspaces").exists() {
            for entry in fs::read_dir(self.root.join("workspaces"))? {
                let path = entry?.path();
                if path.extension().is_none_or(|ext| ext != "json") {
                    continue;
                }
                let mut receipt: WorkspacePreparationReceipt =
                    read_optional(&path, MAX_METADATA_BYTES)?
                        .ok_or(WorkerStoreError::Quarantined)?;
                validate_prepare_command(&receipt.command)?;
                proof::overlay_bind(&self.root, &self.worker, &mut receipt)?;
                if receipt.version != STORE_VERSION
                    || receipt.worker != self.worker
                    || !valid_digest(&receipt.prepare_digest)
                    || path != self.workspace_path(&receipt.command)
                {
                    return Err(WorkerStoreError::Quarantined);
                }
                workspaces += 1;
                if let Some(view) = &receipt.commit_view {
                    view.validate(&receipt)?;
                    if !view_keys.insert(view.key.clone()) {
                        return Err(WorkerStoreError::Quarantined);
                    }
                    let owned = self.view_companion_bytes(&view.key)?;
                    used = used.saturating_add(view.reserved_bytes.saturating_sub(owned));
                }
                // Une préparation achevée est immutable ; son rejeu ne fait que la lire.
                if !receipt.commit_view.as_ref().is_some_and(|view| view.closed)
                    && (receipt.record.is_none()
                        || receipt.commit_view.as_ref().is_some_and(|view| {
                            view.materialization.as_ref().is_some_and(|operation| {
                                !operation.stopped || operation.available.is_none()
                            })
                        }))
                {
                    used = used.saturating_add(MAX_METADATA_BYTES as u64);
                }
            }
        }
        let count = commands.len()
            + slots.len()
            + workspaces
            + proof::count(&self.root, &self.worker)?
            + self.view_orphan_entries(&view_keys)?;
        if used > MAX_WORKER_STORE_BYTES as u64 || count > MAX_WORKER_STORE_ENTRIES {
            return Err(WorkerStoreError::Quota);
        }
        Ok((commands, slots, used, count))
    }

    fn view_root(&self) -> PathBuf {
        self.root.join("view-manifests")
    }

    fn view_companion_names(key: &str) -> [String; 5] {
        [
            format!("{key}.plan.json"),
            format!("{key}.intent.json"),
            format!("{key}.published.json"),
            format!("{key}.lock"),
            format!("view-{key}.json"),
        ]
    }

    fn view_companion_bytes(&self, key: &str) -> Result<u64, WorkerStoreError> {
        let mut bytes = 0u64;
        for name in Self::view_companion_names(key) {
            let path = self.view_root().join(name);
            match fs::symlink_metadata(&path) {
                Ok(metadata) => {
                    check_private(&path, false)?;
                    bytes = bytes.saturating_add(metadata.len());
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(bytes)
    }

    fn view_orphan_entries(&self, known: &BTreeSet<String>) -> Result<usize, WorkerStoreError> {
        let directory = self.view_root();
        match fs::symlink_metadata(&directory) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e.into()),
            Ok(_) => check_private(&directory, true)?,
        }
        let mut orphans = BTreeSet::new();
        let mut unidentified = 0usize;
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            check_private(&entry.path(), false)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let key = [".plan.json", ".intent.json", ".published.json", ".lock"]
                .iter()
                .find_map(|suffix| {
                    name.strip_suffix(suffix)
                        .filter(|candidate| valid_digest(candidate))
                })
                .or_else(|| {
                    name.strip_prefix("view-")
                        .and_then(|value| value.strip_suffix(".json"))
                        .filter(|candidate| valid_digest(candidate))
                });
            match key {
                Some(key) if known.contains(key) => {}
                Some(key) => {
                    orphans.insert(key.to_owned());
                }
                None => {
                    unidentified = unidentified.saturating_add(1);
                }
            }
        }
        Ok(orphans.len().saturating_add(unidentified))
    }

    /// Primitive interne encore non branchée à l'admission. Le reçu garde le
    /// budget avant les fichiers et le processus, y compris après coupure.
    #[cfg(test)]
    fn reserve_view_intent(
        &self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        self.health()?;
        validate_prepare_command(command)?;
        let digest = json_digest(&(command, request))?;
        if !matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) || request.posture != Some(SpawnPosture::Discovery)
            || intent.binding.worker != self.worker
            || intent.binding.command != *command
            || intent.binding.prepare_digest != digest
            || request.base_commit.as_deref() != Some(intent.base_commit())
            || intent.source != Path::new(&request.remote_root).join(&request.relative_cwd)
            || intent
                .data_root
                .starts_with(self.root.parent().ok_or(WorkerStoreError::Unknown)?)
        {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope));
        }
        let view = ViewReservation {
            key: intent.binding.key().map_err(workspace_failure)?,
            intent_digest: intent.proof_digest().map_err(workspace_failure)?,
            planning: None,
            reserved_bytes: intent.reserved_bytes().map_err(workspace_failure)?,
            source_marker: None,
            source_stopped: false,
            source_failure: None,
            source_attempted: false,
            closed: false,
            materialization: None,
        };
        self.reserve_view_receipt(command, request, view)?;
        private_dir(&self.view_root())?;
        crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .write_intent(intent)
            .map_err(workspace_failure)
    }

    fn reserve_view_receipt(
        &self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        view: ViewReservation,
    ) -> Result<(), WorkerStoreError> {
        let digest = json_digest(&(command, request))?;
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let path = self.workspace_path(command);
        if self.inventory()?.0.iter().any(|receipt| {
            receipt.identity.task_id == command.task_id && receipt.pre_start_fence.is_some()
        }) {
            return Err(WorkerStoreError::InvalidTransition);
        }
        if let Some(receipt) = proof::read_workspace_receipt(&self.root, &self.worker, command)? {
            if receipt.version != STORE_VERSION
                || receipt.worker != self.worker
                || receipt.command != *command
                || receipt.prepare_digest != digest
                || receipt.commit_view.as_ref().is_none_or(|existing| {
                    existing.key != view.key
                        || existing.intent_digest != view.intent_digest
                        || existing.reserved_bytes != view.reserved_bytes
                        || existing.planning != view.planning
                        || existing.closed
                })
                || receipt.fenced_previous.is_some()
            {
                return Err(WorkerStoreError::Conflict);
            }
        } else {
            if proof::read_bind(&self.root, &self.worker, command)?.is_some() {
                return Err(WorkerStoreError::Conflict);
            }
            self.admit(2 * MAX_METADATA_BYTES as u64 + view.reserved_bytes, 1)?;
            private_dir(&self.root.join("workspaces"))?;
            let receipt = WorkspacePreparationReceipt {
                version: STORE_VERSION,
                worker: self.worker.clone(),
                command: command.clone(),
                prepare_digest: digest,
                fenced_previous: None,
                record: None,
                commit_view: Some(view),
            };
            self.persist(&path, &receipt)?;
        }
        Ok(())
    }

    fn reserve_view_plan(
        &self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        seed: &crate::workspace_git::ViewPlan,
    ) -> Result<(), WorkerStoreError> {
        self.health()?;
        validate_prepare_command(command)?;
        if !matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) || request.posture != Some(SpawnPosture::Discovery)
            || seed.binding.worker != self.worker
            || seed.binding.command != *command
            || seed.binding.prepare_digest != json_digest(&(command, request))?
            || request.base_commit.as_deref() != Some(seed.base.as_str())
            || seed.source_root != Path::new(&request.remote_root)
            || seed.git_identity != request.source_root_identity
            || seed.source != Path::new(&request.remote_root).join(&request.relative_cwd)
            || seed
                .data_root
                .starts_with(self.root.parent().ok_or(WorkerStoreError::Unknown)?)
        {
            return Err(WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope));
        }
        let digest = seed.proof_digest().map_err(workspace_failure)?;
        let key = seed.binding.key().map_err(workspace_failure)?;
        // Relecture exacte : ne jamais recréer un délai ni retenter après fork.
        if let Some(receipt) = proof::read_workspace_receipt(&self.root, &self.worker, command)? {
            self.view_receipt_for_binding(&seed.binding)?;
            let view = receipt
                .commit_view
                .as_ref()
                .ok_or(WorkerStoreError::Unknown)?;
            if view.closed
                || view
                    .planning
                    .as_ref()
                    .is_none_or(|p| p.seed_digest != digest)
            {
                return Err(WorkerStoreError::Conflict);
            }
            let proofs = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
                .map_err(workspace_failure)?;
            if !view.source_attempted
                && !view.source_stopped
                && view.source_marker.is_none()
                && receipt.record.is_none()
                && view.planning.as_ref().is_some_and(|plan| !plan.adopted)
            {
                proofs.write_plan(seed).map_err(workspace_failure)?;
            }
            if proofs.load_plan(&key).map_err(workspace_failure)? != *seed {
                return Err(WorkerStoreError::Conflict);
            }
            return Ok(());
        }
        seed.live().map_err(workspace_failure)?;
        let view = ViewReservation {
            key,
            intent_digest: digest.clone(),
            planning: Some(ViewPlanning {
                seed_digest: digest,
                adopted: false,
            }),
            reserved_bytes: seed.reserved_bytes(),
            source_marker: None,
            source_stopped: false,
            source_failure: None,
            source_attempted: false,
            closed: false,
            materialization: None,
        };
        self.reserve_view_receipt(command, request, view)?;
        private_dir(&self.view_root())?;
        crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .write_plan(seed)
            .map_err(workspace_failure)
    }

    fn adopt_view_plan(
        &self,
        binding: &crate::workspace_git::ViewBinding,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        let proofs = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?;
        let seed = proofs
            .load_plan(&binding.key().map_err(workspace_failure)?)
            .map_err(workspace_failure)?;
        seed.matches(intent).map_err(workspace_failure)?;
        let digest = intent.proof_digest().map_err(workspace_failure)?;
        let seed_digest = seed.proof_digest().map_err(workspace_failure)?;
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.view_receipt_for_binding(binding)?;
        let view = receipt.commit_view.as_mut().unwrap();
        let planning = view.planning.as_mut().ok_or(WorkerStoreError::Conflict)?;
        if planning.seed_digest != seed_digest
            || view.closed
            || !view.source_stopped
            || view.source_failure.is_some()
            || view.source_marker.is_none()
            || seed.reserved_bytes() < intent.reserved_bytes().map_err(workspace_failure)?
        {
            return Err(WorkerStoreError::Conflict);
        }
        let reserved = seed
            .adopted_reserved_bytes(intent)
            .map_err(workspace_failure)?;
        if reserved > view.reserved_bytes {
            return Err(WorkerStoreError::Conflict);
        }
        if planning.adopted {
            if view.intent_digest != digest {
                return Err(WorkerStoreError::Conflict);
            }
            if view.reserved_bytes == reserved {
                return Ok(());
            }
        } else {
            seed.live().map_err(workspace_failure)?;
            planning.adopted = true;
            view.intent_digest = digest;
        }
        // Le manifeste est maintenant exact et immutable. Ne pas garder8Mio
        // inconnus pendant toute la mission ; les preuves privées restent
        // comptées. Les données exportées relèvent du volume, pas de ce quota.
        view.reserved_bytes = reserved;
        self.persist(&self.workspace_path(&binding.command), &receipt)
    }

    #[cfg(test)]
    fn mark_view_source_attempt(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        self.checked_view_receipt(command, intent)?;
        self.mark_view_source_attempt_binding(&intent.binding)
    }

    fn mark_view_source_attempt_binding(
        &self,
        binding: &crate::workspace_git::ViewBinding,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.view_receipt_for_binding(binding)?;
        let view = receipt.commit_view.as_mut().unwrap();
        if receipt.record.is_some()
            || view.closed
            || view.source_attempted
            || view.source_stopped
            || view.source_marker.is_some()
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        view.source_attempted = true;
        self.persist(&self.workspace_path(&binding.command), &receipt)
    }

    /// Seulement après Child arrêté, ou échec de spawn attesté par le contrôleur
    /// courant. Au redémarrage, l'absence du marqueur ne suffit jamais.
    fn record_view_source_exit(
        &self,
        binding: &crate::workspace_git::ViewBinding,
        result: Option<Result<(), crate::workspace_git::ExportFailure>>,
    ) -> Result<bool, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.view_receipt_for_binding(binding)?;
        let view = receipt.commit_view.as_ref().unwrap();
        if view.closed {
            return Ok(true);
        } // arrêt déjà durci ; ignorer le résultat tardif sans réécriture
        if !view.source_stopped
            && self.marker_for_view(&receipt)?.is_some_and(|marker| {
                crate::managed_process::group_exists(marker.pgid).unwrap_or(true)
                    && crate::managed_process::process_birth(marker.pgid)
                        .ok()
                        .is_none_or(|birth| birth == marker.birth)
            })
        {
            return Ok(false);
        }
        let view = receipt.commit_view.as_mut().unwrap();
        let changed = !view.source_stopped
            || (matches!(result, Some(Err(_))) && view.source_failure.is_none());
        view.source_stopped = true;
        if let Some(Err(cause)) = result {
            view.source_failure.get_or_insert(cause);
        }
        if changed {
            self.persist(&self.workspace_path(&binding.command), &receipt)?;
        }
        Ok(true)
    }

    #[cfg(test)]
    fn authorize_view_source(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
        marker: &ManagedMarker,
    ) -> Result<(), WorkerStoreError> {
        self.checked_view_receipt(command, intent)?;
        self.authorize_view_source_binding(&intent.binding, marker)
    }
    fn authorize_view_source_binding(
        &self,
        binding: &crate::workspace_git::ViewBinding,
        marker: &ManagedMarker,
    ) -> Result<(), WorkerStoreError> {
        let command = &binding.command;
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.view_receipt_for_binding(binding)?;
        if receipt.record.is_some() {
            return Err(WorkerStoreError::Conflict);
        }
        let view = receipt.commit_view.as_ref().unwrap();
        if view.closed || view.source_stopped {
            return Err(WorkerStoreError::InvalidTransition);
        }
        if marker.instance_id != command.task_id
            || marker.command_id != command.command_id
            || marker.generation != 1
            || marker.pgid == 0
            || marker.birth == 0
        {
            return Err(WorkerStoreError::Conflict);
        }
        if let Some(previous) = &view.source_marker {
            return if previous == marker {
                Ok(())
            } else {
                Err(WorkerStoreError::Conflict)
            };
        }
        let proofs = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?;
        if proofs
            .source_marker(binding)
            .map_err(workspace_failure)?
            .as_ref()
            != Some(marker)
        {
            return Err(WorkerStoreError::Conflict);
        }
        receipt.commit_view.as_mut().unwrap().source_attempted = true;
        receipt.commit_view.as_mut().unwrap().source_marker = Some(marker.clone());
        self.persist(&self.workspace_path(command), &receipt)
    }

    /// Le seul writer natif d'un record de vue : l'export et son groupe doivent
    /// avoir fini avant publication. Un rapport exact seul ne clôt aucun PID.
    fn prepare_view_workspace_publication(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        {
            let _guard = self
                .transition
                .lock()
                .map_err(|_| WorkerStoreError::Quarantined)?;
            let mut receipt = self.checked_view_receipt(command, intent)?;
            let view = receipt.commit_view.as_ref().unwrap();
            if self.inventory()?.0.iter().any(|known| {
                known.identity.task_id == command.task_id && known.pre_start_fence.is_some()
            }) {
                return Err(WorkerStoreError::InvalidTransition);
            }
            if view.closed {
                return Err(WorkerStoreError::InvalidTransition);
            }
            if view.source_failure.is_some() {
                return Err(WorkerStoreError::Workspace(
                    WorkerRefusal::WorkspacePrepareFailed,
                ));
            }
            if view
                .materialization
                .as_ref()
                .is_some_and(|operation| !operation.stopped)
            {
                return Err(WorkerStoreError::Unknown);
            }
            let marker = view
                .source_marker
                .as_ref()
                .ok_or(WorkerStoreError::Unknown)?;
            if !view.source_stopped && crate::managed_process::group_exists(marker.pgid)? {
                return Err(WorkerStoreError::Unknown);
            }
            if !view.source_stopped {
                receipt.commit_view.as_mut().unwrap().source_stopped = true;
                self.persist(&self.workspace_path(command), &receipt)?;
            }
            let known = if receipt
                .commit_view
                .as_ref()
                .unwrap()
                .materialization
                .is_some()
            {
                self.marker_for_materialization(&receipt)?
            } else {
                self.marker_for_view(&receipt)?
            };
            if let Some(marker) = known {
                crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
                    .map_err(workspace_failure)?
                    .remove_stopped_marker(&intent.binding, &marker)
                    .map_err(workspace_failure)?;
            }
        }
        // Preuve petite de publication avant d'allouer le lecteur ; un exit0
        // sans publication ou un fence n'est jamais mis en attente d'audit.
        crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .workspace_metadata(intent, crate::delegation::WorkspaceState::Active)
            .map_err(workspace_failure)?;
        Ok(())
    }

    #[cfg(test)]
    fn publish_view_workspace(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
        self.prepare_view_workspace_publication(command, intent)?;
        let record = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .workspace(intent)
            .map_err(workspace_failure)?;
        self.publish_view_snapshot(command, intent, record, &Mutex::new(BTreeSet::new()))
    }

    fn publish_view_snapshot(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
        record: crate::delegation::WorkspaceRecord,
        cancelled: &Mutex<BTreeSet<String>>,
    ) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
        let expected = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .workspace_metadata(intent, crate::delegation::WorkspaceState::Active)
            .map_err(workspace_failure)?;
        if serde_json::to_value(&expected).map_err(|_| WorkerStoreError::Conflict)?
            != serde_json::to_value(&record).map_err(|_| WorkerStoreError::Conflict)?
        {
            return Err(WorkerStoreError::Conflict);
        }
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.checked_view_receipt(command, intent)?;
        if receipt.commit_view.as_ref().unwrap().closed
            || !receipt.commit_view.as_ref().unwrap().source_stopped
            || receipt
                .commit_view
                .as_ref()
                .unwrap()
                .materialization
                .as_ref()
                .is_some_and(|operation| !operation.stopped)
        {
            return Err(WorkerStoreError::Unknown);
        }
        if self.inventory()?.0.iter().any(|known| {
            known.identity.task_id == command.task_id && known.pre_start_fence.is_some()
        }) {
            return Err(WorkerStoreError::InvalidTransition);
        }
        // Le lecteur réseau ne prend jamais transition : il pose seulement
        // cette clé puis enfile Cancel. Le writer unique prend le jeu après
        // ses lectures, le garde jusqu'au persist et ne fait aucun audit ici.
        let cancellations = cancelled
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        if cancellations.contains(&json_digest(command)?) {
            return Err(WorkerStoreError::InvalidTransition);
        }
        if let Some(existing) = receipt.record {
            return if same_workspace_anchor(
                &existing,
                &serde_json::to_value(&record).map_err(|_| WorkerStoreError::Conflict)?,
            ) {
                Ok(existing)
            } else {
                Err(WorkerStoreError::Conflict)
            };
        }
        receipt.record = Some(record.clone());
        self.persist(&self.workspace_path(command), &receipt)?;
        Ok(record)
    }

    fn checked_view_receipt(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<WorkspacePreparationReceipt, WorkerStoreError> {
        self.health()?;
        let receipt = proof::read_workspace_receipt(&self.root, &self.worker, command)?
            .ok_or(WorkerStoreError::Unknown)?;
        let binding = crate::workspace_git::ViewBinding {
            worker: self.worker.clone(),
            command: command.clone(),
            prepare_digest: receipt.prepare_digest.clone(),
        };
        if receipt.version != STORE_VERSION
            || receipt.worker != self.worker
            || receipt.command != *command
            || intent.binding != binding
            || receipt.fenced_previous.is_some()
        {
            return Err(WorkerStoreError::Conflict);
        }
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        view.validate(&receipt)?;
        if view.intent_digest != intent.proof_digest().map_err(workspace_failure)?
            || (view.planning.as_ref().is_some_and(|plan| !plan.adopted))
            || ((if view.planning.is_some() {
                view.reserved_bytes < intent.reserved_bytes().map_err(workspace_failure)?
            } else {
                view.reserved_bytes != intent.reserved_bytes().map_err(workspace_failure)?
            }) && (!view.closed
                || view.reserved_bytes
                    != intent.closed_reserved_bytes().map_err(workspace_failure)?))
        {
            return Err(WorkerStoreError::Conflict);
        }
        Ok(receipt)
    }

    /// Le reçu de commande existe avant que Start retire PreparedRuntime.
    /// Sa rétention dure tant que la vue est ouverte ; la fermeture de vue
    /// bloque ensuite tout réexport, même après collecte de cette commande.
    fn view_preparation_unused(
        &self,
        command: &WorkerCommandIdentity,
    ) -> Result<bool, WorkerStoreError> {
        self.health()?;
        validate_prepare_command(command)?;
        let receipt: Option<WorkerCommandReceipt> =
            read_optional(&self.command_path(command), MAX_METADATA_BYTES)?;
        let Some(receipt) = receipt else {
            return Ok(true);
        };
        if receipt.identity.command_id != command.command_id
            || receipt.identity.task_id != command.task_id
        {
            return Err(WorkerStoreError::Conflict);
        }
        self.read_command(&receipt.identity)?
            .ok_or(WorkerStoreError::Unknown)?;
        Ok(false)
    }

    fn view_receipt_for_binding(
        &self,
        binding: &crate::workspace_git::ViewBinding,
    ) -> Result<WorkspacePreparationReceipt, WorkerStoreError> {
        self.health()?;
        let receipt = proof::read_workspace_receipt(&self.root, &self.worker, &binding.command)?
            .ok_or(WorkerStoreError::Unknown)?;
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        view.validate(&receipt)?;
        if receipt.worker != self.worker
            || binding.worker != self.worker
            || receipt.command != binding.command
            || receipt.prepare_digest != binding.prepare_digest
            || receipt.fenced_previous.is_some()
            || binding.key().ok().as_deref() != Some(view.key.as_str())
        {
            return Err(WorkerStoreError::Conflict);
        }
        Ok(receipt)
    }

    /// O(1) reçus bornés, sans chargement du manifeste ni audit de fichiers.
    /// Cette liveness sert aussi à interrompre le helper d'une mission terminée.
    fn active_view_command(
        &self,
        execution: &WorkerResultIdentity,
    ) -> Result<WorkerCommandReceipt, WorkerStoreError> {
        self.health()?;
        execution.validate()?;
        let current = self
            .read_command(&execution.command)?
            .ok_or(WorkerStoreError::Unknown)?;
        let slot = self
            .read_slot(execution)?
            .ok_or(WorkerStoreError::Unknown)?;
        if current.phase != WorkerCommandPhase::Released
            || !current.native_ready
            || slot.closed.is_some()
            || slot.producer_terminal
            || slot.disposition.is_some()
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        Ok(current)
    }

    /// Lecture de l'exécution attestée avant toute matérialisation. Aucun
    /// répertoire ni processus n'est créé par ce contrôle. Après Release, le
    /// délai du bootstrap ne borne plus la mission : le slot corrélé et les
    /// gardes natives attestent son activité, sans prolonger ce délai.
    fn active_view_execution(
        &self,
        execution: &WorkerResultIdentity,
    ) -> Result<
        (
            crate::workspace_git::ViewIntent,
            WorkspacePreparationReceipt,
        ),
        WorkerStoreError,
    > {
        let current = self.active_view_command(execution)?;
        let (request, prepared) = current
            .preparation
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        if !matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) || prepared.effective_posture != SpawnPosture::Discovery
        {
            return Err(WorkerStoreError::Conflict);
        }
        let mut command = execution.command.clone();
        command.instance_id.clear();
        command.generation = 0;
        let receipt = view_anchor_receipt_at(&self.root, &self.worker, &command, Some(request))?;
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        if view.closed || view.source_failure.is_some() {
            return Err(WorkerStoreError::InvalidTransition);
        }
        let intent = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .load(&view.key)
            .map_err(workspace_failure)?;
        let receipt = self.checked_view_receipt(&intent.binding.command, &intent)?;
        let record = receipt.record.as_ref().ok_or(WorkerStoreError::Unknown)?;
        let digest = intent.proof_digest().map_err(workspace_failure)?;
        let (_, cwd) = intent
            .execution_paths(Path::new(&request.remote_root))
            .map_err(workspace_failure)?;
        let proof_root = self.view_root().canonicalize()?;
        if !matches!(
            record.strategy,
            crate::delegation::WorkspaceStrategy::CommitView
        ) || Path::new(&record.path) != intent.destination().map_err(workspace_failure)?
            || Path::new(&record.toplevel) != intent.source
            || record.base_commit != intent.base_commit()
            || Path::new(&current.placement.execution_cwd) != cwd
            || !receipt.commit_view.as_ref().unwrap().source_stopped
            || receipt
                .commit_view
                .as_ref()
                .unwrap()
                .source_marker
                .is_none()
            || record.commit_view.as_ref().is_none_or(|reference| {
                reference.binding != intent.binding
                    || reference.intent_digest != digest
                    || Path::new(&reference.proof_root) != proof_root
            })
            || !prepared
                .workspace
                .as_ref()
                .is_some_and(|value| same_workspace_anchor(record, value))
            || prepared.placement != current.placement
            || request.base_commit.as_deref() != Some(intent.base_commit())
            || current.placement.base_commit.as_deref() != Some(intent.base_commit())
        {
            return Err(WorkerStoreError::Conflict);
        }
        Ok((intent, receipt))
    }

    fn begin_materialization(
        &self,
        execution: &WorkerResultIdentity,
        path: &str,
    ) -> Result<
        (
            crate::workspace_git::ViewIntent,
            crate::workspace_git::ViewJobRequest,
        ),
        WorkerStoreError,
    > {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let (intent, mut receipt) = self.active_view_execution(execution)?;
        let view = receipt.commit_view.as_ref().unwrap();
        let mut sequence = 1;
        if let Some(previous) = &view.materialization {
            let crate::workspace_git::ViewJobRequest::Materialize {
                sequence: old,
                path: old_path,
                ..
            } = &previous.request
            else {
                return Err(WorkerStoreError::Quarantined);
            };
            // Le contrôle différé possède encore cette opération après
            // l'arrêt du helper. Son résultat ne doit pas être remplacé.
            let pending = !previous.stopped || previous.available.is_none();
            if previous.execution == *execution && old_path == path && pending {
                return Ok((intent, previous.request.clone()));
            }
            if pending {
                return Err(WorkerStoreError::Locked);
            }
            sequence = old.checked_add(1).ok_or(WorkerStoreError::Quota)?;
        }
        let request = crate::workspace_git::ViewJobRequest::Materialize {
            command: execution.command.clone(),
            sequence,
            path: path.to_owned(),
        };
        request
            .identity(&intent)
            .map_err(|_| WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope))?;
        // Le même budget de marqueur est réutilisé après l'arrêt de l'export ;
        // les métadonnées du reçu et leur temporaire sont admis avant écriture.
        self.admit(2 * MAX_METADATA_BYTES as u64, 0)?;
        let old_marker = if view.materialization.is_some() {
            self.marker_for_materialization(&receipt)?
        } else {
            self.marker_for_view(&receipt)?
        };
        if let Some(marker) = old_marker {
            crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
                .map_err(workspace_failure)?
                .remove_stopped_marker(&intent.binding, &marker)
                .map_err(workspace_failure)?;
        }
        receipt.commit_view.as_mut().unwrap().materialization = Some(ViewMaterialization {
            execution: execution.clone(),
            request: request.clone(),
            marker: None,
            attempted: false,
            stopped: false,
            available: None,
            failure: None,
        });
        self.persist(&self.workspace_path(&receipt.command), &receipt)?;
        Ok((intent, request))
    }

    fn mark_materialization_attempt(
        &self,
        execution: &WorkerResultIdentity,
        request: &crate::workspace_git::ViewJobRequest,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let (_, mut receipt) = self.active_view_execution(execution)?;
        let operation = receipt
            .commit_view
            .as_mut()
            .unwrap()
            .materialization
            .as_mut()
            .ok_or(WorkerStoreError::Unknown)?;
        if operation.execution != *execution
            || operation.request != *request
            || operation.stopped
            || operation.attempted
            || operation.marker.is_some()
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        operation.attempted = true;
        let command = receipt.command.clone();
        self.persist(&self.workspace_path(&command), &receipt)
    }

    fn authorize_materialization(
        &self,
        execution: &WorkerResultIdentity,
        request: &crate::workspace_git::ViewJobRequest,
        marker: &ManagedMarker,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let (intent, mut receipt) = self.active_view_execution(execution)?;
        let operation = receipt
            .commit_view
            .as_mut()
            .unwrap()
            .materialization
            .as_mut()
            .ok_or(WorkerStoreError::Unknown)?;
        if operation.execution != *execution || operation.request != *request || operation.stopped {
            return Err(WorkerStoreError::Conflict);
        }
        let expected = request.identity(&intent).map_err(workspace_failure)?;
        if marker.instance_id != expected.instance_id
            || marker.command_id != expected.command_id
            || marker.generation != expected.generation
            || marker.pgid == 0
            || marker.birth == 0
        {
            return Err(WorkerStoreError::Conflict);
        }
        let actual = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .operation_marker(&intent.binding, &expected)
            .map_err(workspace_failure)?;
        if actual.as_ref() != Some(marker) {
            return Err(WorkerStoreError::Conflict);
        }
        if let Some(previous) = &operation.marker {
            return if previous == marker {
                Ok(())
            } else {
                Err(WorkerStoreError::Conflict)
            };
        }
        operation.attempted = true;
        operation.marker = Some(marker.clone());
        self.persist(&self.workspace_path(&receipt.command), &receipt)
    }

    fn marker_for_materialization(
        &self,
        receipt: &WorkspacePreparationReceipt,
    ) -> Result<Option<ManagedMarker>, WorkerStoreError> {
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        let Some(operation) = &view.materialization else {
            return Ok(None);
        };
        if let Some(marker) = &operation.marker {
            return Ok(Some(marker.clone()));
        }
        let binding = crate::workspace_git::ViewBinding {
            worker: self.worker.clone(),
            command: receipt.command.clone(),
            prepare_digest: receipt.prepare_digest.clone(),
        };
        let expected = operation
            .request
            .identity_for_binding(&binding)
            .map_err(workspace_failure)?;
        crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?
            .operation_marker(&binding, &expected)
            .map_err(workspace_failure)
    }

    /// Un résultat de helper n'est pas une preuve d'arrêt. Durcir l'arrêt avant
    /// le contrôle du fichier, puis son constat ; aucune commande n'est relancée.
    fn stop_materialization(
        &self,
        execution: &WorkerResultIdentity,
        request: &crate::workspace_git::ViewJobRequest,
        result: Option<Result<(), crate::workspace_git::ExportFailure>>,
    ) -> Result<Option<crate::workspace_git::ViewIntent>, WorkerStoreError> {
        let mut prepare = execution.command.clone();
        prepare.instance_id.clear();
        prepare.generation = 0;
        let intent = {
            let _guard = self
                .transition
                .lock()
                .map_err(|_| WorkerStoreError::Quarantined)?;
            self.health()?;
            let mut receipt = view_anchor_receipt_at(&self.root, &self.worker, &prepare, None)?;
            prepare = receipt.command.clone();
            let view = receipt
                .commit_view
                .as_ref()
                .ok_or(WorkerStoreError::Unknown)?;
            if view.closed {
                return Err(WorkerStoreError::InvalidTransition);
            }
            view.validate(&receipt)?;
            let operation = view
                .materialization
                .as_ref()
                .ok_or(WorkerStoreError::Unknown)?;
            if operation.execution != *execution || operation.request != *request {
                return Err(WorkerStoreError::Conflict);
            }
            let marker = self.marker_for_materialization(&receipt)?;
            if !operation.stopped
                && (marker.as_ref().is_some_and(|marker| {
                    crate::managed_process::group_exists(marker.pgid).unwrap_or(true)
                }) || (marker.is_none() && operation.attempted && result.is_none()))
            {
                return Ok(None);
            }
            if !operation.stopped {
                receipt
                    .commit_view
                    .as_mut()
                    .unwrap()
                    .materialization
                    .as_mut()
                    .unwrap()
                    .stopped = true;
                self.persist(&self.workspace_path(&prepare), &receipt)?;
            }
            let view = receipt.commit_view.as_ref().unwrap();
            let intent = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
                .map_err(workspace_failure)?
                .load(&view.key)
                .map_err(workspace_failure)?;
            self.checked_view_receipt(&prepare, &intent)?;
            intent
        };
        Ok(Some(intent))
    }

    fn finish_materialization(
        &self,
        execution: &WorkerResultIdentity,
        request: &crate::workspace_git::ViewJobRequest,
        result: Option<Result<(), crate::workspace_git::ExportFailure>>,
    ) -> Result<bool, WorkerStoreError> {
        let Some(intent) = self.stop_materialization(execution, request, result)? else {
            return Ok(false);
        };
        let crate::workspace_git::ViewJobRequest::Materialize { path, .. } = request else {
            return Err(WorkerStoreError::Conflict);
        };
        let (available, failure) = match result {
            Some(Err(cause)) => (Some(false), Some(cause)),
            observed => {
                let available =
                    crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
                        .map_err(workspace_failure)?
                        .file_available(&intent, path);
                match available {
                    Ok(true) => (Some(true), None),
                    Ok(false) if observed.is_none() => (Some(false), None),
                    _ => (
                        Some(false),
                        Some(crate::workspace_git::ExportFailure::IntegrityFailed),
                    ),
                }
            }
        };
        self.publish_materialization_report(execution, request, &intent, available, failure)
    }

    fn publish_materialization_report(
        &self,
        execution: &WorkerResultIdentity,
        request: &crate::workspace_git::ViewJobRequest,
        intent: &crate::workspace_git::ViewIntent,
        available: Option<bool>,
        failure: Option<crate::workspace_git::ExportFailure>,
    ) -> Result<bool, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let prepare = &intent.binding.command;
        let mut receipt = self.checked_view_receipt(prepare, intent)?;
        if receipt.commit_view.as_ref().unwrap().closed {
            return Err(WorkerStoreError::InvalidTransition);
        }
        let operation = receipt
            .commit_view
            .as_mut()
            .unwrap()
            .materialization
            .as_mut()
            .ok_or(WorkerStoreError::Unknown)?;
        if operation.execution != *execution || operation.request != *request || !operation.stopped
        {
            return Err(WorkerStoreError::Conflict);
        }
        // Une preuve antérieure d'intégrité ne devient pas un succès de processus ;
        // il s'agit seulement de la disponibilité constatée du fichier.
        operation.available = available;
        operation.failure = failure;
        self.persist(&self.workspace_path(prepare), &receipt)?;
        Ok(true)
    }

    fn conserve_unreserved_view(
        &self,
        binding: &crate::workspace_git::ViewBinding,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        if binding.worker != self.worker {
            return Err(WorkerStoreError::Conflict);
        }
        validate_prepare_command(&binding.command)?;
        if proof::read_workspace_receipt(&self.root, &self.worker, &binding.command)?.is_some()
            || !self
                .view_receipts(Some(&binding.command.task_id))?
                .is_empty()
        {
            return Err(WorkerStoreError::Unknown);
        }
        let key = binding.key().map_err(workspace_failure)?;
        if self.view_root().exists() {
            check_private(&self.view_root(), true)?;
        }
        for name in Self::view_companion_names(&key) {
            match fs::symlink_metadata(self.view_root().join(name)) {
                Ok(_) => return Err(WorkerStoreError::Unknown),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        let (commands, slots, _, _) = self.inventory()?;
        let mut exact_fence = false;
        for command in commands
            .iter()
            .filter(|command| command.identity.task_id == binding.command.task_id)
        {
            if command.phase != WorkerCommandPhase::Refused
                || command.pre_start_fence.is_none()
                || command.marker.is_some()
                || command.permit.is_some()
                || command.preparation.is_some()
                || command.release_epoch.is_some()
                || command.native_ready
            {
                return Err(WorkerStoreError::Unknown);
            }
            exact_fence |= command.identity.command_id == binding.command.command_id;
        }
        if !exact_fence
            || slots
                .iter()
                .any(|slot| slot.slot.identity.command.task_id == binding.command.task_id)
        {
            return Err(WorkerStoreError::Unknown);
        }
        self.admit(2 * MAX_METADATA_BYTES as u64 + 5120, 1)?;
        private_dir(&self.root.join("workspaces"))?;
        self.persist(
            &self.workspace_path(&binding.command),
            &WorkspacePreparationReceipt {
                version: STORE_VERSION,
                worker: self.worker.clone(),
                command: binding.command.clone(),
                prepare_digest: binding.prepare_digest.clone(),
                fenced_previous: None,
                record: None,
                commit_view: Some(ViewReservation {
                    key,
                    intent_digest: binding.prepare_digest.clone(),
                    planning: None,
                    reserved_bytes: 5120,
                    source_marker: None,
                    source_stopped: true,
                    source_failure: None,
                    source_attempted: false,
                    closed: true,
                    materialization: None,
                }),
            },
        )
    }

    fn conserve_unpublished_view(
        &self,
        binding: &crate::workspace_git::ViewBinding,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.view_receipt_for_binding(binding)?;
        if receipt.record.is_some() {
            return Err(WorkerStoreError::InvalidTransition);
        }
        let view = receipt.commit_view.as_ref().unwrap();
        if view.closed {
            return Ok(());
        }
        if view.materialization.is_some()
            || (view.source_attempted
                && (!view.source_stopped
                    || (view.source_marker.is_none() && view.source_failure.is_none())))
        {
            return Err(WorkerStoreError::Unknown);
        }
        if let Some(marker) = self.marker_for_view(&receipt)?
            && crate::managed_process::group_exists(marker.pgid)?
            && crate::managed_process::process_birth(marker.pgid)
                .ok()
                .is_none_or(|birth| birth == marker.birth)
        {
            return Err(WorkerStoreError::Unknown);
        }
        let (commands, slots, _, _) = self.inventory()?;
        let related = commands
            .iter()
            .filter(|command| command.identity.task_id == binding.command.task_id);
        for command in related {
            if command.phase != WorkerCommandPhase::Refused
                || command.pre_start_fence.is_none()
                || command.marker.is_some()
                || command.permit.is_some()
                || command.release_epoch.is_some()
                || command.native_ready
                || command.preparation.is_some()
            {
                return Err(WorkerStoreError::InvalidTransition);
            }
        }
        if slots
            .iter()
            .any(|slot| slot.slot.identity.command.task_id == binding.command.task_id)
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        let view = receipt.commit_view.as_mut().unwrap();
        view.closed = true;
        view.source_stopped = true;
        view.reserved_bytes = view.reserved_bytes.min(5120 + 16 * 1024); // jamais d'augmentation à la clôture
        self.persist(&self.workspace_path(&binding.command), &receipt)
    }

    /// O(c+s) reçus bornés. Cette garde prouve seulement la clôture native,
    /// jamais l'absence de lecteur externe ni une autorisation de supprimer.
    #[cfg(test)]
    fn check_view_native_closure(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.check_view_native_closure_locked(command, intent)
    }

    /// Appelant sous transition.lock ; mêmes preuves, sans deuxième autorité.
    fn check_view_native_closure_locked(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        self.health()?;
        let receipt = self.checked_view_receipt(command, intent)?;
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        if view.closed {
            return Ok(());
        }
        if receipt.record.is_none()
            || !view.source_stopped
            || view.source_marker.is_none()
            || view
                .materialization
                .as_ref()
                .is_some_and(|operation| !operation.stopped)
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        let (commands, slots, _, _) = self.inventory()?;
        let related = commands
            .iter()
            .filter(|current| current.identity.task_id == command.task_id)
            .collect::<Vec<_>>();
        if !related
            .iter()
            .any(|current| current.identity.command_id == command.command_id)
        {
            return Err(WorkerStoreError::Unknown);
        }
        let identities = related
            .iter()
            .map(|current| command_key(&current.identity))
            .collect::<HashSet<_>>();
        for current in &related {
            if !matches!(
                current.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) {
                return Err(WorkerStoreError::InvalidTransition);
            }
            if let Some(marker) = &current.marker {
                if crate::managed_process::group_exists(marker.pgid)?
                    && crate::managed_process::process_birth(marker.pgid)
                        .ok()
                        .is_none_or(|birth| birth == marker.birth)
                {
                    return Err(WorkerStoreError::InvalidTransition);
                }
                if !self.drainage_proven(current)? {
                    return Err(WorkerStoreError::Unknown);
                }
            } else if current.pre_start_fence.is_none()
                || current.permit.is_some()
                || current.release_epoch.is_some()
                || current.native_ready
                || current.startup_failed
            {
                // Le seul état Terminal, ou un reçu manquant, ne vaut pas preuve
                // d'arrêt. Sans naissance, exiger le fence natif persistant.
                return Err(WorkerStoreError::Unknown);
            }
        }
        for slot in slots
            .iter()
            .filter(|slot| slot.slot.identity.command.task_id == command.task_id)
        {
            if !identities.contains(&command_key(&slot.slot.identity.command)) {
                return Err(WorkerStoreError::Unknown);
            }
            let disposed = slot
                .disposition
                .as_ref()
                .is_some_and(|entry| entry.disposition.is_final());
            let closed_without_result = slot.closed.is_some()
                && slot.disposition.is_none()
                && self.read_blob(slot)?.is_none();
            if !slot.producer_terminal || !(disposed || closed_without_result) {
                return Err(WorkerStoreError::InvalidTransition);
            }
        }
        Ok(())
    }

    /// Décision de conservation autorisée par le propriétaire à l'entrée native
    /// (port encore fermé). Aucun inventaire Linux incomplet ne mène au retrait.
    /// La clôture monotone précède toute réduction, sous le verrou du store.
    fn freeze_view(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.check_view_native_closure_locked(command, intent)?;
        let mut receipt = self.checked_view_receipt(command, intent)?;
        if receipt.commit_view.as_ref().unwrap().closed {
            return Ok(());
        }
        receipt.commit_view.as_mut().unwrap().closed = true;
        receipt
            .record
            .as_mut()
            .ok_or(WorkerStoreError::Unknown)?
            .state = crate::delegation::WorkspaceState::Kept;
        self.persist(&self.workspace_path(command), &receipt)
    }

    /// Arrêts natifs prouvés, mais données conservées : pas de déplacement,
    /// retrait ou nouveau droit. Une coupure garde la réservation pleine jusqu'à
    /// ce que la petite preuve soit durable. L'ancre close reste sans expiration.
    #[cfg(test)]
    fn compact_closed_view(
        &self,
        command: &WorkerCommandIdentity,
        intent: &crate::workspace_git::ViewIntent,
    ) -> Result<(), WorkerStoreError> {
        self.freeze_view(command, intent)?;
        let proofs = crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(workspace_failure)?;
        let reserved = proofs.compact_closed(intent).map_err(workspace_failure)?;
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        let mut receipt = self.checked_view_receipt(command, intent)?;
        let view = receipt.commit_view.as_mut().unwrap();
        if !view.closed {
            return Err(WorkerStoreError::InvalidTransition);
        }
        if view.reserved_bytes != reserved {
            view.reserved_bytes = reserved;
            self.persist(&self.workspace_path(command), &receipt)?;
        }
        Ok(())
    }

    fn view_receipts(
        &self,
        task_id: Option<&str>,
    ) -> Result<Vec<WorkspacePreparationReceipt>, WorkerStoreError> {
        self.health()?;
        let directory = self.root.join("workspaces");
        if !directory.exists() {
            return Ok(Vec::new());
        }
        check_private(&directory, true)?;
        let mut result = Vec::new();
        for (index, entry) in fs::read_dir(&directory)?.enumerate() {
            if index >= MAX_WORKER_STORE_ENTRIES {
                return Err(WorkerStoreError::Quota);
            }
            let path = entry?.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let receipt: WorkspacePreparationReceipt =
                read_optional(&path, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Quarantined)?;
            if task_id.is_some_and(|task| receipt.command.task_id != task) {
                continue;
            }
            validate_prepare_command(&receipt.command)?;
            if receipt.version != STORE_VERSION
                || receipt.worker != self.worker
                || self.workspace_path(&receipt.command) != path
            {
                return Err(WorkerStoreError::Quarantined);
            }
            match &receipt.commit_view {
                Some(view) => view.validate(&receipt)?,
                None if receipt.record.as_ref().is_some_and(|record| {
                    matches!(
                        record.strategy,
                        crate::delegation::WorkspaceStrategy::CommitView
                    )
                }) =>
                {
                    return Err(WorkerStoreError::Unknown);
                }
                None => continue,
            }
            result.push(receipt);
        }
        Ok(result)
    }

    fn marker_for_view(
        &self,
        receipt: &WorkspacePreparationReceipt,
    ) -> Result<Option<ManagedMarker>, WorkerStoreError> {
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        if let Some(marker) = &view.source_marker {
            return Ok(Some(marker.clone()));
        }
        if !self.view_root().exists() {
            return Ok(None);
        }
        let binding = crate::workspace_git::ViewBinding {
            worker: self.worker.clone(),
            command: receipt.command.clone(),
            prepare_digest: receipt.prepare_digest.clone(),
        };
        crate::workspace_git::ViewProofs::open(&self.view_root().canonicalize()?)
            .map_err(|_| WorkerStoreError::Unknown)?
            .source_marker(&binding)
            .map_err(|_| WorkerStoreError::Unknown)
    }

    fn view_source_markers(
        &self,
        task_id: &str,
    ) -> Result<Vec<(String, WorkerCommandIdentity, ManagedMarker)>, WorkerStoreError> {
        let mut markers = Vec::new();
        for receipt in self.view_receipts(Some(task_id))? {
            let view = receipt.commit_view.as_ref().unwrap();
            if !view.source_stopped
                && let Some(marker) = self.marker_for_view(&receipt)?
            {
                markers.push((view.key.clone(), receipt.command.clone(), marker));
            }
            if view
                .materialization
                .as_ref()
                .is_some_and(|operation| !operation.stopped)
                && let Some(marker) = self.marker_for_materialization(&receipt)?
            {
                let operation = view.materialization.as_ref().unwrap();
                markers.push((
                    format!("{}-file-{}", view.key, marker.command_id),
                    operation.execution.command.clone(),
                    marker,
                ));
            }
        }
        Ok(markers)
    }

    /// transition est détenu. Le reçu fait autorité même sans sidecar.
    fn confirm_view_receipt_stopped(
        &self,
        receipt: &mut WorkspacePreparationReceipt,
    ) -> Result<bool, WorkerStoreError> {
        let mut changed = false;
        if !receipt.commit_view.as_ref().unwrap().source_stopped {
            match self.marker_for_view(receipt)? {
                Some(marker) if crate::managed_process::group_exists(marker.pgid)? => {
                    return Ok(false);
                }
                None if receipt.commit_view.as_ref().unwrap().source_attempted => return Ok(false),
                _ => {}
            }
            receipt.commit_view.as_mut().unwrap().source_stopped = true;
            changed = true;
        }
        if receipt
            .commit_view
            .as_ref()
            .unwrap()
            .materialization
            .as_ref()
            .is_some_and(|operation| !operation.stopped)
        {
            match self.marker_for_materialization(receipt)? {
                Some(marker) if crate::managed_process::group_exists(marker.pgid)? => {
                    return Ok(false);
                }
                None if receipt
                    .commit_view
                    .as_ref()
                    .unwrap()
                    .materialization
                    .as_ref()
                    .unwrap()
                    .attempted =>
                {
                    return Ok(false);
                }
                _ => {}
            }
            receipt
                .commit_view
                .as_mut()
                .unwrap()
                .materialization
                .as_mut()
                .unwrap()
                .stopped = true;
            changed = true;
        }
        if changed {
            self.persist(&self.workspace_path(&receipt.command), receipt)?;
        }
        Ok(true)
    }

    fn confirm_view_sources_stopped(&self, task_id: &str) -> Result<bool, WorkerStoreError> {
        for mut receipt in self.view_receipts(Some(task_id))? {
            if !self.confirm_view_receipt_stopped(&mut receipt)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// O(1) reçu borné par commande ; pas de parcours global par Reconcile.
    fn settle_view_command_sources(
        &self,
        command: &WorkerCommandIdentity,
    ) -> Result<bool, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut prep = command.clone();
        prep.instance_id.clear();
        prep.generation = 0;
        let Some(mut receipt) = proof::read_workspace_receipt(&self.root, &self.worker, &prep)?
        else {
            return Ok(true);
        };
        if receipt.worker != self.worker || receipt.command != prep {
            return Err(WorkerStoreError::Conflict);
        }
        let Some(view) = &receipt.commit_view else {
            return Ok(true);
        };
        view.validate(&receipt)?;
        self.confirm_view_receipt_stopped(&mut receipt)
    }

    fn report_retained_view(
        &self,
        command: &WorkerCommandIdentity,
        record: &crate::delegation::WorkspaceRecord,
    ) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
        let mut prep = command.clone();
        prep.instance_id.clear();
        prep.generation = 0;
        let receipt = view_anchor_receipt_at(&self.root, &self.worker, &prep, None)?;
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        view.validate(&receipt)?;
        let original = receipt.record.as_ref().ok_or(WorkerStoreError::Unknown)?;
        let reference = original
            .commit_view
            .as_ref()
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt.worker != self.worker
            || receipt.command.task_id != prep.task_id
            || !view.source_stopped
            || view
                .materialization
                .as_ref()
                .is_some_and(|operation| !operation.stopped)
        {
            return Err(WorkerStoreError::Unknown);
        }
        if !same_workspace_anchor(
            original,
            &serde_json::to_value(record).map_err(|_| WorkerStoreError::Conflict)?,
        ) || reference.binding.worker != self.worker
            || reference.binding.command != receipt.command
            || reference.binding.prepare_digest != receipt.prepare_digest
            || reference.intent_digest != view.intent_digest
            || Path::new(&reference.proof_root) != self.view_root().canonicalize()?
        {
            return Err(WorkerStoreError::Conflict);
        }
        crate::workspace_git::report(original).map_err(workspace_failure)
    }

    fn settle_view_sources(&self, task_id: &str) -> Result<bool, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        self.confirm_view_sources_stopped(task_id)
    }

    fn workspace_path(&self, command: &WorkerCommandIdentity) -> PathBuf {
        self.root
            .join("workspaces")
            .join(format!("{}.json", marker_key(command)))
    }

    /// L'appelant détient transition. Le reçu fence est l'autorité ; le
    /// message du maître ne peut ni créer ni remplacer son ancre Git.
    fn fenced_workspace_locked(
        &self,
        previous: &WorkerCommandIdentity,
        placement: &RemotePlacement,
        workspace: &serde_json::Value,
    ) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
        let fence = self
            .read_command(previous)?
            .ok_or(WorkerStoreError::Unknown)?;
        if fence.pre_start_fence.is_none() || fence.phase != WorkerCommandPhase::Refused {
            return Err(WorkerStoreError::Unknown);
        }
        if fence.placement != *placement {
            return Err(WorkerStoreError::Conflict);
        }
        exact_fence_at(&self.root, &self.worker, previous, placement)?;
        let mut prepare_command = previous.clone();
        prepare_command.instance_id.clear();
        prepare_command.generation = 0;
        let record = original_workspace_at(&self.root, &self.worker, &prepare_command)?;
        if !same_workspace_anchor(&record, workspace) {
            return Err(WorkerStoreError::Conflict);
        }
        Ok(record)
    }

    fn fenced_workspace(
        &self,
        previous: &WorkerCommandIdentity,
        placement: &RemotePlacement,
        workspace: &serde_json::Value,
    ) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        self.fenced_workspace_locked(previous, placement, workspace)
    }

    fn fenced_placement(
        &self,
        previous: &WorkerCommandIdentity,
        placement: &RemotePlacement,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let receipt = self
            .read_command(previous)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt.pre_start_fence.is_none() || receipt.phase != WorkerCommandPhase::Refused {
            return Err(WorkerStoreError::Unknown);
        }
        exact_fence_at(&self.root, &self.worker, previous, placement)
    }

    /// O(n) via admit/inventory ; n est borné par MAX_WORKER_STORE_ENTRIES.
    fn bind_fenced_workspace(
        &self,
        previous: &WorkerCommandIdentity,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        placement: &RemotePlacement,
        workspace: &serde_json::Value,
    ) -> Result<(), WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        command.validate_preparation()?;
        if previous.task_id != command.task_id || previous.command_id == command.command_id {
            return Err(WorkerStoreError::Conflict);
        }
        self.fenced_workspace_locked(previous, placement, workspace)?;
        let digest = fenced_prepare_digest(command, request)?;
        let path = self.workspace_path(command);
        let proof = proof::BindProof {
            version: STORE_VERSION,
            worker: self.worker.clone(),
            command: command.clone(),
            prepare_digest: digest.clone(),
            previous: previous.clone(),
        };
        let stored = proof::read_bind(&self.root, &self.worker, command)?;
        if stored.as_ref().is_some_and(|stored| *stored != proof) {
            return Err(WorkerStoreError::Conflict);
        }
        if let Some(existing) = proof::read_workspace_receipt(&self.root, &self.worker, command)? {
            return if existing.version == STORE_VERSION
                && existing.worker == self.worker
                && existing.command == *command
                && existing.prepare_digest == digest
                && existing.fenced_previous.as_ref() == Some(previous)
                && existing.record.is_none()
            {
                Ok(())
            } else {
                Err(WorkerStoreError::Conflict)
            };
        }
        if stored.is_none() {
            self.admit(3 * MAX_METADATA_BYTES as u64, 2)?;
            proof::ensure_dir(&self.root)?;
            self.persist(&proof::bind_path(&self.root, command), &proof)?;
        } else {
            self.admit(2 * MAX_METADATA_BYTES as u64, 1)?;
        }
        self.persist(
            &path,
            &WorkspacePreparationReceipt {
                version: STORE_VERSION,
                worker: self.worker.clone(),
                command: command.clone(),
                prepare_digest: digest,
                fenced_previous: None,
                record: None,
                commit_view: None,
            },
        )
    }

    fn prepare_workspace(
        &self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        root: &Path,
    ) -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
        self.health()?;
        let digest = json_digest(&(command, request))?;
        let path = self.workspace_path(command);
        if let Some(receipt) = proof::read_workspace_receipt(&self.root, &self.worker, command)? {
            if receipt.worker != self.worker
                || receipt.command != *command
                || receipt.prepare_digest != digest
                || receipt.fenced_previous.is_some()
            {
                return Err(WorkerStoreError::Conflict);
            }
            let record = receipt.record.ok_or(WorkerStoreError::Workspace(
                WorkerRefusal::WorkspacePrepareIncomplete,
            ))?;
            crate::workspace_failure::stage(
                bridget_transport::protocol::WorkspacePrepareStage::Report,
                crate::workspace_git::report(&record),
            )
            .map_err(workspace_failure)?;
            return Ok(record);
        }
        let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        else {
            return Err(WorkerRefusal::InvalidEnvelope.into());
        };
        if let Some(commit) = request.base_commit.as_deref() {
            crate::workspace_failure::stage(
                bridget_transport::protocol::WorkspacePrepareStage::Import,
                crate::workspace_git::import_remote_commit(root, commit),
            )
            .map_err(workspace_failure)?;
        }
        let plan = crate::workspace_failure::stage(
            bridget_transport::protocol::WorkspacePrepareStage::Preflight,
            crate::workspace_git::preflight_remote(root, base_ref.as_deref(), branch.as_deref()),
        )
        .map_err(workspace_failure)?;
        if request
            .base_commit
            .as_ref()
            .is_some_and(|expected| expected != &plan.base_commit)
        {
            return Err(WorkerStoreError::Conflict);
        }
        let mut receipt = WorkspacePreparationReceipt {
            version: STORE_VERSION,
            worker: self.worker.clone(),
            command: command.clone(),
            prepare_digest: digest,
            fenced_previous: None,
            record: None,
            commit_view: None,
        };
        {
            let _guard = self
                .transition
                .lock()
                .map_err(|_| WorkerStoreError::Quarantined)?;
            // Un lien198 sans reçu interdit de créer un autre worktree.
            if proof::read_bind(&self.root, &self.worker, command)?.is_some() {
                return Err(WorkerStoreError::Conflict);
            }
            self.admit(2 * MAX_METADATA_BYTES as u64, 1)?;
            private_dir(&self.root.join("workspaces"))?;
            self.persist(&path, &receipt)?;
        }
        // Le maître a réservé sa Task avant ce second Prepare. Aucun bootstrap
        // n'existe encore et aucun verrou métier ne traverse cette E/S Git.
        let record = crate::workspace_failure::stage(
            bridget_transport::protocol::WorkspacePrepareStage::Create,
            crate::workspace_git::create(&plan, &command.task_id),
        )
        .map_err(workspace_failure)?;
        receipt.record = Some(record.clone());
        self.persist(&path, &receipt)?;
        Ok(record)
    }
}

/// Lecture du wrapper : aucun verrou écrivain du worker, aucune mutation.
/// Le tuple d'exécution doit désigner exactement un slot actif et attesté.
pub fn find_result_reservation(
    root: &Path,
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
    mission_id: &str,
    execution_id: &str,
    execution_generation: u64,
) -> Result<WorkerResultReservation, WorkerStoreError> {
    worker.validate()?;
    command.validate()?;
    check_private(root, true)?;
    let parent = root.parent().ok_or(WorkerStoreError::Unknown)?;
    check_private(parent, true)?;
    let enrollment: Enrollment = read_optional(
        &parent.join(format!(
            ".worker-{}.enrolled",
            digest_bytes(root.as_os_str().as_encoded_bytes())
        )),
        MAX_METADATA_BYTES,
    )?
    .ok_or(WorkerStoreError::Quarantined)?;
    if enrollment.version != STORE_VERSION || enrollment.worker != *worker || enrollment.quarantined
    {
        return Err(WorkerStoreError::Quarantined);
    }
    let slots = root.join("slots");
    check_private(&slots, true)?;
    let mut found = None;
    for (count, entry) in fs::read_dir(slots)?.enumerate() {
        if count >= MAX_WORKER_STORE_ENTRIES {
            return Err(WorkerStoreError::Quota);
        }
        let directory = entry?.path();
        check_private(&directory, true)?;
        let receipt: SlotReceipt =
            read_optional(&directory.join("receipt.json"), MAX_METADATA_BYTES)?
                .ok_or(WorkerStoreError::Quarantined)?;
        receipt.slot.validate()?;
        let identity = &receipt.slot.identity;
        let slot_id = json_digest(identity)?;
        if receipt.version != STORE_VERSION
            || receipt.worker != *worker
            || directory
                .file_name()
                .is_none_or(|name| name != slot_id.as_str())
        {
            return Err(WorkerStoreError::Quarantined);
        }
        if identity.command == *command
            && identity.mission_id == mission_id
            && identity.execution_id == execution_id
            && identity.execution_generation == execution_generation
        {
            if found.is_some() {
                return Err(WorkerStoreError::Conflict);
            }
            if receipt.closed.is_some() || receipt.producer_terminal {
                return Err(WorkerStoreError::InvalidTransition);
            }
            found = Some(WorkerResultReservation {
                worker: worker.clone(),
                slot: receipt.slot,
                slot_id,
                blob_path: directory.join("payload/blob.json"),
            });
        }
    }
    found.ok_or(WorkerStoreError::Unknown)
}

/// Appelé par le seul wrapper propriétaire du slot. Rejeu égal : lecture ;
/// divergence : erreur, aucun remplacement. Aucun fichier metadata n'est écrit.
pub fn publish_result_blob(
    reservation: &WorkerResultReservation,
    result: &RemoteResultEnvelope,
) -> Result<(), WorkerStoreError> {
    reservation.slot.validate()?;
    result.validate()?;
    if reservation.worker != result.worker
        || reservation.slot.identity != result.identity
        || reservation.slot_id != json_digest(&result.identity)?
    {
        return Err(WorkerStoreError::Conflict);
    }
    let payload = reservation
        .blob_path
        .parent()
        .ok_or(WorkerStoreError::Unknown)?;
    let slot_directory = payload.parent().ok_or(WorkerStoreError::Unknown)?;
    if reservation
        .blob_path
        .file_name()
        .is_none_or(|name| name != "blob.json")
        || payload.file_name().is_none_or(|name| name != "payload")
        || slot_directory
            .file_name()
            .is_none_or(|name| name != reservation.slot_id.as_str())
    {
        return Err(WorkerStoreError::Conflict);
    }
    check_private(slot_directory, true)?;
    check_private(payload, true)?;
    let slots = slot_directory.parent().ok_or(WorkerStoreError::Unknown)?;
    let root = slots.parent().ok_or(WorkerStoreError::Unknown)?;
    let state = root.parent().ok_or(WorkerStoreError::Unknown)?;
    if slots.file_name().is_none_or(|name| name != "slots") {
        return Err(WorkerStoreError::Conflict);
    }
    for path in [slots, root, state] {
        check_private(path, true)?;
    }
    let enrollment: Enrollment = read_optional(
        &state.join(format!(
            ".worker-{}.enrolled",
            digest_bytes(root.as_os_str().as_encoded_bytes())
        )),
        MAX_METADATA_BYTES,
    )?
    .ok_or(WorkerStoreError::Quarantined)?;
    if enrollment.quarantined
        || enrollment.version != STORE_VERSION
        || enrollment.worker != reservation.worker
    {
        return Err(WorkerStoreError::Quarantined);
    }
    let _publication = lock_file(&payload.join("publish.lock"), false)?;
    if fs::symlink_metadata(payload.join("conflict")).is_ok() {
        return Err(WorkerStoreError::Quarantined);
    }
    let receipt: SlotReceipt =
        read_optional(&slot_directory.join("receipt.json"), MAX_METADATA_BYTES)?
            .ok_or(WorkerStoreError::Unknown)?;
    if receipt.worker != reservation.worker
        || receipt.slot != reservation.slot
        || receipt.closed.is_some()
        || receipt.producer_terminal
    {
        return Err(WorkerStoreError::InvalidTransition);
    }
    if let Some(existing) = read_optional::<RemoteResultEnvelope>(
        &reservation.blob_path,
        MAX_WORKER_RESULT_ENVELOPE_BYTES,
    )? {
        return if existing == *result {
            Ok(())
        } else {
            // Le wrapper ne modifie pas les métadonnées du worker. Cette preuve
            // immutable lui permet de mettre lui-même le store en quarantaine.
            let conflict = payload.join("conflict");
            if fs::symlink_metadata(&conflict).is_err() {
                write_private_file_atomic(&conflict, b"result_conflict\n")?;
            }
            Err(WorkerStoreError::Conflict)
        };
    }
    // Un temporaire de publication antérieur ne doit pas coexister avec un
    // second blob de 2 Mio : issue inconnue, réservation conservée.
    if fs::read_dir(payload)?.count() != 1 {
        return Err(WorkerStoreError::Quarantined);
    }
    let bytes = serde_json::to_vec(result)
        .map_err(|_| WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope))?;
    if bytes.len() > MAX_WORKER_RESULT_ENVELOPE_BYTES {
        return Err(WorkerStoreError::Quota);
    }
    write_private_file_atomic(&reservation.blob_path, &bytes)?;
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn json_digest(value: &impl Serialize) -> Result<String, WorkerStoreError> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|_| WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope))
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn check_private(path: &Path, directory: bool) -> Result<(), WorkerStoreError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || metadata.uid() != bridget_transport::fsutil::effective_uid()
        || metadata.permissions().mode() & 0o077 != 0
        || (directory && !metadata.is_dir())
        || (!directory && (!metadata.is_file() || metadata.nlink() != 1))
    {
        return Err(WorkerStoreError::Quarantined);
    }
    Ok(())
}

fn private_dir(path: &Path) -> Result<(), WorkerStoreError> {
    match fs::symlink_metadata(path) {
        Ok(_) => check_private(path, true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create_private_dir(path)?;
            check_private(path, true)
        }
        Err(error) => Err(error.into()),
    }
}

// Le verrou appartient à cette garde, même si un fork/dup garde le même
// descripteur ouvert jusqu’à exec. Fermer le File seul ne suffit pas dans ce cas.
struct WorkerFileLock(File);
impl AsRawFd for WorkerFileLock {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.0.as_raw_fd()
    }
}
impl Drop for WorkerFileLock {
    fn drop(&mut self) {
        // SAFETY: les pointeurs passés à flock désignent des variables locales valides pendant l'appel.
        while unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) } != 0 {
            if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                break;
            }
        }
    }
}

fn lock_file(path: &Path, create: bool) -> Result<WorkerFileLock, WorkerStoreError> {
    check_private(path.parent().ok_or(WorkerStoreError::Unknown)?, true)?;
    if fs::symlink_metadata(path).is_ok() {
        check_private(path, false)?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    check_private(path, false)?;
    // SAFETY: les pointeurs passés à flock désignent des variables locales valides pendant l'appel.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let error = io::Error::last_os_error();
        if matches!(error.raw_os_error(), Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN)
        {
            return Err(WorkerStoreError::Locked);
        }
        return Err(error.into());
    }
    Ok(WorkerFileLock(file))
}

fn read_optional<T: DeserializeOwned>(
    path: &Path,
    max: usize,
) -> Result<Option<T>, WorkerStoreError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(_) => check_private(path, false)?,
    }
    check_private(path.parent().ok_or(WorkerStoreError::Unknown)?, true)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != bridget_transport::fsutil::effective_uid()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.nlink() != 1
        || metadata.len() > max as u64
    {
        return Err(WorkerStoreError::Quarantined);
    }
    let mut bytes = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(WorkerStoreError::Quarantined);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| WorkerStoreError::Quarantined)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), WorkerStoreError> {
    check_private(path.parent().ok_or(WorkerStoreError::Unknown)?, true)?;
    if fs::symlink_metadata(path).is_ok() {
        check_private(path, false)?;
    }
    let bytes = serde_json::to_vec(value)
        .map_err(|_| WorkerStoreError::Invalid(WorkerRefusal::InvalidEnvelope))?;
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(WorkerStoreError::Quota);
    }
    write_private_file_atomic(path, &bytes)?;
    Ok(())
}

fn private_tree_bytes(path: &Path) -> Result<u64, WorkerStoreError> {
    private_tree_bytes_bounded(path, 0, &mut 0)
}

fn private_tree_bytes_bounded(
    path: &Path,
    depth: usize,
    count: &mut usize,
) -> Result<u64, WorkerStoreError> {
    if depth > 4 {
        return Err(WorkerStoreError::Quarantined);
    }
    check_private(path, true)?;
    let mut bytes = 0u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        *count = count.saturating_add(1);
        if *count > MAX_WORKER_STORE_ENTRIES * 4 + 8 {
            return Err(WorkerStoreError::Quota);
        }
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir() {
            bytes =
                bytes.saturating_add(private_tree_bytes_bounded(&entry.path(), depth + 1, count)?);
        } else {
            check_private(&entry.path(), false)?;
            bytes = bytes.saturating_add(metadata.len());
        }
        if bytes > MAX_WORKER_STORE_BYTES as u64 {
            return Err(WorkerStoreError::Quota);
        }
    }
    Ok(bytes)
}

// Le runtime ne possède aucune Fleet ni base métier. Tous les messages viennent
// de la connexion du maître et sont corrélés à son époque authentifiée.
use crate::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, MarkedManagedChild, RunningManagedChild,
};
use crate::registry::{AgentRegistry, ProjectMapping};
use bridget_transport::protocol::{
    CandidateProbe, CandidateProbeCandidate, CandidateProbeRefusal, CandidateProbeResult,
    CandidateProbeV2, CandidateProbeV2Result, CandidateProbeV2Verdict, CandidateProbeVerdict,
    ClientCapability, DaemonToWrapper, IdentityCredential, MAX_WORKER_FRAME_BYTES,
    REMOTE_WORKER_VERSION, ResolvedAgentDefinition, SpawnPosture,
    WORKER_CANDIDATE_PROBE_V2_VERSION, WORKER_CANDIDATE_PROBE_VERSION, WorkerCancelScope,
    WorkerCapability, WorkerHello, WorkerHelloWithCapabilities, WorkerJournalFragment,
    WorkerObservation, WorkerPrepare, WorkerPrepared, WorkerRequest, WorkerRequestOperation,
    WorkerResponse, WorkerResponseOperation, WorkerStart, WrapperToDaemon,
};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct WorkerRuntimeConfig {
    pub machine_id: String,
    pub worker_id: String,
    pub provisioning_revision: u64,
    pub credential_path: PathBuf,
    pub state_root: PathBuf,
    pub registry_path: PathBuf,
    pub master_socket: PathBuf,
}

pub fn worker_command_digest(
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
    start: &WorkerStart,
) -> Result<String, WorkerStoreError> {
    worker.validate()?;
    command.validate()?;
    start.validate()?;
    json_digest(&(worker, command, start))
}

fn validate_prepare_command(command: &WorkerCommandIdentity) -> Result<(), WorkerRefusal> {
    if !command.instance_id.is_empty() || command.generation != 0 {
        return Err(WorkerRefusal::InvalidEnvelope);
    }
    let mut bounded = command.clone();
    bounded.instance_id = "prepare".into();
    bounded.generation = 1;
    bounded.validate()
}

fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !matches!(value, "." | "..")
        && !value.contains('/')
        && !value.chars().any(char::is_control)
}
fn worker_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}
fn worker_now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}
fn unavailable(_: impl std::fmt::Display) -> WorkerRefusal {
    WorkerRefusal::StoreUnavailable
}
fn workspace_failure(error: String) -> WorkerStoreError {
    WorkerStoreError::Workspace(match error.split(':').next() {
        Some("workspace_disk_full") => WorkerRefusal::DiskFull,
        Some("workspace_access_denied") => WorkerRefusal::FilesystemAccessDenied,
        _ => WorkerRefusal::WorkspacePrepareFailed,
    })
}
fn refuse(error: WorkerStoreError) -> WorkerRefusal {
    match error {
        WorkerStoreError::Quota => WorkerRefusal::ResourceLimit,
        WorkerStoreError::Workspace(reason) => reason,
        // Le magasin porte déjà un refus fermé et précis (permis ou créneau
        // expiré, connexion périmée, enveloppe, placement) : le transmettre.
        // Le masquer en StoreUnavailable faisait passer un permis expiré pour
        // un Release incertain côté maître (vu sous charge, remote162).
        WorkerStoreError::Invalid(reason) => reason,
        WorkerStoreError::Io(ref error)
            if matches!(error.raw_os_error(), Some(libc::ENOSPC | libc::EDQUOT)) =>
        {
            WorkerRefusal::DiskFull
        }
        WorkerStoreError::Io(ref error)
            if matches!(
                error.raw_os_error(),
                Some(libc::EACCES | libc::EPERM | libc::EROFS)
            ) =>
        {
            WorkerRefusal::FilesystemAccessDenied
        }
        _ => WorkerRefusal::StoreUnavailable,
    }
}
fn refuse_fenced_workspace(error: WorkerStoreError) -> WorkerRefusal {
    match error {
        WorkerStoreError::Conflict => WorkerRefusal::CommandMismatch,
        WorkerStoreError::Unknown => WorkerRefusal::UnknownExecution,
        error => refuse(error),
    }
}
#[cfg(test)]
mod refuse243_tests {
    use super::*;
    #[test]
    fn spec243_refus_precis_du_magasin_transmis_sans_masque_store_unavailable() {
        for reason in [
            WorkerRefusal::ReleaseExpired,
            WorkerRefusal::StaleConnection,
            WorkerRefusal::ResultSlotExpired,
            WorkerRefusal::PlacementMismatch,
            WorkerRefusal::InvalidEnvelope,
        ] {
            assert_eq!(refuse(WorkerStoreError::Invalid(reason)), reason);
            assert_eq!(
                refuse_fenced_workspace(WorkerStoreError::Invalid(reason)),
                reason
            );
        }
        // Les pannes réelles du magasin restent StoreUnavailable.
        for error in [
            WorkerStoreError::Locked,
            WorkerStoreError::Quarantined,
            WorkerStoreError::InvalidTransition,
        ] {
            assert_eq!(refuse(error), WorkerRefusal::StoreUnavailable);
        }
    }
}
fn marker_key(command: &WorkerCommandIdentity) -> String {
    digest_bytes(command.command_id.as_bytes())
}

struct PreparedRuntime {
    routing: Option<RoutingPreparation>,
    command: WorkerCommandIdentity,
    request: WorkerPrepare,
    prepared: WorkerPrepared,
    expires: Instant,
}

/// Faits de lecture communs à `Prepare` et au probe163. Cette structure ne
/// contient ni token, ni reçu, ni état de processus : elle ne permet donc pas
/// de transformer une sonde en admission.
struct CandidateReadiness {
    definition: ResolvedAgentDefinition,
    child_policy: serde_json::Value,
    effective_posture: SpawnPosture,
}

// Faits de chemins/Git seulement, jamais droits, modèle ou définition.
// Cache privé au lot v2 : au plus 128 clés, aucune persistance ni partage entre époques.
type ProbeGitFacts = BTreeMap<Vec<u8>, Result<(ProjectMapping, PathBuf), CandidateProbeRefusal>>;

struct ProbeReadPermit(Arc<AtomicBool>);
impl ProbeReadPermit {
    fn claim(slot: &Arc<AtomicBool>) -> Option<Self> {
        slot.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        Some(Self(slot.clone()))
    }
}
impl Drop for ProbeReadPermit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Le permis appartient au thread physique, même si son lien disparaît.
fn spawn_probe_read(
    permit: ProbeReadPermit,
    writer: &Arc<Mutex<UnixStream>>,
    disconnected: Arc<AtomicBool>,
    failure: WrapperToDaemon,
    work: impl FnOnce() -> WrapperToDaemon + Send + 'static,
) -> Result<(), String> {
    let reply = writer.clone();
    let stopped = disconnected.clone();
    let panic_refusal = failure.clone();
    match std::thread::Builder::new()
        .name("worker-probe-read".into())
        .spawn(move || {
            let _permit = permit;
            if stopped.load(Ordering::Acquire) {
                return;
            }
            let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
                .unwrap_or_else(|_| {
                    log::warn!("worker_probe_read_failed reason=panic");
                    panic_refusal
                });
            if stopped.load(Ordering::Acquire) {
                return;
            }
            if send_worker_frame(&reply, &response).is_err() {
                stopped.store(true, Ordering::Release);
            }
        }) {
        Ok(_) => Ok(()),
        Err(_) => send_worker_frame(writer, &failure),
    }
}

fn worker_probe_refusal(inbound: &WorkerInbound, reason: CandidateProbeRefusal) -> WrapperToDaemon {
    match inbound {
        WorkerInbound::CandidateProbe(probe) => WrapperToDaemon::CandidateProbeResult {
            result: CandidateProbeResult {
                version: probe.version,
                correlation_id: probe.correlation_id.clone(),
                connection_epoch: probe.connection_epoch.clone(),
                worker: probe.worker.clone(),
                admission_id: probe.admission_id.clone(),
                policy_revision: probe.policy_revision,
                verdicts: probe
                    .candidates
                    .iter()
                    .map(|c| CandidateProbeVerdict::Ineligible {
                        candidate_id: c.candidate_id.clone(),
                        reason,
                    })
                    .collect(),
            },
        },
        WorkerInbound::CandidateProbeV2(probe) => WrapperToDaemon::CandidateProbeResultV2 {
            result: CandidateProbeV2Result {
                tool_observations: Vec::new(),
                version: probe.version,
                correlation_id: probe.correlation_id.clone(),
                connection_epoch: probe.connection_epoch.clone(),
                worker: probe.worker.clone(),
                admission_id: probe.admission_id.clone(),
                policy_revision: probe.policy_revision,
                resources: HostResourceSnapshotV1 {
                    source_observed_at: worker_now(),
                    received_at: worker_now(),
                    cpu_percent: None,
                    available_memory_percent: None,
                    normalized_load_percent: None,
                    compilation_slots: None,
                },
                verdicts: probe
                    .candidates
                    .iter()
                    .map(|c| CandidateProbeV2Verdict::Ineligible {
                        candidate_id: c.candidate_id.clone(),
                        reason,
                    })
                    .collect(),
            },
        },
        _ => unreachable!("sonde seulement"),
    }
}

enum WorkerInbound {
    Request(WorkerRequest),
    CandidateProbe(CandidateProbe),
    CandidateProbeV2(CandidateProbeV2),
}

// Une file pleine produit une réponse corrélée et ne coupe pas les autres
// missions. La demande refusée n'entre jamais dans la file du consommateur.
fn worker_inbound_busy(inbound: WorkerInbound) -> WrapperToDaemon {
    match inbound {
        WorkerInbound::Request(
            request @ WorkerRequest {
                request: WorkerRequestOperation::StartDetailed { .. },
                ..
            },
        ) => WrapperToDaemon::WorkerResponse {
            response: WorkerResponse {
                version: REMOTE_WORKER_VERSION,
                correlation_id: request.correlation_id.clone(),
                connection_epoch: request.connection_epoch.clone(),
                worker: request.worker.clone(),
                command: request.command.clone(),
                response: WorkerResponseOperation::StartRejected {
                    reason: WorkerRefusal::ResourceLimit,
                    cause: if request.validate() == Err(WorkerRefusal::ResourceLimit) {
                        WorkerStartCapacityCause::RequestFrame
                    } else {
                        WorkerStartCapacityCause::InboundQueue
                    },
                },
            },
        },
        WorkerInbound::Request(request) => WrapperToDaemon::WorkerResponse {
            response: WorkerResponse {
                version: REMOTE_WORKER_VERSION,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                command: request.command,
                response: WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::ResourceLimit,
                },
            },
        },
        WorkerInbound::CandidateProbe(probe) => WrapperToDaemon::CandidateProbeResult {
            result: CandidateProbeResult {
                version: probe.version,
                correlation_id: probe.correlation_id,
                connection_epoch: probe.connection_epoch,
                worker: probe.worker,
                admission_id: probe.admission_id,
                policy_revision: probe.policy_revision,
                verdicts: probe
                    .candidates
                    .into_iter()
                    .map(|candidate| CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate.candidate_id,
                        reason: CandidateProbeRefusal::ResourceLimit,
                    })
                    .collect(),
            },
        },
        WorkerInbound::CandidateProbeV2(probe) => {
            let now = worker_now();
            WrapperToDaemon::CandidateProbeResultV2 {
                result: CandidateProbeV2Result {
                    tool_observations: Vec::new(),
                    version: probe.version,
                    correlation_id: probe.correlation_id,
                    connection_epoch: probe.connection_epoch,
                    worker: probe.worker,
                    admission_id: probe.admission_id,
                    policy_revision: probe.policy_revision,
                    resources: HostResourceSnapshotV1 {
                        source_observed_at: now,
                        received_at: now,
                        cpu_percent: None,
                        available_memory_percent: None,
                        normalized_load_percent: None,
                        compilation_slots: None,
                    },
                    verdicts: probe
                        .candidates
                        .into_iter()
                        .map(|candidate| CandidateProbeV2Verdict::Ineligible {
                            candidate_id: candidate.candidate_id,
                            reason: CandidateProbeRefusal::ResourceLimit,
                        })
                        .collect(),
                },
            }
        }
    }
}

const WORKER_PROBE_READ_BUDGET: Duration = Duration::from_secs(45);

fn dispatch_worker_probe(
    sender: &mpsc::SyncSender<(WorkerInbound, Instant)>,
    inbound: WorkerInbound,
    writer: &Arc<Mutex<UnixStream>>,
    disconnected: &AtomicBool,
) -> Result<(), String> {
    if disconnected.load(Ordering::Acquire) {
        return Err("worker_channel_stopping".into());
    }
    match sender.try_send((inbound, Instant::now())) {
        Ok(()) => Ok(()),
        Err(mpsc::TrySendError::Full((inbound, _))) => send_worker_frame(
            writer,
            &worker_probe_refusal(&inbound, CandidateProbeRefusal::ResourceLimit),
        ),
        Err(mpsc::TrySendError::Disconnected(_)) => Err("worker_probe_receiver_closed".into()),
    }
}

struct ProbeReadLane {
    receiver: mpsc::Receiver<(WorkerInbound, Instant)>,
    pending: Option<(WorkerInbound, Instant)>,
    slot: Arc<AtomicBool>,
}
impl ProbeReadLane {
    fn drive(
        &mut self,
        runtime: &WorkerRuntime,
        writer: &Arc<Mutex<UnixStream>>,
        disconnected: &Arc<AtomicBool>,
        workspace_diagnostics: bool,
    ) -> Result<(), String> {
        if self.pending.is_none() {
            self.pending = self.receiver.try_recv().ok();
        }
        let Some((inbound, received)) = self.pending.as_ref() else {
            return Ok(());
        };
        let deadline = *received + WORKER_PROBE_READ_BUDGET;
        if Instant::now() >= deadline {
            let refusal = worker_probe_refusal(inbound, CandidateProbeRefusal::ResourceLimit);
            self.pending.take();
            return send_worker_frame(writer, &refusal);
        }
        if self.slot.load(Ordering::Acquire) {
            return Ok(());
        }
        let (inbound, _) = self.pending.take().expect("sonde présente");
        runtime.dispatch_probe_read(
            inbound,
            &self.slot,
            writer,
            disconnected,
            workspace_diagnostics,
            deadline,
        )
    }
}

fn dispatch_worker_inbound(
    sender: &mpsc::SyncSender<WorkerInbound>,
    inbound: WorkerInbound,
    writer: &Arc<Mutex<UnixStream>>,
    disconnected: &AtomicBool,
    cancelled: &Mutex<BTreeSet<String>>,
) -> Result<(), String> {
    if disconnected.load(Ordering::Acquire) {
        return Err("worker_channel_stopping".into());
    }
    // L'annulation est vue avant la mise en file, même si celle-ci est pleine.
    if let WorkerInbound::Request(WorkerRequest {
        command,
        request:
            WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {},
            },
        ..
    }) = &inbound
    {
        let key = json_digest(command).map_err(|_| "worker_cancel_invalid")?;
        let mut cancelled = cancelled.lock().map_err(|_| "worker_cancel_poisoned")?;
        if cancelled.len() >= MAX_WORKER_STORE_ENTRIES {
            return Err("worker_cancel_capacity".into());
        }
        cancelled.insert(key);
    }
    match sender.try_send(inbound) {
        Ok(()) => Ok(()),
        Err(mpsc::TrySendError::Full(inbound)) => {
            send_worker_frame(writer, &worker_inbound_busy(inbound))
        }
        Err(mpsc::TrySendError::Disconnected(_)) => Err("worker_inbound_receiver_closed".into()),
    }
}

struct HeldRuntime {
    prepared: PreparedRuntime,
    mission_id: String,
    child: MarkedManagedChild,
    ready_at: i64,
    expires: Instant,
}
fn worker_stop_once(
    issued: &mut BTreeSet<String>,
    key: &str,
    stop: impl FnOnce() -> Result<(), WorkerRefusal>,
) -> Result<(), WorkerRefusal> {
    if !issued.contains(key) {
        stop()?;
        issued.insert(key.to_owned());
    }
    Ok(())
}

// Le diagnostic reste fermé : aucun détail Git ni chemin n'entre dans la trame.
fn workspace_probe_refusal(error: &str) -> CandidateProbeRefusal {
    match error.split(':').next() {
        Some("workspace_limit") => CandidateProbeRefusal::WorkspaceLimit,
        _ => CandidateProbeRefusal::WorkspaceUnavailable,
    }
}

// Un maître ancien garde le refus générique qu'il sait décoder.
fn program_probe_reason_for_master(
    reason: CandidateProbeRefusal,
    supported: bool,
) -> CandidateProbeRefusal {
    if !supported
        && matches!(
            reason,
            CandidateProbeRefusal::ProgramUpgradeCheckPending
                | CandidateProbeRefusal::ProgramProtocolIncompatible
                | CandidateProbeRefusal::ProgramCheckUnavailable
        )
    {
        CandidateProbeRefusal::DefinitionUnavailable
    } else {
        reason
    }
}

fn workspace_probe_reason_for_master(
    reason: CandidateProbeRefusal,
    supports_workspace_diagnostics: bool,
) -> CandidateProbeRefusal {
    if reason == CandidateProbeRefusal::WorkspaceLimit && !supports_workspace_diagnostics {
        CandidateProbeRefusal::WorkspaceUnavailable
    } else {
        reason
    }
}

/// Forme et empreinte partagées ; l'appelant authentifie la session et relit
/// ses autorités avant tout effet. Cette fonction n'accorde aucun droit.
pub(crate) fn commit_view_preparation_binding(
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
    request: &WorkerPrepare,
) -> Result<crate::delegation::ViewBinding, WorkerRefusal> {
    worker.validate()?;
    command.validate_preparation()?;
    request.validate()?;
    if request.machine_id != worker.machine_id
        || request.provisioning_revision != worker.provisioning_revision
        || !matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        )
        || request.posture != Some(SpawnPosture::Discovery)
    {
        return Err(WorkerRefusal::InvalidEnvelope);
    }
    Ok(crate::delegation::ViewBinding {
        worker: worker.clone(),
        command: command.clone(),
        prepare_digest: json_digest(&(command, request)).map_err(refuse)?,
    })
}

type ViewFileError = (WorkerRefusal, Option<crate::workspace_git::ExportFailure>);
fn view_file_error(error: WorkerStoreError) -> ViewFileError {
    let failure = match &error {
        WorkerStoreError::Quota => Some(crate::workspace_git::ExportFailure::MetadataCapacity),
        WorkerStoreError::Io(error) => match error.raw_os_error() {
            Some(libc::ENOSPC | libc::EDQUOT) => {
                Some(crate::workspace_git::ExportFailure::DiskFull)
            }
            Some(libc::EACCES | libc::EPERM | libc::EROFS) => {
                Some(crate::workspace_git::ExportFailure::AccessDenied)
            }
            _ => None,
        },
        _ => None,
    };
    (
        match error {
            WorkerStoreError::Conflict => WorkerRefusal::CommandMismatch,
            WorkerStoreError::Unknown | WorkerStoreError::InvalidTransition => {
                WorkerRefusal::UnknownExecution
            }
            WorkerStoreError::Locked | WorkerStoreError::Quota => WorkerRefusal::ResourceLimit,
            WorkerStoreError::Invalid(reason) => reason,
            error => refuse(error),
        },
        failure,
    )
}

/// Jeton interne, non sérialisé et sans copie du manifeste. Le futur port
/// authentifie la mission avant begin et conserve ce jeton pour ses polls.
struct ViewFileRead {
    binding: crate::workspace_git::ViewBinding,
    execution: WorkerResultIdentity,
    request: crate::workspace_git::ViewJobRequest,
}

struct ViewSourceRead {
    binding: crate::workspace_git::ViewBinding,
}

/// Contexte privé du constructeur, jamais sérialisé ni fourni par un client.
/// L'appelant authentifie la commande du maître avant begin ; aucun droit de
/// Start ne découle de ce jeton tant que poll n'a pas publié WorkerPrepared.
struct ViewPrepareRead {
    source: ViewSourceRead,
    request: WorkerPrepare,
    routing: Option<RoutingPreparation>,
    definition_digest: String,
    permission_digest: String,
    expires: Instant,
}

// Le runtime reste seul propriétaire. Seules les corrélations du lien courant
// attendent une réponse ; la graine et son échéance survivent à la reconnexion.
struct PendingViewPreparation {
    operation_digest: String,
    read: ViewPrepareRead,
    epoch: String,
    correlations: BTreeSet<String>,
}

struct ViewFileAudit {
    binding: crate::workspace_git::ViewBinding,
    execution: WorkerResultIdentity,
    request: crate::workspace_git::ViewJobRequest,
    state: ViewFileAuditState,
}

// Le constat terminé reste possédé jusqu'au CAS. Une panne d'écriture ne
// recommence ni l'audit des données ni le helper de matérialisation.
enum ViewFileAuditState {
    Reading(
        std::thread::JoinHandle<(
            crate::workspace_git::ViewIntent,
            Result<bool, crate::workspace_git::ExportFailure>,
        )>,
    ),
    Ready {
        intent: crate::workspace_git::ViewIntent,
        available: Option<bool>,
        failure: Option<crate::workspace_git::ExportFailure>,
        retry_at: Instant,
        last_error: Option<String>,
    },
}

struct PendingViewFile {
    request: WorkerRequest,
    read: ViewFileRead,
    expires: Instant,
    correlations: BTreeSet<String>,
}

struct PendingViewControl {
    request: WorkerRequest,
    operation_digest: String,
    expires: Instant,
    correlations: BTreeSet<String>,
    reader: std::thread::JoinHandle<Result<crate::delegation::WorkspaceRecord, WorkerRefusal>>,
}

struct ViewReportRead {
    binding: crate::workspace_git::ViewBinding,
    expires: Instant,
    reader: std::thread::JoinHandle<
        Result<
            (
                crate::workspace_git::ViewIntent,
                crate::delegation::WorkspaceRecord,
            ),
            String,
        >,
    >,
}

struct ViewSourceProcess {
    binding: crate::workspace_git::ViewBinding,
    child: crate::workspace_git::ViewJob,
    materialization: Option<(WorkerResultIdentity, crate::workspace_git::ViewJobRequest)>,
}

struct WorkerRuntime {
    config: WorkerRuntimeConfig,
    identity: WorkerIdentity,
    credential: IdentityCredential,
    store: WorkerStore,
    markers: ManagedMarkerStore,
    source: crate::lifecycle::SourceEnvironment,
    prepared: BTreeMap<String, PreparedRuntime>,
    held: BTreeMap<String, HeldRuntime>,
    running: BTreeMap<String, RunningManagedChild>,
    abandoned: BTreeMap<String, std::process::Child>,
    view_sources: BTreeMap<String, ViewSourceProcess>,
    view_reports: BTreeMap<String, ViewReportRead>,
    view_preparations: BTreeMap<String, PendingViewPreparation>,
    view_controls: BTreeMap<String, PendingViewControl>,
    view_files: BTreeMap<String, PendingViewFile>,
    view_file_audits: BTreeMap<String, ViewFileAudit>,
    last_maintenance: Instant,
    cancelled: Arc<Mutex<BTreeSet<String>>>,
    cancel_signal_issued: BTreeSet<String>,
    host_resources: crate::host_resources::HostResourceCollector,
    last_resource_refresh: Instant,
    provider_accounts_negotiated: bool,
    program_compatibility_negotiated: bool,
    permission_reasons_negotiated: bool,
}

impl WorkerRuntime {
    /// Empreinte seule sur le fil ; la liaison et les chemins viennent du reçu
    /// privé. Toute preuve manquante conserve les données et refuse la clôture.
    fn conserve_commit_view(
        &self,
        command: &WorkerCommandIdentity,
        prepare_digest: &str,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        validate_prepare_command(command)?;
        let binding = crate::workspace_git::ViewBinding {
            worker: self.identity.clone(),
            command: command.clone(),
            prepare_digest: prepare_digest.into(),
        };
        if proof::read_workspace_receipt(&self.store.root, &self.identity, command)
            .map_err(refuse)?
            .is_none()
        {
            if self
                .view_sources
                .values()
                .any(|source| source.binding.command.task_id == command.task_id)
                || self
                    .view_reports
                    .values()
                    .any(|report| report.binding.command.task_id == command.task_id)
            {
                return Err(WorkerRefusal::UnknownExecution);
            }
            self.store
                .conserve_unreserved_view(&binding)
                .map_err(refuse_fenced_workspace)?;
            return Ok(WorkerResponseOperation::CommitViewConserved {
                prepare_digest: prepare_digest.into(),
            });
        }
        let receipt = self
            .store
            .view_receipt_for_binding(&binding)
            .map_err(refuse)?;
        if receipt.record.is_none() {
            self.store
                .conserve_unpublished_view(&binding)
                .map_err(refuse)?;
            return Ok(WorkerResponseOperation::CommitViewConserved {
                prepare_digest: prepare_digest.into(),
            });
        }
        let proofs = crate::workspace_git::ViewProofs::open(
            &self.store.view_root().canonicalize().map_err(unavailable)?,
        )
        .map_err(|_| WorkerRefusal::WorkspacePrepareFailed)?;
        let intent = proofs
            .load(&binding.key().map_err(|_| WorkerRefusal::InvalidEnvelope)?)
            .map_err(|_| WorkerRefusal::WorkspacePrepareFailed)?;
        if intent.binding != binding {
            return Err(WorkerRefusal::CommandMismatch);
        }
        // La conservation fige les écritures, sans audit de512Mio dans la
        // boucle worker. La réduction reste une opération privée séparée.
        self.store.freeze_view(command, &intent).map_err(refuse)?;
        Ok(WorkerResponseOperation::CommitViewConserved {
            prepare_digest: prepare_digest.into(),
        })
    }

    /// Begin ne fait aucune attente de fin d'export. Le port natif conserve
    /// le petit jeton et revient au prochain tick ; aucune mission n'est créée.
    #[cfg(test)]
    fn begin_view_source(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        intent: crate::workspace_git::ViewIntent,
    ) -> Result<ViewSourceRead, ViewFileError> {
        let cancel_key = json_digest(command).map_err(view_file_error)?;
        let cancelled = self
            .cancelled
            .lock()
            .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?
            .contains(&cancel_key);
        if cancelled {
            return Err((
                WorkerRefusal::WorkspacePrepareFailed,
                Some(crate::workspace_git::ExportFailure::Cancelled),
            ));
        }
        self.store
            .reserve_view_intent(command, request, &intent)
            .map_err(view_file_error)?;
        let key = marker_key(command);
        let read = ViewSourceRead {
            binding: intent.binding.clone(),
        };
        let receipt = self
            .store
            .checked_view_receipt(command, &intent)
            .map_err(view_file_error)?;
        let state = receipt.commit_view.as_ref().unwrap();
        if state.closed {
            return Err((WorkerRefusal::UnknownExecution, None));
        }
        if let Some(cause) = state.source_failure {
            return Err((WorkerRefusal::WorkspacePrepareFailed, Some(cause)));
        }
        if !self.view_sources.contains_key(&key)
            && state.source_attempted
            && !state.source_stopped
            && state.source_marker.is_none()
        {
            return Err((
                WorkerRefusal::UnknownExecution,
                Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
            ));
        }
        if !self.view_sources.contains_key(&key)
            && (receipt.record.is_some() || state.source_marker.is_some() || state.source_stopped)
        {
            return Ok(read);
        }
        if !self.view_sources.contains_key(&key) {
            let proofs = crate::workspace_git::ViewProofs::open(
                &self
                    .store
                    .view_root()
                    .canonicalize()
                    .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
            )
            .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
            let executable = std::env::current_exe().map_err(|_| {
                (
                    WorkerRefusal::WorkerUnavailable,
                    Some(crate::workspace_git::ExportFailure::StartupUnavailable),
                )
            })?;
            let store = &self.store;
            let cancelled = &self.cancelled;
            match crate::workspace_git::ViewJob::start_registered(
                &proofs,
                &intent,
                &crate::workspace_git::ViewJobRequest::Export,
                &executable,
                &self.source,
                || {
                    store
                        .mark_view_source_attempt(command, &intent)
                        .map_err(|_| crate::workspace_git::ExportFailure::ProofUnavailable)
                },
                |marker| {
                    if cancelled
                        .lock()
                        .map_err(|_| crate::workspace_git::ExportFailure::ProofUnavailable)?
                        .contains(&cancel_key)
                    {
                        return Err(crate::workspace_git::ExportFailure::Cancelled);
                    }
                    store
                        .authorize_view_source(command, &intent, marker)
                        .map_err(|_| crate::workspace_git::ExportFailure::ProofUnavailable)
                },
            ) {
                Ok(child) => {
                    self.view_sources.insert(
                        key,
                        ViewSourceProcess {
                            binding: intent.binding.clone(),
                            child,
                            materialization: None,
                        },
                    );
                }
                Err((cause, retained)) => {
                    if let Some(child) = retained {
                        self.view_sources.insert(
                            key,
                            ViewSourceProcess {
                                binding: intent.binding.clone(),
                                child,
                                materialization: None,
                            },
                        );
                    } else {
                        self.store
                            .record_view_source_exit(&intent.binding, Some(Err(cause)))
                            .map_err(view_file_error)?;
                    }
                    return Err((WorkerRefusal::WorkspacePrepareFailed, Some(cause)));
                }
            }
        }
        Ok(read)
    }

    fn begin_planned_view_source(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        seed: crate::workspace_git::ViewPlan,
    ) -> Result<ViewSourceRead, ViewFileError> {
        if !self
            .store
            .view_preparation_unused(command)
            .map_err(view_file_error)?
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        let cancel_key = json_digest(command).map_err(view_file_error)?;
        if self
            .cancelled
            .lock()
            .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?
            .contains(&cancel_key)
        {
            return Err((
                WorkerRefusal::WorkspacePrepareFailed,
                Some(crate::workspace_git::ExportFailure::Cancelled),
            ));
        }
        self.store
            .reserve_view_plan(command, request, &seed)
            .map_err(view_file_error)?;
        let key = marker_key(command);
        let read = ViewSourceRead {
            binding: seed.binding.clone(),
        };
        let receipt = self
            .store
            .view_receipt_for_binding(&seed.binding)
            .map_err(view_file_error)?;
        let state = receipt.commit_view.as_ref().unwrap();
        if state.closed {
            return Err((WorkerRefusal::UnknownExecution, None));
        }
        if let Some(cause) = state.source_failure {
            return Err((WorkerRefusal::WorkspacePrepareFailed, Some(cause)));
        }
        if self.view_sources.contains_key(&key)
            || receipt.record.is_some()
            || state.source_stopped
            || state.source_marker.is_some()
        {
            return Ok(read);
        }
        if state.source_attempted {
            return Err((
                WorkerRefusal::UnknownExecution,
                Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
            ));
        }
        // Un seul export natif à la fois. Les graines suivantes attendent sous
        // leur échéance d'origine ; un marqueur inconnu ne libère pas le créneau.
        if !self.view_writer_available(&seed.binding, &key)? {
            return Ok(read);
        }
        let proofs = crate::workspace_git::ViewProofs::open(
            &self
                .store
                .view_root()
                .canonicalize()
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
        )
        .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
        let executable =
            std::env::current_exe().map_err(|_| (WorkerRefusal::WorkerUnavailable, None))?;
        let store = &self.store;
        let cancelled = &self.cancelled;
        match crate::workspace_git::ViewJob::start_plan_registered(
            &proofs,
            &seed,
            &executable,
            &self.source,
            || {
                store
                    .mark_view_source_attempt_binding(&seed.binding)
                    .map_err(|_| crate::workspace_git::ExportFailure::ProofUnavailable)
            },
            |marker| {
                if cancelled
                    .lock()
                    .map_err(|_| crate::workspace_git::ExportFailure::ProofUnavailable)?
                    .contains(&cancel_key)
                {
                    return Err(crate::workspace_git::ExportFailure::Cancelled);
                }
                store
                    .authorize_view_source_binding(&seed.binding, marker)
                    .map_err(|_| crate::workspace_git::ExportFailure::ProofUnavailable)
            },
        ) {
            Ok(child) => {
                self.view_sources.insert(
                    key,
                    ViewSourceProcess {
                        binding: seed.binding,
                        child,
                        materialization: None,
                    },
                );
            }
            Err((cause, retained)) => {
                if let Some(child) = retained {
                    self.view_sources.insert(
                        key,
                        ViewSourceProcess {
                            binding: seed.binding,
                            child,
                            materialization: None,
                        },
                    );
                } else {
                    self.store
                        .record_view_source_exit(&seed.binding, Some(Err(cause)))
                        .map_err(view_file_error)?;
                }
                return Err((WorkerRefusal::WorkspacePrepareFailed, Some(cause)));
            }
        }
        Ok(read)
    }

    /// O(1) petits reçus pendant l'export. L'audit final reste borné et ne
    /// survient qu'après la preuve d'arrêt, jamais à chaque poll en cours.
    fn poll_view_source(
        &mut self,
        read: &ViewSourceRead,
    ) -> Result<Option<crate::delegation::WorkspaceRecord>, ViewFileError> {
        let command = &read.binding.command;
        let receipt = self
            .store
            .view_receipt_for_binding(&read.binding)
            .map_err(view_file_error)?;
        let state = receipt.commit_view.as_ref().unwrap();
        if state.closed {
            return Err((WorkerRefusal::UnknownExecution, None));
        }
        let cancel_key = json_digest(command).map_err(view_file_error)?;
        let cancelled = self
            .cancelled
            .lock()
            .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?
            .contains(&cancel_key);
        let key = marker_key(command);
        if let Some(source) = self.view_sources.get_mut(&key) {
            if source.binding != read.binding || source.materialization.is_some() {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            if cancelled {
                let _ = source.child.abort(
                    crate::workspace_git::ExportFailure::Cancelled,
                    Duration::ZERO,
                );
            }
            match source
                .child
                .poll()
                .map_err(|_| (WorkerRefusal::UnknownExecution, None))?
            {
                crate::workspace_git::JobPoll::Running => return Ok(None),
                crate::workspace_git::JobPoll::StopUnconfirmed => {
                    return Err((
                        WorkerRefusal::UnknownExecution,
                        Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
                    ));
                }
                crate::workspace_git::JobPoll::Stopped(result) => {
                    if !self
                        .store
                        .record_view_source_exit(&source.binding, Some(result))
                        .map_err(view_file_error)?
                    {
                        return Err((WorkerRefusal::UnknownExecution, None));
                    }
                    self.view_sources.remove(&key);
                }
            }
        } else if !state.source_stopped
            && !self
                .store
                .confirm_view_sources_stopped(&command.task_id)
                .map_err(view_file_error)?
        {
            return Err((
                WorkerRefusal::UnknownExecution,
                Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
            ));
        }
        if cancelled {
            return Err((
                WorkerRefusal::WorkspacePrepareFailed,
                Some(crate::workspace_git::ExportFailure::Cancelled),
            ));
        }
        let receipt = self
            .store
            .view_receipt_for_binding(&read.binding)
            .map_err(view_file_error)?;
        if let Some(cause) = receipt.commit_view.as_ref().unwrap().source_failure {
            return Err((WorkerRefusal::WorkspacePrepareFailed, Some(cause)));
        }
        if !receipt.commit_view.as_ref().unwrap().source_stopped {
            return Err((
                WorkerRefusal::UnknownExecution,
                Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
            ));
        }
        if let Some(report) = self.view_reports.get(&key) {
            if report.binding != read.binding {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            if !report.reader.is_finished() {
                return Ok(None);
            }
            let report = self.view_reports.remove(&key).unwrap();
            let (intent, record) = report
                .reader
                .join()
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
            if intent.binding != read.binding {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            let result =
                self.store
                    .publish_view_snapshot(command, &intent, record, &self.cancelled);
            if result.is_err()
                && self
                    .cancelled
                    .lock()
                    .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?
                    .contains(&cancel_key)
            {
                return Err((
                    WorkerRefusal::WorkspacePrepareFailed,
                    Some(crate::workspace_git::ExportFailure::Cancelled),
                ));
            }
            return result.map(Some).map_err(view_file_error);
        }
        let proofs = crate::workspace_git::ViewProofs::open(
            &self
                .store
                .view_root()
                .canonicalize()
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
        )
        .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
        let intent = proofs
            .load(
                &read
                    .binding
                    .key()
                    .map_err(|_| (WorkerRefusal::CommandMismatch, None))?,
            )
            .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
        if receipt
            .commit_view
            .as_ref()
            .is_some_and(|view| view.planning.is_some())
        {
            self.store
                .adopt_view_plan(&read.binding, &intent)
                .map_err(view_file_error)?;
        }
        self.store
            .prepare_view_workspace_publication(command, &intent)
            .map_err(view_file_error)?;
        let private = self
            .store
            .view_root()
            .canonicalize()
            .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
        #[cfg(test)]
        let hold_report = self.source.contains_key("B221_VIEW_TEST_REPORT_HOLD");
        let reader = std::thread::Builder::new()
            .name("bridget-view-audit".into())
            .spawn(move || {
                #[cfg(test)]
                if hold_report {
                    let deadline = Instant::now() + Duration::from_secs(10);
                    while !private.join("allow-report").exists() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
                let record =
                    crate::workspace_git::ViewProofs::open(&private)?.workspace(&intent)?;
                Ok((intent, record))
            })
            .map_err(|_| (WorkerRefusal::WorkerUnavailable, None))?;
        self.view_reports.insert(
            key,
            ViewReportRead {
                binding: read.binding.clone(),
                expires: Instant::now() + Duration::from_secs(120),
                reader,
            },
        );
        Ok(None)
    }

    #[cfg(test)]
    fn prepare_view_source(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        intent: crate::workspace_git::ViewIntent,
    ) -> Result<crate::delegation::WorkspaceRecord, ViewFileError> {
        let read = self.begin_view_source(command, request, intent)?;
        loop {
            if let Some(record) = self.poll_view_source(&read)? {
                return Ok(record);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Un seul écrivain de données C19 par worker, y compris après coupure.
    /// Les lecteurs d'audit n'écrivent pas ; un marqueur inconnu garde le créneau.
    fn view_writer_available(
        &self,
        binding: &crate::workspace_git::ViewBinding,
        operation: &str,
    ) -> Result<bool, ViewFileError> {
        if self.view_sources.keys().any(|key| key != operation) {
            return Ok(false);
        }
        Ok(!self
            .store
            .view_receipts(None)
            .map_err(view_file_error)?
            .iter()
            .any(|receipt| {
                receipt.command != binding.command
                    && receipt.commit_view.as_ref().is_some_and(|view| {
                        (view.source_attempted && !view.source_stopped)
                            || view
                                .materialization
                                .as_ref()
                                .is_some_and(|file| file.attempted && !file.stopped)
                    })
            }))
    }

    fn native_view_plan(
        &self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
    ) -> Result<crate::workspace_git::ViewPlan, ViewFileError> {
        validate_prepare_command(command).map_err(|reason| (reason, None))?;
        request.validate().map_err(|reason| (reason, None))?;
        let source_root = Path::new(&request.remote_root);
        let data_root = crate::workspace_git::ViewPlan::native_data_root(source_root)
            .map_err(|_| (WorkerRefusal::PlacementMismatch, None))?;
        crate::registry::validate_unprotected_local_path(source_root, &self.protected())
            .map_err(|_| (WorkerRefusal::PlacementMismatch, None))?;
        if self
            .protected()
            .iter()
            .any(|path| data_root.starts_with(path))
        {
            return Err((WorkerRefusal::PlacementMismatch, None));
        }
        let binding = commit_view_preparation_binding(&self.identity, command, request)
            .map_err(|reason| (reason, None))?;
        // Une graine persistée est l'unique échéance au rejeu.
        if proof::read_workspace_receipt(&self.store.root, &self.identity, command)
            .map_err(view_file_error)?
            .is_some()
        {
            let proofs = crate::workspace_git::ViewProofs::open(
                &self
                    .store
                    .view_root()
                    .canonicalize()
                    .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
            )
            .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
            let seed = proofs
                .load_plan(
                    &binding
                        .key()
                        .map_err(|_| (WorkerRefusal::CommandMismatch, None))?,
                )
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
            if seed.binding != binding || seed.data_root != data_root || !seed.native_placement_v1 {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            return Ok(seed);
        }
        crate::workspace_git::ViewPlan::new_native(
            binding,
            &source_root.join(&request.relative_cwd),
            request
                .base_commit
                .as_deref()
                .ok_or((WorkerRefusal::InvalidEnvelope, None))?,
            source_root,
            &request.source_root_identity,
        )
        .map_err(|_| (WorkerRefusal::InvalidEnvelope, None))
    }

    fn begin_prepared_view(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        routing: Option<(RoutingPreparation, &str, &str)>,
        seed: crate::workspace_git::ViewPlan,
    ) -> Result<ViewPrepareRead, ViewFileError> {
        validate_prepare_command(command).map_err(|r| (r, None))?;
        request.validate().map_err(|r| (r, None))?;
        if !matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) || request.posture != Some(SpawnPosture::Discovery)
        {
            return Err((WorkerRefusal::InvalidEnvelope, None));
        }
        if seed.binding.worker != self.identity
            || seed.binding.command != *command
            || seed.binding.prepare_digest
                != json_digest(&(command, request)).map_err(view_file_error)?
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        if !self
            .store
            .view_preparation_unused(command)
            .map_err(view_file_error)?
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        if !self.held.is_empty()
            || (self.prepared.len() >= 128 && !self.prepared.contains_key(&command.command_id))
        {
            return Err((
                WorkerRefusal::ResourceLimit,
                Some(if self.held.is_empty() {
                    crate::workspace_git::ExportFailure::MetadataCapacity
                } else {
                    crate::workspace_git::ExportFailure::OperationCapacity
                }),
            ));
        }
        let registry = self.registry().map_err(|r| (r, None))?;
        if let Some((selection, _, _)) = &routing {
            self.check_routing_connection(&registry, request, selection)
                .map_err(|r| (r, None))?;
        }
        let mapping = self
            .mapping_authority(request, &registry)
            .map_err(|r| (r, None))?;
        let (mapping, cwd) = self
            .read_view()
            .mapping_paths(request, mapping)
            .map_err(|r| (r, None))?;
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let initial = self
            .inspect_candidate(&registry, request, &mapping, &cwd, &mut operation, None)
            .map_err(|r| (r, None))?;
        operation
            .finish()
            .map_err(|_| (WorkerRefusal::PermissionsChanged, None))?;
        if initial.effective_posture != SpawnPosture::Discovery {
            return Err((WorkerRefusal::PermissionsChanged, None));
        }
        let permission_digest = json_digest(&initial.child_policy).map_err(view_file_error)?;
        if let Some((_, definition, permission)) = &routing {
            if initial.definition.digest != *definition {
                return Err((WorkerRefusal::DefinitionChanged, None));
            }
            if permission_digest != *permission {
                return Err((WorkerRefusal::PermissionsChanged, None));
            }
        }
        let expires = Instant::now()
            + seed
                .remaining()
                .map_err(|_| (WorkerRefusal::PrepareExpired, None))?;
        let source = self.begin_planned_view_source(command, request, seed)?;
        Ok(ViewPrepareRead {
            source,
            request: request.clone(),
            routing: routing.map(|(selection, _, _)| selection),
            definition_digest: initial.definition.digest,
            permission_digest,
            expires,
        })
    }

    /// La boucle réseau a déjà validé identité, époque et capacité créateur.
    /// Aucun travail lourd n'attend dans cette boucle : begin possède le
    /// sous-processus, puis poll récolte seulement les lecteurs terminés.
    fn defer_view_preparation(&mut self, request: &WorkerRequest) -> Result<bool, ViewFileError> {
        let operation = match &request.request {
            WorkerRequestOperation::PrepareDiagnosed { prepare } => prepare.as_ref(),
            other => other,
        };
        let (prepare, routing) = match operation {
            WorkerRequestOperation::Prepare { prepare } => (prepare, None),
            WorkerRequestOperation::PrepareV2 {
                prepare,
                inspect_only: false,
                workload,
                connection_id,
                engine,
                expected_definition_digest,
                expected_permission_digest,
            } => (
                prepare,
                Some((
                    RoutingPreparation {
                        connection_id: connection_id.clone(),
                        engine: engine.clone(),
                        workload: *workload,
                    },
                    expected_definition_digest.as_str(),
                    expected_permission_digest.as_str(),
                )),
            ),
            _ => return Ok(false),
        };
        if !matches!(
            prepare.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        ) {
            return Ok(false);
        }
        request.validate().map_err(|reason| (reason, None))?;
        if request.worker != self.identity {
            return Err((WorkerRefusal::PlacementMismatch, None));
        }
        let key = marker_key(&request.command);
        let digest = json_digest(&request.request).map_err(view_file_error)?;
        if let Some(pending) = self.view_preparations.get_mut(&key) {
            if pending.read.source.binding.command != request.command
                || pending.operation_digest != digest
            {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            if Instant::now() >= pending.read.expires {
                return Err((
                    WorkerRefusal::PrepareExpired,
                    Some(crate::workspace_git::ExportFailure::PreparationExpired),
                ));
            }
            if pending.epoch != request.connection_epoch {
                pending.correlations.clear();
                pending.epoch = request.connection_epoch.clone();
            }
            if pending.correlations.len() >= 8
                && !pending.correlations.contains(&request.correlation_id)
            {
                return Err((
                    WorkerRefusal::ResourceLimit,
                    Some(crate::workspace_git::ExportFailure::OperationCapacity),
                ));
            }
            pending.correlations.insert(request.correlation_id.clone());
            return Ok(true);
        }
        if self.view_preparations.len() >= 128 {
            return Err((
                WorkerRefusal::ResourceLimit,
                Some(crate::workspace_git::ExportFailure::OperationCapacity),
            ));
        }
        let seed = self.native_view_plan(&request.command, prepare)?;
        let read = self.begin_prepared_view(&request.command, prepare, routing, seed)?;
        self.view_preparations.insert(
            key,
            PendingViewPreparation {
                operation_digest: digest,
                read,
                epoch: request.connection_epoch.clone(),
                correlations: BTreeSet::from([request.correlation_id.clone()]),
            },
        );
        Ok(true)
    }

    fn view_prepare_rejection((reason, failure): ViewFileError) -> WorkerResponseOperation {
        // Le pair sait lire la cause fermée. Une erreur sans preuve reste inconnue.
        WorkerResponseOperation::CommitViewRejected {
            reason,
            failure: failure.unwrap_or(crate::workspace_git::ExportFailure::Unknown),
        }
    }

    fn poll_view_preparations(&mut self, epoch: &str) -> Vec<WorkerResponse> {
        let keys = self.view_preparations.keys().cloned().collect::<Vec<_>>();
        let mut responses = Vec::new();
        for key in keys {
            let mut pending = self.view_preparations.remove(&key).unwrap();
            if pending.epoch != epoch {
                pending.correlations.clear();
                pending.epoch = epoch.into();
            }
            let response = match self.poll_prepared_view(&pending.read) {
                Ok(None) => {
                    self.view_preparations.insert(key, pending);
                    continue;
                }
                Ok(Some(prepared)) => WorkerResponseOperation::Prepared { prepared },
                Err(error) => Self::view_prepare_rejection(error),
            };
            for correlation_id in pending.correlations {
                responses.push(WorkerResponse {
                    version: REMOTE_WORKER_VERSION,
                    correlation_id,
                    connection_epoch: epoch.into(),
                    worker: self.identity.clone(),
                    command: pending.read.source.binding.command.clone(),
                    response: response.clone(),
                });
            }
        }
        responses
    }

    /// Audit de fichiers seul ; le lecteur ne reçoit ni store ni fournisseur.
    /// Start/Release relisent ensuite leurs autorités avant tout effet.
    fn defer_view_control(&mut self, request: &WorkerRequest) -> Result<bool, WorkerRefusal> {
        let (value, expires) = match &request.request {
            WorkerRequestOperation::Start { .. } | WorkerRequestOperation::StartDetailed { .. } => {
                if !matches!(
                    self.store.command(&request.command),
                    Err(WorkerStoreError::Unknown)
                ) {
                    return Ok(false);
                }
                let Some(prepared) = self.prepared.get(&request.command.command_id) else {
                    return Ok(false);
                };
                if !matches!(
                    prepared.request.workspace,
                    Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
                ) {
                    return Ok(false);
                }
                (
                    prepared
                        .prepared
                        .workspace
                        .clone()
                        .ok_or(WorkerRefusal::CommandMismatch)?,
                    prepared.expires,
                )
            }
            WorkerRequestOperation::Release { .. } => {
                let Some(held) = self.held.get(&request.command.command_id) else {
                    return Ok(false);
                };
                if !matches!(
                    held.prepared.request.workspace,
                    Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
                ) {
                    return Ok(false);
                }
                (
                    held.prepared
                        .prepared
                        .workspace
                        .clone()
                        .ok_or(WorkerRefusal::CommandMismatch)?,
                    held.expires,
                )
            }
            WorkerRequestOperation::PrepareFenced {
                previous,
                prepare,
                placement,
                workspace: Some(workspace),
                ..
            } if matches!(
                prepare.workspace,
                Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
            ) =>
            {
                let original = self
                    .store
                    .fenced_workspace(previous, placement, workspace)
                    .map_err(refuse_fenced_workspace)?;
                (
                    serde_json::to_value(original).map_err(unavailable)?,
                    self.prepared
                        .get(&request.command.command_id)
                        .map(|prepared| prepared.expires)
                        .unwrap_or_else(|| Instant::now() + Duration::from_secs(30)),
                )
            }
            _ => return Ok(false),
        };
        request.validate()?;
        if request.worker != self.identity {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        if Instant::now() >= expires {
            return Err(WorkerRefusal::PrepareExpired);
        }
        let key = json_digest(&request.command).map_err(refuse)?;
        let digest = json_digest(&request.request).map_err(refuse)?;
        if let Some(pending) = self.view_controls.get_mut(&key) {
            if pending.request.command != request.command || pending.operation_digest != digest {
                return Err(WorkerRefusal::CommandMismatch);
            }
            if pending.request.connection_epoch != request.connection_epoch {
                pending.correlations.clear();
                pending.request.connection_epoch = request.connection_epoch.clone();
            }
            if pending.correlations.len() >= 8
                && !pending.correlations.contains(&request.correlation_id)
            {
                return Err(WorkerRefusal::ResourceLimit);
            }
            pending.correlations.insert(request.correlation_id.clone());
            return Ok(true);
        }
        if self.view_controls.len() >= 128 {
            return Err(WorkerRefusal::ResourceLimit);
        }
        let record: crate::delegation::WorkspaceRecord =
            serde_json::from_value(value).map_err(|_| WorkerRefusal::CommandMismatch)?;
        if !matches!(
            record.strategy,
            crate::delegation::WorkspaceStrategy::CommitView
        ) || record.commit_view.is_none()
        {
            return Err(WorkerRefusal::CommandMismatch);
        }
        #[cfg(test)]
        let hold = self.source.contains_key("B221_VIEW_TEST_CONTROL_HOLD");
        #[cfg(test)]
        let allow = self.config.state_root.join("allow-view-control");
        let reader = std::thread::Builder::new()
            .name("bridget-view-control-audit".into())
            .spawn(move || {
                #[cfg(test)]
                if hold {
                    let until = Instant::now() + Duration::from_secs(10);
                    while !allow.exists() && Instant::now() < until {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
                crate::workspace_git::report(&record)
                    .map_err(|_| WorkerRefusal::WorkspacePrepareFailed)
            })
            .map_err(|_| WorkerRefusal::WorkerUnavailable)?;
        self.view_controls.insert(
            key,
            PendingViewControl {
                request: request.clone(),
                operation_digest: digest,
                expires,
                correlations: BTreeSet::from([request.correlation_id.clone()]),
                reader,
            },
        );
        Ok(true)
    }

    fn control_refusal(
        operation: &WorkerRequestOperation,
        reason: WorkerRefusal,
    ) -> WorkerResponseOperation {
        if matches!(operation, WorkerRequestOperation::StartDetailed { .. }) {
            WorkerResponseOperation::StartRejected {
                reason,
                cause: WorkerStartCapacityCause::Unknown,
            }
        } else {
            WorkerResponseOperation::Rejected { reason }
        }
    }

    fn poll_view_controls(
        &mut self,
        epoch: &str,
        disconnected: &AtomicBool,
    ) -> Vec<WorkerResponse> {
        let keys = self.view_controls.keys().cloned().collect::<Vec<_>>();
        let mut responses = Vec::new();
        for key in keys {
            let mut pending = self.view_controls.remove(&key).unwrap();
            if pending.request.connection_epoch != epoch {
                pending.correlations.clear();
            }
            let cancelled = self
                .cancelled
                .lock()
                .map_or(true, |cancelled| cancelled.contains(&key));
            let expired = Instant::now() >= pending.expires;
            let result = if cancelled || expired || disconnected.load(Ordering::Acquire) {
                Some(if expired {
                    if matches!(
                        pending.request.request,
                        WorkerRequestOperation::Release { .. }
                    ) {
                        WorkerRefusal::ReleaseExpired
                    } else {
                        WorkerRefusal::PrepareExpired
                    }
                } else {
                    WorkerRefusal::UnknownExecution
                })
            } else if pending.reader.is_finished() && !pending.correlations.is_empty() {
                let report = pending
                    .reader
                    .join()
                    .unwrap_or(Err(WorkerRefusal::WorkspacePrepareFailed));
                let result = report.and_then(|report| match &pending.request.request {
                    WorkerRequestOperation::Start { start }
                    | WorkerRequestOperation::StartDetailed { start } => {
                        let mut capacity = WorkerStartCapacityCause::Unknown;
                        let result = self.start_with_view_report(
                            &pending.request.command,
                            start,
                            disconnected,
                            &mut capacity,
                            Some(&report),
                        );
                        if matches!(
                            pending.request.request,
                            WorkerRequestOperation::StartDetailed { .. }
                        ) {
                            Ok(match result {
                                Ok(response) => response,
                                Err(reason) => WorkerResponseOperation::StartRejected {
                                    reason,
                                    cause: if reason == WorkerRefusal::ResourceLimit {
                                        capacity
                                    } else {
                                        WorkerStartCapacityCause::Unknown
                                    },
                                },
                            })
                        } else {
                            result
                        }
                    }
                    WorkerRequestOperation::Release { permit } => self.release_with_view_report(
                        &pending.request.command,
                        permit,
                        epoch,
                        disconnected,
                        Some(&report),
                    ),
                    WorkerRequestOperation::PrepareFenced {
                        previous,
                        prepare,
                        placement,
                        workspace,
                        routing,
                    } => self
                        .prepare_fenced_with_view_report(
                            &pending.request.command,
                            previous,
                            prepare,
                            placement,
                            workspace.as_ref(),
                            routing.as_ref(),
                            Some(&report),
                        )
                        .map(|prepared| WorkerResponseOperation::Prepared { prepared }),
                    _ => Err(WorkerRefusal::InvalidEnvelope),
                });
                for correlation_id in pending.correlations {
                    responses.push(WorkerResponse {
                        version: REMOTE_WORKER_VERSION,
                        correlation_id,
                        connection_epoch: epoch.into(),
                        worker: self.identity.clone(),
                        command: pending.request.command.clone(),
                        response: match &result {
                            Ok(response) => response.clone(),
                            Err(reason) => Self::control_refusal(&pending.request.request, *reason),
                        },
                    });
                }
                continue;
            } else {
                None
            };
            if let Some(reason) = result {
                for correlation_id in std::mem::take(&mut pending.correlations) {
                    responses.push(WorkerResponse {
                        version: REMOTE_WORKER_VERSION,
                        correlation_id,
                        connection_epoch: epoch.into(),
                        worker: self.identity.clone(),
                        command: pending.request.command.clone(),
                        response: Self::control_refusal(&pending.request.request, reason),
                    });
                }
            }
            if pending.reader.is_finished() && pending.correlations.is_empty() {
                let _ = pending.reader.join();
            } else {
                self.view_controls.insert(key, pending);
            }
        }
        responses
    }

    fn check_prepared_view_replay(&self, read: &ViewPrepareRead) -> Result<(), ViewFileError> {
        let command = &read.source.binding.command;
        let Some(previous) = self.prepared.get(&command.command_id) else {
            return Ok(());
        };
        if previous.command != *command
            || previous.routing != read.routing
            || json_digest(&previous.request).map_err(view_file_error)?
                != json_digest(&read.request).map_err(view_file_error)?
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        if Instant::now() >= previous.expires {
            return Err((WorkerRefusal::PrepareExpired, None));
        }
        let receipt = self
            .store
            .view_receipt_for_binding(&read.source.binding)
            .map_err(view_file_error)?;
        let record = receipt
            .record
            .ok_or((WorkerRefusal::CommandMismatch, None))?;
        if !previous
            .prepared
            .workspace
            .as_ref()
            .is_some_and(|value| same_workspace_anchor(&record, value))
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        let intent = crate::workspace_git::ViewProofs::open(
            &self
                .store
                .view_root()
                .canonicalize()
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
        )
        .and_then(|proofs| proofs.load(&read.source.binding.key()?))
        .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
        let (_, cwd) = intent
            .execution_paths(Path::new(&read.request.remote_root))
            .map_err(|_| (WorkerRefusal::CommandMismatch, None))?;
        if Path::new(&previous.prepared.placement.execution_cwd) != cwd {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        Ok(())
    }

    fn poll_prepared_view(
        &mut self,
        read: &ViewPrepareRead,
    ) -> Result<Option<WorkerPrepared>, ViewFileError> {
        let command = &read.source.binding.command;
        if read.source.binding.worker != self.identity
            || read.source.binding.prepare_digest
                != json_digest(&(command, &read.request)).map_err(view_file_error)?
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        if !self
            .store
            .view_preparation_unused(command)
            .map_err(view_file_error)?
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        self.check_prepared_view_replay(read)?;
        if Instant::now() >= read.expires {
            let stopped = if let Some(source) = self.view_sources.get_mut(&marker_key(command)) {
                if source.binding != read.source.binding || source.materialization.is_some() {
                    return Err((WorkerRefusal::CommandMismatch, None));
                }
                source
                    .child
                    .abort(
                        crate::workspace_git::ExportFailure::PreparationExpired,
                        Duration::ZERO,
                    )
                    .unwrap_or(false)
            } else {
                self.store
                    .view_receipt_for_binding(&read.source.binding)
                    .map_err(view_file_error)?
                    .commit_view
                    .as_ref()
                    .is_some_and(|view| view.source_stopped || !view.source_attempted)
            };
            return Err((
                WorkerRefusal::PrepareExpired,
                Some(if stopped {
                    crate::workspace_git::ExportFailure::PreparationExpired
                } else {
                    crate::workspace_git::ExportFailure::StopUnconfirmed
                }),
            ));
        }
        let waiting = self
            .store
            .view_receipt_for_binding(&read.source.binding)
            .map_err(view_file_error)?;
        if waiting
            .commit_view
            .as_ref()
            .is_some_and(|view| !view.source_attempted && !view.source_stopped && !view.closed)
        {
            let seed = self.native_view_plan(command, &read.request)?;
            self.begin_planned_view_source(command, &read.request, seed)?;
            let after = self
                .store
                .view_receipt_for_binding(&read.source.binding)
                .map_err(view_file_error)?;
            if after
                .commit_view
                .as_ref()
                .is_some_and(|view| !view.source_attempted)
            {
                return Ok(None);
            }
        }
        let Some(record) = self.poll_view_source(&read.source)? else {
            return Ok(None);
        };
        let registry = self.registry().map_err(|r| (r, None))?;
        if let Some(selection) = &read.routing {
            self.check_routing_connection(&registry, &read.request, selection)
                .map_err(|r| (r, None))?;
        }
        let mapping = self
            .mapping_authority(&read.request, &registry)
            .map_err(|r| (r, None))?;
        let (mut mapping, source_cwd) = self
            .read_view()
            .mapping_paths(&read.request, mapping)
            .map_err(|r| (r, None))?;
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let initial = self
            .readiness_at(
                &registry,
                &read.request,
                &mapping,
                &source_cwd,
                &mut operation,
                None,
            )
            .map_err(|r| (r, None))?;
        if initial.definition.digest != read.definition_digest {
            return Err((WorkerRefusal::DefinitionChanged, None));
        }
        if initial.effective_posture != SpawnPosture::Discovery
            || json_digest(&initial.child_policy).map_err(view_file_error)?
                != read.permission_digest
        {
            return Err((WorkerRefusal::PermissionsChanged, None));
        }
        let reference = record
            .commit_view
            .as_ref()
            .ok_or((WorkerRefusal::CommandMismatch, None))?;
        if reference.binding != read.source.binding {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        let intent = crate::workspace_git::ViewProofs::open(
            &self
                .store
                .view_root()
                .canonicalize()
                .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
        )
        .and_then(|proofs| proofs.load(&read.source.binding.key()?))
        .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?;
        let (root, cwd) = intent
            .execution_paths(&mapping.remote_root)
            .map_err(|_| (WorkerRefusal::PlacementMismatch, None))?;
        mapping.remote_root =
            crate::registry::validate_unprotected_local_path(&root, &self.protected())
                .map_err(|_| (WorkerRefusal::PlacementMismatch, None))?;
        let cwd = crate::registry::validate_unprotected_local_path(&cwd, &self.protected())
            .map_err(|_| (WorkerRefusal::PlacementMismatch, None))?;
        if !cwd.starts_with(&mapping.remote_root) {
            return Err((WorkerRefusal::PlacementMismatch, None));
        }
        let readiness = self
            .readiness_at(
                &registry,
                &read.request,
                &mapping,
                &cwd,
                &mut operation,
                None,
            )
            .map_err(|r| (r, None))?;
        operation
            .finish()
            .map_err(|_| (WorkerRefusal::PermissionsChanged, None))?;
        if readiness.effective_posture != SpawnPosture::Discovery {
            return Err((WorkerRefusal::PermissionsChanged, None));
        }
        if Instant::now() >= read.expires {
            return Err((WorkerRefusal::PrepareExpired, None));
        }
        let permission = json_digest(&readiness.child_policy).map_err(view_file_error)?;
        if let Some(previous) = self.prepared.get(&command.command_id) {
            if previous.command != *command
                || previous.routing != read.routing
                || json_digest(&previous.request).map_err(view_file_error)?
                    != json_digest(&read.request).map_err(view_file_error)?
            {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            if Instant::now() >= previous.expires {
                return Err((WorkerRefusal::PrepareExpired, None));
            }
            if previous.prepared.definition.digest != readiness.definition.digest {
                return Err((WorkerRefusal::DefinitionChanged, None));
            }
            if previous.prepared.placement.permission_digest != permission {
                return Err((WorkerRefusal::PermissionsChanged, None));
            }
            if Path::new(&previous.prepared.placement.execution_cwd) != cwd
                || !previous
                    .prepared
                    .workspace
                    .as_ref()
                    .is_some_and(|value| same_workspace_anchor(&record, value))
            {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            return Ok(Some(previous.prepared.clone()));
        }
        if self.prepared.len() >= 128 || !self.held.is_empty() {
            return Err((
                WorkerRefusal::ResourceLimit,
                Some(if self.held.is_empty() {
                    crate::workspace_git::ExportFailure::MetadataCapacity
                } else {
                    crate::workspace_git::ExportFailure::OperationCapacity
                }),
            ));
        }
        let prepared = WorkerPrepared {
            workspace: Some(
                serde_json::to_value(record)
                    .map_err(|_| (WorkerRefusal::WorkspacePrepareFailed, None))?,
            ),
            placement: RemotePlacement {
                machine_id: self.identity.machine_id.clone(),
                worker_id: self.identity.worker_id.clone(),
                provisioning_revision: self.identity.provisioning_revision,
                project_id: read.request.project_id.clone(),
                mapping_revision: read.request.mapping_revision,
                origin_host: read.request.origin_host.clone(),
                origin_cwd: read.request.origin_cwd.clone(),
                source_root_identity: read.request.source_root_identity.clone(),
                relative_cwd: read.request.relative_cwd.clone(),
                execution_cwd: cwd.to_string_lossy().into(),
                base_commit: read.request.base_commit.clone(),
                definition_digest: readiness.definition.digest.clone(),
                permission_digest: permission,
            },
            token: uuid::Uuid::new_v4().to_string(),
            definition: readiness.definition,
            child_policy: readiness.child_policy,
            effective_posture: readiness.effective_posture,
            prepared_at: worker_now(),
        };
        self.prepared.insert(
            command.command_id.clone(),
            PreparedRuntime {
                routing: read.routing.clone(),
                command: command.clone(),
                request: read.request.clone(),
                prepared: prepared.clone(),
                expires: Instant::now() + Duration::from_secs(30),
            },
        );
        Ok(Some(prepared))
    }

    /// Relire les droits ne constitue pas une admission de vue ou de modèle.
    fn recheck_view_file_actor(
        &self,
        execution: &WorkerResultIdentity,
        path: Option<&str>,
    ) -> Result<(), WorkerRefusal> {
        let (intent, _) =
            self.store
                .active_view_execution(execution)
                .map_err(|error| match error {
                    WorkerStoreError::Conflict => WorkerRefusal::CommandMismatch,
                    WorkerStoreError::Unknown | WorkerStoreError::InvalidTransition => {
                        WorkerRefusal::UnknownExecution
                    }
                    WorkerStoreError::Invalid(reason) => reason,
                    error => refuse(error),
                })?;
        let current = self.store.command(&execution.command).map_err(refuse)?;
        let (request, prepared) = current
            .preparation
            .as_ref()
            .ok_or(WorkerRefusal::UnknownExecution)?;
        let registry = self.registry()?;
        if let Some(routing) = &current.routing {
            self.check_routing_connection(&registry, request, routing)?;
        }
        let (mut mapping, _) = self.mapping_for(
            request,
            &registry,
            current.routing.as_ref().map(|routing| routing.workload),
        )?;
        let (root, cwd) = intent
            .execution_paths(&mapping.remote_root)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        let protected = self.protected();
        mapping.remote_root = crate::registry::validate_unprotected_local_path(&root, &protected)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        let cwd = crate::registry::validate_unprotected_local_path(&cwd, &protected)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !cwd.starts_with(&mapping.remote_root)
            || cwd != Path::new(&current.placement.execution_cwd)
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        if let Some(path) = path
            && (!bridget_transport::protocol::commit_view_file_path(path)
                || !intent
                    .destination()
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?
                    .join(path)
                    .starts_with(&mapping.remote_root))
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let fresh = self.readiness_at(&registry, request, &mapping, &cwd, &mut operation, None)?;
        operation
            .finish()
            .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        if fresh.definition != prepared.definition {
            return Err(WorkerRefusal::DefinitionChanged);
        }
        if fresh.effective_posture != SpawnPosture::Discovery
            || fresh.child_policy != prepared.child_policy
            || json_digest(&fresh.child_policy).map_err(refuse)?
                != current.placement.permission_digest
        {
            return Err(WorkerRefusal::PermissionsChanged);
        }
        Ok(())
    }

    /// Le fil maître a déjà prouvé l'acteur ; cette boucle ne reçoit qu'une
    /// exécution courante et un chemin fermé, sous WorkspaceViewV1.
    fn defer_view_file(&mut self, request: &WorkerRequest) -> Result<bool, ViewFileError> {
        let WorkerRequestOperation::ReadCommitViewFile { execution, path } = &request.request
        else {
            return Ok(false);
        };
        request.validate().map_err(|reason| (reason, None))?;
        if request.worker != self.identity {
            return Err((WorkerRefusal::PlacementMismatch, None));
        }
        let key = json_digest(&(&execution, path)).map_err(view_file_error)?;
        if let Some(pending) = self.view_files.get_mut(&key) {
            if pending.request.command != request.command
                || pending.request.request != request.request
            {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            if Instant::now() >= pending.expires {
                return Err((
                    WorkerRefusal::PrepareExpired,
                    Some(crate::workspace_git::ExportFailure::TimedOut),
                ));
            }
            if pending.request.connection_epoch != request.connection_epoch {
                pending.correlations.clear();
                pending.request.connection_epoch = request.connection_epoch.clone();
            }
            if pending.correlations.len() >= 8
                && !pending.correlations.contains(&request.correlation_id)
            {
                return Err((
                    WorkerRefusal::ResourceLimit,
                    Some(crate::workspace_git::ExportFailure::OperationCapacity),
                ));
            }
            pending.correlations.insert(request.correlation_id.clone());
            return Ok(true);
        }
        if self.view_files.len() >= 128 {
            return Err((
                WorkerRefusal::ResourceLimit,
                Some(crate::workspace_git::ExportFailure::OperationCapacity),
            ));
        }
        let read = self.begin_view_file(execution, path)?;
        self.view_files.insert(
            key,
            PendingViewFile {
                request: request.clone(),
                read,
                expires: Instant::now() + Duration::from_secs(120),
                correlations: BTreeSet::from([request.correlation_id.clone()]),
            },
        );
        Ok(true)
    }

    fn view_file_response(
        request: &WorkerRequest,
        result: Result<(), ViewFileError>,
    ) -> WorkerResponseOperation {
        let WorkerRequestOperation::ReadCommitViewFile { execution, path } = &request.request
        else {
            return WorkerResponseOperation::Rejected {
                reason: WorkerRefusal::InvalidEnvelope,
            };
        };
        let (available, reason, failure) = match result {
            Ok(()) => (true, None, None),
            Err((reason, failure)) => (
                false,
                Some(reason),
                Some(failure.unwrap_or(crate::workspace_git::ExportFailure::Unknown)),
            ),
        };
        WorkerResponseOperation::CommitViewFile {
            execution: execution.clone(),
            path: path.clone(),
            available,
            reason,
            failure,
        }
    }

    fn poll_view_files(&mut self, epoch: &str) -> Vec<WorkerResponse> {
        let keys = self.view_files.keys().cloned().collect::<Vec<_>>();
        let mut responses = Vec::new();
        for key in keys {
            let mut pending = self.view_files.remove(&key).unwrap();
            if pending.request.connection_epoch != epoch {
                pending.correlations.clear();
            }
            let result = if Instant::now() >= pending.expires {
                let source = format!("file-{}", marker_key(&pending.read.binding.command));
                let stopped = if let Some(source) = self.view_sources.get_mut(&source) {
                    source
                        .child
                        .abort(
                            crate::workspace_git::ExportFailure::TimedOut,
                            Duration::ZERO,
                        )
                        .unwrap_or(false)
                } else {
                    // L'absence en mémoire n'atteste pas l'arrêt d'un helper.
                    proof::read_workspace_receipt(
                        &self.store.root,
                        &self.identity,
                        &pending.read.binding.command,
                    )
                    .ok()
                    .flatten()
                    .is_some_and(|receipt| {
                        receipt
                            .commit_view
                            .as_ref()
                            .and_then(|view| view.materialization.as_ref())
                            .is_some_and(|operation| {
                                operation.execution == pending.read.execution
                                    && operation.request == pending.read.request
                                    && operation.stopped
                            })
                    })
                };
                Err((
                    WorkerRefusal::PrepareExpired,
                    Some(if stopped {
                        crate::workspace_git::ExportFailure::TimedOut
                    } else {
                        crate::workspace_git::ExportFailure::StopUnconfirmed
                    }),
                ))
            } else {
                match self.poll_view_file(&pending.read) {
                    Ok(false) => {
                        self.view_files.insert(key, pending);
                        continue;
                    }
                    Ok(true) => Ok(()),
                    Err(error) => Err(error),
                }
            };
            let response = Self::view_file_response(&pending.request, result);
            for correlation_id in pending.correlations {
                responses.push(WorkerResponse {
                    version: REMOTE_WORKER_VERSION,
                    correlation_id,
                    connection_epoch: epoch.into(),
                    worker: self.identity.clone(),
                    command: pending.request.command.clone(),
                    response: response.clone(),
                });
            }
        }
        responses
    }

    /// Le runtime possède la matérialisation ; aucun acteur ni chemin de preuve
    /// n'est accepté depuis le lecteur.
    fn begin_view_file(
        &mut self,
        execution: &WorkerResultIdentity,
        path: &str,
    ) -> Result<ViewFileRead, ViewFileError> {
        let plain = view_file_error;
        self.recheck_view_file_actor(execution, Some(path))
            .map_err(|reason| (reason, None))?;
        let cancel_key = json_digest(&execution.command).map_err(plain)?;
        let cancelled = self.cancelled.clone();
        let cancellation = || {
            cancelled
                .lock()
                .map(|set| set.contains(&cancel_key))
                .map_err(|_| (WorkerRefusal::StoreUnavailable, None))
        };
        if cancellation()? {
            return Err((
                WorkerRefusal::WorkspacePrepareFailed,
                Some(crate::workspace_git::ExportFailure::Cancelled),
            ));
        }
        let (anchor, _) = self.store.active_view_execution(execution).map_err(plain)?;
        let key = format!("file-{}", marker_key(&anchor.binding.command));
        if !self.view_writer_available(&anchor.binding, &key)? {
            return Err((
                WorkerRefusal::ResourceLimit,
                Some(crate::workspace_git::ExportFailure::OperationCapacity),
            ));
        }
        let (intent, request) =
            self.store
                .begin_materialization(execution, path)
                .map_err(|error| match error {
                    WorkerStoreError::Locked => (
                        WorkerRefusal::ResourceLimit,
                        Some(crate::workspace_git::ExportFailure::OperationCapacity),
                    ),
                    error => plain(error),
                })?;
        let read = ViewFileRead {
            binding: intent.binding.clone(),
            execution: execution.clone(),
            request: request.clone(),
        };
        let receipt = self
            .store
            .checked_view_receipt(&intent.binding.command, &intent)
            .map_err(plain)?;
        let operation = receipt
            .commit_view
            .as_ref()
            .unwrap()
            .materialization
            .as_ref()
            .unwrap();
        if !self.view_sources.contains_key(&key)
            && (operation.marker.is_some() || operation.attempted)
        {
            return Ok(read);
        }
        if !self.view_sources.contains_key(&key) {
            let proofs = crate::workspace_git::ViewProofs::open(
                &self
                    .store
                    .view_root()
                    .canonicalize()
                    .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?,
            )
            .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?;
            let executable =
                std::env::current_exe().map_err(|_| (WorkerRefusal::WorkerUnavailable, None))?;
            let release_reason = std::cell::Cell::new(None);
            let started = crate::workspace_git::ViewJob::start_registered(
                &proofs,
                &intent,
                &request,
                &executable,
                &self.source,
                || {
                    self.store
                        .mark_materialization_attempt(execution, &request)
                        .map_err(|error| {
                            release_reason.set(Some(plain(error)));
                            crate::workspace_git::ExportFailure::ProofUnavailable
                        })
                },
                |marker| {
                    if let Err(reason) = self.recheck_view_file_actor(execution, Some(path)) {
                        release_reason.set(Some((reason, None)));
                        return Err(crate::workspace_git::ExportFailure::ProofUnavailable);
                    }
                    match cancellation() {
                        Ok(false) => {}
                        Ok(true) => {
                            release_reason.set(Some((
                                WorkerRefusal::WorkspacePrepareFailed,
                                Some(crate::workspace_git::ExportFailure::Cancelled),
                            )));
                            return Err(crate::workspace_git::ExportFailure::Cancelled);
                        }
                        Err(error) => {
                            release_reason.set(Some(error));
                            return Err(crate::workspace_git::ExportFailure::ProofUnavailable);
                        }
                    }
                    self.store
                        .authorize_materialization(execution, &request, marker)
                        .map_err(|error| {
                            release_reason.set(Some(plain(error)));
                            crate::workspace_git::ExportFailure::ProofUnavailable
                        })
                },
            );
            match started {
                Ok(child) => {
                    self.view_sources.insert(
                        key.clone(),
                        ViewSourceProcess {
                            binding: intent.binding.clone(),
                            child,
                            materialization: Some((execution.clone(), request.clone())),
                        },
                    );
                }
                Err((cause, retained)) => {
                    if let Some(child) = retained {
                        self.view_sources.insert(
                            key,
                            ViewSourceProcess {
                                binding: intent.binding.clone(),
                                child,
                                materialization: Some((execution.clone(), request)),
                            },
                        );
                    } else {
                        self.store
                            .finish_materialization(execution, &request, Some(Err(cause)))
                            .map_err(plain)?;
                    }
                    return Err(release_reason
                        .get()
                        .unwrap_or((WorkerRefusal::WorkspacePrepareFailed, Some(cause))));
                }
            }
        }
        Ok(read)
    }

    /// Arrêt durci avant le lecteur ; le store n'entre jamais dans son thread.
    fn begin_view_file_audit(
        &mut self,
        read: &ViewFileRead,
        result: Option<Result<(), crate::workspace_git::ExportFailure>>,
    ) -> Result<bool, ViewFileError> {
        let key = format!("file-{}", marker_key(&read.binding.command));
        if let Some(audit) = self.view_file_audits.get(&key) {
            if audit.binding != read.binding
                || audit.execution != read.execution
                || audit.request != read.request
            {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            return Ok(true);
        }
        let Some(intent) = self
            .store
            .stop_materialization(&read.execution, &read.request, result)
            .map_err(view_file_error)?
        else {
            return Ok(false);
        };
        if intent.binding != read.binding {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        if let Some(Err(cause)) = result {
            return self
                .store
                .publish_materialization_report(
                    &read.execution,
                    &read.request,
                    &intent,
                    Some(false),
                    Some(cause),
                )
                .map_err(view_file_error);
        }
        let receipt = self
            .store
            .checked_view_receipt(&read.binding.command, &intent)
            .map_err(view_file_error)?;
        if receipt
            .commit_view
            .as_ref()
            .and_then(|view| view.materialization.as_ref())
            .is_some_and(|operation| operation.available.is_some())
        {
            return Ok(true);
        }
        if self.view_file_audits.len() >= 128 {
            return Err((
                WorkerRefusal::ResourceLimit,
                Some(crate::workspace_git::ExportFailure::OperationCapacity),
            ));
        }
        let crate::workspace_git::ViewJobRequest::Materialize { path, .. } = &read.request else {
            return Err((WorkerRefusal::InvalidEnvelope, None));
        };
        let path = path.clone();
        let root = self
            .store
            .view_root()
            .canonicalize()
            .map_err(|_| (WorkerRefusal::StoreUnavailable, None))?;
        #[cfg(test)]
        let hold = self.source.contains_key("B221_VIEW_TEST_FILE_AUDIT_HOLD");
        #[cfg(test)]
        let allow = self.config.state_root.join("allow-file-audit");
        let reader = std::thread::Builder::new()
            .name("bridget-view-file-audit".into())
            .spawn(move || {
                let available = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    #[cfg(test)]
                    if hold {
                        let until = Instant::now() + Duration::from_secs(10);
                        while !allow.exists() && Instant::now() < until {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                    }
                    crate::workspace_git::ViewProofs::open(&root)
                        .and_then(|proofs| proofs.file_available(&intent, &path))
                        .map_err(
                            |error| match crate::workspace_git::view_export_failure(&error) {
                                crate::workspace_git::ExportFailure::Unknown => {
                                    crate::workspace_git::ExportFailure::ProofUnavailable
                                }
                                cause => cause,
                            },
                        )
                        .and_then(|available| {
                            if !available && result.is_some() {
                                Err(crate::workspace_git::ExportFailure::IntegrityFailed)
                            } else {
                                Ok(available)
                            }
                        })
                }))
                .unwrap_or(Err(crate::workspace_git::ExportFailure::ProofUnavailable));
                (intent, available)
            })
            .map_err(|_| (WorkerRefusal::WorkerUnavailable, None))?;
        self.view_file_audits.insert(
            key,
            ViewFileAudit {
                binding: read.binding.clone(),
                execution: read.execution.clone(),
                request: read.request.clone(),
                state: ViewFileAuditState::Reading(reader),
            },
        );
        Ok(true)
    }

    /// Reprendre seulement un audit de données dont l'arrêt est déjà durci.
    /// Aucun helper, mission ou copie n'est démarré après réouverture du worker.
    fn resume_stopped_view_file_audits(&mut self) {
        let receipts = match self.store.view_receipts(None) {
            Ok(receipts) => receipts,
            Err(_) => {
                log::warn!("view_file_recovery_inventory_unavailable");
                return;
            }
        };
        for receipt in receipts {
            let Some(view) = &receipt.commit_view else {
                continue;
            };
            if view.closed {
                continue;
            }
            let Some(operation) = &view.materialization else {
                continue;
            };
            if !operation.stopped || operation.available.is_some() {
                continue;
            }
            let read = ViewFileRead {
                binding: crate::workspace_git::ViewBinding {
                    worker: receipt.worker.clone(),
                    command: receipt.command.clone(),
                    prepare_digest: receipt.prepare_digest.clone(),
                },
                execution: operation.execution.clone(),
                request: operation.request.clone(),
            };
            if let Err((reason, _)) = self.begin_view_file_audit(&read, None) {
                log::warn!("view_file_recovery_unavailable reason={reason:?}");
            }
        }
    }

    fn collect_view_file_audits(&mut self) {
        // O(n), n <= 128 ; aucun join avant fin, aucun nouvel audit après CAS refusé.
        let keys = self
            .view_file_audits
            .iter()
            .filter_map(|(key, audit)| match &audit.state {
                ViewFileAuditState::Reading(reader) => reader.is_finished().then_some(key.clone()),
                ViewFileAuditState::Ready { retry_at, .. } => {
                    (*retry_at <= Instant::now()).then_some(key.clone())
                }
            })
            .collect::<Vec<_>>();
        for key in keys {
            let ViewFileAudit {
                binding,
                execution,
                request,
                state,
            } = self.view_file_audits.remove(&key).unwrap();
            let (intent, available, failure, last_error) = match state {
                ViewFileAuditState::Reading(reader) => {
                    let Ok((intent, available)) = reader.join() else {
                        log::warn!("view_file_audit_unavailable");
                        continue;
                    };
                    let (available, failure) = match available {
                        Ok(value) => (Some(value), None),
                        Err(cause) => (Some(false), Some(cause)),
                    };
                    (intent, available, failure, None)
                }
                ViewFileAuditState::Ready {
                    intent,
                    available,
                    failure,
                    last_error,
                    ..
                } => (intent, available, failure, last_error),
            };
            if intent.binding != binding {
                log::warn!("view_file_audit_binding_unconfirmed");
                continue;
            }
            if let Err(error) = self
                .store
                .publish_materialization_report(&execution, &request, &intent, available, failure)
            {
                // Une clôture native exacte rend le constat obsolète, sans
                // promettre sa disponibilité. Une panne de stockage le conserve.
                let closed = self
                    .store
                    .view_receipt_for_binding(&binding)
                    .ok()
                    .and_then(|receipt| receipt.commit_view)
                    .is_some_and(|view| view.closed);
                if closed {
                    continue;
                }
                let code = error.to_string(); // Display contient seulement un code fermé.
                if last_error.as_deref() != Some(code.as_str()) {
                    log::warn!("view_file_audit_commit_unconfirmed reason={code}");
                }
                self.view_file_audits.insert(
                    key,
                    ViewFileAudit {
                        binding,
                        execution,
                        request,
                        state: ViewFileAuditState::Ready {
                            intent,
                            available,
                            failure,
                            retry_at: Instant::now() + Duration::from_secs(1),
                            last_error: Some(code),
                        },
                    },
                );
            }
        }
    }

    /// Un seul pas, jamais d'attente de fin de lecture dans la boucle worker.
    /// Les polls en cours ne chargent pas le manifeste et n'auditent pas la vue.
    fn poll_view_file(&mut self, read: &ViewFileRead) -> Result<bool, ViewFileError> {
        let execution = &read.execution;
        let request = &read.request;
        let key = format!("file-{}", marker_key(&read.binding.command));
        let receipt =
            proof::read_workspace_receipt(&self.store.root, &self.identity, &read.binding.command)
                .map_err(view_file_error)?
                .ok_or((WorkerRefusal::UnknownExecution, None))?;
        let view = receipt
            .commit_view
            .as_ref()
            .ok_or((WorkerRefusal::CommandMismatch, None))?;
        view.validate(&receipt).map_err(view_file_error)?;
        if receipt.prepare_digest != read.binding.prepare_digest
            || receipt.worker != read.binding.worker
            || receipt.command != read.binding.command
            || read.binding.key().ok().as_deref() != Some(view.key.as_str())
            || !view.materialization.as_ref().is_some_and(|operation| {
                operation.execution == *execution && operation.request == *request
            })
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        // Ne pas rendre un fichier à une mission déjà terminée. Le collecteur
        // conserve puis ferme indépendamment le propriétaire de son helper.
        self.store
            .active_view_command(execution)
            .map_err(view_file_error)?;
        let cancel_key = json_digest(&execution.command).map_err(view_file_error)?;
        let cancelled = match self.cancelled.lock() {
            Ok(set) => set.contains(&cancel_key),
            Err(_) => return Err((WorkerRefusal::StoreUnavailable, None)),
        };
        if cancelled {
            self.stop_owned_view_sources(&execution.command)
                .map_err(|reason| (reason, None))?;
        }
        if let Some(owned) = self.view_sources.get_mut(&key) {
            if !owned
                .materialization
                .as_ref()
                .is_some_and(|(owner, operation)| owner == execution && operation == request)
            {
                return Err((WorkerRefusal::CommandMismatch, None));
            }
            match owned
                .child
                .poll()
                .map_err(|_| (WorkerRefusal::UnknownExecution, None))?
            {
                crate::workspace_git::JobPoll::Running => return Ok(false),
                crate::workspace_git::JobPoll::StopUnconfirmed => {
                    return Err((
                        WorkerRefusal::UnknownExecution,
                        Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
                    ));
                }
                crate::workspace_git::JobPoll::Stopped(result) => {
                    if !self.begin_view_file_audit(read, Some(result))? {
                        return Err((WorkerRefusal::UnknownExecution, None));
                    }
                    self.view_sources.remove(&key);
                }
            }
        } else if (!view.materialization.as_ref().unwrap().stopped
            || view.materialization.as_ref().unwrap().available.is_none())
            && !self.begin_view_file_audit(read, None)?
        {
            return Err((
                WorkerRefusal::UnknownExecution,
                Some(crate::workspace_git::ExportFailure::StopUnconfirmed),
            ));
        }
        self.collect_view_file_audits();
        if cancelled {
            return Err((
                WorkerRefusal::WorkspacePrepareFailed,
                Some(crate::workspace_git::ExportFailure::Cancelled),
            ));
        }
        if self.view_file_audits.contains_key(&key) {
            return Ok(false);
        }
        let crate::workspace_git::ViewJobRequest::Materialize { path, .. } = &read.request else {
            return Err((WorkerRefusal::InvalidEnvelope, None));
        };
        self.recheck_view_file_actor(execution, Some(path))
            .map_err(|reason| (reason, None))?;
        let done =
            proof::read_workspace_receipt(&self.store.root, &self.identity, &read.binding.command)
                .map_err(view_file_error)?
                .ok_or((WorkerRefusal::UnknownExecution, None))?;
        let operation = done
            .commit_view
            .as_ref()
            .and_then(|view| view.materialization.as_ref())
            .ok_or((WorkerRefusal::UnknownExecution, None))?;
        if operation.execution != *execution || operation.request != *request || !operation.stopped
        {
            return Err((WorkerRefusal::CommandMismatch, None));
        }
        if operation.available == Some(true) && operation.failure.is_none() {
            Ok(true)
        } else {
            Err((WorkerRefusal::WorkspacePrepareFailed, operation.failure))
        }
    }

    /// Seulement l'adaptateur des fixtures. Aucun port de production ne doit
    /// attendre ici : il conserve ViewFileRead et revient au prochain tick.
    #[cfg(test)]
    fn materialize_view_file(
        &mut self,
        execution: &WorkerResultIdentity,
        path: &str,
    ) -> Result<(), ViewFileError> {
        let read = self.begin_view_file(execution, path)?;
        loop {
            if self.poll_view_file(&read)? {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn collect_view_sources(&mut self) {
        self.collect_view_file_audits();
        let completed = self
            .view_reports
            .iter()
            .filter_map(|(key, report)| {
                let cancelled = json_digest(&report.binding.command)
                    .ok()
                    .and_then(|digest| self.cancelled.lock().ok().map(|set| set.contains(&digest)))
                    .unwrap_or(false);
                ((cancelled || Instant::now() >= report.expires) && report.reader.is_finished())
                    .then_some(key.clone())
            })
            .collect::<Vec<_>>();
        for key in completed {
            if let Some(report) = self.view_reports.remove(&key) {
                let _ = report.reader.join();
            }
        }

        let keys = self.view_sources.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            let source = self.view_sources.get_mut(&key).unwrap();
            if source.materialization.is_none() {
                let cancel = json_digest(&source.binding.command).and_then(|key| {
                    self.cancelled
                        .lock()
                        .map(|set| set.contains(&key))
                        .map_err(|_| WorkerStoreError::Quarantined)
                });
                let cause = match cancel {
                    Ok(false) => None,
                    Ok(true) => Some(crate::workspace_git::ExportFailure::Cancelled),
                    Err(_) => Some(crate::workspace_git::ExportFailure::ProofUnavailable),
                };
                if let Some(cause) = cause {
                    let _ = source.child.abort(cause, Duration::ZERO);
                }
            }
            if let Some((execution, _)) = &source.materialization {
                let cancel_key = json_digest(&execution.command);
                let cancellation = cancel_key.and_then(|key| {
                    self.cancelled
                        .lock()
                        .map(|set| set.contains(&key))
                        .map_err(|_| WorkerStoreError::Quarantined)
                });
                let cause = match cancellation {
                    Ok(true) => Some(crate::workspace_git::ExportFailure::Cancelled),
                    Ok(false) if self.store.active_view_command(execution).is_ok() => None,
                    _ => Some(crate::workspace_git::ExportFailure::ProofUnavailable),
                };
                if let Some(cause) = cause {
                    let _ = source.child.abort(cause, Duration::ZERO);
                }
            }
            match source.child.poll() {
                Ok(crate::workspace_git::JobPoll::Stopped(result)) => {
                    let task = source.binding.command.task_id.clone();
                    if let Some((execution, request)) = &source.materialization {
                        let read = ViewFileRead {
                            binding: source.binding.clone(),
                            execution: execution.clone(),
                            request: request.clone(),
                        };
                        if self
                            .begin_view_file_audit(&read, Some(result))
                            .is_ok_and(|started| started)
                        {
                            self.view_sources.remove(&key);
                        }
                        continue;
                    }
                    let settled = self
                        .store
                        .record_view_source_exit(&source.binding, Some(result))
                        .and_then(|stopped| {
                            if stopped {
                                self.store.settle_view_sources(&task)
                            } else {
                                Ok(false)
                            }
                        });
                    let already_closed =
                        matches!(settled, Err(WorkerStoreError::InvalidTransition))
                            && source.materialization.as_ref().is_some_and(
                                |(execution, request)| {
                                    self.store
                                        .view_receipt_for_binding(&source.binding)
                                        .ok()
                                        .and_then(|receipt| receipt.commit_view)
                                        .is_some_and(|view| {
                                            view.closed
                                                && view.materialization.as_ref().is_some_and(
                                                    |operation| {
                                                        operation.stopped
                                                            && operation.execution == *execution
                                                            && operation.request == *request
                                                    },
                                                )
                                        })
                                },
                            );
                    if matches!(settled, Ok(true)) || already_closed {
                        self.view_sources.remove(&key);
                    }
                }
                Ok(_) => {}
                Err(_) => log::warn!("source_view_status_unconfirmed key={key}"),
            }
        }
    }

    fn stop_owned_view_sources(
        &mut self,
        command: &WorkerCommandIdentity,
    ) -> Result<(), WorkerRefusal> {
        for (_, source) in self.view_sources.iter_mut().filter(|(_, source)| {
            source.materialization.as_ref().map_or_else(
                || {
                    source.binding.command.task_id == command.task_id
                        && source.binding.command.command_id == command.command_id
                },
                |(execution, _)| execution.command == *command,
            )
        }) {
            let suffix = source
                .materialization
                .as_ref()
                .map(|(_, request)| {
                    request
                        .identity_for_binding(&source.binding)
                        .map(|identity| format!("-file-{}", identity.command_id))
                })
                .transpose()
                .map_err(|_| WorkerRefusal::CommandMismatch)?
                .unwrap_or_default();
            let stop_key = format!(
                "view-source-{}{}",
                source
                    .binding
                    .key()
                    .map_err(|_| WorkerRefusal::CommandMismatch)?,
                suffix
            );
            worker_stop_once(&mut self.cancel_signal_issued, &stop_key, || {
                source
                    .child
                    .cancel(Duration::from_millis(500))
                    .map(|_| ())
                    .map_err(|_| WorkerRefusal::UnknownExecution)
            })?;
        }
        for (key, owner, marker) in self
            .store
            .view_source_markers(&command.task_id)
            .map_err(refuse)?
        {
            let owns = if owner.validate_preparation().is_ok() {
                owner.task_id == command.task_id && owner.command_id == command.command_id
            } else {
                owner == *command
            };
            if !owns {
                continue;
            }
            let stop_key = format!("view-source-{key}");
            if self.cancel_signal_issued.contains(&stop_key)
                || !crate::managed_process::group_exists(marker.pgid).map_err(unavailable)?
            {
                continue;
            }
            if crate::managed_process::process_birth(marker.pgid).ok() != Some(marker.birth) {
                return Err(WorkerRefusal::UnknownExecution);
            }
            worker_stop_once(&mut self.cancel_signal_issued, &stop_key, || {
                // Identité relue juste avant le signal ; jamais un PID du RPC.
                if crate::managed_process::process_birth(marker.pgid).ok() != Some(marker.birth) {
                    return Err(WorkerRefusal::UnknownExecution);
                }
                crate::managed_process::signal_group(marker.pgid, libc::SIGTERM)
                    .map_err(unavailable)
            })?;
        }
        self.collect_view_sources();
        Ok(())
    }

    fn read_view(&self) -> read::WorkerReadView<'_> {
        read::WorkerReadView {
            config: &self.config,
            identity: &self.identity,
            credential: &self.credential,
            source: &self.source,
            quiet: false,
        }
    }

    fn new(config: WorkerRuntimeConfig) -> Result<Self, String> {
        let (identity, credential, source) = read::inputs(&config)?;
        let store = WorkerStore::open(&config.state_root.join("store"), identity.clone())
            .map_err(|e| e.to_string())?;
        private_dir(&store.root.join("markers")).map_err(|e| e.to_string())?;
        private_dir(&config.state_root.join("journals")).map_err(|e| e.to_string())?;
        let markers = ManagedMarkerStore::at_directory(store.root.join("markers"));
        let mut runtime = Self {
            config,
            identity,
            credential,
            store,
            markers,
            source,
            prepared: BTreeMap::new(),
            held: BTreeMap::new(),
            running: BTreeMap::new(),
            abandoned: BTreeMap::new(),
            view_sources: BTreeMap::new(),
            view_reports: BTreeMap::new(),
            view_preparations: BTreeMap::new(),
            view_controls: BTreeMap::new(),
            view_files: BTreeMap::new(),
            view_file_audits: BTreeMap::new(),
            last_maintenance: Instant::now() - Duration::from_secs(1),
            cancelled: Arc::new(Mutex::new(BTreeSet::new())),
            cancel_signal_issued: BTreeSet::new(),
            host_resources: crate::host_resources::HostResourceCollector::default(),
            last_resource_refresh: Instant::now(),
            provider_accounts_negotiated: false,
            program_compatibility_negotiated: false,
            permission_reasons_negotiated: false,
        };
        runtime
            .reconcile_all()
            .map_err(|e| format!("worker_reconcile:{e:?}"))?;
        runtime.resume_stopped_view_file_audits();
        Ok(runtime)
    }

    fn registry(&self) -> Result<AgentRegistry, WorkerRefusal> {
        self.read_view().registry()
    }
    fn protected(&self) -> Vec<PathBuf> {
        self.read_view().protected()
    }
    fn mapping_for(
        &self,
        request: &WorkerPrepare,
        registry: &AgentRegistry,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        self.read_view().mapping_for(request, registry, workload)
    }

    fn mapping_authority(
        &self,
        request: &WorkerPrepare,
        registry: &AgentRegistry,
    ) -> Result<ProjectMapping, WorkerRefusal> {
        self.read_view().mapping_authority(request, registry)
    }

    /// O(h), h<=256 ancêtres ; une erreur ou un marqueur ambigu reste un refus.
    /// Aucun stderr/code Git ne sert à déduire l'absence de dépôt.
    fn check_routing_connection(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        routing: &RoutingPreparation,
    ) -> Result<(), WorkerRefusal> {
        self.read_view()
            .check_routing_connection(registry, request, routing)
    }

    fn compilation_capacity(&self, registry: &AgentRegistry) -> Option<u8> {
        registry
            .routing_policy()?
            .machine_policies
            .get(&self.identity.machine_id)?
            .compilation_capacity
    }

    /// Contrôle Git de lecture, également utilisé pour le cwd partagé et la
    /// base d'un worktree déjà créé par `Prepare`.
    fn verify_base_commit(cwd: &Path, base_commit: Option<&str>) -> Result<(), WorkerRefusal> {
        let Some(base_commit) = base_commit else {
            return Ok(());
        };
        let actual = crate::workspace_git::run_git(
            cwd,
            &["rev-parse", "--verify", "--end-of-options", "HEAD^{commit}"],
            crate::workspace_git::READ_TIMEOUT,
        )
        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !actual.success() || actual.truncated || actual.stdout.trim() != base_commit {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok(())
    }

    /// Partie strictement lecture de `Prepare`. Elle vérifie le binaire, le
    /// couple modèle/effort et les droits projetés, mais ne crée ni worktree,
    /// ni token Prepared, ni slot ou processus fournisseur.
    /// Convertit un tuple opaque de probe en la même demande de lecture que
    /// `Prepare`. Les champs de machine restent attestés par l'identité active,
    /// jamais proposés par le décideur.
    /// Préconditions d'un worktree futur : Git reste strictement en lecture.
    /// La création et le cwd réel restent réservés à `prepare_workspace` du
    /// seul élu après admission.
    /// Le worker traite ses commandes en série. Les préparations et reçus
    /// durables couvrent aussi un redémarrage : aucune mission connue ne peut
    /// changer de fichiers sous ses pieds, même si son état est incertain.
    /// La correspondance identifie le depot ; les mutations Shared visent
    /// son checkout reel, comme refreshCheckedOutBranch(cwd) dans T3.
    fn remap_prepared_worktree(
        &self,
        mapping: &mut ProjectMapping,
        cwd: &Path,
        record: &crate::delegation::WorkspaceRecord,
    ) -> Result<PathBuf, WorkerRefusal> {
        self.read_view()
            .remap_prepared_worktree(mapping, cwd, record)
    }

    fn inspect_candidate(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    ) -> Result<CandidateReadiness, WorkerRefusal> {
        self.read_view().inspect_candidate(
            registry,
            request,
            mapping,
            cwd,
            operation,
            diagnostics,
            self.provider_accounts_negotiated,
        )
    }
    fn readiness_at(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    ) -> Result<CandidateReadiness, WorkerRefusal> {
        self.read_view()
            .readiness_at(registry, request, mapping, cwd, operation, diagnostics)
    }
    fn checkout_root(
        &self,
        mapping: &ProjectMapping,
        cwd: &Path,
    ) -> Result<PathBuf, WorkerRefusal> {
        self.read_view().checkout_root(mapping, cwd)
    }
    fn probe_workspace_preconditions(
        &self,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
    ) -> Result<(), CandidateProbeRefusal> {
        self.read_view()
            .probe_workspace_preconditions(request, mapping, cwd, |root, base| {
                self.shared_refresh_allowed(root, base).map(|_| ())
            })
    }
    fn probe_read_context(&self) -> read::ProbeReadContext {
        let prepared = self
            .prepared
            .values()
            .map(|p| PathBuf::from(&p.prepared.placement.execution_cwd))
            .collect();
        read::ProbeReadContext {
            config: self.config.clone(),
            identity: self.identity.clone(),
            credential: self.credential.clone(),
            source: self.source.clone(),
            prepared,
            command_roots: std::cell::RefCell::new(None),
            provider_accounts_negotiated: self.provider_accounts_negotiated,
            permission_reasons_negotiated: self.permission_reasons_negotiated,
            host_resources: self.host_resources.clone(),
        }
    }

    fn dispatch_probe_read(
        &self,
        inbound: WorkerInbound,
        slot: &Arc<AtomicBool>,
        writer: &Arc<Mutex<UnixStream>>,
        disconnected: &Arc<AtomicBool>,
        workspace_diagnostics: bool,
        deadline: Instant,
    ) -> Result<(), String> {
        let Some(permit) = ProbeReadPermit::claim(slot) else {
            return send_worker_frame(
                writer,
                &worker_probe_refusal(&inbound, CandidateProbeRefusal::ResourceLimit),
            );
        };
        let context = self.probe_read_context();
        let program_compatibility = self.program_compatibility_negotiated;
        let failure = worker_probe_refusal(&inbound, CandidateProbeRefusal::ResourceLimit);
        let stopped = disconnected.clone();
        spawn_probe_read(permit, writer, disconnected.clone(), failure, move || {
            let mut context = context;
            let result = match &inbound {
                WorkerInbound::CandidateProbe(probe) => context
                    .candidate_probe(probe, &stopped, deadline)
                    .map(|mut result| {
                        for v in &mut result.verdicts {
                            if let CandidateProbeVerdict::Ineligible { reason, .. } = v {
                                *reason = workspace_probe_reason_for_master(
                                    *reason,
                                    workspace_diagnostics,
                                );
                                *reason =
                                    program_probe_reason_for_master(*reason, program_compatibility);
                            }
                        }
                        WrapperToDaemon::CandidateProbeResult { result }
                    }),
                WorkerInbound::CandidateProbeV2(probe) => context
                    .candidate_probe_v2(probe, &stopped, deadline)
                    .map(|mut result| {
                        for v in &mut result.verdicts {
                            if let CandidateProbeV2Verdict::Ineligible { reason, .. } = v {
                                *reason = workspace_probe_reason_for_master(
                                    *reason,
                                    workspace_diagnostics,
                                );
                                *reason =
                                    program_probe_reason_for_master(*reason, program_compatibility);
                            }
                        }
                        WrapperToDaemon::CandidateProbeResultV2 { result }
                    }),
                _ => unreachable!("sonde seulement"),
            };
            result.unwrap_or_else(|_| {
                worker_probe_refusal(&inbound, CandidateProbeRefusal::CapabilityUnavailable)
            })
        })
    }

    fn shared_refresh_allowed(
        &self,
        root: &Path,
        base: &str,
    ) -> Result<bool, CandidateProbeRefusal> {
        if !crate::workspace_git::shared_refresh_needed(root, base)? {
            return Ok(false);
        }
        let mut occupied = self
            .prepared
            .values()
            .map(|p| PathBuf::from(&p.prepared.placement.execution_cwd))
            .collect::<Vec<_>>();
        occupied.extend(self.read_view().active_command_roots()?);
        read::shared_paths_available(root, &occupied)?;
        Ok(true)
    }

    fn refresh_shared_workspace(&self, root: &Path, base: &str) -> Result<(), WorkerRefusal> {
        if self
            .shared_refresh_allowed(root, base)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?
        {
            crate::workspace_git::refresh_shared(root, base)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        }
        Ok(())
    }

    /// Sonde groupée corrélée à une seule admission. Elle ne touche jamais le
    /// store, `prepared`, `held`, un worktree, un slot ou un fournisseur.
    /// Probe168 : le registre local est l'autorité de la connexion opaque.
    /// Le contrôle est lecture seule et précède toute conversion en Prepare.
    fn prepare(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
    ) -> Result<WorkerPrepared, WorkerRefusal> {
        self.prepare_for(command, request, None, false)
    }
    fn prepare_for(
        &mut self,
        command: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        routing: Option<(RoutingPreparation, &str, &str)>,
        inspect_only: bool,
    ) -> Result<WorkerPrepared, WorkerRefusal> {
        validate_prepare_command(command)?;
        let is_view = matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        );
        if is_view && (!crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED || !inspect_only) {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if inspect_only
            && (routing.is_none()
                || !matches!(
                    request.workspace,
                    Some(
                        bridget_transport::protocol::WorkspaceRequest::Worktree { .. }
                            | bridget_transport::protocol::WorkspaceRequest::CommitView {}
                    )
                ))
        {
            return Err(WorkerRefusal::InvalidEnvelope);
        }
        if !inspect_only && !self.held.is_empty() {
            eprintln!(
                "bridget_prepare_capacity {}",
                serde_json::json!({"command":command,"cause":"held_bootstrap","prepared":self.prepared.len(),"held":self.held.len(),"prepared_limit":128})
            );
            return Err(WorkerRefusal::ResourceLimit);
        }
        if !inspect_only
            && self.prepared.len() >= 128
            && !self.prepared.contains_key(&command.command_id)
        {
            eprintln!(
                "bridget_prepare_capacity {}",
                serde_json::json!({"command":command,"cause":"prepared_slots","prepared":self.prepared.len(),"held":self.held.len(),"prepared_limit":128})
            );
            return Err(WorkerRefusal::ResourceLimit);
        }
        // Une extraction de worktree peut durer jusqu'à WORKTREE_ADD_TIMEOUT.
        let expires = Instant::now()
            + Duration::from_secs(30)
            + if matches!(
                &request.workspace,
                Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
            ) {
                crate::workspace_git::WORKTREE_ADD_TIMEOUT
            } else {
                Duration::ZERO
            };
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let registry = self.registry()?;
        if let Some((selection, _, _)) = &routing {
            self.check_routing_connection(&registry, request, selection)?;
        }
        let (mut mapping, mut cwd) = self.mapping_for(
            request,
            &registry,
            routing.as_ref().map(|(selection, _, _)| selection.workload),
        )?;
        let mut inspected_base = None;
        // Les droits et la définition sont revalidés avant toute création de
        // worktree. Une révocation entre Probe et Prepare reste donc sans
        // slot, répertoire ni lancement fournisseur.
        if let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        {
            // Pour un worktree, `base_commit` qualifie la base future choisie
            // par `base_ref`, pas le HEAD courant du dépôt source. Ce même
            // préflight est relu avant toute écriture, puis le cwd créé est
            // contrôlé une seconde fois ci-dessous.
            let root = self.checkout_root(&mapping, &cwd)?;
            let preflight = crate::workspace_git::preflight_remote(
                &root,
                base_ref.as_deref(),
                branch.as_deref(),
            )
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if request
                .base_commit
                .as_ref()
                .is_some_and(|expected| expected != &preflight.base_commit)
            {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            inspected_base = Some(preflight.base_commit);
        } else if is_view {
            self.probe_workspace_preconditions(request, &mapping, &cwd)
                .map_err(|_| WorkerRefusal::WorkspacePrepareFailed)?;
            inspected_base = request.base_commit.clone();
        } else if let Some(base) = request.base_commit.as_deref() {
            let root = self.checkout_root(&mapping, &cwd)?;
            self.shared_refresh_allowed(&root, base)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        }
        let initial =
            self.inspect_candidate(&registry, request, &mapping, &cwd, &mut operation, None)?;
        if let Some((_, expected_definition, expected_permission)) = &routing {
            if initial.definition.digest != *expected_definition {
                return Err(WorkerRefusal::DefinitionChanged);
            }
            if json_digest(&initial.child_policy).map_err(refuse)? != *expected_permission {
                return Err(WorkerRefusal::PermissionsChanged);
            }
        }
        if inspect_only {
            operation
                .finish()
                .map_err(|_| WorkerRefusal::PermissionsChanged)?;
            if Instant::now() >= expires {
                return Err(WorkerRefusal::PrepareExpired);
            }
            // Ce résultat sert uniquement à l admission durable du maître.
            // Il ne crée aucun reçu, slot, worktree, garde Git ou PreparedRuntime.
            return Ok(WorkerPrepared {
                workspace: None,
                placement: RemotePlacement {
                    machine_id: self.identity.machine_id.clone(),
                    worker_id: self.identity.worker_id.clone(),
                    provisioning_revision: self.identity.provisioning_revision,
                    project_id: request.project_id.clone(),
                    mapping_revision: request.mapping_revision,
                    origin_host: request.origin_host.clone(),
                    origin_cwd: request.origin_cwd.clone(),
                    source_root_identity: request.source_root_identity.clone(),
                    relative_cwd: request.relative_cwd.clone(),
                    execution_cwd: cwd.to_string_lossy().into_owned(),
                    base_commit: inspected_base,
                    definition_digest: initial.definition.digest.clone(),
                    permission_digest: json_digest(&initial.child_policy).map_err(refuse)?,
                },
                token: bridget_transport::protocol::WORKER_INSPECTION_TOKEN.into(),
                definition: initial.definition,
                child_policy: initial.child_policy,
                effective_posture: initial.effective_posture,
                prepared_at: worker_now(),
            });
        }
        let mut worktree_base = None;
        let mut workspace = None;
        if matches!(
            &request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
        ) {
            let root = self.checkout_root(&mapping, &cwd)?;
            let record = self
                .store
                .prepare_workspace(command, request, &root)
                .map_err(refuse)?;
            worktree_base = Some(record.base_commit.clone());
            workspace = Some(serde_json::to_value(&record).map_err(unavailable)?);
            cwd = self.remap_prepared_worktree(&mut mapping, &cwd, &record)?;
        }
        if !matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
        ) && let Some(base) = request.base_commit.as_deref()
        {
            let root = self.checkout_root(&mapping, &cwd)?;
            self.refresh_shared_workspace(&root, base)?;
        }
        Self::verify_base_commit(&cwd, request.base_commit.as_deref())?;
        // Le second contrôle conserve le verdict exact du cwd de worktree
        // choisi, après que le premier a fermé toute création sur révocation.
        let readiness =
            self.inspect_candidate(&registry, request, &mapping, &cwd, &mut operation, None)?;
        operation
            .finish()
            .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        if Instant::now() >= expires {
            return Err(WorkerRefusal::PrepareExpired);
        }
        let prepared = WorkerPrepared {
            workspace,
            placement: RemotePlacement {
                machine_id: self.identity.machine_id.clone(),
                worker_id: self.identity.worker_id.clone(),
                provisioning_revision: self.identity.provisioning_revision,
                project_id: request.project_id.clone(),
                mapping_revision: request.mapping_revision,
                origin_host: request.origin_host.clone(),
                origin_cwd: request.origin_cwd.clone(),
                source_root_identity: request.source_root_identity.clone(),
                relative_cwd: request.relative_cwd.clone(),
                execution_cwd: cwd.to_string_lossy().into_owned(),
                base_commit: worktree_base.or_else(|| request.base_commit.clone()),
                definition_digest: readiness.definition.digest.clone(),
                permission_digest: json_digest(&readiness.child_policy).map_err(refuse)?,
            },
            token: uuid::Uuid::new_v4().to_string(),
            definition: readiness.definition,
            child_policy: readiness.child_policy,
            effective_posture: readiness.effective_posture,
            prepared_at: worker_now(),
        };
        self.prepared.insert(
            command.command_id.clone(),
            PreparedRuntime {
                routing: routing.map(|(selection, _, _)| selection),
                command: command.clone(),
                request: request.clone(),
                prepared: prepared.clone(),
                expires,
            },
        );
        Ok(prepared)
    }

    /// Reprend l'emplacement attesté sans mutation Git. Le worktree garde
    /// l'ancre originale ; Shared garde le HEAD et la politique courants.
    #[cfg(test)]
    fn prepare_fenced(
        &mut self,
        command: &WorkerCommandIdentity,
        previous: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        placement: &RemotePlacement,
        workspace: Option<&serde_json::Value>,
    ) -> Result<WorkerPrepared, WorkerRefusal> {
        self.prepare_fenced_routed(command, previous, request, placement, workspace, None)
    }

    /// `fenced_routing` None ne vaut que pour un reçu Prepare162 ; un reçu
    /// PrepareV2 garde sa sélection168 jusqu'à Release.
    fn prepare_fenced_routed(
        &mut self,
        command: &WorkerCommandIdentity,
        previous: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        placement: &RemotePlacement,
        workspace: Option<&serde_json::Value>,
        fenced_routing: Option<&bridget_transport::protocol::WorkerFencedRouting>,
    ) -> Result<WorkerPrepared, WorkerRefusal> {
        self.prepare_fenced_with_view_report(
            command,
            previous,
            request,
            placement,
            workspace,
            fenced_routing,
            None,
        )
    }

    fn prepare_fenced_with_view_report(
        &mut self,
        command: &WorkerCommandIdentity,
        previous: &WorkerCommandIdentity,
        request: &WorkerPrepare,
        placement: &RemotePlacement,
        workspace: Option<&serde_json::Value>,
        fenced_routing: Option<&bridget_transport::protocol::WorkerFencedRouting>,
        view_report: Option<&crate::delegation::WorkspaceRecord>,
    ) -> Result<WorkerPrepared, WorkerRefusal> {
        let is_view = matches!(
            request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
        );
        if is_view && view_report.is_none() {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        let routing = fenced_routing.map(|value| RoutingPreparation {
            connection_id: value.connection_id.clone(),
            engine: value.engine.clone(),
            workload: value.workload,
        });
        validate_prepare_command(command)?;
        previous.validate()?;
        request.validate()?;
        placement.validate()?;
        if previous.task_id != command.task_id
            || previous.command_id == command.command_id
            || !placement.matches_worker(&self.identity)
            || !matches!(
                (&request.workspace, workspace),
                (
                    Some(
                        bridget_transport::protocol::WorkspaceRequest::Worktree { .. }
                            | bridget_transport::protocol::WorkspaceRequest::CommitView {}
                    ),
                    Some(_)
                ) | (
                    Some(bridget_transport::protocol::WorkspaceRequest::Shared {}),
                    None
                )
            )
        {
            return Err(WorkerRefusal::InvalidEnvelope);
        }
        if !self.held.is_empty() {
            return Err(WorkerRefusal::ResourceLimit);
        }
        if self.prepared.len() >= 128 && !self.prepared.contains_key(&command.command_id) {
            return Err(WorkerRefusal::ResourceLimit);
        }
        let original = if let Some(workspace) = workspace {
            let original = self
                .store
                .fenced_workspace(previous, placement, workspace)
                .map_err(refuse_fenced_workspace)?;
            if is_view {
                let Some(report) = view_report else {
                    return Err(WorkerRefusal::CapabilityRequired);
                };
                let mut prior = previous.clone();
                prior.instance_id.clear();
                prior.generation = 0;
                let anchor = view_anchor_receipt_at(&self.store.root, &self.identity, &prior, None)
                    .map_err(refuse_fenced_workspace)?;
                let state = anchor
                    .commit_view
                    .as_ref()
                    .ok_or(WorkerRefusal::UnknownExecution)?;
                if state.closed
                    || !state.source_stopped
                    || state.source_marker.is_none()
                    || state.source_failure.is_some()
                    || state
                        .materialization
                        .as_ref()
                        .is_some_and(|operation| !operation.stopped)
                {
                    return Err(WorkerRefusal::UnknownExecution);
                }
                if !matches!(
                    original.strategy,
                    crate::delegation::WorkspaceStrategy::CommitView
                ) || request.posture != Some(SpawnPosture::Discovery)
                    || !original.branch.is_empty()
                    || original.commit_view.is_none()
                    || report.dirty != Some(false)
                    || !same_workspace_anchor(
                        &original,
                        &serde_json::to_value(report).map_err(unavailable)?,
                    )
                    || request.base_commit.as_deref() != Some(original.base_commit.as_str())
                    || placement.base_commit.as_deref() != Some(original.base_commit.as_str())
                {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
            } else if !matches!(
                original.strategy,
                crate::delegation::WorkspaceStrategy::Worktree
            ) || request.base_commit.as_deref() != Some(original.base_commit.as_str())
                || placement.base_commit.as_deref() != Some(original.base_commit.as_str())
                || !matches!(&request.workspace,
                    Some(bridget_transport::protocol::WorkspaceRequest::Worktree { branch, .. })
                        if branch.as_ref().is_none_or(|value| value == &original.branch))
            {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            Some(original)
        } else {
            self.store
                .fenced_placement(previous, placement)
                .map_err(refuse_fenced_workspace)?;
            if request.base_commit.is_none() || request.base_commit != placement.base_commit {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            None
        };
        let registry = self.registry()?;
        if let Some(selection) = &routing {
            self.check_routing_connection(&registry, request, selection)?;
        }
        let (mut mapping, source_cwd) = self.mapping_for(
            request,
            &registry,
            routing.as_ref().map(|selection| selection.workload),
        )?;
        let cwd = if is_view {
            let original = original.as_ref().ok_or(WorkerRefusal::CommandMismatch)?;
            let reference = original
                .commit_view
                .as_ref()
                .ok_or(WorkerRefusal::CommandMismatch)?;
            let private = self
                .store
                .view_root()
                .canonicalize()
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if Path::new(&reference.proof_root) != private
                || Path::new(&original.toplevel) != source_cwd
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            let intent = crate::workspace_git::ViewProofs::open(&private)
                .and_then(|proofs| proofs.load(&reference.binding.key()?))
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if intent
                .proof_digest()
                .map_err(|_| WorkerRefusal::PlacementMismatch)?
                != reference.intent_digest
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            let (root, cwd) = intent
                .execution_paths(&mapping.remote_root)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            mapping.remote_root =
                crate::registry::validate_unprotected_local_path(&root, &self.protected())
                    .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            let cwd = crate::registry::validate_unprotected_local_path(&cwd, &self.protected())
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if !cwd.starts_with(&mapping.remote_root) {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            cwd
        } else if let Some(original) = &original {
            self.remap_prepared_worktree(&mut mapping, &source_cwd, original)?
        } else {
            Self::verify_base_commit(&source_cwd, request.base_commit.as_deref())?;
            source_cwd
        };
        if cwd != Path::new(&placement.execution_cwd) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        if let Some(original) = &original
            && !is_view
        {
            let report = crate::workspace_git::report(original)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if report.head_commit.as_deref() != Some(original.base_commit.as_str())
                || report.head_on_branch != Some(true)
            {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let readiness = if is_view {
            self.readiness_at(&registry, request, &mapping, &cwd, &mut operation, None)?
        } else {
            self.inspect_candidate(&registry, request, &mapping, &cwd, &mut operation, None)?
        };
        operation
            .finish()
            .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        if let Some(value) = fenced_routing {
            if readiness.definition.digest != value.expected_definition_digest {
                return Err(WorkerRefusal::DefinitionChanged);
            }
            if json_digest(&readiness.child_policy).map_err(refuse)?
                != value.expected_permission_digest
            {
                return Err(WorkerRefusal::PermissionsChanged);
            }
        }
        let expected = RemotePlacement {
            machine_id: self.identity.machine_id.clone(),
            worker_id: self.identity.worker_id.clone(),
            provisioning_revision: self.identity.provisioning_revision,
            project_id: request.project_id.clone(),
            mapping_revision: request.mapping_revision,
            origin_host: request.origin_host.clone(),
            origin_cwd: request.origin_cwd.clone(),
            source_root_identity: request.source_root_identity.clone(),
            relative_cwd: request.relative_cwd.clone(),
            execution_cwd: cwd.to_string_lossy().into_owned(),
            base_commit: request.base_commit.clone(),
            definition_digest: readiness.definition.digest.clone(),
            permission_digest: json_digest(&readiness.child_policy).map_err(refuse)?,
        };
        if expected.definition_digest != placement.definition_digest {
            return Err(WorkerRefusal::DefinitionChanged);
        }
        if expected.permission_digest != placement.permission_digest {
            return Err(WorkerRefusal::PermissionsChanged);
        }
        if expected != *placement {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        let workspace_value = original
            .as_ref()
            .map(|value| serde_json::to_value(value).map_err(unavailable))
            .transpose()?;
        if let Some(retained) = self
            .prepared
            .get(&command.command_id)
            .filter(|retained| retained.expires > Instant::now())
        {
            if retained.command != *command
                || fenced_prepare_canon(&retained.request) != fenced_prepare_canon(request)
                || retained.routing != routing
                || retained.prepared.placement != *placement
                || retained.prepared.workspace != workspace_value
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            if let Some(workspace) = workspace {
                self.store
                    .bind_fenced_workspace(previous, command, request, placement, workspace)
                    .map_err(refuse_fenced_workspace)?;
            } else {
                self.store
                    .fenced_placement(previous, placement)
                    .map_err(refuse_fenced_workspace)?;
            }
            // Le canon durable exclut le numéro de tour, pas la preuve fraîche
            // qui devra signer Release. Garder le token sans garder l'ancienne révision.
            let renewed = PreparedRuntime {
                routing: retained.routing.clone(),
                command: retained.command.clone(),
                request: request.clone(),
                prepared: retained.prepared.clone(),
                expires: retained.expires,
            };
            self.read_view().recheck_at_with_view_report(
                &renewed,
                true,
                None,
                &read::ReadTrace::default(),
                &mut None,
                view_report,
            )?;
            let prepared = renewed.prepared.clone();
            self.prepared.insert(command.command_id.clone(), renewed);
            return Ok(prepared);
        }
        if let Some(workspace) = workspace {
            self.store
                .bind_fenced_workspace(previous, command, request, placement, workspace)
                .map_err(refuse_fenced_workspace)?;
        } else {
            self.store
                .fenced_placement(previous, placement)
                .map_err(refuse_fenced_workspace)?;
        }
        let now = worker_now();
        let prepared = WorkerPrepared {
            placement: placement.clone(),
            workspace: workspace_value,
            token: uuid::Uuid::new_v4().to_string(),
            definition: readiness.definition,
            child_policy: readiness.child_policy,
            effective_posture: readiness.effective_posture,
            prepared_at: now,
        };
        self.prepared.insert(
            command.command_id.clone(),
            PreparedRuntime {
                routing,
                command: command.clone(),
                request: request.clone(),
                prepared: prepared.clone(),
                expires: Instant::now() + Duration::from_secs(30),
            },
        );
        Ok(prepared)
    }

    #[cfg(test)]
    fn recheck(&self, prepared: &PreparedRuntime) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at(prepared, true)
    }

    #[cfg(test)]
    fn recheck_at(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at_with_repair(prepared, before_release, None)
    }
    fn recheck_at_with_repair(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
        repair: Option<&crate::workspace_git::RepairAnchor>,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        self.read_view().recheck_at_with_repair(
            prepared,
            before_release,
            repair,
            &read::ReadTrace::default(),
        )
    }

    fn start(
        &mut self,
        command: &WorkerCommandIdentity,
        start: &WorkerStart,
        disconnected: &AtomicBool,
        capacity_cause: &mut WorkerStartCapacityCause,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        self.start_with_view_report(command, start, disconnected, capacity_cause, None)
    }

    fn start_with_view_report(
        &mut self,
        command: &WorkerCommandIdentity,
        start: &WorkerStart,
        disconnected: &AtomicBool,
        capacity_cause: &mut WorkerStartCapacityCause,
        view_report: Option<&crate::delegation::WorkspaceRecord>,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        // Même un rejeu durable ne transforme jamais une inspection en Start.
        if start.prepared_token == bridget_transport::protocol::WORKER_INSPECTION_TOKEN {
            return Err(WorkerRefusal::InvalidEnvelope);
        }
        let digest = worker_command_digest(&self.identity, command, start).map_err(refuse)?;
        // Le rejeu durable ne dépend pas d'un token volatil toujours présent.
        match self.store.command(command) {
            Ok(receipt) => {
                if receipt.pre_start_fence.is_some() {
                    return Err(WorkerRefusal::CommandMismatch);
                }
                if receipt.command_digest != digest || receipt.placement != start.placement {
                    return Err(WorkerRefusal::CommandMismatch);
                }
                return Ok(Self::observation_native(
                    receipt.phase,
                    receipt.native_ready,
                    None,
                ));
            }
            Err(WorkerStoreError::Unknown) => {}
            Err(WorkerStoreError::Conflict) => return Err(WorkerRefusal::CommandMismatch),
            Err(error) => return Err(refuse(error)),
        }
        let prepared = self
            .prepared
            .get(&command.command_id)
            .ok_or(WorkerRefusal::PrepareExpired)?;
        if prepared.expires <= Instant::now() || prepared.prepared.token != start.prepared_token {
            return Err(WorkerRefusal::PrepareExpired);
        }
        if prepared.command.command_id != command.command_id
            || prepared.command.task_id != command.task_id
            || prepared.prepared.placement != start.placement
            || prepared.prepared.definition != start.definition
            || prepared.prepared.child_policy != start.child_policy
            || prepared.prepared.effective_posture != start.effective_posture
            || !safe_component(&start.child_id)
            || !safe_component(&command.instance_id)
        {
            return Err(WorkerRefusal::CommandMismatch);
        }
        let registry = self.read_view().recheck_at_with_view_report(
            prepared,
            true,
            None,
            &read::ReadTrace::default(),
            &mut None,
            view_report,
        )?;
        // Le quota est testé avant de retirer le token volatil.
        match self
            .store
            .reserve_command(
                command,
                &digest,
                &start.placement,
                start.deadline_at,
                worker_now(),
            )
            .map_err(|error| {
                if matches!(error, WorkerStoreError::Quota) {
                    *capacity_cause = WorkerStartCapacityCause::CommandStore;
                }
                refuse(error)
            })? {
            CommandReservation::Existing(receipt) => {
                return Ok(Self::observation_native(
                    receipt.phase,
                    receipt.native_ready,
                    None,
                ));
            }
            CommandReservation::Created(_) => {}
        }
        let prepared = self
            .prepared
            .remove(&command.command_id)
            .ok_or(WorkerRefusal::PrepareExpired)?;
        // Toute erreur de ce bloc précède une naissance réussie : le helper
        // de spawn ne fait aucune opération faillible après command.spawn().
        let launch_result = (|| {
            let cancel_key = json_digest(command).map_err(refuse)?;
            self.store
                .bind_preparation(command, &prepared)
                .map_err(|error| {
                    if matches!(error, WorkerStoreError::Quota) {
                        *capacity_cause = WorkerStartCapacityCause::PreparationMetadata;
                    }
                    refuse(error)
                })?;
            self.store
                .bind_launch(command, &start.child_id)
                .map_err(refuse)?;
            let lease = crate::fleet::SpawnLease {
                command_id: command.command_id.clone(),
                name: start.child_id.clone(),
                instance_id: command.instance_id.clone(),
                generation: command.generation,
                deadline_at: start.deadline_at,
                persistent: false,
                project: None,
                link_id: None,
                ownership: None,
                agent_path: None,
                remote_placement: Some(start.placement.clone()),
            };
            let mut launch = crate::lifecycle::prepare_spawn_parts(
                &registry,
                &self.source,
                &prepared.request.agent_type,
                Path::new(&start.placement.execution_cwd),
                lease,
                &crate::lifecycle::SpawnHosts {
                    searched_on: self.identity.machine_id.clone(),
                    requested_from: start.placement.origin_host.clone(),
                },
            )
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
            launch.env.insert(
                crate::native_permissions::CHILD_POLICY_ENV.into(),
                OsString::from(serde_json::to_string(&start.child_policy).map_err(unavailable)?),
            );
            let sources = start
                .child_policy
                .get("launch_context")
                .map(|value| vec![value.clone()])
                .unwrap_or_default();
            launch.env.insert(
                crate::native_permissions::CHILD_SOURCES_ENV.into(),
                OsString::from(serde_json::to_string(&sources).map_err(unavailable)?),
            );
            launch.env.insert(
                crate::wrapper::NATIVE_MISSION_BOOTSTRAP_ENV.into(),
                OsString::from(
                    serde_json::to_string(&crate::wrapper::NativeMissionBootstrap {
                        instance_id: command.instance_id.clone(),
                        mission_id: start.mission_id.clone(),
                    })
                    .map_err(unavailable)?,
                ),
            );
            launch.env.remove(crate::wrapper::NATIVE_CHILD_REQUESTS_ENV);
            if start.forward_child_requests {
                launch.env.insert(
                    crate::wrapper::NATIVE_CHILD_REQUESTS_ENV.into(),
                    OsString::from("forward"),
                );
            }
            launch.env.insert(
                "BRIDGET_REMOTE_RESULT_CONTEXT".into(),
                OsString::from(
                    serde_json::to_string(&serde_json::json!({
            "worker":self.identity,"command":command,"store_root":self.store.root }))
                    .map_err(unavailable)?,
                ),
            );
            let executable = std::env::current_exe().map_err(unavailable)?;
            let managed = ManagedLaunch {
                bootstrap_executable: executable.clone(),
                wrapper_executable: executable,
                identity: ManagedIdentity {
                    instance_id: command.instance_id.clone(),
                    command_id: command.command_id.clone(),
                    generation: command.generation,
                },
                wrapper_args: vec![
                    "managed-wrapper".into(),
                    prepared.request.agent_type.clone(),
                    start.child_id.clone(),
                    serde_json::to_string(&start.definition).map_err(unavailable)?,
                ],
                cwd: launch.cwd,
                env: launch.env,
            };
            // Conserver le diagnostic privé du bootstrap et du fournisseur,
            // comme pour un lancement local, sans le publier dans le protocole.
            let (stderr, _) = crate::managed_process::ManagedStderrStore::at_directory(
                self.store.root.join("managed-stderr"),
            )
            .open(&start.child_id, &managed.identity)
            .map_err(unavailable)?;
            let child = crate::managed_process::spawn_managed_bootstrap_with_stderr(
                &managed,
                std::process::Stdio::from(stderr),
            )
            .map_err(unavailable)?;
            Ok::<_, WorkerRefusal>((child, cancel_key))
        })();
        let (child, cancel_key) = match launch_result {
            Ok(launch) => launch,
            Err(error) => {
                self.store
                    .set_phase(command, WorkerCommandPhase::Refused, worker_now())
                    .map_err(refuse)?;
                return Err(error);
            }
        };
        let ready = match child.wait_ready_or_cancel_retained(
            || {
                disconnected.load(Ordering::Acquire)
                    || self
                        .cancelled
                        .lock()
                        .map_or(true, |cancelled| cancelled.contains(&cancel_key))
                    || prepared.expires <= Instant::now()
                    || worker_now() >= start.deadline_at
            },
            Duration::from_millis(25),
        ) {
            Ok(ready) => ready,
            Err((error, child)) => {
                self.abandoned.insert(command.command_id.clone(), child);
                return Err(unavailable(error));
            }
        };
        let Some(ready) = ready else {
            self.store
                .set_phase(command, WorkerCommandPhase::Refused, worker_now())
                .map_err(refuse)?;
            return Err(WorkerRefusal::PrepareExpired);
        };
        if ready.ready().instance_id != command.instance_id
            || ready.ready().command_id != command.command_id
            || ready.ready().generation != command.generation
        {
            self.abandoned
                .insert(command.command_id.clone(), ready.abandon());
            return Err(WorkerRefusal::CommandMismatch);
        }
        let marked = match ready.persist_marker_retained(&self.markers, &marker_key(command)) {
            Ok(marked) => marked,
            Err((error, child)) => {
                self.abandoned.insert(command.command_id.clone(), child);
                return Err(unavailable(error));
            }
        };
        if let Err(error) = self
            .store
            .mark_command(command, marked.marker().marker(), worker_now())
        {
            self.abandoned
                .insert(command.command_id.clone(), marked.abandon().0);
            return Err(refuse(error));
        }
        let ready_at = worker_now();
        if Instant::now() >= prepared.expires {
            self.abandoned
                .insert(command.command_id.clone(), marked.abandon().0);
            return Err(WorkerRefusal::PrepareExpired);
        }
        self.held.insert(
            command.command_id.clone(),
            HeldRuntime {
                prepared,
                mission_id: start.mission_id.clone(),
                child: marked,
                ready_at,
                expires: Instant::now() + Duration::from_secs(WORKER_RELEASE_LEASE_SECS as u64),
            },
        );
        Ok(WorkerResponseOperation::ReadyToRelease { ready_at })
    }

    fn fence_unstarted(
        &mut self,
        command: &WorkerCommandIdentity,
        placement: &RemotePlacement,
        deadline_at: i64,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        command.validate()?;
        placement.validate()?;
        if !placement.matches_worker(&self.identity) || deadline_at <= 0 {
            return Err(WorkerRefusal::InvalidEnvelope);
        }
        if self.held.contains_key(&command.command_id)
            || self.running.contains_key(&command.command_id)
            || self.abandoned.contains_key(&command.command_id)
        {
            return Err(WorkerRefusal::UnknownExecution);
        }
        self.stop_owned_view_sources(command)?;
        self.store
            .fence_unstarted(command, placement, deadline_at, worker_now())
            .map_err(|error| match error {
                WorkerStoreError::Conflict => WorkerRefusal::CommandMismatch,
                WorkerStoreError::Unknown => WorkerRefusal::UnknownExecution,
                error => refuse(error),
            })?;
        Ok(WorkerResponseOperation::StartFenced {})
    }

    fn release(
        &mut self,
        command: &WorkerCommandIdentity,
        permit: &WorkerReleasePermit,
        epoch: &str,
        disconnected: &AtomicBool,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        self.release_with_view_report(command, permit, epoch, disconnected, None)
    }

    fn release_with_view_report(
        &mut self,
        command: &WorkerCommandIdentity,
        permit: &WorkerReleasePermit,
        epoch: &str,
        disconnected: &AtomicBool,
        view_report: Option<&crate::delegation::WorkspaceRecord>,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        // Vérifier le tuple avant de retirer le bootstrap, même sur une erreur.
        self.store.command(command).map_err(|error| match error {
            WorkerStoreError::Conflict => WorkerRefusal::CommandMismatch,
            WorkerStoreError::Unknown => WorkerRefusal::UnknownExecution,
            error => refuse(error),
        })?;
        let held = self
            .held
            .remove(&command.command_id)
            .ok_or(WorkerRefusal::UnknownExecution)?;
        let decision = (|| {
            if Instant::now() >= held.expires
                || permit.expires_at > held.ready_at.saturating_add(WORKER_RELEASE_LEASE_SECS)
                || permit.issued_at < held.ready_at
                || permit.permission_revision != held.prepared.request.parent.revision
            {
                return Err(WorkerRefusal::ReleaseExpired);
            }
            self.read_view().recheck_at_with_view_report(
                &held.prepared,
                true,
                None,
                &read::ReadTrace::default(),
                &mut None,
                view_report,
            )?;
            let (_, slots, _, _) = self.store.inventory().map_err(refuse)?;
            if !slots.iter().any(|slot| {
                slot.slot.identity.command == *command
                    && slot.slot.identity.mission_id == held.mission_id
                    && slot.slot.identity.round == 1
                    && slot.closed.is_none()
                    && !slot.producer_terminal
            }) {
                return Err(WorkerRefusal::ResourceLimit);
            }
            let key = json_digest(command).map_err(refuse)?;
            if Instant::now() >= held.expires
                || disconnected.load(Ordering::Acquire)
                || self
                    .cancelled
                    .lock()
                    .map_or(true, |cancelled| cancelled.contains(&key))
            {
                return Err(WorkerRefusal::ReleaseExpired);
            }
            self.store
                .authorize_release(command, permit, epoch, worker_now())
                .map_err(refuse)
        })();
        if let Err(error) = decision {
            self.abandoned
                .insert(command.command_id.clone(), held.child.abandon().0);
            return Err(error);
        }
        let (child, release_error) = match held.child.release_retained() {
            Ok(child) => (child, None),
            Err((error, child)) => (child, Some(error)),
        };
        // L'autorité sur le processus est conservée avant toute autre opération
        // faillible, y compris la configuration de son canal et le stockage.
        self.running.insert(command.command_id.clone(), child);
        let child = self.running.get_mut(&command.command_id).unwrap();
        let error = release_error.or_else(|| child.set_status_nonblocking().err().map(Into::into));
        if let Some(error) = error {
            // Le canal porte un échec transitoire persistant en mémoire : sa
            // fermeture ne peut pas devenir NativeReady si le store échoue.
            let stopped = child.request_group_termination();
            let persisted = self.store.native_status(command, true);
            stopped.map_err(unavailable)?;
            persisted.map_err(refuse)?;
            return Err(unavailable(error));
        }
        self.store
            .set_phase(command, WorkerCommandPhase::Released, worker_now())
            .map_err(refuse)?;
        Ok(WorkerResponseOperation::Released {})
    }

    fn observation(phase: WorkerCommandPhase, exit_code: Option<i32>) -> WorkerResponseOperation {
        Self::observation_native(phase, false, exit_code)
    }
    fn observation_native(
        phase: WorkerCommandPhase,
        native_ready: bool,
        exit_code: Option<i32>,
    ) -> WorkerResponseOperation {
        WorkerResponseOperation::Observation {
            observation: WorkerObservation {
                startup_failure: None,
                phase,
                native_ready: native_ready && phase == WorkerCommandPhase::Released,
                exit_code,
                observed_at: worker_now(),
                workspace: None,
                permission_digest: None,
            },
        }
    }
    /// Observation sans effet : aucun signal, wait, rapport Git, écriture ou
    /// clôture de créneau. Une preuve insuffisante n'est jamais un arrêt.
    fn execution_status(
        &self,
        command: &WorkerCommandIdentity,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let receipt = self.store.command(command).map_err(refuse)?;
        let mut phase = receipt.phase;
        let mut ready = false;
        if phase == WorkerCommandPhase::Released {
            if let Some(marker) = receipt.marker.as_ref().filter(|m| {
                m.command_id == command.command_id
                    && m.instance_id == command.instance_id
                    && m.generation == command.generation
            }) {
                ready = receipt.native_ready
                    && !receipt.startup_failed
                    && crate::managed_process::process_matches_live(marker.pgid, marker.birth)
                        .map_err(unavailable)?;
            }
            if !ready {
                phase = WorkerCommandPhase::Unknown;
            }
        }
        let mut response = Self::observation_native(phase, ready, None);
        if let WorkerResponseOperation::Observation { observation } = &mut response {
            observation.startup_failure = receipt.startup_failure;
            observation.exit_code = receipt.terminal_exit_code;
        }
        Ok(response)
    }

    fn reconcile(
        &mut self,
        command: &WorkerCommandIdentity,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let mut response = self.reconcile_inner(command)?;
        let receipt = self.store.command(command).map_err(refuse)?;
        if let WorkerResponseOperation::Observation { observation } = &mut response {
            observation.startup_failure = receipt.startup_failure;
            if observation.phase == WorkerCommandPhase::Terminal {
                observation.exit_code = receipt.terminal_exit_code.or(observation.exit_code);
            }
        }
        if matches!(&response,WorkerResponseOperation::Observation {observation}
            if matches!(observation.phase,WorkerCommandPhase::Terminal|WorkerCommandPhase::Refused))
            && !self
                .store
                .settle_view_command_sources(command)
                .map_err(refuse)?
        {
            return Err(WorkerRefusal::UnknownExecution);
        }
        Ok(response)
    }

    fn reconcile_inner(
        &mut self,
        command: &WorkerCommandIdentity,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let receipt = self.store.command(command).map_err(refuse)?;
        if matches!(
            receipt.phase,
            WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
        ) {
            // Une panne peut séparer la preuve terminale de la clôture des
            // réservations. Reprendre cette dernière avant de publier la fin.
            self.finish_result_slots(command)?;
            return Ok(Self::observation_native(
                receipt.phase,
                receipt.native_ready,
                None,
            ));
        }
        if self.held.contains_key(&command.command_id) {
            return Ok(Self::observation_native(
                receipt.phase,
                receipt.native_ready,
                None,
            ));
        }
        if let Some(child) = self.running.get_mut(&command.command_id) {
            let exit = child.try_wait().map_err(unavailable)?;
            let status = child.try_status();
            use bridget_transport::protocol::WorkerStartupFailure as Failure;
            let failure = match &status {
                Ok(Some(crate::managed_process::ManagedStatus::StartupFailed {
                    kind,
                    instance_id,
                    command_id,
                    generation,
                    ..
                })) if instance_id == &command.instance_id
                    && command_id == &command.command_id
                    && *generation == command.generation =>
                {
                    Some(match kind.as_str() {
                        "command_missing" => Failure::CommandMissing,
                        _ => Failure::NegotiationFailed,
                    })
                }
                Ok(Some(_)) | Err(_) => Some(Failure::InvalidStatus),
                _ if exit.is_some() && !receipt.native_ready => Some(Failure::ExitedBeforeReady),
                _ => None,
            };
            self.store
                .record_startup_outcome(command, failure, exit.and_then(|s| s.code()))
                .map_err(refuse)?;
            // Lire la preuve après la fin : le wrapper la publie avant sa sortie.
            if exit.is_some() && !self.store.drainage_proven(&receipt).unwrap_or(false) {
                self.store
                    .set_phase(command, WorkerCommandPhase::Unknown, worker_now())
                    .map_err(refuse)?;
                return Ok(Self::observation(WorkerCommandPhase::Unknown, None));
            }
            if exit.is_some() && child.reap_terminated_group().map_err(unavailable)? {
                self.running.remove(&command.command_id);
                self.finish_command(command)?;
                return Ok(Self::observation(
                    WorkerCommandPhase::Terminal,
                    exit.and_then(|s| s.code()),
                ));
            }
            let mut failed = receipt.startup_failed;
            // Toute trame après BootstrapReady est un échec explicite
            // ou une seconde ready interdite, comme le superviseur local.
            if matches!(status, Ok(Some(_)) | Err(_)) {
                failed = true;
            }
            let native_ready = !failed && !child.status_is_open() && exit.is_none();
            if failed && !receipt.startup_failed {
                let stopped = child.request_group_termination();
                let persisted = self.store.native_status(command, true);
                stopped.map_err(unavailable)?;
                persisted.map_err(refuse)?;
            } else if native_ready && !receipt.native_ready {
                self.store.native_status(command, false).map_err(refuse)?;
            }
            return Ok(Self::observation_native(
                receipt.phase,
                native_ready || (receipt.native_ready && !failed),
                None,
            ));
        }
        if let Some(child) = self.abandoned.get_mut(&command.command_id) {
            if child.try_wait().map_err(unavailable)?.is_some() {
                if !self.store.drainage_proven(&receipt).unwrap_or(false) {
                    self.store
                        .set_phase(command, WorkerCommandPhase::Unknown, worker_now())
                        .map_err(refuse)?;
                    return Ok(Self::observation(WorkerCommandPhase::Unknown, None));
                }
                // Garder la preuve récoltée tant que sa clôture durable échoue.
                self.finish_command(command)?;
                self.abandoned.remove(&command.command_id);
                return Ok(Self::observation(WorkerCommandPhase::Terminal, None));
            }
            return Ok(Self::observation_native(
                receipt.phase,
                receipt.native_ready,
                None,
            ));
        }
        let Some(marker) = receipt.marker.as_ref() else {
            self.store
                .set_phase(command, WorkerCommandPhase::Unknown, worker_now())
                .map_err(refuse)?;
            return Ok(Self::observation(WorkerCommandPhase::Unknown, None));
        };
        let live = crate::managed_process::group_exists(marker.pgid).map_err(unavailable)?;
        if !live {
            // Enveloppe tuée avant sa preuve (incident 171, c3b80e27) : sans
            // autre preuve, l'annulation restait « unknown » pour toujours.
            // Un groupe de contrôle qui ne contient plus que ce worker, sans
            // enfant suivi, prouve qu'aucun descendant de la commande ne vit.
            let only_worker = self.running.is_empty()
                && self.abandoned.is_empty()
                && service_cgroup_holds_only_self();
            if !only_worker && !self.store.drainage_proven(&receipt).unwrap_or(false) {
                self.store
                    .set_phase(command, WorkerCommandPhase::Unknown, worker_now())
                    .map_err(refuse)?;
                return Ok(Self::observation(WorkerCommandPhase::Unknown, None));
            }
            self.finish_command(command)?;
            return Ok(Self::observation(WorkerCommandPhase::Terminal, None));
        }
        // Une naissance différente, ou un leader absent avec des descendants,
        // ne prouve pas que le groupe de la commande est soldé.
        let phase = if crate::managed_process::process_birth(marker.pgid).ok() == Some(marker.birth)
            && matches!(receipt.phase, WorkerCommandPhase::Released)
        {
            receipt.phase
        } else {
            WorkerCommandPhase::Unknown
        };
        if phase == WorkerCommandPhase::Unknown {
            self.store
                .set_phase(command, phase, worker_now())
                .map_err(refuse)?;
        }
        Ok(Self::observation_native(
            phase,
            receipt.native_ready && !receipt.startup_failed,
            None,
        ))
    }
    /// Lecture explicite seulement : le tick de supervision ne lance pas Git.
    fn reconcile_attested(
        &mut self,
        command: &WorkerCommandIdentity,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let mut response = self.reconcile(command)?;
        let receipt = self.store.command(command).map_err(refuse)?;
        let Some((request, prepared)) = receipt.preparation else {
            return Ok(response);
        };
        if prepared.placement != receipt.placement
            || json_digest(&prepared.child_policy).map_err(refuse)?
                != receipt.placement.permission_digest
        {
            return Err(WorkerRefusal::CommandMismatch);
        }
        let mut prepare_command = command.clone();
        prepare_command.instance_id.clear();
        prepare_command.generation = 0;
        let mut retained = PreparedRuntime {
            routing: receipt.routing,
            command: prepare_command,
            request,
            prepared,
            expires: Instant::now(),
        };
        let repair = receipt
            .workspace_repair
            .as_ref()
            .filter(|r| r.phase == "published");
        let anchor = repair
            .and_then(|r| workspace_repair::proof(r).ok())
            .map(|v| v.0);
        if let Some(prepared) = repair.and_then(|r| r.prepared.as_ref()) {
            retained.prepared = prepared.clone();
        }
        if let WorkerResponseOperation::Observation { observation } = &mut response {
            if receipt
                .workspace_repair
                .as_ref()
                .is_some_and(|r| r.blocks_dispatch())
            {
                observation.native_ready = false;
                return Ok(response);
            }
            let mut workspace_report = None;
            // Un seul rapport frais sert au contrôle et à la réponse. Aucune
            // preuve n'est conservée entre deux Reconcile.
            let permitted = observation.phase == WorkerCommandPhase::Released
                && observation.native_ready
                && self
                    .read_view()
                    .recheck_at_with_workspace(
                        &retained,
                        false,
                        anchor.as_ref(),
                        &read::ReadTrace::default(),
                        &mut workspace_report,
                    )
                    .is_ok();
            // Une clôture ou un refus précoce des droits garde son observation
            // technique, sans devenir une autorisation de remise.
            if workspace_report.is_none()
                && let Some(value) = retained.prepared.workspace.as_ref()
                && let Ok(record) =
                    serde_json::from_value::<crate::delegation::WorkspaceRecord>(value.clone())
            {
                workspace_report = Some(
                    if matches!(
                        record.strategy,
                        crate::delegation::WorkspaceStrategy::CommitView
                    ) {
                        self.store
                            .report_retained_view(&retained.command, &record)
                            .map_err(refuse_fenced_workspace)
                    } else {
                        if let Some(anchor) = anchor.as_ref() {
                            crate::workspace_git::report_repaired_workspace(&record, anchor)
                        } else {
                            crate::workspace_git::report(&record)
                        }
                        .map_err(|_| WorkerRefusal::PlacementMismatch)
                    },
                );
            }
            observation.workspace = workspace_report
                .and_then(Result::ok)
                .and_then(|report| serde_json::to_value(report).ok());
            if permitted {
                observation.permission_digest =
                    Some(retained.prepared.placement.permission_digest.clone());
            }
        }
        Ok(response)
    }

    fn finish_command(&self, command: &WorkerCommandIdentity) -> Result<(), WorkerRefusal> {
        self.store
            .set_phase(command, WorkerCommandPhase::Terminal, worker_now())
            .map_err(refuse)?;
        self.finish_result_slots(command)
    }
    /// O(s) pour les slots bornés du store. L'arrêt attesté interdit toute
    /// nouvelle publication, mais les blobs restent relayés jusqu'à disposition.
    fn finish_result_slots(&self, command: &WorkerCommandIdentity) -> Result<(), WorkerRefusal> {
        for slot in self.store.inventory().map_err(refuse)?.1 {
            if slot.slot.identity.command == *command {
                self.finish_result_slot(&slot)?;
            }
        }
        Ok(())
    }
    fn finish_result_slot(&self, slot: &SlotReceipt) -> Result<(), WorkerRefusal> {
        if !slot.producer_terminal {
            self.store
                .mark_result_producer_terminal(&slot.slot.identity)
                .map_err(refuse)?;
        }
        if slot.closed.is_none() {
            self.store
                .close_result_slot(&slot.slot, WorkerSlotClosureProof::Cancelled, worker_now())
                .map_err(refuse)?;
        }
        Ok(())
    }
    fn reconcile_all(&mut self) -> Result<(), WorkerRefusal> {
        // Un seul inventaire pour les commandes terminales conservées sept jours.
        // Index en O(c log c), jointure en O(s log c), aucune relecture globale
        // par commande terminale. Les transitions actives gardent leur preuve fraîche.
        let (commands, slots, _, _) = {
            let _guard = self.store.transition.lock().map_err(unavailable)?;
            self.store.health().map_err(refuse)?;
            self.store.inventory().map_err(refuse)?
        };
        let terminal: BTreeMap<_, _> = commands
            .iter()
            .filter(|command| {
                matches!(
                    command.phase,
                    WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                )
            })
            .map(|command| (command.identity.command_id.as_str(), &command.identity))
            .collect();
        for slot in &slots {
            if terminal
                .get(slot.slot.identity.command.command_id.as_str())
                .is_some_and(|identity| **identity == slot.slot.identity.command)
            {
                self.finish_result_slot(slot)?;
            }
        }
        drop(terminal);
        let mut retained_commands = commands
            .iter()
            .map(|command| json_digest(&command.identity))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(unavailable)?;
        for receipt in self.store.view_receipts(None).map_err(refuse)? {
            let view = receipt.commit_view.as_ref().unwrap();
            retained_commands.insert(format!("view-source-{}", view.key));
            if let Some(operation) = &view.materialization {
                let binding = crate::workspace_git::ViewBinding {
                    worker: self.identity.clone(),
                    command: receipt.command.clone(),
                    prepare_digest: receipt.prepare_digest.clone(),
                };
                let identity = operation
                    .request
                    .identity_for_binding(&binding)
                    .map_err(|_| WorkerRefusal::CommandMismatch)?;
                retained_commands.insert(format!(
                    "view-source-{}-file-{}",
                    view.key, identity.command_id
                ));
            }
        }
        self.cancel_signal_issued
            .retain(|key| retained_commands.contains(key));
        for command in commands {
            if matches!(
                command.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) {
                continue;
            }
            let cancel_key = json_digest(&command.identity).map_err(unavailable)?;
            let cancel_requested = self
                .cancelled
                .lock()
                .map_err(unavailable)?
                .contains(&cancel_key);
            let bootstrap_expired = matches!(
                command.phase,
                WorkerCommandPhase::Reserved
                    | WorkerCommandPhase::Marked
                    | WorkerCommandPhase::ReleaseAuthorized
            ) && command.deadline_at <= worker_now();
            if cancel_requested || command.startup_failed || bootstrap_expired {
                match self.cancel(&command.identity) {
                    Ok(_) => continue,
                    Err(reason) => log::warn!("worker_cancel_pending reason={reason:?}"),
                }
            }
            // L'absence de preuve de signal n'empêche pas l'observation : une
            // commande sans marqueur devient Unknown, jamais Terminal par défaut.
            if let Err(reason) = self.reconcile(&command.identity) {
                log::warn!("worker_reconcile_pending reason={reason:?}");
            }
        }
        Ok(())
    }
    /// O(n), n <= 128 par famille : préserver l'identité d'exécution des
    /// fichiers, y compris quand leur ancre appartient à une préparation antérieure.
    fn retain_view_cancellations(&self) -> Result<(), String> {
        let commands = self
            .view_preparations
            .values()
            .map(|pending| &pending.read.source.binding.command)
            .chain(
                self.view_sources
                    .values()
                    .map(|source| &source.binding.command),
            )
            .chain(
                self.view_reports
                    .values()
                    .map(|report| &report.binding.command),
            )
            .chain(
                self.view_files
                    .values()
                    .map(|pending| &pending.read.execution.command),
            )
            .chain(self.view_sources.values().filter_map(|source| {
                source
                    .materialization
                    .as_ref()
                    .map(|(execution, _)| &execution.command)
            }))
            .chain(
                self.view_file_audits
                    .values()
                    .map(|audit| &audit.execution.command),
            );
        let retained = commands
            .map(json_digest)
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|_| "worker_cancel_identity_invalid")?;
        self.cancelled
            .lock()
            .map_err(|_| "worker_cancel_poisoned")?
            .retain(|key| retained.contains(key) || self.view_controls.contains_key(key));
        Ok(())
    }

    fn abandon_held(&mut self) {
        for pending in self.view_preparations.values_mut() {
            pending.correlations.clear();
        }
        for pending in self.view_controls.values_mut() {
            pending.correlations.clear();
        }
        for pending in self.view_files.values_mut() {
            pending.correlations.clear();
        }
        self.prepared.clear();
        for (command, held) in std::mem::take(&mut self.held) {
            self.abandoned.insert(command, held.child.abandon().0);
        }
    }
    fn tick(&mut self) -> Result<(), WorkerRefusal> {
        self.collect_view_sources();
        self.prepared
            .retain(|_, value| value.expires > Instant::now());
        let expired = self
            .held
            .iter()
            .filter(|(_, held)| held.expires <= Instant::now())
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in expired {
            if let Some(held) = self.held.remove(&id) {
                self.abandoned.insert(id, held.child.abandon().0);
            }
        }
        if self.last_maintenance.elapsed() >= Duration::from_secs(1) {
            self.reconcile_all()?;
            self.store.collect(worker_now()).map_err(refuse)?;
            self.last_maintenance = Instant::now();
        }
        if self.last_resource_refresh.elapsed() >= Duration::from_secs(10) {
            let capacity = self
                .registry()
                .ok()
                .and_then(|registry| self.compilation_capacity(&registry));
            let _ = self.host_resources.collect(worker_now(), capacity);
            self.last_resource_refresh = Instant::now();
        }
        Ok(())
    }

    fn handle(
        &mut self,
        request: &WorkerRequest,
        epoch: &str,
        disconnected: &AtomicBool,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        crate::workspace_failure::clear();
        if let Err(reason) = request.validate() {
            return if matches!(
                request.request,
                WorkerRequestOperation::StartDetailed { .. }
            ) {
                Ok(WorkerResponseOperation::StartRejected {
                    reason,
                    cause: if reason == WorkerRefusal::ResourceLimit {
                        WorkerStartCapacityCause::RequestFrame
                    } else {
                        WorkerStartCapacityCause::Unknown
                    },
                })
            } else {
                Err(reason)
            };
        }
        if request.worker != self.identity
            || request.connection_epoch != epoch
            || disconnected.load(Ordering::Acquire)
        {
            return if matches!(
                request.request,
                WorkerRequestOperation::StartDetailed { .. }
            ) {
                Ok(WorkerResponseOperation::StartRejected {
                    reason: WorkerRefusal::StaleConnection,
                    cause: WorkerStartCapacityCause::Unknown,
                })
            } else {
                Err(WorkerRefusal::StaleConnection)
            };
        }
        if request.request.requires_provider_accounts() && !self.provider_accounts_negotiated {
            return if matches!(
                request.request,
                WorkerRequestOperation::StartDetailed { .. }
            ) {
                Ok(WorkerResponseOperation::StartRejected {
                    reason: WorkerRefusal::CapabilityRequired,
                    cause: WorkerStartCapacityCause::Unknown,
                })
            } else {
                Err(WorkerRefusal::CapabilityRequired)
            };
        }
        match &request.request {
            // Le port différé possède le helper ; jamais de boucle d'attente ici.
            WorkerRequestOperation::ReadCommitViewFile { .. } => {
                Err(WorkerRefusal::CapabilityRequired)
            }
            WorkerRequestOperation::ConserveCommitView { prepare_digest } => {
                self.conserve_commit_view(&request.command, prepare_digest)
            }
            WorkerRequestOperation::PrepareDiagnosed { prepare } => {
                let mut inner = request.clone();
                inner.request = *prepare.clone();
                let outcome = self.handle(&inner, epoch, disconnected);
                let failure = crate::workspace_failure::take();
                match (outcome, failure) {
                    (Err(reason), Some(failure)) if failure.validate(reason).is_ok() => {
                        eprintln!(
                            "bridget_workspace_prepare210 {}",
                            serde_json::json!({
                            "command":request.command, "failure":failure, "reason":reason})
                        );
                        Ok(WorkerResponseOperation::WorkspaceRejected { reason, failure })
                    }
                    (outcome, _) => outcome,
                }
            }
            WorkerRequestOperation::Prepare { prepare } => self
                .prepare(&request.command, prepare)
                .map(|prepared| WorkerResponseOperation::Prepared { prepared }),
            WorkerRequestOperation::PrepareV2 {
                prepare,
                inspect_only,
                workload,
                connection_id,
                engine,
                expected_definition_digest,
                expected_permission_digest,
            } => self
                .prepare_for(
                    &request.command,
                    prepare,
                    Some((
                        RoutingPreparation {
                            connection_id: connection_id.clone(),
                            engine: engine.clone(),
                            workload: *workload,
                        },
                        expected_definition_digest,
                        expected_permission_digest,
                    )),
                    *inspect_only,
                )
                .map(|prepared| WorkerResponseOperation::Prepared { prepared }),
            WorkerRequestOperation::PrepareFenced {
                previous,
                prepare,
                placement,
                workspace,
                routing,
            } => self
                .prepare_fenced_routed(
                    &request.command,
                    previous,
                    prepare,
                    placement,
                    workspace.as_ref(),
                    routing.as_ref(),
                )
                .map(|prepared| WorkerResponseOperation::Prepared { prepared }),
            WorkerRequestOperation::Start { start } => self.start(
                &request.command,
                start,
                disconnected,
                &mut WorkerStartCapacityCause::Unknown,
            ),
            WorkerRequestOperation::StartDetailed { start } => {
                let mut cause = WorkerStartCapacityCause::Unknown;
                match self.start(&request.command, start, disconnected, &mut cause) {
                    Ok(response) => Ok(response),
                    Err(reason) => Ok(WorkerResponseOperation::StartRejected {
                        reason,
                        cause: if reason == WorkerRefusal::ResourceLimit {
                            cause
                        } else {
                            WorkerStartCapacityCause::Unknown
                        },
                    }),
                }
            }
            WorkerRequestOperation::FenceUnstarted {
                placement,
                deadline_at,
            } => self.fence_unstarted(&request.command, placement, *deadline_at),
            WorkerRequestOperation::Release { permit } => {
                self.release(&request.command, permit, epoch, disconnected)
            }
            WorkerRequestOperation::ReserveResultSlot { slot }
            | WorkerRequestOperation::ReserveResultSlotUntil { slot, .. } => {
                let key = json_digest(&request.command).map_err(unavailable)?;
                if self
                    .cancelled
                    .lock()
                    .map_or(true, |cancelled| cancelled.contains(&key))
                {
                    return Err(WorkerRefusal::UnknownExecution);
                }
                let expires_at_ms = match &request.request {
                    WorkerRequestOperation::ReserveResultSlotUntil { expires_at_ms, .. } => {
                        Some(*expires_at_ms)
                    }
                    _ => None,
                };
                if expires_at_ms.is_some_and(|deadline| worker_now_millis() >= deadline) {
                    return Err(WorkerRefusal::ResultSlotExpired);
                }
                // Un reçu existant peut être relu après désactivation. Une nouvelle
                // ronde exige toujours une route active avec la révision admise.
                if self
                    .store
                    .read_slot(&slot.identity)
                    .map_err(refuse)?
                    .is_none()
                {
                    let receipt = self.store.command(&request.command).map_err(refuse)?;
                    let registry = self.registry()?;
                    let mapping = registry
                        .project_mapping(&self.identity.machine_id, &receipt.placement.project_id)
                        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
                    if mapping.mapping_revision != receipt.placement.mapping_revision {
                        return Err(WorkerRefusal::PlacementMismatch);
                    }
                }
                let reservation = self
                    .store
                    .reserve_result_slot_before(slot, worker_now(), expires_at_ms)
                    .map_err(refuse)?;
                Ok(WorkerResponseOperation::ResultSlotReserved {
                    slot: slot.clone(),
                    slot_id: reservation.slot_id,
                })
            }
            WorkerRequestOperation::ResultDisposition { result } => {
                self.store
                    .record_disposition(result, worker_now())
                    .map_err(refuse)?;
                Ok(WorkerResponseOperation::DispositionRecorded {
                    result: result.clone(),
                })
            }
            WorkerRequestOperation::RepairWorkspace {
                repair,
                action,
                expires_at,
            } => self.repair_workspace(&request.command, repair, *action, *expires_at),
            WorkerRequestOperation::Reconcile {} => self.reconcile_attested(&request.command),
            // Observation de commande seulement : pas de rapport Git ni de
            // relecture fournisseur. La remise garde Reconcile et ses droits.
            WorkerRequestOperation::ExecutionStatus {} => self.execution_status(&request.command),
            WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::ResultSlot { slot, proof },
            } => {
                self.store
                    .close_result_slot(slot, *proof, worker_now())
                    .map_err(refuse)?;
                self.reconcile(&request.command)
            }
            WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {},
            } => self.cancel(&request.command),
            WorkerRequestOperation::ParentProof {
                parent,
                expected_revision,
            } => {
                let receipt = self.store.command(&request.command).map_err(refuse)?;
                if !matches!(receipt.phase, WorkerCommandPhase::Released)
                    || receipt.effective_placement().execution_cwd != parent.cwd
                {
                    return Err(WorkerRefusal::Unauthorized);
                }
                let permission_digest =
                    crate::native_permissions::prove_parent_permissions(parent, *expected_revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                Ok(WorkerResponseOperation::ParentProof {
                    parent: parent.clone(),
                    permission_digest,
                })
            }
            WorkerRequestOperation::ParentSourceProof { parent, source } => {
                let receipt = self.store.command(&request.command).map_err(refuse)?;
                if receipt.phase != WorkerCommandPhase::Released
                    || receipt.effective_placement().execution_cwd != parent.cwd
                {
                    return Err(WorkerRefusal::Unauthorized);
                }
                let permission_digest =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let source_error = (|| {
                    let root = fs::canonicalize(&parent.cwd)
                        .map_err(|_| "workspace_base_preflight_failed")?;
                    let cwd = fs::canonicalize(&source.cwd)
                        .map_err(|_| "workspace_base_preflight_failed")?;
                    if !cwd.starts_with(&root) {
                        return Err("cwd_outside_parent_project".to_owned());
                    }
                    let mut canonical_source = source.clone();
                    canonical_source.cwd = cwd.to_string_lossy().into_owned();
                    crate::lifecycle::validate_parent_source_base(parent, &canonical_source)
                })()
                .err()
                .map(|mut code| {
                    code.truncate(code.floor_char_boundary(1024));
                    code
                });
                let after =
                    crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                if after != permission_digest {
                    return Err(WorkerRefusal::PermissionsChanged);
                }
                Ok(WorkerResponseOperation::ParentSourceProof {
                    permission_digest,
                    source_error,
                })
            }
            WorkerRequestOperation::JournalPage { after_seq, limit } => {
                self.journal(&request.command, *after_seq, *limit)
            }
        }
    }
    fn cancel(
        &mut self,
        command: &WorkerCommandIdentity,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let receipt = self.store.command(command).map_err(|error| match error {
            WorkerStoreError::Conflict => WorkerRefusal::CommandMismatch,
            WorkerStoreError::Unknown => WorkerRefusal::UnknownExecution,
            error => refuse(error),
        })?;
        let source_stop = self.stop_owned_view_sources(command);
        let result: Result<WorkerResponseOperation, WorkerRefusal> = (|| {
            let cancel_key = json_digest(command).map_err(unavailable)?;
            self.prepared.remove(&command.command_id);
            if let Some(held) = self.held.remove(&command.command_id) {
                self.abandoned
                    .insert(command.command_id.clone(), held.child.abandon().0);
                return self.reconcile(command);
            }
            if matches!(
                receipt.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) {
                return self.reconcile(command);
            }
            if self.cancel_signal_issued.contains(&cancel_key) {
                return self.reconcile(command);
            }
            let marker = receipt.marker.ok_or(WorkerRefusal::UnknownExecution)?;
            if marker.command_id != command.command_id
                || marker.instance_id != command.instance_id
                || marker.generation != command.generation
                || self
                    .markers
                    .load(&marker_key(command))
                    .map_err(unavailable)?
                    != marker
            {
                return Err(WorkerRefusal::UnknownExecution);
            }
            // Une annulation peut arriver après la sortie du wrapper. Reprendre la
            // clôture durable avant de demander l'identité d'un processus disparu.
            // reconcile conserve l'exigence de preuve de drainage après RELEASE.
            if !crate::managed_process::group_exists(marker.pgid).map_err(unavailable)? {
                return self.reconcile(command);
            }
            if crate::managed_process::process_birth(marker.pgid).ok() != Some(marker.birth) {
                return Err(WorkerRefusal::UnknownExecution);
            }
            // Un échec ne mémorise pas un signal réussi. Après envoi, les appels
            // suivants observent le drainage sans répéter le signal.
            let running = &mut self.running;
            let markers = &self.markers;
            worker_stop_once(&mut self.cancel_signal_issued, &cancel_key, || {
                if let Some(child) = running.get_mut(&command.command_id) {
                    child.request_group_termination().map_err(unavailable)?;
                } else {
                    markers
                        .stop_current_group(
                            &marker_key(command),
                            Duration::from_millis(100),
                            Duration::from_millis(10),
                        )
                        .map_err(unavailable)?;
                }
                Ok(())
            })?;
            self.reconcile(command)
        })();
        source_stop?;
        result
    }
    fn journal(
        &self,
        command: &WorkerCommandIdentity,
        after: u64,
        limit: u32,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        let receipt = self.store.command(command).map_err(refuse)?;
        let child = receipt
            .child_id
            .as_deref()
            .ok_or(WorkerRefusal::JournalUnavailable)?;
        if !safe_component(&command.instance_id) || !safe_component(child) {
            return Err(WorkerRefusal::JournalUnavailable);
        }
        let root = self
            .config
            .state_root
            .join("journals")
            .join(format!("{}-g{}", command.instance_id, command.generation))
            .join(child);
        let page = crate::attach::lineage_journal_page(&root, &command.task_id, after, limit)
            .map_err(|_| WorkerRefusal::JournalUnavailable)?;
        let fragments = page["events"]
            .as_array()
            .ok_or(WorkerRefusal::JournalUnavailable)?
            .iter()
            .map(|event| {
                Ok(WorkerJournalFragment {
                    seq: event["seq"]
                        .as_u64()
                        .ok_or(WorkerRefusal::JournalUnavailable)?,
                    offset: 0,
                    final_fragment: true,
                    bytes: serde_json::to_vec(event).map_err(unavailable)?,
                })
            })
            .collect::<Result<Vec<_>, WorkerRefusal>>()?;
        Ok(WorkerResponseOperation::JournalPage {
            fragments,
            through_seq: page["next_seq"].as_u64(),
            more: page["caught_up"].as_bool() != Some(true),
        })
    }
}

fn send_worker_frame(writer: &Mutex<UnixStream>, message: &WrapperToDaemon) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(message).map_err(|_| "worker_frame_invalid")?;
    bytes.push(b'\n');
    if bytes.len() > MAX_WORKER_FRAME_BYTES {
        return Err("worker_frame_too_large".into());
    }
    let mut writer = writer.lock().map_err(|_| "worker_writer_poisoned")?;
    writer
        .write_all(&bytes)
        .map_err(|_| "worker_connection_lost")?;
    writer.flush().map_err(|_| "worker_connection_lost".into())
}

// Metadata may be skipped while a mission response owns the writer. A partial
// frame closes the channel; it is never retried into a different JSON line.
fn send_catalogue_frame(
    writer: &Mutex<UnixStream>,
    message: &WrapperToDaemon,
) -> Result<bool, String> {
    use std::os::fd::AsRawFd;
    let Ok(writer) = writer.try_lock() else {
        return Ok(false);
    };
    let mut bytes = serde_json::to_vec(message).map_err(|_| "worker_frame_invalid")?;
    bytes.push(b'\n');
    if bytes.len() > MAX_WORKER_FRAME_BYTES {
        return Err("worker_frame_too_large".into());
    }
    // Darwin does not make every send-buffer wait nonblocking with MSG_DONTWAIT.
    // SO_SNDTIMEO bounds those waits without changing the reader's O_NONBLOCK.
    // Every writer uses this mutex, so its previous timeout can be restored.
    #[cfg(target_os = "macos")]
    let previous_timeout = writer
        .write_timeout()
        .map_err(|_| "worker_connection_lost")?;
    #[cfg(target_os = "macos")]
    writer
        .set_write_timeout(Some(Duration::from_millis(1)))
        .map_err(|_| "worker_connection_lost")?;
    let result = (|| {
        let deadline = Instant::now() + Duration::from_millis(250);
        let mut offset = 0;
        while offset < bytes.len() {
            if Instant::now() >= deadline {
                return Err("worker_connection_lost".into());
            }
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            let count = unsafe {
                libc::send(
                    writer.as_raw_fd(),
                    bytes[offset..].as_ptr().cast(),
                    bytes.len() - offset,
                    libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
                )
            };
            if count > 0 {
                offset += count as usize;
                continue;
            }
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::WouldBlock && offset == 0 {
                return Ok(false);
            }
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if error.kind() == std::io::ErrorKind::WouldBlock {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            return Err("worker_connection_lost".into());
        }
        Ok(true)
    })();
    #[cfg(target_os = "macos")]
    if writer.set_write_timeout(previous_timeout).is_err() {
        let _ = writer.shutdown(std::net::Shutdown::Both);
        return Err("worker_connection_lost".into());
    }
    if result.is_err() {
        let _ = writer.shutdown(std::net::Shutdown::Both);
    }
    result
}

/// Lectures d’origine bornées comme les attentes du maître, conservées au travers des reconnexions.
/// Elles ne bloquent ni Release ni la maintenance du worker.
fn dispatch_origin_proof(
    request: bridget_transport::protocol::OriginProofRequest,
    writer: Arc<Mutex<UnixStream>>,
    disconnected: Arc<AtomicBool>,
    slots: Arc<std::sync::atomic::AtomicUsize>,
    registry: PathBuf,
) {
    use bridget_transport::protocol::{OriginProofResponse, OriginProofResult};
    let limit = if matches!(
        &request.request,
        bridget_transport::protocol::OriginProofOperation::PrepareProject { .. }
    ) {
        crate::origin_proof::MAX_ORIGIN_PROJECT_PREPARATIONS
    } else {
        crate::origin_proof::MAX_ORIGIN_PROOFS
    };
    let busy = slots
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
            (n < limit).then_some(n + 1)
        })
        .is_err();
    let envelope = request.clone();
    let response = move |result| WrapperToDaemon::OriginProofResponse {
        response: OriginProofResponse {
            version: envelope.version,
            correlation_id: envelope.correlation_id.clone(),
            connection_epoch: envelope.connection_epoch.clone(),
            worker: envelope.worker.clone(),
            response: result,
        },
    };
    if busy {
        let operation = if matches!(
            &request.request,
            bridget_transport::protocol::OriginProofOperation::PrepareProject { .. }
        ) {
            "prepare_project"
        } else {
            "read"
        };
        eprintln!(
            "bridget_origin_proof {}",
            serde_json::json!({
            "refused":"capacity","operation":operation,"limit":limit,
            "correlation_id":request.correlation_id})
        );
        let _ = send_worker_frame(
            &writer,
            &response(OriginProofResult::Rejected {
                reason: WorkerRefusal::ResourceLimit,
            }),
        );
        return;
    }
    struct Permit(Arc<std::sync::atomic::AtomicUsize>);
    impl Drop for Permit {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::AcqRel);
        }
    }
    let permit = Permit(slots);
    std::thread::spawn(move || {
        let _permit = permit;
        if disconnected.load(Ordering::Acquire) {
            return;
        }
        let result = crate::origin_proof::execute(&request, &registry);
        if disconnected.load(Ordering::Acquire) {
            return;
        }
        let frame = response(result);
        if let WrapperToDaemon::OriginProofResponse { response } = &frame
            && response.validate_shape().is_err()
        {
            return;
        }
        let _ = send_worker_frame(&writer, &frame);
    });
}

/// Sonde historique sur la même connexion, avant toute variante ou secret162.
/// Le drapeau retourné est une attestation du maître pour proposer la nouvelle
/// enveloppe de capacités ; il ne dépend jamais du build du pair.

#[derive(Debug, Clone, Copy, Default)]
struct MasterWorkerFeatures {
    candidate_probe: bool,
    probe_v2: bool,
    permission_reasons: bool,
    workload_tools: bool,
    release_lease: bool,
    source_proof: bool,
    slot_deadline: bool,
    workspace_diagnostics: bool,
    origin_proof: bool,
    origin_project_context: bool,
    workspace_repair: bool,
    start_recovery: bool,
    workspace_failure: bool,
    provider_catalogue: bool,
    model_options: bool,
    provider_accounts: bool,
    workspace_view: bool,
    workspace_view_create: bool,
    execution_status: bool,
    program_compatibility: bool,
}
impl MasterWorkerFeatures {
    fn supports(&self, capability: WorkerCapability) -> bool {
        match capability {
            WorkerCapability::CandidateProbeV1 => self.candidate_probe,
            WorkerCapability::CandidateProbeV2 => self.probe_v2,
            WorkerCapability::CandidateProbePermissionReasonsV1 => {
                self.probe_v2 && self.permission_reasons
            }
            WorkerCapability::WorkloadToolsV1 => self.workload_tools && self.probe_v2,
            WorkerCapability::ReleaseLeaseV2 => self.release_lease,
            WorkerCapability::ParentSourceProofV1 => self.source_proof,
            WorkerCapability::ResultSlotDeadlineV1 => self.slot_deadline,
            WorkerCapability::OriginProofV1 => self.origin_proof,
            WorkerCapability::OriginProjectContextV1 => {
                self.origin_project_context && self.origin_proof
            }
            WorkerCapability::WorkspaceRepairV1 => self.workspace_repair,
            WorkerCapability::StartRecoveryV1 => self.start_recovery,
            WorkerCapability::WorkspaceFailureV1 => self.workspace_failure,
            WorkerCapability::ProviderCatalogueV1 => self.provider_catalogue,
            WorkerCapability::ModelOptionsV1 => self.model_options,
            WorkerCapability::ProviderAccountsV1 => self.provider_accounts,
            WorkerCapability::WorkspaceViewV1 => self.workspace_view,
            WorkerCapability::WorkspaceViewCreateV1 => self.workspace_view_create,
            WorkerCapability::ExecutionStatusV1 => self.execution_status,
            WorkerCapability::ProgramCompatibilityV1 => self.program_compatibility,
        }
    }
    fn capabilities(&self) -> Vec<WorkerCapability> {
        [
            WorkerCapability::CandidateProbeV1,
            WorkerCapability::CandidateProbeV2,
            WorkerCapability::CandidateProbePermissionReasonsV1,
            WorkerCapability::WorkloadToolsV1,
            WorkerCapability::ReleaseLeaseV2,
            WorkerCapability::ParentSourceProofV1,
            WorkerCapability::ResultSlotDeadlineV1,
            WorkerCapability::OriginProofV1,
            WorkerCapability::OriginProjectContextV1,
            WorkerCapability::WorkspaceRepairV1,
            WorkerCapability::StartRecoveryV1,
            WorkerCapability::WorkspaceFailureV1,
            WorkerCapability::ProviderCatalogueV1,
            WorkerCapability::ModelOptionsV1,
            WorkerCapability::ProviderAccountsV1,
            WorkerCapability::WorkspaceViewV1,
            WorkerCapability::WorkspaceViewCreateV1,
            WorkerCapability::ExecutionStatusV1,
            WorkerCapability::ProgramCompatibilityV1,
        ]
        .into_iter()
        .filter(|v| {
            self.supports(*v)
                && (*v != WorkerCapability::ModelOptionsV1 || self.provider_catalogue)
                && (*v != WorkerCapability::WorkspaceViewCreateV1
                    || crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED)
        })
        .collect()
    }
    fn admits(&self, accepted: &[WorkerCapability]) -> bool {
        accepted.len() <= 32
            && (!accepted.contains(&WorkerCapability::CandidateProbePermissionReasonsV1)
                || accepted.contains(&WorkerCapability::CandidateProbeV2))
            && (!accepted.contains(&WorkerCapability::WorkloadToolsV1)
                || accepted.contains(&WorkerCapability::CandidateProbeV2))
            && (!accepted.contains(&WorkerCapability::OriginProjectContextV1)
                || accepted.contains(&WorkerCapability::OriginProofV1))
            && (!accepted.contains(&WorkerCapability::ModelOptionsV1)
                || accepted.contains(&WorkerCapability::ProviderCatalogueV1))
            && (!accepted.contains(&WorkerCapability::WorkspaceViewCreateV1)
                || accepted.contains(&WorkerCapability::WorkspaceViewV1))
            && accepted.iter().enumerate().all(|(index, capability)| {
                self.capabilities().contains(capability) && !accepted[..index].contains(capability)
            })
    }
}

#[cfg(test)]
#[test]
fn spec241_tool_contract_requires_probe_v2_and_explicit_feature() {
    let old = MasterWorkerFeatures {
        probe_v2: true,
        ..Default::default()
    };
    let modern = MasterWorkerFeatures {
        probe_v2: true,
        workload_tools: true,
        ..Default::default()
    };
    assert!(!old.supports(WorkerCapability::WorkloadToolsV1));
    assert!(!old.admits(&modern.capabilities()));
    assert!(!modern.admits(&[WorkerCapability::WorkloadToolsV1]));
    assert!(modern.admits(&modern.capabilities()));
    assert!(
        !MasterWorkerFeatures {
            workload_tools: true,
            ..Default::default()
        }
        .supports(WorkerCapability::WorkloadToolsV1)
    );
}

#[test]
fn spec229_origin_context_is_negotiated_without_build_number() {
    let old = MasterWorkerFeatures {
        origin_proof: true,
        ..Default::default()
    };
    assert_eq!(old.capabilities(), vec![WorkerCapability::OriginProofV1]);
    let modern = MasterWorkerFeatures {
        origin_proof: true,
        origin_project_context: true,
        ..Default::default()
    };
    assert!(modern.admits(&modern.capabilities()));
    assert!(modern.admits(&old.capabilities()));
    assert!(!modern.admits(&[WorkerCapability::OriginProjectContextV1]));
    assert!(!old.admits(&modern.capabilities()));
}

fn probe_worker_capability(
    writer: &Mutex<UnixStream>,
    reader: &mut BufReader<UnixStream>,
) -> Result<MasterWorkerFeatures, String> {
    send_worker_frame(writer, &WrapperToDaemon::DaemonIdentityRequest)?;
    let line = bridget_transport::jsonl::read_unix_line(
        reader,
        MAX_WORKER_FRAME_BYTES,
        bridget_transport::jsonl::LineDeadline::Absolute(Instant::now() + Duration::from_secs(5)),
    )
    .map_err(|_| "worker_capability_unavailable")?
    .ok_or("worker_capability_unavailable")?;
    match serde_json::from_slice::<DaemonToWrapper>(&line) {
        Ok(DaemonToWrapper::DaemonIdentityReport { features, .. })
            if features.iter().any(|v| v == "remote_worker_v1") =>
        {
            let has = |name: &str| features.iter().any(|v| v == name);
            Ok(MasterWorkerFeatures {
                candidate_probe: has("worker_candidate_probe_v1"),
                probe_v2: has("worker_candidate_probe_v2"),
                permission_reasons: has("worker_candidate_probe_permission_reasons_v1"),
                workload_tools: has("worker_workload_tools_v1"),
                release_lease: has("worker_release_lease_v2"),
                source_proof: has("worker_parent_source_proof_v1"),
                slot_deadline: has("worker_result_slot_deadline_v1"),
                workspace_diagnostics: has("worker_workspace_diagnostics_v1"),
                origin_proof: has("worker_origin_proof_v1"),
                origin_project_context: has("worker_origin_project_context_v1"),
                workspace_repair: has("worker_workspace_repair_v1"),
                start_recovery: has("worker_start_recovery_v1"),
                workspace_failure: has("worker_workspace_failure_v1"),
                provider_catalogue: has("worker_provider_catalogue_v1"),
                model_options: has("worker_model_options_v1"),
                provider_accounts: has("worker_provider_accounts_v1"),
                workspace_view: has("worker_workspace_view_v1"),
                workspace_view_create: has("worker_workspace_view_create_v1"),
                execution_status: has("worker_execution_status_v1"),
                program_compatibility: has("worker_program_compatibility_v1"),
            })
        }
        _ => Err("worker_capability_required".into()),
    }
}

#[cfg(test)]
pub(crate) fn probe_capabilities198_for_test(
    writer: &Mutex<UnixStream>,
    reader: &mut BufReader<UnixStream>,
) -> Result<Vec<WorkerCapability>, String> {
    probe_worker_capability(writer, reader).map(|features| features.capabilities())
}

// O(1), projection fermée : aucun corps de trame, credential ou chemin.
fn unexpected_frame_diagnostic235(
    message: &DaemonToWrapper,
    worker: &WorkerIdentity,
    epoch: &str,
) -> serde_json::Value {
    let (kind, identity_matches, epoch_matches, shape_valid, refusal) = match message {
        DaemonToWrapper::WorkerRejected { reason } => {
            ("master_rejected", None, None, None, Some(*reason))
        }
        DaemonToWrapper::UpdateInProgress => ("update_in_progress", None, None, None, None),
        DaemonToWrapper::OriginProofRequest { request } => (
            "origin_proof_request",
            Some(request.worker == *worker),
            Some(request.connection_epoch == epoch),
            Some(request.validate_shape().is_ok()),
            None,
        ),
        DaemonToWrapper::ProviderCatalogueRefresh {
            worker: identity,
            connection_epoch,
            refresh_id,
        } => (
            "catalogue_refresh",
            Some(identity == worker),
            Some(connection_epoch == epoch),
            Some(bridget_transport::provider_catalogue::catalogue_token(
                refresh_id,
            )),
            None,
        ),
        DaemonToWrapper::CandidateProbeV2 { probe } => (
            "candidate_probe_v2",
            Some(probe.worker == *worker),
            Some(probe.connection_epoch == epoch),
            Some(probe.validate_shape().is_ok()),
            None,
        ),
        DaemonToWrapper::CandidateProbe { probe } => (
            "candidate_probe",
            Some(probe.worker == *worker),
            Some(probe.connection_epoch == epoch),
            Some(probe.validate_shape().is_ok()),
            None,
        ),
        DaemonToWrapper::WorkerRequest { request } => (
            "worker_request",
            Some(request.worker == *worker),
            Some(request.connection_epoch == epoch),
            Some(request.validate().is_ok()),
            None,
        ),
        _ => ("other", None, None, None, None),
    };
    serde_json::json!({"event":"worker_channel_reader_closed","reason":"unexpected_frame",
        "machine_id":worker.machine_id,"frame_kind":kind,"identity_matches":identity_matches,
        "epoch_matches":epoch_matches,"shape_valid":shape_valid,"refusal":refusal})
}

/// Client technique supervisé par l'unité existante. Une reconnexion ne détruit
/// ni processus relâché ni reçu ; seuls les permis en mémoire sont abandonnés.
pub fn run_worker(config: WorkerRuntimeConfig) -> Result<(), String> {
    let mut runtime = WorkerRuntime::new(config)?;
    let catalogues = crate::provider_catalogue::CatalogueService::start(
        runtime.config.registry_path.clone(),
        runtime.config.state_root.clone(),
        runtime.identity.machine_id.clone(),
        runtime.source.clone(),
    );
    let _catalogue_guard = crate::provider_catalogue::ServiceGuard(catalogues.clone());
    let origin_slots = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let origin_preparation_slots = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // Unique pour toutes les reconnexions ; aucune seconde lecture tant que
    // la précédente n'a pas effectivement terminé.
    let candidate_probe_slot = Arc::new(AtomicBool::new(false));
    loop {
        runtime
            .tick()
            .map_err(|reason| format!("worker_state_unavailable:{reason:?}"))?;
        let stream = match bridget_transport::jsonl::connect_nonblocking(
            &runtime.config.master_socket,
            Instant::now() + Duration::from_secs(5),
        ) {
            Ok(stream) => stream,
            Err(_) => {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|_| "worker_socket_configuration")?;
        let writer = Arc::new(Mutex::new(
            stream.try_clone().map_err(|_| "worker_socket_clone")?,
        ));
        let mut reader = BufReader::new(stream);
        let master_features = match probe_worker_capability(&writer, &mut reader) {
            Ok(capabilities) => capabilities,
            Err(code)
                if matches!(
                    code.as_str(),
                    "worker_connection_lost" | "worker_capability_unavailable"
                ) =>
            {
                // Le même runtime garde ses enfants relâchés pendant la panne du maître.
                // Aucun credential ni Hello n'est envoyé sans preuve de capacité.
                drop(reader);
                drop(writer);
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            Err(code) => return Err(code),
        };
        let master_supports_workspace_diagnostics = master_features.workspace_diagnostics;
        let offered_caps = master_features.capabilities();
        let worker_hello = WorkerHello {
            version: REMOTE_WORKER_VERSION,
            identity: runtime.identity.clone(),
            credential: runtime.credential.clone(),
            build: crate::build_info::BUILD_ID.to_owned(),
            capability: ClientCapability::RemoteWorkerV1,
        };
        let hello = if !offered_caps.is_empty() {
            WrapperToDaemon::WorkerHelloWithCapabilities {
                hello: WorkerHelloWithCapabilities {
                    hello: worker_hello,
                    capabilities: offered_caps.clone(),
                },
            }
        } else {
            // Le wire162 reste octet pour octet identique pour le maître ancien.
            WrapperToDaemon::WorkerHello {
                hello: worker_hello,
            }
        };
        if send_worker_frame(&writer, &hello).is_err() {
            std::thread::sleep(Duration::from_secs(1));
            continue;
        }
        let line = match bridget_transport::jsonl::read_unix_line(
            &mut reader,
            MAX_WORKER_FRAME_BYTES,
            bridget_transport::jsonl::LineDeadline::Absolute(
                Instant::now() + Duration::from_secs(5),
            ),
        ) {
            Ok(Some(line)) => line,
            _ => {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        let (epoch, negotiated) = match serde_json::from_slice::<DaemonToWrapper>(&line) {
            Ok(DaemonToWrapper::WorkerAccepted { accepted })
                if offered_caps.is_empty()
                    && accepted.validate().is_ok()
                    && accepted.identity == runtime.identity =>
            {
                (accepted.connection_epoch, Vec::new())
            }
            Ok(DaemonToWrapper::WorkerAcceptedWithCapabilities { accepted })
                if !offered_caps.is_empty()
                    && accepted.accepted.validate().is_ok()
                    && accepted.accepted.identity == runtime.identity
                    && master_features.admits(&accepted.capabilities) =>
            {
                (accepted.accepted.connection_epoch, accepted.capabilities)
            }
            Ok(DaemonToWrapper::WorkerRejected { .. }) => {
                return Err("worker_registration_refused".into());
            }
            _ => return Err("worker_handshake_invalid".into()),
        };
        runtime.provider_accounts_negotiated =
            negotiated.contains(&WorkerCapability::ProviderAccountsV1);
        runtime.permission_reasons_negotiated =
            negotiated.contains(&WorkerCapability::CandidateProbePermissionReasonsV1);
        runtime.program_compatibility_negotiated =
            negotiated.contains(&WorkerCapability::ProgramCompatibilityV1);
        if negotiated.contains(&WorkerCapability::ProviderCatalogueV1) {
            catalogues.activate(&epoch);
        } else {
            catalogues.deactivate();
        }
        let mut catalogue_sent = BTreeMap::<String, (String, u64)>::new();
        let candidate_probe_negotiated = negotiated.contains(&WorkerCapability::CandidateProbeV1);
        let candidate_probe_v2_negotiated =
            negotiated.contains(&WorkerCapability::CandidateProbeV2);
        let workload_tools_negotiated = negotiated.contains(&WorkerCapability::WorkloadToolsV1);
        let slot_deadline_negotiated = negotiated.contains(&WorkerCapability::ResultSlotDeadlineV1);
        let origin_proof_negotiated = negotiated.contains(&WorkerCapability::OriginProofV1);
        let origin_context_negotiated =
            negotiated.contains(&WorkerCapability::OriginProjectContextV1);
        let workspace_repair_negotiated = negotiated.contains(&WorkerCapability::WorkspaceRepairV1);
        let start_recovery_negotiated = negotiated.contains(&WorkerCapability::StartRecoveryV1);
        let workspace_failure_negotiated =
            negotiated.contains(&WorkerCapability::WorkspaceFailureV1);
        let catalogue_negotiated = negotiated.contains(&WorkerCapability::ProviderCatalogueV1);
        let workspace_view_negotiated = negotiated.contains(&WorkerCapability::WorkspaceViewV1);
        let execution_status_negotiated = negotiated.contains(&WorkerCapability::ExecutionStatusV1);
        let workspace_view_create_negotiated =
            negotiated.contains(&WorkerCapability::WorkspaceViewCreateV1);
        let read_catalogues = catalogues.clone();
        runtime.host_resources.reset();
        runtime.last_resource_refresh = Instant::now();
        let disconnected = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::sync_channel::<WorkerInbound>(8);
        // Sept trames et une tête datée : huit sondes en attente au maximum,
        // distinctes des huit commandes. Une seule lecture physique à la fois.
        let (probe_sender, probe_receiver) = mpsc::sync_channel::<(WorkerInbound, Instant)>(7);
        let mut probe_lane = ProbeReadLane {
            receiver: probe_receiver,
            pending: None,
            slot: candidate_probe_slot.clone(),
        };
        let read_disconnected = disconnected.clone();
        // Une coupure n'efface pas une annulation déjà reçue pour une lecture
        // encore possédée. Les autres identités reprennent leur contrôle durable.
        runtime.retain_view_cancellations()?;
        let cancelled = runtime.cancelled.clone();
        let read_epoch = epoch.clone();
        let read_worker = runtime.identity.clone();
        let writer_for_reader = writer.clone();
        let origin_slots = origin_slots.clone();
        let origin_preparation_slots = origin_preparation_slots.clone();
        let origin_registry = runtime.config.registry_path.clone();
        let reader_thread = std::thread::spawn(move || {
            while !read_disconnected.load(Ordering::Acquire) {
                let frame = bridget_transport::jsonl::read_unix_line(
                    &mut reader,
                    MAX_WORKER_FRAME_BYTES,
                    bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(5)),
                );
                let inbound = match frame {
                    Ok(Some(bytes)) => match serde_json::from_slice::<DaemonToWrapper>(&bytes) {
                        Ok(DaemonToWrapper::ProviderCatalogueRefresh {worker,connection_epoch,refresh_id})
                            if catalogue_negotiated && worker==read_worker && connection_epoch==read_epoch
                                && bridget_transport::provider_catalogue::catalogue_token(&refresh_id) => {
                            read_catalogues.activate(&refresh_id);continue;
                        }
                        Ok(DaemonToWrapper::OriginProofRequest { request })
                            if origin_proof_negotiated && request.connection_epoch==read_epoch
                                && request.worker==read_worker && request.validate_shape().is_ok()
                                && (origin_context_negotiated || !matches!(&request.request,
                                    bridget_transport::protocol::OriginProofOperation::PrepareProject {return_context:true,..})) => {
                            let permits=if matches!(&request.request,bridget_transport::protocol::OriginProofOperation::PrepareProject {..}) {
                                origin_preparation_slots.clone()
                            } else {origin_slots.clone()};
                            dispatch_origin_proof(request,writer_for_reader.clone(),read_disconnected.clone(),permits,origin_registry.clone());
                            continue;
                        }
                        Ok(DaemonToWrapper::WorkerRequest { request })
                            if request.connection_epoch == read_epoch
                                && request.worker == read_worker
                                && (request.validate().is_ok()
                                    || (matches!(&request.request, WorkerRequestOperation::StartDetailed { .. })
                                        && request.validate() == Err(WorkerRefusal::ResourceLimit)))
                                && (candidate_probe_v2_negotiated
                                    || !(matches!(&request.request,WorkerRequestOperation::PrepareV2 {..})
                                        || matches!(&request.request,WorkerRequestOperation::PrepareDiagnosed {prepare}
                                            if matches!(prepare.as_ref(),WorkerRequestOperation::PrepareV2 {..}))))
                                && (slot_deadline_negotiated || !matches!(
                                    &request.request, WorkerRequestOperation::ReserveResultSlotUntil { .. }
                                ))
                                && (execution_status_negotiated || !matches!(&request.request,WorkerRequestOperation::ExecutionStatus {}))
                                && (workspace_view_negotiated || !request.request.requires_commit_view_reader())
                                && (workspace_view_create_negotiated || !request.request.requires_commit_view_creation())
                                && (workspace_repair_negotiated || !matches!(
                                    &request.request, WorkerRequestOperation::RepairWorkspace { .. }
                                ))
                                && (workspace_failure_negotiated || !matches!(
                                    &request.request, WorkerRequestOperation::PrepareDiagnosed { .. }
                                ))
                                && (start_recovery_negotiated || !matches!(
                                    &request.request,
                                    WorkerRequestOperation::PrepareFenced { .. }
                                        | WorkerRequestOperation::StartDetailed { .. }
                                        | WorkerRequestOperation::FenceUnstarted { .. }
                                )) =>
                        {
                            WorkerInbound::Request(request)
                        }
                        Ok(DaemonToWrapper::CandidateProbe { probe })
                            if candidate_probe_negotiated
                                && probe.connection_epoch == read_epoch
                                && probe.worker == read_worker
                                && probe.validate_shape().is_ok() =>
                        {
                            WorkerInbound::CandidateProbe(probe)
                        }
                        Ok(DaemonToWrapper::CandidateProbeV2 { probe })
                            if candidate_probe_v2_negotiated
                                && (probe.required_tools.is_empty() || workload_tools_negotiated)
                                && probe.connection_epoch == read_epoch
                                && probe.worker == read_worker
                                && probe.validate_shape().is_ok() =>
                        {
                            WorkerInbound::CandidateProbeV2(probe)
                        }
                        Ok(message) => {
                            eprintln!("bridget_worker_channel235 {}",unexpected_frame_diagnostic235(&message,&read_worker,&read_epoch));
                            break;
                        }
                        Err(_) => {
                            eprintln!("bridget_worker_channel235 {}",serde_json::json!({"event":"worker_channel_reader_closed","reason":"frame_decode_invalid","machine_id":read_worker.machine_id}));
                            break;
                        }
                    },
                    Ok(None) => {
                        if !read_disconnected.load(Ordering::Acquire) {
                            log::warn!("worker_channel_reader_closed reason=eof");
                        }
                        break;
                    }
                    Err(_) => {
                        if !read_disconnected.load(Ordering::Acquire) {
                            log::warn!("worker_channel_reader_closed reason=frame_read_failed");
                        }
                        break;
                    }
                };
                let dispatched = if matches!(
                    &inbound,
                    WorkerInbound::CandidateProbe(_) | WorkerInbound::CandidateProbeV2(_)
                ) {
                    dispatch_worker_probe(
                        &probe_sender,
                        inbound,
                        &writer_for_reader,
                        &read_disconnected,
                    )
                } else {
                    dispatch_worker_inbound(
                        &sender,
                        inbound,
                        &writer_for_reader,
                        &read_disconnected,
                        &cancelled,
                    )
                };
                if dispatched.is_err() {
                    if !read_disconnected.load(Ordering::Acquire) {
                        log::warn!("worker_channel_reader_closed reason=inbound_dispatch_failed");
                    }
                    break;
                }
            }
            read_disconnected.store(true, Ordering::Release);
        });
        let heartbeat_writer = writer.clone();
        let heartbeat_disconnected = disconnected.clone();
        let heartbeat_thread = std::thread::spawn(move || {
            let mut last = Instant::now();
            while !heartbeat_disconnected.load(Ordering::Acquire) {
                if last.elapsed() >= Duration::from_secs(10) {
                    if send_worker_frame(&heartbeat_writer, &WrapperToDaemon::Heartbeat).is_err() {
                        heartbeat_disconnected.store(true, Ordering::Release);
                        break;
                    }
                    last = Instant::now();
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        let mut last_relay = Instant::now() - Duration::from_secs(2);
        let mut observed: BTreeMap<String, (WorkerCommandPhase, bool)> = BTreeMap::new();
        let mut fatal = None;
        while !disconnected.load(Ordering::Acquire) {
            if let Err(reason) = runtime.tick() {
                fatal = Some(format!("worker_state_unavailable:{reason:?}"));
                break;
            }
            if last_relay.elapsed() >= Duration::from_secs(2) {
                match runtime.store.pending_results(worker_now()) {
                    Ok(results) => {
                        for result in results {
                            let response = WorkerResponse {
                                version: REMOTE_WORKER_VERSION,
                                correlation_id: result.message_id.clone(),
                                connection_epoch: epoch.clone(),
                                worker: runtime.identity.clone(),
                                command: result.identity.command.clone(),
                                response: WorkerResponseOperation::FinalResult { result },
                            };
                            if send_worker_frame(
                                &writer,
                                &WrapperToDaemon::WorkerResponse { response },
                            )
                            .is_err()
                            {
                                disconnected.store(true, Ordering::Release);
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        fatal = Some(error.to_string());
                        break;
                    }
                }
                match runtime.store.commands() {
                    Ok(commands) => {
                        // Même borne que le snapshot du store, y compris après collecte.
                        let current = commands
                            .iter()
                            .map(|receipt| receipt.identity.command_id.as_str())
                            .collect::<BTreeSet<_>>();
                        observed.retain(|command, _| current.contains(command.as_str()));
                        for receipt in commands {
                            if observed.get(&receipt.identity.command_id)
                                == Some(&(receipt.phase, receipt.native_ready))
                            {
                                continue;
                            }
                            let response = WorkerResponse {
                                version: REMOTE_WORKER_VERSION,
                                correlation_id: uuid::Uuid::new_v4().to_string(),
                                connection_epoch: epoch.clone(),
                                worker: runtime.identity.clone(),
                                command: receipt.identity.clone(),
                                response: WorkerRuntime::observation_native(
                                    receipt.phase,
                                    receipt.native_ready,
                                    None,
                                ),
                            };
                            if send_worker_frame(
                                &writer,
                                &WrapperToDaemon::WorkerResponse { response },
                            )
                            .is_err()
                            {
                                disconnected.store(true, Ordering::Release);
                                break;
                            }
                            observed.insert(
                                receipt.identity.command_id,
                                (receipt.phase, receipt.native_ready),
                            );
                        }
                    }
                    Err(error) => {
                        fatal = Some(error.to_string());
                        break;
                    }
                }
                last_relay = Instant::now();
            }
            if negotiated.contains(&WorkerCapability::ProviderCatalogueV1) {
                for (refresh_id, sequence, catalogue) in catalogues.snapshots() {
                    if catalogue_sent.get(&catalogue.connection_id)
                        == Some(&(refresh_id.clone(), sequence))
                    {
                        continue;
                    }
                    let catalogue = if negotiated.contains(&WorkerCapability::ModelOptionsV1) {
                        catalogue
                    } else {
                        catalogue.legacy_projection()
                    };
                    let connection = catalogue.connection_id.clone();
                    let update = bridget_transport::provider_catalogue::WorkerCatalogueUpdate {
                        version: 1,
                        worker: runtime.identity.clone(),
                        connection_epoch: epoch.clone(),
                        refresh_id: refresh_id.clone(),
                        sequence,
                        catalogue,
                    };
                    match send_catalogue_frame(
                        &writer,
                        &WrapperToDaemon::ProviderCatalogueUpdate { update },
                    ) {
                        Ok(true) => {
                            catalogue_sent.insert(connection, (refresh_id, sequence));
                        }
                        Ok(false) => {}
                        Err(_) => {
                            disconnected.store(true, Ordering::Release);
                            break;
                        }
                    }
                }
            }
            if workspace_view_create_negotiated {
                for response in runtime.poll_view_preparations(&epoch) {
                    if response.validate().is_err() {
                        fatal = Some("worker_view_response_invalid".into());
                        disconnected.store(true, Ordering::Release);
                        break;
                    }
                    if send_worker_frame(&writer, &WrapperToDaemon::WorkerResponse { response })
                        .is_err()
                    {
                        disconnected.store(true, Ordering::Release);
                        break;
                    }
                }
            }
            if workspace_view_negotiated {
                for response in runtime.poll_view_files(&epoch) {
                    if response.validate().is_err() {
                        fatal = Some("worker_view_file_response_invalid".into());
                        disconnected.store(true, Ordering::Release);
                        break;
                    }
                    if send_worker_frame(&writer, &WrapperToDaemon::WorkerResponse { response })
                        .is_err()
                    {
                        disconnected.store(true, Ordering::Release);
                        break;
                    }
                }
                for response in runtime.poll_view_controls(&epoch, &disconnected) {
                    if response.validate().is_err() {
                        fatal = Some("worker_view_control_response_invalid".into());
                        disconnected.store(true, Ordering::Release);
                        break;
                    }
                    if send_worker_frame(&writer, &WrapperToDaemon::WorkerResponse { response })
                        .is_err()
                    {
                        disconnected.store(true, Ordering::Release);
                        break;
                    }
                }
            }
            if probe_lane
                .drive(
                    &runtime,
                    &writer,
                    &disconnected,
                    master_supports_workspace_diagnostics,
                )
                .is_err()
            {
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(WorkerInbound::Request(request)) => {
                    let file = if workspace_view_negotiated && !disconnected.load(Ordering::Acquire)
                    {
                        runtime.defer_view_file(&request)
                    } else {
                        Ok(false)
                    };
                    let file_error = match file {
                        Ok(true) => continue,
                        Ok(false) => None,
                        Err(error) => Some(error),
                    };
                    let control = if file_error.is_some() {
                        Ok(false)
                    } else if workspace_view_negotiated && !disconnected.load(Ordering::Acquire) {
                        runtime.defer_view_control(&request)
                    } else {
                        Ok(false)
                    };
                    let control_error = match control {
                        Ok(true) => continue,
                        Ok(false) => None,
                        Err(reason) => Some(reason),
                    };
                    let deferred = if control_error.is_some() {
                        Ok(false)
                    } else if workspace_view_create_negotiated
                        && !disconnected.load(Ordering::Acquire)
                    {
                        runtime.defer_view_preparation(&request)
                    } else {
                        Ok(false)
                    };
                    let response = if let Some(error) = file_error {
                        WorkerRuntime::view_file_response(&request, Err(error))
                    } else {
                        match deferred {
                            Ok(true) => continue,
                            Err(error) => WorkerRuntime::view_prepare_rejection(error),
                            Ok(false) => {
                                if let Some(reason) = control_error {
                                    WorkerRuntime::control_refusal(&request.request, reason)
                                } else {
                                    match runtime.handle(&request, &epoch, &disconnected) {
                                        Ok(response) => response,
                                        Err(reason) => WorkerResponseOperation::Rejected { reason },
                                    }
                                }
                            }
                        }
                    };
                    let response = WorkerResponse {
                        version: REMOTE_WORKER_VERSION,
                        correlation_id: request.correlation_id,
                        connection_epoch: epoch.clone(),
                        worker: runtime.identity.clone(),
                        command: request.command,
                        response,
                    };
                    if response.validate().is_err() {
                        fatal = Some("worker_response_invalid".into());
                        break;
                    }
                    if send_worker_frame(&writer, &WrapperToDaemon::WorkerResponse { response })
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(WorkerInbound::CandidateProbe(_) | WorkerInbound::CandidateProbeV2(_)) => {
                    fatal = Some("worker_probe_lane_mismatch".into());
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
        disconnected.store(true, Ordering::Release);
        if let Ok(socket) = writer.lock() {
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
        catalogues.deactivate();
        let _ = reader_thread.join();
        let _ = heartbeat_thread.join();
        runtime.abandon_held();
        if let Some(error) = fatal {
            return Err(error);
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(test)]
#[path = "remote_worker_tests.rs"]
mod tests;

#[cfg(test)]
mod catalogue220_tests {
    use super::*;
    #[test]
    fn spec220_metadata_writer_never_waits_on_a_mission_lock_and_times_out_full_socket() {
        use std::os::fd::AsRawFd;
        let (a, _b) = UnixStream::pair().unwrap();
        let writer = Mutex::new(a);
        let guard = writer.lock().unwrap();
        let start = Instant::now();
        assert!(!send_catalogue_frame(&writer, &WrapperToDaemon::DaemonIdentityRequest).unwrap());
        assert!(start.elapsed() < Duration::from_millis(100));
        drop(guard);
        writer.lock().unwrap().set_nonblocking(true).unwrap();
        let bytes = vec![0; 65536];
        for _ in 0..1024 {
            // SAFETY: le tampon `bytes` est valide pour sa longueur pendant l'appel send.
            let n = unsafe {
                libc::send(
                    writer.lock().unwrap().as_raw_fd(),
                    bytes.as_ptr().cast(),
                    bytes.len(),
                    libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
                )
            };
            if n < 0 {
                break;
            }
        }
        writer.lock().unwrap().set_nonblocking(false).unwrap();
        let original_timeout = Some(Duration::from_secs(3));
        writer
            .lock()
            .unwrap()
            .set_write_timeout(original_timeout)
            .unwrap();
        let start = Instant::now();
        assert!(!send_catalogue_frame(&writer, &WrapperToDaemon::DaemonIdentityRequest).unwrap());
        assert!(start.elapsed() < Duration::from_millis(100));
        assert_eq!(
            writer.lock().unwrap().write_timeout().unwrap(),
            original_timeout
        );
        let (a, _peer) = UnixStream::pair().unwrap();
        let size: libc::c_int = 4096;
        assert_eq!(
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                libc::setsockopt(
                    a.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_SNDBUF,
                    (&size as *const libc::c_int).cast(),
                    std::mem::size_of_val(&size) as libc::socklen_t,
                )
            },
            0
        );
        let writer = Mutex::new(a);
        let large = WrapperToDaemon::NativeDelegation {
            request: bridget_transport::protocol::NativeDelegationRequest::ProviderCatalogueV1 {
                refresh: false,
                machine_id: Some("x".repeat(128 * 1024)),
            },
        };
        let reader = writer.lock().unwrap().try_clone().unwrap();
        assert_eq!(
            // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
            unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_GETFL) } & libc::O_NONBLOCK,
            0
        );
        let start = Instant::now();
        assert!(send_catalogue_frame(&writer, &large).is_err());
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(writer.lock().unwrap().write_timeout().unwrap(), None);
        assert_eq!(
            // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
            unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_GETFL) } & libc::O_NONBLOCK,
            0
        );
        let (a, mut peer) = UnixStream::pair().unwrap();
        let writer = Mutex::new(a);
        assert!(send_catalogue_frame(&writer, &WrapperToDaemon::DaemonIdentityRequest).unwrap());
        use std::io::BufRead;
        let mut line = String::new();
        std::io::BufReader::new(&mut peer)
            .read_line(&mut line)
            .unwrap();
        assert!(matches!(
            serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),
            WrapperToDaemon::DaemonIdentityRequest
        ));
        assert_eq!(writer.lock().unwrap().write_timeout().unwrap(), None);
    }
}

#[cfg(test)]
mod program_compatibility221_tests {
    use super::*;
    #[test]
    fn spec221_program_compat_reasons_are_downgraded_for_older_master() {
        for reason in [
            CandidateProbeRefusal::ProgramUpgradeCheckPending,
            CandidateProbeRefusal::ProgramProtocolIncompatible,
            CandidateProbeRefusal::ProgramCheckUnavailable,
        ] {
            assert_eq!(
                program_probe_reason_for_master(reason, false),
                CandidateProbeRefusal::DefinitionUnavailable
            );
            assert_eq!(program_probe_reason_for_master(reason, true), reason);
        }
        let old = MasterWorkerFeatures::default();
        assert!(
            !old.capabilities()
                .contains(&WorkerCapability::ProgramCompatibilityV1)
        );
        assert!(!old.admits(&[WorkerCapability::ProgramCompatibilityV1]));
        let current = MasterWorkerFeatures {
            program_compatibility: true,
            ..Default::default()
        };
        assert!(current.admits(&[WorkerCapability::ProgramCompatibilityV1]));
    }
}

#[cfg(test)]
#[test]
fn spec241_tool_probe_distinguishes_presence_usability_and_timeout() {
    use bridget_transport::protocol::{WorkloadTool as T, WorkloadToolState as S};
    let root = std::env::temp_dir().join(format!("b241-tools-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let mut env = crate::lifecycle::SourceEnvironment::new();
    env.insert("PATH".into(), root.clone().into_os_string());
    let probe = |deadline| qualify_workload_tools(&env, &[T::Docker], &[], deadline)[0].state;
    assert_eq!(probe(Instant::now() + Duration::from_secs(1)), S::Absent);
    let script = root.join("docker");
    fs::write(&script, "#!/bin/sh\nexit 23\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(probe(Instant::now() + Duration::from_secs(1)), S::Unusable);
    fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
    assert_eq!(probe(Instant::now() + Duration::from_secs(1)), S::Available);
    assert_eq!(
        qualify_workload_tools(
            &env,
            &[T::Docker],
            &[root.canonicalize().unwrap()],
            Instant::now() + Duration::from_secs(1)
        )[0]
        .state,
        S::Absent
    );
    fs::write(&script,"#!/bin/sh\n[ \"$RUSTUP_AUTO_INSTALL\" = 0 ] && [ \"$COREPACK_ENABLE_NETWORK\" = 0 ] && [ \"$COREPACK_ENV_FILE\" = 0 ]\n").unwrap();
    assert_eq!(probe(Instant::now() + Duration::from_secs(1)), S::Available);
    fs::write(&script, "#!/bin/sh\nwhile :; do :; done\n").unwrap();
    let start = Instant::now();
    assert_eq!(probe(start + Duration::from_millis(30)), S::Unknown);
    assert!(start.elapsed() < Duration::from_millis(400));
    fs::remove_dir_all(root).unwrap();
}
