//! Daemon bridget — écoute sur socket locale Unix, route les messages
//! entre les wrappers connectés, persiste l'état en SQLite.

mod dispatch_diagnostics;
mod late_worker_prepares;
mod native_child_requests;
mod native_delegation;
mod native_provider_catalogue;
pub(crate) use native_delegation::catalogue_entries177;
mod agent_tracking;
mod mobile_admin;
mod mobile_artifacts;
mod mobile_operations;
mod mobile_questions;
mod mobile_queue;
mod mobile_tasks;
mod native_lineage;
mod origin_proof;
mod specialist_requests;
mod user_input;
mod wrapper_dispatch;

use bridget_core::{CircuitBreaker, Deduplicator, EnvelopeGuard, Router, RouterAction};
use bridget_transport::greffe_authorization::{
    GreffeAuthorizationGate, GreffeDepositAuthorization, GreffeMutationAction,
};
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::{
    ARTIFACT_READ_VERSION, ArtifactReadOutcome, ArtifactReadRefusal, ArtifactReadRequest,
};
use bridget_transport::protocol::{
    AttachRefusal, CLIENT_CONTRACT_VERSION, COORDINATION_EVENTS_VERSION,
    COORDINATION_STREAM_VERSION, ClientCapability, ClientRefusal, ConnectionRole,
    DelegatedRuntimeEventFrame, DiskSpaceFact, ExecutionControlCommand, ExecutionControlOperation,
    ExecutionControlOutcome, ExecutionControlRefusal, ExecutionDeliveryContext, IdempotencyIssue,
    PresenceMode, ProjectAdminOutcome, ProjectAdminRequest, ProjectBindOutcome, ProjectBindRequest,
    ProjectBindStatus, ProjectProfileOutcome, ProjectProfileRefusal, ProjectProfileRequest,
    ProjectRegistryRefusal, ProjectRoundOutcome, ProjectRoundRefusal, ProjectRoundRequest,
    ProjectRuntimeOutcome, ProjectRuntimeRefusal, ProjectRuntimeRequest,
    REVIEW_DELEGATE_CONTRACT_VERSION, RelaunchOutcome, RuntimeIngressRefusal,
    SERVICE_CONTRACT_VERSION, ServiceCapability, ServiceRefusal, SpawnRefusal, StopOutcome, decode,
    encode,
};
use bridget_transport::protocol::{
    ProjectReference, ProjectRoundDispatchOutcome, ProjectRoundDispatchRequest,
};
use bridget_transport::{ChannelReport, DaemonToWrapper, ResolvedAgentDefinition, WrapperToDaemon};
use log::{error, info, warn};
use std::collections::{HashMap, HashSet, VecDeque};
#[cfg(test)]
use std::io::BufRead;
use std::io::{BufReader, BufWriter, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::artifact_policy::ArtifactPolicy;
use crate::artifact_service::{ArtifactService, ArtifactServiceError};
use crate::artifact_store::{ArtifactPersistResult, ArtifactPublicationContext};
use crate::artifact_types::{ARTIFACT_CONTRACT_VERSION, ArtifactPublicationV1};
use crate::communication::canonical_send;
use crate::execution_store::{
    ConditionalTransition, ControlCommandStatus, ControlReservation, ExecutionRecoveryOutcome,
    ExecutionStore, ExecutionUsageSample, ProviderBindingOutcome,
};
use crate::idempotency::{
    DelegatedRuntimeEventInput, DeliveryExecutionLink, IdempotencyKey, IdempotencyStore,
    LookupResult, OperationKind, ReplyTracking, Reservation, SendDelivery,
};
use crate::managed_supervisor::{
    GovernedContinuation, GovernedContinuationSource, ManagedSupervisorGuard,
    autonomy_runtime_for_control, reserve_governed_continuation,
};
use crate::store::{
    GuichetCoordinationEvent, GuichetDeposit, GuichetLifecycleEvent, GuichetNext,
    GuichetReplyInput, GuichetResult, MAX_GUICHET_FRAME_BYTES, Store, StoreError,
};
use crate::{
    desired_state::{DesiredLifecycleState, DesiredStateStore},
    fleet::{
        AdoptStoppedResult, FleetConfig, FleetSupervisor, SpawnLease, SpawnOrder as FleetSpawnOrder,
    },
    lifecycle::{
        PreparedSpawn, SourceEnvironment, SpawnDecision, prepare_recovery, source_environment,
        submit_relaunch_from_resolved, submit_spawn_from_resolved,
    },
    managed_process::{
        ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStatus, ManagedStderrStore,
        ManagedStopResult, RunningManagedChild, spawn_managed_bootstrap_with_stderr,
    },
    recovery_trace::{
        REASON_ABSENT_FROM_FLEET, REASON_FROZEN_DEFINITION, REASON_NON_PERSISTENT, REASON_QUOTA,
        REASON_RECOVERY_FAILED, RecoveryLossEntry,
    },
    registry::{AgentRegistry, DEFAULT_NOTIFY_TIMEOUT_SECS},
};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use uuid::Uuid;

// Métriques du daemon (M-005)
/// Une interaction humaine non marquée reply doit tout de même obtenir une
/// issue rapide : sinon un steer peut rester muet pendant 45 minutes.
const HUMAN_STEER_TIMEOUT_SECS: u64 = 30;
/// Identité unique du relais UI local. Le service est singleton : une nouvelle
/// génération doit pouvoir remplacer proprement une socket morte ou devenue
/// muette après un redémarrage, sans jamais ouvrir cette reprise aux agents.
const UI_HUMAN_AGENT_ID: &str = "550e8400-e29b-41d4-a716-4466554400f0";
/// Libellé conversationnel de l'opérateur dans le ledger. Ce n'est pas un
/// agent routable : le relais UI s'enregistre sous `UI_HUMAN_AGENT_ID`.
const UI_HUMAN_SENDER: &str = "humain";
pub struct Metrics {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub errors: AtomicU64,
    pub active_connections: AtomicUsize,
    pub max_connections: AtomicUsize,
    pub execution_admitted: AtomicU64,
    pub execution_started: AtomicU64,
    pub execution_transition_rejected: AtomicU64,
    pub execution_delivery_failures: AtomicU64,
    pub execution_queue_saturated: AtomicU64,
    pub execution_start_latency_secs_total: AtomicU64,
    pub execution_start_latency_samples: AtomicU64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            max_connections: AtomicUsize::new(0),
            execution_admitted: AtomicU64::new(0),
            execution_started: AtomicU64::new(0),
            execution_transition_rejected: AtomicU64::new(0),
            execution_delivery_failures: AtomicU64::new(0),
            execution_queue_saturated: AtomicU64::new(0),
            execution_start_latency_secs_total: AtomicU64::new(0),
            execution_start_latency_samples: AtomicU64::new(0),
        }
    }

    pub fn increment_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_connections(&self) {
        let n = self.active_connections.fetch_add(1, Ordering::Relaxed) + 1;
        if self.max_connections.fetch_max(n, Ordering::Relaxed) < n && n.is_power_of_two() {
            info!("connexions actives : {n} (nouveau maximum)");
        }
    }

    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_execution_admitted(&self) {
        self.execution_admitted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_execution_started(&self, latency_secs: u64) {
        self.execution_started.fetch_add(1, Ordering::Relaxed);
        self.execution_start_latency_samples
            .fetch_add(1, Ordering::Relaxed);
        self.execution_start_latency_secs_total
            .fetch_add(latency_secs, Ordering::Relaxed);
    }

    pub fn record_execution_transition_rejected(&self) {
        self.execution_transition_rejected
            .fetch_add(1, Ordering::Relaxed);
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_execution_delivery_failure(&self) {
        self.execution_delivery_failures
            .fetch_add(1, Ordering::Relaxed);
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_execution_queue_saturated(&self) {
        self.execution_queue_saturated
            .fetch_add(1, Ordering::Relaxed);
    }
}

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
const MAX_DAEMON_WAITS: usize = 64;

/// Une réservation O(1), libérée sur chaque sortie, erreur ou panique.
struct WaitPermit(Arc<std::sync::atomic::AtomicUsize>);

impl WaitPermit {
    fn acquire(active: &Arc<std::sync::atomic::AtomicUsize>) -> Result<Self, String> {
        active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_DAEMON_WAITS).then_some(n + 1)
            })
            .map_err(|_| "wait_capacity".to_string())?;
        Ok(Self(active.clone()))
    }
}

impl Drop for WaitPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Sonde sans consommation et sans changer O_NONBLOCK sur la connexion partagée.
fn wait_client_gone(peer: Option<&UnixStream>) -> bool {
    let Some(peer) = peer else { return false };
    let fd = peer.as_raw_fd();
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    // HUP couvre aussi une fermeture qui laisse une trame suivante dans le socket.
    // SAFETY: `pfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
    if unsafe { libc::poll(&mut pfd, 1, 0) } > 0
        && pfd.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0
    {
        return true;
    }
    let mut byte = 0u8;
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    let read = unsafe {
        libc::recv(
            fd,
            (&mut byte as *mut u8).cast(),
            1,
            libc::MSG_PEEK | libc::MSG_DONTWAIT,
        )
    };
    read == 0
        || (read < 0
            && !std::io::Error::last_os_error()
                .raw_os_error()
                .is_some_and(|e| e == libc::EAGAIN || e == libc::EWOULDBLOCK || e == libc::EINTR))
}

/// O(log n) par snapshot indexé ; même compteur de capacité que les tâches.
fn wait_request(
    conn: &str,
    id: &str,
    timeout: u64,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
    slice: Duration,
) -> DaemonToWrapper {
    let result = (|| -> Result<DaemonToWrapper, String> {
        if id.is_empty() || id.len() > 128 || id.chars().any(bridget_core::is_disallowed_control) {
            return Err("invalid_request".into());
        }
        if !(1..=3600).contains(&timeout) {
            return Err("wait_timeout_out_of_range".into());
        }
        let start = Instant::now();
        let deadline = start + Duration::from_secs(timeout);
        let (identity, epoch, active) = {
            let st = state.lock().recover_poison();
            (
                live_connection_identity(&st, conn).ok_or("identity_unavailable")?,
                st.delegation_store.wait_epoch(),
                st.active_waits.clone(),
            )
        };
        let mut permit = None;
        loop {
            let seen = epoch.get();
            if wait_client_gone(peer) || SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                return Err("daemon_unreachable".into());
            }
            let (current, reply_deadline, summary) = {
                let st = state.lock().recover_poison();
                if live_connection_identity(&st, conn).as_ref() != Some(&identity) {
                    return Err("request_unavailable".into());
                }
                let (current, reply_deadline, summary) = st
                    .store
                    .request_wait_snapshot(id, &identity.0)
                    .map_err(|_| "store_unavailable")?
                    .ok_or("request_unavailable")?;
                let frozen = if current == "open" {
                    native_delegation::request_frozen_secs(&st, id, unix_timestamp())?.unwrap_or(0)
                } else {
                    0
                };
                if permit.is_none() {
                    permit = Some(WaitPermit::acquire(&active)?);
                }
                (
                    current,
                    reply_deadline.saturating_add(frozen as i64),
                    summary,
                )
            };
            // L'échéance métier est calculée : cette lecture ne ferme jamais la ligne.
            let now = unix_timestamp();
            let current = if current == "open" && reply_deadline <= now {
                "timed_out".into()
            } else {
                current
            };
            if current != "open" || Instant::now() >= deadline {
                let reply = (current == "answered")
                    .then(|| {
                        summary
                            .as_deref()
                            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                            .filter(serde_json::Value::is_object)
                    })
                    .flatten();
                let reply_reason = (current == "answered" && reply.is_none())
                    .then(|| "reply_unavailable".to_string());
                return Ok(DaemonToWrapper::RequestWaitResult {
                    timed_out_wait: current == "open",
                    state: current,
                    waited_secs: start.elapsed().as_secs(),
                    reply,
                    reply_reason,
                });
            }
            let until_reply = Duration::from_secs(reply_deadline.saturating_sub(now) as u64);
            epoch.wait_changed(
                seen,
                slice
                    .min(deadline.saturating_duration_since(Instant::now()))
                    .min(until_reply),
            );
        }
    })();
    result.unwrap_or_else(|code| DaemonToWrapper::NativeDelegationResult {
        result: serde_json::json!({"status":"refused","code":code}),
    })
}

const PRESENCE_RETENTION: Duration = Duration::from_secs(300);
/// Seuil provisoire : un quart de la période du tour (1 s). À ajuster après mesure réelle (programme 260, D7).
const NATIVE_TICK_SLOW: Duration = Duration::from_millis(250);
/// Code de refus partagé entre l'émetteur et les comparaisons de texte (programme 260, lot 8).
const CROSS_PROJECT_REASON_REQUIRED: &str = "cross_project_reason_required";

#[cfg(test)]
mod native151_sql_trace_tests {
    use super::*;

    struct TraceContext {
        start: Instant,
        prefix: &'static [u8],
        selects: Mutex<Vec<Duration>>,
    }

    pub(super) struct SelectTrace {
        connection: *mut rusqlite::ffi::sqlite3,
        context: Box<TraceContext>,
    }

    impl SelectTrace {
        pub(super) fn new(connection: &rusqlite::Connection, prefix: &'static [u8]) -> Self {
            unsafe extern "C" fn observe(
                event: u32,
                context: *mut std::ffi::c_void,
                _statement: *mut std::ffi::c_void,
                sql: *mut std::ffi::c_void,
            ) -> i32 {
                if event != rusqlite::ffi::SQLITE_TRACE_STMT || context.is_null() || sql.is_null() {
                    return rusqlite::ffi::SQLITE_OK;
                }
                // Aucun unwind ne traverse la frontière C. SQLite garantit la chaîne SQL.
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                    let context = unsafe { &*context.cast::<TraceContext>() };
                    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                    let sql = unsafe { std::ffi::CStr::from_ptr(sql.cast()) }.to_bytes();
                    if sql.starts_with(context.prefix) {
                        context
                            .selects
                            .lock()
                            .recover_poison()
                            .push(context.start.elapsed());
                    }
                }));
                rusqlite::ffi::SQLITE_OK
            }
            let mut trace = Self {
                // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                connection: unsafe { connection.handle() },
                context: Box::new(TraceContext {
                    start: Instant::now(),
                    prefix,
                    selects: Mutex::new(Vec::new()),
                }),
            };
            // Le Box garde la même adresse jusqu'à la désinstallation dans Drop.
            assert_eq!(
                // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                unsafe {
                    rusqlite::ffi::sqlite3_trace_v2(
                        trace.connection,
                        rusqlite::ffi::SQLITE_TRACE_STMT,
                        Some(observe),
                        (&mut *trace.context as *mut TraceContext).cast(),
                    )
                },
                rusqlite::ffi::SQLITE_OK
            );
            trace
        }

        pub(super) fn assert_calm(&self, label: &str) {
            let times = self.context.selects.lock().unwrap().clone();
            eprintln!(
                "SC008 {label} : SELECT pertinents={} aux millisecondes {:?}",
                times.len(),
                times.iter().map(Duration::as_millis).collect::<Vec<_>>()
            );
            assert_eq!(times.len(), 3, "{label}: {times:?}");
            assert!(times[0] < Duration::from_millis(500), "{times:?}");
            assert!(times[1] - times[0] >= Duration::from_secs(5), "{times:?}");
            assert!(times[1] <= Duration::from_secs(6), "{times:?}");
            assert!(times[2] >= Duration::from_secs(10), "{times:?}");
            assert!(times[2] <= Duration::from_secs(12), "{times:?}");
        }
    }

    impl Drop for SelectTrace {
        fn drop(&mut self) {
            // Les tests gardent l'état propriétaire vivant au-delà de ce garde.
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                rusqlite::ffi::sqlite3_trace_v2(self.connection, 0, None, std::ptr::null_mut());
            }
        }
    }

    #[test]
    fn native151_sc008_request_wait_select_calme_cinq_secondes() {
        let label = format!(
            "n151-rq-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        );
        let (mut st, config) = presence_tests::state_with_registered_agent(&label);
        let (writer, _reader) = presence_tests::control_socket(&label);
        st.connections.insert("conn-1".into(), writer);
        let owner = live_connection_identity(&st, "conn-1").unwrap().0;
        st.store
            .create_request("calm151", &owner, "target", 3600)
            .unwrap();
        let before = st.store.request_wait_snapshot("calm151", &owner).unwrap();
        let epoch = st.delegation_store.wait_epoch();
        let seen = epoch.get();
        let active = st.active_waits.clone();
        let shared = Arc::new(Mutex::new(st));
        let trace = SelectTrace::new(
            shared.lock().unwrap().store.connection(),
            b"SELECT state, deadline_at, reply_summary",
        );
        let result = wait_request(
            "conn-1",
            "calm151",
            10,
            &shared,
            None,
            Duration::from_secs(5),
        );
        assert!(
            matches!(result,
            DaemonToWrapper::RequestWaitResult { ref state, timed_out_wait: true, waited_secs: 10, reply: None, reply_reason: None }
                if state == "open"),
            "{result:?}"
        );
        trace.assert_calm("RequestWait");
        drop(trace);
        assert_eq!(epoch.get(), seen);
        assert_eq!(active.load(Ordering::Acquire), 0);
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .store
                .request_wait_snapshot("calm151", &owner)
                .unwrap(),
            before
        );
        eprintln!(
            "SC008 RequestWait : epoch inchangé={seen}, capacité libérée, échéance métier 3600 s et état open conservés"
        );
        drop(shared);
        let _ = std::fs::remove_file(config.db_path);
    }
}

/// Pas de scrutation des threads de fond pendant leur sieste.
///
/// Une sieste longue n'est pas seulement un délai de réaction : tant qu'un
/// thread de fond dort, il retient son clone de `Arc<Mutex<DaemonState>>`, donc
/// l'état, donc le `Sender` du superviseur — et l'arrêt du daemon reste
/// suspendu à son réveil.
const SHUTDOWN_POLL: Duration = Duration::from_millis(100);

/// Dort au plus `total`, en se réveillant pour voir si l'arrêt est demandé.
///
/// Rend `true` quand l'arrêt est demandé : l'appelant doit alors SORTIR de sa
/// boucle et laisser tomber sa référence à l'état.
fn sleep_until_shutdown(total: Duration) -> bool {
    sleep_until_flag(total, &SHUTDOWN_REQUESTED)
}

/// Le drapeau est un paramètre : l'oracle éprouve la boucle sans toucher au
/// statique global, que d'autres tests du même binaire partageraient.
fn sleep_until_flag(total: Duration, flag: &AtomicBool) -> bool {
    let deadline = Instant::now() + total;
    loop {
        if flag.load(Ordering::SeqCst) {
            return true;
        }
        let reste = deadline.saturating_duration_since(Instant::now());
        if reste.is_zero() {
            return false;
        }
        thread::sleep(reste.min(SHUTDOWN_POLL));
    }
}

#[cfg(test)]
mod arret_tests {
    use super::{SHUTDOWN_POLL, SHUTDOWN_REQUESTED, sleep_until_flag, sleep_until_shutdown};
    use bridget_transport::poison::RecoverPoison;
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    const ADAPTATEUR_CHILD_ENV: &str = "BRIDGET_SHUTDOWN_ADAPTER_CHILD";

    /// Sérialise les tests qui manipulent le drapeau GLOBAL d'arrêt.
    static VERROU_DRAPEAU: Mutex<()> = Mutex::new(());

    /// Un thread de fond en sieste longue doit LÂCHER sa référence à l'état dès
    /// l'ordre d'arrêt. Sans cela, l'état — et le `Sender` qu'il contient —
    /// survit jusqu'au réveil, et l'arrêt du daemon attend d'autant.
    ///
    /// Mutant qui tue ce test : revenir à `thread::sleep(total)` sans scrutation
    /// → l'attente dure la sieste entière (3 s ici) et l'assertion sur le délai
    /// mesuré meurt en affichant la valeur.
    #[test]
    fn une_sieste_longue_est_interrompue_par_l_ordre_d_arret() {
        let flag = Arc::new(AtomicBool::new(false));
        let observe = Arc::clone(&flag);
        let sieste = Duration::from_secs(3);
        let debut = Instant::now();
        let dormeur = std::thread::spawn(move || sleep_until_flag(sieste, &observe));
        std::thread::sleep(SHUTDOWN_POLL * 2);
        flag.store(true, Ordering::SeqCst);
        let interrompu = dormeur.join().expect("le dormeur ne panique pas");
        let ecoule = debut.elapsed();

        assert!(
            interrompu,
            "la sieste doit rendre true quand l'arrêt est demandé"
        );
        assert!(
            ecoule < sieste / 2,
            "l'arrêt doit interrompre la sieste : {} ms écoulées pour une sieste de {} ms",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }

    /// Contrôle positif : sans ordre d'arrêt, la sieste va bien à son terme et
    /// rend `false`. Sans lui, une fonction qui rendrait TOUJOURS `true`
    /// passerait le test ci-dessus.
    #[test]
    fn sans_ordre_d_arret_la_sieste_va_a_son_terme() {
        let flag = AtomicBool::new(false);
        let sieste = SHUTDOWN_POLL * 3;
        let debut = Instant::now();
        let interrompu = sleep_until_flag(sieste, &flag);
        let ecoule = debut.elapsed();
        assert!(!interrompu, "aucun ordre d'arrêt : la sieste rend false");
        assert!(
            ecoule >= sieste,
            "la sieste ne doit pas être écourtée : {} ms pour {} ms demandées",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }

    /// LE CHEMIN REEL, celui que les fils de fond empruntent.
    ///
    /// Les deux oracles ci-dessus éprouvent `sleep_until_flag`, le helper
    /// GÉNÉRIQUE. Or les fils rappels et purge appellent `sleep_until_shutdown`,
    /// l'ADAPTATEUR qui le lie au drapeau global. Remplacer le seul adaptateur
    /// par `thread::sleep(total)` les rendait insensibles à l'arrêt sans faire
    /// rougir quoi que ce soit : les unités gardaient une couche VOISINE du
    /// chemin de production, et le raccord n'était gardé par rien.
    ///
    /// Mutant qui tue ce test : `fn sleep_until_shutdown(total) { thread::sleep(total); false }`
    /// → l'appel rend `false` après la sieste entière, les deux assertions
    /// meurent en affichant le délai mesuré.
    #[test]
    fn l_adaptateur_des_fils_de_fond_observe_le_drapeau_d_arret() {
        // D'autres tests appellent `daemon::run` dans ce même binaire. L'oracle
        // qui positionne le drapeau global vit donc dans un processus dédié :
        // il garde le vrai adaptateur sans arrêter les daemons des voisins.
        let mut child = Command::new(std::env::current_exe().expect("binaire de test courant"))
            .arg("--exact")
            .arg("daemon::arret_tests::adaptateur_d_arret_child")
            .arg("--ignored")
            .arg("--nocapture")
            .env(ADAPTATEUR_CHILD_ENV, "1")
            .spawn()
            .expect("lancer l'oracle d'arrêt isolé");
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = child.try_wait().expect("attente oracle d'arrêt") {
                break status;
            }
            if Instant::now() >= deadline {
                let pid = child.id() as libc::pid_t;
                // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
                let sent = unsafe { libc::kill(pid, libc::SIGTERM) };
                let cleanup_deadline = Instant::now() + Duration::from_secs(2);
                while child
                    .try_wait()
                    .expect("nettoyage oracle d'arrêt")
                    .is_none()
                    && Instant::now() < cleanup_deadline
                {
                    std::thread::sleep(Duration::from_millis(10));
                }
                assert_eq!(sent, 0, "TERM de l'oracle d'arrêt isolé a échoué");
                panic!("l'oracle d'arrêt isolé a dépassé son budget de 5 s");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "l'oracle d'arrêt isolé a échoué: {status}"
        );
    }

    #[test]
    #[ignore = "processus enfant lancé par un autre test (variable ADAPTATEUR_CHILD_ENV) ; sans effet lancé seul"]
    fn adaptateur_d_arret_child() {
        if std::env::var(ADAPTATEUR_CHILD_ENV).ok().as_deref() != Some("1") {
            return;
        }
        let _ordre = VERROU_DRAPEAU.lock().recover_poison();
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

        let sieste = Duration::from_secs(3);
        let debut = Instant::now();
        let dormeur = std::thread::spawn(move || sleep_until_shutdown(sieste));
        std::thread::sleep(SHUTDOWN_POLL * 2);
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        let interrompu = dormeur.join().expect("le dormeur ne panique pas");
        let ecoule = debut.elapsed();

        // Rendre le drapeau à son état de repos AVANT d'assertir : un échec ne
        // doit pas laisser le binaire de test avec un arrêt demandé.
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

        assert!(
            interrompu,
            "l'adaptateur doit rendre true quand l'arrêt est demandé —              il a rendu false après {} ms",
            ecoule.as_millis()
        );
        assert!(
            ecoule < sieste / 2,
            "ARRÊT PRÉCOCE attendu : {} ms écoulées pour une sieste de {} ms",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }

    /// Contrôle positif de l'adaptateur : sans ordre d'arrêt, il dort jusqu'au
    /// bout et rend `false`. Sans lui, un adaptateur qui rendrait TOUJOURS
    /// `true` passerait le test ci-dessus.
    #[test]
    fn l_adaptateur_sans_ordre_d_arret_va_au_bout_de_sa_sieste() {
        let _ordre = VERROU_DRAPEAU.lock().recover_poison();
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

        let sieste = SHUTDOWN_POLL * 3;
        let debut = Instant::now();
        let interrompu = sleep_until_shutdown(sieste);
        let ecoule = debut.elapsed();

        assert!(!interrompu, "aucun ordre d'arrêt : l'adaptateur rend false");
        assert!(
            ecoule >= sieste,
            "la sieste ne doit pas être écourtée : {} ms pour {} ms demandées",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }
}

/// Gestionnaires de signaux du **service** daemon uniquement.
///
/// Sous `cfg(test)`, no-op volontaire : un binaire de test qui hériterait de
/// ces handlers (SIGTERM → drapeau sans `_exit`, SIGINT/SIGHUP ignorés)
/// devient ininterruptible. Les suites interrompues laissent alors des
/// orphelins adoptés par launchd, sourds à tout signal propre.
#[cfg(test)]
fn install_daemon_signal_handlers() {}

#[cfg(not(test))]
fn install_daemon_signal_handlers() {
    // Seul SIGTERM déclenche le shutdown propre (c'est ce que launchd/systemd envoie).
    // SIGINT (Ctrl+C) est ignoré en mode daemon — l'utilisateur doit utiliser
    // launchctl stop ou kill -TERM pour arrêter le daemon.
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    unsafe {
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
            eprintln!(
                "[BRIDGET] *** SIGTERM REÇU *** à {}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        });
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGINT, || {
            eprintln!(
                "[BRIDGET] *** SIGINT REÇU (ignoré) *** à {}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
        });
        // Ignorer SIGHUP (envoyé quand le terminal se ferme)
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGHUP, || {
            eprintln!("[BRIDGET] *** SIGHUP REÇU (ignoré) ***");
        });
    }
}

/// Retain : horloge **lien** seule (`link_seen`). Aucun état — y compris
/// `connected` — n'est immortel (lot B, composé avec A). La capacité
/// (`capacity_seen`) reste exposée par `who` (`last_seen_secs`) mais ne
/// décide plus du retain.
fn presence_within_retention(presence: &Presence) -> bool {
    presence.link_seen.elapsed() <= PRESENCE_RETENTION
}

/// Mutant lot B : l'ancienne exemption `connected`. Hors chemin de production
/// pour prouver que la restaurer cacherait le défaut (relec1 / relec5).
#[cfg(test)]
fn presence_within_retention_mutant_exempt_connected(presence: &Presence) -> bool {
    presence.state == "connected" || presence.link_seen.elapsed() <= PRESENCE_RETENTION
}
const ATTACH_VIEW_BUFFER_BYTES: usize = 1024 * 1024;
const ATTACH_VIEW_WRITE_TIMEOUT: Duration = Duration::from_secs(1);
/// Plafond global (toutes connexions attach confondues) des accusés tardifs.
/// Chaque entrée expire aussi après `PENDING_ATTACH_SEND_TTL`.
const MAX_PENDING_ATTACH_SENDS: usize = 1024;
const PENDING_ATTACH_SEND_TTL: Duration = Duration::from_secs(300);
const MANAGED_STOP_COOPERATIVE_GRACE: Duration = Duration::from_millis(500);
const MANAGED_STOP_FORCED_GRACE: Duration = Duration::from_secs(1);
const MANAGED_STOP_POLL: Duration = Duration::from_millis(20);
const MANAGED_STOP_REPLY_TIMEOUT: Duration = Duration::from_secs(3);
const MANAGED_RECOVERY_DEADLINE_SECS: i64 = 30;
const CLIENT_IDEMPOTENCY_HORIZON_SECS: i64 = 7 * 24 * 60 * 60;
const CLIENT_ISSUED_AT_TOLERANCE_SECS: i64 = 60;
const MAX_ACTIVE_ISSUER_SCOPES: usize = 4096;

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

// Constante pour la période de grâce des timeouts (M-004)
const TIMEOUT_GRACE_PERIOD: u64 = 30; // secondes

/// Plancher d'échéance quand un humain est de la partie (émetteur OU
/// destinataire).
///
/// Mesure du 28/08 sur `tracked_requests` : 291 expirations, dont **220 à
/// 60 s** (76 %). Deux demandes de l'humain en font partie — `0f2c81ee89924`,
/// créée 27/08 18:31:19, expirée 18:32:20, soit **61 secondes**, pendant que
/// l'humain mettait huit heures à répondre. Et `bridget → humain` a expiré en
/// 120 s le 28/08 à 16:31 pendant qu'il vivait sa journée.
///
/// Une fenêtre de 60 s est calibrée pour un agent, dont le tour est borné par
/// une boucle. Un humain n'a pas de boucle : appliquée à lui, la garde
/// n'échantillonne pas un silence anormal, elle mesure le fait qu'il est
/// humain. Elle expire donc *systématiquement*, et un signal qui se déclenche
/// toujours n'est plus un signal.
const HUMAN_REPLY_TIMEOUT_SECS: u64 = 24 * 60 * 60;

/// Échéance effective d'une demande suivie.
///
/// `implique_humain` vaut vrai dès qu'un humain est à l'une des deux
/// extrémités. Le plancher ne **remplace** pas le délai demandé par
/// l'émetteur : il le relève. Un émetteur qui réclame plus de 24 h le garde.
///
/// La garde elle-même est préservée dans tous les cas — c'est un plancher, pas
/// une suppression. Une demande vers un agent mort continue d'expirer à sa
/// fenêtre d'origine, et une demande vers un humain finit par expirer aussi :
/// on ne remplace pas une fausse alerte par une file qui ne se vide jamais.
///
/// Complexité : O(1).
fn effective_reply_timeout(timeout_secs: u64, implique_humain: bool) -> u64 {
    if implique_humain {
        timeout_secs.max(HUMAN_REPLY_TIMEOUT_SECS)
    } else {
        timeout_secs
    }
}

/// Vrai si un tour `busy` a dépassé notify_timeout + grâce et doit redevenir
/// mandatable. Sans `busy_since`, on retombe sur `capacity_seen` (Register
/// historique / upgrade à chaud).
fn busy_turn_is_stale(presence: &Presence, ttl_secs: u64, now: Instant) -> bool {
    if presence.state != "busy" {
        return false;
    }
    let start = if presence.turn_attest {
        // Le délai n'expire pas un tour que le porteur réatteste encore.
        if presence
            .last_attested_at
            .is_none_or(|at| now.saturating_duration_since(at) <= Duration::from_secs(90))
        {
            return false;
        }
        presence.busy_period_start.unwrap_or(presence.capacity_seen)
    } else {
        presence.busy_since.unwrap_or(presence.capacity_seen)
    };
    let elapsed = now.saturating_duration_since(start);
    elapsed >= Duration::from_secs(ttl_secs)
}

#[derive(Clone)]
struct Presence {
    name: String,
    agent_type: String,
    host: String,
    /// Protocole d'agent réellement parlé.
    transport: String,
    /// Canal de connexion au daemon, absent s'il n'a pas été attesté.
    channel: Option<String>,
    /// Mode de présence attesté à l'enregistrement. Les présences historiques
    /// restent `None` : ne jamais le déduire du transport ou du type d'agent.
    mode: Option<PresenceMode>,
    /// Localisation tmux attestée au format `session:window.pane`.
    location: Option<String>,
    /// Journal append-only attesté par le pilote vivant. Il est réinitialisé
    /// à chaque Register : un mode de présence ne prouve jamais un journal.
    journal_available: bool,
    os: String,
    state: String,
    /// Instant où le tour `busy` a commencé (`TurnState true` / Register
    /// `turn_in_progress`). Absent hors busy. Sert à libérer un tour qui
    /// n'aboutit jamais : sans cela le service compagnon refuse tout mandat (connected|dnd).
    busy_since: Option<Instant>,
    // Le début stable ne remplace pas l'horloge legacy.
    busy_period_start: Option<Instant>,
    last_attested_at: Option<Instant>,
    turn_attest: bool,
    queue_fallback: bool,
    /// Dernière attestation de CAPACITÉ (register, tour, runtime…) — pas le
    /// heartbeat. C'est ce que `last_seen_secs` expose à who / bridget-idle.
    capacity_seen: Instant,
    /// Dernière attestation de LIEN (socket / heartbeat). Le retain s'appuie
    /// dessus pour ne pas jeter un long tour `busy` vivant.
    link_seen: Instant,
    reconnect_count: u32,
    /// Modèle courant, `None` tant qu'aucune observation n'a eu lieu.
    model: Option<String>,
    /// Niveau d'effort courant, `None` si jamais observé ou observé absent.
    effort: Option<String>,
    /// Limites fournisseur attestées, indexées par fenêtre. Une observation
    /// n'écrase que sa propre clé : `five_hour` ne touche pas `seven_day`.
    rate_limits: std::collections::BTreeMap<String, bridget_transport::protocol::RateLimitFact>,
    /// Modèle annoncé par le flux natif. Distinct du modèle épinglé : un flux
    /// muet laisse ce champ vide et n'invente aucun écart.
    served_model: Option<String>,
    /// Domaine dérivé annoncé à l'enregistrement, conservé pour pouvoir revenir
    /// dessus après une surcharge.
    derived_domain: Option<String>,
    /// Domaine effectif : la surcharge si elle existe, le domaine dérivé sinon.
    domain: Option<String>,
    /// Échéance jusqu'à laquelle l'agent refuse d'être dérangé.
    ///
    /// Une échéance plutôt qu'un booléen : l'expiration devient une simple
    /// comparaison à la lecture, sans tâche de fond pour balayer les statuts.
    dnd_until: Option<Instant>,
    /// Relevé local transmis par le wrapper juste après Register. Ce fait est
    /// affiché uniquement : il ne pilote aucune politique du daemon.
    disk_space: Option<DiskSpaceFact>,
}

impl Presence {
    /// Vrai tant que l'agent refuse d'être dérangé.
    fn is_dnd(&self) -> bool {
        self.dnd_until
            .map(|until| Instant::now() < until)
            .unwrap_or(false)
    }

    /// Minutes restantes de refus, arrondies au supérieur, minimum 1.
    fn dnd_minutes_left(&self) -> u64 {
        self.dnd_until
            .and_then(|until| until.checked_duration_since(Instant::now()))
            .map(|left| left.as_secs().div_ceil(60))
            .unwrap_or(0)
            .max(1)
    }

    /// Capacité d'exécution observée : met à jour les deux horloges.
    /// Aussi appelée à l'observation d'une *incapacité* (stopped /
    /// unreachable) : ce n'est pas une attestation de vie, c'est dater
    /// l'entrée pour le retain.
    fn touch_capacity(&mut self) {
        let now = Instant::now();
        self.capacity_seen = now;
        self.link_seen = now;
    }

    /// Un message émis atteste une capacité métier, mais pas forcément le
    /// socket principal : MCP et la CLI passent par une connexion auxiliaire.
    /// Ne jamais prolonger le retain du wrapper à partir de cette observation.
    fn touch_message_activity(&mut self) {
        self.capacity_seen = Instant::now();
    }

    /// Lien socket seul (heartbeat) : ne prouve aucune capacité.
    fn touch_link(&mut self) {
        self.link_seen = Instant::now();
    }
}

/// Configuration du daemon.
#[derive(Clone)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub circuit_breaker_window: u64,
    pub circuit_breaker_limit: usize,
    pub dedup_window: u64,
    pub quarantine_window: u64,
    pub retention_days: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let namespace = crate::environment::Namespace::from_environment()
            .expect("namespace Bridget invalide ; valider avant de construire la configuration");
        let cache_dir = namespace.root;
        DaemonConfig {
            socket_path: namespace.socket,
            db_path: cache_dir.join("bridget.db"),
            log_path: cache_dir.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        }
    }
}

impl DaemonConfig {
    /// Contenus canoniques dans le même namespace privé. Une configuration
    /// explicite de test dérive ses contenus de son propre magasin.
    pub fn canonical_artifact_root(&self) -> Result<PathBuf, String> {
        if let Some(raw) = std::env::var_os("BRIDGET_ARTIFACT_ROOT") {
            let path = PathBuf::from(raw);
            crate::environment::validate_private_directory_if_present(&path)?;
            let root = self.db_path.parent().ok_or("magasin sans racine")?;
            if !path.starts_with(root) {
                return Err("BRIDGET_ARTIFACT_ROOT doit rester dans la racine du magasin".into());
            }
            return Ok(path);
        }
        let root = self.db_path.parent().ok_or("magasin sans racine")?;
        Ok(root.join("artifacts"))
    }
}

fn desired_state_path(config: &DaemonConfig) -> PathBuf {
    crate::desired_state::path_for_daemon_db(&config.db_path)
}

/// État partagé du daemon.
#[cfg(test)]
struct FixtureRoot(std::path::PathBuf);

#[cfg(test)]
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct DaemonState {
    update_gate: Arc<crate::updates::Gate>,
    /// Quotas observés, volatils et bornés par type de fournisseur.
    provider_limits: crate::provider_limits::ProviderLimits,
    host_resources: crate::host_resources::HostResourceCollector,
    /// Identifiants métier retenus : filtre des rappels, source durable send_deliveries.
    held_ids: HashSet<String>,
    /// Suivis de consultation : aucune observation, mission ou donnée durable.
    human_thread_watches: HashMap<String, HumanThreadWatch>,
    lineage_watches: HashMap<String, native_lineage::Watch>,
    lineage_journals: HashMap<String, native_lineage::Journal>,
    /// Autorité éphémère : None interdit un fait contradictoire sur ce lien.
    t3_thread_bindings: HashMap<String, Option<String>>,
    native_permission_facts: HashMap<
        String,
        (
            bridget_transport::protocol::ProviderPermissions,
            Option<String>,
        ),
    >,
    native_permission_refusals: HashMap<String, String>,
    workspace_work: Arc<Mutex<native_delegation::WorkspaceWork>>,
    /// La projection projet historique ne conserve pas son origine.
    t3_project_connections: HashSet<String>,
    /// Fait de communication de la connexion primaire (None = ambigu).
    communication_projects: HashMap<
        String,
        (
            Option<bridget_transport::protocol::CommunicationProject>,
            Option<bridget_transport::protocol::CommunicationProject>,
            bool,
        ),
    >,
    observations: crate::observation::Observations,
    observation_sequences: HashMap<String, u64>,
    /// Session 102 : version d'alerte de fil acceptée par connexion principale,
    /// annoncée après l'enregistrement et effacée avec la connexion.
    thread_notice_versions: HashMap<String, u16>,
    /// Session 102 : instants des derniers départs d'alertes de fil (débit 5/s).
    thread_wake_departures: VecDeque<Instant>,
    /// Session 104 : permis de recherche/relecture (2 simultanés, sans file).
    ledger_read_permits: crate::ledger::search::ReadPermits,
    observation_tx: mpsc::SyncSender<ObservationJob>,
    observation_result_tx: mpsc::SyncSender<ObservationWriteResult>,
    observation_result_rx: mpsc::Receiver<ObservationWriteResult>,
    cli_wake_cursor: Option<(i64, String)>,
    observation_lost: Arc<std::sync::atomic::AtomicU64>,
    #[cfg(test)]
    fixture_root: Option<FixtureRoot>,
    /// Machine, base et identité **de ce daemon**, retenues une fois au
    /// démarrage.
    ///
    /// Le rapport dédié les atteste ensuite sur chaque connexion : un client
    /// fédéré ne peut pas les déduire, ni confondre deux redémarrages locaux.
    host: String,
    db_path: PathBuf,
    socket_path: PathBuf,
    /// Racine privée des contenus publiés par cette instance. Elle est
    /// distincte du cache Desktop et ne sort jamais dans une réponse MCP.
    artifact_root: PathBuf,
    /// Valeur éphémère créée une fois pour ce démarrage. Elle rend observable
    /// le remplacement d'un daemon même quand hôte, base et build sont égaux.
    instance_id: String,
    /// Époque monotone au sens d'une incarnation : tous les reçus créés par ce
    /// daemon portent la même valeur. Elle ne dépend donc jamais de l'instant
    /// d'admission et permet à l'entretien de ne reprendre qu'après restart.
    coordinator_epoch: i64,
    /// Préparations V2 en cours, indexées par propriétaire et clé. État volatile.
    routing_admission_claims: Arc<Mutex<HashSet<(String, String)>>>,
    router: Router,
    circuit_breaker: CircuitBreaker,
    execution_store: ExecutionStore,
    delegation_store: crate::delegation::DelegationStore,
    /// Permis HTTP 163 partagé par l'instance, conservé même si le demandeur
    /// atteint sa deadline pendant qu'un DNS/TLS distant reste bloqué.
    decision_service: crate::decision_service::DecisionServiceClient,
    active_waits: Arc<std::sync::atomic::AtomicUsize>,
    deduplicator: Deduplicator,
    envelope_guard: EnvelopeGuard,
    store: Store,
    idempotency: IdempotencyStore,
    fleet: Arc<FleetSupervisor>,
    registry: AgentRegistry,
    source_env: SourceEnvironment,
    recovering: bool,
    recovery_commands: HashSet<String>,
    managed_tx: Sender<ManagedSupervisorCommand>,
    marker_store: ManagedMarkerStore,
    managed_spawns: HashMap<String, ManagedSpawnRecord>,
    /// Demandeurs d'une nouvelle génération qui attendent un résultat de
    /// relance, distinct du résultat public d'un spawn neuf.
    relaunch_requesters: HashMap<String, Vec<String>>,
    /// Noms dont le décommissionnement a franchi les gardes mais attend
    /// encore l'arrêt supervisé. Cette réservation ferme la course avec une
    /// relance concurrente avant l'écriture de la tombstone.
    decommissioning_names: HashSet<String>,
    managed_by_instance: HashMap<String, String>,
    managed_terminal_instances: HashSet<String>,
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_hosts: HashMap<String, String>,
    conn_operating_systems: HashMap<String, String>,
    conn_instances: HashMap<String, String>,
    /// Connexions MCP filles : elles portent le principal canonique pour les
    /// gardes, sans devenir propriétaires de la présence du wrapper.
    auxiliary_connections: HashSet<String>,
    /// Secrets éphémères détenus par les seules connexions propriétaires.
    identity_credentials: HashMap<String, bridget_transport::protocol::IdentityCredential>,
    /// Les clients attach négocient ce rôle explicite ; l'absence d'entrée
    /// reste un wrapper pour préserver les agents 007 déjà connectés.
    connection_roles: HashMap<String, ConnectionRole>,
    remote_workers: HashMap<String, RemoteWorkerConnection>,
    remote_native_ready: HashSet<String>,
    /// Curseur de sondage uniquement ; aucune preuve d'arrêt ni permis durable.
    remote_start_reconcile_last: Option<(String, Instant)>,
    remote_start_reconciles: HashMap<String, (String, Receiver<()>)>,
    /// Rotation des réservations ; ne remplace aucune preuve de droits.
    dispatch_diagnostics: dispatch_diagnostics::Cache,
    remote_result_reservation_last: Option<(String, Instant)>,
    /// Un seul travail réseau en vol ; résultat consommé sans attendre dans tick.
    remote_result_reservation: Option<Receiver<Result<Option<crate::delegation::Task>, String>>>,
    worker_probe_connections: HashSet<String>,
    pending_worker_responses: HashMap<String, PendingWorkerResponse>,
    late_worker_prepares: late_worker_prepares::Cache,
    pending_workspace_repairs: HashMap<String, native_delegation::workspace_repair::Pending>,
    pending_origin_proofs: HashMap<String, origin_proof::Pending>,
    user_input_dispatch_after: Option<(i64, String)>,
    mobile_notifier: Option<crate::mobile::notifications::Notifier>,
    pending_candidate_probes: HashMap<String, PendingCandidateProbe>,
    pending_candidate_probes_v2: HashMap<String, PendingCandidateProbeV2>,
    terminal_sessions: HashSet<String>,
    /// Une négociation appartient à la connexion, tandis que le scope peut
    /// volontairement être partagé par plusieurs retries coopératifs.
    client_negotiations: HashMap<String, NegotiatedClient>,
    /// La capacité du guichet ne dépend jamais d'un nom déclaré : elle est
    /// attachée à cette négociation de service et disparaît avec la connexion.
    service_negotiations: HashMap<String, NegotiatedService>,
    /// UID SO_PEERCRED observé à l'acceptation. Il ne vient jamais de JSON.
    peer_uids: HashMap<String, u32>,
    /// Relevés 016 v2 en cours. Une entrée non fraîche reste volontairement
    /// muette jusqu'à une nouvelle souscription ayant atteint son snapshot.
    coordination_subscriptions: HashMap<String, CoordinationSubscription>,
    /// Souscriptions attach actives, distinctes de l'annuaire des équipiers.
    attach_subscriptions: HashMap<String, AttachSubscription>,
    attach_views: HashMap<String, Arc<AttachView>>,
    view_closed_tx: Sender<String>,
    view_closed_rx: Receiver<String>,
    pending_attach_sends: HashMap<String, PendingAttachSend>,
    pending_runtime_selections: HashMap<
        String,
        (
            String,
            std::sync::mpsc::Sender<bridget_transport::protocol::RuntimeSelectionOutcome>,
        ),
    >,
    pending_runtime_catalogues: HashMap<
        String,
        (
            String,
            std::sync::mpsc::Sender<bridget_transport::protocol::RuntimeCatalogueOutcome>,
        ),
    >,
    runtime_catalogue_connections: HashMap<String, (String, String)>,
    presences: HashMap<String, Presence>,
    conn_counter: u64,
    /// Messages --reply en attente de réponse : (msg_id, from, to, expire_at, target_conn)
    pending_replies: Vec<PendingReply>,
    /// Remises idempotentes à écrire juste après la réponse `Registered`.
    /// L'ordre rend le réenregistrement observable avant toute redélivrance.
    specialist_capture_active: HashSet<String>,
    pending_post_response_controls: HashMap<String, Vec<DeferredControl>>,
}

struct RemoteWorkerConnection {
    identity: bridget_transport::protocol::WorkerIdentity,
    conn_id: String,
    epoch: String,
    /// Négociées à l'ouverture seulement : reconnexion ou remplacement les
    /// efface avec l'ancienne époque, sans déduire une capacité du build.
    capabilities: Vec<bridget_transport::protocol::WorkerCapability>,
    catalogues: HashMap<String, native_provider_catalogue::CachedCatalogue>,
    catalogue_refresh_id: String,
    catalogue_refresh_at: Option<Instant>,
    last_contact: Instant,
    last_contact_at: i64,
    contact: bridget_transport::protocol::WorkerContactState,
}

struct PendingWorkerResponse {
    conn_id: String,
    epoch: String,
    worker: bridget_transport::protocol::WorkerIdentity,
    command: bridget_transport::protocol::WorkerCommandIdentity,
    reply: mpsc::SyncSender<bridget_transport::protocol::WorkerResponseOperation>,
}

/// Attente éphémère d'une sonde163. Elle ne contient ni Task, ni réservation,
/// ni autorité ; son unique rôle est de corréler une lecture worker à l'époque
/// négociée qui l'a autorisée.
struct PendingCandidateProbe {
    conn_id: String,
    epoch: String,
    worker: bridget_transport::protocol::WorkerIdentity,
    admission_id: String,
    policy_revision: u64,
    candidate_ids: std::collections::BTreeSet<String>,
    reply: mpsc::SyncSender<bridget_transport::protocol::CandidateProbeResult>,
}

/// Corrélation éphémère168 : même garde qu'une sonde163, avec la version
/// négociée et le relevé de ressources borné du worker.
struct PendingCandidateProbeV2 {
    conn_id: String,
    epoch: String,
    worker: bridget_transport::protocol::WorkerIdentity,
    admission_id: String,
    policy_revision: u64,
    candidate_ids: std::collections::BTreeSet<String>,
    reply: mpsc::SyncSender<bridget_transport::protocol::CandidateProbeV2Result>,
}

struct ManagedSpawnRecord {
    lease: SpawnLease,
    agent_type: String,
    requester_conns: Vec<String>,
    wrapper_conn: Option<String>,
    stop: Arc<ManagedStopControl>,
}

const HUMAN_WATCH_MAX: usize = 128;
const HUMAN_WATCH_QUEUE_MAX: usize = 16;

struct HumanThreadWatch {
    request: bridget_transport::protocol::HumanThreadWatchV1,
    project: Option<bridget_transport::protocol::CommunicationProject>,
    owner: String,
    agent: String,
    queue: HumanWatchQueue,
    notify: mpsc::SyncSender<()>,
}

struct HumanWatchQueue {
    generation: String,
    seq: u64,
    pending: VecDeque<bridget_transport::protocol::HumanThreadWatchEvent>,
    terminal: bool,
    #[cfg(test)]
    guard_checks: u64,
}

impl HumanWatchQueue {
    fn new() -> Self {
        use bridget_transport::protocol::{HUMAN_THREAD_WATCH_VERSION, HumanThreadWatchEvent as E};
        let generation = uuid::Uuid::new_v4().to_string();
        Self {
            pending: VecDeque::from([E::Ready {
                version: HUMAN_THREAD_WATCH_VERSION,
                generation: generation.clone(),
                seq: 0,
            }]),
            generation,
            seq: 0,
            terminal: false,
            #[cfg(test)]
            guard_checks: 0,
        }
    }

    fn terminate(&mut self, code: bridget_transport::protocol::HumanThreadViewError) {
        use bridget_transport::protocol::HumanThreadWatchEvent as E;
        if self.terminal {
            return;
        }
        // Une révocation prime sur un ready non encore émis. Le terminal
        // remplace les changements, puis ne peut plus être coalescé.
        self.pending.clear();
        self.pending.push_back(E::error(code));
        self.terminal = true;
    }

    fn changed(&mut self) {
        use bridget_transport::protocol::{
            HUMAN_THREAD_WATCH_MAX_SEQ, HUMAN_THREAD_WATCH_VERSION, HumanThreadViewError,
            HumanThreadWatchEvent as E,
        };
        if self.terminal {
            return;
        }
        if self.seq == HUMAN_THREAD_WATCH_MAX_SEQ {
            self.terminate(HumanThreadViewError::ResponseTooLarge);
            return;
        }
        self.seq += 1;
        let coalesced = self.pending.len() == HUMAN_WATCH_QUEUE_MAX
            || self
                .pending
                .iter()
                .any(|event| matches!(event, E::Resync { .. }));
        if coalesced {
            self.pending
                .retain(|event| matches!(event, E::Ready { .. }));
        }
        let event = if coalesced {
            E::Resync {
                version: HUMAN_THREAD_WATCH_VERSION,
                generation: self.generation.clone(),
                seq: self.seq,
            }
        } else {
            E::Changed {
                version: HUMAN_THREAD_WATCH_VERSION,
                generation: self.generation.clone(),
                seq: self.seq,
            }
        };
        self.pending.push_back(event);
    }
}

type ManagedRecovery = (PreparedSpawn, Arc<ManagedStopControl>);

pub(crate) struct ManagedStopControl {
    requested: AtomicBool,
    handshake_complete: AtomicBool,
    waiters: Mutex<Vec<Sender<StopOutcome>>>,
}

impl ManagedStopControl {
    fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
            handshake_complete: AtomicBool::new(false),
            waiters: Mutex::new(Vec::new()),
        }
    }

    fn request(&self) -> Receiver<StopOutcome> {
        let (sender, receiver) = mpsc::channel();
        self.waiters.lock().recover_poison().push(sender);
        self.requested.store(true, Ordering::SeqCst);
        receiver
    }

    fn is_requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }

    fn mark_handshake_complete(&self) {
        self.handshake_complete.store(true, Ordering::SeqCst);
    }

    fn is_actionable(&self) -> bool {
        self.is_requested() && self.handshake_complete.load(Ordering::SeqCst)
    }

    fn complete(&self, outcome: StopOutcome) {
        let waiters = std::mem::take(&mut *self.waiters.lock().recover_poison());
        for waiter in waiters {
            let _ = waiter.send(outcome.clone());
        }
    }
}

enum ManagedStopTarget {
    Supervised {
        receiver: Receiver<StopOutcome>,
        completion: Arc<ManagedStopControl>,
        wrapper: Option<Arc<Mutex<BufWriter<UnixStream>>>>,
    },
    Marker {
        store: ManagedMarkerStore,
        fallback: StopOutcome,
    },
    Immediate(StopOutcome),
}

// Snapshot sans demande d'arrêt : le lot entier est vérifié avant ses effets.
enum StopKind {
    Supervised {
        lease: SpawnLease,
        completion: Arc<ManagedStopControl>,
        wrapper: Option<Arc<Mutex<BufWriter<UnixStream>>>>,
    },
    Marker {
        store: ManagedMarkerStore,
        fallback: StopOutcome,
        stop_desired: bool,
    },
    Immediate(StopOutcome),
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum ManagedSupervisorCommand {
    /// Arrêt explicite du superviseur, indépendant des clones du canal encore vivants.
    Shutdown,
    Start {
        prepared: PreparedSpawn,
        stop: Arc<ManagedStopControl>,
    },
    Registered {
        instance_id: String,
        conn_id: String,
        name: String,
    },
}
pub(crate) enum ManagedSupervisorEvent {
    Connected {
        lease: SpawnLease,
        conn_id: String,
        definition: Box<bridget_transport::ResolvedAgentDefinition>,
    },
    Failed {
        lease: SpawnLease,
        kind: String,
        reason: String,
        /// Connexion wrapper déjà inscrite (Register) — à retirer de l'annuaire
        /// quand le spawn échoue après naissance du processus.
        conn_id: Option<String>,
    },
    Exited {
        lease: SpawnLease,
        conn_id: Option<String>,
        reason: String,
    },
    Stopped {
        lease: SpawnLease,
        conn_id: Option<String>,
        outcome: StopOutcome,
        completion: Arc<ManagedStopControl>,
    },
    StopTimedOut {
        lease: SpawnLease,
        outcome: StopOutcome,
        completion: Arc<ManagedStopControl>,
    },
}

#[derive(Clone)]
struct AttachSubscription {
    agent: String,
    attach_conn: String,
    wrapper_conn: String,
    caught_up: bool,
}

struct PendingAttachSend {
    conn_id: String,
    expires_at: Instant,
}

#[derive(Clone)]
struct NegotiatedClient {
    version: u16,
    issuer_scope: String,
    capabilities: Vec<ClientCapability>,
}

#[derive(Clone)]
struct NegotiatedService {
    version: u16,
    capabilities: Vec<ServiceCapability>,
}

#[derive(Clone)]
struct CoordinationSubscription {
    fresh: bool,
    next_cursor: u64,
}

struct QueuedAttachMessage {
    encoded: String,
    seq: Option<u64>,
    terminal: bool,
}

struct AttachViewBuffer {
    messages: VecDeque<QueuedAttachMessage>,
    bytes: usize,
    gap: Option<(u64, u64, Option<String>)>,
    /// Séquence évincée dont les fragments suivants doivent encore être
    /// ignorés jusqu'à sa frontière finale.
    dropping_seq: Option<u64>,
    terminal_enqueued: bool,
    closed: bool,
    close_notified: bool,
}

struct AttachView {
    queue: Arc<(Mutex<AttachViewBuffer>, Condvar)>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
    subscription_id: String,
    closed_tx: Sender<String>,
}

impl AttachView {
    #[cfg(test)]
    fn suspended(_subscription_id: impl Into<String>) -> Arc<Self> {
        Self::suspended_with_notifications(_subscription_id).0
    }

    #[cfg(test)]
    fn suspended_with_notifications(
        subscription_id: impl Into<String>,
    ) -> (Arc<Self>, Receiver<String>) {
        let (closed_tx, closed_rx) = mpsc::channel();
        (
            Self::suspended_with_sender(subscription_id, closed_tx),
            closed_rx,
        )
    }

    #[cfg(test)]
    fn suspended_with_sender(
        subscription_id: impl Into<String>,
        closed_tx: Sender<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            queue: Arc::new((
                Mutex::new(AttachViewBuffer {
                    messages: VecDeque::new(),
                    bytes: 0,
                    gap: None,
                    dropping_seq: None,
                    terminal_enqueued: false,
                    closed: false,
                    close_notified: false,
                }),
                Condvar::new(),
            )),
            worker: Mutex::new(None),
            subscription_id: subscription_id.into(),
            closed_tx,
        })
    }

    fn start(
        subscription_id: String,
        writer: &Arc<Mutex<BufWriter<UnixStream>>>,
        closed_tx: Sender<String>,
    ) -> Option<Arc<Self>> {
        // Tous les producteurs d'une même connexion (worker de vue et messages
        // de contrôle) partagent ce writer : le mutex sérialise les frames JSONL.
        let _ = writer
            .lock()
            .ok()?
            .get_ref()
            .set_write_timeout(Some(ATTACH_VIEW_WRITE_TIMEOUT));
        let queue = Arc::new((
            Mutex::new(AttachViewBuffer {
                messages: VecDeque::new(),
                bytes: 0,
                gap: None,
                dropping_seq: None,
                terminal_enqueued: false,
                closed: false,
                close_notified: false,
            }),
            Condvar::new(),
        ));
        let view = Arc::new(Self {
            queue: queue.clone(),
            worker: Mutex::new(None),
            subscription_id: subscription_id.clone(),
            closed_tx: closed_tx.clone(),
        });
        let worker_queue = queue.clone();
        let worker_subscription = subscription_id;
        let worker_writer = writer.clone();
        let handle = thread::spawn(move || {
            loop {
                let next = {
                    let (lock, wake) = &*worker_queue;
                    let mut buffer = lock.lock().recover_poison();
                    while buffer.messages.is_empty() && buffer.gap.is_none() && !buffer.closed {
                        buffer = wake.wait(buffer).recover_poison();
                    }
                    if let Some((from_seq, to_seq, reason)) = buffer.gap.take() {
                        encode(&DaemonToWrapper::Gap {
                            subscription_id: worker_subscription.clone(),
                            from_seq,
                            to_seq,
                            reason,
                        })
                        .ok()
                        .map(|encoded| (encoded, false))
                    } else if let Some(message) = buffer.messages.pop_front() {
                        buffer.bytes = buffer.bytes.saturating_sub(message.encoded.len());
                        Some((message.encoded, message.terminal))
                    } else {
                        None
                    }
                };
                let Some((next, terminal)) = next else {
                    if worker_queue.0.lock().recover_poison().closed {
                        break;
                    }
                    continue;
                };
                let write_failed = worker_writer
                    .lock()
                    .map(|mut writer| {
                        writeln!(writer, "{next}")
                            .and_then(|_| writer.flush())
                            .is_err()
                    })
                    .unwrap_or(true);
                if write_failed {
                    let (lock, wake) = &*worker_queue;
                    let mut buffer = lock.lock().recover_poison();
                    buffer.closed = true;
                    let notify = !buffer.close_notified;
                    buffer.close_notified = true;
                    drop(buffer);
                    wake.notify_all();
                    if notify {
                        let _ = closed_tx.send(worker_subscription.clone());
                    }
                    break;
                }
                if terminal {
                    let (lock, wake) = &*worker_queue;
                    lock.lock().recover_poison().closed = true;
                    wake.notify_all();
                    break;
                }
            }
        });
        *view.worker.lock().recover_poison() = Some(handle);
        Some(view)
    }

    fn enqueue(&self, message: DaemonToWrapper) -> bool {
        let (seq, final_fragment, terminal) = match &message {
            DaemonToWrapper::JournalFragment {
                seq,
                final_fragment,
                ..
            } => (Some(*seq), *final_fragment, false),
            DaemonToWrapper::End { .. } => (None, false, true),
            _ => (None, false, false),
        };
        let Ok(encoded) = encode(&message) else {
            self.close_and_notify();
            return false;
        };
        let (lock, wake) = &*self.queue;
        let mut buffer = lock.lock().recover_poison();
        if buffer.closed || buffer.terminal_enqueued {
            return false;
        }
        if terminal {
            // Le terminal clôt la file : les données restées en attente sont
            // explicitement abandonnées avant End, jamais écrites après lui.
            buffer.messages.clear();
            buffer.bytes = 0;
            buffer.gap = None;
            buffer.dropping_seq = None;
            buffer.terminal_enqueued = true;
            buffer.messages.push_back(QueuedAttachMessage {
                encoded,
                seq: None,
                terminal: true,
            });
            wake.notify_one();
            return true;
        }
        if let Some(seq) = seq {
            if buffer.dropping_seq == Some(seq) {
                if final_fragment {
                    buffer.dropping_seq = None;
                }
                return false;
            }
            while buffer.bytes.saturating_add(encoded.len()) > ATTACH_VIEW_BUFFER_BYTES {
                let Some(dropped_seq) = buffer.messages.iter().find_map(|item| item.seq) else {
                    buffer.dropping_seq = Some(seq);
                    buffer.gap = Some((seq, seq, Some("vue trop lente".to_string())));
                    wake.notify_one();
                    return false;
                };
                let mut retained = VecDeque::new();
                while let Some(item) = buffer.messages.pop_front() {
                    if item.seq == Some(dropped_seq) {
                        buffer.bytes = buffer.bytes.saturating_sub(item.encoded.len());
                    } else {
                        retained.push_back(item);
                    }
                }
                buffer.messages = retained;
                buffer.dropping_seq = Some(dropped_seq);
                buffer.gap = Some(match buffer.gap.take() {
                    Some((from, to, reason)) => {
                        (from.min(dropped_seq), to.max(dropped_seq), reason)
                    }
                    None => (dropped_seq, dropped_seq, Some("vue trop lente".to_string())),
                });
            }
            if buffer.dropping_seq == Some(seq) {
                if final_fragment {
                    buffer.dropping_seq = None;
                }
                return false;
            }
        } else if buffer.bytes.saturating_add(encoded.len()) > ATTACH_VIEW_BUFFER_BYTES {
            buffer.closed = true;
            let notify = !buffer.close_notified;
            buffer.close_notified = true;
            drop(buffer);
            wake.notify_all();
            if notify {
                let _ = self.closed_tx.send(self.subscription_id.clone());
            }
            return false;
        }
        buffer.bytes += encoded.len();
        buffer.messages.push_back(QueuedAttachMessage {
            encoded,
            seq,
            terminal: false,
        });
        wake.notify_one();
        true
    }

    fn close(&self) {
        let (lock, wake) = &*self.queue;
        lock.lock().recover_poison().closed = true;
        wake.notify_all();
    }

    fn close_and_notify(&self) {
        let (lock, wake) = &*self.queue;
        let mut buffer = lock.lock().recover_poison();
        if buffer.closed {
            return;
        }
        buffer.closed = true;
        let notify = !buffer.close_notified;
        buffer.close_notified = true;
        drop(buffer);
        wake.notify_all();
        if notify {
            let _ = self.closed_tx.send(self.subscription_id.clone());
        }
    }

    fn close_and_join(&self) {
        self.close();
        if let Some(worker) = self.worker.lock().recover_poison().take() {
            let _ = worker.join();
        }
    }
}

struct PendingReply {
    msg_id: String,
    from: String,
    from_conn: String,
    to: String,
    target_conn: String,
    /// Timeout total en secondes (configurable par l'émetteur)
    timeout_secs: u64,
    created_at: std::time::Instant,
    frozen_secs: u64,
    /// Palier d'escalade atteint : 0 = rien, 1 = rappel discret,
    /// 2 = rappel ferme, 3 = notification échec à l'émetteur
    escalation_level: u8,
    deferred_level: Option<u8>,
}

enum ReminderAction {
    PushDeferred {
        target_conn: String,
    },
    Gentle {
        to: String,
        from: String,
        msg_id: String,
        target_conn: String,
    },
    Firm {
        to: String,
        from: String,
        msg_id: String,
        target_conn: String,
    },
    Timeout {
        to: String,
        from: String,
        msg_id: String,
        from_conn: String,
        timeout_secs: u64,
        /// t3code reçoit un contrôle, jamais un nouveau tour textuel.
        cancel_target: Option<String>,
    },
    Deferred {
        to: String,
        msg_id: String,
        /// `tour en cours` ou `pause` (SPEC-087) : le motif est consigné.
        reason: &'static str,
        level: u8,
    },
}

// Orchestration158B : aucune transaction SQL n'est gardée pendant une E/S.
// La preuve native est capturée à l'admission puis revalidée au gel/départ.
fn wake_native_origin(
    st: &DaemonState,
    owner: &str,
    frozen: Option<&crate::wake_schedules::NativeWakeOrigin>,
) -> Result<Option<crate::wake_schedules::NativeWakeOrigin>, &'static str> {
    let task = st
        .delegation_store
        .for_child(owner)
        .map_err(|_| "storage_unavailable")?;
    let Some(task) = task else {
        return if frozen.is_some() {
            Err("owner_unavailable")
        } else {
            Ok(None)
        };
    };
    let instance = task.child_instance.as_deref().ok_or("owner_unavailable")?;
    if frozen.is_some_and(|old| {
        old.task_id != task.task_id || old.instance_id != instance || old.mission != task.mission
    }) || task.cleanup_done
        || !matches!(
            task.state.as_str(),
            "starting" | "mission_pending" | "working" | "waiting_for_children"
        )
    {
        return Err("owner_unavailable");
    }
    let route = st.router.get_agent(owner).ok_or("owner_unavailable")?;
    if live_connection_identity(st, &route.connection_id)
        != Some((owner.to_owned(), instance.to_owned()))
    {
        return Err("owner_unavailable");
    }
    let link = st
        .fleet
        .agent_link_for_child(instance)
        .map_err(|_| "storage_unavailable")?
        .ok_or("owner_unavailable")?;
    if link.role != "native_delegate"
        || !link.state.owns_child()
        || link.delegation_id.as_deref() != Some(task.task_id.as_str())
        || st.managed_by_instance.get(instance) != Some(&task.task_id)
    {
        return Err("owner_unavailable");
    }
    let mut current = task.clone();
    let mut seen = HashSet::from([task.task_id.clone()]);
    while let Some(id) = current.parent_task_id.as_deref() {
        if seen.len() >= 8 || !seen.insert(id.to_owned()) {
            return Err("owner_unavailable");
        }
        current = st
            .delegation_store
            .get(id)
            .map_err(|_| "storage_unavailable")?
            .ok_or("owner_unavailable")?;
        if current.root_owner() != task.root_owner()
            || current.cleanup_done
            || !matches!(
                current.state.as_str(),
                "starting" | "mission_pending" | "working" | "waiting_for_children"
            )
        {
            return Err("owner_unavailable");
        }
    }
    if let Some(id) = &task.parent_execution_id {
        let execution = st
            .execution_store
            .execution_snapshot(id)
            .map_err(|_| "storage_unavailable")?
            .ok_or("owner_unavailable")?;
        if matches!(
            execution.state.as_str(),
            "interrupted" | "completed" | "failed" | "unreachable"
        ) {
            return Err("owner_unavailable");
        }
    }
    Ok(Some(crate::wake_schedules::NativeWakeOrigin {
        task_id: task.task_id,
        instance_id: instance.to_owned(),
        mission: task.mission,
    }))
}

// Trois puits réels : admission, gel et dernière vérification avant émission.
fn wake_target_guard(
    st: &DaemonState,
    owner: &str,
    target: &bridget_transport::protocol::WakeTarget,
) -> Result<(), &'static str> {
    use bridget_transport::protocol::{ThreadAction, ThreadRequest, WakeTarget};
    match target {
        WakeTarget::SelfTarget {} => {
            if crate::agent_tracking::dormant(st.store.connection(), owner)
                .map_err(|_| "tracking_storage_unavailable")?
            {
                return Err("agent_dormant");
            }
            Ok(())
        }
        WakeTarget::Agent { id } => {
            if crate::threads::canonical_uuid(id).as_deref() != Some(id.as_str()) {
                return Err("invalid_request");
            }
            if crate::agent_tracking::dormant(st.store.connection(), id)
                .map_err(|_| "tracking_storage_unavailable")?
            {
                return Err("agent_dormant");
            }
            if !st.presences.values().any(|p| p.name == *id) && st.router.get_agent(id).is_none() {
                return Err("target_unknown");
            }
            if owner != id {
                crate::communication::project_scope(
                    agent_communication_project(st, owner).as_ref(),
                    agent_communication_project(st, id).as_ref(),
                    id,
                    None,
                )
                .map_err(|_| "project_scope_unavailable")?;
            }
            Ok(())
        }
        WakeTarget::Thread { id } => {
            let (result, _) = crate::threads::handle_with_change(
                &st.store,
                &DaemonDirectory { state: st },
                owner,
                ThreadRequest {
                    version: 1,
                    cross_project_reason: None,
                    request: ThreadAction::Show {
                        thread_id: id.clone(),
                    },
                },
                unix_now_secs(),
            );
            if result.result["status"] != "shown" || result.result["state"] != "open" {
                return Err("thread_unavailable");
            }
            let members: Vec<String> = result.result["members"]
                .as_array()
                .ok_or("storage_unavailable")?
                .iter()
                .map(|m| {
                    m["agent_id"]
                        .as_str()
                        .map(str::to_owned)
                        .ok_or("storage_unavailable")
                })
                .collect::<Result<_, _>>()?;
            crate::threads::Directory::communication_scope(
                &DaemonDirectory { state: st },
                owner,
                &members,
                None,
            )
            .map_err(|_| "project_scope_unavailable")?;
            Ok(())
        }
    }
}

/// Calcul civil hors mutex ; la décision est revalidée avant sa seule écriture.
fn handle_wake_request(
    conn_id: &str,
    request: bridget_transport::protocol::WakeRequest,
    state: &Arc<Mutex<DaemonState>>,
    now: i64,
) -> DaemonToWrapper {
    use crate::wake_schedules::{WakeSchedule, calculate_cadence, timezone_fingerprint};
    use bridget_transport::protocol::{WakeAction, WakeResult, WakeTarget};
    let result = (|| -> Result<serde_json::Value, &'static str> {
        if request.version != 1 {
            return Err("invalid_request");
        }
        let (owner, native, admin) = {
            let st = state.lock().recover_poison();
            let admin = matches!(
                request.request,
                WakeAction::AdminList {} | WakeAction::AdminCancel { .. }
            );
            if admin {
                if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Client) {
                    return Err("identity_unavailable");
                }
                let negotiated = st
                    .client_negotiations
                    .get(conn_id)
                    .ok_or("negotiation_required")?;
                if !negotiated
                    .capabilities
                    .contains(&ClientCapability::ControlStateV1)
                {
                    return Err("capability_not_negotiated");
                }
                control_client_actor(&st, conn_id).ok_or("human_actor_required")?;
                (String::new(), None, true)
            } else if let Some((owner, _)) = live_connection_identity(&st, conn_id) {
                let native = wake_native_origin(&st, &owner, None)?;
                (owner, native, false)
            } else if matches!(request.request, WakeAction::CliOnce { .. })
                && st.connection_roles.get(conn_id) == Some(&ConnectionRole::Client)
                && st
                    .client_negotiations
                    .get(conn_id)
                    .is_some_and(|n| n.version == CLIENT_CONTRACT_VERSION)
                && st.peer_uids.get(conn_id).copied()
                    == Some(bridget_transport::fsutil::effective_uid())
            {
                ("bridget-local-cli".to_owned(), None, false)
            } else {
                return Err("identity_unavailable");
            }
        };
        if matches!(request.request, WakeAction::CliOnce { .. }) && native.is_some() {
            return Err("identity_unavailable");
        }
        match request.request {
            WakeAction::Schedule {
                target,
                cadence,
                label,
            } => {
                let label = label.unwrap_or_default();
                if label.len() > 2048 {
                    return Err("invalid_request");
                }
                let mut schedule = WakeSchedule {
                    schedule_id: uuid::Uuid::new_v4().to_string(),
                    owner: owner.clone(),
                    owner_native: native,
                    owner_project: None,
                    target,
                    cadence,
                    label,
                    anchor_at: now,
                    next_at: now.saturating_add(1),
                    timezone: String::new(),
                    next_calendar_date: None,
                    previous_calendar_date: None,
                    occurrence: None,
                    cancelled_at: None,
                };
                // La première échéance d'un intervalle est now+seconds, jamais now.
                if let bridget_transport::protocol::WakeCadence::Interval { seconds } =
                    schedule.cadence
                {
                    schedule.next_at = now
                        .checked_add(seconds.try_into().map_err(|_| "invalid_request")?)
                        .ok_or("invalid_request")?;
                }
                schedule.owner_project = {
                    let st = state.lock().recover_poison();
                    agent_communication_project(&st, &owner)
                };
                let calculated = calculate_cadence(&schedule, now)?;
                schedule.next_at = calculated.next_at;
                schedule.timezone = calculated.fingerprint.clone();
                schedule.next_calendar_date = calculated.next_calendar_date.clone();
                let current_timezone = timezone_fingerprint(now)?;
                let st = state.lock().recover_poison();
                if live_connection_identity(&st, conn_id)
                    .is_none_or(|(current, _)| current != owner)
                {
                    return Err("identity_unavailable");
                }
                wake_native_origin(&st, &owner, schedule.owner_native.as_ref())?;
                wake_target_guard(&st, &owner, &schedule.target)?;
                if current_timezone != calculated.fingerprint {
                    return Err("calendar_unavailable");
                }
                st.store
                    .create_wake_schedule(&schedule)
                    .map_err(|e| match e {
                        crate::store::StoreError::Invariant("schedule_limit") => "schedule_limit",
                        _ => "storage_unavailable",
                    })?;
                Ok(serde_json::json!({"status":"scheduled","schedule":schedule}))
            }
            WakeAction::List {} | WakeAction::AdminList {} => {
                let st = state.lock().recover_poison();
                let schedules = st
                    .store
                    .list_wake_schedules((!admin).then_some(owner.as_str()))
                    .map_err(|_| "storage_unavailable")?;
                Ok(
                    serde_json::json!({"status":"listed","schedules":schedules,"calendar":"daemon_local","nonexistent_times":"skipped"}),
                )
            }
            WakeAction::Cancel { schedule_id } | WakeAction::AdminCancel { schedule_id } => {
                if crate::threads::canonical_uuid(&schedule_id).as_deref()
                    != Some(schedule_id.as_str())
                {
                    return Err("invalid_request");
                }
                let st = state.lock().recover_poison();
                let changed = st
                    .store
                    .cancel_wake_schedule((!admin).then_some(owner.as_str()), &schedule_id, now)
                    .map_err(|e| match e {
                        crate::store::StoreError::Invariant("target_unknown") => "target_unknown",
                        crate::store::StoreError::Invariant("not_owner") => "not_owner",
                        _ => "storage_unavailable",
                    })?;
                Ok(
                    serde_json::json!({"status":if changed{"cancelled"}else{"already_cancelled"},"schedule_id":schedule_id}),
                )
            }
            WakeAction::CliOnce { target, label, id } => {
                let label = label.unwrap_or_default();
                if label.len() > 2048 || id.is_empty() || id.len() > 256 {
                    return Err("invalid_request");
                }
                let mut st = state.lock().recover_poison();
                let directory = st.agent_infos();
                let agent = if crate::threads::canonical_uuid(&target).is_some() {
                    directory
                        .iter()
                        .find(|p| p.agent_id == target)
                        .map(|p| p.agent_id.clone())
                } else {
                    match crate::cli::resolve_name_in_directory(
                        &target,
                        directory
                            .iter()
                            .map(|p| (p.display_name.as_str(), p.agent_id.as_str())),
                    ) {
                        Ok(id) => Some(id),
                        Err(error) if error.contains("ambigu") => return Err("target_ambiguous"),
                        Err(_) => None,
                    }
                };
                let thread = if crate::threads::canonical_uuid(&target).is_some() {
                    st.store
                        .connection()
                        .query_row(
                            "SELECT EXISTS(SELECT 1 FROM discussion_threads WHERE thread_id=?1)",
                            [&target],
                            |r| r.get::<_, bool>(0),
                        )
                        .map_err(|_| "storage_unavailable")?
                } else {
                    false
                };
                if agent.is_some() && thread {
                    return Err("target_ambiguous");
                }
                let target = if thread {
                    if live_connection_identity(&st, conn_id).is_none() {
                        return Err("identity_unavailable");
                    }
                    WakeTarget::Thread { id: target }
                } else {
                    WakeTarget::Agent {
                        id: agent.ok_or("target_unknown")?,
                    }
                };
                if owner != "bridget-local-cli" {
                    wake_target_guard(&st, &owner, &target)?;
                }
                let project = agent_communication_project(&st, &owner);
                let (reservation, frozen) = match st.idempotency.reserve_cli_wake(
                    &id,
                    &owner,
                    &target,
                    &label,
                    project.as_ref(),
                    now,
                ) {
                    Ok(value) => value,
                    Err(crate::idempotency::IdempotencyError::CliWakeRateLimited {
                        retry_after_secs,
                    }) => {
                        return Ok(
                            serde_json::json!({"status":"refused","code":"rate_limited","retry_after_secs":retry_after_secs}),
                        );
                    }
                    Err(_) => return Err("storage_unavailable"),
                };
                match reservation {
                    Reservation::EnvelopeMismatch => Err("envelope_mismatch"),
                    Reservation::IdempotencyExpired => Err("request_expired"),
                    Reservation::Prepared { .. } => Ok(
                        serde_json::json!({"status":"prepared","id":id,"issued_at":frozen.issued_at,"target":frozen.target}),
                    ),
                    Reservation::Replayed(result) => Ok(
                        serde_json::json!({"status":"replayed","id":id,"issued_at":frozen.issued_at,"result":format!("{result:?}")}),
                    ),
                }
            }
        }
    })();
    DaemonToWrapper::WakeResult { result: WakeResult {version:1,result:result.unwrap_or_else(|code|serde_json::json!({"status":if matches!(code,"storage_unavailable"|"calendar_unavailable"){"error"}else{"refused"},"code":code}))} }
}

// Une seule frontière de départ pour occurrences, CLI et résumés ; guard au write.
fn deliver_frozen_wake(
    state: &Arc<Mutex<DaemonState>>,
    wake: &crate::wake_schedules::FrozenWake,
    scheduled_id: Option<&str>,
    now: i64,
    clock: Instant,
) -> Result<crate::wake_schedules::WakePhase, &'static str> {
    use crate::wake_schedules::{
        CLI_WAKE_SCOPE, MISSED_SUMMARY_SCOPE, SCHEDULED_WAKE_SCOPE, WakePhase, WakeSource,
    };
    use bridget_transport::protocol::WakeTarget;
    // Wall avant Mono fournit une borne conservatrice de la fraction du tick.
    // Le résultat102 déjà acquis reste prioritaire même si cette horloge échoue.
    let wall_before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok();
    let mono_anchor = Instant::now();
    let age_before = mono_anchor.saturating_duration_since(clock);
    let tick_wall_lower = wall_before.and_then(|wall| wall.checked_sub(age_before));
    let departure_deadline =
        (clock + Duration::from_secs(1)).min(mono_anchor + Duration::from_secs(1));
    let scope = match wake.source {
        WakeSource::Scheduled => SCHEDULED_WAKE_SCOPE,
        WakeSource::CliOnce => CLI_WAKE_SCOPE,
        WakeSource::MissedSummary => MISSED_SUMMARY_SCOPE,
    };
    let key =
        IdempotencyKey::new(scope, OperationKind::Send, &wake.id).map_err(|_| "invalid_request")?;
    let route = {
        let mut st = state.lock().recover_poison();
        // Un dépôt102 déjà committé n'est pas une nouvelle émission : retrouver
        // sa preuve avant les gardes mutables et sans republier ses intentions.
        if let WakeTarget::Thread { id } = &wake.target {
            let operation_id = wake
                .thread_operation_id
                .as_deref()
                .ok_or("invalid_request")?;
            let canonical_hash = crate::threads::digest(&[
                b"post",
                b"v1",
                id.as_bytes(),
                wake.body.as_bytes(),
                b"all",
                b"reply_to",
                b"ack",
            ]);
            match st
                .store
                .thread_operation_replay(&wake.owner, operation_id, &canonical_hash)
                .map_err(|_| "storage_unavailable")?
            {
                Some(crate::store::threads::ThreadTxOutcome::Replayed(result))
                    if result["status"] == "posted"
                        && result["thread_id"].as_str() == Some(id.as_str()) =>
                {
                    if matches!(wake.source, WakeSource::CliOnce) {
                        let (_, phase, expires_at) = st
                            .idempotency
                            .cli_wake(&wake.id)
                            .map_err(|_| "storage_unavailable")?
                            .ok_or("storage_unavailable")?;
                        if phase == crate::idempotency::RecordState::Prepared {
                            st.idempotency
                                .transition(
                                    &key,
                                    crate::idempotency::RecordState::Prepared,
                                    crate::idempotency::RecordState::Dispatching,
                                )
                                .map_err(|_| "storage_unavailable")?;
                        }
                        if phase != crate::idempotency::RecordState::Terminal {
                            st.idempotency
                                .finalize(
                                    &key,
                                    crate::idempotency::PublicResult::Accepted { expires_at },
                                )
                                .map_err(|_| "storage_unavailable")?;
                        }
                    }
                    return Ok(WakePhase::CommittedThread);
                }
                Some(crate::store::threads::ThreadTxOutcome::Refused(
                    crate::store::threads::ThreadRefusal::EnvelopeMismatch,
                )) => return Err("envelope_mismatch"),
                Some(_) => return Err("thread_unavailable"),
                None => {}
            }
        }
        match st
            .idempotency
            .send_delivery_phase(&key)
            .map_err(|_| "storage_unavailable")?
            .as_deref()
        {
            Some("acked") => return Ok(WakePhase::Confirmed),
            Some("dispatching") if now < wake.grace_until => {
                return Ok(WakePhase::DispatchingAgent);
            }
            Some(_) => return Ok(WakePhase::OutcomeUnknown),
            None => {}
        }
        if now >= wake.grace_until {
            return Ok(WakePhase::MissedBeforeDeparture);
        }
        if let Some(id) = scheduled_id {
            let current = st
                .store
                .wake_schedule(id)
                .map_err(|_| "storage_unavailable")?
                .ok_or("owner_unavailable")?;
            if current.cancelled_at.is_some()
                || current.occurrence.as_ref().is_none_or(|w| w.id != wake.id)
            {
                return Ok(WakePhase::OwnerUnavailable);
            }
        }
        if agent_communication_project(&st, &wake.owner) != wake.owner_project {
            return Err("project_scope_unavailable");
        }
        wake_native_origin(&st, &wake.owner, wake.owner_native.as_ref())?;
        if !(matches!(wake.source, WakeSource::CliOnce) && wake.owner == "bridget-local-cli") {
            wake_target_guard(&st, &wake.owner, &wake.target)?;
        }
        let control = crate::referent_control::read(st.store.connection())
            .map_err(|_| "storage_unavailable")?;
        if !matches!(
            crate::referent_control::admit_autonomous_effect(
                crate::referent_control::AutonomousEffect::ScheduledWake,
                &control
            ),
            crate::referent_control::Admission::Admitted
        ) {
            return Ok(WakePhase::Deferred);
        }
        if let WakeTarget::Thread { id } = &wake.target {
            let current_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|wall| {
                    tick_wall_lower.and_then(|tick| wall.as_secs().checked_sub(tick.as_secs()))
                })
                .and_then(|delta| i64::try_from(delta).ok())
                .and_then(|delta| now.checked_add(delta))
                .ok_or("calendar_unavailable")?;
            if current_now >= wake.grace_until {
                return Ok(WakePhase::MissedBeforeDeparture);
            }
            if matches!(wake.source, WakeSource::CliOnce)
                && st
                    .idempotency
                    .cli_wake(&wake.id)
                    .map_err(|_| "storage_unavailable")?
                    .is_some_and(|(_, phase, _)| phase == crate::idempotency::RecordState::Prepared)
            {
                st.idempotency
                    .transition(
                        &key,
                        crate::idempotency::RecordState::Prepared,
                        crate::idempotency::RecordState::Dispatching,
                    )
                    .map_err(|_| "storage_unavailable")?;
            }
            let current_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|wall| {
                    tick_wall_lower.and_then(|tick| wall.as_secs().checked_sub(tick.as_secs()))
                })
                .and_then(|delta| i64::try_from(delta).ok())
                .and_then(|delta| now.checked_add(delta))
                .ok_or("calendar_unavailable")?;
            if current_now >= wake.grace_until {
                return Ok(WakePhase::MissedBeforeDeparture);
            }
            let (posted, changed) = crate::threads::handle_with_change(
                &st.store,
                &DaemonDirectory { state: &st },
                &wake.owner,
                bridget_transport::protocol::ThreadRequest {
                    version: 1,
                    cross_project_reason: None,
                    request: bridget_transport::protocol::ThreadAction::Post {
                        thread_id: id.clone(),
                        body: wake.body.clone(),
                        notify: bridget_transport::protocol::ThreadNotify::All(
                            bridget_transport::protocol::ThreadNotifyAll::All,
                        ),
                        operation_id: wake.thread_operation_id.clone().ok_or("invalid_request")?,
                        reply_to_seq: None,
                        ack_receipt: None,
                        kind: None,
                        supersedes_seq: None,
                    },
                },
                current_now,
            );
            if posted.result["status"] != "posted" {
                return Err("thread_unavailable");
            }
            if let Some(thread) = changed {
                publish_human_thread_change(&mut st, &thread);
            }
            if matches!(wake.source, WakeSource::CliOnce) {
                st.idempotency
                    .finalize(
                        &key,
                        crate::idempotency::PublicResult::Accepted {
                            expires_at: wake.issued_at.saturating_add(7 * 86400),
                        },
                    )
                    .map_err(|_| "storage_unavailable")?;
            }
            return Ok(WakePhase::CommittedThread);
        }
        let target = match &wake.target {
            WakeTarget::SelfTarget {} => wake.owner.as_str(),
            WakeTarget::Agent { id } => id.as_str(),
            _ => unreachable!(),
        };
        if crate::agent_tracking::dormant(st.store.connection(), target)
            .map_err(|_| "tracking_storage_unavailable")?
        {
            return Ok(WakePhase::MissedBeforeDeparture);
        }
        let route = st.router.get_agent(target).ok_or("target_unavailable")?;
        let conn = route.connection_id.clone();
        let instance = st
            .conn_instances
            .get(&conn)
            .cloned()
            .ok_or("target_unavailable")?;
        let presence = st.presences.get(&instance).ok_or("target_unavailable")?;
        if presence.dnd_until.is_some_and(|until| clock < until)
            || !matches!(presence.state.as_str(), "connected" | "busy")
            || live_connection_identity(&st, &conn) != Some((target.to_owned(), instance.clone()))
        {
            return Ok(WakePhase::Deferred);
        }
        let definition = st
            .registry
            .get(&presence.agent_type)
            .map_err(|_| "target_unavailable")?;
        if !definition
            .capabilities
            .execution_paths
            .iter()
            .any(|path| path == &definition.protocol)
        {
            return Ok(WakePhase::Deferred);
        }
        let writer = st
            .connections
            .get(&conn)
            .cloned()
            .ok_or("target_unavailable")?;
        (target.to_owned(), conn, instance, writer)
    };
    let (target, conn, instance, writer) = route;
    let mut permit = None;
    let mut committed = None;
    let placeholder = DaemonToWrapper::Nack {
        id: wake.id.clone(),
        reason: "réveil non admis".into(),
    };
    let result =
        push_control_message_until_checked(&writer, &placeholder, departure_deadline, || {
            let mut st = match state.try_lock() {
                Ok(st) => st,
                Err(std::sync::TryLockError::Poisoned(e)) => e.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => {
                    return Err(std::io::ErrorKind::WouldBlock.into());
                }
            };
            let refusal = || {
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "wake departure refused",
                )
            };
            if agent_communication_project(&st, &wake.owner) != wake.owner_project
                || (!(matches!(wake.source, WakeSource::CliOnce)
                    && wake.owner == "bridget-local-cli")
                    && wake_target_guard(&st, &wake.owner, &wake.target).is_err())
                || wake_native_origin(&st, &wake.owner, wake.owner_native.as_ref()).is_err()
            {
                return Err(refusal());
            }
            if let Some(id) = scheduled_id {
                let current = st
                    .store
                    .wake_schedule(id)
                    .map_err(|_| refusal())?
                    .ok_or_else(refusal)?;
                if current.cancelled_at.is_some()
                    || current.occurrence.as_ref().is_none_or(|w| w.id != wake.id)
                {
                    return Err(refusal());
                }
            }
            let control =
                crate::referent_control::read(st.store.connection()).map_err(|_| refusal())?;
            let current_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|wall| {
                    tick_wall_lower.and_then(|tick| wall.as_secs().checked_sub(tick.as_secs()))
                })
                .and_then(|delta| i64::try_from(delta).ok())
                .and_then(|delta| now.checked_add(delta))
                .ok_or_else(refusal)?;
            let current_clock = clock
                .checked_add(age_before.saturating_add(mono_anchor.elapsed()))
                .ok_or_else(refusal)?;
            if control.paused
                || current_now >= wake.grace_until
                || live_connection_identity(&st, &conn) != Some((target.clone(), instance.clone()))
                || st
                    .router
                    .get_agent(&target)
                    .is_none_or(|route| route.connection_id != conn)
                || st.presences.get(&instance).is_none_or(|p| {
                    p.dnd_until.is_some_and(|until| current_clock < until)
                        || !matches!(p.state.as_str(), "connected" | "busy")
                })
                || st.presences.get(&instance).is_none_or(|presence| {
                    !st.registry
                        .get(&presence.agent_type)
                        .is_ok_and(|definition| {
                            definition
                                .capabilities
                                .execution_paths
                                .iter()
                                .any(|path| path == &definition.protocol)
                        })
                })
                || !st
                    .circuit_breaker
                    .check_at(&wake.owner, &target, current_clock)
            {
                return Err(refusal());
            }
            let expiry = match st
                .idempotency
                .reserve(
                    &key,
                    &wake.canonical_bytes,
                    wake.issued_at,
                    7 * 86400,
                    current_now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .map_err(std::io::Error::other)?
            {
                Reservation::Prepared { expires_at } => expires_at,
                Reservation::Replayed(_) => st
                    .idempotency
                    .prepared_expiry(&key, current_now)
                    .map_err(std::io::Error::other)?
                    .ok_or_else(refusal)?,
                _ => return Err(refusal()),
            };
            let mut message = bridget_core::BridgetMessage::new("bridget", &target, &wake.body);
            message.id = wake.id.clone();
            message.reply = false;
            message.origin = Some(bridget_core::MessageOrigin::System);
            message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
            message.deadline_at = Some(wake.grace_until.max(0) as u64);
            let delivery = SendDelivery {
                delivery_id: uuid::Uuid::new_v4().to_string(),
                recipient_instance_id: instance.clone(),
                delivery_generation: 1,
                expires_at: expiry,
                message_bytes: serde_json::to_vec(&message).map_err(std::io::Error::other)?,
            };
            // La réserve peut attendre SQLite ; refaire le contrôle juste avant le départ.
            let commit_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|wall| {
                    tick_wall_lower.and_then(|tick| wall.as_secs().checked_sub(tick.as_secs()))
                })
                .and_then(|delta| i64::try_from(delta).ok())
                .and_then(|delta| now.checked_add(delta))
                .ok_or_else(refusal)?;
            let commit_clock = clock
                .checked_add(age_before.saturating_add(mono_anchor.elapsed()))
                .ok_or_else(refusal)?;
            if commit_now >= wake.grace_until || Instant::now() >= departure_deadline {
                return Err(refusal());
            }
            st.idempotency
                .begin_send_delivery(&key, &delivery)
                .map_err(std::io::Error::other)?;
            st.circuit_breaker
                .record_at(&wake.owner, &target, commit_clock);
            committed = Some(delivery.clone());
            let frame = DaemonToWrapper::DeliverIdempotent {
                delivery_id: delivery.delivery_id,
                recipient_instance_id: instance.clone(),
                delivery_generation: 1,
                expires_at: expiry,
                message,
                execution: None,
                mid_turn_ok: Some(true),
            };
            permit = Some(st);
            Ok(Some(frame))
        });
    drop(permit);
    if let Some(delivery) = committed {
        if result.is_err() {
            let mut st = state.lock().recover_poison();
            let _ = st
                .idempotency
                .mark_delivery_indeterminate(&delivery.delivery_id, &instance, 1);
        }
        Ok(WakePhase::DispatchingAgent)
    } else {
        Ok(WakePhase::Deferred)
    }
}

/// O(128+B+calendrier borné), tick1s existant ; aucun calendrier sous mutex.
fn scheduled_wakes_tick(state: &Arc<Mutex<DaemonState>>, now: i64, clock: Instant) {
    use crate::wake_schedules::{
        FrozenWake, WakePhase, WakeSource, calculate_cadence, timezone_fingerprint,
    };
    let schedules = {
        let st = state.lock().recover_poison();
        match st.store.list_wake_schedules(None) {
            Ok(s) => s,
            Err(e) => {
                warn!("wake snapshot unavailable: {e}");
                return;
            }
        }
    };
    for schedule in schedules {
        let mut frozen = schedule.occurrence.clone().filter(|w| !w.phase.terminal());
        if schedule.cancelled_at.is_some() && frozen.is_none() {
            continue;
        }
        // Cette lecture prime aussi l'annulation et la perte de l'origine.
        // Seul None autorise ensuite les gardes d'un départ encore absent.
        let committed_thread = if let Some(wake) = frozen.as_ref() {
            if let bridget_transport::protocol::WakeTarget::Thread { id } = &wake.target {
                let Some(operation_id) = wake.thread_operation_id.as_deref() else {
                    continue;
                };
                let canonical_hash = crate::threads::digest(&[
                    b"post",
                    b"v1",
                    id.as_bytes(),
                    wake.body.as_bytes(),
                    b"all",
                    b"reply_to",
                    b"ack",
                ]);
                let st = state.lock().recover_poison();
                match st
                    .store
                    .thread_operation_replay(&wake.owner, operation_id, &canonical_hash)
                {
                    Ok(Some(crate::store::threads::ThreadTxOutcome::Replayed(result)))
                        if result["status"] == "posted"
                            && result["thread_id"].as_str() == Some(id.as_str()) =>
                    {
                        true
                    }
                    Ok(None) => false,
                    _ => continue,
                }
            } else {
                false
            }
        } else {
            false
        };
        let native_status = {
            let st = state.lock().recover_poison();
            wake_native_origin(&st, &schedule.owner, schedule.owner_native.as_ref())
        };
        let owner_closed = matches!(native_status, Err("owner_unavailable"));
        if native_status.is_err() && !owner_closed && !committed_thread {
            continue;
        }
        let cancellation_phase = if committed_thread {
            Some(WakePhase::CommittedThread)
        } else if schedule.cancelled_at.is_some() || owner_closed {
            if let Some(wake) = &frozen {
                let st = state.lock().recover_poison();
                let key = match IdempotencyKey::new(
                    crate::wake_schedules::SCHEDULED_WAKE_SCOPE,
                    OperationKind::Send,
                    &wake.id,
                ) {
                    Ok(key) => key,
                    Err(_) => continue,
                };
                match st.idempotency.send_delivery_phase(&key) {
                    Ok(None) => Some(WakePhase::OwnerUnavailable),
                    Ok(Some(p)) if p == "acked" => Some(WakePhase::Confirmed),
                    Ok(Some(p)) if p == "dispatching" && now < wake.grace_until => None,
                    Ok(Some(_)) => Some(WakePhase::OutcomeUnknown),
                    Err(_) => continue,
                }
            } else {
                None
            }
        } else {
            None
        };
        let tz = match timezone_fingerprint(now) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let calendar = if (frozen.is_none() && (schedule.next_at <= now || tz != schedule.timezone))
            || (cancellation_phase.is_some() && schedule.next_at <= now)
        {
            match calculate_cadence(&schedule, now) {
                Ok(c) => c,
                Err(e) => {
                    warn!("wake calendar unavailable: {e}");
                    continue;
                }
            }
        } else {
            crate::wake_schedules::CalendarSnapshot {
                fingerprint: tz,
                next_at: schedule.next_at,
                due_at: None,
                calendar_date: None,
                next_calendar_date: schedule.next_calendar_date.clone(),
            }
        };
        let current_tz = match timezone_fingerprint(now) {
            Ok(t) => t,
            Err(_) => continue,
        };
        {
            let st = state.lock().recover_poison();
            if owner_closed {
                let _ = st.store.cancel_wake_schedule(
                    Some(&schedule.owner),
                    &schedule.schedule_id,
                    now,
                );
                if let (Some(phase), Some(wake)) = (cancellation_phase, frozen.as_ref()) {
                    let _ = st.store.settle_wake_occurrence(
                        &schedule.schedule_id,
                        &wake.id,
                        phase,
                        calendar.next_at.max(schedule.next_at),
                        calendar.next_calendar_date.as_deref(),
                    );
                }
                continue;
            }
            if let (Some(phase), Some(wake)) = (cancellation_phase, frozen.as_ref()) {
                let _ = st.store.settle_wake_occurrence(
                    &schedule.schedule_id,
                    &wake.id,
                    phase,
                    calendar.next_at.max(schedule.next_at),
                    calendar.next_calendar_date.as_deref(),
                );
                continue;
            }
            if frozen.is_none() && schedule.cancelled_at.is_none() {
                let control = match crate::referent_control::read(st.store.connection()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if control.paused {
                    continue;
                }
                if let Some(due) = calendar.due_at {
                    let target_guard = wake_target_guard(&st, &schedule.owner, &schedule.target);
                    let dormant = target_guard == Err("agent_dormant");
                    if target_guard.is_err() && !dormant {
                        continue;
                    }
                    let mut wake = FrozenWake {
                        version: 1,
                        source: WakeSource::Scheduled,
                        target: schedule.target.clone(),
                        owner: schedule.owner.clone(),
                        owner_native: schedule.owner_native.clone(),
                        owner_project: schedule.owner_project.clone(),
                        due_at: due,
                        issued_at: now,
                        grace_until: due.saturating_add(600),
                        id: format!("wake:{}:{due}", schedule.schedule_id),
                        label: schedule.label.clone(),
                        body: format!(
                            "Réveil programmé par {}\n{}",
                            schedule.owner, schedule.label
                        ),
                        canonical_bytes: Vec::new(),
                        thread_operation_id: matches!(
                            schedule.target,
                            bridget_transport::protocol::WakeTarget::Thread { .. }
                        )
                        .then(|| uuid::Uuid::new_v4().to_string()),
                        delivery_id: None,
                        phase: WakePhase::Prepared,
                    };
                    if wake.freeze_canonical().is_err() {
                        continue;
                    }
                    if matches!(
                        st.store.freeze_wake_occurrence(
                            &schedule.schedule_id,
                            schedule.next_at,
                            &calendar.fingerprint,
                            &current_tz,
                            &wake,
                            calendar.next_at,
                            calendar.calendar_date.as_deref(),
                            calendar.next_calendar_date.as_deref()
                        ),
                        Ok(true)
                    ) {
                        if dormant {
                            // Pas de rattrapage au réveil : cette occurrence est close,
                            // sans émission. La cadence suivante reste programmée.
                            let _ = st.store.settle_wake_occurrence(
                                &schedule.schedule_id,
                                &wake.id,
                                WakePhase::MissedBeforeDeparture,
                                calendar.next_at,
                                calendar.next_calendar_date.as_deref(),
                            );
                            continue;
                        }
                        frozen = Some(wake);
                    }
                } else if calendar.next_at != schedule.next_at
                    || calendar.fingerprint != schedule.timezone
                {
                    let _ = st.store.advance_wake_calendar(
                        &schedule.schedule_id,
                        schedule.next_at,
                        &calendar.fingerprint,
                        &current_tz,
                        calendar.next_at,
                        calendar.next_calendar_date.as_deref(),
                        now,
                    );
                }
            }
        }
        if let Some(wake) = frozen
            && let Ok(phase) =
                deliver_frozen_wake(state, &wake, Some(&schedule.schedule_id), now, clock)
        {
            let future = if phase.terminal() && schedule.next_at <= now {
                match calculate_cadence(&schedule, now) {
                    Ok(c) => c,
                    Err(_) => continue,
                }
            } else {
                calendar.clone()
            };
            let st = state.lock().recover_poison();
            let _ = st.store.settle_wake_occurrence(
                &schedule.schedule_id,
                &wake.id,
                phase,
                future.next_at.max(schedule.next_at),
                future.next_calendar_date.as_deref(),
            );
        }
    }
    let pending = {
        let st = state.lock().recover_poison();
        st.idempotency
            .pending_cli_wakes(now, st.cli_wake_cursor.clone())
    };
    if let Ok(pending) = pending {
        if pending.is_empty() {
            state.lock().recover_poison().cli_wake_cursor = None;
        }
        for (wake, _, _) in pending {
            let outcome = deliver_frozen_wake(state, &wake, None, now, clock);
            let mut st = state.lock().recover_poison();
            st.cli_wake_cursor = Some((wake.issued_at, wake.id.clone()));
            let terminal = match outcome {
                Ok(WakePhase::MissedBeforeDeparture) => {
                    Some(crate::idempotency::PublicResult::Rejected {
                        category: "missed_before_departure".into(),
                        reason: "réveil expiré avant départ".into(),
                    })
                }
                Ok(WakePhase::OutcomeUnknown) => Some(crate::idempotency::PublicResult::Orphaned {
                    reason: "outcome_unknown".into(),
                }),
                Err("owner_unavailable") => Some(crate::idempotency::PublicResult::Rejected {
                    category: "owner_unavailable".into(),
                    reason: "origine native terminée".into(),
                }),
                _ => None,
            };
            if let Some(result) = terminal
                && let Ok(key) = IdempotencyKey::new(
                    crate::wake_schedules::CLI_WAKE_SCOPE,
                    OperationKind::Send,
                    &wake.id,
                )
            {
                if matches!(
                    st.idempotency.cli_wake(&wake.id),
                    Ok(Some((_, crate::idempotency::RecordState::Prepared, _)))
                ) {
                    let _ = st.idempotency.transition(
                        &key,
                        crate::idempotency::RecordState::Prepared,
                        crate::idempotency::RecordState::Dispatching,
                    );
                }
                let _ = st.idempotency.finalize(&key, result);
            }
        }
    }
}

fn deliver_to_agent(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    target_name: &str,
    body: &str,
) -> std::io::Result<String> {
    let msg = bridget_core::BridgetMessage::new("bridget", target_name, body);
    let message_id = msg.id.clone();
    let dtw = DaemonToWrapper::Deliver {
        message: msg,
        mid_turn_ok: Some(true),
    };
    // Les notifications système partagent la même borne que les remises :
    // une cible non lectrice ne doit jamais immobiliser le thread de rappel.
    push_control_message_until(writer, &dtw, Instant::now() + CANCEL_NOTIFICATION_BUDGET)?;

    info!("Message délivré à {}", target_name);
    Ok(message_id)
}

// Le worker de notification existant reste unique. Aucun accès Store ici.
enum ObservationJob {
    #[cfg(test)]
    Control(Instant, DeferredControl),
    Notice {
        deadline: Instant,
        control: DeferredControl,
        owner: String,
        fact_id: String,
        result_tx: mpsc::SyncSender<ObservationWriteResult>,
    },
}
struct ObservationWriteResult {
    owner: String,
    fact_id: String,
    outcome: crate::store::observation_missed::ObservationWriteOutcome,
}
fn observation_output() -> (
    mpsc::SyncSender<ObservationJob>,
    Arc<std::sync::atomic::AtomicU64>,
) {
    let (tx, rx) = mpsc::sync_channel::<ObservationJob>(64);
    let lost = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let failures = lost.clone();
    thread::spawn(move || {
        while let Ok(job) = rx.recv() {
            let (deadline, control, reply) = match job {
                #[cfg(test)]
                ObservationJob::Control(deadline, control) => (deadline, control, None),
                ObservationJob::Notice {
                    deadline,
                    control,
                    owner,
                    fact_id,
                    result_tx,
                } => (deadline, control, Some((owner, fact_id, result_tx))),
            };
            use crate::store::observation_missed::ObservationWriteOutcome as O;
            let outcome = if Instant::now() >= deadline {
                O::FailedBeforeWrite
            } else {
                match push_control_message_until(
                    &control.writer,
                    &control.message,
                    deadline.min(Instant::now() + CANCEL_NOTIFICATION_BUDGET),
                ) {
                    Ok(()) => O::Written,
                    Err(_) => O::WriteUnknown,
                }
            };
            if !matches!(outcome, O::Written) {
                failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            if let Some((owner, fact_id, result_tx)) = reply {
                let _ = result_tx.try_send(ObservationWriteResult {
                    owner,
                    fact_id,
                    outcome,
                });
            }
        }
    });
    (tx, lost)
}

fn observe_fact(st: &mut DaemonState, fact: crate::observation::Fact) {
    observe_fact_at(st, fact, unix_now_secs(), Instant::now());
}
// Même puits métier avec horloges explicites ; pas d'horloge globale de test.
fn observe_fact_at(st: &mut DaemonState, fact: crate::observation::Fact, now: i64, clock: Instant) {
    let before = st.observations.subscriptions.clone();
    let revision = st.observations.revision;
    let notices = st.observations.observe(fact, clock);
    let snapshot = if revision != st.observations.revision {
        match st.observations.snapshot() {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                st.observations.subscriptions = before;
                st.observations.revision = revision;
                st.observation_lost
                    .fetch_add(notices.len() as u64, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        }
    } else {
        None
    };
    if !queue_observation_notifications_at(st, notices, now, clock, snapshot.as_deref()) {
        st.observations.subscriptions = before;
        st.observations.revision = revision;
    }
}
fn queue_observation_notifications(
    st: &mut DaemonState,
    notices: Vec<crate::observation::Notification>,
) {
    let snapshot = st.observations.snapshot().ok();
    let _ = queue_observation_notifications_at(
        st,
        notices,
        unix_now_secs(),
        Instant::now(),
        snapshot.as_deref(),
    );
}
// Snapshotonce et provenance sont committés avant le dépôt borné/nonbloquant.
fn queue_observation_notifications_at(
    st: &mut DaemonState,
    notices: Vec<crate::observation::Notification>,
    now: i64,
    clock: Instant,
    snapshot: Option<&[u8]>,
) -> bool {
    let accepted = match st.store.save_observation_batch(snapshot, &notices, now) {
        Ok(ids) => ids,
        Err(error) => {
            st.observation_lost
                .fetch_add(notices.len() as u64, std::sync::atomic::Ordering::Relaxed);
            warn!("observation snapshot/provenance unavailable: {error}");
            return false;
        }
    };
    for notice in notices {
        if !accepted.contains(&notice.fact_id) {
            continue;
        }
        let target = st.router.get_agent(&notice.owner).and_then(|route| {
            if !st.identity_credentials.contains_key(&route.connection_id) {
                return None;
            }
            let instance = st.conn_instances.get(&route.connection_id)?;
            let presence = st.presences.get(instance)?;
            if crate::agent_tracking::dormant(st.store.connection(), &notice.owner).unwrap_or(true)
                || presence.dnd_until.is_some_and(|until| clock < until)
            {
                return None;
            }
            st.connections.get(&route.connection_id).cloned()
        });
        let Some(writer) = target else {
            // Le fait conservé ne prouve pas la remise de sa notice immédiate.
            st.observation_lost
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let _ = st
                .store
                .observation_owner_presence(&notice.owner, false, now);
            let _ = st.store.finish_observation_write(
                &notice.owner,
                &notice.fact_id,
                crate::store::observation_missed::ObservationWriteOutcome::FailedBeforeWrite,
                now,
            );
            continue;
        };
        let mut message = bridget_core::BridgetMessage::new("bridget", &notice.owner, &notice.body);
        message.id = format!("bridget-observation:{}", notice.fact_id);
        message.origin = Some(bridget_core::MessageOrigin::System);
        let job = ObservationJob::Notice {
            deadline: clock + Duration::from_secs(5),
            control: DeferredControl {
                writer,
                message: DaemonToWrapper::Deliver {
                    message,
                    mid_turn_ok: Some(true),
                },
            },
            owner: notice.owner,
            fact_id: notice.fact_id,
            result_tx: st.observation_result_tx.clone(),
        };
        if st.observation_tx.try_send(job).is_err() {
            st.observation_lost
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        // Full : conserver le pending et sa deadline initiale. Le tick le solde unknown.
    }
    true
}

/// Même tick1s ; preuves de retour avant gel, ACK099 avant expiration/supersession.
fn observer_return_tick(state: &Arc<Mutex<DaemonState>>, now: i64, clock: Instant) {
    use crate::wake_schedules::{FrozenWake, MISSED_SUMMARY_SCOPE, WakePhase, WakeSource};
    let summaries = {
        let mut st = state.lock().recover_poison();
        // Une deadline durable dépassée prime sur un callback arrivé tardivement.
        if let Err(error) = st.store.maintain_observation_missed(now, false) {
            warn!("observation maintenance unavailable: {error}");
            return;
        }
        while let Ok(result) = st.observation_result_rx.try_recv() {
            if let Err(error) = st.store.finish_observation_write(
                &result.owner,
                &result.fact_id,
                result.outcome,
                now,
            ) {
                warn!("observation completion unavailable: {error}");
            }
        }
        // Une seule décision par owner. Une présence historique ne décrit pas la route actuelle.
        let owners: std::collections::BTreeSet<_> = st
            .presences
            .values()
            .map(|presence| presence.name.clone())
            .collect();
        for owner in owners {
            let available = st
                .router
                .get_agent(&owner)
                .and_then(|route| {
                    if !st.identity_credentials.contains_key(&route.connection_id) {
                        return None;
                    }
                    let instance = st.conn_instances.get(&route.connection_id)?;
                    if live_connection_identity(&st, &route.connection_id)
                        != Some((owner.clone(), instance.clone()))
                    {
                        return None;
                    }
                    let presence = st.presences.get(instance)?;
                    Some(
                        presence.name == owner
                            && !crate::agent_tracking::dormant(st.store.connection(), &owner)
                                .unwrap_or(true)
                            && matches!(presence.state.as_str(), "connected" | "busy")
                            && presence.dnd_until.is_none_or(|until| clock >= until),
                    )
                })
                .unwrap_or(false);
            if st
                .store
                .observation_owner_presence(&owner, available, now)
                .is_err()
            {
                continue;
            }
            if available {
                match st.store.observation_summary_current(&owner) {
                    Ok(Some(summary)) => {
                        let key = match IdempotencyKey::new(
                            MISSED_SUMMARY_SCOPE,
                            OperationKind::Send,
                            &summary.summary_id,
                        ) {
                            Ok(key) => key,
                            Err(_) => continue,
                        };
                        match st.idempotency.send_delivery_phase(&key) {
                            Ok(Some(phase)) if phase == "acked" => {
                                if let Ok(Some(delivery)) =
                                    st.idempotency.stored_send_delivery(&key)
                                {
                                    let _ = st.store.bind_observation_summary_delivery(
                                        &summary.summary_id,
                                        &delivery.delivery_id,
                                        &delivery.recipient_instance_id,
                                        delivery.delivery_generation,
                                    );
                                    let _ = st.store.confirm_observation_summary(
                                        &owner,
                                        &summary.summary_id,
                                        summary.generation,
                                        &delivery.delivery_id,
                                        &delivery.recipient_instance_id,
                                        delivery.delivery_generation,
                                    );
                                }
                            }
                            Err(_) => continue,
                            _ => {}
                        }
                    }
                    Err(_) => continue,
                    Ok(None) => {}
                }
                let _ = st.store.freeze_observation_summary(&owner, now);
            }
        }
        match st.store.observation_summaries_pending() {
            Ok(s) => s,
            Err(error) => {
                warn!("observation summaries unavailable: {error}");
                return;
            }
        }
    };
    for summary in summaries {
        let key = match IdempotencyKey::new(
            MISSED_SUMMARY_SCOPE,
            OperationKind::Send,
            &summary.summary_id,
        ) {
            Ok(key) => key,
            Err(_) => continue,
        };
        {
            let mut st = state.lock().recover_poison();
            match st.idempotency.send_delivery_phase(&key) {
                Ok(Some(phase)) if phase == "acked" => {
                    if let Ok(Some(delivery)) = st.idempotency.stored_send_delivery(&key) {
                        let _ = st.store.bind_observation_summary_delivery(
                            &summary.summary_id,
                            &delivery.delivery_id,
                            &delivery.recipient_instance_id,
                            delivery.delivery_generation,
                        );
                        let _ = st.store.confirm_observation_summary(
                            &summary.owner,
                            &summary.summary_id,
                            summary.generation,
                            &delivery.delivery_id,
                            &delivery.recipient_instance_id,
                            delivery.delivery_generation,
                        );
                    }
                    continue;
                }
                Ok(Some(phase)) => {
                    if phase != "dispatching" || now >= summary.expires_at {
                        let _ = st.store.expire_observation_summary(
                            &summary.summary_id,
                            now.max(summary.expires_at),
                        );
                    }
                    continue;
                }
                Err(error) => {
                    warn!("observation ACK proof unavailable: {error}");
                    continue;
                }
                Ok(None) => {}
            }
            if now >= summary.expires_at {
                let _ = st
                    .store
                    .expire_observation_summary(&summary.summary_id, now);
                continue;
            }
        }
        let project = {
            let st = state.lock().recover_poison();
            agent_communication_project(&st, &summary.owner)
        };
        let mut wake = FrozenWake {
            version: 1,
            source: WakeSource::MissedSummary,
            target: bridget_transport::protocol::WakeTarget::SelfTarget {},
            owner: summary.owner.clone(),
            owner_native: None,
            owner_project: project,
            due_at: summary.issued_at,
            issued_at: summary.issued_at,
            grace_until: summary.expires_at,
            id: summary.summary_id.clone(),
            label: String::new(),
            body: summary.body,
            canonical_bytes: Vec::new(),
            thread_operation_id: None,
            delivery_id: None,
            phase: WakePhase::Prepared,
        };
        if wake.freeze_canonical().is_err() {
            continue;
        }
        if let Ok(WakePhase::DispatchingAgent) = deliver_frozen_wake(state, &wake, None, now, clock)
        {
            let mut st = state.lock().recover_poison();
            match st.idempotency.stored_send_delivery(&key) {
                Ok(Some(delivery)) => {
                    let _ = st.store.bind_observation_summary_delivery(
                        &summary.summary_id,
                        &delivery.delivery_id,
                        &delivery.recipient_instance_id,
                        delivery.delivery_generation,
                    );
                }
                Ok(None) => {}
                Err(error) => warn!("observation delivery proof unavailable: {error}"),
            }
        }
    }
}

#[cfg(test)]
#[path = "daemon_core_089_cancel_pressure_tests.rs"]
mod core_089_cancel_pressure_tests;

/// Budget global d'une sortie de contrôle ou remise, incluant le verrou writer.
/// L'issue durable ne dépend pas de la disponibilité du destinataire.
const CANCEL_NOTIFICATION_BUDGET: Duration = Duration::from_secs(1);

fn push_control_message_until(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    message: &DaemonToWrapper,
    deadline: Instant,
) -> std::io::Result<()> {
    push_control_message_until_checked(writer, message, deadline, || Ok(None))
}

fn push_control_message_until_checked(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    message: &DaemonToWrapper,
    deadline: Instant,
    mut before_emit: impl FnMut() -> std::io::Result<Option<DaemonToWrapper>>,
) -> std::io::Result<()> {
    let timeout = || {
        std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "budget de notification dépassé",
        )
    };
    let (mut writer, replacement_message) = loop {
        if Instant::now() >= deadline {
            return Err(timeout());
        }
        match writer.try_lock() {
            Ok(writer) => match before_emit() {
                Ok(message) => break (writer, message),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    // Ne jamais attendre state avec writer détenu : les
                    // chemins historiques peuvent prendre les verrous à l'inverse.
                    drop(writer);
                    thread::sleep(
                        deadline
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(1)),
                    );
                }
                Err(error) => return Err(error),
            },
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(std::io::Error::other("writer empoisonné"));
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                thread::sleep(
                    deadline
                        .saturating_duration_since(Instant::now())
                        .min(Duration::from_millis(1)),
                );
            }
        }
    };
    // Le contrôle intervient après acquisition du writer. Le callback peut
    // conserver le verrou métier jusqu'à la fin de l'écriture.
    let mut json = encode(replacement_message.as_ref().unwrap_or(message))
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    json.push('\n');
    // Reprendre le tampon EXISTANT sans le perdre ni le rejouer. into_parts
    // ne flush pas : chaque write ci-dessous reçoit le temps encore disponible.
    // Ne pas changer O_NONBLOCK, qui affecterait aussi le lecteur du wrapper.
    let replacement = BufWriter::with_capacity(writer.capacity(), writer.get_ref().try_clone()?);
    let old = std::mem::replace(&mut *writer, replacement);
    let (old_socket, pending) = old.into_parts();
    let result = (|| {
        let pending = pending.map_err(|_| std::io::Error::other("tampon du writer indéterminé"))?;
        let old_timeout = old_socket.write_timeout()?;
        let result = (|| {
            for mut bytes in [pending.as_slice(), json.as_bytes()] {
                while !bytes.is_empty() {
                    let remaining = deadline
                        .checked_duration_since(Instant::now())
                        .filter(|remaining| !remaining.is_zero())
                        .ok_or_else(timeout)?;
                    writer.get_ref().set_write_timeout(Some(remaining))?;
                    match writer.get_mut().write(bytes) {
                        Ok(0) => return Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
                        Ok(count) => bytes = &bytes[count..],
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(error) => return Err(error),
                    }
                }
            }
            Ok(())
        })();
        let restored = writer.get_ref().set_write_timeout(old_timeout);
        result.and(restored)
    })();
    if result.is_err() {
        // Une ligne partielle n'est pas rejouable sur ce flux : fermer plutôt
        // que concaténer une future trame à des octets incomplets.
        let _ = writer.get_ref().shutdown(std::net::Shutdown::Both);
    }
    result
}

fn push_control_message(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    message: &DaemonToWrapper,
) -> bool {
    let Ok(json) = encode(message) else {
        return false;
    };
    let Ok(mut writer) = writer.lock() else {
        return false;
    };
    let written = writeln!(writer, "{json}")
        .and_then(|_| writer.flush())
        .is_ok();
    if !written && matches!(message, DaemonToWrapper::End { .. }) {
        // Une vue ne doit jamais rester suspendue après l'échec du terminal :
        // l'EOF de la connexion est alors son signal de clôture garanti.
        let _ = writer.get_ref().shutdown(std::net::Shutdown::Both);
    }
    written
}
fn delegated_runtime_frame(
    event: crate::idempotency::DelegatedRuntimeEventRecord,
) -> DelegatedRuntimeEventFrame {
    DelegatedRuntimeEventFrame {
        cursor: event.cursor,
        event_id: event.event_id,
        link_id: event.link_id,
        child_instance_id: event.child_instance_id,
        child_execution_id: event.child_execution_id,
        kind: event.kind,
        code: event.code,
        reference: event.reference,
        observed_at: event.observed_at,
        project: event.project,
    }
}

fn replay_delegated_runtime_events_for_connection(state: &Arc<Mutex<DaemonState>>, conn_id: &str) {
    let (parent_instance_id, writer, fleet) = {
        let st = state.lock().recover_poison();
        // Comme la remise en direct, le rejeu appartient au parent primaire.
        // Un auxiliaire MCP attend une réponse RPC, jamais cet avis asynchrone.
        if st.auxiliary_connections.contains(conn_id) {
            return;
        }
        let Some(parent_instance_id) = st.conn_instances.get(conn_id).cloned() else {
            return;
        };
        let Some(writer) = st.connections.get(conn_id).cloned() else {
            return;
        };
        (parent_instance_id, writer, Arc::clone(&st.fleet))
    };
    let events = match fleet.delegated_runtime_events_for_parent(&parent_instance_id) {
        Ok(events) => events,
        Err(error) => {
            warn!("rejeu des incidents délégués impossible: {error}");
            return;
        }
    };
    for event in events {
        if !push_control_message(
            &writer,
            &DaemonToWrapper::DelegatedRuntimeEvent {
                event: delegated_runtime_frame(event),
            },
        ) {
            break;
        }
    }
}

struct DeferredControl {
    writer: Arc<Mutex<BufWriter<UnixStream>>>,
    message: DaemonToWrapper,
}

fn execute_controls(controls: Vec<DeferredControl>) -> Vec<DeferredControl> {
    controls
        .into_iter()
        .filter(|control| {
            let written = push_control_message(&control.writer, &control.message);
            #[cfg(feature = "test-support")]
            if written && matches!(&control.message, DaemonToWrapper::DeliverIdempotent { .. }) {
                // Frontière après writeln+flush réels, hors verrou writer ; aucun ACK supposé.
                crate::test_sync::checkpoint("after_idempotent_socket_push");
            }
            !written
        })
        .collect()
}

fn defer_control(
    state: &DaemonState,
    conn_id: &str,
    message: DaemonToWrapper,
    controls: &mut Vec<DeferredControl>,
) {
    if let Some(writer) = state.connections.get(conn_id) {
        controls.push(DeferredControl {
            writer: writer.clone(),
            message,
        });
    }
}

fn purge_expired_attach_sends(state: &mut DaemonState) {
    let now = Instant::now();
    state
        .pending_attach_sends
        .retain(|_, pending| pending.expires_at > now);
}

/// Le rôle éphémère vient du Register typé et de la propriété de la route.
/// Un UUID, un nom affiché ou le mode déclaré ne prouvent jamais ce rôle.
fn is_ephemeral_cli_route(
    conn_id: &str,
    route: Option<&bridget_core::router::RegisteredAgent>,
) -> bool {
    route.is_some_and(|agent| {
        agent.connection_id == conn_id
            && matches!(&agent.agent_type, bridget_core::AgentType::Custom(kind) if kind == "cli")
    })
}

/// Sort de l'attribution de l'expéditeur pour un envoi hors rôle attach.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SenderAttribution {
    /// Le nom est remplacé par celui de la connexion émettrice.
    UseConnectionName,
    /// L'émetteur s'est nommé, mais ce nom n'est adressable par personne :
    /// l'envoi est refusé au lieu d'être attribué à une identité éphémère.
    RefuseUnaddressable,
    /// L'émetteur s'est nommé sous l'identité d'un agent connecté depuis une
    /// connexion de ligne de commande, qui ne prouve pas être cet agent :
    /// l'envoi est refusé pour usurpation.
    RefuseImpersonation,
}

/// Décide à qui attribuer un envoi. Un wrapper garde toujours son nom
/// enregistré (anti-usurpation). Une connexion de ligne de commande est
/// éphémère et ne prouve rien sur qui la lance :
///
/// - si elle se nomme explicitement (`--from`), elle ne peut ni emprunter
///   l'identité d'un agent connecté — ce serait une usurpation — ni se donner
///   un nom que personne ne porte — ce serait un expéditeur sans adresse de
///   retour. Les deux cas sont refusés, avec des motifs distincts ;
/// - un nom hérité de l'environnement ne constitue pas davantage une preuve :
///   les clients rattachés utilisent RegisterAuxiliary et ne passent plus par
///   la route CLI éphémère.
///
/// Complexité : O(1).
pub(crate) fn resolve_sender_attribution(
    ephemeral_cli: bool,
    from_declared: bool,
    from_is_addressable: bool,
) -> SenderAttribution {
    if !ephemeral_cli {
        return SenderAttribution::UseConnectionName;
    }
    if from_declared {
        if from_is_addressable {
            return SenderAttribution::RefuseImpersonation;
        }
        return SenderAttribution::RefuseUnaddressable;
    }
    if from_is_addressable {
        return SenderAttribution::RefuseImpersonation;
    }
    SenderAttribution::UseConnectionName
}

#[cfg(test)]
mod attribution_emetteur_cli_tests {
    use super::{SenderAttribution, is_ephemeral_cli_route, resolve_sender_attribution};

    #[test]
    fn cli_identifie_par_type_et_propriete_de_connexion_jamais_par_identifiant() {
        let mut route = bridget_core::router::RegisteredAgent {
            agent_id: "89000000-0000-4000-8000-000000000091".into(),
            agent_type: bridget_core::AgentType::Custom("cli".into()),
            connection_id: "cli-connection".into(),
        };
        assert!(is_ephemeral_cli_route("cli-connection", Some(&route)));
        assert!(!is_ephemeral_cli_route("other-connection", Some(&route)));
        assert!(!is_ephemeral_cli_route("cli-connection", None));
        for kind in [
            bridget_core::AgentType::Codex,
            bridget_core::AgentType::Custom("mcp".into()),
        ] {
            route.agent_type = kind;
            assert!(!is_ephemeral_cli_route("cli-connection", Some(&route)));
        }
    }

    /// Témoin de l'assertion métier : un émetteur en ligne de commande qui se
    /// nomme sous une identité que personne ne porte est refusé, et non
    /// réattribué en silence à `cli-send-<pid>`. C'est la propriété demandée —
    /// une identité déclarée doit être adressable en retour.
    #[test]
    fn emetteur_cli_nomme_non_adressable_est_refuse() {
        assert_eq!(
            resolve_sender_attribution(true, true, false),
            SenderAttribution::RefuseUnaddressable
        );
    }

    /// Résout l'attribution pour un nom littéral, en donnant la liste des
    /// agents connectés. Le nom déclaré reste visible dans le témoin, ce qui
    /// permet d'attester le cas « humain » nommément et pas par un booléen.
    fn attribution(
        ephemeral_cli: bool,
        declare: Option<&str>,
        agents_connectes: &[&str],
    ) -> SenderAttribution {
        let nom_porte = declare.unwrap_or("human");
        resolve_sender_attribution(
            ephemeral_cli,
            declare.is_some(),
            agents_connectes.contains(&nom_porte),
        )
    }

    /// Témoin exigé par le mandat 3d7053a6 : c'est le cas qui a coûté une
    /// fausse alerte de dette humaine le 28/08. Deux sondes de rc7-flux et une
    /// du référent ont été inscrites au ledger sous « humain » sans qu'aucun
    /// humain n'écrive, et un agent a été mis en demeure de répondre à un
    /// message que personne n'avait envoyé.
    #[test]
    fn un_cli_temporaire_ne_peut_pas_emettre_sous_le_nom_humain() {
        assert_eq!(
            attribution(true, Some("humain"), &["humain", "bridget", "rc7-flux"]),
            SenderAttribution::RefuseImpersonation
        );
    }

    /// Généralisation du précédent : aucun agent connecté ne peut être emprunté
    /// depuis une connexion de ligne de commande. Ce témoin remplace celui de la
    /// session 060, `emetteur_cli_nomme_adressable_est_conserve`, qui attestait
    /// exactement le comportement que le mandat demande de fermer.
    #[test]
    fn emetteur_cli_ne_peut_pas_usurper_un_agent_connecte() {
        assert_eq!(
            attribution(true, Some("bridget"), &["humain", "bridget"]),
            SenderAttribution::RefuseImpersonation
        );
    }

    /// Un nom hérité ne vaut pas la preuve privée du wrapper.
    #[test]
    fn nom_herite_de_l_environnement_ne_prouve_pas_identite() {
        assert_eq!(
            resolve_sender_attribution(true, false, true),
            SenderAttribution::RefuseImpersonation
        );
    }

    /// Contrôle de non-régression : sans `--from`, le comportement par défaut
    /// reste l'attribution au nom de connexion. Le mandat ne demandait pas de
    /// le changer.
    #[test]
    fn emetteur_cli_sans_nom_declare_garde_le_defaut() {
        assert_eq!(
            resolve_sender_attribution(true, false, false),
            SenderAttribution::UseConnectionName
        );
    }

    /// Un wrapper ne peut pas emprunter une autre identité, même en se
    /// nommant : son nom enregistré fait foi.
    #[test]
    fn wrapper_ne_peut_pas_usurper_par_from() {
        assert_eq!(
            resolve_sender_attribution(false, true, false),
            SenderAttribution::UseConnectionName
        );
    }
}

#[derive(Debug)]
struct AttachSubscriptionRefusal {
    reason: AttachRefusal,
    mode: Option<PresenceMode>,
    location: Option<String>,
}

impl AttachSubscriptionRefusal {
    fn without_presence(reason: AttachRefusal) -> Self {
        Self {
            reason,
            mode: None,
            location: None,
        }
    }
}

fn attach_refusal_for_subscription(
    state: &DaemonState,
    agent: &str,
) -> Result<String, AttachSubscriptionRefusal> {
    let registered = match state.router.get_agent(agent) {
        Some(registered) => registered,
        None if state
            .presences
            .values()
            .any(|presence| presence.name == agent && presence.state == "stopped") =>
        {
            return Err(AttachSubscriptionRefusal::without_presence(
                AttachRefusal::AgentStopped,
            ));
        }
        None => {
            return Err(AttachSubscriptionRefusal::without_presence(
                AttachRefusal::AgentUnknown,
            ));
        }
    };
    let presence = state
        .conn_instances
        .get(&registered.connection_id)
        .and_then(|instance_id| state.presences.get(instance_id));
    if !presence.is_some_and(|presence| presence.journal_available) {
        return Err(AttachSubscriptionRefusal {
            reason: AttachRefusal::JournalUnavailable,
            mode: presence.and_then(|presence| presence.mode),
            location: presence.and_then(|presence| presence.location.clone()),
        });
    }
    if !state.connections.contains_key(&registered.connection_id) {
        return Err(AttachSubscriptionRefusal::without_presence(
            AttachRefusal::WrapperUnavailable,
        ));
    }
    Ok(registered.connection_id.clone())
}

/// Retire les souscriptions affectées sous le verrou. Les E/S et les `join`
/// retournés doivent impérativement être exécutés après l'avoir relâché.
fn close_attach_subscriptions(
    state: &mut DaemonState,
    conn_id: &str,
) -> (Vec<DeferredControl>, Vec<Arc<AttachView>>) {
    let mut controls = Vec::new();
    let mut views = Vec::new();
    let affected = state
        .attach_subscriptions
        .iter()
        .filter(|(_, subscription)| {
            subscription.attach_conn == conn_id || subscription.wrapper_conn == conn_id
        })
        .map(|(id, subscription)| (id.clone(), subscription.clone()))
        .collect::<Vec<_>>();
    for (subscription_id, subscription) in affected {
        state.attach_subscriptions.remove(&subscription_id);
        let view = state.attach_views.remove(&subscription_id);
        if subscription.attach_conn == conn_id {
            defer_control(
                state,
                &subscription.wrapper_conn,
                DaemonToWrapper::Unsubscribe { subscription_id },
                &mut controls,
            );
        } else if let Some(view) = &view {
            let _ = view.enqueue(DaemonToWrapper::End {
                subscription_id,
                reason: "wrapper indisponible".to_string(),
            });
        }
        if let Some(view) = view {
            views.push(view);
        }
    }
    state
        .pending_attach_sends
        .retain(|_, pending| pending.conn_id != conn_id);
    (controls, views)
}

fn collect_closed_attach_views(
    state: &mut DaemonState,
) -> (Vec<DeferredControl>, Vec<Arc<AttachView>>) {
    let mut controls = Vec::new();
    let mut views = Vec::new();
    while let Ok(subscription_id) = state.view_closed_rx.try_recv() {
        let Some(subscription) = state.attach_subscriptions.remove(&subscription_id) else {
            continue;
        };
        if let Some(view) = state.attach_views.remove(&subscription_id) {
            views.push(view);
        }
        defer_control(
            state,
            &subscription.wrapper_conn,
            DaemonToWrapper::Unsubscribe {
                subscription_id: subscription_id.clone(),
            },
            &mut controls,
        );
        defer_control(
            state,
            &subscription.attach_conn,
            DaemonToWrapper::End {
                subscription_id,
                reason: "vue trop lente".to_string(),
            },
            &mut controls,
        );
    }
    (controls, views)
}

// Fonction pour exposer les métriques publiquement (M-005)
pub fn get_metrics() -> &'static Metrics {
    // Note: ceci est un stub pour l'observabilité
    // Dans une implémentation complète, les métriques seraient partagées globalement
    use std::sync::OnceLock;
    static METRICS: OnceLock<Metrics> = OnceLock::new();
    METRICS.get_or_init(Metrics::new)
}

struct SupervisedChild(RunningManagedChild);

impl SupervisedChild {
    fn try_status(
        &mut self,
    ) -> Result<Option<ManagedStatus>, crate::managed_process::ManagedProcessError> {
        self.0.try_status()
    }
    fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        self.0.try_wait()
    }
    fn status_is_open(&self) -> bool {
        self.0.status_is_open()
    }
    fn remove_marker(&self) -> Result<(), std::io::Error> {
        self.0.remove_marker()
    }
    fn stop(&mut self) -> Result<ManagedStopResult, crate::managed_process::ManagedProcessError> {
        self.0.stop_group(
            MANAGED_STOP_COOPERATIVE_GRACE,
            MANAGED_STOP_FORCED_GRACE,
            MANAGED_STOP_POLL,
        )
    }
    fn request_daemon_shutdown(
        &mut self,
    ) -> Result<(), crate::managed_process::ManagedProcessError> {
        self.0.request_group_termination()
    }
    fn reap_daemon_shutdown(
        &mut self,
    ) -> Result<bool, crate::managed_process::ManagedProcessError> {
        self.0.reap_terminated_group()
    }
}

struct SupervisedProcess {
    prepared: PreparedSpawn,
    child: SupervisedChild,
    registered: Option<(String, String)>,
    connected: bool,
    failure_sent: bool,
    stop: Arc<ManagedStopControl>,
    stop_attempted: bool,
}

/// Éteint toute la flotte hôte en deux phases, afin de ne pas additionner le
/// délai d'arrêt de chaque groupe lorsque le daemon reçoit son propre arrêt.
/// Les groupes qui résistent restent marqués pour la réconciliation suivante.
fn shutdown_active_managed_processes(active: &mut HashMap<String, SupervisedProcess>) {
    for process in active.values_mut() {
        if let Err(error) = process.child.request_daemon_shutdown() {
            warn!(
                "arrêt daemon: signal du groupe {} impossible: {error}",
                process.prepared.lease.name
            );
        }
    }

    let deadline = Instant::now() + MANAGED_STOP_FORCED_GRACE;
    loop {
        let mut survivors = Vec::new();
        for process in active.values_mut() {
            match process.child.reap_daemon_shutdown() {
                Ok(true) => {}
                Ok(false) => survivors.push(process.prepared.lease.name.clone()),
                Err(error) => {
                    warn!(
                        "arrêt daemon: état du groupe {} illisible: {error}",
                        process.prepared.lease.name
                    );
                    survivors.push(process.prepared.lease.name.clone());
                }
            }
        }
        if survivors.is_empty() {
            break;
        }
        if Instant::now() >= deadline {
            warn!("arrêt daemon: groupes encore vivants après SIGTERM: {survivors:?}");
            break;
        }
        thread::sleep(MANAGED_STOP_POLL);
    }
    active.clear();
}

pub(crate) fn spawn_managed_supervisor_thread(
    fleet: Arc<FleetSupervisor>,
    config: &DaemonConfig,
    commands: Receiver<ManagedSupervisorCommand>,
    events: Sender<ManagedSupervisorEvent>,
    executable_override: Option<PathBuf>,
) -> thread::JoinHandle<()> {
    let marker_store = ManagedMarkerStore::at_directory(
        config
            .db_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("managed"),
    );
    let stderr_store = ManagedStderrStore::at_directory(
        config
            .db_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("managed-stderr"),
    );
    let retention_days = config.retention_days;
    thread::spawn(move || {
        if let Err(error) = stderr_store.purge_older_than_days(retention_days) {
            warn!("purge stderr des équipiers impossible: {error}");
        }
        let mut last_stderr_purge = Instant::now();
        let mut active = HashMap::<String, SupervisedProcess>::new();
        loop {
            match commands.recv_timeout(Duration::from_millis(50)) {
                Ok(ManagedSupervisorCommand::Shutdown) => {
                    shutdown_active_managed_processes(&mut active);
                    return;
                }
                Ok(command) => handle_managed_command(
                    command,
                    &fleet,
                    &marker_store,
                    &stderr_store,
                    &events,
                    &mut active,
                    executable_override.as_ref(),
                ),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    shutdown_active_managed_processes(&mut active);
                    return;
                }
            }
            while let Ok(command) = commands.try_recv() {
                if matches!(&command, ManagedSupervisorCommand::Shutdown) {
                    shutdown_active_managed_processes(&mut active);
                    return;
                }
                handle_managed_command(
                    command,
                    &fleet,
                    &marker_store,
                    &stderr_store,
                    &events,
                    &mut active,
                    executable_override.as_ref(),
                );
            }
            poll_managed_processes(&fleet, &events, &mut active);
            if last_stderr_purge.elapsed() >= Duration::from_secs(60 * 60) {
                if let Err(error) = stderr_store.purge_older_than_days(retention_days) {
                    warn!("purge stderr des équipiers impossible: {error}");
                }
                last_stderr_purge = Instant::now();
            }
        }
    })
}

fn handle_managed_command(
    command: ManagedSupervisorCommand,
    fleet: &FleetSupervisor,
    marker_store: &ManagedMarkerStore,
    stderr_store: &ManagedStderrStore,
    events: &Sender<ManagedSupervisorEvent>,
    active: &mut HashMap<String, SupervisedProcess>,
    executable_override: Option<&PathBuf>,
) {
    match command {
        ManagedSupervisorCommand::Shutdown => {
            shutdown_active_managed_processes(active);
        }
        ManagedSupervisorCommand::Start { prepared, stop } => {
            let identity = ManagedIdentity {
                instance_id: prepared.lease.instance_id.clone(),
                command_id: prepared.lease.command_id.clone(),
                generation: prepared.lease.generation,
            };
            let start = (|| {
                let (stderr, _) = stderr_store.open(&prepared.lease.name, &identity)?;
                let executable = executable_override
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(std::env::current_exe)?;
                let launch = ManagedLaunch {
                    bootstrap_executable: executable.clone(),
                    identity,
                    wrapper_executable: executable,
                    wrapper_args: vec![
                        "managed-wrapper".to_string(),
                        prepared.agent_type.clone(),
                        prepared.lease.name.clone(),
                        serde_json::to_string(&prepared.resolved_definition)
                            .map_err(std::io::Error::other)?,
                    ],
                    cwd: prepared.cwd.clone(),
                    env: {
                        #[cfg(feature = "test-support")]
                        {
                            let mut env = prepared.env.clone();
                            if let Some(path) = std::env::var_os(crate::test_sync::DIRECTORY_ENV) {
                                env.insert(crate::test_sync::DIRECTORY_ENV.to_string(), path);
                                if std::env::var("BRIDGET_T010_DEFAULT_TERM").ok().as_deref()
                                    == Some("1")
                                {
                                    env.insert("BRIDGET_T010_DEFAULT_TERM".into(), "1".into());
                                }
                            }
                            env
                        }
                        #[cfg(not(feature = "test-support"))]
                        {
                            prepared.env.clone()
                        }
                    },
                };
                let child = spawn_managed_bootstrap_with_stderr(&launch, Stdio::from(stderr))?;
                let Some(ready) = child.wait_ready_or_cancel(
                    || stop.is_actionable() || unix_timestamp() >= prepared.lease.deadline_at,
                    MANAGED_STOP_POLL,
                )?
                else {
                    return Ok(None);
                };
                if ready.ready().instance_id != prepared.lease.instance_id
                    || ready.ready().command_id != prepared.lease.command_id
                    || ready.ready().generation != prepared.lease.generation
                {
                    return Err(crate::managed_process::ManagedProcessError::InvalidStatus(
                        "BootstrapReady non corrélé".to_string(),
                    ));
                }
                let authority_guard = prepared
                    .specialist_authority
                    .as_ref()
                    .map(crate::lifecycle::SpecialistLaunchAuthority::lock)
                    .transpose()?;
                let mut child = ready
                    .persist_marker(marker_store, &prepared.lease.name)?
                    .release()?;
                if let Some(guard) = authority_guard {
                    guard.finish()?;
                }
                child.set_status_nonblocking()?;
                Ok(Some(child))
            })();
            match start {
                Ok(Some(child)) => {
                    active.insert(
                        prepared.lease.instance_id.clone(),
                        SupervisedProcess {
                            prepared,
                            child: SupervisedChild(child),
                            registered: None,
                            connected: false,
                            failure_sent: false,
                            stop,
                            stop_attempted: false,
                        },
                    );
                }
                Ok(None) if stop.is_actionable() => {
                    let _ = events.send(ManagedSupervisorEvent::Stopped {
                        lease: prepared.lease,
                        conn_id: None,
                        outcome: StopOutcome::Stopped,
                        completion: stop,
                    });
                }
                Ok(None) => {
                    let reason = "délai absolu dépassé avant BootstrapReady".to_string();
                    let _ = fleet.fail(&prepared.lease, "spawn_timeout", &reason);
                    let _ = events.send(ManagedSupervisorEvent::Failed {
                        lease: prepared.lease,
                        kind: "spawn_timeout".to_string(),
                        reason,
                        conn_id: None,
                    });
                }
                Err(error) => {
                    if stop.is_actionable() {
                        let _ = events.send(ManagedSupervisorEvent::Stopped {
                            lease: prepared.lease,
                            conn_id: None,
                            outcome: StopOutcome::Stopped,
                            completion: stop,
                        });
                        return;
                    }
                    let reason = error.to_string();
                    let _ = fleet.fail(&prepared.lease, "negotiation_failed", &reason);
                    let _ = events.send(ManagedSupervisorEvent::Failed {
                        lease: prepared.lease,
                        kind: "negotiation_failed".to_string(),
                        reason,
                        conn_id: None,
                    });
                }
            }
        }
        ManagedSupervisorCommand::Registered {
            instance_id,
            conn_id,
            name,
        } => {
            if let Some(process) = active.get_mut(&instance_id)
                && process.prepared.lease.name == name
            {
                process.registered = Some((conn_id, name));
            }
        }
    }
}

fn poll_managed_processes(
    fleet: &FleetSupervisor,
    events: &Sender<ManagedSupervisorEvent>,
    active: &mut HashMap<String, SupervisedProcess>,
) {
    let mut finished = Vec::new();
    for (instance_id, process) in active.iter_mut() {
        if process.stop.is_actionable() && !process.stop_attempted {
            let outcome = match process.child.stop() {
                Ok(ManagedStopResult::Stopped) => StopOutcome::Stopped,
                Ok(ManagedStopResult::StoppedForced { survivors_killed }) => {
                    StopOutcome::StoppedForced { survivors_killed }
                }
                Ok(ManagedStopResult::Timeout) => StopOutcome::Timeout {
                    state: "groupe encore vivant".to_string(),
                },
                Err(error) => StopOutcome::Timeout {
                    state: error.to_string(),
                },
            };
            if matches!(outcome, StopOutcome::Timeout { .. }) {
                process.stop_attempted = true;
                let _ = events.send(ManagedSupervisorEvent::StopTimedOut {
                    lease: process.prepared.lease.clone(),
                    outcome,
                    completion: Arc::clone(&process.stop),
                });
            } else {
                let _ = events.send(ManagedSupervisorEvent::Stopped {
                    lease: process.prepared.lease.clone(),
                    conn_id: process.registered.as_ref().map(|value| value.0.clone()),
                    outcome,
                    completion: Arc::clone(&process.stop),
                });
                finished.push(instance_id.clone());
                continue;
            }
        }

        // Délai absolu avant Connected : le client a déjà (ou va) recevoir un
        // refus — le processus enfant ne doit PAS survivre (fantôme hors quota).
        if !process.connected
            && !process.failure_sent
            && unix_timestamp() >= process.prepared.lease.deadline_at
        {
            let reason = "délai absolu dépassé avant Connected".to_string();
            let _ = fleet.fail(&process.prepared.lease, "spawn_timeout", &reason);
            if reap_failed_managed_child(process, events, "spawn_timeout", reason) {
                finished.push(instance_id.clone());
            }
            continue;
        }

        let status = process.child.try_status();
        match status {
            Ok(Some(ManagedStatus::StartupFailed {
                kind,
                reason,
                instance_id: reported_instance,
                command_id,
                generation,
            })) => {
                let correlated = reported_instance == *instance_id
                    && command_id == process.prepared.lease.command_id
                    && generation == process.prepared.lease.generation;
                let (kind, reason) = if correlated {
                    (kind, reason)
                } else {
                    (
                        "negotiation_failed".to_string(),
                        "StartupFailed non corrélé".to_string(),
                    )
                };
                if !process.failure_sent {
                    let _ = fleet.fail(&process.prepared.lease, &kind, &reason);
                    if reap_failed_managed_child(process, events, &kind, reason) {
                        finished.push(instance_id.clone());
                    }
                    continue;
                }
            }
            Ok(Some(ManagedStatus::BootstrapReady(_))) => {
                if !process.failure_sent {
                    let reason = "second BootstrapReady interdit".to_string();
                    let _ = fleet.fail(&process.prepared.lease, "negotiation_failed", &reason);
                    if reap_failed_managed_child(process, events, "negotiation_failed", reason) {
                        finished.push(instance_id.clone());
                    }
                    continue;
                }
            }
            Err(error) if !process.failure_sent => {
                let reason = error.to_string();
                let _ = fleet.fail(&process.prepared.lease, "negotiation_failed", &reason);
                if reap_failed_managed_child(process, events, "negotiation_failed", reason) {
                    finished.push(instance_id.clone());
                }
                continue;
            }
            Ok(None) | Err(_) => {}
        }

        let exited = match process.child.try_wait() {
            Ok(Some(status)) => {
                let reason = format!("équipier terminé avec {status}");
                if process.connected {
                    let _ = events.send(ManagedSupervisorEvent::Exited {
                        lease: process.prepared.lease.clone(),
                        conn_id: process.registered.as_ref().map(|value| value.0.clone()),
                        reason,
                    });
                } else if !process.failure_sent {
                    let _ = fleet.fail(&process.prepared.lease, "negotiation_failed", &reason);
                    let _ = events.send(ManagedSupervisorEvent::Failed {
                        lease: process.prepared.lease.clone(),
                        kind: "negotiation_failed".to_string(),
                        reason,
                        conn_id: process.registered.as_ref().map(|value| value.0.clone()),
                    });
                    process.failure_sent = true;
                }
                let _ = process.child.remove_marker();
                finished.push(instance_id.clone());
                true
            }
            Ok(None) => false,
            Err(error) => {
                warn!("waitpid non bloquant impossible pour {instance_id}: {error}");
                false
            }
        };
        if exited {
            continue;
        }

        if !process.connected
            && !process.failure_sent
            && !process.child.status_is_open()
            && let Some((conn_id, _)) = process.registered.clone()
        {
            match fleet.register_connected(&process.prepared.lease, instance_id, unix_timestamp()) {
                Ok(_) => {
                    process.connected = true;
                    let _ = events.send(ManagedSupervisorEvent::Connected {
                        lease: process.prepared.lease.clone(),
                        conn_id,
                        definition: Box::new((*process.prepared.resolved_definition).clone()),
                    });
                }
                Err(error) => {
                    // DeadlineElapsed a déjà clos la lease (expire_locked) :
                    // ne pas rappeler fail — seulement tuer l'enfant.
                    let (kind, reason, fleet_already_closed) = match &error {
                        crate::fleet::FleetError::DeadlineElapsed => {
                            ("spawn_timeout", error.to_string(), true)
                        }
                        _ => ("negotiation_failed", error.to_string(), false),
                    };
                    if !fleet_already_closed {
                        let _ = fleet.fail(&process.prepared.lease, kind, &reason);
                    }
                    if reap_failed_managed_child(process, events, kind, reason) {
                        finished.push(instance_id.clone());
                    }
                }
            }
        }
    }
    for instance_id in finished {
        active.remove(&instance_id);
    }
}

/// Émet l'échec au client et termine le groupe (SIGTERM, jamais SIGKILL).
/// Retourne true si le groupe a disparu et peut quitter `active`.
fn reap_failed_managed_child(
    process: &mut SupervisedProcess,
    events: &Sender<ManagedSupervisorEvent>,
    kind: &str,
    reason: String,
) -> bool {
    let conn_id = process.registered.as_ref().map(|value| value.0.clone());
    let _ = events.send(ManagedSupervisorEvent::Failed {
        lease: process.prepared.lease.clone(),
        kind: kind.to_string(),
        reason,
        conn_id,
    });
    process.failure_sent = true;
    match process.child.stop() {
        Ok(ManagedStopResult::Stopped | ManagedStopResult::StoppedForced { .. }) => {
            let _ = process.child.remove_marker();
            true
        }
        Ok(ManagedStopResult::Timeout) => {
            warn!(
                "groupe encore vivant après SIGTERM suite à échec de spawn {} - pas de SIGKILL",
                process.prepared.lease.name
            );
            false
        }
        Err(error) => {
            warn!(
                "arrêt du groupe après échec de spawn {} impossible: {error}",
                process.prepared.lease.name
            );
            false
        }
    }
}

fn drain_managed_events(
    state: &Arc<Mutex<DaemonState>>,
    events: &Receiver<ManagedSupervisorEvent>,
) {
    while let Ok(event) = events.try_recv() {
        let mut controls = Vec::new();
        let mut views = Vec::new();
        let mut stop_completion = None;
        {
            let mut st = state.lock().recover_poison();
            match event {
                ManagedSupervisorEvent::Connected {
                    lease,
                    conn_id,
                    definition,
                } => {
                    let relaunch_requesters = st
                        .relaunch_requesters
                        .remove(&lease.command_id)
                        .unwrap_or_default();
                    if let Some(record) = st.managed_spawns.get_mut(&lease.command_id) {
                        record.wrapper_conn = Some(conn_id.clone());
                        let requesters = std::mem::take(&mut record.requester_conns);
                        for requester in requesters {
                            defer_control(
                                &st,
                                &requester,
                                DaemonToWrapper::SpawnAccepted {
                                    command_id: lease.command_id.clone(),
                                    agent_id: lease.name.clone(),
                                    definition: Some((*definition).clone()),
                                },
                                &mut controls,
                            );
                        }
                    }
                    for requester in relaunch_requesters {
                        defer_control(
                            &st,
                            &requester,
                            DaemonToWrapper::RelaunchResult {
                                command_id: lease.command_id.clone(),
                                outcome: RelaunchOutcome::Started {
                                    agent_id: lease.name.clone(),
                                    generation: lease.generation,
                                },
                            },
                            &mut controls,
                        );
                    }
                    // Register peut précéder l'identité Fleet. Connected retente
                    // les mêmes gardes, sans modifier restart_resumes ni ses bornes.
                    if st.conn_instances.get(&conn_id) == Some(&lease.instance_id)
                        && st.managed_by_instance.get(&lease.instance_id) == Some(&lease.command_id)
                        && !st.auxiliary_connections.contains(&conn_id)
                        && st
                            .router
                            .get_agent(&lease.name)
                            .is_some_and(|route| route.connection_id == conn_id)
                        && st
                            .presences
                            .get(&lease.instance_id)
                            .is_some_and(|presence| {
                                presence.name == lease.name
                                    && presence.state == "connected"
                                    && presence.busy_since.is_none()
                            })
                    {
                        match st.delegation_store.for_child(&lease.name) {
                            Ok(Some(task)) if task.state == "working" => {
                                schedule_execution_recovery(
                                    &mut st,
                                    &conn_id,
                                    &lease.instance_id,
                                    &lease.name,
                                    false,
                                );
                                if let Some(mut ready) =
                                    st.pending_post_response_controls.remove(&conn_id)
                                {
                                    controls.append(&mut ready);
                                }
                            }
                            Err(error) => {
                                error!("mission native après connexion illisible: {error}")
                            }
                            _ => {}
                        }
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                }
                ManagedSupervisorEvent::Failed {
                    lease,
                    kind,
                    reason,
                    conn_id,
                } => {
                    let relaunch_requesters = st
                        .relaunch_requesters
                        .remove(&lease.command_id)
                        .unwrap_or_default();
                    let refusal = match kind.as_str() {
                        "command_missing" => SpawnRefusal::CommandMissing {
                            command: reason.clone(),
                            registry: "canal managed-status".to_string(),
                        },
                        "spawn_timeout" => SpawnRefusal::SpawnTimeout,
                        _ => SpawnRefusal::NegotiationFailed {
                            detail: reason.clone(),
                        },
                    };
                    if let Some(record) = st.managed_spawns.remove(&lease.command_id) {
                        if record.stop.is_requested() {
                            stop_completion =
                                Some((Arc::clone(&record.stop), StopOutcome::Stopped));
                        }
                        st.managed_by_instance.remove(&lease.instance_id);
                        st.managed_terminal_instances
                            .insert(lease.instance_id.clone());
                        for requester in record.requester_conns {
                            defer_control(
                                &st,
                                &requester,
                                DaemonToWrapper::SpawnRejected {
                                    command_id: lease.command_id.clone(),
                                    reason: refusal.clone(),
                                },
                                &mut controls,
                            );
                        }
                        let wrapper_conn = conn_id.or(record.wrapper_conn);
                        if let Some(conn_id) = wrapper_conn {
                            let (attach_controls, attach_views) =
                                close_attach_subscriptions(&mut st, &conn_id);
                            controls.extend(attach_controls);
                            views.extend(attach_views);
                            st.router.unregister_by_conn(&conn_id);
                            st.mark_stopped(&conn_id);
                        }
                        if let Some(presence) = st.presences.get_mut(&lease.instance_id) {
                            presence.state = "stopped".to_string();
                            presence.touch_capacity();
                        }
                    }
                    let _ = st.fleet.mark_stopped(&lease.name);
                    for requester in relaunch_requesters {
                        defer_control(
                            &st,
                            &requester,
                            DaemonToWrapper::RelaunchResult {
                                command_id: lease.command_id.clone(),
                                outcome: RelaunchOutcome::Rejected {
                                    reason: refusal.clone(),
                                },
                            },
                            &mut controls,
                        );
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                }
                ManagedSupervisorEvent::Exited {
                    lease,
                    conn_id,
                    reason,
                } => {
                    let relaunch_requesters = st
                        .relaunch_requesters
                        .remove(&lease.command_id)
                        .unwrap_or_default();
                    let _ = st.fleet.mark_stopped(&lease.name);
                    let record = st.managed_spawns.remove(&lease.command_id);
                    st.managed_by_instance.remove(&lease.instance_id);
                    st.managed_terminal_instances
                        .insert(lease.instance_id.clone());
                    if let Some(record) = record.as_ref()
                        && record.stop.is_requested()
                    {
                        stop_completion = Some((Arc::clone(&record.stop), StopOutcome::Stopped));
                    }
                    let wrapper_conn = conn_id
                        .or_else(|| record.as_ref().and_then(|value| value.wrapper_conn.clone()));
                    if let Some(wrapper_conn) = wrapper_conn {
                        let (attach_controls, attach_views) =
                            close_attach_subscriptions(&mut st, &wrapper_conn);
                        controls.extend(attach_controls);
                        views.extend(attach_views);
                        let pending = std::mem::take(&mut st.pending_replies);
                        for reply in pending {
                            if reply.target_conn == wrapper_conn {
                                defer_control(
                                    &st,
                                    &reply.from_conn,
                                    DaemonToWrapper::DeliveryRejected {
                                        id: reply.msg_id,
                                        reason: reason.clone(),
                                    },
                                    &mut controls,
                                );
                            } else {
                                st.pending_replies.push(reply);
                            }
                        }
                        st.router.unregister_by_conn(&wrapper_conn);
                        st.mark_stopped(&wrapper_conn);
                    }
                    if let Some(presence) = st.presences.get_mut(&lease.instance_id) {
                        presence.state = "stopped".to_string();
                        presence.touch_capacity();
                    }
                    for requester in relaunch_requesters {
                        defer_control(
                            &st,
                            &requester,
                            DaemonToWrapper::RelaunchResult {
                                command_id: lease.command_id.clone(),
                                outcome: RelaunchOutcome::Rejected {
                                    reason: SpawnRefusal::NegotiationFailed {
                                        detail: reason.clone(),
                                    },
                                },
                            },
                            &mut controls,
                        );
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                }
                ManagedSupervisorEvent::Stopped {
                    lease,
                    conn_id,
                    outcome,
                    completion,
                } => {
                    let relaunch_requesters = st
                        .relaunch_requesters
                        .remove(&lease.command_id)
                        .unwrap_or_default();
                    let record = st.managed_spawns.remove(&lease.command_id);
                    st.managed_by_instance.remove(&lease.instance_id);
                    st.managed_terminal_instances
                        .insert(lease.instance_id.clone());
                    let wrapper_conn = conn_id
                        .or_else(|| record.as_ref().and_then(|value| value.wrapper_conn.clone()));
                    if let Some(record) = record {
                        for requester in record.requester_conns {
                            defer_control(
                                &st,
                                &requester,
                                DaemonToWrapper::SpawnRejected {
                                    command_id: lease.command_id.clone(),
                                    reason: SpawnRefusal::NegotiationFailed {
                                        detail: "lancement annulé par stop".to_string(),
                                    },
                                },
                                &mut controls,
                            );
                        }
                    }
                    if let Some(wrapper_conn) = wrapper_conn {
                        let (attach_controls, attach_views) =
                            close_attach_subscriptions(&mut st, &wrapper_conn);
                        controls.extend(attach_controls);
                        views.extend(attach_views);
                        let pending = std::mem::take(&mut st.pending_replies);
                        for reply in pending {
                            if reply.target_conn == wrapper_conn {
                                defer_control(
                                    &st,
                                    &reply.from_conn,
                                    DaemonToWrapper::DeliveryRejected {
                                        id: reply.msg_id,
                                        reason: "équipier arrêté".to_string(),
                                    },
                                    &mut controls,
                                );
                            } else {
                                st.pending_replies.push(reply);
                            }
                        }
                        st.router.unregister_by_conn(&wrapper_conn);
                        st.mark_stopped(&wrapper_conn);
                    }
                    if let Some(presence) = st.presences.get_mut(&lease.instance_id) {
                        presence.state = "stopped".to_string();
                        presence.touch_capacity();
                    }
                    for requester in relaunch_requesters {
                        defer_control(
                            &st,
                            &requester,
                            DaemonToWrapper::RelaunchResult {
                                command_id: lease.command_id.clone(),
                                outcome: RelaunchOutcome::Rejected {
                                    reason: SpawnRefusal::NegotiationFailed {
                                        detail: "relance annulée par arrêt".to_string(),
                                    },
                                },
                            },
                            &mut controls,
                        );
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                    stop_completion = Some((completion, outcome));
                }
                ManagedSupervisorEvent::StopTimedOut {
                    lease,
                    outcome,
                    completion,
                } => {
                    warn!(
                        "arrêt incomplet de l'équipier géré {} génération {} : groupe toujours supervisé",
                        lease.name, lease.generation
                    );
                    let relaunch_requesters = st
                        .relaunch_requesters
                        .remove(&lease.command_id)
                        .unwrap_or_default();
                    for requester in relaunch_requesters {
                        defer_control(
                            &st,
                            &requester,
                            DaemonToWrapper::RelaunchResult {
                                command_id: lease.command_id.clone(),
                                outcome: RelaunchOutcome::Timeout {
                                    state: "groupe encore supervisé".to_string(),
                                },
                            },
                            &mut controls,
                        );
                    }
                    stop_completion = Some((completion, outcome));
                }
            }
        }
        let _ = execute_controls(controls);
        for view in views {
            view.close_and_join();
        }
        if let Some((completion, outcome)) = stop_completion {
            completion.complete(outcome);
        }
    }
}

fn finish_recovery_command(state: &mut DaemonState, command_id: &str) {
    state.recovery_commands.remove(command_id);
    if state.recovery_commands.is_empty() {
        state.recovering = false;
    }
}

impl DaemonState {
    fn new(
        config: &DaemonConfig,
        managed_tx: Sender<ManagedSupervisorCommand>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
        let mut store = Store::open(&config.db_path)?;
        let artifact_root = config
            .canonical_artifact_root()
            .map_err(std::io::Error::other)?;
        store.recover_guichet_claims_after_restart()?;
        let idempotency = IdempotencyStore::open(&config.db_path)?;
        let desired = DesiredStateStore::at_path(desired_state_path(config));
        let fleet = Arc::new(FleetSupervisor::open(
            &config.db_path,
            desired,
            FleetConfig::from_env(),
        )?);
        let execution_store = ExecutionStore::open(&config.db_path)?;
        let delegation_store = crate::delegation::DelegationStore::open(&config.db_path)?;
        let registry = AgentRegistry::load()?;
        if let Some(diagnostic) = crate::decision_service::startup_diagnostic() {
            warn!("{diagnostic}");
        }
        let (view_closed_tx, view_closed_rx) = mpsc::channel();
        let (observation_tx, observation_lost) = observation_output();
        let (observation_result_tx, observation_result_rx) = mpsc::sync_channel(64);
        store.maintain_observation_missed(unix_now_secs(), true)?;
        let observations = crate::observation::Observations::restore(
            &store.observation_snapshot()?,
            Instant::now(),
            unix_now_secs().max(0) as u64,
        )?;
        let held_ids = {
            let mut rows = store.connection().prepare(
                "SELECT delivery_id, message_bytes FROM send_deliveries WHERE phase = 'held'",
            )?;
            let held = rows.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<Vec<u8>>>(1)?))
            })?;
            let mut ids = HashSet::new();
            for row in held {
                let (delivery_id, bytes) = row?;
                match bytes.as_deref().and_then(|bytes| {
                    serde_json::from_slice::<bridget_core::BridgetMessage>(bytes).ok()
                }) {
                    Some(message) => {
                        ids.insert(message.id);
                    }
                    None => warn!("retenu illisible non indexé en mémoire : {delivery_id}"),
                }
            }
            ids
        };
        let instance_uuid = Uuid::new_v4();
        // Allocation durable monotone : un timestamp d'admission ou un UUID
        // aléatoire ne permettrait pas le prédicat SQL « époque antérieure ».
        let coordinator_epoch = delegation_store.next_auto_coordinator_epoch()?;
        let mut provider_limits = crate::provider_limits::ProviderLimits::restore_routing244(
            store.connection(),
            unix_timestamp(),
        );
        for connection in registry
            .routing_policy()
            .into_iter()
            .flat_map(|policy| &policy.connections)
        {
            if let (Some(old), Some(new)) = (
                registry.routing_account_scope_legacy245(&connection.connection_id),
                registry.routing_account_scope244(&connection.connection_id),
            ) && provider_limits
                .migrate_account_scope245(store.connection(), &old, &new)
                .is_err()
            {
                warn!(
                    "reclassement du cache de quota indisponible ; observations originales conservées"
                );
            }
        }
        Ok(DaemonState {
            update_gate: crate::updates::Gate::open(
                config.db_path.parent().ok_or("update_root_missing")?,
            )?,
            provider_limits,
            host_resources: Default::default(),
            held_ids,
            human_thread_watches: HashMap::new(),
            lineage_watches: HashMap::new(),
            lineage_journals: HashMap::new(),
            t3_thread_bindings: HashMap::new(),
            native_permission_facts: HashMap::new(),
            native_permission_refusals: HashMap::new(),
            workspace_work: Arc::new(Mutex::new(native_delegation::WorkspaceWork::default())),
            t3_project_connections: HashSet::new(),
            observations,
            observation_sequences: HashMap::new(),
            thread_notice_versions: HashMap::new(),
            communication_projects: HashMap::new(),
            ledger_read_permits: crate::ledger::search::ReadPermits::default(),
            thread_wake_departures: VecDeque::new(),
            observation_tx,
            observation_result_tx,
            observation_result_rx,
            cli_wake_cursor: None,
            observation_lost,
            #[cfg(test)]
            fixture_root: None,
            host: crate::build_info::local_host(),
            db_path: config.db_path.clone(),
            socket_path: config.socket_path.clone(),
            artifact_root,
            instance_id: instance_uuid.to_string(),
            coordinator_epoch,
            routing_admission_claims: Arc::default(),
            router: Router::new(),
            circuit_breaker: CircuitBreaker::new(
                config.circuit_breaker_window,
                config.circuit_breaker_limit,
            ),
            deduplicator: Deduplicator::new(config.dedup_window),
            envelope_guard: EnvelopeGuard::new(Duration::from_secs(config.quarantine_window)),
            store,
            execution_store,
            delegation_store,
            decision_service: crate::decision_service::DecisionServiceClient::new(),
            active_waits: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            idempotency,
            fleet,
            registry,
            source_env: source_environment(),
            recovering: false,
            recovery_commands: HashSet::new(),
            managed_tx,
            marker_store: ManagedMarkerStore::at_directory(
                config
                    .db_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("/tmp"))
                    .join("managed"),
            ),
            managed_spawns: HashMap::new(),
            relaunch_requesters: HashMap::new(),
            decommissioning_names: HashSet::new(),
            managed_by_instance: HashMap::new(),
            managed_terminal_instances: HashSet::new(),
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_hosts: HashMap::new(),
            conn_operating_systems: HashMap::new(),
            conn_instances: HashMap::new(),
            auxiliary_connections: HashSet::new(),
            identity_credentials: HashMap::new(),
            connection_roles: HashMap::new(),
            remote_workers: HashMap::new(),
            remote_native_ready: HashSet::new(),
            remote_start_reconcile_last: None,
            remote_start_reconciles: HashMap::new(),
            dispatch_diagnostics: Default::default(),
            remote_result_reservation_last: None,
            remote_result_reservation: None,
            worker_probe_connections: HashSet::new(),
            pending_worker_responses: HashMap::new(),
            late_worker_prepares: Default::default(),
            pending_workspace_repairs: HashMap::new(),
            pending_origin_proofs: HashMap::new(),
            user_input_dispatch_after: None,
            mobile_notifier: config
                .db_path
                .parent()
                .and_then(crate::mobile::notifications::Notifier::start),
            pending_candidate_probes: HashMap::new(),
            pending_candidate_probes_v2: HashMap::new(),
            terminal_sessions: HashSet::new(),
            client_negotiations: HashMap::new(),
            service_negotiations: HashMap::new(),
            peer_uids: HashMap::new(),
            coordination_subscriptions: HashMap::new(),
            attach_subscriptions: HashMap::new(),
            attach_views: HashMap::new(),
            view_closed_tx,
            view_closed_rx,
            pending_attach_sends: HashMap::new(),
            pending_runtime_selections: HashMap::new(),
            pending_runtime_catalogues: HashMap::new(),
            runtime_catalogue_connections: HashMap::new(),
            presences: HashMap::new(),
            conn_counter: 0,
            pending_replies: Vec::new(),
            specialist_capture_active: HashSet::new(),
            pending_post_response_controls: HashMap::new(),
        })
    }

    fn next_conn_id(&mut self) -> String {
        self.conn_counter += 1;
        format!("conn-{}", self.conn_counter)
    }

    /// Un redémarrage du daemon ne doit pas transformer une interruption de
    /// service en arrêt demandé par l’utilisateur. Les wrappers gérés sont
    /// donc laissés à systemd, puis recréés depuis le fleet persistant au boot.
    /// `Disconnect` reste réservé aux wrappers externes et aux arrêts explicites.
    fn notify_wrapper_on_daemon_shutdown(&self, conn_id: &str) -> bool {
        // 090 : la TUI appartient au terminal, pas au daemon. Un arrêt de
        // Bridget ferme le lien et déclenche sa reconnexion, pas son Codex.
        // Un Disconnect explicite (stop) conserve sa sémantique d'arrêt.
        if self.terminal_sessions.contains(conn_id) {
            return false;
        }
        !self
            .conn_instances
            .get(conn_id)
            .is_some_and(|instance_id| self.managed_by_instance.contains_key(instance_id))
    }

    fn mark_unreachable(&mut self, conn_id: &str) {
        self.revoke_identity_authorizations(conn_id);
        self.terminal_sessions.remove(conn_id);
        if self.auxiliary_connections.remove(conn_id) {
            self.conn_instances.remove(conn_id);
            return;
        }
        if let Some(instance_id) = self.conn_instances.remove(conn_id) {
            // Une ré-inscription concurrente peut déjà détenir la même
            // instance : la fermeture de l'ancienne connexion ne doit pas
            // écraser busy/connected que le nouveau Register vient d'attester.
            if self.conn_instances.iter().any(|(owner, owned)| {
                owned == &instance_id && !self.auxiliary_connections.contains(owner)
            }) {
                return;
            }
            if let Some(presence) = self.presences.get_mut(&instance_id) {
                if presence.state == "stopped" {
                    return;
                }
                presence.state = "unreachable".to_string();
                presence.touch_capacity();
            }
        }
    }

    fn mark_stopped(&mut self, conn_id: &str) {
        self.revoke_identity_authorizations(conn_id);
        if self.auxiliary_connections.remove(conn_id) {
            self.conn_instances.remove(conn_id);
            return;
        }
        if let Some(instance_id) = self.conn_instances.remove(conn_id) {
            if self.conn_instances.iter().any(|(owner, owned)| {
                owned == &instance_id && !self.auxiliary_connections.contains(owner)
            }) {
                return;
            }
            if let Some(presence) = self.presences.get_mut(&instance_id) {
                presence.state = "stopped".to_string();
                presence.touch_capacity();
            }
        }
    }

    fn revoke_identity_authorizations(&mut self, owner: &str) {
        self.native_permission_facts.remove(owner);
        self.native_permission_refusals.remove(owner);
        self.t3_thread_bindings.remove(owner);
        self.t3_project_connections.remove(owner);
        native_lineage::invalidate(self);
        for watch in self
            .human_thread_watches
            .values_mut()
            .filter(|watch| watch.owner == owner)
        {
            watch
                .queue
                .terminate(bridget_transport::protocol::HumanThreadViewError::BindingUnavailable);
            let _ = watch.notify.try_send(());
        }
        let had_credential = self.identity_credentials.remove(owner).is_some();
        self.observation_sequences.remove(owner);
        self.runtime_catalogue_connections.remove(owner);
        self.observations.remove_source(owner, Instant::now());
        if !had_credential {
            return;
        }
        let Some(instance) = self.conn_instances.get(owner).cloned() else {
            return;
        };
        let auxiliaries: Vec<_> = self
            .auxiliary_connections
            .iter()
            .filter(|conn| self.conn_instances.get(*conn) == Some(&instance))
            .cloned()
            .collect();
        for conn in auxiliaries {
            self.auxiliary_connections.remove(&conn);
            self.conn_instances.remove(&conn);
            self.conn_names.remove(&conn);
            self.thread_notice_versions.remove(&conn);
        }
    }

    /// Fantôme reclaimable : le routeur pointe une instance absente (retain) ou
    /// une présence `unreachable`/`stopped`. Un Register sans `instance_id`
    /// (MCP éphémère, wrapper terminal) n'est PAS un fantôme — il n'a simplement
    /// pas de présence riche.
    fn connection_is_phantom_holder(&self, conn_id: &str) -> bool {
        match self.conn_instances.get(conn_id) {
            Some(instance_id) => match self.presences.get(instance_id) {
                None => true,
                Some(presence) => {
                    matches!(presence.state.as_str(), "unreachable" | "stopped")
                }
            },
            None => false,
        }
    }

    /// Libère un nom tenu par un fantôme. Retourne `false` si le nom est libre,
    /// live, ou tenu par une connexion sans instance (hors cas fantôme).
    fn reclaim_phantom_name(&mut self, name: &str) -> bool {
        let Some(existing) = self.router.get_agent(name) else {
            return false;
        };
        let old_conn = existing.connection_id.clone();
        if !self.connection_is_phantom_holder(&old_conn) {
            return false;
        }
        self.router.unregister_by_conn(&old_conn);
        self.revoke_identity_authorizations(&old_conn);
        self.conn_instances.remove(&old_conn);
        self.conn_names.remove(&old_conn);
        self.conn_hosts.remove(&old_conn);
        self.communication_projects.remove(&old_conn);
        self.conn_operating_systems.remove(&old_conn);
        self.auxiliary_connections.remove(&old_conn);
        self.thread_notice_versions.remove(&old_conn);
        true
    }

    /// Le relais HTTP UI est l'unique propriétaire de l'identité humaine
    /// canonique. Lorsqu'il redémarre, l'ancienne socket peut encore être
    /// retenue par le daemon alors que son lecteur local est déjà mort. Une
    /// reprise UI explicite chasse cette seule route, lui envoie Disconnect,
    /// puis laisse le nouveau processus s'enregistrer normalement.
    fn replace_stale_ui_human_route(
        &mut self,
        incoming_conn: &str,
        agent_type: &str,
        agent_id: &str,
    ) {
        if agent_type != "ui" || agent_id != UI_HUMAN_AGENT_ID {
            return;
        }
        let Some(existing) = self.router.get_agent(agent_id).cloned() else {
            return;
        };
        if existing.connection_id == incoming_conn {
            return;
        }
        // SPEC-087 T048 : une route encore vivante (écrivain joignable et
        // présence vue dans la fenêtre de rétention) qui se fait remplacer
        // est peut-être une usurpation. On ne l'empêche pas, on la montre au
        // référent, par un canal que l'agent ne lit pas (ADR 027).
        let existing_alive = self.connections.contains_key(&existing.connection_id)
            && self
                .conn_instances
                .get(&existing.connection_id)
                .and_then(|instance| self.presences.get(instance))
                .is_some_and(|presence| presence.link_seen.elapsed() < PRESENCE_RETENTION);
        if let Some(writer) = self.connections.get(&existing.connection_id)
            && let Ok(mut writer) = writer.lock()
            && let Ok(frame) = encode(&DaemonToWrapper::Disconnect)
        {
            let _ = writeln!(writer, "{frame}");
            let _ = writer.flush();
        }
        self.router.unregister_by_conn(&existing.connection_id);
        info!(
            "présence UI humaine reprise: {} remplace {}",
            incoming_conn, existing.connection_id
        );
        if existing_alive {
            let now = unix_now_secs();
            let incoming_instance = self
                .conn_instances
                .get(incoming_conn)
                .cloned()
                .unwrap_or_else(|| "instance inconnue".to_string());
            warn!(
                "route humaine vivante remplacée: {} → {} ({incoming_instance})",
                existing.connection_id, incoming_conn
            );
            let context = serde_json::json!({
                "summary": format!(
                    "Une nouvelle connexion ({incoming_instance}) a pris l’identité humaine alors que l’interface était encore vivante."
                ),
                "replaced_connection": existing.connection_id,
                "incoming_connection": incoming_conn,
                "incoming_instance": incoming_instance,
                "at": now,
            })
            .to_string();
            let _ = daemon_deposit_human_inbox(
                self,
                &format!("human-route:{incoming_conn}"),
                bridget_transport::protocol::HumanInboxKind::HumanRouteReplaced,
                &bridget_transport::protocol::HumanInboxSubject::default(),
                &context,
                &["ack".to_string()],
                now,
            );
        }
    }

    /// Quand le retain jette une présence, retire aussi le nom du routeur :
    /// sinon `agent_infos` projetait `unix`/`connected` inventés (fantôme).
    fn release_router_for_dangling_instances(&mut self) {
        let dangling: Vec<String> = self
            .conn_instances
            .iter()
            .filter(|(_, instance_id)| !self.presences.contains_key(instance_id.as_str()))
            .map(|(conn_id, _)| conn_id.clone())
            .collect();
        for conn_id in dangling {
            self.router.unregister_by_conn(&conn_id);
            self.revoke_identity_authorizations(&conn_id);
            self.conn_instances.remove(&conn_id);
            self.conn_names.remove(&conn_id);
            self.conn_hosts.remove(&conn_id);
            self.communication_projects.remove(&conn_id);
            self.conn_operating_systems.remove(&conn_id);
            self.auxiliary_connections.remove(&conn_id);
        }
    }

    /// Pousse les signaux ORPHELIN encore non notifiés (y compris après crash).
    ///
    /// Décision (écrite — charge jury) : rejouer au **Register de l'émetteur**,
    /// pas seulement au moment de la purge. Les chemins « à aller chercher »
    /// (MCP send / ledger / CLI) sont durables ; le Deliver poussé, lui, écrit
    /// directement sur le socket — ni ledger, ni idempotence, ni accusé. Or la
    /// purge frappe quand la flotte bouge, donc souvent quand l'émetteur est
    /// hors ligne. Sans rejeu au Register, « pas en silence » ne tiendrait que
    /// dans le cas le moins probable. Symétrique de
    /// `schedule_idempotent_delivery_recovery` (côté destinataire, remises
    /// `dispatching`) : ici le destinataire du *signal* est l'émetteur.
    ///
    /// Limite déclarée : `notified_at` atteste l'**émission** (`writeln!`+`flush`
    /// Ok sur la socket), pas la **réception** par le wrapper. Si le wrapper
    /// meurt juste après, le signal est perdu et la base dit qu'il a été
    /// délivré — passage direct de rien à « notifié », sans état « en vol ».
    /// Dimensionnement accepté ; pas corrigé dans ce lot.
    fn flush_pending_orphan_emitter_notices(&mut self) {
        let pending = match self.idempotency.pending_orphan_emitter_notices() {
            Ok(pending) => pending,
            Err(error) => {
                error!("lecture des notices orphelines: {error}");
                return;
            }
        };
        let now = unix_now_secs();
        for (delivery_id, sender, body) in pending {
            if let Some(agent) = self.router.get_agent(&sender)
                && let Some(writer) = self.connections.get(&agent.connection_id)
            {
                match deliver_to_agent(writer, &sender, &body) {
                    Ok(_) => {
                        if let Err(error) = self
                            .idempotency
                            .mark_orphan_emitter_notified(&delivery_id, now)
                        {
                            error!("mark notice orphelin {delivery_id}: {error}");
                        }
                    }
                    Err(error) => warn!("signal orphelin non délivré à {sender}: {error}"),
                }
            } else {
                warn!("émetteur {sender} hors ligne pour signal orphelin {delivery_id}");
            }
        }
    }

    fn set_turn_state(&mut self, conn_id: &str, in_progress: bool) -> Result<(), String> {
        self.set_turn_state_at(conn_id, in_progress, Instant::now())
    }

    fn set_turn_state_at(
        &mut self,
        conn_id: &str,
        in_progress: bool,
        now: Instant,
    ) -> Result<(), String> {
        let instance_id = self
            .conn_instances
            .get(conn_id)
            .ok_or_else(|| "état de tour reçu d'une connexion non enregistrée".to_string())?;
        let presence = self
            .presences
            .get_mut(instance_id)
            .ok_or_else(|| "présence de l'équipier introuvable".to_string())?;
        if presence.turn_attest {
            let owner = presence.name.clone();
            if self
                .router
                .get_agent(&owner)
                .is_none_or(|route| route.connection_id != conn_id)
            {
                return Err("signal de tour d’une ancienne connexion".into());
            }
            if in_progress {
                self.idempotency
                    .clear_held_turn_gate(&owner)
                    .map_err(|e| e.to_string())?;
                if presence.busy_period_start.is_none() {
                    presence.busy_period_start = Some(now);
                }
            } else {
                presence.busy_period_start = None;
            }
            presence.last_attested_at = Some(now);
        }
        if in_progress {
            presence.state = "busy".to_string();
            presence.busy_since = Some(now);
        } else {
            presence.state = "connected".to_string();
            presence.busy_since = None;
        }
        presence.touch_capacity();
        Ok(())
    }

    /// Libère les tours `busy` plus vieux que notify_timeout + grâce.
    /// Sans TurnState false (tour non abouti), l'agent resterait non mandatable
    /// indéfiniment : le service compagnon n'envoie qu'aux connected|dnd.
    fn release_stale_busy_turns(&mut self) {
        self.release_stale_busy_turns_at(Instant::now());
    }

    fn release_stale_busy_turns_at(&mut self, now: Instant) {
        let busy_ids: Vec<(String, String)> = self
            .presences
            .iter()
            .filter(|(_, presence)| presence.state == "busy")
            .map(|(id, presence)| (id.clone(), presence.agent_type.clone()))
            .collect();
        let mut stale = Vec::new();
        for (id, agent_type) in busy_ids {
            let ttl = live_notify_timeout_secs(&self.registry, &agent_type)
                .saturating_add(TIMEOUT_GRACE_PERIOD);
            if let Some(presence) = self.presences.get(&id)
                && busy_turn_is_stale(presence, ttl, now)
            {
                stale.push(id);
            }
        }
        for id in stale {
            if let Some(presence) = self.presences.get_mut(&id) {
                presence.state = "connected".to_string();
                presence.busy_since = None;
                presence.touch_capacity();
            }
        }
    }

    fn provider_ui_projection(
        &self,
        agent_type: &str,
    ) -> Option<bridget_transport::protocol::ProviderUiProjection> {
        let observation = self
            .registry
            .get(agent_type)
            .ok()?
            .capabilities
            .observed
            .as_ref()?;
        let fallback = if observation.operations.is_empty() {
            Some("aucune opération de contrôle attestée".to_string())
        } else {
            Some("refus explicite hors matrice attestée".to_string())
        };
        Some(bridget_transport::protocol::ProviderUiProjection {
            binary_version: observation.binary_version.clone(),
            contract_version: observation.contract_version.clone(),
            operations: observation.operations.clone(),
            fallback,
        })
    }

    fn agent_infos(&mut self) -> Vec<bridget_transport::protocol::AgentInfo> {
        // Avant le retain : un busy périmé redevient connected (mandatable)
        // sans attendre un TurnState false qui peut ne jamais venir.
        self.release_stale_busy_turns();
        // Lot B (déjà sur main) : exemption `connected` levée — retain =
        // horloge lien seule via `presence_within_retention`.
        // Une présence n'est qu'une route temporaire. Sa purge ne clôt pas
        // les remises durables : la même instance attestée les récupère au
        // Register, avec le même delivery_id et la déduplication du récepteur.
        self.presences
            .retain(|_, presence| presence_within_retention(presence));
        self.release_router_for_dangling_instances();
        // Les notices historiques déjà closes restent consultables/remises.
        self.flush_pending_orphan_emitter_notices();
        // Une seule lecture du roster pour tout l'annuaire : la persistance se
        // résout ensuite par nom, sans E/S par agent.
        let persistence = self.fleet.named_persistence();
        let execution_summaries = match self.execution_store.agent_execution_summaries() {
            Ok(summaries) => summaries,
            Err(error) => {
                warn!("projection d'exécution indisponible: {error}");
                std::collections::HashMap::new()
            }
        };
        let execution_projection = |name: &str| {
            let summary = execution_summaries.get(name)?;
            let wait_state = match summary.state.as_deref() {
                Some("waiting_approval") => Some("waiting_approval".to_string()),
                Some("waiting_user_input") => Some("waiting_user_input".to_string()),
                _ => None,
            };
            Some(bridget_transport::protocol::ExecutionUiProjection {
                state: summary.state.clone(),
                wait_state,
                progress_age_secs: summary.updated_at.and_then(|updated_at| {
                    u64::try_from(unix_now_secs().saturating_sub(updated_at)).ok()
                }),
                queue_depth: summary.queue_depth,
                continuation_mode: summary.continuation_mode.clone(),
            })
        };
        let display_name = |agent_id: &str| {
            self.store
                .agent_display_name(agent_id)
                .ok()
                .flatten()
                .unwrap_or_else(|| "Agent non identifié".to_string())
        };
        let agent_link_projection =
            |instance_id: &str| match self.fleet.agent_link_summary_for_child(instance_id) {
                Ok(Some(summary)) => Some(bridget_transport::protocol::AgentLinkUiProjection {
                    link_id: summary.link.link_id,
                    parent_instance_id: summary.link.parent_instance_id,
                    parent_execution_id: summary.link.parent_execution_id,
                    objective_id: summary.link.objective_id,
                    delegation_id: summary.link.delegation_id,
                    project: summary.link.project,
                    role: summary.link.role,
                    agent_path: summary.link.agent_path,
                    state: summary.link.state.as_str().to_string(),
                    direct_descendants: summary.direct_descendants,
                    descendants: summary.descendants,
                }),
                Ok(None) => None,
                Err(error) => {
                    warn!("projection de lien agent indisponible: {error}");
                    None
                }
            };

        let mut agents: Vec<_> = self
            .router
            .list_agents()
            .iter()
            .filter_map(|agent| {
                let instance_id = self.conn_instances.get(&agent.connection_id)?;
                let presence = self.presences.get(instance_id)?;
                // Jamais inventer connected/unix : sans présence attestée, hors
                // annuaire public (connexions MCP éphémères sans instance_id).
                Some(bridget_transport::protocol::AgentInfo {
                    agent_id: agent.agent_id.clone(),
                    display_name: display_name(&agent.agent_id),
                    agent_type: agent.agent_type.to_string(),
                    connection_id: agent.connection_id.clone(),
                    host: presence.host.clone(),
                    transport: presence.transport.clone(),
                    channel: presence.channel.clone(),
                    mode: presence.mode,
                    location: presence.location.clone(),
                    execution: execution_projection(&agent.agent_id),
                    os: presence.os.clone(),
                    // Un agent qui refuse d'être dérangé est connecté mais non
                    agent_link: agent_link_projection(instance_id),
                    // joignable : du point de vue de l'appelant, la question
                    // « puis-je lui écrire » a la même forme que pour un agent
                    // injoignable, d'où un état unique plutôt qu'une colonne.
                    state: if presence.is_dnd() {
                        "dnd".to_string()
                    } else {
                        presence.state.clone()
                    },
                    // Âge de CAPACITÉ (capacity_seen), pas du lien. Honnête à lire ;
                    // aucune décision de service ou de reaper ne s'en sert — elles
                    // regardent `state`. ACP sans événement de contenu → âge
                    // figé (voir regles-chantier, limites de ce lot).
                    last_seen_secs: presence.capacity_seen.elapsed().as_secs(),
                    reconnect_count: presence.reconnect_count,
                    domain: presence.domain.clone(),
                    model: presence.model.clone(),
                    effort: presence.effort.clone(),
                    rate_limits: presence.rate_limits.values().cloned().collect(),
                    model_mismatch: bridget_transport::protocol::ModelMismatchFact::observe(
                        presence.model.as_deref(),
                        presence.served_model.as_deref(),
                    ),
                    disk_space: presence.disk_space.clone(),
                    persistent: persistence.get(&agent.agent_id).copied(),
                    provider: self.provider_ui_projection(&presence.agent_type),
                })
            })
            .collect();
        let live_names: std::collections::HashSet<String> =
            agents.iter().map(|agent| agent.agent_id.clone()).collect();
        for record in self.managed_spawns.values().filter(|record| {
            (record.lease.remote_placement.is_some()
                || self.recovery_commands.contains(&record.lease.command_id)
                || self
                    .relaunch_requesters
                    .contains_key(&record.lease.command_id))
                && !live_names.contains(&record.lease.name)
        }) {
            let managed_definition = self
                .fleet
                .resolved_definition_for_command(&record.lease.command_id);
            let (model, effort) = managed_definition
                .as_ref()
                .and_then(definition_runtime)
                .map(|(model, effort)| (Some(model), effort))
                .unwrap_or((None, None));
            // Même dérivation que handle_register : transport et mode viennent
            // de la définition figée, jamais d'un amalgame unix+acp.
            let (transport, mode) = managed_definition
                .as_ref()
                .map(definition_presence_fields)
                .unwrap_or_else(|| ("unknown".to_string(), None));
            let lifecycle_state = if record.lease.remote_placement.is_some() {
                "unreachable"
            } else if self.recovery_commands.contains(&record.lease.command_id) {
                "recovering"
            } else {
                "relaunching"
            };
            agents.push(bridget_transport::protocol::AgentInfo {
                agent_id: record.lease.name.clone(),
                display_name: display_name(&record.lease.name),
                agent_type: record.agent_type.clone(),
                connection_id: String::new(),
                host: record.lease.remote_placement.as_ref().map_or_else(
                    || "local".to_string(),
                    |placement| placement.machine_id.clone(),
                ),
                transport,
                channel: None,
                mode,
                location: None,
                execution: execution_projection(&record.lease.name),
                os: if record.lease.remote_placement.is_some() {
                    "unknown"
                } else {
                    std::env::consts::OS
                }
                .to_string(),
                agent_link: agent_link_projection(&record.lease.instance_id),
                state: lifecycle_state.to_string(),
                last_seen_secs: 0,
                reconnect_count: 0,
                domain: self.fleet.desired_domain(&record.lease.name),
                model,
                effort,
                rate_limits: Vec::new(),
                model_mismatch: None,
                disk_space: None,
                persistent: persistence.get(&record.lease.name).copied(),
                provider: self.provider_ui_projection(&record.agent_type),
            });
        }
        let listed_names: std::collections::HashSet<String> =
            agents.iter().map(|agent| agent.agent_id.clone()).collect();
        for (instance_id, presence) in self.presences.iter().filter(|(_, presence)| {
            matches!(presence.state.as_str(), "stopped" | "unreachable")
                && !listed_names.contains(&presence.name)
        }) {
            agents.push(bridget_transport::protocol::AgentInfo {
                agent_id: presence.name.clone(),
                display_name: display_name(&presence.name),
                agent_type: presence.agent_type.clone(),
                connection_id: String::new(),
                host: presence.host.clone(),
                transport: presence.transport.clone(),
                channel: presence.channel.clone(),
                mode: presence.mode,
                execution: execution_projection(&presence.name),
                agent_link: agent_link_projection(instance_id),
                location: presence.location.clone(),
                os: presence.os.clone(),
                state: presence.state.clone(),
                last_seen_secs: presence.capacity_seen.elapsed().as_secs(),
                reconnect_count: presence.reconnect_count,
                domain: presence.domain.clone(),
                // FR-010 : un agent injoignable garde sa dernière capacité connue.
                model: presence.model.clone(),
                effort: presence.effort.clone(),
                rate_limits: presence.rate_limits.values().cloned().collect(),
                model_mismatch: bridget_transport::protocol::ModelMismatchFact::observe(
                    presence.model.as_deref(),
                    presence.served_model.as_deref(),
                ),
                disk_space: presence.disk_space.clone(),
                persistent: persistence.get(&presence.name).copied(),
                provider: self.provider_ui_projection(&presence.agent_type),
            });
        }
        let listed_names: HashSet<String> =
            agents.iter().map(|agent| agent.agent_id.clone()).collect();
        if let Ok(desired) = self.fleet.desired_fleet() {
            for (name, entry) in desired.equipiers.into_iter().filter(|(name, entry)| {
                entry.lifecycle_state == DesiredLifecycleState::Stopped
                    && !listed_names.contains(name)
            }) {
                let (model, effort) = entry
                    .resolved_definition
                    .as_ref()
                    .and_then(definition_runtime)
                    .map(|(model, effort)| (Some(model), effort))
                    .unwrap_or((None, None));
                let (transport, mode) = entry
                    .resolved_definition
                    .as_ref()
                    .map(definition_presence_fields)
                    .unwrap_or_else(|| ("unknown".to_string(), None));
                agents.push(bridget_transport::protocol::AgentInfo {
                    agent_id: name.clone(),
                    display_name: display_name(&name),
                    agent_type: entry.agent_type.clone(),
                    connection_id: String::new(),
                    host: self.host.clone(),
                    transport,
                    channel: None,
                    mode,
                    location: None,
                    execution: execution_projection(&name),
                    agent_link: None,
                    os: std::env::consts::OS.to_string(),
                    state: "stopped".to_string(),
                    last_seen_secs: 0,
                    reconnect_count: 0,
                    domain: entry.domain,
                    model,
                    effort,
                    rate_limits: Vec::new(),
                    model_mismatch: None,
                    disk_space: None,
                    persistent: Some(entry.persistent),
                    provider: self.provider_ui_projection(&entry.agent_type),
                });
            }
        }
        agents.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then_with(|| left.agent_id.cmp(&right.agent_id))
        });
        agents
    }

    fn restore_pending_for_agent(&mut self, name: &str, conn_id: &str) {
        // Une reconnexion conserve le cycle suivi déjà en mémoire, mais son
        // routage doit suivre la nouvelle connexion. Sans ce rattachement,
        // l'état `busy` est restauré alors que les relances continuent de
        // viser l'ancien identifiant et ne sont jamais différées.
        let frozen: HashMap<_, _> = self
            .pending_replies
            .iter()
            .map(|pending| {
                (
                    pending.msg_id.clone(),
                    native_delegation::request_frozen_secs(self, &pending.msg_id, unix_timestamp()),
                )
            })
            .collect();
        for pending in &mut self.pending_replies {
            if let Some(Ok(Some(secs))) = frozen.get(&pending.msg_id) {
                pending.frozen_secs = *secs;
            }
            if pending.to == name {
                pending.target_conn = conn_id.to_string();
            }
            if pending.from == name {
                pending.from_conn = conn_id.to_string();
            }
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let Ok(requests) = self.store.open_requests() else {
            return;
        };
        for request in requests {
            if request.sender != name && request.target != name
                || self
                    .pending_replies
                    .iter()
                    .any(|pending| pending.msg_id == request.id)
            {
                continue;
            }
            let Some(target_conn) = self
                .router
                .get_agent(&request.target)
                .map(|agent| agent.connection_id.clone())
            else {
                continue;
            };
            let Some(from_conn) = self
                .router
                .get_agent(&request.sender)
                .map(|agent| agent.connection_id.clone())
            else {
                continue;
            };
            let timeout_secs = (request.deadline_at - request.created_at).max(1) as u64;
            let elapsed = (now - request.created_at).max(0) as u64;
            let created_at = std::time::Instant::now()
                .checked_sub(Duration::from_secs(elapsed))
                .unwrap_or_else(std::time::Instant::now);
            let frozen_secs = match native_delegation::request_frozen_secs(self, &request.id, now) {
                Ok(secs) => secs.unwrap_or(0),
                Err(error) => {
                    warn!("gel de suivi illisible à la reconnexion: {error}");
                    continue;
                }
            };
            self.pending_replies.push(PendingReply {
                msg_id: request.id,
                from: request.sender,
                from_conn,
                to: request.target,
                target_conn,
                timeout_secs,
                created_at,
                frozen_secs,
                escalation_level: request.escalation_level,
                deferred_level: None,
            });
        }
    }
}

/// Réserve toutes les reprises en ordre lexical avant de libérer la boucle
/// normale. Les `PreparedSpawn` ne sont envoyés au superviseur qu'après la fin
/// de cette fonction, ce qui rend tous les noms visibles à `stop` même si le
/// premier bootstrap bloque.
fn reserve_managed_recoveries(
    state: &mut DaemonState,
    now: i64,
) -> Result<Vec<ManagedRecovery>, Box<dyn std::error::Error>> {
    state.recovering = true;
    let native_children = native_delegation::prepare_restart(state)?;
    let mut absents = Vec::new();
    let roster_persistents = state.fleet.persistent_named();
    let candidates = state.fleet.recovery_candidates();
    let in_flight_names = candidates
        .iter()
        .map(|candidate| candidate.lease.name.clone())
        .collect::<HashSet<_>>();
    let desired = state.fleet.desired_fleet()?;
    let mut prepared = Vec::new();

    for (name, _) in state.fleet.drain_legacy_non_persistent_named()? {
        absents.push(RecoveryLossEntry {
            name,
            reason: REASON_NON_PERSISTENT.to_string(),
            detail: Some(
                "ancien agent nommé non persistant absent du registre durable".to_string(),
            ),
        });
    }

    for candidate in candidates {
        if candidate.lease.remote_placement.is_some() {
            state.managed_by_instance.insert(
                candidate.lease.instance_id.clone(),
                candidate.lease.command_id.clone(),
            );
            state.managed_spawns.insert(
                candidate.lease.command_id.clone(),
                ManagedSpawnRecord {
                    lease: candidate.lease,
                    agent_type: candidate.agent_type,
                    requester_conns: Vec::new(),
                    wrapper_conn: None,
                    stop: Arc::new(ManagedStopControl::new()),
                },
            );
            continue;
        }
        if native_children.contains(&candidate.lease.name) {
            continue;
        }
        if candidate.lease.deadline_at <= now {
            if let Some(mut task) = state.delegation_store.for_child(&candidate.lease.name)? {
                native_delegation::fail_restart(state, &mut task, "spawn_timeout")?;
            }
            let _ =
                state
                    .fleet
                    .expire(&candidate.lease.command_id, candidate.lease.generation, now);
            absents.push(RecoveryLossEntry {
                name: candidate.lease.name.clone(),
                reason: REASON_RECOVERY_FAILED.to_string(),
                detail: Some("échéance de spawn dépassée avant reprise".to_string()),
            });
            continue;
        }
        let lease = candidate.lease.clone();
        match prepare_recovery(&state.source_env, candidate) {
            Ok(recovery) => prepared.push(recovery),
            Err(reason) => {
                let detail = serde_json::to_string(&reason)
                    .unwrap_or_else(|_| "échec de préparation de reprise".to_string());
                absents.push(RecoveryLossEntry {
                    name: lease.name.clone(),
                    reason: REASON_RECOVERY_FAILED.to_string(),
                    detail: Some(detail.clone()),
                });
                let _ = state.fleet.fail(&lease, "recovery_failed", detail);
                if let Some(mut task) = state.delegation_store.for_child(&lease.name)? {
                    native_delegation::fail_restart(state, &mut task, "unreachable")?;
                }
            }
        }
    }

    for (name, equipier) in desired.equipiers {
        if native_children.contains(&name) {
            continue;
        }
        if in_flight_names.contains(&name) {
            continue;
        }
        if equipier.remote_placement.is_some() {
            // Sans reçu SQL réhydratable, aucun nouveau lancement ni PID local déduit.
            continue;
        }
        if equipier.lifecycle_state != DesiredLifecycleState::Running || !equipier.persistent {
            continue;
        }
        if equipier.project.is_some() || equipier.runtime_execution.is_some() {
            // Refus fondé sur l'exigence HISTORIQUE, jamais sur la présence
            // d'un binding. Un conteneur absent ne devient pas un processus hôte.
            let project_id = equipier
                .project
                .as_ref()
                .map(|p| p.project_id.clone())
                .or_else(|| {
                    equipier
                        .runtime_execution
                        .as_ref()
                        .map(|e| e.project_id.clone())
                })
                .unwrap_or_default();
            absents.push(RecoveryLossEntry {
                name: name.clone(),
                reason: REASON_RECOVERY_FAILED.into(),
                detail: Some(serde_json::to_string(
                    &SpawnRefusal::DockerRuntimeUnavailable { project_id },
                )?),
            });
            state.fleet.mark_stopped(&name)?;
            continue;
        }
        let ownership = if state.delegation_store.for_child(&name)?.is_some() {
            crate::fleet::ownership_from_desired(&equipier)
        } else {
            None
        };
        let Some(resolved_definition) = equipier.resolved_definition else {
            warn!("reprise de {name} refusée: définition figée absente");
            absents.push(RecoveryLossEntry {
                name: name.clone(),
                reason: REASON_FROZEN_DEFINITION.to_string(),
                detail: Some("définition figée absente".to_string()),
            });
            state.fleet.mark_stopped(&name)?;
            continue;
        };
        let order = FleetSpawnOrder {
            remote_placement: equipier.remote_placement.clone(),
            posture: None,
            agent_type: equipier.agent_type,
            requested_name: Some(name.clone()),
            cwd: equipier.cwd,
            persistent: equipier.persistent,
            command_id: format!("recovery-{}", Uuid::new_v4()),
            issued_at: now,
            deadline_at: now.saturating_add(MANAGED_RECOVERY_DEADLINE_SECS),
            ownership,
            project: equipier.project,
        };
        match submit_spawn_from_resolved(
            &state.fleet,
            &state.source_env,
            &order,
            now,
            &resolved_definition,
            // Reprise : le daemon relance ses propres agents chez lui, donc
            // demandeur et exécutant sont la même machine.
            &crate::lifecycle::SpawnHosts::local(),
        )? {
            SpawnDecision::Ready(recovery) => prepared.push(recovery),
            SpawnDecision::Rejected(reason) => {
                if let SpawnRefusal::QuotaExceeded { limit } = &reason {
                    // WARN visible (agent + quota + levier). Trace durable = D20.
                    warn!(
                        "{}",
                        crate::fleet::resume_quota_refusal_message(&name, *limit)
                    );
                    absents.push(RecoveryLossEntry {
                        name: name.clone(),
                        reason: REASON_QUOTA.to_string(),
                        detail: Some(crate::fleet::quota_exceeded_detail(*limit)),
                    });
                } else {
                    warn!("reprise de {name} refusée: {reason:?}");
                    absents.push(RecoveryLossEntry {
                        name: name.clone(),
                        reason: REASON_RECOVERY_FAILED.to_string(),
                        detail: Some(format!("{reason:?}")),
                    });
                }
                state.fleet.mark_stopped(&name)?;
            }
            SpawnDecision::EnvelopeMismatch => {
                warn!("reprise de {name} refusée: enveloppe divergente");
                absents.push(RecoveryLossEntry {
                    name: name.clone(),
                    reason: REASON_RECOVERY_FAILED.to_string(),
                    detail: Some("enveloppe divergente".to_string()),
                });
                state.fleet.mark_stopped(&name)?;
            }
            SpawnDecision::Await(_) | SpawnDecision::Accepted { .. } => {
                warn!("reprise de {name} rattachée à un état inattendu");
                absents.push(RecoveryLossEntry {
                    name: name.clone(),
                    reason: REASON_RECOVERY_FAILED.to_string(),
                    detail: Some("état de reprise inattendu".to_string()),
                });
                state.fleet.mark_stopped(&name)?;
            }
        }
    }

    let remaining = state.fleet.desired_fleet()?;
    let already: HashSet<String> = absents.iter().map(|entry| entry.name.clone()).collect();
    for (name, _) in roster_persistents {
        if remaining.equipiers.contains_key(&name) || already.contains(&name) {
            continue;
        }
        absents.push(RecoveryLossEntry {
            name: name.clone(),
            reason: REASON_ABSENT_FROM_FLEET.to_string(),
            detail: Some("présent au roster persistant, absent de fleet.json".to_string()),
        });
        state.fleet.forget_named(&name);
    }

    if let Err(error) = state.fleet.persist_recovery_losses(now, absents) {
        warn!("trace de reprise non écrite: {error}");
    }

    let mut verified = Vec::with_capacity(prepared.len());
    for mut recovery in prepared {
        if let Some(mut task) = state.delegation_store.for_child(&recovery.lease.name)? {
            // Refaire les contrôles juste avant le lancement. L'environnement
            // natif porte le bail de ce chemin, jamais celui de la génération perdue.
            // Cette reprise conserve la définition/option déjà admise.
            // Un cache volatil vide après redémarrage ne doit pas transformer
            // cette mission en échec. Les gardes de définition, droits et
            // identité restent appliquées ; l'admission neuve vérifie le nonce.
            let authorized = native_delegation::native_launch_registry(state, &task)
                .and_then(|_| native_delegation::native_child_env(&task, &mut recovery));
            if let Err(reason) = authorized {
                native_delegation::fail_restart(state, &mut task, &reason)?;
                let _ = state
                    .fleet
                    .fail(&recovery.lease, "recovery_failed", &reason);
                continue;
            }
        }
        verified.push(recovery);
    }
    let mut prepared = verified;
    prepared.sort_by(|left, right| left.lease.name.cmp(&right.lease.name));
    for (recovery, stop) in prepared
        .iter()
        .map(|recovery| (recovery, Arc::new(ManagedStopControl::new())))
    {
        state.managed_by_instance.insert(
            recovery.lease.instance_id.clone(),
            recovery.lease.command_id.clone(),
        );
        state
            .recovery_commands
            .insert(recovery.lease.command_id.clone());
        state.managed_spawns.insert(
            recovery.lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: recovery.lease.clone(),
                agent_type: recovery.agent_type.clone(),
                requester_conns: Vec::new(),
                wrapper_conn: None,
                stop,
            },
        );
    }
    state.recovering = !state.recovery_commands.is_empty();

    Ok(prepared
        .into_iter()
        .map(|recovery| {
            let stop = state
                .managed_spawns
                .get(&recovery.lease.command_id)
                .expect("reprise insérée sous le même verrou")
                .stop
                .clone();
            (recovery, stop)
        })
        .collect())
}

fn defer_idempotent_delivery(
    state: &DaemonState,
    target_conn: &str,
    delivery: SendDelivery,
    controls: &mut Vec<DeferredControl>,
    trusted_producer: Option<bridget_core::MessageOrigin>,
    origin_conn: Option<&str>,
) -> Result<(), String> {
    let mut message: bridget_core::BridgetMessage = serde_json::from_slice(&delivery.message_bytes)
        .map_err(|error| format!("enveloppe de remise idempotente corrompue: {error}"))?;
    if message.intent == Some(bridget_core::MessageIntent::StoreOnly) {
        return Err(
            "information_not_deliverable : dépôt silencieux, aucune remise fournisseur".into(),
        );
    }
    // O(log E): le reçu de promotion est l’autorité durable, y compris après une panne.
    // Les octets et le canon de création restent intacts (contrat159).
    let promoted:bool=state.store.connection().query_row(
        "SELECT EXISTS(SELECT 1 FROM queue_edits WHERE delivery_id=?1 AND action='promote' AND state='applied')",
        [&delivery.delivery_id],|r|r.get(0),
    ).map_err(|e|format!("reçu de promotion illisible: {e}"))?;
    if promoted {
        message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
        if !message.references.iter().any(|r| r == "steer-or-queue") {
            message.references.push("steer-or-queue".into());
        }
    }
    // Le journal et l'idempotence conservent l'Agent ID. Le seul nom destiné
    // au fournisseur est une projection de présentation ajoutée à la remise.
    message.from_display_name = provider_display_name(state, &message.from);
    message.thread_display_title = provider_thread_display_title(state, &message);
    // Même autorité que Deliver classique : relire à la poussée. Les octets
    // persistés (souvent sans deadline pour reply=false) ne doivent pas
    // condamner le tour au notify figé du fleet (600 s mesuré sur relec6).
    let agent_type = state
        .conn_instances
        .get(target_conn)
        .and_then(|instance_id| state.presences.get(instance_id))
        .map(|presence| presence.agent_type.as_str())
        .unwrap_or("");
    native_delegation::stamp_mission_deadline_for_delivery(state, &mut message)?;
    stamp_turn_deadline_for_delivery(&mut message, &state.registry, agent_type);
    let execution = match state
        .idempotency
        .delivery_execution_link(&delivery.delivery_id)
    {
        Ok(None) => None,
        Ok(Some(link)) => {
            if link.submission_id != message.id {
                return Err("lien causal divergent du message remis".to_string());
            }
            let snapshot = state
                .execution_store
                .execution_snapshot(&link.execution_id)
                .map_err(|error| format!("exécution liée illisible: {error}"))?
                .ok_or_else(|| "exécution liée absente".to_string())?;
            Some(ExecutionDeliveryContext {
                execution_id: snapshot.execution_id,
                generation: snapshot.generation,
                revision: snapshot.revision,
            })
        }
        Err(error) => {
            return Err(format!("lien causal de remise illisible: {error}"));
        }
    };
    let mid_turn_ok = Some(guidance_mid_turn_ok(
        state,
        &message,
        &message.to,
        trusted_producer,
        origin_conn,
    ));
    defer_control(
        state,
        target_conn,
        DaemonToWrapper::DeliverIdempotent {
            delivery_id: delivery.delivery_id,
            recipient_instance_id: delivery.recipient_instance_id,
            delivery_generation: delivery.delivery_generation,
            expires_at: delivery.expires_at,
            message,
            execution,
            mid_turn_ok,
        },
        controls,
    );
    Ok(())
}

/// FR004 : cinq déclencheurs partagent State, route attestée et transaction.
/// O(log N+G+B), G réservations, B enveloppe. Aucun verrou writer ici.
fn schedule_held_release(st: &mut DaemonState, agent: &str, now: Instant) -> bool {
    if crate::agent_tracking::dormant(st.store.connection(), agent).unwrap_or(true) {
        return false;
    }
    let Some(route) = st.router.get_agent(agent) else {
        return false;
    };
    let conn = route.connection_id.clone();
    let Some((owner, instance)) = live_connection_identity(st, &conn) else {
        return false;
    };
    if owner != agent || !st.connections.contains_key(&conn) {
        return false;
    }
    let Some(p) = st.presences.get(&instance) else {
        return false;
    };
    let mobile_held = st.store.connection().query_row(
        "SELECT EXISTS(SELECT 1 FROM send_deliveries WHERE recipient_agent=?1 AND phase='held' AND issuer_scope GLOB 'mobile177_*')",
        [agent], |row| row.get::<_,bool>(0),
    ).unwrap_or(false);
    if !p.turn_attest || (p.transport == "t3code" && !mobile_held) || p.state != "connected" {
        return false;
    }
    let fresh = p.busy_period_start.is_none()
        && p.last_attested_at
            .is_some_and(|at| now.saturating_duration_since(at) <= Duration::from_secs(90));
    let forced = p
        .last_attested_at
        .is_some_and(|at| now.saturating_duration_since(at) > Duration::from_secs(90))
        && p.busy_period_start.is_some_and(|at| {
            now.saturating_duration_since(at)
                >= Duration::from_secs(
                    live_notify_timeout_secs(&st.registry, &p.agent_type)
                        .saturating_add(TIMEOUT_GRACE_PERIOD),
                )
        });
    if !fresh && !forced {
        return false;
    }
    let (delivery, settled) = match st.idempotency.release_next_held(agent, unix_now_secs()) {
        Ok(result) => result,
        Err(error) => {
            warn!("libération retenue différée pour {agent}: {error}");
            return false;
        }
    };
    if let Some(notice) = settled {
        st.held_ids.remove(&notice.message_id);
        st.pending_replies
            .retain(|pending| pending.msg_id != notice.message_id);
        // La notice durable est reprise par le mécanisme existant, sans writer ici.
    }
    let Some(delivery) = delivery else {
        return false;
    };
    let message: bridget_core::BridgetMessage =
        match serde_json::from_slice(&delivery.message_bytes) {
            Ok(message) => message,
            Err(error) => {
                error!("enveloppe libérée illisible: {error}");
                return false;
            }
        };
    st.held_ids.remove(&message.id);
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("after_held_release_commit");
    let mut controls = Vec::new();
    if let Err(error) =
        defer_idempotent_delivery(st, &conn, delivery.clone(), &mut controls, None, None)
    {
        error!("projection de remise retenue impossible: {error}");
        if let Err(error) = st.idempotency.mark_delivery_indeterminate(
            &delivery.delivery_id,
            &delivery.recipient_instance_id,
            delivery.delivery_generation,
        ) {
            warn!("quarantaine de libération non confirmée: {error}");
        }
        return false;
    }
    st.pending_post_response_controls
        .entry(conn)
        .or_default()
        .extend(controls);
    true
}

/// La reprise n'est déclenchée qu'au réenregistrement de l'instance ciblée :
/// au démarrage, aucun socket wrapper n'existe encore. Les bytes persistés et
/// le couple instance/génération sont les seules autorités ; aucun nom n'est
/// résolu une seconde fois.
fn schedule_idempotent_delivery_recovery(
    state: &mut DaemonState,
    conn_id: &str,
    instance_id: &str,
) {
    let deliveries = match state
        .idempotency
        .dispatching_deliveries_for_instance(instance_id, unix_now_secs())
    {
        Ok(deliveries) => deliveries,
        Err(error) => {
            error!("reprise des remises idempotentes: {error}");
            return;
        }
    };
    let mut controls = Vec::new();
    for delivery in deliveries {
        if !matches!(
            state.idempotency.is_wake_delivery(&delivery.delivery_id),
            Ok(false)
        ) {
            continue;
        }
        // Une remise native incertaine n'est jamais une autorisation de
        // réexécuter sa mission, même sur la même instance reconnectée.
        let native_mission =
            serde_json::from_slice::<bridget_core::BridgetMessage>(&delivery.message_bytes)
                .map_err(|error| error.to_string())
                .and_then(|message| {
                    state
                        .delegation_store
                        .for_mission(&message.id)
                        .map(|task| task.is_some())
                });
        if !matches!(native_mission, Ok(false)) {
            let _ = state.idempotency.mark_delivery_indeterminate(
                &delivery.delivery_id,
                &delivery.recipient_instance_id,
                delivery.delivery_generation,
            );
            continue;
        }
        if let Err(error) =
            defer_idempotent_delivery(state, conn_id, delivery.clone(), &mut controls, None, None)
        {
            error!("reprise idempotente mise en quarantaine: {error}");
            let _ = state.idempotency.mark_delivery_indeterminate(
                &delivery.delivery_id,
                &delivery.recipient_instance_id,
                delivery.delivery_generation,
            );
        }
    }
    if !controls.is_empty() {
        state
            .pending_post_response_controls
            .entry(conn_id.to_string())
            .or_default()
            .extend(controls);
    }
    if let Some((agent, _)) = live_connection_identity(state, conn_id) {
        schedule_held_release(state, &agent, Instant::now());
    }
}

fn schedule_execution_recovery(
    state: &mut DaemonState,
    conn_id: &str,
    instance_id: &str,
    agent_name: &str,
    turn_in_progress: bool,
) {
    if turn_in_progress {
        return;
    }
    let now = unix_now_secs();
    let candidates = match state
        .execution_store
        .recoverable_execution_ids_for_agent(agent_name)
    {
        Ok(candidates) => candidates,
        Err(error) => {
            error!("sélection des exécutions à reprendre pour {agent_name}: {error}");
            return;
        }
    };
    let Some(parent_execution_id) = candidates.first() else {
        return;
    };
    let native_resume = match state.delegation_store.for_child(agent_name) {
        Ok(Some(task)) => {
            match native_delegation::may_resume(state, &task, parent_execution_id, instance_id) {
                Ok(true) => true,
                Ok(false) => return,
                Err(error) => {
                    error!("garde de reprise native refusée: {error}");
                    return;
                }
            }
        }
        Err(error) => {
            error!("identité de mission native avant reprise illisible: {error}");
            return;
        }
        _ => false,
    };
    if candidates.len() > 1 {
        warn!(
            "reprise refusée pour {agent_name}: exécutions actives concurrentes {:?}",
            candidates
        );
        return;
    }
    match state.idempotency.dispatching_delivery_for_execution(
        parent_execution_id,
        instance_id,
        now,
    ) {
        Ok(Some(_)) => return,
        Ok(None) => {}
        Err(error) => {
            error!("vérification de remise active pour {parent_execution_id}: {error}");
            return;
        }
    }
    let stranded_delivery = match state
        .idempotency
        .dispatching_delivery_for_execution_any_instance(parent_execution_id, now)
    {
        Ok(delivery) => delivery,
        Err(error) => {
            error!("recherche de remise liée hors présence pour {parent_execution_id}: {error}");
            return;
        }
    };
    if let Some(delivery) = stranded_delivery {
        match state.idempotency.reassign_dispatching_deliveries(
            &delivery.recipient_instance_id,
            instance_id,
            now,
        ) {
            Ok(migrated) => {
                schedule_idempotent_delivery_recovery(state, conn_id, instance_id);
                if !native_resume || migrated == 0 {
                    return;
                }
            }
            Err(error) => {
                error!(
                    "réaffectation de la remise {} vers {}: {error}",
                    delivery.delivery_id, instance_id
                );
                return;
            }
        }
        // La remise native incertaine reste absorbante. La garde ci-dessus
        // autorise une continuation, jamais le rejeu brut de cette remise.
        // Sous ce verrou, seule sa quarantaine peut retirer le dispatching
        // non expiré observé avant la réaffectation.
        match state
            .idempotency
            .dispatching_delivery_for_execution_any_instance(parent_execution_id, now)
        {
            Ok(None) => {}
            Ok(Some(_)) => return,
            Err(error) => {
                error!("quarantaine native non confirmée pour {parent_execution_id}: {error}");
                return;
            }
        }
    }

    let parent = match state
        .execution_store
        .execution_snapshot(parent_execution_id)
    {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => return,
        Err(error) => {
            error!("lecture de l'exécution à reprendre {parent_execution_id}: {error}");
            return;
        }
    };
    let control = match crate::referent_control::read(state.store.connection()) {
        Ok(control) => control,
        Err(error) => {
            error!("lecture du contrôle avant reprise de {parent_execution_id}: {error}");
            return;
        }
    };
    let continuation_id = format!("continuation-recovery-{parent_execution_id}");
    let specialist_budget =
        match specialist_requests::budget_for_execution(state, parent_execution_id) {
            Ok(policy) => policy,
            Err(code) => {
                warn!("continuation conservée sans départ : {code}");
                return;
            }
        };
    match reserve_governed_continuation(
        &state.execution_store,
        specialist_budget,
        autonomy_runtime_for_control(&control, crate::fleet::AutonomyRuntimeState::Ready),
        GovernedContinuationSource::RecoveryAfterIdleWrapper,
        parent_execution_id,
        parent.generation,
        parent.revision,
        &continuation_id,
        now,
        now,
    ) {
        Ok(
            GovernedContinuation::Reserved
            | GovernedContinuation::Reservation(
                crate::execution_store::ContinuationReservation::Replayed,
            ),
        ) => {}
        Ok(GovernedContinuation::Budget(outcome)) => {
            warn!("reprise gouvernée différée pour {parent_execution_id}: {outcome:?}");
            return;
        }
        Ok(GovernedContinuation::Reservation(reservation)) => {
            warn!("reprise gouvernée refusée pour {parent_execution_id}: {reservation:?}");
            return;
        }
        Ok(GovernedContinuation::MissingFacts) => {
            warn!("reprise gouvernée sans faits pour {parent_execution_id}");
            return;
        }
        Err(error) => {
            error!("réservation gouvernée de {parent_execution_id}: {error}");
            return;
        }
    }

    let child_execution_id = format!("execution-recovery-{}", Uuid::new_v4());
    let reconstructed = match state.execution_store.reconstruct_active_for_agent(
        agent_name,
        instance_id,
        &child_execution_id,
        now,
    ) {
        Ok(ExecutionRecoveryOutcome::Reconstructed(reconstructed)) => reconstructed,
        Ok(ExecutionRecoveryOutcome::None) => return,
        Ok(ExecutionRecoveryOutcome::PayloadUnavailable {
            parent_execution_id,
        }) => {
            warn!(
                "reprise fermée pour {agent_name}: enveloppe absente ou corrompue sur {parent_execution_id}"
            );
            return;
        }
        Ok(ExecutionRecoveryOutcome::Ambiguous { execution_ids }) => {
            warn!(
                "reprise refusée pour {agent_name}: exécutions actives concurrentes {:?}",
                execution_ids
            );
            return;
        }
        Err(error) => {
            error!("reconstruction d'exécution pour {agent_name}: {error}");
            return;
        }
    };

    if state
        .delegation_store
        .for_child(agent_name)
        .is_ok_and(|task| task.is_some_and(|task| task.remote_placement.is_some()))
    {
        // La prochaine tick réserve le slot distant hors verrou, puis remet
        // cette même reconstruction durable. Aucun second UUID ni fournisseur.
        return;
    }
    if let Err(error) =
        deliver_reconstructed(state, conn_id, instance_id, &reconstructed, now, false)
    {
        error!("remise de reprise non préparée: {error}");
    }
}

// Erreur interne de l’extraction : le tick ne confond pas SQL et refus de remise.
#[derive(Debug)]
pub(super) enum ReconstructedDeliveryError {
    Storage(String),
    NonReprenable(String),
}

impl std::fmt::Display for ReconstructedDeliveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(reason) | Self::NonReprenable(reason) => formatter.write_str(reason),
        }
    }
}

impl From<crate::idempotency::IdempotencyError> for ReconstructedDeliveryError {
    fn from(error: crate::idempotency::IdempotencyError) -> Self {
        if matches!(
            &error,
            crate::idempotency::IdempotencyError::Sqlite(_)
                | crate::idempotency::IdempotencyError::Schema(_)
        ) {
            Self::Storage(error.to_string())
        } else {
            Self::NonReprenable(error.to_string())
        }
    }
}

/// T019 : O(log E+log D+log X+B), E ledger, D remises, X exécutions, B octets ; mémoire O(B).
/// Le caller garde State et les gardes jusqu’à la gravure. Les contrôles sont différés hors State.
/// Ordinary=false n’accepte que Prepared ; quota=true fige issued_at et ne repousse pas au rejeu.
fn deliver_reconstructed(
    state: &mut DaemonState,
    conn_id: &str,
    instance_id: &str,
    reconstructed: &crate::execution_store::ReconstructedExecution,
    issued_at: i64,
    quota_relaunch: bool,
) -> Result<&'static str, ReconstructedDeliveryError> {
    use ReconstructedDeliveryError::{NonReprenable, Storage};
    if let Some(task) = state
        .delegation_store
        .for_mission(&reconstructed.message.id)
        .map_err(Storage)?
        && task.remote_placement.is_some()
    {
        let command = native_delegation::remote_command_identity(state, &task)
            .map_err(|error| NonReprenable(format!("remote_identity:{error:?}")))?;
        if !task.remote_result_slots.iter().any(|slot| {
            slot.identity.command == command
                && slot.identity.mission_id == task.mission
                && slot.identity.round == task.round_no()
                && slot.identity.execution_id == reconstructed.snapshot.execution_id
                && slot.identity.execution_generation == reconstructed.snapshot.generation
        }) {
            return Err(NonReprenable("remote_result_slot_pending".into()));
        }
    }
    // Ordinary conserve exactement son ancien now. Quota relit l’horloge réelle
    // pour l’expiration, sans l’introduire dans le canonique de la continuation.
    let now = if quota_relaunch {
        unix_now_secs()
    } else {
        issued_at
    };
    let issuer_scope = state.idempotency.supervisor_scope()?;
    let key = IdempotencyKey::new(
        issuer_scope,
        OperationKind::Send,
        format!("recovery-{}", reconstructed.snapshot.execution_id),
    )?;
    let canonical = canonical_send(
        &key.issuer_scope,
        &key.idempotency_key,
        &reconstructed.message,
        issued_at,
    );
    let expires_at = match state.idempotency.reserve(
        &key,
        &canonical,
        issued_at,
        CLIENT_IDEMPOTENCY_HORIZON_SECS,
        now,
        CLIENT_ISSUED_AT_TOLERANCE_SECS,
    )? {
        Reservation::Prepared { expires_at } => expires_at,
        Reservation::Replayed(_) if quota_relaunch => {
            if let Some(phase) = state.idempotency.send_delivery_phase(&key)? {
                let stored = state
                    .idempotency
                    .stored_send_delivery(&key)?
                    .ok_or_else(|| {
                        NonReprenable("remise de relance absente après lecture de phase".into())
                    })?;
                if stored.delivery_id
                    != format!("delivery-recovery-{}", reconstructed.snapshot.execution_id)
                    || stored.phase != phase
                {
                    return Err(NonReprenable(
                        "identité ou phase de relance divergente".into(),
                    ));
                }
                let message = state
                    .idempotency
                    .stored_send_message(&key)?
                    .ok_or_else(|| NonReprenable("enveloppe de relance absente".into()))?;
                if message
                    != serde_json::to_value(&reconstructed.message)
                        .map_err(|error| NonReprenable(error.to_string()))?
                {
                    return Err(NonReprenable("enveloppe de relance divergente".into()));
                }
                let link = state
                    .idempotency
                    .delivery_execution_link(&stored.delivery_id)?
                    .ok_or_else(|| NonReprenable("lien de relance absent".into()))?;
                if link.execution_id != reconstructed.snapshot.execution_id
                    || link.submission_id != reconstructed.message.id
                {
                    return Err(NonReprenable("lien causal de relance divergent".into()));
                }
                // Aucune écriture, génération nouvelle ou émission dans ces quatre branches.
                return match phase.as_str() {
                    "dispatching" => Ok("dispatching"),
                    "acked" => Ok("acked"),
                    "indeterminate" => Ok("indeterminate"),
                    "orphaned" => Ok("orphaned"),
                    _ => Err(NonReprenable(format!(
                        "phase de relance non reprenable: {phase}"
                    ))),
                };
            }
            // Réservation préparée survivante : expiry initiale, aucun upsert ni nouveau canonique.
            state
                .idempotency
                .prepared_expiry(&key, now)?
                .ok_or_else(|| {
                    NonReprenable("réservation de relance non préparée ou expirée".into())
                })?
        }
        other => {
            return Err(NonReprenable(format!(
                "réservation interne inattendue pour {}: {other:?}",
                reconstructed.snapshot.execution_id
            )));
        }
    };
    let message_bytes = serde_json::to_vec(&reconstructed.message)
        .map_err(|error| NonReprenable(error.to_string()))?;
    let delivery = SendDelivery {
        delivery_id: format!("delivery-recovery-{}", reconstructed.snapshot.execution_id),
        recipient_instance_id: instance_id.to_string(),
        delivery_generation: next_delivery_generation(),
        expires_at,
        message_bytes,
    };
    let link = DeliveryExecutionLink {
        delivery_id: delivery.delivery_id.clone(),
        submission_id: reconstructed.message.id.clone(),
        execution_id: reconstructed.snapshot.execution_id.clone(),
    };
    state
        .idempotency
        .begin_send_delivery_with_execution(&key, &delivery, None, &link)?;
    get_metrics().record_execution_admitted();
    let mut controls = Vec::new();
    if let Err(error) =
        defer_idempotent_delivery(state, conn_id, delivery.clone(), &mut controls, None, None)
    {
        // L’ancien chemin garde sa quarantaine. Le contrat String de defer ne
        // permet pas d’inventer une classification métier d’une erreur SQL.
        if !quota_relaunch {
            let _ = state.idempotency.mark_delivery_indeterminate(
                &delivery.delivery_id,
                &delivery.recipient_instance_id,
                delivery.delivery_generation,
            );
        }
        return Err(Storage(error));
    }
    state
        .pending_post_response_controls
        .entry(conn_id.to_string())
        .or_default()
        .extend(controls);
    Ok("dispatching")
}

/// Lance le daemon.
pub fn run(config: DaemonConfig) -> Result<(), Box<dyn std::error::Error>> {
    crate::service_limits::ensure_open_file_limit()?;
    let state_root = config.db_path.parent().ok_or("magasin sans racine")?;
    crate::environment::validate_existing_tree(state_root)?;
    crate::environment::ensure_private_directory(state_root)?;
    let socket_root = config.socket_path.parent().ok_or("socket sans racine")?;
    if socket_root != state_root {
        crate::environment::validate_existing_tree(socket_root)?;
    }
    crate::environment::ensure_private_directory(socket_root)?;
    crate::environment::validate_state_file(&config.db_path, false)?;
    crate::environment::validate_state_file(&config.log_path, false)?;
    crate::environment::validate_state_file(&config.socket_path, true)?;
    crate::store_schema::validate_existing(&config.db_path)?;
    // Gestionnaires de signaux AVANT toute trace visible de l'extérieur.
    //
    // Ils étaient installés après la liaison de la socket : entre le moment où
    // le daemon devenait joignable et celui où il devenait interruptible
    // proprement, un SIGTERM tombait sur la disposition PAR DÉFAUT et tuait le
    // processus net — socket et fichier PID abandonnés derrière lui. Un
    // démarrage suivi d'un arrêt immédiat laissait donc des reliques que le
    // démarrage suivant devait déblayer. Le drapeau est remis à zéro d'abord :
    // un SIGTERM arrivé avant la boucle sera vu à sa première itération.
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
    install_daemon_signal_handlers();

    // Verrouillage exclusif avec flock — empêche deux daemons de démarrer en même temps
    // Évite la race condition TOCTOU du PID file traditionnel
    let pid_file = config.socket_path.with_extension("pid");
    crate::environment::validate_state_file(&pid_file, false)?;

    // Créer le fichier et obtenir un verrou exclusif avec flock
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        // Ce descripteur ne sert qu'au verrou flock ; l'écriture du PID suit
        // avec std::fs::write et effectue explicitement le remplacement.
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&pid_file)
        .map_err(|e| format!("Impossible de créer PID file {}: {}", pid_file.display(), e))?;

    // Tenter d'obtenir un verrou exclusif (non-bloquant)
    // SAFETY: les pointeurs passés à flock désignent des variables locales valides pendant l'appel.
    unsafe {
        if libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) != 0 {
            // Le verrou échoue = un autre daemon tourne
            eprintln!(
                "bridget: un daemon tourne déjà (verrou sur {})",
                pid_file.display()
            );
            return Err("namespace occupé : daemon déjà démarré".into());
        }
    }

    // Écrire notre PID maintenant qu'on a le verrou
    file.set_len(0)?;
    file.write_all(std::process::id().to_string().as_bytes())?;
    eprintln!(
        "[BRIDGET] PID file écrit avec verrou exclusif: {} (PID {})",
        pid_file.display(),
        std::process::id()
    );

    // Lire les autorités durables avant la réconciliation, sous le flock et
    // le préflight privé. Aucun superviseur ni socket nouvelle n'est lancé.
    let (managed_tx, managed_rx) = mpsc::channel();
    let (managed_event_tx, managed_event_rx) = mpsc::channel();
    let state = Arc::new(Mutex::new(DaemonState::new(&config, managed_tx.clone())?));
    let native_identities = {
        let st = state.lock().recover_poison();
        native_delegation::restart_identities(&st)?
    };
    let marker_store = ManagedMarkerStore::at_directory(state_root.join("managed"));
    let reconciled = marker_store.reconcile_stale_groups_with_native(
        MANAGED_STOP_FORCED_GRACE,
        MANAGED_STOP_POLL,
        &native_identities,
        Duration::from_secs(8),
    )?;
    if !reconciled.is_empty() {
        info!(
            "réconciliation: {} ancien(s) groupe(s) terminé(s): {}",
            reconciled.len(),
            reconciled.join(", ")
        );
    }

    if config.socket_path.exists() {
        if UnixStream::connect(&config.socket_path).is_ok() {
            return Err("socket occupée : refus de remplacer un daemon actif".into());
        }
        std::fs::remove_file(&config.socket_path)?;
    }
    std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
    crate::wrapper::purge_orphan_mcp_configs(config.socket_path.parent().unwrap());
    // WARN disque synchrone (statvfs = O(1)). Le ramassage /tmp sort du chemin
    // critique — thread détaché après readiness (bisect fable2).
    crate::disk_hygiene::warn_if_disk_low(std::path::Path::new("/"));

    let listener = UnixListener::bind(&config.socket_path)?;
    info!("bridget daemon écoute sur {}", config.socket_path.display());

    // Ne jamais balayer le /tmp partagé : seuls nos temporaires privés.
    let tmp_dir = state_root.join("tmp");
    crate::environment::ensure_private_directory(&tmp_dir)?;
    if let Err(error) = std::thread::Builder::new()
        .name("ramasse-copies".into())
        .spawn(move || {
            let report = crate::disk_hygiene::purge_orphan_bridget_tmp(&tmp_dir);
            if !report.deleted.is_empty() {
                info!(
                    "ramasse-copies: {} temporaire(s) privé(s) retiré(s)",
                    report.deleted.len()
                );
            }
            if report.deferred > 0 {
                info!(
                    "ramasse-copies: {} restant(s) pour un prochain passage",
                    report.deferred
                );
            }
        })
    {
        warn!("ramasse-copies: thread détaché impossible: {error}");
    }

    let fleet = state.lock().recover_poison().fleet.clone();
    #[cfg(any(test, feature = "test-support"))]
    let executable_override =
        std::env::var_os("BRIDGET_T908_MANAGED_EXECUTABLE").map(PathBuf::from);
    #[cfg(not(any(test, feature = "test-support")))]
    let executable_override = None;
    let supervisor_guard = ManagedSupervisorGuard::from_existing_sender(
        managed_tx,
        managed_rx,
        fleet,
        &config,
        managed_event_tx,
        executable_override,
    );
    let recoveries = {
        let mut st = state.lock().recover_poison();
        reserve_managed_recoveries(&mut st, unix_timestamp())?
    };
    for (prepared, stop) in recoveries {
        let command_id = prepared.lease.command_id.clone();
        let instance_id = prepared.lease.instance_id.clone();
        let sent = {
            let st = state.lock().recover_poison();
            st.managed_tx
                .send(ManagedSupervisorCommand::Start { prepared, stop })
        };
        if sent.is_err() {
            let mut st = state.lock().recover_poison();
            if let Some(record) = st.managed_spawns.remove(&command_id) {
                let _ = st.fleet.fail(
                    &record.lease,
                    "negotiation_failed",
                    "superviseur indisponible pendant la reprise",
                );
            }
            st.managed_by_instance.remove(&instance_id);
            finish_recovery_command(&mut st, &command_id);
        }
    }

    // Purge au démarrage
    {
        let st = state.lock().recover_poison();
        if let Ok(n) = st.store.purge_older_than_days(config.retention_days)
            && n > 0
        {
            info!(
                "purge: {} messages supprimés (> {} jours)",
                n, config.retention_days
            );
        }
    }

    // Thread de surveillance des --reply sans réponse (escalade progressive)
    // Palier 1 (T/3) : rappel discret au destinataire
    // Palier 2 (2T/3) : rappel ferme au destinataire
    // Palier 3 (T) : notification d'échec à l'émetteur
    // Après T + 30s : abandon (retiré de la liste)
    // Session 116 : entretien périodique. Le ramassage lancé plus haut ne
    // passe qu'au démarrage, soit une fois tous les quelques jours : sagas
    // d'envoi, état d'identité des processus disparus et journaux
    // s'accumulaient entre deux relances. Ce fil tient un clone de l'état : il
    // sort à l'arrêt, sans retarder l'extinction.
    let st_entretien = state.clone();
    let racine_entretien = state_root.to_path_buf();
    if let Err(error) = thread::Builder::new()
        .name("entretien".into())
        .spawn(move || {
            let mut attente = MAINTENANCE_FIRST_DELAY;
            loop {
                if sleep_until_shutdown(attente) {
                    return;
                }
                run_maintenance(&st_entretien, &racine_entretien);
                attente = MAINTENANCE_INTERVAL;
            }
        })
    {
        warn!("entretien: thread détaché impossible: {error}");
    }

    // Les vues possèdent leurs rédacteurs. Leur fermeture peut attendre une
    // socket lente ; aucune inscription, lecture ou annulation n'en dépend.
    // Un seul collecteur, sans file supplémentaire, et sans Arc gardé en veille.
    let views_state = Arc::downgrade(&state);
    thread::Builder::new()
        .name("attach-cleanup".into())
        .spawn(move || {
            loop {
                if sleep_until_shutdown(Duration::from_millis(100)) {
                    return;
                }
                let Some(shared) = views_state.upgrade() else {
                    return;
                };
                let Some(_activity) = update_activity(&shared) else {
                    continue;
                };
                let (controls, views) = {
                    let mut st = shared.lock().recover_poison();
                    collect_closed_attach_views(&mut st)
                };
                drop(_activity);
                drop(shared);
                let _ = execute_controls(controls);
                for view in views {
                    view.close_and_join();
                }
            }
        })?;

    let st_reminder = state.clone();
    thread::spawn(move || {
        loop {
            // Sortie sur arrêt : ce thread détient un clone de l'Arc d'état, et
            // l'état détient `managed_tx`. Tant qu'il boucle, le canal du
            // superviseur ne peut PAS se fermer et l'arrêt reste suspendu.
            if sleep_until_shutdown(Duration::from_secs(1)) {
                return;
            }
            let now = std::time::Instant::now();

            let Some(_update_activity) = update_activity(&st_reminder) else {
                continue;
            };
            user_input::tick(&st_reminder);
            remote_worker_contact_tick(&st_reminder);
            let tick_start = Instant::now();
            native_delegation::tick(&st_reminder);
            let tick_elapsed = tick_start.elapsed();
            if tick_elapsed > NATIVE_TICK_SLOW {
                warn!("tour de délégation lent : {} ms", tick_elapsed.as_millis());
            }
            let wake_clock = Instant::now();
            let wake_now = unix_now_secs();
            scheduled_wakes_tick(&st_reminder, wake_now, wake_clock);
            observer_return_tick(&st_reminder, wake_now, wake_clock);

            // Collecter les actions à faire
            let actions: Vec<ReminderAction> = {
                let mut st = st_reminder.lock().recover_poison();
                collect_reminder_actions(&mut st, now)
            };
            // Session 102 : réévaluation bornée des sollicitations de fil
            // (membre revenu, DND levé, capacité annoncée, échéance passée).
            let wake_controls = {
                let mut st = st_reminder.lock().recover_poison();
                let mut controls = Vec::new();
                dispatch_thread_wakes(&mut st, unix_now_secs(), now, &mut controls);
                controls
            };
            let _ = execute_controls(wake_controls);
            // Session 119 : avis d'état de source, émis une fois stables.
            {
                let mut st = st_reminder.lock().recover_poison();
                let notices = st.observations.due_source_notices(now);
                queue_observation_notifications(&mut st, notices);
            }

            // Exécuter les actions hors lock
            for action in actions {
                match action {
                    ReminderAction::PushDeferred { target_conn } => {
                        let controls = st_reminder
                            .lock()
                            .recover_poison()
                            .pending_post_response_controls
                            .remove(&target_conn)
                            .unwrap_or_default();
                        let retry = execute_controls(controls);
                        if !retry.is_empty() {
                            st_reminder
                                .lock()
                                .recover_poison()
                                .pending_post_response_controls
                                .entry(target_conn)
                                .or_default()
                                .extend(retry);
                        }
                    }
                    ReminderAction::Gentle {
                        to,
                        from,
                        msg_id,
                        target_conn,
                    } => {
                        let target_writer = {
                            let st = st_reminder.lock().recover_poison();
                            st.connections.get(&target_conn).cloned()
                        };
                        if let Some(target_writer) = target_writer {
                            let body = format!(
                                "Rappel : {} attend ta reponse au message #{}.\nReponds avec: bridget reply \"ta reponse\"",
                                from,
                                &msg_id[..msg_id.len().min(8)]
                            );
                            match deliver_to_agent(&target_writer, &to, &body) {
                                Ok(reminder_message_id) => attest_reminder_sent(
                                    &st_reminder,
                                    &msg_id,
                                    &reminder_message_id,
                                    &to,
                                    1,
                                ),
                                Err(e) => {
                                    error!("Impossible de délivrer le rappel doux à {}: {}", to, e)
                                }
                            }
                            info!("palier 1 (rappel doux) → {} pour {}", to, from);
                        }
                    }
                    ReminderAction::Firm {
                        to,
                        from,
                        msg_id,
                        target_conn,
                    } => {
                        let target_writer = {
                            let st = st_reminder.lock().recover_poison();
                            st.connections.get(&target_conn).cloned()
                        };
                        if let Some(target_writer) = target_writer {
                            let body = format!(
                                "URGENT : {} attend toujours ta reponse au message #{}.\nTu DOIS repondre maintenant avec: bridget reply \"ta reponse\"\nSi tu ne peux pas repondre, notifie-le : bridget reply \"impossible de repondre : <raison>\"",
                                from,
                                &msg_id[..msg_id.len().min(8)]
                            );
                            match deliver_to_agent(&target_writer, &to, &body) {
                                Ok(reminder_message_id) => attest_reminder_sent(
                                    &st_reminder,
                                    &msg_id,
                                    &reminder_message_id,
                                    &to,
                                    2,
                                ),
                                Err(e) => {
                                    error!("Impossible de délivrer le rappel ferme à {}: {}", to, e)
                                }
                            }
                            info!("palier 2 (rappel ferme) → {} pour {}", to, from);
                        }
                    }
                    ReminderAction::Timeout {
                        to,
                        from,
                        msg_id,
                        from_conn,
                        timeout_secs,
                        cancel_target,
                    } => {
                        if let Some(target_conn) = cancel_target {
                            let writer = st_reminder
                                .lock()
                                .recover_poison()
                                .connections
                                .get(&target_conn)
                                .cloned();
                            if let Some(writer) = writer {
                                let _ = push_control_message_until(
                                    &writer,
                                    &DaemonToWrapper::CancelDelivery {
                                        id: msg_id.clone(),
                                        reason: "échéance de la demande dépassée".into(),
                                        interrupt_active: None,
                                    },
                                    Instant::now() + Duration::from_secs(1),
                                );
                            }
                        }
                        let st = st_reminder.lock().recover_poison();
                        let sender_writer = st.connections.get(&from_conn).cloned();
                        // SPEC-087 T029 : une demande du référent restée sans
                        // réponse est une dette humaine ; elle arrive dans sa
                        // boîte, pas seulement dans le fil.
                        if from == UI_HUMAN_SENDER || from == UI_HUMAN_AGENT_ID {
                            let now = unix_now_secs();
                            let context = serde_json::json!({
                                "summary": format!(
                                    "{to} n’a pas répondu en {timeout_secs} s à ta demande #{}.",
                                    &msg_id[..msg_id.len().min(8)]
                                ),
                                "to": to,
                                "timeout_secs": timeout_secs,
                            })
                            .to_string();
                            let _ = daemon_deposit_human_inbox(
                                &st,
                                &format!("reply-debt:{msg_id}"),
                                bridget_transport::protocol::HumanInboxKind::ReplyDebt,
                                &bridget_transport::protocol::HumanInboxSubject {
                                    message_id: Some(msg_id.clone()),
                                    agent_id: Some(to.clone()),
                                    ..Default::default()
                                },
                                &context,
                                &["ack".to_string()],
                                now,
                            );
                        }
                        drop(st);
                        // Aucun verrou global pendant la notification réseau.
                        if let Some(sender_writer) = sender_writer {
                            let body = format!(
                                "{} n'a pas repondu en {}s au message #{}.\nTu peux reessayer, changer de destinataire ou abandonner.",
                                to,
                                timeout_secs,
                                &msg_id[..msg_id.len().min(8)]
                            );
                            match deliver_to_agent(&sender_writer, &from, &body) {
                                Ok(_) => info!(
                                    "palier 3 (timeout notifié à {} : {} n'a pas répondu)",
                                    from, to
                                ),
                                Err(e) => error!(
                                    "Impossible de délivrer la notification de timeout à {}: {}",
                                    from, e
                                ),
                            }
                        }
                    }
                    ReminderAction::Deferred {
                        to,
                        msg_id,
                        level,
                        reason,
                    } => {
                        info!(
                            "relance différée ({reason}) : palier {} pour {} sur demande {}",
                            level, to, msg_id
                        );
                    }
                }
            }
        }
    });

    // Thread de purge périodique
    let st_purge = state.clone();
    let retention = config.retention_days;
    thread::spawn(move || {
        loop {
            // Même raison qu'au-dessus, en plus aigu : une sieste d'une heure
            // gardait l'état vivant une heure de plus après l'ordre d'arrêt.
            if sleep_until_shutdown(Duration::from_secs(3600)) {
                return;
            }
            let st = st_purge.lock().recover_poison();
            // SPEC-087 T031 : rappel des items de boîte ouverts depuis trop
            // longtemps, seule décision « de ronde » admise par la spec.
            match crate::human_inbox::remind_overdue(
                st.store.connection(),
                human_channel_config().as_ref(),
                unix_now_secs(),
            ) {
                Ok(0) => {}
                Ok(count) => {
                    info!("boîte humaine: {count} rappel(s) poussé(s) sur le canal externe")
                }
                Err(error) => warn!("boîte humaine: rappel impossible: {error}"),
            }
            if let Ok(n) = st.store.purge_older_than_days(retention)
                && n > 0
            {
                info!("purge périodique: {} messages supprimés", n);
            }
        }
    });

    // Boucle d'acceptation avec timeout pour vérifier shutdown
    listener.set_nonblocking(true)?;
    loop {
        if let Some(_update_activity) = update_activity(&state) {
            drain_managed_events(&state, &managed_event_rx);
        }
        // Vérifier si shutdown demandé
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            info!("shutdown demandé - notification des wrappers externes...");
            let st = state.lock().recover_poison();
            for (conn_id, writer) in &st.connections {
                if !st.notify_wrapper_on_daemon_shutdown(conn_id) {
                    continue;
                }
                let msg = DaemonToWrapper::Disconnect;
                if let Ok(json) = encode(&msg)
                    && let Ok(mut w) = writer.lock()
                {
                    let _ = writeln!(w, "{}", json);
                    let _ = w.flush();
                }
            }
            drop(st);
            if let Err(e) = std::fs::remove_file(&config.socket_path) {
                log::warn!(
                    "Impossible de supprimer socket {}: {}",
                    config.socket_path.display(),
                    e
                );
            }
            if let Err(e) = std::fs::remove_file(config.socket_path.with_extension("pid")) {
                log::warn!(
                    "Impossible de supprimer PID file {}: {}",
                    config.socket_path.with_extension("pid").display(),
                    e
                );
            }
            info!("daemon arrêté proprement");
            drop(state);
            supervisor_guard.shutdown();
            return Ok(());
        }

        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(e) = stream.set_nonblocking(false) {
                    error!("set_nonblocking failed: {} — connexion ignorée", e);
                    continue;
                }
                let st = state.clone();
                get_metrics().increment_connections();
                thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handle_connection(stream, st)
                    })) {
                        error!("PANIC dans thread connexion: {:?}", e);
                    }
                    get_metrics().decrement_connections();
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                error!("accept: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Récolte sans attente les connexions des sockets privés de projets. Elles
/// restent hors de la socket utilisateur afin qu'un chemin monté ne suffise pas
/// à se faire passer pour un wrapper hôte.
fn handle_connection(
    stream: UnixStream,
    state: Arc<Mutex<DaemonState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let peer_uid = unix_peer_uid(&stream).ok();
    let conn_id = state.lock().recover_poison().next_conn_id();
    log::debug!("handle_connection: nouvelle connexion {}", conn_id);
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);

    // Enregistrer le writer dans la map pour les push
    let push_stream = stream.try_clone()?;
    {
        let mut st = state.lock().recover_poison();
        st.connections.insert(
            conn_id.clone(),
            Arc::new(Mutex::new(BufWriter::new(push_stream))),
        );
        if let Some(peer_uid) = peer_uid {
            st.peer_uids.insert(conn_id.clone(), peer_uid);
        }
    }
    info!("connexion {} établie", conn_id);

    let mut my_writer = BufWriter::new(stream);

    // Toute sortie, y compris un échec d'écriture de réponse, traverse le
    // nettoyage commun ci-dessous. Une capacité de service est strictement
    // attachée à la connexion : elle ne doit jamais survivre à son socket.
    let connection_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let frame = match bridget_transport::jsonl::read_unix_line(
                &mut reader,
                bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES,
                bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(10)),
            ) {
                Ok(Some(frame)) => frame,
                Ok(None) => break,
                Err(error) => return Err(error.into()),
            };
            let wire_len = frame.len();
            if wire_len > bridget_transport::protocol::MAX_WORKER_FRAME_BYTES
                && state.lock().recover_poison().connection_roles.get(&conn_id)
                    == Some(&ConnectionRole::Worker)
            {
                return Err("worker frame exceeds bound".into());
            }
            let raw = std::str::from_utf8(&frame)?;
            let line = raw
                .strip_suffix('\n')
                .unwrap()
                .strip_suffix('\r')
                .unwrap_or_else(|| raw.strip_suffix('\n').unwrap());
            if line.is_empty() {
                continue;
            }
            if guichet_frame_exceeds_wire_limit(line, wire_len) {
                let json = encode(&DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::FrameTooLarge,
                })?;
                writeln!(my_writer, "{}", json)?;
                my_writer.flush()?;
                continue;
            }

            // Contrat fermé de lecture : une extension inconnue ne disparaît
            // pas silencieusement dans les defaults Serde de l'enveloppe commune.
            let display_name_kind = serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|value| {
                    value
                        .get("type")
                        .and_then(|kind| kind.as_str())
                        .map(str::to_owned)
                });
            if display_name_kind.as_deref() == Some("HumanLineage")
                && (decode::<WrapperToDaemon>(line).is_err()
                    || serde_json::from_str::<serde_json::Value>(line)
                        .ok()
                        .is_none_or(|v| v.as_object().is_none_or(|o| o.len() != 2)))
            {
                writeln!(
                    my_writer,
                    "{}",
                    encode(&DaemonToWrapper::HumanLineageResult {
                        result: bridget_transport::protocol::HumanLineageError::InvalidRequest
                            .result()
                    })?
                )?;
                my_writer.flush()?;
                break;
            }
            if display_name_kind.as_deref() == Some("HumanThreadViewV1")
                && decode::<WrapperToDaemon>(line).is_err()
            {
                let response = DaemonToWrapper::HumanThreadViewResult {
                    result: bridget_transport::protocol::HumanThreadViewResult::error(
                        bridget_transport::protocol::HumanThreadViewError::InvalidRequest,
                    ),
                };
                writeln!(my_writer, "{}", encode(&response)?)?;
                my_writer.flush()?;
                continue;
            }
            if display_name_kind.as_deref() == Some("HumanThreadWatchV1")
                && (decode::<WrapperToDaemon>(line).is_err()
                    || !serde_json::from_str::<serde_json::Value>(line)
                        .ok()
                        .and_then(|value| value.as_object().map(|object| object.len() == 2))
                        .unwrap_or(false))
            {
                let event = bridget_transport::protocol::HumanThreadWatchEvent::error(
                    bridget_transport::protocol::HumanThreadViewError::InvalidRequest,
                );
                writeln!(
                    my_writer,
                    "{}",
                    encode(&DaemonToWrapper::HumanThreadWatchEvent { event })?
                )?;
                my_writer.flush()?;
                break;
            }
            if matches!(
                display_name_kind.as_deref(),
                Some("display_name_set" | "display_name_resolve")
            ) && !decode::<WrapperToDaemon>(line)
                .ok()
                .is_some_and(|message| encode(&message).is_ok_and(|canonical| canonical == line))
            {
                let response = if display_name_kind.as_deref() == Some("display_name_resolve") {
                    DaemonToWrapper::DisplayNameResolved {
                        outcome: bridget_transport::protocol::DisplayNameResolution::Rejected {
                            reason: bridget_transport::protocol::DisplayNameRefusal::InvalidRequest,
                        },
                    }
                } else {
                    DaemonToWrapper::DisplayNameResult {
                        outcome: bridget_transport::protocol::DisplayNameOutcome::Rejected {
                            reason: bridget_transport::protocol::DisplayNameRefusal::InvalidRequest,
                        },
                    }
                };
                writeln!(my_writer, "{}", encode(&response)?)?;
                my_writer.flush()?;
                continue;
            }
            if serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .is_some_and(|value| {
                    value.get("type").and_then(|kind| kind.as_str()) == Some("artifact_read")
                })
                && !decode::<WrapperToDaemon>(line).ok().is_some_and(|message| {
                    matches!(message, WrapperToDaemon::ArtifactRead { .. })
                        && encode(&message).is_ok_and(|canonical| canonical == line)
                })
            {
                writeln!(
                    my_writer,
                    "{}",
                    encode(&artifact_read_response(
                        None,
                        ArtifactReadOutcome::Rejected {
                            reason: ArtifactReadRefusal::InvalidRequest
                        }
                    ))?
                )?;
                my_writer.flush()?;
                continue;
            }

            let msg: WrapperToDaemon = match decode(line) {
                Ok(m) => m,
                Err(_) => {
                    // Les erreurs Serde peuvent citer une valeur fournie,
                    // notamment un credential mal typé. Ne jamais la journaliser.
                    warn!("message de protocole illisible de {}", conn_id);
                    if raw_guichet_frame(line) {
                        let json = encode(&DaemonToWrapper::ServiceRejected {
                            reason: ServiceRefusal::InvalidEnvelope,
                        })?;
                        writeln!(my_writer, "{}", json)?;
                        my_writer.flush()?;
                    }
                    continue;
                }
            };

            // Le compteur couvre aussi la réponse et ses contrôles différés.
            let _update_activity = if update_exempt(&msg) {
                None
            } else {
                match update_activity(&state) {
                    Some(guard) => Some(guard),
                    None => {
                        writeln!(my_writer, "{}", encode(&DaemonToWrapper::UpdateInProgress)?)?;
                        my_writer.flush()?;
                        break;
                    }
                }
            };

            // Le guichet compare les octets, non une valeur JSON reparsée :
            // cette vérification rejette aussi champs inconnus, doublons et
            // ordre de clés divergent avant toute écriture SQLite.
            if is_guichet_frame(&msg) && encode(&msg).is_ok_and(|canonical| canonical != line) {
                let json = encode(&DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::CanonicalBytesMismatch,
                })?;
                writeln!(my_writer, "{}", json)?;
                my_writer.flush()?;
                continue;
            }

            let response =
                handle_wrapper_message_with_peer(&conn_id, msg, &state, Some(reader.get_ref()));
            let registered_just_now = matches!(response, Some(DaemonToWrapper::Registered { .. }));
            if let Some(dtw) = response {
                let watch_terminal = matches!(
                    &dtw,
                    DaemonToWrapper::HumanThreadWatchEvent {
                        event: bridget_transport::protocol::HumanThreadWatchEvent::Error { .. }
                    }
                );
                let worker_writer = {
                    let st = state.lock().recover_poison();
                    (st.connection_roles.get(&conn_id) == Some(&ConnectionRole::Worker))
                        .then(|| st.connections.get(&conn_id).cloned())
                        .flatten()
                };
                if let Some(writer) = worker_writer {
                    push_control_message_until(
                        &writer,
                        &dtw,
                        Instant::now() + CANCEL_NOTIFICATION_BUDGET,
                    )?;
                } else {
                    let json = encode(&dtw)?;
                    writeln!(my_writer, "{}", json)?;
                    my_writer.flush()?;
                }
                if watch_terminal {
                    break;
                }
            }
            let post_response_controls = state
                .lock()
                .recover_poison()
                .pending_post_response_controls
                .remove(&conn_id)
                .unwrap_or_default();
            let _ = execute_controls(post_response_controls);
            // Après Registered : rejouer les notices ORPHELIN dont l'émetteur
            // était hors ligne à la purge (voir flush_pending_orphan_emitter_notices).
            if registered_just_now {
                {
                    let mut st = state.lock().recover_poison();
                    if let Some((owner, instance)) = live_connection_identity(&st, &conn_id)
                        && !st.auxiliary_connections.contains(&conn_id)
                        && st.presences.get(&instance).is_some_and(|p| !p.is_dnd())
                    {
                        let notices = st.observations.owner_returned(&owner, Instant::now());
                        queue_observation_notifications(&mut st, notices);
                    }
                }
                state
                    .lock()
                    .recover_poison()
                    .flush_pending_orphan_emitter_notices();
                replay_delegated_runtime_events_for_connection(&state, &conn_id);
                let return_clock = Instant::now();
                observer_return_tick(&state, unix_now_secs(), return_clock);
            }
        }
        Ok(())
    })();

    // Connexion fermée : désenregistrer avec nettoyage explicite pour éviter fuites
    let (writer_opt, removed, controls, views) = {
        let mut st = state.lock().recover_poison();
        let (controls, views) = close_attach_subscriptions(&mut st, &conn_id);

        // Récupérer le writer AVANT suppression pour nettoyage explicite
        let writer_opt = st.connections.remove(&conn_id);

        let removed = st.router.unregister_by_conn(&conn_id);
        if let Some(route) = &removed {
            let _ = st
                .store
                .observation_owner_presence(&route.agent_id, false, unix_now_secs());
        }
        st.mark_unreachable(&conn_id);
        st.conn_names.remove(&conn_id);
        st.observation_sequences.remove(&conn_id);
        st.runtime_catalogue_connections.remove(&conn_id);
        st.observations.remove_source(&conn_id, Instant::now());
        st.conn_hosts.remove(&conn_id);
        st.communication_projects.remove(&conn_id);
        st.conn_operating_systems.remove(&conn_id);
        st.connection_roles.remove(&conn_id);
        st.late_worker_prepares.forget_connection(&conn_id);
        remote_worker_disconnected(&mut st, &conn_id);
        st.worker_probe_connections.remove(&conn_id);
        st.terminal_sessions.remove(&conn_id);
        st.client_negotiations.remove(&conn_id);
        st.human_thread_watches.remove(&conn_id);
        st.lineage_watches.remove(&conn_id);
        if let Some(journal) = st.lineage_journals.remove(&conn_id)
            && let Some(socket) = journal.shutdown
        {
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
        st.service_negotiations.remove(&conn_id);
        st.peer_uids.remove(&conn_id);
        st.coordination_subscriptions.remove(&conn_id);
        let _ = st.store.release_guichet_claims(&conn_id);
        (writer_opt, removed, controls, views)
    };
    let _ = execute_controls(controls);
    for view in views {
        view.close_and_join();
    }

    // Nettoyage explicite du writer pour éviter fuites de ressources
    if let Some(writer_mutex) = writer_opt
        && let Ok(mut writer) = writer_mutex.lock()
    {
        use std::io::Write;
        let _ = writer.flush();
        // Le drop explicite fermera le stream proprement
    }

    if let Some(agent) = removed {
        info!("agent '{}' déconnecté ({})", agent.agent_id, conn_id);
    } else {
        info!("connexion {} fermée (non enregistrée)", conn_id);
    }
    log::debug!("handle_connection {} terminée", conn_id);

    connection_result
}

#[cfg(target_os = "linux")]
pub(crate) fn unix_peer_uid(stream: &UnixStream) -> std::io::Result<u32> {
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut length,
        )
    };
    if result != 0 || length != std::mem::size_of::<libc::ucred>() as libc::socklen_t {
        return Err(std::io::Error::last_os_error());
    }
    Ok(credentials.uid)
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) fn unix_peer_uid(stream: &UnixStream) -> std::io::Result<u32> {
    let mut uid: libc::uid_t = 0;
    let mut gid: libc::gid_t = 0;
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    let result = unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(uid)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "ios")))]
fn unix_peer_uid(_stream: &UnixStream) -> std::io::Result<u32> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "SO_PEERCRED indisponible sur cette plateforme",
    ))
}

fn project_registry_failure(
    request: ProjectBindRequest,
    reason: ProjectRegistryRefusal,
    observed_at: i64,
) -> DaemonToWrapper {
    DaemonToWrapper::ProjectRegistryOutcome {
        outcome: project_registry_failed_outcome(
            request.contract_version,
            request.command_id,
            request.project_id,
            reason,
            observed_at,
        ),
    }
}

fn project_registry_failed_outcome(
    contract_version: u16,
    command_id: String,
    project_id: String,
    reason: ProjectRegistryRefusal,
    observed_at: i64,
) -> ProjectBindOutcome {
    ProjectBindOutcome {
        contract_version,
        command_id,
        project_id,
        status: ProjectBindStatus::BindingFailed,
        binding_generation: None,
        backend: None,
        runtime_policy: None,
        reason: Some(reason),
        existing_project_id: None,
        existing_binding_generation: None,
        observed_at,
    }
}

fn project_registry_admin_failure(
    request: &ProjectAdminRequest,
    reason: ProjectRegistryRefusal,
    observed_at: i64,
) -> DaemonToWrapper {
    DaemonToWrapper::ProjectRegistryAdminOutcome {
        outcome: ProjectAdminOutcome {
            contract_version: request.contract_version,
            command_id: request.command_id.clone(),
            operation: request.operation,
            bindings: Vec::new(),
            reason: Some(reason),
            observed_at,
        },
    }
}

fn project_round_failure(
    request: &ProjectRoundRequest,
    reason: ProjectRoundRefusal,
    observed_at: i64,
) -> DaemonToWrapper {
    DaemonToWrapper::ProjectRoundOutcome {
        outcome: ProjectRoundOutcome {
            contract_version: request.contract_version,
            command_id: request.command_id.clone(),
            operation: request.operation,
            policies: Vec::new(),
            reason: Some(reason),
            observed_at,
        },
    }
}

fn project_round_dispatch_failure(
    request: &ProjectRoundDispatchRequest,
    reason: ProjectRoundRefusal,
    observed_at: i64,
) -> DaemonToWrapper {
    DaemonToWrapper::ProjectRoundDispatchOutcome {
        outcome: ProjectRoundDispatchOutcome {
            contract_version: request.contract_version,
            occurrence_at: request.occurrence_at,
            project: request.project.clone(),
            issue: None,
            reason: Some(reason),
            observed_at,
        },
    }
}

fn project_profile_failure(
    request: &ProjectProfileRequest,
    reason: ProjectProfileRefusal,
    observed_at: i64,
) -> DaemonToWrapper {
    DaemonToWrapper::ProjectProfileOutcome {
        outcome: ProjectProfileOutcome {
            contract_version: request.proposal.contract_version,
            command_id: request.proposal.command_id.clone(),
            project: request.proposal.project.clone(),
            backend: None,
            profile: None,
            reason: Some(reason),
            observed_at,
        },
    }
}

fn project_runtime_failure(
    request: &ProjectRuntimeRequest,
    reason: ProjectRuntimeRefusal,
    observed_at: i64,
) -> DaemonToWrapper {
    DaemonToWrapper::ProjectRuntimeOutcome {
        outcome: ProjectRuntimeOutcome {
            contract_version: request.contract_version,
            command_id: request.command_id.clone(),
            project_id: request.project_id.clone(),
            operation: request.operation,
            binding_generation: None,
            state: None,
            runtime_policy: None,
            last_reason: None,
            reason: Some(reason),
            observed_at,
        },
    }
}

fn is_guichet_frame(message: &WrapperToDaemon) -> bool {
    matches!(
        message,
        WrapperToDaemon::ServiceHello { .. }
            | WrapperToDaemon::ProjectRegistryRequest { .. }
            | WrapperToDaemon::ProjectRegistryAdminRequest { .. }
            | WrapperToDaemon::ProjectSystemRequest { .. }
            | WrapperToDaemon::ProjectProfileRequest { .. }
            | WrapperToDaemon::CoordinationSubscribe { .. }
            | WrapperToDaemon::ServiceRequest { .. }
            | WrapperToDaemon::GuichetClaimNext { .. }
            | WrapperToDaemon::GuichetClaim { .. }
            | WrapperToDaemon::GuichetLookup { .. }
            | WrapperToDaemon::GuichetReply { .. }
    )
}

/// `BufRead::lines` enlève le séparateur : la borne du contrat porte bien sur
/// la trame JSONL entière, donc sur la ligne plus son LF filaire.
fn guichet_frame_exceeds_wire_limit(line: &str, wire_len: usize) -> bool {
    wire_len > MAX_GUICHET_FRAME_BYTES && raw_guichet_frame(line)
}

/// Classe la famille du message à partir du JSON, jamais d'une sous-chaîne :
/// les espaces et l'ordre des clés ne doivent pas contourner la garde avant
/// toute consommation de capacité ou écriture durable.
fn raw_guichet_frame(line: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|kind| {
            matches!(
                kind.as_str(),
                "ServiceHello"
                    | "project_registry_request"
                    | "coordination_subscribe"
                    | "service_request"
                    | "guichet_claim_next"
                    | "guichet_claim"
                    | "guichet_lookup"
                    | "guichet_reply"
                    | "coordination_event"
            )
        })
}

fn guichet_request_is_valid(
    version: u16,
    request_id: &str,
    operation: bridget_transport::protocol::ServiceRequestOperation,
    payload: &bridget_transport::protocol::ServiceRequestPayload,
) -> bool {
    let identifier = |value: &str| {
        !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
    };
    let version_matches_payload = match (version, operation, payload) {
        (
            SERVICE_CONTRACT_VERSION,
            bridget_transport::protocol::ServiceRequestOperation::Delegate,
            bridget_transport::protocol::ServiceRequestPayload::Delegate {
                review_target,
                origin,
                focus,
                ..
            },
        ) => review_target.is_none() && origin.is_none() && focus.is_none(),
        (SERVICE_CONTRACT_VERSION, _, _) => true,
        (
            REVIEW_DELEGATE_CONTRACT_VERSION,
            bridget_transport::protocol::ServiceRequestOperation::Delegate,
            bridget_transport::protocol::ServiceRequestPayload::Delegate {
                review_target: Some(target),
                ..
            },
        ) => target.is_valid(),
        // SPEC-087 : une origine humaine ou un focus voyagent en v2, sans
        // cible de revue obligatoire (le service compagnon la mesure elle-même).
        (
            REVIEW_DELEGATE_CONTRACT_VERSION,
            bridget_transport::protocol::ServiceRequestOperation::Delegate,
            bridget_transport::protocol::ServiceRequestPayload::Delegate {
                review_target: None,
                origin,
                focus,
                ..
            },
        ) => origin.is_some() || focus.is_some(),
        _ => false,
    };
    if !identifier(request_id) || !version_matches_payload {
        return false;
    }
    match (operation, payload) {
        (
            bridget_transport::protocol::ServiceRequestOperation::DeliveryReport,
            bridget_transport::protocol::ServiceRequestPayload::DeliveryReport {
                objective_id,
                delegation_id,
                delivery_hash,
                in_reply_to,
                review_verdict,
            },
        ) => {
            identifier(objective_id)
                && identifier(delegation_id)
                && identifier(in_reply_to)
                && delivery_hash.len() == 64
                && delivery_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
                && review_verdict
                    .as_ref()
                    .is_none_or(|evidence| evidence.is_valid())
        }
        (
            bridget_transport::protocol::ServiceRequestOperation::MissionStatus
            | bridget_transport::protocol::ServiceRequestOperation::DeadlineQuestion,
            bridget_transport::protocol::ServiceRequestPayload::Delegation { delegation_id },
        ) => identifier(delegation_id),
        (
            bridget_transport::protocol::ServiceRequestOperation::Delegate,
            bridget_transport::protocol::ServiceRequestPayload::Delegate {
                goal,
                review_target,
                explicit_target,
                required_tags,
                duration: _,
                suite,
                depends_on,
                references,
                origin,
                focus,
            },
        ) => {
            // SPEC-087 : forme seulement ; la vérité de l'origine est rejouée par
            // le service compagnon, et son droit d'être là est tranché par le dispatch.
            let origin_is_valid = origin.as_ref().is_none_or(|origin| match origin {
                bridget_transport::protocol::DelegateOrigin::Human {
                    message_id,
                    observed,
                    attestation,
                } => {
                    identifier(message_id)
                        && observed.message_id == *message_id
                        && !observed.sender.is_empty()
                        && !observed.target.is_empty()
                        && attestation.version == 1
                        && attestation.canonical_request_sha256.len() == 64
                        && attestation.signature.len() == 64
                }
            });
            let focus_is_valid = focus
                .as_ref()
                .is_none_or(|focus| identifier(&focus.project_id));
            let suite_is_valid = match suite {
                bridget_transport::protocol::ServiceSuiteDeclaration::Aucune => true,
                bridget_transport::protocol::ServiceSuiteDeclaration::Objectif { objective_id } => {
                    identifier(objective_id)
                }
            };
            let mut relations = std::collections::BTreeSet::new();
            let relations_are_valid = depends_on
                .iter()
                .chain(references)
                .all(|value| identifier(value) && relations.insert(value.as_str()));
            !goal.trim().is_empty()
                && goal.len() <= 16 * 1024
                && review_target
                    .as_ref()
                    .is_none_or(|target| target.is_valid())
                && explicit_target
                    .as_ref()
                    .is_none_or(|target| identifier(target))
                && required_tags.len() <= 32
                && required_tags.iter().all(|tag| identifier(tag))
                && required_tags
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == required_tags.len()
                && depends_on.len() <= 100
                && references.len() <= 100
                && suite_is_valid
                && relations_are_valid
                && origin_is_valid
                && focus_is_valid
        }
        (
            bridget_transport::protocol::ServiceRequestOperation::RegistreAdd,
            bridget_transport::protocol::ServiceRequestPayload::RegistreAdd { line },
        ) => !line.trim().is_empty() && line.len() <= 32 * 1024,
        (
            bridget_transport::protocol::ServiceRequestOperation::ObjectiveClose,
            bridget_transport::protocol::ServiceRequestPayload::ObjectiveClose {
                objective_id,
                reason,
            },
        ) => identifier(objective_id) && !reason.trim().is_empty() && reason.len() <= 4096,
        _ => false,
    }
}

fn greffe_mutation_action(
    operation: bridget_transport::protocol::ServiceRequestOperation,
) -> Option<GreffeMutationAction> {
    match operation {
        bridget_transport::protocol::ServiceRequestOperation::Delegate => {
            Some(GreffeMutationAction::Delegate)
        }
        bridget_transport::protocol::ServiceRequestOperation::RegistreAdd => {
            Some(GreffeMutationAction::RegistreAdd)
        }
        bridget_transport::protocol::ServiceRequestOperation::ObjectiveClose => {
            Some(GreffeMutationAction::ObjectiveClose)
        }
        bridget_transport::protocol::ServiceRequestOperation::DeliveryReport
        | bridget_transport::protocol::ServiceRequestOperation::MissionStatus
        | bridget_transport::protocol::ServiceRequestOperation::DeadlineQuestion => None,
    }
}

fn guichet_reply_is_valid(
    request_id: &str,
    response_message_id: &str,
    in_reply_to: &str,
    payload: &bridget_transport::protocol::GuichetReplyPayload,
) -> bool {
    let identifier = |value: &str| {
        !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
    };
    if !identifier(request_id) || !identifier(response_message_id) || !identifier(in_reply_to) {
        return false;
    }
    match payload {
        bridget_transport::protocol::GuichetReplyPayload::DeliveryReport {
            objective_id,
            delegation_id,
            delivery_hash,
            review_verdict,
        } => {
            identifier(objective_id)
                && identifier(delegation_id)
                && delivery_hash.len() == 64
                && delivery_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
                && review_verdict
                    .as_ref()
                    .is_none_or(|evidence| evidence.is_valid())
        }
        bridget_transport::protocol::GuichetReplyPayload::MissionStatus {
            delegation_id,
            objective_id,
            transport_observation,
            ..
        } => {
            identifier(delegation_id)
                && identifier(objective_id)
                && transport_observation.as_ref().is_none_or(|observation| {
                    !observation.source.is_empty()
                        && observation.source.len() <= 128
                        && observation
                            .subscription_id
                            .as_deref()
                            .is_none_or(identifier)
                        && observation.request_state.as_deref().is_none_or(identifier)
                })
        }
        bridget_transport::protocol::GuichetReplyPayload::DeadlineQuestion {
            delegation_id,
            ..
        } => identifier(delegation_id),
        bridget_transport::protocol::GuichetReplyPayload::Delegate {
            status,
            objective_id,
            delegation_id,
            message_id,
            participant,
            candidates,
            waiting_on_prerequisites,
            ..
        } => match status {
            bridget_transport::protocol::GuichetDelegateMutationStatus::Created => {
                objective_id.as_deref().is_some_and(identifier)
                    && delegation_id.as_deref().is_some_and(identifier)
                    && message_id.as_deref().is_none_or(identifier)
                    && participant.as_deref().is_some_and(identifier)
                    && candidates.is_empty()
                    && !(*waiting_on_prerequisites && message_id.is_some())
            }
            bridget_transport::protocol::GuichetDelegateMutationStatus::WaitingForAgent => {
                objective_id.as_deref().is_some_and(identifier)
                    && delegation_id.is_none()
                    && message_id.is_none()
                    && participant.is_none()
                    && candidates.is_empty()
                    && *waiting_on_prerequisites
            }
            bridget_transport::protocol::GuichetDelegateMutationStatus::SelectionRequired => {
                objective_id.is_none()
                    && delegation_id.is_none()
                    && message_id.is_none()
                    && participant.is_none()
                    && !*waiting_on_prerequisites
                    && candidates.len() <= 128
                    && candidates.iter().all(|value| identifier(value))
                    && candidates
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == candidates.len()
            }
        },
        bridget_transport::protocol::GuichetReplyPayload::RegistreAdd { constat_id, .. } => {
            identifier(constat_id)
        }
        bridget_transport::protocol::GuichetReplyPayload::ObjectiveClose {
            objective_id,
            decision_id,
            ..
        } => identifier(objective_id) && identifier(decision_id),
        bridget_transport::protocol::GuichetReplyPayload::Refused { .. } => true,
    }
}

fn guichet_result_response(
    issuer_scope: String,
    request_id: String,
    result: GuichetResult,
) -> DaemonToWrapper {
    let (issue, expires_at, payload) = match result {
        GuichetResult::Queued { expires_at } => ("queued".to_string(), expires_at, None),
        GuichetResult::OutcomeUnknown { expires_at } => {
            ("outcome_unknown".to_string(), expires_at, None)
        }
        GuichetResult::Terminal {
            issue,
            expires_at,
            reply_bytes,
            ..
        } => {
            let Ok(reply) = std::str::from_utf8(&reply_bytes)
                .ok()
                .and_then(|line| decode(line).ok())
                .ok_or(())
            else {
                return DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::TransitionInvalid,
                };
            };
            let WrapperToDaemon::GuichetReply { payload, .. } = reply else {
                return DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::TransitionInvalid,
                };
            };
            (issue, expires_at, Some(payload))
        }
        GuichetResult::CanonicalBytesMismatch => ("canonical_bytes_mismatch".to_string(), 0, None),
        GuichetResult::IdempotencyExpired => ("idempotency_expired".to_string(), 0, None),
        GuichetResult::InvalidIssuedAt => ("invalid_issued_at".to_string(), 0, None),
        GuichetResult::ClaimStale => ("claim_stale".to_string(), 0, None),
    };
    DaemonToWrapper::GuichetResult {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope,
        request_id,
        issue,
        expires_at,
        payload,
    }
}

fn guichet_claim_response(claim: crate::store::GuichetClaim) -> DaemonToWrapper {
    DaemonToWrapper::GuichetClaimed {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: claim.issuer_scope,
        request_id: claim.request_id,
        canonical_request: claim.canonical_request,
        authorization_attestation: claim.authorization_attestation,
        claimed_at: claim.claimed_at,
        claim_generation: claim.claim_generation,
        claim_token: claim.claim_token,
        claim_lease_expires_at: claim.claim_lease_expires_at,
        expires_at: claim.expires_at,
    }
}

fn guichet_lifecycle_response(event: GuichetLifecycleEvent) -> DaemonToWrapper {
    DaemonToWrapper::RequestLifecycleEvent {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: event.issuer_scope,
        event_id: event.event_id,
        request_id: event.request_id,
        state: event.state,
        observed_at: event.observed_at,
        in_reply_to: event.in_reply_to,
        response_message_id: event.response_message_id,
    }
}

fn guichet_coordination_response(event: GuichetCoordinationEvent, version: u16) -> DaemonToWrapper {
    DaemonToWrapper::CoordinationEvent {
        version,
        event_id: event.event_id,
        request_id: event.request_id,
        kind: event.kind,
        reminder_message_id: event.reminder_message_id,
        recipient: event.recipient,
        generation: event.generation,
        observed_at: event.observed_at,
        cursor: (version == COORDINATION_STREAM_VERSION).then_some(event.cursor),
    }
}

/// Atteste le rappel seulement après l'écriture et le flush effectifs vers son
/// destinataire. B (persistance le service compagnon) relève ce fait pour l'observation,
/// C (reconcile) le déduplique par `event_id`; aucun consommateur ne calcule
/// l'instant ni la génération à la place du transport.
fn attest_reminder_sent(
    state: &Arc<Mutex<DaemonState>>,
    request_id: &str,
    reminder_message_id: &str,
    recipient: &str,
    generation: u64,
) {
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("before_coordination_persist");
    let controls = {
        let mut st = state.lock().recover_poison();
        let event = match st.store.record_reminder_sent(
            request_id,
            reminder_message_id,
            recipient,
            generation,
            unix_timestamp(),
        ) {
            Ok(event) => event,
            Err(error) => {
                error!("persistance du rappel de coordination: {error}");
                return;
            }
        };
        #[cfg(feature = "test-support")]
        crate::test_sync::checkpoint("after_coordination_persist");
        let mut controls = Vec::new();
        let negotiations = st
            .service_negotiations
            .iter()
            .map(|(connection_id, negotiated)| (connection_id.clone(), negotiated.clone()))
            .collect::<Vec<_>>();
        for (connection_id, negotiated) in negotiations {
            let version = if negotiated
                .capabilities
                .contains(&ServiceCapability::CoordinationEventsV1)
            {
                Some(COORDINATION_EVENTS_VERSION)
            } else if negotiated
                .capabilities
                .contains(&ServiceCapability::CoordinationEventsV2)
                && st
                    .coordination_subscriptions
                    .get(&connection_id)
                    .is_some_and(|subscription| {
                        subscription.fresh && subscription.next_cursor == event.cursor
                    })
            {
                Some(COORDINATION_STREAM_VERSION)
            } else {
                None
            };
            let Some(version) = version else {
                continue;
            };
            if version == COORDINATION_STREAM_VERSION
                && let Some(subscription) = st.coordination_subscriptions.get_mut(&connection_id)
            {
                subscription.next_cursor = event.cursor.saturating_add(1);
            }
            if let Some(writer) = st.connections.get(&connection_id) {
                controls.push(DeferredControl {
                    writer: writer.clone(),
                    message: guichet_coordination_response(event.clone(), version),
                });
            }
        }
        controls
    };
    let _ = execute_controls(controls);
}

/// Traite l'enregistrement d'un wrapper
///
/// Les champs du message `Register` restent dépliés ici pour refléter le
/// protocole de transport ; les regrouper imposerait un refactor hors scope.
fn definition_runtime(definition: &ResolvedAgentDefinition) -> Option<(String, Option<String>)> {
    crate::registry::runtime_model_and_effort(&definition.args)
}

/// Le mode d'un équipier géré vient de sa définition figée, pas de son type
/// de fournisseur ni de la version du wrapper qui se réenregistre.
fn definition_presence_mode(definition: &ResolvedAgentDefinition) -> Option<PresenceMode> {
    match definition.protocol.as_str() {
        "acp" => Some(PresenceMode::Acp),
        "claude_stream_json" => Some(PresenceMode::Cli),
        "codex_app_server" => Some(PresenceMode::Cli),
        "tmux" => Some(PresenceMode::Tmux),
        _ => None,
    }
}

/// Transport et mode affichés pour un géré : toujours `definition.protocol`,
/// jamais un canal inventé. Unique dérivation, partagée par `handle_register`
/// et la projection `recovering` de `who`.
fn definition_presence_fields(
    definition: &ResolvedAgentDefinition,
) -> (String, Option<PresenceMode>) {
    (
        definition.protocol.clone(),
        definition_presence_mode(definition),
    )
}

/// Valeurs historiques qui décrivent un tuyau plutôt qu'un protocole
/// d'agent. Elles ne doivent jamais réapparaître dans `AgentInfo.transport`.
fn legacy_non_protocol_transport(value: &str) -> bool {
    matches!(value, "unix" | "ssh" | "ssh-unix" | "stdio")
}

fn non_empty_registration_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Définition figée d'un géré déjà connu. L'instance courante est la clé
/// nominale ; une ré-inscription (reprise, wrapper antérieur) peut arriver
/// avec un autre identifiant tout en portant le même nom de lease.
fn managed_definition_for_register(
    state: &DaemonState,
    instance_id: &str,
    agent_id: &str,
) -> Option<ResolvedAgentDefinition> {
    state
        .managed_by_instance
        .get(instance_id)
        .and_then(|command_id| state.fleet.resolved_definition_for_command(command_id))
        .or_else(|| {
            state.managed_spawns.values().find_map(|record| {
                (record.lease.name == agent_id)
                    .then(|| {
                        state
                            .fleet
                            .resolved_definition_for_command(&record.lease.command_id)
                    })
                    .flatten()
            })
        })
}

fn ensure_agent_profile_for_registration(state: &DaemonState, agent_id: &str) {
    let result = crate::agent_profile::AgentProfileStore::open(&state.db_path)
        .and_then(|mut profiles| profiles.ensure_agent_ids([agent_id]));
    if let Err(error) = result {
        warn!("profil de l'agent {agent_id} non créé : {error}");
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_register_with_channel(
    conn_id: &str,
    identity_version: u8,
    agent_type: String,
    agent_id: String,
    host: Option<String>,
    transport: Option<String>,
    channel: ChannelReport,
    mode: Option<PresenceMode>,
    location: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
    turn_in_progress: bool,
    journal_available: Option<bool>,
    state: &mut DaemonState,
    turn_attest: bool,
) -> DaemonToWrapper {
    log::debug!(
        "Register reçu de {}: version_identité={}, type={}, agent_id={}, host={:?}",
        conn_id,
        identity_version,
        agent_type,
        agent_id,
        host
    );

    if identity_version != 2 {
        return DaemonToWrapper::Nack {
            id: "register".to_string(),
            reason: "contrat d’identité v2 requis".to_string(),
        };
    }

    if agent_type == "mcp" {
        return DaemonToWrapper::Nack {
            id: "register".into(),
            reason:
                "auxiliary_credential_required : mettez à jour le wrapper et le client auxiliaire"
                    .into(),
        };
    }
    // La sonde d'inventaire n'est pas un équipier. Sa route de réponse est
    // éphémère et ne doit pas créer une identité/un profil à chaque lecture.
    // Aucun droit supplémentaire : l'autorisation des mutations reste celle
    // de la connexion, et une sonde ne revendique aucune instance fournisseur.
    let ephemeral_status_probe =
        agent_type == "status-probe" && mode == Some(PresenceMode::Cli) && instance_id.is_none();
    let parsed_type = agent_type
        .parse()
        .unwrap_or(bridget_core::AgentType::Custom(agent_type.clone()));

    // Le relais UI singleton reprend sa route humaine dédiée après un
    // redémarrage, puis les routes fantômes ordinaires suivent leur garde
    // habituelle. Aucun autre type d'agent ne peut emprunter ce chemin.
    state.replace_stale_ui_human_route(conn_id, &agent_type, &agent_id);
    // Takeover sans stop : une route UUID fantôme est libérée avant la réinscription.
    let _ = state.reclaim_phantom_name(&agent_id);

    match state.router.register(&agent_id, &parsed_type, conn_id) {
        Ok(()) => {
            let final_agent_id = agent_id.clone();
            if !ephemeral_status_probe {
                ensure_agent_profile_for_registration(state, &final_agent_id);
            }
            state
                .conn_names
                .insert(conn_id.to_string(), final_agent_id.clone());
            state.conn_hosts.insert(
                conn_id.to_string(),
                // Sentinelle PARTAGEE : ce champ est relu par les gardes de
                // localite, qui doivent pouvoir la reconnaitre.
                host.clone()
                    .unwrap_or_else(|| bridget_core::HOTE_NON_ATTESTE.to_string()),
            );
            state.conn_operating_systems.insert(
                conn_id.to_string(),
                os.clone().unwrap_or_else(|| "inconnu".to_string()),
            );

            if let Some(instance_id) = instance_id.filter(|id| !id.is_empty()) {
                let presence_owned_by_live_connection =
                    state
                        .conn_instances
                        .iter()
                        .any(|(existing_conn, existing_instance)| {
                            existing_conn != conn_id
                                && !state.auxiliary_connections.contains(existing_conn)
                                && existing_instance == &instance_id
                                && state.presences.get(&instance_id).is_some_and(|presence| {
                                    matches!(presence.state.as_str(), "connected" | "busy")
                                })
                        });
                if presence_owned_by_live_connection {
                    let same_equipier = state
                        .presences
                        .get(&instance_id)
                        .is_some_and(|presence| presence.name == final_agent_id);
                    if !same_equipier {
                        // Une connexion auxiliaire issue de la filiation MCP peut
                        // revendiquer la même instance que le wrapper. Elle garde
                        // son entrée de routage éphémère, mais ne devient jamais
                        // propriétaire de la présence : sa fermeture ne doit pas
                        // rendre le wrapper inaccessible ni effacer ses faits.
                        info!(
                            "présence {} conservée : connexion auxiliaire {} ignorée",
                            instance_id, conn_id
                        );
                        state.restore_pending_for_agent(&final_agent_id, conn_id);
                        return DaemonToWrapper::Registered {
                            agent_id: final_agent_id,
                            credential: None,
                        };
                    }
                    // Réconnexion du même équipier avant l'EOF de l'ancienne
                    // connexion : voler l'instance pour que mark_unreachable
                    // retardé ne puisse plus écraser busy / connected.
                    let old_owners: Vec<_> = state
                        .conn_instances
                        .iter()
                        .filter(|(conn, owned)| {
                            *conn != conn_id
                                && *owned == &instance_id
                                && !state.auxiliary_connections.contains(*conn)
                        })
                        .map(|(conn, _)| conn.clone())
                        .collect();
                    for owner in old_owners {
                        state.revoke_identity_authorizations(&owner);
                    }
                    let auxiliary_connections = &state.auxiliary_connections;
                    state
                        .conn_instances
                        .retain(|existing_conn, existing_instance| {
                            existing_conn == conn_id
                                || auxiliary_connections.contains(existing_conn)
                                || existing_instance.as_str() != instance_id
                        });
                }

                let previous_instance_key = if state.presences.contains_key(&instance_id) {
                    Some(instance_id.clone())
                } else {
                    state
                        .presences
                        .iter()
                        .find(|(_, presence)| presence.name == final_agent_id)
                        .map(|(key, _)| key.clone())
                };
                let previous = previous_instance_key
                    .as_ref()
                    .and_then(|key| state.presences.get(key).cloned());
                let reconnect_count = previous
                    .as_ref()
                    .map(|presence| {
                        presence.reconnect_count + u32::from(presence.state != "connected")
                    })
                    .unwrap_or(0);
                let managed_definition =
                    managed_definition_for_register(state, &instance_id, &final_agent_id);
                let managed_runtime = managed_definition
                    .as_ref()
                    .and_then(definition_runtime)
                    .map(|(model, effort)| (Some(model), effort));
                let (managed_transport, managed_mode) = match managed_definition.as_ref() {
                    Some(definition) => {
                        let (transport, mode) = definition_presence_fields(definition);
                        (Some(transport), mode)
                    }
                    None => (None, None),
                };
                // Un géré tient son runtime de la définition figée. Une
                // reconnexion interactive conserve, elle, la dernière sonde.
                let (model, effort) = managed_runtime.unwrap_or_else(|| {
                    previous
                        .as_ref()
                        .map(|presence| (presence.model.clone(), presence.effort.clone()))
                        .unwrap_or((None, None))
                });
                let derived_domain = previous
                    .as_ref()
                    .and_then(|presence| {
                        presence
                            .derived_domain
                            .clone()
                            .or_else(|| presence.domain.clone())
                    })
                    .or_else(|| domain.clone());
                let dnd_until = previous.as_ref().and_then(|presence| presence.dnd_until);
                // Les faits de limite restent indexés par fenêtre à travers
                // une reconnexion ; une fenêtre absente demeure absente.
                let rate_limits = previous
                    .as_ref()
                    .map(|presence| presence.rate_limits.clone())
                    .unwrap_or_default();
                let served_model = previous
                    .as_ref()
                    .and_then(|presence| presence.served_model.clone());
                let disk_space = previous
                    .as_ref()
                    .and_then(|presence| presence.disk_space.clone());
                // Le mode annoncé par le wrapper prime : une identité déjà
                // connue peut changer de voie (session 097 : tmux → PTY). Seule
                // une reconnexion par un binaire antérieur au champ (mode
                // absent) conserve l'observation déjà attestée ; une présence
                // historique sans valeur reste volontairement inconnue.
                let mode = managed_mode
                    .or(mode)
                    .or_else(|| previous.as_ref().and_then(|presence| presence.mode));
                let location = match mode {
                    Some(PresenceMode::Tmux) => previous
                        .as_ref()
                        .and_then(|presence| presence.location.clone())
                        .or(location),
                    _ => None,
                };
                let host = previous
                    .as_ref()
                    .filter(|presence| bridget_core::host_is_attested(&presence.host))
                    .map(|presence| presence.host.clone())
                    .or(host)
                    .unwrap_or_else(|| "inconnu".to_string());
                let os = previous
                    .as_ref()
                    .filter(|presence| presence.os != "inconnu")
                    .map(|presence| presence.os.clone())
                    .or(os)
                    .unwrap_or_else(|| "inconnu".to_string());
                let reported_transport = non_empty_registration_value(transport);
                let channel = match channel {
                    ChannelReport::Known(value) => non_empty_registration_value(Some(value)),
                    ChannelReport::Unknown => None,
                    ChannelReport::Omitted => match mode {
                        Some(PresenceMode::Tmux) => {
                            reported_transport.clone().filter(|value| value != "tmux")
                        }
                        _ => reported_transport
                            .clone()
                            .filter(|value| legacy_non_protocol_transport(value)),
                    }
                    .or_else(|| {
                        previous
                            .as_ref()
                            .and_then(|presence| presence.channel.clone())
                    }),
                };
                let transport = managed_transport.unwrap_or_else(|| match mode {
                    Some(PresenceMode::Tmux) => "tmux".to_string(),
                    Some(PresenceMode::Acp) => reported_transport
                        .clone()
                        .filter(|value| !legacy_non_protocol_transport(value))
                        .or_else(|| previous.as_ref().map(|presence| presence.transport.clone()))
                        .unwrap_or_else(|| "acp".to_string()),
                    Some(PresenceMode::Cli) => reported_transport
                        .filter(|value| !legacy_non_protocol_transport(value))
                        .or_else(|| previous.as_ref().map(|presence| presence.transport.clone()))
                        .unwrap_or_else(|| "cli".to_string()),
                    _ => reported_transport
                        .filter(|value| !legacy_non_protocol_transport(value))
                        .or_else(|| previous.as_ref().map(|presence| presence.transport.clone()))
                        .unwrap_or_else(|| "unknown".to_string()),
                });
                let agent_type = previous
                    .as_ref()
                    .filter(|presence| !matches!(presence.agent_type.as_str(), "mcp" | "cli"))
                    .map(|presence| presence.agent_type.clone())
                    .unwrap_or_else(|| parsed_type.to_string());

                state
                    .conn_instances
                    .insert(conn_id.to_string(), instance_id.clone());
                if state.managed_by_instance.contains_key(&instance_id) {
                    let _ = state
                        .fleet
                        .set_desired_domain(&final_agent_id, derived_domain.as_deref());
                }
                // Les wrappers récents réinitialisent explicitement ce fait à
                // `false` puis annoncent `JournalReady`. Les binaires
                // historiques ne portent pas ce champ, mais leur chemin ACP
                // n'atteint la boucle qu'après activation du journal : les
                // conserver attachables évite de les casser lors d'un
                // redémarrage progressif du daemon.
                let journal_available = journal_available.unwrap_or_else(|| {
                    previous
                        .as_ref()
                        .is_some_and(|presence| presence.journal_available)
                        || transport == "acp"
                });
                // Migration des remises orphelines : un nouvel instance_id pour
                // le même nom reprend les `dispatching` de l'ancienne instance
                // avant de la retirer, sinon elles restent invisibles jusqu'à
                // expiration (canal latéral muet après respawn).
                if let Some(old_instance_id) = previous_instance_key
                    .as_ref()
                    .filter(|old| old.as_str() != instance_id.as_str())
                {
                    match state.idempotency.reassign_dispatching_deliveries(
                        old_instance_id,
                        &instance_id,
                        unix_now_secs(),
                    ) {
                        Ok(migrated) if migrated > 0 => info!(
                            "reprises idempotentes migrées : {} remise(s) {} → {}",
                            migrated, old_instance_id, instance_id
                        ),
                        Ok(_) => {}
                        Err(error) => error!(
                            "migration des remises {} → {} : {error}",
                            old_instance_id, instance_id
                        ),
                    }
                    state.presences.remove(old_instance_id);
                }
                let register_now = Instant::now();
                state.presences.insert(
                    instance_id.clone(),
                    Presence {
                        name: final_agent_id.clone(),
                        agent_type,
                        host,
                        transport,
                        channel,
                        mode,
                        location,
                        journal_available,
                        os,
                        state: if turn_in_progress {
                            "busy"
                        } else {
                            "connected"
                        }
                        .to_string(),
                        busy_since: if turn_in_progress {
                            Some(Instant::now())
                        } else {
                            None
                        },
                        busy_period_start: (turn_attest && turn_in_progress)
                            .then_some(register_now),
                        last_attested_at: turn_attest.then_some(register_now),
                        turn_attest,
                        queue_fallback: false,
                        capacity_seen: Instant::now(),
                        link_seen: Instant::now(),
                        reconnect_count,
                        model,
                        effort,
                        rate_limits,
                        served_model,
                        domain: derived_domain.clone(),
                        derived_domain,
                        dnd_until,
                        disk_space,
                    },
                );
                // L'identité finale et sa présence sont établies. L'adoption
                // ne dépend pas d'une ancienne présence après un redémarrage.
                let recover_dispatching =
                    state.presences.get(&instance_id).is_some_and(|presence| {
                        presence.turn_attest
                            && presence.transport != "t3code"
                            && state
                                .registry
                                .get(&presence.agent_type)
                                .ok()
                                .is_some_and(|definition| definition.held_queue)
                    });
                if let Err(error) = state.idempotency.adopt_held_for_agent(
                    &final_agent_id,
                    &instance_id,
                    recover_dispatching,
                ) {
                    warn!("rattachement des retenus refusé pour {final_agent_id}: {error}");
                    return DaemonToWrapper::Nack {
                        id: "register".into(),
                        reason: format!("rattachement des retenus indisponible: {error}"),
                    };
                }
                schedule_idempotent_delivery_recovery(state, conn_id, &instance_id);
                schedule_execution_recovery(
                    state,
                    conn_id,
                    &instance_id,
                    &final_agent_id,
                    turn_in_progress,
                );
            }

            state.restore_pending_for_agent(&final_agent_id, conn_id);
            let mut specialist_controls = Vec::new();
            specialist_requests::resume_for_registered(
                state,
                &final_agent_id,
                &mut specialist_controls,
            );
            // Une entrée vide ferait croire à une activité en attente
            // (contrôle de maintenance) jusqu'à la vidange après réponse.
            if !specialist_controls.is_empty() {
                state
                    .pending_post_response_controls
                    .entry(conn_id.into())
                    .or_default()
                    .extend(specialist_controls);
            }
            info!("agent '{}' enregistré ({})", final_agent_id, conn_id);
            state.revoke_identity_authorizations(conn_id);
            let credential = state.conn_instances.contains_key(conn_id).then(|| {
                let proof = bridget_transport::protocol::IdentityCredential::new(format!(
                    "{}{}",
                    uuid::Uuid::new_v4().simple(),
                    uuid::Uuid::new_v4().simple()
                ));
                state
                    .identity_credentials
                    .insert(conn_id.into(), proof.clone());
                proof
            });
            DaemonToWrapper::Registered {
                agent_id: final_agent_id,
                credential,
            }
        }
        Err(e) => {
            log::warn!("enregistrement refusé pour {}: {}", conn_id, e);
            DaemonToWrapper::Nack {
                id: "register".to_string(),
                reason: format!("enregistrement refusé: {}", e),
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
/// Simule les enregistrements antérieurs à FR-2407, donc sans canal explicite.
// 14 paramètres : regroupement différé (programme 260, lot 9).
fn handle_register(
    conn_id: &str,
    agent_type: String,
    agent_id: Option<String>,
    host: Option<String>,
    transport: Option<String>,
    mode: Option<PresenceMode>,
    location: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
    turn_in_progress: bool,
    journal_available: Option<bool>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    handle_register_with_channel(
        conn_id,
        2,
        agent_type,
        agent_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        host,
        transport,
        ChannelReport::Omitted,
        mode,
        location,
        os,
        instance_id,
        domain,
        turn_in_progress,
        journal_available,
        state,
        false,
    )
}

/// Longueur maximale acceptée pour un identifiant de modèle ou un niveau
/// d'effort, alignée sur la validation des noms d'agent côté CLI.
const MAX_RUNTIME_VALUE_LENGTH: usize = 100;

/// Rejette une valeur trop longue ou porteuse de caractères de contrôle / format
/// (bidi), qui casserait l'alignement de l'annuaire ou mentirait à l'affichage.
fn validate_runtime_value(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("valeur vide".to_string());
    }
    if value.chars().count() > MAX_RUNTIME_VALUE_LENGTH {
        return Err(format!(
            "valeur trop longue (max {} caractères)",
            MAX_RUNTIME_VALUE_LENGTH
        ));
    }
    if value.chars().any(bridget_core::is_disallowed_control) {
        return Err("valeur contenant des caractères de contrôle".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod validate_runtime_value_tests {
    use super::validate_runtime_value;

    /// Contrôle positif Cc ; puis bidi réel. Aveuglement Cc prouvé sans appeler
    /// le prédicat sous test pour construire l'attente.
    #[test]
    fn oracle_runtime_refuse_une_valeur_bidi_reelle() {
        assert!(
            validate_runtime_value("claude\u{0007}").is_err(),
            "PROMESSE — un Cc (BEL) DOIT être refusé"
        );
        assert!(
            validate_runtime_value("claude-opus").is_ok(),
            "PROMESSE — une valeur légitime DOIT passer"
        );
        let bidi = "claude\u{202e}edualc";
        assert!(
            !bidi.chars().any(char::is_control),
            "preuve d'aveuglement Cc : sans is_format_character cette chaîne passerait"
        );
        let err = validate_runtime_value(bidi).expect_err("oracle — U+202E doit être refusé");
        assert!(
            err.contains("contrôle"),
            "motif de refus attendu, reçu: {err}"
        );
        assert!(
            validate_runtime_value("high\u{2069}").is_err(),
            "oracle — isolat U+2069 DOIT être refusé"
        );
    }
}

/// Applique une observation de runtime à la présence d'un agent nommé.
///
/// L'agent est désigné par son nom et non par la connexion émettrice : le hook
/// Claude et `bridget runtime` transitent par le client CLI, dont la connexion
/// est éphémère. Même résolution que `Rename` et `CancelRequest`.
///
/// Le couple `(model, effort)` remplace l'état courant **en bloc** : un effort
/// absent efface l'effort connu, parce qu'il décrit une observation réelle et
/// non une lacune. Sans cela, un agent passant d'un modèle qui expose l'effort
/// à un modèle qui ne l'expose pas conserverait indéfiniment l'ancienne valeur.
///
/// Ce message ne traverse ni le routeur ni le disjoncteur : ce n'est pas du
/// trafic entre agents, il ne peut ni être routé ni boucler.
fn handle_runtime(
    agent: &str,
    model: String,
    effort: Option<String>,
    source: bridget_transport::protocol::RuntimeSource,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    handle_runtime_for_instance(agent, model, effort, source, None, state)
}

fn handle_runtime_for_instance(
    agent: &str,
    model: String,
    effort: Option<String>,
    source: bridget_transport::protocol::RuntimeSource,
    instance_id: Option<&str>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    if let Err(reason) = validate_runtime_value(&model) {
        return DaemonToWrapper::Nack {
            id: "runtime".to_string(),
            reason: format!("modèle invalide: {}", reason),
        };
    }
    if let Some(Err(reason)) = effort.as_deref().map(validate_runtime_value) {
        return DaemonToWrapper::Nack {
            id: "runtime".to_string(),
            reason: format!("effort invalide: {}", reason),
        };
    }

    let presence = match instance_id {
        Some(instance_id) => state
            .presences
            .get_mut(instance_id)
            .filter(|presence| presence.name == agent),
        None => presence_of_agent(state, agent),
    };
    let Some(presence) = presence else {
        return DaemonToWrapper::Nack {
            id: "runtime".to_string(),
            reason: format!("agent introuvable: {}", agent),
        };
    };

    let unchanged = presence.model.as_deref() == Some(model.as_str())
        && presence.effort.as_deref() == effort.as_deref();
    if !unchanged {
        log::debug!(
            "runtime de '{}' mis à jour par {} : modèle={} effort={:?}",
            presence.name,
            source,
            model,
            effort
        );
        presence.model = Some(model);
        presence.effort = effort;
    }
    presence.touch_capacity();

    DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: "runtime".to_string(),
    }
}

/// Enregistre le modèle servi par le flux, sans toucher au modèle épinglé ni
/// au routage. Un flux muet n'appelle jamais cette fonction : l'écart reste
/// alors non-verdict.
fn handle_served_model(agent: &str, model: String, state: &mut DaemonState) -> DaemonToWrapper {
    if let Err(reason) = validate_runtime_value(&model) {
        return DaemonToWrapper::Nack {
            id: "served-model".to_string(),
            reason: format!("modèle servi invalide: {reason}"),
        };
    }
    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "served-model".to_string(),
            reason: format!("agent introuvable: {agent}"),
        };
    };
    presence.served_model = Some(model);
    presence.touch_capacity();
    DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: "served-model".to_string(),
    }
}

/// Enregistre un fait de limite fournisseur sans changer l'état de l'agent.
/// Les champs sont validés comme les valeurs runtime : ils resteront affichés
/// dans `who`, donc aucun contrôle ni valeur démesurée ne traverse la frontière.
/// Mémoire partagée : coût ajouté O(log t+B), t types du registre, B chaînes copiées,
/// avec 8 fenêtres au plus ; aucune I/O et aucun appel fournisseur.
fn handle_rate_limit(
    agent: &str,
    window: String,
    status: String,
    resets_at: Option<i64>,
    used_percent: Option<u8>,
    credits: Option<bridget_transport::protocol::CodexCredits>,
    source: bridget_transport::protocol::RateLimitSource,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    if let Err(reason) = validate_runtime_value(&window) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: format!("fenêtre de limite invalide: {reason}"),
        };
    }
    if let Err(reason) = validate_runtime_value(&status) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: format!("statut de limite invalide: {reason}"),
        };
    }
    if resets_at.is_some_and(|timestamp| timestamp <= 0) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: "instant de retour invalide".to_string(),
        };
    }
    if used_percent.is_some_and(|pct| pct > 100) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: "pourcentage de limite invalide".to_string(),
        };
    }
    let instance = state
        .router
        .get_agent(agent)
        .and_then(|found| state.conn_instances.get(&found.connection_id))
        .cloned();
    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: format!("agent introuvable: {agent}"),
        };
    };
    // Upsert par fenêtre : un fait five_hour ne doit pas effacer seven_day.
    let agent_type = presence.agent_type.clone();
    let fact = bridget_transport::protocol::RateLimitFact {
        window: window.clone(),
        status,
        resets_at,
        used_percent,
        credits: credits
            .filter(|_| source == bridget_transport::protocol::RateLimitSource::CodexAppServer),
    };
    presence.rate_limits.insert(window, fact.clone());
    presence.touch_capacity();
    log::debug!("limite de '{}' mise à jour par {}", presence.name, source);
    // Dernier usage de presence ci-dessus : la mémoire partagée ne garde pas son emprunt.
    if let Some(instance) = instance {
        state.provider_limits.record_scoped(
            &agent_type,
            &instance,
            fact.clone(),
            source,
            unix_timestamp(),
        );
        // Le runtime doit réattester la définition exacte du lanceur. Une
        // observation sans cette preuve reste disponible au chemin legacy,
        // mais ne devient pas un fait de compte pour le routage168.
        let binding = state.registry.routing_policy().and_then(|policy| {
            // O(T+C), T<=4096 missions conservées, C<=128 connexions déclarées.
            if let Some(task) = state.delegation_store.for_child(agent).ok().flatten()
                && let Some(placement) = task.remote_placement.as_ref()
            {
                if !task.remote_prepared
                    || task.child_instance.as_deref() != Some(instance.as_str())
                    || task.definition.digest != placement.definition_digest
                    || state.managed_by_instance.get(&instance) != Some(&task.task_id)
                {
                    return None;
                }
                let mut matches = policy.connections.iter().filter(|connection| {
                    connection.enabled
                        && connection.agent_type == agent_type
                        && connection.machine_id == placement.machine_id
                });
                let connection = matches.next()?;
                if matches.next().is_some() {
                    return None;
                }
                return Some((
                    connection.connection_id.clone(),
                    placement.definition_digest.clone(),
                ));
            }
            let mut matches = policy.connections.iter().filter(|connection| {
                connection.enabled
                    && connection.agent_type == agent_type
                    && (connection.machine_id == "local" || connection.machine_id == state.host)
            });
            let connection = matches.next()?;
            if matches.next().is_some() {
                return None;
            }
            let declared = state.registry.resolved_definition(&agent_type).ok()?;
            let runtime = state
                .idempotency
                .latest_connected_spawn_by_name(agent)
                .ok()??;
            if runtime.resolved_definition != declared
                || state.managed_by_instance.get(&instance) != Some(&runtime.command_id)
            {
                return None;
            }
            Some((connection.connection_id.clone(), declared.digest))
        });
        if let Some((connection_id, digest)) = binding {
            if let Some(account) = state.registry.routing_account_scope244(&connection_id) {
                state.provider_limits.record_account244(
                    state.store.connection(),
                    &account,
                    &fact,
                    source,
                    unix_timestamp(),
                );
            }
            if let Some(scope) = state.registry.routing_connection_scope(&connection_id) {
                state.provider_limits.bind_routing_instance(
                    &connection_id,
                    &agent_type,
                    &instance,
                    &digest,
                    state.coordinator_epoch,
                    &scope,
                );
            }
        }
    }
    DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: "rate-limit".to_string(),
    }
}

/// Enregistre un échantillon de consommation dans le ledger horodaté.
/// L'agent doit être présent (géré vivant) ; sans source côté pilote, aucun
/// message n'arrive ici — le greffe rendra « inconnu », jamais zéro.
fn handle_usage(
    agent: &str,
    tokens: bridget_transport::protocol::UsageTokens,
    source: bridget_transport::protocol::UsageSource,
    provider_kind: Option<String>,
    execution_id: Option<String>,
    execution_generation: Option<u64>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    let observed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    if observed_at <= 0 {
        return DaemonToWrapper::Nack {
            id: "usage".to_string(),
            reason: "horloge indisponible".to_string(),
        };
    }
    let Some((registered_provider, observed_model)) = presence_of_agent(state, agent)
        .map(|presence| (presence.agent_type.clone(), presence.model.clone()))
    else {
        return DaemonToWrapper::Nack {
            id: "usage".to_string(),
            reason: format!("agent introuvable: {agent}"),
        };
    };
    let provider_kind = provider_kind.unwrap_or(registered_provider);
    if let Err(error) = state.store.record_usage_sample(
        agent,
        observed_at,
        tokens,
        &source.to_string(),
        Some(provider_kind.as_str()),
        observed_model.as_deref(),
    ) {
        return DaemonToWrapper::Nack {
            id: "usage".to_string(),
            reason: format!("ledger usage: {error}"),
        };
    }
    match (execution_id, execution_generation) {
        (None, None) => {}
        (Some(execution_id), Some(generation)) => {
            let target = match state
                .execution_store
                .execution_control_target(&execution_id)
            {
                Ok(Some(target)) if target.target_agent == agent => target,
                Ok(_) => {
                    return DaemonToWrapper::Nack {
                        id: "usage".to_string(),
                        reason: "corrélation usage non autorisée".to_string(),
                    };
                }
                Err(error) => {
                    return DaemonToWrapper::Nack {
                        id: "usage".to_string(),
                        reason: format!("lecture exécution usage: {error}"),
                    };
                }
            };
            if target.snapshot.generation != generation {
                return DaemonToWrapper::Nack {
                    id: "usage".to_string(),
                    reason: "génération usage périmée".to_string(),
                };
            }
            let sample = ExecutionUsageSample {
                execution_id,
                generation,
                tokens,
                source: source.to_string(),
                observed_at,
            };
            match state.execution_store.record_execution_usage(&sample) {
                Ok(true) => {}
                Ok(false) => {
                    return DaemonToWrapper::Nack {
                        id: "usage".to_string(),
                        reason: "échantillon usage non attribuable".to_string(),
                    };
                }
                Err(error) => {
                    return DaemonToWrapper::Nack {
                        id: "usage".to_string(),
                        reason: format!("budget usage: {error}"),
                    };
                }
            }
        }
        _ => {
            return DaemonToWrapper::Nack {
                id: "usage".to_string(),
                reason: "corrélation usage incomplète".to_string(),
            };
        }
    }
    if let Some(presence) = presence_of_agent(state, agent) {
        presence.touch_capacity();
        log::debug!("usage de {} enregistré par {}", presence.name, source);
    }
    DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: "usage".to_string(),
    }
}

fn handle_usage_window(
    agent: &str,
    from_secs: i64,
    to_secs: i64,
    state: &DaemonState,
) -> DaemonToWrapper {
    if agent.trim().is_empty() || from_secs <= 0 || to_secs < from_secs {
        return DaemonToWrapper::Nack {
            id: "usage-window".to_string(),
            reason: "fenêtre d'usage invalide".to_string(),
        };
    }
    match state
        .store
        .aggregate_usage_window(agent, from_secs, to_secs)
    {
        Ok(aggregate) => DaemonToWrapper::UsageWindowResult {
            agent: agent.to_string(),
            aggregate,
        },
        Err(error) => DaemonToWrapper::Nack {
            id: "usage-window".to_string(),
            reason: format!("lecture usage: {error}"),
        },
    }
}

/// Décide si un rappel d'escalade doit être délivré.
///
/// Un destinataire qui refuse d'être dérangé ne reçoit ni le rappel discret ni
/// le rappel ferme : respecter le statut à l'aller pour le violer au rappel
/// n'aurait aucun sens. La demande reste ouverte et son échéance court toujours.
///
/// Le palier 2 et au-delà correspond à la notification d'échec adressée à
/// l'**émetteur** : elle ne dérange pas le destinataire et part donc toujours.
fn should_remind(target_is_undisturbed: bool, escalation_level: u8) -> bool {
    !target_is_undisturbed || escalation_level >= 2
}

fn deferred_reminder_level(
    target_is_busy: bool,
    elapsed_secs: u64,
    timeout_secs: u64,
) -> Option<u8> {
    if !target_is_busy {
        return None;
    }
    if elapsed_secs >= (timeout_secs * 2) / 3 {
        Some(2)
    } else if elapsed_secs >= timeout_secs / 3 {
        Some(1)
    } else {
        None
    }
}

/// Applique une itération complète de la surveillance des demandes suivies.
/// Les écritures de socket restent dans la boucle du daemon, hors verrou ; ce
/// facteur ne produit que les actions et persiste les transitions associées.
fn collect_reminder_actions(state: &mut DaemonState, now: Instant) -> Vec<ReminderAction> {
    let mut actions = Vec::new();
    let held_agents: Vec<String> = state.presences.values().map(|p| p.name.clone()).collect();
    for agent in held_agents {
        schedule_held_release(state, &agent, now);
        // Une réservation déjà posée n'empêche pas la reprise d'une E/S différée.
        if let Some(route) = state.router.get_agent(&agent) {
            let conn = route.connection_id.clone();
            if live_connection_identity(state, &conn).is_some()
                && state
                    .pending_post_response_controls
                    .get(&conn)
                    .is_some_and(|v| !v.is_empty())
            {
                actions.push(ReminderAction::PushDeferred { target_conn: conn });
            }
        }
    }
    let mut state_updates = Vec::new();
    let mut timeout_candidates = Vec::new();
    let mut deferred_events = Vec::new();
    let t3_connections: HashSet<_> = state
        .conn_instances
        .iter()
        .filter(|(_, instance)| {
            state
                .presences
                .get(*instance)
                .is_some_and(|p| p.transport == "t3code")
        })
        .map(|(conn, _)| conn.clone())
        .collect();
    let dormant =
        crate::agent_tracking::dormant_ids(state.store.connection()).unwrap_or_else(|_| {
            // Une fiche illisible diffère son rappel ; les autres restent indépendantes.
            state
                .presences
                .values()
                .filter(|p| {
                    crate::agent_tracking::dormant(state.store.connection(), &p.name)
                        .unwrap_or(true)
                })
                .map(|p| p.name.clone())
                .collect()
        });
    let undisturbed: std::collections::HashSet<String> = state
        .presences
        .values()
        .filter(|presence| presence.is_dnd() || dormant.contains(&presence.name))
        .map(|presence| presence.name.clone())
        .collect();
    // SPEC-087 : en pause, les paliers 1 et 2 sont différés comme pour un
    // tour en cours ; le palier 3 (échec notifié à l'émetteur) reste émis,
    // c'est une information, pas un réveil.
    let paused = match crate::referent_control::read(state.store.connection()) {
        Ok(control) => matches!(
            crate::referent_control::admit_autonomous_effect(
                crate::referent_control::AutonomousEffect::ReminderNudge,
                &control,
            ),
            crate::referent_control::Admission::Deferred { .. }
        ),
        Err(error) => {
            warn!("état de contrôle illisible pour les relances: {error}");
            false
        }
    };
    let busy_connections: std::collections::HashSet<String> = state
        .conn_instances
        .iter()
        .filter(|(_, instance_id)| {
            state
                .presences
                .get(*instance_id)
                .is_some_and(|presence| presence.state == "busy")
        })
        .map(|(connection_id, _)| connection_id.clone())
        .collect();

    // Les humains sont reconnus par leur TYPE d'inscription à l'annuaire, pas
    // par leur nom : `agent_type == "ui"` est ce que pose `ui.rs` au moment de
    // l'enregistrement. Un humain renommé reste couvert ; un agent qui se
    // nommerait « humain » ne l'est pas.
    let mut humains: std::collections::HashSet<String> = state
        .presences
        .values()
        .filter(|presence| presence.agent_type == "ui")
        .map(|presence| presence.name.clone())
        .collect();
    // Le ledger garde volontairement le libellé stable `humain`, alors que
    // l'inscription du relais UI utilise son UUID canonique. Les deux doivent
    // désigner la même extrémité pour le délai humain, sinon des relances
    // d'agent peuvent parasiter une réponse encore en cours.
    if humains.contains(UI_HUMAN_AGENT_ID) {
        humains.insert(UI_HUMAN_SENDER.to_string());
    }

    // O(R * (log t + log r + log N + P)). Les demandes ordinaires restent
    // inchangées ; une pause native couvre aussi le palier d'expiration.
    let frozen: HashMap<_, _> = state
        .pending_replies
        .iter()
        .map(|pending| {
            (
                pending.msg_id.clone(),
                native_delegation::request_frozen_secs(state, &pending.msg_id, unix_timestamp())
                    .and_then(|frozen| match frozen {
                        None => Ok(None),
                        Some(secs) => {
                            state
                                .delegation_store
                                .for_mission(&pending.msg_id)
                                .map(|task| {
                                    let quota_waiting = task.is_some_and(|task| {
                                        task.quota_hold.is_some()
                                            || task.quota_waits.last().is_some_and(|wait| {
                                                wait.phase
                                                    != crate::delegation::QuotaWaitPhase::Done
                                                    && unix_timestamp()
                                                        < wait.reset_at.saturating_add(30)
                                            })
                                    });
                                    Some((secs, quota_waiting))
                                })
                        }
                    }),
            )
        })
        .collect();
    let remote_result_pending: HashSet<String> = state
        .pending_replies
        .iter()
        .filter(|pending| {
            native_delegation::remote_result_suppresses_reminders(state, &pending.msg_id)
        })
        .map(|pending| pending.msg_id.clone())
        .collect();
    for pending in state.pending_replies.iter_mut() {
        // Un résultat distant accepté est déjà canonique, même si la clôture
        // SQL du guichet a échoué. Le tick natif réessaie cette clôture ; entre
        // temps, aucun rappel ni timeout ne doit être fabriqué.
        if remote_result_pending.contains(&pending.msg_id) {
            continue;
        }
        match frozen.get(&pending.msg_id) {
            Some(Ok(Some((secs, quota_waiting)))) => {
                pending.frozen_secs = *secs;
                if paused || *quota_waiting {
                    continue;
                }
            }
            Some(Err(error)) => {
                warn!("gel de suivi natif illisible: {error}");
                continue;
            }
            _ => {}
        }
        let elapsed = now
            .duration_since(pending.created_at)
            .as_secs()
            .saturating_sub(pending.frozen_secs);
        // Les deux extrémités comptent : `humain → agent` expirait aussi vite
        // que `agent → humain`, et les deux sens ont été mesurés le 28/08.
        let implique_humain = humains.contains(&pending.to) || humains.contains(&pending.from);
        // Le timeout effectif pilote AUSSI les paliers de relance (`timeout/3`,
        // `2*timeout/3`) : relever le plancher évite du même geste de rappeler
        // un humain toutes les vingt secondes.
        let timeout = effective_reply_timeout(pending.timeout_secs, implique_humain);
        if elapsed >= timeout {
            pending.escalation_level = 3;
            timeout_candidates.push((
                pending.to.clone(),
                pending.from.clone(),
                pending.msg_id.clone(),
                pending.from_conn.clone(),
                timeout,
                t3_connections
                    .contains(&pending.target_conn)
                    .then(|| pending.target_conn.clone()),
            ));
            continue;
        }
        // Le destinataire n'a pas reçu ce message ; seul le palier3 reste admis.
        if state.held_ids.contains(&pending.msg_id) {
            continue;
        }
        // Le pont corrèle la fin du tour automatiquement. Un rappel destiné
        // au CLI deviendrait ici un nouveau travail fournisseur indépendant,
        // capable de démarrer après l'expiration de la demande originale.
        if t3_connections.contains(&pending.target_conn) {
            continue;
        }
        if !should_remind(undisturbed.contains(&pending.to), pending.escalation_level) {
            continue;
        }
        if let Some(level) = deferred_reminder_level(
            paused || busy_connections.contains(&pending.target_conn),
            elapsed,
            timeout,
        ) {
            if pending.deferred_level != Some(level) {
                pending.deferred_level = Some(level);
                deferred_events.push((pending.msg_id.clone(), level));
                actions.push(ReminderAction::Deferred {
                    to: pending.to.clone(),
                    msg_id: pending.msg_id.clone(),
                    level,
                    reason: if paused { "pause" } else { "tour en cours" },
                });
            }
            continue;
        }
        if let Some(level) = pending.deferred_level.take() {
            pending.escalation_level = level;
            state_updates.push((pending.msg_id.clone(), level));
            actions.push(if level == 1 {
                ReminderAction::Gentle {
                    to: pending.to.clone(),
                    from: pending.from.clone(),
                    msg_id: pending.msg_id.clone(),
                    target_conn: pending.target_conn.clone(),
                }
            } else {
                ReminderAction::Firm {
                    to: pending.to.clone(),
                    from: pending.from.clone(),
                    msg_id: pending.msg_id.clone(),
                    target_conn: pending.target_conn.clone(),
                }
            });
            continue;
        }
        if pending.escalation_level == 0 && elapsed >= timeout / 3 {
            pending.escalation_level = 1;
            state_updates.push((pending.msg_id.clone(), 1));
            actions.push(ReminderAction::Gentle {
                to: pending.to.clone(),
                from: pending.from.clone(),
                msg_id: pending.msg_id.clone(),
                target_conn: pending.target_conn.clone(),
            });
        } else if pending.escalation_level == 1 && elapsed >= (timeout * 2) / 3 {
            pending.escalation_level = 2;
            state_updates.push((pending.msg_id.clone(), 2));
            actions.push(ReminderAction::Firm {
                to: pending.to.clone(),
                from: pending.from.clone(),
                msg_id: pending.msg_id.clone(),
                target_conn: pending.target_conn.clone(),
            });
        }
    }

    let held_ids = &state.held_ids;
    state.pending_replies.retain(|pending| {
        held_ids.contains(&pending.msg_id)
            || pending.escalation_level < 3
            || now.duration_since(pending.created_at).as_secs()
                < pending.timeout_secs + TIMEOUT_GRACE_PERIOD
    });
    for (id, level) in state_updates {
        let _ = state.store.set_escalation_level(&id, level);
    }
    for (id, level) in deferred_events {
        let _ = state.store.record_deferred_reminder(&id, level);
    }
    for (to, from, msg_id, from_conn, timeout_secs, cancel_target) in timeout_candidates {
        if claim_timeout(&mut state.store, &msg_id) {
            state.delegation_store.wait_epoch().bump();
            actions.push(ReminderAction::Timeout {
                to,
                from,
                msg_id,
                from_conn,
                timeout_secs,
                cancel_target,
            });
        }
    }
    // Le palier3 clôt d'abord la demande ; le même tick retire ensuite le retenu.
    match sweep_held(state, unix_now_secs()) {
        Ok(()) => {
            // Notice durable acquise seulement après COMMIT ; writer legacy sous State.
            state.flush_pending_orphan_emitter_notices();
        }
        Err(error) => warn!("balayage retenu différé après relances: {error}"),
    }
    actions
}

/// Retourne vrai pour le seul chemin autorisé à notifier l'émetteur d'une
/// échéance. SQLite arbitre l'intercalage transport ↔ thread de relance.
fn claim_timeout(store: &mut Store, id: &str) -> bool {
    store.mark_timed_out(id).unwrap_or(false)
}

/// Retrouve la présence d'un agent désigné par son nom.
///
/// Même résolution que `handle_runtime` : les commandes de contrôle arrivent par
/// le client CLI, dont la connexion est éphémère et distincte de celle de
/// l'agent visé.
fn presence_of_agent<'a>(state: &'a mut DaemonState, agent: &str) -> Option<&'a mut Presence> {
    let instance_id = state
        .router
        .get_agent(agent)
        .map(|found| found.connection_id.clone())
        .and_then(|conn| state.conn_instances.get(&conn).cloned())?;
    state.presences.get_mut(&instance_id)
}

/// Date l'activité d'un expéditeur uniquement lorsqu'il correspond à une
/// présence enregistrée. Un client éphémère peut transporter le message au nom
/// de l'agent : la connexion courante n'est donc pas l'autorité, le principal
/// logique du message l'est.
fn touch_message_sender_activity(state: &mut DaemonState, sender: &str) {
    if let Some(presence) = presence_of_agent(state, sender) {
        presence.touch_message_activity();
    }
}

/// Remplace le domaine d'un agent, ou le ramène à son domaine dérivé.
#[cfg(test)]
fn handle_domain(agent: &str, domain: Option<String>, state: &mut DaemonState) -> DaemonToWrapper {
    handle_domain_for_instance(agent, domain, None, state)
}

fn handle_domain_for_instance(
    agent: &str,
    domain: Option<String>,
    instance_id: Option<&str>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    if let Some(Err(reason)) = domain
        .as_deref()
        .map(bridget_core::router::validate_technical_label)
    {
        return DaemonToWrapper::Nack {
            id: "domain".to_string(),
            reason: format!("domaine invalide: {}", reason),
        };
    }
    let presence = match instance_id {
        Some(instance_id) => state
            .presences
            .get_mut(instance_id)
            .filter(|presence| presence.name == agent),
        None => presence_of_agent(state, agent),
    };
    let Some(presence) = presence else {
        return DaemonToWrapper::Nack {
            id: "domain".to_string(),
            reason: format!("agent introuvable: {}", agent),
        };
    };
    presence.domain = match domain {
        Some(domain) => Some(domain),
        // Réinitialisation : on retombe sur ce que le wrapper avait annoncé.
        None => presence.derived_domain.clone(),
    };
    log::debug!("domaine de '{}' : {:?}", presence.name, presence.domain);
    DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: "domain".to_string(),
    }
}

/// Déclare la disponibilité d'un agent.
#[cfg(test)]
fn handle_availability(
    agent: &str,
    until_secs: Option<u64>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    handle_availability_for_instance(agent, until_secs, None, state)
}

fn handle_availability_for_instance(
    agent: &str,
    until_secs: Option<u64>,
    instance_id: Option<&str>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    const MAX_DND_WINDOW_SECS: u64 = 7 * 24 * 60 * 60;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let remaining = match until_secs {
        None => None,
        Some(until) => match until.checked_sub(now).filter(|remaining| *remaining > 0) {
            Some(remaining) => Some(remaining),
            None => {
                return DaemonToWrapper::Nack {
                    id: "availability".to_string(),
                    reason: "échéance DND déjà expirée".to_string(),
                };
            }
        },
    };
    if remaining.is_some_and(|remaining| remaining > MAX_DND_WINDOW_SECS) {
        return DaemonToWrapper::Nack {
            id: "availability".to_string(),
            reason: "durée DND supérieure à 7 jours".to_string(),
        };
    }
    let deadline = match remaining {
        Some(remaining) => match Instant::now().checked_add(Duration::from_secs(remaining)) {
            Some(deadline) => Some(deadline),
            None => {
                return DaemonToWrapper::Nack {
                    id: "availability".to_string(),
                    reason: "échéance DND hors plage".to_string(),
                };
            }
        },
        None => None,
    };
    let presence = match instance_id {
        Some(instance_id) => state
            .presences
            .get_mut(instance_id)
            .filter(|presence| presence.name == agent),
        None => presence_of_agent(state, agent),
    };
    let Some(presence) = presence else {
        return DaemonToWrapper::Nack {
            id: "availability".to_string(),
            reason: format!("agent introuvable: {}", agent),
        };
    };
    presence.dnd_until = deadline;
    log::debug!(
        "disponibilité de '{}' : dnd={}",
        presence.name,
        presence.is_dnd()
    );
    DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: "availability".to_string(),
    }
}

/// Session 116 : cadence de l'entretien périodique du daemon.
const MAINTENANCE_FIRST_DELAY: Duration = Duration::from_secs(120);
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(3600);
/// Marge après l'expiration d'une saga d'envoi avant sa purge : pendant elle,
/// un rejeu tardif reçoit encore « expiré » au lieu d'être réexpédié.
const SEND_PURGE_GRACE_SECS: i64 = 30 * 24 * 3600;
/// Âge minimal d'un nom ou d'une preuve d'identité orphelins avant retrait.
const IDENTITY_FILES_MIN_AGE: Duration = Duration::from_secs(7 * 24 * 3600);

/// FR006 : deux puits (tick des relances et entretien), un retrait transactionnel.
/// O(N + B + H(log N + log R) + L + P), N lignes SQL, R demandes, H retenus,
/// P suivis mémoire ; les identifiants retirés sont regroupés pour un seul retain.
/// B enveloppes et L dépôts liés ; le balayage global n’est pas un scan borné par LIMIT.
/// Le caller garde State ; la mémoire ne change qu'après le COMMIT de settle_held.
fn sweep_held(st: &mut DaemonState, now: i64) -> Result<(), String> {
    let held = {
        let mut statement = st
            .store
            .connection()
            .prepare(
                "SELECT d.delivery_id, d.message_bytes, d.expires_at, r.expires_at
             FROM send_deliveries d JOIN idempotency_records r
               ON r.issuer_scope=d.issuer_scope AND r.operation_kind=d.operation_kind
                 AND r.idempotency_key=d.idempotency_key
             WHERE d.phase='held' ORDER BY d.queue_rank",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };
    let mut settled_ids = HashSet::new();
    // Même pattern Result local que wait_request : les erreurs SQL restent inchangées.
    let result = (|| -> Result<(), String> {
        for (delivery_id, bytes, delivery_expires, record_expires) in held {
            let message: bridget_core::BridgetMessage = serde_json::from_slice(&bytes)
                .map_err(|error| format!("retenu {delivery_id} illisible: {error}"))?;
            let reason = if delivery_expires <= now || record_expires <= now {
                Some("expired_in_queue")
            } else if message.reply {
                let request = st
                    .store
                    .get_request(&message.id)
                    .map_err(|error| error.to_string())?;
                (!request.is_some_and(|request| {
                    request.state == "open"
                        && request.sender == message.from
                        && request.target == message.to
                        && request.deadline_at > now
                }))
                .then_some("request_closed_in_queue")
            } else {
                None
            };
            if let Some(reason) = reason
                && let Some(notice) = st
                    .idempotency
                    .settle_held(&delivery_id, reason, now)
                    .map_err(|error| error.to_string())?
            {
                st.held_ids.remove(&notice.message_id);
                settled_ids.insert(notice.message_id);
            }
        }
        Ok(())
    })();
    // Une erreur tardive ne doit pas conserver les suivis des COMMIT déjà réussis.
    if !settled_ids.is_empty() {
        st.pending_replies
            .retain(|pending| !settled_ids.contains(&pending.msg_id));
    }
    result
}

/// Entretien des magasins SQLite, sous le verrou de l'état. Ordre imposé (session 150) :
/// la recopie des accusés natifs précède `purge_expired_sends`, qui efface les remises
/// quelle que soit leur phase ; la purge des arbres suit, jamais au démarrage.
fn maintain_stores(st: &mut DaemonState, now: i64) {
    if let Err(error) = sweep_held(st, now) {
        warn!("entretien: balayage retenu refusé, purges suspendues: {error}");
        return;
    }
    match st.idempotency.settle_expired_dispatching(now) {
        Ok(0) => {}
        Ok(count) => info!(
            "entretien: {count} remise(s) expirée(s) passée(s) de « en vol » à « sort inconnu »"
        ),
        Err(error) => warn!("entretien: remises expirées : {error}"),
    }
    // Sans preuve enregistrée, les remises ne sont pas effacées dans ce passage.
    match native_delegation::copy_delivery_acks(st) {
        Ok(acked) => {
            if acked > 0 {
                info!("entretien: {acked} accusé(s) de remise native recopié(s) dans les tâches");
            }
            match st
                .idempotency
                .purge_expired_sends(now, SEND_PURGE_GRACE_SECS)
            {
                Ok(0) => {}
                Ok(count) => {
                    info!("entretien: {count} envoi(s) expiré(s) depuis plus de 30 jours purgé(s)")
                }
                Err(error) => warn!("entretien: purge des envois : {error}"),
            }
        }
        Err(error) => {
            warn!("entretien: accusés natifs non recopiés, purge des envois suspendue : {error}")
        }
    }
    native_delegation::maintain_retention(st, now);
    if let Err(error) = st.idempotency.purge_guidance_events(now) {
        warn!("entretien: purge du guidage : {error}");
    }
    if let Err(error) = st.idempotency.purge_queue_edits(now) {
        warn!("entretien: purge des reçus d'édition : {error}");
    }
}

/// Session 116 : un passage d'entretien. Chaque étape est indépendante : un
/// échec est journalisé. Un retrait retenu refusé suspend les purges dépendantes.
/// State couvre SQL et les notices historiques ; les remises normales restent
/// différées hors State. Les fichiers sont traités hors verrou.
fn run_maintenance(state: &Arc<Mutex<DaemonState>>, root: &std::path::Path) {
    let Some(_update_activity) = update_activity(state) else {
        return;
    };
    let now = unix_now_secs();
    let connected: HashSet<String> = {
        let mut st = state.lock().recover_poison();
        maintain_stores(&mut st, now);
        // Réemploi des notices durables ; ce chemin historique prend le writer sous State.
        st.flush_pending_orphan_emitter_notices();
        st.conn_instances.values().cloned().collect()
    };
    let identite = crate::mcp_identity::purge_stale_identity_files(
        root,
        &connected,
        IDENTITY_FILES_MIN_AGE,
        std::time::SystemTime::now(),
    );
    if identite != crate::mcp_identity::IdentityPurge::default() {
        info!(
            "entretien: état d'identité des disparus retiré : {} marqueur(s), {} nom(s), {} preuve(s)",
            identite.markers, identite.names, identite.proofs
        );
    }
    let tmp = crate::disk_hygiene::purge_orphan_bridget_tmp(&root.join("tmp"));
    if !tmp.deleted.is_empty() {
        info!(
            "entretien: {} temporaire(s) privé(s) retiré(s)",
            tmp.deleted.len()
        );
    }
    crate::wrapper::purge_orphan_mcp_configs(root);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    for log in crate::disk_hygiene::service_logs(root, home.as_deref()) {
        match crate::disk_hygiene::rotate_log_if_large(&log, crate::disk_hygiene::LOG_ROTATE_BYTES)
        {
            Ok(true) => info!(
                "entretien: journal {} tourné (génération précédente en .1)",
                log.display()
            ),
            Ok(false) => {}
            Err(error) => warn!("entretien: rotation de {} : {error}", log.display()),
        }
    }
}

fn unix_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

/// Échéance de tour lue à CHAUD : `agents.json` courant, sinon registre en
/// mémoire, sinon défaut. Contourne la définition figée au spawn — sans
/// exiger de relancer les agents déjà connectés.
fn live_notify_timeout_secs(fallback: &AgentRegistry, agent_type: &str) -> u64 {
    let from = |registry: &AgentRegistry| -> u64 {
        if agent_type.is_empty() {
            return DEFAULT_NOTIFY_TIMEOUT_SECS;
        }
        registry
            .get(agent_type)
            .map(|definition| definition.notify_timeout_secs)
            .unwrap_or(DEFAULT_NOTIFY_TIMEOUT_SECS)
    };
    match AgentRegistry::load() {
        Ok(live) => from(&live),
        Err(_) => from(fallback),
    }
}

/// Pose `deadline_at` absolue d'exécution au moment de la
/// POUSSÉE vers le wrapper. Point unique : couvre Deliver classique ET
/// DeliverIdempotent (le service compagnon / reprise après redémarrage). Sans cela, le
/// worker retombe sur `notify_timeout` figé dans fleet.json — encore 600 s
/// pour les codex absents de agents.json.
///
/// `reply_timeout` reste un délai métier de suivi de réponse. Il ne borne pas
/// le tour du fournisseur : une question humaine doit pouvoir dépasser 60 s.
/// SPEC-088 : en posture découverte, seule la définition de découverte
/// (lecture seule attestée par le registre) est lancée ; un type qui n'en a
/// pas est refusé, jamais lancé en posture complète par repli. Un état de
/// contrôle illisible ne bloque pas le lancement mais est journalisé.
fn resolve_spawn_agent_type_for_posture(
    st: &DaemonState,
    agent_type: &str,
) -> Result<String, SpawnRefusal> {
    // Session 127 : un type absent du registre est dit tel quel. Le refus
    // « posture_decouverte » qui le masquait a été pris deux fois par des agents
    // pour la garde du terminal humain.
    if st.registry.get(agent_type).is_err() {
        return Err(SpawnRefusal::UnknownType {
            requested_type: agent_type.to_string(),
            known_types: st.registry.known_types(),
            registry: st.registry.source().display().to_string(),
        });
    }
    match crate::referent_control::read(st.store.connection()) {
        Ok(control)
            if control.agent_posture
                == Some(bridget_transport::protocol::AgentPosture::Discovery) =>
        {
            match st.registry.project_discovery_agent_type(agent_type) {
                Some(discovery_type) => Ok(discovery_type),
                None => {
                    warn!(
                        "lancement refusé: posture découverte exigée et aucune définition de découverte pour '{agent_type}'"
                    );
                    Err(SpawnRefusal::UnsupportedCapability {
                        agent_type: agent_type.to_string(),
                        model: String::new(),
                        capability: "posture_decouverte".to_string(),
                    })
                }
            }
        }
        Ok(_) => Ok(agent_type.to_string()),
        Err(error) => {
            // Repli fermé : sans état lisible, on ne lance jamais en posture
            // complète ; la découverte si elle existe, sinon refus.
            warn!("lancement: état de contrôle illisible, repli en posture découverte: {error}");
            st.registry
                .project_discovery_agent_type(agent_type)
                .ok_or_else(|| SpawnRefusal::UnsupportedCapability {
                    agent_type: agent_type.to_string(),
                    model: String::new(),
                    capability: "posture_decouverte".to_string(),
                })
        }
    }
}

fn stamp_turn_deadline_for_delivery(
    message: &mut bridget_core::BridgetMessage,
    registry: &AgentRegistry,
    agent_type: &str,
) {
    if message.deadline_at.is_some() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let timeout_secs = if message.intent == Some(bridget_core::MessageIntent::SteerCurrent) {
        HUMAN_STEER_TIMEOUT_SECS
    } else {
        live_notify_timeout_secs(registry, agent_type)
    };
    message.deadline_at = Some(now.saturating_add(timeout_secs));
    info!(
        "échéance de tour posée: to={} type={} timeout_secs={} deadline_at={}",
        message.to,
        if agent_type.is_empty() {
            "?"
        } else {
            agent_type
        },
        timeout_secs,
        message.deadline_at.unwrap_or(0)
    );
}

fn next_delivery_generation() -> u64 {
    loop {
        let generation = (Uuid::new_v4().as_u128() as u64) & i64::MAX as u64;
        if generation != 0 {
            return generation;
        }
    }
}

fn issue_response(key: &IdempotencyKey, issue: IdempotencyIssue) -> DaemonToWrapper {
    DaemonToWrapper::IdempotencyResult {
        project_warnings: Vec::new(),
        operation_kind: key.operation_kind.as_str().to_string(),
        idempotency_key: key.idempotency_key.clone(),
        issue,
    }
}

fn issue_response_with_project(
    st: &DaemonState,
    key: &IdempotencyKey,
    issue: IdempotencyIssue,
) -> DaemonToWrapper {
    let mut response = issue_response(key, issue);
    if let DaemonToWrapper::IdempotencyResult {
        project_warnings,
        issue,
        ..
    } = &mut response
        && matches!(
            issue,
            IdempotencyIssue::Accepted { .. }
                | IdempotencyIssue::OutcomeUnknown { .. }
                | IdempotencyIssue::Orphaned { .. }
        )
        && let Ok(Some(envelope)) = st.idempotency.stored_send_message(key)
        && let Some(frozen) = envelope.get("_project_warnings")
        && let Ok(warnings) = serde_json::from_value(frozen.clone())
    {
        *project_warnings = warnings;
    }
    response
}

fn replay_issue(
    st: &DaemonState,
    key: &IdempotencyKey,
    result: LookupResult,
) -> Result<IdempotencyIssue, String> {
    match result {
        LookupResult::Accepted { expires_at } => Ok(IdempotencyIssue::Accepted { expires_at }),
        LookupResult::Rejected {
            category,
            reason,
            expires_at,
        } => Ok(IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        }),
        LookupResult::OutcomeUnknown { expires_at } => Ok(IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id: st
                .idempotency
                .stored_send_delivery(key)
                .map_err(|error| error.to_string())?
                .filter(|delivery| matches!(delivery.phase.as_str(), "held" | "dispatching"))
                .map(|delivery| delivery.delivery_id),
        }),
        LookupResult::Orphaned { expires_at, reason } => Ok(IdempotencyIssue::Orphaned {
            expires_at,
            delivery_id: st
                .idempotency
                .orphaned_delivery_id(key)
                .map_err(|error| error.to_string())?
                .unwrap_or_default(),
            reason,
        }),
        LookupResult::IdempotencyExpired => Ok(IdempotencyIssue::IdempotencyExpired),
    }
}

fn handle_idempotency_lookup(
    conn_id: &str,
    operation_kind: String,
    idempotency_key: String,
    st: &DaemonState,
) -> DaemonToWrapper {
    let Some(negotiated) = st.client_negotiations.get(conn_id) else {
        return DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired,
        };
    };
    if operation_kind == "execution_control" {
        return match st.execution_store.lookup_control_command(
            &negotiated.issuer_scope,
            &idempotency_key,
            unix_timestamp(),
        ) {
            Ok(Some(record)) => DaemonToWrapper::ControlExecutionResult {
                command_id: idempotency_key,
                execution_id: record.execution_id,
                outcome: control_outcome_from_status(record.status),
                project_warnings: record.project_warnings,
            },
            Ok(None) => DaemonToWrapper::Nack {
                id: idempotency_key,
                reason: "commande de contrôle inconnue ou expirée".to_string(),
            },
            Err(error) => DaemonToWrapper::Nack {
                id: idempotency_key,
                reason: format!("lecture de commande de contrôle impossible: {error}"),
            },
        };
    }
    if operation_kind != "send" {
        return DaemonToWrapper::Nack {
            id: idempotency_key,
            reason: "opération idempotente inconnue".to_string(),
        };
    }
    let key = match IdempotencyKey::new(
        negotiated.issuer_scope.clone(),
        OperationKind::Send,
        idempotency_key,
    ) {
        Ok(key) => key,
        Err(error) => {
            return DaemonToWrapper::Nack {
                id: "lookup".to_string(),
                reason: error.to_string(),
            };
        }
    };
    match st.idempotency.lookup(&key, unix_now_secs()) {
        Ok(result) => match replay_issue(st, &key, result) {
            Ok(issue) => issue_response_with_project(st, &key, issue),
            Err(error) => DaemonToWrapper::Nack {
                id: key.idempotency_key,
                reason: error,
            },
        },
        Err(error) => DaemonToWrapper::Nack {
            id: key.idempotency_key,
            reason: error.to_string(),
        },
    }
}

/// Applique un fait d'exécution seulement quand il appartient encore au même
/// cycle durable. Un wrapper ne reçoit aucun acquittement optimiste : un état
/// refusé reste visible par la projection, jamais réessayé comme un autre tour.
fn handle_execution_provider_context(
    conn_id: &str,
    context: bridget_transport::protocol::ExecutionProviderContext,
    st: &mut DaemonState,
) -> Option<DaemonToWrapper> {
    let target = match st
        .execution_store
        .execution_control_target(&context.execution_id)
    {
        Ok(Some(target)) => target,
        Ok(None) => {
            return Some(DaemonToWrapper::Nack {
                id: context.execution_id,
                reason: "contexte fournisseur pour exécution inconnue".to_string(),
            });
        }
        Err(error) => {
            return Some(DaemonToWrapper::Nack {
                id: context.execution_id,
                reason: format!("lecture propriétaire contexte fournisseur: {error}"),
            });
        }
    };
    let owner = st
        .router
        .get_agent(&target.target_agent)
        .is_some_and(|agent| agent.connection_id == conn_id);
    if !owner {
        return Some(DaemonToWrapper::Nack {
            id: context.execution_id,
            reason: "contexte fournisseur émis par un wrapper non propriétaire".to_string(),
        });
    }
    match st.execution_store.record_provider_context(&context) {
        Ok(ProviderBindingOutcome::Applied) => None,
        Ok(ProviderBindingOutcome::Missing) => Some(DaemonToWrapper::Nack {
            id: context.execution_id,
            reason: "contexte fournisseur pour exécution inconnue".to_string(),
        }),
        Ok(ProviderBindingOutcome::GenerationMismatch) => Some(DaemonToWrapper::Nack {
            id: context.execution_id,
            reason: "contexte fournisseur de génération périmée".to_string(),
        }),
        Ok(ProviderBindingOutcome::Incompatible) => Some(DaemonToWrapper::Nack {
            id: context.execution_id,
            reason: "contexte fournisseur incompatible".to_string(),
        }),
        Err(error) => Some(DaemonToWrapper::Nack {
            id: context.execution_id,
            reason: format!("contexte fournisseur non persisté: {error}"),
        }),
    }
}

fn handle_execution_transition(
    conn_id: &str,
    transition: bridget_transport::protocol::ExecutionStateTransition,
    st: &mut DaemonState,
) -> Option<DaemonToWrapper> {
    if !st.conn_instances.contains_key(conn_id) {
        get_metrics().record_execution_transition_rejected();
        return Some(DaemonToWrapper::Nack {
            id: transition.execution_id,
            reason: "transition d exécution émise par une instance inconnue".to_string(),
        });
    }

    let target = match st
        .execution_store
        .execution_control_target(&transition.execution_id)
    {
        Ok(Some(target)) => target,
        Ok(None) => {
            get_metrics().record_execution_transition_rejected();
            return Some(DaemonToWrapper::Nack {
                id: transition.execution_id,
                reason: "exécution inconnue".to_string(),
            });
        }
        Err(error) => {
            get_metrics().record_execution_transition_rejected();
            return Some(DaemonToWrapper::Nack {
                id: transition.execution_id,
                reason: format!("propriétaire d exécution illisible: {error}"),
            });
        }
    };
    let is_owner = st
        .router
        .get_agent(&target.target_agent)
        .is_some_and(|agent| agent.connection_id == conn_id);
    if !is_owner {
        get_metrics().record_execution_transition_rejected();
        return Some(DaemonToWrapper::Nack {
            id: transition.execution_id,
            reason: "transition d exécution émise par un wrapper non propriétaire".to_string(),
        });
    }

    match st.execution_store.transition_if_current(
        &transition.execution_id,
        &transition.expected_state,
        transition.expected_revision,
        transition.generation,
        &transition.next_state,
        &transition.reason,
        transition.observed_at,
    ) {
        Ok(ConditionalTransition::Applied(applied)) => {
            let metrics = get_metrics();
            if transition.next_state == "running" {
                let latency_secs =
                    u64::try_from(transition.observed_at.saturating_sub(applied.created_at))
                        .unwrap_or(0);
                metrics.record_execution_started(latency_secs);
            }
            if transition.reason == "provider_queue_full" {
                metrics.record_execution_queue_saturated();
            }
            let in_progress = transition.next_state == "running";
            let attested = st
                .conn_instances
                .get(conn_id)
                .and_then(|id| st.presences.get(id))
                .is_some_and(|presence| presence.turn_attest);
            // Un terminal d'exécution n'atteste pas le repos de la session réelle.
            // Les porteurs legacy gardent leur projection historique.
            if (in_progress || !attested)
                && let Err(reason) = st.set_turn_state(conn_id, in_progress)
            {
                warn!("projection de tour après transition ignorée: {reason}");
            }
            if matches!(
                transition.next_state.as_str(),
                "completed" | "failed" | "cancelled" | "interrupted" | "unreachable"
            ) {
                schedule_held_release(st, &target.target_agent, Instant::now());
            }
            None
        }
        Ok(ConditionalTransition::Rejected(current)) => {
            get_metrics().record_execution_transition_rejected();
            Some(DaemonToWrapper::Nack {
                id: transition.execution_id,
                reason: format!(
                    "transition périmée: état={}, révision={}, génération={}",
                    current.state, current.revision, current.generation
                ),
            })
        }
        Ok(ConditionalTransition::Missing) => {
            get_metrics().record_execution_transition_rejected();
            Some(DaemonToWrapper::Nack {
                id: transition.execution_id,
                reason: "exécution inconnue".to_string(),
            })
        }
        Err(error) => {
            get_metrics().record_execution_transition_rejected();
            Some(DaemonToWrapper::Nack {
                id: transition.execution_id,
                reason: format!("transition d exécution non persistée: {error}"),
            })
        }
    }
}

fn handle_delivery_ack(
    conn_id: &str,
    delivery_id: String,
    delivery_generation: u64,
    st: &mut DaemonState,
) -> Option<DaemonToWrapper> {
    let Some(instance_id) = st.conn_instances.get(conn_id).cloned() else {
        return Some(DaemonToWrapper::Nack {
            id: delivery_id,
            reason: "accusé idempotent émis par une instance inconnue".to_string(),
        });
    };
    match st
        .idempotency
        .acknowledge_send_delivery(&delivery_id, &instance_id, delivery_generation)
    {
        Ok(answered_request) => {
            if let Some((owner, _)) = live_connection_identity(st, conn_id)
                && let Ok(Some(summary)) = st.store.observation_summary_current(&owner)
                && let Ok(key) = IdempotencyKey::new(
                    crate::wake_schedules::MISSED_SUMMARY_SCOPE,
                    OperationKind::Send,
                    &summary.summary_id,
                )
                && let Ok(Some(delivery)) = st.idempotency.stored_send_delivery(&key)
                && delivery.delivery_id == delivery_id
                && delivery.recipient_instance_id == instance_id
                && delivery.delivery_generation == delivery_generation
            {
                let _ = st.store.bind_observation_summary_delivery(
                    &summary.summary_id,
                    &delivery_id,
                    &instance_id,
                    delivery_generation,
                );
                let _ = st.store.confirm_observation_summary(
                    &owner,
                    &summary.summary_id,
                    summary.generation,
                    &delivery_id,
                    &instance_id,
                    delivery_generation,
                );
            }
            if let Some(request_id) = answered_request {
                st.delegation_store.wait_epoch().bump();
                st.pending_replies
                    .retain(|pending| pending.msg_id != request_id);
                info!("demande {} répondue après accusé idempotent", request_id);
            }
            settle_thread_wake_by_delivery(
                st,
                &delivery_id,
                crate::store::threads::WakeOutcome::Dispatched,
            );
            #[cfg(feature = "test-support")]
            crate::test_sync::checkpoint("after_delivery_acked");
            None
        }
        Err(error) => Some(DaemonToWrapper::Nack {
            id: delivery_id,
            reason: error.to_string(),
        }),
    }
}

fn reject_idempotent_send(
    st: &mut DaemonState,
    key: &IdempotencyKey,
    category: impl Into<String>,
    reason: impl Into<String>,
) -> Result<DaemonToWrapper, String> {
    let category = category.into();
    let reason = reason.into();
    st.idempotency
        .reject_prepared(key, &category, &reason)
        .map_err(|error| error.to_string())?;
    let expires_at = match st.idempotency.lookup(key, unix_now_secs()) {
        Ok(LookupResult::Rejected { expires_at, .. }) => expires_at,
        Ok(_) => return Err("refus idempotent non terminal".to_string()),
        Err(error) => return Err(error.to_string()),
    };
    Ok(issue_response(
        key,
        IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        },
    ))
}

/// Gardes communes précédant toute remise. Les deux appels publics conservent
/// leurs identités de déduplication propres, mais partagent les invariants de
/// réponse, DND, sauts et routage.
struct PreparedDispatch {
    project_warnings: Vec<bridget_transport::protocol::ProjectWarning>,
    content_key: String,
    message_guard_id: String,
    target_conn: String,
    logical_sender: String,
    reply_sender_conn: Option<String>,
    valid_tracked_reply: bool,
}

/// Les échanges affichent « humain », alors que le routeur ne connaît que
/// l'UUID canonique du relais UI. Ne jamais modifier le message : son libellé
/// reste une donnée métier et historique. Cette translation est strictement
/// interne au routage et vaut dans les deux sens.
fn routing_agent_id(agent_id: &str) -> &str {
    if agent_id == UI_HUMAN_SENDER {
        UI_HUMAN_AGENT_ID
    } else {
        agent_id
    }
}

/// Le libellé `humain` est conservé dans le fil, mais le fournisseur ne doit
/// jamais le résoudre via un profil historique homonyme. Sans cette exception,
/// l'agent peut croire que la demande vient d'un autre agent et répondre au
/// mauvais destinataire.
fn provider_display_name(st: &DaemonState, sender: &str) -> Option<String> {
    if sender == UI_HUMAN_SENDER {
        Some("Utilisateur".to_string())
    } else {
        st.store.agent_display_name(sender).ok().flatten()
    }
}

/// Projection de remise : l'appartenance est relue, aucun titre client ne fait
/// autorité. Une erreur de lecture retire le titre sans bloquer la remise.
/// O(log T + M), via thread_show existant ; M est borné à 16 membres.
fn provider_thread_display_title(
    st: &DaemonState,
    message: &bridget_core::BridgetMessage,
) -> Option<String> {
    let notice = message.thread_notice.as_ref()?;
    st.store
        .thread_show(&message.to, &notice.thread_id)
        .ok()?
        .ok()
        .map(|view| view.thread.title)
}

/// Une conversation sans projet explicite reste une portée privée et stable.
/// Son identifiant est dérivé de l'agent déjà authentifié par Bridget, jamais
/// d'un paramètre fourni par le navigateur ou le fournisseur.
fn artifact_scope_for_agent(agent_id: &str) -> String {
    format!("conversation-agent:{agent_id}")
}

/// Session 102 : départ figé d'une sollicitation de fil.
struct ThreadWakeDeparture {
    thread_id: String,
    agent_id: String,
    through_seq: u64,
    generation: u64,
    delivery_id: String,
    delivery_key: String,
    recipient_instance_id: String,
    delivery_generation: u64,
    issued_at: i64,
    expires_at: i64,
    target_conn: String,
}

/// Destinataire vivant d'une alerte : connexion principale et instance.
fn thread_wake_recipient(st: &DaemonState, agent_id: &str) -> Option<(String, String)> {
    let route = st.router.get_agent(agent_id)?;
    let conn = route.connection_id.clone();
    if !st.connections.contains_key(&conn) {
        return None;
    }
    let instance = st.conn_instances.get(&conn)?.clone();
    st.presences
        .get(&instance)
        .filter(|presence| matches!(presence.state.as_str(), "connected" | "busy"))?;
    Some((conn, instance))
}

/// Session 102 : sélection, réservation durable et projection 099 des
/// sollicitations prêtes. Tout se passe sous le verrou d'état, sans E/S
/// fournisseur ; la poussée vers la connexion est différée dans `controls`.
/// Au plus `WAKE_BATCH` candidats et `WAKE_DEPARTURES_PER_SEC` départs par tick.
fn dispatch_thread_wakes(
    st: &mut DaemonState,
    now_secs: i64,
    now: Instant,
    controls: &mut Vec<DeferredControl>,
) {
    if let Err(error) = st.store.thread_wakes_expire(now_secs) {
        error!("fils 102 : échéances de sollicitation : {error}");
    }
    recover_thread_wakes_in_flight(st, now_secs, controls);
    let candidates = match st
        .store
        .thread_wake_candidates(now_secs, crate::threads::WAKE_BATCH)
    {
        Ok(candidates) => candidates,
        Err(error) => {
            error!("fils 102 : candidats de sollicitation : {error}");
            return;
        }
    };
    for candidate in candidates {
        st.thread_wake_departures
            .retain(|departure| now.duration_since(*departure) < Duration::from_secs(1));
        if crate::agent_tracking::dormant(st.store.connection(), &candidate.agent_id)
            .unwrap_or(true)
        {
            continue;
        }
        if st.thread_wake_departures.len() >= crate::threads::WAKE_DEPARTURES_PER_SEC as usize {
            let _ = st.store.thread_wake_defer(
                &candidate.thread_id,
                &candidate.agent_id,
                "rate_limited",
                now_secs,
            );
            break;
        }
        let Some((target_conn, instance)) = thread_wake_recipient(st, &candidate.agent_id) else {
            let _ = st.store.thread_wake_defer(
                &candidate.thread_id,
                &candidate.agent_id,
                "offline",
                now_secs,
            );
            continue;
        };
        if st
            .presences
            .get(&instance)
            .is_some_and(|presence| presence.is_dnd())
        {
            let _ = st.store.thread_wake_defer(
                &candidate.thread_id,
                &candidate.agent_id,
                "dnd",
                now_secs,
            );
            continue;
        }
        if st.thread_notice_versions.get(&target_conn).copied()
            != Some(crate::threads::THREAD_NOTICE_VERSION)
        {
            let _ = st.store.thread_wake_defer(
                &candidate.thread_id,
                &candidate.agent_id,
                "capability_unavailable",
                now_secs,
            );
            continue;
        }
        let generation = candidate.generation + 1;
        let departure = ThreadWakeDeparture {
            delivery_id: Uuid::new_v4().to_string(),
            delivery_key: format!(
                "thread-wake:{}:{}:{generation}",
                candidate.thread_id, candidate.agent_id
            ),
            thread_id: candidate.thread_id.clone(),
            agent_id: candidate.agent_id.clone(),
            through_seq: candidate.pending_seq,
            generation,
            recipient_instance_id: instance,
            delivery_generation: next_delivery_generation(),
            issued_at: now_secs,
            expires_at: now_secs + crate::threads::WAKE_INJECTION_DEADLINE_SECS,
            target_conn,
        };
        let reserved = st.store.thread_wake_reserve(
            crate::store::threads::WakeReservation {
                thread_id: &departure.thread_id,
                agent_id: &departure.agent_id,
                expected_generation: candidate.generation,
                through_seq: departure.through_seq,
                delivery_id: &departure.delivery_id,
                delivery_key: &departure.delivery_key,
                recipient_instance_id: &departure.recipient_instance_id,
                delivery_generation: departure.delivery_generation,
                issued_at: departure.issued_at,
                expires_at: departure.expires_at,
            },
            now_secs,
        );
        match reserved {
            Ok(true) => {}
            Ok(false) => continue,
            Err(error) => {
                error!("fils 102 : réservation de sollicitation : {error}");
                continue;
            }
        }
        st.thread_wake_departures.push_back(now);
        if let Err(reason) = project_thread_wake(st, &departure, controls) {
            warn!(
                "fils 102 : sollicitation {} → {} refusée avant injection : {reason}",
                departure.thread_id, departure.agent_id
            );
            let _ = st.store.thread_wake_settle(
                &departure.thread_id,
                &departure.agent_id,
                departure.generation,
                crate::store::threads::WakeOutcome::Refused(reason),
            );
        }
    }
}

/// Reprise : une réservation de fil figée avant sa gravure 099 (arrêt entre
/// les deux écritures) reprend la même clé, la même instance et la même
/// génération ; jamais une nouvelle identité de remise.
fn recover_thread_wakes_in_flight(
    st: &mut DaemonState,
    now_secs: i64,
    controls: &mut Vec<DeferredControl>,
) {
    let Ok(in_flight) = st.store.thread_wakes_in_flight() else {
        return;
    };
    let Ok(issuer_scope) = st.idempotency.supervisor_scope() else {
        return;
    };
    for wake in in_flight {
        let (
            Some(delivery_id),
            Some(delivery_key),
            Some(instance),
            Some(generation),
            Some(issued_at),
            Some(expires_at),
            Some(through_seq),
        ) = (
            wake.delivery_id.clone(),
            wake.delivery_key.clone(),
            wake.recipient_instance_id.clone(),
            wake.delivery_generation,
            wake.issued_at,
            wake.expires_at,
            wake.active_through_seq,
        )
        else {
            continue;
        };
        let Ok(key) = IdempotencyKey::new(
            issuer_scope.clone(),
            OperationKind::Send,
            delivery_key.clone(),
        ) else {
            continue;
        };
        // Déjà gravée : la reprise 099 au réenregistrement de l'instance fait foi.
        if !matches!(
            st.idempotency.lookup(&key, now_secs),
            Ok(LookupResult::IdempotencyExpired)
        ) {
            continue;
        }
        let Some((target_conn, live_instance)) = thread_wake_recipient(st, &wake.agent_id) else {
            continue;
        };
        if live_instance != instance {
            // L'instance figée a disparu : issue inconnue visible, pas de reroutage.
            continue;
        }
        let departure = ThreadWakeDeparture {
            thread_id: wake.thread_id.clone(),
            agent_id: wake.agent_id.clone(),
            through_seq,
            generation: wake.generation,
            delivery_id,
            delivery_key,
            recipient_instance_id: instance,
            delivery_generation: generation,
            issued_at,
            expires_at,
            target_conn,
        };
        if let Err(reason) = project_thread_wake(st, &departure, controls) {
            warn!("fils 102 : reprise de sollicitation impossible : {reason}");
        }
    }
}

/// Projection d'une sollicitation figée vers la remise idempotente 099 :
/// scope superviseur durable, clé métier de fil, canon avec notice typée,
/// remise bornée par l'échéance d'injection puis par l'horizon de clé.
fn project_thread_wake(
    st: &mut DaemonState,
    departure: &ThreadWakeDeparture,
    controls: &mut Vec<DeferredControl>,
) -> Result<(), String> {
    if crate::agent_tracking::dormant(st.store.connection(), &departure.agent_id)
        .map_err(|_| "tracking_storage_unavailable")?
    {
        return Err("agent_dormant".into());
    }
    let issuer_scope = st
        .idempotency
        .supervisor_scope()
        .map_err(|error| error.to_string())?;
    let key = IdempotencyKey::new(
        issuer_scope,
        OperationKind::Send,
        departure.delivery_key.clone(),
    )
    .map_err(|error| error.to_string())?;
    let mut message = bridget_core::BridgetMessage::new(
        "bridget",
        &departure.agent_id,
        crate::threads::notice_body(&departure.thread_id, departure.through_seq),
    );
    message.id = crate::threads::notice_message_id(
        &departure.thread_id,
        &departure.agent_id,
        departure.generation,
    );
    message.reply = false;
    message.origin = Some(bridget_core::MessageOrigin::System);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    message.deadline_at = Some(departure.expires_at.max(0) as u64);
    message.thread_notice = Some(bridget_core::ThreadNotice {
        version: crate::threads::THREAD_NOTICE_VERSION,
        thread_id: departure.thread_id.clone(),
        through_seq: departure.through_seq,
        generation: departure.generation,
    });
    let canonical = canonical_send(
        &key.issuer_scope,
        &key.idempotency_key,
        &message,
        departure.issued_at,
    );
    let reservation_expiry = match st.idempotency.reserve(
        &key,
        &canonical,
        departure.issued_at,
        CLIENT_IDEMPOTENCY_HORIZON_SECS,
        departure.issued_at,
        CLIENT_ISSUED_AT_TOLERANCE_SECS,
    ) {
        Ok(Reservation::Prepared { expires_at }) => expires_at,
        // Déjà gravée par un passage précédent : la remise existante fait foi.
        Ok(Reservation::Replayed(_)) => return Ok(()),
        Ok(other) => return Err(format!("réservation inattendue : {other:?}")),
        Err(error) => return Err(error.to_string()),
    };
    let delivery = SendDelivery {
        delivery_id: departure.delivery_id.clone(),
        recipient_instance_id: departure.recipient_instance_id.clone(),
        delivery_generation: departure.delivery_generation,
        expires_at: departure.expires_at.min(reservation_expiry),
        message_bytes: serde_json::to_vec(&message).map_err(|error| error.to_string())?,
    };
    st.idempotency
        .begin_send_delivery(&key, &delivery)
        .map_err(|error| error.to_string())?;
    defer_idempotent_delivery(
        st,
        &departure.target_conn,
        delivery,
        controls,
        Some(bridget_core::MessageOrigin::System),
        None,
    )
}

/// Corrèle une issue de remise 099 à la génération de sollicitation figée.
fn settle_thread_wake_by_delivery(
    st: &mut DaemonState,
    delivery_id: &str,
    outcome: crate::store::threads::WakeOutcome,
) {
    let Ok(Some(wake)) = st.store.thread_wake_by_delivery(delivery_id) else {
        return;
    };
    if let Err(error) =
        st.store
            .thread_wake_settle(&wake.thread_id, &wake.agent_id, wake.generation, outcome)
    {
        error!("fils 102 : issue de sollicitation : {error}");
    }
}

/// Session 102 : regard de l'annuaire vivant pour les fils. Un membre peut
/// être déconnecté s'il reste connu du routeur ou d'une présence retenue.
struct DaemonDirectory<'a> {
    state: &'a DaemonState,
}

impl crate::threads::Directory for DaemonDirectory<'_> {
    fn communication_scope(
        &self,
        actor: &str,
        members: &[String],
        reason: Option<&str>,
    ) -> Result<Vec<bridget_transport::protocol::ProjectWarning>, serde_json::Value> {
        let sender = agent_communication_project(self.state, actor);
        let mut warnings = Vec::new();
        for member in members.iter().filter(|m| m.as_str() != actor) {
            warnings.extend(
                crate::communication::project_scope(
                    sender.as_ref(),
                    agent_communication_project(self.state, member).as_ref(),
                    member,
                    reason,
                )
                .map_err(|detail| {
                    let code = detail
                        .split(" : ")
                        .next()
                        .unwrap_or("invalid_cross_project_reason");
                    crate::threads::error(code, detail.clone(), false)
                })?,
            );
        }
        Ok(warnings)
    }
    fn known_agent(&self, agent_id: &str) -> bool {
        self.state.router.get_agent(agent_id).is_some()
            || self
                .state
                .presences
                .values()
                .any(|presence| presence.name == agent_id)
    }

    fn member_facts(&self, agent_id: &str) -> crate::threads::MemberFacts {
        let connection = self
            .state
            .router
            .get_agent(agent_id)
            .map(|route| route.connection_id.clone());
        // Connecté = route vivante ET présence active ; un membre connu sans
        // route est simplement absent, jamais « inconnu ».
        let connected = Some(connection.as_ref().is_some_and(|conn| {
            self.state.connections.contains_key(conn)
                && self
                    .state
                    .conn_instances
                    .get(conn)
                    .and_then(|instance| self.state.presences.get(instance))
                    .is_some_and(|presence| matches!(presence.state.as_str(), "connected" | "busy"))
        }));
        let thread_notice_version = connection
            .as_ref()
            .and_then(|conn| self.state.thread_notice_versions.get(conn).copied());
        crate::threads::MemberFacts {
            dormant: crate::agent_tracking::dormant(self.state.store.connection(), agent_id)
                .unwrap_or(true),
            connected,
            thread_notice_version,
        }
    }
}

/// Autorité unique de publication ET lecture : principal/instance attestés,
/// route vivante, ou connexion MCP auxiliaire déjà reconnue pour cette même
/// instance. Le client n'élargit jamais sa portée en fournissant un identifiant.
fn live_connection_identity(st: &DaemonState, conn_id: &str) -> Option<(String, String)> {
    let agent = st.conn_names.get(conn_id)?;
    let instance = st.conn_instances.get(conn_id)?;
    let route = st.router.get_agent(agent)?;
    if !st.connections.contains_key(&route.connection_id)
        || st.conn_instances.get(&route.connection_id) != Some(instance)
        || (route.connection_id != conn_id && !st.auxiliary_connections.contains(conn_id))
        || !st.presences.get(instance).is_some_and(|presence| {
            presence.name == *agent
                && presence_within_retention(presence)
                && matches!(presence.state.as_str(), "connected" | "busy")
        })
    {
        return None;
    }
    Some((agent.clone(), instance.clone()))
}

fn connection_communication_project(
    st: &DaemonState,
    conn_id: &str,
) -> Option<bridget_transport::protocol::CommunicationProject> {
    let owner = if let Some((agent, _)) = live_connection_identity(st, conn_id) {
        st.router.get_agent(&agent)?.connection_id.as_str()
    } else if st.conn_names.contains_key(conn_id) {
        return None;
    } else {
        conn_id
    };
    st.communication_projects
        .get(owner)
        .and_then(|fact| fact.0.clone())
}

fn agent_communication_project(
    st: &DaemonState,
    agent: &str,
) -> Option<bridget_transport::protocol::CommunicationProject> {
    let route = st.router.get_agent(routing_agent_id(agent))?;
    if !st.connections.contains_key(&route.connection_id) {
        return None;
    }
    live_connection_identity(st, &route.connection_id)?;
    st.communication_projects
        .get(&route.connection_id)
        .and_then(|fact| fact.0.clone())
}

fn announce_communication_project(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    root: &str,
    source: bridget_transport::protocol::CommunicationProjectSource,
    host: &str,
    worktree: Option<&str>,
) -> DaemonToWrapper {
    use bridget_transport::protocol::CommunicationProjectSource;
    let (owner, local, foreign, native_command, proof_host, generation) = {
        let st = state.lock().recover_poison();
        let owner = live_connection_identity(&st, conn_id);
        let bare = owner.is_none()
            && !st.conn_names.contains_key(conn_id)
            && st.client_negotiations.get(conn_id).is_some_and(|n| {
                n.capabilities
                    .contains(&ClientCapability::CommunicationProjectsV1)
            });
        if st.auxiliary_connections.contains(conn_id) || (owner.is_none() && !bare) {
            return DaemonToWrapper::Nack { id: "communication-project".into(), reason: "communication_project_owner_required : une connexion auxiliaire hérite du projet".into() };
        }
        let t3 = st
            .conn_instances
            .get(conn_id)
            .and_then(|instance| st.presences.get(instance))
            .is_some_and(|presence| presence.transport == "t3code");
        let host_ok = bridget_core::host_is_attested(host)
            && (bare && host == st.host || st.conn_hosts.get(conn_id).is_some_and(|v| v == host))
            && (source == CommunicationProjectSource::Git || t3);
        // Une tâche native ne devient jamais locale par simple égalité de hostname.
        // Une erreur de lecture de sa relation interdit aussi le repli local.
        let native_owner = source == CommunicationProjectSource::Git
            && owner.as_ref().is_some_and(|(agent, _)| {
                !matches!(st.delegation_store.for_child(agent), Ok(None))
            });
        let native_command =
            if host_ok && source == CommunicationProjectSource::Git && worktree.is_none() {
                native_delegation::native_communication_project_worker(&st, conn_id, root)
            } else {
                None
            };
        let foreign = host_ok
            && owner.is_some()
            && ((host != st.host && t3 && source == CommunicationProjectSource::T3)
                || native_command.is_some())
            && owner
                .as_ref()
                .and_then(|(_, instance)| st.presences.get(instance))
                .is_some_and(|p| p.host == host);
        (
            owner,
            host_ok && host == st.host && !native_owner,
            foreign,
            native_command.as_ref().map(|(_, command)| command.clone()),
            native_command.map_or_else(|| host.to_owned(), |(host, _)| host),
            st.instance_id.clone(),
        )
    };
    let (resolved, unavailable) = if local {
        (
            crate::communication::resolve_communication_project(root, host, source, worktree),
            false,
        )
    } else if foreign {
        match origin_proof::communication_project(state, &proof_host, root, source, worktree) {
            Ok(project) => (project, false),
            Err(_) => (None, true),
        }
    } else {
        (None, false)
    };
    let verified = resolved.is_some();
    let mut st = state.lock().recover_poison();
    let unchanged = generation == st.instance_id
        && live_connection_identity(&st, conn_id) == owner
        && (local
            || st
                .conn_hosts
                .get(conn_id)
                .is_some_and(|current| current == host))
        && native_command.as_ref().is_none_or(|command| {
            native_delegation::native_communication_project_worker(&st, conn_id, root).as_ref()
                == Some(&(proof_host.clone(), command.clone()))
        });
    let previous = st
        .communication_projects
        .get(conn_id)
        .cloned()
        .unwrap_or_default();
    let conflicting =
        previous.2 || matches!((&previous.1, &resolved), (Some(a), Some(b)) if a != b);
    let project = if unchanged && !conflicting {
        resolved.or_else(|| {
            previous
                .0
                .clone()
                .filter(|p| unavailable && p.host == proof_host)
        })
    } else {
        None
    };
    let last_known = previous.1.or_else(|| project.clone());
    st.communication_projects
        .insert(conn_id.into(), (project.clone(), last_known, conflicting));
    if verified
        && project.is_some()
        && (local || foreign)
        && source == CommunicationProjectSource::T3
    {
        st.t3_project_connections.insert(conn_id.into());
    } else {
        st.t3_project_connections.remove(conn_id);
        st.t3_thread_bindings.remove(conn_id);
    }
    invalidate_human_authority(&mut st);
    let warnings = if project.is_none() {
        crate::communication::project_scope(None, None, "", None).unwrap_or_default()
    } else {
        Vec::new()
    };
    DaemonToWrapper::ProjectContextResult {
        project,
        project_warnings: warnings,
    }
}

/// Session 104 : sous verrou bref, identité attestée, chemin de base
/// configuré et permis de lecture. `Err((code, reason))` sinon.
fn ledger_read_context(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
) -> Result<(String, PathBuf, crate::ledger::search::ReadPermit), (&'static str, &'static str)> {
    let st = state.lock().recover_poison();
    let Some((actor, _)) = live_connection_identity(&st, conn_id) else {
        return Err(("identity_unavailable", "identité active requise"));
    };
    let Some(permit) = st.ledger_read_permits.try_acquire() else {
        return Err((
            "busy",
            "deux recherches ou relectures sont déjà en cours : réessayer dans un instant",
        ));
    };
    Ok((actor, st.db_path.clone(), permit))
}

/// Session 104 : l'identité doit être la même connexion attestée qu'au
/// départ ; sinon le contenu est abandonné.
fn ledger_identity_still_live(state: &Arc<Mutex<DaemonState>>, conn_id: &str, actor: &str) -> bool {
    let st = state.lock().recover_poison();
    live_connection_identity(&st, conn_id).is_some_and(|(live, _)| live == actor)
}

/// Réutilise l'autorité de connexion des mutations ; un scope de rejeu ou un
/// champ from ne devient jamais une preuve d'identité.
fn sender_is_authorized(st: &DaemonState, conn_id: &str, sender: &str) -> bool {
    let sender = routing_agent_id(sender);
    st.conn_names
        .get(conn_id)
        .is_some_and(|name| name == sender)
        && (st
            .router
            .get_agent(sender)
            .is_some_and(|route| route.connection_id == conn_id)
            || live_connection_identity(st, conn_id).is_some())
}

fn register_auxiliary(
    st: &mut DaemonState,
    conn_id: &str,
    agent_id: String,
    instance_id: String,
    credential: bridget_transport::protocol::IdentityCredential,
) -> DaemonToWrapper {
    let refusal = match st.router.get_agent(&agent_id) {
        None => Some("primary_absent"),
        Some(route) if route.connection_id == conn_id || st.conn_names.contains_key(conn_id) => {
            Some("connection_already_registered")
        }
        Some(route) if st.identity_credentials.get(&route.connection_id) != Some(&credential) => {
            Some("credential_mismatch")
        }
        Some(route) if st.conn_instances.get(&route.connection_id) != Some(&instance_id) => {
            Some("instance_mismatch")
        }
        Some(route)
            if live_connection_identity(st, &route.connection_id)
                != Some((agent_id.clone(), instance_id.clone())) =>
        {
            Some("primary_not_live")
        }
        Some(_) => None,
    };
    if let Some(reason) = refusal {
        warn!("rattachement auxiliaire refusé : agent={agent_id} cause={reason}");
        return DaemonToWrapper::Nack {
            id: "register-auxiliary".into(),
            reason: format!("auxiliary_identity_unproven : {reason}"),
        };
    }
    st.conn_names.insert(conn_id.into(), agent_id.clone());
    st.conn_instances.insert(conn_id.into(), instance_id);
    st.auxiliary_connections.insert(conn_id.into());
    DaemonToWrapper::Registered {
        agent_id,
        credential: None,
    }
}

fn self_mutation_instance(
    st: &DaemonState,
    conn_id: &str,
    requested_agent: &str,
    operation: &str,
) -> Result<String, Box<DaemonToWrapper>> {
    match live_connection_identity(st, conn_id) {
        Some((agent, instance_id)) if agent == requested_agent => Ok(instance_id),
        _ => Err(Box::new(DaemonToWrapper::Nack {
            id: operation.to_string(),
            reason: "identité ou instance de connexion indisponible".to_string(),
        })),
    }
}

fn artifact_access_scope(
    st: &DaemonState,
    conn_id: &str,
) -> Result<(String, String, String), ArtifactReadRefusal> {
    let (agent, instance) =
        live_connection_identity(st, conn_id).ok_or(ArtifactReadRefusal::IdentityUnavailable)?;
    let scope = match st.fleet.project_for_agent(&agent) {
        Some(project) => {
            let binding = st
                .store
                .project_binding(&project.project_id)
                .map_err(|_| ArtifactReadRefusal::StorageUnavailable)?
                .ok_or(ArtifactReadRefusal::ScopeUnavailable)?;
            if binding.state != crate::store::ProjectBindingState::Active
                || binding.generation != project.binding_generation
            {
                return Err(ArtifactReadRefusal::ScopeUnavailable);
            }
            project.project_id
        }
        None => artifact_scope_for_agent(&agent),
    };
    Ok((agent, instance, scope))
}

fn artifact_read_response(
    request: Option<&ArtifactReadRequest>,
    outcome: ArtifactReadOutcome,
) -> DaemonToWrapper {
    DaemonToWrapper::ArtifactReadResult {
        version: ARTIFACT_READ_VERSION,
        artifact_ref: request
            .filter(|r| r.artifact_ref.len() <= bridget_transport::protocol::MAX_ARTIFACT_REF_BYTES)
            .map(|r| r.artifact_ref.clone())
            .unwrap_or_default(),
        version_ref: request
            .filter(|r| r.version_ref.len() <= bridget_transport::protocol::MAX_ARTIFACT_REF_BYTES)
            .map(|r| r.version_ref.clone())
            .unwrap_or_default(),
        outcome,
    }
}

// T015–T025 : mêmes faits sur les cinq puits ; aucun from/origin client ne prouve un droit.
// O(log p+log routes), identité primaire ou auxiliaire attestée149 réutilisée.
fn guidance_source(
    st: &DaemonState,
    conn: Option<&str>,
) -> (
    crate::guidance::GuidanceActor,
    Option<(String, String)>,
    bool,
) {
    use crate::guidance::GuidanceActor;
    let Some(conn) = conn else {
        return (GuidanceActor::BareClient, None, false);
    };
    let identity = live_connection_identity(st, conn);
    let ui = identity.as_ref().is_some_and(|(agent, instance)| {
        agent == UI_HUMAN_AGENT_ID
            && st
                .presences
                .get(instance)
                .is_some_and(|p| p.agent_type == "ui")
    });
    let actor = if st.connection_roles.get(conn) == Some(&ConnectionRole::Attach)
        && st.connections.contains_key(conn)
        && !st.conn_names.contains_key(conn)
    {
        GuidanceActor::AttachedHuman
    } else if ui {
        GuidanceActor::BareClient
    } else if let Some((agent, _)) = &identity {
        GuidanceActor::Agent { id: agent.clone() }
    } else {
        GuidanceActor::BareClient
    };
    (actor, identity, ui)
}

// O(log p), avant les deux canons et la persistance ; aucune origine Human fabriquée.
fn normalize_guidance_origin(
    st: &DaemonState,
    conn: &str,
    message: &mut bridget_core::BridgetMessage,
    control: bool,
) {
    let (actor, _, ui) = guidance_source(st, Some(conn));
    if message.origin == Some(bridget_core::MessageOrigin::System)
        || message.origin == Some(bridget_core::MessageOrigin::Human)
            && (control || actor != crate::guidance::GuidanceActor::AttachedHuman && !ui)
    {
        message.origin = None;
    }
}

// O(log p+log liens+log tâches+log événements+k), lectures indexées, k événements récents.
fn guidance_facts(
    st: &DaemonState,
    conn: Option<&str>,
    target: &str,
    action: crate::guidance::GuidanceAction,
    path: crate::guidance::GuidancePath,
    trusted_producer: Option<bridget_core::MessageOrigin>,
    child_proof: bool,
    now: i64,
) -> Result<crate::guidance::GuidanceFacts, String> {
    use crate::guidance::{GuidanceAction, GuidanceActor, GuidanceFacts, GuidancePath};
    let (mut actor, identity, ui) = guidance_source(st, conn);
    if conn.is_none() && action == GuidanceAction::Interrupt && path == GuidancePath::Control {
        actor = GuidanceActor::InternalPause;
    }
    let target = routing_agent_id(target);
    let target_instance = st
        .router
        .get_agent(target)
        .and_then(|r| live_connection_identity(st, &r.connection_id))
        .map(|(_, i)| i);
    let mut parent = false;
    let mut child = false;
    let task = st.delegation_store.for_child(target)?;
    if let (Some((agent, instance)), Some(target_instance)) = (&identity, &target_instance) {
        if let Some(link) = st
            .fleet
            .agent_link_for_child(target_instance)
            .map_err(|e| e.to_string())?
        {
            parent = link.parent_instance_id == *instance
                && matches!(
                    link.state,
                    crate::idempotency::AgentLinkState::Open
                        | crate::idempotency::AgentLinkState::Transferred
                );
        }
        if let Some(task) = &task {
            parent |= task.owner == *agent
                && task.owner_instance == *instance
                && task.child_instance.as_ref() == Some(target_instance);
        }
        if let Some(link) = st
            .fleet
            .agent_link_for_child(instance)
            .map_err(|e| e.to_string())?
        {
            child = link.parent_instance_id == *target_instance
                && matches!(
                    link.state,
                    crate::idempotency::AgentLinkState::Open
                        | crate::idempotency::AgentLinkState::Transferred
                );
        }
        if let Some(own) = st.delegation_store.for_child(agent)? {
            child |= own.owner == target
                && own.owner_instance == *target_instance
                && own.child_instance.as_ref() == Some(instance);
        }
    }
    let cross_project = conn
        .and_then(|c| connection_communication_project(st, c))
        .zip(agent_communication_project(st, target))
        .is_some_and(|(from, to)| from != to)
        && !conn.is_some_and(|conn| {
            native_delegation::admitted_task_communication248(st, conn, target, None)
        });
    let control =
        crate::referent_control::read(st.store.connection()).map_err(|e| e.to_string())?;
    let paused = matches!(
        crate::referent_control::admit_autonomous_effect(
            crate::referent_control::AutonomousEffect::AgentGuidance,
            &control
        ),
        crate::referent_control::Admission::Deferred { .. }
    );
    let (pair_count, last_pair, target_count, third_target) = if action == GuidanceAction::Interrupt
        && let GuidanceActor::Agent { id } = &actor
    {
        let (pair, last) = st
            .idempotency
            .count_interrupts_pair(id, target, now.saturating_sub(30))
            .map_err(|e| e.to_string())?;
        let (count, third) = st
            .idempotency
            .count_interrupts_target(target, now.saturating_sub(600))
            .map_err(|e| e.to_string())?;
        (pair, last, count, third)
    } else {
        (0, None, 0, None)
    };
    Ok(GuidanceFacts {
        action,
        path,
        identity_attested: identity.is_some() || actor == GuidanceActor::AttachedHuman,
        actor_is_target: identity.as_ref().is_some_and(|(agent, _)| agent == target),
        actor_is_parent: parent,
        actor_is_child: child,
        actor_is_author: false,
        message_editable: false,
        recipient_turn_active: false,
        carrier_queue_fallback: false,
        actor,
        trusted_origin: trusted_producer.or(if ui {
            Some(bridget_core::MessageOrigin::Human)
        } else {
            None
        }),
        child_proof,
        cross_project,
        paused,
        target_has_active_native_task: task.as_ref().is_some_and(|task| {
            !task.terminal()
                && !task.mission.is_empty()
                && task.child_instance.is_some()
                && task.child_instance == target_instance
        }),
        recent_interrupts_pair: pair_count,
        recent_interrupts_target: target_count,
        now,
        last_interrupt_pair_at: last_pair,
        third_latest_interrupt_target_at: third_target,
    })
}

// O(1), noms fermés partagés par l'audit et les quatre chemins de contrôle.
fn guidance_event_labels(
    actor: &crate::guidance::GuidanceActor,
    action: crate::guidance::GuidanceAction,
    path: crate::guidance::GuidancePath,
) -> (&str, &'static str, &'static str) {
    use crate::guidance::{GuidanceAction, GuidanceActor, GuidancePath};
    let actor = match actor {
        GuidanceActor::Agent { id } => id.as_str(),
        GuidanceActor::AttachedHuman => "human",
        GuidanceActor::BareClient => "unattested",
        GuidanceActor::InternalPause => "internal_pause",
    };
    let action = match action {
        GuidanceAction::Steer => "steer",
        GuidanceAction::Interrupt => "interrupt",
        GuidanceAction::MidTurnDelivery
        | GuidanceAction::QueueList
        | GuidanceAction::QueueCancel
        | GuidanceAction::QueueReorder => "read_only",
        GuidanceAction::QueuePromote => "steer",
    };
    let path = match path {
        GuidancePath::SendIdempotent => "send_idempotent",
        GuidancePath::Send => "send",
        GuidancePath::Control => "control",
        GuidancePath::Cancel => "cancel",
    };
    (actor, action, path)
}

// FR016 : quatre puits réels de refus avant la garde ; aucune autorité ni quota admis.
// O(log p+log n), identité existante puis INSERT ; aucune lecture des faits/rates/projets.
fn record_guidance_refusal(
    st: &DaemonState,
    conn: Option<&str>,
    target: &str,
    id: &str,
    action: Option<crate::guidance::GuidanceAction>,
    path: crate::guidance::GuidancePath,
    code: &str,
) {
    use crate::guidance::{GuidanceAction, GuidancePath};
    let Some(action @ (GuidanceAction::Steer | GuidanceAction::Interrupt)) = action else {
        return;
    };
    if conn.is_none() && path == GuidancePath::Control {
        return; // Pause interne : ni admission ni refus de guidage utilisateur.
    }
    let (actor, _, _) = guidance_source(st, conn);
    let (actor, action, path) = guidance_event_labels(&actor, action, path);
    if let Err(error) = st.idempotency.record_guidance_event(
        unix_timestamp(),
        actor,
        routing_agent_id(target),
        action,
        path,
        Some(id),
        &format!("refused:{code}"),
    ) {
        warn!("refus de guidage non journalisé : {error}");
    }
}

// O(log n), refus/admissions seulement ; pause interne et lecture ne sont jamais comptées.
fn record_guidance_outcome(
    st: &DaemonState,
    facts: &crate::guidance::GuidanceFacts,
    target: &str,
    id: &str,
    outcome: &str,
) -> Result<(), String> {
    use crate::guidance::{GuidanceAction, GuidanceActor};
    if facts.actor == GuidanceActor::InternalPause
        || !matches!(
            facts.action,
            GuidanceAction::Steer | GuidanceAction::Interrupt
        )
    {
        return Ok(());
    }
    let (actor, action, path) = guidance_event_labels(&facts.actor, facts.action, facts.path);
    st.idempotency
        .record_guidance_event(
            facts.now,
            actor,
            routing_agent_id(target),
            action,
            path,
            Some(id),
            outcome,
        )
        .map_err(|e| e.to_string())
}

// O(log p+log liens+log tâches+log événements+k), une décision commune et nommée.
fn check_guidance(
    st: &DaemonState,
    conn: Option<&str>,
    target: &str,
    action: crate::guidance::GuidanceAction,
    path: crate::guidance::GuidancePath,
    trusted_producer: Option<bridget_core::MessageOrigin>,
    child_proof: bool,
    id: &str,
) -> Result<crate::guidance::GuidanceFacts, (String, String)> {
    let facts = guidance_facts(
        st,
        conn,
        target,
        action,
        path,
        trusted_producer,
        child_proof,
        unix_timestamp(),
    )
    .map_err(|error| {
        record_guidance_refusal(
            st,
            conn,
            target,
            id,
            Some(action),
            path,
            "intent_not_permitted",
        );
        warn!("faits de guidage indisponibles : {error}");
        ("intent_not_permitted".into(), "identity_unattested".into())
    })?;
    if let Err(refusal) = crate::guidance::authorize(&facts) {
        let (code, reason) = guidance_refusal_fields(refusal);
        if let Err(error) =
            record_guidance_outcome(st, &facts, target, id, &format!("refused:{code}"))
        {
            warn!("refus de guidage non journalisé : {error}");
        }
        return Err((code.into(), reason));
    }
    Ok(facts)
}

// O(log p+log liens+log tâches), recalcul à chaque remise ; provenance legacy non probante.
fn guidance_mid_turn_ok(
    st: &DaemonState,
    message: &bridget_core::BridgetMessage,
    target: &str,
    trusted_producer: Option<bridget_core::MessageOrigin>,
    origin_conn: Option<&str>,
) -> bool {
    let conn = origin_conn
        .or_else(|| {
            st.pending_attach_sends
                .get(&message.id)
                .map(|pending| pending.conn_id.as_str())
        })
        .or_else(|| {
            // Une étiquette humaine ou UI durable ne prouve jamais la connexion qui l'a émise.
            (message.from != UI_HUMAN_AGENT_ID && Uuid::parse_str(&message.from).is_ok())
                .then(|| {
                    st.router
                        .get_agent(&message.from)
                        .map(|r| r.connection_id.as_str())
                })
                .flatten()
        });
    guidance_facts(
        st,
        conn,
        target,
        crate::guidance::GuidanceAction::MidTurnDelivery,
        crate::guidance::GuidancePath::SendIdempotent,
        trusted_producer,
        message.delegated_origin.is_some(),
        unix_timestamp(),
    )
    .is_ok_and(|facts| crate::guidance::authorize(&facts).is_ok())
}

// Lecture SQL par rang puis droits156 avant aperçu ; au plus 200 entrées, aucune écriture.
// Même rendu156 pour le contrôle existant et les éditions; aucune autorité ici.
fn guidance_refusal_fields(refusal: crate::guidance::GuidanceRefusal) -> (&'static str, String) {
    use crate::guidance::GuidanceRefusal;
    match refusal {
        GuidanceRefusal::IntentNotPermitted { reason } => ("intent_not_permitted", reason.to_string()),
        GuidanceRefusal::ReferentPaused => ("referent_paused", "referent_paused".into()),
        GuidanceRefusal::MissionActive => ("mission_active", "mission_active".into()),
        GuidanceRefusal::NotEditable => ("not_held", "not_held".into()),
        GuidanceRefusal::NoActiveTurn => ("no_active_turn", "no_active_turn".into()),
        GuidanceRefusal::SteerUnavailable => ("steer_unavailable", "steer_unavailable".into()),
        GuidanceRefusal::InterruptRateLimited { retry_after_secs } => ("interrupt_rate_limited", serde_json::json!({"code":"interrupt_rate_limited","retry_after_secs":retry_after_secs}).to_string()),
    }
}

pub(crate) fn queue_edit_refusal_code(refusal: crate::guidance::GuidanceRefusal) -> String {
    let (code, reason) = guidance_refusal_fields(refusal);
    if code == "intent_not_permitted" {
        format!("{code}:{reason}")
    } else {
        code.into()
    }
}

// O(log n+mB): reçu scoped ou au plus deux Held du destinataire; bytes authentiques.
// Cette lecture d'autorité ne rend aucun résultat de reçu et ne change aucune phase.
fn queue_edit_actor_is_author(
    st: &DaemonState,
    scope: &str,
    request: &str,
    recipient: &str,
    message_id: &str,
    actor: &str,
) -> rusqlite::Result<bool> {
    use rusqlite::OptionalExtension;
    let original: Option<Option<Vec<u8>>> = st
        .store
        .connection()
        .query_row(
            "SELECT d.message_bytes FROM queue_edits e LEFT JOIN send_deliveries d
           ON d.delivery_id=e.delivery_id WHERE e.issuer_scope=?1 AND e.request_id=?2",
            rusqlite::params![scope, request],
            |row| row.get(0),
        )
        .optional()?;
    let bytes = match original {
        Some(bytes) => bytes,
        None => {
            let mut stmt = st.store.connection().prepare(
                "SELECT message_bytes FROM send_deliveries WHERE recipient_agent=?1 AND phase='held'
                 AND CASE WHEN json_valid(message_bytes) THEN json_extract(message_bytes,'$.id') END=?2 LIMIT 2",
            )?;
            let rows = stmt
                .query_map(rusqlite::params![recipient, message_id], |row| {
                    row.get::<_, Vec<u8>>(0)
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            if rows.len() != 1 {
                return Ok(false);
            }
            rows.into_iter().next()
        }
    };
    Ok(bytes
        .as_deref()
        .and_then(|bytes| serde_json::from_slice::<bridget_core::BridgetMessage>(bytes).ok())
        .is_some_and(|message| {
            message.id == message_id
                && routing_agent_id(&message.to) == recipient
                && routing_agent_id(&message.from) == actor
        }))
}

// O(C+log E+log N+log R+Bgroup+B), mémoire O(B+I).
// C garde156/preuve auteur, E reçus, N remises, R demandes, Bgroup JSON cible, B octets, I acteur.
// State est tenu jusqu’au COMMIT ; le reçu seul ne prouve aucun droit.
#[allow(clippy::too_many_arguments)]
fn handle_queue_edit(
    st: &mut DaemonState,
    conn: &str,
    request_id: String,
    target: String,
    action: bridget_transport::protocol::QueueEditAction,
    message_id: String,
    before_message_id: Option<String>,
    controls: &mut Vec<DeferredControl>,
) -> (
    DaemonToWrapper,
    Option<crate::idempotency::OrphanedDeliveryNotice>,
) {
    use bridget_transport::protocol::{QueueEditAction, QueueEditState};
    let failed = |id: &str| DaemonToWrapper::Nack {
        id: id.into(),
        reason: "queue_edit_storage_unavailable".into(),
    };
    let Some(negotiated) = st.client_negotiations.get(conn) else {
        return (
            DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::NegotiationRequired,
            },
            None,
        );
    };
    let scope = negotiated.issuer_scope.clone();
    let recipient = routing_agent_id(&target).to_string();
    let mut facts = match guidance_facts(
        st,
        Some(conn),
        &recipient,
        match action {
            QueueEditAction::Cancel => crate::guidance::GuidanceAction::QueueCancel,
            QueueEditAction::Reorder => crate::guidance::GuidanceAction::QueueReorder,
            QueueEditAction::Promote => crate::guidance::GuidanceAction::QueuePromote,
        },
        crate::guidance::GuidancePath::Cancel,
        None,
        false,
        unix_now_secs(),
    ) {
        Ok(facts) => facts,
        Err(error) => {
            warn!("faits d'édition indisponibles: {error}");
            return (failed(&request_id), None);
        }
    };
    let (actor, identity, _) = guidance_source(st, Some(conn));
    let actor_label = match &actor {
        crate::guidance::GuidanceActor::Agent { id } => id.clone(),
        crate::guidance::GuidanceActor::AttachedHuman => "human".into(),
        _ => "unattested".into(),
    };
    if let Some((agent, _)) = identity {
        facts.actor_is_author = match queue_edit_actor_is_author(
            st,
            &scope,
            &request_id,
            &recipient,
            &message_id,
            &agent,
        ) {
            Ok(author) => author,
            Err(error) => {
                warn!("preuve auteur indisponible: {error}");
                return (failed(&request_id), None);
            }
        };
    }
    if let Err(refusal) = crate::guidance::authorize(&facts) {
        return (
            DaemonToWrapper::QueueEditResult {
                request_id,
                state: QueueEditState::Refused,
                refusal: Some(queue_edit_refusal_code(refusal)),
                effect: None,
            },
            None,
        );
    }
    // Faits primaires du porteur, protégés par State ; aucune déduction depuis le message.
    let target_conn = st.router.get_agent(&recipient).and_then(|route| {
        live_connection_identity(st, &route.connection_id)
            .filter(|(agent, _)| agent == &recipient)
            .filter(|_| st.connections.contains_key(&route.connection_id))
            .map(|(_, instance)| (route.connection_id.clone(), instance))
    });
    if let Some((_, instance)) = &target_conn
        && let Some(presence) = st.presences.get(instance)
    {
        facts.recipient_turn_active = presence.turn_attest
            && presence.state == "busy"
            && presence.last_attested_at.is_some_and(|at| {
                Instant::now().saturating_duration_since(at) <= Duration::from_secs(90)
            });
        facts.carrier_queue_fallback = presence.queue_fallback;
    }
    let canon = match serde_json::to_vec(&serde_json::json!({
        "action": action, "target": recipient, "message_id": message_id, "before_message_id": before_message_id,
    })) {
        Ok(canon) => canon,
        Err(error) => {
            warn!("canon d'édition indisponible: {error}");
            return (failed(&request_id), None);
        }
    };
    let result = match action {
        QueueEditAction::Cancel => st
            .idempotency
            .apply_cancel(
                &scope,
                &request_id,
                &canon,
                &actor_label,
                &recipient,
                &message_id,
                unix_now_secs(),
            )
            .map(|(receipt, notice)| (receipt, notice, None)),
        QueueEditAction::Reorder => st
            .idempotency
            .apply_reorder(
                &scope,
                &request_id,
                &canon,
                &actor_label,
                &recipient,
                &message_id,
                before_message_id.as_deref(),
                unix_now_secs(),
            )
            .map(|(receipt, _fresh)| (receipt, None, None)),
        QueueEditAction::Promote => st
            .idempotency
            .apply_promote(
                &scope,
                &request_id,
                &canon,
                &actor_label,
                &recipient,
                &message_id,
                &facts,
                queue_edit_refusal_code,
                unix_now_secs(),
            )
            .map(|(receipt, delivery, notice)| (receipt, notice, delivery)),
    };
    match result {
        Ok((receipt, notice, promoted)) => {
            if let Some(notice) = &notice {
                st.held_ids.remove(&notice.message_id);
                st.pending_replies
                    .retain(|pending| pending.msg_id != notice.message_id);
            }
            if let Some(delivery) = promoted {
                st.held_ids.remove(&message_id);
                // Seule la copie envoyée change ; BLOB, canon, ID et génération SQL restent intacts.
                let projection =
                    serde_json::from_slice::<bridget_core::BridgetMessage>(&delivery.message_bytes)
                        .map_err(|error| error.to_string())
                        .and_then(|mut message| {
                            message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
                            if !message
                                .references
                                .iter()
                                .any(|reference| reference == "steer-or-queue")
                            {
                                message.references.push("steer-or-queue".into());
                            }
                            serde_json::to_vec(&message).map_err(|error| error.to_string())
                        })
                        .and_then(|bytes| {
                            let target = target_conn
                                .as_ref()
                                .map(|(conn, _)| conn.as_str())
                                .ok_or_else(|| {
                                    "connexion primaire de promotion absente".to_string()
                                })?;
                            let mut copy = delivery.clone();
                            copy.message_bytes = bytes;
                            defer_idempotent_delivery(st, target, copy, controls, None, Some(conn))
                        });
                if let Err(error) = projection {
                    warn!("promotion committée, projection non confirmée: {error}");
                    if let Err(error) = st.idempotency.mark_delivery_indeterminate(
                        &delivery.delivery_id,
                        &delivery.recipient_instance_id,
                        delivery.delivery_generation,
                    ) {
                        warn!("quarantaine de promotion non confirmée: {error}");
                    }
                }
            }
            (
                DaemonToWrapper::QueueEditResult {
                    request_id,
                    state: receipt.state,
                    refusal: receipt.refusal,
                    effect: receipt.effect,
                },
                notice,
            )
        }
        Err(error) => {
            warn!("transaction d'édition refusée: {error}");
            (failed(&request_id), None)
        }
    }
}

fn handle_guidance_queue(st: &DaemonState, conn: &str, target: &str) -> DaemonToWrapper {
    let unavailable = || DaemonToWrapper::Nack {
        id: "queue".into(),
        reason: "target_unavailable".into(),
    };
    let target = routing_agent_id(target);
    let Some(route) = st.router.get_agent(target) else {
        return unavailable();
    };
    let Some(instance) = st.conn_instances.get(&route.connection_id) else {
        return unavailable();
    };
    if live_connection_identity(st, &route.connection_id).is_none() {
        return unavailable();
    }
    let now = unix_timestamp();
    let facts = match guidance_facts(
        st,
        Some(conn),
        target,
        crate::guidance::GuidanceAction::QueueList,
        crate::guidance::GuidancePath::SendIdempotent,
        None,
        false,
        now,
    ) {
        Ok(facts) if crate::guidance::authorize(&facts).is_ok() => facts,
        _ => return unavailable(),
    };
    let (actor, identity, ui) = guidance_source(st, Some(conn));
    let entire = actor == crate::guidance::GuidanceActor::AttachedHuman
        || ui
        || facts.actor_is_target
        || facts.actor_is_parent;
    let queue_bound = st.presences.get(instance).and_then(|presence| {
        if presence.transport == "t3code" {
            return None;
        }
        st.registry
            .get(&presence.agent_type)
            .ok()
            .and_then(|definition| definition.held_queue.then_some(definition.queue_capacity))
    });
    let rows = match st.idempotency.queued_deliveries_for_recipient(
        instance,
        now,
        200,
        queue_bound.is_some(),
    ) {
        Ok(rows) => rows,
        Err(error) => {
            warn!("lecture file impossible : {error}");
            return unavailable();
        }
    };
    let mut entries = Vec::new();
    for (delivery, issued_at, held) in rows {
        let message: bridget_core::BridgetMessage =
            match serde_json::from_slice(&delivery.message_bytes) {
                Ok(message) => message,
                Err(error) => {
                    warn!("message en file illisible : {error}");
                    return unavailable();
                }
            };
        let own = if let Some((agent, _)) = &identity {
            routing_agent_id(&message.from) == agent
        } else if let Some(negotiated) = st.client_negotiations.get(conn) {
            // Client nu : ses reçus dans son scope, jamais tous les from=human.
            let key = match IdempotencyKey::new(
                negotiated.issuer_scope.clone(),
                OperationKind::Send,
                message.id.clone(),
            ) {
                Ok(key) => key,
                Err(_) => return unavailable(),
            };
            match st.idempotency.stored_send_delivery(&key) {
                Ok(owned) => owned.is_some_and(|owned| owned.delivery_id == delivery.delivery_id),
                Err(_) => return unavailable(),
            }
        } else {
            false
        };
        if !entire && !own {
            continue;
        }
        let intent = match message.intent {
            Some(bridget_core::MessageIntent::SteerCurrent) => "steer",
            Some(bridget_core::MessageIntent::InterruptAndStart) => "interrupt",
            _ => "queue",
        };
        let position = entries.len() + 1;
        entries.push(serde_json::json!({"message_id":message.id,"from":message.from,"intent":intent,"reply":message.reply,
            "age_secs":now.saturating_sub(issued_at).max(0),"preview":message.body.chars().take(120).collect::<String>(),
            "position":position,"state":if held { "held" } else { "released" }}));
    }
    DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    }
}

fn prepare_dispatch(
    st: &mut DaemonState,
    message: &mut bridget_core::BridgetMessage,
    conn_id: &str,
    logical_sender: String,
    content_key: String,
    message_guard_id: String,
    native_task_exchange: bool,
) -> Result<PreparedDispatch, (&'static str, String)> {
    let routing_from = routing_agent_id(&message.from).to_string();
    let routing_to = routing_agent_id(&message.to).to_string();
    match crate::agent_tracking::dormant(st.store.connection(),&routing_to) {
        Ok(true) if !native_task_exchange && message.intent != Some(bridget_core::MessageIntent::StoreOnly)=>return Err(("agent_dormant",crate::agent_tracking::DORMANT_MESSAGE.into())),
        Ok(true)=>{},
        Err(_)=>return Err(("tracking_storage_unavailable","État de sommeil illisible. Conservez cette demande ; les lectures et les autres travaux autorisés peuvent continuer.".into())),
        Ok(false)=>{},
    }
    let reply_sender_conn = st
        .router
        .get_agent(&routing_from)
        .map(|agent| agent.connection_id.clone());
    if message.reply
        && reply_sender_conn.is_none()
        && !specialist_requests::is_human_request(st, message)
    {
        return Err((
            "reply_sender_unavailable",
            "--reply requiert un agent Bridget connecté ; lance la commande depuis un wrapper actif ou envoie sans --reply".to_string(),
        ));
    }

    let valid_tracked_reply = message.in_reply_to.as_deref().is_some_and(|request_id| {
        st.store
            .get_request(request_id)
            .ok()
            .flatten()
            .is_some_and(|request| {
                request.state == "open"
                    && request.sender == message.to
                    && request.target == message.from
            })
    });

    let explicit_reason = crate::communication::validate_cross_project_reason(
        message.cross_project_reason.as_deref(),
    )
    .map_err(|reason| ("invalid_cross_project_reason", reason))?;
    let inherited_reason = if explicit_reason.is_none() && valid_tracked_reply {
        message.in_reply_to.as_deref().and_then(|id| {
            st.idempotency
                .correlated_project_reason(id, &message.to, &message.from)
                .ok()
                .flatten()
        })
    } else {
        None
    };
    let scope_reason = explicit_reason.or(inherited_reason);
    // La remise interne exacte et la coordination par le parent attesté ne
    // confondent pas les emplacements d’une même mission avec deux projets.
    // La coordination ordinaire ne gagne ni autorité de mission ni exception au sommeil.
    let project_warnings = if native_task_exchange
        || native_delegation::admitted_task_communication248(
            st,
            conn_id,
            &routing_to,
            Some(&message.id),
        ) {
        Vec::new()
    } else {
        crate::communication::project_scope(
            connection_communication_project(st, conn_id).as_ref(),
            agent_communication_project(st, &routing_to).as_ref(),
            &message.to,
            scope_reason.as_deref(),
        )
        .map_err(|reason| {
            let category = if reason.starts_with(CROSS_PROJECT_REASON_REQUIRED) {
                CROSS_PROJECT_REASON_REQUIRED
            } else {
                "invalid_cross_project_reason"
            };
            (category, reason)
        })?
    };
    message.cross_project_reason = scope_reason;

    let store_only = message.intent == Some(bridget_core::MessageIntent::StoreOnly);
    // Ces gardes évitent les boucles de remise ; un dépôt ne remet ni ne relaie.
    if !store_only {
        if !st.circuit_breaker.check(&logical_sender, &message.to) {
            return Err((
                "circuit_breaker",
                format!(
                    "disjoncteur: limite {} échanges / {}s",
                    st.circuit_breaker.limit(),
                    st.circuit_breaker.window_secs()
                ),
            ));
        }
        if st.deduplicator.is_duplicate(&content_key, &message.to) {
            return Err(("duplicate_content", "doublon de contenu".to_string()));
        }
        if st
            .envelope_guard
            .is_quarantined(&message_guard_id, &message.to)
        {
            return Err((
                "quarantined",
                "message déjà relayé (quarantaine)".to_string(),
            ));
        }
        if !message.decrement_hops() {
            return Err(("hops_exhausted", "budget de sauts épuisé".to_string()));
        }
    }
    if message.intent != Some(bridget_core::MessageIntent::StoreOnly)
        && !valid_tracked_reply
        && let Some(presence) = presence_of_agent(st, &routing_to)
        && presence.is_dnd()
    {
        let minutes = presence.dnd_minutes_left();
        let target = presence.name.clone();
        return Err((
            "dnd",
            format!("« {target} » ne souhaite pas être dérangé (encore {minutes} min)"),
        ));
    }
    let target_conn = match st.router.resolve(
        &routing_from,
        &routing_to,
        if store_only { 1 } else { message.hops },
        conn_id,
    ) {
        RouterAction::Deliver { target_conn } => target_conn,
        RouterAction::Reject(error) => return Err(("routing", error.to_string())),
    };
    Ok(PreparedDispatch {
        project_warnings,
        content_key,
        message_guard_id,
        target_conn,
        logical_sender,
        reply_sender_conn,
        valid_tracked_reply,
    })
}

/// Rend durable l'attente d'une réponse après qu'une remise a été préparée.
/// Le même mécanisme est partagé par les envois historiques et idempotents,
/// afin que la reprise du daemon recharge la demande dans `pending_replies`.
fn track_reply_cycle(
    st: &mut DaemonState,
    message: &bridget_core::BridgetMessage,
    reply_sender_conn: Option<String>,
    target_conn: String,
) -> Result<(), String> {
    if !message.reply {
        return Ok(());
    }
    let timeout = message.reply_timeout.unwrap_or(60);
    st.store
        .create_request(&message.id, &message.from, &message.to, timeout)
        .map_err(|error| format!("impossible de suivre la demande: {error}"))?;
    remember_reply_cycle(st, message, reply_sender_conn, target_conn);
    Ok(())
}

fn remember_reply_cycle(
    st: &mut DaemonState,
    message: &bridget_core::BridgetMessage,
    reply_sender_conn: Option<String>,
    target_conn: String,
) {
    if !message.reply || reply_sender_conn.is_none() {
        // prepare_dispatch a déjà validé l'expéditeur, avant de diminuer hops.
        // Les demandes humaines222 utilisent tracked_requests et le journal,
        // sans socket fictive ni pending_reply destiné à un agent.
        return;
    }
    let timeout = message.reply_timeout.unwrap_or(60);
    st.pending_replies.push(PendingReply {
        msg_id: message.id.clone(),
        from: message.from.clone(),
        from_conn: reply_sender_conn
            .expect("une réponse suivie a été validée avec un expéditeur connecté"),
        to: message.to.clone(),
        target_conn,
        timeout_secs: timeout,
        created_at: std::time::Instant::now(),
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
}

/// Décision de rétention159, pure : les faits viennent des puits existants.
/// O(1), seuls les métadonnées et les préfixes fermés du message sont consultés.
fn should_hold(
    definition: &crate::registry::AgentDefinition,
    presence: &Presence,
    message: &bridget_core::BridgetMessage,
    has_held: bool,
    native_mission: bool,
) -> bool {
    definition.held_queue
        && presence.turn_attest
        && matches!(
            message.intent,
            None | Some(bridget_core::MessageIntent::QueueOnly)
        )
        && !matches!(
            message.origin,
            Some(bridget_core::MessageOrigin::Human | bridget_core::MessageOrigin::System)
        )
        && message.thread_notice.is_none()
        && !message.id.starts_with("bridget-observation:")
        && !message.id.starts_with("delegated-runtime-")
        && !native_mission
        && presence.transport != "t3code"
        && (presence.state == "busy" || has_held)
}

struct IdempotentSendAdmission {
    project: Option<ProjectReference>,
    issued_at_tolerance_secs: i64,
}

fn handle_idempotent_send(
    conn_id: &str,
    message: bridget_core::BridgetMessage,
    message_id: String,
    issued_at: i64,
    admission: IdempotentSendAdmission,
    st: &mut DaemonState,
    controls: &mut Vec<DeferredControl>,
) -> DaemonToWrapper {
    handle_idempotent_send_with_result_authority(
        conn_id, message, message_id, issued_at, admission, st, controls, None, false,
    )
}

#[derive(Clone, Copy)]
enum RetainedSendAuthority<'a> {
    WrapperReply(&'a str),
    Result(&'a native_delegation::RetainedResultAuthority),
    Specialist(&'a specialist_requests::SpecialistSendAuthority),
    Lineage(&'a native_lineage::ComposeAuthority),
}

/// Une réponse automatique ordinaire garde les droits de sa connexion primaire.
/// Ce chemin ne transporte jamais le résultat courant ou passé d'une mission.
fn authorize_wrapper_reply(
    st: &DaemonState,
    conn: &str,
    message: &bridget_core::BridgetMessage,
    instance: &str,
) -> Result<(), String> {
    if st.auxiliary_connections.contains(conn)
        || st.connection_roles.get(conn) == Some(&ConnectionRole::Attach)
        || live_connection_identity(st, conn) != Some((message.from.clone(), instance.to_owned()))
        || message.reply
        || !matches!(
            message.intent,
            None | Some(bridget_core::MessageIntent::QueueOnly)
        )
    {
        return Err("wrapper_reply_identity_required".into());
    }
    let correlation = message
        .in_reply_to
        .as_deref()
        .ok_or("wrapper_reply_correlation_required")?;
    let request = st
        .store
        .get_request(correlation)
        .map_err(|_| "wrapper_reply_request_unavailable")?
        .ok_or("wrapper_reply_request_unavailable")?;
    if request.sender != message.to || request.target != message.from {
        return Err("wrapper_reply_correlation_mismatch".into());
    }
    if st.delegation_store.for_mission(correlation)?.is_some()
        || st
            .delegation_store
            .past_mission_task(correlation)?
            .is_some()
    {
        return Err("remote_result_requires_worker".into());
    }
    Ok(())
}

fn retain_wrapper_reply(
    conn: &str,
    message: bridget_core::BridgetMessage,
    instance: &str,
    st: &mut DaemonState,
) -> DaemonToWrapper {
    let id = message.id.clone();
    let scope = crate::communication::issuer_scope(&format!("wrapper-reply:{instance}"));
    let key = match IdempotencyKey::new(scope, OperationKind::Send, id.clone()) {
        Ok(key) => key,
        Err(_) => {
            return DaemonToWrapper::Nack {
                id,
                reason: "wrapper_reply_id_invalid".into(),
            };
        }
    };
    let issued_at = match st.idempotency.original_issued_at(&key) {
        Ok(stored) => stored.unwrap_or_else(unix_now_secs),
        Err(_) => {
            return DaemonToWrapper::Nack {
                id,
                reason: "wrapper_reply_store_unavailable".into(),
            };
        }
    };
    let mut controls = Vec::new();
    let response = handle_idempotent_send_with_result_authority(
        conn,
        message,
        id.clone(),
        issued_at,
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        st,
        &mut controls,
        Some(RetainedSendAuthority::WrapperReply(instance)),
        false,
    );
    st.pending_post_response_controls
        .entry(conn.into())
        .or_default()
        .extend(controls);
    match response {
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Accepted { .. },
            project_warnings,
            ..
        }
        | DaemonToWrapper::IdempotencyResult {
            issue:
                IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(_),
                    ..
                },
            project_warnings,
            ..
        } => DaemonToWrapper::Ack {
            id,
            project_warnings,
        },
        DaemonToWrapper::IdempotencyResult { issue, .. } => DaemonToWrapper::Nack {
            id,
            reason: format!("wrapper_reply_not_deposited:{issue:?}"),
        },
        other => other,
    }
}

fn handle_idempotent_send_with_result_authority(
    conn_id: &str,
    mut message: bridget_core::BridgetMessage,
    message_id: String,
    issued_at: i64,
    admission: IdempotentSendAdmission,
    st: &mut DaemonState,
    controls: &mut Vec<DeferredControl>,
    authority: Option<RetainedSendAuthority<'_>>,
    scoped_mobile_human: bool,
) -> DaemonToWrapper {
    if message.intent == Some(bridget_core::MessageIntent::StoreOnly)
        && !st.client_negotiations.get(conn_id).is_some_and(|n| {
            n.capabilities
                .contains(&ClientCapability::SendInformationV1)
        })
    {
        return DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated,
        };
    }
    if message.reply && message.intent == Some(bridget_core::MessageIntent::SteerCurrent) {
        return DaemonToWrapper::Nack {
            id: message_id,
            reason: "invalid_intent".into(),
        };
    }
    // Session 102 : une alerte de fil n'est construite que par le daemon. Une
    // notice fournie par un client est neutralisée AVANT le canon d'idempotence,
    // pour que le rejeu reste identique et qu'aucun DM ne se fasse passer pour
    // une sollicitation.
    message.thread_notice = None;
    message.thread_display_title = None;
    // Un client négocié sans identité propre parle comme humain, comme sur le
    // chemin de contrôle : l'étiquette humaine n'est pas une identité d'agent
    // que l'on emprunte. Toute autre attribution exige une preuve.
    let human_from_bare_client = matches!(message.from.as_str(), "" | "human" | "humain")
        && !st.conn_names.contains_key(conn_id);
    if let Some(authority) = authority {
        let verified = match authority {
            RetainedSendAuthority::Result(authority) => {
                native_delegation::authorize_retained_result(st, authority, conn_id, &message)
            }
            RetainedSendAuthority::Specialist(authority) => {
                specialist_requests::authorize_prepared_send(st, authority, &message)
            }
            RetainedSendAuthority::WrapperReply(instance) => {
                authorize_wrapper_reply(st, conn_id, &message, instance)
            }
            RetainedSendAuthority::Lineage(proof) => {
                native_lineage::authorize_compose_send(st, conn_id, &message, proof)
            }
        };
        if let Err(reason) = verified {
            return DaemonToWrapper::Nack {
                id: message_id,
                reason,
            };
        }
    } else if !human_from_bare_client && !sender_is_authorized(st, conn_id, &message.from) {
        return DaemonToWrapper::Nack {
            id: message_id,
            reason: "identité expéditeur non attestée : rattachement auxiliaire requis".into(),
        };
    }
    let issuer_scope = if let Some(RetainedSendAuthority::Lineage(proof)) = authority {
        proof.issuer_scope()
    } else if let Some(RetainedSendAuthority::Specialist(authority)) = authority {
        crate::communication::issuer_scope(&format!("specialist:{}", authority.issuer_scope))
    } else if let Some(RetainedSendAuthority::WrapperReply(instance)) = authority {
        crate::communication::issuer_scope(&format!("wrapper-reply:{instance}"))
    } else if let Some(negotiated) = st.client_negotiations.get(conn_id) {
        negotiated.issuer_scope.clone()
    } else {
        return DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired,
        };
    };
    // L'identifiant métier est l'autorité publique ; l'ancien champ `id` de
    // Bridget est donc normalisé avant toute comparaison ou garde mutable.
    message.id = message_id.clone();
    let key = match IdempotencyKey::new(issuer_scope, OperationKind::Send, message_id) {
        Ok(key) => key,
        Err(_error) => {
            return DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::MessageOutsideClientRole,
            };
        }
    };
    message.cross_project_reason = match crate::communication::validate_cross_project_reason(
        message.cross_project_reason.as_deref(),
    ) {
        Ok(reason) => reason,
        Err(reason) => {
            return DaemonToWrapper::Nack {
                id: key.idempotency_key.clone(),
                reason,
            };
        }
    };
    let canonical = canonical_send(&key.issuer_scope, &key.idempotency_key, &message, issued_at);
    let now = unix_now_secs();
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("before_reservation");
    let reservation = match st.idempotency.reserve(
        &key,
        &canonical,
        issued_at,
        CLIENT_IDEMPOTENCY_HORIZON_SECS,
        now,
        admission.issued_at_tolerance_secs,
    ) {
        Ok(reservation) => reservation,
        Err(crate::idempotency::IdempotencyError::InvalidIssuedAt) => {
            return issue_response(&key, IdempotencyIssue::InvalidIssuedAt);
        }
        Err(error) => {
            error!("idempotence send: {error}");
            return DaemonToWrapper::Nack {
                id: key.idempotency_key.clone(),
                reason: "erreur de persistance idempotente".to_string(),
            };
        }
    };
    let expires_at = match reservation {
        Reservation::Prepared { expires_at } => expires_at,
        Reservation::Replayed(LookupResult::OutcomeUnknown { expires_at }) => {
            match st.idempotency.prepared_expiry(&key, now) {
                Ok(Some(expires_at)) => expires_at,
                Ok(None) => {
                    return issue_response_with_project(
                        st,
                        &key,
                        IdempotencyIssue::OutcomeUnknown {
                            expires_at,
                            delivery_id: st
                                .idempotency
                                .stored_send_delivery(&key)
                                .ok()
                                .flatten()
                                .filter(|delivery| {
                                    matches!(delivery.phase.as_str(), "held" | "dispatching")
                                })
                                .map(|delivery| delivery.delivery_id),
                        },
                    );
                }
                Err(error) => {
                    error!("idempotence reprise préparée: {error}");
                    return DaemonToWrapper::Nack {
                        id: key.idempotency_key.clone(),
                        reason: "issue idempotente illisible".to_string(),
                    };
                }
            }
        }
        Reservation::Replayed(result) => match replay_issue(st, &key, result) {
            Ok(issue) => return issue_response_with_project(st, &key, issue),
            Err(error) => {
                error!("idempotence replay: {error}");
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: "issue idempotente illisible".to_string(),
                };
            }
        },
        Reservation::EnvelopeMismatch => {
            return issue_response(&key, IdempotencyIssue::EnvelopeMismatch);
        }
        Reservation::IdempotencyExpired => {
            return issue_response(&key, IdempotencyIssue::IdempotencyExpired);
        }
    };
    // Une information ne peut ni capturer ni masquer une réponse attendue.
    // Refus durable AVANT capture_reply et avant tout effet fournisseur.
    if message.intent == Some(bridget_core::MessageIntent::StoreOnly)
        && (message.reply
            || message.reply_timeout.is_some()
            || message.in_reply_to.is_some()
            || authority.is_some()
            || st
                .delegation_store
                .for_mission(&message.id)
                .map_or(true, |task| task.is_some()))
    {
        return reject_idempotent_send(st, &key, "information_requires_uncorrelated_message",
            "Pour une demande ou un résultat attendu, gardez l'envoi normal et sa corrélation. information conserve seulement un avis sans action : reply=false, sans in_reply_to ni délai de réponse. Ce refus définitif concerne cet envoi ; aucun résultat n'a été capturé. Pour corriger, gardez la demande ou le résultat sur son parcours normal avec une nouvelle clé après ce refus attesté ; ne modifiez pas cette enveloppe sous sa clé close.")
            .unwrap_or_else(|reason| DaemonToWrapper::Nack { id: message.id.clone(), reason });
    }
    // L'autorité de résultat a déjà été vérifiée ci-dessus. Sa notification
    // est une sortie du store canonique, jamais un nouvel ingress du wrapper.
    let capture = if matches!(
        authority,
        Some(RetainedSendAuthority::Result(_) | RetainedSendAuthority::Specialist(_))
    ) {
        // Le résultat canonique reste conservé, sans nouvelle notification au dormant.
        if matches!(authority, Some(RetainedSendAuthority::Result(_))) {
            crate::agent_tracking::dormant(st.store.connection(), &message.to)
                .map_err(|e| e.to_string())
        } else {
            Ok(false)
        }
    } else {
        native_delegation::capture_reply(st, conn_id, &message)
    };
    let captured = match capture {
        Ok(captured) => captured,
        Err(error) if error == "remote_result_requires_worker" => {
            // Refus métier constaté avant capture, pas perte d'accusé.
            // La réponse corrélée clôt aussi une ancienne réservation Prepared.
            let reason = "Ce résultat de mission distante se publie par le worker. Donnez votre réponse finale normale : le worker conserve et remet le résultat natif au parent. N'utilisez pas bridget_send avec in_reply_to pour remplacer cette publication et ne rejouez pas ce message refusé. Le parent lit le résultat avec bridget_task_status et le task_id existant ; une remise en vol distincte ne prouve pas la perte de ce résultat. Ce refus concerne cet envoi manuel, sans nouvelle mission ni changement de droits. Les autres travaux autorisés suivent leur propre état.";
            return reject_idempotent_send(st, &key, error, reason).unwrap_or_else(|reason| {
                DaemonToWrapper::Nack {
                    id: message.id.clone(),
                    reason,
                }
            });
        }
        Err(error) => {
            error!("capture native non persistée: {error}");
            return DaemonToWrapper::Nack {
                id: message.id.clone(),
                reason: if error == "remote_result_requires_worker"
                    || error.starts_with("retained_reply_")
                {
                    error
                } else {
                    "native_result_not_persisted".into()
                },
            };
        }
    };
    if captured {
        if st
            .idempotency
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .is_err()
            || st
                .idempotency
                .finalize(
                    &key,
                    crate::idempotency::PublicResult::Accepted { expires_at },
                )
                .is_err()
        {
            return DaemonToWrapper::Nack {
                id: message.id.clone(),
                reason: "résultat capturé mais accusé non persisté".into(),
            };
        }
        return issue_response(&key, IdempotencyIssue::Accepted { expires_at });
    }
    // Les gardes ci-dessous peuvent consulter ou modifier les limites
    // historiques, mais seulement après la réservation d'une clé neuve.
    // Les gardes historiques restent applicables à une clé neuve,
    // mais leur espace est celui de l'émetteur idempotent : deux
    // scopes sont deux émetteurs logiques et ne se contaminent pas.
    let content_key = format!("{}:{}", key.issuer_scope, message.content_key());
    let scoped_message_id = format!("{}:{}", key.issuer_scope, message.id);
    let logical_sender = format!("client:{}", key.issuer_scope);
    let guidance_action = match message.intent {
        Some(bridget_core::MessageIntent::SteerCurrent) => {
            Some(crate::guidance::GuidanceAction::Steer)
        }
        Some(bridget_core::MessageIntent::InterruptAndStart) => {
            Some(crate::guidance::GuidanceAction::Interrupt)
        }
        _ => None,
    };
    let native_task_exchange = if matches!(
        authority,
        Some(
            RetainedSendAuthority::Result(_)
                | RetainedSendAuthority::Specialist(_)
                | RetainedSendAuthority::Lineage(_)
        )
    ) {
        // authorize_retained_result a déjà contrôlé le résultat canonique.
        true
    } else {
        match native_delegation::admitted_mission_exchange(st, conn_id, &message) {
            Ok(admitted) => admitted,
            Err(reason) => {
                return reject_idempotent_send(st, &key, "native_mission_invalid", reason)
                    .unwrap_or_else(|reason| DaemonToWrapper::Nack {
                        id: message.id.clone(),
                        reason,
                    });
            }
        }
    };
    let prepared = match prepare_dispatch(
        st,
        &mut message,
        conn_id,
        logical_sender,
        content_key,
        scoped_message_id,
        native_task_exchange,
    ) {
        Ok(prepared) => prepared,
        Err((category, reason)) => {
            record_guidance_refusal(
                st,
                Some(conn_id),
                &message.to,
                &message.id,
                guidance_action,
                crate::guidance::GuidancePath::SendIdempotent,
                category,
            );
            return reject_idempotent_send(st, &key, category, reason).unwrap_or_else(|error| {
                DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: error,
                }
            });
        }
    };
    if message.intent == Some(bridget_core::MessageIntent::StoreOnly) {
        return match st.idempotency.store_information(&key, &message, now) {
            Ok(()) => DaemonToWrapper::IdempotencyResult {
                project_warnings: prepared.project_warnings,
                operation_kind: key.operation_kind.as_str().into(),
                idempotency_key: key.idempotency_key.clone(),
                issue: IdempotencyIssue::Accepted { expires_at },
            },
            Err(crate::idempotency::IdempotencyError::InformationCollision) => {
                reject_idempotent_send(st, &key, "information_id_conflict",
                    "Cet identifiant est déjà utilisé dans le journal pour ce destinataire. Le contenu existant est conservé. Ce dépôt est refusé définitivement, sans remise au modèle. Pour publier cet avis distinct, utilisez un nouvel id et issued_at après ce refus attesté ; conservez la clé refusée pour la relire. Les autres travaux autorisés peuvent continuer.")
                    .unwrap_or_else(|reason| DaemonToWrapper::Nack { id: message.id.clone(), reason })
            }
            Err(error) => {
                warn!("information non persistée : {error}");
                DaemonToWrapper::Nack { id: message.id.clone(), reason:
                    "information_store_unavailable : dépôt non confirmé. Conservez la même clé et la même enveloppe pour relire ou reprendre ; aucun envoi au modèle n'a été effectué. Les autres travaux autorisés peuvent continuer.".into() }
            }
        };
    }
    let guidance_admission = match guidance_action {
        Some(action) => {
            let facts = if (scoped_mobile_human
                || matches!(authority, Some(RetainedSendAuthority::Lineage(_))))
                && action == crate::guidance::GuidanceAction::Steer
            {
                // The mobile ingress has just verified this device and this recipient's scope.
                guidance_facts(
                    st,
                    Some(conn_id),
                    &message.to,
                    action,
                    crate::guidance::GuidancePath::SendIdempotent,
                    Some(bridget_core::MessageOrigin::Human),
                    false,
                    unix_timestamp(),
                )
                .map(|mut facts| {
                    facts.actor = crate::guidance::GuidanceActor::AttachedHuman;
                    facts.identity_attested = true;
                    facts
                })
                .map_err(|_| ("intent_not_permitted".into(), "identity_unattested".into()))
                .and_then(|facts| {
                    crate::guidance::authorize(&facts)
                        .map(|()| facts)
                        .map_err(|refusal| {
                            let (code, reason) = guidance_refusal_fields(refusal);
                            (code.into(), reason)
                        })
                })
            } else {
                check_guidance(
                    st,
                    Some(conn_id),
                    &message.to,
                    action,
                    crate::guidance::GuidancePath::SendIdempotent,
                    None,
                    message.delegated_origin.is_some(),
                    &message.id,
                )
            };
            match facts {
                Ok(facts) => Some(facts),
                Err((code, reason)) => {
                    return reject_idempotent_send(st, &key, code, reason).unwrap_or_else(
                        |reason| DaemonToWrapper::Nack {
                            id: key.idempotency_key.clone(),
                            reason,
                        },
                    );
                }
            }
        }
        _ => None,
    };
    let Some(recipient_instance_id) = st.conn_instances.get(&prepared.target_conn).cloned() else {
        if let Some(facts) = &guidance_admission
            && let Err(error) = record_guidance_outcome(
                st,
                facts,
                &message.to,
                &message.id,
                "refused:recipient_unavailable",
            )
        {
            warn!("refus de guidage non journalisé : {error}");
        }
        return reject_idempotent_send(
            st,
            &key,
            "recipient_unavailable",
            "instance destinataire inconnue",
        )
        .unwrap_or_else(|error| DaemonToWrapper::Nack {
            id: key.idempotency_key.clone(),
            reason: error,
        });
    };
    // O(log N): messages mobiles retenus avant le porteur, même pour T3.
    let mobile_hold = scoped_mobile_human && message.intent.is_none()
        && st.presences.get(&recipient_instance_id).is_some_and(|p| p.turn_attest
            && (p.state == "busy" || st.store.connection().query_row(
                "SELECT EXISTS(SELECT 1 FROM send_deliveries WHERE recipient_agent=?1 AND phase='held')",
                [&message.to], |row| row.get::<_,bool>(0),
            ).unwrap_or(true)));
    let held_for_agent = if mobile_hold {
        Some((message.to.clone(), 32))
    } else if let Some(presence) = st.presences.get(&recipient_instance_id)
        && let Ok(definition) = st.registry.get(&presence.agent_type)
        && definition.held_queue
        && presence.turn_attest
    {
        let native_mission = match st.delegation_store.for_mission(&message.id) {
            Ok(task) => task.is_some(),
            Err(error) => {
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: format!("mission avant rétention illisible: {error}"),
                };
            }
        };
        let has_held = match st.store.connection().query_row(
            "SELECT EXISTS(SELECT 1 FROM send_deliveries
             WHERE recipient_agent = ?1 AND phase = 'held')",
            [&presence.name],
            |row| row.get::<_, bool>(0),
        ) {
            Ok(value) => value,
            Err(error) => {
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: format!("file retenue illisible: {error}"),
                };
            }
        };
        should_hold(definition, presence, &message, has_held, native_mission)
            .then(|| (presence.name.clone(), definition.queue_capacity))
    } else {
        None
    };
    let delivery = SendDelivery {
        delivery_id: Uuid::new_v4().to_string(),
        recipient_instance_id,
        delivery_generation: next_delivery_generation(),
        expires_at,
        message_bytes: match serde_json::to_value(&message).and_then(|mut envelope| {
            if !prepared.project_warnings.is_empty() {
                envelope["_project_warnings"] = serde_json::to_value(&prepared.project_warnings)?;
            }
            serde_json::to_vec(&envelope)
        }) {
            Ok(message_bytes) => message_bytes,
            Err(error) => {
                if let Some(facts) = &guidance_admission
                    && let Err(error) = record_guidance_outcome(
                        st,
                        facts,
                        &message.to,
                        &message.id,
                        "refused:persistence_failed",
                    )
                {
                    warn!("refus de guidage non journalisé : {error}");
                }
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: format!("impossible de sérialiser la remise: {error}"),
                };
            }
        },
    };
    let reply_tracking = message.reply.then(|| ReplyTracking {
        request_id: message.id.clone(),
        sender: message.from.clone(),
        target: message.to.clone(),
        created_at: now,
        // Le suivi métier de réponse garde son propre délai. L'enveloppe
        // remise au fournisseur recevra plus tard son échéance d'exécution.
        deadline_at: issued_at
            .saturating_add(message.reply_timeout.unwrap_or(60).min(i64::MAX as u64) as i64)
            .max(0),
    });
    let execution_link = if matches!(
        message.intent,
        Some(
            bridget_core::MessageIntent::TriggerTurn
                | bridget_core::MessageIntent::InterruptAndStart
        )
    ) {
        let execution_id = format!("execution-{}", message.id);
        match st.execution_store.admit_starting_message_for_project(
            &message,
            &execution_id,
            admission.project.as_ref(),
            now,
        ) {
            Ok(true) => {
                get_metrics().record_execution_admitted();
                Some(DeliveryExecutionLink {
                    delivery_id: delivery.delivery_id.clone(),
                    submission_id: message.id.clone(),
                    execution_id,
                })
            }
            Ok(false) => match st.execution_store.execution_snapshot(&execution_id) {
                Ok(Some(snapshot))
                    if !matches!(
                        snapshot.state.as_str(),
                        "interrupted" | "completed" | "failed" | "unreachable"
                    ) =>
                {
                    Some(DeliveryExecutionLink {
                        delivery_id: delivery.delivery_id.clone(),
                        submission_id: message.id.clone(),
                        execution_id,
                    })
                }
                Ok(_) => {
                    if let Some(facts) = &guidance_admission
                        && let Err(error) = record_guidance_outcome(
                            st,
                            facts,
                            &message.to,
                            &message.id,
                            "refused:terminal_execution",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    return DaemonToWrapper::Nack {
                        id: key.idempotency_key.clone(),
                        reason: "soumission déjà terminale".to_string(),
                    };
                }
                Err(error) => {
                    if let Some(facts) = &guidance_admission
                        && let Err(error) = record_guidance_outcome(
                            st,
                            facts,
                            &message.to,
                            &message.id,
                            "refused:persistence_failed",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    return DaemonToWrapper::Nack {
                        id: key.idempotency_key.clone(),
                        reason: format!("soumission existante illisible: {error}"),
                    };
                }
            },
            Err(error) => {
                if let Some(facts) = &guidance_admission
                    && let Err(error) = record_guidance_outcome(
                        st,
                        facts,
                        &message.to,
                        &message.id,
                        "refused:persistence_failed",
                    )
                {
                    warn!("refus de guidage non journalisé : {error}");
                }
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: format!("démarrage non persisté: {error}"),
                };
            }
        }
    } else {
        None
    };
    let guidance_context = guidance_admission.as_ref().map(|facts| {
        let (actor, action, path) = guidance_event_labels(&facts.actor, facts.action, facts.path);
        (
            facts.now,
            actor,
            routing_agent_id(&message.to),
            action,
            path,
            Some(message.id.as_str()),
        )
    });
    let delivery_result = st.idempotency.begin_send_delivery_with_guidance(
        &key,
        &delivery,
        reply_tracking.as_ref(),
        execution_link.as_ref(),
        guidance_context,
        held_for_agent.as_ref().map(|(_, capacity)| *capacity),
        held_for_agent.as_ref().map(|(agent, _)| agent.as_str()),
    );
    if let Err(error) = delivery_result {
        if matches!(error, crate::idempotency::IdempotencyError::HeldQueueFull) {
            return reject_idempotent_send(st, &key, "queue_full", "file retenue pleine")
                .unwrap_or_else(|reason| DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason,
                });
        }
        if let Some(facts) = &guidance_admission
            && let Err(error) = record_guidance_outcome(
                st,
                facts,
                &message.to,
                &message.id,
                "refused:persistence_failed",
            )
        {
            warn!("refus de guidage non journalisé : {error}");
        }
        error!("idempotence dispatch: {error}");
        return DaemonToWrapper::Nack {
            id: key.idempotency_key.clone(),
            reason: "impossible de préparer la remise".to_string(),
        };
    }
    // La préparation durable est le premier point qui atteste l'envoi. Avant
    // lui, un refus ne doit jamais faire passer l'expéditeur pour actif.
    if held_for_agent.is_some() {
        st.held_ids.insert(message.id.clone());
    }
    touch_message_sender_activity(st, &message.from);
    remember_reply_cycle(
        st,
        &message,
        prepared.reply_sender_conn.clone(),
        prepared.target_conn.clone(),
    );
    st.circuit_breaker
        .record(&prepared.logical_sender, &message.to);
    st.deduplicator
        .mark_sent(&prepared.content_key, &message.to);
    st.envelope_guard
        .mark_relayed(&prepared.message_guard_id, &message.to);
    if held_for_agent.is_none()
        && let Err(error) = defer_idempotent_delivery(
            st,
            &prepared.target_conn,
            delivery.clone(),
            controls,
            message
                .origin
                .filter(|origin| *origin == bridget_core::MessageOrigin::System),
            Some(conn_id),
        )
    {
        error!("remise idempotente préparée mais non sérialisable: {error}");
    }
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("after_delivery_before_issue");
    let response = issue_response_with_project(
        st,
        &key,
        IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id: Some(delivery.delivery_id),
        },
    );
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("after_issue_before_client_ack");
    response
}

fn control_result(
    command: &ExecutionControlCommand,
    outcome: ExecutionControlOutcome,
) -> DaemonToWrapper {
    control_result_with_project(command, outcome, Vec::new())
}

fn control_result_with_project(
    command: &ExecutionControlCommand,
    outcome: ExecutionControlOutcome,
    project_warnings: Vec<bridget_transport::protocol::ProjectWarning>,
) -> DaemonToWrapper {
    DaemonToWrapper::ControlExecutionResult {
        command_id: command.command_id.clone(),
        execution_id: command.execution_id.clone(),
        outcome,
        project_warnings,
    }
}

fn control_refusal_from_reason(reason: &str) -> ExecutionControlRefusal {
    match reason {
        "not_permitted" => ExecutionControlRefusal::NotPermitted,
        "capability_unavailable" => ExecutionControlRefusal::CapabilityUnavailable,
        "generation_mismatch" => ExecutionControlRefusal::GenerationMismatch,
        "revision_mismatch" => ExecutionControlRefusal::RevisionMismatch,
        "execution_not_found" => ExecutionControlRefusal::ExecutionNotFound,
        "terminal_execution" => ExecutionControlRefusal::TerminalExecution,
        "target_unavailable" => ExecutionControlRefusal::TargetUnavailable,
        "message_required" => ExecutionControlRefusal::MessageRequired,
        CROSS_PROJECT_REASON_REQUIRED => ExecutionControlRefusal::CrossProjectReasonRequired,
        "invalid_cross_project_reason" => ExecutionControlRefusal::InvalidCrossProjectReason,
        _ => ExecutionControlRefusal::InvalidCommand,
    }
}

fn control_outcome_from_status(status: ControlCommandStatus) -> ExecutionControlOutcome {
    match status {
        ControlCommandStatus::Accepted => ExecutionControlOutcome::Accepted,
        ControlCommandStatus::Refused { reason } => {
            ExecutionControlOutcome::Refused(control_refusal_from_reason(&reason))
        }
        ControlCommandStatus::Prepared | ControlCommandStatus::Dispatched => {
            ExecutionControlOutcome::OutcomeUnknown
        }
    }
}

/// Valide et remet un contrôle sans jamais transformer une intention en prompt
/// implicite. Les opérations non implémentées sont refusées avant toute E/S.
fn handle_execution_control(
    conn_id: Option<&str>,
    issuer_scope: &str,
    command: ExecutionControlCommand,
    st: &mut DaemonState,
) -> DaemonToWrapper {
    handle_execution_control_authorized(conn_id, issuer_scope, command, st, None)
}

fn handle_execution_control_authorized(
    conn_id: Option<&str>,
    issuer_scope: &str,
    mut command: ExecutionControlCommand,
    st: &mut DaemonState,
    proof: Option<&native_lineage::ComposeAuthority>,
) -> DaemonToWrapper {
    if let Some(proof) = proof
        && conn_id.is_none_or(|conn| {
            native_lineage::authorize_compose_control(st, conn, &command, proof).is_err()
        })
    {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
        );
    }
    if command.version != 1 || command.command_id.is_empty() || command.command_id.len() > 256 {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::InvalidCommand),
        );
    }
    let requires_message = matches!(command.operation, ExecutionControlOperation::SteerCurrent);
    if requires_message != command.message.is_some() {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(if requires_message {
                ExecutionControlRefusal::MessageRequired
            } else {
                ExecutionControlRefusal::InvalidCommand
            }),
        );
    }
    if let Some(message) = command.message.as_ref()
        && (message.to.is_empty()
            || message.body.is_empty()
            || message.intent != Some(bridget_core::MessageIntent::SteerCurrent))
    {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::InvalidCommand),
        );
    }
    if !matches!(
        command.operation,
        ExecutionControlOperation::SteerCurrent | ExecutionControlOperation::Interrupt
    ) {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::CapabilityUnavailable),
        );
    }
    let Some(target) = (match st
        .execution_store
        .execution_control_target(&command.execution_id)
    {
        Ok(target) => target,
        Err(error) => {
            error!("lecture cible de contrôle: {error}");
            return control_result(
                &command,
                ExecutionControlOutcome::Refused(ExecutionControlRefusal::ExecutionNotFound),
            );
        }
    }) else {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::ExecutionNotFound),
        );
    };
    if command
        .message
        .as_ref()
        .is_some_and(|message| message.to != target.target_agent)
    {
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::InvalidCommand),
        );
    }
    if let (Some(conn), Some(message)) = (conn_id, command.message.as_mut()) {
        normalize_guidance_origin(st, conn, message, true);
    }
    let invalid_reason = if let Some(message) = command.message.as_mut() {
        match crate::communication::validate_cross_project_reason(
            message.cross_project_reason.as_deref(),
        ) {
            Ok(reason) => {
                message.cross_project_reason = reason;
                false
            }
            Err(_) => true,
        }
    } else {
        false
    };
    let canonical = match encode(&command) {
        Ok(canonical) => canonical.into_bytes(),
        Err(_) => {
            return control_result(
                &command,
                ExecutionControlOutcome::Refused(ExecutionControlRefusal::InvalidCommand),
            );
        }
    };
    let now = unix_timestamp();
    let reservation = match st.execution_store.reserve_control_command(
        issuer_scope,
        &command.command_id,
        &command.execution_id,
        &canonical,
        now,
        now.saturating_add(CLIENT_IDEMPOTENCY_HORIZON_SECS),
    ) {
        Ok(reservation) => reservation,
        Err(error) => {
            error!("réservation contrôle: {error}");
            return control_result(&command, ExecutionControlOutcome::OutcomeUnknown);
        }
    };
    match reservation {
        ControlReservation::Replayed(record) => {
            return control_result_with_project(
                &command,
                control_outcome_from_status(record.status),
                record.project_warnings,
            );
        }
        ControlReservation::EnvelopeMismatch => {
            return control_result(
                &command,
                ExecutionControlOutcome::Refused(ExecutionControlRefusal::InvalidCommand),
            );
        }
        ControlReservation::Expired => {
            return control_result(&command, ExecutionControlOutcome::OutcomeUnknown);
        }
        ControlReservation::New => {}
    }
    // Le reçu durable est rendu avant toute réévaluation du contexte projet.
    // L'identité source vient de la connexion, jamais du champ from du message.
    let action = if command.operation == ExecutionControlOperation::SteerCurrent {
        crate::guidance::GuidanceAction::Steer
    } else {
        crate::guidance::GuidanceAction::Interrupt
    };
    let scope_result = if proof.is_some() {
        Ok(Vec::new())
    } else if invalid_reason {
        Err("invalid_cross_project_reason")
    } else if let Some(message) = command.message.as_ref() {
        match conn_id {
            Some(conn_id) => {
                if live_connection_identity(st, conn_id)
                    .is_some_and(|(name, _)| name != message.from)
                    || (st.conn_names.contains_key(conn_id)
                        && live_connection_identity(st, conn_id).is_none())
                {
                    Err("invalid_command")
                } else {
                    crate::communication::project_scope(
                        connection_communication_project(st, conn_id).as_ref(),
                        agent_communication_project(st, &target.target_agent).as_ref(),
                        &target.target_agent,
                        message.cross_project_reason.as_deref(),
                    )
                    .map_err(|error| {
                        if error.starts_with(CROSS_PROJECT_REASON_REQUIRED) {
                            CROSS_PROJECT_REASON_REQUIRED
                        } else {
                            "invalid_cross_project_reason"
                        }
                    })
                }
            }
            None => Err("invalid_command"),
        }
    } else if let Some(conn) = conn_id {
        crate::communication::project_scope(
            connection_communication_project(st, conn).as_ref(),
            agent_communication_project(st, &target.target_agent).as_ref(),
            &target.target_agent,
            None,
        )
        .map_err(|error| {
            if error.starts_with(CROSS_PROJECT_REASON_REQUIRED) {
                CROSS_PROJECT_REASON_REQUIRED
            } else {
                "invalid_cross_project_reason"
            }
        })
    } else {
        Ok(Vec::new())
    };
    let project_warnings = match scope_result {
        Ok(warnings) => warnings,
        Err(reason) => {
            record_guidance_refusal(
                st,
                conn_id,
                &target.target_agent,
                &command.command_id,
                Some(action),
                crate::guidance::GuidancePath::Control,
                reason,
            );
            let _ = st.execution_store.refuse_control_command(
                issuer_scope,
                &command.command_id,
                reason,
                now,
            );
            return control_result(
                &command,
                ExecutionControlOutcome::Refused(control_refusal_from_reason(reason)),
            );
        }
    };
    let admission = if proof.is_some() {
        // La preuve privée a revérifié racine, tâche, instance et ronde sous ce verrou.
        guidance_facts(
            st,
            conn_id,
            &target.target_agent,
            action,
            crate::guidance::GuidancePath::Control,
            Some(bridget_core::MessageOrigin::Human),
            false,
            now,
        )
        .map(|mut facts| {
            facts.actor = crate::guidance::GuidanceActor::AttachedHuman;
            facts.identity_attested = true;
            facts
        })
        .map_err(|_| {
            (
                "intent_not_permitted".to_owned(),
                "identity_unattested".to_owned(),
            )
        })
    } else {
        check_guidance(
            st,
            conn_id,
            &target.target_agent,
            action,
            crate::guidance::GuidancePath::Control,
            None,
            command
                .message
                .as_ref()
                .is_some_and(|m| m.delegated_origin.is_some()),
            &command.command_id,
        )
    };
    let guidance_admission = match admission {
        Ok(facts) => facts,
        Err((_code, _reason)) => {
            let _ = st.execution_store.refuse_control_command(
                issuer_scope,
                &command.command_id,
                "not_permitted",
                now,
            );
            return control_result(
                &command,
                ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
            );
        }
    };
    let refusal = if target.snapshot.generation != command.generation {
        Some("generation_mismatch")
    } else if target.snapshot.revision != command.revision {
        Some("revision_mismatch")
    } else if matches!(
        target.snapshot.state.as_str(),
        "interrupted" | "completed" | "failed" | "unreachable"
    ) {
        Some("terminal_execution")
    } else if matches!(command.operation, ExecutionControlOperation::SteerCurrent)
        && target.snapshot.state != "running"
    {
        Some("capability_unavailable")
    } else {
        None
    };
    if let Some(refusal) = refusal {
        if let Err(error) = record_guidance_outcome(
            st,
            &guidance_admission,
            &target.target_agent,
            &command.command_id,
            &format!("refused:{refusal}"),
        ) {
            warn!("refus de guidage non journalisé : {error}");
        }
        let _ = st.execution_store.refuse_control_command(
            issuer_scope,
            &command.command_id,
            refusal,
            now,
        );
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(control_refusal_from_reason(refusal)),
        );
    }
    let target_conn = st
        .router
        .get_agent(&target.target_agent)
        .map(|agent| agent.connection_id.clone());
    let Some(target_conn) = target_conn else {
        if let Err(error) = record_guidance_outcome(
            st,
            &guidance_admission,
            &target.target_agent,
            &command.command_id,
            "refused:target_unavailable",
        ) {
            warn!("refus de guidage non journalisé : {error}");
        }
        let _ = st.execution_store.refuse_control_command(
            issuer_scope,
            &command.command_id,
            "target_unavailable",
            now,
        );
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::TargetUnavailable),
        );
    };
    let Some(writer) = st.connections.get(&target_conn).cloned() else {
        if let Err(error) = record_guidance_outcome(
            st,
            &guidance_admission,
            &target.target_agent,
            &command.command_id,
            "refused:target_unavailable",
        ) {
            warn!("refus de guidage non journalisé : {error}");
        }
        let _ = st.execution_store.refuse_control_command(
            issuer_scope,
            &command.command_id,
            "target_unavailable",
            now,
        );
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::TargetUnavailable),
        );
    };
    if !st
        .execution_store
        .mark_control_dispatched(issuer_scope, &command.command_id, now, &project_warnings)
        .unwrap_or(false)
    {
        if let Err(error) = record_guidance_outcome(
            st,
            &guidance_admission,
            &target.target_agent,
            &command.command_id,
            "refused:persistence_failed",
        ) {
            warn!("refus de guidage non journalisé : {error}");
        }
        return control_result(&command, ExecutionControlOutcome::OutcomeUnknown);
    }
    if let Err(error) = record_guidance_outcome(
        st,
        &guidance_admission,
        &target.target_agent,
        &command.command_id,
        "admitted",
    ) {
        let _ = st.execution_store.resolve_control_command(
            issuer_scope,
            &command.command_id,
            false,
            Some("not_permitted"),
            now,
        );
        if let Err(error) = record_guidance_outcome(
            st,
            &guidance_admission,
            &target.target_agent,
            &command.command_id,
            "refused:persistence_failed",
        ) {
            warn!("refus de guidage non journalisé : {error}");
        }
        warn!("guidage de contrôle non persisté : {error}");
        return control_result(
            &command,
            ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
        );
    }
    if push_control_message_until(
        &writer,
        &DaemonToWrapper::ControlExecutionDispatch {
            issuer_scope: issuer_scope.to_string(),
            command: command.clone(),
        },
        Instant::now() + CANCEL_NOTIFICATION_BUDGET,
    )
    .is_err()
    {
        // Une écriture partielle peut avoir atteint le wrapper. Garder le reçu
        // dispatched : seul son accusé peut résoudre cette issue inconnue.
        return control_result_with_project(
            &command,
            ExecutionControlOutcome::OutcomeUnknown,
            project_warnings,
        );
    }
    control_result_with_project(
        &command,
        ExecutionControlOutcome::OutcomeUnknown,
        project_warnings,
    )
}

fn prepare_managed_stop(state: &Arc<Mutex<DaemonState>>, name: &str) -> ManagedStopTarget {
    let remote = state
        .lock()
        .recover_poison()
        .delegation_store
        .for_child(name);
    match remote {
        Ok(Some(task)) if task.remote_placement.is_some() => {
            return ManagedStopTarget::Immediate(native_delegation::cancel_remote_task(
                state, &task,
            ));
        }
        Err(error) => return ManagedStopTarget::Immediate(StopOutcome::Timeout { state: error }),
        _ => {}
    }
    let mut st = state.lock().recover_poison();
    let record = st
        .managed_spawns
        .values()
        .find(|spawn| spawn.lease.name == name);
    let desired = if record.is_none() {
        match st.fleet.desired_entry(name) {
            Ok(desired) => desired,
            Err(error) => {
                return ManagedStopTarget::Immediate(StopOutcome::Timeout {
                    state: error.to_string(),
                });
            }
        }
    } else {
        None
    };
    let kind = classify_stop(&st, name, record, desired.as_ref());
    prepare_classified_stop(&mut st, name, kind)
}

/// Classe sans mutation ni signal. Record et Desired viennent de la même
/// lecture sous verrou ; un arbre construit ces index une seule fois.
/// O(b) pour lire un marqueur de b octets ; O(1) pour un membre supervisé.
fn classify_stop(
    st: &DaemonState,
    name: &str,
    record: Option<&ManagedSpawnRecord>,
    desired: Option<&crate::desired_state::DesiredEquipier>,
) -> StopKind {
    match record {
        Some(record) if record.lease.remote_placement.is_some() => {
            StopKind::Immediate(StopOutcome::Timeout {
                state: "arrêt distant exige une preuve worker".into(),
            })
        }
        Some(record) => StopKind::Supervised {
            lease: record.lease.clone(),
            completion: Arc::clone(&record.stop),
            wrapper: record
                .wrapper_conn
                .as_ref()
                .and_then(|conn_id| st.connections.get(conn_id))
                .cloned(),
        },
        None => {
            if desired
                .as_ref()
                .is_some_and(|entry| entry.lifecycle_state == DesiredLifecycleState::Stopped)
            {
                return StopKind::Immediate(StopOutcome::Stopped);
            }
            if desired
                .as_ref()
                .is_some_and(|entry| entry.lifecycle_state == DesiredLifecycleState::Decommissioned)
            {
                return StopKind::Immediate(StopOutcome::NotFound);
            }
            let fallback = if st.router.get_agent(name).is_some() {
                StopOutcome::NotManaged
            } else if desired.is_some() {
                StopOutcome::Stopped
            } else {
                StopOutcome::NotFound
            };
            match st.marker_store.load(name) {
                Ok(_) => StopKind::Marker {
                    store: st.marker_store.clone(),
                    fallback,
                    stop_desired: desired.is_some(),
                },
                Err(crate::managed_process::ManagedProcessError::Io(error))
                    if error.kind() == std::io::ErrorKind::NotFound =>
                {
                    if matches!(fallback, StopOutcome::NotManaged) {
                        StopKind::Immediate(fallback)
                    } else {
                        StopKind::Marker {
                            store: st.marker_store.clone(),
                            fallback,
                            stop_desired: desired.is_some(),
                        }
                    }
                }
                Err(error) => StopKind::Immediate(StopOutcome::Timeout {
                    state: error.to_string(),
                }),
            }
        }
    }
}

/// Consomme la classification déjà vérifiée ; aucune seconde recherche de record.
fn prepare_classified_stop(st: &mut DaemonState, name: &str, kind: StopKind) -> ManagedStopTarget {
    match kind {
        StopKind::Supervised {
            lease,
            completion,
            wrapper,
        } => {
            if !completion.is_requested()
                && let Err(error) = st.fleet.invalidate_for_stop(&lease)
            {
                ManagedStopTarget::Immediate(StopOutcome::Timeout {
                    state: error.to_string(),
                })
            } else {
                st.managed_terminal_instances
                    .insert(lease.instance_id.clone());
                if wrapper.is_none() {
                    completion.mark_handshake_complete();
                }
                ManagedStopTarget::Supervised {
                    receiver: completion.request(),
                    completion,
                    wrapper,
                }
            }
        }
        StopKind::Marker {
            store,
            fallback,
            stop_desired,
        } => {
            if stop_desired && let Err(error) = st.fleet.mark_stopped(name) {
                return ManagedStopTarget::Immediate(StopOutcome::Timeout {
                    state: error.to_string(),
                });
            }
            ManagedStopTarget::Marker { store, fallback }
        }
        StopKind::Immediate(outcome) => ManagedStopTarget::Immediate(outcome),
    }
}

fn await_managed_stop(target: ManagedStopTarget, name: &str) -> StopOutcome {
    match target {
        ManagedStopTarget::Supervised {
            receiver,
            completion,
            wrapper,
        } => {
            if let Some(writer) = wrapper {
                let _ = push_control_message(&writer, &DaemonToWrapper::Disconnect);
            }
            completion.mark_handshake_complete();
            receiver
                .recv_timeout(MANAGED_STOP_REPLY_TIMEOUT)
                .unwrap_or(StopOutcome::Timeout {
                    state: "superviseur sans issue dans le délai".to_string(),
                })
        }
        ManagedStopTarget::Marker { store, fallback } => {
            match store.stop_current_group(name, MANAGED_STOP_FORCED_GRACE, MANAGED_STOP_POLL) {
                Ok(Some(ManagedStopResult::Stopped)) => StopOutcome::Stopped,
                Ok(Some(ManagedStopResult::StoppedForced { survivors_killed })) => {
                    StopOutcome::StoppedForced { survivors_killed }
                }
                Ok(Some(ManagedStopResult::Timeout)) => StopOutcome::Timeout {
                    state: "groupe du marqueur encore vivant".to_string(),
                },
                Ok(None) => fallback,
                Err(error) => StopOutcome::Timeout {
                    state: error.to_string(),
                },
            }
        }
        ManagedStopTarget::Immediate(outcome) => outcome,
    }
}

fn prune_agent_presence(state: &mut DaemonState, name: &str) {
    state.presences.retain(|_, presence| presence.name != name);
    invalidate_human_authority(state);
}

fn artifact_publication_success(
    receipt: crate::artifact_types::ArtifactReceiptV1,
    replayed: bool,
) -> DaemonToWrapper {
    DaemonToWrapper::ArtifactPublicationResult {
        contract_version: ARTIFACT_CONTRACT_VERSION,
        replayed,
        receipt_json: Some(
            serde_json::to_vec(&receipt).expect("le reçu d'artefact est sérialisable"),
        ),
        refusal_code: None,
        refusal_message: None,
    }
}

fn artifact_publication_refusal(code: &str, message: &str) -> DaemonToWrapper {
    DaemonToWrapper::ArtifactPublicationResult {
        contract_version: ARTIFACT_CONTRACT_VERSION,
        replayed: false,
        receipt_json: None,
        refusal_code: Some(code.to_string()),
        refusal_message: Some(message.to_string()),
    }
}

// ---------------------------------------------------------------------------
// SPEC-087 : état de contrôle du référent et boîte de réception humaine.
// Chaque bras du dispatch délègue ici ; la logique vit dans les modules
// `referent_control` et `human_inbox`. Rien d'autre que le verrou, la
// négociation et l'horloge n'est lu ici.
// ---------------------------------------------------------------------------

fn control_client_actor(st: &DaemonState, conn_id: &str) -> Option<&'static str> {
    st.client_negotiations.get(conn_id).and_then(|negotiated| {
        crate::referent_control::human_principal_actor(&negotiated.issuer_scope)
    })
}

/// Lecture froide : négociation/capacité contrôlées par le dispatch client.
fn handle_delegation_overview(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    version: u16,
    filter: &str,
    limit: u32,
) -> DaemonToWrapper {
    use bridget_transport::protocol::DelegationControlRefusal as E;
    if version != 1 {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    if control_client_actor(&st, conn_id).is_none() {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::HumanActorRequired,
        };
    }
    match st.delegation_store.overview_page(filter, limit) {
        Ok(page) => {
            serde_json::from_value(page).unwrap_or(DaemonToWrapper::DelegationControlRejected {
                reason: E::StoreUnavailable,
            })
        }
        Err(reason) => DaemonToWrapper::DelegationControlRejected { reason },
    }
}

/// Réemploie l'annulation humaine ; le moteur travaille après le verrou.
fn handle_delegation_cancel(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    version: u16,
    task_id: &str,
    request_id: &str,
) -> DaemonToWrapper {
    use bridget_transport::protocol::DelegationControlRefusal as E;
    if version != 1 {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::UnsupportedVersion,
        };
    }
    if crate::threads::canonical_uuid(task_id).as_deref() != Some(task_id)
        || request_id.is_empty()
        || request_id.len() > 128
        || request_id.chars().any(bridget_core::is_disallowed_control)
    {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::InvalidRequest,
        };
    }
    let drive = {
        let st = state.lock().recover_poison();
        if control_client_actor(&st, conn_id).is_none() {
            return DaemonToWrapper::DelegationControlRejected {
                reason: E::HumanActorRequired,
            };
        }
        let task = match st.delegation_store.get(task_id) {
            Ok(Some(task)) => task,
            Ok(None) => {
                return DaemonToWrapper::DelegationControlRejected {
                    reason: E::TaskUnavailable,
                };
            }
            Err(_) => {
                return DaemonToWrapper::DelegationControlRejected {
                    reason: E::StoreUnavailable,
                };
            }
        };
        match st
            .delegation_store
            .human_cancel(task.root_owner(), task_id, request_id)
        {
            Ok((_, drive)) => drive,
            Err(bridget_transport::protocol::HumanLineageError::TaskUnavailable) => {
                return DaemonToWrapper::DelegationControlRejected {
                    reason: E::TaskUnavailable,
                };
            }
            Err(bridget_transport::protocol::HumanLineageError::EnvelopeMismatch) => {
                return DaemonToWrapper::DelegationControlRejected {
                    reason: E::InvalidRequest,
                };
            }
            Err(_) => {
                return DaemonToWrapper::DelegationControlRejected {
                    reason: E::StoreUnavailable,
                };
            }
        }
    };
    if drive {
        native_delegation::tick(state);
    }
    let st = state.lock().recover_poison();
    let task = match st.delegation_store.get(task_id) {
        Ok(Some(task)) => task,
        Ok(None) => {
            return DaemonToWrapper::DelegationControlRejected {
                reason: E::TaskUnavailable,
            };
        }
        Err(_) => {
            return DaemonToWrapper::DelegationControlRejected {
                reason: E::StoreUnavailable,
            };
        }
    };
    let halt = match st.delegation_store.delegation_halt() {
        Ok(halt) => halt,
        Err(reason) => return DaemonToWrapper::DelegationControlRejected { reason },
    };
    let total_active = match st.delegation_store.admission_counts(&task.owner_instance) {
        Ok((active, _, _)) => active as u32,
        Err(_) => {
            return DaemonToWrapper::DelegationControlRejected {
                reason: E::StoreUnavailable,
            };
        }
    };
    DaemonToWrapper::DelegationOverview {
        halted: halt.halted,
        since: halt.since,
        by: halt.by,
        reason: halt.reason,
        items: vec![task.overview_row(unix_now_secs())],
        total_active,
    }
}

/// Snapshot O(B+n*d), n<=4096 tâches, d<=8 parents ; clé neuve seulement.
/// Hors coûts existants de human_cancel et du tick natif.
/// Le reçu fige les arbres visés avant l'annulation. Un crash peut laisser une
/// annulation partielle ; le même command_id ne la relance jamais (FR-009).
#[allow(clippy::too_many_arguments)]
fn handle_delegation_halt_set(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    version: u16,
    command_id: &str,
    halted: bool,
    reason: Option<&str>,
    cancel_active: bool,
) -> DaemonToWrapper {
    use bridget_transport::protocol::DelegationControlRefusal as E;
    if version != 1 {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::UnsupportedVersion,
        };
    }
    if command_id.is_empty()
        || command_id.len() > 128
        || command_id.chars().any(bridget_core::is_disallowed_control)
        || (cancel_active && !halted)
    {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::InvalidRequest,
        };
    }
    let st = state.lock().recover_poison();
    let Some(actor) = control_client_actor(&st, conn_id) else {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::HumanActorRequired,
        };
    };
    let mut targets = Vec::new();
    let response = st.delegation_store.set_delegation_halt(
        command_id,
        halted,
        actor,
        reason,
        || {
            if !cancel_active {
                return Ok(0);
            }
            let tasks = st
                .delegation_store
                .tasks()
                .map_err(|_| E::StoreUnavailable)?;
            if tasks.len() > crate::delegation::MAX_RETAINED_TASKS {
                return Err(E::StoreUnavailable);
            }
            let by_id: HashMap<_, _> = tasks
                .iter()
                .map(|task| (task.task_id.as_str(), task))
                .collect();
            let active = crate::delegation::Task::holds_slot;
            for task in tasks.iter().filter(|task| active(task)) {
                let mut parent = task.parent_task_id.as_deref();
                let mut active_ancestor = false;
                let mut depth = 0;
                while let Some(parent_id) = parent {
                    depth += 1;
                    if depth >= 8 {
                        return Err(E::StoreUnavailable);
                    }
                    let ancestor = by_id.get(parent_id).ok_or(E::StoreUnavailable)?;
                    if ancestor.root_owner() != task.root_owner() {
                        return Err(E::StoreUnavailable);
                    }
                    active_ancestor |= active(ancestor);
                    parent = ancestor.parent_task_id.as_deref();
                }
                if !active_ancestor {
                    targets.push((task.root_owner().to_owned(), task.task_id.clone()));
                }
            }
            Ok(targets.len() as u32)
        },
        unix_now_secs(),
    );
    let response: DaemonToWrapper = match response {
        Ok(response) => match serde_json::from_value(response) {
            Ok(response) => response,
            Err(_) => {
                return DaemonToWrapper::DelegationControlRejected {
                    reason: E::StoreUnavailable,
                };
            }
        },
        Err(reason) => return DaemonToWrapper::DelegationControlRejected { reason },
    };
    let DaemonToWrapper::DelegationHaltState {
        replayed,
        cancelled,
        ..
    } = &response
    else {
        return DaemonToWrapper::DelegationControlRejected {
            reason: E::StoreUnavailable,
        };
    };
    let mut drive = !targets.is_empty();
    let mut failed = false;
    if !replayed {
        // État et reçu sont déjà committés. Le verrou garde ce snapshot stable
        // jusqu'au dernier human_cancel ; aucune nouvelle admission ne s'intercale.
        for (root, task_id) in &targets {
            let request_id = format!("{command_id}:{task_id}");
            match st.delegation_store.human_cancel(root, task_id, &request_id) {
                Ok((_, run)) => drive |= run,
                Err(error) => {
                    warn!("arrêt des délégations : reçu committé, annulation refusée : {error:?}");
                    failed = true;
                    break;
                }
            }
        }
        info!(
            "{}",
            serde_json::json!({
                "event":"delegations_halt_set", "actor":actor,
                "reason":reason.map(|text| text.chars().take(crate::referent_control::MAX_REASON_CHARS).collect::<String>()),
                "targeted_trees":*cancelled, "halted":halted,
            })
        );
    }
    drop(st);
    if drive {
        native_delegation::tick(state);
    }
    if failed {
        DaemonToWrapper::DelegationControlRejected {
            reason: E::StoreUnavailable,
        }
    } else {
        response
    }
}

fn human_channel_config() -> Option<crate::human_inbox::HumanChannelConfig> {
    let path = crate::human_inbox::default_channel_config_path()?;
    match crate::human_inbox::load_channel_config(&path) {
        Ok(config) => config,
        Err(error) => {
            warn!("canal humain ignoré: {error}");
            None
        }
    }
}

/// Dépôt interne du daemon (relances, reprise de route humaine, focus en
/// attente). Le canal externe est poussé si un item a été créé.
fn daemon_deposit_human_inbox(
    st: &DaemonState,
    dedup_key: &str,
    kind: bridget_transport::protocol::HumanInboxKind,
    subject: &bridget_transport::protocol::HumanInboxSubject,
    context: &str,
    options: &[String],
    now: i64,
) -> Result<crate::human_inbox::Deposited, String> {
    #[cfg(test)]
    let channel = if st.fixture_root.is_some() {
        None
    } else {
        human_channel_config()
    };
    #[cfg(not(test))]
    let channel = human_channel_config();
    let request = crate::human_inbox::DepositRequest {
        dedup_key,
        kind,
        subject,
        context,
        options,
        producer: bridget_transport::protocol::HumanInboxProducer::Daemon,
        now,
    };
    match crate::human_inbox::deposit_and_notify(st.store.connection(), request, channel.as_ref()) {
        Ok(Ok(deposited)) => {
            if deposited.created {
                if let Some(notifier) = &st.mobile_notifier {
                    notifier.changed();
                }
                info!(
                    "boîte humaine: item {} déposé ({})",
                    deposited.item_id,
                    kind.as_sql()
                );
            }
            Ok(deposited)
        }
        Ok(Err(refusal)) => {
            warn!("boîte humaine: dépôt {dedup_key} refusé: {refusal:?}");
            Err(format!("human_inbox_refused:{refusal:?}"))
        }
        Err(error) => {
            warn!("boîte humaine: dépôt {dedup_key} impossible: {error}");
            Err(error.to_string())
        }
    }
}

fn handle_control_state_read(state: &Arc<Mutex<DaemonState>>, version: u16) -> DaemonToWrapper {
    use bridget_transport::protocol::ControlStateRefusal;
    if version != bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    let conn = st.store.connection();
    match (
        crate::referent_control::read(conn),
        crate::human_inbox::open_count(conn),
    ) {
        (Ok(state), Ok(inbox_open_count)) => DaemonToWrapper::ControlState {
            state,
            inbox_open_count,
        },
        (Ok(_), Err(error)) => {
            warn!("compteur de boîte humaine illisible: {error}");
            DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::StoreUnavailable,
            }
        }
        (Err(error), _) => {
            warn!("état de contrôle illisible: {error}");
            DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_control_focus_read(state: &Arc<Mutex<DaemonState>>, version: u16) -> DaemonToWrapper {
    use bridget_transport::protocol::ControlStateRefusal;
    if version != bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    match crate::referent_control::read_focus(st.store.connection()) {
        Ok(focus) => DaemonToWrapper::ControlFocus { focus },
        Err(error) => {
            warn!("projection focus illisible: {error}");
            DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_control_focus_publish(
    state: &Arc<Mutex<DaemonState>>,
    version: u16,
    focus: Option<bridget_transport::protocol::ControlFocusFrame>,
) -> DaemonToWrapper {
    use bridget_transport::protocol::ControlStateRefusal;
    if version != bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    match crate::referent_control::publish_focus(st.store.connection(), focus.as_ref()) {
        Ok(()) => DaemonToWrapper::ControlFocus { focus },
        Err(error) => {
            warn!("projection focus impossible: {error}");
            DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_control_history(
    state: &Arc<Mutex<DaemonState>>,
    version: u16,
    limit: u32,
) -> DaemonToWrapper {
    use bridget_transport::protocol::ControlStateRefusal;
    if version != bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    match crate::referent_control::history(st.store.connection(), limit.clamp(1, 200)) {
        Ok(events) => DaemonToWrapper::ControlHistory {
            events: events
                .into_iter()
                .map(|event| bridget_transport::protocol::ControlEventFrame {
                    at: event.at,
                    actor: event.actor,
                    kind: event.kind.to_string(),
                    reason: event.reason,
                    generation_after: event.generation_after,
                })
                .collect(),
        },
        Err(error) => {
            warn!("journal de contrôle illisible: {error}");
            DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::StoreUnavailable,
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_control_state_set(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    version: u16,
    command_id: &str,
    expected_generation: u64,
    paused: Option<bool>,
    auto_objectives_cap: Option<u32>,
    reason: Option<&str>,
    agent_posture: Option<bridget_transport::protocol::AgentPosture>,
    auto_reassignment: Option<bool>,
) -> DaemonToWrapper {
    use bridget_transport::protocol::ControlStateRefusal;
    if version != bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::UnsupportedVersion,
        };
    }
    let mut st = state.lock().recover_poison();
    let Some(actor) = control_client_actor(&st, conn_id) else {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::HumanPrincipalRequired,
        };
    };
    if paused.is_none()
        && auto_objectives_cap.is_none()
        && agent_posture.is_none()
        && auto_reassignment.is_none()
    {
        return DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::NothingToChange,
        };
    }
    let mutation = crate::referent_control::ControlMutation {
        command_id,
        expected_generation,
        paused,
        auto_objectives_cap,
        reason,
        actor,
        now: unix_now_secs(),
        agent_posture,
        auto_reassignment,
    };
    let outcome = crate::referent_control::set(st.store.connection(), mutation);
    match outcome {
        Ok(Ok(control_frame)) => {
            let mut resume_controls = Vec::new();
            if paused == Some(true) {
                interrupt_executions_for_pause(&mut st, control_frame.generation);
            } else if paused == Some(false) {
                resume_controls = resume_executions_after_pause(&mut st);
            }
            info!(
                "état de contrôle: génération {} par {actor} (pause={}, plafond={})",
                control_frame.generation, control_frame.paused, control_frame.auto_objectives_cap
            );
            let response = DaemonToWrapper::ControlState {
                state: control_frame,
                inbox_open_count: crate::human_inbox::open_count(st.store.connection())
                    .unwrap_or(0),
            };
            if paused == Some(false) && resume_controls.is_empty() {
                let agents: Vec<_> = st.presences.values().map(|p| p.name.clone()).collect();
                for agent in agents {
                    if schedule_held_release(&mut st, &agent, Instant::now())
                        && let Some(route) = st.router.get_agent(&agent)
                    {
                        let conn = route.connection_id.clone();
                        if let Some(controls) = st.pending_post_response_controls.remove(&conn) {
                            resume_controls.push((conn, controls));
                        }
                    }
                }
            }
            drop(st);
            for (target_conn, controls) in resume_controls {
                let retry = execute_controls(controls);
                if !retry.is_empty() {
                    state
                        .lock()
                        .recover_poison()
                        .pending_post_response_controls
                        .entry(target_conn)
                        .or_default()
                        .extend(retry);
                }
            }
            response
        }
        Ok(Err(reason)) => DaemonToWrapper::ControlStateRejected { reason },
        Err(error) => {
            warn!("état de contrôle inécrivable: {error}");
            DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::StoreUnavailable,
            }
        }
    }
}

fn interrupt_executions_for_pause(st: &mut DaemonState, control_generation: u64) {
    let now = unix_now_secs();
    let ids = match st.execution_store.recoverable_execution_ids() {
        Ok(ids) => ids,
        Err(error) => {
            warn!("lecture des exécutions à geler: {error}");
            return;
        }
    };
    let issuer_scope = match st.idempotency.supervisor_scope() {
        Ok(scope) => scope,
        Err(error) => {
            warn!("identité superviseur indisponible pour pause: {error}");
            return;
        }
    };
    for execution_id in ids {
        let Ok(Some(target)) = st.execution_store.execution_control_target(&execution_id) else {
            continue;
        };
        if !matches!(
            target.snapshot.state.as_str(),
            "starting" | "running" | "waiting_approval" | "waiting_user_input"
        ) {
            continue;
        }
        if let Err(error) = st.execution_store.record_pause_interruption(
            &execution_id,
            &target.target_agent,
            control_generation,
            now,
        ) {
            warn!("mémoire gel {execution_id}: {error}");
            continue;
        }
        let command = ExecutionControlCommand {
            version: 1,
            command_id: format!("control-pause-{control_generation}-{execution_id}"),
            execution_id,
            generation: target.snapshot.generation,
            revision: target.snapshot.revision,
            operation: ExecutionControlOperation::Interrupt,
            message: None,
        };
        let outcome = handle_execution_control(None, &issuer_scope, command, st);
        info!("interruption de pause demandée: {outcome:?}");
    }
}

fn resume_executions_after_pause(st: &mut DaemonState) -> Vec<(String, Vec<DeferredControl>)> {
    let mut ready = Vec::new();
    let pending = match st.execution_store.pending_pause_interruptions() {
        Ok(rows) => rows,
        Err(error) => {
            warn!("lecture gels à reprendre: {error}");
            return ready;
        }
    };
    for (_, agent_name) in pending {
        let Some(agent) = st.router.get_agent(&agent_name) else {
            continue;
        };
        let connection_id = agent.connection_id.clone();
        let Some((instance_id, _)) = st
            .presences
            .iter()
            .find(|(_, presence)| presence.name == agent_name && presence.busy_since.is_none())
        else {
            continue;
        };
        let instance_id = instance_id.clone();
        schedule_execution_recovery(st, &connection_id, &instance_id, &agent_name, false);
        if let Some(controls) = st.pending_post_response_controls.remove(&connection_id) {
            ready.push((connection_id, controls));
        }
    }
    ready
}

fn handle_human_inbox_deposit(
    state: &Arc<Mutex<DaemonState>>,
    version: u16,
    dedup_key: &str,
    kind: bridget_transport::protocol::HumanInboxKind,
    subject: &bridget_transport::protocol::HumanInboxSubject,
    context: &str,
    options: &[String],
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanInboxRefusal;
    if version != bridget_transport::protocol::HUMAN_INBOX_CONTRACT_VERSION {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnsupportedVersion,
        };
    }
    let channel = human_channel_config();
    let st = state.lock().recover_poison();
    let request = crate::human_inbox::DepositRequest {
        dedup_key,
        kind,
        subject,
        context,
        options,
        producer: bridget_transport::protocol::HumanInboxProducer::Guichet,
        now: unix_now_secs(),
    };
    match crate::human_inbox::deposit_and_notify(st.store.connection(), request, channel.as_ref()) {
        Ok(Ok(deposited)) => {
            if deposited.created {
                info!(
                    "boîte humaine: item {} déposé par le service compagnon ({})",
                    deposited.item_id,
                    kind.as_sql()
                );
            }
            DaemonToWrapper::HumanInboxDeposited {
                item_id: deposited.item_id,
                created: deposited.created,
                occurrences: deposited.occurrences,
            }
        }
        Ok(Err(reason)) => DaemonToWrapper::HumanInboxRejected { reason },
        Err(error) => {
            warn!("boîte humaine inécrivable: {error}");
            DaemonToWrapper::HumanInboxRejected {
                reason: HumanInboxRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_human_inbox_list(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    version: u16,
    filter: bridget_transport::protocol::HumanInboxListFilter,
    limit: u32,
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanInboxRefusal;
    if version != bridget_transport::protocol::HUMAN_INBOX_CONTRACT_VERSION {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    if control_client_actor(&st, conn_id).is_none() {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::HumanPrincipalRequired,
        };
    }
    match crate::human_inbox::list(st.store.connection(), filter, limit) {
        Ok((items, open_count)) => DaemonToWrapper::HumanInbox { items, open_count },
        Err(error) => {
            warn!("boîte humaine illisible: {error}");
            DaemonToWrapper::HumanInboxRejected {
                reason: HumanInboxRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_human_inbox_resolve(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    version: u16,
    item_id: &str,
    choice: &str,
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanInboxRefusal;
    if version != bridget_transport::protocol::HUMAN_INBOX_CONTRACT_VERSION {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    let Some(actor) = control_client_actor(&st, conn_id) else {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::HumanPrincipalRequired,
        };
    };
    let conn = st.store.connection();
    match crate::human_inbox::resolve(conn, item_id, choice, actor, unix_now_secs()) {
        Ok(Ok(item)) => {
            info!("boîte humaine: item {item_id} tranché « {choice} » par {actor}");
            let open_count = crate::human_inbox::open_count(conn).unwrap_or(0);
            DaemonToWrapper::HumanInbox {
                items: vec![item],
                open_count,
            }
        }
        Ok(Err(reason)) => DaemonToWrapper::HumanInboxRejected { reason },
        Err(error) => {
            warn!("boîte humaine inécrivable: {error}");
            DaemonToWrapper::HumanInboxRejected {
                reason: HumanInboxRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_human_inbox_decisions(
    state: &Arc<Mutex<DaemonState>>,
    version: u16,
    limit: u32,
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanInboxRefusal;
    if version != bridget_transport::protocol::HUMAN_INBOX_CONTRACT_VERSION {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    match crate::human_inbox::pending_decisions(
        st.store.connection(),
        bridget_transport::protocol::HumanInboxProducer::Guichet,
        limit,
    ) {
        Ok(decisions) => DaemonToWrapper::HumanInboxDecisionsBatch { decisions },
        Err(error) => {
            warn!("décisions humaines illisibles: {error}");
            DaemonToWrapper::HumanInboxRejected {
                reason: HumanInboxRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_human_inbox_ack(
    state: &Arc<Mutex<DaemonState>>,
    version: u16,
    decision_id: &str,
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanInboxRefusal;
    if version != bridget_transport::protocol::HUMAN_INBOX_CONTRACT_VERSION {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    match crate::human_inbox::ack(st.store.connection(), decision_id, unix_now_secs()) {
        Ok(Some(acked_at)) => DaemonToWrapper::HumanInboxAcked {
            decision_id: decision_id.to_string(),
            acked_at,
        },
        Ok(None) => DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnknownItem,
        },
        Err(error) => {
            warn!("acquittement humain inécrivable: {error}");
            DaemonToWrapper::HumanInboxRejected {
                reason: HumanInboxRefusal::StoreUnavailable,
            }
        }
    }
}

fn handle_human_inbox_close(
    state: &Arc<Mutex<DaemonState>>,
    version: u16,
    item_id: &str,
    reason: &str,
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanInboxRefusal;
    if version != bridget_transport::protocol::HUMAN_INBOX_CONTRACT_VERSION {
        return DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::UnsupportedVersion,
        };
    }
    let st = state.lock().recover_poison();
    match crate::human_inbox::close_self(st.store.connection(), item_id, reason, unix_now_secs()) {
        Ok(closed) => DaemonToWrapper::HumanInboxClosed {
            item_id: item_id.to_string(),
            closed,
        },
        Err(error) => {
            warn!("fermeture boîte humaine impossible: {error}");
            DaemonToWrapper::HumanInboxRejected {
                reason: HumanInboxRefusal::StoreUnavailable,
            }
        }
    }
}

/// Traite un message wrapper et retourne une réponse optionnelle.
/// Contrôle attach borné, sans verrou d'état pendant l'I/O fournisseur.
/// O(n) sur les abonnements bornés. Le token est généré ici et attaché à la
/// connexion cible : ni nom mutable ni token fourni par le client ne fait foi.
fn select_runtime_from_attach(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    agent: &str,
    selection: bridget_transport::protocol::RuntimeSelection,
) -> DaemonToWrapper {
    use bridget_transport::protocol::{
        RuntimeSelectionOutcome as Outcome, RuntimeSelectionRefusal as Refusal,
    };
    let refuse = |reason| DaemonToWrapper::RuntimeSelectionResult {
        outcome: Outcome::Refused { reason },
    };
    let deadline = Instant::now() + Duration::from_secs(4);
    let token = uuid::Uuid::new_v4().to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    let writer = {
        let mut st = state.lock().recover_poison();
        if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Attach) {
            return refuse(Refusal::AttachRequired);
        }
        if !selection.valid() {
            return refuse(Refusal::InvalidSelection);
        }
        if st.pending_runtime_selections.len() >= 64 {
            return refuse(Refusal::Busy);
        }
        let Some(target) = st
            .attach_subscriptions
            .values()
            .find(|sub| sub.attach_conn == conn_id && sub.agent == agent)
            .map(|sub| sub.wrapper_conn.clone())
        else {
            return refuse(Refusal::TargetUnavailable);
        };
        let Some(writer) = st.connections.get(&target).cloned() else {
            return refuse(Refusal::TargetUnavailable);
        };
        st.pending_runtime_selections
            .insert(token.clone(), (target, sender));
        writer
    };
    let dispatched = push_control_message_until(
        &writer,
        &DaemonToWrapper::SelectRuntime {
            token: token.clone(),
            selection,
        },
        deadline,
    );
    let outcome = if dispatched.is_ok() {
        receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap_or(Outcome::OutcomeUnknown {})
    } else {
        // Une écriture partielle peut avoir été reçue : aucune fausse certitude.
        Outcome::OutcomeUnknown {}
    };
    state
        .lock()
        .recover_poison()
        .pending_runtime_selections
        .remove(&token);
    DaemonToWrapper::RuntimeSelectionResult { outcome }
}

#[cfg(test)]
#[path = "daemon_spec099_classic_delivery_tests.rs"]
mod spec099_classic_delivery_tests;

#[cfg(test)]
#[path = "daemon_spec145_human_view_tests.rs"]
mod spec145_human_view_tests;

/// Garde commune de lecture/suivi : O(1), aucune maintenance ni écriture.
fn human_client_guard(
    st: &DaemonState,
    conn_id: &str,
    version: u16,
    t3_thread_id: &str,
    capability: ClientCapability,
) -> Result<(), bridget_transport::protocol::HumanThreadViewError> {
    use bridget_transport::protocol::HumanThreadViewError as E;
    if version != bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION {
        return Err(E::UnsupportedVersion);
    }
    if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Client)
        || st.conn_names.contains_key(conn_id)
        || st.auxiliary_connections.contains(conn_id)
    {
        return Err(E::BindingUnavailable);
    }
    if !st.client_negotiations.get(conn_id).is_some_and(|n| {
        n.version == CLIENT_CONTRACT_VERSION && n.capabilities.as_slice() == [capability]
    }) {
        return Err(E::UnsupportedVersion);
    }
    let lineage = matches!(
        capability,
        ClientCapability::HumanLineageViewV1
            | ClientCapability::HumanLineageWatchV1
            | ClientCapability::HumanLineageCancelV1
            | ClientCapability::HumanLineageComposeV1
    );
    if (lineage
        && (t3_thread_id.is_empty()
            || t3_thread_id.len() > 2048
            || t3_thread_id
                .chars()
                .any(bridget_core::is_disallowed_control)))
        || (!lineage && crate::threads::canonical_uuid(t3_thread_id).is_none())
    {
        return Err(E::InvalidRequest);
    }
    Ok(())
}

/// O(C), C connexions ; identité et projet viennent de la liaison primaire.
fn human_request_project(
    state: &Arc<Mutex<DaemonState>>,
    thread: &str,
    root: &str,
) -> Option<bridget_transport::protocol::CommunicationProject> {
    let (host, local) = {
        let st = state.lock().recover_poison();
        let owner = crate::t3code::stable_uuid(thread);
        let host = origin_proof::host_for_owner(&st, &owner).ok()?;
        let local = host == st.host;
        (host, local)
    };
    if local {
        crate::communication::resolve_communication_project(
            root,
            &host,
            bridget_transport::protocol::CommunicationProjectSource::T3,
            None,
        )
    } else {
        origin_proof::communication_project(
            state,
            &host,
            root,
            bridget_transport::protocol::CommunicationProjectSource::T3,
            None,
        )
        .ok()
        .flatten()
    }
}
fn human_authority_guard(
    st: &DaemonState,
    t3_thread_id: &str,
    project: &Option<bridget_transport::protocol::CommunicationProject>,
) -> Result<(String, String), bridget_transport::protocol::HumanThreadViewError> {
    use bridget_transport::protocol::HumanThreadViewError as E;
    let mut candidates = st
        .t3_thread_bindings
        .iter()
        .filter(|(_, thread)| thread.as_deref() == Some(t3_thread_id));
    let Some((owner, _)) = candidates.next() else {
        return Err(E::BindingUnavailable);
    };
    if candidates.next().is_some() {
        return Err(E::BindingUnavailable);
    }
    let agent = crate::t3code::stable_uuid(t3_thread_id);
    let instance = crate::t3code::stable_uuid(&format!("instance:{t3_thread_id}"));
    if st.auxiliary_connections.contains(owner)
        || st
            .router
            .get_agent(&agent)
            .is_none_or(|route| route.connection_id != *owner)
        || live_connection_identity(st, owner) != Some((agent.clone(), instance.clone()))
        || !st.presences.get(&instance).is_some_and(|p| {
            p.transport == "t3code"
                && match project {
                    Some(project) if project.host != st.host => {
                        p.host == project.host
                            && st
                                .conn_hosts
                                .get(owner)
                                .is_some_and(|host| host == &project.host)
                            && origin_proof::worker_is_current(st, &project.host)
                    }
                    _ => p.host == st.host,
                }
        })
    {
        return Err(E::BindingUnavailable);
    }
    if !st.t3_project_connections.contains(owner)
        || !st
            .communication_projects
            .get(owner)
            .is_some_and(|fact| !fact.2 && fact.0.is_some() && fact.0 == *project)
    {
        return Err(E::ProjectMismatch);
    }
    Ok((owner.clone(), agent))
}

/// O(C + log T + P), C connexions ; même autorité que le suivi147.
fn handle_human_thread_view(
    conn_id: &str,
    request: &bridget_transport::protocol::HumanThreadViewV1,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    use bridget_transport::protocol::HumanThreadViewResult as R;
    let refusal = |code| DaemonToWrapper::HumanThreadViewResult {
        result: R::error(code),
    };
    {
        let st = state.lock().recover_poison();
        if let Err(code) = human_client_guard(
            &st,
            conn_id,
            request.version,
            &request.t3_thread_id,
            request.request.required_capability(),
        ) {
            return refusal(code);
        }
    }
    let project = human_request_project(state, &request.t3_thread_id, &request.project_root);
    let st = state.lock().recover_poison();
    if let Err(code) = human_client_guard(
        &st,
        conn_id,
        request.version,
        &request.t3_thread_id,
        request.request.required_capability(),
    ) {
        return refusal(code);
    }
    let (_, agent) = match human_authority_guard(&st, &request.t3_thread_id, &project) {
        Ok(authority) => authority,
        Err(code) => return refusal(code),
    };
    DaemonToWrapper::HumanThreadViewResult {
        result: crate::threads::human_view(&st.store, &agent, &request.request),
    }
}

fn human_watch_guard(
    st: &DaemonState,
    conn: &str,
) -> Result<(), bridget_transport::protocol::HumanThreadViewError> {
    use bridget_transport::protocol::HumanThreadViewError as E;
    let watch = st
        .human_thread_watches
        .get(conn)
        .ok_or(E::BindingUnavailable)?;
    human_client_guard(
        st,
        conn,
        watch.request.version,
        &watch.request.t3_thread_id,
        ClientCapability::HumanThreadWatchV1,
    )?;
    let (owner, agent) = human_authority_guard(st, &watch.request.t3_thread_id, &watch.project)?;
    if owner != watch.owner || agent != watch.agent {
        return Err(E::BindingUnavailable);
    }
    Ok(())
}

/// O(log T+M+S*C), M<=16, S<=128 ; une requête de membres par mutation.
/// Aucun corps ni E/S socket et aucune requête SQL par suivi humain.
fn publish_human_thread_change(st: &mut DaemonState, thread_id: &str) {
    if let Some(notifier) = &st.mobile_notifier {
        notifier.changed();
    }
    let connections: Vec<_> = st.human_thread_watches.keys().cloned().collect();
    if connections.is_empty() {
        return;
    }
    let members = st.store.human_thread_members(thread_id);
    for conn in connections {
        let allowed = human_watch_guard(st, &conn).and_then(|()| {
            members
                .as_ref()
                .map(|members| members.contains(&st.human_thread_watches[&conn].agent))
                .map_err(|_| bridget_transport::protocol::HumanThreadViewError::StorageUnavailable)
        });
        let watch = st.human_thread_watches.get_mut(&conn).unwrap();
        match allowed {
            Ok(true) => watch.queue.changed(),
            Ok(false) => continue,
            Err(code) => watch.queue.terminate(code),
        }
        let _ = watch.notify.try_send(());
    }
}

/// Deux requêtes de métadonnées indexées au plus, sans pagination.
/// O(M+S*C), M appartenances pertinentes, S<=128. Aucun parcours de messages.
fn publish_human_name_change(st: &mut DaemonState, renamed: &HashSet<String>) {
    if st.human_thread_watches.is_empty() {
        return;
    }
    // Deux identités au plus : demandeur et éventuel détenteur éteint déplacé.
    let viewers = renamed
        .iter()
        .try_fold(HashSet::new(), |mut viewers, agent| {
            viewers.extend(st.store.human_thread_viewers_for_member(agent)?);
            Ok::<_, crate::store::StoreError>(viewers)
        });
    let connections: Vec<_> = st.human_thread_watches.keys().cloned().collect();
    for conn in connections {
        let allowed = human_watch_guard(st, &conn).and_then(|()| {
            viewers
                .as_ref()
                .map(|viewers| viewers.contains(&st.human_thread_watches[&conn].agent))
                .map_err(|_| bridget_transport::protocol::HumanThreadViewError::StorageUnavailable)
        });
        let watch = st.human_thread_watches.get_mut(&conn).unwrap();
        match allowed {
            Ok(true) => watch.queue.changed(),
            Ok(false) => continue,
            Err(code) => watch.queue.terminate(code),
        }
        let _ = watch.notify.try_send(());
    }
}

/// La file est sous verrou mutation ; son writer indépendant reste hors verrou.
/// O(S*C), S<=128 ; exécuté à changement d'autorité, jamais sur minuterie.
fn invalidate_human_authority(st: &mut DaemonState) {
    native_lineage::invalidate(st);
    let connections: Vec<_> = st.human_thread_watches.keys().cloned().collect();
    for conn in connections {
        if let Err(code) = human_watch_guard(st, &conn) {
            let watch = st.human_thread_watches.get_mut(&conn).unwrap();
            watch.queue.terminate(code);
            let _ = watch.notify.try_send(());
        }
    }
}

fn human_watch_output(
    state: std::sync::Weak<Mutex<DaemonState>>,
    conn: String,
    receiver: Receiver<()>,
    writer: Arc<Mutex<BufWriter<UnixStream>>>,
    shutdown: UnixStream,
) {
    use bridget_transport::protocol::HumanThreadWatchEvent as E;
    loop {
        let Some(state) = state.upgrade() else {
            break;
        };
        let event = {
            let mut st = state.lock().recover_poison();
            if !st.human_thread_watches.contains_key(&conn) {
                break;
            }
            #[cfg(test)]
            {
                st.human_thread_watches
                    .get_mut(&conn)
                    .unwrap()
                    .queue
                    .guard_checks += 1;
            }
            if let Err(code) = human_watch_guard(&st, &conn) {
                st.human_thread_watches
                    .get_mut(&conn)
                    .unwrap()
                    .queue
                    .terminate(code);
            }
            st.human_thread_watches
                .get_mut(&conn)
                .unwrap()
                .queue
                .pending
                .pop_front()
        };
        if let Some(event) = event {
            let mut terminal = matches!(event, E::Error { .. });
            let result = push_control_message_until_checked(
                &writer,
                &DaemonToWrapper::HumanThreadWatchEvent { event },
                Instant::now() + CANCEL_NOTIFICATION_BUDGET,
                || {
                    let st = match state.try_lock() {
                        Ok(st) => st,
                        Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                        Err(std::sync::TryLockError::WouldBlock) => {
                            return Err(std::io::ErrorKind::WouldBlock.into());
                        }
                    };
                    Ok(human_watch_guard(&st, &conn).err().map(|code| {
                        terminal = true;
                        DaemonToWrapper::HumanThreadWatchEvent {
                            event: E::error(code),
                        }
                    }))
                },
            );
            if terminal || result.is_err() {
                state
                    .lock()
                    .recover_poison()
                    .human_thread_watches
                    .remove(&conn);
                break;
            }
        } else {
            drop(state);
            // Une mutation autorisée, révocation ou EOF libère cette attente.
            // Aucun scan du contexte ou du contenu tant que rien ne change.
            if receiver.recv().is_err() {
                break;
            }
        }
    }
    let _ = shutdown.shutdown(std::net::Shutdown::Both);
}

fn handle_human_thread_watch(
    conn_id: &str,
    request: &bridget_transport::protocol::HumanThreadWatchV1,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    use bridget_transport::protocol::{HumanThreadViewError as E, HumanThreadWatchEvent};
    let refusal = |code| {
        Some(DaemonToWrapper::HumanThreadWatchEvent {
            event: HumanThreadWatchEvent::error(code),
        })
    };
    {
        let st = state.lock().recover_poison();
        if let Err(code) = human_client_guard(
            &st,
            conn_id,
            request.version,
            &request.t3_thread_id,
            ClientCapability::HumanThreadWatchV1,
        ) {
            return refusal(code);
        }
    }
    let project = human_request_project(state, &request.t3_thread_id, &request.project_root);
    let mut st = state.lock().recover_poison();
    if let Err(code) = human_client_guard(
        &st,
        conn_id,
        request.version,
        &request.t3_thread_id,
        ClientCapability::HumanThreadWatchV1,
    ) {
        return refusal(code);
    }
    let (owner, agent) = match human_authority_guard(&st, &request.t3_thread_id, &project) {
        Ok(authority) => authority,
        Err(code) => return refusal(code),
    };
    if st.human_thread_watches.len() >= HUMAN_WATCH_MAX
        || st.human_thread_watches.contains_key(conn_id)
    {
        return refusal(E::ResponseTooLarge);
    }
    let Some(writer) = st.connections.get(conn_id).cloned() else {
        return refusal(E::BindingUnavailable);
    };
    let shutdown = match writer
        .try_lock()
        .ok()
        .and_then(|writer| writer.get_ref().try_clone().ok())
    {
        Some(socket) => socket,
        None => return refusal(E::BindingUnavailable),
    };
    let (notify, receiver) = mpsc::sync_channel(1);
    st.human_thread_watches.insert(
        conn_id.into(),
        HumanThreadWatch {
            request: request.clone(),
            project,
            owner,
            agent,
            queue: HumanWatchQueue::new(),
            notify,
        },
    );
    // Inscription et ready0 précèdent toute mutation qui prend le même verrou.
    let weak = Arc::downgrade(state);
    let conn = conn_id.to_string();
    thread::spawn(move || human_watch_output(weak, conn, receiver, writer, shutdown));
    None
}

fn complete_remote_registration(st: &mut DaemonState, command_id: &str) {
    if !st.remote_native_ready.contains(command_id) {
        return;
    }
    let Some(record) = st.managed_spawns.get(command_id) else {
        return;
    };
    if record.lease.remote_placement.is_none() {
        return;
    }
    let Some(conn) = record.wrapper_conn.as_ref() else {
        return;
    };
    if st.conn_instances.get(conn) != Some(&record.lease.instance_id)
        || st.conn_names.get(conn) != Some(&record.lease.name)
    {
        return;
    }
    if st
        .fleet
        .desired_entry(&record.lease.name)
        .ok()
        .flatten()
        .is_some_and(|entry| {
            entry.command_id == record.lease.command_id
                && entry.generation == record.lease.generation
                && entry.lifecycle_state == crate::desired_state::DesiredLifecycleState::Running
        })
    {
        return;
    }
    if let Err(error) =
        st.fleet
            .register_connected(&record.lease, &record.lease.instance_id, unix_timestamp())
    {
        warn!("connexion worker natif non publiée: {error}");
    }
}

fn remote_worker_command_known(
    st: &DaemonState,
    worker: &bridget_transport::protocol::WorkerIdentity,
    command: &bridget_transport::protocol::WorkerCommandIdentity,
) -> Result<bool, bridget_transport::protocol::WorkerRefusal> {
    st.idempotency
        .spawn_command(&st.fleet.supervisor_scope(), &command.command_id)
        .map(|known| {
            known.is_some_and(|known| {
                known
                    .remote_placement
                    .as_ref()
                    .is_some_and(|placement| placement.matches_worker(worker))
                    && known.generation == command.generation
                    && known.instance_id.as_deref() == Some(command.instance_id.as_str())
            })
        })
        .map_err(|_| bridget_transport::protocol::WorkerRefusal::StoreUnavailable)
}

fn remote_worker_has_command(
    st: &DaemonState,
    worker: &bridget_transport::protocol::WorkerIdentity,
    command: Option<&bridget_transport::protocol::WorkerCommandIdentity>,
) -> bool {
    if let Some(command) = command {
        return remote_worker_command_known(st, worker, command).unwrap_or(false);
    }
    st.idempotency
        .spawn_commands(&st.fleet.supervisor_scope())
        .is_ok_and(|commands| {
            commands.iter().any(|known| {
                known
                    .remote_placement
                    .as_ref()
                    .is_some_and(|placement| placement.matches_worker(worker))
            })
        })
}

fn remote_worker_disconnected(st: &mut DaemonState, conn_id: &str) {
    use bridget_transport::protocol::WorkerContactState;
    for worker in st
        .remote_workers
        .values_mut()
        .filter(|w| w.conn_id == conn_id && w.contact != WorkerContactState::Stale)
    {
        worker.contact = WorkerContactState::Stale;
        if let Err(error) = st
            .delegation_store
            .remote_contact_changed(&worker.identity.machine_id, &worker.identity.worker_id)
        {
            warn!("worker contact invalidation failed: {error}");
        }
    }
    invalidate_human_authority(st);
    st.pending_worker_responses
        .retain(|_, pending| pending.conn_id != conn_id);
    st.pending_origin_proofs
        .retain(|_, pending| pending.conn_id != conn_id);
    st.pending_candidate_probes
        .retain(|_, pending| pending.conn_id != conn_id);
    st.pending_candidate_probes_v2
        .retain(|_, pending| pending.conn_id != conn_id);
}

fn remote_worker_contact_tick(state: &Arc<Mutex<DaemonState>>) {
    use bridget_transport::protocol::{WORKER_STALE_SECS, WorkerContactState};
    let mut st = state.lock().recover_poison();
    let stale: Vec<_> = st
        .remote_workers
        .values()
        .filter(|worker| {
            worker.contact == WorkerContactState::Connected
                && worker.last_contact.elapsed() >= Duration::from_secs(WORKER_STALE_SECS as u64)
        })
        .map(|worker| worker.conn_id.clone())
        .collect();
    for conn_id in stale {
        remote_worker_disconnected(&mut st, &conn_id);
    }
}

fn handle_worker_message(
    state: &Arc<Mutex<DaemonState>>,
    conn_id: &str,
    message: WrapperToDaemon,
) -> Option<DaemonToWrapper> {
    use bridget_transport::protocol::*;
    // Projection fermée : aucun corps de commande ni credential dans le diagnostic.
    let refused_context = match &message {
        WrapperToDaemon::WorkerResponse { response } => Some(serde_json::json!({
            "machine_id": response.worker.machine_id,
            "correlation_id": response.correlation_id,
            "command": response.command,
            "operation": match &response.response {
                WorkerResponseOperation::Observation { observation } => format!("observation_{:?}", observation.phase),
                WorkerResponseOperation::FinalResult { .. } => "final_result".into(),
                WorkerResponseOperation::Rejected { reason } => format!("rejected_{reason:?}"),
                WorkerResponseOperation::Prepared { .. } => "prepared".into(),
                WorkerResponseOperation::CommitViewRejected { reason, .. } => format!("commit_view_rejected_{reason:?}"),
                WorkerResponseOperation::CommitViewConserved { .. } => "commit_view_conserved".into(),
                _ => "other".into(),
            },
        })),
        _ => None,
    };
    let refused = |reason| {
        if matches!(
            reason,
            WorkerRefusal::CommandMismatch | WorkerRefusal::StoreUnavailable
        ) {
            eprintln!(
                "bridget_worker_refusal235 {}",
                serde_json::json!({
                    "reason":reason,"context":refused_context,
                })
            );
        }
        Some(DaemonToWrapper::WorkerRejected { reason })
    };
    let mut st = state.lock().recover_poison();
    let hello_capabilities = match &message {
        WrapperToDaemon::WorkerHello { hello } => Some((hello.clone(), Vec::new())),
        WrapperToDaemon::WorkerHelloWithCapabilities { hello } => {
            let capabilities = hello.capabilities.clone();
            (!capabilities.is_empty()
                && capabilities.len() <= 32
                && (!capabilities.contains(&WorkerCapability::OriginProjectContextV1)
                    || capabilities.contains(&WorkerCapability::OriginProofV1))
                && (!capabilities.contains(&WorkerCapability::ModelOptionsV1)
                    || capabilities.contains(&WorkerCapability::ProviderCatalogueV1))
                && (!capabilities.contains(&WorkerCapability::WorkspaceViewCreateV1)
                    || capabilities.contains(&WorkerCapability::WorkspaceViewV1))
                && capabilities.iter().all(|capability| {
                    matches!(
                        capability,
                        WorkerCapability::CandidateProbeV1
                            | WorkerCapability::CandidateProbeV2
                            | WorkerCapability::WorkloadToolsV1
                            | WorkerCapability::ReleaseLeaseV2
                            | WorkerCapability::ParentSourceProofV1
                            | WorkerCapability::ResultSlotDeadlineV1
                            | WorkerCapability::OriginProofV1
                            | WorkerCapability::OriginProjectContextV1
                            | WorkerCapability::WorkspaceRepairV1
                            | WorkerCapability::StartRecoveryV1
                            | WorkerCapability::WorkspaceFailureV1
                            | WorkerCapability::ProviderCatalogueV1
                            | WorkerCapability::ModelOptionsV1
                            | WorkerCapability::ProviderAccountsV1
                            | WorkerCapability::WorkspaceViewV1
                            | WorkerCapability::ExecutionStatusV1
                            | WorkerCapability::ProgramCompatibilityV1
                    ) || (crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED
                        && matches!(capability, WorkerCapability::WorkspaceViewCreateV1))
                })
                && capabilities
                    .iter()
                    .enumerate()
                    .all(|(index, value)| !capabilities[..index].contains(value)))
            .then(|| (hello.hello.clone(), capabilities))
        }
        _ => None,
    };
    if matches!(
        &message,
        WrapperToDaemon::WorkerHelloWithCapabilities { .. }
    ) && hello_capabilities.is_none()
    {
        return refused(WorkerRefusal::InvalidEnvelope);
    }
    if let Some((hello, capabilities)) = hello_capabilities {
        if let Err(reason) = hello.validate() {
            return refused(reason);
        }
        if st.peer_uids.get(conn_id).copied() != Some(bridget_transport::fsutil::effective_uid()) {
            return refused(WorkerRefusal::Unauthorized);
        }
        let Some(provision) = st.registry.machines().get(&hello.identity.machine_id) else {
            return refused(WorkerRefusal::Unauthorized);
        };
        if !provision.enabled && !remote_worker_has_command(&st, &hello.identity, None) {
            return refused(WorkerRefusal::Unauthorized);
        }
        if provision.worker_id != hello.identity.worker_id
            || provision.provisioning_revision != hello.identity.provisioning_revision
            || !provision.matches_credential(&hello.credential)
        {
            return refused(WorkerRefusal::Unauthorized);
        }
        if st
            .remote_workers
            .values()
            .any(|worker| worker.conn_id == conn_id)
        {
            return refused(WorkerRefusal::StaleConnection);
        }
        let old = st.remote_workers.get(&hello.identity.machine_id);
        if old.is_some_and(|worker| worker.identity != hello.identity) {
            return refused(WorkerRefusal::PlacementMismatch);
        }
        let previous_conn = old.map(|worker| worker.conn_id.clone());
        let previous_epoch = old.map(|worker| worker.epoch.clone());
        if old.is_none_or(|worker| worker.contact != WorkerContactState::Connected)
            && st
                .delegation_store
                .remote_contact_changed(&hello.identity.machine_id, &hello.identity.worker_id)
                .is_err()
        {
            return refused(WorkerRefusal::StoreUnavailable);
        }
        st.connection_roles
            .insert(conn_id.into(), ConnectionRole::Worker);
        st.worker_probe_connections.remove(conn_id);
        let epoch = uuid::Uuid::new_v4().to_string();
        st.remote_workers.insert(
            hello.identity.machine_id.clone(),
            RemoteWorkerConnection {
                identity: hello.identity.clone(),
                conn_id: conn_id.into(),
                epoch: epoch.clone(),
                capabilities: capabilities.clone(),
                catalogues: HashMap::new(),
                catalogue_refresh_id: epoch.clone(),
                catalogue_refresh_at: None,
                last_contact: Instant::now(),
                last_contact_at: unix_now_secs(),
                contact: WorkerContactState::Connected,
            },
        );
        let old_writer = previous_conn
            .as_ref()
            .and_then(|id| st.connections.get(id).cloned());
        if let Some(previous) = &previous_conn {
            st.late_worker_prepares.forget_connection(previous);
            st.pending_worker_responses
                .retain(|_, pending| &pending.conn_id != previous);
            st.pending_origin_proofs
                .retain(|_, pending| &pending.conn_id != previous);
            st.pending_candidate_probes
                .retain(|_, pending| &pending.conn_id != previous);
            st.pending_candidate_probes_v2
                .retain(|_, pending| &pending.conn_id != previous);
        }
        drop(st);
        if let Some(previous_epoch) = previous_epoch {
            info!(
                "worker_connection_replaced machine_id={} worker_id={} previous_epoch={} connection_epoch={}",
                hello.identity.machine_id, hello.identity.worker_id, previous_epoch, epoch
            );
        }
        // Le remplacement est déjà publié ; fermer le socket hors verrou global.
        if let Some(writer) = old_writer {
            let _ = writer
                .lock()
                .recover_poison()
                .get_ref()
                .shutdown(std::net::Shutdown::Both);
        }
        let accepted = WorkerAccepted {
            version: REMOTE_WORKER_VERSION,
            identity: hello.identity,
            connection_epoch: epoch,
        };
        return Some(if capabilities.is_empty() {
            DaemonToWrapper::WorkerAccepted { accepted }
        } else {
            DaemonToWrapper::WorkerAcceptedWithCapabilities {
                accepted: WorkerAcceptedWithCapabilities {
                    accepted,
                    capabilities,
                },
            }
        });
    }
    let Some(machine) = st
        .remote_workers
        .values()
        .find(|worker| worker.conn_id == conn_id)
        .map(|worker| worker.identity.machine_id.clone())
    else {
        return refused(WorkerRefusal::Unauthorized);
    };
    match message {
        WrapperToDaemon::ProviderCatalogueUpdate { update } => {
            native_provider_catalogue::ingest(&mut st, conn_id, update)
        }
        WrapperToDaemon::Heartbeat => {
            let worker = &st.remote_workers[&machine];
            if worker.contact != WorkerContactState::Connected
                && st
                    .delegation_store
                    .remote_contact_changed(&machine, &worker.identity.worker_id)
                    .is_err()
            {
                return refused(WorkerRefusal::StoreUnavailable);
            }
            let worker = st.remote_workers.get_mut(&machine).unwrap();
            worker.contact = WorkerContactState::Connected;
            worker.last_contact = Instant::now();
            worker.last_contact_at = unix_now_secs();
            None
        }
        WrapperToDaemon::WorkerResponse { response } => {
            if let Err(reason) = response.validate() {
                return refused(reason);
            }
            let worker = &st.remote_workers[&machine];
            if response.connection_epoch != worker.epoch || response.worker != worker.identity {
                return refused(WorkerRefusal::StaleConnection);
            }
            if let WorkerResponseOperation::FinalResult { result } = &response.response {
                match native_delegation::capture_remote_result(&mut st, &response.worker, result) {
                    Ok(disposition) => {
                        return Some(DaemonToWrapper::WorkerRequest {
                            request: WorkerRequest {
                                version: REMOTE_WORKER_VERSION,
                                correlation_id: response.correlation_id,
                                connection_epoch: response.connection_epoch,
                                worker: response.worker,
                                command: response.command,
                                request: WorkerRequestOperation::ResultDisposition {
                                    result: WorkerResultDisposition {
                                        identity: result.identity.clone(),
                                        result_digest: result.digest.clone(),
                                        disposition,
                                    },
                                },
                            },
                        });
                    }
                    Err(reason) => return refused(reason),
                }
            }
            if let WorkerResponseOperation::Observation { observation } = &response.response {
                // La reconnexion republie aussi les refus des anciennes tentatives.
                // Un tuple historique exact, déjà connu de Fleet, ne modifie
                // ni la tentative courante ni la connexion authentifiée.
                if observation.phase == WorkerCommandPhase::Refused
                    && remote_worker_has_command(&st, &response.worker, Some(&response.command))
                    && st
                        .delegation_store
                        .get(&response.command.task_id)
                        .is_ok_and(|task| {
                            task.is_some_and(|task| {
                                native_delegation::remote_command_identity(&st, &task)
                                    .is_ok_and(|current| current != response.command)
                                    && task.start_recovery.as_ref().is_some_and(|plan| {
                                        plan.previous == response.command
                                            || plan
                                                .history
                                                .iter()
                                                .any(|old| old.command == response.command)
                                    })
                            })
                        })
                {
                    // Même une ancienne observation peut être la réponse à une
                    // lecture déjà en vol. Ne jamais absorber son attente corrélée.
                    if let Some(pending) = st.pending_worker_responses.get(&response.correlation_id)
                    {
                        if pending.conn_id != conn_id
                            || pending.epoch != response.connection_epoch
                            || pending.worker != response.worker
                            || pending.command != response.command
                        {
                            return refused(WorkerRefusal::CommandMismatch);
                        }
                        let pending = st
                            .pending_worker_responses
                            .remove(&response.correlation_id)
                            .unwrap();
                        let _ = pending.reply.try_send(response.response);
                    }
                    return None;
                }
                match remote_worker_command_known(&st, &response.worker, &response.command) {
                    Ok(true) => {}
                    Ok(false) => return refused(WorkerRefusal::CommandMismatch),
                    Err(reason) => return refused(reason),
                }
                // Une commande gérée hors délégation porte son propre command_id
                // comme task_id. Son tuple durable exact reste obligatoire ;
                // une délégation existante garde son contrôle d’identité complet.
                let command_matches = match st.delegation_store.get(&response.command.task_id) {
                    Ok(Some(task)) => {
                        match native_delegation::remote_command_identity(&st, &task) {
                            Ok(identity) => identity == response.command,
                            Err(WorkerRefusal::StoreUnavailable) => {
                                return refused(WorkerRefusal::StoreUnavailable);
                            }
                            Err(_) => false,
                        }
                    }
                    Ok(None) => response.command.task_id == response.command.command_id,
                    Err(_) => return refused(WorkerRefusal::StoreUnavailable),
                };
                if !command_matches {
                    return refused(WorkerRefusal::CommandMismatch);
                }
                if let Some(failure) = observation.startup_failure
                    && let Ok(Some(mut task)) = st.delegation_store.get(&response.command.task_id)
                    && task.state == "starting"
                    && task.mission_issued_at.is_none()
                {
                    let detail = match observation.exit_code {
                        Some(code) => format!("remote_start_failed:{}:exit={code}", failure.code()),
                        None => format!("remote_start_failed:{}", failure.code()),
                    };
                    task.error = Some(detail);
                    if st.delegation_store.save(&task).is_err() {
                        return refused(WorkerRefusal::StoreUnavailable);
                    }
                }
                if matches!(
                    observation.phase,
                    WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                ) && let Some(lease) = st
                    .managed_spawns
                    .get(&response.command.command_id)
                    .map(|record| record.lease.clone())
                {
                    if lease.instance_id != response.command.instance_id
                        || lease.generation != response.command.generation
                    {
                        return refused(WorkerRefusal::CommandMismatch);
                    }
                    if st.fleet.invalidate_for_stop(&lease).is_err() {
                        return refused(WorkerRefusal::StoreUnavailable);
                    }
                    st.managed_spawns.remove(&lease.command_id);
                    st.managed_by_instance.remove(&lease.instance_id);
                    st.remote_native_ready.remove(&lease.command_id);
                    st.managed_terminal_instances.insert(lease.instance_id);
                }
                if observation.native_ready && observation.phase == WorkerCommandPhase::Released {
                    st.remote_native_ready
                        .insert(response.command.command_id.clone());
                    complete_remote_registration(&mut st, &response.command.command_id);
                }
            }
            if matches!(
                response.response,
                WorkerResponseOperation::CommitViewRejected { .. }
            ) && !st.remote_workers[&machine]
                .capabilities
                .contains(&WorkerCapability::WorkspaceViewCreateV1)
            {
                return refused(WorkerRefusal::CapabilityRequired);
            }
            if matches!(
                response.response,
                WorkerResponseOperation::CommitViewConserved { .. }
                    | WorkerResponseOperation::CommitViewFile { .. }
            ) && !st.remote_workers[&machine]
                .capabilities
                .contains(&WorkerCapability::WorkspaceViewV1)
            {
                return refused(WorkerRefusal::CapabilityRequired);
            }
            if matches!(
                response.response,
                WorkerResponseOperation::WorkspaceRejected { .. }
            ) && !st.remote_workers[&machine]
                .capabilities
                .contains(&WorkerCapability::WorkspaceFailureV1)
            {
                return refused(WorkerRefusal::CapabilityRequired);
            }
            let Some(pending) = st.pending_worker_responses.get(&response.correlation_id) else {
                if st.late_worker_prepares.ignores(conn_id, &response) {
                    eprintln!(
                        "bridget_late_prepare240 {}",
                        serde_json::json!({
                        "machine_id": response.worker.machine_id,
                        "correlation_id": response.correlation_id,
                        "event": "expired_prepare_response_ignored"})
                    );
                    return None;
                }
                // Une réponse de préparation tardive n'est pas une commande Fleet.
                // Identité et forme sont déjà vérifiées ; aucun effet de tâche.
                // PrepareDiagnosed exclut PrepareFenced. L'unique Prepare initial
                // émis par delegate_remote_with_routing porte command_id=task_id ;
                // une autre commande sans preuve durable reste refusée.
                if matches!(
                    response.response,
                    WorkerResponseOperation::WorkspaceRejected { .. }
                ) && st
                    .delegation_store
                    .get(&response.command.task_id)
                    .ok()
                    .flatten()
                    .is_some_and(|task| {
                        task.task_id == response.command.command_id
                            && task
                                .remote_placement
                                .as_ref()
                                .is_some_and(|p| p.matches_worker(&response.worker))
                    })
                {
                    return None;
                }
                // ACK tardif de conservation : pas d'effet de Task ni preuve
                // nouvelle. Seule une liaison connue exactement est tolérée.
                if let WorkerResponseOperation::CommitViewConserved { prepare_digest } =
                    &response.response
                    && st
                        .delegation_store
                        .get(&response.command.task_id)
                        .ok()
                        .flatten()
                        .is_some_and(|task| {
                            commit_view_conservation_matches(
                                &task,
                                &response.worker,
                                &response.command,
                                prepare_digest,
                            )
                        })
                {
                    return None;
                }
                // Une conservation refusée après expiration n'a aucun effet. Le tuple
                // doit encore désigner la vue exacte d'une tâche close, hors continuation.
                if matches!(&response.response, WorkerResponseOperation::Rejected { .. })
                    && st.remote_workers[&machine]
                        .capabilities
                        .contains(&WorkerCapability::WorkspaceViewV1)
                    && st
                        .delegation_store
                        .get(&response.command.task_id)
                        .ok()
                        .flatten()
                        .is_some_and(|task| {
                            task.commit_view_binding().is_some_and(|binding| {
                                commit_view_conservation_matches(
                                    &task,
                                    &response.worker,
                                    &response.command,
                                    &binding.prepare_digest,
                                )
                            })
                        })
                {
                    return None;
                }
                // Une sonde sans attente ou une réponse tardive ne ferme pas le canal de sa commande connue.
                if matches!(
                    &response.response,
                    WorkerResponseOperation::Observation { .. }
                        | WorkerResponseOperation::DispositionRecorded { .. }
                        | WorkerResponseOperation::Released {}
                        | WorkerResponseOperation::Rejected { .. }
                        | WorkerResponseOperation::StartFenced {}
                        | WorkerResponseOperation::StartRejected { .. }
                ) && remote_worker_has_command(&st, &response.worker, Some(&response.command))
                {
                    return None;
                }

                // Les résultats spontanés seront raccordés à l'ingress durable T023.
                return refused(WorkerRefusal::CommandMismatch);
            };
            if pending.conn_id != conn_id
                || pending.epoch != response.connection_epoch
                || pending.worker != response.worker
                || pending.command != response.command
            {
                return refused(WorkerRefusal::CommandMismatch);
            }
            let pending = st
                .pending_worker_responses
                .remove(&response.correlation_id)
                .unwrap();
            let _ = pending.reply.try_send(response.response);
            None
        }
        WrapperToDaemon::OriginProofResponse { response } => {
            origin_proof::receive(&mut st, conn_id, response)
        }
        WrapperToDaemon::CandidateProbeResult { result } => {
            let worker = &st.remote_workers[&machine];
            if !worker
                .capabilities
                .contains(&WorkerCapability::CandidateProbeV1)
                || result.connection_epoch != worker.epoch
                || result.worker != worker.identity
                || result.validate_shape().is_err()
            {
                return refused(WorkerRefusal::StaleConnection);
            }
            let Some(pending) = st.pending_candidate_probes.get(&result.correlation_id) else {
                // Sonde expirée ou déjà consommée : ignorer ce résultat en lecture,
                // comme T3 résout une réponse dont l’attente a été retirée.
                // L’époque, l’identité et la forme ont déjà été contrôlées.
                return None;
            };
            let result_ids = result
                .verdicts
                .iter()
                .map(|verdict| match verdict {
                    CandidateProbeVerdict::Eligible { candidate_id, .. }
                    | CandidateProbeVerdict::Ineligible { candidate_id, .. } => candidate_id,
                })
                .collect::<std::collections::BTreeSet<_>>();
            if pending.conn_id != conn_id
                || pending.epoch != result.connection_epoch
                || pending.worker != result.worker
                || pending.admission_id != result.admission_id
                || pending.policy_revision != result.policy_revision
                || pending.candidate_ids.len() != result_ids.len()
                || !pending
                    .candidate_ids
                    .iter()
                    .all(|candidate_id| result_ids.contains(candidate_id))
            {
                return refused(WorkerRefusal::CommandMismatch);
            }
            let pending = st
                .pending_candidate_probes
                .remove(&result.correlation_id)
                .expect("la sonde vient d'être trouvée");
            let _ = pending.reply.try_send(result);
            None
        }
        WrapperToDaemon::CandidateProbeResultV2 { result } => {
            let worker = &st.remote_workers[&machine];
            if !worker
                .capabilities
                .contains(&WorkerCapability::CandidateProbeV2)
                || result.connection_epoch != worker.epoch
                || result.worker != worker.identity
                || result.validate_shape().is_err()
            {
                return refused(WorkerRefusal::StaleConnection);
            }
            let Some(pending) = st.pending_candidate_probes_v2.get(&result.correlation_id) else {
                // Sonde expirée ou déjà consommée : ignorer ce résultat en lecture,
                // comme T3 résout une réponse dont l’attente a été retirée.
                // L’époque, l’identité et la forme ont déjà été contrôlées.
                return None;
            };
            let result_ids = result
                .verdicts
                .iter()
                .map(|verdict| match verdict {
                    CandidateProbeV2Verdict::Eligible { candidate_id, .. }
                    | CandidateProbeV2Verdict::Ineligible { candidate_id, .. } => candidate_id,
                })
                .collect::<std::collections::BTreeSet<_>>();
            if pending.conn_id != conn_id
                || pending.epoch != result.connection_epoch
                || pending.worker != result.worker
                || pending.admission_id != result.admission_id
                || pending.policy_revision != result.policy_revision
                || pending.candidate_ids.len() != result_ids.len()
                || !pending
                    .candidate_ids
                    .iter()
                    .all(|candidate_id| result_ids.contains(candidate_id))
            {
                return refused(WorkerRefusal::CommandMismatch);
            }
            let pending = st
                .pending_candidate_probes_v2
                .remove(&result.correlation_id)
                .expect("la sonde v2 vient d'être trouvée");
            let _ = pending.reply.try_send(result);
            None
        }
        _ => refused(WorkerRefusal::Unauthorized),
    }
}

thread_local! {
    /// Cause locale du dernier ResourceLimit du maître sur ce thread (198) :
    /// aucune E/S n'a atteint le worker. Lue puis effacée par l'appelant.
    static LOCAL_ROUNDTRIP_LIMIT: std::cell::Cell<Option<&'static str>> =
        const { std::cell::Cell::new(None) };
}

/// Cause fermée d'un refus produit par le maître lui-même, sinon `None`.
pub(crate) fn take_local_roundtrip_limit() -> Option<&'static str> {
    LOCAL_ROUNDTRIP_LIMIT.with(|cell| cell.take())
}

/// Une conservation d'espace ne découle ni d'un résultat lu ni d'une commande
/// inconnue. Arrêts/gardes et rapport sont déjà clos, sans suite possible ; le
/// dernier bit cleanup_done attend encore l'accusé de conservation du worker.
fn commit_view_conservation_matches(
    task: &crate::delegation::Task,
    worker: &bridget_transport::protocol::WorkerIdentity,
    command: &bridget_transport::protocol::WorkerCommandIdentity,
    prepare_digest: &str,
) -> bool {
    task.terminal()
        && !task.continuation_guard_open()
        && (task.workspace.is_none()
            || task.workspace_report_mission.as_deref() == Some(task.mission.as_str()))
        && (!task.guarded || task.guard_released)
        && task.task_id == command.task_id
        && task
            .remote_placement
            .as_ref()
            .is_some_and(|p| p.matches_worker(worker))
        && task.commit_view_binding().is_some_and(|binding| {
            binding.worker == *worker
                && binding.command == *command
                && binding.prepare_digest == prepare_digest
        })
}

fn remote_worker_roundtrip(
    state: &Arc<Mutex<DaemonState>>,
    worker: &bridget_transport::protocol::WorkerIdentity,
    command: &bridget_transport::protocol::WorkerCommandIdentity,
    operation: bridget_transport::protocol::WorkerRequestOperation,
    deadline: Instant,
) -> Result<
    bridget_transport::protocol::WorkerResponseOperation,
    bridget_transport::protocol::WorkerRefusal,
> {
    remote_worker_roundtrip_with_send_deadline(
        state, worker, command, operation, deadline, deadline,
    )
}

// The response budget must not retain the shared writer during a slow read-only probe.
fn remote_worker_roundtrip_with_send_deadline(
    state: &Arc<Mutex<DaemonState>>,
    worker: &bridget_transport::protocol::WorkerIdentity,
    command: &bridget_transport::protocol::WorkerCommandIdentity,
    operation: bridget_transport::protocol::WorkerRequestOperation,
    deadline: Instant,
    send_deadline: Instant,
) -> Result<
    bridget_transport::protocol::WorkerResponseOperation,
    bridget_transport::protocol::WorkerRefusal,
> {
    use bridget_transport::protocol::*;
    LOCAL_ROUNDTRIP_LIMIT.with(|cell| cell.set(None));
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::sync_channel(1);
    let (writer, request, conn_id) = {
        let mut st = state.lock().recover_poison();
        let provision = st
            .registry
            .machines()
            .get(&worker.machine_id)
            .ok_or(WorkerRefusal::Unauthorized)?;
        let conserving =
            if let WorkerRequestOperation::ConserveCommitView { prepare_digest } = &operation {
                let known = st
                    .delegation_store
                    .get(&command.task_id)
                    .ok()
                    .flatten()
                    .is_some_and(|task| {
                        commit_view_conservation_matches(&task, worker, command, prepare_digest)
                    });
                if !known {
                    return Err(WorkerRefusal::Unauthorized);
                }
                true
            } else {
                false
            };
        if !provision.enabled
            && matches!(
                &operation,
                WorkerRequestOperation::Prepare { .. }
                    | WorkerRequestOperation::PrepareDiagnosed { .. }
                    | WorkerRequestOperation::PrepareV2 { .. }
                    | WorkerRequestOperation::Start { .. }
                    | WorkerRequestOperation::Release { .. }
                    | WorkerRequestOperation::ReserveResultSlot { .. }
                    | WorkerRequestOperation::ReserveResultSlotUntil { .. }
            )
        {
            return Err(WorkerRefusal::Unauthorized);
        }
        if !provision.enabled
            && !conserving
            && !remote_worker_has_command(&st, worker, Some(command))
        {
            return Err(WorkerRefusal::CommandMismatch);
        }
        if provision.worker_id != worker.worker_id
            || provision.provisioning_revision != worker.provisioning_revision
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        let session = st
            .remote_workers
            .get(&worker.machine_id)
            .ok_or(WorkerRefusal::WorkerUnavailable)?;
        if session.identity != *worker
            || session.contact != WorkerContactState::Connected
            || session.last_contact.elapsed() >= Duration::from_secs(WORKER_STALE_SECS as u64)
        {
            return Err(WorkerRefusal::WorkerUnavailable);
        }
        if matches!(&operation, WorkerRequestOperation::PrepareDiagnosed { .. })
            && !session
                .capabilities
                .contains(&WorkerCapability::WorkspaceFailureV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        let routed_prepare = matches!(&operation, WorkerRequestOperation::PrepareV2 { .. })
            || matches!(&operation, WorkerRequestOperation::PrepareDiagnosed {prepare}
                if matches!(prepare.as_ref(), WorkerRequestOperation::PrepareV2 {..}));
        if routed_prepare
            && !session
                .capabilities
                .contains(&WorkerCapability::CandidateProbeV2)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if matches!(&operation, WorkerRequestOperation::ExecutionStatus {})
            && !session
                .capabilities
                .contains(&WorkerCapability::ExecutionStatusV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if operation.requires_provider_accounts()
            && !session
                .capabilities
                .contains(&WorkerCapability::ProviderAccountsV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if operation.requires_model_options()
            && !session
                .capabilities
                .contains(&WorkerCapability::ModelOptionsV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if operation.requires_commit_view_creation()
            && (!crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED
                || !session
                    .capabilities
                    .contains(&WorkerCapability::WorkspaceViewCreateV1))
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if operation.requires_commit_view_reader()
            && !session
                .capabilities
                .contains(&WorkerCapability::WorkspaceViewV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if matches!(&operation, WorkerRequestOperation::RepairWorkspace { .. })
            && !session
                .capabilities
                .contains(&WorkerCapability::WorkspaceRepairV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if matches!(&operation, WorkerRequestOperation::ParentSourceProof { .. })
            && !session
                .capabilities
                .contains(&WorkerCapability::ParentSourceProofV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if matches!(
            &operation,
            WorkerRequestOperation::ReserveResultSlotUntil { .. }
        ) && !session
            .capabilities
            .contains(&WorkerCapability::ResultSlotDeadlineV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        // Reprise198 : un ancien pair ne reçoit jamais ces enveloppes.
        if matches!(
            &operation,
            WorkerRequestOperation::StartDetailed { .. }
                | WorkerRequestOperation::FenceUnstarted { .. }
                | WorkerRequestOperation::PrepareFenced { .. }
        ) && !session
            .capabilities
            .contains(&WorkerCapability::StartRecoveryV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        let writer = st
            .connections
            .get(&session.conn_id)
            .cloned()
            .ok_or(WorkerRefusal::WorkerUnavailable)?;
        let conn_id = session.conn_id.clone();
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION,
            correlation_id: correlation_id.clone(),
            connection_epoch: session.epoch.clone(),
            worker: worker.clone(),
            command: command.clone(),
            request: operation,
        };
        if let Err(reason) = request.validate() {
            if reason == WorkerRefusal::ResourceLimit {
                LOCAL_ROUNDTRIP_LIMIT.with(|cell| cell.set(Some("master_request_frame")));
            }
            return Err(reason);
        }
        if st.pending_worker_responses.len() >= MAX_WORKER_STORE_ENTRIES {
            LOCAL_ROUNDTRIP_LIMIT.with(|cell| cell.set(Some("master_pending")));
            return Err(WorkerRefusal::ResourceLimit);
        }
        st.pending_worker_responses.insert(
            correlation_id.clone(),
            PendingWorkerResponse {
                conn_id: conn_id.clone(),
                epoch: request.connection_epoch.clone(),
                worker: worker.clone(),
                command: command.clone(),
                reply: tx,
            },
        );
        (writer, request, conn_id)
    };
    let epoch = request.connection_epoch.clone();
    let initial_prepare = late_worker_prepares::initial_prepare(command, &request.request);
    let written = push_control_message_until(
        &writer,
        &DaemonToWrapper::WorkerRequest { request },
        send_deadline.min(deadline),
    )
    .is_ok();
    let response = if written {
        rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .ok()
    } else {
        None
    };
    let mut st = state.lock().recover_poison();
    st.pending_worker_responses.remove(&correlation_id);
    // Un contact périmé peut avoir retiré l'attente sans fermer le socket.
    // L'histoire exige toujours une requête écrite et la même connexion/époque.
    if written
        && initial_prepare
        && response.is_none()
        && st.connections.contains_key(&conn_id)
        && st
            .remote_workers
            .get(&worker.machine_id)
            .is_some_and(|current| {
                current.conn_id == conn_id && current.epoch == epoch && current.identity == *worker
            })
    {
        st.late_worker_prepares
            .remember(correlation_id.clone(), &conn_id, &epoch, worker, command);
    }
    if !st
        .remote_workers
        .get(&worker.machine_id)
        .is_some_and(|current| {
            current.conn_id == conn_id && current.epoch == epoch && current.identity == *worker
        })
    {
        return Err(WorkerRefusal::StaleConnection);
    }
    match response.ok_or(WorkerRefusal::WorkerUnavailable)? {
        WorkerResponseOperation::Rejected { reason } => Err(reason),
        response => Ok(response),
    }
}

/// Une lecture groupée163 attend un seul résultat corrélé. Elle n'emprunte ni
/// `WorkerPrepare` ni le canal de commandes : aucun worktree, fournisseur,
/// slot ou Task ne peut donc être créé avant la décision durable.
fn remote_candidate_probe(
    state: &Arc<Mutex<DaemonState>>,
    mut probe: bridget_transport::protocol::CandidateProbe,
    deadline: Instant,
) -> Result<
    bridget_transport::protocol::CandidateProbeResult,
    bridget_transport::protocol::WorkerRefusal,
> {
    use bridget_transport::protocol::{
        MAX_WORKER_STORE_ENTRIES, WORKER_STALE_SECS, WorkerCapability, WorkerContactState,
        WorkerRefusal,
    };
    probe.validate_shape()?;
    let correlation_id = probe.correlation_id.clone();
    let machine_id = probe.worker.machine_id.clone();
    let candidate_ids = probe
        .candidates
        .iter()
        .map(|candidate| candidate.candidate_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let (tx, rx) = mpsc::sync_channel(1);
    let (writer, conn_id, epoch) = {
        let mut st = state.lock().recover_poison();
        let provision = st
            .registry
            .machines()
            .get(&probe.worker.machine_id)
            .ok_or(WorkerRefusal::Unauthorized)?;
        if !provision.enabled
            || provision.worker_id != probe.worker.worker_id
            || provision.provisioning_revision != probe.worker.provisioning_revision
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        let session = st
            .remote_workers
            .get(&probe.worker.machine_id)
            .ok_or(WorkerRefusal::WorkerUnavailable)?;
        if session.identity != probe.worker
            || session.contact != WorkerContactState::Connected
            || session.last_contact.elapsed() >= Duration::from_secs(WORKER_STALE_SECS as u64)
        {
            return Err(WorkerRefusal::WorkerUnavailable);
        }
        if !session
            .capabilities
            .contains(&WorkerCapability::CandidateProbeV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if st.pending_candidate_probes.len() >= MAX_WORKER_STORE_ENTRIES
            || st.pending_candidate_probes.contains_key(&correlation_id)
        {
            return Err(WorkerRefusal::ResourceLimit);
        }
        let writer = st
            .connections
            .get(&session.conn_id)
            .cloned()
            .ok_or(WorkerRefusal::WorkerUnavailable)?;
        let conn_id = session.conn_id.clone();
        let epoch = session.epoch.clone();
        probe.connection_epoch = epoch.clone();
        st.pending_candidate_probes.insert(
            correlation_id.clone(),
            PendingCandidateProbe {
                conn_id: conn_id.clone(),
                epoch: epoch.clone(),
                worker: probe.worker.clone(),
                admission_id: probe.admission_id.clone(),
                policy_revision: probe.policy_revision,
                candidate_ids,
                reply: tx,
            },
        );
        (writer, conn_id, epoch)
    };
    let written = push_control_message_until(
        &writer,
        &DaemonToWrapper::CandidateProbe { probe },
        deadline,
    )
    .is_ok();
    let result = if written {
        rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => WorkerRefusal::ProbeExpired,
                mpsc::RecvTimeoutError::Disconnected => WorkerRefusal::WorkerUnavailable,
            })
    } else {
        Err(WorkerRefusal::WorkerUnavailable)
    };
    let mut st = state.lock().recover_poison();
    st.pending_candidate_probes.remove(&correlation_id);
    if !st
        .remote_workers
        .get(&machine_id)
        .is_some_and(|current| current.conn_id == conn_id && current.epoch == epoch)
    {
        return Err(WorkerRefusal::StaleConnection);
    }
    result
}

/// Sonde168 corrélée au worker. Elle ne passe jamais par Prepare et ne porte
/// aucune réservation ; le relevé de ressources retourné reste le fait du
/// worker, pas une valeur inventée par le maître.
fn remote_candidate_probe_v2(
    state: &Arc<Mutex<DaemonState>>,
    mut probe: bridget_transport::protocol::CandidateProbeV2,
    deadline: Instant,
) -> Result<
    bridget_transport::protocol::CandidateProbeV2Result,
    bridget_transport::protocol::WorkerRefusal,
> {
    use bridget_transport::protocol::{
        DaemonToWrapper, MAX_WORKER_STORE_ENTRIES, WORKER_STALE_SECS, WorkerCapability,
        WorkerContactState, WorkerRefusal,
    };
    probe.validate_shape()?;
    let correlation_id = probe.correlation_id.clone();
    let machine_id = probe.worker.machine_id.clone();
    let candidate_ids = probe
        .candidates
        .iter()
        .map(|candidate| candidate.candidate_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let (tx, rx) = mpsc::sync_channel(1);
    let (writer, conn_id, epoch) = {
        let mut st = state.lock().recover_poison();
        let provision = st
            .registry
            .machines()
            .get(&probe.worker.machine_id)
            .ok_or(WorkerRefusal::Unauthorized)?;
        if !provision.enabled
            || provision.worker_id != probe.worker.worker_id
            || provision.provisioning_revision != probe.worker.provisioning_revision
        {
            return Err(WorkerRefusal::PlacementMismatch);
        }
        let session = st
            .remote_workers
            .get(&probe.worker.machine_id)
            .ok_or(WorkerRefusal::WorkerUnavailable)?;
        if !session
            .capabilities
            .contains(&WorkerCapability::CandidateProbeV2)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if probe.candidates.iter().any(|candidate| {
            st.registry
                .get(&candidate.agent_type)
                .is_ok_and(|definition| definition.capabilities.account_profiles)
        }) && !session
            .capabilities
            .contains(&WorkerCapability::ProviderAccountsV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if !probe.required_tools.is_empty()
            && !session
                .capabilities
                .contains(&WorkerCapability::WorkloadToolsV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if probe
            .candidates
            .iter()
            .any(|candidate| candidate.service_tier.is_some())
            && !session
                .capabilities
                .contains(&WorkerCapability::ModelOptionsV1)
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if probe.candidates.iter().any(|candidate| {
            matches!(
                candidate.workspace,
                Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
            )
        }) && (!crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED
            || !session
                .capabilities
                .contains(&WorkerCapability::WorkspaceViewCreateV1)
            || !session
                .capabilities
                .contains(&WorkerCapability::WorkspaceViewV1))
        {
            return Err(WorkerRefusal::CapabilityRequired);
        }
        if session.identity != probe.worker
            || session.contact != WorkerContactState::Connected
            || session.last_contact.elapsed() >= Duration::from_secs(WORKER_STALE_SECS as u64)
            || !session
                .capabilities
                .contains(&WorkerCapability::CandidateProbeV2)
        {
            return Err(WorkerRefusal::WorkerUnavailable);
        }
        if st.pending_candidate_probes_v2.len() >= MAX_WORKER_STORE_ENTRIES
            || st.pending_candidate_probes_v2.contains_key(&correlation_id)
        {
            return Err(WorkerRefusal::ResourceLimit);
        }
        let writer = st
            .connections
            .get(&session.conn_id)
            .cloned()
            .ok_or(WorkerRefusal::WorkerUnavailable)?;
        let conn_id = session.conn_id.clone();
        let epoch = session.epoch.clone();
        probe.connection_epoch = epoch.clone();
        st.pending_candidate_probes_v2.insert(
            correlation_id.clone(),
            PendingCandidateProbeV2 {
                conn_id: conn_id.clone(),
                epoch: epoch.clone(),
                worker: probe.worker.clone(),
                admission_id: probe.admission_id.clone(),
                policy_revision: probe.policy_revision,
                candidate_ids,
                reply: tx,
            },
        );
        (writer, conn_id, epoch)
    };
    let written = push_control_message_until(
        &writer,
        &DaemonToWrapper::CandidateProbeV2 { probe },
        deadline,
    )
    .is_ok();
    let result = if written {
        rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => WorkerRefusal::ProbeExpired,
                mpsc::RecvTimeoutError::Disconnected => WorkerRefusal::WorkerUnavailable,
            })
    } else {
        Err(WorkerRefusal::WorkerUnavailable)
    };
    let mut st = state.lock().recover_poison();
    st.pending_candidate_probes_v2.remove(&correlation_id);
    if !st
        .remote_workers
        .get(&machine_id)
        .is_some_and(|current| current.conn_id == conn_id && current.epoch == epoch)
    {
        return Err(WorkerRefusal::StaleConnection);
    }
    result
}

#[cfg(test)]
fn handle_wrapper_message(
    conn_id: &str,
    msg: WrapperToDaemon,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    handle_wrapper_message_with_peer(conn_id, msg, state, None)
}

fn update_activity(state: &Arc<Mutex<DaemonState>>) -> Option<crate::updates::Activity> {
    let gate = state.lock().recover_poison().update_gate.clone();
    gate.enter()
}
fn update_exempt(msg: &WrapperToDaemon) -> bool {
    matches!(
        msg,
        WrapperToDaemon::MobileAdministration { .. }
            | WrapperToDaemon::RoleHandshake { .. }
            | WrapperToDaemon::ClientHello { .. }
            | WrapperToDaemon::DaemonIdentityRequest
    ) || matches!(msg,WrapperToDaemon::MobileOperation{request} if matches!(request.operation,bridget_transport::protocol::MobileOperation::UpdatesStatus|bridget_transport::protocol::MobileOperation::UpdatesConfigure{..}))
}
fn handle_wrapper_message_with_peer(
    conn_id: &str,
    msg: WrapperToDaemon,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
) -> Option<DaemonToWrapper> {
    let _update_activity = if update_exempt(&msg) {
        None
    } else {
        match update_activity(state) {
            Some(guard) => Some(guard),
            None => return Some(DaemonToWrapper::UpdateInProgress),
        }
    };
    {
        let mut st = state.lock().recover_poison();
        if native_lineage::invalid_message(&mut st, conn_id) {
            return None;
        }
        if let Some(watch) = st.human_thread_watches.get_mut(conn_id) {
            watch
                .queue
                .terminate(bridget_transport::protocol::HumanThreadViewError::InvalidRequest);
            let _ = watch.notify.try_send(());
            return None;
        }
    }
    if let WrapperToDaemon::Specialists { request } = &msg {
        return Some(specialist_requests::handle_human(conn_id, request, state));
    }
    if let WrapperToDaemon::AgentTracking { request } = &msg {
        return Some(agent_tracking::human(conn_id, request, state));
    }
    if let WrapperToDaemon::UserInput { request } = &msg {
        return Some(user_input::handle(conn_id, request, state, peer));
    }
    if let WrapperToDaemon::MobileOperation { request } = &msg {
        return Some(mobile_operations::handle(conn_id, request.clone(), state));
    }
    if let WrapperToDaemon::MobileAdministration { request } = &msg {
        return Some(mobile_admin::handle(conn_id, request.clone(), state));
    }
    if let WrapperToDaemon::HumanLineage { request } = &msg {
        return native_lineage::handle(conn_id, request, state);
    }
    // Cette consultation n'exécute aucune maintenance d'attach, même refusée.
    if let WrapperToDaemon::HumanThreadWatchV1 { request } = &msg {
        return handle_human_thread_watch(conn_id, request, state);
    }
    if let WrapperToDaemon::HumanThreadViewV1 { request } = &msg {
        return Some(handle_human_thread_view(conn_id, request, state));
    }
    // Sonde seule : aucun principal n'est acquis par l'identité publique du daemon.
    if matches!(&msg, WrapperToDaemon::DaemonIdentityRequest) {
        let mut st = state.lock().recover_poison();
        if !st.connection_roles.contains_key(conn_id) {
            st.worker_probe_connections.insert(conn_id.into());
        }
    } else if matches!(
        &msg,
        WrapperToDaemon::WorkerHello { .. } | WrapperToDaemon::WorkerHelloWithCapabilities { .. }
    ) {
        let allowed = {
            let st = state.lock().recover_poison();
            matches!(
                st.connection_roles.get(conn_id),
                None | Some(ConnectionRole::Worker)
            ) || (st.worker_probe_connections.contains(conn_id)
                && st.connection_roles.get(conn_id) == Some(&ConnectionRole::Wrapper)
                && !st.conn_names.contains_key(conn_id)
                && !st.identity_credentials.contains_key(conn_id)
                && !st.client_negotiations.contains_key(conn_id)
                && !st.service_negotiations.contains_key(conn_id))
        };
        if allowed {
            return handle_worker_message(state, conn_id, msg);
        }
    } else {
        state
            .lock()
            .recover_poison()
            .worker_probe_connections
            .remove(conn_id);
    }

    if !state
        .lock()
        .recover_poison()
        .connection_roles
        .contains_key(conn_id)
    {
        match &msg {
            // Un hello client ne doit jamais sélectionner implicitement le rôle
            // wrapper : l'ordre public est strict et sans effet de bord.
            WrapperToDaemon::ClientHello { .. } => {
                return Some(DaemonToWrapper::ClientRejected {
                    reason: ClientRefusal::RoleHandshakeRequired,
                });
            }
            // Le nom réservé `guichet` ne donne aucun droit : seule une
            // connexion explicitement négociée comme service peut le demander.
            WrapperToDaemon::ServiceHello { .. } => {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::RoleHandshakeRequired,
                });
            }
            _ => {}
        }
    }

    if !matches!(msg, WrapperToDaemon::RoleHandshake { .. }) {
        // La première commande non négociée choisit définitivement la
        // compatibilité wrapper. Une tentative d'upgrade ultérieure vers
        // attach ne doit jamais élargir une connexion déjà active.
        let mut st = state.lock().recover_poison();
        st.connection_roles
            .entry(conn_id.to_string())
            .or_insert(ConnectionRole::Wrapper);
    }

    if let WrapperToDaemon::RoleHandshake { role } = &msg {
        let mut st = state.lock().recover_poison();
        if st.connection_roles.contains_key(conn_id) {
            match role {
                ConnectionRole::Worker => {
                    return Some(DaemonToWrapper::WorkerRejected {
                        reason: bridget_transport::protocol::WorkerRefusal::Unauthorized,
                    });
                }
                ConnectionRole::Client => {
                    return Some(DaemonToWrapper::ClientRejected {
                        reason: ClientRefusal::ClientRoleRequired,
                    });
                }
                ConnectionRole::Service => {
                    return Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::ServiceRoleRequired,
                    });
                }
                ConnectionRole::Wrapper | ConnectionRole::Attach => {}
            }
            return Some(DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::MessageOutsideAttachRole,
                mode: None,
                location: None,
            });
        }
        st.connection_roles.insert(conn_id.to_string(), *role);
        return Some(DaemonToWrapper::RoleAccepted { role: *role });
    }

    if state.lock().recover_poison().connection_roles.get(conn_id) == Some(&ConnectionRole::Worker)
    {
        return handle_worker_message(state, conn_id, msg);
    }

    let attach_refusal = {
        let st = state.lock().recover_poison();
        let queue_attach = st.connection_roles.get(conn_id) == Some(&ConnectionRole::Attach)
            && !st.conn_names.contains_key(conn_id)
            && st.connections.contains_key(conn_id)
            && match &msg {
                WrapperToDaemon::ClientHello { capabilities, .. } => {
                    !capabilities.is_empty()
                        && capabilities.iter().all(|capability| {
                            matches!(
                                capability,
                                ClientCapability::QueueListV1 | ClientCapability::QueueEditV1
                            )
                        })
                }
                WrapperToDaemon::QueueList { .. } | WrapperToDaemon::QueueEditRequest { .. } => {
                    true
                }
                _ => false,
            };
        if queue_attach {
            None
        } else {
            (st.connection_roles.get(conn_id) == Some(&ConnectionRole::Attach))
                .then(|| msg.attach_refusal())
                .flatten()
        }
    };
    if let Some(reason) = attach_refusal {
        return Some(DaemonToWrapper::AttachRejected {
            subscription_id: None,
            reason,
            mode: None,
            location: None,
        });
    }

    let service_refusal = wrapper_dispatch::service_refusal(&msg, conn_id, state);
    if let Some(reason) = service_refusal {
        return Some(DaemonToWrapper::ServiceRejected { reason });
    }

    let client_refusal = wrapper_dispatch::client_refusal(&msg, conn_id, state);
    if let Some(reason) = client_refusal {
        return Some(DaemonToWrapper::ClientRejected { reason });
    }

    match msg {
        // Le contrat est disponible ; T008 ajoute son admission authentifiée.
        // Aucune trame distante ne rejoint entre-temps la voie locale.
        WrapperToDaemon::WorkerHello { .. }
        | WrapperToDaemon::WorkerHelloWithCapabilities { .. }
        | WrapperToDaemon::WorkerResponse { .. }
        | WrapperToDaemon::CandidateProbeResult { .. }
        | WrapperToDaemon::CandidateProbeResultV2 { .. }
        | WrapperToDaemon::ProviderCatalogueUpdate { .. }
        | WrapperToDaemon::OriginProofResponse { .. } => Some(DaemonToWrapper::WorkerRejected {
            reason: bridget_transport::protocol::WorkerRefusal::Unauthorized,
        }),
        WrapperToDaemon::WakeRequest { request } => Some(handle_wake_request(
            conn_id,
            request,
            state,
            unix_now_secs(),
        )),
        WrapperToDaemon::QueueEditRequest {
            request_id,
            target,
            action,
            message_id,
            before_message_id,
        } => wrapper_dispatch::on_queue_edit_request(
            request_id,
            target,
            action,
            message_id,
            before_message_id,
            conn_id,
            state,
        ),
        WrapperToDaemon::QueueList { target } => {
            let st = state.lock().recover_poison();
            Some(handle_guidance_queue(&st, conn_id, &target))
        }
        WrapperToDaemon::CommunicationProjectFact {
            root,
            source,
            host,
            worktree_root,
        } => wrapper_dispatch::on_communication_project_fact(root, source, host, worktree_root, conn_id, state),
        WrapperToDaemon::DirectoryScoped { scope } => {
            wrapper_dispatch::on_directory_scoped(scope, conn_id, state)
        }
        WrapperToDaemon::ObservationGap { dropped } => {
            wrapper_dispatch::on_observation_gap(dropped, conn_id, state)
        }
        WrapperToDaemon::ObservationCapabilities {
            events,
            runtime_catalogue,
        } => {
            wrapper_dispatch::on_observation_capabilities(events, runtime_catalogue, conn_id, state)
        }
        WrapperToDaemon::HumanLineage { request } => {
            native_lineage::handle(conn_id, &request, state)
        }
        WrapperToDaemon::HumanThreadViewV1 { request } => {
            Some(handle_human_thread_view(conn_id, &request, state))
        }
        WrapperToDaemon::HumanThreadWatchV1 { request } => {
            handle_human_thread_watch(conn_id, &request, state)
        }
        WrapperToDaemon::T3ThreadBindingFact {
            version,
            t3_thread_id,
        } => wrapper_dispatch::on_t3_thread_binding_fact(version, t3_thread_id, conn_id, state),
        WrapperToDaemon::ThreadNoticeCapability { versions } => {
            wrapper_dispatch::on_thread_notice_capability(versions, conn_id, state)
        }
        WrapperToDaemon::ThreadRequest { request } => {
            wrapper_dispatch::on_thread_request(request, conn_id, state)
        }
        // Session 104 : lecture seule hors verrou. Identité et permis sous
        // verrou bref ; SQL et repli sur le thread de connexion ; identité
        // revérifiée avant publication ; permis relâché par RAII.
        WrapperToDaemon::LedgerSearch { request } => {
            wrapper_dispatch::on_ledger_search(request, conn_id, state)
        }
        WrapperToDaemon::LedgerRead { request } => {
            wrapper_dispatch::on_ledger_read(request, conn_id, state)
        }
        WrapperToDaemon::ObservationRequest { request } => {
            wrapper_dispatch::on_observation_request(request, conn_id, state)
        }
        WrapperToDaemon::ObservedActivity { seq, event, file } => {
            wrapper_dispatch::on_observed_activity(seq, event, file, conn_id, state)
        }
        WrapperToDaemon::RoleHandshake { .. } => unreachable!("handshake traité avant le dispatch"),
        WrapperToDaemon::DelegationOverviewRead {
            version,
            state: filter,
            limit,
        } => Some(handle_delegation_overview(
            state, conn_id, version, &filter, limit,
        )),
        WrapperToDaemon::DelegationCancel {
            version,
            task_id,
            request_id,
        } => Some(handle_delegation_cancel(
            state,
            conn_id,
            version,
            &task_id,
            &request_id,
        )),
        WrapperToDaemon::DelegationHaltSet {
            version,
            command_id,
            halted,
            reason,
            cancel_active,
        } => wrapper_dispatch::on_delegation_halt_set(
            version,
            command_id,
            halted,
            reason,
            cancel_active,
            conn_id,
            state,
        ),
        WrapperToDaemon::ControlStateRead { version } => {
            Some(handle_control_state_read(state, version))
        }
        WrapperToDaemon::ControlFocusRead { version } => {
            Some(handle_control_focus_read(state, version))
        }
        WrapperToDaemon::ControlHistory { version, limit } => {
            Some(handle_control_history(state, version, limit))
        }
        WrapperToDaemon::ControlStateSet {
            version,
            command_id,
            expected_generation,
            paused,
            auto_objectives_cap,
            reason,
            agent_posture,
            auto_reassignment,
        } => wrapper_dispatch::on_control_state_set(
            version,
            command_id,
            expected_generation,
            paused,
            auto_objectives_cap,
            reason,
            agent_posture,
            auto_reassignment,
            conn_id,
            state,
        ),
        WrapperToDaemon::HumanInboxDeposit {
            version,
            dedup_key,
            kind,
            subject,
            context,
            options,
        } => Some(handle_human_inbox_deposit(
            state, version, &dedup_key, kind, &subject, &context, &options,
        )),
        WrapperToDaemon::ControlFocusPublish { version, focus } => {
            Some(handle_control_focus_publish(state, version, focus))
        }
        WrapperToDaemon::HumanInboxList {
            version,
            state: filter,
            limit,
        } => Some(handle_human_inbox_list(
            state, conn_id, version, filter, limit,
        )),
        WrapperToDaemon::HumanInboxResolve {
            version,
            command_id: _,
            item_id,
            choice,
        } => Some(handle_human_inbox_resolve(
            state, conn_id, version, &item_id, &choice,
        )),
        WrapperToDaemon::HumanInboxDecisions { version, limit } => {
            Some(handle_human_inbox_decisions(state, version, limit))
        }
        WrapperToDaemon::HumanInboxAck {
            version,
            decision_id,
        } => Some(handle_human_inbox_ack(state, version, &decision_id)),
        WrapperToDaemon::HumanInboxClose {
            version,
            item_id,
            reason,
        } => Some(handle_human_inbox_close(state, version, &item_id, &reason)),
        WrapperToDaemon::RuntimeIngressHello { .. }
        | WrapperToDaemon::RuntimeIngressPreflight { .. } => {
            Some(DaemonToWrapper::RuntimeIngressRejected {
                reason: RuntimeIngressRefusal::ReservationMissing,
            })
        }
        WrapperToDaemon::ArtifactRead { request } => {
            wrapper_dispatch::on_artifact_read(request, conn_id, state)
        }
        WrapperToDaemon::ArtifactPublish {
            contract_version,
            canonical_publication,
        } => wrapper_dispatch::on_artifact_publish(
            contract_version,
            canonical_publication,
            conn_id,
            state,
        ),
        // Surface historique fermée : aucun moteur, aucune écriture de
        // projet. Les données de liaison restent seulement l'autorité ACL des
        // contenus déjà publiés, jamais une autorisation de lancer sur l'hôte.
        WrapperToDaemon::ProjectRegistryRequest { request } => Some(project_registry_failure(
            request,
            ProjectRegistryRefusal::ProjectRootPolicyUnavailable,
            unix_now_secs(),
        )),
        WrapperToDaemon::ProjectRegistryAdminRequest { request } => {
            Some(project_registry_admin_failure(
                &request,
                ProjectRegistryRefusal::ProjectRootPolicyUnavailable,
                unix_now_secs(),
            ))
        }
        WrapperToDaemon::ProjectSystemRequest { request } => {
            wrapper_dispatch::on_project_system_request(request)
        }
        WrapperToDaemon::ProjectRoundRequest { request } => Some(project_round_failure(
            &request,
            ProjectRoundRefusal::PolicyDisabled,
            unix_now_secs(),
        )),
        WrapperToDaemon::ProjectRoundDispatch { request } => Some(project_round_dispatch_failure(
            &request,
            ProjectRoundRefusal::PolicyDisabled,
            unix_now_secs(),
        )),
        WrapperToDaemon::ProjectProfileRequest { request } => Some(project_profile_failure(
            &request,
            ProjectProfileRefusal::CatalogUnavailable,
            unix_now_secs(),
        )),
        WrapperToDaemon::ProjectRuntimeRequest { request } => Some(project_runtime_failure(
            &request,
            ProjectRuntimeRefusal::PolicyUnavailable,
            unix_now_secs(),
        )),
        WrapperToDaemon::DelegatedRuntimeEvent {
            execution_id,
            kind,
            code,
            reference,
        } => wrapper_dispatch::on_delegated_runtime_event(
            execution_id,
            kind,
            code,
            reference,
            conn_id,
            state,
        ),
        WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id } => {
            wrapper_dispatch::on_delegated_runtime_event_acknowledged(event_id, conn_id, state)
        }
        WrapperToDaemon::WaitAgentLinks {
            after_cursor,
            timeout_ms,
        } => wrapper_dispatch::on_wait_agent_links(after_cursor, timeout_ms, conn_id, state),
        WrapperToDaemon::ServiceHello {
            version,
            service,
            issuer_scope,
            capabilities,
        } => wrapper_dispatch::on_service_hello(
            version,
            service,
            issuer_scope,
            capabilities,
            conn_id,
            state,
        ),
        WrapperToDaemon::CoordinationSubscribe {
            version,
            after_cursor,
        } => wrapper_dispatch::on_coordination_subscribe(version, after_cursor, conn_id, state),
        WrapperToDaemon::ServiceRequest {
            version,
            issuer_scope,
            request_id,
            issued_at,
            from,
            to,
            operation,
            payload,
        } => wrapper_dispatch::on_service_request(
            version,
            issuer_scope,
            request_id,
            issued_at,
            from,
            to,
            operation,
            payload,
            conn_id,
            state,
        ),
        WrapperToDaemon::GuichetClaimNext { version } => {
            wrapper_dispatch::on_guichet_claim_next(version, conn_id, state)
        }
        WrapperToDaemon::GuichetClaim {
            version,
            issuer_scope,
            request_id,
            claim_token,
        } => wrapper_dispatch::on_guichet_claim(
            version,
            issuer_scope,
            request_id,
            claim_token,
            conn_id,
            state,
        ),
        WrapperToDaemon::GuichetLookup {
            version,
            issuer_scope,
            request_id,
        } => wrapper_dispatch::on_guichet_lookup(version, issuer_scope, request_id, state),
        WrapperToDaemon::GuichetReply {
            version,
            issuer_scope,
            request_id,
            claim_generation,
            claim_token,
            response_message_id,
            in_reply_to,
            outcome,
            payload,
        } => wrapper_dispatch::on_guichet_reply(
            version,
            issuer_scope,
            request_id,
            claim_generation,
            claim_token,
            response_message_id,
            in_reply_to,
            outcome,
            payload,
            conn_id,
            state,
        ),
        WrapperToDaemon::ClientHello {
            contract_version,
            issuer_scope,
            capabilities,
        } => wrapper_dispatch::on_client_hello(
            contract_version,
            issuer_scope,
            capabilities,
            conn_id,
            state,
        ),
        WrapperToDaemon::RegisterAuxiliary {
            agent_id,
            instance_id,
            credential,
        } => wrapper_dispatch::on_register_auxiliary(
            agent_id,
            instance_id,
            credential,
            conn_id,
            state,
        ),
        WrapperToDaemon::SendIdempotent {
            message,
            message_id,
            issued_at,
        } => wrapper_dispatch::on_send_idempotent(message, message_id, issued_at, conn_id, state),
        WrapperToDaemon::Lookup {
            operation_kind,
            idempotency_key,
        } => {
            let st = state.lock().recover_poison();
            Some(handle_idempotency_lookup(
                conn_id,
                operation_kind,
                idempotency_key,
                &st,
            ))
        }
        WrapperToDaemon::ControlExecution { command } => {
            wrapper_dispatch::on_control_execution(command, conn_id, state)
        }
        WrapperToDaemon::ControlExecutionReported {
            issuer_scope,
            command_id,
            execution_id,
            accepted,
            refusal_reason,
        } => wrapper_dispatch::on_control_execution_reported(
            issuer_scope,
            command_id,
            execution_id,
            accepted,
            refusal_reason,
            conn_id,
            state,
        ),
        WrapperToDaemon::DeliverAcked {
            delivery_id,
            delivery_generation,
        } => {
            let mut st = state.lock().recover_poison();
            handle_delivery_ack(conn_id, delivery_id, delivery_generation, &mut st)
        }
        WrapperToDaemon::DeliveryIndeterminate {
            delivery_id,
            delivery_generation,
        } => wrapper_dispatch::on_delivery_indeterminate(
            delivery_id,
            delivery_generation,
            conn_id,
            state,
        ),
        WrapperToDaemon::RequestWait {
            request_id,
            wait_timeout,
        } => Some(wait_request(
            conn_id,
            &request_id,
            wait_timeout,
            state,
            peer,
            Duration::from_secs(5),
        )),
        WrapperToDaemon::NativeDelegation { request } => Some(native_delegation::handle_with_peer(
            conn_id, request, state, peer,
        )),
        WrapperToDaemon::NativeDelegationT3 { request, proof } => Some(
            native_delegation::handle_attested(conn_id, request, &proof, state, peer),
        ),
        WrapperToDaemon::NativePermissionFact {
            fact,
            request_id,
            observation_id,
            unavailable_code,
        } => wrapper_dispatch::on_native_permission_fact(
            fact,
            request_id,
            observation_id,
            unavailable_code,
            conn_id,
            state,
        ),
        WrapperToDaemon::SpawnOrder {
            posture,
            agent_type,
            project,
            agent_id,
            cwd,
            persistent,
            command_id,
            issued_at,
            deadline_at,
            ownership,
        } => wrapper_dispatch::on_spawn_order(
            posture,
            agent_type,
            project,
            agent_id,
            cwd,
            persistent,
            command_id,
            issued_at,
            deadline_at,
            ownership,
            conn_id,
            state,
        ),
        WrapperToDaemon::StopOrder {
            agent_id: name,
            command_id,
        } => wrapper_dispatch::on_stop_order(name, command_id, state),
        WrapperToDaemon::RelaunchOrder {
            agent_id: name,
            command_id,
        } => wrapper_dispatch::on_relaunch_order(name, command_id, conn_id, state),
        WrapperToDaemon::DecommissionOrder {
            agent_id: name,
            command_id,
        } => wrapper_dispatch::on_decommission_order(name, command_id, state),
        WrapperToDaemon::AdoptStoppedOrder {
            agent_id: name,
            command_id,
        } => wrapper_dispatch::on_adopt_stopped_order(name, command_id, state),
        WrapperToDaemon::Subscribe { agent, window } => {
            wrapper_dispatch::on_subscribe(agent, window, conn_id, state)
        }
        WrapperToDaemon::Unsubscribe { subscription_id } => {
            wrapper_dispatch::on_unsubscribe(subscription_id, conn_id, state)
        }
        WrapperToDaemon::Subscribed { subscription_id } => {
            wrapper_dispatch::on_subscribed(subscription_id, conn_id, state)
        }
        WrapperToDaemon::AttachRejected {
            subscription_id,
            reason,
        } => wrapper_dispatch::on_attach_rejected(subscription_id, reason, conn_id, state),
        WrapperToDaemon::End {
            subscription_id,
            reason,
        } => wrapper_dispatch::on_end(subscription_id, reason, conn_id, state),
        WrapperToDaemon::JournalReadError {
            subscription_id,
            line,
            offset,
            reason,
        } => wrapper_dispatch::on_journal_read_error(
            subscription_id,
            line,
            offset,
            reason,
            conn_id,
            state,
        ),
        WrapperToDaemon::JournalFragment {
            subscription_id,
            seq,
            offset,
            final_fragment,
            bytes,
        } => wrapper_dispatch::on_journal_fragment(
            subscription_id,
            seq,
            offset,
            final_fragment,
            bytes,
            conn_id,
            state,
        ),
        WrapperToDaemon::LiveJournalFragment {
            seq,
            offset,
            final_fragment,
            bytes,
        } => wrapper_dispatch::on_live_journal_fragment(
            seq,
            offset,
            final_fragment,
            bytes,
            conn_id,
            state,
        ),
        WrapperToDaemon::SnapshotCaughtUp {
            subscription_id,
            through_seq,
        } => wrapper_dispatch::on_snapshot_caught_up(subscription_id, through_seq, conn_id, state),
        WrapperToDaemon::Gap {
            subscription_id,
            from_seq,
            to_seq,
            reason,
        } => wrapper_dispatch::on_gap(subscription_id, from_seq, to_seq, reason, conn_id, state),
        WrapperToDaemon::Register {
            identity_version,
            agent_type,
            agent_id,
            host,
            transport,
            channel,
            mode,
            location,
            os,
            instance_id,
            domain,
            turn_in_progress,
            journal_available,
            turn_attest,
            queue_fallback,
        } => wrapper_dispatch::on_register(
            identity_version,
            agent_type,
            agent_id,
            host,
            transport,
            channel,
            mode,
            location,
            os,
            instance_id,
            domain,
            turn_in_progress,
            journal_available,
            turn_attest,
            queue_fallback,
            conn_id,
            state,
        ),

        WrapperToDaemon::DiskSpace { fact } => {
            wrapper_dispatch::on_disk_space(fact, conn_id, state)
        }

        WrapperToDaemon::JournalReady { receipt_probe } => {
            wrapper_dispatch::on_journal_ready(receipt_probe, conn_id, state)
        }

        WrapperToDaemon::TerminalSessionReady => {
            wrapper_dispatch::on_terminal_session_ready(conn_id, state)
        }

        WrapperToDaemon::Unregister => wrapper_dispatch::on_unregister(conn_id, state),

        WrapperToDaemon::SelectRuntime { agent, selection } => Some(select_runtime_from_attach(
            state, conn_id, &agent, selection,
        )),
        WrapperToDaemon::RuntimeSelectionReported { token, outcome } => wrapper_dispatch::on_runtime_selection_reported(token, outcome, conn_id, state),
        WrapperToDaemon::RuntimeCatalogueReported { token, outcome } => wrapper_dispatch::on_runtime_catalogue_reported(token, outcome, conn_id, state),
        WrapperToDaemon::Send(bridge_msg) => wrapper_dispatch::on_send(bridge_msg, conn_id, state),

        WrapperToDaemon::Heartbeat => {
            let mut st = state.lock().recover_poison();
            if let Some(instance_id) = st.conn_instances.get(conn_id).cloned()
                && let Some(presence) = st.presences.get_mut(&instance_id)
            {
                // Lien ≠ capacité : le heartbeat prouve le socket, pas l'exécution.
                presence.touch_link();
            }
            None
        }

        WrapperToDaemon::TurnState { in_progress } => {
            wrapper_dispatch::on_turn_state(in_progress, conn_id, state)
        }

        WrapperToDaemon::ExecutionStateChanged { transition } => {
            let mut st = state.lock().recover_poison();
            handle_execution_transition(conn_id, transition, &mut st)
        }

        WrapperToDaemon::WorkspaceRepairResult { task_id, status } => {
            let mut st = state.lock().recover_poison();
            native_delegation::workspace_repair::response(&mut st, conn_id, &task_id, status);
            None
        }
        WrapperToDaemon::ExecutionProviderObserved { context } => {
            let mut st = state.lock().recover_poison();
            handle_execution_provider_context(conn_id, context, &mut st)
        }

        WrapperToDaemon::DaemonIdentityRequest => {
            wrapper_dispatch::on_daemon_identity_request(state)
        }

        WrapperToDaemon::SentinelStatusRead => {
            let path = { state.lock().recover_poison().db_path.clone() };
            Some(DaemonToWrapper::SentinelStatus {
                report: crate::sentinel::status(path.parent().unwrap()),
            })
        }

        WrapperToDaemon::ListAgents => {
            let mut st = state.lock().recover_poison();
            let agents = st.agent_infos();
            Some(DaemonToWrapper::AgentList { agents })
        }
        WrapperToDaemon::DisplayNameResolve { request } => {
            wrapper_dispatch::on_display_name_resolve(request, state)
        }
        WrapperToDaemon::DisplayNameSet { request } => {
            wrapper_dispatch::on_display_name_set(request, conn_id, state)
        }
        WrapperToDaemon::Runtime {
            agent,
            model,
            effort,
            source,
        } => wrapper_dispatch::on_runtime(agent, model, effort, source, conn_id, state),

        WrapperToDaemon::ServedModel { agent, model } => {
            let mut st = state.lock().recover_poison();
            Some(handle_served_model(&agent, model, &mut st))
        }

        WrapperToDaemon::RateLimit {
            agent,
            window,
            status,
            resets_at,
            used_percent,
            credits,
            source,
        } => wrapper_dispatch::on_rate_limit(
            agent,
            window,
            status,
            resets_at,
            used_percent,
            credits,
            source,
            state,
        ),

        WrapperToDaemon::Usage {
            agent,
            input_tokens,
            execution_id,
            execution_generation,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            provider_kind,
            source,
        } => wrapper_dispatch::on_usage(
            agent,
            input_tokens,
            execution_id,
            execution_generation,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            provider_kind,
            source,
            state,
        ),

        WrapperToDaemon::UsageWindow {
            agent,
            from_secs,
            to_secs,
        } => {
            let st = state.lock().recover_poison();
            Some(handle_usage_window(&agent, from_secs, to_secs, &st))
        }

        WrapperToDaemon::Domain { agent, domain } => wrapper_dispatch::on_domain(agent, domain, conn_id, state),

        WrapperToDaemon::Availability { agent, until_secs } => {
            wrapper_dispatch::on_availability(agent, until_secs, conn_id, state)
        }

        WrapperToDaemon::DeliveryRejected {
            id,
            reason,
            execution_id,
            generation,
        } => wrapper_dispatch::on_delivery_rejected(
            id,
            reason,
            execution_id,
            generation,
            conn_id,
            state,
        ),

        WrapperToDaemon::CancelRequest { id, sender, reason } => {
            wrapper_dispatch::on_cancel_request(id, sender, reason, conn_id, state)
        }

        WrapperToDaemon::ListRequests { sender, limit } => {
            wrapper_dispatch::on_list_requests(sender, limit, state)
        }

        WrapperToDaemon::UserInput { request } => {
            Some(user_input::handle(conn_id, &request, state, None))
        }
        WrapperToDaemon::ChildRequest { request } => {
            let primary = {
                let st = state.lock().recover_poison();
                user_input::primary_request(conn_id, &request, &st)
            };
            if primary {
                Some(user_input::receive_native(conn_id, request, state))
            } else {
                native_child_requests::receive(conn_id, request, state)
            }
        }
        WrapperToDaemon::ChildRequestAnswered {
            request_id,
            applied,
            reason,
        } => {
            wrapper_dispatch::on_child_request_answered(request_id, applied, reason, conn_id, state)
        }
        WrapperToDaemon::ChildRequestClosed {
            request_id,
            state: closed_state,
        } => {
            let st = state.lock().recover_poison();
            if user_input::native_ack(conn_id, &request_id, None, Some(closed_state), &st) {
                return None;
            }
            native_child_requests::closed(conn_id, &request_id, closed_state, &st);
            None
        }

        // Le contrôle de rôle mobile est exécuté avant cette matrice.
        WrapperToDaemon::AgentTracking { .. }
        | WrapperToDaemon::Specialists { .. }
        | WrapperToDaemon::MobileAdministration { .. }
        | WrapperToDaemon::MobileOperation { .. } => None,
        WrapperToDaemon::LedgerProjection { scope, limit } => wrapper_dispatch::on_ledger_projection(scope, limit, state),
    }
}

/// Statut du daemon — interroge le daemon via la socket locale.
pub fn get_status(config: &DaemonConfig) -> Result<DaemonStatus, String> {
    get_status_until(config, std::time::Instant::now() + Duration::from_secs(10))
}

fn get_status_until(
    config: &DaemonConfig,
    deadline: std::time::Instant,
) -> Result<DaemonStatus, String> {
    use crate::communication::client::DaemonConnection;

    let identity = match daemon_identity_until(
        &config.socket_path,
        deadline.min(std::time::Instant::now() + DAEMON_IDENTITY_READ_TIMEOUT),
    )? {
        Some(identity) => identity,
        None => return Ok(daemon_absent_status()),
    };
    // Identité, connexion et inventaire consomment le même budget. Une seconde
    // connexion muette ne doit ni suspendre status ni annoncer « aucun agent ».
    let mut connection = match DaemonConnection::connect_until(&config.socket_path, deadline) {
        Ok(connection) => connection,
        Err(_) => return Ok(daemon_inventory_unavailable(&identity)),
    };
    let probe_id = uuid::Uuid::new_v4().to_string();
    let registration = WrapperToDaemon::Register {
        agent_type: "status-probe".to_string(),
        identity_version: 2,
        agent_id: probe_id.clone(),
        host: Some(crate::build_info::local_host()),
        transport: None,
        channel: ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
        turn_attest: false,
        queue_fallback: false,
    };
    if !matches!(
        connection.exchange(&registration),
        Ok(DaemonToWrapper::Registered { credential: _, agent_id }) if agent_id == probe_id
    ) {
        return Ok(daemon_inventory_unavailable(&identity));
    }
    let agents = match connection.exchange(&WrapperToDaemon::ListAgents) {
        Ok(DaemonToWrapper::AgentList { agents }) => agents
            .into_iter()
            .filter(|agent| agent.agent_id != probe_id)
            .collect(),
        _ => return Ok(daemon_inventory_unavailable(&identity)),
    };
    Ok(DaemonStatus {
        running: true,
        agents,
        agents_inventory_available: true,
        // Le protocole ne publie pas de total exhaustif. Une lecture locale
        // créait/migrait SQLite et présentait au plus 1000 lignes comme total.
        // Ne pas inventer ce chiffre, même si le daemon paraît être local.
        message_count: None,
        build_id: Some(identity.build_id),
        daemon_host: identity.host,
        daemon_db_path: identity.db_path,
    })
}

const BUILD_ID_PROBE_IDENTITY: &str = "bridget-status-build-id";
const DAEMON_IDENTITY_READ_TIMEOUT: Duration = Duration::from_secs(2);

fn build_id_probe_issuer_scope() -> String {
    crate::communication::issuer_scope(BUILD_ID_PROBE_IDENTITY)
}

/// Matrice message-role : la sonde d'identite doit rester atteignable.
///
/// Ce module est DELIBEREMENT hors de `presence_tests` : cette famille est
/// ecartee de toutes nos mesures a cause d'un test qui bloque, et un oracle
/// range dans une famille que personne ne joue ne garde rien.
#[cfg(test)]
#[path = "daemon_matrice_roles_tests.rs"]
mod matrice_roles_tests;

/// Le compte de messages n'est mesurable d'ici que si la base locale est
/// EXACTEMENT celle que le daemon atteste. Sans attestation, il ne l'est pas :
/// une absence n'autorise pas à compter dans le fichier qu'on a sous la main.
pub(crate) fn daemon_store_is_local(
    daemon_host: Option<&str>,
    daemon_db_path: Option<&str>,
    local_host: &str,
    local_db_path: &std::path::Path,
) -> bool {
    // MÊME CHEMIN N'EST PAS MÊME MACHINE. Deux hôtes Linux portent couramment
    // le même `/home/moi/.cache/bridget/bridget.db` : comparer les chemins seuls
    // déclarait locale une base qui vit ailleurs, et faisait lire le fichier d'à
    // côté comme s'il décrivait le daemon. L'hôte était disponible et écarté de
    // la décision — le système savait nommer la machine, il ne l'attribuait pas.
    //
    // Source UNIQUE : `get_status` et la carte de reprise passent tous deux ici,
    // pour que deux vues du même système ne puissent pas diverger.
    // UNE ABSENCE N'EST PAS UNE IDENTITE, ET DEUX ABSENCES NE SONT PAS EGALES.
    // `local_host` retombe sur une sentinelle quand la machine ne peut pas etre
    // determinee. Deux machines DIFFERENTES qui echouent toutes deux a se nommer
    // rendent alors la MEME chaine — et avec le chemin de base standard,
    // identique partout, l egalite les declarerait locales l une pour l autre.
    // C'est la charge « meme chemin n'est pas meme machine » qui revient par la
    // porte du repli : on ferme ici les hotes INDETERMINES, pas seulement les
    // hotes DIFFERENTS.
    bridget_core::host_is_attested(local_host)
        && daemon_host
            .is_some_and(|host| bridget_core::host_is_attested(host) && host == local_host)
        && daemon_db_path.is_some_and(|path| std::path::Path::new(path) == local_db_path)
}

#[cfg(test)]
mod attribution_tests {
    use super::daemon_store_is_local;
    use std::path::Path;

    /// POINT 1 — la garde qui empêche de compter dans la base d'à côté.
    ///
    /// Éprouvée sur la décision elle-même, et non à travers `get_status` : un
    /// premier oracle passait par une socket absente, donc `get_status` sortait
    /// AVANT la garde et le test restait vert même sans elle.
    ///
    /// DEUX CONTRÔLES OPPOSÉS SUR LE MÊME CHEMIN, c'est le cœur du test :
    /// même chemin + hôtes distincts doit rendre FAUX ; même chemin + même hôte
    /// doit rendre VRAI. Un seul des deux ne prouverait rien — une garde qui
    /// refuserait toujours passerait le premier, une garde qui ignorerait l'hôte
    /// passerait le second.
    ///
    /// Mutant qui tue ce test : retirer la comparaison d'hôtes → le premier cas,
    /// deux machines partageant le même chemin, devient vrai et meurt.
    #[test]
    fn compter_exige_la_meme_machine_et_le_meme_chemin() {
        let chemin = Path::new("/home/moi/.cache/bridget/bridget.db");

        // MÊME CHEMIN, HÔTES DISTINCTS — le cas que deux Linux produisent tout
        // seuls. La base nommée existe ici ET là-bas ; elle n'est pas la même.
        assert!(
            !daemon_store_is_local(
                Some("monordinateur"),
                Some("/home/moi/.cache/bridget/bridget.db"),
                "cartae",
                chemin
            ),
            "même chemin sur deux machines distinctes ne doit PAS être local"
        );

        // MÊME CHEMIN, MÊME HÔTE — contrôle positif opposé.
        assert!(
            daemon_store_is_local(
                Some("cartae"),
                Some("/home/moi/.cache/bridget/bridget.db"),
                "cartae",
                chemin
            ),
            "même chemin sur la même machine DOIT être local"
        );

        // MÊME HÔTE, CHEMINS DISTINCTS — l'hôte seul ne suffit pas non plus.
        assert!(
            !daemon_store_is_local(Some("cartae"), Some("/autre/bridget.db"), "cartae", chemin),
            "un autre chemin sur la même machine n'est pas cette base"
        );

        // DEUX HOTES INDETERMINES, MEME CHEMIN — signale par rc7. Les deux
        // cotes rendent la sentinelle de repli ; l'egalite ne prouve alors rien
        // du tout, et le cas courant est justement le meme chemin partout.
        assert!(
            !daemon_store_is_local(
                Some(bridget_core::HOTE_NON_ATTESTE),
                Some("/home/moi/.cache/bridget/bridget.db"),
                bridget_core::HOTE_NON_ATTESTE,
                chemin
            ),
            "deux machines indeterminees ne sont pas la meme machine"
        );
        // Et le cas mixte : un cote atteste, l'autre non.
        assert!(!daemon_store_is_local(
            Some(bridget_core::HOTE_NON_ATTESTE),
            Some("/home/moi/.cache/bridget/bridget.db"),
            "cartae",
            chemin
        ));
        assert!(!daemon_store_is_local(
            Some("cartae"),
            Some("/home/moi/.cache/bridget/bridget.db"),
            bridget_core::HOTE_NON_ATTESTE,
            chemin
        ));

        // Daemon antérieur : rien d'attesté, donc rien à compter.
        assert!(!daemon_store_is_local(None, None, "cartae", chemin));
        assert!(!daemon_store_is_local(
            Some("cartae"),
            None,
            "cartae",
            chemin
        ));
        assert!(!daemon_store_is_local(
            None,
            Some("/home/moi/.cache/bridget/bridget.db"),
            "cartae",
            chemin
        ));
    }
}

/// Ce que le daemon atteste de LUI-MÊME au client qui l'interroge.
#[derive(Debug, Clone, Default)]
pub struct DaemonIdentity {
    pub build_id: String,
    pub host: Option<String>,
    pub db_path: Option<String>,
    /// `None` désigne un daemon antérieur, donc une instance non attestée.
    pub instance_id: Option<String>,
}

#[cfg(test)]
fn daemon_identity(socket_path: &std::path::Path) -> Result<Option<DaemonIdentity>, String> {
    daemon_identity_until(
        socket_path,
        std::time::Instant::now() + DAEMON_IDENTITY_READ_TIMEOUT,
    )
}

fn daemon_identity_until(
    socket_path: &std::path::Path,
    deadline: std::time::Instant,
) -> Result<Option<DaemonIdentity>, String> {
    use std::io::BufReader;

    let stream = match crate::communication::client::connect_nonblocking(socket_path, deadline) {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(None);
        }
        Err(error) => {
            return Err(identity_probe_error("connexion impossible", error));
        }
    };
    let read_stream = stream
        .try_clone()
        .map_err(|error| identity_probe_error("clonage de la socket impossible", error))?;
    // Une échéance absolue couvre connexion, écritures et trois réponses,
    // y compris un pair qui fournit un octet juste avant chaque ancien délai.
    let mut writer = stream;
    let mut reader = BufReader::new(read_stream);
    write_identity_message(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
        "du rôle",
        deadline,
    )?;
    let mut line = String::new();
    if read_identity_line(&mut reader, &mut line, "l'acceptation du rôle", deadline)? == 0 {
        return Err(
            "identité du daemon indisponible: connexion fermée avant l'acceptation du rôle"
                .to_string(),
        );
    }
    if !matches!(
        decode(line.trim())
            .map_err(|error| identity_probe_error("acceptation du rôle invalide", error))?,
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ) {
        return Err("identité du daemon indisponible: rôle Client non accepté".to_string());
    }
    write_identity_message(
        &mut writer,
        &WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            // Même dérivation que les clients normaux : la sonde reste compatible
            // avec toute évolution de la validation de portée.
            issuer_scope: build_id_probe_issuer_scope(),
            capabilities: Vec::new(),
        },
        "du contrat",
        deadline,
    )?;
    line.clear();
    match read_identity_line(&mut reader, &mut line, "l'accueil du client", deadline)? {
        0 => Err(
            "identité du daemon indisponible: connexion fermée avant l'accueil du client"
                .to_string(),
        ),
        _ => match decode(line.trim())
            .map_err(|error| identity_probe_error("accueil du client invalide", error))?
        {
            DaemonToWrapper::ClientWelcome { build_id, .. } => {
                // Second aller-retour, sur la MÊME connexion : la machine et la
                // base ne sont pas déductibles côté client.
                let (host, db_path, instance_id) =
                    match probe_daemon_identity(&mut writer, &mut reader, &mut line, deadline)? {
                        Some((host, db_path, instance_id)) => {
                            (Some(host), Some(db_path), Some(instance_id))
                        }
                        // Daemon antérieur au message : non attesté, jamais deviné.
                        None => (None, None, None),
                    };
                Ok(Some(DaemonIdentity {
                    build_id,
                    host,
                    db_path,
                    instance_id,
                }))
            }
            _ => Err("identité du daemon indisponible: accueil Client absent".to_string()),
        },
    }
}

/// Demande au daemon ce qu'il atteste de lui-même. `None` = daemon antérieur.
fn probe_daemon_identity(
    writer: &mut std::os::unix::net::UnixStream,
    reader: &mut std::io::BufReader<std::os::unix::net::UnixStream>,
    line: &mut String,
    deadline: std::time::Instant,
) -> Result<Option<(String, String, String)>, String> {
    write_identity_message(
        writer,
        &WrapperToDaemon::DaemonIdentityRequest,
        "de la sonde",
        deadline,
    )?;
    line.clear();
    if read_identity_line(reader, line, "le rapport d'identité", deadline)? == 0 {
        // Une ancienne version peut fermer après un message qu'elle ne connaît
        // pas. Le ClientWelcome déjà reçu atteste sa présence, pas son identité.
        return Ok(None);
    }
    match decode(line.trim()) {
        Ok(DaemonToWrapper::DaemonIdentityReport {
            host,
            db_path,
            instance_id,
            ..
        }) => Ok(Some((host, db_path, instance_id))),
        // Refus ou réponse connue mais différente : daemon présent, rapport
        // récent absent. Cette compatibilité est distincte d'une expiration.
        Ok(_) => Ok(None),
        Err(_) if legacy_identity_report(line) => Ok(None),
        Err(error) => Err(identity_probe_error("rapport invalide", error)),
    }
}

fn write_identity_message(
    writer: &mut std::os::unix::net::UnixStream,
    message: &WrapperToDaemon,
    phase: &str,
    deadline: std::time::Instant,
) -> Result<(), String> {
    let mut encoded = encode(message)
        .map_err(|error| identity_probe_error(&format!("encodage {phase} impossible"), error))?;
    encoded.push('\n');
    let mut remaining_bytes = encoded.as_bytes();
    while !remaining_bytes.is_empty() {
        let remaining = deadline
            .checked_duration_since(std::time::Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| identity_probe_error("budget total dépassé", phase))?;
        writer
            .set_write_timeout(Some(remaining))
            .map_err(|error| identity_probe_error("écriture non bornable", error))?;
        match writer.write(remaining_bytes) {
            Ok(0) => {
                return Err(identity_probe_error(
                    "connexion fermée pendant l'envoi",
                    phase,
                ));
            }
            Ok(size) => remaining_bytes = &remaining_bytes[size..],
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(identity_probe_error("envoi impossible", error)),
        }
    }
    Ok(())
}

fn read_identity_line(
    reader: &mut std::io::BufReader<std::os::unix::net::UnixStream>,
    line: &mut String,
    phase: &str,
    deadline: std::time::Instant,
) -> Result<usize, String> {
    let frame = bridget_transport::jsonl::read_unix_line(
        reader,
        bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES,
        bridget_transport::jsonl::LineDeadline::Absolute(deadline),
    )
    .map_err(|error| {
        if matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ) {
            format!(
                "identité du daemon indisponible: délai de lecture de {} s dépassé pendant {phase}",
                DAEMON_IDENTITY_READ_TIMEOUT.as_secs()
            )
        } else {
            identity_probe_error(&format!("lecture impossible pendant {phase}"), error)
        }
    })?;
    let Some(frame) = frame else { return Ok(0) };
    *line = String::from_utf8(frame)
        .map_err(|_| identity_probe_error("réponse invalide", "UTF-8 attendu"))?;
    Ok(line.len())
}

fn identity_probe_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("identité du daemon indisponible: {context}: {error}")
}

fn legacy_identity_report(line: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(line.trim())
        .ok()
        .is_some_and(|value| {
            value.get("type").and_then(serde_json::Value::as_str) == Some("DaemonIdentityReport")
        })
}

#[derive(Default, Clone)]
pub struct DaemonStatus {
    pub running: bool,
    pub agents: Vec<bridget_transport::protocol::AgentInfo>,
    pub agents_inventory_available: bool,
    /// Total non publié par le protocole : `None`, jamais une lecture locale
    /// ou le nombre tronqué d'une projection présenté comme exhaustif.
    pub message_count: Option<usize>,
    pub build_id: Option<String>,
    /// Machine et base attestées par le daemon lui-même.
    pub daemon_host: Option<String>,
    pub daemon_db_path: Option<String>,
}

/// Absence déterminée : aucune socket ne répond, donc l'inventaire vide est
/// une connaissance complète, pas une défaillance de collecte.
fn daemon_absent_status() -> DaemonStatus {
    DaemonStatus {
        agents_inventory_available: true,
        ..DaemonStatus::default()
    }
}

/// Une identité valide a attesté le daemon, puis la collecte a échoué. Ne pas
/// rabattre cette incertitude vers une absence : un appelant doit pouvoir
/// distinguer « aucun agent » de « agents inconnus ».
fn daemon_inventory_unavailable(identity: &DaemonIdentity) -> DaemonStatus {
    DaemonStatus {
        running: true,
        agents_inventory_available: false,
        message_count: None,
        build_id: Some(identity.build_id.clone()),
        daemon_host: identity.host.clone(),
        daemon_db_path: identity.db_path.clone(),
        ..DaemonStatus::default()
    }
}

#[cfg(test)]
#[path = "daemon_inventory_provenance_tests.rs"]
mod inventory_provenance_tests;

#[cfg(test)]
mod signal_disposition_tests {
    use super::install_daemon_signal_handlers;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const CHILD_ENV: &str = "BRIDGET_SIGNAL_DISPOSITION_CHILD";
    const TERM_DEADLINE: Duration = Duration::from_secs(2);

    /// Enfant de l'oracle : emprunte le chemin d'installation des signaux du
    /// daemon (no-op sous `cfg(test)`), puis reste vivant jusqu'à un TERM.
    #[test]
    #[ignore = "processus enfant lancé par un autre test (variable CHILD_ENV) ; sans effet lancé seul"]
    fn signal_disposition_child() {
        if std::env::var(CHILD_ENV).ok().as_deref() != Some("1") {
            return;
        }
        install_daemon_signal_handlers();
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }
    }

    fn assert_dies_on_term(mut child: std::process::Child, label: &str) {
        // Prouve d'abord qu'il était vivant — sinon l'oracle passe sur une
        // projection vide (processus déjà mort, TERM jamais évalué).
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            child.try_wait().expect("try_wait").is_none(),
            "{label}: déjà mort avant TERM — projection vide"
        );
        assert_eq!(
            // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM) },
            0,
            "{label}: kill(SIGTERM) a échoué"
        );
        let deadline = Instant::now() + TERM_DEADLINE;
        loop {
            if child.try_wait().expect("try_wait").is_some() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "{label}: encore vivant après SIGTERM — le binaire de test \
                 a hérité d'un gestionnaire qui empêche la mort propre"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Oracle : un binaire de test qui a emprunté `install_daemon_signal_handlers`
    /// DOIT mourir sous TERM. La baseline `sleep` prouve d'abord que la mesure
    /// elle-même fonctionne (sinon projection vide).
    #[test]
    fn binaire_de_test_meurt_sous_term_apres_install_signaux_daemon() {
        let baseline = Command::new("sleep")
            .arg("120")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep baseline");
        assert_dies_on_term(baseline, "baseline sleep (disposition par défaut)");

        let child = Command::new(std::env::current_exe().expect("current_exe"))
            .arg("--exact")
            .arg("daemon::signal_disposition_tests::signal_disposition_child")
            .arg("--ignored")
            .arg("--nocapture")
            .env(CHILD_ENV, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn signal_disposition_child");
        assert_dies_on_term(
            child,
            "binaire de test après install_daemon_signal_handlers",
        );
    }
}

#[cfg(test)]
#[path = "daemon_spec140_title_tests.rs"]
mod spec140_title_tests;

/// ⚠ FAMILLE COURAMMENT SAUTÉE EN REVUE — un test ajouté ici peut n'être joué
/// par personne.
///
/// Elle n'est PAS exclue par le harnais : `cargo test` la joue. Ce sont les
/// relecteurs qui l'écartent à la main, chacun de son côté, avec
/// `--skip daemon::presence_tests`, parce qu'elle rend un nombre de rouges qui
/// VARIE d'une exécution à l'autre — 2, 7, 8 puis 10 mesurés le 2026-08-27,
/// dont deux comptes différents sur le MÊME commit.
///
/// Le méta-gate `scripts/check-review-witness-selection.sh` reliste cette
/// famille et refuse les arguments libtest qui en omettent un seul témoin.
/// Son banc exécutable est `make test-review-witnesses`.
///
/// CONSÉQUENCE POUR QUI ARRIVE ICI : un oracle rangé dans cette famille sera
/// vert dans votre intention et ne gardera rien — ni chez vous, ni en revue, ni
/// au merge. Si votre test doit protéger quelque chose, mettez-le ailleurs et
/// dites pourquoi ; si sa place est vraiment ici, dites-le aussi, pour que le
/// prochain sache que vous avez choisi.
#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
#[path = "daemon_presence_tests.rs"]
mod presence_tests;

#[cfg(test)]
#[test]
fn temoin_commande_controle_est_recue_puis_resolue_par_le_wrapper_cible() {
    let (mut state, config) = presence_tests::state_with_registered_agent("execution-control");
    let (target_writer, mut target_reader) =
        presence_tests::control_socket("execution-control-target");
    let (client_writer, mut client_reader) =
        presence_tests::control_socket("execution-control-client");
    state
        .connections
        .insert("conn-1".to_string(), target_writer);
    state
        .connections
        .insert("client-control".to_string(), client_writer);
    state
        .execution_store
        .record_starting(
            "submission-control",
            "execution-control",
            "89000000-0000-4000-8000-000000000102",
            10,
        )
        .unwrap();
    assert!(matches!(
        state.execution_store.transition_if_current(
            "execution-control",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            11,
        ),
        Ok(ConditionalTransition::Applied(_))
    ));
    // La remise de contrôle exige une véritable identité parent149/156.
    let (parent_writer, _parent_reader) =
        presence_tests::control_socket("execution-control-parent");
    state
        .connections
        .insert("control-parent".into(), parent_writer);
    let DaemonToWrapper::Registered {
        credential: Some(parent_credential),
        ..
    } = handle_register_with_channel(
        "control-parent",
        2,
        "fixture".into(),
        "89000000-0000-4000-8000-000000000101".into(),
        Some("fixture".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp),
        None,
        Some("test".into()),
        Some("control-parent-instance".into()),
        None,
        false,
        Some(false),
        &mut state,
        false,
    )
    else {
        panic!("identité primaire du parent de contrôle absente");
    };
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "client-control",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "client-control",
            WrapperToDaemon::RegisterAuxiliary {
                agent_id: "89000000-0000-4000-8000-000000000101".into(),
                instance_id: "control-parent-instance".into(),
                credential: parent_credential,
            },
            &shared,
        ),
        Some(DaemonToWrapper::Registered {
            credential: None,
            ..
        })
    ));
    presence_tests::guidance156_link_parent(
        &mut shared.lock().unwrap(),
        "client-control",
        "conn-1",
    );
    assert!(matches!(
        handle_wrapper_message(
            "client-control",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "012_scope_controltestx".to_string(),
                capabilities: vec![
                    ClientCapability::ExecutionControlV1,
                    ClientCapability::Lookup
                ],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    let mut correction = bridget_core::BridgetMessage::new(
        "89000000-0000-4000-8000-000000000101",
        "89000000-0000-4000-8000-000000000102",
        "corrige le point",
    );
    correction.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "control-1".to_string(),
        execution_id: "execution-control".to_string(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::SteerCurrent,
        message: Some(correction),
    };
    assert!(matches!(
        handle_wrapper_message(
            "client-control",
            WrapperToDaemon::ControlExecution {
                command: command.clone(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::OutcomeUnknown,
            ..
        })
    ));
    assert!(matches!(
        presence_tests::read_control(&mut target_reader),
        DaemonToWrapper::ControlExecutionDispatch {
            issuer_scope,
            command: dispatched,
        } if issuer_scope == "012_scope_controltestx" && dispatched == command
    ));
    let report = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ControlExecutionReported {
            issuer_scope: "012_scope_controltestx".to_string(),
            command_id: "control-1".to_string(),
            execution_id: "execution-control".to_string(),
            accepted: false,
            refusal_reason: Some("capability_unavailable".to_string()),
        },
        &shared,
    );
    assert!(report.is_none());
    assert!(matches!(
        presence_tests::read_control(&mut client_reader),
        DaemonToWrapper::ControlExecutionResult {
            command_id,
            execution_id,
            outcome: ExecutionControlOutcome::Refused(ExecutionControlRefusal::CapabilityUnavailable),
            ..
        } if command_id == "control-1" && execution_id == "execution-control"
    ));
    assert!(matches!(
        handle_wrapper_message(
            "client-control",
            WrapperToDaemon::Lookup {
                operation_kind: "execution_control".to_string(),
                idempotency_key: "control-1".to_string(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::Refused(
                ExecutionControlRefusal::CapabilityUnavailable
            ),
            ..
        })
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
#[test]
fn intentions_de_soumission_restent_distinctes_et_persistantes() {
    let (mut state, config) = presence_tests::state_with_registered_agent("intentions-us1");
    let registry_path = config.db_path.with_extension("intentions.json");
    let registry_json = serde_json::json!({
        "agents": {
            "claude": {
                "command": "/bin/sh",
                "protocol": "acp",
                "forbidden_env": [],
                "pass_env": []
            }
        }
    })
    .to_string();
    std::fs::write(&registry_path, registry_json).unwrap();
    std::fs::set_permissions(
        &registry_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .unwrap();
    state.registry = AgentRegistry::from_json(
        &std::fs::read_to_string(&registry_path).unwrap(),
        &registry_path,
    )
    .unwrap();
    // La distinction des intentions s’exerce avec un parent réellement inscrit.
    let (sender_writer, _sender_reader) = presence_tests::control_socket("intentions-us1-sender");
    state
        .connections
        .insert("conn-sender".into(), sender_writer);
    assert!(matches!(
        handle_register_with_channel(
            "conn-sender",
            2,
            "fixture".into(),
            "89000000-0000-4000-8000-000000000101".into(),
            Some("fixture".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("test".into()),
            Some("intentions-parent-instance".into()),
            None,
            false,
            Some(false),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered {
            credential: Some(_),
            ..
        }
    ));
    let (target_writer, mut target_reader) =
        presence_tests::control_socket("intentions-us1-target");
    state
        .connections
        .insert("conn-1".to_string(), target_writer);
    presence_tests::guidance156_link_parent(&mut state, "conn-sender", "conn-1");
    let shared = Arc::new(Mutex::new(state));

    let mut queue = bridget_core::BridgetMessage::new(
        "89000000-0000-4000-8000-000000000101",
        "89000000-0000-4000-8000-000000000102",
        "conserver",
    );
    queue.id = "queue-us1".to_string();
    queue.intent = Some(bridget_core::MessageIntent::QueueOnly);
    let mut queued_expected = queue.clone();
    queued_expected.hops -= 1;
    assert!(matches!(
        handle_wrapper_message("conn-sender", WrapperToDaemon::Send(queue.clone()), &shared),
        Some(DaemonToWrapper::Ack { id , ..}) if id == "queue-us1"
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .execution_store
            .take_next_submission("89000000-0000-4000-8000-000000000102")
            .unwrap()
            .unwrap()
            .message,
        Some(queued_expected)
    );

    let mut trigger = bridget_core::BridgetMessage::new(
        "89000000-0000-4000-8000-000000000101",
        "89000000-0000-4000-8000-000000000102",
        "démarrer",
    );
    trigger.id = "trigger-us1".to_string();
    trigger.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    let mut trigger_expected = trigger.clone();
    trigger_expected.hops -= 1;
    assert!(matches!(
        handle_wrapper_message("conn-sender", WrapperToDaemon::Send(trigger), &shared),
        Some(DaemonToWrapper::Ack { id , ..}) if id == "trigger-us1"
    ));
    assert!(matches!(
        presence_tests::read_control(&mut target_reader),
        DaemonToWrapper::DeliverExecution { message, execution_id, .. }
            if message.id == "trigger-us1"
                && message.intent == Some(bridget_core::MessageIntent::TriggerTurn)
                && execution_id == "execution-trigger-us1"
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .execution_store
            .execution_message("execution-trigger-us1")
            .unwrap(),
        Some(trigger_expected)
    );

    let mut steer = bridget_core::BridgetMessage::new(
        "89000000-0000-4000-8000-000000000101",
        "89000000-0000-4000-8000-000000000102",
        "corriger",
    );
    steer.id = "steer-us1".to_string();
    steer.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    assert!(matches!(
        handle_wrapper_message("conn-sender", WrapperToDaemon::Send(steer), &shared),
        Some(DaemonToWrapper::Ack { id , ..}) if id == "steer-us1"
    ));
    assert!(matches!(
        presence_tests::read_control(&mut target_reader),
        DaemonToWrapper::Deliver { message, .. }
            if message.id == "steer-us1"
                && message.intent == Some(bridget_core::MessageIntent::SteerCurrent)
    ));

    let mut interrupt = bridget_core::BridgetMessage::new(
        "89000000-0000-4000-8000-000000000101",
        "89000000-0000-4000-8000-000000000102",
        "interrompre et démarrer",
    );
    interrupt.id = "interrupt-us1".to_string();
    interrupt.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    assert!(matches!(
        handle_wrapper_message("conn-sender", WrapperToDaemon::Send(interrupt), &shared),
        Some(DaemonToWrapper::Ack { id , ..}) if id == "interrupt-us1"
    ));
    assert!(matches!(
        presence_tests::read_control(&mut target_reader),
        DaemonToWrapper::DeliverExecution { message, execution_id, .. }
            if message.id == "interrupt-us1"
                && message.intent == Some(bridget_core::MessageIntent::InterruptAndStart)
                && execution_id == "execution-interrupt-us1"
    ));

    let mut control = bridget_core::BridgetMessage::new(
        "89000000-0000-4000-8000-000000000101",
        "89000000-0000-4000-8000-000000000102",
        "ne devient pas un prompt",
    );
    control.id = "control-us1".to_string();
    control.intent = Some(bridget_core::MessageIntent::ControlOnly);
    assert!(matches!(
        handle_wrapper_message("conn-sender", WrapperToDaemon::Send(control), &shared),
        Some(DaemonToWrapper::Nack { reason, .. }) if reason.contains("ControlExecution")
    ));

    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_file(registry_path);
}

#[cfg(test)]
#[test]
fn transition_execution_refusee_si_wrapper_non_proprietaire() {
    let (mut state, config) = presence_tests::state_with_registered_agent("transition-owner");
    state
        .execution_store
        .record_starting(
            "submission-owner",
            "execution-owner",
            "89000000-0000-4000-8000-000000000102",
            10,
        )
        .unwrap();
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000104",
            &bridget_core::AgentType::Codex,
            "conn-attacker",
        )
        .unwrap();
    state.conn_names.insert(
        "conn-attacker".to_string(),
        "89000000-0000-4000-8000-000000000104".to_string(),
    );
    state
        .conn_instances
        .insert("conn-attacker".to_string(), "instance-attacker".to_string());

    let outcome = handle_execution_transition(
        "conn-attacker",
        bridget_transport::protocol::ExecutionStateTransition {
            execution_id: "execution-owner".to_string(),
            generation: 1,
            expected_state: "starting".to_string(),
            expected_revision: 0,
            next_state: "running".to_string(),
            reason: "provider_accepted".to_string(),
            observed_at: 11,
        },
        &mut state,
    );

    assert!(matches!(
        outcome,
        Some(DaemonToWrapper::Nack { reason, .. }) if reason == "transition d exécution émise par un wrapper non propriétaire"
    ));
    let snapshot = state
        .execution_store
        .execution_snapshot("execution-owner")
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.state, "starting");
    assert_eq!(snapshot.revision, 0);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
#[test]
fn spec_068_daemon_persiste_remet_et_accuse_l_incident_delegue_au_seul_parent() {
    use crate::idempotency::{AgentLinkRecord, AgentLinkState};

    let (mut state, config) = presence_tests::state_with_registered_agent("spec-068-delegated");
    state
        .idempotency
        .create_agent_link(&AgentLinkRecord {
            link_id: "link-068".to_string(),
            parent_instance_id: "instance-parent".to_string(),
            child_instance_id: "instance-1".to_string(),
            parent_execution_id: Some("parent-execution".to_string()),
            objective_id: None,
            delegation_id: None,
            project: None,
            role: "worker".to_string(),
            agent_path: "instance-parent/instance-1".to_string(),
            state: AgentLinkState::Reserved,
            created_at: 1_700_000_000,
            closed_at: None,
            revision: 0,
        })
        .unwrap();
    let (writer_stream, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    state.connections.insert(
        "conn-parent".to_string(),
        Arc::new(Mutex::new(BufWriter::new(writer_stream))),
    );
    state
        .conn_instances
        .insert("conn-parent".to_string(), "instance-parent".to_string());
    state
        .conn_instances
        .insert("conn-foreign".to_string(), "instance-foreign".to_string());
    let shared = Arc::new(Mutex::new(state));
    let reference = format!("sha256:{}", "c".repeat(64));
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DelegatedRuntimeEvent {
                execution_id: "child-execution".to_string(),
                kind: bridget_transport::protocol::DelegatedRuntimeEventKind::Warning,
                code: "unsupported_provider_request".to_string(),
                reference: reference.clone(),
            },
            &shared
        )
        .is_none()
    );

    let mut line = String::new();
    BufReader::new(peer).read_line(&mut line).unwrap();
    let event_id = match decode(line.trim()).unwrap() {
        DaemonToWrapper::DelegatedRuntimeEvent { event } => {
            assert_eq!(event.child_instance_id, "instance-1");
            assert_eq!(
                event.kind,
                bridget_transport::protocol::DelegatedRuntimeEventKind::Warning
            );
            assert_eq!(event.reference, reference);
            assert!(!line.contains("params"));
            event.event_id
        }
        other => panic!("événement délégué attendu, reçu {other:?}"),
    };
    assert!(matches!(
        handle_wrapper_message(
            "conn-foreign",
            WrapperToDaemon::DelegatedRuntimeEventAcknowledged {
                event_id: event_id.clone()
            },
            &shared
        ),
        Some(DaemonToWrapper::Nack { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "conn-parent",
            WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id },
            &shared
        ),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(
        shared
            .lock()
            .unwrap()
            .fleet
            .delegated_runtime_events_for_parent("instance-parent")
            .unwrap()
            .is_empty()
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
#[test]
fn spec_068_register_rejoue_apres_registered_les_incidents_delegues_en_ordre() {
    use crate::idempotency::{AgentLinkRecord, AgentLinkState, DelegatedRuntimeEventInput};
    use bridget_transport::protocol::DelegatedRuntimeEventKind;

    let (mut state, config) = presence_tests::state_with_registered_agent("spec-068-replay");
    state.conn_counter = 1;
    state
        .idempotency
        .create_agent_link(&AgentLinkRecord {
            link_id: "link-068-replay".to_string(),
            parent_instance_id: "instance-parent".to_string(),
            child_instance_id: "instance-1".to_string(),
            parent_execution_id: None,
            objective_id: None,
            delegation_id: None,
            project: None,
            role: "worker".to_string(),
            agent_path: "instance-parent/instance-1".to_string(),
            state: AgentLinkState::Reserved,
            created_at: 1_700_000_000,
            closed_at: None,
            revision: 0,
        })
        .unwrap();
    for (code, reference) in [
        (
            "unsupported_provider_request",
            format!("sha256:{}", "d".repeat(64)),
        ),
        ("provider_failed", format!("sha256:{}", "e".repeat(64))),
    ] {
        state
            .fleet
            .record_delegated_runtime_event(DelegatedRuntimeEventInput {
                child_instance_id: "instance-1".to_string(),
                child_execution_id: "child-execution".to_string(),
                kind: if code == "provider_failed" {
                    DelegatedRuntimeEventKind::Failed
                } else {
                    DelegatedRuntimeEventKind::Warning
                },
                code: code.to_string(),
                reference,
                observed_at: 1_700_000_001,
            })
            .unwrap();
    }
    let shared = Arc::new(Mutex::new(state));
    let listener = UnixListener::bind(&config.socket_path).unwrap();
    let server_state = Arc::clone(&shared);
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_connection(stream, server_state).unwrap();
    });
    let stream = UnixStream::connect(&config.socket_path).unwrap();
    let read_stream = stream.try_clone().unwrap();
    read_stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut writer = BufWriter::new(stream);
    let register = WrapperToDaemon::Register {
        agent_type: "claude".to_string(),
        identity_version: 2,
        agent_id: "89000000-0000-4000-8000-000000000168".to_string(),
        host: Some("test".to_string()),
        transport: Some("acp".to_string()),
        channel: ChannelReport::Known("unix".to_string()),
        mode: Some(PresenceMode::Acp),
        location: None,
        os: Some("Linux".to_string()),
        instance_id: Some("instance-parent".to_string()),
        domain: None,
        turn_in_progress: false,
        journal_available: Some(true),
        turn_attest: false,
        queue_fallback: false,
    };
    writeln!(writer, "{}", encode(&register).unwrap()).unwrap();
    writer.flush().unwrap();
    let mut reader = BufReader::new(read_stream);
    let mut first = String::new();
    reader.read_line(&mut first).unwrap();
    assert!(matches!(
        decode(first.trim()).unwrap(),
        DaemonToWrapper::Registered { .. }
    ));
    let mut codes = Vec::new();
    for _ in 0..2 {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        match decode(line.trim()).unwrap() {
            DaemonToWrapper::DelegatedRuntimeEvent { event } => {
                codes.push((event.cursor, event.code))
            }
            other => panic!("rejeu runtime attendu, reçu {other:?}"),
        }
    }
    assert_eq!(
        codes
            .iter()
            .map(|(_, code)| code.as_str())
            .collect::<Vec<_>>(),
        vec!["unsupported_provider_request", "provider_failed"]
    );
    assert!(codes[0].0 < codes[1].0);
    drop(reader);
    drop(writer);
    server.join().unwrap();
    drop(shared);
    let _ = std::fs::remove_file(config.socket_path);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
#[test]
fn spec172_auxiliary_scope_excludes_runtime_replay_without_losing_parent_notice() {
    use crate::idempotency::{AgentLinkRecord, AgentLinkState, DelegatedRuntimeEventInput};
    use bridget_transport::protocol::{DelegatedRuntimeEventKind, NativeDelegationRequest};
    for kind in [
        DelegatedRuntimeEventKind::Warning,
        DelegatedRuntimeEventKind::Failed,
    ] {
        let (mut state, config) =
            presence_tests::state_with_registered_agent("spec172-scope-replay");
        state.conn_counter = 1;
        let agent = state.conn_names["conn-1"].clone();
        let credential = state.identity_credentials["conn-1"].clone();
        state
            .idempotency
            .create_agent_link(&AgentLinkRecord {
                link_id: "link-scope172".into(),
                parent_instance_id: "instance-1".into(),
                child_instance_id: "child-scope172".into(),
                parent_execution_id: None,
                objective_id: None,
                delegation_id: None,
                project: None,
                role: "worker".into(),
                agent_path: "instance-1/child-scope172".into(),
                state: AgentLinkState::Reserved,
                created_at: 1_700_000_000,
                closed_at: None,
                revision: 0,
            })
            .unwrap();
        let event = state
            .fleet
            .record_delegated_runtime_event(DelegatedRuntimeEventInput {
                child_instance_id: "child-scope172".into(),
                child_execution_id: "execution-scope172".into(),
                kind,
                code: if kind == DelegatedRuntimeEventKind::Failed {
                    "provider_failed".into()
                } else {
                    "unsupported_provider_request".into()
                },
                reference: format!("sha256:{}", "e".repeat(64)),
                observed_at: 1_700_000_001,
            })
            .unwrap();
        let (parent_writer, parent_peer) = UnixStream::pair().unwrap();
        parent_peer
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        state.connections.insert(
            "conn-1".into(),
            Arc::new(Mutex::new(BufWriter::new(parent_writer))),
        );
        let shared = Arc::new(Mutex::new(state));
        let listener = UnixListener::bind(&config.socket_path).unwrap();
        let serving = Arc::clone(&shared);
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, serving).unwrap();
        });
        let stream = UnixStream::connect(&config.socket_path).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        writeln!(
            writer,
            "{}",
            encode(&WrapperToDaemon::RegisterAuxiliary {
                agent_id: agent,
                instance_id: "instance-1".into(),
                credential,
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim()).unwrap(),
            DaemonToWrapper::Registered { .. }
        ));
        writeln!(
            writer,
            "{}",
            encode(&WrapperToDaemon::NativeDelegation {
                request: NativeDelegationRequest::CallerScope {},
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        let response = decode(line.trim()).unwrap();
        assert!(
            matches!(response, DaemonToWrapper::NativeDelegationResult { ref result }
            if result == &serde_json::json!({"version":1,"caller_read_only":false})),
            "CallerScope reçoit un avis au lieu du plafond : {response:?}"
        );
        // Le correctif ne lit ni n’acquitte l’avis à la place du parent.
        {
            let st = shared.lock().unwrap();
            let pending = st
                .fleet
                .delegated_runtime_events_for_parent("instance-1")
                .unwrap();
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].event_id, event.event_id);
        }
        replay_delegated_runtime_events_for_connection(&shared, "conn-1");
        let mut parent_reader = BufReader::new(parent_peer);
        line.clear();
        parent_reader.read_line(&mut line).unwrap();
        assert!(matches!(decode(line.trim()).unwrap(),
            DaemonToWrapper::DelegatedRuntimeEvent { event: ref actual }
            if actual.event_id == event.event_id && actual.kind == kind));
        assert!(matches!(
            handle_wrapper_message(
                "conn-1",
                WrapperToDaemon::DelegatedRuntimeEventAcknowledged {
                    event_id: event.event_id
                },
                &shared
            ),
            Some(DaemonToWrapper::Ack { .. })
        ));
        assert!(
            shared
                .lock()
                .unwrap()
                .fleet
                .delegated_runtime_events_for_parent("instance-1")
                .unwrap()
                .is_empty()
        );
        drop(reader);
        drop(writer);
        server.join().unwrap();
        drop(parent_reader);
        drop(shared);
        let _ = std::fs::remove_file(config.socket_path);
        let _ = std::fs::remove_file(config.db_path);
    }
}

#[cfg(test)]
#[test]
fn spec_068_enfant_sans_lien_ne_notifie_aucun_coordinateur() {
    let (state, config) = presence_tests::state_with_registered_agent("spec-068-no-link");
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DelegatedRuntimeEvent {
                execution_id: "orphan-execution".to_string(),
                kind: bridget_transport::protocol::DelegatedRuntimeEventKind::Warning,
                code: "unsupported_provider_request".to_string(),
                reference: format!("sha256:{}", "f".repeat(64)),
            },
            &shared
        ),
        Some(DaemonToWrapper::Nack { .. })
    ));
    assert!(
        shared
            .lock()
            .unwrap()
            .fleet
            .delegated_runtime_events_for_parent("instance-1")
            .unwrap()
            .is_empty()
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
fn spec094_live_test_connection(state: &mut DaemonState, conn_id: &str) -> UnixStream {
    let (daemon, peer) = UnixStream::pair().unwrap();
    state.connections.insert(
        conn_id.to_string(),
        Arc::new(Mutex::new(BufWriter::new(daemon))),
    );
    peer
}

#[cfg(test)]
#[test]
fn spec099_identite_connue_ne_permet_pas_inscription_mcp() {
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec099-mcp-forge");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    let response = handle_register_with_channel(
        "attacker",
        2,
        "mcp".into(),
        agent,
        None,
        None,
        ChannelReport::Unknown,
        Some(PresenceMode::Cli),
        None,
        None,
        Some("instance-1".into()),
        None,
        false,
        None,
        &mut state,
        false,
    );
    assert!(matches!(response, DaemonToWrapper::Nack { .. }));
}

#[cfg(test)]
#[test]
fn spec099_cli_flag_false_ne_permet_pas_usurpation() {
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec099-cli-forge");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    let mut message = bridget_core::BridgetMessage::new(&agent, &agent, "faux émetteur");
    message.from_declared = false;
    let response = handle_wrapper_message(
        "attacker",
        WrapperToDaemon::Send(message),
        &Arc::new(Mutex::new(state)),
    );
    assert!(
        matches!(response, Some(DaemonToWrapper::Nack { reason, .. }) if reason.contains("identité"))
    );
}

#[cfg(test)]
#[test]
fn spec099_preuve_auxiliaire_valide_bornee_et_revoquee() {
    use bridget_transport::protocol::IdentityCredential;
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec099-proof");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    let credential = state.identity_credentials["conn-1"].clone();
    for (name, instance, proof) in [
        (
            agent.clone(),
            "instance-1",
            IdentityCredential::new("incorrecte".into()),
        ),
        (
            uuid::Uuid::new_v4().to_string(),
            "instance-1",
            credential.clone(),
        ),
        (agent.clone(), "autre-instance", credential.clone()),
    ] {
        assert!(matches!(
            register_auxiliary(&mut state, "attacker", name, instance.into(), proof),
            DaemonToWrapper::Nack { .. }
        ));
        assert!(!state.conn_names.contains_key("attacker"));
    }
    let _aux = spec094_live_test_connection(&mut state, "aux");
    assert!(matches!(
        register_auxiliary(
            &mut state,
            "aux",
            agent.clone(),
            "instance-1".into(),
            credential.clone()
        ),
        DaemonToWrapper::Registered {
            credential: None,
            ..
        }
    ));
    assert!(sender_is_authorized(&state, "aux", &agent));
    assert!(self_mutation_instance(&state, "aux", &agent, "rename").is_ok());
    let private_wire = serde_json::to_string(&credential).unwrap();
    let directory = encode(&DaemonToWrapper::AgentList {
        agents: state.agent_infos(),
    })
    .unwrap();
    assert!(!directory.contains(&private_wire));
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");
    assert!(!sender_is_authorized(&state, "aux", &agent));
    assert!(self_mutation_instance(&state, "aux", &agent, "rename").is_err());
    assert!(matches!(
        register_auxiliary(
            &mut state,
            "retry",
            agent.clone(),
            "instance-1".into(),
            credential.clone()
        ),
        DaemonToWrapper::Nack { .. }
    ));
    let _new_owner = spec094_live_test_connection(&mut state, "owner-reconnected");
    let DaemonToWrapper::Registered {
        credential: Some(renewed),
        ..
    } = handle_register_with_channel(
        "owner-reconnected",
        2,
        "fixture".into(),
        agent.clone(),
        Some("fixture".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp),
        None,
        Some("test".into()),
        Some("instance-1".into()),
        None,
        false,
        Some(false),
        &mut state,
        false,
    )
    else {
        panic!("reconnexion propriétaire attendue");
    };
    assert_ne!(renewed, credential);
    assert!(matches!(
        register_auxiliary(
            &mut state,
            "retry-old",
            agent.clone(),
            "instance-1".into(),
            credential
        ),
        DaemonToWrapper::Nack { .. }
    ));
    assert!(matches!(
        register_auxiliary(
            &mut state,
            "aux",
            agent.clone(),
            "instance-1".into(),
            renewed
        ),
        DaemonToWrapper::Registered {
            credential: None,
            ..
        }
    ));
    assert!(sender_is_authorized(&state, "aux", &agent));
}

#[cfg(test)]
#[test]
fn spec172_binding_refusal_preserves_guards_and_explains_failed_check() {
    use bridget_transport::protocol::IdentityCredential;
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec172-binding");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    let credential = state.identity_credentials["conn-1"].clone();
    for (name, instance, proof, reason) in [
        (
            Uuid::new_v4().to_string(),
            "instance-1",
            credential.clone(),
            "primary_absent",
        ),
        (
            agent.clone(),
            "instance-1",
            IdentityCredential::new("private-canary172".into()),
            "credential_mismatch",
        ),
        (
            agent.clone(),
            "another-instance",
            credential.clone(),
            "instance_mismatch",
        ),
    ] {
        let response = register_auxiliary(&mut state, "aux172", name, instance.into(), proof);
        assert!(
            matches!(&response, DaemonToWrapper::Nack { reason: detail, .. }
            if detail == &format!("auxiliary_identity_unproven : {reason}"))
        );
        assert!(!format!("{response:?}").contains("private-canary172"));
        assert!(!state.conn_names.contains_key("aux172"));
    }
    state.presences.get_mut("instance-1").unwrap().state = "unreachable".into();
    assert!(
        matches!(register_auxiliary(&mut state, "aux172", agent, "instance-1".into(), credential),
        DaemonToWrapper::Nack { reason, .. } if reason.ends_with("primary_not_live"))
    );
    assert!(!state.conn_names.contains_key("aux172"));
}

#[cfg(test)]
#[test]
fn spec099_humain_idempotent_depuis_client_nu_reste_admis() {
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec099-humain-nu");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    state.client_negotiations.insert(
        "client".into(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "human-scope".into(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    );
    let mut controls = Vec::new();
    let response = handle_idempotent_send(
        "client",
        bridget_core::BridgetMessage::new("human", &agent, "bonjour"),
        "spec099-humain-id".into(),
        unix_now_secs(),
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        &mut state,
        &mut controls,
    );
    assert!(
        !matches!(&response, DaemonToWrapper::Nack { reason, .. } if reason.contains("non attestée")),
        "l'humain d'un client nu ne doit pas être traité comme une identité empruntée : {response:?}"
    );
    // Le même client, une fois porteur d'une identité d'agent, ne peut plus
    // se dire humain pour contourner la preuve.
    state.conn_names.insert("client".into(), agent.clone());
    let response = handle_idempotent_send(
        "client",
        bridget_core::BridgetMessage::new("human", &agent, "bonjour"),
        "spec099-humain-id-2".into(),
        unix_now_secs(),
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        &mut state,
        &mut controls,
    );
    assert!(
        matches!(&response, DaemonToWrapper::Nack { reason, .. } if reason.contains("non attestée"))
    );
}

#[cfg(test)]
#[test]
fn spec099_client_scope_ne_prouve_pas_identite_avant_reservation() {
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec099-client-forge");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    state.client_negotiations.insert(
        "client".into(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "known-scope".into(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    );
    let id = "spec099-forged-id";
    let response = handle_idempotent_send(
        "client",
        bridget_core::BridgetMessage::new(&agent, &agent, "forgé"),
        id.into(),
        unix_now_secs(),
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        &mut state,
        &mut Vec::new(),
    );
    assert!(matches!(response, DaemonToWrapper::Nack { .. }));
    assert_eq!(state.idempotency.record_count().unwrap(), 0);
}

#[cfg(test)]
#[test]
fn spec099_roles_attach_et_service_refusent_preuve_auxiliaire() {
    let (mut state, _config) = presence_tests::state_with_registered_agent("spec099-roles");
    let agent = state.presences["instance-1"].name.clone();
    let _owner = spec094_live_test_connection(&mut state, "conn-1");
    let credential = state.identity_credentials["conn-1"].clone();
    state
        .connection_roles
        .insert("attach".into(), ConnectionRole::Attach);
    state
        .connection_roles
        .insert("service".into(), ConnectionRole::Service);
    let shared = Arc::new(Mutex::new(state));
    for conn in ["attach", "service"] {
        assert!(!matches!(
            handle_wrapper_message(
                conn,
                WrapperToDaemon::RegisterAuxiliary {
                    agent_id: agent.clone(),
                    instance_id: "instance-1".into(),
                    credential: credential.clone(),
                },
                &shared
            ),
            Some(DaemonToWrapper::Registered { .. })
        ));
    }
}

#[cfg(test)]
#[test]
fn spec094_mutations_propres_refusent_connexion_brute_cible_etrangere_et_ancienne_instance() {
    use bridget_transport::protocol::RuntimeSource;

    let mutations = |agent: &str, suffix: &str| {
        [
            WrapperToDaemon::Domain {
                agent: agent.to_string(),
                domain: Some(format!("domaine-{suffix}")),
            },
            WrapperToDaemon::Availability {
                agent: agent.to_string(),
                until_secs: Some(u64::MAX),
            },
            WrapperToDaemon::Runtime {
                agent: agent.to_string(),
                model: format!("modele-{suffix}"),
                effort: None,
                source: RuntimeSource::Declared,
            },
        ]
    };

    let (mut state, config) = presence_tests::state_with_registered_agent("spec094-authority");
    let agent_a = "89000000-0000-4000-8000-000000000102";
    let agent_b = "89000000-0000-4000-8000-000000000194";
    let _peer_a = spec094_live_test_connection(&mut state, "conn-1");
    state
        .conn_names
        .insert("conn-1".to_string(), agent_a.to_string());

    let _peer_b = spec094_live_test_connection(&mut state, "conn-b");
    let registered_b = handle_register_with_channel(
        "conn-b",
        2,
        "fixture".to_string(),
        agent_b.to_string(),
        Some("fixture".to_string()),
        Some("acp".to_string()),
        ChannelReport::Known("unix".to_string()),
        Some(PresenceMode::Cli),
        None,
        Some("test".to_string()),
        Some("instance-b".to_string()),
        None,
        false,
        Some(false),
        &mut state,
        false,
    );
    assert!(matches!(registered_b, DaemonToWrapper::Registered { .. }));
    let shared = Arc::new(Mutex::new(state));

    for message in mutations(agent_a, "brut-interdit") {
        assert!(matches!(
            handle_wrapper_message("connexion-brute", message, &shared),
            Some(DaemonToWrapper::Nack { .. })
        ));
    }

    for message in mutations(agent_b, "usurpation") {
        assert!(matches!(
            handle_wrapper_message("conn-1", message, &shared),
            Some(DaemonToWrapper::Nack { .. })
        ));
    }
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .presences
            .get("instance-b")
            .and_then(|presence| presence.domain.as_deref()),
        None
    );
    {
        let state = shared.lock().unwrap();
        let presence = state.presences.get("instance-b").unwrap();
        assert!(presence.model.is_none());
        assert!(!presence.is_dnd());
    }

    let (_old_peer, _new_peer) = {
        let mut state = shared.lock().unwrap();
        let peer = spec094_live_test_connection(&mut state, "mcp-old");
        let credential = state.identity_credentials["conn-1"].clone();
        let registered = register_auxiliary(
            &mut state,
            "mcp-old",
            agent_a.into(),
            "instance-1".into(),
            credential,
        );
        assert!(matches!(registered, DaemonToWrapper::Registered { .. }));
        state.router.unregister_by_conn("conn-1");
        state.mark_unreachable("conn-1");
        let new_peer = spec094_live_test_connection(&mut state, "conn-new");
        let reconnected = handle_register_with_channel(
            "conn-new",
            2,
            "claude".to_string(),
            agent_a.to_string(),
            Some("macbook".to_string()),
            Some("acp".to_string()),
            ChannelReport::Known("unix".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-new".to_string()),
            None,
            false,
            Some(true),
            &mut state,
            false,
        );
        assert!(matches!(reconnected, DaemonToWrapper::Registered { .. }));
        (peer, new_peer)
    };
    for message in mutations(agent_a, "ancienne-instance") {
        assert!(matches!(
            handle_wrapper_message("mcp-old", message, &shared),
            Some(DaemonToWrapper::Nack { .. })
        ));
    }
    let _stale_retry_peer = {
        let mut state = shared.lock().unwrap();
        let peer = spec094_live_test_connection(&mut state, "mcp-retry-old");
        let response = handle_register_with_channel(
            "mcp-retry-old",
            2,
            "mcp".to_string(),
            agent_a.to_string(),
            None,
            None,
            ChannelReport::Unknown,
            Some(PresenceMode::Cli),
            None,
            None,
            Some("instance-1".to_string()),
            None,
            false,
            None,
            &mut state,
            false,
        );
        assert!(matches!(response, DaemonToWrapper::Nack { .. }));
        peer
    };
    {
        let state = shared.lock().unwrap();
        let presence = state.presences.get("instance-new").unwrap();
        assert!(presence.model.is_none());
        assert!(!presence.is_dnd());
    }

    let _current_auxiliary_peer = {
        let mut state = shared.lock().unwrap();
        let peer = spec094_live_test_connection(&mut state, "mcp-new");
        let credential = state.identity_credentials["conn-new"].clone();
        let registered = register_auxiliary(
            &mut state,
            "mcp-new",
            agent_a.into(),
            "instance-new".into(),
            credential,
        );
        assert!(matches!(registered, DaemonToWrapper::Registered { .. }));
        peer
    };
    for invalid_domain in ["avec espace", "avec/slash", "équipe"] {
        assert!(matches!(
            handle_wrapper_message(
                "mcp-new",
                WrapperToDaemon::Domain {
                    agent: agent_a.to_string(),
                    domain: Some(invalid_domain.to_string()),
                },
                &shared,
            ),
            Some(DaemonToWrapper::Nack { .. })
        ));
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .presences
                .get("instance-new")
                .and_then(|presence| presence.domain.as_deref()),
            None
        );
    }
    for (model, effort) in [("   ", None), ("modele", Some("   "))] {
        assert!(matches!(
            handle_wrapper_message(
                "mcp-new",
                WrapperToDaemon::Runtime {
                    agent: agent_a.to_string(),
                    model: model.to_string(),
                    effort: effort.map(str::to_string),
                    source: RuntimeSource::Declared,
                },
                &shared,
            ),
            Some(DaemonToWrapper::Nack { .. })
        ));
        let state = shared.lock().unwrap();
        let presence = state.presences.get("instance-new").unwrap();
        assert!(presence.model.is_none());
        assert!(presence.effort.is_none());
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    for message in [
        WrapperToDaemon::Domain {
            agent: agent_a.to_string(),
            domain: Some("courant".to_string()),
        },
        WrapperToDaemon::Availability {
            agent: agent_a.to_string(),
            until_secs: Some(now + 60),
        },
        WrapperToDaemon::Runtime {
            agent: agent_a.to_string(),
            model: "modele-courant".to_string(),
            effort: Some("high".to_string()),
            source: RuntimeSource::Declared,
        },
    ] {
        assert!(matches!(
            handle_wrapper_message("mcp-new", message, &shared),
            Some(DaemonToWrapper::Ack { .. })
        ));
    }
    let before_expired_dnd = shared
        .lock()
        .unwrap()
        .presences
        .get("instance-new")
        .unwrap()
        .dnd_until;
    assert!(before_expired_dnd.is_some());
    assert!(matches!(
        handle_wrapper_message(
            "mcp-new",
            WrapperToDaemon::Availability {
                agent: agent_a.to_string(),
                until_secs: Some(now),
            },
            &shared,
        ),
        Some(DaemonToWrapper::Nack { .. })
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .presences
            .get("instance-new")
            .unwrap()
            .dnd_until,
        before_expired_dnd
    );

    assert!(matches!(
        handle_wrapper_message(
            "legacy-hook",
            WrapperToDaemon::Runtime {
                agent: agent_a.to_string(),
                model: "modele-observe".to_string(),
                effort: None,
                source: RuntimeSource::ClaudeHook,
            },
            &shared,
        ),
        Some(DaemonToWrapper::Ack { .. })
    ));

    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
#[test]
fn spec094_panne_compteur_inbox_refuse_control_state_sans_inventer_zero() {
    use bridget_transport::protocol::{CONTROL_STATE_CONTRACT_VERSION, ControlStateRefusal};

    let (state, config) = presence_tests::state_with_registered_agent("spec094-control-count");
    state
        .store
        .connection()
        .execute("DROP TABLE human_inbox", [])
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_control_state_read(&shared, CONTROL_STATE_CONTRACT_VERSION),
        DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::StoreUnavailable
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[cfg(test)]
#[test]
fn spec094_control_status_negocie_lookup_sans_autoriser_une_mutation() {
    use bridget_transport::protocol::{CLIENT_CONTRACT_VERSION, ClientCapability, ClientRefusal};

    let (state, config) = presence_tests::state_with_registered_agent("spec094-control-readonly");
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "control-status",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "control-status",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("instance-094"),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "control-status",
            WrapperToDaemon::ControlStateSet {
                version: 1,
                command_id: "interdit-094".to_string(),
                expected_generation: 0,
                paused: Some(true),
                auto_objectives_cap: None,
                reason: None,
                agent_posture: None,
                auto_reassignment: None,
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}
