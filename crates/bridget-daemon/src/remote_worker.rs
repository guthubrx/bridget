//! État technique du worker 162. Aucune tâche métier ni réservation Fleet.
//!
//! Un seul worker écrit les reçus. Chaque wrapper publie un seul blob dans son
//! slot réservé ; il n'ouvre jamais le verrou du store ou ses métadonnées.

use crate::managed_process::ManagedMarker;
use bridget_transport::fsutil::{create_private_dir, write_private_file_atomic};
use bridget_transport::protocol::{
    MAX_WORKER_RESULT_ENVELOPE_BYTES, MAX_WORKER_STORE_BYTES, MAX_WORKER_STORE_ENTRIES, HostResourceSnapshotV1,
    WORKER_RELEASE_LEASE_SECS,
    RemotePlacement, RemoteResultEnvelope, WORKER_RECEIPT_RETENTION_SECS, WorkerCommandIdentity,
    WorkerCommandPhase, WorkerIdentity, WorkerRefusal, WorkerReleasePermit,
    WorkerStartCapacityCause,
    WorkerResultDisposition, WorkerResultIdentity, WorkerResultSlot, WorkerSlotClosureProof,
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

#[path = "remote_worker_proof198.rs"]
mod proof198;
#[path = "remote_worker_read.rs"]
mod read;
pub(crate) use read::{diagnose_live_local, diagnostic_failure, live_worker_binding, LiveWorkerBinding};
#[cfg(test)]
use read::diagnose_local;

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
enum ProbePermissionSite { Projection, ScopedConstructor, Refresh, Recheck, BatchFinish }
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
}
impl<'a> ProbePermissionDiagnostics<'a> {
    fn new(correlation_id: &'a str, admission_id: &'a str) -> Self {
        Self { correlation_id, admission_id, emitted: std::cell::Cell::new(0) }
    }
    fn note(&self, candidate_id: &str, site: ProbePermissionSite, error: &str) -> Option<String> {
        self.note_with_witness(candidate_id, site, error, None)
    }
    fn note_with_witness(&self, candidate_id: &str, site: ProbePermissionSite, error: &str,
        detail: Option<(Option<crate::native_permissions::ProjectionGuard>,
            &crate::native_permissions::ProjectionWitnessSummary171)>,
    ) -> Option<String> {
        if self.emitted.get() >= MAX_PROBE_PERMISSION_NOTES
            || self.correlation_id.len() != 36
            || uuid::Uuid::parse_str(self.correlation_id).is_err()
            || self.admission_id.len() != 34
            || !self.admission_id.starts_with("a-")
            || !self.admission_id[2..].bytes().all(|b| b.is_ascii_hexdigit())
            || !bridget_transport::protocol::is_routing_candidate_id(candidate_id)
        { return None; }
        // Exact closed codes only, never an error message or a prefix match.
        let code = permission_diagnostic_code(error);
        let mut value = serde_json::json!({"event":"worker_probe_permission_refused",
            "correlation_id":self.correlation_id,"admission_id":self.admission_id,
            "candidate_id":candidate_id,"site":site.as_str(),"code":code});
        if let Some((guard, summary)) = detail {
            value["reason"] = serde_json::json!(guard.map(|g| g.as_str()).unwrap_or("unknown"));
            value["witness"] = summary.json();
        }
        let note = value.to_string();
        if note.len() > MAX_PROBE_PERMISSION_NOTE_BYTES { return None; }
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
    if let Some((batch, candidate_id)) = diagnostics {
        if let Some(note) = batch.note(candidate_id, site, error) { emit(&note); }
    }
    WorkerRefusal::PermissionsChanged
}
fn observe_probe_permission_failure(
    error: &str, diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>, site: ProbePermissionSite,
) -> WorkerRefusal {
    map_probe_permission_failure(error, diagnostics, site,
        |note| eprintln!("bridget_probe_permission_refusal {note}"))
}

// The internal reason augments the existing note; the external refusal is unchanged.
fn map_probe_projection_failure171(
    error: &str, diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    guard: Option<crate::native_permissions::ProjectionGuard>, parent_policy: &serde_json::Value,
    emit: impl FnOnce(&str),
) -> WorkerRefusal {
    if error != "permission_hook_equivalence_unavailable" {
        return map_probe_permission_failure(error, diagnostics, ProbePermissionSite::Projection, emit);
    }
    if let Some((batch, candidate_id)) = diagnostics {
        let summary = crate::native_permissions::projection_witness_summary171(parent_policy);
        if let Some(note) = batch.note_with_witness(candidate_id, ProbePermissionSite::Projection,
            error, Some((guard, &summary))) { emit(&note); }
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
    fn effective_placement(&self)->&RemotePlacement {
        self.workspace_repair.as_ref().filter(|v|v.phase=="published")
            .and_then(|v|v.prepared.as_ref()).map(|v|&v.placement).unwrap_or(&self.placement)
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

fn fenced_prepare_digest(command: &WorkerCommandIdentity, request: &WorkerPrepare)
    -> Result<String, WorkerStoreError> {
    json_digest(&(command, fenced_prepare_canon(request)))
}

pub(crate) fn same_workspace_anchor(record: &crate::delegation::WorkspaceRecord,
    workspace: &serde_json::Value) -> bool {
    let Ok(sent) = serde_json::from_value::<crate::delegation::WorkspaceRecord>(workspace.clone())
        else { return false; };
    matches!((&record.strategy, &sent.strategy),
        (crate::delegation::WorkspaceStrategy::Worktree,
         crate::delegation::WorkspaceStrategy::Worktree)) && record.path == sent.path
        && record.toplevel == sent.toplevel && record.branch == sent.branch
        && record.base_commit == sent.base_commit
}

fn exact_fence_at(root: &Path, worker: &WorkerIdentity,
    previous: &WorkerCommandIdentity, placement: &RemotePlacement) -> Result<(), WorkerStoreError> {
    let path = root.join("commands").join(format!("{}.json", marker_key(previous)));
    let mut receipt: WorkerCommandReceipt = read_optional(&path, MAX_METADATA_BYTES)?
        .ok_or(WorkerStoreError::Unknown)?;
    proof198::overlay_fence(root, worker, &mut receipt)?;
    let digest = proof198::fence_digest(worker, previous, &receipt.placement, receipt.deadline_at)?;
    if receipt.version != STORE_VERSION || receipt.worker != *worker
        || receipt.identity != *previous || receipt.placement != *placement
        || receipt.deadline_at <= 0 || receipt.created_at < 0
        || receipt.pre_start_fence.as_deref() != Some(digest.as_str())
        || receipt.command_digest != digest || receipt.phase != WorkerCommandPhase::Refused
        || receipt.marker.is_some() || receipt.permit.is_some() || receipt.child_id.is_some()
        || receipt.preparation.is_some() || receipt.routing.is_some()
        || receipt.release_epoch.is_some() || receipt.native_ready || receipt.startup_failed
        || receipt.startup_failure.is_some() || receipt.terminal_exit_code.is_some()
        || receipt.workspace_repair.is_some() || !receipt.workspace_repair_history.is_empty()
    {
        return Err(WorkerStoreError::Conflict);
    }
    Ok(())
}

/// Les reçus de reprise ne contiennent que le lien vers le fence précédent.
/// Une chaîne de reprises se termine toujours sur la création Git originale.
fn original_workspace_at(root: &Path, worker: &WorkerIdentity,
    first: &WorkerCommandIdentity)
    -> Result<crate::delegation::WorkspaceRecord, WorkerStoreError> {
    let mut command = first.clone();
    let mut visited = BTreeSet::new();
    for _ in 0..MAX_WORKER_STORE_ENTRIES {
        if !visited.insert(command.command_id.clone()) { return Err(WorkerStoreError::Conflict); }
        let receipt = proof198::read_workspace_receipt(root, worker, &command)?
            .ok_or(WorkerStoreError::Unknown)?;
        if receipt.version != STORE_VERSION || receipt.worker != *worker
            || receipt.command != command || !valid_digest(&receipt.prepare_digest)
        {
            return Err(WorkerStoreError::Conflict);
        }
        match (receipt.record, receipt.fenced_previous) {
            (Some(record), None) => return Ok(record),
            (None, Some(previous)) => {
                if previous.validate().is_err() || previous.task_id != command.task_id
                    || previous.command_id == command.command_id {
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
        if proof198::read_fence(&self.root, &self.worker, identity)?.is_some() {
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
        let _guard = self.transition.lock().map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        identity.validate()?;
        placement.validate()?;
        if !placement.matches_worker(&self.worker) || deadline_at <= 0 || now < 0 {
            return Err(WorkerRefusal::InvalidEnvelope.into());
        }
        let digest = proof198::fence_digest(&self.worker, identity, placement, deadline_at)?;
        // La preuve est écrite avant le reçu ; seule elle reste après un crash.
        let existing = proof198::read_fence(&self.root, &self.worker, identity)?;
        if existing.as_ref().is_some_and(|proof| proof.digest != digest
            || proof.placement != *placement || proof.deadline_at != deadline_at) {
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
        match fs::symlink_metadata(self.root.join("markers").join(format!("{}.json", marker_key(identity)))) {
            Ok(_) => return Err(WorkerStoreError::Unknown),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {},
            Err(error) => return Err(error.into()),
        }
        let proof = match existing {
            Some(proof) => {
                self.admit(MAX_METADATA_BYTES as u64 * 2 + DRAIN_PROOF_RESERVATION, 1)?;
                proof
            }
            None => {
                self.admit(MAX_METADATA_BYTES as u64 * 3 + DRAIN_PROOF_RESERVATION, 2)?;
                let proof = proof198::FenceProof {
                    version: STORE_VERSION,
                    worker: self.worker.clone(),
                    identity: identity.clone(),
                    placement: placement.clone(),
                    deadline_at,
                    created_at: now,
                    digest: digest.clone(),
                };
                proof198::ensure_dir(&self.root)?;
                self.persist(&proof198::fence_path(&self.root, identity), &proof)?;
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
            deadline_at: proof198::legacy_deadline(proof.deadline_at, proof.created_at),
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
        let _guard = self.transition.lock().map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let mut receipt = self.read_command(identity)?.ok_or(WorkerStoreError::Unknown)?;
        let before = (receipt.startup_failure, receipt.terminal_exit_code);
        if receipt.startup_failure.is_none() { receipt.startup_failure = failure; }
        if exit_code.is_some() { receipt.terminal_exit_code = exit_code; }
        if before != (receipt.startup_failure, receipt.terminal_exit_code) {
            self.persist(&self.command_path(identity), &receipt)?;
            eprintln!("bridget_remote_start171 {}", serde_json::json!({
                "task_id":identity.task_id,"command_id":identity.command_id,
                "generation":identity.generation,"failure":receipt.startup_failure,
                "exit_code":receipt.terminal_exit_code
            }));
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
        ) || !matches!(
            phase,
            WorkerCommandPhase::Terminal
                | WorkerCommandPhase::Refused
                | WorkerCommandPhase::Unknown
        ) && !(receipt.phase == WorkerCommandPhase::ReleaseAuthorized
            && phase == WorkerCommandPhase::Released)
        {
            return Err(WorkerStoreError::InvalidTransition);
        }
        receipt.phase = phase;
        receipt.updated_at = now;
        self.persist(&self.command_path(identity), &receipt)
    }

    pub fn reserve_result_slot(
        &self, slot: &WorkerResultSlot, now: i64,
    ) -> Result<WorkerResultReservation, WorkerStoreError> {
        self.reserve_result_slot_before(slot, now, None)
    }

    pub fn reserve_result_slot_before(
        &self, slot: &WorkerResultSlot, now: i64, expires_at_ms: Option<i64>,
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
            ) && now.saturating_sub(command.deadline_at.max(command.updated_at))
                >= WORKER_RECEIPT_RETENTION_SECS
                && !referenced.contains(&command_key(&command.identity))
            {
                let proof = drain_proof_path(&self.root, &command.identity);
                match fs::remove_file(&proof) {
                    Ok(()) => File::open(self.root.join("markers"))?.sync_all()?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                workspace_repair::collect_markers(&self.root,&command)?;
                // Preuve annexe d'abord : un crash laisse au pire un refus ordinaire.
                if command.pre_start_fence.is_some() {
                    fs::remove_file(proof198::fence_path(&self.root, &command.identity))?;
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
        identity.validate()?;
        let mut result: Option<WorkerCommandReceipt> =
            read_optional(&self.command_path(identity), MAX_METADATA_BYTES)?;
        if let Some(receipt) = result.as_mut() {
            if receipt.worker == self.worker && receipt.identity == *identity {
                proof198::overlay_fence(&self.root, &self.worker, receipt)?;
            }
        }
        if let Some(receipt) = &result {
            if receipt.worker != self.worker
                || receipt.version != STORE_VERSION
                || receipt.identity != *identity
            {
                return Err(WorkerStoreError::Conflict);
            }
            receipt.placement.validate()?;
            let expected_fence = receipt.pre_start_fence.as_ref().map(|_| {
                proof198::fence_digest(&self.worker, identity,
                    &receipt.placement, receipt.deadline_at)
            }).transpose()?;
            if !receipt.placement.matches_worker(&self.worker)
                || !valid_digest(&receipt.command_digest)
                || receipt.created_at < 0
                || (receipt.pre_start_fence.is_none() && receipt.deadline_at <= receipt.created_at)
                || receipt.pre_start_fence.as_ref().is_some_and(|digest|
                    !valid_digest(digest) || Some(digest) != expected_fence.as_ref()
                    || *digest != receipt.command_digest
                    || receipt.phase != WorkerCommandPhase::Refused
                    || receipt.marker.is_some() || receipt.permit.is_some()
                    || receipt.child_id.is_some() || receipt.preparation.is_some()
                    || receipt.routing.is_some() || receipt.release_epoch.is_some()
                    || receipt.native_ready || receipt.startup_failed
                    || receipt.startup_failure.is_some() || receipt.terminal_exit_code.is_some())
            {
                return Err(WorkerStoreError::Quarantined);
            }
        }
        Ok(result)
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
            let command = self.read_command(&command.identity)?
                .ok_or(WorkerStoreError::Quarantined)?;
            // Terminal/Refused ne peut plus recevoir de préparation ni de permis.
            // Les 4 Kio gardent la preuve de drainage et les détails de sortie tardifs.
            used = used.saturating_add(DRAIN_PROOF_RESERVATION);
            if !matches!(command.phase, WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused) {
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
            if !final_disposition && !(slot.closed.is_some() && slot.producer_terminal) {
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
                proof198::overlay_bind(&self.root, &self.worker, &mut receipt)?;
                if receipt.version != STORE_VERSION
                    || receipt.worker != self.worker
                    || !valid_digest(&receipt.prepare_digest)
                    || path != self.workspace_path(&receipt.command)
                {
                    return Err(WorkerStoreError::Quarantined);
                }
                workspaces += 1;
                // Une préparation achevée est immutable ; son rejeu ne fait que la lire.
                if receipt.record.is_none() {
                    used = used.saturating_add(MAX_METADATA_BYTES as u64);
                }
            }
        }
        let count = commands.len() + slots.len() + workspaces
            + proof198::count(&self.root, &self.worker)?;
        if used > MAX_WORKER_STORE_BYTES as u64 || count > MAX_WORKER_STORE_ENTRIES {
            return Err(WorkerStoreError::Quota);
        }
        Ok((commands, slots, used, count))
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
        let fence = self.read_command(previous)?.ok_or(WorkerStoreError::Unknown)?;
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
        let _guard = self.transition.lock().map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        self.fenced_workspace_locked(previous, placement, workspace)
    }

    fn fenced_placement(&self, previous: &WorkerCommandIdentity,
        placement: &RemotePlacement) -> Result<(), WorkerStoreError> {
        let _guard = self.transition.lock().map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        let receipt = self.read_command(previous)?.ok_or(WorkerStoreError::Unknown)?;
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
        let _guard = self.transition.lock().map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
        command.validate_preparation()?;
        if previous.task_id != command.task_id || previous.command_id == command.command_id {
            return Err(WorkerStoreError::Conflict);
        }
        self.fenced_workspace_locked(previous, placement, workspace)?;
        let digest = fenced_prepare_digest(command, request)?;
        let path = self.workspace_path(command);
        let proof = proof198::BindProof {
            version: STORE_VERSION, worker: self.worker.clone(), command: command.clone(),
            prepare_digest: digest.clone(), previous: previous.clone(),
        };
        let stored = proof198::read_bind(&self.root, &self.worker, command)?;
        if stored.as_ref().is_some_and(|stored| *stored != proof) {
            return Err(WorkerStoreError::Conflict);
        }
        if let Some(existing) = proof198::read_workspace_receipt(&self.root, &self.worker, command)? {
            return if existing.version == STORE_VERSION && existing.worker == self.worker
                && existing.command == *command && existing.prepare_digest == digest
                && existing.fenced_previous.as_ref() == Some(previous)
                && existing.record.is_none() {
                Ok(())
            } else { Err(WorkerStoreError::Conflict) };
        }
        if stored.is_none() {
            self.admit(3 * MAX_METADATA_BYTES as u64, 2)?;
            proof198::ensure_dir(&self.root)?;
            self.persist(&proof198::bind_path(&self.root, command), &proof)?;
        } else {
            self.admit(2 * MAX_METADATA_BYTES as u64, 1)?;
        }
        self.persist(&path, &WorkspacePreparationReceipt {
            version: STORE_VERSION, worker: self.worker.clone(), command: command.clone(),
            prepare_digest: digest, fenced_previous: None, record: None,
        })
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
        if let Some(receipt) =
            proof198::read_workspace_receipt(&self.root, &self.worker, command)?
        {
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
            crate::workspace_git::report(&record).map_err(workspace_failure)?;
            return Ok(record);
        }
        let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        else {
            return Err(WorkerRefusal::InvalidEnvelope.into());
        };
        if let Some(commit) = request.base_commit.as_deref() {
            crate::workspace_git::import_remote_commit(root, commit)
                .map_err(workspace_failure)?;
        }
        let plan = crate::workspace_git::preflight_remote(root, base_ref.as_deref(), branch.as_deref())
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
        };
        {
            let _guard = self
                .transition
                .lock()
                .map_err(|_| WorkerStoreError::Quarantined)?;
            // Un lien198 sans reçu interdit de créer un autre worktree.
            if proof198::read_bind(&self.root, &self.worker, command)?.is_some() {
                return Err(WorkerStoreError::Conflict);
            }
            self.admit(2 * MAX_METADATA_BYTES as u64, 1)?;
            private_dir(&self.root.join("workspaces"))?;
            self.persist(&path, &receipt)?;
        }
        // Le maître a réservé sa Task avant ce second Prepare. Aucun bootstrap
        // n'existe encore et aucun verrou métier ne traverse cette E/S Git.
        let record = crate::workspace_git::create(&plan, &command.task_id)
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
        || metadata.uid() != unsafe { libc::geteuid() }
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
    fn as_raw_fd(&self) -> std::os::fd::RawFd { self.0.as_raw_fd() }
}
impl Drop for WorkerFileLock {
    fn drop(&mut self) {
        while unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) } != 0 {
            if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted { break; }
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
        || metadata.uid() != unsafe { libc::geteuid() }
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
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64).unwrap_or(0)
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
        WorkerStoreError::Invalid(WorkerRefusal::ResultSlotExpired) => WorkerRefusal::ResultSlotExpired,
        WorkerStoreError::Io(ref error)
            if matches!(error.raw_os_error(), Some(libc::ENOSPC | libc::EDQUOT)) =>
        {
            WorkerRefusal::DiskFull
        }
        WorkerStoreError::Io(ref error)
            if matches!(error.raw_os_error(), Some(libc::EACCES | libc::EPERM | libc::EROFS)) =>
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

enum WorkerInbound {
    Request(WorkerRequest),
    CandidateProbe(CandidateProbe),
    CandidateProbeV2(CandidateProbeV2),
}

// Une file pleine produit une réponse corrélée et ne coupe pas les autres
// missions. La demande refusée n'entre jamais dans la file du consommateur.
fn worker_inbound_busy(inbound: WorkerInbound) -> WrapperToDaemon {
    match inbound {
        WorkerInbound::Request(request @ WorkerRequest {
            request: WorkerRequestOperation::StartDetailed { .. }, ..
        }) => WrapperToDaemon::WorkerResponse {
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
                    } else { WorkerStartCapacityCause::InboundQueue },
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
                response: WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResourceLimit },
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
                verdicts: probe.candidates.into_iter().map(|candidate| CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id, reason: CandidateProbeRefusal::ResourceLimit,
                }).collect(),
            },
        },
        WorkerInbound::CandidateProbeV2(probe) => {
            let now = worker_now();
            WrapperToDaemon::CandidateProbeResultV2 {
                result: CandidateProbeV2Result {
                    version: probe.version,
                    correlation_id: probe.correlation_id,
                    connection_epoch: probe.connection_epoch,
                    worker: probe.worker,
                    admission_id: probe.admission_id,
                    policy_revision: probe.policy_revision,
                    resources: HostResourceSnapshotV1 {
                        source_observed_at: now, received_at: now,
                        cpu_percent: None, available_memory_percent: None,
                        normalized_load_percent: None, compilation_slots: None,
                    },
                    verdicts: probe.candidates.into_iter().map(|candidate| CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate.candidate_id, reason: CandidateProbeRefusal::ResourceLimit,
                    }).collect(),
                },
            }
        }
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
        command, request: WorkerRequestOperation::Cancel { target: WorkerCancelScope::Command {} }, ..
    }) = &inbound {
        let key = json_digest(command).map_err(|_| "worker_cancel_invalid")?;
        let mut cancelled = cancelled.lock().map_err(|_| "worker_cancel_poisoned")?;
        if cancelled.len() >= MAX_WORKER_STORE_ENTRIES {
            return Err("worker_cancel_capacity".into());
        }
        cancelled.insert(key);
    }
    match sender.try_send(inbound) {
        Ok(()) => Ok(()),
        Err(mpsc::TrySendError::Full(inbound)) => send_worker_frame(writer, &worker_inbound_busy(inbound)),
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
    last_maintenance: Instant,
    cancelled: Arc<Mutex<BTreeSet<String>>>,
    cancel_signal_issued: BTreeSet<String>,
    host_resources: crate::host_resources::HostResourceCollector,
    last_resource_refresh: Instant,
}

impl WorkerRuntime {
    fn read_view(&self) -> read::WorkerReadView<'_> {
        read::WorkerReadView { config: &self.config, identity: &self.identity,
            credential: &self.credential, source: &self.source, quiet: false }
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
            last_maintenance: Instant::now() - Duration::from_secs(1),
            cancelled: Arc::new(Mutex::new(BTreeSet::new())),
            cancel_signal_issued: BTreeSet::new(),
            host_resources: crate::host_resources::HostResourceCollector::default(),
            last_resource_refresh: Instant::now(),
        };
        runtime
            .reconcile_all()
            .map_err(|e| format!("worker_reconcile:{e:?}"))?;
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

    fn mapping_paths_and_git(
        &self,
        request: &WorkerPrepare,
        mapping: ProjectMapping,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        self.read_view().mapping_paths_and_git(request, mapping, workload)
    }

    /// O(h), h<=256 ancêtres ; une erreur ou un marqueur ambigu reste un refus.
    /// Aucun stderr/code Git ne sert à déduire l'absence de dépôt.

    fn check_routing_connection(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        routing: &RoutingPreparation,
    ) -> Result<(), WorkerRefusal> {
        self.read_view().check_routing_connection(registry, request, routing)
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
    fn inspect_candidate(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    ) -> Result<CandidateReadiness, WorkerRefusal> {
        let pinned = registry
            .clone()
            .pin_delegation_model(
                &request.agent_type,
                &request.model,
                request.effort.as_deref(),
            )
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        crate::lifecycle::launch_preflight(&pinned, &request.agent_type, &self.source)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        let projection_definition = pinned.get(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        let (projection, projection_guard) = crate::native_permissions::observe_projection171(||
            crate::native_permissions::project_remote_permissions_in_operation(
                &request.parent,
                projection_definition,
                mapping,
                cwd,
                &self.source,
                request.posture,
                &self.protected(), operation,
            ));
        let (mut policy, effective_posture) = projection.map_err(|code|
            map_probe_projection_failure171(&code, diagnostics, projection_guard, &request.parent.provider_policy,
                |note| eprintln!("bridget_probe_permission_refusal {note}")))?;
        let scoped = pinned
            .for_inherited_delegation_in_operation(
                &request.agent_type,
                &request.model,
                request.effort.as_deref(),
                &policy, operation,
            )
            .map_err(|code| observe_probe_permission_failure(&code, diagnostics, ProbePermissionSite::ScopedConstructor))?;
        let definition = scoped
            .get(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        // Les arguments effectivement contrôlés sont capturés avant le digest
        // de droits. La capture reste une lecture de configuration locale.
        if definition.protocol == "claude_stream_json" {
            crate::native_permissions::refresh_remote_permissions_context_in_operation(
                &mut policy, definition, cwd, &self.source, mapping, operation,
            ).map_err(|code| observe_probe_permission_failure(&code, diagnostics, ProbePermissionSite::Refresh))?;
        }
        crate::native_permissions::recheck_remote_child_permissions_in_operation(
            &policy,
            definition,
            cwd,
            &self.source,
            &self.protected(),
            Some(mapping),
            Some(&request.parent), operation,
        )
        .map_err(|code| observe_probe_permission_failure(&code, diagnostics, ProbePermissionSite::Recheck))?;
        let definition = scoped
            .resolved_definition(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        Ok(CandidateReadiness {
            definition,
            child_policy: policy,
            effective_posture,
        })
    }

    /// Convertit un tuple opaque de probe en la même demande de lecture que
    /// `Prepare`. Les champs de machine restent attestés par l'identité active,
    /// jamais proposés par le décideur.
    fn candidate_prepare(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        mapping_revision: u64,
    ) -> WorkerPrepare {
        WorkerPrepare {
            machine_id: self.identity.machine_id.clone(),
            project_id: candidate.project_id.clone(),
            provisioning_revision: self.identity.provisioning_revision,
            // La révision n'est jamais fournie par le décideur. Elle provient
            // de la lecture fraîche du registre worker et est renvoyée dans le
            // verdict afin que le maître fige le fait réellement observé.
            mapping_revision,
            origin_host: candidate.origin_host.clone(),
            origin_cwd: candidate.origin_cwd.clone(),
            source_root: candidate.source_root.clone(),
            remote_root: candidate.remote_root.clone(),
            source_root_identity: candidate.source_root_identity.clone(),
            relative_cwd: candidate.relative_cwd.clone(),
            base_commit: candidate.base_commit.clone(),
            agent_type: candidate.agent_type.clone(),
            model: candidate.model.clone(),
            effort: candidate.effort.clone(),
            posture: candidate.posture,
            parent: probe.parent.clone(),
            workspace: candidate.workspace.clone(),
        }
    }

    /// Préconditions d'un worktree futur : Git reste strictement en lecture.
    /// La création et le cwd réel restent réservés à `prepare_workspace` du
    /// seul élu après admission.
    /// Le worker traite ses commandes en série. Les préparations et reçus
    /// durables couvrent aussi un redémarrage : aucune mission connue ne peut
    /// changer de fichiers sous ses pieds, même si son état est incertain.
    /// La correspondance identifie le depot ; les mutations Shared visent
    /// son checkout reel, comme refreshCheckedOutBranch(cwd) dans T3.
    fn checkout_root(&self, mapping: &ProjectMapping, cwd: &Path) -> Result<PathBuf, WorkerRefusal> {
        let git_path = |dir: &Path, flag: &str| -> Result<PathBuf, WorkerRefusal> {
            let output = crate::workspace_git::run_git(dir, &["rev-parse", flag], crate::workspace_git::READ_TIMEOUT)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if !output.success() || output.truncated { return Err(WorkerRefusal::PlacementMismatch); }
            let path = Path::new(output.stdout.trim_end_matches(['\r', '\n']));
            let absolute = if path.is_absolute() { path.to_path_buf() } else { dir.join(path) };
            fs::canonicalize(absolute).map_err(|_| WorkerRefusal::PlacementMismatch)
        };
        let root = git_path(cwd, "--show-toplevel")?;
        if (!root.starts_with(&mapping.remote_root) && !mapping.remote_root.starts_with(&root))
            || !cwd.starts_with(&root)
            || git_path(&root, "--git-common-dir")? != git_path(&mapping.remote_root, "--git-common-dir")? {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        // Une correspondance limitee a un sous-dossier ne donne pas le droit
        // de rafraichir le checkout entier : garder sa borne historique.
        let authorized = if root.starts_with(&mapping.remote_root) { &root } else { &mapping.remote_root };
        crate::registry::validate_unprotected_local_path(authorized, &self.protected())
            .map_err(|_| WorkerRefusal::PlacementMismatch)
    }

    fn remap_prepared_worktree(&self, mapping: &mut ProjectMapping, cwd: &Path,
        record: &crate::delegation::WorkspaceRecord) -> Result<PathBuf, WorkerRefusal> {
        self.read_view().remap_prepared_worktree(mapping, cwd, record)
    }

    fn shared_refresh_allowed(&self, root: &Path, base: &str) -> Result<bool, CandidateProbeRefusal> {
        if !crate::workspace_git::shared_refresh_needed(root, base)? { return Ok(false); }
        let overlaps = |cwd: &Path| cwd.starts_with(root) || root.starts_with(cwd);
        if self.prepared.values().any(|p| overlaps(Path::new(&p.prepared.placement.execution_cwd)))
            || self.store.commands().map_err(|_| CandidateProbeRefusal::WorkspaceUnavailable)?.iter().any(|c|
                !matches!(c.phase, WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused)
                && overlaps(Path::new(&c.placement.execution_cwd))) {
            return Err(CandidateProbeRefusal::WorkspaceBusy);
        }
        Ok(true)
    }

    fn refresh_shared_workspace(&self, root: &Path, base: &str) -> Result<(), WorkerRefusal> {
        if self.shared_refresh_allowed(root, base).map_err(|_| WorkerRefusal::PlacementMismatch)? {
            crate::workspace_git::refresh_shared(root, base).map_err(|_| WorkerRefusal::PlacementMismatch)?;
        }
        Ok(())
    }

    fn probe_workspace_preconditions(
        &self,
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
    ) -> Result<(), CandidateProbeRefusal> {
        if let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        {
            let root = self.checkout_root(mapping, cwd)
                .map_err(|_| CandidateProbeRefusal::MappingUnavailable)?;
            let preflight = crate::workspace_git::preflight_remote(
                &root,
                base_ref.as_deref(),
                branch.as_deref(),
            )
            .map_err(|error| workspace_probe_refusal(&error))?;
            if request
                .base_commit
                .as_ref()
                .is_some_and(|expected| expected != &preflight.base_commit)
            {
                return Err(CandidateProbeRefusal::WorkspaceUnavailable);
            }
            return Ok(());
        }
        if let Some(base) = request.base_commit.as_deref() {
            let root = self.checkout_root(mapping, cwd)
                .map_err(|_| CandidateProbeRefusal::MappingUnavailable)?;
            self.shared_refresh_allowed(&root, base)?;
        }
        Ok(())
    }

    fn probe_one_candidate(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        registry: &AgentRegistry,
    ) -> CandidateProbeVerdict {
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let verdict = self.probe_one_candidate_for(probe, candidate, registry, None, None, &mut operation, None);
        if operation.finish().is_err() {
            return CandidateProbeVerdict::Ineligible { candidate_id: candidate.candidate_id.clone(), reason: CandidateProbeRefusal::PermissionsChanged };
        }
        verdict
    }
    fn probe_one_candidate_for(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        registry: &AgentRegistry,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
        git_facts: Option<&mut ProbeGitFacts>,
        operation: &mut crate::native_permissions::PermissionReadOperation,
        diagnostics: Option<(&ProbePermissionDiagnostics<'_>, &str)>,
    ) -> CandidateProbeVerdict {
        let mapping_revision =
            match registry.project_mapping(&self.identity.machine_id, &candidate.project_id) {
                Ok(mapping) => mapping.mapping_revision,
                Err(_) => {
                    return CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::MappingUnavailable,
                    };
                }
            };
        let request = self.candidate_prepare(probe, candidate, mapping_revision);
        if request.validate().is_err() {
            return CandidateProbeVerdict::Ineligible {
                candidate_id: candidate.candidate_id.clone(),
                reason: CandidateProbeRefusal::InvalidRequest,
            };
        }
        // Cette autorité reste individuelle, même si les faits Git du projet
        // sont déjà connus pour un autre modèle ou agent du même lot.
        let mapping = match self.mapping_authority(&request, registry) {
            Ok(mapping) => mapping,
            Err(_) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::MappingUnavailable,
                };
            }
        };
        let read_git_facts = || {
            let (mapping, cwd) = self
                .mapping_paths_and_git(&request, mapping, workload)
                .map_err(|_| CandidateProbeRefusal::MappingUnavailable)?;
            self.probe_workspace_preconditions(&request, &mapping, &cwd)?;
            Ok((mapping, cwd))
        };
        let checked = if let Some(cache) = git_facts {
            // Worker/provisioning/epoch/registre/protections sont constants dans
            // ce seul lot. Tout fait variable de mapping/base figure dans la clé.
            let key = match serde_json::to_vec(&(
                &request.project_id,
                request.mapping_revision,
                &request.origin_host,
                &request.origin_cwd,
                &request.source_root,
                &request.remote_root,
                &request.source_root_identity,
                &request.relative_cwd,
                &request.workspace,
                &request.base_commit,
                workload,
            )) {
                Ok(key) => key,
                Err(_) => {
                    return CandidateProbeVerdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::InvalidRequest,
                    };
                }
            };
            // Le refus commun est lui aussi mémorisé : une base absente ne
            // provoque pas 128 préflights. Aucun résultat ne quitte ce lot.
            cache.entry(key).or_insert_with(read_git_facts).clone()
        } else {
            read_git_facts()
        };
        let (mapping, cwd) = match checked {
            Ok(values) => values,
            Err(reason) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason,
                };
            }
        };
        let readiness = match self.inspect_candidate(registry, &request, &mapping, &cwd, operation, diagnostics) {
            Ok(readiness) => readiness,
            Err(WorkerRefusal::DefinitionChanged) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::DefinitionUnavailable,
                };
            }
            Err(WorkerRefusal::PermissionsChanged) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::PermissionsChanged,
                };
            }
            Err(_) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::CapabilityUnavailable,
                };
            }
        };
        let permission_digest = match json_digest(&readiness.child_policy) {
            Ok(digest) => digest,
            Err(_) => {
                return CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::CapabilityUnavailable,
                };
            }
        };
        CandidateProbeVerdict::Eligible {
            candidate_id: candidate.candidate_id.clone(),
            mapping_revision: mapping.mapping_revision,
            definition_digest: readiness.definition.digest,
            permission_digest,
            observed_at: worker_now(),
        }
    }

    /// Sonde groupée corrélée à une seule admission. Elle ne touche jamais le
    /// store, `prepared`, `held`, un worktree, un slot ou un fournisseur.
    fn candidate_probe(
        &self,
        probe: &CandidateProbe,
    ) -> Result<CandidateProbeResult, WorkerRefusal> {
        probe.validate_shape()?;
        if probe.worker != self.identity {
            return Err(WorkerRefusal::StaleConnection);
        }
        let registry = self.registry();
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let mut verdicts: Vec<CandidateProbeVerdict> = probe
            .candidates
            .iter()
            .map(|candidate| match &registry {
                Ok(registry) => self.probe_one_candidate_for(probe, candidate, registry, None, None, &mut operation, None),
                Err(_) => CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::CapabilityUnavailable,
                },
            })
            .collect();
        if operation.finish().is_err() {
            for verdict in &mut verdicts {
                if let CandidateProbeVerdict::Eligible { candidate_id, .. } = verdict {
                    *verdict = CandidateProbeVerdict::Ineligible { candidate_id: candidate_id.clone(), reason: CandidateProbeRefusal::PermissionsChanged };
                }
            }
        }
        Ok(CandidateProbeResult {
            version: WORKER_CANDIDATE_PROBE_VERSION,
            correlation_id: probe.correlation_id.clone(),
            connection_epoch: probe.connection_epoch.clone(),
            worker: self.identity.clone(),
            admission_id: probe.admission_id.clone(),
            policy_revision: probe.policy_revision,
            verdicts,
        })
    }

    /// Probe168 : le registre local est l'autorité de la connexion opaque.
    /// Le contrôle est lecture seule et précède toute conversion en Prepare.
    fn candidate_probe_v2(
        &mut self,
        probe: &CandidateProbeV2,
    ) -> Result<CandidateProbeV2Result, WorkerRefusal> {
        probe.validate_shape()?;
        let started=Instant::now();
        if probe.worker != self.identity {
            return Err(WorkerRefusal::StaleConnection);
        }
        let registry = self.registry()?;
        let capacity = self.compilation_capacity(&registry);
        let candidates = registry
            .routing_candidates()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        let policy = registry
            .routing_policy()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        let enabled = policy.enabled
            && policy
                .machine_policies
                .get(&self.identity.machine_id)
                .is_some_and(|policy| policy.enabled);
        let index = candidates
            .iter()
            .map(|entry| {
                (
                    (
                        entry.machine_id.as_str(),
                        entry.connection_id.as_str(),
                        entry.engine.as_str(),
                        entry.agent_type.as_str(),
                        entry.model.as_str(),
                    ),
                    entry,
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        let mut git_facts = ProbeGitFacts::new();
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let diagnostics = ProbePermissionDiagnostics::new(&probe.correlation_id, &probe.admission_id);
        let mut timings=Vec::with_capacity(probe.candidates.len());
        let mut verdicts: Vec<CandidateProbeV2Verdict> = probe
            .candidates
            .iter()
            .map(|candidate| {
                let candidate_started=Instant::now();
                let registered = enabled
                    && index
                        .get(&(
                            self.identity.machine_id.as_str(),
                            candidate.connection_id.as_str(),
                            candidate.engine.as_str(),
                            candidate.agent_type.as_str(),
                            candidate.model.as_str(),
                        ))
                        .is_some_and(|entry| {
                            entry.enabled
                                && candidate
                                    .effort
                                    .as_ref()
                                    .map_or(entry.default_effort.is_none(), |effort| {
                                        entry.supported_efforts.contains(effort)
                                    })
                        });
                if !registered {
                    return CandidateProbeV2Verdict::Ineligible {
                        candidate_id: candidate.candidate_id.clone(),
                        reason: CandidateProbeRefusal::CapabilityUnavailable,
                    };
                }
                let legacy = CandidateProbeCandidate {
                    candidate_id: "c01".into(),
                    project_id: candidate.project_id.clone(),
                    origin_host: candidate.origin_host.clone(),
                    origin_cwd: candidate.origin_cwd.clone(),
                    source_root: candidate.source_root.clone(),
                    remote_root: candidate.remote_root.clone(),
                    source_root_identity: candidate.source_root_identity.clone(),
                    relative_cwd: candidate.relative_cwd.clone(),
                    base_commit: candidate.expected_base_commit.clone(),
                    agent_type: candidate.agent_type.clone(),
                    model: candidate.model.clone(),
                    effort: candidate.effort.clone(),
                    posture: candidate.posture,
                    workspace: candidate.workspace.clone(),
                };
                let legacy_probe = CandidateProbe {
                    version: WORKER_CANDIDATE_PROBE_VERSION,
                    correlation_id: probe.correlation_id.clone(),
                    connection_epoch: probe.connection_epoch.clone(),
                    worker: probe.worker.clone(),
                    admission_id: probe.admission_id.clone(),
                    policy_revision: probe.policy_revision,
                    parent: probe.parent.clone(),
                    candidates: vec![legacy],
                };
                let verdict=match self.probe_one_candidate_for(
                    &legacy_probe,
                    &legacy_probe.candidates[0],
                    &registry,
                    Some(candidate.workload),
                    Some(&mut git_facts), &mut operation,
                    Some((&diagnostics, candidate.candidate_id.as_str())),
                ) {
                    CandidateProbeVerdict::Eligible {
                        mapping_revision,
                        definition_digest,
                        permission_digest,
                        observed_at,
                        ..
                    } => CandidateProbeV2Verdict::Eligible {
                        candidate_id: candidate.candidate_id.clone(),
                        mapping_revision,
                        definition_digest,
                        permission_digest,
                        observed_at,
                    },
                    CandidateProbeVerdict::Ineligible { reason, .. } => {
                        CandidateProbeV2Verdict::Ineligible {
                            candidate_id: candidate.candidate_id.clone(),
                            reason,
                        }
                    }
                };
                timings.push((candidate.candidate_id.clone(),candidate_started.elapsed().as_millis() as u64));
                verdict
            })
            .collect();
        if let Err(code) = operation.finish() {
            for verdict in &mut verdicts {
                if let CandidateProbeV2Verdict::Eligible { candidate_id, .. } = verdict {
                    observe_probe_permission_failure(&code, Some((&diagnostics, candidate_id)), ProbePermissionSite::BatchFinish);
                    *verdict = CandidateProbeV2Verdict::Ineligible { candidate_id: candidate_id.clone(), reason: CandidateProbeRefusal::PermissionsChanged };
                }
            }
        }
        // Les contrôles Git peuvent durer : observer la charge à la remise,
        // pour ne pas rendre périmé un échantillon pris avant leur lecture.
        let resources = self.host_resources.collect(worker_now(), capacity);
        self.last_resource_refresh = Instant::now();
        eprintln!("bridget_worker_probe172 {}",serde_json::json!({
            "correlation_id":probe.correlation_id,"admission_id":probe.admission_id,
            "candidates":probe.candidates.len(),"elapsed_ms":started.elapsed().as_millis() as u64,
            "candidate_elapsed_ms":timings}));
        Ok(CandidateProbeV2Result {
            version: WORKER_CANDIDATE_PROBE_V2_VERSION,
            correlation_id: probe.correlation_id.clone(),
            connection_epoch: probe.connection_epoch.clone(),
            worker: self.identity.clone(),
            admission_id: probe.admission_id.clone(),
            policy_revision: probe.policy_revision,
            resources,
            verdicts,
        })
    }

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
        if inspect_only
            && (routing.is_none()
                || !matches!(
                    request.workspace,
                    Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
                ))
        {
            return Err(WorkerRefusal::InvalidEnvelope);
        }
        if !self.held.is_empty() {
            return Err(WorkerRefusal::ResourceLimit);
        }
        if self.prepared.len() >= 128 && !self.prepared.contains_key(&command.command_id) {
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
        } else if let Some(base) = request.base_commit.as_deref() {
            let root = self.checkout_root(&mapping, &cwd)?;
            self.shared_refresh_allowed(&root, base)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        }
        let initial = self.inspect_candidate(&registry, request, &mapping, &cwd, &mut operation, None)?;
        if let Some((_, expected_definition, expected_permission)) = &routing {
            if initial.definition.digest != *expected_definition {
                return Err(WorkerRefusal::DefinitionChanged);
            }
            if json_digest(&initial.child_policy).map_err(refuse)? != *expected_permission {
                return Err(WorkerRefusal::PermissionsChanged);
            }
        }
        if inspect_only {
            operation.finish().map_err(|_| WorkerRefusal::PermissionsChanged)?;
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
        if !matches!(request.workspace, Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. }))
            && let Some(base) = request.base_commit.as_deref()
        {
            let root = self.checkout_root(&mapping, &cwd)?;
            self.refresh_shared_workspace(&root, base)?;
        }
        Self::verify_base_commit(&cwd, request.base_commit.as_deref())?;
        // Le second contrôle conserve le verdict exact du cwd de worktree
        // choisi, après que le premier a fermé toute création sur révocation.
        let readiness = self.inspect_candidate(&registry, request, &mapping, &cwd, &mut operation, None)?;
        operation.finish().map_err(|_| WorkerRefusal::PermissionsChanged)?;
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
        let routing = fenced_routing.map(|value| RoutingPreparation {
            connection_id: value.connection_id.clone(),
            engine: value.engine.clone(),
            workload: value.workload,
        });
        validate_prepare_command(command)?;
        previous.validate()?;
        request.validate()?;
        placement.validate()?;
        if previous.task_id != command.task_id || previous.command_id == command.command_id
            || !placement.matches_worker(&self.identity)
            || !matches!((&request.workspace, workspace),
                (Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. }), Some(_))
                    | (Some(bridget_transport::protocol::WorkspaceRequest::Shared {}), None))
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
            let original = self.store.fenced_workspace(previous, placement, workspace)
                .map_err(refuse_fenced_workspace)?;
            if !matches!(original.strategy, crate::delegation::WorkspaceStrategy::Worktree)
                || request.base_commit.as_deref() != Some(original.base_commit.as_str())
                || placement.base_commit.as_deref() != Some(original.base_commit.as_str())
                || !matches!(&request.workspace,
                    Some(bridget_transport::protocol::WorkspaceRequest::Worktree { branch, .. })
                        if branch.as_ref().is_none_or(|value| value == &original.branch)) {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            Some(original)
        } else {
            self.store.fenced_placement(previous, placement).map_err(refuse_fenced_workspace)?;
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
            request, &registry, routing.as_ref().map(|selection| selection.workload))?;
        let cwd = if let Some(original) = &original {
            self.remap_prepared_worktree(&mut mapping, &source_cwd, original)?
        } else {
            Self::verify_base_commit(&source_cwd, request.base_commit.as_deref())?;
            source_cwd
        };
        if cwd != Path::new(&placement.execution_cwd) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        if let Some(original) = &original {
            let report = crate::workspace_git::report(original)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if report.head_commit.as_deref() != Some(original.base_commit.as_str())
                || report.head_on_branch != Some(true) {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let readiness = self.inspect_candidate(&registry, request, &mapping, &cwd,
            &mut operation, None)?;
        operation.finish().map_err(|_| WorkerRefusal::PermissionsChanged)?;
        if let Some(value) = fenced_routing {
            if readiness.definition.digest != value.expected_definition_digest {
                return Err(WorkerRefusal::DefinitionChanged);
            }
            if json_digest(&readiness.child_policy).map_err(refuse)?
                != value.expected_permission_digest {
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
        let workspace_value = original.as_ref().map(|value|
            serde_json::to_value(value).map_err(unavailable)).transpose()?;
        if let Some(retained) = self.prepared.get(&command.command_id)
            .filter(|retained| retained.expires > Instant::now()) {
            if retained.command != *command
                || fenced_prepare_canon(&retained.request) != fenced_prepare_canon(request)
                || retained.routing != routing
                || retained.prepared.placement != *placement
                || retained.prepared.workspace != workspace_value
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            if let Some(workspace) = workspace {
                self.store.bind_fenced_workspace(previous, command, request, placement, workspace)
                    .map_err(refuse_fenced_workspace)?;
            } else {
                self.store.fenced_placement(previous, placement).map_err(refuse_fenced_workspace)?;
            }
            // Le canon durable exclut le numéro de tour, pas la preuve fraîche
            // qui devra signer Release. Garder le token sans garder l'ancienne révision.
            let renewed = PreparedRuntime {routing:retained.routing.clone(),command:retained.command.clone(),
                request:request.clone(),prepared:retained.prepared.clone(),expires:retained.expires};
            self.recheck(&renewed)?;
            let prepared=renewed.prepared.clone();
            self.prepared.insert(command.command_id.clone(),renewed);
            return Ok(prepared);
        }
        if let Some(workspace) = workspace {
            self.store.bind_fenced_workspace(previous, command, request, placement, workspace)
                .map_err(refuse_fenced_workspace)?;
        } else {
            self.store.fenced_placement(previous, placement).map_err(refuse_fenced_workspace)?;
        }
        let now = worker_now();
        let prepared = WorkerPrepared {
            placement: placement.clone(), workspace: workspace_value,
            token: uuid::Uuid::new_v4().to_string(),
            definition: readiness.definition, child_policy: readiness.child_policy,
            effective_posture: readiness.effective_posture, prepared_at: now,
        };
        self.prepared.insert(command.command_id.clone(), PreparedRuntime {
            routing, command: command.clone(), request: request.clone(),
            prepared: prepared.clone(), expires: Instant::now() + Duration::from_secs(30),
        });
        Ok(prepared)
    }

    fn recheck(&self, prepared: &PreparedRuntime) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at(prepared, true)
    }

    fn recheck_at(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at_with_repair(prepared,before_release,None)
    }
    fn recheck_at_with_repair(&self, prepared:&PreparedRuntime, before_release:bool,
        repair:Option<&crate::workspace_git::RepairAnchor>) -> Result<AgentRegistry,WorkerRefusal> {
        self.read_view().recheck_at_with_repair(prepared, before_release, repair, &read::ReadTrace::default())
    }

    fn start(
        &mut self,
        command: &WorkerCommandIdentity,
        start: &WorkerStart,
        disconnected: &AtomicBool,
        capacity_cause: &mut WorkerStartCapacityCause,
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
        let registry = self.recheck(prepared)?;
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
            })?
        {
            CommandReservation::Existing(receipt) => {
                return Ok(Self::observation_native(
                    receipt.phase,
                    receipt.native_ready,
                    None,
                ));
            }
            CommandReservation::Created(_) => {}
        }
        let prepared = self.prepared.remove(&command.command_id)
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
        &self,
        command: &WorkerCommandIdentity,
        placement: &RemotePlacement,
        deadline_at: i64,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        if self.held.contains_key(&command.command_id)
            || self.running.contains_key(&command.command_id)
            || self.abandoned.contains_key(&command.command_id)
        {
            return Err(WorkerRefusal::UnknownExecution);
        }
        self.store.fence_unstarted(command, placement, deadline_at, worker_now())
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
            self.recheck(&held.prepared)?;
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
                    kind, instance_id, command_id, generation, ..
                })) if instance_id == &command.instance_id && command_id == &command.command_id
                    && *generation == command.generation => Some(match kind.as_str() {
                        "command_missing" => Failure::CommandMissing,
                        _ => Failure::NegotiationFailed,
                    }),
                Ok(Some(_)) | Err(_) => Some(Failure::InvalidStatus),
                _ if exit.is_some() && !receipt.native_ready => Some(Failure::ExitedBeforeReady),
                _ => None,
            };
            self.store.record_startup_outcome(command, failure, exit.and_then(|s| s.code())).map_err(refuse)?;
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
        let repair=receipt.workspace_repair.as_ref().filter(|r|r.phase=="published");
        let anchor=repair.and_then(|r|workspace_repair::proof(r).ok()).map(|v|v.0);
        if let Some(prepared)=repair.and_then(|r|r.prepared.as_ref()) {
            retained.prepared=prepared.clone();
        }
        if let WorkerResponseOperation::Observation { observation } = &mut response {
            if receipt.workspace_repair.as_ref().is_some_and(|r|r.blocks_dispatch()) {
                observation.native_ready=false;
                return Ok(response);
            }
            let mut workspace_report = None;
            // Un seul rapport frais sert au contrôle et à la réponse. Aucune
            // preuve n'est conservée entre deux Reconcile.
            let permitted = observation.phase == WorkerCommandPhase::Released
                && observation.native_ready
                && self.read_view().recheck_at_with_workspace(
                    &retained, false, anchor.as_ref(), &read::ReadTrace::default(),
                    &mut workspace_report,
                ).is_ok();
            // Une clôture ou un refus précoce des droits garde son observation
            // technique, sans devenir une autorisation de remise.
            if workspace_report.is_none()
                && let Some(value) = retained.prepared.workspace.as_ref()
                && let Ok(record) = serde_json::from_value::<crate::delegation::WorkspaceRecord>(value.clone())
            {
                workspace_report = Some(if let Some(anchor) = anchor.as_ref() {
                    crate::workspace_git::report_repaired_workspace(&record, anchor)
                } else { crate::workspace_git::report(&record) }.map_err(|_| WorkerRefusal::PlacementMismatch));
            }
            observation.workspace = workspace_report.and_then(Result::ok)
                .and_then(|report| serde_json::to_value(report).ok());
            if permitted {
                observation.permission_digest = Some(retained.prepared.placement.permission_digest.clone());
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
        let retained_commands = commands.iter().map(|command| json_digest(&command.identity))
            .collect::<Result<BTreeSet<_>, _>>().map_err(unavailable)?;
        self.cancel_signal_issued.retain(|key| retained_commands.contains(key));
        for command in commands {
            if matches!(
                command.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) {
                continue;
            }
            let cancel_key = json_digest(&command.identity).map_err(unavailable)?;
            let cancel_requested = self.cancelled.lock().map_err(unavailable)?.contains(&cancel_key);
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
    fn abandon_held(&mut self) {
        self.prepared.clear();
        for (command, held) in std::mem::take(&mut self.held) {
            self.abandoned.insert(command, held.child.abandon().0);
        }
    }
    fn tick(&mut self) -> Result<(), WorkerRefusal> {
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
        if let Err(reason) = request.validate() {
            return if matches!(request.request, WorkerRequestOperation::StartDetailed { .. }) {
                Ok(WorkerResponseOperation::StartRejected {
                    reason,
                    cause: if reason == WorkerRefusal::ResourceLimit {
                        WorkerStartCapacityCause::RequestFrame
                    } else { WorkerStartCapacityCause::Unknown },
                })
            } else {
                Err(reason)
            };
        }
        if request.worker != self.identity
            || request.connection_epoch != epoch
            || disconnected.load(Ordering::Acquire)
        {
            return if matches!(request.request, WorkerRequestOperation::StartDetailed { .. }) {
                Ok(WorkerResponseOperation::StartRejected {
                    reason: WorkerRefusal::StaleConnection,
                    cause: WorkerStartCapacityCause::Unknown,
                })
            } else {
                Err(WorkerRefusal::StaleConnection)
            };
        }
        match &request.request {
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
                previous, prepare, placement, workspace, routing,
            } => self.prepare_fenced_routed(&request.command, previous, prepare, placement,
                workspace.as_ref(), routing.as_ref())
                .map(|prepared| WorkerResponseOperation::Prepared { prepared }),
            WorkerRequestOperation::Start { start } => {
                self.start(&request.command, start, disconnected, &mut WorkerStartCapacityCause::Unknown)
            }
            WorkerRequestOperation::StartDetailed { start } => {
                let mut cause = WorkerStartCapacityCause::Unknown;
                match self.start(&request.command, start, disconnected, &mut cause) {
                    Ok(response) => Ok(response),
                    Err(reason) => Ok(WorkerResponseOperation::StartRejected {
                        reason,
                        cause: if reason == WorkerRefusal::ResourceLimit { cause }
                            else { WorkerStartCapacityCause::Unknown },
                    }),
                }
            }
            WorkerRequestOperation::FenceUnstarted { placement, deadline_at } => {
                self.fence_unstarted(&request.command, placement, *deadline_at)
            }
            WorkerRequestOperation::Release { permit } => {
                self.release(&request.command, permit, epoch, disconnected)
            }
            WorkerRequestOperation::ReserveResultSlot { slot }
            | WorkerRequestOperation::ReserveResultSlotUntil { slot, .. } => {
                let key = json_digest(&request.command).map_err(unavailable)?;
                if self.cancelled.lock().map_or(true, |cancelled| cancelled.contains(&key)) {
                    return Err(WorkerRefusal::UnknownExecution);
                }
                let expires_at_ms = match &request.request {
                    WorkerRequestOperation::ReserveResultSlotUntil { expires_at_ms, .. } => Some(*expires_at_ms),
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
            WorkerRequestOperation::RepairWorkspace { repair, action, expires_at } => self.repair_workspace(&request.command,repair,*action,*expires_at),
            WorkerRequestOperation::Reconcile {} => self.reconcile_attested(&request.command),
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
                if receipt.phase != WorkerCommandPhase::Released || receipt.effective_placement().execution_cwd != parent.cwd {
                    return Err(WorkerRefusal::Unauthorized);
                }
                let permission_digest = crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                    .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                let source_error = (|| {
                    let root = fs::canonicalize(&parent.cwd).map_err(|_| "workspace_base_preflight_failed")?;
                    let cwd = fs::canonicalize(&source.cwd).map_err(|_| "workspace_base_preflight_failed")?;
                    if !cwd.starts_with(&root) { return Err("cwd_outside_parent_project".to_owned()); }
                    let mut canonical_source = source.clone();
                    canonical_source.cwd = cwd.to_string_lossy().into_owned();
                    crate::lifecycle::validate_source_base(&canonical_source)
                })().err().map(|mut code| { code.truncate(code.floor_char_boundary(1024)); code });
                let after = crate::native_permissions::prove_parent_permissions(parent, parent.revision)
                    .map_err(|_| WorkerRefusal::PermissionsChanged)?;
                if after != permission_digest { return Err(WorkerRefusal::PermissionsChanged); }
                Ok(WorkerResponseOperation::ParentSourceProof { permission_digest, source_error })
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
            || self.markers.load(&marker_key(command)).map_err(unavailable)? != marker
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
                markers.stop_current_group(&marker_key(command),
                    Duration::from_millis(100), Duration::from_millis(10)).map_err(unavailable)?;
            }
            Ok(())
        })?;
        self.reconcile(command)
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

/// Deux lectures d’origine au plus, conservées au travers des reconnexions.
/// Elles ne bloquent ni Release ni la maintenance du worker.
fn dispatch_origin_proof(request: bridget_transport::protocol::OriginProofRequest,
    writer:Arc<Mutex<UnixStream>>,disconnected:Arc<AtomicBool>,slots:Arc<std::sync::atomic::AtomicUsize>,registry:PathBuf) {
    use bridget_transport::protocol::{OriginProofResult,OriginProofResponse};
    let limit=if matches!(&request.request,bridget_transport::protocol::OriginProofOperation::PrepareProject {..}) {1} else {2};
    let busy=slots.fetch_update(Ordering::AcqRel,Ordering::Acquire,|n| (n<limit).then_some(n+1)).is_err();
    let envelope=request.clone();
    let response=move |result| WrapperToDaemon::OriginProofResponse {response:OriginProofResponse {
        version:envelope.version,correlation_id:envelope.correlation_id.clone(),connection_epoch:envelope.connection_epoch.clone(),
        worker:envelope.worker.clone(),response:result}};
    if busy {let _=send_worker_frame(&writer,&response(OriginProofResult::Rejected {reason:WorkerRefusal::ResourceLimit}));return;}
    struct Permit(Arc<std::sync::atomic::AtomicUsize>);
    impl Drop for Permit {fn drop(&mut self){self.0.fetch_sub(1,Ordering::AcqRel);}}
    let permit=Permit(slots);
    std::thread::spawn(move||{
        let _permit=permit;
        if disconnected.load(Ordering::Acquire){return;}
        let result=crate::origin_proof::execute(&request,&registry);
        if disconnected.load(Ordering::Acquire){return;}
        let frame=response(result);
        if let WrapperToDaemon::OriginProofResponse {response}=&frame {if response.validate_shape().is_err(){return;}}
        let _=send_worker_frame(&writer,&frame);
    });
}

/// Sonde historique sur la même connexion, avant toute variante ou secret162.
/// Le drapeau retourné est une attestation du maître pour proposer la nouvelle
/// enveloppe de capacités ; il ne dépend jamais du build du pair.

#[derive(Debug,Clone,Copy,Default)]
struct MasterWorkerFeatures {
    candidate_probe:bool,probe_v2:bool,release_lease:bool,source_proof:bool,
    slot_deadline:bool,workspace_diagnostics:bool,origin_proof:bool,workspace_repair:bool,
    start_recovery:bool,
}
impl MasterWorkerFeatures {
    fn supports(&self,capability:WorkerCapability)->bool {
        match capability {
            WorkerCapability::CandidateProbeV1=>self.candidate_probe,
            WorkerCapability::CandidateProbeV2=>self.probe_v2,
            WorkerCapability::ReleaseLeaseV2=>self.release_lease,
            WorkerCapability::ParentSourceProofV1=>self.source_proof,
            WorkerCapability::ResultSlotDeadlineV1=>self.slot_deadline,
            WorkerCapability::OriginProofV1=>self.origin_proof,
            WorkerCapability::WorkspaceRepairV1=>self.workspace_repair,
            WorkerCapability::StartRecoveryV1=>self.start_recovery,
        }
    }
    fn capabilities(&self)->Vec<WorkerCapability> {
        [WorkerCapability::CandidateProbeV1,WorkerCapability::CandidateProbeV2,
            WorkerCapability::ReleaseLeaseV2,WorkerCapability::ParentSourceProofV1,
            WorkerCapability::ResultSlotDeadlineV1,WorkerCapability::OriginProofV1,
            WorkerCapability::WorkspaceRepairV1,WorkerCapability::StartRecoveryV1]
            .into_iter().filter(|v|self.supports(*v)).collect()
    }
    fn admits(&self,accepted:&[WorkerCapability])->bool {
        accepted.len()<=8 && accepted.iter().enumerate().all(|(index,capability)|
            self.supports(*capability) && !accepted[..index].contains(capability))
    }
}

fn probe_worker_capability(
    writer:&Mutex<UnixStream>,reader:&mut BufReader<UnixStream>,
)->Result<MasterWorkerFeatures,String> {
    send_worker_frame(writer,&WrapperToDaemon::DaemonIdentityRequest)?;
    let line=bridget_transport::jsonl::read_unix_line(reader,MAX_WORKER_FRAME_BYTES,
        bridget_transport::jsonl::LineDeadline::Absolute(Instant::now()+Duration::from_secs(5)))
        .map_err(|_|"worker_capability_unavailable")?.ok_or("worker_capability_unavailable")?;
    match serde_json::from_slice::<DaemonToWrapper>(&line) {
        Ok(DaemonToWrapper::DaemonIdentityReport {features,..})
            if features.iter().any(|v|v=="remote_worker_v1")=>{
            let has=|name:&str|features.iter().any(|v|v==name);
            Ok(MasterWorkerFeatures {
                candidate_probe:has("worker_candidate_probe_v1"),probe_v2:has("worker_candidate_probe_v2"),
                release_lease:has("worker_release_lease_v2"),source_proof:has("worker_parent_source_proof_v1"),
                slot_deadline:has("worker_result_slot_deadline_v1"),
                workspace_diagnostics:has("worker_workspace_diagnostics_v1"),
                origin_proof:has("worker_origin_proof_v1"),workspace_repair:has("worker_workspace_repair_v1"),
                start_recovery:has("worker_start_recovery_v1"),
            })
        }
        _=>Err("worker_capability_required".into()),
    }
}

#[cfg(test)]
pub(crate) fn probe_capabilities198_for_test(writer:&Mutex<UnixStream>,reader:&mut BufReader<UnixStream>)
    ->Result<Vec<WorkerCapability>,String> {
    probe_worker_capability(writer,reader).map(|features|features.capabilities())
}

/// Client technique supervisé par l'unité existante. Une reconnexion ne détruit
/// ni processus relâché ni reçu ; seuls les permis en mémoire sont abandonnés.
pub fn run_worker(config: WorkerRuntimeConfig) -> Result<(), String> {
    let mut runtime = WorkerRuntime::new(config)?;
    let origin_slots=Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let origin_preparation_slots=Arc::new(std::sync::atomic::AtomicUsize::new(0));
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
        let master_features =
            match probe_worker_capability(&writer, &mut reader) {
                Ok(capabilities) => capabilities,
                Err(code) if matches!(code.as_str(), "worker_connection_lost" | "worker_capability_unavailable") => {
                    // Le même runtime garde ses enfants relâchés pendant la panne du maître.
                    // Aucun credential ni Hello n'est envoyé sans preuve de capacité.
                    drop(reader);
                    drop(writer);
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                Err(code) => return Err(code),
            };
        let master_supports_workspace_diagnostics=master_features.workspace_diagnostics;
        let offered_caps=master_features.capabilities();
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
        let (epoch, negotiated) =
            match serde_json::from_slice::<DaemonToWrapper>(&line) {
                Ok(DaemonToWrapper::WorkerAccepted { accepted })
                    if offered_caps.is_empty() && accepted.validate().is_ok()
                        && accepted.identity==runtime.identity =>
                {
                    (accepted.connection_epoch, Vec::new())
                }
                Ok(DaemonToWrapper::WorkerAcceptedWithCapabilities { accepted })
                    if !offered_caps.is_empty() && accepted.accepted.validate().is_ok()
                        && accepted.accepted.identity==runtime.identity
                        && master_features.admits(&accepted.capabilities) =>
                {
                    (accepted.accepted.connection_epoch, accepted.capabilities)
                }
                Ok(DaemonToWrapper::WorkerRejected { .. }) => {
                    return Err("worker_registration_refused".into());
                }
                _ => return Err("worker_handshake_invalid".into()),
            };
        let candidate_probe_negotiated=negotiated.contains(&WorkerCapability::CandidateProbeV1);
        let candidate_probe_v2_negotiated=negotiated.contains(&WorkerCapability::CandidateProbeV2);
        let slot_deadline_negotiated=negotiated.contains(&WorkerCapability::ResultSlotDeadlineV1);
        let origin_proof_negotiated=negotiated.contains(&WorkerCapability::OriginProofV1);
        let workspace_repair_negotiated=negotiated.contains(&WorkerCapability::WorkspaceRepairV1);
        let start_recovery_negotiated=negotiated.contains(&WorkerCapability::StartRecoveryV1);
        runtime.host_resources.reset();
        runtime.last_resource_refresh = Instant::now();
        let disconnected = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::sync_channel::<WorkerInbound>(8);
        let read_disconnected = disconnected.clone();
        runtime
            .cancelled
            .lock()
            .map_err(|_| "worker_cancel_poisoned")?
            .clear();
        let cancelled = runtime.cancelled.clone();
        let read_epoch = epoch.clone();
        let read_worker = runtime.identity.clone();
        let writer_for_reader = writer.clone();
        let origin_slots=origin_slots.clone();
        let origin_preparation_slots=origin_preparation_slots.clone();
        let origin_registry=runtime.config.registry_path.clone();
        let reader_thread = std::thread::spawn(move || {
            while !read_disconnected.load(Ordering::Acquire) {
                let frame = bridget_transport::jsonl::read_unix_line(
                    &mut reader,
                    MAX_WORKER_FRAME_BYTES,
                    bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(5)),
                );
                let inbound = match frame {
                    Ok(Some(bytes)) => match serde_json::from_slice::<DaemonToWrapper>(&bytes) {
                        Ok(DaemonToWrapper::OriginProofRequest { request })
                            if origin_proof_negotiated && request.connection_epoch==read_epoch
                                && request.worker==read_worker && request.validate_shape().is_ok() => {
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
                                    || !matches!(
                                        &request.request,
                                        WorkerRequestOperation::PrepareV2 { .. }
                                    ))
                                && (slot_deadline_negotiated || !matches!(
                                    &request.request, WorkerRequestOperation::ReserveResultSlotUntil { .. }
                                ))
                                && (workspace_repair_negotiated || !matches!(
                                    &request.request, WorkerRequestOperation::RepairWorkspace { .. }
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
                                && probe.connection_epoch == read_epoch
                                && probe.worker == read_worker
                                && probe.validate_shape().is_ok() =>
                        {
                            WorkerInbound::CandidateProbeV2(probe)
                        }
                        _ => {
                            log::warn!("worker_channel_reader_closed reason=unexpected_frame");
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
                if dispatch_worker_inbound(&sender, inbound, &writer_for_reader,
                    &read_disconnected, &cancelled).is_err()
                {
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
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(WorkerInbound::Request(request)) => {
                    let response = match runtime.handle(&request, &epoch, &disconnected) {
                        Ok(response) => response,
                        Err(reason) => WorkerResponseOperation::Rejected { reason },
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
                Ok(WorkerInbound::CandidateProbe(probe)) => {
                    if disconnected.load(Ordering::Acquire) {
                        break;
                    }
                    let mut result = match runtime.candidate_probe(&probe) {
                        Ok(result) => result,
                        Err(_) => {
                            // Un probe hors contrat ou hors époque a déjà fermé
                            // le lecteur. Ne jamais répondre par une admission
                            // partielle ni tenter une reconnexion locale.
                            fatal = Some("worker_candidate_probe_invalid".into());
                            break;
                        }
                    };
                    for verdict in &mut result.verdicts {
                        if let CandidateProbeVerdict::Ineligible { reason, .. } = verdict {
                            *reason = workspace_probe_reason_for_master(*reason, master_supports_workspace_diagnostics);
                        }
                    }
                    if send_worker_frame(&writer, &WrapperToDaemon::CandidateProbeResult { result })
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(WorkerInbound::CandidateProbeV2(probe)) => {
                    if disconnected.load(Ordering::Acquire) {
                        break;
                    }
                    let mut result = match runtime.candidate_probe_v2(&probe) {
                        Ok(result) => result,
                        Err(_) => {
                            fatal = Some("worker_candidate_probe_v2_invalid".into());
                            break;
                        }
                    };
                    for verdict in &mut result.verdicts {
                        if let CandidateProbeV2Verdict::Ineligible { reason, .. } = verdict {
                            *reason = workspace_probe_reason_for_master(*reason, master_supports_workspace_diagnostics);
                        }
                    }
                    if send_worker_frame(
                        &writer,
                        &WrapperToDaemon::CandidateProbeResultV2 { result },
                    )
                    .is_err()
                    {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
        disconnected.store(true, Ordering::Release);
        if let Ok(socket) = writer.lock() {
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
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
mod tests {
    use super::*;
    use bridget_transport::protocol::{AutoDelegateWorkload, CandidateProbeV2Candidate, ProviderPermissions};
    use bridget_transport::protocol::{RemoteResultDeferredReason, RemoteResultDisposition};
    use std::os::unix::fs::symlink;





    #[test]
    fn spec188_diagnostic_refusal_preserves_identity_and_actionable_unknown() {
        let v=diagnostic_failure("diagnostic_receipt_unavailable",&command());
        assert_eq!(v["command"],serde_json::to_value(command()).unwrap());
        assert!(v["recovery"].as_str().unwrap().contains("ne prouve"));
        assert!(!v["human_action"].as_str().unwrap().is_empty());
        let v=diagnostic_failure("diagnostic_observation_changed",&command());
        assert!(v["recovery"].as_str().unwrap().contains("même tuple"));
    }

    #[test]
    fn spec188_diagnostic_checks_stability_after_early_identity_and_context_errors() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let mut foreign=command();foreign.instance_id="foreign".into();
        let result=read::diagnose_observed(runtime.read_view(),&foreign,|| {
            let mut receipt=runtime.store.command(&command()).unwrap();receipt.updated_at+=1;
            runtime.store.persist(&runtime.store.command_path(&command()),&receipt).unwrap();
        });
        assert_eq!(result.unwrap_err(),"diagnostic_observation_changed");
        let original=fs::read(&runtime.config.registry_path).unwrap();
        fs::write(&runtime.config.registry_path,b"invalid registry").unwrap();
        let result=read::diagnose_local_observed(&runtime.config,&command(),|| {
            fs::write(&runtime.config.registry_path,&original).unwrap();
        });
        assert_eq!(result.unwrap_err(),"diagnostic_observation_changed");
    }
    #[test]
    fn spec188_registry_fifo_and_oversize_are_bounded_without_writer() {
        let fixture=Fixture::new();let p=fixture.0.join("registry-fifo");
        let c=std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe {libc::mkfifo(c.as_ptr(),0o600)},0);
        let (tx,rx)=std::sync::mpsc::channel();let fifo=p.clone();
        let reader=std::thread::spawn(move||tx.send(AgentRegistry::load_from_path(fifo).is_err()).unwrap());
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap(),true);reader.join().unwrap();
        fs::remove_file(&p).unwrap();write_private_file_atomic(&p,b"{}").unwrap();
        fs::OpenOptions::new().write(true).open(&p).unwrap().set_len(16*1024*1024+1).unwrap();
        assert!(AgentRegistry::load_from_path(p).err().unwrap().contains("volumineux"));
    }
    fn snapshot188(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out=BTreeMap::new();let mut dirs=vec![root.to_path_buf()];
        while let Some(dir)=dirs.pop() {
            for entry in fs::read_dir(dir).unwrap() {
                let entry=entry.unwrap();let p=entry.path();let m=fs::symlink_metadata(&p).unwrap();
                if m.is_dir() {dirs.push(p)} else if m.is_file() {out.insert(p,fs::read(entry.path()).unwrap());}
            }
        }
        out
    }
    #[test]
    fn spec188_diagnostic_reads_locked_store_without_write_or_terminal_inference() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let before=snapshot188(&fixture.0);
        let result=diagnose_local(&runtime.config,&command()).unwrap();
        assert_eq!(result["scope"],"local_read_only_recheck");
        assert_eq!(result["authoritative"],false);
        assert_eq!(result["check"],"refused");assert_eq!(result["stage"],"preparation");
        assert_eq!(result["reason"],"UnknownExecution");
        assert_eq!(runtime.store.command(&command()).unwrap().phase,WorkerCommandPhase::Reserved);
        assert_eq!(snapshot188(&fixture.0),before,"ni reçu, verrou ni fichier nouveau");
        assert!(!result.to_string().contains("worker162-test-secret"));
    }
    #[test]
    fn spec188_diagnostic_rejects_foreign_identity_and_absent_receipt_without_write() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let before=snapshot188(&fixture.0);
        let mut foreign=command();foreign.instance_id="foreign-instance".into();
        assert_eq!(diagnose_local(&runtime.config,&foreign).unwrap_err(),"diagnostic_identity_mismatch");
        foreign.command_id="missing-command".into();
        assert_eq!(diagnose_local(&runtime.config,&foreign).unwrap_err(),"diagnostic_receipt_unavailable");
        assert_eq!(snapshot188(&fixture.0),before);
    }
    #[test]
    fn spec188_diagnostic_rejects_symlink_and_fifo_before_open() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let p=runtime.store.command_path(&command());let moved=p.with_extension("preserved");fs::rename(&p,&moved).unwrap();
        symlink(&moved,&p).unwrap();
        assert_eq!(diagnose_local(&runtime.config,&command()).unwrap_err(),"diagnostic_file_type_invalid");
        fs::remove_file(&p).unwrap();
        let name=std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe {libc::mkfifo(name.as_ptr(),0o600)},0);
        assert_eq!(diagnose_local(&runtime.config,&command()).unwrap_err(),"diagnostic_file_type_invalid");
        assert!(moved.is_file());
    }
    #[test]
    fn spec188_diagnostic_marks_concurrent_change_instead_of_returning_stale_verdict() {
        let fixture=Fixture::new();let runtime=runtime_fixture(&fixture);reserve(&runtime.store);
        let observed=read::diagnose_observed(runtime.read_view(),&command(),|| {
            let mut receipt=runtime.store.command(&command()).unwrap();receipt.updated_at+=1;
            runtime.store.persist(&runtime.store.command_path(&command()),&receipt).unwrap();
        });
        assert_eq!(observed.unwrap_err(),"diagnostic_observation_changed");
        assert_eq!(runtime.store.command(&command()).unwrap().phase,WorkerCommandPhase::Reserved);
    }
    #[test]
    fn spec188_diagnostic_permission_codes_are_closed() {
        assert_eq!(permission_diagnostic_code("permission_hook_settings_changed"),"permission_hook_settings_changed");
        for secret in ["private token=secret", "permission_secret_token", "permission_hook_settings_changed:secret"] {
            assert_eq!(permission_diagnostic_code(secret),"unknown");
        }
    }

    #[test]
    fn worker172_inbound_r7_failed_signal_can_be_retried() {
        let mut issued = BTreeSet::new();
        let attempts = std::cell::Cell::new(0);
        let first = worker_stop_once(&mut issued, "command", || {
            attempts.set(attempts.get() + 1);
            Err(WorkerRefusal::StoreUnavailable)
        });
        assert_eq!(first, Err(WorkerRefusal::StoreUnavailable));
        assert!(issued.is_empty(), "un refus de signal ne vaut pas envoi");
        worker_stop_once(&mut issued, "command", || {
            attempts.set(attempts.get() + 1);
            Ok(())
        }).unwrap();
        worker_stop_once(&mut issued, "command", || panic!("signal déjà envoyé")).unwrap();
        assert_eq!(attempts.get(), 2);
        assert!(issued.contains("command"));
    }

    #[test]
    fn worker172_inbound_r7_expired_marker_error_is_scoped() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        let first = command();
        let mut second = first.clone();
        second.command_id = "other-expired-command".into();
        second.task_id = "other-expired-task".into();
        runtime.store.reserve_command(&first, &"d".repeat(64), &placement(), now - 1, now - 2).unwrap();
        runtime.store.reserve_command(&second, &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let marker = ManagedMarker { pgid: 9999999, birth: 1, instance_id: first.instance_id.clone(),
            command_id: first.command_id.clone(), generation: first.generation };
        runtime.store.mark_command(&first, &marker, now - 2).unwrap();
        let outcome = runtime.tick();
        eprintln!("{}", serde_json::json!({"case":"expired_marker_error","tick_ok":outcome.is_ok(),
            "other_phase":runtime.store.command(&second).unwrap().phase}));
        assert_eq!(outcome, Ok(()));
        assert_eq!(runtime.store.command(&second).unwrap().phase, WorkerCommandPhase::Unknown);
    }

    #[test]
    fn worker172_inbound_r7_cancel_before_marker_reconciles_unknown() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        runtime.tick().unwrap();
        let phase = runtime.store.command(&command()).unwrap().phase;
        eprintln!("{}", serde_json::json!({"case":"cancel_before_marker","phase":phase}));
        assert_eq!(phase, WorkerCommandPhase::Unknown);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn worker172_inbound_r7_held_abandon_does_not_hide_stop_signal() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let script_path = fixture.0.join("held-bootstrap.py");
        let trace = fixture.0.join("held-term");
        let stop = fixture.0.join("held-stop");
        struct StopGuard(PathBuf);
        impl Drop for StopGuard {
            fn drop(&mut self) { let _ = fs::write(&self.0, b"stop"); }
        }
        let _stop_guard = StopGuard(stop.clone());
        let script = format!(r#"#!/usr/bin/python3
import json,os,signal,sys,time
from pathlib import Path
trace=Path({trace:?})
stop=Path({stop:?})
def term(sig,frame):
 trace.write_text("TERM\n")
 sys.exit(0)
signal.signal(signal.SIGTERM,term)
os.setpgid(0,0)
args=sys.argv
birth=int(Path("/proc/self/stat").read_text().rsplit(")",1)[1].split()[19])
ready={{"event":"bootstrap_ready","pid":os.getpid(),"pgid":os.getpid(),"birth":birth,
 "instance_id":args[args.index("--instance-id")+1],"command_id":args[args.index("--command-id")+1],
 "generation":int(args[args.index("--generation")+1])}}
os.write(101,(json.dumps(ready)+"\n").encode())
while not stop.exists(): time.sleep(.01)
"#, trace=trace.to_string_lossy(),stop=stop.to_string_lossy());
        fs::write(&script_path,script).unwrap();
        fs::set_permissions(&script_path,fs::Permissions::from_mode(0o700)).unwrap();
        let launch = crate::managed_process::ManagedLaunch {
            bootstrap_executable: script_path, wrapper_executable: "/bin/true".into(), wrapper_args: vec![],
            identity: crate::managed_process::ManagedIdentity { instance_id: command().instance_id,
                command_id: command().command_id, generation: command().generation },
            cwd: fixture.0.clone(), env: BTreeMap::new(),
        };
        let mut bootstrap = crate::managed_process::spawn_managed_bootstrap(&launch).unwrap();
        bootstrap.set_ready_timeout(Duration::from_secs(2)).unwrap();
        let marked = bootstrap.wait_ready().unwrap().persist_marker(&runtime.markers,&marker_key(&command())).unwrap();
        let marker = marked.marker().marker().clone();
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        runtime.store.mark_command(&command(),&marker,now).unwrap();
        let parent = ProviderPermissions {
            version:1, source:"native_wrapper".into(),run_id:"run".into(),provider_session_id:"session".into(),
            provider_instance_id:"parent".into(),revision:1,driver:"codex_app_server".into(),
            cwd:"/origin".into(),runtime_mode:"full-access".into(),interaction_mode:"default".into(),
            provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
        };
        let request = WorkerPrepare {
            machine_id:"cartae".into(),project_id:"project".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin".into(),origin_cwd:"/origin".into(),source_root:"/origin".into(),remote_root:"/remote".into(),
            source_root_identity:"git:held".into(),relative_cwd:".".into(),base_commit:None,agent_type:"codex".into(),
            model:"fixture".into(),effort:None,posture:None,workspace:None,parent,
        };
        let definition = ResolvedAgentDefinition {
            command:"/bin/true".into(),args:vec![],protocol:"codex_app_server".into(),forbidden_env:vec![],
            pass_env:vec![],permissions:"plan".into(),claude_config_dir:None,queue_capacity:1,notify_timeout_secs:60,
            mcp:bridget_transport::protocol::ResolvedMcpDefinition{interactive:"off".into(),acp_session:false},
            capabilities:Default::default(),digest:"a".repeat(64),
        };
        let prepared = PreparedRuntime { routing:None,command:command(),request,
            prepared:WorkerPrepared { token:"held".into(),placement:placement(),definition,child_policy:serde_json::json!({}),
                effective_posture:SpawnPosture::Discovery,prepared_at:now,workspace:None },
            expires:Instant::now()+Duration::from_secs(300) };
        runtime.held.insert(command().command_id,HeldRuntime { prepared,mission_id:"held-mission".into(),
            child:marked,ready_at:now,expires:Instant::now()+Duration::from_secs(300) });
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        for _ in 0..3 {
            runtime.last_maintenance=Instant::now()-Duration::from_secs(2);
            runtime.tick().unwrap();
        }
        let signalled = trace.exists();
        eprintln!("{}",serde_json::json!({"case":"held_stop","signal_observed":signalled}));
        // Sortie garantie même sur RED : le processus de test observe ce fichier.
        fs::write(&stop,b"stop").unwrap();
        if let Some(child)=runtime.abandoned.get_mut(&command().command_id) { child.wait().unwrap(); }
        assert!(signalled,"fermer RELEASE n'est pas une preuve d'envoi de SIGTERM");
    }

    #[test]
    fn worker172_inbound_cancel_maintenance_keeps_other_commands_alive() {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        struct ChildGuard(std::process::Child);
        impl Drop for ChildGuard {
            fn drop(&mut self) { drop(self.0.stdin.take()); let _ = self.0.wait(); }
        }
        let child = ChildGuard(Command::new("/bin/cat").stdin(Stdio::piped())
            .stdout(Stdio::null()).stderr(Stdio::null()).process_group(0).spawn().unwrap());
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        let first = command();
        let mut second = first.clone();
        second.command_id = "other-command".into();
        second.task_id = "other-task".into();
        for identity in [&first, &second] {
            runtime.store.reserve_command(identity, &"d".repeat(64), &placement(), now + 300, now).unwrap();
        }
        let result_slot = slot(24);
        runtime.store.reserve_result_slot(&result_slot, now).unwrap();
        let marker = ManagedMarker {
            pgid: child.0.id(), birth: crate::managed_process::process_birth(child.0.id()).unwrap(), instance_id: first.instance_id.clone(),
            command_id: first.command_id.clone(), generation: first.generation,
        };
        runtime.store.mark_command(&first, &marker, now).unwrap();
        // Le reçu est valide, mais le fichier de marqueur manque : cancel renvoie
        // StoreUnavailable avant tout signal. Ce défaut doit rester à cette commande.
        runtime.cancelled.lock().unwrap().insert(json_digest(&first).unwrap());
        let tick = runtime.tick();
        eprintln!("{}", serde_json::json!({"case":"cancel_maintenance_error",
            "tick_ok":tick.is_ok(),"first_phase":runtime.store.command(&first).unwrap().phase,
            "second_phase":runtime.store.command(&second).unwrap().phase}));
        assert_eq!(tick, Ok(()));
        assert_eq!(runtime.store.command(&second).unwrap().phase, WorkerCommandPhase::Unknown);
        assert!(!matches!(runtime.store.command(&first).unwrap().phase, WorkerCommandPhase::Terminal));
        assert!(runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().closed.is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn worker172_inbound_cancel_maintenance_signals_once_and_waits_for_drain() {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        struct ChildGuard(std::process::Child);
        impl Drop for ChildGuard {
            fn drop(&mut self) { drop(self.0.stdin.take()); let _ = self.0.wait(); }
        }
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let trace = fixture.0.join("term-count");
        let script = "import signal,sys\ndef stop(sig,frame):\n with open(sys.argv[1],'a') as f: f.write('TERM\\n')\nsignal.signal(signal.SIGTERM,stop)\nprint('ready',flush=True)\nsys.stdin.buffer.read()\n";
        let mut child = ChildGuard(Command::new("python3").args(["-u","-c",script])
            .arg(&trace).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
            .process_group(0).spawn().unwrap());
        let mut ready = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(child.0.stdout.take().unwrap()), &mut ready).unwrap();
        assert_eq!(ready.trim(), "ready");
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let result_slot = slot(25);
        runtime.store.reserve_result_slot(&result_slot, now).unwrap();
        let marker = ManagedMarker {
            pgid: child.0.id(), birth: crate::managed_process::process_birth(child.0.id()).unwrap(),
            instance_id: command().instance_id, command_id: command().command_id, generation: command().generation,
        };
        runtime.store.mark_command(&command(), &marker, now).unwrap();
        write_json(&runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command()))), &marker).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        for _ in 0..3 {
            runtime.last_maintenance = Instant::now() - Duration::from_secs(2);
            runtime.tick().unwrap();
        }
        let signals = fs::read_to_string(&trace).unwrap().lines().count();
        let phase = runtime.store.command(&command()).unwrap().phase;
        let open = runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().closed.is_none();
        let live = crate::managed_process::group_exists(marker.pgid).unwrap();
        eprintln!("{}", serde_json::json!({"case":"cancel_signal_once","signals":signals,
            "group_live":live,"phase":phase,"slot_open":open}));
        drop(child.0.stdin.take());
        child.0.wait().unwrap();
        let receipt = runtime.store.command(&command()).unwrap();
        write_json(&drain_proof_path(&runtime.store.root, &command()), &WorkerDrainProof::expected(&receipt).unwrap()).unwrap();
        runtime.last_maintenance = Instant::now() - Duration::from_secs(2);
        runtime.tick().unwrap();
        let closed = runtime.store.read_slot(&result_slot.identity).unwrap().unwrap();
        assert_eq!(signals, 1, "un même arrêt ne doit pas envoyer SIGTERM à chaque tick");
        assert!(live && open && phase != WorkerCommandPhase::Terminal, "pas de clôture fictive tant que le groupe vit");
        assert_eq!(runtime.store.command(&command()).unwrap().phase, WorkerCommandPhase::Terminal);
        assert!(closed.producer_terminal && closed.closed == Some(WorkerSlotClosureProof::Cancelled));
    }

    #[test]
    fn worker172_inbound_expired_reservation_has_no_store_effect() {
        let fixture = Fixture::new();
        let store = fixture.open();
        let now = worker_now();
        store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let result = store.reserve_result_slot_before(&slot(20), now, Some((now - 1) * 1000));
        let slots = store.inventory().unwrap().1;
        eprintln!("{}", serde_json::json!({"case":"expired_reservation","request_deadline_ms":(now-1)*1000,
            "worker_now":now,"command_deadline":now+300,"accepted":result.is_ok(),"slots_written":slots.len()}));
        assert!(result.is_err(), "une réservation déjà expirée doit être refusée");
        assert!(slots.is_empty(), "aucune écriture après échéance");
    }

    #[test]
    fn worker172_inbound_cancelled_reservation_is_refused() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let reserved = slot(21);
        runtime.store.reserve_result_slot(&reserved, now).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&command()).unwrap());
        let WorkerInbound::Request(request) = inbound172("cancelled-slot", WorkerRequestOperation::ReserveResultSlot { slot: reserved }) else { unreachable!() };
        let result = runtime.handle(&request, "epoch", &AtomicBool::new(false));
        eprintln!("{}", serde_json::json!({"case":"cancelled_reservation","cancel_marker":true,"accepted":result.is_ok(),
            "open_slots":runtime.store.inventory().unwrap().1.iter().filter(|s|s.closed.is_none()).count()}));
        assert!(result.is_err(), "une annulation déjà observée interdit une réservation");
    }

    #[test]
    fn worker172_inbound_deadline_is_checked_after_queueing() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let now = worker_now();
        runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let (socket, mut peer) = UnixStream::pair().unwrap();
        let mut input = std::io::BufReader::new(socket.try_clone().unwrap());
        let writer = Arc::new(Mutex::new(socket));
        let expires_at_ms = worker_now_millis() + 100;
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "queued-deadline".into(),
            connection_epoch: "epoch".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::ReserveResultSlotUntil { slot: slot(22), expires_at_ms },
        };
        request.validate().unwrap();
        writeln!(peer, "{}", serde_json::to_string(&DaemonToWrapper::WorkerRequest { request }).unwrap()).unwrap();
        let bytes = bridget_transport::jsonl::read_unix_line(&mut input, MAX_WORKER_FRAME_BYTES,
            bridget_transport::jsonl::LineDeadline::Absolute(Instant::now() + Duration::from_secs(1)))
            .unwrap().unwrap();
        let DaemonToWrapper::WorkerRequest { request } = serde_json::from_slice(&bytes).unwrap() else { unreachable!() };
        let (sender, receiver) = mpsc::sync_channel(8);
        let disconnected = AtomicBool::new(false);
        dispatch_worker_inbound(&sender, WorkerInbound::Request(request), &writer, &disconnected, &runtime.cancelled).unwrap();
        std::thread::sleep(Duration::from_millis(150));
        let WorkerInbound::Request(request) = receiver.try_recv().unwrap() else { unreachable!() };
        let response = runtime.handle(&request, "epoch", &disconnected);
        let slots = runtime.store.inventory().unwrap().1;
        eprintln!("{}", serde_json::json!({"case":"queued_deadline","request_deadline_ms":expires_at_ms,
            "worker_now_ms":worker_now_millis(),"accepted":response.is_ok(),"slots_written":slots.len()}));
        assert_eq!(response.unwrap_err(), WorkerRefusal::ResultSlotExpired);
        assert!(slots.is_empty());
        send_worker_frame(&writer, &WrapperToDaemon::WorkerResponse { response: WorkerResponse {
            version: REMOTE_WORKER_VERSION, correlation_id: request.correlation_id.clone(),
            connection_epoch: request.connection_epoch.clone(), worker: request.worker.clone(),
            command: request.command.clone(),
            response: WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResultSlotExpired },
        }}).unwrap();
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut std::io::BufReader::new(peer)) else { unreachable!() };
        response.validate().unwrap();
        assert_eq!(response.correlation_id, "queued-deadline");
        assert_eq!(response.response, WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResultSlotExpired });
    }

    #[test]
    fn worker172_inbound_deadline_is_checked_after_store_lock() {
        let fixture = Fixture::new();
        let store = Arc::new(fixture.open());
        let now = worker_now();
        store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
        let guard = store.transition.lock().unwrap();
        let delayed = store.clone();
        let expires_at_ms = worker_now_millis() + 100;
        let waiting = std::thread::spawn(move || delayed.reserve_result_slot_before(&slot(23), now, Some(expires_at_ms)));
        std::thread::sleep(Duration::from_millis(150));
        drop(guard);
        assert!(matches!(waiting.join().unwrap(), Err(WorkerStoreError::Invalid(WorkerRefusal::ResultSlotExpired))));
        assert!(store.inventory().unwrap().1.is_empty());
    }

    fn inbound172(correlation: &str, request: WorkerRequestOperation) -> WorkerInbound {
        WorkerInbound::Request(WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: correlation.into(),
            connection_epoch: "epoch".into(), worker: worker(), command: command(), request,
        })
    }

    fn read_inbound172(reader: &mut std::io::BufReader<UnixStream>) -> WrapperToDaemon {
        let bytes = bridget_transport::jsonl::read_unix_line(reader, MAX_WORKER_FRAME_BYTES,
            bridget_transport::jsonl::LineDeadline::Absolute(Instant::now() + Duration::from_secs(1)))
            .unwrap().unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn worker172_inbound_saturation_refuses_slot_without_late_effect() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let (sender, receiver) = mpsc::sync_channel(8);
        for index in 0..8 {
            sender.send(inbound172(&format!("queued-{index}"), WorkerRequestOperation::Reconcile {})).ok().unwrap();
        }
        let (socket, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let mut reader = std::io::BufReader::new(peer);
        let disconnected = AtomicBool::new(false);
        let cancelled = Mutex::new(BTreeSet::new());
        let slot = slot(17);
        dispatch_worker_inbound(&sender,
            inbound172("refused-slot", WorkerRequestOperation::ReserveResultSlot { slot }),
            &writer, &disconnected, &cancelled).unwrap();
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut reader) else { panic!("réponse attendue"); };
        response.validate().unwrap();
        assert_eq!(response.correlation_id, "refused-slot");
        assert_eq!(response.connection_epoch, "epoch");
        assert_eq!(response.command, command());
        assert_eq!(response.response, WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResourceLimit });
        assert!(!disconnected.load(Ordering::Acquire));
        // Le consommateur reste bloqué au-delà du délai maître de trois secondes.
        std::thread::sleep(Duration::from_millis(3100));
        for index in 0..8 {
            let WorkerInbound::Request(request) = receiver.try_recv().unwrap() else { panic!("requête attendue"); };
            assert_eq!(request.correlation_id, format!("queued-{index}"));
            let _ = runtime.handle(&request, "epoch", &disconnected);
        }
        assert!(receiver.try_recv().is_err(), "la réservation refusée ne peut être exécutée plus tard");
        assert!(runtime.store.inventory().unwrap().1.is_empty(), "aucun slot orphelin");
        dispatch_worker_inbound(&sender, inbound172("after-saturation", WorkerRequestOperation::Reconcile {}),
            &writer, &disconnected, &cancelled).unwrap();
        let WorkerInbound::Request(request) = receiver.try_recv().unwrap() else { panic!("requête attendue"); };
        assert_eq!(request.correlation_id, "after-saturation", "le canal reste utilisable");
    }

    #[test]
    fn worker172_inbound_saturation_refuses_probes_and_observes_cancel() {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(inbound172("queued", WorkerRequestOperation::Reconcile {})).ok().unwrap();
        let (socket, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let mut reader = std::io::BufReader::new(peer);
        let disconnected = AtomicBool::new(false);
        let cancelled = Mutex::new(BTreeSet::new());
        let parent = ProviderPermissions {
            version: 1, source: "provider_turn".into(), run_id: "inbound-run".into(),
            provider_session_id: "session".into(), provider_instance_id: "codex".into(),
            revision: 1, driver: "codex_app_server".into(), cwd: "/project".into(),
            runtime_mode: "full-access".into(), interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never","sandbox_policy":{"type":"readOnly"}}),
        };
        let candidate = CandidateProbeCandidate {
            candidate_id: "c01".into(), project_id: "project-162".into(), origin_host: "mac".into(),
            origin_cwd: "/project".into(), source_root: "/project".into(), remote_root: "/remote".into(),
            source_root_identity: "repo".into(), relative_cwd: ".".into(), base_commit: None,
            agent_type: "codex".into(), model: "model".into(), effort: None,
            posture: Some(SpawnPosture::Discovery), workspace: None,
        };
        let probe = CandidateProbe {
            version: WORKER_CANDIDATE_PROBE_VERSION, correlation_id: "probe-v1".into(),
            connection_epoch: "epoch".into(), worker: worker(), admission_id: "admission".into(),
            policy_revision: 1, parent: parent.clone(), candidates: vec![candidate.clone()],
        };
        dispatch_worker_inbound(&sender, WorkerInbound::CandidateProbe(probe.clone()), &writer, &disconnected, &cancelled).unwrap();
        let WrapperToDaemon::CandidateProbeResult { result } = read_inbound172(&mut reader) else { panic!("sonde v1 attendue"); };
        result.validate_shape().unwrap();
        assert_eq!(result.correlation_id, probe.correlation_id);
        assert_eq!(result.connection_epoch, probe.connection_epoch);
        assert_eq!(result.admission_id, probe.admission_id);
        assert_eq!(result.policy_revision, probe.policy_revision);
        assert!(matches!(result.verdicts[0], CandidateProbeVerdict::Ineligible { reason: CandidateProbeRefusal::ResourceLimit, .. }));
        let probe = CandidateProbeV2 {
            version: WORKER_CANDIDATE_PROBE_V2_VERSION, correlation_id: "probe-v2".into(),
            connection_epoch: "epoch".into(), worker: worker(), admission_id: "admission".into(),
            policy_revision: 1, parent,
            candidates: vec![CandidateProbeV2Candidate {
                candidate_id: "c001".into(), project_id: candidate.project_id, origin_host: candidate.origin_host,
                origin_cwd: candidate.origin_cwd, source_root: candidate.source_root, remote_root: candidate.remote_root,
                source_root_identity: candidate.source_root_identity, relative_cwd: candidate.relative_cwd,
                expected_base_commit: None, engine: "codex".into(), connection_id: "codex-cartae".into(),
                agent_type: candidate.agent_type, model: candidate.model, effort: None,
                workload: AutoDelegateWorkload::General, posture: candidate.posture, workspace: None,
            }],
        };
        dispatch_worker_inbound(&sender, WorkerInbound::CandidateProbeV2(probe.clone()), &writer, &disconnected, &cancelled).unwrap();
        let WrapperToDaemon::CandidateProbeResultV2 { result } = read_inbound172(&mut reader) else { panic!("sonde v2 attendue"); };
        result.validate_shape().unwrap();
        assert_eq!(result.correlation_id, probe.correlation_id);
        assert_eq!(result.connection_epoch, probe.connection_epoch);
        assert_eq!(result.admission_id, probe.admission_id);
        assert_eq!(result.policy_revision, probe.policy_revision);
        assert!(matches!(result.verdicts[0], CandidateProbeV2Verdict::Ineligible { reason: CandidateProbeRefusal::ResourceLimit, .. }));
        assert!(result.resources.cpu_percent.is_none() && result.resources.available_memory_percent.is_none());
        dispatch_worker_inbound(&sender, inbound172("cancel", WorkerRequestOperation::Cancel { target: WorkerCancelScope::Command {} }),
            &writer, &disconnected, &cancelled).unwrap();
        assert!(cancelled.lock().unwrap().contains(&json_digest(&command()).unwrap()), "annulation observée sans attendre le consommateur");
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut reader) else { panic!("refus corrélé attendu"); };
        assert_eq!(response.correlation_id, "cancel");
        assert!(!disconnected.load(Ordering::Acquire));
        assert!(matches!(receiver.try_recv().unwrap(), WorkerInbound::Request(_)));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn worker172_inbound_shutdown_never_blocks_or_delivers() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let (socket, _peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let stopped = AtomicBool::new(true);
        let cancelled = Mutex::new(BTreeSet::new());
        assert!(dispatch_worker_inbound(&sender, inbound172("stopped", WorkerRequestOperation::Reconcile {}),
            &writer, &stopped, &cancelled).is_err());
        assert!(receiver.try_recv().is_err());
        drop(receiver);
        assert!(dispatch_worker_inbound(&sender, inbound172("closed", WorkerRequestOperation::Reconcile {}),
            &writer, &AtomicBool::new(false), &cancelled).is_err());
    }

    #[test]
    fn worker172_inbound_late_slot_is_closed_by_terminal_reconciliation() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let late = slot(18);
        runtime.store.reserve_result_slot(&late, 101).unwrap();
        runtime.store.set_phase(&command(), WorkerCommandPhase::Terminal, 102).unwrap();
        runtime.reconcile(&command()).unwrap();
        let slots = runtime.store.inventory().unwrap().1;
        assert_eq!(slots.len(), 1);
        assert!(slots[0].producer_terminal);
        assert_eq!(slots[0].closed, Some(WorkerSlotClosureProof::Cancelled));
    }

    #[test]
    fn native171_probe_projection_guard_reasons_closed_bounded_same_refusal() {
        use crate::native_permissions::{ProjectionGuard, equivalence_fail_at,
            observe_projection171, projection_witness_summary171};
        let correlation = "16679ded-7e80-4918-ad3e-7df3ebcdda5d";
        let admission = "a-86a844fb3499046e4428fc09348a1e50";
        let digest = format!("sha256:{}", "a".repeat(64));
        let policy = serde_json::json!({"launch_context":{"source_equivalence":{
            "version":2,"scope":"origin","profile":{"revision":digest}}}});
        let batch = ProbePermissionDiagnostics::new(correlation, admission);
        let mut seen = std::collections::BTreeSet::new();
        for guard in ProjectionGuard::ALL {
            let (result, observed) = observe_projection171::<()>(|| Err(equivalence_fail_at(guard)));
            assert_eq!(result, Err("permission_hook_equivalence_unavailable".into()));
            assert_eq!(observed, Some(guard));
            assert!(seen.insert(guard.as_str()));
            let mut notes = Vec::new();
            let refusal = map_probe_projection_failure171(result.as_ref().unwrap_err(),
                Some((&batch,"c001")), observed, &policy, |note| notes.push(note.to_owned()));
            assert!(matches!(refusal,WorkerRefusal::PermissionsChanged));
            assert_eq!(notes.len(),1);
            let note = &notes[0];
            assert!(note.len() <= MAX_PROBE_PERMISSION_NOTE_BYTES);
            let v: serde_json::Value = serde_json::from_str(note).unwrap();
            assert_eq!(v.as_object().unwrap().len(),8);
            assert_eq!(v["reason"],guard.as_str());
            assert_eq!(v["code"],"permission_hook_equivalence_unavailable");
            assert_eq!(v["site"],"projection");
            assert_eq!(v["candidate_id"],"c001");
            assert_eq!(v["correlation_id"],correlation);
            assert_eq!(v["admission_id"],admission);
            assert_eq!(v["witness"],serde_json::json!({"shape":"object","version":2,
                "scope":"origin","profile":"object","revision":digest}));
        }
        assert_eq!(seen.len(),ProjectionGuard::ALL.len());
        // No previous call's guard survives, including nested synchronous reads.
        let (unknown, observed) = observe_projection171::<()>(|| Err("permission_hook_equivalence_unavailable".into()));
        assert!(unknown.is_err());assert_eq!(observed,None);
        let (success, observed) = observe_projection171(|| Ok(7));
        assert_eq!(success,Ok(7));assert_eq!(observed,None);
        let (_, observed) = observe_projection171::<()>(|| {
            let _ = equivalence_fail_at(ProjectionGuard::G001);
            let (_, inner) = observe_projection171::<()>(|| Err(equivalence_fail_at(ProjectionGuard::G002)));
            assert_eq!(inner,Some(ProjectionGuard::G002));
            Err("permission_hook_equivalence_unavailable".into())
        });
        assert_eq!(observed,Some(ProjectionGuard::G001));
        let (_, observed) = observe_projection171::<()>(|| Err("permission_hook_equivalence_unavailable".into()));
        assert_eq!(observed,None);
        // Malformed and very large user-controlled witness values become type tags.
        let secret = "SECRET /private/settings --token env=hidden STACK".repeat(2000);
        for witness in [serde_json::Value::Null,serde_json::json!(secret),
            serde_json::json!([secret]),serde_json::json!({"version":secret,"scope":secret,
                "profile":{"revision":secret,"unexported":secret},"binding":secret}),
            serde_json::json!({"version":999,"scope":"other","profile":{"revision":"sha256:BAD"}})] {
            let bad = serde_json::json!({"launch_context":{"source_equivalence":witness}});
            let mut note = None;
            let refusal = map_probe_projection_failure171("permission_hook_equivalence_unavailable",
                Some((&batch,"c001")),None,&bad,|v| note=Some(v.to_owned()));
            assert!(matches!(refusal,WorkerRefusal::PermissionsChanged));
            let note=note.unwrap();assert!(note.len()<=512);
            for forbidden in ["SECRET","/private/","--token","hidden","STACK","unexported","binding"] {
                assert!(!note.contains(forbidden));
            }
            let v:serde_json::Value=serde_json::from_str(&note).unwrap();assert_eq!(v["reason"],"unknown");
            assert_eq!(v["witness"].as_object().unwrap().len(),5);
        }
        let missing=projection_witness_summary171(&serde_json::json!({})).json();
        assert_eq!(missing,serde_json::json!({"shape":"absent","version":"absent",
            "scope":"absent","profile":"absent","revision":"absent"}));
        let cap=ProbePermissionDiagnostics::new(correlation,admission);
        for n in 1..=128 {
            let mut emitted=false;
            let r=map_probe_projection_failure171("permission_hook_equivalence_unavailable",
                Some((&cap,&format!("c{n:03}"))),Some(ProjectionGuard::G001),&policy,|v|{
                    emitted=true;assert!(v.len()<=512);
                });
            assert!(emitted);assert!(matches!(r,WorkerRefusal::PermissionsChanged));
        }
        let mut emitted=false;
        let r=map_probe_projection_failure171("permission_hook_equivalence_unavailable",
            Some((&cap,"c001")),Some(ProjectionGuard::G001),&policy,|_|emitted=true);
        assert!(!emitted);assert!(matches!(r,WorkerRefusal::PermissionsChanged));
        for (corr,adm,candidate) in [("bad",admission,"c001"),(correlation,"a-bad","c001"),
            (correlation,admission,"c01"),(correlation,admission,"c129")] {
            let bad=ProbePermissionDiagnostics::new(corr,adm);let mut emitted=false;
            let r=map_probe_projection_failure171("permission_hook_equivalence_unavailable",
                Some((&bad,candidate)),Some(ProjectionGuard::G001),&policy,|_|emitted=true);
            assert!(!emitted);assert!(matches!(r,WorkerRefusal::PermissionsChanged));
        }
        let mut emitted=false;
        assert!(matches!(map_probe_projection_failure171("permission_hook_equivalence_unavailable",
            None,Some(ProjectionGuard::G001),&policy,|_|emitted=true),WorkerRefusal::PermissionsChanged));
        assert!(!emitted);
        // Another error still uses the unchanged closed-code path, without witness fields.
        let mut note=None;
        let other=map_probe_projection_failure171("unknown SECRET",Some((&batch,"c001")),
            None,&policy,|v|note=Some(v.to_owned()));
        assert!(matches!(other,WorkerRefusal::PermissionsChanged));
        let note=note.unwrap();assert!(!note.contains("SECRET"));
        let value:serde_json::Value=serde_json::from_str(&note).unwrap();
        assert_eq!(value["code"],"unknown");assert!(value.get("witness").is_none());
    }

    #[test]
    fn native171_probe_permission_observation_closed_correlated_bounded_same_verdict() {
        let correlation = "9c9608cf-5f4c-41d5-a6af-1e0a3f0c012c";
        let admission = "a-7e51301c7e8dcfd3718419b98c7523a7";
        let batch = ProbePermissionDiagnostics::new(correlation, admission);
        let sites = [ProbePermissionSite::Projection, ProbePermissionSite::ScopedConstructor,
            ProbePermissionSite::Refresh, ProbePermissionSite::Recheck, ProbePermissionSite::BatchFinish];
        let codes = ["permission_hook_profile_unavailable", "permission_not_inherited",
            "settings_revision_changed", "permission_hook_dependency_changed", "permission_hook_profile_timeout"];
        let mut notes = Vec::new();
        for (index, site) in sites.into_iter().enumerate() {
            let candidate = format!("c{:03}", index + 1);
            let refusal = map_probe_permission_failure(codes[index],
                Some((&batch, &candidate)), site, |note| notes.push(note.to_owned()));
            assert!(matches!(refusal, WorkerRefusal::PermissionsChanged));
            let value: serde_json::Value = serde_json::from_str(notes.last().unwrap()).unwrap();
            assert_eq!(value.as_object().unwrap().len(), 6);
            assert_eq!(value["event"], "worker_probe_permission_refused");
            assert_eq!(value["correlation_id"], correlation);
            assert_eq!(value["admission_id"], admission);
            assert_eq!(value["candidate_id"], candidate);
            assert_eq!(value["site"], site.as_str());
            assert_eq!(value["code"], codes[index]);
            assert!(notes.last().unwrap().len() <= MAX_PROBE_PERMISSION_NOTE_BYTES);
        }
        for opaque in ["permission_source_unavailable SECRET", "--token=SECRET\nSTACK /private/settings",
            "permission_hook_dependency_changed\n{\"env\":\"SECRET\"}", &"SECRET".repeat(100_000)] {
            let refusal = map_probe_permission_failure(opaque, Some((&batch, "c001")),
                ProbePermissionSite::Projection, |note| notes.push(note.to_owned()));
            assert!(matches!(refusal, WorkerRefusal::PermissionsChanged));
            let note = notes.last().unwrap();
            assert!(!note.contains("SECRET") && !note.contains("STACK") && !note.contains("--token") && !note.contains("env"));
            assert_eq!(serde_json::from_str::<serde_json::Value>(note).unwrap()["code"], "unknown");
        }
        for (bad_correlation, bad_admission, bad_candidate) in [
            ("bad", admission, "c001"), (correlation, "a-not-hex", "c001"),
            (correlation, admission, "c01"), (correlation, admission, "c129"),
            ("9c9608cf-5f4c-41d5-a6af-1e0a3f0c012x", admission, "c001"),
            (correlation, "a-gggggggggggggggggggggggggggggggg", "c001"),
        ] {
            let invalid = ProbePermissionDiagnostics::new(bad_correlation, bad_admission);
            assert!(invalid.note(bad_candidate, ProbePermissionSite::Recheck, "settings_revision_changed").is_none());
        }
        let cap = ProbePermissionDiagnostics::new(correlation, admission);
        for index in 0..MAX_PROBE_PERMISSION_NOTES {
            assert!(cap.note(&format!("c{:03}", index + 1), ProbePermissionSite::BatchFinish,
                "permission_hook_equivalence_unavailable").is_some());
        }
        assert!(cap.note("c001", ProbePermissionSite::BatchFinish, "settings_revision_changed").is_none());
        let old_count = notes.len();
        assert!(matches!(map_probe_permission_failure("opaque", None, ProbePermissionSite::Projection,
            |note| notes.push(note.to_owned())), WorkerRefusal::PermissionsChanged));
        assert_eq!(notes.len(), old_count);
        let success: Result<u8, String> = Ok(7);
        assert_eq!(success.map_err(|code| map_probe_permission_failure(&code, Some((&batch, "c001")),
            ProbePermissionSite::Projection, |note| notes.push(note.to_owned()))).unwrap(), 7);
        assert_eq!(notes.len(), old_count);
    }

    #[test]
    fn incident171_startup_outcome_survives_terminal_and_reopen_without_raw_message() {
        use bridget_transport::protocol::{WorkerStartupFailure as Failure, WorkerObservation};
        let fixture = Fixture::new();
        let store = fixture.open(); reserve(&store);
        let before = store.command(&command()).unwrap();
        store.record_startup_outcome(&command(), Some(Failure::CommandMissing), None).unwrap();
        store.record_startup_outcome(&command(), Some(Failure::ExitedBeforeReady), Some(127)).unwrap();
        let after = store.command(&command()).unwrap();
        assert_eq!(after.phase, before.phase);
        assert_eq!(after.native_ready, before.native_ready);
        assert_eq!(after.startup_failed, before.startup_failed);
        drop(store);
        let store = fixture.open();
        let receipt = store.command(&command()).unwrap();
        assert_eq!(receipt.startup_failure, Some(Failure::CommandMissing));
        assert_eq!(receipt.terminal_exit_code, Some(127));
        let observation = WorkerObservation { startup_failure: receipt.startup_failure,
            phase: WorkerCommandPhase::Terminal, native_ready:false, exit_code:receipt.terminal_exit_code,
            observed_at:104, workspace:None, permission_digest:None };
        let encoded=serde_json::to_value(&observation).unwrap();
        let decoded:WorkerObservation=serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded,observation);
        let mut foreign=encoded.clone();foreign["startup_failure"]=serde_json::json!("SECRET arbitrary stderr");
        assert!(serde_json::from_value::<WorkerObservation>(foreign).is_err());
        let mut legacy=encoded;legacy.as_object_mut().unwrap().remove("startup_failure");
        assert!(serde_json::from_value::<WorkerObservation>(legacy).unwrap().startup_failure.is_none());
    }


    #[test]
    fn spec172_worker_reconnects_after_probe_eof_before_sending_credential() {
        use std::io::BufRead;
        use std::os::unix::net::UnixListener;
        let fixture = Fixture::new();
        let runtime = runtime_fixture(&fixture);
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        let listener = UnixListener::bind(&config.master_socket).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config.master_socket,fs::Permissions::from_mode(0o600)).unwrap();
        listener.set_nonblocking(true).unwrap();
        drop(runtime);
        let running = std::thread::spawn(move || run_worker(config));
        fn accept(listener: &UnixListener, running: &std::thread::JoinHandle<Result<(),String>>) -> Option<UnixStream> {
            let until = Instant::now()+Duration::from_secs(4);
            loop {
                match listener.accept() {
                    Ok((stream,_)) => return Some(stream),
                    Err(error) if error.kind()==std::io::ErrorKind::WouldBlock => {},
                    Err(error) => panic!("accept: {error}"),
                }
                if running.is_finished() || Instant::now()>=until { return None; }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let Some(first) = accept(&listener,&running) else {
            panic!("pas de première connexion : {:?}",running.join().unwrap());
        };
        first.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut reader = BufReader::new(first);
        let mut line = String::new();reader.read_line(&mut line).unwrap();
        assert!(matches!(serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),WrapperToDaemon::DaemonIdentityRequest));
        drop(reader); // Coupure pendant le redémarrage du maître, avant capacité/credential.
        let Some(mut second) = accept(&listener,&running) else {
            panic!("le worker est sorti au lieu de se reconnecter : {:?}",running.join().unwrap());
        };
        second.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut reader = BufReader::new(second.try_clone().unwrap());
        line.clear();reader.read_line(&mut line).unwrap();
        assert!(matches!(serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),WrapperToDaemon::DaemonIdentityRequest));
        let report = DaemonToWrapper::DaemonIdentityReport {host:"host".into(),db_path:"db".into(),instance_id:"daemon".into(),features:vec!["remote_worker_v1".into()]};
        writeln!(second,"{}",serde_json::to_string(&report).unwrap()).unwrap();
        line.clear();reader.read_line(&mut line).unwrap();
        assert!(matches!(serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),WrapperToDaemon::WorkerHello{..}));
        // Un refus de protocole permanent reste fatal, sans boucle ni downgrade.
        writeln!(second,"{}",serde_json::to_string(&DaemonToWrapper::Nack{id:"stop-test".into(),reason:"closed".into()}).unwrap()).unwrap();
        assert_eq!(running.join().unwrap(),Err("worker_handshake_invalid".into()));
    }

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("bridget-worker162-{}", uuid::Uuid::new_v4()));
            create_private_dir(&path).unwrap();
            Self(path)
        }
        fn root(&self) -> PathBuf {
            self.0.join("worker")
        }
        fn open(&self) -> WorkerStore {
            WorkerStore::open(&self.root(), worker()).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn worker() -> WorkerIdentity {
        WorkerIdentity {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
        }
    }
    fn command() -> WorkerCommandIdentity {
        WorkerCommandIdentity {
            command_id: "command-162".into(),
            task_id: "task-162".into(),
            instance_id: "instance-162".into(),
            generation: 1,
        }
    }
    fn placement() -> RemotePlacement {
        RemotePlacement {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
            project_id: "project-162".into(),
            mapping_revision: 1,
            origin_host: "mac".into(),
            origin_cwd: "/project".into(),
            source_root_identity: "repo".into(),
            relative_cwd: ".".into(),
            execution_cwd: "/project".into(),
            base_commit: None,
            definition_digest: "a".repeat(64),
            permission_digest: "b".repeat(64),
        }
    }
    fn slot(index: usize) -> WorkerResultSlot {
        WorkerResultSlot {
            identity: WorkerResultIdentity {
                command: command(),
                mission_id: format!("mission-{index}"),
                round: 1,
                execution_id: format!("execution-{index}"),
                execution_generation: 1,
            },
            admission_digest: "c".repeat(64),
        }
    }
    fn reserve(store: &WorkerStore) {
        assert!(matches!(
            store
                .reserve_command(&command(), &"d".repeat(64), &placement(), 1000, 100)
                .unwrap(),
            CommandReservation::Created(_)
        ));
    }
    fn result(slot: &WorkerResultSlot, body: &str) -> RemoteResultEnvelope {
        let mut result = RemoteResultEnvelope {
            worker: worker(),
            identity: slot.identity.clone(),
            message_id: "message-162".into(),
            body: body.into(),
            digest: String::new(),
        };
        result.digest = result.canonical_digest().unwrap();
        result
    }

    #[test]
    fn worker162_rejeu_reouvert_ne_recree_jamais_commande() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        drop(store);
        let store = fixture.open();
        assert!(matches!(
            store
                .reserve_command(&command(), &"d".repeat(64), &placement(), 1000, 101)
                .unwrap(),
            CommandReservation::Existing(WorkerCommandReceipt {
                phase: WorkerCommandPhase::Reserved,
                ..
            })
        ));
        assert!(matches!(
            store.reserve_command(&command(), &"e".repeat(64), &placement(), 1000, 101),
            Err(WorkerStoreError::Conflict)
        ));
        store
            .set_phase(&command(), WorkerCommandPhase::Unknown, 102)
            .unwrap();
        assert!(matches!(
            store.set_phase(&command(), WorkerCommandPhase::Released, 103),
            Err(WorkerStoreError::InvalidTransition)
        ));
        assert!(store.reserve_result_slot(&slot(1), 103).is_err());
    }

    #[test]
    fn worker162_verrou_et_enrollment_ne_masquent_pas_disparition() {
        let fixture = Fixture::new();
        let store = fixture.open();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Locked)
        ));
        drop(store);
        fs::remove_dir_all(fixture.root()).unwrap();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
        assert!(!fixture.root().exists());
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
    }

    #[test]
    fn worker162_store_preexistant_corrompu_ou_non_prive_refuse() {
        let fixture = Fixture::new();
        create_private_dir(&fixture.root()).unwrap();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let path = store.command_path(&command());
        fs::write(&path, b"{invalid").unwrap();
        drop(store);
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
        let fixture = Fixture::new();
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
    }

    #[test]
    fn worker172_workspace_refusals_preserve_cause_without_private_details() {
        for (error, expected) in [
            ("workspace_disk_full: fatal: private/path secret-token", WorkerRefusal::DiskFull),
            ("workspace_access_denied: private/path secret-token", WorkerRefusal::FilesystemAccessDenied),
            ("workspace_create_failed: private/path secret-token", WorkerRefusal::WorkspacePrepareFailed),
            ("workspace_create_failed: workspace_disk_full: private/path", WorkerRefusal::WorkspacePrepareFailed),
        ] {
            let refusal = refuse(workspace_failure(error.into()));
            assert_eq!(refusal, expected);
            let encoded = serde_json::to_string(&refusal).unwrap();
            assert!(!encoded.contains("private") && !encoded.contains("secret"));
            assert_eq!(serde_json::from_str::<WorkerRefusal>(&encoded).unwrap(), expected);
        }
        for errno in [libc::ENOSPC, libc::EDQUOT] {
            assert_eq!(refuse(io::Error::from_raw_os_error(errno).into()), WorkerRefusal::DiskFull);
        }
        for errno in [libc::EACCES, libc::EPERM, libc::EROFS] {
            assert_eq!(refuse(io::Error::from_raw_os_error(errno).into()), WorkerRefusal::FilesystemAccessDenied);
        }
        assert_eq!(refuse(io::Error::other("private secret").into()), WorkerRefusal::StoreUnavailable);
        assert_eq!(refuse(WorkerStoreError::Workspace(WorkerRefusal::WorkspacePrepareIncomplete)), WorkerRefusal::WorkspacePrepareIncomplete);
        assert_eq!(refuse(WorkerStoreError::Quota), WorkerRefusal::ResourceLimit);
        assert_eq!(refuse(WorkerStoreError::Quarantined), WorkerRefusal::StoreUnavailable);
    }

    #[test]
    fn worker172_release_apres_attestation_lente_garde_revision_et_expiration() {
        for (now, mapping_revision, accepted) in [(112, 1, true), (162, 1, false), (112, 2, false)] {
            let fixture = Fixture::new();
            let store = fixture.open();
            reserve(&store);
            store.mark_command(&command(), &ManagedMarker {
                pgid:42, birth:7, instance_id:command().instance_id,
                command_id:command().command_id, generation:command().generation,
            }, 101).unwrap();
            let permit = WorkerReleasePermit {
                permit_id:"slow-proof172".into(), command_digest:"d".repeat(64),
                permission_revision:1, mapping_revision, provisioning_revision:1,
                issued_at:102, expires_at:102 + WORKER_RELEASE_LEASE_SECS,
            };
            let result = store.authorize_release(&command(), &permit, "epoch172", now);
            assert_eq!(result.is_ok(), accepted, "{now}/{mapping_revision}: {result:?}");
            assert_eq!(store.command(&command()).unwrap().phase,
                if accepted { WorkerCommandPhase::ReleaseAuthorized } else { WorkerCommandPhase::Marked });
            if accepted {
                assert!(store.authorize_release(&command(), &permit, "other-epoch", now).is_err());
            }
        }
    }

    #[test]
    fn worker162_marqueur_et_permis_precedent_release() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        assert!(
            store
                .set_phase(&command(), WorkerCommandPhase::Released, 101)
                .is_err()
        );
        let mut marker = ManagedMarker {
            pgid: 42,
            birth: 7,
            instance_id: command().instance_id,
            command_id: command().command_id,
            generation: 1,
        };
        marker.generation = 2;
        assert!(store.mark_command(&command(), &marker, 101).is_err());
        marker.generation = 1;
        store.mark_command(&command(), &marker, 101).unwrap();
        let permit = WorkerReleasePermit {
            permit_id: "permit".into(),
            command_digest: "d".repeat(64),
            permission_revision: 1,
            mapping_revision: 1,
            provisioning_revision: 1,
            issued_at: 102,
            expires_at: 107,
        };
        assert!(
            store
                .authorize_release(&command(), &permit, "epoch-1", 107)
                .is_err()
        );
        store
            .authorize_release(&command(), &permit, "epoch-1", 103)
            .unwrap();
        assert!(
            store
                .authorize_release(&command(), &permit, "epoch-2", 103)
                .is_err()
        );
        assert_eq!(
            store.command(&command()).unwrap().phase,
            WorkerCommandPhase::ReleaseAuthorized
        );
        store
            .set_phase(&command(), WorkerCommandPhase::Released, 104)
            .unwrap();
        assert_eq!(store.command(&command()).unwrap().marker, Some(marker));
        assert!(!store.command(&command()).unwrap().native_ready);
        store.native_status(&command(), false).unwrap();
        // Même description de fichier qu’un descripteur hérité entre fork et exec.
        // CLOEXEC ne libère pas le verrou avant exec ; le propriétaire doit le faire.
        use std::os::fd::{FromRawFd, OwnedFd};
        let inherited_fd=unsafe { libc::fcntl(store._lock.as_raw_fd(),libc::F_DUPFD_CLOEXEC,0) };
        assert!(inherited_fd>=0);
        let inherited=unsafe { OwnedFd::from_raw_fd(inherited_fd) };
        drop(store);
        let store = fixture.open();
        drop(inherited);
        assert!(store.command(&command()).unwrap().native_ready);
        store.native_status(&command(), true).unwrap();
        store.native_status(&command(), false).unwrap();
        let receipt = store.command(&command()).unwrap();
        assert!(receipt.startup_failed && !receipt.native_ready);

        let failed_fixture = Fixture::new();
        let failed_store = failed_fixture.open();
        reserve(&failed_store);
        failed_store
            .mark_command(&command(), &receipt.marker.unwrap(), 101)
            .unwrap();
        failed_store
            .authorize_release(&command(), &permit, "epoch-1", 103)
            .unwrap();
        assert!(matches!(
            failed_store.native_status(&command(), false),
            Err(WorkerStoreError::InvalidTransition)
        ));
        failed_store.native_status(&command(), true).unwrap();
        drop(failed_store);
        let failed_store = failed_fixture.open();
        let failed_receipt = failed_store.command(&command()).unwrap();
        assert_eq!(failed_receipt.phase, WorkerCommandPhase::ReleaseAuthorized);
        assert!(failed_receipt.startup_failed && !failed_receipt.native_ready);
        assert!(matches!(
            failed_store.native_status(&command(), false),
            Err(WorkerStoreError::InvalidTransition)
        ));
    }

    #[test]
    fn worker162_blob_immutable_et_relais_apres_reouverture() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let slot = slot(1);
        let reservation = store.reserve_result_slot(&slot, 101).unwrap();
        let metadata =
            fs::read(store.slot_dir(&slot.identity).unwrap().join("receipt.json")).unwrap();
        let result = result(&slot, "résultat exact");
        publish_result_blob(&reservation, &result).unwrap();
        publish_result_blob(&reservation, &result).unwrap();
        assert_eq!(
            metadata,
            fs::read(store.slot_dir(&slot.identity).unwrap().join("receipt.json")).unwrap()
        );
        let resolved = find_result_reservation(
            &fixture.root(),
            &worker(),
            &command(),
            &slot.identity.mission_id,
            &slot.identity.execution_id,
            slot.identity.execution_generation,
        )
        .unwrap();
        assert_eq!(resolved, reservation);
        drop(store);
        let store = fixture.open();
        assert_eq!(
            store.read_result(&slot.identity).unwrap(),
            Some(result.clone())
        );
        let mut divergent = result.clone();
        divergent.body = "autre résultat".into();
        divergent.digest = divergent.canonical_digest().unwrap();
        assert!(matches!(
            publish_result_blob(&reservation, &divergent),
            Err(WorkerStoreError::Conflict)
        ));
        assert!(matches!(
            store.read_result(&slot.identity),
            Err(WorkerStoreError::Quarantined)
        ));
        assert_eq!(
            read_optional::<RemoteResultEnvelope>(
                &reservation.blob_path,
                MAX_WORKER_RESULT_ENVELOPE_BYTES
            )
            .unwrap(),
            Some(result)
        );
        drop(store);
        assert!(matches!(
            WorkerStore::open(&fixture.root(), worker()),
            Err(WorkerStoreError::Quarantined)
        ));
    }

    #[test]
    fn spec172_worker_store_conserve_replays_sans_reserver_les_recus_termines() {
        let fixture = Fixture::new();
        let store = fixture.open();
        let mut first = slot(0);
        first.identity.command.command_id = "completed-0".into();
        first.identity.command.task_id = "completed-task-0".into();
        let expected_result = result(&first, "résultat conservé");
        let mut first_receipt = Vec::new();
        for index in 0..120 {
            let mut slot = slot(index);
            slot.identity.command.command_id = format!("completed-{index}");
            slot.identity.command.task_id = format!("completed-task-{index}");
            let command = &slot.identity.command;
            store.reserve_command(command, &"d".repeat(64), &placement(), 1000, 100)
                .unwrap();
            let reservation = store.reserve_result_slot(&slot, 101)
                .unwrap_or_else(|error| panic!("ancien reçu {index}: {error}"));
            let result = result(&slot, "résultat conservé");
            publish_result_blob(&reservation, &result).unwrap();
            store.record_disposition(&WorkerResultDisposition {
                identity: slot.identity.clone(),
                result_digest: result.digest,
                disposition: RemoteResultDisposition::Accepted {},
            }, 102).unwrap();
            store.mark_result_producer_terminal(&slot.identity).unwrap();
            store.close_result_slot(&slot, WorkerSlotClosureProof::Cancelled, 103).unwrap();
            store.set_phase(command, WorkerCommandPhase::Terminal, 104).unwrap();
            if index == 0 {
                first_receipt = fs::read(store.slot_dir(&slot.identity).unwrap().join("receipt.json")).unwrap();
            }
        }
        // Les anciens résultats et leurs reçus restent présents, byte pour byte.
        assert_eq!(store.read_result(&first.identity).unwrap(), Some(expected_result.clone()));
        assert_eq!(first_receipt, fs::read(store.slot_dir(&first.identity).unwrap().join("receipt.json")).unwrap());
        assert_eq!(store.inventory().unwrap().3, 240);
        drop(store);
        let store = fixture.open();
        assert_eq!(store.read_result(&first.identity).unwrap(), Some(expected_result));
        assert!(matches!(
            store.reserve_command(&first.identity.command, &"d".repeat(64), &placement(), 1000, 100).unwrap(),
            CommandReservation::Existing(_)
        ));
        // Les résultats vivants continuent de réserver leur enveloppe complète.
        reserve(&store);
        for index in 0..7 {
            store.reserve_result_slot(&slot(index), 101).unwrap();
        }
        assert!(matches!(store.reserve_result_slot(&slot(7), 101), Err(WorkerStoreError::Quota)));
    }

    #[test]
    fn spec172_worker_store_quota_distincte_dune_panne_de_stockage() {
        assert_eq!(refuse(WorkerStoreError::Quota), WorkerRefusal::ResourceLimit);
        assert_eq!(refuse(WorkerStoreError::Quarantined), WorkerRefusal::StoreUnavailable);
        assert_eq!(refuse(WorkerStoreError::Io(io::Error::other("détail privé"))),
            WorkerRefusal::StoreUnavailable);
    }

    #[test]
    fn worker162_budget_reserve_sept_slots_et_compte_orphelins() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        for index in 0..7 {
            store.reserve_result_slot(&slot(index), 101).unwrap();
        }
        assert!(matches!(
            store.reserve_result_slot(&slot(7), 101),
            Err(WorkerStoreError::Quota)
        ));
        // Un fichier temporaire compte même hors d'une entrée métier.
        write_private_file_atomic(&store.root.join(".orphan.tmp"), &vec![0; 2 * 1024 * 1024])
            .unwrap();
        assert!(matches!(
            store.reserve_result_slot(&slot(8), 101),
            Err(WorkerStoreError::Quota)
        ));
        assert!(store.root.join(".orphan.tmp").exists());
    }

    #[test]
    fn worker162_publication_compte_json_et_refuse_symlink_et_temporaire() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let first = slot(1);
        let reservation = store.reserve_result_slot(&first, 101).unwrap();
        let control_body = "\0".repeat(262_144);
        let escaped = result(&first, &control_body);
        assert!(serde_json::to_vec(&escaped).unwrap().len() > control_body.len() * 5);
        publish_result_blob(&reservation, &escaped).unwrap();
        let second = slot(2);
        let reservation = store.reserve_result_slot(&second, 101).unwrap();
        let payload = reservation.blob_path.parent().unwrap();
        write_private_file_atomic(&payload.join(".blob.json.orphan.tmp"), b"incomplete").unwrap();
        assert!(matches!(
            publish_result_blob(&reservation, &result(&second, "result")),
            Err(WorkerStoreError::Quarantined)
        ));
        let third = slot(3);
        let reservation = store.reserve_result_slot(&third, 101).unwrap();
        symlink(&fixture.0.join("outside"), &reservation.blob_path).unwrap();
        assert!(matches!(
            publish_result_blob(&reservation, &result(&third, "result")),
            Err(WorkerStoreError::Quarantined)
        ));
        assert!(!fixture.0.join("outside").exists());
    }

    #[test]
    fn worker162_collecte_exige_disposition_producteur_et_cloture() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let slot = slot(1);
        let reservation = runtime.store.reserve_result_slot(&slot, 101).unwrap();
        let result = result(&slot, "result");
        publish_result_blob(&reservation, &result).unwrap();
        let mut disposition = WorkerResultDisposition {
            identity: slot.identity.clone(),
            result_digest: result.digest.clone(),
            disposition: RemoteResultDisposition::Deferred {
                reason: RemoteResultDeferredReason::QuotaHold,
                retry_after: 120,
            },
        };
        runtime.store.record_disposition(&disposition, 110).unwrap();
        assert!(runtime.store.pending_results(119).unwrap().is_empty());
        assert_eq!(runtime.store.pending_results(120).unwrap().len(), 1);
        let empty = self::slot(2);
        runtime.store.reserve_result_slot(&empty, 111).unwrap();
        let reserved_before = runtime.store.inventory().unwrap().2;
        // Simule le crash après la preuve terminale, avant la clôture des slots.
        runtime
            .store
            .set_phase(&command(), WorkerCommandPhase::Terminal, 111)
            .unwrap();
        runtime.reconcile(&command()).unwrap();
        let closed = runtime.store.read_slot(&empty.identity).unwrap().unwrap();
        assert!(closed.producer_terminal);
        assert_eq!(closed.closed, Some(WorkerSlotClosureProof::Cancelled));
        assert!(
            reserved_before - runtime.store.inventory().unwrap().2
                >= (MAX_WORKER_RESULT_ENVELOPE_BYTES - MAX_METADATA_BYTES) as u64
        );
        // La reprise est idempotente : elle ne repousse pas les sept jours.
        runtime.reconcile(&command()).unwrap();
        assert_eq!(
            runtime
                .store
                .read_slot(&empty.identity)
                .unwrap()
                .unwrap()
                .finalized_at,
            closed.finalized_at
        );
        assert_eq!(
            runtime.store.pending_results(120).unwrap(),
            vec![result.clone()]
        );
        let mut next = command();
        next.command_id = "next-command162".into();
        next.instance_id = "next-instance162".into();
        runtime
            .store
            .reserve_command(&next, &"d".repeat(64), &placement(), 1000, 112)
            .unwrap();
        let mut next_slot = self::slot(3);
        next_slot.identity.command = next;
        runtime.store.reserve_result_slot(&next_slot, 112).unwrap();
        runtime
            .store
            .set_phase(
                &next_slot.identity.command,
                WorkerCommandPhase::Refused,
                113,
            )
            .unwrap();
        runtime.reconcile(&next_slot.identity.command).unwrap();
        assert_eq!(
            runtime
                .store
                .command(&next_slot.identity.command)
                .unwrap()
                .phase,
            WorkerCommandPhase::Refused
        );
        assert!(
            runtime
                .store
                .read_slot(&next_slot.identity)
                .unwrap()
                .unwrap()
                .closed
                .is_some()
        );
        assert_eq!(runtime.store.collect(200).unwrap(), 0);
        assert!(reservation.blob_path.exists());
        disposition.disposition = RemoteResultDisposition::Accepted {};
        runtime.store.record_disposition(&disposition, 112).unwrap();
        assert!(runtime.store.pending_results(120).unwrap().is_empty());
        assert_eq!(runtime.store.collect(200).unwrap(), 1);
        assert!(
            runtime
                .store
                .slot_dir(&slot.identity)
                .unwrap()
                .join("receipt.json")
                .exists()
        );
        runtime.store.record_disposition(&disposition, 201).unwrap(); // ACK perdu : compact rejouable.
        runtime
            .store
            .collect(112 + WORKER_RECEIPT_RETENTION_SECS)
            .unwrap();
        assert!(!runtime.store.slot_dir(&slot.identity).unwrap().exists());
        runtime
            .store
            .set_phase(&command(), WorkerCommandPhase::Terminal, 200)
            .unwrap();
        runtime
            .store
            .collect(worker_now() + 1000 + WORKER_RECEIPT_RETENTION_SECS)
            .unwrap();
        assert!(matches!(
            runtime.store.command(&command()),
            Err(WorkerStoreError::Unknown)
        ));
        assert!(
            runtime
                .store
                .reserve_command(&command(), &"d".repeat(64), &placement(), 1000, 1001)
                .is_err()
        );
    }

    #[test]
    fn worker162_slot_caduc_ne_libere_que_sur_preuve_sans_arret_wrapper() {
        let fixture = Fixture::new();
        let store = fixture.open();
        reserve(&store);
        let slot = slot(1);
        let reservation = store.reserve_result_slot(&slot, 101).unwrap();
        store
            .close_result_slot(&slot, WorkerSlotClosureProof::NeverDispatched, 102)
            .unwrap();
        assert!(publish_result_blob(&reservation, &result(&slot, "late")).is_err());
        assert!(store.reserve_result_slot(&slot, 103).is_err());
        assert_eq!(
            store.command(&command()).unwrap().phase,
            WorkerCommandPhase::Reserved
        );
        assert_eq!(store.collect(200).unwrap(), 0);
    }

    #[test]
    fn worker174_collect_distingue_terminal_actif_et_reference() {
        let fixture = Fixture::new();
        let store = fixture.open();
        let terminal = command_174("terminal");
        let neighbour = command_174("terminal-voisin");
        let active = command_174("active");
        let kept_by_slot = command_174("kept");
        for identity in [&terminal, &neighbour, &active, &kept_by_slot] {
            reserve_174(&store, identity);
        }
        let pending = slot_174(&neighbour);
        store.reserve_result_slot(&pending, 100).unwrap();
        let kept_slot = slot_174(&kept_by_slot);
        store.reserve_result_slot(&kept_slot, 100).unwrap();
        store.set_phase(&terminal, WorkerCommandPhase::Terminal, 100).unwrap();
        store.set_phase(&kept_by_slot, WorkerCommandPhase::Terminal, 100).unwrap();
        assert_eq!(store.collect(101 + WORKER_RECEIPT_RETENTION_SECS).unwrap(), 0);
        assert!(matches!(store.command(&terminal), Err(WorkerStoreError::Unknown)));
        assert!(store.command(&neighbour).is_ok());
        assert!(store.command(&active).is_ok());
        assert!(store.command(&kept_by_slot).is_ok());
        assert!(store.read_slot(&pending.identity).unwrap().is_some());
        assert!(store.read_slot(&kept_slot.identity).unwrap().is_some());
    }

    fn command_174(id: &str) -> WorkerCommandIdentity {
        WorkerCommandIdentity {
            command_id: id.into(),
            task_id: "task-174".into(),
            instance_id: "instance-174".into(),
            generation: 1,
        }
    }
    fn reserve_174(store: &WorkerStore, identity: &WorkerCommandIdentity) {
        store
            .reserve_command(identity, &"d".repeat(64), &placement(), 101, 100)
            .unwrap();
    }
    fn slot_174(command: &WorkerCommandIdentity) -> WorkerResultSlot {
        WorkerResultSlot {
            identity: WorkerResultIdentity {
                command: command.clone(),
                mission_id: "mission-174".into(),
                round: 1,
                execution_id: "execution-174".into(),
                execution_generation: 1,
            },
            admission_digest: "c".repeat(64),
        }
    }

    fn runtime_fixture(fixture: &Fixture) -> WorkerRuntime {
        let state = fixture.0.join("runtime");
        private_dir(&state).unwrap();
        let network = fixture.0.join("network");
        private_dir(&network).unwrap();
        let credential = fixture.0.join("credential");
        write_private_file_atomic(&credential, b"worker162-test-secret").unwrap();
        let registry = fixture.0.join("registry.json");
        write_json(&registry, &serde_json::json!({"agents":{},"machines":{"cartae":{
            "enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
            "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],"mappings":{}
        }}})).unwrap();
        WorkerRuntime::new(WorkerRuntimeConfig {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
            credential_path: credential,
            state_root: state,
            registry_path: registry,
            master_socket: network.join("s"),
        })
        .unwrap()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn incident171_cancel_after_exit_requires_matching_drain_and_closes_slots() {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        reserve(&runtime.store);
        let result_slot = slot(1);
        runtime.store.reserve_result_slot(&result_slot, 101).unwrap();
        let mut child = Command::new("/bin/cat")
            .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null())
            .process_group(0).spawn().unwrap();
        let marker = ManagedMarker {
            pgid: child.id(),
            birth: crate::managed_process::process_birth(child.id()).unwrap(),
            instance_id: command().instance_id,
            command_id: command().command_id,
            generation: command().generation,
        };
        runtime.store.mark_command(&command(), &marker, 101).unwrap();
        write_json(&runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command()))), &marker).unwrap();
        let permit = WorkerReleasePermit {
            permit_id: "cancel-after-exit171".into(), command_digest: "d".repeat(64),
            permission_revision: 1, mapping_revision: 1, provisioning_revision: 1,
            issued_at: 101, expires_at: 106,
        };
        runtime.store.authorize_release(&command(), &permit, "epoch", 101).unwrap();
        drop(child.stdin.take());
        assert!(child.wait().unwrap().success());
        assert!(!crate::managed_process::group_exists(marker.pgid).unwrap());
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Unknown));
        assert!(!runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().producer_terminal);
        let receipt = runtime.store.command(&command()).unwrap();
        let path = drain_proof_path(&runtime.store.root, &command());
        let mut proof = WorkerDrainProof::expected(&receipt).unwrap();
        proof.birth += 1;
        write_json(&path, &proof).unwrap();
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Unknown));
        proof.birth -= 1;
        write_json(&path, &proof).unwrap();
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Terminal));
        let closed = runtime.store.read_slot(&result_slot.identity).unwrap().unwrap();
        assert!(closed.producer_terminal);
        assert_eq!(closed.closed, Some(WorkerSlotClosureProof::Cancelled));
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation }
            if observation.phase == WorkerCommandPhase::Terminal));
        assert_eq!(runtime.store.read_slot(&result_slot.identity).unwrap().unwrap().finalized_at, closed.finalized_at);
        let mut foreign = command(); foreign.generation += 1;
        assert_eq!(runtime.cancel(&foreign).unwrap_err(), WorkerRefusal::CommandMismatch);
    }

    #[test]
    fn worker162_runtime_rejeu_reserved_reste_unknown_sans_bootstrap() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let definition = runtime
            .registry()
            .unwrap()
            .resolved_definition("codex")
            .unwrap();
        let mut placement = placement();
        placement.definition_digest = definition.digest.clone();
        let mut start = WorkerStart {
            prepared_token: "token-perdu".into(),
            child_id: "child162".into(),
            mission_id: "mission162".into(),
            forward_child_requests: false,
            placement,
            definition,
            child_policy: serde_json::json!({}),
            effective_posture: bridget_transport::protocol::SpawnPosture::Development,
            deadline_at: worker_now() + 300,
        };
        let digest = worker_command_digest(&worker(), &command(), &start).unwrap();
        runtime
            .store
            .reserve_command(
                &command(),
                &digest,
                &start.placement,
                start.deadline_at,
                worker_now(),
            )
            .unwrap();
        assert!(matches!(
            runtime.reconcile(&command()).unwrap(),
            WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase: WorkerCommandPhase::Unknown,
                    ..
                }
            }
        ));
        let disconnected = AtomicBool::new(false);
        assert!(matches!(
            runtime.start(&command(), &start, &disconnected, &mut WorkerStartCapacityCause::Unknown).unwrap(),
            WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase: WorkerCommandPhase::Unknown,
                    ..
                }
            }
        ));
        assert!(
            runtime.running.is_empty() && runtime.held.is_empty() && runtime.abandoned.is_empty()
        );
        start.child_id = "autre-child".into();
        assert_eq!(
            runtime
                .start(&command(), &start, &disconnected, &mut WorkerStartCapacityCause::Unknown)
                .unwrap_err(),
            WorkerRefusal::CommandMismatch
        );
    }

    #[test]
    fn worker162_reprise_frontieres_durables_ne_reemet_ni_bootstrap_ni_release() {
        use std::os::unix::process::CommandExt;
        use std::process::{Child, Command, Stdio};
        struct EofProcess(Child);
        impl Drop for EofProcess {
            fn drop(&mut self) {
                drop(self.0.stdin.take());
                let _ = self.0.wait();
            }
        }
        // Les EOF et crash avant/après marqueur sont éprouvés par les primitives
        // managed_process. Ici, leur état durable traverse le vrai recover/start.
        for phase in [
            WorkerCommandPhase::Reserved,
            WorkerCommandPhase::Marked,
            WorkerCommandPhase::ReleaseAuthorized,
            WorkerCommandPhase::Released,
        ] {
            let fixture = Fixture::new();
            let mut runtime = runtime_fixture(&fixture);
            // Released a dépassé son délai de bootstrap, pas son mandat métier.
            let admitted_at = worker_now()
                - if phase == WorkerCommandPhase::Released {
                    400
                } else {
                    0
                };
            let definition = runtime
                .registry()
                .unwrap()
                .resolved_definition("codex")
                .unwrap();
            let mut placement = placement();
            placement.definition_digest = definition.digest.clone();
            let start = WorkerStart {
                prepared_token: "token-avant-crash".into(),
                child_id: "child162".into(),
                mission_id: "mission162".into(),
                forward_child_requests: false,
                placement,
                definition,
                child_policy: serde_json::json!({}),
                effective_posture: bridget_transport::protocol::SpawnPosture::Discovery,
                deadline_at: admitted_at + 300,
            };
            let digest = worker_command_digest(&worker(), &command(), &start).unwrap();
            runtime
                .store
                .reserve_command(
                    &command(),
                    &digest,
                    &start.placement,
                    start.deadline_at,
                    admitted_at,
                )
                .unwrap();
            // Un processus possédé reste vivant au passage création→marqueur.
            // Son EOF termine le groupe sans signal ni processus laissé derrière.
            let mut owned = EofProcess(
                Command::new("/bin/cat")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .process_group(0)
                    .spawn()
                    .unwrap(),
            );
            let process = &mut owned.0;
            let marker = ManagedMarker {
                pgid: process.id(),
                birth: crate::managed_process::process_birth(process.id()).unwrap(),
                instance_id: command().instance_id,
                command_id: command().command_id,
                generation: 1,
            };
            let permit = WorkerReleasePermit {
                permit_id: "permit-avant-crash".into(),
                command_digest: digest,
                permission_revision: 1,
                mapping_revision: 1,
                provisioning_revision: 1,
                issued_at: admitted_at,
                expires_at: admitted_at + 5,
            };
            if phase != WorkerCommandPhase::Reserved {
                runtime
                    .store
                    .mark_command(&command(), &marker, admitted_at)
                    .unwrap();
            }
            if matches!(
                phase,
                WorkerCommandPhase::ReleaseAuthorized | WorkerCommandPhase::Released
            ) {
                runtime
                    .store
                    .authorize_release(&command(), &permit, "epoch-avant", admitted_at)
                    .unwrap();
            }
            if phase == WorkerCommandPhase::Released {
                runtime
                    .store
                    .set_phase(&command(), phase, admitted_at)
                    .unwrap();
                runtime.store.native_status(&command(), false).unwrap();
            }
            let mut second = slot(2);
            second.identity.round = 2;
            let second_result = result(&second, "résultat après délai bootstrap");
            if phase == WorkerCommandPhase::Released {
                let mut registry: serde_json::Value =
                    serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap())
                        .unwrap();
                registry["machines"]["cartae"]["mappings"]["project-162"] = serde_json::json!({
                    "source_host":"mac","source_root":"/project","remote_root":"/project",
                    "expected_git_identity":"repo","mapping_revision":1
                });
                write_json(&runtime.config.registry_path, &registry).unwrap();
                let request = WorkerRequest {
                    version: REMOTE_WORKER_VERSION,
                    correlation_id: "second-slot".into(),
                    connection_epoch: "epoch".into(),
                    worker: worker(),
                    command: command(),
                    request: WorkerRequestOperation::ReserveResultSlot {
                        slot: second.clone(),
                    },
                };
                assert!(worker_now() > start.deadline_at);
                assert!(
                    matches!(runtime.handle(&request, "epoch", &AtomicBool::new(false)).unwrap(),
                    WorkerResponseOperation::ResultSlotReserved { slot, .. } if slot == second)
                );
                let reservation = runtime.store.reservation(&second).unwrap();
                publish_result_blob(&reservation, &second_result).unwrap();
                registry["machines"]["cartae"]["enabled"] = false.into();
                write_json(&runtime.config.registry_path, &registry).unwrap();
            } else {
                assert!(matches!(
                    runtime
                        .store
                        .reserve_result_slot(&second, start.deadline_at + 1),
                    Err(WorkerStoreError::InvalidTransition)
                ));
                assert!(matches!(
                    runtime.store.authorize_release(
                        &command(),
                        &permit,
                        "epoch-avant",
                        start.deadline_at + 1
                    ),
                    Err(WorkerStoreError::Invalid(WorkerRefusal::ReleaseExpired))
                ));
            }
            let config = WorkerRuntimeConfig {
                machine_id: runtime.config.machine_id.clone(),
                worker_id: runtime.config.worker_id.clone(),
                provisioning_revision: runtime.config.provisioning_revision,
                credential_path: runtime.config.credential_path.clone(),
                state_root: runtime.config.state_root.clone(),
                registry_path: runtime.config.registry_path.clone(),
                master_socket: runtime.config.master_socket.clone(),
            };
            drop(runtime);
            let mut recovered = WorkerRuntime::new(config).unwrap();
            let expected = if phase == WorkerCommandPhase::Released {
                phase
            } else {
                WorkerCommandPhase::Unknown
            };
            let observed = recovered
                .start(&command(), &start, &AtomicBool::new(false), &mut WorkerStartCapacityCause::Unknown)
                .unwrap();
            assert!(
                matches!(observed, WorkerResponseOperation::Observation { observation }
                if observation.phase == expected && observation.native_ready == (phase == WorkerCommandPhase::Released)),
                "frontière {phase:?}"
            );
            let receipt = recovered.store.command(&command()).unwrap();
            assert_eq!(receipt.phase, expected);
            assert_eq!(
                receipt.marker,
                (phase != WorkerCommandPhase::Reserved).then_some(marker.clone())
            );
            assert!(
                recovered.prepared.is_empty()
                    && recovered.held.is_empty()
                    && recovered.running.is_empty()
            );
            assert_eq!(
                recovered.release(&command(), &permit, "epoch-apres", &AtomicBool::new(false)),
                Err(WorkerRefusal::UnknownExecution)
            );
            if phase == WorkerCommandPhase::Released {
                assert!(recovered.registry().is_ok()); // Authentification conserve la reprise disabled.
                assert_eq!(
                    recovered.registry().unwrap().machine("cartae").unwrap_err(),
                    "machine_disabled"
                );
                assert_eq!(
                    recovered.store.pending_results(worker_now()).unwrap(),
                    vec![second_result.clone()]
                );
                let mut request = WorkerRequest {
                    version: REMOTE_WORKER_VERSION,
                    correlation_id: "second-replay".into(),
                    connection_epoch: "epoch".into(),
                    worker: worker(),
                    command: command(),
                    request: WorkerRequestOperation::ReserveResultSlot {
                        slot: second.clone(),
                    },
                };
                assert!(
                    matches!(recovered.handle(&request, "epoch", &AtomicBool::new(false)).unwrap(),
                    WorkerResponseOperation::ResultSlotReserved { slot, .. } if slot == second)
                );
                let mut third = slot(3);
                third.identity.round = 3;
                request.request = WorkerRequestOperation::ReserveResultSlot { slot: third };
                assert_eq!(
                    recovered.handle(&request, "epoch", &AtomicBool::new(false)),
                    Err(WorkerRefusal::PlacementMismatch)
                );
                let disposition = WorkerResultDisposition {
                    identity: second.identity.clone(),
                    result_digest: second_result.digest.clone(),
                    disposition: RemoteResultDisposition::Accepted {},
                };
                request.request = WorkerRequestOperation::ResultDisposition {
                    result: disposition.clone(),
                };
                assert_eq!(
                    recovered
                        .handle(&request, "epoch", &AtomicBool::new(false))
                        .unwrap(),
                    WorkerResponseOperation::DispositionRecorded {
                        result: disposition
                    }
                );
                assert!(
                    recovered
                        .store
                        .pending_results(worker_now())
                        .unwrap()
                        .is_empty()
                );
            }
            assert!(process.try_wait().unwrap().is_none());
            assert_eq!(
                crate::managed_process::process_birth(process.id()).unwrap(),
                marker.birth
            );
            drop(process.stdin.take());
            assert!(process.wait().unwrap().success());
            #[cfg(target_os = "linux")]
            if matches!(
                phase,
                WorkerCommandPhase::ReleaseAuthorized | WorkerCommandPhase::Released
            ) {
                // Wrapper disparu après RELEASE : ni la reprise ni l'ancien reçu
                // ne prouvent le drainage. La capacité doit rester retenue.
                let observed = recovered.reconcile(&command()).unwrap();
                assert!(
                    matches!(observed, WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Unknown)
                );
                // Même refus pour la branche abandoned après RELEASE possible.
                let mut reaped = Command::new("/bin/true").spawn().unwrap();
                reaped.wait().unwrap();
                recovered.abandoned.insert(command().command_id, reaped);
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Unknown));
                assert!(recovered.abandoned.remove(&command().command_id).is_some());
                let receipt = recovered.store.command(&command()).unwrap();
                let path = drain_proof_path(&recovered.store.root, &command());
                write_private_file_atomic(&path, b"not-json").unwrap();
                assert!(recovered.store.drainage_proven(&receipt).is_err());
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Unknown));
                let mut proof = WorkerDrainProof::expected(&receipt).unwrap();
                proof.birth += 1;
                write_json(&path, &proof).unwrap();
                assert!(!recovered.store.drainage_proven(&receipt).unwrap());
                proof.birth -= 1;
                write_json(&path, &proof).unwrap();
                assert!(recovered.store.drainage_proven(&receipt).unwrap());
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Terminal));
                fs::remove_file(path).unwrap();
                // Les reçus déjà terminaux restent historiques.
                assert!(matches!(recovered.reconcile(&command()).unwrap(),
                    WorkerResponseOperation::Observation { observation }
                    if observation.phase == WorkerCommandPhase::Terminal));
            }
        }
    }

    #[test]
    fn incident171_shared_refresh_exact_commit_preserves_work_and_active_missions() {
        use bridget_transport::protocol::CandidateProbeRefusal as R;
        let fixture = Fixture::new();
        let root = fixture.0.canonicalize().unwrap();
        let origin = root.join("origin"); fs::create_dir(&origin).unwrap();
        let git = |cwd: &Path, args: &[&str]| {
            let out = std::process::Command::new("git").current_dir(cwd)
                .env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1")
                .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgSign=false"])
                .args(args).output().unwrap();
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
            String::from_utf8(out.stdout).unwrap().trim().to_owned()
        };
        git(&origin, &["init", "-q"]);
        fs::write(origin.join("file"), "old\n").unwrap();
        git(&origin, &["add", "."]); git(&origin, &["commit", "-qm", "old"]);
        let before = git(&origin, &["rev-parse", "HEAD"]);
        let project = root.join("project");
        git(&root, &["clone", "-q", "--no-local", origin.to_str().unwrap(), project.to_str().unwrap()]);
        fs::write(origin.join("file"), "new\n").unwrap();
        git(&origin, &["commit", "-qam", "new"]);
        let expected = git(&origin, &["rev-parse", "HEAD"]);
        let runtime = runtime_fixture(&fixture);
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Ok(true));
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), before, "probe must not move HEAD");
        assert!(!project.join(".git/FETCH_HEAD").exists());
        fs::write(project.join("file"), "human work\n").unwrap();
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceDirty));
        assert!(runtime.refresh_shared_workspace(&project, &expected).is_err());
        assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "human work\n");
        fs::write(project.join("file"), "old\n").unwrap();
        fs::write(project.join("untracked"), "keep\n").unwrap();
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceDirty));
        fs::remove_file(project.join("untracked")).unwrap();
        let busy_fixture = Fixture::new(); let busy = runtime_fixture(&busy_fixture);
        let mut placement = placement(); placement.execution_cwd = project.to_string_lossy().into_owned();
        busy.store.reserve_command(&command(), &"d".repeat(64), &placement, 1000, 100).unwrap();
        assert_eq!(busy.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceBusy));
        assert!(busy.refresh_shared_workspace(&project, &expected).is_err());
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), before);
        assert_eq!(runtime.shared_refresh_allowed(&project, &"a".repeat(40)), Err(R::WorkspaceBaseUnavailable));
        let branch = git(&project, &["symbolic-ref", "HEAD"]);
        runtime.refresh_shared_workspace(&project, &expected).unwrap();
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), expected);
        assert_eq!(git(&project, &["symbolic-ref", "HEAD"]), branch);
        assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "new\n");
        assert!(!project.join(".git/FETCH_HEAD").exists());
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Ok(false));
        assert_eq!(runtime.shared_refresh_allowed(&project, &before), Err(R::WorkspaceDiverged));
        fs::write(project.join("file"), "local commit\n").unwrap();
        git(&project, &["commit", "-qam", "local"]);
        assert_eq!(runtime.shared_refresh_allowed(&project, &expected), Err(R::WorkspaceDiverged));
        assert!(runtime.refresh_shared_workspace(&project, &expected).is_err());
        assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "local commit\n");
        // Les codes distincts traversent le protocole fermé existant.
        for reason in [R::WorkspaceDirty, R::WorkspaceBusy, R::WorkspaceDiverged, R::WorkspaceBaseUnavailable] {
            assert_eq!(serde_json::from_value::<R>(serde_json::to_value(&reason).unwrap()).unwrap(), reason);
        }
    }

    #[test]
    fn spec172_linked_checkout_prepare_preserves_main() {
        use bridget_transport::protocol::{ProviderPermissions, WorkspaceRequest, CandidateProbeRefusal as R};
        for (diverged, isolated, refresh, parent_at_project_root, native_claude) in [
            (false, false, false, false, false), (true, false, false, false, false),
            (false, true, false, false, false), (true, true, false, false, false),
            (false, false, true, false, false), (false, true, false, true, false),
            (false, true, false, true, true)
        ] {
            let fixture = Fixture::new();
            let project_fixture = Fixture::new();
            let project = project_fixture.0.canonicalize().unwrap();
            let git = |cwd: &Path, args: &[&str]| {
                let out = std::process::Command::new("git").current_dir(cwd)
                    .env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1")
                    .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "-c", "commit.gpgSign=false", "-c", "core.hooksPath=/dev/null"])
                    .args(args).output().unwrap();
                assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
                String::from_utf8(out.stdout).unwrap().trim().to_owned()
            };
            git(&project, &["init", "-q"]);
            git(&project, &["config", "remote.origin.url", "fixture:linked172"]);
            fs::write(project.join(".gitignore"), ".worktrees/\n").unwrap();
            fs::write(project.join("file"), "base\n").unwrap();
            git(&project, &["add", "."]); git(&project, &["commit", "-qm", "base"]);
            let base = git(&project, &["rev-parse", "HEAD"]);
            let linked = project.join(".worktrees/linked172");
            git(&project, &["worktree", "add", "--detach", linked.to_str().unwrap(), &base]);
            fs::write(linked.join("file"), "linked\n").unwrap();
            git(&linked, &["commit", "-qam", "linked"]);
            let expected = git(&linked, &["rev-parse", "HEAD"]);
            if diverged {
                fs::write(project.join("main-only"), "main\n").unwrap();
                git(&project, &["add", "main-only"]); git(&project, &["commit", "-qm", "main"]);
            }
            let main_head = git(&project, &["rev-parse", "HEAD"]);
            if refresh {
                git(&linked, &["checkout", "--detach", &base]);
                assert_eq!(git(&linked, &["rev-parse", "HEAD"]), base);
                assert_ne!(base, expected);
            }
            let mut runtime = runtime_fixture(&fixture);
            fs::create_dir(fixture.0.join(".claude")).unwrap();
            let agent = if native_claude { "claude" } else { "codex" };
            let protocol = if native_claude { "claude_stream_json" } else { "codex_app_server" };
            let args = if native_claude { vec!["--dangerously-skip-permissions"] } else { vec!["app-server"] };
            write_json(&runtime.config.registry_path, &serde_json::json!({
                "agents":{(agent):{"command":"/bin/echo","args":args,"protocol":protocol,
                    "capabilities":{"execution_paths":[protocol],"models":{"fixture-model":{"efforts":["high"]}}}}},
                "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                    "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":[agent],
                    "mappings":{"linked172":{"source_host":"origin172","source_root":"/origin172",
                        "remote_root":project,"expected_git_identity":"fixture:linked172","mapping_revision":1}}}}
            })).unwrap();
            runtime.source.clear();
            runtime.source.insert("HOME".into(), fixture.0.as_os_str().to_owned());
            runtime.source.insert("PATH".into(), OsString::from("/usr/bin:/bin"));
            let mut identity = command();
            identity.task_id = uuid::Uuid::new_v4().to_string();
            identity.instance_id.clear();
            identity.generation = 0;
            let request = WorkerPrepare {
                machine_id:"cartae".into(),project_id:"linked172".into(),provisioning_revision:1,mapping_revision:1,
                origin_host:"origin172".into(),origin_cwd:"/origin172/.worktrees/linked172".into(),
                source_root:"/origin172".into(),remote_root:project.to_string_lossy().into_owned(),
                source_root_identity:"fixture:linked172".into(),relative_cwd:".worktrees/linked172".into(),
                base_commit:Some(expected.clone()),agent_type:agent.into(),model:"fixture-model".into(),
                effort:Some("high".into()),posture:None,workspace:Some(if isolated {
                    WorkspaceRequest::Worktree { base_ref:Some(expected.clone()), branch:None }
                } else { WorkspaceRequest::Shared {} }),
                parent:ProviderPermissions {
                    version:1,source:"native_wrapper".into(),run_id:"run172".into(),
                    provider_session_id:"session172".into(),provider_instance_id:"parent172".into(),revision:1,
                    driver:"codex_app_server".into(),cwd:if parent_at_project_root {
                        "/origin172".into()
                    } else { "/origin172/.worktrees/linked172".into() },
                    runtime_mode:"full-access".into(),interaction_mode:"default".into(),
                    provider_policy:serde_json::json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user",
                        "sandbox_policy":if native_claude { serde_json::json!({"type":"dangerFullAccess"}) }
                        else { serde_json::json!({"type":"readOnly","networkAccess":false}) }}),
                },
            };
            let registry = runtime.registry().unwrap();
            let (mapping, cwd) = runtime.mapping_for(&request, &registry, None).unwrap();
            assert_eq!(runtime.probe_workspace_preconditions(&request, &mapping, &cwd), Ok(()));
            if parent_at_project_root {
                let mut outside = request.clone();
                outside.parent.cwd = "/origin172-sibling".into();
                assert!(matches!(runtime.prepare(&identity, &outside),
                    Err(WorkerRefusal::PermissionsChanged)));
                assert!(!linked.join(".worktrees").exists());
            }
            let ready = runtime.prepare(&identity, &request).unwrap();
            if native_claude {
                assert_eq!(ready.child_policy["permission_mode"], "bypassPermissions");
            } else {
                assert_eq!(ready.child_policy["sandbox_policy"], request.parent.provider_policy["sandbox_policy"]);
            }
            if isolated {
                let record: crate::delegation::WorkspaceRecord =
                    serde_json::from_value(ready.workspace.clone().unwrap()).unwrap();
                assert_eq!(ready.placement.execution_cwd, record.path);
                assert_eq!(fs::read_to_string(Path::new(&record.path).join("file")).unwrap(), "linked\n");
            } else { assert_eq!(Path::new(&ready.placement.execution_cwd), linked); }
            runtime.recheck(runtime.prepared.get(&command().command_id).unwrap()).unwrap();
            assert_eq!(git(&project, &["rev-parse", "HEAD"]), main_head, "main must not move");
            assert_eq!(fs::read_to_string(project.join("file")).unwrap(), "base\n");
            assert_eq!(git(&linked, &["rev-parse", "HEAD"]), expected);
            if !isolated {
                let mut stale = request.clone(); stale.base_commit = Some(base);
                assert_eq!(runtime.probe_workspace_preconditions(&stale, &mapping, &cwd), Err(R::WorkspaceDiverged));
            }
            assert!(runtime.store.commands().unwrap().is_empty());
            assert!(runtime.running.is_empty() && runtime.held.is_empty());
        }
    }

    #[test]
    fn worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications() {
        use bridget_transport::protocol::{ProviderPermissions, SpawnPosture, WorkspaceRequest};
        let fixture = Fixture::new();
        let project_fixture = Fixture::new();
        let project = project_fixture.0.canonicalize().unwrap();
        let git = |cwd: &Path, args: &[&str]| {
            let output = std::process::Command::new("/usr/bin/git")
                .args(args)
                .current_dir(cwd)
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", &fixture.0)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_AUTHOR_NAME", "Fixture")
                .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
                .env("GIT_COMMITTER_NAME", "Fixture")
                .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&project, &["init", "--quiet"]);
        git(
            &project,
            &[
                "config",
                "--local",
                "remote.origin.url",
                "fixture:remote162",
            ],
        );
        fs::write(project.join(".gitignore"), b".worktrees/\n").unwrap();
        fs::write(project.join("base.txt"), b"base\n").unwrap();
        git(&project, &["add", ".gitignore", "base.txt"]);
        git(&project, &["commit", "--quiet", "-m", "fixture initiale"]);
        let base = git(&project, &["rev-parse", "HEAD"]);
        let mut runtime = runtime_fixture(&fixture);
        write_json(&runtime.config.registry_path, &serde_json::json!({
            "agents":{"codex":{"command":"/bin/echo","args":["app-server"],"protocol":"codex_app_server",
                "capabilities":{"execution_paths":["codex_app_server"],"models":{"fixture-model":{"efforts":["high"]}}}}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],
                "mappings":{"project162":{"source_host":"origin162","source_root":"/origin162",
                    "remote_root":project,"expected_git_identity":"fixture:remote162","mapping_revision":1}}}}
        })).unwrap();
        // L'origine est volontairement absente sur le worker.
        runtime.source.clear();
        runtime
            .source
            .insert("HOME".into(), fixture.0.as_os_str().to_owned());
        runtime
            .source
            .insert("PATH".into(), OsString::from("/usr/bin:/bin"));
        let mut identity = command();
        // L'isolation154 dérive son chemin depuis le UUID natif de Task.
        identity.task_id = uuid::Uuid::new_v4().to_string();
        identity.instance_id.clear();
        identity.generation = 0;
        let mut request = WorkerPrepare {
            machine_id: "cartae".into(),
            project_id: "project162".into(),
            provisioning_revision: 1,
            mapping_revision: 1,
            origin_host: "origin162".into(),
            origin_cwd: "/origin162".into(),
            source_root: "/origin162".into(),
            remote_root: project.to_string_lossy().into_owned(),
            source_root_identity: "fixture:remote162".into(),
            relative_cwd: ".".into(),
            base_commit: Some(base.clone()),
            agent_type: "codex".into(),
            model: "fixture-model".into(),
            effort: Some("high".into()),
            posture: None,
            workspace: None,
            parent: ProviderPermissions {
                version: 1,
                source: "native_wrapper".into(),
                run_id: "run162".into(),
                provider_session_id: "session162".into(),
                provider_instance_id: "parent162".into(),
                revision: 1,
                driver: "codex_app_server".into(),
                cwd: "/origin162".into(),
                runtime_mode: "full-access".into(),
                interaction_mode: "default".into(),
                provider_policy: serde_json::json!({"kind":"codex",
                    "approval_policy":"never","approvals_reviewer":"user",
                    "sandbox_policy":{"type":"readOnly","networkAccess":false}}),
            },
        };
        let root_prepared = runtime.prepare(&identity, &request).unwrap();
        assert!(root_prepared.workspace.is_none());
        assert_eq!(root_prepared.effective_posture, SpawnPosture::Discovery);
        assert_eq!(Path::new(&root_prepared.placement.execution_cwd), project);
        // Quota du store avant reçu : le même token reste utilisable après
        // retour de capacité. Aucun reçu ni processus n'est né.
        let quota_file = runtime.store.root.join("quota198");
        let quota = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(&quota_file).unwrap();
        quota.set_len(MAX_WORKER_STORE_BYTES as u64).unwrap();
        let mut quota_command = identity.clone();
        quota_command.instance_id = "instance-quota198".into();
        quota_command.generation = 1;
        let quota_start = WorkerStart {
            prepared_token: root_prepared.token.clone(), child_id: "child-quota198".into(),
            mission_id: "mission-quota198".into(), forward_child_requests: false,
            placement: root_prepared.placement.clone(), definition: root_prepared.definition.clone(),
            child_policy: root_prepared.child_policy.clone(),
            effective_posture: root_prepared.effective_posture, deadline_at: worker_now() + 300,
        };
        let mut capacity_cause = WorkerStartCapacityCause::Unknown;
        assert_eq!(runtime.start(&quota_command, &quota_start, &AtomicBool::new(false),
            &mut capacity_cause).unwrap_err(), WorkerRefusal::ResourceLimit);
        assert_eq!(capacity_cause, WorkerStartCapacityCause::CommandStore);
        assert_eq!(runtime.prepared.get(&identity.command_id).unwrap().prepared.token, root_prepared.token);
        assert!(!runtime.store.command_path(&quota_command).exists());
        assert!(runtime.running.is_empty() && runtime.held.is_empty() && runtime.abandoned.is_empty());
        fs::remove_file(quota_file).unwrap();
        let source_status = git(&project, &["status", "--porcelain=v1"]);
        let mut previous_shared = identity.clone();
        previous_shared.command_id = "previous-shared-198".into();
        previous_shared.instance_id = "old-shared-198".into();
        previous_shared.generation = 1;
        let mut next_shared = identity.clone();
        next_shared.command_id = "next-shared-198".into();
        let mut shared_request = request.clone();
        shared_request.workspace = Some(WorkspaceRequest::Shared {});
        shared_request.parent.revision += 1;
        shared_request.parent.run_id = "new-shared-run-198".into();
        runtime.fence_unstarted(&previous_shared, &root_prepared.placement,
            worker_now() - 1).unwrap();
        let shared = runtime.prepare_fenced(&next_shared, &previous_shared,
            &shared_request, &root_prepared.placement, None).unwrap();
        assert_eq!(shared.placement, root_prepared.placement);
        assert!(shared.workspace.is_none());
        assert!(!runtime.store.workspace_path(&next_shared).exists());
        assert_eq!(git(&project, &["status", "--porcelain=v1"]), source_status);
        assert_eq!(git(&project, &["rev-parse", "HEAD"]), base);
        let mut changed_shared = shared_request.clone();
        changed_shared.base_commit = Some("0".repeat(40));
        assert_eq!(runtime.prepare_fenced(&next_shared, &previous_shared,
            &changed_shared, &root_prepared.placement, None).unwrap_err(),
            WorkerRefusal::PlacementMismatch);
        runtime.recheck(runtime.prepared.get(&next_shared.command_id).unwrap()).unwrap();
        runtime.prepared.remove(&next_shared.command_id);
        fs::remove_file(runtime.store.command_path(&previous_shared)).unwrap();
        request.workspace = Some(WorkspaceRequest::Worktree {
            base_ref: None,
            branch: None,
        });
        let prepared = runtime.prepare(&identity, &request).unwrap();
        let record: crate::delegation::WorkspaceRecord =
            serde_json::from_value(prepared.workspace.clone().unwrap()).unwrap();
        assert_ne!(Path::new(&record.path), project);
        assert_eq!(prepared.placement.execution_cwd, record.path);
        assert_eq!(record.base_commit, base);
        assert_eq!(
            fs::read(Path::new(&record.path).join("base.txt")).unwrap(),
            b"base\n"
        );
        let before = git(&project, &["worktree", "list", "--porcelain"]);
        let replay = runtime.prepare(&identity, &request).unwrap();
        assert_eq!(replay.workspace, prepared.workspace);
        assert_eq!(git(&project, &["worktree", "list", "--porcelain"]), before);
        let original_receipt = fs::read(runtime.store.workspace_path(&identity)).unwrap();
        let worktree_status = git(Path::new(&record.path), &["status", "--porcelain=v1"]);
        let mut previous = identity.clone();
        previous.instance_id = "previous-198".into();
        previous.generation = 1;
        let mut next = identity.clone();
        next.command_id = "fenced-prepare-198".into();
        let mut next_request = request.clone();
        next_request.parent.revision += 1;
        next_request.parent.run_id = "new-run-198".into();
        next_request.parent.source = "provider_turn".into();
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::UnknownExecution);
        let old_now = worker_now();
        runtime.store.reserve_command(&previous, &"a".repeat(64), &prepared.placement,
            old_now + 300, old_now).unwrap();
        runtime.store.set_phase(&previous, WorkerCommandPhase::Refused,
            old_now + 1).unwrap();
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::UnknownExecution);
        assert!(!runtime.store.workspace_path(&next).exists());
        fs::remove_file(runtime.store.command_path(&previous)).unwrap();
        runtime.fence_unstarted(&previous, &prepared.placement, worker_now() + 300).unwrap();
        let mut other_placement = prepared.placement.clone();
        other_placement.mapping_revision += 1;
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &other_placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        let mut other_workspace = prepared.workspace.clone().unwrap();
        other_workspace["branch"] = serde_json::json!("other-branch");
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, Some(&other_workspace)).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        let mut changed_mapping = next_request.clone();
        changed_mapping.mapping_revision += 1;
        assert_eq!(runtime.prepare_fenced(&next, &previous, &changed_mapping,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::PlacementMismatch);
        let mut gained_rights = next_request.clone();
        gained_rights.parent.provider_policy["sandbox_policy"]["networkAccess"] =
            serde_json::json!(true);
        assert!(runtime.prepare_fenced(&next, &previous, &gained_rights,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        assert!(!runtime.store.workspace_path(&next).exists());
        assert_eq!(git(&project, &["worktree", "list", "--porcelain"]), before);
        let mut volatile_workspace = prepared.workspace.clone().unwrap();
        volatile_workspace["dirty"] = serde_json::json!(true);
        volatile_workspace["state"] = serde_json::json!("kept");
        // Sélection168 d'un reçu PrepareV2 : candidat absent ou désactivé refuse avant tout lien.
        let disabled = bridget_transport::protocol::WorkerFencedRouting {
            workload: AutoDelegateWorkload::General,
            connection_id: "connexion-desactivee-198".into(),
            engine: "codex".into(),
            expected_definition_digest: prepared.definition.digest.clone(),
            expected_permission_digest: prepared.placement.permission_digest.clone(),
        };
        assert_eq!(runtime.prepare_fenced_routed(&next, &previous, &next_request,
            &prepared.placement, Some(&volatile_workspace), Some(&disabled)).unwrap_err(),
            WorkerRefusal::CapabilityRequired);
        assert!(!runtime.store.workspace_path(&next).exists());
        assert!(!proof198::bind_path(&runtime.store.root, &next).exists());
        let fresh = runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, Some(&volatile_workspace)).unwrap();
        assert_eq!(fresh.placement, prepared.placement);
        assert_eq!(fresh.workspace, prepared.workspace);
        assert_ne!(fresh.token, prepared.token);
        assert_eq!(runtime.prepare_fenced(&next, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap().token, fresh.token);
        // ACK Prepare perdu puis nouveau tour équivalent : même preuve, mêmes droits recontrôlés.
        let mut later_turn = next_request.clone();
        later_turn.parent.revision += 1;
        later_turn.parent.run_id = "later-run-198".into();
        later_turn.parent.provider_session_id = "later-session-198".into();
        let replay = runtime.prepare_fenced(&next, &previous, &later_turn,
            &prepared.placement, prepared.workspace.as_ref());
        assert_eq!(replay.unwrap().token, fresh.token);
        assert_eq!(runtime.prepared.get(&next.command_id).unwrap().request.parent,later_turn.parent,
            "Release doit employer la révision actuelle, même quand le token reste identique");
        let proof_before=fs::read(proof198::bind_path(&runtime.store.root,&next)).unwrap();
        let restart_config=WorkerRuntimeConfig {
            machine_id:runtime.config.machine_id.clone(),worker_id:runtime.config.worker_id.clone(),
            provisioning_revision:runtime.config.provisioning_revision,credential_path:runtime.config.credential_path.clone(),
            state_root:runtime.config.state_root.clone(),registry_path:runtime.config.registry_path.clone(),
            master_socket:runtime.config.master_socket.clone()};
        drop(runtime);
        let mut runtime=WorkerRuntime::new(restart_config).unwrap();
        runtime.source.clear();runtime.source.insert("HOME".into(),fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(),OsString::from("/usr/bin:/bin"));
        let fresh=runtime.prepare_fenced(&next,&previous,&later_turn,&prepared.placement,
            prepared.workspace.as_ref()).unwrap();
        assert_eq!(fs::read(proof198::bind_path(&runtime.store.root,&next)).unwrap(),proof_before,
            "ACK perdu puis redémarrage et parent renouvelé : canon durable inchangé");
        let mut other_cwd = next_request.clone();
        other_cwd.parent.cwd.push_str("-autre");
        assert!(runtime.prepare_fenced(&next, &previous, &other_cwd,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        let mut other_driver = next_request.clone();
        other_driver.parent.driver.push_str("-autre");
        assert!(runtime.prepare_fenced(&next, &previous, &other_driver,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        let mut other_policy = next_request.clone();
        other_policy.parent.provider_policy["sandbox_policy"]["networkAccess"] =
            serde_json::json!(true);
        assert!(runtime.prepare_fenced(&next, &previous, &other_policy,
            &prepared.placement, prepared.workspace.as_ref()).is_err());
        let bound = proof198::read_workspace_receipt(&runtime.store.root, &runtime.store.worker, &next)
            .unwrap().unwrap();
        assert_eq!(bound.fenced_previous, Some(previous.clone()));
        assert!(bound.record.is_none(), "la nouvelle préparation ne copie pas l'ancre Git");
        let raw: serde_json::Value = serde_json::from_slice(
            &fs::read(runtime.store.workspace_path(&next)).unwrap()).unwrap();
        assert!(raw.get("fenced_previous").is_none(), "reçu workspace strictement au format183");
        assert!(proof198::bind_path(&runtime.store.root, &next).exists());
        assert!(runtime.store.inventory().is_ok());
        assert!(assert_legacy183_readable(&runtime.store.root) >= 1);
        runtime.recheck(runtime.prepared.get(&next.command_id).unwrap()).unwrap();
        let mut launched = next.clone();
        launched.instance_id = "new-instance-198".into();
        launched.generation = 2;
        let start = WorkerStart {
            prepared_token: fresh.token.clone(), child_id: "child-198".into(),
            mission_id: "mission-198".into(), forward_child_requests: false,
            placement: fresh.placement.clone(), definition: fresh.definition.clone(),
            child_policy: fresh.child_policy.clone(), effective_posture: fresh.effective_posture,
            deadline_at: worker_now() + 300,
        };
        let digest = worker_command_digest(&runtime.identity, &launched, &start).unwrap();
        runtime.store.reserve_command(&launched, &digest, &fresh.placement,
            start.deadline_at, worker_now()).unwrap();
        runtime.store.bind_preparation(&launched,
            runtime.prepared.get(&next.command_id).unwrap()).unwrap();
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut runtime = WorkerRuntime::new(config).unwrap();
        runtime.source.clear();
        runtime.source.insert("HOME".into(), fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(), OsString::from("/usr/bin:/bin"));
        let persisted = runtime.store.command(&launched).unwrap();
        let (persisted_request, persisted_prepared) = persisted.preparation.unwrap();
        assert_eq!(persisted_request.parent.run_id, "later-run-198");
        let reconstructed = PreparedRuntime {
            routing: persisted.routing, command: next.clone(), request: persisted_request,
            prepared: persisted_prepared, expires: Instant::now() + Duration::from_secs(30),
        };
        runtime.recheck(&reconstructed).unwrap();
        assert_eq!(fs::read(runtime.store.workspace_path(&identity)).unwrap(), original_receipt);
        assert_eq!(git(&project, &["worktree", "list", "--porcelain"]), before);
        assert_eq!(git(Path::new(&record.path), &["status", "--porcelain=v1"]), worktree_status);
        assert_eq!(fs::read(Path::new(&record.path).join("base.txt")).unwrap(), b"base\n");
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        // Le reste de ce test historique conserve le Prepared initial.
        runtime.prepared.insert(identity.command_id.clone(), PreparedRuntime {
            routing: None, command: identity.clone(), request: request.clone(),
            prepared: prepared.clone(), expires: Instant::now() + Duration::from_secs(30),
        });
        let retained = runtime.prepared.get(&identity.command_id).unwrap();
        runtime.recheck(retained).unwrap();
        fs::write(
            Path::new(&record.path).join("travail.txt"),
            b"travail a conserver\n",
        )
        .unwrap();
        git(Path::new(&record.path), &["add", "travail.txt"]);
        git(
            Path::new(&record.path),
            &["commit", "--quiet", "-m", "fixture travail"],
        );
        runtime.recheck_at(retained, false).unwrap();
        assert_eq!(
            runtime.recheck(retained).unwrap_err(),
            WorkerRefusal::PlacementMismatch
        );
        let report = crate::workspace_git::report(&record).unwrap();
        assert_eq!(report.commit_count, Some(1));
        assert_eq!(report.dirty, Some(false));
        assert_ne!(report.head_commit.as_ref(), Some(&record.base_commit));
        assert!(!project.join("travail.txt").exists());
        let after_work = git(&project, &["worktree", "list", "--porcelain"]);
        assert!(runtime.prepare(&identity, &request).is_err());
        assert_eq!(
            fs::read(Path::new(&record.path).join("travail.txt")).unwrap(),
            b"travail a conserver\n"
        );
        assert_eq!(
            git(&project, &["worktree", "list", "--porcelain"]),
            after_work
        );
        let mut later = next.clone();
        later.command_id = "later-after-head-198".into();
        assert_eq!(runtime.prepare_fenced(&later, &previous, &next_request,
            &prepared.placement, prepared.workspace.as_ref()).unwrap_err(),
            WorkerRefusal::PlacementMismatch);
        assert!(!runtime.store.workspace_path(&later).exists());
        fs::remove_file(runtime.store.command_path(&previous)).unwrap();
        runtime.recheck_at(&reconstructed, false).unwrap();
        fs::remove_file(runtime.store.command_path(&launched)).unwrap();
        fs::remove_file(runtime.store.workspace_path(&next)).unwrap();
        assert!(runtime.store.commands().unwrap().is_empty());
        assert!(runtime.running.is_empty() && runtime.held.is_empty());
        // Le contrat réseau accepte 256 octets ; le bootstrap historique en
        // accepte 200. Ce refus réel intervient avant spawn, sans injection.
        let mut refused_command = identity.clone();
        refused_command.command_id = "refused-before-birth162".into();
        let mut root_request = request.clone();
        root_request.workspace = None;
        let ready = runtime.prepare(&refused_command, &root_request).unwrap();
        refused_command.instance_id = "i".repeat(201);
        refused_command.generation = 1;
        let start = WorkerStart {
            prepared_token: ready.token,
            child_id: "child162".into(),
            mission_id: "mission162".into(),
            forward_child_requests: false,
            placement: ready.placement,
            definition: ready.definition,
            child_policy: ready.child_policy,
            effective_posture: ready.effective_posture,
            deadline_at: worker_now() + 300,
        };
        assert_eq!(
            runtime
                .start(&refused_command, &start, &AtomicBool::new(false), &mut WorkerStartCapacityCause::Unknown)
                .unwrap_err(),
            WorkerRefusal::StoreUnavailable
        );
        assert_eq!(
            runtime.store.command(&refused_command).unwrap().phase,
            WorkerCommandPhase::Refused
        );
        assert!(
            runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty()
        );
        assert!(matches!(
            runtime.reconcile(&refused_command).unwrap(),
            WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase: WorkerCommandPhase::Refused,
                    ..
                }
            }
        ));
        let mut registry: serde_json::Value =
            serde_json::from_slice(&fs::read(&runtime.config.registry_path).unwrap()).unwrap();
        registry["machines"]["cartae"]["enabled"] = false.into();
        write_json(&runtime.config.registry_path, &registry).unwrap();
        assert!(runtime.registry().is_ok());
        assert_eq!(
            runtime.prepare(&identity, &request).unwrap_err(),
            WorkerRefusal::PlacementMismatch
        );
    }

    #[test]
    fn worker162_parent_proof_handle_correle_hote_tuple_revision_et_epoch() {
        use bridget_transport::protocol::ProviderPermissions;
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let cwd = fixture.0.canonicalize().unwrap();
        let mut placed = placement();
        placed.execution_cwd = cwd.to_string_lossy().into_owned();
        let now = worker_now();
        runtime
            .store
            .reserve_command(&command(), &"a".repeat(64), &placed, now + 300, now)
            .unwrap();
        // Le reçu représente un parent déjà libéré. Aucun processus n'est créé ici.
        runtime
            .store
            .mark_command(
                &command(),
                &ManagedMarker {
                    pgid: 42,
                    birth: 1,
                    instance_id: command().instance_id,
                    command_id: command().command_id,
                    generation: command().generation,
                },
                now,
            )
            .unwrap();
        runtime
            .store
            .authorize_release(
                &command(),
                &WorkerReleasePermit {
                    permit_id: "parent-proof".into(),
                    command_digest: "a".repeat(64),
                    permission_revision: 1,
                    mapping_revision: 1,
                    provisioning_revision: 1,
                    issued_at: now,
                    expires_at: now + 5,
                },
                "epoch",
                now,
            )
            .unwrap();
        runtime
            .store
            .set_phase(&command(), WorkerCommandPhase::Released, now)
            .unwrap();
        let parent = ProviderPermissions {
            version: 1,
            source: "native_wrapper".into(),
            run_id: "run162".into(),
            provider_session_id: "session162".into(),
            provider_instance_id: command().instance_id,
            revision: 1,
            driver: "codex_app_server".into(),
            cwd: placed.execution_cwd,
            runtime_mode: "full-access".into(),
            interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never",
                "approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
        };
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION,
            correlation_id: "parent-proof162".into(),
            connection_epoch: "epoch".into(),
            worker: worker(),
            command: command(),
            request: WorkerRequestOperation::ParentProof {
                parent: parent.clone(),
                expected_revision: 1,
            },
        };
        let disconnected = AtomicBool::new(false);
        assert_eq!(
            runtime.handle(&request, "epoch", &disconnected).unwrap(),
            WorkerResponseOperation::ParentProof {
                parent: parent.clone(),
                permission_digest: json_digest(&parent).unwrap()
            }
        );
        // Source172 : vrai dépôt sur le worker, jamais lu sur le coordinateur.
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git").arg("-C").arg(&cwd)
                .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgSign=false"])
                .args(args).output().unwrap();
            assert!(output.status.success(), "{output:?}");
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&["init", "-q", "--initial-branch=main"]);
        fs::write(cwd.join("source172"), "base").unwrap();
        git(&["add", "source172"]); git(&["commit", "-qm", "Base172"]);
        let head = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "-qb", "feature172"]);
        fs::write(cwd.join("source172"), "feature").unwrap();
        git(&["commit", "-qam", "Feature172"]);
        let feature = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "-q", "main"]);
        let mut source = bridget_transport::protocol::WorkerSourceBase {
            cwd: parent.cwd.clone(), expected_base_commit: Some(head.clone()),
            workspace: Some(bridget_transport::protocol::WorkspaceRequest::Shared {}),
            workload: bridget_transport::protocol::AutoDelegateWorkload::General,
        };
        let prove = |runtime: &mut WorkerRuntime, source| {
            let mut source_request = request.clone();
            source_request.request = WorkerRequestOperation::ParentSourceProof { parent: parent.clone(), source };
            runtime.handle(&source_request, "epoch", &disconnected).unwrap()
        };
        let valid = WorkerResponseOperation::ParentSourceProof { permission_digest: json_digest(&parent).unwrap(), source_error: None };
        assert_eq!(prove(&mut runtime, source.clone()), valid);
        source.expected_base_commit = Some("b".repeat(40));
        assert!(matches!(prove(&mut runtime, source.clone()), WorkerResponseOperation::ParentSourceProof { source_error: Some(code), .. } if code == "expected_base_commit_mismatch"));
        source.expected_base_commit = None;
        assert!(matches!(prove(&mut runtime, source.clone()), WorkerResponseOperation::ParentSourceProof { source_error: Some(code), .. } if code == "expected_base_commit_required"));
        source.workspace = Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref: Some("feature172".into()), branch: None });
        source.expected_base_commit = Some(feature);
        assert_eq!(prove(&mut runtime, source.clone()), valid);
        source.expected_base_commit = Some(head.clone());
        assert!(matches!(prove(&mut runtime, source), WorkerResponseOperation::ParentSourceProof { source_error: Some(code), .. } if code == "expected_base_commit_mismatch"));
        assert_eq!(git(&["rev-parse", "HEAD"]), head);
        assert!(runtime.running.is_empty() && runtime.held.is_empty());
        let receipt_before = fs::read(runtime.store.command_path(&command())).unwrap();
        let mut stale = request.clone();
        stale.command.generation += 1;
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::StoreUnavailable)
        );
        stale = request.clone();
        stale.connection_epoch = "old".into();
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::StaleConnection)
        );
        stale = request.clone();
        if let WorkerRequestOperation::ParentProof {
            expected_revision, ..
        } = &mut stale.request
        {
            *expected_revision = 2;
        }
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::InvalidEnvelope)
        );
        stale = request.clone();
        if let WorkerRequestOperation::ParentProof { parent, .. } = &mut stale.request {
            parent.cwd = runtime.config.state_root.to_string_lossy().into_owned();
        }
        assert_eq!(
            runtime.handle(&stale, "epoch", &disconnected),
            Err(WorkerRefusal::Unauthorized)
        );
        assert_eq!(
            fs::read(runtime.store.command_path(&command())).unwrap(),
            receipt_before
        );
        assert!(runtime.running.is_empty() && runtime.held.is_empty());
    }

    #[test]
    fn worker162_runtime_epoch_fence_et_journal_refusent_mauvaise_identite() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        runtime
            .store
            .reserve_command(
                &command(),
                &"a".repeat(64),
                &placement(),
                worker_now() + 300,
                worker_now(),
            )
            .unwrap();
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION,
            correlation_id: "corr".into(),
            connection_epoch: "old".into(),
            worker: worker(),
            command: command(),
            request: WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {},
            },
        };
        assert_eq!(
            runtime
                .handle(&request, "new", &AtomicBool::new(false))
                .unwrap_err(),
            WorkerRefusal::StaleConnection
        );
        assert_eq!(
            runtime.store.command(&command()).unwrap().phase,
            WorkerCommandPhase::Reserved
        );
        let mut hostile = command();
        hostile.instance_id = "../outside".into();
        hostile.command_id = "other".into();
        runtime
            .store
            .reserve_command(
                &hostile,
                &"b".repeat(64),
                &placement(),
                worker_now() + 300,
                worker_now(),
            )
            .unwrap();
        runtime.store.bind_launch(&hostile, "child162").unwrap();
        assert_eq!(
            runtime.journal(&hostile, 0, 1).unwrap_err(),
            WorkerRefusal::JournalUnavailable
        );
        assert!(runtime.running.is_empty());
    }
    #[test]
    fn spec172_workspace_limit_reason_is_closed_and_legacy_safe() {
        use CandidateProbeRefusal as R;
        let reason = workspace_probe_refusal("workspace_limit: private path or stderr");
        assert_eq!(reason, R::WorkspaceLimit);
        assert_eq!(serde_json::to_value(reason).unwrap(), "workspace_limit");
        assert_eq!(workspace_probe_refusal("Git 128: workspace_limit"), R::WorkspaceUnavailable);
        assert_eq!(workspace_probe_reason_for_master(reason, false), R::WorkspaceUnavailable);
        assert_eq!(workspace_probe_reason_for_master(reason, true), R::WorkspaceLimit);
        assert_eq!(workspace_probe_reason_for_master(R::WorkspaceBusy, false), R::WorkspaceBusy);
    }

    #[test]
    fn worker162_maitre_ancien_refuse_avant_hello_et_credential() {
        use std::io::BufRead;
        for supported in [false, true] {
            let (client, server) = UnixStream::pair().unwrap();
            let peer = std::thread::spawn(move || {
                let mut reader = BufReader::new(server.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    serde_json::from_str::<WrapperToDaemon>(&line).unwrap(),
                    WrapperToDaemon::DaemonIdentityRequest
                ));
                let features = if supported {
                    vec!["remote_worker_v1".to_string(), "worker_workspace_diagnostics_v1".to_string()]
                } else {
                    vec![]
                };
                let report = DaemonToWrapper::DaemonIdentityReport {
                    host: "host".into(),
                    db_path: "db".into(),
                    instance_id: "daemon".into(),
                    features,
                };
                let mut server = server;
                server
                    .write_all(
                        bridget_transport::protocol::encode(&report)
                            .unwrap()
                            .as_bytes(),
                    )
                    .unwrap();
                server.write_all(b"\n").unwrap();
                let mut rest = Vec::new();
                reader.read_to_end(&mut rest).unwrap();
                assert!(
                    rest.is_empty(),
                    "aucun WorkerHello/credential avant preuve de capacité"
                );
            });
            let writer = Mutex::new(client.try_clone().unwrap());
            let mut reader = BufReader::new(client);
            let probe = probe_worker_capability(&writer, &mut reader);
            assert_eq!(probe.is_ok(), supported, "sonde: {probe:?}");
            if supported { assert!(probe.unwrap().workspace_diagnostics); }
            drop(writer);
            drop(reader);
            peer.join().unwrap();
        }
    }
    #[cfg(target_os="linux")]
    #[test]
    fn spec185_spec193_worker_repair_publishes_overlay_and_keeps_original_command_and_slots() {
        use bridget_transport::protocol::{WorkspaceRequest,ProviderPermissions,SpawnPosture,WorkspaceRepairRequestV1};
        use std::os::unix::process::CommandExt;
        let fixture=Fixture::new();
        let mut runtime=runtime_fixture(&fixture);
        let project_fixture=Fixture::new();
        let project=project_fixture.0.join("project185");fs::create_dir(&project).unwrap();
        let git=|cwd:&Path,args:&[&str]| {
            let output=std::process::Command::new("git").arg("-C").arg(cwd).args(args)
                .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
                .env("GIT_AUTHOR_NAME","Fixture").env("GIT_COMMITTER_NAME","Fixture")
                .env("GIT_AUTHOR_EMAIL","fixture@example.invalid").env("GIT_COMMITTER_EMAIL","fixture@example.invalid")
                .output().unwrap();
            assert!(output.status.success(),"{args:?}: {}",String::from_utf8_lossy(&output.stderr));
            String::from_utf8(output.stdout).unwrap().trim().to_string()
        };
        git(&project,&["init","-q"]);
        git(&project,&["config","remote.origin.url","fixture:repair185"]);
        fs::write(project.join(".gitignore"),".worktrees/\n").unwrap();
        fs::write(project.join("file"),"conservé\n").unwrap();
        git(&project,&["add","."]);git(&project,&["commit","-qm","base"]);
        let base=git(&project,&["rev-parse","HEAD"]);
        write_json(&runtime.config.registry_path,&serde_json::json!({
            "agents":{"codex":{"command":"/bin/echo","args":["app-server"],"protocol":"codex_app_server",
                "forbidden_env":[],"capabilities":{"execution_paths":["codex_app_server"],
                     "models":{"fixture-model":{"efforts":["high"]}},
                    "observed":{"binary_path":"/bin/echo","binary_version":"fixture185",
                        "binary_digest":digest_bytes(&fs::read("/bin/echo").unwrap()),"contract_version":"codex-app-server-fixture-v1",
                        "operations":["resume"]}}}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":digest_bytes(b"worker162-test-secret"),"allowed_agent_types":["codex"],
                "mappings":{"repair185":{"source_host":"origin185","source_root":"/origin185",
                    "remote_root":project,"expected_git_identity":"fixture:repair185","mapping_revision":1}}}}
        })).unwrap();
        runtime.source.clear();
        runtime.source.insert("HOME".into(),fixture.0.as_os_str().to_owned());
        runtime.source.insert("PATH".into(),"/usr/bin:/bin".into());
        let mut identity=command();identity.task_id=uuid::Uuid::new_v4().to_string();
        let mut prepare_identity=identity.clone();prepare_identity.instance_id.clear();prepare_identity.generation=0;
        let prepare=WorkerPrepare {
            machine_id:"cartae".into(),project_id:"repair185".into(),provisioning_revision:1,mapping_revision:1,
            origin_host:"origin185".into(),origin_cwd:"/origin185".into(),source_root:"/origin185".into(),
            remote_root:project.to_string_lossy().into_owned(),source_root_identity:"fixture:repair185".into(),
            relative_cwd:".".into(),base_commit:Some(base.clone()),agent_type:"codex".into(),
            model:"fixture-model".into(),effort:Some("high".into()),posture:Some(SpawnPosture::Development),
            workspace:Some(WorkspaceRequest::Worktree {base_ref:None,branch:None}),
            parent:ProviderPermissions {version:1,source:"native_wrapper".into(),run_id:"run185".into(),
                provider_session_id:"parent-session185".into(),provider_instance_id:"parent185".into(),revision:1,
                driver:"codex_app_server".into(),cwd:"/origin185".into(),runtime_mode:"full-access".into(),
                interaction_mode:"default".into(),provider_policy:serde_json::json!({"kind":"codex",
                    "approval_policy":"never","approvals_reviewer":"user",
                    "sandbox_policy":{"type":"workspaceWrite","writableRoots":["/origin185"],"networkAccess":false,"excludeTmpdirEnvVar":true,"excludeSlashTmp":true}})},
        };
        let prepared=runtime.prepare(&prepare_identity,&prepare).unwrap();
        let original:crate::delegation::WorkspaceRecord=serde_json::from_value(prepared.workspace.clone().unwrap()).unwrap();
        runtime.store.reserve_command(&identity,&"c".repeat(64),&prepared.placement,worker_now()+600,worker_now()).unwrap();
        runtime.store.bind_preparation(&identity,runtime.prepared.get(&identity.command_id).unwrap()).unwrap();
        let mut child=std::process::Command::new("/bin/cat").stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).process_group(0).spawn().unwrap();
        let mut receipt=runtime.store.command(&identity).unwrap();
        receipt.phase=WorkerCommandPhase::Released;receipt.native_ready=true;
        receipt.marker=Some(ManagedMarker {pgid:child.id(),birth:crate::managed_process::process_birth(child.id()).unwrap(),
            command_id:identity.command_id.clone(),instance_id:identity.instance_id.clone(),generation:identity.generation});
        runtime.store.persist(&runtime.store.command_path(&identity),&receipt).unwrap();
        let before188=snapshot188(&runtime.config.state_root);
        let diag188=read::diagnose_observed(runtime.read_view(),&identity,||{}).unwrap();
        assert_eq!(diag188["check"],"passed","{diag188}");
        assert_eq!(diag188["stage"],"complete");
        assert_eq!(snapshot188(&runtime.config.state_root),before188);
        // Mesure du vrai parcours : un seul status Git par réponse, sans cache inter-appels.
        crate::workspace_git::take_status_reads193();
        let measured=Instant::now();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 attendue")};
        let reads=crate::workspace_git::take_status_reads193();
        eprintln!("spec193 normal_status_reads={reads} elapsed_ms={}",measured.elapsed().as_millis());
        assert_eq!(reads,1);
        assert!(observation.permission_digest.is_some());
        assert_eq!(snapshot188(&runtime.config.state_root),before188,"observation sans mutation du store");
        let worktree=PathBuf::from(&original.path);
        fs::write(worktree.join("untracked193"),"changement entre deux appels").unwrap();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 fraîche attendue")};
        assert_eq!(crate::workspace_git::take_status_reads193(),1);
        assert_eq!(observation.workspace.as_ref().unwrap()["dirty"],true);
        fs::remove_file(worktree.join("untracked193")).unwrap();
        let registry_before=fs::read(&runtime.config.registry_path).unwrap();
        let mut changed:serde_json::Value=serde_json::from_slice(&registry_before).unwrap();
        changed["agents"]["codex"]["args"]=serde_json::json!(["app-server","changed193"]);
        write_json(&runtime.config.registry_path,&changed).unwrap();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 refusée attendue")};
        assert_eq!(crate::workspace_git::take_status_reads193(),1);
        assert!(observation.permission_digest.is_none(),"définition modifiée toujours refusée");
        assert!(observation.workspace.is_some(),"le rapport technique reste disponible après refus des droits");
        fs::write(&runtime.config.registry_path,registry_before).unwrap();
        git(&worktree,&["config","filter.blocked193.process","/bin/false"]);
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("observation193 Git refusé attendue")};
        assert!(observation.permission_digest.is_none());
        assert!(observation.workspace.is_none(),"aucun ancien rapport après une erreur Git");
        git(&worktree,&["config","--unset","filter.blocked193.process"]);
        let target=project_fixture.0.join("disk185");
        fs::rename(&original.path,&target).unwrap();
        std::os::unix::fs::symlink(&target,&original.path).unwrap();
        let mut request=WorkspaceRepairRequestV1 {request_id:"repair185-worker".into(),
            target_cwd:target.to_string_lossy().into_owned(),expected_head:base};
        let WorkerResponseOperation::WorkspaceRepair {status}=runtime.repair_workspace(&identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("statut attendu")};
        assert_eq!(status.code.as_deref(),Some("workspace_repair_child_upgrade_required"));
        assert_eq!(serde_json::to_value(runtime.store.command(&identity).unwrap()).unwrap(),
            serde_json::to_value(&receipt).unwrap(),"aucun processus ni canon modifié avant capacité");
        private_dir(&runtime.store.root.join("markers")).unwrap();
        write_json(&workspace_repair::capability_path(&runtime.store.root,&identity),
            &workspace_repair::RepairCapability {version:1,worker:runtime.identity.clone(),command:identity.clone(),
                wrapper_pid:child.id(),wrapper_birth:receipt.marker.as_ref().unwrap().birth,thread_id:"thread185".into()}).unwrap();
        let WorkerResponseOperation::WorkspaceRepair {status}=runtime.repair_workspace(&identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("statut attendu")};
        assert_eq!(status.phase,"prepared","{status:?}");
        // Un refus sans effet clôt la tentative ; une nouvelle clé est alors permise.
        let mut refused_status=status.clone();refused_status.phase="refused".into();
        refused_status.code=Some("workspace_repair_busy".into());
        workspace_repair::persist_wrapper_result(&runtime.store.root,&identity,&refused_status).unwrap();
        let WorkerResponseOperation::WorkspaceRepair {status:refused}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("refus durable attendu")};
        assert_eq!(refused.phase,"refused");
        request.request_id="repair185-worker-second".into();
        let WorkerResponseOperation::WorkspaceRepair {status}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap()
            else {panic!("nouvelle préparation attendue")};
        assert_eq!(status.phase,"prepared","{status:?}");
        let retained=runtime.store.command(&identity).unwrap();
        assert_eq!(retained.workspace_repair_history.len(),1);
        assert_eq!(retained.workspace_repair_history[0].phase,"refused");
        let (anchor,source,_)=workspace_repair::proof(&status).unwrap();
        crate::workspace_git::apply_repair_git(&source,&anchor).unwrap();
        // Frontière wrapper simulée : ce test vérifie les contrôles worker.
        // Le test wrapper185 qualifie séparément l'arrêt et la reprise réels.
        let mut resumed=status.clone();resumed.phase="resumed".into();
        workspace_repair::persist_wrapper_result(&runtime.store.root,&identity,&resumed).unwrap();
        let WorkerResponseOperation::WorkspaceRepair {status:observed}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Prepare,worker_now()+60).unwrap() else {panic!("reçu observé attendu")};
        assert_eq!(observed.phase,"resumed","un ACK perdu ne rejoue pas le contrôle wrapper");
        assert_eq!(runtime.store.command(&identity).unwrap().workspace_repair.as_ref().unwrap().phase,"prepared",
            "l'observation ne publie aucune liaison");
        let before=runtime.store.command(&identity).unwrap();
        assert_eq!(runtime.repair_workspace(&identity,&request,
            bridget_transport::protocol::WorkspaceRepairActionV1::Commit,worker_now()-1).unwrap_err(),
            WorkerRefusal::PrepareExpired);
        assert_eq!(runtime.store.command(&identity).unwrap(),before,"expiration avant commit sans mutation");
        let WorkerResponseOperation::WorkspaceRepair {status:published}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Commit,worker_now()+60).unwrap() else {panic!("publication attendue")};
        assert_eq!(published.phase,"committed","{published:?}");
        assert!(!workspace_repair::published(&runtime.store.root,&identity,&request));
        let before=runtime.store.command(&identity).unwrap();
        assert_eq!(runtime.repair_workspace(&identity,&request,
            bridget_transport::protocol::WorkspaceRepairActionV1::Publish,worker_now()-1).unwrap_err(),
            WorkerRefusal::PrepareExpired);
        assert_eq!(runtime.store.command(&identity).unwrap(),before,"expiration avant publication sans mutation");
        assert!(!workspace_repair::published(&runtime.store.root,&identity,&request),"barrière conservée");

        let WorkerResponseOperation::WorkspaceRepair {status:published}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Publish,worker_now()+60).unwrap()
            else {panic!("publication attendue")};
        assert_eq!(published.phase,"published","{published:?}");
        let after=runtime.store.command(&identity).unwrap();
        assert_eq!(after.identity,receipt.identity);assert_eq!(after.command_digest,receipt.command_digest);
        assert_eq!(after.placement,receipt.placement);assert_eq!(after.marker,receipt.marker);
        assert_eq!(after.preparation,receipt.preparation);
        assert!(runtime.store.inventory().unwrap().1.is_empty(),"aucun slot métier créé");
        crate::workspace_git::take_status_reads193();
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("preuve attendue")};
        assert_eq!(crate::workspace_git::take_status_reads193(),1,"placement réparé sans lectures répétées");
        assert!(observation.native_ready);
        assert_eq!(observation.permission_digest.as_ref(),
            published.prepared.as_ref().map(|v|&v.placement.permission_digest));
        assert_eq!(observation.workspace.as_ref().unwrap()["path"],target.to_string_lossy().as_ref());
        fs::write(target.join("next-round"),"nouveau travail conservé").unwrap();
        git(&target,&["add","next-round"]);git(&target,&["commit","-qm","ronde suivante"]);
        let next_head=git(&target,&["rev-parse","HEAD"]);
        let WorkerResponseOperation::Observation {observation}=runtime.reconcile_attested(&identity).unwrap()
            else {panic!("preuve après commit attendue")};
        assert!(observation.native_ready);
        assert!(observation.permission_digest.is_some(),"un commit après migration doit rester continuable");
        assert_eq!(observation.workspace.as_ref().unwrap()["head_commit"],next_head);
        // Les délégations du parent réparé prouvent les nouveaux droits et le nouveau dossier.
        let parent=ProviderPermissions {cwd:target.to_string_lossy().into_owned(),
            provider_policy:published.prepared.as_ref().unwrap().child_policy.clone(),
            provider_instance_id:identity.instance_id.clone(),..prepare.parent.clone()};
        let request_frame=WorkerRequest {version:REMOTE_WORKER_VERSION,correlation_id:"parent-repaired185".into(),
            connection_epoch:"epoch185".into(),worker:runtime.identity.clone(),command:identity.clone(),
            request:WorkerRequestOperation::ParentProof {parent:parent.clone(),expected_revision:parent.revision}};
        let disconnected=AtomicBool::new(false);
        assert!(matches!(runtime.handle(&request_frame,"epoch185",&disconnected).unwrap(),
            WorkerResponseOperation::ParentProof {..}));
        let mut source_frame=request_frame;
        source_frame.request=WorkerRequestOperation::ParentSourceProof {parent:parent.clone(),
            source:bridget_transport::protocol::WorkerSourceBase {cwd:parent.cwd.clone(),
                expected_base_commit:Some(next_head),workspace:Some(WorkspaceRequest::Shared {}),
                workload:bridget_transport::protocol::AutoDelegateWorkload::General}};
        assert!(matches!(runtime.handle(&source_frame,"epoch185",&disconnected).unwrap(),
            WorkerResponseOperation::ParentSourceProof {source_error:None,..}));
        // Une annulation arrivée après les relectures ferme le point atomique de publication.
        let before=runtime.store.command(&identity).unwrap();
        runtime.cancelled.lock().unwrap().insert(json_digest(&identity).unwrap());
        assert_eq!(runtime.publish_repair_status(&identity,published.clone(),worker_now()+60).unwrap_err(),WorkerRefusal::Unauthorized);
        assert_eq!(runtime.store.command(&identity).unwrap(),before);
        let WorkerResponseOperation::WorkspaceRepair {status:observed}=runtime.repair_workspace(
            &identity,&request,bridget_transport::protocol::WorkspaceRepairActionV1::Observe,0).unwrap()
            else {panic!("lecture sans effet attendue")};
        assert_eq!(observed.phase,"published","l'annulation n'efface pas le reçu publié");
        let capability_path=workspace_repair::capability_path(&runtime.store.root,&identity);
        let result_path=workspace_repair::result_path(&runtime.store.root,&identity,&request.request_id);
        let refused_path=workspace_repair::result_path(&runtime.store.root,&identity,"repair185-worker");
        runtime.store.collect(worker_now()+2*WORKER_RECEIPT_RETENTION_SECS).unwrap();
        assert!(capability_path.exists()&&result_path.exists()&&refused_path.exists(),"commande active conservée");
        bridget_transport::managed_session::stop_owned_child(&mut child,true,Duration::from_secs(2)).unwrap();
        let mut ended=runtime.store.command(&identity).unwrap();ended.phase=WorkerCommandPhase::Terminal;
        runtime.store.persist(&runtime.store.command_path(&identity),&ended).unwrap();
        runtime.store.collect(worker_now()+2*WORKER_RECEIPT_RETENTION_SECS).unwrap();
        assert!(!capability_path.exists()&&!result_path.exists()&&!refused_path.exists(),"marqueurs collectés avec le reçu");
        assert!(!runtime.store.command_path(&identity).exists());
    }

    #[test]
    fn spec185_capabilities_remain_independent_and_reject_unoffered_origin_proof() {
        let features=MasterWorkerFeatures {workspace_repair:true,..Default::default()};
        assert_eq!(features.capabilities(),vec![WorkerCapability::WorkspaceRepairV1]);
        assert!(features.admits(&[WorkerCapability::WorkspaceRepairV1]));
        assert!(!features.admits(&[WorkerCapability::OriginProofV1]));
        assert!(!features.admits(&[WorkerCapability::WorkspaceRepairV1,WorkerCapability::WorkspaceRepairV1]));
    }

    fn start198(deadline_at: i64) -> WorkerStart {
        WorkerStart {
            prepared_token: "token-198".into(),
            child_id: "child-198".into(),
            mission_id: "mission-198".into(),
            forward_child_requests: false,
            placement: placement(),
            definition: ResolvedAgentDefinition {
                command: "/bin/true".into(), args: vec![], protocol: "codex_app_server".into(),
                forbidden_env: vec![], pass_env: vec![], permissions: "plan".into(),
                claude_config_dir: None, queue_capacity: 1, notify_timeout_secs: 60,
                mcp: bridget_transport::protocol::ResolvedMcpDefinition {
                    interactive: "off".into(), acp_session: false,
                },
                capabilities: Default::default(), digest: "a".repeat(64),
            },
            child_policy: serde_json::json!({}),
            effective_posture: SpawnPosture::Discovery,
            deadline_at,
        }
    }

    /// Schémas copiés de 7478e9e7 (183) : tout champ inconnu ou délai incohérent échoue.
    #[allow(dead_code)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Legacy183Command {
        version: u16, worker: WorkerIdentity, identity: WorkerCommandIdentity,
        command_digest: String, placement: RemotePlacement, deadline_at: i64,
        phase: WorkerCommandPhase, marker: Option<ManagedMarker>,
        permit: Option<WorkerReleasePermit>, release_epoch: Option<String>,
        #[serde(default)] child_id: Option<String>,
        #[serde(default)] native_ready: bool,
        #[serde(default)] startup_failed: bool,
        #[serde(default)] startup_failure: Option<bridget_transport::protocol::WorkerStartupFailure>,
        #[serde(default)] terminal_exit_code: Option<i32>,
        #[serde(default)] preparation: Option<(WorkerPrepare, WorkerPrepared)>,
        #[serde(default)] routing: Option<RoutingPreparation>,
        created_at: i64, updated_at: i64,
    }
    #[allow(dead_code)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Legacy183Workspace {
        version: u16, worker: WorkerIdentity, command: WorkerCommandIdentity,
        prepare_digest: String, record: Option<crate::delegation::WorkspaceRecord>,
    }
    /// Relit tout `commands` et `workspaces` comme le ferait l'inventaire183.
    fn assert_legacy183_readable(root: &Path) -> usize {
        let mut seen = 0;
        for entry in fs::read_dir(root.join("commands")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|ext| ext != "json") { continue; }
            let receipt: Legacy183Command =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert!(receipt.deadline_at > receipt.created_at, "règle183 deadline > created_at");
            seen += 1;
        }
        if let Ok(entries) = fs::read_dir(root.join("workspaces")) {
            for entry in entries {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|ext| ext != "json") { continue; }
                let _: Legacy183Workspace =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                seen += 1;
            }
        }
        seen
    }

    #[test]
    fn spec198_canon_fence_ne_normalise_que_le_renouvellement_provider_turn() {
        let parent = ProviderPermissions {
            version: 1, source: "provider_turn".into(), run_id: "run".into(),
            provider_session_id: "session".into(), provider_instance_id: "parent".into(),
            revision: 1, driver: "codex_app_server".into(), cwd: "/origin".into(),
            runtime_mode: "full-access".into(), interaction_mode: "default".into(),
            provider_policy: serde_json::json!({"kind":"codex","approval_policy":"never"}),
        };
        let a = WorkerPrepare {
            machine_id: "cartae".into(), project_id: "project".into(), provisioning_revision: 1,
            mapping_revision: 1, origin_host: "origin".into(), origin_cwd: "/origin".into(),
            source_root: "/origin".into(), remote_root: "/remote".into(),
            source_root_identity: "git:held".into(), relative_cwd: ".".into(), base_commit: None,
            agent_type: "codex".into(), model: "fixture".into(), effort: None, posture: None,
            workspace: None, parent,
        };
        let mut b = a.clone();
        b.parent.run_id = "autre".into();
        b.parent.provider_session_id = "autre-session".into();
        b.parent.revision += 7;
        assert_eq!(fenced_prepare_canon(&a), fenced_prepare_canon(&b));
        for mutate in [
            (|r: &mut WorkerPrepare| r.parent.cwd.push('x')) as fn(&mut WorkerPrepare),
            |r| r.parent.driver.push('x'),
            |r| r.parent.provider_instance_id.push('x'),
            |r| r.parent.runtime_mode.push('x'),
            |r| r.parent.provider_policy = serde_json::json!({"autre": true}),
            |r| r.model.push('x'),
        ] {
            let mut c = a.clone();
            mutate(&mut c);
            assert_ne!(fenced_prepare_canon(&a), fenced_prepare_canon(&c));
        }
        let mut native = a.clone();
        native.parent.source = "native_wrapper".into();
        let mut native_b = native.clone();
        native_b.parent.run_id = "autre".into();
        assert_ne!(fenced_prepare_canon(&native), fenced_prepare_canon(&native_b));
    }

    #[test]
    fn spec198_fence_reste_lisible_format183_et_preuve_seule_bloque_start() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let deadline = worker_now() - 1;
        runtime.fence_unstarted(&command(), &placement(), deadline).unwrap();
        let raw: serde_json::Value = serde_json::from_slice(
            &fs::read(runtime.store.command_path(&command())).unwrap()).unwrap();
        assert!(raw.get("pre_start_fence").is_none(), "reçu commande strictement au format183");
        assert!(raw["deadline_at"].as_i64().unwrap() > raw["created_at"].as_i64().unwrap());
        let proof = proof198::read_fence(&runtime.store.root, &runtime.store.worker, &command())
            .unwrap().unwrap();
        assert_eq!(proof.deadline_at, deadline, "le délai historique reste exact dans la preuve");
        assert_eq!(runtime.store.command(&command()).unwrap().deadline_at, deadline);
        assert!(runtime.store.inventory().is_ok());
        assert_eq!(assert_legacy183_readable(&runtime.store.root), 1);

        // Crash après la preuve et avant le reçu : aucun Start neuf, le rejeu termine.
        let other = WorkerCommandIdentity { command_id: "command-198-proof-only".into(), ..command() };
        let created_at = worker_now();
        let digest = proof198::fence_digest(&runtime.store.worker, &other, &placement(), deadline)
            .unwrap();
        proof198::ensure_dir(&runtime.store.root).unwrap();
        runtime.store.persist(&proof198::fence_path(&runtime.store.root, &other),
            &proof198::FenceProof { version: STORE_VERSION, worker: runtime.store.worker.clone(),
                identity: other.clone(), placement: placement(), deadline_at: deadline,
                created_at, digest }).unwrap();
        let (_, _, _, before_orphan) = runtime.store.inventory().unwrap();
        assert!(before_orphan >= 2, "la preuve annexe orpheline compte dans la quota d'entrées");
        assert!(matches!(runtime.store.reserve_command(&other, &"a".repeat(64), &placement(),
            worker_now() + 300, worker_now()), Err(WorkerStoreError::Conflict)));
        assert_eq!(runtime.fence_unstarted(&other, &placement(), deadline + 1).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        assert!(!runtime.store.command_path(&other).exists());
        runtime.fence_unstarted(&other, &placement(), deadline).unwrap();
        let receipt = runtime.store.command(&other).unwrap();
        assert_eq!((receipt.created_at, receipt.deadline_at), (created_at, deadline));
        assert_eq!(receipt.phase, WorkerCommandPhase::Refused);
        assert_eq!(assert_legacy183_readable(&runtime.store.root), 2);
        // Réponse perdue puis redémarrage : même inventaire, aucune quarantaine, rejeu identique.
        let bytes = fs::read(runtime.store.command_path(&other)).unwrap();
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let mut reopened = WorkerRuntime::new(config).unwrap();
        assert_eq!(reopened.fence_unstarted(&other, &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(reopened.store.command_path(&other)).unwrap(), bytes);
        assert_eq!(reopened.store.command(&command()).unwrap().deadline_at, deadline);
        assert_eq!(assert_legacy183_readable(&reopened.store.root), 2);
    }

    #[test]
    fn spec198_fence_durable_replay_divergence_et_start_tardif() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let deadline = worker_now() - 1;
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        let receipt = runtime.store.command(&command()).unwrap();
        assert_eq!(receipt.phase, WorkerCommandPhase::Refused);
        assert!(receipt.deadline_at < receipt.created_at,
            "le fence conserve le délai historique, même déjà expiré");
        assert!(receipt.pre_start_fence.is_some());
        assert!(receipt.marker.is_none() && receipt.permit.is_none() && receipt.child_id.is_none());
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        assert!(!runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command()))).exists());
        let path = runtime.store.command_path(&command());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(matches!(runtime.cancel(&command()).unwrap(),
            WorkerResponseOperation::Observation { observation: WorkerObservation {
                phase: WorkerCommandPhase::Refused, ..
            } }));
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline + 1).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        let mut divergent = placement();
        divergent.mapping_revision += 1;
        assert_eq!(runtime.fence_unstarted(&command(), &divergent, deadline).unwrap_err(),
            WorkerRefusal::CommandMismatch);
        assert_eq!(runtime.start(&command(), &start198(deadline), &AtomicBool::new(false),
            &mut WorkerStartCapacityCause::Unknown).unwrap_err(), WorkerRefusal::CommandMismatch);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let config = WorkerRuntimeConfig {
            machine_id: runtime.config.machine_id.clone(),
            worker_id: runtime.config.worker_id.clone(),
            provisioning_revision: runtime.config.provisioning_revision,
            credential_path: runtime.config.credential_path.clone(),
            state_root: runtime.config.state_root.clone(),
            registry_path: runtime.config.registry_path.clone(),
            master_socket: runtime.config.master_socket.clone(),
        };
        drop(runtime);
        let reopened = WorkerRuntime::new(config).unwrap();
        assert_eq!(reopened.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(fs::read(path).unwrap(), bytes);
    }

    #[test]
    fn spec198_fence_quota_refuse_sans_recu_puis_accepte_meme_cle() {
        let fixture = Fixture::new();
        let runtime = runtime_fixture(&fixture);
        let deadline = worker_now() - 1;
        let quota_file = runtime.store.root.join("quota-fence198");
        let quota = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(&quota_file).unwrap();
        quota.set_len(MAX_WORKER_STORE_BYTES as u64).unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap_err(),
            WorkerRefusal::ResourceLimit);
        assert!(!runtime.store.command_path(&command()).exists());
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
        fs::remove_file(quota_file).unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), deadline).unwrap(),
            WorkerResponseOperation::StartFenced {});
        assert_eq!(runtime.store.command(&command()).unwrap().phase, WorkerCommandPhase::Refused);
    }

    #[test]
    fn spec198_fence_refuse_tous_reçus_existants_et_marqueur_orphelin() {
        for phase in [WorkerCommandPhase::Reserved, WorkerCommandPhase::Unknown,
            WorkerCommandPhase::Refused] {
            let fixture = Fixture::new();
            let runtime = runtime_fixture(&fixture);
            let now = worker_now();
            runtime.store.reserve_command(&command(), &"d".repeat(64), &placement(), now + 300, now).unwrap();
            if phase != WorkerCommandPhase::Reserved {
                runtime.store.set_phase(&command(), phase, now + 1).unwrap();
            }
            let path = runtime.store.command_path(&command());
            let before = fs::read(&path).unwrap();
            assert_eq!(runtime.fence_unstarted(&command(), &placement(), now + 300).unwrap_err(),
                WorkerRefusal::UnknownExecution);
            assert_eq!(fs::read(path).unwrap(), before);
        }
        let fixture = Fixture::new();
        let runtime = runtime_fixture(&fixture);
        let marker = runtime.store.root.join("markers").join(format!("{}.json", marker_key(&command())));
        write_private_file_atomic(&marker, b"orphan").unwrap();
        assert_eq!(runtime.fence_unstarted(&command(), &placement(), worker_now() - 1).unwrap_err(),
            WorkerRefusal::UnknownExecution);
        assert!(!runtime.store.command_path(&command()).exists());
    }

    #[test]
    fn spec198_file_pleine_distingue_start_detaille_et_wire_historique() {
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "request-198".into(),
            connection_epoch: "epoch-198".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::StartDetailed { start: start198(worker_now() + 300) },
        };
        request.validate().unwrap();
        let wire = serde_json::to_value(&request).unwrap();
        assert_eq!(wire["request"]["operation"], "start_detailed");
        assert_eq!(serde_json::from_value::<WorkerRequest>(wire).unwrap(), request);
        let busy = worker_inbound_busy(WorkerInbound::Request(request.clone()));
        assert!(matches!(busy, WrapperToDaemon::WorkerResponse { response: WorkerResponse {
            response: WorkerResponseOperation::StartRejected {
                reason: WorkerRefusal::ResourceLimit,
                cause: WorkerStartCapacityCause::InboundQueue,
            }, ..
        } }));
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(inbound172("occupant", WorkerRequestOperation::Reconcile {})).unwrap();
        let (socket, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(socket));
        let disconnected = AtomicBool::new(false);
        dispatch_worker_inbound(&sender, WorkerInbound::Request(request.clone()), &writer,
            &disconnected, &Mutex::new(BTreeSet::new())).unwrap();
        let WrapperToDaemon::WorkerResponse { response } = read_inbound172(&mut std::io::BufReader::new(peer))
            else { panic!("réponse détaillée attendue"); };
        response.validate().unwrap();
        assert_eq!(response.response, WorkerResponseOperation::StartRejected {
            reason: WorkerRefusal::ResourceLimit, cause: WorkerStartCapacityCause::InboundQueue,
        });
        assert!(matches!(receiver.try_recv().unwrap(), WorkerInbound::Request(WorkerRequest {
            request: WorkerRequestOperation::Reconcile {}, ..
        })));
        assert!(receiver.try_recv().is_err(), "Start refusé ne doit pas rester en file");
        let mut legacy = request;
        legacy.request = WorkerRequestOperation::Start { start: start198(worker_now() + 300) };
        assert_eq!(serde_json::to_value(&legacy).unwrap()["request"]["operation"], "start");
        let busy = worker_inbound_busy(WorkerInbound::Request(legacy));
        assert!(matches!(busy, WrapperToDaemon::WorkerResponse { response: WorkerResponse {
            response: WorkerResponseOperation::Rejected { reason: WorkerRefusal::ResourceLimit }, ..
        } }));
        let features = MasterWorkerFeatures { start_recovery: true, ..Default::default() };
        assert_eq!(features.capabilities(), vec![WorkerCapability::StartRecoveryV1]);
        assert!(features.admits(&[WorkerCapability::StartRecoveryV1]));
        assert!(!MasterWorkerFeatures::default().admits(&[WorkerCapability::StartRecoveryV1]));
        assert_eq!(WorkerStartCapacityCause::PreparationMetadata.code(), "preparation_metadata");
        assert_eq!(serde_json::to_value(WorkerCapability::StartRecoveryV1).unwrap(), "start_recovery_v1");
        let fence = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "fence-198".into(),
            connection_epoch: "epoch-198".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::FenceUnstarted { placement: placement(), deadline_at: 1 },
        };
        fence.validate().unwrap();
        let wire = serde_json::to_value(&fence).unwrap();
        assert_eq!(wire["request"]["operation"], "fence_unstarted");
        assert_eq!(serde_json::from_value::<WorkerRequest>(wire).unwrap(), fence);
        assert!(matches!(worker_inbound_busy(WorkerInbound::Request(fence)),
            WrapperToDaemon::WorkerResponse { response: WorkerResponse {
                response: WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::ResourceLimit,
                }, ..
            } }));
    }

    #[test]
    fn spec198_fence_exige_epoque_et_frame_trop_grande_reste_sans_effet() {
        let fixture = Fixture::new();
        let mut runtime = runtime_fixture(&fixture);
        let mut request = WorkerRequest {
            version: REMOTE_WORKER_VERSION, correlation_id: "fence-198".into(),
            connection_epoch: "epoch-198".into(), worker: worker(), command: command(),
            request: WorkerRequestOperation::FenceUnstarted {
                placement: placement(), deadline_at: worker_now() - 1,
            },
        };
        let connected = AtomicBool::new(false);
        assert_eq!(runtime.handle(&request, "other-epoch", &connected).unwrap_err(),
            WorkerRefusal::StaleConnection);
        assert!(!runtime.store.command_path(&command()).exists());
        assert_eq!(runtime.handle(&request, "epoch-198", &connected).unwrap(),
            WorkerResponseOperation::StartFenced {});
        request.request = WorkerRequestOperation::StartDetailed {
            start: start198(worker_now() + 300),
        };
        if let WorkerRequestOperation::StartDetailed { start } = &mut request.request {
            start.child_policy = serde_json::json!({"large": "x".repeat(65 * 1024)});
        }
        assert_eq!(request.validate(), Err(WorkerRefusal::ResourceLimit));
        assert!(matches!(worker_inbound_busy(WorkerInbound::Request(request.clone())),
            WrapperToDaemon::WorkerResponse { response: WorkerResponse {
                response: WorkerResponseOperation::StartRejected {
                    reason: WorkerRefusal::ResourceLimit,
                    cause: WorkerStartCapacityCause::RequestFrame,
                }, ..
            } }));
        assert_eq!(runtime.handle(&request, "epoch-198", &connected).unwrap(),
            WorkerResponseOperation::StartRejected {
                reason: WorkerRefusal::ResourceLimit,
                cause: WorkerStartCapacityCause::RequestFrame,
            });
        if let WorkerRequestOperation::StartDetailed { start } = &mut request.request {
            start.child_policy = serde_json::json!({});
            start.prepared_token = bridget_transport::protocol::WORKER_INSPECTION_TOKEN.into();
        }
        assert_eq!(runtime.handle(&request, "epoch-198", &connected).unwrap(),
            WorkerResponseOperation::StartRejected {
                reason: WorkerRefusal::InvalidEnvelope,
                cause: WorkerStartCapacityCause::Unknown,
            });
        assert!(runtime.held.is_empty() && runtime.running.is_empty() && runtime.abandoned.is_empty());
    }

}
