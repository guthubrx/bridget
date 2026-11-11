//! Réponses tardives de préparation : corrélation éphémère, aucune autorité de tâche.
use bridget_transport::protocol::{
    MAX_WORKER_STORE_ENTRIES, WorkerCommandIdentity, WorkerIdentity, WorkerRequestOperation,
    WorkerResponse, WorkerResponseOperation,
};
use std::{collections::HashMap, time::Instant};

struct Retired {
    connection: String,
    epoch: String,
    worker: WorkerIdentity,
    command: WorkerCommandIdentity,
    retired_at: Instant,
}

#[derive(Default)]
pub(super) struct Cache {
    entries: HashMap<String, Retired>,
}

/// Seul Prepare initial est sans commande Fleet. Une reprise fenced reste exclue.
pub(super) fn initial_prepare(
    command: &WorkerCommandIdentity,
    operation: &WorkerRequestOperation,
) -> bool {
    if command.validate_preparation().is_err() || command.command_id != command.task_id {
        return false;
    }
    match operation {
        WorkerRequestOperation::Prepare { .. } | WorkerRequestOperation::PrepareV2 { .. } => true,
        WorkerRequestOperation::PrepareDiagnosed { prepare } => initial_prepare(command, prepare),
        _ => false,
    }
}

impl Cache {
    pub(super) fn remember(
        &mut self,
        correlation: String,
        connection: &str,
        epoch: &str,
        worker: &WorkerIdentity,
        command: &WorkerCommandIdentity,
    ) {
        if self.entries.len() >= MAX_WORKER_STORE_ENTRIES
            && let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.retired_at)
                .map(|(k, _)| k.clone())
        {
            self.entries.remove(&oldest);
        }
        self.entries.insert(
            correlation,
            Retired {
                connection: connection.into(),
                epoch: epoch.into(),
                worker: worker.clone(),
                command: command.clone(),
                retired_at: Instant::now(),
            },
        );
    }

    pub(super) fn forget_connection(&mut self, connection: &str) {
        self.entries.retain(|_, e| e.connection != connection);
    }

    /// Appelé après validation de l'enveloppe et de l'époque active. Ne rend aucun résultat au demandeur.
    pub(super) fn ignores(&self, connection: &str, response: &WorkerResponse) -> bool {
        matches!(
            response.response,
            WorkerResponseOperation::Rejected { .. }
                | WorkerResponseOperation::WorkspaceRejected { .. }
                | WorkerResponseOperation::CommitViewRejected { .. }
        ) && self.entries.get(&response.correlation_id).is_some_and(|e| {
            e.connection == connection
                && e.epoch == response.connection_epoch
                && e.worker == response.worker
                && e.command == response.command
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridget_transport::protocol::{REMOTE_WORKER_VERSION, WorkerRefusal};

    fn response() -> WorkerResponse {
        WorkerResponse {
            version: REMOTE_WORKER_VERSION,
            correlation_id: "expired-prepare".into(),
            connection_epoch: "epoch".into(),
            worker: WorkerIdentity {
                machine_id: "cartae".into(),
                worker_id: "worker-cartae".into(),
                provisioning_revision: 1,
            },
            command: WorkerCommandIdentity {
                command_id: "prepare".into(),
                task_id: "prepare".into(),
                instance_id: String::new(),
                generation: 0,
            },
            response: WorkerResponseOperation::Rejected {
                reason: WorkerRefusal::PrepareExpired,
            },
        }
    }

    #[test]
    fn spec240_retired_prepare_requires_the_exact_issued_tuple() {
        let r = response();
        let mut cache = Cache::default();
        assert!(!cache.ignores("worker", &r));
        cache.remember(
            r.correlation_id.clone(),
            "worker",
            &r.connection_epoch,
            &r.worker,
            &r.command,
        );
        assert!(cache.ignores("worker", &r));
        assert!(!cache.ignores("other", &r));
        for field in 0..6 {
            let mut bad = r.clone();
            match field {
                0 => bad.correlation_id = "unknown".into(),
                1 => bad.connection_epoch = "old".into(),
                2 => bad.worker.provisioning_revision += 1,
                3 => bad.command.task_id = "other".into(),
                4 => bad.command.instance_id = "process".into(),
                _ => bad.command.generation += 1,
            }
            assert!(!cache.ignores("worker", &bad));
        }
        let mut released = r.clone();
        released.response = WorkerResponseOperation::Released {};
        assert!(!cache.ignores("worker", &released));
        cache.forget_connection("worker");
        assert!(!cache.ignores("worker", &r));
    }

    #[test]
    fn spec240_retired_prepare_history_is_bounded_and_keeps_newest() {
        let mut cache = Cache::default();
        let r = response();
        for i in 0..=MAX_WORKER_STORE_ENTRIES {
            cache.remember(
                i.to_string(),
                "worker",
                &r.connection_epoch,
                &r.worker,
                &r.command,
            );
        }
        assert_eq!(cache.entries.len(), MAX_WORKER_STORE_ENTRIES);
        assert!(!cache.entries.contains_key("0"));
        assert!(
            cache
                .entries
                .contains_key(&MAX_WORKER_STORE_ENTRIES.to_string())
        );
    }
}
