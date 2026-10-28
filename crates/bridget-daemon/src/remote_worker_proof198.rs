//! Preuves privées annexes198. Les dossiers `commands` et `workspaces` restent
//! strictement au format183 : ce module garde ici ce que l'ancien lecteur
//! ne sait pas relire. Une preuve seule, sans reçu, interdit tout nouveau Start.
use super::*;

const PROOF_DIR: &str = "start198";

/// Délai, placement et date d'origine d'un fence, tels que reçus du maître.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct FenceProof {
    pub version: u16,
    pub worker: WorkerIdentity,
    pub identity: WorkerCommandIdentity,
    pub placement: RemotePlacement,
    pub deadline_at: i64,
    pub created_at: i64,
    pub digest: String,
}

/// Lien d'une préparation de reprise vers le fence précédent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct BindProof {
    pub version: u16,
    pub worker: WorkerIdentity,
    pub command: WorkerCommandIdentity,
    pub prepare_digest: String,
    pub previous: WorkerCommandIdentity,
}

pub(super) fn fence_digest(
    worker: &WorkerIdentity,
    identity: &WorkerCommandIdentity,
    placement: &RemotePlacement,
    deadline_at: i64,
) -> Result<String, WorkerStoreError> {
    json_digest(&(
        "pre_start_fence_v1",
        worker,
        identity,
        placement,
        deadline_at,
    ))
}

/// Un reçu format183 exige deadline_at > created_at.
pub(super) fn legacy_deadline(deadline_at: i64, created_at: i64) -> i64 {
    deadline_at.max(created_at.saturating_add(1))
}

fn dir(root: &Path) -> PathBuf {
    root.join(PROOF_DIR)
}
pub(super) fn ensure_dir(root: &Path) -> Result<(), WorkerStoreError> {
    private_dir(&dir(root))
}
pub(super) fn fence_path(root: &Path, identity: &WorkerCommandIdentity) -> PathBuf {
    dir(root).join(format!("fence-{}.json", marker_key(identity)))
}
pub(super) fn bind_path(root: &Path, command: &WorkerCommandIdentity) -> PathBuf {
    dir(root).join(format!("bind-{}.json", marker_key(command)))
}

pub(super) fn read_fence(
    root: &Path,
    worker: &WorkerIdentity,
    identity: &WorkerCommandIdentity,
) -> Result<Option<FenceProof>, WorkerStoreError> {
    let Some(proof) = read_optional::<FenceProof>(&fence_path(root, identity), MAX_METADATA_BYTES)?
    else {
        return Ok(None);
    };
    proof.placement.validate()?;
    if proof.version != STORE_VERSION
        || proof.worker != *worker
        || proof.identity != *identity
        || !proof.placement.matches_worker(worker)
        || proof.deadline_at <= 0
        || proof.created_at < 0
        || proof.digest != fence_digest(worker, identity, &proof.placement, proof.deadline_at)?
    {
        return Err(WorkerStoreError::Quarantined);
    }
    Ok(Some(proof))
}

pub(super) fn read_bind(
    root: &Path,
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
) -> Result<Option<BindProof>, WorkerStoreError> {
    let Some(proof) = read_optional::<BindProof>(&bind_path(root, command), MAX_METADATA_BYTES)?
    else {
        return Ok(None);
    };
    if proof.version != STORE_VERSION
        || proof.worker != *worker
        || proof.command != *command
        || !valid_digest(&proof.prepare_digest)
        || proof.previous.validate().is_err()
        || proof.previous.task_id != command.task_id
        || proof.previous.command_id == command.command_id
    {
        return Err(WorkerStoreError::Quarantined);
    }
    Ok(Some(proof))
}

/// Relie le reçu format183 à sa preuve : le fence retrouve son délai d'origine.
/// Sans preuve, le reçu est un refus ordinaire et garde la règle183.
pub(super) fn overlay_fence(
    root: &Path,
    worker: &WorkerIdentity,
    receipt: &mut WorkerCommandReceipt,
) -> Result<(), WorkerStoreError> {
    let Some(proof) = read_fence(root, worker, &receipt.identity)? else {
        return Ok(());
    };
    if receipt.command_digest != proof.digest
        || receipt.placement != proof.placement
        || receipt.created_at != proof.created_at
        || receipt.deadline_at != legacy_deadline(proof.deadline_at, proof.created_at)
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
        return Err(WorkerStoreError::Quarantined);
    }
    receipt.pre_start_fence = Some(proof.digest);
    receipt.deadline_at = proof.deadline_at;
    Ok(())
}

pub(super) fn read_workspace_receipt(
    root: &Path,
    worker: &WorkerIdentity,
    command: &WorkerCommandIdentity,
) -> Result<Option<WorkspacePreparationReceipt>, WorkerStoreError> {
    let path = root
        .join("workspaces")
        .join(format!("{}.json", marker_key(command)));
    let Some(mut receipt) =
        read_optional::<WorkspacePreparationReceipt>(&path, MAX_METADATA_BYTES)?
    else {
        return Ok(None);
    };
    overlay_bind(root, worker, &mut receipt)?;
    Ok(Some(receipt))
}

pub(super) fn overlay_bind(
    root: &Path,
    worker: &WorkerIdentity,
    receipt: &mut WorkspacePreparationReceipt,
) -> Result<(), WorkerStoreError> {
    let Some(proof) = read_bind(root, worker, &receipt.command)? else {
        return Ok(());
    };
    if receipt.record.is_some() || receipt.prepare_digest != proof.prepare_digest {
        return Err(WorkerStoreError::Quarantined);
    }
    receipt.fenced_previous = Some(proof.previous);
    Ok(())
}

/// Entrées annexes bornées par la même quota ; les noms inconnus mettent en quarantaine.
pub(super) fn count(root: &Path, worker: &WorkerIdentity) -> Result<usize, WorkerStoreError> {
    let directory = dir(root);
    match fs::symlink_metadata(&directory) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
        Ok(_) => check_private(&directory, true)?,
    }
    let mut total = 0usize;
    for entry in fs::read_dir(&directory)? {
        let path = entry?.path();
        total += 1;
        if total > MAX_WORKER_STORE_ENTRIES {
            return Err(WorkerStoreError::Quota);
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let valid = if name.starts_with("fence-") {
            read_optional::<FenceProof>(&path, MAX_METADATA_BYTES)?.is_some_and(|p| {
                read_fence(root, worker, &p.identity)
                    .ok()
                    .flatten()
                    .is_some()
                    && fence_path(root, &p.identity) == path
            })
        } else if name.starts_with("bind-") {
            read_optional::<BindProof>(&path, MAX_METADATA_BYTES)?.is_some_and(|p| {
                read_bind(root, worker, &p.command).ok().flatten().is_some()
                    && bind_path(root, &p.command) == path
            })
        } else {
            false
        };
        if !valid {
            return Err(WorkerStoreError::Quarantined);
        }
    }
    Ok(total)
}
