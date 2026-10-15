//! État technique du worker 162. Aucune tâche métier ni réservation Fleet.
//!
//! Un seul worker écrit les reçus. Chaque wrapper publie un seul blob dans son
//! slot réservé ; il n'ouvre jamais le verrou du store ou ses métadonnées.

use crate::managed_process::ManagedMarker;
use bridget_transport::fsutil::{create_private_dir, write_private_file_atomic};
use bridget_transport::protocol::{
    MAX_WORKER_RESULT_ENVELOPE_BYTES, MAX_WORKER_STORE_BYTES, MAX_WORKER_STORE_ENTRIES,
    RemotePlacement, RemoteResultEnvelope, WORKER_RECEIPT_RETENTION_SECS, WorkerCommandIdentity,
    WorkerCommandPhase, WorkerIdentity, WorkerRefusal, WorkerReleasePermit,
    WorkerResultDisposition, WorkerResultIdentity, WorkerResultSlot, WorkerSlotClosureProof,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_METADATA_BYTES: usize = 64 * 1024;
const STORE_VERSION: u16 = 1;
const DRAIN_PROOF_RESERVATION: u64 = 4096;
const MAX_DRAIN_PROOF_BYTES: usize = 2048;

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
    #[serde(default)]
    pub startup_failed: bool,
    /// Canon préparé conservé avant bootstrap pour les rechecks après reprise.
    #[serde(default)]
    preparation: Option<(WorkerPrepare, WorkerPrepared)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    routing: Option<RoutingPreparation>,
    pub created_at: i64,
    pub updated_at: i64,
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

fn drain_proof_path(root: &Path, command: &WorkerCommandIdentity) -> PathBuf {
    root.join("markers")
        .join(format!("{}.drained", marker_key(command)))
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
    record: Option<crate::delegation::WorkspaceRecord>,
}

pub struct WorkerStore {
    root: PathBuf,
    enrollment: PathBuf,
    worker: WorkerIdentity,
    _lock: File,
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
            if receipt.identity != *identity
                || receipt.command_digest != command_digest
                || receipt.placement != *placement
                || receipt.deadline_at != deadline_at
            {
                return Err(WorkerStoreError::Conflict);
            }
            return Ok(CommandReservation::Existing(receipt));
        }
        self.admit(MAX_METADATA_BYTES as u64 * 2 + DRAIN_PROOF_RESERVATION, 1)?;
        let receipt = WorkerCommandReceipt {
            version: STORE_VERSION,
            worker: self.worker.clone(),
            identity: identity.clone(),
            command_digest: command_digest.into(),
            placement: placement.clone(),
            deadline_at,
            phase: WorkerCommandPhase::Reserved,
            marker: None,
            permit: None,
            release_epoch: None,
            child_id: None,
            native_ready: false,
            startup_failed: false,
            preparation: None,
            routing: None,
            created_at: now,
            updated_at: now,
        };
        self.persist(&self.command_path(identity), &receipt)?;
        Ok(CommandReservation::Created(receipt))
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
        &self,
        slot: &WorkerResultSlot,
        now: i64,
    ) -> Result<WorkerResultReservation, WorkerStoreError> {
        let _guard = self
            .transition
            .lock()
            .map_err(|_| WorkerStoreError::Quarantined)?;
        self.health()?;
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
        let mut removed = 0;
        for slot in slots {
            let command_terminal = commands.iter().any(|c| {
                c.identity == slot.slot.identity.command
                    && matches!(
                        c.phase,
                        WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                    )
            });
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
        for command in commands {
            if matches!(
                command.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) && now.saturating_sub(command.deadline_at.max(command.updated_at))
                >= WORKER_RECEIPT_RETENTION_SECS
                && !remaining_slots
                    .iter()
                    .any(|slot| slot.slot.identity.command == command.identity)
            {
                let proof = drain_proof_path(&self.root, &command.identity);
                match fs::remove_file(&proof) {
                    Ok(()) => File::open(self.root.join("markers"))?.sync_all()?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
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
        let result: Option<WorkerCommandReceipt> =
            read_optional(&self.command_path(identity), MAX_METADATA_BYTES)?;
        if let Some(receipt) = &result {
            if receipt.worker != self.worker
                || receipt.version != STORE_VERSION
                || receipt.identity != *identity
            {
                return Err(WorkerStoreError::Conflict);
            }
            receipt.placement.validate()?;
            if !receipt.placement.matches_worker(&self.worker)
                || !valid_digest(&receipt.command_digest)
                || receipt.created_at < 0
                || receipt.deadline_at <= receipt.created_at
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

    /// Chaque payload actif compte au moins 2 Mio, y compris le temporaire de
    /// son unique producteur. Chaque metadata réserve 64 Kio de remplacement.
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
            self.read_command(&command.identity)?;
            used = used.saturating_add(MAX_METADATA_BYTES as u64 + DRAIN_PROOF_RESERVATION);
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
            used = used.saturating_add(MAX_METADATA_BYTES as u64);
            slots.push(slot);
        }
        let mut workspaces = 0;
        if self.root.join("workspaces").exists() {
            for entry in fs::read_dir(self.root.join("workspaces"))? {
                let path = entry?.path();
                if path.extension().is_none_or(|ext| ext != "json") {
                    continue;
                }
                let receipt: WorkspacePreparationReceipt =
                    read_optional(&path, MAX_METADATA_BYTES)?
                        .ok_or(WorkerStoreError::Quarantined)?;
                validate_prepare_command(&receipt.command)?;
                if receipt.version != STORE_VERSION
                    || receipt.worker != self.worker
                    || !valid_digest(&receipt.prepare_digest)
                    || path != self.workspace_path(&receipt.command)
                {
                    return Err(WorkerStoreError::Quarantined);
                }
                workspaces += 1;
                used = used.saturating_add(MAX_METADATA_BYTES as u64);
            }
        }
        let count = commands.len() + slots.len() + workspaces;
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
            read_optional::<WorkspacePreparationReceipt>(&path, MAX_METADATA_BYTES)?
        {
            if receipt.worker != self.worker
                || receipt.command != *command
                || receipt.prepare_digest != digest
            {
                return Err(WorkerStoreError::Conflict);
            }
            let record = receipt.record.ok_or(WorkerStoreError::Unknown)?;
            crate::workspace_git::report(&record).map_err(|_| WorkerStoreError::Unknown)?;
            return Ok(record);
        }
        let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        else {
            return Err(WorkerRefusal::InvalidEnvelope.into());
        };
        let plan = crate::workspace_git::preflight(root, base_ref.as_deref(), branch.as_deref())
            .map_err(|_| WorkerStoreError::Unknown)?;
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
            record: None,
        };
        {
            let _guard = self
                .transition
                .lock()
                .map_err(|_| WorkerStoreError::Quarantined)?;
            self.admit(2 * MAX_METADATA_BYTES as u64, 1)?;
            private_dir(&self.root.join("workspaces"))?;
            self.persist(&path, &receipt)?;
        }
        // Le maître a réservé sa Task avant ce second Prepare. Aucun bootstrap
        // n'existe encore et aucun verrou métier ne traverse cette E/S Git.
        let record = crate::workspace_git::create(&plan, &command.task_id)
            .map_err(|_| WorkerStoreError::Unknown)?;
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

fn lock_file(path: &Path, create: bool) -> Result<File, WorkerStoreError> {
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
    Ok(file)
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
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
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
fn refuse(_: impl std::fmt::Display) -> WorkerRefusal {
    WorkerRefusal::StoreUnavailable
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

struct HeldRuntime {
    prepared: PreparedRuntime,
    mission_id: String,
    child: MarkedManagedChild,
    ready_at: i64,
    expires: Instant,
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
    host_resources: crate::host_resources::HostResourceCollector,
    last_resource_refresh: Instant,
}

impl WorkerRuntime {
    fn new(config: WorkerRuntimeConfig) -> Result<Self, String> {
        let identity = WorkerIdentity {
            machine_id: config.machine_id.clone(),
            worker_id: config.worker_id.clone(),
            provisioning_revision: config.provisioning_revision,
        };
        identity.validate().map_err(|_| "worker_identity_invalid")?;
        for path in [
            &config.credential_path,
            &config.state_root,
            &config.registry_path,
            &config.master_socket,
        ] {
            if !path.is_absolute() {
                return Err("worker_path_not_absolute".into());
            }
        }
        check_private(&config.state_root, true).map_err(|e| e.to_string())?;
        check_private(
            config
                .credential_path
                .parent()
                .ok_or("worker_credential_parent")?,
            true,
        )
        .map_err(|e| e.to_string())?;
        check_private(&config.credential_path, false).map_err(|e| e.to_string())?;
        let mut secret = String::new();
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&config.credential_path)
            .map_err(|_| "worker_credential_unavailable")?
            .take(4097)
            .read_to_string(&mut secret)
            .map_err(|_| "worker_credential_invalid")?;
        if secret.len() > 4096 || secret.trim().is_empty() {
            return Err("worker_credential_invalid".into());
        }
        let credential = IdentityCredential::new(secret.trim_end_matches(['\r', '\n']).to_owned());
        let registry = AgentRegistry::load_from_path(config.registry_path.clone())?;
        let provision = registry
            .machines()
            .get(&identity.machine_id)
            .ok_or("worker_machine_unknown")?;
        if provision.worker_id != identity.worker_id
            || provision.provisioning_revision != identity.provisioning_revision
            || !provision.matches_credential(&credential)
        {
            return Err("worker_provisioning_mismatch".into());
        }
        let store = WorkerStore::open(&config.state_root.join("store"), identity.clone())
            .map_err(|e| e.to_string())?;
        private_dir(&store.root.join("markers")).map_err(|e| e.to_string())?;
        private_dir(&config.state_root.join("journals")).map_err(|e| e.to_string())?;
        let markers = ManagedMarkerStore::at_directory(store.root.join("markers"));
        let mut source = crate::lifecycle::source_environment();
        let namespace = crate::environment::Namespace::resolve(
            config.master_socket.parent().map(Path::to_path_buf),
            Some(config.master_socket.clone()),
            source.get("HOME").map(PathBuf::from),
        )?;
        for (name, value) in namespace.child_environment() {
            source.insert(name.to_string_lossy().into_owned(), value);
        }
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
            host_resources: crate::host_resources::HostResourceCollector::default(),
            last_resource_refresh: Instant::now(),
        };
        runtime
            .reconcile_all()
            .map_err(|e| format!("worker_reconcile:{e:?}"))?;
        Ok(runtime)
    }

    fn registry(&self) -> Result<AgentRegistry, WorkerRefusal> {
        let registry = AgentRegistry::load_from_path(self.config.registry_path.clone())
            .map_err(|_| WorkerRefusal::Unauthorized)?;
        let provision = registry
            .machines()
            .get(&self.identity.machine_id)
            .ok_or(WorkerRefusal::Unauthorized)?;
        if provision.worker_id != self.identity.worker_id
            || provision.provisioning_revision != self.identity.provisioning_revision
            || !provision.matches_credential(&self.credential)
        {
            return Err(WorkerRefusal::Unauthorized);
        }
        Ok(registry)
    }
    fn protected(&self) -> Vec<PathBuf> {
        vec![
            self.config.credential_path.clone(),
            self.config.credential_path.parent().unwrap().into(),
            self.config.state_root.clone(),
        ]
    }
    fn mapping_for(
        &self,
        request: &WorkerPrepare,
        registry: &AgentRegistry,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        let mapping = self.mapping_authority(request, registry)?;
        self.mapping_paths_and_git(request, mapping, workload)
    }

    fn mapping_authority(
        &self,
        request: &WorkerPrepare,
        registry: &AgentRegistry,
    ) -> Result<ProjectMapping, WorkerRefusal> {
        let provision = registry
            .machine(&self.identity.machine_id)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if request.machine_id != self.identity.machine_id
            || request.provisioning_revision != self.identity.provisioning_revision
            || !provision.permits_agent_type(&request.agent_type)
        {
            return Err(WorkerRefusal::Unauthorized);
        }
        let mapping = registry
            .project_mapping(&request.machine_id, &request.project_id)
            .map_err(|_| WorkerRefusal::PlacementMismatch)?
            .clone();
        if mapping.mapping_revision != request.mapping_revision
            || mapping.source_host != request.origin_host
            || mapping.source_root != Path::new(&request.source_root)
            || mapping.remote_root != Path::new(&request.remote_root)
            || mapping.expected_git_identity != request.source_root_identity
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        // Le côté origine reste une suite de composants approuvée. Aucun stat
        // ni canonicalize sur un chemin appartenant à l'autre machine.
        let origin = mapping.source_root.join(&request.relative_cwd);
        if origin != Path::new(&request.origin_cwd) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok(mapping)
    }

    fn mapping_paths_and_git(
        &self,
        request: &WorkerPrepare,
        mapping: ProjectMapping,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
    ) -> Result<(ProjectMapping, PathBuf), WorkerRefusal> {
        let protected = self.protected();
        let root =
            crate::registry::validate_unprotected_local_path(&mapping.remote_root, &protected)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        let cwd = crate::registry::validate_unprotected_local_path(
            &root.join(&request.relative_cwd),
            &protected,
        )
        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !cwd.starts_with(&root) || !cwd.is_dir() {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        if let Some(workload) = workload {
            let git = Self::git_markers_present(&cwd)?;
            let worktree = matches!(
                &request.workspace,
                Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
            );
            if !git {
                if request.base_commit.is_some()
                    || worktree
                    || workload == bridget_transport::protocol::AutoDelegateWorkload::Compilation
                {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
                // HorsGit attesté : l'autorité est le mapping explicite et les
                // mêmes chemins/protections/droits, jamais un échec Git supposé.
                return Ok((mapping, cwd));
            }
            if request.base_commit.is_none() {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        let origin = crate::workspace_git::run_git(
            &root,
            &[
                "config",
                "--local",
                "--no-includes",
                "--get",
                "remote.origin.url",
            ],
            crate::workspace_git::READ_TIMEOUT,
        )
        .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        if !origin.success()
            || origin.truncated
            || origin.stdout.trim_end_matches(['\r', '\n']) != mapping.expected_git_identity
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        Ok((mapping, cwd))
    }

    /// O(h), h<=256 ancêtres ; une erreur ou un marqueur ambigu reste un refus.
    /// Aucun stderr/code Git ne sert à déduire l'absence de dépôt.
    fn git_markers_present(cwd: &Path) -> Result<bool, WorkerRefusal> {
        for (depth, ancestor) in cwd.ancestors().enumerate() {
            if depth >= 256 {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            match fs::symlink_metadata(ancestor.join(".git")) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(WorkerRefusal::PlacementMismatch);
                }
                Ok(metadata) if metadata.is_file() || metadata.is_dir() => return Ok(true),
                Ok(_) => return Err(WorkerRefusal::PlacementMismatch),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => return Err(WorkerRefusal::PlacementMismatch),
            }
            let mut bare = 0;
            for name in ["HEAD", "objects", "refs"] {
                match fs::symlink_metadata(ancestor.join(name)) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        return Err(WorkerRefusal::PlacementMismatch);
                    }
                    Ok(_) => bare += 1,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(_) => return Err(WorkerRefusal::PlacementMismatch),
                }
            }
            if bare == 3 {
                return Ok(true);
            }
            if bare != 0 {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        Ok(false)
    }

    fn check_routing_connection(
        &self,
        registry: &AgentRegistry,
        request: &WorkerPrepare,
        routing: &RoutingPreparation,
    ) -> Result<(), WorkerRefusal> {
        let policy = registry
            .routing_policy()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        if !policy.enabled
            || !policy
                .machine_policies
                .get(&self.identity.machine_id)
                .is_some_and(|policy| policy.enabled)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        let candidates = registry
            .routing_candidates()
            .ok_or(WorkerRefusal::CapabilityRequired)?;
        if !candidates.iter().any(|entry| {
            entry.enabled
                && entry.machine_id == self.identity.machine_id
                && entry.connection_id == routing.connection_id
                && entry.engine == routing.engine
                && entry.agent_type == request.agent_type
                && entry.model == request.model
                && request
                    .effort
                    .as_ref()
                    .map_or(entry.default_effort.is_none(), |effort| {
                        entry.supported_efforts.contains(effort)
                    })
        }) {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        Ok(())
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
        let (mut policy, effective_posture) =
            crate::native_permissions::project_remote_permissions(
                &request.parent,
                pinned
                    .get(&request.agent_type)
                    .map_err(|_| WorkerRefusal::DefinitionChanged)?,
                mapping,
                cwd,
                &self.source,
                request.posture,
                &self.protected(),
            )
            .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        let scoped = pinned
            .for_inherited_delegation(
                &request.agent_type,
                &request.model,
                request.effort.as_deref(),
                &policy,
            )
            .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        let definition = scoped
            .get(&request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        // Les arguments effectivement contrôlés sont capturés avant le digest
        // de droits. La capture reste une lecture de configuration locale.
        if definition.protocol == "claude_stream_json" {
            policy["launch_context"] =
                crate::native_permissions::capture_context(definition, cwd, &self.source)
                    .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        }
        crate::native_permissions::recheck_remote_child_permissions(
            &policy,
            definition,
            cwd,
            &self.source,
            &self.protected(),
        )
        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
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
    fn probe_workspace_preconditions(
        request: &WorkerPrepare,
        mapping: &ProjectMapping,
        cwd: &Path,
    ) -> Result<(), CandidateProbeRefusal> {
        if let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
            &request.workspace
        {
            let preflight = crate::workspace_git::preflight(
                &mapping.remote_root,
                base_ref.as_deref(),
                branch.as_deref(),
            )
            .map_err(|_| CandidateProbeRefusal::WorkspaceUnavailable)?;
            if request
                .base_commit
                .as_ref()
                .is_some_and(|expected| expected != &preflight.base_commit)
            {
                return Err(CandidateProbeRefusal::WorkspaceUnavailable);
            }
            return Ok(());
        }
        Self::verify_base_commit(cwd, request.base_commit.as_deref())
            .map_err(|_| CandidateProbeRefusal::WorkspaceUnavailable)
    }

    fn probe_one_candidate(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        registry: &AgentRegistry,
    ) -> CandidateProbeVerdict {
        self.probe_one_candidate_for(probe, candidate, registry, None, None)
    }
    fn probe_one_candidate_for(
        &self,
        probe: &CandidateProbe,
        candidate: &CandidateProbeCandidate,
        registry: &AgentRegistry,
        workload: Option<bridget_transport::protocol::AutoDelegateWorkload>,
        git_facts: Option<&mut ProbeGitFacts>,
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
            Self::probe_workspace_preconditions(&request, &mapping, &cwd)?;
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
        let readiness = match self.inspect_candidate(registry, &request, &mapping, &cwd) {
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
        let verdicts = probe
            .candidates
            .iter()
            .map(|candidate| match &registry {
                Ok(registry) => self.probe_one_candidate(probe, candidate, registry),
                Err(_) => CandidateProbeVerdict::Ineligible {
                    candidate_id: candidate.candidate_id.clone(),
                    reason: CandidateProbeRefusal::CapabilityUnavailable,
                },
            })
            .collect();
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
        if probe.worker != self.identity {
            return Err(WorkerRefusal::StaleConnection);
        }
        let registry = self.registry()?;
        let capacity = self.compilation_capacity(&registry);
        let resources = self.host_resources.collect(worker_now(), capacity);
        self.last_resource_refresh = Instant::now();
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
        let verdicts = probe
            .candidates
            .iter()
            .map(|candidate| {
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
                match self.probe_one_candidate_for(
                    &legacy_probe,
                    &legacy_probe.candidates[0],
                    &registry,
                    Some(candidate.workload),
                    Some(&mut git_facts),
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
                }
            })
            .collect();
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
        let expires = Instant::now() + Duration::from_secs(30);
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
            let preflight = crate::workspace_git::preflight(
                &mapping.remote_root,
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
        } else {
            Self::verify_base_commit(&cwd, request.base_commit.as_deref())?;
        }
        let initial = self.inspect_candidate(&registry, request, &mapping, &cwd)?;
        if let Some((_, expected_definition, expected_permission)) = &routing {
            if initial.definition.digest != *expected_definition {
                return Err(WorkerRefusal::DefinitionChanged);
            }
            if json_digest(&initial.child_policy).map_err(refuse)? != *expected_permission {
                return Err(WorkerRefusal::PermissionsChanged);
            }
        }
        if inspect_only {
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
            let record = self
                .store
                .prepare_workspace(command, request, &mapping.remote_root)
                .map_err(refuse)?;
            worktree_base = Some(record.base_commit.clone());
            workspace = Some(serde_json::to_value(&record).map_err(refuse)?);
            let root_offset = mapping
                .remote_root
                .strip_prefix(&record.toplevel)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            mapping.remote_root = Path::new(&record.path).join(root_offset);
            cwd = crate::registry::validate_unprotected_local_path(
                &mapping.remote_root.join(&request.relative_cwd),
                &self.protected(),
            )
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if !cwd.starts_with(&mapping.remote_root) {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        Self::verify_base_commit(&cwd, request.base_commit.as_deref())?;
        // Le second contrôle conserve le verdict exact du cwd de worktree
        // choisi, après que le premier a fermé toute création sur révocation.
        let readiness = self.inspect_candidate(&registry, request, &mapping, &cwd)?;
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

    fn recheck(&self, prepared: &PreparedRuntime) -> Result<AgentRegistry, WorkerRefusal> {
        self.recheck_at(prepared, true)
    }

    fn recheck_at(
        &self,
        prepared: &PreparedRuntime,
        before_release: bool,
    ) -> Result<AgentRegistry, WorkerRefusal> {
        let registry = self.registry()?;
        if let Some(routing) = &prepared.routing {
            self.check_routing_connection(&registry, &prepared.request, routing)?;
        }
        let (mut mapping, mut cwd) = self.mapping_for(
            &prepared.request,
            &registry,
            prepared.routing.as_ref().map(|routing| routing.workload),
        )?;
        if matches!(
            &prepared.request.workspace,
            Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
        ) {
            let receipt: WorkspacePreparationReceipt = read_optional(
                &self.store.workspace_path(&prepared.command),
                MAX_METADATA_BYTES,
            )
            .map_err(refuse)?
            .ok_or(WorkerRefusal::UnknownExecution)?;
            if receipt.worker != self.identity
                || receipt.command != prepared.command
                || receipt.prepare_digest
                    != json_digest(&(&prepared.command, &prepared.request)).map_err(refuse)?
            {
                return Err(WorkerRefusal::CommandMismatch);
            }
            let record = receipt.record.ok_or(WorkerRefusal::UnknownExecution)?;
            let report = crate::workspace_git::report(&record)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if before_release
                && (report.head_commit.as_ref() != Some(&record.base_commit)
                    || report.head_on_branch != Some(true))
            {
                return Err(WorkerRefusal::PlacementMismatch);
            }
            let root_offset = mapping
                .remote_root
                .strip_prefix(&record.toplevel)
                .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            mapping.remote_root = Path::new(&record.path).join(root_offset);
            cwd = crate::registry::validate_unprotected_local_path(
                &mapping.remote_root.join(&prepared.request.relative_cwd),
                &self.protected(),
            )
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
        } else if before_release
            && let Some(base) = prepared.prepared.placement.base_commit.as_deref()
        {
            let observed = crate::workspace_git::run_git(
                &cwd,
                &["rev-parse", "--verify", "--end-of-options", "HEAD^{commit}"],
                crate::workspace_git::READ_TIMEOUT,
            )
            .map_err(|_| WorkerRefusal::PlacementMismatch)?;
            if !observed.success() || observed.truncated || observed.stdout.trim() != base {
                return Err(WorkerRefusal::PlacementMismatch);
            }
        }
        if cwd != Path::new(&prepared.prepared.placement.execution_cwd) {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        let scoped = registry
            .for_inherited_delegation(
                &prepared.request.agent_type,
                &prepared.request.model,
                prepared.request.effort.as_deref(),
                &prepared.prepared.child_policy,
            )
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        if scoped
            .resolved_definition(&prepared.request.agent_type)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?
            != prepared.prepared.definition
        {
            return Err(WorkerRefusal::DefinitionChanged);
        }
        crate::lifecycle::launch_preflight(&scoped, &prepared.request.agent_type, &self.source)
            .map_err(|_| WorkerRefusal::DefinitionChanged)?;
        crate::native_permissions::recheck_remote_child_permissions(
            &prepared.prepared.child_policy,
            scoped
                .get(&prepared.request.agent_type)
                .map_err(|_| WorkerRefusal::DefinitionChanged)?,
            &cwd,
            &self.source,
            &self.protected(),
        )
        .map_err(|_| WorkerRefusal::PermissionsChanged)?;
        Ok(scoped)
    }

    fn start(
        &mut self,
        command: &WorkerCommandIdentity,
        start: &WorkerStart,
        disconnected: &AtomicBool,
    ) -> Result<WorkerResponseOperation, WorkerRefusal> {
        // Même un rejeu durable ne transforme jamais une inspection en Start.
        if start.prepared_token == bridget_transport::protocol::WORKER_INSPECTION_TOKEN {
            return Err(WorkerRefusal::InvalidEnvelope);
        }
        let digest = worker_command_digest(&self.identity, command, start).map_err(refuse)?;
        // Le rejeu durable ne dépend pas d'un token volatil toujours présent.
        match self.store.command(command) {
            Ok(receipt) => {
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
        // Une trame retardée/divergente ne consomme jamais le token courant.
        let prepared = self
            .prepared
            .remove(&command.command_id)
            .ok_or(WorkerRefusal::PrepareExpired)?;
        match self
            .store
            .reserve_command(
                command,
                &digest,
                &start.placement,
                start.deadline_at,
                worker_now(),
            )
            .map_err(refuse)?
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
        // Toute erreur de ce bloc précède une naissance réussie : le helper
        // de spawn ne fait aucune opération faillible après command.spawn().
        let launch_result = (|| {
            let cancel_key = json_digest(command).map_err(refuse)?;
            self.store
                .bind_preparation(command, &prepared)
                .map_err(refuse)?;
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
                OsString::from(serde_json::to_string(&start.child_policy).map_err(refuse)?),
            );
            let sources = start
                .child_policy
                .get("launch_context")
                .map(|value| vec![value.clone()])
                .unwrap_or_default();
            launch.env.insert(
                crate::native_permissions::CHILD_SOURCES_ENV.into(),
                OsString::from(serde_json::to_string(&sources).map_err(refuse)?),
            );
            launch.env.insert(
                crate::wrapper::NATIVE_MISSION_BOOTSTRAP_ENV.into(),
                OsString::from(
                    serde_json::to_string(&crate::wrapper::NativeMissionBootstrap {
                        instance_id: command.instance_id.clone(),
                        mission_id: start.mission_id.clone(),
                    })
                    .map_err(refuse)?,
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
                    .map_err(refuse)?,
                ),
            );
            let executable = std::env::current_exe().map_err(refuse)?;
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
                    serde_json::to_string(&start.definition).map_err(refuse)?,
                ],
                cwd: launch.cwd,
                env: launch.env,
            };
            let child = crate::managed_process::spawn_managed_bootstrap_with_stderr(
                &managed,
                std::process::Stdio::null(),
            )
            .map_err(refuse)?;
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
                return Err(refuse(error));
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
                return Err(refuse(error));
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
                expires: Instant::now() + Duration::from_secs(5),
            },
        );
        Ok(WorkerResponseOperation::ReadyToRelease { ready_at })
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
                || permit.expires_at > held.ready_at.saturating_add(5)
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
            stopped.map_err(refuse)?;
            persisted.map_err(refuse)?;
            return Err(refuse(error));
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
            let exit = child.try_wait().map_err(refuse)?;
            // Lire la preuve après la fin : le wrapper la publie avant sa sortie.
            if exit.is_some() && !self.store.drainage_proven(&receipt).unwrap_or(false) {
                self.store
                    .set_phase(command, WorkerCommandPhase::Unknown, worker_now())
                    .map_err(refuse)?;
                return Ok(Self::observation(WorkerCommandPhase::Unknown, None));
            }
            if exit.is_some() && child.reap_terminated_group().map_err(refuse)? {
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
            if matches!(child.try_status(), Ok(Some(_)) | Err(_)) {
                failed = true;
            }
            let native_ready = !failed && !child.status_is_open() && exit.is_none();
            if failed && !receipt.startup_failed {
                let stopped = child.request_group_termination();
                let persisted = self.store.native_status(command, true);
                stopped.map_err(refuse)?;
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
            if child.try_wait().map_err(refuse)?.is_some() {
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
        let live = crate::managed_process::group_exists(marker.pgid).map_err(refuse)?;
        if !live {
            if !self.store.drainage_proven(&receipt).unwrap_or(false) {
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
        let retained = PreparedRuntime {
            routing: receipt.routing,
            command: prepare_command,
            request,
            prepared,
            expires: Instant::now(),
        };
        if let WorkerResponseOperation::Observation { observation } = &mut response {
            // Le rapport réutilise uniquement le record du canon authentifié, jamais un chemin caller.
            if let Some(value) = retained.prepared.workspace.as_ref() {
                if let Ok(record) =
                    serde_json::from_value::<crate::delegation::WorkspaceRecord>(value.clone())
                    && let Ok(report) = crate::workspace_git::report(&record)
                {
                    observation.workspace = serde_json::to_value(report).ok();
                }
            }
            // Une révocation interdit une nouvelle remise, mais ne masque jamais
            // l'observation technique nécessaire à la clôture et à l'annulation.
            if observation.phase == WorkerCommandPhase::Released
                && observation.native_ready
                && self.recheck_at(&retained, false).is_ok()
            {
                observation.permission_digest = Some(receipt.placement.permission_digest);
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
            let _guard = self.store.transition.lock().map_err(refuse)?;
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
        for command in commands {
            if matches!(
                command.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            ) {
                continue;
            }
            let bootstrap_expired = matches!(
                command.phase,
                WorkerCommandPhase::Reserved
                    | WorkerCommandPhase::Marked
                    | WorkerCommandPhase::ReleaseAuthorized
            ) && command.deadline_at <= worker_now();
            if (command.startup_failed || bootstrap_expired)
                && !matches!(
                    command.phase,
                    WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                )
            {
                match self.cancel(&command.identity) {
                    Ok(_) | Err(WorkerRefusal::UnknownExecution) => {}
                    Err(error) => return Err(error),
                }
            }
            self.reconcile(&command.identity)?;
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
        request.validate()?;
        if request.worker != self.identity
            || request.connection_epoch != epoch
            || disconnected.load(Ordering::Acquire)
        {
            return Err(WorkerRefusal::StaleConnection);
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
            WorkerRequestOperation::Start { start } => {
                self.start(&request.command, start, disconnected)
            }
            WorkerRequestOperation::Release { permit } => {
                self.release(&request.command, permit, epoch, disconnected)
            }
            WorkerRequestOperation::ReserveResultSlot { slot } => {
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
                    .reserve_result_slot(slot, worker_now())
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
                    || receipt.placement.execution_cwd != parent.cwd
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
        let marker = receipt.marker.ok_or(WorkerRefusal::UnknownExecution)?;
        if marker.command_id != command.command_id
            || marker.instance_id != command.instance_id
            || marker.generation != command.generation
            || self.markers.load(&marker_key(command)).map_err(refuse)? != marker
            || crate::managed_process::process_birth(marker.pgid).ok() != Some(marker.birth)
        {
            return Err(WorkerRefusal::UnknownExecution);
        }
        if let Some(child) = self.running.get_mut(&command.command_id) {
            child.request_group_termination().map_err(refuse)?;
        } else {
            self.markers
                .stop_current_group(
                    &marker_key(command),
                    Duration::from_millis(100),
                    Duration::from_millis(10),
                )
                .map_err(refuse)?;
        }
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
                    bytes: serde_json::to_vec(event).map_err(refuse)?,
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

/// Sonde historique sur la même connexion, avant toute variante ou secret162.
/// Le drapeau retourné est une attestation du maître pour proposer la nouvelle
/// enveloppe de capacités ; il ne dépend jamais du build du pair.
fn probe_worker_capability(
    writer: &Mutex<UnixStream>,
    reader: &mut BufReader<UnixStream>,
) -> Result<(bool, bool), String> {
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
            if features.iter().any(|feature| feature == "remote_worker_v1") =>
        {
            Ok((
                features
                    .iter()
                    .any(|feature| feature == "worker_candidate_probe_v1"),
                features
                    .iter()
                    .any(|feature| feature == "worker_candidate_probe_v2"),
            ))
        }
        _ => Err("worker_capability_required".into()),
    }
}

/// Client technique supervisé par l'unité existante. Une reconnexion ne détruit
/// ni processus relâché ni reçu ; seuls les permis en mémoire sont abandonnés.
pub fn run_worker(config: WorkerRuntimeConfig) -> Result<(), String> {
    let mut runtime = WorkerRuntime::new(config)?;
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
        let (master_supports_candidate_probe, master_supports_probe_v2) =
            probe_worker_capability(&writer, &mut reader)?;
        let worker_hello = WorkerHello {
            version: REMOTE_WORKER_VERSION,
            identity: runtime.identity.clone(),
            credential: runtime.credential.clone(),
            build: crate::build_info::BUILD_ID.to_owned(),
            capability: ClientCapability::RemoteWorkerV1,
        };
        let hello = if master_supports_candidate_probe || master_supports_probe_v2 {
            WrapperToDaemon::WorkerHelloWithCapabilities {
                hello: WorkerHelloWithCapabilities {
                    hello: worker_hello,
                    capabilities: [
                        (
                            master_supports_candidate_probe,
                            WorkerCapability::CandidateProbeV1,
                        ),
                        (master_supports_probe_v2, WorkerCapability::CandidateProbeV2),
                    ]
                    .into_iter()
                    .filter_map(|(supported, capability)| supported.then_some(capability))
                    .collect(),
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
        let (epoch, candidate_probe_negotiated, candidate_probe_v2_negotiated) =
            match serde_json::from_slice::<DaemonToWrapper>(&line) {
                Ok(DaemonToWrapper::WorkerAccepted { accepted })
                    if !master_supports_candidate_probe
                        && !master_supports_probe_v2
                        && accepted.validate().is_ok()
                        && accepted.identity == runtime.identity =>
                {
                    (accepted.connection_epoch, false, false)
                }
                Ok(DaemonToWrapper::WorkerAcceptedWithCapabilities { accepted })
                    if (master_supports_candidate_probe || master_supports_probe_v2)
                        && (!accepted
                            .capabilities
                            .contains(&WorkerCapability::CandidateProbeV1)
                            || master_supports_candidate_probe)
                        && accepted.accepted.validate().is_ok()
                        && accepted.accepted.identity == runtime.identity
                        && (!accepted
                            .capabilities
                            .contains(&WorkerCapability::CandidateProbeV2)
                            || master_supports_probe_v2)
                        && accepted.capabilities.iter().all(|capability| {
                            matches!(
                                capability,
                                WorkerCapability::CandidateProbeV1
                                    | WorkerCapability::CandidateProbeV2
                            )
                        }) =>
                {
                    let candidate_probe_negotiated = accepted
                        .capabilities
                        .contains(&WorkerCapability::CandidateProbeV1);
                    (
                        accepted.accepted.connection_epoch,
                        candidate_probe_negotiated,
                        accepted
                            .capabilities
                            .contains(&WorkerCapability::CandidateProbeV2),
                    )
                }
                Ok(DaemonToWrapper::WorkerRejected { .. }) => {
                    return Err("worker_registration_refused".into());
                }
                _ => return Err("worker_handshake_invalid".into()),
            };
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
        let reader_thread = std::thread::spawn(move || {
            while !read_disconnected.load(Ordering::Acquire) {
                let frame = bridget_transport::jsonl::read_unix_line(
                    &mut reader,
                    MAX_WORKER_FRAME_BYTES,
                    bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(5)),
                );
                let inbound = match frame {
                    Ok(Some(bytes)) => match serde_json::from_slice::<DaemonToWrapper>(&bytes) {
                        Ok(DaemonToWrapper::WorkerRequest { request })
                            if request.connection_epoch == read_epoch
                                && request.worker == read_worker
                                && request.validate().is_ok()
                                && (candidate_probe_v2_negotiated
                                    || !matches!(
                                        &request.request,
                                        WorkerRequestOperation::PrepareV2 { .. }
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
                        _ => break,
                    },
                    _ => break,
                };
                if matches!(
                    &inbound,
                    WorkerInbound::Request(WorkerRequest {
                        request: WorkerRequestOperation::Cancel {
                            target: WorkerCancelScope::Command {}
                        },
                        ..
                    })
                ) {
                    let WorkerInbound::Request(request) = &inbound else {
                        unreachable!();
                    };
                    if let Ok(key) = json_digest(&request.command) {
                        if let Ok(mut cancelled) = cancelled.lock() {
                            if cancelled.len() >= MAX_WORKER_STORE_ENTRIES {
                                break;
                            }
                            cancelled.insert(key);
                        } else {
                            break;
                        }
                    }
                }
                if sender.try_send(inbound).is_err() {
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
                    let result = match runtime.candidate_probe(&probe) {
                        Ok(result) => result,
                        Err(_) => {
                            // Un probe hors contrat ou hors époque a déjà fermé
                            // le lecteur. Ne jamais répondre par une admission
                            // partielle ni tenter une reconnexion locale.
                            fatal = Some("worker_candidate_probe_invalid".into());
                            break;
                        }
                    };
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
                    let result = match runtime.candidate_probe_v2(&probe) {
                        Ok(result) => result,
                        Err(_) => {
                            fatal = Some("worker_candidate_probe_v2_invalid".into());
                            break;
                        }
                    };
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
    use bridget_transport::protocol::{RemoteResultDeferredReason, RemoteResultDisposition};
    use std::os::unix::fs::symlink;

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
        drop(store);
        let store = fixture.open();
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
            runtime.start(&command(), &start, &disconnected).unwrap(),
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
                .start(&command(), &start, &disconnected)
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
                .start(&command(), &start, &AtomicBool::new(false))
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
                .start(&refused_command, &start, &AtomicBool::new(false))
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
                    vec!["remote_worker_v1".to_string()]
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
            drop(writer);
            drop(reader);
            peer.join().unwrap();
        }
    }
}
