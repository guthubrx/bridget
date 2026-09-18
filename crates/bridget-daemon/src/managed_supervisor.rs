//! Garde RAII du thread superviseur managed.
//!
//! Propriété : à la sortie du scope (fin de test, panic, interruption cargo),
//! un ordre d'arrêt explicite est envoyé et le thread superviseur est rejoint —
//! même si un autre `Sender` vit encore dans l'état du daemon.

use log::warn;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::daemon::{
    DaemonConfig, ManagedSupervisorCommand, ManagedSupervisorEvent, spawn_managed_supervisor_thread,
};
use crate::fleet::FleetSupervisor;

use crate::execution_store::{ContinuationReservation, ExecutionStore};
use crate::fleet::{AutonomyBudgetPolicy, AutonomyRuntimeState, evaluate_autonomy_budget};
use bridget_transport::protocol::{ControlStateFrame, ExecutionBudgetOutcome};

/// Traduit l'état de contrôle du référent (SPEC-087) en précondition de
/// continuation. La pause globale est une pause d'autonomie : la garde de
/// continuation rend `ExecutionBudgetOutcome::Paused`, vocabulaire déjà
/// défini par SPEC-064, sans nouvelle limite.
pub fn autonomy_runtime_for_control(
    control: &ControlStateFrame,
    observed: AutonomyRuntimeState,
) -> AutonomyRuntimeState {
    match crate::referent_control::admit_autonomous_effect(
        crate::referent_control::AutonomousEffect::Continuation,
        control,
    ) {
        crate::referent_control::Admission::Deferred { .. } => AutonomyRuntimeState::Paused,
        crate::referent_control::Admission::Admitted => observed,
    }
}

/// Verdict de la garde de continuation. Une limite est une issue Bridget :
/// elle ne modifie aucune délégation ni objectif le service compagnon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernedContinuation {
    Reserved,
    Budget(ExecutionBudgetOutcome),
    Reservation(ContinuationReservation),
    MissingFacts,
}

/// Source attestée d'une demande de continuation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernedContinuationSource {
    /// Le parent a déjà atteint un état terminal.
    InactiveParent,
    /// Le wrapper s'est reconnecté sans tour actif ; son état SQLite peut être
    /// encore actif jusqu'à la reconstruction atomique.
    RecoveryAfterIdleWrapper,
}

/// Point unique de réservation d une continuation gouvernée. Les faits sont
/// relus dans la même passe de contrôle puis la réservation SQLite atomique
/// refuse une course entre deux continuations ou un tour concurrent.
#[allow(clippy::too_many_arguments)]
pub fn reserve_governed_continuation(
    store: &ExecutionStore,
    policy: AutonomyBudgetPolicy,
    runtime: AutonomyRuntimeState,
    source: GovernedContinuationSource,
    parent_execution_id: &str,
    expected_generation: u64,
    expected_revision: u64,
    continuation_id: &str,
    proof_idle_at: i64,
    observed_at: i64,
) -> rusqlite::Result<GovernedContinuation> {
    let Some(facts) = store.execution_budget_facts(parent_execution_id, observed_at)? else {
        return Ok(GovernedContinuation::MissingFacts);
    };
    if let Some(outcome) = evaluate_autonomy_budget(policy, &facts, runtime) {
        return Ok(GovernedContinuation::Budget(outcome));
    }
    Ok(
        match match source {
            GovernedContinuationSource::InactiveParent => store.reserve_continuation_if_idle(
                parent_execution_id,
                expected_generation,
                expected_revision,
                continuation_id,
                proof_idle_at,
                observed_at,
            ),
            GovernedContinuationSource::RecoveryAfterIdleWrapper => store
                .reserve_recovery_continuation_after_idle_wrapper(
                    parent_execution_id,
                    expected_generation,
                    expected_revision,
                    continuation_id,
                    proof_idle_at,
                    observed_at,
                ),
        }? {
            ContinuationReservation::Reserved => GovernedContinuation::Reserved,
            reservation => GovernedContinuation::Reservation(reservation),
        },
    )
}
/// Possède le `Sender` canonique et le `JoinHandle` du superviseur managed.
///
/// La garde envoie `Shutdown` avant de déposer son sender : l'arrêt ne dépend
/// donc pas du cycle de vie des clones détenus par `DaemonState` ou ses threads.
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
        let join =
            spawn_managed_supervisor_thread(fleet, config, receiver, events, executable_override);
        Self {
            sender: Some(sender),
            join: Some(join),
        }
    }

    #[cfg(test)]
    pub(crate) fn start_with_executable(
        fleet: Arc<FleetSupervisor>,
        config: &DaemonConfig,
        events: Sender<ManagedSupervisorEvent>,
        executable_override: Option<PathBuf>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let join =
            spawn_managed_supervisor_thread(fleet, config, receiver, events, executable_override);
        Self {
            sender: Some(sender),
            join: Some(join),
        }
    }
    #[cfg(test)]
    pub(crate) fn sender(&self) -> &Sender<ManagedSupervisorCommand> {
        self.sender
            .as_ref()
            .expect("ManagedSupervisorGuard consommé")
    }

    fn request_shutdown(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(ManagedSupervisorCommand::Shutdown);
        }
    }

    /// Ordonne l'arrêt puis attend la fin du thread (hors Drop automatique).
    pub(crate) fn shutdown(mut self) {
        self.request_shutdown();
        self.join_supervisor();
    }

    /// Délai laissé au superviseur pour sortir de lui-même après la fermeture
    /// du canal. Au-delà, on l'ABANDONNE : le processus se termine juste après.
    const JOIN_DEADLINE: Duration = Duration::from_secs(3);

    /// Attente **réellement** bornée de la fin du superviseur.
    ///
    /// L'échéance précédente était décorative : elle sortait de la boucle, elle
    /// journalisait, puis la ligne suivante appelait `join.join()` SANS
    /// CONDITION — une attente sans échéance. La pile d'un daemon bloqué le
    /// montrait mot pour mot : `__pthread_clockjoin_ex` avec `abstime=0x0`,
    /// attendant le TID du superviseur, lui-même vivant dans son `recv_timeout`.
    /// Le daemon avait pourtant reçu SIGTERM, lu son drapeau et fait tout son
    /// arrêt propre : il mourait à la dernière ligne, et devenait un orphelin
    /// que plus aucun signal capturable n'atteignait.
    ///
    /// Abandonner ce thread est sûr ICI et seulement ici : on est sur le chemin
    /// de terminaison du processus. Le superviseur n'écrit rien qui ne soit
    /// transactionnel, et le chemin nominal (canal fermé) le laisse finir sa
    /// commande en cours avant de sortir — l'abandon n'est qu'un filet.
    fn join_supervisor(&mut self) {
        let Some(join) = self.join.take() else {
            return;
        };
        let deadline = Instant::now() + Self::JOIN_DEADLINE;
        while !join.is_finished() {
            if Instant::now() >= deadline {
                warn!(
                    "ManagedSupervisorGuard: le superviseur n'a pas quitté sous {} s \
                     après fermeture du canal — abandonné pour ne pas bloquer l'arrêt",
                    Self::JOIN_DEADLINE.as_secs()
                );
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = join.join();
    }
}

impl Drop for ManagedSupervisorGuard {
    fn drop(&mut self) {
        self.request_shutdown();
        self.join_supervisor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desired_state::DesiredStateStore;
    use std::sync::atomic::{AtomicBool, Ordering};

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
        let desired =
            DesiredStateStore::at_path(config.db_path.parent().unwrap().join("desired-state.json"));
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
    fn TEMOIN_managed_supervisor_guard_arrete_meme_avec_un_sender_survivant() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ms-guard-ok-{}-{}",
            std::process::id(),
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        let config = minimal_config(&root);
        let fleet = empty_fleet(&config);
        let (events_tx, _events_rx) = mpsc::channel();
        {
            let guard =
                ManagedSupervisorGuard::start_with_executable(fleet, &config, events_tx, None);
            // Reproduit le sender détenu par DaemonState pendant son arrêt.
            // Sans ordre explicite, le superviseur ne verrait jamais Disconnect.
            let extra = guard.sender().clone();
            guard.shutdown();
            drop(extra);
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

    /// Le garde-fou d'arrêt doit TENIR son échéance.
    ///
    /// Avant correctif, `join_supervisor` sortait de sa boucle bornée, émettait
    /// un avertissement, puis appelait `join.join()` SANS CONDITION : l'attente
    /// redevenait infinie juste après avoir annoncé une échéance. Un daemon réel
    /// restait alors vivant après SIGTERM, orphelin et hors d'atteinte de tout
    /// signal capturable ; sa pile montrait `__pthread_clockjoin_ex` avec
    /// `abstime=0x0` — aucune échéance là où le code croyait en avoir une.
    ///
    /// Mutant qui tue ce test : remettre `let _ = join.join();` après le `return`
    /// → `shutdown` ne rend jamais la main, le `recv_timeout` ci-dessous expire
    /// et l'assertion meurt en affichant le délai réellement attendu.
    #[test]
    fn shutdown_tient_son_echeance_meme_si_le_superviseur_ne_sort_pas() {
        // Superviseur qui refuse de sortir tant que le test ne le libère pas.
        let libere = Arc::new(AtomicBool::new(false));
        let observe = Arc::clone(&libere);
        let (sender, _receiver) = mpsc::channel::<ManagedSupervisorCommand>();
        let join = thread::spawn(move || {
            while !observe.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
            }
        });
        let guard = ManagedSupervisorGuard {
            sender: Some(sender),
            join: Some(join),
        };

        let (fini_tx, fini_rx) = mpsc::channel();
        let debut = Instant::now();
        let appelant = thread::spawn(move || {
            guard.shutdown();
            let _ = fini_tx.send(());
        });

        // Marge généreuse au-dessus de l'échéance annoncée : on éprouve qu'elle
        // est TENUE, pas qu'elle est rapide.
        let verdict = fini_rx.recv_timeout(ManagedSupervisorGuard::JOIN_DEADLINE * 3);
        let ecoule = debut.elapsed();

        // Libérer AVANT d'assertir : un échec ne doit pas laisser de thread pendu.
        libere.store(true, Ordering::SeqCst);
        let _ = appelant.join();

        assert!(
            verdict.is_ok(),
            "shutdown doit rendre la main sous {} s ; toujours bloqué après {} ms",
            (ManagedSupervisorGuard::JOIN_DEADLINE * 3).as_secs(),
            ecoule.as_millis()
        );
        assert!(
            ecoule >= ManagedSupervisorGuard::JOIN_DEADLINE,
            "l'échéance ne doit pas être écourtée : {} ms écoulées pour {} s annoncées",
            ecoule.as_millis(),
            ManagedSupervisorGuard::JOIN_DEADLINE.as_secs()
        );
    }

    /// Contrôle positif : un superviseur qui SORT est rejoint normalement, et
    /// bien avant l'échéance. Sans lui, un `shutdown` qui abandonnerait
    /// systématiquement sans jamais rejoindre passerait le test ci-dessus.
    #[test]
    fn shutdown_rejoint_normalement_un_superviseur_qui_sort() {
        let (sender, receiver) = mpsc::channel::<ManagedSupervisorCommand>();
        let join = thread::spawn(move || {
            // Sort dès que tous les senders sont tombés — le chemin nominal.
            while receiver.recv_timeout(Duration::from_millis(10)).is_ok() {}
        });
        let guard = ManagedSupervisorGuard {
            sender: Some(sender),
            join: Some(join),
        };
        let debut = Instant::now();
        guard.shutdown();
        let ecoule = debut.elapsed();
        assert!(
            ecoule < ManagedSupervisorGuard::JOIN_DEADLINE,
            "le chemin nominal ne doit pas attendre l'échéance : {} ms",
            ecoule.as_millis()
        );
    }
}

/// SPEC-087 T014 : la pause du référent se traduit par `Paused` dans la garde
/// de continuation, sans nouvelle limite. Aucun appelant de production ne
/// réserve encore de continuation gouvernée sur main : la garde attend son
/// producteur, et ce test la tient prête.
#[cfg(test)]
mod control_pause_tests {
    use super::*;
    use crate::execution_store::ExecutionBudgetFacts;

    fn control(paused: bool) -> ControlStateFrame {
        ControlStateFrame {
            version: bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION,
            generation: 1,
            paused,
            paused_since: None,
            paused_by: None,
            pause_reason: None,
            auto_objectives_cap: 5,
            agent_posture: Some(bridget_transport::protocol::AgentPosture::Complete),
            auto_reassignment: Some(true),
            updated_at: 0,
        }
    }

    #[test]
    fn la_pause_globale_rend_la_continuation_paused() {
        let facts = ExecutionBudgetFacts {
            duration_secs: 1,
            descendants: 0,
            usage: None,
        };
        let runtime = autonomy_runtime_for_control(&control(true), AutonomyRuntimeState::Ready);
        assert_eq!(runtime, AutonomyRuntimeState::Paused);
        assert_eq!(
            evaluate_autonomy_budget(AutonomyBudgetPolicy::default(), &facts, runtime),
            Some(ExecutionBudgetOutcome::Paused)
        );
        let runtime = autonomy_runtime_for_control(&control(false), AutonomyRuntimeState::Ready);
        assert_eq!(runtime, AutonomyRuntimeState::Ready);
        assert_eq!(
            evaluate_autonomy_budget(AutonomyBudgetPolicy::default(), &facts, runtime),
            None
        );
        // Une terminaison observée n'est jamais effacée par l'absence de pause.
        assert_eq!(
            autonomy_runtime_for_control(&control(false), AutonomyRuntimeState::Terminated),
            AutonomyRuntimeState::Terminated
        );
    }
}
