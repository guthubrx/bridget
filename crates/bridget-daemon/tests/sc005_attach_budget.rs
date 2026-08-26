use bridget_core::BridgetMessage;
use bridget_daemon::daemon::{self, DaemonConfig};
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::wrapper::launch_acp_with;
use bridget_transport::journal::{AppendLatencyProbe, current_host_date};
use bridget_transport::protocol::{AgentInfo, AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const WARMUP_TURNS: usize = 100;
const MEASURED_TURNS: usize = 1_000;
/// Bornes échantillonnées par `AppendLatencyProbe` (`turn_start` | `turn_end`
/// uniquement — `is_turn_boundary`). Distinct du cardinal journal/attach.
const SAMPLED_BOUNDARIES_PER_TURN: usize = 2;
/// Événements journal/attach par tour ACP après L4 : start + reasoning + end.
const JOURNAL_EVENTS_PER_TURN: usize = 3;
const GLOBAL_TIMEOUT: Duration = Duration::from_secs(60);
const SC001_TURNS: usize = 600;
const SC001_CADENCE: Duration = Duration::from_millis(100);
const SC001_MEASURED_CAMPAIGNS: usize = 21;
const SC005_INTERNAL_PAIRS: usize = 5;

/// Les deux bancs de latence mesurent des délais de quelques microsecondes :
/// ils doivent donc s'exclure mutuellement dans le même binaire de test.
static LATENCY_BENCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_latency_bench() -> std::sync::MutexGuard<'static, ()> {
    LATENCY_BENCH_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn unique_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-sc005-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn daemon_config(root: &Path) -> DaemonConfig {
    DaemonConfig {
        socket_path: PathBuf::from(format!(
            "/tmp/bg-sc5-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        )),
        db_path: root.join("bridget.db"),
        log_path: root.join("daemon.log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 10_000,
        dedup_window: 180,
        quarantine_window: 3_600,
        retention_days: 7,
    }
}

fn wait_until(deadline: Instant, detail: &str, predicate: impl Fn() -> bool) {
    while !predicate() {
        assert!(Instant::now() < deadline, "timeout global SC-005: {detail}");
        thread::sleep(Duration::from_millis(1));
    }
}

fn write_message(writer: &mut BufWriter<UnixStream>, message: &WrapperToDaemon) {
    writeln!(writer, "{}", encode(message).expect("message sérialisable"))
        .expect("écriture socket");
    writer.flush().expect("flush socket");
}

fn read_message(reader: &mut impl BufRead) -> DaemonToWrapper {
    let mut line = String::new();
    reader.read_line(&mut line).expect("lecture socket");
    assert!(!line.is_empty(), "EOF daemon inattendu");
    decode(line.trim()).expect("frame daemon valide")
}

fn query_agent_list(socket: &Path) -> Option<Vec<AgentInfo>> {
    let stream = UnixStream::connect(socket).ok()?;
    let reader_stream = stream.try_clone().ok()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(reader_stream);
    write_message(&mut writer, &WrapperToDaemon::ListAgents);
    match read_message(&mut reader) {
        DaemonToWrapper::AgentList { agents } => Some(agents),
        other => panic!("ListAgents inattendu: {other:?}"),
    }
}

fn agent_state(socket: &Path, name: &str) -> Option<String> {
    query_agent_list(socket)?
        .into_iter()
        .find(|agent| agent.name == name)
        .map(|agent| agent.state)
}

fn agent_ready_for_send(socket: &Path, name: &str) -> bool {
    query_agent_list(socket).into_iter().flatten().any(|agent| {
        agent.name == name && agent.state == "connected" && !agent.connection_id.is_empty()
    })
}

fn wait_for_agent_ready_for_send(socket: &Path, name: &str, deadline: Instant) {
    wait_until(
        deadline,
        &format!("équipier {name} présent mais pas prêt pour Send (attendu connected + route)"),
        || agent_ready_for_send(socket, name),
    );
}

fn expect_send_ack(reader: &mut impl BufRead, socket: &Path, agent: &str, turn: &str) {
    match read_message(reader) {
        DaemonToWrapper::Ack { .. } => {}
        other => {
            // Trame d'abord : si ListAgents échoue, le non-Ack reste lisible.
            let frame = format!("{other:?}");
            let state = agent_state(socket, agent);
            panic!("Send {turn} (agent={state:?}): accusé Ack attendu, reçu {frame}");
        }
    }
}

struct AttachViewConsumer {
    final_fragments: Arc<AtomicUsize>,
    caught_up: Arc<AtomicUsize>,
    final_sequences: Arc<Mutex<Vec<u64>>>,
    rendered_at: Arc<Mutex<HashMap<u64, Instant>>>,
    handle: thread::JoinHandle<Vec<String>>,
}

fn connect_attach(socket: &Path, agent: &str) -> AttachViewConsumer {
    let stream = UnixStream::connect(socket).expect("connexion attach");
    let reader_stream = stream.try_clone().expect("clone lecteur attach");
    reader_stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout lecteur attach");
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(reader_stream);
    write_message(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
    );
    match read_message(&mut reader) {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach,
        } => {}
        other => panic!("RoleHandshake attach inattendu: {other:?}"),
    }
    write_message(
        &mut writer,
        &WrapperToDaemon::Subscribe {
            agent: agent.to_string(),
            window: AttachWindow::Seq(0),
        },
    );
    let subscription_id = match read_message(&mut reader) {
        DaemonToWrapper::Subscribed { subscription_id } => subscription_id,
        other => panic!("abonnement attach refusé ou inattendu: {other:?}"),
    };
    let final_fragments = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&final_fragments);
    let caught_up = Arc::new(AtomicUsize::new(0));
    let observed_caught_up = Arc::clone(&caught_up);
    let final_sequences = Arc::new(Mutex::new(Vec::new()));
    let observed_sequences = Arc::clone(&final_sequences);
    let rendered_at = Arc::new(Mutex::new(HashMap::new()));
    let observed_rendered_at = Arc::clone(&rendered_at);
    let handle = thread::spawn(move || {
        let _writer_kept_alive = writer;
        let mut diagnostics = Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => match decode::<DaemonToWrapper>(line.trim()) {
                    Ok(DaemonToWrapper::JournalFragment {
                        subscription_id: received,
                        seq,
                        final_fragment,
                        ..
                    }) if received == subscription_id => {
                        if final_fragment {
                            observed.fetch_add(1, Ordering::SeqCst);
                            observed_sequences
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner())
                                .push(seq);
                            observed_rendered_at
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner())
                                .entry(seq)
                                .or_insert_with(Instant::now);
                        }
                    }
                    Ok(DaemonToWrapper::SnapshotCaughtUp {
                        subscription_id: received,
                        ..
                    }) if received == subscription_id => {
                        observed_caught_up.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(DaemonToWrapper::Gap { reason, .. }) => diagnostics.push(format!(
                        "Gap inattendu: {}",
                        reason.unwrap_or_else(|| "sans motif".to_string())
                    )),
                    Ok(DaemonToWrapper::JournalReadError { reason, .. }) => {
                        diagnostics.push(format!("journal illisible: {reason}"));
                    }
                    Ok(DaemonToWrapper::End { .. }) => break,
                    Ok(_) => {}
                    Err(error) => diagnostics.push(format!("frame invalide: {error}")),
                },
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => {
                    diagnostics.push(format!("lecture attach: {error}"));
                    break;
                }
            }
        }
        diagnostics
    });
    AttachViewConsumer {
        final_fragments,
        caught_up,
        final_sequences,
        rendered_at,
        handle,
    }
}

fn connect_sender(socket: &Path) -> (BufWriter<UnixStream>, BufReader<UnixStream>) {
    let stream = UnixStream::connect(socket).expect("connexion émetteur");
    let reader_stream = stream.try_clone().expect("clone émetteur");
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(reader_stream);
    write_message(
        &mut writer,
        &WrapperToDaemon::Register {
            agent_type: "fixture".to_string(),
            name: Some("bench-sender".to_string()),
            host: Some("test-host".to_string()),
            transport: Some("unix".to_string()),
            mode: Some(bridget_transport::protocol::PresenceMode::Cli),
            location: None,
            os: Some("test".to_string()),
            instance_id: Some(format!("sender-{}", uuid::Uuid::new_v4())),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        },
    );
    match read_message(&mut reader) {
        DaemonToWrapper::Registered { name } if name == "bench-sender" => {}
        other => panic!("Register bench-sender inattendu: {other:?}"),
    }
    (writer, reader)
}

fn fake_registry(root: &Path, exit_after: usize) -> AgentRegistry {
    let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session_new
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"bench-session"}}'
id=3
count=0
while read prompt; do
  printf '{"jsonrpc":"2.0","id":%s,"result":{"stopReason":"end_turn"}}\n' "$id"
  id=$((id + 1))
  count=$((count + 1))
  if [ "$count" -eq __EXIT_AFTER__ ]; then
    exit 0
  fi
done
"#
    .replace("__EXIT_AFTER__", &exit_after.to_string());
    let registry = serde_json::json!({
        "agents": {
            "bench": {
                "command": "sh",
                "args": ["-c", script],
                "protocol": "acp",
                "permissions": "allow",
                "queue_capacity": 1,
                "notify_timeout_secs": 2
            }
        }
    });
    AgentRegistry::from_json(&registry.to_string(), root.join("agents.json"))
        .expect("registre de banc valide")
}

fn percentile_95(samples: &[Duration]) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn median(samples: &mut [Duration]) -> Duration {
    assert!(!samples.is_empty(), "médiane sans campagne SC-005");
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn median_delta(samples: &mut [i128]) -> i128 {
    assert!(!samples.is_empty(), "médiane sans delta SC-005");
    samples.sort_unstable();
    samples[samples.len() / 2]
}

struct BenchHarness {
    root: PathBuf,
    socket: PathBuf,
    probe: AppendLatencyProbe,
    planned_turns: usize,
    historical_events: usize,
    sender: BufWriter<UnixStream>,
    sender_reader: BufReader<UnixStream>,
    wrapper_done: mpsc::Receiver<Result<(), String>>,
    wrapper: thread::JoinHandle<()>,
    views: Vec<AttachViewConsumer>,
}

impl BenchHarness {
    fn start(
        label: &str,
        view_count: usize,
        planned_turns: usize,
        historical_events: usize,
        deadline: Instant,
        prepare_journal: impl FnOnce(&Path),
    ) -> Self {
        let root = unique_root(label);
        std::fs::create_dir_all(&root).expect("racine campagne");
        let config = daemon_config(&root);
        let socket = config.socket_path.clone();
        thread::spawn(move || daemon::run(config).expect("daemon SC-005"));
        wait_until(deadline, "socket daemon absente", || socket.exists());

        let journal_root = root.join("home/.cache/bridget/sessions");
        let probe = AppendLatencyProbe::install(&journal_root);
        let registry = fake_registry(&root, planned_turns + 1);
        let wrapper_socket = socket.clone();
        let wrapper_home = root.join("home");
        std::fs::create_dir_all(&wrapper_home).expect("home wrapper");
        prepare_journal(&journal_root.join("codex-bench"));
        let (wrapper_done_tx, wrapper_done) = mpsc::channel();
        let wrapper = thread::spawn(move || {
            let result = launch_acp_with(
                "bench",
                &[],
                Some("codex-bench"),
                &registry,
                &wrapper_socket,
                &wrapper_home,
            )
            .map_err(|error| error.to_string());
            let _ = wrapper_done_tx.send(result);
        });
        wait_for_agent_ready_for_send(&socket, "codex-bench", deadline);
        let (sender, sender_reader) = connect_sender(&socket);
        let views = (0..view_count)
            .map(|_| connect_attach(&socket, "codex-bench"))
            .collect::<Vec<_>>();
        for view in &views {
            wait_until(deadline, "snapshot initial non terminé", || {
                view.caught_up.load(Ordering::SeqCst) >= 1
            });
        }
        wait_for_agent_ready_for_send(&socket, "codex-bench", deadline);
        Self {
            root,
            socket,
            probe,
            planned_turns,
            historical_events,
            sender,
            sender_reader,
            wrapper_done,
            wrapper,
            views,
        }
    }

    fn send_turn(&mut self, turn: usize) {
        let message = BridgetMessage::new(
            "bench-sender",
            "codex-bench",
            format!("tour-déterministe-{turn}"),
        );
        write_message(&mut self.sender, &WrapperToDaemon::Send(message));
        expect_send_ack(
            &mut self.sender_reader,
            &self.socket,
            "codex-bench",
            &format!("tour {turn}"),
        );
    }

    fn wait_for_appends(&self, expected: usize, deadline: Instant) {
        wait_until(deadline, "append du tour absent", || {
            self.probe.sample_count() >= expected
        });
    }

    fn take_samples(&self, expected: usize) -> Vec<Duration> {
        let samples = self.probe.take();
        assert_eq!(samples.len(), expected);
        samples
    }

    fn finish(mut self, deadline: Instant) {
        let expected_view = self.historical_events + self.planned_turns * JOURNAL_EVENTS_PER_TURN;
        for view in &self.views {
            wait_until(deadline, "vue attach en retard en fin de campagne", || {
                view.final_fragments.load(Ordering::SeqCst) >= expected_view
            });
        }
        let final_message = BridgetMessage::new(
            "bench-sender",
            "codex-bench",
            "tour-final-hors-mesure".to_string(),
        );
        write_message(&mut self.sender, &WrapperToDaemon::Send(final_message));
        expect_send_ack(
            &mut self.sender_reader,
            &self.socket,
            "codex-bench",
            "tour final",
        );
        let result = self
            .wrapper_done
            .recv_timeout(Duration::from_secs(5))
            .expect("wrapper non terminé après EOF ACP");
        assert!(result.is_ok(), "wrapper ACP en échec: {result:?}");
        self.wrapper.join().expect("thread wrapper");
        for view in self.views {
            let diagnostics = view.handle.join().expect("thread vue attach");
            assert!(diagnostics.is_empty(), "{diagnostics:?}");
        }
        let _ = std::fs::remove_file(&self.socket);
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn run_interleaved_campaign() -> (Duration, usize, Duration, usize) {
    let deadline = Instant::now() + GLOBAL_TIMEOUT;
    let mut baseline = BenchHarness::start(
        "baseline",
        0,
        WARMUP_TURNS + MEASURED_TURNS,
        0,
        deadline,
        |_| {},
    );
    let mut observed = BenchHarness::start(
        "observed",
        2,
        WARMUP_TURNS + MEASURED_TURNS,
        0,
        deadline,
        |_| {},
    );
    for turn in 0..WARMUP_TURNS + MEASURED_TURNS {
        if turn % 2 == 0 {
            baseline.send_turn(turn);
            observed.send_turn(turn);
        } else {
            observed.send_turn(turn);
            baseline.send_turn(turn);
        }
        let expected = if turn < WARMUP_TURNS {
            (turn + 1) * SAMPLED_BOUNDARIES_PER_TURN
        } else {
            (turn + 1 - WARMUP_TURNS) * SAMPLED_BOUNDARIES_PER_TURN
        };
        baseline.wait_for_appends(expected, deadline);
        observed.wait_for_appends(expected, deadline);
        if turn + 1 == WARMUP_TURNS {
            baseline.take_samples(WARMUP_TURNS * SAMPLED_BOUNDARIES_PER_TURN);
            observed.take_samples(WARMUP_TURNS * SAMPLED_BOUNDARIES_PER_TURN);
        }
    }
    let baseline_samples = baseline.take_samples(MEASURED_TURNS * SAMPLED_BOUNDARIES_PER_TURN);
    let observed_samples = observed.take_samples(MEASURED_TURNS * SAMPLED_BOUNDARIES_PER_TURN);
    let result = (
        percentile_95(&baseline_samples),
        baseline_samples.len(),
        percentile_95(&observed_samples),
        observed_samples.len(),
    );
    baseline.finish(deadline);
    observed.finish(deadline);
    result
}

#[test]
fn sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent() {
    let _lock = lock_latency_bench();
    let mut baselines = Vec::with_capacity(SC005_INTERNAL_PAIRS);
    let mut observed = Vec::with_capacity(SC005_INTERNAL_PAIRS);
    let mut deltas = Vec::with_capacity(SC005_INTERNAL_PAIRS);
    for _ in 0..SC005_INTERNAL_PAIRS {
        let (baseline, baseline_count, with_views, observed_count) = run_interleaved_campaign();
        assert_eq!(baseline_count, MEASURED_TURNS * SAMPLED_BOUNDARIES_PER_TURN);
        assert_eq!(observed_count, MEASURED_TURNS * SAMPLED_BOUNDARIES_PER_TURN);
        baselines.push(baseline);
        observed.push(with_views);
        deltas.push(with_views.as_nanos() as i128 - baseline.as_nanos() as i128);
    }
    let baseline = median(&mut baselines);
    let with_views = median(&mut observed);
    let paired_delta = median_delta(&mut deltas);
    let relative_limit = (baseline.as_nanos() / 20) as i128;
    let limit = relative_limit.max(Duration::from_micros(5).as_nanos() as i128);
    eprintln!(
        "SC-005 p95 appariés de {SC005_INTERNAL_PAIRS} campagnes entrelacées: 0 vue médian={baseline:?} ({baselines:?}), 2 vues médian={with_views:?} ({observed:?}), deltas ns={deltas:?}, médiane delta={paired_delta}ns, limite={limit}ns"
    );
    assert!(
        paired_delta <= limit,
        "delta p95 médian avec 2 vues réelles={paired_delta}ns, p95 sans vue={baseline:?}, limite={limit}ns"
    );
}

/// SC-001 est une campagne locale explicite : le noyau, la charge et les deux
/// vues Unix réelles font varier sa mesure. Les 75 secondes sont seulement le
/// disjoncteur d'une campagne bloquée, jamais un seuil de performance.
#[test]
#[ignore = "mesure locale explicite SC-001 ; voir specs/008-attach/implementation.md"]
fn sc001_append_vers_rendu_attach_reel_reste_sous_les_seuils_locaux() {
    let _lock = lock_latency_bench();
    let mut campaigns = Vec::with_capacity(SC001_MEASURED_CAMPAIGNS);
    for campaign in 0..SC001_MEASURED_CAMPAIGNS {
        campaigns.push(run_sc001_campaign(campaign));
    }

    let p95s = campaigns
        .iter()
        .map(|campaign| campaign.p95)
        .collect::<Vec<_>>();
    let p95 = percentile_95(&p95s);
    let max = campaigns
        .iter()
        .map(|campaign| campaign.max)
        .max()
        .unwrap_or_default();
    let report = serde_json::json!({
        "v": 1,
        "criterion": "SC-001",
        "commit": git_commit(),
        "machine": machine_reference(),
        "system": system_reference(),
        "load_1m": load_average(),
        "charge": "campagne locale explicite ; autres charges à consigner par l'opérateur",
        "campaigns": campaigns.iter().map(|campaign| serde_json::json!({
            "index": campaign.index,
            "p95_ms": duration_ms(campaign.p95),
            "max_ms": duration_ms(campaign.max),
            "raw_latency_ms": campaign.raw_latencies.iter().map(|latency| duration_ms(*latency)).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "p95_of_campaign_p95_ms": duration_ms(p95),
        "max_ms": duration_ms(max),
        "p95_budget_ms": 1_000.0,
        "max_budget_ms": 3_000.0,
        "watchdog_per_campaign_secs": 75,
    });
    eprintln!("SC-001 rapport={report}");
    write_optional_report(&report);
    assert!(p95 < Duration::from_secs(1), "p95 SC-001={p95:?}");
    assert!(max < Duration::from_secs(3), "max SC-001={max:?}");
}

struct Sc001Campaign {
    index: usize,
    raw_latencies: Vec<Duration>,
    p95: Duration,
    max: Duration,
}

fn run_sc001_campaign(index: usize) -> Sc001Campaign {
    let started = Instant::now();
    let deadline = started + Duration::from_secs(75);
    let mut harness = BenchHarness::start(
        &format!("sc001-local-{index}"),
        2,
        SC001_TURNS,
        0,
        deadline,
        |_| {},
    );

    for turn in 0..SC001_TURNS {
        assert!(
            Instant::now() < deadline,
            "SC-001 campagne {index} bloquée pendant l'émission au tour {turn} (watchdog de 75 s)"
        );
        let due = started + SC001_CADENCE * turn as u32;
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            thread::sleep(wait);
        }
        harness.send_turn(turn);
    }

    let expected_samples = SC001_TURNS * SAMPLED_BOUNDARIES_PER_TURN;
    let expected_journal = SC001_TURNS * JOURNAL_EVENTS_PER_TURN;
    harness.wait_for_appends(expected_samples, deadline);
    for view in &harness.views {
        wait_until(deadline, "rendu attach absent après append", || {
            view.final_fragments.load(Ordering::SeqCst) >= expected_journal
        });
    }

    let samples = harness.probe.take_samples();
    assert_eq!(samples.len(), expected_samples, "append incomplet");
    let first_view = &harness.views[0];
    let rendered = first_view
        .rendered_at
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let latencies = samples
        .iter()
        .map(|sample| {
            let rendered_at = rendered
                .get(&sample.seq)
                .copied()
                .unwrap_or_else(|| panic!("seq {} non rendue", sample.seq));
            rendered_at
                .checked_duration_since(sample.completed_at)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    drop(rendered);
    assert_eq!(latencies.len(), expected_samples);
    let p95 = percentile_95(&latencies);
    let max = latencies.iter().copied().max().unwrap_or_default();
    harness.finish(deadline);
    Sc001Campaign {
        index,
        raw_latencies: latencies,
        p95,
        max,
    }
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn git_commit() -> String {
    command_output("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "inconnu".to_string())
}

fn machine_reference() -> String {
    command_output("sysctl", &["-n", "hw.model"])
        .or_else(|| command_output("uname", &["-m"]))
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}

fn system_reference() -> String {
    command_output("uname", &["-sr"]).unwrap_or_else(|| std::env::consts::OS.to_string())
}

fn command_output(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn load_average() -> Option<f64> {
    let mut values = [0.0_f64; 3];
    // Valeur informative de rapport uniquement, jamais un seuil de ce test.
    (unsafe { libc::getloadavg(values.as_mut_ptr(), 3) } > 0).then_some(values[0])
}

fn write_optional_report(report: &serde_json::Value) {
    let Some(path) = std::env::var_os("BRIDGET_PERF_REPORT") else {
        return;
    };
    std::fs::write(
        path,
        serde_json::to_vec_pretty(report).expect("rapport JSON"),
    )
    .expect("écriture référence locale SC-001");
}

fn previous_host_date() -> String {
    let today = current_host_date();
    let mut parts = today
        .split('-')
        .map(|part| part.parse::<u32>().expect("date hôte"));
    let mut year = parts.next().expect("année");
    let mut month = parts.next().expect("mois");
    let mut day = parts.next().expect("jour");
    if day > 1 {
        day -= 1;
    } else {
        if month == 1 {
            year -= 1;
            month = 12;
        } else {
            month -= 1;
        }
        day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
            2 => 28,
            _ => unreachable!("mois calendaire"),
        };
    }
    format!("{year:04}-{month:02}-{day:02}")
}

#[test]
fn sc002_rejeu_vers_suivi_traverse_la_rotation_sans_perte_ni_doublon() {
    let deadline = Instant::now() + Duration::from_secs(15);
    let previous_date = previous_host_date();
    let mut harness = BenchHarness::start(
        "sc002-rotation",
        1,
        1,
        1,
        deadline,
        move |journal_directory| {
            std::fs::create_dir_all(journal_directory).expect("répertoire historique");
            std::fs::write(
                journal_directory.join(format!("{previous_date}.jsonl")),
                b"{\"v\":1,\"seq\":5}\n",
            )
            .expect("événement avant minuit");
        },
    );
    let final_fragments = Arc::clone(&harness.views[0].final_fragments);
    let final_sequences = Arc::clone(&harness.views[0].final_sequences);
    let caught_up = Arc::clone(&harness.views[0].caught_up);
    wait_until(deadline, "rejeu historique absent", || {
        final_fragments.load(Ordering::SeqCst) >= 1
    });
    wait_until(
        deadline,
        "SnapshotCaughtUp absent avant le suivi live",
        || caught_up.load(Ordering::SeqCst) == 1,
    );
    assert_eq!(
        caught_up.load(Ordering::SeqCst),
        1,
        "SnapshotCaughtUp ne doit être émis qu'une fois"
    );
    assert_eq!(
        final_sequences
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .as_slice(),
        [5]
    );

    harness.send_turn(0);
    wait_until(deadline, "suivi live absent après rotation", || {
        final_fragments.load(Ordering::SeqCst) >= 1 + JOURNAL_EVENTS_PER_TURN
    });
    let seqs = final_sequences
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone();
    assert_eq!(
        seqs,
        vec![5, 6, 7, 8],
        "continuité rejeu→suivi (hist + start+reasoning+end)"
    );
    let current_file = harness
        .root
        .join("home/.cache/bridget/sessions/codex-bench")
        .join(format!("{}.jsonl", current_host_date()));
    assert!(
        current_file.exists(),
        "rotation vers le fichier courant absente"
    );
    harness.finish(deadline);
}

/// Contrôle d'instrument : un non-Ack doit apparaître dans le message de panique,
/// pas seulement « matches! a échoué ».
#[test]
fn sc005_diagnostic_non_ack_affiche_la_trame() {
    let frame = encode(&DaemonToWrapper::Nack {
        id: "probe-id".to_string(),
        reason: "cible introuvable".to_string(),
    })
    .expect("Nack sérialisable");
    let mut reader = BufReader::new(std::io::Cursor::new(format!("{frame}\n")));
    let socket = PathBuf::from("/tmp/bridget-sc005-diagnostic-absent.sock");
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        expect_send_ack(&mut reader, &socket, "codex-bench", "diagnostic");
    }));
    let message = caught.expect_err("le non-Ack doit paniquer");
    let text = message
        .downcast_ref::<String>()
        .map(|s| s.as_str())
        .or_else(|| message.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        text.contains("accusé Ack attendu")
            && text.contains("Nack")
            && text.contains("cible introuvable"),
        "diagnostic muet ou incomplet: {text:?}"
    );
}
