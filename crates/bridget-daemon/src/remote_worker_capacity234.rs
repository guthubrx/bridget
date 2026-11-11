//! Reprise froide du seul défaut de borne des réservations de vues232.
//! Aucun reçu, marqueur, dossier de travail ou processus n'est modifié.
use super::*;

const LEGACY_RESERVATION_MAX: u64 = 8 * 1024 * 1024 + 16 * 1024 + 5120;

fn require_stopped(marker: &ManagedMarker) -> Result<(), WorkerStoreError> {
    if marker.pgid == 0 || marker.birth == 0 || crate::managed_process::group_exists(marker.pgid)? {
        return Err(WorkerStoreError::Unknown);
    }
    Ok(())
}

impl WorkerStore {
    pub(super) fn recover_legacy_view_capacity(
        root: &Path,
        worker: WorkerIdentity,
    ) -> Result<serde_json::Value, WorkerStoreError> {
        worker.validate()?;
        if !root.is_absolute() {
            return Err(WorkerStoreError::Unknown);
        }
        let parent = root.parent().ok_or(WorkerStoreError::Unknown)?;
        check_private(parent, true)?;
        let key = digest_bytes(root.as_os_str().as_encoded_bytes());
        // Même verrou que le worker : un détenteur vivant interdit la reprise.
        let lock = lock_file(&parent.join(format!(".worker-{key}.lock")), false)?;
        let enrollment = parent.join(format!(".worker-{key}.enrolled"));
        let before: Enrollment =
            read_optional(&enrollment, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Quarantined)?;
        if before.version != STORE_VERSION || before.worker != worker || !before.quarantined {
            return Err(WorkerStoreError::Conflict);
        }
        let store = Self {
            root: root.into(),
            enrollment,
            worker,
            _lock: lock,
            transition: Mutex::new(()),
        };
        // Validation intégrale par le lecteur corrigé. Aucun health contourné
        // pour lancer une commande : le magasin reste en quarantaine ici.
        let (commands, slots, _, _) = store.inventory()?;
        if commands.iter().any(|receipt| {
            !matches!(
                receipt.phase,
                WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
            )
        }) {
            return Err(WorkerStoreError::InvalidTransition);
        }
        for receipt in &commands {
            if let Some(marker) = &receipt.marker {
                require_stopped(marker)?;
            }
        }
        let mut witnesses = Vec::new();
        for (index, entry) in fs::read_dir(root.join("workspaces"))?.enumerate() {
            if index >= MAX_WORKER_STORE_ENTRIES {
                return Err(WorkerStoreError::Quota);
            }
            let path = entry?.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            // inventory a validé ces reçus ; relire sans lever la quarantaine.
            let receipt: WorkspacePreparationReceipt =
                read_optional(&path, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Quarantined)?;
            if receipt.worker != store.worker || path != store.workspace_path(&receipt.command) {
                return Err(WorkerStoreError::Conflict);
            }
            let Some(view) = receipt.commit_view.as_ref() else {
                continue;
            };
            view.validate(&receipt)?;
            let marker = store.marker_for_view(&receipt)?;
            if let Some(marker) = &marker {
                require_stopped(marker)?;
            }
            if view.source_attempted && !view.source_stopped && marker.is_none() {
                return Err(WorkerStoreError::Unknown);
            }
            if let Some(operation) = &view.materialization {
                let marker = store.marker_for_materialization(&receipt)?;
                if let Some(marker) = &marker {
                    require_stopped(marker)?;
                }
                if operation.attempted && !operation.stopped && marker.is_none() {
                    return Err(WorkerStoreError::Unknown);
                }
            }
            if view.reserved_bytes > LEGACY_RESERVATION_MAX
                && view.reserved_bytes == crate::workspace_git::MAX_VIEW_RESERVATION_BYTES
                && view.planning.as_ref().is_some_and(|plan| !plan.adopted)
                && receipt.record.is_none()
                && !view.closed
                && !view.source_attempted
                && !view.source_stopped
                && marker.is_none()
            {
                witnesses.push(receipt.command.task_id);
            }
        }
        if witnesses.is_empty() {
            return Err(WorkerStoreError::InvalidTransition);
        }
        // Inclure les marqueurs orphelins, sans les retirer ni les réconcilier.
        for (directory, views) in [(root.join("markers"), false), (store.view_root(), true)] {
            if !directory.exists() {
                continue;
            }
            check_private(&directory, true)?;
            for entry in fs::read_dir(directory)? {
                let path = entry?.path();
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or(WorkerStoreError::Unknown)?;
                if path.extension().is_none_or(|ext| ext != "json")
                    || (views && !name.starts_with("view-"))
                {
                    continue;
                }
                let marker: ManagedMarker =
                    read_optional(&path, MAX_METADATA_BYTES)?.ok_or(WorkerStoreError::Unknown)?;
                require_stopped(&marker)?;
            }
        }
        // Le verrou exclut le worker ; tous les producteurs connus sont arrêtés.
        // Relire les invariants avant la seule écriture autorisée.
        store.inventory()?;
        let current: Enrollment = read_optional(&store.enrollment, MAX_METADATA_BYTES)?
            .ok_or(WorkerStoreError::Quarantined)?;
        if current.version != before.version
            || current.worker != before.worker
            || !current.quarantined
        {
            return Err(WorkerStoreError::Conflict);
        }
        let backup = parent.join(format!(
            ".worker-{key}.before-view-capacity-{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        if backup.exists() {
            return Err(WorkerStoreError::Conflict);
        }
        write_json(&backup, &before)?;
        write_json(
            &store.enrollment,
            &Enrollment {
                version: STORE_VERSION,
                worker: store.worker.clone(),
                quarantined: false,
            },
        )?;
        if let Err(error) = store.health() {
            let _ = store.quarantine();
            return Err(error);
        }
        Ok(
            serde_json::json!({"status":"recovered","scope":"this_store",
            "cause":"legacy_view_reservation_limit","witness_tasks":witnesses,
            "commands_validated":commands.len(),"slots_validated":slots.len(),
            "backup":backup,"receipts_unchanged":true,"processes_started":0}),
        )
    }
}

pub(crate) fn recover_view_capacity(
    config: &WorkerRuntimeConfig,
) -> Result<serde_json::Value, String> {
    // Les mêmes fichiers privés, l'identité provisionnée et son credential.
    let (worker, _, _) = read::inputs(config)?;
    WorkerStore::recover_legacy_view_capacity(&config.state_root.join("store"), worker)
        .map_err(|error| format!("view_capacity_recovery_refused:{error}"))
}
