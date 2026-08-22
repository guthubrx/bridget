use bridget_core::BridgetMessage;
use bridget_daemon::daemon::{self, DaemonConfig};
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::wrapper::launch_acp_with;
use bridget_transport::journal::AppendLatencyProbe;
use bridget_transport::protocol::{AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const WARMUP_TURNS: usize = 20;
const MEASURED_TURNS: usize = 200;
const EVENTS_PER_TURN: usize = 2;
const GLOBAL_TIMEOUT: Duration = Duration::from_secs(30);

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
        circuit_breaker_limit: 1_000,
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

fn read_message(reader: &mut BufReader<UnixStream>) -> DaemonToWrapper {
    let mut line = String::new();
    reader.read_line(&mut line).expect("lecture socket");
    assert!(!line.is_empty(), "EOF daemon inattendu");
    decode(line.trim()).expect("frame daemon valide")
}

fn wait_for_agent(socket: &Path, name: &str, deadline: Instant) {
    wait_until(deadline, "équipier ACP non enregistré", || {
        let Ok(stream) = UnixStream::connect(socket) else {
            return false;
        };
        let Ok(reader_stream) = stream.try_clone() else {
            return false;
        };
        let mut writer = BufWriter::new(stream);
        let mut reader = BufReader::new(reader_stream);
        write_message(&mut writer, &WrapperToDaemon::ListAgents);
        matches!(
            read_message(&mut reader),
            DaemonToWrapper::AgentList { agents }
                if agents.iter().any(|agent| agent.name == name)
        )
    });
}

struct AttachViewConsumer {
    final_fragments: Arc<AtomicUsize>,
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
    assert!(matches!(
        read_message(&mut reader),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        }
    ));
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
                        final_fragment,
                        ..
                    }) if received == subscription_id => {
                        if final_fragment {
                            observed.fetch_add(1, Ordering::SeqCst);
                        }
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
            os: Some("test".to_string()),
            instance_id: Some(format!("sender-{}", uuid::Uuid::new_v4())),
            domain: None,
            turn_in_progress: false,
        },
    );
    assert!(matches!(
        read_message(&mut reader),
        DaemonToWrapper::Registered { name } if name == "bench-sender"
    ));
    (writer, reader)
}

fn fake_registry(root: &Path) -> AgentRegistry {
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
  if [ "$count" -eq 441 ]; then
    exit 0
  fi
done
"#;
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

fn exercise_phase(
    sender: &mut BufWriter<UnixStream>,
    sender_reader: &mut BufReader<UnixStream>,
    probe: &AppendLatencyProbe,
    views: &[AttachViewConsumer],
    first_turn: usize,
    initial_view_events: usize,
    deadline: Instant,
) -> (Duration, usize) {
    for phase_turn in 0..WARMUP_TURNS + MEASURED_TURNS {
        let turn = first_turn + phase_turn;
        let message = BridgetMessage::new(
            "bench-sender",
            "codex-bench",
            format!("tour-déterministe-{turn}"),
        );
        write_message(sender, &WrapperToDaemon::Send(message));
        assert!(matches!(
            read_message(sender_reader),
            DaemonToWrapper::Ack { .. }
        ));
        let expected_probe = if phase_turn < WARMUP_TURNS {
            (phase_turn + 1) * EVENTS_PER_TURN
        } else {
            (phase_turn + 1 - WARMUP_TURNS) * EVENTS_PER_TURN
        };
        wait_until(deadline, "append du tour absent", || {
            probe.sample_count() >= expected_probe
        });
        if phase_turn + 1 == WARMUP_TURNS {
            assert_eq!(probe.take().len(), WARMUP_TURNS * EVENTS_PER_TURN);
        }
    }
    let samples = probe.take();
    assert_eq!(samples.len(), MEASURED_TURNS * EVENTS_PER_TURN);
    let expected_view = initial_view_events + (WARMUP_TURNS + MEASURED_TURNS) * EVENTS_PER_TURN;
    for view in views {
        wait_until(deadline, "vue attach en retard en fin de campagne", || {
            view.final_fragments.load(Ordering::SeqCst) >= expected_view
        });
    }
    (percentile_95(&samples), samples.len())
}

fn run_paired_campaign() -> (Duration, usize, Duration, usize) {
    let root = unique_root("paired");
    std::fs::create_dir_all(&root).expect("racine campagne");
    let config = daemon_config(&root);
    let socket = config.socket_path.clone();
    thread::spawn(move || daemon::run(config).expect("daemon SC-005"));
    let deadline = Instant::now() + GLOBAL_TIMEOUT;
    wait_until(deadline, "socket daemon absente", || socket.exists());

    let journal_root = root.join("home/.cache/bridget/sessions");
    let probe = AppendLatencyProbe::install(&journal_root);
    let registry = fake_registry(&root);
    let wrapper_socket = socket.clone();
    let wrapper_home = root.join("home");
    std::fs::create_dir_all(&wrapper_home).expect("home wrapper");
    let (wrapper_done_tx, wrapper_done_rx) = mpsc::channel();
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
    wait_for_agent(&socket, "codex-bench", deadline);
    let (mut sender, mut sender_reader) = connect_sender(&socket);
    let (baseline, baseline_count) =
        exercise_phase(&mut sender, &mut sender_reader, &probe, &[], 0, 0, deadline);
    let views = (0..2)
        .map(|_| connect_attach(&socket, "codex-bench"))
        .collect::<Vec<_>>();
    let replayed_events = (WARMUP_TURNS + MEASURED_TURNS) * EVENTS_PER_TURN;
    for view in &views {
        wait_until(deadline, "rejeu initial de la vue incomplet", || {
            view.final_fragments.load(Ordering::SeqCst) >= replayed_events
        });
    }
    let (observed, observed_count) = exercise_phase(
        &mut sender,
        &mut sender_reader,
        &probe,
        &views,
        WARMUP_TURNS + MEASURED_TURNS,
        replayed_events,
        deadline,
    );
    let final_message = BridgetMessage::new(
        "bench-sender",
        "codex-bench",
        "tour-final-hors-mesure".to_string(),
    );
    write_message(&mut sender, &WrapperToDaemon::Send(final_message));
    assert!(matches!(
        read_message(&mut sender_reader),
        DaemonToWrapper::Ack { .. }
    ));
    let result = wrapper_done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("wrapper non terminé après EOF ACP");
    assert!(result.is_ok(), "wrapper ACP en échec: {result:?}");
    wrapper.join().expect("thread wrapper");
    for view in views {
        let diagnostics = view.handle.join().expect("thread vue attach");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
    let _ = std::fs::remove_file(&socket);
    (baseline, baseline_count, observed, observed_count)
}

#[test]
fn sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent() {
    let (baseline, baseline_count, observed, observed_count) = run_paired_campaign();
    eprintln!(
        "SC-005 append p95 réel: 0 vue={baseline:?} ({baseline_count} échantillons), 2 vues={observed:?} ({observed_count} échantillons)"
    );
    assert!(
        observed.as_nanos() * 100 < baseline.as_nanos() * 105,
        "p95 append avec 2 vues réelles={observed:?}, sans vue={baseline:?}"
    );
}
