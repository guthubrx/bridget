//! Métriques du daemon, budget de connexions Unix et tube d'éveil arrêt propre.

use log::info;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicUsize, Ordering};

/// Plafond de connexions Unix simultanées pour prévenir l'épuisement de threads OS.
pub(super) const MAX_UNIX_CONNECTIONS: usize = 256;
pub(super) const RESERVED_UNIX_CONNECTIONS: usize = 32;

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
    native_tick_latencies_ms: std::sync::RwLock<VecDeque<u64>>,
    directory_snapshot: std::sync::RwLock<ReadMostlyDirectory>,
}

/// Vue en lecture majoritaire protégée par RwLock pour éviter de bloquer
/// les lectures d'annuaire/présences sur le verrou exclusif global `Mutex<DaemonState>`.
#[derive(Debug, Clone, Default)]
pub struct ReadMostlyDirectory {
    pub connected_agents: usize,
    pub registered_agents: usize,
    pub registry_agent_count: usize,
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
            native_tick_latencies_ms: std::sync::RwLock::new(VecDeque::with_capacity(256)),
            directory_snapshot: std::sync::RwLock::new(ReadMostlyDirectory::default()),
        }
    }

    pub fn record_native_tick_ms(&self, duration_ms: u64) {
        if let Ok(mut window) = self.native_tick_latencies_ms.write() {
            if window.len() >= 256 {
                window.pop_front();
            }
            window.push_back(duration_ms);
        }
    }

    pub fn update_directory_snapshot(&self, snapshot: ReadMostlyDirectory) {
        if let Ok(mut guard) = self.directory_snapshot.write() {
            *guard = snapshot;
        }
    }

    pub fn directory_snapshot(&self) -> ReadMostlyDirectory {
        self.directory_snapshot
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn native_tick_quantiles_ms(&self) -> (u64, u64, u64, usize) {
        let mut samples: Vec<u64> = self
            .native_tick_latencies_ms
            .read()
            .map(|w| w.iter().copied().collect())
            .unwrap_or_default();
        if samples.is_empty() {
            return (0, 0, 0, 0);
        }
        samples.sort_unstable();
        let n = samples.len();
        let pick = |pct: usize| samples[(n.saturating_mul(pct) / 100).min(n - 1)];
        (pick(50), pick(95), pick(99), n)
    }

    pub fn snapshot_json(&self) -> serde_json::Value {
        let (p50, p95, p99, samples) = self.native_tick_quantiles_ms();
        let dir = self.directory_snapshot();
        serde_json::json!({
            "messages_sent": self.messages_sent.load(Ordering::Relaxed),
            "messages_received": self.messages_received.load(Ordering::Relaxed),
            "errors": self.errors.load(Ordering::Relaxed),
            "active_connections": self.active_connections.load(Ordering::Relaxed),
            "max_connections": self.max_connections.load(Ordering::Relaxed),
            "execution_admitted": self.execution_admitted.load(Ordering::Relaxed),
            "execution_started": self.execution_started.load(Ordering::Relaxed),
            "execution_transition_rejected": self.execution_transition_rejected.load(Ordering::Relaxed),
            "execution_delivery_failures": self.execution_delivery_failures.load(Ordering::Relaxed),
            "execution_queue_saturated": self.execution_queue_saturated.load(Ordering::Relaxed),
            "execution_start_latency_secs_total": self.execution_start_latency_secs_total.load(Ordering::Relaxed),
            "execution_start_latency_samples": self.execution_start_latency_samples.load(Ordering::Relaxed),
            "native_tick_latency_ms": {
                "p50": p50,
                "p95": p95,
                "p99": p99,
                "samples": samples
            },
            "directory": {
                "connected_agents": dir.connected_agents,
                "registered_agents": dir.registered_agents,
                "registry_agent_count": dir.registry_agent_count
            }
        })
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

pub(super) struct UnixConnectionBudget {
    active: AtomicUsize,
    general_limit: usize,
    total_limit: usize,
}

impl UnixConnectionBudget {
    pub(super) const fn new(general_limit: usize, reserved: usize) -> Self {
        Self {
            active: AtomicUsize::new(0),
            general_limit,
            total_limit: general_limit + reserved,
        }
    }

    pub(super) fn try_acquire(self: &Arc<Self>, use_reserve: bool) -> Option<UnixConnectionPermit> {
        let limit = if use_reserve {
            self.total_limit
        } else {
            self.general_limit
        };
        let mut current = self.active.load(Ordering::Acquire);
        loop {
            if current >= limit {
                return None;
            }
            match self.active.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Some(UnixConnectionPermit {
                        budget: Arc::clone(self),
                    });
                }
                Err(observed) => current = observed,
            }
        }
    }

    pub(super) fn active_count(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }
}

pub(super) struct UnixConnectionPermit {
    budget: Arc<UnixConnectionBudget>,
}

impl Drop for UnixConnectionPermit {
    fn drop(&mut self) {
        self.budget.active.fetch_sub(1, Ordering::AcqRel);
    }
}

static SHUTDOWN_WAKE_WRITE_FD: AtomicI32 = AtomicI32::new(-1);

pub(super) fn notify_shutdown_wake() {
    let fd = SHUTDOWN_WAKE_WRITE_FD.load(Ordering::Acquire);
    if fd >= 0 {
        let byte: u8 = 1;
        // SAFETY: `write(2)` est async-signal-safe (POSIX signal-safety(7)) ;
        // `&byte` est valide sur la pile pendant l'appel et le descripteur est non bloquant.
        unsafe {
            let _ = libc::write(fd, &byte as *const u8 as *const libc::c_void, 1);
        }
    }
}

pub(super) struct ShutdownWakePipe {
    pub(super) read_fd: std::os::unix::io::RawFd,
    write_fd: std::os::unix::io::RawFd,
}

impl ShutdownWakePipe {
    pub(super) fn open() -> std::io::Result<Self> {
        let (read_fd, write_fd) = bridget_transport::posix::pipe_cloexec_nonblocking()?;
        SHUTDOWN_WAKE_WRITE_FD.store(write_fd, Ordering::Release);
        Ok(Self { read_fd, write_fd })
    }

    pub(super) fn drain(&self) {
        let mut buf = [0u8; 32];
        loop {
            // SAFETY: `self.read_fd` est un descripteur de pipe non bloquant ouvert et `buf` est valide.
            let n = unsafe {
                libc::read(
                    self.read_fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                )
            };
            if n <= 0 {
                break;
            }
        }
    }
}

impl Drop for ShutdownWakePipe {
    fn drop(&mut self) {
        let _ = SHUTDOWN_WAKE_WRITE_FD.compare_exchange(
            self.write_fd,
            -1,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        // SAFETY: les deux descripteurs appartiennent exclusivement à `ShutdownWakePipe`.
        unsafe {
            libc::close(self.read_fd);
            libc::close(self.write_fd);
        }
    }
}

pub(super) fn poll_accept_or_shutdown(
    listener_fd: std::os::unix::io::RawFd,
    wake_fd: std::os::unix::io::RawFd,
    timeout_ms: libc::c_int,
) {
    let mut fds = [
        libc::pollfd {
            fd: listener_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: wake_fd,
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    let _ = bridget_transport::posix::poll_fds(&mut fds, timeout_ms);
}
