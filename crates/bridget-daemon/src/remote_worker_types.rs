//! Erreur typée du magasin de worker distant (extraite de `remote_worker.rs`).

use bridget_transport::protocol::WorkerRefusal;
use std::io;

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
