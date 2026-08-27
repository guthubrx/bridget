//! Garde RAII du thread superviseur managed.
//!
//! Propriété : à la sortie du scope (fin de test, panic, interruption cargo),
//! le canal commandes est fermé et le thread superviseur est rejoint —
//! jamais laissé détaché avec un `Sender` vivant.

use log::warn;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::daemon::{
    spawn_managed_supervisor_thread, DaemonConfig, ManagedSupervisorCommand,
    ManagedSupervisorEvent,
};
use crate::fleet::FleetSupervisor;

/// Possède le `Sender` canonique et le `JoinHandle` du superviseur managed.
///
/// Déclarer **après** les clones du sender (ex. `DaemonState`) pour que le Drop
/// libère le canal une fois les autres détenteurs relâchés.
pub(crate) struct ManagedSupervisorGuard {
    sender: Option<Sender<ManagedSupervisorCommand>>,
    join: Option<JoinHandle<()>>,
}

impl ManagedSupervisorGuard {
    pub(crate) fn from_existing_sender(
        sender: Sender<ManagedSupervisorCommand>,
        receiver: mpsc::Receiver<ManagedSupervisorCommand>,
        fleet: Arc<FleetSupervisor>,
        config: &DaemonConfig,
        events: Sender<ManagedSupervisorEvent>,
        executable_override: Option<PathBuf>,
    ) -> Self {
        let join = spawn_managed_supervisor_thread(
            fleet,
            config,
            receiver,
            events,
            executable_override,
        );
        Self {
            sender: Some(sender),
            join: Some(join),
        }
    }

    pub(crate) fn start_with_executable(
        fleet: Arc<FleetSupervisor>,
        config: &DaemonConfig,
        events: Sender<ManagedSupervisorEvent>,
        executable_override: Option<PathBuf>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let join = spawn_managed_supervisor_thread(
            fleet,
            config,
            receiver,
            events,
            executable_override,
        );
        Self {
            sender: Some(sender),
            join: Some(join),
        }
    }

    pub(crate) fn sender(&self) -> &Sender<ManagedSupervisorCommand> {
        self.sender
            .as_ref()
            .expect("ManagedSupervisorGuard consommé")
    }

    /// Ferme le canal puis attend la fin du thread (hors Drop automatique).
    pub(crate) fn shutdown(mut self) {
        self.sender.take();
        self.join_supervisor();
    }

    fn join_supervisor(&mut self) {
        let Some(join) = self.join.take() else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(3);
        while !join.is_finished() {
            if Instant::now() >= deadline {
                warn!(
                    "ManagedSupervisorGuard: le superviseur n'a pas quitté sous 3 s \
                     après fermeture du canal"
                );
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = join.join();
    }
}

impl Drop for ManagedSupervisorGuard {
    fn drop(&mut self) {
        self.sender.take();
        self.join_supervisor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desired_state::DesiredStateStore;

    fn minimal_config(root: &std::path::Path) -> DaemonConfig {
        std::fs::create_dir_all(root).unwrap();
        DaemonConfig {
            socket_path: root.join("bridget.sock"),
            db_path: root.join("bridget.db"),
            log_path: root.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        }
    }

    fn empty_fleet(config: &DaemonConfig) -> Arc<FleetSupervisor> {
        let desired = DesiredStateStore::at_path(
            config
                .db_path
                .parent()
                .unwrap()
                .join("desired-state.json"),
        );
        Arc::new(
            FleetSupervisor::open(
                &config.db_path,
                desired,
                crate::fleet::FleetConfig::from_env(),
            )
            .unwrap(),
        )
    }

    fn wait_join_finished(join: JoinHandle<()>, label: &str) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !join.is_finished() {
            assert!(
                Instant::now() < deadline,
                "{label}: le superviseur doit quitter après fermeture du canal"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let _ = join.join();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_managed_supervisor_guard_libere_le_canal_a_la_sortie() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ms-guard-ok-{}-{}",
            std::process::id(),
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        let config = minimal_config(&root);
        let fleet = empty_fleet(&config);
        let (events_tx, _events_rx) = mpsc::channel();
        {
            let guard = ManagedSupervisorGuard::start_with_executable(
                fleet,
                &config,
                events_tx,
                None,
            );
            let extra = guard.sender().clone();
            drop(extra);
            guard.shutdown();
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_cycle_normal_joint_le_superviseur() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ms-guard-cycle-{}-{}",
            std::process::id(),
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        let config = minimal_config(&root);
        let fleet = empty_fleet(&config);
        let (events_tx, _events_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let join = spawn_managed_supervisor_thread(fleet, &config, rx, events_tx, None);
        let extra = tx.clone();
        drop(extra);
        drop(tx);
        wait_join_finished(join, "TEMOIN cycle normal");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_sender_survivant_tue_TEMOIN_managed_supervisor_guard() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ms-guard-mut-{}-{}",
            std::process::id(),
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        let config = minimal_config(&root);
        let fleet = empty_fleet(&config);
        let (events_tx, _events_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let join = spawn_managed_supervisor_thread(fleet, &config, rx, events_tx, None);
        thread::sleep(Duration::from_millis(100));
        assert!(
            !join.is_finished(),
            "mutant: tant que le sender vit, le superviseur reste actif"
        );
        drop(tx);
        wait_join_finished(join, "mutant nettoyé");
        let _ = std::fs::remove_dir_all(&root);
    }
}
