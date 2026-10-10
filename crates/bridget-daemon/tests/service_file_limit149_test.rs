//! Correctif 149 : le daemon réel démarre avec une limite de descripteurs
//! héritée trop basse (soft 256, défaut launchd) et doit la relever tout seul.
//!
//! Chaque cas lance le vrai binaire `bridget daemon` dans un enfant isolé dont
//! la limite est imposée entre `fork` et `exec` : le processus du test ne
//! change jamais sa propre limite. Le nombre de descripteurs ouverts par le
//! daemon est mesuré de l'extérieur avec `lsof`. Un daemon à 256 ne peut pas en
//! tenir plus de 256 : dépasser ce nombre prouve que la limite a été relevée.
#[path = "support/idempotent.rs"]
pub mod fixture;

use fixture::*;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Chaque connexion cliente coûte 3 descripteurs au daemon (socket + 2 clones).
/// 120 clients pendants font donc 360 descripteurs, au-delà de 256.
const CLIENTS: usize = 120;
const DEFAULT_SOFT: libc::rlim_t = 256;
const LOG_MARK: &str = "service RLIMIT_NOFILE: soft=";

/// Les cas ouvrent beaucoup de descripteurs dans le processus du test :
/// ils passent l'un après l'autre.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|error| error.into_inner())
}

fn current_limits() -> libc::rlimit {
    let mut limits = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    assert_eq!(
        unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) },
        0,
        "getrlimit du test"
    );
    limits
}

/// Impose soft (et hard si donné) dans l'enfant seulement, après le fork.
fn impose_limits(command: &mut Command, soft: libc::rlim_t, hard: Option<libc::rlim_t>) {
    unsafe {
        command.pre_exec(move || {
            let mut limits = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            limits.rlim_cur = soft;
            if let Some(hard) = hard {
                limits.rlim_max = hard;
            }
            if libc::setrlimit(libc::RLIMIT_NOFILE, &limits) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

struct Daemon {
    child: Child,
    logs: Arc<Mutex<Vec<String>>>,
}

impl Daemon {
    fn start(root: &Path, soft: libc::rlim_t, hard: Option<libc::rlim_t>) -> Self {
        let mut command = isolated_command(root);
        command
            .arg("daemon")
            .env("RUST_LOG", "info")
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        impose_limits(&mut command, soft, hard);
        let mut child = command.spawn().expect("daemon réel");
        track(&child);
        let stderr = child.stderr.take().expect("stderr du daemon");
        let logs = Arc::new(Mutex::new(Vec::new()));
        let sink = logs.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                sink.lock().unwrap().push(line);
            }
        });
        let daemon = Self { child, logs };
        daemon.wait_log("daemon écoute");
        daemon
    }

    fn lines(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }

    fn wait_log(&self, needle: &str) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if self.lines().iter().any(|line| line.contains(needle)) {
                return;
            }
            thread::sleep(Duration::from_millis(20)); // attente d'un journal, pas une barrière métier
        }
        panic!("journal « {needle} » absent: {:#?}", self.lines());
    }

    /// Ligne de journal INFO du helper : « soft=S, hard=H, budget=B ».
    fn reported_limits(&self) -> (u64, u64) {
        let lines = self.lines();
        let line = lines
            .iter()
            .find(|line| line.contains(LOG_MARK))
            .unwrap_or_else(|| panic!("ligne RLIMIT_NOFILE absente: {lines:#?}"));
        let tail = &line[line.find(LOG_MARK).unwrap() + LOG_MARK.len()..];
        let number = |prefix: &str, text: &str| -> u64 {
            let start = text
                .find(prefix)
                .unwrap_or_else(|| panic!("{prefix} dans {line}"))
                + prefix.len();
            text[start..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .unwrap_or_else(|_| panic!("nombre après {prefix} dans {line}"))
        };
        assert!(tail.contains("budget=4096"), "budget annoncé: {line}");
        (number("", tail), number("hard=", tail))
    }

    fn warned_restrictive_hard(&self) -> bool {
        self.lines()
            .iter()
            .any(|line| line.contains("WARN") && line.contains("RLIMIT_NOFILE"))
    }

    /// Descripteurs réellement ouverts par le daemon, vus par le système.
    fn open_descriptors(&self) -> usize {
        let output = Command::new("/usr/sbin/lsof")
            .args(["-n", "-P", "-Ff", "-p", &self.child.id().to_string()])
            .output()
            .expect("lsof");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| {
                line.strip_prefix('f').is_some_and(|name| {
                    name.chars().all(|c| c.is_ascii_digit()) && !name.is_empty()
                })
            })
            .count()
    }

    fn wait_descriptors_at_least(&self, wanted: usize) -> usize {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let count = self.open_descriptors();
            if count >= wanted || Instant::now() >= deadline {
                return count;
            }
            thread::sleep(Duration::from_millis(50)); // le daemon crée ses clones après accept
        }
    }
}

/// Arrêt propre uniquement : SIGTERM sur le groupe du daemon identifié comme
/// enfant direct du test, puis attente bornée. Aucun SIGKILL, même en cas
/// d'échec : l'échec est rapporté et le processus reste visible.
impl Drop for Daemon {
    fn drop(&mut self) {
        let pid = self.child.id() as i32;
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            untrack(&self.child);
            return;
        }
        if is_owned_running_process(pid) {
            unsafe {
                libc::kill(-pid, libc::SIGTERM);
            }
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                untrack(&self.child);
                return;
            }
            thread::sleep(Duration::from_millis(20)); // arrêt borné du daemon
        }
        eprintln!("daemon {pid} toujours vivant 5 s après SIGTERM : non forcé");
    }
}

fn connect_clients(root: &Path) -> Vec<UnixStream> {
    (0..CLIENTS)
        .map(|index| {
            UnixStream::connect(socket(root))
                .unwrap_or_else(|error| panic!("client {index}/{CLIENTS} refusé: {error}"))
        })
        .collect()
}

fn assert_daemon_answers(root: &Path, held: &[UnixStream]) {
    // La CLI réelle interroge le daemon pendant que les clients restent pendants.
    let output = run_isolated(root, &["status"], false);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("Daemon: en ligne"),
        "daemon sans réponse avec {} clients: {stdout}{}",
        held.len(),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Contrôle du banc : l'enfant démarre vraiment avec la limite imposée.
#[test]
fn harness_imposes_the_initial_limit() {
    let _serial = serial();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "ulimit -Sn"])
        .env_clear()
        .stdout(Stdio::piped());
    impose_limits(&mut command, DEFAULT_SOFT, None);
    let output = command.output().expect("sh");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "256");
}

/// Soft 256 hérité, hard confortable : le daemon monte à 4096 sans toucher au hard.
#[test]
fn daemon_started_at_256_raises_soft_to_4096_and_holds_over_256_descriptors() {
    let _serial = serial();
    let initial = current_limits();
    assert!(
        initial.rlim_max >= 4096,
        "hard du test {} < 4096 : cas non vérifiable ici",
        initial.rlim_max
    );
    let root = test_root("fd149-raise");
    let daemon = Daemon::start(&root, DEFAULT_SOFT, None);

    let (soft, hard) = daemon.reported_limits();
    assert_eq!(soft, 4096, "soft relevée au budget");
    assert_eq!(hard, initial.rlim_max as u64, "hard inchangée");
    assert!(
        !daemon.warned_restrictive_hard(),
        "aucun avertissement attendu"
    );

    let held = connect_clients(&root);
    let count = daemon.wait_descriptors_at_least(CLIENTS * 3);
    assert!(
        count > DEFAULT_SOFT as usize,
        "le daemon ne tient que {count} descripteurs (<= 256) avec {CLIENTS} clients"
    );
    assert_daemon_answers(&root, &held);
}

/// Soft déjà à 8192 : le helper ne la baisse jamais vers 4096.
#[test]
fn daemon_started_at_8192_keeps_its_soft_limit() {
    let _serial = serial();
    let initial = current_limits();
    assert!(
        initial.rlim_max >= 8192,
        "hard du test {} < 8192 : cas non vérifiable ici",
        initial.rlim_max
    );
    let root = test_root("fd149-keep");
    let daemon = Daemon::start(&root, 8192, None);

    let (soft, hard) = daemon.reported_limits();
    assert_eq!(soft, 8192, "soft supérieure au budget conservée");
    assert_eq!(hard, initial.rlim_max as u64, "hard inchangée");
    assert!(!daemon.warned_restrictive_hard());
}

/// Hard restrictif à 512 : soft relevée jusqu'à 512 seulement, hard intacte, avertissement.
#[test]
fn daemon_with_hard_512_is_clamped_to_hard_and_warns() {
    let _serial = serial();
    let root = test_root("fd149-hard512");
    let daemon = Daemon::start(&root, DEFAULT_SOFT, Some(512));

    let (soft, hard) = daemon.reported_limits();
    assert_eq!(
        (soft, hard),
        (512, 512),
        "soft plafonnée au hard, hard intacte"
    );
    assert!(
        daemon.warned_restrictive_hard(),
        "avertissement de hard restrictif"
    );

    let held = connect_clients(&root);
    let count = daemon.wait_descriptors_at_least(CLIENTS * 3);
    assert!(
        (DEFAULT_SOFT as usize + 1..=512).contains(&count),
        "descripteurs du daemon {count} hors de ]256, 512]"
    );
    assert_daemon_answers(&root, &held);
}
